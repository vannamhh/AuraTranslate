---
baseline_commit: 4136f3f
---

# Story 1.18: Auto-Lookup

Status: done

> 🔴 **STORY NÀY DỰNG MỘT TRỪU TƯỢNG CHO BỐN PANEL, KHÔNG PHẢI MỘT TÍNH NĂNG CHO MỘT
> PANEL.** `epics.md:1762` nói nguyên văn: cơ chế Auto-Lookup *"gắn vào một **hợp đồng vùng
> chọn dùng chung cho mọi panel văn bản**"*, và AI Translation + Editor *"nhận được cùng hành
> vi khi chúng có nội dung ở các epic sau, **không cần cài lại**"*. Một cài đặt chỉ chạy
> cho `SourcePanel.vue` **đạt AC1 và trượt AC2** — và không cổng nào hôm nay bắt được
> khác biệt đó. **AC2 phải nghiệm thu bằng MÁY**, xem AC2 và Task 3.
>
> 🔴 **VÀ NÓ MANG MỘT MÓN NỢ ĐÃ CÓ CHỦ: bôi đen bằng BÀN PHÍM CHƯA TỒN TẠI.**
> `deferred-work.md:608` — *"Ice chốt 2026-08-06 ở lượt code review: **ghi nợ cho 1.18**"*.
> AC1 của epic nói *"thả chuột **hoặc kết thúc vùng chọn bằng bàn phím**"* — vế thứ hai hôm
> nay **không thực hiện được**: `.original`/`.hv-parallel` là `<div>`/`<p>` không sửa được,
> và một phần tử như vậy **không** nhận `Shift+Mũi tên` nếu không bật caret browsing.
> ⇒ **Quyết định #2** là phần ĐẮT NHẤT của story, và nó đụng hợp đồng tiêu điểm mà Story
> 1.14 dặn không chạm.
>
> ⚠️ **Bảy quyết định phải chốt ở Task 0.** **#1 chặn thật** *(hình dạng hợp đồng — sửa sau
> là mổ lại cả bốn panel)*. **#2 đắt nhất**. **#3 chặn phạm vi** *(bật `Substring` hay không —
> nếu bật thì ba mục `deferred-work` nữa rơi vào story này)*.
>
> 🔴 **Cây làm việc không SẠCH lúc tạo story:** `src/modes/libraryImport.ts` có một bản vá
> **chưa commit** *(gọi `ensureChapterLoaded()` sau `finishSubmit`, bắt bằng test tay
> 2026-08-07)*. Đọc §Bối cảnh git trước khi gõ dòng đầu tiên.

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-07 | **BẢY QUYẾT ĐỊNH CHỐT + bốn câu hỏi Ice trả lời.** **#1 = (a)** sổ đăng ký OPT-IN, một listener `document`, vị từ đọc `anchorNode` — *nhưng* **không theo cách Task 1 mô tả**: ĐO thật hai engine cho thấy `anchorNode` **không bao giờ** nằm trong `<input>` *(nó là phần tử CHA, `nodeType 1`)*, `deferred-work.md:635` **SAI trên cả hai engine** *(`toString()` trả `"nội "`, không `''`)*, và `document.activeElement` cho **âm tính giả**. ⇒ vị từ đọc **`anchorNode.nodeType === TEXT_NODE`**. **#2 = (a)** `tabindex="0"` + `Selection.modify()` qua 5 command; **ĐO THẬT** WKWebView *(bộ đo Swift)* + Chromium *(headless)*, 19 phép thử, hai engine **khớp từng dòng** — và **Bẫy 9 không CÓ THẬT**: `'word'` trên văn xuôi tiếng Trung phân đoạn ĐÚNG *(`他` / `打開`)* ở cả hai. **#3 = (b) — Ice chốt NGƯỢC mặc định story: BẬT `Substring`**, cài như **ĐƯỜNG LUI** *(`Exact` trước; rỗng **và** ≤ 4 ký tự ⇒ tra lại)* chứ không phép thay thế — thay thẳng theo độ dài **hồi quy AC1** *(bôi đen `山` sẽ trả mọi đầu mục CHỨA `山`)*. **#4 = (a)** qua `dispatch`. **#5 = (a)** sự kiện + dedupe *(chỉ đường tự động)*. **#6 = (a)** CSS thuần. **#7 = (a)** cờ TẮT mặc định, mốc cuối sau `rAF`. **Câu hỏi Ice #2 = (c)** ánh xạ ngược âm → ký tự — cài bằng **bảng VỊ TRÍ**, không bảng tra âm→chữ *(thứ đó đa trị: `"lương"` → 良/涼/糧/量/粱, và là FR113/Story 3.7)*. **#3** `Tab` dừng ở thân panel: **chấp nhận**. **#4** vạch tiến trình: `ornament` *(mặc định story)*. **Bản vá `libraryImport.ts` commit riêng** = `09d9c87`. 🔴 **Ba mệnh đề của story bị phép đo lật:** ① Bẫy 9 không có thật; ② `:635` sai lý do; ③ **`Selection.modify()` đi XUYÊN QUA `user-select: none` trên WKWebView** ⇒ đường bàn phím AC11 làm rò âm Hán Việt vào truy vấn ⇒ `resolveParallel` đọc node `.hv-char` thay vì `toString()`. Baseline đo lại: `cargo test` **225** · **9/9** lệnh DoD xanh. |
| 2026-08-07 | Tạo story. Baseline `4136f3f`, cây làm việc **không sạch** *(một bản vá chưa commit ở `libraryImport.ts`)*. **Baseline đã đo, không chép:** `cargo test` **225/225 xanh** · `npm run build` xanh · **7 cổng `check:*` xanh**. Phân tích: `epics.md:1752-1805` §Story 1.18 + §1.16/1.17/1.19/1.20 *(ranh giới hai đầu)* + `:106` FR21 + `:324` NFR1 · `ARCHITECTURE-SPINE.md` *(AD-1 · AD-15 · AD-19 · AD-21 · AD-24 · AD-34 · AD-44)* · `DESIGN.md` §Motion *(bảng chín hàng — **đặc tả nguyên văn của story này**)* + §Sàn tương phản *(luật `opacity`)* · `EXPERIENCE.md:131` *(FR21 — *"**không được thiết kế lại cho khác đi**"*)* · `mockups/motion-auto-lookup.html` *(bảng đặc tả chuyển động + ba cặp minh hoạ đúng/sai)* · story `1-17`, `1-16`, `1-14` · **toàn bộ `deferred-work.md`** *(**năm** mục gọi đích danh Story 1.18)* · mã thật `src/panels/**` + `src/commands/**` + `src/main.ts` + `scripts/check-layout.mjs` + `scripts/check-commands.mjs`. **Phát hiện:** **0** đường nghe vùng chọn nào tồn tại *(`window.getSelection` có đúng **một** chỗ gọi — `main.ts:182`, dep TỐI THIỂU của 1.17)* · `window.matchMedia` **không có trong `ALLOWED_GLOBAL_MEMBERS`** và Story 1.17 §⑥ đòi giữ số đó ở **0** ⇒ `prefers-reduced-motion` phải đi bằng **CSS thuần** *(Quyết định #6)* · `.lookup-head` khoá `height: 76px; overflow: hidden` ⇒ vạch tiến trình 250 ms **không được thêm một pixel nào** vào chiều cao đó *(Bẫy 6)* · Panel Lookup **tự nó chứa chữ**, nên một listener `document` toàn cục dựng một **vòng tự thay thế** *(Bẫy 1 — nguy hiểm nhất của story)*. |

**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story key:** `1-18-auto-lookup`
**Covers:** **FR21** *(Auto-Lookup: bôi đen ở Source, AI Translation **hoặc** Editor → kết quả hiện **ngay** ở Panel Lookup; không copy, không paste, không chuyển cửa sổ)*
**Nhận nợ từ 1.16:** **bôi đen nguyên văn bằng BÀN PHÍM** *(`deferred-work.md:608` — Ice chốt ghi nợ cho story này)*
**Nhận nợ từ 1.17:** **hợp đồng vùng chọn thật** thay dep TỐI THIỂU `currentSelection` *(`deferred-work.md:635`)*
**Governed by:** **AD-34** *(**mọi** thao tác qua `CommandRegistry`; sàn NFR17 là **cấu trúc**, không kỷ luật; không màu viết thẳng)* · **AD-1** *(quy tắc nghiệp vụ ở Rust; webview render + state UI)* · **AD-24** *(một cửa sổ OS)* · **AD-15** *(đúng **ba** điểm ra mạng — không cái nào trong đường tra cứu)* · **AD-19** *(không hợp nhất nguồn — story này không chạm tầng đó)* · **AD-21/NFR16** *(không chuỗi hiển thị từ Rust)* · **AD-44 ④** *(rỗng **có lý do** không được trông giống rỗng im lặng)*
**UX-DR phải tôn trọng:** **UX-DR6** *(không `opacity` làm mờ **chữ** ở trạng thái nghỉ — hiệu ứng 0.4→1 là **quá độ**, được phép; xem `DESIGN.md:216-219`)* · **UX-DR5** *(`ornament` là màu của **nét** — vạch tiến trình LÀ một nét)* · **UX-DR8/UX-DR17** *(hợp đồng tiêu điểm — **Quyết định #2 là chỗ DUY NHẤT story này được chạm**, và phải ghi ra)* · **UX-DR16** *(không elevation)* · **UX-DR15** *(thứ tự hy sinh panel — không chạm, Story 4.12)* · **UX-DR27** *(không trạng thái rỗng câm)*
**Ràng buộc xuôi dòng phải để lại chỗ đứng:** **FR37/FR38 · Story 1.19** *(bật/tắt nguồn)* · **FR41 · Story 1.20** *(lịch sử + ghim — tab thứ ba; **lịch sử ĐỌC từ đường tra, nên đường đó phải có đúng MỘT điểm nghẽn**)* · **FR64 · Story 7.7** *(Concordance — chủ của `LookupMode::Substring`)* · **FR50 · Story 3.4** *(đánh dấu thuật ngữ Glossary trong Panel Source — **cùng bề mặt chữ, cùng vùng chọn**)* · **Epic 2 · Epic 4** *(Editor + AI Translation có nội dung ⇒ **không được sửa một dòng nào của hợp đồng**)* · **Story 4.12** *(màn hình hẹp)*
**NFR:** **NFR1** *(p95 < 100 ms **đầu-cuối, từ lúc thả chuột**)* · **NFR13** *(ngoại tuyến)* · NFR14 *(hai nền tảng)* · **NFR15** *(**0** phụ thuộc mới)* · NFR16 *(chuỗi ở `vi.json`)* · **NFR17** *(bàn phím — story này là chỗ món nợ `:608` đóng)*
**Ngày tạo:** 2026-08-07

---

## 🔴 ĐỌC TRƯỚC TIÊN — NĂM VIỆC STORY NÀY KHÔNG LÀM

### ① KHÔNG đụng một dòng nào của tầng dữ liệu tra cứu

`core/dict/**` · `ports/dict_source.rs` · `commands/dict.rs` · `src/config/dict.ts` ·
`src/panels/lookupPanelState.ts::runLookup` — **năm chỗ này đã đúng và đã đo**. Story 1.17
đã ghi sẵn ở `lookupPanelState.ts:246`:

> *"`deps.currentSelection` (tiêm ở `src/main.ts`) cấp văn bản; Story 1.18 chỉ cần thay ĐÚNG
> dep đó bằng hợp đồng vùng chọn thật, **không phải chạm hàm này**."*

⇒ `runLookup` giữ nguyên chữ ký, giữ nguyên bộ đếm `sequence`, giữ nguyên `resetLookupPanel`.
Nếu bản cài của bạn phải sửa `runLookup`, **dừng lại và đọc lại Quyết định #1** — gần như
chắc chắn hợp đồng đang bị đặt sai chỗ.

⚠️ **Ngoại lệ DUY NHẤT** *(và nó là một phép THÊM, không một phép sửa)*: mốc thời gian đo NFR1
— xem **Quyết định #7**.

### ② KHÔNG bật `LookupMode::Substring` *(trừ khi Quyết định #3 chốt ngược)*

`deferred-work.md:615` đoán rằng *"Story 1.18 (Auto-Lookup, dùng `Substring` khi bôi đen
ngắn)"* sẽ là chỗ mở nhánh đó. 🔴 **Đó là một PHỎNG ĐOÁN của tác giả 1.17, không phải một
AC.** Đọc lại `epics.md:1754-1805`: **không một mệnh đề nào của Story 1.18 nhắc tới chuỗi con,
độ dài truy vấn, hay chế độ tra.**

⇒ Mặc định đề xuất: **KHÔNG bật `Substring`**, và **ba** mục `deferred-work` đi kèm nó
*(`:615` `query_too_short` không thực thi được · `:631` nhánh `Substring` nạp toàn bộ vào RAM ·
`:633` chuỗi `query_too_short` chỉ một thao tác không tồn tại)* **chuyển nguyên sang Story 7.7**.
Chốt ngược ⇒ story phồng lên đúng ba mục đó **cộng** một lượt đo NFR1 thứ hai trên nhánh
`fts_trigram`. Xem **Quyết định #3**.

### ③ KHÔNG dựng lịch sử tra cứu — FR41 là Story 1.20

Auto-Lookup sinh ra **hàng trăm lượt tra mỗi Chương** *(`DESIGN.md:334`)*, nên cám dỗ *"ghi
lại chúng luôn thể"* rất mạnh. **Không.** Lịch sử, ghim, và **tab thứ ba** của Panel Lookup
là Story 1.20.

🔴 **Nhưng ràng buộc phải để lại:** mọi lượt tra — tay lẫn tự động — phải đi qua **đúng MỘT
điểm nghẽn** *(`lookupPanelState.ts::runLookup`)*. 1.20 chỉ được phép **thêm một dòng ghi**
vào chỗ đó, không phải đi tìm hai đường gọi rải rác.

### ④ KHÔNG đụng ngưỡng màn hình hẹp, không đụng thanh trạng thái, không đụng `matchMedia`

UX-DR15 *(*"Tra cứu rút về thanh trạng thái"*)* là **Story 4.12**. Cây `src/**` hôm nay có
**0** lời gọi `matchMedia` và **0** lời gọi `window.innerWidth`, và `ALLOWED_GLOBAL_MEMBERS`
của `scripts/check-layout.mjs` **không chứa `window.matchMedia`**.

🔴 ⇒ `prefers-reduced-motion` **phải đi bằng CSS thuần** *(`@media (prefers-reduced-motion:
reduce)`)*. Thêm `window.matchMedia` vào danh sách cho phép để đọc nó bằng JS là **mở đúng
cánh cửa** mà §⑥ của Story 1.17 vừa khoá — và nó không mua thêm gì cả. Xem **Quyết định #6**.

### ⑤ KHÔNG dựng một spinner, không dựng một trạng thái "đang tải"

AC7 của Story 1.17 cấm **vĩnh viễn**, và story này không nới nó. Vạch tiến trình 250 ms của AC8
**không phải một spinner**: nó là một **nét mảnh**, không quay, không nhấp nháy, và nó chỉ xuất hiện ở
một ca mà `DESIGN.md` tự nói *"lẽ ra không xảy ra"*.

⚠️ Và nó không được đẩy `.lookup-head` cao thêm một pixel — xem **Bẫy 6**.

---

## Story

As a người dịch,
I want bôi đen một cụm từ là **thấy ngay** nghĩa của nó,
So that tôi không phải copy, paste hay chuyển cửa sổ **hàng trăm lần mỗi Chương**.

---

## Ranh giới phạm vi — ĐỌC TRƯỚC KHI GÕ DÒNG ĐẦU TIÊN

| Trong phạm vi | không Ngoài phạm vi (và ai sở hữu) |
|---|---|
| **Hợp đồng vùng chọn dùng chung** — một module, **bốn** panel đăng ký, không một listener toàn cục | Một cài đặt chỉ cho `SourcePanel.vue` *(trượt AC2)* · một listener `document` không lọc nguồn *(Bẫy 1)* |
| **Bôi đen bằng BÀN PHÍM** trên bề mặt nguyên văn *(`deferred-work.md:608`)* | Caret browsing của hệ điều hành · `contenteditable` thật *(Quyết định #2)* |
| Thay dep `currentSelection` ở `main.ts` bằng hợp đồng | Sửa `runLookup`/`resetLookupPanel`/`config/dict.ts` *(§KHÔNG-LÀM ①)* |
| **Hiệu ứng 90 ms** `opacity` 0.4→1 `ease-out`, huỷ được, không xếp hàng | Bất kỳ hiệu ứng nào khác trong ứng dụng · `translate`/`scale` *(`DESIGN.md` cấm cả ứng dụng)* |
| **Cuộn về đầu tức thì** khi kết quả mới đến | `scroll-behavior: smooth` ở bất kỳ đâu *(cấm — `DESIGN.md:342`)* |
| **Vạch tiến trình 250 ms** ở đáy vùng đầu mục | Spinner · trạng thái "đang tải" *(AC7 của 1.17 — cấm vĩnh viễn)* |
| `prefers-reduced-motion` bằng **CSS thuần** | `window.matchMedia` *(§KHÔNG-LÀM ④)* |
| **Đo NFR1 đầu-cuối ≥ 100 lượt liên tiếp**, ghi SỐ | Đóng món nợ *"vòng IPC Tauri thật chưa đo"* nếu không dựng được bản đóng gói — **ghi ra**, không khai đạt |
| Nâng mọi hằng `*_FLOOR` bị vượt | Nới một phép cưỡng chế · bỏ tệp khỏi tầm quét |
| **0 phụ thuộc mới** (NFR15) | Một thư viện debounce/animation |
| | **Substring / `query_too_short`** *(Quyết định #3 — mặc định: **7.7**)* |
| | **Lịch sử + ghim** *(1.20)* · **bật/tắt nguồn** *(1.19)* · **Glossary highlight** *(3.4)* |
| | **Màn hình hẹp / thanh trạng thái** *(4.12)* · **Sync scrolling** *(2.12)* |
| | Nội dung THẬT của Editor *(Epic 2)* / AI Translation *(Epic 4)* — story này chỉ nối **điểm cắm** |

---

## 🔴 BẢY QUYẾT ĐỊNH — PHẢI CHỐT Ở TASK 0, TRƯỚC DÒNG MÃ ĐẦU TIÊN

> Mỗi quyết định có một **mặc định đề xuất kèm lý do**. Chốt theo mặc định ⇒ ghi một dòng vào
> Change Log. Chốt ngược ⇒ ghi **lý do**, không chỉ ghi lựa chọn.

### 🔴 Quyết định #1 — **Hình dạng hợp đồng vùng chọn** *(CHẶN THẬT)*

Sửa sau là mổ lại cả bốn panel. Đây là quyết định đắt nhất về mặt **kiến trúc** *(Quyết định
#2 đắt nhất về mặt **công**)*.

**(a) — MẶC ĐỊNH ĐỀ XUẤT: sổ đăng ký OPT-IN theo phần tử, một listener trên `document`.**

```ts
// src/panels/selectionContract.ts  (NEW)
export function registerSelectionSource(el: HTMLElement): () => void
export const currentSelectionText: () => string          // ← thay dep của main.ts
export function attachSelectionWatcher(target: Document): () => void
```

- Mỗi panel văn bản gọi `registerSelectionSource(rootEl)` lúc `mount`, nhả lúc `unmount`.
- **Một** listener `mouseup`/`keyup` trên `document` *(không bốn listener, không một listener mỗi
  panel — vùng chọn kéo ra NGOÀI panel rồi mới thả là ca thường ngày, xem Bẫy 2)*.
- Vị từ *"vùng chọn này có thuộc một nguồn đã đăng ký không"* đọc **`anchorNode`**, không đọc
  `event.target`.

**Vì sao (a):** nó là hình dạng **duy nhất** thoả cả bốn ràng buộc cùng lúc —
① AC2 *(bốn panel, một cài đặt)*; ② **Bẫy 1** *(Panel Lookup không được là nguồn)*;
③ `deferred-work.md:635` *(`<input>`/`<textarea>` của Library không được là nguồn)*;
④ Epic 2/4 chỉ thêm **một dòng** vào panel của họ.

**(b) — listener toàn cục + danh sách LOẠI TRỪ.** **Bác.** Danh sách loại trừ phải được
bảo trì tay qua chín epic; quên một bề mặt mới là mở lại Bẫy 1 **im lặng**. Cùng lớp lỗi mà
AD-44 ① bác *"sổ đăng ký tệp nào chứa ngôn ngữ nào"*.

**(c) — mỗi panel tự nghe `mouseup` trên gốc của nó.** **Bác.** Thả chuột ngoài panel ⇒
không sự kiện nào ⇒ vùng chọn không bao giờ được tra. Đó là ca **thường ngày**, không ca biên.

⚠️ **Tệp sống ở `src/panels/`, không `src/commands/`** — nó dùng `ref` của Vue và bị `main.ts`
tiêm vào, đúng cửa mà `sourcePanelState.ts`/`lookupPanelState.ts` đã đi qua. Một `import`
của nó trong `src/commands/index.ts` **giết Kiểm C/D/E** của `check:commands`.

---

### 🔴 Quyết định #2 — **Bôi đen bằng BÀN PHÍM** *(ĐẮT NHẤT — `deferred-work.md:608`)*

Sự thật kỹ thuật, không ý kiến: một `<div>`/`<p>` không sửa được **không nhận** `Shift+Mũi tên` để mở
rộng vùng chọn. Trình duyệt chỉ cho điều đó khi ① caret browsing bật *(mặc định TẮT, và không
bật được bằng mã)*, hoặc ② phần tử `contenteditable`, hoặc ③ ứng dụng **tự dựng `Range`**.

**(a) — MẶC ĐỊNH ĐỀ XUẤT: `tabindex="0"` trên bề mặt chữ + `Selection.modify()` qua
CommandRegistry.**

- Bề mặt chữ *(`.original`, `.hv-parallel`, `.hv-switch`)* nhận `tabindex="0"` ⇒ vào được
  vòng `Tab`, đặt được caret bằng `el.focus()`.
- **Bốn command mới** trong `CommandRegistry` — đúng AD-34 §1, và nhờ vậy Story 1.21 gán lại
  được phím cho chúng:
  `selection.extend_left` · `selection.extend_right` · `selection.extend_word_left` ·
  `selection.extend_word_right` *(hợp âm mặc định: `Shift+ArrowLeft/Right`,
  `Mod+Shift+ArrowLeft/Right`)*.
- Cài đặt: `window.getSelection()?.modify('extend', 'left'|'right', 'character'|'word')`.
  ⚠️ `Selection.modify()` là **không chuẩn**, nhưng nó có ở **cả Chromium lẫn WebKit** — tức
  **cả hai** engine mà Tauri dùng *(WebView2 = Chromium · WKWebView = WebKit)*. Task 0 phải
  **đo thật trên cả hai**, không tin bảng tương thích.
- Một command thứ năm để **đặt caret vào bề mặt**: `selection.focus_source` *(hoặc tái dùng
  `focus.next_panel` + một `Range` rỗng ở đầu — chốt ở Task 0)*.

**Vì sao (a):** nó đóng `:608` bằng **cấu trúc** *(AD-34: *"sàn khả năng tiếp cận là cấu
trúc, không kỷ luật"*)*, không bằng một API trình duyệt cho không; và nó để Story 1.21 sửa được.

**(b) — `contenteditable="plaintext-only"` + chặn mọi `beforeinput`.** **Bác.** Nó biến
nguyên văn thành một ô nhập trên mọi đường mà ta không chặn hết được *(kéo-thả, paste của OS,
IME, undo stack của webview)*, và AD-1 nói nguyên văn là **dữ liệu không sửa được**. Một lỗ ở
đây là mất văn bản gốc của người dùng.

**(c) — hoãn tiếp sang một story sau.** **Bác** — Ice đã chốt ngược ở lượt code review
2026-08-06, và AC1 của epic gọi tên nó bằng chữ.

🔴 **Ràng buộc phải ghi ra dù chốt đường nào:** đây là chỗ **DUY NHẤT** story này được chạm
hợp đồng tiêu điểm *(UX-DR8/UX-DR17, Story 1.14 dặn không chạm)*. `PanelFrame` mang
`tabindex="-1"` và không vào vòng `Tab`; thêm một `tabindex="0"` **bên trong** nó là một thay
đổi **nhìn thấy được** *(bấm `Tab` nay dừng ở thân panel)*. **Ghi vào Completion Notes, không
im lặng.**

---

### Quyết định #3 — **Có bật `LookupMode::Substring` không** *(CHẶN PHẠM VI)*

**(a) — MẶC ĐỊNH ĐỀ XUẤT: KHÔNG.** `LookupMode::Exact` giữ nguyên cho mọi lượt Auto-Lookup.

**Vì sao:** không một mệnh đề nào của `epics.md:1754-1805` nhắc tới chuỗi con. Bôi đen một cụm
từ là *"tra CHÍNH cụm này"* — một **ý định người dùng** rõ ràng, đúng vế mà AD-44 ① nói
*"`LookupMode` là luật về **ý định người dùng** và phải do chỗ gọi khai"*. Chuỗi con là
**Concordance (FR64, Story 7.7)**, một năng lực khác với một màn hình khác.

⇒ Ba mục `deferred-work` *(`:615` · `:631` · `:633`)* **chuyển nguyên sang 7.7**, ghi ra ở
Task 9. **Không đóng chúng, không im lặng bỏ qua.**

**(b) — CÓ, khi vùng chọn ngắn.** ⇒ story nhận thêm: một trần an toàn ở SQL cho ba nhánh
`verify_*` *(`:631`)* · sửa chuỗi `panel.lookup.query_too_short` *(`:633` — nó đang chỉ một
thao tác không tồn tại: *"gõ thêm ít nhất ba ký tự"*, mà panel không có ô nhập nào)* · **verify bằng
MẮT** rằng chuỗi đó thật sự hiện ra *(`:615`)* · **và một lượt đo NFR1 thứ hai** trên nhánh
`fts_trigram` *(AD-44 ⑥: *"NFR1 đo TRÊN đường tiếng Anh, không suy ra từ số đo tiếng Trung"*)*.

---

### Quyết định #4 — **Đường đi của một lượt Auto-Lookup: qua `dispatch` hay thẳng `runLookup`**

**(a) — MẶC ĐỊNH ĐỀ XUẤT: qua `dispatch('lookup.lookup_selection')`.**

Hợp đồng phát hiện vùng chọn đã dừng ⇒ gọi `dispatch('lookup.lookup_selection')` ⇒ command
đọc `deps.currentSelection()` *(nay là hợp đồng)* ⇒ gọi `deps.runLookup(text)`.

**Vì sao:** ① AD-34 — *"không thao tác nào chỉ tồn tại trong một handler"*; ② **`Mod+Alt+L` và
Auto-Lookup thành ĐÚNG MỘT đường**, nên không có ca nào một đường sửa mà đường kia quên *(và
Story 1.20 chỉ có một chỗ để cắm lịch sử vào)*; ③ không thêm một command mới nào ⇒ không đụng
`COMMAND_FLOOR` vì lý do này.

⚠️ **Hệ quả phải ghi ra:** `dispatch` là **đồng bộ** và `registry.dispatch` **NÉM** với id
lạ. Hợp đồng gọi nó từ một handler DOM ⇒ một lần ném ở đó không được giết listener. Bọc hay không
là một chi tiết cài đặt, **không một chỗ để im lặng** — ghi lý do tại chỗ.

**(b) — gọi thẳng `runLookup`.** **Bác.** Nó dựng đường thứ hai vào cùng một hành vi, và
`check:commands` **không nhìn thấy** đường đó *(Kiểm A chỉ canh `@click`)*.

---

### Quyết định #5 — **Tín hiệu *"vùng chọn đã dừng"*, và có DEDUPE không**

**(a) — MẶC ĐỊNH ĐỀ XUẤT: sự kiện, không hẹn giờ.**

| Đường | Tín hiệu | Vì sao không phải cái khác |
|---|---|---|
| Chuột | `mouseup` trên `document` | `selectionchange` bắn **liên tục** trong lúc kéo ⇒ *"tra hàng loạt cụm dở dang"* mà `mockups/motion-auto-lookup.html:200` cấm đích danh |
| Bàn phím | `keyup` khi **`Shift` nhả** | Giữ `Shift` + gõ `→` nhiều lần là **một** thao tác chọn, không phải năm |
| Cả hai | vùng chọn **rỗng** ⇒ **không làm gì**, không xoá panel | Một cú **bấm** *(không kéo)* cũng bắn `mouseup` và thu vùng chọn về một caret. Xoá panel ở đó là mất kết quả người dùng đang đọc |

**Dedupe:** **CÓ** — vùng chọn không đổi so với lượt tra đang hiện ⇒ **không phát lượt IPC mới**.

**Vì sao dedupe:** người dùng bấm lại vào **chính** cụm vừa tra là thao tác thường ngày; một
lượt IPC + một lượt hiệu ứng 90 ms cho **không thông tin mới** là chi phí thuần. ⚠️ **Và nó
CHẠM AC4**: *"đo trên ít nhất 100 lần tra liên tiếp"* — nếu dedupe bật thì **bảng đo phải
dùng 100 truy vấn KHÁC NHAU**, không 100 lượt cùng một chữ. Ghi ra ở Task 8.

**(b) — `selectionchange` + debounce ~120 ms.** **Bác** *(mặc định)*: nó thêm một hằng số
thời gian không ai đo được vào **đúng đường nóng NFR1**, và nó vẫn không phân biệt được *"đã dừng
kéo"* với *"đang kéo chậm"*.

---

### Quyết định #6 — **`prefers-reduced-motion` đi bằng CSS hay JS**

**(a) — MẶC ĐỊNH ĐỀ XUẤT: CSS thuần.**

```css
@media (prefers-reduced-motion: reduce) {
  .lookup-fade   { animation: none; opacity: 1; }
  .lookup-progress { animation: none; }   /* nét TĨNH, vẫn HIỆN — xem dưới */
}
```

**Vì sao:** ① `window.matchMedia` không có trong `ALLOWED_GLOBAL_MEMBERS` và §⑥ của Story 1.17
đòi giữ số đó ở **0** *(Story 4.12 là chủ)*; ② CSS phản ứng **tức thì** khi người dùng đổi
thiết lập hệ điều hành giữa phiên, một `matchMedia` đọc một lần thì không; ③ `DESIGN.md:348` nói
*"thuộc **sàn khả năng tiếp cận**, không phải tuỳ chọn"* — sàn thì không nên phụ thuộc một nhánh
JavaScript chạy đúng.

🔴 **Câu con phải chốt cùng lượt:** dưới `reduced-motion`, vạch tiến trình 250 ms **vẫn HIỆN
nhưng TĨNH** *(không chạy qua lại)* — nó là một **thông tin** *("lượt tra này đang lâu")*, không
một trang trí. Bỏ hẳn nó là biến một trạng thái có tên thành im lặng, đúng thứ UX-DR27 cấm.

**(b) — đọc bằng `matchMedia` rồi gắn cờ.** **Bác** — ba lý do trên.

---

### Quyết định #7 — **Đo NFR1 đầu-cuối: mốc đầu và mốc cuối đặt ở ĐÂU**

`epics.md:1774` đòi *"độ trễ đầu-cuối **từ lúc thả chuột** tới lúc kết quả **hiển thị**"*,
p95 trên ≥ 100 lượt. Hai mốc đó nằm ở **hai module khác nhau**.

**(a) — MẶC ĐỊNH ĐỀ XUẤT: một hằng đo TẮT ĐƯỢC, mốc đầu ở hợp đồng, mốc cuối sau `paint`.**

- Mốc đầu: `performance.now()` ngay trong handler `mouseup` của hợp đồng — **trước**
  `dispatch`.
- Mốc cuối: `nextTick()` → `requestAnimationFrame()` trong `LookupPanel.vue` — tức **sau khi
  trình duyệt đã vẽ**, không chỉ sau khi Vue cập nhật DOM.
- Cả hai nằm sau **một** cờ module-level *(mặc định TẮT)*, không rải `console.log` trong đường
  sản phẩm.

**Vì sao:** đo tới `nextTick` mà không tới `rAF` là bỏ mất lượt **vẽ** — đúng phần mà
`epics.md` gọi là *"hiển thị"*, và nó là phần đắt nhất khi bản ghi có 22 nghĩa
*(`mockups/lookup-real-density.html`)*.

🔴 **Giới hạn phải khai TRƯỚC KHI đo, không sau:** nếu không dựng được bản Tauri thật *(WKWebView /
WebView2)*, con số này không **bao gồm vòng IPC thật** — đúng món nợ `deferred-work.md` mà
Story 1.17 để lại. ⇒ Bảng đo phải có **một cột ghi rõ engine và cách `invoke` được cấp**,
và §Completion Notes phải nói *"không đóng món nợ đó"* nếu chưa đo được. **không Ghi một con số mà
không ghi điều kiện của nó.**

**(b) — đo từ `dispatch` tới `nextTick`.** **Bác** — nó đo **một đoạn ngắn hơn** đoạn mà
AC đòi, rồi báo cáo như thể đã đo đủ.

---

## Acceptance Criteria

### AC1 — Bôi đen ⇒ kết quả hiện, **không thao tác nào khác** *(FR21)*

**Given** người dùng bôi đen một cụm từ ở Panel Source *(bằng **chuột**)*
**When** thả chuột
**Then** kết quả tra cứu hiện ở Panel Lookup **không một thao tác nào khác** — không phím, không nút,
không menu

**Given** người dùng mở rộng vùng chọn **bằng bàn phím** *(Quyết định #2)*
**When** kết thúc vùng chọn *(nhả `Shift`)*
**Then** cùng kết quả, cùng đường, **không một nhánh mã riêng cho bàn phím** ở tầng tra cứu

**Given** người dùng chỉ **bấm** *(không kéo)*, hoặc vùng chọn rỗng
**When** `mouseup` bắn
**Then** **không lượt tra nào được phát**, và Panel Lookup **giữ nguyên** nội dung đang hiện

**Given** phím `Mod+Alt+L` *(Story 1.17)*
**When** bấm
**Then** vẫn hoạt động **y hệt trước story này** — không hồi quy

### AC2 — 🔴 **HỢP ĐỒNG DÙNG CHUNG, và nó nghiệm thu bằng MÁY**

**Given** cơ chế Auto-Lookup
**When** đăng ký
**Then** nó gắn vào **một hợp đồng vùng chọn dùng chung cho mọi panel văn bản** — **không một
listener nào sống trong `SourcePanel.vue`**

**Given** **cả bốn** panel của Workspace
**When** mount
**Then** **cả bốn** gọi đúng một hàm đăng ký của hợp đồng *(`registerSelectionSource`)*
**And** một **cổng đếm** cưỡng chế con số **4** — không một chú thích hứa hẹn

> 🔴 Vì sao AC này đòi **cổng**, không chỉ đòi mã: Panel AI Translation và Editor hôm nay **không
> có chữ**, nên một lượt đăng ký thiếu ở đó không để lại **bất kỳ triệu chứng nào** cho tới
> Epic 2/4 — tức **hai epic sau**, và tới lúc đó không ai nhớ AC này tồn tại. Đây chính xác là
> lớp lỗi mà AD-34 §2 dựng sổ `FOCUS_OWNERS` đối chiếu **hai chiều** để chặn.

**Given** Epic 2 / Epic 4 đổ nội dung thật vào Editor / AI Translation
**When** chúng làm vậy
**Then** không **một dòng nào** của hợp đồng phải sửa — chúng đã đăng ký sẵn từ story này

### AC3 — 🔴 **Panel Lookup và ô nhập không BAO GIỜ là nguồn vùng chọn** *(Bẫy 1 · `:635`)*

**Given** người dùng bôi đen chữ **bên trong Panel Lookup** *(một nghĩa, một ví dụ, một trích
dẫn)*
**When** thả chuột
**Then** **không lượt tra nào được phát** — kết quả đang đọc **không bị thay dưới tay người đọc**

**Given** người dùng bôi đen trong ô nhập của Library *(`<input>` / `<textarea>`)*
**When** thả chuột
**Then** **không lượt tra nào được phát**

**Given** một bề mặt chữ **mới** được thêm ở một story sau mà **không đăng ký**
**When** người dùng bôi đen ở đó
**Then** **không lượt tra nào được phát** — hợp đồng là **OPT-IN**, không opt-out *(Quyết định #1)*

### AC4 — 🔴 **NFR1 đo ĐẦU-CUỐI, ≥ 100 lượt, ghi SỐ chứ không ghi lời hứa**

**Given** độ trễ **từ lúc thả chuột** tới lúc kết quả **hiển thị**
**When** đo trên **ít nhất 100 lượt tra liên tiếp**
**Then** **p95 < 100 ms**
**And** bảng số ghi vào §Debug Log References mang: **n · p50 · p95 · p99 · max · engine ·
cách `invoke` được cấp · truy vấn dùng** *(không một dòng văn xuôi)*
**And** nếu vòng IPC Tauri thật không đo được thì **điều kiện đó ghi thẳng**, và món nợ
`deferred-work.md` **không được đánh dấu đóng**

⚠️ Nếu Quyết định #5 chốt **có dedupe** thì 100 lượt phải là **100 truy vấn KHÁC NHAU** —
100 lượt cùng một chữ sẽ đo đúng đường dedupe, không đường tra.

### AC5 — **Hiệu ứng nội dung mới vào: 90 ms · `opacity` 0.4 → 1 · `ease-out`**

**Given** nội dung mới vào Panel Lookup
**When** hiển thị
**Then** hiệu ứng **90 ms**, `opacity` **0.4 → 1**, `ease-out`
**And** **không `translate`**, **không `scale`** *(`DESIGN.md:341` — cấm cho cả ứng dụng)*
**And** nội dung **cũ** bị thay **thẳng**, **không hiệu ứng ra**

**Given** trạng thái **không tìm thấy** *(và cả bốn trạng thái rỗng của AC6 Story 1.17)*
**When** hiển thị
**Then** **cùng 90 ms** — trạng thái rỗng không được hiện **chậm hơn** trạng thái có kết quả
*(`DESIGN.md:347`: chậm hơn sẽ bị đọc thành *"đang tìm tiếp"*)*

**Given** luật `opacity` của `DESIGN.md:216-219`
**When** hiệu ứng chạy
**Then** `opacity` chỉ đi **giữa 0.4 và 1 trong quá độ**, không dừng ở một mức trung gian nào ở
**trạng thái nghỉ** — chữ mờ thường trực là UX-DR6 vỡ

### AC6 — **Tra liên tiếp: HUỶ, không xếp hàng**

**Given** một lượt tra mới **trong lúc hiệu ứng đang chạy**
**When** xảy ra
**Then** hiệu ứng cũ bị **huỷ** và `opacity` đặt **thẳng về 1**
**And** hiệu ứng **không xếp hàng**, **không cộng dồn**, và **không khởi động lại**

> 🔴 **Đọc kỹ vế cuối** — nó là chỗ dễ cài sai nhất của story. *"Huỷ và đặt thẳng về 1"* không
> **đồng nghĩa** *"chạy lại hiệu ứng từ đầu"*. Mockup nói bằng chữ *(`:187`)*: **"Tra nhanh
> liên tiếp thì thành không có hiệu ứng — đúng như người dùng muốn."** Một bản cài `:key` lại
> node để hiệu ứng chạy lại **trông giống** đúng và **là** sai.

### AC7 — **Vị trí cuộn về đầu TỨC THÌ**

**Given** kết quả mới đến
**When** Panel Lookup cập nhật
**Then** vị trí cuộn về đầu **tức thì** — **không cuộn có hiệu ứng**
**And** không một `scroll-behavior: smooth` nào trong `src/**`
**And** vùng đầu mục *(`.lookup-head`)* **không cuộn** — nó `flex: none`, đúng AC7 của 1.17

### AC8 — **Vượt 250 ms ⇒ vạch tiến trình mảnh, không spinner**

**Given** một lượt tra vượt **250 ms**
**When** xảy ra
**Then** một **vạch tiến trình mảnh** hiện ở **đáy vùng đầu mục**
**And** **không spinner**, **không chuỗi "đang tải"** *(AC7 của Story 1.17 — cấm vĩnh viễn)*

**Given** vạch tiến trình
**When** hiện
**Then** `.lookup-head` **không đổi một pixel** chiều cao — hằng `--lookup-head-height: 76px`
giữ nguyên giá trị **và** giữ nguyên vai trò *(Bẫy 6)*

**Given** một lượt tra **dưới 250 ms** *(ca thường — NFR1 p95 < 100 ms)*
**When** xảy ra
**Then** vạch **không bao giờ xuất hiện**, kể cả một nháy

**Given** lượt tra trả lời, **hoặc** `resetLookupPanel()` chạy, **hoặc** một lượt tra mới
vượt mặt
**When** xảy ra
**Then** hẹn giờ 250 ms bị **huỷ** — không một vạch mồ côi nào ở lại trên màn hình

### AC9 — **`prefers-reduced-motion` ⇒ bỏ TOÀN BỘ hiệu ứng**

**Given** `prefers-reduced-motion: reduce`
**When** bất kỳ hiệu ứng nào của story này
**Then** bị **bỏ hoàn toàn**, đổi **tức thì**
**And** đường này đi bằng **CSS**, không bằng `window.matchMedia` *(Quyết định #6)*
**And** `matchMedia` / `window.innerWidth` trong `src/**` vẫn là **0 / 0**

**Given** `reduced-motion` bật **và** một lượt tra vượt 250 ms
**When** xảy ra
**Then** vạch tiến trình **vẫn hiện**, nhưng **TĨNH** — nó là thông tin, không trang trí
*(Quyết định #6)*

### AC10 — **Ngắt kết nối mạng ⇒ mọi đường tra cứu vẫn hoạt động ĐẦY ĐỦ** *(NFR13)*

**Given** không kết nối mạng
**When** tra cứu qua Auto-Lookup
**Then** mọi đường tra cứu hoạt động **đầy đủ**
**And** mệnh đề này nghiệm thu bằng **một dữ kiện đếm được** — đường tra cứu chạm **0** trong
ba điểm ra mạng của AD-15 — không bằng một câu khẳng định

### AC11 — 🔴 **Bôi đen bằng bàn phím: đóng `deferred-work.md:608`** *(NFR17 · AD-34)*

**Given** bề mặt nguyên văn *(cả `.original` LẪN tab Hán Việt)*
**When** người dùng dùng **chỉ bàn phím**
**Then** đặt được caret vào bề mặt **và** mở rộng được vùng chọn theo **ký tự** và theo **từ**

**Given** mọi thao tác chọn bằng bàn phím
**When** gọi
**Then** **có command đăng ký trong `CommandRegistry`** và **gán được phím** — đúng AD-34 §1,
và Story 1.21 sửa được

**Given** hai engine `WKWebView` *(macOS)* và `WebView2` *(Windows)*
**When** đo `Selection.modify()` *(hoặc đường thay thế của Quyết định #2)*
**Then** kết quả đo ghi ra cho **cả hai**, không suy từ một bảng tương thích

⚠️ Nếu vế Windows không đo được trong phiên ⇒ **ghi ra**, không đánh dấu đạt *(cùng kỷ luật món nợ
hai nền tảng mà 1.6/1.14/1.16/1.17 đã giữ)*.

### AC12 — **Vùng chọn ở tab Hán Việt không được hồi quy AC6 của Story 1.16**

**Given** kiểu xem **song song**
**When** bôi đen một đoạn rồi Auto-Lookup phát
**Then** truy vấn gửi đi **đúng bằng chuỗi ký tự nguồn** — không lẫn âm Hán Việt, không lẫn khoảng
trắng chèn thêm *(nguyên văn AC6 của 1.16 — `.hv-reading` `user-select: none`,
`position: absolute`)*
**And** phép kiểm này chạy **lại** trong story này, không tin số đo cũ: Auto-Lookup chạm bề mặt
đó **hàng trăm lần mỗi Chương** thay vì một lần mỗi lượt bấm phím

### AC13 — **Mọi cổng xanh, sàn nâng theo số THẬT, ranh giới không CHẠM giữ nguyên**

**Given** bộ DoD **chín lệnh**
**When** chạy
**Then** cả chín **exit 0**, và `cargo test` không tụt dưới **225**

**Given** mọi hằng `*_FLOOR` bị story này vượt
**When** rà
**Then** **được nâng theo số thật** — không để lại một sàn không còn canh được gì
*(`VUE_FLOOR` · `TS_FLOOR` · `COMMAND_FLOOR` · `DISPATCH_FLOOR` · `CLICK_FLOOR` ·
`FILE_FLOOR` ×2 · `COMPONENT_FILE_FLOOR` · `RS_FLOOR`)*

**Given** mỗi mục mới thêm vào `ALLOWED_GLOBAL_MEMBERS`
**When** thêm
**Then** đi kèm **một dòng nói nó phục vụ AC nào** *(luật của chính `check-layout.mjs:455`)*

**Given** ba ranh giới không chạm
**When** đếm lại sau story
**Then** `matchMedia` **0** · `window.innerWidth` **0** · phụ thuộc mới **0** *(npm lẫn crate)*

---

## Tasks / Subtasks

### Task 0 — Chốt bảy quyết định, ĐO trước khi chốt *(AC toàn bộ)*

- [x] Xác nhận baseline: `git status` · `cargo test` *(**225** — số của story)* · **chín** cổng
      DoD. ⚠️ Cây làm việc **không sạch** — xem §Bối cảnh git; quyết định xử lý bản vá
      `libraryImport.ts` **trước** khi gõ dòng đầu tiên.
- [x] **Quyết định #2 — ĐO THẬT, không đọc bảng tương thích.** Dựng một trang thử tối thiểu và
      chạy `Selection.modify('extend', …)` trên **cả hai** engine *(WKWebView qua
      `tauri dev`/Safari; WebView2 hoặc Chromium)*. Ghi: có chạy không · `word` có nhảy đúng
      biên từ **tiếng Trung** không *(⚠️ văn xuôi tiếng Trung không có khoảng trắng — `word` có thể
      nuốt cả câu; đây là ca **phải** thử, không ca biên)*.
- [x] Chốt **#1** *(hình dạng hợp đồng)* — **CHẶN**, mọi task sau đứng trên nó.
- [x] Chốt **#3** *(Substring không)* — **CHẶN PHẠM VI**.
- [x] Chốt **#4 · #5 · #6 · #7**.
- [x] Ghi cả bảy vào Change Log kèm **số đo** *(không chỉ kèm lựa chọn)*.

### Task 1 — Hợp đồng vùng chọn *(AC2 · AC3 · Quyết định #1)*

- [x] `src/panels/selectionContract.ts` **NEW** — `registerSelectionSource(el)` trả hàm nhả ·
      `currentSelectionText()` · `attachSelectionWatcher(document)`.
- [x] Vị từ thuộc-nguồn đọc **`anchorNode`**, không `event.target` *(Bẫy 2)*.
- [x] **Loại trừ CƠ HỌC ô nhập**: `anchorNode` nằm trong `<input>`/`<textarea>` ⇒ không nguồn.
      ⚠️ Trên Chromium/WebKit `window.getSelection().toString()` trả `''` cho vùng chọn bên
      trong ô nhập *(`deferred-work.md:635`)* — **không dựa vào đó**: nó là hành vi không chuẩn hoá,
      và nó biến một loại trừ CÓ Ý thành một tai nạn may mắn.
- [x] Doc-comment đầu tệp: vì sao **OPT-IN** *(Bẫy 1)* · vì sao **một** listener trên
      `document` *(Bẫy 2)* · vì sao tệp sống ở `panels/` không `commands/`.
- [x] `attachSelectionWatcher` gọi ở `main.ts`, **sau `mount()`**.

### Task 2 — Nối hợp đồng vào command *(AC1 · AC4 · Quyết định #4)*

- [x] `src/main.ts` — thay `currentSelection: () => window.getSelection()?.toString() ?? ''`
      bằng hợp đồng. ⚠️ **Đây là lượt gỡ dep TỐI THIỂU mà 1.17 đã hẹn** *(`main.ts:179-182`)*
      — sửa cả doc-comment ở đó, không để lại một lời hứa đã hết hạn.
- [x] Hợp đồng phát `dispatch('lookup.lookup_selection')` khi vùng chọn dừng.
- [x] **Dedupe** *(Quyết định #5)*: so với truy vấn **đang hiện**, không với truy vấn đang bay.
- [x] `scripts/check-layout.mjs` — mọi thành viên `window.`/`document.` mới vào
      `ALLOWED_GLOBAL_MEMBERS` **kèm dòng lý do AC** *(AC13)*.

### Task 3 — 🔴 Bốn panel đăng ký + CỔNG đếm *(AC2)*

- [x] `SourcePanel.vue` · `LookupPanel.vue`※ · `AiTranslationPanel.vue` · `EditorPanel.vue` —
      cả bốn gọi hàm đăng ký.
      ※ ⚠️ **Panel Lookup đăng ký với vai KHÁC** *(hoặc **không đăng ký** — chốt ở Task 0 cùng
      Quyết định #1)*: nó **không được** là nguồn *(AC3)*. Dù chọn đường nào, **ghi lý do tại
      chỗ** — một panel văn bản không đăng ký là đúng thứ AC2 tồn tại để chặn, nên ngoại lệ này
      phải đọc được từ chính mã.
- [x] **Cổng mới ở `scripts/check-commands.mjs`** *(hoặc `check-layout.mjs` — chốt ở Task 0)*:
      đếm số lời gọi đăng ký trong `src/panels/*.vue`, **sàn = 4**, kèm **đối chứng âm**
      *(hạ một lời gọi ⇒ cổng ĐỎ ⇒ khôi phục ⇒ XANH; ghi số vào Completion Notes)*.
- [x] ⚠️ Lời gọi viết **LITERAL**, không qua biến — cùng luật `owner`/`status-key` mà Kiểm E đã
      đặt: một biểu thức bị đếm rồi **bỏ qua**, tức mất lưới.

### Task 4 — Bôi đen bằng bàn phím *(AC11 · Quyết định #2 · `deferred-work.md:608`)*

- [x] `tabindex="0"` *(hoặc đường đã chốt)* trên bề mặt chữ của Panel Source.
- [x] Đăng ký command chọn-bằng-phím trong `src/commands/index.ts` + dep tiêm ở `main.ts`
      *(cùng cửa `selectSourceTab`/`runLookup`)*.
- [x] Hợp âm mặc định không đụng: `Mod+1..3` *(chế độ)* · `Mod+Alt+1/2` *(preset)* ·
      `Mod+Alt+←/→` *(vòng panel)* · `Mod+Alt+O/J/V/L` · `Tab` · `⌥←/⌥→` *(Chương trước/sau,
      Story 2.11)* · `⌘⇧…` *(UX-DR35)*.
      🔴 **Và không đụng phím hệ điều hành** — `check:commands` chỉ kiểm trùng **nội bộ**, lưới ở
      đây là **con người** *(bài học `Mod+Alt+H` = "Hide Others", Ice chốt 2026-08-06)*.
- [x] Khoá nhãn `command.selection.*` vào `vi.json` *(NFR16 — Kiểm E đối chiếu hai chiều)*.
- [x] 🔴 **Ghi vào Completion Notes**: `Tab` nay dừng ở thân Panel Source — một thay đổi
      **nhìn thấy được** trên hợp đồng tiêu điểm mà Story 1.14 dặn không chạm.

### Task 5 — Hiệu ứng 90 ms, huỷ được *(AC5 · AC6)*

- [x] Bọc **thân** Panel Lookup *(không `.lookup-head` — nó bất biến, AC7 của 1.17)* bằng bề mặt
      nhận hiệu ứng.
- [x] `opacity` **0.4 → 1**, **90 ms**, `ease-out`. không `translate`, không `scale`, không hiệu ứng ra.
- [x] **Huỷ không khởi động lại** *(AC6)*: hiệu ứng đang chạy + kết quả mới ⇒ **không hiệu ứng nào**,
      `opacity` = 1.
- [x] Cùng hiệu ứng cho **cả bốn trạng thái rỗng** của AC6 Story 1.17 *(`DESIGN.md:347`)*.
- [x] Đối chứng bằng mắt/đo: bấm tra **năm lượt trong một giây** ⇒ không nháy, không chồng.

### Task 6 — Cuộn về đầu tức thì *(AC7)*

- [x] `ref` lên vùng cuộn *(`.lookup-body`, `overflow: auto`)*, đặt `scrollTop = 0` khi kết
      quả mới hiển thị.
- [x] không `scrollIntoView`, không `behavior: 'smooth'`, không `scroll-behavior` trong CSS.
- [x] Đối chứng: cuộn xuống cuối một bản ghi dài *(dùng mật độ của
      `mockups/lookup-real-density.html` — 22 nghĩa, 5 nguồn)* rồi tra chữ khác ⇒ về đầu ngay.

### Task 7 — Vạch tiến trình 250 ms *(AC8 · AC9)*

- [x] Hẹn giờ **250 ms** khởi động cùng lượt phát tra; **huỷ** ở: trả lời · `resetLookupPanel`
      · một lượt tra mới vượt mặt.
- [x] Vạch neo **tuyệt đối** ở đáy `.lookup-head` — 🔴 `.lookup-head` cần `position: relative`
      và **không đổi `--lookup-head-height`**, **không bỏ `overflow: hidden`** *(Bẫy 6)*.
- [x] Màu từ **token** *(`primary` hoặc `ornament` — nó là một **NÉT**, UX-DR5 cho phép)*.
      Màu viết thẳng *(AD-34 §3, Kiểm B của `check:tokens`)*.
- [x] `@media (prefers-reduced-motion: reduce)` ⇒ vạch **TĨNH**, vẫn hiện *(AC9)*.
- [x] Đối chứng: làm chậm giả một lượt tra > 250 ms ⇒ vạch hiện; một lượt < 250 ms ⇒ **không một
      nháy**.

### Task 8 — 🔴 Đo NFR1 đầu-cuối *(AC4 · Quyết định #7)*

- [x] Cài mốc đo *(cờ mặc định TẮT)*, mốc cuối sau `requestAnimationFrame`.
- [x] Chạy **≥ 100** lượt liên tiếp. ⚠️ Truy vấn **khác nhau** nếu dedupe bật *(Quyết định #5)*.
- [x] Ghi bảng: **n · p50 · p95 · p99 · max · engine · cách `invoke` được cấp · bộ truy vấn**.
- [x] 🔴 **Đọc bài học 1.17 trước khi kết luận**: con số `p99 70,742 ms` của 1.17 là **nhiễu
      page-cache của lượt đo đầu**, không một thuộc tính của mã — nó chỉ lộ ra khi **đo lại ba
      lượt độc lập**. ⇒ **Đo lại ít nhất hai lượt độc lập**, và kết luận đứng trên **p99**,
      không chỉ p95.
- [x] Khai giới hạn phép đo **thẳng**: engine nào, IPC thật hay giả lập, món nợ nào không đóng.

### Task 9 — Cổng, sàn, và bàn giao *(AC13 · AC10 · AC12)*

- [x] Nâng **mọi** hằng `*_FLOOR` bị vượt *(danh sách ở AC13)* — số thật ghi vào Completion
      Notes. 🔴 Story 1.17 đã bắt được rằng ba story liên tiếp không ai nâng làm
      `COMPONENT_FILE_FLOOR` tụt xuống ~62% số thật; **không lặp lại**.
- [x] **Chứng minh cổng mới đỏ được** *(Task 3)*: hạ sàn/gỡ một lời gọi ⇒ ĐỎ ⇒ khôi phục ⇒
      XANH. Con số vào Completion Notes.
- [x] **AC10** — đếm điểm ra mạng trên đường tra cứu: **0**. Ghi cách đếm.
- [x] **AC12** — chạy lại phép kiểm `Selection.toString()` của AC6 Story 1.16 trên kiểu **song
      song**, ghi chuỗi vào/ra.
- [x] Đếm lại ba ranh giới không chạm: `matchMedia` **0** · `window.innerWidth` **0** · phụ thuộc
      mới **0**.
- [x] `deferred-work.md` — đóng `:608` và `:635`; **chuyển** `:615`/`:631`/`:633` sang **7.7**
      *(nếu Quyết định #3 = (a))*; thêm mục *"Deferred from: 1-18-auto-lookup"*.
- [x] `src/panels/README.md` — hàng 1.18, và đoạn mô tả hợp đồng vùng chọn.
- [x] `src/commands/README.md` — command chọn-bằng-phím mới *(nếu có)*.

### Task 10 — Nghiệm thu bằng mắt, và nói THẬT cái không nghiệm thu được

- [x] Chạy **chín** cổng DoD lần cuối cùng lượt với toàn bộ thay đổi.
- [x] Bảng chạy tay **có số** vào §Debug Log References *(không văn xuôi — không có bộ chạy test
      frontend, và không được thêm: NFR15, Ice đã chốt và giữ qua **sáu** story)*.
- [x] Liệt kê thẳng những gì **không đánh dấu đạt** *(khuôn Completion Notes của 1.17)*.

---

### Review Findings

> **Cả bốn nhóm đã review và vá xong** *(A: hợp đồng + wiring. B: panel + state. C: cổng
> `check:*`. D: docs/i18n/deferred-work)*. Chín cổng DoD + typecheck chạy lại XANH sau mỗi
> nhóm vá.

**Nhóm A:**

- [x] [Review][Decision] `installTimingProbe()` lách cổng `ALLOWED_GLOBAL_MEMBERS`/AC13 — **Ice chốt: chỉ bật ở dev, tắt hẳn ở production.** Đã vá: `src/main.ts` bọc lời gọi bằng `if (import.meta.env.DEV)`. `Reflect.set(globalThis, …)` giữ nguyên (né Kiểm C có chủ ý, danh sách đó phục vụ Story 4.12), nhưng nó không còn treo trên cửa sổ production thật.
- [x] [Review][Patch] `lookupTiming.ts` không ghép cặp dispatch/paint khi chồng lấn [src/panels/lookupTiming.ts:41] — đã vá: `dispatchedAt` đơn thay bằng hàng đợi FIFO `pending`, `markPainted()` ghép query+mẫu theo cặp
- [x] [Review][Patch] Shift+click có thể phát hai lượt dispatch trùng cho cùng một vùng chọn [src/panels/selectionContract.ts:237] — đã vá: thêm `lastAutoDispatched` bên cạnh `currentQuery.value` trong `shouldDispatch`
- [x] [Review][Patch] `modifySelection()`/`focusSelectionSource()` thiếu try/catch nhất quán với `dispatchLookup()` [src/panels/selectionContract.ts:306] — đã vá: cả hai bọc try/catch, kêu `console.error` đích danh khi ném
- [x] [Review][Patch] Comment biện minh try/catch của `dispatchLookup()` sai về kỹ thuật [src/panels/selectionContract.ts:248] — đã sửa: bỏ khẳng định sai "ném thì gỡ luôn hành vi cho phần còn lại của phiên"

**Nhóm B:**

- [x] [Review][Patch] 🔴 AC12 vỡ — `resolveParallel()` lấy TRỌN `textContent` của một `<span>` mẩu không-Hán dù vùng chọn chỉ chạm MỘT PHẦN [src/panels/SourceHanViet.vue:254] — đã vá: cắt theo `range.startOffset`/`endOffset` thật của text node, không lấy trọn child
- [x] [Review][Patch] 🔴 AC12 vỡ — `resolveSelection()` (kiểu chuyển đổi) cùng lỗi trên đoạn `text` nhiều ký tự [src/panels/SourceHanViet.vue:311] — đã vá: thêm bảng `starts[]` vào `switchView`, cắt `seg.text` theo toạ độ cục bộ
- [x] [Review][Patch] 🔴 AC11 vỡ — `tabindex`/đăng ký đặt trên `.hv-surface` (bọc cả `.hv-notice`/`.hv-sources`) thay vì trên `.hv-switch`/`.hv-parallel`, nên caret bàn phím khởi đầu ở dòng trạng thái/nguồn thay vì văn bản — ca THƯỜNG, không ca biên [src/panels/SourceHanViet.vue:205] — đã vá: đăng ký + `tabindex` chuyển sang đúng đoạn văn bản đang hiện
- [x] [Review][Patch] NFR1/AC4 — một lượt `resetLookupPanel()` giữa lúc đang bay bị đếm nhầm thành một mẫu đo thật [src/panels/LookupPanel.vue:173] — đã vá: chỉ `markPainted()` khi `!neverLookedUp.value` (round-trip thật, không phải huỷ)
- [x] [Review][Patch] Vị trí cuộn không reset khi `resetLookupPanel()` chạy lúc không có lượt tra nào đang bay (vd. tạo Tác phẩm mới trong khi Panel Lookup đang hiện một bản ghi dài) [src/panels/LookupPanel.vue:173] — đã vá: thêm `watch(neverLookedUp, …)` riêng
- [x] [Review][Patch] Comment trỏ sai tên cổng "Kiểm I" (không tồn tại) — cổng thật tên "Kiểm F" [src/panels/AiTranslationPanel.vue:23, src/panels/EditorPanel.vue:15] — đã sửa
- [x] [Review][Patch] Comment overclaim "`resetLookupPanel()` luôn bắt được ca đó" — chỉ đúng khi có lượt đang bay [src/panels/LookupPanel.vue:171] — đã sửa

**Nhóm C:**

- [x] [Review][Patch] 🔴 `SELECTION_SURFACE_FLOOR = 4` không canh được bề mặt THỨ NĂM (`SourceHanViet.vue`, AC11/AC12) — xoá đúng lời gọi đó vẫn còn 4, ĐÚNG sàn cũ, cổng xanh, mất lưới cho toàn bộ đường bàn phím Hán Việt vừa vá ở Nhóm B [scripts/check-commands.mjs:1635] — đã vá: nâng sàn lên 5 (số thật), kèm lý do tại chỗ
- [x] [Review][Patch] Ba chỗ gỡ `⛔` bị thay bằng chữ "không" thay vì xoá hẳn, làm câu chẩn đoán cổng đọc thành phủ định kép/vô nghĩa ("không Sàn AA không phải…", "không Và đừng thêm…", "không Loại một cặp…") [scripts/check-tokens.mjs:730,1065,1090] — đã sửa: bỏ chữ "không" thừa, đúng luật 3/5 của lượt gỡ ký hiệu 2026-08-07
- [x] [Review][Defer] Bộ đếm Kiểm F dựa trên `p.masked` — `maskScript`/`maskTemplate` chỉ che comment (`//`, `/* */`, `<!-- -->`), KHÔNG che nội dung chuỗi literal/template literal, nên một chuỗi giả dạng lời gọi (`"useSelectionSurface(original, 'source')"` nằm trong văn bản, không phải mã) vẫn bị đếm là một lượt đăng ký thật — deferred, pre-existing (đặc tính chung của mọi cổng regex trong tệp này, không riêng Kiểm F; vá đúng nghĩa là đổi `maskScript`/`maskTemplate` toàn cục, ngoài phạm vi story này)
- [x] [Review][Defer] `SURFACE_CALL_RE` không khớp dạng gọi thay thế (`registerSelectionSurface` trực tiếp, đối số đầu chứa dấu phẩy, role viết sai hoa/thường) — deferred, chưa có ca thật nào trong mã hôm nay dùng các dạng đó; cùng lớp giới hạn với mọi cổng regex khác trong tệp (NFR15 cấm phụ thuộc một bộ phân tích cú pháp thật)
- [x] [Review][Defer] `SELECTION_PANEL_FILES` là danh sách chép tay từ `workspaceLayout.ts`, không tự đồng bộ khi có panel Workspace mới (Story 3.4…) — deferred, cùng khuôn với `PANEL_SUFFIXES` đã dùng ở nơi khác trong tệp, panel mới ngoài phạm vi story này

**Nhóm D:**

- [x] [Review][Patch] `src/panels/README.md` thiếu hàng 1.18 ở bảng "Ranh giới sở hữu" (Task 9 khai đã làm) — chỉ có đoạn mô tả hợp đồng, không có hàng bảng — đã thêm
- [x] [Review][Patch] `src/panels/README.md` §Hợp đồng vùng chọn ghi "sàn 4" — lệch với sàn thật sau lượt vá Nhóm C (5) — đã sửa, kèm giải thích tại sao sàn từng sai
- [x] [Review][Patch] Năm chỗ gỡ `⛔` bị thay bằng chữ "không" thay vì xoá hẳn, làm câu đọc thành phủ định kép/vô nghĩa hoặc **đảo ngược nghĩa một trích dẫn** — nặng nhất: `deferred-work.md:581` trích `prd.md:922` thành *"Còn bản quyền · không Đã loại"*, đảo ngược trạng thái thật của `prd.md` (là "Đã loại") ngay trong một đoạn đang giải thích rằng story sau ĐẢO quyết định "Đã loại" đó [deferred-work.md:152,154,251,313,581] — đã sửa cả năm
- [x] [Review][Patch] Hai chỗ gỡ `⛔` tương tự trong `commands/README.md` — một tiêu đề mục ("### không Vì sao…") và một câu liệt kê mẫu cấm ("không `@click=...` · … đều là FAIL") [src/commands/README.md:18,64] — đã sửa
- [x] [Review][Patch] `command.selection.focus_source` dùng từ nội bộ kiến trúc "bề mặt văn bản" (thuật ngữ của `selectionContract.ts`), lệch giọng văn cụ thể/dễ hiểu của các nhãn command lân cận [src/i18n/vi.json:32] — đã đổi thành "Đặt con trỏ vào văn bản"

---

## Dev Notes

### Trạng thái repo hôm nay — SỐ, không phải mô tả *(đo 2026-08-07, `4136f3f`)*

| | Số thật |
|---|---|
| `cargo test` | **225 xanh** |
| Cổng `check:*` xanh | **7/7** *(+`npm run build` + `cargo test` = **9** lệnh DoD)* |
| `#[tauri::command]` đã đăng ký | **7** — `bootstrap_config` · `put_config` · `create_work_from_text` · `create_work_from_file` · `read_open_chapter` · `read_han_viet` · `lookup_dictionary` |
| **Đường nghe vùng chọn** | 🔴 **không CÓ** — story này dựng nó |
| Chỗ gọi `window.getSelection()` | **1** — `src/main.ts:182` *(dep TỐI THIỂU của 1.17)* |
| Command trong `CommandRegistry` | **17** *(3 mode · 2 preset · 4 toggle · 2 focus · 2 library · 3 source · 1 lookup)* |
| Lời gọi `dispatch()` literal | **12** *(4 lời gọi truyền biến — cổng đếm rồi in ra)* · `@click`: **8** |
| Điểm vào focus đã khai | **7** *(3 chế độ + 4 panel)* |
| Tệp `.vue` / `.ts` / `src/**` quét được / `.rs` | **13** / **24** / **37** / **40** |
| Khoá `vi.json` | **75** |
| Token typography | **17** *(thứ 17 là `ui-md-wrap`, Story 1.17 §Quyết định #7)* |
| `matchMedia` / `window.innerWidth` trong `src/**` | **0** / **0** — 🔴 **phải giữ nguyên** |
| `animation:` / `@keyframes` do **ta** viết trong `src/**` | 🔴 **0** — story này là **người đầu tiên**. *(3 biến `--dv-*-transition-*` ở `dockview-theme.css:139-141` là của **thư viện**, không của ta)* |
| `scroll-behavior` trong `src/**` | **0** — 🔴 phải giữ nguyên *(AC7)* |
| `ALLOWED_GLOBAL_MEMBERS` | **11** mục — `window.{addEventListener, removeEventListener, getSelection}` · `document.{documentElement, activeElement, body, fonts, getElementById, createElement, addEventListener, removeEventListener}` |
| Tệp `.db` từ điển trong git | **0** *(`.gitignore: *.db`, AD-25)* — mọi bản dựng hôm nay lên với **0 lớp** ⇒ đường tra cứu trả `layers_loaded: false` |

### API thật — chép từ MÃ, không từ trí nhớ

```ts
// src/panels/lookupPanelState.ts — không SỬA (§KHÔNG-LÀM ①)
export async function runLookup(rawQuery: string): Promise<void>   // trim + sequence guard
export function resetLookupPanel(): void                           // sequence += 1, vứt state
export const currentQuery:      DeepReadonly<Ref<string | null>>   // truy vấn ĐANG HIỆN
export const lookupPending:     DeepReadonly<Ref<boolean>>         // đang bay
export const lookupError:       DeepReadonly<Ref<IpcError | null>>
export const lookupResolved:    ComputedRef<boolean>   // không pending && không error && có response
export const lookupDisplayable: ComputedRef<boolean>   // không error && có response (GIỮ khi chờ)
export const neverLookedUp / notFound / queryTooShort / layersLoaded
export const someLayerFailed / someLayerTruncated / queryWasTruncated
export const groupedLookup / sensesByLayer

// src/commands/index.ts — điểm cắm của story này
export type CommandDeps = { …, runLookup?: (q: string) => void, currentSelection?: () => string }
export function dispatch(id: CommandId): void          // NÉM với id chưa đăng ký
export function installCommands(deps: CommandDeps): Keymap   // gọi MỘT lần, trước mount()

// src/main.ts:182 — DÒNG STORY NÀY THAY
currentSelection: () => window.getSelection()?.toString() ?? '',
```

```css
/* src/panels/LookupPanel.vue — hai bất biến không ĐƯỢC PHÁ */
.lookup-head { --lookup-head-height: 76px; flex: none; height: var(--lookup-head-height);
               overflow: hidden; }          /* AC7 của 1.17 — không đổi một pixel */
.lookup-body { display: flex; flex-direction: column; height: 100%;
               min-height: 0; overflow: auto; }   /* ← vùng cuộn của AC7 story này */
```

### Doctrine đã chốt ở 1.14/1.16/1.17 mà story này **thừa kế nguyên**

- **`src/commands/**` không được `import` Vue, không `@tauri-apps/api`.** `check-commands.mjs`
  Kiểm C/D/E nạp thư mục đó bằng **Node thuần**. Hướng phụ thuộc **một chiều**:
  `panels/` → `commands/`. Mọi handler phụ thuộc trạng thái **TIÊM VÀO** qua `CommandDeps`,
  nối ở `src/main.ts`.
- **State panel sống module-level, không trong `<script setup>`.** Một lượt đổi preset gọi
  `api.clear()` rồi dựng lại **cả bốn** panel — `ref` cục bộ chết cùng lượt đó.
  ⇒ **Hợp đồng vùng chọn cũng phải là module-level**, và lượt đăng ký của panel phải
  **idempotent** qua mount/unmount.
- **Mọi `export` phải có một chỗ TIÊU THỤ nhìn thấy được.** Một hàm `export` mà không ai `import`
  là lỗi **im lặng hoàn toàn** — đã xảy ra **hai lần** *(1.16 `sourceChapterError`, 1.17
  `lookupError`)*, cả hai tốn một lượt code review.
- **`owner` / `status-key` viết LITERAL ở chỗ gọi.** Kiểm E đọc **tĩnh**. Một biểu thức bị
  đếm rồi **bỏ qua** = mất lưới. ⇒ áp cùng luật cho lời gọi đăng ký vùng chọn *(Task 3)*.
- **`dockview-vue` mount mọi component với đúng MỘT prop tên `params`** — không khai prop khác.
- **Cổng vắng ⇒ KÊU** *(`portMissing`, `console.error` nêu đích danh)*, không ném và không im.
- **Rỗng im lặng bị cấm; rỗng CÓ LÝ DO thì không** *(AD-44 ④)*.

### ⚠️ MƯỜI CÁI BẪY — tám trong mười cho ra một lượt CI **XANH** với kết quả **VÔ NGHĨA**

1. 🔴 **VÒNG TỰ THAY THẾ — nguy hiểm nhất của story.** Panel Lookup **tự nó chứa chữ**
   *(nghĩa, ví dụ, trích dẫn — `LookupRecord.vue`)*. Một listener `document` không lọc nguồn ⇒
   bôi đen một nghĩa để đọc kỹ ⇒ **một lượt tra mới thay chính đoạn đang đọc**, cộng một hiệu
   ứng 90 ms, cộng một lượt cuộn về đầu. Người dùng mất chỗ, và không hiểu vì sao. **Không**
   test nào hôm nay bắt được; không cổng nào nhìn thấy. ⇒ **AC3**, và đó là lý do Quyết định #1
   chọn **OPT-IN**.
2. **Nghe `mouseup` trên gốc PANEL thay vì trên `document`.** Kéo chọn từ trong panel rồi thả
   chuột **ngoài** panel là thao tác **thường ngày** *(và gần như bắt buộc khi chọn tới cuối
   một khối)*. Listener trên gốc panel không bao giờ thấy `mouseup` đó ⇒ vùng chọn **không được tra**,
   không lỗi, không dấu hiệu. ⇒ listener trên `document`, vị từ đọc **`anchorNode`**.
3. 🔴 **Nghe `selectionchange` mà không chờ dừng** ⇒ *"tra hàng loạt cụm dở dang"*
   *(`mockups/motion-auto-lookup.html:200` cấm đích danh)*. Mỗi lần kéo qua một ký tự là một
   lượt IPC. **CI vẫn xanh**, NFR1 vẫn *"đạt"* trên từng lượt, và sản phẩm giật.
4. 🔴 **Cài AC6 bằng cách KHỞI ĐỘNG LẠI hiệu ứng.** `:key` lại node, hoặc gỡ-rồi-gắn class,
   trông **giống** *"huỷ hiệu ứng cũ"* và **là** thứ ngược lại: tra liên tiếp cho ra một chuỗi
   nháy 0.4 liên tục. Mockup nói thẳng: *"Tra nhanh liên tiếp thì thành **không có hiệu ứng**"*.
5. **Xoá Panel Lookup khi vùng chọn rỗng.** Một cú **bấm** cũng bắn `mouseup`. Xoá ở đó nghĩa
   là kết quả biến mất mỗi lần người dùng bấm vào văn bản để đọc tiếp — không lỗi, không cổng nào đỏ,
   và nó phá đúng thứ FR21 vừa mua.
6. 🔴 **Vạch tiến trình đẩy `.lookup-head` cao thêm.** Hằng `--lookup-head-height: 76px` +
   `overflow: hidden` là **cơ học** của bất biến AC7 Story 1.17 *(và bản đầu của 1.17 đã vỡ
   đúng chỗ này: bọc `.lookup-head` trong `v-if` làm chiều cao đi 0 → 76px)*. Một vạch `2px`
   thêm vào **luồng** ⇒ 78px ⇒ **đầu mục và thanh nhịp dịch chỗ mỗi lượt tra chậm** — đúng
   *"layout nhảy"* mà `DESIGN.md:336` gọi là **thủ phạm gây giật**.
7. **Đo NFR1 tới `nextTick` rồi báo cáo như đã đo tới *"hiển thị"*.** `nextTick` là *"Vue đã
   ghi DOM"*, không *"trình duyệt đã vẽ"*. Với 22 nghĩa / 5 nguồn, khoảng cách đó không nhỏ.
   ⇒ Quyết định #7.
8. 🔴 **Kết luận NFR1 trên MỘT lượt đo.** 1.17 đo `p99 70,742 ms` ở lượt đầu và **không tái lập
   được** — ba lượt đo lại cho **0,566 / 1,136 / 1,793 ms**. Nguyên nhân: **nhiễu page-cache**.
   ⇒ không đóng story trên một con số chưa hiểu; đo lại ≥ 2 lượt độc lập.
9. **`Selection.modify('extend', …, 'word')` trên văn xuôi tiếng Trung.** Tiếng Trung không có
   khoảng trắng giữa từ; `word` có thể nhảy qua **cả câu** hoặc **không nhảy**. Đây là ca **phải
   đo** ở Task 0, không ca biên — nó là **ngôn ngữ chính** của sản phẩm.
10. **Thêm `window.matchMedia` vào `ALLOWED_GLOBAL_MEMBERS` để đọc `reduced-motion`.** Nó mở
    đúng cánh cửa mà §⑥ của Story 1.17 vừa khoá cho Story 4.12, và nó không mua thêm gì so với
    một `@media` trong CSS. ⇒ Quyết định #6.

### 🔴 BA mâu thuẫn tài liệu đã phát hiện — không dev KHÔNG sửa tài liệu, chỉ NÓI RA

1. **`deferred-work.md:615` giao `Substring` cho *"Story 1.18 hoặc 7.7"*; `epics.md` §Story
   1.18 không nhắc một chữ nào về chuỗi con.** ⇒ **`epics.md` thắng** *(nó là tài liệu nghiệm thu
   của story)*; `:615` là một **phỏng đoán** của tác giả 1.17, không một AC. Quyết định #3 chốt
   dứt điểm, và **ghi ra** dù chốt đường nào.
2. **`DESIGN.md:346` nói vạch tiến trình *"ở đáy vùng đầu mục"*; `LookupPanel.vue` khoá vùng
   đó `overflow: hidden` + chiều cao cố định.** Hai mệnh đề không mâu thuẫn về **ý định**, nhưng
   mâu thuẫn về **cơ học**: *"ở đáy"* đọc tự nhiên thành *"trong luồng, dưới cùng"*, mà đường
   đó phá AC7 của 1.17. ⇒ neo **tuyệt đối**, ghi lý do tại chỗ *(Bẫy 6)*.
3. **`epics.md` §Story 1.18 AC nói *"kết quả tra cứu hiện ở Panel Lookup"*; UX-DR15 cho phép
   Panel Lookup **bị ẩn**** *(nó nằm trong `SACRIFICE_ORDER`)*. Khi panel đang ẩn, một lượt
   Auto-Lookup cập nhật state mà không ai thấy. ⇒ **không tự động hiện lại panel** *(đó là hành vi
   thanh trạng thái của **Story 4.12**)*; ghi ra là một ca **đã biết, có chủ**.

### Bàn giao — NĂM mục `deferred-work.md` gọi đích danh Story 1.18

| Dòng | Mục | Story này làm gì |
|---|---|---|
| `:608` | **Bôi đen bằng BÀN PHÍM không cài** — *"Ice chốt: ghi nợ cho 1.18"* | ✅ **Đóng** *(AC11, Quyết định #2, Task 4)* — 🔴 **phần đắt nhất** |
| `:635` | `window.getSelection()` mù với ô nhập; vùng chọn rỗng = im lặng tuyệt đối | ✅ **Đóng** *(AC3, Task 1 — loại trừ **cơ học**, không dựa vào hành vi không chuẩn hoá)* |
| `:615` | `query_too_short` không thực thi được qua đường `Exact`-only | ⚠️ **Chuyển 7.7** nếu Quyết định #3 = (a). không đóng, không im lặng bỏ |
| `:631` | Nhánh `Substring` nạp toàn bộ hàng vào RAM trước khi cắt | ⚠️ **Chuyển 7.7** — latent tới ngày `Substring` bật |
| `:633` | Chuỗi `query_too_short` chỉ một thao tác không tồn tại trong panel | ⚠️ **Chuyển 7.7** — cùng lượt với `:615` |

**không đóng, kế thừa nguyên** *(không phải món nợ của story này)*: vòng IPC Tauri **thật** chưa đo
*(1.17)* · nghiệm thu thị giác **hai nền tảng thật** *(1.6/1.14/1.16/1.17)* · hình dạng hiển
thị mục từ **tiếng Anh** `:317` *(chủ: Sally, UX)* · `.parallel-note` đổi cỡ 11,5→12px chưa
nghiệm thu trên máy thật.

### 🧠 Trí tuệ từ story trước — thứ đắt tiền, đừng học lại bằng tiền

- **1.17 · code review:** *"một `export` không ai `import` là lỗi im lặng hoàn toàn"* — tái phát
  **nguyên văn** từ 1.16 sang 1.17. ⇒ ở đây: mỗi hàm của hợp đồng phải có chỗ tiêu thụ
  **nhìn thấy được**, và Task 3 dựng **một cổng** thay vì tin vào mắt.
- **1.17 · Task 0/Task 8:** **đo lật ngược giả định của chính story** — nhánh `char_idx` hoá
  ra không được đường sản phẩm chạm tới. ⇒ ở đây: **đo `Selection.modify()` trước khi chốt
  Quyết định #2**, không chốt rồi mới phát hiện WKWebView làm khác.
- **1.17 · code review:** một lượt IPC đang bay ghi đè kết quả của lượt mới hơn — vá bằng
  **bộ đếm `sequence`**. 🔴 Auto-Lookup **biến lỗ đó thành thường trực** *(chú thích tại chỗ
  đã ghi đúng chữ này)*. ⇒ **đụng** cơ chế đó; và Task 5/7 phải dùng **cùng** bộ đếm cho
  hiệu ứng và hẹn giờ, không dựng bộ đếm thứ hai.
- **1.16 · code review:** state module-level không có đường vô hiệu hoá ⇒ Tác phẩm B hiện dữ liệu
  của A. ⇒ hợp đồng vùng chọn **cũng** là state module-level: hỏi *"cái gì phải chết khi Tác
  phẩm đổi"* **trước** khi viết dòng đầu.
- **1.16 · Task 8 / AC6:** `display: flex` bên trong `.hv-unit` làm Chromium chèn `\n` vào
  `Selection.toString()`. ⇒ **AC12** chạy lại phép kiểm đó; **không tin số đo cũ** khi story này
  chạm bề mặt ấy hàng trăm lần mỗi Chương.
- **1.16 · code review:** một lượt trượt khoá **vĩnh viễn** đường tra vì cờ `requested` không được
  nhả ở nhánh lỗi. ⇒ hẹn giờ 250 ms và cờ hiệu ứng phải nhả ở **mọi** nhánh thoát.

### Testing standards

Bộ DoD **chín lệnh** *(khuôn 1.14→1.17)* — **mã thoát là phán quyết**, không đầu ra:

```
cargo test --manifest-path src-tauri/Cargo.toml
npm run build            npm run check:tokens     npm run check:i18n
npm run check:commands   npm run check:layout     npm run check:deps
npm run check:dict-manifest                       npm run check:scope
```

- 🔴 **không có bộ chạy test frontend, và không được thêm** *(NFR15 — Ice chốt ở 1.5, giữ qua **sáu**
  story)*. ⇒ **toàn bộ** vế DOM của story này *(vùng chọn, hiệu ứng, cuộn, vạch tiến trình,
  bàn phím)* nghiệm thu bằng **bảng chạy tay CÓ SỐ** trong §Debug Log References — không bằng văn
  xuôi, không bằng *"đã kiểm bằng mắt"*.
- ⚠️ Story này **gần như không có bề mặt Rust** ⇒ `cargo test` không canh được gì cho nó. **Lưới duy
  nhất chạy tự động là các cổng `check:*`** — đó là lý do **AC2 đòi một cổng mới** *(Task 3)*
  và **AC13 đòi nâng sàn**.
- **Đỏ-rồi-xanh cho mọi cổng bị đụng**: mỗi mệnh đề mới phải có **ít nhất một ca làm cổng ĐỎ**
  cộng **một đối chứng âm**. Con số vào Completion Notes.
- Nếu có thêm test Rust *(chỉ khi Quyết định #3 = (b))*: test **HÀNH VI qua biên**, đặt ở
  `src-tauri/tests/**`, **dùng lại** fixture của `dict_sources.rs`/`dict_lookup.rs`.

### Project Structure Notes

```
src/
  panels/selectionContract.ts   NEW     🔴 hợp đồng vùng chọn — module-level, OPT-IN
                                        KHÔNG import vào src/commands/**
  panels/SourcePanel.vue        UPDATE  đăng ký nguồn + tabindex/bàn phím (Quyết định #2)
  panels/SourceHanViet.vue      UPDATE  ⚠️ nếu bề mặt Hán Việt cũng là nguồn — xem Câu hỏi #2
  panels/LookupPanel.vue        UPDATE  hiệu ứng 90ms · cuộn về đầu · vạch 250ms
                                        🔴 không ĐỔI `--lookup-head-height`, không bỏ `overflow: hidden`
  panels/AiTranslationPanel.vue UPDATE  🔴 CHỈ một lời gọi đăng ký (AC2) — không nội dung
  panels/EditorPanel.vue        UPDATE  🔴 CHỈ một lời gọi đăng ký (AC2) — không nội dung
  commands/index.ts             UPDATE  command chọn-bằng-phím + dep mới (Task 4)
  main.ts                       UPDATE  🔴 thay dep `currentSelection` + gắn watcher SAU mount()
  i18n/vi.json                  UPDATE  nhãn `command.selection.*`
  panels/README.md              UPDATE  hàng 1.18 + đoạn hợp đồng vùng chọn
  commands/README.md            UPDATE  command mới
scripts/check-commands.mjs      UPDATE  🔴 CỔNG MỚI đếm lời gọi đăng ký (sàn 4) + nâng sàn
scripts/check-layout.mjs        UPDATE  ALLOWED_GLOBAL_MEMBERS + dòng lý do AC + FILE_FLOOR
scripts/check-tokens.mjs        UPDATE  FILE_FLOOR / COMPONENT_FILE_FLOOR
scripts/check-i18n.mjs          UPDATE  VUE_FLOOR / RS_FLOOR
_bmad-output/implementation-artifacts/deferred-work.md   UPDATE  đóng :608 :635 · chuyển 3 mục
```

⚠️ **không tệp Rust nào trong danh sách** — và đó là một **dữ kiện của story**, không một thiếu sót:
toàn bộ tầng dữ liệu đã đúng *(§KHÔNG-LÀM ①)*. Nếu bản cài của bạn chạm `src-tauri/**` mà
Quyết định #3 = (a), **dừng lại** và đọc lại phạm vi.

### 📌 Bối cảnh git

`4136f3f` *(1.17 — Panel Lookup hoàn chỉnh + token thứ 17)* · `cb03974` *(1.10c/1.16)* ·
`564be15` *(1.15)* · `c3efb20` *(1.14 — bốn panel + dockview)* · `7e38de8` *(test `core::matching`)*.

🔴 **Cây làm việc không SẠCH.** `src/modes/libraryImport.ts` mang một bản vá **chưa commit**:
`finishSubmit` gọi thêm `ensureChapterLoaded()` sau `resetSourcePanel()`/`resetLookupPanel()`.
Lý do ghi trong chính bản vá — ba chế độ sống trong `<KeepAlive>` nên lượt hiện thứ hai không có
`mounted`, và Panel Source ở lại *"Chưa có Chương nào được mở"* **vĩnh viễn** *(bắt bằng test
tay 2026-08-07)*. ⇒ **Quyết định ở Task 0**: commit riêng trước khi bắt đầu, hay cuốn vào
story này. **Đừng bỏ qua nó** — nó nằm ở đúng điểm nghẽn mà story này sẽ chạm.

**Đọc gì trước khi gõ:**
`src/panels/lookupPanelState.ts` *(trọn — 299 dòng; bộ đếm `sequence`, `lookupDisplayable` vs
`lookupResolved`, `resetLookupPanel`)* · `src/panels/LookupPanel.vue` *(trọn — bất biến
`.lookup-head`)* · `src/panels/SourceHanViet.vue` *(trọn — **vì sao** `Selection.toString()`
đúng ở kiểu song song; không phá nó)* · `src/main.ts:163-187` *(cửa tiêm dep)* ·
`src/commands/index.ts:126-216, 458-479` *(hợp đồng `CommandDeps` + command `lookup.*`)* ·
`scripts/check-layout.mjs:389-460` *(Kiểm C — danh sách cho phép)* ·
`scripts/check-commands.mjs:194-243` *(khuôn sàn — mẫu để dựng cổng mới)* ·
`DESIGN.md` §Motion *(bảng chín hàng)* · `mockups/motion-auto-lookup.html`
*(ba cặp đúng/sai — **đây là bản vẽ hành vi, không một gợi ý**)*.

### 🌐 Phiên bản đang ghim — KHÔNG đổi một dòng nào

`tauri 2.11.1` *(api)* / `2.11.4` *(cli)* · `vue 3.5.40` · `dockview-vue 7.0.4` ·
`vite 8.2.0` · `typescript 5.9.3` · `vue-tsc 3.3.9` · `rusqlite` bundled.
**0 phụ thuộc mới, npm lẫn crate** (NFR15) — mọi phụ thuộc mới phải qua rà GPLv3 và vào
bảng Stack **trước**, và `check-deps.mjs` có danh sách cấm cùng ngưỡng sàn.

⚠️ **Web API story này dựa vào, và độ chắc của từng cái:**

| API | Độ chắc | Ghi chú |
|---|---|---|
| `window.getSelection()` · `Selection.toString()` | ✅ chuẩn, đã dùng thật ở 1.16/1.17 | — |
| `Selection.anchorNode` · `Node.contains()` | ✅ chuẩn | vị từ thuộc-nguồn |
| CSS `@media (prefers-reduced-motion)` | ✅ chuẩn | Quyết định #6 |
| CSS `animation` / `@keyframes` | ✅ chuẩn | ⚠️ **0 `animation`/`@keyframes` do ta viết trong `src/**` hôm nay** — story này là người đầu tiên |
| `Element.animate()` *(WAAPI)* | 🟡 có ở cả hai engine | dùng nếu cần `.cancel()` tường minh *(AC6)* |
| 🔴 **`Selection.modify()`** | 🔴 **không CHUẨN** | có ở Chromium **và** WebKit — nhưng **PHẢI ĐO** ở Task 0, không tin bảng |

### References

- `epics.md:1752-1805` — **Story 1.18, chín mệnh đề AC** *(nguồn nghiệm thu)* · `:1706+` 1.17
  *(ranh giới trên)* · `:1807+` 1.19 · `:1845+` 1.20 · `:1881+` 1.21
- `epics.md:106` **FR21** · `:324` **NFR1** *(*"từ lúc thả chuột"* + ngân sách 99,95 ms cho
  IPC + render)* · `:641` · `:762` · `:807-809` *(bảng FR/NFR của Epic 1)*
- `ARCHITECTURE-SPINE.md:406` **AD-34** *(mọi thao tác qua CommandRegistry; sàn NFR17 là **cấu
  trúc**)* · `:322` AD-24 · `:206` AD-15 · `:290` AD-19 · `:302` AD-21 · `:571` AD-44 ·
  `:635` *(bảng Hard Rules — §Thao tác giao diện, §Màu)*
- `DESIGN.md:334-350` — 🔴 **§Motion, bảng chín hàng = đặc tả nguyên văn của AC5–AC9** ·
  `:208-219` *(§Sàn tương phản — luật `opacity`, vì sao 0.4→1 được phép)* · `:360` §Components
- `EXPERIENCE.md:131` *(FR21 — *"tương tác lặp nhiều nhất trong sản phẩm … **không được thiết kế
  lại cho khác đi**"*)* · `:272` *(cảnh dùng thật)* · `:322` *(bảng mockup → `motion-auto-lookup`)*
  · `:389` §Trạng thái rỗng
  🔵 **2026-08-13 — con trỏ này nay trỏ vào một văn bản KHÁC.** Mệnh đề *"không được thiết kế
  lại cho khác đi"* đã được **sửa**, không phải bị vượt: nó chặt hơn nguồn của chính nó
  *(`prd.md:56` chỉ nói "lệch **quá xa** sẽ bị từ chối")* và mâu thuẫn với lượt Ice **loại
  hướng C "Kế thừa QuickTranslator"** ngày 2026-08-02. Bản mới: *QuickTranslator là mốc tham
  khảo để vượt qua* — bất biến là **thao tác**, **cài đặt** thì mở. Cùng lượt đó FR21 thu hẹp
  còn **Panel Source**. ⇒ Câu trích ở trên là **ảnh chụp 2026-08-07**, đừng trích lại nó như
  một mệnh đề đang hiệu lực. Xem `planning-artifacts/sprint-change-proposal-2026-08-13.md`.
- `mockups/motion-auto-lookup.html:185-205` — **bảng đặc tả chuyển động** · `:187`
  *(*"Tra nhanh liên tiếp thì thành không có hiệu ứng"*)* · `:200` *(*"không tra khi con trỏ còn
  đang kéo"*)*
- `deferred-work.md:608` *(bàn phím — **món nợ chính**)* · `:615` · `:631` · `:633` · `:635`
- `src/panels/lookupPanelState.ts:132-140, 244-299` *(bộ đếm `sequence` + `resetLookupPanel` —
  **không sửa**)* · `src/panels/LookupPanel.vue:74-80, 156-170` *(bất biến `.lookup-head`)* ·
  `src/panels/SourceHanViet.vue:1-24, 275-331` *(vì sao vùng chọn đúng ở kiểu song song)* ·
  `src/main.ts:163-187` · `src/commands/index.ts:458-479` · `scripts/check-layout.mjs:389-412`

---

## Câu hỏi cho Ice

> ⚠️ Cả bốn câu **không chặn Task 0** — mỗi câu có một mặc định chạy được. Nhưng câu **#1** đổi
> **phạm vi** story, nên trả lời sớm là rẻ hơn.

1. 🔴 **`LookupMode::Substring` — có bật ở story này không?** *(Quyết định #3)*
   `deferred-work.md:615` **đoán** rằng 1.18 sẽ bật; `epics.md` §Story 1.18 **không nhắc một chữ
   nào**. Mặc định story chọn: **không bật**, ba mục `deferred-work` chuyển sang **7.7**.
   ⇒ Chốt ngược làm story phồng thêm: một trần an toàn ở SQL, một chuỗi `vi.json` phải sửa,
   một lượt verify bằng mắt, **và** một lượt đo NFR1 thứ hai trên đường tiếng Anh *(AD-44 ⑥)*.

2. **Bôi đen ở tab Hán Việt kiểu *chuyển đổi* — tra cái gì?** Ở kiểu **song song**, vùng chọn
   trả về **ký tự Hán** *(đúng, đã đo ở 1.16)*. Ở kiểu **chuyển đổi**, màn hình chỉ có **âm
   Hán Việt** *(chữ Latin)* ⇒ vùng chọn trả `"bắc lương"` ⇒ AD-44 định tuyến sang **đường
   tiếng Anh** ⇒ **không tìm thấy**.
   - **(a) mặc định story:** vẫn là nguồn; người dùng đọc *"không tìm thấy trong từ điển"* — một
     trạng thái **có tên**, không im lặng.
   - **(b)** không đăng ký `.hv-switch` ⇒ bôi đen ở đó **không làm gì cả** — im lặng tuyệt đối, đúng
     thứ AD-44 ④ cấm.
   - **(c)** ánh xạ ngược âm → ký tự — 🔴 **không đề xuất**: đó là một quy tắc nghiệp vụ mới ở
     webview *(AD-1)*, và nó là **FR113 / Story 3.7**.

3. **`Tab` nay dừng ở thân Panel Source — chấp nhận không?** *(Quyết định #2, hệ quả nhìn thấy
   được)* Hôm nay `PanelFrame` mang `tabindex="-1"` và không vào vòng `Tab`; đường đi giữa panel
   là `Mod+Alt+←/→`. Đóng `:608` bằng đường (a) thêm **một** điểm dừng `Tab` cho mỗi bề mặt
   chữ. Đó là **giá** của việc bôi đen bằng bàn phím, và story **không giấu nó**.

4. **Vạch tiến trình 250 ms dùng token màu nào — `primary` hay `ornament`?** UX-DR5 nói
   `ornament` là màu của **nét** *(không bao giờ của chữ)*, và vạch này **là** một nét ⇒ `ornament`
   đúng luật. Nhưng `DESIGN.md` §Do's dành `primary` cho **đúng ba việc** *(thuật ngữ Glossary,
   nhãn nguồn, tiêu điểm bàn phím)* — một vạch tiến trình không nằm trong ba việc đó.
   ⇒ **Mặc định story: `ornament`.** Ghi ra để Ice bác nếu muốn.

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (`claude-opus-5`), Claude Code.

### Debug Log References

#### ① Task 0 — `Selection.modify()` trên HAI engine *(19 phép thử, hai engine khớp từng dòng)*

Bộ đo: `WKWebView` dựng bằng Swift *(`swiftc`, đúng lớp Tauri dùng trên macOS — không Safari,
vì Safari khác cấu hình)* · Chromium qua `Chrome --headless --dump-dom` *(engine WebView2)*.

| # | Phép thử | WKWebView | Chromium |
|---|---|---|---|
| 1 | `Selection.modify` tồn tại trên `<p tabindex="0">` không sửa được | `function` | `function` |
| 2 | `tabindex="0"` nhận `el.focus()` | ĐẠT | ĐẠT |
| 3 | `extend·right·character` ×3 | `他打開` | `他打開` |
| 5 | 🔴 `extend·right·**word**` ×1 *(văn xuôi tiếng Trung)* | `"他"` **1 ký tự** | `"他"` **1 ký tự** |
| 6 | `extend·right·word` ×2 *(tiếng Trung)* | `"他打開"` **3 ký tự** | `"他打開"` **3 ký tự** |
| 7 | `extend·right·word` ×1 *(Latin)* | `"The"` | `"The"` |
| 8 | `extend·**left**·character` ×2 | `了那` | `了那` |
| 10 | `toString()` trong `<input>` rời | **`"nội "`** · `anchorNode=BODY` | **`"nội "`** · `anchorNode=BODY` |
| 11 | ô nhập LỒNG trong bề mặt — `host.contains(anchorNode)` | `anchorNode=DIV` · **`true`** | `anchorNode=DIV` · **`true`** |
| 13 | vị từ A *(`activeElement`)* · vùng chọn thật | **TRƯỢT** *(âm tính giả)* | **TRƯỢT** |
| 15–18 | vị từ B *(`nodeType`)* · bốn ca | **ĐẠT ×4** | **ĐẠT ×4** |
| 19 | vùng chọn RỖNG *(một cú bấm)* | `toString() === ''` | `toString() === ''` |

🔴 **Ba kết luận lật mệnh đề của story:**
① **Bẫy 9 không CÓ THẬT** — `'word'` phân đoạn tiếng Trung ĐÚNG trên cả hai *(ICU)*, không nuốt câu.
② **`deferred-work.md:635` SAI** — `toString()` trong `<input>` trả văn bản THẬT, không `''`.
③ Mệnh đề *"`anchorNode` nằm trong `<input>` ⇒ không nguồn"* của Task 1 **không cài được như đã
viết**: `anchorNode` không bao giờ nằm trong ô nhập. Vị từ giao đọc **`nodeType`**.

#### ② AC12 — vùng chọn ở tab Hán Việt, chạy LẠI *(không tin số đo 1.16)*

Vùng chọn dựng bằng `Selection.modify()` — thuật toán chọn của chính trình duyệt, tức đường
mà **AC11 của story này** vừa tạo ra.

| Vùng chọn cả đoạn, kiểu SONG SONG | Chromium | **WKWebView** |
|---|---|---|
| `Selection.toString()` | ✅ `他打開了那扇門，走進了黑暗之中。` | ❌ **`他tha打đả開khai了liễu…`** |
| đọc node `.hv-char` *(vị từ đã giao)* | ✅ đúng | ✅ đúng |
| không chèn `\n` *(bẫy hộp dòng, AC6 của 1.16)* | 0 ký tự | 0 ký tự |

🔴 **`user-select: none` không ràng buộc `Selection.modify()` trên WebKit.** Số đo Playwright của
Story 1.16 *(kéo chuột)* vẫn đúng; story này tự tạo ra một đường thứ hai mà nó không đúng ⇒
`resolveParallel` đọc node, không `toString()`.

| Kiểu CHUYỂN ĐỔI — ánh xạ ngược theo VỊ TRÍ | Cả hai engine |
|---|---|
| màn hình hiện | `tha đả khai liễu na phiến môn，tẩu tiến liễu hắc ám chi trung。` |
| bôi đen `"tha đả khai"` ⇒ gửi đi | **`他打開`** ✅ |
| bôi đen `"môn"` ⇒ | `門` ✅ · qua dấu câu `"môn，tẩu"` ⇒ `門，走` ✅ |
| vùng chọn rỗng ⇒ | `null` *(không phát lượt tra)* ✅ |

#### ③ AC4 — NFR1, đường sản phẩm `commands::dict::lookup`, **166 truy vấn KHÁC NHAU**

`--release` · **4 lớp `.db` THẬT** *(`src-tauri/resources/dict`, 373 MB)* · cửa sổ trượt
1–3 ký tự trên văn xuôi Hán thật + 10 truy vấn Latin *(AD-44 ⑥ — đường tiếng Anh đo RIÊNG)*.
Truy vấn **khác nhau** vì Quyết định #5 bật dedupe. **78/166** đi qua đường lui `Substring`;
**2** lượt chạm `query_too_short` ⇒ `:615` thực thi được bằng dữ liệu thật.

| Tiến trình | lượt | p50 | p95 | p99 | max |
|---|---|---|---|---|---|
| #1 | 1 | 3,786 | 9,087 | **34,930** | 43,954 |
| #1 | 2 | 1,047 | 1,772 | 6,377 | 19,592 |
| #2 | 1 | 1,084 | 2,378 | 9,832 | 25,213 |
| #2 | 2 | 1,003 | 1,832 | 5,535 | 20,366 |

🔴 **Bẫy 8 tái lập ĐÚNG như 1.17 cảnh báo**: `p99 34,930 ms` của lượt đầu là **nhiễu
page-cache**, không một thuộc tính của mã — nó biến mất ở ba lượt sau. Kết luận đứng trên **p99
trạng thái ổn định: 5,5–9,8 ms**, không trên p95 của một lượt.

⚠️ **Giới hạn, khai TRƯỚC khi đo:** đây là **đường Rust**. Nó không gồm vòng IPC Tauri lẫn lượt
VẼ. Cơ chế đo đầu-cuối thật đã cài *(`lookupTiming.ts`, mốc cuối sau `requestAnimationFrame`,
cờ TẮT mặc định, bật bằng `__auraLookupTiming.enable()`)* nhưng **chưa chạy trong một bản
Tauri đóng gói** — xem §Completion Notes.

#### ④ Đối chứng ĐỎ-rồi-XANH

| Cổng / ca | Cách làm đỏ | Kết quả |
|---|---|---|
| `the_candidate_ceiling_keeps_the_truncated_flag_honest` | `SAFETY_FACTOR` 50 → 100.000 *(≈ không trần)* | **ĐỎ** → khôi phục → **XANH** |
| **Kiểm F** ①, `check:commands` | gỡ lời gọi đăng ký ở `EditorPanel.vue` | **ĐỎ** → khôi phục → **XANH** |
| **Kiểm F** ②, `check:commands` | `LookupPanel` đổi vai `'display'` → `'source'` | **ĐỎ** *(Bẫy 1)* → khôi phục → **XANH** |

#### ⑤ AC10 — điểm ra mạng trên đường tra cứu

Đếm trên **mười** tệp của đường tra *(`commands/dict.rs` · `core/dict/{mod,query,layer,senses}.rs`
· `ports/dict_source.rs` · `config/dict.ts` · `lookupPanelState.ts` · `selectionContract.ts`
· `lookupTiming.ts`)* với mẫu `fetch( · XMLHttpRequest · WebSocket · EventSource ·
sendBeacon · reqwest · ureq · http(s)://` ⇒ **0 / 10 tệp**. `reqwest` khai trong `Cargo.toml`
nhưng có **0** lời gọi thật trong `src-tauri/src/**` *(chỉ hai dòng doc-comment ở
`core/webimport` và `core/ai` — Epic 4/6)*.

#### ⑥ Ranh giới không CHẠM — đếm lại sau story

| | Trước | Sau |
|---|---|---|
| `window.matchMedia(` — **lời gọi** | 0 | **0** *(6 lần nhắc tên, đều trong chú thích `không …`)* |
| `window.innerWidth` — lời gọi | 0 | **0** |
| `scroll-behavior` / `scrollIntoView` — lời gọi | 0 | **0** *(3 lần nhắc trong chú thích)* |
| `translate(` / `scale(` trong `LookupPanel.vue` | 0 | **0** *(2 hit là `text-transform`, typography không hình học)* |
| Phụ thuộc mới *(npm + crate)* | — | **0** |

#### ⑦ Sàn đã nâng theo SỐ THẬT

| Hằng | Cũ | **Mới** | Số thật |
|---|---|---|---|
| `check-layout.mjs::FILE_FLOOR` | 30 | **32** | 39 |
| `check-tokens.mjs::FILE_FLOOR` | 32 | **34** | 42 |
| `check-tokens.mjs::COMPONENT_FILE_FLOOR` | 30 | **32** | 39 |
| `check-commands.mjs::TS_FLOOR` | 20 | **21** | 26 |
| `check-commands.mjs::COMMAND_FLOOR` | 14 | **18** | 22 |
| `check-commands.mjs::DISPATCH_FLOOR` | 10 | **11** | 13 |
| `SELECTION_SURFACE_FLOOR` *(MỚI)* | — | **4** | 5 |

không nâng *(story không vượt)*: `VUE_FLOOR` 11/13 · `CLICK_FLOOR` 7/8 · `RS_FLOOR` 34/40.

#### ⑧ Bộ DoD chín lệnh — lượt cuối

`cargo test` **232 xanh** *(baseline 225, +7 ca mới)* · `npm run build` ✅ · `check:tokens`
✅ · `check:i18n` ✅ · `check:commands` ✅ *(gồm **Kiểm F** mới)* · `check:layout` ✅ ·
`check:deps` ✅ · `check:dict-manifest` ✅ · `check:scope` ✅ — **9/9 exit 0**.

### Completion Notes List

**Đã làm, và nghiệm thu được:**

- **Hợp đồng vùng chọn dùng chung** *(`selectionContract.ts` MỚI)* — OPT-IN theo phần tử, một
  listener `document`, vị từ đọc `anchorNode.nodeType`. **Bốn** panel đăng ký; Panel Lookup
  mang vai `'display'` nên nó **không bao giờ** là nguồn *(Bẫy 1 — vòng tự thay thế)*.
- **Kiểm F** ở `check-commands.mjs` cưỡng chế AC2 **bằng máy**, cả hai chiều, đỏ được.
- **Bôi đen bằng bàn phím** — 5 command mới, đóng `deferred-work.md:608`.
- **Hiệu ứng 90 ms** `opacity` 0.4→1 `ease-out`, **huỷ không khởi động lại** *(AC6 — Bẫy 4)*;
  **cuộn về đầu tức thì**; **vạch tiến trình 250 ms** neo tuyệt đối, `ornament`;
  `prefers-reduced-motion` bằng **CSS thuần** *(`matchMedia` vẫn **0**)*.
- **Đường lui `Substring`** *(Ice chốt)* + **trần an toàn SQL** ở ba nhánh `verify_*`, kèm
  phần mà `:631` không nêu: cờ `truncated` không được nói dối khi trần chạm.
- **Năm** mục `deferred-work` gọi đích danh 1.18 đều **ĐÓNG** *(`:608` `:615` `:631` `:633`
  `:635`)* — không mục nào chuyển sang 7.7, vì Ice chốt ngược mặc định story ở Quyết định #3.

🔴 **THAY ĐỔI NHÌN THẤY ĐƯỢC, không giấu:**

1. **`Tab` nay DỪNG ở thân Panel Source** *(và ở bề mặt Hán Việt)*. `PanelFrame` mang
   `tabindex="-1"` và không vào vòng `Tab`; hai `tabindex="0"` bên trong nó là chỗ **DUY NHẤT**
   story này chạm hợp đồng tiêu điểm mà Story 1.14 dặn không chạm *(UX-DR8/UX-DR17)*. Ice chốt
   chấp nhận 2026-08-07 — đó là **giá** của việc đóng `:608`.
2. **`.lookup-head` nay nằm NGOÀI vùng cuộn.** AC7 nói *"vùng đầu mục không cuộn — nó `flex:
   none`"*; `flex: none` chặn **co giãn**, nó **không chặn cuộn**. Bản 1.17 để `.lookup-head`
   bên trong `.lookup-body` *(`overflow: auto`)* nên đầu mục **trôi mất** khi đọc một bản ghi
   dài — ý định của AC7 rõ, cơ học thì vỡ. Tách `.lookup-scroll` ra một lớp riêng.
   `--lookup-head-height: 76px` và `overflow: hidden` **giữ nguyên**; thêm đúng
   `position: relative`.
3. **Truy vấn từ tab Hán Việt kiểu song song nay đọc node `.hv-char`**, không `Selection.
   toString()` — xem §Debug Log ②.

⚠️ **NHỮNG GÌ không ĐƯỢC ĐÁNH DẤU ĐẠT** *(khuôn 1.17)*:

- **AC4 không đạt TRỌN.** p95/p99 đo được xa dưới trần, nhưng con số là **đường Rust** — nó không
  gồm vòng IPC Tauri thật lẫn lượt VẼ. Cơ chế đo đầu-cuối đã cài và bật được tay, **chưa
  chạy trong bản đóng gói**. Món nợ 1.17 **kế thừa, không đóng**.
- **Vế thị giác không nghiệm thu trên hai nền tảng thật.** Hiệu ứng 90 ms, vạch 250 ms,
  `reduced-motion`, điểm dừng `Tab` mới — mới qua `vue-tsc` + cổng tĩnh + hai bộ đo engine
  rời, không qua mắt người trên `tauri dev` lẫn một máy Windows.
- **AC11 vế Windows không đo trên WebView2 thật** — đo trên Chromium *(cùng engine, khác vỏ)*.
- **AC12 không chạy lại bằng một cú KÉO CHUỘT thật** — `Selection.modify()` là thuật toán chọn
  của chính trình duyệt, nhưng nó không **là** một lượt kéo. Vế đó cần Playwright *(1.16 đã
  dùng)*, và story này không thêm phụ thuộc nào *(NFR15)*.
- **Giữ phím không mở rộng vùng chọn liên tục** *(`keys.ts:295`, `event.repeat`)* — một món nợ
  **MỚI**, có tên, chủ là Story 1.21.
- **`user-select: none` là một cái bẫy CÒN MỞ cho bề mặt tương lai** — story này vá chỗ
  dùng, không vá được nguyên nhân, và không cổng nào canh. Story 3.4 / Epic 2 thừa hưởng.

Sáu mục trên đã ghi vào `deferred-work.md` §1.18 kèm chủ sở hữu.

### File List

**MỚI**

- `src/panels/selectionContract.ts`
- `src/panels/lookupTiming.ts`

**SỬA**

- `src/main.ts`
- `src/commands/index.ts`
- `src/commands/README.md`
- `src/i18n/vi.json`
- `src/panels/SourcePanel.vue`
- `src/panels/SourceHanViet.vue`
- `src/panels/LookupPanel.vue`
- `src/panels/AiTranslationPanel.vue`
- `src/panels/EditorPanel.vue`
- `src/panels/README.md`
- `scripts/check-commands.mjs`
- `scripts/check-layout.mjs`
- `scripts/check-tokens.mjs`
- `src-tauri/src/commands/dict.rs`
- `src-tauri/src/core/dict/query.rs`
- `src-tauri/tests/dict_lookup.rs`
- `src-tauri/tests/dict_sources.rs`
- `_bmad-output/implementation-artifacts/deferred-work.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/1-18-auto-lookup.md`

⚠️ `src/modes/libraryImport.ts` **không thuộc story này** — nó là commit riêng `09d9c87`
*(bản vá chưa commit lúc tạo story; Ice chốt tách ra ở Task 0)*.

