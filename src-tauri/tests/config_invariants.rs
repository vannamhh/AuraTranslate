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

/// Tách CSP thành `chỉ thị → tập nguồn`.
///
/// ⚠️ Đừng quay lại kiểu so chuỗi con trên cả chuỗi CSP. Bản trước làm vậy và có
/// **bốn** lối lách: `http://asset.localhost.evil.com` sống sót phép `replace` dạng
/// tiền tố; origin không scheme (`cdn.example.com`) không chứa `//` nên lọt; `data:`
/// bị ĐẾM chứ không bị ĐỊNH VỊ nên chuyển sang `script-src` vẫn xanh; và
/// `'unsafe-inline'` trong `script-src` không bị cấm ở đâu cả.
fn csp_directives(csp: &str) -> std::collections::BTreeMap<String, Vec<String>> {
    csp.split(';')
        .filter_map(|part| {
            let mut it = part.split_whitespace();
            let name = it.next()?.to_owned();
            Some((name, it.map(str::to_owned).collect()))
        })
        .collect()
}

/// Nguồn cục bộ được phép, đã cân nhắc từng cái. Mọi thứ ngoài danh sách này là
/// origin từ xa cho tới khi có người chứng minh ngược lại.
///
/// - `'self'` — chính bản cài.
/// - `asset:` / `http://asset.localhost` — asset protocol, tài nguyên đã nằm trong
///   bản cài (`http://asset.localhost` là dạng của Windows; bỏ nó thì macOS vẫn chạy
///   và Windows hỏng im lặng).
/// - `ipc:` / `http://ipc.localhost` — kênh IPC của chính Tauri. Thiếu chúng trong
///   `connect-src` thì `fetch` IPC bị CSP chặn và Tauri âm thầm tụt xuống
///   `postMessage` — chỉ có một `console.warn`. Xem SECURITY-NOTES.md.
/// - `data:` — CHỈ ở `img-src`, cưỡng chế riêng bên dưới.
const ALLOWED_LOCAL_SOURCES: &[&str] = &[
    "'self'",
    "asset:",
    "http://asset.localhost",
    "ipc:",
    "http://ipc.localhost",
];

/// Test này tồn tại để chứng minh `tests/` `use` được mã sản phẩm — lý do Task 1 bắt
/// bố cục `lib.rs` + `main.rs`.
///
/// ⚠️ Phép kiểm thật ở đây là **khâu link**, không phải một `assert!`: nếu bố cục crate
/// bị phá thì tệp này không biên dịch được. Bản trước có `assert!(entry as usize != 0)`
/// — con trỏ hàm trong Rust an toàn không bao giờ null, nên assertion đó không thể đỏ
/// và chỉ tạo ảo giác có thêm một phép kiểm. Đã bỏ.
#[test]
fn product_crate_is_linkable_from_tests() {
    let _entry: fn() = auratranslate_lib::run;
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
    let directives = csp_directives(&csp);

    // Danh sách CHO PHÉP, không phải danh sách CẤM. Một danh sách cấm chỉ chặn được
    // những hình dạng ai đó đã nghĩ ra; `cdn.example.com` là origin từ xa hợp lệ theo
    // spec CSP và không chứa `//`, nên nó lọt qua mọi danh sách cấm dựa trên chuỗi.
    for (directive, sources) in &directives {
        for src in sources {
            if directive == "img-src" && src == "data:" {
                continue; // hợp lệ, và chỉ ở đây — xem test riêng bên dưới
            }
            assert!(
                ALLOWED_LOCAL_SOURCES.contains(&src.as_str()),
                "`{directive}` chứa nguồn không nằm trong danh sách cục bộ cho phép: `{src}`.\n\
                 Nếu đây thật sự là nguồn cục bộ mới thì thêm vào ALLOWED_LOCAL_SOURCES kèm lý do; \
                 mọi origin TỪ XA đều bị AD-15 cấm (không CDN, không font ngoài, không ảnh ngoài).\n\
                 CSP: {csp}"
            );
        }
    }

    assert!(
        csp.starts_with("default-src 'self'"),
        "CSP phải mở đầu bằng `default-src 'self'`: {csp}"
    );

    // `unsafe-eval` và `unsafe-inline` đều là nới, và `script-src` là mục tiêu nguy
    // hiểm hơn `style-src` hẳn một bậc. Bản trước chỉ cấm `unsafe-eval`, nên
    // `script-src 'self' 'unsafe-inline'` đi qua toàn bộ suite mà không ai biết.
    for unsafe_token in ["'unsafe-eval'", "'unsafe-inline'"] {
        assert!(
            !csp.contains(unsafe_token),
            "`{unsafe_token}` không được phép ở bất kỳ chỉ thị nào. \
             Story 1.2 đã hạ `style-src` xuống `'self'` và kiểm chứng trên bản release — \
             mở lại phải là quyết định có ghi lý do: {csp}"
        );
    }
}

#[test]
fn csp_scheme_sources_are_pinned_to_the_directive_that_needs_them() {
    // Comment thôi không cưỡng chế được gì: bản trước ĐẾM số lần `data:` xuất hiện
    // rồi khẳng định nó "chỉ ở `img-src`". Chuyển `data:` sang `script-src` giữ nguyên
    // số đếm là 1 → xanh, trong khi vừa mở đường thực thi mã tuỳ ý từ URI `data:`.
    let conf = read_json("tauri.conf.json");
    let csp = conf["app"]["security"]["csp"].as_str().unwrap();
    let directives = csp_directives(csp);

    for (directive, sources) in &directives {
        if directive != "img-src" {
            assert!(
                !sources.iter().any(|s| s == "data:"),
                "`data:` chỉ được phép ở `img-src`, thấy ở `{directive}`: {csp}"
            );
        }
        assert!(
            !sources.iter().any(|s| s.contains('*')),
            "`{directive}` chứa wildcard: {csp}"
        );
    }

    // `connect-src` phải khai tường minh và phải chứa kênh IPC. Không khai thì nó rơi
    // về `default-src 'self'`, và `fetch` IPC của Tauri bị CSP chặn — Tauri tụt xuống
    // `postMessage` với đúng MỘT dòng `console.warn`, kể cả đường dữ liệu của Channel
    // (AD-22). Hỏng im lặng, và chỉ lộ ra khi Story 4.x đo throughput streaming.
    let connect = directives
        .get("connect-src")
        .unwrap_or_else(|| panic!("CSP phải khai `connect-src` tường minh: {csp}"));
    for needed in ["ipc:", "http://ipc.localhost"] {
        assert!(
            connect.iter().any(|s| s == needed),
            "`connect-src` thiếu `{needed}` — IPC sẽ âm thầm tụt xuống `postMessage`: {csp}"
        );
    }
}

#[test]
fn no_dev_csp_and_no_platform_config_overrides() {
    // Bộ test này đọc `tauri.conf.json`. Tauri merge `tauri.<platform>.conf.json` tự
    // động, và `devCsp` ghi đè `csp` ở chế độ dev — mà Kiểm 3 chạy ở chính chế độ dev.
    // Không chốt chúng thì `csp` và `scope` THẬT lúc build/dev có thể khác hẳn thứ
    // các test trên đang khẳng định, và mọi khẳng định ở đây thành vô nghĩa.
    let conf = read_json("tauri.conf.json");
    assert!(
        conf["app"]["security"]["devCsp"].is_null(),
        "`devCsp` ghi đè `csp` ở chế độ dev — nơi Kiểm 3 chạy. Thêm nó là vô hiệu hoá \
         mọi test CSP ở tệp này mà không test nào đỏ."
    );

    for platform in ["macos", "windows", "linux", "android", "ios"] {
        let overlay = manifest_dir().join(format!("tauri.{platform}.conf.json"));
        assert!(
            !overlay.exists(),
            "`{}` tồn tại và được Tauri merge tự động — bộ test này chỉ đọc \
             `tauri.conf.json` nên sẽ khẳng định sai. Mở rộng test trước khi thêm tệp đó.",
            overlay.display()
        );
    }
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
        vec![
            "core:path:default",
            "core:event:default",
            "core:resources:default"
        ],
        "Tối thiểu THẬT, ba tập: path (resolveResource) · event (emit/Channel, AD-22) · \
         resources (dọn resource table).\n\
         ⛔ Đừng quay lại `core:default`: nó là một BUNDLE kéo theo cả \
         `core:window:default`, `core:webview:default`, `core:menu:default`, \
         `core:tray:default`, `core:app:default` — dự án này dựng cả một script để cấm \
         `tauri-plugin-fs` vì bề mặt IPC, mở sẵn nhóm kia là tự mâu thuẫn.\n\
         Mọi quyền `<plugin>:…` ở đây là một bề mặt IPC mới — phải là một AD mới trước đã"
    );
}

#[test]
fn capabilities_directory_holds_exactly_the_one_reviewed_file() {
    // Tauri nạp MỌI tệp trong `capabilities/`. Test trên chỉ đọc `main.json`, nên thêm
    // một `extra.json` với `"permissions": ["fs:default"]` là cấp một bề mặt IPC mới
    // mà không test nào đỏ — đúng loại hỏng im lặng mà cả tệp này tồn tại để chặn.
    let dir = manifest_dir().join("capabilities");
    let mut files: Vec<String> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".json"))
        .collect();
    files.sort();

    assert_eq!(
        files,
        vec!["main.json"],
        "Mọi tệp trong `capabilities/` đều được Tauri nạp. Thêm tệp thứ hai là mở một \
         bề mặt IPC mới — cập nhật test này CÙNG LÚC, đừng chỉ thêm tệp"
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
        "productName là tên hiển thị của bundle (`.app`, `.msi`, tiêu đề cửa sổ).\n\
         ⚠️ Nó KHÔNG quyết định tên tiến trình: tên tiến trình lấy từ `package.name` của \
         Cargo, tức `auratranslate` chữ thường (đã đo 2026-08-03 — binary thật là \
         `target/release/auratranslate`). Công thức quan sát mạng của AC5 phải là \
         `pgrep -x auratranslate`, KHÔNG phải `pgrep -n AuraTranslate` (trả rỗng)"
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
