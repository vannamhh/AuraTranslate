# Sprint Change Proposal — 2026-08-25 · Story 3.10b

**Người soạn:** `bmad-correct-course` (chế độ Batch) · **Người duyệt:** Ice
**Baseline:** `fd46007` (master, đã đẩy) · `grep -c "^### AD-"` = **48**

---

## 1. Vấn đề

Story 3.10 (*Xuất và nhập Glossary qua CSV/TSV*) đã `done`, nhưng nó chỉ dựng **nửa định dạng**. Nửa **chọn tệp** — đúng hai chỗ: lấy đường dẫn nguồn khi nhập, lấy đường dẫn đích khi xuất — bị tách ra và hoãn ngày 2026-08-24 vì nó đòi một `AD` mới, và `AD` giao Winston soạn chứ không phải dev tự viết.

Hệ quả đo được ở `fd46007`: `core::glossary::exchange` cùng `export_tier`/`import_into_tier` **chạy được, nghiệm thu được bằng 39 ca `cargo test`, và không một lối vào nào cho người dùng**. `grep -rn "export_tier\|import_into_tier" src-tauri/src/commands/` trả **rỗng** — không vỏ `#[tauri::command]` nào, đúng §Never của spec 3.10, có chủ ý.

**Loại vấn đề:** *technical limitation discovered during implementation* — không phải yêu cầu mới, không phải hiểu sai spec ban đầu. AC1 của Story 3.10 luôn đòi *"sinh ra file CSV hoặc TSV"*; thứ chưa có là **đường** để một tệp ra đời.

**Cửa chặn nay đã mở:** `AD-48` (*Hộp thoại chọn tệp gọi TỪ RUST; không một quyền plugin nào ra JavaScript*) đã viết, đã đẩy (`fd46007`), Ice chốt nhánh (a) ngày 2026-08-25.

---

## 2. Phân tích tác động

### 2.1 Tác động Epic

**Epic 3 vẫn hoàn tất được như quy hoạch — nó chỉ cần thêm một story.** Không epic nào bị vô hiệu, không epic nào cần đổi thứ tự, không epic tương lai nào bị chạm. Đây là **cùng hình dạng** với lượt thêm Story 3.4b ngày 2026-08-21: một nửa tách ra khỏi một story quá lớn, đi vào bằng `correct-course`, mục đầy đủ ở `epics.md`.

⚠️ **Khác 3.4b ở đúng một chỗ, và chỗ đó đáng ghi:** 3.4b tách vì **cửa đếm token** của `bmad-build` (5.000–5.800 token so với trần 1.600). 3.10b tách vì một **cửa chặn kiến trúc** — không phải vì spec dài. Hai lý do khác nhau, đừng đọc lượt này như một tiền lệ về kích thước.

**FR49 chưa đóng.** Nó đóng khi 3.10b xanh; Story 3.9 đóng vế *quản lý*, Story 3.10 đóng vế *định dạng*.

### 2.2 Tác động Story

| Story | Tác động |
|---|---|
| 3.10 (`done`) | **Không mở lại.** Mọi thứ nó dựng vẫn đúng và vẫn xanh; 3.10b chỉ **nối** vào hai hàm đã có |
| 3.10b (mới) | Story này |
| 4.1 và về sau | Không chạm |

### 2.3 Xung đột artifact

| Artifact | Xung đột | Hành động |
|---|---|---|
| **PRD** | **Không.** FR49/NFR9 mô tả đúng đích đến; năng lực chưa dựng ≠ lệch spec | Không sửa |
| **Architecture** | **Không còn.** `AD-48` là lượt đóng chính chỗ hở này | Không sửa |
| **UX** | **Không.** `mockups/glossary-manage.html:212-213` đã vẽ sẵn hai nút *Xuất CSV* / *Nhập CSV* | Không sửa |
| **`epics.md`** | 🔴 **CÓ** — không mục nào cho story này | §3.1 |
| **`sprint-status.yaml`** | 🔴 **CÓ** — không khoá nào | §3.2 |
| **`AGENTS.md`** | ⚠️ **CÓ, do `AD-48` gây ra** — hai dòng hết đúng | §3.3 |
| **`deferred-work.md`** | Mục 🟡 cần trỏ đích danh khoá story | §3.4 |
| **`check-deps.mjs` · `capabilities/main.json` · `Cargo.toml`** | Là **mã**, thuộc phạm vi story | Không sửa ở lượt này |

### 2.4 Tác động kỹ thuật

Đây là chỗ story mang rủi ro thật, và nó **không** nằm ở hộp thoại:

- **NFR6.** Chín crate mới vào cây với dư địa còn **3.104.634 byte**. `AD-48` đặt ngưỡng xét lại **1 MB** — nhưng đặt nó trên một con số **chưa ai đo**. Phép đo đó là một AC của story này, không phải một ghi chú.
- **Cổng `check:deps`.** `BANNED_CRATES` đi từ sáu tên xuống bốn. Một lượt gỡ hai hàng khỏi một cổng là chỗ dễ gỡ quá tay nhất trong cả story.
- **`config_invariants.rs`** không được sửa. Nếu nó đỏ, nghĩa là ai đó đã cấp một quyền plugin — đó là tín hiệu đúng, không phải một ca cần vá.

---

## 3. Đề xuất sửa cụ thể

### 3.1 `epics.md` — thêm §Story 3.10b, và bốn chỗ đếm

**(a) Chèn mục mới sau §Story 3.10, trước `## Epic 4`** (nội dung đầy đủ ở §4 dưới).

**(b) Dòng 699 — hàng FR49**, theo đúng khuôn FR50 đã dùng cho lượt tách 3.4/3.4b:

```
CŨ:  | FR49 | Epic 3 | Quản lý + xuất/nhập CSV/TSV |
MỚI: | FR49 | Epic 3 | Quản lý + xuất/nhập CSV/TSV — **quản lý: Story 3.9 · định dạng + đường ghi: Story 3.10 · hộp thoại chọn tệp: Story 3.10b**
     🔵 *(tách 2026-08-25 qua `correct-course`, cửa chặn kiến trúc AD-48)* |
```

**(c) Dòng 784 — tổng kiểm:** `Epic 3: **11**` → `Epic 3: **12**`.

**(d) Dòng 800 — hàng NFR9:** giữ nguyên. Nó nói *"Epic 3 (CSV)"*, vẫn đúng.

### 3.2 `sprint-status.yaml` — một khoá mới

Chèn ngay sau `3-10-…: done`, kèm chú thích theo khuôn 3.4b (**mục đầy đủ ở `epics.md`, không đúc nội dung vào tệp này**):

```yaml
  # 🔵 2026-08-25 — THÊM QUA `correct-course` (sprint-change-proposal-2026-08-25-story-3-10b.md).
  # Nửa CHỌN TỆP của FR49, tách khỏi 3.10 ngày 2026-08-24 vì một CỬA CHẶN KIẾN TRÚC (không
  # phải cửa đếm token như 3.4b): kho cấm `tauri-plugin-dialog`, và gỡ lệnh cấm là một `AD`.
  # CỬA CHẶN ĐÃ MỞ: `AD-48` viết 2026-08-25 (`fd46007`) — hộp thoại gọi TỪ RUST, `capabilities/
  # main.json` giữ đúng ba quyền, `tauri_plugin_fs::init()` không đăng ký.
  # Mục ĐẦY ĐỦ ở `epics.md` §Story 3.10b.
  # ⚠️ FR49 chỉ đóng khi story này xanh.
  3-10b-noi-hop-thoai-chon-tep-vao-xuat-nhap-glossary: backlog
```

### 3.3 `AGENTS.md` — hai dòng `AD-48` vừa làm hết đúng

```
Dòng 2:  "Verified 2026-08-24 against b290336"     → đã cũ hai commit
Dòng 19: "Bất biến kiến trúc (AD-1…AD-47, …)"      → nay là AD-48
```

🔴 **KHÔNG sửa tay.** Cả hai nằm trong khối `bmad:context`, và dòng 2 tự khai *"edits inside this block are replaced on refresh"* — một lượt sửa tay sẽ bị lượt refresh sau xoá, và tệ hơn: nó làm dòng `Verified` nói dối về một lượt đối chứng chưa từng chạy.

⇒ **Đóng bằng `bmad-project-context`** ở lượt refresh kế tiếp. Đây là một **action item có chủ**, không phải một ô tick của story này.

### 3.4 `deferred-work.md` — trỏ đích danh

Mục 🟡 ngày 2026-08-25 ghi *"Chủ phần còn hở: story thi hành nối tiếp 3.10"*. Sửa thành khoá thật: `3-10b-noi-hop-thoai-chon-tep-vao-xuat-nhap-glossary`. Bốn mục nhỏ khác của Story 3.10 cũng ghi *"story nối tiếp 3.10"* — cùng lượt trỏ đích danh, để không mục nào trỏ vào một cái tên không tồn tại.

---

## 4. Mục đầy đủ cho `epics.md`

```markdown
### Story 3.10b: Nối hộp thoại chọn tệp vào xuất/nhập Glossary

**Covers:** FR49 *(vế cuối)*

As a người dịch,
I want chọn tệp Glossary bằng hộp thoại của hệ điều hành,
So that bộ thuật ngữ tôi dựng cả tháng ra khỏi được máy tôi.

> 🔵 *(Thêm 2026-08-25 qua `correct-course`. Nửa **định dạng** của FR49 xong ở Story 3.10;
> nửa này tách ra ngày 2026-08-24 vì một **cửa chặn kiến trúc** — kho cấm `tauri-plugin-dialog`
> bằng `check-deps.mjs`, và gỡ một lệnh cấm là một `AD` mới, không một dòng cấu hình. Cửa mở
> bằng `AD-48` ngày 2026-08-25. **Khác lượt tách 3.4b**, vốn tách vì cửa đếm token.)*

**Acceptance Criteria:**

**Given** người dùng ở màn hình quản lý Glossary
**When** chọn xuất
**Then** hộp thoại **lưu tệp của hệ điều hành** mở ra
**And** tệp được ghi đúng nơi người dùng chọn, và đường dẫn đã ghi hiện ra

**Given** người dùng huỷ hộp thoại
**When** đóng nó mà không chọn gì
**Then** **không tệp nào được ghi, không lỗi nào hiện ra** — huỷ là một lựa chọn, không một thất bại

**Given** người dùng chọn nhập
**When** hộp thoại mở
**Then** lọc theo `.csv` và `.tsv`
**And** tệp được đọc rồi đi vào **đúng** đường phân tích mà Story 3.10 đã dựng — không bản sao thứ hai nào của bước đọc định dạng

**Given** một tệp nhập lớn bất thường
**When** đọc
**Then** có **trần kích thước**, và vượt trần thì từ chối tường minh — không đọc trọn một tệp vài GB vào bộ nhớ trên luồng invoke

**Given** một tệp không phải UTF-8
**When** đọc
**Then** từ chối tường minh, **không đoán bảng mã** *(dò bảng mã là Epic 6 — cùng ranh giới `import_file` đã đặt ở Story 1.15)*

**Given** `capabilities/main.json`
**When** story xong
**Then** nó vẫn mang **đúng ba** quyền, và `config_invariants.rs::main_capability_grants_the_minimum_and_no_plugin_permission` xanh **không cần sửa một chữ** *(AD-48 §Rule ②)*

**Given** cây phụ thuộc Rust
**When** kiểm
**Then** `tauri_plugin_fs::init()` **không xuất hiện** ở bất kỳ đâu trong `src-tauri/src/**` *(AD-48 §Rule ③)*
**And** `check:deps` Kiểm 1 còn **bốn** tên cấm, và chú thích của nó nói đúng thứ nó canh — **mã trong nhị phân**, không phải bề mặt IPC

**Given** bản dựng release
**When** đo
**Then** **payload sản phẩm được đo bằng byte** và ghi vào `deferred-work.md` kèm ngày và phiên bản toolchain
**And** nếu chín crate mới ăn quá **1 MB**, dừng và trình số cho Ice — `AD-48` đã đặt sẵn đường quay lui (`rfd` thẳng) và nó **không** chạm một chữ nào của ba mệnh đề `Rule`

**Given** mọi thao tác của story này
**When** thực hiện
**Then** làm được **bằng bàn phím**
```

---

## 5. Đường đi được chọn

**Option 1 — Direct Adjustment: thêm một story vào Epic 3 đang chạy.** ✅ Chọn.

- **Effort:** Thấp–Trung. Nửa nặng đã xong và đã xanh; story này nối dây cộng một lượt đo.
- **Risk:** Trung, và rủi ro **không** nằm ở hộp thoại — nó nằm ở ① lượt gỡ hai hàng khỏi một cổng, và ② dư địa NFR6 3,1 MB đứng trước chín crate chưa ai cân.

**Option 2 — Rollback:** ❌ Không khả thi và không có lý do. Story 3.10 không sai; nó dừng đúng chỗ nó nên dừng.

**Option 3 — MVP Review:** ❌ Không cần. MVP không đổi, FR49 không đổi, không gì bị hoãn.

---

## 6. Bàn giao

**Phân loại phạm vi: Moderate** — cần một mục quy hoạch mới cộng một khoá sprint, rồi mới tới mã.

| Việc | Giao cho |
|---|---|
| Sửa `epics.md` · `sprint-status.yaml` · `deferred-work.md` | Lượt `correct-course` này, ngay sau khi Ice duyệt |
| Thi hành Story 3.10b | `bmad-build` |
| Làm mới khối `bmad:context` của `AGENTS.md` | `bmad-project-context`, lượt refresh kế tiếp |

**Tiêu chí thành công:** FR49 đóng trọn; `config_invariants.rs` xanh **không sửa**; `check:deps` còn bốn tên và chú thích nói đúng thứ nó canh; và **một con số byte payload có thật** nằm trong `deferred-work.md` — không phải một lời khai *"chắc là nhỏ"*.
