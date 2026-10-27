---
title: 'Câu chỉ còn khoảng trắng không hỏi xác nhận ghi đè'
type: 'bugfix'
ticket: ''
created: '2026-10-27'
status: 'built'
baseline_revision: '181ff2809e62da17182196b76ce921d2e3f12b46'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ice xoá trắng một câu rồi bấm "Đưa sang bản dịch", app vẫn hỏi "Bản đang có chưa từng được xác nhận" với ô nháp trống (2026-10-27). Trên đĩa `target_text` của câu là `"\n"` (đo ở `test (12).atproj`, segment 7). Bốn chỗ hỏi trước khi ghi đè miễn hỏi bằng `is_empty()`, trong khi "chưa dịch" ở mọi chỗ khác là `trim().is_empty()`.

**Approach:** Bốn đường ghi không qua gõ (Đưa sang bản dịch, khôi phục phiên bản, nhận thay đổi Reviewer, nhận gợi ý TM) coi `target_text` chỉ có khoảng trắng là không có gì để mất, cùng một quy tắc đặt ở một chỗ.

## Decisions (Ice, 2026-10-27)

1. Chỉ sửa Rust. Editor vẫn có thể lưu `"\n"` khi ô bị xoá trắng; không đụng `GridPanel.vue`.

</frozen-after-approval>

## Implementation Notes

Route oneshot: một hàm dùng chung trong `src-tauri/src/commands/segment/targets.rs` thay bốn bản sao phép kiểm (`targets.rs`, `history.rs`, `review_accept.rs`, `tm_match.rs`), cộng một ca test cho mỗi đường.

- `is_unsigned_draft(tx, segment_id, current_text)` gộp miễn trừ ô trống (`trim().is_empty()`, cùng tập `White_Space` với `aitranslate.rs:513`) và phép tìm bản sao trong `segment_version`. Đường TM giữ thêm vế `current_text != baseline` trước lời gọi.
- Xoá khối chú thích có ngày và lịch sử review ở `history.rs` (dòng đang sửa); lý do nằm trong doc của hàm mới.
- Ca test đặt `"\n"` qua đường thật khi có: `flush_segment_targets` (TM), `save_segment_targets` (khôi phục); promote và Reviewer ghi bằng SQL như ca `accepting_an_empty_segment_needs_no_confirmation`.
- Đối chứng: đổi `trim().is_empty()` về `is_empty()` ⇒ đúng bốn ca mới đỏ, mỗi ca ở một tệp test.
- Chưa rõ vì sao Editor lưu `"\n"` khi ô bị xoá trắng; Ice chọn không sửa ở đợt này.

## Review Triage Log

Lượt 1 (quick): high 0 · medium 0 · low 1 · false 4 · maybe-false 0.

| Verdict | Route | Finding | Evidence |
|---|---|---|---|
| false | — | Ca Reviewer có thể không chạm chốt vì truyền `"\n"` làm đối số | Đối chứng `is_empty()` làm chính ca này đỏ (`alignment_contract`) |
| false | — | Ca TM có thể bị vế `current_text != baseline` cắt ngắn | `type_text` không chạm baseline (vẫn `""`); đối chứng làm ca này đỏ (`tm_contract`) |
| false | — | `pub(super)` + đọc `bool` từ `EXISTS` có thể không biên dịch | Bốn tệp test và clippy `-D warnings` xanh |
| false | — | Đối chứng "bốn ca đỏ" không có bằng chứng | Đã chạy `--no-fail-fast`: đúng bốn ca mới FAILED, mỗi tệp một ca |
| low | reject | Chú thích xoá ở `history.rs` từng ghi giới hạn "xoá sạch rồi khôi phục không được hỏi" | Văn bản mất ở lượt xoá của người dùng, không ở lượt khôi phục; doc của hàm mới nói ô trống không có gì để mất |

## Verification

**Commands:**
- `npm run build`, rồi `cargo test --test ai_translate_contract --test segment_contract --test alignment_contract --test tm_contract` -- expected: xanh.
- Đối chứng: đổi lại `trim().is_empty()` thành `is_empty()` trong hàm dùng chung ⇒ bốn ca mới đỏ.

**Manual checks:**
- App thật: xoá trắng một câu, bấm "Đưa sang bản dịch" ⇒ ghi thẳng, không hỏi.
