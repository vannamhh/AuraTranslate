# Bảng kê lớp phủ — 2026-11-17

Đếm tĩnh trên `<style>` của từng tệp ở master `528dec8a` (khai báo `overflow: auto|scroll`, `max-width` của khung). Chưa đo trong app; "lồng" là suy từ tên selector (khung + danh sách bên trong), cần xác nhận bằng DOM thật.

Cửa sổ: mặc định 1280×860, tối thiểu 800×600 (`tauri.conf.json`).

| Khung rộng | Lớp phủ |
|---|---|
| 560px | ProofreadMemory |
| 637px | Palette (bảng lệnh) |
| 721px | ChapterMerge · ChapterSplit · ConventionImport · GlossaryHarvest · GlossaryImport · LibraryImport · PromptImport · ReviewerImport · TmImport |
| 882px | TmManage |
| 917px | Settings · ConventionReplace |
| 959px | Alignment · PromptLibrary |
| 1099px | GlossaryManage · ConventionManage · SegmentHistory |
| 1120px | GlossaryQueue |
| 100% | Attribution · Export · ImportPreview · AiPromptInspector · BilingualImportPreview |
| ôm nội dung, `min-width` 322px, góc trên phải | ProofreadScope (bộ chọn phạm vi Proofreader) |

Tám giá trị chiều rộng khác nhau, cộng một nhóm tràn cửa sổ và một popup. Tổng 26 tệp `*Overlay.vue`. Mọi giá trị đều là bội của 7.

| Vùng cuộn trong CSS | Lớp phủ |
|---|---|
| Khung cuộn + danh sách bên trong cuộn (nghi lồng) | Alignment · ChapterSplit · ConventionReplace · GlossaryHarvest · PromptImport · PromptLibrary · TmManage (`.tm-mono`) · Attribution (`.attr-license-text`) |
| Ba vùng | Export (`.ex-config` · `.ex-preview` · `.ex-chapters`) |
| Hai cột cạnh nhau, không lồng | Palette (`.pal-preview` · `.pal-list`) |
| Ô nhập tự cuộn | LibraryImport (`.li-url-textarea`) |
| Một vùng | 14 lớp phủ còn lại; chỉ GlossaryQueue và ImportPreview có thanh dính |

Menu `Công cụ ▾` (`src/TitlebarToolsMenu.vue`): 16 mục phẳng, không đường phân cách (`.ui-menu-sep` có sẵn trong `controls.css` nhưng chưa dùng ở đây). Thứ tự hiện tại: Quản lý Glossary · Ngưỡng quét Glossary · Quản lý TM · Cài đặt · Chỗ đã bỏ qua · Quản lý quy ước chữ Việt · Thay cả Chương theo quy ước · Thêm Chương… · Nhập bản reviewer · Nối tay bản reviewer · Duyệt thu hoạch thuật ngữ · Mở bảng lệnh · Sắp xếp panel · Ẩn/hiện Đối chiếu · Ẩn/hiện Tra cứu · Ẩn/hiện Đề xuất AI.

Lệch với EXPERIENCE.md §Thanh tiêu đề và lối vào: bảng ở đó ghi "Quét chính tả" có trong `Công cụ ▾`, mã không có mục này; mã có "Thêm Chương…" và "Mở bảng lệnh", bảng không ghi.
