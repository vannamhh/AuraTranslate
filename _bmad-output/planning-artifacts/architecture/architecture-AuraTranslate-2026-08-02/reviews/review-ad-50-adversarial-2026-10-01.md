# Lens đối kháng — AD-50 (mốc so xuất xứ lưu trên `segment`)

**Phương pháp:** dựng hai đơn vị tầng dưới, mỗi đơn vị tuân AD-50 đúng chữ, tìm chỗ chúng vẫn không khớp. Đọc cả mã hiện tại (`commands/segment.rs`, `core/segment/regroup.rs`, `core/store/schema.rs`, `commands/project/mod.rs`). Không re-litigate D1 (mốc phía Rust), hai cột, hàm phân xử đơn, không-IPC.

**Verdict: 5 lỗ (2 cao, 2 trung, 1 thấp). Mệnh đề "phân xử không bao giờ sinh lời nói dối đắt cho hàng sau di trú" là SAI: khôi phục (FR101) tạo được nó.**

Kiểm xong, KHÔNG có lỗ: xác nhận segment đã xác nhận (`Ok(false)` trước phân xử, không ghi); văn bản rỗng/khoảng trắng (nhánh `trim().is_empty()` loại trước, rule 3 trả `''`); nhánh `needs_confirmation` của promote-AI (không ghi, không chạm mốc); segment về hưu (`confirm`/`restore`/`promote` đều từ chối `retired_at`; gộp/tách chỉ ghi `retired_at`, mốc giữ nguyên); bước 28 không có trigger nào trên `segment` nên `UPDATE` hàng loạt không làm nhảy `updated_at` (`meta.rs:325` đọc `MAX(updated_at)`).

---

## F1 — CAO — Khôi phục để `baseline_translation_origin` cũ ⇒ lời nói dối ĐẮT, đến được sau di trú, rồi lan sang gộp

**Hai đơn vị:** A = Story 7.4 (điền sẵn TM, ghi `(văn bản cặp, xuất xứ cặp)`); B = Story 2.x/FR101 `restore_segment_version` theo AD-50 mục 2 (ghi `baseline_target_text`, không chạm `baseline_translation_origin`). Cả hai đúng chữ.

**Kịch bản (một segment):** (1) promote-AI văn bản A ⇒ mốc `(A, other)`; ký ⇒ `v1 = A` (`segment_version` không mang xuất xứ). (2) Người dùng xoá sạch, flush; TM 100% từ cặp *của tôi* điền `T1` ⇒ mốc `(T1, self)`. (3) Khôi phục `v1` (bấm đồng ý hộp thoại `needs_confirmation`) ⇒ `baseline_target_text = A`, `baseline_translation_origin = self` (không chạm). (4) Ký không sửa ⇒ rule 3: `A == baseline_text`, origin `self` ⇒ **cặp TM `A` mang `self`** = chữ của máy gắn nhãn của tôi, đúng lớp hỏng FR117 chống (AD-18 xếp nó lên đầu `RagInjector`).

**Cùng ca, không cần TM:** hàng di trú (bước 28) có `translation_origin = self` và lịch sử chứa `v1 = A` ký `other` ⇒ mốc `(T2, self)`; khôi phục `v1` ⇒ `(A, self)` ⇒ ký ⇒ `self`. Đây là ca thường nhật.

**Lan:** khôi phục xong rồi gộp: hàm phân xử của mảnh trả `baseline_origin = self` cho văn bản của máy ⇒ segment mới mang mốc `(…, self)` vĩnh viễn (AD-47 ④ đọc "giá trị mảnh" qua hàm).

AD-47 ⑤ chỉ thừa nhận chiều "giữ xuất xứ **hiện tại**, có thể thuộc phiên bản khác" mà không nói chiều này là chiều ĐẮT; mục 5 của AD-50 không liệt nó.

**Hai lựa chọn hợp lệ (trình Ice kèm số đo, không chọn hộ):**
- **(A)** Khôi phục ghi `baseline_translation_origin = 'other'` (chiều rẻ, đúng luật AD-47 ④ "khi không có giá trị đúng thì chọn chiều rẻ"). Giá: khôi phục chữ chính mình rồi ký không sửa ⇒ nhãn `other` (một cặp TM xếp sau); gõ một ký tự ⇒ `self`. Không đổi schema.
- **(B)** `segment_version` thêm cột `translation_origin` (bước di trú 29, backfill `''` cho phiên bản cũ ⇒ rơi về `other`), khôi phục chép từ phiên bản. Đúng nhất; thêm một bước di trú cho mọi `.atproj` (AD-47 ⑥ đã nêu giá này).

**Rule đề xuất (thay dòng "Khôi phục" ở bảng mục 2), theo phương án A:**
> Khôi phục phiên bản (FR101): `baseline_target_text` = văn bản phiên bản; `baseline_translation_origin` = `'other'`, **trừ khi** nhánh *"đã đúng nội dung đó"* (không ghi byte nào) — nhánh đó không chạm mốc. Không lượt nào ngoài bảng này được để một cột mốc mang giá trị do lượt ghi **khác** đặt cho văn bản khác.

Và sửa mệnh đề đầu: *"không bao giờ sinh `self` cho chữ của người khác"* chỉ được viết sau khi F1 đóng; cho tới đó phải ghi "trừ khôi phục".

---

## F2 — CAO — Bước 28: danh sách "cái mất" thiếu ca thứ ba (chiều ĐẮT), và `DEFAULT ''` làm một chỗ ghi quên mốc thành `self` im lặng

**2a. Ca thiếu.** Hàng trước di trú: ký `v1 = A` (AI, `other`), sửa `T2`, ký ⇒ `translation_origin = self`; sau đó **khôi phục `v1`** (khôi phục không chạm xuất xứ, hạ về `draft`) ⇒ hàng `draft`, `target_text = A`, `translation_origin = self`. Bước 28 chụp `(A, self)` ⇒ ký không sửa ⇒ `self` cho chữ của máy. Mục 5 chỉ ghi hai ca (viết lại chưa ký ⇒ `other`; ký rồi sửa ngược ⇒ `self`), và ca thứ ba cùng chiều với ca thứ hai nhưng không cần người dùng sửa ngược thủ công.

**Hai lựa chọn:** (A) chấp nhận và ghi ra; (B) backfill thận trọng: hàng `status='draft' AND translation_origin='self' AND` văn bản trùng một `segment_version` cũ KHÔNG phải mới nhất ⇒ `baseline_translation_origin = 'other'`. Phải đo số hàng khớp trên một `.atproj` thật trước khi chọn.

**Rule đề xuất (mục 5, thêm câu):** *"⚠️ Cái mất thứ ba: hàng đã khôi phục về một phiên bản cũ trước bước 28 mà còn `translation_origin = self` ra `self` cho văn bản của phiên bản đó."* (nếu chọn A) hoặc câu `UPDATE` điều kiện ở (B).

**2b. `DEFAULT ''` + không có seam.** `baseline_target_text = ''` và `baseline_translation_origin = ''` đều hợp lệ cho segment gõ từ đầu, nên một `INSERT INTO segment (… target_text …)` hay `UPDATE … SET target_text` của story sau quên hai cột mốc **không đỏ ở đâu cả**: mốc `''` ≠ văn bản ⇒ hàm trả `self` mãi mãi (đúng chiều đắt). Điểm ghi hiện có: `commands/segment.rs:145`, `:194`, `:862` (khôi phục), `:2197` (promote), `:3241` (gộp/tách, `INSERT` thẳng từ `RegroupRow`). AGENTS.md cấm cái giá trị "chưa biết" làm `''`/`0`; mục 5 "không NULL" mâu thuẫn tinh thần đó, nhưng `ADD COLUMN NOT NULL` trong SQLite buộc có `DEFAULT` nên không sửa được ở schema ⇒ phải sửa ở cổng.

**Rule đề xuất (mục 2, thêm):**
> Mọi câu `INSERT INTO segment` có cột `target_text`, và mọi `UPDATE segment SET … target_text …` **ngoài flush (AD-35)**, phải nêu tên `baseline_target_text` trong chính câu đó. Một cổng đọc dòng mã (không dòng chú thích) trong `src-tauri/src/` thi hành điều này, và danh sách ngoại lệ gồm đúng một mục, flush, kèm lý do.

---

## F3 — TRUNG — Không có seam ghi duy nhất cho cặp mốc; "cùng câu lệnh" không phủ `translation_origin`/`status`; giá trị ngoài tập FR117 vào được `baseline_translation_origin`

**Hai đơn vị:** Epic 8 (FR94 chấp nhận từ Review Mode) và Story 7.4 (điền sẵn TM). Mỗi bên tự viết một `UPDATE`. Mục 2 buộc hai cột mốc cùng câu với `target_text` nhưng KHÔNG buộc `translation_origin` (AD-47 ①(b)) và `status` (AD-31: FR94/FR58 ⇒ chưa xác nhận) vào cùng câu; một bên ghi `translation_origin` ở câu thứ hai trong cùng hoặc khác giao dịch (mã hiện tại có tiền lệ: `lib.rs:967` phải dặn "MỘT câu"). Khôi phục lại là ngoại lệ ghi một nửa cặp, nên bên thứ ba (một "undo" tương lai) thấy tiền lệ "ghi một cột mốc" là hợp lệ.

**Tập giá trị:** mục 1 nói "cùng tập giá trị" nhưng chỉ `translation_origin` được Rust cưỡng chế lúc mở (`reject_unknown_translation_origin` chỉ quét cột đó) và lúc ghi (`PairOrigin`). Một writer nhận `&str` (như `promote_ai_translation` đang nhận hằng `&str`) ghi giá trị lạ vào `baseline_translation_origin` ⇒ `PairOrigin::from_stored` ở `confirm_segment` từ chối `UnknownOrigin` mãi mãi cho segment đó (kẹt, người dùng chỉ thoát bằng cách sửa chữ) — lỗi to nhưng đến sau, xa chỗ gây.

**Rule đề xuất (mục 2, thêm hai câu):**
> Chỉ **một** hàm Rust `write_non_user_target(tx, segment_id, text, origin: PairOrigin | None, status)` ghi `target_text` + hai cột mốc + `translation_origin` + `status` trong **một** câu `UPDATE`; mọi hàng "đúng giá trị vừa ghi" của bảng gọi nó. `INSERT` của nhập song ngữ và gộp/tách dùng `INSERT` một câu tương đương, nêu đủ bốn cột. Tham số xuất xứ có kiểu `PairOrigin` (hoặc `''` có tên), không `&str`.
>
> `reject_unknown_translation_origin` quét **cả** `baseline_translation_origin`; một giá trị ngoài `TRANSLATION_ORIGINS` ở cột nào cũng khiến `Store::open` từ chối.

---

## F4 — TRUNG — Hàm phân xử "duy nhất" chưa có chủ sở hữu mô-đun; hai nơi vẫn tự cài `trim + NFC`; mảnh rỗng bỏ phiếu thành "bất đồng"

**Hai đơn vị:** Story 7.2 (`confirm_segment`, ở `commands/`) và một story gộp/tách (`core/segment/regroup.rs`, thuần, không có `tx`). Hôm nay `confirm_segment` tự cài phép so `trim().nfc()` và `merged_origin` đọc `translation_origin` lưu sẵn. AD-50 mục 3 nói "một hàm" nhưng không nói nó ở đâu; `regroup.rs` thuần không gọi được hàm nhận `Transaction`, nên một bên sẽ cài lại hoặc gọi bản đọc `translation_origin` (đúng thứ AD-50 loại bỏ). Mỗi bên đúng chữ.

**Bất đồng giả:** tách một segment `self` ⇒ mảnh đầu `self`, các mảnh sau `''` (đúng như `regroup.rs` làm, và theo rule 3: rỗng ⇒ `''`); gộp ngược lại ⇒ `{self, ''}` ⇒ `other` cho chữ của chính người dùng. AD-47 ④ đã ghi "gộp `''` với `self`" là cái mất, nhưng tách rồi gộp lại là thao tác hoàn tác hằng ngày, không phải ca biên. (Chiều rẻ; không phải lời nói dối đắt.)

**Rule đề xuất (mục 3, thêm):**
> Hàm là `fn arbitrate(target: &str, baseline_text: &str, baseline_origin: &str) -> PairOrigin|''` **thuần**, ở `core/segment/`; `confirm_segment` và gộp/tách đều gọi nó; không nơi nào khác trong `src-tauri/src/` chứa phép `.nfc()` so với một mốc (cổng đếm). Khi tính "giá trị của mảnh" cho gộp (AD-47 ④), mảnh có kết quả `''` **không bỏ phiếu**; chỉ khi mọi mảnh đều `''` kết quả mới là `''`.

(Bỏ phiếu cho mảnh rỗng sửa AD-47 ④ "cái mất"; nếu Ice muốn giữ nguyên AD-47 ④ thì giữ nguyên câu đầu của đề xuất và bỏ câu sau.)

---

## F5 — THẤP — "Không qua IPC" mới đúng nhờ danh sách cột viết tay; không cổng nào giữ nó, và `translation_origin` vẫn đi qua dây với chú thích cũ

**Kiểm mã:** không có `SELECT *` nào trong `src-tauri/src/`; ba câu `SELECT` trả `ChapterSegment` (`segment.rs:925`, `:3313`, và đường đọc khác) đều liệt kê cột; `ChapterSegment` không derive từ hàng. ⇒ hôm nay hai cột mốc KHÔNG lộ.

**Hai đơn vị gây lỗ:** story thêm `baseline_*` vào `ChapterSegment` "để gỡ lỗi / để dựng UI chỉ báo *đã sửa*" (Review Mode, Epic 8); hoặc một `#[derive(FromRow)]`/`SELECT *` cho một struct mới `Serialize`. Mục 4 cấm bằng chữ, không cổng.

**Chú thích còn sống:** `ChapterSegment.translation_origin` (`segment.rs:296-299`) ghi "mốc mà `confirm_segment` cần làm `origin_at_load`: webview giữ nguyên giá trị này" — sai sau D1, và mục 4 không nói cột đó có còn qua dây không. Nếu còn, nó nay là xuất xứ của **lượt ký/lượt ghi gần nhất**, không phải mốc; một màn hình đọc nó như "ai dịch câu hiện tại" sẽ hiện `other` cho bản người dùng đã viết lại chưa ký.

**Rule đề xuất (mục 4, thêm):**
> Không struct `Serialize` nào trong `src-tauri/src/commands/` mang trường tên bắt đầu bằng `baseline_`; không câu SQL nào dùng `SELECT *` trên `segment`. Cổng đọc dòng mã thi hành cả hai. `translation_origin` đi qua dây chỉ để **hiển thị** và webview không gửi lại nó; chú thích "origin_at_load" bị xoá cùng Story 7.2.

---

## Hàng đợi việc tách khỏi lỗ

- Mục 2 dòng khôi phục: nhánh *"đã đúng nội dung đó"* (`restore` bước ④, không ghi byte nào) phải ghi rõ **không** chạm mốc; nếu không, một cách cài ghi `baseline_target_text = version_text` ở nhánh đó biến chữ tự gõ đã ký thành `other` (rẻ, nhưng phá "không ghi byte nào"). Đã gộp vào rule F1.
- Người ghi không-phải-người-dùng tương lai (một "AI sửa hàng loạt", nhập TM): luật "danh mục ĐÓNG" của AD-47 ③ đã phủ; thêm câu "writer mới phải gọi `write_non_user_target`" (F3) là đủ.
- Hồ sơ phản chứng nên có (`AGENTS.md` §Tests): gỡ thật lời gọi `arbitrate` khỏi `confirm_segment`, gỡ thật cột mốc khỏi `INSERT` gộp/tách, và ca "khôi phục `v1` AI rồi ký không sửa" phải đỏ, theo thứ tự đó.
