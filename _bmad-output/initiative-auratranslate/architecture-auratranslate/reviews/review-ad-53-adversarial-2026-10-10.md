# Review đối kháng AD-53 — 2026-10-10

Phạm vi đọc: AD-53 (mục 1–7, Phương án bị loại), AD-47 ③④, AD-49 mục 2–3, AD-50 mục 2–4, `core/ai/proofread.rs::locate_findings`, `core/segment/translation_origin.rs::arbitrate`, `commands/segment/targets.rs::write_non_user_target`, tickets 9.2/9.3/9.4/9.6/9.8. Không sửa spine.

Verdict: CHƯA ĐỦ KÝ. Mỗi cặp dưới đây tuân đúng chữ AD-53 mà vẫn dựng lệch nhau. Có 2 lỗ hở im lặng (H1, H2), 1 lỗ hở nhãn chiều đắt (H3).

## H1 (cao nhất) — Chữ ký S3: ngữ cảnh rỗng ở biên là ký tự đại diện, và cụm lặp bị định vị nhầm lần xuất hiện

Hai nửa của cùng một lỗ.

(a) Luật khớp: "ngữ cảnh trước đã lưu là đuôi của ngữ cảnh trước hiện tại, sau đã lưu là đầu của sau hiện tại". Chuỗi rỗng là đuôi và đầu của mọi chuỗi. Ghi nhớ tạo ở cụm đứng đầu hoặc cuối segment (ngữ cảnh bị cắt ở biên, nhiều khi rỗng; thoại ngắn "Được." là ca thường) khớp cụm đó ở MỌI nơi, bất kể ngữ cảnh. Đó đúng là phương án đã loại "ghi nhớ theo cụm cho mọi loại". Chiều ngược lại bất đối xứng: ghi nhớ tạo giữa câu dài (đủ N) không khớp cụm đó đứng đầu câu ngắn, nên báo lại. Hai story (9.2 khai nhánh, 9.6 cài khớp) mỗi bên đúng chữ mà ra hành vi khác nhau tuỳ vị trí cụm trong segment. Gộp segment "sống sót" chính là ca này, nên không thể chỉ cấm ngữ cảnh rỗng.

(b) `locate_findings` đặt mỗi cụm vào lần xuất hiện đầu tiên chưa bị finding trước chiếm, theo thứ tự trả lời của model, không theo ý model. Segment có cụm lặp với hai ngữ cảnh khác nhau: model báo lần 2 ⇒ Rust gắn vào lần 1. Hậu quả: bộ lọc S3 đánh giá ngữ cảnh của lần 1 (lỗi thật ở lần 2 bị lọc, đếm vào "đã lọc", không ai thấy); hoặc ngược lại báo sai chỗ và lệnh chấp nhận vượt bốn phép kiểm (`text[start..end)` vẫn bằng cụm) rồi thay NHẦM lần xuất hiện. Phép kiểm cụm của mục 4 không bắt được sai-lần-xuất-hiện, mà mục 5 lại viện nó làm lưới an toàn.

(c) Hai chi tiết nhỏ cùng nhóm: cắt cửa sổ N theo UTF-16 có thể cắt đôi cặp surrogate (không lưu được vào TEXT UTF-8, mục 4 chỉ canh start/end, không canh cửa sổ ngữ cảnh); và `trim` cụm trước khi cắt ngữ cảnh hay sau thì ngữ cảnh khác nhau một khoảng trắng ⇒ báo lại.

Đóng: thêm vào mục 6 — (i) ngữ cảnh lưu kèm cờ "chạm biên segment" mỗi phía; một phía chạm biên khớp khi phía hiện tại cũng chạm biên HOẶC khi chuỗi lưu rỗng chỉ khớp phía chạm biên; phía không chạm biên thì khớp đuôi/đầu như cũ; cửa sổ cắt theo ranh giới scalar Unicode ≤ N đơn vị UTF-16; ngữ cảnh lấy từ khoảng đã `trim`. (ii) Model phải trả cụm kèm neo (vài ký tự trước), Rust chỉ nhận nếu neo xác định đúng một lần xuất hiện; cụm lặp mà không xác định được ⇒ đếm vào `ambiguous` (cùng hàng với `unlocated`), không chấp nhận được, không lọc.

## H2 — Chấp nhận treo với webview: ô đang soạn, lượt gõ chen ngang, hoàn lại không theo LIFO

Mục 1 cho Webview flush trước, rồi gọi lệnh; mục 5 để webview dời vị trí phát hiện và nhận "văn bản mới Rust trả". Spec không nói ai ghi văn bản mới vào editing host, và không khoá ô trong lúc lệnh chạy. Hai khả năng cùng sai im lặng: (1) webview thay DOM bằng văn bản Rust ⇒ phím vừa gõ trong lúc chờ rơi mất; (2) webview không thay ⇒ lượt flush kế (AD-35) ghi `target_text` cũ + phím mới, đè lên lượt chấp nhận, trong khi dải vẫn nói "đã chấp nhận" và nút hoàn lại còn sống. `save_segment_targets`/flush không mang văn bản mong đợi nên Rust không bắt được. Phương án đã loại "bộ đệm gõ" bị bác vì "lần đầu webview tự ghi vào ô đang soạn", mà mục 5 tự đưa đúng hành vi đó vào (tiền lệ `replaceEditorSegment` của FR101 chưa được nhắc).

Cùng nhóm: hoàn lại mang "văn bản mong đợi" = văn bản ngay sau lượt chấp nhận đó. Chấp nhận A rồi B trên một segment, hoàn lại A trước ⇒ `proofread.text_changed` luôn. Mục 5 dời "phát hiện còn lại", không dời mục hoàn lại. Hứa "chấp nhận rồi hoàn lại trả đúng byte cũ" chỉ đúng với hoàn lại LIFO.

Đóng: thêm vào mục 1 "Trong lúc lệnh chấp nhận/hoàn lại đang bay, ô của segment đó chỉ-đọc; webview áp văn bản Rust trả bằng đúng đường FR101 (`replaceEditorSegment`), một chỗ; thứ tự hoàn lại là LIFO theo segment, lượt không phải đỉnh ngăn xếp bị vô hiệu (không gửi)". Và mục 5: bản ghi hoàn lại dời theo cùng phép dời của phát hiện, hoặc bị bỏ.

## H3 — Nhãn chiều đắt: chấp nhận loại 9.2 gắn "tôi dịch" lên chữ AI dịch lại từ nguồn

Mục 2 chỉ ghi ⚠️ cho ngữ pháp "viết lại cả mệnh đề". Loại 9.2 (nghi về nghĩa) có đề xuất do model sinh có tham chiếu bản gốc, nghĩa là chính là một bản dịch lại của cụm. Trên câu người dùng gõ, `arbitrate` trả *tôi dịch*; ký ngay ⇒ cặp TM mang nhãn *của tôi* cho chữ máy dịch, chiều đắt của AD-47 ④ (đầu độc TM, `RagInjector` ưu tiên theo AD-18). AD này không giới hạn kích thước cụm bị thay, không phân biệt loại, và mục 6 lại bắt 9.2 "khai nhánh" chỉ cho chữ ký, không khai xuất xứ. 9.2 (after = [1]) có thể xong và chấp nhận được trước khi ai đọc lại mục 2.

Thêm: "chấp nhận rồi hoàn lại trả `target_text` đúng byte và phép phân xử lúc ký cho cùng kết quả" chỉ đúng nếu ký NGAY. Hoàn lại đặt mốc = văn bản người dùng gõ (T), còn mốc gốc B (chữ AI) mất. Trước chấp nhận: T ≠ B, người dùng gõ ngược về đúng B ⇒ hàm phân xử ra *người khác dịch* (đúng). Sau chấp nhận rồi hoàn lại: mốc = T ⇒ gõ ngược về B ra *tôi dịch* (nhãn người dùng trên chữ AI). Mục 2 có ghi mất mốc cũ cho lượt chấp nhận, nhưng mục 3 hứa "khôi phục" mà không nhắc ca này; lượt hoàn lại nên cũng phải liệt kê vào ⚠️.

Đóng: mục 2 thêm cột "loại": loại 9.2 ⇒ xuất xứ = nhỏ nhất (*người khác dịch*) trừ khi Ice duyệt ngưỡng độ dài; hoặc giữ nguyên và ghi ⚠️ cho 9.2. Đây là hai lựa chọn hợp lệ ⇒ theo AGENTS.md phải đưa Ice kèm số đo, không tự chọn. Mục 3 sửa câu hứa thành "…khi ký ngay; mốc gốc trước chấp nhận không được khôi phục".

## H4 — Phạm vi quét và vô hiệu hoá phát hiện: 9.3 vs 9.4 vs mọi lượt ghi không-phải-người-dùng

- "Vùng chọn" (9.3): dây 9.1 gắn offset vào `scanned_text`, mục 4 bắt `target_text == scanned_text` bằng byte. Nếu quét vùng chọn gửi lát cắt, mọi chấp nhận đều `text_changed`; nếu offset được quy về segment, `scanned_text` phải là cả segment. AD không nói. Vùng chọn vắt nhiều segment càng không định nghĩa.
- Quét cả Chương gồm nhiều lô, kết quả về muộn sau khi người dùng đã đổi Chương: mục 4 chỉ nói "reset khi đổi Chương", không nói lô về sau bị vứt (token thế hệ). Phát hiện của Chương cũ có thể hiện lại dưới Chương mới, số "N phát hiện" sai.
- Webview chỉ bỏ phát hiện khi "người dùng gõ". Các lượt ghi không-phải-người-dùng khác (FR94, promote AI, FR59/FR58, FR101, gộp/tách AD-5) làm phát hiện cũ đi mà gạch chân vẫn vẽ. Rust chặn ghi sai (`text_changed`/`segment_retired`), nhưng người dùng thấy gạch chân sống, bấm, gặp lỗi; và phát hiện của segment về hưu vẫn tính vào bộ đếm/"phát hiện kế". Chương quét có 500 phát hiện, gộp 1 segment: bộ đếm nói dối.
- Mẫu số cho 9.8: "số phát hiện đã lọc" theo lượt quét; lượt quét Chương bị huỷ giữa chừng hay lô lỗi cần đếm "chưa quét" tách khỏi "0 phát hiện".

Đóng: mục 4 thêm "Mọi lượt ghi `target_text` hay về hưu segment (AD-47 ③, AD-5) phát tín hiệu vô hiệu hoá phát hiện của segment đó; `scanned_text` luôn là cả segment (vùng chọn chỉ giới hạn danh sách segment và cụm báo); mỗi lượt quét mang `generation` và lô không khớp bị vứt; kết quả quét mang `scanned`, `filtered`, `unlocated`, `ambiguous`, `not_scanned`".

## H5 — Hai chủ của "bỏ qua" và khoá loại do model chọn

- Mục 4 cho lệnh bỏ qua mang bốn phép kiểm (9.4 AC: "bỏ qua từng cái"); mục 6 đặt lệnh bỏ qua và bảng `PROOF_IGNORE` ở 9.6 (after = [3,4,10]). Giữa hai story, "bỏ qua" của 9.4 hoặc là lệnh Rust không lưu gì (rồi 9.6 đổi hợp đồng) hoặc là trạng thái webview cùng khoá riêng ⇒ hai định nghĩa "cùng một phát hiện", đúng lớp AD-53 Prevents 3. Bước di trú và vòng đời `PROOF_IGNORE` (xoá Tác phẩm, danh sách xem lại) không ai sở hữu trong AD.
- `loại` trong khoá là chuỗi model trả. Cùng cụm báo "chính tả" lần này, "ngữ pháp" lần sau ⇒ khoá khác ⇒ báo lại, FR84 hỏng. 9.2 có ba nhãn con (dịch sai / thoát nghĩa / tối nghĩa) mà AD chỉ có "mọi loại đối chiếu". Khoá của loại 9.2 chỉ chứa văn bản đích: cùng cụm + cùng ngữ cảnh đích nhưng nguồn khác (nhân vật 他/她) vẫn bị lọc, giấu một dịch sai thật.
- N "do 9.6 chốt" nhưng 9.2/9.3 chạy trước và phải cắt ngữ cảnh khi lọc/gửi, hoặc chưa lọc gì. Nếu 9.2/9.3 gửi ngữ cảnh trên dây theo N tạm, 9.6 đổi N sẽ lệch dữ liệu đã lưu (khoá "không đổi hình dạng" nhưng đổi nội dung).

Đóng: mục 6 thêm "9.4 sở hữu lệnh bỏ qua và bốn phép kiểm, chỉ ghi vào webview; 9.6 sở hữu `PROOF_IGNORE`, bước di trú và việc chuyển lệnh sang lưu; `loại` trong khoá là nhánh (chính tả | ngữ cảnh), enum đóng do Rust gán, không chuỗi model; khoá loại 9.2 gồm cụm nguồn đối ứng nếu model trả được, nếu không loại 9.2 chỉ ghi nhớ trong phạm vi segment (ngoại lệ có tên, sống đến khi segment về hưu); N là hằng số trong `core/ai/proofread.rs` do 9.2 đặt tạm và 9.6 chốt trước khi bất kỳ hàng nào được lưu".

## Cặp đã thử, không thành lỗ hở
- Chấp nhận vs xác nhận (AD-31) tranh nhau: cùng hàng đợi `store::Writer`, chấp nhận kiểm văn bản trong giao dịch ⇒ chạy trước hay sau đều nhất quán (kết quả: xác nhận rồi chấp nhận ⇒ draft, cặp TM cũ còn như mọi lần sửa câu đã ký).
- Chấp nhận liên tiếp trên câu AI chưa sửa: mỗi lượt `arbitrate` trên mốc vừa đặt vẫn ra *người khác dịch*; không trôi.
- Dời offset khi chấp nhận nhiều lần ở 9.3: offset theo segment, dời cục bộ trong từng segment; không có tương tác chéo segment ngoài H4.
