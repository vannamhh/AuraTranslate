---
title: 'Menu Nguồn: kéo thả để sắp xếp thay cho nút ↑↓, và danh sách không còn tràn ra ngoài panel'
type: 'feature'
ticket: ''
created: '2026-11-13'
status: 'built'
baseline_revision: '1cfaa08e5b493b8cb2e042199c084d8393a13d54'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Muốn đưa một nguồn lên đầu menu Nguồn của Panel Lookup, Ice phải bấm nút ↑ nhiều lần, mà nút thì nhỏ, khó bấm. Ngoài ra danh sách bị lỗi hiển thị: tên nguồn dài (`white-space: nowrap`) làm danh sách rộng hơn panel. Danh sách neo `right: 0` nên tràn sang trái và bị cắt mất công tắc cùng đầu tên. Nút ↑ ở hàng đầu và ↓ ở hàng cuối bị vô hiệu nên hiện viền đứt.

**Approach:** Ice quyết (2026-11-13): sắp xếp bằng kéo thả. Bỏ hai nút ↑↓ trên mỗi hàng. Một lần thả ghi đúng một lần toàn bộ thứ tự, theo đường ghi sẵn có (`dict_sources_order`, tra lại sau khi ghi, ghi lỗi thì quay về). Hai lệnh bàn phím `Alt+Shift+↑/↓` vẫn giữ. Bề rộng danh sách bị giới hạn trong panel; tên quá dài thì cắt bằng dấu `…`.
- Cơ chế (Ice chọn 2026-11-13): pointer events tự viết với `setPointerCapture`, không dùng HTML5 `draggable`. Lý do: HTML5 trên Windows đòi `dragDropEnabled: false` trong `tauri.conf.json`, còn happy-dom không giả được `DataTransfer`.
- Chỗ cầm (Ice chọn 2026-11-13): một tay kéo ⋮⋮ ở cuối hàng, đúng chỗ hai nút ↑↓ cũ, cao bằng cả hàng. Chỉ tay kéo khởi động kéo; bấm vào phần còn lại của hàng vẫn chỉ bật/tắt.

## Boundaries & Constraints

**Always:**
- Trong lúc kéo, các hàng dời chỗ ngay để thấy trước vị trí thả. Chưa thả thì chưa ghi gì.
- Thả về đúng chỗ cũ, nhấn Esc hay mất pointer capture thì không ghi và thứ tự giữ nguyên.
- Kéo mà không bật/tắt nguồn. Bấm (không kéo) vẫn bật/tắt như hiện nay.
- Danh sách cao hơn chỗ trống (có `maxHeight`) thì tự cuộn khi kéo tới gần mép trên hay mép dưới.
- Focus đang ở hàng nào thì ở lại hàng đó sau khi thả (`refocusSourceRow`).
- `lookup.move_source_up`/`down` mất điểm chuột, nên khai `pointer: { kind: 'gesture' }` (AD-62 §3) và giữ nguyên hợp âm.
- Tên đầy đủ của một hàng bị cắt `…` vẫn đọc được qua `title`.

**Never:**
- Không đổi sắp xếp phía Rust, khoá config hay cách mã hoá.
- Không thêm Tauri command, không thêm dependency, không sửa `tauri.conf.json`.
- Không thêm token màu hay chữ mới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Kéo lên đầu | kéo hàng thứ 5 lên trên hàng 1 rồi thả | hàng 5 thành hàng 1; đúng một `put_config` với toàn bộ thứ tự; tra lại | — |
| Thả tại chỗ | kéo rồi thả lại vào đúng vị trí cũ | không ghi | — |
| Huỷ | đang kéo, nhấn Esc | menu đóng, thứ tự như trước khi kéo, không ghi | — |
| Bấm | bấm phần công tắc/tên của một hàng | bật/tắt nguồn, không ghi thứ tự | — |
| Bấm tay kéo | nhấn rồi nhả tay kéo, không di chuột | không bật/tắt, không ghi | — |
| Ghi lỗi | `put_config` từ chối | thứ tự quay về như trước khi kéo | khuôn rollback của `moveDictSource` |
| Tên dài | tên 60+ ký tự, panel hẹp | danh sách nằm gọn trong panel, tên cắt `…`, công tắc thấy đủ | — |

</frozen-after-approval>

## Code Map

- `src/panels/LookupSourcesMenu.vue:41-47,127-167,188-211` -- `rows` lấy từ `dictSources`; hai nút ↑↓ là `ui-icon-btn` (`:disabled` ở biên ⇒ viền đứt `controls.css:217`). Đo `listMaxHeight` theo `.lookup-body` (:31-39): bề rộng tối đa đo theo cùng khuôn, tính từ mép trái `.lookup-body` tới mép phải của danh sách. `onKeydown` xử Esc trước. Listener `mousedown` trên `document` ở pha capture (AD-58 Rule 4): không đụng.
- `src/tokens/controls.css:516-532` -- `.ui-menu-item` có `white-space: nowrap`. Sửa ở `.sources-name` scoped (`overflow: hidden; text-overflow: ellipsis`), không sửa class dùng chung.
- `src/panels/dictSourcesState.ts:378-447` -- `moveDictSource(code, ±1)`: đổi chỗ ngay, `writeQueue` với `moveSequence`, rollback, `listDictSources` rồi `runLookup`. Tổng quát hoá thành `placeDictSource(code, toIndex)`; `moveDictSource` gọi hàm đó với `from ± 1`. `aimDictSourceFrom` (:344) vẫn nhắm cho bấm bật/tắt.
- `src/commands/index.ts:3292-3305` -- vòng đăng ký hai lệnh dời. Thêm `pointer: { kind: 'gesture', reason: … }` theo mẫu `:3431`. Port ở `:732-735` giữ nguyên.
- `src/i18n/vi.json:1743` -- `panel.lookup.source_move_up`/`_down` mất chỗ dùng ⇒ xoá. Thêm nhãn cho tay kéo nếu có.
- `src/AppIcon.vue` -- chưa có biểu tượng tay kéo. Thêm `grip` (6 chấm, cùng `viewBox 0 0 16 16`).
- AD-34 §1 và AD-62 §1: cú thả mang tham số (chỉ số đích), nên gọi thẳng `placeDictSource`, không qua `dispatch`. Tiền lệ: `saveCurrentChapterOrigin` (`src/modes/libraryChapters.ts:605`). `@pointerdown` nằm ngoài Kiểm A/K/M.
- Tauri: `tauri-utils-2.9.3/src/config.rs:1943-1947`: *"Disabling it is required to use HTML5 drag and drop on the frontend on Windows"*. `tauri.conf.json` không khai `dragDropEnabled` (mặc định `true`), và mã app không dùng `onDragDropEvent`.
- Test: `tests/frontend/lookupSourcesPopup.test.ts:202-283`. Có ba ca nút ↑↓ (:202, :212, :257); ca Alt+Shift (:229), ca rollback (:268) và harness `h.order` (:44) dùng lại được. Môi trường vitest là `happy-dom`, nên `getBoundingClientRect` trả 0 và phải stub chiều cao từng hàng.

## Tasks & Acceptance

**Execution:**
- [ ] `src/panels/dictSourcesState.ts` -- `placeDictSource(code, toIndex)` (không đổi ⇒ không ghi); `moveDictSource` uỷ cho nó -- một đường ghi cho cả phím lẫn chuột.
- [ ] `src/panels/LookupSourcesMenu.vue` -- bỏ nút ↑↓; thêm tay kéo `grip` (`aria-hidden`, `tabindex=-1`, `cursor: grab`, `touch-action: none`); `@pointerdown` trên tay kéo + `setPointerCapture`, xem trước bằng bản sao thứ tự cục bộ, tự cuộn ở mép, Esc/`lostpointercapture` huỷ; `max-width` đo theo panel; tên `…` + `title`.
- [ ] `src/commands/index.ts`, `src/i18n/vi.json`, `src/AppIcon.vue` -- `pointer: gesture`, dọn khoá i18n, biểu tượng tay kéo.
- [ ] `tests/frontend/lookupSourcesPopup.test.ts` -- thay ba ca nút bằng các ca của I/O Matrix: kéo lên đầu (một lần ghi), thả tại chỗ, Esc huỷ, bấm tay kéo không bật/tắt, ghi lỗi thì quay về; cùng một ca `max-width` được đặt từ bề rộng panel đã stub.

**Acceptance Criteria:**
- Given menu Nguồn có 10 nguồn, when Ice kéo nguồn thứ 8 lên đầu bằng một lần kéo, then nó đứng đầu cả menu lẫn nhóm kết quả, và vẫn giữ vị trí đó sau khi khởi động lại.
- Given Panel Lookup hẹp như trong ảnh Ice gửi, when mở menu Nguồn, then thấy đủ công tắc và đầu tên của mọi hàng, không hàng nào bị panel cắt.
- Given `npm run check:commands`, when chạy sau thay đổi, then xanh mà hai lệnh dời không vào sổ nợ.

## Implementation Notes

- Pointer capture đặt trên `<ul>`, không trên tay kéo: khi Vue sắp lại các hàng có `key`, nó dời node DOM của tay kéo và có thể làm mất capture.
- Tự cuộn chạy bằng một vòng `requestAnimationFrame` khi con trỏ ở trong vùng mép 24px. Chỉ số đích được tính lại sau mỗi bước cuộn (`retarget`).
- `moveDictSource` (phím `Alt+Shift+↑/↓`) nay gọi `placeDictSource`, nên phím và chuột đi chung một đường ghi.
- Ca "ghi lỗi" ban đầu vẫn xanh dù cú thả không ghi lần nào. Đã thêm assert rằng có đúng một `put_config`.
- Phép thử tay trong app thật đã ghi vào `deferred-work.md` (`Chủ: Ice`).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 5 · false 2 · maybe-false 0.
- medium · patch · tự cuộn chỉ chạy trong `pointermove`, nên giữ chuột yên ở mép thì danh sách không cuộn, trái với "tự cuộn khi kéo tới gần mép". Cần một vòng `requestAnimationFrame` khi con trỏ nằm trong vùng mép, và một ca test chạm tới nhánh cuộn.
- low · patch · ca `max-width` spy `HTMLElement.prototype.getBoundingClientRect` và `body` mà không gỡ khi assert đỏ (`afterEach` không có `vi.restoreAllMocks`). Spy sẽ rò sang các ca sau.
- low · patch · trong lúc kéo, hàng đang kéo trông như mọi hàng khác và con trỏ vẫn là `grab`. Ice gặp mỗi lần kéo; sửa bằng một class trên hàng đang kéo.
- low · reject · Esc không huỷ được khi focus nằm ngoài menu (menu mở bằng `lookup.toggle_sources_menu` từ lưới). Ca hiếm; sửa phải thêm một listener trên `document`.
- low · reject · `dictSources` đổi giữa lúc kéo, ví dụ một lượt tải lại từ lần ghi Alt+Shift trước đó, thì `dragOrder` cũ làm mất hàng. Khoảng thời gian đó chỉ vài ms; sửa phải thêm nhánh canh.
- low · reject · `min-width` (32 đơn vị) thắng `max-width` khi panel hẹp hơn mức đó, và bề rộng chỉ đo một lần lúc mở menu. Panel trong ảnh của Ice rộng hơn nhiều; `listMaxHeight` có sẵn cùng giới hạn đo một lần.
- false · thứ tự `drag = null` trước `releasePointerCapture` là bất biến chưa ghi: nếu đảo lại, `cancelDrag` chỉ xoá `dragOrder`, còn `onListPointerUp` đã giữ `order` cục bộ và vẫn gọi `placeDictSource`, nên kết quả không đổi.
- false · lý do `gesture` đọc ngược: AD-62 §3 định nghĩa `gesture` là command "thay cho một cử chỉ chuột", và câu "Thay cho kéo thả hàng nguồn bằng tay kéo" nói đúng điều đó. `check:commands` đã chạy xanh (exit 0) sau thay đổi.

## Design Notes

Xem trước bằng mảng cục bộ: `dragOrder` là bản sao các `code`. `rows` đọc theo `dragOrder` khi đang kéo, và `:key="row.code"` cho Vue tự dời node. Chỉ số đích là số hàng có tâm nằm trên `clientY`, tính lại ở mỗi `pointermove` từ `getBoundingClientRect` của từng `li`. Khi thả, gọi `placeDictSource(code, dragOrder.indexOf(code))`. Huỷ thì xoá `dragOrder` và không ghi.

## Verification

**Commands:**
- `npx vitest run tests/frontend/lookupSourcesPopup.test.ts` -- xanh
- `npm run check:commands && npm run check:i18n && npm run check:tokens` -- xanh

**Manual checks (if no CLI):**
- Ice dùng thật một lượt (nợ `Chủ: Ice`): kéo trên macOS mượt, không bật/tắt nhầm, danh sách không bị cắt trong panel hẹp.
