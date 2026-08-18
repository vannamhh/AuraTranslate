# Hồ sơ bàn giao cho Winston — một `AD` mới về **mô hình hoàn tác**

**Ngày:** 2026-08-17 · **Người bàn giao:** Amelia (dev-story) · **Người nhận:** Winston (architect)
**Nguồn gốc:** cửa chặn **Task 0** của Story 2.9 (`2-9-gop-bang-backspace-dau-o.md`)
**Trạng thái Story 2.9:** `in-progress`. Cửa chặn này **chỉ chặn AC5**; năm AC còn lại đứng được
một mình — xem §7.
**Baseline cây nguồn:** `4d72cd4` — cargo test `--locked` **401/0/5** · `segment_contract` **121** ·
vitest **141/141** · 9 cổng đọc-tệp xanh · `COMMAND_FLOOR` sàn **38**.

---

## 1. Vì sao hồ sơ này tồn tại

**AC5 của Story 2.9** viết trọn vẹn ba dòng:

> **Given** gộp vừa xảy ra · **When** người dùng bấm `⌘Z` · **Then** hoàn tác được

Không tài liệu nào của dự án nói **hoàn tác được nghĩa là gì**. Đo lại trên cây nguồn hôm nay
2026-08-17 (Task 0.1 của story, chạy lại chứ không tin ảnh chụp lúc soạn story):

| Phép đo | Kết quả |
|---|---|
| `grep -rniE "undo\|redo\|UndoManager" src/ src-tauri/src/` | **0 cơ chế.** 8 dòng trúng: 7 là chữ `dock`/`undock` của dockview, 1 là chú thích `GridPanel.vue:1064` |
| `grep -rn "KeyZ" src/commands/` | **0.** Không command `Mod+Z` nào |
| `grep -rniE "undo\|hoàn tác\|⌘Z" prd.md` | Trúng **duy nhất** chữ `undock` ở FR17. **Không FR nào** định nghĩa mô hình hoàn tác |
| `EXPERIENCE.md` bảng Phím `:261-268` | **Không hàng `⌘Z`.** Chữ *"hoàn tác bằng `⌘Z`"* chỉ xuất hiện trong **văn xuôi** UX-DR32 (`:171`) — một **lời hứa**, không một mô hình |
| `grep -c "^### AD-" ARCHITECTURE-SPINE.md` | **47.** `AD` kế tiếp là **48** |

`project-context.md:461-463` và Task 0.4 của story cùng nói một điều: **đổi hoặc thêm một bất biến
kiến trúc là một `AD` MỚI, không một dòng mã.** ⇒ AC5 dừng, `AD` viết trước.

🔴 **Và đây không phải một `AD` "cho đủ thủ tục".** Hai đường cài đặt ở §3 **đều biên dịch sạch,
đều đi qua cả mười một cổng**, và chúng khác nhau ở **dữ liệu nằm trên đĩa người dùng vĩnh viễn**.
Đúng mục *"chỗ hỏng là VĨNH VIỄN"* của `project-context.md`.

---

## 2. Vì sao nó không giải được bằng một lượt "tiện tay"

### 2.1 — `segment_version` KHÔNG phải `⌘Z`, và điều đó đã được khoá bằng test

AD-31 (`SPINE:381`) có một hàng viết thẳng: *"Về hưu do gộp/tách (AD-5) | về hưu | **không** tạo"*.
Story 2.8 cài đúng thế, và doc-comment của `write_regroup` (`segment.rs:2160-2195`) giải thích tại
sao bản năng ngược lại là sai:

> *Bản năng khi đọc "đừng để mất bản dịch của người dùng" là chụp một `segment_version` trước khi
> cho về hưu — và lượt chụp đó **phá AC3** (segment mới bắt đầu với lịch sử RỖNG) theo một cách đọc
> rất giống một tính năng.*

Khoá bằng `neither_merge_nor_split_ever_writes_a_segment_version_row` (`segment_contract.rs`).

⇒ **Một lượt gộp không để lại bản sao nào** mà cơ chế lịch sử hiện có hoàn tác được.

### 2.2 — Native undo của `contenteditable` KHÔNG đủ, và nhận nó là một số đo không có thật

Từ Story 2.5b (chữ ký #3 của Ice), **mỗi ô bản dịch là một editing host riêng**. Một lượt gộp:

1. xoá **hai** ô khỏi DOM,
2. dựng **một** ô mới,
3. và đã ghi xuống **WAL** qua `store::Writer` trước đó.

Native undo chỉ hoàn tác ký tự **trong một** editing host. Nó không đụng bước 1–3.
⇒ Hạ AC5 xuống *"native undo"* là khai đạt trên một thứ không chạy.

### 2.3 — Không có đường ghi nào đưa `retired_at` về `NULL`

Đo trên cây hiện tại: `retired_at` được **đọc** ở sáu chỗ *(bốn hàng rào từ chối ghi, một đường đọc
lưới, một đường đọc lịch sử)*, và được **đặt** ở đúng một chỗ — `write_regroup`. `core/i18n/mod.rs:215`
ghi bằng chữ: *"`retired_at` chỉ đặt được bằng SQL"*.

⇒ Đường (A) ở §3 đòi một **năng lực ghi chưa từng tồn tại**, không phải một lượt gọi lại hàm có sẵn.

---

## 3. Câu hỏi thứ nhất — hoàn tác **một lượt gộp** là gì trên đĩa?

Hai đường. Chúng cho **hai cái đĩa khác nhau**, và người dùng không phân biệt được trên màn hình.

| | **(A) Nghịch đảo thật** | **(B) Một lượt tách mới** |
|---|---|---|
| Việc làm | Gỡ `retired_at` của hai hàng cũ **và xoá** hàng mới | Chạy `split` trên hàng vừa gộp, tại đúng chỗ nối |
| `segment.id` cũ | **Quay lại** | **Không bao giờ quay lại** |
| Dữ liệu gắn theo id *(lịch sử phiên bản, trạng thái xác nhận, xuất xứ FR117, ghi nhớ proofreader)* | Còn nguyên | **Mất** — hai id thứ ba và thứ tư ra đời với lịch sử rỗng 🔵 *(2026-08-18: hàng này ĐÚNG nhưng **thiếu vế nặng nhất** — đường (B) còn không trả lại **chính bản dịch**. Xem §9.2.)* |
| Chỗ đánh dấu FR119 | Trỏ về đúng câu | Trỏ vào một câu **thứ ba** không ai từng thấy |
| Số hàng trên đĩa sau một vòng gộp→hoàn tác | 3 *(2 sống + 1 xoá)* → 2 sống | **5** hàng, 3 về hưu |

### 🔴 Điểm căng của câu hỏi, và nó không giải được bằng cách chọn "đường an toàn"

**Đường (A) đụng thẳng AD-3** (`SPINE:89-93`), nguyên văn:

> **Rule:** `segment.id` **bất biến, không tái dùng sau khi về hưu**.

Một hàng đã về hưu rồi sống lại là **đúng cái AD-3 cấm bằng chữ** — hoặc ít nhất là một ca AD-3
chưa xét. AD-3 nêu lý do là *định danh theo vị trí* và *định danh theo băm nội dung*; nó **không**
xét ca *"một lượt ghi bị hoàn tác trong cùng phiên"*. Winston phải phán định: đây là một **ngoại lệ
có tên** của AD-3, hay AD-3 đứng nguyên và (A) chết?

**Đường (B) đụng thẳng AD-5** (`:103-111`) — nhưng theo chiều ngược: nó **tuân thủ** AD-5 hoàn hảo
*(về hưu + tạo mới, lịch sử rỗng)*, và chính vì tuân thủ mà nó **mất dữ liệu người dùng**. AD-5 có
một câu về đúng chuyện này:

> *Chỗ đánh dấu khi đọc (FR119) trỏ tới segment về hưu thì **ở lại, không bị xoá im lặng*** — hiện
> kèm ghi chú *câu này đã đổi*.

⇒ Với (B), một lượt `⌘Z` biến **một** chỗ đánh dấu FR119 thành **hai** chỗ mang ghi chú *"câu này
đã đổi"*, cho một thao tác người dùng vừa **huỷ bỏ**. Đó là một sự thật sai trên màn hình.

⚠️ **Cả hai đường đều làm một lời hứa của spine kém đúng đi.** Đây là lý do nó là một `AD`, không
phải một lựa chọn cài đặt.

---

## 4. Câu hỏi thứ hai — `⌘Z` có phạm vi tới đâu?

AC5 chỉ nói về **một lượt gộp**. Nhưng một phím `⌘Z` trên bàn phím không mang phạm vi; người dùng
bấm nó **sau bất cứ việc gì**. Ba mức, mỗi mức là một `AD` khác hẳn:

| Mức | Phạm vi | Cái giá |
|---|---|---|
| **①** | **Chỉ** lượt gộp/tách vừa xảy ra, một bậc, hết hiệu lực khi rời segment | Hẹp nhất, cài được trong Epic 2. ⚠️ Người dùng bấm `⌘Z` sau khi **gõ** sẽ thấy **không gì xảy ra** — một `⌘Z` chạy nửa lúc là một `⌘Z` không tin được |
| **②** | Mọi thao tác **rời rạc** của Editor *(gộp · tách · cắt bỏ FR133 · ngắt đoạn FR134 · khôi phục FR101)* | Đòi một **nhật ký thao tác nghịch đảo được** — một cấu trúc chưa tồn tại, và mỗi thao tác phải khai hàm nghịch đảo của nó |
| **③** | ② **cộng** văn bản đang gõ | Đụng AD-35: bộ đệm gõ ở webview còn lượt ghi ở WAL. Hai đồng hồ, một phím |

🔴 **Và mức nào cũng phải trả lời một câu mà không mức nào tự trả lời:** `⌘Z` sống **bao lâu**?
Qua một lượt đóng/mở Tác phẩm? Qua một lượt đổi Chương? `.atproj` **không có bảng nhật ký thao tác**
và thêm một bảng như thế là một bước di trú *(số kế tiếp: **12**)* cộng một chủ sở hữu dữ liệu mới.

---

## 5. Ràng buộc cứng — `AD` phải đứng vừa trong bộ này

| Ràng buộc | Nguồn | Nó cấm gì |
|---|---|---|
| `segment.id` bất biến, không tái dùng | **AD-3** `:89-93` | Đường (A), trừ khi `AD` mới khai một ngoại lệ **có tên** |
| Gộp/tách = về hưu + tạo mới, lịch sử rỗng | **AD-5** `:103-111` | Một hàng "sống lại" mà vẫn tự nhận là chưa từng về hưu |
| Chỗ đánh dấu FR119 trỏ segment về hưu **ở lại** | **AD-5** | Đường (B) làm chỗ đánh dấu nhân đôi và nói sai |
| Về hưu do gộp/tách **không** tạo `segment_version` | **AD-31** `:381` | Dùng lịch sử phiên bản làm cơ chế undo |
| Mọi lệnh ghi qua **một** `store::Writer` nối tiếp | **AD-11** `:153-157` | Một đường ghi riêng cho undo |
| Lược đồ có phiên bản, di trú **chỉ tiến** | **AD-30** | Một bảng nhật ký thêm vào mà không qua bước di trú |
| Mọi quy tắc nghiệp vụ ở **Rust** | **AD-1** `:75-79` | Một ngăn xếp undo sống trong TypeScript |
| Mọi thao tác qua **`CommandRegistry`** | **AD-34 §1** | `⌘Z` cài thẳng trong `GridPanel.vue` |
| Flush trước mọi thao tác **rời rạc** | **AD-35** | Một lượt undo chạy trên bộ đệm gõ chưa xuống WAL |

⚠️ **Một ràng buộc kỹ thuật dễ bỏ sót:** `⌘Z` có `primaryMod` nên nó **không** bị `keys.ts:510` chặn
trong vùng gõ *(khác hẳn `Backspace` — xem §6 của story)*. Tức `Mod+Z` **đăng ký được** như một
command bình thường và sẽ bắn **cả khi con trỏ đang ở trong ô bản dịch**. Đây là một thuận lợi cho
mức ①/②, và là một **cái bẫy** cho mức ③.

---

## 6. Điều kiện nghiệm thu của `AD` này

1. Khuôn **Binds / Prevents / Rule** như 46 `AD` kia; `lint_spine.py` chạy sạch.
2. Trả lời **cả hai** câu hỏi: đường (A) hay (B) ở §3, **và** phạm vi ①/②/③ ở §4.
3. Nói rõ **AD-3 và AD-5 đổi gì** — hoặc khai bằng chữ rằng chúng **không đổi một chữ** *(khuôn
   AD-47 đã dùng với AD-31/AD-5)*.
4. Trả lời được **vòng đời**: `⌘Z` sống qua đóng/mở Tác phẩm không, và nếu có thì dữ liệu nằm ở đâu.
5. **Không** khai một bề mặt nhìn thấy mới cho Epic 2 — chữ ký #4(a) của Story 2.7 đang đứng.
6. Nếu `AD` chọn một đường đòi bảng mới ⇒ nói rõ nó là **bước di trú 12**, và ai sở hữu bảng đó.

---

## 7. Khuyến nghị phạm vi — năm AC giao được ngay, và chúng đứng được một mình

| AC | Nội dung | Phụ thuộc `AD-48`? |
|---|---|---|
| AC1 | Cử chỉ `Backspace` ở đầu ô ⇒ gộp | **Không** |
| AC2 | Kết quả xác định ở cả hai vế | **Không** — đã cài ở `regroup.rs`, đã khoá ở `segment_contract.rs` |
| AC3 | Đúng ngữ nghĩa AD-5 | **Không** — như trên |
| AC4 | Dòng báo hệ quả | **Không** — chủ là Story 2.9, ghi ở `deferred-work.md:4103-4109` |
| **AC5** | **`⌘Z`** | 🔴 **CÓ** |
| AC6 | Không chặn, không hỏi lại | **Không** |

⇒ **Khuyến nghị: giao 5/6 AC, ghi AC5 thành một món nợ có chủ.** Cử chỉ + ngữ nghĩa + dòng báo là
một lượt giao **có giá trị đứng một mình**: hôm nay người dùng bấm `Backspace` ở đầu ô thì **không
gì xảy ra**, và bấm ở câu đầu Chương cũng **không gì xảy ra** *(lượt từ chối `segment.no_previous`
chưa component nào đọc — đúng lớp "rỗng IM LẶNG" mà `project-context.md` cấm)*.

⚠️ **Năng lực chưa dựng ≠ lệch spec.** AC5 **không sai** vì đường đi chưa tới nó.
**Không sửa `epics.md`** cho khớp mã đã viết.

---

## 8. Thứ hồ sơ này **KHÔNG** làm

Không đề xuất một đường. Hai đường ở §3 được trình **cùng cái giá của chúng**, và cái giá nằm trên
đĩa người dùng — đó là loại quyết định `project-context.md:464-466` giao cho Ice và cho một `AD`,
không cho một lượt dev.

---

## 9. 🔵 Bổ sung 2026-08-18 — đo lại trên HEAD `91cfed1`, và MỘT VẾ BỊ THIẾU

**Vì sao có mục này:** Ice yêu cầu một hồ sơ đủ để phân định. Lượt đo lại cho thấy §1–§7 **đứng
nguyên**, nhưng bảng §3 **thiếu vế nặng nhất của đường (B)** — và vế đó đổi cái giá của nó về chất,
không về lượng.

### 9.1 Tiền đề của §1 — đo lại, **7/7 còn đúng**

| Phép đo | Brief *(2026-08-17, `4d72cd4`)* | Hôm nay *(2026-08-18, `91cfed1`)* |
|---|---|---|
| `grep -c "^### AD-"` | 47 | **47** ✅ |
| Cơ chế undo trong `src/` + `src-tauri/src/` | 0 | **0** ✅ *(1 dòng trúng là chú thích)* |
| Command `Mod+Z` | 0 | **0** ✅ |
| Nơi **ĐẶT** `retired_at` | 1 | **1** ✅ — `commands/segment.rs:2220` |
| Bảng tham chiếu `segment_id` | — | **1** — chỉ `segment_version` |
| Cột thật của `segment` | 13 | **13** ✅ *(8 gốc + `target_text` · `status` · `is_omitted` · `is_target_paragraph_end` · `translation_origin`)* |
| Bước di trú `project.db` kế tiếp | 12 | **12** ✅ *(đã tiêu: 1·2·3·5·6·7·8…)* |

⇒ **Không tiền đề nào của hồ sơ đã hết đúng.** Hai story chen vào giữa *(2.10, 2.11)* không chạm mô
hình hoàn tác.

### 9.2 🔴 VẾ BỊ THIẾU — đường (B) không trả lại chính **BẢN DỊCH**

Bảng §3 nói (B) mất *"lịch sử phiên bản, trạng thái xác nhận, xuất xứ, ghi nhớ proofreader"*. Đúng,
nhưng nó **giả định** hai hàng mới ít nhất mang lại **văn bản dịch cũ**. Đo trên `regroup.rs` thì
không:

| Đo | Kết quả |
|---|---|
| `split` chia gì | **chỉ `source_text`** — `chars[dau..cuoi]` *(`regroup.rs:278`)* |
| `target_text` của mảnh **đầu** | **toàn bộ** bản dịch của hàng bị tách *(`:280-281`)* |
| `target_text` của **mọi mảnh sau** | **RỖNG** |
| `translation_origin` của mọi mảnh sau | **`""`** *(`ORIGIN_NONE`, `:302-306`)* — có doc-comment giải thích và nó **đúng cho ca tách thường**: một mảnh chưa có bản dịch thì chưa có xuất xứ để khai |

⇒ **Một lượt `⌘Z` theo đường (B) trên một lượt gộp hai câu ĐÃ DỊCH cho ra:**
**một** segment giữ **toàn bộ** bản dịch của cả hai, và **một** segment **rỗng**.

🔴 **Đó không phải một lượt hoàn tác. Đó là một trạng thái sai nhìn thấy được trên màn hình** — và
nó nằm trên đĩa vĩnh viễn. Cái giá của (B) không phải *"mất siêu dữ liệu"*; nó là **mất bản dịch của
người dùng ngay trên bề mặt**.

⚠️ **Ghi rõ để không đọc thành một lỗi:** `split` cư xử **đúng** cho việc nó được dựng — tách một
câu nguồn chưa dịch. Vế thiếu là **không có** năng lực *"chia lại `target_text` tại chỗ nối"*, và
`⌘Z` theo đường (B) đòi đúng năng lực đó. Đây là một **năng lực chưa dựng**, không một khuyết tật.

⇒ **Đường (B) như mô tả ở §3 không cài được nếu không thêm một năng lực thứ ba** — và năng lực ấy
lại đòi biết **chỗ nối nằm ở đâu trong bản dịch**, một thông tin **không có trên đĩa** *(gộp nối hai
`target_text` và không lưu vị trí nối)*.

### 9.3 Hai vế của bảng §3 nhẹ hơn brief hàm ý — đo được

| Vế của §3 | Đo hôm nay |
|---|---|
| *"(B) mất **xuất xứ FR117**"* | ⚠️ **Nửa đúng.** Lượt **gộp** GIỮ xuất xứ *(`merge` → `merged_origin`, `regroup.rs:144-151`: mọi mảnh cùng giá trị ⇒ giữ; bất đồng ⇒ `ORIGIN_OTHER`)*. Cái mất nằm ở lượt **tách** ngược lại *(§9.2)*, và ở ca bất đồng thì xuất xứ riêng đã mất **ngay lúc gộp**, trước cả `⌘Z` |
| *"(B) làm hỏng **chỗ đánh dấu FR119**"* | ⚠️ **Cái giá TƯƠNG LAI, không phải hôm nay.** `grep -rniE "bookmark\|danh_dau\|needsReview" src/ src-tauri/src/` = **0**. FR119 thuộc **C1/Library**, chưa dựng. Vế này vẫn thật, nhưng nó tới ở Epic 5, không ở Epic 2 |

🔴 **Và một vế NẶNG HƠN brief nói:** `write_regroup` *(`commands/segment.rs:2234-2249`)* **không**
chèn `status` ⇒ hàng mới mặc định `'draft'`. Cộng với AD-31 *(gộp không tạo `segment_version`)*, một
lượt gộp **đã** làm mất trạng thái xác nhận và bỏ lại lịch sử ở hàng về hưu — **trước** khi ai bấm
`⌘Z`. ⇒ **Chỉ đường (A) trả lại được thứ chính lượt gộp đã lấy đi.** Đường (B) không có đường nào
với tới hai thứ đó, kể cả khi thêm năng lực chia lại `target_text`.

### 9.4 ⇒ Câu hỏi §3 nên đọc lại thành một câu khác

Sau khi đo, hai đường **không** phải hai cách cài cùng một tính năng. Chúng là **hai định nghĩa khác
nhau của `⌘Z`**:

| | Câu `⌘Z` thật sự hứa |
|---|---|
| **(A)** | *"Đưa đĩa về đúng trạng thái trước lượt gộp"* — gồm cả lịch sử và trạng thái xác nhận. Đòi một **ngoại lệ có tên của AD-3** |
| **(B)** | *"Tháo nhóm vừa gộp"* — không hứa khôi phục gì. Đòi thêm **năng lực chia lại `target_text`**, và cần một thông tin **không có trên đĩa** |

**Câu Winston cần Ice trả lời, gọn lại thành một:** `⌘Z` sau một lượt gộp là một lời hứa **khôi
phục**, hay một lời hứa **tháo nhóm**? Mọi thứ còn lại — AD-3 có ngoại lệ không, bước di trú 12 có
cần không, phạm vi ①/②/③ — đi theo câu đó.

### 9.5 Thứ mục này **KHÔNG** làm

**Không đề xuất một đường** — giữ nguyên §8. Số đo ở đây làm cái giá của (B) **nặng hơn** brief hàm
ý, nhưng *"đắt hơn"* không phải *"sai"*: `project-context.md:464-466` — *"đừng loại một phương án chỉ
vì nó đắt"*. Nếu Ice chọn (B) với lời hứa **tháo nhóm**, đó là một `AD` đứng được, và nó **rẻ hơn**
một ngoại lệ của AD-3.

**Không sửa `epics.md`** — khuyến nghị §7 *(giao 5/6 AC, AC5 thành nợ có chủ)* giữ nguyên.

---

## 10. ✅ CHỮ KÝ CỦA ICE — câu §3 đã chốt 2026-08-18

> ### ~~🖊️ Câu hỏi thứ nhất: đường (A) — NGHỊCH ĐẢO THẬT.~~ 🔵 **ĐÃ BỊ THAY — xem §11**
> ~~**Ice ký 2026-08-18.** `⌘Z` sau một lượt gộp là một lời hứa **KHÔI PHỤC**, không phải một lời hứa
> tháo nhóm. Đường (B) bị loại.~~
>
> 🔵 **Chữ ký này sống **vài giờ** và bị chính Ice thay cùng ngày bằng một đường **thứ ba** mà cả hồ
> sơ lẫn lượt đo đều chưa nêu: **rút `⌘Z` cho gộp/tách**. Giữ nguyên khối này thay vì xoá — lịch sử
> của một quyết định là bằng chứng cho quyết định kế tiếp. **Vế loại (B) vẫn đứng** và §9.2 vẫn là
> lý do loại nó.

**Đường bị loại và vì sao, ghi lại thay vì để trôi:** (B) rẻ hơn và **không** sai — nhưng §9.2 đo
được rằng nó cho ra *"một segment giữ toàn bộ bản dịch, một segment rỗng"*, và nó **không** với tới
được trạng thái xác nhận cùng lịch sử phiên bản mà chính lượt gộp đã lấy đi *(§9.3)*.

⇒ **`AD-48` phải khai một NGOẠI LỆ CÓ TÊN của AD-3** *(`SPINE:89-93` — `segment.id` bất biến, không
tái dùng sau khi về hưu)*. Đây là việc của **Winston**, không của một lượt dev.

### 10.1 (A) kéo theo gì — đo trên HEAD `91cfed1`

| Năng lực (A) đòi | Tồn tại hôm nay? | Bằng chứng |
|---|---|---|
| Đưa `retired_at` về `NULL` | **KHÔNG** | `retired_at` **đặt** ở đúng một chỗ *(`segment.rs:2220`)*; `core/i18n/mod.rs` ghi *"`retired_at` chỉ đặt được bằng SQL"* |
| Gỡ hàng gộp khỏi lưới | **KHÔNG** | xem 10.2 |
| Giữ được *"lượt gộp vừa rồi gồm những id nào"* | **KHÔNG** | không bảng nhật ký; xem 10.3 |

### 10.2 🔴 PHÁT HIỆN MỚI — (A) có HAI biến thể, và một trong hai phá một mệnh đề chưa ai viết ra

Brief §3 mô tả (A) là *"gỡ `retired_at` của hai hàng cũ **và xoá** hàng mới"*. Đo:

| Phép đo | Kết quả |
|---|---|
| `grep -rn "DELETE FROM segment" src-tauri/src/` | **0** — chưa từng có |
| Mọi `DELETE FROM` trong cả kho | **2**, và cả hai trên **dữ liệu dẫn xuất/cấu hình**: `config_value` *(`scope/store.rs`)* · `pinned_entry` *(`pinned.rs`)* |

⇒ **Kho chưa bao giờ xoá một dòng nội dung người dùng nào.** Mệnh đề đó **không được viết ra ở đâu**
— không `AD`, không `project-context.md` — nhưng nó **đúng trên toàn cây**, và (A) như mô tả sẽ là
lần đầu phá nó.

**Hai biến thể, Winston phải chọn một *(hoặc Ice, nếu Winston thấy nó là câu của Ice)*:**

| | **(A1) XOÁ hàng gộp** | **(A2) CHO HÀNG GỘP VỀ HƯU** |
|---|---|---|
| Việc | `DELETE` hàng mới, gỡ `retired_at` hai hàng cũ | `retired_at` cho hàng mới, gỡ `retired_at` hai hàng cũ |
| Đĩa sau một vòng gộp→`⌘Z` | **2** hàng, sạch như chưa từng gộp | **3** hàng *(2 sống + 1 về hưu)* |
| Phá mệnh đề *"không xoá nội dung người dùng"* | **CÓ** — lần đầu trong kho | **KHÔNG** |
| Còn dấu vết lượt gộp để lần ngược | **Không** | **Có** |
| Đụng AD-3 | Chỉ ở vế *"sống lại"* | Cùng vế, **cộng** một hàng về hưu chưa từng được ai xác nhận |

⚠️ **(A2) không tự động đúng hơn.** Nó để lại một hàng về hưu mà người dùng **chưa bao giờ thấy tồn
tại**, và mọi đường đọc lịch sử sẽ gặp nó. Ghi ra vì *"đường an toàn"* ở đây không miễn phí.

### 10.3 Câu §4 còn mở — nhưng nay phân định được bằng số

`AD-48` chưa đủ điều kiện nghiệm thu §6 mục 2 cho tới khi phạm vi ①/②/③ có chữ ký. Đo **năm** bề mặt
ghi rời rạc đang tồn tại, và **hai trong năm đã có nghịch đảo là một lệnh người dùng có sẵn**:

| Thao tác rời rạc | Lệnh | Nghịch đảo hôm nay |
|---|---|---|
| Gộp / tách / xoá dấu cắt | `editor.merge_segments` · `split_segment` · `clear_source_cuts` | 🔴 **Không** — đây chính là chỗ (A) phải dựng |
| Xác nhận segment | `editor.confirm_segment` | 🔴 **Không có lệnh bỏ xác nhận.** Và AD-31 nói lượt xác nhận **TẠO** một `segment_version` ⇒ hoàn tác nó đòi `DELETE` trên `segment_version` — **lại** đúng mệnh đề của §10.2 |
| Cắt bỏ câu (FR133) | `editor.omit_segment` ↔ `editor.restore_segment` | ✅ **Có sẵn, là một cặp** |
| Ngắt / nối đoạn đích (FR134) | `editor.end_target_paragraph` ↔ `editor.join_target_paragraph` | ✅ **Có sẵn, là một cặp** |
| Khôi phục phiên bản (FR101) | `restore_segment_version` | ⚠️ Ghi `target_text` **kèm mốc**, và là **ngoại lệ có tên duy nhất** của AD-47 |

⇒ **Mức ② rẻ hơn brief §4 hàm ý:** 2/5 bề mặt đã nghịch đảo được bằng lệnh đang chạy. Chỗ đắt thật
là **gộp/tách** *(đằng nào (A) cũng phải dựng cho mức ①)* và **xác nhận** *(đòi lần `DELETE` thứ hai
trên nội dung người dùng)*.

**Và câu vòng đời của §4 nay có một câu trả lời đo được cho mức ①:** thứ cần giữ để hoàn tác **một**
lượt gộp là *"id hàng mới + id hai hàng cũ"*. Giữ nó **trong bộ nhớ tiến trình Rust**, phạm vi một
phiên, thì **không cần bảng mới và không cần bước di trú 12** — AD-1 cho phép *(quy tắc nghiệp vụ ở
Rust)*, AD-11 không đụng tới *(mọi lượt ghi vẫn qua một `Writer`)*.
🔴 Chỉ khi phạm vi là **②/③** hoặc `⌘Z` phải **sống qua một lượt đóng/mở Tác phẩm** thì bảng nhật ký
— và bước **12** — mới trở thành bắt buộc.

### 10.4 Winston cần gì để soạn `AD-48`

1. ✅ Câu §3 — **(A)**, Ice ký 2026-08-18.
2. 🔲 Câu §4 — phạm vi **①/②/③**, và `⌘Z` có sống qua đóng/mở Tác phẩm không.
3. 🔲 Biến thể **(A1) xoá** hay **(A2) về hưu** *(§10.2)* — và nếu (A1), khai bằng chữ rằng đây là
   lần đầu kho xoá nội dung người dùng.
4. Khai **AD-3 đổi gì**, hoặc rằng nó **không đổi một chữ** và đây là một ngoại lệ có tên *(khuôn
   AD-47 đã dùng với AD-31/AD-5)*.
5. Khai **AD-5 đổi gì** — một hàng đã về hưu sống lại là ca AD-5 chưa xét.
6. Nếu phạm vi đòi bảng mới ⇒ nói rõ **bước di trú 12** và ai sở hữu bảng đó.
7. Giữ ràng buộc §6 mục 5: **không** khai một bề mặt nhìn thấy mới cho Epic 2 *(chữ ký #4(a) của
   Story 2.7 đang đứng)*.

🔵 **§10.4 ĐÃ HẾT ĐÚNG từ §11.** Bảy mục trên viết cho một `AD` phải **chọn giữa (A) và (B)**. Quyết
định cuối rút cả hai. Danh sách còn hiệu lực nằm ở **§11.4**.

---

## 11. ✅ QUYẾT ĐỊNH CUỐI — đường (C): RÚT `⌘Z` cho gộp/tách. Ice ký 2026-08-18

> ### 🖊️ **Không dựng mô hình hoàn tác cho gộp/tách. AC5 của Story 2.9 RÚT.**
> Gộp và tách **đã là lệnh người dùng**. Muốn quay lại thì gọi lại chính chúng.
> **Ice ký 2026-08-18.** Thay chữ ký (A) ở §10.

**Đường thứ ba này do Ice nêu**, sau khi §9 và §10 đo xong cái giá của (A) và (B). Hồ sơ **không**
nghĩ ra nó — ghi ra vì đó là một dữ kiện về cách quyết định này được tìm thấy.

### 11.1 Bốn phép đo chống lưng

| # | Phép đo | Kết quả |
|---|---|---|
| ① | `grep -rniE "undo\|hoàn tác\|⌘Z" prd.md` | Chỉ trúng `undock` *(FR17)*. **Không FR nào đòi hoàn tác** |
| ② | `EXPERIENCE.md:169` *(Ice ký 2026-08-17)* | *"…trên dữ liệu mà **AD-5 không cho hoàn tác**"* |
| ③ | `vi.json:101` — dòng báo **đang chạy** | *"Đã gộp hai câu. Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được."* — **không hứa** hoàn tác |
| ④ | `EXPERIENCE.md:171` | Chỗ **duy nhất** hứa `⌘Z`, và nó là một **mẩu sót** — xem 11.2 |

### 11.2 🔴 Lời hứa `⌘Z` chưa bao giờ là một quyết định

`EXPERIENCE.md:171` mang dấu 🔵 **SỬA 2026-08-17 (Story 2.8)**: lượt ấy thay **cử chỉ kích hoạt**
*(gõ đè → `Backspace` ở đầu ô)* nhưng **để nguyên** vế `hoàn tác bằng ⌘Z` ở cuối câu.

⇒ Đúng khuôn **F1** của retro Epic 2 — *"chữ ký thi hành ĐÚNG MỘT NỬA, lặp năm lần; nửa rơi luôn là
nửa RẺ: một dòng chuỗi hoặc một câu phải xoá."* AC5 **kế thừa** mẩu sót đó. Rút nó là **đóng nốt nửa
còn lại**, không cắt một tính năng đã cân nhắc.

### 11.3 Tiền đề của Ice — kiểm, và nó ĐÚNG

| Phép đo | Kết quả |
|---|---|
| Lượt gộp có phá `target_text` không | **Không** — `join_targets(parts)`, **nối** *(`regroup.rs:186`)* |
| Tách lại thì văn bản đi đâu | Mảnh đầu giữ **toàn bộ**, mảnh sau rỗng *(`regroup.rs:280-281`)* ⇒ cắt/dán được |
| Lịch sử hai câu cũ | **Vẫn đọc được** *(`lib.rs:346`)* |

⇒ **Không byte nào bị phá.** Mất **công sức thao tác**, không mất dữ liệu.

🔵 **Đính chính §9.2 của chính hồ sơ này:** ở đó mình gọi kết quả tách-lại là *"một trạng thái sai
nhìn thấy được"*. Đúng **với một nút dán nhãn "hoàn tác"** — nó hứa khôi phục rồi giao thứ khác.
Cùng trạng thái ấy khi **người dùng tự làm** thì không lời hứa nào bị phá. **Khác biệt nằm ở LỜI HỨA,
không ở byte.** Mệnh đề §9.2 **vẫn đúng cho đường (B)** và vẫn là lý do loại (B).

### 11.4 `AD-48` — vẫn cần, nhưng đổi hẳn kích thước

Winston soạn **một** mệnh đề: *Epic 2 không có mô hình hoàn tác; gộp/tách không hoàn tác được; đường
quay lại là gọi lại chính lệnh gộp/tách — và đây là lý do.*

**Vì sao vẫn là một `AD`:** Epic 3 trở đi còn thêm thao tác rời rạc *(duyệt glossary hàng loạt, điền
sẵn từ TM, đề xuất AI)*. Không viết ra thì câu *"`⌘Z` làm gì"* quay lại ở **mỗi** epic.

**Không còn cần:** ngoại lệ AD-3 · đường `DELETE` trên nội dung người dùng · năng lực `retired_at →
NULL` · bảng nhật ký · bước di trú 12. AD-3, AD-5, AD-31 **không đổi một chữ**.

### 11.5 Chỗ hở, ghi ra

Bấm `⌘Z` **không có gì xảy ra và không phản hồi nào** *(`grep "KeyZ" src/commands/` = 0)* — đúng lớp
**"rỗng IM LẶNG"**. Ghi thành một món nợ **có chủ là Ice**, không tự chọn hình dạng.

**Thi hành:** Sprint Change Proposal **2026-08-18b**.
