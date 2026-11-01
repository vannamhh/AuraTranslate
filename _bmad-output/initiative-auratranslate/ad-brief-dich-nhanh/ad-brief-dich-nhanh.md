---
type: ad-brief
title: "Hồ sơ bàn giao cho Winston — một `AD` mới: **dịch nhanh bằng model riêng, tuỳ chọn tự chạy, bộ đệm đến khi dọn**"
status: done
created: 2026-11-01
skill: bmad-architecture
---
# Hồ sơ bàn giao cho Winston — dịch nhanh

🔵 2026-11-01: AD-64 đã viết vào spine (§AD-64), qua Reviewer Gate (`architecture-auratranslate/reviews/review-ad-64-*-2026-11-01.md`), Ice ký ngày 2026-11-01.

**Ngày:** 2026-11-01 · **Người bàn giao:** Amelia · **Người nhận:** Winston (architect)
**Baseline:** `abfd6026` · spine dừng ở AD-63, không `ad-brief-*` nào giữ AD-64 ⇒ số kế tiếp **AD-64**.

## 1. Quyết định của Ice

Ý của Ice: *"Dịch thử có một model giá cực rẻ setup riêng dành cho phần dịch nhanh, có tuỳ chọn dịch tự động, lưu data cho đến khi dọn dẹp, có chế độ dọn riêng, vì cần dịch cả phần review khi nhập từ url"*.

- **Model riêng:** cấu hình đầy đủ gồm nhà cung cấp, endpoint, model và khoá; để trống thì dùng cấu hình chính.
- **Tự động ở Bảng chờ Glossary:** dịch khi mở hàng, tra bộ đệm trước. Không dịch trước cả danh sách.
- **Nhập từ URL:** dịch các khối nội dung ở tầng chọn Giữ/Loại khối, không dịch toàn bộ Chương xem trước.
- **Lưu trữ:** giữ đến khi người dùng dọn, có chế độ dọn riêng.

Hai điều này lật quyết định trong `plan-cau-dich-tham-khao-bang-cho-glossary.md`: câu AI giữ trong bộ nhớ, và không tự gọi AI khi chuyển hàng.

## 2. Vì sao cần một AD

AD-15 ghi cả ba điểm ra mạng *"chỉ chạy theo một thao tác người dùng"*, và `prd-auratranslate.md` §Kết nối mạng ghi cả ba *"đều do người dùng chủ động kích hoạt"*. Lời gọi AI khi mở hàng hoặc khi hiện khối không phải một thao tác như vậy, kể cả khi người dùng đã bật tuỳ chọn trước đó. Điểm ra mạng vẫn là adapter `TranslationProvider`, không có điểm thứ tư.

## 3. Sự thật đã đọc trong mã

- `resolve_call_config` đọc năm trường theo hai tầng; `read_api_key` chỉ có một chỗ trong keychain.
- `prepare_context_reference_call` và `ContextReferenceGeneration` (`commands/aitranslate.rs`) là đường dịch một đoạn chữ rời, bộ đếm riêng, không ghi đĩa.
- `assemble_context_reference_prompt` (`core/ai/rag.rs`) là prompt cố định, một câu mỗi lần gọi.
- Màn xem trước nhập chạy trước khi có Tác phẩm nên chưa có `project.db`; bộ đệm dùng chung cho hai nơi chỉ có thể ở `global.db`.
- Chưa có cơ chế dọn cho loại dữ liệu này. Mục `cleanup` hiện có là luật dọn văn bản lúc nhập, không phải dọn dữ liệu.
- Tầng khối đã có `blockFocusedIndex` nhưng chưa có khái niệm dịch khối; số khối mỗi trang và kích cỡ khối chưa đo.

## 4. Câu hỏi cho Winston

1. AD-64 nới AD-15 tới đâu: tuỳ chọn tự chạy mặc định tắt, chỉ áp cho dịch nhanh và không áp cho hai điểm ra mạng kia?
2. Khoá thứ hai lưu ở đâu trong keychain, và `keychain::read` mở rộng thế nào mà không đổi đường của cấu hình chính?
3. Khoá bộ đệm gồm những gì: hash chữ nguồn, model, phiên bản prompt? Đổi model có phải dịch lại không?
4. Cấu hình dịch nhanh nằm ở tầng nào khi `resolve_call_config` đòi `OpenWork`, trong khi màn xem trước chưa có Tác phẩm?
5. Khối ở tầng xem trước gọi từng khối hay gom theo lô như AD-60? Có trần số lượt hoặc số ký tự cho mỗi lượt xem trước không?
6. Dịch tự động huỷ thế nào khi chuyển hàng nhanh, để không tốn lượt cho hàng đã rời đi (AD-22)?
7. Chế độ dọn: xoá tất cả, theo tuổi, hay theo model? Có đặt trần dung lượng cho `global.db` không?
8. Lời gọi tự chạy có cần xem được theo FR71 không? Hiện lần gọi `Dịch thử` không vào `AiCallRecords`.

## 5. Việc sau AD

PRD thêm FR cho dịch nhanh và sửa §Kết nối mạng; `DESIGN.md` và `EXPERIENCE.md` thêm mục Cài đặt và chế độ dọn; Sofia cắt story trong cây vé.
