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
///
/// 🔴 **Lối lách thứ NĂM, do chính bản viết lại tạo ra — đã sửa ở đây.** Bản trước
/// `.collect()` thẳng vào `BTreeMap`, và `FromIterator` giữ mục **CUỐI** khi khoá trùng.
/// Trình duyệt làm NGƯỢC LẠI: trong **một** policy, chỉ thị xuất hiện lần đầu là thứ
/// được thi hành, mọi lần lặp sau bị **bỏ qua** (CSP Level 3 §"Should directive be
/// executed"). Nên
/// `"… font-src https://cdn.evil.com; font-src 'self' asset: …"` đi qua sạch sẽ toàn bộ
/// tệp test này — map chỉ thấy bản lành — trong khi webview thật nạp font từ CDN.
///
/// Nay hàm **giữ bản ĐẦU**, đúng ngữ nghĩa trình duyệt, và
/// `csp_declares_each_directive_exactly_once` cấm hẳn việc khai trùng để không ai phải
/// nhớ luật này lần nữa.
fn csp_directives(csp: &str) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut map = std::collections::BTreeMap::new();
    for part in csp.split(';') {
        let mut it = part.split_whitespace();
        let Some(name) = it.next() else { continue };
        map.entry(name.to_owned())
            .or_insert_with(|| it.map(str::to_owned).collect::<Vec<String>>());
    }
    map
}

/// Đếm số lần MỖI tên chỉ thị xuất hiện trong chuỗi CSP thô.
fn csp_directive_counts(csp: &str) -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for part in csp.split(';') {
        if let Some(name) = part.split_whitespace().next() {
            *counts.entry(name.to_owned()).or_insert(0usize) += 1;
        }
    }
    counts
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
/// - `'none'` — tập nguồn RỖNG. Chặt hơn `'self'`, dùng cho bốn chỉ thị siết ở
///   `csp_declares_the_directives_that_do_not_inherit_default_src`.
const ALLOWED_LOCAL_SOURCES: &[&str] = &[
    "'self'",
    "'none'",
    "asset:",
    "http://asset.localhost",
    "ipc:",
    "http://ipc.localhost",
];

/// Bốn chỉ thị **KHÔNG kế thừa `default-src`** theo spec CSP — vắng mặt nghĩa là
/// **không giới hạn**, không phải "rơi về `'self'`". Đây là điểm mù của
/// `csp_allows_no_remote_origin`: nó chỉ duyệt các chỉ thị ĐANG CÓ MẶT, nên sự vắng
/// mặt vô hình với toàn bộ suite.
///
/// `base-uri` đáng lo nhất: AD-16 tồn tại vì nội dung nhập từ web là không tin được,
/// mà một điểm chèn DOM đủ để ghi `<base href="https://…">` và trỏ lại **mọi** đường
/// dẫn tương đối — một đường ra mạng nằm ngoài ba điểm của AD-15. `form-action` là
/// cùng lớp: `<form action="https://…">` không bị `default-src` ngăn.
const DIRECTIVES_THAT_DO_NOT_INHERIT_DEFAULT_SRC: &[&str] =
    &["base-uri", "form-action", "object-src", "frame-ancestors"];

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
fn csp_declares_each_directive_exactly_once() {
    // 🔴 Trình duyệt thi hành lần xuất hiện ĐẦU của một chỉ thị và BỎ QUA mọi lần sau.
    // Một `BTreeMap` thì ngược lại — nó giữ bản cuối. Khoảng chênh đó là một lối lách
    // trọn vẹn: khai `font-src` hai lần, bản đầu mở ra CDN và bản sau viết lành, thì
    // mọi test ở tệp này đọc bản lành còn webview nạp bản mở.
    //
    // `csp_directives` nay giữ bản đầu (đúng ngữ nghĩa trình duyệt). Test này đóng nốt
    // đường còn lại: cấm hẳn việc khai trùng, để không ai phải nhớ luật ưu tiên nữa.
    let conf = read_json("tauri.conf.json");
    let csp = conf["app"]["security"]["csp"].as_str().unwrap();

    let dupes: Vec<String> = csp_directive_counts(csp)
        .into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|(name, n)| format!("`{name}` ×{n}"))
        .collect();

    assert!(
        dupes.is_empty(),
        "CSP khai trùng chỉ thị: {}.\n\
         Trình duyệt thi hành lần xuất hiện ĐẦU và bỏ qua các lần sau — nên bản thứ hai \
         chỉ làm bộ test này đọc sai, không làm webview an toàn hơn. Gộp thành một chỉ \
         thị duy nhất.\n\
         CSP: {csp}",
        dupes.join(", ")
    );
}

#[test]
fn csp_declares_the_directives_that_do_not_inherit_default_src() {
    // Bốn chỉ thị này KHÔNG rơi về `default-src`. Vắng mặt = không giới hạn, và sự vắng
    // mặt là thứ `csp_allows_no_remote_origin` không thể thấy vì nó chỉ duyệt những gì
    // đã được khai. Đây là chỗ bịt điểm mù đó.
    let conf = read_json("tauri.conf.json");
    let csp = conf["app"]["security"]["csp"].as_str().unwrap();
    let directives = csp_directives(csp);

    for needed in DIRECTIVES_THAT_DO_NOT_INHERIT_DEFAULT_SRC {
        let sources = directives.get(*needed).unwrap_or_else(|| {
            panic!(
                "CSP thiếu `{needed}` — chỉ thị này KHÔNG kế thừa `default-src`, nên vắng \
                 mặt nghĩa là KHÔNG GIỚI HẠN, không phải 'rơi về self'.\n\
                 Đọc doc-comment của DIRECTIVES_THAT_DO_NOT_INHERIT_DEFAULT_SRC trước khi \
                 sửa test này.\n\
                 CSP: {csp}"
            )
        });
        assert!(
            !sources.is_empty(),
            "`{needed}` khai rỗng — viết `'none'` tường minh: {csp}"
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

/// 🔴 Story 1.9, Task 10 — Ice chốt 2026-08-04: **gỡ** `$RESOURCE/dict/**` khỏi
/// `assetProtocol.scope`. Webview KHÔNG BAO GIỜ đọc tệp từ điển — AD-1 và AD-11 đặt
/// mọi truy cập dữ liệu ở Rust, và `rusqlite` mở tệp bằng đường dẫn hệ thống, không
/// đi qua asset protocol. Mục scope đó là một QUYỀN THỪA; mâu thuẫn với `connect-src`
/// (`deferred-work.md:56-57`) chỉ là hệ quả của việc nó thừa.
///
/// ⚠️ Tên hàm đã ĐỔI cùng lúc với giá trị — một tên còn nói `..._the_two_...` trong khi
/// scope chỉ còn MỘT mục là để lại một cái tên nói dối, và tên test là thứ lượt rà soát
/// sau đọc TRƯỚC tiên.
///
/// 🔴 Lưới thay thế: sau lượt này, KHÔNG còn dòng nào trong `tauri.conf.json` nhắc tới
/// `dict` cho tới Story 10.1 (đóng gói `.db` vào `bundle.resources` + lưới mới). Xem
/// mục Story 10.1 mới trong `deferred-work.md`.
#[test]
fn asset_protocol_scope_has_exactly_the_one_readonly_resource_area() {
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
        vec!["$RESOURCE/fonts/**"],
        "AD-23 (Story 1.9 Task 10): đúng MỘT mục — webview không bao giờ đọc tệp từ \
         điển, `$RESOURCE/dict/**` là quyền thừa đã gỡ"
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
         Đừng quay lại `core:default`: nó là một BUNDLE kéo theo cả \
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
    //
    // 🔴 Bản trước bịt lỗ đó **chưa kín**, theo đúng hai cách:
    //   1. `.filter(|n| n.ends_with(".json"))` — Tauri nhận **ba** phần mở rộng.
    //      `tauri-utils/src/acl/build.rs`: `CAPABILITY_FILE_EXTENSIONS = ["json","json5","toml"]`.
    //      Nên `capabilities/extra.toml` được nạp thật mà test vẫn xanh.
    //   2. `fs::read_dir` **không đệ quy**, còn Tauri nạp bằng glob `"{capabilities}/**/*"`.
    //      Nên `capabilities/sub/extra.json` cũng lọt.
    // Nay: liệt kê **mọi** tệp, **mọi** phần mở rộng, **đệ quy**.
    let dir = manifest_dir().join("capabilities");
    let mut files = Vec::new();
    collect_files_recursively(&dir, &dir, &mut files);
    files.sort();

    assert_eq!(
        files,
        vec!["main.json".to_owned()],
        "Mọi tệp trong `capabilities/` đều được Tauri nạp — MỌI phần mở rộng \
         (`.json`, `.json5`, `.toml`) và MỌI thư mục con (glob của Tauri là `**/*`). \
         Thêm tệp thứ hai là mở một bề mặt IPC mới — cập nhật \
         `main_capability_grants_the_minimum_and_no_plugin_permission` CÙNG LÚC, \
         đừng chỉ thêm tệp"
    );
}

/// Liệt kê mọi tệp dưới `dir`, đường dẫn tương đối so với `base`, dùng `/` trên mọi
/// nền tảng để thông báo lỗi đọc giống nhau ở macOS và Windows (NFR14).
fn collect_files_recursively(base: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("đọc mục trong capabilities/").path();
        if path.is_dir() {
            collect_files_recursively(base, &path, out);
        } else {
            let rel = path.strip_prefix(base).unwrap_or(&path);
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
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

// ── Story 1.3 · AC6 — lớp phủ đo dung lượng font ────────────────────────────────

/// `tauri.nofonts.conf.json` là NỬA KIA của phép trừ đo bộ font trong `.msi`/`.dmg`.
/// CI dựng hai bản từ CÙNG một khâu biên dịch: bản thường (có font) và bản
/// `--config src-tauri/tauri.nofonts.conf.json` (không font); hiệu là dung lượng font.
///
/// 🔴 **Vì sao test này tồn tại: `{}` và `null` khác nhau, và `{}` hỏng IM LẶNG.**
/// Tauri merge cấu hình theo **JSON Merge Patch (RFC 7396)** — `json_patch::merge`,
/// gọi từ `tauri-utils-2.9.3/src/config/parse.rs:7,185`. Đọc `json-patch-3.0.1/src/lib.rs:661-681`:
/// hàm duyệt **từng khoá của patch**; khoá có giá trị `null` thì `map.remove(key)`,
/// còn lại thì merge đệ quy. Patch là object **rỗng** ⇒ vòng lặp chạy **0 lần** ⇒ tài
/// liệu **không đổi**.
///
/// ⇒ Với `{ "bundle": { "resources": {} } }` thì bản "không font" **vẫn có font**, hai
/// số bằng nhau, chênh lệch **0 MiB**, và **không lỗi nào được ném** — CI xanh với một
/// con số vô nghĩa. §Công thức đo trên Windows của báo cáo mũi thăm dò viết đúng chữ
/// `{}` đó; đây là chỗ sửa nó lại, và chỗ khoá nó lại.
///
/// ⚠️ Tên tệp là `tauri.nofonts.conf.json`, KHÔNG phải `tauri.windows.conf.json`: Tauri
/// tự merge `tauri.<platform>.conf.json` vào **mọi** lượt build, và
/// `no_dev_csp_and_no_platform_config_overrides` ở trên cấm đúng điều đó.
#[test]
fn nofonts_overlay_drops_resources_with_an_explicit_null() {
    let path = manifest_dir().join("tauri.nofonts.conf.json");
    assert!(
        path.exists(),
        "thiếu {} — không có nó thì AC6 của Story 1.3 không có nửa kia của phép trừ",
        path.display()
    );

    let conf = read_json("tauri.nofonts.conf.json");

    let root = conf
        .as_object()
        .expect("`tauri.nofonts.conf.json` phải là một object JSON");
    let keys: Vec<&str> = root.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        vec!["bundle"],
        "Lớp phủ này chỉ được làm ĐÚNG MỘT việc: gỡ `bundle.resources`. Mọi khoá thêm \
         vào đây đều lặng lẽ đổi bản build đang được đem đi trừ, và phép đo font hết \
         nghĩa. Thấy: {keys:?}"
    );

    let bundle = root["bundle"]
        .as_object()
        .expect("`bundle` phải là object");
    let bundle_keys: Vec<&str> = bundle.keys().map(String::as_str).collect();
    assert_eq!(
        bundle_keys,
        vec!["resources"],
        "cùng lý do như trên — đúng một khoá. Thấy: {bundle_keys:?}"
    );

    // Điểm mấu chốt. `conf["bundle"]["resources"]` cũng trả `Null` khi khoá VẮNG MẶT,
    // nên phải hỏi qua `get()`: khoá phải CÓ MẶT và giá trị phải là `null` tường minh.
    let resources = bundle
        .get("resources")
        .expect("`bundle.resources` phải CÓ MẶT — vắng mặt là một no-op y hệt `{}`");
    assert!(
        resources.is_null(),
        "`bundle.resources` phải là `null` tường minh, không phải `{resources}`.\n\
         RFC 7396: chỉ `null` mới XOÁ khoá. `{{}}` duyệt 0 khoá và không đổi gì — bản \
         'không font' vẫn có font, chênh lệch bằng 0, CI xanh, phép đo vô nghĩa.\n\
         Đọc doc-comment của test này trước khi sửa."
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
//  Móc chuyển hướng `$APPDATA` của bộ e2e — AD-45, Story 1.22 AC2
// ═════════════════════════════════════════════════════════════════════════════════

/// Chính sách CHUNG của hai móc e2e, kiểm ở **mọi** bộ feature.
///
/// Hàm đó cố ý KHÔNG bị `cfg` gác, đúng để ca này chạy cả trong `cargo test` mặc định mà
/// hook `pre-push` gọi. Thứ bị gác là phép **đọc** biến môi trường, không phải luật.
/// Một chính sách dùng chung cho cả `$APPDATA` lẫn thư mục gốc Library, vì hai móc khác
/// nhau ở **cái gì bị chuyển hướng**, không ở **giá trị nào hợp lệ**.
#[test]
fn the_dir_override_rejects_everything_that_is_not_an_absolute_path() {
    use auratranslate_lib::absolute_dir_override_from_raw as parse;
    use std::ffi::OsStr;

    assert_eq!(parse(None), None, "không đặt biến ⇒ không chuyển hướng");

    assert_eq!(
        parse(Some(OsStr::new(""))),
        None,
        "chuỗi RỖNG là một lượt đặt hỏng. Nó phải trả None chứ không được coi là một \
         đường dẫn — nhưng cũng đừng đọc kết quả này thành 'rỗng thì an toàn': ở chỗ gọi \
         nó rơi về `$APPDATA` thật, và đó chính là lý do bộ lái phải TỰ KIỂM rằng kho \
         nằm trong thư mục tạm sau mỗi lượt chạy."
    );

    assert_eq!(
        parse(Some(OsStr::new("e2e-data"))),
        None,
        "đường dẫn TƯƠNG ĐỐI bị từ chối, không được phân giải. Nó phân giải theo thư mục \
         làm việc của tiến trình con — thứ bộ lái đặt, không thứ người viết ca test nghĩ \
         — nên nó đẻ một `global.db` ở một chỗ bất kỳ trong kho mà không ai báo."
    );

    let abs = if cfg!(windows) {
        r"C:\Temp\aura-e2e"
    } else {
        "/tmp/aura-e2e"
    };
    assert_eq!(
        parse(Some(OsStr::new(abs))),
        Some(std::path::PathBuf::from(abs)),
        "đường dẫn tuyệt đối là ca DUY NHẤT được nhận"
    );
}

/// Phép **đọc** biến môi trường chỉ tồn tại sau `all(debug_assertions, feature = "wdio")`.
///
/// 🔴 Vì sao một ca đọc MÃ NGUỒN chứ không phải một ca hành vi: hai lớp gác của AD-45 là
/// một tính chất **lúc biên dịch**, và một nhị phân test chỉ chạy được đúng một bộ feature
/// mỗi lượt — nó không tự quan sát được bộ feature kia. Đọc nguồn là cách duy nhất khẳng
/// định cả hai nhánh cùng lúc, và nó chạy ở **mọi** bộ feature.
#[test]
fn the_env_read_lives_only_behind_debug_assertions_and_the_wdio_feature() {
    const GUARD: &str = r#"#[cfg(all(debug_assertions, feature = "wdio"))]"#;

    // 🔴 Danh sách này phải mọc theo MỌI móc e2e mới. Một móc thứ ba không có tên ở đây là
    // một đường ghi vào dữ liệu thật của người chạy mà không bất biến nào canh — đúng cách
    // bề mặt THỨ HAI (thư mục gốc Library) đã lọt qua AC2 và chỉ lộ ra ở lượt đọc mã của C2.
    const ENV_NAMES: [&str; 2] = [
        "AURATRANSLATE_E2E_DATA_DIR",
        "AURATRANSLATE_E2E_LIBRARY_ROOT",
    ];

    let path = manifest_dir().join("src/lib.rs");
    let src = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

    for name in ENV_NAMES {
        // Tên biến xuất hiện ĐÚNG MỘT LẦN trong toàn bộ mã sản phẩm. Hai chỗ khai cùng một
        // tên là hai chỗ có thể trôi khỏi nhau, và chỗ trôi sẽ ghi vào dữ liệu thật.
        //
        // ⚠️ `_DATA_DIR` là tiền tố của chính nó chứ không của tên kia, và hai tên không
        // chứa nhau, nên phép đếm chuỗi con ở đây không chồng lấn. Thêm một móc tên
        // `AURATRANSLATE_E2E_DATA_DIR_2` sẽ làm phép đếm này sai — đừng đặt tên như vậy.
        let occurrences = src.matches(name).count();
        assert_eq!(
            occurrences, 1,
            "`{name}` phải xuất hiện đúng MỘT lần trong `src/lib.rs`, thấy {occurrences}."
        );
    }

    // Mỗi lần **dùng** hằng đó trong MÃ phải đứng sau một dòng gác
    // `all(debug_assertions, wdio)`.
    //
    // ⚠️ Giới hạn ghi thẳng: dòng chú thích bị bỏ qua. Doc-comment của `AD-45` và của
    // `data_dir_override` nhắc tên hằng bằng văn xuôi, và một ca bắt cả văn xuôi sẽ đỏ vì
    // một câu giải thích — tức nó dạy người đọc rằng cách cho xanh là **xoá lời giải
    // thích**. Bộ lọc dưới đây thô (`//` ở đầu dòng đã trim); nó bỏ sót một lời gọi nhét
    // sau `/* … */` trên cùng dòng, và đó là một hình dạng mã không tồn tại trong tệp này.
    let lines: Vec<&str> = src.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if !line.contains("E2E_DATA_DIR_ENV") && !line.contains("E2E_LIBRARY_ROOT_ENV") {
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            continue;
        }
        let window_start = i.saturating_sub(40);
        let guarded = lines[window_start..i]
            .iter()
            .any(|earlier| earlier.trim() == GUARD);
        assert!(
            guarded,
            "dòng {} nhắc một hằng móc e2e mà không có `{GUARD}` ở trên trong 40 dòng \
             gần nhất:\n  {}\n\n\
             AD-45 đòi HAI lớp gác và cần cả hai. Bỏ `feature = \"wdio\"` để lại một \
             biến môi trường đọc được trong MỌI bản debug — tức mọi lượt `tauri dev` của \
             người khác — và một biến như vậy chuyển hướng kho dữ liệu mà không ai bấm \
             gì. Nếu bạn tới đây để nới nó: đừng, hãy hỏi trước.",
            i + 1,
            line.trim()
        );
    }
}

/// Bộ lái e2e và mã Rust phải khai **cùng một** tên biến.
///
/// 🔴 Đây là ca đắt nhất của nhóm này, vì chỗ trôi hỏng **IM LẶNG và theo hướng tệ nhất**:
/// đổi tên ở Rust mà quên `wdio.conf.mjs` thì móc chuyển hướng ngừng có tác dụng, bộ e2e
/// quay lại ghi thẳng vào `global.db` THẬT của người chạy, và **mọi ca vẫn xanh** — vì
/// một kho thật cũng là một kho mở được. Không có ca này thì lượt hồi quy đó không có
/// một cổng nào chặn.
#[test]
fn the_e2e_runner_and_the_rust_side_name_the_same_variables() {
    const ENV_NAMES: [&str; 2] = [
        "AURATRANSLATE_E2E_DATA_DIR",
        "AURATRANSLATE_E2E_LIBRARY_ROOT",
    ];

    let path = manifest_dir()
        .join("..")
        .join("e2e")
        .join("wdio.conf.mjs");
    let conf = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));

    for name in ENV_NAMES {
        assert!(
            conf.contains(name),
            "`e2e/wdio.conf.mjs` không nhắc `{name}`.\n\n\
             Nếu vừa đổi tên biến ở `src/lib.rs`, đổi luôn ở đó. Bỏ qua ca này nghĩa là bộ \
             e2e ghi vào dữ liệu THẬT của người chạy trong khi mọi ca vẫn xanh — `global.db` \
             với biến thứ nhất, `~/Documents/AuraTranslate/` với biến thứ hai."
        );
    }
}
