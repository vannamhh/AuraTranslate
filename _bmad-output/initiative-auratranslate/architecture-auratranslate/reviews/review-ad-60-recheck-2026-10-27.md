# Recheck AD-60 sau bản sửa, 2026-10-27

Phạm vi: AD-60 bản sửa (15 mục Rule), ba dòng 🔵 2026-10-27 dưới AD-14 (dòng 212), AD-47 (dòng 764), AD-59 (dòng 1203), hàng "Gọi AI" của Consistency Conventions. Đối chiếu AD-1, 14, 18, 21, 22, 36, 44, 47, 53, 57, 59 và `core/dict/mod.rs::is_han`.

## Phán quyết

Bản sửa đóng 20 trong 29 phát hiện cũ, 9 còn một phần (nhiều nhất là hình dạng dây, ngữ pháp khối, chủ vòng lặp, marker), nhưng chưa nên ký: còn ba chỗ hai bên thi hành tuân đúng chữ mà vẫn lắp không khớp (N1, N2, N3), và một mâu thuẫn chữ với 🔴 của AD-44 (N4).

## Bảng phát hiện cũ

Ký hiệu: R = rubric, W = reality, A = adversary.

| Phát hiện | Trạng thái | Ghi chú |
|---|---|---|
| R-F1 hình dạng dây, trạng thái câu | Đã đóng | Mục 5 và 6 có đủ tên sự kiện, tập trạng thái, `Discarded` xoá chữ đã stream. Lỗi mới sinh ra từ chính thiết kế này: N1, N2. |
| R-F2 khối ngữ cảnh, dấu va nguồn | Đã đóng (còn dư) | Mục 4: dấu `<<<CONTEXT`, khớp cả dòng, `unclosed`, chèn U+200B. Còn N5 (ZWSP lọt vào `target_text`; dạng lỏng `SEGMENT n` có được escape không). |
| R-F3 trùng từ "chưa dịch", "cần dịch" | Đã đóng | Đổi thành *thiếu kết quả AI*, định nghĩa "cần dịch" cho hai chế độ, nói rõ chỉ ở phiên và không ghi `target_text`. |
| R-F4 chủ chia lượt, hằng số | Một phần | Mục 1 và Binds: Rust chia, hằng ở `core/ai/`. Còn: Binds vẫn liệt kê `src/config/aitranslate.ts` mà không nói nó giữ gì, và không có cổng/test đỏ nếu frontend cài lại số 100. Thấp. |
| R-F5 đoạn rời so với câu ngăn, kế hoạch trôi | Đã đóng | Kế hoạch một lần trên ảnh chụp, `retired` thành thiếu. Lỗ nhỏ về ngữ cảnh đầu lượt: N7. |
| R-F6 khối rỗng, chỉ số trùng/ngoài khoảng, `stop` mà thiếu | Đã đóng | Mục 7 liệt kê đủ; test (c), (d), (e). Chỉ còn mơ hồ giữa mã `unclosed` và `no_finish` ở cuối luồng: N8. |
| R-F7 đứt luồng/huỷ, khối đã đóng | Đã đóng | Mục 7 và 8: giữ khối đã đóng, `CallDone` luôn phát. |
| R-F8 AD-57/AD-36/AD-18 nhiều lượt | Đã đóng | "Một lần mỗi thao tác", phân giải ghi đè trước khi bỏ trùng, sổ chèn cộng theo câu. |
| R-F9 đường đơn, system prompt mặc định, FR145 | Đã đóng | Mục 12, 13, 15. Quyết định "mặc định không đổi" không kèm lý do, nhưng `do-m2.md` đo cả cấu hình `default` và `minimal`, đủ đỡ. |
| R-F10 test thiếu | Một phần | Từ (a) đến (u) phủ gần hết. Mục 14 hứa "mỗi ca đỏ có đối chứng bằng phép GỠ" nhưng chỉ (a), (d), (g), (p) nêu phép gỡ. Thiếu ca cho N1, N2, N3. |
| R-F11 phạm vi bằng chứng | Một phần | Bằng chứng ghi rõ một mô hình, một lần mỗi cấu hình, `gemini-3.8-flash` và nhà cung cấp cục bộ chưa đo, Prevents 3 ghi là suy ra. Chưa có mục nợ có chủ trong `deferred-work.md` (grep: 0 chỗ nhắc AD-60). |
| R-F12 cờ Hán | Một phần | Có dải, theo từng câu, theo hình dạng. Nhưng dải khác `is_han` của AD-44: N4. |
| R-F13 dòng 🔵 AD-14, hàng Gọi AI | Đã đóng | Hàng "Gọi AI" đã có "thử lại chỉ gửi câu thiếu". |
| W-F1 bằng chứng `[DONE]` 1/3 | Đã đóng | Prevents 1 nay nói rõ chỉ lượt thứ ba có `[DONE]`. |
| W-F2 `finish_reason` không được tài liệu hoá | Một phần | Đã ghi "hành vi đo được, không phải cam kết", `length` ghi "chưa đo thấy". Chưa nói so sánh `"stop"` có phân biệt hoa thường hay không (`do-m2.md` thấy `STOP` ở API gốc). Thấp: sai theo hướng an toàn, mọi câu thành `missing(no_finish)` nhưng khối đã đóng vẫn giữ. |
| W-F3 "đều chạy trọn" ở 200/300 | Đã đóng | Nay ghi "stop ở 7/7 lượt" và "lượt 200 thiếu câu cuối", mẫu một lần ghi ở phần bằng chứng. |
| W-F4 đếm zh 1–2/356 | Đã đóng | Ghi "đếm cả khối thiếu `<<<` là đã nhận". |
| W-F5 mã khớp mô tả | Đã đóng | Không phải lỗi spine. |
| W-F6 chi phí, thời gian | Đã đóng | Khớp. |
| A-F1 ngữ pháp đầu ra, nguồn chứa dấu | Đã đóng (còn dư) | Như R-F2. TM con không có dấu đặt tên (N5). |
| A-F2 Token trước hiệu lực, rút chữ, đường Editor | Một phần | `Received`/`Discarded` có, Editor chỉ đọc `Received`. Nhưng `Received` "phát khi khối đóng" mâu thuẫn với điều kiện "chỉ số xuất hiện một lần" chỉ biết cuối luồng: N1. |
| A-F3 ai chia lượt, `params` của AD-59 | Đã đóng | Mục 1, 5, 8; 🔵 dưới AD-59. Thiếu sự kiện kế hoạch để webview biết các lượt chưa gửi: N2. |
| A-F4 marker FR69/FR70, hàm lượt thứ hai | Đã đóng | Mục 3 giữ nghĩa ba marker, "không có hàm thứ hai", sổ chèn của lượt. |
| A-F5 máy trạng thái hàng, vị từ "đã dịch" | Một phần | Có `queued/streaming/received/missing/not_sent`. Webview không có đủ dữ liệu để đặt `queued` và `not_sent`: N2. Thử lại ở chế độ vùng chọn: N6. |
| A-F6 nguồn đổi giữa chừng, kiểm trước khi đưa sang | Một phần | Mục 10 và 🔵 dưới AD-47 có phép kiểm. Chưa nói "ảnh chụp" nằm ở đâu: N3. |
| A-F7 usage theo lượt, bản ghi theo lượt | Đã đóng | Mục 5 và 13. |
| A-F8 ai phán `finish_reason`, Proofreader | Đã đóng | `translate` không phán, Proofreader không đổi, đường đơn có cờ `maybe_truncated` (Ice chọn). |
| A-F9 cờ Hán theo hình dạng | Một phần | Theo hình dạng, theo câu, không chặn. Nhưng định nghĩa "Hán" lệch AD-44: N4. |
| A-F10 câu `is_omitted`, đánh số lệch | Đã đóng | Loại khỏi gửi (Ice chọn, nằm trong Phương án bị loại). ⚠️ ở mục 7 ghi đúng cái mất. |

Đếm theo 29 dòng bảng (13 rubric, 6 reality, 10 adversary): đã đóng 20, một phần 9, mở 0, bác có lý do 0.

## Phát hiện mới

### N1 (cao) `Received` phát khi khối đóng, nhưng điều kiện nhận cần cả luồng

Vị trí: mục 5 (`Received`, "phát khi khối đóng và đủ điều kiện ở mục 7"), mục 6 (không có chuyển `received` sang `missing`), mục 7 ("chỉ số đó chỉ xuất hiện một lần"), test (d).

Hai bên thi hành đúng chữ vẫn lệch: bên A phát `Received` ngay khi khối k đóng (đúng "phát khi khối đóng") rồi khối k thứ hai tới. Mục 7 buộc không lần nào được giữ, mà máy trạng thái ở mục 6 không có đường `received` sang `missing`, và `Discarded` sau `Received` không được định nghĩa. Bên B chờ hết luồng mới phát mọi `Received`, mất tiến trình từng câu. Cùng lỗi với trường hợp chỉ số vượt phạm vi sau đó. Trong lúc đó mục 10 cho đưa sang Editor ngay khi có `Received`, nên bản dịch của lần xuất hiện đầu có thể đã lên `target_text` trước khi lần trùng xuất hiện: đúng Prevents 3.

Sửa: chọn một. (i) `Received` chỉ phát sau khi luồng của lượt kết thúc, ngay trước `CallDone`; `Token` lo phần hiển thị trong lúc stream (rẻ nhất, và đã có cơ chế xoá khi `Discarded`). Hoặc (ii) cho phép `Discarded` sau `Received`, thêm chuyển `received` sang `missing`, và nút đưa sang chỉ bật sau `CallDone`. Thêm ca test: khối k, rồi khối k lần hai ⇒ không `Received` nào của k được phát (hoặc nút đưa sang tắt) cho tới và sau `CallDone`.

### N2 (cao) Webview không có dữ liệu để đặt `queued` và `not_sent`

Vị trí: mục 1 ("Webview không biết ranh giới lượt cho tới khi nhận sự kiện"), mục 5 (`CallStarted` chỉ cho lượt đang chạy), mục 6 (`queued`, `not_sent`), mục 8 ("mọi lượt sau thành `not_sent`"), mục 9.

Khi lượt 2 lỗi, webview chưa từng nhận `CallStarted` của lượt 3 và 4, nên không biết `segment_id` của chúng; không có sự kiện nào liệt kê. Cũng không có cách đặt `queued` cho hàng ở chế độ cả Chương, vì tập "cần dịch" do Rust tính. Một bên thi hành sẽ thêm sự kiện kế hoạch, bên kia để webview tự tính lại từ trạng thái Editor (kéo logic miền sang Vue, trái AD-1 và đúng điều mục 1 cấm). Nút thử lại (mục 9) cần đúng tập này.

Sửa: thêm `Planned { calls: [{ call_index, segment_ids }] }` phát một lần ở đầu thao tác (kế hoạch đã được tính một lần, mục 1), và quy định `not_sent` là suy ra từ `Planned` trừ đi những gì đã có `CallStarted` kết thúc. Thêm ca test: lỗi ở lượt 2 trong kế hoạch 4 lượt ⇒ hàng của lượt 3 và 4 hiện `not_sent`.

### N3 (cao) "Ảnh chụp của lượt" không có nơi lưu và hạn dùng

Vị trí: mục 10, 🔵 dưới AD-47, test (s).

Lệnh đưa sang "kiểm `source_text` vẫn bằng bản trong ảnh chụp của lượt". `Received` chỉ mang `segment_id`, `text`, `has_han`; `AssembledPromptRecord` (mục 13) giữ prompt, không giữ nguồn từng câu. Hai cách đọc đúng chữ: Rust giữ ảnh chụp theo thao tác trong bộ nhớ (hết hạn khi thao tác bị dọn, và lệnh phải nhận mã thao tác); hoặc webview gửi lại `source_text` nó thấy, nhưng webview không có ảnh chụp, chỉ có bản hiện hành, nên phép kiểm không bao giờ đỏ. Test (s) xanh với cả hai nếu sửa nguồn trước khi đưa sang, và chỉ phân biệt được khi nguồn đổi sau khi webview đã nạp lại. Chưa nói cũng: `target_text` người dùng gõ sau khi lượt bắt đầu (chế độ cả Chương chọn câu rỗng) bị đưa sang ghi đè hay không.

Sửa: `Received` mang `source_hash` do Rust tính lúc lắp lượt (hoặc Rust giữ bản đồ `(operation_id, segment_id)` với vòng đời nêu rõ); lệnh nhận hash và so với `source_text` hiện hành. Nêu luôn điều kiện với `target_text` hiện hành (đã khác rỗng và không phải bản trong ảnh chụp thì từ chối hoặc hỏi, theo AD-47 "chỉ ghi cột người dùng đã đồng ý").

### N4 (trung) Cờ Hán định nghĩa lại "ký tự Hán", trái 🔴 của AD-44

Vị trí: mục 11 so với AD-44 ("Định nghĩa *ký tự Hán* là MỘT … `char_idx.rs::is_han`") và `core/dict/mod.rs::is_han` (7 dải gồm Ext B–G, tương thích). AD-60 chỉ dùng U+4E00–9FFF và U+3400–4DBF. Một bản thi hành gọi `dict::is_han`, bản kia chép hai dải; câu có chữ Ext B (hay gặp ở tên riêng) cho cờ khác nhau. `core/tm/tmx.rs::has_han` là bản thứ ba đã có.
Sửa: mục 11 chỉ ra `core/dict::is_han` làm nguồn duy nhất (cho cả bản dịch lẫn nguồn), hoặc ghi một ngoại lệ có tên trong AD-44 kèm lý do; xoá hai dải khỏi AD-60.

### N5 (trung) Escape U+200B: bao phủ chưa trọn, và vết của nó lọt vào bản dịch

Vị trí: mục 3 (khối TM con), mục 4.
- "Một dòng của nguồn hoặc ngữ cảnh mà trùng một dấu": không nói cặp TM (chữ của người khác) có được escape không, và "dấu" gồm cả dạng lỏng `SEGMENT n` mà bộ đọc nhận hay chỉ dạng `<<<SEGMENT n`. Nếu chỉ dạng chặt, một dòng nguồn `SEGMENT 7` đi nguyên, mô hình chép lại và bộ đọc mở khối 7.
- Mô hình chép dòng đã chèn U+200B thì `Received.text` mang U+200B vào `target_text`. Không nói có bỏ không (bỏ mù thì xoá cả ZWSP hợp lệ của người dùng).
- Khối TM con không có dấu đặt tên trong "hằng định nghĩa dấu", nên bộ đọc không biết bỏ chữ mô hình chép lại khối TM.
Sửa: liệt kê trong mục 4 tập dấu được escape (cả dạng lỏng, cả cặp TM); quy định bộ đọc chỉ bỏ U+200B ở những dòng mà sau khi bỏ nó trùng một dấu; đặt tên dấu cho khối TM con.

### N6 (trung) Thử lại ở chế độ vùng chọn lập lại kế hoạch theo chế độ nào

Vị trí: mục 9 ("lập kế hoạch lại theo mục 1"), mục 1 (định nghĩa "cần dịch" khác nhau theo chế độ), mục 2 (ngữ cảnh chỉ ở chế độ cả Chương).
Thử lại gửi các câu `missing` và `not_sent`. Nếu thao tác gốc là vùng chọn dịch lại câu đã có bản dịch, các câu đó có `target_text` không rỗng; lập kế hoạch lại theo định nghĩa cả Chương sẽ coi chúng là "đã dịch" và đưa vào khối ngữ cảnh thay vì dịch. Bên khác lập kế hoạch theo chế độ vùng chọn (đúng).
Sửa: thử lại là một thao tác chế độ vùng chọn trên tập `missing` ∪ `not_sent`, không bao giờ chế độ cả Chương; ghi vào mục 9 và thêm vào test (r) một câu có sẵn bản dịch.

### N7 (thấp) Ngữ cảnh đầu lượt không đối xứng

Mục 2: câu đã dịch chỉ đi kèm khi "nằm giữa hai câu cần dịch của cùng lượt"; lượt cắt vì đoạn dài hơn `CONTEXT_GAP_MAX` mang 3 câu đã dịch đứng trước. Nhưng lượt đầu của Chương không mang ngữ cảnh phía trước, và lượt cắt vì chạm trần 100 (không có câu đã dịch ở giữa) không nói có mang không. Hai bên chọn khác nhau sinh prompt khác nhau cho cùng đầu vào, ảnh hưởng chất lượng chứ không ảnh hưởng đúng sai.
Sửa: một câu: ngữ cảnh đầu lượt chỉ có khi lượt bị cắt vì đoạn đã dịch dài hơn `CONTEXT_GAP_MAX`; cắt vì trần 100 không mang gì (hoặc ngược lại).

### N8 (thấp) Mã lý do cho khối chưa đóng ở cuối luồng

`unclosed` chỉ định nghĩa cho "gặp mở n+1 khi n chưa đóng" (mục 4); khối cuối chưa đóng khi luồng hết có `stop` thì mục 7 cho `missing`, không có `stop` thì test (a) nói `no_finish`. Một bên dùng `unclosed` cho ca này. Sửa: nêu thứ tự ưu tiên mã (khối chưa đóng ở cuối luồng: `unclosed` hay mã của lượt) trong mục 5.

### N9 (thấp) Câu đã gửi nhưng lượt lỗi hay bị huỷ gọi là `not_sent`

Mục 8: câu còn lại của lượt đang chạy khi lỗi/huỷ thành `not_sent`, dù đã gửi và đã tốn tiền. Nhãn sai nghĩa; thử lại vẫn đúng. Sửa: đặt `missing(interrupted)` cho câu của lượt đã gửi, giữ `not_sent` cho lượt chưa gửi.

### N10 (thấp) Sự kiện kết thúc cấp thao tác và đường lỗi

Mục 5 liệt kê `CallDone` nhưng không có sự kiện kết thúc cả thao tác; lỗi mục 8 đi qua kết quả lệnh hay qua Channel không nói. Hai bên chọn khác nhau, webview kết thúc spinner không đồng nhất. Sửa: một câu: thao tác kết thúc khi lệnh trả (Ok khi xong hoặc huỷ, Err mang `params` của AD-59 khi lỗi).

## Mâu thuẫn với AD kế thừa (tóm tắt)

| AD | Kết quả |
|---|---|
| AD-1 | Không mâu thuẫn trên giấy (Rust chia lượt). N2 là chỗ dễ khiến logic miền lọt sang Vue. |
| AD-14 | Khớp: hàm thuần, 🔵 đúng. |
| AD-18 | Khớp: phân giải ghi đè trước, bỏ trùng sau. |
| AD-21 | Khớp: `params` là dữ liệu; ngoại lệ `provider_message` có tên ở dòng 311. |
| AD-22 | Khớp: một Channel, không tự thử lại. N9, N10 chỉ là nhãn. |
| AD-36 | Khớp gần đủ: dedupe nằm ở `ai/` còn truy vấn đủ điều kiện ở `glossary/`; nên ghi rằng dedupe chỉ trên kết quả truy vấn đó (thấp). |
| AD-44 | Mâu thuẫn chữ với 🔴 (N4). |
| AD-47 | Khớp, nhưng 🔵 thiếu nơi lưu ảnh chụp (N3). |
| AD-53 | Khớp: cùng kiểu phép kiểm "văn bản còn như lúc quét". |
| AD-57 | Khớp: một lần mỗi thao tác, sổ chèn có số cặp ẩn. |
| AD-59 | Khớp: 🔵 `call_index`, `first_segment_id`, `last_segment_id`. |

## Thứ tự đóng đề xuất

1. N1 và N2 (đổi hình dạng dây, phải chốt trước khi hai story Rust và Webview song song).
2. N3 (đổi lệnh đưa sang và `Received`), cùng đợt với N1 vì đều chạm `Received`.
3. N4, N5, N6 trong một lượt sửa chữ; N7 đến N10 là một câu mỗi chỗ.
4. Mở mục nợ có chủ cho phép đo lại với `gemini-3.8-flash` và nhà cung cấp cục bộ (R-F11), và thêm phép gỡ cho các ca test còn thiếu (R-F10).
