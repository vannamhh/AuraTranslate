# Bàn đo 14-11: phím không lọt xuống ô bản dịch sau lớp phủ mở bằng chuột

Bản dựng: binary debug `cargo build --locked --features wdio` với `TAURI_CONFIG` đặt `devUrl` về Vite riêng ở cổng 1421 (khởi động lại trước mỗi lượt), WKWebView trên máy Ice, 2026-11-01, mã vòng 2. Khuôn của `14-10-ban-do`: chuột thật `cliclick` (rê chuột trước, giữ nút 90 ms, `ensureFront()` trước mỗi cú), phím `Q`/`zz` qua `cliclick t:`, `Esc` và hợp âm qua `osascript`; wdio chỉ đọc trạng thái. Cửa sổ 2600x1800. Chạy từng tệp bằng cấu hình tạm ngoài cây (`specs` ghi đè, `e2e/wdio.conf.mjs` không sửa) vì `--spec` không thay `specs` của cấu hình: `ONE_SPEC=<đường dẫn chuot-that.e2e.mjs> BENCH_PHASE=sau2 npx wdio run <cấu hình tạm>`. Số thô: `ket-qua-truoc.json` (mã HEAD cũ), `ket-qua-sau2.json` (vòng 2, lượt chốt), `ket-qua-gor-removeAllRanges.json` (vòng 2 với `removeAllRanges` gỡ thật), `ket-qua-sau.json` và `ket-qua-tab-6x.json` (vòng 1, giữ làm lịch sử).

Quần thể: Tác phẩm nhập song ngữ 9 cặp (`Ban dich N.`), một Chương, caret ở offset 4 của câu 5 (`Ban dich 5.`). Mỗi đường: bấm ô, đặt caret, mở bề mặt bằng cú bấm thật, gõ `Q`, đọc ô, `Esc` (hoặc bấm ra ngoài với menu), gõ `zz`, đọc ô, chờ 3,5 s, đọc `target_text` từ đĩa.
Tải máy (`uptime`) lúc bắt đầu lượt chốt: 6,24 / 4,70 / 4,57. Lượt chốt: 73 cú bấm có nhãn, 79 lần bấm vật lý, 6 lần phải bấm lại vì mất `click` (mọi lần bấm lại ra `click` ở lần sau); 33/33 ca bàn đo qua trong 6 phút 31 giây.

## Bảng "sau" (vòng 2, `ket-qua-sau2.json`)

`leakedQ` = ô đổi chữ sau khi gõ `Q` khi bề mặt đang hiện; `disk` đọc `target_text` bằng `read_open_chapter_segments` sau 3,5 s. Với mọi dòng "đúng điểm cũ" dưới đây, `leakedQ=false`, `qOnDisk=false`, ô và `disk` cùng đọc `Ban zzdich 5.`.

| Đường | `Q` vào ô | `Esc`/ra ngoài rồi `zz` |
|---|---|---|
| Công cụ: Glossary, Cài đặt Glossary, TM, Cài đặt, Phạm vi soát, Bộ nhớ soát, Nhập bản góp ý (hộp thoại tệp), Căn chỉnh, Thu hoạch Glossary (9 mục) | không | đúng điểm cũ, đĩa khớp |
| Menu `Công cụ ▾` mở, bấm ra ngoài | không | đúng điểm cũ |
| Menu `Công cụ ▾` mở, bấm thẳng nút hàng đợi Glossary ở thanh trạng thái (mới) | không | đúng điểm cũ |
| Thanh trạng thái: hàng đợi Glossary | không | đúng điểm cũ |
| Ngăn kéo Tra cứu, cửa sổ thu về 2000 px (mới; nút `[data-lookup-drawer-open]` hiện ở tầng này) | không | đúng điểm cũ |
| Panel AI: Kiểm Prompt, thư viện Prompt, Cài đặt cấu hình AI | không | đúng điểm cũ |
| Tra cứu: Attribution; menu nguồn Tra cứu (bấm ra ngoài) | không | đúng điểm cũ |
| Hợp âm `⌘⌥M`, `⌘,`, `⌘H` | không | đúng điểm cũ |
| Bấm `Đã xong` rồi gõ `zz` (mới) | không áp dụng | đúng điểm cũ (`Ban zzdich 5.`), đĩa khớp |
| Tab chế độ Thư viện, Đọc | không | không áp dụng: tiêu điểm ở tab, `zz` không vào ô |
| Chuột phải (WebKit bôi `dich`), mục Lịch sử | không | `Ban dichzz 5.`: chèn sau từ bị bôi, đúng hàng Matrix "WebKit tự bôi" |

Ba dòng ngoài "đúng điểm cũ" đều có tên (tab chế độ: tiêu điểm ở tab; Lịch sử qua chuột phải: điểm giữ thu về cuối vùng chọn). Không đường nào đo được mà `Q` lọt.

Ô nhập trong lớp phủ, cú bấm thật vào ô tìm kiếm TM rồi gõ `Qa` (`ket-qua-input-click.json`, một lượt, 2/2 ca qua): `activeElement` là `INPUT.tm-input`, ô nhập đọc `Qa`, ô bản dịch vẫn `Ban dich 5.`. Ba lớp phủ không tự đặt tiêu điểm vào ô nhập khi mở (dòng dưới).

Ô nhập trong lớp phủ, không bấm (mới): ba lớp phủ có ô nhập (TM, Glossary, Cài đặt) giữ tiêu điểm ở panel (`SECTION.tm-panel`, `.gm-panel`, `.set-panel`), không tự đặt tiêu điểm vào ô nhập; `Q` không vào ô bản dịch (`leakedQ=false`) cũng không vào ô nhập (`qInInput=false`). AC 5 đo được ở lớp phủ TM bằng cú bấm thật vào ô nhập; Glossary và Cài đặt chỉ đo ở dạng "không lọt xuống ô bản dịch".

## Đối chứng gỡ `removeAllRanges` khỏi `holdTargetCaret` (`ket-qua-gor-removeAllRanges.json`)

Cùng bàn đo, cùng quần thể, chỉ gỡ dòng `selection.removeAllRanges()` (gỡ thật, đã trả lại). `Q` lọt vào ô và xuống đĩa ở 17 trên 24 dòng đo ô: 9 mục Công cụ trừ Nhập bản góp ý (8), menu `Công cụ ▾`, hàng đợi Glossary, ba liên kết panel AI, Attribution, menu nguồn Tra cứu, ngăn kéo Tra cứu (tầng 2000 px), menu sang nút thanh trạng thái; đĩa đọc `Ban zzQdich 5.` hoặc `Ban zzqdich 5.` (`Q` thành `q` do `cliclick` mất Shift). Không lọt: Nhập bản góp ý (không có lớp phủ), ba hợp âm (lớp phủ lấy tiêu điểm nên phím theo tiêu điểm vào lớp phủ), hai tab chế độ, Lịch sử qua chuột phải (đường menu câu tự xoá vùng chọn trước khi mở). Ba dòng ô nhập cũng lọt (`input.*` leak true). Trước vòng 2, ở mã HEAD cũ, 14 đường lớp phủ và hai menu lọt (`ket-qua-truoc.json`).

## Nguyên nhân gốc của lỗi (đo ở HEAD cũ, giữ nguyên)

`ket-qua-truoc.json` ghi `activeWhileShown` là ô bản dịch và `escClosed: false` ở 14 đường lớp phủ cộng hai menu. Lớp phủ gọi `focus()` lên panel và `focusin` tới panel, không có `focus()` nào lên ô, nên `GridPanel.vue` (`selectionchange`) không giành lại tiêu điểm. Vùng chọn vẫn nằm trong ô contenteditable nên phím gõ đi theo vùng chọn vào ô, và chính cú gõ `Q` đưa `activeElement` về ô; `Esc` sau đó có đích ngoài lớp phủ nên không đóng được. Hai việc cùng một gốc: vùng chọn còn nguyên trong ô.

Trình điều khiển wdio khác chuột thật ở một điểm: `button.focus()` ở `mousedown` làm mất vùng chọn trong ô ngay (`rangeCount` 0 lúc `click`), chuột thật `cliclick` thì không. Vì thế vòng 2 không giữ caret ở `mousedown`, và e2e dựng lại trạng thái chuột thật (tiêu điểm ở nút, `Range` trong ô, rồi mới bắn `click`) ở `pressWithSelection`, kể cả khi mở menu `Công cụ ▾`.

## Đối chứng e2e `overlay-caret-hold.e2e.mjs` (6 ca, một lượt mỗi phép gỡ)

Lượt nguyên vẹn: 6/6 xanh. Các ca e2e chỉ chứng minh caret trở về sau `Esc` (`addRange` chuyển tiêu điểm về ô nên cả `Q` không là phép đo rò); rò và nhánh trả ô do bàn đo chuột thật và Vitest khoá. Bốn phép gỡ thật (đã trả lại, `diff` với bản gốc rỗng):

| Phép gỡ | Ca đỏ |
|---|---|
| lượt trả điểm trong `onHeldFocusIn` (`removeAllRanges` + `addRange(held)`), lượt chạy lại trên spec 6 ca | 5 đỏ, cả năm đọc `zzabc` (caret về đầu ô) thay `abzzc`: Phạm vi soát, TM, Cài đặt, nút thanh trạng thái, hợp âm; menu xanh (đường menu không đi qua `focusin`) |
| `addRange(mine)` trong `restoreHeldSelection` | 1: đúng ca menu `Công cụ ▾` |
| dòng trả ô đã giữ trong `focusReturnTargetOnOpen` | 0 trên e2e (6/6 xanh, kể cả ca nút thanh trạng thái thêm vào để thử); ở các đường e2e dựng, `document.activeElement` đã là ô khi lớp phủ mở nên nhánh này không là nhánh quyết định. Khoá bằng Vitest: ca "đóng rồi gõ: đích trả tiêu điểm là ô" đỏ đúng 1 |
| `removeAllRanges` trong `holdTargetCaret` | e2e: 1 ca (menu, vì `Q` lọt); Vitest: 8 ca đỏ; bàn đo chuột thật: `Q` lọt 17/24 như trên |

Hai nhận xét cho người sau: các ca e2e overlay xanh khi gỡ `removeAllRanges` vì `execCommand('insertText')` của driver chèn theo tiêu điểm, không theo vùng chọn như phím thật; nên bàn đo chuột thật mới là phép đo của AC 2, e2e chỉ khoá lượt trả caret. Nhánh trả ô trong `focusReturnTargetOnOpen` chỉ còn Vitest khoá.

## Chưa đo

- `strip.memory` (liên kết bộ nhớ ở dải soát) cần dải soát có kết quả; `review.notice_import` và `review.align_link` cần Review Mode đang mở với thông báo; ba dòng này `skipped: opener not present in this state` trong `ket-qua-sau2.json`. Dựng: cần chạy một lượt soát AI và mở Review Mode trong quần thể bàn đo, chưa làm.
- `status.lookup_drawer` ở cửa sổ 2600 px không có nút (đúng thiết kế); đã đo ở tầng 2000 px.
- `ImportPreviewOverlay` và `BilingualImportPreviewOverlay` chỉ mở từ màn Thư viện qua hộp thoại chọn tệp gốc; lớp phủ nào thiếu `role="dialog"` thì lớp lồng trong nó trả về ô thay vì lớp cha.
- Chữ vào ô nhập của lớp phủ Glossary và Cài đặt khi ô nhập có tiêu điểm; WebView2 (`B7`).
- `Esc` không đóng được menu `Công cụ ▾` và menu nguồn (tiêu điểm không ở trong menu): có từ trước, không đổi.

## Lượt chốt trên mã cuối

Sau các bản vá review vòng 2, cùng bản dựng, Vite 1421 dựng mới, tải 4,4 (`ket-qua-chot.json`): bàn đo 34/34 ca, 0 đường lọt, 22/25 dòng chèn `zz` đúng điểm cũ cả ở ô lẫn đĩa, ba ngoại lệ là Lịch sử qua chuột phải (`Ban dichzz 5.`, đúng Matrix) và hai tab chế độ. `e2e/specs/overlay-caret-hold.e2e.mjs` 6/6.
