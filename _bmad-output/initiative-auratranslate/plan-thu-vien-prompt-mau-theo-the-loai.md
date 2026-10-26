---
title: 'Thư viện prompt mẫu: một bộ mẫu cho mỗi thể loại'
type: 'feature'
ticket: ''
created: '2026-10-26'
status: 'built'
baseline_revision: 'ff123af314ba435cac58e08a46a4407091549181'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Mở Tác phẩm thì bộ prompt được tự chọn theo thể loại (plan `the-loai-chon-tu-danh-sach`), nhưng chỉ khi đã có bộ trùng tên với thể loại. App không có sẵn bộ nào, nên với 29 thể loại thì Ice phải tự soạn từng bộ từ đầu, kể cả ba biến chèn động.

**Approach:** Viết sẵn 29 thân prompt mẫu, mỗi mẫu cho một thể loại trong `GENRE_IDS`. Mẫu dùng làm bộ prompt: tên là nhãn thể loại, thân chứa đủ `{{source_segment}}`, `{{glossary_terms}}` và `{{tm_similar_segments}}`. Ngoài ra thân mẫu có một đoạn hướng dẫn riêng cho thể loại đó, ví dụ cách xưng hô, mức dùng Hán Việt, văn phong hay thuật ngữ.

**Đã chốt (Ice, 2026-10-26):**
- Mẫu là bảng tĩnh trong webview (khuôn `AI_PROVIDER_PRESETS`) cộng một lối "Thêm từ mẫu" trong Thư viện prompt. Bấm thì mẫu được tạo thành bộ thật qua `prompt_set_create`. Không thêm lệnh Rust, không thêm migration.
- Phần hướng dẫn trong thân mẫu viết bằng tiếng Việt.
- System prompt cố định, sửa được là plan riêng (`plan-system-prompt-sua-duoc.md`, commit `983ef03bef91b02be123ad7df5fcf533fed2b74a`), làm TRƯỚC plan này. Thân mẫu bỏ những gì system prompt đã nói (dịch sang tiếng Việt, chỉ trả về bản dịch, giữ dòng, dùng đúng thuật ngữ), chỉ giữ phần riêng của thể loại và ba biến. Ice chấp nhận rằng khi system prompt bị tắt thì bộ mẫu không còn dặn "chỉ trả về bản dịch" (2026-10-26).
- Giữ một plan dù vượt 1600 token. Plan 1429 từ, chưa đếm được bằng tiktoken vì máy không có gói này.

## Boundaries & Constraints

**Always:**
- Tên bộ tạo ra từ mẫu lấy bằng `genreLabel(id)` lúc tạo. Như vậy hàm tự chọn (`applyGenre`, so khớp sau khi trim và không phân biệt hoa thường) nhận ra bộ đó, và đổi nhãn trong `vi.json` không làm mẫu lệch tên.
- Thân mẫu không lặp lệnh chung của system prompt mặc định (`src-tauri/src/core/aiconfig/default_system_prompt.txt`).
- Khối Glossary và khối TM có thể rỗng, nên câu trong mẫu không được giả định là chúng có nội dung.
- Thân mẫu nhắc rằng nguồn là tiếng Trung hoặc tiếng Anh (khớp `sourceLangChoices`).
- Một bộ đã có cùng tên ở tầng đích không bao giờ bị ghi đè hay sửa thân.
- Kiểu `Record<GenreId, …>` ép mẫu phủ đủ cả 29 thể loại ngay lúc biên dịch.

**Never:**
- Không thêm cột vào `prompt_set`: bảng chỉ giữ name và body (Quyết định #4).
- Không tự tạo bộ khi mở Tác phẩm mà chưa có bộ trùng tên. Thêm bộ là việc Ice chủ động làm.
- Không tự cập nhật bộ đã thêm khi văn bản mẫu đổi ở bản phát hành sau.
- Không có mẫu cho "Chưa chọn".

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Thêm mẫu | Tầng Toàn cục chưa có "Tiên hiệp" | Có bộ "Tiên hiệp" với thân mẫu | Không lỗi |
| Đã có bộ trùng tên | Toàn cục có "tiên hiệp" (chữ thường) | Mẫu đó hiện "Đã có", không tạo bộ thứ hai | Gặp `name_taken` ngoài dự kiến: bỏ qua mẫu đó, các mẫu khác vẫn được tạo |
| Tác phẩm đang mở | Tác phẩm thể loại Kiếm hiệp đang mở, chưa có bộ được chọn; Ice thêm mẫu "Kiếm hiệp" | Bộ "Kiếm hiệp" được tự chọn ngay, không cần mở lại Tác phẩm | — |
| Ice đã chọn tay | Ice đã chọn tay bộ "X", sau đó thêm mẫu trùng thể loại | Giữ nguyên "X" | — |
| Lỗi ghi | `prompt_set_create` lỗi IO | Báo lỗi qua `tError`; các bộ đã tạo trước đó vẫn còn | Không rollback |

</frozen-after-approval>

## Code Map

- `src/modes/libraryWorkCard.ts:28-73` -- `GENRE_IDS` (29 id, đang private), `GenreId`, `genreLabel`. Phải export `GENRE_IDS` và `GenreId`.
- `src/i18n/vi.json:837-866` -- nhãn `mode.library.genre_<id>`. Khoá `prompt.library.*` đặt cạnh khoá của Thư viện.
- `src/config/promptset.ts:41-51` -- `PromptSetWire` có `tier` và `shadowed_body`. Danh sách đã phân giải gộp hai tầng, nên "đã có ở Toàn cục" nghĩa là một bộ `tier === 'global'`, hoặc một bộ `tier === 'work'` có `shadowed_body !== null` (che một bộ Toàn cục cùng tên). "Đã có ở Tác phẩm" nghĩa là `tier === 'work'`.
- `src/promptSetState.ts:54-56,166-204` -- `selectionIsAutomatic`, `pendingGenre`, `applyGenre`, `autoSelectPromptSetForGenre`, `createPromptSet` (dòng 215: có `busy` và re-validate). Tự chọn lại sau khi thêm mẫu cần nhớ thể loại gần nhất.
- `src/PromptLibraryOverlay.vue:732-739` -- hàng "Bộ mới" / "Nhập từ file". Thêm hàng "Thêm từ mẫu" ở đây. Tệp đã 1421 dòng nên bộ chọn mẫu là thành phần riêng.
- `src/aiConfigState.ts:42-54` -- khuôn bảng preset tĩnh để tham khảo.
- `src-tauri/src/core/promptset/vars.rs:42-66` -- ba biến. `core/ai/rag.rs:380-433`: khối Glossary và khối TM là các dòng `nguồn → đích`; khối TM có nhãn `TM_REFERENCE_LABEL` đứng trước phần cặp của người khác.
- Không đổi: schema `prompt_set`, các lệnh `commands/promptset.rs`, `assemble_prompt`.

## Tasks & Acceptance

**Execution:**
- [x] `src/modes/libraryWorkCard.ts` -- export `GENRE_IDS`, `GenreId` -- để bảng mẫu được kiểm đủ 29 khoá.
- [x] `src/promptTemplates.ts` (mới) -- `PROMPT_TEMPLATES: Record<GenreId, string>`: khung chung + đoạn riêng theo thể loại -- nguồn mẫu duy nhất.
- [x] `src/promptSetState.ts` -- `addPromptTemplates(tier, ids)`: bỏ qua id đã có tên trùng ở tầng đích (so sau khi trim, không phân biệt hoa thường), tạo tuần tự, nạp lại một lần, rồi áp lại thể loại gần nhất qua `applyGenre` -- giữ quy tắc "chọn tay thắng".
- [x] `src/PromptTemplatePicker.vue` (mới) + hàng mở trong `PromptLibraryOverlay.vue` -- danh sách 29 mẫu, ô chọn, nhãn "Đã có", xem trước thân chỉ-đọc, chọn tầng (Toàn cục mặc định; Tác phẩm chỉ hiện khi `workTierAvailable`), "Chọn mọi mẫu còn thiếu", "Thêm N bộ".
- [x] `src/i18n/vi.json` -- khoá `prompt.library.template_*`.
- [x] `tests/frontend/promptTemplates.test.ts` (mới) -- mỗi mẫu có đủ ba biến, không có dấu ngoặc biến lạ (dùng hàm tách mà `promptLibraryOverlayRender.test.ts:710` kiểm bằng `tests/fixtures/prompt-variable-markers.json`), đủ 29 khoá, không mẫu nào chứa "chỉ trả về bản dịch".
- [x] `tests/frontend/promptSetState.test.ts` -- các ca của ma trận I/O: trùng tên khác hoa thường, tự chọn sau khi thêm, chọn tay được giữ, lỗi giữa chừng.
- [x] `tests/frontend/promptTemplatePicker.test.ts` (mới) -- render, "Đã có" bị khoá, đếm N, tầng Tác phẩm ẩn khi không có Tác phẩm.

**Acceptance Criteria:**
- Given Thư viện prompt mở, when Ice chọn "Thêm từ mẫu", then thấy 29 mẫu theo nhãn thể loại, xem được thân từng mẫu trước khi thêm.
- Given đã thêm mẫu "Pháp lý" ở Toàn cục, when mở một Tác phẩm thể loại Pháp lý, then dòng "Đang dùng" là "Pháp lý".
- Given đã thêm cả 29 mẫu, when mở lại bộ chọn mẫu, then cả 29 mẫu hiện "Đã có" và không còn gì để thêm.

## Implementation Notes

- Phase Tests bắt được một lỗi: áp lại thể loại sau khi thêm mẫu (`applyGenre`) thay luôn bộ Ice đã chọn tay. Đã sửa: thêm mẫu chỉ áp lại thể loại khi chưa có bộ nào được chọn hoặc bộ đang chọn là bộ tự chọn. Lúc mở Tác phẩm, hành vi cũ giữ nguyên (thể loại khớp thì thắng).
- Thể loại gần nhất được nhớ trong `lastGenre` (đặt bởi `autoSelectPromptSetForGenre`, xoá ở `resetPromptSets`).
- Bộ chọn mẫu là thành phần `PromptTemplatePicker.vue`, thay chỗ khung soạn của Thư viện; Escape đóng bộ chọn trước, rồi mới tới lớp phủ.
- Mọi nút đều là nút `submit` trong form nhỏ, khai `nonCommand: R_SUBMIT_DIRECT` ở `check:commands`, theo khuôn của lớp phủ.
- Diff tính từ baseline có lẫn một dòng `src/aiConfigState.ts`. Đó là bản sửa `check:panel-refs` của plan system prompt, đã commit riêng ở `351207a4fb6cb6824d45396103e1e45401c8b29d`.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 0 · low 4 · false 7.
- low, patch — mọi lượt tạo đều gặp `name_taken` thì không nạp lại, nên bộ chọn vẫn báo các mẫu đó là còn thiếu ⇒ nạp lại hễ đã thử tạo.
- false — báo "Đã thêm 0 bộ": nhánh `busy` không tới được vì nút đã khoá khi `busy`; còn khi mọi lượt gặp `name_taken` thì 0 là đúng, và sau bản vá trên các mẫu đó hiện "Đã có".
- low, rejected — tầng `work` còn kẹt khi Tác phẩm đóng: lớp phủ Thư viện là modal; nếu xảy ra thì lệnh trả `work_tier_unavailable` qua `tError`, không im lặng.
- low, rejected — `lastGenre` còn sau khi đóng Tác phẩm: lần mở Tác phẩm kế tiếp gọi lại `autoSelectPromptSetForGenre`, bộ tự chọn cũ bị thay; người dùng không gặp.
- false — tên không trim/validate: `genreLabel` của một `GenreId` lấy từ `vi.json`, mà `check:i18n` cấm giá trị rỗng.
- false — sổ plan chưa cập nhật: phần này do người điều phối ghi, đã ghi.
- false — `aiConfigState.ts` trong diff: bản sửa đã commit riêng ở `351207a`, nằm sau baseline.
- low, rejected — test chỉ dò một cụm từ của system prompt: task ghi đúng cụm đó; fixture `known` có thêm biến thì test đỏ ồn ào, không im lặng.
- false — `@change` của `onToggle`/`onTierChange` không có trong `HANDLER_TABLE`: `check:commands` không quét `@change` (src/AGENTS.md).
- false — `onOpenTemplates` im lặng khi lưu thân dở thất bại: giống hệt `onOpenCreate` có sẵn, không do thay đổi này sinh ra.
- low, patch — Escape đóng bộ chọn trước và dòng báo lỗi khi ghi lỗi chưa có ca test chạm tới ⇒ thêm hai ca, ca Escape có đối chứng đỏ.

## Design Notes

Khung chung cho mỗi thân. Phần lệnh chung đã nằm ở system prompt nên khung không lặp lại:

```
<Đoạn riêng thể loại, 3–6 dòng: xưng hô, mức Hán Việt, văn phong, thuật ngữ, điều cần tránh>

Thuật ngữ bắt buộc (trống thì bỏ qua):
{{glossary_terms}}

Câu tương tự đã dịch, tham khảo văn phong (trống thì bỏ qua):
{{tm_similar_segments}}

Đoạn nguồn (tiếng Trung hoặc tiếng Anh):
{{source_segment}}
```

Ví dụ đoạn riêng thể loại:
- Tiên hiệp: giữ Hán Việt cho cảnh giới, công pháp, pháp bảo, tông môn (Trúc Cơ, Kim Đan, Nguyên Anh); xưng hô cổ phong (ta – ngươi, tại hạ, sư huynh, tiền bối); văn phong trang trọng, tránh từ hiện đại.
- Thời sự: văn phong báo chí, câu ngắn, trung tính; tên người và địa danh Trung Quốc theo Hán Việt, tên phương Tây giữ nguyên; số liệu và ngày tháng viết theo kiểu Việt Nam.

## Verification

**Commands:**
- `npx vitest run tests/frontend/promptTemplates.test.ts tests/frontend/promptSetState.test.ts tests/frontend/promptTemplatePicker.test.ts tests/frontend/promptLibraryOverlayRender.test.ts` -- expected: xanh.
- `npm run check:i18n && npm run check:tokens` -- expected: xanh.

**Manual checks (if no CLI):**
- Trong app thật: thêm mẫu "Kiếm hiệp", mở một Tác phẩm Kiếm hiệp, dịch một đoạn qua AI Translation, rồi Xem prompt để thấy thân mẫu đã điền đủ biến.
