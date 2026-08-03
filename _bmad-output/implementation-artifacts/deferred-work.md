
## Deferred from: code review of 1-1-mui-tham-do-font-do-dung-luong-that-va-ra-giay-phep (2026-08-03)

- **`.memlog.md` của architecture còn `scope: 112 FR, 16 NFR`** — spine ghi 131 FR / 19 NFR, PRD hiện có 132 FR. Tệp memlog đã bị chạm trong lượt này (bump `updated`) nhưng dòng `scope` lỗi thời vẫn để nguyên. Có sẵn từ trước, không do Story 1.1 gây ra.
- **Chưa đo trên Apple Silicon / universal binary / Windows ARM64** — máy đo là Intel x86_64. Chênh lệch font gần như chắc chắn không đổi theo kiến trúc nhưng baseline thì có, và universal binary nhân đôi baseline. Báo cáo đã nêu ở §Việc chưa làm được nhưng không story nào nhận việc.
- **Chưa khai artifact phát hành chính thức cho Windows** — Tauri dựng được cả `.msi` lẫn NSIS. AC1 đòi `.msi`, nhưng nếu bản phát hành thật là NSIS thì con số NFR6 không áp cho thứ người dùng tải về. Thuộc Story 1.3 / 10.2.
- **Đường nạp font chưa từng chạy trên Windows** — cấu hình CSP + `assetProtocol` scope + `FontFace` API mới chỉ kiểm chứng trên macOS. CI của Story 1.3 chỉ `cargo test` và build, không xác minh font nạp được lúc chạy. Thuộc Story 1.3 / 1.4.
- ✅ **ĐÃ ĐÓNG 2026-08-03 ngay trong lượt rà soát** — ~~Ba tệp giấy phép OFL chưa có AC nào đưa vào bundle~~ — báo cáo kết luận *"cả ba tệp giấy phép gốc phải đi kèm bản phát hành (FR38, FR109)"* nhưng không story nào cưỡng chế điều đó, và phép đo 20,30 MiB cũng chưa gồm chúng. Thuộc Story 1.2 / 10.5.
- **Rà NFR15 chưa đọc name ID 13/14 của tệp font phát hành** — đã mở `LICENSE` / `OFL.txt` trong zip mà đọc (đúng yêu cầu "rà tường minh"), nhưng chưa đối chiếu với trường License Description nhúng trong chính tệp `.otf`/`.ttf` sẽ được đóng gói.
