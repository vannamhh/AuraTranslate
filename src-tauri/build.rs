fn main() {
    embed_manifest_for_tests();
    tauri_build::build()
}

/// Nhét app manifest của Windows vào NHỊ PHÂN TEST — vá cho `STATUS_ENTRYPOINT_NOT_FOUND`.
///
/// # Đo được, không suy đoán (2026-08-11)
///
/// `cargo test` trên `windows-2025` ĐỎ ở **12 trên 12** lượt CI kể từ lượt đầu tiên
/// (`30970979907`, 2026-08-05) tới `31467748678`, trong khi job `macos-26` XANH ở cùng
/// commit. Hình dạng lỗi giống hệt nhau ở mọi lượt:
///
/// ```text
/// Running unittests src\lib.rs  (auratranslate_lib-….exe)  -> 5 passed
/// Running unittests src\main.rs (auratranslate-….exe)      -> 0 passed
/// Running tests\config_invariants.rs (config_invariants-….exe)
/// error: process didn't exit successfully: … (exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND)
/// ```
///
/// Nó chết ở khâu NẠP, trước một assert nào — nên `cargo test` dừng ở nhị phân test tích
/// hợp ĐẦU TIÊN theo thứ tự chữ cái và **mười hai** tệp `tests/**` còn lại chưa từng chạy
/// một lần nào trên Windows.
///
/// # Cơ chế
///
/// `tauri-build` nhét manifest qua `tauri-winres` -> `embed_resource::compile()`, và hàm
/// đó phát ra `cargo:rustc-link-arg-BINS` (`embed-resource-3.0.11/src/lib.rs:443`). Từ
/// khoá `bins` nghĩa là: nhị phân sản phẩm CÓ manifest, nhị phân test thì KHÔNG. Manifest
/// khai phụ thuộc side-by-side vào `Microsoft.Windows.Common-Controls` **6.0.0.0**; thiếu
/// nó, trình nạp gắn `comctl32.dll` **v5** và các entry point mà tầng Win32 của `tauri`
/// nhập không tồn tại ở phiên bản đó.
///
/// Vì sao nhị phân unittest của `src/lib.rs` SỐNG mà `tests/config_invariants.rs` CHẾT,
/// dù cả hai đều là target `test`: `config_invariants.rs:105` viết
/// `let _entry: fn() = auratranslate_lib::run;` — lấy địa chỉ hàm ép trình liên kết giữ
/// trọn cây `tauri`, kéo theo bảng nhập Win32. Bộ unittest của lib không chạm `run()` nên
/// cây đó bị loại và bảng nhập không bao giờ hình thành. Tức đây KHÔNG phải một khuyết tật
/// của `config_invariants`; nó là nhị phân đầu tiên đủ "nặng" để lộ ra.
///
/// # Vì sao `-tests` chứ không phải `cargo:rustc-link-arg` trần
///
/// Bản vá thượng nguồn (`crates/tauri/build.rs`, hàm `embed_manifest_for_tests`) dùng
/// `cargo:rustc-link-arg` áp cho MỌI target. Ở kho này làm vậy là nhét manifest **hai
/// lần** vào nhị phân sản phẩm — một lần qua `.res` của `rustc-link-arg-bins`, một lần
/// qua `/MANIFESTINPUT` — và trình liên kết phải hoà giải hai nguồn cho cùng một tài
/// nguyên `RT_MANIFEST`. `-tests` chạm đúng chỗ đang hỏng và để nhị phân phát hành y
/// nguyên: `tauri build` không đi qua đường này.
///
/// Thượng nguồn còn thêm `/WX` (cảnh báo trình liên kết thành lỗi). KHÔNG lấy: nó không
/// thêm sức chẩn đoán nào ở đây — manifest hỏng đã tự nói bằng đúng mã `0xc0000139` ở
/// bước sau — mà mở thêm một đường cho một cảnh báo không liên quan làm đỏ cả job.
///
/// # Vì sao đọc biến môi trường thay vì `#[cfg(windows)]`
///
/// `#[cfg(windows)]` trong `build.rs` nói về máy ĐANG BIÊN DỊCH, không về đích. Hai thứ đó
/// bằng nhau ở CI hôm nay và sẽ khác nhau ở lượt cross-compile đầu tiên.
fn embed_manifest_for_tests() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os != "windows" || target_env != "msvc" {
        return;
    }

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("windows-app-manifest.xml");
    println!("cargo:rerun-if-changed=windows-app-manifest.xml");
    println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
        manifest.display()
    );
}
