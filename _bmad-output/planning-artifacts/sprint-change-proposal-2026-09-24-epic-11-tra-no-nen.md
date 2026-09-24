# Sprint Change Proposal — 2026-09-24 · Epic 11 trả nợ nền

**Người soạn:** John (PM), `bmad-correct-course` chế độ Batch · **Người duyệt:** Ice
**Baseline:** `e3aea77` (master, cây sạch)
**Nguồn:** commit `e570d62` — *"154 mục còn mở không có story nào nhận ngoài Ice"*
**Trạng thái:** Ice duyệt 2026-09-24; đã thi hành.

---

## 1. Vấn đề

Lượt rà sổ nợ 2026-09-23 (`e570d62`) giao lại chủ cụ thể cho 305 mục. Phần lớn rơi về `Chủ: Ice` vì không story nào nhận. Ice yêu cầu xếp số nợ này vào các epic sắp tới.

**Con số đo lại** bằng chính bộ phân tích của `check-debt-owner.mjs`, trên `e3aea77`: **210** mục mở/🟡 có `Chủ:` cụ thể cuối cùng là Ice và mang dòng `(rà sổ nợ)` (160 mở · 50 🟡). Con số 154 trong thông điệp commit không tái lập được với luật nhận diện của cổng. Toàn sổ có 449 mục mở/🟡 đứng tên Ice; 239 mục ngoài lượt rà không thuộc phạm vi đề xuất này.

**Phát hiện chặn:** bốn agent xếp độc lập 210 mục vào các story backlog của Epic 7–10, với luật "chỉ gán khi story đó chạm đúng mã hay hành vi". Chỉ **9** mục vừa. **201** mục còn lại nằm trong mã của Epic 1–6 và hạ tầng cổng, nơi Epic 7–10 (TM · Reviewer · Proofreader · Phát hành) không chạm. Gán gượng sẽ dựng lại đúng lớp "chủ mơ hồ" mà `e570d62` vừa dọn, và `check:debt-owner` vẫn xanh vì Epic 7–10 đang `backlog`.

## 2. Phân tích tác động

| Tài liệu | Tác động |
|---|---|
| `epics.md` | Thêm Epic 11 (§Epic List + khối story). Sửa hai chỗ lệch đã có mục nợ: `:1481` (Story 1.10b) và `:1590` (Story 1.12). |
| `build-sequence.md` | Thêm Epic 11 vào thứ tự thực thi. |
| `sprint-status.yaml` | Thêm khối `epic-11` và bảy khoá story, cộng một dòng ghi thứ tự. |
| `deferred-work.md` | Nối `→ … Chủ: <story>` vào 208 mục; đóng/🟡 hai mục lệch `epics.md`. |
| PRD · spine · UX | Không đổi. Epic 11 không mang FR, không đổi bất biến nào. |

**Thứ tự thực thi mới:** `1 → 2 → 3 → 5 → 6 → 4 → 11 → 7 → 8 → 9 → 10`. Epic 7 sẽ dựng trên matcher, cổng và e2e đã vá. Cái giá: tính năng TM đến muộn hơn bảy story.

⚠️ `sprint_plan.py status` đề xuất story theo thứ tự ĐỌC của tệp, nên khối `epic-11` nằm cuối tệp sẽ bị đề xuất sau Epic 10. Thứ tự đúng ghi bằng chữ ở đầu `sprint-status.yaml`, theo tiền lệ Epic 4.

## 3. Quyết định của Ice (2026-09-24)

| # | Quyết định | Ice chọn |
|---|---|---|
| 1 | 201 mục không vừa Epic 7–10 | **Epic 11 "Trả nợ nền"**, story theo nhóm |
| 2 | 9 mục vừa Epic 7–10 | **Giao cả 9**, kể cả 7 mục khớp yếu |
| 3 | Vị trí Epic 11 | **Ngay sau Epic 4, trước Epic 7** |
| 4 | Chế độ duyệt | Batch |

## 4. Đề xuất sửa chi tiết

### 4.1 Giao 9 mục cho story đã có

| Mục (dòng trên `e3aea77`) | Chủ mới | Khớp |
|---|---|---|
| 522 (NFD/NFC ở `core/matching`) | Story 7.6 | chắc |
| 511, 513 (`core/matching/mod.rs`) | Story 7.6 | yếu |
| 1755 (`isTmFilled` hằng `false`) | Story 7.4 | yếu (có thể 7.1 hoặc 7.3) |
| 762 (NFR1 đầu-cuối trên bản đóng gói) | Story 10.9 | chắc |
| 330, 4657, 5286, 5402 (đo NFR6/NFR2) | Story 10.9 | yếu |

Với mục khớp yếu, story nhận sẽ tự quyết đóng hay chuyển chủ khi soạn spec. AC của các story này không đổi.

### 4.2 Epic 11 — bảy story

| Story | Nhóm | Mục | Treo điều kiện |
|---|---|---|---|
| 11.1 | Cổng và công cụ kiểm (`scripts/check-*`, hook, lint) | 45 | 3 |
| 11.2 | Hạ tầng e2e và bộ chạy test | 23 | 4 |
| 11.3 | Tra cứu và dữ liệu từ điển | 32 (+1 mục §4.3 = 33) | 2 |
| 11.4 | Glossary | 32 | 2 |
| 11.5 | Editor, segment và tầng ghi | 25 | 3 |
| 11.6 | Nhập và Library | 28 | 0 |
| 11.7 | Nền giao diện dùng chung (lệnh, tiêu điểm, phím tắt, a11y) và AI | 14 | 2 |
| | **Cộng** | **199** | **16** |

Danh sách mục của mỗi story là `grep 'Chủ: Story 11.N' deferred-work.md`, không chép vào đây. 199 + 2 mục đóng ở §4.3 = 201.

Khối Epic 11 thêm vào `epics.md`:

> ## Epic 11: Trả nợ nền — đóng nợ đã hoãn của Epic 1–6 trước khi xây tiếp
>
> Lượt rà sổ nợ 2026-09-23 để lại 199 mục còn đúng trên mã, nằm trong phần nền Epic 1–6 đã dựng. Không epic tính năng nào còn lại chạm tới chúng. Epic này không thêm năng lực người dùng thấy được; nó trả nợ trước khi Epic 7 dựng TM lên cùng phần nền đó.
>
> **FRs covered:** không. **Nguồn:** `sprint-change-proposal-2026-09-24-epic-11-tra-no-nen.md`.
>
> **AC chung cho mọi story của Epic 11:**
>
> **Given** các mục `deferred-work.md` có `Chủ:` cuối cùng là story này
> **When** soạn spec
> **Then** Task 0 đọc lại từng mục trên mã HEAD, vì mục có thể đã tự đóng hoặc đổi dạng từ lượt rà 2026-09-23
>
> **Given** story chuyển sang `done`
> **When** `check:debt-owner` chạy
> **Then** mỗi mục kết thúc bằng đúng một trong ba: `→ ✅ ĐÃ ĐÓNG` kèm bằng chứng mã hoặc test · `→ KHÔNG LÀM <ngày> (Story 11.N) — <lý do>` · `→ … Chủ: <chủ cụ thể mới>` kèm lý do
> **And** Kiểm C của cổng đỏ nếu còn mục mở trỏ vào story đã `done`, nên không cần cổng mới
>
> **Given** một mục treo điều kiện ("mở lại khi …")
> **When** điều kiện vẫn chưa xảy ra
> **Then** không dựng mã phòng trước; chốt `KHÔNG LÀM` hoặc chuyển về `Chủ: Ice` với điều kiện ghi rõ
>
> **Given** một mục đòi đổi bất biến kiến trúc
> **When** phát hiện
> **Then** dừng và chuyển `Chủ: Winston`, không vá trong story

Mỗi story giữ khuôn chung: *As a chủ dự án, I want nợ nhóm <X> được đóng hoặc quyết dứt điểm, so that <epic kế tiếp> không dựng lên phần nền còn lỗ.* AC riêng của từng story do lượt `create-story` rút ra từ các mục nó nhận. Nhóm 45 mục của 11.1 có thể tách đôi lúc đó.

### 4.3 Hai chỗ lệch `epics.md`, PM sửa ngay trong lượt này

**Story 1.10b, `epics.md:1481`**

OLD: `🔴 **Quyết định phải chốt TRONG story:** lớp này vào **`dict-core.db`** … hay thành **tệp `.db` riêng**? …`

NEW: `🔵 **Đã chốt 2026-08-05 (AC5 của Story 1.10b):** lớp này vào **`dict-core.db`**, là nguồn nền thứ sáu; `dict-core.db` dựng lại và `[base].sha256` điền lại.`

Lý do: spec 1.10b ghi AC5 ĐẠT. Đóng mục nợ dòng 360.

**Story 1.12, `epics.md:1590`**

OLD: `**And** `dict/` dùng nó; `glossary/` và `tm/` sẽ dùng chính nó ở các epic sau, không cài lại`

NEW: `**And** `glossary/` và `tm/` dùng chính nó ở các epic sau, không cài lại; `core/dict/**` **không** gọi nó (AD-17, AD-44 ③), và `matching_boundary.rs` canh ranh giới này`

Lý do: AD-17 đã lật vế `dict/`. Mục nợ dòng 405 thành 🟡: thông điệp trong `src-tauri/tests/matching_boundary.rs:325` vẫn nói `epics.md:1510 ĐANG LỆCH`, giao cho Story 11.3.

## 5. Bàn giao

**Phạm vi:** Moderate (xếp lại backlog). PM thi hành ngay sau khi Ice duyệt: sửa bốn tệp ở §2, chạy `check:debt-owner`, commit riêng. Story 11.1 là story kế tiếp sau Epic 4, soạn bằng `create-story`.

**Tiêu chí thành công:**
- `check:debt-owner` xanh; 0 mục trong 210 còn `Chủ: Ice` là chủ cuối.
- `grep -c 'Chủ: Story 11\.'` theo từng story khớp bảng §4.2.
- Thứ tự `… 4 → 11 → 7 …` có mặt ở `epics.md` §Epic List, `build-sequence.md` và đầu `sprint-status.yaml`.
