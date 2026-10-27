# Đo M2: dịch cả Chương trong một lượt gọi

Ngày đo: 2026-10-27, trên máy của Ice. Mô hình `gemini-3.5-flash-lite`, cấu hình đang dùng trong app. Khoá lấy từ Keychain của app.

Đường gọi:
- **Endpoint tương thích OpenAI**, streaming. Thân request có cùng các trường như `build_request_body` ở `7bd1f90`, cộng `reasoning_effort` khi đo mức minimal.
- **API gốc**, không streaming, chỉ dùng để so sánh.

Script nằm trong `do-m2/`. Kết quả từng lượt là `do-m2/*.json`.

## Dữ liệu

Văn bản công cộng, vì máy không có Chương thật nào dài hơn 10 câu:
- **zh:** Tây Du Ký hồi 1, lấy từ Wikisource; 356 câu, khoảng 11,2k token đầu vào.
- **en:** "A Scandal in Bohemia", lấy từ Project Gutenberg #1661; 667 câu, khoảng 19,5k token đầu vào.

Câu do script tự cắt theo dấu câu, không qua bộ cắt của app.

Prompt:
- System prompt mặc định, viết lại cho dạng nhiều khối.
- Phần hướng dẫn của mẫu `literary`.
- Mỗi câu một khối `<<<SEGMENT n … SEGMENT>>>`.
- Không có Glossary, không có TM.

## Kết quả

| Lượt | Đường gọi | Gửi | Nhận | Thiếu | `finish_reason` | Token vào (đã cache) | Token ra | Giây |
|---|---|---|---|---|---|---|---|---|
| zh, thinking mặc định | tương thích, stream | 356 | 354 | 2 (136, 166) | `stop` | 11.168 | 12.948 | 38,4 |
| zh, minimal | tương thích, stream | 356 | 355 | 1 (22) | `stop` | 11.168 (8.165) | 13.011 | 37,8 |
| en, mặc định (1) | tương thích, stream | 667 | 375 | 292 | không có | 19.504 | 13.119 | 39,0 |
| en, mặc định (2) | tương thích, stream | 667 | 142 | 525 | không có; có `[DONE]` | 19.504 (16.353) | 5.184 | 17,4 |
| en, minimal | tương thích, stream | 667 | 142 | 525 | không có | 19.504 | 5.145 | 16,9 |
| en, mặc định | gốc, không stream | 667 | 666 | 1 | `STOP` | 19.503 | 22.464 | — |
| en, mặc định | gốc, stream | 667 | 0 | — | — | — | — | 222 (không nhận được khung nào) |

Ghi chú:
- Cột "Nhận" đếm cả khối có dấu mở viết sai. zh-default có 2 khối mất `<<<` (355, 356). zh-minimal có 2 khối như vậy.
- Thời gian tới câu đầu tiên ở mọi lượt stream là 2,1–2,4 giây.

## Điều số đo nói

1. **Rơi câu vẫn xảy ra khi mô hình báo `stop`.** Mỗi lượt zh rơi 1–2 trên 356 câu, có một câu đầy đủ (câu 166). Như vậy phải đối chiếu số khối ở *mọi* lượt gọi, không chỉ khi lượt đó báo lỗi.
2. **⚠️ Cắt ngang im lặng.** Cả ba lượt en qua endpoint tương thích đều dừng giữa câu:
   - Không có `finish_reason`, nhưng vẫn có `[DONE]`.
   - Hai lần dừng đúng ở câu 143.
   - Không phải do trần token: lượt zh ra 13k token vẫn qua trọn.
   - Client hiện tại coi "`[DONE]` có chữ" là thành công (`src-tauri/src/core/ai/client.rs:468 (translate) @ 7bd1f90`). Với cách gom cả Chương, 292–525 câu sẽ biến mất mà không có lỗi nào báo ra. Đây đúng là lớp lỗi "rỗng im lặng".
   - Nguyên nhân chưa rõ. Cùng Chương đó gọi API gốc không streaming thì chạy xong (`STOP`, 666/667 câu). Lượt streaming qua API gốc không cho kết quả vì mạng không ổn định (trước đó còn một lần `IncompleteRead`).
3. **Sót chữ Hán.** Câu 104 ra "Thủy l帘 động" ở cả hai lượt zh. Các câu 332–336 cố ý giữ chữ Hán vì đang giải nghĩa chữ, nên bộ kiểm không được coi mọi chữ Hán còn lại là lỗi.
4. **Dấu khối viết sai** ở 2 trên 356 câu. Bộ đọc cần chấp nhận dạng thiếu `<<<`, hoặc coi những khối đó là thiếu.
5. **Thinking không phải khoản tiền chính với mô hình này.**
   - `total_tokens` luôn bằng token vào cộng token ra.
   - Mức mặc định và minimal ra gần bằng nhau: 12.948 so với 13.011 token.
   - Câu hỏi về R1 vẫn còn mở với `gemini-3.8-flash`, preset của app, vì lượt này chưa đo mô hình đó.
6. **Endpoint tương thích có báo token cache.** Lượt gọi thứ hai trên cùng một tiền tố trả về `prompt_tokens_details.cached_tokens` (8.165/11.168 và 16.353/19.504). Vậy câu hỏi [19] trong báo cáo nghiên cứu đã có câu trả lời là *có*. Bảng giá của app hiện chưa tính phần cache.
7. **Chi phí một Chương** theo giá $0,30 / $2,50:
   - zh: khoảng $0,036.
   - en, đường gốc trọn Chương: khoảng $0,062.
   - Token ra chiếm khoảng 90% tiền.

## Kích thước lượt (đo lần hai, cùng ngày)

Chương en 667 câu, chia thành các lượt liền nhau, gọi tuần tự qua endpoint tương thích có streaming, mỗi kích thước chạy một lần. Kết quả: `do-m2/en{100,200,300}_*-run.json`.

| Kích thước lượt | Số lượt gọi | Lượt có `finish_reason: stop` | Câu thiếu | Khối mở sai dấu | Token vào (đã cache) | Token ra | Tổng giây | Câu đầu tiên chậm nhất (giây) | Tiền |
|---|---|---|---|---|---|---|---|---|---|
| 100 | 7 | 7/7 | 0 | 8 | 20.382 (0) | 21.806 | 98,5 | 3,0 | $0,061 |
| 200 | 4 | 4/4 | 1 (câu 667, câu cuối Chương) | 2 | 19.959 (0) | 22.372 | 102,6 | 3,1 | $0,062 |
| 300 | 3 | 3/3 | 0 | 11 | 19.818 (6.119) | 22.220 | 114,3 | 3,2 | $0,060 |
| 667 (cả Chương) | 1 | 0/3 lần chạy | 292–525 | — | 19.504 | 5–13k | 17–39 | 2,4 | — |

Điều số đo nói:
- Ở 100, 200 và 300 câu, mọi lượt đều chạy trọn. Cả Chương 667 câu thì hỏng cả 3 lần. Ngưỡng hỏng nằm đâu đó giữa 300 và 667 câu và chưa đo được. Lượt zh 356 câu (11k token vào, 13k token ra) thì qua trọn.
- Tiền và thời gian gần như không đổi theo kích thước lượt, vì 90% tiền là token ra, mà số token ra không đổi. Phần system prompt lặp lại thêm ở các lượt nhỏ chưa tới 600 token.
- Câu cuối Chương bị rơi 2 lần: một lần ở lượt 200, một lần ở lượt gọi API gốc trọn Chương.
- Mỗi kích thước chỉ chạy một lần, nên mẫu còn nhỏ.

## Còn mở

- Nguyên nhân bị cắt ngang khi stream qua endpoint tương thích, và ngưỡng hỏng giữa 300 và 667 câu.
- Đo lại với `gemini-3.8-flash`.
- Glossary và TM làm token vào tăng bao nhiêu.
