# Story 6.18 — kết quả đo NFR3/NFR4/NFR5 trên thư viện 5.000 Chương thật

Ngày đo, commit, máy, OS, toolchain, profile và tải máy nằm trong `environment.txt`. Thư viện
(50 Tác phẩm × 100 Chương × 10 segment =
5000 Chương / 50000 segment) được dựng qua `confirm_bilingual_import` +
`lifecycle::set_chapter_status` + `Indexer::rebuild` thật — KHÔNG một fixture SQL thô nào — rồi
xoá theo HOME nháp; chỉ mẫu TSV và báo cáo này được giữ lại. Đây **THAY THẾ**, không cộng thêm,
bản sơ bộ của Story 5.14 (một fixture tổng hợp/0 lớp từ điển). Con số này **THAY THẾ** bản sơ bộ
Story 5.14 — Epic 5 retro F5 gọi con số đó "không phải một con số về sản phẩm"; đây là con số về
sản phẩm.

## Phán quyết

| NFR | Phép đo quyết định | Ngưỡng | Phán quyết |
| --- | ---: | ---: | --- |
| NFR3 | p95 xấu nhất của từng ca = 183.890 ms | p95 < 500 ms | dưới ngưỡng |
| NFR4 | cold median/max = 3151.724 ms / 5770.631 ms; warm median/max = 3331.658 ms / 6144.974 ms | < 3.000 ms | vượt ngưỡng |
| NFR5 | phys_footprint lớn nhất = 700018688 byte · 700.019 MB · 667.590 MiB; RSS đối chiếu lớn nhất = 462450688 byte · 462.451 MB · 441.027 MiB | < 300 MB | vượt ngưỡng (theo cả MB và MiB) |

NFR4 kết thúc khi probe thấy `[data-library-grid]` mang đủ 50 `[data-library-work-cell]`
thật (kể cả Tác phẩm đích Reading sẽ mở) VÀ ít nhất một lớp từ điển đã nạp; mốc ngoài tiến trình
chạy từ trước spawn tới sau khi marker đã được ghi vào `global.db`, nên là cận trên nhỏ của mốc
DOM. NFR5 từ chối mọi mẫu chỉ có PID app: mỗi hàng `ok` có PID app cộng ít nhất một WebKit mới
sinh; hàng thiếu PID/footprint/RSS được giữ nguyên là `error` và làm phán quyết thành
`chưa phân xử`. Reading pha `full` mở ĐÚNG MỘT Tác phẩm (không toàn thư viện — Design Notes
spec 6.18: "Reading runs over the open Work only"), nên số NFR5 pha Reading KHÔNG so sánh được
với 894.570.496 byte của Story 5.14 (nơi Reading đọc cả 50.000 segment của một fixture tổng hợp).

## Bộ nhớ theo pha

| Fixture | Pha idle | Số mẫu hợp lệ | phys_footprint lớn nhất | RSS lớn nhất |
| --- | --- | ---: | ---: | ---: |
| full | library | 100 | 422752256 byte · 422.752 MB · 403.168 MiB | 294449152 byte · 294.449 MB · 280.809 MiB |
| full | reading | 100 | 632696832 byte · 632.697 MB · 603.387 MiB | 457908224 byte · 457.908 MB · 436.695 MiB |
| full | back_library_keepalive | 100 | 700018688 byte · 700.019 MB · 667.590 MiB | 462450688 byte · 462.451 MB · 441.027 MiB |
| frontier | library | 100 | 379621376 byte · 379.621 MB · 362.035 MiB | 297086976 byte · 297.087 MB · 283.324 MiB |
| frontier | reading | 100 | 409534464 byte · 409.534 MB · 390.563 MiB | 305725440 byte · 305.725 MB · 291.563 MiB |
| frontier | back_library_keepalive | 100 | 391389184 byte · 391.389 MB · 373.258 MiB | 308477952 byte · 308.478 MB · 294.188 MiB |

Mỗi session full đo Library (50 Tác phẩm) → Reading 1000
segment (một Tác phẩm, mọi Chương `done`) → quay lại Library. Mỗi session frontier đo Library →
Reading frontier-only trên Chương đầu Tác phẩm đích (100 Chương vừa `not_started` qua
`lifecycle::set_chapter_status`, không một câu SQL ghi nào) → quay lại Library. Dữ liệu thô:
`nfr3-raw.tsv`, `startup-raw.tsv`, `memory-raw.tsv`, `transition-raw.tsv`, `fixture.txt`.
