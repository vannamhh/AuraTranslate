---
id: 2
type: story
title: "Library: dải trên, lưới bốn cột, form nhập, trạng thái rỗng"
parent: epic-hoan-thien-giao-dien-theo-mockup
after: [1]
hitl: false
risk: high
---

# Library: dải trên, lưới bốn cột, form nhập, trạng thái rỗng

## Description

Library mở đầu bằng một dải trên. Dải có ô tìm kiếm với biểu tượng ⌕, không nhãn và không nút "Tìm"; Enter chạy tìm như nút cũ. Kế đó là hai đoạn chế độ dấu `Chính xác dấu | Không dấu`, các bộ lọc, và nút `Nhập tài liệu`. Dưới dải là lưới Tác phẩm bốn cột rộng cả khung nội dung. Lưới xuống ba cột rồi hai cột khi khung hẹp lại.

Mỗi thẻ ghi ngôn ngữ nguồn trên bìa và dòng "lĩnh vực · sửa … trước". Cả hai lấy từ dữ liệu hàng Tác phẩm đã có.

Form nhập rời khỏi trang và thành một lớp phủ. Nút `Nhập tài liệu` và `⌘N` mở lớp phủ đó. Khi đóng, tiêu điểm về lại nút mở (UX-DR17). Lệnh `⌘N` của story 1 đổi từ cuộn tới form sang mở lớp phủ. Nội dung form và đường tới màn xem trước giữ nguyên; story 6 sửa màn xem trước.

Library đã tải mà không có Tác phẩm nào thì hiện trạng thái rỗng thay lưới, theo `empty-states.html`. Trạng thái đó có câu giải thích Tác phẩm, hai lối mời Nhập từ file và Dán văn bản, và phím `⌘N`. Hai lối mời mở lớp phủ nhập.

Các khối Thư mục gốc, Mục mồ côi, Trạng thái Tác phẩm đang mở và Chương nằm dưới lưới, rộng cả khung, theo thứ tự hiện tại. Story 3 và story 4 dời chúng đi.

Dòng: a:16, a:22, a:23, a:28, a:30, a:31, a:34, a:167, a:186.

## Acceptance Criteria

Verify: Một spec e2e. Ở 1440×790, lưới Library có bốn cột và rộng bằng khung nội dung trừ lề. Thu hẹp cửa sổ thì lưới xuống ba cột rồi hai cột. Mỗi thẻ ghi ngôn ngữ nguồn và dòng "lĩnh vực · sửa … trước". Ô tìm kiếm không có nút "Tìm"; Enter ra cùng kết quả như nút cũ. Hai đoạn chế độ dấu đổi cách khớp. Khi Library có Tác phẩm, trang không có form nhập. Nút `Nhập tài liệu` và `⌘N` mở lớp phủ nhập; đóng lớp phủ thì tiêu điểm về nút mở. Tạo Tác phẩm qua lớp phủ vẫn ra một thẻ trong lưới. Library rỗng hiện trạng thái rỗng thay lưới. Bộ lọc ra 0 Tác phẩm, hay danh sách chưa tải hoặc tải lỗi, thì không hiện trạng thái rỗng của Library lần đầu.

## References

- parent — _bmad-output/initiative-auratranslate/epic-hoan-thien-giao-dien-theo-mockup/epic-hoan-thien-giao-dien-theo-mockup.md
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/library-and-import.html, màn Library (dải trên, lưới, thẻ)
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/empty-states.html, Library lần đầu
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/narrow-layout.html, §Ngưỡng và thứ tự hy sinh (lưới Library 4 → 3 → 2)
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, §Information Architecture, §Trạng thái rỗng
- rows — _bmad-output/initiative-auratranslate/change-hoan-thien-giao-dien-theo-mockup/bang-lech-a.md, dòng 16, 22, 23, 28, 30, 31, 34, 167, 186

## Notes

- Decision: form nhập rời trang thành lớp phủ, mở bằng nút `Nhập tài liệu` trong dải trên và `⌘N`; Library có Tác phẩm thì trang không có form (Ice, 2026-10-13).
- Decision: bốn khối cũ nằm dưới lưới tới khi story 3 và story 4 dời chúng, như cách Xuất nằm tạm trong `Công cụ ▾` (Ice, 2026-10-13).
- Decision: nhãn chế độ dấu ở Notes của epic.
- Open question: ngưỡng 4 → 3 → 2 cột. `narrow-layout.html` chỉ ghi thứ tự, không ghi số. Build đề xuất ngưỡng theo bề ngang khung nội dung và đưa Ice chọn.
- Open question: "sửa … trước" đọc `updated_at` của hàng chỉ mục Library. Chưa kiểm hàng chỉ mục cập nhật lúc nào sau một lượt sửa câu. Nếu nó chỉ mới lúc quét lại thì thẻ ghi thời điểm cũ.
- Helper e2e tạo Tác phẩm qua form (`createWorkThroughForm`) có 20 trên 31 spec e2e dùng. Ba tệp vitest dựa vào form luôn có trong DOM: `libraryWorks`, `titlebarToolsMenu`, `importPreviewDestination`.
  🔵 2026-10-13: sai. Chỉ `story-5-4` và `story-5-5` gọi `createWorkThroughForm`; 17 spec tạo Tác phẩm qua IPC, không chạm form.
- Kiểm ngoài tiêu chí (risk high): lượt e2e nightly macOS chạy đủ mọi spec sau khi story này merge, vì hai phần ba spec tạo Tác phẩm qua form mà story này dời.
  🔵 2026-10-13: lý do đúng là các spec `story-5-x` đọc `[data-library-*]`, và `story-5-9`, `story-5-10` bấm nút "Tìm" mà story này gỡ.
