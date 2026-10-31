---
title: 'Tầng khối của nhập URL dùng được bằng chuột'
type: 'feature'
ticket: '6'
created: '2026-10-31'
status: 'built'
baseline_revision: '216e3de8267b308e77dd1aeb6730ef763a877bce'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-san-chuot/ux-san-chuot/EXPERIENCE.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 8 id của tầng khối xem trước nhập URL chỉ có phím: khối không nhắm được bằng chuột, không có nút giữ/loại, đánh dấu vùng, đổi Chương, tới luật làm sạch.

**Approach:** Thêm `@mousedown` nhắm khối, cụm nút trên khối đang nhắm, hai nút ‹ › và nút Tới luật làm sạch theo `EXPERIENCE.md` §Bảng chỗ đặt Story 14.6; `block_next`/`block_prev` khai `pointer: aim`, 6 id còn lại thành điểm chuột, rút 8 id khỏi `POINTER_DEBT`.

## Boundaries & Constraints

**Decision (repo chứng minh, không cần ký):** Rust nhận dải đảo chiều (`block_overrides_for_range` tự hoán đầu cuối), nên `Cuối vùng giữ` hiện cả khi khối đang nhắm đứng trên đầu vùng.

**Decision:** cụm nút nằm cuối hàng nhãn trạng thái, sát phải, trong `.ip-block-head` bọc nhãn; mọi hàng nhãn giữ chiều cao một nút. Nhắm khối không làm chữ chảy lại và không dời khối dưới con trỏ.

**Decision:** ‹ › tắt ở hai đầu và khi `importPreviewChapterDetailLoading`: `moveImportPreviewChapterCursor` im lặng ở các trạng thái đó (R9). Nút trong cụm tắt khi `importPreviewBlockToggling`, `importPreviewBlockRangeConfirming` hoặc `importPreviewConfirming`, vì hai hàm ghi bỏ qua im lặng.

**Decision:** cụm nút và ‹ › mang `@mousedown.prevent`: `onScrimKeydown` bỏ qua phím khi tiêu điểm ở `<button>`, và nút biến mất sau khi bấm làm tiêu điểm rơi về `body`. Cùng lý do §Bấm điểm cắt song ngữ. `Tới luật làm sạch` không cần, vì `watch` chuyển tiêu điểm sang tầng 3.

**Decision:** `tier2_range_start_marker` nói được cả đường chuột, vì `]` không còn là đường duy nhất. `tier2_range_missing_start` giữ nguyên: chỉ đường phím tới được nó.

**Decision (Ice chọn 2026-10-31):** ở Chương k > 0 ba nút vẫn hiện nhưng tắt, `title` nói chỉ sửa được ở Chương đầu, theo phiếu quyết #16. Rust không đổi, đường phím vẫn nhận lỗi như cũ. KF-7 nhận dòng 🔵: sửa khối diễn ra ở Chương đầu.

**Decision (Ice chọn 2026-10-31, mở lại phạm vi sau khi thi công):** vá một điểm ở `webimport::fetch`, chạy phần `reqwest::blocking` trên luồng riêng, giữ nguyên chữ ký, kèm ca Rust tái hiện qua `tauri::async_runtime::spawn`. Trên bản debug, ba đường `start_url_import`, `reload_url_import_item`, tải ảnh lúc xác nhận panic ở tokio worker nên promise không trả về. Đây là ngoại lệ duy nhất của vế Rust ở **Never**.

**Always:** `@click` là đúng một `dispatch` literal. `Loại khối này`/`Giữ khối này` là một nút, nhãn đổi theo `block.kept`. Nút `title` qua `commandTitle`, dáng theo `DESIGN.md` §Components. Hàm nhắm mới kiểm phạm vi và dọn `blockActionError` như `nextImportPreviewBlock`. `Tới luật làm sạch` chỉ hiện khi `importPreviewSelectedBlocks !== null`, cùng điều kiện phím `R`.

**Never:** Đổi `onTier2Keydown`, `onChapterCursorKeydown`, ý nghĩa phím, Rust hoặc IPC. Sửa `POINTER_DEBT_BASELINE`. Ghi `tickets.toml`. `box-shadow` trên khối (Kiểm F). Chép `focusOnPointerDown`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Nhắm | Bấm khối 3 khi đang nhắm khối 0 | Khối 3 nhắm, đúng một cụm nút, `J` tới khối 4, `K` về khối 2; danh sách không nhảy | Không |
| Nhắm cùng khối | Bấm khối đang nhắm khi tiêu điểm ở `panel` | `J` chạy tiếp từ khối đó | Không |
| Bôi chữ | Kéo trên chữ của khối | Nhắm khối, vùng bôi giữ nguyên | Không |
| Giữ/loại | Bấm nút trong cụm | Nhãn và bộ đếm đổi; `J` chạy tiếp | Lỗi hiện ở `importPreviewBlockActionError` |
| Vùng giữ | `Đầu vùng giữ` ở khối 4, nhắm khối 1, `Cuối vùng giữ` | Gọi `(4, 1, total, …)`, nút biến mất, `J` chạy tiếp | Lỗi hiện như trên |
| Cùng khối | Nhắm đúng khối đầu vùng | Không có `Cuối vùng giữ` | Không |
| Chương k > 0 | Con trỏ ở Chương 2, khối đang nhắm | Cụm nút hiện, cả ba tắt, `title` nói lý do, 0 IPC | Không |
| Biên Chương | Con trỏ ở đầu hoặc cuối | Nút phía đó tắt | Không |
| Một Chương | `chapterCursorTotal <= 1` | Không có ‹ › | Không |
| Không phải URL | Đường tệp/dán | Không cụm nút, không ‹ ›, không nút tới luật | Không |

</frozen-after-approval>

## Code Map

- `src/ImportPreviewOverlay.vue` -- `<li class="ip-block">`:1240 nhận `@mousedown`; hàng tiêu đề tầng 2:1180; `.ip-chapter-cursor-note`:1190 đặt ‹ › trước `<p>`, ngoài `role="status"`; `watch(importPreviewBlockFocusedIndex)`:773; `.ip-block`:2006, `.ip-block-tag`:2052.
- ⚠️ `watch`:773 gọi `focus()` trong `nextTick`, tức trước mặc định `mousedown` với cú bấm tin cậy; `<ol>` chưa có tiêu điểm thì trình duyệt có thể cuộn. Đổi thành `focus({ preventScroll: true })`, `scrollIntoView` ngay trên đã cuộn đủ; đo trên bàn đo.
- `src/importPreviewState.ts` -- thêm `setImportPreviewBlockFocus(index)` cạnh `nextImportPreviewBlock`:1302. `importPreviewBlockToggling`:460 đang chưa ai đọc. `confirmImportPreviewBlockRange`:1370, `moveImportPreviewChapterCursor`:1551.
- `src/commands/index.ts` -- khai `pointer: aim`, `at: 'src/ImportPreviewOverlay.vue'` ở :1883 và :1894, `reason` như `glossary.harvest.next`:4517.
- `src/GlossaryHarvestOverlay.vue`:163 -- khuôn nhắm. `src/StatusBar.vue`:578 -- khuôn nút ‹ ›, biểu tượng và `commandTitle`.
- `src/i18n/vi.json` -- khoá `command.*`:798, `mode.library.preview.tier2_*`:968.
- `scripts/check-commands.mjs` -- `POINTER_DEBT`:2902 chỉ xoá; `CLICK_FLOOR`:251, `POINTER_FLOOR`:2979 theo `ceil(0.85 × live)` cổng in ra. ⚠️ Kiểm M `aim` chỉ đòi tệp có `@mousedown`, nên `@mousedown.prevent` của cụm nút làm nó xanh dù gỡ handler nhắm: ca Vitest mới là chốt thật.
- `src-tauri/src/core/webimport/fetcher.rs` -- `fetch`:230, điểm vá duy nhất của Rust; ca ở `src-tauri/tests/webimport_contract.rs`.
- `tests/frontend/importPreviewBlocks.test.ts` -- bộ command thật:362, khuôn mount; `e2e/specs/story-12-6-import-preview.e2e.mjs`, `e2e/support/pointer.mjs`.
- `_bmad-output/implementation-artifacts/14-5-ban-do/chuot-that.e2e.mjs` -- khuôn bàn đo `cliclick`, chốt frontmost.

## Tasks & Acceptance

**Execution:**
- [x] `src/importPreviewState.ts`, `src/commands/index.ts` -- hàm nhắm, hai khai `pointer`.
- [x] `src/ImportPreviewOverlay.vue`, `src/i18n/vi.json` -- `@mousedown` nhắm, `.ip-block-head`, cụm nút, ‹ ›, nút tới luật, `preventScroll`, CSS, chuỗi.
- [x] `scripts/check-commands.mjs` -- gỡ 8 id, nâng sàn theo cổng, xếp handler mới vào `HANDLER_TABLE`.
- [x] `src-tauri/src/core/webimport/fetcher.rs`, `src-tauri/tests/webimport_contract.rs` -- chạy `reqwest::blocking` trên luồng riêng, ca tái hiện qua `async_runtime::spawn`.
- [x] `tests/frontend/importPreviewBlocks.test.ts` -- ca (i) cho sáu nút qua bộ command thật; ca `mousedown` thật cho `block_next` và `block_prev`; mọi hàng I/O Matrix kiểm được bằng happy-dom.
- [x] `e2e/specs/import-preview-mouse.e2e.mjs` (mới) -- máy chủ `node:http` trên `127.0.0.1` phục vụ hai trang, đi đường URL thật, bấm loại một khối, đặt vùng, xác nhận, đọc segment Tác phẩm đã nhập. `src-tauri/tests/webimport_contract.rs`:51 đã tải được `http://127.0.0.1`; nếu đường IPC trong app vẫn chặn thì trả lời giả ở `window.fetch` như `review-mouse.e2e.mjs` và ghi rõ phần kết quả nhập chưa đo.
- [x] `_bmad-output/implementation-artifacts/14-6-ban-do/` -- bàn đo `cliclick`: nhắm khối trong danh sách đã cuộn, `J` sau khi bấm từng nút, bôi chữ.
- [x] `deferred-work.md` -- đóng mục 8 id; thêm lượt dùng thật vào `Chủ: Epic 14`, focus nút trên WebView2 vào `B7`.
- [x] `EXPERIENCE.md` -- dòng 🔵 ở §State Patterns (ba nút và ‹ › tắt khi nào) và KF-7 (sửa khối ở Chương đầu).

**Acceptance Criteria:**
- Given cây sau story, when chạy `npm run check:commands`, then Kiểm M xanh với 283 điểm chuột, 31 khai `pointer`, 16 nợ.
- Given app thật, when làm đúng phần Verify của ticket, then kết quả nhập khớp khối đã giữ/loại và vùng đã đặt.
- Given ca (i) của một id, when gỡ `@click` khỏi nút đó, then chính ca đó đỏ.
- Given ca `mousedown`, when gỡ handler nhắm khỏi `<li>`, then ca đỏ vì khối nhắm không đổi.

## Implementation Notes

- `start_url_import` treo trên bản debug vì `reqwest::blocking` dựng rồi huỷ shell runtime dưới `debug_assertions`, panic ở tokio worker, promise không trả về; phép đo 2026-09-06 ở `core::webimport` chỉ đo `spawn_blocking`. `fetch` nay chạy trên luồng riêng; gỡ phép vá thì ca Rust đỏ đúng panic đó. Bản release chưa đo.
- Bàn đo chuột thật bắt hai lỗi Vitest và e2e không thấy: › kéo tiêu điểm và cuộn xuống danh sách Chương; `<ol>` bị huỷ lúc Chương nạp làm tiêu điểm về `body` và `J`/`K` chết. Đường chuột nay giữ tầm nhìn và đặt tiêu điểm ở `panel`; đường `⌥→` không đổi.
- `min-height` hàng nhãn từng là `* 7` thành 49px vì `--space-unit` là 7px, không phải 4px; nay `* 4` là 28px. Cụm nút từng dùng dáng Chìm, nay về Nút Phụ theo `DESIGN.md`.
- `cliclick c:` nhả ngay nên `mouseup` không tới trang: 0/12 `click`, giữ 90 ms thì 12/12, xen kẽ trong một lượt, gồm cả checkbox có sẵn. Tap-to-click trên trackpad bàn đo chưa đo; 🔵 2026-10-31: Ice đã dùng thật và bỏ mục nợ.
- Hai lượt đo mất cú bấm đầu bị nhiễu (checkbox lấy tiêu điểm nên `J` không dời khối), số đã bỏ.
- Kiểm M `aim` chỉ đòi tệp có `@mousedown`, nên ca Vitest là chốt thật của đường nhắm: gỡ handler thì 7 ca đỏ. Mười hai phép gỡ (sáu `@click`, handler nhắm, hai `.prevent`, `preventScroll`, giữ tầm nhìn, tiêu điểm `panel`) đều làm đúng ca đỏ.
- Chương k > 0 của HTML một dòng dài hơn 4 KiB báo lỗi là nợ có sẵn của Rust, `Chủ: Ice`; ngoài ngoại lệ Rust của story nên không sửa.
- Kiểm K buộc xếp `setImportPreviewBlockFocus` và `holdViewForChapterMove` vào `HANDLER_TABLE`; cổng buộc nâng `DISPATCH_FLOOR` 360 lên 385.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick): high 0 · medium 0 · low 2 · false 1 · maybe-false 0.

- false: `POINTER_FLOOR` và `CLICK_FLOOR` không nâng; `judgeFloor` chỉ đòi nâng khi sàn dưới 80% số thật, 236×5 = 1180 ≥ 283×4 = 1132, cổng xanh và không in yêu cầu, đúng task "theo cổng in ra".
- low, patch: ba khai báo chèn giữa doc-comment và `watch(importPreviewChapterCursor)` làm comment tả sai khai báo; đã chuyển lên trên.
- low, patch: ca Rust mới thiếu `SERIAL_LOCK` mà mọi ca khác trong tệp đều lấy; đã thêm.

## Design Notes

KF-7 cần `Cuối vùng giữ` ở khối thứ hai, nên người dùng bấm khối cuối trước rồi mới bấm nút; cụm nút theo khối nhắm nên hai cú bấm đó không đổi. Dải đảo chiều không cần xử lý ở webview.

## Verification

**Commands:**
- `npm run check:commands` -- exit 0, in 283/31/16.
- `npm run check:i18n`, `npm run check:tokens`, `npm run check:debt-owner` -- exit 0.
- `npm run test:story 14-6` -- exit 0.
- `npx vue-tsc --noEmit`, `npm run check:lint` -- exit 0.
- `cargo test --manifest-path src-tauri/Cargo.toml --locked --test webimport_contract` -- exit 0.
- `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, rồi `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/import-preview-mouse.e2e.mjs` -- xanh, chạy ngoài sandbox.
- Chạy đủ bộ một lần: story chạm `scripts/check-*` (AGENTS.md §Tests).

**Manual checks (if no CLI):**
- 🔵 2026-10-31: Ice đã dùng thật trên app thật: nhắm, loại, đặt vùng, đổi Chương, kết quả nhập khớp.
