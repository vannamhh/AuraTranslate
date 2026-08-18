---
baseline_commit: 5d94ba181cf6ede4fcecaa3acce97c4c540c0f97
---

# Story 2.11: Chuyển Chương trong Workspace

Status: review

**Covers:** FR26
**Epic:** 2 — Biên tập theo segment
**Soạn:** 2026-08-18 · trên HEAD `5d94ba1`, cây làm việc **sạch**

---

## Story

As a **người dịch**,
I want **sang Chương kế tiếp mà không phải quay về Library**,
So that **mạch làm việc của tôi không bị cắt**.

---

## 🔴 ĐỌC TRƯỚC DÒNG MÃ ĐẦU TIÊN — tiền đề của BA trên sáu AC không tồn tại hôm nay

**Một Tác phẩm có ĐÚNG MỘT Chương.** Đây không phải một ấn tượng, nó đo được từ ba phía:

| Phép đo | Kết quả | Nguồn |
|---|---|---|
| `grep -rn "INSERT INTO chapter" src-tauri/src` | **1** kết quả duy nhất | `commands/project.rs:138` |
| Hàng đó chèn gì | `VALUES (1, NULL, ?1, ?2, …)` — `ord = 1` **viết cứng**, một lượt, không vòng lặp | `project.rs:130-145` |
| `grep -rn "list_chapters\|read_chapters" src-tauri/src src` | **0** kết quả | — |

Và hai lệnh đọc đều chọn Chương bằng **cùng một câu SQL cứng**, không nhận tham số:

- `commands/chapter.rs:77` — `SELECT id, source_text FROM chapter ORDER BY ord LIMIT 1`
- `commands/segment.rs:833` — `SELECT id FROM chapter ORDER BY ord LIMIT 1`

⇒ **Hệ quả phải nhìn thẳng, không làm tròn lên:**

- **AC1 và AC2** *(Chương sau / Chương trước mở ra)* — trên **mọi** `.atproj` tồn tại hôm nay
  *(21 Tác phẩm thật, mỗi cái đúng 1 Chương — `deferred-work.md:559-560`)* **không có Chương thứ hai
  để mở**. Không đường sản phẩm nào, không đường e2e nào *(`e2e/support/workspace.mjs:63` chỉ có
  `create_work_from_text`)*.
- **AC4** *(hành vi ở biên)* là ca **DUY NHẤT** với tới được bằng sản phẩm hôm nay — vì mọi Chương
  hiện có vừa là Chương đầu **vừa là** Chương cuối. Đây là AC dễ nghiệm thu nhất, không phải khó nhất.
- **AC5** *(khôi phục segment + vị trí cuộn)* phát biểu đúng nguyên văn **FR12**, mà bảng ánh xạ
  giao FR12 cho **Epic 5** *(`epics.md:660`)*. Story này khai `Covers: FR26` và **chỉ** FR26
  *(`epics.md:2625`)*.

**Ba AC còn lại thì lành:** **AC3** *(flush)* có nguyên hạ tầng để tái dùng · **AC4** *(biên)* với
tới được như trên · **AC6** *(command đăng ký)* là một khuôn đã lặp nhiều lần. ⇒ Story này **không**
bị chặn — nó bị **chia làm hai nửa có tính chất khác nhau**, và Task 0 tồn tại để Ice chọn cách giao
nửa kia.

**Đường sinh ra Chương thứ hai thuộc epic khác:** FR14 *(nhập hàng loạt + mẫu phân tách ⇒ nhiều
Chương)* → **Epic 6** *(`epics.md:662`)*; FR15 *(gộp/tách/sắp lại Chương, AD-32)* → **Epic 5**
*(`epics.md:663`)*.

🔴 **Đây KHÔNG phải một lý do sửa `epics.md`.** `project-context.md:456-458`: *"Năng lực chưa dựng
≠ lệch spec. Một AC mô tả đích đến không sai chỉ vì đường đi chưa tới đó."* ⇒ Việc của Task 0 là
**hỏi Ice chọn hình dạng giao hàng**, không phải tự thu hẹp AC rồi chấm đạt.

---

## Acceptance Criteria

*(Nguyên văn `epics.md:2627-2656`. Số hiệu do story này gán để Task tham chiếu được.)*

1. **AC1** — **Given** một Chương đang mở trong Workspace · **When** gọi lệnh **Chương sau** ·
   **Then** Chương kế tiếp trong cùng Tác phẩm mở ra.
2. **AC2** — **Given** người dùng gọi lệnh **Chương trước** · **When** xảy ra · **Then** Chương
   liền trước mở ra.
3. **AC3** — **Given** chuyển Chương · **When** xảy ra · **Then** văn bản đang gõ ở Chương cũ được
   **flush trước khi chuyển**.
4. **AC4** — **Given** Chương đầu tiên hoặc Chương cuối cùng của Tác phẩm · **When** gọi lệnh vượt
   biên · **Then** báo rõ đã ở biên, không sập và **không quay vòng im lặng**.
5. **AC5** — **Given** một Chương được mở lại về sau · **When** mở · **Then** khôi phục đúng
   segment và vị trí cuộn lần trước.
6. **AC6** — **Given** hai lệnh này · **When** gọi · **Then** là command đăng ký, gán phím được.

---

## 🔴 Task 0 — CỬA CHẶN: tám quyết định mở, phải có chữ ký của Ice

**Không một dòng mã sản phẩm nào được viết trước khi tám mục dưới đây có chữ ký.**
Trình mỗi quyết định **kèm số đo hoặc trích dẫn nguồn**, không kèm một khuyến nghị đã tự chốt.

🔴 **Task 0.4 là một CỬA CHẶN THẬT, khuôn này ĐÃ KÍCH HOẠT hai lần trong Epic 2** — AD-47 giao
Winston ở Story 2.7, AD-48 giao Winston ở Story 2.9. Quyết định **#7** ở đây là ứng viên trực tiếp.
Nếu nó kích hoạt: **dừng story**, soạn **hồ sơ bàn giao**, **đừng tự soạn AD**.

---

### Quyết định #1 🔴 — Story này giao cái gì, khi tiền đề của AC1/AC2 không tồn tại

**Số đo:** xem §ĐỌC TRƯỚC ở trên — 1 lượt `INSERT INTO chapter`, `ord = 1` viết cứng, 0 hàm liệt kê
Chương, 0 đường sản phẩm sinh Chương thứ hai.

| Đường | Nội dung | Cái giá |
|---|---|---|
| **(a)** | Dựng **trọn** cơ chế *(Rust + lệnh + báo biên)*; AC1/AC2 nghiệm thu bằng **test hợp đồng Rust** chèn Chương thứ hai bằng SQL trực tiếp; ghi nợ có chủ *"không đường sản phẩm nào sinh Chương thứ hai — chủ: Epic 6"* | AC1/AC2 **không** có đường e2e, và một tính năng xanh 100% mà chưa ai bấm được |
| **(b)** | 2.11 dựng thêm một đường tạo Chương thứ hai | Lấn FR14/Epic 6, và AD-39 *(SPINE:468-496)* đóng băng thứ tự các bước nhập ⇒ một đường nhập thứ hai là một quyết định kiến trúc |
| **(c)** | Dời 2.11 xuống sau Epic 6 | Epic 2 đóng lại với FR26 chưa phủ; thứ tự thực thi đã có chữ ký *(`sprint-change-proposal-2026-08-13b`)* |

⚠️ **Tiền lệ cho đường (a) đã có chữ ký hai lần:** chữ ký #8(a) của Story 2.6 *(`retired_at` — test
hợp đồng dùng SQL trực tiếp vì chưa bề mặt nào sinh ra giá trị đó)*, và AC3 của Story 2.7 *(xuất xứ
phi-`self` — cùng lý do)*. Cả hai đều **ghi nợ có chủ** thay vì tự chấm đạt.

### Quyết định #2 🔴 — "Chương đang mở" sống ở đâu

**Số đo:** `OpenWork` *(`commands/project.rs:43-53`)* mang đúng bốn trường `dir` · `store` · `scope`
· `meta` — **không** trường nào là `chapter_id`. `OpenWorkState = Mutex<Option<OpenWork>>`
*(`project.rs:246`)*, thay bởi `replace_open_work` *(`:268-279`)*. Tức hôm nay *"Chương đang mở"*
**không được lưu ở đâu cả** — nó được **suy ra động** mỗi lượt gọi bằng `ORDER BY ord LIMIT 1`.

- **(a)** Thêm một trường vào `OpenWork` — Rust giữ, hai lệnh đọc hỏi nó.
- **(b)** Webview giữ *(`editorPanelState.ts:43` đã có `chapterId`)* và **truyền qua dây** mỗi lượt
  đọc. ⚠️ Đụng AD-1: *"Không quy tắc nghiệp vụ nào ở TypeScript"* — câu hỏi phải trả lời là
  **"Chương nào đang mở" là state UI hay là một quy tắc nghiệp vụ?**
- **(c)** Lưu **xuống đĩa** — kéo theo Quyết định #4, và làm AC5 thành hệ quả miễn phí thay vì một
  hạ tầng thứ hai.

🔴 **RÀNG BUỘC CỨNG, và nó là một lỗ MẤT DỮ LIỆU không AC nào nêu:** đường ghi
`save_segment_targets` / `flush_segment_targets` **nhận `chapter_id` từ webview**
*(`segment.rs:1112-1116` · `:1828-1836` · vỏ `:2601-2625`)*. Nếu Chương đổi trong lúc **một lô flush
đang bay**, lô đó mang `chapter_id` **CŨ** ⇒ Rust trả `segment.unknown_ids` ⇒ **bản dịch biến mất
trong im lặng**. Đây **đúng** lớp lỗi mà `modes/libraryImport.ts:119-132` đã ghi ra bằng chữ cho
lượt đổi **Tác phẩm**, và lời giải ở đó là *"flush TRƯỚC lượt `replace_open_work`"* — không phải
*"flush trước `resetEditorPanel()`"*.

### Quyết định #3 — hình dạng lệnh Rust

**Mã sản phẩm đã giao đích danh story này, hai chỗ, bằng chữ:**

> `commands/segment.rs:773-775` — *"Story 2.11 sở hữu biến thể nhận `chapter_id`. **Đừng** thêm sẵn
> một tham số `Option<i64>` hôm nay: một nhánh không chỗ gọi nào đi qua là một nhánh không ai nghiệm
> thu được."*
>
> `commands/chapter.rs:72-74` — *"Chọn Chương / chuyển Chương là Epic 2 — không thuộc phạm vi story
> này."*

- **(a)** Một lệnh `open_adjacent_chapter(direction)` — **Rust** quyết Chương kề, webview chỉ nói
  hướng. Hợp AD-1 nhất.
- **(b)** `list_chapters()` + webview tự chọn — webview phải mang luật *"kề là gì"*, đụng AD-1.
- **(c)** Thêm `chapter_id: Option<i64>` vào hai lệnh đọc hiện có — nhưng chú thích trên **cấm
  tường minh** hình dạng `Option` chưa có chỗ gọi.

🔴 **CẤM `ord + 1`, và đây là một luật đã có tiền lệ trong chính kho:** `ord` **cố ý không `UNIQUE`**
*(`schema.rs:249`, doc-comment `:233-235`)* và không bảo đảm liên tục. Lượt tìm câu liền trên của
Story 2.8 đã viết thẳng lý do: giả định `ord` liên tục *"sẽ làm một phép trừ im lặng trỏ sai hàng"*
⇒ dùng **so sánh bộ đôi** `ORDER BY ord, id` / `ORDER BY ord DESC, id DESC`, đúng khuôn
`segment.rs` đã dùng.

### Quyết định #4 🔴 — AC5: dựng bây giờ, hay ghi nợ cho Epic 5

**Số đo — hôm nay KHÔNG có một mảnh hạ tầng nào:**

| Thứ | Trạng thái | Nguồn |
|---|---|---|
| `ScopeKind` cho vị trí đọc | **0/9** — chín loại là `Glossary` · `Prompt` · `AiConfig` · `TranslatorName` · `TranslationMemory` · `ImportCleanupRule` · `Shortcut` · `LayoutPreset` · `AppConfig` | `core/scope/kinds.rs:157-219` |
| Bảng trên đĩa | `config_value` nằm ở **`global.db`**, chỉ phục vụ ba loại `GlobalOnly`, cột `value TEXT` phẳng | `schema.rs:98-105` |
| Tiền lệ lưu một vị trí đọc | `grep -rn "scroll" src-tauri/src` = **0** · `grep "lastOpened\|last_opened"` = **0** | — |
| Cuộn trong lưới | **không** một dòng `scrollIntoView`/`scrollTop` nào — cuộn đến từ **hành vi engine** sau `target.focus()` | `GridPanel.vue:903-923`, phép đo `:934-964` |

- **(a)** Chỉ trong **PHIÊN** *(bộ nhớ webview)* — đọc *"mở lại về sau"* là *"trong cùng phiên chạy"*.
- **(b)** Lưu xuống **`project.db`** bằng một bảng riêng *(khuôn `pinned_entry`, `schema.rs:158-167`
  — `config_value` **cấm** nhồi dữ liệu nhiều trường)*. ⇒ **Buộc bước di trú 12** và ba neo số học
  ở §Cạm bẫy ⑤.
- **(c)** Ghi nợ **trọn** AC5 cho Epic 5 *(FR12 là của Epic 5)*, story này giao 5/6 AC — đúng khuôn
  chữ ký ① của Story 2.9 *(AC5 `⌘Z` giao 5/6 AC, ghi nợ có chủ)*.

🔴 **Nếu chọn (b): "vị trí cuộn" lưu bằng GÌ.** AD-3 *(SPINE:93)* nói bằng chữ: *"Mọi dữ liệu gắn
theo segment tham chiếu `id`, **không bao giờ tham chiếu vị trí**."* Một `scrollTop` pixel vô nghĩa
ngay khi người dùng đổi cỡ chữ hoặc kéo sash; một `segment.id` thì bền theo AD-3 nhưng **không**
phải *"vị trí cuộn"* theo nghĩa đen của AC.

⚠️ **Hạ tầng này có BA chỗ tiêu thụ, không riêng 2.11** — UX-DR34 *(`epics.md:601`)* đòi y hệt cho
lượt **đổi chế độ**: *"rời Workspace sang Chế độ đọc rồi quay lại thì vẫn đúng Chương, đúng câu,
đúng vị trí cuộn"*. Chọn hình dạng ở đây là chọn cho cả ba.

### Quyết định #5 — kênh báo biên (AC4)

**Số đo:** `NavNotice` là **danh mục ĐÓNG năm giá trị** —
`'no-untranslated' | 'at-first' | 'at-last' | 'confirmed-last' | 'loading'`
*(`editorPanelState.ts:1443-1485`)*. `StatusBar.vue::NAV_NOTICE_KEYS` *(`:195-203`)* là một `Record`
**đủ khoá** trên nó ⇒ thêm một giá trị mà quên bảng tra thì **`vue-tsc` đỏ**. Đây là một cổng THẬT,
đã được chứng minh ở lượt thêm `'loading'`.

🔴 **`panel.grid.nav_at_first` / `nav_at_last` KHÔNG tái dùng được** *(`vi.json:107-108`)* — hai chuỗi
đó nói *"câu đầu/cuối Chương"*. Dùng lại cho biên **Chương** là để màn hình **nói dối**.

- **(a)** Thêm hai giá trị vào `NavNotice` + hai khoá `vi.json` mới. Giữ được bất biến một-cửa.
- **(b)** Một ô nhớ **THỨ TƯ** riêng. ⚠️ Quyết định #4(b) của Story 2.10 đã cân đúng chuyện này và
  ghi ra: một ô mới làm bất biến *"ai ghi một ô thì dọn ô còn lại"* thành **N chiều**.

⚠️ **Bất biến một-cửa cài bằng CHỮ KÝ HÀM, không bằng kỷ luật** — `datThongBao`
*(`editorPanelState.ts:1516-1524`)* nhận một object ba trường tuỳ chọn và gán `?? null` cho cả ba,
nên **không tồn tại cú pháp** để ghi một ô mà không dọn hai ô kia. Khuôn tái dùng cho AC4 đã có
nguyên vẹn: `dieuHuongVaBao(doi, khiKhongDoi)` *(`:1244-1265`)*, đã chặn sẵn ca *"đang tải"* bằng
`editorHasLoaded()`.

### Quyết định #6 — id lệnh và phím mặc định (AC6)

**Khuôn "HAI id, không một id nhận hướng"** đã có chữ ký *(`commands/index.ts:1159-1163`, dẫn lại
Quyết định #3 của Story 2.5c)*: *"một id **là** thứ người dùng gán phím vào và thấy trong bảng phím
tắt"*.

🔴 **Không gian phím ĐÃ CHẬT, đo trên bộ 49 command hiện có:**

| Hợp âm | Ai đang giữ | Nguồn |
|---|---|---|
| `Mod+Alt+←` / `Mod+Alt+→` | `focus.prev_panel` / `focus.next_panel` | `commands/index.ts:640-659` |
| `Mod+Alt+↓` | `editor.next_untranslated` | `index.ts:1114-1134` |
| `⌥←` / `⌥→` **trần** | **bị `keys.ts:510` nuốt trong vùng gõ** | xem dưới |
| `⌘⇧…` | không gian của **UX-DR35** *(`epics.md:603`)* | — |

🔴 **`⌥←`/`⌥→` trần chết ở đúng ca thường nhất của FR26.** `keys.ts:509-510`:

```ts
if (entry.code !== event.code || !sameMods(entry.mods, mods)) continue
if (lacksPrimaryMod(entry.mods) && isTypingZone(event.target)) return false
```

`lacksPrimaryMod = (m) => !m.meta && !m.ctrl` *(`:415`)*; `isTypingZone` trả `true` cho
`isContentEditable` *(`:434-439`)*. Ca thường nhất của *"sang Chương sau"* là **người dùng vừa gõ
xong câu cuối** ⇒ caret **đang** trong ô bản dịch ⇒ phím không bắn. **Đây đúng lớp lỗi đã đo thật
và đã lật một chữ ký ở Story 2.10** — `⌥↓` → `⌘⌥↓`, số đo ghi tại `index.ts:1092-1112`.

🔴 **VÀ "chỗ đã đặt trước" cho `⌥←`/`⌥→` DỰA TRÊN MỘT LƯỢT ĐỌC NHẦM — đo lại 2026-08-18.**
`deferred-work.md:151` *(Story 1.14)* viết: *"không đụng `⌥←` `⌥→` trần (Chương trước/sau —
`EXPERIENCE.md:148`, Story 2.11)"*. Đo thật hôm nay:

- Dòng **148** của `EXPERIENCE.md` nay là đoạn **Auto-Lookup** — số dòng đã trôi.
- Hàng thật `| ⌥← ⌥→ | Chương trước / sau trong cùng lần nhập |` nằm ở **`EXPERIENCE.md:184`**, và
  nó thuộc bảng *"**Sửa ranh giới bóc** — bàn phím là đường chính"* *(`:174-186`)*, tức **màn xem
  trước NHẬP**, không phải Workspace. Cùng bảng đó khai `J`/`K`/`Space`/`[`/`]`/`E`/`R`/`⌥W`/`⌘↵`.
- `epics.md:599` xác nhận: đó là **UX-DR33**, và UX-DR33 nói về màn xem trước.
- Bảng **Phím của Workspace** *(`EXPERIENCE.md:261-269`)* — **không một hàng nào** cho chuyển Chương.

⇒ **`⌥←`/`⌥→` chưa bao giờ được đặt chỗ cho Workspace.** Ba đường: **(a)** không phím mặc định
*(khuôn `editor.next_segment`, chữ ký #2(c) của 2.10 — người dùng tự gán)*; **(b)** một cặp
`Mod+Alt+…` còn trống; **(c)** một họ khác.

⚠️ **Và id đặt tên ở đâu:** họ `editor.*` *(nơi mọi lệnh của lưới sống)* hay một họ `chapter.*` mới.
🔴 Đổi tên về sau là **mồ côi phím người dùng đã gán, im lặng** — bài học Quyết định #5 của Story
2.5b, và `ScopeKind::Shortcut` lưu **theo id** *(`kinds.rs:200-204`)*. Chốt **một lần**.

### Quyết định #7 🔴 — AC3 có phải một vế THỨ SÁU của AD-35 không *(ứng viên cửa chặn Task 0.4)*

**AD-35 liệt kê đúng NĂM đường** *(SPINE:419-425)*:

> *"văn bản Editor flush xuống Rust khi: (a) người dùng ngừng gõ khoảng 2 giây; (b) trần cứng 5
> giây…; (c) xác nhận segment; (d) rời segment; (e) đóng Tác phẩm hoặc thoát ứng dụng."*

**"Chuyển Chương" KHÔNG có tên trong đó.** Hai cách đọc, và chúng dẫn tới hai kết cục khác nhau:

- **Lập luận A — không AD mới.** Chuyển Chương **là** rời segment *(vế d)* theo **cấu tạo**: không
  đường nào rời Chương mà không rời câu đang gõ. AC3 khi đó chỉ là một lượt **thi hành** AD-35, và
  0 chữ nào của spine bị sửa.
- **Lập luận B — có.** Vế (d) trong **mã** được định nghĩa là *"`caretSegmentId` đổi giá trị"*
  *(`editorPanelState.ts:146-152`, nguyên văn: **"không có một widget nào để 'rời'… Caret đi từ câu
  A sang câu B LÀ một lượt rời A"**)*, tức nó đòi **có một câu B**. Một lượt chuyển Chương rời câu A
  mà **không** sang một câu B nào của Chương cũ ⇒ có thể **không** đi qua đường đó. Nếu Ice đọc AC3
  là một vế mới của bảng ⇒ **AD mới** ⇒ Task 0.4 dừng story.

**Đề xuất trình cho Ice: lập luận A.** Nhưng nó phải có **chữ ký**, không được suy ra rồi đi tiếp —
`project-context.md:461-463`.

⚠️ **Bảng tổng hợp `SPINE:764` chỉ liệt kê BỐN vế** *(thiếu "thoát ứng dụng")* — một chỗ rút gọn,
không một mâu thuẫn nội dung. Đừng đọc bảng đó thay cho AD-35.

### Quyết định #8 — dọn state khi đổi CHƯƠNG: tái dùng `resetEditorPanel()` hay dựng đường thứ hai

**Số đo — `resetEditorPanel()` *(`editorPanelState.ts:462-543`)* dọn 13 ô, và BỎ SÓT hai ô:**

| Ô bỏ sót | Khai | Chở gì của Chương cũ |
|---|---|---|
| `sourceCut` | `:1342` | `{ segmentId, offsets[] }` — một `⌘/` sau lượt chuyển cắt vào một `segment.id` **không còn trên màn hình** |
| `omitError` | `:956` *(export `:965`)* | một `IpcError` mang `params.segment_id` của Chương cũ |

`sourceCut` **đã có nợ ghi bằng chữ ngay trong hàm** *(`:522-525`)*. `omitError` thì **chưa ai nêu**
— nó cùng hạng với `confirmError` *(`:507`)* và `regroupError` *(`:526`)*, cả hai **có** được dọn.

⚠️ **Chỗ gọi duy nhất hôm nay là `libraryImport.ts:171`** *(đổi **Tác phẩm**)*, và doc-comment
`:460` viết: *"Đừng rải lời gọi này ra."*

- **(a)** Tái dùng `resetEditorPanel()` cho lượt đổi Chương, và **vá hai ô sót cùng lượt**.
- **(b)** Một `resetChapterState()` riêng — ⚠️ hai hàm cùng canh một tập ô là **hai nguồn sự thật**.
- **(c)** Dọn tối thiểu tại chỗ — đúng cái sinh ra hai ô sót ở trên.

🔴 **Luật *"mọi ô nhớ mới phải qua `resetEditorPanel()`" KHÔNG CÓ CỔNG NÀO CANH**, và đã bị bỏ sót
**hai story liên tiếp** — món nợ này đã ghi ở lượt code review Story 2.9, chủ là một story hạ tầng
cổng. Story này **không** tự dựng cổng đó; nó chỉ không được làm món nợ dày thêm.

---

### 0.9 — Việc phải làm ở Task này

1. Trình **tám** quyết định cho Ice, mỗi cái kèm số đo hoặc trích dẫn nguồn.
2. Ghi chữ ký vào §Dev Agent Record **kèm ngày**, và ghi cả **đường bị loại kèm lý do**.
3. 🔴 **Task 0.4** — nếu chữ ký nào đòi sửa một bất biến *(AD-35 ở #7, AD-1 ở #2/#3, AD-3 ở #4)*
   thì đó là một **AD MỚI**: dừng story, soạn hồ sơ bàn giao cho Winston, **không tự soạn AD**.
   Khuôn có sẵn: `planning-artifacts/ad-brief-2026-08-16-xuat-xu-ban-dich.md` *(AD-47)* và
   `ad-brief-2026-08-17-mo-hinh-hoan-tac.md` *(AD-48)*.
4. 🔴 **LUẬT DỪNG:** ba vòng chẩn đoán liên tiếp trên một giả thuyết về **sản phẩm** bị phép đo bác
   ⇒ **DỪNG, báo Ice**. *(Đếm vòng bị bác, không đếm lượt sửa thước.)*

---

## Tasks / Subtasks

### Task 0 — Cửa chặn: tám quyết định (AC: 1-6) — **CHẶN MỌI TASK KHÁC**

- [x] 0.1 Đo lại **từ nguồn** bảy tiền đề ở §Điều kiện khởi hành. **Không chép số của story này.**
      ⇒ **7/7 khớp** + 4 phép đo bổ sung + 1 chỗ story ghi lệch. Xem §Debug Log Ⓐ và Ⓑ.
- [x] 0.2 Trình tám quyết định, thu chữ ký, ghi vào §Dev Agent Record kèm ngày. ⇒ §Debug Log Ⓒ.
- [x] 0.3 Ghi **đường bị loại + lý do** cho từng quyết định. ⇒ cột phải của bảng Ⓒ.
- [x] 0.4 🔴 **Cửa chặn AD** — **KHÔNG kích hoạt.** Ice ký **#7 = Lập luận A** *(AC3 là một lượt
      **thi hành** AD-35 vế (d))* ⇒ 0 chữ của spine bị sửa. #2 = (a) và #3 = (a) đều là đường
      **hợp** AD-1 *(Rust giữ quy tắc)*; #4 = (c) **không** đụng AD-3 vì không có gì được lưu.
- [x] 0.5 Cây làm việc: xác nhận sạch trước dòng mã đầu tiên *(xem §Git)*.
      ⇒ `git status --short` = đúng **hai** tạo tác của chính story này *(tệp story `??` + entry
      `sprint-status.yaml` ` M`)*, không một thứ nào khác ⇒ **không** commit riêng, đúng §Git.

### Task 1 — Tầng Rust: chọn Chương và tìm Chương kề (AC: 1, 2, 4) — theo chữ ký #2, #3

- [x] 1.1 Hình dạng lệnh theo chữ ký #3. Khuôn **hai lớp bắt buộc**: một **hàm thuần** nhận
      `Option<&OpenWork>` *(thứ `tests/**` gọi được không cần webview)* + một vỏ `#[tauri::command]`
      **mỏng** trong `mod wire` dùng **`try_state`**, không `state()`.
      Tiền lệ đọc thẳng: `commands/chapter.rs:69-92` *(thuần)* và `:104-115` *(vỏ)*.
      ⇒ `open_adjacent_chapter(Option<&mut OpenWork>, ChapterDirection)` + vỏ `wire` với
      `try_state`. 🔵 **`&mut`, không `&`** — hàm dời con trỏ Chương, nên vỏ giữ khoá `Mutex`
      **qua** lời gọi; ghi lý do tại chỗ.
- [x] 1.2 Truy vấn Chương kề bằng **so sánh bộ đôi** `(ord, id)`. 🔴 **Cấm `ord + 1`** *(§Quyết định
      #3)*. Ca biên trả một trạng thái **phân biệt được** với lỗi — không `Err` cho *"đã ở Chương
      cuối"*, vì đó không phải một lỗi.
      ⇒ `ord > ?1 OR (ord = ?1 AND id > ?2)` + `ORDER BY ord, id` *(và bản đối xứng `DESC`)*.
      Biên trả `ChapterSwitchOutcome::AtFirst`/`AtLast` — một **enum ba giá trị**, không một
      `Option` trần: `null` không tự nói vì sao nó `null`, mà AC4 đòi *"báo rõ đã ở biên"*.
- [x] 1.3 🔴 **Đóng món nợ `deferred-work.md:650`** — đây là story được giao đích danh:
      `read_open_chapter` với **0 Chương** hôm nay ném `QueryReturnedNoRows` ⇒ `store.read_failed`
      ⇒ người dùng đọc *"không mở được kho dữ liệu"* cho một Tác phẩm lành lặn. Vá bằng
      `query_map().next()` + một `MessageKey` riêng. Tiền lệ đúng: `segment.rs:236-244`
      `chapter_not_found(chapter_id)` với `MessageKey::SegmentChapterNotFound`
      *(`core/i18n/mod.rs:174`, khai `["chapter_id"]`)*.
      ⇒ Đã vá. Ca nghiệm thu: `a_missing_chapter_row_is_a_named_error_not_a_store_error`.
- [x] 1.4 Nếu chữ ký #2 = **(a)**: thêm trường vào `OpenWork` và cập nhật **cả hai** đường đọc
      *(`chapter.rs:77` · `segment.rs:833`)* — để sót một đường là hai nguồn sự thật.
      ⇒ Cả hai đã đổi sang `open.chapter_id`. Và `create_work` nay đưa `chapter_id` **ra khỏi**
      closure ghi *(`store.write` vốn generic trên `T`)* thay vì đọc lại bằng một câu SQL thứ ba.
- [x] 1.5 `MessageKey` mới khai qua `macro_rules! message_keys!` trong `core/i18n/` — **không** danh
      sách song song. Chuỗi trong `src-tauri/src/**` viết **KHÔNG DẤU** *(Kiểm A của `check:i18n`)*.
      🔵 **KHÔNG khoá mới, và đó là kết luận chứ không phải một lượt bỏ qua:** Task 1.3 tự trỏ
      tiền lệ là `MessageKey::SegmentChapterNotFound`, và nó đã khai đúng `["chapter_id"]`
      *(`core/i18n/mod.rs:174`)*. Cùng câu, cùng nghĩa, cùng tham số ⇒ một khoá **thứ hai** là hai
      chuỗi phải giữ khớp nhau bằng kỷ luật — đúng lập luận `no_work_open` đã đi qua hai lần.
- [x] 1.6 Test hợp đồng: chèn Chương thứ hai bằng **SQL trực tiếp** *(chữ ký #1(a))*. Tên hàm test
      là một **CÂU khẳng định**, không `test_foo`.
      ⇒ **8 ca mới** trong `project_contract.rs`, cargo **401 → 409 / 0 / 5**. Hai ca canh riêng
      luật cấm `ord + 1`: ba Chương **cùng `ord = 1`** và một dãy `ord` **thưa** (1, 7, 900).

### Task 2 — Đường chuyển Chương có flush (AC: 3) — theo chữ ký #7, #8

- [x] 2.1 Tái dùng `flushEditorBeforeDiscreteWrite()` *(`editorPanelState.ts:445-450`)* — **đừng**
      dựng đường flush thứ hai. Hàm này chạy **hai lượt** có chủ ý *(`:429-443`)*: lượt đầu chụp
      `snapshot` trước IPC, ký tự gõ trong lúc lô bay nằm ngoài snapshot.
- [x] 2.2 🔴 Xử lý **cả ba** giá trị trả về `'clean' | 'failed' | 'still-dirty'`. Khuôn đã có chữ ký:
      `libraryImport.ts:145-150` **CHẶN** lượt đổi Tác phẩm khi flush trượt. Chuyển Chương mà bỏ qua
      `'failed'` là **mất bản dịch trên dữ liệu AD-5 không cho hoàn tác**.
- [x] 2.3 🔴 Thứ tự bắt buộc: **flush xong** ⇒ **rồi mới** đổi `chapterId`/`segments`. Đây là ràng
      buộc cứng của §Quyết định #2 *(lô flush mang `chapter_id` cũ ⇒ `segment.unknown_ids`)*.
- [x] 2.4 Dọn state theo chữ ký #8. Nếu #8 = (a): vá `sourceCut` **và** `omitError` **cùng lượt**,
      kèm chú thích tại chỗ nói vì sao.
- [x] 2.5 🔴 Sau lượt chuyển, tiêu điểm **phải** ở lại `panel.grid` — AD-34 §2 cấm để focus rơi về
      `body`. `FOCUS_OWNERS` *(`commands/index.ts:66-73`)* **không** cần thành viên mới.
- [x] 2.6 `requested` và `sequence` *(`editorPanelState.ts:46` · `:56`)*: một lượt nạp Chương mới
      phải **huỷ được** một lượt nạp cũ đang bay, đúng khuôn `ensureSegmentsLoaded` *(`:110-134`)*.

### Task 3 — Hai lệnh đăng ký (AC: 6) — theo chữ ký #6

- [x] 3.1 Hai `register()` theo khuôn `editor.next_segment`/`editor.prev_segment`
      *(`commands/index.ts:1165-1186`)*: `id` · `labelKey` · `run` gọi một dep, `portMissing` khi
      dep vắng.
- [x] 3.2 Handler thật **tiêm qua `CommandDeps`** *(`index.ts:162-485`)*. 🔴 Luật **erasable-only**:
      `src/commands/{index,registry,focus}.ts` phải nạp được bằng **Node thuần** — không `import`
      giá trị của `vue`/`dockview`, không `enum`, không `namespace`, không parameter property. Một
      `import` giá trị ở đó **giết ba phép kiểm cùng lúc**.
- [x] 3.3 Đăng ký ở `main.ts`, **không** trong `App.vue` *(một lượt HMR gọi `installCommands()` lần
      hai ⇒ `register()` ném vì id trùng)*.
- [x] 3.4 Phím theo chữ ký #6. Nếu có phím: kiểm **không hợp âm nào giành nhau** trên **cả hai** nền
      tảng — cổng `check:commands` Kiểm C đã có hai ca chạy thật cho việc này.
- [x] 3.5 `COMMAND_FLOOR` *(`scripts/check-commands.mjs:277`, hiện **41**)* là một **cận dưới**; cổng
      in **49** hôm nay và sẽ in 51. ⚠️ Sàn không đỏ vì thêm command — cân nhắc nâng để nó còn nói
      được điều gì, và ghi lý do tại chỗ.
- [x] 3.6 🔴 **Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU.** Ghi chẩn đoán nêu đích
      danh rồi trả `false`.

### Task 4 — Báo biên (AC: 4) — theo chữ ký #5

- [x] 4.1 Tái dùng `dieuHuongVaBao` *(`editorPanelState.ts:1244-1265`)* hoặc khuôn của nó — cửa chặn
      `editorHasLoaded()` đã nằm sẵn trong đó.
- [x] 4.2 🔴 Ghi thông báo **chỉ** qua `datThongBao` *(`:1516-1524`)*. Đừng gán thẳng một `ref`.
- [x] 4.3 Khoá `vi.json` **mới** cho biên Chương — 🔴 **không** tái dùng `panel.grid.nav_at_first`/
      `nav_at_last` *(`vi.json:107-108`, chúng nói *"câu"*)*.
- [x] 4.4 Nếu #5 = (a): thêm giá trị vào `NavNotice` **và** `StatusBar.vue::NAV_NOTICE_KEYS`
      *(`:195-203`)* **cùng lượt** — `Record` đủ khoá làm `vue-tsc` đỏ nếu quên, và đó là **cổng
      thật**, đừng để nó bắt hộ ở lượt cuối.
- [x] 4.5 🔴 **Không quay vòng.** Tiền lệ có lý do ghi bằng chữ tại chỗ — `segmentNavigation.ts:80-81`:
      *"KHÔNG quay vòng về đầu. Hết Chương thì trả `null`, và chỗ gọi để con trỏ ở nguyên. Một lượt
      quay vòng im lặng đưa người dùng về đầu Chương mà không dấu hiệu nào…"*

### Task 5 — AC5: khôi phục segment và vị trí cuộn (AC: 5) — theo chữ ký #4

🔴 **CHỮ KÝ #4 = (c) ⇒ TASK NÀY KHÔNG CHẠY.** Bốn ô dưới đây để `[⊘]`, **không** `[x]` — một
dấu `[x]` ở đó là một **lời khai sai**. Món nợ đã ghi vào `deferred-work.md` kèm **chủ: Epic 5**.

- [⊘] 5.1 *(Đây chính là nhánh đã kích hoạt.)* Ghi nợ trọn AC5 cho Epic 5 — story giao **5/6 AC**,
      đúng khuôn chữ ký ① của Story 2.9. Ba lý do đã đo: FR12 thuộc Epic 5 *(`epics.md:660`)* ·
      0 mảnh hạ tầng tồn tại · hạ tầng có **ba** chỗ tiêu thụ *(UX-DR34 đòi y hệt)*.
- [⊘] 5.2 Không chạy — **KHÔNG bước di trú 12.** 🔴 Và ba neo số học ở §Cạm bẫy ⑤
      *(`segment_contract.rs:511` · `:1562` · `pinned_contract.rs:174-184`)* **CỐ Ý không đổi**.
      Ghi ra bằng chữ vì im lặng ở đó đọc giống một lượt quên — đúng khuôn Story 2.8 đã đi qua.
      Đối chứng: `PROJECT_MIGRATIONS` sau story vẫn `[1,2,3,5,6,7,8,9,10,11]`, đích **11**.
- [⊘] 5.3 Không chạy — không cơ chế cuộn nào được dựng.
- [⊘] 5.4 Không chạy — không `scrollIntoView` nào được thêm, nên luật `behavior: 'instant'` chưa
      có chỗ áp. ⚠️ **Số đo phải đọc cho đúng:** `grep -rn "scrollIntoView" src/` cho **3** kết
      quả, và cả ba là **chú thích** *(`LookupPanel.vue:269` · `GridPanel.vue:914` · `:929`)* —
      **0** dòng mã. Ghi ra vì một lượt `grep | wc -l` ở đây đọc thành *"đã có ba chỗ cuộn"*.

### Task 6 — Nghiệm thu (AC: 1-6)

- [x] 6.1 **11 cổng npm** *(9 cổng đọc-tệp + `check:scope` + `check:scope:bundled` chạy tay, cần
      cổng 1420 trống)*. ⇒ **11/11 XANH**.
- [x] 6.2 `npm run build` · `vue-tsc` · `npm run test` *(vitest)* · `cargo test --locked`.
      ⇒ build **✓ 552ms** · vue-tsc **sạch** · vitest **242/242 (21 tệp)** · cargo **409/0/5**.
- [x] 6.3 e2e — chạy **trọn bộ**, giữ **trọn output**. ⚠️ **Đừng `tail`** output một lượt e2e: bài
      học đã ghi ở Story 2.6, một lượt `tail -45` đã cắt mất chính dòng cần đọc.
      ⇒ **8/11 spec**, log giữ nguyên vẹn *(1.973 dòng)*. Ba ca đỏ **phân xử bằng phép đo trên cả
      hai cây** — xem §Completion Notes. 🔴 **KHÔNG chấm e2e xanh.**
- [x] 6.4 🔴 Ghi **số thật**, đối chiếu với baseline ở §Điều kiện khởi hành. Một số Rust nhúc nhích
      mà story không sửa Rust *(hoặc ngược lại)* là một mệnh đề phải giải thích, không phải một sự
      tình cờ.
      ⇒ cargo **401 → 409** *(+8, đúng số ca Rust mới)* · vitest **228 → 242** *(+14, đúng số ca TS
      mới)* · command **49 → 51** *(+2, đúng hai lệnh)* · di trú **11 → 11** *(cố ý không đổi)*.
      Không một con số nào nhúc nhích ngoài bốn dòng trên.
- [x] 6.5 **Đột biến mã sản phẩm** cho mỗi ca test mới: gỡ chốt ⇒ ca đó phải **đỏ**; trả lại ⇒ xanh.
      Một ca chưa bao giờ đỏ là một ca chưa ai biết nó có chạy không.
      ⇒ **8 phép đột biến, 8 lượt đỏ-rồi-xanh** *(bảng ở §Completion Notes)*. 🔴 Và một phép trong số
      đó **bác chính một ca test của story này** — đã sửa, xem mục ① của §"Ba thứ phép đo bác".

### Task 7 — Sổ nợ và tài liệu

- [x] 7.1 Mọi vế không nghiệm thu được ⇒ `deferred-work.md` **kèm một CHỦ**. Không mục mồ côi.
      🔴 **Không XOÁ** một mục đã đóng — đóng bằng cách **nối tiếp** `→ ✅ ĐÃ ĐÓNG <ngày> (Story
      2.11)`. Đóng **một nửa** thì ghi 🟡 và liệt kê phần còn hở.
- [x] 7.2 Đóng/định chính `deferred-work.md:151` — chỗ đặt trước `⌥←`/`⌥→` dựa trên một lượt đọc
      nhầm *(§Quyết định #6)*. **Định chính**, đừng xoá.
- [x] 7.3 `deferred-work.md:650` — đóng nếu Task 1.3 chạy.
- [x] 7.4 `EXPERIENCE.md` bảng Phím Workspace *(`:261-269`)*: thêm hàng cho hai lệnh mới, kèm 🔵 và
      ngày. 🔴 **`epics.md` KHÔNG sửa** — story này không lệch spec.
- [x] 7.5 Mệnh đề nào trong mã hết đúng sau story này ⇒ **sửa tại chỗ** kèm 🔵 và ngày. Hai chỗ đã
      biết: `chapter.rs:72-74` và `segment.rs:761-775` *(cả hai trỏ tới "Story 2.11" như một tương
      lai)*.

---

## Dev Notes

### Đường dây đã có — vẽ đầy đủ, đừng dựng lại một mảnh nào

```
[phím / @click]
   └─ dispatch('editor.next_chapter')            ← Task 3, chưa có
        └─ CommandDeps.goToNextChapter?()         ← index.ts:162-485 khai dep
             └─ editorPanelState.ts               ← Task 2, chưa có
                  ├─ flushEditorBeforeDiscreteWrite()   :445-450  ✅ CÓ SẴN
                  │     └─ flushEditorNow() ×2          :343
                  │          └─ flush_segment_targets (Rust)  segment.rs:1828-1836  ✅
                  ├─ [dọn state]  resetEditorPanel()    :462-543  ✅ (thiếu 2 ô)
                  ├─ [nạp]        ensureSegmentsLoaded():110-134  ✅
                  │     └─ readOpenChapterSegments()  config/segment.ts:437
                  │          └─ read_open_chapter_segments (Rust) segment.rs:827  ⚠️ CỨNG
                  └─ [báo]        datThongBao()         :1516-1524  ✅
                        └─ StatusBar.vue :234/:258/:270  ✅
```

**Bốn thứ đã dựng sẵn, dev KHÔNG được phát minh lại:**

1. **Hợp đồng flush hai lượt** — `flushEditorBeforeDiscreteWrite()`. Nó tồn tại **vì** Story 2.6 đã
   chép thiếu một lượt khi copy sang tệp khác; đường đúng là **gọi**, không **chép**.
2. **Cửa ghi thông báo duy nhất** — `datThongBao()`, bất biến cài bằng chữ ký hàm.
3. **Khuôn điều hướng có báo** — `dieuHuongVaBao()`, đã chặn sẵn ca *"đang tải"*.
4. **Khuôn hai lớp IPC** — hàm thuần + vỏ `wire` với `try_state`.

### 🔴 Bảy cạm bẫy — mỗi cái có bằng chứng, không một khả năng lý thuyết

**① Lô flush mang `chapter_id` CŨ ⇒ mất bản dịch im lặng.**
`save_segment_targets` nhận `chapter_id` từ webview *(`segment.rs:1112-1116`)*. Đổi Chương giữa lúc
một lô bay ⇒ Rust trả `segment.unknown_ids` ⇒ chữ biến mất. `libraryImport.ts:119-132` đã ghi nguyên
lớp lỗi này cho lượt đổi **Tác phẩm**, và lời giải ở đó là **thứ tự**, không phải một `try/catch`.

**② `ord` không `UNIQUE` và không bảo đảm liên tục.**
`schema.rs:249` + doc-comment `:233-235`. `ord + 1` là *"một phép trừ im lặng trỏ sai hàng"* — chính
chữ của lượt code review 2026-08-17 trên `segment.rs`. Dùng bộ đôi `(ord, id)`.

**③ `resetEditorPanel()` bỏ sót `sourceCut` và `omitError`.**
`:1342` và `:956`. Tái dùng nó cho lượt đổi Chương **kế thừa nguyên hai lỗ**. `sourceCut` mang một
`segmentId` của Chương cũ, và `⌘/` sau lượt chuyển sẽ cắt vào một hàng không còn trên màn hình —
trên dữ liệu mà **AD-5 không cho hoàn tác**.

**④ `panel.grid.nav_at_first`/`nav_at_last` nói *"câu"*, không *"Chương"*.**
`vi.json:107-108`. Tái dùng = màn hình nói dối. Và `NAV_NOTICE_KEYS` là `Record` **đủ khoá**
*(`StatusBar.vue:195-203`)* nên thêm một giá trị `NavNotice` mà quên bảng ⇒ `vue-tsc` đỏ.

**⑤ Ba neo số học của bộ di trú, KHÔNG cổng nào canh, đã sai BA lần liên tiếp.**
Chỉ áp **nếu** chữ ký #4 sinh bước 12:
- `tests/segment_contract.rs:511` — `vec![1, 2, 3, 5, 6, 7, 8, 9, 10, 11]`
- `tests/segment_contract.rs:1562` — `static STEP_TWELVE: [Migration; 11]` + số giả `to_version: 12`.
  ⚠️ **Ba thứ phải đổi cùng lượt**: tên hằng · kích thước mảng · số giả. Chỉ **kích thước mảng** báo
  được, và bằng một lỗi **biên dịch `E0080`**, không một ca đỏ.
- `tests/pinned_contract.rs:174-184` — `PROJECT_MIGRATIONS.len()` = **10**, `schema_version` = **11**.

🔴 **Nếu #4 KHÔNG sinh bước 12 thì ba neo này CỐ Ý không đổi — và phải ghi ra bằng chữ** ở
§Completion Notes. Im lặng ở đó **đọc giống một lượt quên**. *(Khuôn này Story 2.8 đã đi qua.)*

**⑥ `⌥←`/`⌥→` trần chết trong vùng gõ.**
`keys.ts:510` + `lacksPrimaryMod` `:415` + `isTypingZone` `:434-439`. Ca thường nhất của FR26 là
caret **đang** trong ô bản dịch. Story 2.10 đã đo thật và lật một chữ ký vì đúng chuyện này.

**⑦ `insert_segments` và mọi lượt ghi phải đặt CẢ HAI: mốc so VÀ cột xuất xứ (AD-47).**
Story này **không** ghi `target_text`, nên nó **không** rơi vào danh mục đóng của AD-47. Ghi ra để
người sau khỏi đi tìm: chuyển Chương chỉ **đọc**, và một lượt flush thì chở đúng bộ đệm gõ — AD-47
loại trừ flush AD-35 **bằng chữ** *(SPINE:694)*.

### Ranh giới phạm vi — bốn thứ KHÔNG thuộc story này

1. **Một đường sinh Chương thứ hai** — FR14 → Epic 6, FR15 → Epic 5. Trừ khi chữ ký #1 = (b).
2. **Danh sách Chương nhìn thấy được** — `mockups/chapter-list.html` là màn của FR5/6/7/12/15,
   thuộc Epic 5. `mockups/key-screen-workspace.html`: `grep "Chương"` = **0** kết quả.
3. **Gộp/tách Chương (AD-32)** — 🔴 **BẪY SONG SINH**: AD-32 *(SPINE:394-398)* nói gộp/tách **Chương**
   giữ nguyên `segment.id`, **ngược** AD-5 vốn nói gộp/tách **segment** thì về hưu. Đọc nhầm một cái
   thành cái kia phá sạch lịch sử của những Chương đã dịch xong.
4. **NFR2** — một lượt đổi con trỏ trên 9.850 câu đo được **706-770 ms** *(trần 50 ms)*. Chủ vẫn là
   **Story 2.4**. Story này **đo và ghi số** nếu chạm đường nóng, **không tự chấm đạt và không tự vá**.

### Nghiệm thu — bốn đường, bốn vai, chọn đúng đường

| Mệnh đề | Đường | Vì sao không đường khác |
|---|---|---|
| Truy vấn Chương kề đúng ở biên và với `ord` thưa | **`cargo test`** hợp đồng | Đây là hợp đồng dữ liệu, không phải hành vi DOM |
| Thứ tự flush-rồi-mới-đổi | **vitest** | Hành vi module thuần; `happy-dom` đủ |
| Báo biên hiện đúng chuỗi | **vitest** | Bảng `NAV_NOTICE_KEYS` là hành vi TS |
| Hai lệnh không giành hợp âm | **cổng tĩnh** `check:commands` Kiểm C | Mệnh đề khai báo trên toàn cây |
| Tiêu điểm không rơi về `body` sau lượt chuyển | **e2e** *(WKWebView thật)* | `happy-dom` **không phải** WebKit |
| Vị trí cuộn đúng chỗ *(nếu #4 dựng)* | **e2e + bàn đo** | Hình học và bố cục thuộc engine thật |

🔴 **Chọn sai đường là dựng nguồn sự thật thứ hai.** Trước khi viết một phép kiểm mới, hỏi: **mệnh
đề này đã có chủ ở đường nào chưa.**

⚠️ **Và một cảnh báo phương pháp đã trả giá:** `happy-dom` không bố cục. Mọi mệnh đề về **hình học**
thuộc bàn đo/e2e. Story 2.5 có **74/74 vitest xanh** trên một sản phẩm mà `isConfirmed` **luôn
`false`** trong app thật — fixture chép tay có sẵn trường mà dây không gửi.

### Điều kiện khởi hành — baseline ĐO LẠI TỪ NGUỒN 2026-08-18, HEAD `5d94ba1`

| Thứ | Số | Cách đo |
|---|---|---|
| `cargo test --locked` | **401 passed / 0 failed / 5 ignored** | chạy thật |
| `vitest` | **228 / 228**, **20 tệp** | `npx vitest run` |
| Command đã đăng ký | **49** *(sàn `COMMAND_FLOOR` = **41**)* | `npm run check:commands`, Kiểm C |
| Bộ di trú `project.db` | `[1,2,3,5,6,7,8,9,10,11]`, đích **11** ⇒ bước kế tiếp **12** | `schema.rs:849-911` |
| `grep -rn "next_chapter\|prev_chapter\|nextChapter\|prevChapter" src src-tauri/src` | **0** | — |
| `grep -rn "INSERT INTO chapter" src-tauri/src` | **1** *(`project.rs:138`)* | — |
| `grep -rn "list_chapters\|read_chapters" src src-tauri/src` | **0** | — |

🔴 **Task 0.1 phải ĐO LẠI cả bảy dòng, không chép.** Bài học lặp lại nhiều lần trong epic này: một
số chép là một số sẽ lệch trong im lặng. *(Story 2.8 đã bắt được một tiền đề của chính tệp story sai
số — grep ghi 0, đo được 2.)*

### Git — trạng thái cây khi story này được soạn

`git status --short` = **rỗng**. HEAD = `5d94ba1`
*(`docs(project-context): một danh sách rỗng không tự nói vì sao nó rỗng — luật thứ 131`)*.

⇒ Hai thứ chưa theo dõi duy nhất sau lượt soạn này là **tạo tác của chính story 2.11** *(tệp story +
entry `sprint-status.yaml`)* ⇒ **không** commit riêng.
⚠️ Nếu tới lúc dev cây đã bẩn vì thứ khác: `project-context.md:425-426` — **hỏi Ice, commit riêng,
TRƯỚC dòng mã đầu tiên**.

### Project Structure Notes

**Tệp sẽ chạm** *(dự kiến; chữ ký của Task 0 có thể đổi danh sách)*:

| Tệp | Loại | Việc |
|---|---|---|
| `src-tauri/src/commands/chapter.rs` | UPDATE | lệnh chọn/chuyển Chương · vá món nợ `:650` |
| `src-tauri/src/commands/segment.rs` | UPDATE | biến thể nhận `chapter_id` *(đã giao đích danh `:773`)* |
| `src-tauri/src/commands/project.rs` | UPDATE | `OpenWork` mang Chương đang mở *(nếu #2 = a)* |
| `src-tauri/src/core/i18n/mod.rs` | UPDATE | `MessageKey` mới qua macro |
| `src-tauri/src/lib.rs` | UPDATE | `generate_handler!` |
| `src-tauri/src/core/store/schema.rs` | UPDATE | **chỉ nếu** #4 = (b) — bước 12 |
| `src/config/chapter.ts` | UPDATE | adapter mới. 🔴 **KHÔNG BAO GIỜ ném**; trả hình dạng ba trạng thái |
| `src/panels/editorPanelState.ts` | UPDATE | đường chuyển có flush · `NavNotice` · dọn state |
| `src/commands/index.ts` | UPDATE | hai `register()` + hai dep |
| `src/main.ts` | UPDATE | nối hai dep |
| `src/StatusBar.vue` | UPDATE | `NAV_NOTICE_KEYS` |
| `src/i18n/vi.json` | UPDATE | khoá `command.*` + khoá báo biên |
| `tests/frontend/**` | NEW/UPDATE | 🔴 **KHÔNG** đồng vị trí trong `src/**` |
| `src-tauri/tests/*_contract.rs` | UPDATE | hợp đồng Chương kề |
| `e2e/specs/*.e2e.mjs` | NEW | 🔴 cấm `.click()` của driver — dùng `realClick()` |

**Quy ước bắt buộc:**
- Ánh xạ thuật ngữ **cố định**: Chương → `Chapter`. 🔴 Cấm `Project`/`Book`/`Novel`/`Document` cho `Work`.
- Command id dùng **cùng văn phạm khoá chấm** với khoá i18n.
- `invoke()` gửi tham số **camelCase** *(`chapterId`)*; trường của struct **TRẢ VỀ** giữ `snake_case`
  *(`chapter_id`)*. Hai chiều khác nhau — chỗ dễ sai nhất trên dây.
- 🔴 `Ref` **không** tự bóc trong `<script>`. `if (someRef)` chạy trên **đối tượng** và **luôn đúng**.

### References

**Đặc tả**
- `_bmad-output/planning-artifacts/epics.md:2623-2656` — Story 2.11, sáu AC nguyên văn
- `epics.md:660` · `:662` · `:663` — FR12 → Epic 5 · FR14 → Epic 6 · FR15 → Epic 5
- `epics.md:599` *(UX-DR33)* · `:601` *(UX-DR34)* · `:603` *(UX-DR35)*
- `prds/prd-AuraTranslate-2026-08-02/prd.md:456` — FR26 · `:293` — FR12

**Kiến trúc**
- `architecture/…/ARCHITECTURE-SPINE.md:419-425` — **AD-35** hợp đồng flush *(năm vế)*
- `SPINE:93` — AD-3 *(dữ liệu gắn theo segment tham chiếu `id`, không vị trí)*
- `SPINE:350-354` — AD-28 *(`chapter.id` là id **cục bộ** trong `project.db`)*
- `SPINE:394-398` — AD-32 *(gộp/tách **Chương** giữ nguyên segment — bẫy song sinh của AD-5)*
- `SPINE:694` — AD-47 loại trừ flush AD-35 khỏi danh mục ghi không-phải-người-dùng

**Trải nghiệm**
- `ux-designs/…/EXPERIENCE.md:174-186` — bảng phím **màn xem trước nhập** *(nơi `⌥←`/`⌥→` thật sự ở)*
- `EXPERIENCE.md:201` — NFR17: *"…xác nhận, sang Chương kế — không chạm chuột một lần nào"*
- `EXPERIENCE.md:261-269` — bảng Phím **Workspace** *(không hàng nào cho chuyển Chương)*

**Sổ nợ**
- `deferred-work.md:151` — chỗ đặt trước `⌥←`/`⌥→` *(dựa trên lượt đọc nhầm — §Quyết định #6)*
- `deferred-work.md:650` — `read_open_chapter` với 0 Chương, **giao đích danh story này**
- `deferred-work.md:2837-2860` — *"đi đâu khi hết Chương"*, 🟡 đóng một nửa ở Story 2.10

**Story trước**
- `2-10-dieu-huong-segment.md` — khuôn `dieuHuongVaBao` · `NavNotice` · chữ ký #1(c) `⌘⌥↓` · #2(c)
  không phím mặc định · #7 cơ chế cuộn

**Luật kho**
- `_bmad-output/project-context.md` — 131 luật. Đọc **trước** dòng mã đầu tiên.

### Thông tin kỹ thuật mới nhất

⚠️ **Story này KHÔNG cần một phụ thuộc mới**, nên cửa NFR15 *(rà giấy phép, ba bước —
`project-context.md:92-100`)* **không** mở. Ghi ra thay vì im lặng: một lượt *"tiện tay thêm một
gói"* ở đây là một lượt đi vòng qua một cửa đang đứng.

Hai crate `similar` / `dissimilar` **cố ý chưa cài** *(`Cargo.toml:86-89`)* — không liên quan story
này, và cài một trong hai là **âm thầm đóng một quyết định kiến trúc đang mở**.

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Amelia / dev-story) — phiên 2026-08-18.

### Debug Log References

#### Ⓐ Task 0.1 — bảy tiền đề ĐO LẠI TỪ NGUỒN, 2026-08-18, HEAD `5d94ba1`

Toolchain lúc đo: `cargo 1.97.1` · `vitest 4.1.10` · Node 22.

| # | Tiền đề | Story ghi | **Đo lại** | Khớp |
|---|---|---|---|---|
| 1 | `cargo test --locked` | 401 / 0 / 5 | **401 passed / 0 failed / 5 ignored** | ✅ |
| 2 | `vitest` | 228/228, 20 tệp | **228/228, 20 tệp** | ✅ |
| 3 | Command đã đăng ký (sàn `COMMAND_FLOOR`) | 49 (sàn 41) | **49** · `check-commands.mjs:277` = **41** | ✅ |
| 4 | Bộ di trú `project.db` | `[1,2,3,5,6,7,8,9,10,11]` ⇒ kế tiếp **12** | mười `to_version` đọc thẳng `schema.rs`: **1·2·3·5·6·7·8·9·10·11** ⇒ **12** | ✅ |
| 5 | `grep "next_chapter\|prev_chapter\|nextChapter\|prevChapter" src src-tauri/src` | 0 | **0** | ✅ |
| 6 | `grep "INSERT INTO chapter" src-tauri/src` | 1 (`project.rs:138`) | **1**, đúng `project.rs:138` | ✅ |
| 7 | `grep "list_chapters\|read_chapters" src src-tauri/src` | 0 | **0** | ✅ |

**Bốn phép đo BỔ SUNG chạy cùng lượt** *(không có trong bảng của story — đo để trình quyết định)*:

- `OpenWork` *(`project.rs:43-53`)* mang **đúng bốn** trường `dir` · `store` · `scope` · `meta` —
  xác nhận **không** trường nào là `chapter_id`. *(Quyết định #2)*
- `resetEditorPanel()` **không** đụng `sourceCut` *(khai `:1342`)* và **không** đụng `omitError`
  *(khai `:956`)*; `confirmError` *(`:638`)* và `regroupError` *(`:1395`)* thì **có**. Xác nhận hai lỗ.
  *(Quyết định #8)*
- `grep -rn "scroll" src-tauri/src` = **0**. *(Quyết định #4)*
- `Mod+Alt+…` đang bị chiếm: `1` `2` `←` `→` `↓` `O` `J` `V` `L` `S` `X` `R` `P` `U` — **14 chỗ**.
  Còn trống có nghĩa: `↑` · `[` · `]` · `N` · `B` · `PageUp`/`PageDown`. *(Quyết định #6)*

#### Ⓑ Một chỗ story ghi hơi lệch — đo được, sửa tại chỗ để người sau khỏi tin nhầm

Task 2.2 của story dẫn `libraryImport.ts:145-150` là tiền lệ *"CHẶN khi flush trượt"*. **Vế "chặn"
đúng**, nhưng đường đó gọi **`flushEditorNow()` một lượt** *(`libraryImport.ts:145`)*, **không** gọi
`flushEditorBeforeDiscreteWrite()` hai lượt — nên nó chỉ đọc `'failed'`, không có `'still-dirty'` để
đọc. ⇒ Chuyển Chương dùng hàm **hai lượt** sẽ là nơi gọi **đầu tiên** phải xử lý `'still-dirty'` như
một phán quyết thật, không có tiền lệ để chép. Ghi ra thay vì để nó lộ ra ở lượt review.

#### Ⓒ Task 0.2 / 0.3 — TÁM CHỮ KÝ CỦA ICE, 2026-08-18

Trình kèm số đo, không kèm khuyến nghị đã tự chốt. Ghi cả **đường bị loại + lý do**.

| # | Ice ký | Đường bị loại, và vì sao |
|---|---|---|
| **#7** | **Lập luận A** — AC3 là một lượt **thi hành** AD-35 vế (d), **không** một vế thứ sáu | **B** *(vế thứ sáu ⇒ AD mới)*: loại. Chuyển Chương **là** rời segment theo cấu tạo — không đường nào rời Chương mà không rời câu đang gõ. ⇒ 0 chữ của spine bị sửa, **cửa chặn Task 0.4 KHÔNG kích hoạt** |
| **#1** | **(a)** dựng **trọn** cơ chế; AC1/AC2 nghiệm thu bằng **test hợp đồng Rust** chèn Chương thứ hai bằng SQL trực tiếp; ghi nợ có chủ | **(b)** lấn FR14/Epic 6 và đụng AD-39 *(thứ tự các bước nhập bị đóng băng)*. **(c)** Epic 2 đóng lại với FR26 chưa phủ, và thứ tự thực thi đã có chữ ký *(`sprint-change-proposal-2026-08-13b`)* |
| **#2** | **(a)** một trường mới trên **`OpenWork`** — Rust giữ, cả hai đường đọc hỏi nó | **(b)** đụng AD-1 *(webview mang một quy tắc nghiệp vụ)*. **(c)** kéo theo một bước di trú mà #4 vừa quyết là không dựng |
| **#3** | **(a)** một lệnh **`open_adjacent_chapter(direction)`** — Rust quyết Chương kề | **(b)** webview phải mang luật *"kề là gì"* ⇒ AD-1. **(c)** chú thích tại chỗ **cấm tường minh** hình dạng `Option` chưa có chỗ gọi *(`segment.rs:773-775`)* |
| **#4** | **(c)** ghi nợ **trọn** AC5 cho **Epic 5** — story giao **5/6 AC** | **(a)/(b)** loại: 0 mảnh hạ tầng tồn tại, FR12 vốn là của Epic 5, và hạ tầng này có **ba** chỗ tiêu thụ *(UX-DR34 đòi y hệt cho lượt đổi chế độ)* ⇒ chọn hình dạng ở đây là chọn cho cả ba. **(b-px)** đụng AD-3 bằng chữ ⇒ sẽ là một AD mới |
| **#5** | **(a)** mở rộng `NavNotice` + **hai khoá `vi.json` MỚI** | **(b)** một ô nhớ thứ tư làm bất biến *"ai ghi một ô thì dọn ô còn lại"* thành **N chiều** — Quyết định #4(b) của Story 2.10 đã cân đúng chuyện này |
| **#6** | **phím = (b1) `Mod+Alt+]` / `Mod+Alt+[`** · **id = `editor.next_chapter` / `editor.prev_chapter`** | **`⌥←`/`⌥→` trần**: chết ở đúng ca thường nhất của FR26 *(`keys.ts:510`)*, và **chỗ đặt trước cho nó dựa trên một lượt đọc nhầm** — hàng thật ở `EXPERIENCE.md:184` thuộc màn xem trước NHẬP *(UX-DR33)*. **(a) không phím**: loại vì story không có bề mặt bấm được nào ⇒ FR26 sẽ không với tới được. **(b2) PageUp/PageDown**: loại. **Họ `chapter.*`**: loại — mọi lệnh của lưới sống ở `editor.*` |
| **#8** | **(a)** tái dùng `resetEditorPanel()` **và vá cả hai ô sót cùng lượt** | **(b)** hai hàm cùng canh một tập ô là **hai nguồn sự thật**. **(c)** đúng cái đã sinh ra hai ô sót |

🔵 **Xác nhận kỹ thuật cho #6 trước khi chốt:** `BracketLeft`/`BracketRight` **đã có sẵn** trong
`NAMED_CODES` *(`keys.ts:121-122`)* và có ký hiệu hiển thị `[`/`]` *(`:299-300`)* ⇒ hợp âm viết
`Mod+Alt+BracketRight`, không cần một hàng mới nào trong bảng mã phím.

### Completion Notes List

#### Đã giao — 5/6 AC, và cái thứ sáu ghi nợ CÓ CHỦ chứ không tự chấm

| AC | Trạng thái | Nghiệm thu bằng |
|---|---|---|
| **AC1** Chương sau mở ra | 🟡 **cơ chế xong, đường sản phẩm chưa tới** | `cargo` hợp đồng *(4 ca)*, `vitest` *(3 ca)*. **Không** đường e2e — xem nợ ①. |
| **AC2** Chương trước mở ra | 🟡 như AC1 | `cargo` *(2 ca)*, `vitest` *(1 ca)* |
| **AC3** flush trước khi chuyển | ✅ | `vitest` *(3 ca, gồm ca **thứ tự** và ca **chặn khi flush trượt**)* |
| **AC4** báo biên, không quay vòng | ✅ | `cargo` *(3 ca)*, `vitest` *(4 ca)* |
| **AC5** khôi phục segment + vị trí cuộn | ⊘ **ghi nợ trọn cho Epic 5** | — *(chữ ký #4(c), Ice ký)* |
| **AC6** command đăng ký, gán phím được | ✅ | cổng tĩnh `check:commands` Kiểm C *(51 command, không hợp âm nào giành nhau trên **cả hai** nền tảng)* |

#### Ba thứ PHÉP ĐO bác, và cả ba đã sửa trong chính lượt này

1. 🔴 **Ca test *"flush TRƯỚC lượt chuyển"* của chính tôi KHÔNG canh gì — phát hiện thật của
   Task 6.5.** Bản đầu đẩy **một** mốc `save:` lúc vào rồi khẳng định `iLuu < iChuyen`. Phép
   đột biến *(bỏ `await` trước lượt flush — tức dựng lại đúng cuộc đua mất dữ liệu)* để ca đó
   **vẫn xanh**, vì lời gọi flush vẫn **bắt đầu** trước. Mệnh đề thật là *"flush **ĐÃ XONG**
   trước"* — AD-35: *"một flush chỉ được coi là xong sau khi đã ghi vào WAL"*. ⇒ Thêm mốc
   `save-done:`; đột biến chạy lại thì ca **đỏ**. *Nếu Task 6.5 không chạy, story này giao một
   ca xanh vĩnh viễn trên đúng mệnh đề trung tâm của nó.*
2. 🔵 **Tiền lệ mà story dẫn cho Task 2.2 chỉ phủ MỘT NỬA.** Story viết `libraryImport.ts:145-150`
   là khuôn *"chặn khi flush trượt"*. Vế *"chặn"* đúng, nhưng đường đó gọi `flushEditorNow()`
   **một** lượt nên nó chỉ có `'failed'` để đọc. ⇒ `switchChapter` là nơi gọi **đầu tiên** của
   kho phải phán quyết `'still-dirty'`, không có tiền lệ để chép. Ghi nợ *(nhánh đúng theo cấu
   tạo nhưng chưa ca nào đi qua)*.
3. 🔵 **Số đo `grep "scrollIntoView" src/` = 3, không 0** — nhưng cả **ba** là **chú thích**,
   0 dòng mã. Kết luận của story không đổi; cách đọc số thì phải đổi.

#### Đột biến mã sản phẩm — TÁM phép, tám lượt đỏ-rồi-xanh (Task 6.5)

| # | Đột biến | Ca đỏ |
|---|---|---|
| TS-1 | bỏ `await` trước `flushEditorBeforeDiscreteWrite()` | 2 *(thứ tự · chặn-khi-trượt)* |
| TS-2 | biên báo bằng `'at-last'`/`'at-first'` *(câu của **câu**)* | 2 |
| TS-3 | gỡ chốt `dangChuyenChuong` | 1 |
| TS-4 | gỡ hai dòng vá `sourceCut`/`omitError` | 2 |
| TS-5 | bỏ `ensureSegmentsLoaded()` sau lượt dọn | 2 |
| R-1 | `ord = ?1 + 1` thay so sánh bộ đôi | 2 *(ord trùng · ord thưa)* |
| R-2 | biên trả `Moved` *(quay vòng)* | 3 |
| R-3 | trả `query_row` về chỗ cũ *(mất guard `chapter_not_found`)* | 1 |

#### 🔴 E2E — 8/11 spec, và tôi KHÔNG chấm nó xanh

**Số đo trọn bộ (11 spec, 18m51s):** **8 passed / 3 failed**. Ba ca đỏ đã phân xử **bằng phép đo**,
không bằng suy luận — mỗi ca chạy trên **cả hai** cây *(cây story và baseline `5d94ba1`)*:

| Spec | Trong lô | Chạy MỘT MÌNH, cây story | Chạy MỘT MÌNH, baseline | Phân xử |
|---|---|---|---|---|
| `editor-typing-flush` | ✖ *(`activeElement` không mang `data-segment-id`)* | ✓ ở lượt chạy lại | — | **chập chờn** |
| `attribution-focus` | ✖ *(hai kiểu hỏng KHÁC nhau ở hai lượt)* | **4/4 ✓** | **4/4 ✓** | **chập chờn trong lô** — hai cây không phân biệt được |
| `segment-navigation` | ✖ trên **CẢ HAI** cây *(before-hook hết 60 s chờ 40 hàng)* | **9/10 ✓** | **5/5 ✓** | xem dưới |

🔴 **Vế phải nói thẳng, không làm tròn:** `segment-navigation` đỏ **1 lần trong 10** trên cây story
*(`hangCoVach()` trả 31 thay vì 0 ngay sau một `realClick` — một phép khẳng định về hàng nào mang
vạch `primary`)* và **0 lần trong 5** trên baseline. **1/10 so với 0/5 KHÔNG phân biệt được hai
cây** — nó không chứng minh có hồi quy, và nó cũng **không chứng minh không có**. Ghi ra đúng như
vậy, kèm cả hai mẫu.

⚠️ **Tôi KHÔNG chạy lại tới khi xanh rồi báo xanh.** Bộ e2e **không** cho một lượt xanh trọn bộ ở
story này, và đó là một **món nợ hạ tầng đã ghi từ Story 2.5b** *("bộ e2e từng 7/7 ĐỎ vì một khuyết
tật của BÀN ĐO — `wdio.conf.mjs::devServerIsUp()` tin một Vite hấp hối")*. Lượt này thêm một dữ kiện
cho món đó: **chế độ LÔ là nơi cả ba ca đỏ tập trung**, và `segment-navigation` đỏ trong lô **trên
cả baseline**, tức khuyết tật bàn đo **có trước** story này.

🔴 **Và một giới hạn của chính lượt phân xử:** e2e **không** chạm được một dòng nào của story này —
không đường sản phẩm nào sinh Chương thứ hai, nên **không spec nào gọi `open_adjacent_chapter`**.
⇒ Ba ca đỏ ở trên nói về **hồi quy**, không về **tính năng mới**. Vế *"người dùng bấm phím và Chương
đổi"* vẫn không có đường nghiệm thu nào — món nợ ① ở `deferred-work.md`.

#### 🔴 Ba neo số học của bộ di trú — CỐ Ý KHÔNG ĐỔI, ghi ra vì im lặng đọc giống một lượt quên

Chữ ký #4 = (c) ⇒ **không** bước di trú 12. ⇒ `segment_contract.rs:511`
*(`vec![1,2,3,5,6,7,8,9,10,11]`)* · `:1562` *(`STEP_TWELVE: [Migration; 11]`)* ·
`pinned_contract.rs:174-184` **giữ nguyên từng ký tự**. Đối chứng đã chạy: `PROJECT_MIGRATIONS`
sau story vẫn mười bước, đích **11**, bước kế tiếp **12**. *(Khuôn này Story 2.8 đã đi qua.)*

#### 🔴 Cái story này KHÔNG làm, nói thẳng

- **Không** tự chấm AC1/AC2 là đã đạt ở tầng sản phẩm. Cơ chế xanh 100% mà **chưa ai bấm được** —
  và đó là hai câu khác nhau.
- **Không** đo NFR2, **không** vá NFR2. Chủ vẫn là Story 2.4.
- **Không** sửa `epics.md` một chữ. AC5 mô tả một đích đến; đường đi chưa tới không làm nó sai
  *(`project-context.md:456-458`)*.
- **Không** thêm một phụ thuộc nào ⇒ cửa NFR15 không mở.
- **Không** tự soạn một `AD`. Cửa chặn #7 đã cân và Ice ký **lập luận A** — 0 chữ của spine bị sửa.

### File List

**Rust**
- `src-tauri/src/commands/chapter.rs` — UPDATE · `chapter_not_found()` · `read_open_chapter` đọc
  `OpenWork::chapter_id` · `ChapterDirection`/`ChapterSwitchOutcome`/`ChapterSwitch` ·
  `open_adjacent_chapter()` + vỏ `wire`
- `src-tauri/src/commands/project.rs` — UPDATE · trường `OpenWork::chapter_id`; `create_work` đưa
  `chapter_id` ra khỏi closure ghi
- `src-tauri/src/commands/segment.rs` — UPDATE · `read_open_chapter_segments` đọc
  `OpenWork::chapter_id` *(bỏ câu SQL suy-ra-động thứ hai)*; sửa doc-comment `:773-775` đã hết đúng
- `src-tauri/src/lib.rs` — UPDATE · `generate_handler!` thêm `open_adjacent_chapter`
- `src-tauri/tests/project_contract.rs` — UPDATE · **8 ca mới** + helper `insert_chapter_directly`

**Webview**
- `src/config/chapter.ts` — UPDATE · adapter `openAdjacentChapter()` + ba kiểu dây, hình dạng
  **ba trạng thái**, không ném
- `src/panels/editorPanelState.ts` — UPDATE · `switchChapter()` · `goToNextChapter`/`goToPrevChapter`
  · chốt `dangChuyenChuong` · hai giá trị `NavNotice` mới · vá `sourceCut`+`omitError` trong
  `resetEditorPanel()` · `enterFocus('panel.grid')` sau `nextTick()`
- `src/commands/index.ts` — UPDATE · hai dep + hai `register()` với `Mod+Alt+BracketRight/Left`
- `src/main.ts` — UPDATE · nối hai dep
- `src/StatusBar.vue` — UPDATE · hai hàng `NAV_NOTICE_KEYS`
- `src/i18n/vi.json` — UPDATE · 2 khoá `command.editor.*_chapter` + 2 khoá `panel.grid.nav_at_*_chapter`

**Cổng và test**
- `scripts/check-commands.mjs` — UPDATE · `COMMAND_FLOOR` 41 → 43 *(51 thật, 84,3 %)*
- `tests/frontend/editorChapterSwitch.test.ts` — **NEW** · 14 ca

**Tài liệu**
- `_bmad-output/implementation-artifacts/deferred-work.md` — UPDATE · đóng `:650`, định chính
  `:151`, thêm 6 mục mới có chủ
- `_bmad-output/planning-artifacts/ux-designs/…/EXPERIENCE.md` — UPDATE · hàng `⌘⌥]`/`⌘⌥[` vào
  bảng Phím Workspace
- `_bmad-output/implementation-artifacts/2-11-chuyen-chuong-trong-workspace.md` — story
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — trạng thái

### Change Log

- **2026-08-18** — Task 0 xong: bảy tiền đề **đo lại từ nguồn, 7/7 khớp**; tám quyết định có chữ ký
  của Ice; **cửa chặn AD KHÔNG kích hoạt** *(#7 = lập luận A)*.
- **2026-08-18** — Task 1: tầng Rust. `OpenWork::chapter_id` · `open_adjacent_chapter` với so sánh
  bộ đôi `(ord, id)` · đóng món nợ `deferred-work.md:650`. cargo **401 → 409 / 0 / 5**.
- **2026-08-18** — Task 2–4: đường chuyển có flush · hai lệnh `Mod+Alt+]`/`Mod+Alt+[` · hai câu báo
  biên. vitest **228 → 242**, command **49 → 51**, `COMMAND_FLOOR` 41 → 43.
- **2026-08-18** — Task 5 **KHÔNG chạy** *(chữ ký #4(c))*: AC5 ghi nợ trọn cho Epic 5, bốn ô `[⊘]`.
- **2026-08-18** — Task 6.5: **tám phép đột biến**, tám lượt đỏ-rồi-xanh — và một trong số đó **bác
  chính một ca test của story này**, đã sửa *(mốc `save-done`)*.
- **2026-08-18** — Task 7: đóng `deferred-work.md:650` · **định chính** `:151` *(lượt đọc nhầm
  `EXPERIENCE.md:148`)* · 6 mục nợ mới có chủ · hàng phím mới trong `EXPERIENCE.md` · sửa hai
  doc-comment trong mã đã hết đúng. `epics.md` **không sửa một chữ**.

### Review Findings

---

## Câu hỏi cho Ice — chốt ở Task 0, trước dòng mã đầu tiên

1. **#1** — Tiền đề của AC1/AC2 không tồn tại *(một Tác phẩm = một Chương)*. Dựng trọn cơ chế và
   nghiệm thu bằng test hợp đồng SQL **(a)**, dựng thêm đường sinh Chương **(b)**, hay dời story
   xuống sau Epic 6 **(c)**?
2. **#2** — *"Chương đang mở"* sống ở `OpenWork` **(a)**, ở webview **(b)**, hay trên đĩa **(c)**?
   *(Vế (b) đụng AD-1.)*
3. **#3** — Rust quyết Chương kề **(a)**, webview quyết từ một danh sách **(b)**, hay thêm
   `Option<i64>` vào hai lệnh đọc **(c)** *(hình dạng mà chú thích tại chỗ đang cấm)*?
4. **#4** — AC5 dựng trong phiên **(a)**, dựng xuống đĩa + bước di trú 12 **(b)**, hay ghi nợ trọn
   cho Epic 5 **(c)**? Và nếu dựng: lưu bằng `segment.id` hay bằng pixel?
5. **#5** — Biên Chương dùng `NavNotice` mở rộng **(a)** hay một ô nhớ thứ tư **(b)**?
6. **#6** — Không phím mặc định **(a)**, một cặp `Mod+Alt+…` còn trống **(b)**, hay một họ khác
   **(c)**? Và id là `editor.next_chapter` hay `chapter.next`? *(Đổi tên về sau là mồ côi phím người
   dùng đã gán.)*
7. **#7** 🔴 — AC3 là một lượt **thi hành** AD-35 vế (d), hay một vế **thứ sáu** *(⇒ AD mới ⇒ dừng
   story, bàn giao Winston)*?
8. **#8** — Đổi Chương tái dùng `resetEditorPanel()` + vá hai ô sót **(a)**, một hàm riêng **(b)**,
   hay dọn tối thiểu **(c)**?

🔴 **Và một câu không nằm trong tám quyết định, vì nó là hệ quả của #6 giao với việc story này không
có bề mặt nhìn thấy được:** nếu **#6 = (a)** *(không phím mặc định)* thì FR26 chỉ với tới được **sau
khi người dùng tự gán phím** ở màn hình phím tắt. FR26 tồn tại để *"mạch làm việc không bị cắt"* —
Ice có chấp nhận hình dạng đó, hay muốn một phím mặc định / một bề mặt bấm được?
