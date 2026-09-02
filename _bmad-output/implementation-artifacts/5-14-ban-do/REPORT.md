# Story 5.14 — kết quả đo sơ bộ NFR3/NFR4/NFR5

Ngày đo, commit, máy, OS, toolchain, profile và tải máy nằm trong `environment.txt`. Fixture nằm trong HOME nháp và bị xoá bởi trap; chỉ mẫu TSV và báo cáo này được giữ lại. Đây là fixture tổng hợp một Work/5.000 Chương, chưa phải thư viện 5.000 Chương tạo qua FR14, nên mọi phán quyết đều **sơ bộ**; A6–A8/Q4 vẫn mở tới Story 6.18.

## Phán quyết

| NFR | Phép đo quyết định | Ngưỡng tạm | Phán quyết |
| --- | ---: | ---: | --- |
| NFR3 | p95 xấu nhất của từng ca = 57.981 ms | p95 < 500 ms | dưới ngưỡng (sơ bộ) |
| NFR4 | cold median/max = 2581.373 ms / 4364.883 ms; warm median/max = 2054.714 ms / 4083.545 ms | < 3.000 ms | vượt ngưỡng (sơ bộ) |
| NFR5 | phys_footprint lớn nhất = 894570496 byte · 894.570 MB · 853.129 MiB; RSS đối chiếu lớn nhất = 1112899584 byte · 1112.900 MB · 1061.344 MiB | < 300 MB | vượt ngưỡng (sơ bộ theo cả MB và MiB) |

NFR4 kết thúc khi probe thấy `[data-library-grid]` mang đúng một `[data-library-work-cell]` tên “5.14 Fixture”; mốc ngoài tiến trình chạy từ trước spawn tới sau khi marker đã được ghi vào `global.db`, nên là cận trên nhỏ của mốc DOM. NFR5 từ chối mọi mẫu chỉ có PID app: mỗi hàng `ok` có PID app cộng ít nhất một WebKit mới sinh; hàng thiếu PID/footprint/RSS được giữ nguyên là `error` và làm phán quyết thành `chưa phân xử`.

## Hai chi phí `read_reading_run`

| Hình dạng | Mẫu | p50 | p95 | p99 | Xấu nhất |
| --- | ---: | ---: | ---: | ---: | ---: |
| frontier_5000_chapters_0_segments | 30 | 2.922 ms | 3.272 ms | 3.303 ms | 3.303 ms |
| full_run_5000_chapters_50000_segments | 30 | 179.744 ms | 198.362 ms | 220.650 ms | 220.650 ms |

## Bộ nhớ theo pha

| Fixture | Pha idle | Số mẫu hợp lệ | phys_footprint lớn nhất | RSS lớn nhất |
| --- | --- | ---: | ---: | ---: |
| full | library | 30 | 434012160 byte · 434.012 MB · 413.906 MiB | 249954304 byte · 249.954 MB · 238.375 MiB |
| full | reading | 30 | 894570496 byte · 894.570 MB · 853.129 MiB | 1036324864 byte · 1036.325 MB · 988.316 MiB |
| full | back_library_keepalive | 30 | 797896704 byte · 797.897 MB · 760.934 MiB | 1112899584 byte · 1112.900 MB · 1061.344 MiB |
| frontier | library | 30 | 305475584 byte · 305.476 MB · 291.324 MiB | 250667008 byte · 250.667 MB · 239.055 MiB |
| frontier | reading | 30 | 393441280 byte · 393.441 MB · 375.215 MiB | 256512000 byte · 256.512 MB · 244.629 MiB |
| frontier | back_library_keepalive | 30 | 403013632 byte · 403.014 MB · 384.344 MiB | 259747840 byte · 259.748 MB · 247.715 MiB |

Mỗi session full đo Library → Reading 50.000 segment → quay lại Library (Reading component nằm trong KeepAlive). Mỗi session frontier đo Library → Reading frontier-only trên Chương đầu chưa `done` → quay lại Library. Dữ liệu thô: `nfr3-raw.tsv`, `reading-run-raw.tsv`, `startup-raw.tsv`, `memory-raw.tsv`.
