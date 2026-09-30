//! A source-text scan, no new dependency: every business parameter (camelCase, injected
//! `AppHandle`/`State`/`Window` params dropped) of a `#[tauri::command]` registered in
//! `generate_handler!` must match exactly the keys `invoke()` sends for that command in
//! `src/**/*.ts`.
//!
//! `nfr_bench_mark_and_wait_phase` bị loại: nó chỉ tồn tại sau `#[cfg(feature = "nfr-bench")]`
//! và được một khối JS tự tiêm (`lib.rs::bridge`) gọi thẳng, không qua `invoke()` của
//! `@tauri-apps/api/core` — không có vế `src/**` nào để đối chiếu.

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // module dùng chung: không phải hàm nào ở đây cũng gọi
mod boundary_scan;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn frontend_src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("src")
}

// Rust side — registered command names + parameter signatures

/// Tên lệnh (đoạn cuối đường dẫn) trong `generate_handler![...]` của `lib.rs`, bỏ mọi mục
/// bị `#[cfg(feature = "nfr-bench")]` che — mục đó không có lời gọi `invoke()` nào ở
/// `src/**` để đối chiếu (xem doc-comment đầu tệp).
fn registered_command_names(lib_rs_text: &str) -> Vec<String> {
    let lines: Vec<(usize, String)> = boundary_scan::code_lines(lib_rs_text).collect();
    let start = lines
        .iter()
        .position(|(_, l)| l.contains("generate_handler!"))
        .unwrap_or_else(|| panic!("khong tim thay `generate_handler!` trong lib.rs"));

    let mut names = Vec::new();
    let mut skip_next = false;
    let mut i = start;
    loop {
        i += 1;
        let (line_no, line) = lines.get(i).unwrap_or_else(|| {
            panic!("het tep lib.rs truoc khi gap `])` dong `generate_handler!` bat dau o dong {}", lines[start].0)
        });
        let trimmed = line.trim();
        if trimmed.starts_with("])") {
            break;
        }
        if trimmed.starts_with("#[cfg(") {
            skip_next = true;
            continue;
        }
        let entry = trimmed.trim_end_matches(',').trim();
        if entry.is_empty() {
            continue;
        }
        let name = entry.rsplit("::").next().unwrap_or(entry).to_string();
        if skip_next {
            skip_next = false;
            continue;
        }
        if name.is_empty() {
            panic!("dong {line_no} trong khoi generate_handler! khong doc duoc ten lenh: {entry:?}");
        }
        names.push(name);
    }
    names
}

fn is_injected_param_type(ty: &str) -> bool {
    ty.contains("AppHandle") || ty.contains("Window") || ty.contains("State<") || ty.contains("State ")
}

fn to_camel_case(snake: &str) -> String {
    let mut out = String::with_capacity(snake.len());
    for (i, part) in snake.split('_').enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            out.push_str(part);
        } else {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                out.extend(first.to_uppercase());
                out.push_str(chars.as_str());
            }
        }
    }
    out
}

/// Tách một danh sách tham số Rust theo dấu phẩy Ở TẦNG NGOÀI CÙNG (không tách trong
/// `<...>`/`(...)`, ví dụ `tauri::State<'_, OpenWorkState>` hay `Channel<AiTranslateBatchEventWire>`).
fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth_angle = 0i32;
    let mut depth_paren = 0i32;
    let mut cur = String::new();
    for c in s.chars() {
        match c {
            '<' => {
                depth_angle += 1;
                cur.push(c);
            }
            '>' => {
                depth_angle -= 1;
                cur.push(c);
            }
            '(' => {
                depth_paren += 1;
                cur.push(c);
            }
            ')' => {
                depth_paren -= 1;
                cur.push(c);
            }
            ',' if depth_angle == 0 && depth_paren == 0 => {
                let trimmed = cur.trim().to_string();
                if !trimmed.is_empty() {
                    parts.push(trimmed);
                }
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    let last = cur.trim().to_string();
    if !last.is_empty() {
        parts.push(last);
    }
    parts
}

/// Tách "tên: kiểu" tại dấu `:` ĐẦU TIÊN không phải nửa của `::`.
fn split_name_type(part: &str) -> (String, String) {
    let chars: Vec<char> = part.chars().collect();
    for i in 0..chars.len() {
        if chars[i] != ':' {
            continue;
        }
        let prev_is_colon = i > 0 && chars[i - 1] == ':';
        let next_is_colon = i + 1 < chars.len() && chars[i + 1] == ':';
        if prev_is_colon || next_is_colon {
            continue;
        }
        let name: String = chars[..i].iter().collect::<String>().trim().to_string();
        let ty: String = chars[i + 1..].iter().collect::<String>().trim().to_string();
        return (name, ty);
    }
    panic!("tham so khong co dau `:` phan tach ten/kieu: {part:?}");
}

/// Hai dạng chữ ký một lệnh có thể mang: thường, và generic trên `tauri::Runtime` (vỏ `wire`
/// chạy được trên `MockRuntime`).
fn signature_markers(fn_name: &str) -> [String; 2] {
    [format!("fn {fn_name}("), format!("fn {fn_name}<R: tauri::Runtime>(")]
}

/// Toàn bộ nội dung tham số (không kể ngoặc bao) của `fn {fn_name}(` bắt đầu ở
/// `lines[start_idx]`, gộp thêm các dòng sau nếu chữ ký tràn nhiều dòng (khuôn
/// `confirm_segment`, mỗi tham số một dòng).
fn extract_param_list_text(lines: &[(usize, String)], start_idx: usize, fn_name: &str) -> String {
    let (_, first_line) = &lines[start_idx];
    let (pos, marker) = signature_markers(fn_name)
        .into_iter()
        .find_map(|m| first_line.find(&m).map(|at| (at, m)))
        .unwrap_or_else(|| panic!("chu ky cua `{fn_name}` phai co trong dong da tim thay"));
    let after_open_paren = pos + marker.len();
    let mut buf = String::new();
    buf.push_str(&first_line[after_open_paren..]);
    let mut idx = start_idx;
    loop {
        let mut depth = 1i32;
        let mut found: Option<usize> = None;
        for (i, c) in buf.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        found = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(end) = found {
            return buf[..end].to_string();
        }
        idx += 1;
        if idx >= lines.len() {
            panic!("khong tim thay `)` khep danh sach tham so cua `{fn_name}`");
        }
        buf.push(' ');
        buf.push_str(&lines[idx].1);
    }
}

/// Bộ tham số nghiệp vụ (camelCase) mà `#[tauri::command] fn {fn_name}` đòi — tìm DUY NHẤT
/// một chỗ trong toàn cây `src-tauri/src` nơi `fn {fn_name}(` đứng ngay sau (trong cửa sổ 3
/// dòng mã, đã bỏ chú thích) một dòng `#[tauri::command...]` — phân biệt với hàm THUẦN cùng
/// tên (không lệnh nào của kho này mang thuộc tính đó).
///
/// `files_lines` là `boundary_scan::code_lines` đã chạy MỘT LẦN cho mỗi tệp trước vòng lặp
/// ~98 lệnh của caller (xem `every_registered_commands_business_parameters_match_camel_case
/// _across_the_wire`) — gọi lại `code_lines` (một lượt quét từng ký tự trên TOÀN VĂN BẢN của
/// mọi tệp Rust) một lần cho MỖI lệnh là nguyên nhân đo được của phép quét này chạy 68s một
/// mình: no ~98 lệnh × kích cỡ cả cây nguồn, thay vì đúng một lượt.
fn rust_required_params(files_lines: &[(String, Vec<(usize, String)>)], fn_name: &str) -> BTreeSet<String> {
    let mut matches: Vec<(String, BTreeSet<String>)> = Vec::new();
    let markers = signature_markers(fn_name);

    for (rel, lines) in files_lines {
        for i in 0..lines.len() {
            if !markers.iter().any(|m| lines[i].1.contains(m)) {
                continue;
            }
            let window_start = i.saturating_sub(3);
            let attributed = lines[window_start..i]
                .iter()
                .any(|(_, l)| l.starts_with("#[tauri::command"));
            if !attributed {
                continue;
            }

            let params_text = extract_param_list_text(lines, i, fn_name);
            let mut required = BTreeSet::new();
            for part in split_top_level_commas(&params_text) {
                let (name, ty) = split_name_type(&part);
                if is_injected_param_type(&ty) {
                    continue;
                }
                required.insert(to_camel_case(&name));
            }
            matches.push((rel.clone(), required));
        }
    }

    match matches.len() {
        0 => panic!(
            "khong tim thay chu ky `#[tauri::command] fn {fn_name}(...)` nao trong src-tauri/src \
             -- lenh nay dang ky trong generate_handler! nhung khong co dinh nghia doc duoc"
        ),
        1 => matches.remove(0).1,
        _ => panic!(
            "tim thay NHIEU HON MOT chu ky `#[tauri::command] fn {fn_name}(` trong cay nguon: {:?} \
             -- ten lenh phai duy nhat",
            matches.iter().map(|(rel, _)| rel).collect::<Vec<_>>()
        ),
    }
}

// TypeScript side — CMD_* constants + the keys sent at every call site

/// `text` với mọi chú thích `//`/`/* */` bị trắng ra khoảng trắng (giữ số dòng), còn chuỗi
/// (`'...'`/`"..."`/`` `...` ``) giữ NGUYÊN VĂN — cùng khuôn `boundary_scan::code_lines`
/// nhưng cho cú pháp TypeScript (không có raw string/char literal của Rust).
fn ts_strip_comments(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    while i < n {
        let c = chars[i];
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            while i < n && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            out.push(' ');
            out.push(' ');
            i += 2;
            while i < n && !(chars[i] == '*' && i + 1 < n && chars[i + 1] == '/') {
                out.push(if chars[i] == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            if i < n {
                out.push(' ');
                out.push(' ');
                i += 2;
            }
            continue;
        }
        if c == '\'' || c == '"' || c == '`' {
            let quote = c;
            out.push(c);
            i += 1;
            while i < n {
                if chars[i] == '\\' && i + 1 < n {
                    out.push(chars[i]);
                    out.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                out.push(chars[i]);
                let hit_quote = chars[i] == quote;
                i += 1;
                if hit_quote {
                    break;
                }
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// `chars[i]` mở một chuỗi (`'`/`"`/`` ` ``) — chỉ số ngay sau dấu đóng khớp, bỏ qua escape.
fn skip_string(chars: &[char], i: usize) -> usize {
    let quote = chars[i];
    let mut j = i + 1;
    while j < chars.len() {
        if chars[j] == '\\' && j + 1 < chars.len() {
            j += 2;
            continue;
        }
        if chars[j] == quote {
            return j + 1;
        }
        j += 1;
    }
    chars.len()
}

/// Chỉ số của `}` khớp với `{` đang mở ở `chars[open_idx]`, bỏ qua nội dung mọi chuỗi gặp
/// giữa đường (một `{`/`}` bên trong `'...'` không được đếm).
fn matching_close_brace(chars: &[char], open_idx: usize) -> usize {
    let mut depth = 1i32;
    let mut i = open_idx + 1;
    while i < chars.len() {
        match chars[i] {
            '\'' | '"' | '`' => {
                i = skip_string(chars, i);
                continue;
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    panic!("khong tim thay `}}` khop voi `{{` o vi tri ky tu {open_idx}");
}

/// Tách nội dung một object literal (không kể `{`/`}` bao ngoài) theo dấu phẩy TẦNG NGOÀI
/// CÙNG, bỏ qua nội dung nằm trong `{}`/`[]`/`()`/chuỗi lồng bên trong mỗi mục.
fn split_object_entries(chars: &[char]) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    let mut i = 0usize;
    while i < chars.len() {
        match chars[i] {
            '\'' | '"' | '`' => {
                let end = skip_string(chars, i);
                cur.extend(&chars[i..end]);
                i = end;
                continue;
            }
            '{' | '[' | '(' => {
                depth += 1;
                cur.push(chars[i]);
            }
            '}' | ']' | ')' => {
                depth -= 1;
                cur.push(chars[i]);
            }
            ',' if depth == 0 => {
                let trimmed = cur.trim().to_string();
                if !trimmed.is_empty() {
                    parts.push(trimmed);
                }
                cur.clear();
                i += 1;
                continue;
            }
            _ => cur.push(chars[i]),
        }
        i += 1;
    }
    let last = cur.trim().to_string();
    if !last.is_empty() {
        parts.push(last);
    }
    parts
}

/// Khoá của một mục object literal: vế trái của dấu `:` TẦNG NGOÀI CÙNG, hoặc cả mục nếu
/// nó là dạng viết tắt (`{ segmentId }` khớp `segmentId: segmentId`).
fn entry_key(entry: &str) -> String {
    if entry.starts_with("...") {
        panic!(
            "spread `{entry}` trong doi tuong tham so cua invoke() chua duoc phep quet nay ho tro \
             -- can sua tay ham `entry_key`"
        );
    }
    let chars: Vec<char> = entry.chars().collect();
    let mut depth = 0i32;
    let mut i = 0usize;
    while i < chars.len() {
        match chars[i] {
            '\'' | '"' | '`' => {
                i = skip_string(&chars, i);
                continue;
            }
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            ':' if depth == 0 => {
                return chars[..i].iter().collect::<String>().trim().to_string();
            }
            _ => {}
        }
        i += 1;
    }
    entry.trim().to_string()
}

/// Mọi hằng `const CMD_XXX = 'ten_lenh'` (khai báo lệnh trên dây, quy ước cả kho) trong một
/// tệp — `text` đã bỏ chú thích (`ts_strip_comments`).
fn extract_command_constants(text: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut search_from = 0usize;
    while let Some(rel) = text[search_from..].find("const CMD_") {
        let pos = search_from + rel + "const ".len();
        let rest = &text[pos..];
        let ident_end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(rest.len());
        let ident = &rest[..ident_end];
        search_from = pos + ident_end;
        let after_ident = rest[ident_end..].trim_start();
        let Some(after_eq) = after_ident.strip_prefix('=') else { continue };
        let after_eq = after_eq.trim_start();
        let Some(quote) = after_eq.chars().next() else { continue };
        if quote != '\'' && quote != '"' {
            continue;
        }
        let value_rest = &after_eq[1..];
        let Some(end) = value_rest.find(quote) else { continue };
        let value = &value_rest[..end];
        if !value.is_empty() && value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            map.insert(ident.to_string(), value.to_string());
        }
    }
    map
}

/// Ký tự liền trước `pos` (bỏ khoảng trắng) trong `chars`, hoặc `None` ở đầu tệp.
fn prev_non_whitespace(chars: &[char], pos: usize) -> Option<char> {
    let mut j = pos;
    while j > 0 {
        j -= 1;
        if !chars[j].is_whitespace() {
            return Some(chars[j]);
        }
    }
    None
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Với mỗi lần `const_name` xuất hiện làm THAM SỐ ĐẦU TIÊN của một lời gọi hàm (ký tự liền
/// trước, bỏ khoảng trắng, là `(`) trong `chars` -- một bộ khoá camelCase: rỗng nếu lệnh gọi
/// không tham số thứ hai, hoặc các khoá của object literal theo ngay sau dấu phẩy.
fn call_site_key_sets(chars: &[char], const_name: &str) -> Vec<BTreeSet<String>> {
    let name_chars: Vec<char> = const_name.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + name_chars.len() <= chars.len() {
        if chars[i..i + name_chars.len()] != name_chars[..] {
            i += 1;
            continue;
        }
        let before_ok = i == 0 || !is_ident_char(chars[i - 1]);
        let after_idx = i + name_chars.len();
        let after_ok = after_idx >= chars.len() || !is_ident_char(chars[after_idx]);
        if !before_ok || !after_ok {
            i += 1;
            continue;
        }
        if prev_non_whitespace(chars, i) != Some('(') {
            i += 1;
            continue;
        }

        let mut j = after_idx;
        while j < chars.len() && chars[j].is_whitespace() {
            j += 1;
        }
        if j < chars.len() && chars[j] == ')' {
            out.push(BTreeSet::new());
        } else if j < chars.len() && chars[j] == ',' {
            j += 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && chars[j] == '{' {
                let close = matching_close_brace(chars, j);
                let entries = split_object_entries(&chars[j + 1..close]);
                out.push(entries.iter().map(|e| entry_key(e)).collect());
            } else {
                panic!(
                    "tham so thu hai sau `{const_name}` khong phai mot object literal `{{...}}` \
                     -- phep quet nay chi ho tro khuon do; xem ngu canh quanh vi tri ky tu {j}"
                );
            }
        } else {
            panic!(
                "sau `{const_name}` khong phai `)` cung khong phai `,` -- hinh dang loi goi ngoai \
                 pham vi ho tro cua phep quet nay (vi tri ky tu {j})"
            );
        }
        i = after_idx;
    }
    out
}

// The test — every registered command, keys match exactly on both sides

/// Sàn số lệnh đăng ký trong `generate_handler!`, sau khi trừ mục bị `#[cfg(feature =
/// "nfr-bench")]` che (xem doc-comment đầu tệp).
const REGISTERED_COMMAND_FLOOR: usize = 98;

#[test]
fn every_registered_commands_business_parameters_match_camel_case_across_the_wire() {
    let src_tauri_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let rust_files = boundary_scan::rust_sources(&src_tauri_root);

    let lib_rs_text = rust_files
        .iter()
        .find(|(rel, _)| rel == "lib.rs")
        .map(|(_, text)| text.clone())
        .unwrap_or_else(|| panic!("khong tim thay src-tauri/src/lib.rs trong danh sach quet"));

    let command_names = registered_command_names(&lib_rs_text);
    boundary_scan::assert_population_floor(
        REGISTERED_COMMAND_FLOOR,
        command_names.len(),
        "REGISTERED_COMMAND_FLOOR",
        "lệnh IPC đăng ký trong generate_handler! (đã trừ mục nfr-bench)",
    );

    // Một lượt `code_lines` cho mỗi tệp Rust, TRƯỚC vòng lặp ~98 lệnh bên dưới — xem
    // doc-comment của `rust_required_params` về vì sao lặp lại lượt này theo lệnh là chỗ đắt.
    let rust_files_lines: Vec<(String, Vec<(usize, String)>)> = rust_files
        .iter()
        .map(|(rel, text)| (rel.clone(), boundary_scan::code_lines(text).collect()))
        .collect();

    let ts_root = frontend_src_root();
    let ts_paths = boundary_scan::paths_with_extensions(&ts_root, &["ts"]);
    // (nội dung đã bỏ chú thích, các ký tự của tệp) -- giữ cả hai để tránh làm lại việc quét
    // hằng CMD_* và việc tìm chỗ gọi trên cùng một tệp hai lần.
    let ts_files: Vec<(PathBuf, String, Vec<char>)> = ts_paths
        .into_iter()
        .map(|p| {
            let raw = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("doc {}: {e}", p.display()));
            let stripped = ts_strip_comments(&raw);
            let chars: Vec<char> = stripped.chars().collect();
            (p, stripped, chars)
        })
        .collect();
    assert!(
        !ts_files.is_empty(),
        "khong tim thay tep .ts nao duoi {} -- cay qua nho de la that",
        ts_root.display()
    );

    // command_name -> (da tim thay >=1 cho goi that, hop cac bo khoa quan sat duoc)
    let mut observed: BTreeMap<String, (bool, BTreeSet<String>)> = BTreeMap::new();

    for (_, stripped, chars) in &ts_files {
        let constants = extract_command_constants(stripped);
        for (const_name, command_name) in &constants {
            let key_sets = call_site_key_sets(chars, const_name);
            let entry = observed.entry(command_name.clone()).or_insert((false, BTreeSet::new()));
            for keys in key_sets {
                entry.0 = true;
                for k in keys {
                    entry.1.insert(k);
                }
            }
        }
    }

    // Lệnh còn đăng ký nhưng KHÔNG có adapter TS nào gọi — mỗi mục ở đây có lý do ghi tại
    // đúng chỗ nó bị xoá, không phải một lỗ hổng của phép quét: `cleanup_list_rules`
    // (`src/config/project.ts` gần `CMD_CLEANUP_ADD_RULE`) và hai vỏ `create_work_from_*`
    // (`src/config/project.ts` gần `CMD_PREVIEW_FROM_TEXT`) chỉ còn sống làm hạ tầng fixture
    // của `e2e/specs/**` (`internals.invoke` trực tiếp, ngoài `src/**`).
    const NO_FRONTEND_CALLER: [&str; 3] = ["cleanup_list_rules", "create_work_from_text", "create_work_from_file"];

    let mut missing_commands = Vec::new();
    let mut mismatches = Vec::new();

    for name in &command_names {
        if NO_FRONTEND_CALLER.contains(&name.as_str()) {
            continue;
        }
        let required = rust_required_params(&rust_files_lines, name);
        match observed.get(name) {
            None => missing_commands.push(name.clone()),
            Some((seen, keys)) => {
                if !seen {
                    missing_commands.push(name.clone());
                    continue;
                }
                if keys != &required {
                    let missing: Vec<_> = required.difference(keys).cloned().collect();
                    let extra: Vec<_> = keys.difference(&required).cloned().collect();
                    mismatches.push(format!(
                        "`{name}`: Rust doi {required:?}, invoke() gui {keys:?} (thieu o TS: \
                         {missing:?}, thua o TS: {extra:?})"
                    ));
                }
            }
        }
    }

    assert!(
        missing_commands.is_empty(),
        "cac lenh sau dang ky trong generate_handler! nhung khong tim thay lan goi invoke() nao \
         (truc tiep hoac qua mot ham boc) trong src/**/*.ts: {missing_commands:?}"
    );
    assert!(
        mismatches.is_empty(),
        "lech tham so IPC giua Rust va invoke():\n{}",
        mismatches.join("\n")
    );
}
