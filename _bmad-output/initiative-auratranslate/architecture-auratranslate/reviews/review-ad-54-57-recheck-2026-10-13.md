# Recheck AD-54 · AD-55 · AD-56 · AD-57 sau khi viết lại (2026-10-13)

Phạm vi: spine dòng 990–1140 (bốn AD) cộng các dòng 286, 439, 732, 800, 821, 955, 974, 1146 và Capability Map; ba review cùng ngày; mã đọc: `core/store/schema.rs` (trigger `glossary_entry_*`, hai thang di trú), `commands/segment/history.rs::restore_segment_version`, `commands/segment/targets.rs::write_non_user_target`, `core/tm/mod.rs`, `core/tm/tmx.rs`, `commands/tm.rs`, `commands/segment/tm_match.rs`, `core/ai/rag.rs`. Không chạy gì; không sửa spine hay mã. Số dòng AD dưới đây là số dòng trong tệp spine.

## Phán quyết

Bản viết lại đóng gần hết (22 trên 31 phát hiện), AD-54 và AD-56 sạch ở lõi; AD-57 còn ba chỗ hở mới do chính phần sửa gây ra (kiểu chính sách không dựng được chính sách xuất riêng, số "cặp bị ẩn" không đo đúng thứ cần nói, `TierRows` mang chính sách nhưng không ai buộc bên tiêu thụ kiểm nó).

## Bảng đóng từng phát hiện

Ký hiệu: closed · partly · open. Không phát hiện nào ở trạng thái rejected-with-reason.

### Rubric

| # | Trạng thái | Chứng cứ |
|---|---|---|
| H1 dải chốt FR146 | closed | AD-57 Binds (dòng ~1089) nêu "đề xuất TM của dải chốt Glossary"; Rule 5 xếp nó vào *Áp*. |
| H2 người ghi `translation` thiếu | closed | AD-54 Rule 3 bỏ ý "mỗi người ghi tự xoá cột", Rule 4 thêm `AFTER UPDATE OF translation` tự xoá; "Không người ghi `translation` nào khác phải biết cột tồn tại". Không còn danh sách để thiếu. Khoảng trống còn lại: không điều khoản nào đòi test mỗi đường trên hàng đã chốt (xem adversarial H1). |
| H3 nhãn 12.19 | partly | AD-55 Rule 6 có bảng bốn giá trị→bốn nhãn, `current_version_id` ở Rule 5. Còn thiếu: không nợ có chủ để sửa `verify`/`description` của ticket 12.19 (đang đòi "gõ tay · từ AI · từ TM"); không nói "bạn sửa · đã xác nhận" (mockup A17) thuộc hay không thuộc FR141 (Rule 6 chỉ nói *đã xác nhận* không phải nhãn riêng). |
| M1 Conventions | closed | Dòng 1146 có 🔵 2026-10-13 đứng đầu câu. |
| M2 cổng `MineFirst` | closed | AD-57 Rule 3: kiểu `TmSidePolicy` chỉ dựng qua hai hàm, tham số bắt buộc, cổng đọc dòng mã cho `browse()`, đối chứng đỏ bằng đổi một chỗ gọi của điền sẵn sang `browse()`. Xem N1 và N3 cho lỗ mới. |
| M3 nút gỡ chốt | closed | AD-54 Rule 6: lối vào duy nhất, không phím mặc định, điều kiện là trạng thái hàng, không cửa sổ thời gian. |
| M4 mất chữ | closed | AD-54 Rule 7 ⚠️ "Cái mất, ghi ra". |
| M5 `proof_decision` | closed | AD-56 Rule 2 (bảng là lịch sử, `decision_missing` không ghi gì kể cả văn bản), Rule 3 (cộng dồn, không đặt lại). |
| M6 AD-55 diff/đang dùng | partly | Đã đóng: flush trước khi đọc (Rule 5), so sau `trim`+NFC qua một hàm chung (Rule 3). Còn mở: trần diff chỉ là ⚠️ "Chưa đo" giao cho 12.19 đo, chưa có ngưỡng; chấp nhận được nhưng chưa phải quyết định. |
| L1 dấu giữa câu | closed | Đã quét dòng 1000–1140: mọi 🔵/⚠️ đứng đầu câu hoặc đầu khối; AD-54/55/56 dùng "nhận một dòng cập nhật". |
| L2 Capability Map | closed | C9 có đủ AD-54..AD-57; C4/C5/C6/C7 khớp Binds. |
| L3 di trú | closed | AD-57 Rule 9 gán thứ tự 12.14→12.15→12.19→12.20, "số bậc kế tiếp có thật lúc dựng"; AD-56 Rule 1 có `AUTOINCREMENT`; AD-55 Rule 1 trigger tạo sau mọi lượt chép. |

### Reality

| # | Trạng thái | Chứng cứ |
|---|---|---|
| F1 hai người ghi thiếu | closed | Như rubric H2; `TakeTheirs` và `candidate_store` được trigger tự xoá phủ, nhánh sửa của `confirm_translation` là câu lệnh khác (Rule 3). |
| F2 TMX không qua ba hàm | partly | `distinct_tier_pairs` đã vào Binds và Rule 8; `merge_tagged_tiers` (private) đã bỏ. Còn mở: Rule 3 nói "các hàm đọc công khai của `core/tm`" nhưng mã còn `existing_pair_keys` (tmx.rs:571), `pair_by_id`, `live_copies`, `fuzzy_pairs_for_source` (pub, không ai gọi) đọc `tm_unit` mà không nhận chính sách, và AD không nêu chúng được miễn. Lỗ mới của lối xuất: N1. |
| F3 cổng không tồn tại | closed | AD-56 Rule 5 nói thẳng phép thử chưa dựng và AD này không dựng. |
| F4 chung hàm accept/revert | closed | Rule 2: kết quả accept là kiểu riêng mang `decision_id`. |
| F5 "cùng khuôn `status_override`" | closed | AD-57 Rule 1: "như `status_override` (cột đó không có ràng buộc; cột này có)". |
| F6 phép kiểm tập | open | AD-55 Rule 1 vẫn viết "phủ thêm cột này" mà không nói `reject_unknown_translation_origin` dùng tập bốn giá trị có `''` còn cột này ba giá trị. Low. |
| F7 tên `PROOF_IGNORE` | open | AD-56 Rule 1 vẫn "cấu trúc như `PROOF_IGNORE`" (bảng là `proof_ignore`). Low. |

### Adversarial

| # | Trạng thái | Chứng cứ |
|---|---|---|
| H1 AD-54 người ghi | closed | Trigger `AFTER` (Rule 4) đúng như đề nghị 1; `update_manual_term` sửa mỗi `note` không đổi `translation` nên không chạm cột (điều kiện `NEW.translation IS NOT OLD.translation`); `WHERE … translation IS NULL` ở nhánh chốt (Rule 3). Đề nghị 4 (test mỗi người ghi trên hàng đã chốt) không được ghi: Rule 8 chỉ có phép kiểm lược đồ. Chấp nhận vì lỗi cấu trúc đã hết, nhưng "guard must be able to fail" vẫn thiếu một test chạy thật `TakeTheirs` trên hàng có cột. |
| H2 chiều rẻ | closed | AD-57 Rule 10 ⚠️ ghi cái mất, Ice chọn (b); AD-47 ④/AD-53/AD-55 được nêu tên. Số đo 47 `.atproj` không có (dữ liệu thử), AD nói rõ ở lựa chọn. |
| H3 AD-57 | partly | 3.1 đếm cái bị ẩn: Rule 6 có, nhưng đo sai thứ (N2). 3.2 hai chỗ gắn chính sách: `TierRows` mang chính sách, lọc ở `read_tier_rows`, nhưng không buộc bên tiêu thụ (N3). 3.3 cờ đổi giữa lô: closed (Rule 6 đọc một lần). 3.4 điền sẵn đã lưu: closed (Rule 7). |
| M1 nút gỡ chốt | closed | `expected_pending_confirmed_at` (Rule 5), nút chết theo đổi Chương/Tác phẩm (Rule 6), tập *Để sau*, làm mới dấu, `promote_to_global` trong ⚠️. |
| M2 AD-55 | closed | Khôi phục và nhãn dùng cùng hàm chọn (Rule 3, 4), Rule 6 ghi lịch sử rỗng sau gộp/tách. Xem N4 cho phần ④/⑤ chưa đổi. |
| M3 AD-56 | partly | Hạt quyết định (Rule 4), `AUTOINCREMENT`, `DELETE … AND outcome AND kind`, thứ tự phép kiểm AD-53 trước (Rule 2): đóng. Câu "ngoài lệnh hoàn lại, không đường nào sửa hay xoá `proof_decision`" là điều kiện sống còn của lệnh hoàn lại và vẫn không có cổng (AD tự nói không dựng cổng). |
| M4 di trú | closed | AD-57 Rule 9 cho thứ tự; AD-54 Rule 8 phép kiểm lược đồ hai kho (cột + hai trigger) bắt bậc dựng lại quên cột. |
| L1 lỗi trigger thô | open | AD-54 không nói `confirm_translation` ánh xạ lỗi trigger ra lỗi có kiểu. Low. |
| L2 xuất TMX | closed | Rule 8; xem N1 cho phần chính sách. |
| L3 lô khi đóng Tác phẩm | closed | Rule 3 + 6 (chính sách đọc một lần, đi xuống cùng ảnh chụp). |
| L4 `kind` skipped | closed | AD-56 ⚠️ ở Rule 4. |
| L5 diff chưa đo | partly | AD-55 Rule 5 ⚠️ "Chưa đo", story đo ≥ 50 phiên bản, phương án dự phòng nêu sẵn; chưa có số. |

Tổng 31 phát hiện: closed 22 · partly 6 · open 3.

## Cặp tương tác đã kiểm, không có lỗ

AD-54 `AFTER UPDATE OF translation` với `BEFORE UPDATE OF translation, pending_confirmed_at`: câu `UPDATE … SET pending_confirmed_at = NULL` lồng không đặt `translation` nên không kích lại chính trigger `AFTER`; nó kích trigger `BEFORE` (cột `pending_confirmed_at` nằm trong danh sách `OF`) nhưng cả hai mệnh đề đều sai (`translation` không đổi, `pending` đi từ khác `NULL` sang `NULL`), nên qua. `recursive_triggers` không được đặt ở đâu trong `src-tauri/src` (grep không ra) và không cần, vì trigger khác nhau gọi nhau không phải đệ quy. `CHECK (pending_confirmed_at IS NULL OR translation IS NOT NULL)` chỉ đỏ khi `translation → NULL` mà `pending` còn; trigger `BEFORE` ABORT trước, nên hành vi cũ giữ nguyên. Trigger zero-width (`glossary_entry_update_rejects_zero_width_only_text`, `schema.rs:625`, `BEFORE UPDATE OF translation, note`) không bị kích bởi câu lồng (không đặt hai cột đó) và với `SET translation = NULL` của lệnh gỡ chốt thì `NEW.translation IS NOT NULL` sai, nhánh `note` cho `NULL`: qua. Hai thang di trú: global chỉ có bậc của 12.20, project có bốn bậc; "sau bậc của Story 12.19" ở AD-54 Rule 8 đúng cho thang project, thang global chỉ cần "bậc kế". Câu hỏi nhỏ về chữ, không phải lỗ.

AD-56 thứ tự kiểm của lệnh hoàn lại: phép kiểm văn bản AD-53 mục 4 chạy trước nên bấm đúp (lượt hai thấy văn bản đã hoàn) nhận lỗi văn bản, không đụng `decision_missing`; `DELETE … WHERE id AND outcome='accepted' AND kind` cùng giao dịch, 0 hàng ⇒ rollback cả lượt ghi văn bản: nhất quán. Kiểu kết quả accept riêng: không lệch `PromoteAiTranslationOutcome` dùng chung với `promote_ai_translation`.

## Phát hiện mới

### N1 (medium) AD-57 Rule 3 và Rule 8 mâu thuẫn: xuất `mine`/`all` không có chính sách để dựng
`TmSidePolicy` chỉ dựng được bằng hàm đọc cờ và `browse()`, và `browse()` bị cổng giới hạn "đúng hai chỗ" (concordance, danh sách quản lý). Rule 8 đòi `tm_export_tier` lọc `all | mine` độc lập với cờ và `distinct_tier_pairs` "lọc bằng cùng hàm ở mục 4". `mine` cần một `MineOnly` không phụ thuộc cờ, `all` cần `MineFirst` mà không phải một trong hai chỗ được phép. Cả hai đều không dựng được, hoặc buộc story 12.14 thêm biến thể thứ ba và chỗ `browse()` thứ ba, phá cổng. Sửa: nêu hàm dựng thứ ba (ví dụ `explicit(Mine|All)` chỉ cho `tm_export_tier`) và liệt kê nó trong cổng.

### N2 (medium) AD-57 Rule 4 và 6: "số cặp bị ẩn" đếm ở `TierRows` là số trên cả kho, không phải số cặp đáng lẽ khớp
Chính sách lọc trong `read_tier_rows`, trước khi biết câu nguồn; `TierRows` mang "số cặp bị ẩn" của cả kho (mã: `read_tier_rows` đọc mọi hàng). Điền sẵn, khớp mờ và prompt dùng nó nói "rỗng vì công tắc" khi số đó > 0, kể cả khi không cặp ẩn nào khớp câu này: đúng lớp báo sai "N cặp bị ẩn" thay cho "không có" mà Rule 6 muốn diệt, theo chiều ngược. Đồng thời `pairs_for_source` trả `Vec<TmPair>` hôm nay, không có chỗ mang số đếm. Sửa: số cần đếm là số cặp ẩn *khớp bề mặt đó* (cùng nguồn, hoặc đạt ngưỡng mờ), tính bằng cùng phép khớp trên tập bị loại; nêu kiểu kết quả mới của `pairs_for_source`.

### N3 (medium) AD-57 Rule 3–4: chính sách là thuộc tính của ảnh chụp nhưng bên tiêu thụ không buộc kiểm nó, và cổng `browse()` không nêu hạt
`fuzzy_pairs_in_candidates(&TierRows, …)` và `rank_concordance(TierRows, …)` nhận `TierRows` bất kể chính sách đã đọc. Một `TierRows` đọc bằng `browse()` (concordance) truyền vào đường RAG vẫn biên dịch; AD nói ảnh chụp "mang chính sách" nhưng không đòi hàm *áp* từ chối ảnh chụp `browse()`. Cổng "`browse()` xuất hiện ngoài hai chỗ" không nêu hạt: `commands/segment/tm_match.rs` chứa cả gợi ý mờ (áp, dòng 130) lẫn concordance (miễn, dòng 235), nên cổng mức tệp không phân biệt được, phải mức hàm. Sửa: hàm nhận `TierRows` ở các bề mặt *áp* kiểm `policy != Browse` (kiểu riêng hoặc assert trả lỗi), và cổng nêu hạt là tên hàm.

### N4 (low) AD-55 Rule 3: "hai thứ luôn chỉ cùng một phiên bản" chỉ đúng giữa nhãn và xuất xứ, không đúng với hai phép so còn lại của khôi phục
`restore_segment_version` ④ so byte (`current_text == version_text`) và ⑤ `has_copy` so byte (`history.rs`), trong khi nhãn *đang dùng* so sau `trim`+NFC. Văn bản hiện tại có khoảng trắng cuối và phiên bản tương ứng: nhãn sáng hàng đó, nhưng bấm khôi phục chính hàng ấy không phải no-op và hiện cảnh báo "bản nháp chưa ký sẽ mất". AD không nói ④/⑤ có đổi sang hàm chung hay giữ byte. Sửa: một câu quyết định.

### N5 (low) AD-54 Rule 5: kiểu của `expected_pending_confirmed_at`
Nếu là `Option`, bản gửi `NULL` khớp hàng `NULL` thì qua phép kiểm thứ ba, rồi `SET translation = NULL` bị trigger `BEFORE` ABORT (chiều lùi không kèm xoá cột) ra lỗi kho thô thay vì `glossary.not_unconfirmable`. Câu "kể cả `NULL`" mơ hồ. Sửa: tham số không `Option` hoặc phép kiểm đòi cột hàng khác `NULL`.

### N6 (low) AD-55 Rule 1: trigger `BEFORE INSERT … ABORT` làm đỏ mọi chỗ chèn `segment_version` ngoài `confirm.rs`
Có bốn câu `INSERT INTO segment_version` trong `tests/alignment_contract.rs`, `tests/segment_contract.rs`, `tests/project_contract.rs`; story 12.19 phải sửa cả ba trong cùng commit. AD chỉ nhắc `confirm.rs`. Đưa vào phạm vi "Tests that move" của story.

## Việc còn lại cho Winston

Vá N1, N2, N3 trước khi giao 12.14; N4, N5 là một câu mỗi cái; F6, F7, L1 và nợ ticket 12.19 (H3) mở một mục sổ nợ có chủ.
