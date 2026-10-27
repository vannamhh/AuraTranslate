# Review AD-60 — lăng kính thực tế (reality), 2026-10-27

Phạm vi: AD-60 và dòng 🔵 dưới AD-14. Đối chiếu với commit `7bd1f90`, các JSON trong `ad-brief-dich-ca-chuong-mot-luot/do-m2/`, và tài liệu Gemini.

**Kết luận:** Mã nguồn và số đo phần lớn khớp thực tế, nhưng AD-60 dựa vào hành vi `finish_reason` của Gemini mà tài liệu chính thức không bảo đảm. Nó cũng gọi "chạy trọn" một cấu hình có 1 câu thiếu, và dùng mẫu 1 lượt mỗi cấu hình. Chưa có điểm chặn, nhưng nên sửa F1–F4 trước khi coi là đã kiểm chứng.

## Phát hiện

### F1 — Trung bình — "Luồng có `[DONE]` nhưng không có `finish_reason`" chỉ có bằng chứng JSON cho 1 trong 3 lượt
- **Claim (Prevents 1):** cả 3/3 lượt 667 câu có `[DONE]` mà không có `finish_reason`.
- **Thực tế:** `do-m2/en-default-2.json` có `"done": true` và `last_events` kết thúc bằng `[DONE]`. `en-default-1.json` và `en-minimal-1.json` không có khoá `done` nào. `do-m2.md` ghi `[DONE]` chỉ ở dòng `en (2)` và mục "Điều số đo nói" 2 nói chung cho cả ba. Hai lượt kia nhiều khả năng đo trước khi `measure.py` ghi `done`. `finish_reason` là `null` ở cả ba, đúng.
- **Hệ quả:** nếu hai lượt kia thực ra đóng luồng không có `[DONE]`, chúng sẽ thành lỗi theo mục 6 (đóng không có `[DONE]`), không phải "cắt im lặng". Cần đo lại hoặc sửa lời cho đúng bằng chứng (1/3 xác nhận `[DONE]`, 3/3 không `finish_reason`).
- **Dẫn:** `do-m2/en-default-1.json`, `do-m2/en-minimal-1.json`, `do-m2/en-default-2.json`, `do-m2.md` bảng kết quả.

### F2 — Trung bình — Nền tảng của mục 5 là hành vi không được tài liệu hoá
- **Claim:** một lượt trọn khi và chỉ khi `finish_reason == "stop"`; thiếu `finish_reason` hay `length` thì mọi câu chưa đóng là *chưa dịch*.
- **Thực tế (WebFetch `https://ai.google.dev/gemini-api/docs/openai`, 2026-10-27):** trang không nêu giá trị `finish_reason` nào, không nêu cách luồng kết thúc (`[DONE]`), không nêu cắt ở trần token. Chỉ có câu "Support for the OpenAI libraries is still in beta". `stop` xuất hiện trong dữ liệu đo thực (14/14 lượt chia nhỏ, zh 2/2) và trong chính mã (`EmptyReply { finish_reason }` ở `client.rs:130`), nhưng `length` chưa từng được quan sát: không có lượt đo nào kết thúc bằng `length`. Việc luồng cụt mà thiếu `finish_reason` là quan sát một lần (ngày 2026-10-27, model `gemini-3.5-flash-lite`), "nguyên nhân chưa rõ" theo `do-m2.md`.
- **Hệ quả:** quy tắc "chỉ `stop` mới trọn" an toàn theo hướng thận trọng (không bao giờ nhận nhầm), nhưng một nhà cung cấp khác hoặc bản Gemini khác trả `finish_reason` tên khác (ví dụ `STOP` viết hoa ở API gốc, `do-m2.md` đã thấy `STOP`) sẽ khiến mọi câu thành *chưa dịch*. AD nên nói rõ so sánh có phân biệt hoa thường hay không, và ghi đây là hành vi quan sát được, không phải cam kết của nhà cung cấp.
- **Dẫn:** WebFetch trên; `do-m2.md` bảng dòng "gốc, không stream" `STOP`; `client.rs:588`.

### F3 — Trung bình — "Trần 200 hoặc 300 câu: đo đều chạy trọn" mâu thuẫn với định nghĩa trọn của chính AD
- **Thực tế:** `do-m2/en200_4-run.json` có `missing` = 1 phần tử (câu cuối Chương, 667), dù `finish_reason: "stop"`; `unparsed_tail` cho thấy khối 66 đầy đủ nhưng khối 67 vắng. Theo mục 5 lượt này không trọn. Câu "14/14 lượt có `stop`, thiếu 0, 1 và 0" ở phần bằng chứng đúng; câu "đều chạy trọn" ở Phương án bị loại thì không.
- **Mẫu nhỏ:** mỗi cấu hình chỉ một lần chạy trên một Chương (100: 7 lượt, 200: 4, 300: 3, tổng 667 câu). Quyết định 100 so với 200/300 dựa trên một thử nghiệm không lặp, một văn bản en, một mô hình. Con số 1 câu thiếu ở 200 so với 0 ở 100 và 300 nằm trong nhiễu.
- **Hệ quả:** lý do loại 200/300 ("cùng tiền, cùng thời gian") đúng, nhưng chữ "trọn" cần sửa; và nên ghi thẳng mẫu là 1 lần/cấu hình.
- **Dẫn:** `do-m2/en200_4-run.json`; `do-m2.md` dòng 69.

### F4 — Thấp — Số zh "1–2 trên 356, có một câu đầy đủ" phụ thuộc cách đếm
- **Thực tế:** JSON `zh-default-1.json` có `missing` = [136,166,355,356] (4 câu), `zh-minimal-1.json` có 1. `do-m2.md` tính 355 và 356 là "nhận" vì khối mất `<<<` (khớp mục 4 "Dòng mở thiếu `<<<` vẫn được nhận"), nên báo 2 và 1. Con số "1–2" đúng theo cách đếm khoan dung, sai theo `missing` trong JSON. Cũng có `han_left` = 6 câu (104, 332–336, 339) mà AD chỉ nhắc ở mục 6. "có một câu đầy đủ" (câu 166) chỉ có trong `do-m2.md`, JSON không chứng minh được.
- **Dẫn:** `do-m2/zh-default-1.json`, `zh-minimal-1.json`, `do-m2.md` bảng và ghi chú 1.

### F5 — Thấp — Mã nguồn: chính xác, nhưng "phải trả `finish_reason` ra ngoài" là việc còn lại, không phải thực trạng
- **Đã xác nhận ở `7bd1f90`:**
  - `assemble_prompt` tại `core/ai/rag.rs:704` nhận một `sentence` (khớp mô tả "một câu", 🔵 dưới AD-14 đúng).
  - `translate` tại `client.rs:468`, `interpret_sse_event` tại `client.rs:770`; `SseEventOutcome::Chunk` mang `finish_reason` (dòng 767) và `translate` đã lưu `last_finish_reason` (dòng 535, 588), nhưng chỉ dùng cho `EmptyReply`. `TranslateOutcome::Done(usage)` không mang nó. Vậy mô tả "lượt phải trả `finish_reason` ra ngoài" đúng là việc cần làm. Lưu ý commit `7bd1f90` (sửa 4-14) vừa thêm `finish_reason` vào khung; AD nên tránh viết như thể trước đó chưa có.
  - `prepare_batch_call_with` (`aitranslate.rs:558`), `run_batch_call` (:1034), `AiTranslateBatchEventWire` (:978), `batch_stopped_error` (:218) đều có. `run_batch_call` hiện chạy mỗi câu một `translate` và phát `Skipped` cho câu `is_omitted`, đúng như AD mô tả để thay.
  - `AssembledPromptRecord` (`aiprompt.rs:377`) có `segment_id` đơn và `sent_usage` ("None ... when the send was a batch"): mục 8 đổi cả hai.
  - `src/config/aitranslate.ts` và `src/aiTranslateBatchState.ts` tồn tại; trạng thái hàng là `pending/running/done/skipped/error` — chưa có trạng thái *chưa dịch* hay *đã nhận*, đó là từ mới AD đưa vào.
  - `is_omitted` đúng tên (`aitranslate.rs:84-86`). Không có hằng `TRANSLATE_CALL_MAX_SEGMENTS` trong mã (mong đợi, là hằng mới).
  - Thứ tự phân giải system prompt: `resolve_system_prompt` (`core/aiconfig/mod.rs:235`) lấy giá trị đã phân giải (Work kế thừa Global, mục 151), rồi mặc định `DEFAULT_SYSTEM_PROMPT`; khớp "Work, rồi Global, rồi mặc định". Prompt trống trả `None` — khớp ca (k).
  - Commit `915f1af` và `dc8b39d` đều là commit thật.
- **Điểm nhỏ:** AD nói ở mục 4 "`Token` mang `segment_id`" — khớp `AiTranslateBatchEventWire::Token { segment_id, text }`. Mục 5 nhắc "kể cả `length`"; mã chưa có nhánh nào cho `length` riêng.
- **Dẫn:** `git show 7bd1f90:src-tauri/src/core/ai/client.rs`, `.../commands/aitranslate.rs`, `.../commands/aiprompt.rs`, `.../core/aiconfig/mod.rs`.

### F6 — Thấp (xác nhận khớp) — Số đo chi phí và thời gian
- Cộng từ JSON: 100 câu/lượt tổng 99 s, 20.382 token vào, 21.806 token ra; 200: 103 s, 19.959/22.372; 300: 114 s, 19.818/22.220. Giá $0,30/$2,50 mỗi triệu token cho ra khoảng $0,061, $0,062, $0,060. Khớp "khoảng $0,06 và 100–114 giây" (làm tròn). Số lượt 7/4/3 khớp. Lượt cuối của mỗi cách đều là 67 câu (667 = 6×100+67 = 3×200+67 = 2×300+67). Bảng giá $0,30/$2,50 là của `gemini-3.5-flash-lite`, lấy từ `do-m2.md`, không tự kiểm lại tại trang giá nhà cung cấp trong lượt review này.
- Token đã cache không tính vào chi phí ($0,06) nhưng `do-m2.md` đã ghi "bảng giá của app chưa tính phần cache".

## Điều không kiểm được
- Giá model và việc `gemini-3.5-flash-lite` còn tồn tại: chỉ dựa vào `do-m2.md`, không có nguồn web ngoài.
- Không có tài liệu nào của Google nói luồng có thể kết thúc bằng `[DONE]` mà thiếu `finish_reason`; đây là quan sát thực nghiệm duy nhất (F2).
