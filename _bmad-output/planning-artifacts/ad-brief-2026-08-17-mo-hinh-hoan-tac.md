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
