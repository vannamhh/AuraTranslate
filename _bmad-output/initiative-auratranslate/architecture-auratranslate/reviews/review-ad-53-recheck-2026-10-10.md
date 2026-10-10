# Recheck AD-53 (bản sửa) — 2026-10-10

Phạm vi đọc: `### AD-53` (mục 1–8, Phương án bị loại), AD-3 :95, AD-4 :100, AD-47 ③ :718, AD-49 :786–:801, AD-50, AD-35, dòng Consistency Conventions :83, bản đồ C7 :1291, ba review trước. Mã chỉ đọc ở chỗ AD nêu tên.

## Kết luận: needs fixes (nhẹ, 5 điểm, không điểm nào chặn kiến trúc)

Các lỗ cao của ba review trước đã đóng. Còn một lỗ hở khớp ký tự đại diện do chữ "chạm biên" mơ hồ, một lớp AD-49 chưa khai, và ba điểm nhỏ. Tham chiếu chéo mục trong AD-53 và các dòng AD-3/4/47/49/Consistency/C7 đều trỏ đúng (không còn lỗi kiểu "mục 7" thay "mục 4").

## Kiểm mã

| Mệnh đề | Kết quả |
|---|---|
| `segment.retired`, `segment.not_in_open_chapter` | Có (`commands/segment/confirm.rs:43,52`). Hai hàm dựng lỗi vẫn `pub(super)`; hàm kiểm `pub(crate)` của mục 4 nằm trong `commands/segment/` nên gọi được; `commands/proofread.rs` cần `pub use` ở `mod.rs`. |
| `replaceEditorSegment` | Có, `src/reviewModeState.ts:27,373` dùng cho FR94 (xem điểm 4). |
| `prepare_proofread_call` | Có, `commands/proofread.rs:76`. |
| `ai_proofread.*` | Tiền tố backend khớp (`ai_proofread.reply_malformed`). `text_changed`, `range_invalid`, `text_would_be_empty` chưa có trong mã, đúng vai trò của 9.4. |
| Loại trên dây | `kind` `spelling`/`grammar` đã có ở `src/config/proofread.ts:15,40` và `core/ai/proofread.rs:17`; enum nằm trong `core/ai/`, AD mục 1 đòi tập đóng ở `core/segment/` (xem điểm 5). |

## Đối chiếu từng phát hiện

### Reality

| # | Phát hiện | Trạng thái | Chỗ đóng |
|---|---|---|---|
| R1 | `segment_retired` sai mã; helper không với tới từ `proofread.rs` | Đóng | Mục 4: `segment.retired`, một hàm `pub(crate)` dùng chung |
| R2 | Migration, đường dẫn thứ sáu ai_boundary, lọc thuần, ngoại lệ "read-only" | Đóng | Mục 7 bullet "Lọc là hàm thuần…" và "9.6 sở hữu…" |
| R3 | "Dây 9.1 không đổi" nửa đúng | Đóng | Mục 5 cuối: hình dạng phát hiện không đổi, `Done` thêm `ambiguous`, `filtered` |
| R4 | Khoá i18n, tiền tố, ca rỗng | Đóng | Mục 4 và mục 2 (`ai_proofread.text_would_be_empty`); tiền tố chọn `ai_proofread.*`. Bộ khoá i18n đầy đủ (MessageKey, vi.json) là việc của 9.4, không cần ghi ở AD |
| R5 | Chuẩn hoá chữ ký, helper UTF-16 | Đóng | Mục 7 "Cụm" (trim + NFC cả hai phía), mục 4 (`range_invalid`) |

### Rubric

| # | Phát hiện | Trạng thái | Ghi chú |
|---|---|---|---|
| F1 | Phép kiểm dùng chung | Phần | Mục 4 đóng phần "một hàm". Chưa có cổng cấm chỗ khác tự so (xem điểm 3) |
| F2 | AD-49 "không trạng thái hoàn tác sống qua lượt ghi" | Phần | Mục 3 và 8 + 🔵 ở AD-49 đóng câu mở đầu. Lớp của lượt chấp nhận trong (i)/(ii)/(iii) vẫn chưa khai (điểm 2) |
| F3 | Chữ thay nguyên văn, ranh giới dời, hoàn lại dời ngược | Đóng | Mục 2 "nguyên văn"; mục 4 khoảng rỗng ⇒ `range_invalid`; mục 6 `end ≤ s` / `start ≥ e` / chồng; áp cả hoàn lại |
| F4 | Chữ ký còn ba chỗ mơ hồ, N, "bỏ qua" hai chủ | Phần | NFC rồi cắt N ký tự Unicode (đóng a, b); N chỉ tăng + migrate (đóng c); "bỏ qua" vs "không phải lỗi" tách (đóng). Còn "chạm biên" (điểm 1) |
| F5a | AD-4 Prevents | Đóng | 🔵 ở :100 |
| F5b | Segment ngoài Chương đang mở | Đóng | Mục 4 vế hai |
| F5c | "loại" là chuỗi ổn định | Phần | Mục 1 tập đóng, Rust gán; tên chuỗi trên dây chưa ghim cho 9.2 (điểm 5) |

### Đối kháng

| # | Phát hiện | Trạng thái | Ghi chú |
|---|---|---|---|
| H1a | Ngữ cảnh rỗng là ký tự đại diện | Phần | Cờ chạm biên và luật khớp có, nhưng chữ "chạm biên" = "ngắn hơn N" làm phía rỗng khớp được cả phía hiện tại ngắn nhưng không rỗng (điểm 1) |
| H1b | Cụm lặp bị đặt nhầm lần xuất hiện | Đóng | Mục 5: loại theo ngữ cảnh mà cụm xuất hiện nhiều lần ⇒ `ambiguous`, không chấp nhận, không lọc |
| H1c | Cửa sổ ngữ cảnh cắt surrogate, trim | Đóng | N ký tự Unicode, cắt trên NFC(`text[..start]`); không còn đơn vị UTF-16 |
| H2 | Ô đang soạn, gõ chen, LIFO | Đóng | Mục 6: khoá gõ, `replaceEditorSegment`, dời cả phát hiện lẫn ngăn hoàn lại; mục 3: LIFO |
| H3 | Nhãn chiều đắt của loại 9.2; hoàn lại mất mốc | Đóng | Mục 1 bảng + mục 2 bảng bốn hàng + ⚠️; mục 3 bullet 1 trỏ ⚠️ mục 2; mục 3 bullet 2 nêu chiều rẻ cho 9.2. Số đo vẫn là suy từ mã, AD tự nhận ⚠️ cuối |
| H4 | Phạm vi quét, thế hệ, vô hiệu hoá | Phần | Đóng: `scanned_text` cả segment, thế hệ, xoá khi ghi rời/gộp/tách/về hưu, `unlocated`/`ambiguous`/`filtered`. Chưa có đếm "chưa quét" cho lô lỗi hay quét Chương bị huỷ giữa chừng (điểm 5) |
| H5a | Hai chủ của "bỏ qua", `PROOF_IGNORE` | Đóng | Mục 7: bỏ qua = bộ nhớ webview; 9.6 sở hữu bảng và migration. Vòng đời xoá Tác phẩm: `project.db` mỗi Tác phẩm nên bảng đi theo kho |
| H5b | `loại` do model chọn; khoá loại 9.2 chỉ chứa văn bản đích | Mở | Mục 7 khoá theo `loại` gán từ trả lời model; không nói gì về lật chính tả/ngữ pháp hay về nguồn đối ứng (điểm 5) |
| H5c | N do ai chốt | Đóng | Hằng số trong `core/ai/proofread.rs`, 9.6 chốt trước hàng đầu tiên, chỉ tăng |

## Điểm còn mở hoặc mới (tối đa 5)

1. **Mục 7 "chạm biên" có hai nghĩa (mới, thuộc H1a).** Định nghĩa là "phía đó ngắn hơn N" nhưng luật là "phía đã lưu rỗng và chạm biên chỉ khớp phía hiện tại cũng chạm biên". Phía hiện tại 3 ký tự (ngắn hơn N) cũng là "chạm biên", nên cụm đứng đầu segment ("Được." với ngữ cảnh trước rỗng) khớp mọi câu ngắn hơn N ký tự trước nó: lại là ký tự đại diện cho segment ngắn, đúng lớp phương án đã loại. Sửa: phía đã lưu rỗng chỉ khớp phía hiện tại **rỗng**; "chạm biên" chỉ còn là cờ lưu để biết một phía ngắn là do hết segment chứ không do N, dùng khi N tăng.
2. **Lớp AD-49 của lượt chấp nhận chưa khai.** Mục 8 nói ba lớp không đổi chữ, nhưng AD-49 mục 2 bắt mỗi thao tác khai đúng một lớp; (i) đòi lệnh nghịch đảo trả lại mọi nội dung, mà mục 3 ⚠️ thừa nhận ngăn hoàn lại mất thì cụm gốc không còn bản sao, tức (iii). Sửa: thêm vào mục 3/8 một câu "lượt chấp nhận thuộc (i) khi ngăn còn, và là ngoại lệ có tên của (iii) khi ngăn mất, vì cái bị thay là cụm model chỉ ra", và thêm 🔵 tương ứng ở AD-49 mục 2.
3. **Hai nơi tính chữ ký và phép kiểm chưa có người sở hữu duy nhất (còn lại của F1).** Hàm lọc (`prepare_proofread_call`) và lệnh "không phải lỗi" (`commands/proofread.rs`) đều dựng chữ ký (cụm, N, NFC, chạm biên); AD chỉ gọi tên hàm lọc ở đường dẫn thứ sáu, và không có cổng cấm chỗ khác tự so văn bản/khoảng. Sửa: nêu một hàm thuần `core/ai/proofread.rs` dựng chữ ký, cả lọc lẫn lệnh gọi nó (cùng một đường dẫn trong danh sách ai_boundary), và 9.4 thêm cổng "chỉ hàm kiểm mục 4 so văn bản mong đợi" như AD-50 mục 3.
4. **Phương án bị loại tự mâu thuẫn với mục 6.** Bullet "Đi qua bộ đệm gõ" bác vì "đây sẽ là lần đầu webview tự ghi vào ô đang soạn", nhưng mục 6 áp văn bản Rust vào ô qua `replaceEditorSegment`, đường FR94 đã dùng (`reviewModeState.ts:373`). Một agent đọc lý do bác sẽ nghi mục 6. Sửa: đổi lý do thành "sửa AD-35 đoạn 2 và phép từ chối ở webview trái AD-1"; bỏ vế "lần đầu".
5. **Phần còn hở của H4/H5, chưa ghi nợ.** (a) `loại` lấy từ trả lời model nên cùng cụm lật chính tả/ngữ pháp thì khoá khác ⇒ báo lại (FR84); khoá loại 9.2 chỉ chứa văn bản đích nên cùng cụm đích khác nguồn vẫn bị lọc; tên chuỗi `kind` trên dây của loại 9.2 chưa ghim; enum hiện ở `core/ai/` còn mục 1 đòi `core/segment/`. (b) Kết quả quét không đếm "chưa quét" cho lô lỗi/huỷ giữa chừng nên "0 phát hiện" vẫn có thể che lô chưa quét (Prevents 5 hứa chặn). Sửa: thêm một câu ⚠️ "Cái mất" nêu hai chỗ này kèm nợ `Chủ: Story 9.2` / `Story 9.3`, hoặc thêm `not_scanned` vào `Done`; ghi rõ enum nào sở hữu chuỗi `kind`.

Ghi chú nhỏ, không tính vào năm điểm: dòng Consistency Conventions viết khoá `(work, chữ ký)` còn mục 7 nói phạm vi Tác phẩm là cấu trúc, không có cột work; cả hai đúng nếu mỗi Tác phẩm một `project.db`, nhưng nên một chữ "do kho mỗi Tác phẩm" để khỏi bị đọc là thiếu cột. Khoá gõ trong lúc lệnh chạy làm rơi phím im lặng (mục 6); chấp nhận được nếu lệnh ngắn, 9.4 nên đo.
