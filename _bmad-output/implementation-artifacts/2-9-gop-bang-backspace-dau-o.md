---
baseline_commit: 4d72cd44c95c4dac132833d96097c01bccc061fa
---

# Story 2.9: Gộp bằng `Backspace` ở đầu ô

Status: done

**Covers:** FR78 · UX-DR32 · AD-5
**Epic:** 2 — Biên tập theo segment
**Story trước:** 2.8 — Gộp và tách segment tường minh (`done`, 2026-08-17)

---

## Story

As a **người dịch**,
I want **nối câu này với câu trên bằng chính phím xoá lui**,
so that **tôi không phải dừng lại ra lệnh cho công cụ giữa dòng suy nghĩ**.

---

## 🔴 ĐỌC TRƯỚC DÒNG MÃ ĐẦU TIÊN — story này KHÔNG dựng nghiệp vụ gộp

Story 2.8 đã dựng **trọn vẹn** đường gộp: hàm thuần Rust, lệnh IPC, adapter TS, khuôn
`regroup()` ba nhịp, vá ảnh chụp, dời caret, và mọi lỗi từ chối kèm khoá i18n.
`mergeCurrentSegment()` (`src/panels/editorPanelState.ts:1229`) **đã là đúng hàm story này
cần gọi** — nó gộp câu đang có caret với câu liền trên, đúng từng chữ AC1.

⇒ **Phạm vi thật của 2.9 là ba thứ, và cả ba đều là chỗ story 2.8 cố ý dừng lại:**

| # | Việc | Trạng thái nền |
|---|---|---|
| ① | **Cử chỉ `Backspace` ở đầu ô** kích hoạt lượt gộp | Chưa có. Điểm cắm đã chờ sẵn ở `onEditKeydown` |
| ② | **Dòng báo hệ quả** sau lượt gộp | Khe thông điệp `StatusBar` đang ĐÓNG, phải mở. Chủ: **story này** |
| ③ | **Hoàn tác `⌘Z`** | Không FR, không AD, không hạ tầng. Chủ: **Ice** — xem Task 0 |

🔴 **Viết một đường gộp thứ hai là lỗi thiết kế, không phải một lựa chọn.** Nhánh
`Backspace` phải gọi **chính** `mergeCurrentSegment()` mà `⌘M` gọi. Hai đường gộp lệch nhau
biên dịch sạch, đi qua **cả mười một cổng**, và chỉ lộ ra khi một trong hai quên flush
AD-35 — tức mất chữ người dùng vừa gõ, im lặng.

---

## Acceptance Criteria

Sáu AC, nguyên văn ngữ nghĩa từ `epics.md:2543-2568`.

**AC1 — Cử chỉ**
**Given** con trỏ ở **đầu ô bản dịch** của một câu
**When** người dùng bấm `Backspace`
**Then** hệ thống **gộp câu đó với câu trên**

**AC2 — Kết quả xác định ở cả hai vế**
**Given** thao tác gộp
**When** chạy
**Then** kết quả xác định ở **cả hai vế** — nối `source_text` của hai câu, nối `target_text`
của hai câu
🔴 Vì thế **gộp chạy được từ cả hai phía**, khác với **tách** *(Story 2.8)* vốn bắt buộc làm
ở **cột nguyên văn**: không có phép chiếu nào từ vị trí con trỏ bên tiếng Việt sang chỗ cắt
bên tiếng Trung.

**AC3 — Đúng ngữ nghĩa AD-5**
**Given** gộp xảy ra
**When** thực hiện
**Then** đúng ngữ nghĩa của gộp tường minh — hai câu cũ **về hưu** và **vẫn tra lại được
lịch sử**, câu mới **chưa xác nhận với lịch sử rỗng**

**AC4 — Dòng báo hệ quả**
**Given** gộp xảy ra
**When** thực hiện
**Then** một dòng báo hiện ở lề nêu **hệ quả**, ví dụ *"Đã gộp hai câu. Câu mới chưa xác
nhận — lịch sử của hai câu cũ vẫn tra lại được."*

**AC5 — Hoàn tác** *(xem Task 0 — AC này có một cửa chặn kiến trúc)*
**Given** gộp vừa xảy ra
**When** người dùng bấm `⌘Z`
**Then** hoàn tác được

> 🔵 **AC5 ĐÃ RÚT 2026-08-18** *(Ice ký; SCP 2026-08-18b)* — giữ nguyên văn ở trên vì lịch sử của
> một AC là bằng chứng cho quyết định kế tiếp.
>
> **Cửa chặn `AD-48` phân giải bằng cách RÚT AC, không bằng một mô hình.** Cả hai đường của hồ sơ
> bàn giao bị loại: **(A)** đòi một ngoại lệ có tên của AD-3 cộng đường `DELETE` **đầu tiên** trên
> nội dung người dùng; **(B)** cho ra *"một segment giữ toàn bộ bản dịch, một segment rỗng"*
> *(đo ở `regroup.rs:280-281`)*. Ice nêu một đường **thứ ba** mà hồ sơ chưa có: **không dựng `⌘Z`
> cho gộp/tách** — gộp và tách đã là lệnh người dùng, muốn quay lại thì gọi lại chúng.
>
> **Bốn phép đo chống lưng:** ① `grep "undo|hoàn tác|⌘Z" prd.md` chỉ trúng `undock` ⇒ **không FR nào
> đòi hoàn tác**; ② `EXPERIENCE.md:169` *(Ice ký 2026-08-17)* đã viết *"trên dữ liệu mà AD-5 không
> cho hoàn tác"*; ③ dòng báo đang chạy *(`vi.json:101`)* nói đúng hệ quả và **không hứa** hoàn tác;
> ④ chỗ duy nhất hứa `⌘Z` là `EXPERIENCE.md:171`, và nó là **mẩu sót** của lượt sửa 2026-08-17 ở
> chính dòng ấy — khuôn **F1** của retro Epic 2.
>
> **Tiền đề *"tự gộp/tách lại được"* đã kiểm và ĐÚNG:** lượt gộp **nối** `target_text`
> *(`regroup.rs:186`)*, lịch sử hai câu cũ **vẫn đọc được** *(`lib.rs:346`)* ⇒ **không byte nào bị
> phá**; cái mất là **công sức thao tác**.
>
> `epics.md` và `EXPERIENCE.md:171` đã sửa cùng lượt. `Status: done` của story **không đổi** — AC5
> nay là *đã rút*, không phải *chưa đạt*. Chỗ hở còn lại *(bấm `⌘Z` không phản hồi)* và `AD-48`
> *(bản nhỏ, chủ Winston)* ghi ở `deferred-work.md`, khối **SCP 2026-08-18b**.

**AC6 — Không chặn**
**Given** người dùng bấm `Backspace` ở đầu ô
**When** xảy ra
**Then** hệ thống **không chặn và không hỏi lại**

**AC7 — Cử chỉ đánh dấu chỗ cắt không được cướp cú bấm trơn** *(🔵 THÊM 2026-08-17, Ice ký)*
**Given** con trỏ chuột ở **cột nguyên văn**
**When** người dùng bấm đơn hoặc double-click **không giữ phím bổ trợ**
**Then** hệ thống **không đánh dấu chỗ cắt nào** — cú bấm để trống cho **tra cứu** (FR21)
**And** khi người dùng giữ **`Mod`** rồi bấm, hệ thống **đánh dấu chỗ cắt** tại đó

🔴 **AC này lật một cử chỉ của Story 2.8 (đã `done`), và nó đến từ một lượt DÙNG THẬT** — đúng
khuôn đã lặp ở 2.5b *(hàng về hưu trong lưới)* và 2.8 *(chữ ký #6 lật lần thứ ba)*: mọi lý lẽ
trên giấy đều đứng, và tất cả cộng lại vẫn thua một lượt người thật nhìn vào một Chương thật.

**Xung đột đo được:** cột nguyên văn mang **hai** cử chỉ chuột treo trên **cùng một** `mouseup`
— đánh dấu chỗ cắt *(2.8)* và **Auto-Lookup** *(FR21, Story 1.18, **đã phát hành**,
`useSelectionSurface(colSrc, 'source', …)`)*. ⇒ Mỗi lượt tra một từ để **đọc** cũng rơi một dấu
cắt; và một cú **double-click bắn HAI `mouseup`** ⇒ hai lượt `setEditorSourceCut`, hàm đó
**toggle**, nên nó để lại **hai** dấu.

✅ **Và cùng lượt ký đó đóng một món nợ 🔴 lớn hơn story này:** `deferred-work.md:4100` ghi
*"Auto-Lookup bằng chuột ở cột nguồn CHƯA CÓ đường nghiệm thu, **và có thể đang chết**"* — vì
qua driver, **không cử chỉ chuột nào** tạo được vùng chọn ở cột đó. Món đó khai bằng chữ rằng
*"một lượt bôi đen bằng tay của Ice đóng được vế 'sản phẩm có hỏng không'"*, và ứng viên còn
lại *"driver không lái được máy chọn văn bản của WebKit"* **không loại trừ được bằng chính
driver đó, theo cấu tạo**. Ice xác nhận 2026-08-17: **double-click TRA ĐƯỢC** trên máy thật.
⇒ FR21 **sống**; vế kia là giới hạn của **BỘ ĐO**, không của sản phẩm.

---

**AC8 — `Esc` xoá tập điểm cắt** *(🔵 THÊM 2026-08-17, Ice yêu cầu)*
**Given** cột nguyên văn đang có một hoặc nhiều điểm cắt chờ
**When** người dùng bấm `Esc`
**Then** **cả tập** bị xoá, không chỉ điểm cuối

**AC9 — Chỗ cắt phải đúng ở tab HÁN VIỆT** *(🔵 THÊM 2026-08-17, Ice báo)*
**Given** người dùng đang xem tab Hán Việt, ở kiểu `switch` hoặc `parallel`
**When** `Mod`+click vào cột nguyên văn
**Then** một điểm cắt **đúng chỗ** được ghi nhận — theo **ranh giới từ** ở `switch` *(chữ Hán
gốc không có trên màn hình)*, **chính xác từng chữ** ở `parallel` *(base `<ruby>` có mặt)*
**And** dấu cắt **hiện ra** ở cả hai kiểu xem

🔴 **Đây là một lỗ HỎNG DỮ LIỆU IM LẶNG, không một khuyết tật hiển thị.** Đo trên WKWebView
thật (`2-9-ban-do/han-viet-cho-cat.e2e.mjs`), nguyên văn `京都春風。` — **5 ký tự**:

| Tab | `sourceCutOffsetOf` trả | Neo vào | Đúng phải là |
|---|---|---|---|
| Nguyên văn *(đối chứng)* | 5 | text node của ô | — |
| Hán Việt **switch** | **17** | `.hv-syl` = `"kinh"` — **ÂM**, không phải chữ Hán | ≤ 4 |
| Hán Việt **parallel** | **19** | base `<ruby>` = `"京都"` | **2** |

Nguyên nhân lớn nhất **không nằm trong ba giả thuyết ban đầu**: dòng `Nguồn: thieu-chuu`
(`.hv-sources`, **17 ký tự**) nằm **trong ô** và bị phép đếm mù cộng vào.

🔴 **Và hôm nay nó chưa hỏng im lặng chỉ vì MAY:** `17`/`19` tình cờ **vượt biên** một câu 5
chữ nên Rust từ chối — đó là thứ Ice thấy dưới dạng *"chưa cắt được"*. Trên một câu Chương
thật *(40–60 chữ)*, `19` nằm **trong biên** và `⌘/` cắt **sai chỗ, im lặng**, trên dữ liệu mà
**AD-5 không cho hoàn tác**.

## 🔴 Task 0 — CỬA CHẶN: AC5 (`⌘Z`) chưa có mô hình, và chọn một mô hình là một `AD` MỚI

**Đây là task đầu tiên, và nó có thể dừng story.** Không viết dòng mã nào của AC5 trước
khi cửa này được phán định.

### 0.1 — Sự thật đã đo, không phải một nghi ngờ

- `grep -rn "undo|redo|UndoManager" src/ src-tauri/src/ -i` ⇒ **0** cơ chế. Ba lượt trúng
  đều là chữ *"undock"* của dockview cộng một câu chú thích ở `GridPanel.vue:1064`.
- **0** command `Mod+Z` trong `src/commands/index.ts` và `src/commands/keys.ts`.
- `prd.md`: grep *"undo / hoàn tác / ⌘Z"* ⇒ trúng **duy nhất** chữ *"undock"* ở FR17.
  **Không FR nào định nghĩa mô hình hoàn tác.**
- `EXPERIENCE.md` bảng phím `:261-268`: **không có hàng `⌘Z`**.
- `ARCHITECTURE-SPINE.md`: **không `AD` nào** về undo. AD cao nhất hiện tại là **AD-47**.

### 0.2 — Vì sao nó không giải được bằng một lượt "tiện tay"

`segment_version` **không phải** `⌘Z`. AD-31 khai thẳng, và Story 2.8 đã cài đúng thế:
*"Về hưu do gộp/tách (AD-5) ⇒ **không** tạo `segment_version`"*
(`src-tauri/src/commands/segment.rs:2173-2183`, khoá bằng test
`neither_merge_nor_split_ever_writes_a_segment_version_row`, `segment_contract.rs:6168`).
⇒ Một lượt gộp **không để lại bản sao nào** mà cơ chế lịch sử hiện có hoàn tác được.

Câu hỏi chưa ai trả lời, và **hai câu trả lời cho hai cái đĩa khác nhau**:

| Đường | Hoàn tác một lượt gộp là… | Hệ quả |
|---|---|---|
| **(A)** | gỡ `retired_at` của hai hàng cũ **và xoá** hàng mới | `segment.id` cũ **quay lại** — mọi dữ liệu gắn theo id còn nguyên |
| **(B)** | một lượt **tách mới** | về hưu + tạo mới lần nữa; **`id` cũ không bao giờ quay lại** (AD-3), và chỗ đánh dấu FR119 trỏ vào một câu thứ ba |

🔴 Cả hai đều biên dịch sạch. Cả hai đi qua mọi cổng. Chọn sai là **hỏng vĩnh viễn dữ liệu
người dùng** theo đúng nghĩa mục *"chỗ hỏng là VĨNH VIỄN"* của `project-context.md`.

### 0.3 — Món nợ đã ghi, và nó ghi chủ là AI

`deferred-work.md:4103-4109`, nguyên văn:

> 🟡 **Khe thông điệp của `StatusBar` vẫn đóng, và `⌘Z` vẫn chưa có mô hình.** Chữ ký #9(a)
> (Ice, 2026-08-17) giữ 2.8 đúng phạm vi tám AC: **không** dòng báo hệ quả, **không** hoàn tác.
> **Chủ: Story 2.9** cho dòng báo; **Ice** cho `⌘Z` *(chưa FR/AD/UX-DR nào chốt mô hình undo,
> và chọn một mô hình là một `AD` MỚI)*.

⇒ **Dòng báo (AC4) là việc của story này. Mô hình `⌘Z` (AC5) thì KHÔNG.**

### 0.4 — Việc phải làm ở task này

1. **Dừng, và soạn hồ sơ bàn giao** cho một `AD-48` — không tự soạn `AD`.
   Hồ sơ nêu: hai đường (A)/(B) ở trên, hệ quả trên đĩa của từng đường, ràng buộc AD-3
   *(`segment.id` không bao giờ tái dùng)*, ràng buộc FR119 *(chỗ đánh dấu trỏ segment về
   hưu ở lại)*, và phạm vi *(chỉ lượt gộp/tách, hay mọi thao tác Editor)*.
2. **Báo Ice**, kèm khuyến nghị phạm vi: 5/6 AC giao được ngay và **có giá trị đứng một
   mình** — cử chỉ + ngữ nghĩa + dòng báo. AC5 chờ `AD-48`.
3. **Không** hạ AC5 xuống *"native undo của contenteditable"*. Đo được: mỗi ô là một editing
   host **riêng**, và một lượt gộp xoá hai ô rồi dựng một ô mới — native undo chỉ hoàn tác
   ký tự **trong một ô**, nó **không** đụng lượt ghi đã xuống WAL. Nhận nó là AC5 đạt sẽ là
   một *"số đo không có thật"*.

⚠️ **Năng lực chưa dựng ≠ lệch spec.** AC5 **không sai** vì đường đi chưa tới nó. **Đừng
sửa `epics.md`** cho khớp mã. Ghi một món nợ **có chủ** vào `deferred-work.md`.

---

## Tasks / Subtasks

### Task 0 — Cửa chặn `AD-48` cho AC5 (AC: 5)
- [x] 0.1 Xác nhận lại ba phép grep ở §0.1 trên cây nguồn **hiện tại** (đừng tin bảng này — nó là ảnh chụp 2026-08-17)
- [x] 0.2 Soạn hồ sơ bàn giao `AD-48` (hai đường, hệ quả đĩa, ràng buộc AD-3/FR119, phạm vi)
- [x] 0.3 Báo Ice, chờ phán định. **Không viết mã AC5 trước phán định.**

### Task 1 — Bàn đo: `Backspace` ở đầu ô trên WKWebView thật (AC: 1)
- [x] 1.1 Dựng bàn đo trong `_bmad-output/implementation-artifacts/2-9-ban-do/` theo khuôn `2-8-ban-do/`
- [x] 1.2 Đo `keydown` khi caret ở offset 0 của ô **có chữ** và ô **rỗng**: `key`, `code`, `isComposing`, `repeat`, và `preventDefault()` có chặn được lượt xoá không — 🟡 **ba vế đầu ĐO ĐƯỢC; vế `preventDefault` KHÔNG đo được qua driver** *(sự kiện `isTrusted: false` không có default action)*, ghi món cho Ice
- [x] 1.3 Đo hình dạng `Selection` ở đầu ô sau khi ô đã có `\n` (Story 2.5d): `rangeCount`, `collapsed`, `startContainer`, `startOffset`, `cell.childNodes.length` — 🔴 **số quyết định của cả story**
- [x] 1.4 Đo ca **giữ phím**: xoá lui liên tục tới offset 0 rồi tiếp tục giữ — `event.repeat` bằng gì, và bao nhiêu `keydown` bắn ra mỗi giây — 🟡 `repeat: false` ở lượt rời rạc ĐO ĐƯỢC; **auto-repeat của OS driver không sinh ra**, ghi món cho Ice
- [x] 1.5 Ghi số kèm **ngày + phiên bản engine**; số không truy nguyên được thì không phải số đo

### Task 2 — Nhánh `Backspace` trong `onEditKeydown` (AC: 1, 2, 3, 6)
- [x] 2.1 Dựng helper *"caret ở đầu ô bản dịch"* — xem §Cạm bẫy ④ về vì sao `startOffset === 0` **một mình là sai**
- [x] 2.2 Thêm nhánh vào `onEditKeydown` (`GridPanel.vue:1041`), **sau** chốt `isComposing`, **không** đổi thứ tự dòng đó
- [x] 2.3 Nhánh gọi `mergeCurrentSegment()` — **không** viết logic gộp mới — 🔵 **cài qua `dispatch('editor.merge_segments')`**, xem §Ⓑ Dev Agent Record: đó là ý đồ của câu này ở một bậc **cao hơn**, không một lượt lệch
- [x] 2.4 `event.preventDefault()` khi và chỉ khi nhánh nhận việc; bỏ qua thì engine xoá thêm một ký tự
- [x] 2.5 Vùng chọn **không collapsed** ⇒ **không** gộp, để engine xoá vùng chọn (AC6 nói *không chặn*, không nói *cướp phím*)
- [x] 2.6 Xử lý `event.repeat` theo phán định Task 1.4 + §Câu hỏi ④ — Ice ký ③: **gộp một lần rồi dừng**

### Task 3 — Dòng báo hệ quả (AC: 4)
- [x] 3.1 Mở khe thông điệp: `CONFIRM_NOTICE_KEYS` (`StatusBar.vue:102-106`) là `Record` **ĐÓNG** trên `ConfirmResult` — nó không mang được một thông điệp **thành công**
- [x] 3.2 Dựng kênh thông điệp cho kết quả gộp, giữ nguyên tính **đóng** của bảng tra *(không `?? 'khoá nào đó'` — xem `StatusBar.vue:93-96`)*
- [x] 3.3 Thêm khoá `vi.json` PHẲNG, khoá chấm, tiền tố miền; giọng văn theo UX-DR47 *(nói việc, nêu hệ quả)*
- [x] 3.4 Nối vào `StatusBar.vue` theo đúng khuôn loại trừ `v-if`/`v-else-if` đã có — thanh 34px chỉ chứa **một** mệnh đề
- [x] 3.5 Cho `editorRegroupError` một người đọc: hôm nay nó tồn tại mà **không component nào đọc**, nên một lượt từ chối *(ví dụ `segment.no_previous` ở câu đầu Chương)* **không đổi một pixel nào**
- [x] 3.6 Dọn dòng báo bằng **SỰ KIỆN**, không bằng hẹn giờ — khuôn đã có ở `StatusBar.vue:134-135`

### Task 4 — Nghiệm thu (AC: 1-4, 6)
- [x] 4.1 vitest cho helper *"đầu ô"*: ô rỗng · ô có chữ · caret giữa ô · caret đầu **dòng thứ hai** sau `\n` · vùng chọn không collapsed — **11 ca**, và ca ⑨b sinh ra từ một lượt đột biến bắt được lỗ hổng
- [x] 4.2 vitest cho kênh thông điệp mới (khuôn `tests/frontend/statusBar.test.ts`) — **6 ca**
- [x] 4.3 e2e: ca `Backspace` đầu ô ⇒ gộp; ca câu **đầu Chương** ⇒ từ chối **có báo**; ca caret **giữa ô** ⇒ xoá bình thường, **không** gộp — 🔵 **spec RIÊNG** `e2e/specs/segment-backspace-merge.e2e.mjs`, không nhét vào `segment-merge-split.e2e.mjs`: xem §Ⓒ về trần `mochaOpts.timeout`
- [x] 4.4 Đo lại từ nguồn: `cargo test --locked`, `npm run test`, chín cổng. **Không chép số baseline của 2.8.**

### Task 6 — Cử chỉ đánh dấu chỗ cắt đổi sang `Mod`+click (AC: 7) *(🔵 THÊM 2026-08-17)*
- [x] 6.1 Vị từ `Mod` cho chuột ở `editorSegments.ts`, nhận nền tảng qua **tham số** — §Trap 1 của `keys.ts`, và `commands/README.md:73` cấm `event.metaKey` trần bằng chữ
- [x] 6.2 `onSourceCellMouseUp` thoát sớm khi không giữ `Mod`; **không** đụng phần còn lại của chuỗi
- [x] 6.3 vitest lái **cả hai** nền tảng — nửa Windows của kho không có đường nghiệm thu tại chỗ *(action item A5, retro Epic 1)*
- [x] 6.4 e2e: bấm trơn **hai lượt** *(đúng số `mouseup` của một double-click)* ⇒ 0 dấu cắt; `Mod`+click ⇒ có dấu — đối chứng **hai chiều**
- [x] 6.5 Sửa **hai** ca của `segment-merge-split.e2e.mjs` *(chúng bắn `mouseup` trơn)* — cùng lượt, không để lại một cổng đỏ không ai hiểu
- [x] 6.6 Sửa tài liệu cùng lượt: `EXPERIENCE.md:169` *(văn xuôi)* + bảng Phím `:267` *(một hàng mới)*, kèm 🔵 và ngày

### Task 7 — `Esc` xoá tập điểm cắt (AC: 8) *(🔵 THÊM 2026-08-17)*
- [x] 7.1 `clearEditorSourceCut()` — về `null`, **không** một tập rỗng; vô hại khi chưa có điểm nào
- [x] 7.2 Command `editor.clear_source_cuts` + hợp âm `Escape` + dep ở `main.ts` — vế làm phím **gán lại được** (FR22) và hiện trong bảng phím
- [x] 7.3 🔴 Cửa **thứ hai** ở `onEditKeydown`: `Escape` không mang `Mod` nên `keys.ts:510` chặn nó trong vùng gõ — **đúng chỗ người dùng đang đứng**. Hai cửa, **một** command
- [x] 7.4 **Không** `preventDefault()` — `Esc` còn là phím đóng của các lớp phủ; ăn nó là bịt đường thoát của chúng
- [x] 7.5 vitest 5 ca, gồm ca `isComposing` *(chốt bộ gõ đứng trước nhánh mới)*; `COMMAND_FLOOR` 38 → **39** *(cổng in 47)*

### Task 8 — Chỗ cắt ở tab Hán Việt (AC: 9) *(🔵 THÊM 2026-08-17)*
- [x] 8.1 Bàn đo trên WKWebView thật — đo **hình dạng DOM** và chạy chính phép ánh xạ sản phẩm ở cả hai kiểu xem
- [x] 8.2 `buildSegments` mang **tầm nguồn** (`srcStart`) cho mọi segment, kèm phép ánh xạ `\r\n` → gốc *(FR125 còn `backlog`, `\r` CÓ thể có mặt)*
- [x] 8.3 Phát neo `data-src-start` ra DOM ở cả hai kiểu xem; `.hv-word` thêm `data-src-atomic` *(chữ ký "ranh giới từ" của Ice)*
- [x] 8.4 Nhánh văn bản thuần cũng mang neo — `<span class="src-piece">`, **không** thêm một ký tự nào vào `Selection.toString()`
- [x] 8.5 🔴 `sourceCutOffsetOf` đổi từ **đếm mù** sang **đọc neo**; không neo ⇒ `null` *(từ chối)* — `.hv-sources`, `.hv-notice`, `<rt>` rơi vào đó theo **CẤU TẠO**
- [x] 8.6 Dấu cắt vẽ bằng **`::before`**, không một phần tử con — `resolveSwitch` ánh xạ `host.children[i]` ↔ `segments[i]`, thêm một con là làm tra cứu **sai im lặng**
- [x] 8.7 Sửa 6 ca của `editorSourceCut.test.ts` sang hình dạng mới, **giữ nguyên bài học** (`<rt>` · code point)

### Task 9 — Dấu cắt dễ nhận diện, và gỡ kênh thay thế (AC: 9) *(🔵 THÊM 2026-08-17, Ice dùng thật)*
- [x] 9.1 Dấu cắt cao `1em` → **`1,3em`**, màu `ornament` → **`primary`** — `ornament` là `#a9a196`/`#6a6459`, đo được **2,44/2,64** trên `surface` nên `check:tokens` **cấm** nó làm màu chữ; một dấu cắt là thứ phải **tìm thấy**
- [x] 9.2 Sửa **cả hai** khối CSS cùng lượt (`GridPanel.vue::.cut-mark` và `SourceHanViet.vue::.cut-here::before`) — **không cổng nào canh** việc chúng khớp nhau, ghi cảnh báo tại chỗ
- [x] 9.3 🔴 **ĐO** chiều cao hàng trước/sau — `subgrid` làm một phần tử inline cao hơn line box đẩy **cả track** và kéo ô bản dịch theo *(cái giá đã đo ở 2.5b: 388px)*. Kết quả: **71px → 71px**, chênh **0/0**, dấu cắt có mặt
- [x] 9.4 Gỡ viền `has-cuts` **và gỡ luôn lớp đó** — nó là kênh THAY THẾ cho ngày dấu cắt chưa vẽ được ở tab Hán Việt; AC9 làm nó vẽ được ⇒ kênh hết việc. Giữ một lớp không tạo kiểu gì chỉ để e2e chọn được là mở đúng tiền lệ kho này cố ý chưa mở
- [x] 9.5 Sửa hai spec e2e đọc lớp vừa gỡ sang `data-cut-count` *(chở một SỐ, chặt hơn một cờ)*; sửa hai doc-comment đã hết đúng kèm 🔵 + ngày

### Task 5 — Sổ nợ
- [x] 5.1 Đóng món `deferred-work.md:3036-3061` *(tiền đề `beforeinput` đã lật)* bằng `→ ✅ ĐÃ ĐÓNG <ngày> (Story 2.9)` — **nối thêm, không xoá mục gốc**
- [x] 5.2 Đóng vế *"dòng báo"* của `:4103-4109`; vế `⌘Z` thì **giữ 🟡** và ghi chủ theo phán định Task 0
- [x] 5.3 Ghi món *"`@keydown` nay mang một thao tác thật ⇒ luật Kiểm A phải được xem lại"* — chủ theo phán định của Ice *(xem cạm bẫy ②)*
- [x] 5.4 Rà lại `deferred-work.md:3821-3829` *(`restore_segment_version` khi văn bản rỗng)* — còn hở không, và story này có chạm không — **KHÔNG chạm**, lý do ghi tại chỗ
- [x] 5.5 Ghi mọi món mới phát sinh, **mỗi món một chủ**, không món nào mồ côi — **5 món mới**, 2 món đóng bằng cách nối tiếp

### Review Findings

Lượt code review BA TẦNG *(Blind Hunter · Edge Case Hunter · Acceptance Auditor, `claude-opus-5`)*,
2026-08-17, mốc diff `4d72cd4` → cây làm việc.

🔵 **Mọi số đo của Dev Agent Record đã được đo lại từ nguồn và KHỚP 100%** — `npx vitest run`
177/177 (17 tệp) · `check-commands` 47 command / `COMMAND_FLOOR` 39 · 6 khoá `panel.grid.regroup_*`
· 11/6/6/5/16 ca mỗi tệp test · `src-tauri/**` không đổi một byte · `vue-tsc` sạch. Không một
con số nào là *"số đo không ai đo"*.

- [x] [Review][Patch] **Chỗ cắt giữa một mảnh `.hv-text` có hiệu lực nhưng KHÔNG vẽ được dấu — rộng hơn món nợ đã ghi** — ✅ **Ice ký 2026-08-17: TỪ CHỐI offset giữa mảnh.** `sourceCutOffsetOf` neo về điểm **ĐẦU** mảnh `.hv-text`, đúng khuôn `.hv-word` đã làm qua `data-src-atomic`. Bất biến mới: **mọi offset tạo được đều vẽ được dấu**. Đánh đổi đã cân: người dùng mất độ chính xác giữa `」，` — nhưng không bao giờ có một chỗ cắt tàng hình trên dữ liệu AD-5 không cho hoàn tác, và `resolveSwitch` **không bị đụng**. Chi tiết đường hỏng: `sourceCutOffsetOf` tính offset **chính xác từng ký tự** bên trong mảnh không-Hán (`editorSegments.ts:302-317`, mảnh này **không** mang `data-src-atomic`), nên người dùng đặt được một chỗ cắt hợp lệ ở **giữa** một mảnh. Nhưng `cutSet.has(seg.srcStart)` (`SourceHanViet.vue:806, 828`) chỉ so với điểm **ĐẦU** mảnh ⇒ chỗ cắt đó **không hiện dấu ở đâu cả**, ở **cả hai** kiểu xem. Món nợ đã ghi (`GridPanel.vue:699-700`) chỉ phủ ca *"giữa một TỪ HÁN ở kiểu `parallel`"*, không phủ ca này. Mảnh `.hv-text` là dấu câu/Latin/số — chuỗi `」，` hay `……` dài ≥2 ký tự là chuyện thường trong tiểu thuyết. ⇒ Một chỗ cắt Rust **sẽ thực thi đúng** mà người dùng không thấy nó nằm ở đâu, trên dữ liệu AD-5 không cho hoàn tác. Ba đường sửa khác nhau vật chất, một trong số đó đụng bất biến `resolveSwitch` — cần Ice chốt.
- [x] [Review][Patch] **`buildSegments`/`originalOffsets` (AC9) không có lưới hồi quy tự động** — ✅ **Ice ký 2026-08-17: tách `originalOffsets` sang `editorSegments.ts` + vitest.** Đúng khuôn `caretAtCellStart` đã đi *(một mệnh đề thuần, kiểm được bằng vitest)*. Ca bắt buộc: `\r\n` · `\r` trần · ký tự ngoài BMP. Đóng nửa rủi ro cao nhất — phép ánh xạ làm lệch **mọi** chỗ cắt sau một `\r\n`. Vế `buildSegments` *(đọc `hanVietByChar.value`, cần fixture từ điển)* **không** thuộc lượt này. Chi tiết: `editorSourceCut.test.ts` chỉ kiểm `sourceCutOffsetOf()` trên DOM **dựng tay** đã có sẵn `data-src-start`; không ca nào đi qua `buildSegments()`/`originalOffsets()` thật. Đường duy nhất verify phép ánh xạ `\r\n`→gốc và ca ngoài BMP là `2-9-ban-do/han-viet-cho-cat.e2e.mjs` — một **bàn đo tay**, không nằm trong `e2e/specs/`, không chạy trong `npm run test`. `originalOffsets` là hàm **thuần**, tách được ngay; `buildSegments` đọc `hanVietByChar.value` nên tách cần tiêm phụ thuộc. Phạm vi phủ đến đâu là một quyết định.
- [x] [Review][Patch] **`confirmNotice` đè `regroupNotice` — AC4 KHÔNG đạt trong một ca thật, và chú thích tại chỗ tự khai SAI** [src/StatusBar.vue:195] — `v-if="confirmNoticeKey !== null"` thắng vô điều kiện `v-else-if="regroupNoticeText !== null"`. Chú thích khẳng định *"Hai ô nhớ không bao giờ cùng mang giá trị trong một lượt dùng thật"* — **sai, đo được**: `confirmNotice` chỉ bị dọn ở BA chỗ (`editorPanelState.ts:298` · `:491` · `:730`), và `regroup()` **không** nằm trong số đó. Ba giá trị `'no-caret'`/`'flush-failed'`/`'still-dirty'` ở lại vô thời hạn. Cử chỉ `Backspace` **không** gọi `noteEditorEdit` (`preventDefault()` cắt hẳn chuỗi `beforeinput`→`input`→`reportEdit`). ⇒ `⌘Enter` hụt → click sang câu khác → `Backspace` gộp **THÀNH CÔNG** → thanh vẫn hiện câu cũ. Chiều ngược lại cũng hỏng: một `regroupNotice` cũ che mốc *"Đã lưu"* mà UX-DR30 đòi. **Hai tầng độc lập tái hiện bằng vitest thật.** Không ca nào trong 177 ca dựng lại kịch bản có sẵn một `confirmNotice` trước lượt gộp.
- [x] [Review][Patch] **`resetEditorPanel()` không dọn `regroupNotice`/`regroupError` — câu báo rò sang Tác phẩm khác** [src/panels/editorPanelState.ts:443] — hàm dọn `confirmError` · `caretPlacement` · `confirmNotice` nhưng bỏ hai ô nhớ mới của story này (`:1125`, `:1150`). 🔴 Chính khối chú thích ngay trên nó (`:479-487`) viết ra bằng chữ cái luật *"áp cho mọi ô nhớ THÊM VÀO TỆP NÀY sau này: hỏi 'ô này thuộc Tác phẩm hay thuộc ứng dụng?'"* — và ghi rõ `segment.id` đếm lại từ 1 ở **từng** Tác phẩm nên đụng id là chuyện gần như chắc chắn. ⇒ Ở Tác phẩm B, câu *"Câu số N là câu đầu Chương…"* của Tác phẩm A hiện lên **trước khi** người dùng bấm gì. Đúng lớp lỗi đã bị bắt và vá ở code review 2026-08-15, tái diễn ở story tiếp theo trong **cùng tệp**.
- [x] [Review][Patch] **Không có khoá `regroupInFlight` — hai lượt `Backspace` RỜI RẠC bấm nhanh dispatch hai lần** [src/panels/editorPanelState.ts:1212] — `event.repeat` (`GridPanel.vue:1158`) chỉ chặn auto-repeat của **hệ điều hành**, không chặn hai cú bấm rời rạc (*"bấm lại cho chắc"*). `confirmCurrentSegment` **đã có** khoá riêng cho đúng đường hỏng này (`confirmInFlight`, `:738`, thêm ở code review 2026-08-13 kèm lý do *"một promise chứ không một cờ"*); `regroup`/`mergeCurrentSegment` **không có**. Vì `preventDefault()` giữ nguyên DOM và caret ở offset 0, cú bấm thứ hai vẫn qua `caretAtCellStart` và dispatch lại cho **cùng** `id`. Lượt IPC thứ hai trả `refused` (segment đã về hưu) và **ghi đè** `regroupNotice` từ `'merged'` thành `'refused'` ⇒ thanh báo *"chưa gộp được"* cho một thao tác **đã gộp xong** — nói dối đúng chiều nguy hiểm, trên dữ liệu không lui được.
- [x] [Review][Defer] **`sourceCut` (Story 2.8) cũng không được dọn ở `resetEditorPanel()`** [src/panels/editorPanelState.ts:443] — deferred, pre-existing. Cùng lớp với món Patch ở trên nhưng thuộc 2.8, nằm ngoài diff này. ⚠️ Nó củng cố một quan sát đáng ghi: luật *"mọi ô nhớ mới phải qua `resetEditorPanel()`"* **không có cổng nào canh**, và đã bị bỏ sót **hai story liên tiếp**.

#### Ⓐ Lượt vá — năm món, và một lỗ hổng trong lưới của CHÍNH lượt review

🔴 **Cả năm bản vá được nghiệm thu bằng ĐỘT BIẾN MÃ SẢN PHẨM**, không bằng một lượt chạy xanh:

| Đột biến | Ca bắt được |
|---|---|
| `ghiRegroupNotice` không dọn `confirmNotice` | ① — 1 đỏ |
| `confirmCurrentSegment` không dọn `regroupNotice` | ①b — 1 đỏ |
| `resetEditorPanel` không dọn hai ô nhớ | ② — 1 đỏ |
| gỡ khoá `regroupInFlight` | ③ và ③b — 2 đỏ |
| gỡ `data-src-atomic` khỏi `.hv-text` *(lượt đầu)* | 🔴 **KHÔNG ca nào — 191/191 xanh** |
| gỡ `data-src-atomic`, **sau khi thêm `hanVietCutAnchors.test.ts`** | 2 đỏ *(cả hai kiểu xem)* |

🔴 **Lượt thứ năm là một lỗ hổng thật trong lưới tôi vừa viết, và chính đột biến tìm ra nó.**
`editorSourceCut.test.ts` dựng DOM **bằng tay** rồi tự gắn `data-src-atomic` ⇒ nó canh
`sourceCutOffsetOf` **tôn trọng** neo, nhưng **không** canh `SourceHanViet.vue` **phát ra** neo.
Hai mệnh đề khác nhau, và chỗ cắt tàng hình xuất hiện khi vế **thứ hai** hỏng. Đóng bằng một tệp
mount **component thật** — đúng khuôn ca ⑨b mà lượt dev đã tự trả giá để biết.

⚠️ **Một vế Ice chốt KHÔNG phủ trọn như câu đề xuất của tôi viết, và tôi thu hẹp nó thay vì thi
hành quá tay.** Bất biến *"mọi offset tạo được đều vẽ được dấu"* đòi neo cả `.hv-unit` *(base
`<ruby>` ở `parallel`)* — nhưng làm thế **vi phạm AC9**, vốn đòi bằng chữ *"chính xác từng chữ ở
`parallel`"*. ⇒ Chỉ `.hv-text` mang neo; bất biến thu về *"mọi offset trong một mảnh KHÔNG-Hán đều
vẽ được dấu"*, và ca *"giữa một TỪ HÁN ở `parallel`"* **vẫn là món nợ đã ghi có chủ**, không bị
lượt này đóng. Một ca test giữ cho nó không bị lặng lẽ đóng bằng một dòng thuộc tính.

🔵 **`'busy'` từ chối và KÊU, không NHẬP vào lượt đang bay** — khác `confirmInFlight` có chủ ý.
Lượt xác nhận là một thao tác trên một câu nên nhập vào là đúng; `regroup` là cửa chung của **hai**
lệnh, nên cho lượt sau nhận kết quả lượt trước là **đánh rơi một thao tác người dùng trong im
lặng** — đúng lớp vừa vá, chỉ dời chỗ.

**Số đo sau lượt vá — đo lại từ nguồn, không chép của lượt dev:**

| Đường | Sau lượt dev | Sau lượt vá | Δ |
|---|---|---|---|
| `npx vitest run` | 177/177, 17 tệp | **199/199, 19 tệp** | **+22** ca, +2 tệp |
| Chín cổng đọc-tệp | 9/9 | **9/9 exit 0** | — |
| `check:scope` · `check:scope:bundled` | exit 0 | **exit 0 · exit 0** | — |
| `npm run build` · `vue-tsc --noEmit` | xanh | **exit 0 · exit 0** | — |
| `COMMAND_FLOOR` | sàn 39 | **sàn 39** | 0 — không command mới |
| `src-tauri/**` | không đổi | **`git diff` rỗng** | 0 ⇒ số `cargo` 401/0/5 đứng theo cấu tạo |

**e2e sau lượt vá — chạy lại hai spec đúng đường bị chạm, trên WKWebView 605.1.15 thật:**

| Spec | Vì sao spec này | Kết quả |
|---|---|---|
| `segment-backspace-merge.e2e.mjs` | khẳng định **câu chữ** của thanh trạng thái — đúng thứ vá ① ② ③ đụng | **3/3 xanh** (3m 43s) |
| `segment-merge-split.e2e.mjs` | canh đường `⌘M`/`⌘/`, tức cửa chung `regroup()` nay có khoá | **3/3 xanh** (3m 22s) |

🔵 **Vá ④ KHÔNG đụng đường e2e nào, và đó là một phép đo chứ không một suy luận:**
`grep -ln "hv-\|han-viet\|hanViet" e2e/specs/*.mjs` ⇒ **rỗng**. Không spec nào chạm tab Hán Việt;
bốn spec đặt chỗ cắt đều bấm ở **cột nguyên văn thuần**, nơi `.src-piece` **không** mang
`data-src-atomic` nên phép đếm từng ký tự ở đó giữ nguyên. Đường verify của vá ④ là bàn đo tay
`han-viet-cho-cat.e2e.mjs`, và nó nằm ngoài `e2e/specs/` theo thiết kế.

⚠️ **Chưa chạy e2e TRỌN BỘ** *(~15 phút, cần máy rảnh)*. Hai spec trên là hai spec duy nhất chạm
bề mặt lượt vá sửa; ba spec đỏ-trong-bộ mà lượt dev ghi *(`attribution-focus` ·
`editor-confirm-segment` · `editor-typing-flush`)* **không** chạm bề mặt nào của lượt vá này, và
món *"xanh riêng, đỏ trong bộ"* đã có chủ ở `deferred-work.md`.

---

## Dev Notes

### Đường dây đã có — vẽ đầy đủ, đừng dựng lại một mảnh nào

```
Backspace ở đầu ô (MỚI — story này)
  │
  └─→ mergeCurrentSegment()                    src/panels/editorPanelState.ts:1229
        │   const id = caretSegmentId.value
        │   return await regroup(id, () => mergeSegments(id), 'gop')
        │
        └─→ regroup(id, goi, ten)               editorPanelState.ts:1163
              ① flushEditorBeforeDiscreteWrite()          :1170   ← AD-35 vế (c)/(d)
              ② await mergeSegments(id)                    :1181
                   └→ invoke('merge_segments', { segmentId })   src/config/segment.ts:829
                        └→ wire::merge_segments               src-tauri/src/commands/segment.rs:2800
                             └→ merge_segments(open, segment_id)          segment.rs:2353
                                  ├→ câu liền trên: ORDER BY ord DESC, id DESC LIMIT 1   :2392
                                  ├→ core::segment::regroup::merge(...)   regroup.rs:163
                                  └→ write_regroup(tx, ...)               segment.rs:2195
              ③ applyRegroup(outcome)                     :1190   ← đặt mốc AD-47 ①
              ④ setEditorCaret(new_segments[0].id)        :1204
              ⑤ dọn sourceCut nếu trỏ hàng vừa về hưu     :1219
```

**Trả về:** `RegroupResultCode = 'done' | 'no-caret' | 'no-cut' | 'flush-failed' |
'still-dirty' | 'refused'` (`editorPanelState.ts:1021`). Hàm **trả về, không ném** — đúng
luật *"hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném, nó KÊU"*.

⇒ **Không sửa một dòng Rust nào cho AC1–4 và AC6.** AC2 (nối hai vế), AC3 (về hưu + lịch sử
rỗng), luật xuất xứ AD-47④ và cờ kết đoạn AD-37 đều đã cài và đã có test hợp đồng.

⚠️ **Mệnh đề này KHÔNG phủ AC5.** Một mô hình hoàn tác — bất kể đường (A) hay (B) ở Task 0.2
— sẽ đụng tầng ghi Rust *(gỡ `retired_at` là một lượt ghi mới; và mọi lệnh ghi phải đi qua
`store::Writer` nối tiếp)*. Phạm vi Rust của AC5 chỉ biết được **sau** `AD-48`.

### AC2/AC3 đã có chủ ở tầng Rust — đừng dựng nguồn sự thật thứ hai

| Mệnh đề | Đã cài ở | Đã khoá bởi |
|---|---|---|
| Nối `source_text` theo **ngôn ngữ nguồn** (`''` cho `zh`, `' '` khác) | `regroup.rs:100` `source_joiner` | `segment_contract.rs` |
| Nối `target_text` bằng khoảng trắng cố định *(bản dịch luôn tiếng Việt)* | `regroup.rs:108` `join_targets` | `segment_contract.rs` |
| Lọc bỏ mảnh **rỗng** trước khi nối *(tránh `"A "` — một ký tự người dùng chưa gõ)* | `regroup.rs:121` | code review 2.8 |
| Xuất xứ: cùng ⇒ giữ; bất đồng ⇒ `other` (AD-47④) | `regroup.rs:144` `merged_origin` | `segment_contract.rs` |
| Cờ kết đoạn theo **câu cuối** nhóm | `paragraph.rs:99` `merged()` | `segment_contract.rs` |
| Về hưu **không** tạo `segment_version` | `segment.rs:2173` | `neither_merge_nor_split_ever_writes_a_segment_version_row` |
| Lịch sử câu về hưu vẫn tra lại được | — | `the_history_of_a_genuinely_retired_segment_still_reads_back_after_a_real_merge` |

🔴 **Đừng viết lại ca hợp đồng Rust cho *"gộp bằng Backspace"*.** Nghiệp vụ dùng chung; ca
mới duy nhất có giá trị là ca **chuỗi dây** (e2e), vì cử chỉ khác.

---

### 🔴 Sáu cạm bẫy — mỗi cái đã có bằng chứng, không phải một khả năng lý thuyết

#### ① `Backspace` ở offset 0 KHÔNG phát `beforeinput` trên WebKit

`deferred-work.md:3036-3061`, đo 2026-08-14 (Story 2.5b Task 1.2/1.3):

| Engine | đầu ô **CÓ CHỮ** | ô **đã rỗng** |
|---|---|---|
| WKWebView 605.1.15 (`execCommand('delete')`) | **0** `beforeinput` | **0** `beforeinput` |
| Playwright-WebKit (phím **vật lý**) | **0** `beforeinput` | **0** `beforeinput` |
| Blink (phím **vật lý**) | `deleteContentBackward`, huỷ được | `deleteContentBackward`, huỷ được |

⇒ `onBeforeInput` (`GridPanel.vue:877`) **không dùng được** làm điểm móc, dù nó đang là
*"cửa duy nhất mà một lượt sửa văn bản đi qua"*. Đường còn lại là `keydown`.

⚠️ Đây là chỗ dev agent dễ đi sai nhất: `onBeforeInput` có doc-comment dài nói nó phủ ba
đường mà `keydown` bỏ lọt, và câu đó **vẫn đúng** — nhưng nó không đúng cho **ca này**, và
nửa Blink sẽ cho một bàn phát triển **xanh** trên một sản phẩm macOS **chết**.

#### ② Một command `Backspace` trần qua `CommandRegistry` sẽ KHÔNG BAO GIỜ chạy

`src/commands/keys.ts:510`:

```ts
if (lacksPrimaryMod(entry.mods) && isTypingZone(event.target)) return false
```

`lacksPrimaryMod = (m) => !m.meta && !m.ctrl` (`:415`); `isTypingZone` trả `true` cho mọi
phần tử `isContentEditable` (`:434-439`); ô bản dịch khai `contenteditable="true"` tĩnh
(`GridPanel.vue:1302`).

⇒ Đăng ký `editor.merge_backspace` với hợp âm `'Backspace'` cho **đúng cái không bao giờ
bắn** — ở đúng chỗ duy nhất thao tác này có nghĩa. `Backspace` **có** trong `NAMED_CODES`
(`keys.ts:102`) và có glyph `⌫` (`:308`), nên nó **đăng ký được** và **không cổng nào đỏ**.

🔴 Đây là bẫy nguy hiểm nhất của story, vì AC8 của Story 2.8 vừa dạy *"phải là command đã
đăng ký, không phải hệ quả phụ của việc gõ"* — và luật đó áp cho `⌘M`/`⌘/`, **không** cho
cử chỉ này. UX-DR32 tách hai vế đó ra bằng chữ: cử chỉ `Backspace` **là** hệ quả của việc
gõ, đó là điểm của nó.

⇒ Bắt **trực tiếp trong `onEditKeydown`**. `check:commands` Kiểm A chỉ canh `@click`, nên
nhánh này không vi phạm cổng — và cũng nghĩa là **không cổng nào bắt được** nếu nó lệch khỏi
`mergeCurrentSegment()`.

🔵 **Và story này chính là cái ngày mà cổng tự dặn trước.** `scripts/check-commands.mjs:2348-2349`
in ra mỗi lượt chạy:

> *"1. Kiểm A chỉ canh `@click`. `@keydown`/`@input`/`@submit` KHÔNG thuộc luật này; **ngày
> một `@keydown` mang thao tác thật xuất hiện, luật phải được xem lại**."*

Trước story này, `onEditKeydown` **không mang thao tác nào** — nó chỉ có chốt `isComposing`.
Sau story này nó mang một thao tác **phá huỷ và không lui được**. ⇒ Task 5 phải **ghi một
món nợ có chủ** cho lượt xem lại đó *(hoặc nêu với Ice nếu nên đóng ngay)*. Đi qua im lặng
là đúng thứ mà chính dòng dặn ấy tồn tại để chống.

#### ③ Chốt `isComposing` phải đứng trước mọi nhánh mới

`GridPanel.vue:1042-1047` mang một dòng dặn viết sẵn cho chính story này:

> 🔴 **DÒNG NÀY KHÔNG ĐƯỢC CHẠM** […] Nó đứng TRƯỚC mọi nhánh khác, và nó phải ở lại kể cả
> khi hàm này không còn nhánh nào sau nó: một hàm rỗng ở đây là một chỗ mà lượt sửa kế tiếp
> sẽ viết vào, và người viết sẽ không biết về chốt này.

Người dùng sản phẩm này gõ tiếng Việt **bằng bộ gõ**. Một lượt commit composition phát
`keydown` mang `code` vật lý — ăn nó là ăn mất chữ.

⚠️ **Không đường nghiệm thu nào của dự án bắt được lớp lỗi này** — không bộ chạy test nào
mô phỏng được một bộ gõ tiếng Việt thật. Nó chỉ lộ ở tay người dùng.

#### ④ `startOffset === 0` KHÔNG có nghĩa là "đầu ô"

Từ Story 2.5d (FR134/AD-46), ô bản dịch **chứa được `\n`**, và `white-space: pre-line`
(`GridPanel.vue:1623`) làm engine dựng `\n` thành **text node thật**
(`textContent === "A\nB"`).

⇒ Caret ở **đầu dòng thứ hai** cũng cho `startOffset === 0`. Một phép kiểm chỉ hỏi offset
sẽ **gộp câu khi người dùng chỉ muốn xoá một lần xuống dòng** — mất một segment vì một phép
kiểm đọc thiếu một node.

Phép kiểm đúng phải hỏi: **không còn ký tự nào phía trước caret trong cả ô**. Ô bản dịch
render **một mustache duy nhất** (`{{ s.target_text }}`, `:1303`), không có `#comment` chen
giữa như cột nguyên văn — nên nó đơn giản hơn `sourceCutOffsetOf` nhiều, nhưng **không**
đơn giản bằng `startOffset === 0`.

⚠️ Ca ô **rỗng**: `cell.childNodes.length === 0`, `startContainer` là **chính ô**. Đây là ca
thường nhất *(xoá lui hết câu rồi bấm thêm một lần)* và là ca khuyết tật *"sập hố"* cũ. Bàn
đo Task 1.3 phải phủ nó.

#### ⑤ Vùng chọn không collapsed ⇒ đừng gộp

`Backspace` khi đang bôi đen là *"xoá vùng chọn"*, không phải *"xoá lui"*. Cướp phím ở đó
làm người dùng mất cả đoạn vừa bôi đen **và** mất một ranh giới câu, cùng lúc, bằng một
phím họ bấm hàng trăm lần mỗi Chương.

#### ⑥ Ca **câu đầu Chương** phải KÊU, không im

`merge_segments` từ chối bằng `segment.no_previous` (`segment.rs:2067`), khoá i18n đã có:

> `err.segment.no_previous`: *"Câu số {segment_id} là câu đầu Chương — không có câu nào phía
> trên để gộp vào."* (`vi.json:23`)

Hôm nay `editorRegroupError` (`editorPanelState.ts:1103`) **chưa component nào đọc** ⇒ lượt
từ chối **không đổi một pixel nào**. Người dùng bấm `Backspace` ở câu đầu Chương và **không
có gì xảy ra** — đúng lớp *"rỗng IM LẶNG"* mà `project-context.md` cấm. Task 3.5 đóng nó.

---

### AC4 — "dòng báo ở lề" và một chỗ lệch giữa hai tài liệu

🔴 **UX-DR32 nói *"ở lề"*; chữ ký của Ice nói *"thanh trạng thái"*.** Hai chỗ, và story này
phải đi theo một:

- `epics.md:2560` + `EXPERIENCE.md:171`: *"một dòng báo ở **lề**"*.
- **Quyết định #8, Ice ký 2026-08-14** (Story 2.5): hợp đồng UX-DR30 tối thiểu là *"cột nhãn
  trạng thái của **chính hàng** cộng **một dòng ở thanh trạng thái**"*.

**Bằng chứng nghiêng về thanh trạng thái:**
- Lưới có **đúng năm cột** (`grid-template-columns: 3px 30px minmax(0,1fr) minmax(0,1fr) 96px`,
  `GridPanel.vue:1391`). **Không có cột lề thứ sáu** nào để chứa một dòng báo.
- Ba mockup vẽ dòng báo là một **khối nổi trong luồng văn bản** đều vẽ **Editor liền mạch** —
  hình dạng đã bị UX-DR13 lật ngày 2026-08-14. **Không mockup nào** vẽ dòng báo trong lưới.
- `check:tokens` **Kiểm F** cấm bóng đổ, gradient, **lớp nổi** ⇒ dựng lại đúng mockup cũ
  (nền riêng `#faf4e8`, bo góc, viền trái màu) là một lượt cổng **đỏ**.

⇒ **Khuyến nghị: đi theo chữ ký của Ice** — mở khe `StatusBar`. Nhưng đây là một chỗ hai
tài liệu nói hai điều, nên nó nằm trong §Câu hỏi cho Ice.

**Ràng buộc kỹ thuật của khe đó:**
- Thanh cao `--space-status-height` = **34px**, chỉ chứa **một** mệnh đề. Câu báo hiện dùng
  `v-if`/`v-else-if` để **thay chỗ** mốc *"Đã lưu"*, không đứng cạnh nó (`StatusBar.vue:137-140`).
- `CONFIRM_NOTICE_KEYS` là `Record` **ĐÓNG** trên ba giá trị của `ConfirmResult`
  (`:102-106`). Doc-comment cấm một nhánh mặc định: *"một `?? 'khoá nào đó'` sẽ nuốt im lặng
  một giá trị thứ tư"*. ⇒ Mở khe **phải giữ tính đóng** — thêm một `Record` thứ hai đóng
  trên kiểu mới, không nới cái cũ thành `string`.
- Nó chỉ mang **lỗi**. Story này cần một thông điệp **thành công**, tức một kiểu mới.
- `role="status"`, `aria-live` mặc định (`polite`) — **không** `role="alert"` (`:117-122`).
- Token: `--face-ui-sm` / `--font-ui-sm` / `--leading-ui-sm`, màu
  `--color-on-surface-variant` (`StatusBar.vue:186-191`). **Không màu cảnh báo riêng** —
  `:178-185` nêu lý do: các câu đi qua đây nói *"thao tác chưa xong"*, không *"dữ liệu
  hỏng"*; nhuộm đỏ là dạy người dùng rằng thanh trạng thái có một mức khẩn cấp, và mức đó
  **sẽ phải lạm phát ở story sau**. Câu báo thành công của story này càng đúng luật đó.
- Dọn bằng **sự kiện** (`noteEditorEdit`), **không** bằng hẹn giờ (`:134-135`).

**Giọng văn** — UX-DR47: nói việc, nêu **hệ quả**, không xưng *"chúng tôi"/"bạn"*. Câu ví dụ
trong AC4 đã đúng khuôn; dùng nguyên nếu không có lý do đổi.

---

### Ranh giới phạm vi — bốn thứ KHÔNG thuộc story này

1. **Gộp một NHÓM `n` câu.** `deferred-work.md:4002-4013` — 🟡 đóng một nửa, cần *"bề mặt
   chọn nhiều hàng"* chưa tồn tại. Chủ: **một story sau của Epic 2**.
   ⚠️ `Backspace` liên tiếp **vẫn chạy** — mỗi lượt gộp 2→1 rồi dừng. Đó **không** phải
   *"tích luỹ rồi gộp một lượt"*, nên **không mở lại** món nợ này. Story 2.8 đã **bác bằng
   số đo** đường *"gọi `⌘M` nhiều lần để có một nhóm"*: nó cho **5 hàng về hưu thay vì 3**
   cộng một segment trung gian mang một `id` không ai từng thấy. Cơ chế của `Backspace`
   khác — mỗi lượt là một lượt gộp **hoàn chỉnh**, không có segment trung gian.
2. **Xung đột `⌘M` với Quản lý TM** (`mockups/tm-manage.html:128`) — chủ **Epic 7**.
3. **Cặp TM ở lại nguyên khi gộp** — chủ **Epic 7** (bảng TM chưa tồn tại).
4. **Luật `is_omitted` khi gộp chưa có chỗ đứng trong spine** — chủ **Ice**. Story này **tái
   dùng** `regroup::merge` nên thừa hưởng luật đó; đừng cài lại, cũng đừng "sửa" nó.

⚠️ Một món cần **rà lại**, chưa chắc thuộc phạm vi: `deferred-work.md:3821-3829` —
`restore_segment_version` không hỏi lại khi văn bản hiện tại **rỗng**; ghi chủ là Story 2.8
nhưng phần tổng kết nợ của 2.8 **không nhắc lại**. Story này sinh thêm đường tạo segment
rỗng do gộp ⇒ kiểm xem món còn hở không, và nếu còn thì ghi chủ, **đừng tự đóng**.

---

### Nghiệm thu — bốn đường, bốn vai, chọn đúng đường

| Mệnh đề | Đường | Vì sao không phải đường khác |
|---|---|---|
| Helper *"đầu ô"* trả đúng/sai trên năm hình dạng `Selection` | **vitest** | Là hàm thuần; `happy-dom` dựng được `Range` giả |
| Kênh thông điệp mới hiện đúng câu | **vitest** | Khuôn `tests/frontend/statusBar.test.ts` đã có |
| `Backspace` thật trong WKWebView ⇒ gộp thật | **e2e** | `happy-dom` **không phải WebKit**; chính mệnh đề `beforeinput` đã lật ở đây |
| Nối văn bản, xuất xứ, cờ đoạn, về hưu | **đã có chủ ở `segment_contract.rs`** | Dựng lại = nguồn sự thật thứ hai |
| Vế thị giác của dòng báo | **bàn đo tay** | Cổng chụp được ảnh, nó không **phán xét** ảnh |

**Cây test frontend:** `tests/frontend/**`, **không** đồng vị trí trong `src/**` — bốn cổng
đếm quần thể `src/**`, một tệp test đổ vào đó thổi phồng mẫu số.

**e2e — hai luật tại chỗ, đã trả giá để biết:**
- `e2e/specs/segment-merge-split.e2e.mjs` dùng `browser.execute` dispatch một
  `KeyboardEvent` **tổng hợp**, **không** `browser.keys()` — driver gửi sai `code` *(đã làm
  `⌘/` câm một lượt: `code: "/"` vs `"Slash"`)*. Ca `Backspace` đi cùng khuôn:
  `new KeyboardEvent('keydown', { key: 'Backspace', code: 'Backspace', bubbles: true, cancelable: true })`.
- **Cấm `.click()` của driver** — dùng `realClick()` ở `e2e/support/pointer.mjs`. Driver bắn
  `click` **trước** `focusin`, ngược chuột thật.
- Chạy e2e trên máy **rảnh**: `FLUSH_WAIT_MS = 3500ms` chỉ hơn biên 1500ms; một máy đang
  biên dịch Rust làm ca chập chờn. **Đừng nới hằng số** — đó là vá triệu chứng.

**Cổng:**
- `check:layout` Kiểm C là **danh sách CHO PHÉP** cho mọi thành viên `window.`/`document.`
  mà `src/**` chạm tới. `window.getSelection` đã có *(GridPanel dùng ở nhiều chỗ)*; bất kỳ
  API **mới** nào phải thêm tường minh — thêm một cái tên là một quyết định phải viết ra.
- `check:i18n` Kiểm E `import()` thẳng `src/i18n/resolve.ts` bằng Node trần ⇒ **luật
  erasable-only** đứng nguyên. Khoá mới phải khớp hai chiều với `vi.json`.
- `check:commands` sàn `COMMAND_FLOOR` — story này **không thêm command nào** *(xem cạm bẫy
  ②)*, nên sàn không đổi. Nếu nó đổi, đó là dấu hiệu đã đi sai đường.

---

### Bài học từ Story 2.8 — thứ dev agent trước đã trả giá để biết

- **Món nợ đóng ở lượt code review, không ở lượt dev.** Diff của 2.8 *"chỉ nối thêm ở cuối
  tệp và bỏ sót cả bốn món có chủ 2.8"*. ⇒ Task 5 không phải một việc dọn dẹp cuối; đọc
  **từng** món có chủ 2.9 và đóng **tại chỗ**.
- **`caretPositionFromPoint` NÉM `TypeError` trên WKWebView** — đã vá ở 2.8 bằng
  `caretPointAt` (`GridPanel.vue:463`), thử `caretRangeFromPoint` **trước**. Story này đọc
  `Selection` **hiện có**, không dò từ toạ độ, nên nhiều khả năng không chạm — nhưng nếu
  chạm thì dùng đường đã vá, đừng gọi `caretPositionFromPoint` trần.
- **`applyRegroup` gỡ hàng về hưu khỏi lưới** (`:1114-1130`) — chữ ký #6 đã **lật một lần**
  sau khi Ice dùng thật *(bản cũ giữ hàng về hưu lại với vạch mờ)*. Đừng "khôi phục" hành vi
  cũ vì thấy nhánh `'ornament'` còn trong mã — đó là một món nợ có chủ **Ice**.
- **`.at(0)` chứ không `[0]`** cho dữ liệu qua dây — `[0]` khai kiểu **đặc** nên phép kiểm
  `undefined` thành *"thừa"* và `no-unnecessary-condition` đỏ; đường sai rẻ là một
  `eslint-disable`. Dữ liệu qua IPC là một **lời khai**, không một bảo đảm.
- **Đo lại từ nguồn.** Số cuối 2.8 *(cargo 401, vitest 141/141, `segment_contract` 121,
  `COMMAND_FLOOR` sàn 38)* là **ảnh chụp 2026-08-17**, và cây hiện có **chưa commit**. Chép
  chúng vào story này là dựng một *"số đo không ai đo"*.

### Git — trạng thái cây khi story này được soạn

Ba commit gần nhất: `dfa9c95` (2.7 hoàn tất) · `9f46bd9` (2.7) · `5a7e007` (AD-47 vào spine).

🔴 **Cây đang BẨN, và toàn bộ chỗ bẩn là công việc của Story 2.8** — 20 tệp sửa
(+2.870/−48) cộng 5 tệp chưa theo dõi, **trong đó có `src-tauri/src/core/segment/regroup.rs`
và `e2e/specs/segment-merge-split.e2e.mjs`** — tức chính hạ tầng story này đứng lên.
`sprint-status.yaml` đã ghi `2-8: done`.

⇒ **Commit 2.8 riêng, TRƯỚC, và hỏi Ice trước khi commit** — diff của story 2.9 phải đọc
được một mình. Xem §Câu hỏi ③.

---

### Project Structure Notes

**Tệp sẽ sửa** — tất cả đều **UPDATE**, không tệp nguồn mới nào là bắt buộc:

| Tệp | Sửa gì | Giữ nguyên gì |
|---|---|---|
| `src/panels/GridPanel.vue` | Nhánh `Backspace` trong `onEditKeydown` (`:1041`); có thể thêm helper *"đầu ô"* | Chốt `isComposing` **dòng đầu**; `onBeforeInput` **không đụng**; `white-space: pre-line` (`:1623`) là tiền đề vận hành, không trang trí |
| `src/StatusBar.vue` | Khe thông điệp thứ hai cho kết quả gộp | Tính **đóng** của bảng tra; khuôn `v-if`/`v-else-if`; `role="status"` |
| `src/panels/editorPanelState.ts` | State thông điệp gộp + người đọc cho `editorRegroupError` | `regroup()`, `mergeCurrentSegment()`, `applyRegroup()` — **không đụng** |
| `src/i18n/vi.json` | Khoá mới, PHẲNG, khoá chấm, tiền tố miền | Không giá trị rỗng; placeholder khớp `[a-z_][a-z0-9_]*` |
| `tests/frontend/*.test.ts` | Ca mới cho helper + kênh thông điệp | Cây test **không** đổ vào `src/**` |
| `e2e/specs/segment-merge-split.e2e.mjs` | Ca `Backspace` | Khuôn `KeyboardEvent` tổng hợp; `realClick()` |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Đóng/ghi nợ | **Không xoá** mục đã đóng |

**Không sửa:** `src-tauri/**` *(nghiệp vụ gộp đã đủ cho AC1–4, AC6 — xem cảnh báo về AC5 ở
§Đường dây)* · `epics.md` · `prd.md` · `ARCHITECTURE-SPINE.md` *(một `AD` mới đi qua thủ tục
viết ra, không qua một lượt tiện tay — và nó do Winston soạn, không do dev)*.

**Đặt tên:** Rust `snake_case` · Vue `PascalCase.vue` · khoá i18n phẳng có tiền tố miền
(`panel.grid.*` / `status.*`) · command id cùng văn phạm khoá chấm. Ánh xạ thuật ngữ cố định:
segment → `Segment`, **không** `Project`/`Book`/`Novel`/`Document` cho `Work`.

**Chú thích:** tiếng Việt, dày, chở **LÝ DO**. Một quyết định không hiển nhiên phải kèm một
**PHÉP ĐO** — con số, ngày đo, `tệp:dòng`. Ký hiệu chỉ đứng **đầu câu**: 🔴 luật không được
phá · ⚠️ bẫy/giới hạn · ✅ đã đóng · 🟡 đóng một nửa · 🔵 mệnh đề cũ đã hết đúng · ⇒ kết luận.
🔴 Emoji `U+26D4` **bị cấm trong toàn kho** — viết `không`/`KHÔNG` thành chữ.

---

### References

- `_bmad-output/planning-artifacts/epics.md:2531-2568` — Story 2.9, sáu AC
- `_bmad-output/planning-artifacts/epics.md:595-597` — UX-DR32 (kèm khối 🔵 sửa 2026-08-14)
- `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md:437` — FR78
- `.../architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md:103-111` — AD-5
- `ARCHITECTURE-SPINE.md:419-425` — AD-35 (hợp đồng flush) · `:406-417` — AD-34 · `:75-79` — AD-1
- `ARCHITECTURE-SPINE.md:675-745` — AD-47 (mốc so + xuất xứ; ④ là luật gộp)
- `_bmad-output/planning-artifacts/sprint-change-proposal-2026-08-14.md:294` — lượt đổi tên + đổi tiền đề
- `.../ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md:171` — UX-DR32 bản đã sửa · `:261-268` — bảng phím
- `_bmad-output/implementation-artifacts/2-8-gop-va-tach-segment-tuong-minh.md:190-214` — Quyết định #9, Ice ký (a)
- `deferred-work.md:3036-3061` — đo `beforeinput` trên WebKit · `:4103-4109` — khe `StatusBar` + `⌘Z`
- `src/panels/GridPanel.vue:42-46, 1041-1052, 1252-1261, 1289-1303, 1623`
- `src/panels/editorPanelState.ts:1021-1027, 1101-1103, 1132-1154, 1163-1221, 1229-1233`
- `src/commands/keys.ts:415, 434-439, 502-524` — luật vùng gõ
- `src/StatusBar.vue:85-112, 117-122, 134-140, 181-191`
- `src/config/segment.ts:366, 797-848` · `src-tauri/src/commands/segment.rs:2027-2032, 2353-2443, 2800-2813`
- `src-tauri/src/core/segment/regroup.rs:41-72, 100-127, 144-196`

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Amelia, dev-story) — 2026-08-17

### Ⓐ Task 0 — cửa chặn phán định, ba chữ ký của Ice

**Điều kiện khởi hành của story ĐÃ HẾT ĐÚNG.** Story viết *"cây đang BẨN, 20 tệp sửa + 5 tệp chưa
theo dõi, toàn bộ là Story 2.8"* và Câu hỏi ③ hỏi có commit 2.8 riêng không. Đo 2026-08-17: 2.8 đã
vào commit `4d72cd4`. Cây **sạch**; hai thứ chưa commit đều là tạo tác của **chính 2.9** (tệp story
+ entry `sprint-status.yaml` do create-story sinh). ⇒ Câu hỏi ③ không còn việc.

**Baseline đo TRƯỚC khi chạm dòng đầu tiên**, trên HEAD `4d72cd4` — đo lại từ nguồn, **không chép
số của 2.8** *(story cấm bằng chữ, và số của 2.8 là ảnh chụp trên một cây chưa commit)*:

| Đường | Số | Khớp story? |
|---|---|---|
| `cargo test --locked` | **401** passed / 0 failed / 5 ignored | ✅ |
| `segment_contract.rs` | **121** | ✅ |
| `npx vitest run` | **141/141**, 13 tệp | ✅ |
| Chín cổng đọc-tệp | 9/9 exit 0 | ✅ |
| `COMMAND_FLOOR` | sàn **38** (`check-commands.mjs:259`) | ✅ |

#### Task 0.1 — năm mệnh đề của §0.1 đo lại trên cây HIỆN TẠI, cả năm VẪN ĐÚNG

| Phép đo | Kết quả 2026-08-17 |
|---|---|
| `grep -rniE "undo\|redo\|UndoManager" src/ src-tauri/src/` | **0 cơ chế.** 8 dòng trúng: 7 là chữ `dock`/`undock` của dockview, 1 là chú thích `GridPanel.vue:1064` |
| `grep -rn "KeyZ" src/commands/` | **0** |
| `grep -rniE "undo\|hoàn tác\|⌘Z" prd.md` | Trúng **duy nhất** chữ `undock` ở FR17 |
| `EXPERIENCE.md` bảng Phím `:261-268` | **Không hàng `⌘Z`.** Bảng **có** hàng *"`Backspace` đầu ô — Gộp với câu trên"* |
| `grep -c "^### AD-" ARCHITECTURE-SPINE.md` | **47** ⇒ `AD` kế tiếp là **48** |

🔵 Chữ *"hoàn tác bằng `⌘Z`"* **có** tồn tại — nhưng ở **văn xuôi** UX-DR32 (`EXPERIENCE.md:171`),
tức một **lời hứa**, không một mô hình. Phân biệt này là toàn bộ lý do Task 0 tồn tại.

#### Task 0.2 — hồ sơ bàn giao

`_bmad-output/planning-artifacts/ad-brief-2026-08-17-mo-hinh-hoan-tac.md`. Tám mục: hai đường
(A)/(B) kèm hệ quả trên đĩa · ba mức phạm vi ①/②/③ · tám ràng buộc cứng (AD-3 · AD-5 · AD-31 ·
AD-11 · AD-30 · AD-1 · AD-34 §1 · AD-35) · sáu điều kiện nghiệm thu · bảng 5/6 AC.

🔴 **Hai sự thật hồ sơ đào ra mà story chưa nêu:**
1. **Không đường ghi nào đưa `retired_at` về `NULL`.** Đo: `retired_at` **đọc** ở sáu chỗ, **đặt**
   ở đúng một (`write_regroup`), và `core/i18n/mod.rs:215` ghi bằng chữ *"chỉ đặt được bằng SQL"*.
   ⇒ Đường (A) đòi một **năng lực ghi chưa từng tồn tại**, không phải một lượt gọi lại hàm có sẵn.
2. **Đường (B) mất dữ liệu CHÍNH VÌ nó tuân thủ AD-5 hoàn hảo.** Một lượt `⌘Z` biến **một** chỗ
   đánh dấu FR119 thành **hai** chỗ mang ghi chú *"câu này đã đổi"* — cho một thao tác người dùng
   vừa **huỷ bỏ**. Một sự thật sai trên màn hình.
   ⇒ Cả hai đường đều làm một lời hứa của spine kém đúng đi. Đó là lý do nó là một `AD`.

#### Task 0.3 — ba chữ ký của Ice, 2026-08-17

| # | Câu hỏi | Ice ký | Hệ quả thi hành |
|---|---|---|---|
| **①** | Phạm vi 2.9 khi AC5 chưa có mô hình | **Giao 5/6 AC, ghi nợ AC5** | 🔵 **Cửa chặn Task 0.4 KHÔNG dừng story** — nó chỉ dừng **AC5**. Khác 2.7, nơi cửa chặn dừng cả story vì chữ ký #8(b) đụng một AC đang thi công |
| **②** | AC4 — *"ở lề"* hay thanh trạng thái | **Thanh trạng thái** | Mở khe thứ hai ở `StatusBar.vue`, **giữ tính ĐÓNG** của bảng tra. Không cột lề thứ sáu |
| **③** | Giữ `Backspace` liên tục | **Gộp một lần rồi dừng** | Chốt bằng `event.repeat`. Muốn gộp tiếp: nhả phím, bấm lại |

🔵 **Chữ ký ② giải một chỗ hai tài liệu nói hai điều**, và nó đứng về phía chữ ký cũ chứ không về
phía chữ. Ba bằng chứng đã trình: lưới có **đúng năm cột** (`GridPanel.vue:1391`) nên không có chỗ
cho cột lề thứ sáu; mọi mockup vẽ *"lề"* đều thuộc hình dạng **Editor liền mạch** đã bị UX-DR13 lật
2026-08-14; và `check:tokens` Kiểm F cấm **lớp nổi** nên dựng lại đúng mockup cũ là một lượt cổng đỏ.

🔴 **Chữ ký ③ đi ngược bản năng của một trình soạn thảo, có chủ ý.** Trong một trình soạn thảo
thường, giữ `Backspace` xoá liên tục là đúng. Ở đây mỗi lượt là một lượt **ghi xuống WAL** mà
**AD-5 không cho hoàn tác** — và AC5 (`⌘Z`) đang là món nợ. ~30 keydown/giây trên một thao tác
không lui được là mất ranh giới câu hàng loạt bằng một phím bấm hàng trăm lần mỗi Chương.

### Ⓑ Task 1 — bàn đo, và một lượt sửa THƯỚC không tính vào LUẬT DỪNG

Bàn đo ở `_bmad-output/implementation-artifacts/2-9-ban-do/`, hai vòng.
Môi trường: WebKit **605.1.15** (WKWebView thật của Tauri), macOS 24.6.0, `--features wdio` ·
rustc/cargo 1.97.1 · Node 22.22.2. Đo **2026-08-17**.

#### 🔵 Vòng 1 — BỎ SỐ, GIỮ BÀI HỌC

Vòng 1 gửi `Backspace` bằng `browser.keys()` và đo được, ở **cả năm** bước: `isTrusted: false` ·
`document.hasFocus(): false` · **0** `beforeinput` · **0** `input` · `textContent` không đổi.

🔴 **Đối chứng Ⓔ là chỗ bảng tự tố cáo:** caret ở **GIỮA** ô (`startOffset: 3`) — một lượt xoá
lui tầm thường nhất trần đời — cũng **không xoá gì**. ⇒ Con số *"0 `beforeinput` ở offset 0"*
trả lời câu *"một phím KHÔNG TIN CẬY trong một tài liệu KHÔNG CÓ TIÊU ĐIỂM làm được gì"*.

🔵 **Và giới hạn ấy ĐÃ CÓ CHỦ từ 2026-08-13** — `e2e/specs/editor-typing-flush.e2e.mjs:38-54`
ghi nguyên văn: *"`browser.keys()` KHÔNG GÕ ĐƯỢC CHỮ […] nó synthesize `keydown`/`keyup` và
không đi vào đường nhập văn bản gốc"*. Tôi **đo lại một thứ đã ghi** thay vì đọc nó.
⇒ Bài học phương pháp: trước khi dựng một bàn đo, `grep` chính giới hạn mình sắp gặp.

⚠️ **LUẬT DỪNG KHÔNG kích hoạt** — nó đếm những vòng mà một **giả thuyết về sản phẩm** bị phép
đo bác. Đây là một lượt sửa **thước**. Tiền lệ: `2-8-ban-do/` §Vòng 1 · `2-5d-ban-do/` §Ⓐ.

**Ba số của vòng 1 sống sót** *(phép đọc JS thuần, không phụ thuộc độ tin cậy)*, và cả ba là
tiền đề vận hành của nhánh: `key`/`code` = `"Backspace"` · `cancelable: true` · `repeat: false`.

#### 🔴 Vòng 2 — số quyết định của cả story

Caret đặt bằng `document.caretRangeFromPoint` *(API mà 2.8 đã đo là **có thật** trên WKWebView
này, khác `caretPositionFromPoint` vốn NÉM `TypeError`)*. Xác nhận lại tại chỗ: `"function"` vs
`"undefined"`. **3/3 passing** (1m 06s).

| Ca | `startContainer` | `startOffset` | `startOffset === 0`? | **ứng viên** | Đúng |
|---|---|---|---|---|---|
| ô **RỖNG** | `DIV` *(chính ô)* | 0 | ✅ true | **true** | đầu ô |
| một dòng, mép trái | `#text`(11) | 0 | ✅ true | **true** | đầu ô |
| một dòng, giữa chữ | `#text`(11) | 2 | false | **false** | không |
| hai dòng, mép trái dòng 1 | `#text`(3) | 0 | ✅ true | **true** | đầu ô |
| 🔴 **hai dòng, mép trái DÒNG 2** | **`#text`(3)** | **0** | 🔴 **true — SAI** | **false** | **không** |
| hai dòng, giữa chữ dòng 2 | `#text`(3) | 1 | false | **false** | không |
| 🔴 **vùng chọn TỪ đầu ô, không collapsed** | `#text` | 0 | 🔴 **true — SAI** | **false** | **không** |

**Ứng viên đúng 7/7. `startOffset === 0` sai 2/7.**

🔴 **Cạm bẫy ④ của story ĐÚNG, và số đo cho nó một cơ chế cụ thể:** dưới `white-space: pre-line`,
`insertLineBreak` của WebKit để lại **ba text node** — `"AAA"` · `"\n"` · `"BBB"`, **không** một
`<br>`. Engine đặt caret ở đầu dòng 2 vào **offset 0 của node THỨ BA**.

⚠️ **Và phép kiểm ngược lại cũng sai.** *"`startContainer` là con ĐẦU của ô"* hỏng ở ca ô rỗng
*(`startContainer` = chính ô, 0 con)*. Không phép kiểm nào **theo hình dạng** đúng cả bốn ⇒ phải
hỏi đúng câu định nghĩa: *"không còn ký tự nào phía trước caret trong cả ô"*, cài bằng một
`Range` từ `(cell, 0)` tới caret rồi đo `toString().length`.

🔴 **Tiền đề ① xác nhận lại trên cây hôm nay, CÓ ĐỐI CHỨNG DƯƠNG** — thứ bảng cũ thiếu:

| `execCommand('delete')` | trả về | `beforeinput` | `input` | `textContent` |
|---|---|---|---|---|
| caret **offset 0** | **`true`** | `[]` | `[]` | **không đổi** |
| **đối chứng dương**, offset 3 | `true` | `["deleteContentBackward"]` | idem | `"bốn năm sáu"` → `"bố năm sáu"` |

⇒ Thước hoạt động; con số ở offset 0 là mệnh đề về **engine**. `onBeforeInput` không dùng được
làm điểm móc — đường còn lại là `keydown`.
🔴 **Chi tiết đáng ghi riêng:** `execCommand('delete')` trả **`true`** trong khi KHÔNG làm gì.
Giá trị trả về nói *"lệnh được nhận"*, không *"lệnh có tác dụng"* — cùng lớp *"rỗng IM LẶNG"*.

### Ⓒ Ba quyết định cài đặt tôi tự chốt, và lý do từng cái

#### ① `dispatch('editor.merge_segments')`, không gọi thẳng `mergeCurrentSegment()`

Story viết *"gọi **chính** `mergeCurrentSegment()` mà `⌘M` gọi"*, và **ý đồ** của câu đó là
*"đừng viết một đường gộp thứ hai"*. `dispatch` thi hành ý đồ ấy **mạnh hơn**: nó dùng chung
đường **cao hơn một bậc**, tức cả lượt xử lý kết quả lẫn dòng chẩn đoán ở `main.ts:320-328`.
Một lời gọi thẳng phải chép lại khúc đó — và `project-context.md` ghi bằng chữ rằng *"một lời
gọi thẳng dựng một đường thứ hai mà `check:commands` KHÔNG nhìn thấy"*, cộng AD-34 §1.

⚠️ Đây **không** phải một command mới, và `COMMAND_FLOOR` **không đổi** (sàn 38). Cạm bẫy ② của
story cấm **đăng ký một hợp âm `Backspace`** *(`keys.ts:510` chặn mọi hợp âm không-mod trong
vùng gõ)*. Bắt trực tiếp ở `onEditKeydown` rồi `dispatch` một id **đã đăng ký** đi vòng qua bảng
hợp âm mà vẫn giữ **một** đường thao tác.

#### ② Một `Record` THỨ HAI, không nới `CONFIRM_NOTICE_KEYS`

Toàn bộ giá trị của bảng cũ là `vue-tsc` **đỏ** khi ai đó thêm một kết quả vào `ConfirmResult`
mà quên bảng. Nới nó thành `string` để nhét thêm câu của lượt gộp gỡ đúng cái chốt ấy, cho **cả
hai** lượt. ⇒ Hai ô nhớ, hai bảng, **cả hai đóng**; giá là một nhánh `v-else-if` nữa.

`'refused'` **cố ý vắng** khỏi bảng: lý do từ chối là câu chữ của **Rust**, ra màn hình qua
`tError()`. Chép nó thành khoá thứ bảy là dựng nguồn sự thật thứ hai.

#### ③ Một spec e2e RIÊNG, không nhét vào `segment-merge-split.e2e.mjs`

Đo được: mỗi lệnh WebDriver trả giá **~5 giây** *(service in `Tauri core.invoke not available
after 5s timeout` ở mỗi lệnh)*, nên chi phí một ca tỉ lệ **số lệnh**. Spec 2.8 đã có 3 ca; thêm
3 ca nữa vào đó là một tệp chắc chắn vượt trần và một lượt đỏ không nói gì về sản phẩm.

### Debug Log References

#### Ⓐ Một lượt đỏ ĐÚNG mà tôi tạo ra, và nó bắt chỗ tôi đọc sai sản phẩm

Ca e2e đối chứng ② ban đầu khẳng định `.not.toContain('Đã gộp')` và **đỏ** với nguyên văn
`Matcher error: received value must not be null nor undefined · Received has value: null`.

Nguyên nhân **không** phải một khuyết tật: lượt `execCommand('insertText')` ở bước trước phát
`input` → `onEditInput` → `reportEdit` → `noteEditorEdit`, và hàm đó **dọn** `regroupNotice`
*(Task 3.6, dọn bằng SỰ KIỆN)*. ⇒ Hai tính năng của chính story này giao nhau, và tôi chưa lường.
Sửa thành `toBe(null)` — một mệnh đề **mạnh hơn**: nó khẳng định cả *"không câu gộp nào"* lẫn
*"câu cũ đã được dọn"*, trong một phép so.

#### Ⓑ Bốn lượt tự kiểm ĐỎ-RỒI-XANH, đột biến MÃ SẢN PHẨM chứ không đột biến test

| Đột biến trên `caretAtCellStart` | Ca bắt được |
|---|---|
| `truoc.toString().length === 0` → `r.startOffset === 0` | ⑤ *(đầu dòng thứ hai)* — 1 đỏ / 9 xanh |
| gỡ chốt `if (!r.collapsed) return false` | ⑦ *(vùng chọn không collapsed)* — 1 đỏ / 9 xanh |
| gỡ chốt `if (!cell.contains(...))` | 🔴 **KHÔNG ca nào bắt được** — 10/10 xanh |
| gỡ chốt `contains`, **sau khi thêm ca ⑨b** | ⑨b — 1 đỏ / 10 xanh |

🔴 **Lượt thứ ba là một lỗ hổng thật trong bộ test của tôi, và đột biến tìm ra nó.** Ca ⑨ *(caret
ở một ô **SAU** trong cây)* sống sót **nhờ tình cờ**: `Range` từ `(a, 0)` tới một điểm trong `b`
đứng sau là một `Range` **xuôi**, `toString()` trả `"câu A"` nên phép đo dài ≠ 0 — chốt `contains`
không tham gia. Chiều ngược lại thì khác hẳn: theo DOM, `setEnd` với một biên **đứng trước**
`start` **thu `Range` về điểm cuối** ⇒ `toString()` rỗng ⇒ hàm trả `true` cho một caret **không
nằm trong ô**, tức một lượt gộp **sai câu**, im lặng. Ca ⑨b là bằng chứng của chốt đó.

#### Ⓓ Ba lượt e2e cho AC7, và HAI lượt đỏ đều là khuyết tật của BÀN ĐO

Ghi cả ba, vì hai lượt đỏ đầu dễ bị đọc thành hồi quy:

| Lượt | Kết quả | Nguyên văn | Là gì |
|---|---|---|---|
| ① | 2/3 | `Expected: 3 · Received: 2` | 🔴 **khuyết tật bàn đo tôi viết** |
| ② | 2/3 | `Vào workspace rồi mà không thấy [data-attribution-open] sau 30 giây` | chập chờn của **fixture**, đã có chủ |
| ③ | **3/3** (4m 06) | — | — |

🔴 **Lượt ① là lỗi của tôi, và bản vá không phải một `pause`:** `waitForExist('[data-col="src"]')`
**không phân biệt** *"Chương mới đã nạp"* với *"Chương CŨ còn nằm đó"*. Ca đầu gộp 3 hàng thành
2; ca sau dựng Tác phẩm mới rồi đọc ngay và thấy **2**. Vá bằng `doiLuoiCo(n)` — chờ **số hàng
mong đợi**. Đây là một **tiền đề** của ca, không phải mệnh đề đang kiểm, nên chờ nó hợp lệ;
khác hẳn việc nới một ngưỡng cho hết đỏ.

⚠️ Lượt ② đỏ **trong fixture** (`openWorkspaceWithWork` không thấy dải chip từ điển lúc app khởi
động nguội), không trong mã story. Khớp khuôn *"bộ e2e chập chờn"* đã có chủ. **Không chấm "đã
chẩn đoán"**.

#### Ⓔ Lượt đổi cử chỉ làm ĐỎ hai ca của spec 2.8 — sửa cùng lượt, có chủ ý

`segment-merge-split.e2e.mjs` bắn `MouseEvent('mouseup', …)` **trơn** ở hai chỗ để đặt chỗ cắt.
Sau AC7 chúng không đặt được gì ⇒ `⌘/` trả `'no-cut'`. Đã thêm `metaKey: true` vào cả hai kèm lý
do tại chỗ. **Nghiệm thu: spec 2.8 vẫn 3/3** (2m 36s).
🔴 Đây là hệ quả **bắt buộc** của lượt đổi, không một lựa chọn: để lại là một cổng đỏ mà người
sau phải đi chẩn đoán từ đầu.

#### Ⓒ Một lượt sửa cổng, không một miễn trừ

`check:lint` đỏ ở `StatusBar.vue`: `Unnecessary conditional … left-hand side of ??`. Tôi viết
`tError(err, err.params ?? undefined)`. Đọc nguồn: `tError` **đã** tự đọc `err.params` ở nhánh
cuối (`i18n/index.ts`, `t(key, params ?? err.params)`) — tham số thứ hai chỉ dành cho nơi gọi
muốn **ghi đè**. ⇒ Gỡ tham số thừa, không thêm một `eslint-disable`.

### Completion Notes List

#### ⓪ Phạm vi giao — NĂM trên sáu AC, và vế còn lại có chủ

| AC | Trạng thái | Đường nghiệm thu |
|---|---|---|
| **AC1** cử chỉ | ✅ | e2e `segment-backspace-merge` ca ① · vitest 11 ca cho `caretAtCellStart` |
| **AC2** kết quả hai vế | ✅ | **đã có chủ** ở `regroup.rs` + `segment_contract.rs` — không dựng lại |
| **AC3** ngữ nghĩa AD-5 | ✅ | idem, cộng `neither_merge_nor_split_ever_writes_a_segment_version_row` |
| **AC4** dòng báo hệ quả | ✅ | vitest 6 ca · e2e ca ① *(câu thành công)* và ca ② *(câu từ chối)* |
| **AC5** `⌘Z` | 🔴 **NỢ, có chủ** | chờ `AD-48`. Hồ sơ bàn giao đã soạn |
| **AC6** không chặn | ✅ | e2e ca ② — lưới không đổi, không hộp thoại, và lượt từ chối **nói ra** |
| **AC7** cử chỉ chỗ cắt | ✅ | vitest 6 ca *(hai nền tảng)* · e2e ca ③ *(đối chứng hai chiều)* |
| **AC8** `Esc` xoá tập | ✅ | vitest 5 ca, gồm ca `isComposing` · 2 lượt đột biết đỏ đúng chỗ |
| **AC9** chỗ cắt ở Hán Việt | 🟡 | bàn đo trên WKWebView: 17→**0** · 19→**2** · vitest 16 ca · chiều cao hàng **71→71px**. **Vế dấu cắt GIỮA từ ở `parallel` còn hở**, có chủ |

🔴 **Không vế nào tự chấm đạt bằng suy luận.** AC5 **không sai** vì đường đi chưa tới nó
*(`project-context.md` §"Năng lực chưa dựng ≠ lệch spec")*; `epics.md` **không bị sửa một chữ**.

#### ① Hai khoảng IM LẶNG đóng lại — và cả hai đã được ghi ra bằng chữ trước đó

- **Người dùng bấm `Backspace` ở đầu ô và không có gì xảy ra** — cử chỉ chưa tồn tại.
- **Người dùng bấm `Backspace` ở câu ĐẦU CHƯƠNG và không có gì xảy ra** — `merge_segments` từ
  chối bằng `segment.no_previous`, khoá i18n đã có từ 2.8, mà `editorRegroupError` **chưa
  component nào đọc**. Đây là ca **thường nhất** của cử chỉ: câu số 1 của mọi Chương.

#### ② Thứ story này KHÔNG làm, có chủ ý

**Không sửa một dòng Rust nào.** `cargo test --locked` giữ nguyên **401/0/5** và
`segment_contract` giữ nguyên **121** — đó là một mệnh đề nghiệm thu, không một sự tình cờ:
nghiệp vụ gộp đã đủ từ 2.8, và một con số Rust nhúc nhích ở đây nghĩa là tôi đã dựng một đường
gộp thứ hai. **Không bước di trú nào**; số kế tiếp vẫn là **12** *(đọc từ `PROJECT_MIGRATIONS`,
không từ dòng này)*. **`COMMAND_FLOOR` không đổi** (sàn 38) — story không thêm command nào.

#### ③ Ba mệnh đề KHÔNG đường nghiệm thu nào của dự án mô phỏng được — món cho Ice

Đo được (`2-9-ban-do/` §Vòng 1): **mọi** sự kiện driver giao đều `isTrusted: false`, và một sự
kiện không tin cậy **không có default action** *(đối chứng: caret giữa ô, `Backspace` qua driver
cũng không xoá một ký tự)*. ⇒ ① `preventDefault()` chặn nổi lượt xoá của một phím **thật**
không *(rủi ro thấp — ở offset 0 WebKit không có gì để xoá lui, `2-9-ban-do/` §Ⓓ)*; ② auto-repeat
của hệ điều hành *(chữ ký ③)*; ③ một lượt chốt của **bộ gõ tiếng Việt** không bị nhánh mới ăn mất.
🔴 **Chữ ký của Ice là đường nghiệm thu duy nhất** cho cả ba — cùng lớp với Task 1.4 của 2.5b.

#### ④ Nghiệm thu — mọi số đo lại từ nguồn, không chép của 2.8

| Đường | Baseline `4d72cd4` | Sau story | Δ |
|---|---|---|---|
| `cargo test --locked` | 401 / 0 / 5 | **401 / 0 / 5** | **0** *(có chủ ý — §②)* |
| `segment_contract.rs` | 121 | **121** | **0** |
| `npx vitest run` | 141/141, 13 tệp | **177/177, 17 tệp** | **+36** ca, +4 tệp |
| Chín cổng đọc-tệp | 9/9 | **9/9** | — |
| `check:scope` · `check:scope:bundled` | — | **exit 0 · exit 0** | *(chạy tay)* |
| `npm run build` · `vue-tsc --noEmit` | — | **xanh · xanh** | — |
| `COMMAND_FLOOR` | sàn 38 | **sàn 39** *(cổng in 47)* | +1 — `editor.clear_source_cuts` |
| e2e spec mới | — | **3/3 ca** (4m 06s) | — |
| e2e spec 2.8 *(sau lượt đổi cử chỉ)* | 3/3 | **3/3** (2m 36s) | 0 |

#### ⑤ e2e TRỌN BỘ — ba lượt, ghi cả ba

**Lượt ① trọn bộ:** **7/10 spec** *(14m 46s)*. Spec của story này **2/2 xanh** (0-5, 2m 28s).
Và `segment-merge-split.e2e.mjs` — spec canh đường gộp `⌘M`, tức đường **gần nhất** với thay đổi
của tôi — **3/3 xanh** (0-7, 3m 19s).

**Ba spec đỏ, nguyên văn bắt đủ trước khi kết luận** *(luật sau Story 1.22)*:

| Spec | Ca đỏ | Nguyên văn |
|---|---|---|
| `attribution-focus` | 1 | `expect(received).toBe(expected) · Expected: true · Received: false` |
| `editor-confirm-segment` | 2 | `Couldn't find element for "pointerMove" action sequence` |
| `editor-typing-flush` | 1 | `Couldn't find element for "pointerMove" action sequence` |

**Lượt ②③ chạy riêng từng spec:** **1/1 · 2/2 · 2/2 — tất cả XANH** (57s · 2m 06 · 2m 23).

⇒ Khớp đúng khuôn *"xanh riêng, đỏ trong bộ"* đã có chủ ở `deferred-work.md` *(ba mục:
`devServerIsUp` tin một Vite hấp hối · fixture không reset state panel · quan sát 2026-08-17 về
`core.invoke` của bàn đo không lên)*. Nguyên văn `Couldn't find element for "pointerMove"` là
**y hệt** thứ Story 2.8 đã ghi ở lượt ② của nó.

🔴 **Không chấm *"đã chẩn đoán"*.** Luật sau 1.22 đòi bắt nguyên văn trước, và tôi có đủ để nói
**"không do story này gây ra"** — ba bằng chứng: ① cả năm ca đỏ xanh khi chạy riêng; ② không spec
đỏ nào chạm bề mặt story này sửa; ③ spec canh chính đường gộp *(`segment-merge-split`)* xanh 3/3
**trong cùng lượt trọn bộ**. Tôi **không** có đủ để nói *"nguyên nhân là X"*.

### File List

**Sửa (UPDATE):**
- `src/panels/editorSegments.ts` — thêm `caretAtCellStart()`, `hasPrimaryModifier()`,
  `neoNguonCua()`; `sourceCutOffsetOf()` đổi từ **đếm mù** sang **đọc neo** (AC9)
- `src/panels/SourceHanViet.vue` — `srcStart` cho mọi segment + phép ánh xạ `\r\n`; neo
  `data-src-start`/`data-src-atomic` ra DOM; prop `cuts`; dấu cắt `::before` (AC9)
- `src/commands/index.ts` — dep `clearSourceCuts` + command `editor.clear_source_cuts` (AC8)
- `src/main.ts` — nối dep `clearSourceCuts` (AC8)
- `scripts/check-commands.mjs` — `COMMAND_FLOOR` 38 → 39
- `src/panels/GridPanel.vue` — `.cut-mark` cao 1,3em + `primary`; **gỡ** lớp và viền
  `has-cuts`; hai doc-comment đã hết đúng sửa tại chỗ (Task 9)
- `src/panels/SourceHanViet.vue` — `.cut-here::before` khớp `.cut-mark` (Task 9)
- `e2e/specs/segment-merge-split.e2e.mjs` · `e2e/specs/segment-backspace-merge.e2e.mjs` —
  đọc `data-cut-count` thay cho lớp `has-cuts` (Task 9)
- `src/panels/GridPanel.vue` — nhánh `Backspace` trong `onEditKeydown`; chốt `Mod` ở
  `onSourceCellMouseUp` (AC7); hằng `PLATFORM`; import `caretAtCellStart`/`hasPrimaryModifier`/`detectIsMac`
- `src/panels/editorPanelState.ts` — `RegroupNotice` + `regroupNotice` + `editorRegroupNotice`;
  đặt ô nhớ ở sáu chỗ; dọn ở `noteEditorEdit`; `regroup()` nhận `ten: 'gop' | 'tach'`
- `src/StatusBar.vue` — `REGROUP_NOTICE_KEYS` + `regroupNoticeText`; nhánh `v-else-if`
- `src/i18n/vi.json` — 6 khoá `panel.grid.regroup_*`
- `_bmad-output/implementation-artifacts/deferred-work.md` — 2 món đóng bằng cách nối tiếp, 5 món mới
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — trạng thái + ghi chép
- `_bmad-output/implementation-artifacts/2-9-gop-bang-backspace-dau-o.md` — tệp story này
- `e2e/specs/segment-merge-split.e2e.mjs` — hai lượt `mouseup` thêm `metaKey` (hệ quả của AC7)
- `tests/frontend/editorSourceCut.test.ts` — 6 ca đổi sang hình dạng neo, **giữ nguyên bài
  học**; cộng 8 ca mới cho AC9
- `_bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md` —
  văn xuôi `:169` + một hàng mới trong bảng Phím `:267`

**Mới (ADD):**
- `tests/frontend/editorCaretAtCellStart.test.ts` — 11 ca
- `tests/frontend/editorRegroupNotice.test.ts` — 6 ca
- `tests/frontend/editorSourceCutGesture.test.ts` — 6 ca, lái **cả hai** nền tảng
- `tests/frontend/editorClearSourceCuts.test.ts` — 5 ca (AC8)
- `_bmad-output/implementation-artifacts/2-9-ban-do/han-viet-cho-cat.e2e.mjs` — bàn đo AC9
- `_bmad-output/implementation-artifacts/2-9-ban-do/dau-cat-chieu-cao.e2e.mjs` — bàn đo Task 9.3
- `e2e/specs/segment-backspace-merge.e2e.mjs` — 3 ca
- `_bmad-output/planning-artifacts/ad-brief-2026-08-17-mo-hinh-hoan-tac.md` — hồ sơ bàn giao `AD-48`
- `_bmad-output/implementation-artifacts/2-9-ban-do/README.md`
- `_bmad-output/implementation-artifacts/2-9-ban-do/backspace-dau-o-wkwebview.e2e.mjs` — vòng 1
- `_bmad-output/implementation-artifacts/2-9-ban-do/caret-dau-o-vong2.e2e.mjs` — vòng 2

**Không sửa, có chủ ý:** `src-tauri/**` · `epics.md` · `prd.md` · `ARCHITECTURE-SPINE.md`.
⚠️ `EXPERIENCE.md` **có** sửa — nó là tài liệu UX mô tả cử chỉ, và luật *"khi một mệnh đề hết
đúng, SỬA TẠI CHỖ kèm ngày và lý do"* áp cho nó. `epics.md` thì **không**: AC7 là một cử chỉ mới
Ice ký, không một lượt sửa spec cho khớp mã.

---

## Câu hỏi cho Ice — chốt trước khi thi công

1. 🔴 **AC5 (`⌘Z`)** — Task 0 là một cửa chặn thật. Giao 5/6 AC ngay và ghi nợ `⌘Z` chờ
   `AD-48`, hay dừng story tới khi có `AD-48`? *(Khuyến nghị: giao 5/6 — cử chỉ + ngữ nghĩa
   + dòng báo đứng được một mình, và `AD-48` cần một lượt suy nghĩ riêng về `segment.id`.)*
2. **AC4 "ở lề"** — `epics.md`/`EXPERIENCE.md` nói *"lề"*; Quyết định #8 Ice ký nói *"thanh
   trạng thái"*. Lưới không có cột lề thứ sáu, và mọi mockup vẽ *"lề"* đều thuộc hình dạng
   Editor đã bị lật. Đi theo thanh trạng thái, hay dựng một bề mặt mới trong lưới?
3. **Cây bẩn** — 20 tệp sửa + 5 tệp chưa theo dõi, toàn bộ là Story 2.8 *(đã `done`)*, gồm cả
   `regroup.rs` mà story này đứng lên. Commit 2.8 riêng trước chứ?
4. **Giữ `Backspace`** — xoá lui liên tục tới offset 0 rồi **vẫn giữ phím**: gộp một lần rồi
   dừng *(chốt bằng `event.repeat`)*, hay để nó gộp tiếp? Story 1.21 đã có tiền lệ cờ
   `repeatable` cho lựa chọn ngược lại. Task 1.4 sẽ đo, nhưng phán quyết là của Ice —
   *"không cổng nào đỏ"* nếu chọn sai, và cái sai là **mất ranh giới câu hàng loạt**.

## Nhật ký sprint-status

Gỡ nguyên văn từ `sprint-status.yaml` ngày 2026-08-19: tệp đó giữ TRẠNG THÁI, nội dung story thuộc về tệp này. Không sửa một ký tự.

```
  # 2.9 DOI TEN 2026-08-14. Ten cu: 2-9-gop-ngam-khi-go-de-len-ranh-gioi. Tien de cu ("con
  # tro o dung vi tri ranh gioi giua hai segment") KHONG con ton tai trong luoi — ranh gioi
  # nay la ranh gioi HANG. Nam AC giu nguyen NGU NGHIA, chi doi cu chi kich hoat. Covers
  # FR78 khong doi. UX-DR32 cung da sua theo.
  # 2.9 CONTEXT DA SOAN 2026-08-17. Pham vi THAT chi ba viec: cu chi Backspace, dong bao he
  #   qua, va ⌘Z. Nghiep vu gop da du tu 2.8 — story nay KHONG sua mot dong Rust nao.
  #   🔴 AC5 (⌘Z) mang mot CUA CHAN: khong FR/AD/UX-DR nao chot mo hinh undo, va chon mot
  #   mo hinh la mot AD MOI (AD-48). Chu cua ⌘Z la Ice, khong phai story — deferred-work.md
  #   :4103-4109. Task 0 dung story va soan ho so ban giao, khong tu soan AD.
  #   ⚠️ Hai bay da do, ca hai cho XANH tren moi cong ma CHET trong tay nguoi dung:
  #     ① Backspace o offset 0 KHONG phat `beforeinput` tren WebKit (0 su kien, ca WKWebView
  #        lan Playwright-WebKit) — phai bat o `keydown`. Blink thi CO, nen ban dev xanh.
  #     ② Mot command `Backspace` tran qua CommandRegistry KHONG BAO GIO chay trong o ban
  #        dich — keys.ts:510 chan hop am khong-Mod trong vung go.
  # 🔵 2026-08-17 — 2.9 chuyen sang in-progress (dev-story). Task 0 la mot CUA CHAN THAT:
  #   AC5 (`⌘Z`) khong co mo hinh, va chon mot mo hinh la mot AD MOI (AD-48). Khuon nay DA
  #   KICH HOAT THAT mot lan o 2.7 (AD-47 giao Winston) — dev soan HO SO BAN GIAO, khong soan AD.
  #   🔵 DIEU KIEN KHOI HANH cua story HET DUNG: story viet "cay dang BAN, 20 tep sua + 5 tep
  #   chua theo doi, toan bo la Story 2.8". Do 2026-08-17: 2.8 DA VAO commit `4d72cd4`. Cay sach,
  #   hai thu chua commit deu la tao tac cua CHINH 2.9 (tep story + entry nay) => Cau hoi ③ cua
  #   story khong con viec.
  #   Baseline do TRUOC khi cham dong dau tien, tren HEAD 4d72cd4: cargo test --locked
  #   **401/0/5** · segment_contract **121** · vitest **141/141** (13 tep) · 9 cong doc-tep xanh ·
  #   COMMAND_FLOOR san **38**. KHOP moi so ghi trong story (story CAM chep so 2.8 — day la luot
  #   do lai tu nguon, khong phai luot chep).
  #   ✅ TASK 0.1 — ba phep grep cua §0.1 do lai tren cay HIEN TAI, ca ba VAN DUNG:
  #     - `undo|redo|UndoManager` tren src/ + src-tauri/src/: 8 dong trung, **0** co che —
  #       bay dong la chu "undock"/"dock" cua dockview, mot dong la chu thich GridPanel.vue:1064.
  #     - `KeyZ` trong src/commands/: **0**. Khong command `Mod+Z` nao.
  #     - prd.md `undo|hoan tac|⌘Z`: trung DUY NHAT chu "undock" o FR17. Khong FR nao dinh nghia
  #       mo hinh hoan tac.
  #     - EXPERIENCE.md bang Phim (:261-268): **khong hang `⌘Z`**. Chu "hoan tac bang ⌘Z" chi
  #       xuat hien trong VAN XUOI cua UX-DR32 (:171) — mot LOI HUA, khong mot mo hinh.
  #     - `grep -c "^### AD-"` = **47**. AD ke tiep la 48.
  # ✅ 2026-08-17 — 2.9 XONG, chuyen sang `review`. Moi task tick. **NAM tren sau AC** —
  #   AC5 (`⌘Z`) la mot mon no CO CHU theo chu ky ① cua Ice, khong mot ve bi bo quen.
  #   ✅ ICE KY BA quyet dinh: ① giao 5/6 AC + ghi no AC5 · ② dong bao o THANH TRANG THAI
  #   (khong cot le thu sau) · ③ giu Backspace GOP MOT LAN roi dung (chot bang `event.repeat`).
  #   🔴 CUA CHAN TASK 0.4 KICH HOAT MOT NUA, va Ice phan dinh no chi chan **AC5**, khong chan
  #   ca story — khac 2.7, noi chu ky #8(b) dung ca story vi no dung mot AC dang thi cong.
  #   Ho so ban giao AD-48 da soan: planning-artifacts/ad-brief-2026-08-17-mo-hinh-hoan-tac.md
  #   (hai duong (A)/(B) kem he qua tren dia · ba muc pham vi · tam rang buoc cung · sau dieu
  #   kien nghiem thu). Chu: Ice phan dinh -> Winston soan AD.
  #   Nghiem thu: 11 cong npm (9 doc-tep + check:scope + check:scope:bundled chay tay) · build ·
  #   vue-tsc · vitest **158/158** (15 tep) · cargo test --locked **401/0/5**.
  #   🔵 BA SO CO Y KHONG DOI, va do la MOT MENH DE nghiem thu chu khong mot su tinh co:
  #   cargo 401/0/5 · segment_contract 121 · COMMAND_FLOOR san 38. Story KHONG sua mot dong
  #   Rust nao (nghiep vu gop da du tu 2.8) va KHONG them mot command nao. Mot so Rust nhuc
  #   nhich o day nghia la da dung mot duong gop THU HAI. Buoc di tru ke tiep van la **12**.
  #   Baseline 141 vitest => +17 ca (11 cho `caretAtCellStart`, 6 cho kenh thong diep).
  #
  # 🔴 SO DO QUYET DINH CA STORY — ban do vong 2, 7 hinh dang tren WKWebView 605.1.15 that:
  #   phep kiem `startOffset === 0` SAI **2/7**. Ca lat no: caret o **dau DONG THU HAI** cua
  #   mot o co `\n` cung cho `startOffset === 0` — duoi `pre-line`, `insertLineBreak` cua WebKit
  #   de lai BA text node ("AAA" · "\n" · "BBB") va engine dat caret vao offset 0 cua node THU
  #   BA. Mot phep kiem hoi offset se GOP CAU khi nguoi dung chi muon xoa mot lan xuong dong —
  #   tren thao tac ma AD-5 khong cho hoan tac. Ca sai thu hai: vung chon TU dau o, khong
  #   collapsed. ⇒ Ung vien `caretAtCellStart` (Range tu (cell,0) toi caret, do `toString()
  #   .length`) dung **7/7**.
  #   ⚠️ Va phep kiem NGUOC LAI ("startContainer la con DAU cua o") cung sai, o ca o RONG —
  #   startContainer la CHINH O. Khong phep kiem nao theo HINH DANG dung ca bon.
  #
  # 🔵 VONG 1 CUA BAN DO: BO SO, GIU BAI HOC — thuoc hong, khong engine hong. `browser.keys()`
  #   giao `isTrusted: false` + `document.hasFocus(): false`, va doi chung Ⓔ (caret GIUA o,
  #   offset 3) cung KHONG xoa gi. Gioi han nay DA CO CHU tu 2026-08-13
  #   (`editor-typing-flush.e2e.mjs:38-54`) — toi do lai mot thu da ghi thay vi doc no.
  #   ⚠️ LUAT DUNG **KHONG** kich hoat: no dem nhung vong ma mot GIA THUYET VE SAN PHAM bi bac.
  #   Tien de ① (`beforeinput` o offset 0) xac nhan lai bang duong `execCommand('delete')`,
  #   CO doi chung duong: offset 0 => 0 su kien, textContent khong doi; offset 3 =>
  #   `deleteContentBackward` + "bon" -> "bo". 🔴 Va `execCommand('delete')` tra **`true`**
  #   trong khi KHONG lam gi — mot luot "thanh cong" khong co that.
  #
  # 🔵 MOT QUYET DINH CAI DAT DI XA HON CHU CUA STORY, ghi ra thay vi im: story viet "goi CHINH
  #   `mergeCurrentSegment()`", da cai bang `dispatch('editor.merge_segments')`. Y do cua cau do
  #   la "dung viet mot duong gop thu hai", va `dispatch` thi hanh no MANH HON — dung chung
  #   duong cao hon mot bac (ca luot xu ly ket qua lan dong chan doan o main.ts:320-328), dung
  #   AD-34 §1 + project-context ("mot loi goi thang dung mot duong thu hai ma check:commands
  #   KHONG nhin thay"). COMMAND_FLOOR khong doi — day KHONG phai mot command moi.
  #
  # 🔴 MOT LO HONG TRONG BO TEST CUA CHINH TOI, tim ra bang DOT BIEN MA SAN PHAM: go chot
  #   `cell.contains(...)` ma 10/10 ca van XANH. Ca ⑨ song sot NHO TINH CO — Range tu (a,0) toi
  #   mot diem trong `b` dung SAU la mot Range XUOI nen toString() ra "cau A", chot khong tham
  #   gia. Chieu nguoc lai thi `setEnd` voi mot bien dung TRUOC start THU Range ve diem cuoi =>
  #   toString() rong => ham tra `true` cho mot caret NGOAI o, tuc gop SAI CAU im lang. Da them
  #   ca ⑨b; bon luot dot bien, moi luot do dung mot ca.
  #
  # 🔵 MOT LUOT DO **DUNG** ma chinh toi tao ra, va no bat cho toi doc sai san pham: ca e2e doi
  #   chung ② khang dinh `.not.toContain('Da gop')` va do voi "Received has value: null". Ly do
  #   khong phai khuyet tat — luot `execCommand('insertText')` truoc do phat `input` ->
  #   `noteEditorEdit`, va ham do DON `regroupNotice` (Task 3.6). Hai tinh nang cua chinh story
  #   giao nhau, toi chua luong. Sua thanh `toBe(null)` — menh de MANH HON.
  #
  # 🔴 E2E TRON BO: **7/10 spec** (14m46). Spec cua story 2/2 xanh; `segment-merge-split` (duong
  #   `⌘M`, gan nhat voi thay doi) **3/3 xanh trong cung luot do**. Ba spec do —
  #   `attribution-focus` (Expected true/Received false) · `editor-confirm-segment` va
  #   `editor-typing-flush` (ca hai: "Couldn't find element for pointerMove") — CHAY RIENG deu
  #   XANH: **1/1 · 2/2 · 2/2**. Khop khuon "xanh rieng, do trong bo" da co chu (ba muc o
  #   deferred-work.md), va nguyen van `pointerMove` la Y HET thu Story 2.8 ghi o luot ② cua no.
  #   ⚠️ KHONG cham "da chan doan": du de noi "khong do story nay gay ra", KHONG du de noi
  #   "nguyen nhan la X".
  #
  # 🟡 BA MENH DE KHONG DUONG NGHIEM THU NAO CUA DU AN MO PHONG DUOC, chu Ice, ghi rieng vi
  #   chu ky cua Ice LA duong nghiem thu duy nhat: ① `preventDefault()` co chan noi luot xoa cua
  #   mot phim THAT khong (rui ro THAP — o offset 0 WebKit khong co gi de xoa lui, da do);
  #   ② auto-repeat cua he dieu hanh (chu ky ③ — driver giu phim 600ms cho DUNG MOT keydown,
  #   `repeat: false`); ③ mot luot chot cua BO GO TIENG VIET khong bi nhanh moi an mat.
  #   Ly do chung: MOI su kien driver giao deu `isTrusted: false`, va mot su kien khong tin cay
  #   KHONG co default action (doi chung: caret giua o, Backspace qua driver cung khong xoa gi).
  #
  # 🔵 Nam mon no moi + hai mon dong bang cach NOI TIEP (khong xoa muc goc) vao deferred-work.md.
  #   Trong do mon ":3821-3829" (`restore_segment_version` khi van ban rong) RA LAI va KHONG
  #   cham: `regroup.rs:121` loc bo manh rong truoc khi noi, nen story chi doi CACH GOI chu
  #   khong doi CAI DUOC GHI. Giu nguyen chu Story 2.8.
  # 🔴 MON MOI dang chu y nhat: `@keydown` NAY MANG MOT THAO TAC THAT, va chinh cong da dan
  #   truoc ngay nay — `check-commands.mjs:2348-2349` in ra moi luot chay: "ngay mot @keydown
  #   mang thao tac that xuat hien, luat phai duoc xem lai". Chu: mot story ha tang cong / Ice.
  #
  # 🔵 2026-08-17, SAU LUOT DEV — **AC7 THEM VAO, Ice ky**, va no LAT mot cu chi cua Story 2.8
  #   (da `done`). Ice chot ghi vao 2.9 chu khong tach commit: 2.9 CHINH LA story ve CU CHI, va
  #   hai cu chi cua cung mot luoi doc duoc nhu mot dien.
  #   VAN DE ICE TIM RA BANG CACH **DUNG THAT** (khuon da lap o 2.5b va 2.8): cot nguyen van
  #   mang HAI cu chi chuot treo tren CUNG MOT `mouseup` — danh dau cho cat (2.8) va Auto-Lookup
  #   (FR21, Story 1.18, DA PHAT HANH). => Moi luot tra mot tu de DOC cung roi mot dau cat; va
  #   mot cu **double-click ban HAI `mouseup`** nen no de lai **hai** dau — cho mot luot `⌘/`
  #   nguoi dung khong dinh goi, tren du lieu ma AD-5 khong cho hoan tac.
  #   ⇒ Ban va: `Mod`+click danh dau; bam don/double-click TRON khong danh dau gi.
  #   🔴 `hasPrimaryModifier`, KHONG `event.metaKey` tran — §Trap 1 cua keys.ts, va
  #   commands/README.md:73 cam bang chu. Tren macOS `Ctrl`+click con la cu bam phu cua HE DIEU
  #   HANH, nen nhanh isMac hoi DUNG `metaKey`. Nen tang di vao qua THAM SO; nua Windows cua kho
  #   khong co duong nghiem thu tai cho (action item A5) nen vitest la luoi duy nhat lai hai ca.
  #   Bon luot dot bien MA SAN PHAM, moi luot do dung cho.
  #
  # ✅ **MOT MON NO 🔴 TO HON STORY NAY DA DONG, bang chu ky cua Ice** — deferred-work.md:4100
  #   ghi "Auto-Lookup bang chuot o cot nguon CHUA CO duong nghiem thu, VA CO THE DANG CHET":
  #   qua driver, KHONG cu chi chuot nao tao duoc vung chon o cot do. Muc do khai bang chu rang
  #   "mot luot boi den bang tay cua Ice dong duoc ve 'san pham co hong khong'", va ung vien con
  #   lai ("driver khong lai duoc may chon van ban cua WebKit") KHONG loai tru duoc bang chinh
  #   driver do, THEO CAU TAO. Ice xac nhan 2026-08-17: **double-click TRA DUOC** tren may that.
  #   ⇒ **FR21 SONG**, va ve kia la gioi han cua BO DO chu khong cua san pham.
  #   🟡 Ve "chua co duong nghiem thu" thi CON MO, giu nguyen chu (story ha tang e2e).
  #
  # 🔴 LUOT DOI CU CHI LAM DO HAI CA CUA SPEC 2.8 (chung ban `mouseup` TRON) — da sua cung luot
  #   bang `metaKey: true` kem ly do tai cho. Nghiem thu: spec 2.8 van **3/3** (2m36).
  #   De lai la mot cong do ma nguoi sau phai di chan doan tu dau.
  #
  # 🔴 BAN DO CUA CHINH TOI MANG MOT KHUYET TAT, ghi vi bai hoc dang hon ban va:
  #   `waitForExist('[data-col="src"]')` KHONG phan biet "Chuong moi da nap" voi "Chuong CU con
  #   nam do". Ca dau gop 3 hang thanh 2; ca sau dung Tac pham moi roi doc ngay va thay **2**
  #   (nguyen van: `Expected: 3, Received: 2`). Va bang `doiLuoiCo(n)` — cho SO HANG mong doi,
  #   khong cho "ton tai". Day la mot TIEN DE cua ca, khong menh de dang kiem.
  #   ⚠️ Khuon `waitForExist` roi doc ngay CON NGUYEN o cac spec khac — mot ung vien CHUA AI XET
  #   cho mon "bo e2e chap chon". Chu: story ha tang e2e.
  #
  # ⚠️ NGHIEM THU LAI SAU AC7: vitest **164/164** (16 tep, +6 ca) · cargo **401/0/5** (KHONG
  #   doi — Rust khong cham mot dong) · build · vue-tsc · 9 cong doc-tep · check:scope +
  #   check:scope:bundled · e2e spec 2.9 **3/3** (4m06, luot thu ba; hai luot truoc do deu do
  #   vi khuyet tat BAN DO, ghi day o §Debug Log Ⓓ) · e2e spec 2.8 **3/3**.
  #   Tai lieu sua tai cho kem 🔵 + ngay: EXPERIENCE.md:169 (van xuoi) va :267 (mot hang moi
  #   trong bang Phim). `epics.md` KHONG sua — AC7 la mot cu chi MOI Ice ky, khong mot luot sua
  #   spec cho khop ma.
  # 🔵 Ba mon no moi: cu chi chuot cua luoi khong cong nao canh (nay da co BA) · `PLATFORM` cua
  #   GridPanel khong tiem duoc · khuon `waitForExist` o cac spec khac. Moi mon mot chu.
  #
  # 🔵 2026-08-17, LUOT THU HAI SAU KHI ICE DUNG THAT — **AC8 va AC9 THEM VAO**.
  #   AC8 (`Esc` xoa tap diem cat): hai cua, MOT command. `Escape` khong mang `Mod` nen
  #   keys.ts:510 CHAN no trong vung go — dung cho nguoi dung dang dung sau khi vua go. Cua
  #   thu hai o `onEditKeydown` dispatch CHINH id da dang ky. COMMAND_FLOOR 38 -> **39**
  #   (cong in 47). KHONG `preventDefault()`: `Esc` con la phim dong cua cac lop phu.
  #
  # 🔴 AC9 — MOT LO HONG DU LIEU IM LANG, to hon trieu chung Ice bao. Ice noi "chua thay diem
  #   cat, va chua cat duoc"; ban do tren WKWebView that (`京都春風。`, **5 ky tu**) cho:
  #     tab nguyen van (doi chung): 5 · Han Viet `switch`: **17** · `parallel`: **19**
  #   Nguyen nhan lon nhat KHONG nam trong ba gia thuyet ban dau: dong `Nguon: thieu-chuu`
  #   (`.hv-sources`, 17 ky tu) nam TRONG o va bi phep dem mu cong vao.
  #   🔴 Hom nay no chua hong im lang **chi vi MAY** — hai so tinh co vuot bien mot cau 5 chu
  #   nen Rust tu choi. Tren mot cau Chuong that (40-60 chu), `19` nam TRONG BIEN va `⌘/` cat
  #   SAI CHO, IM LANG, tren du lieu ma AD-5 khong cho hoan tac.
  #   ✅ Da va: phep DEM MU thay bang DOC NEO `data-src-start`; khong neo => `null` (tu choi),
  #   nen `.hv-sources` · `.hv-notice` · `<rt>` roi vao do theo **CAU TAO**, khong nho mot danh
  #   sach loai tru phai bao tri. Do lai cung ban do sau va: `switch` **0** · `parallel` **2**.
  #   Ice ky: cat theo RANH GIOI TU o `switch` (chu Han goc khong co tren man hinh), chinh xac
  #   tung chu o `parallel`.
  #
  # 🔴 DAU CAT VE BANG `::before`, KHONG mot phan tu con — mot RANG BUOC, khong mot lua chon:
  #   `resolveSwitch()` anh xa nguoc bang CHI SO (`host.children[i]` <-> `segments[i]`), va
  #   doc-comment cua template ghi thang "them/bot/doi thu tu mot phan tu o day la lam truy van
  #   tra cuu SAI IM LANG". Mot dau cat bang text node thi di vao `Selection.toString()` cua
  #   Auto-Lookup. Hai duong re deu pha mot thu dang chay.
  #   🟡 HE QUA CON HO: mot cho cat nam GIUA mot tu o kieu `parallel` khong ve duoc dau. Phep
  #   anh xa lam dung (do: offset 2 trong base `京都`), chi cai dau la khong bam duoc vao dau.
  #   Mon no co chu **Ice** — chon giua (a) cho `.hv-unit` nguyen khoi luon, hay (b) giu do
  #   chinh xac va nhan mot dau cat vo hinh o ca giua tu.
  #
  # 🔴 BAI HOC MOI VE BAN DO: **mot ban do CHEP ham san pham se do BAN CHEP, va ban chep cu di.**
  #   Sau luot va AC9, ban do chay lai VAN cho 17 va 19 y het luot truoc, trong khi DOM da mang
  #   neo (`neoVao: "src-piece"` chung minh). No bao "chua va" tren mot san pham DA VA. Cung ho
  #   voi "mot con so THAT, tra loi SAI cau hoi" ma 2-5d-ban-do da dat ten, o mot co che moi.
  #
  # 🔵 MOT KHUYET TAT CUA THUOC vitest, cung khuon "xanh rieng, do trong bo" nhung o vitest:
  #   `attachTo: document.body` + `vi.resetModules()` moi ca de lai NHIEU GridPanel chong nhau,
  #   va `querySelector` bat duoc o cua lan mount CU NHAT — handler cua no tro vao mot module
  #   instance khac. Do duoc: `.col-tgt` = **3**, `wrapper.element.contains(o)` = **false**.
  #   Da va bang `afterEach` unmount; ca do them mot chot kiem chinh dieu do.
  #
  # ⚠️ NGHIEM THU LAI: **11 cong npm** · build · vue-tsc · vitest **177/177** (17 tep) ·
  #   cargo test --locked **401/0/5** (Rust VAN khong cham mot dong) · e2e spec 2.9 **3/3** va
  #   spec 2.8 **3/3** (chay chung mot luot).
  #   ⚠️ `check:scope` do MOT luot voi "Port 1420 is already in use" — mot Vite sot lai tu luot
  #   e2e. Cong TU KHAI dung ("Self-check chua chay toi noi... dung coi day la 'dat'"), khong
  #   bao dat oan. Don cong roi chay lai: exit 0.
  # 🔵 Nam mon no moi vao deferred-work.md, moi mon mot chu. Mot chu thich o GridPanel.vue da
  #   het dung ("kenh duy nhat o che do Han Viet, noi dau cat khong ve duoc") — sua tai cho.
  #
  # 🔵 2026-08-17, LUOT THU BA SAU KHI ICE NHIN MAN HINH THAT — hai viec, ca hai la THAM MY
  #   nhung mot cai co RUI RO BO CUC phai do:
  #   ① Dau cat cao `1em` -> **`1,3em`**, mau `ornament` -> **`primary`**. `ornament` la
  #      `#a9a196`/`#6a6459` va do duoc **2,44/2,64** tren `surface` — mo den muc check:tokens
  #      CAM no lam mau chu. Mot dau cat la thu nguoi dung phai TIM THAY.
  #      ⚠️ KHONG dung `error`: mot cho cat chua thuc hien khong phai mot loi, va nhuom do no
  #      la day mot muc khan cap se phai lam phat o story sau (cung ly le StatusBar da ghi).
  #   ② **GO vien `has-cuts` VA go luon chinh lop do.** Ice: *"bo dau gach dung o truoc cau
  #      di, no khong can thiet"*. No la kenh THAY THE cho ngay dau cat chua ve duoc o tab Han
  #      Viet; AC9 lam no ve duoc => kenh het viec. Giu mot lop khong tao kieu gi chi de e2e
  #      chon duoc la mo dung tien le kho nay CO Y chua mo. Hai spec doc no chuyen sang
  #      `data-cut-count` (cho mot SO, chat hon mot co).
  #      🔵 Day chinh la cau hoi tham my toi ghi vao so no o luot truoc, Ice vua tra loi.
  #
  # 🔴 MOT RUI RO TOI TU NEU RA VA DO, khong doan: `subgrid` lam mot phan tu inline CAO HON
  #   line box day CA TRACK HANG va keo o ban dich theo — cai gia do da do mot lan o 2.5b
  #   (hang Han Viet song song 388px). Ban do rieng (`2-9-ban-do/dau-cat-chieu-cao.e2e.mjs`)
  #   do chieu cao TRUOC/SAU khi dat mot diem cat: **71px -> 71px**, chenh **0/0**, va dau cat
  #   CO mat (soDauCat 1 — khong thi phep do la mot phep do ve hu khong). => `1,3em` an toan.
  #
  # ⚠️ MON NO MOI: hai khoi CSS cua dau cat (`GridPanel.vue::.cut-mark` va
  #   `SourceHanViet.vue::.cut-here::before`) ve CUNG mot khai niem bang HAI khoi roi — mot cai
  #   la phan tu, cai kia la pseudo-element, va SourceHanViet co `<style scoped>` rieng. Luot
  #   nay sua ca hai va ghi canh bao o ca hai dau, nhung mot luot sau van co the sua mot nua:
  #   dau cat doi hinh khi bat tab Han Viet, va KHONG phep kiem nao do. Chu: story ha tang cong.
  #
  # ⚠️ NGHIEM THU LAI: 11 cong npm · build · vue-tsc · vitest **177/177** · e2e spec 2.9 **3/3**
  #   va spec 2.8 **3/3** (chay chung mot luot, sau khi go lop `has-cuts`).
  #   ⚠️ Ban do chieu cao do MOT luot voi "khong thay [data-attribution-open] sau 30 giay" —
  #   chap chon cua FIXTURE luc app khoi dong nguoi, nguyen van y het luot da ghi. Chay lai: xanh.
  #
  # ✅ 2026-08-17 — code review BA TANG (Blind Hunter · Edge Case Hunter · Acceptance Auditor,
  #   claude-opus-5, moc diff `4d72cd4`): **2 quyet dinh Ice ky + 5 patch DA VA + 1 ghi no**.
  #   Story chuyen `review` -> `done`.
  #
  # 🔵 Tang Acceptance Auditor DO LAI tu nguon moi con so cua Dev Agent Record thay vi tin bang:
  #   vitest 177/177 · COMMAND_FLOOR 39 / 47 command · 6 khoa i18n · 11/6/6/5/16 ca moi tep ·
  #   `src-tauri/**` khong doi mot byte · vue-tsc sach. **Khop 100%.** Ca sau cam bay ①-⑥ tuan
  #   dung; quyet dinh Ⓒ① (`dispatch` thay vi goi thang) xac minh bang ma — khong duong gop thu hai.
  #
  # 🔴 NAM PATCH, va ca nam cung MOT LOP: mot o nho MOI thieu mot cua ma tep do DA CO SAN cua
  #   cho o nho CU. Ba lan trong cung mot tep, va ca ba di qua tron 177 ca cong muoi mot cong.
  #   ① `confirmNotice` DE `regroupNotice` => AC4 khong dat trong mot ca that (⌘Enter hut roi
  #      Backspace gop THANH CONG, thanh van hien cau cu). HAI tang doc lap tai hien bang vitest.
  #      Chu thich tai cho tu khai *"hai o nho khong bao gio cung mang gia tri"* — SAI. Va bay gio
  #      no dung theo CAU TAO: bat bien `ghiRegroupNotice` — *thao tac vua xay ra so huu thanh
  #      trang thai*, ai ghi mot o thi don o con lai. Ve doi xung o cua co khoa cua confirm.
  #   ② `resetEditorPanel()` bo sot `regroupNotice`/`regroupError` — roi sang Tac pham khac. Chinh
  #      khoi chu thich TREN ham do viet ra bang chu cai luat bi vi pham.
  #   ③ Khong khoa `regroupInFlight`. `event.repeat` chi chan auto-repeat cua HE DIEU HANH, khong
  #      chan hai cu bam ROI RAC nhanh. Luot IPC thu hai tra `refused` va GHI DE `'merged'` =>
  #      bao "chua gop duoc" cho mot thao tac DA GOP XONG. Them `'busy'`: TU CHOI va KEU, khong
  #      NHAP vao luot dang bay (khac `confirmInFlight` co chu y — `regroup` la cua chung HAI lenh,
  #      cho luot sau nhan ket qua luot truoc la danh roi mot thao tac nguoi dung trong im lang).
  #   ④ Cho cat GIUA mot manh `.hv-text` co hieu luc ma KHONG ve duoc dau — rong hon mon no da ghi
  #      (mon cu chi phu ca "giua mot TU HAN o `parallel`"). Ice ky: TU CHOI offset giua manh.
  #   ⑤ `originalOffsets` tach sang `editorSegments.ts` + 8 ca vitest (`\r\n` · `\r` tran · ngoai
  #      BMP) — no la con so ma moi cho cat sai im lang di ra tu, va khong luoi nao canh.
  #
  # 🔴 MOT LO HONG TRONG LUOI CUA CHINH LUOT REVIEW, va dot bien tim ra no: go
  #   `data-src-atomic` khoi `.hv-text` => **191/191 VAN XANH**. `editorSourceCut.test.ts` dung DOM
  #   bang tay roi tu gan neo, nen no canh `sourceCutOffsetOf` TON TRONG neo ma khong canh
  #   component PHAT RA neo. Dong bang `hanVietCutAnchors.test.ts` (mount component that, 8 ca);
  #   dot bien lap lai: 2 do, ca hai kieu xem. Ca nam ban va deu nghiem thu bang DOT BIEN MA SAN
  #   PHAM, khong bang mot luot chay xanh.
  #
  # ⚠️ MOT VE ICE KY KHONG PHU TRON NHU CAU DE XUAT, va da thu hep thay vi thi hanh qua tay:
  #   bat bien *"moi offset tao duoc deu ve duoc dau"* doi neo ca `.hv-unit` (base `<ruby>` o
  #   `parallel`) — nhung lam the VI PHAM AC9, von doi bang chu *"chinh xac tung chu o parallel"*.
  #   => Chi `.hv-text` mang neo; ca "giua mot TU HAN o parallel" VAN la mon no da ghi co chu, va
  #   mot ca test giu cho no khong bi lang le dong bang mot dong thuoc tinh.
  #
  # ⚠️ NGHIEM THU SAU LUOT VA (do lai tu nguon): vitest **199/199, 19 tep** (+22 ca, +2 tep) ·
  #   9/9 cong doc-tep exit 0 · check:scope + check:scope:bundled exit 0 · build + vue-tsc exit 0 ·
  #   COMMAND_FLOOR san 39 khong doi · `src-tauri/**` git diff RONG (=> cargo 401/0/5 dung theo cau
  #   tao). e2e chay lai HAI spec dung duong bi cham: `segment-backspace-merge` **3/3** (3m43) va
  #   `segment-merge-split` **3/3** (3m22), WKWebView 605.1.15 that.
  #   🔵 Va PHAI DO chu khong suy luan: `grep -ln "hv-\|han-viet" e2e/specs/*.mjs` => RONG, khong
  #   spec nao cham tab Han Viet, nen va ④ khong dung duong e2e nao.
  #   ⚠️ CHUA chay e2e TRON BO (~15 phut, can may ranh) — hai spec tren la hai spec duy nhat cham
  #   be mat luot va sua.
  #
  # ⚠️ MON GHI NO (defer, tien-ton-tai): `sourceCut` (Story 2.8) CUNG khong duoc don o
  #   `resetEditorPanel()`. Quan sat rong hon moi la mon that: luat *"moi o nho moi phai qua
  #   resetEditorPanel()"* KHONG CO CONG NAO CANH, va da bi bo sot HAI story lien tiep. Chu: mot
  #   story ha tang cong (dem `ref` cap module va doi chieu voi than `resetEditorPanel`).
  # 🔵 2026-08-18 — CUA CHAN AD-48 DA PHAN GIAI, VA KHONG BANG MOT MO HINH. Ice ky duong (C):
  #   RUT ⌘Z cho gop/tach. AC5 rut khoi epics.md (SCP 2026-08-18b). Moi menh de o cac khoi log
  #   phia tren noi "chon mot mo hinh la mot AD MOI" nay la BAN GHI LICH SU — huong da doi.
  #   Bon phep do chong lung: ① grep "undo|hoan tac|⌘Z" tren prd.md chi trung `undock` => KHONG
  #   FR nao doi hoan tac; ② EXPERIENCE.md:169 (Ice ky 2026-08-17) da viet "tren du lieu ma AD-5
  #   khong cho hoan tac"; ③ dong bao dang chay (vi.json:101) noi dung he qua va KHONG hua hoan
  #   tac; ④ cho duy nhat hua ⌘Z la EXPERIENCE.md:171 — va no la MAU SOT cua chinh luot sua
  #   2026-08-17 o dong ay (khuon F1 cua retro Epic 2: chu ky thi hanh dung mot nua).
  #   Tien de "tu gop/tach lai duoc" da kiem va DUNG: gop NOI target_text (regroup.rs:186), lich
  #   su hai cau cu VAN DOC DUOC (lib.rs:346) => khong byte nao bi pha, mat CONG SUC THAO TAC.
  #   ⇒ KHONG con can: ngoai le AD-3 · duong DELETE dau tien tren noi dung nguoi dung · nang luc
  #   retired_at -> NULL · bang nhat ky · buoc di tru 12. AD-3/AD-5/AD-31 khong doi mot chu.
  #   🔴 CON HO, ca hai ghi o deferred-work.md khoi "SCP 2026-08-18b": (a) bam ⌘Z KHONG phan hoi
  #   gi — rong IM LANG, chu la Ice; (b) AD-48 VAN phai soan nhung ban NHO (mot menh de), chu la
  #   Winston — dev khong tu soan AD. Status `done` cua 2-9 KHONG doi: AC5 nay la DA RUT, khong
  #   phai chua dat.
```
