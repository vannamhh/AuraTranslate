# Bàn đo AD-38 — Story 6.12 (đọc `.docx`)

`baseline_commit`: `d58cb771c39bfc51aa4f756717225abb2ccea7e4` (xem `environment.txt` cho môi trường đầy đủ).

## Trạng thái: **0 mẫu — CHƯA ĐO**

`src-tauri/tests/fixtures/docx/` (gitignore) không có tệp `.docx` nào tại thời điểm story này
đóng. `docx_probe.rs` xác nhận đúng trạng thái đó bằng `assert!` (thoát khác 0, không phải một
lượt xanh im lặng):

```
$ cargo test --locked --manifest-path src-tauri/Cargo.toml --test docx_probe -- --ignored --nocapture
DOCX_SAMPLES	0
thread '...' panicked at tests/docx_probe.rs:80:5:
0 mẫu trong .../src-tauri/tests/fixtures/docx — Ice CHƯA thả tệp .docx THẬT vào. Đây là LỖI HẠ
TẦNG của bàn đo, không phải một phép đo với tỉ lệ đúng 0% — ghi nợ vào deferred-work.md, chủ
Ice, không suy phán quyết AD-38 từ đây.
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

⚠️ **Đây KHÔNG phải một phán quyết "AD-38 đạt 0%".** Nó là câu "chưa có dữ liệu để đo" —
đúng phân biệt mà spec 6.12 đòi (khuôn `webimport_probe.rs:260`, Story 6.1).

## Việc ĐÃ đo được, không cần fixture Word thật

- `docx_contract.rs` (17 ca, `cargo test --locked --test docx_contract`) kiểm `core::docx::read_docx`
  trên bảy fixture **tự sinh bằng `docx-rs`** — đếm hàng/ô/đoạn-mỗi-ô đúng, ảnh nhúng đọc ra
  byte thật ở đúng vị trí khối, `.docx` rỗng/hỏng/giả mạo đuôi bị từ chối phân biệt được, và
  một `.docx` có bảng đi TRỌN đường sản phẩm không để segment nào vắt qua ranh giới ô (đóng nợ
  `:2214`). Đây là bằng chứng **luật đọc của TA đúng với OOXML mà TA tự sinh ra**.
- `docx_boundary.rs` (9 ca) khoá `core::docx` ở 0 dòng gõ mạng và 0 điểm panic — mệnh đề mà
  Quyết định 3 (tự đọc, không gọi `docx_rs::read_docx`) dựa lên.

## Việc CHƯA đo được — nợ có chủ

🔴 **0 tệp Word THẬT đi qua `core::docx::read_docx`.** Fixture tự sinh (`docx-rs`) chứng minh
luật CỦA TA đọc đúng cấu trúc CỦA TA tự tạo ra; nó **không** chứng minh Word thật (mọi phiên
bản, mọi hệ điều hành) sinh ra hình dạng XML giống hệt — ví dụ Word có thể dùng
`<w:pict>`/VML cho ảnh cũ hơn `<w:drawing>`/DrawingML (module này chỉ đọc DrawingML đầy đủ,
VML chỉ đọc qua `v:imagedata` một cách hạn chế — xem doc-comment `core/docx/mod.rs` mục 7),
hoặc namespace prefix khác chuẩn (đơn giản hoá "theo local name" ở mục 3 doc-comment đó).
**Chủ: Ice** — thả tệp `.docx` thật (đặc biệt: xuất từ Word thật, không phải LibreOffice/Google
Docs xuất lại, để đo đúng "Word thật sinh ra hình dạng gì") vào
`src-tauri/tests/fixtures/docx/`, rồi chạy lại lệnh ở trên; `docx-raw.tsv` sẽ xuất hiện trong
thư mục này.

Xem `_bmad-output/implementation-artifacts/deferred-work.md` (mục Story 6.12) cho đầy đủ danh
sách nợ mới, bao gồm mục này.
