---
title: 'Bấm một mục "Tra riêng" ở panel Lookup thì mở popup tra cứu tại chỗ, panel giữ nguyên truy vấn'
type: 'feature'
ticket: ''
created: '2026-11-14'
status: 'built'
baseline_revision: '628afb40388fe37427d1301a4b77286ab82f73fe'
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

**Problem:** Ice báo (2026-11-14, ảnh panel Lookup tra cả một câu, nhánh `not_found`). Bấm một chữ ở dải "Tra riêng:" chạy `lookup.lookup_char` → `runLookup(char)`, nên truy vấn của panel bị thay bằng đúng một chữ: câu gốc và dải "Tra riêng" biến mất, muốn tra chữ kế phải tra lại cả câu.

**Approach:** Bấm một mục của dải mở một popup nổi neo dưới mục đó, hiện bản ghi tra cứu của mục, trong khi panel giữ nguyên truy vấn, dải và nút Concordance. Popup có state tra cứu riêng, không đi qua state toàn cục của panel.

**Decisions (Ice, 2026-11-14):**
- Dải giữ nguyên từng chữ như hiện nay (`singleCharsOf`, khử trùng, gồm cả dấu câu); chỉ đổi đích của cú bấm sang popup.
- Lượt tra trong popup KHÔNG ghi vào Lịch sử tra cứu; lịch sử chỉ giữ truy vấn của panel.
- Giữ plan dài hơn trần 1600 token, chưa đếm được số thật vì máy không có tiktoken offline.
- Nút ghim trên từng mục trong popup ghim được đúng mục đó, như ở panel (quyết sau lượt rà soát 1).

## Boundaries & Constraints

**Always:**
- Panel giữ `currentQuery`, thân panel và dải "Tra riêng" y như trước cú bấm; mở/đóng popup không phát một lượt `runLookup` nào.
- Popup dựng bản ghi bằng `LookupRecord.vue` như panel; nhánh trạng thái (lỗi, không thấy, mọi nguồn tắt, có kết quả, đang chờ) theo cùng thứ ưu tiên với `lookupBody`.
- Lượt tra trong popup có số thứ tự riêng: bấm mục B khi mục A chưa về thì A không bao giờ hiện dưới B.
- Popup đóng khi: bấm ngoài popup, Esc, bấm lại đúng mục đang mở, truy vấn panel đổi, Tác phẩm đổi (`resetLookupPanel`), panel unmount.
- Mục đang mở mang `aria-expanded="true"`; bàn phím (Tab tới mục, Enter/Space) mở được popup như chuột.

**Never:**
- Không đổi popup của Chế độ đọc (`ReadingLookupPopup.vue`): ở đó cú bấm vẫn tra tại chỗ như hiện nay.
- Không biến `lookupPanelState.ts` thành factory, không đổi `runLookup`.
- Không thêm phụ thuộc npm, không đổi Rust.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Có kết quả | Panel `not_found` cho `鐵鏽`, bấm `鐵` | Popup dưới `鐵` hiện bản ghi của `鐵`; panel vẫn "Không nguồn nào có 鐵鏽" | Không |
| Mục không có trong từ điển | Bấm một mục mà lượt tra trả nhóm rỗng | Popup hiện "Không nguồn nào có {mục}" và "Đã tra N nguồn đang bật." | Không |
| IPC lỗi | `lookupDictionary` trả `error` | Popup hiện `panel.lookup.lookup_failed` | Không |
| Lịch sử | Bất kỳ lượt tra nào trong popup, thành công hay không | Lịch sử tra cứu không thêm dòng nào | Không |
| Bấm nhanh hai mục | Bấm A rồi B trước khi A về | Popup neo dưới B, chỉ hiện kết quả B | Lượt A bị bỏ |
| Panel đổi truy vấn khi popup mở | Auto-lookup hoặc `Mod+Alt+L` | Popup đóng | Không |
| Ghim trong popup | Panel `not_found`, popup `鐵` có kết quả, bấm ghim một mục | Mục đó vào bộ ghim, nút chuyển sang trạng thái đã ghim; không hiện `pin_no_target` | Lỗi ghi hiện như ở panel |

</frozen-after-approval>

## Code Map

- `src/panels/LookupPanel.vue:556-580` -- nhánh `not_found`: dải `.lookup-chars` (`@mousedown="aimLookupCharFrom"`, nút `.lookup-char` `data-lookup-char` `@click="dispatch('lookup.lookup_char')"`), nút `.lookup-concordance-pointer`. Vùng cuộn là `.lookup-scroll` (dòng ~501, CSS ~809), không phải `.lookup-body`.
- `src/panels/lookupPanelState.ts:135-149` -- `aimLookupCharFrom`/`lookupAimedChar` (handler của `lookup.lookup_char`); `runLookup` 323 (mẫu số thứ tự lượt `sequence` và `recordLookup` sau guard); `resetLookupPanel` 372; `singleCharsOf` 100.
- `src/panels/lookupView.ts` -- các computed của bản ghi (`lookupGroups`, `lookupHeadReading` qua `sharedReadingText`, `lookupSensesFor`, `lookupBody`) đọc state toàn cục; popup cần bản tương đương đọc state riêng của nó (tách hàm thuần nhận `response` nếu cần, giữ nguyên export cũ).
- `src/config/dict.ts:264` -- `lookupDictionary(query)` → `{ response, error }`.
- `src/panels/dictSourcesState.ts` -- `everySourceOffForRoute`, `enabledSourceCount`.
- `src/modes/ReadingLookupPopup.vue` -- mẫu popup: đặt dưới/lật lên theo hộp cuộn, đóng bằng mousedown bắt pha capture ngoài popup, Esc, resize; trả tiêu điểm khi unmount. Đọc để chép khuôn, không sửa.
- `src/commands/index.ts:768-769, 3448` và `src/main.ts:206, 985` -- đăng ký `lookup.lookup_char` qua port `lookupAimedChar`; đổi handler sang mở/đóng popup, giữ id lệnh.
- `tests/frontend/lookupPanelBlocks.test.ts:342-356` -- ca "clicking looks that character up" đang khẳng định `h.lookups` chứa chữ; phải viết lại theo hành vi mới.
- `tests/frontend/readingLookupPopup.test.ts:270-310` -- dải trong popup đọc; phải còn xanh không sửa.
- `tests/frontend/controls-adoption.test.ts:41` -- miễn trừ `.lookup-char`; giữ nếu lớp không đổi.
- `src/panels/lookupHistoryState.ts` -- bộ ghim: `currentTargets` (149) chỉ `recordLookup` (344) đặt, `resolvePinTarget` (462-486) tìm khoá đã nhắm trong `currentTargets` rồi `pinnedEntries`; `targetsOf` (575) chuyển `LookupResponse` thành `PinTarget[]`; `resetLookupHistory` (398) xoá state. Popup cần một danh sách đích riêng mà `resolvePinTarget` cũng tìm theo khoá đã nhắm (không tham gia nhánh "chỉ một mục"), đặt khi kết quả popup về, xoá khi popup đóng. Không gọi `recordLookup` (sẽ ghi lịch sử).
- `src/panels/PanelFrame.vue:104-108` -- `section.panel` có `tabindex="-1"`: bấm vùng trống của panel đưa tiêu điểm vào section này, không về `body`.
- `src/panels/dictSourcesState.ts:240, 420` -- bật/tắt hoặc xếp lại nguồn gọi `runLookup(currentQuery.value)` với CÙNG truy vấn; `currentQuery` không đổi. `lookupPending` (`lookupPanelState.ts:190`) lên `true` ở mọi lượt `runLookup`, kể cả lượt này.
- `src/modes/PaletteOverlay.vue:100` và các dải `@keydown.esc` không `.stop` -- Esc của chúng vẫn nổi lên `document`.
- `tests/frontend/readingLookupPopup.test.ts:14,116` -- cài port `lookupAimedChar`; phải cài đúng handler mà `main.ts` cài để nhánh Chế độ đọc có ca chạm tới.

## Tasks & Acceptance

**Execution:**
- [ ] `src/panels/lookupCharPopupState.ts` (mới) -- state popup: mục đang mở, nút neo, query/response/error/pending riêng có số thứ tự lượt; `openLookupCharPopup`, `closeLookupCharPopup`; đóng khi một lượt tra của panel BẮT ĐẦU (`lookupPending` lên `true`, bắt cả lượt cùng truy vấn), khi `currentQuery` đổi và trong `resetLookupPanel` -- tách khỏi state panel để panel giữ truy vấn.
- [ ] `src/panels/lookupView.ts`, `src/panels/lookupPanelState.ts` -- rút thành hàm thuần nhận `LookupResponse`/`grouped`: vị từ không-thấy, vị từ quá ngắn, lọc nghĩa của một nhóm, âm đọc chung, và thứ tự nhánh (`lookupBodyOf`); computed cũ của panel và popup cùng gọi các hàm đó -- một luật, không bản chép.
- [ ] `src/panels/lookupHistoryState.ts` -- danh sách đích ghim của popup: đặt/xoá qua một hàm export, `resolvePinTarget` tìm khoá đã nhắm trong nó sau `currentTargets`; xoá cả trong `resetLookupHistory` -- ghim được trong popup mà không ghi lịch sử.
- [ ] `src/panels/LookupCharPopup.vue` (mới) -- popup dùng `LookupRecord`, đặt vị trí theo khuôn `ReadingLookupPopup`, `role="dialog"`. Esc chỉ xử lý khi không `isComposing` và tiêu điểm ở `body`, trong panel Lookup chứa popup; đóng xong trả tiêu điểm về nút khi tiêu điểm đang ở `body`, trong popup, hoặc ở một tổ tiên chứa nút (vd `section.panel`).
- [ ] `src/panels/LookupPanel.vue` -- gắn popup vào `.lookup-scroll`, nút chữ mang `aria-expanded`; dải giữ nguyên `lookupNotFoundChars`.
- [ ] `src/panels/lookupPanelState.ts`, `src/main.ts`, `src/commands/index.ts` -- MỘT handler cho `lookup.lookup_char`: nút nằm trong `[data-reading-lookup]` thì tra tại chỗ như cũ, còn lại mở/đóng popup; xoá handler cũ không còn ai cài; doc của port nói đúng cả hai nhánh.
- [ ] `src/i18n/vi.json` -- nhãn dialog popup; `command.lookup.lookup_char` đúng cho cả panel lẫn Chế độ đọc.
- [ ] `tests/frontend/lookupCharPopup.test.ts` (mới) + sửa `lookupPanelBlocks.test.ts`, `readingLookupPopup.test.ts` (cài handler mới) -- phủ từng dòng của Ma trận, mọi đường đóng (gồm tra lại cùng truy vấn khi đang mở và Esc khi tiêu điểm ở chỗ khác/đang IME không đóng), ghim trong popup, AC1 với truy vấn ba chữ.
- [ ] `scripts/check-commands.mjs`, `scripts/check-i18n.mjs`, `scripts/check-tokens.mjs` -- nâng sàn đếm tệp nếu cổng yêu cầu vì thêm tệp mới (lượt 1 phải nâng).

**Acceptance Criteria:**
- Given panel đang ở `not_found` cho một câu, when bấm lần lượt ba mục khác nhau, then mỗi lần popup hiện bản ghi của mục vừa bấm và panel suốt thời gian đó vẫn hiện tiêu đề "Không nguồn nào có {câu}" cùng đủ dải.
- Given popup đang mở, when nhấn Esc hoặc bấm vào vùng trống của panel, then popup đóng và tiêu điểm về lại mục đã mở nó.
- Given popup đang mở gần đáy `.lookup-scroll`, when không đủ chỗ bên dưới, then popup lật lên trên mục và không tràn ra ngoài hộp cuộn.

## Design Notes

State riêng thay cho factory: state panel có lịch sử, ghim, Concordance probe và hai popup đang dùng chung nó; tách một bản sao nhỏ chỉ có query/response cho popup là thay đổi hẹp nhất. Nếu sau này cần thêm một bề mặt tra thứ ba, lúc đó mới đáng gộp thành factory.

## Verification

**Commands:**
- `npx vitest run tests/frontend/lookupCharPopup.test.ts tests/frontend/lookupPanelBlocks.test.ts tests/frontend/readingLookupPopup.test.ts tests/frontend/controls-adoption.test.ts` -- expected: xanh, mã thoát 0, không dòng Errors.
- `npm run check:i18n && npm run check:tokens` -- expected: xanh.

**Manual checks (if no CLI):**
- Ice, app thật (`npm run tauri dev`): tra một câu không có trong từ điển, bấm vài mục ở "Tra riêng", popup hiện tại chỗ và panel giữ câu.

## Implementation Notes

- Popup giữ state riêng ở `lookupCharPopupState.ts`, không import `lookupPanelState.ts`; chiều import ngược lại (panel đóng popup) để tránh vòng import.
- Đóng khi panel bắt đầu tra bằng `watch(pending, …, { flush: 'sync' })` ngoài `runLookup`, nên lượt tra lại cùng truy vấn sau khi bật/tắt nguồn cũng đóng.
- Một handler `activateAimedLookupChar` cho `lookup.lookup_char`: nút trong `[data-reading-lookup]` tra tại chỗ, còn lại mở/đóng popup; port giữ tên `lookupAimedChar`.
- Ghim: `setPopupPinTargets` cấp đích riêng cho `resolvePinTarget`, chỉ dò theo khoá đã nhắm; popup không gọi `recordLookup`.
- Luật hiển thị rút thành hàm thuần (`lookupBodyOf`, `notFoundOf`, `queryTooShortOf`, `sensesOfGroup`, `headReadingOf`) mà panel và popup cùng gọi.
- Ba cổng (`check:commands`, `check:i18n`, `check:tokens`) đòi nâng sàn đếm tệp khi thêm hai tệp mới.
- Hình học (lật lên, không tràn) và tiêu điểm thật không đo được trong happy-dom; mục nợ Chủ: Ice trong `deferred-work.md`.

## Plan Change Log

- Lượt rà soát 1 (2026-11-14): intent_gap về ghim trong popup, Ice chọn "ghim được". Sửa Code Map/Tasks: đích ghim riêng của popup, đóng khi lượt tra panel bắt đầu, Esc theo tiêu điểm, trả tiêu điểm qua `section.panel`, rút hàm thuần thay bản chép, một handler có ca chạm nhánh Chế độ đọc, AC1 ba chữ. Trạng thái đã tránh: popup treo trên nút đã gỡ, Esc của Mod+K đóng popup, nút ghim báo `pin_no_target`. KEEP: `lookupBodyOf(facts)` dùng chung; `takeAimedCharButton` giữ phần tử nút (không chuỗi) để biết bề mặt; guard số thứ tự lượt; `.lookup-scroll` `position: relative` làm hộp neo; popup gắn trong `.lookup-scroll` với `v-if` theo chữ đang mở.

## Review Triage Log

Lượt 1 (quick): medium 4 · low 2 · false 0 · maybe-false 0 · loại 3. Có intent_gap ⇒ loopback, mã đã revert; bản diff lượt 1 giữ ở scratchpad `tra-rieng.diff`.
- medium · intent_gap · `LookupCharPopup.vue` dựng `LookupRecord` có nút ghim, nhưng `resolvePinTarget` (`lookupHistoryState.ts:462-486`) chỉ tìm trong `currentTargets` của panel (rỗng khi panel `not_found`), nên ghim từ popup báo `pin_no_target`; plan không nói popup có ghim hay không.
- medium · patch · `lookupCharPopupState.ts` đóng popup theo `currentQuery`, mà nó chỉ đổi khi lượt tra về: trong khoảng chờ dải bị gỡ và popup treo trên nút đã tách khỏi DOM; tra lại cùng truy vấn (`dictSourcesState.ts:240,420` sau khi bật/tắt nguồn) không đóng và popup giữ kết quả theo bộ nguồn cũ. Phải đóng khi lượt tra của panel bắt đầu (`lookupPending` lên `true`).
- medium · patch · `LookupCharPopup.vue::onDocumentKeydown` nghe Esc trên toàn `document`, không xét `isComposing` hay tiêu điểm: Esc dành cho bảng Mod+K (`PaletteOverlay.vue:100`) hoặc huỷ IME trong ô dịch cũng đóng popup và kéo tiêu điểm về nút. Chỉ xử lý khi tiêu điểm ở trong panel Lookup hoặc ở `body`, bỏ qua khi đang soạn IME.
- medium · patch · `restoreFocus` chỉ trả tiêu điểm khi `activeElement` là `body`/trong popup; bấm vùng trống làm `section.panel` (`PanelFrame.vue:104-108`, `tabindex="-1"`) nhận tiêu điểm, nên AC2 hỏng ở app thật. Coi tổ tiên chứa nút là "chưa có chỗ" và trả tiêu điểm.
- low · patch · Vị từ không-thấy, lọc nghĩa theo nhóm và âm đọc chung bị chép vào `lookupCharPopupState.ts` thay vì rút hàm thuần nhận `response` ở `lookupView.ts`/`lookupPanelState.ts`; một bên sửa thì bên kia trôi.
- low · patch · Nhánh `[data-reading-lookup]` của `activateAimedLookupChar` không ca nào chạm (`readingLookupPopup.test.ts` vẫn cài `lookupAimedChar` cũ); doc lệnh `commands/index.ts:768` và chuỗi `command.lookup.lookup_char` "…trong một popup" sai với popup Chế độ đọc. AC1 chỉ thử hai mục vì fixture `鐵鏽` có hai chữ.
- loại (low) · Thu gọn khối nguồn trong popup dùng chung tập `collapsed` của panel theo mã nguồn: là sở thích theo nguồn, panel `not_found` không có khối nào để lệch.
- loại (low) · Kéo sash dockview không phát `window.resize` nên vị trí popup cũ đi: chưa rõ mousedown trên sash có đóng popup trước không, và `.lookup-scroll` `overflow: auto` cắt phần tràn; sửa cần thêm ResizeObserver. AC3 không đo được trong happy-dom: Ice kiểm ở app thật.
- loại (low) · Ca bàn phím gọi `button.click()` thay cho phím Enter: nút gốc đổi Enter/Space thành click, happy-dom không mô phỏng.

Lượt 2 (quick): medium 4 · low 2 · false 0 · maybe-false 0 · loại 0. Không intent_gap/bad_plan; 6 patch gửi lại agent dựng mã, 1 defer.
- medium · patch · `lookupPanelState.ts::runLookup` có thêm `closeLookupCharPopup()`, trái luật Never "không đổi `runLookup`"; chuyển sang `watch` đồng bộ trên `pending` nằm ngoài hàm.
- medium · patch · Gỡ panel không đóng popup: `LookupPanel.vue` `onBeforeUnmount` (338) và `LookupCharPopup.vue` `onBeforeUnmount` không gọi `closeLookupCharPopup`, nên lúc gắn lại popup hiện ngay, neo vào nút đã tách và chữ mới mang `aria-expanded="true"`; gọi đóng trong `onBeforeUnmount` của popup.
- medium · patch · `<LookupCharPopup>` gắn ngoài nhánh `lookupTab === 'record'`; `selectLookupTab` qua F3, phím mũi tên hay Mod+K để popup treo trên tab khác; dời vào nhánh record.
- medium · patch · `onDocumentMouseDown` hẹn `restoreFocus` cho mọi mousedown ngoài popup; bấm ngoài panel (tiêu điểm về `body`) hoặc vào nút tab/Concordance (WebKit không trao tiêu điểm) kéo tiêu điểm về chữ; chỉ trả khi đích nằm trong `section.panel` và không phải điều khiển.
- low · patch · Ca "unmounting the panel" chỉ khẳng định Esc không ném lỗi, không thể đỏ; khẳng định `lookupCharPopupChar === null`, thêm ca đổi tab và ca bấm ngoài panel.
- low · patch · `readingLookupPopup.test.ts:306` khẳng định `lookups` và popup còn, vẫn đúng khi gỡ nhánh `[data-reading-lookup]`; khẳng định đầu mục popup đọc thành `控`.
- medium · defer · AC3 (lật lên, không tràn) và lượt dùng thật chưa có mục nợ có chủ; đã thêm vào `deferred-work.md`, Chủ: Ice.
