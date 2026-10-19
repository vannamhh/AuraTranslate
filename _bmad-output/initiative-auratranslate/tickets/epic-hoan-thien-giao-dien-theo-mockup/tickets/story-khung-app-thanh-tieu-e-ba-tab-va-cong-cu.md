---
id: 1
type: story
title: "Khung app: thanh tiêu đề ba tab và Công cụ ▾"
parent: epic-hoan-thien-giao-dien-theo-mockup
after: []
hitl: false
risk: high
---

# Khung app: thanh tiêu đề ba tab và Công cụ ▾

## Description

Thanh tiêu đề chỉ còn ba tab Library · Workspace · Chế độ đọc bên trái. Bên phải là trạng thái Proofreader, công tắc bố cục Ⓑ-2/Ⓑ-1 và nút `Công cụ ▾` (D1, D10, D11). Mười nút lối vào hiện có rời thanh tiêu đề.

`Công cụ ▾` là danh sách thả xuống trong webview. Các mục và phím của chúng theo bảng ở EXPERIENCE.md §Thanh tiêu đề và lối vào; mỗi mục ghi kèm phím. Xuất nằm tạm trong `Công cụ ▾` cho tới khi story 4 dựng thanh hành động danh sách Chương.

Story này dựng trạng thái Proofreader "Chương N · đã quét N câu" trên thanh tiêu đề. Quản lý TM có phím mặc định, không va `Mod+M` (gộp segment). Mỗi lối vào giữ đường lui tiêu điểm `data-*-open` (UX-DR17), gắn vào nút mở mới. Story này đặt phím `⌘N` cho việc mở nhập tài liệu; nút Nhập tài liệu là việc của story 2. Dòng: a:19.

Story 2, 4, 10, 16 đặt chỗ theo ngữ cảnh cho các lối vào. Story 13 thêm mục Duyệt thu hoạch vào `Công cụ ▾`.

## Acceptance Criteria

Verify: Một spec e2e. Thanh tiêu đề chỉ có ba tab, trạng thái Proofreader, công tắc bố cục và `Công cụ ▾`. Mỗi mục trong `Công cụ ▾` (Quản lý Glossary, Quản lý TM, Cài đặt, Quét chính tả, Chỗ đã bỏ qua, Nhập bản reviewer, Nối tay, Xuất) ghi phím nếu có, mở được, và trả tiêu điểm về `Công cụ ▾` khi đóng. Phím mặc định của Quản lý TM mở Quản lý TM; `Mod+M` vẫn gộp segment. `⌘N` mở nhập tài liệu. Sau khi quét một Chương, trạng thái Proofreader ghi đúng số câu đã quét.

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-hoan-thien-giao-dien-theo-mockup/epic-hoan-thien-giao-dien-theo-mockup.md
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, §Thanh tiêu đề và lối vào
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/key-screen-workspace.html (công tắc bố cục), proofreader.html §3 (trạng thái Proofreader)
- decisions — _bmad-output/initiative-auratranslate/change-hoan-thien-giao-dien-theo-mockup/handoff-buoc-1-bang-lech.md, D1, D10, D11
- rows — _bmad-output/initiative-auratranslate/change-hoan-thien-giao-dien-theo-mockup/bang-lech-a.md, §Bảng lối vào ở thanh tiêu đề và dòng 19

## Notes

- Decision: `⌘N` chỉ là phím. Thanh tiêu đề giữ hình IA, không có nút `Nhập tài liệu ⌘N` mà `library-and-import.html` vẽ ở Library; nút thuộc story 2 (Ice, 2026-10-13).
- Decision: trạng thái Proofreader trên thanh tiêu đề do story này dựng, không phải story 10 (Ice, 2026-10-13).
- Open question: phím mặc định của Quản lý TM. Build liệt kê phím còn trống và đưa Ice chọn.
- Open question: `⌘N` gắn vào lệnh nào trong bốn `library.import_*`, hay vào một lệnh mới mở form nhập.
- Open question: mục Quét chính tả mở bộ chọn phạm vi (`ai.proofread.scope.open`, không có phím), còn `⌘P` mà IA ghi cạnh nó là quét ngay (`ai.proofread.run`). Ghi `⌘P` cạnh mục là ghi phím của một hành động khác.
- Năm tệp vitest và ba spec e2e đang mở lớp phủ qua nút thanh tiêu đề (`exportScope`, `reviewerImportFocus`, `tmManage`, `proofreadWiring`, `proofreadMemory`; `story-3-5-review`, `shortcuts-capture-mouse`, `shortcuts-focus`).
- Kiểm ngoài tiêu chí (risk high): lượt e2e nightly macOS chạy đủ mọi spec sau khi story này merge, vì mọi spec mở lớp phủ đều đi qua lối vào mà story này dời.
