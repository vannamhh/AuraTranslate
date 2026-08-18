# Sprint Change Proposal — 2026-08-18

**Kích hoạt bởi:** retro Epic 2 *(2026-08-18)*, action item **B2**
**Người soạn:** correct-course workflow
**Chế độ:** Batch
**Phân loại phạm vi:** 🟢 **Minor** — một artifact, một khối chèn, không FR mới, không chạm MVP

---

## 1. Tóm tắt vấn đề

**Story 2.12 tồn tại ở tầng thực thi nhưng không tồn tại ở tầng quy hoạch.**

Retro Epic 2 dựng **hai cửa chặn Ice ký** trước khi Epic 3 được mở. Cửa ② là *"một story hạ tầng e2e
phải chạy TRƯỚC Epic 3"*, giao đích danh cho `create-story` *(action item B2, chủ: `Ice → create-story`)*.
Story đã được soạn ngày 2026-08-18 và đang ở `ready-for-dev` trong `sprint-status.yaml`.

**Nhưng `epics.md` không có mục nào cho nó** — và chỗ mắt người đọc tìm nó đang bị chiếm bởi một khối
đã chết.

### Bằng chứng đo được

| Phép đo | Kết quả |
|---|---|
| `grep "^### Story 2\.12" epics.md` | **0** mục đang hoạt động |
| Thứ nằm ở đúng vị trí đó | `### ~~Story 2.12: Sync Scrolling~~ — XOÁ` *(`epics.md:2659-2667`)* |
| `grep "^### Story .*(e2e\|hạ tầng\|cổng)" epics.md` | chỉ **Story 1.22** *(`in-progress`)* và **Story 8.8** *(cổng `.docx`)* |
| Món nợ trong `deferred-work.md` mang chủ *"story hạ tầng e2e"* | **11**, và **không có ngày** cho tới khi cửa chặn ② ra đời |
| Khoá trong `sprint-status.yaml` | `2-12-ha-tang-e2e-va-cong-con-thieu: ready-for-dev` ✅ |
| Tệp story | `implementation-artifacts/2-12-ha-tang-e2e-va-cong-con-thieu.md`, 7 AC, 8 chữ ký ✅ |

### Loại vấn đề

**Misunderstanding of original requirements — không phải.**
**Technical limitation — không phải.**

⇒ **Đây là một khoảng trống ở tầng quy hoạch, đúng lớp mà Story 1.22 đã gặp và đã xử.** `epics.md:1993`
ghi nguyên văn lý do vẫn dựng story cho 1.22 dù mã đã viết trước:

> *"bộ chạy đang mang **ba khuyết tật có tên** mà **không tạo tác nào ở tầng quy hoạch chịu trách
> nhiệm cho chúng**. Không có story thì ba mục đó sống trong một tệp đề xuất."*

Hôm nay là **cùng một câu**, ở quy mô lớn hơn: **11** món nợ sống trong một tệp retro.

🔴 **Và đây KHÔNG phải *"sửa spec cho khớp mã"*** *(`project-context.md:456-458`)*. Không một AC nào
bị thu hẹp, không một FR nào bị đổi. Đây là **thêm một tạo tác quy hoạch cho một việc đã được ký**.

---

## 2. Phân tích ảnh hưởng

### 2.1 Epic — `[x] Done`

| Câu hỏi | Trả lời | Bằng chứng |
|---|---|---|
| Epic 2 còn hoàn thành được như kế hoạch? | **Có.** 12/14 story `done`; `2-3` và `2-4` là đối tượng của retro, không phần thiếu của nó | retro §1 |
| Epic 2 cần đổi phạm vi / AC? | **Không.** Story 2.12 **không mang FR nào** | — |
| Có cần epic mới? | **Không** | — |
| Epic nào thành lỗi thời? | **Không** | — |
| Thứ tự epic có đổi? | **Không đổi thứ tự EPIC.** Nhưng thứ tự **trong** Epic 2 có: Ice chốt 2026-08-18 **2.4 → 2.12 → Epic 3** | đã ghi vào `sprint-status.yaml` và tệp story |

### 2.2 Xung đột artifact

| Artifact | Ảnh hưởng | Trạng thái |
|---|---|---|
| **`epics.md`** | Thiếu một khối `### Story 2.12` | **`[!] Action-needed`** — đây là toàn bộ đề xuất này |
| `epics.md` — bảng FR→Epic | **Không.** Story 2.12 covers 0 FR | `[N/A]` |
| `epics.md` — *"Tổng kiểm: 132/132 FR ánh xạ… Epic 2: 9"* *(`:784`)* | **Không đổi.** 0 FR mới | `[N/A]` |
| `epics.md` — blurb Epic 2 *(`:862`, `:2039`)* | **Không đổi.** 🔴 **Tiền lệ đo được:** Story 1.22 xuất hiện **đúng một chỗ** trong cả tệp — chỉ là heading của chính nó; blurb Epic 1 và `FRs covered` của Epic 1 **không nhắc nó** | `[N/A]` |
| **`prd.md`** | **Không.** 0 FR mới, 0 hành vi người dùng mới | `[N/A]` |
| **`ARCHITECTURE-SPINE.md`** | **Không — ở lượt này.** ⚠️ Chữ ký **#4** của story *(tín hiệu "flush xong" lộ ra bằng gì)* là **ứng viên cửa `AD`**. Nếu nó kích hoạt thì đó là một hồ sơ bàn giao cho Winston, **không** phải một mục của đề xuất này | `[N/A]` — có điều kiện |
| **UX / `EXPERIENCE.md`** | **Không.** Story không có bề mặt người dùng | `[N/A]` |
| **`sprint-status.yaml`** | Đã cập nhật 2026-08-18 | `[x] Done` |
| **Testing strategy / CI** | Story **sẽ** thêm 1-2 cổng vào ba danh sách *(`package.json` · `ci.yml` · `.githooks/pre-push`)* — nhưng đó là **việc của story**, không của đề xuất này | `[x] Done` — đã nằm trong Task 7 |

### 2.3 MVP

**Không ảnh hưởng.** Story 2.12 mang 0 FR. Nó là một **cửa chất lượng** đứng trước Epic 3, không một
phần phạm vi giao hàng.

---

## 3. Đường đi được chọn

| Phương án | Đánh giá |
|---|---|
| **① Direct Adjustment** — thêm một khối `### Story 2.12` vào `epics.md` | ✅ **Viable.** Effort **Low** · Risk **Low**. Đúng tiền lệ Story 1.22 |
| **② Rollback** | ❌ **Not viable.** Không có gì để hoàn nguyên — không mã nào đã viết cho story này |
| **③ PRD MVP Review** | ❌ **Not viable.** 0 FR, 0 tác động MVP. Mở PRD ra ở đây là một lượt đụng vào tài liệu không có lý do |

**Chọn: ① Direct Adjustment.**

**Lý lẽ:** thay đổi nhỏ nhất đóng được khoảng trống, và nó có **một tiền lệ trong chính tệp** — Story
1.22 đã giải đúng bài toán này ở Epic 1 bằng đúng hình dạng đó. Chép hình dạng đã được ký là rẻ hơn và
ít rủi ro hơn phát minh một hình dạng thứ hai.

---

## 4. Đề xuất thay đổi chi tiết

### 4.1 `epics.md` — CHÈN một khối, không sửa một dòng nào đang có

**Vị trí:** sau AC cuối của Story 2.11, **TRƯỚC** khối `### ~~Story 2.12: Sync Scrolling~~ — XOÁ`.

🔴 **Vì sao đặt TRƯỚC khối XOÁ, không phải sau:** story sống đứng đúng thứ tự số ngay sau 2.11; bản
ghi lịch sử lùi xuống sau nó như một phụ lục. Đảo lại thì thứ đầu tiên người đọc gặp khi tìm "2.12"
là một khối gạch ngang.

**OLD** *(`epics.md:2655-2659`)*

```
**Then** là command đăng ký, gán phím được

---

### ~~Story 2.12: Sync Scrolling~~ — XOÁ
```

**NEW**

```
**Then** là command đăng ký, gán phím được

---

### Story 2.12: Hạ tầng e2e và cổng còn thiếu

**Covers:** đường đóng cho **11 món nợ hạ tầng e2e** của Epic 2 *(không FR mới)* · liên đới **AD-35**,
**AD-45** · action item **B2** của retro Epic 2

> ⚠️ **Story này sinh từ một action item retro, KHÔNG từ lượt quy hoạch gốc — ghi thẳng thay vì để
> nó trông bình thường.** Cùng khuôn Story 1.22: một cụm khuyết tật có tên đang sống trong một tệp
> retro mà **không tạo tác nào ở tầng quy hoạch chịu trách nhiệm cho chúng**. Khác một điểm và điểm
> đó nặng hơn: cụm này sở hữu **11** món nợ và **không có ngày** cho tới khi cửa chặn ② ra đời.
>
> 🔴 **SỐ HIỆU `2.12` ĐƯỢC TÁI DÙNG.** Khối `~~Story 2.12: Sync Scrolling~~ — XOÁ` ngay dưới đây là
> một **bản ghi lịch sử** *(FR20 rút, Ice ký 2026-08-14)*. Hai story **không liên quan gì nhau**;
> slug trong `sprint-status.yaml` khác nhau nên hai khoá không đụng nhau.
>
> 🔴 **ĐÂY LÀ CỬA CHẶN ② — Epic 3 không mở trước khi story này chạy xong** *(Ice ký 2026-08-18)*.
> Thứ tự Ice chốt: **Story 2.4 → Story 2.12 → Epic 3.**

As a người dựng AuraTranslate,
I want bộ đo e2e nói thật và hai cổng còn thiếu đứng dậy,
So that Epic 3 không dựng một cột dữ liệu mới lên trên một lưới nghiệm thu đang nói dối.

**Vì sao nó là cửa chặn, đo được:** bộ e2e là lưới **DUY NHẤT** bắt được lỗi hình dạng dây — đã chứng
minh **bốn** lần *(2.5: 74/74 vitest xanh trong khi `isConfirmed` luôn `false` trong app thật; 2.7:
382 Rust + 133 vitest đều xanh trong khi dây gãy)*. **Và cùng lúc** nó là một bộ đo nói dối: *"xanh
riêng, đỏ trong bộ"* ở **năm** story *(2.6 · 2.8 · 2.9 · 2.10 · 2.11)*, 8 lượt trọn bộ = 6 xanh · 2
đỏ, trong đó **một lần đỏ chưa bao giờ được chẩn đoán**. Epic 3 thêm một cột dữ liệu mới *(Glossary)*
và một kênh trang trí thứ hai lên **đúng** bề mặt đó.

**Acceptance Criteria:**

**Given** Vite đang chạy nhưng module graph đã vỡ *(vẫn trả 200 cho `/`)*
**When** `devServerIsUp()` chạy
**Then** nó trả `false`, và bộ e2e dừng với một câu nói **đúng nguyên nhân**

**Given** hai spec chạy nối tiếp trong cùng một lượt trọn bộ
**When** spec thứ hai bắt đầu
**Then** không state cấp module nào của panel sống sót từ spec trước
**And** điều đó đúng **do fixture**, không do mỗi spec tự gọi `window.location.reload()`

**Given** một spec chờ một thay đổi trạng thái mà trạng thái cũ và mới có **cùng hình dạng DOM**
**When** nó chờ
**Then** nó chờ **trạng thái đích**, không chờ *"phần tử tồn tại"*

**Given** một spec cần biết một lượt flush đã xong
**When** nó chờ
**Then** nó chờ một **sự kiện quan sát được**, không chờ một khoảng thời gian
**And** không hằng số nào của **AD-35** bị nới

**Given** một ô nhớ cấp module mới thêm vào một tệp state của panel
**When** cổng chạy
**Then** cổng **đỏ** nếu ô đó không đi qua hàm reset tương ứng, hoặc không có một **miễn trừ có tên**

**Given** một cột mới của bảng `segment` ra đời qua một bước di trú
**When** cổng chạy
**Then** cổng **đỏ** nếu cột đó chưa xuất hiện ở mọi chỗ bắt buộc — **không chỉ** ở đường flush mà
cổng-AC8 hiện canh

**Given** bộ e2e trọn bộ chạy trên máy đã thoả điều kiện đã ký
**When** chạy
**Then** kết quả **tái lập được**, và mọi ca đỏ còn lại **có nguyên văn lỗi được bắt**

---

### ~~Story 2.12: Sync Scrolling~~ — XOÁ
```

**Rationale:** đóng khoảng trống quy hoạch bằng đúng hình dạng đã được ký ở Story 1.22, và đặt cảnh
báo tái dùng số hiệu ở nơi người đọc gặp nó **trước** khối gạch ngang, không sau.

### 4.3 `epics.md` — CHÈN khối thứ hai: Story 2.13 *(sửa đổi, Ice duyệt 2026-08-18)*

**Kích hoạt:** cùng lớp vấn đề với §4.1, phát hiện muộn hơn trong cùng phiên. Ice yêu cầu đo chính
xác sổ nợ; lượt đo cho ra một con số đủ để dựng story riêng, và story đó gặp **đúng** khoảng trống
quy hoạch mà §4.1 vừa đóng.

**Vị trí:** **SAU** khối `### ~~Story 2.12: Sync Scrolling~~ — XOÁ`, **trước** `## Epic 3`.
*(Khác §4.1 — 2.13 không tranh số hiệu với khối XOÁ nên không cần đứng trước nó.)*

**Nội dung chèn:** khối `### Story 2.13: Phân loại sổ nợ — 199 mục mồ côi và một luật đang bị phá`,
cùng khuôn đã duyệt ở §4.1 — `Covers:` không FR mới · khối `> ⚠️` khai nguồn · 6 AC dạng Given/When/Then.

**Số đo chống lưng cho khối này** *(đếm thật, 4.691 dòng, HEAD `4b30199`)*:

| | mở | nửa 🟡 | đóng ✅ | tổng | mở không chủ |
|---|---|---|---|---|---|
| Epic 1 | 197 | 23 | 16 | 236 | **156** |
| Epic 2 | 81 | 13 | 13 | 107 | 16 |
| khác | 83 | 14 | 8 | 105 | 27 |
| **TỔNG** | **361** | **50** | **37** | **448** | **199** |

🔴 **Phát hiện lật hướng xử lý, ghi vào đây vì nó đổi phạm vi story:** quy ước `Chủ:` **đang hoạt
động** — Epic 2 đạt **80 %** mục mở có chủ, Epic 1 chỉ **21 %**. ⇒ Không dựng cơ chế định kỳ, không
dựng trần nợ. Đây là **một khối di sản dùng một lần**.

⚠️ **Đính chính một con số đang lưu hành:** retro Epic 2 ghi *"~157 mục, 22 đóng"* — đó là **khoảng
Epic 2**, không phải cả sổ. Retro không sai; hai con số đo hai phạm vi khác nhau. Ghi ra thay vì để
chúng cùng tồn tại mà không ai nói chúng khác nhau ở đâu.

**Rationale:** giữ một quy tắc duy nhất cho cả hai story hạ tầng của Epic 2 — story tồn tại ở tầng
thực thi thì có mặt ở tầng quy hoạch. Một quy tắc áp cho 2.12 mà không áp cho 2.13 là một ngoại lệ
không ai viết ra được lý do.

### 4.2 Không artifact nào khác đổi

`prd.md` · `ARCHITECTURE-SPINE.md` · `EXPERIENCE.md` · bảng FR→Epic · tổng kiểm 132/132 · blurb Epic 2
— **giữ nguyên**, lý do ở §2.2.

---

## 5. Bàn giao

**Phân loại: 🟢 Minor** — thực hiện trực tiếp.

| Việc | Chủ | Điều kiện xong |
|---|---|---|
| Chèn khối §4.1 vào `epics.md` | Dev *(lượt này)* | `grep "^### Story 2.12: Hạ tầng"` = **1**; khối XOÁ **còn nguyên**, không sửa một chữ |
| Commit riêng, message nói **điều đã tìm ra** | Dev, **sau khi Ice duyệt** | `type(scope): câu tiếng Việt` |
| Dev story 2.12 | Ice → dev-story | **Sau khi Story 2.4 đóng** |

**Tiêu chí thành công:** một người mở `epics.md` tìm "Story 2.12" gặp story sống **trước** bản ghi
lịch sử, và biết ngay ba điều: nó là cửa chặn ②, nó không phải sync-scrolling, và 2.4 chạy trước nó.

**Ngoài phạm vi đề xuất này, ghi ra thay vì để trôi:**
- **B5** — rà tồn dư tài liệu quy hoạch sau correct-course 2026-08-14 *(chủ: Winston)*. `prd.md` còn
  mục `#### Panel Source` *(`:401`)*, bảng thuật ngữ *(`:221`)*; `epics.md` còn hàng `FR19 | Epic 1`.
- **B6** — `AD-48` chưa viết, đang chặn AC5 của Story 2.9 *(chủ: Ice → Winston)*.
