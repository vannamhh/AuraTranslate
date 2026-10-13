# Review rubric AD-54 … AD-57 (2026-10-13)

Phạm vi: spine dòng 996–1115 (bốn AD mới), các 🔵 tại dòng 286, 439, 732, 800, 821, 955, 974, Capability Map (1418–1425), Consistency Conventions (1126). Đối chiếu FR141/147/149/150, tickets 12.14/12.15/12.19/12.20, `.memlog.md` cuối tệp, mã `src-tauri/src` (chỉ đọc).

## Phán quyết

Bốn AD đúng hướng Ice đã chọn (A cho G1–G4) và không mâu thuẫn AD-1/3/13/18/20/21/31/34/47/49/50/53, nhưng chưa đạt: AD-57 bỏ sót một đường tiêu thụ TM mà ticket 12.14 đòi lọc (dải chốt Glossary FR146), AD-54 liệt kê thiếu một người ghi `translation` nên trigger mới sẽ ném lỗi lúc chạy trên dữ liệu thật, và ticket 12.19 đòi ba nhãn "gõ tay · từ AI · từ TM" trong khi AD-55 gộp AI và TM vào một giá trị. Cần vá trước khi ký.

## Phát hiện

### H1 (high) AD-57 không nói gì về đề xuất bản dịch TM của dải chốt Glossary (FR146)
- Vị trí: AD-57 Binds (~1091) và Rule 4 "Ai áp" (~1104–1108).
- Ticket 12.14 viết "Bộ lọc FR149 đặt trong hàm truy TM mà dải chốt của story 10 dùng" và `verify` đòi "dải TM và dải chốt không còn đề xuất từ cặp nhập ngoài". Ticket 12.10 (FR146) đề xuất bản dịch lấy từ TM cho thuật ngữ. Danh sách *Áp* chỉ có điền sẵn FR58, khớp mờ FR59, Smart RAG, dịch lô; danh sách *Không áp* chỉ có concordance và quản lý TM. Dải chốt không nằm ở danh sách nào, và `Binds` không nêu chỗ gọi của nó.
- Hai story có thể quyết khác nhau: 12.10 gọi `MineFirst` (compile vẫn qua), 12.14 theo `verify` thì phải `MineOnly`. "Dải TM" trong ticket cũng không được ánh xạ sang hàm nào.
- Sửa: thêm vào Binds chỗ gọi của FR146 và vào *Áp*: "đề xuất bản dịch TM của dải chốt Glossary (FR146) — `MineOnly` khi cờ bật; không có cặp của tôi ⇒ không đề xuất, không rơi sang cặp người khác". Ghi "dải TM" của mockup là FR59 hay concordance để `verify` của 12.14 đọc được một nghĩa.

### H2 (high) AD-54 Rule 3 liệt kê thiếu người ghi `translation`; trigger mới biến thiếu sót đó thành lỗi lúc chạy
- Vị trí: AD-54 Rule 3 (~1009) và Rule 4 (~1010–1015, mệnh đề 2).
- Mã hiện có bốn câu `UPDATE glossary_entry SET translation`: `core/glossary/store.rs:266` (chốt), `store.rs:1045` (sửa thủ công), `core/glossary/candidate_store.rs:376` (thu hoạch review), `store.rs:2029` (nhập CSV, nhánh `TakeTheirs` của xung đột). Rule 3 chỉ kể "sửa thủ công, thu hoạch review"; nhập CSV không có tên.
- Với trigger mới (đổi chữ khi `NEW.pending_confirmed_at IS NOT NULL` ⇒ `RAISE(ABORT)`), người ghi nào quên đặt cột về `NULL` sẽ làm cả giao dịch hỏng trên đúng các hàng FR114 đã chốt. Fixture test dùng hàng `pending_confirmed_at = NULL` nên xanh; lỗi chỉ lộ với dữ liệu thật. Đây là lớp "xanh nhưng sai" của AGENTS.md.
- Sửa: Rule 3 liệt kê đủ bốn đường theo tên hàm và đòi một test mỗi đường chạy trên hàng đã chốt qua `confirm_translation` (hàng có `pending_confirmed_at` khác `NULL`), kèm khẳng định cột về `NULL` sau lượt ghi. Thêm một câu: người ghi `translation` thứ năm là một lượt sửa spine.

### H3 (high) Ticket 12.19 mâu thuẫn AD-55 về nhãn xuất xứ; AD không khai ánh xạ nhãn
- Vị trí: AD-55 Prevents 3 và Rule 7 (~1040, ~1050); tickets.toml entry 19 (`description` "bốn nhãn của mockup", `verify` "ba nhãn đúng: gõ tay, từ AI, từ TM").
- AD-55 giữ ba giá trị (`self`, `other`, `bilingual_import`) nên AI và TM cùng ra *người khác dịch*. `verify` của ticket không thể đạt. AD ghi "lệch có chủ đích" nhưng không nói nhãn người dùng thấy cho từng giá trị (và `NULL`), nhãn *đang dùng*, hay chỗ của "bạn sửa · đã xác nhận" (mockup A17). Ba cách đặt nhãn khả dĩ cho `bilingual_import`.
- Sửa: Rule 8 mới của AD-55 là bảng 4 hàng giá trị → khoá i18n (`self`, `other`, `bilingual_import`, `NULL`) và nêu "đang dùng" lấy từ `current_version_id`. Ghi rõ "bạn sửa · đã xác nhận" thuộc hay không thuộc FR141. Mở nợ có chủ (Winston → correct-course) sửa `verify`/`description` của ticket 12.19 sang ba giá trị thật; đó là việc ngoài spine nhưng phải có tên.

### M1 (medium) Consistency Conventions chưa cập nhật, và AD-55 Rule 2 khẳng định điều dòng đó phủ nhận
- Vị trí: dòng 1126 (hàng *Xuất xứ bản dịch*) so với AD-55 Rule 2 (~1045) "vẫn là chỗ thứ hai … không phải chỗ thứ ba".
- Hàng 1126 nói cột đó chỉ ghi ở hai chỗ và "ba giá trị cộng `''`". Cột mới `segment_version.translation_origin` không thuộc hàng, tập của nó không có `''`. Người đọc Conventions không biết cột thứ ba tồn tại.
- Sửa: thêm vào hàng 1126 một 🔵 2026-10-13 đứng đầu câu: `segment_version.translation_origin` chỉ ghi ở lượt chèn trong `confirm_segment`, tập ba giá trị (không `''`) cộng `NULL` cho hàng có trước (AD-55).

### M2 (medium) AD-57 Rule 3–4: tham số bắt buộc không ngăn chỗ gọi tự gõ `MineFirst`
- Vị trí: AD-57 Rule 3 và 4 (~1100–1108); Prevents 2.
- Prevents 2 là "chỗ gọi mới quên đọc công tắc, rơi im lặng về *ưu tiên của tôi*". Tham số không mặc định chỉ buộc chọn; chỗ gọi vẫn viết `MineFirst` cho khỏi nghĩ và biên dịch xanh. "Một hàm đọc duy nhất" không có cổng.
- Sửa: Rule 3 thêm cổng đọc mã (khuôn `check:*` hoặc test trong `tests/`) đỏ khi `MineFirst` xuất hiện ngoài hai miễn trừ có tên (concordance, danh sách quản lý TM) và hàm đọc cờ; đối chứng đỏ theo AGENTS.md bằng cách gỡ cuộc gọi hàm đọc cờ ở một chỗ gọi. Thêm đếm: số chỗ gọi `read_tier_rows`/`pairs_for_source`/`merge_tagged_tiers` đều được liệt kê trong Binds.

### M3 (medium) AD-54: Prevents 5 không có Rule đỡ; điểm vào và thời hạn gỡ chốt chưa quyết
- Vị trí: AD-54 Prevents 5 (~1004), Rule 5–6 (~1016–1021).
- Prevents 5 nói tiêu điểm trở về ô gõ sau lượt chốt, nên `Mod+Z` nhường vùng gõ (AD-49 ①) sẽ không bao giờ bắn đúng lúc cần. Rule 6 chỉ nói "phím của nó nhường vùng gõ" và "hiện phím thật", không nói lệnh có phím mặc định không. Hai story có thể gán `Mod+Z` (vi phạm AD-49) hoặc `Mod+Shift+Z`, hoặc không gán.
- `pending_confirmed_at` sống mãi đến khi có lượt sửa, trong khi FR147 nói "ngay sau khi chốt". Rule 6 gọi nút trên thông báo là "lối chính" nhưng không cấm lối khác (dòng trong danh sách Glossary). Một story khác có thể đặt nút gỡ chốt cạnh mục đã chốt từ tuần trước.
- Sửa: Rule 6 ghi lệnh không có phím mặc định (người dùng gán qua FR22) hoặc chọn một phím không phải `Mod+Z`; nêu lối vào duy nhất là thông báo *Đã chốt* và bản ghi `pending_confirmed_at` là điều kiện cho phép chứ không phải cửa sổ thời gian, hay đặt một ngưỡng thời gian có tên.

### M4 (medium) AD-54: gỡ chốt xoá chữ người dùng không bản sao; luận cứ lớp (i) chưa nói ra điều đó
- Vị trí: AD-54 Rule 1, 5, 6; AD-49 (i) và mục 3 (`glossary_entry` là nội dung người dùng).
- Lượt chốt đổi `translation` từ `NULL` sang chữ; nếu chữ do người dùng gõ tay trong dải, gỡ chốt đặt lại `NULL` và chữ đó mất, không bản sao, không hỏi. Rule 6 lập luận đúng ở khía cạnh "trả đúng cột đã đổi" nhưng bỏ qua rằng AD-49 (i) đòi lệnh nghịch đảo "trả lại mọi nội dung người dùng" mà lệnh đi đã đổi, và ở đây nội dung nằm ở lệnh đi.
- Sửa: một câu nêu thẳng đây là cái mất chấp nhận được (người dùng vừa gõ chữ đó vài giây trước, gỡ chốt là chủ ý huỷ) hoặc để thông báo giữ chữ vừa chốt trong webview đến khi đóng để "Chốt lại" được; chọn một và ghi vào Rule 6.

### M5 (medium) AD-56: các đường làm đổi một quyết định đã ghi chưa được quyết đồng đều
- Vị trí: AD-56 Rule 2 (đoạn cuối "Không đường nào khác … xoá"), Rule 3.
- Hoàn lại xoá hàng *accepted*; gỡ một mục khỏi *Chỗ đã bỏ qua* (xoá `proof_ignore`, nếu 9.6 có thao tác này) để lại hàng *not_error*, nên phần trăm *không phải lỗi* trôi đúng lớp "trôi không tính lại được" mà phương án bộ đếm bị loại vì nó. Quét lại sau khi gỡ ghi nhớ sinh phát hiện mới và quyết định mới cho cùng chữ ký.
- Ranh giới phạm vi: tỉ lệ cộng dồn cả đời Tác phẩm, không theo lượt quét; ticket 12.15 `verify` ("1 · 0 · 1 và còn sau khi mở lại") hợp với cộng dồn, nhưng mockup nói ba phần cộng lại 100% mà không nêu phạm vi. AD chưa nói "không theo lượt quét, không đặt lại".
- Hoàn lại khi `decision_missing` chặn cả lượt khôi phục văn bản; cần nói đó có chủ ý (văn bản giữ nguyên, báo lỗi có kiểu).
- Sửa: thêm Rule nêu rõ (a) phạm vi tỉ lệ = toàn bộ quyết định còn lại của Tác phẩm đang mở, (b) gỡ ghi nhớ `proof_ignore` có xoá hay giữ hàng *not_error*, ghi lý do, (c) `decision_missing` không ghi gì kể cả văn bản.

### M6 (medium) AD-55: ba điểm chưa khoá của diff và dòng "đang dùng"
- Vị trí: AD-55 Rule 4–5 (~1047–1048).
- Diff và `current_version_id` đọc `segment.target_text` trên đĩa, còn ô gõ có thể chưa flush (AD-35, trần 5 giây). Overlay mở giữa hai lượt flush hiện diff với bản không phải bản người dùng đang thấy. Cần nêu lệnh đọc lịch sử chạy sau một flush bắt buộc, hay overlay chỉ mở sau flush.
- So `current_version_id` bằng byte, trong khi AD-50 so sau `trim` + NFC. Một câu có khoảng trắng cuối ra "không phiên bản nào đang dùng" dù vừa ký.
- Diff tính cho mọi phiên bản mỗi lần mở overlay; không trần số phiên bản hay số span. Câu đã ký hàng trăm lần làm lệnh chậm im lặng.
- Sửa: Rule 5 so cùng phép chuẩn hoá của AD-50 Rule 4; Rule 4 đòi flush trước khi đọc (hoặc nói rõ diff theo bản trên đĩa); đặt trần phiên bản trả diff (ví dụ N gần nhất) hoặc diff lười theo phiên bản được mở, ghi lý do.

### L1 (low) Dấu 🔵/⚠️ giữa câu trái AGENTS.md
Các chỗ: AD-54 Rule 7 ("nhận 🔵 trỏ AD-54"), AD-55 Rule 7 ("🔵 `segment_version` mang…" sau dấu hai chấm, "chỗ yếu ⚠️ của ⑤", "nhận 🔵 theo mục 3"), AD-56 Rule 5 ("nhận 🔵 —"), AD-49 dòng 800 (🔵 trong ngoặc sau dấu chấm phẩy), AD-50 dòng 821 (🔵 trong ô bảng sau gạch ngang; chấp nhận được vì đứng đầu mệnh đề mới, nhưng nên tách câu). Sửa: viết "nhận một dòng cập nhật trỏ AD-54" thay cho ký hiệu, hoặc đưa ký hiệu về đầu câu.

### L2 (low) Capability Map thiếu C9 cho AD-54/56/57 và C7 cho AD-54
AD-54 đổi lược đồ ở cả hai kho, AD-56 thêm bảng `project.db`, AD-57 thêm cột `work`; chỉ AD-55 được thêm vào C9 (dòng 1425). C4/C5/C6/C7 khớp Binds. Sửa: thêm AD-54, AD-56, AD-57 vào C9.

### L3 (low) Chi tiết di trú
- AD-54 Rule 2: `ADD COLUMN` kèm `CHECK` tham chiếu cột khác, và trigger thay thân ở "cả hai thang", phải kiểm với bậc dựng lại trigger đang có (`schema.rs:328`); nêu số bậc kế (project 35, global 14) hoặc "bậc kế sau bậc mới nhất khi dựng" để 12.14/12.15/12.19/12.20 không tranh số (tickets chỉ nói thứ tự).
- AD-55 Rule 1: trigger `BEFORE INSERT` abort khi `NULL` phải được tạo sau mọi lượt chép hàng trong bậc (nếu bậc dựng lại bảng).
- AD-56 Rule 1: `id INTEGER PRIMARY KEY` tái dùng id sau khi xoá hàng lớn nhất; `decision_id` giữ ở webview nên dùng `AUTOINCREMENT` hoặc ghi lý do không cần.

## Đã kiểm, không phát hiện
- AD-1/AD-3: không văn bản người dùng, không `segment_id` trong `proof_decision`; webview chỉ giữ `decision_id`, mục vừa chốt, và không gửi `baseline_*`.
- AD-13: hàm đếm thuần trong `core/ai/proofread.rs`, `commands/segment` không import `core::ai` như AD-53 mục 1 yêu cầu.
- AD-18/AD-47 ⑥: `tm_only_mine` không phải loại bảng ngữ nghĩa, dùng `side()` có sẵn (`tm/mod.rs`, `bilingual_import` ⇒ người khác); 🔵 tại AD-18 khớp AD-57.
- AD-20/AD-36: ứng viên không lùi, truy vấn chèn không đổi; 🔵 AD-36 khớp AD-54.
- AD-49 (i), AD-47 ⑤, AD-50 mục 2, AD-53 mục 3 và 7: các 🔵 khớp nội dung AD mới.
- AD-21: phần trăm do webview định dạng; mã lỗi có kiểu (`glossary.*`, `ai_proofread.decision_missing`).
- NULL không phải 0: `pending_confirmed_at`, `segment_version.translation_origin`, `current_version_id` đều `NULL`/`None` khi không rõ; tổng quyết định 0 hiện *chưa có quyết định*. `tm_only_mine DEFAULT 0` là cờ boolean, không phải giá trị chưa biết. Tên `origin` trần không xuất hiện thành định danh.
- FR đã phủ: FR141 (nhãn + diff + đang dùng, trừ H3), FR147, FR149 (cờ + xuất riêng, trừ H1), FR150 (ba số + mẫu số).
