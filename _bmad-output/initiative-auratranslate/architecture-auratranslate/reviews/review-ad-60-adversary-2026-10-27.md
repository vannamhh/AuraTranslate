# Review AD-60 — thấu kính đối kháng (adversary)

Ngày: 2026-10-27. Đối tượng: AD-60 trong spine, đọc cùng AD-14, AD-21, AD-22, AD-36, AD-47, AD-53, AD-59 và bảng Consistency Conventions. Có đối chiếu mã tại `core/ai/rag.rs` (khuôn `<<<SEGMENT` của Proofreader, `assemble_prompt`, `InjectionLedger`), `core/ai/client.rs` (`translate`, `interpret_sse_event`, `is_empty_reply`), `commands/aitranslate.rs` (`batch_stopped_error`, `prepare_batch_call_with`).

Phương pháp: dựng từng cặp "hai đơn vị bên dưới, mỗi bên tuân AD-60 đúng chữ, vẫn lắp không khớp nhau". Đơn vị: (A) assembler + bộ đọc luồng Rust; (B) command/wire + bộ chạy lô Rust; (C) hàng lô, thử lại, màn xem prompt ở webview; (D) đường dịch một câu + Proofreader dùng chung client; (E) đường đưa đề xuất sang Editor (AD-47).

Kết luận: AD-60 chặn đúng lỗi đã đo (cắt ngang, rơi câu) nhưng còn 4 cặp mức Cao và 6 cặp mức Trung mà hai bên cùng tuân đúng chữ vẫn lệch. Hai chỗ AD-60 mâu thuẫn trực tiếp với AD kế thừa (F3, F6 phần "Mâu thuẫn"). Chưa nên đóng AD cho tới khi siết F1–F4.

Mức độ: Cao = hỏng im lặng trên dữ liệu người dùng hoặc tiền; Trung = hai bản lắp lệch nhưng có cổng/đường thấy được; Thấp = tối nghĩa, không hỏng dữ liệu.

---

## F1 (Cao) — Ngữ pháp đầu ra của mô hình và dấu phân khối không được định nghĩa; bộ đọc "dễ dãi" gặp văn bản nguồn chứa dấu

Cặp: A-assembler (lắp prompt) × A-bộ đọc luồng (Rule 4).

- Assembler theo Rule 3 đặt mỗi câu trong `<<<SEGMENT n … SEGMENT>>>`, chép nguyên văn nguồn (khuôn Proofreader trong `rag.rs` còn thêm `<<<SOURCE/<<<TEXT`). Rule 3 viết "…" cho phần trong khối, nên một bên thêm `<<<SOURCE … SOURCE>>>`, bên kia không.
- Bộ đọc theo Rule 4 tách khối trong luồng, nhận cả dòng mở thiếu `<<<` ("`SEGMENT n`"). Không rule nào nói: dấu chỉ được nhận ở đầu dòng hay ở giữa; khoảng trắng đầu/cuối khối có cắt không; mô hình chép lại `<<<SOURCE` hay `<<<TEXT` thì tính là chữ dịch hay dấu; chữ nằm NGOÀI khối (lời dẫn, khối ngữ cảnh mà mô hình lặp lại) bị bỏ hay tính vào khối gần nhất.
- Xung đột cụ thể: câu nguồn (nhập từ web theo FR122, hoặc một tài liệu nói về chính định dạng này) chứa dòng `SEGMENT 3` hoặc `SEGMENT>>>`. Bản dịch của câu đó được mô hình chép lại, bộ đọc "dễ dãi" coi là dấu đóng khối, câu k+1 thành câu k. Đây chính là Prevents 3 mà AD-60 nói chặn. Bản đọc nghiêm ngặt thì bỏ câu; bản dễ dãi thì dịch lệch chỗ, mà cả hai đều "tuân Rule 4".
- Chữ cắt khoảng trắng hay không đổi văn bản ghi vào Editor (AD-47: mốc so sánh là đúng chữ đã ghi); hai bên cắt khác nhau cho hai chuỗi khác nhau ở câu cùng nội dung.

Siết Rule 4 (đề xuất, thêm hai mục):
> 4a. **Ngữ pháp đầu ra là một đặc tả đóng, một nơi.** Bộ đọc chỉ nhận dấu mở `<<<SEGMENT n` (hoặc `SEGMENT n`) và dấu đóng `SEGMENT>>>` khi chúng là TOÀN BỘ nội dung một dòng (trước/sau chỉ có khoảng trắng). Chữ trong khối là mọi thứ giữa dòng mở và dòng đóng, cắt đúng một dòng trống đầu và cuối, không chuẩn hoá thêm. Chữ ngoài mọi khối bị bỏ và không phát `Token`. Phần dặn định dạng khối trong `core/ai/` và bộ đọc dùng CHUNG một hằng định nghĩa dấu; không cho khối con (`<<<SOURCE`, `<<<TEXT`) trong đầu ra.
> 4b. **Nguồn chứa dấu.** Assembler phát hiện câu cần dịch mà có một dòng trùng dấu mở/đóng của 4a thì chèn một ký tự không in được vào dòng đó trong prompt (hoặc đổi sang bộ dấu có nonce theo lượt), và có test: nguồn chứa `SEGMENT>>>` không làm khối nào đóng sớm.

---

## F2 (Cao) — Token phát trước khi biết câu đó có hợp lệ; "chưa dịch" không rút lại chữ đã hiện; đường đưa sang Editor lấy chữ từ đâu

Cặp: A-bộ đọc (phát `Token` ngay trong khối) × C-webview (cộng dồn Token vào hàng) × E-đường đưa sang Editor (AD-47).

- Rule 4 phát `Token` mang `segment_id` NGAY khi có chữ trong khối n. Rule 5 chỉ biết khối n trùng chỉ số, hoặc lượt thiếu `finish_reason`, SAU KHI chữ đã phát.
- Webview (đúng chữ Rule 4) cộng Token vào ô đề xuất của câu k. Khi Rule 5 nói "câu đó thành chưa dịch", Rule 5 không có sự kiện nào bảo webview xoá chữ đã cộng. Ca trùng chỉ số: khối thứ hai của k cộng NỐI vào chữ khối thứ nhất, rồi bị gắn "chưa dịch" mà ô vẫn mang hai bản dịch dính nhau.
- Ca cắt ngang: câu k mở khối, đã phát nửa câu, luồng đứt thiếu `finish_reason` ⇒ k "chưa dịch" nhưng nửa câu vẫn trong ô. Nếu đường đưa sang Editor (AD-47, xuất xứ `other`) đọc "ô đề xuất" thay vì "trạng thái đã nhận", nửa câu bị ghi vào `target_text` làm mốc so sánh. Không cổng nào đỏ — chính lớp hỏng AD-47 mô tả.
- Hai bên đều đúng chữ: bên A không cần sự kiện rút lại (Rule 4 không đòi), bên C không có nguồn nào khác để biết.

Siết (thêm Rule 4c, 5c):
> 4c. Dây có sự kiện `Received { segment_id, text }` (khi gặp dấu đóng, chữ cuối cùng của câu) và `Discarded { segment_id, reason }` (reason là dữ liệu: `duplicate_index` · `out_of_range` · `unclosed` · `no_finish`). `Token` chỉ để hiển thị tiến trình; webview xoá ô khi gặp `Discarded` và đặt ô bằng đúng `Received.text`.
> 5c. Đường đưa đề xuất sang Editor và mọi phép đếm "đã dịch" chỉ đọc chữ từ `Received`, không bao giờ từ chữ cộng dồn của `Token`. Có test (l): khối k phát nửa chữ rồi luồng đứt ⇒ ô k rỗng và nút đưa sang Editor tắt; (m): hai khối cùng chỉ số ⇒ không có chữ nào của k để đưa.

---

## F3 (Cao) — Ai chia lượt và ai vòng lặp; "nêu tên lượt" mâu thuẫn AD-59

Cặp: B-command (`run_batch_call`) × C-webview (`aiTranslateBatchState.ts`); AD-60 × AD-59.

- Rule 1 nói "thao tác dịch lô chia thành nhiều lượt, chạy tuần tự, huỷ kiểm trước mỗi lượt" nhưng không nói ai chia. Mã hiện có: `prepare_batch_call_with(segment_ids)` là một lượt gọi, nên webview đang điều phối. Hai kiến trúc đều đúng chữ: (i) webview chia theo `ord` rồi gọi lệnh N lần, mỗi lần một Channel; (ii) một lệnh Rust nhận vùng chọn, tự chia và lặp.
  - (i) cần quy tắc Câu ngăn `G = 3` và khối ngữ cảnh thực hiện ở webview (webview có `ord`, `target_text`, `is_omitted`?), tức logic miền lọt sang Vue, trái với dòng AD "frontend không phải chủ dữ liệu".
  - (ii) cần một token huỷ và một mô hình sự kiện có chỉ số lượt; (i) không cần.
  - Test (f) "đúng k lượt đã gửi" và (h) "250 câu ⇒ 100, 100, 50" thành test của hai tầng khác nhau tuỳ cách.
- Rule 6 viết "nêu tên lượt đó" nhưng `batch_stopped_error(segment_id, err)` (AD-59, Binds và mục 7 "không đổi") mang `segment_id` đơn lẻ làm tham số `params`. Một lượt có tới 100 câu; AD-21 chỉ cho `params` mang DỮ LIỆU. Không rule nào nói tham số tên lượt là gì (id đầu? khoảng `ord`? chỉ số lượt?). Thế là bên B chọn `segment_id` đầu lượt, bên C hiển thị "câu đầu" như câu gây lỗi, người dùng đọc nhầm thành câu bị từ chối (đúng lỗi "đổ lỗi sai chỗ" AD-59 Prevents 1).

Siết Rule 1 và Rule 6:
> 1a. **Rust chia lượt và là chủ vòng lặp.** Một command nhận vùng chọn (hoặc Chương) và một Channel, tự chia theo Rule 1–2 (hàm thuần trong `core/`), tự giữ một cờ huỷ cho cả thao tác. Webview không biết ranh giới lượt trước khi nhận sự kiện.
> 1b. Mọi sự kiện lô mang `call_index` (từ 1) và `call_segment_ids` ở sự kiện bắt đầu lượt.
> 6a. Lỗi dừng thao tác mang `params`: `call_index`, `first_segment_id`, `last_segment_id` (dữ liệu). AD-59 mục "Binds" ghi 🔵 ở `batch_stopped_error` rằng tham số `segment_id` nay là `first_segment_id` của lượt.

---

## F4 (Cao) — Marker của prompt người dùng (`{{source_segment}}`, `{{glossary_terms}}`, `{{tm_similar_segments}}`) và sổ chèn FR70 không được nói trong lượt

Cặp: A-assembler hàm lượt × D-đường một câu (`assemble_prompt`) × C-webview (hiển thị trạng thái chèn).

- `assemble_prompt` hôm nay mở rộng marker trong thân prompt do người dùng viết, và ghi `InjectionLedger` phân biệt "không hỏi" (thân không mang marker) với "hỏi, không thấy". Đó là phần FR70, AD-14, AD-36 canh.
- Rule 3 nói: "system prompt đã phân giải" + phần dặn khối nối sau, Glossary của các câu "gộp lại", TM "tối đa 3 cặp mỗi câu". Không nói: (1) `{{source_segment}}` trong prompt người dùng thay bằng gì khi có 100 câu (khối đầu? tất cả? để nguyên?); (2) Glossary/TM chỉ chèn khi thân có marker (bản giữ ledger) hay luôn chèn một mục riêng (bản "đơn giản"); (3) `omitted_count`/`hidden_count` của TM tính theo câu hay theo lượt.
- Hai bản đều đúng chữ: bản 1 tái dùng `assemble_prompt` cho phần thân rồi nối các khối; bản 2 viết hàm lượt mới bỏ marker, luôn chèn Glossary ở cuối. Bản 2 làm người dùng đã bỏ `{{glossary_terms}}` để TẮT Glossary (FR69) nhận Glossary không mong muốn, còn ledger "NotAsked" nói sai. Cổng canh AD-36 "mục chờ chốt không vào prompt" vẫn xanh, vì truy vấn chung không đổi.
- TM trộn trong lượt mất gắn kết câu: cặp TM của câu 7 hiện chung, mô hình không biết nó dành cho SEGMENT nào; AD-18 xếp ưu tiên `tôi dịch` theo từng câu nhưng gộp trộn thì thứ tự mất.

Siết Rule 3:
> 3a. Marker trong thân prompt giữ nghĩa cũ: `{{glossary_terms}}` và `{{tm_similar_segments}}` mở rộng thành phần đã gộp của lượt, chỉ khi thân mang marker; thân không mang ⇒ `NotAsked` cho cả lượt, không chèn gì (FR69, FR70). `{{source_segment}}` trong thân ở chế độ lô = không được thay, giữ nguyên văn và ghi cờ `source_segment_missing`-tương-đương vào sổ; các khối SEGMENT luôn đi sau thân.
> 3b. Cặp TM đi KÈM khối của câu nó thuộc về (một mục con trong khối, hoặc nhãn `for SEGMENT n`), thứ tự theo ưu tiên AD-18 trong từng câu. Sổ chèn là sổ của lượt: đếm `omitted_count`/`hidden_count` theo câu rồi cộng.
> 3c. Không có một hàm lượt thứ hai viết lại khâu mở rộng marker; hàm lượt gọi lại `assemble_prompt` cho phần thân.

---

## F5 (Trung) — Máy trạng thái hàng lô: "chưa dịch", "lỗi", "chưa gửi", "đã nhận" và tên trùng với trạng thái segment

Cặp: C-webview (hàng lô, nút thử lại) × B-wire.

- Rule 5 đặt trạng thái "chưa dịch"; Rule 7 gọi "chưa dịch hoặc lỗi"; Rule 6 dừng thao tác khi lỗi nhà cung cấp, nghĩa là các lượt sau và các câu chưa đóng trong lượt đang chạy có trạng thái nào? Không phải "chưa dịch" (đó là kết quả đối chiếu của lượt đã kết thúc), không phải "lỗi" (lỗi của lượt, không của câu), không phải "đã nhận". Huỷ giữa lượt cũng vậy (test (f) chỉ canh huỷ GIỮA hai lượt).
- Hai bên chọn khác nhau: wire chỉ phát `Received` / `Discarded` cho câu đã vào lượt gửi đi, webview đặt "chưa gửi" cho phần còn lại; hoặc wire phát `Discarded{reason: not_sent}` cho cả phần còn lại. Nút "thử lại" gom khác nhau: một bản chỉ thử lại câu `Discarded`, bản kia kể cả các lượt chưa gửi (tốn tiền hơn người dùng mong).
- Nhãn "chưa dịch" trùng nghĩa với `target_text = ''` (*chưa dịch*, AD-47 ④ ghi `''` *(chưa dịch)*). Câu có bản dịch cũ nhưng bị chọn dịch lại, bị "chưa dịch" trong lô, hiện hai trạng thái trái nhau. Rule 2 và Rule 1 dùng "cần dịch", "đã dịch", `is_omitted` mà không định vị vị từ: "đã dịch" là `target_text` khác rỗng, có `translation_origin`, hay đã xác nhận? TM 100% prefill (FR58) có tính "đã dịch"? Hai bên chọn khác ⇒ cùng vùng chọn ra hai chia lượt khác (test (h) 250 câu chỉ đúng với một vị từ).

Siết: thêm một bảng đóng "Trạng thái hàng lô" trong Rule 5: `queued` · `streaming` · `received` · `untranslated(reason)` · `not_sent`, mỗi trạng thái nói sự kiện nào đặt nó, và nút thử lại chỉ gửi `untranslated`, còn `not_sent` đi qua "dịch tiếp" cần bấm riêng. Đặt tên khác cho nhãn hiển thị để khỏi trùng `''`; định nghĩa "cần dịch" = `target_text` rỗng sau cắt khoảng trắng và `is_omitted = 0`, "đã dịch" = `target_text` khác rỗng, bất kể xuất xứ.

---

## F6 (Trung) — Lượt chạy lâu mà văn bản/segment đổi bên dưới; không có phép kiểm "còn đúng câu lúc lắp" trước khi đưa sang Editor

Cặp: A/B (lượt chạy ~15 giây theo đo, 100–114 giây cả Chương) × segment merge/split/edit (AD-5, AD-35) × E-đường đưa sang Editor.

- AD-53 đã học bài này cho Proofreader: kiểm văn bản đã khác lúc quét. AD-60 không có phép kiểm tương đương. Trong lúc lượt chạy, người dùng gộp/tách (retire + create theo AD-5) hoặc gõ vào câu k. `Received{segment_id=k}` đến cho id đã về hưu hoặc câu đã đổi nguồn.
- Bên B (gửi tiếp lượt k+1 theo danh sách chốt lúc đầu) và bên C (hiển thị theo id hiện tại) không biết bên kia làm gì. Đưa bản dịch của nguồn cũ vào segment mới sinh từ gộp là đúng loại hỏng AD-47: ghi `other` lên chữ lệch nguồn.
- Khối ngữ cảnh mang bản dịch HIỆN CÓ của câu ngăn; nếu người dùng đang gõ, bộ đệm AD-35 chưa flush ⇒ prompt mang bản dịch cũ. Chấp nhận được nhưng chưa ghi ra.

Siết: mỗi `Received` đi kèm `source_hash` do Rust lắp lúc dựng lượt; lệnh đưa sang Editor kiểm id còn sống và nguồn còn khớp hash, không khớp ⇒ từ chối, không ghi gì. Gộp/tách trong lúc lô chạy: thao tác lô hủy phần còn lại hoặc, nếu AD-5 cấm, khoá gộp/tách tới khi lô xong (chọn một, đo trước).

Mâu thuẫn kế thừa liên quan: Rule 10 viết "Không đổi: AD-47". Nhưng AD-47 bảng ③ có hàng "Đưa đề xuất AI sang Editor", mà AD-60 làm đề xuất AI thành nhiều đề xuất đến từ một lượt, không hề nói đề xuất nào đủ điều kiện được đưa sang. F2 và F6 là phần AD-47 thật sự phải nhận một dòng 🔵.

---

## F7 (Trung) — Usage theo lượt, bản ghi prompt theo lượt: cộng dồn đôi, và "prompt của lượt chứa câu đó" mơ hồ

Cặp: C-webview (tổng token/tiền của thao tác, FR76) × B-wire × màn xem prompt (FR71).

- Rule 4: sự kiện usage mang danh sách `segment_id` và usage của lượt, "không chia cho từng câu". Webview theo thói quen cũ gán usage vào từng hàng; tổng thao tác = cộng usage mọi hàng ⇒ gấp tới 100 lần. Bản khác cộng theo sự kiện. Cả hai đúng chữ. Tiền là thứ người dùng tin (FR75, AD-22).
- Danh sách `segment_id` của usage có gồm câu ngữ cảnh không? Câu ngăn cũng tốn token nhưng không phải "câu cần dịch".
- Rule 8: bản ghi prompt là bản ghi của lượt. `AssembledPromptRecord` hiện khoá theo `segment_id` (một câu). Một câu có thể thuộc nhiều bản ghi: lượt chính, lượt thử lại sau, và là khối ngữ cảnh của lượt kế (Rule 2 mang 3 câu trước sang lượt mới). "Prompt của lượt chứa câu đó" không chọn được giữa các bản. Rule 8 cũng không nói n ⇒ `segment_id` lấy từ đâu (chỉ số trong `segment_ids` có theo thứ tự tài liệu không), mà nút "đánh dấu khối của nó" cần đúng ánh xạ đó.

Siết Rule 4/8: usage là sự kiện của lượt (khoá `call_index`), tổng thao tác cộng theo `call_index` một lần; bản ghi prompt khoá `(operation_id, call_index)` và mang `segment_ids` theo thứ tự n = 1..; màn xem prompt của câu hiện bản ghi MỚI NHẤT mà câu là câu cần dịch, ghi rõ nếu câu chỉ là ngữ cảnh.

---

## F8 (Trung) — Client dùng chung: phán `finish_reason` nằm ở đâu; Proofreader và đường một câu

Cặp: A/B (batch) × D (đường một câu, Proofreader); AD-60 Rule 10 × `7bd1f90`.

- Binds: "`translate` … lượt phải trả `finish_reason` ra ngoài". Không nói phép phán "thiếu `finish_reason` ⇒ cắt" ở trong `translate` hay ở bộ chạy lô. `translate` hiện coi `[DONE]` có chữ là xong và `7bd1f90` thêm `EmptyReply`.
  - Phán trong `translate` ⇒ Proofreader (cùng `translate`) đổi hành vi: nhà cung cấp cục bộ (Ollama, LM Studio) hay bỏ `finish_reason` trong khung cuối thành lỗi ở Proofreader. Rule 10 nói "Proofreader giữ khuôn" nên đây là sửa vỡ ngầm.
  - Phán ở bộ chạy lô ⇒ đường dịch MỘT câu (Rule 1: "không đổi") vẫn cắt im lặng, đúng Prevents 1, và AD-60 không ghi đó là phạm vi bỏ.
- Cũng nên nói: nhà cung cấp không gửi `finish_reason` ở lượt dịch lô (mức hiện chưa đo cho Ollama/LM Studio) khiến mọi câu CHƯA đóng thành "chưa dịch", mà dấu đóng của khối cuối thường có. Đã có `last_finish_reason` mang `Option`, đúng. Nhưng Rule 5 "trọn" được định nghĩa mà không quy tắc nào dùng chữ "trọn" làm điều kiện (chỉ mệnh đề cho từng khối). Hai bên hiểu "lượt không trọn" khác nhau (đánh dấu cả lượt hay không).
- Khung đđặt dấu `<<<SEGMENT` trong `rag.rs` đã là của Proofreader (`PROOFREAD_SEGMENT_OPEN/CLOSE`). Assembler lô thêm hằng thứ hai y hệt hoặc dùng lại hằng Proofreader; sửa một bên (F1) sửa bên kia.

Siết: `translate` trả `(text, finish_reason: Option<String>)` và KHÔNG phán; phán thuộc lớp gọi. Ghi tên ba người gọi và phạm vi: lô phán theo Rule 5; Proofreader giữ nguyên; đường một câu ghi vào Deferred (có chủ) rằng cắt ngang chưa được canh. Hằng dấu dùng chung đặt ở một mô-đun, test khoá hai bên.

---

## F9 (Trung) — Cờ chữ Hán: "nguồn là tiếng Trung" và phạm vi "ký tự Hán" có hai cách hiểu; AD-44

Cặp: A (Rust gắn cờ) × D/C.

- AD-44 và bảng Conventions: tra cứu chọn đường zh/en theo HÌNH DẠNG chuỗi, không theo ngôn ngữ Tác phẩm. Rule 6 viết "Nguồn là tiếng Trung" không nói theo cái gì: ngôn ngữ nguồn của Work, hay nguồn của câu có chứa ký tự Hán.
- Một bản đọc ngôn ngữ Work (câu nguồn tiếng Anh trong Work zh bị cờ), bản kia kiểm hình dạng câu (đúng tinh thần AD-44). Phạm vi "ký tự Hán": CJK Unified chính, thêm Ext A–G, Compatibility, hay cả dấu 「」? Câu zh chứa tên riêng giữ Hán cố ý (Rule 6 nói) nhưng tên Latin hoá không bị cờ.
- Hậu quả: số cờ khác nhau cho cùng bản dịch; test (j) chỉ có một ca, nên cả hai bản xanh.

Siết: định nghĩa theo hình dạng, dùng đúng hàm phân loại của `dict/` (một nơi), cờ chỉ là dữ liệu `has_han: bool` trên `Received`, không chặn đưa sang Editor.

---

## F10 (Thấp–Trung) — Câu ngăn `is_omitted` và ngữ cảnh được đưa lên nhà cung cấp; Prevents 3 hứa nhiều hơn khả năng canh

- `is_omitted` hôm nay chặn dịch một câu (`segment_is_omitted` trong `aitranslate.rs`). Rule 2 gửi câu `is_omitted` LÀM ngữ cảnh, nguồn kèm bản dịch, lên nhà cung cấp. Nếu ý nghĩa của `is_omitted` là "đừng đưa ra ngoài" (quảng cáo, nội dung nhạy cảm, dòng dán chéo), Rule 2 đổi nghĩa mà Rule 10 không liệt kê (AD-15 liên quan: dữ liệu đi qua điểm mạng nào). Cần một câu: hoặc loại câu omitted khỏi ngữ cảnh, hoặc ghi rõ đã chọn gửi.
- Prevents 3 ("bản dịch của k+1 hiện dưới tên k") được cho là đã chặn. Nhưng Rule 5 cấm khớp theo nội dung, chỉ khớp theo chỉ số. Mô hình bỏ câu 5 rồi đánh lại số 5 cho câu 6, đủ n khối, `stop`, chỉ số duy nhất: lượt "trọn", mọi câu "đã nhận", câu 5 mang bản dịch của câu 6. Không cổng nào đỏ. Đo 2026-10-27 ghi "rơi 1–2 trên 356 câu, có `stop`" và không nói trường hợp đánh lại số có xảy ra không. Cần ghi ⚠️ cái mất: Rule 5 canh dấu hiệu cấu trúc, không canh dịch lệch nội dung; hoặc thêm kiểm rẻ không dựa nội dung (độ dài tỉ lệ quá lệch gắn cờ cảnh báo như `has_han`, không loại).

---

## Mâu thuẫn hoặc làm yếu AD kế thừa (tổng hợp)

| AD | Chỗ | Mức |
|---|---|---|
| AD-59 (Binds, mục 7 "không đổi") | `batch_stopped_error(segment_id, …)` mang một `segment_id`; AD-60 Rule 6 "nêu tên lượt đó". Không thể đồng thời giữ nguyên và nêu lượt. | Cao (F3) |
| AD-47 (bảng ③ hàng "Đưa đề xuất AI sang Editor") | Rule 10 ghi "không đổi" nhưng AD-60 tạo đề xuất bán phần, bị rút, hoặc lệch nguồn mà AD-47 không có điều kiện "đủ điều kiện được đưa". | Cao (F2, F6) |
| AD-14 | Dòng 🔵 đã thêm nhưng chưa nói hàm lượt dùng lại hay thay `assemble_prompt`; marker FR69/FR70 mơ hồ. | Cao (F4) |
| AD-36 | Rule 10 "không đổi", song Rule 3 "gộp lại, bỏ trùng theo thuật ngữ nguồn" tạo một quy tắc thứ hai về cái gì vào prompt ngoài truy vấn của `glossary/`; dedupe phải nằm ở `ai/` hay ở `glossary/` chưa rõ. | Thấp–Trung |
| AD-21 | Tham số của lỗi lượt phải là dữ liệu; chưa khai khoá nào. | Trung (F3) |
| AD-22 | Giữ nguyên, nhưng trạng thái "chưa gửi" sau khi dừng không có đường thử lại tường minh (F5). | Trung |
| AD-44 | "Nguồn là tiếng Trung" lệch triết lý chọn theo hình dạng chuỗi. | Trung (F9) |

## Test canh còn thiếu (đếm theo cổng nghe cục bộ)

Mười một ca (a)–(k) của Rule 9 không chạm: (l) nửa chữ rồi luồng đứt (F2); (m) chỉ số trùng không để lại chữ (F2); (n) nguồn chứa `SEGMENT>>>` (F1); (o) thân không mang marker ⇒ không chèn Glossary (F4); (p) gộp/tách trong lúc lô chạy (F6); (q) tổng usage thao tác cộng một lần (F7); (r) Proofreader nhận luồng không có `finish_reason` ⇒ hành vi như trước `translate` đổi (F8). Test (d) "không câu nào khác bị dịch chỗ" chưa có đối chứng đỏ nêu bằng phép GỠ.

## Đề xuất thứ tự đóng

1. F3 và F2 trước (đổi hình dạng dây, phải chốt trước khi hai story Rust và Webview song song).
2. F1 và F4 (đổi nội dung prompt, ảnh hưởng FR69/FR70).
3. F5–F9 gộp vào lượt sửa AD-60 một lần; F10 hỏi Ice (chọn gửi hay không gửi câu omitted; có thêm kiểm độ dài hay ghi ⚠️).
