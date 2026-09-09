//! Xuất: docx · md · TMX + segment alignment + khối ghi nguồn (AD-38, AD-43).
//!
//! Cấu trúc đoạn là dữ liệu ĐƯỢC LƯU, không phải thứ suy ra lúc xuất (AD-37).
//! Khối ghi nguồn dựng từ `SOURCE_ORIGIN` lúc chạy, không lưu sẵn (AD-43).
//!
//! Crate dành cho module này: `docx-rs` (bộ GHI `.docx`).
//!
//! 🔵 **SỬA 2026-09-09 (Story 6.12) — `docx-rs` nay cũng có một đường ĐỌC trong kho, nhưng
//! KHÔNG ở đây.** `core::docx` (mới) tự đọc OOXML bằng `zip` + `quick-xml`, không gọi
//! `docx_rs::read_docx` — 140 điểm panic trong `docx-rs/src/reader/` dưới `panic = "abort"`
//! làm một cổng "tệp hỏng" không đóng được bằng bất kỳ lớp chắn nào (xem doc-comment đầu
//! `core::docx`). `docx-rs` ở lại ĐÚNG một vai tại đây: bộ GHI cho `core::export` (AD-38,
//! Epic 8) — và thêm vai THỨ HAI, bộ SINH FIXTURE cho bộ test của Story 6.12
//! (`tests/fixtures_docx.rs`), một cài đặt ĐỘC LẬP với `core::docx` nên không phải một vòng
//! tròn "tự sinh rồi tự đọc lại".
