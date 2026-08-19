---
baseline_commit: 64cf7cbd88ac420551dda37b4a1a743aeb375885
---
# Story 2.6: Lịch sử phiên bản segment và khôi phục

Status: done

**Covers:** FR101 · bước di trú **10** · hai món nợ có chủ đích danh *(`deferred-work.md:2780-2785` · `:2787-2792`)*
**Epic:** 2 — Biên tập theo segment, một vòng dịch tay hoàn chỉnh
**Story trước:** 2.5d (Ngắt đoạn của bản dịch) — `done` 2026-08-16
**Story sau:** 2.7 (Xuất xứ bản dịch cấp segment)
**baseline_commit:** `64cf7cb` — cây làm việc **SẠCH** lúc dựng story (`git status --porcelain` trả 0 dòng, không cần commit vá riêng)

---

## Story

As a **người dịch**,
I want **xem lại các bản dịch trước của một câu và quay về một trong số đó**,
so that **tôi thử một cách diễn đạt khác mà không sợ mất bản cũ**.

---

## Acceptance Criteria

Chép nguyên văn từ `epics.md:2421-2442`. Số hiệu AC1–AC5 là của story này, dùng để tham chiếu trong Tasks.

**AC1 — danh sách phiên bản, mới nhất trước**
**Given** một segment đã được xác nhận nhiều lần
**When** mở lịch sử
**Then** thấy các phiên bản kèm thời điểm, mới nhất trước

**AC2 — khôi phục**
**Given** một phiên bản cũ
**When** người dùng chọn khôi phục
**Then** văn bản đích của segment quay về nội dung đó
**And** trạng thái segment về **chưa xác nhận**

**AC3 — trạng thái rỗng nói ra cơ chế**
**Given** một segment chưa từng được xác nhận
**When** mở lịch sử
**Then** hiện trạng thái rỗng nêu rõ lịch sử sinh ra khi **xác nhận**, không phải khi gõ

**AC4 — segment đã về hưu vẫn tra được lịch sử**
**Given** một segment đã về hưu do gộp hoặc tách
**When** tra lịch sử của nó
**Then** lịch sử vẫn tra lại được

**AC5 — thời điểm**
**Given** thời điểm của mỗi phiên bản
**When** lưu
**Then** ISO-8601 UTC trong database, định dạng hiển thị chỉ ở frontend

---

## 🔴 Quyết định mở — Ice chốt TRƯỚC khi viết dòng mã đầu tiên

Tám chỗ dưới đây có từ hai phương án hợp lệ và **đặc tả không chọn hộ**. Luật của dự án: *"Ice là người chốt các quyết định mở. Gặp một chỗ hai phương án đều hợp lệ: nêu cả hai kèm số đo, đừng tự chọn rồi đi tiếp — và cũng đừng loại một phương án chỉ vì nó đắt"* (`project-context.md:464-466`).

Dev agent **dừng ở đây** và trình tám quyết định. Không tự chọn.

⚠️ **Story này khác ba story trước ở một điểm có thật:** 2.5c và 2.5d thêm một **cột** cho một khái niệm đã có bề mặt. Story này dựng một **bề mặt mới hoàn toàn** cho một bảng đã có dữ liệu, và mockup của nó vẽ **bốn năng lực chưa tồn tại**. Phần lớn tám quyết định dưới đây là về việc *cắt đúng chỗ*, không phải về cách cài.

### Quyết định #1 — Khôi phục ghi cái gì vào lịch sử? Bảng AD-31 KHÔNG có hàng nào cho nó

Đây là quyết định nặng nhất của story, và nó là một **mâu thuẫn đo được giữa hai tài liệu quy hoạch**.

**Số đo — bảng Rule của AD-31 (`ARCHITECTURE-SPINE.md:374-381`) có đúng SÁU hàng**, và không hàng nào là *"khôi phục"*:

| Sự kiện | Trạng thái | `SegmentVersion` |
|---|---|---|
| Auto-save (FR100) | không đổi | **không** tạo |
| Xác nhận segment (FR24) | → đã xác nhận | **tạo một** phiên bản |
| Sửa văn bản của segment đã xác nhận | → **chưa xác nhận** | không tạo |
| Điền sẵn từ TM khớp 100% (FR58) | chưa xác nhận, gắn nhãn *gợi ý* | không tạo |
| Chấp nhận thay đổi từ Review Mode (FR94) | → chưa xác nhận | không tạo |
| Về hưu do gộp/tách (AD-5) | về hưu | không tạo |

**Mockup nói ngược lại, bằng chữ đậm** (`mockups/data-integrity.html:226-229`):

> **Khôi phục là tạo phiên bản mới, không phải xoá phiên bản sau.** Lấy lại bản "hôm qua 21:04" sẽ đẩy nó lên thành phiên bản **thứ sáu** — lịch sử chỉ dài thêm, không bao giờ ngắn đi. Người dịch đổi ý rồi đổi lại là chuyện thường, và một thao tác hoàn tác làm mất bản hiện tại thì lần sau họ sẽ không dám bấm nữa.

Và chân trang cùng tệp (`:307`) lặp lại: *"FR101 — lịch sử phiên bản từng câu · **khôi phục là thêm phiên bản, không xoá**"*.

🔴 **AC2 đứng về phía AD-31, không về phía mockup:** nó nói trạng thái về **chưa xác nhận**. Một phiên bản được ghi cho một văn bản **chưa ai ký** làm bảng `segment_version` mang **hai nghĩa trộn lẫn** — *"bản đã được ký"* và *"bản từng hiện diện"* — đúng lớp lỗi mà `U+26D4` bị gỡ khỏi kho vì mắc phải.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Khôi phục **chỉ** đặt lại `target_text` và hạ `status` về `'draft'`. **Không** `INSERT`. Phiên bản "thứ sáu" của mockup xuất hiện ở **lượt xác nhận kế tiếp**, do chính hàng 2 của AD-31 sinh ra | AD-31 **không sửa một chữ**, AC2 đúng nguyên văn, và lời hứa *"lịch sử chỉ dài thêm"* vẫn giữ — nó chỉ dài thêm **muộn hơn một nhịp**. Cái giá: người dùng khôi phục rồi **không** ký thì lịch sử không dài thêm, và câu *"đẩy nó lên thành phiên bản thứ sáu"* của mockup sai về thời điểm |
| **(b)** | Khôi phục `INSERT` ngay một hàng `segment_version` mang văn bản vừa khôi phục | Đúng nguyên văn mockup. Cái giá: một **`AD` MỚI** — hàng thứ bảy của AD-31 — đi qua thủ tục viết ra (`project-context.md:461-463`), **không** một dòng mã tiện tay. Và bảng nay chứa cả bản chưa ký |
| **(c)** | Khôi phục ghi một phiên bản của **văn bản đang bị thay thế** *(chụp lại trước khi ghi đè)*, không phải của văn bản được khôi phục | Đóng thẳng Quyết định #2 bằng cùng một lượt ghi. Cái giá: cũng là một `AD` mới, và nó làm *"phiên bản"* mang nghĩa thứ ba — *"bản bị thay"* |

🔴 Dù chọn gì: nếu đường ký làm hẹp AC hoặc lệch mockup, phần lệch ghi vào `deferred-work.md` **kèm chủ** ngay lúc ký, và **không** sửa `epics.md`/mockup — dev không sửa tài liệu quy hoạch (`project-context.md:456-458`, Quyết định #3 của Story 1.3).

### Quyết định #2 — Văn bản chưa từng được ký bị khôi phục ghi đè: mất VĨNH VIỄN, và không AC nào nói ra

**Số đo, đọc thẳng từ mã:** một hàng `segment_version` chỉ sinh ra ở **đúng một chỗ** — `segment.rs:955-959`, bên trong `confirm_segment`, sau khi đã qua chốt `if status == SEGMENT_STATUS_CONFIRMED { return Ok(false) }` (`:946-948`). ⇒ Văn bản đích **chưa bao giờ được ký** *(status `'draft'` có nội dung — chính là giá trị `draft` thứ 17 mà Story 2.5b dựng token cho)* **không có một bản sao nào** ở bất cứ đâu.

⇒ Một lượt khôi phục lên một segment đang mang bản nháp chưa ký **xoá vĩnh viễn** bản nháp đó. Đây là §*"Dữ liệu người dùng — chỗ hỏng là VĨNH VIỄN"* (`project-context.md:496`), và **không AC nào của story này nêu nó**. Nó cũng là chính cái lo mà mockup viết ra bằng chữ: *"một thao tác hoàn tác làm mất bản hiện tại thì lần sau họ sẽ không dám bấm nữa"*.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Khôi phục **hỏi lại** khi văn bản hiện tại chưa từng được ký và khác bản sẽ khôi phục | Không mất gì. Cái giá: một hộp thoại xác nhận — mà UX-DR16 và Quyết định #8 của Story 2.5 đã bác *("Không hộp thoại, không lớp nổi")* cho đường báo lỗi. Đây là một ngữ cảnh khác, nên **phải ký lại tường minh**, không suy từ tiền lệ đó |
| **(b)** | Chụp bản chưa ký thành một hàng `segment_version` **trước khi** ghi đè *(gộp với #1(c))* | Không hỏi, không mất. Cái giá: bảng mang bản chưa ký ⇒ kéo theo `AD` mới của #1 |
| **(c)** | Không làm gì — chấp nhận mất, ghi nợ có chủ | Rẻ nhất, trung thực với năm AC đã viết. Cái giá: một lỗ mất dữ liệu **im lặng** trong đúng story mà lời hứa của nó là *"không sợ mất bản cũ"* |

⚠️ Phép so *"đã đổi hay chưa"* **phải so văn bản**, không dùng cờ *dirty* — AD-31 §Hợp đồng phụ bắt buộc (`ARCHITECTURE-SPINE.md:390`). Và phải so với **tập chờ trong bộ nhớ**, không chỉ với đĩa: `editorEditedText` có thể mang ký tự chưa flush.

### Quyết định #3 — Bề mặt: lớp nổi cấp App, một tab trong panel, hay trong chính lưới?

**Số đo:**
- Mockup vẽ một **lớp nổi hai cột** — cột trái 270 px liệt kê phiên bản, cột phải xem nội dung — mở bằng `⌘H`, đóng bằng `Esc`, duyệt bằng `↑`/`↓` (`data-integrity.html:186-247`).
- Kho có **đúng hai** tiền lệ lớp nổi: `ShortcutsOverlay.vue` (569 dòng) và `AttributionOverlay.vue` (401 dòng). Cả hai là **con trực tiếp của `App.vue`** (`:270,273`), **không** phải panel, và **không** có mục nào trong `FOCUS_OWNERS` (`commands/index.ts:66-73`, sáu mục: ba chế độ + ba panel). Chúng dùng `focusReturnTargetOnOpen('[data-…-open]')` (`focus.ts:394-405`) cộng một bẫy `Tab` tự viết.
- Kho có **một** tiền lệ tab trong panel: `LookupPanel.vue:410` (`role="tablist"`, Story 1.20) và `GridPanel.vue:966` (dải tab Hán Việt). Khuôn năm thuộc tính bắt buộc ghi ở `1-20-…md:350-373`.
- 🔴 `check-tokens.mjs` Kiểm F cấm **lớp nổi** ở mức kiểu dáng *(không bóng đổ, không gradient)*; hai overlay đang sống đi qua được nhờ một chú thích miễn trừ **có tên** cho `z-index`. Một bề mặt thứ ba phải đi qua đúng cửa đó, **không** bằng một miễn trừ mới không tên.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Lớp nổi cấp `App.vue` theo khuôn `ShortcutsOverlay.vue` — hai lệnh `open`/`close`, scrim, `role="dialog"`, bẫy `Tab`, `Esc` cục bộ | Đúng mockup, đúng tiền lệ đã qua mười một cổng. Cái giá: một thành phần ~400 dòng, và một `z-index` thứ ba phải khai miễn trừ có tên |
| **(b)** | Một **tab thứ hai** trong panel Lookup, khuôn Story 1.20 | Rẻ hơn, không lớp nổi nào. Cái giá: panel Lookup nay mang một khái niệm **không thuộc tra cứu**, và mockup vẽ hai cột 270 px + nội dung — không lọt vào cột hẹp của bố cục Ⓑ-2 *(đo thật: cột 238,5 px, `deferred-work.md:3131-3162`)* |
| **(c)** | Một vùng mở ra **trong chính hàng của lưới** | Không bề mặt mới. Cái giá: 🔴 **một hàng KHÔNG phải một phần tử DOM** (`GridPanel.vue:5-24`) — năm cột là năm `subgrid` chia chung một tập track (`:1218-1227`); chèn một khối vào giữa một track là thứ hình dạng này **không diễn đạt được**. Và mọi thay đổi chiều cao track kéo theo phép đo `subgrid` 388 px |

⚠️ Dù chọn gì: `Esc` **cục bộ**, không đăng ký trong `CommandRegistry` — hai overlay hiện có ghi lý do bằng chữ *(đăng ký `Escape` toàn cục biến nó thành một phím gán lại được trên toàn ứng dụng)*. Ngược lại, **nút đóng** thì phải đi qua `dispatch('<id>')`: Kiểm A của `check:commands` quét tĩnh mọi `@click` trong `.vue`.
⚠️ `↑`/`↓` duyệt phiên bản cũng **cục bộ**: một hợp âm không phím bổ trợ chính bị nuốt trong vùng gõ (`keys.ts` — `lacksPrimaryMod && isTypingZone`), và bề mặt này không phải vùng gõ.

### Quyết định #4 — Diff `<del>`/`<ins>`: mockup vẽ nó, mà hai crate diff CỐ Ý chưa cài

**Số đo:** `src-tauri/Cargo.toml:86-89` ghi sẵn **cả hai** số phiên bản — `similar` 3.1.1 và `dissimilar` 1.0.11 — và **không cài cái nào**. `project-context.md:112-116` nói thẳng vì sao: *"Cài một trong hai hôm nay là âm thầm đóng một quyết định kiến trúc đang mở (chốt ở Giai đoạn 5, sau khi thử cả hai trên bản review thật)"*. Hàng Deferred của spine (`ARCHITECTURE-SPINE.md:915`) khai cùng một điều.

🔴 **Và năm AC của story này KHÔNG có AC nào đòi diff.** Mockup vẽ nó (`data-integrity.html:216-221`: *"So với phiên bản trước · 14 phút trước"* kèm `<del>`/`<ins>`) vì cùng một bề mặt sẽ phục vụ Diff Viewer của Review Mode (FR93) ở một giai đoạn sau.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Story này **không** dựng diff — mỗi phiên bản hiện **toàn văn**. Vế so sánh ghi nợ có chủ | Trung thực với năm AC, không đóng một quyết định đang mở, không đi qua cửa NFR15. Cái giá: nút *"So với bản đang dùng"* của mockup không có ở bản dựng này |
| **(b)** | Tự viết một hàm diff thuần trong Rust *(không thêm gói)* | Có diff mà không đụng NFR15. Cái giá: 🔴 một thuật toán diff tự viết là **nguồn sự thật thứ hai** với thứ Giai đoạn 5 sẽ chọn — và nó sẽ ở lại vì "đang chạy tốt". Đúng hình dạng mà AD-17 *(một Matcher dùng chung)* tồn tại để cấm |
| **(c)** | Cài `similar` hoặc `dissimilar` ngay | Đúng mockup trọn vẹn. Cái giá: 🔴 đóng một quyết định kiến trúc đang mở **bằng một lượt tiện tay**, cộng ba bước bắt buộc của NFR15 *(mở tệp giấy phép trong nguồn ĐÃ TẢI mà đọc · ghi vào bảng Stack TRƯỚC · chỉ giấy phép tương thích GPLv3)* |

🔴 Nếu diff được dựng ở bất kỳ đường nào: **không** `v-html`. Rust phân tích thành **mô hình dữ liệu có cấu trúc**, Vue render từ mô hình đó, và mô hình **không có nhánh nào mang HTML** (AD-16, `project-context.md:528-530`).

### Quyết định #5 — Bốn nhãn của mockup trỏ vào bốn năng lực chưa dựng

**Số đo:** bảng `segment_version` có **đúng bốn cột** — `id` · `segment_id` · `target_text` · `created_at` (`schema.rs:460-467`). Doc-comment ngay trên (`:436-459`) khai bằng chữ rằng cột **xuất xứ** (FR117) thuộc **Story 2.7** và cột **cặp TM** (FR56) thuộc **Epic 7**, cố ý không thêm sớm.

Mockup thì vẽ mỗi hàng phiên bản kèm một nhãn và một dòng xuất xứ:

| Nhãn mockup | Cần năng lực nào | Trạng thái hôm nay |
| --- | --- | --- |
| `đang dùng` | so `target_text` với hàng `segment` | ⚠️ suy được, **nhưng không an toàn** — hai phiên bản trùng văn bản thì nhãn khớp **nhiều** hàng |
| `từ bản review` | Review Mode (FR94) | chưa dựng |
| `từ AI` | Epic 4 | chưa dựng |
| `từ TM` | Epic 7 (FR58) | chưa dựng |
| *"bạn sửa · đã xác nhận"* | xuất xứ FR117 | **Story 2.7** — story ngay sau story này |

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Story này hiện **thời điểm** và không nhãn nào. Bốn nhãn ghi nợ, mỗi món một chủ *(2.7 · Epic 4 · Epic 7 · FR94)* | Không trỏ tới năng lực chưa tồn tại — đúng luật §KHÔNG-LÀM của Story 1.17 và Quyết định #4 của Story 1.20 *(bỏ tab Concordance vì cùng lý do)*. Cái giá: danh sách phiên bản chỉ có thời điểm, mỏng hơn mockup |
| **(b)** | Thêm nhãn `đang dùng`, giải bài trùng văn bản bằng cách so theo **`id` phiên bản đã khôi phục gần nhất**, không theo nội dung | Một nhãn có nghĩa thật. Cái giá: đòi một cột hoặc một quy ước mới để nhớ *"phiên bản nào đang được dùng"* — tức mở rộng lược đồ cho một thứ mockup vẽ như một nhãn |
| **(c)** | Thêm cột `origin` vào bước di trú **10** ngay bây giờ, để Story 2.7 không phải mở bước thứ hai | Một bước di trú thay vì hai. Cái giá: 🔴 đi ngược quyết định đã ghi tại `schema.rs:436-459`, và ghi một cột mà **story này không có AC nào đòi** — nó sẽ mang giá trị mặc định cho mọi hàng cũ, tức một cột nói dối cho tới khi 2.7 chạy |

### Quyết định #6 — Định dạng thời điểm: kho CHƯA có một hàm định dạng ngày nào

**Số đo:** `grep -rn "toLocale\|Intl\.DateTimeFormat\|new Date(" src --include="*.ts" --include="*.vue"` cho **0** kết quả. Bề mặt "thời gian trôi" duy nhất đang sống — `StatusBar.vue:36-83` (*"Đã lưu N giây trước"*) — làm **số học epoch thuần**, không gọi một API ngày nào:

```ts
const now = ref(Date.now())
const secondsSinceSave = computed<number | null>(() => {
  const at = editorLastSavedAt.value
  if (at === null) return null
  return Math.max(0, Math.floor((now.value - at) / 1000))
})
```

Mockup thì dùng **ba dạng cùng lúc**: *"12 phút trước"* · *"14 phút trước"* · *"Hôm qua 21:04"*.

Ràng buộc đã cố định (không phải chỗ chọn): `created_at` về từ Rust là **chuỗi ISO-8601 UTC** sinh bằng `strftime('%Y-%m-%dT%H:%M:%fZ','now')` (`segment.rs:957`); mọi định dạng làm ở **frontend** (Consistency Conventions §Ngày giờ); mọi chuỗi hiển thị nằm trong `vi.json`.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Tuyệt đối, rút gọn: `2026-08-16 21:04` | Không phụ thuộc locale, không thư viện, kiểm được **tất định**, một khoá `vi.json`. Cái giá: xa mockup nhất, và *"12 phút trước"* đọc nhanh hơn với một danh sách vừa sinh trong phiên |
| **(b)** | Tương đối + tuyệt đối như mockup, bằng một **hàm thuần** nhận `now` qua **tham số** | Đúng mockup. Cái giá: bốn nhánh ngôn ngữ *(vừa xong · N phút trước · Hôm qua HH:mm · ngày đầy đủ)*, bốn khoá `vi.json`, và một quy ước mới cho cả kho |

🔴 Dù chọn gì: **hàm định dạng KHÔNG được tự đọc `Date.now()`** — mọi thời điểm đi vào qua tham số. Luật này đã có sẵn cho `writeSchedule.ts` và lý do ghi tại chỗ: một hàm tự đọc đồng hồ buộc cổng phải `sleep` thật, tức một phép kiểm chậm và chập chờn (`project-context.md:242-243`). Và **không** `vi.useFakeTimers()` khi hàm đã nhận thời điểm qua tham số.
⚠️ `Math.max(0, …)` của `StatusBar.vue` là một chốt chống `Date.now()` **không đơn điệu**. Một hàm mới phải mang chốt tương đương, không bỏ.

### Quyết định #7 — Index trên `segment_version`: hình dạng, và nó có phải một bước di trú không

**Số đo:** món nợ ghi chủ đích danh là story này (`deferred-work.md:2780-2785`):

> *"Bảng `segment_version` KHÔNG có index, và đó là một quyết định chứ không một lượt quên. Story 2.5 **chỉ ghi**, không đọc… Story 2.6 mang đường đọc (lịch sử theo `segment_id`, sắp theo thời điểm), nên nó mang index **cùng lượt** — đúng cách bước 5 mang `idx_segment_chapter_ord` cùng lúc với đường đọc cần nó. **Chủ: Story 2.6.**"*

Doc-comment trong mã nói y hệt (`schema.rs:454-459`).

**Số kế tiếp là 10.** Đo lại từ nguồn 2026-08-16: `PROJECT_MIGRATIONS` (`schema.rs:665-711`) có **tám** bước `[1,2,3,5,6,7,8,9]`, đích **9**; số **4 đã cháy** và có cổng riêng canh việc không tái dùng (`segment_contract.rs:474`).
🔴 **Nguồn sự thật cho số kế tiếp là `PROJECT_MIGRATIONS`, không phải một ghi chép ở nơi khác — kể cả dòng này.** Đo lại lúc bắt đầu.

| Đường | DDL | Ghi chú |
| --- | --- | --- |
| **(a)** | `CREATE INDEX idx_segment_version_segment_created ON segment_version (segment_id, created_at DESC);` | Khớp đúng hình dạng truy vấn của AC1 *(lọc theo `segment_id`, sắp theo thời điểm giảm dần)*. Tên theo khuôn `idx_segment_chapter_ord` |
| **(b)** | `CREATE INDEX idx_segment_version_segment ON segment_version (segment_id);` | Nhỏ hơn, để SQLite tự sắp. ⚠️ Với một segment có ít phiên bản thì hai đường **không phân biệt được bằng phép đo** — nói ra thay vì giả vờ có số |
| **(c)** | Sắp theo `id DESC` thay vì `created_at DESC`, dùng thẳng khoá chính | `id` là `AUTOINCREMENT` nên đơn điệu ⇒ thứ tự `id` **là** thứ tự thời gian, và không cần index thứ hai nào. ⚠️ Nhưng nó khoá thứ tự hiển thị vào một chi tiết cài đặt của SQLite thay vì vào cột mà AC5 nói tới |

⚠️ **Hai mốc thời gian KHÔNG nói cùng một chuyện** — món nợ thứ hai có chủ 2.6 (`deferred-work.md:2787-2792`): `segment.updated_at` là *"mốc sửa **văn bản**"*, do `save_segment_targets` sinh; `segment_version.created_at` là **mốc ký**. Một lượt ký không sửa một ký tự nào nên **không** đụng `updated_at`. Màn hình lịch sử đọc `created_at`, và món nợ đòi **xác nhận lại mệnh đề này** khi dựng màn hình, không chép lại nó.
🔴 **Không** `CHECK` trong DDL — giá trị hợp lệ cưỡng chế ở tầng Rust, đúng khuôn `status`, `is_omitted` và `chapter.status` (`schema.rs:400-402`).
🔴 **Không** sửa `SEGMENT_STATUS_AND_VERSION_DDL` tại chỗ. Một `project.db` đã ở v7 không bao giờ chạy lại nó, và sửa nó cho ra **hai lược đồ khác nhau cho cùng một số phiên bản** — đúng vết sẹo số 4.

### Quyết định #8 — AC4: segment "đã về hưu" chưa có một đường mã nào tạo ra nó

**Số đo:** `retired_at` là `None` cho **mọi** segment hôm nay. `grep "fn merge_segments\|merge_segment\|MergeSegment"` trên `src-tauri/src/**` cho **0** kết quả; Story 2.8 *(gộp và tách segment tường minh)* là `backlog`. `schema.rs:296-298` ghi thẳng: *"Story 2.1 không cho segment nào về hưu; cột có mặt để Story 2.8 không phải mở một bước di trú thứ hai"*.

Điều **đã đúng theo cấu trúc**: `segment_version` **không có `FOREIGN KEY`, không có `ON DELETE CASCADE`** (`schema.rs:440-444`) — về hưu là một tombstone (`retired_at` khác `NULL`), không phải một lượt xoá, nên lịch sử không đi đâu cả.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Khẳng định AC4 bằng **test hợp đồng**: dựng trạng thái về hưu bằng SQL trực tiếp *(khuôn Story 2.5 đã dùng cho `MessageKey::SegmentRetired`)*, khẳng định đường đọc lịch sử vẫn trả đủ và **không** từ chối. Vế *"bề mặt vào"* ghi nợ có chủ **Story 2.8** | Đóng đúng mệnh đề của AC4 ở tầng đóng được. AC4 đóng **một nửa** 🟡, đúng và trung thực |
| **(b)** | Thêm một đường vào riêng cho segment đã về hưu *(một lối mở lịch sử từ ngoài lưới)* | Vế người dùng cũng đóng. Cái giá: **chưa ai đặc tả** thao tác đó, và nó dựng một tương tác cho một trạng thái chưa có cách nào đạt tới |

🔴 **Đường đọc lịch sử KHÔNG được từ chối một segment vì nó đã về hưu.** Ba lệnh ghi hiện có (`confirm_segment` · `set_segment_omitted` · `set_segment_paragraph_end`) đều trả `MessageKey::SegmentRetired` khi `retired_at` khác `NULL` — đúng, vì chúng **ghi**. Một lượt **đọc** lịch sử mà từ chối là AC4 hỏng. Đường **khôi phục** thì ngược lại: nó ghi, nên nó phải từ chối, và AC2 không nói ra điều đó.

---

## Tasks / Subtasks

> Task 0 chạy **trước** mọi task khác và **chặn** chúng.

- [x] **Task 0 — Trình tám quyết định mở cho Ice** (AC1–AC5)
  - [x] 0.1 Trình Quyết định #1–#8 ở mục trên, kèm số đo đã ghi. Không tự chọn đường nào
  - [x] 0.2 Ghi chữ ký của Ice vào `§Dev Agent Record` kèm ngày
  - [x] 0.3 Nếu một chữ ký làm hẹp phạm vi *(ví dụ #4 đường (a), #5 đường (a), #8 đường (a))*: ghi phần còn hở vào `deferred-work.md` **kèm chủ**, ngay lúc ký. **Không** sửa `epics.md` và **không** sửa mockup — ✅ **bảy mục** ghi vào `deferred-work.md` lúc ký *(ba chữ ký thu hẹp đúng như dự đoán, cộng bốn món không nằm trong bảng "Nợ dự kiến")*
  - [x] 0.4 🔴 Nếu chữ ký #1 rơi vào (b) hoặc (c): **dừng lại và nói** — đó là một **`AD` MỚI**, không một dòng mã. Thủ tục ở `project-context.md:461-463`. Story không đi tiếp cho tới khi `AD` được viết ra — 🔵 **KHÔNG kích hoạt**: Ice ký **#1(a)**, AD-31 không sửa một chữ
  - [x] 0.5 Đo baseline **trước khi chạm dòng đầu tiên** và ghi vào story: `cargo test --locked` *(ghi chép nói **359/0/5**)* · `npm run test` *(ghi chép nói **103/103**)*. Số lệch thì **dừng và nói**, đừng sửa cho khớp
  - [x] 0.6 Đo lại **bốn tiền đề** từ NGUỒN, không từ ghi chép của story này: ① `PROJECT_MIGRATIONS` đích mấy ⇒ số kế tiếp · ② `segment_version` có index nào chưa · ③ `grep merge_segment` trên `src-tauri/src` · ④ `grep "toLocale\|Intl\.DateTimeFormat\|new Date("` trên `src`

- [x] **Task 1 — Bước di trú 10: index trên `segment_version`** (AC1, AC5, và món nợ `deferred-work.md:2780-2785`)
  - [x] 1.1 Thêm một hằng `SEGMENT_VERSION_INDEX_DDL` *(tên theo Quyết định #7)* trong `src-tauri/src/core/store/schema.rs`, cạnh `SEGMENT_TARGET_PARAGRAPH_END_DDL` (`:583-586`)
  - [x] 1.2 Doc-comment theo đúng khuôn bốn hằng trước: vì sao số **10**, vì sao một `CREATE INDEX` chứ không sửa `SEGMENT_STATUS_AND_VERSION_DDL` tại chỗ *(vết sẹo số 4)*, và vì sao index đến **bây giờ** mới có *(trích món nợ đã ghi — đường đọc là thứ biện minh cho index)*
  - [x] 1.3 Thêm `Migration { to_version: 10, sql: … }` vào cuối `PROJECT_MIGRATIONS` (`schema.rs:665-711`), kèm comment một dòng nêu Story + lý do số
  - [x] 1.4 Cập nhật khối `🔵 CẬP NHẬT` ở doc-comment đầu `PROJECT_MIGRATIONS`: đích 9 → **10**, tám bước → **chín** bước
  - [x] 1.5 🔴 Nâng fixture `STEP_TEN` của `a_project_database_newer_than_the_app_is_refused_and_never_written_to` (`segment_contract.rs:1313-1345`) lên **`STEP_ELEVEN`** với `to_version: 11` và **chín** phần tử `PROJECT_MIGRATIONS[0..=8]`. Không nâng thì cổng AD-30 *(db mới hơn app bị từ chối mở)* **vẫn xanh** mà chạy trên một db **không** mới hơn app — chết lâm sàng. Doc-comment tại `:1294-1311` đã ghi luật *"số của fixture phải luôn là `target + 1`"*; đây là lượt lặp lại **thứ ba**
  - [x] 1.6 Cập nhật `the_project_migration_set_matches_the_declared_ladder_step_for_step` (`segment_contract.rs:506-514`): `[1,2,3,5,6,7,8,9]` → thêm **10**. ⚠️ Tên hàm này **đã được gỡ số** ở 2.5d — giữ nguyên tên, chỉ sửa mảng
  - [x] 1.7 Rà `pinned_contract.rs:160-175` — hai neo `PROJECT_MIGRATIONS.len()` **8 → 9** và `schema_version()` **9 → 10**, cộng một dòng `🔵 CẬP NHẬT` theo khuôn ba dòng đã có
  - [x] 1.8 Test hình dạng index theo khuôn `a_fresh_project_database_carries_an_is_omitted_column_with_the_shape_ice_signed` (`segment_contract.rs:946`): đọc `sqlite_master` khẳng định index tồn tại, đúng tên, đúng bảng, đúng cột — và khẳng định **không** `CHECK`, **không** `FOREIGN KEY` mới trên `segment_version`
  - [x] 1.9 Test di trú trên dữ liệu **đã có**, khuôn `a_project_database_at_version_eight_backfills_the_target_flag_from_the_source_flag_row_by_row` (`:1212`): dựng fixture ở phiên bản 9 có sẵn hàng `segment_version`, di trú lên 10, khẳng định **không hàng nào đổi** và index có mặt. Fixture dựng từ các bước **THẬT**, không chép tay DDL
  - [x] 1.10 ⚠️ Dự kiến **có** ca đỏ ngoài danh sách: mọi ca neo vào *"đích là 9"*. 2.5c gặp **bốn**, 2.5d gặp **sáu**. Ghi ra ở `§Debug Log References`, đừng sửa im lặng

- [x] **Task 2 — Đường ĐỌC lịch sử ở Rust** (AC1, AC3, AC4, AC5)
  - [x] 2.1 Một DTO mới cho một hàng phiên bản. 🔴 **Không** `#[serde(rename_all = "camelCase")]` — tên trường trả về giữ `snake_case`, và `tests/ipc_contract.rs` khoá hình dạng lỗi theo cùng luật
  - [x] 2.2 Hàm **thuần** nhận `Option<&OpenWork>` theo khuôn hai lớp (`segment.rs:340-384` là ca đọc gần nhất). `SELECT` lọc theo `segment_id`, sắp theo Quyết định #7. Đọc qua `open.store.read(…)`, **không** mở kết nối riêng (AD-11)
  - [x] 2.3 🔴 **Đường đọc KHÔNG từ chối segment đã về hưu** (AC4). Nó vẫn từ chối *"không tìm thấy"* — dùng lại `MessageKey::SegmentNotFound` (`i18n/mod.rs:209`). Khoá `err.segment.*` **mới** chỉ khi có một nhánh từ chối thật sự mới, và phải qua `message_keys!` — một danh sách song song viết tay cho một test đồng bộ **xanh giả**
  - [x] 2.4 Vỏ `#[tauri::command]` **mỏng** trong `pub mod wire` (`segment.rs:1362-1540`), lấy `State` qua **`try_state`**, không `state()`. Tên command trên dây **LÀ** tên hàm ⇒ vỏ sống trong module lồng, không mang hậu tố
  - [x] 2.5 Đăng ký vỏ trong `src-tauri/src/lib.rs` — 2.5d quên bước này một lần và nó không làm cổng nào đỏ
  - [x] 2.6 🔴 Test hợp đồng theo khuôn `the_load_command_carries_the_status_column_over_the_wire` (`segment_contract.rs:2544`; hai bản sao cho `is_omitted` và cờ đích ở `:2607` và `:2683`). **Đây là cổng chống một lỗi ĐÃ XẢY RA THẬT:** bản đầu Story 2.5 thêm `status` vào kiểu TypeScript nhưng quên thêm vào **struct** và vào câu `SELECT` ⇒ `undefined` phía webview, `isConfirmed` **luôn `false`** trên sản phẩm thật, và **74/74 test frontend vẫn xanh** vì fixture chép tay có sẵn cột. Chỉ e2e bắt được. Nguyên vụ ghi ở `segment.rs:154-163`
  - [x] 2.7 Ca: segment **đã về hưu** vẫn trả đủ lịch sử (AC4). Dựng `retired_at` bằng SQL trực tiếp — đó là khuôn Story 2.5 đã dùng, và nó **là** đường duy nhất hôm nay
  - [x] 2.8 Ca: segment **chưa từng được xác nhận** trả danh sách **rỗng** — không lỗi, không `null` (AC3). Phân biệt được với *"không tìm thấy"*: đây đúng là §*"Rỗng IM LẶNG bị cấm; rỗng CÓ LÝ DO thì không"* (`project-context.md:473`)
  - [x] 2.9 Ca: thứ tự **mới nhất trước** đúng với ít nhất ba phiên bản (AC1)

- [x] **Task 3 — Đường KHÔI PHỤC ở Rust** (AC2 — hình dạng theo chữ ký #1 và #2)
  - [x] 3.1 Hàm thuần nhận `Option<&OpenWork>`, một `open.store.write(|tx| …)`, **một** giao dịch — khuôn `set_segment_omitted` (`segment.rs:1078-1158`) và `confirm_segment` (`:888-984`)
  - [x] 3.2 🔴 Ghi **rời rạc**, ghi NGAY: **không** định tuyến qua bộ đệm gõ / `saveSegmentTargets`. Một thao tác người dùng *thấy đã xong* nằm chờ tới 5 giây rồi biến mất nếu app sập (`project-context.md:520-522`, AD-35 §*"Thao tác rời rạc"*)
  - [x] 3.3 Từ chối có hình dạng, dùng khuôn `enum ConfirmReject` + `Arc<Mutex<Option<…>>>` (`segment.rs:821-828`, `:964-983`) — **không** so chuỗi lỗi. Ba nhánh tối thiểu: segment không tồn tại · segment **đã về hưu** *(đường này GHI nên nó phải từ chối — ngược với Task 2.3)* · phiên bản không thuộc segment đó
  - [x] 3.4 🔴 **Ca biên mà AC2 không nêu:** khôi phục về **đúng nội dung đang có**. Không đổi một byte ⇒ có hạ `status` không? `confirm_segment` đã có tiền lệ cho lớp này *(AC13: ký lại một câu đã ký là VÔ HẠI — `Ok(false)`, không hàng mới, không đổi `updated_at`)*. Trả một cờ phân biệt được, khuôn `ConfirmOutcome.version_created` (`segment.rs:768`)
  - [x] 3.5 Thi hành chữ ký #2. Nếu là (a): phép so **phải so văn bản**, không cờ *dirty* (AD-31 §Hợp đồng phụ), và phải so với tập chờ **chưa flush**, không chỉ với đĩa
  - [x] 3.6 ⚠️ **AD-46 chưa ai nói tới ở đây, và đó là một khoảng hở thật.** `is_target_paragraph_end` (bước 9, Story 2.5d) là **dữ liệu riêng của bản dịch**. AC2 nói khôi phục *"văn bản đích"* — không nói cờ đích. Bảng `segment_version` **không lưu cờ** ⇒ khôi phục không thể trả nó về. Ghi ra bằng chú thích tại chỗ và **ghi nợ có chủ**; không tự chấm là đã xét
  - [x] 3.7 Vỏ `wire` mỏng + đăng ký `lib.rs`, cùng luật Task 2.4/2.5
  - [x] 3.8 Ca hợp đồng: khôi phục ⇒ `target_text` đúng bằng nội dung phiên bản **và** `status = 'draft'` (AC2). Cộng ca: lịch sử **không ngắn đi** *(số hàng trước = số hàng sau, hoặc +1 tuỳ chữ ký #1)*
  - [x] 3.9 Ca hợp đồng: một `version_id` **thuộc segment khác** bị từ chối và **không ghi gì** — khuôn `a_segment_id_from_another_chapter_is_refused_and_never_crosses_over` (`segment_contract.rs:3593`)

- [x] **Task 4 — Dây IPC phía TypeScript** (AC1, AC2)
  - [x] 4.1 Hai adapter mới ở `src/config/segment.ts` theo đúng khuôn ba trạng thái: một `invoke`, một `try/catch`, trả `{ <giá trị> | null, error: IpcError | null }`. 🔴 **KHÔNG BAO GIỜ ném**
  - [x] 4.2 🔴 Tham số gửi **camelCase** (`segmentId`, `versionId`) dù hàm Rust nhận `snake_case`; trường **trả về** giữ **`snake_case`**. Hai chiều khác nhau — `segment.ts:494-496` gọi đây là *"chỗ dễ sai nhất trên dây"*
  - [x] 4.3 Kiểu TS cho một hàng phiên bản, `snake_case`, khớp **đúng** tên trên dây. ⚠️ Kiểm kiểu **lúc chạy** cho dữ liệu qua dây — `IpcError` phía TS là một lời khai, không phải bảo đảm của trình biên dịch
  - [x] 4.4 Ba nhánh `catch` đủ cả ba, khuôn đã có: `isIpcError` · có cầu IPC nhưng sai hình dạng · **không** có cầu IPC (`npm run dev` ngoài Tauri) trả `{ x: null, error: null }`

- [x] **Task 5 — Bề mặt lịch sử** (AC1, AC3 — hình dạng theo chữ ký #3)
  - [x] 5.1 Nếu là #3(a): thành phần mới cạnh `ShortcutsOverlay.vue`, mount trong `App.vue`, `role="dialog"` + `aria-modal="true"`, `tabindex="-1"` trên panel
  - [x] 5.2 🔴 Bẫy `Tab` tự viết theo khuôn `trapTab` — `preventDefault()` **luôn luôn**, `focusableWithin(root)`, quay vòng cả hai chiều, và tự lái được từ **mọi** vị trí bắt đầu, không chỉ hai đầu
  - [x] 5.3 🔴 Trả tiêu điểm về đúng chỗ khi đóng: `focusReturnTargetOnOpen('[data-…-open]')` (`focus.ts:394-405`), rơi về nút mở nếu phần tử cũ đã rời DOM, và `console.warn` **tiếng Anh** nếu cả hai trượt
  - [x] 5.4 ⚠️ **Không** thêm mục vào `FOCUS_OWNERS` (`commands/index.ts:66-73`). Hai overlay đang sống **không** có mục nào ở đó — đây là tiền lệ đã đo, không một lượt quên
  - [x] 5.5 Trạng thái mở/đóng là một `ref` **cấp module**, không phải `ref` cục bộ trong `<script setup>` — đổi preset bố cục gọi `api.clear()` rồi dựng lại panel, và chỉ state cấp module sống sót (`lookupPanelState.ts:6-10`)
  - [x] 5.6 Trạng thái rỗng của AC3 phải **nói ra cơ chế**: *"lịch sử sinh ra khi xác nhận, không phải khi gõ"*. Một câu, khoá `vi.json`, giọng **vô nhân xưng** — không xưng *"bạn"*/*"chúng tôi"* (Kiểm D của `check-i18n.mjs`)
  - [x] 5.7 🔴 Màu **và** cỡ chữ **chỉ** đến từ token (`check:tokens` Kiểm B + B2). Không bóng đổ, không gradient (Kiểm F). Một `z-index` mới phải mang chú thích miễn trừ **có tên**, khuôn `aura-allow-z-index` của hai overlay
  - [x] 5.8 ⚠️ **Không** `opacity` trung gian cho *"mờ đi"* (Kiểm D). Và **không** `ornament` làm màu chữ — đo 2026-08-15: **2,44** sáng / **2,64** tối, trượt sàn AA 4,5. Đường đã giải: `on-surface-variant` (**5,60** / **5,56**)
  - [x] 5.9 `Esc` và `↑`/`↓` là handler **cục bộ**, không đăng ký. Nút bấm thì `@click` phải là **đúng một** `dispatch('<id>')` (Kiểm A)

- [x] **Task 6 — Lệnh, phím, chuỗi** (AC1, AC2)
  - [x] 6.1 Đăng ký command trong `installCommands()` ở `src/commands/index.ts` — **chỉ ở đó**. Một lượt HMR dựng lại component sẽ gọi lần hai và `register()` **ném** vì id trùng
  - [x] 6.2 Id theo văn phạm `^[a-z0-9]+(\.[a-z0-9_]+)+$` (`registry.ts:125`), tiền tố miền. ⚠️ Cùng văn phạm với khoá i18n — id trần sẽ bị hai giai đoạn cách nhau nhiều tháng đăng ký trùng và ghi đè nhau âm thầm
  - [x] 6.3 🔴 **Luật hai lệnh, không một lệnh bập bênh** nếu cần cả mở và đóng — Quyết định #3 của 2.5c bác hình dạng bập bênh bằng chữ: *"nhãn của một phím bập bênh không nói được nó sắp làm gì"*, và bảng phím tắt của Story 1.21 hiện đúng một nhãn cho mỗi hàng. Khuôn đã chạy hai lượt: `attribution.open`/`attribution.close` · `shortcuts.open`/`shortcuts.close`
  - [x] 6.4 ⚠️ **Hợp âm phải chưa ai chiếm** — `conflictFor` chạy trên **toàn registry**, không theo chế độ, nên trùng lộ ra ngay ở `register()`. Đã chiếm: `Mod+1/2/3` · `Mod+Alt+1/2` · `Mod+Alt+←/→` · `Mod+Alt+O/J/V/L/S/X/R/P/U` · `Mod+D` · `Mod+Enter` · `Alt+ArrowDown` · `Shift+←/→` · `Alt+Shift+←/→`. Mockup vẽ `⌘H`; grep `KeyH` trên `src/commands/index.ts` cho **0** kết quả ⇒ trống. **Đo lại, đừng chép dòng này**
  - [x] 6.5 🔴 Hợp âm **phải mang `Mod`**: một hợp âm thiếu phím bổ trợ chính bị nuốt trong vùng gõ (`keys.ts` — `lacksPrimaryMod && isTypingZone`), và con trỏ người dùng đang nằm **trong ô bản dịch** khi họ muốn mở lịch sử. `editor.next_untranslated` và bốn lệnh của 2.5c/2.5d đều đã phải ghi ra giới hạn này
  - [x] 6.6 🔴 Luật **erasable-only**: `commands/index.ts` phải **nạp được bằng Node trần**. Handler đi vào qua `CommandDeps` *(prop tuỳ chọn + `portMissing` fallback, khuôn `confirmSegment?`/`setSegmentOmitted?`)*, **không** `import` giá trị từ một module Vue. Một `import` sai giết **ba** phép kiểm cùng lúc, và Kiểm I `abort()` chứ không FAIL — nó **dừng hẳn** CI
  - [x] 6.7 Cắm dep ở `src/main.ts`, cùng chỗ bốn lệnh trước đã cắm
  - [x] 6.8 ⚠️ Nâng `COMMAND_FLOOR` (`check-commands.mjs:241`, hiện **33**) kèm một dòng ghi **số thật mới**. Comment tại chỗ ghi *"đo lại 2026-08-16: **39** command thật — 33/39 = 84,6 %"*. **Đo lại, đừng chép.** Một sàn không được nâng **không làm cổng đỏ** — nó chỉ lặng lẽ mất ý nghĩa
  - [x] 6.9 Nhãn lệnh vào `vi.json` khoá `command.<id>`; chuỗi giao diện khoá chấm có tiền tố miền. `vi.json` **phẳng**, không giá trị rỗng, placeholder đúng dải `{ten_tham_so}`
  - [x] 6.10 ⚠️ **Command id nằm cứng trong spec e2e và không cổng nào canh** (`deferred-work.md:3117-3129`) — `check:commands` **không đọc `e2e/**`**. Thêm id thì tự rà `e2e/**` bằng tay

- [x] **Task 7 — Định dạng thời điểm** (AC5 — theo chữ ký #6)
  - [x] 7.1 Một **hàm thuần** trong một module riêng, nhận `now` **qua tham số**. 🔴 Không tự đọc `Date.now()`
  - [x] 7.2 ⚠️ Nếu module này bị bất kỳ cổng nào `import()` thì nó chịu luật **erasable-only** — kiểm trước khi đặt chỗ, đừng đặt rồi sửa
  - [x] 7.3 Chốt chống `Date.now()` không đơn điệu, khuôn `Math.max(0, …)` của `StatusBar.vue:36-83`
  - [x] 7.4 Ca vitest **tất định**: truyền `now` cố định, khẳng định từng nhánh. 🔴 **Không** `vi.useFakeTimers()` — hàm đã nhận thời điểm qua tham số thì bọc đồng hồ giả là đổi một bảo đảm lấy một thói quen
  - [x] 7.5 ⚠️ Ca biên: `created_at` mang **mili giây** (`%f`) và hậu tố `Z`. `new Date(iso)` phân giải được, nhưng khẳng định nó bằng một ca thay vì tin

- [x] **Task 8 — Test frontend** (AC1, AC2, AC3)
  - [x] 8.1 Tệp mới ở `tests/frontend/**` *(phẳng, **không** đồng vị trí trong `src/`)* — bốn cổng đếm quần thể `src/**` và một tệp test đổ vào đó **thổi phồng mẫu số**
  - [x] 8.2 Mock ở **ranh giới IPC** (`src/config/segment.ts`), khuôn `tests/frontend/support/segmentFixture.ts` + `editorConfirmSegment.test.ts`. **Không** dựng một setter chỉ-dành-cho-test trên mã sản phẩm — kho đã bác hình dạng đó hai lần
  - [x] 8.3 ⚠️ Thêm trường mới vào kiểu `ChapterSegment` *(nếu chữ ký nào đòi)* làm **ba** fixture chép tay đỏ dưới `vue-tsc` — và đó **đúng vai**. Đừng "sửa cho hết đỏ" bằng một `as any`
  - [x] 8.4 ⚠️ Mọi vá `happy-dom` sống ở `tests/frontend/support/setup.ts`, mỗi mục kèm một dòng nói nó thiếu gì. 🔴 **Đường sai rất rẻ:** thêm một `?.` vào **mã sản phẩm** cho hết đỏ là một nhánh mà kiểu nói **không bao giờ chạy** — mã chết vĩnh viễn trong sản phẩm để phục vụ một bản mô phỏng

- [x] **Task 9 — Tài liệu và sổ nợ**
  - [x] 9.1 🔴 Đóng **hai** món nợ có chủ 2.6 bằng cách **nối tiếp** `→ ✅ ĐÃ ĐÓNG <ngày> (Story 2.6)` kèm cách đóng — **không xoá** mục gốc. Nếu chỉ đóng một nửa thì 🟡 và liệt kê phần **CÒN HỞ** kèm chủ mới
  - [x] 9.2 Món nợ `:2787-2792` đòi **xác nhận lại** mệnh đề *"`updated_at` không đổi ở lượt xác nhận"* khi dựng màn hình lịch sử — đọc mã mà xác nhận, đừng chép lại câu đó
  - [x] 9.3 Cập nhật `src/panels/README.md` *(hoặc README của thư mục chứa bề mặt mới)* nếu story thêm một khái niệm
  - [x] 9.4 ⚠️ Rà xem có doc-comment nào trong mã nói ngược với thứ story này vừa dựng không — `schema.rs:436-459` khai *"chưa có index"*; sau story này nó **sai về mã** và phải sửa **tại chỗ** kèm dấu 🔵 và ngày. **Không** sửa `ARCHITECTURE-SPINE.md` trừ khi Quyết định #1 đẻ ra một `AD` mới
  - [x] 9.5 Mọi vế không nghiệm thu được ở tầng này ⇒ `deferred-work.md` **kèm chủ**. Không mục nào mồ côi, không tự chấm đạt

- [x] **Task 10 — Nghiệm thu**
  - [x] 10.1 `npm run check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:layout` · `check:dict` · `check:dict-manifest` · `check:lint` · `check:gates`, **cộng** `check:scope` + `check:scope:bundled` chạy tay *(cần **cổng 1420 trống**)*
  - [x] 10.2 `npm run test` (vitest) · `npm run build` *(`vue-tsc` hai lượt + `vite build`)* · `cargo test --locked`
  - [x] 10.3 e2e chạy tay. 🔴 Ít nhất **một** ca đi trọn vòng trên sản phẩm thật: ký một câu, sửa, ký lại, mở lịch sử, khôi phục, đọc lại qua `read_open_chapter_segments`. Đây là đường duy nhất bắt được lớp lỗi *"cột quên ở một trong hai chỗ ⇒ `undefined` phía webview"* — bốn lớp lỗi nặng nhất của Epic 2 tới nay đều **chỉ** e2e bắt được
  - [x] 10.4 ⚠️ Bảo đảm **cổng 1420 trống**. Nếu **4445** bị chiếm: đặt `TAURI_WEBDRIVER_PORT`, **không** giết tiến trình của Ice *(đo 2026-08-15: `gdrive-su` PID 48486 giữ 4445)*. Và `wdio.conf.mjs::devServerIsUp()` chỉ hỏi `res.ok` — một Vite hấp hối vẫn trả 200 và làm **7/7 spec đỏ oan** (`deferred-work.md:3274-3330`, chưa vá)
  - [x] 10.5 ⚠️ **Đo lại độ trễ dời con trỏ** nếu Task 5 thêm bất kỳ tính toán nào chạy trên **toàn danh sách** mỗi lượt dời con trỏ. 2.5b đo **706–770 ms** trên 9.850 câu — vượt trần NFR2 (50 ms/frame) **~15 lần**, còn hở, chủ là Story 2.4 (`deferred-work.md:3164-3194`). Ghi số, đừng suy luận
  - [x] 10.6 ⚠️ **Đo lại chiều cao hàng** nếu Task 5 chạy đường #3(c). `deferred-work.md:3131-3162` đo `subgrid` ép ô bản dịch cao **388 px** theo cột nguyên văn
  - [x] 10.7 File List kê từ `git status --porcelain`, **không** từ trí nhớ

---

## Dev Notes

### Đọc trước khi viết dòng đầu tiên

`_bmad-output/project-context.md` — 130 luật. Bốn mục sát story này: §Critical Don't-Miss Rules *("Dữ liệu người dùng — chỗ hỏng là VĨNH VIỄN" và "Rỗng IM LẶNG bị cấm")*, §Testing Rules *(bốn đường nghiệm thu, bốn vai không chồng nhau)*, §Code Quality *(chú thích nói **lý do**, kèm **phép đo**)*, §Story và spec *("năng lực chưa dựng ≠ lệch spec")*.

### 🔴 Story này KHÔNG thêm phụ thuộc nào — và có MỘT cám dỗ đã được đặt tên

Mọi thứ cần đã có: `rusqlite` cho index và truy vấn · `CommandRegistry` cho lệnh · khuôn `ShortcutsOverlay.vue` cho bề mặt · `vitest` cho test frontend · khuôn `set_segment_omitted` cho đường ghi rời rạc.

⚠️ Cám dỗ cụ thể ở story này là **crate diff**. Xem Quyết định #4: `Cargo.toml:86-89` ghi sẵn cả hai số và **cố ý không cài cái nào**. Nếu dev agent thấy mình muốn thêm một gói, **dừng lại** — cửa NFR15 ba bước vẫn đứng (`project-context.md:92-100`), và sáu tên bị cấm cưỡng chế bằng `npm run check:deps`.

⚠️ Cám dỗ thứ hai, nhẹ hơn: một thư viện định dạng ngày. Xem Quyết định #6 — kho làm số học epoch thuần và không có một API ngày nào trong `src/`.

### Trạng thái hôm nay của khái niệm "phiên bản" — đọc trước khi sửa

**Bảng đã có, dữ liệu đã có, đường đọc thì chưa.** Story 2.5 dựng bảng ở bước di trú **7** và **chỉ ghi**; story này là lượt đầu tiên đọc nó.

`SEGMENT_STATUS_AND_VERSION_DDL` (`schema.rs:460-467`):

```sql
ALTER TABLE segment ADD COLUMN status TEXT NOT NULL DEFAULT 'draft';
CREATE TABLE segment_version (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  segment_id  INTEGER NOT NULL,
  target_text TEXT    NOT NULL,
  created_at  TEXT    NOT NULL
);
```

**Bốn cột, và cả bốn mệnh đề của lược đồ này là quyết định có chủ ý** (`schema.rs:436-459`):

| Mệnh đề | Vì sao | Hệ quả cho story này |
| --- | --- | --- |
| **Không** `FOREIGN KEY`, **không** `ON DELETE CASCADE` | AD-5: segment **về hưu**, không bị xoá | AC4 đúng **theo cấu trúc** — lịch sử không đi đâu cả |
| **Không** index | Story 2.5 chỉ ghi; index là tối ưu cho một đường đọc chưa ai đo | **Chủ là story này** — Task 1 |
| **Không** cột xuất xứ | FR117 thuộc **Story 2.7** | Quyết định #5 |
| **Không** cột cặp TM | FR56 thuộc **Epic 7** | Quyết định #5 |

**Chỗ DUY NHẤT một hàng phiên bản sinh ra** (`segment.rs:955-959`, trong `confirm_segment`):

```rust
tx.execute(
    "INSERT INTO segment_version (segment_id, target_text, created_at) \
     VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
    (segment_id, &target_text),
)?;
```

⚠️ `created_at` sinh **trong SQL**, không truyền từ Rust — chú thích tại chỗ nói rõ *"Story 2.6 đòi thời điểm, và nó phải sinh từ một đồng hồ"*. Định dạng là **ISO-8601 UTC có mili giây**, đúng AC5.
⚠️ Chốt AC13 đứng **trước** câu `INSERT` (`:946-948`): ký lại một câu **đã ký** trả `Ok(false)`, không hàng mới, không đổi `updated_at`. ⇒ Giữ phím xác nhận **không** bơm lịch sử đầy bản sao.
⚠️ `ConfirmOutcome.version_created: bool` (`segment.rs:768`) đã phân biệt sẵn *"lượt này có tạo phiên bản không"*.

### Lược đồ và di trú

`PROJECT_MIGRATIONS` (`schema.rs:665-711`) hôm nay có **tám** bước, đích **9**:

| `to_version` | hằng | story |
| --- | --- | --- |
| 1 | `SCHEMA_MIGRATION_LOG_DDL` | 1.15 |
| 2 | `WORK_DDL` | 1.15 |
| 3 | `CHAPTER_DDL` | 1.15 |
| **5** | `SEGMENT_DDL` | 2.1 |
| 6 | `SEGMENT_TARGET_TEXT_DDL` | 2.2 |
| 7 | `SEGMENT_STATUS_AND_VERSION_DDL` | 2.5 |
| 8 | `SEGMENT_OMITTED_DDL` | 2.5c |
| 9 | `SEGMENT_TARGET_PARAGRAPH_END_DDL` | 2.5d |

**Số 4 đã cháy.** Cổng thật là `segment_contract.rs::the_project_migration_set_never_reuses_the_burned_number_four` (`:474`); `validate_strictly_increasing` chỉ đòi tăng dần nghiêm ngặt và **không** bắt được việc tái dùng.

🔴 **Nguồn sự thật cho số kế tiếp là `PROJECT_MIGRATIONS`, không phải một ghi chép ở nơi khác — kể cả story này.** Đo lại lúc bắt đầu (Task 0.6).

**Cách di trú chạy** (`schema.rs:812-866`): lọc `m.to_version > from`, mỗi bước **một** `Transaction` riêng, `tx.execute_batch(m.sql)` — nhiều câu ngăn bằng `;` chạy trọn trong một giao dịch — rồi một dòng vào `schema_migration_log`, rồi `PRAGMA user_version`, rồi commit. `Migration::sql` là `&'static str`, và `concat!` chỉ ghép **literal**, không ghép một hằng có tên.

**Bảng `segment` hôm nay** (mười hai cột): `id` · `chapter_id` · `ord` · `source_text` · `is_paragraph_end` · `retired_at` · `created_at` · `updated_at` · `target_text` · `status` · `is_omitted` · `is_target_paragraph_end`. Một chỉ mục: `idx_segment_chapter_ord`.

⚠️ **21** `project.db` thật và **10.477** hàng `segment` từ 21 Chương *(đo 2026-08-12)* sẽ chạy bước 10. Một `CREATE INDEX` trên một bảng gần rỗng là rẻ — nhưng **đo, đừng suy**.

### Dây IPC — và một lỗi đã xảy ra thật

`ChapterSegment` (`segment.rs:187-198`) đi trên dây **không** có `#[serde(rename_all)]`, nên trường trả về giữ `snake_case`. ⚠️ Chiều ngược lại khác: `invoke()` gửi **tham số** dạng camelCase dù hàm Rust nhận `snake_case`.

🔴 **Tiền lệ lỗi phải chặn, ghi nguyên vụ tại `segment.rs:154-163`:** bản đầu Story 2.5 thêm `status` vào kiểu TypeScript nhưng quên thêm vào **struct** và vào câu `SELECT`. Kết quả: `segment.status` luôn `undefined` phía webview, `isConfirmed` luôn `false` **trên sản phẩm thật** — và **74/74 test frontend vẫn xanh**, vì fixture chép tay có sẵn cột. Chỉ e2e bắt được. 2.5c và 2.5d đã dựng lưới riêng cho cột thứ hai và thứ ba; story này dựng một **struct mới hoàn toàn**, tức cùng lớp rủi ro ở một hình dạng khác.

**Sáu command hiện có trong `wire`** (`segment.rs:1362-1540`): `split_chapter_into_segments` · `read_open_chapter_segments` · `save_segment_targets` · `confirm_segment` · `set_segment_omitted` · `set_segment_paragraph_end`. Cả sáu theo đúng khuôn hai lớp: một **hàm thuần** nhận `Option<&OpenWork>` *(thứ `tests/**` gọi được không cần webview)* và một vỏ mỏng lấy `State` qua **`try_state`**.

🔴 `try_state`, **không** `state()`: mở kho có thể đã thất bại và `app.manage()` chưa từng chạy ⇒ `state()` panic ⇒ `panic = "abort"` giết cả tiến trình.

⚠️ Tên `editor.restore_segment` **đã bị chiếm** — nó là lệnh **bỏ cờ cắt bỏ** của Story 2.5c (FR133), **không** phải khôi phục phiên bản. Đặt tên cho lệnh của story này mà không đâm vào nó.

### Bề mặt: hai tiền lệ lớp nổi, đọc trước khi dựng cái thứ ba

| | `ShortcutsOverlay.vue` (569 dòng) | `AttributionOverlay.vue` (401 dòng) |
| --- | --- | --- |
| Chỗ mount | con trực tiếp của `App.vue:273` | `App.vue:270` |
| Bật/tắt | `ref` **cấp module** + hai lệnh `shortcuts.open`/`shortcuts.close` | `attribution.open`/`attribution.close` |
| Vào tiêu điểm | `watch(isOpen, …)` → `nextTick()` → `panel.value?.focus()` | y hệt |
| Ra tiêu điểm | `focusReturnTargetOnOpen('[data-…-open]')`, rơi về nút mở, cuối cùng `console.warn` **tiếng Anh** | y hệt |
| `Escape` | handler **cục bộ**, cố ý **không** đăng ký | y hệt |
| Nút đóng | `dispatch('<id>')` — Kiểm A đòi đúng một lời gọi | y hệt |
| `FOCUS_OWNERS` | **không có mục nào** | **không có mục nào** |

🔴 `nextTick` là bắt buộc: `v-if` chưa mount nút DOM tại thời điểm watcher bắn.
🔴 `Escape` đăng ký toàn cục biến nó thành một phím **gán lại được** trên toàn ứng dụng — sai. Lý do ghi bằng chữ trong cả hai tệp.

### Lưới: vì sao không mở lịch sử "ngay trong hàng"

`GridPanel.vue:1012` là `<div class="grid">` cha khai `grid-template-rows` động. **Năm cột là năm con trực tiếp**, mỗi cột `grid-row: 1 / -1` + `grid-template-rows: subgrid` (`:1218-1227`).

🔴 **Một hàng KHÔNG phải một phần tử DOM** (`:5-24`). Mọi kiểu dáng cấp hàng phải nhân ra từng ô. Một khối mở ra "giữa hai hàng" là thứ hình dạng này **không diễn đạt được** — xem Quyết định #3(c).

**Neo:** `data-segment-id` xuất hiện **hai lần** mỗi câu (ô nguyên văn + ô bản dịch), phân biệt bằng `data-col="src"|"tgt"`. Đây là đường lấy *"segment nào đang được xem lịch sử"* nếu bề mặt cần nó — nhưng `editorCaretSegmentId` (`editorPanelState.ts`) là nguồn sạch hơn.

⚠️ **Ảnh chụp hiển thị dựng bằng MẢNG MỚI** *(trải phần tử cũ)* — `shallowRef` không theo dõi sửa tại chỗ (`editorPanelState.ts:704-709`). Một lượt khôi phục phải đi qua đúng khuôn đó, nếu không đĩa đổi mà lưới thì không. Story 2.5c đã mất một vòng chẩn đoán vì chính chỗ này (commit `4ce5bb4`).

### Luật "erasable-only" — một `import` sai giết ba phép kiểm

Tệp phải **nạp được bằng Node trần** *(cổng `import()` chúng để chạy kiểm **hành vi** trên chính mã sản phẩm)*: `src/commands/{registry,focus,keys,index}.ts` · `src/panels/editorSegments.ts` · `src/panels/segmentNavigation.ts` · `src/layout/{workspaceLayout,writeSchedule}.ts`.

⇒ Không `import` **giá trị** của `vue`/`dockview`/`@tauri-apps/api`; không `enum`, `namespace`, parameter property. Kiểm I `abort()` chứ không FAIL — nó **dừng hẳn** CI (`check-commands.mjs:794-813`).

`editorPanelState.ts` · `editorFlush.ts` · `GridPanel.vue` · hai overlay **không** chịu luật này. Một module định dạng thời gian mới thì **tuỳ** — xem Task 7.2.

### Bài học từ ba story trước

**① Đo trước khi vá, và luật dừng là thật.** 2.5b tốn **bốn** lượt vá "hợp lý" cho một chẩn đoán sai; cả bốn bị phép đo bác. 2.5d lật **một** chữ ký của Ice bằng chính bàn đo của nó.

**② Một chữ ký thi hành ĐÚNG MỘT NỬA là khuôn lặp ba lần.** Lượt rà của 2.5b tìm ra: *"Nửa khó, có chú thích 🔵 đẹp thì làm; nửa là MỘT DÒNG CHUỖI hoặc MỘT CÂU PHẢI XOÁ thì rơi. Không cổng nào canh nửa đó."* Hậu quả nặng nhất đã vá: một nhãn bố cục **nói dối**.

**③ Số của fixture "tương lai" phải luôn là `target + 1`.** 2.5c nâng 8→9, 2.5d nâng 9→10. Story này là lượt lặp lại **thứ ba** — nâng 10 → **11**.

**④ Kiểu bắt được thứ e2e từng phải bắt** — chỉ vì `tsconfig.json` include cây test. Đừng "sửa cho hết đỏ" bằng một `as any`.

**⑤ Story có thể nói SAI một điều kiện.** 2.5d: Task 5.4 viết một điều kiện **sai**, và phép đo tìm ra *(mọi Chương mới sẽ có cờ đích tắt hết)*. ⇒ Đọc mã mà xác nhận từng tiền đề của story này, đừng thi hành nó như một mệnh lệnh.

**⑥ File List kê thừa là một món nợ.** Kê từ `git status --porcelain`.

### Project Structure Notes

- Test frontend ở `tests/frontend/**` **phẳng**, không đồng vị trí trong `src/`.
- Chuỗi literal trong `src-tauri/src/**` viết **KHÔNG DẤU** (`check-i18n` Kiểm A); `tests/**` và `tools/dict-build/**` được miễn trừ. Comment tiếng Việt có dấu thì được ở cả hai.
- Chẩn đoán `console.warn` trong `.vue` viết **tiếng Anh** — cùng luật Kiểm A. ⚠️ Kiểm A quét `.rs` và `.vue`, **không** quét `.ts` (`check-i18n.mjs:839,860-861`) — một chú thích nói ngược điều này đã bị bác ở lượt rà 2.5d.
- Tên hàm test là một **câu khẳng định**, không `test_foo`, và **không mang số hiệu** nếu số đó sẽ trôi.
- Hai họ test Rust: `*_contract.rs` (hợp đồng) · `*_boundary.rs` (ranh giới module).
- Bàn đo, nếu có, đi vào `2-6-ban-do/` cạnh story, khuôn `2-5b-ban-do/` · `2-5c-ban-do/` · `2-5d-ban-do/`. Nó là **tạo tác của một lượt đo**, không phải công cụ của dự án — không có gì ở đó vào `package.json`.
- Commit: `type(scope): câu tiếng Việt`, `scope = story-2.6`. Câu sau dấu hai chấm **nói ĐIỀU ĐÃ TÌM RA**, không chỉ điều đã sửa. Mỗi lớp một commit sạch.
- Thư mục mang một khái niệm thì có `README.md` — cập nhật cùng lượt.

### Bẫy đã biết, ghi ra thay vì để phát hiện lại

- ⚠️ **Mockup mang một cây `.atproj` đã LỖI THỜI.** `data-integrity.html:259-271` vẽ `segments.db` · `history.db` · `tm.db` · `chapters/*.chapter` **rời nhau**. Thực tế trên đĩa là **một** `project.db` (`core/library/atproj.rs:6`, AD-9). Đừng dùng cây đó làm hướng dẫn lược đồ.
- ⚠️ **Mockup vẽ `⌘H` mà bảng phím tắt của `mockups/settings.html` không có hàng nào cho nó**, và `EXPERIENCE.md:262-268` *(sáu hàng)* cũng không. Id lệnh và hợp âm là quyết định **của story này**, không phải một thứ chép về.
- ⚠️ **`EXPERIENCE.md` chưa có hàng *"đã dịch, chưa xác nhận"*** — món nợ chủ **Ice** (`deferred-work.md:2775-2778`). Dev **không** sửa `EXPERIENCE.md`.
- 🔴 **`check-i18n.mjs` Kiểm A báo FAIL SAI CHỖ** khi một tên thẻ được nhắc trong **comment** của template `.vue` — nó báo ở một comment **khác cách đó 20 dòng** (`deferred-work.md:3551-3564`, chưa vá). Story này viết một `.vue` mới; nếu cổng đỏ ở một chỗ vô lý, đây là lý do.
- ⚠️ **Fixture e2e không reset state panel giữa các spec** (`deferred-work.md:3093-3115`, chủ Story 1.22). Spec mới nên tự nạp lại webview sau khi tạo Tác phẩm.
- ⚠️ **`editorOmitError` và `editorConfirmError` được export mà chỉ một chỗ đọc.** Nếu story này dựng một lệnh mới, đừng nhân thêm một ô lỗi thứ ba không ai đọc — món nợ *"đường báo lỗi dùng chung cho lệnh Editor"* đã có (`deferred-work.md:3543-3549`).
- ⚠️ **Ba khoá `err.segment.*` vẫn chưa có đường ra màn hình đầy đủ**: `editorConfirmError` hiện một chuỗi **cố định**, không đọc `message_key` thật (`deferred-work.md:2825-2840`, 🟡, chủ Ice). Một lượt từ chối khôi phục sẽ thừa hưởng đúng bề mặt dở đó.
- ⚠️ **Vế Blink của mọi phép đo hình học là khoảng mù có tên** — chỉ tới được qua WebView2/Windows (`deferred-work.md:3564-3575`, chủ Story 1.22).

### References

- `_bmad-output/planning-artifacts/epics.md:2413-2443` — Story 2.6, năm AC · `:2038-2040` — mục tiêu Epic 2 *(gồm câu "Mọi phiên bản cũ của một segment xem lại và khôi phục được")*
- `_bmad-output/planning-artifacts/epics.md:2215-2249` — Story 2.5 *(nguồn của `segment_version`)* · `:2446-2484` — Story 2.7 *(xuất xứ, story kế)* · `:2487-2528` — Story 2.8 *(gộp/tách, `backlog`)*
- `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md:777` — FR101 nguyên văn · `:775` — FR100 · `:767` — FR97 *(`.atproj` tự chứa, gồm lịch sử phiên bản)* · `:443` — FR117 · `:831` — NFR2 · `:904` — NFR18
- `.../architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md:368-392` — **AD-31 đầy đủ, bảng sáu hàng** · `:89-93` — AD-3 · `:103-111` — AD-5 · `:153-157` — AD-11 · `:362-366` — AD-30 · `:394-398` — AD-32 · `:406-417` — AD-34 · `:419-425` — AD-35 · `:652-673` — AD-46 · `:675-699` — Consistency Conventions · `:831` — sơ đồ ER `SEGMENT ||--o{ SEGMENT_VERSION` · `:915` — hàng Deferred của Diff Viewer
- `.../ux-designs/ux-AuraTranslate-2026-08-02/mockups/data-integrity.html:179-248` — **bề mặt FR101 đầy đủ** · `:226-229` — hộp *"Khôi phục là tạo phiên bản mới"* · `:259-271` — cây `.atproj` **đã lỗi thời** · `:307` — chân trang FR101
- `.../ux-designs/.../EXPERIENCE.md:261-268` — bảng phím sáu hàng · `:56` · `:171` · `:382` — ba chỗ nói *"lịch sử của hai câu cũ vẫn tra lại được"*
- `_bmad-output/planning-artifacts/sprint-change-proposal-2026-08-14.md:344` — *"2.6 → 2.11 theo thứ tự cũ"*, story này **không** bị correct-course chạm
- `src-tauri/src/core/store/schema.rs:436-459` *(bốn mệnh đề của lược đồ + chủ của index)* · `:460-467` *(DDL)* · `:296-298` *(`retired_at`)* · `:583-586` *(tiền lệ DDL+DML)* · `:665-711` *(`PROJECT_MIGRATIONS`)* · `:812-866` *(`migrate`)*
- `src-tauri/src/commands/segment.rs:154-163` *(nguyên vụ cột bị quên)* · `:187-198` *(`ChapterSegment`)* · `:340-384` *(đường đọc)* · `:759-769` *(`ConfirmOutcome`)* · `:821-828` *(`ConfirmReject`)* · `:888-984` *(`confirm_segment` trọn vẹn)* · `:946-959` *(chốt AC13 + `INSERT`)* · `:1078-1158` *(`set_segment_omitted`)* · `:1362-1540` *(`wire`)*
- `src-tauri/src/core/i18n/mod.rs:62-91` *(`message_keys!`)* · `:209` · `:220` · `:229` · `:248` *(bốn khoá `err.segment.*`)* · `:350-387` *(`IpcError::new`)*
- `src-tauri/tests/segment_contract.rs:474` · `:506-514` · `:946` · `:1212` · `:1294-1345` *(fixture "tương lai")* · `:1895` · `:2091` *(`read_state`)* · `:2184` · `:2226-2258` · `:2279` · `:2314` · `:2418` · `:2544` · `:3593`
- `src-tauri/tests/pinned_contract.rs:160-175` — hai neo phiên bản
- `src/panels/GridPanel.vue:5-24` · `:1012` · `:1218-1227` · `:1138-1140` *(ô lỗi)* · `:966` *(dải tab)*
- `src/panels/editorPanelState.ts:187-190` *(mốc lưu là số epoch, không phải chuỗi)* · `:704-709` *(mảng mới)*
- `src/config/segment.ts:494-496` *(hai chiều camelCase/snake_case)*
- `src/ShortcutsOverlay.vue` · `src/AttributionOverlay.vue` — hai tiền lệ lớp nổi · `src/commands/focus.ts:394-405` *(`focusReturnTargetOnOpen`)* · `src/commands/index.ts:66-73` *(`FOCUS_OWNERS`)*
- `src/StatusBar.vue:36-83` — bề mặt "thời gian trôi" duy nhất, số học epoch thuần
- `scripts/check-commands.mjs:241` *(`COMMAND_FLOOR` = 33)* · `:794-813` *(Kiểm I `abort`)* · `:2116-2269` *(Kiểm I)*
- `_bmad-output/implementation-artifacts/deferred-work.md:2780-2785` · `:2787-2792` *(hai món có chủ 2.6)* · `:1959-1968` · `:2825-2840` · `:3093-3129` · `:3131-3162` · `:3164-3194` · `:3274-3330` · `:3543-3575`
- `_bmad-output/implementation-artifacts/1-20-lich-su-tra-cuu-va-muc-da-ghim.md:350-378` — khuôn dải tab + state cấp module
- `_bmad-output/implementation-artifacts/2-5-xac-nhan-segment-va-may-trang-thai.md:169-185` — **Quyết định #6 của 2.5**, chỗ bảng `segment_version` được đặt ra và ranh giới với story này
- SQLite `CREATE INDEX` — https://www.sqlite.org/lang_createindex.html

---

## Testing

Bốn đường nghiệm thu, **bốn vai không chồng nhau**. Chọn sai đường là dựng nguồn sự thật thứ hai — trước khi viết một phép kiểm mới, hỏi: **mệnh đề này đã có chủ ở đường nào chưa?**

| Mệnh đề của story này | Đường đúng |
| --- | --- |
| Bước 10 tồn tại, danh sách di trú đúng, index đúng tên/bảng/cột trên db thật | `cargo test` — `segment_contract.rs` |
| Di trú lên 10 **không đổi một hàng `segment_version` nào** | `cargo test` |
| Danh sách phiên bản theo `segment_id`, **mới nhất trước** (AC1) | `cargo test` |
| Segment **chưa từng ký** trả danh sách rỗng, phân biệt được với "không tìm thấy" (AC3) | `cargo test` |
| Segment **đã về hưu** vẫn trả đủ lịch sử (AC4) | `cargo test` — dựng `retired_at` bằng SQL trực tiếp |
| Khôi phục ⇒ `target_text` đúng và `status = 'draft'` (AC2) | `cargo test` |
| Lịch sử **không bao giờ ngắn đi** sau một lượt khôi phục | `cargo test` |
| Một `version_id` thuộc segment khác bị từ chối và **không ghi gì** | `cargo test` |
| `created_at` là ISO-8601 UTC có mili giây (AC5, vế lưu) | `cargo test` |
| Cột/struct mới đi qua dây IPC *(không `undefined` phía webview)* | `cargo test` — khuôn `the_load_command_carries_…_over_the_wire` |
| Hàm định dạng thời điểm, từng nhánh, với `now` truyền vào (AC5, vế hiển thị) | `vitest` — **không** `useFakeTimers` |
| Adapter IPC không ném ở cả ba nhánh `catch` | `vitest` |
| Lượt khôi phục dựng **mảng mới** ⇒ lưới thấy thay đổi | `vitest` |
| Bẫy `Tab` quay vòng từ **mọi** vị trí bắt đầu; `Esc` đóng; tiêu điểm trả về nút mở | `vitest` *(hành vi)* + **e2e** *(engine thật)* |
| Lớp nổi vẽ ra đúng chỗ, chiều cao/tràn trên WKWebView thật | **bàn đo / e2e** — `happy-dom` **không phải** WebKit |
| Trọn vòng: ký → sửa → ký lại → mở lịch sử → khôi phục → đọc lại từ đĩa | **e2e** — đường duy nhất bắt được lớp lỗi *"cột quên ở một trong hai chỗ"* |
| Độ trễ dời con trỏ sau khi thêm bề mặt mới (NFR2) | **bàn đo** — giao số cho Story 2.4, không tự chấm |

**Luật cổng:** mã thoát là phán quyết; lỗi hạ tầng **không phải** một phép kiểm đỏ (`abort()`, thoát khác 0, nói rõ *"đây là lỗi hạ tầng"*); không phán quyết nào đọc tham số từ chính thứ nó đang kiểm; *"cây rỗng không phải cây sạch"* — sàn quần thể phải xét lại khi thêm tệp vào `src/**`.

**Luật đo:** không đánh dấu đạt bằng suy luận. Số đo ghi kèm **phiên bản toolchain và ngày** — *"số đo không truy nguyên được thì không phải số đo"*.

**Thêm một cổng = sửa BA danh sách** (`package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`), `check:gates` canh cả ba. Story này nhiều khả năng **không** thêm cổng nào.

⚠️ **`npm run test:e2e` không nằm trong CI và không nằm trong pre-push** — nó chỉ chạy tay. Bốn lớp lỗi nặng nhất của Epic 2 tới nay đều **chỉ** e2e bắt được.

---

## Nợ dự kiến (ghi vào `deferred-work.md` kèm chủ, không tự chấm đạt)

| Món | Trạng thái dự kiến | Chủ |
| --- | --- | --- |
| AC4 vế **bề mặt vào** — không đường mã nào cho segment về hưu, nên vế người dùng không đối chứng được | 🟡 | Story 2.8 |
| Bốn nhãn của mockup *(`từ bản review` · `từ AI` · `từ TM` · dòng xuất xứ)* — trỏ vào năng lực chưa dựng | 🟡, bốn mục | Story 2.7 · Epic 4 · Epic 7 · FR94 |
| Vế **diff** *(`So với phiên bản trước`)* — nếu Quyết định #4 chọn (a) | 🟡 | Giai đoạn 5 *(cùng lượt chốt `similar` vs `dissimilar`)* |
| `is_target_paragraph_end` **không** được khôi phục cùng `target_text` — bảng không lưu cờ *(Task 3.6)* | 🔴 hở | Ice *(quyết định ngữ nghĩa)* |
| Bản nháp **chưa từng ký** bị ghi đè khi khôi phục — nếu Quyết định #2 chọn (c) | 🔴 hở | theo chữ ký của Ice |
| Nếu Quyết định #1 chọn (b)/(c): một **`AD` mới** phải được viết ra | 🔴 chặn | Ice |
| Lượt từ chối khôi phục thừa hưởng ô lỗi hiện một **chuỗi cố định**, không đọc `message_key` thật | 🟡 | món nợ đã có, chủ Ice |
| Vế **Blink** của mọi phép đo hình học của bề mặt mới | khoảng mù có tên | Story 1.22 |
| Độ trễ dời con trỏ nếu Task 5 thêm tính toán chạy trên toàn danh sách | phụ thuộc | Story 2.4 |

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, skill `bmad-dev-story`) — 2026-08-16.

### Baseline đo trước khi chạm dòng đầu tiên

Đo trên `HEAD = 64cf7cb`, **trước** khi sửa một dòng mã nào. Cây làm việc lúc bắt đầu mang đúng
hai thay đổi, và **cả hai là tạo tác của `create-story` cho chính story này** — `M sprint-status.yaml`
và `?? 2-6-….md`. Không có công việc lạ nào cần một commit vá riêng.

| Đường | Số đo | Ghi chép của story | Phán quyết |
| --- | --- | --- | --- |
| `cargo test --locked` | **359 passed · 0 failed · 5 ignored** | 359/0/5 | ✅ KHỚP |
| `npm run test` (vitest) | **103/103**, 10 tệp, 2,26 s | 103/103 | ✅ KHỚP |

Toolchain lúc đo: `vitest 4.1.10` · Node/npm theo `package.json` của kho. *(Luật đo: "số đo không
truy nguyên được thì không phải số đo".)*

### Bốn tiền đề đo lại từ NGUỒN, không từ ghi chép

| # | Tiền đề | Nguồn đọc thật | Kết quả | So với story |
| --- | --- | --- | --- | --- |
| ① | `PROJECT_MIGRATIONS` đích mấy | `schema.rs:667-711` — tám `to_version`: **1·2·3·5·6·7·8·9** | đích **9** ⇒ kế tiếp **10** | ✅ KHỚP |
| ② | `segment_version` có index chưa | `grep "CREATE INDEX" schema.rs` — chỉ **một** index trong cả lược đồ: `idx_segment_chapter_ord` (`:355`) | **chưa có index nào** | ✅ KHỚP |
| ③ | `merge_segment` trên `src-tauri/src` | `grep` cho **1** dòng | **0 đường mã** — xem bẫy ngay dưới | ✅ KHỚP *(về bản chất)* |
| ④ | API ngày tháng trên `src/` | `grep "toLocale\|Intl.DateTimeFormat\|new Date("` | **0** | ✅ KHỚP |

🔴 **Bẫy đo tìm ra ở ①-③, ghi ra thay vì để người sau vấp lại: một phép grep tự bắt chính câu nói
về phép grep.** Tiền đề ③ trả **1** kết quả chứ không 0, và kết quả đó là
`core/segment/paragraph.rs:10` — một **doc-comment** viết nguyên văn
`grep "fn merge_segments\|merge_segment\|MergeSegment" trên src-tauri/src/** cho **0**`.
Tức số thật vẫn là **0 đường mã**, nhưng một dev agent đọc số thô sẽ kết luận ngược. Bài học ⑤ của
story *("story có thể nói SAI một điều kiện")* có một anh em sinh đôi: **một phép đo có thể nói sai
vì chính tài liệu đã ghi kết quả của nó vào cây nguồn.** Mọi lượt grep tiền đề từ nay đọc **nội
dung dòng khớp**, không chỉ đếm.

### Ba tiền đề PHỤ đo thêm (không nằm trong Task 0.6, nhưng tám quyết định dựa vào chúng)

| Tiền đề | Nguồn | Kết quả |
| --- | --- | --- |
| Bảng Rule của AD-31 có mấy hàng | `ARCHITECTURE-SPINE.md:374-381` | đúng **sáu** hàng, **không** hàng nào là *"khôi phục"* — mâu thuẫn với mockup là **thật** |
| Hai crate diff đã cài chưa | `src-tauri/Cargo.toml:86-89` | ghi sẵn `similar` 3.1.1 · `dissimilar` 1.0.11, **không cài cái nào**. 🔵 Mã nói chủ cụ thể hơn story: chốt ở **Story 8.1**, không chỉ *"Giai đoạn 5"* |
| `Mod+H` có ai chiếm chưa | `grep KeyH src/commands/index.ts` | **0** ⇒ trống. Bảng hợp âm đã chiếm khớp nguyên văn Task 6.4 |

🔵 Một chi tiết của Quyết định #3 hẹp hơn story mô tả: hai overlay **không** cần một `z-index` mới
— cả hai dùng **cùng một giá trị `10`** và **cùng một câu miễn trừ có tên** `aura-allow-z-index`
(`ShortcutsOverlay.vue:371-372` · `AttributionOverlay.vue:273-274`). Bề mặt thứ ba tái dùng đúng
khuôn đó, không khai một tầng thứ ba.

### Chữ ký của Ice cho tám quyết định mở

**Ngày ký: 2026-08-16.** Tám chỗ, tám chữ ký, trình trọn gói kèm số đo đã xác minh bằng cách
**đọc mã**, không chép ghi chép của story.

| # | Đường ký | Hệ quả thi hành |
| --- | --- | --- |
| **#1** | **(a)** Khôi phục **không** `INSERT` — chỉ đặt lại `target_text` và hạ `status` về `'draft'` | 🔵 **AD-31 không sửa một chữ, và Task 0.4 KHÔNG kích hoạt** — không `AD` mới, story đi tiếp. Phiên bản *"thứ sáu"* của mockup sinh ở **lượt xác nhận kế tiếp**, do chính hàng 2 của AD-31. Vế lệch mockup *(mockup nói nó xuất hiện NGAY)* ghi nợ |
| **#2** | **(a)** **Hỏi lại** khi văn bản hiện tại chưa từng được ký **và** khác bản sẽ khôi phục | Không lỗ mất dữ liệu im lặng. ⚠️ Phép so **phải so văn bản** *(AD-31 §Hợp đồng phụ)*, và phải so với **tập chờ chưa flush**, không chỉ với đĩa |
| **#3** | **(a)** Lớp nổi cấp `App.vue`, khuôn `ShortcutsOverlay.vue` | Tái dùng `z-index: 10` + câu miễn trừ có tên `aura-allow-z-index` đã có — **không** khai tầng thứ ba. **Không** thêm mục vào `FOCUS_OWNERS` |
| **#4** | **(a)** **Không** dựng diff — mỗi phiên bản hiện **toàn văn** | Không thêm gói, không đi qua cửa NFR15, không đóng quyết định `similar` vs `dissimilar`. Ghi nợ 🟡 — 🔵 chủ là **Story 8.1**, đọc từ `Cargo.toml:86-88`, cụ thể hơn *"Giai đoạn 5"* mà story ghi |
| **#5** | **(a)** Chỉ hiện **thời điểm**, không nhãn nào | Không trỏ tới năng lực chưa tồn tại. Bốn nhãn ghi nợ, **bốn chủ tách rời** |
| **#6** | **(b)** Tương đối + tuyệt đối như mockup, bằng một **hàm thuần** nhận `now` qua **tham số** | Bốn nhánh, bốn khoá `vi.json`, và đây là **quy ước định dạng thời gian đầu tiên của cả kho** *(đo: 0 lời gọi API ngày trên `src/`)*. 🔴 Hàm **không** tự đọc `Date.now()`; **không** `vi.useFakeTimers()` |
| **#7** | **(a)** `CREATE INDEX idx_segment_version_segment_created ON segment_version (segment_id, created_at DESC);` | Khớp đúng hình dạng truy vấn của AC1. Thứ tự hiển thị neo vào **cột mà AC5 nói tới**, không vào một chi tiết cài đặt của SQLite |
| **#8** | **(a)** Khẳng định AC4 bằng **test hợp đồng**, dựng `retired_at` bằng SQL trực tiếp | AC4 đóng **một nửa** 🟡, chủ **Story 2.8**. 🔴 Đường **ĐỌC** và đường **GHI** từ chối khác nhau |

#### 🔵 Một quyết định PHÁI SINH từ #2(a), tôi chốt và nói ra thay vì làm im lặng

Chữ ký #2(a) nói *"hỏi lại"* nhưng **không** nói hỏi **ở đâu**, và ba hình dạng khả dĩ không tương
đương nhau:

- `window.confirm()` — **loại**. Nó là một dialog **chặn** của trình duyệt: không kiểm soát được
  chuỗi *(tức nằm ngoài `vi.json` và ngoài Kiểm A/D của `check:i18n`)*, không token hoá được, và
  `check:layout` Kiểm C là một **danh sách CHO PHÉP** cho mọi thành viên của `window` mà `src/**`
  chạm tới — thêm `confirm` là một quyết định phải viết ra, cho một thứ đã có đường tốt hơn.
- Một **lớp nổi thứ hai** chồng lên lớp nổi #3(a) — loại: đó là tầng `z-index` thứ ba thật sự, và
  UX-DR16 bác lớp nổi chồng lớp nổi.
- ⇒ **Một bước xác nhận NGAY TRONG lớp nổi đã có** *(cùng `role="dialog"`, cùng bẫy `Tab`, cùng
  `Esc`)*. Không bề mặt mới, không `z-index` mới, không thành viên `window` mới, chuỗi nằm trong
  `vi.json`. Đây là đường duy nhất đi lọt cả ba cổng cùng lúc.

### Debug Log References

#### Ⓐ Task 1 — sáu ca đỏ neo vào *"đích là 9"*, đúng như Task 1.10 dự kiến

Nâng đích 9 → 10 làm **sáu** ca đỏ cùng lúc, không ca nào nằm trong danh sách Task 1.5–1.7.
*(2.5c gặp bốn, 2.5d gặp sáu — story này cũng sáu.)* Tất cả cùng một hình dạng: một
`assert_eq!(…schema_version(), 9)`. Sửa **kèm câu khẳng định**, không chỉ sửa con số — một câu
*"bước 8 VÀ 9 phải chạy"* để nguyên sau khi thêm bước 10 là một câu nói thiếu.

| Ca | Sửa |
| --- | --- |
| `a_fresh_project_database_lands_at_the_target_…` | 9 → 10 |
| `a_project_database_at_version_five_migrates_up_…` | 9 → 10, *"bốn bước"* → **năm** bước một lượt |
| `a_project_database_at_version_six_migrates_up_…` | 9 → 10, thêm mệnh đề bước 10 không đụng `status` |
| `a_project_database_at_version_seven_migrates_up_…` | 9 → 10 |
| `a_project_database_at_version_eight_backfills_…` | 9 → 10, thêm mệnh đề bước 10 không đụng một hàng nào |
| `a_project_database_stranded_at_the_burned_version_four_…` | 9 → 10 |

#### Ⓑ 🔵 Fixture "tương lai" là một neo LÚC BIÊN DỊCH, chặt hơn thứ story mô tả

Phép tự kiểm đỏ-rồi-xanh cho hai ca mới lộ ra một tính chất **chưa ai viết ra**: gỡ bước 10
khỏi `PROJECT_MIGRATIONS` **không** cho một ca đỏ — nó cho một **lỗi biên dịch**:

```
error[E0080]: index out of bounds: the length is 8 but the index is 8
```

Vì `STEP_ELEVEN` liệt kê `PROJECT_MIGRATIONS[0..=8]` **tường minh**, số phần tử của bộ di trú bị
neo ở tầng kiểu chứ không ở tầng khẳng định. ⇒ Một lượt **gỡ** bước không bao giờ đi lọt, kể cả
khi ai đó đồng thời sửa hết các `assert`. Doc-comment của fixture nói về *"xanh mà mất ý nghĩa"*
— đó là rủi ro của việc **quên nâng số**, và nó có thật. Nhưng rủi ro **gỡ một bước** thì đã được
đóng chặt hơn hẳn, và không chỗ nào ghi điều đó. Ghi ở đây.

#### Ⓒ Đỏ-rồi-xanh cho hai ca mới — hai đòn bẩy, vì một đòn bẩy không đủ

Luật của kho: *"một cổng chưa bao giờ đỏ là một cổng chưa ai biết nó có chạy không"*.

| Đòn bẩy | `…index_has_the_shape…` | `…version_nine_gains_the_index…` |
| --- | --- | --- |
| Gỡ bước 10 | — **lỗi biên dịch** *(xem Ⓑ)* | — lỗi biên dịch |
| **Đổi chỗ hai cột** `(created_at DESC, segment_id)` | 🔴 **ĐỎ** | ✅ xanh *(đúng vai — nó chỉ khẳng định index **tồn tại**)* |
| **Đổi tên** index → `idx_sai_ten` | 🔴 **ĐỎ** | 🔴 **ĐỎ** |
| Khôi phục nguồn | ✅ xanh | ✅ xanh |

🔴 **Đòn bẩy "đổi chỗ hai cột" là thứ biện minh cho một lựa chọn cài đặt, và nó đáng ghi:** SQL
sau khi đổi chỗ **vẫn chứa** cả `ON SEGMENT_VERSION` lẫn `CREATED_AT DESC`. ⇒ Một phép kiểm bằng
`contains()` trên chuỗi DDL sẽ **XANH trên một index sai thứ tự cột** — tức xanh trên đúng thứ nó
tồn tại để bắt. Đó là lý do ca này đọc `pragma_index_info(...) ORDER BY seqno` thay vì so chuỗi.
Thứ tự cột **là** mệnh đề của Quyết định #7(a), và nó phải được đọc như một danh sách có thứ tự.

Nghiệm thu Task 1: `cargo test --locked` **361/0/5** *(baseline 359 + 2 ca mới)*.

#### Ⓓ Task 2 — ba ca đầu tôi viết SAI ĐƯỜNG, và phép đo chỉ ra chỗ sai là ở CA chứ không ở sản phẩm

Ba ca đọc đầu tiên đỏ với `left: 1, right: 3` — ba lượt ký chỉ sinh **một** phiên bản. Chẩn đoán
sai đầu tiên *("chốt AC13 chặn, phải đi vòng")* bị bác bằng cách đọc mã: đường thật của sản phẩm
là **hai hàm, đúng thứ tự**, và vỏ `wire::save_segment_targets` ghép chúng:

```
unconfirm_edited_segments(...)   ← hạ 'confirmed' → 'draft', so bằng VĂN BẢN (AD-31 §Hợp đồng phụ)
save_segment_targets(...)        ← ghi văn bản, KHÔNG đụng `status` (AD-31 hàng 1)
confirm_segment(...)             ← ký, sinh một `segment_version`
```

🔴 **Hạ-rồi-ghi, không phải ghi-rồi-hạ** — và lý do đã ghi sẵn tại `segment.rs:1357-1363`: sập
giữa chừng ở đường ghi-rồi-hạ để lại văn bản **đã đổi** trên một segment vẫn `'confirmed'` ⇒
không lần xác nhận nào nữa xảy ra ⇒ cặp TM mới không bao giờ được ghi, im lặng vĩnh viễn. Đúng
hố (2) của AD-31 §Prevents.

⇒ Ba ca đỏ vì **chính chúng** đi sai đường, không vì sản phẩm hỏng. Đây đúng bài học ⑤ của story
*("đọc mã mà xác nhận từng tiền đề, đừng thi hành story như một mệnh lệnh")* — áp cho cả thứ
**tôi tự viết ra**, không chỉ cho thứ story viết sẵn.

#### Ⓔ 🔴 Một phép đo lật một dòng tôi định viết như trang trí: vế gỡ hoà `id DESC` CHỊU LỰC

Câu `SELECT` của đường đọc sắp theo `created_at DESC, id DESC`. Vế thứ hai **suýt** được ghi vào
như một dòng phòng xa. Đo trước khi tin:

**Bàn đo tạm** *(12 lượt ký liên tiếp trên cùng một segment, `tests/tmp_gap.rs`, đã xoá sau khi
đọc số — nó là tạo tác của một lượt đo, không một công cụ)*:

```
2026-08-16T02:10:55.850Z   …854Z   …856Z   ← trùng
2026-08-16T02:10:55.851Z   …855Z   …856Z   ← trùng
…
SO MOC KHAC NHAU: 11 tren 12 luot ky
```

⇒ **Hai lượt ký rơi trúng cùng một mili giây trong vòng mười hai lượt.** `strftime('%f')` chính
xác tới mili giây và một lượt ký mất **~1 ms**, nên va chạm không phải khả năng lý thuyết. SQLite
**không bảo đảm** thứ tự các hàng bằng nhau ở cột sắp ⇒ thiếu `id DESC`, thứ tự của cặp hàng đó
**không tất định** và AC1 nói sai ở đúng cặp đó.

🔴 **Và bốn ca đọc kia KHÔNG canh mệnh đề này** — đo được: gỡ `id DESC`, chạy `the_history_command_returns…`
**8 lượt, 8/8 xanh**. Một ca dựa vào việc đồng hồ *tình cờ* va chạm là một ca chập chờn: xanh giả
ở đa số lượt chạy, chỉ đỏ khi máy đủ nhanh. ⇒ Ca thứ năm dựng va chạm bằng **SQL trực tiếp**
*(ba hàng cùng mốc `…55.856Z`)*, tất định. Va chạm phải được **dựng**, không được **chờ**.

⚠️ Vế `id DESC` cố ý **không** vào index của bước 10: index phục vụ phép lọc và phép sắp chính;
một cột thứ ba chỉ để gỡ hoà cho vài hàng trùng mili giây là một cái giá thường trực cho một ca
hiếm. SQLite sắp nốt phần còn lại trong bộ nhớ, trên một tập **đã** được index thu hẹp về một
`segment_id`.

#### Ⓕ Đỏ-rồi-xanh cho năm ca của Task 2

| Đòn bẩy | Ca đỏ |
| --- | --- |
| Gỡ `, id DESC` khỏi `ORDER BY` | 🔴 `two_versions_sharing_a_millisecond…` — và thứ tự trả về **ngược hẳn**: `["som nhat", "giua", "muon nhat"]` thay vì đảo lại |
| *(cùng đòn bẩy)* | ✅ bốn ca kia xanh — **đúng vai**, chúng nói về mệnh đề khác |
| Khôi phục | ✅ cả năm xanh |

Nghiệm thu Task 2: `cargo test --locked` **366/0/5** *(361 + 5 ca mới)*.

#### Ⓖ Task 3 — hình dạng của đường khôi phục, và HAI chỗ đặc tả không nói mà mã phải nói

Chữ ký #2(a) nói *"hỏi lại"*, và hai câu hỏi đi kèm không có trong đặc tả:

**① *"Chưa từng được ký"* đo bằng gì?** Không phải bằng `status`, và không phải bằng một cờ
`dirty` *(AD-31 §Hợp đồng phụ cấm)*. Phép so đúng là: **văn bản hiện tại có một bản sao trong
`segment_version` không.** Vì thứ đáng lo không phải *"đã sửa hay chưa"* mà *"cái sắp mất có bản
sao ở đâu không"*.

⇒ Nó cũng là phép so **đúng hơn** chứ không chỉ hợp lệ: người dùng gõ một thứ khác rồi hoàn tác
về đúng một bản đã ký thì **không mất gì**. Cờ `dirty` hỏi thừa ở đúng ca đó — và một hộp thoại
hỏi thừa là thứ làm người dùng bấm *"đồng ý"* theo phản xạ, tức làm chốt thật mất tác dụng. Ca
`text_that_still_has_a_copy_in_the_history_does_not_trigger_the_confirmation_hold` khoá chỗ hai
cách rẽ nhau.

**② Một lượt "giữ lại" là một LỖI hay một KẾT QUẢ?** Chọn **kết quả** — `RestoreOutcome` mang
`needs_confirmation: bool` + `unsigned_draft: Option<String>`, và `force: bool` ở lượt gọi thứ
hai. Ba lý do, không một sở thích:
- một `IpcError` sẽ chảy vào **ô lỗi dùng chung** đang hiện một **chuỗi cố định** *(món nợ
  `:2825-2840`, chủ Ice)* — tức thông điệp quan trọng nhất của story này sẽ hiện ra sai;
- chốt chạy **trong cùng giao dịch** với lượt ghi ở lượt gọi thứ hai, nên không có cửa sổ
  kiểm-rồi-ghi cho người dùng gõ chen vào;
- `unsigned_draft` mang **chính bản nháp** ra ngoài để webview **hiện nó ra**, thay vì chỉ nói
  *"có thứ sẽ mất"*.

🔴 **`restored = false` và `needs_confirmation = true` phải phân biệt được** — một cái là *"không
có gì để làm"*, cái kia là *"đang chờ bạn"*. Gộp vào một `bool` là dựng lại đúng lớp lỗi mà
`ConfirmOutcome::version_created` tồn tại để tránh.

🔴 **Và `force` KHÔNG bỏ qua hàng rào quyền sở hữu** — nó chỉ bỏ qua chốt chống mất bản nháp. Gộp
hai thứ đó làm một là biến một lời xác nhận của người dùng thành một **giấy phép ghi bất kỳ đâu**.
Ca crossover khẳng định cả hai chiều (`force = false` **và** `force = true`).

#### Ⓗ Một ca biên AC2 không nêu, và tôi chọn NGƯỢC với hướng "cho nhất quán"

Khôi phục về **đúng nội dung đang có**: không ghi gì, và 🔴 **không hạ `status`**. Khuôn AC13 của
`confirm_segment` *(ký lại một câu đã ký ⇒ `Ok(false)`)*. Vế không-hạ mới là vế đắt: một segment
đang `'confirmed'` bị hạ xuống `'draft'` vì người dùng bấm khôi phục lên **chính bản đang dùng**
là một lượt **huỷ chữ ký của họ mà không đổi lấy gì** — họ phải ký lại một câu chưa hề đổi.

⚠️ Đọc AC2 theo nghĩa đen *("trạng thái segment về chưa xác nhận")* sẽ cho đường ngược lại. Tôi
đọc AC2 là mệnh đề về một lượt khôi phục **có thật sự đổi văn bản**, và ghi lựa chọn đó ra đây
thay vì để nó nằm im trong mã.

#### Ⓘ Đỏ-rồi-xanh cho sáu ca của Task 3 — bốn đòn bẩy, bốn mệnh đề mang chữ ký

| Đòn bẩy *(mỗi cái là một lượt "sửa cho hợp lý" có thật)* | Ca đỏ | Số |
| --- | --- | --- |
| Thêm một `INSERT segment_version` vào đường khôi phục *(= hàng thứ **bảy** của AD-31)* | `restoring_rewrites_…_without_growing_the_history` | `left: 4, right: 3` |
| Gỡ hàng rào `AND segment_id = ?2` | `a_version_belonging_to_another_segment_…` | ghi đè xuyên segment |
| Gỡ chốt chống mất bản nháp | `restoring_over_an_unsigned_draft_…` | không giữ lại |
| Hạ `status` ở ca vô hại | `restoring_to_the_text_already_in_place_…` | `left: "draft", right: "confirmed"` |
| Khôi phục nguồn | cả sáu ✅ xanh | |

🔴 Đòn bẩy thứ nhất là đòn bẩy quan trọng nhất của cả story: nó làm mockup **đúng** và AD-31
**sai** cùng lúc, và nó là một lượt sửa **hai dòng**. Ca đó là thứ buộc một lượt như vậy đi qua
thủ tục viết một `AD` mới thay vì một lượt tiện tay.

Nghiệm thu Task 3: `cargo test --locked` **372/0/5** *(366 + 6 ca mới)*.

#### Ⓙ 🔴 Task 5 — chữ ký #3(a) va vào AD-34 §1, và cách gỡ là một khuôn ĐÃ CÓ

Bản dựng đầu của lớp phủ có ba nút mang `@click="onRestore(version.id)"` kiểu. `check:commands`
Kiểm A **đỏ ba chỗ**: mọi `@click` phải là **đúng một** `dispatch('<id>')` với id **literal**.

Đây không phải một cổng phiền — nó là AD-34 §1, và hai luật của kho **thoạt nhìn xung khắc** ở
đúng chỗ này:

| Luật | Nó cấm gì |
| --- | --- |
| **AD-34 §1** *(Kiểm A)* | `@click` mang tham số ⇒ **không** truyền được `version.id` qua nút |
| **§KHÔNG-LÀM** | một command cho mỗi hàng ⇒ **không** sinh được id theo `version_id` *(nó phá `COMMAND_FLOOR`, và Story 1.21 không gán lại phím cho một id chưa tồn tại lúc dựng bảng)* |

⇒ Khuôn giải đã có và đã chạy: **`aimedShortcutRow` của Story 1.21**. Hàng được **nhắm** bằng
`@mousedown`/`@focusin` — Kiểm A nói nguyên văn *"chỉ `@click`"*, nên hai sự kiện đó được xử lý
tự do — rồi **ba command không tham số** đọc mục tiêu **lúc chạy**:
`history.restore` · `history.confirm_restore` · `history.cancel_restore`.

🔴 **Một chi tiết của khuôn này KHÔNG chép được từ Story 1.21, và nó là một lỗ bảo mật nhỏ nếu
chép:** `confirmPendingRestore()` đọc `versionId` từ **chính lượt đang chờ**, **không** từ hàng
đang nhắm. Người dùng có thể đã nhắm sang hàng khác trong lúc đọc câu hỏi — khôi phục **hàng
khác** với lời đồng ý dành cho hàng này là biến một lời xác nhận thành một giấy phép ghi bất kỳ
đâu. Có ca riêng khẳng định *(nhắm sang 202 giữa chừng, vẫn phải khôi phục 201)*.

#### Ⓚ Ba text node cần miễn trừ có tên, và hai loại lý do khác nhau

`check:i18n` Kiểm A đỏ ba chỗ trong `.vue` mới. Cả ba hợp lệ, nhưng **hai lý do khác hạng**:

| Chỗ | Lý do miễn trừ |
| --- | --- |
| `historyPendingRestore.draft` · `version.target_text` | **DỮ LIỆU NGƯỜI DÙNG** — chính bản dịch họ gõ. Phải hiện nguyên văn; đẩy qua `t()` là vô nghĩa **và sẽ hỏng ở ký tự `{`** *(nó rơi vào dải nội suy placeholder)* |
| `labelFor(version.created_at)` | **KẾT QUẢ của `t()`** — helper chỉ chọn nhánh rồi gọi `t(key, params)`. Bốn khoá `history.time_*` nằm trong `vi.json` và Kiểm E chạy trên chúng |

⚠️ Cái bẫy *"Kiểm A báo FAIL SAI CHỖ khi một tên thẻ được nhắc trong comment của template"* mà
story ghi sẵn **không** kích hoạt lượt này — cả ba FAIL đều báo đúng dòng.

#### Ⓛ Đỏ-rồi-xanh cho hai mệnh đề mà CHỈ tầng frontend canh được

| Đòn bẩy | Ca đỏ |
| --- | --- |
| Gỡ `await flushEditorNow()` khỏi `restoreVersion` | 🔴 `④ gõ chưa flush ⇒ lượt flush đi TRƯỚC lượt khôi phục` |
| Đổi `replaceEditorSegment` sang `Object.assign` **tại chỗ** | 🔴 `② khôi phục ⇒ editorSegments thay bằng mảng mới` |
| Khôi phục | ✅ cả 16 ca xanh |

🔴 Mệnh đề flush-trước **không cưỡng chế được ở tầng Rust** — lệnh khôi phục chỉ đọc thứ đã ở
trên đĩa và không biết gì về văn bản đang gõ trong webview. Ca đó là **lưới duy nhất**, và nó
khẳng định **thứ tự trên dây** (`['flush', 'restore']`) chứ không chỉ *"có gọi flush"*.

#### Ⓜ Task 6.8 — `COMMAND_FLOOR` 33 → 35, và khuôn này lặp lại lần thứ BA

Đo lại chứ không chép: **41** command thật *(39 → 41)*. Sàn nâng 33 → **35** (85,4 %, khớp tỷ lệ
33/39 = 84,6 % của lượt trước). Giữ 33 trên 41 là 80,5 % — tám command có thể biến mất mà cổng
vẫn xanh.

🔴 Ba story liên tiếp *(2.5c · 2.5d · 2.6)* đều phải nâng con số này **bằng tay**, và **không
cổng nào canh chính cái sàn đó**. Thứ duy nhất nhắc là dòng chú thích ngay trên nó. Đã ghi thẳng
điều đó vào chú thích thay vì để lượt thứ tư phát hiện lại.

#### Ⓠ Một lượt tự rà bắt được MÃ CHẾT do chính tôi vừa viết, kèm một lời biện minh yếu

Rà cuối: `reloadHistory()` được **export mà không ai gọi**. Tôi đã viết sẵn một doc-comment biện
minh cho nó — *"nó vẫn ở đây vì nó giữ cho bề mặt đúng nếu Ice đổi chữ ký #1 sau này"*.

🔴 Đó **đúng là** lời biện minh mà kho đã từ chối **hai lần**, và `store_contract.rs` ghi nguyên
văn: *"thêm một hàm `pub` vào mã sản phẩm mà chỉ test gọi — mã không ai dùng, đúng thứ Story 1.5
và 1.6 đã từ chối hai lần"*. Với chữ ký **#1(a)** thì một lượt khôi phục **không** làm lịch sử
dài thêm, nên hàm đó không có việc gì để làm hôm nay; *"nếu Ice đổi chữ ký sau này"* là một
tương lai giả định, không một chỗ gọi.

⇒ **Gỡ.** Cùng lượt gỡ một đường vòng: `segmentHistoryState.ts` đang **re-export**
`editorSegments` chỉ để bề mặt import lại từ đó. Bề mặt nay import **thẳng** từ
`editorPanelState`.

⚠️ **Không cổng nào bắt được cả hai** — `check:lint` xanh trước và sau, vì cả hai là *export* chứ
không phải biến cục bộ không dùng. Đây là cùng hình dạng với món nợ đã ghi cho
`editorOmitError`/`editorConfirmError` *(export mà chỉ một chỗ đọc)*. Thứ bắt được nó là một lượt
đọc lại bằng mắt, và tôi ghi ra để lượt rà sau biết chỗ này đã được hỏi.

#### Ⓟ Task 10.3–10.4 — e2e trên WKWebView thật, và cổng 4445 đúng là đang bị chiếm

**Cổng:** 1420 trống. 4445 **bị chiếm** — `gdrive-su` PID **6703** *(story ghi PID 48486 ngày
2026-08-15; cùng chương trình, phiên khác)*. Theo đúng Task 10.4: đặt `TAURI_WEBDRIVER_PORT=4455`,
**không** giết tiến trình của Ice. Đã kiểm plugin có đọc biến đó thật —
`tauri-plugin-wdio-webdriver-1.3.0/src/lib.rs:24` khai `PORT_ENV_VAR`.

⚠️ **Và một bộ e2e KHÁC của Ice đang chạy cùng lúc trên máy này** — `gdrive_suite_manager` (PID
6686, `wdio run e2e/wdio.conf.ts`). Nó chính là thứ giữ 4445. Hai bộ chạy song song **không**
đụng nhau sau khi đổi cổng, nhưng ghi ra vì nó là bối cảnh của mọi số dưới đây.

**Kết quả — hai ca của Story 2.6 XANH cả hai:**

```
» e2e/specs/segment-history-restore.e2e.mjs
   ✓ ký → sửa → ký lại → đọc lịch sử → khôi phục → đọc lại từ đĩa
   ✓ chốt chống mất bản nháp giữ lượt ghi lại, và KHÔNG một byte nào xuống đĩa
   2 passing (51.7s)
```

🔴 **Đây là đường DUY NHẤT đo được mệnh đề trung tâm**: bốn trường của `SegmentVersionRow` có
thật trên dây, không `undefined`. Cả 130 ca vitest đều **không** bắt được lớp lỗi đó — fixture
chép tay có sẵn bốn trường, đúng như 74/74 đã xanh trên một sản phẩm hỏng ở Story 2.5.

Ca thứ hai đo trọn chốt chống mất bản nháp **trên đĩa thật**: lượt gọi đầu trả
`needs_confirmation` + `unsigned_draft`, `read_open_chapter_segments` đọc lại thấy đĩa **còn
nguyên**, lượt thứ hai với `force` mới ghi.

**🔴 BA lượt chạy, và tôi ghi cả ba chứ không chỉ lượt xanh:**

| Lượt | Kết quả | Đọc thế nào |
| --- | --- | --- |
| ① trọn bộ | **7 passed, 1 failed** / 8 spec (9m00) | hai ca của 2.6 **xanh**; spec đỏ **không xác định được** — tôi đã `tail -45` output nên phần đầu bị cắt. Một lỗi của **cách tôi chạy**, không của bộ đo |
| ② bốn spec đầu, chạy riêng | **4 passed, 4 total** (6m01) | `attribution-focus` · `editor-confirm-segment` · `editor-typing-flush` · `grid-empty-cell` — cả bốn xanh |
| ③ trọn bộ, giữ trọn output | **8 passed, 8 total · 11/11 ca** (9m40) | không spec nào đỏ |

⇒ Lượt đỏ ở ① là **chập chờn của bàn đo**, không một khuyết tật sản phẩm — và nó khớp đúng khuôn
hai món nợ **đã ghi từ trước**: *"fixture e2e không reset state panel giữa các spec"*
(`:3093-3115`, chủ Story 1.22) và *`devServerIsUp()` tin một Vite hấp hối* (`:3274-3330`, chưa vá).

⚠️ **Nhưng tôi KHÔNG chấm nó là "đã chẩn đoán"** — luật của kho ghi bằng chữ sau lượt 1.22:
*"gặp một luợt đỏ không tái lập được thì BẮT NGUYÊN VĂN TRƯỚC"*. Tôi **không** bắt được nguyên
văn, vì chính lệnh của tôi đã cắt nó. Thứ đo được là: *"cùng bộ spec đó cho 8/8 ở lượt kế, và
bốn ứng viên đều xanh khi chạy riêng"* — đủ để đi tiếp, **không** đủ để đặt tên nguyên nhân.
🔵 Bài học cho lượt sau: **đừng `tail` output của một lượt e2e**; ghi trọn ra tệp rồi lọc.

#### Ⓞ Task 10.5 — NFR2: đường dời con trỏ **không** mọc thêm một phép tính nào, và đây là một lập luận CẤU TRÚC chứ không một phép đo

Task 10.5 buộc đo lại **nếu** Task 5 thêm tính toán chạy trên toàn danh sách mỗi lượt dời con
trỏ. Rà bằng cách đọc mã:

| Thứ | Phụ thuộc | Có chạy khi dời con trỏ không |
| --- | --- | --- |
| `SegmentHistoryOverlay` | `v-if="historyIsOpen"` | **không** — đóng thì không render gì |
| `currentText` *(computed, O(n) `find`)* | `historySegmentId` · `editorSegments` | **không** — `historySegmentId` chỉ đổi lúc **mở** lớp phủ |
| `openSegmentHistory()` | đọc `editorCaretSegmentId` | **không** — đọc **mệnh lệnh, một lần**, trong thân hàm; **không** trong một `computed`/`watch`, nên nó **không** dựng một phụ thuộc phản ứng lên caret |

⇒ **0** phép tính mới trên đường dời con trỏ. 🔴 Và tôi ghi đây là một lập luận **cấu trúc**,
không một phép đo — luật của kho cấm chấm đạt bằng suy luận, nên câu đúng là: *"Story 2.6 không
thêm gì vào đường đó"*, **không** phải *"đường đó đã đạt NFR2"*.

⚠️ Món nợ NFR2 đã đo của 2.5b — **706–770 ms** trên 9.850 câu, vượt trần 50 ms/frame **~15
lần** — **không đổi và vẫn còn hở**, chủ vẫn là **Story 2.4**. Story này không chạm nó theo cả
hai chiều.

⚠️ Task 10.6 *(đo lại chiều cao hàng)* **không kích hoạt**: nó chỉ áp cho đường #3(c) *(mở
trong chính hàng của lưới)*, mà Ice ký **#3(a)**. Lớp phủ nằm ngoài `subgrid`, không đụng một
track nào.

#### Ⓝ Task 6.10 — rà tay `e2e/**`, và nó lộ ra một thứ khác

`check:commands` **không đọc `e2e/**`**, nên command id nằm cứng ở đó không cổng nào canh. Rà
tay: **0** id `history.*` trong `e2e/` — không có gì phải sửa.

⚠️ Nhưng lượt rà bắt được `layout.toggle_source` trong hai tệp e2e — một command **đã thôi tồn
tại** ở Story 2.5b. Kiểm lại: cả hai chỗ là **chú thích đã mang dấu 🔵** ghi rõ lượt đổi tên.
Không phải một khuyết tật sống. Ghi ra vì lượt rà tiếp theo sẽ gặp lại đúng hai dòng đó.

### Completion Notes List

#### Điều story này thật sự giao

Bước di trú **10** *(index trên `segment_version`)* · một đường **đọc** lịch sử · một đường
**khôi phục** · một lớp phủ cấp `App` · năm command · quy ước định dạng thời gian **đầu tiên**
của kho. Tám quyết định mở đều có chữ ký của Ice trước dòng mã đầu tiên, và **#1(a) là thứ giữ
cho story không phải dừng lại viết một `AD` mới**.

#### 🔴 Bốn thứ PHÉP ĐO tìm ra mà không tài liệu nào nói trước

**① Một phép `grep` tiền đề tự bắt chính câu nói về nó.** Task 0.6 đo `merge_segment` và nhận
**1** kết quả thay vì 0 — dòng khớp là một **doc-comment** viết nguyên văn *"grep … cho 0"*. Số
thật vẫn là 0 đường mã, nhưng một lượt đọc số thô kết luận ngược, và kết luận ngược đó chặn đúng
một quyết định (#8). ⇒ Luật rút ra: **đọc nội dung dòng khớp, đừng đếm.** Kho này ghi kết quả đo
vào chú thích rất dày, nên lớp bẫy này sẽ gặp lại.

**② Vế gỡ hoà `id DESC` CHỊU LỰC, và bốn ca đọc không canh được nó.** Bàn đo 12 lượt ký liên
tiếp: **11 mốc `created_at` khác nhau** — hai lượt rơi trúng cùng một mili giây. SQLite không bảo
đảm thứ tự các hàng bằng nhau ⇒ thiếu vế đó, AC1 nói sai ở đúng cặp hàng đó. 🔴 Và gỡ nó ra thì
bốn ca đọc chạy **8/8 xanh** — một ca dựa vào việc đồng hồ *tình cờ* va chạm là một ca chập chờn.
⇒ Ca thứ năm **dựng** va chạm bằng SQL, tất định.

**③ Fixture "tương lai" là một neo LÚC BIÊN DỊCH**, chặt hơn thứ doc-comment của nó mô tả: gỡ
một bước khỏi `PROJECT_MIGRATIONS` cho `error[E0080]: index out of bounds`, không một ca đỏ. Rủi
ro **quên nâng số** là thật và đã ghi; rủi ro **gỡ một bước** thì đã đóng chặt hơn hẳn, và không
chỗ nào ghi điều đó.

**④ Một phép so chuỗi DDL sẽ XANH trên một index sai thứ tự cột.** Đổi chỗ hai cột, SQL vẫn chứa
cả `ON SEGMENT_VERSION` lẫn `CREATED_AT DESC`. ⇒ Ca hình dạng đọc `pragma_index_info(...) ORDER
BY seqno`. Thứ tự cột **là** mệnh đề của Quyết định #7(a).

#### 🔴 Ba chỗ tôi đọc AC theo nghĩa HẸP HƠN nghĩa đen, và nói ra thay vì làm im lặng

- **AC2 "trạng thái về chưa xác nhận"** — tôi đọc là mệnh đề về một lượt khôi phục **có thật sự
  đổi văn bản**. Khôi phục về đúng nội dung đang có thì **không** hạ `status`: hạ một chữ ký mà
  không đổi lấy gì là huỷ công của người dùng.
- **Chữ ký #2(a) "hỏi lại"** không nói hỏi **ở đâu**. `window.confirm()` bị loại *(chuỗi nằm
  ngoài `vi.json`, và `check:layout` Kiểm C là danh sách CHO PHÉP cho mọi thành viên `window`)*;
  một lớp phủ thứ hai cũng bị loại. ⇒ Một bước xác nhận **ngay trong** lớp phủ đã có.
- **`force`** chỉ bỏ qua chốt chống mất bản nháp, **không** bỏ qua hàng rào quyền sở hữu — gộp
  hai thứ đó là biến một lời xác nhận thành giấy phép ghi bất kỳ đâu. Có ca riêng cho cả hai giá
  trị của `force`.

#### Đỏ-rồi-xanh: mười một đòn bẩy, không một ca nào chưa từng đỏ

Mỗi mệnh đề mang chữ ký đều có ít nhất một đòn bẩy làm nó đỏ, rồi khôi phục cho xanh lại. Nặng
nhất là **thêm một `INSERT` vào đường khôi phục** *(= hàng thứ bảy của AD-31)* — một lượt sửa
hai dòng làm mockup **đúng** và AD-31 **sai** cùng lúc. Ca đó là thứ buộc một lượt như vậy đi qua
thủ tục viết một `AD` mới. Chi tiết ở §Debug Log Ⓒ · Ⓕ · Ⓘ · Ⓛ.

#### 🟡 Ba AC đóng MỘT NỬA, cả ba ghi nợ có chủ — không tự chấm đạt

| AC | Vế đóng | Vế còn hở | Chủ |
| --- | --- | --- | --- |
| **AC4** | *"lịch sử vẫn tra lại được"* — test hợp đồng, `retired_at` dựng bằng SQL | **bề mặt vào**: không đường mã nào cho một segment về hưu *(`merge_segment` = 0, Story 2.8 `backlog`)* | Story 2.8 |
| **AC1** vế nhãn | thời điểm, mới nhất trước | bốn nhãn của mockup trỏ vào bốn năng lực chưa dựng | 2.7 · Epic 4 · Epic 7 · Epic 8 |
| **AC2** vế cờ đoạn | `target_text` + `status` | `is_target_paragraph_end` **không** khôi phục được — `segment_version` không lưu cờ | **Ice** *(quyết định ngữ nghĩa)* |

#### ⚠️ Hai món nợ MỚI mà chính lượt này đẻ ra

- **Ca `toISOString()` RỖNG NGHĨA trên CI.** Nó bắt được cái bẫy trên máy của Ice (UTC+7) —
  đỏ-rồi-xanh đã chạy — và ca được viết để **tự chọn chiều** theo offset. Nhưng ở UTC đúng cả hai
  chiều biến mất, và runner GitHub chạy UTC. Ca **tự khai** điều đó bằng một nhánh
  `expect(offsetMin).toBe(0)` thay vì giả vờ đã đo.
- **`src/config/segment.ts` nay có HAI loại adapter** — sáu cái tin payload, hai cái kiểm nó lúc
  chạy. Lý do lệch là một lớp lỗi **đã xảy ra thật**, nhưng lý do đó áp cho **cả sáu** cái kia y
  hệt. Câu hỏi là một câu hỏi **quy ước**, chủ **Ice**.

### File List

Kê từ `git status --porcelain`, **không** từ trí nhớ.

**Sửa (15)**

| Tệp | Việc |
| --- | --- |
| `src-tauri/src/core/store/schema.rs` | hằng `SEGMENT_VERSION_INDEX_DDL` + bước di trú **10**; 🔵 cập nhật doc-comment `PROJECT_MIGRATIONS` *(tám bước/đích 9 → chín bước/đích 10)* và doc-comment bước 7 *(mệnh đề "không index" nay chỉ đúng về hằng, không về lược đồ)* |
| `src-tauri/src/commands/segment.rs` | DTO `SegmentVersionRow` · `RestoreOutcome` · `enum RestoreReject`; hàm thuần `read_segment_history` · `restore_segment_version`; hai vỏ `wire` |
| `src-tauri/src/lib.rs` | đăng ký hai command mới |
| `src-tauri/tests/segment_contract.rs` | fixture `STEP_TEN` → `STEP_ELEVEN`; bậc thang `[…,10]`; **sáu** ca neo vào đích 9 nâng lên 10; **13 ca mới** |
| `src-tauri/tests/pinned_contract.rs` | hai neo: `len()` 8 → 9, `schema_version()` 9 → 10 |
| `src/config/segment.ts` | kiểu `SegmentVersion` · `RestoreOutcome` + hai kết quả ba trạng thái; hai adapter; guard `isSegmentVersionArray` |
| `src/commands/index.ts` | năm cổng `CommandDeps`; **năm** command `history.*` |
| `src/main.ts` | cắm năm handler |
| `src/App.vue` | mount `SegmentHistoryOverlay` |
| `src/panels/editorPanelState.ts` | `replaceEditorSegment()` — cửa hẹp mang luật *"mảng mới"* |
| `src/i18n/vi.json` | 5 nhãn lệnh + 14 chuỗi giao diện *(gồm bốn khoá `history.time_*`)* |
| `src/panels/README.md` | §Lịch sử phiên bản segment |
| `scripts/check-commands.mjs` | `COMMAND_FLOOR` 33 → **35** |
| `_bmad-output/implementation-artifacts/deferred-work.md` | đóng **hai** món có chủ 2.6 *(nối tiếp, không xoá)* + **chín** món mới/giao lại |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | trạng thái story |

**Mới (7)**

| Tệp | Vai |
| --- | --- |
| `src/SegmentHistoryOverlay.vue` | lớp phủ cấp `App` |
| `src/panels/segmentHistoryState.ts` | state cấp module + định tuyến |
| `src/panels/segmentHistoryTime.ts` | hàm thuần định dạng thời điểm |
| `tests/frontend/segmentHistory.test.ts` | 16 ca — trạng thái webview |
| `tests/frontend/segmentHistoryTime.test.ts` | 11 ca — bốn nhánh, tất định |
| `e2e/specs/segment-history-restore.e2e.mjs` | vòng trọn trên WKWebView thật |
| `_bmad-output/implementation-artifacts/2-6-….md` | chính story này |

⚠️ **`ARCHITECTURE-SPINE.md` KHÔNG bị chạm một dòng** — đo bằng `git diff --stat` trên
`planning-artifacts/`: **rỗng**. Đó là hệ quả trực tiếp của chữ ký #1(a), và nó là thứ đáng
kiểm chứ không đáng giả định: hai đường ký còn lại đều đẻ ra một `AD` mới.

### Change Log

| Ngày | Việc |
| --- | --- |
| 2026-08-16 | `create-story` — tám quyết định mở; một **mâu thuẫn đo được** giữa AD-31 và mockup về việc khôi phục có ghi phiên bản không; một **lỗ mất dữ liệu** không AC nào nêu *(bản nháp chưa ký bị ghi đè)*; và mockup vẽ một diff mà hai crate diff **cố ý chưa cài** |
| 2026-08-16 | `dev-story` Task 0 — Ice ký **tám** quyết định: #1(a) · #2(a) · #3(a) · #4(a) · #5(a) · #6(b) · #7(a) · #8(a). **#1(a) là chữ ký gỡ cửa chặn**: AD-31 không sửa một chữ ⇒ Task 0.4 không kích hoạt ⇒ không `AD` mới. Baseline khớp cả hai *(cargo 359/0/5 · vitest 103/103)*, bốn tiền đề khớp cả bốn — nhưng ③ lộ ra một **bẫy đo**: grep tự bắt chính câu nói về grep |
| 2026-08-16 | Task 1 — bước di trú **10**. Nâng đích 9 → 10 làm **sáu** ca đỏ ngoài danh sách *(đúng như Task 1.10 dự kiến; 2.5c gặp bốn, 2.5d gặp sáu)*. Fixture "tương lai" nâng 10 → **11**, lượt lặp lại **thứ ba**. Phát hiện: fixture đó là một neo **lúc biên dịch**, chặt hơn thứ doc-comment của nó mô tả |
| 2026-08-16 | Task 2 — đường **đọc**. Ba ca đầu tôi viết **sai đường**, và phép đo chỉ ra chỗ sai ở **ca** chứ không ở sản phẩm: đường thật là `unconfirm` → `save` → `confirm`, **hạ trước ghi sau**. Một phép đo lật một dòng tôi định viết như trang trí: vế gỡ hoà `id DESC` **chịu lực** — 11 mốc khác nhau trên 12 lượt ký |
| 2026-08-16 | Task 3 — đường **khôi phục**. Không `INSERT` *(chữ ký #1(a))*; chốt chống mất bản nháp so bằng *"văn bản này có bản sao chưa"* chứ không cờ `dirty`; ca vô hại **giữ chữ ký** thay vì hạ nó |
| 2026-08-16 | Task 4–8 — dây IPC, lớp phủ, năm command, quy ước thời gian đầu tiên của kho, 27 ca vitest. Chữ ký #3(a) **va vào AD-34 §1**, gỡ bằng khuôn `aimedShortcutRow` của Story 1.21 *(nhắm bằng `@mousedown`/`@focusin`, ba command không tham số)*. `COMMAND_FLOOR` 33 → **35** |
| 2026-08-16 | Task 9 — đóng **hai** món nợ có chủ 2.6 bằng cách **nối tiếp**. Món thứ hai đóng kèm một **vế mới**: `restore_segment_version` **CÓ** đụng `updated_at` *(nó sửa chữ thật)*, ngược `confirm_segment` — hai lệnh ghi, hai hành vi ngược nhau trên cùng một cột, và cả hai đúng |

### Review Findings

Rà ba tầng *(Blind Hunter · Edge Case Hunter · Acceptance Auditor)* — 2026-08-16. 10 phát hiện thô, **9 giữ**, 1 bác. Mọi mục dưới đây đã được đối chứng bằng cách **đọc mã thật**, không chấm từ hunk.

🔵 **Cả ba tầng độc lập nhau đều chỉ vào cùng một chỗ nặng nhất**, và ba tầng cũng đồng ý một điều thứ hai: **đường Rust không có khuyết tật nào**. Mọi mục dưới đây nằm ở tầng điều phối TypeScript — đúng tầng **không** có lưới tương đương bộ 13 ca hợp đồng của Task 2–3.

- [x] [Review][Patch] **`.hist-current` là chỉ dấu *"đang dùng"* suy từ phép so văn bản — chính phép so mà Quyết định #5 gọi là KHÔNG AN TOÀN** — 🔵 **Ice chốt 2026-08-16: GỠ**, và ghi vế *"hàng nào đang dùng"* vào `deferred-work.md` kèm chủ **Story 2.7**, cùng chỗ với bốn nhãn kia. — `SegmentHistoryOverlay.vue:265` gắn class theo `version.target_text === currentText`, `:409-412` vẽ nó thành viền trái `--color-primary`. Chữ ký **#5(a)** là *"hiện thời điểm và **không nhãn nào**"*, và bảng của chính Quyết định #5 mô tả phép so này là *"suy được, **nhưng không an toàn** — hai phiên bản trùng văn bản thì nhãn khớp **nhiều** hàng"*; đường an toàn là (b), chưa dựng. Comment tại chỗ tự nhận biết điều đó *("nó cố ý không mang chữ")* — tức lập luận là **chữ thì cấm, màu thì không**. Không mục nào trong `deferred-work.md` ghi rằng đường (a) đã bị lách một phần. 🔴 Ba câu trùng bản dịch *(rất thường: ký → sửa → hoàn tác về bản cũ)* làm **nhiều hàng** cùng sáng viền, và người dùng không có cách nào biết hàng nào thật sự đang dùng. Ice chốt: ① gỡ `hist-current`, ghi nợ chủ 2.7 · ② giữ và ghi nợ tường minh · ③ đổi sang đường (b)
- [x] [Review][Patch] **Chốt chống mất bản nháp thi hành ĐÚNG MỘT NỬA — doc-comment khai hai lượt flush, mã chạy một** [src/panels/segmentHistoryState.ts:230-240] — comment `:230-234` viết nguyên văn *"Khuôn hai lượt flush của `confirmCurrentSegment` (Quyết định #8, Ice ký 2026-08-14) **áp nguyên ở đây**… ⇒ Thử lại **đúng một** lượt; còn dơ nữa thì **từ chối**"*. Mã `:240` là `if ((await flushEditorNow()) === 'failed') return 'flush-failed'` — không `flush.isDirty()`, không lượt thứ hai, không đường từ chối, và `RestoreResult` không có biến thể `'still-dirty'`. Khuôn thật ở `editorPanelState.ts:716-728` có đủ ba nhịp. 🔴 Đường hỏng: nhánh *originator* của `flushEditorNow` chụp `snapshot` **trước** lượt IPC và **không** đệ quy ⇒ ký tự gõ trong lúc lô đang bay nằm ngoài ảnh chụp, `flushEditorNow` vẫn trả `'saved'`, và chốt ở Rust so trên **đĩa cũ hơn thứ đang trên màn hình** ⇒ nó **không hỏi** ở đúng ca cần hỏi nhất. Đây là lỗ mất dữ liệu mà chữ ký **#2(a)** tồn tại để mua, mất trong im lặng — và là khuôn *"chữ ký thi hành đúng một nửa"* mà Dev Notes bài học ② của chính story này cảnh báo, lặp lần thứ tư. Ca `④` của `segmentHistory.test.ts` chỉ khẳng định **thứ tự** `['flush','restore']`, không khẳng định thử lại/từ chối *(nguồn: cả ba tầng)*
- [x] [Review][Patch] **`COMMAND_FLOOR` dựng trên một số đo SAI — cổng in 44, chú thích ghi 41** [scripts/check-commands.mjs:235-243] — chú thích viết *"🔵 ĐO LẠI 2026-08-16 (Story 2.6), **không chép**: **41** command thật — 35/41 = 85,4 %"* và lý giải *"khi 2.6 thêm `history.open`/`history.close` (→ 41)"* — nhưng story đăng ký **năm** command `history.*` (`commands/index.ts:827-834`), không hai. Chạy thật `npm run check:commands`: `OK 44 command`. ⇒ Tỷ lệ thật **35/44 = 79,5 %**, rơi **dưới** dải 80–85 % mà luật sàn quần thể đặt; chín command có thể biến mất mà cổng vẫn xanh. 🔴 Chính dòng chú thích tự xưng *"đo lại chứ không chép"* là chỗ chép sai, và nó nằm ngay dưới một dòng cảnh báo rằng khuôn này đã hỏng ba lượt liên tiếp. Sửa: sàn → **37** (84,1 %) và chú thích → 44 *(nguồn: Acceptance Auditor, tôi chạy lại cổng để xác nhận)*
- [x] [Review][Patch] **Hai trong sáu giá trị của `RestoreResult` không có ĐƯỜNG RA nào — `'flush-failed'` và `'no-segment'` rơi xuống đất** [src/panels/segmentHistoryState.ts:238,240] — hai nhánh này `return` **trước** khi đặt `restoreError`/`restoreNotice`, và `restoreAimedVersion:165` · `confirmPendingRestore:178` gọi bằng `void restoreVersion(...)` nên giá trị trả về bị vứt; `main.ts:347-349` cắm thẳng cả hai, **không** `.then()`. ⇒ Người dùng bấm khôi phục, **không một pixel nào đổi**, không banner, không cả một dòng `console`. 🔴 Đây đúng lớp *"rỗng im lặng"* mà `main.ts:246-257` ghi lại là code review 2026-08-15 đã bắt và vá cho `confirmSegment` — vá đó **không** được thừa hưởng, và ba dep ngay cạnh (`confirmSegment` · `setSegmentOmitted` · `setSegmentParagraphEnd`) đều có `.then()` tiêu thụ kết quả *(nguồn: Edge Case Hunter)*
- [x] [Review][Patch] **`restoreVersion` thiếu vế chống kết quả cũ mà `loadHistory` đã có — hai hệ quả** [src/panels/segmentHistoryState.ts:236-282] — `loadHistory:187` có `if (segmentId.value !== id) return`; `restoreVersion` **không** có vế tương đương sau `await`. ⇒ ① kết quả của câu A ghi vào `restoreNotice`/`restoreError`/`pendingRestore` *(biến cấp module, không phân biệt theo segment)* và hiện dưới lớp phủ đang mở cho câu **B** — kể cả hộp thoại *"bản đang soạn sẽ mất"* của một câu khác; ② `versions.value.find(v => v.id === versionId)` `:273` tra trên mảng **đã bị `openSegmentHistory` dọn về `[]`** ⇒ `replaceEditorSegment` **không chạy**, đĩa đã đổi mà lưới vẫn hiện văn bản cũ, im lặng — đúng chỗ Story 2.5c mất một vòng chẩn đoán (commit `4ce5bb4`) *(nguồn: Blind + Edge)*
- [x] [Review][Patch] **Câu hỏi xác nhận KHÔNG chặn được danh sách phía dưới — nó bị thay im lặng** [src/SegmentHistoryOverlay.vue:259,277] — khi `historyPendingRestore !== null`, `<ol class="hist-list">` vẫn render và nút mỗi hàng chỉ khoá bằng `:disabled="historyPending"` — mà `pending` đã về `false` trong khối `finally` **trước** khi câu hỏi hiện ra. ⇒ Người dùng Tab hoặc bấm sang hàng khác, `dispatch('history.restore')` chạy một lượt mới và ghi đè `pendingRestore.value` `:256`; câu hỏi cũ **biến mất không dấu vết**, không ai từng trả lời nó. Một chốt an toàn đi vòng qua được bằng đúng thao tác mà bề mặt vẫn mời làm. *(Không mất dữ liệu — Rust vẫn hỏi lại cho lượt mới.)* *(nguồn: Blind Hunter)*
- [x] [Review][Patch] **Lượt `force` thất bại giữ nguyên `pendingRestore` ⇒ hộp thoại và banner lỗi hiện cùng lúc** [src/panels/segmentHistoryState.ts:245-248] — nhánh `outcome === null` đặt `restoreError` nhưng **không** xoá `pendingRestore`. Template render cả `hist-confirm` lẫn `history.restore_failed`: màn hình vừa hỏi *"có ghi đè không"* vừa báo *"chưa khôi phục được"* — hai trạng thái mâu thuẫn, và người dùng không đọc được lượt bấm vừa rồi đã đi tới đâu *(nguồn: Edge Case Hunter)*
- [x] [Review][Patch] **`aimedVersionId` rò qua ranh giới segment — không reset ở mở lẫn đóng** [src/panels/segmentHistoryState.ts:108-124,127-130] — `openSegmentHistory` reset năm ô *(`versions` · `loadError` · `restoreError` · `restoreNotice` · `pendingRestore`)* nhưng **không** `aimedVersionId`. ⇒ Nhắm một hàng ở câu A, đóng, mở lịch sử câu B, gọi `history.restore` trước khi chuột/Tab nhắm lại ⇒ gửi `version_id` của A cho B; Rust từ chối đúng nhờ hàng rào `AND segment_id = ?2`, nhưng người dùng nhận *"không tìm thấy phiên bản"* thay vì *"chưa nhắm hàng nào"*. ⚠️ Khả năng chạm tới **thấp**: `history.restore` mang **0 hợp âm mặc định** (`commands/index.ts:832`), chỉ tới được nếu người dùng tự gán phím qua Story 1.21 *(nguồn: Blind + Edge)*
- [x] [Review][Patch] **`!current_text.is_empty()` miễn chốt an toàn mà không một dòng lý do, không một ca** [src-tauri/src/commands/segment.rs:646] — vế này cho một segment có `target_text` rỗng đi **thẳng** tới bước ghi, bỏ qua trọn khối *"có bản sao trong `segment_version` chưa"*. Hành vi nhiều khả năng đúng *(rỗng thì không có gì để mất, và không có nó thì mọi câu chưa dịch đều bị hỏi một câu vô nghĩa)* — nhưng nó là một nhánh **ngoài** năm mệnh đề mà Quyết định #2 liệt kê, và cả file còn lại giải thích mọi quyết định bằng chữ. §Code Quality: *"một quyết định không hiển nhiên phải kèm một phép đo, không một sở thích"* *(nguồn: Blind + Auditor)*

**Đã bác (1):** *"nhánh hôm-nay-quá-một-giờ rơi về mốc tuyệt đối"* (`segmentHistoryTime.ts:80-91`) — đó **đúng là** nhánh thứ tư mà chữ ký **#6(b)** đặc tả *(vừa xong · N phút trước · Hôm qua HH:mm · **ngày đầy đủ**)*, không một khoảng hở.

#### ✅ Chín patch ĐÃ ÁP — 2026-08-16, và nghiệm thu lại từ đầu

| Đường | Trước rà | Sau vá |
| --- | --- | --- |
| `cargo test --locked` | 372/0/5 | **372/0/5** |
| `npm run test` | 130/130 | **130/130** |
| `npm run build` | xanh | **xanh** |
| Sáu cổng đọc-tệp *(`deps` · `tokens` · `i18n` · `commands` · `layout` · `lint`)* | xanh | **xanh** |

🔵 **Một lượt vá đẻ ra một hàm dùng chung, và đó là phần đáng giữ nhất của lượt rà này.** Khuôn hai lượt flush của Quyết định #8 sống **trong thân** `confirmCurrentSegmentUnguarded`, nên Story 2.6 chép được **doc-comment** của nó mà chép thiếu **mã** — và không cổng nào đỏ. Nay nó là `editorPanelState.ts::flushEditorBeforeDiscreteWrite()`, hai nơi gọi cùng đi qua một cửa, và nơi gọi **thứ ba** *(Story 2.8 gộp/tách, hay bất kỳ lệnh ghi rời rạc nào)* không còn cách nào thi hành nó đúng một nửa. Hành vi của đường ký **không đổi một bước nào** — 372 ca Rust và 130 ca vitest xác nhận.

✅ **e2e ĐÃ chạy lại sau vá — trọn bộ, trên WKWebView thật.** `TAURI_WEBDRIVER_PORT=4455` *(4445 vẫn bị `gdrive-su` PID **49798** giữ — cùng chương trình story ghi ở PID 6703/48486, phiên khác; **không** giết tiến trình của Ice)*. Output ghi **trọn** ra tệp, không `tail` — đúng bài học 🔵 của §Debug Log Ⓟ.

```
Spec Files:  8 passed, 8 total (100% completed) in 00:09:29
```

🔴 **Ca đáng giá nhất của lượt này KHÔNG phải hai ca của 2.6, mà là `editor-confirm-segment` — 2/2 xanh.** Lượt vá rút khuôn hai lượt flush ra khỏi thân `confirmCurrentSegmentUnguarded`, tức nó **sửa đường ký của Story 2.5b**. Mệnh đề *"hành vi không đổi một bước nào"* là thứ **phải đo trên engine thật**, không phải thứ suy ra từ việc 372 ca Rust vẫn xanh. Hai ca đó là lưới duy nhất cho nó.

⚠️ **Một vế vẫn KHÔNG đóng, ghi ra thay vì để tưởng đã xét: chưa có ca nào canh chính bản vá.** Ba mệnh đề mới — *"tập chờ dơ sau hai lượt flush ⇒ TỪ CHỐI"*, *"kết quả của câu A không ghi vào ô của câu B"*, *"câu hỏi đang chờ khoá được danh sách"* — dựa vào một lượt đọc mắt cộng một lượt e2e đi **đường xanh**, không một đòn bẩy nào làm chúng **đỏ**. Luật của kho đòi ngược lại: *"một cổng chưa bao giờ đỏ là một cổng chưa ai biết nó có chạy không"*. Cụ thể, ca `④` hiện có chỉ khẳng định thứ tự `['flush','restore']` — nó **vẫn xanh** nếu ai đó gỡ lượt flush thứ hai. **Chủ: Ice** *(quyết định có mở một lượt bổ sung ca hay không)*.

**Đã kiểm và KHỚP** *(ghi ra vì một lượt rà không tìm thấy gì cũng là một phép đo)*: bước di trú 10 đúng số/DDL/hai ca hợp đồng · `read_segment_history` và `restore_segment_version` đúng hình dạng #1(a) *(không `INSERT`)* · hàng rào `segment_id` · AC4 đọc-khác-ghi · `historyTimeLabel` không đọc `Date.now()` · tái dùng `z-index: 10` và câu miễn trừ có tên, không thêm `FOCUS_OWNERS` · bảy mục ghi nợ lúc ký khớp tám chữ ký, gồm khoảng hở AD-46 · `cargo test --locked` **372/0/5** và `npm run test` **130/130** khớp đúng số story tự khai.

## Nhật ký sprint-status

Gỡ nguyên văn từ `sprint-status.yaml` ngày 2026-08-19: tệp đó giữ TRẠNG THÁI, nội dung story thuộc về tệp này. Không sửa một ký tự.

```
  # 🔵 2026-08-16 — create-story: chuyen sang ready-for-dev. So di tru ke tiep DO LAI tu nguon
  #   (`PROJECT_MIGRATIONS`, schema.rs:665-711): tam buoc [1,2,3,5,6,7,8,9], dich 9 => ke tiep
  #   la **10**. Bang `segment_version` DA CO tu buoc 7 (Story 2.5) — story nay la luot DAU TIEN
  #   doc no. Hai mon no ghi chu dich danh la 2.6: deferred-work.md:2780-2785 (index) va
  #   :2787-2792 (hai moc thoi gian KHONG noi cung mot chuyen).
  #   Story mang TAM quyet dinh mo phai co chu ky cua Ice TRUOC dong ma dau tien (Task 0 chan
  #   moi task khac):
  #   🔴 #1 KHOI PHUC GHI GI VAO LICH SU — mot MAU THUAN DO DUOC giua hai tai lieu quy hoach.
  #      Bang Rule cua AD-31 (ARCHITECTURE-SPINE.md:374-381) co dung SAU hang va KHONG hang nao
  #      la "khoi phuc"; mockup data-integrity.html:226-229 thi viet dam "Khoi phuc la tao phien
  #      ban moi... day no len thanh phien ban thu sau". AC2 dung ve phia AD-31 (trang thai ve
  #      CHUA XAC NHAN). Neu Ice ky duong (b)/(c) thi do la mot **AD MOI**, khong mot dong ma —
  #      Task 0.4 DUNG story lai cho toi khi AD duoc viet ra.
  #   🔴 #2 MOT LO MAT DU LIEU KHONG AC NAO NEU. Mot hang segment_version chi sinh o DUNG MOT
  #      cho (segment.rs:955-959, trong confirm_segment) => van ban dich CHUA TUNG DUOC KY khong
  #      co ban sao nao o dau ca. Mot luot khoi phuc len mot segment dang mang ban nhap chua ky
  #      XOA VINH VIEN ban nhap do. Chinh cai lo ma mockup viet ra bang chu.
  #   #3 be mat: lop noi cap App (khuon ShortcutsOverlay/AttributionOverlay — hai tien le, ca
  #      hai KHONG co muc trong FOCUS_OWNERS) · mot tab trong panel Lookup · hay trong chinh luoi.
  #      🔴 Duong "trong luoi" vuong mot rang buoc CUNG: mot hang KHONG phai mot phan tu DOM
  #      (GridPanel.vue:5-24) — nam cot la nam subgrid chia chung mot tap track.
  #   🔴 #4 mockup ve DIFF <del>/<ins>, ma `similar` 3.1.1 va `dissimilar` 1.0.11 duoc ghi san
  #      trong Cargo.toml:86-89 va **co y khong cai cai nao** — cai mot cai hom nay la am tham
  #      dong mot quyet dinh kien truc chot o Giai doan 5. VA nam AC cua story KHONG doi diff.
  #   #5 bon nhan cua mockup tro vao bon nang luc chua dung (2.7 · Epic 4 · Epic 7 · FR94).
  #      `segment_version` co DUNG BON cot, va schema.rs:436-459 khai bang chu rang cot xuat xu
  #      thuoc Story 2.7, cot cap TM thuoc Epic 7.
  #   #6 dinh dang thoi diem: grep toLocale|Intl.DateTimeFormat|new Date( tren src/ = **0**.
  #      StatusBar.vue:36-83 lam so hoc epoch thuan. Mockup dung BA dang cung luc.
  #   #7 hinh dang index + so buoc di tru (mon no :2780-2785).
  #   #8 AC4 "segment da ve huu" — `retired_at` la None cho MOI segment hom nay; grep
  #      merge_segment tren src-tauri/src = 0; Story 2.8 la backlog.
  # 🔴 DUONG DOC va DUONG GHI PHAI TU CHOI KHAC NHAU: doc lich su cua mot segment da ve huu
  #   PHAI tra du (AC4); khoi phuc thi PHAI tu choi (no ghi). Ba lenh ghi hien co deu tra
  #   MessageKey::SegmentRetired — dung cho chung, sai cho duong doc.
  # ⚠️ Bay da ghi trong story: (a) fixture "tuong lai" STEP_TEN (segment_contract.rs:1313-1345)
  #   dang dung so 10 — sau story nay phai nang len **11** va doi ten, neu khong cong AD-30 xanh
  #   ma vo nghia (luot lap lai THU BA cua luat do); (b) hai neo o pinned_contract.rs:160-175
  #   (len() 8->9, schema_version 9->10); (c) ten `editor.restore_segment` DA BI CHIEM — no la
  #   lenh bo co cat bo cua 2.5c (FR133), khong phai khoi phuc phien ban; (d) cay `.atproj`
  #   trong mockup (`history.db`/`segments.db`/`tm.db` roi nhau) DA LOI THOI — thuc te la MOT
  #   `project.db` (atproj.rs:6, AD-9); (e) `⌘H` chi xuat hien trong mockup, khong co trong
  #   bang phim EXPERIENCE.md:261-268 lan bang cua settings.html — id lenh va hop am la quyet
  #   dinh CUA STORY NAY. Grep KeyH tren commands/index.ts = 0 => `Mod+H` trong.
  # ⚠️ AD-46 la mot khoang HO chua ai noi toi: `is_target_paragraph_end` la du lieu rieng cua
  #   ban dich, ma `segment_version` KHONG luu co => khoi phuc khong the tra no ve. Ghi no.
  # ⚠️ Cong `check-i18n.mjs` Kiem A bao FAIL SAI CHO khi mot ten the duoc nhac trong COMMENT
  #   cua template `.vue` (deferred-work.md:3551-3564, chua va). Story nay viet mot `.vue` moi.
  # 🔵 2026-08-16 — 2.6 chuyen sang in-progress (dev-story). Task 0 CHAN moi task khac:
  #   tam quyet dinh mo cho Ice ky, va #1 co the de ra mot AD MOI (Task 0.4 dung story lai).
  #   Baseline do TRUOC khi cham dong dau tien, tren HEAD 64cf7cb: cargo test --locked 359/0/5 ·
  #   vitest 103/103 — KHOP so ghi trong story. Bon tien de do lai tu NGUON deu khop.
  # 🔴 BAY DO tim ra o Task 0.6: grep `merge_segment` tra 1 ket qua chu khong 0, va dong khop
  #   la mot DOC-COMMENT viet nguyen van "grep ... cho 0" (paragraph.rs:10). So that van la 0
  #   duong ma. Luat rut ra: DOC NOI DUNG dong khop, dung dem. Kho nay ghi ket qua do vao chu
  #   thich rat day nen lop bay nay se gap lai.
  # ✅ 2026-08-16 — Ice ky TAM quyet dinh: #1(a) khong INSERT · #2(a) hoi lai · #3(a) lop phu
  #   cap App · #4(a) khong diff · #5(a) khong nhan · #6(b) tuong doi+tuyet doi · #7(a)
  #   (segment_id, created_at DESC) · #8(a) test hop dong.
  #   🔵 #1(a) LA CHU KY GO CUA CHAN: AD-31 khong sua mot chu ⇒ Task 0.4 KHONG kich hoat ⇒
  #   khong AD moi ⇒ story di tiep duoc. Hai duong kia deu de ra mot AD.
  #   Do lai bang git diff --stat tren planning-artifacts: RONG — spine khong bi cham mot dong.
  # ✅ Buoc di tru 10 DA TIEU (`idx_segment_version_segment_created`). So ke tiep la **11**.
  #   Nguon su that van la `PROJECT_MIGRATIONS` (schema.rs), khong phai dong nay.
  #   Fixture "tuong lai" nang 10 -> 11, luot lap lai THU BA cua luat do.
  # 🔵 BA PHEP DO lat/thu hep mot menh de, ghi vi chung doi cach doc so:
  #   (1) ve go hoa `id DESC` CHIU LUC — 12 luot ky lien tiep cho 11 moc khac nhau (hai luot
  #       trung mili giay). Va bon ca doc KHONG canh duoc no: go `id DESC` ra, chung van 8/8
  #       xanh ⇒ phai DUNG va cham bang SQL, khong duoc CHO dong ho va cham;
  #   (2) fixture "tuong lai" la mot neo LUC BIEN DICH — go mot buoc cho E0080, khong mot ca do;
  #   (3) mot phep so chuoi DDL se XANH tren index sai thu tu cot (SQL van chua ca
  #       "ON SEGMENT_VERSION" lan "CREATED_AT DESC") ⇒ ca hinh dang doc `pragma_index_info`.
  # 🔴 CHU KY #3(a) VA VAO AD-34 §1: `@click` phai la DUNG MOT `dispatch('<id>')` voi id
  #   literal, ma §KHONG-LAM cam mot command cho moi hang. Go bang khuon `aimedShortcutRow`
  #   cua Story 1.21 — nham bang @mousedown/@focusin (Kiem A chi canh @click), ba command
  #   khong tham so. `COMMAND_FLOOR` 33 -> 35 (41 command that).
  # ✅ 2026-08-16 — 2.6 XONG, chuyen sang `review`. Moi task tick, khong mon nao treo cho Ice.
  #   Nghiem thu: 11 cong npm (9 doc-tep + check:scope + check:scope:bundled chay tay) · build ·
  #   vue-tsc · vitest 130/130 · cargo test --locked 372/0/5 · e2e 8/8 spec, 11/11 ca (9m40).
  #   Baseline 359/0/5 + 103/103 => +13 ca Rust, +27 ca vitest.
  # 🔴 E2E: BA luot, ghi ca ba. Luot ① 7/8 — spec do KHONG xac dinh duoc vi chinh lenh cua toi
  #   `tail -45` da cat mat phan dau output. Luot ② bon spec dau chay rieng: 4/4. Luot ③ tron
  #   bo giu tron output: 8/8. => Do la chap chon cua BAN DO, khop khuon hai mon no da ghi
  #   (:3093-3115 fixture khong reset state panel · :3274-3330 devServerIsUp tin Vite hap hoi).
  #   ⚠️ KHONG cham "da chan doan": luat sau 1.22 doi BAT NGUYEN VAN TRUOC, va toi khong bat
  #   duoc. Bai hoc: dung `tail` output cua mot luot e2e.
  # 🔵 HAI ca e2e cua 2.6 la duong DUY NHAT do duoc menh de trung tam (bon truong cua
  #   `SegmentVersionRow` co that tren day). Ca 130 ca vitest deu KHONG bat duoc lop loi do —
  #   fixture chep tay co san bon truong, dung nhu 74/74 da xanh tren san pham hong o 2.5.
  # 🟡 BA AC dong MOT NUA, ca ba ghi no co chu: AC4 (be mat vao — Story 2.8) · AC1 ve nhan
  #   (bon nang luc chua dung — 2.7 · Epic 4 · Epic 7 · Epic 8) · AC2 ve co doan
  #   (`is_target_paragraph_end` khong khoi phuc duoc, `segment_version` khong luu co — chu: Ice).
  # 🔴 HAI MON NO MOI do chinh luot nay de ra:
  #   - ca `toISOString()` RONG NGHIA tren CI (runner chay UTC, ca chi co nghia o offset khac 0).
  #     Ca tu khai dieu do bang mot nhanh `expect(offsetMin).toBe(0)`. Chu: story ha tang cong.
  #   - `src/config/segment.ts` nay co HAI loai adapter: sau cai tin payload, hai cai kiem no
  #     luc chay. Ly do lech la mot lop loi DA XAY RA THAT, nhung ly do do ap cho ca sau cai
  #     kia y het. Cau hoi QUY UOC. Chu: Ice.
  # 🔵 Chin mon no moi/da dong vao deferred-work.md — bay ghi LUC KY Task 0, hai luc nghiem thu.
  #   Hai mon co chu 2.6 DA DONG bang cach NOI TIEP, khong xoa muc goc.
  # ✅ 2026-08-16 — code review BA TANG (Blind Hunter · Edge Case Hunter · Acceptance Auditor):
  #   10 phat hien tho, 9 giu + 1 bac. Ice ky MOT quyet dinh (go `.hist-current` — chi dau "dang
  #   dung" suy tu phep so noi dung, dung phep so ma Quyet dinh #5 goi la KHONG an toan). Chin
  #   patch DA VA, khong mon nao de lai lam action item.
  #   🔴 Hai mon nang nhat, ca hai o tang dieu phoi TypeScript — duong Rust KHONG co khuyet tat
  #   nao (13 ca hop dong cua Task 2-3 giu duoc no):
  #     ① Chot chong mat ban nhap thi hanh DUNG MOT NUA: doc-comment khai "khuon hai luot flush
  #       ap nguyen o day", ma ma chay MOT luot. Bao dam cua chu ky #2(a) mat trong im lang.
  #       Va KHONG bang cach chep khuon lan hai — luot chep chinh la thu pham. Nay no la
  #       `editorPanelState.ts::flushEditorBeforeDiscreteWrite()`, hai noi goi mot cua.
  #     ② `COMMAND_FLOOR` dung tren so do SAI: chu thich tu xung "do lai, khong chep" ghi 41,
  #       cong in ra 44 (chi dem 2 trong 5 command moi cua chinh story). San 35 → 37.
  #   Nghiem thu lai sau va: cargo 372/0/5 · vitest 130/130 · build xanh · sau cong doc-tep xanh
  #   · e2e TRON BO 8/8 spec, 11/11 ca (9m29s, port 4455 vi 4445 bi gdrive-su PID 49798 giu).
  #   🔴 `editor-confirm-segment` 2/2 xanh la ca dang gia nhat: luot va SUA duong ky cua 2.5b,
  #   va menh de "hanh vi khong doi" phai do tren engine that chu khong suy tu 372 ca Rust.
  #   ⚠️ CON HO, chu Ice: chua co ca nao canh chinh ban va — ba menh de moi chua tung DO. Ca ④
  #   hien co van xanh neu ai do go luot flush thu hai.
```
