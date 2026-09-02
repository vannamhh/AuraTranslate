# Bàn đo Story 5.14 — NFR3/NFR4/NFR5

Chạy từ gốc kho:

```sh
_bmad-output/implementation-artifacts/5-14-ban-do/run.sh
```

Một lệnh sẽ chạy ba session NFR3 release, bàn đo hai hình dạng `read_reading_run`, dựng app
Tauri release có probe production, rồi chạy ba session app cho cold/warm startup và hai fixture
memory (`full`/`frontier`). Mỗi pha bộ nhớ lấy 10 mẫu idle của **PID app cộng mọi WebKit mới
sinh**. HOME/DB/.app/dist/target chỉ sống ở vị trí nháp hoặc đường build đã bị Git bỏ qua;
`trap` giết app và xoá HOME nháp kể cả khi lượt đo trượt.

Hàng rào trước khi chạy:

- chỉ cho phép hai contract test, tạo tác/tracking của Story 5.14 và đúng hai thay đổi móc
  đo: feature rỗng trong `src-tauri/Cargo.toml` + command pha trong `src-tauri/src/lib.rs`;
- command pha không có trong feature mặc định, không thêm dependency, không mở mạng; nó chỉ
  ghi bốn marker đã liệt kê cứng bằng kho cấu hình, đọc đúng
  `$HOME/.auratranslate-5-14-phase` do runner tạo **trong HOME nháp** và native-eval các kiểm
  DOM/tab. WebKit
  isolated world không phát click tới Vue nên command dùng đúng vỏ `open_work` của sản phẩm
  sau grid `usable` (không tạo state riêng); Reading chỉ được nhận khi marker chứng minh
  `full = 50.000` segment hoặc `frontier = 0`, còn Library chỉ được nhận khi grid thật có đúng
  fixture; các điều kiện đạt vẫn là UI/DOM thật;
- fixture export chỉ nhận đích có marker `auratranslate-5-14-` và thư mục rỗng;
- usable đòi grid có đúng một Work tên `5.14 Fixture`; fixture rỗng hoặc gỡ probe usable sẽ
  timeout thành `unknown` và `run.sh` thoát đỏ;
- một mẫu memory thiếu WebKit mới sinh, PID chết, `phys_footprint` hoặc RSS sẽ được giữ thành
  hàng `error`; app PID đơn lẻ không bao giờ được nhận;
- trạng thái `done`/`not_started` chỉ được đổi trong `project.db` tổng hợp dưới HOME nháp.

Không dùng bàn phím, chuột, `wdio`, debug build hay dữ liệu Library thật trong lượt đo. Trong
lúc chạy không mở thêm ứng dụng WebKit: tập WebKit của AuraTranslate được nhận bằng hiệu PID
trước/sau spawn, nên một WebKit ngoài phạm vi mới sinh giữa session sẽ làm bẩn tập đo.

Đầu ra giữ lại: `environment.txt`, `fixture.txt`, bốn tệp `*-raw.tsv` và `REPORT.md`. Phán
quyết chỉ là **sơ bộ**; A6–A8/Q4 vẫn mở tới Story 6.18, nơi FR14 tạo 5.000 Chương qua sản phẩm.
