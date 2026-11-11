# Handoff: pha Webview xong, sang pha Tests that move

Đã làm (chưa commit): `src/config/quickBatch.ts` (tham số `trigger`), `src/importPreviewState.ts` (bộ lên lịch `scheduleQuickAuto`, `quickAutoFailed`, cờ `importPreviewQuickAutoPaused`, watcher cấu hình/hoạt động/bề mặt kia), `src/ImportPreviewOverlay.vue` (ẩn nút, `data-ip-quick-paused`), `src/i18n/vi.json` (`tier2_quick_paused`), `tests/frontend/importPreviewQuickBatch.test.ts` (describe "tự dịch các khối đang thấy", 13 ca).

Hành vi cần biết khi viết e2e:
- Lô tự chạy chỉ gửi khối chưa có entry và chưa trong `quickAutoFailed`; sau `done` (deferred 0) khối không có kết quả bị đánh dấu bỏ qua; `done` có deferred chỉ lên lịch lại khi có khối xong.
- Cuộn trong lúc chạy không gửi; xong lô thì lên lịch lại qua `QUICK_AUTO_WAIT_MS` (800 ms), nên giữa hai lô có >= 800 ms.
- Huỷ (Esc, tắt tuỳ chọn, đổi cấu hình) không lên lịch lại tới lần tập khối đang thấy đổi kế tiếp.
- Selector: nút `[data-ip-quick-translate]` ẩn khi tự dịch bật trừ lúc tạm dừng; `[data-ip-quick-paused]`; `[data-ip-quick-running]`.
- Đối chứng đã gỡ thật: khoá đã lỗi (sau dọn kho), chặn khi đang chạy (cả hai chốt cùng lúc vì dư), xoá giờ khi tắt (kèm chốt `quickAiAutoEnabled` ở giờ nổ). Mỗi lần đỏ đúng ca canh.
- `check:panel-refs` đang đỏ 16 lỗi có sẵn (15.4: `settingsQuickDataState`, `quickTranslateRun::purges`, `bootstrap`); phần của tôi (`quickAutoTimer`) đã sửa, không thêm lỗi.
- Chưa đo trên WKWebView; e2e `story-15-6-quick-auto-preview.e2e.mjs` chưa viết.

## Pha Tests that move (đo 2026-11-11, sửa lần hai)

`e2e/specs/story-15-6-quick-auto-preview.e2e.mjs` mới, 2 ca xanh (2 passing, 29,2 s), chạy ngoài sandbox. ESLint sạch. Ca bật khẳng định: tập số đoạn đã gửi bằng đúng tập 60 khối chữ trên trang (không thiếu, không lặp), không lời gọi chồng, không có nút `Dịch các khối đang thấy`.

Số đo (build: debug `--features wdio`, Vite development, WKWebView, provider SSE giả; quần thể: trang 60 đoạn, 60 khối liệt kê, màn đầu + 23 nấc cuộn 0,6 màn, mỗi dừng chờ tới khi mọi khối đang thấy có bản dịch, trần 20 s và quá trần thì đỏ): 22 lời gọi, 60 khối gửi, 60 có bản dịch trong DOM, 0 lặp, 0 chồng, khoảng cách giữa hai lời gọi 805 đến 823 ms (khớp `QUICK_AUTO_WAIT_MS` 800 ms). Tuỳ chọn tắt: cuộn hết trang, 0 lời gọi.

Bản đầu nới lỏng AC (57/60 khối, nuốt timeout) là lỗi của spec, không phải sản phẩm: vòng cuộn bắt đầu cuộn ngay ở màn đầu mà không chờ, nên khối 01, 02, 03 (đoạn đầu, đang thấy lúc mở) rời màn trước khi giờ 800 ms nổ. Khoảng cách tối đa 4133 ms ở lần đo đầu là do các dừng bị bỏ ngang. Sau sửa không còn khối nào đang thấy ở một lần dừng mà không được gửi: không có lỗi sản phẩm.

Lưu ý:
- Chỉ số khối trong yêu cầu là số thứ tự trong lô; spec nhận diện khối bằng số "so NN" trong chữ.
- Không chạy đối chứng gỡ seam ở e2e (đã làm ở vitest).

Sửa lần ba: ca tắt từng đọc lại trang `/a` mà ca bật đã nạp vào kho (một `$APPDATA` mỗi lần chạy e2e), nên 0 lời gọi đúng cả khi tự dịch bật. Nay ca tắt dùng trang `/b` với chữ khác (`Khac NN`), khẳng định có >= 6 khối `Khac` hiện ra và 0 khối có bản dịch (không có gì từ kho). Đối chứng: để `enable(true)` ở ca tắt (chỉ trong spec) thì ca đỏ ở `calls.length - before` (Expected 0, Received 1), ca bật vẫn xanh; khôi phục `enable(false)` thì 2 passing.
