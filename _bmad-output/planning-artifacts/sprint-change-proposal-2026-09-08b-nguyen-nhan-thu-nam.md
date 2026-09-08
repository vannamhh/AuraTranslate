# Sprint Change Proposal — 2026-09-08b

**Nguyên nhân *"cần xem"* thứ năm cho FR132 (Story 6.10), và một vế PRD đã hết đúng**

**Người soạn:** `bmad-correct-course`, chạy chế độ **tăng dần** · **Ice duyệt từng phép sửa**
**Baseline:** `b42be3b` (`master`, cây sạch) · **Mọi số đo trong tệp này đo ngày 2026-09-08 trên baseline đó**
**Lượt thứ hai trong ngày.** Lượt đầu — `sprint-change-proposal-2026-09-08-story-6-10.md` — tách Story 6.10
làm đôi (6.10a nền, 6.10 bộ lọc). Lượt này nới AC của nửa còn lại.

---

## 1. Vấn đề

FR132 liệt **bốn** dấu hiệu đưa một Chương vào nhóm *cần xem*. Nó bỏ sót một dấu hiệu **đã tồn tại, đã được
tính, và đang bị vứt đi**: số dòng bị bước chuẩn hoá (FR125) nối lại.

**Phát hiện lúc nào.** Ở **bước 2 (điều tra) của `bmad-build`** cho Story 6.10, trước khi soạn spec. Lượt
điều tra đọc sổ nợ và gặp `deferred-work.md:9478` — một mục **Chủ: Story 6.10** tự ghi rằng nó *"cần Ice
quyết chứ không phải dev tự thêm"*. Lượt build **dừng ở bước 2**, không soạn spec, không đặt `status`,
không chạm `sprint-status.yaml`.

**Vì sao đây là một khoảng trống quy hoạch, không phải một khuyết tật.** Story 6.4 (luật gộp dòng) **tự ghi
ra** trong §Design Notes *"Luật gộp dòng, và cái nó KHÔNG cứu được"* rằng nó sẽ nối **oan** một tiêu đề
không dấu chấm đứng riêng một dòng vào câu kế, khi không có dòng trống ngăn cách. Đó là hư hại **thật** trên
văn bản nguồn, và số dòng bị nối cao bất thường là tín hiệu duy nhất báo nó **trước** khi ghi xuống đĩa —
đúng hình dạng mà FR132 tồn tại để gom. Story 6.4 còn ghi thẳng *"Cổng đã có: 0"*. Nhưng FR132 chưa bao giờ
liệt dấu hiệu ấy, nên không story nào có nghĩa vụ nối nó.

### Bằng chứng

| # | Phép đo | Neo |
|---|---|---|
| ① | `normalize::normalize()` được gọi trên **toàn văn** từng đơn vị, rồi chỉ `.text` được giữ — hai số đếm bị vứt | `src-tauri/src/core/segment/pipeline.rs:644` |
| ② | Cửa sổ `EVIDENCE_WINDOW_BYTES` chỉ áp cho **dải năm ứng viên bảng mã**, không cho lượt pipeline thật | `src-tauri/src/core/segment/encoding.rs:287` |
| ③ | AD-39 đóng băng thứ tự: chuẩn hoá (FR125) là bước **4**, tách Chương (FR14) là bước **5** | `ARCHITECTURE-SPINE.md:478` |
| ④ | `NormalizedPreviewWire.joined_lines` **đã có** trên dây, nhưng là số **theo ứng viên, có cửa sổ** (`window_truncated`) | `src-tauri/src/commands/project.rs:1023,1027` |

⇒ Từ ① và ②: dấu hiệu thứ năm tốn **0 phép tính mới**; việc còn lại là **giữ con số** thay vì vứt.
⇒ Từ ③: trên đường `Blob` + mẫu phân tách, `joined_lines` **không quy về từng Chương được** — và đó là **hệ
quả kiến trúc bắt buộc**, không phải một khuyết tật. Cùng hình dạng `cleanup_match_count` của Story 6.10a,
nên **không cần một `AD` mới**.
⇒ Từ ④: một nguy cơ đặt tên phải ghi ra trước khi ai đó vấp — hai đại lượng khác nhau, đừng trùng tên.

### Vấn đề thứ hai, tìm thấy trong cùng lượt

`prd.md:335` vẫn ghi nguyên nhân thứ tư là *"số Chương tách ra không khớp số đơn vị đầu vào (FR14)"*, trong
khi **ba** nguồn khác đã ghi *"link hỏng"*: `epics.md:92`, AC Story 6.10, `EXPERIENCE.md:142`.

**Vì sao nó sót.** Lượt `correct-course` 2026-09-08 (lượt đầu) kết luận *"**PRD không cần sửa**"* ở `:64` của
chính nó, sau khi đọc `prd.md:333` — câu phát biểu FR132 bằng hai vế *hai con số* và *một thao tác lọc*. Câu
liệt bốn nguyên nhân nằm ở `:335`, **câu kế tiếp**, và không được đọc. Một kết luận đúng về phạm vi nó đã
đọc, sai về phạm vi nó tuyên bố.

---

## 2. Phân tích tác động

### Cấp Epic

**Epic 6 hoàn thành được như kế hoạch.** Chỉ nới AC của **một story chưa bắt đầu** (`6-10: backlog`).
Không thêm epic, không bỏ epic, không định nghĩa lại, không đổi thứ tự. Thứ tự thi hành `6-10a → 6-10` giữ
nguyên; 6.10a đã ở `review`.

**Không epic nào khác bị chạm.** Story 6.18 (đo lại NFR3–NFR5) không phụ thuộc bộ lọc. Epic 7 (TM) không đọc
trạng thái *cần xem*.

### Cấp Story

| Story | Tác động |
|---|---|
| **6.10** — Bộ lọc "cần xem" | **AC nới**: bốn → năm nguyên nhân, cộng một AC mới về *"không đo được khác sạch"* |
| 6.10a — Xem trước theo từng Chương | **Không chạm.** Đã `review`, và không mang mệnh đề *"bốn nguyên nhân"* trong mã |
| 6.4 — Chuẩn hoá | **Không chạm.** Con số đã được tính; chỉ chỗ **tiêu thụ** nó là mới |

### Xung đột tài liệu

| Tài liệu | Kết luận | Chỗ |
|---|---|---|
| **PRD** | **Sửa** — hai việc trong một câu | `prd.md:335` |
| **Epics** | **Sửa** — ba chỗ mang cùng một mệnh đề | `:92` (danh sách FR) · `:587` (UX-DR29) · `:4982` (AC Story 6.10) |
| **UX** | **Sửa** — một chỗ | `EXPERIENCE.md:142` |
| Mockup UX | **Không sửa** | Đo: `grep -ril "cần xem"` trúng đúng `web-import.html`, và `:243-244` chỉ vẽ **hai con số** (*"7 Chương cần xem"* · *"43 Chương sạch"*), không liệt nguyên nhân |
| **Architecture** | **Không sửa** | AD-39 đã nói đúng thứ tự bước và không mâu thuẫn. Không thành phần, lược đồ, hợp đồng API, hay lựa chọn công nghệ nào bị chạm. **Không cần `AD` mới** |
| **Sổ nợ** | **Sửa** — hai mục | `deferred-work.md:9478` · `:9491` |
| CI, hạ tầng, chiến lược test | **Không chạm** | — |

### MVP

**Không bị chạm.** FR132 vẫn là một FR, vẫn cùng hạng, phạm vi nhập không đổi. Bộ lọc vẫn *"không bỏ qua
Chương nào — nó đổi thứ tự chú ý, không đổi phạm vi nhập"*.

### Kỹ thuật

**Chi phí thi công gần bằng không cho vế tính toán**, vì con số đã có. Cái phải làm là nối dây, đúng khuôn
`cleanup_match_count` mà Story 6.10a vừa dựng: một trường trên `flow`, một trường trên
`ChapterSplitPreviewEntryWire`, một vị từ kiểm kiểu phía TS, một chỗ hiển thị.

⚠️ **Hai ràng buộc thi công, ghi ra để lượt build không phải đo lại:**

1. **Trường mới mang `Option<usize>`**, không `usize` — `null` nghĩa *không đo được cho Chương này*, `Some(0)`
   nghĩa *thật sự không có dòng nào bị nối*. Cùng phép sửa Ice đã chốt cho `cleanup_match_count` 2026-09-08.
2. **Đừng đặt tên trần `joined_lines`** trên `ChapterSplitPreviewEntryWire` — tên ấy đã thuộc
   `NormalizedPreviewWire` (`project.rs:1023`) với nghĩa **theo ứng viên bảng mã, có cửa sổ**.

---

## 3. Đường đã chọn, và hai đường đã loại

**Chọn: ① Điều chỉnh trực tiếp** — nới AC của một story chưa bắt đầu. Công **thấp**, rủi ro **thấp**.

**Lý do.** Story chưa bắt đầu nên không có mã nào phải sửa lại. Con số đã được tính sẵn nên chi phí thi công
gần bằng không. Và nới **trước** khi dựng rẻ hơn hẳn nới lần hai sau khi bộ lọc đã có hình dạng bốn nguyên
nhân — thời điểm rẻ nhất để thêm một tín hiệu vào một bộ phân loại là trước khi bộ phân loại tồn tại.

**② Rollback — loại.** Không có gì để rollback: Story 6.10 chưa bắt đầu, và Story 6.10a đã `review` mà không
mang mệnh đề *"bốn nguyên nhân"* ở bất kỳ đâu trong mã.

**③ Rà lại MVP — không cần.** FR132 không đổi hạng, phạm vi nhập không đổi, không vế nào bị hoãn.

---

## 4. Bốn phép sửa, đã áp dụng

Mọi phép sửa dưới đây **đã được Ice duyệt từng cái một** trong chế độ tăng dần và **đã ghi xuống đĩa**.

### 4.1 `prd.md:335` — FR132, hai việc trong một câu

Thêm dấu hiệu **FR125**, và sửa vế thứ tư *"số Chương tách ra không khớp số đơn vị đầu vào (FR14)"* thành
*"link hỏng (FR122)"*. Kèm một khối 🔵 ghi cả hai lý do và ngày.

**Vế "N link ≠ N Chương" không mất:** nó có chủ riêng ở AC4 của Story 6.7, nơi hai con số bằng nhau là một
bằng chứng quan sát được.

**Vì sao gộp hai việc:** cả hai sửa **đúng một câu**. Tách ra là mở cùng một dòng hai lần và để một mệnh đề
sai nằm nguyên thêm một nhịp, trong khi diff vẫn chỉ có một chỗ.

### 4.2 `epics.md` — ba chỗ, cùng một mệnh đề

- **`:92`** (danh sách FR): thêm *"số dòng bị bước chuẩn hoá nối lại bất thường (FR125)"*, và nối một khối 🔵
  **thứ hai** ghi rõ đây là lượt `correct-course` thứ hai trong ngày, kèm phép đo `pipeline.rs:644`.
- **`:587`** (UX-DR29): *"bốn nguyên nhân"* → *"**năm** nguyên nhân"*, thêm mục thứ năm.
- **`:4982`** (AC Story 6.10): *"gom **bốn** nguyên nhân"* → *"gom **năm** nguyên nhân"*, mục thứ năm phát
  biểu bằng **cùng phép so trung vị** như vế *"ngắn bất thường"* — một cơ chế cho nhiều tín hiệu, không hằng
  số phù thuỷ nào.

**Sửa cả ba cùng lượt** vì chúng là cùng một mệnh đề trong cùng một tệp; sửa lẻ là để lại chỗ lệch nội bộ.

### 4.3 `epics.md` — một AC MỚI cho Story 6.10

```text
Given một dấu hiệu KHÔNG ĐO ĐƯỢC cho một Chương
      (AD-39 đặt chuẩn hoá (FR125) và làm sạch (FR124) TRƯỚC tách Chương (FR14),
       nên trên đường tệp/dán tay + mẫu phân tách chỉ Chương đầu có số đếm thật)
When  phân loại
Then  Chương đó KHÔNG được xếp vào nhóm sạch dựa trên dấu hiệu ấy
And   trạng thái "không đo được" PHÂN BIỆT ĐƯỢC với "đã đo và sạch"
```

🔴 **Đây là chỗ lượt này vượt khỏi phạm vi *"thêm nguyên nhân thứ năm"*, và Ice đã duyệt riêng.** Không có AC
này thì trung vị của N−1 số `0` là `0`, và màn hình khẳng định *"M Chương sạch"* cho những Chương **chưa ai
đo** — đúng lớp rỗng-im-lặng mà `AGENTS.md` §Known pitfalls dẫn ra ba lần (Story 1.16 · 2.10 · 3.9). AC áp
cho **cả** nguyên nhân *"luật làm sạch xoá quá nhiều"* **lẫn** nguyên nhân FR125 mới, nên viết một lần ở AC
rẻ hơn để mỗi story tự phát hiện lại.

### 4.4 `deferred-work.md` — hai mục

- **`:9478`** → 🟡 **ĐÓNG MỘT NỬA**. Vế **quy hoạch** đóng (FR125 nay có tên ở **năm** chỗ, khớp nhau); vế
  **thi công** còn hở, **Chủ vẫn Story 6.10**. Kèm một phép sửa tại chỗ: lời dặn *"đếm trên TOÀN Chương"*
  của chính mục ấy đọc như thể phải dựng mới, trong khi lượt đó đã chạy sẵn.
- **`:9491`** → 🔵 **SỬA TẠI CHỖ, mục vẫn MỞ nhưng phạm vi hẹp lại**. Hai mệnh đề của nó hết đúng, và chúng
  kéo nhau: ① *"cần một lượt chạy `normalize()` đầy đủ"* — lượt đó đã chạy; ② *"Story 6.10 có bối cảnh ở
  **cấp Thư viện**"* — sai, FR132 và AC đều đặt bộ lọc ở màn xem trước nhập, nên kết luận *"làm SAU khi
  Chương đã có trong `project.db`"* **rơi theo**. Phần còn lại vẫn thật và hẹp hơn: hai số đếm ở **tầng 1**
  vẫn là số có cửa sổ.

**Không xoá mục nào**, đúng luật sổ nợ: đóng bằng chữ, đóng nửa thì 🟡 kèm phần còn hở.

---

## 5. Nghiệm thu lượt sửa này

| Phép kiểm | Kết quả |
|---|---|
| `npm run check:debt-owner` | **XANH** — `0/453` mục mở thiếu `Chủ:` · 696 mục tổng · 86 nửa · 147 đóng · 15 ca tự kiểm + 1 + 5 đều đúng |
| Năm chỗ nói về nguyên nhân có khớp nhau không | **Có** — `prd.md:335` · `epics.md:92` · `epics.md:587` · `epics.md:4982` · `EXPERIENCE.md:142` |
| Mockup có cần sửa không | **Không** — đo bằng `grep -ril`, chỉ `web-import.html:243-244` trúng và nó chỉ vẽ hai con số |

⚠️ **Giới hạn thật của lượt nghiệm thu này, ghi ra thay vì để người sau tưởng đã được xét:** lượt này **chỉ
sửa tài liệu quy hoạch và sổ nợ**, không chạm một dòng mã nào. Mười cổng còn lại, `cargo test` và `vitest`
**không được chạy** vì không có gì cho chúng kiểm. Con số nền (`1.255` ca Rust · `928` ca vitest, ghi ở
`spec-6-10a:154`) vẫn là **số đã ghi, chưa đo lại** — và nó chép lại *"44 binary"* trong khi
`ls src-tauri/tests/*.rs` đếm **40** tệp hôm nay, đúng lỗi mà `:168` của chính tệp đó cảnh báo. Lượt build
sau **phải tự chạy và tự đo**, đừng chép tiếp.

---

## 6. Bàn giao

**Hạng thay đổi: NHỎ (Minor).** Không đụng mã, không đụng kiến trúc, không đụng MVP; chỉ nới AC của một
story chưa bắt đầu và sửa bốn tài liệu cho khớp nhau.

**Giao cho: agent Developer** (`bmad-build`), chạy lại từ đầu cho Story 6.10 trên AC **đã nới**.

**Thứ lượt build đó thừa hưởng, không phải hỏi lại** — ba quyết định Ice chốt ở bước 2 ngày 2026-09-08, ghi
đầy đủ ở `implementation-artifacts/ho-so-dieu-tra-6-10-bo-loc-can-xem-2026-09-08.md` §9:

1. **Quy tắc ngưỡng: hàng rào Tukey** — *ngắn bất thường* = dưới `Q1 − 1,5 × IQR`; *xoá quá nhiều* (và nay
   *nối nhiều*) = trên `Q3 + 1,5 × IQR`. Một cơ chế cho mọi tín hiệu so-tương-đối. Hằng `1,5` là quy ước
   thống kê **có tên**, không phải số tự đúc. ⚠️ Tứ phân vị vô nghĩa khi N nhỏ, và `N = 1` thì *"các Chương
   khác"* không tồn tại — cả hai ca rơi vào AC *"không đo được"* mới ở §4.3.
2. **Hợp âm `⌘↵`: giữ `Mod+Alt+Enter`**, nợ `deferred-work.md:9943` (**Chủ: Ice**) đứng nguyên. AC mô tả
   đích đến và không sai vì đường đi chưa tới.
3. **`cleanup_match_count` đổi thành `Option<usize>`** — và trường FR125 mới đi theo cùng khuôn.

**Điều kiện nghiệm thu của lượt bàn giao:** spec Story 6.10 phủ **năm** nguyên nhân cộng AC *"không đo được
khác sạch"*, và không có hằng số ngưỡng nào ngoài `1,5` của hàng rào Tukey.

**`sprint-status.yaml`: KHÔNG đổi.** Không epic nào thêm/bớt/đánh số lại, không story nào thêm/bớt.
`6-10-bộ-lọc-cần-xem: backlog` giữ nguyên — nới AC của một story không đổi trạng thái của nó.
