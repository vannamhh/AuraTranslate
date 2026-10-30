# Review AD-62 — đối chiếu với kho thật (2026-10-30)

Phạm vi: spine dòng 1388-1438, tree ở HEAD `7d9ee743` (baseline AD ghi `44573b6e`). Đo bằng Node thuần nạp `src/commands/index.ts` như `check-commands.mjs:1371`, kèm một regex thô trên 67 tệp `.vue`. Script nháp ở scratchpad, không ghi vào kho.

## Verdict

Số liệu cốt lõi đúng, trích dẫn gần như đúng. Một ví dụ miễn trừ (`library.list_works` là `auto`) tự thua điều kiện cổng của chính AD. Có một lỗ thiết kế quanh `tm.manage.*` và vài điểm thiếu rõ. Không có phát hiện chặn.

## Số đo

| Mục | AD nói | Đo được |
|---|---|---|
| command đã đăng ký | 329 | 329 |
| có điểm chuột (literal `dispatch('<id>')` trong `@click/@dblclick/@submit/@contextmenu`, kể cả modifier, `v-on:`) | 259 | 259 (regex thô trên 67 tệp; không id lạ nào) |
| không có điểm chuột | 70 | 70 |
| trong 70 có hợp âm mặc định | 32 | 32 (`spec.keys` và `effectiveBindings()` cùng ra 32) |
| tổng command có hợp âm | không nêu | 70 (`effectiveBindings()` trả mảng 71 hàng, 70 id) |
| command đã có field `pointer` | 0 (tiền đề) | 0 |

Hệ quả: 259/70/32 vững. Sàn quần thể `ceil(0.85 x 259)` = 221.

## Phát hiện

### F1 (Cao) — `library.list_works` không đạt điều kiện `auto` của chính AD
AD mục 3 nêu `library.list_works` khi vào Thư viện là ví dụ `auto` và đòi "ít nhất một `dispatch('<id>')` literal trong mã `src/` ngoài template và ngoài `src/commands/index.ts`". Đo: 0 lời gọi literal. Ngoài `index.ts`, id chỉ xuất hiện trong comment (`src/modes/libraryWorks.ts:9,137`). Việc tải thật là `void loadWorks()` gọi thẳng ở `src/modes/LibraryMode.vue:154,169,189` (command chỉ là cổng phím, `main.ts:696` nối `loadLibraryWorks: loadWorks`). Cùng tình trạng với `library.list_chapters` (0 tham chiếu ngoài `index.ts`). Nếu story thi hành xếp nó `auto`, Kiểm M đỏ. Chọn một trong hai: (a) đổi các `void loadWorks()` thành `dispatch('library.list_works')` để ví dụ thành thật; (b) xếp lại sang `focus`/sổ nợ. Con số "4 auto" ở mục 7 cần đếm lại id theo điều kiện cổng: tôi chỉ xác nhận chắc `lookup.lookup_selection` (`src/main.ts:1302`, literal trong script).

### F2 (Trung bình) — viết lại `@submit="onRowEdit(i)"` thành `dispatch('tm.manage.edit')` mất đối số hàng
`src/TmManageOverlay.vue:722-723` là form theo từng hàng `i`, gọi `onRowEdit(i)`. `dispatch` literal không mang `i`; hai command `tm.manage.edit`/`tm.manage.delete` (dòng 301, 306, 461, 467) đọc `tmManageCursor`. Rewrite chỉ đúng nếu có bước nhắm hàng trước (mousedown/focusin như `aimRow` của Harvest). AD chỉ nói "viết lại thành `dispatch` literal"; ghi rõ điều kiện này vào story vá. Cũng áp dụng cho `PromptLibraryOverlay.vue:691,707` (form theo `row`). Riêng `PromptLibraryOverlay.vue:763` (`onOpenImport`, nút nhập) thì viết lại thẳng được và `prompt.import.open` đúng là id thuộc 70.

### F3 (Trung bình) — chưa nói cách Kiểm M cùng sống với Kiểm K
`@submit` và `@dblclick` nằm trong `HANDLER_ATTR_RE` của Kiểm K (`check-commands.mjs:2355`), có bảng đông cứng. AD mục 1 viết "cùng hình dạng Kiểm A cưỡng chế cho `@click`", nhưng Kiểm A chỉ canh `@click` (`CLICK_ATTR_RE`, dòng ~475). Viết lại `@submit="fn()"` thành `dispatch` sẽ làm bảng K đổi; AD không nói story vá sửa bảng K ra sao. Mục 4 cũng nên nói Kiểm M tự dùng regex literal riêng (hay tái dùng `DISPATCH_ONLY_RE`, hiện không export).

### F4 (Thấp) — ví dụ `via`, `aim`, `gesture` đều khớp mã
- `shortcuts.open`: 0 tham chiếu `.vue`; port `openShortcuts` = `openSettingsToSection('shortcuts')` (`main.ts:958`); `index.ts:~5340` ghi "tới được qua Cài đặt (`Mod+Comma`)". `settings.open` thuộc 259 (có điểm chuột) và không tự khai `pointer`, nên `via` hợp lệ. Có điều đây là suy luận qua UI Cài đặt, cổng không kiểm "mục Phím tắt có trong Cài đặt".
- `glossary.harvest.next`: `GlossaryHarvestOverlay.vue:163` `@mousedown="aimRow(i)"`, phím gọi dispatch ở 100/104. Đúng `aim`.
- `library.work_next`: dispatch từ `LibraryMode.vue:339,342` (phím), danh sách chọn bằng chuột: đúng `aim`.
- `selection.extend_*`: 0 tham chiếu `.vue` (ngoại trừ `segment.selection.extend_down` trong `AiTranslationPanel.vue`): đúng `gesture`.
- `focus.next_panel`, `tm.fuzzy.focus`, `editor.next_segment` là `focus`: hợp lý, nhưng `reason` do review, cổng không kiểm (AD đã tự cảnh báo).

### F5 (Thấp) — độ vững của phép tách 259/70
- Không còn dạng ngoài literal để lọt: `@click` dạng khác (`:onClick`, `v-on=`, `@[dyn]`, hàm) đã bị Kiểm A từ chối đỏ; ở các tệp `.vue` hiện không có `dispatch(` biến số trong thuộc tính chuột. Regex thô với `\s*` xuyên dòng cho cùng 259 như bộ đọc thuộc tính, nên thuộc tính nhiều dòng và nháy kép không làm lệch.
- `dispatch(commandId)` biến số chỉ ở `LookupPanel.vue:144` (`moveTabFocus`, trong script, do `@keydown`), không phải điểm chuột; không tạo false green vì cổng đòi literal.
- Không có component nhận id qua prop rồi dispatch (`:command=`/`commandId` prop: 0 kết quả). Menu dựng từ mảng dữ liệu không tồn tại ở dạng đó; mọi dispatch là literal tĩnh, nên không có false red hiện thời.
- Yếu điểm còn lại: 2 id chỉ có `@dblclick` (`library.open_work`, `reading.lookup_open`) và 5 id chỉ có `@submit` được tính là điểm chuột. `@dblclick` là cửa khó khám phá, nên "có điểm chuột" ở đây yếu hơn nút bấm; AD chấp nhận ngầm, nên nói thẳng. Không id nào chỉ có `@click.self` (backdrop).
- Kiểm tra gate chỉ canh `<template>` của `.vue`; dispatch trong script (ví dụ `AttributionOverlay.vue:117` dùng toán tử ba ngôi) không tính. Đúng thiết kế.

### F6 (Thấp) — sai lệch trích dẫn
| Trích dẫn | Kết quả |
|---|---|
| `GridPanel.vue:962-975` | `onCellMouseDown` bắt đầu 962; doc-comment mouseup tiếp ngay sau. Đúng vùng, nhưng `@mousedown` thật nằm ở export dòng 1871/template; nên trích dòng hàm hơn là dải 14 dòng |
| `TmManageOverlay.vue:723` | đúng (`@submit.prevent="onRowEdit(i)"`) |
| `PromptLibraryOverlay.vue:763` | đúng (`onOpenImport`) |
| `main.ts:1302` | đúng |
| `registry.ts:45` | đúng (`export type CommandSpec`) |
| `index.ts:5391` | đúng (`installCommands`) |
| `check-commands.mjs:238` | đúng (`COMMAND_FLOOR = 269`) |
| `:449-533` | đúng (Kiểm A bắt đầu 449, Kiểm B ở 534) |
| `:1370` | lệch 1: `installCommands(...)` ở 1371; Kiểm E bắt đầu 1341 |
| `commands-scan.mjs:362` | đúng (`scanVueAttrs`) |

### F7 (Thông tin) — các khẳng định cấu trúc đúng
- `maskFile` định nghĩa cục bộ tại `check-commands.mjs:358`, không export; `scanVueAttrs` export chung tại `commands-scan.mjs:362`.
- Chữ Kiểm đang dùng ở `check-commands.mjs`: A B C D E F I J K L; G và H không có ở tệp này (`check-gates.mjs` có "phép G" riêng, khác cổng), M tự do.
- Ba danh sách cổng: `package.json:17`, `.github/workflows/ci.yml:232`, `.githooks/pre-push` (vòng `for gate in ... commands ...`). Header của pre-push còn ghi "12 gates" trong khi chạy 14 (trôi, ngoài phạm vi AD).
- `COMMAND_FLOOR = 269` thấp hơn 85% của 329 (280); sàn mới 221 độc lập, không mâu thuẫn.
- Các số 29 khai + 41 nợ, phân loại `focus 9 · aim 10 · gesture 4 · auto 4 · via 2`, và "15 có hợp âm không đường chuột / 9 không ai gọi" là phân loại đề xuất, tôi không xác nhận được độc lập (chỉ xác nhận 70 = 32 có hợp âm + 38 không hợp âm).

## Việc đề xuất
1. Sửa hoặc thay ví dụ `library.list_works` (F1) trước khi Ice ký; kiểm cả `library.list_chapters`.
2. Ghi vào AD hoặc brief: viết lại form theo hàng cần bước nhắm hàng trước (F2).
3. Nêu cách xử lý bảng Kiểm K khi `@submit` đổi thành `dispatch` (F3).
