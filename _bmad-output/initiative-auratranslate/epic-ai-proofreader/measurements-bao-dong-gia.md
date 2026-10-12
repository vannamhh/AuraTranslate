# Số đo tỷ lệ báo động giả của proofreader (FR81)

Ngưỡng đỗ: tỷ lệ ≤ 20% với ít nhất 30 phát hiện. Số do Ice tạo; agent không điền.

## Quy trình

1. Mở một Chương thật chưa từng quét (không có ghi nhớ), quét trọn đúng một lần.
2. Chờ lượt quét xong hẳn, rồi mới xử lý.
3. Xử lý mọi phát hiện bằng chấp nhận hoặc "không phải lỗi"; không dùng "bỏ qua", không bỏ ghi nhớ, không quét lại.
4. Chạy: `node --no-warnings scripts/measure-proofread-false-alarm.mjs <project.db> --chapter <id> [--scan <id>]`
5. Dán báo cáo vào một khối mới bên dưới; không sửa khối cũ.

## Bản ghi

### <ngày> — <Tác phẩm>, Chương <ord>

- Build / git HEAD:
- scan.id:
- n (reported), filtered:
- Tử số theo loại:
- Tỷ lệ, Wilson 95%:
- Cảnh báo:
- Kết luận:
- Ghi chú của Ice:
