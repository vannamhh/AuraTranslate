//! Hợp đồng dây của AD-21 — Story 1.5, AC1 và AC3.
//!
//! ⚠️ Tệp riêng có chủ ý. `config_invariants.rs` khai phạm vi của nó ở dòng 1
//! (*"bất biến cấu hình của Story 1.2"*) và trộn vào là làm hỏng đúng thứ khiến nó
//! đọc được.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! NGHIỆM THU AC3 BẰNG `serde_json` — VÌ ĐÓ CHÍNH LÀ THỨ CHẠY TRÊN DÂY
//! ─────────────────────────────────────────────────────────────────────────────
//! Tauri v2 đưa giá trị trả về của `#[tauri::command]` qua IPC bằng **chính
//! `serde_json`**, không có tầng biến đổi nào chen giữa (kiểm trên `tauri = 2.11.5`).
//! `serde_json::to_value(…)` cho ra **đúng byte** mà frontend sẽ nhận. Đây là bằng
//! chứng về dây, không phải một phép mô phỏng.
//!
//! Đừng dựng một `#[tauri::command]` giả để "chứng minh cho thật". Nó là mã sản
//! phẩm không ai gọi; chạy nó cần một webview, tức một bước CI cần phiên đồ hoạ và
//! một lượt biên dịch profile `dev` riêng (đắt nhất trên macOS, hệ số ×10).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CẬP NHẬT STORY 1.8 — GIÁ TRỊ ĐEM SERIALIZE NAY ĐẾN TỪ ĐƯỜNG SẢN PHẨM
//! ─────────────────────────────────────────────────────────────────────────────
//! Lúc viết, `src-tauri/src/commands/` chưa có một hàm IPC nào, nên
//! `ipc_error_wire_shape` dựng một `IpcError` bằng tay rồi khẳng định về chính nó — một
//! **mệnh đề vòng** mà `deferred-work.md:49` giao đích danh Story 1.8 phải chữa.
//!
//! Nay dự án có hai command thật, và cả hai đều là vỏ mỏng của một **hàm thuần** nhận
//! `Option<&Store>`. Test gọi thẳng hàm thuần đó: không cần webview, không cần fixture,
//! và thứ được serialize là thứ máy người dùng phát ra.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use auratranslate_lib::commands::config::{BootstrapConfig, bootstrap_config};
use auratranslate_lib::core::i18n::{IpcError, MessageKey};

/// `CARGO_MANIFEST_DIR` trỏ `src-tauri/`, nên phải lùi một cấp. Cùng khuôn
/// `config_invariants.rs`.
fn vi_json_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src")
        .join("i18n")
        .join("vi.json")
}

/// Đọc `vi.json` thành một map PHẲNG.
///
/// ⚠️ Kiểu đích là `BTreeMap<String, String>` chứ không phải `serde_json::Value`, và
/// đó là một phép kiểm chứ không phải một lựa chọn cho tiện: một object lồng
/// (`{"lookup": {"empty_result": "…"}}`) hay một giá trị không phải chuỗi làm
/// deserialize gãy ngay tại đây. Hình dạng phẳng của AC1 vì thế được cưỡng chế ở cả
/// hai phía — `scripts/check-i18n.mjs` Kiểm B nói bằng thông báo rõ ràng cho người
/// sửa, chỗ này chặn bằng kiểu cho người viết Rust.
///
/// không `panic!` kèm đường dẫn, không `unwrap()` trần: một lỗi đọc file phải chỉ ra
/// được đang đọc cái gì.
fn read_vi_json() -> BTreeMap<String, String> {
    let path = vi_json_path();
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("không đọc được {}: {e}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|e| {
        panic!(
            "không parse được {} thành object phẳng `khoá chấm -> chuỗi`: {e}\n\
             AC1 đòi `vi.json` là object PHẲNG, mọi giá trị là chuỗi. Object lồng là sai hình dạng.",
            path.display()
        )
    })
}

/// AC3 — bốn trường, đúng chính tả, `message_key` ra chuỗi khoá chấm.
///
/// 🔴 Phép kiểm quan trọng nhất ở đây là `keys()` so với **bốn chuỗi nguyên văn**.
/// `#[serde(rename_all = "camelCase")]` trên `IpcError` biên dịch sạch và không làm
/// đỏ bất cứ thứ gì khác trong repo — chỉ dòng này đỏ.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 ĐÃ CHỮA MỆNH ĐỀ VÒNG — Story 1.8 đóng `deferred-work.md:49`
/// ─────────────────────────────────────────────────────────────────────────────
/// Bản trước dựng một `IpcError` bằng tay ngay tại đây rồi khẳng định về **chính cái
/// nó vừa dựng**. Nó chứng minh `Serialize` của `IpcError` đúng, và không chứng minh
/// được rằng có **đường sản phẩm nào** thật sự phát ra hình dạng đó — hai mệnh đề khác
/// nhau, và mệnh đề thứ hai là thứ AD-21 nói.
///
/// Nay giá trị đến từ `commands::config::bootstrap_config(None)` — **đường sản phẩm
/// thật**, đúng hàm mà `#[tauri::command]` cùng tên bọc lại, chạy đúng nhánh mà một
/// `$APPDATA` không ghi được sẽ chạy trên máy người dùng.
///
/// Và **không** phải một command giả dựng lên cho vừa lời hứa cũ:
/// `deferred-work.md:49` cấm đích danh đường đó. Hàm này nhận `Option<&Store>` để test
/// gọi được **mà không cần webview** (§Quyết định #6), chứ không phải để test có một
/// thứ riêng để gọi.
#[test]
fn ipc_error_wire_shape() {
    // `None` = kho chưa bao giờ được `manage` — nhánh mà `lib.rs::open_global_store` để
    // ngỏ khi `$APPDATA` không ghi được, và là bề mặt lỗi mà `deferred-work.md:177` chờ.
    let err = bootstrap_config(None).expect_err(
        "`bootstrap_config(None)` phải trả lỗi: không có kho thì không có gì để đọc. \
         Một `Ok` ở đây nghĩa là hàm đã im lặng bịa ra một cấu hình.",
    );

    assert_eq!(
        err.code(),
        "store.open_failed",
        "kho vắng mặt phải nói đúng tên của nó — frontend rẽ nhánh trên `code`"
    );
    assert_eq!(err.message_key(), MessageKey::StoreOpenFailed);
    assert!(
        !err.retryable(),
        "một kho chưa bao giờ mở được không tự mở ra ở lần bấm thứ hai — \
         `retryable` ở đây là nói dối (AD-22)"
    );

    let value = serde_json::to_value(&err).expect("IpcError phải serialize được");
    let object = value
        .as_object()
        .expect("IpcError phải serialize thành một JSON object");

    let keys: Vec<&str> = object.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        vec!["code", "message_key", "params", "retryable"],
        "AD-21 phát biểu bốn trường NGUYÊN VĂN, snake_case. Nhận được: {keys:?}. \
         Nghi phạm số một: `#[serde(rename_all = \"camelCase\")]` trên `IpcError` — \
         nó biến `message_key` thành `messageKey` và mọi chỗ đọc theo AD-21 nhận `undefined`."
    );

    assert_eq!(
        object.get("message_key").and_then(|v| v.as_str()),
        Some("err.store.open_failed"),
        "`message_key` phải serialize thành KHOÁ CHẤM, không phải tên biến thể. \
         Nhận `\"IoReadFailed\"` nghĩa là `Serialize` viết tay đã bị thay bằng `#[derive(Serialize)]` \
         — một chuỗi hợp lệ mà frontend không tra được, tức hỏng im lặng."
    );

    assert_eq!(
        object.get("params").and_then(|v| v.as_object()),
        Some(&serde_json::Map::from_iter([(
            "store".to_owned(),
            serde_json::Value::String("global".to_owned())
        )])),
        "`params` phải là object `chuỗi -> chuỗi` và mang DỮ LIỆU (tên kho), không mang \
         câu — `detail` thô của SQLite không bao giờ đi vào đây (Story 1.7 §Completion Notes #5)"
    );
    assert_eq!(
        object.get("retryable").and_then(|v| v.as_bool()),
        Some(false),
        "`retryable` phải là boolean thật, không phải chuỗi \"false\""
    );
    assert_eq!(
        object.get("code").and_then(|v| v.as_str()),
        Some("store.open_failed"),
        "`code` phải đi nguyên văn — frontend rẽ nhánh trên nó"
    );

    // 🔴 Đối chứng dương của việc chữa mệnh đề vòng: đường thành công cũng phải serialize
    // đúng hình dạng đã hứa. Không có nó thì `bootstrap_config` được phép chỉ đúng ở
    // nhánh lỗi — tức nửa đường sản phẩm vẫn chưa ai quan sát.
    let ok_shape = serde_json::to_value(BootstrapConfig {
        theme: "light".to_owned(),
        mode: "library".to_owned(),
        shortcuts: BTreeMap::new(),
        layout_presets: BTreeMap::new(),
        // ⚠️ Story 1.14 — trường thứ năm. Struct literal ở đây KHÔNG biên dịch được cho tới
        // khi nó có mặt, và đó là hành vi ĐÚNG: một trường mới đi qua IPC phải làm ai đó
        // dừng lại. Đừng "sửa" bằng `..Default::default()` — nó sẽ nuốt luôn trường thứ
        // sáu, thứ bảy, và danh sách khoá đóng băng dưới đây mất hết giá trị.
        workspace_layout: String::new(),
        // ⚠️ Story 1.19 — trường thứ **sáu**, và nó đúng là lượt dừng lại mà chú thích ngay
        // trên vừa hứa. Tên trên dây phải ở lại `snake_case`: một
        // `#[serde(rename_all = "camelCase")]` biến nó thành `dictSourcesDisabled`,
        // `src/config/bootstrap.ts` nhận `undefined`, **không lỗi nào được ném**, và lựa
        // chọn tắt nguồn của người dùng biến mất sau mỗi lần khởi động lại (Bẫy 1).
        dict_sources_disabled: String::new(),
        // ⚠️ Story 3.5 — trường thứ **bảy**. Cùng lời dừng như hai trường trên: một trường
        // mới đi qua IPC phải làm ai đó dừng lại và đối chiếu danh sách khoá đóng băng.
        glossary_scan_threshold: 5,
    })
    .expect("BootstrapConfig phải serialize được");
    // ⚠️ Sắp xếp trước khi so: `serde_json::Map` là `BTreeMap` hay `IndexMap` tuỳ feature
    // `preserve_order`, tức thứ tự khoá là chi tiết cài đặt của một crate. Mệnh đề ở đây
    // là về **chính tả tên khoá**, không về thứ tự — buộc nó vào thứ tự là tự tạo một ca
    // đỏ giả vào ngày ai đó bật một feature không liên quan.
    let mut ok_keys: Vec<&str> = ok_shape
        .as_object()
        .expect("BootstrapConfig phải serialize thành một JSON object")
        .keys()
        .map(String::as_str)
        .collect();
    ok_keys.sort_unstable();
    assert_eq!(
        ok_keys,
        vec![
            "dict_sources_disabled",
            "glossary_scan_threshold",
            "layout_presets",
            "mode",
            "shortcuts",
            "theme",
            "workspace_layout",
        ],
        "khoá trên dây là `snake_case`. Nhận được: {ok_keys:?}. Nghi phạm số một: \
         `#[serde(rename_all = \"camelCase\")]` trên `BootstrapConfig` — nó biến \
         `layout_presets` thành `layoutPresets` và chỗ đọc nhận `undefined`."
    );

    // Không văn bản hiển thị nào được đi qua dây. Mệnh đề trung tâm của AD-21, và
    // nó kiểm được bằng máy: chuỗi hiển thị của dự án là tiếng Việt có dấu.
    //
    // ⚠️ Một BỘ KÝ TỰ TƯỜNG MINH, không phải một dải `'à'..='ỹ'`. Dải đó chạy từ
    // U+00E0 tới U+1EF9 và nuốt trọn Hy Lạp, Cyrillic, Do Thái, Ả Rập — một đường dẫn
    // Cyrillic trong `params` sẽ bị tuyên là "văn bản hiển thị" và test đỏ vì một lý
    // do không có thật. Cùng bộ 134 ký tự mà `scripts/check-i18n.mjs` dùng.
    let wire = serde_json::to_string(&err).expect("IpcError phải serialize được");
    const VI_DIACRITICS: &str = "àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđ";
    let vietnamese_diacritic = wire
        .chars()
        .any(|c| VI_DIACRITICS.contains(c) || VI_DIACRITICS.contains(c.to_lowercase().next().unwrap_or(c)));
    assert!(
        !vietnamese_diacritic,
        "payload lỗi mang ký tự có dấu tiếng Việt ⇒ có văn bản hiển thị trên dây. \
         AD-21: *Rust không bao giờ trả về văn bản hiển thị*. Payload: {wire}"
    );
}

/// AC1 — mọi khoá Rust được phép phát ra đều có trong `vi.json`.
///
/// ⚠️ Chỉ kiểm MỘT CHIỀU, có chủ ý. Chiều ngược lại (`vi.json` có khoá mà Rust không
/// biết) là **bình thường**: phần lớn chuỗi giao diện chỉ frontend dùng và không lỗi
/// nào phát ra chúng.
#[test]
fn every_message_key_exists_in_vi_json() {
    let catalog = read_vi_json();

    // Ngưỡng sàn — "cây rỗng không phải cây sạch", thừa kế từ `check-deps.mjs`. Một
    // `MessageKey::ALL` rỗng làm vòng lặp dưới đây xanh mà không kiểm gì cả, và một
    // `vi.json` rỗng thì đã đỏ ở vòng lặp. Chặn cả hai đường.
    assert!(
        !MessageKey::ALL.is_empty(),
        "`MessageKey::ALL` rỗng — vòng lặp dưới đây sẽ xanh mà không kiểm gì. \
         Nhiều khả năng `message_keys!` đã bị gỡ hoặc khai rỗng."
    );

    let missing: Vec<&str> = MessageKey::ALL
        .iter()
        .map(|k| k.as_str())
        .filter(|k| !catalog.contains_key(*k))
        .collect();

    assert!(
        missing.is_empty(),
        "{} khoá có trong danh mục `MessageKey` nhưng KHÔNG có trong `src/i18n/vi.json`: {missing:?}\n\
         Frontend sẽ hiện khoá nguyên văn ra màn hình (AC4 — đúng hành vi, sai kết quả).\n\
         Thêm chúng vào `vi.json`, đừng gỡ khỏi `MessageKey::ALL`.",
        missing.len()
    );
}

/// `ALL` và `as_str()` sinh từ CÙNG một khai báo — khẳng định điều đó thành một phép
/// kiểm để nó không lặng lẽ thôi đúng.
///
/// Hai khoá trùng `as_str()` là một lỗi gõ phím trong `message_keys!`, và hậu quả của
/// nó đúng bằng hậu quả của một khoá thiếu: một trong hai lỗi sẽ hiện ra câu của lỗi
/// kia, mà không gì báo.
#[test]
fn message_key_catalog_has_no_duplicate_keys() {
    let mut seen: Vec<&str> = MessageKey::ALL.iter().map(|k| k.as_str()).collect();
    let before = seen.len();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(
        seen.len(),
        before,
        "`message_keys!` khai trùng khoá chấm — hai biến thể trỏ về cùng một chuỗi. \
         Danh mục sau khi khử trùng: {seen:?}"
    );

    // Hình dạng khoá phải khớp đúng luật mà `scripts/check-i18n.mjs` Kiểm B áp cho
    // `vi.json`: `^[a-z0-9]+(\.[a-z0-9_]+)+$`.
    //
    // ⚠️ Bản trước kiểm bằng `contains('.')` cộng một bộ lọc ký tự, và nó LỎNG HƠN Kiểm
    // B thật: `err_io.read_failed` (gạch dưới ở đoạn ĐẦU) và `err.` (đoạn cuối rỗng) qua
    // được ở đây rồi đỏ ở cổng. Hai cổng bất đồng về cùng một bất biến là một cổng cộng
    // một cái bẫy — nên chỗ này áp đúng văn phạm ấy, viết ra thành từng đoạn.
    for key in MessageKey::ALL {
        let s = key.as_str();
        let mut segments = s.split('.');
        let head = segments.next().unwrap_or("");
        let tail: Vec<&str> = segments.collect();
        let head_ok = !head.is_empty()
            && head
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
        let tail_ok = !tail.is_empty()
            && tail.iter().all(|seg| {
                !seg.is_empty()
                    && seg
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            });
        assert!(
            head_ok && tail_ok,
            "khoá `{s}` sai hình dạng — phải khớp `^[a-z0-9]+(\\.[a-z0-9_]+)+$`, đúng văn \
             phạm mà `scripts/check-i18n.mjs` Kiểm B áp cho `vi.json`. Ví dụ đúng: \
             `err.io.read_failed`. Bắt được: `err_io.read_failed`, `err.`, `Err.X`."
        );
    }
}

/// 🔴 CHỖ NỐI DUY NHẤT giữa `params` phía Rust và placeholder trong `vi.json` — và
/// trước lượt review này thì không có chỗ nào cả.
///
/// Lỗ hổng: `message_key` có kiểu nên khoá sai không biên dịch được, nhưng `params`
/// là `BTreeMap` tự do. `every_message_key_exists_in_vi_json` chỉ hỏi *"khoá có mặt
/// không"*; Kiểm C của cổng chỉ hỏi *"placeholder có đúng hình dạng không"*. Không ai
/// hỏi *"khoá này cần những tham số nào, và chỗ gọi có đưa đủ không"* — nên
/// `params: BTreeMap::new()` cho `IoReadFailed` xanh cả ba cổng và người dùng đọc được
/// nguyên văn `{path}`.
///
/// Kiểm **cả hai chiều**, vì mỗi chiều hỏng một kiểu khác nhau:
/// - bảng thiếu tham số mà chuỗi có ⇒ `IpcError::new` không chặn được chỗ gọi thiếu
/// - bảng thừa tham số mà chuỗi không có ⇒ `new` đòi một thứ vô nghĩa và mọi chỗ gọi
///   hợp lệ rơi về `Unknown`
#[test]
fn every_message_key_declares_the_params_its_string_needs() {
    let catalog = read_vi_json();

    for key in MessageKey::ALL {
        let template = catalog
            .get(key.as_str())
            .unwrap_or_else(|| panic!("`{}` không có trong vi.json", key.as_str()));

        // Cùng dải với `PLACEHOLDER_RE` của `resolve.ts` và Kiểm C: `{ten_tham_so}`.
        let mut in_string: Vec<String> = Vec::new();
        let mut rest = template.as_str();
        while let Some(open) = rest.find('{') {
            rest = &rest[open + 1..];
            let Some(close) = rest.find('}') else { break };
            let name = &rest[..close];
            let valid = !name.is_empty()
                && name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
            if valid {
                in_string.push(name.to_owned());
            }
            rest = &rest[close + 1..];
        }
        in_string.sort_unstable();
        in_string.dedup();

        let mut declared: Vec<String> = key.required_params().iter().map(|s| (*s).to_owned()).collect();
        declared.sort_unstable();
        declared.dedup();

        assert_eq!(
            declared,
            in_string,
            "`{}` — bảng `required_params` khai {:?} nhưng chuỗi trong `vi.json` dùng {:?}.\n\
             Chuỗi: \"{}\"\n\
             Hai danh sách này PHẢI khớp: `IpcError::new` chặn chỗ gọi bằng bảng, còn thứ \
             người dùng đọc là chuỗi. Lệch một cái tên là một placeholder thô trên màn hình \
             hoặc một lời gọi hợp lệ bị đẩy về `err.unknown`.",
            key.as_str(),
            declared,
            in_string,
            template,
        );
    }
}

/// `IpcError::new` phải NỔ ở debug khi thiếu tham số — đây là chỗ lỗi lập trình được
/// bắt, và `cargo test` chạy ở profile debug nên phép kiểm này thật sự chạy.
///
/// ⚠️ Ở release `debug_assert!` biến mất và khoá rơi về `Unknown` thay vì panic: xem
/// doc-comment của `IpcError::new` để biết vì sao (`panic = "abort"` + writer nối tiếp
/// của AD-11/AD-12 — một panic trong đường BÁO LỖI giết cả tiến trình).
#[test]
#[should_panic(expected = "err.io.read_failed")]
fn ipc_error_new_rejects_missing_params_in_debug() {
    let _ = IpcError::new(
        "io.read_failed",
        MessageKey::IoReadFailed,
        BTreeMap::new(),
        false,
    );
}

/// **THÊM Story 5.3.** Đóng băng tên trường của các struct wire của `commands::library` —
/// cùng khuôn phần `ok_shape`/`ok_keys` của [`ipc_error_wire_shape`] ở trên: một trường mới
/// đi qua IPC mà không ai đối chiếu là đúng thứ ca này tồn tại để chặn.
///
/// 🔵 **SỬA (2026-08-27, phán quyết Ice #3) — thêm `ConflictEntry`, `conflicts` đổi hình
/// dạng.** `RescanReport.conflicts` không còn là một `usize` nén — AC4 nói "phát hiện VÀ
/// cảnh báo", và một con số trần không mang đủ dữ kiện cho vế "cảnh báo" (không nói được CHỖ
/// NÀO trùng). Nay nó là `Vec<ConflictEntry>`, đóng băng CẢ hai tầng khoá (top-level VÀ một
/// mục `conflicts`) cùng lượt với `orphans`/`OrphanEntry`.
#[test]
fn library_wire_structs_keep_snake_case_field_names() {
    let report = auratranslate_lib::commands::library::RescanReport {
        root: "/tmp/library".to_owned(),
        // P1 (vòng rà bốn lớp 2026-08-27) -- trường mới, đóng băng cùng lượt.
        root_missing: false,
        indexed: 1,
        // Phán quyết Ice #3 -- không còn một `usize` trần.
        conflicts: vec![auratranslate_lib::commands::library::ConflictEntry {
            work_id: "id-2".to_owned(),
            kept_path: "/tmp/kept.atproj".to_owned(),
            duplicate_path: "/tmp/duplicate.atproj".to_owned(),
        }],
        skipped: 0,
        orphans: vec![auratranslate_lib::commands::library::OrphanEntry {
            work_id: "id-1".to_owned(),
            name: "Tên".to_owned(),
            atproj_path: "/tmp/x.atproj".to_owned(),
        }],
        // 🔵 THÊM (retro Epic 5, AI-2/AI-3 — 2026-09-03) -- trường mới, đóng băng cùng lượt.
        text_skipped: vec![auratranslate_lib::commands::library::TextSkippedEntry {
            work_id: "id-3".to_owned(),
            reason: "schema_too_old".to_owned(),
        }],
    };
    let value = serde_json::to_value(&report).expect("RescanReport phải serialize được");
    let mut top_keys: Vec<&str> = value
        .as_object()
        .expect("RescanReport phải serialize thành object")
        .keys()
        .map(String::as_str)
        .collect();
    top_keys.sort_unstable();
    assert_eq!(
        top_keys,
        vec!["conflicts", "indexed", "orphans", "root", "root_missing", "skipped", "text_skipped"],
        "khoá trên dây của RescanReport là snake_case. Nhận được: {top_keys:?}."
    );

    let text_skipped_value = value
        .get("text_skipped")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .expect("text_skipped phải mang ít nhất một mục cho ca test này");
    let mut text_skipped_keys: Vec<&str> = text_skipped_value
        .as_object()
        .expect("một mục text_skipped phải serialize thành object")
        .keys()
        .map(String::as_str)
        .collect();
    text_skipped_keys.sort_unstable();
    assert_eq!(
        text_skipped_keys,
        vec!["reason", "work_id"],
        "khoá trên dây của TextSkippedEntry là snake_case. Nhận được: {text_skipped_keys:?}."
    );

    let orphan_value = value
        .get("orphans")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .expect("orphans phải mang ít nhất một mục cho ca test này");
    let mut orphan_keys: Vec<&str> = orphan_value
        .as_object()
        .expect("một mục orphans phải serialize thành object")
        .keys()
        .map(String::as_str)
        .collect();
    orphan_keys.sort_unstable();
    assert_eq!(
        orphan_keys,
        vec!["atproj_path", "name", "work_id"],
        "khoá trên dây của OrphanEntry là snake_case. Nhận được: {orphan_keys:?}. Nghi phạm số \
         một: `#[serde(rename_all = \"camelCase\")]` đặt nhầm lên struct này."
    );

    let conflict_value = value
        .get("conflicts")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .expect("conflicts phải mang ít nhất một mục cho ca test này");
    let mut conflict_keys: Vec<&str> = conflict_value
        .as_object()
        .expect("một mục conflicts phải serialize thành object")
        .keys()
        .map(String::as_str)
        .collect();
    conflict_keys.sort_unstable();
    assert_eq!(
        conflict_keys,
        vec!["duplicate_path", "kept_path", "work_id"],
        "khoá trên dây của ConflictEntry là snake_case. Nhận được: {conflict_keys:?}. Nghi phạm \
         số một: `#[serde(rename_all = \"camelCase\")]` đặt nhầm lên struct này."
    );
}

/// **THÊM Story 5.4.** Đóng băng tên trường `snake_case` của `WorkRow`/`WorkListReport` —
/// cùng lý lẽ và cùng khuôn [`library_wire_structs_keep_snake_case_field_names`] ngay trên:
/// một trường mới đi qua IPC mà không ai đối chiếu là đúng thứ ca này tồn tại để chặn.
#[test]
fn library_work_list_wire_structs_keep_snake_case_field_names() {
    let report = auratranslate_lib::commands::library::WorkListReport {
        total: 4,
        matched: 1,
        works: vec![auratranslate_lib::commands::library::WorkRow {
            work_id: "id-1".to_owned(),
            atproj_path: "/tmp/x.atproj".to_owned(),
            name: "Tên".to_owned(),
            source_lang: "zh".to_owned(),
            genre: "".to_owned(),
            created_at: "2026-08-01T00:00:00.000Z".to_owned(),
            updated_at: "2026-08-01T00:00:00.000Z".to_owned(),
            chapter_count: 1,
            status: Some("paused".to_owned()),
            status_is_override: true,
            // 🔵 THÊM (2026-08-28, Story 5.5).
            chapter_done_count: Some(1),
        }],
        // 🔵 THÊM (2026-08-28, Story 5.6).
        genres: vec!["Tiên hiệp".to_owned()],
        source_langs: vec!["zh".to_owned()],
    };
    let value = serde_json::to_value(&report).expect("WorkListReport phải serialize được");
    let mut top_keys: Vec<&str> =
        value.as_object().expect("phải serialize thành object").keys().map(String::as_str).collect();
    top_keys.sort_unstable();
    assert_eq!(
        top_keys,
        vec!["genres", "matched", "source_langs", "total", "works"],
        "khoá trên dây của WorkListReport là snake_case. Nhận được: {top_keys:?}."
    );

    let work_value = value
        .get("works")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .expect("works phải mang ít nhất một mục cho ca test này");
    let mut work_keys: Vec<&str> = work_value
        .as_object()
        .expect("một mục works phải serialize thành object")
        .keys()
        .map(String::as_str)
        .collect();
    work_keys.sort_unstable();
    assert_eq!(
        work_keys,
        vec![
            "atproj_path",
            "chapter_count",
            "chapter_done_count",
            "created_at",
            "genre",
            "name",
            "source_lang",
            "status",
            "status_is_override",
            "updated_at",
            "work_id",
        ],
        "khoá trên dây của WorkRow là snake_case. Nhận được: {work_keys:?}. Nghi phạm số một: \
         `#[serde(rename_all = \"camelCase\")]` đặt nhầm lên struct này."
    );
}

/// **THÊM Story 5.9, mở rộng Story 5.10.** Đóng băng tên trường `snake_case` của
/// `SearchHit`/`SearchReport` — cùng lý lẽ và cùng khuôn
/// [`library_work_list_wire_structs_keep_snake_case_field_names`] ngay trên.
/// 🔵 SỬA (2026-08-29, Story 5.10) — bốn trường MỚI: `SearchHit::match_kind`,
/// `SearchReport::{mode, effective_mode, widened}`.
#[test]
fn library_search_wire_structs_keep_snake_case_field_names() {
    let report = auratranslate_lib::commands::library::SearchReport {
        hits: vec![auratranslate_lib::commands::library::SearchHit {
            work_id: "id-1".to_owned(),
            work_name: "Tên".to_owned(),
            chapter_id: 7,
            chapter_ord: 1,
            chapter_title: Some("Chương Một".to_owned()),
            segment_id: Some(42),
            field: "target".to_owned(),
            snippet: "‹má› của tôi".to_owned(),
            match_kind: "exact".to_owned(),
        }],
        total: 1,
        indexed_segments: 5,
        short_query: false,
        truncated: true,
        mode: "exact".to_owned(),
        effective_mode: "exact".to_owned(),
        widened: false,
        // 🔵 THÊM (retro Epic 5, AI-3 — 2026-09-03) -- hai trường mới, đóng băng cùng lượt.
        works_total: 47,
        works_with_text: 12,
    };
    let value = serde_json::to_value(&report).expect("SearchReport phải serialize được");
    let mut top_keys: Vec<&str> =
        value.as_object().expect("phải serialize thành object").keys().map(String::as_str).collect();
    top_keys.sort_unstable();
    assert_eq!(
        top_keys,
        vec![
            "effective_mode",
            "hits",
            "indexed_segments",
            "mode",
            "short_query",
            "total",
            "truncated",
            "widened",
            "works_total",
            "works_with_text",
        ],
        "khoá trên dây của SearchReport là snake_case. Nhận được: {top_keys:?}."
    );

    let hit_value = value
        .get("hits")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .expect("hits phải mang ít nhất một mục cho ca test này");
    let mut hit_keys: Vec<&str> =
        hit_value.as_object().expect("một mục hits phải serialize thành object").keys().map(String::as_str).collect();
    hit_keys.sort_unstable();
    assert_eq!(
        hit_keys,
        vec![
            "chapter_id",
            "chapter_ord",
            "chapter_title",
            "field",
            "match_kind",
            "segment_id",
            "snippet",
            "work_id",
            "work_name",
        ],
        "khoá trên dây của SearchHit là snake_case. Nhận được: {hit_keys:?}. Nghi phạm số một: \
         `#[serde(rename_all = \"camelCase\")]` đặt nhầm lên struct này."
    );
}

/// **THÊM Story 5.4.** Đóng băng tên trường `snake_case` của `WorkLifecycle` — struct trả về
/// của cả ba lệnh vòng đời (`read_work_lifecycle`/`set_chapter_status`/
/// `set_work_status_override`).
#[test]
fn lifecycle_wire_struct_keeps_snake_case_field_names() {
    let lifecycle = auratranslate_lib::commands::lifecycle::WorkLifecycle {
        status: Some("paused".to_owned()),
        status_is_override: true,
    };
    let value = serde_json::to_value(&lifecycle).expect("WorkLifecycle phải serialize được");
    let mut keys: Vec<&str> =
        value.as_object().expect("phải serialize thành object").keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec!["status", "status_is_override"],
        "khoá trên dây của WorkLifecycle là snake_case. Nhận được: {keys:?}."
    );
}

/// **THÊM Story 5.7.** Đóng băng tên trường `snake_case` của `ChapterRow` — hàng của danh
/// sách Chương (`list_chapters`). Cùng lý lẽ mọi ca đóng băng khoá khác ở tệp này: một
/// trường mới đi qua IPC mà không ai đối chiếu là đúng thứ ca này tồn tại để chặn.
#[test]
fn chapter_row_wire_struct_keeps_snake_case_field_names() {
    let row = auratranslate_lib::commands::chapter::ChapterRow {
        chapter_id: 1,
        ord: 1,
        title: None,
        status: "not_started".to_owned(),
        segment_count: 0,
        origin_author: None,
        origin_site_name: None,
        origin_url: None,
        origin_published_at: None,
    };
    let value = serde_json::to_value(&row).expect("ChapterRow phải serialize được");
    let mut keys: Vec<&str> =
        value.as_object().expect("phải serialize thành object").keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "chapter_id",
            "ord",
            "origin_author",
            "origin_published_at",
            "origin_site_name",
            "origin_url",
            "segment_count",
            "status",
            "title"
        ],
        "khoá trên dây của ChapterRow là snake_case. Nhận được: {keys:?}. Nghi phạm số một: \
         `#[serde(rename_all = \"camelCase\")]` đặt nhầm lên struct này."
    );
}

/// **THÊM Story 5.7.** Đóng băng tên trường `snake_case` của `OpenedWork` — kết quả của
/// `open_work` (mở lại một `.atproj` đã có trên đĩa).
#[test]
fn opened_work_wire_struct_keeps_snake_case_field_names() {
    let meta = auratranslate_lib::core::library::WorkMeta {
        meta_schema_version: auratranslate_lib::core::library::META_SCHEMA_VERSION,
        work_id: "id-1".to_owned(),
        name: "Ten".to_owned(),
        source_lang: "zh".to_owned(),
        genre: String::new(),
        created_at: "2026-08-01T00:00:00.000Z".to_owned(),
        updated_at: "2026-08-01T00:00:00.000Z".to_owned(),
        chapter_count: 1,
        status: Some("not_started".to_owned()),
        status_is_override: false,
        chapter_done_count: Some(0),
    };
    let opened = auratranslate_lib::commands::project::wire::OpenedWork { meta, folder: "/tmp/x.atproj".to_owned(), chapter_id: 7 };
    let value = serde_json::to_value(&opened).expect("OpenedWork phải serialize được");
    let mut keys: Vec<&str> =
        value.as_object().expect("phải serialize thành object").keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec!["chapter_id", "folder", "meta"],
        "khoá trên dây của OpenedWork là snake_case. Nhận được: {keys:?}. Nghi phạm số một: \
         `#[serde(rename_all = \"camelCase\")]` đặt nhầm lên struct này."
    );
}

/// **THÊM Story 5.7.** Đóng băng trường MỚI `caret_segment_id` của `ChapterSegments` —
/// `None` phải serialize thành `null`, không bị `#[serde(skip_serializing_if = "..")]` nào
/// nuốt mất (đúng lý lẽ AC5: webview phải phân biệt được "chưa có giá trị" với "trường vắng
/// mặt trên dây").
#[test]
fn chapter_segments_wire_struct_carries_caret_segment_id() {
    let loaded = auratranslate_lib::commands::segment::ChapterSegments {
        chapter_id: 1,
        segments: Vec::new(),
        caret_segment_id: None,
        assets: Vec::new(),
        assets_dir: "/tmp/x.atproj/assets".to_owned(),
    };
    let value = serde_json::to_value(&loaded).expect("ChapterSegments phải serialize được");
    let object = value.as_object().expect("phải serialize thành object");
    assert!(
        object.contains_key("caret_segment_id"),
        "truong `caret_segment_id` phai co mat tren day, ke ca khi None -- webview phai phan \
         biet duoc voi mot truong vang mat"
    );
    assert_eq!(
        object.get("caret_segment_id"),
        Some(&serde_json::Value::Null),
        "`caret_segment_id: None` phai serialize thanh `null`, khong bi nuot boi mot \
         `skip_serializing_if`"
    );

    let with_value = auratranslate_lib::commands::segment::ChapterSegments {
        chapter_id: 1,
        segments: Vec::new(),
        caret_segment_id: Some(42),
        assets: Vec::new(),
        assets_dir: "/tmp/x.atproj/assets".to_owned(),
    };
    let value = serde_json::to_value(&with_value).expect("ChapterSegments phải serialize được");
    assert_eq!(
        value.get("caret_segment_id"),
        Some(&serde_json::Value::from(42)),
        "`caret_segment_id: Some(42)` phai serialize thanh so 42"
    );
}

/// **THÊM Story 5.8.** Bốn vỏ tổ chức Chương phải CÓ MẶT trong `generate_handler![…]`, và
/// tên tham số của chúng phải đúng thứ `src/config/chapter.ts` gõ ở phía kia của dây.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO CA NÀY TỒN TẠI — MỘT KHOẢNG TRỐNG ĐO ĐƯỢC, KHÔNG MỘT LO XA
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ **Đo 2026-08-29:** `grep -rn "generate_handler" src-tauri/tests/` cho **0** kết quả —
/// trước ca này **không phép kiểm nào** đối chiếu danh sách handler với các `mod wire`. Một
/// vỏ viết đúng, biên dịch sạch, đi qua cả mười một cổng, mà quên một dòng ở `lib.rs` thì
/// `invoke()` trả *"command not found"* **chỉ khi người dùng bấm nút** — và bộ e2e hôm nay
/// chỉ phủ đường chuột của một phần bề mặt.
///
/// ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tưởng đã xét:** ca này đọc `lib.rs` như
/// **văn bản**, nên nó chứng minh cái tên CÓ MẶT chứ không chứng minh `tauri::generate_handler`
/// nhận đúng nó — một macro chỉ được nghiệm thu đầy đủ bằng một webview thật, thứ mà khối
/// doc-comment đầu tệp này đã phân xử là quá đắt. Nó bắt được lớp lỗi *"quên một dòng"*, và
/// chỉ lớp đó.
#[test]
fn the_four_chapter_organise_wires_are_registered_and_keep_their_parameter_names() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));

    for wire in [
        "crate::commands::chapter::wire::rename_chapter",
        "crate::commands::chapter::wire::move_chapter",
        "crate::commands::chapter::wire::merge_chapter_into_previous",
        "crate::commands::chapter::wire::split_chapter_at_segment",
    ] {
        assert!(
            lib_src.contains(wire),
            "`{wire}` phai co mat trong generate_handler! cua lib.rs. Thieu no thi invoke() tra              \"command not found\" va KHONG cong nao do -- xem doc-comment cua ca test nay."
        );
    }

    let chapter_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands")
        .join("chapter.rs");
    let chapter_src = fs::read_to_string(&chapter_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", chapter_rs.display()));

    // Ten tham so tren day. `invoke()` gui chung o dang camelCase (`chapterId`/`segmentId`/
    // `title`) du Rust nhan snake_case -- hai chieu khac nhau la cho de sai nhat tren day
    // (`src/AGENTS.md`), va `src/config/chapter.ts` la cho duy nhat go ca hai.
    for param in ["chapter_id: i64", "segment_id: i64", "title: String"] {
        assert!(
            chapter_src.contains(param),
            "vo IPC cua commands/chapter.rs phai khai `{param}` -- doi ten tham so la doi DAY,              va `src/config/chapter.ts` gui theo ten cu."
        );
    }
}

/// **THÊM Story 6.15 (FR128/AD-43).** `update_chapter_origin` phải CÓ MẶT trong
/// `generate_handler![…]`, và năm tham số của nó phải đúng thứ `src/config/chapter.ts` gõ ở
/// phía kia của dây — cùng lý lẽ và cùng khuôn
/// [`the_four_chapter_organise_wires_are_registered_and_keep_their_parameter_names`] ngay
/// trên.
#[test]
fn update_chapter_origin_wire_is_registered_and_keeps_its_parameter_names() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));

    assert!(
        lib_src.contains("crate::commands::chapter::wire::update_chapter_origin"),
        "`crate::commands::chapter::wire::update_chapter_origin` phai co mat trong \
         generate_handler! cua lib.rs. Thieu no thi invoke() tra \"command not found\" va \
         KHONG cong nao do."
    );

    let chapter_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands")
        .join("chapter.rs");
    let chapter_src = fs::read_to_string(&chapter_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", chapter_rs.display()));

    for param in [
        "chapter_id: i64",
        "author: String",
        "site_name: String",
        "url: String",
        "published_at: String",
    ] {
        assert!(
            chapter_src.contains(param),
            "vo IPC `update_chapter_origin` phai khai `{param}` -- doi ten tham so la doi DAY, \
             va `src/config/chapter.ts` gui theo ten cu."
        );
    }
}

/// **THÊM Story 6.15 (FR128/AD-43, lượt rà 2026-09-10).** `set_chapter_origin_override` phải
/// CÓ MẶT trong `generate_handler![…]`, và năm tham số của nó phải đúng thứ
/// `src/config/project.ts` gõ ở phía kia của dây — cùng lý lẽ và cùng khuôn
/// [`update_chapter_origin_wire_is_registered_and_keeps_its_parameter_names`] ngay trên.
#[test]
fn set_chapter_origin_override_wire_is_registered_and_keeps_its_parameter_names() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));

    assert!(
        lib_src.contains("crate::commands::project::wire::set_chapter_origin_override"),
        "`crate::commands::project::wire::set_chapter_origin_override` phai co mat trong \
         generate_handler! cua lib.rs. Thieu no thi invoke() tra \"command not found\" va \
         KHONG cong nao do."
    );

    let wire_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands")
        .join("project")
        .join("wire.rs");
    let wire_src = fs::read_to_string(&wire_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", wire_rs.display()));

    for param in [
        "chapter_index: usize",
        "author: Option<String>",
        "site_name: Option<String>",
        "url: Option<String>",
        "published_at: Option<String>",
    ] {
        assert!(
            wire_src.contains(param),
            "vo IPC `set_chapter_origin_override` phai khai `{param}` -- doi ten/kieu tham so la \
             doi DAY, va `src/config/project.ts` gui theo ten/kieu cu."
        );
    }
}

/// 🔴 **Hai vo XEM TRUOC phai DON `ChapterOriginOverridesState`, khong chi `Tier2BlockOverrides`.**
/// Story 6.15, vong ra 1 muc 1/2 (2026-09-10).
///
/// ⚠️ **Vi sao mot phep quet MA NGUON chu khong mot ca goi that.** Vo `#[tauri::command]` doi
/// mot `tauri::AppHandle`, ma crate test nay khong co `MockRuntime` (da ghi o
/// `project_contract.rs:925`) -- nen KHONG ca nao goi duoc vo that. Ca dau tien viet cho lo
/// hong nay (`chapter_origin_contract.rs::a_leftover_override_from_a_cancelled_url_preview_...`)
/// TU GOI `reset_chapter_origin_overrides` roi khang dinh khong ro ri: DO 2026-09-10 bang phep
/// go THAT -- binh luan ca sau `reset_chapter_origin_overrides(&app);` khoi CA SAU cho goi
/// trong `mod wire` roi chay lai -- ca do van **15/15 XANH**. No canh chinh no, khong canh ban
/// va. Phep quet duoi day thi do DUNG dong trong THAN vo, cung khuon
/// `config_invariants.rs::the_blocking_wires_run_off_the_main_thread` da dung cho `(async)`.
#[test]
fn both_preview_wires_reset_the_chapter_origin_overrides_before_building_a_preview() {
    let wire_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands")
        .join("project")
        .join("wire.rs");
    let src = fs::read_to_string(&wire_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", wire_rs.display()));

    // 🔵 SỬA 2026-09-11 (Story 6.16) — thêm vỏ xem trước song ngữ vào danh sách: nó cũng mở
    // một lượt xem trước MỚI (đọc tệp `.csv`/`.tsv` rồi `stash_pending_import_source`), cùng
    // nghĩa vụ dọn hai state override — xem doc-comment `wire::preview_bilingual_import_from_file`.
    for wire in [
        "preview_import_encoding_from_text",
        "preview_import_encoding_from_file",
        "preview_bilingual_import_from_file",
    ] {
        let signature = format!("pub fn {wire}(");
        let start = src
            .find(&signature)
            .unwrap_or_else(|| panic!("khong tim thay vo `{wire}` trong commands/project/wire.rs"));
        // Than vo = tu chu ky toi chu ky `pub fn` KE TIEP (hoac het tep).
        let rest = &src[start + signature.len()..];
        let end = rest.find("\n    pub fn ").unwrap_or(rest.len());
        let body = &rest[..end];

        // 🔴 Dem tren DONG MA, khong tren van ban tho: mot dong bi CHU THICH van con nguyen
        // chuoi trong tep, nen `body.contains(...)` se xanh cho mot lot don da bi vo hieu hoa.
        // Do 2026-09-10: phep doi chung dau tien cua chinh ca nay (chu thich sau dong goi roi
        // chay lai) cho XANH -- day la ly do lop loc duoi day ton tai. Khuon loc chep tu
        // `webimport_boundary.rs:89` (`code_lines`).
        let code_has = |needle: &str| {
            body.lines()
                .map(str::trim_start)
                .filter(|line| !line.starts_with("//") && !line.starts_with("* ") && !line.starts_with("/*"))
                .any(|line| line.contains(needle))
        };

        assert!(
            code_has("reset_tier2_block_overrides(&app);"),
            "than vo `{wire}` phai con lot don `reset_tier2_block_overrides` (Story 6.9) --              neo cho khang dinh ngay duoi"
        );
        assert!(
            code_has("reset_chapter_origin_overrides(&app);"),
            "than vo `{wire}` PHAI don `ChapterOriginOverridesState` (Story 6.15). Thieu no,              mot override con treo tu mot lot nhap URL DA HUY se duoc              `confirm_import_with_encoding` doc lai va dong dau xuat xu cua lot web bi bo len \
             Chuong cua mot Tac pham nhap tu TEP/DAN TAY -- pha hang I/O Matrix (Nhap tu \
             file / dan tay ⇒ ca bon o khong tim thay). Khong cong nao khac do dieu nay."
        );
    }
}


/// 🔴 **Story 6.16 — ba vo song ngu: dang ky theo TEN, doc luat lam sach, va vo DUNG LAI khong
/// doc lai tep.** Them o buoc nghiem thu 2026-09-11.
///
/// ⚠️ Cung ly do quet MA NGUON nhu ca ngay tren: crate test khong co `MockRuntime`, nen khong
/// ca nao goi duoc vo that. Ba dieu duoi day deu la mot DONG trong than vo ma go di van bien
/// dich va van xanh o `bilingual_import_contract.rs` (ca do goi HAM THUAN):
/// ① thieu dong dang ky o `lib.rs` ⇒ `invoke()` tra "command not found" chi khi nguoi dung bam;
/// ② thieu `resolve_cleanup_rules(&app)` ⇒ luat nguoi dung da bat khong toi duong song ngu —
///    dung trang thai ban dau cua story nay (hai cho goi truyen `Vec::new()`);
/// ③ vo dung lai goi `import_bilingual_file` ⇒ moi lan doi cot DOC LAI TEP — trai Quyet dinh
///    Ice "toggling rebuilds the preview in memory", cung la trang thai ban dau cua story nay.
#[test]
fn the_three_bilingual_import_wires_are_registered_read_cleanup_rules_and_rebuild_never_reads_the_file() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));
    let wires = [
        "preview_bilingual_import_from_file",
        "rebuild_bilingual_import_preview",
        "confirm_bilingual_import",
    ];
    for wire in wires {
        let registered = format!("crate::commands::project::wire::{wire}");
        assert!(
            lib_src.lines().map(str::trim_start).filter(|l| !l.starts_with("//")).any(|l| l.contains(&registered)),
            "`{registered}` phai co mat (khong bi chu thich) trong generate_handler! cua lib.rs"
        );
    }

    // `confirm_bilingual_import` co HAI khoi `pub fn` cung ten (ham thuan o `commands/project/mod.rs`,
    // vo IPC o `commands/project/wire.rs`), cung bay ma ca Story 6.3 ben duoi da ghi -- doc thang
    // `wire.rs` la du de tranh nham lan, khong can neo `pub mod wire {` nua.
    let wire_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands")
        .join("project")
        .join("wire.rs");
    let wire_src = fs::read_to_string(&wire_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", wire_rs.display()));

    let code_lines_of = |wire: &str| -> Vec<String> {
        let signature = format!("pub fn {wire}(");
        let start = wire_src
            .find(&signature)
            .unwrap_or_else(|| panic!("khong tim thay vo `{wire}` trong `mod wire`"));
        let rest = &wire_src[start + signature.len()..];
        let end = rest.find("\n    pub fn ").unwrap_or(rest.len());
        rest[..end]
            .lines()
            .map(str::trim_start)
            .filter(|line| !line.starts_with("//") && !line.starts_with("* ") && !line.starts_with("/*"))
            .map(str::to_owned)
            .collect()
    };

    for wire in wires {
        assert!(
            code_lines_of(wire).iter().any(|l| l.contains("resolve_cleanup_rules(&app)")),
            "than vo `{wire}` phai phan giai luat lam sach hai tang (`resolve_cleanup_rules(&app)`) — \
             thieu no thi luat nguoi dung da bat khong bao gio toi duong song ngu"
        );
    }

    let rebuild = code_lines_of("rebuild_bilingual_import_preview");
    assert!(
        rebuild.iter().any(|l| l.contains("p.shape.clone()")),
        "vo dung lai phai clone `shape` tu `PendingImportSourceState` — neo cho hai khang dinh duoi"
    );
    for forbidden in ["import_bilingual_file", "stash_pending_import_source", "std::fs"] {
        assert!(
            !rebuild.iter().any(|l| l.contains(forbidden)),
            "vo `rebuild_bilingual_import_preview` KHONG duoc goi `{forbidden}` — dung lai la tren \
             byte DA CAT luc mo, khong doc lai tep (Quyet dinh Ice 2026-09-11)"
        );
    }
}

/// **THÊM Story 6.3 (FR126).** Ba vỏ của màn xem trước bảng mã phải CÓ MẶT trong
/// `generate_handler![…]`, và tham số của chúng phải đúng thứ `src/config/project.ts` gõ ở
/// phía kia của dây — cùng lý lẽ và cùng khuôn
/// [`the_four_chapter_organise_wires_are_registered_and_keep_their_parameter_names`] ngay
/// trên (khoảng trống đo được, không một lo xa — quên MỘT dòng ở `lib.rs` thì `invoke()` trả
/// "command not found" chỉ khi người dùng bấm nút).
#[test]
fn the_three_import_encoding_preview_wires_are_registered_and_keep_their_parameter_names() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));

    for wire in [
        "crate::commands::project::wire::preview_import_encoding_from_text",
        "crate::commands::project::wire::preview_import_encoding_from_file",
        "crate::commands::project::wire::confirm_import_with_encoding",
    ] {
        assert!(
            lib_src.contains(wire),
            "`{wire}` phai co mat trong generate_handler! cua lib.rs. Thieu no thi invoke() tra              \"command not found\" va KHONG cong nao do -- xem doc-comment cua ca test nay."
        );
    }

    // 🔴 SỬA (vòng rà đối kháng 2, mục 1) — `PendingImportSourceState` phải được `app.manage(...)`
    // TRƯỚC khi webview có thể gọi bất kỳ vỏ nào ở trên, nếu không hai vỏ xem trước rơi vào
    // `try_state::<PendingImportSourceState>() == None`. Đo (2026-09-04): xoá dòng này rồi
    // `cargo test --no-fail-fast` cho exit=0/0 ca đỏ TRƯỚC khi có assert này — chức năng
    // nhập chết hoàn toàn mà không cổng nào thấy. Cùng lớp lỗi "quên một dòng trong `lib.rs`"
    // mà cụm assert `generate_handler!` ngay trên tồn tại để chặn, mở rộng sang dòng đăng ký
    // trạng thái.
    assert!(
        lib_src.contains("app.manage(crate::commands::project::PendingImportSourceState::new(None));"),
        "thieu `app.manage(crate::commands::project::PendingImportSourceState::new(None))` trong          `lib.rs` -- hai vo xem truoc roi vao nhanh state-chua-quan-ly, va (sau vong ra doi khang          2) tra loi tuong minh thay vi `eprintln!` roi van `Ok`."
    );

    // 🔴 SỬA (vòng rà đối kháng 2, mục 16) — bản trước chỉ hỏi "chuỗi `text: String` có xuất
    // hiện Ở ĐÂU ĐÓ trong tệp 2.400+ dòng không", không biết VỎ NÀO sở hữu THAM SỐ NÀO. Một
    // vỏ mới đổi tên tham số trong khi một hàm KHÁC còn mang cùng token vẫn để cổng này
    // xanh. Neo vào ĐÚNG khối `pub fn <tên>(...)` của từng vỏ, khớp TOÀN BỘ danh sách tham
    // số (thứ tự + tên + kiểu), không chỉ một chuỗi con rời rạc.
    // `confirm_import_with_encoding` có HAI khối `pub fn` cùng tên (hàm THUẦN ở
    // `commands/project/mod.rs`, vỏ IPC ở `commands/project/wire.rs`) với danh sách tham số
    // KHÁC HẲN nhau — đọc thẳng `wire.rs` là đủ để tránh khớp nhầm khối đầu tiên (hàm thuần),
    // đúng bẫy mà đối chứng dương ngay dưới chứng minh được.
    let wire_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands")
        .join("project")
        .join("wire.rs");
    let wire_src = fs::read_to_string(&wire_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", wire_rs.display()));

    for (fn_name, expected_params) in [
        // 🔵 SỬA 2026-09-04 (Story 6.4) — thêm tham số `source_lang: String` vào CẢ HAI vỏ
        // xem trước: `encoding::render_candidates` cần biết nhánh Trung/Anh để dựng bản
        // chuẩn hoá của mỗi ứng viên (`normalize::normalize`) — KHÔNG một lệnh mới, `source_lang`
        // đã có sẵn ở form phía frontend trước khi lệnh này chạy (xem doc-comment
        // `preview_import_encoding_from_text` ở `commands/project.rs`).
        //
        // 🔵 SỬA 2026-09-05 (Story 6.6) — thêm tham số `chapter_pattern:
        // Option<ChapterPatternWire>` vào CẢ BA vỏ: mẫu phân tách Chương là tham số MỖI LƯỢT
        // NHẬP (§Always spec 6.6), gửi lại ở MỌI lượt xem trước VÀ xác nhận.
        // 🔴 SỬA 2026-09-16 (Story 6.7b, Phase 4) — tham số CUỐI `destination: Option<String>`
        // thêm vào cả `preview_import_encoding_from_text`/`_from_file` (AC3: màn xem trước cần
        // biết ĐÍCH để phân giải đúng tầng Work của luật làm sạch — `resolve_cleanup_rules_for`).
        // `confirm_import_with_encoding` KHÔNG đổi (đọc đích qua `PendingImportSourceState`,
        // xem Implementation Notes Phase 2 "2026-09-16 correction" — chữ ký của nó ở đây không
        // đổi so với trước story).
        (
            "preview_import_encoding_from_text",
            "app: tauri::AppHandle,\n        text: String,\n        source_lang: String,\n        chapter_pattern: Option<super::ChapterPatternWire>,\n        destination: Option<String>,",
        ),
        // 🔵 SỬA 2026-09-15 (Story 6.6b) — tham số `path: String` đổi thành `paths:
        // Vec<String>`: reason "parameter retyped to a list", KHÔNG một lời nới lỏng — N = 1
        // vẫn build đúng `PipelineShape::Blob` y hệt hôm nay (§Always spec 6.6b), kiểu trả
        // cũng đổi rộng ra thành `FileImportBatchWire` (envelope per-item cho MỌI N).
        (
            "preview_import_encoding_from_file",
            "app: tauri::AppHandle,\n        paths: Vec<String>,\n        source_lang: String,\n        chapter_pattern: Option<super::ChapterPatternWire>,\n        destination: Option<String>,",
        ),
        (
            "confirm_import_with_encoding",
            "app: tauri::AppHandle,\n        name: String,\n        source_lang: String,\n        genre: String,\n        encoding: String,\n        chapter_pattern: Option<super::ChapterPatternWire>,",
        ),
    ] {
        let params = fn_param_list(&wire_src, fn_name);
        assert_eq!(
            normalize_param_list(&params),
            normalize_param_list(expected_params),
            "vo `{fn_name}` trong `pub mod wire` cua commands/project/wire.rs khong con dung danh sach tham so          mong doi -- doi ten/thu tu tham so la doi DAY, va `src/config/project.ts` la cho duy nhat go lai          theo dung ten/thu tu do."
        );
    }
}

/// **THÊM Story 6.7 (FR122).** Ba vỏ "Nhập từ URL bằng danh sách link" phải CÓ MẶT trong
/// `generate_handler![…]`, và tham số phải đúng thứ `src/config/project.ts` gõ ở phía kia của
/// dây — cùng khuôn [`the_three_import_encoding_preview_wires_are_registered_and_keep_their_parameter_names`]
/// ngay trên.
#[test]
fn the_three_url_import_wires_are_registered_and_keep_their_parameter_names() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));

    for wire in [
        "crate::commands::project::wire::start_url_import",
        "crate::commands::project::wire::reload_url_import_item",
        "crate::commands::project::wire::remove_url_import_item",
    ] {
        assert!(
            lib_src.contains(wire),
            "`{wire}` phai co mat trong generate_handler! cua lib.rs. Thieu no thi invoke() tra              \"command not found\" chi khi nguoi dung bam nut."
        );
    }

    assert!(
        lib_src.contains("app.manage(crate::commands::project::UrlImportItemsState::new(None));"),
        "thieu `app.manage(crate::commands::project::UrlImportItemsState::new(None))` trong `lib.rs`          -- ba vo o tren roi vao nhanh state-chua-quan-ly."
    );

    let wire_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands")
        .join("project")
        .join("wire.rs");
    let wire_src = fs::read_to_string(&wire_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", wire_rs.display()));

    for (fn_name, expected_params) in [
        // 🔴 SỬA 2026-09-16 (Story 6.7b, Phase 4) — tham số CUỐI `destination: Option<String>`
        // thêm vào `start_url_import` (mở một phiên URL MỚI ⇒ đích là giá trị người dùng vừa
        // chọn, Quyết định 1). `reload_url_import_item`/`remove_url_import_item` KHÔNG đổi —
        // chúng tinh chỉnh một danh sách ĐÃ có đích, đọc lại đích đã cất qua
        // `current_pending_destination` thay vì nhận tham số mới (xem Implementation Notes
        // Phase 2 "2026-09-16 correction").
        (
            "start_url_import",
            "app: tauri::AppHandle,\n        urls: Vec<String>,\n        source_lang: String,\n        destination: Option<String>,",
        ),
        (
            "reload_url_import_item",
            "app: tauri::AppHandle,\n        index: usize,\n        source_lang: String,",
        ),
        (
            "remove_url_import_item",
            "app: tauri::AppHandle,\n        index: usize,\n        source_lang: String,",
        ),
    ] {
        let params = fn_param_list(&wire_src, fn_name);
        assert_eq!(
            normalize_param_list(&params),
            normalize_param_list(expected_params),
            "vo `{fn_name}` trong `pub mod wire` cua commands/project/wire.rs khong con dung danh sach tham so          mong doi -- doi ten/thu tu tham so la doi DAY, va `src/config/project.ts` la cho duy nhat go lai          theo dung ten/thu tu do."
        );
    }
}

/// **THÊM Story 6.9 (FR123).** `list_domain_log` (Story 6.8) TRÔI qua trọn một story mà không
/// một cổng nào canh nó có mặt trong `generate_handler!`/`app.manage(DomainLogState)` — đóng
/// lỗ đó CÙNG LƯỢT với hai vỏ mới của story này (sửa ranh giới bóc bằng bàn phím), theo đúng
/// khuôn [`the_three_url_import_wires_are_registered_and_keep_their_parameter_names`] ngay
/// trên.
#[test]
fn the_domain_log_wire_and_the_two_tier2_block_wires_are_registered_and_keep_their_parameter_names()
{
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));

    for wire in [
        "crate::commands::project::wire::list_domain_log",
        "crate::commands::project::wire::tier2_block_set_kept",
        "crate::commands::project::wire::tier2_block_confirm_range",
    ] {
        assert!(
            lib_src.contains(wire),
            "`{wire}` phai co mat trong generate_handler! cua lib.rs. Thieu no thi invoke() tra              \"command not found\" chi khi nguoi dung bam nut."
        );
    }

    for managed in [
        "app.manage(crate::core::webimport::DomainLogState::new(Vec::new()));",
        "app.manage(crate::commands::project::Tier2BlockOverridesState::new(Vec::new()));",
    ] {
        assert!(
            lib_src.contains(managed),
            "thieu `{managed}` trong `lib.rs` -- cac vo o tren roi vao nhanh state-chua-quan-ly."
        );
    }

    let wire_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands")
        .join("project")
        .join("wire.rs");
    let wire_src = fs::read_to_string(&wire_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", wire_rs.display()));

    for (fn_name, expected_params) in [
        ("list_domain_log", "app: tauri::AppHandle"),
        (
            "tier2_block_set_kept",
            "app: tauri::AppHandle,\n        index: usize,\n        kept: bool,\n        source_lang: String,",
        ),
        (
            "tier2_block_confirm_range",
            "app: tauri::AppHandle,\n        start: usize,\n        end: usize,\n        total: usize,\n        source_lang: String,",
        ),
    ] {
        let params = fn_param_list(&wire_src, fn_name);
        assert_eq!(
            normalize_param_list(&params),
            normalize_param_list(expected_params),
            "vo `{fn_name}` trong `pub mod wire` cua commands/project/wire.rs khong con dung danh sach tham so          mong doi -- doi ten/thu tu tham so la doi DAY, va `src/config/project.ts` la cho duy nhat go lai          theo dung ten/thu tu do."
        );
    }
}

/// **THÊM Story 6.10a.** `preview_chapter_detail` (con trỏ *Chương đang chọn* — chi tiết tầng
/// 2/3 LAZY khi con trỏ dời) là lệnh IPC MỚI DUY NHẤT của story này — §Always: "lệnh mới phải
/// vào `generate_handler!` VÀ có một ca test mang tên nó", đúng khuôn hai ca ngay trên.
#[test]
fn the_preview_chapter_detail_wire_is_registered_and_keeps_its_parameter_names() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));

    assert!(
        lib_src.contains("crate::commands::project::wire::preview_chapter_detail"),
        "`crate::commands::project::wire::preview_chapter_detail` phai co mat trong          generate_handler! cua lib.rs. Thieu no thi invoke() tra \"command not found\" chi khi          nguoi dung bam nut."
    );

    let wire_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands")
        .join("project")
        .join("wire.rs");
    let wire_src = fs::read_to_string(&wire_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", wire_rs.display()));

    let params = fn_param_list(&wire_src, "preview_chapter_detail");
    assert_eq!(
        normalize_param_list(&params),
        normalize_param_list(
            "app: tauri::AppHandle,\n        chapter_index: usize,\n        encoding: String,\n        source_lang: String,\n        chapter_pattern: Option<super::ChapterPatternWire>,"
        ),
        "vo `preview_chapter_detail` trong `pub mod wire` cua commands/project/wire.rs khong con dung          danh sach tham so mong doi -- doi ten/thu tu tham so la doi DAY, va `src/config/project.ts`          la cho duy nhat go lai theo dung ten/thu tu do."
    );
}

/// Bóc danh sách tham số của khối `pub fn <fn_name>(...)` ĐẦU TIÊN trong `src` — neo vào
/// ĐÚNG chữ ký hàm đó, không phải một chuỗi con rời rạc bất kỳ đâu trong tệp. Giả định (đúng
/// cho cả ba vỏ Story 6.3): thân tham số không chứa dấu `)` nào (không kiểu generic lồng
/// ngoặc tròn) — chữ ký cắt tại dấu `)` đầu tiên SAU `pub fn <fn_name>(`.
///
/// ⚠️ `src` PHẢI được chỗ gọi thu hẹp tới đúng phạm vi trước (`pub mod wire { … }`) khi tệp
/// có nhiều khối `pub fn <fn_name>` cùng tên — hàm này khớp CÁI ĐẦU TIÊN trong `src` truyền
/// vào, không phân biệt module. Xem đối chứng dương ngay dưới cho ca bẫy đó.
fn fn_param_list(src: &str, fn_name: &str) -> String {
    let needle = format!("pub fn {fn_name}(");
    let start = src
        .find(&needle)
        .unwrap_or_else(|| panic!("khong tim thay `{needle}` trong nguon"));
    let after_open = start + needle.len();
    let close = src[after_open..]
        .find(')')
        .unwrap_or_else(|| panic!("khong tim thay dau `)` dong tham so cho `{fn_name}`"));
    src[after_open..after_open + close].to_owned()
}

/// Chuẩn hoá KHOẢNG TRẮNG của một danh sách tham số — CHỈ so TÊN/KIỂU/THỨ TỰ, không so
/// CÁCH XUỐNG DÒNG/THỤT LỀ. **THÊM 2026-09-07 (vòng rà bước 4, mục 27).**
///
/// 🔴 **SỬA — bản trước so `params.trim() == expected_params` TRÊN NGUYÊN VĂN, kể cả
/// `\n        ` (xuống dòng + thụt lề 8 dấu cách) bên trong chuỗi hằng.** `fn_param_list` bóc
/// NGUYÊN VĂN từ mã nguồn — một lượt `cargo fmt` đổi độ rộng dòng (ví dụ gộp một danh sách
/// tham số DÀI xuống MỘT dòng, hoặc đổi số dấu cách thụt lề) làm ba ca này ĐỎ dù tên/kiểu/thứ
/// tự tham số KHÔNG đổi một ký tự nào — đúng cái mà chính câu message của assert tuyên bố nó
/// canh ("đổi tên/thứ tự tham số LÀ đổi DÂY", không phải "đổi cách xuống dòng là đổi dây").
/// Một phép so nhạy với khoảng trắng KHÔNG-CÓ-NGHĨA là một LỜI KHAI SAI về điều cổng này
/// đang bảo vệ. Chuẩn hoá bằng cách gộp MỌI dải khoảng trắng (dấu cách/tab/xuống dòng) liên
/// tiếp thành một dấu cách — Rust không phân biệt hai kiểu đó về mặt cú pháp, nên chuẩn hoá
/// không thể che giấu một khác biệt THẬT về tên/kiểu/thứ tự.
fn normalize_param_list(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Đối chứng dương cho [`fn_param_list`] — khuôn `segment_encoding_boundary.rs`: chứng minh
/// hàm bóc THẬT SỰ khớp đúng khối, không khớp lung tung/khớp rỗng oan; VÀ chứng minh cạm
/// bẫy "hai khối cùng tên" là có thật (đúng hình dạng `confirm_import_with_encoding` trong
/// `commands/project.rs`) — không thu hẹp phạm vi trước thì hàm khớp nhầm khối ĐẦU TIÊN.
#[test]
fn fn_param_list_would_actually_bind_to_the_right_function_block() {
    let src = "pub fn foo(a: i32, b: String) -> bool { true }\npub fn bar(c: u8) -> u8 { c }";
    assert_eq!(fn_param_list(src, "foo"), "a: i32, b: String");
    assert_eq!(fn_param_list(src, "bar"), "c: u8");

    // Ca bẫy — hai khối CÙNG TÊN, tham số KHÁC nhau. Không thu hẹp phạm vi ⇒ khớp nhầm.
    let duped =
        "pub fn same(x: u8) -> u8 { x }\nmod inner {\n  pub fn same(y: String, z: bool) -> bool { z }\n}";
    assert_eq!(
        fn_param_list(duped, "same"),
        "x: u8",
        "khong thu hep pham vi -- PHAI khop khoi DAU TIEN, dung boi canh cua bay nay"
    );
    let inner_start = duped.find("mod inner {").expect("fixture phai co `mod inner {`");
    assert_eq!(fn_param_list(&duped[inner_start..], "same"), "y: String, z: bool");
}

/// **THÊM Story 5.9.** `library_search` phải CÓ MẶT trong `generate_handler![…]`, và tham số
/// của nó phải đúng thứ `src/config/library.ts` gõ ở phía kia của dây — cùng lý lẽ và cùng
/// khuôn [`the_four_chapter_organise_wires_are_registered_and_keep_their_parameter_names`]
/// ngay trên (khoảng trống đo được, không một lo xa).
#[test]
fn the_library_search_wire_is_registered_and_keeps_its_parameter_names() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));

    assert!(
        lib_src.contains("crate::commands::library::wire::library_search"),
        "`crate::commands::library::wire::library_search` phai co mat trong generate_handler! \
         cua lib.rs. Thieu no thi invoke() tra \"command not found\" va KHONG cong nao do."
    );

    let library_rs =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("commands").join("library.rs");
    let library_src = fs::read_to_string(&library_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", library_rs.display()));

    // Ten tham so tren day. `invoke()` gui `query`/`limit`/`mode` (mot tu don, hai chieu trung
    // nhau o day) -- `src/config/library.ts` la cho duy nhat go ca ba. `mode` THEM o Story 5.10.
    for param in ["query: String", "limit: Option<u32>", "mode: Option<String>"] {
        assert!(
            library_src.contains(param),
            "vo IPC `library_search` (commands/library.rs) phai khai `{param}` -- doi ten tham \
             so la doi DAY, va `src/config/library.ts` gui theo ten cu."
        );
    }
}

/// **SỬA TẠI CHỖ Story 5.12** (trước: `reading_wire_structs_keep_snake_case_field_names`,
/// Story 5.11). Đóng băng tên trường `snake_case` của NĂM struct dây — `ReadingSegment`
/// (**thêm** `is_confirmed`) / `ReadingParagraph` / `ReadingChapter` (**thêm**
/// `segment_count`) / `ReadingFrontierChapter` / `ReadingFrontier` — VÀ hai chuỗi biến thể
/// của `ReadingFrontierKind` trên dây (`"next-not-done"` · `"end-of-work"`), cùng lý lẽ và
/// cùng khuôn [`library_search_wire_structs_keep_snake_case_field_names`] ở trên.
#[test]
fn reading_wire_structs_keep_snake_case_field_names() {
    let run = auratranslate_lib::commands::segment::ReadingRun {
        chapters: vec![auratranslate_lib::commands::segment::ReadingChapter {
            chapter_id: 1,
            chapter_ord: 1,
            chapter_title: Some("Chương Một".to_owned()),
            paragraphs: vec![auratranslate_lib::commands::segment::ReadingParagraph {
                segments: vec![auratranslate_lib::commands::segment::ReadingSegment {
                    id: 42,
                    source_text: "原文".to_owned(),
                    target_text: "Bản dịch".to_owned(),
                    is_confirmed: true,
                    is_marked: true,
                }],
            }],
            segment_count: 1,
            // **THÊM Story 6.14** — một ảnh, để `chapter_keys`/nội dung image dưới đây có
            // ít nhất một mục thật đi qua serde.
            images: vec![auratranslate_lib::commands::segment::ReadingImage {
                asset_id: 9,
                file_name: "9.jpg".to_owned(),
                source_url: Some("https://example.test/9.jpg".to_owned()),
                after_segment_id: Some(42),
                alt_text: Some("mo ta".to_owned()),
                caption_text: None,
            }],
        }],
        frontier: auratranslate_lib::commands::segment::ReadingFrontier {
            kind: auratranslate_lib::commands::segment::ReadingFrontierKind::NextNotDone,
            chapter: Some(auratranslate_lib::commands::segment::ReadingFrontierChapter {
                chapter_id: 2,
                chapter_ord: 2,
                chapter_title: None,
                status: "in_progress".to_owned(),
            }),
        },
        // **THÊM Story 6.14**.
        assets_dir: "/tmp/x.atproj/assets".to_owned(),
    };
    let value = serde_json::to_value(&run).expect("ReadingRun phải serialize được");
    let mut top_keys: Vec<&str> =
        value.as_object().expect("phải serialize thành object").keys().map(String::as_str).collect();
    top_keys.sort_unstable();
    assert_eq!(
        top_keys,
        vec!["assets_dir", "chapters", "frontier"],
        "khoá trên dây của ReadingRun là snake_case. Nhận được: {top_keys:?}."
    );

    let chapter_value = value
        .get("chapters")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .expect("chapters phải mang ít nhất một mục cho ca test này");
    let mut chapter_keys: Vec<&str> = chapter_value
        .as_object()
        .expect("một mục chapters phải serialize thành object")
        .keys()
        .map(String::as_str)
        .collect();
    chapter_keys.sort_unstable();
    assert_eq!(
        chapter_keys,
        vec!["chapter_id", "chapter_ord", "chapter_title", "images", "paragraphs", "segment_count"],
        "khoá trên dây của ReadingChapter là snake_case. Nhận được: {chapter_keys:?}. Nghi phạm \
         số một: `#[serde(rename_all = \"camelCase\")]` đặt nhầm lên struct này."
    );

    // **THÊM Story 6.14** — khoá trên dây của `ReadingImage`.
    let image_value = chapter_value
        .get("images")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .expect("images phải mang ít nhất một mục cho ca test này");
    let mut image_keys: Vec<&str> = image_value
        .as_object()
        .expect("một mục images phải serialize thành object")
        .keys()
        .map(String::as_str)
        .collect();
    image_keys.sort_unstable();
    assert_eq!(
        image_keys,
        vec!["after_segment_id", "alt_text", "asset_id", "caption_text", "file_name", "source_url"],
        "khoá trên dây của ReadingImage là snake_case. Nhận được: {image_keys:?}."
    );

    let paragraph_value = chapter_value
        .get("paragraphs")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .expect("paragraphs phải mang ít nhất một mục cho ca test này");
    let mut paragraph_keys: Vec<&str> = paragraph_value
        .as_object()
        .expect("một mục paragraphs phải serialize thành object")
        .keys()
        .map(String::as_str)
        .collect();
    paragraph_keys.sort_unstable();
    assert_eq!(
        paragraph_keys,
        vec!["segments"],
        "khoá trên dây của ReadingParagraph là snake_case. Nhận được: {paragraph_keys:?}."
    );

    let segment_value = paragraph_value
        .get("segments")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .expect("segments phải mang ít nhất một mục cho ca test này");
    let mut segment_keys: Vec<&str> = segment_value
        .as_object()
        .expect("một mục segments phải serialize thành object")
        .keys()
        .map(String::as_str)
        .collect();
    segment_keys.sort_unstable();
    assert_eq!(
        segment_keys,
        vec!["id", "is_confirmed", "is_marked", "source_text", "target_text"],
        "khoá trên dây của ReadingSegment là snake_case. Nhận được: {segment_keys:?}. Nghi phạm \
         số một: `#[serde(rename_all = \"camelCase\")]` đặt nhầm lên struct này."
    );

    let frontier_value = value.get("frontier").expect("frontier phải có mặt trên dây");
    let mut frontier_keys: Vec<&str> = frontier_value
        .as_object()
        .expect("frontier phải serialize thành object")
        .keys()
        .map(String::as_str)
        .collect();
    frontier_keys.sort_unstable();
    assert_eq!(
        frontier_keys,
        vec!["chapter", "kind"],
        "khoá trên dây của ReadingFrontier là snake_case. Nhận được: {frontier_keys:?}."
    );
    assert_eq!(
        frontier_value.get("kind").and_then(|v| v.as_str()),
        Some("next-not-done"),
        "ReadingFrontierKind::NextNotDone phải serialize thành chuỗi \"next-not-done\" NGUYÊN \
         VĂN — webview khớp đúng chuỗi này, không một biến thể nào khác."
    );

    let frontier_chapter_value =
        frontier_value.get("chapter").expect("chapter phải có mặt khi kind == NextNotDone");
    let mut frontier_chapter_keys: Vec<&str> = frontier_chapter_value
        .as_object()
        .expect("frontier.chapter phải serialize thành object")
        .keys()
        .map(String::as_str)
        .collect();
    frontier_chapter_keys.sort_unstable();
    assert_eq!(
        frontier_chapter_keys,
        vec!["chapter_id", "chapter_ord", "chapter_title", "status"],
        "khoá trên dây của ReadingFrontierChapter là snake_case. Nhận được: \
         {frontier_chapter_keys:?}."
    );

    // Biến thể còn lại của `ReadingFrontierKind` — chuỗi NGUYÊN VĂN cũng phải đóng băng,
    // không chỉ biến thể đã kiểm ở trên.
    let end_of_work = auratranslate_lib::commands::segment::ReadingFrontier {
        kind: auratranslate_lib::commands::segment::ReadingFrontierKind::EndOfWork,
        chapter: None,
    };
    let end_value = serde_json::to_value(&end_of_work).expect("ReadingFrontier phải serialize được");
    assert_eq!(
        end_value.get("kind").and_then(|v| v.as_str()),
        Some("end-of-work"),
        "ReadingFrontierKind::EndOfWork phải serialize thành chuỗi \"end-of-work\" NGUYÊN VĂN."
    );
    assert_eq!(
        end_value.get("chapter"),
        Some(&serde_json::Value::Null),
        "chapter phải là null khi kind == EndOfWork."
    );
}

/// **SỬA TẠI CHỖ Story 5.12** (trước: `the_read_reading_chapter_wire_is_registered`, Story
/// 5.11). `read_reading_run` phải CÓ MẶT trong `generate_handler![…]`, và tên CŨ
/// `read_reading_chapter` không còn — cùng lý lẽ và cùng khuôn
/// [`the_library_search_wire_is_registered_and_keeps_its_parameter_names`] ở trên.
/// ⚠️ Không tham số nào đi trên dây (cùng khuôn `read_open_chapter_segments`), nên không
/// có vế "giữ nguyên tên tham số" để nghiệm thu ở đây.
#[test]
fn the_read_reading_run_wire_is_registered() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));

    assert!(
        lib_src.contains("crate::commands::segment::wire::read_reading_run"),
        "`crate::commands::segment::wire::read_reading_run` phai co mat trong \
         generate_handler! cua lib.rs. Thieu no thi invoke() tra \"command not found\" va \
         KHONG cong nao do."
    );
    assert!(
        !lib_src.contains("crate::commands::segment::wire::read_reading_chapter"),
        "ten CU `read_reading_chapter` khong duoc con lai trong generate_handler! -- hai lenh \
         doc cung mot be mat Che do doc la hai nguon su that."
    );

    for wire in ["mark_reading_segment", "list_reading_marks"] {
        assert!(
            lib_src.contains(&format!("crate::commands::segment::wire::{wire}")),
            "wire marker `{wire}` phai co mat trong generate_handler!"
        );
    }
    let segment_rs =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("commands").join("segment.rs");
    let segment_src = fs::read_to_string(&segment_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", segment_rs.display()));
    assert!(
        segment_src.contains("pub fn mark_reading_segment(\n        app: tauri::AppHandle,\n        segment_id: i64,"),
        "wire mark_reading_segment phai giu tham so `segment_id` (webview gui `segmentId`)"
    );
}

#[test]
fn reading_mark_wire_fields_stay_snake_case() {
    let mark = auratranslate_lib::commands::segment::ReadingMark {
        segment_id: 1,
        navigation_segment_id: 7,
        chapter_id: 2,
        chapter_ord: 3,
        chapter_title: Some("Chương Ba".to_owned()),
        source_text: "原文".to_owned(),
        target_text: "Bản dịch".to_owned(),
        is_retired: true,
        marked_at: "2026-08-31T00:00:00.000Z".to_owned(),
    };
    let value = serde_json::to_value(mark).expect("ReadingMark serialize");
    let mut keys: Vec<&str> = value.as_object().expect("object").keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "chapter_id",
            "chapter_ord",
            "chapter_title",
            "is_retired",
            "marked_at",
            "navigation_segment_id",
            "segment_id",
            "source_text",
            "target_text",
        ]
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.7b, Phase 4 (coordinator review 2026-09-16) — nêm cho vỏ của khuôn bốn bước
// (`reindex_library(&app, &root)`) ở `wire.rs`: đối chứng đỏ ① (§Verification) đo được 0 ca
// đỏ khi gỡ dòng đó ở nhánh APPEND — không test nào gọi được `wire::confirm_import_with_
// encoding` (không `tauri::test`/`MockRuntime` harness trong kho), nên một lần xoá dòng gọi
// TẠI CHÍNH VỎ không bị bắt bởi bất kỳ ca nào. Cổng này canh SỰ CÓ MẶT của nguồn (không canh
// HÀNH VI lúc chạy), cùng khuôn `ipc_contract.rs::the_three_bilingual_import_wires_are_
// registered_read_cleanup_rules_and_rebuild_never_reads_the_file` (containment trên nguồn) +
// `webimport_boundary.rs::code_lines`/`text_before_first_cfg_test_line` (lọc chú thích).
// ═════════════════════════════════════════════════════════════════════════════════

/// Dòng gọi CHÍNH XÁC của bước 4 — cả năm chỗ trong `wire.rs` (`:499`, `:545`, `:837`,
/// `:905`, `:1102`) mang ĐÚNG cùng một chuỗi này (đo lại 2026-09-16, sau Phase 1-4).
const REINDEX_LIBRARY_CALL_LINE: &str = "reindex_library(&app, &root);";

/// Dòng KHÔNG phải chú thích — cùng khuôn `webimport_boundary.rs::code_lines`.
fn code_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines().map(str::trim).filter(|code| {
        !code.is_empty()
            && !code.starts_with("//")
            && !code.starts_with("/*")
            && !code.starts_with("* ")
            && !code.starts_with("*/")
    })
}

/// Đếm số dòng MÃ THẬT (đã lọc chú thích) trùng NGUYÊN VĂN [`REINDEX_LIBRARY_CALL_LINE`] — một
/// dòng bị COMMENT ra (`// reindex_library(&app, &root);`) không được đếm là còn mặt, dù
/// chuỗi vẫn CÓ MẶT trong văn bản — đúng cái bẫy "gỡ giả" mà `AGENTS.md:68` gọi tên, và một
/// `str::contains`/`grep` trần sẽ bị nó lừa.
fn count_reindex_library_calls(text: &str) -> usize {
    code_lines(text).filter(|&code| code == REINDEX_LIBRARY_CALL_LINE).count()
}

/// Cắt thân MỘT `pub fn` trong `mod wire` — từ đúng chữ ký `pub fn {fn_name}(` tới NGAY TRƯỚC
/// khối `pub fn` kế tiếp (hoặc hết tệp) — cùng khuôn `code_lines_of` (đóng cục bộ trong
/// [`the_three_import_encoding_preview_wires_are_registered_and_keep_their_parameter_names`]
/// ngay trên), tổng quát hoá thành một hàm dùng lại được cho nhiều ca.
fn wire_fn_body<'a>(wire_src: &'a str, fn_name: &str) -> &'a str {
    let signature = format!("pub fn {fn_name}(");
    let start = wire_src
        .find(&signature)
        .unwrap_or_else(|| panic!("khong tim thay vo `{fn_name}` trong `mod wire`"));
    let rest = &wire_src[start..];
    let end = rest[signature.len()..]
        .find("\n    pub fn ")
        .map(|offset| offset + signature.len())
        .unwrap_or(rest.len());
    &rest[..end]
}

fn read_wire_rs() -> String {
    let wire_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("commands")
        .join("project")
        .join("wire.rs");
    fs::read_to_string(&wire_rs).unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", wire_rs.display()))
}

/// **THÊM 2026-09-16 (Story 6.7b, coordinator review)** — cả NĂM chỗ gọi bước 4
/// (`reindex_library(&app, &root)`) trong `mod wire` phải CÒN MẶT: `create_work_from_text`
/// (`:499`), `create_work_from_file` (`:545`), `confirm_import_with_encoding` — HAI chỗ, nhánh
/// Tác phẩm MỚI (`:837`) và nhánh APPEND (`:905`) — và `confirm_bilingual_import` (`:1102`).
/// Xoá (hoặc COMMENT ra) BẤT KỲ một trong năm phải làm ca này đỏ — đối chứng đo 2026-09-16,
/// bảng năm lượt gỡ ở Implementation Notes của spec 6.7b.
///
/// 🔴 **Vì sao neo THEO TỪNG SEAM, không chỉ đếm tổng** — `confirm_import_with_encoding` mang
/// HAI trong năm chỗ gọi; một phép đếm tổng bằng 5 không nói được CHỖ NÀO chết nếu một chỗ mất
/// trong khi một chỗ khác vô tình được thêm ở đâu đó khác trong `mod wire` (tổng vẫn là 5, ca
/// vẫn xanh — đúng bẫy "một khẳng định đúng trên cả hai nhánh không canh nhánh nào",
/// `AGENTS.md:68`). Bổ đôi thân hàm CHUNG (`confirm_import_with_encoding`) tại đúng dòng chú
/// thích neo `"// Đường APPEND (Story 6.7b)"` — ranh giới THẬT giữa nhánh Tác phẩm MỚI và
/// nhánh APPEND, không phải một số dòng đoán chừng. Sàn quần thể (tổng = 5 trên TOÀN VĂN
/// `wire.rs`) chạy CỘNG THÊM, không thay thế, năm phép neo — phòng một chùm khác len lỏi vào
/// `mod wire` mà năm seam neo ở trên không biết tới.
#[test]
fn all_five_reindex_library_call_sites_in_wire_rs_are_present() {
    let wire_src = read_wire_rs();

    let create_from_text = wire_fn_body(&wire_src, "create_work_from_text");
    let create_from_file = wire_fn_body(&wire_src, "create_work_from_file");
    let confirm = wire_fn_body(&wire_src, "confirm_import_with_encoding");
    let bilingual = wire_fn_body(&wire_src, "confirm_bilingual_import");

    const APPEND_BRANCH_ANCHOR: &str = "// Đường APPEND (Story 6.7b)";
    let append_split = confirm.find(APPEND_BRANCH_ANCHOR).unwrap_or_else(|| {
        panic!(
            "khong tim thay neo `{APPEND_BRANCH_ANCHOR}` trong than `confirm_import_with_encoding` \
             -- neo doi khi ranh gioi giua nhanh Tac pham MOI va nhanh APPEND doi vi tri/loi van, \
             sua lai neo nay theo dung ranh gioi that roi chay lai"
        )
    });
    let new_work_branch = &confirm[..append_split];
    let append_branch = &confirm[append_split..];

    for (seam, body, expected) in [
        ("create_work_from_text (:499)", create_from_text, 1),
        ("create_work_from_file (:545)", create_from_file, 1),
        ("confirm_import_with_encoding, nhanh Tac pham MOI (:837)", new_work_branch, 1),
        ("confirm_import_with_encoding, nhanh APPEND (:905)", append_branch, 1),
        ("confirm_bilingual_import (:1102)", bilingual, 1),
    ] {
        let found = count_reindex_library_calls(body);
        assert_eq!(
            found, expected,
            "seam `{seam}` phai mang DUNG {expected} loi goi `{REINDEX_LIBRARY_CALL_LINE}` -- tim \
             thay {found}. Buoc 4 (Indexer::rebuild qua reindex_library) mat o day lam library-\
             index.db noi doi im lang sau lan ghi ke tiep qua seam nay (src-tauri/AGENTS.md:54)."
        );
    }

    let total = count_reindex_library_calls(&wire_src);
    assert_eq!(
        total, 5,
        "tong so loi goi `{REINDEX_LIBRARY_CALL_LINE}` trong toan bo wire.rs phai la DUNG 5 -- tim \
         thay {total}. Mot con so khac 5 la mot chum da xuat hien/bien mat o dau do trong `mod \
         wire` ma nam seam neo o tren khong biet toi."
    );
}

/// Đối chứng dương/âm cho [`count_reindex_library_calls`] — khuôn `webimport_boundary.rs::
/// the_content_parsing_token_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code`.
/// Chứng minh vị từ THẬT SỰ bắt được một dòng bị COMMENT ra (chuỗi vẫn CÓ MẶT trong văn bản,
/// nhưng không còn là MÃ) — cái bẫy `AGENTS.md:68` gọi tên nguyên văn, và một `str::contains`
/// trần sẽ bị nó lừa.
#[test]
fn count_reindex_library_calls_is_not_fooled_by_a_commented_out_line() {
    let clean = "fn confirm() {\n    reindex_library(&app, &root);\n    Ok(())\n}\n";
    assert_eq!(count_reindex_library_calls(clean), 1, "ca DUONG -- mot dong MA THAT phai duoc dem");

    let commented = "fn confirm() {\n    // reindex_library(&app, &root);\n    Ok(())\n}\n";
    assert_eq!(
        count_reindex_library_calls(commented),
        0,
        "ca AM (bay 'go gia') -- mot dong bi COMMENT RA khong duoc dem la con mat, du chuoi van \
         CO MAT trong van ban -- day chinh la loai loi ma AGENTS.md:68 goi ten"
    );

    let doc_commented = "fn confirm() {\n    /// reindex_library(&app, &root);\n    Ok(())\n}\n";
    assert_eq!(
        count_reindex_library_calls(doc_commented),
        0,
        "ca AM thu hai -- chu thich TAI LIEU (`///`) cung khong duoc dem"
    );

    let unrelated = "fn other() {\n    do_something_else();\n    reindex_library(&app, &different_root);\n}\n";
    assert_eq!(
        count_reindex_library_calls(unrelated),
        0,
        "ca AM thu ba -- mot loi goi GAN GIONG (tham so khac) khong duoc dem la khop, danh cho mot \
         lan sua tay lam sai tham so ma khong xoa han dong goi"
    );
}

/// Đối chứng dương/âm cho [`wire_fn_body`] — chứng minh hàm cắt ĐÚNG khối, không lấn sang hàm
/// kế tiếp, cùng khuôn `fn_param_list_would_actually_bind_to_the_right_function_block`.
#[test]
fn wire_fn_body_stops_before_the_next_pub_fn_and_does_not_bleed_into_it() {
    let src = "    pub fn foo(a: i32) {\n        reindex_library(&app, &root);\n    }\n\n    pub fn bar(b: i32) {\n        reindex_library(&app, &root);\n    }\n";

    let foo = wire_fn_body(src, "foo");
    assert_eq!(count_reindex_library_calls(foo), 1, "than `foo` phai mang DUNG 1 loi goi cua CHINH no");
    assert!(!foo.contains("fn bar"), "than `foo` khong duoc lan sang `bar`");

    let bar = wire_fn_body(src, "bar");
    assert_eq!(count_reindex_library_calls(bar), 1, "than `bar` phai mang DUNG 1 loi goi cua CHINH no");
}

/// **THÊM Story 4.3.** `ai_config_save_key`/`ai_config_delete_key` phải CÓ MẶT trong
/// `generate_handler![…]` — cùng lý lẽ và cùng khuôn
/// [`the_library_search_wire_is_registered_and_keeps_its_parameter_names`]. `aiconfig_contract.rs`
/// gọi thẳng hai hàm thuần (`commands::aiconfig::ai_config_save_key`/`ai_config_delete_key`),
/// không đi qua `wire::`, và mọi test frontend mock `src/config/aiconfig` ở biên module — nên
/// xoá hai dòng đăng ký này khỏi `lib.rs` để `cargo test --locked` VÀ `npx vitest run` xanh cả
/// hai trong khi Lưu/Xoá khoá vỡ trên một bản dựng thật. Phạm vi CHỈ hai lệnh Story 4.3 thêm —
/// ba lệnh `ai_config_*` của Story 4.2 (`ai_config_get`/`ai_config_save_field`/
/// `ai_config_clear_override`) có cùng lỗ hổng từ trước story này và không thuộc phạm vi sửa ở
/// đây.
#[test]
fn the_aiconfig_key_wires_are_registered() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));

    for wire in [
        "crate::commands::aiconfig::wire::ai_config_save_key",
        "crate::commands::aiconfig::wire::ai_config_delete_key",
    ] {
        assert!(
            lib_src.contains(wire),
            "`{wire}` phai co mat trong generate_handler! cua lib.rs. Thieu no thi invoke() tra \
             \"command not found\" va Luu/Xoa khoa API vo tren mot ban dung that."
        );
    }
}

/// **THÊM Story 4.4, Phase 2; MỞ RỘNG Story 4.5.** Chín vỏ `commands::promptset::wire::*`
/// phải CÓ MẶT trong `generate_handler![…]` — cùng lý lẽ và cùng khuôn
/// [`the_aiconfig_key_wires_are_registered`] ngay trên: `ipc_contract.rs:1824` (doc-comment
/// của test đó) tự ghi rằng cổng này là một danh sách gõ tay THEO TỪNG DOMAIN, không phải
/// một phép quét chung — một domain mới không có gì canh nó trừ khi một sibling được thêm
/// vào. Năm vỏ Phase 2 gọi thẳng năm hàm thuần qua `prompt_set_contract.rs`; bốn vỏ Story
/// 4.5 (xuất/mở-xem-trước/xác nhận/huỷ lượt nhập) gọi thẳng qua
/// `prompt_set_exchange_contract.rs` — cả hai không đi qua `wire::`, nên xoá một dòng đăng ký
/// ở đây khỏi `lib.rs` để `cargo test --locked` VẪN xanh trong khi nút bấm tương ứng vỡ trên
/// một bản dựng thật — đúng lỗ mà spec 4.4's AC2 (nhắc lại nguyên văn ở AC2 spec 4.5) đặt tên
/// ("Given the whole repository ... when `generate_handler!` loses any one prompt-set line,
/// then a Rust case goes red").
#[test]
fn the_prompt_set_wires_are_registered() {
    let lib_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let lib_src = fs::read_to_string(&lib_rs)
        .unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", lib_rs.display()));

    for wire in [
        "crate::commands::promptset::wire::prompt_set_list",
        "crate::commands::promptset::wire::prompt_set_create",
        "crate::commands::promptset::wire::prompt_set_rename",
        "crate::commands::promptset::wire::prompt_set_update_body",
        "crate::commands::promptset::wire::prompt_set_delete",
        // Story 4.5 -- xuat/nhap mot bo prompt qua tep `.prompt.md` (FR79, NFR9, AD-48).
        "crate::commands::promptset::wire::prompt_set_export",
        "crate::commands::promptset::wire::prompt_set_open_import_preview",
        "crate::commands::promptset::wire::prompt_set_confirm_import",
        "crate::commands::promptset::wire::prompt_set_cancel_import",
    ] {
        assert!(
            lib_src.contains(wire),
            "`{wire}` phai co mat trong generate_handler! cua lib.rs. Thieu no thi invoke() tra \
             \"command not found\" va man Thu vien Prompt vo tren mot ban dung that."
        );
    }
}
