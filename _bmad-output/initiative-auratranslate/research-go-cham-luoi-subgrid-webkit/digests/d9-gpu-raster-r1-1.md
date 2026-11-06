# D9 - GPU blocklist, software compositing, Windows 2015-era Intel/AMD (digest r1-1)

Ngày truy cập mọi nguồn: 2026-11-06. Ngân sách đã dùng: 10 lệnh gọi công cụ, khoảng 7 nguồn đọc. Không đọc tệp dự án nào.

## Tóm tắt phẳng

Khong thiết lập được (not established) rằng MacBook Pro 2015 Boot Camp đang chạy software compositing. Bằng chứng thu được cho thấy ngược lại với giả thuyết "bị blocklist": trong `software_rendering_list.json` hiện hành không có mục nào khoá `gpu_compositing`, và không có mục nào nêu tên Iris, HD 4000/4400/4600/5000/6000 hay R9 M370X. Khả năng còn lại là GPU process hỏng/không khởi tạo được với driver Boot Camp cũ, hoặc rasterization/D3D11 bị tắt riêng. Việc đó chỉ xác nhận được bằng `edge://gpu` trên máy thật.

## Các khẳng định

1. Tệp `software_rendering_list.json` (nhánh main) không có mục nào liệt `gpu_compositing` trong `features`; các mục Windows có `features: all` là: 34 (S3 Trio), 45 (Parallels cũ), 46 (ATI FireMV 2400, thiết bị 0x3151), 68 (VMware), 94 (Intel driver 8.15.10.1749), 166 (AMD 8.17.10.*, trừ >= 8.17.10.1433). Không mục nào trùng thế hệ 2013-2015 hay R9 M370X (tên, không có mô tả theo tên). URL: https://chromium.googlesource.com/chromium/src/+/refs/heads/main/gpu/config/software_rendering_list.json?format=TEXT . Nhà xuất bản: Chromium project (googlesource). Ngày: tệp nhánh main, không có trường ngày/phiên bản cấp tệp (tệp chỉ có `name` và `entries`). Truy cập 2026-11-06. Độ tin cậy: medium (đọc qua WebFetch tóm tắt bằng mô hình nhỏ, chưa tự giải mã base64; hai lần đọc nhất quán với nhau, nhưng chưa hai nguồn độc lập). Lớp: source-code.

2. Các mục Intel Windows liên quan tới driver cũ chỉ chạm tính năng phụ: id 145 (`accelerated_webgl2`, driver < 8.16.0.0), id 163 (`accelerated_webgl` + `accelerated_2d_canvas`, driver < 8.15.10.2262), id 78/87/165 (video decode), id 181 ("Corrupted browser/webpage rendering on Win10/Intel HD Graphics 4400", thiết bị 0x0a16, chỉ tính năng `skia_graphite`), id 184/187 (tắt Graphite: driver <= 10.18.10.5161, hoặc `intel_gpu_generation` <= 8). Ngưỡng driver Boot Camp thực tế của Apple chưa tra được, nên không biết có rơi vào các ngưỡng này không. Cùng URL như mục 1. Medium, cùng lý do. Lớp: source-code.

3. Id 122 (Windows, `gpu_tile_rasterization`): GPU rasterization chỉ bật trên NVIDIA, Intel, AMD RX-R2 với DX11 trở lên (Intel cần `d3d11_feature_level` >= 11.0; AMD thêm driver >= 8.17.10.1366). Id 124: AMD driver > 8.17.10.1460 có lỗi hiển thị với GPU raster (ngoại lệ >= 8.17.10.1507). Id 139: tắt GPU raster trên AMD pre-GCN trong dải driver hẹp. Hệ quả: nếu Windows chỉ cho D3D11 feature level < 11, thì raster về CPU, nhưng compositing vẫn có thể ở GPU. Cùng URL. Medium (mô hình đọc tóm tắt, không trích nguyên văn). Lớp: source-code.

4. Chromium tách "compositing" và "rasterization" thành hai dòng trạng thái trong trang GPU; "Compositing: Software only" nghĩa là GPU compositing bị tắt (do blocklist, flags hoặc dòng lệnh), trình duyệt rơi về software compositing; các báo cáo ví dụ có Compositing và Rasterization cùng software trong khi Canvas vẫn hardware. URL: https://bugs.launchpad.net/ubuntu/+source/chromium-browser/+bug/1899270/comments/4 và https://forum.doom9.org/archive/index.php/t-183374.html . Nhà xuất bản: Launchpad bug tracker, diễn đàn doom9. Ngày: không rõ (cũ, thời Chromium 60-85). Truy cập 2026-11-06. Độ tin cậy: low-medium: nguồn thứ cấp, người dùng, quá cũ cho ngưỡng "version" 1 tháng; nhãn có thể đổi. Lớp: pattern.

5. Chromium thiết kế GPU raster cũ ghi "GPU Rasterization" kiểm được ở about:gpu, và nội dung có thể bị veto GPU raster. URL: https://www.chromium.org/developers/design-documents/chromium-graphics/how-to-get-gpu-rasterization/ . Chromium.org, ngày không rõ (thời Chrome 37). Medium-low, cũ; chỉ dùng để hiểu khái niệm. Lớp: pattern. (Chỉ thấy qua kết quả tìm kiếm, chưa fetch.)

6. Cờ còn tồn tại trong `gpu/config/gpu_switches.cc` nhánh HEAD: `ignore-gpu-blocklist` ("Ignores the GPU blocklist"), `disable-gpu-rasterization` ("rasterize on the CPU only", thắng `enable-gpu-rasterization`), `enable-gpu-rasterization` (heuristic quyết định, chỉ hợp lệ khi có GPU-accelerated compositing). URL: https://chromium.googlesource.com/chromium/src.git/+/HEAD/gpu/config/gpu_switches.cc . Chromium project, nhánh HEAD, không ngày. Medium (đọc qua tóm tắt; "HEAD" tại thời điểm truy cập, chưa gắn với nhãn 156.x). Lớp: version.

7. `disable-gpu`, `force-gpu-rasterization`, `disable-software-rasterizer` KHÔNG nằm trong `gpu_switches.cc` (chúng ở tệp khác, ví dụ content/ui switches, chưa kiểm). Tài liệu WebView2 (cập nhật 2026-10-13, ms.date 2026-08-28) liệt `disable-gpu` ("Disables GPU hardware acceleration. If a software renderer isn't in place, the GPU process doesn't launch"), `disable-gpu-driver-bug-workarounds` và `ignore-gpu-blocklist` là cờ WebView2 hiện có; trang KHÔNG liệt `enable-gpu-rasterization`, `disable-gpu-rasterization` hay `disable-software-rasterizer`. Điều đó chưa chứng minh chúng bị bỏ ở 156 (trang là danh sách tuyển chọn). Việc `disable-software-rasterizer` còn tồn tại ở 156: KHÔNG xác nhận được. URL: https://learn.microsoft.com/microsoft-edge/webview2/concepts/webview-features-flags . Microsoft Learn (MSEdgeTeam). Độ tin cậy: high cho phần "WebView2 liệt kê disable-gpu và ignore-gpu-blocklist" (đọc nguyên trang); low cho phần còn lại. Lớp: version.

8. Cách đặt cờ trong WebView2: biến môi trường `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` hoặc registry khi chẩn đoán, hoặc `AdditionalBrowserArguments`; Microsoft khuyên không dùng cờ trong sản phẩm vì có thể bị gỡ hoặc đổi; ứng dụng chạy elevated bỏ qua cờ từ biến môi trường. Cùng URL mục 7. High. Lớp: spec.

9. WebView2 dùng cùng cờ Chromium (Chromium flags), và có cờ riêng `force-device-scale-factor` ("Overrides the device scale factor for the Edge WebView browser UI and the contents") và `embedded-browser-webview-dpi-awareness`. Trang không nêu WebView2 dùng đường compositing khác Edge, cũng không nêu khả năng rơi GPU cao hơn. URL như mục 7. High cho văn bản trang; "không có bằng chứng" là phát hiện về sự vắng mặt. Lớp: spec.

10. Một thread Microsoft Q&A: nhà phát triển WebView2 có GPU process cứ crash, `--disable-gpu` không giúp; người điều hành chỉ ra driver hiển thị là nguyên nhân khả dĩ. URL: https://learn.microsoft.com/en-us/answers/questions/1856807/webview2-gpu-related-error-and-some-questions . Microsoft Q&A, ngày không lấy được. Low (chỉ thấy qua kết quả tìm kiếm, một nguồn, giai thoại). Lớp: bug-status (trạng thái hiện tại không rõ). 

11. Trên Windows với backend ANGLE D3D11, Chromium dùng DirectComposition để vẽ nội dung cửa sổ (theo bài Electron về resize). URL: https://electronjs.org/zh/blog/tech-talk-window-resize-behavior . Electron blog, ngày không lấy được. Medium-low, chỉ qua kết quả tìm kiếm. Lớp: pattern.

12. SwiftShader: Chromium đã bỏ việc tự động fallback WebGL sang SwiftShader (Intent to Remove); chỉ nêu cho WebGL, không phải compositing của UI. URL: https://groups.google.com/a/chromium.org/g/Blink-dev/c/yhFguWS_3pM . Blink-dev, ngày không lấy được. Low-medium. Lớp: version.

13. Graphite (raster backend mới) có mục đích giảm jank/độ trễ do Ganesh/OpenGL đơn luồng; blog Chromium 2025-07. Không có số đo cho phần mềm vs GPU. URL: https://blog.chromium.org/2025/07/ . Chromium Blog, 2025-07. Medium-low (qua tóm tắt tìm kiếm). Lớp: pattern.

14. Bài Intel "Software vs. GPU Rasterization in Chromium" (2016): kết luận chọn CPU hay GPU raster theo heuristic từng trang; CPU raster với zero-copy tránh upload. Cũ hơn 2 năm, vượt ngưỡng pattern. URL: https://www.intel.com/content/www/us/en/developer/articles/technical/software-vs-gpu-rasterization-in-chromium.html . Low (quá cũ, chỉ qua tóm tắt). Lớp: performance-number (nhưng không rút được con số nào).

## Cách đọc trang GPU (từ nguồn đã có)

Xem các dòng "Compositing", "Rasterization", "Canvas", "Video Decode" trong mục Graphics Feature Status; giá trị "Software only, hardware acceleration unavailable" hoặc "Disabled" là dấu hiệu fallback. Phần "Problems Detected" và "Driver Bug Workarounds" nêu mục blocklist nào áp dụng (kèm ID mục như 122/145/163). Trường "GL_RENDERER" hoặc "Active GPU" ra SwiftShader/"Microsoft Basic Render Driver" cho thấy chạy toàn CPU. Phần này suy từ các báo cáo người dùng (mục 4), không phải tài liệu chính thức hiện hành; hãy xác nhận trên máy thật.

## Điều đã tìm nhưng KHÔNG thấy

- Số đo độ trễ/thông lượng chính thức của software compositing so với GPU compositing trong Chromium: không tìm thấy design doc hay số đo nào.
- Chi phí của display scaling 200% trong raster phần mềm: không tìm thấy nguồn Chromium nào. Chỉ có suy luận số học (chưa là bằng chứng): 200% tuyến tính nhân 4 số pixel của cùng vùng CSS; cần kiểm bằng đo.
- Mục blocklist nêu tên Iris/Iris Pro 5100/5200, HD 4000/4400/4600/5000/6000 hay R9 M370X: không có (trừ id 181 cho HD 4400, thiết bị 0x0a16, chỉ ảnh hưởng Graphite).
- Phiên bản driver Boot Camp thực tế Apple phát hành cho MacBook Pro 2015 (Intel 6100/5300? và R9 M370X) và việc chúng nằm ngoài/ trong ngưỡng nào: không tra.
- Bằng chứng riêng của WebView2 về tỉ lệ fallback GPU cao hơn hay đường compositing khác Edge: không có.
- Báo cáo cụ thể về MacBook Pro 2015 Boot Camp bị "Software only" trong Chromium: không có (kết quả tìm kiếm chỉ chung chung).
- `disable-software-rasterizer` còn ở Chromium 156 hay không: không xác nhận.
- Cả 156.0.4314.58 gắn với nhánh mã nguồn nào: không đối chiếu.

## Hướng nên đào tiếp

- Fetch trực tiếp `edge://gpu` trên máy đó (hoặc `chrome://gpu` dump), đây là bằng chứng quyết định; lấy dòng Compositing/Rasterization/Canvas, Problems Detected, Active GPU, driver version, D3D11 feature level.
- Đọc `content/public/common/content_switches.cc` và `ui/gl/gl_switches.cc` để xác nhận `disable-gpu`, `disable-software-rasterizer`, `force-gpu-rasterization` ở nhánh tương đương 156.
- Đọc `gpu/config/gpu_driver_bug_list.json` (không chỉ software_rendering_list) và `gpu/ipc/service` cho điều kiện Windows tắt GPU compositing (D3D11 init fail, GPU process crash 3 lần thì rơi về software).
- Tìm issue Chromium về "DirectComposition software" và chi phí `GetDC`/`BitBlt` software output device; và doc `docs/gpu` trong cây nguồn.
- Đo A/B trên máy Boot Camp: chạy WebView2 với `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--ignore-gpu-blocklist` và với `--disable-gpu`, so độ trễ gõ; thêm `force-device-scale-factor=1` để cô lập chi phí pixel 200%.
