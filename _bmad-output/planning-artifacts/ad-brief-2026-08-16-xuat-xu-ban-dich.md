# Hồ sơ bàn giao cho Winston — một `AD` mới về **xuất xứ bản dịch**

**Ngày:** 2026-08-16 · **Người bàn giao:** Amelia (dev-story) · **Người nhận:** Winston (architect)
**Nguồn gốc:** cửa chặn **Task 0.4** của Story 2.7 (`2-7-xuat-xu-ban-dich-cap-segment.md`)
**Trạng thái Story 2.7:** `in-progress`, **DỪNG** trước dòng mã đầu tiên. Cây nguồn nguyên baseline
`440c6d5` — `git diff --stat` trên `src/ src-tauri/ scripts/ tests/ e2e/` = **rỗng**.

---

## 1. Vì sao hồ sơ này tồn tại

Story 2.7 mang tám quyết định mở phải có chữ ký của Ice trước dòng mã đầu tiên. Ice ký cả tám ngày
2026-08-16. **Bảy chữ ký đi qua sạch.** Chữ ký thứ tám — **#8(b)**, *"2.7 tự chốt luôn ngữ nghĩa
xuất xứ cho cả FR94 lẫn gộp/tách"* — đòi **thêm luật vào hai bất biến đang đứng**:

- **AD-31** (`ARCHITECTURE-SPINE.md:383-390`) — bảng xuất xứ có **đúng hai hàng**, cả hai dựa trên
  phép so văn bản.
- **AD-5** (`:103-111`) — đọc lại toàn văn: *"segment mới bắt đầu ở trạng thái chưa xác nhận với
  lịch sử rỗng"*, và **không một chữ** về xuất xứ.

`project-context.md:461-463` và Task 0.4 của story cùng nói một điều: **đổi một bất biến kiến trúc
là một `AD` MỚI, không một dòng mã.** ⇒ Story dừng, `AD` viết trước.

⚠️ **Ice đã biết cái giá khi ký.** Lựa chọn #8(b) được trình kèm nguyên văn hệ quả *"chọn đường này
thì tôi dắt story lại và viết `AD` trước"*. Đây không phải một lượt phát hiện muộn.

---

## 2. Bảy chữ ký đã có — chúng RÀNG BUỘC `AD` mới, đọc trước khi thiết kế

| # | Ice ký 2026-08-16 | Ràng buộc nó đặt lên `AD` |
|---|---|---|
| **#1** | **(a)** cột xuất xứ **chỉ** trên `segment` | 🔴 `segment_version` **không** mang xuất xứ. Mọi luật của `AD` phải phát biểu được ở cấp **segment hiện tại**, không cấp phiên bản. |
| **#2** | **(b)** webview gửi **văn bản lúc nạp**, **Rust** chạy phép so | Phép phân xử là quy tắc nghiệp vụ và ở lại Rust (AD-1). `AD` không được đặt một luật nào chỉ thi hành được ở TypeScript. |
| **#3** | **(b′)** `TEXT NOT NULL DEFAULT ''` | Tập giá trị trên đĩa là `''` *(chưa có bản dịch)* **cộng** ba giá trị của FR117. `''` là hàng thứ tư **có thật**, và `AD` phải nói nó nghĩa gì khi gặp gộp/tách. |
| **#4** | **(a)** không bề mặt nhìn thấy | `AD` không được đòi một chỉ dấu thị giác ở Epic 2. |
| **#5** | **(b)** khôi phục chỉ đụng văn bản | Khôi phục **không** trả xuất xứ về — hệ quả bắt buộc của #1(a). |
| **#6** | **(a)** backfill theo hàng: `confirmed ⇒ tôi dịch` | Dữ liệu đang có trên đĩa sẽ mang *tôi dịch* cho mọi câu đã ký. |
| **#7** | tin payload | Không ràng buộc kiến trúc. |

🔴 **Hai đường CHẾT theo #1(a), ghi ra để Winston không hồi sinh chúng:** #4(b) *(vẽ xuất xứ trong
lớp phủ lịch sử)* và #5(a) *(khôi phục trả xuất xứ về)*. Cả hai đọc/ghi xuất xứ theo **một hàng
phiên bản**, thứ nay không tồn tại.

---

## 3. Câu hỏi thứ nhất — xuất xứ khi **chấp nhận nguyên văn** từ Review Mode (FR94)

### Số đo

- AD-31 liệt kê *"Chấp nhận thay đổi từ Review Mode (FR94)"* trong bảng **máy trạng thái**
  (`SPINE:380`) — nhưng bảng **xuất xứ** ngay dưới (`:385-388`) chỉ có hai hàng, cả hai là phép so
  văn bản.
- Đọc theo **đúng chữ** hôm nay: reviewer sửa câu ⇒ văn bản khác bản lúc nạp ⇒ ghi **tôi dịch**, cho
  một câu người dùng **không gõ một ký tự nào**.
- Điều đó ngược đúng câu biện minh của FR117 (`prd.md:452`): *"hệ thống biết được vì nó thấy bạn có
  gõ hay không"*, và nó thủng lời hứa R13 mà FR117 sinh ra để giữ.

### 🔴 Số đo làm phạm vi rộng hơn vẻ ngoài — đây là thứ Ice chưa thấy lúc ký #8(b)

Mockup `mockups/data-integrity.html` đã vẽ **bốn** nhãn xuất xứ, không ba:

| Nhãn | Dòng |
|---|---|
| `từ bản review` | `:195` |
| `từ AI` | `:203` |
| `từ TM` | `:207` |
| `tôi dịch` | `:221` |

FR117 (`prd.md:443`) khai **ba** giá trị. ⇒ Nếu `AD` chọn một giá trị **riêng** cho ca FR94, nó
**nới tập giá trị của FR117**, và lượt nới đó kéo theo **Epic 4** (từ AI) và **Epic 7** (từ TM) chứ
không chỉ Epic 8. Một `AD` nới ba giá trị thành bốn mà bỏ hai nhãn kia lại là dựng một tập giá trị
sẽ phải nới lần thứ hai.

⚠️ Và Story 7.4 (`epics.md:5168-5170`) đã thêm sẵn một luật **không có trong FR117 gốc**: xác nhận
một segment điền sẵn từ TM khớp 100% mà không sửa ⇒ *"giữ nguyên xuất xứ của cặp TM nguồn"*. Tức
Epic 7 đang **giả định** một hình dạng mà chưa `AD` nào chốt.

### Đường ứng viên (Winston chọn hoặc dựng đường thứ tư)

| Đường | Nội dung | Cái giá đo được |
|---|---|---|
| **(i)** | Ca FR94 ghi **người khác dịch** — reviewer là người khác | Giữ đúng ba giá trị của FR117, không nới đặc tả. ⚠️ Nhưng nó **phá hợp đồng phụ AD-31** *(so văn bản, không cờ dirty)*: văn bản **có** khác bản lúc nạp mà kết quả lại là *người khác dịch* ⇒ phép so văn bản thôi **không còn đủ** để suy xuất xứ, phải thêm một đầu vào thứ hai *(nguồn của lượt thay đổi)*. Đó là một sửa đổi thật vào AD-31. |
| **(ii)** | Ca FR94 ghi **tôi dịch** — người dùng đã ký duyệt nó | Đúng chữ AD-31 hôm nay, **0 dòng sửa**. ⚠️ Nhưng nó chính là cái ngược FR117 `:452` mà Ice đang muốn đóng ⇒ nếu chọn đường này thì `AD` mới thành một `AD` **xác nhận hiện trạng**, và Ice cần biết nó không giải quyết gì. |
| **(iii)** | Thêm giá trị **`từ bản review`** *(và cân nhắc `từ AI`, `từ TM` cùng lượt)* | Khớp mockup, và trả lời được câu *"chữ này của ai"* chính xác nhất. ⚠️ **Nới FR117 từ ba lên bốn/sáu giá trị** ⇒ chạm Epic 4 · 7 · 8; và tập giá trị nằm trên **đĩa người dùng** nên nới nó về sau là một bước di trú nữa. |

---

## 4. Câu hỏi thứ hai — xuất xứ của segment sinh ra từ **gộp/tách** (AD-5)

### Số đo

- AD-5 (`SPINE:103-111`) toàn văn: segment cũ về hưu, *"segment mới bắt đầu ở trạng thái **chưa xác
  nhận** với **lịch sử rỗng**"*, cặp TM đã ghi ở lại nguyên. **Không một chữ** về xuất xứ.
- Story 2.8 (`backlog`) chưa tồn tại đường mã: `grep merge_segment` trên `src-tauri/src` = **0**
  đường mã. ⚠️ *(Kết quả `grep` thô trả 1 dòng, nhưng dòng đó là một **doc-comment** ở
  `paragraph.rs:10` viết nguyên văn *"grep … cho 0"* — bẫy đã bắt được ở Story 2.6. Số thật là 0.)*
- Chữ ký #3(b′) làm câu hỏi này **cụ thể hơn**: tập giá trị trên đĩa nay có `''` = *chưa có bản
  dịch*, nên *"xuất xứ rỗng"* là một trạng thái **biểu diễn được**, không phải một chỗ trống.

### Đường ứng viên

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(i)** | Segment mới nhận `''` *(chưa có bản dịch)* | Nhất quán nhất với AD-5: *chưa xác nhận · lịch sử rỗng · xuất xứ rỗng* — ba vế cùng nói một điều. ⚠️ Mất thông tin: gộp hai câu **đều** *người khác dịch* ra một câu không xuất xứ. |
| **(ii)** | Kế thừa khi **mọi** mảnh đồng ý, còn lại về `''` | Giữ được thông tin ở ca thường nhất *(tách một câu ⇒ mọi mảnh cùng xuất xứ)*. ⚠️ Một luật có nhánh, và nhánh đó phải nghiệm thu được ở Story 2.8. |
| **(iii)** | Luôn **tôi dịch** | Rẻ nhất. ⚠️ Khai *tôi dịch* cho chữ của người khác — đúng lớp lỗi FR117 tồn tại để chống. |

---

## 5. Thứ `AD` mới **KHÔNG** được đụng

- 🔴 **AD-31 §Hợp đồng phụ** — *"so văn bản đích hiện tại với bản lúc nạp segment, không dùng cờ
  dirty"*. Bảy AC của Story 2.7 dựng trên nó; AC4 nói nguyên văn. Nếu đường (i) của câu hỏi 1 buộc
  thêm một đầu vào thứ hai, hãy **nới** hợp đồng chứ đừng thay nó, và nói rõ ca gõ-rồi-hoàn-tác vẫn
  cho *không sửa*.
- 🔴 **Bảng máy trạng thái AD-31** (`:374-381`) — sáu hàng, không hàng nào bị story này chạm. Story
  2.5 đã cài đúng nó và có 372 ca Rust canh.
- 🔴 **AD-46 khai bằng chữ *"AD-37 không sửa một chữ"*** — khuôn đó là tiền lệ tốt: một `AD` nới
  một `AD` khác thì **nói ra bằng chữ** cái gì không đổi.
- **Không cài `similar` lẫn `dissimilar`** (`Cargo.toml:86-89`) — quyết định kiến trúc chốt ở Story
  8.1, đừng đóng sớm.

---

## 6. Nghiệm thu — `AD` xong nghĩa là gì để Story 2.7 mở lại được

1. `AD` mới có mặt trong `ARCHITECTURE-SPINE.md` theo đúng khuôn `### AD-NN — <mệnh đề>` với ba mục
   **Binds** · **Prevents** · **Rule** *(khuôn AD-46 là bản gần nhất và đầy đủ nhất)*.
2. Nó trả lời **cả hai** câu hỏi ở §3 và §4 — bỏ một câu là để lại đúng khoảng im lặng story này
   phát hiện ra.
3. Nó nói bằng chữ **AD-31 và AD-5 đổi cái gì / không đổi cái gì**, khuôn AD-46.
4. Nếu nó nới tập giá trị của FR117: ghi ra **ai chịu** phần Epic 4 · 7 · 8, và cảnh báo rằng tập
   giá trị nằm trên đĩa người dùng ⇒ nới lần hai là một bước di trú nữa.
5. `python3 .claude/skills/bmad-architecture/scripts/lint_spine.py` chạy sạch.

---

## 7. Hai mục metadata của spine đã lệch — chủ là Winston, sửa cùng lượt

Đo 2026-08-16, không thuộc phạm vi `AD` mới nhưng cùng người sở hữu:

- `ARCHITECTURE-SPINE.md` frontmatter ghi `updated: '2026-08-11'`, mà **AD-46** trong chính tệp đó
  dẫn *"bản ghi phiên thiết kế 2026-08-14"* ⇒ trường `updated` đã hết đúng.
- `project-context.md:590` viết *"`ARCHITECTURE-SPINE.md` (45 `AD` …)"*; đếm thật:
  `grep -c "^### AD-"` = **46**.

---

## 8. Số đo baseline, để Winston khỏi đo lại

HEAD `440c6d5` · 2026-08-16 · rustc/cargo 1.97.1 · vitest 4.1.10:

- `cargo test --locked` **372 / 0 / 5**
- `npm run test` (vitest) **130 / 130**, 12 tệp
- `COMMAND_FLOOR` sàn **37**, cổng in **44**
- Bước di trú `project.db` kế tiếp: **11** *(`PROJECT_MIGRATIONS` dịch ở `to_version: 10`,
  `schema.rs:799`; chín bước `[1,2,3,5,6,7,8,9,10]`, số 4 đã cháy vĩnh viễn)*

Mười một tiền đề của tám quyết định đã đo lại từ nguồn — chín đứng nguyên, hai bị **thu hẹp** *(tầng
Rust có **0** định danh `origin`/`provenance`, không phải năm; `config/segment.ts` có **7** adapter
tin payload, không phải sáu)*. Bảng đầy đủ ở `§Dev Agent Record` của Story 2.7.

---

## 9. Phản hồi của Winston — 2026-08-16

**Kết quả: `AD-47` đã có trong `ARCHITECTURE-SPINE.md`.** `lint_spine.py` **0 finding**;
`grep -c "^### AD-"` = **47**; `U+26D4` = **0**. Frontmatter `updated` → `2026-08-16`;
`project-context.md:590` *"45 `AD`"* → **47** *(§7 đóng)*.

### 🔵 Một tiền đề của §3 KHÔNG đứng — và nó THU HẸP phạm vi, không nới

Hồ sơ viết mockup vẽ **bốn nhãn xuất xứ**. Đo lại toàn tệp `data-integrity.html`:

| Đo | Kết quả |
|---|---|
| `grep -n "tagx"` | **bốn** hàng: `:191` *đang dùng* · `:195` *từ bản review* · `:203` *từ AI* · `:207` *từ TM* |
| `grep -n "Xuất xứ"` | **đúng một** hàng trong cả tệp: `:221`, giá trị *tôi dịch* |
| Nhãn khối chứa `:191-207` | `:179` — *"2 · Lịch sử phiên bản của một câu · **FR101**"* |

⇒ Hồ sơ liệt kê ba trong bốn thành viên họ `tagx` rồi ghép chúng với `:221`. Thành viên bị bỏ sót là
`:191` **đang dùng** — một **trạng thái**, không phải một xuất xứ. Một từ vựng chứa *"đang dùng"*
không phải từ vựng xuất xứ. Và `:221` không thuộc họ `tagx`: nó ở `div.vmeta` **cấp segment**, mang
tiền tố chữ `Xuất xứ:`.

**Mockup vẽ HAI trục.** Trục `tagx` = *"phiên bản này từ đâu tới"*, đúng trục mà **#1(a) đã xoá**.
⇒ Mockup **không** ép FR117 nới, và Epic 4 · Epic 7 **không** bị kéo vào qua đường đó. Kết luận
*"nới ba lên bốn/sáu"* mất tiền đề, và **không cần Ice ký lại gì**.

⚠️ Câu hỏi §3 thì **vẫn thật** — nó không sống nhờ mockup mà nhờ chính AD-31. `AD-47` trả lời nó.

### Ba đường ứng viên của §3 đều bị bỏ, vì cả ba hỏi sai câu

`AD-47` không chọn (i), (ii) hay (iii). Số đo bác cách đặt câu hỏi: FR94 **không phải một ca**, nó
là **thành viên đầu tiên của một lớp**. FR58 (Epic 7) và AI (Epic 4) mang **y hệt** lỗ đó — cả ba
ghi `target_text` mà người dùng không gõ. ⇒ Vá ba chỗ ở ba Epic cách nhau nhiều tháng là ba cách
hiểu riêng về cùng một câu hỏi, trên dữ liệu nằm trên đĩa người dùng.

`AD-47` **dời mốc** thay vì thêm giá trị: mốc so là *"bản do lượt ghi không-phải-người-dùng gần nhất
đặt"*, không phải *"bản lúc nạp"*. Hôm nay lượt nạp là lượt ghi loại đó **duy nhất đã cài**, nên hai
cách đọc cho **cùng kết quả trên toàn bộ mã đang chạy** — `AD` này **0 dòng sửa** cho mã hiện có.

Hệ quả: **FR117 giữ đúng ba giá trị** *(cộng `''`)*, **không bước di trú nào thêm**, FR94 ⇒
*người khác dịch*, và **Story 7.4 hết là một giả định** — AC `epics.md:5169-5170` nay là hệ quả của
`AD-47 ③` cộng `②`, không phải một luật rời chưa ai chốt.

### §4 — gộp/tách

`AD-47 ④`: mọi mảnh cùng giá trị ⇒ giữ; **bất kỳ bất đồng nào** ⇒ *người khác dịch*. Tức đường (ii)
của hồ sơ, với nhánh *else* giải về *người khác dịch* thay vì `''` — vì `''` khai *"chưa có bản
dịch"* cho một segment **có** văn bản, tức nạp nghĩa thứ hai lên một sentinel đã có nghĩa. Chiều nói
dối chọn theo giá **không cân nhau**: nhãn *tôi dịch* sai đầu độc kho TM vĩnh viễn; nhãn *người khác
dịch* sai chỉ làm một cặp bị xếp sau.

### Năm điều kiện nghiệm thu §6

| # | Trạng thái |
|---|---|
| 1 | ✅ `### AD-47 — …` với **Binds · Prevents · Rule**, khuôn AD-46 |
| 2 | ✅ trả lời cả §3 *(③ hàng FR94)* và §4 *(④)* |
| 3 | ✅ `⑦` nói bằng chữ AD-31 và AD-5 đổi gì / không đổi gì, cộng AD-18 · AD-14 · AD-6 |
| 4 | ✅ **không nới** tập giá trị ⇒ điều kiện này rơi; `⑥` vẫn ghi luật cho lượt nới tương lai *(phải khai vế trên trục nhị phân FR118, và nó là một bước di trú)* |
| 5 | ✅ `lint_spine.py` **0 finding** |

### 🔴 Món nợ `AD-47` MỞ RA, có chủ — không tự chấm đạt

`AD-47 ⑤`: khôi phục FR101 đặt mốc mà **không** đặt xuất xứ. ⇒ khôi phục văn bản một phiên bản cũ
rồi xác nhận **không sửa** ⇒ giữ xuất xứ hiện tại, thứ có thể thuộc phiên bản khác. **Cùng gốc** với
món nợ bốn nhãn của Story 2.6 (`deferred-work.md:3685-3697`), đóng cùng lúc với nó. Chủ: story nào
cho `segment_version` một cột xuất xứ. **Story 2.7 không được chấm đạt vế này.**

### Story 2.7 — Task 0.4 MỞ

Bảy chữ ký còn nguyên hiệu lực. `AD-47` không đụng chữ ký nào, không đòi ký lại, và không thêm AC
nào cho 2.7: bảy AC hiện có **đủ** cho phần 2.7 phải cài. Ba hàng còn lại của bảng `③` *(FR115 ·
FR94 · FR58 · AI)* thuộc Epic 6 · 8 · 7 · 4 và đi vào `deferred-work.md` theo chủ ghi ở cột thứ ba.
