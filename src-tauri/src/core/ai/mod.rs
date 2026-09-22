//! Nhóm năng lực C6, C7 — dịch bằng AI và Smart RAG Injector.
//!
//! **KHÔNG module nào ngoài `ai/` được import `ai/`** (AD-13). Đây là ranh giới
//! cứng nhất của cây nguồn: gỡ trọn `ai/` ra thì phần còn lại của ứng dụng vẫn phải
//! biên dịch và chạy được. Test cưỡng chế ranh giới này thuộc **Story 4.1**.
//!
//! Lắp prompt là một hàm thuần (AD-14). Streaming qua Channel, không tự kết nối lại (AD-22).
//! Khoá API chỉ tồn tại trong Rust, lấy qua crate `keyring` trực tiếp (AD-29).
//!
//! Crate dành cho module này: `reqwest`.
//!
//! 🔵 SỬA 2026-09-17 (Story 4.3) — dòng này từng liệt `keyring` vào module này; dòng đó
//! được viết trước khi có một mã thật nào để mà đúng hay sai. `keyring` nay sống ở
//! `core/aiconfig/keychain.rs`: bí mật khoá API là một domain CẤU HÌNH hai tầng (cùng
//! `core/aiconfig/mod.rs`, §Intent spec 4.3), không phải logic dịch.
//!
//! 🔵 SỬA 2026-09-21 (Story 4.8) — vế còn lại của đoạn trên (*"`core/ai/` (Story 4.8) sẽ
//! ĐỌC khoá qua `core::aiconfig`"*) nay SAI, đo trên mã thật: AD-13 chỉ mở đúng BA cạnh
//! NGƯỢC cho `ai/` — đọc `glossary/`, `tm/`, `segment/` (spec 4.8 §Code Map) — `aiconfig`
//! không nằm trong đó, vì domain đó chưa tồn tại lúc AD-13 được viết; thêm một cạnh thứ tư
//! là một AD mới, không phải một dòng mã (root `AGENTS.md`). Khoá được đọc ở TẦNG LỆNH
//! (`commands/aitranslate.rs`), lộ ra thành `&str` NGAY TRƯỚC khi truyền vào cổng
//! (`ports::TranslationProvider::TranslateRequest::api_key`) — `core/ai/` không gõ tên
//! `keyring` LẪN `core::aiconfig` ở bất kỳ đâu. Nửa còn LOAD-BEARING của câu cũ ("không tự
//! mở keychain") vẫn đúng, và đúng CHẶT hơn cả những gì nó từng đòi.
//!
//! 🔵 THÊM 2026-09-17 (Story 4.6) — [`rag`], dòng mã ĐẦU TIÊN của module này. Smart RAG
//! Injector: [`rag::gather_glossary_context`] (tạp, một lượt gọi Glossary) +
//! [`rag::assemble_prompt`] (thuần, AD-14) — xem doc-comment của `rag` cho hình dạng đầy đủ.
//! `core/mod.rs:6` giữ nguyên `pub mod ai;` TRẦN — KHÔNG thêm `pub use ai::…` ở đó
//! (`tests/ai_boundary.rs::core_mod_rs_declares_the_ai_module_bare_with_no_reexport`).
//!
//! 🔵 THÊM 2026-09-21 (Story 4.8, Phase 2) — [`client`]: cài đặt DUY NHẤT của
//! `ports::TranslationProvider` hôm nay, một client streaming tương thích OpenAI dựng trên
//! `reqwest` ASYNC (không `blocking` — `read_timeout` chỉ tồn tại trên builder async, xem
//! doc-comment của `client`). Tách khung SSE là một hàm THUẦN trong đó, trên một bộ đệm byte
//! — xem doc-comment của `client` cho lý do.
//!
//! 🔵 THÊM 2026-09-22 (Story 4.11) — [`pricing`]: bảng giá BUNDLED, dated, keyed theo model
//! id, và một hàm THUẦN `(model_id, prompt_tokens, completion_tokens) -> Option<f64>`. `client`
//! gọi module này ngay khi đọc được `usage` của chunk cuối để đúc `cost_usd` TRƯỚC khi trả
//! `TranslateOutcome::Done` ra ngoài ranh giới `core/ai/` — không module nào khác trong cây
//! (kể cả `commands::aitranslate`) gọi thẳng `pricing`, đúng lý do `AI_TRANSLATE_SEAM_COMMAND_FILE_MARKER`'s
//! ghi chú "⚠️ GIỚI HẠN THẬT" ở `tests/ai_boundary.rs` không cần mở rộng cho module này.

pub mod client;
pub mod pricing;
pub mod rag;
