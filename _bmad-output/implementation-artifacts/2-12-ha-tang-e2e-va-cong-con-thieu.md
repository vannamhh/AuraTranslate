---
baseline_commit: 4b30199263b5a5ad9dd13d3a4c8cd810951d1ba7
---

# Story 2.12: Hạ tầng e2e và cổng còn thiếu

Status: ready-for-dev

**Covers:** **không FR nào.** Story hạ tầng — nguồn là **action item B2** của retro Epic 2.
**Epic:** 2 — Biên tập theo segment
**Soạn:** 2026-08-18 · trên HEAD `4b30199`, cây làm việc **sạch**
**Vị thế:** 🔴 **CỬA CHẶN ②** — Ice ký 2026-08-18. **Epic 3 không mở trước khi story này chạy xong.**
**Thứ tự:** 🔴 **Story 2.4 gỡ TRƯỚC story này** *(Ice chốt 2026-08-18)*. Xem §Thứ tự ngay dưới.

---

## ⚠️ Số hiệu `2.12` được TÁI DÙNG — đọc dòng này trước khi tra `epics.md`

`epics.md:2659-2667` mang một khối **`### ~~Story 2.12: Sync Scrolling~~ — XOÁ`** *(Sprint Change
Proposal 2026-08-14, Ice ký; FR20 đã rút)*. Khối đó nằm **ngay trước `## Epic 3`** — đúng chỗ mắt
người đọc sẽ tìm story này.

⇒ **Story 2.12 hôm nay KHÔNG phải sync-scrolling sống lại.** Slug khác nên khoá
`sprint-status.yaml` không đụng nhau *(`2-12-ha-tang-e2e-va-cong-con-thieu` ≠ `2-12-sync-scrolling`)*.
Ghi ra bằng chữ theo đúng yêu cầu của retro *(`epic-2-retro-2026-08-18.md:350-352`)*.

🔴 **Và ghi thẳng chỗ yếu:** `epics.md` hôm nay **không có** một `### Story 2.12` đang hoạt động.
Story này sinh ra từ action item retro, không từ `epics.md`. Việc bổ sung một mục vào `epics.md` là
một lượt **correct-course** *(chủ: PM/Winston)*, **không** phải việc của `create-story` và **không**
phải việc của dev — xem §Câu hỏi cho Ice, mục ⑴.

---

## 🔴 Thứ tự: cửa chặn ① *(Story 2.4)* gỡ TRƯỚC — Ice chốt 2026-08-18

Hai cửa chặn độc lập về nội dung nhưng **không** song song. Ice chốt trình tự:

**Story 2.4 *(B1, NFR2)* → Story 2.12 *(story này)* → Epic 3.**

**Vì sao thứ tự này đổi nội dung story, không chỉ đổi lịch:**

- NFR2 đang vượt trần **~15 lần** *(706-770 ms / 50 ms)*. Con số đó **chưa có bản vá**. Một bản vá
  NFR2 chạm **đúng đường nóng** mà chữ ký **#4** *(tín hiệu "flush xong")* và chữ ký **#8**
  *(ngưỡng "tái lập được")* đang đo.
- Biên **1.500 ms** giữa `FLUSH_WAIT_MS` và `EDITOR_IDLE_MS` **có thể tự nới ra** sau khi 2.4 vá —
  hoặc **hẹp lại**. Đo trước khi 2.4 xong là đo trên một cây sắp đổi.
- 🔴 ⇒ **Task 0.1 phải đo lại baseline SAU khi 2.4 đóng**, không dùng bảng §Điều kiện khởi hành của
  tệp này như một số đã chốt. Bảng đó chụp HEAD `4b30199`, **trước** 2.4.

⚠️ **Nhưng story này KHÔNG chờ 2.4 để được soạn** — nó `ready-for-dev` từ 2026-08-18. Thứ chờ là
**lượt dev**, không phải hồ sơ. Và ranh giới giữ nguyên: story này **không đo, không vá, không chấm**
NFR2 *(xem §Ranh giới phạm vi, mục 1)*.

---

## Story

As a **người dựng AuraTranslate**,
I want **bộ đo e2e nói thật và hai cổng còn thiếu đứng dậy**,
So that **Epic 3 không dựng một cột dữ liệu mới lên trên một lưới nghiệm thu đang nói dối**.

---

## 🔴 ĐỌC TRƯỚC DÒNG MÃ ĐẦU TIÊN — story này sửa BỘ ĐO, không sửa sản phẩm

**Đây là story đầu tiên của kho có đối tượng là chính hạ tầng nghiệm thu.** Ba mệnh đề dưới đây
quyết định mọi thứ còn lại, và cả ba đo được:

### ① e2e là lưới **DUY NHẤT** cho hình dạng dây — đã chứng minh **bốn** lần

| Story | Số xanh **trong lúc sản phẩm đang hỏng** |
|---|---|
| 2.5 | **74/74** vitest xanh trong khi `read_open_chapter_segments` không gửi `status` ⇒ `isConfirmed` **luôn `false`** trong app thật |
| 2.6 | **130/130** vitest xanh; hai ca e2e là đường **duy nhất** đo được bốn trường của `SegmentVersionRow` trên dây |
| 2.7 | **382 Rust + 133 vitest ĐỀU XANH** trong khi hình dạng dây gãy *(`invalid args: missing required key textAtLoad`)* |
| 2.9 review | **191/191** xanh sau khi gỡ `data-src-atomic` |

Nguyên nhân chung, đã viết ra bằng chữ *(`epic-2-retro-2026-08-18.md:155-157`)*: **fixture chép tay
LUÔN có sẵn trường. vitest kiểm cái fixture, không kiểm cái dây.**

### ② Và chính cái lưới duy nhất ấy là một **bộ đo nói dối**

Khuôn *"xanh riêng, đỏ trong bộ"* xuất hiện ở **năm** story: 2.6 · 2.8 · 2.9 · 2.10 · 2.11.
`wdio.conf.mjs:70-80` giữ bản ghi: **8 lượt trọn bộ = 6 xanh · 2 đỏ**, và **lần đỏ ② (`attribution-focus`)
CHƯA BAO GIỜ được chẩn đoán** — xanh khi chạy một mình, xanh ở mọi lượt trọn bộ khác, nguyên văn lỗi
không kịp bắt.

⇒ Hệ quả phải nhìn thẳng: **cả hai mệnh đề trên đúng cùng lúc.** Lưới duy nhất bắt được lớp lỗi đắt
nhất cũng chính là lưới mà kết quả xanh/đỏ của nó phụ thuộc **tải máy và thứ tự chạy**.

### ③ Epic 3 đổ **đúng lớp lỗi đó** lên **đúng bề mặt đó**

Epic 3 thêm một **cột dữ liệu mới** *(Glossary)* và một **kênh trang trí thứ hai** lên cột nguyên văn
của lưới — cột đã mang ánh xạ ngược **bằng CHỈ SỐ** *(`host.children[i] ↔ segments[i]`)*, neo
`data-src-start`, `<rt>` ruby Hán Việt, và **ba** cử chỉ chuột trên cùng một `mouseup`
*(`epics.md` FR50 nay chở nguyên ràng buộc này)*.

🔴 **Đây là lý do cửa chặn tồn tại, không phải một lời cẩn thận.** Cột `status` · `is_omitted` ·
`is_target_paragraph_end` · `translation_origin` đã lặp **bốn** lần cùng một khuôn *"cột mới phải đi
qua đây"*. Cột Glossary là lần **thứ năm** đang tới.

---

## Acceptance Criteria

*(Story này không có AC trong `epics.md` — bảy AC dưới đây được dẫn xuất từ **sáu món của action item
B2** cộng một AC nghiệm thu. Mỗi AC ghi rõ nó đến từ dòng nào.)*

🔴 **CẢNH BÁO ĐỌC NHẦM:** story này **cố ý dừng ở AC7**. Trong kho, chữ **"cổng AC8"** là **biệt danh
của một test Rust** — `a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
*(`src-tauri/tests/segment_contract.rs:2333`)* — **không** phải AC thứ tám của bất kỳ story nào. Nếu
story này có một "AC8" thì hai thứ khác hẳn nhau sẽ mang cùng một cái tên trong cùng một tệp.

1. **AC1** — **Given** Vite đang chạy nhưng module graph đã vỡ *(vẫn trả 200 cho `/`)* · **When**
   `devServerIsUp()` chạy · **Then** nó trả **`false`**, và bộ e2e dừng với một câu nói đúng nguyên
   nhân — **không** để 11 spec đỏ vì một lý do không liên quan.
   *(B2 món ①; nợ `deferred-work.md:3402-3408`; mã `e2e/wdio.conf.mjs:191-198`.)*

2. **AC2** — **Given** hai spec chạy nối tiếp trong cùng một lượt trọn bộ · **When** spec thứ hai bắt
   đầu · **Then** **không** state cấp module nào của panel sống sót từ spec trước, và điều đó đúng
   **do fixture**, không do mỗi spec tự gọi `window.location.reload()`.
   *(B2 món ②. Hôm nay **không một hook `before`/`beforeEach` nào** tồn tại trong `e2e/support/**`; ba
   spec tự vá tại chỗ — `grid-empty-cell:65-67` · `segment-backspace-merge:131-133` ·
   `editor-typing-flush:233-235` — và các spec khác **không** vá.)*

3. **AC3** — **Given** một spec chờ một thay đổi trạng thái *(Chương mới nạp, lưới đổi số hàng)* ·
   **When** nó chờ · **Then** nó chờ **trạng thái ĐÍCH**, không chờ *"phần tử tồn tại"* — tức
   `waitForExist` rồi đọc ngay **không được** dùng ở chỗ trạng thái cũ và mới có **cùng hình dạng DOM**.
   *(B2 món ③; nợ `deferred-work.md:4353-4360`. Khuyết tật đã bắt được ở 2.9 bằng `Expected: 3,
   Received: 2`. Ba spec đã tự chuyển sang `waitUntil` — `segment-navigation:116-124` ·
   `segment-backspace-merge:106-124` · `segment-merge-split:87` — story này đóng phần còn lại.)*

4. **AC4** — **Given** một spec cần biết một lượt flush đã xong · **When** nó chờ · **Then** nó chờ
   một **SỰ KIỆN/trạng thái quan sát được**, không chờ một khoảng thời gian.
   🔴 **Và hằng số của AD-35 KHÔNG được nới** — không `EDITOR_IDLE_MS`, không `EDITOR_HARD_CAP_MS`,
   không `FLUSH_WAIT_MS` to hơn.
   *(B2 món ④, nguyên văn: **"chờ một SỰ KIỆN, không chờ một khoảng thời gian — không nới hằng số"**;
   nợ `deferred-work.md:3978-3992`.)*

5. **AC5** — **Given** một ô nhớ cấp module mới được thêm vào một tệp state của panel · **When** cổng
   chạy · **Then** cổng **ĐỎ** nếu ô đó không đi qua hàm reset tương ứng, hoặc không có một **miễn
   trừ CÓ TÊN kèm lý do đọc được tại chỗ**.
   *(B2 món ⑤ *(F1)*; nợ `deferred-work.md:4684`. Luật này đã bị bỏ sót **hai story liên tiếp** —
   `sourceCut` ở 2.8, `omitError` ở 2.9 — và **không cổng nào canh**.)*

6. **AC6** — **Given** một cột mới của bảng `segment` ra đời qua một bước di trú · **When** cổng chạy ·
   **Then** cổng **ĐỎ** nếu cột đó chưa xuất hiện ở **mọi** chỗ bắt buộc — không chỉ ở đường flush mà
   cổng-AC8 hiện canh.
   *(B2 món ⑥ *(F2)*. Lỗ đã đo: cổng-AC8 chỉ canh đường flush `save_segment_targets`; đường ghi thứ
   hai — `write_regroup`, `src-tauri/src/commands/segment.rs:2210` với `INSERT` ở `:2234` — **không
   cổng nào đỏ** nếu thiếu cột. Ghi ở `2-8-gop-va-tach-segment-tuong-minh.md:508`.)*

7. **AC7** — **Given** bộ e2e trọn bộ chạy trên máy đã thoả điều kiện của chữ ký #8 · **When** chạy ·
   **Then** nó cho một kết quả **tái lập được**, và mọi ca đỏ còn lại **có nguyên văn lỗi được bắt**
   — không còn một lần đỏ nào ở tình trạng *"chưa chẩn đoán"* như lần ② của `wdio.conf.mjs:70-80`.
   *(Không nằm trong sáu món của B2. Nó là điều kiện *"chạy trước Epic 3"* của B2 nói bằng số —
   xem chữ ký #8 cho ngưỡng.)*

**Với AC5 và AC6, mỗi cổng mới phải mang đủ ba thứ của một cổng trong kho này** *(`project-context.md:300-321`)*:
mã thoát là phán quyết · một phép **TỰ KIỂM** chứng minh nó **đỏ được và không đỏ oan** · và có mặt ở
**cả BA danh sách** *(`package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`)*, thứ mà
`check:gates` Kiểm D/E/F canh.

---

## 🔴 Task 0 — CỬA CHẶN: tám quyết định mở, phải có chữ ký của Ice

**Không một dòng mã nào được viết trước khi tám mục dưới đây có chữ ký.**
Trình mỗi quyết định **kèm số đo hoặc trích dẫn nguồn**, không kèm một khuyến nghị đã tự chốt.

🔴 **Task 0.4 là một CỬA CHẶN THẬT, khuôn này ĐÃ KÍCH HOẠT hai lần trong Epic 2** — AD-47 giao
Winston ở Story 2.7, AD-48 giao Winston ở Story 2.9. Quyết định **#4** và **#7** ở đây là ứng viên
trực tiếp. Nếu kích hoạt: **dừng story**, soạn **hồ sơ bàn giao**, **đừng tự soạn `AD`**.

---

### Quyết định #1 🔴 — Story này có ôm vế **Windows/Blink** không

**Số đo:** hai món nợ mang chủ **Story 1.22** còn treo — `deferred-work.md:3671` *(vế Blink của mọi
phép đo hình học/hành vi engine)* và `:3722` *(lượt dán giữ `\n`, vế thị giác trên Blink)*. Retro Epic 1
action item **A5** *(bảng nghiệm thu Windows)* đã **lỡ mốc lần thứ hai** — Epic 2 khép lại với **0**
bằng chứng Windows cho 145 ca Rust mới + 249 ca vitest + 11 spec e2e *(retro **B7**)*.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Chỉ macOS/WKWebView. Hai món nợ 1.22 **giữ nguyên chủ**, không đổi chủ sang 2.12 | Cửa chặn ② đóng được trong phiên; khoảng mù Windows **dày thêm một epic nữa** |
| **(b)** | Ôm luôn vế Windows | Không có máy Windows ⇒ story **chặn ngay**, cửa chặn ② không đóng, Epic 3 không mở |
| **(c)** | Chỉ macOS, **và** story này ghi một mục nợ mới nêu đích danh rằng AC7 của nó là một mệnh đề **một nền tảng** | Trung thực hơn (a) một dòng; cùng chi phí |

🔴 **Không tự chọn.** Ghi ra vì nó đúng khuôn *"đừng loại một phương án chỉ vì nó đắt"*.

---

### Quyết định #2 🔴 — Phạm vi quét của cổng AC5, và ba nhóm loại trừ

**Số đo, đếm từ nguồn hôm nay:**

| Nhóm | Số | Bằng chứng |
|---|---|---|
| Ô nhớ cấp module trong `editorPanelState.ts` **có** qua `resetEditorPanel()` | ~16 | `editorPanelState.ts:509-612` |
| Ô nhớ cấp module **KHÔNG** qua nó | **4** | `kyTrungCauCuoi:886` · `confirmInFlight:889` · `dangChuyenChuong:1404` · `regroupInFlight:2005` |
| Tệp `src/panels/**` có ref cấp module **và** hàm reset riêng | 3 | `sourcePanelState.ts:360` · `lookupPanelState.ts:318` · `lookupHistoryState.ts:360` |
| Tệp `src/panels/**` có ref cấp module **và KHÔNG hàm reset nào** | **2** | `dictSourcesState.ts` *(`:37 :38 :151 :154 :297`)* · `segmentHistoryState.ts` |

**Ba câu phải trả lời riêng:**

- **(2a)** Bốn cờ/mutex tiến trình — **loại trừ CÓ TÊN**, hay bắt buộc reset? *(Chúng không mang dữ
  liệu hiển thị của một Tác phẩm, nhưng đây đúng là lớp mà `sourceCut`/`omitError` đã lọt qua.)*
- **(2b)** ⚠️ **Bẫy đỏ oan, phải quyết trước khi viết một dòng cổng:** `const x = ref()` ở đầu
  `<script setup>` của một `.vue` là state cấp **COMPONENT-INSTANCE**, dựng lại mỗi lần component
  mount — **không** phải state cấp module. Một cổng quét *"mọi `ref(` đầu dòng trong `src/panels/**`"*
  mà không phân biệt `.vue` với `.ts` sẽ **đỏ oan hàng chục chỗ** *(ví dụ `LookupPanel.vue:189,225` ·
  `PanelFrame.vue:78`)*. Phạm vi là `src/panels/**/*.ts` thôi, hay rộng hơn kèm luật phân biệt?
- **(2c)** Hai tệp **không có hàm reset nào** — là lỗ hổng thứ ba mà story này đóng, hay ngoài phạm vi
  và ghi nợ có chủ?

---

### Quyết định #3 🔴 — Cổng AC6 là một cổng TĨNH mới, hay một ca Rust nữa

**Số đo:** hạ tầng chống lớp lỗi này **đã có một nửa**:

- `segment_contract.rs:2333` — `a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
  *(cổng-AC8)*: so **trọn hàng** trước/sau một lượt flush.
- `segment_contract.rs:2236-2261` — tự kiểm `the_raw_column_reader_sees_every_column_the_segment_table_actually_has`:
  đếm cột thật bằng `pragma_table_info('segment')`, so với **13**. Thông báo assert nói thẳng: *"mot
  cot moi PHAI duoc them vao `SegmentRow` CUNG LUOT voi buoc di tru sinh ra no — neu khong, cong AC8
  … mu voi dung cot do va van xanh"*.

**Lỗ thật, đo được:** cổng-AC8 chỉ đi qua **đường flush**. Bảng `segment` có **hai** chỗ `INSERT`:
`commands/segment.rs:132` *(đường nhập)* và `:2234` *(`write_regroup`, gộp/tách segment)*. Đường thứ
hai **không cổng nào đỏ** nếu thiếu cột.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Một ca Rust nữa: so trọn hàng qua đường `write_regroup` | Rẻ, cùng khuôn đã có, chạy trong `cargo test`. Nhưng nó canh **hành vi một đường**, và đường **thứ ba** ra đời sau vẫn mù |
| **(b)** | Một cổng tĩnh `check:*.mjs` đối chiếu **danh sách cột** ↔ **danh sách nơi cột phải xuất hiện** | Canh **khai báo trên toàn cây** — đúng vai của một cổng tĩnh *(`project-context.md:259`)*. Đắt hơn, và phải tự viết parser SQL tập con *(không được thêm phụ thuộc npm)* |
| **(c)** | Cả hai | Đắt nhất; nhưng (a) đóng lỗ **hôm nay** và (b) đóng lỗ **lần sau** |

🔴 **Đây là chỗ dễ chọn sai đường nghiệm thu nhất trong cả story.** `project-context.md:265-267`:
*"Chọn sai đường là dựng nguồn sự thật thứ hai. Trước khi viết một phép kiểm mới, hỏi: mệnh đề này đã
có chủ chưa."* Mệnh đề *"cột mới phải qua AC8"* **đã có chủ một nửa** — chữ ký phải nói rõ nửa còn lại
thuộc đường nào.

---

### Quyết định #4 🔴 — Tín hiệu "đã flush xong" cho AC4 lộ ra bằng gì *(ứng viên cửa AD)*

**Số đo:** `FLUSH_WAIT_MS = 3.500` *(`editor-typing-flush.e2e.mjs:71`)* so với `EDITOR_IDLE_MS = 2.000`
*(`src/panels/editorFlush.ts:43`)* ⇒ biên **1.500 ms**. Trong biên đó phải lọt: timer idle + một lượt
`invoke` + `Store::write` **nối tiếp** của AD-11 + một lượt `fsync` WAL. **Một máy đang biên dịch Rust
ăn hết biên đó** — phân xử bằng năm lượt trọn bộ ở 2.7: máy bận **7/8** ba lần liên tiếp, máy rảnh
**8/8**.

**Tín hiệu đã tồn tại một nửa:** `StatusBar.vue:235` mang câu *"Đã lưu N giây trước"*, và
`editor-typing-flush.e2e.mjs:196-199` **đã đọc nó** để khẳng định AC7 của Story 2.3 — nhưng chỉ đọc
**SAU** `pause(FLUSH_WAIT_MS)`, không dùng nó làm điều kiện `waitUntil`.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | `waitUntil` khớp chuỗi hiển thị của `StatusBar` bằng regex | **0 dòng mã sản phẩm mới.** Nhưng nó buộc bộ đo phụ thuộc **văn bản người dùng thấy** — một lượt đổi `vi.json` làm bộ đo đỏ oan |
| **(b)** | Thêm một `data-*` trên `StatusBar` phơi trạng thái đã lưu | Bộ đo bám một **hợp đồng**, không bám một câu chữ. Nhưng đây là **mã sản phẩm tồn tại để phục vụ bộ đo** |
| **(c)** | Một sự kiện `window` phát khi flush đã vào WAL | Đúng nghĩa nhất với AD-35 *("một flush chỉ xong sau khi đã ghi vào WAL")*. Và **chính vì thế** nó là ứng viên `AD` mạnh nhất |

🔴 **Vì sao đây là ứng viên cửa Task 0.4:** AD-35 định nghĩa **khi nào** flush chạy, không định nghĩa
một **bề mặt quan sát được** cho việc flush đã xong. (b) và (c) đều **thêm một bề mặt mới** — đúng
hình dạng mà AD-45 mô tả cho một cổng lắng nghe, và AD-2 cho một port thứ tư: *một bề mặt mới không có
luật*. Nếu chữ ký rơi vào (b) hoặc (c): **dừng story, soạn hồ sơ bàn giao cho Winston.**

⚠️ Và luật đối lập phải cân: `project-context.md:278-281` — *"khoảng thiếu của bản mô phỏng vá ở
`setup.ts`; khuyết tật sản phẩm vá trong `src/`"*. Câu đó nói về `happy-dom`, không nói về e2e trên
engine thật. **Đừng mượn nó làm lý do bác (b)/(c) mà không nêu ra sự khác chỗ.**

---

### Quyết định #5 — Fixture reset state panel bằng cơ chế nào *(AC2)*

**Số đo:** fixture `openWorkspaceWithWork` *(`e2e/support/workspace.mjs:55-96`)* tạo Tác phẩm qua IPC
`create_work_from_text` — **đi vòng qua** `libraryImport.ts::finishSubmit`, đường **DUY NHẤT** gọi
`resetEditorPanel()`/`resetSourcePanel()`/`resetLookupPanel()`. Mọi spec dùng chung **một** `$APPDATA`
tạm cho cả lượt chạy *(`wdio.conf.mjs::onPrepare`)*, nên `app_config` sống sót qua từng phiên app.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Chuẩn hoá `window.location.reload()` **vào fixture** | Rẻ nhất; đúng thứ ba spec đã tự làm. Nhưng nó reset bằng cách **giết cả webview state**, che luôn những rò rỉ thật mà một cổng nên thấy |
| **(b)** | Fixture gọi thẳng các hàm reset qua một cầu | Reset **đúng cái luật nói**, và ăn khớp với cổng AC5. Cần một đường gọi được từ driver |
| **(c)** | Một phiên app mới cho mỗi spec | Sạch nhất. **~1,5 phút × 11 spec** — và ba ca **đã** đụng trần `mochaOpts.timeout` 120 s *(`deferred-work.md:4325-4335`)* |

---

### Quyết định #6 — Vá `devServerIsUp` bằng gì *(AC1)*

**Mã hôm nay** *(`wdio.conf.mjs:191-198`)*: một `fetch(DEV_URL)` timeout 1 s, trả `res.ok`. Không đọc
body, không kiểm một asset nào. Nợ `deferred-work.md:3402-3408` ghi thẳng: **"Vá hướng nào cũng được,
nhưng phải ĐO chứ đừng đoán."**

Ba hướng để đo, không phải để chọn trên giấy: **(a)** `fetch` một module thật của app *(entry TS
sau transform)* và kiểm content-type/`200`; **(b)** đọc body `index.html` tìm một dấu; **(c)** chờ một
tín hiệu từ chính webview sau khi nạp.

🔴 **Luật dựng bộ đo, đã trả giá ba lần trong Epic 2** *(retro F4)*: ① *một listener chẩn đoán phải
**CHỊU ĐƯỢC** engine nó đang đo*; ② *một bản đồ **CHÉP** hàm sản phẩm sẽ đo **bản chép**, và bản chép
cứ đi*. Bản vá `devServerIsUp` phải có **đối chứng dương**: dựng cho được một Vite hấp hối thật và
chứng minh phép kiểm mới **đỏ** trên nó.

---

### Quyết định #7 🔴 — Có thêm cổng thứ mười và mười một không, hay gộp vào cổng đã có

**Số đo:** hôm nay `pre-push` chạy **chín** cổng đọc-tệp *(`.githooks/pre-push:62`)*, và
`package.json:9-25` khai đủ. `check:gates` Kiểm D/E/F canh việc một cổng có mặt ở **cả ba** danh sách.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Hai cổng mới độc lập *(ví dụ `check:panel-refs`, `check:segment-columns`)* | Mỗi mệnh đề một chủ, đúng khuôn. Sửa **ba** danh sách × 2, và `pre-push` dài thêm *(đo 2026-08-11: chín cổng = 11 s)* |
| **(b)** | Gộp vào `check:layout` *(đã canh `src/**`)* và `check:commands` | `pre-push` không dài thêm. Nhưng một cổng mang hai mệnh đề khác miền là chỗ mệnh đề yếu bị mệnh đề mạnh che |

⚠️ **Ràng buộc kèm theo, không phải một lựa chọn:** cổng nào đọc `src/**` thì phải xét lại **sàn quần
thể**. Sàn hiện tại đã cũ so với số thật — `check-layout.mjs:101` `FILE_FLOOR = 43` ghi chú *"52 tệp"*,
số thật hôm nay **55**. Sàn là **cận dưới** nên nó không đỏ oan, nhưng một sàn cũ là một sàn **vô
nghĩa** *(`project-context.md:312-314`)*.

---

### Quyết định #8 🔴 — AC7 "tái lập được" nghĩa là bao nhiêu, trên máy nào

**Số đo:** `wdio.conf.mjs:70-80` — 8 lượt trọn bộ: **6 xanh · 2 đỏ**, lần đỏ ② chưa chẩn đoán.
Ở 2.7: máy bận **7/8** ba lần liên tiếp; máy rảnh **8/8** trên cả cây story lẫn baseline.

**Ba câu phải trả lời riêng:** bao nhiêu lượt trọn bộ liên tiếp xanh mới tính là đạt *(n = 3? 5?)* ·
đo trên **máy rảnh**, **máy bận**, hay **cả hai** · và một ca đỏ **có nguyên văn được bắt** thì tính
là đạt AC7 hay không đạt.

🔴 **Không tự chấm đạt bằng suy luận.** `project-context.md:334-337`: *"Vế nào không nghiệm thu được ở
tầng đang làm thì ghi vào `deferred-work.md` kèm chủ."* Một lượt xanh **không tái lập** không phải một
lượt xanh — đó chính là mệnh đề story này tồn tại để sửa.

---

### 0.9 — Việc phải làm ở Task này

1. **Trình tám quyết định kèm số đo**, không kèm một khuyến nghị đã tự chốt.
2. **Ghi chữ ký + ngày + đường bị loại** vào §Dev Agent Record ngay khi Ice ký.
3. **Cửa chặn `AD` (Task 0.4):** nếu #4 rơi vào (b)/(c), hoặc bất kỳ quyết định nào chạm một bất biến
   — **dừng story**, `git diff --stat` trên `src/ src-tauri/ scripts/ tests/ e2e/` phải **RỖNG**, soạn
   **hồ sơ bàn giao**, **đừng tự soạn `AD`**.
4. 🔴 **LUẬT DỪNG:** ba vòng chẩn đoán liên tiếp bị **phép đo bác** trên một giả thuyết ⇒ **dừng, báo
   Ice**. ⚠️ Chỉ đếm vòng mà **giả thuyết bị bác**, **không** đếm vòng **sửa thước** *(bài học 2.10:
   hai lần sửa thước — `[data-caret]` không tồn tại; `browser.keys(['Alt','ArrowDown'])` gửi hai
   keydown rời chứ không một hợp âm — đều bắt được nhờ **đối chứng dương**)*.

---

## Tasks / Subtasks

### Task 0 — Cửa chặn: tám quyết định (AC: 1-7) — **CHẶN MỌI TASK KHÁC**

- [ ] 0.1 **ĐO LẠI** cả bảng §Điều kiện khởi hành từ nguồn — **không chép** bảng đó
- [ ] 0.2 Trình tám quyết định kèm số đo; nhận chữ ký của Ice; ghi ngày + đường bị loại
- [ ] 0.3 Đo lại bốn nguyên nhân của F3 từ nguồn — retro tự khai chúng mới *"đo được một PHẦN"*
- [ ] 0.4 🔴 Cửa `AD`: nếu #4 = (b)/(c) ⇒ **dừng story**, soạn hồ sơ bàn giao cho Winston

### Task 1 — `devServerIsUp` nói thật (AC: 1)

- [ ] 1.1 Dựng **đối chứng dương**: một Vite hấp hối thật *(module graph vỡ, `/` vẫn 200)*
- [ ] 1.2 Chứng minh `devServerIsUp()` **hôm nay** trả `true` trên đó — bắt **nguyên văn**
- [ ] 1.3 Vá theo chữ ký #6; chứng minh nó **đỏ** trên đối chứng dương và **xanh** trên Vite lành
- [ ] 1.4 Câu báo khi dừng phải nói **đúng nguyên nhân**, không để 11 spec đỏ vì lý do khác
- [ ] 1.5 ⚠️ **Vế phía client, cạm bẫy 8:** một cửa sổ `about:blank` với `document.body` rỗng đi qua
      `devServerIsUp` sạch. Đo xem nó còn tái lập được không; nếu còn ⇒ nêu với Ice như một mục nợ
      **có chủ**, đừng lặng lẽ gộp vào AC1

### Task 2 — Fixture reset state panel (AC: 2)

- [ ] 2.1 Đo rò rỉ hiện tại: dựng một ca chứng minh state spec trước sống sang spec sau
- [ ] 2.2 Dựng cơ chế reset theo chữ ký #5, đặt ở `e2e/support/**`
- [ ] 2.3 Gỡ ba lượt `window.location.reload()` vá tại chỗ — hoặc ghi rõ vì sao **giữ**
- [ ] 2.4 ⚠️ Đừng phá phép **tự kiểm dương tính `global.db`** *(`wdio.conf.mjs:287-343`)* và hai biến
      chuyển hướng `AURATRANSLATE_E2E_DATA_DIR` · `AURATRANSLATE_E2E_LIBRARY_ROOT`

### Task 3 — Khuôn chờ trạng thái đích (AC: 3)

- [ ] 3.1 Liệt kê **mọi** chỗ `waitForExist(...)` rồi đọc ngay mà trạng thái cũ/mới **cùng hình dạng DOM**
- [ ] 3.2 Rút khuôn dùng chung từ ba chỗ đã tự vá *(`segment-navigation:116-124` ·
      `segment-backspace-merge:106-124` · `segment-merge-split:87`)* vào `e2e/support/**`
- [ ] 3.3 Chuyển các chỗ còn lại sang khuôn đó — **25** lượt `browser.pause()` là cận trên, không phải
      danh sách phải xoá hết *(`realClick` ở `pointer.mjs:54,56` có `pause` **CÓ CHỦ ĐÍCH** — giữ)*

### Task 4 — Chờ SỰ KIỆN thay vì chờ thời gian (AC: 4)

- [ ] 4.1 Dựng tín hiệu theo chữ ký #4 *(hoặc dừng ở cửa `AD` nếu #4 = (b)/(c))*
- [ ] 4.2 Thay `pause(FLUSH_WAIT_MS)` ở `editor-typing-flush.e2e.mjs:179,287` bằng chờ tín hiệu
- [ ] 4.3 🔴 **Chứng minh không hằng số nào bị nới**: `EDITOR_IDLE_MS` = 2000, `EDITOR_HARD_CAP_MS`
      = 5000 *(`editorFlush.ts:43,56`)* **không đổi một chữ số**
- [ ] 4.4 Đo lại trên **máy bận** — đây là điều kiện mà biên 1.500 ms đã trượt

### Task 5 — Cổng ô nhớ cấp module (AC: 5)

- [ ] 5.1 Chốt phạm vi theo chữ ký #2 *(`.ts` cấp module vs `.vue` script-setup — bẫy đỏ oan)*
- [ ] 5.2 Viết cổng theo khuôn `check-layout.mjs` — `abort()` cho lỗi hạ tầng *(`:39-51`)*, `fail()`
      tích luỹ, mã thoát quyết ở cuối tệp *(`:625-645`)*
- [ ] 5.3 **Miễn trừ CÓ TÊN kèm lý do tại chỗ** — khuôn `Map<name, reason>` của `check-gates.mjs:87-130`
- [ ] 5.4 🔴 **Phép TỰ KIỂM** — khuôn `check-layout.mjs:556-617`: ca dương *(ref mới không qua reset ⇒
      phải đỏ)* **và** ca âm đối chứng *(tên chỉ giống ⇒ không được đỏ)*. Gọi **CHÍNH** hàm đang chạy
      thật, **không** một bản chép *(bài học F4 ②)*
- [ ] 5.5 Chứng minh cổng **đỏ** trên `sourceCut` và `omitError` nếu hoàn nguyên chúng khỏi `resetEditorPanel`

### Task 6 — Cổng cột mới của bảng `segment` (AC: 6)

- [ ] 6.1 Dựng theo chữ ký #3 — (a) ca Rust, (b) cổng tĩnh, hay (c) cả hai
- [ ] 6.2 Đóng lỗ **đã đo**: `write_regroup` *(`segment.rs:2210`, `INSERT` ở `:2234`)* không được canh
- [ ] 6.3 Nếu là cổng tĩnh: parser SQL là **tập con nghiêm ngặt tự viết**; cú pháp ngoài tập con ⇒
      **FAIL, không bỏ qua**. 🔴 **Không thêm phụ thuộc npm cho một cổng**
- [ ] 6.4 Phép **TỰ KIỂM** như 5.4
- [ ] 6.5 Đừng dựng lại thứ đã có: `the_raw_column_reader_sees_every_column_the_segment_table_actually_has`
      *(`segment_contract.rs:2236-2261`)* đã canh **một** vế — nâng hằng **13** cùng lượt nếu cột đổi

### Task 7 — Ba danh sách và sàn (AC: 5, 6)

- [ ] 7.1 `package.json` `"scripts"` *(`:9-25`)*
- [ ] 7.2 `.github/workflows/ci.yml` — cùng nhóm với chín bước `check:*` hiện có
- [ ] 7.3 `.githooks/pre-push:62` — thêm tên **không** tiền tố `check:` vào chuỗi `for gate in …`
- [ ] 7.4 `npm run check:gates` phải **xanh** — Kiểm D/E/F canh đúng ba danh sách trên
- [ ] 7.5 Xét lại **sàn quần thể** nếu cổng mới đọc `src/**`; sàn cũ *(`check-layout.mjs:101` = 43,
      chú thích "52", số thật **55**)* nâng theo **số THẬT, đo chứ không ước**

### Task 8 — Nghiệm thu (AC: 1-7)

- [ ] 8.1 Chín cổng cũ + hai cổng mới: **exit 0**
- [ ] 8.2 `npm run test` — không **giảm** so với **249/249** *(21 tệp)*
- [ ] 8.3 `cargo test --locked` — không **giảm** so với **409 passed / 0 failed / 5 ignored**
- [ ] 8.4 🔴 Bộ e2e **trọn bộ**, số lượt và loại máy theo chữ ký #8. Ghi **nguyên văn** mọi ca đỏ
- [ ] 8.5 ⚠️ **Đột biến để chứng minh bộ đo mới không rỗng** — khuôn đã trả lãi ở 2.11: một ca test
      *"flush TRƯỚC lượt chuyển"* ban đầu **không canh gì**, lộ ra chỉ khi bỏ `await` mà ca vẫn xanh
- [ ] 8.6 Ghi số kèm **phiên bản toolchain và ngày** — *"số đo không truy nguyên được thì không phải số đo"*

### Task 9 — Sổ nợ và tài liệu

- [ ] 9.1 Đóng **11** mục nợ chủ *"story hạ tầng e2e"* — `deferred-work.md:3402` `:3671` `:3722`
      `:3978` `:4136` `:4146` `:4167` `:4257` `:4325` `:4353` `:4690`. 🔴 Đóng bằng **nối tiếp**
      `→ ✅ ĐÃ ĐÓNG <ngày> (Story 2.12)`, **không xoá**. Đóng một nửa ⇒ **🟡 kèm phần còn hở**
- [ ] 9.2 Các mục chủ *"story hạ tầng cổng"*: `:4684` *(`resetEditorPanel`)* đóng bởi AC5; `:4288`
      *(`@keydown`)* · `:4337` *(cử chỉ chuột)* · `:4251` *(`ORDER BY ord`)* — **ngoài phạm vi**, giữ
      chủ hoặc đổi chủ **tường minh**
- [ ] 9.3 Mọi vế không nghiệm thu được ⇒ nợ mới **có chủ**, không mục nào mồ côi
- [ ] 9.4 `wdio.conf.mjs:70-80` — cập nhật bản ghi lượt chạy kèm 🔵 và ngày, **sửa tại chỗ** mệnh đề
      đã hết đúng thay vì để nó lặng lẽ sai

---

## Dev Notes

### Đường dây đã có — Story 1.22 dựng nền, đừng dựng lại một mảnh nào

⚠️ **`1-22-*.md` KHÔNG TỒN TẠI.** Story 1.22 được theo dõi **hoàn toàn trong `sprint-status.yaml`**
*(dòng `106`, `in-progress`)*. Đừng đi tìm một tệp story.

**Đã có, `done`, kế thừa nguyên:**

| Thứ | Chỗ | Ghi chú |
|---|---|---|
| Cách ly `$APPDATA` | `AURATRANSLATE_E2E_DATA_DIR`, hai lớp chặn AD-45 | Đo: `global.db` **không đổi một byte** sau hai lượt spec |
| Cách ly Library | `AURATRANSLATE_E2E_LIBRARY_ROOT` | Bề mặt dữ liệu thật **thứ hai** |
| Tự kiểm dương tính | `wdio.conf.mjs:287-343` | Xanh mà **không** thấy `global.db` trong thư mục tạm ⇒ **tự đỏ** |
| Chuột thật | `e2e/support/pointer.mjs:49-58` `realClick()` | Driver bắn `click` **trước** `focusin` — ngược chuột thật |
| Cổng cấm `.click()` | `eslint.config.js:142-155`, `no-restricted-syntax` | `check:lint` nay chạy `eslint src e2e tests` |
| Fixture workspace | `e2e/support/workspace.mjs` | Chính chỗ AC2 sẽ sửa |
| Chạy tuần tự | `maxInstances: 1`, cổng 4445 cố định | Quyết định **có chủ**, không phải một giới hạn |

**Còn hở — chính là story này** *(`sprint-status.yaml:1777-1779`)*: *"🔴 BỘ E2E CHẬP CHỜN — đính chính
bản ghi C3 hôm qua (nói 'ổn định' trên n=2)."*

### Cách chạy — bốn lệnh, và hai thứ CỐ Ý nằm ngoài `pre-push`

```
npm run test                # vitest — 4 s
cargo test --locked         # 34 s  (chạy trong src-tauri/)
npm run test:e2e            # cargo build --features wdio  &&  wdio run e2e/wdio.conf.mjs
npm run check:gates         # canh BA danh sách sau khi thêm cổng mới
```

⚠️ **Bộ e2e chạy TAY** *(~15 phút, cần máy rảnh)*, và nó **tự dựng Vite trên cổng 1420** rồi tự tắt.
🔴 **Va chạm cổng phải biết trước:** `check:scope` và `check:scope:bundled` **cũng cần cổng 1420
trống** và cũng nằm ngoài `pre-push`. Chúng **không** chạy cùng lúc với bộ e2e, và cũng trượt nếu
đang mở `npm run tauri dev`.

⚠️ `check:lint` chạy `eslint src e2e tests` — mọi tệp helper **mới** trong `e2e/support/**` đều bị
cổng soi, gồm cả luật cấm `.click()`.

### 🔴 Tám cạm bẫy — mỗi cái có bằng chứng, không một khả năng lý thuyết

1. **Cổng đếm `ref` đỏ oan trên `.vue`.** `<script setup>` chạy lại mỗi lần mount ⇒ ref ở đó **không**
   là state cấp module. Một cổng không phân biệt sẽ đỏ ở `LookupPanel.vue:189,225` · `PanelFrame.vue:78`
   và hàng chục chỗ nữa — rồi bị nới cho hết đỏ, và cổng thành vô nghĩa.
2. **Sửa bộ đo bằng cách nới hằng số.** B2 cấm đích danh. `FLUSH_WAIT_MS` to hơn làm mọi ca **xanh**
   và làm NFR18 **hết được canh** — đúng khuôn *"một cổng cho exit 0 trên một sản phẩm đang hỏng"*.
3. **Một bản đồ CHÉP hàm sản phẩm.** 2.9 đã trả giá: bản đồ chép hàm sản phẩm vẫn cho **17** và **19**
   y hệt lượt trước **sau khi đã vá**, tức nó **báo "chưa vá" trên một sản phẩm ĐÃ VÁ**.
4. **Một listener chẩn đoán tự ném.** 2.8: listener gọi `caretPositionFromPoint` trần ⇒ ném **trước**
   dòng `push` ⇒ mảng rỗng mọi vòng ⇒ đọc thành *"không một `mouseup` nào tới `document`"*. **Một mệnh
   đề về ENGINE rút ra từ một cú ném của BẢN ĐỒ.**
5. **Kết luận rỗng không có đối chứng dương.** 2.10: `browser.keys(['Meta','/'])` giao `code: "/"`
   chứ không `"Slash"` ⇒ **0 command chạy**; đối chứng `⌘M` giao `code: "KeyM"` ⇒ khớp. Không có đối
   chứng dương thì *"0 kết quả"* đọc thành *"đã xác nhận"*.
6. **Trần `mochaOpts.timeout` 120 s.** Ba ca **đã** đụng *(`deferred-work.md:4325-4335`)*; ~5 s mỗi
   lệnh WebDriver cộng dồn. Một fixture nặng thêm ở Task 2 có thể đẩy ca thứ tư qua trần — và nó sẽ
   trông y hệt một hồi quy sản phẩm.
7. **Vá bằng `?.` trong mã sản phẩm cho hết đỏ.** `project-context.md:278-281`: đó là một nhánh mà
   **kiểu nói không bao giờ chạy** — mã chết vĩnh viễn trong sản phẩm để phục vụ một bản mô phỏng.
8. 🔴 **Một cửa sổ TRẮNG trông giống hệt một ứng dụng chưa kịp render** — và nó **XANH ở mọi khẳng
   định *"không tìm thấy"***. `wdio.conf.mjs:181-187`, đo được ở lượt chạy đầu của chính bàn đo này:
   nhị phân **debug** nạp `devUrl` *(không `frontendDist`)* và cho `url: "about:blank"` với
   `document.body` **rỗng**. ⇒ Đây là **vế thứ hai** của AC1: `devServerIsUp` canh phía **server**;
   một cửa sổ trắng là phía **client**, và không phép kiểm nào hôm nay canh nó. Mọi khẳng định phủ
   định trong spec cần một **đối chứng dương** đi kèm.

### Ranh giới phạm vi — sáu thứ KHÔNG thuộc story này

1. **Story 2.4 / NFR2.** Cửa chặn ① *(B1)*, chủ là **Ice**. Đo được **706-770 ms** *(trần 50 ms)*.
   Story này **không đo, không vá, không chấm**.
2. **Story 2.3.** *(B3)* Tiền đề chặn đã hết đúng, cần một lượt đóng có chủ **riêng** — 17 AC, đừng
   chấm bằng suy luận.
3. **`AD-48`** *(mô hình hoàn tác)* — *(B6)* Ice → Winston. Đang chặn AC5 của Story 2.9.
4. **Bảng nghiệm thu Windows** *(B7)* và **28 hàng bàn đo của 1.20/1.21** *(B8)* — chủ là Ice.
5. **Hai luật F4 vào `project-context.md`** *(B9)* — chủ là Dev, nhưng là một lượt **tài liệu riêng**.
   Story này **dùng** hai luật đó *(cạm bẫy 3 và 4)*, không **ghi** chúng.
6. **Mọi FR của Epic 3.** Story này mở cửa cho Epic 3, không làm một phần nào của nó.

### Nghiệm thu — bốn đường, bốn vai, chọn đúng đường

| Mệnh đề | Đường | Vì sao không đường khác |
|---|---|---|
| *"Mọi ô nhớ cấp module phải qua hàm reset"* | **cổng tĩnh** *(mới)* | Mệnh đề **khai báo trên toàn cây** — đúng vai cổng tĩnh |
| *"Một cột `segment` mới phải xuất hiện ở mọi chỗ bắt buộc"* | **cổng tĩnh** *(mới)* hoặc **`cargo test`** | Xem chữ ký #3 — **chưa chốt**, và chọn sai là dựng nguồn sự thật thứ hai |
| *"`devServerIsUp` phân biệt Vite lành với Vite hấp hối"* | **tự kiểm trong chính bộ e2e** | Nó là mã hạ tầng của bộ đo, không phải mã sản phẩm |
| *"Không state panel nào sống sót giữa hai spec"* | **e2e** *(WKWebView thật)* | `happy-dom` **không phải** WebKit, và rò rỉ này sống ở tầng phiên app |
| *"Flush đã vào WAL"* | **e2e** + hợp đồng Rust đã có | AD-35: một flush chỉ **xong** sau khi đã ghi vào **WAL** |
| *"Hằng AD-35 không bị nới"* | **đọc mã** + `cargo test` hợp đồng | Một mệnh đề về **hằng số**, không về hành vi |

🔴 **Chọn sai đường là dựng nguồn sự thật thứ hai.** Trước khi viết một phép kiểm mới, hỏi: **mệnh đề
này đã có chủ ở đường nào chưa.**

### Điều kiện khởi hành — baseline ĐO LẠI TỪ NGUỒN 2026-08-18, HEAD `4b30199`

| Thứ | Số | Cách đo |
|---|---|---|
| `cargo test --locked` | **409 passed / 0 failed / 5 ignored** | chạy thật |
| `vitest` | **249 / 249**, **21 tệp**, 4,05 s | `npx vitest run` *(vitest 4.1.10)* |
| Cổng đọc-tệp | **9/9 exit 0** | retro 2026-08-18 |
| Spec e2e | **11 tệp · 21 `it()`** | `grep -cE "^\s+it\(" e2e/specs/*.e2e.mjs` |
| `browser.pause()` trong `e2e/specs/` | **25** | `grep -rn "browser.pause(" e2e/specs/ \| wc -l` |
| Ô nhớ cấp module ngoài `resetEditorPanel` | **4** | `editorPanelState.ts:886, 889, 1404, 2005` |
| Tệp `src/panels/**` không có hàm reset nào | **2** | `dictSourcesState.ts` · `segmentHistoryState.ts` |
| `INSERT INTO segment` | **2** *(`segment.rs:132` nhập · `:2234` regroup)* | `grep -rn "INSERT INTO segment" src-tauri/src` |
| Cột bảng `segment` mà `SegmentRow` đọc | **13** | `segment_contract.rs:2236-2261` |
| `FLUSH_WAIT_MS` vs `EDITOR_IDLE_MS` | **3500** vs **2000** ⇒ biên **1500 ms** | `editor-typing-flush.e2e.mjs:71` · `editorFlush.ts:43` |
| Sàn `check-layout` vs số thật `src/**` | **43** vs **55** *(chú thích ghi 52)* | `check-layout.mjs:101` |
| Bộ di trú `project.db` | đích **11** ⇒ bước kế tiếp **12** | `schema.rs` |

🔴 **Task 0.1 phải ĐO LẠI cả bảng, không chép.** Bài học lặp lại nhiều lần trong epic này: **một số
chép là một số sẽ lệch trong im lặng.** *(Story 2.8 đã bắt được một tiền đề của chính tệp story sai
số — grep ghi 0, đo được 2. Story 2.11 đo lại 7 tiền đề, 7/7 khớp, và vẫn tìm ra một chỗ story dẫn
tiền lệ SAI: `libraryImport.ts:145` chỉ gọi flush **một lượt**, không phải khuôn hai lượt.)*

⚠️ Và một cảnh báo riêng cho story này: retro tự khai bốn nguyên nhân của F3 mới *"đã đo được **một
PHẦN**"*. Riêng *"fixture không reset state panel"* **không có** một mục nợ tường minh nào trong
`deferred-work.md` — nó là quan sát rút ra từ chỗ rải rác. **Đo lại từ nguồn trước khi tin.**

### Git — trạng thái cây khi story này được soạn

`git status --short` = **rỗng**. HEAD = `4b30199`
*(`docs(retro-epic-2): một lượt đổi tên đã ký vẫn để lại 11 chỗ tên chết…`)* — cây bẩn của lượt retro
đã được commit **riêng, trước**, đúng `project-context.md:425-426`.

⇒ Hai thứ chưa theo dõi duy nhất sau lượt soạn này là **tạo tác của chính story 2.12** *(tệp story +
entry `sprint-status.yaml`)* ⇒ **không** commit riêng.
⚠️ Nếu tới lúc dev cây đã bẩn vì thứ khác: **hỏi Ice, commit riêng, TRƯỚC dòng mã đầu tiên**.

### Project Structure Notes

**Tệp sẽ chạm** *(dự kiến; chữ ký của Task 0 có thể đổi danh sách)*:

| Tệp | Loại | Việc |
|---|---|---|
| `e2e/wdio.conf.mjs` | UPDATE | `devServerIsUp` *(:191-198)* · bản ghi lượt chạy *(:70-80)*. 🔴 **Đừng phá** tự kiểm `global.db` *(:287-343)* |
| `e2e/support/workspace.mjs` | UPDATE | fixture reset *(:55-96)* |
| `e2e/support/*.mjs` | NEW | khuôn chờ trạng thái đích |
| `e2e/specs/*.e2e.mjs` | UPDATE | 🔴 cấm `.click()` của driver — dùng `realClick()` |
| `scripts/check-*.mjs` | NEW ×1-2 | theo chữ ký #7 |
| `package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push` | UPDATE | **ba danh sách**, `check:gates` Kiểm D/E/F canh |
| `src-tauri/tests/segment_contract.rs` | UPDATE | **chỉ nếu** #3 = (a)/(c) |
| `src/StatusBar.vue` · `src/panels/editorFlush.ts` | UPDATE | **chỉ nếu** #4 = (b)/(c) — và đó là **cửa `AD`** |
| `tests/frontend/**` | NEW/UPDATE | 🔴 **KHÔNG** đồng vị trí trong `src/**` |
| `deferred-work.md` | UPDATE | 11 mục đóng bằng **nối tiếp**, không xoá |

**Quy ước bắt buộc:**
- **Node thuần, không bash** trong `scripts/` — `npm run` trên Windows đi qua `cmd.exe`.
- **Không thêm phụ thuộc npm cho một cổng.** Parser là **tập con nghiêm ngặt tự viết**.
- **Mã thoát là phán quyết.** `abort()` cho lỗi hạ tầng ≠ `fail()` cho một phép kiểm đỏ.
- **Không phán quyết nào đọc tham số từ chính thứ nó đang kiểm.**
- 🔴 `Ref` **không** tự bóc trong `<script>` — `if (someRef)` chạy trên **đối tượng** và **luôn đúng**.

### References

**Nguồn của story này**
- `epic-2-retro-2026-08-18.md:373` — **action item B2**, nguyên văn sáu món
- `epic-2-retro-2026-08-18.md:340-352` — **hai cửa chặn Ice ký**, và cảnh báo số hiệu `2-12`
- `epic-2-retro-2026-08-18.md:146-161` — **F2** *(vitest không bắt được lớp lỗi đắt nhất, 4 lần)*
- `epic-2-retro-2026-08-18.md:163-181` — **F3** *(e2e là lưới duy nhất **và** là bộ đo nói dối)*
- `epic-2-retro-2026-08-18.md:129-144` — **F1** *(chữ ký thi hành đúng một nửa, 5 lần)*
- `epic-2-retro-2026-08-18.md:183-201` — **F4** *(một con số THẬT trả lời SAI câu hỏi)*
- `sprint-status.yaml:1777-1779` — bộ e2e chập chờn, chuyển chủ sang story này

**Kiến trúc**
- `ARCHITECTURE-SPINE.md:419-425` — **AD-35** hợp đồng flush *(năm vế; *"chỉ xong sau khi đã ghi vào WAL"*)*
- `SPINE:635-650` — **AD-45** *(0 cổng lắng nghe; hai lớp chặn; một `cfg` một mình **không đủ**)*
- `SPINE:81-87` — **AD-2** *(một bề mặt mới là một `AD` mới)*
- `SPINE:406-417` — **AD-34** *(CommandRegistry, focus tường minh)*
- ⚠️ **`AD-48` CHƯA TỒN TẠI** — spine dừng ở AD-47. Hồ sơ bàn giao:
  `ad-brief-2026-08-17-mo-hinh-hoan-tac.md`

**Mã — bộ đo**
- `e2e/wdio.conf.mjs:191-198` *(`devServerIsUp`)* · `:70-80` *(bản ghi 8 lượt)* · `:287-343` *(tự
  kiểm `global.db`)* · `:116,:127` *(hai biến chuyển hướng)* · `:227` *(`mochaOpts.timeout` 120 s)*
- `e2e/support/workspace.mjs:55-96` · `e2e/support/pointer.mjs:49-58`
- `eslint.config.js:142-155` *(cấm `.click()`)*

**Mã — sản phẩm**
- `src/panels/editorPanelState.ts:509-612` *(`resetEditorPanel`)*; ngoài nó: `:886 :889 :1404 :2005`
- `src/panels/sourcePanelState.ts:360` · `lookupPanelState.ts:318` · `lookupHistoryState.ts:360`
- `src/panels/editorFlush.ts:43,56` *(`EDITOR_IDLE_MS` · `EDITOR_HARD_CAP_MS`)*
- `src/layout/writeSchedule.ts:32-33` — bảng đối chiếu **hai cặp hằng**; 🔴 chỉ cặp Editor mang bảo
  đảm AD-35, **đừng gộp hai cặp**
- `src/StatusBar.vue:235` *(*"Đã lưu N giây trước"*)*
- `src-tauri/src/commands/segment.rs:2210,2234` *(`write_regroup`)* · `:132` *(đường nhập)*

**Mã — cổng làm khuôn**
- `scripts/check-layout.mjs:39-51` *(abort/fail)* · `:101` *(sàn)* · `:404-479` *(danh sách cho phép)*
  · `:556-617` *(**tự kiểm**)*
- `scripts/check-gates.mjs:87-130` *(ba `Map` miễn trừ có tên)* · `:311-425` *(**tự kiểm**)*
- `src-tauri/tests/segment_contract.rs:2236-2261` *(tự kiểm đếm cột)* · `:2333` *(**cổng-AC8**)*
- `package.json:9-25` · `.githooks/pre-push:62,76`

**Sổ nợ — 11 mục story này nhận chủ**
- `deferred-work.md:3402` `:3671` `:3722` `:3978` `:4136` `:4146` `:4167` `:4257` `:4325` `:4353` `:4690`
- Và `:4684` *(`resetEditorPanel` không cổng nào canh)* — đóng bởi **AC5**

**Story trước**
- `2-11-chuyen-chuong-trong-workspace.md` — khuôn Task 0 · bài học *"đột biến mã sản phẩm"* · lượt
  phân xử e2e **trên cả hai cây** *(story lẫn baseline)*
- `2-10-dieu-huong-segment.md:1159-1175` — `editorHasLoaded` bị quên; vá bằng **thunk** vì *"không tồn
  tại cú pháp để gọi một lệnh điều hướng mà đi vòng qua cửa"* — khuôn thiết kế đáng mượn cho AC5
- `2-8-gop-va-tach-segment-tuong-minh.md:508` — lỗ `write_regroup` của cổng-AC8

**Luật kho**
- `_bmad-output/project-context.md` — 131 luật, đặc biệt `:253-338` *(Testing Rules, bốn đường)* ·
  `:300-321` *(luật của một CỔNG)* · `:456-466` *(story và spec)*. Đọc **trước** dòng mã đầu tiên.

### Thông tin kỹ thuật mới nhất

⚠️ **Story này KHÔNG cần một phụ thuộc mới**, nên cửa NFR15 *(rà giấy phép ba bước —
`project-context.md:92-100`)* **không** mở. Ghi ra thay vì im lặng: một lượt *"tiện tay thêm một
parser"* ở đây là một lượt đi vòng qua một cửa đang đứng. 🔵 **Cửa đó vẫn đứng** kể cả sau lượt lật
2026-08-12 *(vitest + `@vue/test-utils` + happy-dom đã vào qua nó, không đi vòng)*.

**Phiên bản đang dùng, đã rà giấy phép** *(bảng Stack của spine)*: `@wdio/cli` · `@wdio/local-runner` ·
`@wdio/mocha-framework` · `@wdio/spec-reporter` **9.30.1** MIT · `@wdio/tauri-service` **1.3.0** MIT ·
`tauri-plugin-wdio-webdriver` **1.3.0** MIT *(`optional`, feature `wdio`, chỉ debug — AD-45)* ·
`vitest` **4.1.10** MIT · `happy-dom` **20.11.2** MIT.

🔴 **Không nâng phiên bản nào trong story này.** Một lượt nâng `@wdio/*` giữa lúc đang chẩn đoán một
bộ đo chập chờn làm **mọi số đo trước đó hết so sánh được** — cùng lý do CI ghim
`dtolnay/rust-toolchain@1.97.1` chứ không `@stable`.

---

## Dev Agent Record

### Agent Model Used

### Chữ ký Task 0

| # | Quyết định | Chữ ký | Ngày | Đường bị loại và vì sao |
|---|---|---|---|---|
| 1 | Windows/Blink | | | |
| 2 | Phạm vi cổng AC5 | | | |
| 3 | Cổng AC6: tĩnh hay Rust | | | |
| 4 | Tín hiệu flush *(cửa `AD`?)* | | | |
| 5 | Cơ chế fixture reset | | | |
| 6 | Vá `devServerIsUp` | | | |
| 7 | Cổng thứ 10/11 hay gộp | | | |
| 8 | Ngưỡng "tái lập được" | | | |

### Debug Log References

### Completion Notes List

### File List

### Change Log

### Review Findings

---

## Câu hỏi cho Ice — chốt ở Task 0, trước dòng mã đầu tiên

**⑴ 🔴 `epics.md` không có mục cho Story 2.12 — ai bổ sung, và có bổ sung không?**
Story 1.22 *(hạ tầng e2e lần đầu)* **có** mục trong `epics.md:1989-2036`. Story này thì không — nó
sinh từ action item retro. Ba đường: **(a)** giữ nguyên, story sống bằng retro + `sprint-status.yaml`
*(nhưng khi ai đó tra `epics.md` giữa Epic 2 và Epic 3, thứ họ thấy là khối `~~Sync Scrolling~~ — XOÁ`)*;
**(b)** một lượt **correct-course** có phạm vi, chủ là PM/Winston; **(c)** dev thêm thẳng.
🔴 **(c) đi ngược `project-context.md:456-466`** — đó là sửa tài liệu quy hoạch ngoài thủ tục.
Mình **không** tự chọn, và **không** tự sửa `epics.md`.

**⑵ ✅ ĐÃ CHỐT 2026-08-18 — Story 2.4 gỡ TRƯỚC.** Xem §Thứ tự ở đầu tệp. Hệ quả mang vào Task 0.1:
bảng §Điều kiện khởi hành chụp HEAD `4b30199` là số **trước** 2.4 ⇒ **đo lại sau khi 2.4 đóng**,
đừng dùng nó như một số đã chốt.

**⑶ Sổ nợ đã thành *"một hàng đợi không có người phục vụ"* — story này đóng 11 mục, còn ~146 thì sao?**
~157 mục ghi trong khoảng Epic 2, **22** đóng; ~**30** mục chủ là **Ice** đích danh. Retro nói thẳng
*"sổ nợ vẫn TRUNG THỰC — nhưng nó đã thành một hàng đợi không có người phục vụ"*. Đây là câu hỏi về
**hình dạng**, không phải về story này.

**⑷ Ba lượt commit gần đây mang message tiếng Anh** — `67bb147` · `76a42dc` · `1092d38` · `a664dac` ·
`0f67808` · `4d72cd4` · `dfa9c95` đều dạng `feat: implement …`, không phải `type(scope): câu tiếng
Việt` **nói điều đã tìm ra** *(`project-context.md:422-424`)*. Không cổng nào canh chuyện này. Ghi ra
thay vì để trôi — Ice muốn đóng bằng một luật, một hook, hay để nguyên?

**⑸ B10 vẫn chưa có chủ và chưa có lịch** — *"mọi phát hiện đắt nhất của Epic 2 đến từ Ice dùng app
thật, và nó không có chủ, không có lịch, không có tên trong sổ."* Story này **không** đóng B10, và
nó là thứ duy nhất trong retro mà một story không thể đóng được.
