//! Bất biến cấu hình của Story 1.2 — AC3 (phạm vi filesystem) và AC4 (CSP).
//!
//! Vì sao là test chứ không phải chú thích: `tauri.conf.json` là JSON, không mang được
//! chú thích, và cả hai AC này **hỏng im lặng** — `"csp": null` vẫn build thành công,
//! thêm một mục vào `assetProtocol.scope` vẫn build thành công. Lời văn nằm ở
//! `src-tauri/SECURITY-NOTES.md`; chỗ này là phần máy cưỡng chế.

use std::fs;
use std::path::PathBuf;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(rel: &str) -> serde_json::Value {
    let path = manifest_dir().join(rel);
    let raw = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// Test này tồn tại để chứng minh `tests/` `use` được mã sản phẩm — lý do Task 1 bắt
/// bố cục `lib.rs` + `main.rs`. Lấy con trỏ hàm là đủ, không chạy `run()`.
#[test]
fn product_crate_is_linkable_from_tests() {
    let entry: fn() = auratranslate_lib::run;
    assert!(!(entry as usize == 0));
}

// ── AC4 — CSP chặn mọi origin từ xa ─────────────────────────────────────────────

#[test]
fn csp_is_declared_explicitly_not_null() {
    let conf = read_json("tauri.conf.json");
    let csp = &conf["app"]["security"]["csp"];
    assert!(
        csp.is_string(),
        "`csp` phải là chuỗi tường minh. `null` là TẮT CSP, không phải 'dùng mặc định' \
         — đọc src-tauri/SECURITY-NOTES.md"
    );
    assert!(
        !csp.as_str().unwrap().trim().is_empty(),
        "`csp` rỗng thì cũng như tắt"
    );
}

#[test]
fn csp_allows_no_remote_origin() {
    let conf = read_json("tauri.conf.json");
    let csp = conf["app"]["security"]["csp"].as_str().unwrap().to_string();

    // Hai token cục bộ hợp lệ, gỡ ra trước khi soi phần còn lại. Chúng KHÔNG phải
    // origin từ xa: tài nguyên đã nằm trong bản cài (AD-15 cấm CDN / font ngoài /
    // ảnh ngoài). `http://asset.localhost` là dạng của Windows — bỏ nó thì macOS vẫn
    // chạy và Windows hỏng im lặng.
    let stripped = csp.replace("http://asset.localhost", "").replace("asset:", "");

    for needle in ["http://", "https://", "//"] {
        assert!(
            !stripped.contains(needle),
            "CSP chứa origin từ xa qua `{needle}`: {csp}"
        );
    }
    for banned in ["*", "data:"] {
        // `data:` chỉ được phép ở `img-src`; bất kỳ chỗ nào khác là nới.
        let hits = stripped.matches(banned).count();
        if banned == "data:" {
            assert!(hits <= 1, "`data:` chỉ được xuất hiện một lần, ở `img-src`: {csp}");
        } else {
            assert_eq!(hits, 0, "CSP chứa wildcard `{banned}`: {csp}");
        }
    }
    assert!(
        csp.starts_with("default-src 'self'"),
        "CSP phải mở đầu bằng `default-src 'self'`: {csp}"
    );
    assert!(
        !csp.contains("unsafe-eval"),
        "`unsafe-eval` không bao giờ được phép: {csp}"
    );
}

#[test]
fn csp_style_src_stays_at_self() {
    // Story 1.2 hạ `style-src` từ `'self' 'unsafe-inline'` (app thăm dò của Story 1.1)
    // xuống `'self'` và kiểm chứng trên bản build release. Mở lại `'unsafe-inline'`
    // phải là quyết định có ghi lý do, không phải một lần sửa cho qua.
    let conf = read_json("tauri.conf.json");
    let csp = conf["app"]["security"]["csp"].as_str().unwrap();
    assert!(
        csp.contains("style-src 'self';") || csp.ends_with("style-src 'self'"),
        "`style-src` đã bị nới khỏi `'self'` — đọc src-tauri/SECURITY-NOTES.md \
         trước khi sửa test này: {csp}"
    );
}

// ── AC3 — phạm vi filesystem tĩnh ───────────────────────────────────────────────

#[test]
fn asset_protocol_scope_has_exactly_the_two_readonly_resource_areas() {
    let conf = read_json("tauri.conf.json");
    let ap = &conf["app"]["security"]["assetProtocol"];

    assert_eq!(ap["enable"], serde_json::json!(true), "`assetProtocol` phải bật");

    let scope: Vec<&str> = ap["scope"]
        .as_array()
        .expect("`assetProtocol.scope` phải là mảng")
        .iter()
        .map(|v| v.as_str().expect("mỗi mục scope là chuỗi"))
        .collect();

    assert_eq!(
        scope,
        vec!["$RESOURCE/dict/**", "$RESOURCE/fonts/**"],
        "AD-23: đúng hai mục, không hơn, không đổi thứ tự"
    );
}

#[test]
fn asset_protocol_scope_never_contains_appdata() {
    // Nửa `$APPDATA/**` của AD-23 là phạm vi của mã Rust và nghiệm thu bằng VẮNG MẶT
    // bề mặt IPC. Đưa nó vào đây là phơi `global.db` / `library-index.db` ra webview.
    let conf = read_json("tauri.conf.json");
    let ap = conf["app"]["security"]["assetProtocol"].to_string();
    assert!(
        !ap.contains("$APPDATA"),
        "`$APPDATA` không bao giờ được vào `assetProtocol.scope` (AD-1, AD-11, AD-23)"
    );
}

#[test]
fn main_capability_grants_the_minimum_and_no_plugin_permission() {
    let cap = read_json("capabilities/main.json");

    let windows: Vec<&str> = cap["windows"]
        .as_array()
        .expect("`windows` phải là mảng")
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(windows, vec!["main"], "AD-24: một cửa sổ OS, label `main`");

    let perms: Vec<&str> = cap["permissions"]
        .as_array()
        .expect("`permissions` phải là mảng")
        .iter()
        .map(|v| v.as_str().expect("quyền dạng chuỗi, không dạng object"))
        .collect();
    assert_eq!(
        perms,
        vec!["core:default"],
        "Tối thiểu: `core:default` cần cho resolveResource/convertFileSrc, và không gì khác. \
         Mọi quyền `<plugin>:…` ở đây là một bề mặt IPC mới — phải là một AD mới trước đã"
    );
}

// ── AC1 / AC6 — những thứ sai một lần là sai ở 300 chỗ ──────────────────────────

#[test]
fn identifier_and_product_name_are_the_committed_ones() {
    let conf = read_json("tauri.conf.json");

    assert_eq!(
        conf["identifier"], "com.auratranslate.desktop",
        "identifier mặc định kiểu `com.tauri.dev` bị Tauri từ chối build"
    );
    assert!(
        !conf["identifier"].as_str().unwrap().ends_with(".app"),
        "identifier kết thúc bằng `.app` đụng phần mở rộng bundle của macOS"
    );
    assert_eq!(
        conf["productName"], "AuraTranslate",
        "productName quyết định tên tiến trình — công thức quan sát mạng của AC5 \
         `pgrep` theo đúng chuỗi này"
    );
}

#[test]
fn build_block_points_at_the_real_frontend() {
    // Bốn trường này sai một cái là `tauri dev` treo hoặc `tauri build` đóng gói một
    // thư mục rỗng — và bản rỗng vẫn build THÀNH CÔNG. Hỏng im lặng.
    let conf = read_json("tauri.conf.json");
    let build = &conf["build"];
    assert_eq!(build["beforeDevCommand"], "npm run dev");
    assert_eq!(build["beforeBuildCommand"], "npm run build");
    assert_eq!(build["devUrl"], "http://localhost:1420");
    assert_eq!(build["frontendDist"], "../dist");
}
