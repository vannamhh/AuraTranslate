# Bàn đo Story 6.1 — ba mũi thăm dò thư viện đường nhập

Ba giả định của Epic 6 chưa ai đo trước story này: `dom_smoothie` bóc được nội dung chính
hay không (FR123), `chardetng` + `encoding_rs` dò đúng GBK/Big5 hay không (FR126), và
`reqwest` có đủ ba năng lực mà `Fetcher` cần hay không (chặn chuyển hướng theo chặng, cắt
thân theo dòng chảy, báo lỗi mạng). Bàn đo này **không** dựng `Fetcher`/`Extractor` thật —
đó là Story 6.2/6.3/6.9 — nó chỉ gọi ba crate ứng viên đủ để lấy số.

## Cách chạy

```sh
cargo test --locked --manifest-path src-tauri/Cargo.toml --test webimport_probe \
  -- --ignored --nocapture
```

Ba hàm `#[ignore]` sống trong `src-tauri/tests/webimport_probe.rs`:

1. **`dom_smoothie_records_one_tsv_row_of_extraction_measurements_per_real_fetched_article`**
   — đọc `urls.txt` (bài báo epochtimes.com thật, commit được), tự tải HTML vào
   `fixtures/html/` nếu chưa có cache, bóc bằng `dom_smoothie`, ghi `extraction-raw.tsv`.
   Cần mạng ở lượt chạy ĐẦU (sau đó dùng cache).
2. **`chardetng_records_the_true_and_guessed_label_of_every_encoding_fixture_or_fails_loudly_on_zero_samples`**
   — quét `fixtures/encoding/*.txt`, dò bằng `chardetng`, ghi `encoding-raw.tsv`. **Cần Ice
   tự cấp fixture** — xem §Fixture bảng mã. Thư mục rỗng ⇒ bàn đo **thoát khác 0** (đây là
   hành vi ĐÚNG theo thiết kế, không phải một lỗi của bàn đo).
3. **`reqwest_blocks_a_cross_host_redirect_caps_a_streamed_body_and_reports_a_dead_connection`**
   — dựng server HTTP thô trên `127.0.0.1` (cổng hệ điều hành cấp) để đo ba năng lực của
   `reqwest`; không cần mạng ngoài, không cần fixture.

## Fixture bảng mã — Ice tự cấp vào `fixtures/encoding/`

Quy ước tên tệp: **`<mô-tả>__<NHÃN>.txt`** (hai dấu gạch dưới liền nhau trước nhãn), `NHÃN`
là một trong năm bảng của FR126, không phân biệt hoa/thường: `UTF-8` · `GB18030` · `GBK` ·
`BIG5` · `UTF-16`. Ví dụ: `dien-dan-2010__GBK.txt`, `tieu-thuyet-cu__BIG5.txt`.

**Không tự sinh fixture bằng cách mã hoá ngược từ UTF-8** — mã hoá bằng `encoding_rs` rồi
bảo `chardetng` đọc lại là một vòng tròn (xem `spec-6-1-mui-tham-do-ba-lua-chon-thu-vien.md`
§Never). Tệp phải là `.txt` THẬT lấy từ nguồn cũ (diễn đàn, tài liệu cũ) — nó mang BOM lẫn
lộn, tệp trộn mã, hoặc dòng meta ASCII đầu tệp mà một tệp tự sinh không bao giờ có.

## Fixture bóc nội dung — `urls.txt`

Committed, không cần Ice cấp thêm. Bảy dòng: sáu bài báo epochtimes.com thật (`ca thuận`)
cộng một trang KHÔNG phải bài (trang chủ ấn bản Phồn thể, `/b5/`) cho ca "trang không phải
bài" của ma trận I/O. `fixtures/html/` (gitignore) giữ HTML đã tải — **không commit**, nội
dung có bản quyền.

## Đầu ra giữ lại

`environment.txt` · `REPORT.md` · `extraction-raw.tsv` · `encoding-raw.tsv` ·
`reqwest-raw.tsv`. `fixtures/` và `*.log` không commit (`.gitignore`).

## Giới hạn đã biết, ghi ra để không ai đọc lố

- Bàn đo bóc chạy trên **trang báo**, không trên truyện. `dom_smoothie` là cổng
  Readability, vốn tuning cho bài báo — đây là ca THUẬN của nó; tỉ lệ đo được **không nói
  gì** về nguồn khác (blog cá nhân, diễn đàn, trang truyện chữ).
- Cả bảy mẫu bóc đều từ **một site** (epochtimes.com) — không nói gì về site khác có cấu
  trúc DOM khác hẳn.
- Đếm "số đoạn" trong `extraction-raw.tsv` là **xấp xỉ** (đếm khớp chuỗi `"<p"` trong HTML
  đã bóc, gồm cả `<pre`) — đủ cho một mũi thăm dò, không đủ cho một cổng nghiệm thu.
