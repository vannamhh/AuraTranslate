---
title: 'Mục lục đứng ở Chương đang đọc, bấm Chương là mở'
type: 'bugfix'
ticket: ''
created: '2026-11-02'
status: 'done'
baseline_revision: '47f9364281c20fcfb66a1213b8fd7fb64b74a61c'
route: 'oneshot'
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

**Problem:** Đang đọc Chương 2, mở Mục lục (`Mod+L`) thì dòng sáng là Chương 1, vì `openTableOfContents` gán con trỏ về 0. Chương "đang đọc" cũng không theo cuộn. Bấm chuột vào tên Chương không có tác dụng gì, vì dòng không gắn handler nào.

**Approach:** Lúc mở Mục lục, chụp câu đầu tiên đang thấy làm neo rồi đặt con trỏ vào Chương chứa câu được nhắm, nếu không có thì chứa neo, nếu không có nữa thì về 0. Mỗi dòng Mục lục là một nút: bấm vào là mở Chương đó (quyết định của Ice, 2026-11-02), đóng Mục lục và cuộn về đầu Chương. Bàn phím giữ nguyên: dùng `toc_next`/`toc_prev` và `toc_open`.

</frozen-after-approval>

## Implementation Notes

- Đi đường oneshot vì ước tính khoảng 100 dòng trên 2 tệp nguồn và 1 tệp test.
- `src/modes/readingState.ts`: `openTableOfContents` đặt `tocCursor` bằng chỉ số của `readingCurrentChapter.chapter_id` trong `tocChapters`, không tìm thấy thì về 0. Thêm `setReadingTocCursor(index)` theo khuôn `setReadingMarksCursor`. Trong `openCurrentTocChapter`, gọi `resetReading()` trước `reloadReading()` để `run` đi qua `null`, nhờ đó watcher `ReadingMode.vue:158` gọi `restoreReadingAnchor()` với neo `null`, tức `scrollTop = 0`. Đây cũng là cách `libraryChapters.ts:391` đang làm.
- `src/modes/ReadingMode.vue`: thêm một watcher `flush: 'sync'` trên `readingTocOpen`. Khi giá trị thành `true`, watcher gọi `rememberReadingViewportAnchor()`. Watcher phải chạy đồng bộ, trước lượt `await listChapters()`. `<li>` bọc một `<button>` có `@pointerdown`/`@focus` gọi `setReadingTocCursor(index)` và `@click` gọi `dispatch('reading.toc_open')`.
- happy-dom không có hình học, nên test giả `getBoundingClientRect` để kiểm neo theo cuộn. Test phải mount trong `<KeepAlive>`, vì `readingModeActive` chỉ thành `true` ở `onActivated`.
- Sau review: khi mở Mục lục, watcher sync bỏ câu được nhắm nếu câu đó nằm ngoài vùng nhìn (`dropOffscreenReadingAim`). Nếu không bỏ, một câu đang giữ tiêu điểm ở Chương 1 sẽ thắng neo ở Chương 2. `openCurrentTocChapter` dựng `tocBusy` để bấm đúp không mở hai lần.
- Đối chứng bằng phép gỡ thật: gỡ lần lượt watcher sync, con trỏ theo Chương, `resetReading()`, `@click`, `dropOffscreenReadingAim()` và `tocBusy = true`. Mỗi lần gỡ làm đỏ đúng ca của nó.

## Verification

**Commands:**
- `npx vitest run tests/frontend/readingToc.test.ts` -- expected: 6/6 xanh.
- `npx eslint src/modes tests/frontend/readingToc.test.ts` và `npx vue-tsc --noEmit -p tsconfig.json` -- expected: xanh.

**Manual checks (if no CLI):**
- Trong app thật: mở một lượt đọc có từ 2 Chương, cuộn tới Chương 2, nhấn `⌘L` thì dòng sáng là Chương 2. Bấm Chương 1 thì Mục lục đóng và trang đứng ở đầu Chương 1.

## Review Triage Log

- Lượt quick: 0 high · 2 medium · 2 low · 5 false/bác.
- medium · patch — Câu được nhắm (đang giữ tiêu điểm) ở Chương 1 vẫn thắng neo cuộn ở Chương 2. Đã thêm `dropOffscreenReadingAim` và ca test có đối chứng gỡ.
- medium · patch — Bấm đúp một dòng mở Chương hai lần. Đã dựng `tocBusy` quanh `openCurrentTocChapter` và ca test có đối chứng gỡ.
- low · patch — Watcher sync chen giữa doc-comment và watcher mà nó mô tả. Đã dời watcher lên trên.
- low · patch — Verification của plan trỏ `readingState.test.ts`, trong khi ca test nằm ở `readingToc.test.ts`. Đã sửa.
- false — Con trỏ tính sau `await listChapters()` nên có thể bị câu nhắm mới ghi đè. Thực tế lớp phủ che vùng đọc và `panel.focus()` làm câu mất tiêu điểm, nên trong lúc chờ IPC không có câu nhắm mới nào.
- bác (low) — Mục lục trống trong lúc `reloadReading` chạy. Cùng hành vi với `libraryChapters.ts:391`, và khi nạp lỗi thì trước đây `run` cũng thành `null`.
- bác (low) — Thiếu style hover. Đã có `cursor: pointer`, vòng tiêu điểm mặc định của trình duyệt giống `.pl-row`, `check:tokens` xanh.
- bác (low) — `@pointerdown`, `@focus` và `@click` trùng nhau. Đây là cùng khuôn với dòng Đánh dấu và không có hại.
- false — "Chưa đối chứng gỡ `resetReading()`". Đã gỡ thật: ca cuộn về đầu đỏ với `900 ≠ 0`.
