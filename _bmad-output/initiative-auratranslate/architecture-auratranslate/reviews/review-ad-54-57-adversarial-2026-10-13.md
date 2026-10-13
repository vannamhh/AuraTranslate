# Rà soát đối kháng AD-54 · AD-55 · AD-56 · AD-57 (2026-10-13)

Phạm vi: spine dòng ~990–1110 và các AD nó dựa vào (AD-3, 5, 18, 20, 31, 35, 36, 47, 49, 50, 51, 53). Mã đã đọc: `core/glossary/store.rs` (`confirm_translation`, `update_manual_term`, `promote_to_global`, `import_into_tier`, `confirmed_terms_for_injection`), `core/glossary/candidate_store.rs` (duyệt ứng viên), `core/store/schema.rs` (`GLOSSARY_ENTRY_DDL` và các bậc dựng lại bảng), `commands/segment/confirm.rs`, `commands/segment/history.rs`, `commands/segment/chapter_read.rs::fill_exact_tm_matches`, `core/tm/mod.rs`, `core/ai/rag.rs`, `src/glossaryConfirmStripState.ts`.

Giới hạn: không chạy gì; mọi kịch bản dưới đây suy từ đọc mã. Chỗ nào chưa đọc hết ghi rõ "chưa kiểm".

Phán quyết: bốn AD khoá đúng điều chúng gọi tên nhưng mỗi cái còn ít nhất một cặp "tuân từng chữ mà dựng lệch nhau"; nặng nhất là AD-54 (danh sách người ghi `translation` thiếu hai đường có thật, nên trigger mới biến sơ suất thành lỗi lúc chạy) và AD-57 (nó làm chiều "xuất xứ sai về *người khác dịch* là chiều rẻ" của AD-47/53/55 hết rẻ, và bỏ trống ba chỗ im lặng).

---

## HIGH

### H1. AD-54 — người ghi `translation` bị liệt kê thiếu; trigger biến chỗ thiếu thành ABORT thô

Rule 3 chỉ kể "sửa thủ công, thu hoạch review". Mã thật có thêm các đường đổi `translation` của một hàng có thể đang mang `pending_confirmed_at`:

| Đường | Mã | Chuyện gì xảy ra khi hàng mang `pending_confirmed_at != NULL` |
|---|---|---|
| Duyệt ứng viên `ReviewHarvest` vào mục có sẵn | `candidate_store.rs:376` `UPDATE glossary_entry SET translation = ?1, term_origin = ?2` | Không đặt cột về `NULL` ⇒ trigger mệnh đề 2 (`đổi sang chữ khác mà NEW.pending IS NOT NULL`) ABORT. Cả giao dịch duyệt lăn lại, ứng viên không rời bảng chờ, người dùng nhận lỗi SQLite thô. |
| Nhập tệp, `TakeTheirs` | `store.rs:2028` `SET translation = ?1 WHERE id AND translation IS ?3` | Cùng ABORT, mà nằm trong **một** `store.write` của cả tệp (`import_into_tier`): một hàng dính cột mới làm hỏng cả lượt nhập. Nhánh `changed == 0` đã được đọc kỹ để phân biệt hai nghĩa, không có nhánh nào cho lỗi trigger. |
| `confirm_translation` chạy lên mục đã chốt | `store.rs:266`, doc-comment nói rõ "dùng được ở CẢ HAI chiều", không có `WHERE translation IS NULL` | Hai cửa sổ hay bấm đúp: lượt thứ hai trên hàng đã `pending != NULL` với chữ khác ⇒ ABORT thô (mệnh đề 2); cùng chữ ⇒ lặng lẽ đổi mốc thời gian. Mục `pending = NULL` đã chốt từ trước ⇒ mệnh đề 3 ABORT. Rule 3 nói "chỉ `confirm_translation` đặt cột" mà không nói nó phải *từ chối* khi hàng không ở trạng thái chờ chốt. |

Hai bản dựng đều tuân AD: (A) mỗi người ghi tự đặt `pending = NULL` (đúng chữ Rule 3), (B) chỉ đặt khi chữ đổi. Chúng khác nhau ở `update_manual_term` sửa mỗi `note`: câu SQL hiện tại luôn `SET translation = ?1, note = ?2, category = ?3` (`store.rs:1045`). Bản A xoá đường gỡ chốt chỉ vì người dùng sửa ghi chú, trái AGENTS.md "a write touches exactly the columns the user agreed to change"; bản B giữ. "Đổi `translation`" trong Rule 3 không phân xử.

Cổng không bắt được: `GLOSSARY_ONLY_SURFACE`/`glossary_boundary.rs` canh ai được gọi hàm, không canh câu SQL nào chạm cột. Trigger chỉ bắt khi dữ liệu rơi đúng ca; test của từng đường (không có `pending`) vẫn xanh.

Siết AD-54:
1. Chuyển bất biến từ "mỗi người ghi nhớ xoá cột" sang **trigger**: thêm `AFTER UPDATE OF translation` tự đặt `pending_confirmed_at = NULL` khi `NEW.translation IS NOT OLD.translation` và cột `pending_confirmed_at` không đổi trong chính câu lệnh đó. Chỉ `confirm_translation` (đổi cả hai cột cùng lúc, `OLD.translation IS NULL`) và `unconfirm` được đổi cột. Khi đó không người ghi nào phải nhớ, và nhập tệp/duyệt ứng viên không bao giờ ABORT.
2. Câu `UPDATE` của `confirm_translation` phải có `WHERE id = ?2 AND translation IS NULL`; 0 hàng ⇒ lỗi có kiểu (ví dụ `glossary.already_confirmed`), không đi vào trigger.
3. Nói rõ "đổi" = `NEW.translation IS NOT OLD.translation`; `update_manual_term` sửa mỗi `note`/`category` không được chạm cột.
4. Thêm một test cho **mỗi** người ghi `translation` liệt kê trong AD (kể cả 3 đường trên) trên hàng đã có `pending_confirmed_at`.

### H2. AD-57 phá tiền đề "chiều rẻ" mà AD-47 ④, AD-53, AD-55 đang dựa vào

AD-47 ④/AD-53/AD-55 nhiều lần chọn "*người khác dịch* khi không biết" vì hậu quả chỉ là *hạ hạng* trong TM. Với `tm_only_mine = 1`, cặp `Others` bị **loại**, nên cùng quyết định đó thành: câu của chính người dùng biến mất khỏi điền sẵn, dải khớp mờ và RAG. Các đường đang chọn `Other` theo mặc định:

- AD-55 mục 3: phiên bản `NULL` ⇒ khôi phục ra `other`. Mọi phiên bản có trước bậc di trú đều `NULL` (kể cả bản người dùng tự gõ). Khôi phục rồi ký không sửa ⇒ cặp TM mới `other` ⇒ MineOnly không thấy.
- AD-53 mục 3: loại của 9.2 (đối chiếu) và hoàn lại ⇒ luôn `other`, kể cả câu cũ là của người dùng.
- FR94 (ghi rời hạ xuống *người khác dịch*).
- `PairOrigin::side()` (`core/tm/mod.rs:68`): `BilingualImport` thuộc `Others`. Người dùng nhập bản song ngữ **do chính mình dịch** (FR115) rồi bật "chỉ cặp của tôi" thì cả kho đó bị loại. Tên cờ ("của tôi") và phép chiếu (`side`) nói hai chuyện; AD-57 Prevents 4 chỉ cãi lại phép định nghĩa theo Tác phẩm nguồn, không cãi lại điều này.

Hai bản dựng đều tuân AD: 12.19 giữ "NULL ⇒ other, chiều rẻ", 12.14 giữ `Others ⇒ loại`. Kết quả: người dùng bật cờ, mở lại một chương cũ, và không thấy cặp nào của mình, không có lỗi.

Siết: AD-57 phải có mục *"Cái mất, ghi ra"* nêu rằng AD-47 ④ hết đúng ở mọi nơi nó viện "chiều rẻ", và chọn một trong hai (theo AGENTS.md, đưa cả hai kèm số đo cho Ice, không tự chọn):
- (a) thêm giá trị *không rõ* cho phiên bản `NULL` và `MineOnly` coi *không rõ* là của tôi (cần sửa AD-47 ⑥, tức một AD mới + bước di trú);
- (b) giữ nguyên tập, nhưng nhãn UI nêu "kể cả bản nhập song ngữ" và đo số cặp bị loại trên 47 `.atproj` thử (số đo hiện chưa có; dữ liệu thật chỉ là bản thử).
Dù chọn nào, AD-55 mục 3 phải ghi rằng "chiều rẻ" nay có chi phí khi cờ bật.

### H3. AD-57 — rỗng im lặng, ảnh chụp lệch nhau, hai chỗ gắn chính sách

1. **Không đếm cái bị giấu.** MineOnly không có cặp ⇒ điền sẵn không điền, dải khớp mờ rỗng, RAG không có tham khảo; không trường nào nói "N cặp người khác bị giấu". `tm_prefill` đã có `Ran` với 0 điền (`chapter_read.rs:~502`) không phân biệt "TM không có" với "cờ giấu hết". Đây đúng lớp lỗi trung tâm của AGENTS.md (0 hàng, không lỗi). Siết: hàm lọc trả `(cặp giữ lại, số cặp bị giấu)`; mọi bề mặt hiển thị phải mang số này (webview và `ledger` của prompt, cùng khuôn `TmInjectionStatusWire`).
2. **Hai chỗ lọc hợp lệ.** Rule 3 gắn chính sách vào `read_tier_rows`, `pairs_for_source` **và** `merge_tagged_tiers`. Nhưng `fuzzy_pairs_in_candidates` và `rank_concordance` nhận `TierRows` đã đọc, không nhận chính sách (`rag.rs:281/302`: RAG đọc một lần ở `load_tm_rows`, gọi `gather_tm_context` cho nhiều câu). Bản dựng A lọc trong `read_tier_rows` (ảnh chụp đã lọc); bản B lọc trong `merge_tagged_tiers` (ảnh chụp đầy đủ + chính sách theo lượt gọi). Cả hai biên dịch và tuân "tham số bắt buộc". Hậu quả: một `TierRows` đọc bằng `MineFirst` cho concordance (được miễn) có thể đi vào `fuzzy_pairs_in_candidates` của đường RAG vì kiểu giống hệt, và không có gì ghi chính sách đã dùng vào ảnh chụp. Siết: chính sách là **thuộc tính của kiểu** `TierRows` (hoặc `TierRows<Policy>`), hàm lọc nằm đúng một chỗ (`read_tier_rows`), `merge_tagged_tiers` không nhận tham số này.
3. **Cờ đổi giữa lô.** "Cùng lượt khoá với lượt đọc TM" chỉ định nghĩa cho một lượt đọc. Dịch lô gọi `load_tm_rows` một lần ở đầu lô, còn điền sẵn đọc theo từng câu: bật cờ giữa chừng, nửa lô dùng cặp người khác, nửa sau thì không, không dấu vết. Siết: chính sách đọc **một lần ở đầu lô/lượt** và truyền xuống; lô ghi chính sách đã dùng vào ledger.
4. **Điền sẵn đã lưu.** `fill_exact_tm_matches` ghi `target_text` thật (`write_non_user_target`). Bật cờ *sau* khi chương đã được điền bằng cặp `Others` thì chữ đó ở lại trong segment, vì "cờ chỉ đổi đường đọc" (Rule 5). Có thể chấp nhận, nhưng AD phải nói rõ để UI không hứa "bật cờ ⇒ không còn bản người khác" (Rule 5 chỉ nói chiều tắt).

---

## MEDIUM

### M1. AD-54 — vòng đời nút "gỡ chốt" và dải chốt không được khai

- `expected_translation` so theo **chữ**, không theo **lượt chốt**. Chốt "A" (t1) → gỡ → chốt lại "A" (t2, cố ý, có thể ở cửa sổ khác) → bấm nút còn sót của t1: khớp chữ, `pending != NULL`, thành công, và gỡ lượt t2 người dùng vừa chủ ý. Siết: tham số thứ ba `expected_pending_confirmed_at`, so cả hai.
- `(tier, id)` là khoá trong **một kho**; `id` của `project.db` lặp lại giữa các Tác phẩm. Webview "nhớ mục vừa chốt để vẽ nút" không có luật sống/chết. Đóng Tác phẩm A, mở B có mục `id` trùng, cùng chữ, cùng `pending` ⇒ gỡ nhầm (hiếm, nhưng bất khả kiểm bằng cổng). Siết: AD khai nút chết khi đóng Tác phẩm, đổi Chương, hay sau lượt ghi Glossary khác; `unconfirm` cho tầng Work chỉ chạy qua `OpenWorkState`.
- Gỡ chốt xong, mục thành *chờ chốt* ngay trong chương đang mở; `selectPendingSpan` (`glossaryConfirmStripState.ts:~150`) chọn span chờ chốt trái nhất chưa "Để sau" ⇒ dải chốt mọc lại tức thì, lấy tiêu điểm (`focusRequest`), điền sẵn Hán Việt: một cú Enter chốt lại. AD-54 Prevents 5 lo `⌘Z` mà bỏ sót việc này. Siết: gỡ chốt thêm term vào tập `deferred` của dải.
- Dấu hiệu trên trình soạn (`glossary_marks_for_chapter`) nạp một lần; AD không nói ai làm mới sau gỡ chốt. Rule 5 chỉ bảo đảm phía prompt "đọc sống".
- `promote_to_global` chèn hàng Global bằng `insert_entry_row` ⇒ `pending = NULL` (đúng Rule 3 "mọi INSERT để NULL") và xoá hàng Work. Chốt ở Work → đẩy lên Global → nút gỡ chốt báo `glossary.not_found`. Không hỏng dữ liệu nhưng lỗi sai nghĩa; nên ghi vào "Cái mất".

### M2. AD-55 — `current_version_id` và khôi phục có thể trỏ hai phiên bản khác nhau

Rule 5: "id lớn nhất trong các phiên bản có `target_text` bằng byte với văn bản hiện tại". Rule 3: khôi phục lấy xuất xứ của phiên bản **người dùng chọn**. Kịch bản: v1 "A" (`other`), v2 "B", v3 "A" (`self`). Hiện tại là "B". Người dùng bấm khôi phục v1 ⇒ `target_text = "A"`, mốc xuất xứ `other`. Overlay vẽ "đang dùng" ở v3 (id lớn nhất, chữ "A"), mà ký ngay ra `other` (của v1). Nhãn "đang dùng" và xuất xứ sẽ ký nói về hai phiên bản khác nhau. (Bước ④ trong `restore_segment_version`: nếu hiện tại *đã* bằng chữ của v1 thì không ghi gì, nên nhấn khôi phục v1 khi đang dùng chữ "A" không đổi xuất xứ; hành vi vẫn nhất quán nhưng chưa được AD nói.)
Ngoài ra "đang dùng" chỉ là phép so chữ: bản nháp chưa ký có chữ trùng một phiên bản vẫn sáng hàng đó, trong khi `segment.status = draft`.
Siết: khôi phục ghi nhớ id phiên bản vào mốc (hoặc trả `current_version_id` bằng "phiên bản gần nhất mà lượt ký/khôi phục cuối chọn"), hoặc AD khai rõ "trùng chữ ⇒ lấy id lớn nhất, xuất xứ khi ký có thể khác nhãn" và nhãn chỉ nói về chữ.
Kèm: trigger `BEFORE INSERT ... NULL ⇒ ABORT` chặn mọi `INSERT` `segment_version` không có cột mới; hiện chỉ `confirm.rs:279` chèn, nhưng fixture test và mọi đường nhập/sao chép `.atproj` sau này phải truyền cột. Chưa kiểm: có đường nhập `.atproj`/sao chép Tác phẩm nào chèn trực tiếp không.
Với AD-5: gộp/tách là về hưu + tạo mới, phiên bản gắn `segment_id` cũ nên lịch sử của segment mới rỗng; không do AD-55 gây ra nhưng AD-55 vẽ lịch sử có xuất xứ, nên nên ghi "segment sau gộp/tách bắt đầu lịch sử trống" để 12.19 không tự "sửa".

### M3. AD-56 — hạt quyết định, đếm trùng, và `decision_id` lỏng

- Một phát hiện trải qua nhiều quyết định: bỏ qua rồi quét lại rồi bỏ qua nữa (mỗi lần một hàng `skipped`); bỏ qua rồi sau đó chấp nhận (cả `skipped` và `accepted`); *không phải lỗi* cho cụm đã vào `proof_ignore` mà danh sách đang mở chưa lọc (mỗi lần bấm một hàng `not_error`, `INSERT OR IGNORE` thì không). Rule 3 gọi mẫu số là "số quyết định" nên đúng chữ, nhưng nhãn FR150 "ba phần cộng 100%" sẽ bị người dùng đọc là "% phát hiện". Prevents 4 chỉ phân biệt với FR81, chưa phân biệt với "phát hiện". Siết: AD khai rõ "một phát hiện có thể đóng góp nhiều hàng" hoặc khoá `(kind, chữ ký băm?)`... (khoá theo chữ ký sẽ chép văn bản người dùng, đã bị loại; vậy chấp nhận trùng và đổi nhãn UI thành "quyết định").
- `proof_decision.id INTEGER PRIMARY KEY` không `AUTOINCREMENT` ⇒ xoá hàng lớn nhất (qua hoàn lại) rồi chèn lại tái dùng id. `ai_proofread_revert(decision_id)` xoá "đúng hàng *accepted* đó" nhưng AD không buộc kiểm hàng thuộc lượt chấp nhận đang hoàn. Hiện phép kiểm văn bản mục 4 (chạy trước) chặn đúp-bấm, nên chưa thủng; chỉ cần thứ tự "xoá hàng trước, kiểm sau" ở một bản dựng là mở. Siết: `AUTOINCREMENT`, xoá `WHERE id = ? AND outcome = 'accepted' AND kind = ?`, và nói rõ phép kiểm văn bản đứng *trước* việc xoá.
- Ngăn hoàn lại mất khi đổi Chương (AD-53 mục 5) ⇒ hàng `accepted` ở lại dù người dùng sau đó tự gõ lại chữ gốc. AD đã nhận ("lượt chấp nhận đứng") nhưng nên ghi chung vào "Cái mất" của AD-56 vì nó làm tỉ lệ *chấp nhận* phồng lên theo thói quen đổi Chương.
- `ai_proofread_revert` với 0 hàng ⇒ `decision_missing` *không ghi gì*, gồm cả không hoàn văn bản. Nếu một đường dọn (ví dụ "xoá ghi nhớ" của 9.6) từng xoá `proof_decision`, người dùng bị kẹt không hoàn được chữ. AD nói "không đường nào khác xoá"; thêm cổng canh câu đó vì nó là điều kiện sống còn của lệnh hoàn lại.

### M4. Thứ tự di trú giữa bốn story và hai thang

Bốn AD thêm bậc: AD-54 một bậc ở `GLOBAL_MIGRATIONS` **và** một ở `PROJECT_MIGRATIONS`, AD-55 + AD-56 + AD-57 ba bậc ở `PROJECT_MIGRATIONS`. Thang là theo vị trí (xem "bước 16 MỚI", "mười bốn bước" trong `schema.rs`) và có test đếm bậc. 12.14, 12.15, 12.19, 12.20 chạy song song trên nhánh khác nhau sẽ cùng nhận "bước N+1"; AD không cấp số bậc, không nói thứ tự áp (và AD-54 phải chạy cùng một hằng DDL ở hai thang với số bậc khác nhau). Siết: AD (hay plan epic) gán số bậc cố định cho từng story; hoặc nêu luật "append theo thứ tự hợp nhập, story sau rebase số" kèm test đếm.
Thêm: các bậc dựng lại bảng (`GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL`, schema.rs:~520) sao chép bằng danh sách cột tường minh `INSERT INTO glossary_entry_new (id, source_term, ...)`; bậc dựng lại *sau* AD-54 mà quên `pending_confirmed_at` sẽ xoá im lặng toàn bộ giá trị cột ("mọi hàng NULL ⇒ không gỡ chốt được", không lỗi). AD-54 cần một test so `PRAGMA table_info` + `sqlite_master` của hai kho với hằng DDL, và dòng "bậc dựng lại bảng phải sao chép cột này".
Thêm: AD-54 mục 4 "thân mới thay thân cũ" của `glossary_entry_lifecycle_is_one_way`; hai bậc dựng lại cũ (`schema.rs:328`, `:561`) tạo trigger bằng thân cũ. Nếu một bậc dựng lại cũ chạy lại trên kho chưa có cột (thứ tự áp trên kho mới tinh: bậc cũ trước, bậc AD-54 sau) thì đúng; nếu bậc dựng lại mới hơn được thêm sau AD-54 mà chép thân cũ thì mất trigger mới. Cùng test trên bắt được.

---

## LOW

- L1. AD-54 mục 4 đòi `RAISE(ABORT)` nhưng lệnh gỡ chốt kiểm theo thứ tự `not_found → translation_changed → not_unconfirmable`; ngoài lệnh đó mọi lỗi trigger ra dưới dạng lỗi kho thô. Cần ánh xạ lỗi trigger → lỗi có kiểu ở `confirm_translation`.
- L2. AD-57 Rule 6 `tm_export_tier` lọc `mine` "bằng cùng hàm" nhưng `tmx.rs:668` đọc `load_all_pair_rows` riêng (không qua `read_tier_rows`); xác nhận xuất qua một đường duy nhất, không thêm lối thứ ba.
- L3. AD-57 Rule 2 "không nhận work id qua IPC" nhưng cờ cũng bị đọc bởi `commands/aitranslate.rs` chạy nền: nếu Tác phẩm bị đóng giữa lô, "không Tác phẩm nào mở ⇒ `MineFirst`" làm nửa lô sau đổi chính sách mà không báo (cùng gốc H3.3).
- L4. AD-56 Rule 2 "skipped" thành command mới không chạm segment: không có phép kiểm `kind` theo phát hiện thật nào (server không biết phát hiện), nên `kind` do webview nói là tin cậy hoàn toàn; chỉ ảnh hưởng thống kê, nên đặt `low`, nhưng nên ghi "tỉ lệ là lời khai của webview".
- L5. AD-55 Rule 4 diff tính mỗi phiên bản mỗi lần đọc lịch sử (`diff_spans` N lần); chưa đo ở 17+ phiên bản; AGENTS.md đòi số đo trước khi chốt kiến trúc, AD chưa có.

---

## Các điều cổng không bắt được (tổng hợp)

1. Câu SQL nào chạm `glossary_entry.translation` (H1): cổng chỉ canh ai gọi hàm.
2. `TierRows` đọc dưới chính sách nào (H3.2): cùng kiểu.
3. Số cặp bị giấu bằng 0 hay khác 0 khi hiển thị "không có" (H3.1).
4. Nút "gỡ chốt" còn sống sau khi Tác phẩm đổi (M1): trạng thái thuần webview.
5. Cột `pending_confirmed_at` còn sau một bậc dựng lại bảng (M4).
6. Hai story cùng chiếm một số bậc di trú (M4).

## Thứ tự ưu tiên đề nghị

H1 → H2 (cần Ice chọn a/b, kèm số đo) → H3 → M1 → M4 → M2 → M3 → Low.
