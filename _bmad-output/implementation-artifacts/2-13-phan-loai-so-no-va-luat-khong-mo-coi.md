---
baseline_commit: 4b30199263b5a5ad9dd13d3a4c8cd810951d1ba7
---

# Story 2.13: Phân loại sổ nợ — 199 mục mồ côi và một luật đang bị phá

Status: ready-for-dev

**Covers:** **không FR nào.** Story hạ tầng — đóng một **vi phạm luật đang sống**, không thêm năng lực.
**Epic:** 2 — Biên tập theo segment
**Soạn:** 2026-08-18 · trên HEAD `4b30199`, cây làm việc **sạch**
**Thứ tự:** 🔴 **Sau Story 2.4 VÀ sau Story 2.12** *(Ice chốt 2026-08-18)*. Không phải cửa chặn Epic 3.

---

## 🔴 ĐỌC TRƯỚC DÒNG ĐẦU TIÊN — luật này đã tồn tại, và nó đang bị phá 199 lần

`project-context.md:447-448` viết bằng chữ, không phải một gợi ý:

> *"Mọi thứ không nghiệm thu được ở story hiện tại đi vào đây, **KÈM MỘT CHỦ** (story nào sẽ đóng).
> **Không có mục nào mồ côi.**"*

**Đếm thật trên cả 4.691 dòng của `deferred-work.md`, 2026-08-18, HEAD `4b30199`:**

| | mở | nửa 🟡 | đóng ✅ | tổng | mở **có chủ** | mở **KHÔNG chủ** |
|---|---|---|---|---|---|---|
| Epic 1 | 197 | 23 | 16 | **236** | 41 | **156** |
| Epic 2 | 81 | 13 | 13 | **107** | 65 | **16** |
| khác *(correct-course · retro · nghiệm thu tay)* | 83 | 14 | 8 | **105** | 56 | **27** |
| **TỔNG** | **361** | **50** | **37** | **448** | **162** | **199** |

**83** khối `## Deferred from:`. Tỉ lệ đóng trên cả sổ: **37/448 = 8,3 %**.

⚠️ **Retro Epic 2 không sai — nó đếm một KHOANG.** Con số *"~157 mục, 22 đóng"* *(`epic-2-retro-2026-08-18.md:361-364`)*
là **khoảng Epic 2**. Cả sổ lớn gấp gần **ba lần** khung đó. Ghi ra thay vì để hai con số cùng tồn tại
mà không ai nói chúng đo hai thứ khác nhau.

### 🔴 Và đây là phát hiện lật hướng xử lý — quy ước `Chủ:` ĐANG HOẠT ĐỘNG

| Epic | Mục mở có chủ |
|---|---|
| **Epic 2** | **65/81 = 80 %** |
| **Epic 1** | **41/197 = 21 %** |

⇒ Vấn đề **KHÔNG** phải *"sổ nợ thiếu một cơ chế"*, và **KHÔNG** phải *"cần một story dọn nợ định kỳ
mỗi cuối epic"*. Cả hai giải một bài toán mà **số đo nói là đã tự giải**.

Nó là **một khối di sản dùng một lần**: 156 mục Epic 1 được viết **trước khi luật `Chủ:` tồn tại**,
cộng 43 mục lẻ ở hai khoang sau. Epic 2 đã tự lành mà không cần ai can thiệp.

🔴 **Hệ quả cho phạm vi story này:** nó là **một lượt phân loại**, không phải một cơ chế mới. Đừng
dựng một quy trình định kỳ cho một vấn đề dùng một lần — đó là thêm một luật mà số đo không đòi.

### 199 mục ấy, đo theo chủ đề

| Số | Nhóm | Chủ tự nhiên đã tồn tại |
|---|---|---|
| **40** | font / từ điển | phần lớn có thể **đã đóng** ở 1.10b · 1.10c · 1.11b — chỉ chưa đánh dấu |
| **29** | Windows / nền tảng | → **B7** *(bảng nghiệm thu Windows, chủ Ice)* |
| **10** | nghiệm thu tay / bàn đo | → **B10** *(chưa có chủ, chưa có lịch)* |
| **7** | đo lại một con số | |
| **113** | **cần ĐỌC TAY và giao chủ thật** | — |

**113 mục kia bám vào bề mặt nào:**

| Số | Bề mặt |
|---|---|
| 33 | cổng `check-*.mjs` *(ví dụ `check-i18n.mjs:455-499` `scanStyle` không có trạng thái `line_comment`; `check-deps.mjs:95-99` `walk()` đệ quy không có bộ nhớ đã-thăm)* |
| 17 | test Rust / vitest |
| 17 | mã sản phẩm `src/` |
| 6 | tài liệu / spec *(ví dụ `.memlog.md` của architecture còn `scope: 112 FR, 16 NFR`, spine ghi 131/19)* |
| 4 | CI / workflow *(ví dụ `ci.yml` `on: push` + `on: pull_request` ⇒ ma trận chạy **hai lần** mỗi commit)* |
| **36** | **không bám vào một bề mặt nào** — nhóm khó nhất, đọc từng mục |

🔴 **86 mục có chủ tự nhiên KHÔNG phải 86 mục đã xong.** Giao chủ là một thao tác; *"đóng"* là một
thao tác khác và nó đòi một phép kiểm. `project-context.md:334-335`: **không đánh dấu đạt bằng suy luận.**

---

## Story

As a **người duy trì AuraTranslate**,
I want **mọi món nợ đang mở có một chủ đọc được**,
So that **sổ nợ trở lại là một hàng đợi có người phục vụ, thay vì một kho trung thực mà không ai đọc**.

---

## Acceptance Criteria

*(Story không có AC trong `epics.md`. Sáu AC dưới đây dẫn xuất từ luật `project-context.md:445-453`
và từ bảng đo ở §ĐỌC TRƯỚC.)*

1. **AC1** — **Given** `deferred-work.md` sau lượt này · **When** đếm bằng lệnh của AC5 · **Then**
   số mục **MỞ mà không có `Chủ:`** = **0**.

2. **AC2** — **Given** một mục được chuyển sang `✅ ĐÃ ĐÓNG` · **When** đóng · **Then** kèm **một
   phép kiểm đã chạy** *(lệnh + kết quả + ngày)*, **không** một suy luận từ việc story liên quan đã
   `done`. 🔴 Áp trực tiếp cho **40** mục font/từ điển — *"nhiều khả năng đã đóng"* **không phải** một
   bằng chứng.

3. **AC3** — **Given** một mục thuộc nhóm có chủ tự nhiên *(29 Windows → B7 · 10 nghiệm thu tay → B10)* ·
   **When** giao chủ · **Then** giao bằng **một tham chiếu**, **không chép nội dung mục sang chỗ khác**
   — một mục sống ở đúng một chỗ.

4. **AC4** — **Given** bất kỳ mục nào · **When** xử lý · **Then** **không mục nào bị XOÁ**. Đóng bằng
   **nối tiếp** `→ ✅ ĐÃ ĐÓNG <ngày> (Story 2.13)`; đóng một nửa ghi **🟡 kèm phần còn hở**; mệnh đề
   đã hết đúng thì **gạch ngang**, không xoá *(`project-context.md:449-453`)*.

5. **AC5** — **Given** ai đó muốn kiểm lại con số · **When** chạy **một lệnh** ghi trong story ·
   **Then** ra đúng bảng của §ĐỌC TRƯỚC trước lượt này, và bảng mới sau lượt này. 🔴 **Số đo phải
   tái lập được** — *"số đo không truy nguyên được thì không phải số đo"*.

6. **AC6** — **Given** một story tương lai thêm một mục nợ **không có chủ** · **When** cơ chế của chữ
   ký **#3** chạy · **Then** nó **bị bắt**. *(Hình dạng cơ chế — cổng tĩnh, hay một dòng luật, hay
   không làm gì — là **chữ ký #3**, chưa chốt.)*

---

## 🔴 Task 0 — CỬA CHẶN: sáu quyết định mở, phải có chữ ký của Ice

**Không một mục nào được sửa trước khi sáu mục dưới đây có chữ ký.**
Trình mỗi quyết định **kèm số đo**, không kèm một khuyến nghị đã tự chốt.

🔴 **Task 0.4 — cửa `AD`:** story này **không** chạm một bất biến kiến trúc nào. Nhưng chữ ký **#4**
*(thêm một trạng thái thứ tư cho sổ nợ)* là một **đổi quy ước kho**, và quy ước kho sống ở
`project-context.md`. Nếu #4 = có: **dừng**, nêu với Ice như một lượt sửa luật riêng, **đừng tự thêm
một ký hiệu mới rồi dùng nó**.
⚠️ Luật `project-context.md:361-364` cấm đúng việc đó: *"Đừng bắt chước một ký hiệu chưa hiểu… Không
có định nghĩa ⇒ nêu với Ice kèm số đo."*

---

### Quyết định #1 🔴 — Phạm vi: 199 mục cả sổ, hay 156 mục Epic 1

**Số đo:** mồ côi phân bố **156 Epic 1 · 16 Epic 2 · 27 khác**.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Cả **199** | AC1 phát biểu được thành *"= 0"* — một mệnh đề **kiểm được bằng máy**. Đắt nhất |
| **(b)** | Chỉ **156** Epic 1 | Đúng chỗ khối di sản nằm. Nhưng AC1 thành *"= 43"*, một con số **không ai nhớ được vì sao**, và cổng của chữ ký #3 mất mốc |
| **(c)** | **199**, nhưng 43 mục ngoài Epic 1 chỉ **giao chủ**, không xét đóng | Giữ mệnh đề *"= 0"* với chi phí gần (b) |

---

### Quyết định #2 🔴 — 40 mục font/từ điển: ai kiểm, và kiểm bằng gì

**Số đo:** 40 mục, nhóm lớn nhất. Nhiều mục có vẻ đã được 1.10b · 1.10c · 1.11b đóng — **nhưng chưa
ai kiểm**.

🔴 **Đây là chỗ AC2 dễ bị phá nhất trong cả story.** Cám dỗ: story đóng nó rồi ⇒ đánh ✅ cả 40. Đó
đúng nghĩa *"đánh dấu đạt bằng suy luận"*, và nó biến một sổ trung thực thành một sổ **nói dối** — tệ
hơn hẳn tình trạng hôm nay.

**Ba câu phải trả lời riêng:** kiểm bằng đường nào *(chạy `check:dict` · đọc `dict-manifest.toml` ·
mở `.db`)* · mục nào **không kiểm được** vì cần dữ liệu từ điển mà runner không có · và mục không
kiểm được thì **🟡 hay giữ mở**.

---

### Quyết định #3 🔴 — Cơ chế cho AC6: cổng tĩnh, một dòng luật, hay không làm gì

**Số đo:** hôm nay **9** cổng đọc-tệp trong `pre-push`. Story 2.12 có thể thêm **1-2** nữa *(chữ ký #7
của nó)*. Cổng này sẽ là cái **thứ 12** — và nó khác mọi cổng đang có ở một điểm: **nó đọc một tệp
tài liệu, không đọc mã.**

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Một cổng `check:debt-owner` | *"Cưỡng chế bằng lệnh, không bằng kỷ luật"*. Nhưng: một cổng trên **văn xuôi tiếng Việt** phải đoán *"đâu là một mục"* — chính bộ đếm của story này đã phải sửa **hai** lần *(regex bắt nhầm ngày `2026-08-17` thành `Epic 2026`; và `Chủ:` viết nhiều dạng)*. Một cổng đỏ oan trên sổ nợ sẽ bị tắt trong một tuần |
| **(b)** | Một dòng luật trong `project-context.md`, không cổng | Rẻ. Nhưng luật **đã có sẵn ở `:448`** và nó đã bị phá 199 lần ⇒ thêm một dòng nữa là làm **đúng thứ đã chứng minh là không đủ** |
| **(c)** | Không cơ chế nào; AC6 rút | Trung thực nếu (a) chưa đo được. Phải ghi thành một món nợ **có chủ**, không im lặng bỏ |

⚠️ **Tiền lệ đo được, phải cân:** `deferred-work.md:4654-4674` — lớp lỗi `editorHasLoaded` đã hụt
**hai** lần và Ice chọn **đóng bằng một dòng trong `project-context.md`, KHÔNG một cổng**, kèm điều
kiện mở lại: *"nếu lớp lỗi này hụt lần thứ ba, bằng chứng đã đủ để không cần bàn nữa — dựng phép kiểm."*
Ở đây con số là **199**, không phải 2.

---

### Quyết định #4 — Sổ nợ có cần một trạng thái thứ TƯ không

**Số đo:** hôm nay sổ có **ba** trạng thái — mở · **🟡** một nửa · **✅ ĐÃ ĐÓNG**. `grep` cho
*"BÁC / bác bỏ / WONT FIX"* = **3** kết quả lẻ, **không** một quy ước.

**Câu hỏi thật:** trong 113 mục cần đọc tay, chắc chắn có mục mà câu trả lời đúng là *"không làm, và
đây là lý do"*. Hôm nay **không có chỗ để viết câu đó** — nó buộc phải nằm ở *"mở"* mãi mãi, hoặc bị
đánh ✅ sai.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Thêm một trạng thái *(ví dụ `→ ❌ KHÔNG LÀM <ngày> — <lý do>`)* | Sổ nói được sự thật. Nhưng đây là **đổi quy ước kho** ⇒ phải sửa `project-context.md:445-453` **cùng lượt**, và là một quyết định của Ice |
| **(b)** | Dùng `✅ ĐÃ ĐÓNG` kèm lý do *"đóng bằng cách quyết định không làm"* | 0 quy ước mới. Nhưng nó làm ✅ mang **hai nghĩa**, và một ký hiệu hai nghĩa là đúng thứ Ice đã gỡ 8.298 ca hồi 2026-08-07 |
| **(c)** | Không mục nào bị bác; mọi mục hoặc đóng hoặc có chủ | Đơn giản nhất. Nhưng nó **giả định** điều chưa đọc 113 mục thì chưa biết |

🔴 **Đừng tự chọn (a) rồi đi tiếp.** Xem cửa `AD` ở đầu Task 0.

---

### Quyết định #5 — 36 mục "không bám vào bề mặt nào": đọc hết, hay lấy mẫu

**Số đo:** 36 mục mà bộ phân loại **không** gán được vào `check-*` · test · `src/` · tài liệu · CI.
Đây là nhóm đắt nhất trên mỗi mục.

**Hai câu:** đọc **hết 36** *(chính xác, không đo trước được chi phí)*, hay **lấy mẫu 10** rồi quyết
xem phần còn lại có đáng đọc không *(rẻ, nhưng AC1 không đạt được cho tới khi đọc hết)*.

---

### Quyết định #6 — Ngân sách và LUẬT DỪNG cho một story đọc-tay

**Số đo:** 113 mục cần đọc tay. Story này **không có** một phép đo tự động nào nói *"xong"* ngoài AC1.

🔴 **Đây là loại story dễ chạy quá giờ nhất trong kho** — nó không có cổng nào đỏ để bảo nó dừng.
Chữ ký phải nói: một trần *(số mục, hay số vòng)*, và **chuyện gì xảy ra khi chạm trần** — phần còn
lại thành một mục nợ **có chủ**, hay story kéo dài.

⚠️ **Và một cái bẫy riêng của story này:** trong lúc đọc 113 mục, dev sẽ **tìm thấy khuyết tật thật**
*(ví dụ `ci.yml` chạy ma trận hai lần mỗi commit — đó là một lỗi có thật, không phải một ghi chú)*.
Cám dỗ vá tại chỗ. **Vá tại chỗ là đổi story này thành một story khác.** Chữ ký phải nói rõ: tìm thấy
thì **ghi chủ**, không vá.

---

### 0.9 — Việc phải làm ở Task này

1. **ĐO LẠI** bảng §ĐỌC TRƯỚC từ nguồn — **không chép**. Bảng đó chụp HEAD `4b30199`, và story này
   chạy **sau 2.4 và 2.12**, hai story sẽ tự đóng một số mục.
2. Trình sáu quyết định kèm số đo; ghi chữ ký + ngày + đường bị loại.
3. 🔴 Nếu #4 = (a): **dừng**, nêu với Ice như một lượt sửa `project-context.md` riêng.
4. 🔴 **LUẬT DỪNG** theo chữ ký #6.

---

## Tasks / Subtasks

### Task 0 — Cửa chặn: sáu quyết định (AC: 1-6) — **CHẶN MỌI TASK KHÁC**

- [ ] 0.1 Đo lại bảng §ĐỌC TRƯỚC **sau khi 2.4 và 2.12 đóng** — số sẽ khác
- [ ] 0.2 Nhận sáu chữ ký; ghi ngày + đường bị loại
- [ ] 0.3 Chốt **lệnh đếm** của AC5 và ghi nó vào story trước khi sửa mục đầu tiên

### Task 1 — Lệnh đếm tái lập được (AC: 5)

- [ ] 1.1 Dựng lệnh đếm ra đúng bốn con số: **mở · 🟡 · ✅ · mở-không-chủ**
- [ ] 1.2 ⚠️ **Đối chứng dương bắt buộc** — bộ đếm của lượt soạn story đã sai **hai** lần: regex bắt
      `2026-08-17` thành `Epic 2026`; và `Chủ:` viết nhiều dạng. Dựng ca âm/dương trước khi tin số
- [ ] 1.3 Ghi số **TRƯỚC** lượt phân loại, kèm ngày và HEAD

### Task 2 — 86 mục có chủ tự nhiên (AC: 2, 3, 4)

- [ ] 2.1 **40** mục font/từ điển — kiểm theo chữ ký #2. 🔴 **Không đánh ✅ bằng suy luận**
- [ ] 2.2 **29** mục Windows/nền tảng → **B7**, bằng một **tham chiếu**, không chép nội dung
- [ ] 2.3 **10** mục nghiệm thu tay → **B10**. ⚠️ B10 **chưa có chủ và chưa có lịch** — giao vào một
      mục chưa có chủ là chuyển chỗ, không phải giao chủ. Nêu với Ice nếu B10 vẫn trống
- [ ] 2.4 **7** mục "đo lại một con số" — mục nào đo được ngay thì đo; còn lại giao chủ

### Task 3 — 113 mục đọc tay (AC: 1, 3, 4)

- [ ] 3.1 **33** mục cổng `check-*` — nhiều mục là khuyết tật thật của cổng. 🔴 **Ghi chủ, đừng vá**
- [ ] 3.2 **17** test Rust/vitest · **17** mã `src/` · **6** tài liệu · **4** CI
- [ ] 3.3 **36** mục không bám bề mặt — theo chữ ký #5
- [ ] 3.4 Mỗi mục ra khỏi Task này mang **đúng một** trong: một `Chủ:` · `✅ ĐÃ ĐÓNG` kèm phép kiểm ·
      `🟡` kèm phần còn hở · *(nếu #4 = (a))* trạng thái thứ tư kèm lý do

### Task 4 — Cơ chế giữ cho nó không mọc lại (AC: 6)

- [ ] 4.1 Dựng theo chữ ký #3 — hoặc **rút AC6** và ghi một món nợ **có chủ**
- [ ] 4.2 Nếu là cổng: khuôn `check-layout.mjs:39-51` *(abort/fail)* · `:556-617` *(**tự kiểm**)*, và
      **ba danh sách** *(`package.json` · `ci.yml` · `.githooks/pre-push`)* — `check:gates` Kiểm D/E/F canh
- [ ] 4.3 🔴 Nếu cổng đỏ oan dù chỉ **một** ca trên sổ hiện tại ⇒ **không giao nó**. Một cổng đỏ oan
      trên sổ nợ sẽ bị tắt, và lúc đó tình trạng **tệ hơn** không có cổng

### Task 5 — Nghiệm thu (AC: 1-6)

- [ ] 5.1 Chạy lệnh của AC5: **mở-không-chủ = 0** *(hoặc con số của chữ ký #1)*
- [ ] 5.2 Ghi bảng **TRƯỚC / SAU** kèm ngày và HEAD
- [ ] 5.3 Chín cổng cũ *(+ cổng mới của 2.12, + cổng của Task 4 nếu có)*: **exit 0**
- [ ] 5.4 `npm run test` và `cargo test --locked` **không giảm** — story này **không chạm mã sản phẩm**;
      nếu hai số đổi thì có ai đó đã vá tại chỗ, ngược chữ ký #6
- [ ] 5.5 🔴 **Đếm lại số dòng**: không mục nào bị xoá ⇒ `deferred-work.md` chỉ **dài ra**, không ngắn đi

---

## Dev Notes

### Ranh giới phạm vi — năm thứ KHÔNG thuộc story này

1. **Vá bất kỳ khuyết tật nào tìm thấy trong lúc đọc.** Kể cả `ci.yml` chạy ma trận **hai lần** mỗi
   commit, kể cả `check-deps.mjs:95-99` `walk()` không có bộ nhớ đã-thăm. **Ghi chủ, đừng vá.**
2. **Story 2.4** *(NFR2)* và **Story 2.12** *(hạ tầng e2e)* — cả hai chạy **trước**, và cả hai sẽ tự
   đóng một số mục. Story này **không** làm phần của chúng.
3. **B7** *(bảng Windows)* và **B10** *(hình dạng nghiệm thu tay)* — story này **giao mục vào** chúng,
   **không dựng** chúng.
4. **`AD-48`** *(B6, Ice → Winston)*.
5. **Sửa `project-context.md`** — chỉ nếu chữ ký #4 = (a), và đó là một lượt riêng có chữ ký.

### Nghiệm thu — chọn đúng đường

| Mệnh đề | Đường |
|---|---|
| *"Không mục mở nào thiếu `Chủ:`"* | **lệnh đếm của AC5**, và **cổng tĩnh** nếu chữ ký #3 = (a) |
| *"Mục font/từ điển này thật sự đã đóng"* | **phép kiểm của chính món nợ đó** — mỗi mục một đường, không một phán quyết chung |
| *"Không mục nào bị xoá"* | `git diff` + số dòng, Task 5.5 |
| *"Story không chạm mã sản phẩm"* | `npm run test` và `cargo test` **không đổi**, Task 5.4 |

### Điều kiện khởi hành — ĐO 2026-08-18, HEAD `4b30199`

| Thứ | Số |
|---|---|
| `deferred-work.md` | **4.691** dòng · **83** khối `## Deferred from:` |
| Mục *(bullet cấp một)* | **448** |
| Mở · 🟡 · ✅ | **361** · **50** · **37** |
| Mở **không có `Chủ:`** | **199** *(Epic 1: 156 · Epic 2: 16 · khác: 27)* |
| Tỉ lệ đóng | **8,3 %** |
| Mục mở có chủ, Epic 2 vs Epic 1 | **80 %** vs **21 %** |

🔴 **Task 0.1 phải ĐO LẠI.** Story chạy **sau** 2.4 và 2.12; riêng 2.12 nhận chủ **11** mục và Task 9
của nó sẽ đóng chúng.

### Git — trạng thái cây khi story này được soạn

`git status --short` tại lượt soạn mang tạo tác của lượt `create-story` + `correct-course` cùng phiên
*(story 2.12 · story này · `epics.md` · `sprint-status.yaml` · sprint change proposal)*.
⚠️ Tới lúc dev: nếu cây bẩn vì thứ khác — **hỏi Ice, commit riêng, TRƯỚC thao tác đầu tiên**
*(`project-context.md:425-426`)*.

### Project Structure Notes

| Tệp | Loại | Việc |
|---|---|---|
| `_bmad-output/implementation-artifacts/deferred-work.md` | UPDATE | **tệp chính**; chỉ dài ra, không ngắn đi |
| `scripts/check-*.mjs` | NEW ×0-1 | **chỉ nếu** chữ ký #3 = (a) |
| `package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push` | UPDATE | **chỉ nếu** có cổng mới |
| `_bmad-output/project-context.md` | UPDATE | **chỉ nếu** chữ ký #4 = (a) — và đó là một lượt riêng |
| `src/**` · `src-tauri/**` | 🔴 **KHÔNG CHẠM** | Task 5.4 canh vế này bằng số test |

### References

**Luật bị phá**
- `project-context.md:445-453` — sổ nợ: kèm một chủ · **không mục nào mồ côi** · không bao giờ xoá ·
  🟡 cho nửa đóng
- `project-context.md:334-337` — *không đánh dấu đạt bằng suy luận*; số đo phải truy nguyên được

**Nguồn**
- `epic-2-retro-2026-08-18.md:361-364` — *"sổ nợ vẫn TRUNG THỰC — nhưng nó đã thành một hàng đợi
  không có người phục vụ"* ⚠️ con số ở đó là **khoảng Epic 2**, không phải cả sổ
- `epic-2-retro-2026-08-18.md:378` *(**B7**)* · `:381` *(**B10**)* — hai chủ tự nhiên của 39 mục

**Tiền lệ cho chữ ký #3**
- `deferred-work.md:4654-4674` — lớp lỗi `editorHasLoaded`: hụt **hai** lần, Ice chọn **một dòng luật,
  không một cổng**, kèm điều kiện mở lại ở lần thứ ba

**Khuôn cổng** *(chỉ nếu #3 = (a))*
- `scripts/check-layout.mjs:39-51` · `:556-617` · `scripts/check-gates.mjs:87-130` · `:311-425`

**Story liên đới**
- `2-12-ha-tang-e2e-va-cong-con-thieu.md` — Task 9 nhận chủ **11** mục; chạy **trước** story này

### Thông tin kỹ thuật mới nhất

⚠️ **Story này KHÔNG cần một phụ thuộc mới** — cửa NFR15 **không** mở. Nếu chữ ký #3 = (a), parser là
**tập con nghiêm ngặt tự viết**, Node thuần, **không thêm gói npm cho một cổng**
*(`project-context.md:317-318`)*.

---

## Dev Agent Record

### Agent Model Used

### Chữ ký Task 0

| # | Quyết định | Chữ ký | Ngày | Đường bị loại và vì sao |
|---|---|---|---|---|
| 1 | Phạm vi 199 hay 156 | | | |
| 2 | 40 mục font: kiểm bằng gì | | | |
| 3 | Cơ chế AC6 | | | |
| 4 | Trạng thái thứ tư *(cửa luật)* | | | |
| 5 | 36 mục: đọc hết hay lấy mẫu | | | |
| 6 | Ngân sách + luật dừng | | | |

### Debug Log References

### Completion Notes List

### File List

### Change Log

### Review Findings

---

## Câu hỏi cho Ice — chốt ở Task 0

**⑴ B10 vẫn chưa có chủ và chưa có lịch.** Task 2.3 giao **10** mục vào nó. Giao một mục vào một mục
chưa có chủ là **chuyển chỗ**, không phải giao chủ — và nó làm AC1 xanh trên một sổ chưa thật sự lành.
Ice muốn B10 có hình dạng **trước** story này, hay chấp nhận 10 mục đó đứng ở một chủ rỗng?

**⑵ Story này không có cổng nào bảo nó dừng.** 113 mục đọc tay, không phép đo tự động nào nói *"xong"*
ngoài AC1. Chữ ký #6 là chỗ duy nhất chặn nó chạy quá giờ — Ice muốn trần là **số mục**, **số vòng**,
hay một mốc thời gian?

**⑶ Tỉ lệ đóng 8,3 % có phải một vấn đề, hay là hình dạng đúng của một sổ nợ dự án?** Story này làm
mọi mục **có chủ**; nó **không** làm chúng được đóng. Nếu 8,3 % là thấp thì đó là một câu hỏi khác và
chưa có story nào cho nó.
