# Review AD-60 theo rubric (Reviewer Gate), 2026-10-27

Phạm vi: AD-60 (dòng 1201–1280 của spine), dòng 🔵 dưới AD-14 (dòng 212), hàng "Gọi AI" của Consistency Conventions (dòng 1306). Đối chiếu: `ad-brief-dich-ca-chuong-mot-luot.md`, `do-m2.md`, PRD FR69–FR76 và FR145, AD-14, AD-18, AD-21, AD-22, AD-36, AD-47, AD-57, AD-59.

## Phán quyết

Cần sửa trước khi ký: AD-60 phủ đủ 11 quyết định Ice đã ký, không làm yếu AD nào được thừa kế, và hai luật cốt lõi (đối chiếu chỉ số ở mọi lượt, kiểm `finish_reason`) có số đo đỡ. Nhưng ba điểm khiến hai bên thi hành rẽ nhánh còn bỏ ngỏ: hình dạng sự kiện trên dây và trạng thái từng câu, ranh giới khối (khối ngữ cảnh mất dấu, văn bản nguồn chứa dấu), và từ *chưa dịch* trùng nghĩa với "chưa dịch" của Editor.

## Phát hiện

### F1 (high) Hình dạng dây và trạng thái từng câu không được định nghĩa
Vị trí: Binds (`AiTranslateBatchEventWire`), Rule 4, 5, 6.
Spine chỉ nói "một sự kiện mang danh sách `segment_id` và usage" và "cờ cảnh báo gửi qua dây". Không có tên biến thể, không có tập trạng thái của một câu (chờ, đang nhận, đã nhận, chưa dịch, lỗi, đã huỷ), không nói cờ Hán nằm ở biến thể nào. Brief còn nêu ví dụ `CallDone{segment_ids, usage}`; AD-60 bỏ mất cả ví dụ. Hai hệ quả:
- Rule 7 dùng "chưa dịch hoặc lỗi". Sau Rule 6, "lỗi" không còn định nghĩa: các câu của lượt hỏng và các lượt sau chưa từng được gửi nằm ở trạng thái nào?
- Rule 4 phát `Token` ngay khi có chữ trong khối n. Khi khối đó về sau thành *chưa dịch* (trùng chỉ số, bị cắt giữa khối, thiếu `stop`), chữ nửa vời đã lên panel. Không có sự kiện thu hồi, và không nói chữ đó bị bỏ hay giữ. Đây là đúng lớp "bản dịch hiện ở chỗ không tin được" mà Prevents 3 muốn chặn.
Sửa: thêm một mục nêu (i) tập trạng thái câu như dữ liệu trên dây (AD-21), (ii) tên và trường của ba loại sự kiện: chữ của câu, câu đã nhận / chưa dịch (kèm lý do dạng mã), kết thúc lượt (danh sách `segment_id`, usage, `finish_reason`, cờ Hán theo từng câu), (iii) luật "câu chuyển sang chưa dịch thì bỏ chữ đã stream, panel xoá phần đó", kèm một ca test.

### F2 (high) Khối ngữ cảnh và dấu khối: ranh giới mơ hồ, va chạm với văn bản nguồn
Vị trí: Rule 2, Rule 3, Rule 4 (dòng mở thiếu `<<<`).
- Brief ghi dạng `<<<CONTEXT … CONTEXT>>>`; AD-60 bỏ dạng đó và chỉ nói "khối ngữ cảnh chỉ đọc". Hai người thi hành sẽ chọn hai dấu khác nhau. Bộ đọc cũng phải bỏ qua đoạn mô hình lặp lại khối ngữ cảnh.
- Không nói gì khi văn bản nguồn hoặc bản dịch có sẵn chuỗi `<<<`, `SEGMENT n` hoặc `>>>` (truyện kể về lập trình, ví dụ trích dẫn). Quy tắc chấp nhận "thiếu `<<<` nhưng có `SEGMENT n`" làm nguy cơ này tăng, vì bộ đọc nhận dòng mở lỏng hơn.
- Không nói bộ đọc nhận dấu mở/đóng chỉ khi nằm đầu dòng hay ở bất kỳ đâu, và khi khối mở dở rồi gặp dấu mở của khối n+1 thì khối trước đóng hay hỏng.
Sửa: chốt ba dấu trong Rule 3 (mở SEGMENT, đóng SEGMENT, CONTEXT), neo vào đầu dòng, nêu cách vô hiệu hoá chuỗi trùng trong nguồn (thay một ký tự của dấu khi lắp) và ghi: gặp mở n+1 khi n chưa đóng thì n thành *chưa dịch*. Thêm ca test cho nguồn chứa dấu.

### F3 (high) *Chưa dịch* trùng từ với "chưa dịch" của Editor, và "cần dịch" không được định nghĩa
Vị trí: tiêu đề, Rule 1, 2, 5, 7.
Rule 2 dùng "đã dịch" cho câu có `target_text`. Rule 5 dùng *chưa dịch* cho câu mất khỏi kết quả AI. Hai khái niệm khác tầng (Editor lưu, panel AI phiên) mang cùng chữ; story thi hành sẽ nối chúng, mà AD-47 và FR72 cấm đường đó. Thêm nữa:
- "Câu cần dịch" không được định nghĩa: trong "Dịch các câu đã chọn", câu đã chọn mà đã có bản dịch thì dịch lại hay thành khối ngữ cảnh?
- Trạng thái *chưa dịch* có sống qua đóng panel hoặc tải lại không? Rule 7 nói "thử lại chỉ gửi câu chưa dịch" nhưng không nói nguồn sự thật (trạng thái phiên ở `aiTranslateBatchState.ts`, hay suy lại từ Editor).
Sửa: đổi tên (ví dụ *thiếu kết quả AI*, `Missing`) và khai nó chỉ ở phiên panel, không bao giờ ghi vào `target_text`; định nghĩa "cần dịch" (vùng chọn: theo cờ người dùng; cả Chương: `target_text` rỗng và không `is_omitted`).

### F4 (high) Chủ sở hữu việc chia lượt và nơi đặt hằng số 100 không nêu
Vị trí: Binds (`src/config/aitranslate.ts` cùng `commands/aitranslate.rs`), Rule 1, test (h).
Binds liệt kê cả frontend và Rust. Rule 1 không nói bên nào chia. Nếu `TRANSLATE_CALL_MAX_SEGMENTS` ở `src/config/aitranslate.ts` thì webview quyết định ranh giới tiền; nếu ở Rust thì file config frontend chỉ là bản sao, hai nơi trôi lệch. Cũng chưa có `G` hoặc `TM_PAIRS_PER_SEGMENT` thành hằng số có tên. Test (h) không nói chạy ở lớp nào.
Sửa: một câu "Rust chia lượt; hằng số nằm ở `core/ai/`; frontend chỉ nhận danh sách lượt trên dây", gom `TRANSLATE_CALL_MAX_SEGMENTS`, `CONTEXT_GAP_MAX = 3`, `TM_PAIRS_PER_SEGMENT = 3` vào đó và cho một gate hoặc test đỏ nếu frontend cài lại số.

### F5 (medium) Trục chia lượt: "đoạn rời của vùng chọn" so với "câu ngăn" chưa tách rạch ròi
Vị trí: Rule 1 và Rule 2.
Rule 1: vùng chọn nhiều đoạn rời thì mỗi đoạn đi lượt riêng. Rule 2: câu đã dịch hoặc `is_omitted` giữa hai câu cần dịch của cùng lượt là ngữ cảnh. Không nói câu nằm giữa hai đoạn mà người dùng không chọn thuộc loại nào, và `G` đếm gì (chỉ câu ngăn, hay cả câu ngoài vùng chọn). Cũng không nói danh sách lượt được tính một lần lúc bắt đầu hay lại từng lượt: nếu người dùng gộp, tách, hoặc sửa câu giữa các lượt (AGENTS: gộp/tách câu = retire + create) thì `segment_id` trong lượt sau có thể đã chết.
Sửa: nêu "tính kế hoạch lượt một lần ở đầu thao tác trên ảnh chụp danh sách câu; `segment_id` không còn tồn tại khi tới lượt thì bị bỏ qua và đánh *thiếu kết quả AI*"; nêu rõ nhóm theo tập chọn (không liền theo `ord` giữa các câu được chọn thì tách), còn câu ngăn chỉ tính trong cùng một đoạn chọn.

### F6 (medium) Đối chiếu chỉ số: khối rỗng, khối cuối thiếu dấu đóng, chỉ số ngoài khoảng
Vị trí: Rule 5.
- Khối đóng nhưng nội dung rỗng hoặc chỉ khoảng trắng vẫn tính "xuất hiện một lần". Mô hình hay gộp hai câu vào khối k và để khối k+1 rỗng; chữ sai chỗ vẫn lọt. `7bd1f90` từng coi "chỉ khoảng trắng" là rỗng, nên AD-60 phải giữ lại điều đó ở cấp khối. Sửa: khối rỗng ⇒ *thiếu kết quả AI*, thêm ca test.
- `finish_reason == stop` mà khối cuối không có dấu đóng (luồng dừng giữa chừng) chưa thuộc ca nào; Rule 5 chỉ nói trường hợp thiếu `finish_reason`. Do-m2 đã ghi câu cuối Chương bị rơi 2 lần, nên ca này có thật.
- "Chỉ số trùng, hoặc nằm ngoài 1..n thì câu mang chỉ số đó thành chưa dịch": chỉ số ngoài khoảng không thuộc câu nào, nên mệnh đề vô nghĩa. Trùng thì cả hai lần xuất hiện bỏ, hay lần đầu giữ? Test (d) nói "câu đó" nhưng không rõ. Sửa: chỉ số ngoài khoảng bị bỏ im; trùng thì không lần nào được giữ.
- Mục "lượt trọn" được định nghĩa nhưng không điều khoản nào dùng lại nó; Rule 5 và Rule 6 vận hành bằng từng ca riêng. Hoặc dùng nó (ví dụ: hiển thị trạng thái lượt), hoặc bỏ.

### F7 (medium) Luồng đóng không có `[DONE]`: kết quả đã đóng của lượt hiện tại được giữ hay không
Vị trí: Rule 6.
"Dừng cả thao tác… kết quả của các lượt trước được giữ" không nêu số phận các khối đã đóng của chính lượt đứt. Rule 5 giữ chúng khi thiếu `finish_reason`; Rule 6 ngầm im. Tương tự, khi người dùng huỷ giữa khung SSE: khối đã đóng giữ, khối dở bỏ, usage của lượt bị huỷ tính hay không (đã tốn tiền). Hiện chỉ có test (f) đếm lượt.
Sửa: một câu thống nhất "mọi khối đã đóng đều giữ, ở mọi đường kết thúc (stop, thiếu `stop`, đứt, huỷ)", và sự kiện kết thúc lượt luôn phát với usage nếu có.

### F8 (medium) AD-57 và AD-36 trong nhiều lượt
Vị trí: Rule 3.
AD-57 Rule 6 bắt chính sách TM đọc **một lần mỗi thao tác** (Prevents 7: công tắc đổi giữa lô). AD-60 chia một thao tác thành nhiều lượt nhưng không nói chính sách (và ảnh chụp Glossary) đọc ở đầu thao tác hay ở đầu từng lượt. Đọc theo lượt thì tái tạo chính lỗi AD-57 đã cấm. Nói rõ cả "quét theo từng câu" có áp cho câu ngữ cảnh không. Glossary gộp "bỏ trùng theo thuật ngữ nguồn" cần đi qua phân giải ghi đè của AD-18 trước khi bỏ trùng, nếu không hai tầng cho hai bản dịch cùng thuật ngữ. Số cặp TM bị ẩn bởi công tắc (AD-57 Rule 6) phải vào bản ghi lượt.
Sửa: thêm "ảnh chụp chính sách TM, Glossary đã phân giải được dựng một lần ở đầu thao tác và dùng cho mọi lượt; số cặp ẩn ghi vào bản ghi từng lượt".

### F9 (medium) Đường dịch một câu và system prompt mặc định
Vị trí: Rule 1 (dòng cuối), Rule 10.
- "Dịch một câu tại con trỏ không đổi" nhưng không nói đường đó giữ `assemble_prompt` cũ hay đi qua hàm lắp mới với n=1. Lô chỉ có một câu cần dịch (cả Chương còn 1 câu, thử lại 1 câu) đi đường nào? Hai đường cho hai định dạng đầu ra và hai bộ đọc. Khi người dùng có system prompt tuỳ chỉnh dặn "chỉ trả bản dịch", đường lô cần phần dặn khối, đường đơn thì không.
- Rule 10 ghi "Không đổi: system prompt mặc định", trái brief mục 9 (viết lại dòng "Giữ số dòng và dấu câu tương ứng với đoạn nguồn" cho dạng nhiều khối). Nếu giữ nguyên, dòng đó mâu thuẫn với cấu trúc khối. Phải chọn: viết lại, hay phần hằng số ghi đè được ý đó.
- Không nêu FR145 (gửi lại đúng prompt đã lắp từ màn Xem prompt): Rule 8 làm bản ghi thành bản ghi lượt, vậy gửi lại một lượt trả về panel Đề xuất AI thế nào (nhiều câu một lần?). Phải nêu hoặc loại FR145 khỏi lượt.
Sửa: ghi đường đơn nguyên vẹn và mọi lô (kể cả n=1) dùng đường khối; nêu quyết định về dòng mặc định; một câu cho FR145.

### F10 (medium) Test canh thiếu các ca bảo vệ luật quan trọng
Vị trí: Rule 9.
Chưa có ca cho: Rule 7 (thử lại chỉ gửi câu thiếu và gom lại theo Rule 1); Rule 6 (lỗi non-2xx ở lượt k dừng thao tác, giữ lượt trước, nêu đúng lượt); khối rỗng; bản ghi FR71 theo lượt và đánh dấu khối; phần dặn định dạng đứng sau system prompt không rỗng (ca k chỉ phủ system rỗng, nên một cài đặt đặt phần dặn *trước* vẫn xanh); G=3 cắt lượt; khử trùng Glossary/TM; chính sách TM chỉ đọc một lần. Cũng cần ghi các ca đỏ (c), (d), (e) mỗi ca đối chứng bằng phép gỡ nào, vì AGENTS chỉ chấp nhận gỡ thật.

### F11 (low) Chuẩn đo và phạm vi bằng chứng
Vị trí: Prevents 1–3, Phương án bị loại.
Mọi số đo là một mô hình (`gemini-3.5-flash-lite`), một endpoint, mỗi cỡ lượt chạy một lần; preset của app là `gemini-3.8-flash`, chưa đo. Rule 5 coi `finish_reason` khác `stop` là cắt: với Ollama / LM Studio hay endpoint tự dựng không phát `finish_reason`, quy tắc chỉ giữ các khối đã đóng nên vẫn chạy, nhưng chưa ai đo. Prevents 3 ("bản dịch của k+1 hiện dưới tên câu k") không có số đo trong `do-m2.md`; nên ghi là rủi ro suy ra, không phải đo. Nên thêm câu "dựa trên một mô hình, một lần chạy mỗi cỡ" và mở mục nợ có chủ cho phép đo lại.

### F12 (low) Cờ chữ Hán: tiêu chí chưa cụ thể
Vị trí: Rule 6 (dòng cuối).
Không nói "nguồn là tiếng Trung" lấy từ đâu (ngôn ngữ nguồn của Tác phẩm theo AD-18), "ký tự Hán" là dải Unicode nào (CJK Unified, mở rộng A–F, ký tự tương thích), có loại dấu câu CJK không, và cờ gắn theo câu hay theo lượt. Hai bên thi hành sẽ chọn hai dải khác nhau. Sửa: nêu dải, nguồn ngôn ngữ, và gắn cờ theo `segment_id`.

### F13 (low) Dòng 🔵 dưới AD-14 và hàng Gọi AI
Dòng 🔵 dưới AD-14 đúng và không làm yếu AD-14 (hàm vẫn thuần); nhưng Rule của AD-14 vẫn nói đầu vào là "(câu nguồn, scope, Glossary, TM)". Chấp nhận được theo mẫu 🔵 của spine. Hàng "Gọi AI" nhất quán với AD-60; nên thêm "không bao giờ tự thử lại" kèm "thử lại chỉ gửi câu thiếu" nếu F3 đổi tên.

## Điểm đạt

- Phủ đủ 11 quyết định đã ký: P (1), gom cả Chương và vùng chọn (2), tự chia (3), đoạn rời (4), trần 100 (5), Q1–Q5 (7–11); quyết định 6 (đo trước) là quy trình, được thể hiện bằng phần bằng chứng.
- Không làm yếu AD-22 / FR75 (không tự thử lại), AD-47 (khớp chỉ theo chỉ số, bản dịch sai chỗ không bao giờ hiện), AD-21 (cờ qua dây là dữ liệu), AD-59 (lỗi nhà cung cấp dừng thao tác), AD-36 và AD-57 (quét theo từng câu, đi qua cùng truy vấn).
- Số liệu trong Prevents khớp `do-m2.md` (292–525 câu mất; 1–2/356; 14/14 lượt `stop`, thiếu 0/1/0; $0,06 và 100–114 giây).
- Đối chứng đỏ ở (a) là phép gỡ thật.
