---
type: ad-brief
title: "Hồ sơ cho `AD-60`: **dịch lô gửi các câu liền nhau trong một lượt gọi, tối đa 100 câu, và mọi lượt đều đối chiếu chỉ số từng câu**"
status: done
created: 2026-10-27
skill: bmad-architecture
---
# Hồ sơ cho `AD-60`: dịch lô gom các câu liền nhau, tối đa 100 câu mỗi lượt gọi

**Ngày:** 2026-10-27 · **Người soạn:** Winston · **Người ký:** Ice
> 🔵 2026-10-27: AD-60 đã viết vào spine (§AD-60), đã qua Reviewer Gate và một lượt kiểm lại (`architecture-auratranslate/reviews/review-ad-60-*-2026-10-27.md`), Ice đã ký ngày 2026-10-27. Sau gate, Ice đổi một điểm: câu `is_omitted` không bao giờ được gửi lên nhà cung cấp, kể cả làm ngữ cảnh (khác Q1 ở dưới). Đường dịch một câu thì gắn cờ `maybe_truncated`. Spine là bản có hiệu lực; các mục bên dưới là bản nháp trước gate.
**Nguồn gốc:** FR70, FR71, FR73, FR74, FR75, FR76; khuyến nghị R2 (phương án P) trong `../research-llm-translation-cost-and-quality/research-llm-translation-cost-and-quality.md`. Mã hiện tại: `../research-ai-translation-pipeline/research-ai-translation-pipeline.md` §2, §3, §5, §7.
**Bằng chứng đo:** `do-m2.md`, cùng script và JSON trong `do-m2/`.
**Mốc cây nguồn:** `7bd1f90ca2ccd15b661850f3cfd60359df6bdc66`. **Số AD trống kế tiếp: 60.** Spine dừng ở AD-59; các hồ sơ `ad-brief-*` chỉ dùng tới AD-58.

## Quyết định Ice đã ký (2026-10-27)

1. **Chọn phương án P:** gom nhiều câu vào một lượt gọi. Phương án C (mỗi câu một request, thêm câu kề) bị loại.
2. **"Dịch cả Chương"** gom cả Chương. **"Dịch các câu đã chọn"** cũng gom.
3. **Chương dài hơn trần thì tự chia** thành nhiều lượt gọi tuần tự, vẫn trong một thao tác của người dùng.
4. **Vùng chọn không liền nhau** thì mỗi đoạn liền nhau đi riêng một hoặc nhiều lượt.
5. **Trần là 100 câu mỗi lượt**, chọn sau khi đo 100, 200 và 300 (bảng ở `do-m2.md`).
6. Phép đo đi trước, AD đi sau.
7. **Q1:** câu ngăn (đã dịch, hoặc `is_omitted`) nằm chung lượt, đi kèm như khối ngữ cảnh chỉ đọc, không dịch lại.
8. **Q2:** giữ 3 cặp TM mỗi câu, bỏ trùng trong lượt.
9. **Q3:** câu thiếu trong một lượt đã trọn thì chạy tiếp lượt sau; câu đó thành *chưa dịch*.
10. **Q4:** phần dặn định dạng khối là hằng số của app, gửi sau system prompt của người dùng.
11. **Q5:** sót chữ Hán thì gắn cờ cảnh báo trên hàng, không tự loại.

## Vì sao cần một AD

- **AD-14** ghi hàm lắp prompt nhận "(câu nguồn, scope, Glossary, TM)", tức đúng một câu. Gom câu nghĩa là đổi đầu vào của một invariant.
- **Spec 4.9** (commit `915f1af`) có quy tắc "lỗi đầu tiên dừng lô và nêu tên câu". Quy tắc này phải nhường chỗ cho ngữ nghĩa *thành công một phần*.
- Hình dạng dây `AiTranslateBatchEventWire` và bản ghi của FR71 đều đổi.

## Điều phép đo buộc AD phải giải

Mô hình đo là `gemini-3.5-flash-lite`, đi đường streaming qua endpoint tương thích OpenAI, đúng như app.

1. **⚠️ Cắt ngang im lặng.** Cả Chương 667 câu trong một lượt bị cắt cả 3 lần chạy. Luồng có `[DONE]` nhưng không có `finish_reason`, và mất 292–525 câu. Client ở `7bd1f90` coi "`[DONE]` có chữ" là thành công.
2. **Rơi câu khi mô hình báo `stop`:** 1–2 trên 356 câu, có một câu đầy đủ.
3. **Dấu mở viết sai** (thiếu `<<<`): 2–11 trên 667 câu.
4. **Câu cuối Chương bị rơi** 2 lần.
5. **Sót chữ Hán thật** ở một câu, lặp lại ở cả hai lượt zh. Có những câu cố ý giữ chữ Hán để giải nghĩa chữ.
6. **Tiền và thời gian không phụ thuộc kích thước lượt** từ 100 tới 300 câu: khoảng $0,06 và 100 giây cho 667 câu. Câu đầu tiên hiện ra sau tối đa khoảng 3 giây.

## Bất biến đang đứng mà AD phải tôn trọng

- **AD-22 và FR75:** dùng một Channel. Không tự gửi lại. Thử lại do người dùng chủ động. Huỷ được giữa chừng.
- **FR74:** kết quả hiện dần theo từng câu.
- **FR72 và AD-47:** kết quả AI không tự ghi vào Editor. Lượt "đưa sang" là lượt ghi không-phải-người-dùng. Vì vậy một câu lệch chỉ số **không bao giờ** được hiện dưới tên câu khác.
- **AD-36 và FR70:** chỉ chèn mục Glossary đã chốt; mỗi câu vẫn được quét.
- **AD-14:** prompt do một hàm thuần lắp; không nối chuỗi ở chỗ gọi.
- **AD-21:** Rust không trả câu hiển thị; trạng thái từng câu đi qua dây dưới dạng dữ liệu.

## Bản thảo luật cho AD-60

1. **Lượt gọi.**
   - Một lượt gom các câu cần dịch, liền nhau theo `ord` trong cùng một Chương, tối đa `TRANSLATE_CALL_MAX_SEGMENTS = 100` câu.
   - Một thao tác "Dịch cả Chương" hay "Dịch vùng chọn" sinh ra nhiều lượt. Các lượt chạy tuần tự: lượt k+1 chỉ gửi đi khi lượt k đã kết thúc.
   - Huỷ được kiểm trước mỗi lượt và giữa các khung SSE.
   - Dịch một câu tại con trỏ giữ nguyên như hiện nay.
   - **Câu ngăn (Q1).** Câu đã dịch hoặc `is_omitted` nằm giữa các câu cần dịch thì đi kèm trong lượt dưới dạng khối ngữ cảnh chỉ đọc `<<<CONTEXT … CONTEXT>>>`: mang nguồn, và bản dịch hiện có nếu có. Khối này không đánh số và không được dịch lại. Trần 100 chỉ đếm khối cần dịch.
   - Một đoạn ngăn dài quá `G` câu thì cắt sang lượt mới, và lượt mới mang theo `G` câu đứng trước làm ngữ cảnh. Đề xuất `G = 3`, theo DelTA: 3 cặp phía trước đã cho phần lớn mức cải thiện. Số này chưa ký.
2. **Lắp prompt (🔵 cho AD-14).**
   - Hàm thuần mới nhận danh sách câu của lượt, cộng Glossary và TM đã quét *theo từng câu*, và trả về prompt hoàn chỉnh. Mỗi câu nằm trong một khối `<<<SEGMENT n … SEGMENT>>>`, với n đếm từ 1 trong lượt.
   - Glossary của các câu gộp lại và bỏ trùng theo thuật ngữ. TM giữ tối đa 3 cặp mỗi câu và bỏ trùng trong lượt (Q2).
   - Phần dặn định dạng khối là hằng số của app, đặt **sau** system prompt mà người dùng sửa (Q4). Nhờ vậy người dùng không xoá mất được phần định dạng mà bộ đọc cần.
3. **Đọc luồng.**
   - Bộ đọc tách khối ngay trong luồng SSE. Có chữ bên trong khối n thì phát `Token{segment_id}`; gặp dấu đóng của khối n thì câu đó thành *đã nhận*.
   - Khối thiếu `<<<` nhưng có `SEGMENT n` và dấu đóng thì vẫn được nhận.
   - Usage gắn theo lượt, không chia cho từng câu. Đây là sự kiện mới, ví dụ `CallDone{segment_ids, usage}`, đúng với FR76 "mỗi lần gọi".
4. **Đối chiếu chỉ số, ở mọi lượt.**
   - Một lượt *trọn* khi và chỉ khi `finish_reason == "stop"` và mỗi chỉ số từ 1 tới n xuất hiện đúng một lần.
   - Khớp **chỉ theo chỉ số**, không bao giờ đoán theo độ dài hay nội dung để dịch chỗ cho vừa.
   - Chỉ số trùng, hoặc nằm ngoài khoảng 1..n, thì câu đó thành **chưa dịch**.
   - Thiếu `finish_reason`, hoặc `finish_reason` khác `stop` (kể cả `length`), thì mọi câu chưa có dấu đóng thành **chưa dịch**. Các khối đã đóng trước đó vẫn được giữ.
5. **Chính sách khi hỏng.**
   - Lỗi từ nhà cung cấp (non-2xx, hoặc luồng đóng mà không có `[DONE]`) dừng cả thao tác và nêu tên lượt đó. Kết quả của các lượt trước được giữ. Đây là AD-59 áp ở cấp lượt.
   - Câu *chưa dịch* trong một lượt đã trọn **không** dừng thao tác (Q3).
   - **Sót chữ Hán (Q5).** Nguồn là tiếng Trung mà bản dịch của một câu còn ký tự Hán thì câu đó mang cờ cảnh báo, gửi qua dây dưới dạng dữ liệu (AD-21). Không câu nào bị tự loại, vì có câu cố ý giữ chữ Hán để giải nghĩa.
6. **Không tự thử lại (AD-22 giữ nguyên).**
   - Nút thử lại chỉ gửi các câu *chưa dịch* hoặc *lỗi*.
   - Các câu đó được gom lại thành các đoạn liền nhau, mỗi đoạn tối đa 100 câu, theo đúng luật 1.
7. **FR71.**
   - Bản ghi prompt là bản ghi của lượt: prompt, system và danh sách `segment_id`.
   - Màn xem prompt của một câu hiện prompt của lượt chứa câu đó và đánh dấu khối của nó.
   - `sent_usage` là usage của lượt.
8. **Test canh.** Mọi ca đi qua cổng nghe cục bộ với các khung viết tay:
   - (a) Luồng có `[DONE]` mà không có `finish_reason`, dừng giữa khối k ⇒ câu k..n *chưa dịch*. Đối chứng đỏ bằng cách GỠ phép kiểm `finish_reason`.
   - (b) Lượt có `stop` nhưng thiếu khối k ⇒ chỉ câu k *chưa dịch*.
   - (c) Chỉ số trùng ⇒ câu đó *chưa dịch*, không câu nào khác bị dịch chỗ.
   - (d) Dấu mở thiếu `<<<` ⇒ khối vẫn được nhận.
   - (e) Huỷ giữa hai lượt ⇒ đúng k lượt đã gửi.
   - (f) Vùng chọn gồm hai đoạn rời ⇒ hai lượt.
   - (g) 250 câu ⇒ ba lượt: 100, 100, 50.
   - (h) `finish_reason: length` ⇒ giống (a).
   - (i) Câu ngăn ⇒ có trong prompt dưới dạng khối ngữ cảnh, và không có sự kiện `Token` nào mang `segment_id` của nó.
   - (j) Có ký tự Hán trong bản dịch ⇒ câu mang cờ cảnh báo và vẫn ở trạng thái *đã nhận*.
9. **Đổi gì, không đổi gì.**
   - Đổi: AD-14 nhận một dòng 🔵 trỏ về đây. Dòng "Giữ số dòng và dấu câu tương ứng với đoạn nguồn" của system prompt mặc định viết lại cho dạng nhiều khối. Quy tắc dừng lô của spec 4.9 nhận một dòng 🔵.
   - Không đổi: AD-22, AD-15, AD-29, AD-36, AD-47. Proofreader vẫn theo khuôn của nó (commit `dc8b39d`).

**Phương án bị loại:**
- **Cả Chương một lượt:** hỏng 3/3 lần ở Chương 667 câu (`do-m2.md`).
- **Trần 200 hoặc 300 câu:** đều hợp lệ khi đo, Ice chọn 100.
- **Phương án C:** Ice chọn P.
- **Batch API:** không hợp khi người dùng đang chờ, và cần thêm một điểm mạng (AD-15).

## Câu hỏi còn mở

- **Độ dài đoạn ngăn `G`** (đề xuất 3). Có thể chốt ở Reviewer Gate.

## Ngoài phạm vi AD, ghi làm nợ khi lập story

- Bảng giá thêm hàng cho `gemini-3.5-flash-lite` (cấu hình đang dùng) và `gemini-3.8-flash` (preset). Tính cả `cached_tokens`, vì endpoint tương thích có báo số này (`do-m2.md`).
- Đo lại với `gemini-3.8-flash`: R1 (thinking) vẫn còn mở với mô hình đó.
