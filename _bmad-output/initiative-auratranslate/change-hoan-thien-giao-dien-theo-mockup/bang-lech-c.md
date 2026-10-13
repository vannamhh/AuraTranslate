# Bảng lệch nhóm C — lớp phủ quản lý, Chế độ đọc, Review, xuất

Phạm vi: `glossary-queue` · `glossary-manage` · `tm-manage` · `prompt-library` · `proofreader` · `reading-mode` · `reading-mode-typography` · `review-mode` · `export-share` · `export-images-attribution`. Không có ảnh app thật nào; mọi dòng là kết luận từ template/CSS/i18n, chỗ cần kích thước/mật độ/màu thật ghi "cần ảnh".

Loại: 1 khớp · 2 lệch cần sửa · 3 lệch có chủ đích · 4 app có, mockup không vẽ · 5 mockup có, app chưa có.
Quy ước đường dẫn: mã `src/...`, mockup `ux-auratranslate/mockups/...`. EXPERIENCE = `ux-auratranslate/EXPERIENCE.md`.

Nhận xét chung cho mọi lớp phủ (áp cho các bảng dưới, không lặp từng dòng): mockup vẽ mỗi màn như một cửa sổ có thanh tiêu đề 3 tab (`.tb` + `.modes`) và nội dung tràn ngang tối đa 1150px; app dựng lớp phủ `position: fixed; inset: 0` với `.xx-panel` giới hạn 720–960px (`GlossaryQueueOverlay.vue:352` max-width 720, `GlossaryManageOverlay.vue:~714` 880, `TmManageOverlay.vue:~735` 880, `PromptLibraryOverlay.vue:~749` 960, `ExportOverlay.vue:431` 720, `AlignmentOverlay.vue:266` 960, `ReviewerImportOverlay.vue:237` 720). Độ rộng thật so với 1150px: cần ảnh.

## 1. Bảng chờ Glossary (`glossary-queue.html` ↔ `GlossaryQueueOverlay.vue`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Khung màn | Trang trong cửa sổ, tiêu đề "Ứng viên thuật ngữ từ lần nhập" + dòng phụ về ngưỡng 5 lần | Lớp phủ toàn màn, tiêu đề "Duyệt bảng chờ Glossary", nút Đóng cùng hàng, không dòng phụ | 2 | mã: `GlossaryQueueOverlay.vue:209-213`, `i18n/vi.json:1162`; mockup `glossary-queue.html` `.top .h1/.h1s` |
| Thanh tiến độ + chú giải (Đã nhận 47 · Đã bỏ 83 · Còn lại 210) | Có, kèm thanh 4px hai màu | Không có phần tử nào đếm đã nhận/đã bỏ/còn lại; chỉ có `queueUnprocessedCount` trong state, không render | 2 | mã: `glossaryQueueState.ts:112` (không dùng trong template, grep `queueUnprocessedCount` ở `.vue`); mockup `.bar`, `.legend` |
| "Nhịp hiện tại ~2,1 giây/quyết định · còn khoảng 7 phút" | Có | Không có | 5 | mockup `.legend` dòng cuối; mã: không khoá i18n `glossary.queue.*` nào về nhịp (`vi.json:1162-1178`) |
| Bộ lọc chip (Tất cả / Đoán là tên người / địa danh / Chưa đoán được) + "Bỏ hàng loạt" | Có | Không có bộ lọc, không có bỏ hàng loạt | 5 | mockup `.filters .chipf`, `.bulk`; mã: không `filter`/`bulk` trong `GlossaryQueueOverlay.vue`, `glossaryQueueState.ts` |
| Danh sách ứng viên (hàng: thuật ngữ · phân loại đoán · số lần · ví dụ) | Hàng thuật ngữ cỡ 15px serif, cột phân loại + số lần + trích ngữ cảnh | `<ul class="gq-list">` hàng: thuật ngữ, số lần, số Chương, ngữ cảnh, gợi ý Hán Việt; không cột "phân loại đoán" ở hàng | 2 | mã: `GlossaryQueueOverlay.vue:243-290`; mockup `.row .term/.cat/.n/.peek` |
| Hàng đang chọn nở thành thẻ quyết định (thuật ngữ 26px, vị trí 217/340, nhiều ngữ cảnh Chương 3, Chương 47, khối "nhận chưa cần bản dịch") | Có, `.focus` | Không nở: hàng đang chọn chỉ đổi nền (`gq-row-current`); một ví dụ ngữ cảnh/ hàng; không khối giải thích | 2 | mã: `GlossaryQueueOverlay.vue:243-250,276-282`; mockup `.focus .fterm/.ctx/.notrans`; EXPERIENCE `Component Patterns` dòng "Bảng chờ Glossary" (`EXPERIENCE.md:71`) |
| Phân loại: 4 chip có số 1-4, "máy đoán — bấm số để đổi" | Chip kèm `sup` số | Có 4 chip radio, đổi bằng phím 1-4 (`onKeydown`), nhưng chip không hiện số; chỉ hiện cho hàng chờ quyết | 2 | mã: `GlossaryQueueOverlay.vue:123-128,168-173,309-321`; mockup `.guess .gopt sup` |
| Phím Nhận N / Bỏ B | Dải phím ở chân | Có phím N/B + 4 nút "Nhận/Bỏ/Chuyển lên/Chuyển xuống" | 1 | mã: `GlossaryQueueOverlay.vue:184-192,333-345`; mockup dải `N Nhận · B Bỏ` |
| Phím Space "Để lại, tính sau" | Có | Không có nhánh `' '` trong `onKeydown`, không nút | 5 | mã: `GlossaryQueueOverlay.vue:165-197`; mockup khối phím |
| "Dừng bất cứ lúc nào — Esc. Lần sau mở lại đúng chỗ này" | Có | Esc đóng được; nhớ đúng chỗ khi mở lại: chưa xem | 2 | mã: `GlossaryQueueOverlay.vue:203`; "nhớ vị trí" chưa kiểm — cần ảnh/chạy |
| Dòng "Không mục nào tự vào Glossary…" / "Xuất xứ ghi tự động" | Có chân trang | Không có | 5 | mockup chân `.win`; mã: không khoá i18n |
| Ứng viên "thu hoạch từ review" (X → Y, đổi ở n/m lần, không nhất quán) | Không có trong mockup bảng chờ (thuộc `review-mode.html` mục thu hoạch) | Có `gq-harvest`, `gq-inconsistent` ngay hàng | 4 | mã: `GlossaryQueueOverlay.vue:258-272`, `vi.json:1168-1169` |
| Ghi chú "phân loại chỉ tồn tại trong phiên" | Không | Có `gq-hint` | 4 | mã: `GlossaryQueueOverlay.vue:323-325`, `vi.json:1178` |
| Trạng thái rỗng/ tải/ lỗi (ba câu rỗng) | Không vẽ ở bảng chờ (thuộc `empty-states.html`) | Có | 4 | mã: `GlossaryQueueOverlay.vue:224-241` |
| Hàng đã xử lý: lùi bằng màu chữ + ✓/✕, không opacity | Có | Có `gq-mark` ✓/✕ + chữ ẩn cho trình đọc màn hình | 1 | mã: `GlossaryQueueOverlay.vue:288-306`; mockup `.row.done/.skip` |

## 2. Quản lý Glossary (`glossary-manage.html` ↔ `GlossaryManageOverlay.vue`, `GlossaryImportOverlay.vue`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Phạm vi tầng: Toàn cục 412 · Tác phẩm 1.847 · Cả hai | Segmented control đầu thanh lọc, có số đếm | Không có công tắc tầng; mọi hàng hai tầng trong một danh sách, tầng chỉ là badge | 2 | mã: `GlossaryManageOverlay.vue:381-433` (4 select/ checkbox, không tầng), `:508`; mockup `.bar .seg` |
| Ô tìm | Có | Có, `gm-input` có nhãn | 1 | mã: `GlossaryManageOverlay.vue:384-393`; mockup `.srch` |
| Lọc Phân loại | Pill (Tất cả/Tên người/Địa danh/Thuật ngữ) | `<select>` | 2 | mã: `GlossaryManageOverlay.vue:396-405`; mockup `.pill` |
| Lọc "Chờ chốt · 23" | Pill vàng có số | `<select>` Trạng thái (Mọi/Đã chốt/Chờ chốt), không số | 2 | mã: `GlossaryManageOverlay.vue:413-427`, `vi.json:1322-1325`; mockup `.pill.warn` |
| Lọc xuất xứ | Không có bộ lọc xuất xứ (có cột) | Có `<select>` xuất xứ | 4 | mã: `GlossaryManageOverlay.vue:406-412`, `:237-240` |
| Sắp theo tần suất | Chữ "Sắp theo: tần suất trong Tác phẩm" | Checkbox | 2 | mã: `GlossaryManageOverlay.vue:428-433`; mockup `.bar .r` |
| Bảng có đầu cột (Thuật ngữ nguồn · Bản dịch · Phân loại · Xuất xứ · Dùng · Ghi chú · Thêm ngày) | `<table>` có `th` | `<ul role=listbox>` hàng flex-wrap: thuật ngữ, badge tầng, bản dịch, ghi chú, số lần; không đầu cột | 2 | mã: `GlossaryManageOverlay.vue:485-529`, CSS `:831-842`; mockup `table th` |
| Cột Xuất xứ (thủ công / đề xuất khi nhập / thu hoạch từ review) | Có cột + mô tả ("reviewer đổi ở 23/24 lần") | Không hiện ở hàng, chỉ lọc | 2 | mã: `GlossaryManageOverlay.vue:506-526` (không có `term_origin` trong template); mockup `td .orig` |
| Cột Phân loại, cột Thêm ngày | Có | Không hiện ở hàng (phân loại chỉ trong form sửa) | 2 | mã: `GlossaryManageOverlay.vue:546-551`; mockup `th` |
| Cột "Dùng" (số lần, 1.204×) | Có | `gm-count` (— khi null) | 1 | mã: `GlossaryManageOverlay.vue:524-526` |
| Vạch lề trái mang trạng thái (tầng Tác phẩm xanh, chờ chốt vàng) | Có, `td:first-child::before` | Không vạch lề; trạng thái bằng badge chữ | 2 | mã: `GlossaryManageOverlay.vue:831-860` (không ::before); mockup `tr.g/.wait ::before`; ảnh: anh/glossary-quan-ly-co-tac-pham.png (hàng danh sách có badge "Tác phẩm này", chỉ nền hàng chọn đổi màu, không vạch lề) |
| Mục bị che (tầng Toàn cục thụt vào, chữ lùi) | Hàng thụt vào dưới hàng che nó | Hàng có badge "Đang bị che", không thụt, đứng thứ tự do sắp xếp | 2 | mã: `GlossaryManageOverlay.vue:509,858-860`; mockup `tr.shadow` |
| Bản dịch chờ chốt (chữ nghiêng vàng "chờ chốt bản dịch") | Có | Badge "Chưa chốt" thay bản dịch | 1 | mã: `GlossaryManageOverlay.vue:519-520`, `vi.json:1334` |
| Giải thích (dải chờ chốt, hàng thụt) | Có hộp `warnbox`, `expl` | Không (chỉ `gm-note` khi thiếu tầng Tác phẩm) | 5 | mockup `.warnbox`, `.expl`; mã: `GlossaryManageOverlay.vue:377-379` |
| Nút Thêm thuật ngữ | Có ở chân | Không trong màn quản lý (thêm đi qua `GlossaryQuickAdd.vue`) | 2 | mã: dispatch liệt kê `GlossaryManageOverlay.vue` (cancel/close/delete/edit/export_csv/import_csv/next/prev/promote/save, không `add`); mockup `.btnrow` |
| Xuất CSV · Nhập CSV · Chuyển lên Toàn cục | Có | Có (Xuất CSV, Nhập CSV, "Đẩy lên Toàn cục") | 1 | mã: `GlossaryManageOverlay.vue:592-620,645-670`; mockup `.btnrow` |
| Sửa tại chỗ (nhấp đúp ô) · ⌫ xoá | Sửa tại ô, ⌫ xoá | Form sửa riêng (bản dịch, ghi chú, phân loại) + nút Sửa/Xoá xác nhận hai nhịp | 2 | mã: `GlossaryManageOverlay.vue:533-560,592-604`; mockup `.btnrow` dòng cuối |
| Chọn tầng xuất/nhập (radio) | Trong màn xem trước nhập: "Nhập vào tầng Toàn cục / Tác phẩm này" | Radio tầng ở chân màn quản lý, áp cho cả xuất và nhập | 3 | mã: `GlossaryManageOverlay.vue:621-644`; nguồn: `change-tach-nua-chon-tep-glossary/change-tach-nua-chon-tep-glossary.md` (hộp thoại chọn tệp tách khỏi nhập) |
| Xem trước nhập CSV: tệp, số dòng/cột, cột bỏ qua, mới/giống/bất đồng | Có mục 2 của mockup | Có `GlossaryImportOverlay.vue`: tệp, số hàng, cột bỏ qua, mới/giống, bất đồng, Giữ của tôi / Lấy của file, "không ghi trước khi bấm" | 1 | mã: `GlossaryImportOverlay.vue` template (gi-summary, gi-conflict-list, gi-hint); `vi.json:1359-1378`; mockup mục 2 |
| Khối "Đọc được gì trong file" (đoạn mẫu CSV 4 dòng) | Có | Không | 5 | mockup mục 2 `pre`; mã: không khoá i18n `glossary.import.*` tương ứng |
| Nút xác nhận "Nhập 597 mục mới · giữ 7 mục đang có" | Có | Có (`confirm_summary`) | 1 | mã: `vi.json:1378` |
| Lưới hai tầng nhấp tầng khi chưa mở Tác phẩm | Không vẽ | Có `gm-note` "tầng Tác phẩm không khả dụng" | 4 | mã: `GlossaryManageOverlay.vue:377-379` |

## 3. Quản lý Translation Memory (`tm-manage.html` ↔ `TmManageOverlay.vue`, `TmImportOverlay.vue`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Phạm vi tầng (Toàn cục 38.204 · Tác phẩm 12.409 · Cả hai) | Segmented có số đếm | `<select>` "Tầng" (Cả hai/…), không số đếm | 2 | mã: `TmManageOverlay.vue:395-412`, `vi.json:1219-1220`; mockup `.bar .seg` |
| Ô tìm câu nguồn/đích | Có | Có | 1 | mã: `TmManageOverlay.vue:383-393`; mockup `.srch` |
| Lọc xuất xứ (Mọi · Của tôi 11.882 · Của người khác 527) | Pill có số đếm | `<select>` xuất xứ (Mọi/Của tôi/Của người khác kể cả nhập song ngữ), không số | 2 | mã: `TmManageOverlay.vue:414-426`, `vi.json:1221-1223`; mockup `.pill` |
| "Sắp theo: ghi gần nhất" | Có | Không có điều khiển sắp xếp | 5 | mockup `.bar .r`; mã: không `sort` trong `TmManageOverlay.vue`/`tmManageState.ts` |
| Dải sức khoẻ TM (thanh 5px ba màu + chú giải % + câu giải thích) | Thanh chồng + chú giải + `hnote` | `<ul class="tm-health">` các badge chữ "Nhãn: n (p%)", không thanh, không câu giải thích | 2 | mã: `TmManageOverlay.vue:370-380`; mockup `.health .mix .track`, `.hnote` |
| Dòng cặp: nguồn · đích · meta (xuất xứ, vị trí, thời gian) · Sửa/Xoá | Lưới 4 cột `1fr 1fr 128px 92px`, nút Sửa/Xoá mỗi hàng | Hàng flex-wrap: nguồn, đích, badge xuất xứ, badge tầng/bản sao, ngày; nút Sửa/Xoá ở thanh chung dưới danh sách theo hàng đang chọn | 2 | mã: `TmManageOverlay.vue:457-515`, CSS `:857-870`, `:591-637`; mockup `.pair` grid, `.acts` |
| Vạch lề trái theo xuất xứ (own xanh, other vàng, nhập xám) | Có `.pair::before` | Không vạch lề; badge chữ | 2 | mã: `TmManageOverlay.vue:857-870` (không ::before); mockup `.pair::before`; màu: cần ảnh |
| Nhóm "cùng câu nguồn, 3 bản dịch" (đầu nhóm nền vàng nhạt, dòng "— cùng câu nguồn —") | Có, `.mhead`, `.pair.multi` | Có `tm-group` đầu nhóm (nguồn + badge), hàng sau không lặp nguồn | 1 | mã: `TmManageOverlay.vue:466-492`; mockup `.mhead` |
| Dải "Ba bản dịch không phải lỗi cần dọn…" và "Xuất xứ suy ra từ hành vi" | Có hai khối giải thích | Không | 5 | mockup `.expl`; mã: không khoá i18n tương ứng |
| Xuất TMX · Nhập TMX | Có | Có, kèm radio tầng xuất/nhập | 1 | mã: `TmManageOverlay.vue:638-684`, `vi.json:1270-1272`; mockup `.btnrow` |
| "Xoá toàn bộ cặp của người khác" | Có | Có, 3 nhãn (cả hai/Tác phẩm/Toàn cục), hai nhịp xác nhận | 1 | mã: `TmManageOverlay.vue:620-630`, `vi.json:1245-1247` |
| "Chỉ dùng cặp của tôi cho Tác phẩm này" · "Xuất riêng N cặp của tôi" (mục 2 vai biên tập) | Có | Không có hai nút; xuất riêng "chỉ của tôi" cũng không có trong radio tầng | 5 | mockup mục 2 `.btnrow`; mã: không khoá `tm.manage.*` tương ứng (`vi.json:1211-1269`) |
| Dải ngữ cảnh vai biên tập ("Tác phẩm… bạn đang ở vai biên tập, 86,8% …") | Có | Không | 5 | mockup mục 2; mã: không khoá i18n |
| Hộp thoại xuất TMX: phạm vi 4 lựa chọn (+1.640 cặp bản dịch nhiều) và minh hoạ `<prop x-origin>` | Có mục 3 | Chọn tầng xuất bằng 2 radio (Tầng Tác phẩm/Toàn cục), không chọn "chỉ của tôi", không "kèm mọi bản dịch" | 5 | mã: `TmManageOverlay.vue:638-660`; mockup mục 3 `Chọn phạm vi` |
| "Đẩy lên Toàn cục" từng cặp | Không vẽ | Có | 4 | mã: `TmManageOverlay.vue:611-618`, `vi.json:1257-1269` |
| Sửa bản dịch, ghi chú về bản sao tầng Toàn cục | Sửa tại chỗ | Form sửa riêng + ghi chú `edit_global_note` | 4 | mã: `TmManageOverlay.vue:518-548` |
| Tab/ngăn trạng thái rỗng, lỗi, cắt bớt ("hiện n/m nhóm") | Không | Có | 4 | mã: `TmManageOverlay.vue:429-455` |

## 4. Thư viện prompt (`prompt-library.html` ↔ `PromptLibraryOverlay.vue`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Bố cục hai cột (danh sách trái, soạn phải) | Có | Có `grid 260px 1fr` | 1 | mã: `PromptLibraryOverlay.vue:514`, CSS `pl-cols`; mockup mục 1 |
| Phạm vi tầng Toàn cục 9 · Tác phẩm 2 · Cả hai | Segmented có số | Không công tắc; danh sách hai nhóm `h3` ("Tác phẩm", "Toàn cục") xếp dọc | 2 | mã: `PromptLibraryOverlay.vue:~532-545,585-587` (`pl-group-h`); mockup `.seg` |
| Dòng "Đang dùng: Tiên hiệp · giọng cổ trang" + badge "đang dùng" | Có cả dòng và badge | Chỉ badge "đang dùng" ở hàng, không dòng tổng ở đầu | 2 | mã: `PromptLibraryOverlay.vue:~541,552` (`in_use_badge`); mockup `Đang dùng:` |
| Badge hàng "bị che" / "che bản Toàn cục" | Có | Có `shadowed_badge`, `shadows_badge` | 1 | mã: `PromptLibraryOverlay.vue:~543,556` |
| Tiêu đề bộ + ghi chú tầng ("sửa ở đây không chạm bản Toàn cục") | Có | Có `editing_work_note`/`editing_global_note` | 1 | mã: `PromptLibraryOverlay.vue:~656-658` |
| Cặp ngôn ngữ (Trung → Việt) + phạm vi áp dụng (mọi segment / lời thoại / tả cảnh) | Có | Không có trường nào | 5 | mockup `Trung → Việt`, `Áp cho…`; mã: không trong `PromptLibraryOverlay.vue` |
| Ô soạn nội dung prompt | Có, biến chèn dạng `{{…}}` được tô | `<textarea>` thuần (12 hàng), không tô biến | 2 | mã: `PromptLibraryOverlay.vue:~684-686`; mockup `Nội dung prompt`; ảnh: anh/prompt-bo-moi-form.png (ô nội dung trống, thuần) |
| Bảng "Biến chèn động" (4 biến, nhãn bắt buộc/nên có/chưa dùng) | Có, 4 biến | Có `pl-vars` liệt kê biến (3 mô tả: glossary_terms, source_segment, tm_similar_segments); không nhãn bắt buộc/nên có; không `{{chapter_context}}` | 2 | mã: `PromptLibraryOverlay.vue:674-679`, `vi.json:206-209`; mockup `Biến chèn động` |
| Cảnh báo bỏ `{{glossary_terms}}` tự tắt Glossary Enforcement, hiện lúc lưu | Có | Có `warning_glossary_terms_missing` + `warning_unknown_markers` | 1 | mã: `PromptLibraryOverlay.vue:688-694`; mockup mục 1 |
| Nút "Dùng prompt này" | Có | Có "Dùng bộ này" | 1 | mã: `vi.json:213` |
| Nút "Xem prompt cuối cùng đã gửi" (FR71) ngay tại chỗ soạn | Có | Không có trong màn soạn; Inspector là lớp phủ riêng mở từ nơi khác | 2 | mã: grep `inspector` trong `PromptLibraryOverlay.vue` = 0; `App.vue:475-476` |
| Nút Nhân bản | Có | Không | 5 | mockup `Nhân bản`; mã: grep `duplicate\|clone` = 0 |
| Xoá (hai nhịp xác nhận) | Có `Xoá` | Có, đổi nhãn khi chờ xác nhận | 1 | mã: `PromptLibraryOverlay.vue:~718-724` |
| "Lưu ngay khi rời ô" (tự lưu) | Có | Nút "Lưu nội dung" tường minh | 2 | mã: `PromptLibraryOverlay.vue:~700-703`; mockup `Lưu ngay khi rời ô` |
| Đổi tên bộ | Không vẽ | Form đổi tên riêng | 4 | mã: `PromptLibraryOverlay.vue:~661-671` |
| Tạo bộ mới (chọn tầng, tên, nội dung) | Không vẽ | Có form tạo, radio tầng | 4 | mã: `PromptLibraryOverlay.vue:~592-640`, `pl-row-new` `:~569` |
| Xuất nhiều bộ: chọn bộ + danh sách tệp `.prompt.md` | Có (mục 2, kèm minh hoạ front-matter) | Có `pl-export-many` fieldset chọn bộ, danh sách tệp ghi; không minh hoạ front-matter | 1 | mã: `PromptLibraryOverlay.vue:551-570`; mockup mục 2 |
| Nhập từ file | Có | Có `pl-row-import` + `PromptImportOverlay.vue` | 1 | mã: `PromptLibraryOverlay.vue:~573-580` |

## 5. AI Proofreader (`proofreader.html` ↔ `ProofreaderStrip.vue`, `ProofreadScopeOverlay.vue`, `ProofreadMemoryOverlay.vue`, `GridPanel.vue`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Gạch chân lượn sóng dưới đúng cụm, `underline-offset: 4px`, không đụng vạch lề | Có (mục 1, 2) | Có: `.proofread-error`/`.proofread-meaning` `wavy`, `1px`, `offset 4px`, bọc span | 1 | mã: `panels/GridPanel.vue:2446-2461`, `panels/proofreadHighlight.ts:79-97`; mockup mục 2 |
| Hai loại, hai màu (lỗi = error; nghi nghĩa = tm-text) | Có | `--color-error` / `--color-tm-text` | 1 | mã: `panels/GridPanel.vue:2454-2460` (mockup nói `tm-rule`, app dùng `tm-text`; chữ thật hay nền: cần ảnh) |
| Bảy loại phát hiện (chính tả, ngữ pháp, dịch lệch, quá thoáng, khó hiểu) | Hai nhóm FR80/FR81 | Năm khoá `proofread.kind.*` | 1 | mã: `vi.json:127-131` |
| Thẻ phát hiện: loại · cụm · "3 / 6" · giải thích · đề xuất · Nhận / Bỏ qua / Không phải lỗi | Dải "Proofreader — phát hiện 3 trên 6" dưới câu, kèm đoạn `Bản gốc… Bản dịch…` | `<section class="proofreader-strip">` ở `App.vue:422`, `pfs-card`: loại, cụm, vị trí `n / m`, giải thích, đề xuất, bốn nút accept/dismiss/ignore/prev/next | 1 | mã: `ProofreaderStrip.vue:147-190`, `vi.json:132-133`; mockup mục 1 |
| Vị trí của dải: mọc dưới câu đang sửa, trong Editor | Dải mọc dưới câu đang sửa, trong cột Bản dịch | Dải nằm ngoài dock, mount ở `App.vue:422` (viền trái 3px đỏ, `flex: none`); có thật sự dưới Editor hay dưới cả dock: cần ảnh | 2 | mã: `App.vue:422`, `ProofreaderStrip.vue:238-249`; mockup `.strip` dưới câu 88 (cần ảnh) |
| Thứ tự ưu tiên ba dải (Glossary > Proofreader > TM) | Có mục 3 | `inlineStripPriority.ts`; chưa đối chiếu thứ tự | 2 | mã: `panels/inlineStripPriority.ts` (chưa mở); đánh dấu "chưa xem" |
| Câu hiển thị F8/⇧F8 | "F8 kế tiếp · ⇧F8 trước · Esc ẩn" | Nút Trước/Kế tiếp; phím F8 chưa xác nhận | 2 | mã: `ProofreaderStrip.vue:~188-190`; grep F8 trong `config/*.ts` = 0 |
| Chân Editor: "Đã lưu 4 giây trước · Chương 47 câu 88/184 · 2 chính tả 4 nghi nghĩa · Quét lúc 14:02 · 2.610 token · ~0,024 USD" | Có | Dòng usage sau quét có chi phí (`usage_total_with_cost`); không thấy dòng đếm theo hai loại | 2 | mã: `ProofreaderStrip.vue:138-140`; mockup `.foot` |
| Chọn phạm vi: Câu này ⌘P · Vùng chọn 12 câu · Cả Chương 184 câu · ~0,024 USD (chi phí TRƯỚC khi bấm) | Có, ba ô | `ProofreadScopeOverlay.vue` ba ô (Câu này / Vùng chọn / Cả Chương) kèm số câu; không có chi phí ước tính | 2 | mã: `ProofreadScopeOverlay.vue:~26-52` (`proofread.scope.count` chỉ số câu); `vi.json:120-123`; mockup mục 3 |
| Mở màn chọn phạm vi từ thanh tiêu đề | Không (lệnh ⌘P) | Nút `titlebar-act` `data-proofread-scope-open` | 4 | mã: `App.vue:266-275` (thuộc agent A cho vị trí nút) |
| Bỏ qua có ghi nhớ: danh sách "Đã đánh dấu không phải lỗi" (cụm, lý do, đề xuất, ngày, "đã chặn N lần") | Có, kèm cột "đã chặn 4 lần" | `ProofreadMemoryOverlay.vue`: danh sách `kind`, `text` (ngữ cảnh ngắn), nút Bỏ ghi nhớ/Xoá toàn bộ; không có đếm lần chặn, không ngày | 2 | mã: `ProofreadMemoryOverlay.vue:~22-70`; `vi.json:142-149`; mockup mục 4 |
| Tỷ lệ báo động giả (Đã nhận 61% · Không phải lỗi 22% · Bỏ qua 17%) | Có, thanh + giải thích | Không có | 5 | mockup mục 4; mã: grep `percent\|false_alarm` trong `ProofreadMemoryOverlay.vue`/`proofreadMemoryState.ts` = 0 |
| "Khôi phục mục đã bỏ qua" (từng mục) | Có | Nút "Bỏ ghi nhớ" từng hàng | 1 | mã: `vi.json:611`; `ProofreadMemoryOverlay.vue:52-70` |
| "Xoá toàn bộ ghi nhớ của Tác phẩm này" | Có | Có, hai nhịp xác nhận | 1 | mã: `vi.json:147-149,612` |
| Không bao giờ tự sửa văn bản | Có | Chỉ áp qua "Nhận đề xuất" từng cái; chip hoàn lại `reverted_hint` | 1 | mã: `vi.json:135`; chưa thấy "sửa tất cả" |
| Hủy giữa chừng khi quét cả Chương | Có | Nút "Huỷ" trong dải | 1 | mã: `ProofreaderStrip.vue:~151-154`, `vi.json:137-138` |
| Trạng thái "đang quét n/m", "sạch", "đã lọc", "chưa chỉ được chỗ" | Không vẽ | Có | 4 | mã: `vi.json:110-115,139-150` |

## 6. Chế độ đọc (`reading-mode.html`, `reading-mode-typography.html` ↔ `modes/ReadingMode.vue`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Thứ tự thanh công cụ: Song ngữ · Cỡ chữ (3 mức Thoáng/Cân/Đặc) · Tinh chỉnh · Sáng/Tối · Mục lục · Chỗ đã đánh dấu | Có | Cùng thứ tự: Song ngữ, Thoáng, Cân, Đặc, Tinh chỉnh, Đổi giao diện, Mục lục, Chỗ cần sửa | 1 | mã: `modes/ReadingMode.vue:416-481`; mockup `reading-mode.html` `.bar` |
| Nhóm "Cỡ chữ" có nhãn nhóm + 3 nút liền (segmented) | Có nhãn "Cỡ chữ" | Ba nút `.btn` rời, không nhãn nhóm | 2 | mã: `modes/ReadingMode.vue:416-481`, CSS `:794-799` flex wrap gap |
| Gợi ý phím trên nút (B, 1 2 3, D, ⌘L, M) | Có chip `.k` | Không thấy trong template | 2 | mã: `modes/ReadingMode.vue:416-481`; mockup `.k` |
| Công tắc theme hiện "◐ Sáng / ◑ Tối" | Có | Nhãn cố định "Đổi giao diện" | 2 | mã: `vi.json:894`; mockup `◐ Sáng D` |
| Nút "Chỗ đã đánh dấu" kèm số ("⚑ 3 chỗ đã đánh dấu") | Có số | Nhãn cố định "Chỗ cần sửa", không số | 2 | mã: `vi.json:906`, `modes/ReadingMode.vue:~490-492`; mockup `⚑ 3 chỗ đã đánh dấu` |
| Thanh tiêu đề Chương: "Chương 47 · Cánh cửa cuối hành lang" + "128 Chương đã dịch · đang ở 44" | Có | Chỉ `chapter-title` h2 trong cột đọc; không dòng tiến độ | 2 | mã: `modes/ReadingMode.vue:~560-570`; mockup `.bar` hàng hai |
| Dòng xuất xứ bản dịch ("bản dịch của Minh · bạn đã biên tập") | Có | Không trong Chế độ đọc (có `ChapterOrigin.vue` ở nơi khác) | 5 | mockup; mã: grep `ChapterOrigin` trong `ReadingMode.vue` = 0 |
| Cột đọc `68ch` (Cân), serif, giãn 1.8; 62ch/19px/1.95; 76ch/16px/1.66 | Có | `width` theo `ch` (đã đo), sàn 1.66 | 1 | mã: `modes/readingState.ts:438-440,544-555`, `ReadingMode.vue:73-90` CSS cột; EXPERIENCE `§Chế độ đọc` (`EXPERIENCE.md:347`); thước thật: cần ảnh |
| Bộ chỉnh mịn (thanh trượt cỡ chữ, giãn dòng) sau một lần bấm | Có "Tinh chỉnh ⌘," | `tuner` hai thanh | 1 | mã: `modes/ReadingMode.vue:483-505` |
| Điều khiển cỡ chữ `A- A A+` kèm `⌘+/⌘−` | `reading-mode-typography.html` | Không; thay bằng 3 preset | 3 | nguồn: `EXPERIENCE.md:347-354` ("ba preset trên thanh công cụ", văn bản thắng mockup, `EXPERIENCE.md:427`); mockup `reading-mode.html` cũng dùng 3 preset |
| Song ngữ ở lề trái, cỡ nhỏ, không chen dòng đọc | Có | Cột `.margin` trước `.column`, `max-width 22ch`, `source-latin` | 1 | mã: `modes/ReadingMode.vue:543-558,826-845`; mockup `reading-mode.html` ⑦ |
| Song ngữ trong `reading-mode-typography` "nguyên văn xám nhạt PHÍA TRÊN bản dịch, ngăn bằng vạch lề" | Trên bản dịch | Ở lề trái | 3 | nguồn: `EXPERIENCE.md:357` (nguyên văn ở lề trái, không chen giữa dòng đọc) |
| Câu chưa xác nhận: gạch chấm dưới câu | Có `.s.unconf` | `.unconfirmed` `border-bottom: 1px dotted var(--color-ornament)` | 1 | mã: `modes/ReadingMode.vue:~905-910`; mockup `.s.unconf` |
| Affordance đánh dấu ẩn, hiện khi rê/tiêu điểm: "⚑ M ↵ sửa" | Có, gồm "↵ sửa" | `mark-affordance` ẩn, hiện khi hover/focus; nhãn "Đánh dấu/Đã đánh dấu"; không nút "↵ sửa" | 2 | mã: `modes/ReadingMode.vue:~921-945`, `vi.json:903-904`; mockup `⚑ M ↵ sửa` |
| Mốc biên giới cuối trang (Hết phần đã dịch · "Dịch tiếp Chương 48 ↵" · "Xem 3 chỗ đã đánh dấu" · "Về đầu Tác phẩm") | Ba nút | `frontier`: tiêu đề, phạm vi, ghi chú, **một** nút "Dịch tiếp Chương n" | 2 | mã: `modes/ReadingMode.vue:660-681`, `vi.json:880-886`; mockup `.end` |
| Mốc cuối ở `reading-mode-typography`: "← Chương 46 · Chương 48 →" | Có | Không | 3 | nguồn: `reading-mode.html` ⑨ "mốc biên giới thay cho hai nút trước/sau", `EXPERIENCE.md:357` ("chuyển Chương liền mạch") |
| Hình nhúng đúng vị trí, caption đã dịch, không chừa chỗ nếu thiếu | Có | `reading-figure` + `figcaption` chỉ khi có caption | 1 | mã: `modes/ReadingMode.vue:~570-577,637-647`; EXPERIENCE `:357` |
| Danh sách chỗ đã đánh dấu: "Ch 44 · c12 … Sửa →" gom theo Tác phẩm | Lớp phủ riêng, mỗi hàng có nút "Sửa →" | `toc-overlay`/`toc-panel` dùng chung với Mục lục; hàng "tên Chương — câu" (không nút từng hàng); nút Mục trước/kế/Mở trong Workspace/Đóng | 2 | mã: `modes/ReadingMode.vue:683-730`; mockup `Chỗ bạn đánh dấu khi đọc` |
| "Sửa lần lượt cả 3 chỗ" · "Bỏ hết đánh dấu" · phím ↑↓ ↵ ⌫ | Có | Không có hai nút; chuyển bằng prev/next | 5 | mockup `Sửa lần lượt…`, `Bỏ hết đánh dấu`; mã: `ReadingMode.vue:722-728` |
| Mục lục (⌘L) lớp phủ | Nút trên thanh (nội dung không vẽ) | Lớp phủ với danh sách Chương, Chương trước/kế, Mở, Đóng | 4 | mã: `modes/ReadingMode.vue:732-760` |
| Nền tối cùng bố cục, `#26241f` / `#e8e3d8` | Có (màu viết cứng mockup) | Qua token theme; màu thật: cần ảnh | 1 | mockup `reading-mode-typography.html` ③ nền tối; mã: `ReadingMode.vue` (token); màu: cần ảnh |
| Thông báo trạng thái (đang tải, không có Tác phẩm, rỗng, bị cắt hết) và lỗi lưu dấu | Không vẽ | Có `status`, `marker-error` | 4 | mã: `modes/ReadingMode.vue:507-511`, `:243-253`, `vi.json:873-878` |

## 7. Review Mode (`review-mode.html` ↔ `ReviewerImportOverlay.vue`, `AlignmentOverlay.vue`, `layout/ReviewDock.vue`, `panels/Review*.vue`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Bước nhập bản Reviewer: tên tệp, "Khớp cấu trúc đoạn", "Đã khớp 213/220, 7 cần nối tay" | Có | `ReviewerImportOverlay.vue`: kết quả "Đã nhập n Chương (m hàng); k thay bản cũ"; không con số khớp/chưa khớp | 2 | mã: `ReviewerImportOverlay.vue:~100-125`, `vi.json:442`; mockup mục 1 |
| Nối tay: hai cột "Bản của bạn / Bản Reviewer đã sửa", hàng `→` / `?` | Hai cột đối ứng, ký hiệu `→`, `?` | `AlignmentOverlay.vue`: danh sách "Chưa xử lý" (câu dịch / hàng reviewer, mỗi hàng nguồn + đích) rồi "Đã nhóm" (máy khớp/nối tay/bỏ qua) | 2 | mã: `AlignmentOverlay.vue:~190-231`, `vi.json:445-467`; mockup `.pairs` (bố cục cột: cần ảnh) |
| Nút "Nối hai đoạn của tôi thành một" · "Bỏ qua đoạn này" · "Xem 5 chỗ còn lại" · "2/7 · ↑↓ · ↵ nối" | Có | Chọn bằng Space, Enter nối, S bỏ, U tách, Esc; chỉ một dòng gợi ý phím, không nút | 2 | mã: `AlignmentOverlay.vue:231`, `vi.json:467` |
| Hai cửa sổ: "Bản dịch của tôi" trái, "Bản Reviewer đã sửa" phải, nguyên văn ẩn | Có | `ReviewDock.vue` hai panel dockview cạnh nhau (`panel.review_mine` trái, `panel.review_copy` phải) | 1 | mã: `layout/ReviewDock.vue:36-61`, `vi.json:919-920` |
| Preset bố cục "Review Mode ⌘3 · Nguyên văn đã ẩn" trong Workspace | Trạng thái trên thanh tiêu đề | Lệnh `review.open` ⌘⌥3, panel riêng; nút mở ở thanh tiêu đề | 1 | mã: `commands/index.ts:1267`, `App.vue:344-348` |
| Diff từng đoạn: chữ thêm/xoá tô màu | Có | `ReviewSpans.vue` `<ins>` gạch chân nền thêm, `<del>` gạch ngang nền xoá | 1 | mã: `panels/ReviewSpans.vue:12-27` |
| Nhãn đoạn "Đoạn 2 · reviewer có sửa" | Có | Không nhãn số đoạn; nhãn chỉ khi có quyết định (đã chấp nhận/bỏ qua/sửa tay) | 2 | mã: `panels/ReviewMinePanel.vue:~18-30`; `vi.json:943-945` |
| Chấp nhận ↵ / Bỏ qua ⌫ ở đoạn đang chọn | Có | Nút Chấp nhận/Bỏ qua trên hàng hiện tại; xác nhận ghi đè bản nháp | 1 | mã: `panels/ReviewMinePanel.vue:43-60`, `vi.json:941-942,951-953` |
| Dải "11 thay đổi · đã xử lý 1" + phím ↵ ⌫ ↑↓ | Chân | `review-bar` `diff-stats`, `diff-status` | 1 | mã: `layout/ReviewDock.vue:79-83`, `vi.json:936-940` |
| "Thay đổi được chấp nhận đưa câu về chưa xác nhận" | Chú thích | Chưa kiểm hành vi; chuỗi không thấy ở UI | 2 | mockup dòng cuối `Thay đổi được chấp nhận…`; mã: không khoá i18n tương ứng (`vi.json:923-953`) |
| Panel Review chỉ đọc (không sửa trực tiếp) | Không nói | Hai panel read-only | 4 | mã: `commands/index.ts:922` |
| Hàng "Chưa ghép", "Không có bên kia", alt/caption | Không vẽ | Có | 4 | mã: `panels/ReviewCopyPanel.vue:12-28`, `vi.json:926,934-935` |
| Thu hoạch thuật ngữ (hộp "Reviewer đã đổi thuật ngữ nhất quán", Nhận N / Bỏ B từng mục, "Nhận 2 mục đã chọn", "Để lại trong bảng chờ") | Hộp riêng "chạy độc lập với Review Mode" | Không có hộp riêng: ứng viên vào bảng chờ Glossary (`candidate_origin=review_harvest`); sau nhập hiện đếm `harvest_done` | 3 | mã: `ReviewerImportOverlay.vue:~118-121`, `GlossaryQueueOverlay.vue:258-272`, `reviewerImportState.ts:124-127`; nguồn: cần chốt — quyết định "harvest vào bảng chờ" chưa kiểm trong `change-*` (chưa đọc), tạm ghi loại 3 chờ xác nhận ở bước 2 |
| Tỷ lệ đổi "4/11 — không nhất quán" | Có | `harvest_inconsistent` + `changed/seen` | 1 | mã: `GlossaryQueueOverlay.vue:258-272`, `vi.json:1168-1169` |

## 8. Xuất (`export-share.html`, `export-images-attribution.html` ↔ `ExportOverlay.vue`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Tiêu đề "Xuất" + phím ⌘⇧E | Có | "Xuất bản dịch" + nút Đóng | 1 | mã: `ExportOverlay.vue:~123-130`, `vi.json:421` |
| Phạm vi: Chương này · Đã chọn · Cả Tác phẩm | Segmented, "11 Chương · 1.847 câu" | Fieldset radio 3 lựa chọn, `ex-counts` danh sách đếm (n Chương, n câu) | 1 | mã: `ExportOverlay.vue:137-166,205-216`, `vi.json:470-473,479-480`; mockup mục 1 |
| Danh sách Chương: số · tên · trạng thái "Đã xong / Đang dịch" | Có trạng thái | Chỉ nhãn Chương, không trạng thái | 2 | mã: `ExportOverlay.vue:~155-175` (`chapterLabel(row)`); mockup mục 1 danh sách |
| Định dạng: 4 lựa chọn, mỗi cái ghi nhập lại được hay không | Bốn dòng, `.docx hai cột / một khối / .md / .txt` | Bốn radio, ghi chú "nhập lại được / không" đúng ngay dưới | 1 | mã: `ExportOverlay.vue:~255-275`, `vi.json:482-489` |
| Mục "Nội dung": đánh số câu · đánh dấu câu chưa xác nhận · cột ghi chú trống · kèm 84 thuật ngữ | Bốn công tắc | Không có công tắc nào | 5 | mockup `Nội dung`; mã: không khoá i18n `export.*` tương ứng (`vi.json:468-526`) |
| Cảnh báo câu chưa xác nhận trước khi bấm | Có (khối một khối) | Có `ex-warning` cho `docx_two_column` (`unconfirmed_warning`) và cho một khối/text (`block`/`text`) | 1 | mã: `ExportOverlay.vue:~228-252`, `vi.json:481,490-493` |
| Tên tệp xuất, "Xuất 8 Chương ra .docx", "Xem trước" | Có | Chọn thư mục đích (không tên tệp), nút "Xuất"; xem trước là khối cuối | 2 | mã: `ExportOverlay.vue:~345-372`, `vi.json:513-522`; mockup `Tên file:` |
| Xem trước hình dạng file (bảng `#`, nguyên văn, bản dịch; màu giấy viết cứng) | Có, bảng nhiều hàng + câu chưa xác nhận | `ex-preview` một khối mô phỏng Word, mẫu một câu (nguồn/đích mẫu) | 2 | mã: `ExportOverlay.vue:398-415`, `vi.json:523-526`; EXPERIENCE `:481` (ngoại lệ token cho xem trước .docx) |
| Xem trước dạng `.md`, `.docx` một khối | Có | Không (một khối duy nhất) | 5 | mockup mục 2, 2b; mã: `ExportOverlay.vue:398-415` |
| Hình ảnh: "34 ảnh trong phạm vi" · Theo link gốc / Theo file ảnh | Hai tuỳ chọn có mô tả | Fieldset `ex-format data-export-images`: số ảnh, hai radio, ghi chú link không dùng được | 1 | mã: `ExportOverlay.vue:277-312`, `vi.json:494-498`; mockup `export-images-attribution.html` |
| Danh sách "6 ảnh không có link gốc" theo Chương ("Chương 4 · 2 ảnh") | Theo Chương, tổng số | Từng ảnh một ("Chương 4, ảnh thứ 2 (alt)") | 2 | mã: `ExportOverlay.vue:~300-312`, `vi.json:499-501` |
| Màu cảnh báo ảnh thiếu: `tm-rule`, không `error` | Có | `ex-warning` — màu thật: cần ảnh | 2 | mã: `ExportOverlay.vue:~296` (class `ex-warning`); mockup `.warn`; EXPERIENCE `:481`; cần ảnh |
| Khối ghi nguồn: công tắc + năm trường (Tác giả · Báo/website · URL · Ngày đăng gốc · Người dịch) hiện luôn ở màn xuất | Có, năm dòng | Chỉ công tắc + dòng Người dịch (và badge Toàn cục); không hiện bốn trường nguồn | 2 | mã: `ExportOverlay.vue:317-345`, `vi.json:502-507`; mockup `Khối ghi nguồn` |
| "Mặc định tắt. Bản dịch truyện thường không cần khối này…" | Có | Có `default_off` | 1 | mã: `vi.json:504` |
| Xem trước khối ghi nguồn trong tài liệu (Chương 3, ảnh giữ link) | Có | Không (xem trước chung không gồm khối ghi nguồn) | 5 | mockup `Xem trước — Chương 3`; mã: `ExportOverlay.vue:398-415` |
| Phím "Tab chuyển vùng · Space chọn · ⌘↵ xuất" · Huỷ · "Xuất 12 Chương" | Có | Nút "Xuất"; không ⌘↵ trong template; "Huỷ" không (chỉ Đóng) | 2 | mã: `ExportOverlay.vue:~359-364,126-130` |
| Chế độ tối cùng màn | Có | Theo token theme; màu thật: cần ảnh | 1 | mockup `Cùng màn hình, chế độ tối`; màu: cần ảnh |
| Tên người dịch đặt một lần ở Cài đặt | Có ở khối "Người dịch Toàn cục Ice" | Có trong `SettingsExportSection.vue`, màn xuất đọc lại | 1 | mã: `vi.json:508-512`, `ExportOverlay.vue:~335-345` |
| Kết quả xuất (đường dẫn, số ảnh, ảnh bỏ qua, thư mục ảnh) | Không vẽ | Có | 4 | mã: `ExportOverlay.vue:~370-395`, `vi.json:513-519` |
| Vòng khứ hồi 6 bước (mục 3 của `export-share`) | Sơ đồ giải thích | Không có trong app | 3 | nguồn: mục 3 là hình minh hoạ của mockup, không phải phần tử UI; `EXPERIENCE.md:427` ("bản dựng là minh hoạ") |

## Đếm theo loại (chỉ nhóm C, đếm các dòng bảng ở trên)

| Bề mặt | 1 | 2 | 3 | 4 | 5 | Tổng |
|---|---|---|---|---|---|---|
| 1. Bảng chờ Glossary | 2 | 6 | 0 | 3 | 4 | 15 |
| 2. Quản lý Glossary | 6 | 11 | 1 | 2 | 2 | 22 |
| 3. Quản lý TM | 4 | 5 | 0 | 3 | 5 | 17 |
| 4. Thư viện prompt | 8 | 6 | 0 | 2 | 2 | 18 |
| 5. Proofreader | 8 | 6 | 0 | 2 | 1 | 17 |
| 6. Chế độ đọc | 7 | 8 | 3 | 2 | 2 | 22 |
| 7. Review Mode | 6 | 5 | 1 | 2 | 0 | 14 |
| 8. Xuất | 8 | 7 | 1 | 1 | 3 | 20 |
| **Cộng** | **49** | **54** | **6** | **17** | **19** | **145** |

(Đếm bằng awk trên cột Loại của các bảng 1-8. Trong loại 3, một dòng ở bảng 7, "Thu hoạch thuật ngữ", chưa có nguồn xác nhận, xem ghi chú cuối.)

## Cần ảnh (Ice chụp, dpr=2, ghi dpr)

1. `GlossaryQueueOverlay`, 3 trạng thái: danh sách hàng đầu, hàng đã xử lý (✓/✕), hàng "thu hoạch từ review". Cần: bề rộng thật so với mockup 1150px, mật độ hàng, hàng đang chọn có nở không.
2. `GlossaryManageOverlay` có đủ hàng: tầng Tác phẩm, tầng Toàn cục, mục bị che, mục chờ chốt, mục chờ xác nhận xoá. Cần: bố cục flex-wrap thật, badge, màu cho "vạch lề" (mockup dùng, app không).
3. `TmManageOverlay`: dải sức khoẻ, một nhóm "nhiều bản dịch", dòng đang chọn với thanh nút dưới.
4. `PromptLibraryOverlay`: một bộ tầng Tác phẩm đang mở (có cảnh báo `glossary_terms` thiếu), màn Tạo bộ mới.
5. Proofreader: dải `ProofreaderStrip` đang hiển thị một phát hiện (xem nằm ở đâu trong cửa sổ), câu có gạch chân hai loại, `ProofreadScopeOverlay`, `ProofreadMemoryOverlay` có dữ liệu.
6. Chế độ đọc: ba mức Thoáng/Cân/Đặc sáng và tối, song ngữ bật, cuối trang (`frontier`), lớp phủ "Chỗ cần sửa" có dữ liệu, lớp phủ Mục lục. Cần: thước `ch`, hiển thị `mark-affordance` khi hover.
7. Review Mode đang mở trên một Chương đã nhập: hai panel + `review-bar`; `ReviewerImportOverlay` sau khi nhập; `AlignmentOverlay` có hàng chưa xử lý và đã nhóm.
8. `ExportOverlay`: từng định dạng (.docx hai cột, một khối, .md, .txt), có cảnh báo ảnh thiếu link, khối ghi nguồn bật, khối xem trước. Cần màu cảnh báo (`tm-rule` hay `error`?).

## Ghi chú để bước 2 và 3

- Nhiều dòng loại 5 ở TM, Glossary, Proofreader (tỷ lệ báo động giả, "đã chặn N lần", chi phí ước tính trước khi quét), Xuất (bốn công tắc Nội dung, xem trước ghi nguồn) là chức năng mockup đã ký (FR84, FR81 acceptance, FR87) nhưng app chưa có; cần mục nợ có chủ, không cần sửa bố cục.
- Một dòng xếp loại 3 (thu hoạch thuật ngữ vào bảng chờ thay hộp riêng) chưa có nguồn đọc được trong phạm vi tôi; chưa mở `change-*` nào cho nó. Bước 2 nên xác nhận nguồn (có thể `epic-cau-noi-reviewer` hoặc `epic-glossary`) trước khi giữ loại 3.
- `ProofreaderStrip` nằm ngoài dock ở `App.vue:422` trong khi mockup đặt dải dưới câu trong Editor: cần ảnh trước khi xếp loại cuối cùng.
