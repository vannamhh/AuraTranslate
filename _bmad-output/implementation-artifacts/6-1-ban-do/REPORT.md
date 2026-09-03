# Story 6.1 — kết quả mũi thăm dò ba lựa chọn thư viện

Ngày đo, commit, máy, OS, toolchain nằm trong `environment.txt`. Ba bàn đo sống trong
`src-tauri/tests/webimport_probe.rs` (`#[ignore]`), dữ liệu thô ở `extraction-raw.tsv` ·
`encoding-raw.tsv` · `reqwest-raw.tsv`.

## Phán quyết

| Câu hỏi | Số mẫu thật | Kết quả | Phán quyết |
| --- | ---: | --- | --- |
| FR123 — `dom_smoothie` bóc đúng nội dung chính? | **7** (6 bài báo + 1 trang không phải bài) | 7/7 fetch+extract không lỗi; 6/6 bài: tiêu đề đúng, mở bài đúng, độ đầy đủ nội dung 72–99% so với vùng bài viết thật trong HTML (không lẫn menu/quảng cáo); 1/1 trang không phải bài bị chấm đúng là "không giống bài viết" | **Đạt** cho ca thuận trên site đã đo — ghim `dom_smoothie` 0.18.0 |
| FR126 — `chardetng`/`encoding_rs` dò đúng GBK/Big5? | **0** | Thư mục `fixtures/encoding/` rỗng — Ice chưa cấp fixture, Story 6.1 bị cấm tự sinh | **Chưa đo** — không phải "đạt 0%". Crate đã ghim, số đo còn thiếu; nợ chủ Ice ở `deferred-work.md` |
| `reqwest` — đủ ba năng lực cho `Fetcher`? | **3** (một ca mỗi năng lực, trên server cục bộ) | Chặn 1/1 chuyển hướng khác host (server bị chặn nhận 0 kết nối); cắt luồng ở 1.048.576/20.971.520 byte quảng cáo; 1/1 lỗi kết nối được nhận diện đúng | **Đạt** — xác nhận `reqwest` (đã ghim từ Story 1.2), không cần crate mới |

## Phương pháp

**FR123.** `urls.txt` (commit) liệt 7 URL thật trên `epochtimes.com`: 6 bài báo (3 giản
thể `/gb/`, 3 phồn thể `/b5/`) + 1 trang chủ ấn bản phồn thể (ca "trang không phải bài").
Bàn đo tự tải HTML (cache vào `fixtures/html/`, gitignore), gọi
`Readability::is_probably_readable()` rồi `Readability::parse()` từ `dom_smoothie`, ghi
`id, url, is_probably_readable, char_count, paragraph_count_approx, title, first_80_chars,
last_80_chars, note` vào `extraction-raw.tsv`. Chân lý nền là một lượt xử tay: đối chiếu
`char_count` với ký tự trong vùng `<div id="post_content">` của chính HTML (đọc bằng
`html.parser` cân bằng độ sâu thẻ `div`, không phải regex cắt ngang), và đọc `title`/
`first_80_chars` bằng mắt so với tiêu đề/mở bài thật trên trang.

| id | url (rút gọn) | is_probably_readable | char_count | manual post_content | tỉ lệ |
| --- | --- | --- | ---: | ---: | ---: |
| a01 | .../gb/26/8/31/n14840292 | true | 325 | 397 | 82% |
| a02 | .../b5/26/8/31/n14840300 | true | 589 | 666 | 88% |
| a03 | .../gb/26/8/1/n14821642 | true | 3.545 | 3.583 | 99% |
| a04 | .../b5/26/8/27/n14837938 | **false** | 192 | 265 | 72% |
| a05 | .../gb/26/7/12/n14808015 | true | 3.727 | 3.784 | 98% |
| a06 | .../b5/26/8/29/n14839195 | true | 483 | 554 | 87% |
| a07 | .../b5/ (trang chủ) | false | 4.749 | — (không phải bài) | — |

`dom_smoothie` luôn hụt một chút so với vùng thật (hụt nhiều nhất ở bài ngắn dạng tóm tắt
video, a01/a04/a06), không bao giờ vượt — không quan sát được trường hợp lẫn rác (nav,
quảng cáo, liên kết "bài liên quan") vào nội dung đã bóc.

**FR126.** Bàn đo quét `fixtures/encoding/*.txt`, quy ước tên `<mô-tả>__<NHÃN>.txt`. Thư
mục không tồn tại/rỗng ⇒ `assert!` đỏ với thông báo phân biệt rõ "lỗi hạ tầng" khỏi "tỉ lệ
0%", thoát mã **101**. Đã tự kiểm ca này (chạy thật, ghi lại exit code) — không suy đoán.

**`reqwest`.** Ba ca trên server HTTP thô tự dựng ở `127.0.0.1` (cổng hệ điều hành cấp),
không cần mạng ngoài:

- *Chuyển hướng*: server A trả `301` sang server B ở cổng khác (đứng cho host khác).
  `redirect::Policy::custom` chỉ cho đi tiếp nếu cổng đích khớp allowlist (ở đây: cổng của
  chính A) — không khớp ⇒ `attempt.stop()`. Đo: response cuối cùng vẫn là `301` kèm header
  `Location`, chuỗi chuyển hướng ghi lại đúng 1 chặng, server B nhận **0** kết nối.
- *Kích thước*: server C khai `Content-Length: 20.971.520` rồi stream. Client đọc qua
  `impl Read for Response` (không `.bytes()`/`.text()`) với đệm 64 KiB, dừng khi vượt trần
  1.048.576 byte rồi `drop` ngay. Đo: `actually_read = 1.110.628` byte (dừng ngay sau khi
  vượt trần, không đọc hết 20 MiB quảng cáo).
- *Mạng hỏng*: bind rồi thả một cổng (không ai lắng nghe), gửi yêu cầu, trần thời gian 2s.
  Đo: `Err` với `is_connect() == true`.

## Giới hạn

- **Bàn đo FR123 chạy trên trang báo, không trên truyện.** `dom_smoothie` là cổng
  Readability, tuning cho bài báo — đây là ca THUẬN của nó. Tỉ lệ 72–99% **không nói gì**
  về blog cá nhân, diễn đàn, hay trang đọc truyện chữ (Epic 6 sẽ cần đo lại trên nguồn đó
  khi có đường nhập thật, Story 6.9).
- **Một site duy nhất** (`epochtimes.com`, 7 mẫu). Không nói gì về site có cấu trúc DOM
  khác hẳn.
- **`paragraph_count_approx` là xấp xỉ** — đếm khớp chuỗi `"<p"` trong HTML đã bóc (gồm cả
  `<pre`), đủ cho một mũi thăm dò, không đủ cho một cổng nghiệm thu.
- **FR126 chưa có một mẫu thật nào** — xem `deferred-work.md`, chủ Ice.
- **`is_probably_readable()` có ít nhất một âm tính giả đã quan sát** (mẫu `a04`) — ghi nợ
  cho Story 6.9 (màn xem trước) đừng dùng cờ này làm điều kiện duy nhất phân loại "cần
  xem".
- Cây `cargo tree --no-dedupe` thêm hai phần tách được: **253** dòng từ cây con
  `dom_smoothie`+`chardetng` (12 gói mới), và **32** dòng từ việc bật feature `blocking`
  trên `reqwest` (không gói mới — chỉ bật lại cạnh đồ thị đã có). Xem `ARCHITECTURE-SPINE.md`
  §Stack cho rà giấy phép đầy đủ, bao gồm ghi chú `MPL-2.0` của ba gói bắc cầu qua
  `dom_smoothie` (`cssparser`/`cssparser-macros`/`selectors`) — hạng giấy phép đầu tiên
  khác nhóm dễ dãi thường lệ, vẫn tương thích GPLv3 chiều đi vào.
