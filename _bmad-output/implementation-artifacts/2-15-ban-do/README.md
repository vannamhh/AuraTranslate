# Bàn đo bug 2-15 — gõ 10 phím/giây trong 30 giây rồi `Mod+Enter`

Tạo tác của một lượt đo, không vào `package.json`. Bản release thật, phím thật ở tầng hệ điều hành.

- `build-215.sh <repo> <out>`: dựng release của `<repo>` (cây chính hoặc `git worktree` ở `baseline_revision`), nối `bench-215.js` vào `dist/assets/index-*.js` (CSP không cho script riêng), ký ad-hoc, chép `.app` ra `<out>`. Khuôn của `../2-4-ban-do/build-bench.sh`.
- `run-session.sh <nhãn> <app-dir> <122|9850>`: một phiên. Mở app trong HOME nháp, phím nóng Ctrl+Alt+Shift+1/9 tạo Tác phẩm 122/9.850 câu (`create_work_from_text`, câu giống `../2-14-ban-do/`), +2 mở Workspace và đặt con trỏ vào ô đích đầu, `type-10hz.applescript` gõ 300 ký tự ASCII (`delay 0.095`) trong MỘT tiến trình rồi `Cmd+Return`, +3 đổ số qua `put_config`. Cần màn hình mở khoá, ngoài sandbox.
- `analyse.py ket-qua-*.json`: phím cuối (`event.timeStamp` tới frame thứ hai sau `input`), số phím chờ lúc phím cuối được gửi (phím có `input` chạy sau `timeStamp` của phím cuối), `Mod+Enter` tới khi vạch lề hàng có `rule-confirmed`.
- `ket-qua-*.json`: số thô từng phiên (`base-*` là `baseline_revision`, `after-*` là cây làm việc).

Giới hạn: `event.timeStamp` lấy từ sự kiện hệ điều hành của WebKit; bản debug đếm render `PanelFrame` không nằm trong bàn này.
