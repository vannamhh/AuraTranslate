---
title: 'Nút Xuất Chương ở Chế độ đọc'
type: 'feature'
ticket: ''
created: '2026-11-16'
status: 'built'
baseline_revision: '2e0b5fa46d8ca03e5d9017466cb870994f65b43c'
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

**Problem:** Đang đọc một Chương trong ReadingMode, muốn xuất nó thì phải quay về trang Tác phẩm ở Library, bấm `Xuất…`, rồi đổi phạm vi từ "cả Tác phẩm" sang đúng Chương đó.

**Approach:** Thêm một nút `Xuất Chương…` trên thanh công cụ của ReadingMode. Nút mở màn Xuất có sẵn (`ExportOverlay`, FR89) với phạm vi "một Chương" đã chọn sẵn là Chương đang đọc (`readingCurrentChapter`); người dùng vẫn chọn định dạng, thư mục và bấm Xuất như ở Library. Nút tắt khi chưa có Chương nào để đọc.

</frozen-after-approval>

## Implementation Notes

Oneshot vì chỉ thêm một lối vào cho màn Xuất đã có. Không chạm Rust.

- Ice chọn "mở màn Xuất với phạm vi Chương đang đọc", không chọn "xuất ngay một bấm".
- Handler nằm trong `src/readingExportCommandDeps.ts` riêng, không gộp vào `exportCommandDeps.ts`: gộp sẽ kéo `readingState` (và `editorPanelState`) vào mọi test đang mock `config/chapter` của màn Xuất.
- Phạm vi "một Chương" do lối Chế độ đọc đặt được trả lại khi đóng màn, để nút `Xuất…` ở Library vẫn mở theo phạm vi của riêng nó.
- Nút mang `data-export-open` và `@mousedown` trao tiêu điểm (WebKit không trao tiêu điểm cho nút khi bấm), nên Esc trả tiêu điểm về nút; thêm một dòng `R_POINTER_FOCUS` vào bảng Kiểm K của `check-commands.mjs`.

## Verification

**Commands:**
- `npx vitest run tests/frontend/readingExportChapter.test.ts tests/frontend/exportScope.test.ts tests/frontend/readingToolbar.test.ts` -- expected: xanh.
- `npm run check:i18n` · `npm run check:commands` -- expected: xanh.

**Manual checks (if no CLI):**
- Trong app thật: mở một Tác phẩm ở Chế độ đọc, cuộn tới Chương 3, bấm `Xuất Chương…` ⇒ màn Xuất mở với "Một Chương" = Chương 3; Esc trả tiêu điểm về nút.

## Review Triage Log

Lens quick: 4 phát hiện, 4 vá (medium 2, low 2).
- medium · patch — nút thiếu `@mousedown` trao tiêu điểm nên Esc trên WKWebView trả tiêu điểm về chỗ cũ, không về nút; thêm `focusOnPointerDown` như LibraryWorkPage.
- medium · patch — `openExportForChapter` để phạm vi "một Chương" dính sang lần mở `Xuất…` ở Library; `closeExport` nay trả phạm vi cũ, có ca test riêng.
- low · patch — ca "disabled" bấm vào nút đã tắt nên không chạm được guard của handler; nay dispatch thẳng lệnh.
- low · patch — lệnh Verification không chạy tệp test mới; đã sửa.
