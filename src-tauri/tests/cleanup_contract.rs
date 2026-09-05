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
    OpenWork, PendingImportSourceState, confirm_import_with_encoding, create_work,
    preview_import_encoding, stash_pending_import_source,
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
    let preview_on = preview_import_encoding(&shape, "en", &rules_on);
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
    let preview_off = preview_import_encoding(&shape, "en", &rules_off);
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
    let preview = preview_import_encoding(&shape, "en", &rules);

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

    let preview = preview_import_encoding(&shape, "en", &rules);
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
    let preview = preview_import_encoding(&shape, "en", &rules);
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
    let preview = preview_import_encoding(&shape, "en", &rules);
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
    let preview_self = preview_import_encoding(&shape_self_declared, "zh", &rules);
    let self_declared_elapsed = t0.elapsed();
    assert!(preview_self.self_declared_cleanup.is_some(), "tien de: nhanh tu khai phai co khoi");

    // Đường ỨNG VIÊN (5 lượt `run_pipeline` trên TOÀN văn bản, một cho mỗi ô FR126).
    let shape_candidates =
        PipelineShape::Blob(ChapterInput::RawBytes { bytes: text.into_bytes(), label: String::new() });
    let t1 = std::time::Instant::now();
    let preview_candidates = preview_import_encoding(&shape_candidates, "zh", &rules);
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
