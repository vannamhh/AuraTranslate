---
title: 'Chế độ đọc: giãn dòng nhảy bậc 7px'
type: 'feature'
ticket: '5'
created: '2026-10-30'
status: 'built'
baseline_revision: '8fa8368aa02e4bf665d7bd8db90ee3d49669a685'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ở Chế độ đọc, giãn dòng hiển thị là cỡ chữ × tỉ số đã lưu, nên khi kéo thanh trượt cỡ chữ hay giãn dòng, hộp dòng rơi ra ngoài lưới 7px (R7, DESIGN.md §Lưới 7px — Giãn dòng; EXPERIENCE.md Chế độ đọc 🔵 2026-10-29).

**Approach:** `readingStyle.lineHeight` phát tỉ số không đơn vị bằng `px / cỡ chữ`, với `px` là bội 7 nhỏ nhất ≥ cỡ × tỉ số hiệu lực; `effectiveLineHeight`, `lineHeightOverride`, giá trị ghi xuống `KEY_READING_PREFERENCES` và `READING_LINE_HEIGHT_FLOOR` không đổi. Phép so có dung sai dấu phẩy động: `19 × 2.210526315789474` ra `42.00000000000001` trong JS, phép `ceil` trần cho 49.

</frozen-after-approval>

## Code Map

- `src/modes/readingState.ts` `readingStyle` -- chỗ sửa duy nhất trong mã. `effectiveLineHeight` vẫn là nguồn `:value` của hai thanh trượt (`ReadingMode.vue`, `SettingsReadingSection.vue`), nên thanh trượt và số ở Cài đặt giữ tỉ số đã lưu.
- `src/modes/ReadingMode.vue:1122` `.column { line-height: var(--reading-line-height) }` -- không đổi; tỉ số không đơn vị theo DESIGN §Lưới 7px (con kế thừa tỉ số, không kế thừa px).
- `tests/frontend/readingTypography.test.ts:49` -- ca di chuyển: `setLineHeight(1.2)` ở mức Cân 17,5px nay phát `'2'` (35px), không `'1.66'`. Ca ba mức (`:33`) giữ nguyên và canh dung sai: thiếu dung sai thì mức Thoáng đỏ.

## Tasks & Acceptance

- [x] `src/modes/readingState.ts` -- `readingStyle.lineHeight` = `String(bộiBảy / cỡ)`, `bộiBảy = 7 · ceil(cỡ × tỉ số / 7 − ε)`.
- [x] `tests/frontend/readingTypography.test.ts` -- sửa ca `:49`; thêm một ca quét mọi cỡ của thanh trượt (14…28 bước 0,5) × mọi tỉ số (1.66…2.4 bước 0,01) qua `setFontSize`/`setLineHeight`, đọc `readingStyle`.

**Acceptance Criteria:**
- Given một cỡ chữ và một tỉ số bất kỳ của hai thanh trượt, when đọc `readingStyle`, then `cỡ × Number(lineHeight)` là bội 7 (lệch ≤ 0,01), không nhỏ hơn `cỡ × tỉ số`, và trừ 7 thì nhỏ hơn nó.
- Given cùng lượt đó, when đọc `effectiveLineHeight`, then nó bằng đúng tỉ số đã đặt.
- Given ba mức `lg`/`md`/`sm` không tinh chỉnh, when đọc `readingStyle.lineHeight`, then nó bằng token (42/19 · 2 · 1.75).

## Implementation Notes

- Oneshot: một computed và một tệp vitest, dưới 50 dòng.
- `gridLineHeightPx` trừ ε = 1e-9 trước `ceil`: `19 × 2.210526315789474` ra `42.00000000000001`, thiếu ε mức Thoáng ra 49px. Ca quét 2175 tổ hợp không chạm được nhiễu này, chỉ ca ba mức canh nó.
- Đối chứng gỡ thật đã làm và trả lại: `lineHeight` về tỉ số trần ⇒ ca quét và ca sàn đỏ; bỏ ε ⇒ hai ca `lg` đỏ (`2.5789…` thay `2.2105…`).
- Câu miễn trừ `aura-allow-text` của số giãn dòng ở `SettingsReadingSection.vue` đổi lý do: số đó nay là tỉ số đã lưu, không phải giá trị trang áp.
- Chưa đo trên WKWebView: tỉ số phát ra là `px / cỡ` dạng double, có thể thấp hơn bội 7 một chút; nợ giao Story 13.7 (spec e2e đo trang đọc).

## Review Triage Log

- maybe-false (medium nếu thật) — tỉ số `px / cỡ` có thể vẽ hộp dòng thấp hơn bội 7 một phần nhỏ (vd. 21 × 2.6666666666666665) nếu WebKit làm tròn xuống; vitest không bố cục. Cần đo `.column` trên WKWebView ⇒ defer, `deferred-work.md` Chủ: Story 13.7.
- low — tên ca sàn nói "giá trị áp lên trang ghìm ở 1.66" trái assert mới ⇒ patch, đổi tên.
- low, bỏ — ca quét chép `max="2.4"` thành `74`; chỉ lệch khi đổi max thanh trượt, sửa phải thêm hằng xuất ra và sửa hai template.
- low — lý do `aura-allow-text` của số giãn dòng ở Cài đặt nói "đang áp", nay sai ⇒ patch.
- low — plan còn ô `[ ]` và số dòng cũ trong Code Map ⇒ patch.

## Verification

- `npx vitest run tests/frontend/readingTypography.test.ts tests/frontend/readingState.test.ts tests/frontend/settingsReadingDataBodies.test.ts` -- xanh, không dòng Errors.
- Đối chứng gỡ thật: `readingStyle.lineHeight` trả lại `String(effectiveLineHeight.value)` ⇒ ca quét mới đỏ; bỏ ε ⇒ ca ba mức đỏ ở `lg`.
