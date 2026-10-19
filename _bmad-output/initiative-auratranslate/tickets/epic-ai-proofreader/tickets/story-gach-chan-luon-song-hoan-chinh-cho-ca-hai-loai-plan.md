---
title: 'Gạch chân lượn sóng hoàn chỉnh cho cả hai loại'
type: 'feature'
ticket: '5'
created: '2026-10-11'
status: done
baseline_revision: '4cd183f83d09c6542f6d5d91e0522210422a4b59'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Gạch chân của proofreader (9.1/9.2) vẽ bằng `::highlight()`, mà WKWebView bỏ qua `text-underline-offset` trong đó, nên sóng đè lên chấm dưới `ạ ộ ợ` (đo lại 2026-10-11: đè 3 điểm ảnh thiết bị); và loại nghĩa vẽ bằng `tm-rule`, chỉ 2,27–2,56:1 ở theme sáng, dưới 3:1 WCAG AA phi văn bản mà tiêu chí FR86 đòi.

**Approach:** Bọc mỗi cụm có phát hiện trong một `<span>` inline trong ô để `text-decoration` thật nhận `text-underline-offset: 4px`; loại nghĩa đổi sang `tm-text`; giữ nguyên luật xoá phát hiện khi gõ của AD-53 mục 5 và chứng minh nó bằng e2e gõ thật.

**Decision (Ice 2026-10-11), cách vẽ:** span trong ô `contenteditable`, không lớp phủ, không giữ Highlight API. Đo trên WKWebView hệ thống (macOS 15.6, dpr 2, Source Serif 4 15px/1.95, "học ạ ộ ợ"): `::highlight()` có hay không offset đều đè chấm 3 điểm ảnh thiết bị; span offset 4px hở 3; lớp phủ SVG hở 6 nhưng cần miễn trừ `src/AGENTS.md:19`.

**Decision (Ice 2026-10-12, sau review 1), undo gốc:** ô dựa vào undo gốc của `contenteditable` (AD-49 ①). Đo WKWebView hệ thống: gõ → bọc span → ⌘Z ra `"Tôi họcc"` thay vì `"Tôi "`; gõ → bọc → gõ "x" (gỡ không gộp ở `beforeinput`) → ⌘Z ra `"Tôi c"`. Gỡ VÀ gộp text node trước khi engine chọn nút để sửa thì cả ba ca đúng. Ice chọn A': giữ span, gỡ + gộp ở `keydown` của ô và ở `beforeinput` `historyUndo`/`historyRedo`; phím không sinh `input` thì vẽ lại sau phím; không chạm DOM khi đang composition.

**Decision (Ice 2026-10-11), màu loại nghĩa:** `tm-text` (sáng 5,88/5,21/5,70:1 trên `surface`/`surface-accent`/`surface-tm`, tối 7,68/6,22/6,96:1). Đã biết: ΔE với `error` khi mô phỏng mù đỏ-lục chỉ 5,7 ở theme sáng; dải 9.4 vẫn ghi tên loại. `primary` bị loại vì luật "chỉ ba việc", `tm-rule` vì trượt 3:1. Giữ trọn plan (~2.100 token).

## Boundaries & Constraints

**Always:** Hai loại chỉ khác màu, cùng sóng, cùng offset 4px; màu chỉ qua `var(--color-*)`; không vạch lề, không danh sách rời, chữ trong ô vẫn liền (`cell.textContent` không đổi khi vẽ hay gỡ). Span được gỡ và text node được gộp lại TRƯỚC khi engine chọn nút để sửa: ở `keydown` của ô (mọi phím) và ở `beforeinput` `historyUndo`/`historyRedo`; lượt sửa không qua `keydown` (autocorrect, dán bằng menu) gỡ ở `beforeinput` không gộp (gộp trong `beforeinput` làm WebKit nuốt chữ). Phím không sinh `input` cho ô (mũi tên, phím bổ trợ) ⇒ gạch chân vẽ lại sau phím; nhánh `beforeinput` bị chặn mà không ghi ⇒ vẽ lại. Không sửa DOM ô giữa `compositionstart` và `compositionend`. Gỡ và vẽ giữ nguyên vùng chọn theo offset; mọi đường đọc DOM ô (flush, con trỏ, `restoreEditedText`) chịu được span. Gõ vào ô ⇒ gạch chân của segment đó mất ngay (AD-53 mục 5, đã có ở `clearProofreadFor`); văn bản ô khác `scanned_text` ⇒ không vẽ. Không thêm token màu mới (DESIGN.md §Sàn tương phản).

**Never:** Không đổi luật AD-53, không chạm Rust, không đổi giá trị token, không hạ sàn của `check-tokens`. Không lớp phủ, không `position: absolute` trong ô. Không để span lọt vào văn bản được lưu hay lịch sử undo của ô.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Dấu dưới | phát hiện trên "học ạ ộ ợ", theme sáng và tối | sóng hở khỏi chấm dưới ≥1 điểm ảnh thiết bị, đúng màu loại | — |
| Hai loại một câu | một chính tả + một nghĩa | hai sóng, hai màu, mỗi sóng dưới đúng cụm | — |
| Gõ sửa | gõ một ký tự vào ô có phát hiện | mọi gạch chân của segment đó mất; segment khác giữ | — |
| Xuống dòng | cụm chữ vắt qua hai dòng trong ô | sóng dưới cả hai đoạn | — |
| Ô đổi rộng | kéo rộng cột hoặc đổi theme | sóng theo chữ, không lệch | — |
| Gõ giữa cụm | con trỏ trong cụm bị gạch, gõ chữ hoặc IME tiếng Việt | ký tự vào đúng vị trí, ô không còn span, `textContent` = văn bản cũ + ký tự | — |
| Undo sau quét | gõ, quét (vẽ span), ⌘Z | ô về đúng văn bản trước lượt gõ, như khi không có span | — |
| Gõ tiếp rồi undo | gõ, quét, gõ thêm, ⌘Z | ô về đúng văn bản như khi không có span | — |
| Di chuyển con trỏ | mũi tên trong ô có gạch chân, không gõ | gạch chân vẫn hiện sau phím | — |
| Thao tác bị chặn | dán rỗng hoặc dán bị từ chối vào ô có gạch chân | gạch chân vẫn hiện, văn bản không đổi | — |
| Phát hiện chồng/kề nhau | hai khoảng chạm nhau hoặc lồng | mỗi khoảng một span, không span lồng span; khoảng chồng lấn bị bỏ như AD-53 mục 6 | — |

</frozen-after-approval>

## Code Map

- `src/panels/proofreadHighlight.ts` -- `rangeForOffsets` (đi nhiều text node, dùng lại); `rangesForScan` không còn người gọi khi vẽ bằng span ⇒ bỏ cùng ca test của nó, lá chắn văn bản cũ sống một chỗ trong hàm bọc; `paintProofreadRanges` + hai tên highlight bỏ.
- Native undo: `src/commands/keys.ts:510` (AD-49 ①, `Mod+Z` luôn nhường ô gõ) — không thêm undo tự viết.
- `GridPanel.vue` `onEditKeydown` (gắn `@keydown` trên cột tgt) — chỗ gỡ + gộp; `onBeforeInput` :~1335 — nhánh `insertParagraph`, `FROM_OUTSIDE` với các lối ra sau `preventDefault` không ghi (`flat === ''`, dán bị từ chối) cần vẽ lại.
- Đường đọc DOM ô cần soát chịu được span: grep `textContent`/`innerText`/`firstChild`/`childNodes` trên `.cell-tgt` trong `GridPanel.vue` và `editorPanelState.ts` (`doiConTroToi` :1631, `moveEditorCaretTo`, flush của `noteEditorEdit`).
- `src/panels/GridPanel.vue` -- `repaintProofread` :1088, watcher :1107, gọi từ `restoreEditedText` :1082, xoá khi unmount :1108; `reportEdit` :1577 gọi `clearProofreadFor` :1581; CSS `::highlight(proofread-error|meaning)` :2420-2432 (comment WKWebView). Ô thường nền `surface`, `.editing`/`.row-selected` nền `surface-accent` :2467, :2483.
- `src/config/proofread.ts:15` `isMeaningKind` -- phân loại sang hai màu, giữ.
- `src/proofreadState.ts:211` `clearProofreadFor` -- AD-53 mục 5, không sửa.
- `src/tokens/tokens.json` -- roles (`stroke`, `text`), `contrast.neverTextTokens`; `scripts/check-tokens.mjs` không có kiểm 3:1 cho nét; `text-decoration-color` thuộc `PURE_COLOR_PROPS` :816.
- `e2e/specs/proofread-underline.e2e.mjs` -- đo điểm đổi màu (`tokenPixels` :70-88, dpr-agnostic theo 9f6b83b); phần "clears on typing" chỉ kiểm `rangesForScan(text+'x')`.
- Test: `tests/frontend/proofreadHighlight.test.ts`, `proofreadWiring.test.ts`, `proofreadActions.test.ts`.
- `deferred-work.md` -- mục "Gạch chân lượn sóng … chạm dấu nằm dưới" `Chủ: Story 9.5` (grep "chạm dấu nằm dưới"): đóng ở story này.

## Tasks & Acceptance

**Execution:**
- [x] `src/panels/proofreadHighlight.ts` -- bọc khoảng UTF-16 thành span có class theo loại (lá chắn văn bản cũ ở đây, khoảng chồng bị bỏ, kề nhau không lồng), gỡ span của một ô có cờ gộp text node, giữ vùng chọn theo offset; một hàm vẽ lại cả cột (gỡ + gộp rồi bọc theo `proofreadResults`) để vitest gọi được; bỏ `rangesForScan`, `paintProofreadRanges`
- [x] `src/panels/GridPanel.vue` -- `repaintProofread` gọi hàm vẽ lại cả cột, unmount gỡ; `onEditKeydown` gỡ + gộp ô đích rồi hẹn vẽ lại sau phím (bỏ qua khi đang composition, khi ô không còn phát hiện thì không có gì để vẽ); `beforeinput` history* gỡ + gộp, kiểu khác gỡ không gộp; các lối ra `preventDefault` không ghi gọi vẽ lại; cờ composition từ `compositionstart`/`compositionend`; CSS hai class: `text-decoration-line/-style`, `-thickness: 1px`, `text-underline-offset: 4px`, màu `--color-error` / `--color-tm-text`; bỏ hai quy tắc `::highlight()` và comment WKWebView
- [x] `tests/frontend/proofreadHighlight.test.ts` -- hai loại hai class, cụm vắt nhiều text node, văn bản cũ không bọc, gỡ có/không gộp, khoảng kề không lồng, vùng chọn giữ offset, hai ô: gõ một ô thì ô kia vẫn có span sau vẽ lại
- [x] `tests/frontend/proofreadWiring.test.ts` -- dò mã nguồn chỉ cho các lời gọi trong `GridPanel.vue` (keydown, history*, lối ra bị chặn); không ca nào tự gắn listener rồi tự gọi hàm gỡ
- [x] `e2e/specs/proofread-underline.e2e.mjs` -- giữ phép đo lần 1 (hai theme, khoảng hở mực/sóng, cụm vắt hai dòng, đổi theme); thêm: gõ → vẽ → `execCommand('undo')` ra văn bản trước lượt gõ; gõ → vẽ → `KeyboardEvent('keydown')` trên ô → `insertText` → undo đúng; `keydown` `ArrowLeft` không `input` ⇒ span có lại sau phím; `CompositionEvent` giả không làm DOM đổi giữa start/end
- [x] `deferred-work.md` -- đóng mục nợ 9.1 "chạm dấu nằm dưới" bằng `→ ✅ ĐÃ ĐÓNG <ngày> (Story 9.5)`; thêm mục nợ `Chủ: Epic 9`: phím thật, Telex và autocorrect với undo chưa ai thử trong app thật

**Acceptance Criteria:**
- Given một ô có phát hiện, when hiển thị, then văn bản `cell.textContent` và vị trí con trỏ trước/sau khi vẽ là như nhau.
- Given e2e `proofread-underline`, when gỡ thật `text-underline-offset` khỏi CSS, then ca khoảng hở đỏ đúng lý do; when gỡ thật lời gọi gỡ + gộp ở `keydown` hoặc ở history*, then ca undo tương ứng đỏ.
- Given lượt gõ đầu tiên vào ô có gạch chân, when flush lưu, then văn bản lưu bằng `textContent` không có dấu vết span.

## Implementation Notes

- Vẽ bằng span trong ô; hàm vẽ lại cả cột `repaintProofreadHost` tách khỏi `GridPanel.vue`. Gỡ + gộp ở `keydown` và `beforeinput` history*, gỡ không gộp ở các `beforeinput` khác, bỏ qua khi `composing`/`isComposing`; vẽ lại hẹn `setTimeout 0` sau phím, sau `compositionend` và sau ba lối `preventDefault` không ghi.
- e2e 5/5 xanh trên máy Ice (dpr 2): mực 291, sóng 295 hai theme. Đối chứng thật: gỡ `unwrapProofread(typed, true)` ở keydown ⇒ ca undo-keydown đỏ (`"Tôi họcc"` thay `"Tôi "`) và ca mũi tên đỏ; đổi merge history thành `false` ⇒ ca undo-history đỏ (`"Tôi họcc"`); xanh: cả hai `"Tôi "`, `spansAfterWrap`=1.
- Ảnh `after.png` hai theme (sáng `/var/folders/xs/y5b0kcx56rb0gbxjd20s4t5h0000gn/T/auratranslate-e2e-proofread-BtXNqf`, tối `…-L1VLsh`): dưới "học ạ ộ ợ" sóng đỏ hở khỏi chấm dưới, không đè; dưới "trường lớp" sóng vàng-nâu (tm-text), đọc được trên cả nền sáng lẫn tối; hai sóng cùng nét và cùng độ lùi.

## Plan Change Log

- 2026-10-12 loopback 1 (intent_gap): review quick + đo WKWebView thấy span phá undo gốc của ô (bảng ở Decision undo gốc). Ice chọn A' và hoàn tác toàn bộ mã lần 1; diff lần 1 là tư liệu (`scratchpad/diff-9-5-round1.patch` của phiên build), không phải bản để chép. Trạng thái tránh: gỡ không gộp ở `beforeinput` rồi để lượt vẽ lại gộp sau `input`. KEEP: hình `keepingSelection` (offset `textContent` ↔ điểm DOM) và gỡ bằng `span.replaceWith(...span.childNodes)`; hàm vẽ lại cả cột tách khỏi `GridPanel.vue` để ca "ô khác giữ" chạy bằng vitest (gỡ lời gọi bọc ⇒ đỏ 1/14); phép đo e2e lần 1 đã xanh trên máy Ice dpr 2: mực thấp nhất 291, sóng cao nhất 295 hai theme, gỡ offset ⇒ sóng 289; cụm vắt dòng 2 đoạn, sóng 308/49 điểm; ô thu hẹp +60px mới vắt (+30px không); `browser.keys` không gõ được vào ô, gõ bằng `execCommand('insertText')`. ⚠️ `cell.normalize()` trong `beforeinput` làm WebKit nuốt chữ (`insertText` trả `true`, chữ không vào).

## Review Triage Log

- review 1 · high · intent_gap — `proofreadHighlight.ts` `wrapProofread` + native undo (AD-49 ①, `keys.ts:510`): đo WKWebView hệ thống, gõ "đi họcc" → bọc span → ⌘Z cho `"Tôi họcc"` thay vì `"Tôi "`, lượt `input` sau đó lưu văn bản hỏng; gộp text node ở `beforeinput` history* sửa ca này, nhưng gõ tiếp sau khi bọc rồi ⌘Z vẫn ra `"Tôi c"`; chỉ gộp ở `keydown` mới đúng, mà gỡ ở mọi `keydown` thì phím mũi tên làm mất gạch chân, cần vẽ lại sau phím và tránh IME ⇒ quyết định A va với Never "không để span lọt vào lịch sử undo".
- review 1 · low · patch — `GridPanel.vue` `onBeforeInput`: gỡ span trước các nhánh `preventDefault` không ghi (dán rỗng, dán bị từ chối) ⇒ gạch chân tắt trong khi phát hiện còn chờ, không lượt vẽ lại nào tới.
- review 1 · low · patch — `proofreadWiring.test.ts` ca "a beforeinput on a cell with spans…": tự gắn listener rồi gọi `unwrapProofread` trên một div rời, tự canh mình; gỡ lời gọi ở `onBeforeInput` vẫn xanh.
- review 1 · low · patch — Verification thủ công: ảnh hai theme chưa được xem và ghi lại.
- review 1 · reject — ô đánh dấu task và log trống của plan: cách sửa là sửa plan của chính lượt build.
- review 1 · low · patch — `rangesForScan` không còn ai gọi trong `src/`; lá chắn văn bản cũ chép lại trong `wrapProofread`/`repaintProofreadHost`.
- review 1 · false — `repaintProofreadHost` gộp text node ở ô đang focus làm nuốt phím: lượt vẽ lại chạy trong watcher `flush: 'post'`, không nằm trong `beforeinput`, và `keepingSelection` đặt lại vùng chọn; hại thật của việc tách nút là undo, đã ở dòng high.
- review 1 · false — `setCaret(target.firstChild ?? target, 0)` đặt con trỏ trong span: offset 0 trong span bằng offset văn bản 0; lượt gỡ giữ offset.
- review 2 · low · patch — `GridPanel.vue` `onBeforeInput` nhánh `insertParagraph`, lối ra `!inserted`: gỡ span ở đầu hàm rồi thoát không vẽ lại; Enter bàn phím có `keydown` hẹn vẽ lại, Enter không qua `keydown` thì gạch chân tắt.
- review 2 · low · patch — `proofreadWiring.test.ts` ca lối ra bị chặn đếm `>= 3` lời gọi, mất lời gọi ở một lối ra vẫn xanh nếu có khớp khác.
- review 2 · low · patch — `proofreadHighlight.ts` doc comment của `keepingSelection` nhắc lại tên và thân hàm riêng.
- review 2 · maybe-false · defer (medium nếu đúng) — `GridPanel.vue` cờ `composing` chỉ xoá ở `compositionend`; ô bị dựng lại hay mất focus giữa composition mà không có `compositionend` thì mọi lượt vẽ lại bị bỏ im lặng; cần đo: Telex thật, đổi segment giữa composition.
- review 2 · maybe-false · defer (medium nếu đúng) — `onEditKeydown` hẹn vẽ lại bằng `setTimeout(0)`, dựa vào `beforeinput`/`input` chạy cùng task với `keydown`; phím thật giao `input` ở task sau thì span bọc lại trước khi chữ vào và undo hỏng như review 1; cùng gốc: lượt vẽ lại gộp ô đang focus giữa `keydown` và `input`. Cần đo bằng phím thật.
- review 2 · reject — vẽ lại sau mỗi phím không in đặt lại vùng chọn, có thể mất cột nhớ của mũi tên lên/xuống: chưa đo, chỉ ô có phát hiện, hại nhỏ; sửa cần đổi thiết kế keydown.
- review 2 · reject — Implementation Notes chứa log đối chứng và số đo: cách sửa là sửa plan của lượt build.
- review 2 · reject — frontmatter/ô task/Triage Log lệch nhau: cách sửa là sửa plan của lượt build.
- review 2 · false — `setCaret(target.firstChild ?? target, 0)` đặt con trỏ trong span: carried từ review 1, mã ở đó không đổi.

## Design Notes

Span inline không tách ô, `textContent` không đổi ⇒ tiêu chí "chữ liền" giữ. Đo bằng trang tĩnh trong `WKWebView` hệ thống (script Swift `takeSnapshot`, cùng font và cỡ của token `editor`), không phải app; số điểm ảnh là thiết bị dpr 2. Tương phản và ΔE tính từ hex trong `tokens.json`; deutan theo ma trận Machado 2009 mức 1,0.

## Verification

**Commands:**
- `npx vitest run tests/frontend/proofreadHighlight.test.ts tests/frontend/proofreadWiring.test.ts` -- xanh
- `npm run check:tokens check:layout check:panel-refs` -- xanh
- `npm run test:e2e -- --spec e2e/specs/proofread-underline.e2e.mjs` -- xanh hai theme

**Manual checks (if no CLI):**
- Ảnh chụp hai theme đính vào Implementation Notes; mục real-use "⌘P thật" vẫn là nợ `Chủ: Epic 9`.
