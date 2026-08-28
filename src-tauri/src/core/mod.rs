//! Quy tắc nghiệp vụ của AuraTranslate — mọi thứ frontend không được biết (AD-1).
//!
//! Một module cho một khái niệm miền, không phải cho một nhóm năng lực: nhóm C1–C10
//! là từ vựng sản phẩm và không xuất hiện trong tên module (Consistency Conventions).

pub mod ai;
pub mod dict;
pub mod export;
pub mod glossary;
pub mod i18n;
pub mod library;
pub mod lifecycle;
pub mod matching;
pub mod scope;
pub mod segment;
pub mod store;
pub mod tm;
pub mod webimport;
