# Sprint Change Proposal — Epic 4 (AI) lùi xuống sau đường nhập: thứ tự thực thi 1→2→3→5→6→4

**Ngày:** 2026-08-13 · **Chủ dự án:** Ice · **Chủ trì:** John (Product Manager)
**Chế độ:** Batch — cả gói trình một lần
**Baseline:** `8ac9ccb` *(master)* · cây làm việc **sạch** lúc bắt đầu

> **Đây là thay đổi TRÌNH TỰ, không phải thay đổi phạm vi.** Không FR nào bị cắt, không AC
> nào bị sửa nội dung, không epic nào bị bỏ. v1 vẫn gồm trọn mười nhóm năng lực. Thứ duy
> nhất đổi là **thứ tự thực thi** — và một hệ quả kèm theo: mỏ neo của một quyết định hoãn.

---

## 1. Vấn đề

### 1.1 Trigger

Không phải một lỗi phát hiện lúc cài đặt. Đây là **thay đổi chủ đích của chủ dự án**,
nêu ngày 2026-08-13:

> *"tôi muốn đổi epic 5 khai triển trước epic 4 có được không"*

Khi được hỏi động cơ, Ice chọn: **"Muốn thấy sản phẩm dùng được sớm"** — Library và Chế độ
đọc làm ứng dụng trông như một sản phẩm thật, còn AI thì chưa đổi được cảm giác đó.

**Phân loại:** *Strategic pivot* — điều chỉnh trình tự để đạt trạng thái dùng được sớm hơn.
Không phải giới hạn kỹ thuật, không phải hiểu sai yêu cầu, không phải phương án hỏng.

### 1.2 Phát biểu vấn đề

Thứ tự epic hiện tại (`1 → 2 → 3 → 4 → 5 → 6 → …`) đặt **Epic 4 — AI mở & Smart RAG
Injector** vào vị trí thứ tư, trước **Epic 5 — Library** và **Epic 6 — Đường nhập**.

Hệ quả: sau khi hoàn tất Epic 4, sản phẩm biết gọi AI dịch từng segment, nhưng **chưa có
kho tác phẩm, chưa có đường nhập hàng loạt, chưa có chế độ đọc**. Một người dùng thử ở thời
điểm đó vẫn phải dán tay từng Chương một — đúng đường nhập tối thiểu của FR13 từ Epic 1.

### 1.3 Bằng chứng

Ba bằng chứng lấy từ chính tài liệu quy hoạch, không từ suy đoán:

**① `build-sequence.md:20` phát biểu luận điểm ngược với thứ tự hiện tại.**

> *"AuraTranslate đặt cược rằng giá trị nằm ở **môi trường làm việc bao quanh AI**, không
> phải ở bản thân AI. […] Trình tự này phản ánh đúng luận điểm đó: **xây môi trường trước,
> cắm AI vào sau.**"*

Epic 5 và Epic 6 **là** môi trường; Epic 4 **là** AI. Thứ tự hiện tại đi ngược khẩu hiệu
của chính nó một nhịp.

**② Story 4.1 có một AC hôm nay không nghiệm thu được thật.**

`epics.md` §Story 4.1, AC thứ tư:

> **Given** ứng dụng chạy mà chưa cấu hình nhà cung cấp AI nào
> **When** dùng
> **Then** **Library**, Workspace, tra cứu, Glossary và **toàn bộ năng lực ngoài C6/C7**
> chạy đầy đủ

Chạy Epic 4 ở vị trí thứ tư nghĩa là nghiệm thu vế *"Library chạy đầy đủ"* trên một Library
**chưa tồn tại**. Đó là một AC **xanh vì rỗng** — đúng lớp lỗi mà `project-context.md`
§Critical Don't-Miss Rules gọi tên là *"rỗng im lặng"* và cấm ở mọi tầng khác.

**③ `epics.md` §Epic 6 gọi Epic 6 là bề mặt đầu tiên người dùng chạm vào.**

> *"Đây là **bề mặt đầu tiên người dùng chạm vào sản phẩm**, và là nơi hai lỗi đắt nhất của
> ứng dụng có thể xảy ra mà không báo gì cả."*

Nếu mục tiêu là *"sản phẩm dùng được sớm"* thì Epic 5 một mình chưa đủ — Library trên ba
Tác phẩm dán tay không cho cảm giác đó. **Cặp 5 + 6** mới cho.

---

## 2. Phân tích tác động

### 2.1 Thứ tự đề xuất

```
Cũ:  1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10
Mới: 1 → 2 → 3 → 5 → 6 → 4 → 7 → 8 → 9 → 10
                   ▲   ▲   ▲
                   │   │   └─ Epic 4 (AI) lùi hai bậc
                   └───┴───── Epic 5 (Library) + Epic 6 (Đường nhập) lên hai bậc
```

Kèm một ngoại lệ: **Story 4.1 tách khỏi Epic 4 và chạy ngay sau Epic 3** (lý do ở §3.4).

### 2.2 Tác động epic — kiểm phụ thuộc chéo

Quét toàn văn `epics.md` (6.500 dòng) cho mọi tham chiếu chéo epic:

| Kiểm | Cách kiểm | Kết quả |
|---|---|---|
| Epic 5 có cần Epic 4 không? | Quét §Epic 5 (`epics.md:3398-3991`) cho `AI` · `ai/` · `C6` · `C7` | **0 lần** ✅ |
| Epic 6 có cần Epic 4 không? | Quét §Epic 6 (`epics.md:3991-4806`) cho cùng bộ từ khoá | **0 lần** ✅ |
| Epic 4 có cần Epic 5 không? | AC Story 4.1 khai chiều hợp lệ | `ai/` đọc `glossary/`, `tm/`, `segment/` — **không có `library/`** ✅ |
| Epic 4 có cần Epic 3 không? | RAG Injector cần Glossary đã chốt | **Có** — Epic 3 vẫn đứng trước ✅ |
| Epic 6 có cần Epic 5 không? | Đường nhập ghi Chương vào Library | **Có** — thứ tự mới cho 5 liền trước 6 ✅ |
| Epic 7 (TM) cần Epic 4? | Nửa TM của FR70, *"AI học văn phong"* | **Có** — 4 vẫn đứng trước 7 ✅ |
| Epic 8 (Reviewer) cần Epic 4? | FR95 → thuật ngữ vào prompt Chương sau | **Có** — 4 vẫn đứng trước 8 ✅ |
| Epic 9 (AI Proofreader) cần Epic 4? | Toàn bộ epic | **Có** — 4 vẫn đứng trước 9 ✅ |

**Kết luận: thứ tự mới thoả mọi chiều phụ thuộc. Không AC nào vỡ, không story nào phải viết lại.**

### 2.3 Tác động story

**Không story nào bị thêm, bớt, hay sửa nội dung.** Tổng vẫn là 12 story ở Epic 4, 14 ở
Epic 5, 18 ở Epic 6.

Một story đổi **vị trí thực thi**, không đổi nội dung:

| Story | Vị trí cũ | Vị trí mới | Lý do |
|---|---|---|---|
| **4.1** — Module `ai/` cô lập và test cưỡng chế ranh giới | Đầu Epic 4 | **Ngay sau Epic 3** | §3.4 |

### 2.4 Tác động PRD

**Không có.** `prd.md` không nói *"Epic N"* ở bất kỳ đâu — nó nói *"Giai đoạn N"*, và theo
phương án đã chốt ở §3.2, số hiệu Giai đoạn **không đổi**. Cả 17 tham chiếu *"Giai đoạn"*
trong `prd.md` vẫn đúng nguyên văn.

Đặc biệt kiểm hai chỗ nhạy nhất và cả hai **vẫn đúng**:

- `prd.md:1128` (Q4) — *"đo trên thư viện thật **sau Giai đoạn 3b**"*. Giai đoạn 3b là Epic 6,
  vẫn đứng trước Epic 4 lẫn sau Epic 5. Câu này không những vẫn đúng mà còn **được củng cố**:
  §2.6 dưới đây.
- `prd.md:1133` (Q9) — *"đo trên máy thật… Giai đoạn 2 trở đi, ngay khi Workspace bốn panel
  dựng xong"*. Workspace bốn panel là Story 1.14, đã `done`. Không liên quan.

**MVP không đổi.** Phạm vi v1 vẫn gồm trọn mười nhóm năng lực — `SPEC.md:89` và `risks.md:9`
đều ghi R1 được giảm nhẹ **bằng trình tự**, không bằng cắt phạm vi. Thay đổi này là một lượt
dùng đúng công cụ đó.

### 2.5 Tác động Architecture

**Đúng một dòng.** Quét cả 20 tham chiếu *"Giai đoạn"* trong `ARCHITECTURE-SPINE.md`, chỉ
một mỏ neo trong bảng §Deferred trỏ sai đi vì thay đổi này:

| Dòng | Hoãn cái gì | Mỏ neo hôm nay | Vì sao sai đi |
|---|---|---|---|
| `:898` | **Cách phân tích khung SSE** | *"Giai đoạn 2 — rà giấy phép trước khi thêm"* | SSE chỉ dùng cho streaming AI (AD-22). Nội dung đó dời sang cuối, mỏ neo phải dời theo |

Ba mỏ neo *"Giai đoạn 2"* còn lại **không đổi** vì nội dung của chúng đứng yên:
- `:894` Ngưỡng WAL + nhịp flush (AD-12/AD-35) → Editor, ở Epic 2
- `:897` Thư viện editor cho panel Editor → Epic 2

Năm mỏ neo *"Giai đoạn 3"* (`:887` A12 · `:888` A13 · `:889` HTTP client `Fetcher` ·
`:891` màn xem trước FR123 · `:895` Q4 · `:899` ảo hoá danh sách dài · `:905` chỉ mục FTS)
cũng **không đổi** — tất cả thuộc Epic 5/6, và cặp đó giữ nguyên thứ tự tương đối với nhau.

**Không AD nào bị sửa.** Không bất biến kiến trúc nào đổi, nên theo `project-context.md`
(*"Đổi một bất biến kiến trúc là một AD MỚI"*) — **thay đổi này không sinh AD mới**.

### 2.6 Một mục cũ hơn thân của nó — sửa nhân thể

Trong lúc rà, phát hiện `epics.md:892-893` (§Epic List, ghi chú Epic 5) mang một mệnh đề
**đã hết đúng**:

> ⚠️ *"Rủi ro lịch trình đã biết, **chưa xử lý** — Q4 không đóng được ở đây. […]
> **Ba đường xử lý, chưa chọn:** (a) đảo Epic 5 ↔ Epic 6 · (b) giữ thứ tự nhưng dời việc đo
> NFR3/4/5 xuống sau Epic 6 · (c) tách Epic 5 làm đôi. Cho tới khi chọn, Epic 5 đóng lại với
> ba ngưỡng vẫn treo."*

Nhưng thân tài liệu **đã đi trước dòng tóm tắt đó**: **Story 6.18** tồn tại, tên là *"Đo lại
NFR3, NFR4, NFR5 trên thư viện 5.000 Chương thật"*, và AC của nó ghi thẳng:

> **And** đây là điều kiện Story 5.14 không có được: ở Epic 5 chưa có đường nào tạo ra ngần
> ấy Chương

⇒ **Đường (b) đã được chọn và đã viết ra rồi.** Chỉ có dòng tóm tắt ở §Epic List là cũ hơn
thân nó. Theo luật *"Khi một mệnh đề hết đúng, SỬA TẠI CHỖ thay vì để nó lặng lẽ sai"*
(`project-context.md` §Code Quality), proposal này sửa luôn — xem §4.3.

**Hệ quả cho quyết định đang xét:** cái giá của việc dời Epic 4 **nhỏ hơn** ước lượng đầu.
Nó không mở lại một vết thương nào; nó chỉ làm số sơ bộ của Story 5.14 nằm chờ Story 6.18
**đúng một epic** — và ở thứ tự mới, 5.14 và 6.18 nằm **liền kề nhau**, gần hơn hôm nay.

### 2.7 Tác động UX

**Không có.** Sáu tham chiếu *"Giai đoạn"* trong `EXPERIENCE.md` là tiêu đề các mục bổ sung
(*"bề mặt của Giai đoạn 1 và 2"*, …) — chúng mô tả **bề mặt nào thuộc giai đoạn nào**, không
mô tả thứ tự dựng. Số hiệu giai đoạn không đổi ⇒ các tiêu đề này vẫn đúng.
`DESIGN.md`: 0 tham chiếu.

### 2.8 Tác động kỹ thuật (mã, CI, hạ tầng)

**Không có.** Không dòng mã nào tồn tại cho Epic 4, 5 hay 6 hôm nay — cả ba đều `backlog`.
CI, cổng tĩnh, `.githooks/pre-push` không mã hoá thứ tự epic ở bất kỳ đâu.

---

## 3. Phương án và phán quyết

### 3.1 Ba đường đã xét

| # | Phương án | Công sức | Rủi ro | Phán quyết |
|---|---|---|---|---|
| **1** | **Direct Adjustment** — đổi thứ tự thực thi, giữ nguyên mọi nội dung | **Thấp** (6 chỗ sửa, 4 tệp) | **Thấp** (mọi phụ thuộc đã kiểm, 0 vi phạm) | ✅ **CHỌN** |
| **2** | **Rollback** — hoàn tác việc đã xong để đơn giản hoá | — | — | ❌ **Không áp dụng.** Không có gì để hoàn tác: Epic 4/5/6 chưa có một dòng mã |
| **3** | **MVP Review** — cắt hoặc dời phạm vi | Cao | Cao | ❌ **Không cần.** Vấn đề là trình tự, không phải phạm vi. `SPEC.md:89` và `risks.md:9` đã chốt: R1 giảm nhẹ **bằng trình tự**, không bằng cắt |

### 3.2 Quyết định phụ: số hiệu Giai đoạn được ĐÓNG BĂNG

**Ice chốt 2026-08-13.** Số hiệu Giai đoạn (`1`, `2a`, `2b`, `2c`, `3a`, `3b`, `4`…) trở
thành **tên định danh**, không phải thứ tự. Thứ tự thực thi được khai **tường minh thành một
cột riêng**.

Căn cứ số đo: đánh số lại buộc phải rà và sửa **~40 tham chiếu** *"Giai đoạn N"* rải trên
6 tệp (`prd.md` 17 · `ARCHITECTURE-SPINE.md` 20 · `SPEC.md` 7 · `requirements.md` 3 ·
`risks.md` 3 · `EXPERIENCE.md` 6), và **mỗi chỗ sót là một mỏ neo trỏ sai âm thầm** — một
quyết định hoãn được mở lại nhầm lúc, không cổng nào đỏ. Đóng băng chạm **6 chỗ trên 4 tệp**.

Tỷ lệ: **~40 chỗ rủi ro → 6 chỗ.**

### 3.3 Cái giá phải trả — nói thẳng

**AI lùi từ vị trí thứ 4 xuống thứ 6.** Nếu có ai (kể cả Ice, vài tuần nữa) muốn thấy
AuraTranslate dịch được bằng AI để tin dự án đi đúng hướng, họ phải chờ thêm hai epic
(Epic 5: 14 story · Epic 6: 18 story).

Đây là đánh đổi **"chứng minh luận điểm sản phẩm"** lấy **"chứng minh năng lực AI"**.
`build-sequence.md:20` cho thấy Ice đã đặt cược vào vế thứ nhất từ đầu; thay đổi này làm
trình tự khớp với cược đó thay vì lệch một nhịp.

Một khoản lợi kèm theo: mũi thăm dò SSE (`reqwest-sse` / `sseer`, **cả hai chưa xác nhận
giấy phép** — `ARCHITECTURE-SPINE.md:898`) có thêm thời gian đi qua cửa rà NFR15 thay vì bị
ép ở vị trí thứ tư.

### 3.4 Ngoại lệ bắt buộc: Story 4.1 phải đi trước

**Story 4.1 không được đi cùng phần còn lại của Epic 4.**

Nội dung của nó là một module `ai/` rỗng cộng một **test ranh giới cưỡng chế AD-13**
(*không module nào ngoài `ai/` được phụ thuộc `ai/`*). Nếu để nó lùi xuống sau Epic 6, thì
**hai epic — 32 story — được viết trước khi ranh giới `ai/` có người canh**.

Đây đúng loại mà `build-sequence.md:42` gọi tên: *"rẻ nếu làm từ dòng code đầu tiên, rất đắt
nếu vá sau"*. Và hậu quả của việc vá sau đã được `epics.md` ghi: vi phạm AD-13 làm **FR77
chết** (*chạy đầy đủ khi không cấu hình AI*), mà nó **chỉ lộ ra khi một người dùng không có
API key thử** — tức sau khi phát hành.

⇒ **Story 4.1 chạy ngay sau Epic 3.** Nó không cần bất cứ thứ gì từ Epic 4/5/6; AC của nó
chỉ đòi module `ai/` tồn tại và test ranh giới đỏ được.

**Một điều chỉnh AC kèm theo** (§4.5): AC cuối của Story 4.1 hôm nay viết *"bộ test của
Epic 1, 2 và 3"*. Ở vị trí mới, câu đó vẫn đúng nguyên văn tại thời điểm chạy — nhưng khi
Epic 5 và 6 xong, ranh giới cần được canh trên cả chúng. Đề xuất mở rộng AC, **không thu hẹp**.

---

## 4. Đề xuất sửa chi tiết

**Bảy chỗ, bốn tệp.** Trình theo thứ tự quyền lực của tài liệu: `build-sequence.md` (bản chốt)
trước, rồi `epics.md`, `ARCHITECTURE-SPINE.md`, `sprint-status.yaml`.

> 🔵 **Sửa 2026-08-13, lúc áp:** bản đầu của proposal này ghi *"sáu chỗ"* và bỏ sót §4.7.
> §2.5 đã chỉ đích danh `ARCHITECTURE-SPINE.md:898`, nhưng §4 chỉ liệt kê **bản sao** của mỏ
> neo đó trong `build-sequence.md` (§4.2). Hai tệp mang **hai hàng SSE riêng**, và sửa một
> hàng để hàng kia nói ngược lại là đúng thứ mà proposal này tồn tại để chống. Đã áp cả hai.

---

### 4.1 `_bmad-output/specs/spec-AuraTranslate/build-sequence.md` — bảng Giai đoạn

**Vị trí:** dòng 5–14 (bảng chính)

**CŨ:**

```markdown
| Giai đoạn | Nội dung | Năng lực | Trạng thái |
|---|---|---|---|
| **0** | Bốn mũi thăm dò: … | — | ✅ **Hoàn tất 2026-08-02** |
| **1** | Embedded Dictionary … | CAP-3, một phần CAP-2 | **Kế tiếp** |
| **2** | Panel Editor + AI Translation (BYOK/local) + Glossary + Smart RAG Injector | CAP-2, CAP-4, CAP-6 | |
| **3** | Library: … **và toàn bộ đường nhập** … | CAP-1, CAP-9 | |
| **4** | Translation Memory … | CAP-5 | |
```

**MỚI:**

```markdown
> 🔵 **Cập nhật 2026-08-13 — số hiệu Giai đoạn là TÊN, không phải thứ tự.** Cột *Thứ tự*
> mới là nguồn sự thật cho trình tự thực thi. Lý do tách hai: ~40 tham chiếu *"Giai đoạn N"*
> rải trên 6 tệp quy hoạch đang dùng số hiệu làm **mỏ neo cho các quyết định hoãn**; đánh số
> lại thì mỗi chỗ sót là một mỏ neo trỏ sai mà không cổng nào đỏ.

| Giai đoạn | Nội dung | Năng lực | Thứ tự | Trạng thái |
|---|---|---|---|---|
| **0** | Bốn mũi thăm dò: … | — | — | ✅ **Hoàn tất 2026-08-02** |
| **1** | Embedded Dictionary … | CAP-3, một phần CAP-2 | **1** | **Đang chạy** |
| **2** | Panel Editor + Glossary | CAP-2, CAP-6 | **2** | |
| **2c** | AI Translation (BYOK/local) + Smart RAG Injector | CAP-4 | **6** ← dời | |
| **3** | Library + **toàn bộ đường nhập** … | CAP-1, CAP-9 | **3** | |
| **4** | Translation Memory … | CAP-5 | **4** | |
| **5** | Export/Import `.docx`/`.md` … | CAP-8 | **5** | |
| **6** | AI Proofreader | CAP-7 | **7** | |
| **7** | Đóng gói, phát hành | CAP-10 | **8** | |
```

> ⚠️ **Lưu ý biên tập:** bảng trên rút gọn cột *Nội dung* cho dễ đọc trong proposal. Khi
> áp, **giữ nguyên văn** mô tả của từng hàng — chỉ tách CAP-4 khỏi hàng **2** thành hàng
> **2c** mới, và thêm cột *Thứ tự*.

**Bổ sung một đoạn ngay dưới bảng:**

```markdown
**Thứ tự thực thi đầy đủ (chốt 2026-08-13):**

`1 → 2a → 2b → 3a → 3b → 2c → 4 → 5 → 6 → 7`

tức theo epic: **1 → 2 → 3 → 5 → 6 → 4 → 7 → 8 → 9 → 10**.

**Ngoại lệ:** Story 4.1 (module `ai/` cô lập + test cưỡng chế AD-13) tách khỏi Giai đoạn 2c
và chạy **ngay sau Giai đoạn 2b**. Lý do: ranh giới AD-13 thuộc loại *rẻ nếu làm từ dòng
code đầu tiên, rất đắt nếu vá sau* — để nó lùi cùng 2c nghĩa là 32 story được viết trước khi
ranh giới `ai/` có người canh.
```

**Lý do:** đây là bản chốt về trình tự (`epics.md:784` — *"quyền hơn PRD §10"*), nên nó phải
mang thay đổi trước tiên, và phải mang cả **lý do** chứ không chỉ kết quả.

---

### 4.2 `build-sequence.md` — mỏ neo quyết định hoãn

**Vị trí:** dòng 32–36, bảng *"Điểm mở lại các quyết định đã hoãn"*

**CŨ:**

```markdown
| **2** | Thư viện editor cho panel Editor · cách phân tích khung SSE (rà giấy phép trước) · ngưỡng kích thước WAL buộc checkpoint · nhịp auto-save cụ thể đạt NFR18 mà không phạm NFR2 |
```

**MỚI:**

```markdown
| **2** | Thư viện editor cho panel Editor · ngưỡng kích thước WAL buộc checkpoint · nhịp auto-save cụ thể đạt NFR18 mà không phạm NFR2 |
| **2c** | 🔵 **2026-08-13:** cách phân tích khung SSE (rà giấy phép trước) — dời từ Giai đoạn 2 sang đây cùng lượt dời CAP-4. `reqwest-sse` và `sseer` vẫn **chưa xác nhận giấy phép**; cửa rà NFR15 **không đổi**, chỉ mở muộn hơn |
```

**Lý do:** SSE chỉ phục vụ streaming AI (AD-22). Để mỏ neo ở Giai đoạn 2 sau khi nội dung
dời đi là dựng một cái hẹn không ai đến — và nó sẽ được đọc như *"phải rà giấy phép SSE
trước khi làm Editor"*, sai cả hai vế.

---

### 4.3 `epics.md` §Epic List — bảng epic

**Vị trí:** dòng 782–800

**CŨ:**

```markdown
**Mười epic**, bám `build-sequence.md` (bản chốt, quyền hơn PRD §10). Hai giai đoạn bị tách vì có ranh giới rủi ro thật; các giai đoạn còn lại giữ nguyên làm một epic.

| Epic | Giai đoạn | FR | Vì sao đứng riêng |
|---|---|---|---|
| 1 | 1 | 27 | Mốc giá trị sớm nhất — bằng QuickTranslator, trên macOS |
| 2 | 2a | 9 | Editor là chỗ AD-31/AD-35/AD-12 hội tụ; có mũi thăm dò riêng |
| 3 | 2b | 11 | Miền Glossary, cưỡng chế bởi AD-20/AD-36 |
| 4 | 2c | 14 | `ai/` phải cô lập được **bằng test** (AD-13 → FR77) |
| 5 | 3a | 17 | Library + tầng dữ liệu dẫn xuất |
| 6 | 3b | 16 | **Ranh giới rủi ro:** hai giả định chưa đo (A12, A13), hai lớp lỗi im lặng |
…
```

**MỚI:**

```markdown
**Mười epic**, bám `build-sequence.md` (bản chốt, quyền hơn PRD §10). Hai giai đoạn bị tách vì có ranh giới rủi ro thật; các giai đoạn còn lại giữ nguyên làm một epic.

> 🔵 **Cập nhật 2026-08-13 — số Epic là TÊN, cột *Thứ tự* là trình tự.** Epic 4 (AI) lùi
> xuống sau Epic 6 theo quyết định của Ice: *xây môi trường trước, cắm AI vào sau*
> (`build-sequence.md:20`). Không FR nào bị cắt, không AC nào đổi nội dung.
> Xem `sprint-change-proposal-2026-08-13b-thu-tu-epic.md`.

| Epic | Giai đoạn | Thứ tự | FR | Vì sao đứng riêng |
|---|---|---|---|---|
| 1 | 1 | **1** | 27 | Mốc giá trị sớm nhất — bằng QuickTranslator, trên macOS |
| 2 | 2a | **2** | 9 | Editor là chỗ AD-31/AD-35/AD-12 hội tụ; có mũi thăm dò riêng |
| 3 | 2b | **3** | 11 | Miền Glossary, cưỡng chế bởi AD-20/AD-36 |
| 4 | 2c | **6** ← dời | 14 | `ai/` phải cô lập được **bằng test** (AD-13 → FR77). ⚠️ **Story 4.1 tách ra chạy ở thứ tự 3½**, ngay sau Epic 3 — xem ghi chú Epic 4 |
| 5 | 3a | **4** | 17 | Library + tầng dữ liệu dẫn xuất |
| 6 | 3b | **5** | 16 | **Ranh giới rủi ro:** hai giả định chưa đo (A12, A13), hai lớp lỗi im lặng |
| 7 | 4 | **7** | 10 | Translation Memory |
| 8 | 5 | **8** | 13 | Cầu nối Reviewer |
| 9 | 6 | **9** | 7 | Ứng viên cắt số 1 nếu R1 nổ — phải tách được sạch |
| 10 | 7 | **10** | 8 | Phát hành |
```

---

### 4.4 `epics.md` §Epic List — ghi chú Epic 5, mệnh đề đã hết đúng

**Vị trí:** dòng 892–893

**CŨ:**

```markdown
- ⚠️ **Rủi ro lịch trình đã biết, chưa xử lý — Q4 không đóng được ở đây.** […]
  **Ba đường xử lý, chưa chọn:** *(a)* đảo Epic 5 ↔ Epic 6 · *(b)* giữ thứ tự nhưng **dời việc đo NFR3/4/5 xuống sau Epic 6** · *(c)* tách Epic 5 làm đôi — FR1–FR7 lên trước đường nhập, FR8/FR9 + FR11/FR119/FR120 xuống sau. Cho tới khi chọn, **Epic 5 đóng lại với ba ngưỡng vẫn treo**.
```

**MỚI:**

```markdown
- ⚠️ **Q4 không đóng được ở epic này — và đó là thiết kế, không phải chỗ hở.** Điều kiện đóng
  `[A6] [A7] [A8]` là *"đo trên thư viện thật **5.000 Chương**"*. Không có đường nào tạo ra
  ngần ấy Chương trước Epic 6 — đường nhập tối thiểu của Epic 1 chỉ dán tay từng Chương. Sinh
  dữ liệu giả đo được **tốc độ** nhưng không đo được thứ NFR8 tồn tại để bảo vệ: phân bố dấu
  tiếng Việt thật (`má / ma / mà / mả / mã / mạ`). Cùng lớp vấn đề áp cho **bảng chờ Glossary
  của Epic 3**.
  🔵 **Sửa 2026-08-13 — mệnh đề "ba đường xử lý, chưa chọn" đã hết đúng.** Đường *(b)* đã
  được chọn và đã viết ra: **Story 6.18** (*"Đo lại NFR3, NFR4, NFR5 trên thư viện 5.000
  Chương thật"*) mang AC ghi thẳng *"đây là điều kiện Story 5.14 không có được"*. Thân tài
  liệu đã đi trước dòng tóm tắt này từ lúc Epic 6 được viết. ⇒ **Story 5.14 ghi số sơ bộ,
  Story 6.18 đóng Q4.** Ở thứ tự thực thi mới (chốt 2026-08-13) hai story này nằm **liền kề**,
  gần hơn thứ tự cũ.
```

**Lý do:** `project-context.md` §Code Quality — *"Khi một mệnh đề hết đúng, SỬA TẠI CHỖ thay
vì để nó lặng lẽ sai — kèm ngày và lý do đổi."* Dòng cũ đang mời người đọc tưởng một rủi ro
còn treo, trong khi nó đã có chủ.

---

### 4.5 `epics.md` §Story 4.1 — AC cuối, mở rộng phạm vi canh

**Vị trí:** §Story 4.1, AC cuối cùng

**CŨ:**

```markdown
**Given** bộ test của Epic 1, 2 và 3
**When** chạy trong một môi trường **không có cấu hình AI**
**Then** toàn bộ vẫn xanh
```

**MỚI:**

```markdown
**Given** bộ test của **mọi epic đã hoàn tất tại thời điểm chạy story này** — hôm nay là
Epic 1, 2 và 3
**When** chạy trong một môi trường **không có cấu hình AI**
**Then** toàn bộ vẫn xanh

🔵 **2026-08-13 — Story 4.1 tách khỏi Epic 4 và chạy ngay sau Epic 3.** Phần còn lại của
Epic 4 chạy sau Epic 6. ⇒ Khi Epic 4.2–4.12 tới lượt, AC này phải được **chạy lại** trên bộ
test của Epic 5 và Epic 6 nữa — ranh giới AD-13 canh mọi module, không chỉ ba epic đầu.
Đây là **mở rộng**, không phải thu hẹp: `epics.md` không sửa để khớp mã đã viết
(`project-context.md` §Story và spec).
```

**Lý do:** để nguyên văn *"Epic 1, 2 và 3"* thì sau khi Epic 5/6 xong, AC đọc như thể ranh
giới `ai/` chỉ cần canh ba epic đầu — một AC **xanh mà không canh đủ**.

---

### 4.6 `_bmad-output/implementation-artifacts/sprint-status.yaml`

**Vị trí:** đầu tệp (khối comment) + khối `development_status`

**Sửa 1 — thêm vào khối comment đầu tệp:**

```yaml
# 2026-08-13: THU TU THUC THI DOI — Epic 4 (AI) lui xuong sau Epic 6.
#   Thu tu moi: 1 -> 2 -> 3 -> 5 -> 6 -> 4 -> 7 -> 8 -> 9 -> 10
#   SO EPIC LA TEN, KHONG PHAI THU TU — khong doi so hieu, khong doi id story.
#   NGOAI LE: story 4-1 (module ai/ co lap + test cuong che AD-13) chay NGAY SAU epic-3,
#   khong cho phan con lai cua epic-4. Ly do o sprint-change-proposal-2026-08-13b-thu-tu-epic.md §3.4
```

**Sửa 2 — thêm một dòng comment ngay trên `4-1-…`:**

```yaml
  # Epic 4: AI mở & Smart RAG Injector
  # THU TU THUC THI: sau epic-6 (tru story 4-1 duoi day)
  epic-4: backlog
  # 4-1 CHAY NGAY SAU EPIC-3, khong cho epic-4 — xem comment dau tep
  4-1-module-ai-co-lap-va-test-cuong-che-ranh-gioi: backlog
```

**Lý do và giới hạn — ghi ra thay vì để người sau tự phát hiện:**

Tôi **cố ý không** đổi id story, không đổi số epic, và **không** di chuyển khối `epic-4`
xuống dưới `epic-6` trong tệp. Ba lý do:

1. Đóng băng số hiệu là quyết định của §3.2 — id là **danh tính**, đổi id là phá mọi tham
   chiếu chéo trong 30 tệp story đã viết.
2. `sprint-status.yaml` được đọc bởi các skill `bmad-create-story` và `bmad-sprint-status`.
   Di chuyển khối có thể đúng, nhưng tôi **chưa đo** hai skill đó xử lý thứ tự khối thế nào.
   Theo luật *"đo trước khi tin"*, tôi không đổi cấu trúc dựa trên phỏng đoán.
3. ⚠️ **Giới hạn thật:** vì thế, **thứ tự đọc của tệp YAML không phản ánh thứ tự thực thi.**
   Nguồn sự thật cho trình tự là `build-sequence.md` (cột *Thứ tự*) và `epics.md` §Epic List.
   Khối comment đầu tệp tồn tại để người đọc `sprint-status.yaml` một mình không hiểu nhầm.

---

### 4.7 `ARCHITECTURE-SPINE.md` §Deferred — mỏ neo SSE

**Vị trí:** dòng 898, cột *Điều kiện mở lại*

**CŨ:**

```markdown
| **Cách phân tích khung SSE** | … AD-22 đã cố định phần bất biến (Channel, không auto-reconnect, huỷ được) | Giai đoạn 2 — rà giấy phép trước khi thêm |
```

**MỚI:**

```markdown
| **Cách phân tích khung SSE** | … AD-22 đã cố định phần bất biến (Channel, không auto-reconnect, huỷ được) | 🔵 **Giai đoạn 2c** — rà giấy phép trước khi thêm. *(Sửa 2026-08-13: bản trước ghi "Giai đoạn 2". CAP-4 dời sang Giai đoạn 2c và 2c nay chạy SAU Giai đoạn 3b — xem `build-sequence.md` cột "Thứ tự". Mỏ neo cũ để lại sẽ bị đọc thành "phải rà giấy phép SSE trước khi làm Editor", sai cả hai vế. Cửa rà NFR15 **không đổi**, chỉ mở muộn hơn.)* |
```

**Lý do:** đây là **bảng §Deferred của spine**, một tệp khác với bảng mỏ neo trong
`build-sequence.md` (§4.2). Cả hai mang một hàng SSE riêng; sửa một mà bỏ một là dựng hai
nguồn sự thật nói ngược nhau về cùng một quyết định hoãn.

⚠️ **Ba mỏ neo *"Giai đoạn 2"* còn lại trong bảng này KHÔNG đổi** — `:894` (ngưỡng WAL +
nhịp flush, AD-12/AD-35) và `:897` (thư viện editor) đều thuộc Editor, ở Epic 2, đứng yên.

---

## 5. Bàn giao

### 5.1 Phân loại phạm vi

**Moderate** — tổ chức lại backlog, không phải replan nền tảng.

Căn cứ: không FR/NFR/AD nào đổi, không story nào thêm/bớt, không mã nào tồn tại để sửa.
Toàn bộ thay đổi nằm ở **bốn tệp quy hoạch** và là văn bản.

### 5.2 Ai làm gì

| Vai | Việc |
|---|---|
| **Ice** (chủ dự án) | Duyệt proposal. **Quyết định đã chốt trong đây: thứ tự mới + đóng băng số hiệu Giai đoạn + tách Story 4.1** |
| **John** (PM) | Áp 6 chỗ sửa ở §4.1–4.6 sau khi được duyệt |
| **Amelia** (Dev) | **Không có việc từ proposal này.** Story đang chạy (1.3, 1.20, 1.21, 1.22, 2.3, 2.4) không bị ảnh hưởng |
| **Winston** (Architect) | Chỉ để biết: `ARCHITECTURE-SPINE.md:898` đổi mỏ neo. Không AD nào mới |

### 5.3 Thứ tự áp

1. `build-sequence.md` §bảng Giai đoạn (§4.1) — bản chốt đi trước
2. `build-sequence.md` §mỏ neo hoãn (§4.2)
3. `epics.md` §Epic List bảng (§4.3)
4. `epics.md` §ghi chú Epic 5 (§4.4)
5. `epics.md` §Story 4.1 AC (§4.5)
6. `sprint-status.yaml` (§4.6)

Một commit riêng, `docs(epics):`, không trộn với story đang chạy — theo luật *"diff của một
story phải đọc được một mình"*.

### 5.4 Điều kiện thành công

- `build-sequence.md` và `epics.md` khai cùng một thứ tự thực thi, không lệch
- Không tham chiếu *"Giai đoạn N"* nào trong 6 tệp quy hoạch đổi nghĩa *(đã kiểm: 39/40 đứng
  yên, 1 chỗ dời có chủ đích ở §4.2)*
- Story 4.1 có vị trí thực thi tường minh ở cả ba nơi: `build-sequence.md`, `epics.md`,
  `sprint-status.yaml`
- Mệnh đề *"ba đường xử lý, chưa chọn"* không còn tồn tại ở dạng đã hết đúng

### 5.5 Việc KHÔNG thuộc proposal này

- **Lượt CI đỏ trên Windows** (`store_contract.rs::an_idle_pause_triggers_one_passive_checkpoint`,
  run `31717647486`) — không liên quan, và chưa có chủ. Cần một quyết định riêng của Ice.
- **Đo lại `deferred-work.md`** — proposal này không thêm món nợ nào, nên không chạm sổ nợ.

---

## Phụ lục — Change Navigation Checklist

| # | Mục | Trạng thái | Ghi chú |
|---|---|---|---|
| 1.1 | Story kích hoạt | **[N/A]** | Không phải lỗi từ một story — thay đổi chủ đích của chủ dự án |
| 1.2 | Phát biểu vấn đề | **[x]** | §1.2 · phân loại: *strategic pivot* |
| 1.3 | Bằng chứng | **[x]** | §1.3 · ba bằng chứng, đều trích từ tài liệu quy hoạch |
| 2.1 | Epic hiện tại còn hoàn tất được? | **[x]** | Epic 1 và 2 đang chạy, **không bị ảnh hưởng** |
| 2.2 | Thay đổi cấp epic | **[x]** | Chỉ thứ tự. Không thêm/bớt/định nghĩa lại epic nào |
| 2.3 | Rà mọi epic còn lại | **[x]** | §2.2 — bảng 8 dòng, kiểm bằng quét toàn văn |
| 2.4 | Có epic nào thành vô nghĩa? | **[x]** | Không. Không epic mới nào cần thêm |
| 2.5 | Thứ tự/ưu tiên có nên đổi? | **[x]** | **Đây chính là nội dung thay đổi** |
| 3.1 | Xung đột PRD | **[x]** | §2.4 — **không có**, nhờ quyết định đóng băng số hiệu |
| 3.2 | Xung đột Architecture | **[x]** | §2.5 — **đúng một dòng** (`:898`). Không AD mới |
| 3.3 | Xung đột UI/UX | **[x]** | §2.7 — không có |
| 3.4 | Artifact khác | **[x]** | §2.8 — CI, cổng, hook: không mã hoá thứ tự epic |
| 4.1 | Direct Adjustment | **[Viable]** | Công sức **Thấp**, rủi ro **Thấp** ⇒ **CHỌN** |
| 4.2 | Rollback | **[Not viable]** | Không có gì để hoàn tác |
| 4.3 | MVP Review | **[Not viable]** | Không cần — vấn đề là trình tự, không phải phạm vi |
| 4.4 | Chọn đường | **[x]** | Option 1, §3.1 |
| 5.1 | Tóm tắt vấn đề | **[x]** | §1 |
| 5.2 | Tác động epic + artifact | **[x]** | §2 |
| 5.3 | Đường đi kèm lý do | **[x]** | §3, kèm §3.3 *(cái giá phải trả)* |
| 5.4 | Tác động MVP + kế hoạch | **[x]** | §2.4 — **MVP không đổi**; kế hoạch ở §5.3 |
| 5.5 | Bàn giao | **[x]** | §5.2 |
| 6.1 | Rà checklist | **[x]** | Bảng này |
| 6.2 | Kiểm tính chính xác | **[x]** | Mọi trích dẫn kèm `tệp:dòng`, đã đối chiếu nguồn |
| 6.3 | Ice duyệt | **[x]** | ✅ **Duyệt trọn gói 2026-08-13**, cả 6 chỗ (§4.7 phát hiện và áp thêm lúc thi hành) |
| 6.4 | Cập nhật `sprint-status.yaml` | **[x]** | §4.6 · YAML parse lại sạch, 155 mục |
| 6.5 | Xác nhận bước kế | **[x]** | §5.3 · **chưa commit** — chờ Ice theo luật *"hỏi Ice trước khi commit"* |

---

## Nhật ký thi hành — 2026-08-13

| § | Tệp | Trạng thái |
|---|---|---|
| 4.1 | `build-sequence.md` §bảng Giai đoạn | ✅ đã áp |
| 4.2 | `build-sequence.md` §mỏ neo hoãn | ✅ đã áp |
| 4.3 | `epics.md` §Epic List bảng | ✅ đã áp |
| 4.4 | `epics.md` §ghi chú Epic 5 | ✅ đã áp |
| 4.5 | `epics.md` §Story 4.1 AC | ✅ đã áp |
| 4.6 | `sprint-status.yaml` | ✅ đã áp · YAML validate lại: **155 mục, parse sạch** |
| 4.7 | `ARCHITECTURE-SPINE.md` §Deferred | ✅ đã áp *(phát hiện lúc thi hành — xem ghi chú đầu §4)* |

**Diff:** 4 tệp · +76 −27. **Chưa commit.**
