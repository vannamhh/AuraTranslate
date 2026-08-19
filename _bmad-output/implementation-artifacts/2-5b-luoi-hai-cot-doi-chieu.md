---
baseline_commit: f990dd5
created: 2026-08-14
---

# Story 2.5b: Lưới hai cột đối chiếu

Status: done

**Covers:** UX-DR13 · UX-DR15 · UX-DR19 · FR16 · FR19 · FR21 · AD-1 · AD-34
**Supersedes:** 4/8 AC của Story 2.2 · hai AC của Story 1.14 *(«bốn slot panel» · «preset mặc định là lưới 2×2»)*
**Đóng nợ có chủ đích danh:** `deferred-work.md` :2863-2873 · :2875-2885 · :2896-2908
**Đọc lại, không được mặc nhiên giữ kết luận cũ:** `deferred-work.md` :2317-2371 · :2528-2584 · :2801-2816

> 🔴 **Story này là một lượt LẬT HÌNH DẠNG, không một lượt thêm tính năng.** Tầng Rust
> **không sửa một dòng**: `commands/segment.rs` + `config/segment.ts` + `editorPanelState.ts`
> *(2.103 dòng)* không biết gì về hình dạng và ở lại nguyên vẹn. Không bước di trú nào —
> `schema.rs:398` đã mang `status TEXT NOT NULL DEFAULT 'draft'` từ Story 2.5, và
> `is_paragraph_end` đã đi trên dây từ Story 2.1. Số di trú kế tiếp vẫn là **8**, và nó
> thuộc **Story 2.5c**.
>
> ⇒ Toàn bộ story sống ở **tầng vẽ**. Đó vừa là chỗ dễ, vừa là chỗ nguy: tầng vẽ là tầng
> **ít cổng canh nhất** trong kho này *(xem §Cổng nào sẽ nhìn story này — không một test
> mount component nào tồn tại)*.

---

## Story

As a người dịch,
I want thấy nguyên văn và bản dịch của cùng một câu trên cùng một hàng,
so that đối chiếu không còn là việc mắt tôi phải tự làm.

---

## Điều kiện khởi hành

🔴 **Ba story trước còn dang dở, và story này KHÔNG được tự chấm đạt hộ story nào.**

| Món treo | Có chặn 2.5b không? | Vì sao |
|---|---|---|
| **Story 2.3 `in-progress`** — lượt gõ ĐẦU TIÊN vào một câu chưa dịch: `<span>` rỗng rộng 0 px, không text node để neo caret ⇒ `execCommand('insertText')` trả `false` *(`deferred-work.md:2317-2371, 2528-2584`)* | **KHÔNG chặn — nó là TIỀN ĐỀ của story này** | Nguyên nhân gốc là *hình dạng*: một `<span>` rỗng trong dòng văn liên tục. Lưới thay chính hình dạng đó bằng **một ô có chiều cao thật** (AC3). ⚠️ Nhưng *"lưới sửa nguyên nhân"* là một **giả thuyết chưa đo** cho tới khi Task 1 chạy. Task 1 là chỗ nó được nghiệm hoặc bị bác. |
| **Story 2.3** — khuyết tật *"sập hố"* Ice báo 2026-08-14: xoá lui tới khi câu rỗng thì con trỏ thấp xuống và `Backspace` chết | **KHÔNG chặn — 2.5b PHẢI đóng nó** | Hai nguyên nhân chồng nhau, cả hai thuộc hình dạng cũ: span rỗng 0 px co lại nên caret lấy chiều cao từ một hộp rỗng; `contenteditable` đặt trên **đúng một** span nên `Backspace` ở offset 0 không có chỗ nào xoá lui vào. Quyết định #3 của Task 0 là chỗ phân xử. |
| **Story 2.4 `in-progress`** — bộ đo NFR2/NFR18 chưa tiêm được `bench.js` vào webview bản release | **KHÔNG chặn, nhưng nó GIỚI HẠN thứ 2.5b được phép khẳng định** | 2.5b **không** tự chấm NFR2 đạt. Nó **giao số** cho 2.4 *(§Task 8)*. ⚠️ Mọi số hiệu năng cũ của Epic 2 đo trên mô hình *"N `<span>` trong một dòng văn liên tục"* — chúng **mất hiệu lực theo cấu trúc**, không phải bị đóng. |
| **Story 2.5 `done`, còn Task 1.1 chưa chạy** — Ice gõ tay vào một câu chưa dịch trên app thật | **KHÔNG chặn** | Lượt đó đo hình dạng **cũ**. Sau 2.5b hình dạng đó không còn tồn tại ⇒ lượt kiểm tay phải chạy lại trên **lưới**, và đó chính là Task 1 dưới đây. Ghi rõ để không ai đánh dấu Task 1.1 của 2.5 là đạt bằng lượt của 2.5b — hai lượt đo hai bề mặt khác nhau. |

🔴 **LUẬT DỪNG của story này** *(chép khuôn Task 1.0 của Story 2.4)*: nếu sau Task 1, một câu
**chưa dịch** trong lưới **vẫn** không đặt được con trỏ bằng chuột trên WKWebView thật, thì
**DỪNG**, báo Ice, và 2.5b quay về `backlog`. Dựng nốt mười ba AC còn lại trên một bề mặt
chưa gõ được là sản xuất một sản phẩm trông xong mà không dùng được — đúng lớp lỗi mà lượt
correct-course 2026-08-14 vừa lật cả hình dạng để thoát ra.

⚠️ **Ba vòng chẩn đoán bị bác ⇒ DỪNG và báo Ice.** Bài học Story 2.3 → 2.4 → 2.5, đã lặp ba
lần: *"trúng tiền đề chưa phải trúng cơ chế"*.

---

## Acceptance Criteria

### Nhóm A — nguyên văn từ `epics.md:2261-2329`

**AC1 — ba slot panel**
**Given** Workspace **When** mở
**Then** **ba** slot panel `panel.grid`, `panel.lookup`, `panel.ai_translation` tồn tại trong **một** cửa sổ hệ điều hành duy nhất

**AC2 — hàng và năm cột**
**Given** lưới **When** hiển thị
**Then** mỗi câu là **một hàng**; trên hàng, từ trái sang: **vạch trạng thái · số câu · nguyên văn · bản dịch · nhãn trạng thái**

**AC3 — ô trống có chiều cao thật**
**Given** một câu **chưa dịch** **When** hiển thị
**Then** ô bản dịch có **chiều cao thật** và đường **đứt nét**
**And** bấm chuột vào nó **đặt được con trỏ**

**AC4 — sáu giá trị trạng thái**
**Given** sáu giá trị trạng thái **When** hiển thị
**Then** `confirmed` · `primary` · `draft` · `tm-rule` · **trống** · `ornament`
**And** mỗi giá trị khác *trống* có **đúng một** khối `.rule-<giá trị>` trong `<style scoped>` — cổng `check:commands` Kiểm I đối chiếu **hai chiều**

**AC5 — khoảng thở đoạn**
**Given** cờ `is_paragraph_end` đã lưu **When** render
**Then** dựng thành **khoảng thở** giữa các nhóm hàng, **không** phải một hàng rỗng

**AC6 — hai bố cục**
**Given** hai bố cục **Ⓑ-1** và **Ⓑ-2** **When** người dùng chọn
**Then** **cả hai dựng được**, Ⓑ-2 là mặc định
**And** lựa chọn giữ nguyên qua các phiên

**AC7 — hợp đồng vùng chọn theo CỘT**
**Given** hợp đồng vùng chọn **When** lưới đăng ký
**Then** đăng ký theo **CỘT** — cột nguyên văn vai `'source'`, cột bản dịch vai `'display'`
**And** `selectionContract.ts` **không sửa một dòng**
🔴 Đăng ký theo **cột**, KHÔNG theo từng ô: `selectionContract.ts:112` có một cổng đếm đọc **tĩnh**, và N bề mặt thay vì 1 làm nó đỏ.

**AC8 — Hán Việt**
**Given** Hán Việt **When** người dùng bật
**Then** hai chế độ FR19 *(chuyển đổi / song song)* đều chạy, **người dùng tự bật tắt**
**And** **không mặc định thông minh nào** — không buộc chế độ đi theo bố cục, không tự mở riêng cho hàng đang sửa

**AC9 — `NEVER_SACRIFICED`**
**Given** `NEVER_SACRIFICED` **When** đọc
**Then** **đúng một** phần tử `panel.grid`
**And** hai tập rời nhau hợp lại đúng **ba** panel *(mệnh đề "bốn panel" ở `workspaceLayout.ts:153` sửa cùng lượt)*

**AC10 — `⌘Enter`**
**Given** người dùng bấm `⌘Enter` **When** xảy ra
**Then** **xác nhận câu hiện tại và sang câu kế**

**AC11 — `Enter` trơn**
**Given** người dùng bấm `Enter` **trơn** **When** xảy ra
**Then** **không bao giờ** xác nhận
🔴 **Đây là một quyết định có bằng chứng, không một sở thích.** OmegaT dùng `Enter` để sang câu **nhưng** kèm tuỳ chọn *"Use TAB to Advance"* đặt ra **chính vì** `Enter` va chạm với bộ gõ IME. Người dùng của sản phẩm này gõ **tiếng Việt bằng bộ gõ**, nơi `Enter` là phím chốt dấu — giao `Enter` cho việc ký nghĩa là một lượt chốt Telex có thể **xác nhận nhầm một câu rồi nhảy đi**. ⚠️ Lớp lỗi này **không đường nghiệm thu nào của dự án bắt được** *(không bộ chạy test nào mô phỏng được một bộ gõ tiếng Việt thật)*; nó chỉ lộ ra ở tay người dùng. Kho đã biết điều đó: `EditorPanel.vue:841` có `if (event.isComposing) return` kèm chú thích *"một lượt commit composition của bộ gõ tiếng Việt phát keydown mang code vật lý; ăn nó là ăn mất chữ"*.

**AC12 — `⌥↓`**
**Given** người dùng bấm `⌥↓` **When** xảy ra
**Then** nhảy tới **câu chưa dịch kế tiếp**
**And** *"chưa dịch"* định nghĩa là `status = 'draft'` **và** `target_text` rỗng — `draft` nay đã tách khỏi *chưa dịch*

**AC13 — ba lệnh là command đăng ký**
**Given** ba lệnh trên **When** gọi
**Then** đều là **command đăng ký**, gán phím được — không gọi thẳng hàm *(một lời gọi thẳng dựng một đường thứ hai mà `check:commands` Kiểm A không nhìn thấy)*

**AC14 — ba khoá lỗi có đường ra màn hình**
**Given** ba khoá lỗi `err.segment.*` **When** một thao tác bị từ chối
**Then** có **đường ra màn hình** — không bị vứt ở tầng gọi
⚠️ Đây là mục ③ của ba quyết định chồng nhau: hôm nay `editorConfirmError` **không component nào đọc** và `main.ts` vứt `ConfirmResult`, nên một lượt từ chối **không đổi một pixel nào**.

> ⚠️ **Nợ ghi kèm story, không tự chấm đạt** *(nguyên văn `epics.md:2329`)*: chiều cao hàng khi
> bật Hán Việt **song song** ở cột hẹp của Ⓑ-2 *(ước ~330 px ⇒ một hàng có thể cao **6–7
> dòng**, ăn mất chính thứ Ⓑ-2 được chọn để có)* là **ước lượng hình học, CHƯA ĐO trên bản
> dựng thật**. Ai dựng thì **đo lại và ghi số**. → Task 7.

### Nhóm B — suy ra từ bất biến kiến trúc, mỗi mục trỏ nguồn

**B1 — Hai vai vùng chọn KHÔNG được đảo.** Cột nguyên văn `'source'`, cột bản dịch `'display'`.
> Nguồn: `sprint-change-proposal-2026-08-13.md` (Ice ký) · `check-commands.mjs` Kiểm F ③ · `EditorPanel.vue:83-96`.
> 🔴 Editor chứa **tiếng Việt đã dịch**. Tra ở đó cho 0 kết quả **rồi xoá mất** kết quả người dùng vừa tra từ cột nguyên văn. Đây là lỗi đã đi qua sạch 11 cổng một lần rồi *(commit `1c7658d`)* — đảo lại là mở lại đúng cửa vừa đóng.

**B2 — Hợp đồng flush AD-35 không đổi một mệnh đề.** idle **2 s** · trần cứng **5 s không reset bởi phím gõ** · xác nhận · rời segment · đóng Tác phẩm/thoát app. Đi qua đúng `store::Writer` nối tiếp (AD-11). Flush xong **chỉ sau khi đã ghi vào WAL**.
> Nguồn: `ARCHITECTURE-SPINE.md` AD-35 (:419-425) · `src/panels/editorFlush.ts:30`.
> ⚠️ *"Rời segment"* nay nghĩa là **rời hàng**. Nếu Quyết định #3 mở nhiều ô gõ được cùng lúc, hợp đồng này phải đúng cho **từng segment**, không phải một đồng hồ toàn cục cho cả lưới.

**B3 — `is_paragraph_end` chỉ được ĐỌC.** AD-37 cấm suy ra cấu trúc đoạn từ nội dung lúc render.
> Nguồn: AD-37 (:437-453) · `commands/segment.rs:136` · `config/segment.ts:73`.
> 🔴 Cờ đoạn **của bản dịch** là AD-46 và nó thuộc **Story 2.5d** (bước di trú 9). 2.5b **không** dựng nó, **không** đoán nó, **không** thêm cột nào.

**B4 — Không một quy tắc nghiệp vụ nào sang TypeScript (AD-1).** Trạng thái sinh ở Rust; lưới chỉ **đọc**.
> Nguồn: AD-1 (:75-79) · `editorSegments.ts:80-82`.
> ⇒ Cụ thể: đừng cài lại phép *"đã dịch hay chưa"* bằng cách so chuỗi ở nhiều chỗ. Một hàm thuần, một chỗ.

**B5 — `panel.grid` khai đúng một điểm vào focus, và `FOCUS_OWNERS` đối chiếu hai chiều.**
> Nguồn: AD-34 §2 (:406-417) · `PanelFrame.vue:126-133` · `commands/index.ts:64-72` · `check-commands.mjs` Kiểm E.
> ⚠️ `owner` phải viết **LITERAL** trong `.vue` — cổng đọc tĩnh.

**B6 — Màu chỉ từ token đã kiểm tương phản; `ornament` và `tm-rule` KHÔNG BAO GIỜ là màu chữ.**
> Nguồn: AD-34 §3 · `tokens.json:98-101` (`neverTextTokens`, cưỡng chế bằng `check:tokens`) · `DESIGN.md:208`.
> 🔴 Chỗ này va thẳng vào DESIGN.md frontmatter — xem **Quyết định #9**.

**B7 — `workspaceLayout.ts` giữ luật "erasable-only".** Không `import` giá trị, không `enum`/`namespace`/parameter property.
> Nguồn: `workspaceLayout.ts:1-29` · `check-layout.mjs` Kiểm A `import()` thẳng tệp này bằng Node trần.

**B8 — Mọi thao tác qua `CommandRegistry`; mỗi `@click` là ĐÚNG MỘT `dispatch('<id>')`.**
> Nguồn: AD-34 §1 · `check-commands.mjs` Kiểm A.
> ⚠️ Bấm chuột vào một hàng để đặt con trỏ là **thao tác con trỏ**, không phải một command — nó đi qua `mousedown`, không qua `@click`. Đừng nhét nó vào registry cho "đúng luật"; luật nói về **thao tác**, không về mọi sự kiện chuột.

**B9 — 2.5b KHÔNG tự chấm NFR2 đạt.** Đo số dựng-DOM trên hình dạng mới và **giao số** cho Story 2.4.
> Nguồn: `deferred-work.md:2113-2129, 2198-2207, 2484-2489, 2770-2782` · luật đo ở `project-context.md`.

**B10 — Không thêm một phụ thuộc npm nào mà chưa đi qua cửa NFR15.** Mở tệp giấy phép trong nguồn **đã tải** mà đọc; ghi vào bảng Stack của spine **TRƯỚC** khi thêm.
> Nguồn: `project-context.md` §NFR15 · `ARCHITECTURE-SPINE.md:763`. Ba lượt rà đầu của dự án đều là lượt *"đuổi theo"*.

**B11 — `data-segment-id` nay xuất hiện HAI lần cho một câu** *(ô nguyên văn + ô bản dịch)*. Hợp đồng neo phải được khai lại tường minh, và **bộ e2e phụ thuộc vào nó**.
> Nguồn: `e2e/specs/editor-typing-flush.e2e.mjs:87,107,112,151,200` · `editor-confirm-segment.e2e.mjs:121,128,158,171` · `editorGutter.ts:188`.
> 🔴 `document.querySelectorAll('[data-segment-id]')` hôm nay đếm **số câu**. Trong lưới nó đếm **2 × số câu**, và `$('[data-segment-id="X"]')` không còn duy nhất. Hai spec e2e sẽ **xanh giả hoặc đỏ oan** nếu không sửa cùng lượt — xem Quyết định #1 mệnh đề (d).

**B12 — Mọi chuỗi hiển thị mới vào `vi.json`, khoá phẳng có tiền tố miền, placeholder `{ten_tham_so}`.**
> Nguồn: NFR16 · AD-21 · `project-context.md` §Chuỗi và token.

---

## Task 0 — CHÍN quyết định phải chốt TRƯỚC dòng mã đầu tiên

> ✅ **ĐÃ KÝ TRỌN GÓI 2026-08-14 — Ice: *"duyệt 9 quyết định theo đề xuất"*.** Cả chín chốt ở
> đường ⭐. Bảng chữ ký và **ba chỗ chữ ký chưa phủ hết** ở §Dev Agent Record.
>
> 🔴 **Chín mục dưới đây Ở LẠI NGUYÊN VĂN, kể cả các đường bị loại.** Chúng là **bằng chứng**
> — *phương án bị loại đã bị loại bằng gì* — và một story sau đọc lại sẽ cần chúng. Đừng rút
> gọn còn mỗi kết luận.

🔴 **Ice là người chốt.** Mỗi quyết định nêu **cả hai (hoặc ba) đường kèm cái giá**, có một đường
⭐ đề xuất mặc định. Đừng tự chọn rồi đi tiếp, và cũng đừng loại một đường chỉ vì nó đắt.

---

### Quyết định #1 — Hình dạng DOM của lưới, và nó quyết luôn hợp đồng vùng chọn

**Sự kiện đo được.** AC2 đòi **hàng** *(mỗi câu một hàng, năm cột)*. AC7 đòi **cột** là **một
bề mặt đăng ký duy nhất** — nguyên văn: *"đăng ký theo CỘT, KHÔNG theo từng ô"*. Nhưng
`selectionContract.ts:168-190` duyệt bằng `el.contains(anchor)` ⇒ **một bề mặt phải là TỔ TIÊN
DOM của chữ trong nó**.

🔴 **Hai đòi hỏi đó xung khắc trong DOM thường:** trong `<table>` *(cũng là hình dạng của bản
dựng `.working/editor-grid-two-column.html`)*, một **cột không có phần tử tổ tiên nào**.
`<col>`/`<colgroup>` **không chứa** các `<td>` — chúng chỉ mang kiểu dáng. Cùng lý do, một
lưới CSS Grid với hàng là hộp thật *(`tr.row-primary { background }`)* cũng không có tổ tiên
cột. ⚠️ Đây **không** phải một chi tiết cài đặt: nó quyết định AC7 dựng được hay không.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | `<table>` như bản dựng; đăng ký **cả bảng** làm MỘT bề mặt vai `'source'` | 🔴 **Sai AC7 và sai B1**: cột bản dịch (tiếng Việt) trở thành nguồn tra ⇒ mở lại đúng lỗi commit `1c7658d` vừa đóng. **LOẠI**, ghi ra để không ai đi lại. |
| **(b)** ⭐ | **CSS Grid chủ-cột với `subgrid`**: một grid cha khai `grid-template-rows: repeat(N, auto)`; **năm** phần tử con là **năm cột**, mỗi cột `display: grid; grid-template-rows: subgrid; grid-row: 1 / -1`. Ô là con của cột. Hàng thẳng nhau vì cùng chia **một** tập track hàng của cha | Cột **là tổ tiên DOM thật** ⇒ AC7 dựng được nguyên văn, `selectionContract.ts` không sửa một dòng. ⚠️ **Hàng không còn là một hộp** ⇒ nền hàng đang sửa / hàng cắt bỏ phải tô **trên từng ô** của hàng đó, không trên một `<tr>`. ⚠️ `subgrid`: Safari 16+ *(macOS 12.4+)* · Chrome/Edge 117+ — **đủ** cho sàn của dự án, nhưng ba engine **bất đồng ở gap và auto-sizing** ⇒ 🔴 phải **đo trên CẢ HAI engine**, không đọc bảng tương thích *(cùng luật `selectionContract.ts:141`)*. |
| **(c)** | `<table>` giữ nguyên; đăng ký **từng ô** làm một bề mặt | 🔴 Đúng thứ AC7 cấm bằng chữ. `SELECTION_SURFACE_FLOOR = 7` là một **cổng đếm tĩnh**; N ô cho ra hàng nghìn bề mặt ⇒ cổng vô nghĩa, và mảng `surfaces` tuyến tính ở `selectionContract.ts:75` thành O(N) mỗi lượt chọn. **LOẠI.** |
| **(d)** | `<table>` + **sửa hợp đồng** cho nhận một `resolveRole(el)` | 🔴 Phá AC7 vế *"không sửa một dòng"*, và đặt một nhánh mới vào cửa mà **bảy** bề mặt khác đang đi qua. **LOẠI** trừ khi (b) bị đo bác. |

**Đề xuất mặc định: (b).** Lý do: nó là đường **duy nhất** thoả cả AC2 lẫn AC7 mà không sửa
hợp đồng vùng chọn và không phá cổng đếm.
⚠️ **Cái giá phải nói trước, không phát hiện sau:** mọi kiểu dáng **cấp hàng** *(nền
`surface-accent` cho hàng đang sửa · nền `surface-tm` cho hàng TM · gạch ngang hàng cắt bỏ ·
đường kẻ dưới hàng · khoảng thở đoạn AC5)* phải nhân ra **năm ô**, không một chỗ. Bản dựng
`.working/editor-grid-two-column.html:208-229` viết chúng trên `<tr>` — **không chép thẳng
được**.

**Kèm theo, mệnh đề (d) của B11 — neo `data-segment-id`:** với (b), một câu có **hai** ô mang
id. Chốt luôn cùng quyết định này: ô nào mang `data-segment-id` và ô nào mang một thuộc tính
thứ hai phân biệt vai *(ví dụ `data-col="src" | "tgt"`)*, và **hai spec e2e sửa cùng lượt**.

✅ **ĐÃ KÝ 2026-08-14 — đường (b).** ⚠️ Vế **neo `data-segment-id`** ký theo gợi ý
`data-col="src" | "tgt"` và đó là phần **ít bằng chứng nhất** của lượt ký — xác nhận lại sau
Task 1.

---

### Quyết định #2 — `.rule-draft` lấy màu từ đâu

**Sự kiện đo được.** `check-commands.mjs:2140-2152` (Kiểm I ③) đòi mỗi giá trị vạch có một
khối khai **đúng** `background-color: var(--color-<giá trị>)`. Thêm `'draft'` vào
`SEGMENT_RULE_VALUES` ⇒ cổng đòi `var(--color-draft)`. Nhưng bảng token có **16 token mỗi
theme** và `DESIGN.md:196` viết thẳng: *"Đừng thêm một token thứ 17 để cho khớp một con số
cũ"*. Bản dựng thăm dò dùng `background: var(--ornament); opacity: 0.45`
*(`.working/editor-grid-two-column.html:157`)* — thứ Kiểm I sẽ **đỏ**, và `opacity` trung gian
còn cần một miễn trừ **có tên** ở Kiểm D của `check-tokens`.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** ⭐ | Thêm token màu **`draft`** *(token thứ 17)*, khai vai **`stroke`**, đi qua vòng kiểm tương phản | Sửa **ba** chỗ khai số: `tokens.json` · bảng đóng băng trong `check-tokens.mjs` · `DESIGN.md`. Cộng một dòng 🔵 ở `DESIGN.md:196` nói vì sao con số 16 hết đúng. **Được phép** — `DESIGN.md:196` cấm thêm token *"cho khớp một con số cũ"*, không cấm thêm token có lý do. |
| **(b)** | Giữ 16 token; `.rule-draft` dùng `var(--color-ornament)` + `opacity` | 🔴 Kiểm I **đỏ** *(nó đòi `--color-draft`)* ⇒ phải **nới cổng** bằng một bảng alias. Nới một cổng để mã đi lọt là đúng thứ `project-context.md` §Miễn trừ cấm bằng chữ. Cộng một miễn trừ `opacity` có tên. |
| **(c)** | `draft` **không có vạch**, chỉ đọc ở **cột nhãn trạng thái** | 🔴 Sai AC4 nguyên văn *(«mỗi giá trị khác trống có đúng một khối `.rule-<giá trị>`»)* và sai UX-DR19 *(«cộng một vạch 2px đầu hàng»)*. |

**Đề xuất mặc định: (a).** Lý do: nó là đường duy nhất không nới một cổng và không phá AC4.
⚠️ Vai phải là **`stroke`**, không `text` — cùng luật đã đóng cho `ornament`/`tm-rule`.

✅ **ĐÃ KÝ 2026-08-14 — đường (a).** 🔴 **Giá trị màu hai theme CHƯA ký** — Ice ký *đường*, chưa
ký *số*. Đề xuất khởi điểm và lý do ở §Dev Agent Record ⟶ *"Ba chỗ chữ ký chưa phủ hết"* mục ①;
hỏi Ice **trước Task 4.4**. Mọi cặp mới phải khai vào `contrast.pairs` hoặc `contrast.excluded`
*(`check:tokens` Kiểm C)*.

---

### Quyết định #3 — `contenteditable` đặt ở đâu trong lưới

**Sự kiện đo được.** Quyết định #1 đường (c) của Story 2.3 *(Ice ký)*: vùng gõ là **MỘT câu tại
một thời điểm**, cài bằng `contenteditable` trên **đúng một** `<span>`
*(`EditorPanel.vue:930-947`)*. Khuyết tật *"sập hố"* đến **thẳng** từ đó: `Backspace` ở offset 0
của editing host duy nhất **không có chỗ nào để xoá lui vào**
*(`deferred-work.md:2896-2908`)*.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Giữ nguyên: `contenteditable` trên **đúng một ô** *(ô bản dịch của hàng có con trỏ)* | 🔴 *"Sập hố"* **không được sửa** — nó chỉ đổi chỗ từ span sang ô. Và Story 2.9 *(`Backspace` đầu ô = gộp với câu trên, UX-DR32)* mất tiền đề: nếu ô là editing host duy nhất thì `Backspace` ở offset 0 không sinh sự kiện nào để bắt. |
| **(b)** ⭐ | **Mọi ô cột bản dịch** mang `contenteditable="true"`; mỗi ô là một editing host **riêng** | *"Sập hố"* biến mất theo cấu trúc: một ô rỗng vẫn có `min-height: 1.95em` *(AC3)* nên caret có hộp để lấy chiều cao. `Backspace` ở offset 0 sinh một `beforeinput` `deleteContentBackward` **bắt được** ⇒ Story 2.9 có tiền đề. ⚠️ N editing host ⇒ hợp đồng flush B2 phải đúng **theo từng segment**; `isTypingZone` *(`keys.ts`)* phải đọc đúng ô đang gõ. ⚠️ Chi phí dựng DOM tăng — giao số cho Task 8. |
| **(c)** | Một `contenteditable` bọc **cả cột bản dịch** | 🔴 Một editing host chứa N ô ⇒ `Enter` và `Backspace` xoá **ranh giới ô**, tức phá cấu trúc dữ liệu bằng bàn phím. **LOẠI.** |

**Đề xuất mặc định: (b).** Lý do: nó là đường duy nhất **đóng** món nợ mà story này được giao
*(`deferred-work.md:2896-2908`)*, thay vì dời nó sang một hình dạng mới.
🔴 **Đây là một lượt LẬT Quyết định #1 của Story 2.3, và phải được ký lại tường minh** — không
được đi qua bằng im lặng. Tiền đề của quyết định cũ *(«một dòng văn liên tục»)* không còn tồn
tại.

✅ **ĐÃ KÝ 2026-08-14 — đường (b), và chữ ký đó LẬT Quyết định #1 của Story 2.3.** Mệnh đề
*"vùng gõ là MỘT câu tại một thời điểm"* hết hiệu lực từ đây; lý do là **tiền đề của nó không
còn tồn tại**, không phải vì nó sai lúc được ký. Ghi cả hai và ghi **thứ tự** ở
`editorSegments.ts` / `EditorPanel.vue` khi chạm tới *(Task 4.3)*.

---

### Quyết định #4 — số phận `editorGutter.ts` và `editorGutterLanes.test.ts`

**Sự kiện đo được.** `editorGutter.ts` *(273 dòng, 31 chỗ nhắc "làn")* giải bài toán **nhiều
câu trên cùng một dòng ⇒ vạch chồng nhau**. Trong lưới, **một câu một hàng** ⇒ bài toán biến
mất **theo cấu trúc**. Nhưng `assignGutterLanes` mang một **phép đo thật**: O(n²) =
**482,4 / 254,5 / 261,6 ms** trên 9.850 vạch ⇒ quét đường *(tô màu đồ thị khoảng)* =
**8,3 / 5,2 / 4,3 ms** *(2026-08-14, Node 22.22.2, macOS 15.6)*. Gỡ mã là gỡ luôn bằng chứng.
`deferred-work.md:2875-2885` giao đích danh: **KHÔNG xoá im lặng**.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** ⭐ | **Gỡ** `assignGutterLanes` + `measureGutterRules` + `editorGutterLanes.test.ts`; **chép nguyên phép đo và lý do gỡ** vào `deferred-work.md` *(nối tiếp mục :2875-2885, không xoá mục)* và vào §Completion Notes | Mất ~273 + 140 dòng. ⚠️ Phải kiểm lại `FILE_FLOOR`/`TS_FLOOR` của hai cổng sau khi bớt tệp — **sàn là cận dưới**, bớt tệp có thể làm nó vô nghĩa chứ không làm nó đỏ. |
| **(b)** | Giữ tệp, đánh dấu `@deprecated` | 🔴 Mã chết trong sản phẩm để phục vụ một bằng chứng. Bằng chứng thuộc về **sổ**, không thuộc về cây nguồn — đúng luật `deferred-work.md`. |
| **(c)** | Giữ `measureGutterRules`, gỡ riêng phần làn | Cần đo trước: với (b) của Quyết định #1, hàng có **hình học biết trước** ⇒ vạch lấy chiều cao từ chính track hàng, không cần `getClientRects()`. Chỉ chọn (c) nếu phép đo bác được mệnh đề đó. |

✅ **ĐÃ KÝ 2026-08-14 — đường (a).** ⚠️ Quyết định #1 đã chốt (b) ⇒ hàng có **hình học biết
trước**, nên mệnh đề *"vạch lấy chiều cao từ track hàng, không cần `getClientRects()`"* nay là
tiền đề của (a). Nếu Task 1.2 **bác** mệnh đề đó thì (c) sống lại — báo Ice, đừng tự chuyển.

---

### Quyết định #5 — đổi tên `PanelId` và `PresetId`: dữ liệu đã lưu của người dùng đi đâu

**Sự kiện đo được — đây là chỗ mất dữ liệu im lặng, và không cổng nào canh nó.**

1. **`PresetId` được LƯU XUỐNG ĐĨA.** `ScopeKind::LayoutPreset` *(`kinds.rs:213`, `GlobalOnly`)*
   và bố cục đang hiển thị nằm trong `ScopeKind::AppConfig` *(`WorkspaceMode.vue:56-73`)*.
   `presetById(id)` *(`workspaceLayout.ts:132`)* trả `undefined` cho một id lạ.
2. **Command id được LƯU XUỐNG ĐĨA** — Story 1.21 cho gán lại phím, và bảng `keybinding` khoá
   theo **command id**. Đổi `layout.preset_grid` → một tên mới làm **mồ côi** phím tắt người
   dùng đã gán, **im lặng**.
3. `check-commands.mjs:222` mang một phép đếm tĩnh: *"hai `layout.preset_*`"*.
4. `commands/index.ts:457` dựng id bằng nội suy: `` const id = `layout.preset_${preset}` ``.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** ⭐ | **Panel id đổi** *(`panel.source` + `panel.editor` → `panel.grid`)* — chúng **không** nằm trên đĩa. **Preset id GIỮ NGUYÊN hai cái tên cũ**, đổi **nghĩa**: `layout.preset_grid` = **Ⓑ-2** *(mặc định)*, `layout.preset_columns` = **Ⓑ-1**. Đổi `labelKey` và chuỗi `vi.json` cho đúng nghĩa mới | Phím tắt đã gán và preset đã lưu **sống nguyên**. Phép đếm *"hai preset"* không đổi. ⚠️ **Cái giá, ghi ra:** id `preset_columns` không còn tả đúng hình dạng nó dựng — phải có một chú thích 🔵 tại chỗ nói *"tên là lịch sử, nghĩa ở bảng ngay dưới"*. |
| **(b)** | Đổi cả preset id sang `layout.preset_b1`/`layout.preset_b2` | 🔴 Phím tắt người dùng mồ côi + `workspace_layout` đã lưu trỏ vào một preset không tồn tại ⇒ `presetById` trả `undefined`. **Phải** kèm một đường di trú đọc id cũ. Đắt hơn (a) và không mua thêm gì ngoài một cái tên đẹp. |
| **(c)** | Giữ nguyên cả tên lẫn nghĩa, thêm preset thứ ba | 🔴 Preset 4 cột **đã rút** *(`epics.md:539` — nó tách `Nguyên văn` khỏi `Bản dịch`, thứ không còn tồn tại)*. Giữ nó là giữ một bố cục dựng không được. |

**Đề xuất mặc định: (a).** Lý do: nó là đường duy nhất **không** làm mất một thứ người dùng đã
cấu hình.
⚠️ Kèm theo, sửa cùng lượt: `PANEL_IDS` · `PANEL_TITLE_KEYS` · `PANEL_COMPONENTS` ·
`SACRIFICE_ORDER` *(không đổi nội dung)* · `NEVER_SACRIFICED` *(AC9)* · `FOCUS_OWNERS`
*(`commands/index.ts:64-72`)* · `PANEL_SUFFIXES` *(`:375`, dùng cho bốn `layout.toggle_*` ⇒ nay
**ba**)* · ba chuỗi id **cứng** trong `check-layout.mjs:255-272`.

✅ **ĐÃ KÝ 2026-08-14 — đường (a).** 🔴 Hệ quả bắt buộc: `layout.preset_grid` **là Ⓑ-2** và
`layout.preset_columns` **là Ⓑ-1**. Hai cái tên nay là **lịch sử**, không phải mô tả — mỗi chỗ
khai phải mang một dòng 🔵 nói đúng câu đó, nếu không story sau sẽ đọc `preset_columns` thành
*"bốn cột"* và dựng lại một bố cục đã rút.

---

### Quyết định #6 — thư viện editor *(nợ chuyển chủ từ Story 2.4, Ice ký 2026-08-14)*

**Sự kiện đo được.** Hàng Deferred *"thư viện editor cho panel Editor"*
*(`ARCHITECTURE-SPINE.md:920`)* đổi chủ sang story này **vì bài toán đã đổi**: `contenteditable`
trên **một ô mỗi hàng** khác hẳn trên **một dòng văn liên tục**. `deferred-work.md:2896-2908`
viết thẳng: *"Kết luận cũ của Story 2.4 KHÔNG được mặc nhiên giữ"*.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** ⭐ | **Không thư viện.** `contenteditable` trần trên từng ô, `beforeinput` là cửa duy nhất *(khuôn `EditorPanel.vue:764-828` đã chạy)*. Đóng hàng Deferred kèm **lý do đo được** | Giữ 0 phụ thuộc mới ⇒ không phải mở cửa NFR15. ⚠️ Phải **đo** rồi mới ký: ô rỗng đặt được caret, `Backspace` đầu ô sinh `beforeinput`, IME không mất chữ. Ba phép đo đó là điều kiện của chữ ký. |
| **(b)** | Nhận một thư viện editor | 🔴 Đi trọn cửa NFR15: **mở tệp giấy phép trong nguồn đã tải mà đọc**, ghi vào bảng Stack **TRƯỚC** khi `npm i`, chỉ giấy phép tương thích GPLv3 chiều đi vào. Cộng AD-31: hợp đồng trạng thái không được lan ra ngoài module. |

**Đề xuất mặc định: (a)** — **nhưng chỉ sau Task 1**. Ký trước khi đo là đúng thứ dự án cấm.

✅ **ĐÃ KÝ 2026-08-14 — đường (a).** 🔴 **Ice ký TRƯỚC khi có số**, ngược với đề nghị ngay trên.
Ghi ra thay vì để nó trôi, và ghi luôn hệ quả: chữ ký này **đóng hàng Deferred
`ARCHITECTURE-SPINE.md:920` theo hướng "không thư viện"**, nhưng nó **không miễn cho Task 1**.
Nếu ba phép đo của Task 1.2 trượt thì **LUẬT DỪNG của §Điều kiện khởi hành thắng chữ ký này** —
dừng và báo Ice, **đừng** tự đi tìm một thư viện để cứu lượt dựng.

---

### Quyết định #7 — ảo hoá hàng: dựng ngay, hay đo rồi ghi nợ

**Sự kiện đo được.** `ARCHITECTURE-SPINE.md:922` để *"Chiến lược ảo hoá danh sách dài"* ở
**Giai đoạn 3**. Số đã đo trên hình dạng **cũ**: dựng 9.850 `<span>` vượt trần 50 ms/frame —
**6× trên Blink (300,1 ms)**, **26× trên WebKit (1.308,0 ms)** *(`deferred-work.md:2113-2129`)*.
Lưới **tăng** số node mỗi câu *(1 `<span>` → 5 ô)*, nhưng **bỏ** lượt dựng lại toàn danh sách
mỗi `selectionchange` *(`:2198-2207`)* nếu Quyết định #3 chọn (b) *(mỗi ô là host riêng, không
cần `:data-caret` trên mọi câu)*.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** ⭐ | **Không ảo hoá ở story này.** Dựng đủ hàng, **đo** trên cả hai engine, giao số cho Story 2.4, ghi nợ có chủ nếu vượt | Story giữ đúng phạm vi. ⚠️ Nếu số vượt trần thì **báo, đừng tối ưu mù** — đúng khuôn AC14 của Story 2.2 đã đặt. |
| **(b)** | Ảo hoá ngay trong 2.5b | 🔴 Ảo hoá + `contenteditable` + `subgrid` cùng lúc là ba biến chưa đo trong một lượt. Và nó lấn phạm vi của một quyết định spine đang để ở Giai đoạn 3. |

✅ **ĐÃ KÝ 2026-08-14 — đường (a).** ⇒ Nếu số của Task 8 vượt trần 50 ms thì đó là **một kết quả
được báo cáo**, không phải một lượt story trượt: ghi vào `deferred-work.md` với chủ **Story 2.4**
và **đừng tối ưu mù** — đúng khuôn AC14 của Story 2.2 đã đặt.

---

### Quyết định #8 — bề mặt báo lỗi cho `err.segment.*` (AC14)

**Sự kiện đo được — hai nguồn nói ngược nhau, phải phân xử.**
- **AC14 của story này** đòi ba khoá lỗi *"có đường ra màn hình"*.
- **`deferred-work.md:2801-2816`** ghi món này với **Chủ: Ice**, và điều kiện là *"chốt hợp đồng
  UX-DR30 — bề mặt báo lỗi của Editor — TRƯỚC khi story nào cài nó"*. Cùng cảnh ngộ:
  `'still-dirty'` *(Quyết định #8 của Story 2.5)* cũng chỉ `console.error`.
- Hôm nay: `editorConfirmError` export mà **không component nào đọc**; `main.ts:229` viết
  `void confirmCurrentSegment()` — `ConfirmResult` **bị vứt**.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** ⭐ | Bề mặt là **cột nhãn trạng thái của chính hàng** *(cột 5)* + một dòng ở thanh trạng thái. Đóng **cả hai** món cùng lượt *(ba khoá `err.segment.*` và `'still-dirty'`)* | Lỗi hiện **đúng chỗ người dùng vừa bấm**, không một hộp thoại nào *(UX-DR16: không lớp nổi)*. ⚠️ Cần **hai khoá `vi.json` mới** cho nhãn *"từ chối"* và cần Ice ký hợp đồng UX-DR30 **ở mức tối thiểu này**, không mở rộng. |
| **(b)** | Chỉ thanh trạng thái | Rẻ hơn, nhưng thanh trạng thái ở xa chỗ bấm; với một lưới dài, người dùng không nhìn xuống đó. |
| **(c)** | Hoãn AC14 sang một story sau | 🔴 Sai AC14 nguyên văn, và đó là mục ③ trong **ba** quyết định chồng nhau đã sinh ra lượt lật này. Hoãn nó là để nguyên một trong ba nguyên nhân gốc. |

✅ **ĐÃ KÝ 2026-08-14 — đường (a), và chữ ký này LÀ hợp đồng UX-DR30 ở phạm vi tối thiểu.**
⇒ `deferred-work.md:2801-2816` mất điều kiện chặn *("chốt UX-DR30 trước khi story nào cài")*:
điều kiện đó **đã đạt**, và 2.5b là story cài. 🔴 Phạm vi là **tối thiểu** — cột nhãn trạng thái
cộng một dòng ở thanh trạng thái. Đừng nhân lượt ký này thành một hệ thống thông báo.

---

### Quyết định #9 — màu chữ của cột số câu và cột nhãn trạng thái

**Sự kiện đo được — `DESIGN.md` tự mâu thuẫn, và một cổng đứng về một phía.**
- `DESIGN.md` frontmatter `:137-148` khai: `grid-num-col: { color: ornament }` ·
  `grid-state-col: { color: ornament }`.
- `DESIGN.md:208` khai ngược: *"`ornament` và `tm-rule` là **màu của nét**, không bao giờ là màu
  của chữ. Mọi chữ, kể cả nhãn 10px, tối thiểu phải là `on-surface-variant`."*
- `tokens.json:98-101` `neverTextTokens` **cưỡng chế bằng `check:tokens`**: `ornament` = 2,44
  (sáng) / 2,64 (tối) trên `surface` — **trượt AA**.
- ⚠️ Miễn trừ duy nhất đang khai cho `ornament` là *"ký tự ranh giới câu `⏐`"* — thuộc
  **UX-DR20**, thứ đã **RÚT** 2026-08-14. ⇒ Miễn trừ đó nay là một **miễn trừ chết**, và
  `eslint.config.js` đặt `reportUnusedDisableDirectives: 'error'` đúng vì lớp nợ này.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** ⭐ | Dùng **`on-surface-variant`** cho cả hai cột; sửa `DESIGN.md` frontmatter tại chỗ kèm 🔵 và ngày; **gỡ** miễn trừ `⏐` đã chết | Qua `check:tokens` không cần miễn trừ nào. Số câu và nhãn trạng thái **đọc được** — chúng là chữ thật, không phải nét. Đóng thêm một món nợ tài liệu. |
| **(b)** | Giữ `ornament`, xin **hai miễn trừ có tên** | 🔴 Hai nhãn chữ trượt AA trên một bề mặt người dùng nhìn hàng giờ. `project-context.md` §Miễn trừ: *"Sửa KIỂU cho nó nói thật; đừng nhét một miễn trừ để cổng hết đỏ."* |

✅ **ĐÃ KÝ 2026-08-14 — đường (a).** ⇒ Bản dựng `.working/…:161-205` dùng `ornament` cho hai cột
đó; **không chép**. Và **gỡ** miễn trừ `ornament` cho ký tự `⏐` — UX-DR20 đã rút nên nó là một
miễn trừ chết, đúng lớp nợ mà `reportUnusedDisableDirectives: 'error'` tồn tại để chống.
⚠️ Ghi kèm: `td.state.is-confirmed/is-primary/is-tm` của bản dựng dùng `confirmed`/`primary`/
`tm-text` làm màu chữ — cả ba **đã có vai `text`** trong `tokens.json:56-63`, nên ba cặp đó chỉ
cần khai vào `contrast.pairs` nếu nền mới.

---

## Tasks / Subtasks

- [x] **Task 0 — CHÍN quyết định (AC toàn story)** ✅ **XONG 2026-08-14**
  - [x] 0.1 Trình bày chín quyết định trên cho Ice, **kèm số đo**, chờ chữ ký.
  - [x] 0.2 🔴 Ghi lại chữ ký vào §Dev Agent Record **nguyên văn**. → Bảng chín chữ ký đã điền; Quyết định #6 ghi rõ *"Ice ký TRƯỚC khi có số của Task 1"*.
  - [x] 0.3 Không viết một dòng mã sản phẩm nào trước khi 0.2 xong.
  - [x] 0.4 🔴 **Ba chỗ chữ ký chưa phủ hết** *(§Dev Agent Record)* — quay lại hỏi Ice **đúng lúc**, đừng tự lấp: ① giá trị màu token `draft` *(trước Task 4.4)*; ② hình dạng neo `data-segment-id` *(xác nhận lại sau Task 1)*; ③ nếu Task 1.2 trượt thì **LUẬT DỪNG thắng chữ ký #6**.
        → ✅ **CẢ BA ĐÓNG 2026-08-14, sau Task 1.** ① Ice ký **mượn đúng số của `ornament`** *(`#a9a196` sáng / `#6a6459` tối)* — token thứ 17 là một **cái tên mới cho một màu đã kiểm**, không một màu mới chưa ai đo ⇒ **0 cặp mới** cho `contrast.pairs`. ② Ice ký **giữ `data-col="src"|"tgt"`**, và bàn đo WKWebView đã chạy trên chính hình dạng đó. ③ **KHÔNG kích hoạt** — ① và ② của Task 1.2 đều ĐẠT; chữ ký #6 đứng, với phạm vi ghi rõ ở §Debug Log.

- [x] **Task 1 — 🔴 CỬA CHẶN: dựng một mũi thăm dò lưới và GÕ TAY vào nó (AC3, Quyết định #1/#3/#6)**
  - [x] 1.1 Dựng một bàn đo HTML độc lập *(khuôn `2-5-ban-do-hai-vach.html`)*: lưới `subgrid` năm cột, ô bản dịch `contenteditable`, một hàng **chưa dịch** với ô rỗng `min-height: 1.95em` + `border-bottom: 1px dashed`. → `2-5b-ban-do-luoi.html` + `2-5b-ban-do/chup.mjs`.
  - [x] 1.2 Đo trên **WKWebView thật** *(không happy-dom, không Blink một mình)*: ① bấm chuột vào ô rỗng — caret có xuất hiện không; ② gõ một ký tự — `execCommand`/`beforeinput` có ăn không; ③ `Backspace` ở offset 0 — có sinh `beforeinput` `deleteContentBackward` không; ④ chiều cao caret trong ô rỗng có bằng dòng không *("sập hố")*; ⑤ `subgrid` có giữ hàng thẳng khi hai ô lệch chiều cao không.
        → `2-5b-ban-do/luoi-wkwebview.e2e.mjs`, WebKit **605.1.15**. **①②④⑤ ĐẠT · ③ KHÔNG** *(bất đồng engine)*. Bảng đầy đủ ở §Debug Log Ⓑ.
  - [x] 1.3 Đo cùng năm mệnh đề trên **Chromium** *(engine của WebView2)*. 🔴 Không suy từ bảng tương thích — cùng luật `selectionContract.ts:141`. → Playwright 1.62.1, **cả năm ĐẠT** *(gồm ③, thứ WebKit trượt)*.
  - [x] 1.4 🔴 **CHỦ: ICE** — mở app/bàn đo trên máy thật, gõ tiếng Việt **bằng bộ gõ**, xác nhận chữ hạ cánh và dấu không rơi.
        → ✅ **ICE XÁC NHẬN ĐẠT 2026-08-15.** Đây là vế mà **không đường nghiệm thu nào của dự án mô phỏng được** *(§Bất biến, hàng "`Enter` trơn": không bộ chạy test nào dựng lại được một bộ gõ tiếng Việt thật)*, nên chữ ký của Ice **là** đường nghiệm thu duy nhất của nó — không một cổng nào, không một ca e2e nào thay được.
  - [x] 1.5 🔴 **Nếu ① hoặc ② trượt: DỪNG, báo Ice, 2.5b về `backlog`.** Ba vòng chẩn đoán bị bác cũng ⇒ DỪNG. → **KHÔNG kích hoạt.** Ba vòng chẩn đoán đã chạy và vòng ba **CHỐT ĐƯỢC nguyên nhân** *(cổng 4445 bị chiếm)*, không phải bị bác.
  - [x] 1.6 Ghi kết quả + ngày + phiên bản engine vào §Debug Log References. Số này là căn cứ ký Quyết định #6. → §Debug Log Ⓐ/Ⓑ/Ⓒ + `2-5b-ban-do/README.md`.

- [x] **Task 2 — Tầng bố cục: bốn panel → ba (AC1, AC9, Quyết định #5)**
  - [x] 2.1 `workspaceLayout.ts`: `PanelId` còn ba giá trị; `PANEL_IDS`/`PANEL_TITLE_KEYS`/`PANEL_COMPONENTS` theo.
  - [x] 2.2 Hai preset mới thay `GRID_2X2`/`FOUR_COLUMNS`: **Ⓑ-2** *(lưới trái toàn chiều cao; Tra cứu trên, Đề xuất AI dưới, bên phải)* và **Ⓑ-1** *(lưới cả bề ngang ở trên; Tra cứu và Đề xuất AI hàng dưới)*. `DEFAULT_PRESET_ID` = preset của **Ⓑ-2** (AC6).
  - [x] 2.3 `NEVER_SACRIFICED = ['panel.grid']`; sửa chú thích `:154` *"đúng bốn panel"* → **ba** (AC9). `SACRIFICE_ORDER` **không đổi nội dung**.
  - [x] 2.4 🔴 Giữ luật erasable-only (B7) — không thêm một dòng `import` nào vào tệp này.
  - [x] 2.5 `check-layout.mjs:255-272`: sửa ba chuỗi id **cứng** của Kiểm A mệnh đề 3. ⚠️ Mệnh đề 2 duyệt `1 << PANEL_IDS.length` nên tự co từ 16 xuống 8 tập con — **không** sửa.
  - [x] 2.6 `commands/index.ts`: `FOCUS_OWNERS` *(:64-72)* còn sáu mục *(ba chế độ + ba panel)*, sửa chú thích *"BẢY mục… bốn panel"*; `PANEL_SUFFIXES` *(:375)* còn ba ⇒ **ba** `layout.toggle_*`, không bốn. ⚠️ `check-commands.mjs:222` mang một phép **đếm tĩnh** chép tay *("hai `layout.preset_*` · **bốn** `layout.toggle_*` · …")* — sửa cùng lượt, nếu không cổng đỏ ở một chỗ không ai ngờ.
  - [x] 2.9 🔴 **Đừng dựng lại đường lưu bố cục.** AC6 vế *"giữ nguyên qua các phiên"* **đã có sẵn**: `WorkspaceMode.vue::onPersist` ghi bố cục đang hiển thị qua `putConfig(SCOPE_APP_CONFIG, KEY_LAYOUT, json)`, và `bootstrap.ts::workspace_layout` đọc lại lúc khởi động. Với Quyết định #5 (a), **không một dòng nào của đường này phải sửa**. Chỉ kiểm bằng một lượt chạy tay: đổi sang Ⓑ-1, đóng app, mở lại.
        ⚠️ **Không** nhét preset đang chọn vào `layout_presets` dưới một khoá `__current` — `WorkspaceMode.vue:66-68` và `kinds.rs:206-213` đã cấm bằng chữ *(màn hình Story 1.21 sẽ hiện `__current` ra như một preset người dùng tự tạo)*.
  - [x] 2.7 `vi.json`: khoá `panel.grid.title` + `panel.grid.status`; **giữ** `panel.source.*` cho các chuỗi Hán Việt còn dùng, **gỡ** khoá nào không còn chỗ đọc. ⚠️ `check:i18n` canh cả hai chiều.
  - [x] 2.8 `WorkspaceDock.vue`: bảng `components` đăng ký component nội dung mới.

- [x] **Task 3 — Component lưới (AC2, AC3, AC5, Quyết định #1)**
  - [x] 3.1 Tệp mới `src/panels/GridPanel.vue`, `<PanelFrame owner="panel.grid" status-key="panel.grid.status">` — `owner` viết **LITERAL** (B5).
  - [x] 3.2 Dựng năm cột *(Quyết định #1 (b) — CSS Grid **chủ-cột với `subgrid`**)*: vạch trạng thái · số câu · nguyên văn · bản dịch · nhãn trạng thái (AC2). Grid cha khai `grid-template-rows: repeat(N, auto)`; **năm** con là **năm cột**, mỗi cột `display: grid; grid-template-rows: subgrid; grid-row: 1 / -1`. 🔴 Hàng **không** là một hộp ⇒ nền hàng đang sửa *(`surface-accent`)*, nền hàng TM *(`surface-tm`)* và đường kẻ dưới hàng phải tô **trên từng ô** của hàng đó.
  - [x] 3.3 Ô bản dịch rỗng: `min-height` bằng một dòng của token `editor` *(1.95)* + `border-bottom: 1px dashed var(--color-outline)` (AC3). 🔴 Vùng bấm là **cả ô**, không một `<span>` rỗng.
  - [x] 3.4 `is_paragraph_end` ⇒ **khoảng thở** giữa nhóm hàng, **không** một hàng rỗng (AC5). Chỉ **đọc** cờ (B3). ⚠️ Với Quyết định #1 (b), khoảng thở phải nhân ra năm ô của hàng cuối đoạn.
  - [x] 3.5 Nguồn dữ liệu một hàng = **một** `ChapterSegment` *(`source_text` + `target_text` cùng hàng)*. 🔴 Không join theo vị trí, chỉ theo `segment.id` (AD-3).
  - [x] 3.6 Neo `data-segment-id` theo hình dạng đã chốt ở Quyết định #1 (B11).
  - [x] 3.7 Kiểu chữ: cột nguyên văn `source-cjk` *(16,5px/2.05)* hoặc `source-latin` theo `source_lang`; cột bản dịch token `editor` *(15px/1.95)* — bốn AC của Story 2.2 còn sống nguyên gồm mệnh đề này.
  - [x] 3.8 🔴 **Xoá bề mặt cũ:** `EditorPanel.vue` và `SourcePanel.vue` không còn là panel của Workspace. Chuyển phần còn dùng được sang `GridPanel.vue`, đừng chép hai bản.

- [x] **Task 4 — Trạng thái: năm giá trị → sáu (AC4, Quyết định #2)**
  - [x] 4.1 `editorSegments.ts`: `SEGMENT_RULE_VALUES` thêm `'draft'` ⇒ **sáu**.
  - [x] 4.2 `resolveSegmentRule`: thêm nhánh `draft` **đúng chỗ trong thứ tự ưu tiên**. Thứ tự đề nghị: `ornament` › `primary` › `confirmed` › `tm-rule` › **`draft`** › `none`. 🔴 `draft` ⇔ `targetText !== ''` *(và không thuộc bốn nhánh trên)*; `targetText === ''` ⇒ `none`.
  - [x] 4.3 🔴 **SỬA TẠI CHỖ ba khối doc-comment đã hết đúng** trong `editorSegments.ts`, kèm 🔵 và ngày: `:2-37` *("VÌ SAO CẢ NĂM NHÁNH…")* · `:41-49` *("ĐÚNG NĂM GIÁ TRỊ…")* · `:107-139` *("KHE HỞ… Ice ký Quyết định #3 đường (a): không vạch")*.
        ⚠️ Khối `:107-139` chở một mệnh đề **Ice đã ký ngày 2026-08-14** *(«đã dịch, chưa xác nhận ⇒ không vạch»)* và UX-DR19 bản viết lại **cùng ngày** lật nó. Ghi cả hai và ghi **thứ tự** — đừng xoá dấu vết quyết định cũ.
  - [x] 4.4 Token màu **`draft`** *(thứ 17, vai `stroke`)* + khối CSS `.rule-draft { background-color: var(--color-draft) }`. Sửa **ba** chỗ khai số: `tokens.json:16` *("16 token mỗi theme")* · bảng đóng băng trong `check-tokens.mjs` · `DESIGN.md:189-196` *(kèm 🔵 nói vì sao con số 16 hết đúng)*. 🔴 **HỎI ICE giá trị màu trước khi ghi** — Ice mới ký *đường*, chưa ký *số* (Task 0.4 ①).
  - [x] 4.5 `check-commands.mjs` Kiểm I: `EXPECTED_RULE_VALUES` thành sáu; sửa tiêu đề *"ĐÚNG NĂM giá trị"*; sửa `EDITOR_PANEL_VUE` → đường dẫn `GridPanel.vue`; thêm ca thứ sáu cho mệnh đề ②.
  - [x] 4.6 `tests/frontend/editorSegmentRule.test.ts`: đảo mệnh đề *"đã dịch chưa ký ⇒ none"* thành *"⇒ draft"*, **thêm** ca `target_text` rỗng ⇒ `none`, và ca *"hai chỗ đọc trạng thái phải đồng ý"* giữ nguyên.
  - [x] 4.7 Cột **nhãn trạng thái** hiện sáu nhãn — sáu khoá `vi.json` mới, khoá phẳng (B12).
  - [x] 4.8 🔴 Sửa tại chỗ *(🔵 + ngày)* hai mệnh đề tài liệu đã hết đúng: `EXPERIENCE.md:99` *("vạch lề đã dùng hết **năm** giá trị")* → sáu, **lý do không đổi**; `DESIGN.md:386` *("đây là **cách duy nhất** trạng thái segment được hiển thị; văn bản không bị chia khối")* → lưới có thêm **cột nhãn trạng thái**, và văn bản **có** chia ô.

- [x] **Task 5 — Hợp đồng vùng chọn theo cột (AC7, B1)**
  - [x] 5.1 Cột nguyên văn: **một** `useSelectionSurface(colSrc, 'source', resolveHanViet)`; cột bản dịch: **một** `useSelectionSurface(colTgt, 'display')`. Vai viết **LITERAL**.
  - [x] 5.2 🔴 `selectionContract.ts` **không sửa một dòng** (AC7).
  - [x] 5.3 Hán Việt: gộp `SourceHanViet.vue` vào cột nguyên văn sao cho **vẫn chỉ một** lời gọi đăng ký cho cả cột — resolver duyệt âm tiết trong toàn cột, không đăng ký theo ô. Giữ nguyên: `WORD_JOINER` không ra clipboard *(Story 1.18b AC5)* và luật *"một âm tiết là ATOM với ký tự nguồn của nó"*.
  - [x] 5.4 `check-commands.mjs` Kiểm F: `SELECTION_PANEL_FILES` thay hai tệp cũ bằng `GridPanel.vue`; 🔴 mệnh đề ① *("mỗi panel ĐÚNG MỘT lời gọi")* phải nới thành **một số mong đợi theo tệp** *(GridPanel = 2)*, kèm chú thích nói vì sao — nới **có chủ**, không bỏ. `SELECTION_SURFACE_FLOOR = 7` **không đổi** *(hai lời gọi thay hai lời gọi)* — đếm lại để chắc.
  - [x] 5.5 `tests/frontend/editorAutoLookup.test.ts`: cập nhật theo hai cột, giữ nguyên mệnh đề *"cột bản dịch KHÔNG phát lượt tra"*.

- [x] **Task 6 — Ba lệnh bàn phím (AC10, AC11, AC12, AC13)**
  - [x] 6.1 `⌘Enter` → `editor.confirm_segment` **đã tồn tại** *(Story 2.5, `commands/index.ts:883`)*. 🔴 **Tái dùng, đừng đăng ký lần hai.** Vế *"sang câu kế"* cũng đã có *(Quyết định #1 của Story 2.5)*.
  - [x] 6.2 `⌥↓` → command **mới**, id theo văn phạm khoá chấm *(gợi ý `editor.next_untranslated`)*, `labelKey` trong `vi.json`. 🔴 *"Chưa dịch"* = `status === 'draft'` **và** `target_text === ''` (AC12).
  - [x] 6.3 🔴 `Enter` **trơn không bao giờ xác nhận** (AC11). Giữ nguyên `if (event.isComposing) return` **trước** mọi nhánh khác *(`EditorPanel.vue:841`)*. ⚠️ `Enter` trong ô bản dịch **vẫn bị chặn** ở story này — quyền xuống dòng là **FR134/AD-46, Story 2.5d**. Đừng mở sớm.
  - [x] 6.4 Cả ba đi qua `dispatch()`; phím tắt và chuột phát **cùng một** `dispatch` (AC13, B8).
  - [x] 6.5 Sàn `COMMAND_FLOOR`/`DISPATCH_FLOOR` của `check-commands.mjs`: đếm lại sau khi thêm một command và bớt một `layout.toggle_*`.

- [x] **Task 7 — Hán Việt trong ô nguyên văn và PHÉP ĐO chiều cao hàng (AC8, nợ `:2863-2873`)**
  - [x] 7.1 Hai chế độ FR19 *(chuyển đổi / song song)* chạy **bên trong ô nguyên văn**.
  - [x] 7.2 🔴 **Không mặc định thông minh nào** (AC8): không buộc chế độ đi theo bố cục, không tự mở riêng cho hàng đang sửa. Hai phương án đó **đã được nêu và bị bác** *(`EXPERIENCE.md:249-257`, Ice ký)*.
  - [x] 7.3 🔴 **ĐO, không ước:** dựng thật, bật *song song* ở **Ⓑ-2**, đếm số dòng của một hàng và đo px trên **cả hai engine**. Ghi số + ngày + phiên bản. Đối chiếu với ước lượng *"6–7 dòng, cột ~330 px"*.
  - [x] 7.4 Nối kết quả vào `deferred-work.md:2863-2873` — ✅ nếu đo xong và chấp nhận được; 🟡 kèm phần còn hở nếu số xấu. **Không xoá mục.**

- [x] **Task 8 — Đo hiệu năng và bàn giao số cho Story 2.4 (B9, Quyết định #7)**
  - [x] 8.1 Đo lượt dựng lưới trên một Chương thật *(và trên fixture 9.850 câu để so với mốc cũ)*: thời gian dựng, số node DOM, thời gian một lượt `selectionchange`.
  - [x] 8.2 So với mốc cũ **và nói rõ mốc nào mất hiệu lực theo cấu trúc**: `deferred-work.md:2113-2129` *(9.850 `<span>`)* · `:2198-2207` *(`:data-caret` dựng lại toàn danh sách)* · `:2484-2489` *(`restoreEditedText` quét cả `.doc`)* · `:2770-2782` *(`assignGutterLanes`)*.
  - [x] 8.3 🔴 **Không tự chấm NFR2 đạt.** Ghi số vào `deferred-work.md` với chủ **Story 2.4**.

- [x] **Task 9 — Đường ra màn hình cho lỗi (AC14, Quyết định #8)**
  - [x] 9.1 `main.ts:229` **đọc** `ConfirmResult` thay vì `void`.
  - [x] 9.2 `GridPanel.vue` đọc `editorConfirmError` và hiện ở cột nhãn trạng thái của hàng liên quan.
  - [x] 9.3 `'still-dirty'` *(Quyết định #8 của Story 2.5)* đi cùng đường, không chỉ `console.error`.
  - [x] 9.4 Nối `deferred-work.md:2801-2816`: ✅ hoặc 🟡 kèm phần còn hở. **Không xoá mục.**

- [x] **Task 10 — Gỡ `editorGutter.ts` và giữ bằng chứng (Quyết định #4)**
  - [x] 10.1 Gỡ *(đường (a), Ice ký 2026-08-14)*; **chép nguyên phép đo** *(482,4/254,5/261,6 ms → 8,3/5,2/4,3 ms, 2026-08-14, Node 22.22.2, macOS 15.6)* và lý do gỡ vào `deferred-work.md`, nối tiếp mục `:2875-2885`.
  - [x] 10.2 Món nợ *"từ 8 làn trở lên máng 22px hết chỗ"* *(`:2718-2719`, Chủ: Ice)* — nếu khái niệm làn biến mất thì nối **✅ đóng theo cấu trúc**, nói rõ *"biến mất, không phải được vá"*.
  - [x] 10.3 Đếm lại `FILE_FLOOR` *(`check-layout.mjs:97`)*, `TS_FLOOR`/`VUE_FLOOR`/`COMPONENT_FILE_FLOOR` *(`check-commands.mjs`)* sau khi bớt/thêm tệp. ⚠️ Sàn là **cận dưới**: bớt tệp không làm cổng đỏ, nó chỉ làm sàn vô nghĩa.

- [x] **Task 11 — Test frontend (vitest)**
  - [x] 11.1 `editorSegmentRule.test.ts` — sáu giá trị, thứ tự ưu tiên, hai ca `draft` vs `none`.
  - [x] 11.2 Test mới cho phép chọn *"câu chưa dịch kế tiếp"* — hàm **thuần**, nhận danh sách segment, trả `id`. 🔴 Đặt ở một module thuần để `check:commands` gọi được bằng Node trần nếu cần.
  - [x] 11.3 🔴 **Đừng thêm `?.` vào mã sản phẩm cho hết đỏ.** Khoảng thiếu của `happy-dom` vá ở `tests/frontend/support/setup.ts`, mỗi mục kèm một dòng nói nó thiếu gì và ai đọc.
  - [x] 11.4 ⚠️ `happy-dom` **không phải WebKit**: mọi mệnh đề về **hình học** *(chiều cao ô rỗng, `subgrid` giữ hàng thẳng, caret)* thuộc **bàn đo/e2e**, không thuộc vitest (AC25).
  - [x] 11.5 `tsconfig.json` phải `include` cây test; `import { describe, it, expect } from 'vitest'` tường minh.

- [x] **Task 12 — e2e (B11)** ✅ *cả bộ 7/7 xanh — xem §Debug Log Ⓔ*
  - [x] 12.1 Sửa `editor-typing-flush.e2e.mjs` và `editor-confirm-segment.e2e.mjs` theo neo mới. 🔴 `[data-segment-id]` không còn duy nhất — mọi phép đếm và mọi `$()` phải nói rõ **cột nào**.
  - [x] 12.2 Spec mới: bấm vào một ô **chưa dịch** đặt được con trỏ, gõ được một ký tự (AC3). Đây là ca đã đỏ suốt Story 2.3.
  - [x] 12.3 🔴 Cấm `.click()` của driver — dùng `realClick()` *(`e2e/support/pointer.mjs`)*.
  - [x] 12.4 ⚠️ Ghi **cách chạy** *(riêng từng tệp hay cả bộ)* cạnh mọi con số — bộ e2e đỏ oan khi chạy cả bộ *(`deferred-work.md`, Chủ: Story 2.4)*, nên một số không kèm cách chạy là một số không so sánh được.
  - [x] 12.5 ⚠️ `browser.keys()` đánh rơi `Meta` đúng ở phím `Enter` — giới hạn bộ đo, **không** phải sản phẩm *(Chủ: Story 1.22)*. Đừng chẩn đoán lại.

- [x] **Task 13 — Tài liệu và sổ nợ**
  - [x] 13.1 `src/panels/README.md` — bảng *"Ranh giới sở hữu"* đang **trễ tiến độ**; cập nhật cho `GridPanel.vue` và gỡ hai panel cũ.
  - [x] 13.2 `src/layout/README.md`, `src/commands/README.md` — mọi chỗ nói *"bốn panel"*.
  - [x] 13.3 🔴 `grep` toàn cây cụm *"bốn panel"* / *"panel.source"* / *"panel.editor"* — **80 chỗ trong 14 tệp** đo 2026-08-14. Mỗi chỗ hoặc sửa, hoặc mang một dòng 🔵 nói vì sao nó là **lịch sử**.
  - [x] 13.4 `deferred-work.md`: đóng/nối mọi mục ở §Nợ mà story này nhận; **mở** mục mới có chủ cho mọi thứ không nghiệm thu được.
  - [x] 13.5 `sprint-status.yaml`: `2-5b-...` → trạng thái tiếp theo, comment **không dấu**.

- [x] **Task 14 — Nghiệm thu**
  - [x] 14.1 11 cổng npm xanh *(gồm `check:scope` + `check:scope:bundled` chạy tay, cần cổng 1420 trống)*.
  - [x] 14.2 `npm run build` · `npm run test` · `cargo test --locked` *(mốc trước story: **338/0/5**)*.
  - [x] 14.3 🔴 Mỗi cổng/test mới phải chạy **đỏ-rồi-xanh** ít nhất một lượt và ghi lại. Một cổng chưa bao giờ đỏ là một cổng chưa ai biết nó có chạy không.
  - [x] 14.4 Ghi bảng nghiệm thu số **thật** trước/sau vào §Completion Notes.

---

## Dev Notes

### Bản đồ tệp sẽ SỬA

| Tệp | Hôm nay làm gì | 2.5b chạm chỗ nào | Phải giữ nguyên |
|---|---|---|---|
| `src/layout/workspaceLayout.ts` *(189 dòng)* | Bốn panel, hai preset, thứ tự hy sinh | `PanelId` · `PANEL_IDS` · `PANEL_TITLE_KEYS` · `PANEL_COMPONENTS` · hai preset · `NEVER_SACRIFICED` *(:155)* · chú thích *"bốn panel"* *(:154)* | Luật **erasable-only**; `nextToSacrifice`/`nextToRestore` là **hàm thuần**, không đọc `window`; `SACRIFICE_ORDER` không đổi nội dung; ngưỡng màn hình hẹp vẫn là **Story 4.12** |
| `src/panels/GridPanel.vue` *(mới)* | — | Toàn bộ lưới | — |
| `src/panels/EditorPanel.vue` *(1.233 dòng)* | Panel Bản dịch, trang liền mạch | Gỡ khỏi bộ panel; **chuyển** phần còn dùng sang `GridPanel.vue` | 🔴 `if (event.isComposing) return` *(:841)* · `onBeforeInput` là **cửa duy nhất** cho mọi sửa văn bản, chặn theo `inputType` **không theo phím** *(:764-828)* · thứ tự `mousedown` → `mouseup` → `placeCaretAtPoint` *(đo thật trên WKWebView 2026-08-12/13)* · `setAttribute` **đồng bộ** trong `mousedown` *(chẩn đoán đúng của Story 2.3)* |
| `src/panels/SourcePanel.vue` *(291 dòng)* | Panel Nguyên văn, tab Hán Việt | Gỡ khỏi bộ panel; cột nguyên văn kế thừa | `sourcePanelState.ts` sống **module-level** để sống sót qua đổi preset (AC9 Story 1.16) |
| `src/panels/SourceHanViet.vue` *(933 dòng)* | Bề mặt Hán Việt, hai chế độ, resolver | Vào **trong ô nguyên văn**; **một** lần đăng ký cho cả cột | `resolveSelection` hai đường *(`parallel`/`switch`)* đọc **DOM**, không `toString()`; `WORD_JOINER` không ra clipboard; *"một âm tiết là ATOM với ký tự nguồn"* |
| `src/panels/editorSegments.ts` *(204 dòng)* | Bảng ánh xạ trạng thái → vạch, **năm** giá trị | Thành **sáu**; nhánh `draft`; ba khối doc-comment hết đúng | 🔴 **MODULE THUẦN — không `import` giá trị nào.** Một dòng `import` giết Kiểm I |
| `src/panels/editorGutter.ts` *(273 dòng)* + `tests/frontend/editorGutterLanes.test.ts` | Hình học vạch + chia làn | Gỡ theo Quyết định #4 | Phép đo phải sống tiếp trong `deferred-work.md` |
| `src/panels/selectionContract.ts` *(383 dòng)* | Hợp đồng vùng chọn | 🔴 **KHÔNG SỬA MỘT DÒNG** (AC7) | Tất cả |
| `src/commands/index.ts` *(1.149 dòng)* | Đăng ký command, `FOCUS_OWNERS` | `FOCUS_OWNERS` *(:64-72)* · `PANEL_SUFFIXES` *(:375)* · một command `⌥↓` mới | `editor.confirm_segment` *(:883)* **tái dùng**, không đăng ký lại; luật erasable-only |
| `src/main.ts` *(438 dòng)* | Thứ tự khởi động, tiêm cổng | `:229` đọc `ConfirmResult` (AC14) | 🔴 Thứ tự ba mệnh đề khởi động: `applyTheme()` trước `mount()`; `installCommands()` trước `mount()`; `loadFonts()` **trước** `await loadBootstrapConfig()`. Đăng ký command ở `main.ts`, **không** `App.vue` |
| `src/i18n/vi.json` | Chuỗi giao diện | Khoá `panel.grid.*`, sáu nhãn trạng thái, nhãn lệnh mới, khoá lỗi hiển thị | Phẳng, khoá chấm, placeholder `{ten_tham_so}`, **không** giá trị rỗng |
| `src/tokens/tokens.json` + `DESIGN.md` | 16 token | Token `draft` *(Quyết định #2)*; `contrast.pairs` cho cặp mới; gỡ miễn trừ `⏐` đã chết *(Quyết định #9)* | Vai không đổi theo theme; `ornament`/`tm-rule` không bao giờ làm màu chữ |
| `scripts/check-commands.mjs` *(2.229 dòng)* | 9 Kiểm | Kiểm F *(`SELECTION_PANEL_FILES`, số lời gọi mong đợi)* · Kiểm I *(`EXPECTED_RULE_VALUES` sáu, `EDITOR_PANEL_VUE` → `GridPanel.vue`, ca thứ sáu)* · các sàn | 🔴 Luật của một cổng: mã thoát là phán quyết; lỗi hạ tầng ⇒ `abort()`; **không** đọc tham số từ chính thứ nó kiểm |
| `scripts/check-layout.mjs` *(610 dòng)* | 4 Kiểm | Ba chuỗi id **cứng** ở Kiểm A mệnh đề 3 *(:255-272)* · `FILE_FLOOR` | Kiểm C là **danh sách CHO PHÉP** cho `window`/`document` — thêm một cái tên là một quyết định phải viết ra |
| `e2e/specs/editor-*.e2e.mjs` | Hai spec | Neo `data-segment-id`, `.doc` | `realClick()`, không `.click()` |

**Rust: KHÔNG chạm.** `commands/segment.rs` · `core/store/schema.rs` · `core/i18n/mod.rs` ·
`lib.rs` — không một dòng. Nếu thấy mình đang mở một tệp `.rs`, dừng lại và hỏi vì sao.

### 🔴 Đừng dựng lại — tám thứ ĐÃ CÓ và phải tái dùng

Lớp lỗi tốn kém nhất của một story tầng vẽ là **dựng lại một đường đã chạy**, vì bản thứ hai
đi qua mọi cổng *(nó là mã hợp lệ)* rồi lệch dần với bản thứ nhất.

| Đã có | Ở đâu | 2.5b dùng thế nào |
|---|---|---|
| Command **xác nhận** + vế *"sang câu kế"* | `commands/index.ts:883` *(`editor.confirm_segment`)* · Quyết định #1 của Story 2.5 | AC10 **chỉ gán phím**, không đăng ký command thứ hai |
| Đường **lưu bố cục qua phiên** | `WorkspaceMode.vue::onPersist` + `bootstrap.ts::workspace_layout` | AC6 vế "qua các phiên" — **0 dòng mới** |
| Nhịp **flush Editor** | `panels/editorFlush.ts` *(`EDITOR_IDLE_MS 2000` / `EDITOR_HARD_CAP_MS 5000`)*, dựng bằng `createWriteSchedule` — Quyết định #2 của Story 2.3, đường (a) | Tái dùng. 🔴 **Đừng dựng lịch thứ hai**, và **đừng sửa `createWriteSchedule` "cho hợp lưới hơn"** — `check-layout.mjs` Kiểm B đứng trên chính hàm đó. ⚠️ Hai cặp hằng dùng chung *hình dạng*, không dùng chung *bảo đảm* AD-35 — đừng gộp |
| Khung panel: tiêu đề, vạch tiêu điểm, khai điểm vào focus, dòng trạng thái | `panels/PanelFrame.vue` | `GridPanel.vue` bọc trong `<PanelFrame>`, không tự vẽ thanh tiêu đề |
| Vị từ **"đang ở vùng gõ"** | `commands/keys.ts:434` — `isTypingZone`, **không export**, dùng ở `:510` để một hợp âm **không có phím bổ trợ chính** không cướp phím lúc đang gõ | Định tuyến bàn phím ở lại `keys.ts`. Đừng viết một phép kiểm `contenteditable` thứ hai trong `GridPanel.vue`; nếu cần dùng ngoài, **nâng nó lên** một chỗ và ghi lý do — đừng chép |
| **Hợp đồng vùng chọn** và phép loại trừ ô nhập | `panels/selectionContract.ts` *(tín hiệu là `nodeType`, không `activeElement`, không `toString()` — đo trên cả hai engine 2026-08-07)* | Chỉ **đăng ký**; AC7 cấm sửa |
| Bảng **trạng thái → vạch** | `panels/editorSegments.ts::resolveSegmentRule` | Thêm **một** nhánh. Đừng phân giải trạng thái ở `GridPanel.vue` |
| Đường đọc dữ liệu | `config/segment.ts::readOpenChapterSegments` — đã mang đủ `source_text` · `target_text` · `is_paragraph_end` · `retired_at` · `status` | **Không** lệnh IPC mới, **không** cột mới, **không** bước di trú |

### Bất biến không được phá

| Bất biến | Nguồn | Vi phạm được mà không cổng nào đỏ? |
|---|---|---|
| Vai `'source'`/`'display'` đúng cột | Sprint Change 2026-08-13 · Kiểm F ③ | ⚠️ **Nửa** — Kiểm F ③ canh **tệp**, và tệp nay là một. Đổi vai giữa hai cột **trong cùng tệp** có thể đi lọt ⇒ Task 5.4 phải canh **cả hai** vai, không chỉ sự tồn tại |
| `selectionContract.ts` không sửa | AC7 | Không — `git diff` thấy ngay |
| Hợp đồng flush AD-35 | AD-35 · `editorFlush.ts` | 🔴 **CÓ.** Vế (c) *"xác nhận"* chỉ cưỡng chế ở **một** chỗ gọi. Một bề mặt xác nhận thứ hai đi vòng qua nó **không cổng nào đỏ** *(nợ có chủ, `deferred-work.md`)* |
| `is_paragraph_end` chỉ đọc | AD-37 | 🔴 **CÓ.** Một dòng `text.split('\n')` lúc render đi qua mọi cổng và làm hỏng FR121 ở Epic 8 |
| Không quy tắc nghiệp vụ ở TS | AD-1 | 🔴 **CÓ.** Không cổng nào đọc được ý định |
| `ornament` không làm màu chữ | `tokens.json` `neverTextTokens` | Không — `check:tokens` bắt |
| Erasable-only ở `workspaceLayout.ts` | `check-layout.mjs` Kiểm A | Không — cổng nạp tệp bằng Node trần và chết ngay |
| `Enter` trơn không xác nhận | AC11 | 🔴 **CÓ, và tệ nhất.** Không đường nghiệm thu nào của dự án mô phỏng được một bộ gõ tiếng Việt thật. Chỉ tay Ice bắt được |
| Không thêm dep npm ngoài cửa NFR15 | NFR15 | ⚠️ Nửa — `check:deps` canh **sáu tên bị cấm**, không canh một gói mới hợp lệ nhưng chưa rà giấy phép |

### Bài học từ story trước — đọc trước khi chẩn đoán

- 🔴 **"Trúng tiền đề chưa phải trúng cơ chế."** Story 2.3 chẩn đoán *"AD-34 giành tiêu điểm"* và **sai**: không ai giành cả; nguyên nhân thật là một lượt đặt thuộc tính **bất đồng bộ** *(Vue và DOM ở một microtask sau)*, vá bằng `setAttribute` **đồng bộ** trong `mousedown`. Story 2.4 lặp lại lớp lỗi đó với `AppleKeyboardUIMode`.
- 🔴 **Ba vòng chẩn đoán bị bác ⇒ DỪNG và báo Ice.** Story 2.4 treo vì bốn lượt build hỏng và bốn giả thuyết bị bác.
- ⚠️ **Đừng kết luận từ n=1 trên một bộ đo đã ghi là chập chờn.** Kết luận *"sản phẩm không hỏng"* của Story 2.3 đã bị **RÚT LẠI** vì dựa trên một mẫu n=1 *(`deferred-work.md:2528-2584`)*.
- ⚠️ **Baseline lấy bằng ĐO, không bằng phép trừ.** Story 2.1 ghi 274 khi số thật là 267; Story 2.5 ghi 40 khi số thật là 41. Chạy `cargo test --locked` và `npm run test` **trước** khi sửa dòng đầu tiên.
- 🔴 **Bàn đo cũng sai được.** Bản đầu của bàn đo Story 2.5 gom vạch chồng nhau **bắc cầu** và lượt chạy đầu bác ngay; phép đúng là **tô màu đồ thị khoảng**. Một bàn đo là mã, và mã thì sai được.
- 🔴 **e2e bắt được thứ bốn đường kia bỏ lọt.** Story 2.5: `read_open_chapter_segments` không gửi `status` qua dây ⇒ `isConfirmed` **luôn `false`** trong app thật, trong khi **74/74** vitest vẫn xanh — vì fixture chép tay tự cấp `status`. ⇒ Với một story tầng vẽ, **fixture chép tay là chỗ dối trá rẻ nhất**.

### Số đo đã có — dùng để SO, đừng đo lại

| Số | Ngày · môi trường | Ý nghĩa cho 2.5b |
|---|---|---|
| Dựng 9.850 `<span>`: **300,1 ms** (Blink) · **1.308,0 ms** (WebKit) — vượt trần 50 ms/frame 6× và 26× | `deferred-work.md:2113-2129` | Mốc để so. Lưới **tăng** node mỗi câu ⇒ nhiều khả năng xấu hơn nếu không đổi gì khác |
| `assignGutterLanes` O(n²) **482,4 / 254,5 / 261,6 ms** → quét đường **8,3 / 5,2 / 4,3 ms** | 2026-08-14 · Node 22.22.2 · macOS 15.6 · 9.850 vạch | Bằng chứng phải sống tiếp trong sổ dù mã bị gỡ |
| Mỗi câu rỗng đẩy chữ **9,05 px** ⇒ bố cục nhảy | 2026-08-13 | Lý do vế ② của lượt lật. Trong lưới nó **hết đúng** — hàng có chiều cao riêng |
| `prepare_cached`: 105–112 ms → 44–50 ms trên 9.850 hàng | 2026-08-12 | Đường đọc Rust **không** phải nút thắt |
| Cây phụ thuộc mặc định **831** dòng · `--features wdio` **948** | 2026-08-11 | Mốc AD-45 — thêm một dep sẽ dời số này |
| `pre-push`: cổng 11 s · build 5 s · `cargo test` 34 s | 2026-08-11 | Ngân sách một lượt đẩy |
| Baseline story: `cargo test` **338/0/5** · vitest **78/78** · 11 cổng xanh | sau code review Story 2.5 | ⚠️ **Đo lại trước khi sửa** — con số này có thể đã trôi |

### Thư viện và phiên bản

Đọc số trần từ `package.json` · `src-tauri/Cargo.toml`, **đừng chép sang đây**. Điều đáng nhớ:

- `dockview-vue` **7.0.4** — bảng Stack ghi giấy phép MIT với một dấu ⚠️ *(bằng chứng giấy phép yếu hơn)*. Không nâng phiên bản trong story này.
- `vitest` **4.1.10** · `@vue/test-utils` **2.4.11** · `happy-dom` **20.11.2** — ba gói Ice ký 2026-08-12 khi **lật NFR15**. ⚠️ Cửa rà giấy phép của NFR15 **vẫn đứng** cho gói tiếp theo.
- **Không thư viện lưới/bảng/ảo hoá nào đã được duyệt** trong bảng Stack. *"Chiến lược ảo hoá danh sách dài"* còn ở **Giai đoạn 3** *(`ARCHITECTURE-SPINE.md:922`)*.
- `subgrid` *(nếu Quyết định #1 chọn (b))* là **CSS**, không một phụ thuộc — không đi qua NFR15. Sàn: Safari 16+ *(macOS 12.4+)* · Chrome/Edge 117+. 🔴 Ba engine **bất đồng ở gap và auto-sizing** ⇒ **đo**, đừng đọc bảng.
- `verbatimModuleSyntax` bật ⇒ `import type` tường minh. Không `globals: true` ở vitest.

### Cổng nào sẽ nhìn story này

| Cổng | Nhìn gì | Sẽ đỏ ở đâu nếu làm ẩu |
|---|---|---|
| `check:commands` Kiểm A/B | `@click` là đúng một `dispatch`; id có trong bộ đăng ký | Nhét logic vào `@click` của hàng |
| `check:commands` Kiểm E | `FOCUS_OWNERS` **hai chiều** với `owner=` trong `.vue` | Quên gỡ `panel.source`/`panel.editor` |
| `check:commands` Kiểm F | Đăng ký hợp đồng vùng chọn, vai, sàn 7 | Hai lời gọi trong một tệp *(phải nới có chủ, Task 5.4)* |
| `check:commands` Kiểm I | Sáu giá trị vạch, phân giải thật, CSS `.rule-*` **hai chiều** | Thêm `draft` mà quên CSS, hoặc quên đổi `EDITOR_PANEL_VUE` |
| `check:layout` Kiểm A | Thứ tự hy sinh, hai tập rời nhau hợp lại đúng `PANEL_IDS` | `NEVER_SACRIFICED` còn hai phần tử khi `PANEL_IDS` còn ba |
| `check:layout` Kiểm C | Danh sách **CHO PHÉP** mọi thành viên `window`/`document` | `ResizeObserver`, `IntersectionObserver`, `document.getSelection` — mỗi cái là một quyết định phải viết ra |
| `check:tokens` Kiểm B/B2/C/D/F | Màu và cỡ chữ chỉ từ token; cặp mới phải khai; `opacity` trung gian cần miễn trừ **có tên**; không bóng/gradient/lớp nổi | `opacity: 0.45` cho `.rule-draft`; màu viết thẳng cho nền hàng |
| `check:i18n` Kiểm A/E | Không chữ tiếng Việt có dấu ở **vị trí mã**; `resolve.ts` nạp được bằng Node trần | Một chuỗi nhãn viết thẳng trong `.vue` |
| `check:gates` | Ba danh sách cổng khớp nhau | Thêm một cổng mà quên `.githooks/pre-push` |
| `check:lint` | `@typescript-eslint/no-unnecessary-condition` **có kiểu** | 🔴 `Ref` **không tự bóc** trong `<script>` — `if (someRef)` chạy trên **đối tượng** và **luôn đúng**. Lỗi này đã lọt qua **chín trên chín** cổng một lần |
| vitest | Hành vi module thuần | — |
| e2e | Hành vi trong **WKWebView thật** | Neo `data-segment-id` không còn duy nhất |

🔴 **Chỗ hở lớn nhất của story này:** ~~**không một test mount component nào tồn tại** trong
`tests/frontend/**`~~. Toàn bộ tái cấu trúc DOM **không có lưới tự động nào** ngoài Kiểm F,
Kiểm I và Kiểm A của `check:layout`. ⇒ Bàn đo và e2e **không phải phần thêm**, chúng là đường
nghiệm thu chính của Task 1, 3, 7.

🔵 **Mệnh đề gạch ngang ở trên HẾT ĐÚNG, sửa tại chỗ 2026-08-15 (lượt code review).**
`tests/frontend/statusBar.test.ts` `mount(StatusBar)` bằng `@vue/test-utils` **từ Story 2.3
(2026-08-12)** — tức nó đã sai **lúc story này được viết**, không phải hết đúng về sau. Hệ quả
đo được: vế *"đường ra thanh trạng thái"* của Quyết định #8 đóng được bằng **sáu ca vitest**
*(bộ 83 → 89)*, rẻ hơn hẳn một lượt e2e chạy tay — và nó suýt bị giao cho e2e chỉ vì một câu
trong tài liệu.

⚠️ **Kết luận đúng thì KHÔNG đổi, và đó mới là chỗ đáng nhớ:** mệnh đề *"tầng vẽ ít cổng canh
nhất"* vẫn **đứng nguyên**. `mount()` chạy trên `happy-dom`, thứ **không phải WebKit** — nên nó
trả lời được *"câu này có hiện lên không"* và **không** trả lời được *"caret có xuất hiện trong
ô rỗng không"*. Một tiền đề sai đã dẫn tới một kết luận đúng; sửa tiền đề, giữ kết luận.
⇒ Guard `caretTarget` của `ensureCaretNextFrame` **vẫn** thuộc e2e, và nó là vế còn hở duy nhất
của lượt rà *(`deferred-work.md` §code review of 2-5b, mục 🟡)*.

### Project Structure Notes

- Component mới: `src/panels/GridPanel.vue` — `PascalCase.vue`, đúng khuôn `src/panels/`.
- Module thuần đi kèm *(nếu có)*: `src/panels/*.ts`, **không** `import` giá trị nếu một cổng cần nạp nó bằng Node trần.
- Test: `tests/frontend/**` **phẳng**, **không** đồng vị trí trong `src/**` — bốn cổng đếm quần thể `src/**`, và một tệp test đổ vào đó thổi phồng mẫu số *(cộng hai va chạm: Kiểm A của `check-i18n` đỏ với chữ tiếng Việt; Kiểm B của `check-tokens` đỏ với màu viết thẳng)*.
- Bàn đo: `_bmad-output/implementation-artifacts/2-5b-ban-do*` — **ngoài** cây nguồn, khuôn `2-5-ban-do-hai-vach.html` + `2-5-ban-do/`.
- Thư mục mang một khái niệm thì có `README.md` — `src/panels/`, `src/layout/`, `src/commands/` đều có và **cả ba phải cập nhật**.

### Văn hoá viết mã của kho này

- **Chú thích tiếng Việt, dày, chở LÝ DO** — không kể mã làm gì, mà trả lời *vì sao hình dạng này chứ không phải hình dạng kia*, và **phương án bị loại đã bị loại bằng gì**.
- 🔴 **Một quyết định không hiển nhiên phải kèm một PHÉP ĐO, không một sở thích** — con số, ngày đo, `tệp:dòng`. Khuôn: *"⚠️ Đo 2026-08-14: … Hệ quả đo được: …"*.
- **Ghi thẳng chỗ YẾU thay vì giấu.** Mỗi module chở một mục *"GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện"*.
- **Khi một mệnh đề hết đúng, SỬA TẠI CHỖ** kèm 🔵 và ngày — story này có **ít nhất sáu** chỗ như vậy *(Task 4.3, 4.8, 2.3, 2.6, 13.1-13.3)*.
- Ký hiệu: 🔴 luật không được phá · ⚠️ bẫy hoặc chỗ dễ đọc nhầm · ✅ đã đóng · 🟡 đóng một nửa · 🔵 mệnh đề cũ đã hết đúng · ⇒ kết luận. 🔴 **Emoji biển cấm `U+26D4` bị CẤM trong toàn kho** — viết `không`/`KHÔNG` thành chữ.
- 🔴 **Đừng bắt chước một ký hiệu chưa hiểu** — `grep` đếm số lần **và tìm định nghĩa** trước khi dùng lại.

### Git — bối cảnh gần nhất

- Nhánh mặc định là **`master`**, không `main`. Cây **sạch** ở `f990dd5`.
- Commit message: `type(scope): câu tiếng Việt`; `scope` = `story-2.5b`. Câu sau dấu hai chấm **nói ĐIỀU ĐÃ TÌM RA**, không chỉ điều đã sửa.
- ⚠️ Commit `74f8825` gom **hai lớp làm một** *(Story 2.5 + bốn tệp spine UX)* nên diff của 2.5 không đọc được một mình — đã ở trên remote, ghi lại thay vì rewrite. **Từ đây trở đi mỗi lớp một commit sạch.**
- `pre-push` chạy: chín cổng đọc-tệp → `npm run test` → `npm run build` → `cargo test --locked`. Đỏ là **dừng**. Bỏ qua bằng `--no-verify` thì **phải viết lý do vào commit message**.
- ⚠️ Nửa Windows **không có đường nghiệm thu tại chỗ** — `pre-push` chạy trên macOS. Khoảng mù dày lên theo từng epic.

### Điều story này CỐ Ý không làm

| Không làm | Vì sao · chủ |
|---|---|
| Cờ **cắt bỏ** câu, hàng gạch ngang mờ | **FR133 · Story 2.5c**, bước di trú **8**. Bản dựng có `tr.row-omitted` — đó là **xem trước**, không phải phạm vi |
| `Enter` xuống dòng trong ô bản dịch, cờ đoạn của bản dịch | **FR134 · AD-46 · Story 2.5d**, bước di trú **9**. 2.5b **giữ** hai lớp chặn `Enter` |
| `Backspace` đầu ô = gộp với câu trên | **UX-DR32 · Story 2.9**. 2.5b chỉ dựng **tiền đề** *(ô là editing host riêng)*, không cài ngữ nghĩa gộp |
| `⌘T` tách câu ở cột nguyên văn | **Story 2.8** |
| Bốn **ngưỡng** màn hình hẹp, ngăn kéo, *"rút Tra cứu về thanh trạng thái"* | **Story 4.12** — `epics.md` cấm tường minh việc đóng chúng ở tầng bố cục. 2.5b giao đúng **cơ chế**; ⚠️ 4.12 nay phải hiệu chỉnh cho **hai** bố cục |
| Ảo hoá danh sách dài | **Giai đoạn 3** *(spine)* · Quyết định #7 |
| Nguồn `tm-rule` thật | **Epic 7** — `isTmFilled` vẫn là hằng `false` |
| Nguồn `ornament` thật *(segment về hưu)* | **Story 2.8** |
| Đánh dấu **NFR2 đạt** | **Story 2.4** sở hữu bộ đo. 2.5b **giao số** |
| Sửa `EXPERIENCE.md:105-113` cho có hàng của Quyết định #3 Story 2.5 | ✅ **Đã xong** trong lượt correct-course — bảng nay có sáu hàng. Món nợ *(Chủ: Ice)* đóng được, kiểm rồi nối vào sổ |
| Sync Scrolling | **FR20 đã RÚT** *(Ice ký)*. ⚠️ Nếu Epic 4 dựng dịch **theo lô** thì nhu cầu quay lại và không FR nào chứa nó — nợ có chủ **Epic 4** |

---

## References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 2.5b`] — dòng **2252-2329**, mười bốn AC nguyên văn và món nợ Hán Việt.
- [Source: `epics.md#UX Design Requirements`] — **:530** UX-DR13 *(lưới, năm cột, ô trống có chiều cao thật, khoảng thở)* · **:532-539** bảng Ⓑ-1/Ⓑ-2 · **:543-545** UX-DR15 *(bốn ngưỡng, thứ tự hy sinh, kéo theo `NEVER_SACRIFICED`)* · **:551** UX-DR17 · **:555-561** UX-DR19 *(sáu giá trị, `draft` không cần di trú)* · **:563-565** UX-DR20 **RÚT** · **:569** UX-DR22 *(năm → sáu, lý do không đổi)* · **:595-597** UX-DR32 *(cử chỉ gộp đổi, ngữ nghĩa không đổi)*.
- [Source: `epics.md`] — **:1633, :1647-1649, :1672** hai AC của Story 1.14 superseded · **:2092-2096** bốn AC của Story 2.2 superseded và **ba quyết định đúng chồng thành một bề mặt sai** · **:664** FR16 · **:864-874** ghi chú cài đặt Epic 2.
- [Source: `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md`] — **:395** FR16 *(ba panel)* · **:403** FR19 *(cột nguyên văn, bỏ chữ "tab")* · **:419-421** FR20 **rút** kèm bia mộ · **:423** FR21 *(`selectionContract.ts` không phải sửa một dòng)* · **:458** FR133 · **:462-464** FR134.
- [Source: `.../architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md`] — AD-1 **:75-79** · AD-11 **:153-157** · AD-24 **:322-326** · AD-31 **:368-392** *(máy trạng thái, sáu sự kiện)* · AD-34 **:406-417** · AD-35 **:419-425** · AD-37 **:437-453** · AD-46 **:652-673** *(cấu trúc đoạn của bản dịch; câu cuối: **"đường đối chiếu thật là lưới hai cột của Workspace"**)* · Deferred **:917** *(WAL + nhịp flush)* · **:920** *(thư viện editor)* · **:922** *(ảo hoá danh sách dài, Giai đoạn 3)*.
- [Source: `.../ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md`] — **:105-119** bảng **sáu** giá trị trạng thái + ghi chú lật · **:148-167** Auto-Lookup, *"QuickTranslator là mốc để VƯỢT QUA"*, **bố cục là vế MỞ** · **:192-204** Accessibility Floor + tiêu chí NFR17 *(dịch trọn một Chương không chạm chuột)* · **:205-276** mục lưới hai cột · **:238-247** Ⓑ-1/Ⓑ-2 · **:249-257** Hán Việt + cái giá chiều cao hàng · **:259-275** bảng phím · **:374-386** KF-2.
- [Source: `.../ux-designs/.../DESIGN.md`] — frontmatter **:137-148** *(`grid-row-divider` · `grid-para-divider` · `grid-num-col` · `grid-state-col` · `grid-empty-cell` · `grid-row-omitted`)* · **:175-196** bảng 16 token và *"vì sao 16 chứ không 17"* · **:198-225** sàn tương phản, ba màu đã bị loại · **:266-281** typography · **:386** mệnh đề *"cách duy nhất… văn bản không bị chia khối"* — **đã hết đúng**.
- [Source: `.../ux-designs/.../.working/editor-grid-two-column.html`] — **501 dòng**, bản dựng thật của lưới *(2026-08-14 14:48)*: năm cột, `.empty-cell`, `.rule.*`, `tr.row-primary`/`row-tm`/`row-omitted`, hai preview Ⓑ-1/Ⓑ-2, bảng phím, đoạn *"cái giá của Hán Việt"*. ⚠️ **Minh hoạ, không phải nguồn sự thật** — khi mâu thuẫn thì `DESIGN.md`/`EXPERIENCE.md` thắng. Và **kiểu dáng cấp hàng của nó viết trên `<tr>`**, không chép thẳng được nếu Quyết định #1 chọn (b).
- [Source: `_bmad-output/planning-artifacts/sprint-change-proposal-2026-08-14.md`] — gói Ice duyệt: ba panel · AC của 1.14 superseded · nợ *"thư viện editor"* chuyển chủ sang 2.5b. Bảng tác động 26 mục; §5.2 bước 6-8 *(viết mã)* là phạm vi của 2.5b/2.5c/2.5d.
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md`] — **:2863-2873** *(chiều cao hàng Hán Việt, **CHƯA ĐO**, chủ 2.5b)* · **:2875-2885** *(`editorGutter.ts` mất lý do tồn tại, không xoá im lặng, chủ 2.5b)* · **:2896-2908** *(thư viện editor + "sập hố", chủ 2.5b)* · **:2801-2816** *(ba khoá `err.segment.*` không có đường ra màn hình)* · **:2317-2371, :2528-2584** *(gõ vào câu chưa dịch, `<span>` rỗng 0 px, kết luận n=1 đã bị RÚT)* · **:2113-2129, :2198-2207, :2484-2489, :2770-2782** *(hiệu năng, chủ Story 2.4)* · **:2718-2719** *(8 làn, chủ Ice)* · **:2469-2474** *(happy-dom thiếu ba thứ)* · **:2508-2517** *(tiêu điểm sau remount)*.
- [Source: `_bmad-output/implementation-artifacts/2-5-xac-nhan-segment-va-may-trang-thai.md`] — bảy chữ ký của Ice; ba thứ **phép đo bác** và cách sửa; khuyết tật e2e `read_open_chapter_segments` thiếu `status`; khuôn story file.
- [Source: mã sản phẩm] — `src/layout/workspaceLayout.ts:32,41,54,62,103,117,124,130,152,155` · `src/panels/editorSegments.ts:51,141,159,202` · `src/panels/EditorPanel.vue:97,764-828,841,872-956,1043-1084` · `src/panels/SourcePanel.vue:91` · `src/panels/SourceHanViet.vue:600-647` · `src/panels/selectionContract.ts:52,84-87,107-118,168-190` · `src/commands/index.ts:64-72,375,457,883` · `src/config/segment.ts:59-80` · `src/modes/WorkspaceMode.vue:56-78` · `src-tauri/src/core/store/schema.rs:398-458` · `src-tauri/src/commands/segment.rs:136,160,305,316` · `scripts/check-commands.mjs:222,1887-1892,1908,2021-2160` · `scripts/check-layout.mjs:97,219-233,242,255-272` · `src/tokens/tokens.json:16,53-79,98-101` · `src/i18n/vi.json:16-21,28-29,73-100`.
- [Source: web, 2026-08-14] — `subgrid`: Safari **16+** *(macOS 12.4+)*, Chrome/Edge **117+**; ba engine bất đồng ở **gap** và **auto-sizing** ⇒ đo, đừng đọc bảng. IME: `beforeinput` có thể mang `inputType` khác `input` trong lúc composition; **bộ gõ Telex xử lý `Backspace` ngay sau một từ kể cả khi không có marked text** — cộng thêm một lý do cho AC11 và cho việc `Backspace` đầu ô *(Story 2.9)* phải đi qua `beforeinput`, không qua `keydown`.

---

## Dev Agent Record

### Agent Model Used

*(điền khi dev-story chạy)*

### Chín chữ ký của Ice

**Lượt ký: 2026-08-14. Nguyên văn của Ice: *"duyệt 9 quyết định theo đề xuất"*.**
⇒ Cả chín đều chốt ở **đường ⭐ đề xuất mặc định**.

| # | Quyết định | Đường Ice chốt | Ghi chú |
|---|---|---|---|
| 1 | Hình dạng DOM của lưới + neo `data-segment-id` | **(b)** CSS Grid **chủ-cột với `subgrid`**; cột là tổ tiên DOM thật | ⚠️ Kiểu dáng **cấp hàng** phải nhân ra **năm ô** — bản dựng viết chúng trên `<tr>`, không chép thẳng. 🔴 `subgrid` phải **đo trên cả hai engine** *(Task 1.2/1.3)*. ⚠️ **Vế neo còn mỏng** — xem "Ba chỗ chữ ký chưa phủ hết" |
| 2 | Màu của `.rule-draft` | **(a)** thêm token **`draft`** *(thứ 17)*, vai **`stroke`** | ⚠️ **Giá trị màu hai theme CHƯA có** — xem "Ba chỗ chữ ký chưa phủ hết" |
| 3 | `contenteditable` đặt ở đâu | **(b)** **mọi ô cột bản dịch** là một editing host riêng | 🔴 **Kèm chữ ký LẬT Quyết định #1 của Story 2.3** *(«vùng gõ là một câu tại một thời điểm», Ice ký 2026-08-12)*. Tiền đề cũ — một dòng văn liên tục — **không còn tồn tại**. Ghi cả hai và ghi thứ tự, đừng xoá dấu vết |
| 4 | Số phận `editorGutter.ts` | **(a)** **gỡ** mã, **giữ phép đo trong sổ** | Nối tiếp `deferred-work.md:2875-2885`, không xoá mục |
| 5 | Đổi tên `PanelId` / `PresetId` | **(a)** panel id **đổi**; preset id **giữ tên, đổi nghĩa** | `layout.preset_grid` = **Ⓑ-2** *(mặc định)* · `layout.preset_columns` = **Ⓑ-1**. Phím tắt đã gán và preset đã lưu sống nguyên. ⚠️ Bắt buộc một chú thích 🔵 tại chỗ: *"tên là lịch sử, nghĩa ở bảng ngay dưới"* |
| 6 | Thư viện editor | **(a)** **không thư viện** — `contenteditable` trần, `beforeinput` là cửa duy nhất | 🔴 **Ice ký TRƯỚC khi có số của Task 1** — story đề nghị ký *sau*. Chữ ký này **đóng hàng Deferred `ARCHITECTURE-SPINE.md:920` theo hướng "không thư viện"**, nhưng nó **không** miễn cho Task 1: nếu ba phép đo của 1.2 trượt thì đây là chỗ **mở lại**, không phải chỗ đã khoá |
| 7 | Ảo hoá hàng | **(a)** **không ảo hoá** ở story này; đo và **giao số** cho Story 2.4 | Giữ *"Chiến lược ảo hoá danh sách dài"* ở Giai đoạn 3 *(spine `:922`)*. Số vượt trần thì **báo, đừng tối ưu mù** |
| 8 | Bề mặt báo lỗi `err.segment.*` | **(a)** **cột nhãn trạng thái của chính hàng** + một dòng ở thanh trạng thái | 🔴 Đây đồng thời là **chữ ký hợp đồng UX-DR30 ở phạm vi tối thiểu**. Đóng cả `'still-dirty'` cùng lượt. Không hộp thoại, không lớp nổi (UX-DR16) |
| 9 | Màu chữ cột số câu và cột nhãn trạng thái | **(a)** **`on-surface-variant`**; sửa `DESIGN.md` frontmatter tại chỗ; **gỡ** miễn trừ `⏐` đã chết | Qua `check:tokens` không cần một miễn trừ nào |

#### ⚠️ Ba chỗ chữ ký chưa phủ hết — dev phải quay lại hỏi, đừng tự lấp

1. **Giá trị màu của token `draft` (Quyết định #2).** Ice ký *đường*, chưa ký *số*. Token phải qua `check:tokens` Kiểm C *(mọi cặp mới khai vào `contrast.pairs` hoặc `contrast.excluded`)*. **Đề xuất khởi điểm để Ice bác hoặc gật**, không phải một giá trị đã chốt: lấy chính `ornament` của mỗi theme *(`#a9a196` sáng / `#6a6459` tối)* làm giá trị của `draft` — cùng sắc, khác **tên**, nên vạch `draft` đọc yếu hơn `confirmed` đúng như bản dựng muốn, mà **không** cần `opacity` và **không** cần một miễn trừ nào. ⇒ Token thứ 17 lúc đó là một **cái tên mới cho một màu đã kiểm**, không phải một màu mới chưa ai đo. 🔴 Vẫn phải xác nhận với Ice trước khi ghi vào `tokens.json`.
2. **Hình dạng neo `data-segment-id` (Quyết định #1, vế hai).** Story mới **gợi ý** `data-col="src" | "tgt"`; Ice duyệt cả gói nên nó coi như đã ký, nhưng nó là phần **ít bằng chứng nhất** của lượt này. Nếu Task 1 lộ ra một hình dạng tốt hơn thì đổi và ghi lý do — hai spec e2e sửa cùng lượt (B11).
3. **Quyết định #6 ký trước phép đo.** Ghi lại thẳng ở đây thay vì để nó trôi: nếu Task 1.2 cho thấy `contenteditable` trần **không** đặt được caret trong ô rỗng trên WKWebView, thì **LUẬT DỪNG của §Điều kiện khởi hành thắng chữ ký này** — dừng, báo Ice, đừng tự đi tìm một thư viện.

🔴 Nếu Ice ký một đường mà phép đo đã bác, ghi rõ *"Ice ký SAU KHI đọc số bác nó"* và ghi luôn
đường dev chọn để **ít nói dối nhất** — khuôn Quyết định #2 của Story 2.5.

### Debug Log References

#### Ⓐ Baseline — ĐO LẠI trước khi sửa dòng đầu tiên (2026-08-14)

*"Baseline lấy bằng ĐO, không bằng phép trừ"* — §Bài học. Số ghi sẵn trong story **khớp**:

| | Story ghi | Đo lại | |
|---|---|---|---|
| `cargo test --locked` | 338 / 0 / 5 | **338 / 0 / 5** | khớp |
| `npm run test` (vitest) | 78 / 78 | **78 / 78** *(9 tệp, 2,30 s)* | khớp |

#### Ⓑ Task 1 — năm mệnh đề, hai engine, hai đường đo (2026-08-14)

Bảng đầy đủ + giới hạn: `2-5b-ban-do/README.md`. Tóm tắt:

| # | Mệnh đề | WKWebView **605.1.15** | Blink (Playwright 1.62.1) |
|---|---|---|---|
| ① | chuột thật vào ô **rỗng** ⇒ caret | `contenteditable` **trần: KHÔNG** *(`activeElement = SECTION.mode`, `selection = None`, `rangeCount 0`, **0** `focusin`; cú bấm **có** trúng ô)* · **có đường chuột sản phẩm: ĐẠT** *(`Caret`, thứ tự `mousedown → mouseup → focusin → selectionchange → click`)* | ĐẠT |
| ② | gõ một ký tự | **ĐẠT** — `beforeinput insertText` **huỷ được**, chữ hạ cánh | ĐẠT |
| ③ | `Backspace` offset 0 ⇒ `deleteContentBackward` | 🔴 **KHÔNG** — **0** `beforeinput`, cả ở đầu ô **có chữ** lẫn ô **đã rỗng** | ✅ **CÓ**, huỷ được |
| ④ | *"sập hố"* | **ĐẠT** — ô rỗng **38,00 px** = ô có chữ **38,00 px** | ĐẠT — **38,81** = **38,81** |
| ⑤ | `subgrid` giữ hàng thẳng | **ĐẠT** — lệch `top` lớn nhất **0 px** | **ĐẠT** — **0 px** |

🔴 **Ba kết luận:**
1. **`contenteditable` trần KHÔNG đủ trên WKWebView.** Mỗi ô là editing host riêng **không**
   miễn cho đường chuột — cùng họ Story 1.22-C2 *(`<button>`)* và 2.3 *(`<span>`)*.
   ⇒ `GridPanel.vue` giữ nguyên ba mảnh đã thắng: `setPosition` *(không `addRange`)* · caret ở
   `mouseup` *(không `mousedown`)* · một lượt vá ở frame kế tiếp.
2. **Quyết định #6 đứng**, và phạm vi chữ ký nay ghi rõ: nó đứng cho *"`contenteditable` trần
   **+ khuôn `EditorPanel.vue` đã chạy**"*, **không** cho *"không handler nào"*.
3. 🔴 **③ lật một tiền đề Task 0 đã viết ra bằng chữ.** Quyết định #3 khai *"`Backspace` ở
   offset 0 sinh một `beforeinput` `deleteContentBackward` bắt được ⇒ Story 2.9 có tiền đề"*.
   **Trên WebKit đường đó KHÔNG tồn tại** *(cả WKWebView lẫn Playwright-WebKit, cả phím vật lý
   lẫn `execCommand`)*. 2.5b **không** bị chặn — nó chỉ dựng tiền đề *(ô là host riêng)* — nhưng
   Story 2.9 phải đi `keydown` **kèm chốt `event.isComposing`**. Nợ có chủ, đã ghi.

#### Ⓒ Ba vòng chẩn đoán của Task 1 — vòng ba CHỐT ĐƯỢC, không bị bác

Luật dừng *(«ba vòng bị bác ⇒ DỪNG»)* **không** kích hoạt: vòng ba tìm ra nguyên nhân.

| Vòng | Giả thuyết | Phán quyết |
|---|---|---|
| 1 | `user-select: none` kế thừa chặn vùng chọn | **BÁC** — `grep` toàn cây: chỗ duy nhất là `<rt>` của `SourceHanViet.vue:919` |
| 2 | cú bấm không trúng ô | **BÁC bằng số** — `mousedown`/`mouseup`/`click` đều target `DIV.cell.empty[col=tgt id=3]`, `elementFromPoint` giữa ô trả về chính ô |
| 3 | phiên nối vào **webview của một ứng dụng khác** | 🔴 **CHỐT** — `activeElement = BUTTON.sidebar-folder-tree__chevron`, một lớp CSS **0 kết quả** trong toàn kho. `lsof -nP -iTCP:4445 -sTCP:LISTEN` ⇒ `gdrive-su` PID 19811 giữ cổng. Máy chủ nhúng **bám cổng cố định 4445** *(`wdio.conf.mjs`)*. Đường ra: `TAURI_WEBDRIVER_PORT=4467` *(cả hai phía đọc biến này — `getEmbeddedPort` + crate `:24`)*, **không** giết tiến trình của Ice |

🔴 **Lỗ hổng hạ tầng lộ ra ở vòng 3, và nó rộng hơn story này:** bộ e2e thường trực **không
có** phép tự kiểm danh tính phiên. Một lượt chạy trong webview của ứng dụng khác **vẫn trả về
số** và **không ca nào đỏ vì lý do đúng** — cùng hạng với hình dạng hỏng mà `onComplete` của
`wdio.conf.mjs` đã dựng hàng rào *(«mọi ca vẫn xanh, vì một kho thật cũng là một kho mở
được»)*. Nợ có chủ, ghi ở `deferred-work.md`.

#### Ⓓ Task 2 đã VIẾT rồi HOÀN NGUYÊN — Ice chốt 2026-08-14, và lý do phải đọc được

Task 2 *(bốn panel → ba)* được viết trọn trên **7 tệp nguồn** rồi **hoàn nguyên** cùng phiên.
Ghi ra thay vì để lượt sau tưởng nó chưa từng chạy.

**Vì sao hoàn nguyên:** `npm run build` **xanh** với Task 2 trên cây — `PANEL_COMPONENTS` là
`Record<PanelId, string>` và bảng `components` của `WorkspaceDock.vue` là
`Record<string, VueComponent>`, **không có mối nối kiểu nào giữa hai bảng**. Nên một
`PANEL_COMPONENTS['panel.grid'] = 'grid'` trỏ vào một component **chưa tồn tại** đi qua
`vue-tsc` sạch, và biểu hiện là Workspace hiện **panel trắng** kèm đúng một `console.error`
của chính dockview.

⚠️ Đó là *"trông xong mà không dùng được"* — đúng lớp lỗi mà LUẬT DỪNG của story này tồn tại
để chặn, chỉ tới từ hướng khác. Ice chốt: **hoàn nguyên, giữ trọn sổ đo.**

🔴 **Và đây là một khoảng mù ĐO ĐƯỢC, không một chi tiết cài đặt:** `WorkspaceDock.vue:82-84`
đã ghi sẵn *"`PANEL_COMPONENTS` và map này phải khớp nhau, và **không cổng nào canh điều
đó**"*. Lượt này là lần đầu mệnh đề đó được **nghiệm** thay vì được ghi. Món nợ có chủ đã ở
`deferred-work.md`.

**Bảy tệp đã sửa, và story đã đặc tả từng dòng phải sửa lại** *(§Task 2 · §Bản đồ tệp sẽ SỬA)*:
`src/layout/workspaceLayout.ts` *(`PanelId` 4→3 · `PANEL_IDS` · `PANEL_TITLE_KEYS` ·
`PANEL_COMPONENTS` · hai preset Ⓑ-1/Ⓑ-2 · `NEVER_SACRIFICED` · bảng 🔵 "tên là lịch sử")* ·
`src/commands/index.ts` *(`FOCUS_OWNERS` 7→6 · `PANEL_SUFFIXES` 4→3)* ·
`scripts/check-layout.mjs` *(ba chuỗi id cứng ở Kiểm A mệnh đề 3 + hai chuỗi ở phép nghịch đảo)*
· `src/layout/dockController.ts` · `src/modes/WorkspaceMode.vue` · `src/panels/PanelFrame.vue` ·
`src/commands/focus.ts` *(bốn chú thích mang id đã chết)*.

⚠️ **Một số ĐO ĐƯỢC trong lượt đó, và nó bác một mệnh đề của Task 5.4:** story viết
*"`SELECTION_SURFACE_FLOOR = 7` **không đổi** (hai lời gọi thay hai lời gọi)"*. Đếm thật:
hôm nay có **bảy** lời gọi `useSelectionSurface` *(Attribution · Shortcuts · SourcePanel ·
**SourceHanViet** · AiTranslation · EditorPanel · Lookup)*. Lưới thay **ba** trong số đó bằng
**hai** ⇒ **6**, không 7. Sàn phải hạ xuống **6** kèm lý do viết ra — đó là một lượt **đếm lại
quần thể sau một lượt gộp cấu trúc**, không một lượt nới cổng cho mã đi lọt.

#### Ⓔ AC3 từng chặn ở **cú bấm ĐẦU TIÊN** — ✅ **ĐÃ ĐÓNG 2026-08-15, Ice ký đường (A)**

Task 12.2 dựng ca nghiệm thu thường trực cho AC3 và nó **ĐỎ trên WKWebView thật**. Nguyên nhân
**đã chẩn đoán xong**, và nó nằm **ngoài phạm vi story này**.

**Nguyên nhân, đọc được từ mã chứ không suy đoán:** `WorkspaceDock.vue:591-611` nghe
`onDidActivePanelChange` và, với `origin === 'user'`, gọi `enterFocus(id)`;
`focus.ts::enter()` chạy `el.focus()` **vô điều kiện** trên gốc panel — kể cả khi tiêu điểm
**đã** nằm trong panel đó. Cú bấm **đầu tiên** vào lưới kích hoạt panel ⇒ lượt dời ấy chạy
**sau** `mouseup` và **sau** cả hai lượt vá của `ensureCaretNextFrame`.

| Cùng một ô, chuột thật | `activeElement` | `selection.type` |
|---|---|---|
| cú bấm **thứ nhất** | `SECTION.panel.focused` | **`"None"`**, `rangeCount 0` |
| cú bấm **thứ hai** | `DIV` *(chính ô)* | **`"Caret"`** |

⇒ Khuyết tật gói gọn ở **cú bấm đầu tiên vào panel**; mọi cú sau đều ăn.

**Bốn lượt vá đã thử và BỊ BÁC BẰNG PHÉP ĐO** *(ghi ra để không ai đi lại)*: ①
`contenteditable` trần → ② `cell.focus()` trong `mouseup` → ③ `requestAnimationFrame` → ④ thêm
một `setTimeout(0)`. Cả bốn chạy **trước** lượt `enterFocus`. Ba vòng đầu là ba vòng chẩn đoán
mà §Điều kiện khởi hành đếm ⇒ **DỪNG, báo Ice.**

🔴 **Hai đường sửa, mỗi đường một điều kiện — và cả hai chạm hợp đồng tiêu điểm, nên Ice chốt:**
- **(A)** `focus.ts::enter()` bỏ qua khi `el.contains(document.activeElement)`. AD-34 §2 nói
  *"CHUYỂN panel phải dời focus DOM tường minh"* — tiêu điểm đã ở trong panel thì **không có
  lượt chuyển nào**. Một chỗ sửa, phủ **cả sáu** điểm vào focus.
- **(B)** Chỗ gọi ở `WorkspaceDock` tự kiểm cùng điều kiện. Bán kính hẹp hơn, nhưng **không**
  đóng ca tương tự cho panel khác.

✅ **ICE KÝ ĐƯỜNG (A) — 2026-08-15.** `focus.ts::enter()` bỏ qua lượt `focus()` khi
`el.contains(document.activeElement)`. AD-34 §2 **không sửa một chữ**; điều kiện chỉ đọc nó cho
đúng — *"CHUYỂN panel"* không xảy ra khi tiêu điểm đã ở trong panel.

**Nghiệm thu:** `grid-empty-cell.e2e.mjs` **XANH** trên WKWebView thật, và **cả bộ e2e 7/7
xanh** — một lượt sửa hợp đồng dùng chung **không** làm đỏ điểm vào focus nào khác.

🔴 **Và một giả thuyết hợp lý bị PHÉP ĐO bác ngay sau đó:** lượt vá `setTimeout(…, 0)` của
`ensureCaretNextFrame` trông như mã chết sau bản vá — nó được thêm vào **chỉ để** đấu với
`enterFocus`. Gỡ ⇒ ca e2e **đỏ trở lại**; trả lại ⇒ **xanh**. ⇒ Còn **một** nguồn thu vùng chọn
nữa, chạy ngoài vòng `requestAnimationFrame`, và nó **chưa được đặt tên** — ghi đúng mức độ
chắc chắn thay vì gán cho một nguyên nhân nghe hợp lý.

⚠️ **Đừng vá bằng một lượt thứ năm trong `GridPanel.vue`.** Story 2.3 đã trả giá cho đúng lớp
lỗi này — nó chẩn đoán *"AD-34 giành tiêu điểm"* khi **không ai giành cả**. Lần này phép đo nói
**có**, và nói đích danh dòng nào; đó là lý do lượt sửa phải đi qua chữ ký chứ không qua một
bản vá tại chỗ.

⚠️ **Bàn đo Task 1.2 nói THIẾU, và giới hạn đó nay đo được:** nó dựng đúng hình dạng lưới trong
một lớp phủ **không có tổ tiên focus được**, nên nó bỏ lọt đúng biến quyết định. Cùng lớp bài
học *"trúng tiền đề chưa phải trúng cơ chế"* — lần này bàn đo trúng tiền đề, và **e2e trong
WKWebView thật** bắt được cơ chế. Đã ghi vào `2-5b-ban-do/README.md`.

#### Ⓕ Task 7.3 và Task 8 — hai phép đo, và **cả hai cho số xấu** (2026-08-15)

WKWebView **605.1.15**, macOS 15.6, bản dựng thật, `2-5b-ban-do/do-hang-va-hieu-nang.e2e.mjs`.
Bảng đầy đủ + giới hạn: `deferred-work.md`.

**Task 7.3 — chiều cao hàng khi bật Hán Việt.** Ước lượng của `epics.md:2329` *(cột ~330 px ⇒
**6–7 dòng**)* bị **cả hai vế** vượt qua:

| Kiểu xem | Cao hàng | Số dòng |
|---|---|---|
| Nguyên văn | 137 px | 4,05 |
| Hán Việt — *chuyển đổi* | 228 px | 6,74 |
| Hán Việt — **song song** | **388 px** | **11,47** |

Cột thật của Ⓑ-2 chỉ **238,5 px**, hẹp hơn con số ước ~330 px — đó là lý do số xấu hơn.
🔴 Và một cái giá **nợ gốc không nêu**: `subgrid` giữ hàng thẳng nên **ô bản dịch cũng cao
388 px**, tức một câu **chưa dịch** hiện ra thành một ô rỗng cao gần **12 dòng**.
⚠️ Vế **Blink chưa đo** — chỉ tới được qua WebView2/Windows, và dự án **không có đường nghiệm
thu tại chỗ** cho nửa đó. Khoảng mù **có tên**.

**Task 8 — chi phí lưới, giao cho Story 2.4:**

| | 2.000 câu | **9.850 câu** *(mốc cũ)* |
|---|---|---|
| node DOM | 10.005 | **49.256** |
| `selectionchange` + 2 frame | 12 / 34 / 34 ms | 24 / 33 / 33 ms |
| **dời con trỏ** | 161–226 ms | 🔴 **706–770 ms** |

⇒ Đường **dời con trỏ vượt trần 50 ms/frame của NFR2 khoảng 15 lần** ở 9.850 câu — và đó là
đường **thường nhất** của tính năng.

🔴 **Mốc cũ `:2113-2129` mất hiệu lực THEO CẤU TRÚC, không bị đóng**: nó đo *"dựng 9.850
`<span>`"*, còn lưới dựng **49.256 node** trong năm cột. Hai số **không so được**; ghi chúng
cạnh nhau như một lượt cải thiện là nói dối. `:2198-2207` thì **còn hiệu lực về cơ chế** —
lưới vẫn tính lại `ruleById` trên **toàn** danh sách mỗi lượt dời con trỏ.

🔴 **KHÔNG tối ưu mù** *(Quyết định #7)*: số được **báo**, không vá vội. Ba đường đã thấy — ảo
hoá hàng · tính `rule` tại chỗ · tách lớp `editing` khỏi lượt tính lại — **chưa đường nào được đo**.

⚠️ Một khuyết tật của **bàn đo** đã bắt: lượt đo đầu ở 9.850 hàng cho **26.927 ms**, rồi 33 ms
ở hai lượt kế — con số đó là **lượt bố cục lần đầu** của 49.256 node, không phải chi phí thao
tác. `waitForExist` trả về ngay khi ô ĐẦU TIÊN có mặt. Bản sửa chờ nhịp frame rẻ rồi mới bấm
đồng hồ, và nó cũng phải **thăm dò từ phía driver** — một kịch bản async dài trong trang **hết
giờ ở tầng WebDriver**, tức phép đo chết trước khi trả về số.

### Completion Notes List

**Phiên 2026-08-14 → 2026-08-15. Story chuyển sang `review`.**

✅ **MỌI TASK ĐÃ TICK.** Món cuối — **Task 1.4**, gõ tiếng Việt **bằng bộ gõ** trên máy thật —
**Ice xác nhận ĐẠT ngày 2026-08-15**. Đây là bất biến mà §Bất biến của story gọi là *"tệ nhất"*:
không đường nghiệm thu nào của dự án mô phỏng được một bộ gõ tiếng Việt thật, nên chữ ký của Ice
**là** đường nghiệm thu duy nhất — không cổng nào, không ca e2e nào thay được.

✅ **Đã xong và nghiệm thu:**
- **Task 0.4** — ba chỗ chữ ký chưa phủ hết, **cả ba đóng** *(token `draft` mượn số của
  `ornament` ⇒ **0 cặp tương phản mới**; neo giữ `data-col="src"|"tgt"`; luật dừng không kích
  hoạt ở Task 1)*.
- **Task 1** *(CỬA CHẶN)* — mở, trừ **1.4 là món của Ice**. Năm mệnh đề, hai engine, hai đường đo.
- **Task 2** — bốn panel → **ba**. `PanelId`/`PANEL_IDS`/`PANEL_TITLE_KEYS`/`PANEL_COMPONENTS` ·
  hai preset **Ⓑ-1/Ⓑ-2** · `NEVER_SACRIFICED` một phần tử · `FOCUS_OWNERS` 7→6 ·
  `PANEL_SUFFIXES` 4→3 · năm chuỗi id cứng trong `check-layout.mjs`. 🔴 **Preset id GIỮ TÊN,
  ĐỔI NGHĨA** — phím tắt đã gán và bố cục đã lưu sống nguyên.
- **Task 3** — `GridPanel.vue` *(mới)*: lưới **chủ-cột `subgrid`**, năm cột, ô bản dịch là
  editing host riêng, khoảng thở đoạn **đọc** `is_paragraph_end`, dải tab Hán Việt. Gỡ
  `SourcePanel.vue` + `EditorPanel.vue`.
- **Task 4** — vạch **năm → sáu**: token `draft` *(thứ 17)*, nhánh `draft` đúng thứ tự ưu tiên,
  ba khối doc-comment hết đúng **sửa tại chỗ kèm hai lượt ký theo thứ tự**, Kiểm I sáu giá trị,
  cột nhãn trạng thái sáu nhãn.
- **Task 5** — hợp đồng vùng chọn **theo CỘT**: hai lời gọi, `selectionContract.ts` **không sửa
  một dòng**. Hán Việt nhượng lượt đăng ký qua `hanVietSurfaces.ts` *(module mới)*.
- **Task 6** — `⌘Enter` **tái dùng**, `editor.next_untranslated` *(`⌥↓`)* mới, `Enter` trơn
  chặn hai lớp kèm chốt `isComposing`.
- **Task 9** — `main.ts` **đọc** `ConfirmResult`; cột nhãn trạng thái là bề mặt báo lỗi.
- **Task 10.1** — gỡ `editorGutter.ts` + test của nó; phép đo đã ở trong sổ.
- **Task 11** — vitest **83/83** *(+`segmentNavigation.test.ts`; `editorTypingZone` chuyển sang
  `GridPanel` và **ba mệnh đề bị lật** được ghi ra thay vì xoá)*.
- **Task 12** — hai spec e2e sửa theo neo mới; spec AC3 mới **đang ĐỎ có chủ**.

✅ **ĐÃ ĐÓNG — Ice ký đường (A) ngày 2026-08-15.** `focus.ts::enter()` bỏ qua lượt `focus()`
khi tiêu điểm đã ở trong owner. Bộ e2e **7/7 xanh**, gồm ca AC3 mới. §Debug Log Ⓔ.

✅ **Task 12 xong và XANH** — cộng **ba** phát hiện của lượt chạy cả bộ, cả ba đã ghi sổ có chủ:
command id nằm cứng trong spec *(không cổng nào canh `e2e/**`)* · fixture không reset state panel
*(spec sau đọc Tác phẩm của spec trước)* · lượt vá macrotask **không** phải mã chết.

🔴 **CÒN MỘT MÓN CHO ICE — Task 1.4:** gõ tiếng Việt **bằng bộ gõ** trên máy thật.

✅ **Task 7 · Task 8 xong (2026-08-15)** — hai phép đo, **cả hai cho số xấu**, cả hai **giao
lại** thay vì tự chấm: chiều cao hàng Hán Việt song song **388 px / 11,5 dòng** *(ước lượng nói
~330 px / 6–7 dòng)* · dời con trỏ **706–770 ms** ở 9.850 câu, **vượt trần NFR2 ~15 lần**.
§Debug Log Ⓕ.

✅ **Task 10.2 · Task 13 · Task 14 xong (2026-08-15).**
- **10.2** — nợ *"từ 8 làn trở lên máng 22px hết chỗ"* đóng **theo CẤU TRÚC**, và cách đóng được
  ghi rõ vì hai cách **không** đổi cho nhau được: không ai nới `gutter-width`, không ai sửa
  thuật toán. **Khái niệm "làn" thôi tồn tại** — một câu một hàng ⇒ hai vạch không bao giờ còn
  trùng `top`. Kèm **điều kiện để mục ở lại đóng**: một câu vẫn là một hàng.
- **13** — quét toàn cây: **79** chỗ nhắc `EditorPanel.vue`/`SourcePanel.vue`/*"bốn panel"*/
  `editorGutter`. Sửa **51** *(mệnh đề về CƠ CHẾ HIỆN TẠI)*; **28** còn lại là **lời kể lịch
  sử** hoặc chính dấu 🔵 của lượt này — giữ nguyên, đúng luật *"đừng xoá dấu vết quyết định
  cũ"*. Ba `README.md` cập nhật, gồm bảng *"Ranh giới sở hữu"* vốn đang trễ tiến độ.
- **14** — bảng nghiệm thu ở ngay dưới.

**Nghiệm thu CUỐI 2026-08-15 (Task 14):** **11/11** cổng npm **xanh** — gồm cả
`check:scope` và `check:scope:bundled` *(chạy tay, cổng 1420 trống)* ·
`npm run build` **xanh** · vitest **83/83** · `cargo test --locked` **338/0/5**
*(bằng baseline — Rust **không chạm một dòng**)* · **e2e**: bốn lượt cả bộ trong ngày, **3 lượt
7/7**; lượt thứ tư đỏ **một** ca — `attribution-focus`, đúng ca mà `wdio.conf.mjs:68-78` đã ghi
là chập chờn từ 2026-08-12 và nói *"CHƯA chẩn đoán, **nguyên văn lỗi không kịp bắt**"*.
🔵 **Lượt này bắt được nguyên văn** *(fixture hết giờ chờ `[data-attribution-open]`, không một
khẳng định nào về sản phẩm)*, và spec **xanh** khi chạy lại một mình ngay sau đó. Đã vào sổ.

**Nghiệm thu 2026-08-15 (giữa story):** 9 cổng npm **xanh** · `npm run build` **xanh** · vitest **83/83** ·
`cargo test --locked` **338/0/5** *(bằng baseline — Rust **không chạm một dòng**, đúng như story
đòi)* · **bộ e2e 7/7 xanh**, **hai lượt cả-bộ liên tiếp** *(7m52 và 9m15)* trên WKWebView 605.1.15
thật, `TAURI_WEBDRIVER_PORT=4467`.

⚠️ **Hai lượt xanh KHÔNG chứng minh bộ đã hết chập chờn** — `wdio.conf.mjs:68-78` ghi đúng rằng
một cỡ mẫu như thế đã lừa một lần rồi *(lượt chốt C3 kết luận "ổn định" trên n=2, và tám lượt
tính tới hôm đó là 6 xanh · 2 đỏ)*. Ghi n=2 là ghi **một sự kiện**, không một tính chất.
⚠️ `check:scope`/`check:scope:bundled` **chưa chạy** *(cần cổng 1420 trống)*.

⚠️ **Cách chạy phải đi kèm mọi con số e2e** *(Task 12.4)*: các số trên là lượt chạy **CẢ BỘ**.
Spec AC3 cũng xanh khi chạy **một mình** *(ba lượt)*. 🔴 Và `TAURI_WEBDRIVER_PORT=4467` là **bắt
buộc trên máy này** — cổng mặc định 4445 đang bị một tiến trình khác giữ; xem `2-5b-ban-do/README.md`.

### File List

**Mới:** `src/panels/GridPanel.vue` · `src/panels/hanVietSurfaces.ts` ·
`src/panels/segmentNavigation.ts` · `tests/frontend/segmentNavigation.test.ts` ·
`e2e/specs/grid-empty-cell.e2e.mjs` · `_bmad-output/implementation-artifacts/2-5b-ban-do-luoi.html` ·
`_bmad-output/implementation-artifacts/2-5b-ban-do/{chup.mjs,luoi-wkwebview.e2e.mjs,README.md,bao-cao.json,*.png}`

**Gỡ:** `src/panels/SourcePanel.vue` · `src/panels/EditorPanel.vue` · `src/panels/editorGutter.ts` ·
`tests/frontend/editorGutterLanes.test.ts`

**Sửa:** `src/layout/workspaceLayout.ts` · `src/layout/WorkspaceDock.vue` ·
`src/layout/dockController.ts` · `src/commands/index.ts` · `src/commands/focus.ts` ·
`src/main.ts` · `src/modes/WorkspaceMode.vue` · `src/panels/PanelFrame.vue` ·
`src/panels/SourceHanViet.vue` · `src/panels/editorSegments.ts` ·
`src/panels/editorPanelState.ts` · `src/i18n/vi.json` · `e2e/specs/shortcuts-capture-mouse.e2e.mjs` · `e2e/wdio.conf.mjs` ·
`src/tokens/tokens.json` · `scripts/check-commands.mjs` · `scripts/check-layout.mjs` ·
`scripts/check-tokens.mjs` · `tests/frontend/editorSegmentRule.test.ts` ·
`tests/frontend/editorTypingZone.test.ts` · `tests/frontend/editorAutoLookup.test.ts` ·
`tests/frontend/editorFlush.test.ts` · `e2e/specs/editor-typing-flush.e2e.mjs` ·
`e2e/specs/editor-confirm-segment.e2e.mjs` ·
`_bmad-output/implementation-artifacts/{deferred-work.md,sprint-status.yaml}`

🔵 **`src/panels/editorFlush.ts` GỠ khỏi danh sách này 2026-08-15 (code review).** Nó được kê
vào mục *"Sửa"* nhưng `git diff f990dd5..HEAD -- src/panels/editorFlush.ts` **trống** — tệp không
đổi một dòng. Việc **không** sửa nó mới là đúng *(§Đừng dựng lại: "Tái dùng. Đừng dựng lịch thứ
hai… đừng sửa `createWriteSchedule`")*; chỉ có bảng kê khai là sai. ⚠️ Một `File List` kê thừa
không vô hại: nó là thứ lượt rà sau dùng để chọn phạm vi đọc, nên một tên thừa mua thời gian đọc
một tệp không đổi, và — tệ hơn — nó làm người đọc tin rằng hợp đồng flush đã được xem lại ở story
này.

**Bổ sung vào mục *"Sửa"* ở lượt code review 2026-08-15** *(mười ba bản vá — xem §Review
Findings)*: `src/StatusBar.vue` · `src/panels/GridPanel.vue` · `src/panels/README.md` ·
`src/panels/selectionContract.ts` *(hoàn nguyên về **0 dòng sửa**, AC7)* ·
`tests/frontend/statusBar.test.ts` *(sáu ca mới, nhóm ⑤ — vitest 83 → **89**)*.

### Change Log

| Ngày | Nội dung |
|---|---|
| 2026-08-15 | **Rà mã ba tầng ⇒ Status `done`.** 13 phát hiện · 13 bản vá · 0 hoãn · 0 loại làm nhiễu. Ice ký **5 quyết định**. 🔴 Khuôn lặp lại **ba lần** và là phát hiện thật của lượt rà: chữ ký #5(a) · #8 · #9(a) đều được thi hành **đúng một nửa** — nửa khó, có chú thích 🔵 đẹp thì làm; nửa là **một dòng chuỗi hoặc một câu phải xoá** thì rơi, và **không cổng nào canh nửa đó**. Nghiệm thu: 11 cổng · `vue-tsc` · `eslint` · build · vitest **83 → 89** · cargo 338/0/5. AC7 nay **xanh thật** (`git diff f990dd5 -- selectionContract.ts` = **0 dòng**; trước lượt vá là 2). |
| 2026-08-15 | **Hai vế không nghiệm thu được bằng cổng, và cả hai đã đóng bằng ĐO.** ② Đường ra thanh trạng thái: **6 ca vitest**, tự kiểm đỏ **5/6** *(ca thứ sáu ở lại xanh vì nó khẳng định đúng `null` — đỏ được VÀ không đỏ oan)*. ① Guard `caretTarget`: `grid-empty-cell.e2e.mjs` trên WKWebView **605.1.15**, **8 lượt — 5 đạt/3 đỏ**, và 🔴 **không lượt đỏ nào rơi vào phép khẳng định caret** *(cả hai lượt có chi tiết đều chặn ở đường khởi động/fixture)*. ⚠️ Đo được *"guard không phá"*, **không** đo được *"guard chữa được cuộc đua"* — cuộc đua cần hai cú bấm trong một khung hình, không spec nào làm thế; nợ mức thấp, đã ghi. |
| 2026-08-15 | 🔴 **Bộ e2e từng KHÔNG chạy được, và nguyên nhân là một khuyết tật của bàn đo — đã đo, không đoán.** Lượt cả bộ đầu tiên: **7/7 đỏ**, mọi spec chặn tại `openWorkspaceWithWork` vì *"không có cầu IPC"*. Nguyên nhân: `wdio.conf.mjs::devServerIsUp()` hỏi **một** câu (`fetch('/')` có `res.ok` không) rồi tin rằng có người phục vụ — một Vite **hấp hối** vẫn trả `200`. Chạy lại với cổng 1420 **trống** ⇒ app lên, phiên WebKit thiết lập được. ⚠️ Chính `wdio.conf.mjs:186` đã **tiên đoán** hình dạng này bằng chữ. ⚠️ **Hai thông báo lỗi to và tự tin trỏ nhầm chỗ trong cùng một ngày** — `cargo install tauri-driver` *(sai nền tảng: `driverProvider: 'embedded'` là đường duy nhất trên macOS, `wdio.conf.mjs:351-355`)* và *"kiểm `dict/*.db`"* *(cả bốn tệp có đủ)*. Nợ có chủ trong `deferred-work.md`. |
| 2026-08-14 | Tạo story bằng `create-story`. Baseline `f990dd5`, cây sạch. |
| 2026-08-15 | ✅ **Ice xác nhận Task 1.4 ĐẠT** — gõ tiếng Việt bằng bộ gõ trên máy thật, chữ hạ cánh, dấu không rơi. Cửa chặn của story đóng trọn; **mọi task đã tick**. |
| 2026-08-15 | **Task 10.2 · 13 · 14 xong ⇒ Status `review`.** Nợ *"8 làn"* đóng **theo cấu trúc** *(khái niệm "làn" thôi tồn tại — không ai nới `gutter-width`)*. Quét tài liệu: **79** chỗ, sửa **51**, giữ **28** là lịch sử. `DESIGN.md` 16 → **17** token và hai mệnh đề *"cách duy nhất"* / *"không chia khối"* sửa tại chỗ; `EXPERIENCE.md` năm → sáu. Nợ `err.segment.*` đóng **🟡 một nửa**, phần còn hở ghi đủ. Nghiệm thu: **11/11** cổng · build · vitest 83/83 · cargo 338/0/5 · e2e **3/4 lượt cả bộ 7/7**. 🔵 Bắt được **nguyên văn** lượt đỏ chập chờn `attribution-focus` mà `wdio.conf.mjs` ghi là chưa chẩn đoán từ 2026-08-12. |
| 2026-08-15 | **Task 7 · Task 8 — hai phép đo, cả hai cho số xấu và cả hai được GIAO LẠI.** Hán Việt song song: **388 px / 11,47 dòng** ở cột **238,5 px** của Ⓑ-2 — ước lượng của `epics.md:2329` thấp ở **cả hai** vế, và `subgrid` kéo theo **ô bản dịch cũng cao 388 px**. Hiệu năng: **49.256 node** ở 9.850 câu, dời con trỏ **706–770 ms** ⇒ **vượt trần NFR2 ~15 lần**; mốc cũ `:2113-2129` **mất hiệu lực theo cấu trúc**, `:2198-2207` **còn hiệu lực về cơ chế**. Không tối ưu mù (Quyết định #7). §Debug Log Ⓕ. |
| 2026-08-15 | **Ice ký đường (A)** — `focus.ts::enter()` bỏ qua `focus()` khi tiêu điểm đã ở trong owner. AC3 **XANH** trên WKWebView thật, **cả bộ e2e 7/7**. Ba phát hiện phụ của lượt chạy cả bộ vào sổ có chủ. Một giả thuyết *"mã chết"* bị phép đo bác. |
| 2026-08-15 | **Task 2–6 · 9 · 10.1 · 11 · 12 xong.** Lưới `subgrid` năm cột dựng được; vạch năm→sáu; hợp đồng vùng chọn theo cột với `selectionContract.ts` **0 dòng sửa**; `⌥↓` mới. 9 cổng · build · vitest 83/83 · cargo 338/0/5. 🔴 **LUẬT DỪNG kích hoạt ở Task 12.2**: cú bấm ĐẦU TIÊN vào lưới mất caret vì `focus.ts::enter()` dời tiêu điểm về gốc panel vô điều kiện — bốn lượt vá bị bác bằng phép đo, hai đường sửa đều chạm AD-34 ⇒ Ice chốt *(đã ký (A) cùng ngày, hàng trên)*. §Debug Log Ⓔ. |
| 2026-08-14 | **Task 1 (CỬA CHẶN) MỞ.** Bàn đo hai nhánh, năm mệnh đề, hai engine — ①②④⑤ đạt, ③ **không** trên WebKit. Ice ký nốt ba chỗ chữ ký chưa phủ hết (Task 0.4). Hai món nợ mới có chủ vào `deferred-work.md`. Task 2 viết rồi **hoàn nguyên** (Ice chốt) vì nó làm Workspace hỏng lúc chạy trong khi mọi cổng vẫn xanh — §Debug Log Ⓓ. Cây nguồn về đúng baseline. |
| 2026-08-14 | Ice ký **trọn gói chín quyết định theo đề xuất**. Bảng chữ ký điền đủ; chín mục Task 0 mang một dòng ✅ **ĐÃ KÝ** và **giữ nguyên các đường bị loại** làm bằng chứng. Ba chỗ chữ ký **chưa phủ hết** ghi riêng thành Task 0.4 — giá trị màu token `draft`, hình dạng neo `data-segment-id`, và việc #6 được ký **trước** phép đo của Task 1. |

### Review Findings

> **Lượt rà ba tầng, 2026-08-15.** Diff `f990dd5..HEAD -- src/ scripts/` *(36 tệp, +2.068/−2.036)*.
> Ba tầng chạy song song, ngữ cảnh sạch, không tầng nào thấy kết quả tầng kia: **Blind Hunter**
> *(đối kháng chung — tầng này CHẠY THẬT 11 cổng + vue-tsc + eslint + vitest)* · **Edge Case
> Hunter** *(đi hết nhánh/biên)* · **Acceptance Auditor** *(đối chiếu 9 quyết định + 14 AC + 12 B)*.
> Không tầng nào trượt. **13 phát hiện, 0 bị loại làm nhiễu.**
>
> 🔴 Mức nghiêm trọng do ba tầng gán **đã bị bỏ hết** và chấm lại tại đây sau khi mở mã tại từng
> vị trí — ba tầng cố ý bị bịt mắt một phần nên chúng không đủ ngữ cảnh để chấm mức.
>
> ⚠️ **Phạm vi rà, ghi ra thay vì để tưởng đã phủ:** lượt này KHÔNG rà `tests/` + `e2e/`
> *(1.242 dòng)* và KHÔNG rà `_bmad-output/` + `design-artifacts/` *(3.752 dòng)*. Hai nhóm đó
> chưa có ai nhìn.
>
> 🔵 **Một chỗ tầng rà nói sai, sửa tại chỗ:** Blind Hunter mô tả `settled()` trong
> `GridPanel.vue::ensureCaretNextFrame` là *"chỉ kiểm `document.activeElement === cell`"*. Sai —
> nó kiểm **cả hai** vế (`activeElement` **và** `getSelection()?.type === 'Caret'`), đúng như
> doc-comment ngay trên khai. Phát hiện của tầng đó vẫn đứng, nhưng lập luận đã được dựng lại.

**Quyết định cần Ice chốt — 5** · ✅ **ĐÃ KÝ TRỌN GÓI 2026-08-15**, cả năm ở đường ⭐. Chữ ký ghi
ngay dưới mỗi mục; cả năm chuyển thành bản vá ⇒ tổng **13** bản vá.

- [x] [Review][Decision] **Quyết định #5(a) mới làm một nửa: nhãn bố cục đang nói dối người dùng** — Chữ ký 2026-08-14 viết nguyên văn *"Đổi `labelKey` **và chuỗi `vi.json`** cho đúng nghĩa mới"*. Nửa đầu làm rồi và làm tốt *(chú thích 🔵 ở `workspaceLayout.ts:133-156` nói đúng câu "tên là lịch sử, nghĩa ở bảng ngay dưới")*. Nửa sau **rơi**: `src/i18n/vi.json:28-29` vẫn là `"Bố cục lưới 2×2"` và `"Bố cục bốn cột"` — trong diff chúng là dòng ngữ cảnh, không phải dòng sửa; `labelKey` ở `workspaceLayout.ts:159-160` cũng không đổi. `ShortcutsOverlay.vue:282` render `{{ t(row.labelKey) }}` ⇒ **đây là chữ trên màn hình thật**. Hệ quả: người dùng bấm *"Bố cục bốn cột"* — đúng cái tên mà `workspaceLayout.ts:154` cảnh báo *"đã RÚT, đừng dựng lại nó khi đọc thấy chữ `columns`"* — và nhận Ⓑ-1. Chỗ cần Ice: **chữ hiển thị mới**. `Ⓑ-1`/`Ⓑ-2` là ký hiệu nội bộ, không đưa ra màn hình được. Đề xuất: `preset_grid` → *"Lưới bên trái"*, `preset_columns` → *"Lưới trên đỉnh"*.
      → ✅ **ĐÃ KÝ 2026-08-15 — đường (a).** `preset_grid` = **"Lưới bên trái"**, `preset_columns` = **"Lưới trên đỉnh"**. Tả **vị trí lưới**, không tả số ô — nên hai cái tên mới không thể già đi theo số cột như hai cái tên cũ đã già.
- [x] [Review][Decision] **`SELECTION_SURFACE_FLOOR` hạ xuống 6 dựa trên một phép đếm sai — quần thể tĩnh thật là 7** — `scripts/check-commands.mjs:1937-1955`. Doc-comment lý luận *"lưới thay BA lời gọi (`SourcePanel` + `SourceHanViet` + `EditorPanel`) bằng HAI ⇒ 6"*. Vế `SourceHanViet` **sai**: lời gọi của nó không biến mất khỏi mặt chữ — nó nằm trong nhánh `if (props.surfaceRole === 'own')` ở `SourceHanViet.vue:682`, và `SURFACE_CALL_RE` là **regex quét tĩnh**, không phân tích `if`. Đo được: `node scripts/check-commands.mjs` tự in **`7 bề mặt đăng ký`** *(2 nguồn · 5 hiển thị)* trong khi sàn là 6. Cổng này tự khai quy ước riêng — *"Sàn = SỐ THẬT hôm nay (AC13)"*, không phải 80–85 % — nên 6 mâu thuẫn với chính nó và để lại **đúng một đơn vị dư**: bớt một bề mặt thật về sau ⇒ cổng vẫn XANH. Đúng lớp lỗ mà doc-comment của chính cổng này dẫn chứng đã xảy ra một lần với sàn 4. **Hai đường đều hợp lệ, Ice chốt:** (a) nâng sàn về **7** và sửa lại phép đếm trong doc-comment; (b) **gỡ nhánh `'own'` chết** ở `SourceHanViet.vue:681-683` — đo được: tệp này chỉ được mount từ `GridPanel.vue:848` và **luôn** với `surface-role="cell"`, nên nhánh `'own'` chưa từng chạy dù prop mặc định là `'own'` — rồi sàn 6 mới thành đúng.
      → ✅ **ĐÃ KÝ 2026-08-15 — đường (b): gỡ nhánh chết, giữ sàn 6.** 🔴 **Hệ quả bắt buộc, ghi ra vì nó là chỗ lượt vá này có thể tự tạo ra một lỗ mới:** prop `surfaceRole` khai mặc định `'own'` (`SourceHanViet.vue:88-90`). Gỡ nhánh `'own'` mà **giữ** giá trị mặc định ⇒ một chỗ mount tương lai quên khai `surface-role` sẽ **không đăng ký bề mặt nào, im lặng**, và sàn 6 vẫn xanh vì phép đếm là **tĩnh**. ⇒ Bỏ luôn giá trị mặc định, để prop **bắt buộc** — chỗ quên thành một lỗi kiểu mà `vue-tsc` bắt, không thành một bề mặt mất tích.
- [x] [Review][Decision] **Quyết định #8 mới làm một nửa: 3/5 `ConfirmResult` không có đường ra màn hình nào** — Chữ ký khai hợp đồng UX-DR30 tối thiểu là *"cột nhãn trạng thái của **chính hàng** CỘNG **một dòng ở thanh trạng thái**"*. Vế đầu dựng rồi *(`GridPanel.vue:717-735` đọc `editorConfirmError` cho `'refused'`)*. Vế sau **không tồn tại**: `src/StatusBar.vue:31-33` chỉ `import { editorLastSavedAt }`, không biết gì về xác nhận. Hệ quả ở `src/main.ts:246-252`: `'no-caret'` · `'flush-failed'` · `'still-dirty'` chỉ đi vào một `console.warn` ⇒ người dùng bấm `⌘Enter` mà **không một pixel nào đổi**. Đây là hình dạng *"rỗng im lặng"* ở §Critical Don't-Miss Rules, áp lên một thao tác người dùng chủ động. Chỗ cần Ice: **ba chuỗi `vi.json` mới** và phán quyết kết quả nào đáng lên thanh trạng thái — story tự cảnh báo *"đừng nhân lượt ký này thành một hệ thống thông báo"*, nên phạm vi phải do Ice vạch.
      → ✅ **ĐÃ KÝ 2026-08-15 — đường (a): dựng vế thanh trạng thái cho CẢ BA.** Ba khoá `vi.json` mới, hiện ở `StatusBar.vue`. 🔴 Phạm vi đóng băng tại đây: **một dòng ở thanh trạng thái**, không hộp thoại, không lớp nổi (UX-DR16), không hàng đợi thông báo. Vế *"cột nhãn trạng thái của chính hàng"* đã dựng rồi và **không đụng vào**.
- [x] [Review][Decision] **AC7 vế *"`selectionContract.ts` không sửa một dòng"* đang ĐỎ theo mặt chữ** — `src/panels/selectionContract.ts:81,116` đổi thật, hai dòng, `"bốn panel"` → `"ba panel"`. Nội dung sửa **đúng sự thật mới** và hoàn toàn vô hại *(chỉ doc-comment, không một dòng logic)*. Nhưng AC7:92 diễn đạt bằng số không, và bảng Bất biến `:595` tự đặt phép nghiệm thu là *"`git diff` thấy ngay"* — phép ấy đang đỏ. Task 5.2 và Change Log 2026-08-15 đều tick *"`selectionContract.ts` **0 dòng sửa**"*, tức **sổ sách đang lệch cây nguồn**. **Hai đường, Ice chốt:** (a) hoàn nguyên hai dòng, chuyển câu đính chính sang `panels/README.md`, giữ AC7 đúng nguyên văn; (b) sửa AC7 tại chỗ kèm 🔵 + ngày thành *"không sửa một dòng **logic/hợp đồng**"* và ghi rõ hai dòng chú thích là ngoại lệ có tên.
      → ✅ **ĐÃ KÝ 2026-08-15 — đường (a): hoàn nguyên hai dòng.** Câu đính chính *"bốn panel → ba panel"* chuyển sang `panels/README.md`. 🔴 Lý do chọn (a) chứ không (b): AC7 là một bất biến mà phép nghiệm thu của nó — *"`git diff` thấy ngay"* — **rẻ và tuyệt đối**. Nới nó thành *"không sửa logic"* là đổi một phép kiểm máy chạy được lấy một phép kiểm người phải đọc hiểu.
- [x] [Review][Decision] **Món nợ `panel.editor.nothing_translated` (UX-DR27) mồ côi — trỏ vào một tệp và một khoá đã không còn tồn tại** — `deferred-work.md:2179-2186` còn mở, **Chủ: Ice**, và ghi chỗ lật là *"một `v-if` trong `EditorPanel.vue`, và khoá `panel.editor.nothing_translated` trong `vi.json`"*. Diff xoá **cả hai**: `EditorPanel.vue` biến mất trọn, khoá bị gỡ khỏi `vi.json`. Luật kho cấm xoá một mục nợ và bắt đóng bằng cách nối tiếp `→ ✅ ĐÃ ĐÓNG <ngày>` — ở đây mục **không được đóng, cũng không được chuyển chủ**. Chỗ cần Ice: lưới có nhãn `panel.grid.state_untranslated` trên **từng hàng**, nên ca *"đã tách, chưa câu nào có bản dịch"* có thể đã được phủ theo cấu trúc ⇒ đóng ✅; hoặc UX-DR27 vẫn đòi một câu **ở tầng panel** ⇒ đóng 🟡 và ghi phần còn hở.
      → ✅ **ĐÃ KÝ 2026-08-15 — đóng ✅, lưới đã phủ theo CẤU TRÚC.** Mỗi hàng chưa dịch mang nhãn `panel.grid.state_untranslated` ⇒ *"khung trống câm"* mà UX-DR27 cấm **thôi tồn tại được** trong hình dạng lưới, không phải bị che bởi một câu. Đóng bằng cách **nối tiếp** `→ ✅ ĐÃ ĐÓNG` vào mục cũ, **không xoá** — cùng luật đã áp cho mọi món nợ khác.

**Bản vá — 13** *(8 từ lượt rà + 5 sinh ra từ chữ ký ở trên)*

- [x] [Review][Patch] **D1** — hai chuỗi nhãn preset về đúng nghĩa: *"Lưới bên trái"* · *"Lưới trên đỉnh"* [src/i18n/vi.json:28-29]
- [x] [Review][Patch] **D2** — gỡ nhánh `'own'` chết **và** bỏ giá trị mặc định của prop `surfaceRole` [src/panels/SourceHanViet.vue:88-90,681-683]
- [x] [Review][Patch] **D3** — ba khoá `vi.json` mới + đường ra thanh trạng thái cho `'no-caret'` · `'flush-failed'` · `'still-dirty'` [src/StatusBar.vue] [src/main.ts:246-252]
- [x] [Review][Patch] **D4** — hoàn nguyên hai dòng chú thích, chuyển câu đính chính sang README [src/panels/selectionContract.ts:81,116] [src/panels/README.md]
- [x] [Review][Patch] **D5** — nối `→ ✅ ĐÃ ĐÓNG 2026-08-15` vào món nợ UX-DR27, không xoá mục [_bmad-output/implementation-artifacts/deferred-work.md:2179-2186]

- [x] [Review][Patch] 🔴 `sourceChapterError` không có một người đọc nào — lỗi nạp Chương rơi thẳng vào rỗng im lặng [src/panels/GridPanel.vue:80-118]
- [x] [Review][Patch] `resetEditorPanel()` bỏ sót `confirmError` + `caretPlacement` — nhãn *"chưa ký được"* rò sang Tác phẩm khác trùng `segment.id` [src/panels/editorPanelState.ts:347-368]
- [x] [Review][Patch] Chương 0 byte nay báo **sai lý do** — `isEmptyChapter` bị xoá cùng `SourcePanel.vue`, không ai tái lập [src/panels/GridPanel.vue:118-124]
- [x] [Review][Patch] Quyết định #9(a) mới làm một nửa: miễn trừ `⏐` đã chết vẫn được tả như đang hiệu lực [src/tokens/tokens.json:102] [scripts/check-tokens.mjs:1279]
- [x] [Review][Patch] Cổng tự in `undefined` vào chính dòng bằng chứng của nó — `SELECTION_PANEL_FILES.length` trên một object [scripts/check-commands.mjs:2083]
- [x] [Review][Patch] `ensureCaretNextFrame` có thể kéo tiêu điểm về ô cũ trong một khung hình — closure giữ `cell` cũ, không đối chiếu ô đang được nhắm [src/panels/GridPanel.vue:444-476]
- [x] [Review][Patch] `README.md` dạy sai một `owner` đã bị gỡ — ví dụ *"hôm nay"* dùng `panel.source` [src/panels/README.md:56]
- [x] [Review][Patch] `File List` kê `editorFlush.ts` vào mục *"Sửa"* nhưng `git diff` trên tệp đó **trống** [_bmad-output/implementation-artifacts/2-5b-luoi-hai-cot-doi-chieu.md:1017]

**Hoãn — 0.** Không phát hiện nào thuộc lớp *"đã có từ trước, không do lượt này gây ra"*.

## Nhật ký sprint-status

Gỡ nguyên văn từ `sprint-status.yaml` ngày 2026-08-19: tệp đó giữ TRẠNG THÁI, nội dung story thuộc về tệp này. Không sửa một ký tự.

```
  # 🔵 CORRECT COURSE 2026-08-14 (Ice duyet tron goi) — be mat nhap lat sang LUOI HAI COT
  # doi chieu. Nguon: planning-artifacts/sprint-change-proposal-2026-08-14.md
  # Ba story CHEN vao day (khuon 1.10b/1.11b/1.18b) vi 2.9 va 2.10 dua vao hinh dang —
  # luoi phai dung TRUOC chung. Danh so chen de KHONG story done nao doi dinh danh.
  # 2.5b gop panel Source + Editor thanh MOT (panel.grid) ⇒ con BA panel. No mang theo
  # AC superseded cua Story 2.2 (4/8) va Story 1.14 (AC 'bon slot panel').
  # 🔵 2026-08-14 — 2.5b chuyen sang ready-for-dev (create-story).
  # ✅ 2026-08-14 — ICE KY TRON GOI CHIN QUYET DINH cua Task 0, tat ca theo duong de xuat:
  #   #1 (b) CSS Grid chu-cot voi `subgrid` · #2 (a) token `draft` thu 17, vai `stroke` ·
  #   #3 (b) moi o cot ban dich la mot editing host rieng — VA DO LA MOT LUOT LAT Quyet dinh
  #      #1 cua Story 2.3 ("vung go la MOT cau tai mot thoi diem") ·
  #   #4 (a) go `editorGutter.ts`, giu phep do trong so · #5 (a) panel id doi, preset id GIU TEN
  #      doi NGHIA (preset_grid = Ⓑ-2 mac dinh, preset_columns = Ⓑ-1) ·
  #   #6 (a) khong thu vien editor — ⚠️ Ice ky TRUOC khi co so cua Task 1; chu ky nay KHONG mien
  #      cho Task 1, va LUAT DUNG thang chu ky neu phep do truot ·
  #   #7 (a) khong ao hoa, giao so cho Story 2.4 · #8 (a) cot nhan trang thai + thanh trang thai,
  #      VA DAY LA CHU KY HOP DONG UX-DR30 O PHAM VI TOI THIEU ·
  #   #9 (a) `on-surface-variant` cho cot so cau va cot nhan trang thai, go mien tru `⏐` da chet.
  # 🔴 BA CHO CHU KY CHUA PHU HET (Task 0.4) — dev phai quay lai hoi, dung tu lap:
  #   ① gia tri mau hai theme cua token `draft` (Ice ky DUONG, chua ky SO) — hoi truoc Task 4.4;
  #   ② hinh dang neo `data-segment-id` (`data-col="src"|"tgt"`) — xac nhan lai sau Task 1;
  #   ③ Quyet dinh #6 ky truoc phep do — neu Task 1.2 truot thi DUNG, dung di tim thu vien.
  # ── Boi canh chin quyet dinh (giu lai de story sau doc duoc ly do) ──────────────────
  # Bon cai nang nhat:
  #   #1 AC2 doi HANG, AC7 doi COT la MOT be mat dang ky — hai doi hoi nay XUNG KHAC trong DOM
  #      thuong: trong <table> mot cot KHONG co phan tu to tien, ma selectionContract duyet bang
  #      `el.contains(anchor)`. Duong de xuat: CSS Grid chu-cot voi `subgrid` (Safari 16+ /
  #      Chrome 117+), va PHAI DO tren ca hai engine — ba engine bat dong o gap va auto-sizing.
  #   #2 them 'draft' vao SEGMENT_RULE_VALUES ⇒ Kiem I doi `var(--color-draft)` ⇒ hoac mot token
  #      thu 17 (DESIGN.md:196 canh bao) hoac noi cong. Ban dung dung ornament+opacity: Kiem I DO.
  #   #3 `contenteditable` dat o dau. Day la mot luot LAT Quyet dinh #1 cua Story 2.3 (Ice da ky
  #      "mot cau tai mot thoi diem") — tien de cu khong con ton tai, phai ky lai tuong minh.
  #   #5 PresetId nam TREN DIA (ScopeKind::LayoutPreset) va command id nam trong bang keybinding
  #      (Story 1.21) ⇒ doi ten la mo coi phim tat nguoi dung da gan, IM LANG. De xuat: giu ten,
  #      doi nghia.
  # 🔴 CUA CHAN — Task 1: dung mui tham do va GO TAY vao mot cau CHUA DICH tren WKWebView that.
  #   Khong go duoc thi DUNG, bao Ice, 2.5b quay ve `backlog`. Day la mon cua Story 2.3 con treo,
  #   va gia thuyet "luoi sua nguyen nhan goc" CHUA duoc do.
  # 2.5b dong 3 mon co chu dich danh: deferred-work.md :2863-2873 (chieu cao hang Han Viet, CHUA
  #   DO) · :2875-2885 (so phan editorGutter.ts, khong xoa im lang) · :2896-2908 (thu vien editor
  #   + khuyet tat "sap ho"). Va no doc lai :2317-2371 · :2528-2584 · :2801-2816.
  # ⚠️ 2.5b KHONG tu cham NFR2 dat — no GIAO SO cho Story 2.4. Moi so hieu nang cu cua Epic 2 do
  #   tren mo hinh "N <span> trong mot dong van lien tuc": chung MAT HIEU LUC THEO CAU TRUC,
  #   khong phai bi dong.
  # ⚠️ Tang Rust KHONG sua mot dong; KHONG buoc di tru nao. So ke tiep van la 8 (Story 2.5c).
  # 🔵 2026-08-14 — 2.5b chuyen sang in-progress (dev-story). Baseline do lai truoc khi sua
  #   dong dau tien: cargo test 338/0/5 · vitest 78/78 — KHOP voi so ghi trong story.
  # ✅ TASK 1 (CUA CHAN) DA MO. Ban do hai nhanh, nam menh de, hai engine:
  #   (1) caret trong o rong — `contenteditable` TRAN: KHONG (activeElement=SECTION.mode);
  #       CO duong chuot san pham (setPosition o mouseup): DAT. => GridPanel PHAI mang duong chuot.
  #   (2) go mot ky tu: DAT · (4) "sap ho": DAT (o rong 38px = o co chu 38px) ·
  #   (5) subgrid giu hang thang: DAT, lech 0px ca hai engine.
  #   (3) Backspace offset 0: KHONG phat `beforeinput` tren WebKit (Blink CO) — bat dong engine.
  # 🔴 CON MOT MON CHO ICE: Task 1.4 — go tieng Viet BANG BO GO vao o rong tren may that.
  # 🔴 HAI MON NO MOI co chu vao deferred-work.md:
  #   - bo e2e khong tu kiem danh tinh phien (cong 4445 bi chiem => do NHAM ung dung). Chu: 1.22
  #   - Backspace offset 0 khong phat beforeinput tren WebKit => tien de cua 2.9 da LAT. Chu: 2.9
  # ⚠️ Task 2 (bon panel -> ba) DA VIET ROI HOAN NGUYEN — Ice chot. Ly do: build XANH nhung app
  #   hong luc chay (PANEL_COMPONENTS tro 'grid' ma WorkspaceDock chua co GridPanel.vue), va
  #   KHONG cong nao canh moi noi do. Chi tiet o §Dev Agent Record Ⓓ. Cay nguon = baseline.
  # ✅ 2026-08-15 — TASK 2-6 · 9 · 10.1 · 11 · 12 XONG. Luoi `subgrid` nam cot dung duoc;
  #   vach nam->sau (token `draft` thu 17, muon dung so cua `ornament` => 0 cap tuong phan moi);
  #   hop dong vung chon theo COT voi `selectionContract.ts` KHONG sua mot dong; lenh `⌥↓` moi.
  #   Nghiem thu: 9 cong npm · build · vitest 83/83 · cargo 338/0/5 (Rust khong cham mot dong).
  # 🔴 LUAT DUNG KICH HOAT LAN THU HAI o Task 12.2 — AC3 CHAN:
  #   cu bam DAU TIEN vao luoi mat caret. Nguyen nhan DA CHAN DOAN XONG:
  #   `WorkspaceDock.vue:591-611` -> `enterFocus(id)` -> `focus.ts::enter()` chay `el.focus()`
  #   VO DIEU KIEN tren goc panel, ke ca khi tieu diem DA nam trong panel do. Cu bam thu HAI an.
  #   Bon luot va bi BAC bang phep do. Hai duong sua deu cham hop dong tieu diem AD-34:
  #     (A) `focus.ts::enter()` bo qua khi `el.contains(document.activeElement)` — mot cho sua,
  #         phu ca sau diem vao focus;
  #     (B) cho goi o `WorkspaceDock` tu kiem — ban kinh hep hon, khong dong ca cho panel khac.
  #   => ICE CHOT. Dung va bang mot luot thu nam trong `GridPanel.vue`.
  #   Ca nghiem thu DANG DO CO CHU: `e2e/specs/grid-empty-cell.e2e.mjs`.
  # ⬜ Chua lam: Task 7 (do chieu cao hang Han Viet) · Task 8 (do hieu nang, giao so cho 2.4) ·
  #   Task 10.2 · Task 13 (tai lieu — con 37 cho nhac EditorPanel/SourcePanel/"bon panel") ·
  #   Task 14 (nghiem thu cuoi).
  # 🔴 CON MOT MON CHO ICE: Task 1.4 — go tieng Viet BANG BO GO tren may that.
  # ✅ 2026-08-15 — 2.5b XONG, chuyen sang `review`. Moi task tick tru DUNG MOT MON CUA ICE.
  #   Nghiem thu: 11/11 cong npm (gom check:scope + check:scope:bundled chay tay) · build ·
  #   vitest 83/83 · cargo test 338/0/5 (Rust KHONG cham mot dong) · e2e 3/4 luot ca bo 7/7.
  #   Ice ky duong (A) cho `focus.ts::enter()`: bo qua `focus()` khi tieu diem DA o trong owner.
  #   AD-34 §2 khong sua mot chu — no noi ve mot luot CHUYEN, va khong co luot chuyen nao.
  # ✅ 2026-08-15 — ICE XAC NHAN TASK 1.4 DAT: go tieng Viet BANG BO GO tren may that, chu ha
  #   canh, dau khong roi. Day la bat bien ma khong duong nghiem thu nao cua du an mo phong
  #   duoc, nen chu ky cua Ice LA duong nghiem thu duy nhat. MOI TASK DA TICK.
  # 🔴 HAI PHEP DO CHO SO XAU, ca hai DA GIAO LAI, khong tu cham dat:
  #   - Han Viet SONG SONG: hang cao 388px = 11,47 dong (uoc luong noi ~330px / 6-7 dong).
  #     Cot that cua B-2 chi 238,5px. `subgrid` keo theo O BAN DICH cung cao 388px. Chu: Ice.
  #   - Doi con tro tren 9.850 cau: 706-770 ms => VUOT TRAN NFR2 ~15 lan. Chu: Story 2.4.
  #     Moc cu :2113-2129 MAT HIEU LUC THEO CAU TRUC; :2198-2207 CON hieu luc ve co che.
  # 🔵 Nam mon no moi/da dong vao deferred-work.md — xem §Dev Agent Record cua story.
  # ✅ 2026-08-15 — RA MA BA TANG XONG => `done`. 13 phat hien · 13 ban va · 0 hoan · 0 loai.
  #   Ice ky 5 quyet dinh. Nghiem thu: 11 cong · vue-tsc · eslint · build · vitest 83 -> 89 ·
  #   cargo 338/0/5. AC7 nay XANH THAT (`git diff f990dd5 -- selectionContract.ts` = 0 dong).
  # 🔴 KHUON LAP LAI BA LAN — phat hien that cua luot ra: chu ky #5(a) · #8 · #9(a) deu duoc
  #   thi hanh DUNG MOT NUA. Nua kho, co chu thich 🔵 dep thi lam; nua la MOT DONG CHUOI hoac
  #   MOT CAU PHAI XOA thi roi. Khong cong nao canh nua do.
  #   Hau qua nang nhat da va: nhan bo cuc noi doi — nguoi dung bam "Bo cuc bon cot" (mot bo cuc
  #   DA RUT) de nhan B-1. Nay la "Luoi ben trai" / "Luoi tren dinh".
  # 🔵 Hai ve khong cong nao canh, ca hai dong bang DO: (2) thanh trang thai — 6 ca vitest, tu
  #   kiem do 5/6; (1) guard `caretTarget` — e2e 8 luot, 5 dat/3 do, KHONG luot do nao roi vao
  #   phep khang dinh caret. Do duoc "guard khong pha", KHONG do duoc "guard chua duoc cuoc dua".
  # 🔴 MON MOI, TO HON STORY NAY: bo e2e tung 7/7 DO vi mot khuyet tat cua BAN DO —
  #   `wdio.conf.mjs::devServerIsUp()` tin mot Vite hap hoi vi no van tra 200 cho `/`.
  #   Chan moi story sau co dong toi WKWebView. Chu: mot story ha tang e2e. deferred-work.md.
```
