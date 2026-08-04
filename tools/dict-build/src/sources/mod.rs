//! Năm parser, mỗi nguồn một module, cùng chữ ký
//! `parse(reader) -> impl Iterator<Item = Result<RawEntry, ParseIssue>>` (Task 4 của
//! Story 1.9). Cùng hình dạng cho năm nguồn là điều kiện để AC2 vế "không hợp nhất"
//! kiểm được — xem doc-comment `model::RawEntry`.

pub mod cc_cedict;
pub mod cedict_common;
pub mod cvdict;
pub mod en_wiktionary;
pub mod unihan;
pub mod viwiktionary;
pub mod wiktextract_common;
