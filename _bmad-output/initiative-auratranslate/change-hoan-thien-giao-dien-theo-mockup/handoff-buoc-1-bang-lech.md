# Bàn giao bước 1 — bảng lệch mockup ↔ app thật

Ghi chú làm việc cho agent kế tiếp. Không phải spec; sau `bmad-correct-course` thì quyết định sống ở `change-hoan-thien-giao-dien-theo-mockup.md`.

## Mục tiêu chung (Ice, 2026-10-13)

Sắp xếp lại bố cục toàn bộ app cho sát mockup. Hiện thanh tiêu đề dồn 3 tab chế độ cùng 10 nút lối vào (`src/App.vue`, `.titlebar-act`) thành một dãy trông như 13 tab, và Library xếp mọi khối thành một cột hẹp.

**Nguyên tắc Ice ký:** giống mockup hết mức có thể; chỉ khác mockup khi mockup không có chức năng đó. Chỗ nào `DESIGN.md`/`EXPERIENCE.md` đã viết lại sau mockup thì văn bản thắng (EXPERIENCE.md §Mockups), ví dụ §Bề mặt nhập — lưới hai cột (UX-DR13), `change-auto-lookup-thu-ve-panel-source`, `change-tach-nua-giao-dien-fr50`.

## Quyết định Ice đã ký (2026-10-13)

- **D1 Thanh tiêu đề.** Chỉ còn 3 tab Library · Workspace · Chế độ đọc, cùng công tắc preset ở bên phải như `mockups/key-screen-workspace.html`. 10 lối vào kia rời thanh tiêu đề. Chỗ đặt lối vào nào mockup/IA không nói là việc của bước 2.
- **D2 Panel Tra cứu, dải bật/tắt nguồn.** Chuyển vào popup, mở từ nút `Nguồn ▾` ở đầu panel; mỗi nguồn một công tắc. Link "Nguồn dữ liệu" lên ngang hàng tiêu đề "Tra cứu". Quyết định này TRÁI `mockups/sources-attribution.html` mục 1 và FR37 ("bật/tắt từng nguồn từ điển trong panel Lookup"). Ice chọn biết rõ việc đó (phương án đối lại là một hàng chip tên ngắn theo mockup), nên correct-course phải sửa FR37 và mockup.
- **D3 Lưới hai cột.** Ẩn chữ trạng thái trên từng hàng ("đã xác nhận", "chưa dịch"); vạch lề mang trạng thái, khớp chú giải "Vạch lề" của `key-screen-workspace.html`. Trạng thái vẫn phải đọc được bằng trình đọc màn hình: không để màu là kênh duy nhất (WCAG 1.4.1).
- **D4 Hán Việt.** Bỏ dòng "Nguồn: …" trên từng câu (`panel.source.han_viet_sources_prefix` trong `src/panels/SourceHanViet.vue`). Không FR nào đòi nó; khoá vào qua `cb03974`, không qua story. Chỗ render duy nhất là `SourceHanViet.vue:845`; `StatusBar.vue:337` chỉ nhắc khoá trong chú thích. Bỏ dòng này thì nguồn Hán Việt không còn hiện ở đâu trong Workspace; bước 2 hỏi Ice có cần một chỗ hiện một lần (vd. cạnh công tắc Trung · Hán Việt) không.

- **D5 Đầu khối nguồn trong bản ghi tra cứu** (`CVDICT`, `VIETPHRASE`). Giữ, nhưng làm giống `mockups/lookup-real-density.html` ("CVDICT 9 nghĩa ⌄ thu gọn"). Ice thấy bản hiện tại quá to.
- **D6 Dòng dẫn "Có nhiều hơn một nguồn cho mục này…"** (`panel.lookup.sources_disagree`, `src/panels/LookupPanel.vue`). Bỏ hẳn, kể cả dạng chỉ hiện khi nguồn bất đồng thật; Ice xác nhận lại vì dòng này thừa. Dòng này hiện ra hễ có hơn một nguồn, kể cả khi các nguồn không bất đồng. Bỏ nó là lệch EXPERIENCE.md §Component Patterns (bản ghi từ điển: "một dòng dẫn nói rõ" khi hai nguồn ghi khác nhau) và KF-2, nên correct-course phải sửa hai chỗ đó. FR32 (hiện đồng thời, không hợp nhất) không đổi, vì các khối nguồn vẫn xếp chồng.

## Quyết định Ice ký sau bảng lệch (2026-10-13)

- **D7 Phạm vi.** Epic giao diện làm luôn chức năng loại 5 (mockup có, app chưa có; 76 dòng trong `bang-lech.md`), không ghi nợ. Dòng nào cần FR mới hoặc Rust mới thì correct-course nêu ra.
- **D8 Dải gợi ý** (TM, chốt Glossary, Proofreader). Theo mockup: mọc ở đáy panel / dưới câu đang sửa, bỏ slot chung trên thanh trạng thái (`topmostStrip`, plan Story 3.6 và 7.5).
- **D9 Nguồn Hán Việt.** Không hiện ở đâu trong Workspace sau khi bỏ dòng trên từng câu (D4); chỉ ghi công ở màn Attribution.

## Quyết định Ice ký ở bước 2 (2026-10-13)

- **D10 Cửa sổ phụ.** Quản lý Glossary, Quản lý TM, Cài đặt mở bằng phím tắt và bằng một nút `Công cụ ▾` ở góc phải thanh tiêu đề, sau công tắc bố cục; nút mở một danh sách thả xuống ghi kèm phím. Chỉ webview, giống nhau trên macOS và Windows. Đây là mục duy nhất trên thanh tiêu đề mà mockup không vẽ. 🔵 2026-10-13: Ice đổi từ menu gốc hệ điều hành sang nút này; không dùng Tauri Menu. Quản lý TM cần một phím mặc định không va `Mod+M` (gộp segment).
- **D11 Bốn lối vào chưa có chỗ** (Quét chính tả, Chỗ đã bỏ qua, Nhập bản reviewer, Nối tay). Có cả hai: chỗ theo ngữ cảnh (Quét: `⌘P` có sẵn + bộ chọn phạm vi trong dải Proofreader + trạng thái "đã quét N câu" bên phải thanh tiêu đề như `proofreader.html`; Chỗ đã bỏ qua: link trong dải Proofreader; Nhập bản reviewer: thẻ Tác phẩm và thanh hành động danh sách Chương; Nối tay: bước liền sau khi nhập + link trong Review Mode khi còn đoạn chưa nối) VÀ một mục trong `Công cụ ▾` của D10.
- **D12 Popup nguồn** (chi tiết của D2). Nút `Nguồn N/M ▾` ở đầu panel Tra cứu, cạnh link "Nguồn dữ liệu". Popup: mỗi nguồn một dòng — công tắc, tên ngắn, tên đầy đủ, số mục của chữ đang tra, phím. `⌥1…6` vẫn bật/tắt trực tiếp không cần mở popup. Escape đóng và trả tiêu điểm về nút (UX-DR17).
- **D13 Hai quyết định trong plan story làm lại theo mockup.** (a) Lịch sử tra cứu có lại bộ lọc phạm vi như `lookup-history-pins.html` (đảo Quyết định #5 của `story-lich-su-tra-cuu-va-muc-da-ghim-plan.md`). (b) Thu hoạch thuật ngữ có màn duyệt riêng như `review-mode.html` §"Thu hoạch thuật ngữ — chạy độc lập với Review Mode". Không va AD-20: ứng viên vẫn ghi vào bảng chờ; màn này chỉ là chỗ duyệt chúng, có nút "Để lại trong bảng chờ".

## Việc của bước 1

Với từng mockup trong `ux-auratranslate/mockups/` (29 tệp), đối chiếu với màn tương ứng của app thật và ghi ra `bang-lech.md` cùng thư mục này. Mỗi bề mặt một bảng, mỗi phần tử một dòng, xếp vào đúng một trong năm loại:

1. khớp
2. lệch cần sửa
3. lệch có chủ đích (trích nguồn: `change-*`, mục EXPERIENCE.md, hoặc D1–D13 ở trên)
4. app có chức năng mà mockup không vẽ (ghi rõ để bước 2 chọn chỗ)
5. mockup có mà app chưa có (ngoài phạm vi; ghi để thành mục nợ có chủ)

Kèm thêm một bảng riêng cho 10 lối vào ở thanh tiêu đề (`grep dispatch src/App.vue`): mockup hoặc §Information Architecture đặt nó ở đâu, hoặc "không có chỗ".

## Ràng buộc

- Chỉ đọc, không sửa mã, không ghi `deferred-work.md` (chỉ `grep`, tệp > 1 MB).
- Đọc mockup bằng cách bóc thẻ (`sed 's/<[^>]*>/ /g'`) hoặc mở trong trình duyệt; đừng nạp nguyên tệp HTML vào context.
- Ảnh app thật: chạy app (skill `run`) hoặc nhờ Ice chụp. Ghi dpr của ảnh; máy Ice dpr=2.
- Một dòng bảng ghi điều đã thấy, không suy luận. Chỗ nào chưa xem được thì ghi "chưa xem", đừng xếp loại.

## Sau bước 1

Agent bước 1 viết xong `bang-lech.md` thì DỪNG, báo Ice. Mỗi bước dưới đây là một phiên mới do Ice mở, đầu vào là tệp này cùng `bang-lech.md`.

2. ✅ Xong 2026-10-13: bảng lệch gộp ở `bang-lech.md`, ảnh app thật ở `anh/` (`anh/README.md`; 13 dòng "cần ảnh" chỉ tới được khi ghi dữ liệu, để story build tự chụp), quyết định D7–D13 ở trên.
3. `bmad-correct-course`: thêm epic giao diện; sửa FR37 (D2, D12), EXPERIENCE.md §Information Architecture (D1, D10, D11), §Component Patterns và KF-2 (D6); nêu FR/Rust mới mà D7 kéo theo. Ice duyệt đề xuất, rồi `bmad-ticket` cắt story.
4. `bmad-build` từng story, 4 pha. Đường lui tiêu điểm `data-*-open` (UX-DR17) đi theo nút khi dời nút; chạy một spec e2e trước khi push.

Thứ tự với Epic 10: 10.6 (ảnh hướng dẫn cài đặt) và 10.9 (NFR17, WCAG) đợi epic này; 10.5, 10.7, 10.8 không đụng bố cục.
