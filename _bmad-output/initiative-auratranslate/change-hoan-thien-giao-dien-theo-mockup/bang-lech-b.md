# Bảng lệch nhóm B — Workspace, tra cứu, gợi ý

Phạm vi: `key-screen-workspace.html` · `workspace-dark.html` · `lookup-real-density.html` · `lookup-history-pins.html` · `motion-auto-lookup.html` · `sources-attribution.html` · `tm-fuzzy-match.html` · `glossary-confirm-inline.html` · `ai-batch-translate.html` · `prompt-inspector.html`. Thanh tiêu đề thuộc agent A, không có ở đây.

Loại: 1 khớp · 2 lệch cần sửa · 3 lệch có chủ đích · 4 app có, mockup không vẽ · 5 mockup có, app chưa có.

Bằng chứng: `ảnh 1` = Workspace toàn màn, `ảnh 2` = cột nguyên văn chế độ Hán Việt, `ảnh 3` = panel Tra cứu (cả ba dpr=2, Ice chụp, nền sáng). `mã: tệp:dòng` đọc trực tiếp, không chạy app. Đường dẫn mã tính từ `src/`; mockup tính từ `ux-auratranslate/mockups/`.

Hai ghi chú đọc bảng:
- Mockup `key-screen-workspace.html` và `workspace-dark.html` vẽ bố cục 2×2 và Editor liền mạch. EXPERIENCE.md §"Bề mặt nhập — lưới hai cột đối chiếu" (UX-DR13 viết lại, Ice ký 2026-08-14) thay cả hai, nên mọi dòng "2×2" và "Editor liền mạch" là loại 3.
- Nguồn cho loại 3 chỉ lấy từ `change-*`, EXPERIENCE.md hoặc D1–D6. Chỗ mã chọn khác mockup mà nguồn chỉ là một plan story, tôi xếp loại 2 hoặc 5 và ghi plan đó vào cột App thật để Ice quyết.

## B1. Workspace — bố cục và lưới đối chiếu (`key-screen-workspace.html`, `workspace-dark.html`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Bố cục tổng | Hai panel trái (Nguyên văn trên, Bản dịch dưới) + hai panel phải (Tra cứu, Đề xuất AI) | Một panel "Đối chiếu" chiếm trái toàn chiều cao; Tra cứu trên, Đề xuất AI dưới ở bên phải (Ⓑ-2) | 3 | ảnh 1; EXPERIENCE.md §Bố cục (Ⓑ-2 mặc định, Ⓑ-1 chọn được); `change-be-mat-nhap-lat-sang-luoi-hai-cot` |
| Tiêu đề panel nhập | Hai tiêu đề riêng: "Nguyên văn" và "Bản dịch" | Một tiêu đề "Đối chiếu" | 3 | ảnh 1; mã: `i18n/vi.json:916`; EXPERIENCE.md §Hình dạng ("gộp hai panel làm một") |
| Editor liền mạch (không ô, không số câu, ranh giới `⏐` ẩn) | Văn xuôi liền, ranh giới câu chỉ hiện khi rê chuột | Mỗi câu là một hàng: vạch · số · nguyên văn · bản dịch · nhãn trạng thái | 3 | ảnh 1; EXPERIENCE.md §Hình dạng ("Mỗi câu là một HÀNG") |
| Gộp ngầm khi gõ đè lên ranh giới (chú giải "Gõ đè lên ranh giới = gộp ngầm") | Có | Gộp bằng `Backspace` đầu ô và `⌘M`; không gộp ngầm bằng gõ đè | 3 | EXPERIENCE.md §Phím (`Backspace` đầu ô); `change-rut-hoan-tac-cho-gop-tach` |
| Tab nguồn "Trung / Hán Việt" | Hai tab ở đầu panel Nguyên văn | Hai tab "Trung", "Hán Việt" ở đầu panel Đối chiếu, chỉ hiện khi nguồn là tiếng Trung | 1 | ảnh 1, ảnh 2; mã: `panels/GridPanel.vue:1727` |
| Nút "Song song" | Tab thứ ba "Song song" (lookup-history-pins) | Nút chuyển "Song song / Chuyển đổi" bên phải dải tab, chỉ hiện ở tab Hán Việt | 1 | ảnh 2; mã: `panels/GridPanel.vue:1761` |
| Chế độ Hán Việt đi theo bố cục | Hán Việt xếp dưới dòng gốc | Người dùng tự bật tắt; không tự đổi theo bố cục | 3 | EXPERIENCE.md §Hán Việt ("Người dùng tự bật tắt", Ice ký 2026-08-14) |
| Dòng "Nguồn: …" trên từng câu Hán Việt | Không vẽ | Hiện trên mọi hàng, hai dòng chữ hoa nhỏ ở đầu ô | 3 | ảnh 2 (khung đỏ); mã: `panels/SourceHanViet.vue:845`; D4 (bỏ dòng này) |
| Vạch lề mang trạng thái (đã xác nhận · đang sửa · gợi ý TM cần xác nhận) | Vạch lề, kèm chú giải "Vạch lề" | Vạch trái từng hàng; ảnh chỉ thấy vạch lục ở hàng 1 | 1 | ảnh 1; mã: `panels/GridPanel.vue:1900` (`col-rule`) |
| Nhãn chữ trạng thái trên từng hàng | Không vẽ; trạng thái đọc ở vạch lề | "đã xác nhận", "chưa dịch" ở cột phải mỗi hàng | 3 | ảnh 1 (khung đỏ); mã: `panels/GridPanel.vue:1977`; D3 (ẩn chữ, vạch mang trạng thái, giữ cho trình đọc màn hình) |
| Bộ đếm "12/34 đã xác nhận" ở đầu panel Bản dịch | Có | Chưa thấy trong ảnh và trong `i18n/vi.json` (chuỗi tiến độ chỉ có ở Library) | 5 | ảnh 1; mã: `grep "đã xác nhận" i18n/vi.json` chỉ ra `panel.grid.state_confirmed` và chuỗi xuất |
| Gợi ý phím "⌘M gộp · ⌘/ tách" ở đầu panel Bản dịch | Có | Không có dòng gợi ý phím trong panel | 5 | ảnh 1; mã: `panels/GridPanel.vue:1727-1765` (đầu panel chỉ có tab) |
| Toast "Đã gộp hai câu … Hoàn tác ⌘Z" | Hộp dưới Bản dịch có nút Hoàn tác | Thông báo gộp ở thanh trạng thái, không nút Hoàn tác; cách hoàn tác đã rút | 3 | mã: `i18n/vi.json:960` (`panel.grid.regroup_merged`); `change-rut-hoan-tac-cho-gop-tach` |
| Tô thuật ngữ Glossary: nét đứt trên nền vàng = chờ chốt, nét liền = đã chốt | Hai kiểu viền | Đã chốt = chữ màu `primary`; chờ chốt = gạch chân liền 2px, không nền vàng | 3 | mã: `panels/GridPanel.vue:2308-2318`; `change-tach-nua-giao-dien-fr50.md:170` ("kiểu gạch chân, không `opacity`") |
| Vùng chọn tra cứu | Bôi đen câu nguồn | Bôi xanh trong ô nguyên văn ("协奏曲") | 1 | ảnh 1 |
| Khe 2px giữa panel ở nền tối thay đường kẻ (`workspace-dark`) | Khe nền lộ ra | Token `--panel-gap` 0px sáng, 2px tối; `--dv-sash-color` = nền | 1 | mã: `layout/dockview-theme.css:55-63,142-144`; chưa có ảnh nền tối, xem danh sách "cần ảnh" |
| Ba vạch trạng thái ở Editor không chụm (vàng-lục · lam · hổ phách) | Ghi chú trong `workspace-dark` | Chưa có ảnh nền tối để đối chiếu màu | — | cần ảnh |
| Chuyển bố cục "2×2 ⌘1 · 4 cột ⌘2" | Công tắc preset ở thanh tiêu đề | Thuộc thanh tiêu đề | — | agent A, ngoài phạm vi |
| Thanh ngưỡng hẹp "narrow-notice", Review Mode đóng mở trong cùng khung | Không vẽ | Dải thông báo trên dock và ngăn Review | 4 | mã: `modes/WorkspaceMode.vue:125-160` |
| Ngăn kéo Tra cứu khi panel Tra cứu bị nhường chỗ | Không vẽ | Nút "Mở Tra cứu" ở thanh trạng thái và `LookupDrawer` | 4 | mã: `StatusBar.vue:464-478`; `layout/LookupDrawer.vue` |

## B2. Panel Tra cứu — đầu panel, nguồn, tab (`sources-attribution.html` §1, `lookup-real-density.html`, `lookup-history-pins.html`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Tiêu đề "Tra cứu" kèm gợi ý phím F3 | "Tra cứu F3" | "Tra cứu", không hiện F3 | 5 | ảnh 1, ảnh 3 |
| Dải bật/tắt nguồn dưới dạng hàng chip có số mục (`CVDICT 4`, `Unihan 1` …) | Hàng chip ngay dưới tiêu đề, "Bấm để tắt · ⌥1…6" | Dải "Nguồn" gồm chip tên dài (CC-CEDICT gạch ngang khi tắt), cuộn dọc trong khung 52px, cắt ở dòng thứ ba | 3 | ảnh 1 (khung đỏ trên); mã: `panels/LookupPanel.vue:384-440`; D2 (chuyển vào popup `Nguồn ▾`; mỗi nguồn một công tắc) |
| Chip hiện số mục từng nguồn ở dải bật/tắt | Có (`CVDICT 4`) | Dải bật/tắt chỉ có tên; số mục nằm ở thanh nhịp | 3 | mã: `panels/LookupPanel.vue:408-420`; D2 (dải này rời đi) |
| Chú giải "Bấm để tắt · ⌥1…6" | Có | Gợi ý nằm trong `title` của chip, không hiện chữ | 3 | mã: `panels/LookupPanel.vue:416`; D2 |
| Link "Nguồn dữ liệu" | Mockup không đặt link trong panel; Attribution mở từ "Giới thiệu" | Nút "Nguồn dữ liệu" cuối dải chip, ngoài vùng cuộn | 3 | ảnh 1; mã: `panels/LookupPanel.vue:430-437`; D2 (nâng lên ngang tiêu đề "Tra cứu") |
| Ba tab Từ điển · Concordance · Lịch sử | Có, đúng thứ tự | Có, đủ ba, có `tablist` đầy đủ | 1 | ảnh 1, ảnh 3; mã: `panels/LookupPanel.vue:454-495` |
| Đầu mục + thanh nhịp "N nguồn · M nghĩa" + chip nguồn kèm số nghĩa | Có | Có: "2 nguồn · 3 nghĩa  CVDICT 1  VIETPHRASE 2" | 1 | ảnh 3; mã: `panels/LookupPanel.vue:511-545` |
| Phiên âm cạnh đầu mục (`zǒuláng`, `tẩu lang`) | Có ở `key-screen`, `sources-attribution` | Không có; không chuỗi `reading` nào trong `LookupPanel`/`LookupRecord` | 5 | ảnh 3; plan `story-panel-lookup-ban-ghi-co-cau-truc-plan.md:376` ("đòi `dict_entry.reading` chưa qua dây") |
| Dải âm đọc "dǎ · đả — 21 nghĩa / dá · tá — 1 nghĩa" và câu giải thích hai âm | Có ở `lookup-real-density` | Không có | 5 | mã: grep `reading` rỗng; plan dòng 376 (hoãn, không FR đòi) |
| Nút "📌 Ghim ⌘D" ở đầu panel cạnh đầu mục | Một nút ở hàng đầu mục | Nút "Ghim" riêng cho từng cụm mục từ, căn phải trong bản ghi | 2 | ảnh 3 (chữ "Ghim" bên phải "bản concerto"); mã: `panels/LookupRecord.vue:99-117` |
| Chip lọc từ loại ("Tất cả · động từ 17 · giới từ 2 …") | Có | Không có | 5 | mã: grep; plan `story-panel-lookup-ban-ghi-co-cau-truc-plan.md:378` ("không FR nào đòi") |
| Thanh nhịp bị cắt im lặng khi nhiều nguồn | Không vẽ chỗ tràn | `.lookup-head` cao 76px, `overflow: hidden`; chip thừa bị cắt | 2 | mã: `panels/LookupPanel.vue:1070-1074`; `deferred-work.md:912` (nợ đã ghi) |
| Dòng dẫn "Có nhiều hơn một nguồn cho mục này — đối chiếu từng khối để tự chọn." | Mockup `key-screen`/`sources-attribution` vẽ "Hai nguồn ghi khác nhau — cả hai đều được giữ nguyên", chỉ khi bất đồng | Hiện hễ có hơn một nguồn (ảnh: màu vàng nâu, vạch trái) | 3 | ảnh 3 (khung đỏ); mã: `panels/LookupPanel.vue:637`; D6 (bỏ hẳn) |
| Bộ lọc phạm vi trong tab Lịch sử "Phiên này / Chương 47 / Cả Tác phẩm" | Có | Không có; chỉ còn nút "Xoá lịch sử phiên" | 3 | mã: `panels/LookupPanel.vue:754-765`; plan `story-lich-su-tra-cuu-va-muc-da-ghim-plan.md:116` (Quyết định #5). Nguồn là plan story, không nằm trong ba nguồn của handoff: Ice nên xác nhận |
| Tab "Concordance" | Có tên tab, không vẽ nội dung | Danh sách khớp, trạng thái rỗng, cắt bớt, con trỏ sang Concordance từ "không tìm thấy" | 4 | mã: `panels/LookupPanel.vue:649-700` |
| Trạng thái "đã tắt hết nguồn", "không có lớp từ điển", "truy vấn quá ngắn", "lỗi nạp lớp" | Không vẽ | Có câu riêng cho từng trạng thái | 4 | mã: `panels/LookupPanel.vue:591-634` |

## B3. Bản ghi tra cứu (`lookup-real-density.html`, `key-screen-workspace.html`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Đầu khối nguồn | "CVDICT 9 nghĩa ⌄ thu gọn", một dòng nhỏ có mũi tên | Tên nguồn chữ hoa `primary` trên đường kẻ dưới, cỡ `ui-label`, không số nghĩa, không mũi tên | 2 | ảnh 1, ảnh 3 (khung đỏ "CVDICT", "VIETPHRASE"); mã: `panels/LookupRecord.vue:86-90,184-192`; D5 (giữ, làm giống mockup) |
| Thu gọn/mở khối nguồn ("còn 6 nghĩa nữa ⌄", "CC-CEDICT 4 nghĩa › mở ra" một dòng tóm tắt) | Có, nguồn thu gọn vẫn hé một dòng | Khối nguồn luôn mở đủ; không dòng "còn N nghĩa nữa" | 5 | mã: `panels/LookupRecord.vue:121-148`; plan `story-panel-lookup-ban-ghi-co-cau-truc-plan.md:265` (hứa dòng "còn M nữa", chưa dựng). D5 chỉ nói đầu khối, chưa nói thu gọn: Sally chốt |
| Hàng nghĩa: từ loại · nghĩa · ví dụ · trích dẫn | Có | Có đủ: `lookup-pos`, `lookup-gloss`, `lookup-example`, `lookup-citation`, `lookup-note` | 1 | mã: `panels/LookupRecord.vue:121-148` |
| Cờ nhãn ngoại ngữ (FR35) | Có ở "Đã gỡ lớp HVTĐTD" (§3 sources) | Cờ `pos_is_foreign` hiện cạnh từ loại, ví dụ | 1 | mã: `panels/LookupRecord.vue:126-130,139-141` |
| Vạch trái 2px theo từng nghĩa | Có | Vạch trái từng nghĩa, ảnh thấy ở "bản concerto" | 1 | ảnh 3; mã: `panels/LookupRecord.vue:194-200` |
| Cụm mục từ (đầu mục con khi một nguồn có nhiều mục) | Không vẽ | `lookup-entry-headword` khi nguồn có nhiều mục | 4 | mã: `panels/LookupRecord.vue:99-101` |
| Hàng chip "5 nguồn · 22 nghĩa" chia theo từ loại bên trong từng nguồn | Có | Không lọc từ loại | 5 | xem B2, dòng "Chip lọc từ loại" |

## B4. Tab Lịch sử và mục đã ghim (`lookup-history-pins.html`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Hai vùng: "Đã ghim N" rồi "Tra gần đây" trong một danh sách | Có | Có, tiêu đề vùng "Đã ghim" với số đếm, "Tra gần đây" | 1 | mã: `panels/LookupPanel.vue:707-760` |
| Hàng ghim: từ · nghĩa · nguồn · số lần | Có | Từ · nghĩa · nhãn nguồn · số lần tra trong phiên · nút "Bỏ ghim" | 1 | mã: `panels/LookupPanel.vue:733-751` |
| Hàng lịch sử: từ · nghĩa · "N ph" | Có | Có; nhãn tương đối dựng từ khoá `vi.json` | 1 | mã: `panels/LookupPanel.vue:781-797` |
| Tra lại mục có sẵn đẩy lên đầu, không tạo dòng trùng | Có | `:key` = `query`, đúng khoá dedupe | 1 | mã: `panels/LookupPanel.vue:782` |
| Dòng "Ghim là của người, lịch sử là của máy …" | Có | Có, câu chữ đổi: "đi theo mọi Tác phẩm" thay cho "theo Tác phẩm" | 3 | mã: `i18n/vi.json:1049`; ghim ở `global.db`, plan `story-lich-su-tra-cuu-va-muc-da-ghim-plan.md` (Ice ký lại 2026-08-11, theo chú thích `panels/LookupPanel.vue:714`) |
| Hai trạng thái rỗng (lịch sử, ghim) | Có | Có, thêm hai ca mockup không vẽ: nạp ghim lỗi, đang nạp | 1 | mã: `panels/LookupPanel.vue:722-730,768-772` |
| Nút "Xoá lịch sử phiên ⌘⌫" | Có phím ⌘⌫ | Có nút; không gợi ý phím hiển thị | 5 | mã: `panels/LookupPanel.vue:763`; mockup chú giải phím F3 ⌘D ⌘⌫ |

## B5. Chuyển động Auto-Lookup (`motion-auto-lookup.html`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Vùng đầu mục cao cố định | Có | `.lookup-head` 76px, render ở mọi trạng thái, ngoài vùng cuộn | 1 | mã: `panels/LookupPanel.vue:511,1070-1074` |
| Nội dung mới vào 90ms, opacity 0.4 → 1, ease-out, không translate/scale | Có | `animation: lookup-fade-in 90ms ease-out`, `from` 0.4 | 1 | mã: `panels/LookupPanel.vue:961-995` |
| Không hiệu ứng ra; không xếp hàng; tra mới huỷ hiệu ứng | Có | Cờ `fading` huỷ khi tra mới, hẹn giờ thay `animationend` | 1 | mã: `panels/LookupPanel.vue:224-240` |
| Cuộn về đầu tức thì | Có | `scrollTop = 0`, không `scroll-behavior` | 1 | mã: `panels/LookupPanel.vue:321-349` |
| Vạch tiến trình mảnh sau 250ms, không spinner | Có | `.lookup-progress` tuyệt đối ở đáy `.lookup-head` | 1 | mã: `panels/LookupPanel.vue:555,1026` |
| `prefers-reduced-motion` bỏ hiệu ứng | Có | `animation: none; opacity: 1` | 1 | mã: `panels/LookupPanel.vue:1039-1058` |
| Không tra khi con trỏ còn kéo; trạng thái rỗng cùng 90ms | Có | Chỉ đọc mã gián tiếp (hợp đồng vùng chọn); chưa kiểm hành vi chạy thật | — | cần ảnh/quay màn hình nếu Ice muốn kiểm hành vi |

## B6. Màn hình Attribution (`sources-attribution.html` §2, §3)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Khung màn hình | Một màn hình đầy đủ có dòng "Giới thiệu · Nguồn dữ liệu" | Hộp thoại phủ `attr-scrim` + `attr-panel` | 2 | mã: `AttributionOverlay.vue:196-200`. ảnh: anh/attribution-nguon-du-lieu-dau.png (khung viền chiếm ~86% bề ngang và gần hết chiều cao, cuộn trong khung) |
| Tiêu đề "Nguồn dữ liệu và ghi công" + đoạn dẫn | Có | Có, câu chữ bớt viện dẫn FR | 1 | mã: `i18n/vi.json:1054-1055` |
| Bảng: Nguồn · Giấy phép · Vai trò · Ghi chú xuất xứ | Bốn cột, cột nhóm "Lớp" gộp vào Giấy phép | Bốn cột: Nguồn · Giấy phép · Vai trò · Ghi công; nhãn "nền / gỡ rời" nằm ở cột Vai trò | 1 | mã: `AttributionOverlay.vue:237-250`; `i18n/vi.json:1056-1061` |
| Phân loại giấy phép: mở · phạm vi công cộng · còn bản quyền · phép riêng tác giả · không rõ | Có | Có đủ năm nhãn + "chưa phân loại" | 1 | mã: `i18n/vi.json:1080-1085` |
| Nút "Mở văn bản giấy phép", "Sao chép bản ghi công" | Hai nút ở chân bảng, dùng chung cả bảng | Hai nút trên từng hàng nguồn | 2 | mã: `AttributionOverlay.vue:306-322`. Mockup đặt một cặp nút ở chân; app có thể chọn khác có lý do (cần Sally) |
| Văn bản giấy phép mở ngay trên màn hình | Không vẽ khung xem | Khung `attr-license-view` với `<pre>` và nút đóng | 4 | mã: `AttributionOverlay.vue:340-350` |
| Hai thẻ "Giấy phép của phần mềm", "Nếu bạn là chủ sở hữu một nguồn ở đây" | Có | Có hai thẻ `attr-card`, câu chữ rút gọn | 1 | mã: `AttributionOverlay.vue:353-370`; `i18n/vi.json:1075-1079` |
| Dòng "Tắt chỉ giấu nguồn khỏi kết quả — dữ liệu vẫn nằm trong bản cài" | Có | Có (`attribution.off_is_not_removed`) | 1 | mã: `i18n/vi.json:1063` |
| Nhãn "đang tắt" ngay trong bảng ghi công | Không vẽ | Có (`attr-off`) | 4 | mã: `AttributionOverlay.vue` (`attribution.state_off`) |
| Thông báo "đọc một phần", "nạp lỗi", "bản cài trống" | Không vẽ | Có | 4 | mã: `i18n/vi.json:1064-1066` |
| Minh hoạ "Gỡ lớp thì chuyện gì xảy ra" (§3: HVTĐTD có / đã gỡ) | Có | Không có trong app; là hình minh hoạ chứng minh FR36, không phải bề mặt | 5 | mockup §3; chưa thấy chỗ tương ứng trong `AttributionOverlay.vue` (ngoài phạm vi sản phẩm nếu Ice xác nhận) |
| Ghi chú "Tab để duyệt bảng", "Toàn bộ giấy phép kèm trong bản phát hành" | Có | Không thấy | 5 | mã: `AttributionOverlay.vue` không có dòng này |

## B7. Dải khớp mờ TM (`tm-fuzzy-match.html`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Vị trí dải | Mọc dưới câu đang sửa, trong cột Bản dịch | Slot chung toàn bề ngang ngay trên thanh trạng thái, cùng slot với dải Glossary | 2 | mã: `TmFuzzyStrip.vue:1-3`; `App.vue:424-425`. Nguồn quyết định chỉ là plan Story 7.5; chưa có `change-*` hay mục EXPERIENCE.md |
| Tiêu đề "Translation Memory — 3 câu gần giống" + gợi ý phím "⌥1 ⌥2 ⌥3 nhận · Esc ẩn dải" | Có | Tiêu đề "Khớp mờ trong bộ nhớ dịch"; gợi ý "↑ ↓ chọn · Enter hoặc 1–3 nhận · Esc ẩn" | 2 | mã: `TmFuzzyStrip.vue:106-110`; `i18n/vi.json:1195-1196` (phím nhận khác `⌥1…3`) |
| Hàng gợi ý: "92% khớp" | Có | `{percent}%` ở đầu hàng | 1 | mã: `TmFuzzyStrip.vue:187`; `i18n/vi.json:1197` |
| Diff ở câu nguồn (xoá/thêm tô nền) | Có | `<del>`/`<ins>` ở `tmf-source` với token `diff-add-*`, `diff-del-*` | 1 | mã: `TmFuzzyStrip.vue:188-194`; plan `story-khop-mo-plan.md:31` |
| Diff ở câu đích | Có ("Diff nằm ở cả hai phía, và đó là điểm chính") | `tmf-target` là văn bản trơn, không diff | 5 | mã: `TmFuzzyStrip.vue:196`; plan `story-khop-mo-plan.md:45` ("old target" không diff) |
| Dòng xuất xứ: "Chương 12 · câu 44 · văn phong của tôi · 14-07" | Có | Chỉ "Của tôi/Của người khác" và "Tác phẩm này/Toàn cục", không chương, không ngày | 2 | mã: `TmFuzzyStrip.vue:198-205`; `i18n/vi.json:1199-1202` (hàng khớp 100% có ngày qua `historyTimeLabel`) |
| Nút "Nhận" và "Xem cả câu" | Hai nút | Một nút "Nhận N"; không "Xem cả câu" | 5 | mã: `TmFuzzyStrip.vue:207-216`; `i18n/vi.json:1198` |
| Khớp 100% dùng dải riêng "Khớp tuyệt đối" | Mockup nói khớp 100% điền sẵn, không mọc dải | Có danh sách `tmf-exact-list` với "in use" và xác nhận ghi đè | 4 | mã: `TmFuzzyStrip.vue:35-70`; `i18n/vi.json` (`tm.exact.*`) |
| Hộp xác nhận ghi đè bản đang soạn | Không vẽ | `tmf-overwrite` với hai nút | 4 | mã: `TmFuzzyStrip.vue:22-32` |
| Thứ tự ưu tiên: dải Glossary thắng dải TM | Có | `topmostStrip` cho Glossary đứng trước, TM nhường | 1 | mã: `panels/inlineStripPriority.ts:32-40` |
| Dòng chân "TM: 12.409 cặp · 11.882 của tôi · 527 của người khác · Ngưỡng 65%…" | Có | Không thấy | 5 | mã: `TmFuzzyStrip.vue` không có dòng chân |
| §3 Thuật toán theo ngôn ngữ (n-gram ký tự; token + stemming) | Mockup giải thích | Có hàm điểm theo `MatchLang` ở Rust | 1 | plan `story-khop-mo-plan.md:39` (không phải bề mặt giao diện) |

## B8. Dải chốt bản dịch Glossary (`glossary-confirm-inline.html`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Vị trí dải | Mọc ở đáy panel, con trỏ vẫn ở Editor | Slot chung toàn bề ngang trên thanh trạng thái, dưới `GlossaryQuickAdd` | 2 | mã: `GlossaryConfirmStrip.vue:3-6`; `App.vue:416-420`. Cần Ice quyết "đáy panel" hay slot toàn chiều ngang |
| Tiêu đề "聽潮閣 — chờ chốt bản dịch · xuất hiện 644 lần trong Tác phẩm" | Có | "Chờ chốt bản dịch: {thuật ngữ}", nhãn tầng, "Xuất hiện N lần" | 1 | mã: `GlossaryConfirmStrip.vue:104-125`; `i18n/vi.json:1153,1161` |
| Dòng "xuất xứ: đề xuất khi nhập tài liệu · phân loại: địa danh" | Có | Không có | 5 | mã: `GlossaryConfirmStrip.vue:104-165` |
| Danh sách ba đề xuất: "↵ bản dịch bạn vừa viết", "2 âm Hán Việt", "3 Gác Nghe Sóng (TM · 1 lần)" | Có, chọn bằng phím số | Một ô nhập có điền sẵn và nhãn "Âm Hán Việt"; không có đề xuất từ bản đang gõ, không từ TM | 5 | mã: `GlossaryConfirmStrip.vue:120-145`; `i18n/vi.json:1159` |
| "gõ cách khác…" | Có | Ô nhập là chính nó, luôn mở | 3 | EXPERIENCE.md không nói; chưa có nguồn cho lựa chọn ô nhập. Xếp 3 chỉ vì ô nhập bao hàm chức năng; Sally xác nhận |
| Nút "Lưu" và "Để sau" ("Esc để sau — không hỏi lại trong Chương này") | "Esc để sau" làm hành động duy nhất | Hai nút "Lưu", "Để sau" | 1 | mã: `GlossaryConfirmStrip.vue:167-173`; `i18n/vi.json:1155-1156,389` |
| Đoạn "Chốt xong thì Smart RAG Injector bắt đầu ép AI…" | Có | Không thấy trong dải | 5 | mã: `GlossaryConfirmStrip.vue` (không có dòng này) |
| Sau khi chốt: thông báo "Đã chốt … ⌘Z hoàn tác" | Có | Thông báo chốt và trạng thái lưu có; hoàn tác ⌘Z chưa thấy | 5 | mã: `GlossaryConfirmStrip.vue:150-165` (trạng thái lưu/lỗi); không có nút hoàn tác |
| Panel AI báo "Đã chèn 3 thuật ngữ Glossary — tăng 1 so với câu trước" | Có | Có dòng tóm tắt prompt nhưng không có "tăng 1 so với câu trước" | 5 | mã: `i18n/vi.json:158-160` |
| Trạng thái "chưa cài dữ liệu từ điển — không có đề xuất" | Không vẽ | Có | 4 | mã: `GlossaryConfirmStrip.vue:155-165`; `i18n/vi.json:1160` |
| Đếm "Glossary 214 mục · 3 chờ chốt" ở thanh tiêu đề | Có | Thuộc thanh tiêu đề | — | agent A, ngoài phạm vi |

## B9. Đề xuất AI và dịch theo lô (`key-screen-workspace.html`, `ai-batch-translate.html`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Đầu panel "Đề xuất AI ⌘↵" + tên nhà cung cấp và mô hình ("Ollama · qwen2.5:14b ⌘T") | Có | "Đề xuất AI" thuần; không tên mô hình, không phím | 5 | ảnh 1; mã: `panels/AiTranslationPanel.vue:340-362` (không chuỗi nhà cung cấp trong template) |
| Chưa cấu hình AI: chỉ mời cấu hình, không cảnh báo | Có | Câu "Chưa có nhà cung cấp AI nào được cấu hình — mọi năng lực khác vẫn chạy đầy đủ…" | 1 | ảnh 1; mã: `i18n/vi.json:1005` |
| Thanh bộ prompt: chọn bộ, "Thư viện prompt" | Không vẽ ở panel | `ai-prompt-bar` với `<select>` và nút "Thư viện prompt"; ảnh thấy "Chưa có bộ prompt nào…" | 4 | ảnh 1; mã: `panels/AiTranslationPanel.vue:340-361` |
| "Xem prompt" cạnh tiêu đề | Link nhỏ ở đầu panel | Thanh riêng: "Lắp prompt cho câu này", "Xem prompt", tóm tắt prompt, ghi chú cũ | 2 | ảnh 1; mã: `panels/AiTranslationPanel.vue:362-405` |
| Văn bản đề xuất đang sinh, con trỏ nhấp nháy, "Câu đang sửa · đang sinh…" | Có | `ai-translate-text` + "Đang sinh…" | 1 | mã: `panels/AiTranslationPanel.vue:517`; `i18n/vi.json:1014` |
| Dòng "412 token · ước tính ~0,004 USD" mỗi lần gọi | Có | `ai-translate-usage` (token + ước tính, hoặc "chưa có giá") | 1 | mã: `panels/AiTranslationPanel.vue:523`; `i18n/vi.json:104-106` |
| Nút điều khiển: "⌘⇧↵ đưa sang bản dịch · Esc huỷ" | Chú thích phím một dòng | Bốn nút văn bản trần: "Dịch câu này", "Huỷ dịch", "Đưa sang bản dịch", "Thử lại" | 2 | ảnh 1 (các link xám trong panel); mã: `panels/AiTranslationPanel.vue:410-450`; chưa thấy gợi ý phím ngay tại nút |
| Chọn câu rồi mới dịch lô ("12 câu đang chọn ⇧↓") | Có | "Đã chọn N câu" + nút "Dịch các câu đã chọn" | 1 | mã: `panels/AiTranslationPanel.vue:537-560`; `i18n/vi.json:1016` |
| Tiến độ lô: "Lô 12 câu · đã xong 5 · đang câu 6 · còn 6" | Có | "Đã xong {x}/{y} · còn {z} câu chưa tới lượt", "Đang dịch câu số N" | 1 | mã: `i18n/vi.json:1017-1018` |
| Danh sách từng câu của lô: số câu · bản dịch rút gọn · token · trạng thái | Mỗi hàng có đoạn dịch và "218 tk" | Mỗi hàng chỉ số segment và nhãn trạng thái; không có văn bản, không token | 5 | mã: `panels/AiTranslationPanel.vue:595-606`; `i18n/vi.json:1019-1024` |
| Tổng token và chi phí của lô | "2.132 token · ~0,021 USD cho lô này" | `ai-batch-usage` ("Tổng N token · ước tính ~X USD cho M câu") | 1 | mã: `i18n/vi.json:151-154` |
| "Huỷ lô Esc" giữ phần đã xong | Có | Có huỷ; chưa tách riêng "Huỷ lô" khỏi "Huỷ dịch" trong nhãn | 2 | mã: `panels/AiTranslationPanel.vue:410-430`; chưa có nhãn riêng cho lô trong `i18n/vi.json:593` |
| "Đưa 5 câu đã xong sang Bản dịch" và phím `⌘⇧⌥↵` cho cả lô đã duyệt | Có | Không có lệnh đưa cả lô sang | 5 | mã: grep `batch_promote` không có trong `i18n/vi.json`; `panels/AiTranslationPanel.vue:537-606` |
| Khối lỗi giữa lô: "Lô dừng ở câu 6 / 1–5 còn nguyên / 6 không phản hồi / 7–12 chưa gọi, chưa tính phí" | Có | Một cảnh báo lỗi + nút "Thử lại các câu chưa xong"; không bảng ba dòng | 2 | mã: `panels/AiTranslationPanel.vue:575-594`; `i18n/vi.json:599` |
| Không thử lại tự động; lỗi không chạm Bản dịch | Có | Không có vòng thử lại tự động trong panel; Bản dịch không đổi | 1 | mã: `panels/AiTranslationPanel.vue` (chỉ có nút thủ công) |
| "Bỏ lô" | Có | Không có nút riêng | 5 | mã: `panels/AiTranslationPanel.vue:575-594` |
| Bảng token và chi phí cả phiên (§3), cộng dồn "Phiên này: 18.640 token · ~0,18 USD" ở thanh trạng thái | Có | Không có bảng, thanh trạng thái không có token | 5 | mã: `StatusBar.vue:455-460` (chỉ "Đã lưu"); `i18n/vi.json:269` |
| Hộp xác nhận ghi đè khi bản đang soạn chưa từng xác nhận | Không vẽ | `ai-promote-confirm` | 4 | mã: `panels/AiTranslationPanel.vue`; `i18n/vi.json:1121-1124` |
| Cảnh báo "Kết quả này được dịch cho câu số X — câu đang xem là Y" | Không vẽ | Có | 4 | mã: `i18n/vi.json:155` |

## B10. Xem prompt cuối cùng đã gửi (`prompt-inspector.html`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Khung | Màn hình đầy đủ, Esc đóng, hai tab "Kết quả / Prompt đã gửi" | Hộp thoại phủ `aip-scrim`/`aip-panel` | 2 | mã: `AiPromptInspectorOverlay.vue:149`. ảnh: anh/xem-prompt-chua-lap.png (khung ~68% bề ngang, không có tab Kết quả / Prompt đã gửi; mới thấy trạng thái chưa lắp ráp) |
| Dòng nhận diện: "gửi lúc 14:02:37 · mô hình claude-sonnet-5 · lắp bởi RagInjector" | Có | "Câu số N · Chương M · Bộ prompt “tên” (tầng)"; không giờ gửi, không mô hình | 2 | mã: `AiPromptInspectorOverlay.vue:180`; `i18n/vi.json:162` |
| Prompt tách theo tầng: hệ thống (Global), thể loại (Tác phẩm, "ghi đè Global"), Glossary chèn động, TM chèn động, câu cần dịch; mỗi tầng một nhãn và số token | Có, tô màu theo tầng | Một khối `<pre>` liền ("Nội dung đã gửi") có tô nền cho mảnh Glossary; không nhãn tầng, không số token | 2 | mã: `AiPromptInspectorOverlay.vue:202-217,444-460` |
| Danh sách thuật ngữ đã chèn kèm tầng, TM đã chèn kèm phần trăm | Có | Có: nguồn → bản dịch, nhãn tầng, phần trăm TM, thẻ "tham chiếu" | 1 | mã: `AiPromptInspectorOverlay.vue:234-290`; `i18n/vi.json` (`ai.prompt_inspector.tm_percent`) |
| "Đã cân nhắc nhưng không chèn": thuật ngữ chờ chốt, thuật ngữ bị tầng khác ghi đè, câu TM dưới ngưỡng | Ba loại, mỗi dòng một lý do | Chỉ một loại: "bị chặn vì chờ chốt/đè nhau" (`suppressed_by_pending_overlap`); không TM bị loại, không bị tầng khác thắng | 2 | mã: `AiPromptInspectorOverlay.vue:262-282` |
| Lối sửa ngay trên dòng bị loại: "Chốt ngay ⌘K", "Xem mục", "Xem cả 4" | Có | Không có nút | 5 | mã: `AiPromptInspectorOverlay.vue:262-282` |
| Khối "Ngân sách token" (thanh chia tầng, "412 token", "38,8% chèn động", "~0,004 USD") | Có | Không có | 5 | mã: `AiPromptInspectorOverlay.vue:149-292` không có phần ngân sách |
| Ba hành động: "Chép toàn bộ prompt", "Gửi lại với prompt này", "Mở prompt Tác phẩm để sửa" | Có | Không có (chỉ nút "Đóng") | 5 | mã: `AiPromptInspectorOverlay.vue:155-160` |
| Trạng thái "chưa lắp ráp", "lỗi đọc", "bản ghi cũ so với câu hiện tại", "thiếu câu nguồn", "mốc lạ" | Không vẽ | Có đầy đủ | 4 | mã: `AiPromptInspectorOverlay.vue:162-200`; `i18n/vi.json:156,162-166` |
| Tab "Kết quả" (kết quả AI cạnh prompt) | Có tab | Không có tab; chỉ prompt | 5 | mã: `AiPromptInspectorOverlay.vue:149-292` |

## B11. Thanh trạng thái ở chân (`key-screen-workspace.html`, `ai-batch-translate.html`)

| Phần tử | Mockup | App thật | Loại | Bằng chứng |
|---|---|---|---|---|
| Chiều cao 32px | 32px | 34px theo token | 3 | mã: `StatusBar.vue:488,505` (đã ghi phân xử) ; `deferred-work.md:2298` |
| "Đã lưu 2 giây trước" | Có | Có | 1 | mã: `StatusBar.vue:457`; `i18n/vi.json:269` |
| "Tra cứu ngoại tuyến · Glossary 214 mục · TM 8.412 cặp" | Có | Không có; thanh chỉ có "Đã lưu" và các thông báo | 5 | ảnh 1 (chân màn trống); mã: `StatusBar.vue:396-460` |
| "Chương 47 · câu 6/184" | Có (ai-batch) | Không thấy | 5 | mã: `StatusBar.vue:396-460` |
| "Phiên này: N token · ~X USD" | Có | Không có | 5 | xem B9 |
| Thông báo ưu tiên: lỗi xác nhận, gộp/tách, tách Chương, điều hướng, rê thuật ngữ | Không vẽ | Có chuỗi `v-if` năm nhánh | 4 | mã: `StatusBar.vue:397-455` |
| Nút mở ngăn kéo Tra cứu khi panel bị nhường chỗ | Không vẽ | Có | 4 | mã: `StatusBar.vue:464-478` |
| Dòng "Nguồn:" Hán Việt ở thanh trạng thái | Không vẽ | D4 nói `StatusBar` đã hiện tiền tố này một lần; trong mã `StatusBar.vue` chỉ nhắc khoá đó ở một chú thích (dòng 337), không render nó | — | mã: `StatusBar.vue:337`; grep `han_viet_sources_prefix` chỉ render ở `panels/SourceHanViet.vue:845`. Tiền đề của D4 cần Ice kiểm: bỏ dòng ở từng câu thì không còn chỗ nào hiện nguồn Hán Việt |

## Danh sách cần ảnh

Ice chụp (dpr=2, nền sáng nếu không ghi khác):
1. Workspace nền tối, cùng nội dung với ảnh 1. Để đối chiếu khe 2px giữa panel, ba vạch trạng thái và màu `workspace-dark.html`.
2. Panel Tra cứu, tab Lịch sử, có cả mục ghim và lịch sử. Để so với `lookup-history-pins.html`.
3. Panel Tra cứu sau khi tra một chữ nhiều nghĩa, nhiều nguồn (ví dụ 打). Để so khối nguồn, thanh nhịp và chỗ cắt chip với `lookup-real-density.html`.
4. Màn hình Attribution đang mở (bảng nguồn, thẻ pháp lý). Để chốt mức lệch hộp thoại và màn đầy đủ.
5. Dải khớp mờ TM đang mọc (khớp từ 65% đến dưới 100%), và dải khớp 100%. Để so vị trí và diff.
6. Dải chốt bản dịch Glossary đang mở. Để so vị trí dải và ô nhập.
7. Panel Đề xuất AI với AI đã cấu hình, đang sinh một câu, rồi sau một lô đã chạy dở có lỗi. Để so hàng lô và khối lỗi.
8. Màn hình Xem prompt cuối cùng đã gửi, khi có prompt đủ Glossary và TM. Để so cách tách tầng và phần "không chèn".
9. Thanh trạng thái ở chân khi vừa lưu xong, có và không có thông báo gộp. Để kiểm D4.
10. Panel Đối chiếu ở nền tối, một hàng đang sửa và một hàng "TM điền sẵn cần xác nhận". Để kiểm màu vạch lề.
