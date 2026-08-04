//! `dict-build` — build tool nội bộ, gộp năm nguồn từ điển thô thành `dict-core.db`.
//! KHÔNG vào bản phát hành (AD-25, Story 1.9). Bố cục `lib.rs` + `main.rs` (chỉ gọi
//! `build::run`) là điều kiện để `tests/` `use` được mã của chính crate này — cùng quy
//! ước `auratranslate_lib` của `src-tauri/Cargo.toml`.

pub mod build;
pub mod char_idx;
pub mod finalize;
pub mod insert;
pub mod licenses;
pub mod model;
pub mod schema;
pub mod sources;
pub mod sources_meta;
