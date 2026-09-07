//! Hành vi của luật làm sạch lúc nhập — Story 6.5, FR124, AD-18 — I/O & Edge-Case Matrix ở
//! tầng LỆNH/PIPELINE (không ở tầng hàm thuần `core::cleanup::apply`, canh riêng ở
//! `core/cleanup/mod.rs::tests`).
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `glossary_contract.rs`/`project_contract.rs` — một tệp,
//! một mối quan tâm. Phép kiểm TĨNH trên cây nguồn sống ở `cleanup_boundary.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — thừa kế nguyên vẹn từ `project_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + `AtomicU64`). Không thêm `tempfile`.
//! 2. **Drop `Store`/`OpenWork` TRƯỚC khi xoá thư mục** — Windows từ chối xoá tệp đang mở.
//! 3. Không `sleep` dài.
//! 4. Không ca nào treo khi nó trượt.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::cleanup::{
    CleanupRuleWire, cleanup_add_rule, cleanup_delete_rule, cleanup_edit_rule,
    cleanup_list_rules, cleanup_set_enabled,
};
use auratranslate_lib::commands::project::{
    BlockBodyWire, OpenWork, PendingImportSourceState, cleanup_and_chapters_preview_for,
    confirm_import_with_encoding, create_work, preview_import_encoding, set_block_override,
    stash_pending_import_source,
};
use auratranslate_lib::core::cleanup::{CleanupRule, CleanupRuleKind, CleanupRuleTier};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::scope::ScopeResolver;
use auratranslate_lib::core::segment::pipeline::{
    ChapterInput, PipelineInput, PipelineShape, run_import,
};
use auratranslate_lib::core::store::{Store, StoreSpec};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-cleanup-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup_dir(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn open_global(dir: &Path) -> Store {
    Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db")
}

/// Một `OpenWork` THẬT (Tác phẩm dựng qua đúng `create_work`) — chỉ để
/// [`cleanup_add_rule`]/`cleanup_list_rules`/… có một tầng Work mà định tuyến, không cần
/// văn bản/luật gì đặc biệt.
fn open_work_real(documents_root: &Path) -> OpenWork {
    create_work(
        documents_root,
        "Work Stub",
        "en",
        "",
        PipelineShape::Blob(ChapterInput::AlreadyText("noi dung".to_owned())),
        encoding_rs::UTF_8,
        Vec::new(),
    None,
    Vec::new(),
)
    .expect("tao OpenWork that bai")
}

fn read_source_text(opened: &OpenWork) -> String {
    opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT source_text FROM chapter WHERE id = ?1",
                [opened.chapter_id],
                |r| r.get(0),
            )
        })
        .expect("doc source_text")
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 1 — không luật nào ⇒ nguyên trạng, không đổi một byte
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn zero_rules_leaves_source_text_byte_for_byte_unchanged() {
    let root = temp_dir("zero-rules");
    let opened = create_work(
        &root,
        "Khong Luat",
        "zh",
        "",
        PipelineShape::Blob(ChapterInput::AlreadyText("một đoạn văn nguyên vẹn".to_owned())),
        encoding_rs::UTF_8,
        Vec::new(),
    None,
    Vec::new(),
)
    .expect("tao tac pham that bai");

    assert_eq!(read_source_text(&opened), "một đoạn văn nguyên vẹn");

    drop(opened.store);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 2 — mẫu chuỗi trần: ba chỗ biến mất khỏi source_text sau xác nhận
// (🔴 CA DƯƠNG BẮT BUỘC (a) của spec 6.5, Task list)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn confirming_an_import_with_an_enabled_literal_rule_removes_every_match_from_the_written_source_text()
 {
    let root = temp_dir("literal-rule-removes-text");
    let rule = CleanupRule {
        tier: CleanupRuleTier::Global,
        id: 1,
        pattern: "求收藏".to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    };
    let text = "求收藏 phan dau. noi dung that. 求收藏 phan cuoi.".to_owned();

    let opened = create_work(
        &root,
        "Co Luat",
        "zh",
        "",
        PipelineShape::Blob(ChapterInput::AlreadyText(text)),
        encoding_rs::UTF_8,
        vec![rule],
    None,
    Vec::new(),
)
    .expect("tao tac pham that bai");

    let source_text = read_source_text(&opened);
    assert!(
        !source_text.contains("求收藏"),
        "ca luat da BAT phai xoa MOI cho khop khoi source_text ghi xuong, con lai: {source_text:?}"
    );

    drop(opened.store);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 3 — mẫu regex, khớp theo DÒNG (đa dòng, `(?m)`)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_regex_rule_matches_per_line_across_a_multi_line_chapter() {
    let root = temp_dir("regex-rule-per-line");
    let rule = CleanupRule {
        tier: CleanupRuleTier::Work,
        id: 1,
        pattern: "^本章由.*整理$".to_owned(),
        kind: CleanupRuleKind::Regex,
        enabled: true,
    };
    let text = "dong dau khong lien quan\n本章由XYZ整理\ndong cuoi khong lien quan".to_owned();

    let opened = create_work(
        &root,
        "Regex Da Dong",
        "zh",
        "",
        PipelineShape::Blob(ChapterInput::AlreadyText(text)),
        encoding_rs::UTF_8,
        vec![rule],
    None,
    Vec::new(),
)
    .expect("tao tac pham that bai");

    let source_text = read_source_text(&opened);
    assert!(
        !source_text.contains("本章由"),
        "mau regex neo theo DONG phai khop dung dong giua, con lai: {source_text:?}"
    );
    assert!(source_text.contains("dong dau"), "hai dong con lai phai giu nguyen");
    assert!(source_text.contains("dong cuoi"), "hai dong con lai phai giu nguyen");

    drop(opened.store);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 4 — regex hỏng: lưu bị từ chối, bảng không đổi một hàng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn saving_an_invalid_regex_pattern_is_refused_and_the_table_gains_no_row() {
    let root = temp_dir("invalid-regex-refused");
    let global = open_global(&root);

    let err = cleanup_add_rule(
        Some(&global),
        None,
        CleanupRuleTier::Global,
        "[unclosed",
        CleanupRuleKind::Regex,
    )
    .expect_err("mau regex hong phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::CleanupInvalidRegex);

    let rules = cleanup_list_rules(Some(&global), None).expect("liet ke sau lan tu choi");
    assert!(rules.is_empty(), "khong hang nao duoc ghi khi mau regex hong bi tu choi");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 5 — mẫu rỗng/chỉ khoảng trắng: bị từ chối ở tầng lệnh
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn saving_a_whitespace_only_pattern_is_refused_at_the_command_layer() {
    let root = temp_dir("whitespace-only-refused");
    let global = open_global(&root);

    let err = cleanup_add_rule(
        Some(&global),
        None,
        CleanupRuleTier::Global,
        "\u{3000}  ",
        CleanupRuleKind::Literal,
    )
    .expect_err("mau chi khoang trang phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::CleanupEmptyPattern);

    let rules = cleanup_list_rules(Some(&global), None).expect("liet ke sau lan tu choi");
    assert!(rules.is_empty());

    drop(global);
    cleanup_dir(&root);
}

/// Cùng mệnh đề, tầng DDL — `CHECK` của `import_cleanup_rule` phải tự canh nếu một chỗ gọi
/// khác (không đi qua `cleanup_add_rule`) cố ghi thẳng.
#[test]
fn the_ddl_check_constraint_alone_refuses_a_whitespace_only_pattern() {
    let root = temp_dir("ddl-whitespace-refused");
    let global = open_global(&root);

    let result = global.write(|tx: &auratranslate_lib::core::store::Transaction<'_>| {
        tx.execute(
            "INSERT INTO import_cleanup_rule (pattern, kind, enabled, ord, created_at) \
             VALUES (?1, 'literal', 1, 1, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            ["\u{3000}  "],
        )
    });
    assert!(result.is_err(), "CHECK cua DDL phai tu choi mau chi khoang trang o tang SQL");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 6 — hai tầng cùng khớp: xoá một lần, cả hai luật đều đếm
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn two_tiers_matching_the_same_spot_delete_it_once_but_both_rules_count_it_end_to_end() {
    let global_rule = CleanupRule {
        tier: CleanupRuleTier::Global,
        id: 1,
        pattern: "X".to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    };
    let work_rule = CleanupRule {
        tier: CleanupRuleTier::Work,
        id: 1,
        pattern: "X".to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    };

    let input = PipelineInput::default_shaped(
        PipelineShape::Blob(ChapterInput::AlreadyText("aXb".to_owned())),
        "en",
    )
    .with_cleanup_rules(vec![global_rule, work_rule]);

    let outcome = run_import(input).expect("chuoi pipeline khong duoc loi");
    assert_eq!(outcome.chapters[0].source_text, "ab", "cho khop chi bien mat MOT LAN");

    let report = outcome.chapters[0]
        .cleanup_report
        .as_ref()
        .expect("bao cao lam sach phai co mat");
    assert_eq!(report.per_rule_counts[&(CleanupRuleTier::Global, 1)], 1);
    assert_eq!(report.per_rule_counts[&(CleanupRuleTier::Work, 1)], 1);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 7 — trùng id giữa hai tầng: hai hàng riêng biệt, bật/tắt độc lập
// (🔴 CA DƯƠNG BẮT BUỘC (b) của spec 6.5, Task list)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_global_rule_number_one_and_a_work_rule_number_one_coexist_and_toggle_independently() {
    let root = temp_dir("global-and-work-both-id-one");
    let global = open_global(&root);
    let open = open_work_real(&root);

    let global_id = cleanup_add_rule(
        Some(&global),
        None,
        CleanupRuleTier::Global,
        "mau toan cuc",
        CleanupRuleKind::Literal,
    )
    .expect("them luat Toan cuc that bai");
    let work_id = cleanup_add_rule(
        Some(&global),
        Some(&open),
        CleanupRuleTier::Work,
        "mau tac pham",
        CleanupRuleKind::Literal,
    )
    .expect("them luat Tac pham that bai");

    // Hai tầng đánh số ĐỘC LẬP — cả hai đều là hàng ĐẦU TIÊN của kho riêng chúng, nên cùng
    // mang `id = 1`. Đây CHÍNH LÀ mệnh đề "danh tính là cặp (tier, id)".
    assert_eq!(global_id, 1, "hang dau tien cua tang Global phai mang id 1");
    assert_eq!(work_id, 1, "hang dau tien cua tang Work phai mang id 1, DOC LAP voi Global");

    let all = cleanup_list_rules(Some(&global), Some(&open)).expect("liet ke hai tang");
    assert_eq!(all.len(), 2, "hai luat rieng biet phai cung co mat, khong cai nao doi lot cai kia");

    // Tắt luật Work — luật Global PHẢI giữ nguyên trạng thái bật.
    cleanup_set_enabled(Some(&global), Some(&open), CleanupRuleTier::Work, work_id, false)
        .expect("tat luat Work that bai");

    let after: Vec<CleanupRuleWire> =
        cleanup_list_rules(Some(&global), Some(&open)).expect("liet ke sau khi tat");
    let global_row = after
        .iter()
        .find(|r| matches!(r.tier, auratranslate_lib::commands::project::CleanupRuleTierWire::Global))
        .expect("hang Global phai con");
    let work_row = after
        .iter()
        .find(|r| matches!(r.tier, auratranslate_lib::commands::project::CleanupRuleTierWire::Work))
        .expect("hang Work phai con");
    assert!(global_row.enabled, "tat luat Work KHONG duoc lam tat theo luat Global");
    assert!(!work_row.enabled, "luat Work phai da tat");

    drop(open.store);
    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 8 — tắt một luật: chỗ vừa gạch ngang trở về nguyên trạng NGAY, số đếm ở lại
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn disabling_a_previously_matched_rule_removes_its_span_immediately_but_keeps_its_count() {
    let root = temp_dir("disable-removes-span-keeps-count");
    let global = open_global(&root);

    let id = cleanup_add_rule(
        Some(&global),
        None,
        CleanupRuleTier::Global,
        "rac ruoi",
        CleanupRuleKind::Literal,
    )
    .expect("them luat that bai");

    let shape = PipelineShape::Blob(ChapterInput::AlreadyText(
        "dau rac ruoi giua rac ruoi cuoi".to_owned(),
    ));

    let rules_on = auratranslate_lib::core::cleanup::resolve_two_tiers(
        &ScopeResolver::global_only(),
        &global,
        None,
    )
    .expect("phan giai hai tang");
    let preview_on = preview_import_encoding(&shape, "en", &rules_on, None, &[]);
    let cleanup_on = preview_on
        .self_declared_cleanup
        .as_ref()
        .expect("nhanh tu khai phai co khoi lam sach");
    assert_eq!(cleanup_on.spans.len(), 2, "luat dang BAT phai hien hai cho gach ngang");
    assert_eq!(cleanup_on.rules[0].count_in_chapter, 2);

    cleanup_set_enabled(Some(&global), None, CleanupRuleTier::Global, id, false)
        .expect("tat luat that bai");

    let rules_off = auratranslate_lib::core::cleanup::resolve_two_tiers(
        &ScopeResolver::global_only(),
        &global,
        None,
    )
    .expect("phan giai hai tang sau khi tat");
    let preview_off = preview_import_encoding(&shape, "en", &rules_off, None, &[]);
    let cleanup_off = preview_off
        .self_declared_cleanup
        .as_ref()
        .expect("nhanh tu khai phai co khoi lam sach");
    assert!(
        cleanup_off.spans.is_empty(),
        "luat vua TAT khong duoc con span nao trong ban dung"
    );
    assert_eq!(
        cleanup_off.rules[0].count_in_chapter, 2,
        "so dem phai GIU NGUYEN sau khi tat -- tat doi viec xoa, khong doi viec do"
    );
    assert!(!cleanup_off.rules[0].enabled);

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 9 — dán văn bản tay (0 ứng viên bảng mã): tầng 3 vẫn đầy đủ
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn pasted_text_with_zero_encoding_candidates_still_gets_a_full_cleanup_block() {
    let root = temp_dir("pasted-text-self-declared-cleanup");
    let global = open_global(&root);
    cleanup_add_rule(Some(&global), None, CleanupRuleTier::Global, "xoa", CleanupRuleKind::Literal)
        .expect("them luat that bai");
    let rules = auratranslate_lib::core::cleanup::resolve_two_tiers(
        &ScopeResolver::global_only(),
        &global,
        None,
    )
    .expect("phan giai hai tang");

    let shape = PipelineShape::Blob(ChapterInput::AlreadyText("truoc xoa sau".to_owned()));
    let preview = preview_import_encoding(&shape, "en", &rules, None, &[]);

    assert!(preview.candidates.is_empty(), "duong AlreadyText phai cho 0 ung vien bang ma");
    let cleanup = preview
        .self_declared_cleanup
        .as_ref()
        .expect("nhanh tu khai (0 ung vien) van phai co khoi lam sach rieng cua no");
    assert_eq!(cleanup.spans.len(), 1);
    assert_eq!(cleanup.final_text, "truoc  sau");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 10 — 🔴 CA DƯƠNG BẮT BUỘC (c): preview VÀ confirm cho CÙNG văn bản, cùng đầu vào
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn preview_and_confirm_agree_byte_for_byte_on_the_same_input_and_the_same_rules() {
    let root = temp_dir("preview-confirm-agree");
    let global = open_global(&root);
    cleanup_add_rule(
        Some(&global),
        None,
        CleanupRuleTier::Global,
        "quang cao",
        CleanupRuleKind::Literal,
    )
    .expect("them luat that bai");
    let rules = auratranslate_lib::core::cleanup::resolve_two_tiers(
        &ScopeResolver::global_only(),
        &global,
        None,
    )
    .expect("phan giai hai tang");

    // Văn bản NGẮN — lọt trọn trong cửa sổ bằng chứng, nên `window_truncated == false` và
    // phép so byte-for-byte có nghĩa (xem doc-comment `CleanupPreviewWire::final_text`).
    let text = "dau truyen. quang cao. cuoi truyen.".to_owned();
    let shape = PipelineShape::Blob(ChapterInput::AlreadyText(text.clone()));

    let preview = preview_import_encoding(&shape, "en", &rules, None, &[]);
    let cleanup = preview
        .self_declared_cleanup
        .as_ref()
        .expect("nhanh tu khai phai co khoi lam sach");
    assert!(!cleanup.window_truncated, "tien de: van ban phai lot tron cua so");

    let state: PendingImportSourceState = std::sync::Mutex::new(None);
    stash_pending_import_source(&state, shape);
    let opened = confirm_import_with_encoding(
        &root,
        &state,
        "Preview Confirm Agree",
        "en",
        "",
        "UTF-8",
        rules,
        None,
        Vec::new(),
    )
    .expect("xac nhan that bai");

    let written = read_source_text(&opened);
    assert_eq!(
        written, cleanup.final_text,
        "preview_import_encoding va confirm_import_with_encoding phai cho CUNG mot ket qua, \
         tren CUNG mot dau vao -- day la phep dong no deferred-work.md:9359: hai duong phai \
         cung chay chuoi pipeline that, khong phai hai ham thuan dat canh nhau"
    );

    drop(opened.store);
    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 Story 6.9 — nối bất biến "xem trước và xác nhận trùng từng byte" sang hình dạng
// `Chapters(RawBytes)` CÓ OVERRIDE KHÁC RỖNG (Đối chứng đỏ ① của §Verification spec 6.9)
// ═════════════════════════════════════════════════════════════════════════════════
//
// Ca `preview_and_confirm_agree_byte_for_byte_on_the_same_input_and_the_same_rules` ở trên
// dùng `Blob(AlreadyText)` — `extract_main_content == false`, 0 khối nào tồn tại, nên nó
// KHÔNG THỂ đỏ vì lỗi "đường ghi bỏ qua override" (vòng rà 1 đã hụt đúng lỗi này: `create_work`
// không truyền `block_overrides` xuống `PipelineInput`). Ca này dựng hình dạng THẬT của đường
// URL (`Chapters(RawBytes)`, `extract_main_content == true`) với MỘT override khác rỗng, rồi
// đối chiếu văn bản TRÊN ĐĨA với `final_text` mà `preview_import_encoding` (CÙNG override) vừa
// hiện — đúng cặp mà AC🔴 của spec 6.9 đòi.
#[test]
fn preview_and_confirm_agree_byte_for_byte_on_chapters_raw_bytes_shape_with_a_non_empty_block_override() {
    let root = temp_dir("preview-confirm-agree-block-override");

    // Cùng fixture đã đo ở `webimport_contract.rs`
    // (`extracted_text_never_contains_an_angle_bracket_from_the_source_markup`) — đo được
    // BỐN khối: tiêu đề + hai đoạn thân bài `machine_kept == true` (mục 0-2), một khối
    // `<aside>` `machine_kept == false` (mục 3, "Binh luan cua doc gia...").
    let html = "<html><head><title>Bai viet</title></head><body>\
         <nav><a href=\"/menu\">Menu</a></nav>\
         <article><h1>Tieu de bai viet</h1>\
         <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
         nhieu chu hon de vuot nguong do dai toi thieu can thiet.</p>\
         <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
         du dai de dom_smoothie cham diem cao cho khoi nay mot cach ro rang.</p>\
         </article>\
         <aside><p>Binh luan cua doc gia, khong lien quan noi dung bai viet chinh, day chi la \
         rac quanh bai de kiem tra bo loc co loai duoc no khong.</p></aside>\
         </body></html>"
        .to_owned();
    let url = "https://example.com/bai-viet".to_owned();
    let shape_for_preview = PipelineShape::Chapters(vec![ChapterInput::RawBytes {
        bytes: html.clone().into_bytes(),
        label: url.clone(),
    }]);

    // 🔴 SỬA 2026-09-07 (vòng rà bước 4, mục 24) — bản trước khai "khối 0 ('Menu')" ở đây
    // VÀ "ép khối 0 GIỮ" ở lượt 2 bên dưới: SAI CHỈ SỐ. Khối override THẬT SỰ trong ca này
    // là khối 3 (`<aside>`, bình luận độc giả — xem assert `baseline_blocks.blocks[3]` ngay
    // dưới VÀ `set_block_override(&mut overrides, 3, …)` ở lượt 2); "Menu" nằm trong `<nav>`,
    // vốn không tạo ra một `Candidate` nào cả (không khớp `BLOCK_SELECTOR`) nên không có
    // "khối 0" nào ứng với nó để mà loại/giữ. Lượt 1 (KHÔNG override) chỉ để xác nhận khối 3
    // THẬT SỰ bị máy loại, nên ép nó GIỮ ở lượt 2 là một thay đổi QUAN SÁT ĐƯỢC (văn bản DÀI
    // HƠN), không phải một override vô hại trùng với máy đã quyết.
    let baseline = preview_import_encoding(&shape_for_preview, "en", &[], None, &[]);
    let baseline_candidate = baseline
        .candidates
        .iter()
        .find(|c| c.encoding == "UTF-8")
        .expect("phai co ung vien UTF-8");
    let baseline_blocks = baseline_candidate
        .blocks
        .as_ref()
        .expect("extract_main_content == true tren duong URL phai cho Some(blocks)");
    assert!(
        !baseline_blocks.blocks[3].kept,
        "tien de: khoi 3 (<aside>, binh luan) phai bi MAY LOAI that su — neu khong, ca nay \
         khong do duoc gi khi ai go .with_block_overrides() khoi create_work"
    );
    let baseline_text = baseline_candidate
        .cleanup
        .as_ref()
        .expect("UTF-8 phai ra chu")
        .final_text
        .clone();

    // Lượt 2 (CÓ override) — ép khối 3 GIỮ (`Space` trên một khối `ornament`, đúng I/O Matrix
    // spec 6.9: "khối thành `confirmed`, văn bản sẽ ghi DÀI RA").
    let mut overrides: Vec<Option<bool>> = Vec::new();
    set_block_override(&mut overrides, 3, true, baseline_blocks.blocks.len())
        .expect("index 3 phai nam trong tong so khoi that cua baseline");

    let preview_with_override =
        preview_import_encoding(&shape_for_preview, "en", &[], None, &overrides);
    let candidate_with_override = preview_with_override
        .candidates
        .iter()
        .find(|c| c.encoding == "UTF-8")
        .expect("phai co ung vien UTF-8");
    let blocks_with_override = candidate_with_override
        .blocks
        .as_ref()
        .expect("Some(blocks) khong doi giua hai luot");
    assert!(blocks_with_override.blocks[3].kept, "override phai thang machine_kept");
    assert!(blocks_with_override.blocks[3].confirmed, "override co mat o vi tri 3 => confirmed");
    let final_text_with_override = candidate_with_override
        .cleanup
        .as_ref()
        .expect("UTF-8 phai ra chu")
        .final_text
        .clone();
    // 🔴 SỬA 2026-09-07 (vòng rà bước 4, mục 24) — bản trước SO SÁNH bằng `.len()` (BYTE)
    // nhưng IN ra bằng `.chars().count()` (KÝ TỰ) — hai đơn vị LỆCH NHAU trong CÙNG một
    // assert, nên một lượt đọc log khi ca này đỏ thấy hai con số không phải là thứ vừa được
    // so. Cả hai vế giờ CÙNG một đơn vị (ký tự) — đúng thứ AC quan tâm ("văn bản dài ra" là
    // một mệnh đề về NỘI DUNG, không phải về số byte UTF-8 chiếm dụng).
    assert!(
        final_text_with_override.chars().count() > baseline_text.chars().count(),
        "vach lai I/O Matrix spec 6.9: giu mot khoi ornament phai lam van ban DAI RA \
         (baseline {} ky tu, co override {} ky tu)",
        baseline_text.chars().count(),
        final_text_with_override.chars().count()
    );

    // Xác nhận THẬT — CÙNG override, qua ĐÚNG đường ghi `create_work` (`confirm_import_with_encoding`).
    let shape_for_confirm = PipelineShape::Chapters(vec![ChapterInput::RawBytes {
        bytes: html.into_bytes(),
        label: url,
    }]);
    let state: PendingImportSourceState = std::sync::Mutex::new(None);
    stash_pending_import_source(&state, shape_for_confirm);
    let opened = confirm_import_with_encoding(
        &root,
        &state,
        "Preview Confirm Agree Block Override",
        "en",
        "",
        "UTF-8",
        Vec::new(),
        None,
        overrides,
    )
    .expect("xac nhan that bai");

    let written = read_source_text(&opened);
    assert_eq!(
        written, final_text_with_override,
        "🔴 DUONG GHI BO QUA OVERRIDE — day la ca ma vong ra 1 da HUT: `create_work` phai ap \
         `block_overrides` giong het `preview_import_encoding` vua hien; neu ai go \
         `.with_block_overrides(...)` khoi `create_work`, dong nay do (van ban tren dia roi ve \
         phan doan may (baseline), khong con khop voi ban xem truoc da hien)"
    );
    assert_ne!(
        written, baseline_text,
        "tien de kep: neu dong nay bang baseline_text thi override khong he co tac dung o \
         DUONG GHI, dung cach vo hieu hoa ma ca nay ton tai de bat"
    );

    drop(opened.store);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 Story 6.9 — SỬA 2026-09-07 (vòng rà bước 4, mục 25): bằng chứng THẬT cho lời khai
// "ĐÓNG CHO TẦNG 2" của D2 (`deferred-work.md`, khoản "Ba trong bốn bước còn lại là THÂN
// RỖNG").
// ═════════════════════════════════════════════════════════════════════════════════
//
// Ca `preview_and_confirm_agree_byte_for_byte_on_chapters_raw_bytes_shape_with_a_non_empty_block_override`
// ở trên chỉ dùng byte ASCII và chỉ đọc candidate UTF-8 — ASCII giải mã Y HỆT dưới MỌI bảng
// mã byte-đơn-vị (UTF-8/GB18030/GBK/Big5, xem
// `segment_contract.rs::preview_of_an_ascii_only_file_...`), nên ca đó KHÔNG PHÂN BIỆT ĐƯỢC
// "mỗi ứng viên tự giải mã bằng bảng mã CỦA CHÍNH NÓ" (lời khai D2) khỏi một cài đặt lỗi
// "luôn giải mã bằng UTF-8/ứng viên đầu rồi dùng chung cho cả năm ô" — hai cài đặt đó cho ra
// CÙNG kết quả trên input ASCII, đúng lớp chứng cứ yếu mà `AGENTS.md` gọi tên là "triệu
// chứng có ở cả hai phía không phải nguyên nhân". Ca NÀY dùng byte GBK THẬT (tiếng Trung,
// giải mã SAI dưới UTF-8 thành mojibake) để hai giả thuyết đó TÁCH RA được: nếu tầng 2 của
// ứng viên GBK đọc ra CHỮ TRUNG THẬT (không phải mojibake), cơ chế đúng như lời khai (đúng
// điều `commands::project::encoding_candidate_wire` đọc thấy khi kiểm mã: nó gọi
// `encoding::encoding_for_wire_id(c.wire_id)` — bảng mã CỦA ĐÚNG ứng viên `c`, không phải
// một hằng số dùng chung); nếu nó đọc ra mojibake, lời khai sai.
#[test]
fn each_candidates_tier2_blocks_are_decoded_with_that_candidates_own_encoding_not_a_shared_one() {
    let html_text = "<html><body><article><h1>Chuong 01</h1>\
         <p>萧炎在东临村口的一处石壁上练习着最基础的吐纳法门，为了能夠早日突破斗之力三段，甚至\
         每天都要练到深夜才肯罢休，日复一日从未松懈过半分，只求有朝一日能夠离开这个令人窒息的\
         偏僻山村。</p></article></body></html>";
    let (bytes, _, had_errors) = encoding_rs::GBK.encode(html_text);
    assert!(!had_errors, "fixture phai ma hoa GBK sach");
    let shape = PipelineShape::Chapters(vec![ChapterInput::RawBytes {
        bytes: bytes.into_owned(),
        label: "https://example.com/gbk-article".to_owned(),
    }]);

    let preview = preview_import_encoding(&shape, "en", &[], None, &[]);
    let gbk_candidate = preview.candidates.iter().find(|c| c.label == "GBK").expect("phai co o GBK");
    let gbk_blocks = gbk_candidate
        .blocks
        .as_ref()
        .expect("extract_main_content == true tren duong URL phai cho Some(blocks), ke ca ung vien khong duoc chon mac dinh");

    let gbk_text: String = gbk_blocks
        .blocks
        .iter()
        .filter_map(|b| match &b.body {
            BlockBodyWire::Paragraph { text } => Some(text.clone()),
            BlockBodyWire::Caption { text } => Some(text.clone()),
            BlockBodyWire::Image { .. } => None,
        })
        .collect::<Vec<_>>()
        .join(" ");

    assert!(
        gbk_text.contains("萧炎"),
        "ung vien GBK phai TU GIAI MA bang CHINH bang ma cua no -- doc duoc chu Trung THAT, \
         khong phai mojibake tu mot bang ma khac (vi du UTF-8 cua ung vien dau) bi dung chung \
         cho ca nam o. Khoi doc duoc: {gbk_text:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 10 — luật xoá sạch Chương — GHI NHẬN HÀNH VI THẬT, KHÔNG SUY LUẬN
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔵 SỬA vòng rà (2026-09-06) — khối này từng tự khai "Hàng 11", trùng với khối "văn bản
// dài hơn cửa sổ" bên dưới (cũng tự khai "Hàng 11"). Đếm lại theo §I/O Matrix của spec:
// 1 Không luật · 2 Mẫu chuỗi trần · 3 Mẫu regex · 4 Regex hỏng · 5 Hai tầng cùng khớp ·
// 6 Trùng id giữa hai tầng · 7 Tắt một luật · 8 Đổi ứng viên bảng mã · 9 Dán văn bản tay ·
// **10 Luật xoá sạch Chương** · 11 Văn bản dài hơn cửa sổ · 12 Mẫu rỗng/chỉ khoảng trắng.
// Khối NÀY ứng với hàng 10, không phải 11.
//
// ⚠️ Spec 6.5 §I/O Matrix khai "xác nhận ⇒ ImportError::EmptyImport như hôm nay" — biến thể
// đó KHÔNG TỒN TẠI trong `core::segment::import::ImportError` (đo bằng `grep -rn
// "EmptyImport" src-tauri/src/`: 0 kết quả). `create_work` chỉ từ chối khi `chapters.is_empty()`
// (N = 0 Chương), KHÔNG khi một Chương đơn có `source_text` rỗng sau khi luật xoá sạch nó —
// ca đó đã CÓ SẴN từ trước Story 6.5 (dán một chuỗi rỗng cũng đi qua đúng đường này). Ca dưới
// đây ghi lại hành vi THẬT thay vì suy luận theo lời khai của spec — nợ ghi ở
// `deferred-work.md`, không phải một mệnh đề bị làm nhẹ đi.
#[test]
fn a_rule_that_matches_the_entire_chapter_creates_a_chapter_with_empty_source_text_not_an_error() {
    let root = temp_dir("rule-wipes-entire-chapter");
    let rule = CleanupRule {
        tier: CleanupRuleTier::Global,
        id: 1,
        pattern: "toan bo noi dung".to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    };

    let opened = create_work(
        &root,
        "Xoa Sach",
        "en",
        "",
        PipelineShape::Blob(ChapterInput::AlreadyText("toan bo noi dung".to_owned())),
        encoding_rs::UTF_8,
        vec![rule],
    None,
    Vec::new(),
)
    .expect(
        "hanh vi THAT hom nay: create_work KHONG tu choi mot Chuong don co source_text rong \
         sau khi luat xoa sach no -- xem ghi chu tai cho khai bao ham test nay",
    );

    assert_eq!(read_source_text(&opened), "");

    let segment_count: i64 = opened
        .store
        .read(|conn| {
            conn.query_row("SELECT COUNT(*) FROM segment WHERE chapter_id = ?1", [opened.chapter_id], |r| {
                r.get(0)
            })
        })
        .expect("dem segment");
    assert_eq!(segment_count, 0, "0 chu con lai thi phai la 0 segment, khong phai mot loi");

    drop(opened.store);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 12 — xoá một luật đã biến mất, sửa/bật-tắt một luật đã biến mất
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn deleting_an_already_deleted_rule_is_harmless() {
    let root = temp_dir("delete-twice-harmless");
    let global = open_global(&root);
    let id = cleanup_add_rule(Some(&global), None, CleanupRuleTier::Global, "x", CleanupRuleKind::Literal)
        .expect("them luat that bai");

    cleanup_delete_rule(Some(&global), None, CleanupRuleTier::Global, id).expect("xoa lan mot");
    cleanup_delete_rule(Some(&global), None, CleanupRuleTier::Global, id)
        .expect("xoa lan hai (da khong con) phai VO HAI, khong phai loi");

    drop(global);
    cleanup_dir(&root);
}

#[test]
fn editing_a_vanished_rule_is_rejected_and_toggling_a_vanished_rule_is_rejected() {
    let root = temp_dir("edit-toggle-vanished-rejected");
    let global = open_global(&root);
    let id = cleanup_add_rule(Some(&global), None, CleanupRuleTier::Global, "x", CleanupRuleKind::Literal)
        .expect("them luat that bai");
    cleanup_delete_rule(Some(&global), None, CleanupRuleTier::Global, id).expect("xoa that bai");

    let edit_err = cleanup_edit_rule(
        Some(&global),
        None,
        CleanupRuleTier::Global,
        id,
        "y",
        CleanupRuleKind::Literal,
    )
    .expect_err("sua mot luat da bien mat phai bi tu choi");
    assert_eq!(edit_err.message_key(), MessageKey::CleanupRuleMissing);

    let toggle_err = cleanup_set_enabled(Some(&global), None, CleanupRuleTier::Global, id, false)
        .expect_err("bat/tat mot luat da bien mat phai bi tu choi");
    assert_eq!(toggle_err.message_key(), MessageKey::CleanupRuleMissing);

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 13 — chọn tầng Tác phẩm khi chưa mở Tác phẩm nào
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn choosing_the_work_tier_with_no_work_open_is_refused() {
    let root = temp_dir("work-tier-unavailable");
    let global = open_global(&root);

    let err = cleanup_add_rule(
        Some(&global),
        None,
        CleanupRuleTier::Work,
        "mau",
        CleanupRuleKind::Literal,
    )
    .expect_err("chon tang Work khi chua mo Tac pham phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::CleanupWorkTierUnavailable);

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 11 — văn bản DÀI HƠN cửa sổ bằng chứng
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 Mệnh đề CHỊU LỰC của hàng này KHÔNG phải `window_truncated == true` — đó chỉ là tiền
// đề. Mệnh đề thật là §Always spec 6.5: *"Hai con số của mỗi luật ... cả hai đo trên TOÀN
// văn bản, không trên cửa sổ hiển thị. Bản dựng hiển thị ĐƯỢC PHÉP cắt"*. Một số đếm đo
// trên cửa sổ mà KHAI là số của cả Chương là một câu đúng hình dạng, sai sự thật — đúng cái
// bẫy vòng rà 1 của Story 6.3 đã bắt được và §Design Notes Story 6.4 đã ghi lại.
//
// Ca này gieo một luật khớp HAI chỗ: một chỗ nằm TRONG cửa sổ bằng chứng (4096 byte đầu),
// một chỗ nằm NGOÀI nó. Bản dựng chỉ được phép thấy chỗ đầu; SỐ ĐẾM phải thấy cả hai.
#[test]
fn counts_cover_the_whole_chapter_even_when_the_rendered_window_is_truncated() {
    let root = temp_dir("counts-whole-chapter-window-cut");
    let global = open_global(&root);
    cleanup_add_rule(Some(&global), None, CleanupRuleTier::Global, "QUANGCAO", CleanupRuleKind::Literal)
        .expect("them luat that bai");
    let rules =
        auratranslate_lib::core::cleanup::resolve_two_tiers(&ScopeResolver::global_only(), &global, None)
            .expect("phan giai hai tang");

    // Chỗ khớp THỨ NHẤT ở ngay đầu (trong cửa sổ), chỗ THỨ HAI sau hơn 4096 byte đệm.
    let filler = "dong dem khong mang y nghia.\n".repeat(300);
    assert!(filler.len() > 4096, "dem phai vuot cua so bang chung that su: {} byte", filler.len());
    let text = format!("QUANGCAO dau chuong.\n{filler}QUANGCAO cuoi chuong.\n");

    let shape = PipelineShape::Blob(ChapterInput::AlreadyText(text.clone()));
    let preview = preview_import_encoding(&shape, "en", &rules, None, &[]);
    let cleanup =
        preview.self_declared_cleanup.as_ref().expect("nhanh tu khai phai co khoi lam sach");

    assert!(cleanup.window_truncated, "tien de: nguon phai dai hon cua so bang chung");
    assert!(
        cleanup.text.len() < text.len(),
        "tien de: ban dung hien thi phai NGAN hon toan Chuong"
    );

    assert_eq!(
        cleanup.rules.len(),
        1,
        "dung mot luat duoc gieo, khong phai mot tap khac"
    );
    assert_eq!(
        cleanup.rules[0].count_in_chapter, 2,
        "SO DEM phai la so cua TOAN Chuong (2 cho khop), khong phai so dem duoc trong cua so \
         hien thi (1 cho) -- doc-comment `CleanupRuleReportWire::count_in_chapter` khai \
         'tren TOAN van ban', va §Always spec 6.5 cam mot so do tren cua so ma khai la so \
         cua ca Chuong"
    );

    drop(global);
    cleanup_dir(&root);
}

// 🔴 SỬA vòng rà (2026-09-06) — một chỗ khớp VẮT QUA biên cửa sổ hiển thị (bắt đầu TRONG
// cửa sổ, kết thúc NGOÀI nó) từng bị `build_cleanup_preview_wire` LOẠI HẲN khỏi `spans`
// (lọc theo `m.end <= visible_chars`), dù phần đầu chỗ khớp vẫn đang HIỆN trên màn hình —
// người dùng thấy chữ, không thấy gạch ngang, rồi chữ đó biến mất lúc xác nhận. Đúng thủng
// lời hứa cốt lõi FR124 ("hiện thứ sắp xoá"). Ca này dựng văn bản 100 BYTE MỘT DÒNG (để vị
// trí cắt cửa sổ tính được bằng SỐ HỌC, không phải đoán): 40 dòng đầu (99 ký tự + `\n`) đưa
// biên `window_safe_prefix` xuống ĐÚNG cuối dòng thứ 38 (điểm mã 3899 — xem phép tính trong
// chú thích dưới), rồi thêm nhiều dòng đệm để vượt 4096 byte thật sự. Dòng 38 kết bằng
// "TAIL", dòng 39 mở bằng "HEAD"; luật literal `"TAIL\nHEAD"` khớp đúng [3895, 3904) — vắt
// qua điểm cắt 3899.
#[test]
fn a_match_straddling_the_window_boundary_is_clipped_to_it_not_dropped() {
    let root = temp_dir("span-straddles-window-boundary");
    let global = open_global(&root);

    const LINE_CONTENT_CHARS: usize = 99; // + 1 `\n` = 100 byte/dong, so hoc de doan bien
    let plain_line = || "F".repeat(LINE_CONTENT_CHARS);

    let mut text = String::new();
    // Dong 0..37: don thuan, khong mang gi dang chu y.
    for _ in 0..38 {
        text.push_str(&plain_line());
        text.push('\n');
    }
    // Dong 38: ket bang "TAIL" -- 95 'F' + "TAIL" = 99 ky tu.
    text.push_str(&"F".repeat(LINE_CONTENT_CHARS - 4));
    text.push_str("TAIL");
    text.push('\n');
    // Dong 39: mo bang "HEAD" -- "HEAD" + 95 'F' = 99 ky tu. Dong nay bi CAT HAN khoi cua so
    // hien thi (khong anh huong phep tinh bien, chi can dung 100 byte de gia dinh khop).
    text.push_str("HEAD");
    text.push_str(&"F".repeat(LINE_CONTENT_CHARS - 4));
    text.push('\n');
    // Dem them nhieu dong de vuot han 4096 byte (dam bao window_truncated == true THAT).
    for _ in 0..30 {
        text.push_str(&plain_line());
        text.push('\n');
    }
    assert!(text.len() > 4096 + 200, "phai vuot cua so bang chung nhieu du de chac chan bi cat");

    // Phep tinh bien (xem chu thich tren ham): 40 dong dau (99 ky tu + `\n` = 100 byte/dong)
    // dua `window_safe_prefix` ve dung 39 dong TRON VEN dau (dong 0..37 + dong 38 chua
    // "TAIL"), noi voi nhau bang 38 dau `\n` -- 39*99 + 38 = 3899 ky tu. Diem ma 3899 la
    // DUNG vi tri ky tu `\n` ket thuc dong 38 trong van ban goc.
    const EXPECTED_VISIBLE_CHARS: usize = 39 * LINE_CONTENT_CHARS + 38;
    assert_eq!(EXPECTED_VISIBLE_CHARS, 3899, "phep tinh bien tu kiem");

    cleanup_add_rule(
        Some(&global),
        None,
        CleanupRuleTier::Global,
        "TAIL\nHEAD",
        CleanupRuleKind::Literal,
    )
    .expect("them luat vat bien that bai");
    let rules =
        auratranslate_lib::core::cleanup::resolve_two_tiers(&ScopeResolver::global_only(), &global, None)
            .expect("phan giai hai tang");

    let shape = PipelineShape::Blob(ChapterInput::AlreadyText(text.clone()));
    let preview = preview_import_encoding(&shape, "en", &rules, None, &[]);
    let cleanup =
        preview.self_declared_cleanup.as_ref().expect("nhanh tu khai phai co khoi lam sach");

    assert!(cleanup.window_truncated, "tien de: nguon phai dai hon cua so bang chung");
    let visible_chars = cleanup.text.chars().count();
    assert_eq!(
        visible_chars, EXPECTED_VISIBLE_CHARS,
        "cua so hien thi phai cat dung diem da tinh -- neu con so nay lech, ca duoi khong \
         con kiem dung cho vat bien nua"
    );
    assert!(
        cleanup.text.ends_with("TAIL"),
        "cua so hien thi phai ket thuc dung giua cho khop (sau TAIL, truoc \\n+HEAD): {:?}",
        &cleanup.text[cleanup.text.len().saturating_sub(20)..]
    );

    assert_eq!(cleanup.rules[0].count_in_chapter, 1, "cho khop van duoc DEM du");

    assert_eq!(
        cleanup.spans.len(),
        1,
        "cho khop vat bien phai VAN co mat trong spans, khong bi loai han"
    );
    let span = cleanup.spans[0];
    assert_eq!(span.start, 3895, "diem bat dau cho khop khong doi");
    assert_eq!(
        span.end, visible_chars,
        "diem ket thuc phai CAT VE dung bien cua so hien thi (khong phai 3904, vi tri that \
         cua cho khop trong TOAN Chuong) -- gach ngang chi ve duoc phan dang hien"
    );
    assert!(span.end < 3904, "phai la mot phep CAT that, khong phai tinh co giu nguyen 3904");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bàn đo — chi phí CPU của SÁU lượt chạy chuỗi thật trên TOÀN văn bản
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 ĐO, ĐỪNG KHAI (vòng rà 2026-09-06). `cleanup_preview_for` (`commands/project.rs`) nay
// chạy TRỌN chuỗi bảy bước trên TOÀN văn bản — một lần cho MỖI ứng viên (năm ô FR126) CỘNG
// một lần cho đường tự khai, tức TỐI ĐA SÁU lượt `run_pipeline` trên cùng một Chương ở MỘT
// lượt mở màn xem trước. Doc-comment của `cleanup_preview_for` từng khẳng định "CPU của
// `regex`/`normalize` rẻ" mà KHÔNG kèm một con số — đúng thứ kho này cấm (một mệnh đề hiệu
// năng không phép đo). Bàn đo dưới đây đo THẬT, trên một Chương LỚN dựng tay (nghiêng về
// biên trên: 300.000 ký tự tiếng Trung trộn ASCII, ~900 KB — vượt xa một chương tiểu thuyết
// thật, xem `deferred-work.md`: chương thật lớn nhất đã thấy chỉ 351 ký tự) cộng năm luật
// làm sạch (ba literal, hai regex) để mô phỏng một bộ luật đã dùng lâu ngày.
#[test]
fn perf_probe_six_full_pipeline_runs_on_one_large_chapter() {
    let root = temp_dir("perf-probe-six-runs");
    let global = open_global(&root);

    // Năm luật — ba literal, hai regex (mô phỏng một bộ luật thật, không phải 0/1 luật).
    for (pattern, kind) in [
        ("QUANGCAO", CleanupRuleKind::Literal),
        ("求收藏", CleanupRuleKind::Literal),
        ("本章由.*整理", CleanupRuleKind::Literal),
        ("^本章由.*整理$", CleanupRuleKind::Regex),
        ("[慕容][一二三四五]+", CleanupRuleKind::Regex),
    ] {
        cleanup_add_rule(Some(&global), None, CleanupRuleTier::Global, pattern, kind)
            .expect("them luat do that bai");
    }
    let rules =
        auratranslate_lib::core::cleanup::resolve_two_tiers(&ScopeResolver::global_only(), &global, None)
            .expect("phan giai hai tang");

    // Một "cau" tieng Trung lap lai nhieu lan cho toi mot Chuong THAT LON (~900 KB).
    const SENTENCE: &str = "萧炎缓缓睁开双眼，望向远方的天际，心中涌起一股莫名的波动。";
    let mut text = String::with_capacity(SENTENCE.len() * 5_000);
    for _ in 0..5_000 {
        text.push_str(SENTENCE);
        text.push('\n');
    }
    let chapter_bytes = text.len();

    // Đường TỰ KHAI (1 lượt `run_pipeline` trên TOÀN văn bản).
    let shape_self_declared = PipelineShape::Blob(ChapterInput::AlreadyText(text.clone()));
    let t0 = std::time::Instant::now();
    let preview_self = preview_import_encoding(&shape_self_declared, "zh", &rules, None, &[]);
    let self_declared_elapsed = t0.elapsed();
    assert!(preview_self.self_declared_cleanup.is_some(), "tien de: nhanh tu khai phai co khoi");

    // Đường ỨNG VIÊN (5 lượt `run_pipeline` trên TOÀN văn bản, một cho mỗi ô FR126).
    let shape_candidates =
        PipelineShape::Blob(ChapterInput::RawBytes { bytes: text.into_bytes(), label: String::new() });
    let t1 = std::time::Instant::now();
    let preview_candidates = preview_import_encoding(&shape_candidates, "zh", &rules, None, &[]);
    let candidates_elapsed = t1.elapsed();
    assert_eq!(preview_candidates.candidates.len(), 5, "tien de: du nam o FR126");

    eprintln!(
        "[perf_probe_six_full_pipeline_runs_on_one_large_chapter] Chuong {chapter_bytes} byte, \
         5 luat (3 literal + 2 regex) — đường tự khai (1 lượt run_pipeline): {self_declared_elapsed:?}; \
         đường 5 ứng viên (5 lượt run_pipeline): {candidates_elapsed:?}; \
         trung bình MỖI lượt run_pipeline: {:?}",
        candidates_elapsed / 5
    );

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.6 — đóng MỘT PHẦN nợ deferred-work.md:9535: count_in_import == Σ count_in_chapter,
// ĐÚNG BẰNG tổng tính tay, khi N ≥ 2 Chương với số khớp KHÁC NHAU mỗi Chương
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 Vì sao dựng qua `PipelineShape::Chapters`, không qua mẫu phân tách của CHÍNH story
// này: bước 3 (`CleanByRules`) đứng TRƯỚC bước 5 (`SplitChapters`) trong `PIPELINE_ORDER`
// (không đổi — §Always spec 6.6), nên khi N Chương đến từ VIỆC TÁCH một `Blob`, luật làm
// sạch đã chạy XONG trên TOÀN blob dưới dạng MỘT đơn vị trước khi Chương nào tồn tại — không
// có báo cáo RIÊNG cho từng Chương kết quả để mà cộng khác nhau (xem doc-comment
// `commands::project::cleanup_and_chapters_preview_for`, mục "GIỚI HẠN THẬT"). Với
// `PipelineShape::Chapters` (N đơn vị NGAY TỪ ĐẦU — danh sách URL Story 6.7, hoặc bất kỳ
// nguồn nào tự cấp N Chương), `Step::CleanByRules` gọi `cleanup::apply` MỘT LẦN CHO MỖI đơn
// vị (cùng một chỗ gọi nguồn, chạy N lần) — mỗi Chương có báo cáo THẬT của riêng nó.
// 🟡 **Đây là đóng MỘT PHẦN, không trọn vẹn** — `PipelineShape::Chapters` CHƯA có đường sản
// phẩm nào dựng ra hôm nay (Story 6.7, danh sách URL, sẽ là đường ĐẦU TIÊN); đường sản phẩm
// THẬT của CHÍNH story 6.6 (`Blob` + `chapter_pattern`) vẫn cho `count_in_chapter ==
// count_in_import` LUÔN — xem mục nợ MỚI ở `deferred-work.md` (Chủ: Ice) cho lý do kiến
// trúc, và phần còn hở của chính mục `:9535` (Chủ: Story 6.7).
/// 🔴 SỬA (vòng nghiệm thu 2026-09-06) — bản đầu của ca này tự cộng `per_rule_counts` NGAY
/// TRONG chính ca test rồi so với số tính tay: nó khẳng định phép cộng CỦA CHÍNH CA TEST
/// đúng, không khẳng định phép cộng mà SẢN PHẨM (`commands::project::cleanup_and_chapters_preview_for`)
/// làm ra. Đối chứng đo được: thay dòng gán `count_in_import` trong hàm đó bằng
/// `count_in_chapter` (tái tạo NGUYÊN VĂN khuyết tật mà `deferred-work.md:9535` mô tả) rồi
/// chạy `cargo test --locked` — bản test cũ **vẫn xanh** trên một sản phẩm đang hỏng, vì nó
/// không gọi tới hàm đó một lần nào. Ca này gọi THẲNG `cleanup_and_chapters_preview_for` (hàm
/// `pub`, cùng khuôn hai lớp `resolve_chapter_pattern`) và đọc `count_in_import` TỪ
/// `CleanupRuleReportWire` mà nó trả về — cùng phép đột biến trên (đổi `:1451` cũ) làm ca
/// NÀY đỏ (đã tự kiểm tay trước khi nộp, xem chú thích ở dưới).
#[test]
fn count_in_import_equals_the_hand_counted_sum_of_count_in_chapter_across_n_chapters_with_different_match_counts()
 {
    let rule = CleanupRule {
        tier: CleanupRuleTier::Global,
        id: 1,
        pattern: "QUANGCAO".to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    };
    // Chương 1: đúng HAI chỗ khớp. Chương 2: đúng BA chỗ khớp. Chương 3: KHÔNG chỗ nào khớp
    // (0 là một số THẬT, không phải một Chương bị bỏ sót khỏi phép cộng).
    let chapter_1 = "QUANGCAO dau. noi dung. QUANGCAO cuoi.".to_owned();
    let chapter_2 = "QUANGCAO mot. QUANGCAO hai. QUANGCAO ba.".to_owned();
    let chapter_3 = "khong co gi de xoa o day ca.".to_owned();
    let hand_counted_total = 2 + 3 + 0;

    // Đối chứng độc lập ở tầng PIPELINE — chứng minh MỖI Chương thật sự mang báo cáo RIÊNG,
    // số khớp khác nhau thật (điều kiện để phép cộng ở tầng dây có gì đó THẬT để mà cộng).
    let shape_for_pipeline_check = PipelineShape::Chapters(vec![
        ChapterInput::AlreadyText(chapter_1.clone()),
        ChapterInput::AlreadyText(chapter_2.clone()),
        ChapterInput::AlreadyText(chapter_3.clone()),
    ]);
    let input = PipelineInput::default_shaped(shape_for_pipeline_check, "en")
        .with_cleanup_rules(vec![rule.clone()]);
    let outcome = run_import(input).expect("chuoi voi PipelineShape::Chapters khong duoc loi");
    assert_eq!(outcome.chapters.len(), 3, "tien de: dung ba Chuong, khong bi gop/tach lai");
    let key = (rule.tier, rule.id);
    let per_chapter_counts: Vec<usize> = outcome
        .chapters
        .iter()
        .map(|c| {
            c.cleanup_report
                .as_ref()
                .expect(
                    "PipelineShape::Chapters phai cho MOI Chuong mot bao cao RIENG -- \
                     Step::CleanByRules goi apply() mot lan cho MOI don vi da co tu dau",
                )
                .per_rule_counts
                .get(&key)
                .copied()
                .unwrap_or(0)
        })
        .collect();
    assert_eq!(
        per_chapter_counts,
        vec![2, 3, 0],
        "moi Chuong phai mang so khop CUA RIENG NO, khac nhau that su -- khong phai ba lan \
         cung mot con so trung hop"
    );

    // Đối chứng THẬT — gọi thẳng hàm SẢN PHẨM sinh `count_in_import` trên dây, KHÔNG tự cộng
    // trong ca test. Đây là chỗ đột biến `count_in_import = count_in_chapter` phải làm ĐỎ.
    let shape_for_wire = PipelineShape::Chapters(vec![
        ChapterInput::AlreadyText(chapter_1.clone()),
        ChapterInput::AlreadyText(chapter_2),
        ChapterInput::AlreadyText(chapter_3),
    ]);
    let (cleanup_wire, _chapters_wire, _blocks_wire) = cleanup_and_chapters_preview_for(
        shape_for_wire,
        encoding_rs::UTF_8,
        None,
        &chapter_1,
        "en",
        &[rule.clone()],
        false,
        false,
        &[],
    );

    assert_eq!(cleanup_wire.rules.len(), 1, "dung mot luat duoc gieo");
    let rule_wire = &cleanup_wire.rules[0];
    assert_eq!(
        rule_wire.count_in_chapter, 2,
        "count_in_chapter phai la so khop CUA CHUONG DANG HIEN (Chuong 1, 2 cho khop)"
    );
    assert_eq!(
        rule_wire.count_in_import, hand_counted_total,
        "count_in_import (do CHINH commands::project::cleanup_and_chapters_preview_for tinh, \
         khong phai ca test tu cong) phai DUNG BANG tong tinh tay -- dong no deferred-work.md:9535"
    );
    assert_ne!(
        rule_wire.count_in_chapter, rule_wire.count_in_import,
        "hai so nay phai THAT SU khac nhau o day -- neu bang nhau, ca nay khong chung minh \
         duoc gi ve phep CONG, chi chung minh duoc mot phep sao chep"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.6 — "xem trước = xác nhận" ở QUY MÔ N Chương (mẫu phân tách), khuôn hàng 10
// (`preview_and_confirm_agree_byte_for_byte_on_the_same_input_and_the_same_rules`)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn preview_and_confirm_agree_byte_for_byte_when_a_chapter_pattern_yields_n_chapters() {
    use auratranslate_lib::core::segment::chapterpattern::ChapterPattern;

    let root = temp_dir("preview-confirm-agree-n-chapters");
    let global = open_global(&root);
    cleanup_add_rule(
        Some(&global),
        None,
        CleanupRuleTier::Global,
        "quang cao",
        CleanupRuleKind::Literal,
    )
    .expect("them luat that bai");
    let rules = auratranslate_lib::core::cleanup::resolve_two_tiers(
        &ScopeResolver::global_only(),
        &global,
        None,
    )
    .expect("phan giai hai tang");

    let text = "Chuong 1: Mo Dau\n\nquang cao dau. noi dung mot.\n\nChuong 2: Tiep Theo\n\nnoi dung hai. quang cao cuoi.".to_owned();
    let pattern = ChapterPattern::regex(r"^Chuong \d+:.*$");

    // Tính ĐỘC LẬP (không qua `commands::project`) văn bản mà chuỗi pipeline THẬT sẽ tạo ra
    // cho CÙNG đầu vào — đây là "sự thật" mà cả preview lẫn confirm phải khớp.
    let expected = {
        let shape = PipelineShape::Blob(ChapterInput::AlreadyText(text.clone()));
        let input = PipelineInput::default_shaped(shape, "en")
            .with_cleanup_rules(rules.clone())
            .with_chapter_pattern(Some(pattern.clone()));
        run_import(input).expect("chuoi doc lap khong duoc loi")
    };
    assert_eq!(expected.chapters.len(), 2, "tien de: mau phai tach ra dung hai Chuong");

    let shape = PipelineShape::Blob(ChapterInput::AlreadyText(text));
    let state: PendingImportSourceState = std::sync::Mutex::new(None);
    stash_pending_import_source(&state, shape);
    let opened = confirm_import_with_encoding(
        &root,
        &state,
        "Preview Confirm N Chuong",
        "en",
        "",
        "UTF-8",
        rules,
        Some(pattern),
        Vec::new(),
    )
    .expect("xac nhan that bai");

    let written: Vec<(i64, String)> = opened
        .store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT ord, source_text FROM chapter ORDER BY ord")?;
            let mut rows_iter = stmt.query([])?;
            let mut out = Vec::new();
            while let Some(row) = rows_iter.next()? {
                out.push((row.get::<_, i64>(0)?, row.get::<_, String>(1)?));
            }
            Ok(out)
        })
        .expect("doc lai chapter that bai");

    assert_eq!(written.len(), expected.chapters.len());
    for (i, (ord, source_text)) in written.iter().enumerate() {
        assert_eq!(*ord, i as i64 + 1);
        assert_eq!(
            source_text, &expected.chapters[i].source_text,
            "Chuong thu {i} ghi xuong phai giong HET TUNG BYTE voi chuoi pipeline doc lap -- \
             xem truoc va xac nhan phai cung chay run_pipeline tren CUNG dau vao"
        );
    }

    drop(opened.store);
    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.6 — ĐO, ĐỪNG KHAI: xem trước nay chạy chuỗi kèm tách Chương (Task list spec 6.6)
// ═════════════════════════════════════════════════════════════════════════════════

/// Đo chi phí thêm của khối tách Chương (tầng 4) trên một nguồn NHIỀU CHƯƠNG THẬT — mốc so
/// sánh trước story (Story 6.5, `perf_probe_six_full_pipeline_runs_on_one_large_chapter`):
/// 6 lượt `run_pipeline`, ~13-17 ms/lượt trên MỘT Chương 440 KB (`project.rs:1162-1178` cũ).
/// Ca này dựng 2.000 Chương (trần nêu trong I/O Matrix spec 6.6, "Xác nhận N Chương") bằng
/// một mẫu literal khớp đúng 2.000 lần, TOÀN VĂN BẢN cỡ tương đương (~440 KB), rồi đo đường
/// tự khai (1 lượt `run_pipeline`, giờ CỘNG THÊM việc dựng `ChapterSplitPreviewWire` cho cả
/// 2.000 Chương).
#[test]
fn perf_probe_chapter_split_preview_on_two_thousand_chapters() {
    let root = temp_dir("perf-probe-2000-chapters");
    let global = open_global(&root);
    cleanup_add_rule(Some(&global), None, CleanupRuleTier::Global, "QUANGCAO", CleanupRuleKind::Literal)
        .expect("them luat do that bai");
    let rules =
        auratranslate_lib::core::cleanup::resolve_two_tiers(&ScopeResolver::global_only(), &global, None)
            .expect("phan giai hai tang");

    const CHAPTER_COUNT: usize = 2_000;
    let mut text = String::new();
    for i in 0..CHAPTER_COUNT {
        text.push_str(&format!("Chuong {i}: Tieu De\n\nnoi dung ngan cua chuong nay. QUANGCAO.\n\n"));
    }
    let source_bytes = text.len();

    let pattern = auratranslate_lib::core::segment::chapterpattern::ChapterPattern::regex(
        r"^Chuong \d+:.*$",
    );
    let shape = PipelineShape::Blob(ChapterInput::AlreadyText(text));
    let t0 = std::time::Instant::now();
    let preview = preview_import_encoding(&shape, "en", &rules, Some(&pattern), &[]);
    let elapsed = t0.elapsed();

    let chapters =
        preview.self_declared_chapters.as_ref().expect("nhanh tu khai phai co khoi tach Chuong");
    assert_eq!(
        chapters.chapter_count, CHAPTER_COUNT,
        "tien de: mau phai khop dung {CHAPTER_COUNT} lan"
    );

    eprintln!(
        "[perf_probe_chapter_split_preview_on_two_thousand_chapters] nguon {source_bytes} byte, \
         {CHAPTER_COUNT} Chuong, 1 luat literal — đường tự khai (1 lượt run_pipeline + dựng khối \
         tách {CHAPTER_COUNT} Chương): {elapsed:?}"
    );

    drop(global);
    cleanup_dir(&root);
}

/// 🔴 SỬA (vòng rà đối kháng 3, mục 7) — ca NGAY TRÊN chỉ đo đường TỰ KHAI (`AlreadyText`,
/// ĐÚNG MỘT lượt `run_pipeline`). Đường FR126 THẬT (`RawBytes`, dò bảng mã) gọi
/// `encoding_candidate_wire` → `cleanup_and_chapters_preview_for` → MỘT lượt `run_pipeline`
/// **CHO MỖI ứng viên trong NĂM** — cùng khối lượng việc (2.000 Chương, một luật literal) chạy
/// tối đa NĂM LẦN mỗi lượt tải màn xem trước, không phải MỘT lần. Doc-comment cũ suy "45 ms
/// vẫn dưới một phần mười giây" chỉ từ số đo MỘT lượt — không suy tuyến tính (Ice cấm): ca
/// này đo THẲNG đường năm ứng viên trên CÙNG khối lượng để có con số THẬT, không suy diễn.
#[test]
fn perf_probe_chapter_split_preview_on_five_candidates_with_two_thousand_chapters() {
    let root = temp_dir("perf-probe-2000-chapters-five-candidates");
    let global = open_global(&root);
    cleanup_add_rule(Some(&global), None, CleanupRuleTier::Global, "QUANGCAO", CleanupRuleKind::Literal)
        .expect("them luat do that bai");
    let rules =
        auratranslate_lib::core::cleanup::resolve_two_tiers(&ScopeResolver::global_only(), &global, None)
            .expect("phan giai hai tang");

    const CHAPTER_COUNT: usize = 2_000;
    let mut text = String::new();
    for i in 0..CHAPTER_COUNT {
        text.push_str(&format!("Chuong {i}: Tieu De\n\nnoi dung ngan cua chuong nay. QUANGCAO.\n\n"));
    }
    let bytes = text.into_bytes();
    let source_bytes = bytes.len();

    let pattern = auratranslate_lib::core::segment::chapterpattern::ChapterPattern::regex(
        r"^Chuong \d+:.*$",
    );
    // `RawBytes` (khác ca ngay trên dùng `AlreadyText`) là đường THẬT kích hoạt dò bảng mã
    // (`encoding::detect` + `render_candidates`) — luôn cho đủ NĂM ứng viên khi có byte để dò
    // (doc-comment `preview_import_encoding`), mỗi ứng viên đi qua `cleanup_and_chapters_preview_for`
    // của CHÍNH NÓ.
    let shape = PipelineShape::Blob(ChapterInput::RawBytes {
        bytes,
        label: "perf-5-candidates.txt".to_owned(),
    });
    let t0 = std::time::Instant::now();
    let preview = preview_import_encoding(&shape, "en", &rules, Some(&pattern), &[]);
    let elapsed = t0.elapsed();

    assert_eq!(
        preview.candidates.len(),
        5,
        "co byte de do -- FR126 phai cho DUNG NAM ung vien, khong duoc it hon"
    );
    // Không đòi HẾT NĂM ứng viên đều khớp đủ 2.000 lần — một bảng mã SAI giải mã byte UTF-8
    // thành văn bản khác (ký tự khác, dòng khác) không có nghĩa vụ khớp lại đúng mẫu. Chỉ đòi
    // ÍT NHẤT MỘT (ứng viên giải mã ĐÚNG) tái lập đúng tiền đề của ca so sánh — cùng khối
    // lượng việc với `perf_probe_chapter_split_preview_on_two_thousand_chapters`.
    assert!(
        preview
            .candidates
            .iter()
            .any(|c| c.chapters.as_ref().is_some_and(|ch| ch.chapter_count == CHAPTER_COUNT)),
        "it nhat MOT trong nam ung vien (ung vien giai ma DUNG) phai khop du {CHAPTER_COUNT} lan"
    );

    eprintln!(
        "[perf_probe_chapter_split_preview_on_five_candidates_with_two_thousand_chapters] nguon \
         {source_bytes} byte, {CHAPTER_COUNT} Chuong, 1 luat literal — đường NĂM ứng viên (tối đa \
         5 lượt run_pipeline + dựng khối tách {CHAPTER_COUNT} Chương MỖI ứng viên): {elapsed:?}"
    );

    drop(global);
    cleanup_dir(&root);
}
