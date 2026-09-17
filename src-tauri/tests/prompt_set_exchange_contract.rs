//! Xuất/nhập một bộ prompt qua tệp `.prompt.md` — Story 4.5, FR79/NFR9, AD-48 — một ca cho
//! MỖI hàng của I/O & Edge-Case Matrix (`spec-4-5-xuat-va-nhap-bo-prompt.md`), ở tầng LỆNH
//! (`commands::promptset`), đúng khuôn `prompt_set_contract.rs`/`aiconfig_contract.rs`.
//!
//! ⚠️ Gọi thẳng bốn hàm thuần `commands::promptset::prompt_set_{export,open_import_preview,
//! confirm_import,cancel_import}`, không đi qua `wire::` — cổng đăng ký dây là
//! `ipc_contract.rs::the_prompt_set_wires_are_registered`, không nhân đôi ở đây.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{OpenWork, create_work};
use auratranslate_lib::commands::promptset::{
    PendingPromptImportState, PromptImportOutcomeWire, PromptSetTierWire, prompt_set_cancel_import,
    prompt_set_confirm_import, prompt_set_create, prompt_set_export, prompt_set_list,
    prompt_set_open_import_preview, prompt_set_rename,
};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::promptset::exchange::ConflictDecision;
use auratranslate_lib::core::promptset::exchange_io::MAX_PROMPT_SET_IMPORT_BYTES;
use auratranslate_lib::core::promptset::{PromptSetTier, update_body};
use auratranslate_lib::core::segment::pipeline::{ChapterInput, PipelineShape};
use auratranslate_lib::core::store::{Store, StoreSpec};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-promptset-exchange-{}-{}-{}",
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

/// Khuôn chép nguyên văn `prompt_set_contract.rs::open_work_real`.
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

fn pending() -> PendingPromptImportState {
    PendingPromptImportState::new(None)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Export a set
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn export_writes_the_whole_file_and_names_the_path() {
    let root = temp_dir("export-ok");
    let global = open_global(&root);
    let created =
        prompt_set_create(Some(&global), None, PromptSetTier::Global, "Xianxia", "than mau")
            .expect("tao bo that bai");

    let out = root.join("out.prompt.md");
    prompt_set_export(Some(&global), None, PromptSetTier::Global, created.id, &out)
        .expect("xuat that bai");

    let contents = fs::read_to_string(&out).expect("doc tep xuat");
    assert_eq!(contents, "---\nname: Xianxia\n---\nthan mau");

    drop(global);
    cleanup_dir(&root);
}

#[test]
fn export_to_an_unwritable_path_writes_nothing_and_leaves_no_temp_file() {
    let root = temp_dir("export-unwritable");
    let global = open_global(&root);
    let created =
        prompt_set_create(Some(&global), None, PromptSetTier::Global, "X", "than").expect("tao");

    let bad_dir = root.join("khong-ton-tai");
    let out = bad_dir.join("out.prompt.md");
    let err = prompt_set_export(Some(&global), None, PromptSetTier::Global, created.id, &out)
        .expect_err("phai loi vi thu muc dich khong ton tai");
    assert_eq!(err.message_key(), MessageKey::PromptSetExportWriteFailed);
    assert!(!bad_dir.exists(), "khong duoc tu tao thu muc dich");

    drop(global);
    cleanup_dir(&root);
}

#[test]
fn export_a_shadowed_global_set_is_reachable_and_exports_as_itself() {
    let root = temp_dir("export-shadowed");
    let global = open_global(&root);
    let open = open_work_real(&root);

    let global_created =
        prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Xianxia", "than toan cuc")
            .expect("tao bo Global");
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Work, "Xianxia", "than tac pham")
        .expect("tao bo Work cung ten");

    let list = prompt_set_list(Some(&global), Some(&open)).expect("liet ke");
    let winning = list.sets.iter().find(|s| s.name == "Xianxia").expect("hang Xianxia");
    assert_eq!(winning.tier, PromptSetTierWire::Work, "bo Work phai thang theo Quyet dinh #1");
    let shadowed_id =
        winning.shadowed_id.expect("hang Global bi che phai mang shadowed_id (Quyet dinh #3)");
    assert_eq!(shadowed_id, global_created.id);

    let out = root.join("shadowed.prompt.md");
    prompt_set_export(Some(&global), Some(&open), PromptSetTier::Global, shadowed_id, &out)
        .expect("xuat hang Global bi che phai thanh cong");
    let contents = fs::read_to_string(&out).unwrap();
    assert_eq!(contents, "---\nname: Xianxia\n---\nthan toan cuc", "phai xuat DUNG than Global, khong phai than Work dang thang");

    drop(open.store);
    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Round-trip
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn export_then_import_into_an_empty_tier_round_trips_the_body_byte_for_byte_with_an_unratified_marker() {
    let root = temp_dir("round-trip");
    let global = open_global(&root);
    let open = open_work_real(&root);

    let body = "Dich: {{source_segment}}\nThuat: {{glossary_terms}}\nLa: {{glosary_terms}}\n";
    prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Xianxia", body)
        .expect("tao bo goc");

    let out = root.join("x.prompt.md");
    let created_id = prompt_set_list(Some(&global), Some(&open))
        .unwrap()
        .sets
        .iter()
        .find(|s| s.name == "Xianxia")
        .unwrap()
        .id;
    prompt_set_export(Some(&global), Some(&open), PromptSetTier::Global, created_id, &out).unwrap();

    let pend = pending();
    let preview = prompt_set_open_import_preview(Some(&global), Some(&open), &pend, &out)
        .expect("xem truoc that bai");
    assert_eq!(preview.name, "Xianxia");
    assert_eq!(preview.body, body, "than phai byte-for-byte, ke ca marker chua ratify");
    assert_eq!(preview.work.as_ref().map(|t| t.kind), Some("new"), "tang Tac pham dang RONG");

    let outcome =
        prompt_set_confirm_import(Some(&global), Some(&open), &pend, PromptSetTier::Work, None)
            .expect("xac nhan that bai");
    assert_eq!(outcome, PromptImportOutcomeWire::Inserted);

    let list = prompt_set_list(Some(&global), Some(&open)).unwrap();
    let work_row = list.sets.iter().find(|s| s.tier == PromptSetTierWire::Work).unwrap();
    assert_eq!(work_row.body, body, "vong tron xuat->nhap phai giu nguyen tung byte cua than");

    drop(open.store);
    drop(global);
    cleanup_dir(&root);
}

#[test]
fn re_importing_a_just_exported_file_into_the_tier_it_came_from_is_identical_and_skipped() {
    // Review fix -- `PlanKind::Identical` khong co ca nao o tang LENH truoc ban va nay, du day
    // la thao tac binh thuong nhat: xuat mot bo roi nhap lai DUNG vao tang no vua ra.
    let root = temp_dir("reimport-same-tier-identical");
    let global = open_global(&root);

    let body = "than khong doi";
    let created = prompt_set_create(Some(&global), None, PromptSetTier::Global, "Xianxia", body)
        .expect("tao bo goc");

    let out = root.join("x.prompt.md");
    prompt_set_export(Some(&global), None, PromptSetTier::Global, created.id, &out).unwrap();

    let pend = pending();
    let preview = prompt_set_open_import_preview(Some(&global), None, &pend, &out).unwrap();
    assert_eq!(preview.global.kind, "identical", "cung name, cung than -- phai la Identical, khong phai conflict");

    let outcome =
        prompt_set_confirm_import(Some(&global), None, &pend, PromptSetTier::Global, None).unwrap();
    assert_eq!(outcome, PromptImportOutcomeWire::Skipped);

    let list = prompt_set_list(Some(&global), None).unwrap();
    assert_eq!(list.sets.len(), 1, "khong hang nao duoc them -- van dung MOT hang nhu truoc");
    assert_eq!(list.sets[0].id, created.id, "hang goc giu nguyen id");
    assert_eq!(list.sets[0].body, body, "hang goc giu nguyen than");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Import picks the tier
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn confirm_with_tier_work_lands_the_row_in_exactly_the_work_tier() {
    let root = temp_dir("picks-tier");
    let global = open_global(&root);
    let open = open_work_real(&root);

    let file = root.join("new.prompt.md");
    fs::write(&file, "---\nname: MoiToanCuc\n---\nthan moi").unwrap();

    let pend = pending();
    prompt_set_open_import_preview(Some(&global), Some(&open), &pend, &file).unwrap();
    let outcome =
        prompt_set_confirm_import(Some(&global), Some(&open), &pend, PromptSetTier::Work, None)
            .expect("xac nhan that bai");
    assert_eq!(outcome, PromptImportOutcomeWire::Inserted);

    let list = prompt_set_list(Some(&global), Some(&open)).unwrap();
    let row = list.sets.iter().find(|s| s.name == "MoiToanCuc").expect("hang phai co mat");
    assert_eq!(row.tier, PromptSetTierWire::Work, "phai nam DUNG tang da chon, khong phai Global");

    drop(open.store);
    drop(global);
    cleanup_dir(&root);
}

#[test]
fn import_to_work_when_no_work_is_open_is_refused_before_any_write() {
    let root = temp_dir("no-work-open");
    let global = open_global(&root);

    let file = root.join("new.prompt.md");
    fs::write(&file, "---\nname: X\n---\nthan").unwrap();

    let pend = pending();
    let preview = prompt_set_open_import_preview(Some(&global), None, &pend, &file).unwrap();
    assert!(preview.work.is_none(), "khong Tac pham nao mo -- tuy chon Work phai VANG MAT");

    let err = prompt_set_confirm_import(Some(&global), None, &pend, PromptSetTier::Work, None)
        .expect_err("phai tu choi truoc khi ghi");
    assert_eq!(err.message_key(), MessageKey::PromptSetWorkTierUnavailable);

    let list = prompt_set_list(Some(&global), None).unwrap();
    assert!(list.sets.is_empty(), "0 luot ghi -- khong hang nao duoc tao o bat ky tang nao");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Name collision — preview, keep mine, take theirs, stale conflict
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_name_collision_is_shown_in_preview_and_writes_nothing_until_confirmed() {
    let root = temp_dir("collision-preview");
    let global = open_global(&root);
    prompt_set_create(Some(&global), None, PromptSetTier::Global, "X", "than cu").unwrap();

    let file = root.join("x.prompt.md");
    fs::write(&file, "---\nname: X\n---\nthan moi").unwrap();

    let pend = pending();
    let preview = prompt_set_open_import_preview(Some(&global), None, &pend, &file).unwrap();
    assert_eq!(preview.global.kind, "conflict");
    assert_eq!(preview.global.existing_body.as_deref(), Some("than cu"));

    let list = prompt_set_list(Some(&global), None).unwrap();
    assert_eq!(list.sets.iter().find(|s| s.name == "X").unwrap().body, "than cu", "chua xac nhan -- 0 gi doi");

    drop(global);
    cleanup_dir(&root);
}

#[test]
fn collision_keep_mine_leaves_the_existing_row_untouched_and_counts_as_skipped() {
    let root = temp_dir("collision-keep-mine");
    let global = open_global(&root);
    prompt_set_create(Some(&global), None, PromptSetTier::Global, "X", "than cu").unwrap();

    let file = root.join("x.prompt.md");
    fs::write(&file, "---\nname: X\n---\nthan moi").unwrap();

    let pend = pending();
    prompt_set_open_import_preview(Some(&global), None, &pend, &file).unwrap();
    let outcome = prompt_set_confirm_import(
        Some(&global),
        None,
        &pend,
        PromptSetTier::Global,
        Some(ConflictDecision::KeepMine),
    )
    .unwrap();
    assert_eq!(outcome, PromptImportOutcomeWire::Skipped);

    let list = prompt_set_list(Some(&global), None).unwrap();
    assert_eq!(list.sets.iter().find(|s| s.name == "X").unwrap().body, "than cu");

    drop(global);
    cleanup_dir(&root);
}

#[test]
fn collision_take_theirs_replaces_the_existing_body_when_unchanged_since_preview() {
    let root = temp_dir("collision-take-theirs");
    let global = open_global(&root);
    prompt_set_create(Some(&global), None, PromptSetTier::Global, "X", "than cu").unwrap();

    let file = root.join("x.prompt.md");
    fs::write(&file, "---\nname: X\n---\nthan moi").unwrap();

    let pend = pending();
    prompt_set_open_import_preview(Some(&global), None, &pend, &file).unwrap();
    let outcome = prompt_set_confirm_import(
        Some(&global),
        None,
        &pend,
        PromptSetTier::Global,
        Some(ConflictDecision::TakeTheirs),
    )
    .unwrap();
    assert_eq!(outcome, PromptImportOutcomeWire::Updated);

    let list = prompt_set_list(Some(&global), None).unwrap();
    assert_eq!(list.sets.iter().find(|s| s.name == "X").unwrap().body, "than moi");

    drop(global);
    cleanup_dir(&root);
}

#[test]
fn collision_take_theirs_is_refused_with_a_stale_conflict_when_the_row_changed_since_preview_and_the_batch_stays_parked()
 {
    let root = temp_dir("collision-stale");
    let global = open_global(&root);
    let created = prompt_set_create(Some(&global), None, PromptSetTier::Global, "X", "than cu").unwrap();

    let file = root.join("x.prompt.md");
    fs::write(&file, "---\nname: X\n---\nthan moi").unwrap();

    let pend = pending();
    prompt_set_open_import_preview(Some(&global), None, &pend, &file).unwrap();

    // Mot luot ghi KHAC chen vao giua nhip xem truoc va nhip xac nhan.
    update_body(&global, None, PromptSetTier::Global, created.id, "than da bi doi duoi chan").unwrap();

    let err = prompt_set_confirm_import(
        Some(&global),
        None,
        &pend,
        PromptSetTier::Global,
        Some(ConflictDecision::TakeTheirs),
    )
    .expect_err("phai tu choi vi than da doi");
    assert_eq!(err.message_key(), MessageKey::PromptSetImportStaleConflict);

    // Lo GIU LAI de thu lai -- goi lai van thay lo, khong phai NoPendingImport.
    let err_again = prompt_set_confirm_import(
        Some(&global),
        None,
        &pend,
        PromptSetTier::Global,
        Some(ConflictDecision::TakeTheirs),
    )
    .expect_err("lo phai con o do de thu lai");
    assert_eq!(err_again.message_key(), MessageKey::PromptSetImportStaleConflict);

    let list = prompt_set_list(Some(&global), None).unwrap();
    assert_eq!(list.sets.iter().find(|s| s.name == "X").unwrap().body, "than da bi doi duoi chan");

    drop(global);
    cleanup_dir(&root);
}

#[test]
fn collision_take_theirs_is_refused_with_a_stale_conflict_when_the_row_was_renamed_since_preview_with_its_body_untouched()
 {
    // Review fix -- guards the `AND name = ?4` term added to the TakeTheirs optimistic UPDATE:
    // the row's BODY stays byte-identical to what preview captured, but its NAME changed under
    // the user between preview and confirm. Without the `name` term this update still matches
    // on `id`+`body` alone and silently overwrites a row that got renamed out from under the
    // import -- a real instance of the matrix row "Collision, body moved under the user", just
    // triggered by a rename instead of a body edit.
    let root = temp_dir("collision-stale-renamed");
    let global = open_global(&root);
    let created = prompt_set_create(Some(&global), None, PromptSetTier::Global, "X", "than cu").unwrap();

    let file = root.join("x.prompt.md");
    fs::write(&file, "---\nname: X\n---\nthan moi").unwrap();

    let pend = pending();
    prompt_set_open_import_preview(Some(&global), None, &pend, &file).unwrap();

    // Mot luot ghi KHAC doi TEN hang dich giua nhip xem truoc va nhip xac nhan -- THAN giu
    // nguyen byte-for-byte ("than cu").
    prompt_set_rename(Some(&global), None, PromptSetTier::Global, created.id, "Y").unwrap();

    let err = prompt_set_confirm_import(
        Some(&global),
        None,
        &pend,
        PromptSetTier::Global,
        Some(ConflictDecision::TakeTheirs),
    )
    .expect_err("phai tu choi vi ten hang dich da doi, du than khong doi");
    assert_eq!(err.message_key(), MessageKey::PromptSetImportStaleConflict);

    let list = prompt_set_list(Some(&global), None).unwrap();
    let row = list.sets.iter().find(|s| s.id == created.id).expect("hang van con, chi doi ten");
    assert_eq!(row.name, "Y", "ten moi phai con nguyen -- luot nhap KHONG duoc dung lai lan doi ten");
    assert_eq!(row.body, "than cu", "than phai con NGUYEN VAN -- luot nhap bi tu choi truoc khi ghi");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Malformed file — every problem reported, zero rows written
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_malformed_metadata_block_reports_every_problem_and_writes_zero_rows() {
    let root = temp_dir("malformed");
    let global = open_global(&root);

    let file = root.join("broken.prompt.md");
    fs::write(&file, "khong phai mo dau\nkhong phai name\nkhong phai dong dong").unwrap();

    let pend = pending();
    let err = prompt_set_open_import_preview(Some(&global), None, &pend, &file)
        .expect_err("tep hong phai bi tu choi");
    // AC4: "every problem is reported with its line number" -- ca ba dong dau deu hong, nen
    // ca ba phai co mat trong `lines`, khong chi dong dau tien tim duoc.
    assert_eq!(err.message_key(), MessageKey::PromptSetImportMalformed);
    assert_eq!(err.params().get("lines").map(String::as_str), Some("1, 2, 3"));

    // 0 lo nao duoc giu lai -- xac nhan phai bao NoPendingImport.
    let confirm_err = prompt_set_confirm_import(Some(&global), None, &pend, PromptSetTier::Global, None)
        .expect_err("khong lo nao de xac nhan");
    assert_eq!(confirm_err.message_key(), MessageKey::PromptSetNoPendingImport);

    let list = prompt_set_list(Some(&global), None).unwrap();
    assert!(list.sets.is_empty(), "tang dich phai con dung so hang no da co truoc (khong)");

    drop(global);
    cleanup_dir(&root);
}

#[test]
fn two_broken_lines_out_of_three_are_both_named_not_just_the_first() {
    let root = temp_dir("malformed-two");
    let global = open_global(&root);

    // Dong 1 va dong 3 hong, dong 2 (`name:`) DUNG -- ca giua cua ba to hop "dung 2/3".
    let file = root.join("broken.prompt.md");
    fs::write(&file, "khong phai mo dau\nname: X\nkhong phai dong dong").unwrap();

    let pend = pending();
    let err = prompt_set_open_import_preview(Some(&global), None, &pend, &file)
        .expect_err("tep hong hai trong ba dong phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::PromptSetImportMalformed);
    assert_eq!(err.params().get("lines").map(String::as_str), Some("1, 3"), "dong 2 dung nen KHONG co mat trong danh sach");

    let list = prompt_set_list(Some(&global), None).unwrap();
    assert!(list.sets.is_empty());

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Oversized file — refused before parsing
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_oversized_file_is_refused_before_any_parsing() {
    let root = temp_dir("oversized");
    let global = open_global(&root);

    let file = root.join("huge.prompt.md");
    {
        use std::io::{Seek, SeekFrom, Write as _};
        let mut f = fs::File::create(&file).unwrap();
        f.seek(SeekFrom::Start(MAX_PROMPT_SET_IMPORT_BYTES + 1)).unwrap();
        f.write_all(b"x").unwrap();
    }

    let pend = pending();
    let err = prompt_set_open_import_preview(Some(&global), None, &pend, &file)
        .expect_err("tep vuot tran phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::ImportTooLarge);

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Unknown marker — stored verbatim, warning names the token
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_unknown_marker_in_an_imported_body_is_stored_verbatim_and_named_in_a_warning() {
    let root = temp_dir("unknown-marker");
    let global = open_global(&root);

    let file = root.join("x.prompt.md");
    fs::write(&file, "---\nname: X\n---\n{{glosary_terms}} van con day").unwrap();

    let pend = pending();
    let preview = prompt_set_open_import_preview(Some(&global), None, &pend, &file).unwrap();
    assert_eq!(preview.warnings.unknown_markers, vec!["{{glosary_terms}}".to_owned()]);

    prompt_set_confirm_import(Some(&global), None, &pend, PromptSetTier::Global, None).unwrap();
    let list = prompt_set_list(Some(&global), None).unwrap();
    assert_eq!(
        list.sets.iter().find(|s| s.name == "X").unwrap().body,
        "{{glosary_terms}} van con day",
        "token la phai duoc luu NGUYEN VAN, khong bi sua/xoa"
    );

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Cancel — 0 write
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn cancelling_the_preview_writes_nothing_and_clears_the_pending_batch() {
    let root = temp_dir("cancel");
    let global = open_global(&root);

    let file = root.join("x.prompt.md");
    fs::write(&file, "---\nname: X\n---\nthan").unwrap();

    let pend = pending();
    prompt_set_open_import_preview(Some(&global), None, &pend, &file).unwrap();
    prompt_set_cancel_import(&pend);

    let err = prompt_set_confirm_import(Some(&global), None, &pend, PromptSetTier::Global, None)
        .expect_err("lo da bi huy -- khong con gi de xac nhan");
    assert_eq!(err.message_key(), MessageKey::PromptSetNoPendingImport);

    let list = prompt_set_list(Some(&global), None).unwrap();
    assert!(list.sets.is_empty());

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Counter-check on the promise the copy makes (§Verification): một tệp "hợp lệ tới một
// điểm rồi hỏng" phải KHÔNG ghi một phần — với định dạng một-bộ-một-tệp, "hỏng ở hàng cuối"
// dịch thành "khối metadata đúng NHƯNG dòng đóng sai": một phép kiểm chỉ nhìn lỗi mà không
// đếm hàng đích sẽ lọt một lượt ghi bán phần nếu có.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_file_valid_up_to_its_last_structural_line_and_broken_there_writes_nothing() {
    let root = temp_dir("broken-last-line");
    let global = open_global(&root);

    // Dong 1 va dong 2 DUNG dinh dang; DUNG dong 3 (dong DAY LA "hang cuoi" cua khoi metadata)
    // moi hong.
    let file = root.join("x.prompt.md");
    fs::write(&file, "---\nname: X\nkhong phai dong dong\nthan van con day").unwrap();

    let pend = pending();
    let err = prompt_set_open_import_preview(Some(&global), None, &pend, &file)
        .expect_err("dong dong sai phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::PromptSetImportMissingClosingDelimiter);

    let list = prompt_set_list(Some(&global), None).unwrap();
    assert!(list.sets.is_empty(), "mot loi o dong CUOI cua khoi metadata van phai la 0 luot ghi, khong mot ban ghi PARTIAL");

    drop(global);
    cleanup_dir(&root);
}
