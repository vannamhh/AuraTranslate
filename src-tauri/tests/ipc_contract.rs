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

/// **THÊM Story 5.3.** Đóng băng tên trường của HAI struct wire mới
/// (`commands::library::{RescanReport, OrphanEntry}`) — cùng khuôn phần `ok_shape`/`ok_keys`
/// của [`ipc_error_wire_shape`] ở trên: một trường mới đi qua IPC mà không ai đối chiếu là
/// đúng thứ ca này tồn tại để chặn.
#[test]
fn library_wire_structs_keep_snake_case_field_names() {
    let report = auratranslate_lib::commands::library::RescanReport {
        root: "/tmp/library".to_owned(),
        // P1 (vòng rà bốn lớp 2026-08-27) -- trường mới, đóng băng cùng lượt.
        root_missing: false,
        indexed: 1,
        conflicts: 0,
        skipped: 0,
        orphans: vec![auratranslate_lib::commands::library::OrphanEntry {
            work_id: "id-1".to_owned(),
            name: "Tên".to_owned(),
            atproj_path: "/tmp/x.atproj".to_owned(),
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
        vec!["conflicts", "indexed", "orphans", "root", "root_missing", "skipped"],
        "khoá trên dây của RescanReport là snake_case. Nhận được: {top_keys:?}."
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
}
