# Bảng lệch nhóm A — khung app, Library, nhập (mockup ↔ app thật)

Bước 1, phần của một agent trong ba. Chỉ đọc; không chạy app. Mockup đọc bằng bóc thẻ + grep CSS; mã đọc theo dòng.

**Loại:** 1 khớp · 2 lệch cần sửa · 3 lệch có chủ đích (kèm nguồn) · 4 app có, mockup không vẽ · 5 mockup có, app chưa có.
**Bằng chứng:** `ảnh L` = ảnh Library Ice chụp (dpr=2, cửa sổ rộng, chỉ thấy phần đầu trang đến hàng thẻ đầu); `ảnh W` = ảnh Workspace (chỉ dùng cho thanh tiêu đề); `mã` = `tệp:dòng` trong `src/`. Mockup viết tắt `lib-imp` = `library-and-import.html`, `lib-srch` = `library-search-results.html`. Số dòng của `LibraryMode.vue` là dòng tuyệt đối trong tệp (template bắt đầu ở 336, `<style>` ở 1504).
Số dòng `LibraryMode.vue` suy từ vị trí tương đối trong template, có thể lệch vài dòng.
**Không có chỗ nào trong bảng xếp loại 3 mà không có nguồn.** Chỗ cần ảnh mà không có thì ghi "cần ảnh", không xếp loại (liệt kê ở cuối).

---

## 1. `library-and-import.html` — Library (trang chính)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Bố cục trang | Một dải trên (tìm · chế độ dấu · bộ lọc) rồi lưới Tác phẩm 4 cột `repeat(4,1fr)` gap 20 chiếm cả khung (`lib-imp:30,41`) | Một cột hẹp `max-width:420px` bên trái, xếp dọc: Thư mục gốc → Mục mồ côi → Tìm kiếm → Tác phẩm → (Trạng thái Tác phẩm đang mở, Chương) → form nhập; hai phần ba bên phải để trống | 2 | ảnh L · mã `LibraryMode.vue:1731,2093` (`.root-block`, `.search-block` 420px), `.works-block` cùng 420px |
| Thanh tiêu đề: ba tab chế độ | `Library · Workspace · Chế độ đọc` (`lib-imp:102`) | Có ba tab, cùng với 10 nút lối vào (xem bảng §10) | 1 | ảnh L · mã `App.vue:241-257` |
| Thanh tiêu đề: 10 nút lối vào cạnh tab | Không có; chỉ một ô phải theo ngữ cảnh | 10 nút `.titlebar-act` thành dải trông như 13 tab, chạm mép phải cửa sổ | 3 | D1 · ảnh L, ảnh W · mã `App.vue:272-370` |
| Thanh tiêu đề: ô phải của Library | `Nhập tài liệu ⌘N` (`lib-imp:103`) | Không có nút và không có phím mặc định (`library.import_*` không khai `keys`); lối nhập là form cuối trang | 2 | mã `commands/index.ts:1400-1430` (không `keys`), `LibraryMode.vue:1273` |
| Khối "Thư mục gốc" (Đổi thư mục gốc… · Quét lại · dòng trạng thái) | Library không vẽ; `data-integrity.html` vẽ "Quét lại thư mục" thành bề mặt riêng (xem §7) | Khối đầu trang Library, đứng trên cả tìm kiếm | 4 | ảnh L · mã `LibraryMode.vue:345-420` |
| Khối "Mục mồ côi" | Library không vẽ (xem §7) | Khối thứ hai của trang Library, hiện cả khi "Không có mục mồ côi nào." | 4 | ảnh L · mã `LibraryMode.vue:436` |
| Ô tìm kiếm | Một ô có biểu tượng ⌕ trong dải trên, không nhãn, không nút | Nhãn "Tìm trong toàn bộ Library" + ô + nút "Tìm" riêng, xếp dọc | 2 | ảnh L · mã `LibraryMode.vue:500-525` |
| Chế độ dấu | Hai đoạn `Chính xác dấu \| Không dấu` (`lib-imp:110`); `lib-srch` gọi là `Chính xác dấu \| Khoan dung không dấu` | Nhãn "Chế độ dấu" + hai nút `Phân biệt dấu` · `Khoan dung dấu` (`aria-pressed`) | 2 | ảnh L · mã `i18n/vi.json:859-861` |
| Bộ lọc trạng thái | Một chip đang chọn ("Đang dịch") trong dải | Bốn nút `Chưa bắt đầu · Đang dịch · Tạm ngưng · Đã xong` + `Bỏ lọc` | 1 | ảnh L · mã `LibraryMode.vue:700-760` |
| Bộ lọc Lĩnh vực · Ngôn ngữ nguồn · Sắp xếp | Ba bộ chọn một hàng, cùng thứ tự (`lib-imp:113`) | Ba `<select>` có nhãn, cùng thứ tự, xếp ngang trong khối 420px | 1 | ảnh L · mã `LibraryMode.vue:769-810` |
| Nút "Tải danh sách" và đếm "47 / 47 Tác phẩm" | Không vẽ | Có; lưới chỉ hiện sau khi tải, đếm nằm dưới bộ lọc | 4 | ảnh L · mã `LibraryMode.vue:752`, `i18n/vi.json:793` |
| Con trỏ lưới `‹ 1/47 ›` | Không vẽ | Hai nút + vị trí "k / n" trên mọi lưới/danh sách | 4 | ảnh L · mã `LibraryMode.vue:831-850` |
| Lưới thẻ Tác phẩm | 4 cột, thẻ rộng ~150px | `grid-template-columns: repeat(auto-fill, minmax(110px,1fr))` trong khối 420px, ra 3 cột | 2 | ảnh L · mã `LibraryMode.vue:1875-1881` |
| Bìa | Ô cao 112px, nền gradient hoặc ảnh, chữ `Trung`/`Anh`, nhãn "chưa có bìa" (`lib-imp:43-49`) | Ô vuông, chữ cái đầu của tên trên nền token, không ảnh | 3 | `tickets/epic-library/tickets/story-luoi-tac-pham-loc-va-sap-xep-plan.md` §Design Notes (không thêm cột `cover`: 0 đường sản phẩm đặt ảnh bìa) · ảnh L · mã `LibraryMode.vue:1915-1930` |
| Thẻ: nhãn ngôn ngữ nguồn trên bìa | Có (`Trung`/`Anh`) | Thẻ không hiện ngôn ngữ nguồn, dù bộ lọc ngôn ngữ có | 2 | mã `LibraryMode.vue:853-935` |
| Thẻ: dòng "Tiên hiệp · sửa 2 giờ trước" | Có | Không có thể loại, không có thời điểm sửa trên thẻ | 2 | mã `LibraryMode.vue:853-935` |
| Thẻ: tiến độ + trạng thái | Thanh 3px + "128 / 2000 chương" + huy hiệu trạng thái | Chữ "{done} / {total} Chương đã xong" + thanh + chữ trạng thái; ghi đè thủ công có nhãn "(ghi đè thủ công)" | 1 | mã `LibraryMode.vue:870-930`, `i18n/vi.json:760,801` |
| Thẻ: nút "Mở Tác phẩm" | Không vẽ nút; bấm thẻ | Nút hiện ở ô đang chọn (con trỏ lưới) | 4 | mã `LibraryMode.vue:930-940` |
| Form nhập (Đưa vào · tên · ngôn ngữ · thể loại · dán · tệp · URL · song ngữ) | Không có trên Library; nhập đi qua nút titlebar rồi màn xem trước | Form luôn hiện ở cuối trang, kể cả khi Library đã có Tác phẩm | 2 | mã `LibraryMode.vue:1259-1480` |
| Vùng kéo-thả tệp | Không vẽ | Có, chỉ điền ô đường dẫn | 4 | mã `LibraryMode.vue:1390` |

## 2. `library-and-import.html` — màn xem trước kết quả tách Chương

Nguồn app: `ImportPreviewOverlay.vue` (lớp phủ `ip-panel`, `max-width:760px`).

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Dạng bề mặt | Cả màn, tab Library/Workspace/Chế độ đọc vẫn hiện, tên tệp + dung lượng ở ô phải titlebar | Lớp phủ `role="dialog" aria-modal` có scrim, nút đóng riêng | 2 | mã `ImportPreviewOverlay.vue:857,1760` |
| Bố cục trong màn | Hai cột `.two`: trái cấu hình 300px, phải xem trước (`lib-imp:67`) | Các tầng xếp dọc một cột: Bảng mã → Chuẩn hoá → Ranh giới → Luật làm sạch → Tách Chương | 2 | mã `ImportPreviewOverlay.vue:1069-1453` |
| Mẫu phân tách | Có | Có ("Mẫu phân tách Chương", chuỗi hoặc regex) | 1 | mã `i18n/vi.json:705` |
| "Chương nhận ra N" | Có | Có | 1 | mã `i18n/vi.json:707` |
| Tên Tác phẩm · Ngôn ngữ nguồn trong màn | Có (ngôn ngữ ghi "cố định sau khi tạo") | Không hiện lại; nằm ở form Library trước đó | 2 | mã `ImportPreviewOverlay.vue` (không có khoá `name`/`source_lang`) |
| "Trạng thái ban đầu: Chưa bắt đầu" | Có | Không hiện | 2 | mã `i18n/vi.json:652-742` (không khoá) |
| "Ứng viên Glossary sẽ quét ~340" | Có | Không có ước lượng trước nhập | 5 | mã `i18n/vi.json:652-742` |
| Cảnh báo "14 dòng đáng ngờ" + ⚠ từng dòng + "Chỉ hiện 14 dòng đáng ngờ" | Có, nguyên nhân: ngắn bất thường / nằm giữa Chương khác | Có chip "{n} Chương cần xem" · "{n} Chương sạch", huy hiệu "Cần xem", nguyên nhân khác (xoá quá nhiều, nối dòng cao, không đo được) | 1 | mã `i18n/vi.json:714-724` · nguyên nhân do `change-bo-loc-can-xem-dung-tren-nang-luc-chua-dung`; chưa so từng nguyên nhân |
| Danh sách xem trước (ba đầu · ba cuối · ⋯ · chữ) | Có | Có, thêm "Sắp xếp theo độ dài", "{n} ký tự", con trỏ Chương | 4 | mã `i18n/vi.json:709-711`; hình dạng rút gọn: cần ảnh |
| Nút "Xác nhận và nhập 2 000 Chương" (có số) | Có số Chương trong nhãn | "Xác nhận nhập" không số | 2 | mã `ImportPreviewOverlay.vue:1731` · `i18n/vi.json:652` |
| "Huỷ" + "Chưa có gì được ghi xuống đĩa" | Có | Có ("Chưa byte nào được ghi xuống đĩa…") và nút Huỷ | 1 | mã `ImportPreviewOverlay.vue:1717`, `i18n/vi.json:726` |

## 3. `web-import.html` — nhập từ URL

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Bước dán link | Bề mặt riêng "Nhập từ website", ô dán có đánh số dòng, `⌘V`, `⌘↵ tải và xem trước` | Một `<textarea>` trong form Library, không số dòng, không phím mặc định | 2 | mã `LibraryMode.vue:1443-1460` |
| "50 link · sẽ tạo 50 Chương" | Hai con số cạnh nhau + "· 1 domain" | "{n} link đã dán · sẽ tạo {n} Chương", không số domain | 2 | mã `i18n/vi.json:643` |
| Câu cam kết "Chỉ tải đúng 50 link này. Không tìm thêm…" | Có | Không có câu này trên bề mặt dán | 2 | mã `LibraryMode.vue:1443-1470` (không khoá) |
| Nút "Tải 50 link" | Có số | "Tải danh sách URL" | 2 | mã `i18n/vi.json:644` |
| Đưa vào: Tác phẩm mới / đã có | Hai radio, cùng mô tả | Cùng hai radio, cùng mô tả | 1 | mã `i18n/vi.json:626-630` |
| Xem trước hợp nhất: dạng bề mặt | Cả màn, một lần xác nhận "Nhập 50 Chương ⌥⌘↵" | Lớp phủ (xem §2) | 2 | mã `ImportPreviewOverlay.vue:857` |
| Tầng 1 Bảng mã | Bảng năm bảng mã, chip "tự đoán · độ tin cậy thấp" | Tầng "Bảng mã" có chip `data-confidence` và ứng viên không ra chữ | 1 | mã `ImportPreviewOverlay.vue:1069-1075`, `i18n/vi.json:661-664` |
| Tầng "Xuống dòng và khoảng trắng đã chuẩn hoá" | Không vẽ; EXPERIENCE nói FR125 "không có tầng riêng", chạy ngầm | Có tầng riêng với số chỗ nối/dòng trống đã bỏ | 2 | mã `ImportPreviewOverlay.vue:1123`, `i18n/vi.json:665-671` · EXPERIENCE.md §Đường nhập (dòng 300) |
| Tầng 2 Ranh giới nội dung (khối giữ/loại, `J K`, `Space`, `[ ]`) | Có, xếp 4 tầng: Bảng mã → Ranh giới → Luật → Xuất xứ | Có, đủ `j`/`k`/`Space`, dải `[` `]`; chỉ cho nguồn URL | 1 | mã `ImportPreviewOverlay.vue:1172`, `:687-700`, `i18n/vi.json:672-684` |
| Tầng 3 Luật làm sạch (Toàn cục/Tác phẩm, số chỗ khớp) | Có, kèm "thêm luật mới" | Có, hai tầng Toàn cục/Tác phẩm | 1 | mã `ImportPreviewOverlay.vue:1294-1450`, `i18n/vi.json:683,689-690` |
| Tầng 4 Xuất xứ tài liệu (tác giả · báo · URL · ngày) | Tầng thứ 4, ở cuối | Cùng 4 trường (component `ChapterOrigin`) nhưng đặt ở ĐẦU tầng Ranh giới, trên Chương con trỏ | 2 | mã `ImportPreviewOverlay.vue:1205-1221` |
| Bộ lọc "7 Chương cần xem / 43 sạch" | Có, `⌥W` | Có chip và lệnh `chapter_filter_toggle` | 1 | mã `i18n/vi.json:714-715`, `commands/index.ts` (`import.preview.chapter_filter_toggle`) |
| Danh sách link kèm "Tải lại · Bỏ", khoá xác nhận khi còn mục hỏng | Không vẽ | Có (`url_list_title`, `url_item_reload`, `url_list_locked`) | 4 | mã `ImportPreviewOverlay.vue:918-996`, `i18n/vi.json:728-735` |
| Tiến độ tải trang/ảnh + huỷ tải ảnh | Không vẽ | Có | 4 | mã `i18n/vi.json:746-749` |
| "Đã gọi 2 domain · xem" ở chân màn | Có | Có, mở `settings.privacy.open` | 1 | mã `ImportPreviewOverlay.vue:1706-1709` |
| Nhật ký domain (Cài đặt › Quyền riêng tư) | Bảng 5 cột: thời điểm · domain · tầng · vì sao được phép · kết quả | Bảng 6 cột (tách "kết quả" thành outcome + result) trong mục Quyền riêng tư | 1 | mã `SettingsOverlay.vue:505-513` |
| Chế độ tối của màn xem trước | Có | Chưa xem | — | cần ảnh |

## 4. `bilingual-import.html` — nhập song ngữ

Nguồn app: `BilingualImportPreviewOverlay.vue` (`bip-panel`).

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Lối vào | Từ "Nhập tài liệu", file .docx 2 cột | Ô "Hoặc nhập đường dẫn tệp song ngữ (.csv/.tsv/.docx)" + nút "Tạo Tác phẩm từ tệp song ngữ" ở form Library | 4 | mã `LibraryMode.vue:1421-1440`, `i18n/vi.json:646-647` |
| Dạng bề mặt | Cả màn | Lớp phủ `role="dialog" aria-modal` | 2 | mã `BilingualImportPreviewOverlay.vue:360` |
| Gán cột (Cột A → nguồn, Cột B → bản dịch, mẫu nội dung, "⇄ Đổi vai") | Có | Hai bộ chọn "Cột {n}: {mẫu}" + lệnh đổi vai | 1 | mã `i18n/vi.json:753-756`, `commands/index.ts` (`bilingual_swap_columns`) |
| Chọn bảng khi .docx nhiều bảng · "Hàng 1 là tiêu đề" · tầng bảng mã | Không vẽ | Có cả ba | 4 | mã `BilingualImportPreviewOverlay.vue:389,423`, `i18n/vi.json:757-762` |
| Mẫu nhận diện đầu Chương | Có | Có, trong tầng Số đếm | 1 | mã `BilingualImportPreviewOverlay.vue:509-511` |
| Tên Tác phẩm · Ngôn ngữ nguồn | Có trong màn | Ở form Library | 2 | mã `LibraryMode.vue:1345-1360` |
| Số đếm: hàng · Chương · cặp khớp thẳng · cặp cần nối tay | Bốn số | Ba số: hàng, Chương, "câu đã cặp"; số lệch nằm ở tiêu đề "Chỗ lệch i / N" | 1 | mã `i18n/vi.json:763-766` |
| Bảng xem trước hàng (hàng · Cột A · Cột B, ⚠ ở hàng lệch) | Có | Chưa xem hình dạng | — | cần ảnh |
| Nút "Xem 47 chỗ lệch rồi nhập" · "Nhập luôn, nối sau" | Hai lối; "nhập luôn" là lối thoát hợp lệ | Xác nhận bị khoá khi còn hàng lệch ("khớp lại các hàng đó ngay trong màn xem trước"); không có lối nhập luôn | 2 | mã `i18n/vi.json:36,772` |
| Nối câu lệch: tiêu đề "Chỗ lệch i / N · hàng · Chương" | Có | Có | 1 | mã `i18n/vi.json:767` |
| Nối câu lệch: hai cột nguồn/đích, "Máy đề xuất", Đúng ↵, tách bản dịch, nhận tất cả | Nút `Đúng ↵`, `Tách bản dịch ra 2`, `Bỏ qua hàng này`, `Nhận tất cả đề xuất` | Cùng chức năng qua caret `‖` cắt + `A` áp mọi đề xuất + `S` bỏ qua (chỉ khi một phía trống) | 1 | mã `i18n/vi.json:768-771`, `commands/index.ts` (`bilingual_accept_all_proposals`, `_toggle_cut`, `_skip_row`) |
| "Esc để sau — mở lại đúng chỗ này" | Có | Chưa thấy trong mã | — | cần ảnh |

## 5. `chapter-list.html` — danh sách Chương

Nguồn app: khối "Chương" trong `LibraryMode.vue` (`chapters-block`, `chapters-list`, `chapter-reorg`).

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Bề mặt | Trang riêng giữa Library và Workspace, tiêu đề Tác phẩm + `⌘1` | Khối nằm trong cột Library, dưới khối "Trạng thái Tác phẩm đang mở"; chỉ hiện sau khi mở Tác phẩm | 2 | mã `LibraryMode.vue:943-1011` |
| Đầu trang Tác phẩm: tên · hướng dịch · lĩnh vực · ngày tạo/sửa · "2.014 Chương" | Có | Chỉ "{total} Chương" | 2 | mã `i18n/vi.json:827` |
| Bốn ô đếm theo trạng thái (47 · 8 · 12 · 1.947) | Có | Không; chỉ tiến độ "done/total" | 5 | mã `LibraryMode.vue:870-930` (`chapter_done_count`) |
| Ô "Trạng thái Tác phẩm" (đang dùng · suy ra tự động sẽ là gì · ngày đặt · `Bỏ ghi đè ⌘⇧E`) | Ba dữ kiện + nút | Chữ trạng thái + "(ghi đè thủ công)" + nút "Tạm ngưng Tác phẩm này" · "Bỏ ghi đè" · "Đặt Chương này là Đã xong"; không nêu giá trị suy ra, không nêu ngày | 2 | mã `LibraryMode.vue:943-1010`, `i18n/vi.json:800-821` |
| Ô tìm Chương theo tên/số + chip lọc trạng thái có đếm + "Sắp theo" | Có | Không | 5 | mã `LibraryMode.vue:1011-1260` (không có điều khiển lọc Chương) |
| Nhóm theo trạng thái: Đang dịch · Tạm ngưng · Đã xong · Chưa bắt đầu (gấp mặc định) | Bốn nhóm theo thứ tự đáng chú ý | Danh sách phẳng theo `ord`, cửa sổ ảo hoá cao ≤240px | 2 | mã `LibraryMode.vue:1208-1250,2044` |
| Hàng Chương: tay kéo ⠿ · số · tên · "88 / 184 câu" · chip trạng thái · thời điểm | Có | Số · tên · trạng thái · tổng số câu; không tay kéo, không tiến độ câu, không thời điểm | 2 | mã `LibraryMode.vue:1225-1245` · ảnh: anh/library-xuat-xu-tai-lieu-va-chuong-cuoi.png (một hàng "1 · Chương 1 · Chưa bắt đầu · 7", không tay kéo, không thời điểm) |
| "Đã xong · 4 câu chưa xác nhận" trên hàng (FR120) | Có | Không | 5 | mã `LibraryMode.vue:1225-1245` |
| Chọn nhiều Chương + thanh `Gộp · Tách… · Đổi tên hàng loạt… · Đổi trạng thái · Xuất…` | Có | Một con trỏ Chương; nút `Đổi tên` · `Dời lên` · `Dời xuống` · `Gộp vào Chương trước` | 2 | mã `LibraryMode.vue:1098-1145`, `i18n/vi.json:832-836` |
| Tách Chương từ danh sách | `Tách…` | Không có trong Library; lệnh tách tồn tại ở Editor (`editor.split_chapter`) | 5 | mã `commands/index.ts` (`editor.split_chapter`) · `deferred-work.md:7854` |
| Kéo ⠿ để sắp xếp | Có | Chỉ nút Dời lên/xuống | 4 | mã `LibraryMode.vue:1118-1130` |
| Màn "Gộp 3 Chương thành một" (thứ tự gộp → kết quả, "không câu nào mất") | Có | Một nút, không màn xem trước | 5 | mã `LibraryMode.vue:1135-1145` |
| Hoàn tác `⌘Z` sau gộp | Có | Không có | 3 | `change-rut-hoan-tac-cho-gop-tach` (Ice rút `⌘Z` cho gộp/tách, 2026-08-18) |
| Xuất xứ Chương (`ChapterOrigin`) + "Áp xuất xứ đến Chương số" | Không vẽ ở danh sách | Có | 4 | mã `LibraryMode.vue:1150-1185`, `i18n/vi.json:651` |

## 6. `library-search-results.html` — tìm kiếm xuyên Library

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Vị trí | Dải trên cùng; kết quả thay lưới | Khối "Tìm kiếm" thứ ba trong cột 420px, kết quả xếp dưới; lưới Tác phẩm vẫn hiện | 2 | ảnh L · mã `LibraryMode.vue:500-690` |
| Đếm "1.847 Chương · 12 Tác phẩm" + "34 kết quả · 91 ms" | Có | "{total} kết quả", không thời gian, không tổng Chương | 2 | mã `i18n/vi.json:854` |
| Phạm vi `Cả hai · Nguyên văn · Bản dịch` | Có | Không có điều khiển; tìm cả hai, nhãn trên từng kết quả | 5 | mã `LibraryMode.vue:530-545` |
| Bộ lọc khi tìm (trạng thái · hướng dịch · sắp) | Có | Không thấy trên khối tìm | 5 | mã `LibraryMode.vue:500-690` |
| Dòng giải thích chế độ đang dùng ("mảnh không khớp manh, mành, mạnh") | Có ở cả hai chế độ | Không; chỉ nhãn nút | 2 | mã `i18n/vi.json:859-861` |
| Tự nới + nói rõ | Banner "đã tự chuyển sang khoan dung không dấu… không tương đương nhau", `⌘⇧D` quay lại | "{total} kết quả — đã tự chuyển sang khoan dung dấu ở nửa bản dịch vì tìm chính xác không ra" + nhãn "khoan dung dấu" trên từng kết quả | 1 | mã `i18n/vi.json:856,862`, `LibraryMode.vue:660-665` |
| Màu khớp đổi theo chế độ (nhấn ↔ vàng) | Có | Không tô màu; từ khớp chỉ được bọc bằng ký tự ‹ › trong chữ thuần (AD-16) | 2 | ảnh: anh/library-tim-kiem-co-ket-qua.png · mã `LibraryMode.vue:667` |
| Kết quả nhóm theo Tác phẩm (đầu nhóm: tên, hướng dịch, trạng thái, số kết quả; "⋯ 16 kết quả nữa") | Có | Danh sách phẳng, mỗi kết quả "Tác phẩm · Chương · [Nguyên văn/Bản dịch]" | 2 | mã `LibraryMode.vue:640-668` |
| Kết quả: "Chương 47 câu 88 · đã xác nhận", tô khớp, vạch lề đậm/nhạt | Có | Không có số câu, không trạng thái xác nhận, không vạch lề; có "(cấp Chương — chưa tách câu)" | 2 | mã `LibraryMode.vue:648-668` |
| Mở kết quả (`↵`, `⌘↵` mở ở Chế độ đọc, `Esc` đóng) | Có | Nút "Mở kết quả" chỉ ở kết quả đang chọn; không có mở ở Chế độ đọc | 5 | mã `LibraryMode.vue:680-690` |
| Con trỏ `‹ k/n ›` giữa kết quả | Không vẽ | Có | 4 | mã `LibraryMode.vue:610-640` |
| Trạng thái rỗng nêu lý do + số Chương bị lọc chặn + ba bước kế | Có | Câu một dòng ("Không tìm thấy — …"); không nêu bộ lọc, không gợi ý bước kế | 2 | mã `i18n/vi.json:852-853` |
| Cảnh báo "{missing}/{total} Tác phẩm chưa có văn bản trong chỉ mục" · cảnh báo truy vấn ngắn <3 ký tự | Không vẽ | Có cả hai | 4 | mã `i18n/vi.json:851,858` |
| Chân "Chỉ mục: … dựng lại lần cuối 2 ngày trước" | Có | Không | 5 | mã `LibraryMode.vue:500-690` |

## 7. `data-integrity.html` — dữ liệu: quét lại · mục mồ côi · lịch sử · sao lưu

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Quét lại: bề mặt | Cửa sổ "Quét lại thư mục / Kết quả quét" | Khối đầu trang Library (xem §1) | 2 | mã `LibraryMode.vue:345-420` |
| Quét lại: tóm tắt kết quả | Câu nói đường dẫn đã quét, số `.atproj`, thời gian, "chỉ mục là dẫn xuất" | Một dòng số liệu "Đã lập chỉ mục N · trùng work_id N · bỏ qua N · …" | 2 | mã `i18n/vi.json:776` (`mode.library.rescan_result`) |
| Nhóm "Mới xuất hiện" (Thêm vào Library · Bỏ qua) | Có | Không có bước duyệt | 5 | mã `LibraryMode.vue:345-440` |
| Nhóm "Đã di chuyển" (khớp theo mã Tác phẩm · Cập nhật đường dẫn) | Có | Chỉ cảnh báo "Hai Tác phẩm cùng work_id" | 5 | mã `LibraryMode.vue:380-395`, `i18n/vi.json:777` |
| Nhóm "Mồ côi" | Danh sách, mỗi dòng `Tìm lại…` · `Gỡ khỏi Library` | Một mục mỗi lần (con trỏ ‹ ›), đường dẫn "Trỏ tới:", nút "Gỡ khỏi chỉ mục"; không có "Tìm lại…" | 2 | ảnh L · mã `LibraryMode.vue:436-500` |
| `Tìm lại…` cho mồ côi | Có | Không | 5 | mã `LibraryMode.vue:436-500` |
| `Thêm thư mục theo dõi…` | Nhiều thư mục theo dõi | Một thư mục gốc, nút "Đổi thư mục gốc…" | 5 | mã `LibraryMode.vue:370-380` |
| `Dựng lại toàn bộ chỉ mục` + ước thời gian | Có | Không | 5 | mã `LibraryMode.vue:345-440` |
| "Áp dụng 2 thay đổi" | Có | Không (quét tự áp) | 5 | mã `LibraryMode.vue:345-440` |
| Lịch sử phiên bản: bề mặt | Hai ngăn: danh sách trái, chi tiết phải (nguyên văn · so với phiên bản trước · xuất xứ · ghi chú TM), `⌘H` | Lớp phủ `hist-panel` (≤1100px) chỉ có danh sách: thời điểm + bản dịch + "Khôi phục bản này" | 2 | mã `SegmentHistoryOverlay.vue:180,255-310` |
| Lịch sử: bốn nhãn (đang dùng · từ AI · từ TM · từ bản review) + "bạn sửa · đã xác nhận" | Có | Không dựng | 5 | `deferred-work.md:3565` (chủ: Story 2.7 · Epic 4 · Epic 7 · Epic 8) |
| Lịch sử: diff "So với bản đang dùng" | Có | Không dựng | 5 | mã `SegmentHistoryOverlay.vue:262` (vế so sánh ghi nợ có chủ) |
| Lịch sử: thời điểm tương đối ("12 phút trước", "Hôm qua 21:04") | Có | Có | 1 | mã `i18n/vi.json:1125-1128` |
| Khôi phục là thêm phiên bản, không xoá | Có | Câu khôi phục nói câu quay về "chưa xác nhận" để chờ lượt xác nhận mới | 1 | mã `i18n/vi.json:1115` · `deferred-work.md:3625` (đã đóng, phiếu quyết #79) |
| Hộp xác nhận khi bản đang soạn chưa từng xác nhận | Không vẽ | Có ("Ghi đè và khôi phục" / "Giữ bản đang soạn") | 4 | mã `SegmentHistoryOverlay.vue:217-226` |
| "Đã lưu 3 giây trước" | Có | Có ở thanh trạng thái | 1 | mã `i18n/vi.json:269` |
| Cây `.atproj` + `Mở thư mục Tác phẩm` · `Xuất TMX` · `Xuất Glossary CSV` | Ba nút | `Xuất TMX` có trong quản lý TM; không có "Mở thư mục Tác phẩm"; Glossary CSV có ở lớp Glossary | 5 | mã `i18n/vi.json:1271`; (không có `reveal`/`openPath` trong `src/`) |
| Câu nhắc "Copy khi đang mở: thiếu tối đa 5 giây" | Ở thanh trạng thái | Nằm trong thông báo sau khi tạo Tác phẩm | 2 | mã `i18n/vi.json:648` |

## 8. `empty-states.html` — trạng thái rỗng và lỗi

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Lookup không có kết quả | Tiêu đề "Không nguồn nào có 鐵鏽", "Đã tra 5 nguồn đang bật", "Tra riêng: 鐵 · 鏽", trỏ Concordance "có 3 kết quả" | Một câu: "Không tìm thấy trong từ điển. Thử tra từng chữ…" | 2 | mã `i18n/vi.json:998` |
| Lookup chưa tra | "Bôi đen một cụm ở bất kỳ panel nào… Hoặc F3 để gõ tay" | "Chọn một từ trong Nguyên văn để tra cứu." | 2 | mã `i18n/vi.json:997` |
| Hai trạng thái rỗng khác câu (không thấy ≠ chưa tra) | Có | Hai khoá khác nhau | 1 | mã `i18n/vi.json:997-998` |
| Library lần đầu: giải thích Tác phẩm là thư mục mang đi được | Có | Có ("Library chưa có Tác phẩm nào." + câu giải thích) | 1 | mã `i18n/vi.json:619-620` |
| Library lần đầu: hai lối mời (Nhập từ file · Dán văn bản) và `⌘N` | Có | Form luôn hiện, không `⌘N` | 2 | mã `LibraryMode.vue:1259-1280` |
| TM trống: giải thích cơ chế | Có | Có ("tự đầy lên mỗi khi một câu dịch được xác nhận") | 1 | mã `i18n/vi.json:1187-1188,1229` |
| TM trống: dòng "Đã có sẵn TM? Nhập TMX" | Có | Không có trong câu rỗng | 2 | mã `i18n/vi.json:1187-1188` |
| AI chưa cấu hình (không phải lỗi, mọi thứ khác vẫn chạy) | Tiêu đề + 2 đoạn + "panel ẩn được hoàn toàn", Ollama/LM Studio, `⌘, mở cấu hình` | Một đoạn: chưa cấu hình, mọi năng lực khác vẫn chạy, thêm khoá trong Cài đặt; không tiêu đề, không dòng Ollama/LM Studio, không `⌘,` | 2 | ảnh: anh/de-xuat-ai-chua-cau-hinh-co-tac-pham.png · mã `i18n/vi.json:1005` |
| Lỗi AI: `Thử lại` chỉ khi retryable, không tự thử | Có | Nút `ai.translate.retry` chỉ hiện khi `retryable === true` | 1 | mã `AiTranslationPanel.vue:316-333,443` |
| Lỗi AI: câu "Nhà cung cấp không phản hồi… bản dịch của bạn vẫn nguyên" · `Bỏ qua` | Có | Chưa xem | — | cần ảnh |

## 9. `narrow-layout.html` — màn hình hẹp

Nguồn app: `layout/workspaceLayout.ts`, `layout/WorkspaceDock.vue`, `StatusBar.vue`.

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Bốn ngưỡng (1100×820 · <820 cao · <1100 rộng hoặc <700 cao · <860 rộng) | Có | Có, `minFullWidth 1100`, `820`, `700`, `860` | 1 | mã `layout/workspaceLayout.ts:293-320` |
| Tầng "thấp": Tra cứu + Đề xuất AI gộp một khung có tab | Có | Có (`tierMerged`) | 1 | mã `layout/WorkspaceDock.vue:548-680` |
| Tầng "rất hẹp": hàng dưới thành ngăn kéo; AI mở bằng `⌘T`, Tra cứu bằng `F3` | Có | Đề xuất AI ẩn hẳn; Tra cứu rút khỏi lưới, mở bằng nút ở thanh trạng thái / ngăn kéo | 2 | mã `layout/workspaceLayout.ts:255-257`, `StatusBar.vue:474-480` |
| Tra cứu rút về thanh trạng thái: dòng kết quả rút gọn (nghĩa đầu tiên của nguồn đầu tiên) | Có | Chỉ có nút mở ngăn kéo, không có dòng nghĩa | 5 | mã `StatusBar.vue:474-480` · `deferred-work.md:671-674` (chủ: Epic 4) |
| Dưới 860: thông báo không chặn, không đổi bố cục | Có | `unsupported` = bố cục `narrow` + thông báo không chặn | 1 | mã `layout/workspaceLayout.ts:258-261` |
| Cửa sổ tối thiểu | <860 là "không hỗ trợ" | `minWidth: 800` nên tầng `unsupported` tới được | 1 | mã `src-tauri/tauri.conf.json:19` |
| Lưới Library 4 → 3 → 2 cột theo ngưỡng | Có | `auto-fill minmax(110px)` trong khối 420px, không theo ngưỡng | 2 | mã `LibraryMode.vue:1875-1881` |
| Dải TM tối đa 2 gợi ý ở mức hẹp | Có | Chưa xem | — | cần ảnh |
| Người dùng ghi đè bố cục tự đổi, nhớ theo preset (FR18) | Có | Chưa xem | — | cần ảnh |

## 10. `settings.html` — Cài đặt

Nguồn app: `SettingsOverlay.vue` (+ `SettingsGlossarySection`, `SettingsTmSection`, `SettingsExportSection`, `SettingsShortcutsSection`).

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Khung | Cửa sổ có thanh trái + nội dung phải, `⌘,` | Lớp phủ `set-panel` ≤920px, lưới `200px 1fr`, nút đóng; `Mod+Comma` | 1 | mã `SettingsOverlay.vue:239-260,560-600`, `commands/index.ts:3969` |
| Công tắc phạm vi `Toàn cục \| Tác phẩm` ở đầu cửa sổ + "Đang sửa …", "2 mục đang ghi đè", `⌘⇧R` | Có | Radio "Tầng đang sửa" chỉ trong mục AI; không đếm mục ghi đè, không nút trả toàn bộ | 2 | mã `SettingsOverlay.vue:288-296`, `i18n/vi.json:1429-1432` |
| Mục điều hướng trùng tên: AI và mô hình · Prompt · Glossary · TM · Phím tắt | Có | Có (Glossary và TM chỉ chứa ngưỡng) | 1 | mã `i18n/vi.json:1403-1409` |
| Mục "Xuất" | Không có mục này | Có (tên người dịch) | 4 | mã `SettingsOverlay.vue:470`, `i18n/vi.json:508` |
| Mục "Quyền riêng tư" | Có, qua `web-import.html` ("Cài đặt › Quyền riêng tư") | Có | 1 | mã `SettingsOverlay.vue:481` |
| Mục "Từ điển và nguồn" | Có | Không; nguồn bật/tắt ở panel Tra cứu, ghi công ở màn Nguồn dữ liệu | 3 | D2 (nguồn chuyển vào popup `Nguồn ▾`) · mã `App.vue` (`AttributionOverlay`) |
| Mục "Bố cục" (preset, panel đã ẩn) | Có | Không; preset là lệnh `layout.preset_grid` / `layout.preset_columns` | 5 | mã `i18n/vi.json:275-276` |
| Mục "Chế độ đọc" | Có | Không | 5 | mã `i18n/vi.json:1403-1410` |
| Mục "Dữ liệu và sao lưu" | Có | Không | 5 | mã `i18n/vi.json:1403-1410` |
| Mục "Cập nhật" (kiểm bản mới, mở trang phát hành, checksum) | Có | Không | 5 | `epic-phat-hanh-va-tin-cay/epic-phat-hanh-va-tin-cay.md:29` (FR111) |
| AI: chọn nhà cung cấp (Anthropic · OpenAI · Ollama · LM Studio · Endpoint khác) | Chuỗi nút chọn | Trường văn bản "Nhà cung cấp" | 2 | mã `i18n/vi.json:1413`, `SettingsOverlay.vue:312-330` |
| AI: nhãn nguồn giá trị (ghi đè/kế thừa) | Vạch lề đặc/nhạt + nhãn | Chữ "Kế thừa Toàn cục" / "Ghi đè … ở tầng Toàn cục đang là {value}" tô `mark-over`/`mark-inh` | 1 | mã `SettingsOverlay.vue:328-336,785-795` · vạch lề: cần ảnh |
| AI: "Lưu ngay khi rời ô — không có nút Lưu" | Có | Nút `Lưu` mỗi trường, `Lưu khoá`, `Xoá khoá` | 2 | mã `i18n/vi.json:1433,1441-1442` |
| AI: `Trả về kế thừa` | Nút cho cả mục | Nút cho từng trường | 1 | mã `i18n/vi.json:1434` |
| AI: Khoá API (Keychain, không hiện lại) | Có | Có ("khoá đang lưu trong keychain…", một khoá cho mọi Tác phẩm) | 1 | mã `i18n/vi.json:1436-1439` |
| AI: `Thử kết nối` · công tắc "Hiện token và ước tính chi phí" | Có | Không | 5 | mã `i18n/vi.json:1411-1442` |
| Phím tắt: bảng Lệnh · Mã lệnh · Phím · Ghi chú; đếm "247 lệnh · 3 chưa gán" | Có | Bảng Thao tác · Mã lệnh · Phím · Bỏ gán/mặc định; có đếm "Còn {so} thao tác chưa gán phím" | 1 | mã `SettingsShortcutsSection.vue:112-122`, `i18n/vi.json:1090-1098` |
| Phím tắt: ô tìm lệnh/tra ngược phím · cột Ghi chú · Xuất/Nhập bộ phím · Trả về mặc định (toàn bộ) | Có | Không; chỉ "Về mặc định" từng hàng | 5 | mã `i18n/vi.json:1086-1105` |
| Phím tắt: xung đột không giải im lặng | Có | Có ("Trùng với … thao tác gán sau đang bị chặn") | 1 | mã `i18n/vi.json:1100` |
| Phím tắt chỉ ở tầng Toàn cục | Có | Có | 1 | mã `i18n/vi.json:1088` · `deferred-work.md:688` |
| Chế độ tối | Không vẽ | Chưa xem | — | cần ảnh |

---

## Bảng lối vào ở thanh tiêu đề

Mười lối vào `.titlebar-act` (`App.vue:272-370`). D1 đưa chúng ra khỏi thanh tiêu đề; chỗ đặt mới là việc bước 2. "Mockup đặt ở đâu" ghi điều mockup/IA thực sự vẽ. Đã grep cả 29 mockup cho tên mỗi lối vào.

| # | Lối vào (dispatch) | Chữ trên nút | Mockup / IA đặt ở đâu | Kết luận |
|---|---|---|---|---|
| 1 | `ai.proofread.scope.open` | Quét chính tả | `proofreader.html` §3: bộ chọn phạm vi `Câu này ⌘P · Vùng chọn · Cả Chương` nằm trong dải Proofreader mọc dưới câu ở Editor; ô phải titlebar chỉ hiện trạng thái "Chương 47 · đã quét 184 câu ⌘P". IA không liệt kê. Tên "Quét chính tả" không xuất hiện trong mockup nào | Có chỗ cho thao tác (phím `⌘P` + dải Editor), không có chỗ cho nút |
| 2 | `ai.proofread.memory.open` | Chỗ đã bỏ qua | `proofreader.html` §4: bảng "Proofreader · Tuyết Trung Hãn Đao Hành — Đã đánh dấu 'không phải lỗi'" (cột "đã chặn N lần") là một cửa sổ riêng; cách mở không vẽ. IA không liệt kê | Có nội dung, không có lối mở |
| 3 | `glossary.queue.open` | Duyệt bảng chờ Glossary | `glossary-queue.html`: một chế độ nhìn của Workspace (tab Library/Workspace/Chế độ đọc vẫn hiện, tiêu đề "Tuyết Trung Hãn Đao Hành · bảng chờ"). IA: `Workspace › bảng chờ Glossary` | Có chỗ (Workspace); cách mở không vẽ |
| 4 | `glossary.manage.open` | Quản lý Glossary | `glossary-manage.html`: cửa sổ "Glossary · Tuyết Trung Hãn Đao Hành" `⌘G`. IA không liệt kê | Cửa sổ riêng + phím `⌘G`; không vẽ nút mở |
| 5 | `tm.manage.open` | Quản lý bộ nhớ dịch (TM) | `tm-manage.html`: cửa sổ "Translation Memory · …" `⌘M`. IA không liệt kê. `⌘M` trùng với "Gộp hai segment" ở `key-screen-workspace.html:111` và `settings.html:283` | Cửa sổ riêng; mockup tự va phím `⌘M` |
| 6 | `export.open` | Xuất | `export-share.html` (cửa sổ "Xuất" `⌘⇧E`) và `export-images-attribution.html`. IA: `Library › xuất → chọn cách xuất ảnh · khối ghi nguồn`. `chapter-list.html` §3: nút `Xuất…` trên thanh hành động khi chọn nhiều Chương | Có chỗ (Library, thanh hành động Chương, `⌘⇧E`) |
| 7 | `export.reviewer_import.open` | Nhập bản reviewer | `review-mode.html` §1 "Nhập bản Reviewer — bước nối đoạn" (tệp `chuong-40-50-reviewed.docx`); EXPERIENCE KF-3 bước 2 "Ice nhập file vào đúng Tác phẩm". Không vẽ nút/lối mở | Có bề mặt, không có lối mở |
| 8 | `review.open` | Mở Review Mode | `review-mode.html` §2: preset bố cục của Workspace "Bố cục Review Mode ⌘3 · Nguyên văn đã ẩn"; IA: `Workspace › preset Review Mode`. (`⌘3` trong mockup trùng `⌘3` của tab Chế độ đọc) | Có chỗ (preset Workspace) |
| 9 | `export.alignment.open` | Nối tay bản reviewer | `review-mode.html` §1 (khối "7 đoạn cần bạn nối tay", `2 / 7 · ↑ ↓ chuyển · ↵ nối`) là bước thứ hai của chính lần nhập ở dòng 7; `export-share.html` §3 nhắc "nối tay" như hệ quả. Không có lối mở riêng | Là bước tiếp của #7; không có lối mở riêng |
| 10 | `settings.open` | Cài đặt | `settings.html`: cửa sổ "Cài đặt `⌘,`" ; `web-import.html` và `tm-fuzzy-match.html` tham chiếu "Cài đặt". IA (cây §Information Architecture) không có Cài đặt | Có cửa sổ + phím `⌘,`; không có chỗ đặt nút |

**Tóm tắt:** không lối vào nào có nút ở thanh tiêu đề trong mockup nào. Bốn cửa sổ (#4, #5, #6, #10) mockup vẽ như cửa sổ có phím tắt riêng; #3 và #8 nằm trong Workspace theo IA; #1, #2, #7, #9 không có chỗ mở nào được vẽ.

---

## Cần ảnh (không xếp loại hoặc xếp loại còn dựa vào mã)

Ảnh Library hiện chỉ tới hàng thẻ đầu (cắt ngang).

1. **Library, cuộn hết trang**, với ít nhất 3 Tác phẩm đã tải: hàng thẻ (bìa, tên, trạng thái, tiến độ, ghi đè), khối Trạng thái Tác phẩm đang mở, khối Chương, và form nhập ở cuối. Cũng cần **Library rỗng** (0 Tác phẩm).
2. **Danh sách Chương** của một Tác phẩm đã mở (hàng, con trỏ, nút tổ chức lại, khối xuất xứ).
3. **Kết quả tìm kiếm Library**: có kết quả, chế độ nới, không có kết quả.
4. **Lớp phủ xem trước nhập**: (a) một tệp `.txt` tách Chương; (b) 50 link URL; (c) tệp song ngữ, màn bảng xem trước hàng và màn nối chỗ lệch. Cần cả sáng và tối.
5. **Cài đặt**: mục AI ở tầng Tác phẩm có một trường ghi đè; mục Phím tắt; mục Quyền riêng tư.
6. **Lịch sử phiên bản** của một câu có ≥3 phiên bản.
7. **Workspace ở cửa sổ hẹp**: 1440×790 (tầng thấp) và 1180×740 (rất hẹp, có ngăn kéo Tra cứu), kèm dải TM.
8. **Trạng thái rỗng/lỗi**: Lookup không tìm thấy, Lookup chưa tra, TM trống, AI chưa cấu hình, một lỗi AI retryable.
9. **Library sau khi quét lại có mục mồ côi** (khối thư mục gốc + mồ côi với số liệu quét).
