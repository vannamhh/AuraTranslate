---
baseline_commit: a664dac
---

# Story 2.10: Điều hướng segment

Status: review

**Covers:** FR25 · AD-34 §1 · NFR17
**Epic:** 2 — Biên tập theo segment
**Story trước:** 2.9 — Gộp bằng `Backspace` ở đầu ô (`done`, 2026-08-17)

---

## Story

As a **người dịch**,
I want **nhảy tới câu chưa dịch tiếp theo bằng một phím**,
so that **tôi không phải cuộn tìm bằng mắt trong một Chương dài**.

---

## 🔴 ĐỌC TRƯỚC DÒNG MÃ ĐẦU TIÊN — một nửa story này ĐÃ ĐƯỢC DỰNG, ở Story 2.5b

Đây là mệnh đề quan trọng nhất của cả tệp, và nó **đo được**, không phải một ấn tượng.

`src/panels/segmentNavigation.ts` **đã tồn tại** từ Story 2.5b. Nó là một **module thuần**
(không Vue, không DOM, chỉ `import type`) và nó đã cài **đúng từng chữ** ba trong chín mệnh
đề AC của story này:

| Mệnh đề AC | Đã có ở đâu | Trạng thái |
|---|---|---|
| *"chưa dịch" = `status='draft'` **và** `target_text` rỗng* | `segmentNavigation.ts:63-65` (`isUntranslated`) | ✅ **đúng cả hai vế**, kèm khối lý do |
| *bỏ qua câu đã cắt bỏ (FR133)* | `segmentNavigation.ts:102` (`if (s.isOmitted) continue`) | ✅ đã cài, có lý do tại chỗ |
| *nhảy tới câu chưa dịch gần nhất phía sau* | `segmentNavigation.ts:91-106` (`nextUntranslatedId`) + `editorPanelState.ts:1029-1035` (`goToNextUntranslated`) + `commands/index.ts:1070-1084` (`editor.next_untranslated`, phím `Alt+ArrowDown`) | ✅ chạy được hôm nay |
| *vạch lề của segment nhận focus chuyển `primary`* | `editorSegments.ts::resolveSegmentRule` + `ruleById` (`GridPanel.vue:217-226`) | ✅ **tự động** — hễ gọi `setEditorCaret(id)` thì vạch đổi theo phản ứng Vue, **không một dòng mã mới** |

🔴 **Viết lại bất cứ thứ gì trong bảng trên là một lỗi thiết kế, không phải một lựa chọn.**
`nextUntranslatedId` mang **bốn** quyết định đã có chữ ký và ghi lý do tại chỗ (hai vế của
"chưa dịch" · không quay vòng · bỏ qua `retiredAt` · bỏ `isOmitted` **ở hàm gọi** chứ không
nhét vào `isUntranslated`). Một bản sao lệch đi biên dịch sạch và đi qua cả mười một cổng.

⇒ **Phạm vi THẬT của Story 2.10 là năm việc, và không việc nào trong số đó là "logic chọn
câu chưa dịch":**

| # | Việc | Trạng thái nền |
|---|---|---|
| ① | **Hai lệnh tuần tự** `editor.next_segment` / `editor.prev_segment` | Chưa có. `grep -n "nextSegmentId\|prevSegmentId\|previousSegment" src/` = **0 kết quả** |
| ② | **Báo rõ ra MÀN HÌNH** khi hết câu chưa dịch / khi ở biên | Hôm nay chỉ `console.info` (`commands/index.ts:1078-1081`). Theo định nghĩa của dự án, console **là** im lặng ⇒ AC6 **đang hỏng** |
| ③ | **Cuộn tới hàng, tức thì, không hiệu ứng** | Chưa có. `grep -n "scrollIntoView\|scrollTop" src/panels/GridPanel.vue` = **0 kết quả**. Tiền lệ duy nhất của kho: `LookupPanel.vue:269-283` |
| ④ | **Phân xử cả bảng phím điều hướng** | Món nợ **giao đích danh cho story này** trong mã: `commands/index.ts:1060-1065` |
| ⑤ | **"Đi đâu khi hết Chương"** | Món nợ **giao đích danh** trong sổ: `deferred-work.md:2837-2847` |

---

## Acceptance Criteria

Chín mệnh đề, nguyên văn ngữ nghĩa từ `epics.md:2578-2620`. Đánh số để tick được.

**AC1 — Segment kế tiếp**
**Given** con trỏ ở một segment
**When** gọi lệnh **segment kế tiếp**
**Then** focus chuyển sang segment ngay sau nó

**AC2 — Segment trước đó**
**Given** con trỏ ở một segment
**When** gọi lệnh **segment trước đó**
**Then** focus chuyển sang segment ngay trước nó

**AC3 — Segment chưa dịch kế tiếp bỏ qua câu đã dịch**
**Given** một Chương có segment đã dịch xen kẽ segment chưa dịch
**When** gọi lệnh **segment chưa dịch kế tiếp**
**Then** focus nhảy tới segment chưa dịch gần nhất phía sau, bỏ qua các segment đã dịch

**AC4 — Định nghĩa *"chưa dịch"*, hai vế**
**Given** định nghĩa *"chưa dịch"*
**When** lệnh này duyệt
**Then** *"chưa dịch"* là `status = 'draft'` **và** `target_text` **rỗng**
> 🔵 *Thêm 2026-08-14 (Sprint Change Proposal, Ice ký):* `draft` nay **tách khỏi** *chưa dịch*
> trong bảng sáu giá trị của UX-DR19. Không khai vế `target_text` rỗng thì lệnh này nhảy vào
> **mọi** câu chưa xác nhận — tức nó thành vô dụng đúng lúc Chương gần hoàn thành.

**AC5 — Bỏ qua câu đã cắt bỏ**
**Given** một câu đã **cắt bỏ** (FR133)
**When** gọi lệnh **segment chưa dịch kế tiếp**
**Then** **bỏ qua** nó

**AC6 — Hết câu chưa dịch thì BÁO, không im lặng**
**Given** không còn segment chưa dịch nào phía sau
**When** gọi lệnh segment chưa dịch kế tiếp
**Then** báo rõ điều đó thay vì im lặng không làm gì

**AC7 — Biên Chương**
**Given** con trỏ ở segment đầu hoặc cuối Chương
**When** gọi lệnh vượt biên
**Then** hành vi ở biên rõ ràng và không sập

**AC8 — Vạch lề và vùng nhìn**
**Given** focus chuyển segment
**When** xảy ra
**Then** vạch lề của segment nhận focus chuyển `primary`
**And** vùng nhìn cuộn tới nó **tức thì**, không có hiệu ứng cuộn

**AC9 — Ba lệnh là command đăng ký**
**Given** ba lệnh này
**When** gọi
**Then** đều là command đăng ký, gán phím được

---

## 🔴 Task 0 — CỬA CHẶN: bảy quyết định mở, phải có chữ ký của Ice

**Task 0 chặn mọi task khác.** Không viết dòng mã đầu tiên trước khi bảy mục dưới đây có
phán định. Lý do không phải thủ tục: sáu trên bảy mục là chỗ **hai phương án đều hợp lệ**, và
project-context §Story và spec viết bằng chữ — *"Ice là người chốt các quyết định mở. Nêu cả
hai kèm số đo, đừng tự chọn rồi đi tiếp."*

### Quyết định #1 🔴 — `⌥↓` có được chạy TRONG vùng gõ không

Đây là quyết định **nặng nhất**, vì nó định đoạt tính năng có dùng được hay không.

**Sự thật đã đo** (`commands/keys.ts:415` + `:510`):
```ts
const lacksPrimaryMod = (m: Mods): boolean => !m.meta && !m.ctrl
// …
if (lacksPrimaryMod(entry.mods) && isTypingZone(event.target)) return false
```
`Alt` **không** là phím bổ trợ chính ⇒ `Alt+ArrowDown` **bị nuốt** khi caret đang ở trong một
ô `contenteditable`. Story 2.5b đã ghi ra bằng chữ và **giao lại đích danh cho story này**
(`commands/index.ts:1060-1065`):

> *"phím này vì thế chạy khi con trỏ **KHÔNG** ở trong một ô đang gõ […] Món này có chủ:
> **Story 2.10** *(điều hướng segment)* sẽ phải phân xử cả bảng phím điều hướng cùng một
> lượt, và lúc đó nó có đủ ngữ cảnh mà hôm nay chưa có."*

🔴 **Vì sao đây là cửa chặn chứ không một chi tiết:** ca **thường nhất** của FR25 là *người
dùng vừa gõ xong một câu và muốn nhảy tới câu chưa dịch kế tiếp* — tức caret **đang ở trong
vùng gõ**. Với luật hôm nay, phím không bắn. AC3 vẫn "đạt" theo chữ (lệnh chạy được từ ngoài
vùng gõ), nhưng tính năng **chết trong tay người dùng**.

| Đường | Việc phải làm | Cái được | Cái mất |
|---|---|---|---|
| **(a)** Giữ nguyên | Không sửa gì | Không cướp phím hệ điều hành | Tính năng không dùng được ở ca thường nhất. Ghi nợ có chủ |
| **(b)** Bắt ở `onEditKeydown` rồi `dispatch` | Khuôn 2.9 đã dựng hai lần (`Backspace` ở đầu ô, `Escape`) — *"hai cửa, MỘT command"* | Phím chạy ở mọi chỗ | 🔴 `⌥↓` trong một ô văn bản trên macOS **là** *"xuống cuối đoạn"* của hệ điều hành. Cướp nó là lấy mất một phản xạ người dùng đã có |
| **(c)** Đổi sang hợp âm có `Mod` | Đổi `keys` của command đã đăng ký + sửa hàng `⌥↓` trong `EXPERIENCE.md:261-269` | Đi qua `keys.ts:510` sạch, không cần cửa thứ hai, không cướp phím OS | Đổi một phím **đã phát hành** ở 2.5b. Và phím tắt nằm trong bảng keybinding người dùng (Story 1.21) ⇒ một lượt đổi id/phím có thể mồ côi thứ người dùng đã gán |

⚠️ **Phải đo trước khi ký (Task 1.3), đừng suy luận:** đường (b) có thật sự chặn được lượt
"xuống cuối đoạn" của WebKit không. `preventDefault()` trên một sự kiện **thật** là điều
`browser.keys()` **không nghiệm thu được** — mọi sự kiện driver giao đều `isTrusted: false`
và một sự kiện không tin cậy **không có default action** (đã đo ở Story 2.9,
`deferred-work.md` §bàn đo 2.9). Vế này chỉ đóng được bằng **chữ ký tay của Ice trên máy thật**.

### Quyết định #2 🔴 — phím mặc định cho `next_segment` / `prev_segment`

**Đã grep, không phải phỏng đoán:** bảng *"### Phím"* của `EXPERIENCE.md:261-269` có bảy hàng
(`⌘Enter` · `Enter` · `Shift+Enter` · `Backspace` đầu ô · `Mod`+click · `⌘/` · `⌥↓`) và
**không hàng nào** cho *"segment kế tiếp"* / *"segment trước đó"* thuần. AC9 chỉ đòi *"command
đăng ký, gán phím được"* — nó **không** đòi một phím mặc định cụ thể. Nên có ba đường:

- **(a)** `⌥↑` / `⌥↓`-với-Shift…: đối xứng với `⌥↓` sẵn có, nhưng thừa hưởng **nguyên** vấn đề
  của Quyết định #1.
- **(b)** Một hợp âm mang `Mod` (ví dụ `Mod+Alt+↓` / `Mod+Alt+↑`): đi qua `keys.ts:510` sạch.
- **(c)** Không gán phím mặc định nào — chỉ đăng ký command, để người dùng tự gán qua Story
  1.21. Đủ chữ của AC9, và **không** đụng `EXPERIENCE.md`.

🔴 **Kiểm trùng hợp âm là TOÀN registry, không theo chế độ** (bài học đã ghi ở 2.9 về `⌘M` và
Epic 7). Đường nào cũng phải chạy `npm run check:commands` để cổng tự bác một hợp âm trùng —
đừng đọc bảng bằng mắt.

⚠️ **`EXPERIENCE.md` thuộc quyền Ice.** Nếu đường ký cần một hàng mới trong bảng Phím thì đó
là một lượt sửa tài liệu UX — **đề xuất**, không tự sửa (trừ lượt sửa-tại-chỗ một mệnh đề đã
hết đúng, vốn có tiền lệ và luôn kèm 🔵 + ngày).

### Quyết định #3 — next/prev có bỏ qua câu đã cắt bỏ không

AC5 chỉ nói về **lệnh chưa-dịch-kế-tiếp**. Nó **im lặng** về hai lệnh tuần tự — và đó là một
khoảng trống thật, không một chỗ đọc lướt.

🔴 **Có một lập luận mạnh cho "KHÔNG bỏ qua", và nó là một lỗ hổng năng lực nếu chọn sai:**
FR133 (`prd.md:458`) viết bằng chữ rằng hàng đã cắt bỏ **vẫn nằm trong lưới**, gạch ngang và
mờ đi, và **đảo ngược được bất cứ lúc nào**. Lệnh `editor.restore_segment` (Story 2.5c) chạy
**trên câu đang có caret**. Nếu cả ba lệnh điều hướng đều bỏ qua hàng đã cắt bỏ thì
**không đường bàn phím nào đưa caret tới đó được nữa** ⇒ FR133 vế *"đảo ngược được"* chỉ còn
đường chuột, và NFR17 (`prd.md:903`, *"mọi thao tác hoàn toàn bằng bàn phím"*) hỏng im lặng.

⇒ Đề xuất: **(a) next/prev KHÔNG bỏ qua** (chúng là điều hướng **vị trí**), **chưa-dịch-kế-tiếp
CÓ bỏ qua** (nó là điều hướng **theo việc**). Cần chữ ký vì nó là một mệnh đề AC không nêu.

### Quyết định #4 — ô nhớ nào chở thông báo của AC6/AC7

Thanh trạng thái hôm nay có **hai** ô nhớ, cả hai là `Record` **ĐÓNG** trên kiểu (nên
`vue-tsc` đỏ nếu thiếu khoá): `CONFIRM_NOTICE_KEYS` (`StatusBar.vue:107`) và
`REGROUP_NOTICE_KEYS` (`StatusBar.vue:141`). Không có ô thứ ba.

- **(a)** Nới `RegroupNotice`: rẻ nhất, nhưng **sai ngữ nghĩa** — cái tên đó nghĩa là
  *"gộp/tách"*, và một thông báo điều hướng nhét vào đó là một nguồn sự thật lệch tên.
- **(b)** Dựng ô nhớ thứ ba (`navNotice`) theo **đúng** khuôn `ghiRegroupNotice`
  (`editorPanelState.ts:1213`) — bất biến *"thao tác vừa xảy ra sở hữu thanh trạng thái: ai
  ghi một ô thì dọn ô còn lại"*.

🔴 **Đường (b) mang một chi phí phải nêu trước khi ký, không sau:** bất biến trên hôm nay là
**hai chiều**; ô thứ ba làm nó thành **N chiều** (mỗi lượt ghi phải dọn hai ô kia). Đó là
đúng hình dạng đã bị bỏ sót **hai story liên tiếp** — xem Cạm bẫy ⑤. Nếu ký (b) thì phải ký
kèm việc dựng bất biến ấy thành **một cửa duy nhất**, không ba lời gọi rải rác.

### Quyết định #5 — hành vi ở biên cho next/prev (AC7)

AC7 chỉ đòi *"rõ ràng và không sập"* — ba đường đều thoả chữ:
- **(a)** Con trỏ ở nguyên **+ báo trên thanh trạng thái** (đối xứng với AC6).
- **(b)** Con trỏ ở nguyên, **không báo gì** (rẻ, nhưng người dùng không phân biệt được
  *"đã ở cuối"* với *"phím hỏng"*).
- **(c)** Quay vòng về đầu — 🔴 **CẤM theo tiền lệ đã ghi lý do**: `segmentNavigation.ts:74-77`
  viết *"KHÔNG quay vòng về đầu […] Một lượt quay vòng im lặng đưa người dùng về đầu Chương mà
  không dấu hiệu nào — họ đọc thành 'phím này nhảy lung tung'."* Chọn (c) là lật một quyết
  định đã có chữ ký, tức phải lật **tường minh**, không tiện tay.

### Quyết định #6 🔴 — "đi đâu khi hết Chương" (món nợ giao đích danh story này)

`deferred-work.md:2837-2847` — nguyên văn phần chủ:

> *"**Chủ: Story 2.10** (điều hướng segment — nó **dùng lại** đường dời con trỏ tối thiểu của
> 2.5 chứ không dựng đường thứ hai, nên nó là chỗ duy nhất trả lời được câu 'đi đâu khi hết
> Chương')."*

**Món nợ là gì:** AC1 của Story 2.5 đòi *"trạng thái chuyển sang đã xác nhận **và vạch lề
chuyển `confirmed`**"*. Lời giải đã ký là **dời con trỏ sang câu kế tiếp** — vì
`resolveSegmentRule` cho `primary` **thắng** `confirmed` và thứ tự đó là một quyết định 🔴
không được đảo. Ở **câu cuối Chương không có câu kế** ⇒ vạch **ở lại `primary`** cho tới khi
người dùng tự đi chỗ khác. `segment.status` trong CSDL thì **đúng**; chỉ vế thị giác hụt.
⚠️ Nó xảy ra **đúng một lần mỗi Chương**, ở đúng câu cuối — không phải ca hiếm.

Ba đường, và **không đường nào miễn phí**:
- **(a)** Đảo thứ tự `primary` / `confirmed` — 🔴 **CẤM**, là quyết định có chữ ký, không đảo.
- **(b)** `⌘Enter` ở câu cuối **nhả caret** (`setEditorCaret(null)`) ⇒ không còn `primary` ⇒
  vạch hiện `confirmed`. ⚠️ Phải đo: `setEditorCaret(null)` **không** flush
  (`editorPanelState.ts:148-150` chỉ flush khi đi **từ** một câu **sang** một câu khác) — nhả
  caret có bỏ rơi bộ đệm gõ không? Và caret rơi về đâu trong DOM (AD-34 §2 cấm rơi về `body`)?
- **(c)** Chấp nhận vế thị giác và **báo trên thanh trạng thái** *"đã xác nhận câu cuối Chương"*
  — đóng bằng **thông tin** thay vì bằng màu, dùng lại đúng hạ tầng của Quyết định #4.

### Quyết định #7 — cơ chế cuộn (AC8)

Kho có **đúng một** tiền lệ "cuộn tức thì", và nó **cấm** `scrollIntoView` bằng chữ —
`LookupPanel.vue:269-270`:

> *"🔴 AC7 — vị trí cuộn về đầu **TỨC THÌ**. không `scrollIntoView`, không `behavior: 'smooth'`,
> không `scroll-behavior` trong CSS (`DESIGN.md:342` cấm cả ứng dụng)."*

⚠️ **Tiền lệ đó trả lời một câu hỏi KHÁC:** *"về đầu"* là `scrollTop = 0` — một hằng số. Story
này cần *"tới **hàng thứ N**"*, mà hàng trong lưới **không phải một phần tử DOM**
(`GridPanel.vue` — `subgrid` năm cột, style cấp-hàng phải nhân ra năm ô).

🔴 **VÀ VẾ NGOẶC CỦA CHÚ THÍCH ĐÓ ĐÃ HẾT ĐÚNG — đo 2026-08-17, đừng thừa kế nó.**
`grep -rn "scroll-behavior" ux-designs/` ⇒ **0 kết quả**. `DESIGN.md:342` hôm nay nói về
chiều rộng `ch` của Chế độ đọc, **không** về cuộn. Hai mệnh đề gần nhất còn thật:
- `DESIGN.md:373` — *"Vị trí cuộn | Về đầu **tức thì**. Không bao giờ cuộn có hiệu ứng"*, nhưng
  nó nằm trong bảng Motion **của Panel Lookup**, không phải một luật toàn ứng dụng;
- `DESIGN.md:379` — *"Ba cấm chung cho toàn ứng dụng: không `translate` trong thao tác lặp,
  không hiệu ứng nào vượt 150ms trên đường nóng, không hiệu ứng xếp hàng"* — thật sự toàn ứng
  dụng, nhưng nó **không nêu tên** `scroll-behavior`.

⇒ **Không có luật nào cấm `scroll-behavior` toàn kho.** Mệnh đề đó chỉ sống trong hai khối chú
thích của `LookupPanel.vue` (`:270` và `:830`) — đúng lớp *"chú thích cũ hơn mã"* mà
`project-context.md` §Bẫy tài liệu mô tả. Hệ quả trực tiếp cho quyết định này: **lớp bảo vệ
thứ hai mà đường (a) tưởng có thì KHÔNG có.**

Hai đường:

- **(a)** `scrollIntoView({ block: 'nearest' })` trên **một ô** của hàng đích. `block:'nearest'`
  **không cuộn gì cả** khi hàng đã nằm trong vùng nhìn — đúng thứ một phím bấm liên tục cần.
  🔴 **`behavior: 'instant'` là BẮT BUỘC, không một lượt cẩn thận thừa:** mặc định `auto` uỷ
  quyền cho CSS `scroll-behavior`, và ta vừa đo được rằng **không luật nào canh giá trị đó**.
- **(b)** Tính `scrollTop` bằng tay từ `getBoundingClientRect()` của ô đích và của `.grid-scroll`
  (`GridPanel.vue:1304` · `:1524-1527` — **hộp cuộn duy nhất**, bọc cả năm cột vì chúng phải
  cuộn cùng nhau). Không phụ thuộc hành vi engine, nhưng phải tự cài "đã trong vùng nhìn thì
  đừng cuộn" và tự xử lề trên/dưới.
- **(b)** Tính `scrollTop` bằng tay từ `getBoundingClientRect()` của ô đích và của `.grid-scroll`
  (`GridPanel.vue:1304` · `:1524-1527` — **hộp cuộn duy nhất**, bọc cả năm cột vì chúng phải
  cuộn cùng nhau). Không phụ thuộc hành vi engine, nhưng phải tự cài "đã trong vùng nhìn thì
  đừng cuộn" và tự xử lề trên/dưới. ⚠️ Đường này **cũng** đọc CSS `scroll-behavior` khi gán
  `scrollTop` — nó không miễn nhiễm; nó chỉ đổi chỗ rủi ro.

⚠️ **Phải đo trên WKWebView thật trước khi ký (Task 1.2)** — `subgrid` là chỗ ba engine đã
bất đồng ít nhất một lần trong epic này, và câu hỏi *"`block:'nearest'` trên một ô của subgrid
có cuộn đúng track hàng không, hay nó cuộn theo hộp con"* **chưa ai đo**. `happy-dom` không
trả lời được: nó không phải WebKit và nó không bố cục.

### 0.8 — Việc phải làm ở task này

1. Trình bảy quyết định cho Ice, **mỗi cái kèm số đo hoặc trích dẫn nguồn**, không kèm một
   khuyến nghị đã tự chốt.
2. Quyết định #1 và #7 **chờ số của Task 1** — nêu trước, ký sau. Đừng ký trước rồi đo sau
   *(khuôn này đã cắn Story 2.5b ở Quyết định #6)*.
3. Ghi chữ ký vào §Dev Agent Record **kèm ngày**, và ghi cả **đường bị loại cùng lý do**.
4. 🔴 **LUẬT DỪNG:** nếu ba vòng chẩn đoán liên tiếp trên một giả thuyết về **sản phẩm** bị
   phép đo bác ⇒ **DỪNG, báo Ice**, đừng đi vòng thứ tư. *(Đếm vòng bị bác, không đếm lượt
   sửa thước — phân biệt này đã cứu Story 2.5d một lượt báo động giả.)*

---

## Tasks / Subtasks

### Task 0 — Cửa chặn: bảy quyết định (AC: 1-9) — **CHẶN MỌI TASK KHÁC**
- [x] 0.1 Đo lại từ nguồn, đừng chép tệp này: `grep -c "^### AD-" ARCHITECTURE-SPINE.md` ·
      `COMMAND_FLOOR` hiện tại · số command đã đăng ký mà cổng in ra · số bước `PROJECT_MIGRATIONS`
- [x] 0.2 Trình Quyết định #2 · #3 · #4 · #5 · #6 (không phụ thuộc số đo)
- [x] 0.3 Trình Quyết định #1 · #7 **sau** Task 1
- [x] 0.4 Ghi chữ ký + đường bị loại + ngày vào §Dev Agent Record

### Task 1 — Bàn đo WKWebView thật (AC: 8) — **CHẶN Task 4 và Quyết định #1/#7**
Thư mục `_bmad-output/implementation-artifacts/2-10-ban-do/`, đúng khuôn bảy tiền lệ của epic
(`2-2` · `2-4` · `2-5` · `2-5b` · `2-5d` · `2-8` · `2-9`). ⚠️ Tạo tác của một lượt đo — **không**
thứ gì ở đây vào `package.json`.
- [x] 1.1 Tự kiểm danh tính phiên **trước mọi số** (`href` · `#app` · số hàng mong đợi) — khuyết
      tật bàn đo của 2.9 là `waitForExist` không phân biệt "Chương mới đã nạp" với "Chương cũ còn đó"
- [x] 1.2 **Quyết định #7:** `scrollIntoView({block:'nearest', behavior:'instant'})` trên một ô
      `[data-col="tgt"]` của subgrid — cuộn đúng track hàng? có animation? đã-trong-vùng-nhìn thì
      đứng yên? Đối chứng bằng đường (b) `scrollTop` tính tay trên **cùng** hàng
- [x] 1.3 **Quyết định #1:** `⌥↓` trong ô đang gõ — WebKit làm gì mặc định, `preventDefault()`
      có chặn được không. ⚠️ Ghi rõ `isTrusted` của mọi sự kiện; nếu `false` thì **đây là giới
      hạn của thước**, ghi vào §Chờ chữ ký Ice, đừng kết luận về sản phẩm
- [x] 1.4 Đo **một lượt dời con trỏ** trên Chương lớn nhất có thật — đối chiếu mốc 706–770 ms
      của `deferred-work.md` (Cạm bẫy ④). **Ghi số, KHÔNG tự chấm NFR2**
- [x] 1.5 Ghi môi trường truy nguyên được: phiên bản WebKit · macOS · rustc/cargo · Node · ngày
- [x] 1.6 README.md của thư mục bàn đo: cách chạy + §Kết luận + §Giới hạn thật

### Task 2 — Hai vị từ thuần trong `segmentNavigation.ts` (AC: 1, 2, 7)
- [x] 2.1 `nextSegmentId(segments, fromId)` / `prevSegmentId(segments, fromId)` — **cùng tệp,
      cùng luật** (`import type` duy nhất, không Vue/DOM). 🔴 Duyệt bằng **chỉ số mảng**, không
      `segment.ord` — xem Cạm bẫy ⑥
- [x] 2.2 Bỏ qua `retiredAt !== null` ở cả hai vai (đích, và không chặn đường) — đối xứng với
      `nextUntranslatedId:101`
- [x] 2.3 `isOmitted` theo Quyết định #3
- [x] 2.4 Biên theo Quyết định #5. 🔴 **Không quay vòng** trừ khi Ice lật tường minh
- [x] 2.5 `fromId === null` hoặc không tìm thấy ⇒ hành vi có nghĩa, không hành vi rỗng (khuôn
      `nextUntranslatedId:95-98`, và nó có lý do thật: `fromId` trỏ vào một câu vừa bị gộp mất)

### Task 3 — Hai lệnh mới (AC: 1, 2, 9)
- [x] 3.1 `goToNextSegment()` / `goToPrevSegment()` ở `editorPanelState.ts` — **chép khuôn ba
      dòng** của `goToNextUntranslated` (`:1029-1035`): `setEditorCaret(id)` **rồi**
      `caretPlacement.value = id`. 🔴 Không đường dời con trỏ thứ hai
- [x] 3.2 Đăng ký ở `src/commands/index.ts` (chỗ đăng ký **duy nhất**), tiêm dep qua
      `installCommands(deps)` ở `src/main.ts` — **không** trong `App.vue`
- [x] 3.3 Khoá `command.editor.*` mới trong `src/i18n/vi.json` (phẳng, khoá chấm có tiền tố miền)
- [x] 3.4 Phím theo Quyết định #2. Chạy `npm run check:commands` để cổng tự bác hợp âm trùng
- [x] 3.5 🔴 Nâng `COMMAND_FLOOR` (`scripts/check-commands.mjs:263`, hôm nay **39**) — **đếm lại
      từ số cổng in ra**, đừng cộng nhẩm. Bài học đã lặp ở 2.8 và 2.9
- [⊘] 3.6 Quyết định #1 đường (b) ⇒ thêm cửa thứ hai ở `onEditKeydown` (`GridPanel.vue`), và nó
      **`dispatch()` chính id đã đăng ký**, không gọi thẳng hàm — Cạm bẫy ②
      *(**KHÔNG CHẠY** — điều kiện `đường (b)` không xảy ra: Ice ký **#1(c)** *(đổi hợp âm sang
      `Mod+Alt+ArrowDown`)*, và chính đoạn ký liệt kê *"Loại (b) cửa thứ hai ở `onEditKeydown`"*
      cùng hai lý do. Đo:* `git diff HEAD -- src/panels/GridPanel.vue` *KHÔNG một dòng nào chạm
      `onEditKeydown`.* 🔵 *Ô này tick `[x]` cho tới lượt code review 2026-08-17 — sửa tại chỗ vì
      một tick không có bằng chứng là một tuyên bố sai.)*

### Task 4 — Cuộn tới hàng (AC: 8)
- [⊘] 4.1 Cài theo đường Ice ký ở Quyết định #7, đặt cạnh watcher `editorCaretPlacement`
      (`GridPanel.vue:895-916`) — nó là **đường LỆNH**, đúng chỗ, và đường chuột không đi qua đó
      *(**ĐÃ CÀI RỒI GỠ.** `cuonToiHang` được cài theo #7(b), rồi ba lượt đột biến của Task 7.3
      chứng minh nó là **mã chết** — `focus()` đã cuộn, và cuộn khéo hơn. Ice ký **gỡ** 2026-08-18.
      ⇒ Sản phẩm không còn một dòng mã cuộn nào; chỉ còn khối chú thích §"AC8 NỬA SAU" nói vì sao.
      Xem §Completion Notes và Change Log.)* 🔵 *Ô này tick `[x]` cho tới lượt code review
      2026-08-17; story đã tự thuật đầy đủ ở ba chỗ khác, nhưng ô tick đọc riêng vẫn sai.*
- [x] 4.2 🔴 Ghi `behavior: 'instant'` **tường minh** nếu đi đường (a) — **không** luật nào canh
      `scroll-behavior` trong kho này (đã đo, xem Quyết định #7)
- [x] 4.3 Không thêm `scroll-behavior` vào CSS ở bất kỳ đâu — và 🔵 **sửa tại chỗ** hai chú thích
      đã hết đúng ở `LookupPanel.vue:270` và `:830` (chúng trích `DESIGN.md:342` cho một luật
      không tồn tại). Kèm ngày và lý do, đúng khuôn ba tệp đã được sửa trước đây
- [x] 4.4 Nếu đường (a) cần một tên thành viên mới của `window`/`document`: `ALLOWED_GLOBAL_MEMBERS`
      (`scripts/check-layout.mjs:404`) đòi **một dòng lý do kèm theo**. *(`Element.scrollIntoView`
      không phải thành viên của `window`/`document` ⇒ dự kiến không chạm — **kiểm bằng cách chạy
      cổng**, không bằng suy luận này.)*
- [x] 4.5 🔴 **Món cho Ice:** nhìn bằng mắt rằng lượt cuộn không có hiệu ứng và hàng đích nằm
      trọn trong vùng nhìn. `happy-dom` không bố cục ⇒ không đường nghiệm thu nào của dự án mô
      phỏng được vế này
      *(**ĐÃ NGHIỆM THU 2026-08-17 — và nó TRẢ VỀ MỘT PHÁT HIỆN, không một dấu tick.** Lượt nhìn
      đầu tìm ra **hàng cuối Chương tiếng Trung bị cắt mất chữ** — 30 px nội dung không cử chỉ nào
      với tới được, nguyên nhân ở `.panel-body`/`.grid-scroll`, **không** ở lượt cuộn. Vá ở nguyên
      nhân theo đường (a) Ice ký, rồi Ice nhìn lần hai:* **"đã pass, không bị ẩn nữa"**.*)*
      🔴 **Đây là bằng chứng mạnh nhất của cả story cho luật *"bốn đường bốn vai"*:** mệnh đề này
      được giao cho **mắt người** vì `happy-dom` không bố cục — và đúng thứ chỉ mắt thấy được là
      thứ đã lọt qua **225 ca vitest, mười cổng, `build`, `vue-tsc`, bốn spec e2e và sáu spec bàn
      đo**. Một ô tick tự động ở đây sẽ là một lời hứa sai.

### Task 5 — Báo rõ ra màn hình (AC: 6, 7)
- [x] 5.1 Ô nhớ theo Quyết định #4, và 🔴 **thêm vào `resetEditorPanel()`**
      (`editorPanelState.ts:443`) — Cạm bẫy ⑤
- [x] 5.2 Bảng khoá **ĐÓNG** trên `StatusBar.vue` (khuôn `CONFIRM_NOTICE_KEYS:107` /
      `REGROUP_NOTICE_KEYS:141`) ⇒ `vue-tsc` đỏ nếu thiếu một khoá
- [x] 5.3 Khoá `panel.grid.*` mới trong `vi.json`
- [x] 5.4 **Gỡ `console.info`** ở `commands/index.ts:1078-1081` và thay bằng đường báo thật —
      đây là chỗ AC6 đang hỏng hôm nay
- [x] 5.5 Ca vitest cho bất biến "ai ghi một ô thì dọn ô còn lại" (khuôn
      `tests/frontend/editorRegroupNotice.test.ts`)

### Task 6 — Đóng món nợ "đi đâu khi hết Chương" (AC: 7)
- [x] 6.1 Cài theo Quyết định #6
- [x] 6.2 Nối tiếp vào `deferred-work.md:2837-2847` bằng `→ ✅ ĐÃ ĐÓNG <ngày> (Story 2.10)` kèm
      **cách** đóng. 🔴 **Không xoá** mục gốc. Đóng một nửa ⇒ 🟡 + liệt kê phần còn hở

### Task 7 — Nghiệm thu (AC: 1-9)
- [x] 7.1 vitest: hai vị từ mới + ca "ba chỗ đọc trạng thái phải đồng ý" mở rộng
      (`tests/frontend/segmentNavigation.test.ts`)
- [x] 7.2 e2e **spec riêng**, không nhét vào spec đã đầy (trần `mochaOpts.timeout` 120 s, ~5 s
      mỗi lệnh WebDriver — bài học 2.9)
- [x] 7.3 🔴 **Đột biến mã sản phẩm** cho mỗi ca mới: gỡ một chốt ⇒ đúng một ca đỏ. Một bộ test
      xanh chưa chứng minh gì; ba lượt gần nhất đều tìm ra lỗ hổng bằng đúng cách này
- [x] 7.4 11 cổng npm (9 đọc-tệp + `check:scope` + `check:scope:bundled` chạy tay, cần cổng 1420
      trống) · `build` · `vue-tsc` · `cargo test --locked` · e2e
- [x] 7.5 Đối chiếu baseline §Số nghiệm thu. 🔴 `cargo test` **phải không đổi** — xem Ranh giới ①

### Task 8 — Sổ nợ và tài liệu
- [x] 8.1 Mọi vế không nghiệm thu được ở tầng đang làm ⇒ `deferred-work.md` **kèm một chủ**.
      Không mục nào mồ côi
- [x] 8.2 Chú thích đã hết đúng thì **sửa tại chỗ** kèm 🔵 + ngày — đặc biệt
      `commands/index.ts:1053-1065`, khối giao món nợ cho chính story này
- [x] 8.3 `EXPERIENCE.md` chỉ sửa theo Quyết định #2 đã ký

---

## Dev Notes

### Đường dây đã có — vẽ đầy đủ, đừng dựng lại một mảnh nào

```
  phím  ──► keys.ts::handle ──► ⚠️ :510 lacksPrimaryMod && isTypingZone ⇒ NUỐT
                                    │ (đi qua)
                                    ▼
                              dispatch('editor.…')
                                    │
        (cửa thứ hai, chỉ khi Ice ký #1(b))
  GridPanel.vue::onEditKeydown ─────┤
                                    ▼
        commands/index.ts::register.run ──► deps.goToXxx()
                                                 │
                        editorPanelState.ts ─────┤
                                                 ├─► segmentNavigation.ts   (vị từ THUẦN)
                                                 ├─► setEditorCaret(id)  ──► flush AD-35 (d)
                                                 └─► caretPlacement.value = id
                                                            │
                              GridPanel.vue:895-916 (watcher, đường LỆNH) ──► el.focus() + setCaret
                                                            │
                                                            └─► ruleById ⇒ vạch `primary`  [AC8 nửa đầu]
                                                            └─► CUỘN — CHƯA CÓ, Task 4      [AC8 nửa sau]
```

🔴 **`setEditorCaret` KHÔNG chỉ đặt một biến** (`editorPanelState.ts:145-150`):

```ts
export function setEditorCaret(id: number | null): void {
  const left = caretSegmentId.value
  caretSegmentId.value = id
  // Rời câu A sang câu B (cả hai khác `null`, và khác nhau) ⇒ flush ngay cho A.
  if (left !== null && id !== null && left !== id) void flushEditorNow()
}
```

Nó **là** vế (d) *"rời segment"* của hợp đồng flush AD-35. Mỗi lượt điều hướng vì thế **ghi
xuống đĩa** văn bản của câu vừa rời — miễn phí, **nếu** đi qua hàm này. Một đường dời con trỏ
thứ hai làm mất chữ người dùng vừa gõ, **im lặng**, và không cổng nào đỏ.

⚠️ AD-47 **không** bị chạm: `ARCHITECTURE-SPINE.md` khai bằng chữ rằng *"Flush theo AD-35
KHÔNG thuộc loại này"* — nó chở đúng bộ đệm gõ, nên xuất xứ vẫn là **tôi dịch**. Điều hướng
không ghi `target_text` từ một nguồn khác ⇒ không đặt cột xuất xứ.

### AC8 nửa đầu ĐÃ ĐẠT, và cách chứng minh là đọc mã chứ không viết mã

`ruleById` (`GridPanel.vue:217-226`) tính lại mỗi khi `editorCaretSegmentId` đổi;
`resolveSegmentRule` (`editorSegments.ts`) cho `primary` **thắng** `confirmed`. ⇒ gọi
`setEditorCaret(newId)` là vạch đổi. **Đừng thêm một `class` hay một `:data-` nào cho việc
này** — nó đã có, và một đường thứ hai sẽ lệch ở lượt sửa kế tiếp.

⚠️ Hệ quả ngược, phải biết trước khi đo hiệu năng: chính `ruleById` là thứ làm một lượt đổi
caret tốn 706–770 ms trên 9.850 câu. Xem Cạm bẫy ④.

### 🔴 Chín cạm bẫy — mỗi cái có bằng chứng, không một khả năng lý thuyết

**① `draft` có HAI nghĩa ở hai tầng, và chúng KHÔNG cùng một tập hợp.**
- **CSDL** (`schema.rs:407-421`): `segment.status TEXT NOT NULL DEFAULT 'draft'`, hai giá trị
  hợp lệ `'draft'` | `'confirmed'`. `'draft'` ở đây = **chưa xác nhận**, gồm cả câu rỗng, cả
  câu đã gõ tay, cả câu điền từ TM.
- **Giao diện** (`editorSegments.ts::resolveSegmentRule`): `'draft'` là một `SegmentRuleValue`
  chỉ khớp câu **đã có chữ** (`targetText !== ''`).

⇒ DB-`draft` ⊋ UI-`draft`. 🔴 **Cài AC4 bằng `resolveSegmentRule(seg) === 'draft'` cho kết quả
NGƯỢC HOÀN TOÀN**: nó lọc ra đúng tập câu **đã dịch tay** và bỏ sót đúng tập câu **thật sự
chưa dịch**. `isUntranslated` (`segmentNavigation.ts:63-65`) đọc **thẳng cột `status` + chuỗi
rỗng**, có chủ ý, và chuỗi `'draft'` viết thẳng ở đó (không `import`) là **điều kiện kỹ thuật**
của luật "module thuần", không một lối tắt.

**② `keys.ts:510` nuốt mọi hợp âm không-`Mod` trong vùng gõ.** Đã dẫn ở Quyết định #1. Khuôn
thoát đã dựng **hai lần** ở Story 2.9 (`Backspace` đầu ô, `Escape`): bắt ở `onEditKeydown` rồi
**`dispatch()` chính id đã đăng ký** — *"hai cửa, MỘT command"*. 🔴 Gọi thẳng hàm là dựng một
đường thứ hai mà `check:commands` **không nhìn thấy** (Kiểm A chỉ canh `@click`).
⚠️ Và một `@keydown` mang thao tác thật là một món nợ **đã ghi**: `check-commands.mjs` in ra
mỗi lượt chạy *"ngày một `@keydown` mang thao tác thật xuất hiện, luật phải được xem lại"*.

**③ Driver không thay được ngón tay.** Mọi sự kiện `browser.keys()` giao đều `isTrusted: false`,
và một sự kiện không tin cậy **không có default action** — đo ở Story 2.9: `Backspace` qua
driver với caret **giữa** ô cũng không xoá gì. ⇒ Mọi mệnh đề về *"phím thật có bị hệ điều hành
nuốt không"* và *"`preventDefault()` có chặn được không"* **chỉ đóng bằng chữ ký tay của Ice**.
Ghi chúng riêng, đừng trộn vào bảng e2e.

**④ NFR2 — đường nóng của story này ĐANG vượt trần 15 lần, và chủ KHÔNG phải story này.**
Đo trên WKWebView thật (Story 2.5b): một lượt **đổi con trỏ** trên 9.850 câu tốn **706–770 ms**,
trần một frame là **50 ms**. Nguyên nhân đã chẩn: `:data-caret` buộc Vue tính lại `ruleById`
trên **toàn** danh sách. Story 2.10 thêm **hai** lệnh nữa đi đúng đường đó ⇒ mỗi cú bấm trả giá.
🔴 **Chủ là Story 2.4** *(nó sở hữu bộ đo NFR2/NFR18 — đừng dựng bộ đo thứ hai)*. Story này
**đo và ghi số** (Task 1.4), **không tự chấm đạt**, và **không tự vá**.

**⑤ Luật *"mọi ô nhớ mới phải qua `resetEditorPanel()`"* KHÔNG CÓ CỔNG NÀO CANH — và đã bị bỏ
sót HAI story liên tiếp.** `sourceCut` (2.8) và `regroupNotice`/`regroupError` (2.9) đều lọt,
cả hai lượt đi qua trọn mười một cổng. Chú thích tại chỗ (`editorPanelState.ts:505-509`) viết
thẳng: *"Một luật chỉ sống trong một khối chú thích là một luật sẽ bị quên lần thứ ba."*
⇒ Nếu Task 5 thêm ô nhớ, **lần thứ ba là lượt này**. Triệu chứng: mở Tác phẩm B và thấy thông
báo của Tác phẩm A **trước khi bấm gì**.

**⑥ `segment.ord` trong ảnh chụp webview thành CŨ sau một lượt gộp/tách** — món nợ đã ghi ở
`deferred-work.md`, chủ là *"story đầu tiên đọc `segment.ord` ở webview"*. ⇒ Task 2 duyệt bằng
**chỉ số mảng** (mảng do Rust `ORDER BY ord, id` trả về, `commands/segment.rs`), không đọc `ord`.
Đọc `ord` là **nhận** món nợ đó vào story này.

**⑦ `COMMAND_FLOOR` là một sàn, và một sàn sai thì im lặng.** Hôm nay **39**
(`scripts/check-commands.mjs:263`). Thêm hai command mà quên nâng ⇒ cổng vẫn xanh và sàn thành
vô nghĩa. **Đếm lại từ số cổng in ra**, không cộng nhẩm.

**⑧ `happy-dom` không phải WebKit, và nó không bố cục.** Mọi mệnh đề về **cuộn**, **hình học**,
**vị trí caret** thuộc **bàn đo / e2e trên WKWebView thật**. Chọn sai đường nghiệm thu là dựng
một nguồn sự thật thứ hai — bốn đường bốn vai, xem §Nghiệm thu.

**⑨ Một trích dẫn `tệp:dòng` trong chú thích TRÔI SỐ, và lượt soạn story này bắt được một ca
đang sống.** `LookupPanel.vue:270` và `:830` trích `DESIGN.md:342` cho luật *"không
`scroll-behavior`"*; dòng 342 hôm nay nói về chiều rộng `ch` của Chế độ đọc, và
`grep -rn "scroll-behavior" ux-designs/` trả **0 kết quả** — luật ấy **chưa từng tồn tại** ở
dạng toàn ứng dụng. ⇒ **Mở tệp mà đọc trước khi thừa kế một trích dẫn**, kể cả trích dẫn nằm
trong mã sản phẩm của chính kho này, kể cả khi nó mang dấu 🔴. Đây đúng là §Bẫy tài liệu mà
`project-context.md` đã đặt tên: *"Tin cây nguồn hiện tại hơn một chú thích."*

### Ranh giới phạm vi — bốn thứ KHÔNG thuộc story này

**① Tầng Rust — KHÔNG sửa một dòng.** Đã đo từ nguồn 2026-08-17:
`read_open_chapter_segments` (`src-tauri/src/commands/segment.rs`) đã trả **đủ** mọi trường
điều hướng cần: `status` · `target_text` · `is_omitted` · `retired_at` · `ord`, và nó đã lọc
`retired_at IS NULL` **ở tầng SQL**. `PROJECT_MIGRATIONS` (`core/store/schema.rs`) có **mười**
bước `[1,2,3,5,6,7,8,9,10,11]`, đích **11** ⇒ số kế tiếp là **12**, và story này **không tiêu
nó**. 🔴 Một số `cargo test` nhúc nhích ở đây nghĩa là đã đi lạc — xem Task 7.5.
*(Nguồn sự thật là `PROJECT_MIGRATIONS`, **không** dòng này. Đo lại ở Task 0.1.)*

**② `⌘Z` / mô hình hoàn tác.** AD-48 **chưa tồn tại**; hồ sơ bàn giao đã soạn ở
`planning-artifacts/ad-brief-2026-08-17-mo-hinh-hoan-tac.md`. **Chủ: Ice phân định → Winston
soạn AD.** Story này không chạm.

**③ Vá NFR2.** Chủ là Story 2.4. Xem Cạm bẫy ④.

**④ Sửa `epics.md` / `prd.md`.** 🔴 *"Năng lực chưa dựng ≠ lệch spec."* Một AC mô tả đích đến
không sai chỉ vì đường đi chưa tới. Ghi nợ **có chủ**, đừng sửa spec cho khớp mã.

### Nghiệm thu — bốn đường, bốn vai, chọn đúng đường

| Mệnh đề | Đường | Vì sao KHÔNG đường khác |
|---|---|---|
| `nextSegmentId` / `prevSegmentId` đúng ở biên, ở câu về hưu, ở `fromId` không tồn tại | **vitest** (hàm thuần) | Không cần DOM. Tất định, tức thời |
| AC4 hai vế · AC5 bỏ qua `isOmitted` | **vitest** — mở rộng `segmentNavigation.test.ts` | Đã có tiền lệ và fixture |
| Bất biến "ghi ô này dọn ô kia" | **vitest** — khuôn `editorRegroupNotice.test.ts` | Là logic state, không hình học |
| Lệnh chạy được từ bàn phím, caret đúng chỗ | **e2e** WKWebView | `keys.ts` + `contenteditable` chỉ thật trên engine thật |
| **Cuộn** tới hàng, không hiệu ứng, đã-trong-vùng-nhìn thì đứng yên | **bàn đo + e2e** | 🔴 `happy-dom` không bố cục |
| `⌥↓` có bị hệ điều hành nuốt không · `preventDefault()` có chặn phím thật không | **chữ ký tay của Ice** | Cạm bẫy ③ — driver không thay được ngón tay |
| Một frame có vượt 50 ms không | **bàn đo của Story 2.4** | Đừng dựng bộ đo thứ hai |

### Số nghiệm thu — baseline, đo lại từ nguồn 2026-08-17 trên HEAD `a664dac`

| Đường | Số |
|---|---|
| `cargo test --locked` | **401 passed / 0 failed / 5 ignored** |
| `cargo test --test segment_contract` | **121** |
| `npx vitest run` | **199 / 199**, **19** tệp |
| `COMMAND_FLOOR` | **39** |
| Cổng đọc-tệp | **9** xanh (+ `check:scope`, `check:scope:bundled` chạy tay) |
| `PROJECT_MIGRATIONS` | 10 bước, đích **11** ⇒ kế tiếp **12** |

🔴 **Đo lại ở Task 0.1, đừng chép bảng này.** Bảng này là một **ảnh chụp**; một lượt chép là
đúng lớp lỗi mà `sprint-status.yaml` đã phải sửa bốn lần trong epic này.

### Git — trạng thái cây khi story này được soạn

Cây **sạch** tại `a664dac` *("feat: add tests for originalOffsets mapping and source cut
behavior")*. Toàn bộ Story 2.9 đã vào ba commit (`4d72cd4` · `0f67808` · `a664dac`).
⇒ Diff của Story 2.10 sẽ **đọc được một mình**, không cần commit dọn trước.
*(Luật: cây bẩn trước một story ⇒ commit riêng, trước, và **hỏi Ice** trước khi commit.)*

### Project Structure Notes

| Tệp | Việc | Ghi chú bắt buộc |
|---|---|---|
| `src/panels/segmentNavigation.ts` | **UPDATE** — thêm hai vị từ | 🔴 `import type` **duy nhất**. Một `import` giá trị làm tệp hết nạp được bằng Node trần |
| `src/panels/editorPanelState.ts` | **UPDATE** — hai hàm dời con trỏ, có thể một ô nhớ notice, `resetEditorPanel()` | Chép khuôn `goToNextUntranslated:1029-1035` |
| `src/commands/index.ts` | **UPDATE** — hai `register()`, sửa nhánh `console.info:1078-1081`, sửa khối chú thích `:1053-1065` đã hết đúng | Chỗ đăng ký **duy nhất**. Không import Vue/Tauri |
| `src/main.ts` | **UPDATE** — nối dep mới vào `installCommands(deps)` | 🔴 **Không** đăng ký trong `App.vue` (HMR gọi lần hai ⇒ `register()` ném) |
| `src/panels/GridPanel.vue` | **UPDATE** — cuộn cạnh watcher `:895-916`; cửa `onEditKeydown` nếu #1(b) | Hàng **không** phải một phần tử DOM |
| `src/StatusBar.vue` | **UPDATE** — bảng khoá ĐÓNG thứ ba nếu #4(b) | `Record` đầy đủ ⇒ `vue-tsc` canh |
| `src/i18n/vi.json` | **UPDATE** — `command.editor.*` + `panel.grid.*` | Phẳng, khoá chấm, không giá trị rỗng, placeholder `{ten_tham_so}` |
| `scripts/check-commands.mjs` | **UPDATE** — `COMMAND_FLOOR:263` | Đếm lại, đừng cộng nhẩm |
| `tests/frontend/segmentNavigation.test.ts` | **UPDATE** | |
| `e2e/specs/segment-navigation.e2e.mjs` | **NEW** | Spec riêng — trần timeout 120 s |
| `_bmad-output/implementation-artifacts/2-10-ban-do/` | **NEW** | Tạo tác một lượt đo. Không vào `package.json` |
| `deferred-work.md` | **UPDATE** | Nối tiếp, **không xoá** |
| `EXPERIENCE.md` | **UPDATE có điều kiện** | Chỉ theo Quyết định #2 đã ký |
| `src-tauri/**` | 🔴 **KHÔNG CHẠM** | Xem Ranh giới ① |

### References

- `_bmad-output/planning-artifacts/epics.md:2572-2620` — Story 2.10, chín mệnh đề AC
- `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md:454` — FR25 ·
  `:458-460` FR133 (*"lệnh 'câu chưa dịch kế tiếp' sẽ liên tục nhảy vào đúng những câu người
  dùng cố ý bỏ"*) · `:903` NFR17 (sàn khả năng tiếp cận)
- `ARCHITECTURE-SPINE.md` — AD-1 (frontend giữ *focus, cuộn, vùng chọn*) · AD-3 (`ord` là thứ
  tự, `id` là danh tính) · AD-34 §1 (mọi thao tác qua `CommandRegistry`) · AD-34 §2 (không để
  focus rơi về `body`) · AD-35 vế (d) (*rời segment* ⇒ flush)
- `ux-designs/…/EXPERIENCE.md:105-114` UX-DR19 sáu giá trị · `:198` (*tiêu điểm luôn nhìn
  thấy — vạch dọc `primary`*) · `:261-269` bảng Phím
- `ux-designs/…/DESIGN.md:373` (*"Vị trí cuộn — về đầu tức thì, không bao giờ cuộn có hiệu
  ứng"*, phạm vi **Panel Lookup**) · `:376` (`prefers-reduced-motion` bỏ **toàn bộ** hiệu ứng —
  thuộc sàn khả năng tiếp cận) · `:379` (ba cấm chung toàn ứng dụng) · `:391` (bảng vạch lề).
  ⚠️ **`DESIGN.md:342` KHÔNG phải nguồn cho luật cuộn** — xem Quyết định #7
- `_bmad-output/implementation-artifacts/deferred-work.md:2837-2847` — món nợ **chủ 2.10**
- `src/commands/index.ts:1046-1084` — `editor.next_untranslated` và khối giao món nợ `:1053-1065`
- `src/panels/segmentNavigation.ts` — toàn tệp, đặc biệt `:44-62` (định nghĩa *chưa dịch*) và
  `:74-89` (không quay vòng · bỏ qua về hưu · bỏ qua cắt bỏ, và **vì sao lọc ở hàm gọi**)
- `src/panels/editorPanelState.ts:130-150` (`setEditorCaret` = AD-35 vế d) · `:1013-1035`
- `src/panels/LookupPanel.vue:269-283` — tiền lệ *cuộn tức thì* duy nhất của kho
- `_bmad-output/planning-artifacts/ad-brief-2026-08-17-mo-hinh-hoan-tac.md` — AD-48, chủ Ice
- `_bmad-output/project-context.md` — luật kho, đọc **trước** dòng mã đầu tiên

### Thông tin kỹ thuật mới nhất — `scrollIntoView` (tra 2026-08-17)

Ba mệnh đề dùng được cho Quyết định #7, và **cả ba vẫn phải đo lại trên WKWebView thật**:

1. `behavior: 'auto'` (mặc định) **uỷ quyền cho CSS `scroll-behavior`** của phần tử. ⇒ Một
   dòng `scroll-behavior: smooth` thêm vào bất kỳ đâu trong tương lai **làm hỏng AC8 im lặng**,
   và **không cổng nào canh giá trị đó** (đã đo — Quyết định #7). Đây là lý do Task 4.2 đòi
   `'instant'` **tường minh**.
   ⚠️ Cùng lý do, xét `prefers-reduced-motion` (`DESIGN.md:376`, *sàn khả năng tiếp cận*): với
   `'instant'` tường minh thì vế này **không cần nhánh riêng** — nó đã tức thì ở mọi cấu hình.
2. `block: 'nearest'` chọn mép gần nhất và **không cuộn gì** khi phần tử đã trong vùng nhìn —
   đúng thứ một phím bấm liên tục cần.
3. Từ điển tuỳ chọn (object argument) yêu cầu **Safari 15.4+**; các bản cũ hơn chỉ nhận đối số
   boolean. Kho đang chạy WKWebView **605.1.15** ⇒ dự kiến đủ, nhưng ⚠️ **một bảng tương thích
   không phải một phép đo** — đây là kho đã ba lần thấy `subgrid` bất đồng giữa engine.

Sources: [MDN — Element.scrollIntoView()](https://developer.mozilla.org/en-US/docs/Web/API/Element/scrollIntoView) ·
[Apple Developer Forums — Safari scroll-behavior](https://developer.apple.com/forums/thread/703294)

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code) — bắt đầu 2026-08-18.

### Debug Log References

#### Task 0.1 — đo lại từ nguồn, 2026-08-18 (HEAD `a664dac`)

Đo bằng lệnh, không chép bảng §Số nghiệm thu. Bốn số story yêu cầu, cộng ba số đối chứng:

| Thứ | Lệnh | Số đo 2026-08-18 | Bảng story (2026-08-17) |
|---|---|---|---|
| Số `AD` trong spine | `grep -c "^### AD-" …/ARCHITECTURE-SPINE.md` | **47** | 47 ✅ |
| `COMMAND_FLOOR` | `scripts/check-commands.mjs:263` | **39** | 39 ✅ |
| Command **thật sự** đã đăng ký | số `check:commands` in ra | **47** | *(bảng không có ô này)* |
| `PROJECT_MIGRATIONS` | `schema.rs:849-911`, `to_version` | **10 bước** `[1,2,3,5,6,7,8,9,10,11]`, đích **11** ⇒ kế tiếp **12** | khớp ✅ |
| `npx vitest run` | | **199 / 199**, **19** tệp | khớp ✅ |
| Tầm quét `check:commands` | | 16 `.vue` + 39 `.ts` · 25 `@click` · 34 `dispatch()` · 6 điểm vào focus | — |

⚠️ **Ô "command thật sự đã đăng ký" là ô bảng story không khai, và tôi đã đọc SAI nó một lượt —
sửa ở đây thay vì để nguyên.**

Đo được: `COMMAND_FLOOR` = **39**, số command thật = **47**. Lượt đọc đầu của tôi kết luận
*"sàn thấp hơn thực tế 8 đơn vị ⇒ hôm nay nó không canh được gì"* và đề xuất nâng thẳng lên
**49**. 🔴 **Sai.** Doc-comment của chính cơ chế đó (`check-commands.mjs:192-198`) viết rằng sàn
là một **cận dưới CÓ CHỦ Ý**, đặt ở **~80–85 %** số thật, chính vì *"một lượt quét hỏng (glob
sai, `SKIP_DIRS` nuốt nhầm) tụt sâu hơn khoảng đó rất nhiều"*. ⇒ 39/47 = **83 %** là **đúng
thiết kế**, không một khuyết tật. Một sàn đặt **bằng** số thật đổi một cận dưới thành một phép
so bằng, và mọi story thêm command sau đó sẽ đỏ oan.

⇒ **Số đúng sau story này là 41** (49 × 83,7 %), không phải 49. Đã sửa ở Task 3.5 kèm khối lý do
tại chỗ. Bài học đúng lớp `project-context.md` §*"cây nguồn thắng"*: ở đây "cây nguồn" là
doc-comment của chính cơ chế đang sửa, và một phép đo **không kèm ngữ cảnh của thước** vẫn là
một phép đo đọc sai được.

⚠️ Commit `ccffa23` mang mệnh đề sai này trong thông điệp. Không sửa lịch sử — mệnh đề được
đính chính **tại đây** và tại `check-commands.mjs`, đúng luật *"chú thích hết đúng thì sửa tại
chỗ kèm ngày và lý do"*.

#### Task 0.1b — đối chứng ba trích dẫn của tệp story trước khi dựa vào chúng

Luật §Bẫy tài liệu và Cạm bẫy ⑨ áp cho **chính tệp story này**, nên ba mệnh đề nền được mở tệp
kiểm lại thay vì thừa kế:

- ✅ **`keys.ts:415` + `:510` đúng nguyên văn.** `lacksPrimaryMod = (m) => !m.meta && !m.ctrl`
  ở `:415`; `if (lacksPrimaryMod(entry.mods) && isTypingZone(event.target)) return false` ở `:510`.
  ⇒ Tiền đề của Quyết định #1 **đứng**.
- ✅ **`DESIGN.md:342` KHÔNG nói về cuộn** — dòng đó nguyên văn: *"Chế độ đọc giới hạn chiều rộng
  bằng `ch` chứ không bằng `px`…"*. `grep -rn "scroll-behavior"` trên **toàn** `_bmad-output/` ⇒
  **0 kết quả**; trên `src/` ⇒ **đúng 2 kết quả**, cả hai là **chú thích** ở
  `LookupPanel.vue:270` và `:830`, không một dòng CSS nào. ⇒ Mệnh đề *"không luật nào canh
  `scroll-behavior`"* **đã tái đo và đứng**; Task 4.3 có việc thật.
- ✅ **`EXPERIENCE.md:261-269` có đúng bảy hàng** và **không hàng nào** cho *"segment kế tiếp"* /
  *"segment trước đó"*. ⇒ Tiền đề của Quyết định #2 đứng.

#### Task 0.1c — trạng thái cây git lúc bắt đầu, lệch với §Git của story

Story viết *"cây **sạch** tại `a664dac`"*. Đo lúc bắt đầu thi công:

```
 M _bmad-output/implementation-artifacts/sprint-status.yaml
?? _bmad-output/implementation-artifacts/2-10-dieu-huong-segment.md
```

⇒ Hai tệp này **là tạo tác của chính lượt `create-story`** *(tệp story + dòng `ready-for-dev`)*,
không phải việc dở của một story khác. Nên đây **không** phải ca *"cây bẩn trước story"* mà luật
git nhắm tới. ⚠️ Vẫn nêu ra thay vì tự quyết: **đưa vào Câu hỏi cho Ice** — có muốn commit lượt
create-story thành một commit riêng trước khi diff của 2.10 bắt đầu không.

#### Task 0.2 / 0.4 — CHỮ KÝ CỦA ICE, năm quyết định không phụ thuộc bàn đo (2026-08-18)

Ghi kèm **đường bị loại và lý do loại**, theo đúng luật §0.8 mục 3.

**#2 — phím mặc định cho `next_segment` / `prev_segment` ⇒ đường (c): KHÔNG gán phím nào.**
Chỉ đăng ký command; người dùng tự gán qua màn hình phím tắt của Story 1.21.
- Loại **(a) `⌥↑`/`⌥↓`-đối xứng**: hai lý do, cái thứ hai là chặn cứng. ① thừa hưởng nguyên
  vấn đề Quyết định #1 *(`Alt` không phải phím bổ trợ chính — `keys.ts:415`)*. ② 🔴 `⌥↓`
  **đã bị** `editor.next_untranslated` chiếm, và `createKeymap` **ném** khi hai command giành
  một hợp âm (`keys.ts:484-489`) — đường này không chỉ dở, nó không khởi động được.
- Loại **(b) hợp âm mang `Mod`**: hợp lệ về kỹ thuật *(đi qua `keys.ts:510` sạch)*, loại vì
  nó buộc một hàng mới vào bảng Phím của `EXPERIENCE.md` — tài liệu thuộc quyền Ice — để đổi
  lấy một hợp âm bốn ngón mà chưa ai xin.
- ⇒ **Hệ quả cho Task 8.3: `EXPERIENCE.md` KHÔNG bị sửa ở story này.** Task 8.3 đóng bằng
  *"không có việc"*, không phải bằng một lượt sửa.

**#3 — next/prev có bỏ qua câu đã cắt bỏ không ⇒ đường (a): KHÔNG bỏ qua.**
`nextSegmentId`/`prevSegmentId` là điều hướng **vị trí** và đi qua mọi hàng còn sống;
`nextUntranslatedId` là điều hướng **theo việc** và giữ nguyên `if (s.isOmitted) continue`.
- Loại **(b) cả ba cùng bỏ qua**: đối xứng đẹp hơn, nhưng nó đóng **đường bàn phím duy nhất**
  tới một câu đã cắt bỏ ⇒ `editor.restore_segment` *(chạy trên câu đang có caret)* chỉ còn với
  tới được bằng chuột ⇒ FR133 vế *"đảo ngược được bất cứ lúc nào"* và NFR17 (`prd.md:903`)
  hỏng **im lặng**, không cổng nào đỏ.

**#4 — ô nhớ cho AC6/AC7 ⇒ đường (b): ô thứ ba `navNotice`, cộng MỘT CỬA GHI DUY NHẤT.**
- Loại **(a) nới `RegroupNotice`**: rẻ nhất, loại vì sai ngữ nghĩa — tên ô nghĩa là *"gộp/tách"*.
  Và chính chú thích `StatusBar.vue:127-130` đã từ chối đúng lối tắt này một lần rồi *(nới
  `CONFIRM_NOTICE_KEYS` ra `string` làm mất chốt `vue-tsc`)*; đi lại nó là bỏ qua một bài học
  đã trả giá.
- 🔴 **Chi phí đã nêu TRƯỚC khi ký và Ice nhận:** bất biến *"ai ghi một ô thì dọn ô còn lại"*
  hôm nay là **hai chiều** (`ghiRegroupNotice:1213`). Ô thứ ba làm nó thành **N chiều**.
  ⇒ Ký kèm việc dựng nó thành **một cửa duy nhất** — ba hàm ghi nhỏ gọi chung một chỗ đặt
  *"ô này thắng, hai ô kia về `null`"*, **không** ba lời gọi rải rác.
- 🔴 Cộng Cạm bẫy ⑤: `navNotice` phải có mặt trong `resetEditorPanel()` (`:443`). Đây là **lần
  thứ ba** cùng một luật — `sourceCut` (2.8) và `regroupNotice`/`regroupError` (2.9) đều đã lọt.

**#5 — hành vi ở biên cho next/prev ⇒ đường (a): con trỏ ở nguyên + BÁO trên thanh trạng thái.**
- Loại **(b) đứng yên im lặng**: thoả chữ AC7 nhưng người dùng không phân biệt được *"đã ở cuối
  Chương"* với *"phím hỏng"* — đúng lớp *"rỗng IM LẶNG"* mà §Critical của `project-context.md`
  gọi là lớp lỗi trung tâm của dự án.
- Loại **(c) quay vòng về đầu**: là lật một quyết định đã có chữ ký kèm lý do tại chỗ
  (`segmentNavigation.ts:74-77`). Không lật.

**#6 — "đi đâu khi hết Chương" ⇒ đường (c): BÁO bằng chữ, chấp nhận vạch ở lại `primary`.**
Dùng lại đúng `navNotice` vừa ký ở #4 ⇒ không thêm một cơ chế nào.
- Loại **(a) đảo thứ tự `primary`/`confirmed`**: 🔴 quyết định có chữ ký, không đảo.
- Loại **(b) `⌘Enter` ở câu cuối nhả caret** — và lý do loại **KHÔNG** phải lý do story dự đoán:

  🔵 **ĐO 2026-08-18 — vế "nhả caret có bỏ rơi bộ đệm gõ không" BỊ BÁC.** Story cảnh báo rằng
  `setEditorCaret(null)` không flush (`editorPanelState.ts:145-150` — đúng, `null` không kích
  vế (d) của AD-35). Nhưng ở **đúng đường này** điều đó vô hại: `confirmCurrentSegmentUnguarded`
  đã gọi `flushEditorBeforeDiscreteWrite()` ở **bước ①**, trước cả lượt IPC, và trả sớm với
  `'flush-failed'`/`'still-dirty'` nếu tập chờ chưa sạch. ⇒ Tại điểm ta sẽ gọi `setEditorCaret(null)`
  *(bước ③, `:871-875`)*, **tập chờ đã sạch theo cấu tạo**. Lo ngại này không phải cái chặn.

  ⚠️ **Cái chặn thật là một rủi ro KHÁC, chỉ lộ ra khi đọc `onSelectionChange`:** đường (b) tạo
  ra trạng thái `caretSegmentId === null` **trong khi DOM focus vẫn nằm trong ô** — hai nguồn
  sự thật về *"người dùng đang ở đâu"* nói ngược nhau, và **không cổng nào canh câu đó**. Cộng
  thêm: `onSelectionChange` (`GridPanel.vue:875-882`) đặt lại `id` ở **lượt dịch caret kế tiếp**,
  nên hiệu lực thị giác của đường (b) chỉ kéo dài tới khi người dùng chạm vào ô. ⇒ Nó mua một
  vế thị giác **tạm thời** bằng một trạng thái lệch **thường trực**.

- 🟡 **Hệ quả phải ghi trung thực:** đường (c) đóng món nợ **MỘT NỬA**. Vế *"vạch lề chuyển
  `confirmed`"* của AC1 Story 2.5 **vẫn hụt** ở câu cuối Chương; chỉ vế *"người dùng biết
  chuyện gì vừa xảy ra"* được đóng. ⇒ Task 6.2 nối `deferred-work.md` bằng 🟡, **không** ✅,
  và liệt kê phần còn hở kèm chủ mới.

**Xác nhận kèm theo (không phải quyết định):**
- Story **không sửa một dòng Rust** — Ice xác nhận. `PROJECT_MIGRATIONS` không tiêu số 12.
- Bảng §Số nghiệm thu **đã đo lại** ở Task 0.1 thay vì tin, và cả bốn số đều khớp.
- **Cây git:** Ice chọn commit lượt `create-story` thành một commit riêng **trước** khi thi công,
  để diff mã của 2.10 đọc được một mình.

#### Task 0.3 / 1.x — CHỮ KÝ cho hai quyết định còn lại, SAU bàn đo (2026-08-18)

Số đầy đủ ở `2-10-ban-do/README.md`. Tóm tắt vế quyết định:

**#7 — cơ chế cuộn ⇒ đường (b): `scrollTop` tính tay. `scrollIntoView` BỊ LOẠI, và lý do là
một phép đo, không một sở thích.**

Vòng 1 cho đường (a) bốn số sạch *(cuộn đúng track hàng dù là `subgrid` · hai cột cùng `top`
759 · 12/12 mẫu `scrollTop` giống nhau ⇒ không hiệu ứng · `nearest` đứng yên khi hàng đã trong
vùng nhìn, có đối chứng âm `block:'center'` **có** dịch)*. Nó đáng lẽ đã được ký.

🔴 **Một trường phòng xa lật kết quả:** `hopCoTuDichKhong` trả **`true`** — `.grid-scroll` tự
dịch **18 px**. Vòng 2 đi ngược **28 nút** tổ tiên và tìm ra thủ phạm: **`SECTION.panel`**, và
nó mang **`overflow-y: hidden`**, `scrollTop` **0 → 18**.

⚠️ *"`overflow: hidden`"* nghĩa là **không vẽ thanh cuộn**, **không** phải *"không cuộn được"*.
`scrollIntoView` cuộn **mọi** tổ tiên cuộn được. ⇒ Thân panel dịch lên 18 px và **ở lại đó**:
không thanh cuộn để kéo, không cử chỉ nào đưa về. Đo thêm: chỉ xảy ra **một lần**, ở **lượt điều
hướng đầu tiên** của phiên — tức mọi người dùng đều gặp, đúng một lần.

Đường (b) đo trên **cùng** hàng: `scrollTop` **1242** *(trùng đường a)* · hàng nằm trọn trong
hộp · 12/12 mẫu giống nhau · và `nutDaDoi` chỉ có **một** phần tử là chính `.grid-scroll`,
`top` **101 → 101**. **Không một tổ tiên nào đổi.**

- 🔵 **Một lo ngại của story bị bác:** story viết đường (b) *"cũng đọc CSS `scroll-behavior`…
  nó chỉ đổi chỗ rủi ro"*. Đúng về nguyên tắc, nhưng đo được `duongB_coHieuUng: false` và **cả
  hai đường phụ thuộc `scroll-behavior` y như nhau** ⇒ nó không phải một lý lẽ **phân biệt** hai
  đường, nên không phải một lý lẽ chống (b).
- Loại **(a)+dọn tổ tiên sau mỗi lượt**: vá triệu chứng, hai thao tác cho một việc, và nó để
  nguyên câu hỏi thật *(vì sao `SECTION.panel` có `scrollHeight > clientHeight`)* — một dấu hiệu
  bố cục đáng ngờ **không thuộc phạm vi story này**. ⇒ Ghi nợ có chủ ở Task 8.1.

**#1 — `⌥↓` trong vùng gõ ⇒ đường (c): đổi sang hợp âm mang `Mod`.**

🔴 **Luật nuốt nay là một phép đo, không một suy luận từ mã.** Biến duy nhất khác nhau giữa hai
ca là `event.target`:

| ca | target | `defaultPrevented` | vạch lề |
|---|---|---|---|
| Ⓘ | ô đang gõ (`isContentEditable`) | `false` | 0 → 0, **không dời** |
| Ⓙ **đối chứng dương** | `body` | `true` | **0 → 1** |

Và tiền đề của (c) cũng đo trong **cùng một ô**: `Mod+Enter` ⇒ `defaultPrevented: true` *(đi
qua)*; `Alt+ArrowDown` ⇒ `false` *(bị nuốt)*; `F9` chưa đăng ký ⇒ `false` *(đối chứng âm, chứng
minh `true` ở hàng đầu là tín hiệu thật)*.

- Loại **(b) cửa thứ hai ở `onEditKeydown`**: khuôn có sẵn và rẻ, nhưng nó **cướp** `⌥↓` —
  *"xuống cuối đoạn"* của macOS trong một ô văn bản. Và 🔴 vế *"`preventDefault()` có chặn nổi
  không"* **không đường nghiệm thu nào của dự án đóng được** *(mọi sự kiện driver `isTrusted:
  false` ⇒ không có default action ⇒ phép kiểm sẽ trả CÓ trên mọi engine)*. Ký (b) là ký một
  mệnh đề chưa ai kiểm được.
- Loại **(a) giữ nguyên**: đo được rằng tính năng **chết ở ca thường nhất** của FR25.
- 🔵 **Rủi ro *"mồ côi binding người dùng"* NHỎ HƠN story nêu — đo từ kiểu:** `ChordOverrides`
  là `Record<CommandId, readonly string[]>` (`keys.ts:457`), khoá theo **id**. Story này giữ
  **nguyên id**, chỉ đổi `spec.keys`, và `createKeymap` ưu tiên `overrides` qua `hasOwnProperty`
  (`:474-477`) ⇒ ai đã tự gán phím thì **giữ nguyên** phím của họ. Chỉ **mặc định** đổi.
- ⚠️ **Hệ quả sửa lại một câu tôi viết ở #2:** Task 8.3 nói *"`EXPERIENCE.md` chỉ sửa theo Quyết
  định #2 đã ký"*, và ở #2 tôi kết luận *"không sửa"*. Chữ ký #1(c) **lật vế đó**:
  `EXPERIENCE.md:269` *(hàng `⌥↓`)* nay mang một phím đã hết đúng ⇒ **sửa tại chỗ kèm 🔵 + ngày
  + lý do**, đúng khuôn hàng `⌘/` ngay trên nó. Đây là **sửa một mệnh đề đã hết đúng**, không
  phải thêm một hàng mới — vẫn nằm trong ngoại lệ mà story cho phép.

#### Phương pháp — hai lượt sửa THƯỚC, không một vòng chẩn đoán bị bác

LUẬT DỪNG đếm những vòng mà một giả thuyết về **sản phẩm** bị phép đo bác. **Đếm hôm nay: 0.**
Hai lượt dưới đây là sửa **thước**, và phân biệt ấy có tiền lệ *(`2-8-ban-do` §Vòng 1 ·
`2-5d-ban-do` §Debug Log Ⓐ · `2-9-ban-do` §Vòng 1)*:

- **Thước 1 — `[data-caret]` không tồn tại.** Bản đầu của bàn đo hỏi
  `querySelectorAll('[data-caret]')`, thừa kế cách gọi tên của Cạm bẫy ④. Mở `GridPanel.vue` ra
  thì vạch lề là một **`class`** *(`rule-primary`)* trên `<div>` con của **cột riêng**
  `.col-rule` (`:1320-1324`), khớp hàng bằng **thứ tự tài liệu**. Một `[data-caret]` trả mảng
  rỗng ở **mọi** trạng thái ⇒ thước sẽ cho cùng một số ở hai thế giới khác nhau. Bắt được **trước
  khi chạy**, bằng cách mở tệp thay vì tin trích dẫn — đúng Cạm bẫy ⑨.
- **Thước 2 — `browser.keys(['Alt','ArrowDown'])` không gửi một hợp âm.** Nó giao hai `keydown`
  rời, cái thứ hai mang `altKey: false`. Bắt được **sau khi chạy**, và bắt được **chỉ vì** có
  đối chứng dương Ⓒ: nó cũng đứng yên. Không có Ⓒ, vòng 1 sẽ được đọc thành *"đã xác nhận luật
  vùng gõ nuốt `⌥↓`"* — một kết luận **tình cờ đúng** rút từ một con số **rỗng**, tức nguy hiểm
  hơn một kết luận sai.

### Completion Notes List

#### Phạm vi thật hoá ra là BỐN việc, không năm — và việc bị loại bị loại bằng phép đo

Story mở đầu bằng một mệnh đề đúng và quan trọng: *"một nửa story này ĐÃ ĐƯỢC DỰNG ở Story
2.5b"*. Lượt thi công tìm ra rằng mệnh đề ấy **đúng hơn story tưởng** — vế ③ *"cuộn tới hàng"*
cũng đã được dựng sẵn, chỉ là ở một chỗ không ai nghĩ tới.

| # | Việc story giao | Kết cục |
|---|---|---|
| ① | Hai lệnh tuần tự `next_segment` / `prev_segment` | ✅ dựng |
| ② | Báo rõ ra MÀN HÌNH (AC6 đang hỏng) | ✅ dựng — ô nhớ thứ ba + **một cửa ghi duy nhất** |
| ③ | Cuộn tới hàng, tức thì | 🔵 **KHÔNG dựng — đã có sẵn.** `focus()` lo, và lo **khéo hơn** một công thức tự cài. Ice ký gỡ |
| ④ | Phân xử bảng phím điều hướng | ✅ `⌥↓` → `⌘⌥↓`, và món nợ ở `commands/index.ts:1060-1065` đã trả |
| ⑤ | "Đi đâu khi hết Chương" | 🟡 đóng **một nửa** — bằng thông tin, không bằng màu |

#### Ba lượt ĐỘT BIẾN bắt được hai ca test vô hiệu, và một hàm suýt thành mã chết

Đây là kết quả có giá trị nhất của lượt thi công, và nó đến từ **Task 7.3**, không từ một lượt
đọc mã:

1. Ca e2e §Ⓒ *("vùng nhìn đã cuộn" + "hàng nằm trọn trong hộp")* **xanh**. Gỡ lời gọi
   `cuonToiHang` khỏi mã sản phẩm ⇒ **4/4 vẫn xanh**. ⇒ Ca ấy đo `focus()`, không đo hàm mới.
2. Thêm `focus({preventScroll: true})` để buộc công thức phải làm việc, rồi gỡ lại ⇒ **4/4 vẫn
   xanh**. ⇒ Ngay cả vế *nearest* cũng không phân biệt được ở ca đó.
3. Bàn đo vòng 3 chỉ ra vì sao, và con số quyết định **không** phải *"focus có cuộn không"* mà
   là **cuộn TỚI ĐÂU**: `focus()` một mình ⇒ `scrollTop` **1569** *(căn giữa; đối chứng độc lập
   `block:'center'` = 1571)*; `preventScroll` + công thức ⇒ **1242** *(nearest)*; `preventScroll`
   một mình ⇒ **0**, hàng **không** trong vùng nhìn *(đối chứng ÂM — nó là ca làm bảng đọc được)*.

🔴 **Và hành vi sẵn có TỐT HƠN đường tự cài:** WebKit **căn giữa khi hàng ở xa** *(người dùng có
ngữ cảnh trên dưới)* và dùng **nearest khi nó chỉ vừa ló khỏi mép** *(đo ở ca e2e Ⓔ: đi xuống một
hàng dịch **đúng 38 px** = một chiều cao hàng, không phải hàng trăm)*. Công thức tự cài ép
*nearest* ở **mọi** ca ⇒ dán hàng đích vào sát mép dưới sau một lượt nhảy xa.

⇒ **Ice ký gỡ cả `cuonToiHang` lẫn `preventScroll`.** Bài học phương pháp đã ghi vào sổ nợ: *một
ca test khẳng định "X đã xảy ra" KHÔNG chứng minh "mã CỦA TÔI làm X"*, và đường phân biệt duy
nhất là gỡ mã ra rồi chạy lại.

#### Bảy chữ ký, và ba trong số đó lật một mệnh đề của chính tệp story

Tất cả ở §Task 0.2/0.3/0.4 trên, kèm đường bị loại. Ba chỗ story đoán sai, mỗi chỗ có số:

- **#6** — story lo `setEditorCaret(null)` *"bỏ rơi bộ đệm gõ"*. **Bác:** bước ① của
  `confirmCurrentSegmentUnguarded` đã flush trước lượt IPC. Rủi ro thật nằm chỗ khác *(trạng thái
  `caretSegmentId === null` trong khi DOM focus vẫn trong ô)*.
- **#7** — story lo đường (b) *"cũng đọc `scroll-behavior`, chỉ đổi chỗ rủi ro"*. **Bác:** đúng
  về nguyên tắc nhưng **cả hai đường phụ thuộc như nhau**, nên nó không phân biệt được gì. Cái
  thật sự phân biệt là `SECTION.panel` (`overflow: hidden`) bị `scrollIntoView` cuộn 18 px.
- **#1** — story lo *"mồ côi binding người dùng đã gán"*. **Bác từ kiểu:** `ChordOverrides` khoá
  theo **command id**, và lượt này giữ nguyên id.

#### Hai lượt sửa THƯỚC, và cả hai bắt được nhờ ĐỐI CHỨNG DƯƠNG

LUẬT DỪNG đếm vòng mà một giả thuyết **sản phẩm** bị bác. **Đếm cuối story: 0.**

- `browser.keys(['Alt','ArrowDown'])` giao hai `keydown` rời, cái sau `altKey: false` ⇒ hợp âm
  **chưa từng được gửi**. Bắt được **chỉ vì** có đối chứng dương *(caret ngoài vùng gõ cũng đứng
  yên)*. Không có nó, vòng 1 sẽ được đọc thành một kết luận **tình cờ đúng** rút từ số rỗng.
- Bàn đo vòng 3A *(async + `blur()` + rAF lồng)* `timeout` cả ba ca ⇒ thay thước, không vá.
- Cộng một ca bị **gỡ có tên**: phép lấy mẫu 12 khung hình cho `focus()` treo ở **hai** khuôn
  khác nhau; câu hỏi của nó được đóng bằng một phép đo **mạnh hơn** đã có sẵn *(Ⓘ đọc `scrollTop`
  **đồng bộ** ngay sau `focus()` và thấy **giá trị cuối** — một lượt cuộn có hiệu ứng không thể
  đã tới đích trong cùng lượt thực thi đồng bộ)*.

#### Một kết luận của chính tôi bị sửa giữa chừng

Task 0.1 đọc *"`COMMAND_FLOOR` 39 / thật 47"* thành *"sàn không canh được gì"* và đề xuất nâng
lên **49**. Sai: doc-comment của chính cơ chế đó khai sàn là **cận dưới có chủ ý ở ~80–85 %**.
Số đúng là **41**. Đã sửa ở `check-commands.mjs` kèm khối lý do, và đính chính ở §Task 0.1.
⚠️ Commit `ccffa23` mang mệnh đề sai trong thông điệp; không sửa lịch sử, đính chính tại chỗ.

#### Số nghiệm thu cuối — đo lại, không chép

| Đường | Baseline (`a664dac`) | Sau story |
|---|---|---|
| `cargo test --locked` | 401 / 0 / 5 | **401 / 0 / 5** — 🔴 **không đổi**, xác nhận Ranh giới ① |
| `npx vitest run` | 199 / 199, 19 tệp | **224 / 224, 20 tệp** *(+25 ca, +1 tệp)* |
| Command đã đăng ký | 47 | **49** |
| `COMMAND_FLOOR` | 39 | **41** *(41/49 = 83,7 %)* |
| 9 cổng đọc-tệp | xanh | **xanh** |
| `check:scope` · `check:scope:bundled` | — | **xanh** *(chạy tay)* |
| `npm run build` · `vue-tsc` | — | **xanh** |
| e2e `segment-navigation` | — | **4 / 4** |
| Bàn đo | — | 6 spec, **4+3+4+3+4+4** *(hai vòng bỏ số vì hỏng thước)* |
| `PROJECT_MIGRATIONS` | 10 bước, đích 11 | **không đổi** — story không tiêu số 12 |

#### 🔴 CÒN MỞ — một việc, và nó cần Ice

**Task 4.5** *(nhìn bằng mắt rằng lượt cuộn không giật và hàng đích nằm trọn trong vùng nhìn)*
là mục **duy nhất** chưa tick. Nửa **đo được** của nó đã đóng *(hàng nằm trọn: e2e Ⓒ · tức thì:
bàn đo Ⓘ đọc đồng bộ · không giật khi bấm liên tục: e2e Ⓔ, dịch đúng 38 px)*. Nửa còn lại —
*"trông thế nào"* — không thước nào của dự án thay được mắt người.

### File List

**Mã sản phẩm**
- `src/panels/segmentNavigation.ts` — UPDATE: `nextSegmentId` · `prevSegmentId` · `buocTu`
- `src/panels/editorPanelState.ts` — UPDATE: `NavNotice` · `navNotice` · `datThongBao` *(cửa ghi
  duy nhất)* · `ghiNavNotice` · `danhSachDieuHuong` · `doiConTroToi` · `goToNextSegment` ·
  `goToPrevSegment` · ba vỏ `…CoBao` · `kyTrungCauCuoi` · bốn chỗ ghi cũ chuyển sang cửa duy nhất
- `src/commands/index.ts` — UPDATE: hai `register()` mới · `editor.next_untranslated` đổi hợp âm
  sang `Mod+Alt+ArrowDown` và **gỡ `console.info`** · hai dep mới · khối chú thích giao nợ đã sửa
- `src/main.ts` — UPDATE: nối ba dep bản `…CoBao`
- `src/StatusBar.vue` — UPDATE: `NAV_NOTICE_KEYS` *(bảng ĐÓNG thứ ba)* · `navNoticeKey` · nhánh
  `v-else-if` thứ ba
- `src/panels/GridPanel.vue` — UPDATE: khối §"AC8 NỬA SAU" *(vì sao KHÔNG có mã cuộn nào)*
- `src/panels/LookupPanel.vue` — UPDATE: 🔵 sửa **hai** chú thích trích `DESIGN.md:342` cho một
  luật chưa từng tồn tại
- `src/i18n/vi.json` — UPDATE: 2 khoá `command.editor.*` + 4 khoá `panel.grid.nav_*`
- `scripts/check-commands.mjs` — UPDATE: `COMMAND_FLOOR` 39 → 41

**Nghiệm thu**
- `tests/frontend/segmentNavigation.test.ts` — UPDATE: +11 ca cho hai vị từ mới
- `tests/frontend/editorNavNotice.test.ts` — **NEW**: 14 ca *(AC6 · AC7 · bất biến một-cửa sáu
  chiều · Cạm bẫy ⑤ · #6(c) có đối chứng âm · AC5-vs-#3 ở tầng state)*
- `e2e/specs/segment-navigation.e2e.mjs` — **NEW**: 4 ca trên WKWebView thật

**Bàn đo** *(tạo tác một lượt đo — không gì vào `package.json`)*
- `_bmad-output/implementation-artifacts/2-10-ban-do/README.md` — **NEW**
- `_bmad-output/implementation-artifacts/2-10-ban-do/danh-tinh-phien.mjs` — **NEW**
- `_bmad-output/implementation-artifacts/2-10-ban-do/cuon-toi-hang.e2e.mjs` — **NEW**
- `_bmad-output/implementation-artifacts/2-10-ban-do/cuon-vong2-to-tien.e2e.mjs` — **NEW**
- `_bmad-output/implementation-artifacts/2-10-ban-do/alt-mui-ten-trong-vung-go.e2e.mjs` — **NEW**
- `_bmad-output/implementation-artifacts/2-10-ban-do/alt-vong2-hop-am-that.e2e.mjs` — **NEW**
- `_bmad-output/implementation-artifacts/2-10-ban-do/doi-con-tro-chuong-lon.e2e.mjs` — **NEW**
- `_bmad-output/implementation-artifacts/2-10-ban-do/focus-co-tu-cuon-khong.e2e.mjs` — **NEW**
- `_bmad-output/implementation-artifacts/2-10-ban-do/probe-hop-am-driver.e2e.mjs` — **NEW**

**Tài liệu**
- `_bmad-output/implementation-artifacts/deferred-work.md` — UPDATE: 🟡 đóng một nửa món nợ chủ
  2.10, cộng **sáu** mục mới kèm chủ
- `_bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md` —
  UPDATE: 🔵 sửa hàng `⌥↓` → `⌘⌥↓`, thêm hàng hai lệnh tuần tự
- `_bmad-output/implementation-artifacts/2-10-dieu-huong-segment.md` — UPDATE: tệp này
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — UPDATE: trạng thái story

### Change Log

| Ngày | Việc |
|---|---|
| 2026-08-18 | Task 0 — đo lại bốn số từ nguồn; **bảy chữ ký** của Ice, mỗi cái kèm đường bị loại |
| 2026-08-18 | Task 1 — bàn đo WKWebView thật, **sáu spec**. Hai vòng bỏ số vì hỏng thước |
| 2026-08-18 | Task 2–3 — hai vị từ thuần + hai command; `⌥↓` → `⌘⌥↓` (Quyết định #1(c)) |
| 2026-08-18 | Task 5 — ô nhớ thứ ba `navNotice` **cộng một cửa ghi duy nhất**; gỡ `console.info` ⇒ **AC6 nay đạt** |
| 2026-08-18 | Task 4 — 🔵 **KHÔNG thêm mã cuộn nào.** Ba phép đo cho thấy `focus()` đã lo, và lo khéo hơn. Ice ký gỡ |
| 2026-08-18 | Task 6 — 🟡 món nợ *"đi đâu khi hết Chương"* đóng **một nửa**, phần còn hở giao Ice → Winston |
| 2026-08-18 | Task 7 — vitest 199 → **224**; e2e spec mới **4/4**; **ba lượt đột biến** bắt được hai ca vô hiệu |
| 2026-08-18 | Task 8 — sổ nợ +6 mục có chủ; `EXPERIENCE.md` sửa hàng phím; hai chú thích hết đúng ở `LookupPanel.vue` sửa tại chỗ |
| 2026-08-18 | 🔵 Sửa một kết luận của chính lượt thi công: `COMMAND_FLOOR` đúng là **41**, không 49 |

### Review Findings

Lượt rà mã **ba tầng** (Blind Hunter · Edge Case Hunter · Acceptance Auditor, cả ba chạy không
thấy hội thoại của nhau), 2026-08-17. Bảy con số của §Số nghiệm thu cuối **đã đo lại độc lập và
cả bảy đều đứng** *(`cargo test` 401/0/5 · vitest 224/224 20 tệp · 9 cổng xanh · build+`vue-tsc`
xanh · 49 command · `COMMAND_FLOOR` 41 · `src-tauri/**` 0 dòng diff)*.

- [x] [Review][Decision] ✅ **ĐÃ CHỐT 2026-08-17 — Ice ký đường 1: soạn hồ sơ bàn giao.** Tệp:
      `planning-artifacts/ad-brief-2026-08-17-vach-le-cau-cuoi-chuong.md`, đúng khuôn hai tiền lệ.
      🔴 Lượt soạn tìm ra **một vế chưa ai khai**: cột nhãn trạng thái khoá theo cùng
      `SegmentRuleValue` nên ở câu cuối Chương nó đọc *"đang sửa"*, và cột ấy **là** kênh khả năng
      tiếp cận *(vạch mang `aria-hidden`)* ⇒ khoảng hở chạm **AD-34 §2 + NFR17**, không chỉ màu.
      Sổ nợ đã sửa tại chỗ kèm 🔵. **Số `AD` KHÔNG được đặt** — hai hồ sơ đang xếp hàng cùng ghi
      *"kế tiếp là 48"*.
      *(Bối cảnh gốc của mục này:)* — sổ nợ ghi
      đúng khuôn 🟡 kèm *"Chủ: Ice phân định → Winston soạn AD"*, **không** xoá mục gốc. Nhưng
      hai tiền lệ gần nhất của cùng lớp cửa chặn đều sinh một tệp riêng
      (`planning-artifacts/ad-brief-2026-08-16-xuat-xu-ban-dich.md` ·
      `ad-brief-2026-08-17-mo-hinh-hoan-tac.md`); lượt này chỉ có một đoạn văn trong
      `deferred-work.md`. ⇒ Cần Ice phân định: vế *"vạch lề ở câu cuối Chương không chuyển
      `confirmed`"* có đủ tầm một `AD` *(và vì thế cần một hồ sơ bàn giao)*, hay nó ở lại một
      dòng nợ.

- [x] [Review][Patch] 🔴 **Dời con trỏ THÀNH CÔNG không dọn ô nhớ ⇒ thanh trạng thái nói dối,
      và mốc *"Đã lưu N giây trước"* bị che vô thời hạn** [`src/panels/editorPanelState.ts:1172-1175`]
- [x] [Review][Patch] **`deferred-work.md` gán công trạng cho một cơ chế ĐÃ BỊ GỠ, và nêu
      `focus()` là tác nhân trong khi bàn đo đo được nó vô can** [`_bmad-output/implementation-artifacts/deferred-work.md:4437-4441`]
- [x] [Review][Patch] **Task 3.6 tick `[x]` mà không một dòng mã nào chứng minh** — và điều kiện
      của nó *(`nếu` Ice ký #1 đường (b))* **không xảy ra**: chữ ký là (c), và chính đoạn ký
      liệt kê *"Loại (b) cửa thứ hai ở `onEditKeydown`"*. Đo: `git diff HEAD -- src/panels/GridPanel.vue`
      **không một dòng** chạm `onEditKeydown`. Khuôn đúng đã có tiền lệ ở 2.7/2.8: `[⊘]` kèm lý
      do [`2-10-dieu-huong-segment.md:348`]
- [x] [Review][Patch] **Mục nợ *"`⌥↓` thật có bị macOS nuốt không"* ghi *"Chủ: chưa cần"*** — theo
      chữ luật `project-context.md` §Sổ nợ *("Không có mục nào mồ côi")* thì đây là một mục không
      chủ [`deferred-work.md:4472`]
- [x] [Review][Patch] **Task 4.1 tick `[x]` trong khi Task 4 kết thúc bằng *"KHÔNG thêm mã cuộn
      nào"*** — bản thân story đã tự thuật đầy đủ ở §Completion Notes và Change Log, nên người
      đọc cả tệp không bị dẫn sai; nhưng ô tick đọc riêng vẫn là một tuyên bố sai
      [`2-10-dieu-huong-segment.md:352`]

- [x] [Review][Defer] **Đua giữa một lượt xác nhận đang bay và một lượt điều hướng đồng bộ kéo
      caret về chỗ cũ** [`src/panels/editorPanelState.ts:912-924`] — deferred, pre-existing *(lớp
      lỗi có từ Story 2.5; 2.10 mở rộng bề mặt chạm tới bằng hai lệnh bàn phím đồng bộ mới)*

**Ba mục bị bác, ghi ra thay vì im lặng bỏ:**

1. *"`regroupError` nằm ngoài cửa ghi duy nhất `datThongBao`"* — **bác**: `StatusBar.vue:167-172`
   chỉ đọc `regroupError` khi `regroupNotice === 'refused'`, và `regroupUnguarded:1590-1594` luôn
   đặt cặp đó ở cùng một chỗ. Một `regroupError` sót lại không hiển thị được gì.
2. *"Chương rỗng ⇒ câu báo `'at-last'` nói *'Đã ở câu cuối Chương'* trong khi không có câu nào"* —
   **bác vì chưa chứng minh chạm tới được**: không tìm được bằng chứng một Chương đang mở có 0
   segment, hai lệnh này lại **không có phím mặc định**; và nửa sau của câu *("không có câu nào
   phía dưới")* vẫn đúng. Sửa nó đòi một giá trị `NavNotice` thứ năm cộng một khoá `vi.json` —
   không đáng trước khi có một phép đo về khả năng chạm tới.
3. *"Bộ diff giao cho lượt rà thiếu `tests/frontend/segmentNavigation.test.ts`"* — **đúng, nhưng
   là khuyết tật của lượt rà, không của lượt thi công**. Đã tự vá và đối chiếu trực tiếp trong
   kho: **+11 ca `it()`**, khớp đúng số §File List khai.

- [x] [Review][Patch] 🔴 **Hàng cuối Chương tiếng Trung bị CẮT MẤT CHỮ — Ice tìm ra bằng mắt ở
      Task 4.5, và nó là phát hiện có giá trị nhất của lượt rà** [`src/panels/PanelFrame.vue:226`
      · `src/panels/GridPanel.vue:1572`] — `.panel-body` là khối thường, `.grid-scroll` xin
      `height: 100%` trong khi dải tab Hán Việt đã đẩy nó xuống **30 px** *(tab 18px +
      `--space-panel-block` 12px)* ⇒ tràn 30 px, `.panel` `overflow: hidden` cắt, **không cử chỉ
      nào lấy lại**. Cuộn tới đáy thì đáy **hàng cuối** trùng đúng vùng bị cắt.
      **Vá ở nguyên nhân** *(Ice ký đường (a) sau khi lật khỏi (b))*: `.panel-body` thành
      `display: flex; flex-direction: column`; `.grid-scroll` thành `flex: 1; min-height: 0`;
      `.load-error` khai `flex: none`. Đã rà **cả ba** panel tiêu thụ trước khi đổi.
      🔵 Lần **thứ hai** cùng lớp lỗi trong kho *(lần đầu `SourceHanViet.vue:853-861`, code review
      2026-08-06)* — đó là lý do vá ở khung chứ không ở một panel.
      ⚠️ **KHÔNG do Story 2.10 gây ra** — story không chạm `PanelFrame.vue` cũng không thêm mã
      cuộn nào. Nó là món nợ 18px của Epic 1, và câu hỏi mở của món nợ đó nay có câu trả lời.
      🔴 **Vế nghiệm thu CÒN HỞ:** không đường tự động nào thấy được lượt vá — xem §Số nghiệm thu.

**Số nghiệm thu SAU lượt vá — đo lại, không suy luận:**

| Đường | Trước lượt rà | Sau năm patch |
|---|---|---|
| `npx vitest run` | 224 / 224, 20 tệp | **225 / 225, 20 tệp** *(+1 ca: "thất bại RỒI thành công ⇒ câu cũ phải bị DỌN")* |
| 9 cổng đọc-tệp | xanh | **xanh** |
| `npm run build` · `vue-tsc` | xanh | **xanh** |
| `cargo test --locked` | 401 / 0 / 5 | **không chạy lại — `git diff HEAD -- src-tauri/` = 0 dòng**, Ranh giới ① còn nguyên |
| Command đã đăng ký · `COMMAND_FLOOR` | 49 · 41 | **không đổi** — lượt vá không thêm command nào |

🔴 **CẢNH BÁO ĐỌC BẢNG TRÊN — nó KHÔNG chứng minh lượt vá bố cục chạy.** Cả 225 ca và mười cổng
đều xanh **cả TRƯỚC lẫn SAU** lượt vá `.panel-body`/`.grid-scroll`, vì `happy-dom` **không bố cục**.
⇒ Một bảng toàn xanh ở đây nghĩa là *"không thứ gì cổng thấy được bị vỡ"*, **không** phải *"30 px
bị cắt đã hết"*. Mệnh đề sau chỉ đóng được bằng một phép đo trên WKWebView thật *(hai số:
`SECTION.panel.scrollHeight − clientHeight` phải là **0**, và hàng cuối nằm trọn trong hộp cuộn khi
đã cuộn tới đáy)* hoặc bằng mắt Ice lần thứ hai. Đã ghi vào sổ nợ 🟡 kèm chủ.

🔴 **Đột biến cho ca mới, vì một ca chưa bao giờ đỏ là một ca chưa ai biết nó canh gì:** đổi
`if (daDoi) datThongBao({})` thành `if (daDoi) {}` ⇒ **đúng một** ca đỏ *(ca mới)*, 14 ca còn lại
**xanh** — gồm cả ca `'dời được ⇒ KHÔNG nói gì'` ở `:113`, thứ đã đi qua trên sản phẩm hỏng. Đó là
bằng chứng ca mới canh đúng chốt vừa dựng, và là bằng chứng ca cũ **không** canh nó.

⚠️ **Một chỗ lệch nhỏ không tính là phát hiện:** mọi mốc ngày trong §Dev Agent Record ghi
**2026-08-18**, còn đồng hồ hệ thống lúc rà là **2026-08-17**. Ghi ra để người sau không đọc lượt
rà này thành *"rà trước khi viết"*.

---

## Câu hỏi cho Ice — chốt ở Task 0, trước dòng mã đầu tiên

1. **#1** `⌥↓` có được chạy trong vùng gõ không — và nếu (b), có chấp nhận cướp *"xuống cuối
   đoạn"* của macOS không? *(Nêu trước, ký sau Task 1.3.)*
2. **#2** Phím mặc định cho `next_segment` / `prev_segment` — hay không gán phím nào?
3. **#3** next/prev có bỏ qua câu đã cắt bỏ không? *(Bỏ qua ⇒ không đường bàn phím nào tới được
   câu đó để khôi phục.)*
4. **#4** Ô nhớ thứ ba trên thanh trạng thái, hay nới `RegroupNotice`?
5. **#5** Ở biên: đứng yên + báo, hay đứng yên im lặng?
6. **#6** "Đi đâu khi hết Chương" — nhả caret, hay báo bằng chữ và chấp nhận vạch `primary`?
7. **#7** `scrollIntoView({block:'nearest'})` hay `scrollTop` tính tay? *(Nêu trước, ký sau
   Task 1.2.)*

⚠️ **Hai câu hỏi không phải quyết định, chỉ là xác nhận:**
- Story này **không sửa một dòng Rust** — đúng ý không?
- Bảng "Số nghiệm thu" là ảnh chụp lúc soạn; dev **đo lại** ở Task 0.1 thay vì tin nó — đúng
  khuôn bốn story gần nhất.
