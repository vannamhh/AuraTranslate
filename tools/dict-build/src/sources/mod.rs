//! Mỗi nguồn một module, cùng chữ ký
//! `parse(reader) -> impl Iterator<Item = Result<RawEntry, ParseIssue>>` (Task 4 của
//! Story 1.9). Cùng hình dạng cho mọi nguồn là điều kiện để AC2 vế "không hợp nhất"
//! kiểm được — xem doc-comment `model::RawEntry`.
//!
//! **Sáu nguồn NỀN** (`dict-core.db`): `cvdict` · `cc_cedict` · `unihan` ·
//! `viwiktionary` *(vai B)* · `en_wiktionary` · `viwiktionary_en` *(vai A)*.
//! **Hai lớp GỠ RỜI** (mỗi lớp một tệp `.db`, AD-10): `thieu_chuu` · `vietphrase`.
//!
//! ⚠️ `viwiktionary` và `viwiktionary_en` đọc **CÙNG MỘT tệp thô** với hai bộ lọc
//! `lang_code` khác nhau — hai vai song song, hai `source_id` rời nhau. Đọc doc-comment
//! của [`viwiktionary_en`] trước khi định "tối ưu" thành một lượt đọc.

pub mod cc_cedict;
pub mod cedict_common;
pub mod cvdict;
pub mod en_wiktionary;
pub mod thieu_chuu;
pub mod unihan;
pub mod vietphrase;
pub mod viwiktionary;
pub mod viwiktionary_en;
pub mod wiktextract_common;
