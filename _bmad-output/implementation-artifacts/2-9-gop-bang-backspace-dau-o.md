---
baseline_commit: 4d72cd44c95c4dac132833d96097c01bccc061fa
---

# Story 2.9: Gộp bằng `Backspace` ở đầu ô

Status: review

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

### Task 5 — Sổ nợ
- [x] 5.1 Đóng món `deferred-work.md:3036-3061` *(tiền đề `beforeinput` đã lật)* bằng `→ ✅ ĐÃ ĐÓNG <ngày> (Story 2.9)` — **nối thêm, không xoá mục gốc**
- [x] 5.2 Đóng vế *"dòng báo"* của `:4103-4109`; vế `⌘Z` thì **giữ 🟡** và ghi chủ theo phán định Task 0
- [x] 5.3 Ghi món *"`@keydown` nay mang một thao tác thật ⇒ luật Kiểm A phải được xem lại"* — chủ theo phán định của Ice *(xem cạm bẫy ②)*
- [x] 5.4 Rà lại `deferred-work.md:3821-3829` *(`restore_segment_version` khi văn bản rỗng)* — còn hở không, và story này có chạm không — **KHÔNG chạm**, lý do ghi tại chỗ
- [x] 5.5 Ghi mọi món mới phát sinh, **mỗi món một chủ**, không món nào mồ côi — **5 món mới**, 2 món đóng bằng cách nối tiếp

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
| `npx vitest run` | 141/141, 13 tệp | **164/164, 16 tệp** | **+23** ca, +3 tệp |
| Chín cổng đọc-tệp | 9/9 | **9/9** | — |
| `check:scope` · `check:scope:bundled` | — | **exit 0 · exit 0** | *(chạy tay)* |
| `npm run build` · `vue-tsc --noEmit` | — | **xanh · xanh** | — |
| `COMMAND_FLOOR` | sàn 38 | **sàn 38** | **0** |
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
- `src/panels/editorSegments.ts` — thêm `caretAtCellStart()` và `hasPrimaryModifier()`
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
- `_bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md` —
  văn xuôi `:169` + một hàng mới trong bảng Phím `:267`

**Mới (ADD):**
- `tests/frontend/editorCaretAtCellStart.test.ts` — 11 ca
- `tests/frontend/editorRegroupNotice.test.ts` — 6 ca
- `tests/frontend/editorSourceCutGesture.test.ts` — 6 ca, lái **cả hai** nền tảng
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
