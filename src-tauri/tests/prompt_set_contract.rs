//! Hành vi hai tầng của bộ prompt theo thể loại — Story 4.4, FR69, `core/scope/kinds.rs:165`
//! (`ScopeKind::Prompt => "prompt" : Semantics::Override`) — một ca cho MỖI hàng của I/O &
//! Edge-Case Matrix (`spec-4-4-bo-prompt-theo-the-loai.md`), ở tầng LỆNH
//! (`commands::promptset`), đúng khuôn `aiconfig_contract.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — thừa kế nguyên vẹn từ `aiconfig_contract.rs`/`cleanup_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + `AtomicU64`).
//! 2. **Drop `Store`/`OpenWork` TRƯỚC khi xoá thư mục** — Windows từ chối xoá tệp đang mở.
//! 3. Không `sleep` dài.
//! 4. Không ca nào treo khi nó trượt.
//!
//! ⚠️ Gọi thẳng NĂM hàm thuần `commands::promptset::prompt_set_*`, không đi qua `wire::` —
//! cùng lý do `ipc_contract.rs::the_prompt_set_wires_are_registered`'s doc-comment ghi rõ:
//! tệp này canh HÀNH VI domain, cổng kia (đã có sẵn từ Phase 2) canh DÂY ĐĂNG KÝ. Xoá một
//! dòng `wire::prompt_set_*` khỏi `generate_handler!` không làm một ca nào ở đây đỏ — đó là
//! counter-check RIÊNG của `ipc_contract.rs`, chạy lại ở cuối phiên này, không nhân đôi ở đây.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{OpenWork, create_work};
use auratranslate_lib::commands::promptset::{
    PromptSetListWire, PromptSetTierWire, PromptSetWire, prompt_set_create, prompt_set_delete,
    prompt_set_list, prompt_set_rename, prompt_set_update_body,
};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::promptset::{PromptSetTier, PromptVariable};
use auratranslate_lib::core::segment::pipeline::{ChapterInput, PipelineShape};
use auratranslate_lib::core::store::{
    GLOSSARY_ENTRY_DDL, IMPORT_CLEANUP_RULE_DDL, PROMPT_SET_DDL, Store, StoreSpec,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-promptset-{}-{}-{}",
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

/// Một `OpenWork` THẬT — chỉ để các hàm thuần của `commands::promptset` có một tầng Work
/// mà định tuyến, không cần văn bản gì đặc biệt. Khuôn chép nguyên văn từ
/// `aiconfig_contract.rs::open_work_real`.
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
        0,
        1,
        false,
        &[],
        &std::sync::Mutex::new(Vec::new()),
        None,
        &[],
    )
    .expect("tao OpenWork that bai")
}

fn find_set<'a>(list: &'a PromptSetListWire, name: &str) -> &'a PromptSetWire {
    list.sets
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("khong thay bo ten {name:?} trong ket qua prompt_set_list"))
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 1 — Create at Global, không Work nào mở
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn create_at_global_with_no_work_open_lists_the_new_set_under_the_global_tier() {
    let root = temp_dir("create-global");
    let global = open_global(&root);

    let created = prompt_set_create(Some(&global), None, PromptSetTier::Global, "Xianxia", "than toan cuc")
        .expect("tao bo Global that bai");
    assert!(created.warnings.unknown_markers.is_empty());
    assert!(created.warnings.glossary_terms_missing, "than khong mang {{{{glossary_terms}}}}");

    let list = prompt_set_list(Some(&global), None).expect("liet ke that bai");
    let set = find_set(&list, "Xianxia");
    assert_eq!(set.id, created.id);
    assert_eq!(set.tier, PromptSetTierWire::Global);
    assert_eq!(set.body, "than toan cuc");
    assert_eq!(set.shadowed_body, None);

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 2 — Create at Work, tên đã có ở Global: Work thắng, Global bị che
// ═════════════════════════════════════════════════════════════════════════════════

/// Đối chứng trực tiếp cho AC1: xoá lời gọi `resolver.apply_override(...)` khỏi
/// `core::promptset::store::resolve_two_tiers` làm ca này đỏ — `shadowed_body` sẽ mất
/// (hoặc cả hai hàng cùng hiện, không hàng nào bị che).
#[test]
fn create_at_work_when_the_name_already_exists_at_global_makes_work_win_and_shadows_global() {
    let root = temp_dir("create-work-shadow");
    let global = open_global(&root);
    let opened = open_work_real(&root);

    prompt_set_create(Some(&global), None, PromptSetTier::Global, "Fantasy", "than toan cuc")
        .expect("tao bo Global that bai");
    let created_work = prompt_set_create(
        Some(&global),
        Some(&opened),
        PromptSetTier::Work,
        "Fantasy",
        "than tac pham",
    )
    .expect("tao bo Work that bai");

    let list = prompt_set_list(Some(&global), Some(&opened)).expect("liet ke that bai");
    assert_eq!(list.sets.len(), 1, "cung TEN o hai tang phai gop thanh MOT hang da phan giai");
    let set = find_set(&list, "Fantasy");
    assert_eq!(set.id, created_work.id, "hang thang phai la hang Work");
    assert_eq!(set.tier, PromptSetTierWire::Work);
    assert_eq!(set.body, "than tac pham");
    assert_eq!(
        set.shadowed_body.as_deref(),
        Some("than toan cuc"),
        "than Global bi che phai con nguyen o shadowed_body"
    );

    drop(opened.store);
    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 3 — Tên rỗng/toàn khoảng trắng: từ chối TRƯỚC khi cham SQL
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn blank_ish_names_are_rejected_before_any_write_and_leave_the_tier_empty() {
    let root = temp_dir("blank-name");
    let global = open_global(&root);

    for bad in ["\u{3000}", "\t", "", "   "] {
        let err = prompt_set_create(Some(&global), None, PromptSetTier::Global, bad, "than")
            .err()
            .unwrap_or_else(|| panic!("ten {bad:?} phai bi tu choi"));
        assert_eq!(err.message_key(), MessageKey::PromptSetInvalidName);
    }

    let list = prompt_set_list(Some(&global), None).expect("liet ke that bai");
    assert!(list.sets.is_empty(), "0 luot ghi duoc phep xay ra");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 4 — Trùng tên TRONG CÙNG tầng: từ chối, hàng đang có KHÔNG bị đụng tới
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn duplicate_name_in_the_same_tier_is_rejected_and_the_existing_row_is_untouched() {
    let root = temp_dir("duplicate-name");
    let global = open_global(&root);

    prompt_set_create(Some(&global), None, PromptSetTier::Global, "Xianxia", "ban goc")
        .expect("tao lan dau that bai");

    let err = prompt_set_create(Some(&global), None, PromptSetTier::Global, "Xianxia", "ban gia")
        .err()
        .expect("ten trung phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::PromptSetNameTaken);
    assert_eq!(err.params().get("name").map(String::as_str), Some("Xianxia"));

    let list = prompt_set_list(Some(&global), None).expect("liet ke that bai");
    assert_eq!(list.sets.len(), 1, "khong hang thu hai nao duoc tao");
    assert_eq!(find_set(&list, "Xianxia").body, "ban goc", "hang dang co KHONG bi dung toi");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 5 — Đổi tên trùng một tên đã có: từ chối, CẢ HAI hàng không bị đụng tới
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn renaming_to_an_existing_name_in_the_same_tier_is_rejected_and_both_rows_are_untouched() {
    let root = temp_dir("rename-collision");
    let global = open_global(&root);

    prompt_set_create(Some(&global), None, PromptSetTier::Global, "A", "a-body")
        .expect("tao A that bai");
    let b = prompt_set_create(Some(&global), None, PromptSetTier::Global, "B", "b-body")
        .expect("tao B that bai");

    let err = prompt_set_rename(Some(&global), None, PromptSetTier::Global, b.id, "A")
        .err()
        .expect("doi ten trung phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::PromptSetNameTaken);
    assert_eq!(err.params().get("name").map(String::as_str), Some("A"));

    let list = prompt_set_list(Some(&global), None).expect("liet ke that bai");
    assert_eq!(list.sets.len(), 2);
    assert_eq!(find_set(&list, "A").body, "a-body");
    assert_eq!(find_set(&list, "B").body, "b-body", "hang B van giu TEN CU, giao dich rollback tron");

    drop(global);
    cleanup_dir(&root);
}

/// Không phải một hàng của I/O Matrix — Phase 1's handoff notes flag đây là "untested" và đề
/// nghị đóng ở Phase 3 ("Worth one contract-test case ... if the compose screen's rename flow
/// can hit it, e.g. open-rename-dialog-then-save-without-editing"). `UNIQUE` của SQLite không
/// nổ khi giá trị MỚI trùng giá trị CŨ của CHÍNH hàng đang sửa — đóng lại ở đây làm hành vi
/// này thành quyết định đã kiểm, không phải một giả định chưa đo.
#[test]
fn renaming_a_set_to_its_own_current_name_is_a_no_op_success() {
    let root = temp_dir("rename-self");
    let global = open_global(&root);

    let created = prompt_set_create(Some(&global), None, PromptSetTier::Global, "Xianxia", "than")
        .expect("tao that bai");

    prompt_set_rename(Some(&global), None, PromptSetTier::Global, created.id, "Xianxia")
        .expect("doi ten ve chinh ten cu phai THANH CONG, khong duoc va UNIQUE voi hang cua chinh no");

    let list = prompt_set_list(Some(&global), None).expect("liet ke that bai");
    assert_eq!(list.sets.len(), 1);
    assert_eq!(find_set(&list, "Xianxia").body, "than");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 6 — Xoá một bộ Work đang che một bộ Global: bộ Global hiệu lực trở lại
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn deleting_a_shadowing_work_set_makes_the_global_set_effective_again() {
    let root = temp_dir("delete-shadow");
    let global = open_global(&root);
    let opened = open_work_real(&root);

    prompt_set_create(Some(&global), None, PromptSetTier::Global, "Fantasy", "than toan cuc")
        .expect("tao Global that bai");
    let work_set = prompt_set_create(
        Some(&global),
        Some(&opened),
        PromptSetTier::Work,
        "Fantasy",
        "than tac pham",
    )
    .expect("tao Work that bai");

    let before = prompt_set_list(Some(&global), Some(&opened)).expect("liet ke truoc khi xoa");
    assert_eq!(find_set(&before, "Fantasy").tier, PromptSetTierWire::Work);

    prompt_set_delete(Some(&global), Some(&opened), PromptSetTier::Work, work_set.id)
        .expect("xoa bo Work that bai");

    let after = prompt_set_list(Some(&global), Some(&opened)).expect("liet ke sau khi xoa");
    let set = find_set(&after, "Fantasy");
    assert_eq!(set.tier, PromptSetTierWire::Global, "bo Global phai hieu luc tro lai NGAY, khong can thao tac rieng");
    assert_eq!(set.body, "than toan cuc");
    assert_eq!(set.shadowed_body, None, "khong con gi de che nua");

    drop(opened.store);
    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 7 — Liệt kê khi không Work nào mở: chỉ hàng Global; nhóm Work VẮNG MẶT
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn listing_with_no_work_open_returns_global_rows_only_and_the_work_group_is_absent() {
    let root = temp_dir("list-no-work");
    let global = open_global(&root);
    prompt_set_create(Some(&global), None, PromptSetTier::Global, "Solo", "than").expect("tao that bai");

    let list = prompt_set_list(Some(&global), None).expect("liet ke that bai");
    assert!(!list.work_tier_available, "khong Tac pham nao mo ⇒ work_tier_available phai false");
    assert_eq!(list.sets.len(), 1);
    assert_eq!(find_set(&list, "Solo").tier, PromptSetTierWire::Global);

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 8 — Liệt kê khi cả hai tầng rỗng: trạng thái RỖNG, không phải lỗi
// ═════════════════════════════════════════════════════════════════════════════════

/// Đối chứng trực tiếp với hàng 7 ngay trên: CẢ HAI ca đều trả `sets` rỗng-hoặc-nhỏ, nhưng
/// `work_tier_available` phân biệt "chưa từng hỏi tầng Work" (hàng 7, `false`) với "đã hỏi
/// CẢ HAI tầng, không tầng nào có gì" (hàng này, `true`) — đúng luật AGENTS.md "một giá trị
/// có thể UNKNOWN nhận Option/NULL, không bao giờ một 0 mang hai nghĩa".
#[test]
fn listing_with_zero_sets_in_either_tier_is_an_empty_list_not_an_error() {
    let root = temp_dir("list-zero");
    let global = open_global(&root);
    let opened = open_work_real(&root);

    let list = prompt_set_list(Some(&global), Some(&opened)).expect("liet ke that bai");
    assert!(list.work_tier_available, "co Tac pham mo, du no khong bo prompt nao");
    assert!(list.sets.is_empty(), "hai tang rong phai la Vec rong, khong phai loi");

    drop(opened.store);
    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 9 — Chuyển bộ hiệu lực từ panel AI: một lượt `prompt_set_list` là đủ cho cả hai bộ
// ═════════════════════════════════════════════════════════════════════════════════

/// Domain này không có khái niệm "bộ đang được chọn" ở tầng Rust — không bảng, không cột nào
/// giữ một lựa chọn hiện hành (xem Code Map: bộ chuyển đổi là việc của panel, Phase 4b). Cái
/// mà tầng Rust phải bảo đảm là: MỘT lượt `prompt_set_list` mang đủ dữ liệu (tên + thân +
/// tầng + hàng bị che) của MỌI bộ khả dụng, để bộ chuyển đổi không cần mở lại Cài đặt hay gọi
/// thêm lệnh nào khác khi người dùng đổi lựa chọn. Ca này dựng đúng tình huống matrix mô tả —
/// Work mở, hai bộ khả dụng — và xác nhận cả hai đọc đúng từ CÙNG một lượt gọi.
#[test]
fn switching_between_two_resolvable_sets_needs_no_settings_reopen() {
    let root = temp_dir("switch-sets");
    let global = open_global(&root);
    let opened = open_work_real(&root);

    prompt_set_create(Some(&global), None, PromptSetTier::Global, "Xianxia", "a-global")
        .expect("tao Xianxia Global that bai");
    prompt_set_create(Some(&global), Some(&opened), PromptSetTier::Work, "Xianxia", "a-work")
        .expect("tao Xianxia Work that bai");
    prompt_set_create(Some(&global), None, PromptSetTier::Global, "Journalism", "b-global")
        .expect("tao Journalism that bai");

    let list = prompt_set_list(Some(&global), Some(&opened)).expect("liet ke that bai");

    let xianxia = find_set(&list, "Xianxia");
    assert_eq!(xianxia.tier, PromptSetTierWire::Work);
    assert_eq!(xianxia.body, "a-work");
    assert_eq!(xianxia.shadowed_body.as_deref(), Some("a-global"));

    let journalism = find_set(&list, "Journalism");
    assert_eq!(journalism.tier, PromptSetTierWire::Global);
    assert_eq!(journalism.body, "b-global");
    assert_eq!(journalism.shadowed_body, None);

    drop(opened.store);
    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 10 — Phân giải khi kho thiếu: báo KHÔNG SẴN SÀNG, không phải "không có bộ nào"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn resolving_when_the_global_store_is_missing_is_reported_as_unavailable_not_as_no_sets() {
    let err = prompt_set_list(None, None).err().expect("global.db vang mat phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::StoreOpenFailed);
}

/// Phase 2's handoff notes: "every prompt-set write, including a Work-tier one, requires
/// `global.db` to be managed" — hệ quả trực tiếp của chữ ký core
/// `(global: &Store, work: Option<&Store>, tier, …)` (Phase 1) mà lớp lệnh không tự chọn kho
/// theo `tier` trước khi kiểm `global`. Ghim lại đây làm một QUYẾT ĐỊNH đã kiểm, không phải
/// một tai nạn: một yêu cầu tầng Work với `global.db` vắng mặt phải báo `store.open_failed`,
/// KHÔNG `prompt_set.work_tier_unavailable` — nếu ca này đỏ với message_key khác, chữ ký core
/// đã đổi và ghi chú ở `commands/promptset.rs:38-46` cần cập nhật theo.
#[test]
fn every_prompt_set_write_including_a_work_tier_one_requires_global_db_to_be_managed() {
    let root = temp_dir("global-required-for-work-write");
    let opened = open_work_real(&root);

    let err = prompt_set_create(None, Some(&opened), PromptSetTier::Work, "Xianxia", "than")
        .err()
        .expect("global.db vang mat phai chan TRUOC ca mot yeu cau tang Work");
    assert_eq!(
        err.message_key(),
        MessageKey::StoreOpenFailed,
        "phai la store.open_failed, KHONG phai prompt_set.work_tier_unavailable -- neu la \
         work_tier_unavailable nghia la tang lenh da doi tuyen theo `tier` TRUOC khi kiem \
         `global`, khac voi chu ky core da do o Phase 1/2"
    );

    drop(opened.store);
    cleanup_dir(&root);
}

/// Đối chứng CÒN THIẾU cho `PromptSetError::WorkTierUnavailable`: bốn đường ghi
/// (`create`/`rename`/`update_body`/`delete`) đều tài liệu hoá lỗi này
/// (`core/promptset/store.rs:160,214,269,297`) nhưng trước ca này KHÔNG ca nào từng CHẠM nó —
/// ca DUY NHẤT gọi tên `work_tier_unavailable`
/// (`every_prompt_set_write_including_a_work_tier_one_requires_global_db_to_be_managed` ngay
/// trên) khẳng định NGƯỢC LẠI (`store.open_failed`), đúng cho tiền điều kiện `global.db` vắng
/// mặt. Ca này đảo tiền điều kiện: `global.db` CÓ MẶT, không Tác phẩm nào mở, `tier = Work` —
/// cùng khuôn sibling
/// `glossary_commands_contract.rs::glossary_confirm_pending_translation_at_the_work_tier_without_an_open_work_fails`.
#[test]
fn creating_at_the_work_tier_without_an_open_work_fails_with_work_tier_unavailable() {
    let root = temp_dir("work-tier-unavailable");
    let global = open_global(&root);

    let err = prompt_set_create(Some(&global), None, PromptSetTier::Work, "Xianxia", "than")
        .err()
        .expect("global.db co mat nhung khong Tac pham nao mo -- tier Work phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::PromptSetWorkTierUnavailable);

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 11 — Thân mang một token lạ: lưu NGUYÊN VĂN, cảnh báo GỌI TÊN token đó
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn saving_a_body_with_an_unknown_token_saves_it_verbatim_and_names_the_token_in_a_warning() {
    let root = temp_dir("unknown-token");
    let global = open_global(&root);

    let body = "Dung {{glosary_terms}} thay vi ten dung.";
    let created = prompt_set_create(Some(&global), None, PromptSetTier::Global, "Typo", body)
        .expect("tao that bai -- than khong bao gio bi tu choi vi dau ngoac");
    assert_eq!(created.warnings.unknown_markers, vec!["{{glosary_terms}}".to_owned()]);

    let list = prompt_set_list(Some(&global), None).expect("liet ke that bai");
    assert_eq!(find_set(&list, "Typo").body, body, "than phai duoc luu BYTE-CHO-BYTE, khong bi sua");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 12 — Thân không mang `{{glossary_terms}}`: lưu, cảnh báo Glossary Enforcement tắt
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn saving_a_body_with_no_glossary_terms_marker_saves_and_warns_enforcement_is_off() {
    let root = temp_dir("missing-glossary-terms");
    let global = open_global(&root);

    let created = prompt_set_create(
        Some(&global),
        None,
        PromptSetTier::Global,
        "NoEnforcement",
        "{{source_segment}} khong co thuat ngu.",
    )
    .expect("tao that bai");
    assert!(created.warnings.glossary_terms_missing);
    assert!(created.warnings.unknown_markers.is_empty());

    // Cùng mệnh đề, qua đường SỬA thân thay vì TẠO -- I/O Matrix noi "Save", khong phan biet
    // hai duong ghi, nen ca nay phu ca hai.
    let updated = prompt_set_update_body(
        Some(&global),
        None,
        PromptSetTier::Global,
        created.id,
        "van khong co thuat ngu, gio da sua than.",
    )
    .expect("sua than that bai");
    assert!(updated.glossary_terms_missing);
    assert!(updated.unknown_markers.is_empty());

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Xoá một `id` đã biến mất — Phase 1's inferred precedent, kiểm coherence ở đây
// ═════════════════════════════════════════════════════════════════════════════════

/// Phase 1's handoff notes: `delete` trả `NotFound` khi 0 hàng bị đụng, suy từ tiền lệ
/// `core::glossary::store::delete_manual_term` (cùng shape: xoá MỘT hàng khỏi một danh sách
/// nhiều-tên-do-người-dùng-đặt, theo `id` lấy từ một lượt liệt kê trước đó) chứ KHÔNG phải từ
/// một hàng I/O Matrix — và yêu cầu Phase 2/3 tự kiểm coherence trước khi ghim nó vào một ca
/// hợp đồng.
///
/// Đã kiểm: `core::aiconfig::store::clear_field` KHÔNG kiểm số hàng bị đụng (xoá một khoá
/// không tồn tại là thành công im lặng) — nhưng `clear_field` là một động từ "ĐẢM BẢO trường
/// này không còn ghi đè", vốn dĩ tự nhiên là idempotent trên đúng NĂM khoá đã biết trước; nó
/// không phải "xoá MỘT hàng cụ thể khỏi một danh sách nhiều hàng do người dùng tự đặt tên".
/// `promptset::delete` giống `glossary::delete_manual_term` ở đúng hình dạng đó (nhiều hàng,
/// xoá theo `id` lấy từ một lượt liệt kê), không giống `clear_field` -- và bên TRONG chính
/// domain này, `rename`/`update_body` đã cùng trả `NotFound` cho đúng điều kiện `changed == 0`
/// (`core/promptset/store.rs:255-257`, `:285-287`), nên `delete` theo `NotFound` là NHẤT QUÁN
/// NỘI BỘ, không phải một lựa chọn lẻ loi. Kết luận: hành vi này ĐÚNG, ghim làm hợp đồng.
#[test]
fn deleting_an_id_that_already_vanished_returns_not_found_not_a_silent_success() {
    let root = temp_dir("delete-vanished");
    let global = open_global(&root);

    let created = prompt_set_create(Some(&global), None, PromptSetTier::Global, "Temp", "than")
        .expect("tao that bai");

    prompt_set_delete(Some(&global), None, PromptSetTier::Global, created.id)
        .expect("xoa lan dau phai thanh cong");

    let err = prompt_set_delete(Some(&global), None, PromptSetTier::Global, created.id)
        .err()
        .expect("xoa lan hai (id da bien mat) phai that bai, khong phai thanh cong im lang");
    assert_eq!(err.message_key(), MessageKey::PromptSetNotFound);

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Phase 4c — `PromptSetListWire::variables` là chỗ đóng của khoảng hở Phase 4b để lại: xoá
// `PromptLibraryOverlay.vue`'s `PROMPT_VARIABLES` gõ tay, gói `PromptVariable::ALL` vào
// chính phong bì `prompt_set_list` đã có (§Always spec 4.4: "A second hand-written list
// anywhere is a defect"; AC: "when a name is added to or removed from the ratified type,
// then both change together").
// ═════════════════════════════════════════════════════════════════════════════════

/// Đối chứng trực tiếp cho AC6: nếu `prompt_set_list` (`commands/promptset.rs`) từng quay lại
/// gói một mảng TAY thay vì `PromptVariable::ALL.iter().map(|v| v.as_str().to_owned())`, ca
/// này đỏ ngay khi ai đó thêm/bớt một tên khỏi `PromptVariable::ALL` mà quên sửa Wire theo —
/// đây chính là "cả hai đổi cùng nhau" được đo, không phải suy ra từ đọc mã.
#[test]
fn prompt_set_list_variables_field_carries_exactly_what_prompt_variable_all_yields() {
    let root = temp_dir("variables-wire");
    let global = open_global(&root);

    let list = prompt_set_list(Some(&global), None).expect("liet ke that bai");

    let expected: Vec<String> = PromptVariable::ALL.iter().map(|v| v.as_str().to_owned()).collect();
    assert_eq!(list.variables, expected, "PromptSetListWire::variables phai khop DUNG THU TU voi PromptVariable::ALL");
    assert_eq!(list.variables.len(), 3, "dung ba bien so ratify, khong hon");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Rào rỗng ba DDL (`src-tauri/AGENTS.md:37`) — từ story này là một BỘ BA, không còn một CẶP
// ═════════════════════════════════════════════════════════════════════════════════

/// Bóc mọi `char(N)` xuất hiện trong một hằng DDL thành một TẬP số nguyên — KHÔNG so chuỗi.
/// Đo 2026-09-17 (Phase 1's handoff notes, xác nhận lại trong doc-comment `PROMPT_SET_DDL`):
/// ba hằng mang CÙNG TẬP 25 điểm mã nhưng KHÔNG trùng byte (513/465/483 byte) vì thụt lề dòng
/// nối khác nhau theo độ dài `source_term`/`pattern`/`name`. Một phép so CHUỖI giữa ba hằng sẽ
/// đỏ dù cả ba đều đúng -- đây là lý do hàm này tồn tại thay vì `assert_eq!` trên `&str`.
fn white_space_char_codes(ddl: &str) -> BTreeSet<u32> {
    let mut out = BTreeSet::new();

    // 25 điểm mã = 24 lời gọi `char(N)` CỘNG đúng MỘT literal `' '` (dấu cách ASCII, mã 32)
    // đứng đầu danh sách -- `trim(name, ' ' || char(9) || …)`. `char(N)` không tự mang mã 32
    // (không DDL nào viết `char(32)`, vì SQL đã có literal `' '` sẵn), nên bỏ sót nhánh này
    // đếm thiếu đúng 1, luôn ra 24 chứ không bao giờ 25 -- đã đo, xem lần chạy đầu của ca này.
    if ddl.contains("' '") {
        out.insert(32);
    }

    let mut rest = ddl;
    while let Some(at) = rest.find("char(") {
        let after = &rest[at + "char(".len()..];
        let end = after.find(')').unwrap_or_else(|| panic!("`char(` khong dong ngoac trong: {after}"));
        let code: u32 = after[..end]
            .parse()
            .unwrap_or_else(|e| panic!("`char({})` khong phai so: {e}", &after[..end]));
        out.insert(code);
        rest = &after[end + 1..];
    }
    out
}

/// `src-tauri/AGENTS.md:37` nói "không cổng nào canh cặp này" (khi mới có hai) — từ story
/// này là BA hằng (`GLOSSARY_ENTRY_DDL`, `IMPORT_CLEANUP_RULE_DDL`, `PROMPT_SET_DDL`) cùng
/// khai tay 25 điểm mã `White_Space` mà `str::trim()` cắt. Ca này là cổng đó: khớp SET, và
/// khớp CẢ SỐ LƯỢNG (25) -- nên bớt một điểm mã khỏi CẢ BA cùng lúc vẫn bị bắt (một phép so
/// tập rỗng-với-tập-rỗng sẽ xanh giả nếu chỉ so `==` giữa ba tập mà không khoá độ lớn).
#[test]
fn the_three_white_space_guard_ddls_share_the_same_set_of_exactly_25_code_points() {
    let glossary = white_space_char_codes(GLOSSARY_ENTRY_DDL);
    let cleanup_rule = white_space_char_codes(IMPORT_CLEANUP_RULE_DDL);
    let prompt_set = white_space_char_codes(PROMPT_SET_DDL);

    assert_eq!(glossary.len(), 25, "GLOSSARY_ENTRY_DDL phai khai dung 25 diem ma White_Space");
    assert_eq!(cleanup_rule.len(), 25, "IMPORT_CLEANUP_RULE_DDL phai khai dung 25 diem ma White_Space");
    assert_eq!(prompt_set.len(), 25, "PROMPT_SET_DDL phai khai dung 25 diem ma White_Space");

    assert_eq!(glossary, cleanup_rule, "GLOSSARY_ENTRY_DDL va IMPORT_CLEANUP_RULE_DDL phai cung TAP diem ma");
    assert_eq!(glossary, prompt_set, "GLOSSARY_ENTRY_DDL va PROMPT_SET_DDL phai cung TAP diem ma");
}
