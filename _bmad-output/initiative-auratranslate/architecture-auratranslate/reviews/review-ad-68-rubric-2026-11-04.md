# Review AD-68 theo rubric — 2026-11-04

Phạm vi đọc: AD-68 (dòng 1743 trở đi), bảng AD-18, các đoạn mã `schema.rs` (bước 41 `chosen_prompt_set_name`, `PROMPT_SET_DDL`), `promptset/store.rs`, `exchange_io.rs::export_file_name`. Không đọc lại AD-14/AD-60/AD-66 ngoài các đoạn trích dẫn.

## Kết luận
AD-68 đủ chắc ở ba điểm cốt lõi (hợp đồng trả lời là hằng số, thứ tự lắp, tệp bốn dòng), nhưng chưa đóng được vòng đời của lựa chọn (đổi tên, xoá, ai ghi tầng Global), chưa nói xuất tệp va chạm tên giữa hai loại, và bỏ trống FR71 cho proofread. Hai người cài sẽ lệch ở các chỗ đó. Chưa nên đóng AD cho tới khi sửa các mục critical/high.

## Critical

### C1. Đổi tên / xoá bộ đang được chọn không có luật
Mã hiện tại (`schema.rs` bước 41) ghi rõ: tên chọn "không bao giờ ghi lại khi bộ bị đổi tên hay xoá". Rule 3 chỉ xử lý "tên đã chọn mà không còn" ở lượt quét (lỗi, 0 lời gọi mạng). Chưa nói:
- Đổi tên bộ proofread đang được chọn (ở Global hoặc ở Work) thì lựa chọn đi theo hay mồ côi? Hai người cài sẽ chọn hai cách (đi theo = ghi vào `config_value` và `work`, mà tầng Work nằm ở kho khác với bộ tầng Global).
- Xoá bộ tầng Work đang che bộ Global cùng tên: lựa chọn lặng lẽ chuyển sang bộ Global (tên vẫn phân giải được). Đúng theo Rule 3 nhưng người dùng không được báo.
- Màn chọn hiện gì khi tên đã chọn không còn: chỉ lượt quét báo lỗi, còn màn chọn (Cài đặt / Tác phẩm) không có hành vi nêu ra.
Đề xuất thêm Rule 3b: "Đổi tên và xoá không sửa lựa chọn (cùng luật bước 41 của đường dịch). Chỗ chọn ở cả hai tầng hiện tên đã chọn kèm trạng thái *không tìm thấy* (khoá i18n riêng) khi không bộ `proofread` nào mang tên đó ở các tầng đọc được; không tự xoá lựa chọn, không tự đổi sang bộ khác. Lệnh xoá và đổi tên bộ `proofread` đang được chọn ở tầng nào đó vẫn chạy, và hộp xác nhận nêu rằng lựa chọn sẽ mồ côi." Thêm ca test (i).

### C2. Lệnh nào ghi lựa chọn Global không được nêu
Rule 3 nêu nơi lưu (`config_value`, `app_config`, `proofread_prompt_set`) nhưng không nêu: lệnh IPC mới hay dùng chung lệnh ghi `app_config` có sẵn; có đi qua `store::Writer` của `global.db` (AD-11) không; lệnh ghi cột `work.chosen_proofread_prompt_set_name` có theo khuôn `set_chosen_prompt_set_name` của đường dịch (không `updated_at`, không `meta.json`) không; lệnh nhận `kind` hay là hai lệnh riêng; ghi giá trị rỗng/`NULL` nghĩa là gì ở Global (xoá hàng) và ở Work (đặt `NULL`). Đề xuất thêm vào Rule 3: "Hai lệnh ghi lựa chọn nhận tên hoặc `null`; `null` ở Global xoá hàng `config_value`, ở Work đặt cột `NULL`. Lệnh Work theo khuôn `promptset::store` (một cột, không `updated_at`, không `meta.json`). Lệnh không kiểm bộ tồn tại, để C1 hoạt động." Nếu muốn kiểm tồn tại lúc ghi thì phải nói, vì nó đổi hành vi nhập tệp rồi chọn.

## High

### H1. Tên tệp xuất va chạm giữa hai loại
`export_file_name(name)` chỉ dựa trên tên; bộ `translate` và `proofread` cùng tên "Chuẩn" cho ra cùng `Chuẩn.prompt.md`, mà Rule 1 cho phép trùng tên khác loại. Bộ đọc va chạm tên tệp trong thư mục xuất (hậu tố `-n`) sẽ xử lý được trong một lượt xuất nhưng không nói gì về xuất hai lần. Đề xuất Rule 5: tên tệp xuất của `proofread` là `<stem>.proofread.prompt.md`, của `translate` giữ nguyên; bộ đọc không dựa vào đuôi mà dựa vào dòng `kind:`. Bổ sung ca test (f) kiểm tên tệp.

### H2. Thứ tự di trú trong hai kho và chỉ mục
"Mỗi kho có một bước di trú mới" chưa đủ: (1) `ALTER TABLE ADD COLUMN ... CHECK` phải chạy trước khi bỏ chỉ mục `idx_prompt_set_name` và dựng `(kind, name)`, trong CÙNG bước (một transaction) để không có khoảnh khắc không chỉ mục; (2) cột `work.chosen_proofread_prompt_set_name` thuộc bước `project.db`, không phải bước `prompt_set`: hai bước hay một bước ở `project.db`? (3) số bước cụ thể lấy theo bước cuối hiện có, giữ `PROJECT_MIGRATIONS`/`GLOBAL_MIGRATIONS` song sinh như bước 9/41. (4) `.atproj` cũ mở bằng app mới chạy bước; `.atproj` mới mở bằng app cũ vấp `user_version` cao hơn, kiểm cái này đã có luật chung chưa và viện dẫn. Đề xuất: ghi "Một bước mỗi kho: `global.db` = cột `kind` + thay chỉ mục; `project.db` = cột `kind` + thay chỉ mục + cột `chosen_proofread_prompt_set_name`, trong một transaction" và thêm ca test di trú: kho có hàng `prompt_set` trước bước ⇒ sau bước mọi hàng `kind='translate'`, chèn bộ cùng tên khác loại thành công, cùng tên cùng loại bị từ chối.

### H3. FR71 cho lượt quét proofread bị bỏ trống
AD-60 mục 13 và AD-66 mục 5 đều nêu FR71. AD-68 thay đổi prompt người dùng viết được nhưng không nói prompt proofread có được ghi vào `AiCallRecords` / `LastAssembledPromptState` hay xem được không. Đó chính là công cụ chẩn đoán khi chỉ dẫn tuỳ chỉnh làm hỏng kết quả (động cơ của AD). Đề xuất thêm Rule 4: "Prompt proofread đã lắp (kể cả chỉ dẫn) hiện theo cùng đường FR71 như prompt dịch, hoặc, nếu hôm nay lượt quét không đi qua đường đó, ghi quyết định rõ: không áp dụng FR71 cho proofread và tên cái thay thế." Cần kiểm mã `commands/proofread.rs` trước khi chọn vế; AD không được im lặng.

### H4. Hai nguồn sự thật cho "chọn bộ" ở màn Tác phẩm khi `kind` của lựa chọn lệch
Rule 3 phân giải "tên trong loại `proofread` qua hai tầng". Nhưng bộ tầng Work `translate` tên "A" tồn tại, bộ `proofread` "A" chỉ ở Global: Work chọn "A" ⇒ dùng bộ Global. Hợp lệ theo Rule 1, nhưng ca test (c) chỉ cover A/B hai tên khác nhau và (e) chỉ cover trùng tên khác loại; không ca nào cover "Work chọn tên chỉ có ở Global" hay "Work-tier proofread che Global-tier proofread cùng tên". Đó là phần phân giải hai tầng cốt lõi. Thêm ca (c2) với hai cặp đó, đối chứng: "Phân giải theo tầng Work không rơi xuống Global".

## Medium

### M1. Rule 2 và Rule 4 mâu thuẫn nhẹ về "gửi nguyên văn"
Rule 2: "Chữ trông giống marker được gửi nguyên văn." Rule 4: dòng trùng dấu khối bị chèn U+200B. Hai câu cùng đúng nhưng người cài đọc Rule 2 sẽ cấm mọi biến đổi. Sửa Rule 2: "…được gửi nguyên văn, ngoại trừ phép chèn U+200B ở Rule 4."

### M2. Chỉ dẫn "thân rỗng sau khi cắt khoảng trắng = không có chỉ dẫn" và byte-đồng-nhất
Rule 4 nói không chỉ dẫn ⇒ prompt trùng từng byte. Chưa nói chỉ dẫn có khoảng trắng đầu/cuối được cắt hay gửi nguyên (ảnh hưởng số dòng trống quanh chỉ dẫn, nên ảnh hưởng ca (b)). Đề xuất: cắt khoảng trắng hai đầu, ngăn cách chỉ dẫn với hai phần kề bằng đúng một dòng trống, ghi trong Rule 4 mục 2.

### M3. Tên lệnh/khoá i18n và cờ trên mọi đường "nhận loại làm tham số"
Rule 1 cuối đòi mọi đường nhận `kind`, nhưng test (e) chỉ canh một truy vấn. "Mọi đường" không thể canh bằng một ca. Đề xuất: liệt kê các đường (liệt kê, đọc theo tên, tạo, đổi tên, xoá, nhập, xuất, phân giải hai tầng) và đòi một ca mỗi đường, hoặc đòi `kind` là kiểu không có giá trị mặc định (`PromptKind` bắt buộc ở chữ ký hàm) để trình biên dịch canh thay test.

### M4. Tác phẩm chọn bộ rỗng làm "bỏ chỉ dẫn Global"
Cơ chế hoạt động nhưng buộc người dùng tạo bộ Work thân rỗng. Không sai, nhưng UI không có chỗ nào được nêu để nói điều đó, và Prevents 4 ("mỗi Tác phẩm phải chọn lại") không đụng tới. Đề xuất thêm một câu: giao diện chọn Work có mục "Không dùng chỉ dẫn" và nó ghi sao (xem C2): hoặc chọn rõ rằng mục này tạo/chọn bộ rỗng, hoặc nêu rằng chưa có và ghi nợ. Hiện AD để câu hỏi mở.

### M5. Bảng AD-18 xếp lựa chọn Global proofread ở đâu
Bảng AD-18 có hàng "Lựa chọn ứng dụng — chỉ toàn cục". Khoá `proofread_prompt_set` ở `app_config` vừa là lựa chọn Global vừa bị Work ghi đè theo trường. Nên nói rõ trong AD-68 rằng ghi đè này là phân giải của AD-18 hàng Prompt, không phải hàng "Lựa chọn ứng dụng", để không ai đưa nó vào `ScopeResolver` như một khoá chỉ-Global.

## Low

### L1. Khớp "Ice chọn ngày" và bằng chứng
Phần Phương án bị loại không mâu thuẫn quyết định đã ký. Không cần sửa.

### L2. Test (a) cần bản ghi vàng cho cả hai biến thể
`PROOFREAD_INSTRUCTION` và `PROOFREAD_INSTRUCTION_WITH_SOURCE` là hai nhánh; ca (a) phải có hai bản ghi vàng và ít nhất một lô nhiều khối. Thêm câu này.

### L3. Test (d) và (g) có thể đã đỏ vì lý do khác
(d): thay lỗi bằng "rơi về chỉ dẫn gốc" làm đỏ vì lời gọi mạng đếm 0 sai, đúng lý do. Ổn. (g): "Bỏ bước chèn" làm đỏ nếu bộ đọc đếm khối; xác nhận fixture có dòng trùng dấu MỞ và dấu ĐÓNG (AD-60 mục 4 nói cả hai?), nếu chỉ một thì nhánh kia không có ca.

### L4. Đụng chạm AD hiện có
Không thấy mâu thuẫn với AD-14 (hàm thuần giữ nguyên), AD-53/56 (ghi nhớ và đếm không đổi), AD-60 (tái dùng luật U+200B), AD-66 (chỉ nhận dòng 🔵 ở mục 6). Bảng AD-18 đã có dòng 🔵 trỏ AD-68; kiểm AD-66 mục 6 đã nhận dòng 🔵 chưa và Map C6/C7/C9 đã thêm AD-68 chưa (Rule 8 là việc cần làm, chưa phải việc đã làm).

## Tóm lược sửa theo thứ tự ưu tiên
1. C1 và C2: thêm luật đổi tên/xoá/mồ côi và chỉ định lệnh ghi lựa chọn.
2. H1: tên tệp xuất theo loại.
3. H2: viết rõ nội dung từng bước di trú và đòi ca test di trú.
4. H3: quyết FR71 cho proofread.
5. H4 và M3: bổ sung ca test phân giải hai tầng và canh "mọi đường nhận loại".
