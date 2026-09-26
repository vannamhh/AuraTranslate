//! `nfr-bench` isn't in `default` and CI never builds that feature, so a regression in the
//! `"usable"` arm of `nfr_bench_mark_and_wait_phase` (`lib.rs`) can't be caught by an ordinary
//! build/test run. This is a source-text scan that always runs: the `"usable"` arm must hand
//! the phase controller to its own thread (`spawn_phase_controller(`), never call
//! `wait_for_phases` directly on the command-serving thread.

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // module dùng chung: không phải hàm nào ở đây cũng gọi
mod boundary_scan;

use std::path::PathBuf;

/// Nội dung (đã bỏ chú thích) của khối `{ ... }` ngay sau `marker` trong `lines`, tìm bằng
/// cách đếm cân bằng `{`/`}` từ CHÍNH dấu `{` đứng cuối `marker` — gộp thêm các dòng sau nếu
/// khối tràn nhiều dòng.
fn extract_brace_block(lines: &[(usize, String)], marker: &str) -> String {
    let start_idx = lines
        .iter()
        .position(|(_, l)| l.contains(marker))
        .unwrap_or_else(|| panic!("khong tim thay `{marker}` trong lib.rs"));
    let (_, first_line) = &lines[start_idx];
    let pos = first_line
        .find(marker)
        .unwrap_or_else(|| panic!("marker `{marker}` phai co trong dong da tim thay"));
    let after_open_brace = pos + marker.len();
    let mut buf = String::new();
    buf.push_str(&first_line[after_open_brace..]);
    let mut idx = start_idx;
    loop {
        let mut depth = 1i32;
        let mut found: Option<usize> = None;
        for (i, c) in buf.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
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
            panic!("khong tim thay `}}` khep khoi mo boi `{marker}`");
        }
        buf.push(' ');
        buf.push_str(&lines[idx].1);
    }
}

#[test]
fn the_usable_marker_arm_hands_the_phase_controller_to_its_own_thread_instead_of_blocking_the_command_thread() {
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let rust_files = boundary_scan::rust_sources(&src_root);
    let (_, lib_rs_text) = rust_files
        .iter()
        .find(|(rel, _)| rel == "lib.rs")
        .unwrap_or_else(|| panic!("khong tim thay src-tauri/src/lib.rs trong danh sach quet"));

    let lines: Vec<(usize, String)> = boundary_scan::code_lines(lib_rs_text).collect();

    let marker_count = lines.iter().filter(|(_, l)| l.contains("\"usable\" => {")).count();
    assert_eq!(
        marker_count, 1,
        "ky vong DUNG MOT nhanh `\"usable\" => {{` trong lib.rs (tim thay {marker_count}) -- \
         phep quet nay gia dinh no duy nhat de neo dung cho"
    );

    let usable_arm = extract_brace_block(&lines, "\"usable\" => {");
    assert!(
        usable_arm.contains("spawn_phase_controller("),
        "nhanh `\"usable\"` cua `nfr_bench_mark_and_wait_phase` khong con goi \
         `spawn_phase_controller(` -- bo dieu phoi pha se lai chay TREN luong phuc vu command, \
         dung hinh dang loi da do va sua (xem doc-comment `wait_for_phases`/`spawn_phase_controller` \
         o `lib.rs`): mot lenh dong bo giu luong xuyen suot tran cho, tu khoa chinh no"
    );
}
