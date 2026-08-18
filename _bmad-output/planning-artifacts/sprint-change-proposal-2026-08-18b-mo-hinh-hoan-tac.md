# Sprint Change Proposal — 2026-08-18b: rút `⌘Z` cho gộp/tách

**Kích hoạt bởi:** Ice, 2026-08-18 — sau khi hồ sơ `AD-48` §9/§10 đo xong cái giá của cả hai đường
**Phân loại phạm vi:** 🟡 **Moderate** — rút một AC đã phát hành trong spec, sửa hai tài liệu quy hoạch
**Thay thế:** chữ ký **(A)** ở §10 của `ad-brief-2026-08-17-mo-hinh-hoan-tac.md` *(cùng ngày, đã bị thay)*

---

## 1. Tóm tắt vấn đề

**AC5 của Story 2.9 — *"gộp vừa xảy ra · bấm `⌘Z` · hoàn tác được"* — đòi một mô hình hoàn tác mà
dự án chưa bao giờ quyết định, và cả hai đường cài đặt đều đắt hơn giá trị nó mang lại.**

Ice chốt: **rút `⌘Z` cho gộp/tách.** Gộp và tách **đã là lệnh người dùng có sẵn**; muốn quay lại thì
người dùng tự gộp/tách lại.

### Vì sao đây không phải một lượt cắt cho rẻ — bốn phép đo

| # | Phép đo | Kết quả |
|---|---|---|
| ① | `grep -rniE "undo\|hoàn tác\|⌘Z" prd.md` | Chỉ trúng chữ `undock` *(FR17)*. **Không FR nào đòi hoàn tác** |
| ② | `EXPERIENCE.md:169` *(Ice ký 2026-08-17)* | Viết thẳng: *"…trên dữ liệu mà **AD-5 không cho hoàn tác**"* |
| ③ | Dòng báo đang chạy, `vi.json:101` | *"Đã gộp hai câu. Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được."* — nói đúng hệ quả, **không hứa hoàn tác** |
| ④ | `EXPERIENCE.md:171` | Chỗ **DUY NHẤT** hứa `⌘Z`. Và nó là một **mẩu sót** — xem §2 |

⇒ Lời hứa `⌘Z` tồn tại ở **đúng một câu văn xuôi**, mâu thuẫn với một câu khác **trong cùng tệp**,
và **không** được bất kỳ FR nào chống lưng.

---

## 2. 🔴 Lời hứa `⌘Z` là một mẩu SÓT, không một quyết định

`EXPERIENCE.md:171` mang dấu 🔵 **SỬA 2026-08-17 (Story 2.8)**. Lượt sửa ấy thay **cử chỉ kích hoạt**
*(gõ đè lên ranh giới → `Backspace` ở đầu ô)* sau khi Sprint Change Proposal 2026-08-14 bác tiền đề
cũ — nhưng nó **để nguyên** mệnh đề `hoàn tác bằng ⌘Z` ở cuối câu.

🔴 **Đây đúng khuôn F1 của retro Epic 2** — *"một chữ ký được thi hành ĐÚNG MỘT NỬA, lặp năm lần;
nửa khó có chú thích đẹp thì làm, nửa là **một dòng chuỗi** hoặc **một câu phải xoá** thì rơi. Không
cổng nào canh nửa đó."*

⇒ AC5 của Story 2.9 **kế thừa** lời hứa sót ấy. Rút nó là **đóng nốt nửa còn lại** của một lượt sửa
đã ký, không phải cắt một tính năng đã được cân nhắc.

⚠️ **Và luật *"năng lực chưa dựng ≠ lệch spec"* KHÔNG áp ở đây** *(`project-context.md:456-458`)*.
Luật đó bảo vệ một AC mô tả **đích đến** khi đường đi chưa tới. Ca này khác: đây là **quyết định
không dựng**, có chữ ký và có lý do đo được. ⇒ Nó phải đi qua correct-course, **không** được ghi
thành một món nợ rồi để đó.

---

## 3. Phân tích ảnh hưởng

### 3.1 Tiền đề của Ice — kiểm, và nó ĐÚNG

*"Tách gộp đã có sẵn rồi, người dùng có thể tự mình gộp/tách lại."*

| Phép đo | Kết quả |
|---|---|
| Lượt gộp có phá `target_text` không | **Không** — `merge` gọi `join_targets(parts)`, **nối**, không xoá *(`regroup.rs:186`)* |
| Tách lại thì văn bản đi đâu | Mảnh **đầu** giữ **toàn bộ**; mảnh sau **rỗng** *(`regroup.rs:280-281`)* |
| Lịch sử hai câu cũ | **Vẫn đọc được** — `restore_segment_version` cố ý **không** từ chối segment đã về hưu *(`lib.rs:346`)* |

⇒ **Không byte nào của người dùng bị phá.** Cái mất là **công sức thao tác** *(tách lại, rồi cắt/dán
nửa sau sang ô dưới)*, không phải dữ liệu.

🔵 **Đính chính một mệnh đề của chính hồ sơ này:** §9.2 mô tả kết quả ấy là *"một trạng thái sai nhìn
thấy được"*. Điều đó đúng **với một nút dán nhãn "hoàn tác"** — nó hứa khôi phục rồi giao thứ khác.
Cùng trạng thái ấy khi **người dùng tự làm** thì không lời hứa nào bị phá. **Khác biệt nằm ở LỜI
HỨA, không ở byte.**

### 3.2 Cái (C) đóng lại — mọi thứ (A) vừa mở ra

| (A) đòi | (C) |
|---|---|
| Một **ngoại lệ có tên của AD-3** *(`segment.id` sống lại)* | **Không cần** |
| Đường `DELETE` **đầu tiên** trên nội dung người dùng, hoặc một hàng về hưu người dùng chưa từng thấy | **Không cần** |
| Năng lực đưa `retired_at` về `NULL` — **chưa từng tồn tại** | **Không cần** |
| Bảng nhật ký + **bước di trú 12** *(nếu phạm vi ②/③)* | **Không cần** |
| AD-5 phải xét lại ca *"hàng về hưu sống lại"* | **Không đổi một chữ** |

### 3.3 Artifact bị ảnh hưởng

| Artifact | Ảnh hưởng | Trạng thái |
|---|---|---|
| **`epics.md`** — AC5 của Story 2.9 | Rút, kèm 🔵 + ngày + lý do | **`[!]`** — §4.1 |
| **`EXPERIENCE.md:171`** | Bỏ mệnh đề *"hoàn tác bằng `⌘Z`"* | **`[!]`** — §4.2 |
| `prd.md` | **Không** — không FR nào đòi hoàn tác *(§1 ①)* | `[N/A]` |
| `ARCHITECTURE-SPINE.md` | **`AD-48` vẫn nên tồn tại, nhưng NHỎ hơn nhiều** — xem §5 | `[!]` → Winston |
| `2-9-…md` *(story, `done`)* | Ghi chú AC5 đã rút | **`[!]`** — §4.3 |
| `deferred-work.md` | Ba mục `⌘Z`/`AD-48` *(`:4174` · `:4220` · `:4270`)* đóng theo quyết định; **một mục MỚI** cho chỗ hở §6 | **`[!]`** — §4.4 |
| Mã sản phẩm | **KHÔNG CHẠM.** `grep "KeyZ" src/commands/` = 0 — chưa từng có gì để gỡ | `[N/A]` |

### 3.4 MVP

**Không ảnh hưởng.** Không FR nào bị đụng.

---

## 4. Đề xuất thay đổi chi tiết

### 4.1 `epics.md` — Story 2.9, AC5

**OLD**
```
**Given** gộp vừa xảy ra
**When** người dùng bấm `⌘Z`
**Then** hoàn tác được
```

**NEW**
```
> 🔵 **AC5 RÚT 2026-08-18** *(Ice ký; Sprint Change Proposal 2026-08-18b)*. AC cũ:
> *"gộp vừa xảy ra · bấm `⌘Z` · hoàn tác được"*.
>
> **Vì sao rút, đo được:** ① không FR nào đòi hoàn tác — `grep "undo|hoàn tác|⌘Z" prd.md`
> chỉ trúng chữ `undock`; ② `EXPERIENCE.md:169` *(Ice ký 2026-08-17)* đã viết thẳng
> *"trên dữ liệu mà **AD-5 không cho hoàn tác**"*; ③ dòng báo đang chạy nói đúng hệ quả và
> **không hứa** hoàn tác. Lời hứa `⌘Z` tồn tại ở **đúng một câu văn xuôi**
> *(`EXPERIENCE.md:171`)*, và câu đó là một **mẩu sót** của lượt sửa 2026-08-17 — đúng khuôn
> **F1** của retro Epic 2 *(chữ ký thi hành đúng một nửa)*.
>
> **Đường thay thế, đã có sẵn:** gộp và tách **là lệnh người dùng**. Muốn quay lại thì gộp/tách
> lại — không byte nào bị phá *(lượt gộp **nối** `target_text`, lịch sử hai câu cũ vẫn đọc
> được)*; cái mất là **công sức thao tác**, không phải dữ liệu.
>
> ⚠️ Đây **không** phải *"năng lực chưa dựng"* — đây là **quyết định không dựng**, nên nó đi
> qua correct-course chứ không thành một món nợ.
```

**Rationale:** giữ AC cũ nguyên văn trong khối trích để lịch sử đọc được, và ghi lý do rút tại chỗ
thay vì để nó biến mất — cùng luật mà sổ nợ áp cho một mục đã đóng.

### 4.2 `EXPERIENCE.md:171`

**OLD** *(cuối câu)*
```
Một dòng báo ở lề, hoàn tác bằng `⌘Z`. **Không chặn, không hỏi lại**
```

**NEW**
```
Một dòng báo ở lề. 🔵 *(SỬA 2026-08-18: mệnh đề "hoàn tác bằng `⌘Z`" RÚT — Ice ký, SCP
2026-08-18b. Nó là mẩu sót của chính lượt sửa 2026-08-17 ở dòng này, và nó mâu thuẫn với
`:169` trong cùng tệp: "trên dữ liệu mà AD-5 không cho hoàn tác". Muốn quay lại thì gộp/tách
lại — cả hai là lệnh có sẵn.)* **Không chặn, không hỏi lại**
```

### 4.3 `2-9-gop-bang-backspace-dau-o.md` *(story đã `done`)*

Thêm một khối 🔵 ở mục AC5 *(`:75`)* và ở Task 0 *(`:140`)*: cửa chặn `AD-48` **đã được phân giải
bằng cách RÚT AC**, không bằng một mô hình. Không đổi `Status: done`.

### 4.4 `deferred-work.md`

- `:4174` vế `⌘Z` · `:4220` *(tách đa-mảnh)* · `:4270` *(`AC5`/`AD-48`)* — đóng bằng **nối tiếp**
  `→ ✅ ĐÃ ĐÓNG 2026-08-18 (SCP 2026-08-18b — AC5 rút)`, **không xoá**.
- **MỘT MỤC MỚI** cho chỗ hở §6, kèm chủ.

---

## 5. `AD-48` — vẫn nên tồn tại, nhưng đổi hẳn kích thước

Rút AC5 **không** làm câu hỏi biến mất; nó làm câu trả lời rẻ đi. `AD-48` nay chỉ cần khai **một**
mệnh đề:

> **Epic 2 không có mô hình hoàn tác. Gộp/tách không hoàn tác được — đường quay lại là gọi lại chính
> lệnh gộp/tách. Và đây là lý do.**

**Vì sao vẫn cần một `AD` chứ không một dòng chú thích:** Epic 3 trở đi còn thêm thao tác rời rạc
*(duyệt glossary hàng loạt, điền sẵn từ TM, đề xuất AI)*. Không viết ra thì câu *"`⌘Z` làm gì"* quay
lại ở **mỗi** epic, và mỗi lần lại phải đo lại từ đầu — đúng chi phí mà 47 `AD` kia tồn tại để tránh.

**Chủ: Winston.** 🔴 Dev **không** tự soạn `AD`.

⚠️ `AD` này phải nói rõ nó **không đổi một chữ** của AD-3, AD-5, AD-31 *(khuôn AD-47 đã dùng)*.

---

## 6. Chỗ hở (C) để lại — ghi ra thay vì để trôi

**Bấm `⌘Z` sẽ không có gì xảy ra, và không phản hồi nào.** `grep "KeyZ" src/commands/` = **0**.

🔴 Đó đúng lớp **"rỗng IM LẶNG"** mà `project-context.md:473-499` cấm: *"một danh sách rỗng không tự
nói vì sao nó rỗng"*. Một phím mà cả thế giới phần mềm gán nghĩa "hoàn tác", bấm vào không phản hồi,
là một màn hình **im lặng về một điều nó biết**.

**Hai đường đóng, chưa chốt:** ① một dòng báo ở `StatusBar` — *"Gộp/tách không hoàn tác được — tách
lại rồi chuyển chữ sang ô dưới"*; ② để nguyên và ghi một món nợ có chủ.

⇒ §4.4 ghi nó thành **một món nợ có chủ**, không tự chọn.

---

## 7. Bàn giao

| Việc | Chủ | Điều kiện xong |
|---|---|---|
| §4.1 · §4.2 · §4.3 · §4.4 | Dev *(lượt này)* | `grep "⌘Z" epics.md` ở Story 2.9 = 0 lời hứa; `EXPERIENCE.md` không còn mâu thuẫn `:169` ↔ `:171` |
| `AD-48` bản nhỏ *(§5)* | **Winston** | Có mặt trong `ARCHITECTURE-SPINE.md`, `lint_spine.py` sạch |
| Chỗ hở §6 | **Ice** *(chốt hình dạng)* | Một dòng báo, hoặc một quyết định để nguyên viết ra bằng chữ |

**Tiêu chí thành công:** một người đọc `EXPERIENCE.md`, `epics.md` và sản phẩm nhận **cùng một câu**
về `⌘Z` — hôm nay ba nguồn nói ba điều khác nhau.
