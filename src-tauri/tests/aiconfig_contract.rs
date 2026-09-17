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

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, Once};

use auratranslate_lib::commands::aiconfig::{
    AiConfigTierWire, ai_config_clear_override, ai_config_delete_key, ai_config_get,
    ai_config_save_field, ai_config_save_key,
};
use auratranslate_lib::commands::project::{OpenWork, create_work};
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::scope::{ScopeResolver, WorkScope};
use auratranslate_lib::core::segment::pipeline::{ChapterInput, PipelineShape};
use auratranslate_lib::core::store::{Store, StoreSpec};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

// ═════════════════════════════════════════════════════════════════════════════════
// Story 4.3 — mock-store keychain: cài MỘT LẦN cho cả nhị phân, TRƯỚC LƯỢT CHẠM ĐẦU
// TIÊN (không chỉ trước lượt `set` đầu tiên) — xem dòng đỏ ở Phase 3,
// `4-3-phases-2026-09-17.md`, và Design Notes của `spec-4-3-api-key-trong-keychain.md`.
// ═════════════════════════════════════════════════════════════════════════════════

/// Chép NGUYÊN VĂN hai hằng của `core/aiconfig/keychain.rs` — module đó chỉ khai
/// `KEYCHAIN_SERVICE`/`KEYCHAIN_ACCOUNT` `private` (không cả `pub(crate)`), và tệp test
/// này là một CRATE KHÁC nên không với tới được dù có nới lên `pub(crate)`.
///
/// ⚠️ Đây là một chuỗi TRÙNG LẶP có chủ, không phải một lỗ hổng bị bỏ sót: nếu hai bên
/// lệch nhau, ca "keychain từ chối trả lời"/"yêu cầu tầng Work bị chặn TRƯỚC khi chạm
/// keychain" bên dưới tự lộ — chúng sẽ bơm lỗi vào một entry mock KHÁC entry mà mã sản
/// phẩm mở, nên một khẳng định "phải thấy `keychain_unavailable`" sẽ ĐỎ (không quan sát
/// được lỗi đã bơm) thay vì lặng lẽ không kiểm gì cả.
const KEYCHAIN_SERVICE: &str = "com.auratranslate.desktop";
const KEYCHAIN_ACCOUNT: &str = "ai_provider_api_key";

/// Kho mock của `keyring-core` là MỘT kho DÙNG CHUNG cho toàn bộ tiến trình test này
/// (Design Notes spec 4.3: "the store is process-global, so the swap is shared by every
/// test in the binary"), và khoá API chỉ có ĐÚNG MỘT account cố định — không tham số
/// hoá theo ca test hay theo tầng. Mọi ca CHẠM hoặc KHẲNG ĐỊNH trên trạng thái khoá API
/// (`configured`/`save`/`delete`, hay bơm một lỗi giả lập) phải giữ khoá này xuyên suốt
/// đời ca, nếu không hai ca chạy trên hai luồng khác nhau (mặc định của `cargo test`) sẽ
/// giẫm lên đúng MỘT credential. Mười ca Story 4.2 sẵn có ở tệp này không cần khoá này —
/// chúng gọi `ai_config_get` (nên vẫn thăm dò keychain) nhưng không ca nào KHẲNG ĐỊNH gì
/// trên giá trị `key_configured`.
static KEYCHAIN_KEY_TEST_LOCK: Mutex<()> = Mutex::new(());

/// Cài mock-store CHỈ MỘT LẦN cho cả nhị phân này — PHẢI chạy trước LƯỢT CHẠM keychain
/// ĐẦU TIÊN, không chỉ trước lượt `set` đầu tiên.
///
/// 🔴 Đây là một hàng rào bắt buộc, không phải một tiện nghi: `ai_config_get`
/// (`commands/aiconfig.rs:182`) tự thăm dò `keychain::configured()` trên MỌI lượt gọi —
/// kể cả mười ca Story 4.2 sẵn có ở tệp này, vốn không đụng gì tới khoá API. Nếu máy
/// chạy suite này từng lưu một khoá THẬT qua chính ứng dụng, một nhị phân test build lại
/// (bất kỳ thay đổi nào — một tệp test bị chạm, một dependency đổi, kể cả `cargo clean`)
/// đọc lại entry đó gây treo trên hộp thoại xác thực macOS — ĐO ĐƯỢC ở Phase 2
/// (`4-3-phases-2026-09-17.md`), không phải một giả thuyết.
///
/// Thứ tự BẮT BUỘC, đọc từ nguồn đã tải: `keyring::Entry::store_status()` trước, để
/// LazyLock của crate `keyring` tự khởi tạo kho THẬT của nền tảng
/// (`keyring-4.1.6/src/v1.rs:108-121` — chỉ dựng đối tượng kho, không đọc/ghi bí mật nào
/// — đo tại `apple-native-keyring-store-1.0.1/src/keychain.rs:177-180`,
/// `Store::new_internal` không gọi một API Keychain nào), RỒI MỚI
/// `keyring_core::set_default_store` đè bằng mock. Làm ngược thứ tự vô ích:
/// `keyring::Entry::new` chỉ chạy phần khởi tạo kho THẬT đúng MỘT lần (chính `LazyLock`
/// đó) và ghi đè bất cứ mock nào đã cài trước nó
/// (`keyring-core-1.0.0/src/lib.rs:65-71`, `set_default_store` ghi đè vô điều kiện).
///
/// ⚠️ **Chưa có tiền lệ "cài một lần trước mọi test trong nhị phân" ở kho này** — không
/// tệp `tests/*.rs` nào khác dùng `std::sync::Once`/`ctor` cho việc này (đã `grep`). Thứ
/// gần nhất là các hằng `AtomicU64`/`LazyLock` dùng để CẤP PHÁT thư mục tạm riêng cho
/// từng ca, không phải để CÀI ĐẶT một trạng thái toàn cục dùng chung. `std::sync::Once`
/// (thư viện chuẩn, không thêm dependency) là lựa chọn tại chỗ cho story này.
fn install_mock_keychain_store_once() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let real_store_status = keyring::Entry::store_status();
        assert!(
            real_store_status.is_ok(),
            "kho keychain THAT khoi tao that bai ({real_store_status:?}) -- moi ca sau day \
             se cham `NoDefaultStore` thay vi mock (xem canh bao trong `keychain.rs`: mot \
             NoDefaultStore lam MOI ca sau do bao xanh ma khong kiem gi ca)"
        );

        keyring_core::set_default_store(
            keyring_core::mock::Store::new()
                .expect("keyring_core::mock::Store::new that bai -- khong the tao kho gia lap"),
        );
    });
}

/// Bơm một lỗi DÙNG MỘT LẦN vào đúng credential mock mà mã sản phẩm mở (cùng account cố
/// định) — lượt gọi SAU ĐÓ vào `keychain::set`/`delete`/`configured` (qua bất kỳ hàm
/// thuần nào của `commands::aiconfig`) nhận lỗi này, rồi lỗi tự xoá
/// (`mock::Cred::set_error` doc-comment: "the error will then be cleared"). Dùng để mô
/// phỏng hàng I/O Matrix "Keychain refuses" mà không cần khoá/quyền thật nào trên máy.
fn inject_one_shot_keychain_error() {
    // Mỗi hàm CHẠM keychain trong tệp này tự cài mock — KHÔNG dựa vào việc một helper khác
    // (`open_global`) đã chạy trước, vì thứ tự các ca trên nhiều luồng là KHÔNG xác định
    // (review 2026-09-17: ca `a_work_tier_key_request_is_refused_...` không gọi `open_global`
    // và có thể là ca ĐẦU TIÊN chạm keychain trong cả nhị phân).
    install_mock_keychain_store_once();
    let entry = keyring_core::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .expect("keyring_core::Entry::new that bai -- mock store chua duoc cai");
    let mock: &keyring_core::mock::Cred = entry
        .as_any()
        .downcast_ref()
        .expect("entry khong phai keyring_core::mock::Cred -- mock chua duoc cai truoc do");
    mock.set_error(keyring_core::Error::Invalid(
        "mock".to_owned(),
        "keychain tu choi tra loi (Story 4.3, ca gia lap that bai)".to_owned(),
    ));
}

/// Đọc trực tiếp giá trị hiện có trong mock — CHỈ test mới với được (mã sản phẩm không
/// bao giờ trả giá trị qua IPC, §Always spec 4.3). Dùng để đối chứng "Overwrite" thật sự
/// THAY giá trị tại chỗ, không tạo entry thứ hai.
fn read_raw_mock_key_value() -> Result<String, keyring_core::Error> {
    // Cùng lý do `inject_one_shot_keychain_error` tự cài mock -- xem doc-comment ở đó.
    install_mock_keychain_store_once();
    keyring_core::Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT)
        .expect("keyring_core::Entry::new that bai -- mock store chua duoc cai")
        .get_password()
}

/// Đếm số credential mock khớp account của khoá API — luôn phải là 0 hoặc 1: mock tái
/// dùng ĐÚNG MỘT `Cred` cho một cặp (service, user) trùng nhau
/// (`keyring-core-1.0.0/src/mock.rs::Store::build`), nên đây là bằng chứng THẬT (không
/// suy diễn) cho I/O Matrix "Overwrite an existing key": "no second entry created".
fn count_mock_key_entries() -> usize {
    // Cùng lý do `inject_one_shot_keychain_error` tự cài mock -- xem doc-comment ở đó.
    install_mock_keychain_store_once();
    keyring_core::Entry::search(&HashMap::from([
        ("service", KEYCHAIN_SERVICE),
        ("user", KEYCHAIN_ACCOUNT),
    ]))
    .expect("keyring_core::Entry::search that bai")
    .len()
}

/// Đưa mock về "chưa cấu hình" TRƯỚC một ca — các ca chia sẻ đúng MỘT credential (xem
/// [`KEYCHAIN_KEY_TEST_LOCK`]), nên mỗi ca phải tự dọn điểm bắt đầu thay vì giả định nó
/// sạch. "Xoá khi không có gì để xoá" là thành công (I/O Matrix), nên gọi vô điều kiện
/// là an toàn.
fn reset_key_to_not_configured() {
    // Cùng lý do `inject_one_shot_keychain_error` tự cài mock -- xem doc-comment ở đó. Đây là
    // hàm ĐẦU TIÊN mọi ca hàng khoá API gọi, nên đây thực chất là điểm cài đặt chính hôm nay --
    // nhưng KHÔNG được là điểm cài đặt DUY NHẤT: ba helper kia cũng tự cài, để không ca tương
    // lai nào tái tạo lỗ này chỉ vì nó quên gọi qua `reset_key_to_not_configured` trước.
    install_mock_keychain_store_once();
    ai_config_delete_key(AiConfigTier::Global).expect("dat lai trang thai khoa that bai");
}

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
    // Mọi ca của tệp này (kể cả mười ca Story 4.2 sẵn có, vốn không đụng gì tới khoá API
    // nhưng vẫn chạm keychain GIÁN TIẾP qua thăm dò `ai_config_get`) mở qua đây trước —
    // nhưng đây KHÔNG phải chỗ gọi DUY NHẤT của `install_mock_keychain_store_once`: bốn
    // helper chạm keychain trực tiếp (`inject_one_shot_keychain_error`,
    // `read_raw_mock_key_value`, `count_mock_key_entries`, `reset_key_to_not_configured`)
    // cũng tự gọi nó, vì ca `a_work_tier_key_request_is_refused_...` không mở `Store` nào
    // cả và có thể là ca ĐẦU TIÊN chạm keychain trong nhị phân này.
    install_mock_keychain_store_once();
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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 4.3 — I/O & Edge-Case Matrix của khoá API trong keychain (Task 8/Phase 3)
// ═════════════════════════════════════════════════════════════════════════════════

// Hàng 1 — không có entry nào trong keychain
#[test]
fn no_key_anywhere_reads_as_not_configured() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp_dir("key-none");
    let global = open_global(&root);
    reset_key_to_not_configured();

    let result = ai_config_get(Some(&global), None).expect("doc ai_config that bai");
    assert_eq!(result.key_configured, Some(false), "chua ai cau hinh khoa nao phai bao Some(false)");
    assert_eq!(result.fields.len(), AiConfigField::ALL.len(), "nam truong plaintext van phai du");

    drop(global);
    cleanup_dir(&root);
}

// Hàng 2 — lưu một khoá
#[test]
fn saving_a_key_flips_status_to_configured_and_never_returns_the_value() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp_dir("key-save");
    let global = open_global(&root);
    reset_key_to_not_configured();

    ai_config_save_key(AiConfigTier::Global, "sk-vua-luu-mot-khoa").expect("luu khoa that bai");

    let result = ai_config_get(Some(&global), None).expect("doc ai_config that bai");
    assert_eq!(result.key_configured, Some(true), "sau khi luu, trang thai phai la da cau hinh");
    // Kiểu `AiConfigGetWire` cấu trúc không có trường nào mang giá trị khoá (§Always spec
    // 4.3), nên đây là một đối chứng THÊM chứ không phải phép kiểm DUY NHẤT của mệnh đề.
    assert!(
        !format!("{result:?}").contains("sk-vua-luu-mot-khoa"),
        "Debug cua AiConfigGetWire khong duoc mang gia tri khoa"
    );

    drop(global);
    cleanup_dir(&root);
}

// Hàng 3 — lưu một khoá rỗng hoặc chỉ khoảng trắng
#[test]
fn saving_an_empty_or_whitespace_key_is_rejected_before_any_keychain_call() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp_dir("key-invalid");
    let global = open_global(&root);
    reset_key_to_not_configured();

    for bad in ["", "   "] {
        // Bơm một lỗi keychain giả lập TRƯỚC: nếu `validate_key` không chặn được và mã
        // sản phẩm lỡ chạm `keychain::set`, ca này sẽ quan sát `keychain_unavailable`
        // thay vì `key_invalid_value` -- tức phép kiểm bên dưới tự lộ nếu thứ tự bị đảo.
        // `validate_key` từ chối TRƯỚC khi chạm keychain, nên lỗi bơm này KHÔNG bị tiêu
        // thụ ở đây -- nó còn nguyên cho lượt "xả" ngay dưới vòng lặp.
        inject_one_shot_keychain_error();

        let err = ai_config_save_key(AiConfigTier::Global, bad)
            .err()
            .unwrap_or_else(|| panic!("gia tri {bad:?} phai bi tu choi"));
        assert_eq!(
            err.message_key(),
            MessageKey::AiConfigKeyInvalidValue,
            "gia tri {bad:?} phai bao loi khong hop le -- neu bao keychain_unavailable thay vao \
             do, nghia la keychain DA bi cham truoc khi validate_key tu choi"
        );
    }

    // Xả lỗi bơm còn treo (chưa lượt chạm keychain thật nào tiêu thụ nó, đúng như ca
    // trên vừa chứng minh) bằng một lượt gọi bỏ đi -- nếu không, chính lượt gọi
    // `ai_config_get` bên dưới sẽ vô tình tiêu thụ nó và đọc thành `None` thay vì trạng
    // thái THẬT, tự làm ca này báo sai.
    let _ = ai_config_get(Some(&global), None);

    let result = ai_config_get(Some(&global), None).expect("doc ai_config that bai");
    assert_eq!(result.key_configured, Some(false), "khong gia tri nao duoc ghi");

    drop(global);
    cleanup_dir(&root);
}

// Hàng 4 — ghi đè một khoá đã có
#[test]
fn overwriting_an_existing_key_replaces_the_value_in_place_with_a_single_entry() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp_dir("key-overwrite");
    let global = open_global(&root);
    reset_key_to_not_configured();

    ai_config_save_key(AiConfigTier::Global, "sk-truoc").expect("luu lan dau that bai");
    ai_config_save_key(AiConfigTier::Global, "sk-sau").expect("ghi de that bai");

    let result = ai_config_get(Some(&global), None).expect("doc ai_config that bai");
    assert_eq!(result.key_configured, Some(true), "trang thai phai giu nguyen da cau hinh");
    assert_eq!(
        read_raw_mock_key_value().expect("doc raw gia tri mock that bai"),
        "sk-sau",
        "gia tri phai duoc THAY tai cho, khong con gia tri cu"
    );
    assert_eq!(count_mock_key_entries(), 1, "ghi de khong duoc tao entry thu hai");

    drop(global);
    cleanup_dir(&root);
}

// Hàng 5 — xoá khoá đang có
#[test]
fn deleting_the_key_returns_status_to_not_configured() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp_dir("key-delete");
    let global = open_global(&root);
    reset_key_to_not_configured();
    ai_config_save_key(AiConfigTier::Global, "sk-se-bi-xoa").expect("luu khoa that bai");

    ai_config_delete_key(AiConfigTier::Global).expect("xoa khoa that bai");

    let result = ai_config_get(Some(&global), None).expect("doc ai_config that bai");
    assert_eq!(result.key_configured, Some(false), "sau khi xoa phai tro ve chua cau hinh");

    drop(global);
    cleanup_dir(&root);
}

// Hàng 6 — xoá khi không có entry nào
#[test]
fn deleting_a_key_that_does_not_exist_is_treated_as_success() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp_dir("key-delete-none");
    let global = open_global(&root);
    reset_key_to_not_configured();

    ai_config_delete_key(AiConfigTier::Global)
        .expect("xoa mot khoa khong ton tai phai la THANH CONG (hau trang thai la trang thai yeu cau)");

    let result = ai_config_get(Some(&global), None).expect("doc ai_config that bai");
    assert_eq!(result.key_configured, Some(false));

    drop(global);
    cleanup_dir(&root);
}

// Hàng 7 — lưu khoá khi một Tác phẩm đang mở: vẫn Global, khác năm trường kia
#[test]
fn saving_the_key_while_a_work_is_open_still_writes_only_the_global_entry() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp_dir("key-work-open");
    let global = open_global(&root);
    let opened = open_work_real(&root);
    reset_key_to_not_configured();

    // Đối chứng: MỘT trường thường (`provider`) ghi tầng Work khi Tac pham dang mo --
    // day la hanh vi Story 4.2, dung de lam noi bat cai KHAC cua khoa API ngay ben duoi.
    ai_config_save_field(Some(&global), Some(&opened), AiConfigTier::Work, AiConfigField::Provider, "anthropic-work")
        .expect("luu provider tang Work that bai");

    ai_config_save_key(AiConfigTier::Global, "sk-mot-khoa-cho-moi-tac-pham").expect("luu khoa that bai");

    let with_work = ai_config_get(Some(&global), Some(&opened)).expect("doc khi Tac pham dang mo");
    assert_eq!(
        with_work.key_configured,
        Some(true),
        "khoa phai bao da cau hinh du dang co mot Tac pham mo"
    );
    assert_eq!(
        field_wire(&with_work.fields, AiConfigField::Provider).tier,
        AiConfigTierWire::Work,
        "doi chung: provider PHAI o tang Work -- khac han khoa API, thu khong co khai niem tang"
    );

    let without_work = ai_config_get(Some(&global), None).expect("doc khi khong Tac pham nao mo");
    assert_eq!(
        without_work.key_configured,
        Some(true),
        "khoa la Global-only: dong Tac pham khong duoc lam mat trang thai da cau hinh"
    );
    assert_eq!(count_mock_key_entries(), 1, "van dung MOT entry keychain, bat ke Tac pham nao dang mo");

    drop(opened.store);
    drop(global);
    cleanup_dir(&root);
}

// Hàng 8 — một yêu cầu khoá ở tầng Tác phẩm bị chặn TẠI TẦNG LỆNH, 0 lượt chạm keychain
#[test]
fn a_work_tier_key_request_is_refused_at_the_command_layer_with_zero_keychain_interaction() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    reset_key_to_not_configured();

    // Bơm lỗi TRƯỚC mỗi lượt gọi: nếu tầng lệnh lỡ chạm keychain trước khi tu choi tang,
    // ca se quan sat duoc `keychain_unavailable` thay vi `key_is_global` -- day la cach
    // CHUNG MINH "khong doc, khong ghi gi" ma khong can go bo mock (mock la dung chung ca
    // nhi phan, xem doc-comment KEYCHAIN_KEY_TEST_LOCK).
    inject_one_shot_keychain_error();
    let err_save = ai_config_save_key(AiConfigTier::Work, "sk-khong-duoc-ghi")
        .err()
        .expect("yeu cau tang Work phai bi tu choi");
    assert_eq!(
        err_save.message_key(),
        MessageKey::AiConfigKeyIsGlobal,
        "phai la key_is_global, KHONG phai keychain_unavailable -- neu la keychain_unavailable \
         nghia la loi bom vao DA bi cham toi, tuc tang lenh KHONG chan truoc keychain"
    );

    inject_one_shot_keychain_error();
    let err_delete = ai_config_delete_key(AiConfigTier::Work)
        .err()
        .expect("yeu cau xoa tang Work phai bi tu choi");
    assert_eq!(err_delete.message_key(), MessageKey::AiConfigKeyIsGlobal);

    // Xac nhan loi bom van con nguyen (chua bi tieu thu boi hai loi tu choi tren) -- lan
    // cham THAT dau tien (tang Global) phai nhan dung loi keychain_unavailable, chung minh
    // co che bom loi hoat dong dung nhu mo ta, khong phai mot phep do vo hieu.
    let err_real_touch = ai_config_save_key(AiConfigTier::Global, "sk-se-that-bai")
        .err()
        .expect("lan cham that dau tien phai that bai vi loi da bom");
    assert_eq!(err_real_touch.message_key(), MessageKey::AiConfigKeychainUnavailable);

    reset_key_to_not_configured();
}

// Hàng 9 — keychain từ chối trả lời (lượt ghi/xoá): hành động thất bại, màn hình vẫn dùng được
#[test]
fn keychain_refusing_to_answer_fails_the_action_but_keeps_the_section_usable() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp_dir("key-unavailable");
    let global = open_global(&root);
    reset_key_to_not_configured();

    inject_one_shot_keychain_error();
    let err = ai_config_save_key(AiConfigTier::Global, "sk-that-bai")
        .err()
        .expect("keychain tu choi tra loi phai lam hanh dong that bai");
    assert_eq!(err.message_key(), MessageKey::AiConfigKeychainUnavailable);
    assert!(err.retryable(), "mot keychain bi khoa/tu choi quyen co the thanh cong o luot bam lai");
    assert!(err.params().is_empty(), "khong tham so nao duoc phep mang gia tri khoa hay mot phan cua no");

    // Man hinh van dung duoc: nam truong plaintext van luu/doc binh thuong dung luc nay.
    ai_config_save_field(Some(&global), None, AiConfigTier::Global, AiConfigField::Provider, "anthropic")
        .expect("cai dat khac van phai luu duoc du khoa API vua that bai");
    let fields = ai_config_get(Some(&global), None).expect("doc ai_config that bai").fields;
    assert_eq!(field_wire(&fields, AiConfigField::Provider).value, "anthropic");

    drop(global);
    cleanup_dir(&root);
}

// Hàng 9 (nhánh ĐỌC) — thăm dò thất bại phải ra `None`, KHÔNG phải `Some(false)`, và năm
// trường plaintext vẫn phải tải được — đối chứng trực tiếp cho bản sửa 2026-09-17 của Phase 2.
#[test]
fn keychain_probe_failure_on_get_surfaces_as_none_not_a_false_configured_state() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp_dir("key-probe-fails");
    let global = open_global(&root);
    reset_key_to_not_configured();

    inject_one_shot_keychain_error();
    let result = ai_config_get(Some(&global), None)
        .expect("ai_config_get khong duoc that bai chi vi keychain tu choi tra loi mot THAM DO");
    assert_eq!(
        result.key_configured, None,
        "tham do that bai phai ra None -- KHONG duoc doc thanh Some(false) ('da hoi, chua cau \
         hinh'), day la hai trang thai KHAC nhau (AGENTS.md: mot gia tri co the UNKNOWN nhan \
         Option/NULL, khong bao gio mot 0/false)"
    );
    assert_eq!(result.fields.len(), AiConfigField::ALL.len(), "nam truong plaintext van phai tai duoc du");
    for field in AiConfigField::ALL {
        assert_eq!(field_wire(&result.fields, *field).value, "", "chua ai cau hinh field nay trong ca nay");
    }

    drop(global);
    cleanup_dir(&root);
}

// Hàng 10 — một lỗi xảy ra trong khi keychain đang giữ một khoá không được cõng theo nó
#[test]
fn errors_around_a_configured_key_never_carry_the_key_value_or_a_prefix_of_it() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp_dir("key-error-redaction");
    let global = open_global(&root);
    reset_key_to_not_configured();

    const SECRET: &str = "sk-mot-chuoi-danh-dau-rat-rieng-987654321";
    ai_config_save_key(AiConfigTier::Global, SECRET).expect("luu khoa that bai");

    inject_one_shot_keychain_error();
    let err = ai_config_delete_key(AiConfigTier::Global).err().expect("phai that bai vi loi da bom");
    assert_eq!(err.message_key(), MessageKey::AiConfigKeychainUnavailable);

    let err_debug = format!("{err:?}");
    assert!(!err_debug.contains(SECRET), "Debug cua IpcError khong duoc mang khoa");
    assert!(!err_debug.contains(&SECRET[..8]), "Debug cua IpcError khong duoc mang mot TIEN TO cua khoa");
    assert!(err.params().values().all(|v| !v.contains(SECRET)), "khong tham so nao duoc mang khoa");

    let get_result = ai_config_get(Some(&global), None).expect("doc ai_config that bai");
    assert!(
        !format!("{get_result:?}").contains(SECRET),
        "Debug cua AiConfigGetWire khong duoc mang khoa dang duoc keychain giu"
    );

    drop(global);
    cleanup_dir(&root);
}

// AC riêng của spec 4.3 — khoá không bao giờ chạm `global.db`/`project.db` trên đĩa
#[test]
fn saving_a_key_leaves_no_trace_on_disk_in_global_db_or_project_db() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = temp_dir("key-disk-leak");
    let global = open_global(&root);
    let opened = open_work_real(&root);
    reset_key_to_not_configured();

    const SECRET: &str = "sk-kiem-tra-khong-lot-ra-dia-135790";
    ai_config_save_key(AiConfigTier::Global, SECRET).expect("luu khoa that bai");

    // Đóng CẢ HAI kho trước khi đọc bytes trên đĩa — WAL/SHM chỉ được đọc đáng tin khi
    // không còn kết nối nào mở (cùng luật 4 luật đầu tệp: "Drop Store/OpenWork TRUOC khi
    // xoa thu muc").
    drop(opened.store);
    drop(global);

    let mut offenders: Vec<String> = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).unwrap_or_else(|e| panic!("doc {}: {e}", dir.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|e| panic!("duyet {}: {e}", dir.display()));
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let bytes = fs::read(&path).unwrap_or_else(|e| panic!("doc {}: {e}", path.display()));
            if bytes.windows(SECRET.len()).any(|w| w == SECRET.as_bytes()) {
                offenders.push(path.display().to_string());
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "khoa API xuat hien tren dia o: {offenders:?} -- FR65/FR67/NFR11 cam tuyet doi dieu nay"
    );

    cleanup_dir(&root);
}
