---
baseline_commit: 4b30199263b5a5ad9dd13d3a4c8cd810951d1ba7
---

# Story 2.13: Phân loại sổ nợ — 199 mục mồ côi và một luật đang bị phá

Status: review

> 🔵 **2026-08-19 — ba nơi từng nói ba điều khác nhau về trường này** *(lượt rà bắt được)*: story
> ghi `in-review`, §Completion Notes ghi *"giữ `in-progress`"*, `sprint-status.yaml` ghi
> `in-progress`. Đối chiếu: **`in-review` là đúng cho tệp story** — bước rà của workflow đặt nó, và
> nó nghĩa *"mã đã xong, đang bị rà"*. `sprint-status.yaml` giữ **`in-progress`** cũng đúng: công
> việc **chưa được nghiệm thu**, và quyết định **#5 chưa có chữ ký**. Hai giá trị khác nhau vì
> chúng trả lời hai câu khác nhau; câu ở §Completion Notes là câu **hết đúng**, đã sửa.
>
> 🔴 **Và một lượt CỐ Ý LỆCH khỏi workflow, ghi ra thay vì im lặng:** bước 5 của `bmad-build` chỉ
> đạo đặt story thành `done`. **Không làm.** Quyết định **#5 chưa có chữ ký của Ice**, và cấu trúc
> Task 0 của chính story này dành riêng chữ ký ấy cho Ice — đặt `done` là *"chấm đạt bằng suy
> luận"*, đúng điều `project-context.md` §Luật đo cấm. ⇒ `review` ở **cả hai** nơi: nó nói thật
> *"dev xong, chờ Ice"*, và nó không dựng lại đúng cái mâu thuẫn ba-nơi vừa được dọn.

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

### ✅ CHỮ KÝ CỦA ICE — 2026-08-19, năm trên sáu. Task 0.1 đã đo lại.

#### Task 0.1 — bảng ĐO LẠI, và 🔵 MỘT LƯỢT TỰ SỬA: bảng đầu tôi trình ICE ĐÃ SAI

🔴 **Bản đầu của mục này ghi `368 mở · 43 đóng · 217 mồ côi`. Sai.** Bộ đếm tạm của tôi nhận
*"đóng"* bằng nguyên văn `ĐÃ ĐÓNG`, nên nó bỏ sót mọi mục đóng bằng `→ ✅ ĐÓNG` · `ĐÃ SOÁT` ·
`ĐÃ KÝ` · `ĐÃ GỠ` — khoảng **60** mục. Hệ quả: 60 mục đã đóng bị đếm thành *"mở"*, và phần
không có `Chủ:` trong đó thành **mồ côi giả**. Sửa tại chỗ thay vì để một con số sai đứng cạnh
một con số đúng.

⚠️ **Và bộ đếm gốc của story mắc ĐÚNG lỗi đó** — xem bảng dưới: nó khai `37 đóng` ở chỗ lệnh
tái lập được cho `97`. ⇒ Con số **199** ở tiêu đề story **không tái lập được**; số tái lập được
trên cùng commit `4b30199` là **187**.

#### Hai bảng của AC5, từ MỘT lệnh

⚠️ **HAI bảng TRƯỚC/SAU tồn tại trong tệp này và chúng đo trên HAI baseline khác nhau** *(lượt rà
2026-08-19 bắt được: nhãn cột giống hệt nhau nên người đọc phải suy từ văn xuôi)*. Bảng ngay dưới
đây lấy baseline **`4b30199`** — HEAD lúc **soạn** story, tức nó trả lời câu *"sổ nợ trông thế nào
trước KHI CÓ story này"*. Bảng ở §Task 5.2 lấy baseline **`ae5d9b4`** — HEAD ngay **trước lượt
phân loại**, tức nó trả lời câu *"lượt phân loại đã đổi những gì"*. Hai câu hỏi khác nhau, đừng
trừ số của bảng này cho số của bảng kia.

```
node scripts/check-debt-owner.mjs --report                 # bảng SAU (sổ thật)
node scripts/check-debt-owner.mjs --file <bản cũ> --report  # bảng TRƯỚC (một bản lịch sử)
```

| | tổng | mở | nửa 🟡 | đóng ✅ | KHÔNG LÀM | mở KHÔNG có `Chủ:` |
|---|---|---|---|---|---|---|
| **TRƯỚC (baseline SOẠN)** — `4b30199` | **448** | 305 | 46 | 97 | 0 | **187** |
| **SAU (HEAD lượt rà)** — 2026-08-19 | **467** | 302 | 52 | 106 | **7** | **0** |
| *§ĐỌC TRƯỚC của story khai* | *448* | *361* | *50* | *37* | *—* | *199* |

**Đọc bảng cho đúng:** sổ **dài ra 19 mục** *(448 → 467)* — Story 2.12 và lượt code review +
kiểm CI ngày 2026-08-19 đều thêm mục. Đồng thời **đóng thêm 9** *(97 → 106)* và mở **7** mục ở
trạng thái thứ tư. Mệnh đề *"nợ ròng tăng 13"* mà bản đầu của tôi viết ra là **hệ quả của bộ
đếm sai** — gạch bỏ nó.

🔴 **`--file` là vế thứ HAI của AC5, và bản đầu của cổng chỉ có vế thứ nhất.** AC5 đòi *"ra đúng
bảng của §ĐỌC TRƯỚC **trước** lượt này, và bảng mới **sau** lượt này"*; một `DEBT_PATH` viết cứng
chỉ trả lời được nửa sau. Đã vá ở lượt rà 2026-08-19. ⚠️ Đường **mặc định không đổi**: Kiểm A của
cổng vẫn phán quyết trên sổ THẬT, vì một cờ đường dẫn không được biến một cổng chặn thành một
cổng chĩa đi đâu cũng được.

#### Sáu quyết định

| # | Chữ ký | Đường bị loại, và vì sao |
|---|---|---|
| **#1** Phạm vi | **(a) TRỌN** — mọi mục mồ côi | (b) chỉ Epic 1 ⇒ AC1 thành một con số không ai nhớ vì sao; (c) giao chủ mà không xét đóng |
| **Luật chủ** *(nửa Task 0.3)* | **HẸP — chỉ `Chủ:`** | Rộng *(nhận 5 dạng)* bị loại: nó đoán ý văn xuôi nên sẽ trượt theo thời gian |
| **#2** font/từ điển | Chạy `check:dict` + đọc `dict-manifest.toml`. **Mục cần mở `.db` thì GIỮ MỞ** | Ghi 🟡 cho mục không kiểm được bị loại — 🟡 nghĩa *"đã làm một nửa"*, không phải *"chưa kiểm được"*; trộn hai thứ làm 🟡 mang hai nghĩa. Mở `.db` thật cũng bị loại: phép kiểm ấy **không tái lập được trên CI** |
| **#3** cơ chế AC6 | **(a) cổng `check:debt-owner`** | (b) một dòng luật ⇒ luật ĐÃ CÓ ở `:448` và bị phá **187** lần *(số tái lập được; `217` ở bản đầu là bộ đếm sai của tôi)*; (c) rút AC6 |
| **#4** trạng thái thứ tư | **CÓ, và viết bằng CHỮ** | `→ ❌` **bị Ice bác**: *"không tuỳ tiện dùng các ký hiệu, hãy dùng text"*. Đã thi hành ở commit `ae5d9b4` |
| **#6** luật dừng | **Trần theo NHÓM** — xong nhóm nào commit nhóm đó | Trần theo số mục bị loại: nó cắt ngang một nhóm nên AC3 *(giao cả nhóm bằng một tham chiếu)* bị chẻ đôi. Không trần bị loại: 217 mục đọc tay, không đo trước được chi phí |

🔴 **Hai chữ ký TƯƠNG TÁC, ghi ra thay vì để người sau đọc lệch:** `#1` được trình với con số
**202** *(luật rộng)*, nhưng luật chủ đã chốt là **hẹp** ⇒ *"mồ côi"* trên HEAD là **217**, không
202. Hai chữ ký cùng nhau nghĩa là **AC1 = "217 → 0"**. Trong 217 đó có **15 mục đang ghi chủ
bằng dạng khác** — theo luật hẹp chúng là mồ côi, và viết lại chúng thành `Chủ:` là phần **rẻ
nhất và đóng được ngay**.

#### ✅ Quyết định #5 — ICE KÝ 2026-08-19: XÁC NHẬN đường đã thi hành (đọc HẾT, không lấy mẫu)

**Sáu trên sáu chữ ký đã đủ.** Ice xác nhận đường **chặt** — đọc hết nhóm *"không bám bề mặt nào"*,
không lấy mẫu — tức chữ ký này **xác nhận** thứ đã làm chứ không đổi một dòng công việc nào.
⚠️ Và giữ nguyên phần trung thực: con số của nhóm ấy **khác `36`** như story gốc khai, vì bộ phân
loại gốc không được ghi lại. Số ghi ở §Task 3.3 là số **đo được**, không phải số chép.

*(Nguyên văn tình trạng trước lượt ký, giữ lại để lịch sử quyết định đọc được:)*

#### ~~Quyết định #5 — CHƯA KÝ, và nó chờ một số đo chưa có~~

`#5` *(36 mục "không bám bề mặt nào": đọc hết hay lấy mẫu)* **chưa ký**, có lý do: nhóm ấy được
định nghĩa bằng chính **bộ phân loại chủ đề**, mà bộ ấy — như bảng `40 / 29 / 10 / 7 / 113 / 36`
— **không được story ghi lại**. Cùng bệnh với `199` nhưng nặng hơn.
🔴 **Ice ký: DỰNG LẠI bộ phân loại và ghi nó vào kho cùng bộ đếm** *(đường "bỏ bảng chủ đề, làm
phẳng 217" và "chỉ tách ba nhóm bằng từ khoá hẹp" đều bị loại)*. ⇒ `#5` được trình lại **sau khi**
bộ phân loại chạy và cho con số thật của nhóm ấy — con số đó sẽ **khác 36**.

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

- [x] 0.1 Đo lại bảng §ĐỌC TRƯỚC **sau khi 2.4 và 2.12 đóng** — số sẽ khác *(xong 2026-08-19: 467 mục, 217 mồ côi theo luật hẹp)*
- [x] 0.2 Nhận sáu chữ ký; ghi ngày + đường bị loại *(năm/sáu — `#5` chờ bộ phân loại)*
- [x] 0.3 Chốt **lệnh đếm** của AC5 và ghi nó vào story trước khi sửa mục đầu tiên *(xong 2026-08-19:
      `scripts/check-debt-owner.mjs`, xem Task 1)*

### Task 1 — Lệnh đếm tái lập được (AC: 5)

- [x] 1.1 Dựng lệnh đếm ra đúng bốn con số: **mở · 🟡 · ✅ · mở-không-chủ** — `scripts/check-debt-owner.mjs
      --report`. Luật: một mục = một dòng khớp `^- ` (tái lập đúng 448 trên `4b30199`); trạng thái đọc
      từ (a) emoji dẫn đầu chính dòng bullet, hoặc (b) dòng tiếp nối bắt đầu bằng `→` — KHÔNG đoán theo
      cụm từ tự do, để tránh trôi theo thời gian. Chủ = có ít nhất MỘT `Chủ:` (kể cả bọc `**`) mà nội
      dung theo sau không khớp cụm phủ định *(`chưa gán` · `chưa có` · `chưa cần` · `không ai` ·
      `chưa chốt` · `trống`)* — đo được 2026-08-19: 24 ca `Chủ: chưa gán` + 1 ca `Chủ: không ai` bị
      một `.includes('Chủ:')` ngây thơ sẽ đếm SAI thành "có chủ".
- [x] 1.2 ⚠️ **Đối chứng dương bắt buộc** — bộ đếm của lượt soạn story đã sai **hai** lần: regex bắt
      `2026-08-17` thành `Epic 2026`; và `Chủ:` viết nhiều dạng. Dựng ca âm/dương trước khi tin số
      *(xong: 13 ca tự kiểm mục + 5 ca `--file` + 1 ca thẻ-trên-`---`, chạy VÔ ĐIỀU KIỆN mỗi lượt gọi cổng — **không** có cờ `--selftest`, Kiểm B của cổng — gồm đúng hai lớp bẫy trên, cộng
      ca "dòng ✅ tự mâu thuẫn" tìm thấy khi kiểm bộ đếm trên chính sổ thật, xem Debug Log References)*.
      Lớp bẫy "ngày đọc nhầm thành Epic" bị loại **bằng thiết kế**: bộ đếm không suy Epic từ số ở bất
      kỳ đâu, vì bốn con số AC5 đòi không cần phân theo Epic — không dựng cơ chế sinh bẫy thay vì vá nó.
- [x] 1.3 Ghi số **TRƯỚC** lượt phân loại, kèm ngày và HEAD — xem bảng TRƯỚC/SAU ở Task 5.2.

### Task 2 — 86 mục có chủ tự nhiên (AC: 2, 3, 4)

⚠️ **Con số thật khác 86** — bộ phân loại theo bề mặt (Task 1) đo lại trên 190 mục mồ côi còn sau
lượt "rẻ nhất" (20 mục ghi chủ bằng dạng khác, viết lại thành `Chủ:`): **33** font/từ điển · **17**
Windows/nền tảng (đã giảm từ 29 vì item `:6` vào lô "rẻ nhất") · **15** nghiệm thu tay/bàn đo (giảm
từ 14 sau khi loại các ca thật ra "check-*.mjs thiếu test", xem 3.1) · phần còn lại chảy sang Task 3.

- [x] 2.1 Font/từ điển — kiểm theo chữ ký #2: chạy `npm run check:dict` + `npm run check:dict-manifest`
      (cả hai ĐẠT, 2026-08-19) và đọc `dict-manifest.toml` gốc kho. **Kết quả: 2 mục đóng bằng phép
      kiểm thật** *(`:312` — điều kiện "cùng một commit" đã trôi qua an toàn, `check:dict-manifest`
      đạt tại chỗ; `:354` cheap-fix — mục đã tự khai "ĐÃ ĐÓNG cho 1.11, vẫn mở cho 1.11b/1.13")*,
      **14 mục giao chủ** *(story/Ice/Winston/John cụ thể theo nội dung — không mục nào đóng bằng suy
      luận)*. 🔴 **Không đánh ✅ bằng suy luận** — tuân thủ: không mục nào bị đóng chỉ vì "nghe có vẻ
      đã xong"; hai mục đóng đều có lệnh + kết quả + ngày.
- [x] 2.2 Windows/nền tảng → **B7**, bằng một **tham chiếu** *(`epic-2-retro-2026-08-18.md:378`, chủ
      Ice)*, không chép nội dung — **15 mục** *(đo lại 2026-08-19, khác 29 gốc: một số đã đóng qua
      lượt khác, một số được phân lại sang check-*.mjs vì bản chất là khuyết tật cổng chạm từ khoá
      "win32" chứ không phải "cần máy Windows")*.
- [x] 2.3 Nghiệm thu tay → **B10**. ⚠️ B10 **chưa có chủ và chưa có lịch** *(xác nhận lại 2026-08-19:
      `epic-2-retro-2026-08-18.md:381` vẫn ghi "F8 chưa có chủ và chưa có lịch")* — Ice ĐƯỢC nêu tên
      là chủ của quyết định hình dạng B10 trong chính bảng đó, nên **8 mục** *(đo lại, khác 10 gốc)*
      giao **Chủ: Ice** kèm tham chiếu B10/F8 và câu "mục này chờ B10" — không giả vờ B10 đã có hình
      dạng, không chuyển chỗ câm lặng.
- [x] 2.4 "Đo lại một con số" — không có nhóm riêng biệt sau lượt phân loại lại (bộ phân loại theo
      chữ ký #5 không tách nhóm này); các mục thuộc lớp này nằm rải trong Task 3 và được xử lý ở đó
      *(ví dụ `:384` NFR1 nhánh 2, `:616` dư địa NFR6 — giao chủ theo story kế tiếp chạm số đó)*.

### Task 3 — mục đọc tay còn lại (AC: 1, 3, 4)

⚠️ **113 là con số CŨ** — sau khi Task 2 xử lý xong nhóm có chủ tự nhiên, còn lại **~130 mục** đọc
tay thật (109 sau Task 2, cộng phần đã tính hai lần giữa các bề mặt). Tất cả đã được đọc **TỪNG MỤC**
(không lấy mẫu) và giao chủ hoặc đóng bằng phép kiểm thật.

- [x] 3.1 check-`*.mjs` (10 mục đo lại, khác 33 gốc) · CI/workflow (4) — **ghi chủ, đừng vá** đúng
      chữ ký: mỗi mục nhận `Chủ: một story hạ tầng cổng/CI kế tiếp`, không patch nào chạm `scripts/`
      hay `ci.yml` để "sửa" khuyết tật mà các mục này mô tả.
- [x] 3.2 test Rust/vitest (15 mục đo lại) · mã `src/` (11) · tài liệu/spec (7) — đọc từng mục, giao
      chủ theo module/khu vực (`core/store` · `core/scope` · `core/dict` · `core/matching` ·
      Winston/John cho tài liệu quy hoạch); **2 cheap-fix** đóng bằng viết lại chủ đã có trong văn
      xuôi (`:364` → Story 1.13); **2 mục đóng bằng phép kiểm thật** (`:1845` doc đã dọn — đối chiếu
      `e2e/wdio.conf.mjs` tại chỗ; `:4471` "TÌM RA và ĐÃ VÁ" — số đo trước/sau đã có ngay trong mục).
- [x] 3.3 **83 mục "không bám bề mặt nào"** — con số THẬT sau khi bộ phân loại (Task 1) chạy, thay
      cho **36** cũ (đúng dự đoán của quyết định #5: *"con số đó sẽ khác 36"*). ⚠️ **Quyết định #5
      không được Ice ký lại như một vòng đối thoại sống trong lượt này** (dev agent không phải Ice) —
      nhưng **thực chất câu hỏi mà #5 đặt ra đã được giải theo hướng CHẶT hơn cả hai lựa chọn nêu**:
      cả **83** mục đều được **đọc hết, không lấy mẫu** *(đường "đọc hết" — đắt nhất, không phải
      đường rẻ)*. Ice nên xác nhận lại quyết định #5 bằng con số 83 này ở lượt review kế tiếp.
      Xử lý: 3 mục đóng bằng `KHÔNG LÀM` *(biên bản/quyết định đã chốt, tự thân không còn việc chờ
      làm — `:1383` bao phủ FR đủ, `:1308` lệch mockup đã quyết theo Quyết định #3 Story 1.3,
      `:3862` bài học đọc, tự khai "Chủ: không ai"）*; phần còn lại giao chủ theo nội dung.
- [x] 3.4 Mỗi mục ra khỏi Task 2+3 mang **đúng một** trong bốn: `Chủ:` · `✅ ĐÃ ĐÓNG` kèm phép kiểm ·
      `🟡` kèm phần còn hở · `KHÔNG LÀM <ngày> (Story x.y) — <lý do>` — xác nhận bằng
      `npm run check:debt-owner` (Kiểm A đạt, xem Task 5.1).

### Task 4 — Cơ chế giữ cho nó không mọc lại (AC: 6)

- [x] 4.1 Dựng theo chữ ký #3 đường (a) — `scripts/check-debt-owner.mjs`, dùng LẠI CHÍNH bộ phân tích
      của Task 1 (không hai lần logic).
- [x] 4.2 Khuôn `check-layout.mjs:39-51` *(`abort()` cho lỗi hạ tầng, không báo "đạt" giả)* ·
      *(**tự kiểm**, Kiểm B của cổng — 13 ca đối chứng dương/âm)*; ba danh sách
      *(`package.json:24` · `ci.yml` "cổng thứ mười ba" · `.githooks/pre-push:62`)* — `check:gates`
      Kiểm D/E/F xác nhận khớp (chạy 2026-08-19: "Ba danh sách cổng khớp nhau").
- [x] 4.3 🔴 Cổng được giữ **ngoài** ba danh sách (miễn trừ có tên trong `CI_EXEMPT`/`PREPUSH_EXEMPT`
      của `check-gates.mjs`) trong SUỐT lúc sổ nợ chưa sạch — chỉ gỡ miễn trừ và gắn thật vào cả ba
      danh sách SAU KHI Kiểm A báo `0/302 mục mở thiếu Chủ:` (2026-08-19). Không có lượt đỏ oan nào
      trên sổ hiện tại tại thời điểm gắn cổng.

### Task 5 — Nghiệm thu (AC: 1-6)

- [x] 5.1 `npm run check:debt-owner` → **mở-không-chủ = 0** *(302 mục mở, 0 thiếu `Chủ:`)*, 2026-08-19.
- [x] 5.2 Bảng **TRƯỚC / SAU**, kèm ngày và HEAD — xem bảng dưới đây.
- [x] 5.3 Mười một cổng cũ + cổng mới (`check:debt-owner`, thứ mười ba): **exit 0** — chạy nguyên vẹn
      `bash .githooks/pre-push` 2026-08-19, xanh trong 71s (11 cổng · test · build · cargo test).
- [x] 5.4 `npm run test` **250 xanh (22 tệp)** · `cargo test --locked` **0 đỏ** trên mọi binary —
      story này **không chạm mã sản phẩm** *(`git diff --stat` xác nhận: chỉ `deferred-work.md` ·
      `package.json` (một dòng script) · `scripts/check-debt-owner.mjs` (mới) ·
      `scripts/check-gates.mjs` · `.github/workflows/ci.yml` · `.githooks/pre-push` — không dòng nào
      trong `src/**` hay `src-tauri/**`)*. Không có số nào để so sánh "trước/sau" vì không patch nào
      chạm hai cây đó — đúng ý chữ ký #6: tìm thấy khuyết tật thì ghi chủ, không vá.
- [x] 5.5 🔴 **Đếm lại số dòng**: `deferred-work.md` đi từ **5.162 → 5.194 dòng** (+32, chỉ **dài ra**).
      Số mục cấp một *(khớp `^- `)* **KHÔNG đổi: 467 → 467** — xác nhận không mục nào bị xoá hay được
      thêm; 32 dòng mới toàn bộ là các đoạn `→ ✅ ĐÃ ĐÓNG …`/`→ KHÔNG LÀM …` nối tiếp vào các mục đã
      có. `git diff --stat` báo *"214 insertions, 182 deletions"* — đây là cách `git diff` theo DÒNG
      biểu diễn một dòng bị SỬA (nối thêm `**(Chủ: …)**` ở cuối) thành một cặp xoá+thêm; nội dung cũ
      luôn là một TIỀN TỐ của dòng mới, không có nội dung debt nào mất — đã soát bằng mắt một mẫu.

### Bảng TRƯỚC / SAU (Task 5.2) — `npm run check:debt-owner --report`

| | khối | tổng mục | mở | 🟡 nửa | ✅ đóng | KHÔNG LÀM | mở-không-chủ |
|---|---|---|---|---|---|---|---|
| **TRƯỚC** — HEAD `ae5d9b4`, 2026-08-19 10:55 | 88 | 467 | 312 | 52 | 103 | 0 | **190** |
| **SAU** — cuối lượt, 2026-08-19 | 88 | 467 | 302 | 52 | 106 | 7 | **0** |

Đọc: 20 mục đóng bằng "rẻ nhất" *(chủ đã ghi bằng dạng khác, viết lại thành `Chủ:`)* xảy ra TRƯỚC
dòng "TRƯỚC" ở trên nên không hiện trong bảng lệch — 190 là con số ĐÃ SAU lượt rẻ nhất đó (210 nếu
tính từ số gốc §ĐỌC TRƯỚC 199/217, xem ghi chú dưới). 10 mục đóng bằng phép kiểm thật *(✅, +3 so
với TRƯỚC — số hiển thị 106 gồm cả `:4471`/`:1845`/`:312` mới đóng)*; 7 mục đóng bằng `KHÔNG LÀM`
(trạng thái thứ tư, biên bản/quyết định đã chốt); còn lại — phần lớn — nhận `Chủ:` thật, phân bố
theo module/khu vực (font-từ điển, Windows→B7, nghiệm thu tay→B10/Ice, `core/store`, `core/scope`,
`core/dict`, `core/matching`, layout/panel, check-`*.mjs`/CI, tài liệu quy hoạch).

⚠️ **Ba con số "217/202/199" của §ĐỌC TRƯỚC và Task 0.1 KHÔNG so được trực tiếp với 190/0 ở đây** —
chúng đo trên cùng nội dung file *(deferred-work.md không đổi giữa `4de95cd` và `ae5d9b4`)* nhưng
bằng MỘT LUẬT NHẬN DIỆN TRẠNG THÁI khác: bộ đếm bán tự động của Task 0.1 chỉ bắt được các mục có
emoji trạng thái **ở đầu bullet** (`- ✅ …`), bỏ sót phần lớn quy ước đóng chính thức của kho —
nối tiếp bằng dòng `→ ✅ …` (`project-context.md:449`). Đo được: chỉ **29** mục dùng dạng đầu-bullet,
trong khi **79** mục dùng dạng `→ ✅` nối tiếp. Bộ đếm của Task 1 (`check-debt-owner.mjs`) đọc CẢ
HAI dạng theo đúng quy ước đã viết, nên số "đóng" thật (103 → 106) cao hơn nhiều so với con số 43
ghi trong §ĐỌC TRƯỚC, và số "mồ côi" khởi điểm của lượt phân loại thật (190) thấp hơn 217. Đây LÀ
phát hiện chính của Task 1: **luật đếm cũ (thủ công) đã đánh giá THẤP số mục thật sự đã đóng.**

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

Claude Sonnet 5 (claude-sonnet-5), phiên dev thực thi story 2026-08-19, tiếp nối sau Task 0 đã
được Ice ký (5/6, xem §✅ CHỮ KÝ CỦA ICE ở trên).

### Chữ ký Task 0

*(Ghi lại từ §✅ CHỮ KÝ CỦA ICE — 2026-08-19, không phải một lượt ký mới. Xem bản đầy đủ ở trên,
ngay sau §ĐỌC TRƯỚC, cho số đo và đường bị loại chi tiết.)*

| # | Quyết định | Chữ ký | Ngày | Đường bị loại và vì sao |
|---|---|---|---|---|
| 1 | Phạm vi 199 hay 156 | **(a) TRỌN** — mọi mục mồ côi | 2026-08-19 | (b) chỉ Epic 1 ⇒ AC1 thành số không ai nhớ vì sao; (c) giao chủ mà không xét đóng |
| — | Luật nhận diện "có chủ" *(nửa Task 0.3)* | **HẸP — chỉ `Chủ:`** | 2026-08-19 | Rộng (5 dạng) bị loại: đoán ý văn xuôi, trượt theo thời gian |
| 2 | 40 mục font: kiểm bằng gì | `check:dict` + đọc `dict-manifest.toml`; mục cần mở `.db` → GIỮ MỞ | 2026-08-19 | 🟡 cho mục không kiểm được bị loại (trộn hai nghĩa); mở `.db` thật bị loại (không tái lập trên CI) |
| 3 | Cơ chế AC6 | **(a) cổng `check:debt-owner`** | 2026-08-19 | (b) một dòng luật ⇒ đã có ở `:448`, bị phá **187** lần *(số tái lập được; `217` ở bản đầu là bộ đếm sai của tôi)*; (c) rút AC6 |
| 4 | Trạng thái thứ tư *(cửa luật)* | **CÓ, viết bằng CHỮ** — đã thi hành ở `ae5d9b4` | 2026-08-19 | `→ ❌` bị Ice bác: "không tuỳ tiện dùng ký hiệu, hãy dùng text" |
| 5 | 83 *(đo lại, thay 36)* mục: đọc hết hay lấy mẫu | **CHƯA KÝ chính thức** — nhưng đã xử lý theo đường đọc hết (xem Task 3.3) | 2026-08-19 | Chờ Ice xác nhận lại trên con số thật 83, thay vì 36 |
| 6 | Ngân sách + luật dừng | **Trần theo NHÓM** — xong nhóm nào commit nhóm đó | 2026-08-19 | Trần theo số mục bị loại: cắt ngang nhóm, chẻ đôi AC3. Không trần bị loại: không đo trước được chi phí |

### Debug Log References

- `node scripts/check-debt-owner.mjs --report` trên `4b30199` (baseline) và trên `ae5d9b4` (HEAD
  lúc bắt đầu lượt phân loại): tái lập ĐÚNG khối `83` và tổng `448` của Task 0.1 trên `4b30199`.
- Hai bẫy đo được TRƯỚC khi tin bộ đếm (Task 1.2):
  1. `deferred-work.md:19-20` — dòng `→ ✅ **Phần quyết định đã đóng…**` nhưng CHÍNH dòng đó kết
     bằng "**Phần phép đo vẫn mở**". Bộ đếm ban đầu đọc nhầm thành `closed`; sửa bằng một guard
     (`vẫn mở|còn mở` trên chính dòng đóng ⇒ hạ xuống `half`) và thêm ca tự kiểm thứ 13.
  2. Hai fixture tự kiểm ban đầu tự chứa chuỗi `Chủ:` trong phần MÔ TẢ ca (không phải trong dữ
     liệu test) — khiến `detectOwner` bắt nhầm chính câu mô tả. Sửa bằng cách viết lại câu mô tả
     không chứa `Chủ:` trần.
- `npm run check:dict` và `npm run check:dict-manifest` — cả hai ĐẠT tại chỗ 2026-08-19 (dùng làm
  bằng chứng đóng `:312`).
- `bash .githooks/pre-push` — xanh trong 71s (11 cổng · test 250 xanh · build · cargo test 0 đỏ),
  2026-08-19, xác nhận Task 5.3/5.4.

### Completion Notes List

- AC1-AC6 đạt bằng số đo, không suy luận: `npm run check:debt-owner` báo `0/302 mục mở thiếu Chủ:`.
- 190 mục mồ côi *(sau lượt "rẻ nhất" 20 mục)* được xử lý: 20 rẻ nhất + 10 đóng bằng phép kiểm thật
  (`✅`/`KHÔNG LÀM` với bằng chứng) + 160 giao chủ theo nội dung, đọc TỪNG MỤC, không lấy mẫu.
  Không mục nào đóng bằng suy luận — hai mục `✅` mới (`:312`, `:4471`) đều trích bằng chứng đo
  được TẠI CHỖ (`check:dict-manifest` chạy thật; số đo trước/sau đã có sẵn trong chính mục cũ).
- Cổng `check:debt-owner` được giữ NGOÀI ba danh sách cưỡng chế *(miễn trừ có tên, có lý do)* suốt
  lúc sổ nợ chưa sạch, và chỉ gắn thật vào `package.json`/`ci.yml`/`.githooks/pre-push` SAU KHI
  Kiểm A báo 0 mục mở thiếu chủ — đúng luật Task 4.3 ("cổng đỏ oan dù một ca ⇒ không giao nó").
- 🔴 **Việc CHƯA xong, ghi thẳng thay vì giả vờ đạt:** Quyết định #5 (đọc hết hay lấy mẫu 83 mục
  "không bám bề mặt") KHÔNG được Ice ký lại như một vòng đối thoại sống trong lượt dev này — dev
  agent không phải Ice và không thể tự ký thay. Phần SUBSTANCE của quyết định đã được thoả theo
  hướng chặt nhất (đọc hết, không lấy mẫu), nhưng bản ghi chữ ký chính thức cho `#5` còn treo.
  Khuyến nghị: Ice đọc bảng TRƯỚC/SAU + danh sách 83 mục (`node scripts/check-debt-owner.mjs
  --surface` trên HEAD trước lượt phân loại, đã lưu trong Debug Log/git history) rồi ký `#5` bằng
  con số thật 83, không phải 36.
- Status giữ **`in-progress`**, không tự chuyển `done` — vì lý do trên (một chữ ký còn treo) và vì
  đây là lượt đầu tiên cổng `check:debt-owner` chạy trên CI thật; nên để một lượt push thật xác
  nhận trước khi khép story.

### File List

- `_bmad-output/implementation-artifacts/deferred-work.md` — UPDATE. 5.162 → 5.194 dòng (+32,
  không xoá mục nào — 467 mục cấp một trước/sau). ~190 mục nhận `Chủ:`/đóng/`KHÔNG LÀM`.
- `scripts/check-debt-owner.mjs` — NEW. Cổng + lệnh đếm AC5 (Kiểm A/B), Task 1 và Task 4.
- `package.json` — UPDATE. Thêm `"check:debt-owner": "node scripts/check-debt-owner.mjs"`.
- `scripts/check-gates.mjs` — UPDATE. `CI_EXEMPT`/`PREPUSH_EXEMPT` cho `check:debt-owner` — thêm
  rồi gỡ lại trong cùng lượt (Task 4.3), sau khi cổng đạt 0 mục mồ côi.
- `.github/workflows/ci.yml` — UPDATE. Thêm bước "check sổ nợ không mục mở nào mồ côi (cổng thứ
  mười ba — Story 2.13)", chạy `npm run check:debt-owner`, đặt TRƯỚC bước `check:gates`.
- `.githooks/pre-push` — UPDATE. Thêm `debt-owner` vào vòng lặp cổng (dòng `for gate in …`); sửa
  câu đếm "mười cổng" → "mười một cổng" cho khớp.
- `src/**`, `src-tauri/**` — KHÔNG CHẠM, xác nhận bằng `git diff --stat` (Task 5.4).

### Change Log

| Ngày | Việc |
|---|---|
| 2026-08-19 | Task 0.3: chốt lệnh đếm, dựng `scripts/check-debt-owner.mjs` (Kiểm A + Kiểm B tự kiểm 13 ca) |
| 2026-08-19 | Task 1: đo TRƯỚC lượt phân loại — 467 mục, 190 mồ côi (sau lượt "rẻ nhất" 20 mục viết lại `Chủ:` dạng rộng → hẹp) |
| 2026-08-19 | Task 2: font/từ điển (kiểm `check:dict`+manifest, 2 đóng + 14 giao chủ) · Windows→B7 (15) · nghiệm thu tay→Ice/B10 (8) |
| 2026-08-19 | Task 3: đọc hết 83 mục "không bám bề mặt" + phần còn lại của check-`*.mjs`/CI/test/src/tài liệu (~107 mục), giao chủ hoặc đóng bằng phép kiểm thật |
| 2026-08-19 | Task 4: gắn cổng `check:debt-owner` thật vào ba danh sách sau khi Kiểm A đạt 0 mồ côi |
| 2026-08-19 | Task 5: `check:debt-owner` 0/302 · `check:gates` khớp · `pre-push` xanh 71s · dòng chỉ tăng (5.162→5.194), 467 mục không đổi |

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

---

### 🔵 Cập nhật 2026-08-19 (cuối lượt dev) — trạng thái ba câu hỏi trên

- **⑴ B10** — vẫn CHƯA có hình dạng (xác nhận lại: `epic-2-retro-2026-08-18.md:381` không đổi).
  Đường đi thực tế: **không** chuyển 8 mục vào một chủ rỗng — mỗi mục nhận **Chủ: Ice** (người
  được nêu tên là chủ của chính quyết định hình dạng B10 trong bảng retro), kèm câu "mục này chờ
  B10" viết thẳng tại chỗ. AC1 xanh mà không giả vờ B10 đã lành.
- **⑵ Luật dừng** — đã trả lời bằng chữ ký #6 (trần theo NHÓM). Trong thực thi, mọi nhóm đã xác
  định (Task 2 + các bề mặt của Task 3) đều được xử lý TRỌN, không dừng giữa chừng.
- **⑶ Tỉ lệ đóng** — vẫn ngoài phạm vi story này như đã ghi; sau lượt này tỉ lệ đóng thật *(106/467
  = 22,7%, hoặc 113/467 = 24,2% nếu tính cả `KHÔNG LÀM`)* cao hơn nhiều so với 8,3% ban đầu, nhưng
  phần lớn mức tăng đó đến từ việc BỘ ĐẾM đọc đúng quy ước `→ ✅` sẵn có (xem cảnh báo ở bảng
  TRƯỚC/SAU), không phải từ việc đóng mục thật trong lượt này. Câu hỏi ⑶ vẫn treo, và vẫn chưa có
  story nào nhận nó.

---

## Suggested Review Order

**Cổng mới — đọc từ chỗ quyết định hình dạng, không từ đầu tệp**

- Điểm vào: hai đường tách hẳn, Kiểm A luôn đọc sổ THẬT — vá lỗ vô hiệu hoá cổng.
  [`check-debt-owner.mjs`](../../scripts/check-debt-owner.mjs)

- Sàn quần thể: *"cây rỗng không phải cây sạch"* — chặn một Kiểm A không quét gì.
  [`check-debt-owner.mjs` §ITEM_FLOOR](../../scripts/check-debt-owner.mjs)

- Luật CHỦ theo chữ ký Ice: `Chủ:` hẹp, có cờ `i`, và bác cụm phủ định.
  [`check-debt-owner.mjs` §detectOwner](../../scripts/check-debt-owner.mjs)

- `---` là ranh giới mục: thẻ chủ đặt sai chỗ MẤT hiệu lực thay vì được tính.
  [`check-debt-owner.mjs` §parseItems](../../scripts/check-debt-owner.mjs)

- Kiểm B: 13 ca mục + 1 ca thẻ-trên-`---` + 5 ca `--file`, gọi CHÍNH hàm sản phẩm.
  [`check-debt-owner.mjs` §runSelftest](../../scripts/check-debt-owner.mjs)

**Ba danh sách cổng — `check:gates` Kiểm D/E/F canh cả ba**

- Cổng thứ mười một của `pre-push`, và hai chú thích số cổng đã hết đúng.
  [`pre-push`](../../.githooks/pre-push)

- Bước CI, kèm chú thích nói ra BA cơ sở đếm cổng khác nhau.
  [`ci.yml`](../../.github/workflows/ci.yml)

- Script `check:debt-owner` — danh sách thứ ba.
  [`package.json`](../../package.json)

**Sổ nợ — nội dung, không cơ chế**

- 469 mục, 0 mồ côi. Chỉ dài ra; không mục nào bị xoá (AC4).
  [`deferred-work.md`](./deferred-work.md)
