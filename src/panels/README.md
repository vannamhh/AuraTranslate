**Ba** panel: `Grid` · `Lookup` · `AiTranslation`.

> 🔵 **2026-08-15 (Story 2.5b) — BỐN → BA.** `SourcePanel.vue` + `EditorPanel.vue` gộp thành
> **`GridPanel.vue`**, một **lưới hai cột đối chiếu**: nguyên văn và bản dịch của cùng một câu
> nằm trên **cùng một HÀNG** (UX-DR13). Hai tệp cũ đã **gỡ khỏi cây nguồn**; mọi mệnh đề dưới
> đây nhắc tên chúng là **lịch sử**, không phải mô tả.

Component đặt tên `PascalCase.vue` (Consistency Conventions). Panel Lookup → `LookupPanel`.

---

## Ranh giới sở hữu

| | Story | Trạng thái |
|---|---|---|
| `PanelFrame.vue` — vỏ panel + **hợp đồng thị giác tiêu điểm** (AC5, UX-DR8, UX-DR17) | **1.6** | ✅ đã dựng |
| Panel thật trong `dockview`, preset bố cục, ngưỡng màn hình hẹp | **1.14** *(lưới 2×2 · bốn panel — **superseded** bởi 2.5b)* | ✅ đã dựng |
| Nội dung panel Source + tab Hán Việt | **1.16** | ✅ đã dựng |
| Nội dung panel Lookup — bản ghi từ điển có cấu trúc | **1.17** | ✅ đã dựng |
| Panel AiTranslation | **Epic 4** | ⬜ |
| Panel Editor *(trang liền mạch)* | **2.2 · 2.3** — **superseded** bởi 2.5b | 🔵 đã gỡ |
| **`GridPanel.vue`** — lưới hai cột `subgrid`, năm cột, ô bản dịch là editing host riêng | **2.5b** | ✅ đã dựng |
| Hợp đồng vùng chọn dùng chung + Auto-Lookup | **1.18** *(2.5b: đăng ký theo **CỘT**, `SourceHanViet` nhượng lượt đăng ký — xem `hanVietSurfaces.ts`)* | ✅ đã dựng |
| Tách từ tiếng Trung cho tab Hán Việt — double-click chọn cả CỤM TỪ | **1.18b** | ✅ đã dựng |
| Bật/tắt nguồn từ điển *(dải chip)* + bề mặt ghi công *(lớp phủ Attribution)* | **1.19** | ✅ đã dựng |
| **Ngắt đoạn của bản dịch** — `Enter` xuống dòng trong ô, cộng một cờ kết đoạn **riêng cho cột bản dịch** | **2.5d** | ✅ đã dựng |

**Story sở hữu nội dung: 1.14 → 2.5b.** `PanelFrame.vue` hôm nay là **vỏ**, không phải panel: thanh tiêu đề, tiêu đề `ui-md`, và thân **để trống**. `WorkspaceMode.vue` dựng **hai** `PanelFrame` — `panel.source` và `panel.editor`, đúng cặp *Nguyên văn | Bản dịch* mà UX-DR15 nói *"không bao giờ nhường"*. Hai chứ không bốn: một cái không đủ để nhìn thấy tương phản có/không tiêu điểm, bốn cái là dựng trước Story 1.14. **Story 1.14 thay chỗ hai cái này bằng bốn panel trong `dockview`** — và 🔵 **Story 2.5b thu bốn xuống BA**, gộp `panel.source` + `panel.editor` thành `panel.grid`.

---

## Hợp đồng thị giác tiêu điểm — đã đo, đừng sửa lại mà không đo lại

Panel có tiêu điểm: **vạch dọc 2px `primary` ở mép trái** + **tiêu đề chuyển `primary` in đậm**. **Không dùng viền bao quanh để báo tiêu điểm** (AC5, UX-DR8, `DESIGN.md §Components`).

- Vạch **không** được làm bằng `box-shadow`: Kiểm F của `check-tokens.mjs` cấm `box-shadow`/`text-shadow` **không có đường miễn trừ** (AC7 Story 1.4 — không elevation). Cách đúng là một `::before` `position:absolute; left:0; width:2px`.
- ⚠️ Trạng thái tiêu điểm đọc từ **DOM thật** (`focusin`/`focusout` với kiểm `relatedTarget`), không từ một cờ do ứng dụng tự giữ. Một cờ tự giữ sẽ vẫn sáng đúng một panel trong khi focus thật đã rơi ra ngoài — vạch dọc nói dối, và đúng nửa NFR17 mà AC5 tồn tại để giữ thì mất.
- ⚠️ `outline: none` **chỉ** ở gốc `tabindex="-1"` của panel, kèm lý do ngay cạnh dòng CSS. Một `*:focus { outline: none }` phá NFR17 mà **không cổng nào bắt được**.
- ⚠️ `font-weight` đang mượn `var(--weight-read-title)` (600) vì bộ token **không có** biến trọng lượng cho nhãn giao diện đậm (`ui-md` là 400, `ui-label` là 700). Viết thẳng `600` thì Kiểm B2 của `check-tokens.mjs` đỏ, và không khai một biến CSS cục bộ để lách cổng là đúng thứ AD-34 tồn tại để chặn. Story 1.14 quyết token thật — xem `deferred-work.md`.

## Phân tách panel — hai cơ chế, không phải một

Bốn biến do `applyTheme()` ghi (`--panel-border-width` · `--panel-border-color` · `--panel-gap` · `--panel-radius`) mang cơ chế của theme đang chạy: **sáng phân tách bằng NÉT 1px `outline`, tối bằng KHE 2px lộ `background` cộng bo 3px** (AC6 Story 1.4). Component **không bao giờ** phải biết mình đang ở theme nào. Đừng thay `gap: var(--panel-gap)` bằng một khoảng cách viết thẳng — làm vậy là thống nhất hai cơ chế về một cách làm, đúng thứ AC6 cấm.

*Đo thật 2026-08-04 trên panel thật (lần đầu — Story 1.4 mới chỉ nghiệm thu ở tầng token): sáng `border 1px #e2dccf`, khe `0px`; tối `border 0px`, khe `2px` lộ `#201e1b`, bo `3px`.*

## Chữ trong thân panel

⚠️ 🔵 *(2026-08-15: mệnh đề này thu lại còn **một** panel — `AiTranslation`. Cột bản dịch nay khai token `editor` trong `GridPanel.vue`.)* Panel `AiTranslation` hôm nay **để trống**. Ngày chúng đổ chữ vào, bề mặt đó **phải khai token `read-*` / `source-*` / `lookup-*` của chính nó**. Mặc định kế thừa từ `body` là `ui-md` ở giãn dòng **1.5** — dưới sàn 1.66 của AC5 Story 1.4 — và Kiểm E của `check-tokens.mjs` chỉ đọc `tokens.json` nên hoàn toàn mù với việc component nào đang kế thừa gì. Xem mục tương ứng ở `deferred-work.md`.

**Panel Source (Story 1.16) đã đóng nửa của nó**: nguyên văn khai `source-cjk` (tiếng Trung) hoặc `source-latin` (tiếng Anh — token thứ 16, Quyết định #6) tuỳ `work.source_lang`; tab Hán Việt khai `source-hanviet`. State (Chương đã nạp, tab/kiểu xem đang chọn, âm Hán Việt đã tra) sống ở `src/panels/sourcePanelState.ts` — module-level, sống sót qua một lượt đổi preset (AC9). Xem doc-comment đầu `GridPanel.vue`/`SourceHanViet.vue`.

**Panel Lookup (Story 1.17) đã dựng xong bản ghi có cấu trúc**: đầu mục khai `lookup-headword`, nghĩa khai `lookup-gloss`, ví dụ/trích dẫn/ghi chú/từ loại khai `lookup-example`, nhãn nguồn + nhãn ngoại ngữ khai `ui-label`. Khối một nguồn sống ở `src/panels/LookupRecord.vue` (nhận `group`/`senses` đã lọc sẵn); vùng đầu mục cố định + thanh nhịp + bốn trạng thái rỗng + hai banner sống ở `LookupPanel.vue`. State (truy vấn, kết quả pha một+hai, năm vị từ trạng thái) sống ở `src/panels/lookupPanelState.ts` — module-level, sống sót qua đổi preset, reset khi đổi Tác phẩm (AC10). Đường kích hoạt là phím `Mod+Alt+L` (`lookup.lookup_selection`), lấy vùng chọn qua `window.getSelection()` — dep TỐI THIỂU, Story 1.18 đã thay bằng hợp đồng vùng chọn dùng chung. ⚠️ **Món nợ chưa đóng**: hình dạng hiển thị cho mục từ TIẾNG ANH (`deferred-work.md:317`) dùng tạm cấu trúc khối giống tiếng Trung — chủ sở hữu vẫn là Sally (UX), chưa phải chữ ký chính thức.

## Điểm vào focus

`PanelFrame` nhận `owner` qua **prop** (`owner="panel.grid"`) rồi tự `declareFocus(props.owner, …)`. Owner phải có mặt trong `FOCUS_OWNERS` ở `src/commands/index.ts`; cổng đối chiếu hai chiều. Xem `src/commands/README.md`.

🔵 **Ví dụ trên sửa 2026-08-15 (code review).** Nó viết `owner="panel.source"` — một owner **đã bị gỡ** ở Story 2.5b *(`FOCUS_OWNERS` hôm nay: `mode.library` · `mode.workspace` · `mode.reading` · `panel.grid` · `panel.lookup` · `panel.ai_translation`)*. Ba chỗ dùng thật là `GridPanel.vue:741` · `LookupPanel.vue:313` · `AiTranslationPanel.vue:49`. ⚠️ Câu này được viết như một ví dụ **hiện tại**, không mang dấu lịch sử — nên nó dạy sai chứ không chỉ cũ.

## Hợp đồng vùng chọn (Story 1.18) — một module, ba panel, không một listener toàn cục

`src/panels/selectionContract.ts` là bề mặt mà **mọi** panel văn bản đi qua để Auto-Lookup
hoạt động. `epics.md:1762` đòi nó bằng chữ: một hợp đồng dùng chung, và AI Translation +
Editor *"nhận được cùng hành vi khi chúng có nội dung ở các epic sau, **không cần cài
lại"***.

**Cách dùng — đúng một dòng cho mỗi bề mặt:**

```ts
const surface = useTemplateRef<HTMLElement>('surface')
useSelectionSurface(surface, 'source')     // hoặc 'display'
```

⚠️ Vai viết **LITERAL**, không qua biến — **Kiểm F** của `npm run check:commands` đọc TĨNH và
cưỡng chế bằng máy **số lời gọi mong đợi của TỪNG tệp** *(`GridPanel.vue` = **2**, các panel
khác = 1)* và Panel Lookup mang vai `display`.

🔵 **2026-08-15 (Story 2.5b) — sàn 7 → 6, và đó là một lượt ĐẾM LẠI quần thể, không một lượt
nới cổng.** Lưới thay **BA** lời gọi *(`SourcePanel` + `SourceHanViet` + `EditorPanel`)* bằng
**HAI** *(hai cột của `GridPanel.vue`)*. Vế `SourceHanViet` là chỗ dễ đếm sót nhất: AC7 đòi
cột là **một** bề mặt, nên bề mặt Hán Việt **nhượng** lượt đăng ký cho cột và chỉ ghi tên vào
`hanVietSurfaces.ts` — nó **không biến mất**, nó đổi cửa.

🔵 **Bổ sung cùng ngày (code review): con số 6 đúng, nhưng phép đếm dẫn tới nó thì SAI, và
cổng đã xanh với một đơn vị dư trong lúc chờ.** `SourceHanViet.vue` vẫn giữ lời gọi
`useSelectionSurface` **ở mặt chữ** — nó nằm trong nhánh `if (props.surfaceRole === 'own')`, và
Kiểm F là **regex quét tĩnh**, không phân tích `if`. Cổng tự in `7 bề mặt` trong khi sàn là 6.
Đóng bằng cách **gỡ nhánh `'own'` chết** *(chỗ mount duy nhất là `GridPanel.vue:848`, luôn khai
`surface-role="cell"` ⇒ nhánh ấy chưa từng chạy)*, **không** bằng cách nâng sàn lên 7; và prop
`surfaceRole` bỏ luôn giá trị mặc định, để một chỗ mount quên khai nó thành lỗi kiểu thay vì một
bề mặt mất tích.

🔴 **Bài học, ghi ra vì nó rẻ hơn lần sau tự tìm lại:** một con số sàn phải đến từ một lượt
**CHẠY CỔNG**, không từ một phép trừ trên giấy. Phép trừ *"ba thay bằng hai"* đọc rất thuyết
phục — và nó lệch đúng một.

⚠️ **`selectionContract.ts` mang một mệnh đề CỐ Ý không được sửa.** Hai doc-comment trong tệp
đó nói *"một lượt đổi preset dựng lại cả **bốn** panel"*. Từ Story 2.5b con số thật là **ba**.
Câu đính chính sống **ở đây**, không ở tệp kia: AC7 của Story 2.5b đòi `selectionContract.ts`
**không sửa một dòng**, và phép nghiệm thu của nó — *"`git diff` thấy ngay"* — chỉ còn giá trị
chừng nào không ai chạm vào tệp, kể cả để sửa một chú thích đúng. ⇒ Mệnh đề *"bốn panel"* trong
tệp đó đọc là **lịch sử**; số hiện hành ở đây.

🔴 Sàn từng là 4 và đó là một lỗ: `SourceHanViet` không nằm trong các panel Workspace nên phép
kiểm riêng của mỗi panel không canh nó, và bớt đúng lời gọi đó vẫn còn 4 — ĐÚNG sàn cũ, cổng
xanh, mất lưới cho toàn bộ đường bàn phím Hán Việt (lượt review 2026-08-07). Cổng đó tồn tại vì AI Translation và Editor
hôm nay **không có chữ**: một lượt đăng ký thiếu ở đó không để lại triệu chứng nào cho tới
**Epic 2 / Epic 4**.

| Vai | Nghĩa | Ai mang *(🔵 đếm lại 2026-08-15)* |
|---|---|---|
| `'source'` | Bôi đen ở đây **PHÁT** một lượt tra | **cột nguyên văn của `GridPanel.vue`** *(`colSrc`)* — đúng **một** |
| `'display'` | Bề mặt chữ **CỐ Ý không được** là nguồn | cột bản dịch *(`colTgt`)* · Panel Lookup · AI Translation · `ShortcutsOverlay` · `AttributionOverlay` — **năm** |

🔵 **Bảng trên sửa 2026-08-15 (code review) — nó đang liệt kê BA cái tên đã chết.** Bản cũ ghi vai `'source'` cho *"Panel Source (`.original`) · `SourceHanViet` (`.hv-surface`) · AI Translation · Editor"*: `SourcePanel.vue` và `EditorPanel.vue` **đã bị xoá** ở Story 2.5b; `SourceHanViet` **nhượng** lượt đăng ký cho cột nên nó thôi mang vai nào; và AI Translation mang `'display'`, **không** `'source'` — bôi đen trong một bản dịch máy mà phát lượt tra là đúng Bẫy 1 mà Panel Lookup đã bắt. Số thật đọc thẳng từ cổng: `npm run check:commands` Kiểm F in **`6 bề mặt … — 1 nguồn · 5 hiển thị`**.

⚠️ **Một bề mặt `'source'` DUY NHẤT là hình dạng của hôm nay, không phải một bất biến.** Story 1.20/3.4 sẽ thêm bề mặt. Đừng viết một phép kiểm nào dựa vào con số 1.

🔴 **Vì sao OPT-IN, không một danh sách loại trừ:** Panel Lookup tự nó chứa chữ, nên một
listener `document` không lọc nguồn dựng một **vòng tự thay thế** — bôi đen một nghĩa để đọc kỹ
sẽ thay chính đoạn đang đọc. Một danh sách loại trừ phải bảo trì tay qua chín epic; quên một
bề mặt mới là mở lại lỗ đó **im lặng**.

🔴 **Vì sao MỘT listener trên `document`, không một listener mỗi panel:** kéo chọn từ trong
panel rồi thả chuột **ngoài** panel là thao tác thường ngày. Vị từ *"thuộc nguồn nào"* vì vậy
đọc **`anchorNode`**, không `event.target`.

**Ba số đo quyết định hình dạng vị từ** *(2026-08-07, WKWebView + Chromium, khớp nhau)*:
`getSelection().toString()` trong `<input>` trả văn bản THẬT không chuỗi rỗng · `anchorNode` không
bao giờ nằm TRONG một ô nhập *(nó là phần tử CHA)* · `document.activeElement` cho **âm tính
giả**. ⇒ tín hiệu đúng là **`anchorNode.nodeType === TEXT_NODE`**. Xem doc-comment tại chỗ.

⚠️ **Bề mặt cần một cách lấy truy vấn RIÊNG thì truyền `resolve`.** Hôm nay có hai, và **cả
hai đọc DOM trực tiếp** *(một khuôn, một chỗ sửa — Story 1.18b)*: tab Hán Việt kiểu **chuyển
đổi** *(màn hình chỉ có âm Latin ⇒ ánh xạ ngược theo **VỊ TRÍ** về ký tự Hán nguồn — không một
bảng tra âm→chữ, thứ đó đa trị và thuộc FR113/Story 3.7)* và kiểu **song song** *(đọc node văn
bản của `<ruby>`, không tin `Selection.toString()` — WKWebView rò âm Hán Việt qua
`Selection.modify()`; xem `deferred-work.md` §1.18)*.

🔴 **Chú thích cũ ở đây nói *"đọc node `.hv-char`"* — SAI, và đã sai từ trước Story 1.18b.**
Lớp `.hv-char` bị gỡ ở lượt vá `<ruby>` 2026-08-07 *(âm đọc thôi `position: absolute`)*; thứ
`resolveParallel()` đọc là **node văn bản trực tiếp của `<ruby>`**, và mốc cấu trúc nó duyệt
là `.hv-unit`.

## Tách từ tiếng Trung (Story 1.18b) — `Intl.Segmenter`, và **chỉ** cho vùng chọn

`src/panels/wordBoundary.ts` trả **ranh giới TỪ** trong văn bản nguồn, bằng
`Intl.Segmenter('zh', { granularity: 'word' })` — **0** phụ thuộc npm mới, **0** dòng Rust,
**0** điểm ra mạng. Đây là **cùng bộ tách từ ICU** mà trình duyệt dùng cho double-click trên
một khối văn bản thuần, tức trên **tab nguyên văn** — nên hai tab chọn cùng một cụm **theo cấu
trúc**, không nhờ trùng hợp *(đo: bảng đối chiếu 26 vị trí, 26/26 khớp)*.

🔴 **Đây là *"chọn gì để tra"*, KHÔNG phải *"tra thế nào"*.** Ranh giới đã phân xử sẵn ở
`reviews/review-ad-44-2026-08-05.md:50`. Khớp ngôn ngữ vẫn là độc quyền của **AD-17**
*(`jieba-rs`, Rust, Story 1.12)*; story 1.18b **không chạm một dòng** của `core/matching/**`.
`AD-1` cho phép vì `ARCHITECTURE-SPINE.md:75-79` liệt kê **"vùng chọn"** là thứ frontend giữ.

Đơn vị của cả bề mặt vì vậy là **một TỪ**, không còn một ký tự:

| kiểu xem | cấu trúc | ai chặn/không chặn ICU |
|---|---|---|
| **song song** | một `<ruby>` mỗi TỪ, base mang trọn cụm ký tự | `.hv-unit` là `display: **inline**` — `inline-block` làm `Selection.modify('word')` **kẹt hẳn** ở ô đầu (đo 2026-08-07, hồi quy AC11/1.18) |
| **chuyển đổi** | một `.hv-word` mỗi TỪ, mỗi âm một `.hv-syl` | các âm nối bằng `WORD_JOINER` (`U+2060`, rộng 0); khe hở **do CSS vẽ**, vì mọi khoảng trắng THẬT đều cắt từ với ICU |

⚠️ **`U+2060` không được ra clipboard** *(`SourceHanViet.vue::onCopy` đổi nó về một dấu cách)*
— một ký tự vô hình dán ra ngoài là thứ không ai lần ra được.

⚠️ **Engine thiếu `Intl.Segmenter`** ⇒ rơi về **một ký tự một từ** *(hành vi trước 1.18b)* kèm
`console.error` nêu đích danh. Không im lặng, không ném.

---

## Bật/tắt nguồn từ điển (Story 1.19) — bộ lọc ở **Rust**, chip ở webview

`src/panels/dictSourcesState.ts` giữ **danh sách nguồn** và **tập bị tắt**; `LookupPanel.vue`
vẽ dải chip; `src/AttributionOverlay.vue` vẽ bảng ghi công.

**Bốn mệnh đề, và cả bốn đều dễ cài ngược:**

| mệnh đề | vì sao |
|---|---|
| **Danh sách nguồn dẫn xuất từ `list_dict_sources`, KHÔNG từ `groups`** | một nguồn **đang tắt** không sinh nhóm nào ⇒ một dải chip dẫn xuất từ `groups` **không có chip để bật nó lại**, người dùng tự khoá mình ra ngoài |
| **Bộ lọc áp ở RUST, tầng gom, là THAM SỐ từ chỗ gọi** | trần `LIMIT = 20` chạy **trước**, nên lọc ở webview để một nguồn **đang bật** biến mất chỉ vì một nguồn **đã tắt** có `entry_id` nhỏ hơn — và `count_by_source` sẽ đếm cả nguồn đã tắt *(§Quyết định #2a)* |
| **Giá trị lưu là tập BỊ TẮT, không phải tập được bật** | mặc định là *mọi nguồn đều bật*, nên một nguồn **mới** ở bản sau phải tự động bật. Lưu tập được-bật làm nó im lặng **tắt** — đúng lớp lỗi *"rỗng im lặng"* mà AD-44 ④ cấm |
| **Bộ lọc áp cho **cả** tab Hán Việt** | âm Hán Việt mang `source_code` và `sources_used` **hiện tên nguồn lên màn hình**; để nó ngoài bộ lọc là để một nguồn *"đã tắt"* vẫn viết chữ lên tab *(§Quyết định #3a)* |

🔴 **Hệ quả đo được của mệnh đề thứ tư:** `priority_order()` đẩy lớp **nền** xuống cuối, nên
tắt một lớp gỡ rời **ĐỔI ÂM hiển thị** chứ không chỉ giấu bớt — ký tự rơi về âm của lớp nền.
Hành vi **đúng** *(cùng cơ chế FR36 dựa vào khi một lớp bị gỡ)*, và
`dict_sources.rs::disabling_a_detachable_source_changes_the_reading_it_does_not_erase_it`
khẳng định **âm cụ thể**, không chỉ khẳng định `sources_used` sạch.

### TẮT ≠ GỠ

**Tắt** chỉ giấu một nguồn khỏi **kết quả tra cứu** — dữ liệu vẫn nằm trong bản cài, và nguồn
**vẫn được ghi công đầy đủ** trong bảng Attribution *(AC10 — nghĩa vụ CC-BY-SA gắn với việc
**phân phối** dữ liệu, không với việc hiển thị nó)*. **Gỡ** là xoá tệp `.db` khỏi bản phát
hành, việc của **người đóng gói** *(FR112)*. Không nút xoá tệp, không đường ghi vào
`resources/dict/`, không cơ chế tải thêm *(NFR6)*.

### Dải chip **không** nằm trong vùng đầu mục

`.lookup-head` khoá `height: 76px; overflow: hidden`, và Story 1.17/1.18 đã vỡ đúng chỗ này
**hai lần**. Đo trên bố cục hiện tại: đầu mục 24px/1.3 ≈ 31px + `margin-top` 7px + thanh nhịp
≈ 15px + `padding-bottom` ⇒ vùng 76px **đã đầy**. ⇒ dải chip là một hàng **RIÊNG**,
`flex: none`, đứng **trên** vùng đầu mục — đúng thứ tự `mockups/sources-attribution.html`
vẽ. `--lookup-head-height` giữ **nguyên** giá trị và **nguyên** vai trò.

### Trạng thái tắt phân biệt bằng **màu + gạch ngang**, không bằng `opacity`

UX-DR6 cấm làm mờ **chữ** ở trạng thái **nghỉ**, và một chip tắt là một trạng thái nghỉ. Hai
tín hiệu chứ không một: màu một mình không đọc được với người mù màu.

### Ba command tĩnh, **không** một command cho mỗi nguồn

`lookup.toggle_source` *(bật/tắt nguồn đang được nhắm)* · `attribution.open` ·
`attribution.close`. Mockup vẽ `⌥1…6` cho từng nguồn — **bác**: danh sách nguồn **dẫn xuất
lúc chạy** (0 tới 10 nguồn), còn `CommandRegistry` là một danh sách **tĩnh** mà
`check-commands.mjs` đếm bằng máy; `Mod+Alt+1`/`2` đã thuộc preset bố cục; và FR22/Story 1.21
đòi mọi command **gán lại được**, mà một id không tồn tại lúc dựng màn hình phím thì không.

⚠️ Mục tiêu của `lookup.toggle_source` đi bằng `@mousedown` **uỷ quyền ở vùng chứa** cộng
`document.activeElement` — **không** một tham số trên command. WKWebView **không đặt tiêu
điểm cho `<button>` khi bấm chuột**, nên đọc mỗi `activeElement` là để cả đường chuột chết
trên macOS trong khi xanh trên Windows *(NFR14)*.

### Ngắt đoạn của **bản dịch**: một `\n` và một cờ, HAI khái niệm khác nhau

Story 2.5d dựng hai thứ trong cùng một ô, và chúng **rất dễ đọc gộp làm một**:

| Thứ | Là gì | Ở đâu |
|---|---|---|
| `\n` **trong** `target_text` | **xuống dòng bên trong một câu** — người dùng gõ ra bằng `Enter` (AC1) | một ký tự của chuỗi, trên đĩa |
| `segment.is_target_paragraph_end` | **ranh giới đoạn SAU câu** — dữ liệu đã lưu (AC2, AC4) | một cột, bước di trú **9** |

🔴 **Đường mã nào cần cấu trúc đoạn của bản dịch thì ĐỌC CỘT** — không suy từ
`is_paragraph_end` của cột nguyên văn, và **không** đếm `\n`. Cả hai phép suy đều chạy ra kết
quả trông đúng, và cả hai rẽ khỏi đĩa đúng vào ngày người dùng đổi cờ đầu tiên. Rà 2026-08-16:
`split('\n')` · `lines()` trên `src/**` và `src-tauri/src/**` ⇒ **0** đường suy.

⚠️ **`white-space: pre-line` trên `.cell-tgt` KHÔNG phải một dòng trang trí** — nó là **tiền
đề vận hành** của `Enter`. Đo trên WKWebView 605.1.15 thật *(`2-5d-ban-do/`)*: dưới `pre-line`,
`execCommand('insertLineBreak')` dựng một **text node `\n`** *(`textContent === "A\nB"`)*; **không**
`pre-line` thì cùng lệnh đó dựng `<br>` và `textContent` đọc ra **`"AB"`** — mất trắng ranh
giới trên đường ghi. Đổi giá trị đó là lật nhánh ① của `onBeforeInput` trong im lặng.

⚠️ Cờ đích hiển thị bằng **màu đường kẻ đáy** của riêng ô bản dịch, **không** bằng "khoảng
thở" như cờ nguồn — và đó là một phép đo, không một gu: năm cột chia chung một tập track hàng,
nên một `padding-bottom` đặt riêng ở ô bản dịch kéo **cả hàng** cao lên *(đo: 38,00 → 46,00 px,
ô nguyên văn cũng 46)*. Hai cấu trúc đoạn khác nhau **không** biểu diễn được bằng hai khoảng thở.

## Lịch sử phiên bản segment (Story 2.6) — bề mặt ở **App**, state ở đây

Ba tệp của story này, và chỗ đặt từng tệp là một quyết định:

| Tệp | Vai | Vì sao ở đó |
|---|---|---|
| `src/SegmentHistoryOverlay.vue` | lớp phủ | con trực tiếp của `App.vue`, **không** một panel — khuôn `ShortcutsOverlay.vue`/`AttributionOverlay.vue` |
| `segmentHistoryState.ts` | trạng thái + định tuyến | state **cấp module**, sống sót qua một lượt `api.clear()` đổi preset |
| `segmentHistoryTime.ts` | định dạng thời điểm | hàm **thuần**, `now` đi vào qua tham số |

### Vì sao **không** mở lịch sử ngay trong hàng của lưới

🔴 **Một hàng KHÔNG phải một phần tử DOM** (xem đầu `GridPanel.vue`): năm cột là năm `subgrid`
chia chung một tập track, nên chèn một khối "giữa hai hàng" là thứ hình dạng đó **không diễn
đạt được**. Đường tab-trong-panel-Lookup cũng bị loại, bằng số đo: cột thật của bố cục Ⓑ-2 rộng
**238,5 px** còn mockup vẽ một cột danh sách 270 px **cộng** một cột nội dung.

### Hàng đang nhắm — khuôn giữ được HAI luật thoạt nhìn xung khắc

`historyAimedVersionId` + `aimHistoryVersion()` là khuôn `aimedShortcutRow` của Story 1.21:

- **AD-34 §1** *(Kiểm A của `check:commands`)* đòi mọi `@click` là **đúng một**
  `dispatch('<id>')` với id **literal** ⇒ một nút **không thể** mang `@click="restore(row.id)"`.
- **§KHÔNG-LÀM** cấm một command cho mỗi hàng ⇒ không thể sinh id theo `version_id`.

⇒ Hàng được **nhắm** bằng `@mousedown`/`@focusin` *(Kiểm A nói nguyên văn "chỉ `@click`", nên
hai sự kiện đó được xử lý tự do)*, rồi ba command **không tham số** đọc mục tiêu **lúc chạy**.

### Chốt chống mất bản nháp — quy tắc ở **Rust**, nghĩa vụ flush ở **đây**

🔴 Phép so *"bản đang soạn có bản sao trong `segment_version` không"* chạy ở Rust, và nó so
trên **ĐĨA**. `editorEditedText` có thể còn giữ ký tự chưa xuống WAL (AD-35: idle 2 s, trần
cứng 5 s) ⇒ `restoreVersion()` **phải flush trước**, nếu không chốt so với một bản **cũ hơn thứ
người dùng đang nhìn** và nó sẽ **không hỏi** ở đúng ca cần hỏi nhất.

⚠️ Mệnh đề này **không cưỡng chế được ở tầng Rust** — lệnh khôi phục chỉ đọc thứ đã ở trên đĩa
và không biết gì về văn bản đang gõ trong webview. Lưới **duy nhất** là
`tests/frontend/segmentHistory.test.ts` §④, và nó khẳng định **thứ tự trên dây**
(`['flush', 'restore']`), không chỉ *"có gọi flush"*.

⚠️ **Đừng tính lại chốt đó ở TypeScript** dù nó trông dễ *(so `editorEditedText` với danh sách
phiên bản)* — một phép tính thứ hai là một nguồn sự thật thứ hai, và nó sẽ rẽ khỏi Rust ở đúng
ca biên. AD-1.

### `historyTimeLabel` trả một **mô tả**, không một chuỗi

Nó trả `{ key, params }` và nơi gọi ghép bằng `t()`. Ba luật cùng lúc: chuỗi ở lại `vi.json`;
tham số mang **dữ liệu**, không mang **câu**; và hàm kiểm được **tất định** mà không cần một
bảng chuỗi giả.

🔴 Hàm **không tự đọc `Date.now()`** — cùng luật đã có cho `layout/writeSchedule.ts`. Chỗ đọc
đồng hồ là `SegmentHistoryOverlay.vue`, một chỗ duy nhất. ⇒ Test **không** `vi.useFakeTimers()`.

⚠️ Phép so ngày đọc theo **giờ địa phương**, cố ý **không** `toISOString()` — cái sau trả về
theo UTC nên nó rẽ sai ở hai chiều ngược nhau tuỳ dấu offset. **GIỚI HẠN THẬT:** ở UTC đúng
(offset 0) cả hai chiều biến mất và ca canh nó **rỗng nghĩa**; runner CI chạy UTC. Ghi nợ.
