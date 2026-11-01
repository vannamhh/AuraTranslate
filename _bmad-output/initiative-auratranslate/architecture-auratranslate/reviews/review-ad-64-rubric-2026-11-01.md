# Review AD-64 (dịch nhanh) — rubric, 2026-11-01

Phạm vi đọc: AD-64 (spine dòng 1494), brief `ad-brief-dich-nhanh`, 10 dòng cuối `.memlog.md`, AD-7/8/11/13/14/15/22/30/60, bảng Consistency Conventions; mã `core/aiconfig/keychain.rs`, `core/store/{mod,writer}.rs`, `commands/aitranslate.rs`, `commands/glossary.rs`, `core/ai/{batch,rag}.rs`, `core/glossary/scan.rs`.

## Verdict
Cần sửa trước khi ký: AD chốt đúng phần lớn điểm lệch (kho, khoá keychain, thứ tự đọc, huỷ, dọn) nhưng có 1 mâu thuẫn mã, 2 lỗ cưỡng chế và 4 điểm "chưa chốt ngầm" mà hai đơn vị có thể lệch.

## Findings (theo mức nghiêm trọng)

### F1 (cao) — Rule 1 không có cổng cưỡng chế ở Rust; test 10(a) chỉ phủ một bề mặt
- Rule 1 nói "chỉ khi đủ hai điều kiện" (tuỳ chọn tự dịch bật + cấu hình nhanh đã khai) nhưng không nói điều kiện nằm ở đâu, và không nói tuỳ chọn tự dịch LƯU Ở ĐÂU (global.db? localStorage? trường `AiConfigField`?). Hiện `AiConfigField` chỉ có 6 trường (`core/aiconfig/mod.rs:48`), không có cờ nào.
- Lệnh dịch (`glossary_context_translation` / lệnh khối) dùng chung cho cú bấm `Dịch thử` và tự chạy; AD không đòi lệnh phân biệt hai nguồn kích hoạt. Nếu cổng chỉ ở webview thì bề mặt thứ hai lại tự đặt điều kiện riêng — đúng điều Prevents #2 muốn chặn.
- 10(a) chỉ nói "mở hàng" (Bảng chờ Glossary). Không có ca cho tầng khối nhập URL, không có ca "lớp phủ đã đóng thì không chạy" hay "cuộn dừng chưa đủ 300 ms thì không gọi".
- Sửa: một hàm Rust duy nhất `auto_run_allowed(...)` đọc tuỳ chọn + cấu hình nhanh; lệnh nhận `trigger: Click | Auto` và từ chối `Auto` khi hàm đó false; test (a) chạy ở cả hai bề mặt.

### F2 (cao) — Rule 3 tự mâu thuẫn và lệch tiền lệ AD-8 thật trong mã
- Binds khai `QUICK_TRANSLATION_MIGRATIONS`, Rule 3 nói "lệch phiên bản thì xoá rồi dựng lại" (không di trú). Hai điều không cùng tồn tại; tiền lệ `LIBRARY_INDEX_MIGRATIONS` chỉ là một bộ 1 bước phục vụ so phiên bản.
- Mã thật: nhánh không-di-trú nằm ở `Indexer::open`, KHÔNG ở `Store::open` (`core/store/mod.rs:169-190, 339-345, 995`), và `tests/library_index_boundary.rs` canh "chỉ một chỗ mở". AD-64 không nói ai là chỗ mở duy nhất của `quick-translation.db`, không đòi test biên tương đương, và không nêu `delete_if_schema_version_differs` làm hạ tầng dùng lại. Hai story có thể mỗi bên tự mở kho (vi phạm AD-11 "không module nào tự mở kết nối ghi").
- AD-30 liệt kê kho có phiên bản lược đồ và nói riêng `library-index.db` không di trú; AD-64 thêm kho thứ hai không di trú nhưng không có dòng 🔵 ở AD-30 (Rule 11 chỉ liệt kê AD-8, AD-11, AD-15). AD-8: grep cho thấy chưa có 🔵 (AD-7 và AD-11 đã có, AD-15 đã có).
- `store/mod.rs:745` chỉ chạy `check_glossary_lifecycle` cho Global|Project; kho mới cần được loại tường minh ở các `matches!` — AD không nêu.

### F3 (cao) — Rule 2 / Rule 11 mâu thuẫn với mã về tầng cấu hình của `Dịch thử`
- Rule 2: để trống thì `Dịch thử` "dùng cấu hình chính ở tầng Global". Mã hiện tại: `prepare_context_reference_call` → `resolve_call_config(global, open)` đòi `OpenWork` và phân giải HAI tầng (`aitranslate.rs:311-335`), nên ghi đè tầng Work đang có hiệu lực cho `Dịch thử`.
- Rule 11 lại nói "đường dịch của Bảng chờ Glossary giữ nguyên". Hai câu không cùng đúng: hoặc ghi đè Work biến mất ở cú bấm này (một thay đổi hành vi cần nêu và có test), hoặc Rule 2 sai. AD phải chọn và ghi.
- Cũng chưa nói: trường nào thuộc "cấu hình nhanh" (brief nêu provider/endpoint/model/khoá; temperature, max_tokens, system_prompt thì sao), tên account keychain (Rule 2 nói "một account riêng", không đặt tên — hai story sẽ đặt hai tên), và `provider_needs_key` cho nhà cung cấp cục bộ (`read_api_key`, `aitranslate.rs:768`) áp thế nào cho khoá nhanh.

### F4 (trung bình) — Rule 3/4: khoá bộ đệm không đồng nhất giữa hai bề mặt
- Bảng chờ Glossary gửi `context_example` đã bị cắt ở `CONTEXT_EXAMPLE_CHAR_LIMIT = 200` ký tự (`core/glossary/scan.rs:91`), tức nhiều khi là NỬA câu nguồn. Rule 3 nói "một câu nguồn một hàng" và Rule 4 cấm lưu "nửa câu bị cắt"; nhưng khoá của bề mặt này chính là đoạn cắt, và câu dịch của đoạn cắt được lưu vĩnh viễn. Prevents #5 bị vi phạm bởi chính nguồn vào. Ngoài ra khoá cắt không bao giờ trùng khoá khối cùng câu, nên "dùng chung hai nơi" (Prevents #3) không xảy ra.
- Chưa chốt chuẩn hoá khoá: NFC, cắt khoảng trắng, gấp phồn/giản (AD-63)? `source_text` thô là khoá chính; một story cắt khoảng trắng, một story không ⇒ trượt bộ đệm im lặng.
- Q3 của brief hỏi cả "phiên bản prompt": AD chỉ trả lời về model. Đổi hằng prompt ở `rag.rs` để lại câu cũ vô hạn; câu từ prompt đơn (Glossary) và prompt lô (khối) chung một khoá dù văn phong khác. Ice đã chọn "không theo model"; phần prompt/ngôn ngữ đích thì chưa ai chọn.
- Dọn "cũ hơn N ngày" dùng `created_at`, trúng bộ đệm không làm mới; câu dùng hằng ngày vẫn bị dọn. Nên nêu rõ đó là ý định hay thêm `last_used_at`.

### F5 (trung bình) — Rule 6/7: các khoảng trống về lô khối
- Q5 của brief hỏi "trần số lượt hoặc số ký tự mỗi lượt xem trước". AD chỉ có `QUICK_BATCH_MAX_BLOCKS` (chưa đo, để story). Không có trần ký tự mỗi khối hay mỗi lượt, không có trần tổng lượt cho một lần xem trước — đúng rủi ro "tiền chảy theo từng cú cuộn" của Prevents #1. Một "Chưa chốt" kiểu này cho phép hai đơn vị lệch (một đơn vị đặt trần, một không). Cần ít nhất một hằng ký tự và quyết "một lượt gọi mỗi lần dừng cuộn" hay không.
- Rule 7 nói một `QuickTranslateGeneration`, một lượt tại một thời điểm, và `next()` của bộ đếm huỷ lượt cũ (theo `ContextReferenceGeneration` hiện có, `aitranslate.rs:712`). Cuộn tiếp trong lúc lô đang chạy: lô cũ bị huỷ (kết quả bỏ theo Rule 4, tiền mất) hay chạy xong rồi lô mới xếp hàng? Rule 6 ("khối đang nằm trong lượt chạy không được gửi lại") và Rule 7 ("rời hàng, rời màn... thì huỷ"; không nói cuộn) không quyết. Khối ra khỏi tầm nhìn có huỷ lượt không?
- Tầng khối không có đường bấm tay nào được nêu: nếu tuỳ chọn tắt, người dùng dịch khối bằng gì? Rule 1 chỉ nói tự chạy; Rule 2 nói "cú bấm `Dịch thử`" chung chung. Bề mặt Giữ/Loại khối cần quyết định có nút hay không; nếu không thì tầng khối vô dụng khi tắt tự chạy.
- Rule 6 dùng "dấu khối và bộ đọc khối của AD-60" nhưng `core/ai/batch.rs` gắn với `segment_id`/`plan_calls`/`reconcile` theo chỉ số 1..k; khối web không có `segment_id`. AD không nói tái dùng phần nào (bộ đọc `BlockParser` + `reconcile`) và `has_han`/guard U+200B có áp dụng cho nguồn khối web hay không. Rule 4 "khối đóng đúng theo AD-60" cũng chưa nêu `finish_reason != stop` giữ được khối đã đóng (AD-60 mục 7) ⇒ ở đây vẫn được lưu hay không? Hai đọc khác nhau cho hai kết quả.

### F6 (thấp) — Mâu thuẫn chưa sửa và chi phí không nhìn thấy
- Bảng Consistency Conventions: dòng "Ra mạng — Đúng ba điểm, cả ba theo thao tác người dùng (AD-15)" và "Gọi AI — luôn huỷ được; không bao giờ tự thử lại" không được AD-64 hay Rule 11 sửa. Sau khi AD-15 nới, dòng này sai. Cần 🔵 ở bảng.
- Rule 9: FR71 không áp dụng là hợp lý, nhưng Prevents #1 viện FR75 (tiền người dùng) trong khi tự chạy không ghi usage (FR76) ở đâu; người dùng không thấy được số lượt tự gọi. Q8 của brief chỉ hỏi FR71; hệ quả FR76 chưa được nêu (chấp nhận được, nhưng phải ghi là chủ ý và ai sở hữu nợ).
- Rule 8: `Writer::write` luôn bọc giao dịch (`writer.rs:246-290`), AD đã nhận ra cần đường riêng cho `VACUUM`; chưa nêu chỗ gọi nào được dùng đường đó (chỉ `core/quick_translation::purge`), nên chưa có canh để `global.db`/`project.db` không bao giờ nhận `VACUUM` (AD-30 sao lưu/giao dịch). Test 10(e) kiểm dung lượng giảm nhưng không kiểm đường `VACUUM` bị cấm ở kho khác.
- AD-13: `core/quick_translation` "không phụ thuộc `core::ai`" nhưng không đòi test cưỡng chế như AD-13 yêu cầu (test tự động). Thêm vào danh sách `ai_boundary`.

## Đối chiếu danh sách kiểm
| Mục | Kết quả |
|---|---|
| Chốt đúng các điểm lệch thật, không sót | Phần lớn; sót: nơi lưu/cổng tuỳ chọn tự dịch (F1), chỗ mở kho duy nhất (F2), danh sách trường + tên account (F3), chuẩn hoá khoá (F4), trần ký tự/hành vi cuộn/đường bấm tay (F5) |
| Mỗi mục Rule cưỡng chế được | Rule 1 (một bề mặt), Rule 2, 6, 7 (chỉ (d) cho huỷ chéo) và Rule 3 (mở kho/biên) thiếu test hoặc cổng; Rule 4, 5, 8, 9 có phép gỡ nêu rõ |
| "Chưa chốt" không làm lệch | `QUICK_BATCH_MAX_BLOCKS`, 300 ms, N: chấp nhận được vì là hằng, nhưng phải gọi tên hằng (`QUICK_AUTO_DEBOUNCE_MS`) và đặt một chỗ; thiếu trần ký tự (F5) |
| Phê chuẩn hơn là mâu thuẫn mã | F2 (MIGRATIONS vs rebuild; chỗ mở), F3 (hai tầng vs Global) |
| Trả lời đủ §4 brief và §1 | Q1, Q2 (thiếu tên account), Q4, Q6, Q7, Q8 đủ; Q3 thiếu phiên bản prompt; Q5 thiếu trần ký tự/lượt |
| Không mâu thuẫn AD khác khi không sửa | Mâu thuẫn bảng Consistency (F6), AD-30 không nhắc kho không-di-trú thứ hai (F2), AD-8 chưa có 🔵 dù Rule 11 hứa |
