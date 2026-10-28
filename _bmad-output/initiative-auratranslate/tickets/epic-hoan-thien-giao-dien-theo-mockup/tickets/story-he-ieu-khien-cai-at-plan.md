---
title: 'Hệ điều khiển: Cài đặt'
type: 'refactor'
ticket: '31'
created: '2026-10-28'
status: 'done'
baseline_revision: '7090244663c562b5b1e83b8e020fcc657f691ed1'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-hoan-thien-giao-dien-theo-mockup/tickets/story-he-ieu-khien-cac-lop-phu-nhap-va-xuat-plan.md'
  - '{project-root}/src/AGENTS.md'
warnings: ['oversized']
deferred: []
---

<intent-contract>

## Intent

**Problem:** `SettingsOverlay` và tám `Settings*Section.vue` còn 46 control tự vẽ bằng lớp scoped (đo bằng bản sao ca canh chuyển đổi chạy trên chín tệp, mốc `70902446`): nút viền `outline`, nút gạch chân, viên thuốc tô `primary`, ô nhập viền `outline` nền `background` chữ mono, thanh phân đoạn tự vẽ (DESIGN.md §Điều khiển).

**Approach:** Chuyển chín tệp sang lớp `ui-*` của story 27 theo bậc và chiều rộng `ch` của DESIGN, xoá CSS dáng scoped, nối chín tệp vào `tests/frontend/controls-adoption.test.ts`; không thêm lớp vào `controls.css`.

## Decisions (build-auto, 2026-10-28)

1. Bậc: thân Cài đặt là form ⇒ `lg` cho mọi nút và ô (DESIGN: `lg` = "Form trong lớp phủ và Cài đặt"), kể cả các `ui-btn` đã có trong `SettingsOverlay` (thêm `ui-btn--lg`); nút Đóng đầu lớp phủ `md` Chìm (tiền lệ 28–30); bảng Phím tắt là hàng dày ⇒ `sm`.
2. "Nút Nguy hiểm Xoá khoá giữ bậc của nó" đọc là giữ bậc Nguy hiểm (`ui-btn--danger`) trong năm bậc của §Nút; nó cùng hàng với Lưu khoá nên cùng cỡ `lg`. Phân cấp giữ nguyên như quyết định 2 của story 29–30: không thêm Chính hay Nguy hiểm nào, dù mockup `settings.html` tô `btn pri` cho Thử kết nối/Xuất bộ phím tắt/Kiểm tra ngay (ghi vào mục nợ để Ice xem).
3. Chiều rộng: ngắn `ui-field--short` cho ngưỡng Glossary, ngưỡng TM, ô giá trị tự nhập của nhiệt độ/max_tokens; vừa `ui-field--medium` cho model, khoá API, tên người dịch (Xuất), tên preset bố cục (đổi tên, lưu mới); kín hàng cho endpoint, ô nhiều dòng system prompt (`ui-field--full`) và ô tìm Phím tắt (`flex: 1 1 auto` scoped vì nằm trong hàng flex với chip hợp âm và số đếm). Chữ ô là `ui-md`, mono bỏ (DESIGN "Chữ luôn `ui-md`", tiền lệ review story 30).
4. Thanh phạm vi Toàn cục/Tác phẩm: `.set-scope-seg` thành `ui-seg`, hai nút thành `ui-seg__item`. Mỗi nút nằm trong `<form>` riêng nên `.ui-seg__item + .ui-seg__item` không khớp; vạch ngăn là rule bố cục `.set-scope-form + .set-scope-form { border-left: 1px solid var(--color-control-rule) }` kèm một dòng comment workaround. Không gộp form (mẫu `@submit.prevent` giữ `@click` = một `dispatch`).
5. Ba nút Thoáng/Cân/Đặc của Đọc bọc trong `<span class="ui-seg">` mới, mỗi nút `ui-seg__item` (DESIGN: công tắc chế độ xem là nút phân đoạn); `role="group"` vẫn ở `.sr-row`. Nút Song ngữ, nút bật/tắt kiểm tra cập nhật và nút Hiện mức dùng là `ui-btn ui-btn--lg` với `aria-pressed` như hiện có (tiền lệ `data-ai-usage-toggle`).
6. Viên thuốc nhà cung cấp và chip nhiệt độ/max_tokens (`ai-pill`, `aria-pressed`) thành `ui-chip` (chip bậc `sm` theo DESIGN).
7. Hai miễn trừ có tên trong `NOT_A_CONTROL`: `src/SettingsOverlay.vue .set-nav-item` (hàng điều hướng mục có vạch lề trái, mockup `settings.html .nav a`; DESIGN không có loại hàng điều hướng, tiền lệ `pl-row`) và `src/SettingsReadingSection.vue .sr-range` (thanh trượt gốc, DESIGN không có loại thanh trượt, ticket giữ nguyên). CSS của hai lớp này giữ nguyên.
8. Ô phím `sc-key` thành `ui-btn ui-btn--sm`; trạng thái chờ hợp âm `.sc-key-armed` chỉ còn `color: var(--color-primary)` (một thuộc tính; viền focus do `ui-btn:focus-visible`). Nút `sc-act` gạch chân thành `ui-btn ui-btn--quiet ui-btn--sm` (quyết định 3 của story 30), giữ `margin-right`.

## Boundaries & Constraints

**Always:**
- Ràng buộc "Always" của story 27–30: token AD-34, mỗi trạng thái một thuộc tính, scoped chỉ còn bố cục, `aria-*`/phím/`data-*`/`@submit.prevent`/thứ tự DOM của nút và ô giữ nguyên (ngoại lệ duy nhất: `span.ui-seg` bọc ba nút mức đọc).
- Lớp mà test/e2e đọc ở lại trong template (danh sách ở Code Map).
- Mỗi miễn trừ mới trong ca canh có tên và lý do; miễn trừ không còn khớp thì ca đỏ.
- Mục nợ mới `Chủ: Epic 12` cho lượt nhìn app thật.

**Never:**
- Không sửa `controls.css`, tệp của story 27–30 và 32–33, DESIGN.md, `check-tokens.mjs`; không thêm dependency, token hay lớp `ui-*` mới.
- Không đổi thanh trượt (`sr-range`) ngoài việc khai miễn trừ; không đổi hàng điều hướng mục ngoài miễn trừ.
- Không né ca canh bằng `box-shadow` hay mẹo khác.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Lưu khoá | Gõ khoá hợp lệ, bấm Lưu khoá | Hàng Khoá API là cùng nút DOM suốt lượt đọc lại; câu xác nhận hiện | Lỗi ⇒ chỉ câu lỗi |
| Phạm vi bị khoá | `scopeIsLocked` | Toàn cục mang `aria-pressed="true"` và `disabled`, vẫn tô nền `on-surface` | — |
| Bắt hợp âm | Bấm ô phím | `sc-key-armed` gắn, chữ ô `primary`, tiêu điểm ở ô | Esc huỷ, không đóng Cài đặt |
| Miễn trừ chết | Bỏ lớp `set-nav-item` hoặc `sr-range` khỏi template | Ca "every NOT_A_CONTROL key still names a control" đỏ | — |

</intent-contract>

## Code Map

Số dòng ở mốc `70902446`. Mẫu chuyển và quy tắc xoá: Code Map của plan ticket 30 (context). ⚠️ Danh sách selector dấu phẩy: xoá trọn, đọc lại khối quanh mỗi chỗ xoá.

- `src/SettingsOverlay.vue` (template 328-930, style 932-1362) -- 345 `set-close ui-btn ui-btn--quiet`; 352 `set-scope-seg ui-seg`; 354/365 `set-scope-btn ui-seg__item`; 407 `set-nav-item` giữ nguyên (miễn trừ); 443/574 `ai-pill ui-chip`; 483/486/509/559/624/666/720/730/745/760/768/774 thêm `ui-btn--lg` vào lớp `ui-btn` sẵn có; 519 (endpoint/model qua `AI_TEXT_FIELDS` = `endpoint`, `model`) → `ai-field-input ui-field ui-field--lg` + `:class` `field === 'endpoint' ? 'ui-field--full' : 'ui-field--medium'`; 588 → `ui-field ui-field--lg ui-field--short`; 634 → `ai-field-input ai-field-textarea ui-field ui-field--full`; 680 → `ui-field ui-field--lg ui-field--medium`; 820 `set-open-library ui-btn ui-btn--lg`. CSS xoá: 979-993 (`.set-close` cả rule, `:focus-visible`), 1121-1157 (`.set-scope-seg` viền và `:has(focus-visible)`, `.set-scope-btn:focus-visible`, `.set-scope-btn`, `[aria-pressed]`, `:disabled`; giữ `.set-scope-form { margin: 0 }`, thêm rule vạch ngăn), 1195-1214 (`.ai-pill*`; giữ `.ai-pills`, `.ai-pill-form`), 1286-1303 (`.ai-field-input`, `:focus-visible`; giữ `.ai-field-textarea { resize: vertical }`), 1352-1361 (`.set-open-library`). Giữ `.set-nav-item`, `.set-nav-item-on`.
- `src/SettingsDataSection.vue` -- 30/33/38/46 `sd-btn ui-btn ui-btn--lg`; xoá `.sd-btn` 93 và `:focus-visible` 103.
- `src/SettingsExportSection.vue` -- 32 `ts-input ui-field ui-field--lg ui-field--medium`; 51 `ts-save ui-btn ui-btn--lg`; xoá 85, 96, 122, 133.
- `src/SettingsGlossarySection.vue` -- 35 `gs-input ui-field ui-field--lg ui-field--short`; 57 `gs-save ui-btn ui-btn--lg`; xoá 97, 108, 134, 145.
- `src/SettingsTmSection.vue` -- 35 `ts-input … --short`; 58 `ts-save ui-btn ui-btn--lg`; xoá 94, 105, 131, 142.
- `src/SettingsLayoutSection.vue` -- 78/92/106/109/124 `sl-btn ui-btn ui-btn--lg`; 97/115 `sl-input ui-field ui-field--lg ui-field--medium`; xoá 194, 203, 208, 218.
- `src/SettingsReadingSection.vue` -- 37/46/55 bọc `<span class="ui-seg">`, mỗi nút `sr-btn ui-seg__item`; 68 `sr-btn ui-btn ui-btn--lg`; 81/97 `sr-range` giữ nguyên; xoá 135-153 (`.sr-btn`, `:focus-visible`, `[aria-pressed]`); giữ `.sr-range`.
- `src/SettingsShortcutsSection.vue` -- 153 `sc-search-input ui-field ui-field--lg`, rule rút về `flex: 1 1 auto`; 253 `sc-key ui-btn ui-btn--sm`; 278/281 `sc-act ui-btn ui-btn--quiet ui-btn--sm`; 312-328 `sc-file-btn ui-btn ui-btn--lg`. Xoá 434 (`.sc-key`), 445, rút 451 về `color`, 476 (giữ `margin-right: 8px`), 489, 505 (giữ flex), 515, 543, 553.
- `src/SettingsUpdateSection.vue` -- 31/42/61 `su-btn ui-btn ui-btn--lg`; xoá 108, 118, 123.
- `tests/frontend/controls-adoption.test.ts` -- `FILES` thêm chín tệp; `NOT_A_CONTROL` thêm hai khoá ở quyết định 7.

Móc phải giữ: `set-nav-item`, `set-nav-form`, `set-main`, `set-title`, `.ai-field-form .ai-field-input`, `ai-field-save`, `ai-field-clear`, `ai-key-field`, `ai-key-actions`, `gs-input`, `gs-save`, `gs-saved`, `gs-alert`, `sc-act` (e2e `shortcuts-capture-mouse`), `sc-alert`, `sc-table`, `su-row`; mọi `data-*`.

## Tasks & Acceptance

**Execution:**
- [x] Chín tệp `.vue` ở Code Map -- đổi lớp, xoá CSS dáng -- quyết định 1–8.
- [x] `tests/frontend/controls-adoption.test.ts` -- chín tệp, hai khoá `NOT_A_CONTROL` -- ca canh nối dài. Đối chứng thật: gỡ mọi `ui-*` khỏi nút Lưu khoá (`ai-field-save` trong `.ai-key-actions`) ⇒ đỏ đúng `SettingsOverlay.vue:<dòng>`; bỏ `sr-range` khỏi một thanh trượt ⇒ đỏ ca khoá chết (nếu còn thanh kia giữ khoá thì gỡ cả hai); cả hai trả lại trùng byte.
- [x] `tests/frontend/` -- sửa test hiện có chỉ khi móc đổi, ghi lý do.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- cuối tệp, `## Deferred from: hệ điều khiển Cài đặt (2026-10-28, ticket 31 của Epic 12)`, một mục ≤ 5 dòng `Chủ: Epic 12`: ảnh app thật mười mục Cài đặt sáng/tối cạnh `he-dieu-khien.html` và `settings.html`; ô phím chờ hợp âm chỉ đổi màu chữ (quyết định 8); vạch ngăn thanh phạm vi (quyết định 4); mockup tô Chính ba nút mà app không (quyết định 2).

**Acceptance Criteria:**
- Given Cài đặt mở ở bất kỳ mục nào, when nhìn mọi control ở hai theme, then ô và nút thân mục cao 32px, bảng Phím tắt 24px, ô theo chiều rộng quyết định 3, không viền `outline`, không gạch chân, không viên thuốc tô `primary`.
- Given ca canh chuyển đổi, when gỡ lớp `ui-*` khỏi một control của chín tệp, then đỏ đúng tệp:dòng.
- Given test hiện có của Cài đặt, when chạy, then xanh, kể cả ca "Lưu khoá ⇒ lượt đọc lại KHÔNG thay mục bằng dòng đang tải".

## Implementation Notes

- Chín tệp chuyển sang `ui-*` theo quyết định 1–8; CSS dáng scoped xoá, chỉ giữ bố cục (`margin`, `flex`), `.set-nav-item*` và `.sr-range` giữ nguyên dưới miễn trừ.
- Lớp `ui-*` có sẵn trong `SettingsOverlay` (12 nút) thêm `ui-btn--lg`; `Xoá khoá` giữ `ui-btn--danger`.
- Thanh phạm vi: vạch ngăn `.set-scope-form + .set-scope-form` kèm một dòng comment workaround (mỗi segment trong `<form>` riêng).
- Ma trận: thêm assert ở `settingsFrame.test.ts` — Toàn cục khoá vẫn `aria-pressed="true"` và mang `ui-seg__item`, rule pressed đứng sau rule disabled trong `controls.css`; ô phím chờ mang `sc-key-armed` + `ui-btn` rồi mất lớp khi Esc, `.sc-key-armed` chỉ khai `color: var(--color-primary)`.
- Đối chứng gỡ thật, trả lại trùng byte: bỏ mọi `ui-*` khỏi Lưu khoá ⇒ đỏ ca một của `SettingsOverlay.vue`; đổi tên `sr-range` ⇒ đỏ ca một của Reading và ca khoá chết; bỏ bind `sc-key-armed` ⇒ đỏ ca Esc của `settingsFrame`.
- e2e `shortcuts-capture-mouse` lúc đầu không chạy được vì cổng 1420 bận; sau review cổng trống, chạy ngoài sandbox thì xanh (1 spec, 1 ca).
- Mục nợ mới đẩy sổ lên 911 mục, sàn `ITEM_FLOOR` 728 của `check:debt-owner` trôi dưới 80% nên cổng báo lỗi hạ tầng; nâng lên 775 = ceil(0,85 × 911) theo đúng câu cổng in.
- Review: vạch ngăn thanh phạm vi thêm `.set-scope-form + .set-scope-form:has(:disabled)` vẽ nét đứt `outline` như `.ui-seg__item:disabled`.

## Plan Change Log

## Review Triage Log

### 2026-10-28 — Review pass
- verdicts: 5 findings — high 0, medium 0, low 3, false 2, maybe-false 0
- findings:
  - `[low]` `[patch]` Hai `ui-seg` (thanh phạm vi, Thoáng/Cân/Đặc) cao 24px, không 32px như AC1 — `.ui-seg__item` cố định 24px và `controls.css` nằm ngoài phạm vi; mỗi `ui-seg` đứng một hàng riêng nên luật "cùng hàng cùng bậc" không vỡ; vá: ghi vào mục nợ `Chủ: Epic 12` để Ice xét.
  - `[low]` `[patch]` Nút `ui-btn` có `aria-pressed` (Song ngữ, kiểm tra cập nhật, Hiện mức dùng) không có dáng đang-bấm; Song ngữ mất sắc `primary` cũ — trạng thái vẫn đọc từ chữ nhãn; sửa thật cần lớp mới trong `controls.css` (ngoài phạm vi); vá: ghi vào mục nợ.
  - `[low]` `[patch]` Vạch ngăn thanh phạm vi luôn liền nét kể cả khi khoá, trong khi `ui-seg` vẽ vạch đứt cho segment khoá — vá: thêm `.set-scope-form + .set-scope-form:has(:disabled) { border-left-style: dashed; border-left-color: var(--color-outline) }`.
  - `[false]` `[reject]` Ô checkbox Tasks chưa tick, Triage Log trống, đối chứng đỏ không có dấu trong diff — tick và triage log là bước Finalize của chính lượt này; đối chứng gỡ thật được chạy trong phiên và trả lại trùng byte (`cmp`), không thuộc diff theo định nghĩa.
  - `[false]` `[reject]` Màu `primary` của `.sc-key-armed` có thể thua `ui-btn` — trong `controls.css` chỉ `.ui-btn` (0,1,0), `.ui-btn:disabled` và `.ui-btn--quiet:active` khai `color`; ô phím không khoá, không Chìm, nên `.sc-key-armed[data-v]` (0,2,0) thắng; `:hover`/`:active`/`:focus-visible` chỉ đổi viền/nền.

## Verification

**Commands:**
- `npx vitest run tests/frontend/controls-adoption.test.ts tests/frontend/controls-css.test.ts tests/frontend/dialogAccessibleName.test.ts $(grep -lE "SettingsOverlay|Settings[A-Z][a-z]+Section" tests/frontend/*.ts)` -- expected: mã thoát 0, không dòng Errors.
- `npm run check:tokens && npm run check:i18n` -- expected: xanh.
- `npx vue-tsc --noEmit` -- expected: không lỗi.
- `npm run test:e2e -- --spec e2e/specs/shortcuts-capture-mouse.e2e.mjs` (ngoài sandbox, cổng 1420 trống) -- expected: xanh; cổng bận thì ghi lại, không suy ra.

**Manual checks:**
- Ice: ảnh app thật mười mục Cài đặt, sáng và tối (mục nợ `Chủ: Epic 12`).

## Auto Run Result

- Thay đổi: `SettingsOverlay` và tám `Settings*Section.vue` chuyển sang lớp `ui-*` của story 27; CSS dáng scoped xoá; thân Cài đặt bậc `lg`, bảng Phím tắt `sm`, ô theo `ch` (ngắn/vừa/kín hàng); ca canh chuyển đổi đọc thêm chín tệp.
- Tệp: `src/SettingsOverlay.vue` (Đóng Chìm, thanh phạm vi `ui-seg` + vạch ngăn trên form, chip `ui-chip`, ô `ui-field`, 12 nút thêm `--lg`); `src/SettingsDataSection.vue`, `SettingsExportSection.vue`, `SettingsGlossarySection.vue`, `SettingsTmSection.vue`, `SettingsLayoutSection.vue`, `SettingsUpdateSection.vue` (nút `lg`, ô `ui-field`); `src/SettingsReadingSection.vue` (mức đọc `ui-seg`, thanh trượt giữ nguyên); `src/SettingsShortcutsSection.vue` (ô tìm `lg`, ô phím `sm`, chờ hợp âm chỉ đổi màu chữ); `tests/frontend/controls-adoption.test.ts` (chín tệp, hai khoá `NOT_A_CONTROL`); `tests/frontend/settingsFrame.test.ts` (assert ma trận: phạm vi khoá, ô phím chờ); `deferred-work.md` (mục `Chủ: Epic 12`); `scripts/check-debt-owner.mjs` (`ITEM_FLOOR` 728 → 775).
- Review: 5 phát hiện; 3 patch (low: vạch ngăn đứt khi khoá; hai mục ghi nợ: `ui-seg` 24px, nút `aria-pressed` không có dáng bấm), 0 defer, 2 bác (false: ô tick/triage log là việc Finalize và đối chứng chạy ngoài diff; màu `.sc-key-armed` không bị `ui-btn` đè).
- Follow-up review: false (patch 3 low, 0 medium, 0 high).
- Kiểm: vitest 16 tệp xanh 295/295, mã thoát 0, không dòng Errors; `check:tokens`, `check:i18n`, `check:debt-owner` (sau khi nâng `ITEM_FLOOR`), `vue-tsc --noEmit` mã thoát 0; e2e `shortcuts-capture-mouse` xanh (1/1, ngoài sandbox); đối chứng gỡ thật đỏ đúng ca rồi trả lại trùng byte (Lưu khoá; `sr-range`; bind `sc-key-armed`).
- Rủi ro còn: chưa ai nhìn app thật mười mục Cài đặt; hai `ui-seg` 24px giữa thân 32px; Song ngữ mất dáng bấm; mockup tô Chính ba nút mà app không — cả bốn trong mục nợ `Chủ: Epic 12`.
