    use super::{
        ImportScanGeneration, ImportScanNextStep, SavedAsset, create_work_from_text,
        dictionary_inconclusive_event, dictionary_probe_from_grouped,
        filter_and_enqueue_current_import_scan, guarded_dict_layers, guarded_open_store,
        import_scan_next_step, keep_committed_import_when_scan_spawn_fails,
        read_chapter_segment_texts, saved_asset_chapter_index_is_in_range,
        saved_asset_satisfies_asset_check_constraints, swap_locked,
    };
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    /// Thư mục tạm CỦA RIÊNG ca này — pid + `AtomicU64`, cùng luật bốn điều của
    /// `glossary_contract.rs`/`glossary_commands_contract.rs` (mỗi ca một thư mục riêng;
    /// `Store` drop TRƯỚC khi xoá; không `sleep` dài; không ca nào treo khi trượt).
    static NEXT_GUARD_DIR: AtomicU64 = AtomicU64::new(0);

    fn guard_test_dir(tag: &str) -> std::path::PathBuf {
        let n = NEXT_GUARD_DIR.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "auratranslate-project-guard-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
        dir
    }

    fn guard_test_cleanup(dir: &std::path::Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Adapter TEST `bool -> DictionaryProbe`, giữ CỤC BỘ ở bàn test này — 🔵 2026-08-26
    /// (cụm F). `core::glossary::scan::scan_candidates` (vỏ `bool` công khai) đã bị xoá:
    /// nó có 0 chỗ gọi sản phẩm và biến một layer LỖI thành "không có trong từ điển". Đường
    /// sản phẩm thật của `spawn_import_scan` tiêm closure gọi `dictionary_probe_from_grouped`
    /// thẳng vào `scan_candidates_controlled`; ca test dưới đây chỉ cần một vị từ `bool` tất
    /// định nên tự giữ đúng phần thân adapter đã xoá, không phục hồi một API sản phẩm.
    fn scan_candidates_bool_probe(
        segments: &[&str],
        lang: crate::core::matching::MatchLang,
        threshold: u32,
        surnames: &[char],
        is_known: &mut dyn FnMut(&str) -> bool,
    ) -> Vec<crate::core::glossary::ScanCandidate> {
        let mut probe = |term: &str| {
            if is_known(term) {
                crate::core::glossary::DictionaryProbe::Known
            } else {
                crate::core::glossary::DictionaryProbe::Missing
            }
        };
        let mut never_cancelled = || false;
        match crate::core::glossary::scan_candidates_controlled(
            segments,
            lang,
            threshold,
            surnames,
            &mut probe,
            &mut never_cancelled,
        ) {
            crate::core::glossary::ScanOutcome::Completed(out) => out,
            crate::core::glossary::ScanOutcome::DictionaryInconclusive
            | crate::core::glossary::ScanOutcome::Cancelled => Vec::new(),
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════════
    // I/O Matrix — "Kho đóng giữa lượt quét ⇒ luồng nền kết thúc lặng lẽ, không panic",
    // mở rộng cho ca "Tác phẩm đổi" — Story 3.5, rà bảng I/O phát hiện hàng này KHÔNG có
    // test nào canh (`spawn_import_scan` nhận `AppHandle` nên `tests/**` không gọi tới nó
    // được). `guarded_open_store` là đơn vị quyết định được TÁCH RA đúng luật hai lớp của
    // `src-tauri/AGENTS.md` để ba ca dưới đây canh được TRỰC TIẾP, không cần webview/luồng.
    // ═════════════════════════════════════════════════════════════════════════════════

    /// Ca ① — không có Tác phẩm nào đang mở (`OpenWorkState` là `None`, hoặc — như ở đây,
    /// nơi hàm nhận thẳng `Option<&OpenWork>` — chỗ gọi truyền `None`) ⇒ dừng lặng lẽ.
    #[test]
    fn guarded_open_store_returns_none_when_no_work_is_open() {
        assert!(
            guarded_open_store(None, "bat-ky-work-id-nao").is_none(),
            "khong co Tac pham nao dang mo -- phai tra None, khong panic"
        );
    }

    /// Nối trọn hàng Matrix "B thay A": generation B sinh NGAY TRONG pha đếm của A;
    /// hook mà worker thật dùng thấy A stale, scan trả `Cancelled`, lookup = 0 và helper
    /// hậu-scan chọn `Stop` — biến thể duy nhất không enqueue/không phát completion.
    #[test]
    fn a_new_import_generation_cancels_the_old_scan_before_lookup_write_or_completion() {
        let dir = guard_test_dir("generation-cancels-scan");
        let opened = create_work_from_text(&dir, "Generation", "en", "", "source".to_owned())
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));

        let generation = ImportScanGeneration::default();
        let generation_a = generation.next();
        let segments: Vec<String> = (0..500)
            .map(|i| format!("a beast called Fire Dragon appeared at hour {i}."))
            .collect();
        let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
        let mut lookup_calls = 0usize;
        let mut probe = |_term: &str| {
            lookup_calls += 1;
            crate::core::glossary::DictionaryProbe::Missing
        };
        let mut cancellation_checks = 0usize;
        let mut current_generation = || {
            cancellation_checks += 1;
            if cancellation_checks == 3 {
                let _generation_b = generation.next();
            }
            !generation.is_current(generation_a)
        };

        let outcome = crate::core::glossary::scan_candidates_controlled(
            &refs,
            crate::core::matching::MatchLang::En,
            5,
            crate::core::glossary::COMMON_SURNAMES,
            &mut probe,
            &mut current_generation,
        );
        let next = import_scan_next_step(outcome, generation.is_current(generation_a));

        assert_eq!(next, ImportScanNextStep::Stop);
        assert_eq!(lookup_calls, 0, "generation cu phai dung ngay trong count");
        let pending = crate::core::glossary::pending_candidates(&opened.store)
            .expect("doc bang cho doi chung 0 write");
        assert!(pending.is_empty(), "Stop khong duoc xep bat ky batch nao");

        drop(opened.store);
        guard_test_cleanup(&dir);
    }

    /// Lái chính mapping mà worker gọi bằng một `GroupedLookup` có layer lỗi thật về MẶT
    /// kiểu dữ liệu. Outcome phải là `dictionary_inconclusive`; next-step không mang batch,
    /// và bảng Work vẫn rỗng — không chỉ kiểm một predicate thuần tách rời.
    #[test]
    fn a_skipped_dictionary_layer_maps_through_the_worker_decision_to_zero_batch_writes() {
        let dir = guard_test_dir("dictionary-inconclusive");
        let opened =
            create_work_from_text(&dir, "Dict Inconclusive", "en", "", "source".to_owned())
                .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        let grouped = crate::core::dict::GroupedLookup {
            route: crate::core::dict::QueryRoute::En,
            branch: crate::core::dict::QueryBranch::ExactBtree,
            groups: Vec::new(),
            skipped: vec![crate::core::dict::SkippedLayer {
                path: std::path::PathBuf::from("broken-layer.db"),
                reason: crate::core::dict::SkipReason::OpenFailed {
                    detail: "fixture open failure".to_owned(),
                },
            }],
            truncated_layers: Vec::new(),
            // `skipped` phải thắng cả bằng chứng hit bị cắt trang: một layer hỏng làm
            // toàn lượt không kết luận, không cho hit ở layer khác che mất lỗi.
            hidden_sources: vec![("hidden source".to_owned(), 1)],
            layers_loaded: false,
        };
        let segments: Vec<String> = (0..5)
            .map(|i| format!("a beast called Fire Dragon appeared at hour {i}."))
            .collect();
        let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
        let mut probe = |_term: &str| dictionary_probe_from_grouped(&grouped);
        let mut never_cancelled = || false;

        let outcome = crate::core::glossary::scan_candidates_controlled(
            &refs,
            crate::core::matching::MatchLang::En,
            5,
            crate::core::glossary::COMMON_SURNAMES,
            &mut probe,
            &mut never_cancelled,
        );
        let next = import_scan_next_step(outcome, true);

        assert_eq!(next, ImportScanNextStep::EmitDictionaryInconclusive);
        let pending = crate::core::glossary::pending_candidates(&opened.store)
            .expect("doc bang cho doi chung 0 write");
        assert!(
            pending.is_empty(),
            "dictionary inconclusive khong mang batch de enqueue"
        );

        drop(opened.store);
        guard_test_cleanup(&dir);
    }

    #[test]
    fn a_hidden_source_is_a_known_hit_when_no_dictionary_layer_was_skipped() {
        let grouped = crate::core::dict::GroupedLookup {
            route: crate::core::dict::QueryRoute::En,
            branch: crate::core::dict::QueryBranch::ExactBtree,
            groups: Vec::new(),
            skipped: Vec::new(),
            truncated_layers: vec!["base".to_owned()],
            hidden_sources: vec![("source cut cleanly by limit".to_owned(), 2)],
            layers_loaded: true,
        };

        assert_eq!(
            dictionary_probe_from_grouped(&grouped),
            crate::core::glossary::DictionaryProbe::Known,
            "hidden_sources da chung minh co hit, nen Known thang truncated"
        );
    }

    #[test]
    fn a_truncated_layer_without_a_visible_or_hidden_hit_is_inconclusive_not_missing() {
        let grouped = crate::core::dict::GroupedLookup {
            route: crate::core::dict::QueryRoute::En,
            branch: crate::core::dict::QueryBranch::ExactBtree,
            groups: Vec::new(),
            skipped: Vec::new(),
            truncated_layers: vec!["base".to_owned()],
            hidden_sources: Vec::new(),
            layers_loaded: true,
        };

        assert_eq!(
            dictionary_probe_from_grouped(&grouped),
            crate::core::glossary::DictionaryProbe::Inconclusive,
            "truncated khong du bang chung de ket luan Missing"
        );
    }

    #[test]
    fn the_dictionary_inconclusive_payload_serializes_the_reviewed_outcome_and_zero_counts() {
        let payload = dictionary_inconclusive_event(42);

        assert_eq!(
            serde_json::to_value(payload).expect("serialize payload"),
            serde_json::json!({
                "chapter_id": 42,
                "inserted": 0,
                "skipped": 0,
                "outcome": "dictionary_inconclusive",
            })
        );
    }

    /// `thread::Builder::spawn` lỗi được tiêm thẳng qua seam mà hai wire command dùng.
    /// Import đã commit vẫn đọc được từ SQLite và thư mục không bị đảo ngược/xoá.
    #[test]
    fn a_spawn_failure_after_commit_preserves_the_import_and_returns_normally() {
        let dir = guard_test_dir("spawn-failure-after-commit");
        let opened = create_work_from_text(
            &dir,
            "Spawn Failure",
            "en",
            "",
            "a committed source sentence.".to_owned(),
        )
        .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        let project_dir = opened.dir.clone();
        let chapter_id = opened.chapter_id;

        let opened = keep_committed_import_when_scan_spawn_fails(opened, || {
            Err(std::io::Error::other("injected thread spawn failure"))
        });

        let rows = read_chapter_segment_texts(&opened.store, chapter_id)
            .expect("import da commit phai con doc duoc sau spawn Err");
        assert_eq!(rows, vec!["a committed source sentence."]);
        assert!(project_dir.join("project.db").is_file());
        // 🔵 SỬA (2026-08-28, Story 5.5) — dùng `WorkMeta::path_in` thay vì chuỗi `"meta.json"`
        // viết thẳng (hay nhắc thẳng `META_FILE`): `meta_write_boundary.rs` khoá cả hai hình
        // dạng đó CHỈ ở `core/library/meta.rs`, và một bản chép tay ở đây là đúng thứ cổng đó
        // tồn tại để bắt.
        assert!(crate::core::library::meta::WorkMeta::path_in(&project_dir).is_file());

        drop(opened.store);
        guard_test_cleanup(&dir);
    }

    /// Ca ② — `work_id` đã đổi giữa hai lần khoá (Tác phẩm CŨ đã bị thay bằng một Tác
    /// phẩm KHÁC trong `OpenWorkState`) ⇒ dừng lặng lẽ, **0 ghi**. Đối chứng bằng `SELECT`
    /// qua `pending_candidates` — không chỉ tin giá trị trả về `None` — bằng cách lái qua
    /// ĐÚNG hình dạng mà `spawn_import_scan` dùng: chỉ ghi khi `guarded_open_store` trả
    /// `Some`. Nếu vệ bảo vệ bị gỡ (hoặc hỏng), ứng viên GIẢ ở dưới sẽ lọt vào bảng chờ và
    /// ca này đỏ.
    #[test]
    fn guarded_open_store_returns_none_and_blocks_every_write_when_the_work_id_has_changed_mid_scan()
    {
        let dir = guard_test_dir("work-id-changed");
        let opened = create_work_from_text(
            &dir,
            "Doi Tac Pham Giua Chung",
            "zh",
            "",
            "萧炎登场".to_owned(),
        )
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));

        // `work_id` CHỐT LÚC SPAWN không còn khớp `opened.meta.work_id` -- mô phỏng đúng
        // ca "Tác phẩm đổi giữa hai lần khoá": `OpenWorkState` nay trỏ một Tác phẩm KHÁC.
        let stale_work_id = "khong-con-la-tac-pham-nay";
        assert_ne!(
            opened.meta.work_id, stale_work_id,
            "fixture phai thuc su lech work_id"
        );

        let fake_candidates = vec![crate::core::glossary::ScanCandidate {
            source_term: "萧炎".to_owned(),
            occurrence_count: 99,
            context_example: "cau gia.".to_owned(),
        }];

        // Đúng khuôn production ở `spawn_import_scan`: chỉ ghi khi `guarded_open_store`
        // trả `Some`.
        if let Some(store) = guarded_open_store(Some(&opened), stale_work_id) {
            let _ = crate::core::glossary::insert_import_scan_candidates(store, &fake_candidates);
        }

        let pending = crate::core::glossary::pending_candidates(&opened.store)
            .expect("doc bang cho de doi chung -- day la ve SELECT, khong chi tin gia tri tra ve");
        assert!(
            pending.is_empty(),
            "work_id lech ⇒ 0 hang duoc phep ghi vao bang cho ung vien. Nhan: {pending:?}"
        );

        drop(opened.store);
        guard_test_cleanup(&dir);
    }

    /// Ca ③ — ca THƯỜNG: `work_id` khớp ở CẢ HAI lần khoá ⇒ lượt quét chạy hết và ghi.
    /// Đối chứng dương của ca ② — không có nó thì "0 ghi" ở ca ② có thể xanh vì thuật toán
    /// quét/ghi tự nó hỏng, không phải vì vệ bảo vệ đúng.
    #[test]
    fn guarded_open_store_returns_the_store_and_a_normal_scan_runs_to_completion_and_writes() {
        let dir = guard_test_dir("normal-run");
        let text: String = (0..6).map(|i| format!("萧炎在第{i}章登场")).collect();
        let opened = create_work_from_text(&dir, "Ca Thuong", "zh", "", text)
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        let work_id = opened.meta.work_id.clone();

        // Lần khoá THỨ NHẤT (đọc segment) -- cùng `work_id` đã chốt lúc spawn.
        let store = guarded_open_store(Some(&opened), &work_id)
            .expect("work_id khop o lan khoa thu nhat -- phai tra Some(&store)");
        let segments = read_chapter_segment_texts(store, opened.chapter_id).expect("doc segment");
        let segment_refs: Vec<&str> = segments.iter().map(String::as_str).collect();

        let mut is_known = |_: &str| false;
        let candidates = scan_candidates_bool_probe(
            &segment_refs,
            crate::core::matching::MatchLang::Zh,
            5,
            crate::core::glossary::COMMON_SURNAMES,
            &mut is_known,
        );
        assert!(
            !candidates.is_empty(),
            "van ban mau (6 lan '萧炎') phai sinh it nhat mot ung vien"
        );

        // Lần khoá THỨ HAI (ghi lô) -- cùng `work_id`, đúng hình dạng hai-lần-khoá-ngắn.
        let store_for_write = guarded_open_store(Some(&opened), &work_id)
            .expect("work_id van khop o lan khoa thu hai -- phai tra Some(&store)");
        let (inserted, _skipped) =
            crate::core::glossary::insert_import_scan_candidates(store_for_write, &candidates)
                .expect("ghi lo");
        assert!(inserted > 0, "ca thuong phai ghi duoc it nhat mot hang");

        let pending =
            crate::core::glossary::pending_candidates(&opened.store).expect("doc lai bang cho");
        assert!(
            !pending.is_empty(),
            "bang cho phai co hang sau mot luot quet binh thuong"
        );

        drop(opened.store);
        guard_test_cleanup(&dir);
    }

    /// Một term ở Global phải biến mất TRƯỚC câu `INSERT` Work, nhưng vẫn cộng vào
    /// `skipped`. Đối chứng giữ một term khác để chứng minh lô không bị xoá trắng.
    #[test]
    fn global_and_work_glossary_terms_are_resolved_before_the_batch_and_counted_as_skipped() {
        let dir = guard_test_dir("global-filter");
        let global = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .expect("mo global.db");
        crate::commands::glossary::glossary_add_term(
            Some(&global),
            None,
            crate::core::glossary::GlossaryTier::Global,
            "Fire Dragon",
            Some("Hoa Long"),
            "",
            crate::core::glossary::Category::Other,
        )
        .expect("chen term global");

        let opened = create_work_from_text(&dir, "Loc Hai Tang", "en", "", "source".to_owned())
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        crate::commands::glossary::glossary_add_term(
            Some(&global),
            Some(&opened),
            crate::core::glossary::GlossaryTier::Work,
            "Ice Phoenix",
            Some("Bang Phuong"),
            "",
            crate::core::glossary::Category::Other,
        )
        .expect("chen term Work");
        let work_id = opened.meta.work_id.clone();
        let state = Mutex::new(Some(opened));
        let mut candidates = vec![
            crate::core::glossary::ScanCandidate {
                source_term: "Fire Dragon".to_owned(),
                occurrence_count: 7,
                context_example: "A beast called Fire Dragon arrived.".to_owned(),
            },
            crate::core::glossary::ScanCandidate {
                source_term: "Ice Phoenix".to_owned(),
                occurrence_count: 6,
                context_example: "A beast called Ice Phoenix arrived.".to_owned(),
            },
            crate::core::glossary::ScanCandidate {
                source_term: "Storm Tiger".to_owned(),
                occurrence_count: 5,
                context_example: "A beast called Storm Tiger arrived.".to_owned(),
            },
        ];

        let ticket = filter_and_enqueue_current_import_scan(
            &state,
            &work_id,
            &global,
            &mut candidates,
            &|| true,
        )
        .expect("loc va enqueue")
        .expect("work_id dang mo phai cho phep enqueue");
        let (inserted, skipped) = ticket.wait().expect("writer tra loi");
        assert_eq!((inserted, skipped), (1, 2));

        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pending =
            crate::core::glossary::pending_candidates(&guard.as_ref().expect("work con mo").store)
                .expect("doc bang cho");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].source_term, "Storm Tiger");
        drop(guard);

        let opened = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        drop(opened);
        drop(global);
        guard_test_cleanup(&dir);
    }

    /// Generation đổi đúng SAU khi scope filter đã chạy nhưng TRƯỚC enqueue. Callback là
    /// điểm kiểm chính worker dùng, nên `None` ở đây đồng nghĩa không có write-ticket nào
    /// được tạo; bảng chờ là đối chứng SQL cho vế 0 write.
    #[test]
    fn a_generation_that_turns_stale_after_scope_filtering_creates_no_ticket_and_writes_nothing() {
        let dir = guard_test_dir("late-cancellation-after-scope");
        let global = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .expect("mo global.db");
        crate::commands::glossary::glossary_add_term(
            Some(&global),
            None,
            crate::core::glossary::GlossaryTier::Global,
            "Fire Dragon",
            Some("Hoa Long"),
            "",
            crate::core::glossary::Category::Other,
        )
        .expect("chen term Global de chung minh scope filter da chay");

        let opened = create_work_from_text(&dir, "Late Cancel", "en", "", "source".to_owned())
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        let work_id = opened.meta.work_id.clone();
        let state = Mutex::new(Some(opened));
        let mut candidates = vec![
            crate::core::glossary::ScanCandidate {
                source_term: "Fire Dragon".to_owned(),
                occurrence_count: 5,
                context_example: "A beast called Fire Dragon arrived.".to_owned(),
            },
            crate::core::glossary::ScanCandidate {
                source_term: "Ice Phoenix".to_owned(),
                occurrence_count: 5,
                context_example: "A beast called Ice Phoenix arrived.".to_owned(),
            },
        ];
        let generation = ImportScanGeneration::default();
        let generation_a = generation.next();
        let checks = AtomicUsize::new(0);
        let current = || {
            checks.fetch_add(1, Ordering::Relaxed);
            let _generation_b = generation.next();
            generation.is_current(generation_a)
        };

        let ticket = filter_and_enqueue_current_import_scan(
            &state,
            &work_id,
            &global,
            &mut candidates,
            &current,
        )
        .expect("scope filter thanh cong truoc cancellation");

        assert!(
            ticket.is_none(),
            "stale sau filter khong duoc tao write-ticket"
        );
        assert_eq!(
            checks.load(Ordering::Relaxed),
            1,
            "mot check tat dinh ngay truoc enqueue"
        );
        assert_eq!(
            candidates
                .iter()
                .map(|c| c.source_term.as_str())
                .collect::<Vec<_>>(),
            vec!["Ice Phoenix"],
            "Global term da bi loc, chung minh cancellation xay ra SAU scope filtering"
        );
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pending =
            crate::core::glossary::pending_candidates(&guard.as_ref().expect("work con mo").store)
                .expect("doc bang cho doi chung");
        assert!(pending.is_empty(), "0 ticket phai tuong ung 0 write");
        drop(guard);

        let opened = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        drop(opened);
        drop(global);
        guard_test_cleanup(&dir);
    }

    /// Writer bị chặn bằng kênh tất định (không sleep/timing). Helper phải trả ticket và
    /// `OpenWorkState::try_lock` phải thành công TRƯỚC khi job cản được thả.
    #[test]
    fn a_slow_writer_never_keeps_open_work_state_locked_while_the_ticket_waits() {
        let dir = guard_test_dir("writer-ticket-unlocks-state");
        let global = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .expect("mo global.db");
        let opened = create_work_from_text(&dir, "Writer Cham", "en", "", "source".to_owned())
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        let work_id = opened.meta.work_id.clone();

        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let blocker = opened
            .store
            .write_ticket(move |_tx| {
                let _ = started_tx.send(());
                let _ = release_rx.recv();
                Ok(())
            })
            .expect("xep job can writer");
        started_rx.recv().expect("writer phai vao job can");

        let state = Mutex::new(Some(opened));
        let mut candidates = vec![crate::core::glossary::ScanCandidate {
            source_term: "Fire Dragon".to_owned(),
            occurrence_count: 5,
            context_example: "A beast called Fire Dragon arrived.".to_owned(),
        }];
        let scan_ticket = filter_and_enqueue_current_import_scan(
            &state,
            &work_id,
            &global,
            &mut candidates,
            &|| true,
        )
        .expect("loc va enqueue")
        .expect("work dang mo");

        assert!(
            state.try_lock().is_ok(),
            "ticket da xep sau writer cham nhung OpenWorkState phai duoc nha truoc wait"
        );
        release_tx.send(()).expect("tha writer");
        blocker.wait().expect("job can ket thuc");
        scan_ticket.wait().expect("lo scan ket thuc");

        let opened = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        drop(opened);
        drop(global);
        guard_test_cleanup(&dir);
    }

    // ═════════════════════════════════════════════════════════════════════════════════
    // Rà ba lớp 2026-08-22 — `guarded_dict_layers` KHÔNG được nuốt ca "chưa quản lý" thành
    // ca "rỗng bình thường".
    // ═════════════════════════════════════════════════════════════════════════════════

    /// Bản lỗi (`unwrap_or(&DictLayers::empty())`) coi `None` và `Some(rỗng)` là MỘT — ca
    /// này canh đúng vế bị nuốt: `None` (chưa quản lý) phải LAN RA `None`, không âm thầm
    /// đổi thành "rỗng nhưng hợp lệ".
    #[test]
    fn guarded_dict_layers_returns_none_and_does_not_silently_fall_back_to_empty_when_not_managed()
    {
        assert!(
            guarded_dict_layers(None, "import_scan").is_none(),
            "DictLayers chua duoc quan ly -- phai lan None ra ngoai, khong tu doi thanh rong"
        );
    }

    /// Đối chứng dương: một `DictLayers` ĐÃ quản lý (kể cả khi rỗng — trạng thái bình
    /// thường, AD-25) phải đi qua NGUYÊN VẸN, không hàm này tự tráo bằng một bản khác.
    #[test]
    fn guarded_dict_layers_passes_the_managed_layers_through_unchanged() {
        let layers = crate::core::dict::DictLayers::empty();
        let out = guarded_dict_layers(Some(&layers), "import_scan");
        assert!(
            out.is_some(),
            "DictLayers da quan ly (du rong) van phai di qua -- day la trang thai binh thuong"
        );
        assert!(
            std::ptr::eq(out.expect("da kiem is_some o tren"), &layers),
            "phai tra ve DUNG tham chieu da nhan, khong dung mot ban thay the nao khac"
        );
    }

    /// 🔴 **AC10 (Story 1.16)** — kiểm bằng chính cơ chế mà lỗi biểu hiện: giá trị CŨ,
    /// lúc bị drop, tự khoá LẠI cùng một mutex. Bản lỗi (`*guard = Some(new)`) drop giá
    /// trị cũ trong khi `guard` vẫn sống ⇒ `try_lock()` bên dưới trả `Err` và test đỏ.
    /// Bản đã vá nhả khoá trước, nên `try_lock()` thành công.
    #[test]
    fn swap_locked_drops_the_old_value_after_the_lock_is_released() {
        struct ReentrantProbe(Arc<Mutex<Option<ReentrantProbe>>>);

        impl Drop for ReentrantProbe {
            fn drop(&mut self) {
                assert!(
                    self.0.try_lock().is_ok(),
                    "gia tri CU dang bi drop trong khi mutex van con khoa -- AC10 vo hieu"
                );
            }
        }

        let mutex: Arc<Mutex<Option<ReentrantProbe>>> = Arc::new(Mutex::new(None));

        let first = swap_locked(&mutex, ReentrantProbe(Arc::clone(&mutex)));
        assert!(
            first.is_none(),
            "mutex rong luc dau ⇒ khong co gia tri CU nao"
        );

        let second = swap_locked(&mutex, ReentrantProbe(Arc::clone(&mutex)));
        assert!(second.is_some());
        drop(second); // Drop cua ReentrantProbe tu assert ⇒ day la phep kiem that su.

        // 🔴 Lay gia tri CON LAI ra roi tha NGOAI khoa — hai viec trong mot dong.
        //
        // (1) Pha chu trinh `Arc`: gia tri cuoi nam TRONG chinh mutex ma no giu mot `Arc`
        //     toi, nen refcount khong bao gio ve 0 ⇒ `Drop` cua no khong bao gio chay
        //     va bo nho ro o cuoi test. Bat o luot code review 2026-08-06.
        // (2) Cho phep chinh phep kiem chay them mot lan nua: `take()` trong mot khoi rieng
        //     nha `guard` TRUOC, roi `drop(last)` chay `try_lock()` khi mutex da ranh.
        let last = { mutex.lock().unwrap().take() };
        assert!(
            last.is_some(),
            "mutex phai con dung mot gia tri sau ca hai luot swap"
        );
        drop(last);
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // P2 (vòng rà THỨ HAI, 2026-08-27) — `resolve_library_root_from`/
    // `resolve_configured_library_root` KHÔNG có một phép kiểm HÀNH VI nào trước bản vá:
    // cả ba nhánh sống trong `resolve_library_root(app, store)`, đòi `&tauri::AppHandle` mà
    // crate này không có cách dựng giả (không `test-utils`). Tách hai hàm THUẦN để phủ được
    // BA nhánh: giá trị đã cấu hình thắng · `load_global_config` lỗi ⇒ rơi về mặc định ·
    // `store = None` ⇒ rơi về mặc định — cộng ca "override thắng giá trị cấu hình" (nay
    // kiểm được vì không cần `AppHandle`).
    // ═════════════════════════════════════════════════════════════════════════════

    static NEXT_ROOT_DIR: AtomicU64 = AtomicU64::new(0);

    fn root_test_dir(tag: &str) -> std::path::PathBuf {
        let n = NEXT_ROOT_DIR.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "auratranslate-resolve-library-root-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
        dir
    }

    fn root_test_cleanup(dir: &std::path::Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    /// §Always của story 5.3: móc e2e LUÔN thắng, kể cả khi đã có giá trị cấu hình.
    #[test]
    fn resolve_library_root_from_override_wins_even_when_configured_is_present() {
        let result = super::resolve_library_root_from(
            Some(std::path::PathBuf::from("/override")),
            Some("/da-cau-hinh".to_owned()),
            || panic!("default KHONG duoc goi khi override co mat"),
        );
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/override"));
    }

    #[test]
    fn resolve_library_root_from_uses_the_configured_value_when_override_is_absent() {
        let result = super::resolve_library_root_from(
            None,
            Some("/da-cau-hinh".to_owned()),
            || panic!("default KHONG duoc goi khi da co gia tri cau hinh"),
        );
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/da-cau-hinh"));
    }

    #[test]
    fn resolve_library_root_from_calls_the_default_only_when_both_are_absent() {
        let result =
            super::resolve_library_root_from(None, None, || Ok(std::path::PathBuf::from("/mac-dinh")));
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/mac-dinh"));
    }

    #[test]
    fn resolve_library_root_from_propagates_a_default_error() {
        let result = super::resolve_library_root_from(None, None, || {
            Err(crate::core::store::StoreError::OpenFailed {
                store: crate::core::store::StoreKind::Global,
                detail: "gia lap".to_owned(),
            }
            .into())
        });
        assert!(result.is_err(), "loi tu default phai duoc truyen nguyen ven, khong bi nuot");
    }

    #[test]
    fn resolve_configured_library_root_with_no_store_is_not_configured() {
        assert_eq!(super::resolve_configured_library_root(None), None);
    }

    #[test]
    fn resolve_configured_library_root_with_nothing_saved_is_not_configured() {
        let dir = root_test_dir("fresh");
        let store = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .unwrap_or_else(|e| panic!("mo global.db: {e}"));

        assert_eq!(super::resolve_configured_library_root(Some(&store)), None);

        drop(store);
        root_test_cleanup(&dir);
    }

    #[test]
    fn resolve_configured_library_root_returns_a_saved_value() {
        let dir = root_test_dir("configured");
        let store = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .unwrap_or_else(|e| panic!("mo global.db: {e}"));
        crate::core::scope::save_value(&store, "app_config", "library_root", "/tu-cau-hinh")
            .unwrap_or_else(|e| panic!("ghi cau hinh: {e}"));

        assert_eq!(
            super::resolve_configured_library_root(Some(&store)),
            Some("/tu-cau-hinh".to_owned())
        );

        drop(store);
        root_test_cleanup(&dir);
    }

    /// `ReaderPool::close()` (doc-comment của chính nó): "Sau lời gọi này, `read()` trả
    /// `StoreError::PoolClosed`" — cách TẤT ĐỊNH duy nhất để dựng một `load_global_config`
    /// trượt mà không cần một `global.db` hỏng thật trên đĩa.
    #[test]
    fn resolve_configured_library_root_falls_back_to_not_configured_when_the_read_fails() {
        let dir = root_test_dir("read-fails");
        let store = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .unwrap_or_else(|e| panic!("mo global.db: {e}"));
        store.close();

        assert_eq!(
            super::resolve_configured_library_root(Some(&store)),
            None,
            "doc cau hinh truot khong duoc lam ung dung nga -- phai roi ve 'chua cau hinh'"
        );

        drop(store);
        root_test_cleanup(&dir);
    }

    /// R2 (vòng rà đối kháng 3, lớp 3) — vị từ Rust phải khớp ĐÚNG những `CHECK` mà
    /// `ASSET_DDL` khai: một hàng hợp lệ (fixture thật) qua được, một hàng vi phạm MỖI cột
    /// một lượt phải bị bắt.
    fn well_formed_saved_asset() -> SavedAsset {
        SavedAsset {
            chapter_index: 0,
            block_index: 0,
            anchor_after_segment_ord: 1,
            file_name: "abc123.jpg".to_owned(),
            source_url: Some("https://example.test/a.jpg".to_owned()),
            byte_len: 10,
            content_type: "image/jpeg".to_owned(),
        }
    }

    #[test]
    fn a_well_formed_saved_asset_satisfies_every_check_constraint() {
        assert!(saved_asset_satisfies_asset_check_constraints(&well_formed_saved_asset()));
    }

    /// **THÊM 2026-09-09 (Story 6.12)** — `source_url: None` (ảnh `.docx` nhúng) phải THOẢ
    /// mãn CHECK, đúng khuôn `ASSET_DDL` (`source_url IS NULL OR trim(...) <> ''`) — ca ÂM
    /// đi kèm với "source_url rỗng phải bị bắt" ngay dưới, chứng minh vị từ phân biệt được
    /// `None` (hợp lệ) khỏi `Some("")` (không hợp lệ).
    #[test]
    fn a_saved_asset_with_no_source_url_still_satisfies_the_check_constraint() {
        let mut docx_asset = well_formed_saved_asset();
        docx_asset.source_url = None;
        assert!(
            saved_asset_satisfies_asset_check_constraints(&docx_asset),
            "source_url: None (anh .docx) phai duoc CHAP NHAN, dung nghia NULL cua ASSET_DDL"
        );
    }

    #[test]
    fn saved_asset_check_constraints_catch_a_violation_on_each_field_one_at_a_time() {
        let mut bad = well_formed_saved_asset();
        bad.file_name = "   ".to_owned();
        assert!(!saved_asset_satisfies_asset_check_constraints(&bad), "file_name toan khoang trang phai bi bat");

        let mut bad = well_formed_saved_asset();
        bad.source_url = Some(String::new());
        assert!(!saved_asset_satisfies_asset_check_constraints(&bad), "source_url la Some(\"\") phai bi bat");

        let mut bad = well_formed_saved_asset();
        bad.content_type = "\u{3000}".to_owned();
        assert!(!saved_asset_satisfies_asset_check_constraints(&bad), "content_type toan U+3000 phai bi bat");

        let mut bad = well_formed_saved_asset();
        bad.byte_len = -1;
        assert!(!saved_asset_satisfies_asset_check_constraints(&bad), "byte_len am phai bi bat");

        let mut bad = well_formed_saved_asset();
        bad.anchor_after_segment_ord = -1;
        assert!(!saved_asset_satisfies_asset_check_constraints(&bad), "anchor am phai bi bat");
    }

    /// R6 (vòng rà đối kháng 3, lớp 3) — `chapter_index` trong dải `0..chapters_len` phải
    /// qua được vị từ; NGOÀI dải (kể cả ĐÚNG BẰNG `chapters_len`, biên trên không hợp lệ)
    /// phải bị bắt.
    #[test]
    fn saved_asset_chapter_index_in_range_accepts_valid_indices_and_rejects_out_of_range_ones() {
        let mut a = well_formed_saved_asset();
        a.chapter_index = 0;
        assert!(saved_asset_chapter_index_is_in_range(&a, 3), "chapter_index 0 trong dai 0..3 phai qua");

        let mut b = well_formed_saved_asset();
        b.chapter_index = 2;
        assert!(saved_asset_chapter_index_is_in_range(&b, 3), "chapter_index 2 trong dai 0..3 phai qua");

        let mut c = well_formed_saved_asset();
        c.chapter_index = 3;
        assert!(
            !saved_asset_chapter_index_is_in_range(&c, 3),
            "chapter_index == chapters_len (bien tren, KHONG hop le) phai bi bat"
        );

        let mut d = well_formed_saved_asset();
        d.chapter_index = 99;
        assert!(!saved_asset_chapter_index_is_in_range(&d, 3), "chapter_index vuot xa dai phai bi bat");
    }

