//! Hành vi hai tầng của cấu hình nhà cung cấp AI — Story 4.2, FR68, `core/scope/kinds.rs:175`
//! — I/O & Edge-Case Matrix ở tầng LỆNH (`commands::aiconfig`).
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `cleanup_contract.rs`/`glossary_contract.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — thừa kế nguyên vẹn từ `cleanup_contract.rs`/`project_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + `AtomicU64`).
//! 2. **Drop `Store`/`OpenWork` TRƯỚC khi xoá thư mục** — Windows từ chối xoá tệp đang mở.
//! 3. Không `sleep` dài.
//! 4. Không ca nào treo khi nó trượt.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::aiconfig::{
    AiConfigTierWire, ai_config_clear_override, ai_config_get, ai_config_save_field,
};
use auratranslate_lib::commands::project::{OpenWork, create_work};
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::scope::{ScopeResolver, WorkScope};
use auratranslate_lib::core::segment::pipeline::{ChapterInput, PipelineShape};
use auratranslate_lib::core::store::{Store, StoreSpec};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-aiconfig-{}-{}-{}",
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
/// `ai_config_save_field`/`ai_config_get`/`ai_config_clear_override` có một tầng Work mà
/// định tuyến, không cần văn bản gì đặc biệt. Khuôn chép từ `cleanup_contract.rs::open_work_real`.
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

fn field_wire<'a>(
    fields: &'a [auratranslate_lib::commands::aiconfig::AiConfigFieldWire],
    field: AiConfigField,
) -> &'a auratranslate_lib::commands::aiconfig::AiConfigFieldWire {
    fields
        .iter()
        .find(|f| f.field == field.as_str())
        .unwrap_or_else(|| panic!("thieu truong {} trong ket qua ai_config_get", field.as_str()))
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 1 — không cấu hình ở đâu cả
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn no_configuration_anywhere_renders_every_field_empty_from_global() {
    let root = temp_dir("no-config");
    let global = open_global(&root);

    let fields = ai_config_get(Some(&global), None).expect("doc ai_config that bai").fields;
    assert_eq!(fields.len(), AiConfigField::ALL.len(), "phai tra du nam truong, ke ca truong chua cau hinh");

    for field in AiConfigField::ALL {
        let wire = field_wire(&fields, *field);
        assert_eq!(wire.value, "", "truong {} phai rong khi chua ai cau hinh", field.as_str());
        assert_eq!(wire.tier, AiConfigTierWire::Global, "truong chua cau hinh phai bao tier Global (trang thai nghi)");
        assert_eq!(wire.shadowed, None);
    }

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 2 — chỉ Global, không Work mở
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn global_only_every_field_resolves_from_global() {
    let root = temp_dir("global-only");
    let global = open_global(&root);

    for field in AiConfigField::ALL {
        let value = match field {
            AiConfigField::Provider => "anthropic",
            AiConfigField::Endpoint => "https://api.anthropic.com",
            AiConfigField::Model => "claude",
            AiConfigField::Temperature => "0.7",
            AiConfigField::MaxTokens => "2048",
        };
        ai_config_save_field(Some(&global), None, AiConfigTier::Global, *field, value)
            .unwrap_or_else(|e| panic!("luu {} that bai: {e:?}", field.as_str()));
    }

    let fields = ai_config_get(Some(&global), None).expect("doc ai_config that bai").fields;
    for field in AiConfigField::ALL {
        let wire = field_wire(&fields, *field);
        assert_eq!(wire.tier, AiConfigTierWire::Global, "moi truong phai tu Global khi khong Tac pham nao mo");
        assert_eq!(wire.shadowed, None, "Global la tang duoi cung, khong the che gi ca");
        assert_ne!(wire.value, "", "truong {} da duoc luu, khong duoc rong", field.as_str());
    }

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 3 — Work ghi đè ĐÚNG MỘT trường (`endpoint`) — mệnh đề trung tâm của AC1
// ═════════════════════════════════════════════════════════════════════════════════

/// Một cài đặt CẢ STRUCT (thay vì TỪNG TRƯỜNG) sẽ làm `provider`/`model`/`temperature`/
/// `max_tokens` cũng "thắng" về phía Work (rỗng, vì Work không ghi các trường đó) — ca này
/// bắt đúng lỗi đó: bốn trường còn lại PHẢI vẫn đọc được nguyên vẹn từ Global.
#[test]
fn work_overriding_only_endpoint_leaves_every_other_field_resolving_from_global() {
    let root = temp_dir("one-field-override");
    let global = open_global(&root);
    let opened = open_work_real(&root);

    ai_config_save_field(Some(&global), None, AiConfigTier::Global, AiConfigField::Provider, "anthropic")
        .expect("luu provider global that bai");
    ai_config_save_field(Some(&global), None, AiConfigTier::Global, AiConfigField::Endpoint, "https://api.anthropic.com")
        .expect("luu endpoint global that bai");
    ai_config_save_field(Some(&global), None, AiConfigTier::Global, AiConfigField::Model, "claude")
        .expect("luu model global that bai");
    ai_config_save_field(Some(&global), None, AiConfigTier::Global, AiConfigField::Temperature, "0.5")
        .expect("luu temperature global that bai");
    ai_config_save_field(Some(&global), None, AiConfigTier::Global, AiConfigField::MaxTokens, "1024")
        .expect("luu max_tokens global that bai");

    ai_config_save_field(
        Some(&global),
        Some(&opened),
        AiConfigTier::Work,
        AiConfigField::Endpoint,
        "https://api.self-hosted.example.com",
    )
    .expect("luu endpoint work that bai");

    let fields = ai_config_get(Some(&global), Some(&opened)).expect("doc ai_config that bai").fields;

    let endpoint = field_wire(&fields, AiConfigField::Endpoint);
    assert_eq!(endpoint.tier, AiConfigTierWire::Work, "endpoint phai den tu Work");
    assert_eq!(endpoint.value, "https://api.self-hosted.example.com");
    assert_eq!(
        endpoint.shadowed.as_deref(),
        Some("https://api.anthropic.com"),
        "gia tri Global bi che phai con nguyen o shadowed"
    );

    for field in [AiConfigField::Provider, AiConfigField::Model, AiConfigField::Temperature, AiConfigField::MaxTokens] {
        let wire = field_wire(&fields, field);
        assert_eq!(
            wire.tier,
            AiConfigTierWire::Global,
            "truong {} KHONG duoc bi Work che chi vi Work ghi de MOT truong khac -- day la ca \
             mot cai dat CA STRUCT se lam sai",
            field.as_str()
        );
        assert_ne!(wire.value, "", "truong {} phai van doc duoc tu Global", field.as_str());
    }

    drop(opened.store);
    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 4 — lưu một ghi đè Work khi chưa Tác phẩm nào mở
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn saving_a_work_override_with_no_work_open_is_refused_and_writes_nothing() {
    let root = temp_dir("no-work-open");
    let global = open_global(&root);

    let err = ai_config_save_field(Some(&global), None, AiConfigTier::Work, AiConfigField::Endpoint, "https://x.example.com")
        .err()
        .expect("phai bi tu choi khi khong Tac pham nao dang mo");
    assert_eq!(err.message_key(), MessageKey::AiConfigWorkTierUnavailable);

    let fields = ai_config_get(Some(&global), None).expect("doc ai_config that bai").fields;
    assert_eq!(field_wire(&fields, AiConfigField::Endpoint).value, "", "0 luot ghi duoc phep xay ra");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 5..7 — mỗi trường bị từ chối TRƯỚC khi cham SQL, 0 luot ghi
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn temperature_out_of_range_is_rejected_before_any_write() {
    let root = temp_dir("bad-temperature");
    let global = open_global(&root);

    for bad in ["-1", "3", "abc"] {
        let err = ai_config_save_field(Some(&global), None, AiConfigTier::Global, AiConfigField::Temperature, bad)
            .err()
            .unwrap_or_else(|| panic!("gia tri {bad:?} phai bi tu choi"));
        assert_eq!(err.message_key(), MessageKey::AiConfigInvalidValue);
    }

    let fields = ai_config_get(Some(&global), None).expect("doc ai_config that bai").fields;
    assert_eq!(field_wire(&fields, AiConfigField::Temperature).value, "", "khong gia tri nao duoc ghi");

    drop(global);
    cleanup_dir(&root);
}

#[test]
fn max_tokens_not_a_positive_integer_is_rejected_before_any_write() {
    let root = temp_dir("bad-max-tokens");
    let global = open_global(&root);

    for bad in ["0", "-5", "1.5", "abc"] {
        let err = ai_config_save_field(Some(&global), None, AiConfigTier::Global, AiConfigField::MaxTokens, bad)
            .err()
            .unwrap_or_else(|| panic!("gia tri {bad:?} phai bi tu choi"));
        assert_eq!(err.message_key(), MessageKey::AiConfigInvalidValue);
    }

    let fields = ai_config_get(Some(&global), None).expect("doc ai_config that bai").fields;
    assert_eq!(field_wire(&fields, AiConfigField::MaxTokens).value, "", "khong gia tri nao duoc ghi");

    drop(global);
    cleanup_dir(&root);
}

#[test]
fn endpoint_not_an_absolute_url_is_rejected_before_any_write() {
    let root = temp_dir("bad-endpoint");
    let global = open_global(&root);

    for bad in ["localhost:11434", ""] {
        let err = ai_config_save_field(Some(&global), None, AiConfigTier::Global, AiConfigField::Endpoint, bad)
            .err()
            .unwrap_or_else(|| panic!("gia tri {bad:?} phai bi tu choi"));
        assert_eq!(err.message_key(), MessageKey::AiConfigInvalidValue);
    }

    let fields = ai_config_get(Some(&global), None).expect("doc ai_config that bai").fields;
    assert_eq!(field_wire(&fields, AiConfigField::Endpoint).value, "", "khong gia tri nao duoc ghi");

    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 8 — trả một ghi đè Work về kế thừa
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn clearing_a_work_override_returns_the_field_to_inherited() {
    let root = temp_dir("clear-override");
    let global = open_global(&root);
    let opened = open_work_real(&root);

    ai_config_save_field(Some(&global), None, AiConfigTier::Global, AiConfigField::Endpoint, "https://api.anthropic.com")
        .expect("luu endpoint global that bai");
    ai_config_save_field(Some(&global), Some(&opened), AiConfigTier::Work, AiConfigField::Endpoint, "https://local.example.com")
        .expect("luu endpoint work that bai");

    let before = ai_config_get(Some(&global), Some(&opened)).expect("doc truoc khi xoa").fields;
    assert_eq!(field_wire(&before, AiConfigField::Endpoint).tier, AiConfigTierWire::Work);

    ai_config_clear_override(Some(&opened), AiConfigField::Endpoint).expect("xoa ghi de that bai");

    let after = ai_config_get(Some(&global), Some(&opened)).expect("doc sau khi xoa").fields;
    let endpoint = field_wire(&after, AiConfigField::Endpoint);
    assert_eq!(endpoint.tier, AiConfigTierWire::Global, "sau khi xoa, truong phai phan giai lai tu Global");
    assert_eq!(endpoint.value, "https://api.anthropic.com");
    assert_eq!(endpoint.shadowed, None, "Global khong con gi de che nua");

    drop(opened.store);
    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 9 — mở lại một `.atproj` đã lưu
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn reopening_a_saved_atproj_resolves_the_work_tier_in_a_new_session() {
    let root = temp_dir("reopen");
    let global = open_global(&root);
    let opened = open_work_real(&root);
    let work_id = opened.meta.work_id.clone();
    let project_db = opened.dir.join("project.db");

    ai_config_save_field(Some(&global), Some(&opened), AiConfigTier::Work, AiConfigField::Model, "llama3")
        .expect("luu model work that bai");

    // Dong phien hien tai -- gia lap dong ung dung.
    drop(opened.store);

    // Mo lai TU DAU: mot Store project.db MOI + mot ScopeResolver::with_work MOI, dung khuon
    // `commands::project::open_work` rebuild lai (khong goi thang ham do o day vi no doi
    // qua Indexer/library-index.db that -- muc tieu cua ca nay la tang doc cua chinh module
    // nay, khong phai duong mo lai toan bo).
    let reopened_store = Store::open(StoreSpec::project(project_db)).expect("mo lai project.db");
    let resolver = ScopeResolver::with_work(WorkScope { work_id });

    let reopened = OpenWork {
        dir: opened.dir,
        store: reopened_store,
        scope: resolver,
        meta: opened.meta,
        chapter_id: opened.chapter_id,
        images_saved: opened.images_saved,
        images_failed: opened.images_failed,
    };

    let fields = ai_config_get(Some(&global), Some(&reopened)).expect("doc sau khi mo lai").fields;
    let model = field_wire(&fields, AiConfigField::Model);
    assert_eq!(model.tier, AiConfigTierWire::Work, "tang Tac pham phai phan giai duoc o phien MOI");
    assert_eq!(model.value, "llama3");

    drop(reopened.store);
    drop(global);
    cleanup_dir(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 10 — `work_tier_available` bám `OpenWorkState`, KHÔNG một proxy chế độ UI
// ═════════════════════════════════════════════════════════════════════════════════

/// Đối chứng trực tiếp cho khuyết tật đã đo (xem doc-comment `AiConfigGetWire`): trước bản vá
/// này, `src/settingsState.ts` suy "có Tác phẩm đang mở không" từ `currentMode !== 'library'`
/// — một proxy UI KHÔNG tương đương `OpenWorkState` (`close_open_work` chỉ chạy ở
/// `RunEvent::Exit`, không IPC nào đóng một Tác phẩm). Ca này khoá `work_tier_available` vào
/// ĐÚNG MỘT nguồn: tham số `Option<&OpenWork>` của chính lượt gọi — `None` ⇒ `false`, `Some`
/// ⇒ `true`, không phụ thuộc gì khác. Không test nào TRƯỚC bản vá này gọi trực tiếp đến
/// trường này — đây là lỗ hở "0 phép kiểm nào sẽ đỏ nếu tín hiệu tầng bị sai" mà lượt vá phải
/// đóng.
#[test]
fn work_tier_available_tracks_open_work_state_not_a_ui_proxy() {
    let root = temp_dir("work-tier-available");
    let global = open_global(&root);

    let closed = ai_config_get(Some(&global), None).expect("doc khi chua mo Tac pham nao");
    assert!(!closed.work_tier_available, "chua mo Tac pham nao ⇒ work_tier_available phai false");

    let opened = open_work_real(&root);
    let with_work = ai_config_get(Some(&global), Some(&opened)).expect("doc khi da mo Tac pham");
    assert!(with_work.work_tier_available, "co Tac pham dang mo ⇒ work_tier_available phai true");

    drop(opened.store);
    drop(global);
    cleanup_dir(&root);
}
