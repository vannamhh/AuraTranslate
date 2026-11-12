---
name: AuraTranslate
status: final
created: 2026-08-02
updated: 2026-11-03
sources:
  - _bmad-output/initiative-auratranslate/spec-auratranslate/spec-auratranslate.md
  - _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md
  - _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md
colors:
  background: '#f4f1ea'
  surface: '#fbfaf6'
  surface-sunken: '#f0ece1'
  surface-accent: '#e6eeee'
  surface-tm: '#faf6ee'
  on-surface: '#2b2723'
  on-surface-variant: '#6b6459'
  outline: '#e2dccf'
  outline-faint: '#efeade'
  ornament: '#a9a196'
  primary: '#2f5d63'
  on-primary: '#fbfaf6'
  confirmed: '#5a6b3f'
  tm-rule: '#a38347'
  caret-rule: '#25484d'
  control-rule: '#8a8276'
  tm-text: '#7a5d25'
  lookup-active: '#a8430c'
  error: '#8f2f22'
colors-dark:
  background: '#201e1b'
  surface: '#26241f'
  surface-sunken: '#1b1a17'
  surface-accent: '#2c3a3b'
  surface-tm: '#302b21'
  on-surface: '#e8e3d8'
  on-surface-variant: '#a29a8c'
  outline: '#3b382f'
  outline-faint: '#302d26'
  ornament: '#6a6459'
  primary: '#7fb3ba'
  on-primary: '#1b1a17'
  confirmed: '#9cb37a'
  tm-rule: '#92763f'
  caret-rule: '#c7dee1'
  control-rule: '#807868'
  tm-text: '#d3b276'
  lookup-active: '#fb923c'
  error: '#e5867a'
typography:
  # 🔵 2026-10-29 (Ice): giãn dòng là bội 7px, trừ source-cjk · source-hanviet · editor của thân lưới Workspace. Xem §Layout & Spacing.
  read-lg:
    fontFamily: read
    fontSize: 19px
    lineHeight: '42px'
    letterSpacing: 0.004em
  read-md:
    fontFamily: read
    fontSize: 17.5px
    lineHeight: '35px'
  read-sm:
    fontFamily: read
    fontSize: 16px
    lineHeight: '28px'
  read-title:
    fontFamily: read
    fontSize: 23px
    fontWeight: '600'
    lineHeight: '35px'
  source-cjk:
    fontFamily: read
    fontSize: 18px
    lineHeight: '2.05'
  source-hanviet:
    fontFamily: read
    fontSize: 16px
    fontStyle: italic
    lineHeight: '1.95'
  editor:
    fontFamily: read
    fontSize: 16px
    lineHeight: '1.95'
  translation-ref:
    fontFamily: read
    fontSize: 15px
    lineHeight: '28px'
  lookup-headword:
    fontFamily: read
    fontSize: 24px
    lineHeight: '35px'
  cover-name:
    fontFamily: read
    fontSize: 16px
    lineHeight: '23px'
  lookup-gloss:
    fontFamily: read
    fontSize: 14.5px
    lineHeight: '21px'
  lookup-example:
    fontFamily: read
    fontSize: 13px
    fontStyle: italic
    lineHeight: '21px'
  ui-md:
    fontFamily: ui
    fontSize: 13px
    lineHeight: '21px'
  ui-sm:
    fontFamily: ui
    fontSize: 13px
    lineHeight: '21px'
  ui-label:
    fontFamily: ui
    fontSize: 13px
    fontWeight: '700'
    lineHeight: '21px'
    letterSpacing: 0.08em
  ui-mono:
    fontFamily: mono
    fontSize: 13px
    lineHeight: '21px'
families:
  read: '"Source Serif 4", "Noto Serif CJK TC", serif'
  read-cjk: '"Noto Serif CJK TC", serif'
  ui: '"Source Sans 3", ui-sans-serif, -apple-system, "Segoe UI", system-ui, sans-serif'
  mono: 'ui-monospace, SFMono-Regular, Consolas, monospace'
fonts-bundled:
  policy: 'Nhúng trong bản cài. Không CDN, không tải sau khi cài (AD-15, FR27). Tải tệp về rồi đóng gói — CẤM fonts.googleapis.com, đó vẫn là origin từ xa.'
  channel: 'Kênh Google cho cả ba họ (Ice chốt 2026-08-03). Latin: google/fonts ofl/sourceserif4 + ofl/sourcesans3, font biến thiên. CJK: notofonts/noto-cjk release Serif2.003 — cùng một font với Source Han Serif 2.003R, khác nhãn.'
  license: 'SIL OFL 1.1 — ĐÃ RÀ 2026-08-03 (Story 1.1), tương thích GPL v3 theo diện gộp gói. Xác minh bằng cách mở tệp LICENSE trong bản release đã tải, không tin nhãn GitHub. Noto Serif CJK và Source Serif 4 (bản Google Fonts) KHÔNG khai Reserved Font Name; Source Sans 3 CÓ khai RFN "Source" nên subset tệp đó phải đổi tên font. Ba tệp giấy phép gốc đi kèm bản phát hành (FR38, FR109). Ba hàng font đã vào bảng Stack của ARCHITECTURE-SPINE.md.'
  weights: 'Latin dùng font biến thiên nên phủ trọn dải nét 200–900 chỉ với 3 tệp — nét 600 và 700 trong bảng token được lo trọn, không cần đóng gói thêm. CJK chỉ Regular, để ghìm dung lượng.'
  size-budget: 'ĐÃ ĐO 2026-08-03 (Story 1.1). Trên đĩa 25,991 MiB cho 4 tệp (23,405 CJK + 2,586 Latin) — ước 21,6 MB của bản trước quá thấp vì phần CJK là 23,4 MiB chứ không phải 19 MB. Chênh lệch .dmg thật: 20,300 MiB = 21,29 MB (1,337 → 21,637 MiB). Tổng với database 130 MB hiện tại = 151,29 MB, DƯỚI trần NFR6. .msi chưa đo được trên macOS (tauri-cli từ chối target msi) — ước 16,0–20,3 MiB, đóng ở Story 1.3. RỦI RO: trần 150–200 MB là trần của CẢ BẢN CÀI ĐÃ GỒM FONT, nên phép tính là trừ dư địa chứ không cộng lên trần — 200 − 21,29 font − 1,40 baseline app RỖNG − 130 ba nguồn đầu = còn ~47 MB cho các nguồn từ điển còn lại, chỉ mục FTS phụ, VÀ toàn bộ mã sản phẩm chưa viết. Đối chiếu lại ở Story 1.9. 🔄 **CẬP NHẬT 2026-08-05 (NFR6 sửa lần hai):** trần nâng lên **400.000.000 byte**; payload thật đo được **343.991.430 byte** với BẢY nguồn (dư 56.008.570). Mọi con số "dư địa ~47 MB" ở dòng này là **bản ghi tại thời điểm 2026-08-03**, không còn là ràng buộc đang sống — xem `prd.md` §7.2.'
  region-variant: 'CHỐT TC 2026-08-03 (Story 1.1) — NotoSerifCJKtc-Regular.otf. Lý do: phạm vi dự án là dịch thuật tổng quát chứ không phải ngách truyện mạng, và hai lớp từ điển Cổ hán văn + HVTĐTD đều là ngữ liệu cổ văn. Khác biệt nặng nhất không phải dáng chữ mà là VỊ TRÍ DẤU CÂU: TC đặt 「，。」 giữa ô chữ, SC đặt góc dưới trái — hiện ở mọi dòng. Chi phí đổi ý bằng 0: hai tệp lệch nhau 1.176 byte. Vẫn bắt buộc bản vùng ĐẦY ĐỦ (NotoSerifCJKtc), KHÔNG dùng bản subset theo ngôn ngữ (NotoSerifTC).'
rounded:
  none: 0
  sm: 2px
  DEFAULT: 3px
  md: 4px
  window: 9px
  full: 9999px
spacing:
  # 🔵 2026-10-29 (Ice): lưới 7px, trước là đơn vị 4px · 16 · 12 · 36 · 40 · 34 · 22.
  unit: 7px
  panel-inline: 14px
  panel-block: 14px
  screen-inline: 21px
  head-height: 35px
  titlebar-height: 42px
  status-height: 35px
  gutter-width: 21px
  read-measure-lg: 62ch
  read-measure-md: 68ch
  read-measure-sm: 76ch
components:
  panel-head: { height: 35px, typography: ui-md, color: on-surface-variant }
  panel-focus-rule: { width: 2px, color: primary }
  segment-gutter-rule: { width: 2px, radius: sm, inset-left: 8px }
  source-chip: { typography: ui-label, color: primary }
  record-rule: { width: 2px, color: outline-faint, padding-left: 12px }
  grid-row-divider: { height: 1px, color: outline-faint }
  grid-para-divider: { height: 1px, color: outline, space-block: 16px }
  grid-num-col: { width: 34px, typography: ui-mono, color: on-surface-variant, align: right }
  grid-state-col: { width: 108px, typography: ui-label, color: on-surface-variant }
  grid-empty-cell: { min-height: 1.95em, border-bottom: 1px dashed outline }
  grid-row-omitted: { color: on-surface-variant, decoration: line-through }
  control: { height-lg: 35px, height-md: 28px, radius: 3px, border: control-rule, field-background: surface-sunken, check-size: 14px }
---

## Brand & Style

AuraTranslate là **bàn viết**, không phải bảng điều khiển. Người dùng ngồi với nó nửa buổi liền cho một Chương, nên phần mềm phải chịu lùi lại phía sau văn bản. Khung viền tan thành đường kẻ mảnh; thanh công cụ chỉ giữ những gì đang thật sự dùng; không có gradient, không có bóng đổ trang trí, không có màu nào chỉ để cho vui mắt. 🔵 2026-10-29 (Ice): bóng đổ được dùng ở nơi cần tạo chiều sâu cho một lớp nổi, giá trị lấy từ token `shadows`; gradient vẫn cấm.

Phong cách là **Editorial trầm** với kỷ luật của một **nhạc cụ nghề nghiệp**. Nó kế thừa sự quen tay của QuickTranslator ở mô hình tra cứu và bố cục panel, nhưng thay ngôn ngữ thị giác tiện ích bằng ngôn ngữ của trang sách. Mọi thay đổi trạng thái nói khẽ: một vạch lề đổi màu, một nhãn nguồn hiện ra — không hộp thoại, không nhấp nháy.

Nguyên tắc nền: **màu là thông tin, không phải trang trí.** Toàn bộ giao diện chỉ có một màu nhấn. Chỗ nào bạn thấy màu, chỗ đó đang nói một điều có thật.

## Colors

Bảng màu lấy từ giấy và mực, không lấy từ màn hình.

- **Giấy ngà (`background #f4f1ea`, `surface #fbfaf6`)** là nền. Không dùng trắng tinh — trắng tinh trên màn hình sáng là bề mặt gây mỏi nhất cho một phiên đọc dài.
- **Mực nâu đen (`on-surface #2b2723`)** là chữ chính. Không dùng đen tuyền, cùng lý do.
- **Xanh mực (`primary #2f5d63`)** là màu nhấn **duy nhất**. Nó chỉ được dùng cho ba việc: thuật ngữ Glossary đã chốt, nhãn nguồn từ điển, và tiêu điểm bàn phím. Không dùng cho nút bấm thông thường, không dùng cho tiêu đề.
- **Xanh ô liu (`confirmed #5a6b3f`)** chỉ nói một điều: câu này đã xác nhận.
- **Nâu vàng (`tm-rule #b99a5e`)** chỉ nói một điều: đây là gợi ý từ Translation Memory, chưa ai xác nhận.

**Nền tối không phải là đảo ngược.** Nền `#26241f` là nâu rất tối, chữ `#e8e3d8` là ngà — không phải đen tuyền và trắng tinh. Tương phản tuyệt đối gây loá sau vài chương, và Chế độ đọc tồn tại chính vì những phiên dài đó.

### Bảng token màu — **17 token mỗi theme** *(🔵 16 → 17, Story 2.5b · 2026-08-15)*

> **Đây là nguồn sự thật của bộ token màu.** *(Bổ sung 2026-08-03: bảng này vốn được nhiều chỗ tham chiếu — kể cả mục "Giãn dòng" ngay dưới và acceptance criteria của story dựng token — nhưng chưa bao giờ được viết ra. Giá trị lấy từ bản dựng đã kiểm tương phản.)*

| # | Token | Sáng | Tối | Vai trò |
|---|---|---|---|---|
| 1 | `background` | `#f4f1ea` | `#201e1b` | Nền ngoài cùng; ở theme tối còn là **khe phân tách panel** |
| 2 | `surface` | `#fbfaf6` | `#26241f` | Mặt phẳng làm việc của panel |
| 3 | `surface-sunken` | `#f0ece1` | `#1b1a17` | Vùng lùi — chiều sâu duy nhất của sản phẩm |
| 4 | `surface-accent` | `#e6eeee` | `#2c3a3b` | Nền nhấn nhẹ |
| 5 | `surface-tm` | `#faf6ee` | `#302b21` | Nền gợi ý Translation Memory |
| 6 | `on-surface` | `#2b2723` | `#e8e3d8` | Chữ chính |
| 7 | `on-surface-variant` | `#6b6459` | `#a29a8c` | Chữ phụ — **sàn thấp nhất cho mọi chữ** |
| 8 | `outline` | `#e2dccf` | `#3b382f` | Nét phân tách |
| 9 | `outline-faint` | `#efeade` | `#302d26` | Nét phân tách mờ |
| 10 | `ornament` | `#a9a196` | `#6a6459` | **Màu của nét, không bao giờ là màu của chữ** |
| 11 | `primary` | `#2f5d63` | `#7fb3ba` | Màu nhấn **duy nhất** — thuật ngữ đã chốt, nhãn nguồn, tiêu điểm |
| 12 | `on-primary` | `#fbfaf6` | `#1b1a17` | Chữ trên nền `primary` |
| 13 | `confirmed` | `#5a6b3f` | `#9cb37a` | Câu đã xác nhận |
| 14 | `tm-rule` | `#a38347` | `#92763f` | Vạch gợi ý TM — **là vạch, không phải chữ**. 🔵 2026-10-25: từ `#b99a5e` cả hai theme |
| 15 | `tm-text` | `#7a5d25` | `#d3b276` | Chữ trong khối gợi ý TM |
| 16 | `error` | `#8f2f22` | `#e5867a` | Lỗi |
| 17 | `draft` | `#b9b3aa` | `#575249` | Vạch *đã dịch tay, chưa ai ký* — **là vạch, không phải chữ**. 🔵 Story 2.5b. 🔵 2026-10-25: không còn mượn giá trị `ornament` |
| 18 | `caret-rule` | `#25484d` | `#c7dee1` | Vạch lề câu đang sửa — **là vạch, không phải chữ**. 🔵 2026-10-25 |
| 19 | `control-rule` | `#8a8276` | `#807868` | Viền nút phụ, nút phân đoạn — **là nét, không phải chữ**. 🔵 2026-10-26 (Ice duyệt): 3,63:1 trên `surface`, 3,21:1 trên `surface-sunken` (sáng); 3,55:1 và 3,98:1 (tối). Đạt 3:1 của WCAG 1.4.11; `outline` chỉ 1,31:1 và `ornament` 2,44:1 |
| 20 | `lookup-active` | `#a8430c` | `#fb923c` | Chữ cụm đang tra ở cột nguyên văn (Auto-Lookup bằng một cú bấm) |

Sống ở `src/tokens/`. **Cấm giá trị màu viết thẳng trong component** (AD-34) — thứ cần kiểm tra tập trung thì không được rải rác.

> **Vì sao từng là 16, và vì sao nay là 17.** Một số bản nháp ghi *"17 token"* **vì một lý do sai** — `tm-rule` giữ **cùng một giá trị ở cả hai theme** (nó là vạch, không chịu ràng buộc tương phản chữ) nên nó dễ bị đếm thành hai. Cách đếm đó vẫn sai, và mệnh đề *"đừng thêm một token thứ 17 **để cho khớp một con số cũ**"* vẫn đúng từng chữ.
>
> 🔵 **2026-08-15 (Story 2.5b) — con số đổi vì một lý do ĐO ĐƯỢC, không vì một con số cũ.** UX-DR19 viết lại cùng lượt correct-course cấp cho *"đã dịch tay, chưa ai ký"* một giá trị vạch riêng, và `check-commands.mjs` Kiểm I đối chiếu **hai chiều** giữa `SEGMENT_RULE_VALUES` và các khối `.rule-<giá trị>` trong CSS — nó đòi đúng `var(--color-draft)`. Một `var(--color-ornament)` ở khối đó làm cổng **ĐỎ**, và một bảng alias để cho lọt là đúng thứ §Miễn trừ cấm bằng chữ.
>
> ⚠️ **`draft` MƯỢN ĐÚNG GIÁ TRỊ của `ornament` ở cả hai theme** *(Ice ký 2026-08-15)*, nên nó là **một cái tên mới cho một màu ĐÃ kiểm**, không một màu mới chưa ai đo — **0 cặp mới** cho `contrast.pairs`, và bảng cặp giữ nguyên **31** cặp mỗi theme. 🔴 Trùng giá trị **không** phải trùng nghĩa: `ornament` nói *đã về hưu*, `draft` nói *đã dịch tay, chưa ký*; hai vạch không bao giờ cùng xuất hiện trên một câu.

### Tông bìa Tác phẩm — **13 token mỗi theme** *(🔵 thêm 2026-10-25, Ice duyệt)*

🔵 2026-10-25: tổng token màu mỗi theme thành 24 + 12 = 36 (số thật của `tokens.json`: đã có `diff-*` và `word-*` ngoài bảng 17); bảng 17 ở trên giữ nguyên. 🔵 2026-10-25, lượt hai: thêm `on-cover` thành 37. 🔵 2026-10-25, lượt ba: thêm `caret-rule` thành 38. 🔵 2026-10-26: thêm `control-rule` thành 39. Không cặp mới cho `contrast.pairs`: bảng đó chỉ kiểm chữ, và Ice chọn không thêm ngưỡng 3:1 cho nét (2026-10-26, plan 12.26); số đo ở hàng 19 là bằng chứng duy nhất. `scripts/check-tokens.mjs` chép bảng này vào bảng đóng băng của nó cùng lượt dựng.

Bìa thẻ Tác phẩm ở Library là một màu đặc `cover-N-from` (Ice, 2026-10-25: không dải chuyển, giữ luật không gradient của Kiểm F); `cover-N-to` có sẵn nhưng chưa có chỗ dùng. Mỗi Tác phẩm một tông cố định, chọn theo `work_id`, nên đổi tên không đổi màu. Tông 1–3 lấy từ `mockups/library-and-import.html`; 4–6 thêm cho màn có nhiều Tác phẩm. Hình mẫu: `mockups/library-luoi-rong.html`.

🔵 2026-10-25, lượt hai (Ice duyệt): **tên Tác phẩm in trên bìa**, vì app chưa có đường đặt ảnh bìa (FR3) và một khối màu trống không cho biết gì. Chữ nội dung (serif) 16 px, giãn dòng 1,35, cắt ở 3 dòng, góc trên trái; tên đầy đủ ở `aria-label`. Phần dưới thẻ không lặp lại tên. Bìa cao 136 px. 🔵 2026-10-25, lúc dựng (Ice chọn): giãn dòng 1,66 bằng token `read-sm` — tên xuống tới 3 dòng nên chịu §Giãn dòng 1.66 là sàn cứng, mà 1,35 nằm dưới sàn; 3 dòng cao 79,7 px, cộng nút đáy vẫn vừa lòng bìa 112 px. Tên đầy đủ nằm nguyên trong DOM (chỉ CSS cắt) và ở `title`. 🔵 2026-11-01 (Ice): giãn dòng 23px (≈ 1.45) bằng token `cover-name`, ngoại lệ có tên của sàn 1.6 và của bội 7.

Chữ trên bìa dùng token `on-cover` = `#fbfaf6` ở **cả hai theme**. Đây là **6 cặp mới** cho `contrast.pairs` mỗi theme (`on-cover` trên mỗi `cover-N-from`), thấp nhất 7,05:1 ở nền sáng và 6,09:1 ở nền tối, cả hai là tông 4 rêu. Nút `Tiếp tục dịch` và nút `⋯` trên bìa vẫn có nền đặc riêng (`primary` hoặc `surface`), không thêm cặp. Nếu sau này có ảnh bìa thì bìa in tên là biểu diễn thay thế cho Tác phẩm không có ảnh.

| # | Tông | Sáng `from` → `to` | Tối `from` → `to` |
|---|---|---|---|
| 1 | đá lam | `#3d4f52` → `#26312f` | `#46595c` → `#2f3b3a` |
| 2 | gỗ | `#5c4a3a` → `#3a2f26` | `#66533f` → `#42362b` |
| 3 | mực tím | `#4a4658` → `#2e2c3a` | `#534e63` → `#363343` |
| 4 | rêu | `#4e5a3c` → `#2f3725` | `#57643f` → `#37402b` |
| 5 | gạch trầm | `#6b4339` → `#3f2823` | `#74493e` → `#48302a` |
| 6 | lam đêm | `#3a4c64` → `#232f3f` | `#42566f` → `#2a3747` |

Nền tối dùng cùng sắc nhưng sáng hơn một bậc, để bìa tách khỏi `surface` `#26241f`.

### Sàn tương phản — đã kiểm, đừng hạ

Ba màu trong bản dựng đầu **không đạt WCAG AA** và đã được sửa ở bảng token. Ghi lại để không ai khôi phục chúng:

| Đã bỏ | Vấn đề | Thay bằng |
|---|---|---|
| `#7d766c` làm chữ phụ | 4,09:1 trên nền giấy — dưới ngưỡng 4,5:1 cho chữ thường | `on-surface-variant #6b6459` (5,2:1) |
| `#a9a196` làm chữ | 2,5:1 — trượt cả ngưỡng chữ lớn | Đổi vai: nay là `ornament`, **chỉ dùng cho nét không phải chữ** |
| `#b99a5e` làm chữ | 2,5:1 | `tm-text #7a5d25` (5,2:1); `tm-rule` giữ nguyên vì là vạch, không phải chữ |

**Quy tắc rút ra:** `ornament` và `tm-rule` là **màu của nét**, không bao giờ là màu của chữ. Mọi chữ, kể cả nhãn 10px, tối thiểu phải là `on-surface-variant`.

Ngoại lệ duy nhất đã đặc tả: ký tự ranh giới câu `⏐` dùng `ornament` — nó là **nét vẽ bằng ký tự**, không mang chữ nghĩa, và ẩn hoàn toàn cho tới khi con trỏ chạm.

### Opacity không được dùng để làm mờ chữ

Luật màu ở trên chỉ chặn được **token sai**. Nó không chặn được `opacity` — và `opacity` kéo tuột mọi thứ xuống dưới AA **bất kể token có đạt hay không**. Một hàng `opacity: 0.4` mang chữ `on-surface-variant` (5,2:1) ra màn hình ở khoảng **2,3:1**; kiểm token vẫn xanh, mắt vẫn không đọc được.

Phát hiện khi kiểm toán lại bảng chờ Glossary ngày 2026-08-03: hàng đã duyệt và hàng đã bỏ được lùi lại bằng `opacity`, và cả hai đều trượt AA.

| Muốn gì | Làm thế nào |
|---|---|
| Lùi một khối chữ ra sau | **Đổi màu chữ** sang `on-surface-variant`. Không dùng `opacity` |
| Báo trạng thái đã xử lý | Vạch lề đổi màu + dấu `✓` / `✕`, giữ nguyên độ đọc được của chữ |
| Ẩn rồi hiện một affordance | `opacity` được phép — nhưng chỉ giữa **0 và 1**, không dừng ở mức trung gian làm chữ mờ thường trực |
| Hiệu ứng nội dung mới vào | `opacity` 0.4 → 1 trong 90ms được phép: đó là **quá độ**, không phải trạng thái nghỉ |

**Luật:** `opacity` ở trạng thái nghỉ chỉ áp cho **nét và nền**, không áp cho chữ. Chữ mờ đi bằng cách đổi token, không bằng cách giảm độ đục.

## Typography

Hai họ chữ, phân vai tuyệt đối.

- **`read` — chữ có chân.** Dùng cho mọi thứ là *nội dung*: nguyên văn, Hán Việt, bản dịch trong Editor, mục từ và nghĩa trong Panel Lookup, toàn bộ Chế độ đọc.
- **`ui` — chữ không chân.** Dùng cho mọi thứ là *bộ máy*: tiêu đề panel, nhãn trạng thái, thanh trạng thái, phím tắt.

Ranh giới này không được nhoè. Nếu một chuỗi nói về tác phẩm thì nó dùng `read`; nếu nó nói về phần mềm thì nó dùng `ui`.

### Một chương trình chữ, không phải ba font ghép

Ba hệ chữ xuất hiện cùng lúc trên một màn hình: **Hán** ở panel Nguyên văn, **Việt có dấu** ở Bản dịch, **Latin** ở nhãn giao diện. Nếu chúng lệch nhau về chiều cao chữ thường và độ đậm nét, mắt phải tự hiệu chỉnh mỗi lần nhảy panel — loại mỏi tích luỹ mà người dùng không chỉ ra được nguyên nhân, chỉ thấy "dùng lâu thấy nặng đầu".

Vì vậy cả ba đến từ **một chương trình thiết kế duy nhất**: `Source Serif 4` · `Noto Serif CJK` · `Source Sans 3`. Chúng cân nhau **theo thiết kế**, không phải do ghép may mắn.

> **Về cái tên `Noto Serif CJK`** *(sửa 2026-08-03)*. Bản trước ghi `Source Han Serif`. **Đó là cùng một font** — dự án chung Adobe + Google công bố 2017, phát hành song song dưới hai nhãn; hai release trùng ngày và trùng khít dung lượng từng asset. Đổi sang nhãn Noto không đổi một nét chữ nào, và đổi vì hai lý do có thật: bản Noto **không khai Reserved Font Name** nên về sau subset để ghìm dung lượng sẽ không kéo theo việc đổi tên font và sửa lan man vào bảng token; và cùng kênh đó có sẵn `Source Serif 4` với `Source Sans 3` ở dạng **font biến thiên**, nhẹ hơn hẳn.

**Font được nhúng trong bản cài**, không dùng font hệ điều hành. Lý do không phải thẩm mỹ: panel Nguyên văn là nơi mắt ở lâu nhất trong sản phẩm, và dùng font hệ thống nghĩa là macOS ra PingFang còn Windows ra YaHei — **hai sản phẩm khác nhau**, phá `NFR14` ngay ở chỗ dễ thấy nhất.

**Lấy tệp từ kênh Google không có nghĩa là gọi Google lúc chạy.** `fonts.googleapis.com` vẫn bị `AD-15` cấm tuyệt đối như mọi origin từ xa. Kênh phát hành là chỗ **tải tệp về lúc dựng**; luật đóng gói không đổi.

Ba việc chưa xong ở bản trước, **nay xong cả ba** *(Story 1.1, 2026-08-03 — số đo và bằng chứng ở [`research/font-spike-results-2026-08-03.md`](../tickets/epic-nen-mong-ung-dung-va-tra-cuu-ngoai-tuyen-tuc-thi/research-font-spike-results/research-font-spike-results.md))*:

1. ~~**Rà giấy phép tường minh** theo `NFR15`~~ — ✅ **đã rà 2026-08-03.** Cả ba là **SIL OFL 1.1**, tương thích `GPL v3` theo diện **gộp gói** (font nằm cạnh mã, không liên kết vào mã). Một khác biệt phải nhớ: `Noto Serif CJK` và `Source Serif 4` bản Google Fonts **không khai** Reserved Font Name, nhưng `Source Sans 3` **có khai** RFN `'Source'` — nên nếu subset riêng tệp Sans thì phải đổi tên font. Kết luận ghi vào bảng Stack của `ARCHITECTURE-SPINE.md` ở Story 1.1. *(Xác minh lại bằng chính tệp `LICENSE` trong bản release đã tải, không tin nhãn GitHub — cả ba khớp dự đoán.)*
2. ~~**Đo dung lượng thật.**~~ — ✅ **đã đo 2026-08-03.** Trên đĩa **25,991 MiB** cho bốn tệp. Ước 21,6 MB của bản trước **quá thấp**: phần CJK là **23,405 MiB** chứ không phải ≈19 MB, vì phép chia zip 7 nét cho 7 giả định các nét bằng nhau mà chúng không bằng. Chênh lệch `.dmg` thật: **20,300 MiB = 21,29 MB** (1,337 → 21,637 MiB). Tổng với database 130 MB hiện tại = **151,29 MB — dưới trần `NFR6`**. `.msi` chưa đo được trên macOS; ước 16,0–20,3 MiB, đóng ở Story 1.3. **Rủi ro còn mở:** trần 150–200 MB là trần của **cả bản cài, đã bao gồm font**, nên phép tính đúng là **trừ dư địa** chứ không cộng lên trần — 200 − 21,29 (font) − 1,40 (baseline app **rỗng**) − 130 (ba nguồn đầu) = **còn ~47 MB** cho các nguồn từ điển còn lại, chỉ mục FTS phụ, **và toàn bộ mã sản phẩm chưa viết**. Đây là thay đổi ở tầng PRD, không phải tầng thiết kế, và đã ghi vào mục *Cần Ice quyết* của báo cáo.
3. ~~**Chọn biến thể vùng** cho Han~~ — ✅ **chốt `Noto Serif CJK TC` 2026-08-03.** Coverage như nhau, đúng như đã nói. Nhưng khác biệt **nặng nhất lại không phải dáng chữ**: TC đặt 「，」 và 「。」 **giữa ô chữ**, SC đặt chúng ở **góc dưới bên trái** — thứ này hiện ở **mọi dòng**, không chỉ ở vài mã hiếm, và Panel Nguyên văn là nơi mắt ở lâu nhất. Chọn TC vì phạm vi dự án là dịch thuật **tổng quát** (Ice bác giả định ngách truyện mạng từ giai đoạn brief) và vì hai lớp từ điển của chính sản phẩm — Cổ hán văn, HVTĐTD — đều là ngữ liệu cổ văn. **Đổi ý không tốn gì:** hai tệp lệch nhau 1.176 byte.

> ⚠️ **Một cái bẫy đặt tên phải viết ra, vì nó hỏng im lặng.** Google phát hành **hai loại tệp khác nhau** với tên gần giống: `NotoSerifCJKsc` / `NotoSerifCJKtc` là **biến thể vùng đầy đủ** — phủ trọn kho mã CJK, chỉ khác dáng chữ ưu tiên; còn `NotoSerifSC` / `NotoSerifTC` là **subset theo ngôn ngữ**, chỉ phủ bộ mã một vùng. Mục 3 ở trên nói tới loại thứ nhất. Lấy nhầm loại thứ hai thì phần lớn chữ vẫn hiện bình thường, chỉ **ô vuông rỗng** ở những mã thuộc hệ chữ kia — mà người dùng mục tiêu dịch **cả** truyện mạng giản thể **lẫn** cổ văn phồn thể. *(Bẫy này còn nhân đôi khi đối chiếu chéo với nhãn Adobe: `NotoSerifTC` tương đương `SourceHanSerifTW`, **không** tương đương `SourceHanSerifTC`.)*

### Bảng token typography — **14 token, bốn họ chữ**

> **Nguồn sự thật của bộ token chữ.** *(Bổ sung 2026-08-03 cùng lý do với bảng token màu.)*

Bốn họ chữ: **`read`** (`Source Serif 4`, biến thiên) · **`read-cjk`** (`Noto Serif CJK TC`, chỉ Regular) · **`ui`** (`Source Sans 3`, biến thiên) · **`mono`**.

> **Nét 600 và 700 trong bảng dưới không cần tệp riêng** — cả hai nằm trong dải 200–900 của font biến thiên nên là nét **thật**, không phải tổng hợp giả. **Bằng chứng dựng thật chỉ phủ một nửa:** ảnh chụp Story 1.1 dựng `Source Serif 4` ở 200/400/600/700 + Italic thành năm dòng riêng, thấy rõ nét 600 của `read-title` là nét thật. Nhưng `Source Sans 3` chỉ được dựng **một dòng ở một nét**, nên **`ui-label` (700, họ `ui`) chưa từng được kiểm** — và nó là token nằm trên đúng tệp có mặc định trục `wght = 200` và `name ID 1 = Source Sans 3 ExtraLight`. **Story 1.4 phải dựng `Source Sans 3` ở 400/600/700 rồi mới coi mệnh đề này là đã kiểm.**
>
> ⚠️ **Chữ Hán nghiêng giả CÓ phát sinh, không phải chưa.** Bản đầu của ghi chú này soát theo tiêu chí *"không token nào vừa là `read-cjk` vừa đòi nét đậm hay nghiêng"* — sai tiêu chí. Điều kiện sinh lỗi không phải *token khai họ nào*, mà là **ký tự CJK được dựng dưới một token nghiêng hoặc đậm bất kỳ**. Mà `families.read` có `Noto Serif CJK TC` trong chuỗi dự phòng, nên chữ Hán rơi vào nó qua fallback — và tệp ấy chỉ có Regular. Hai token đang dính: **`source-hanviet` (6)** và **`lookup-example` (10)**, cả hai `italic`, cả hai họ `read`. Token 10 là *"Ví dụ và trích dẫn"* của Panel Lookup: với từ điển Trung–Việt thì ví dụ **chắc chắn** có chữ Hán, ở cỡ 12,5px — cỡ mà nghiêng giả xấu nhất. Phát hiện khi rà soát 2026-08-03. **Hướng xử lý thuộc Story 1.4**: hoặc chấp nhận nghiêng giả cho phần Hán, hoặc khai `font-style: normal` cho ký tự CJK trong hai token đó. Thêm một tệp nghiêng CJK **không** phải phương án — đó là thêm ~23 MiB, một phần ba ngân sách font hiện tại.

| # | Token | Cỡ / giãn dòng / khác | Họ | Dùng ở |
|---|---|---|---|---|
| 1 | `read-lg` | 19px / 42px / `0.004em` | `read` | Chế độ đọc — mức **Thoáng** (62ch). 🔵 2026-10-29 (Ice): giãn 42px, từ 1.95 |
| 2 | `read-md` | 17.5px / 35px | `read` | Chế độ đọc — mức **Cân** (68ch, mặc định). 🔵 2026-10-29 (Ice): giãn 35px, từ 1.8 |
| 3 | `read-sm` | 16px / 28px | `read` | Chế độ đọc — mức **Đặc** (76ch). 🔵 2026-10-29 (Ice): giãn 28px, từ 1.66 |
| 4 | `read-title` | 23px / 600 / 35px | `read` | Tiêu đề Chương. 🔵 2026-10-29 (Ice): giãn 35px, từ 1.3 |
| 5 | `source-cjk` | 18px / 2.05 | `read-cjk` | Nguyên văn tiếng Trung ở Panel Source. 🔵 2026-10-29 (Ice): ngoài lưới 7px (thân lưới Workspace). 🔵 2026-10-30 (Ice): từ 16.5px, cả nguyên văn tab Trung lẫn chữ Hán ở kiểu Song song |
| 6 | `source-hanviet` | 16px / italic / 1.95 | `read` | Âm Hán Việt. 🔵 2026-10-29 (Ice): ngoài lưới 7px (thân lưới Workspace). 🔵 2026-10-30 (Ice): từ 12.5px (mã đã chạy 14.5px qua sổ deviation từ 2026-08-07), cả kiểu Chuyển đổi lẫn chú âm Song song |
| 7 | `editor` | 16px / 1.95 | `read` | Bản dịch trong Editor. 🔵 2026-10-29 (Ice): ngoài lưới 7px (thân lưới Workspace). 🔵 2026-10-30 (Ice): từ 15px |
| 8 | `lookup-headword` | 24px / 35px | `read` | Đầu mục Panel Lookup. 🔵 2026-10-29 (Ice): giãn 35px, từ 1.3 |
| 9 | `lookup-gloss` | 14.5px / 21px | `read` | Nghĩa. 🔵 2026-10-29 (Ice): giãn 28px, từ 1.6 (mã chạy 1.66, sổ deviation của `tokens.json`). 🔵 2026-10-30 (Ice): giãn 21px (≈ 1.45), ngoại lệ có tên của sàn 1.6 |
| 10 | `lookup-example` | 13px / italic / 21px | `read` | Ví dụ và trích dẫn. 🔵 2026-10-28 (Ice): từ 12.5px. 🔵 2026-10-29 (Ice): giãn 21px, từ 1.6 (mã chạy 1.66, sổ deviation của `tokens.json`) |
| 11 | `ui-md` | 13px / 21px | `ui` | Tiêu đề panel. 🔵 2026-10-29 (Ice): giãn 21px, từ 1.5 |
| 12 | `ui-sm` | 13px / 21px | `ui` | Nhãn phụ. 🔵 2026-10-28 (Ice): từ 12px. 🔵 2026-10-29 (Ice): giãn 21px, từ 1.5 |
| 13 | `ui-label` | 13px / 700 / 21px / `0.08em` | `ui` | Nhãn nguồn từ điển, số thứ tự câu. 🔵 2026-10-26 (Ice): từ 11px / `0.1em`. 🔵 2026-10-28 (Ice): từ 12px. 🔵 2026-10-29 (Ice): giãn 21px, từ 1.4 |
| 14 | `ui-mono` | 13px / 21px | `mono` | Phím tắt, số liệu. 🔵 2026-10-26 (Ice): từ 11.5px. 🔵 2026-10-28 (Ice): từ 12px. 🔵 2026-10-29 (Ice): giãn 21px, từ 1.4 |
| 15 | `translation-ref` | 15px / 28px | `read` | Bản dịch tham khảo: popup bộ nhớ dịch, gợi ý AI. 🔵 2026-10-30 (Ice): token mới, một bậc dưới `editor` |
| 16 | `source-cjk-strong` | 18px / 700 / 2.05 | `read-cjk` | Chữ Hán ở kiểu Song song của tab Hán Việt. 🔵 2026-10-30 (Ice): token mới, nét 700 thật từ `NotoSerifCJKtc-Bold.otf`; ngoài lưới 7px (thân lưới Workspace). 🔵 2026-11-04 (Ice): token gỡ cùng tệp Bold. Kiểu Song song dùng `source-cjk` (400) ở `line-height: normal`; âm đọc cách chữ Hán 8px và dòng chữ Hán kế cách âm đọc 16px trên mọi dòng, bằng `padding-block` của `<rt>` qua `--space-grid-hv-reading-*` (WebKit 1px/9px, WebView2 2px/14px), đo bằng điểm ảnh mực |
| 17 | `cover-name` | 16px / 23px | `read` | Tên Tác phẩm trên bìa thẻ Library. 🔵 2026-11-01 (Ice): tách khỏi `read-sm`, giãn 23px (≈ 1.45), ngoại lệ có tên của sàn 1.6 và của bội 7 |

🔵 2026-10-26 (Ice): **mọi chữ bấm được** — nút, tab con, nút phân đoạn — dùng `ui-md` 13px, bằng cỡ giao diện hệ thống macOS. `ui-sm` 12px chỉ còn cho số đếm và dòng phụ không bấm được. Không chữ giao diện nào dưới 12px. 🔵 2026-10-28 (Ice): chữ nhỏ nhất toàn app là 13px; `ui-sm`, `ui-sm-italic`, `ui-label`, `ui-mono` và `lookup-example` cùng 13px, nên chữ phụ bằng cỡ `ui-md` và tách khỏi chữ thường bằng màu, độ đậm, chữ hoa.

### Bảng token khoảng cách và hình dạng

Đơn vị **7px**. 🔵 2026-10-29 (Ice): từ 4px; luật ở §Lưới 7px.

| Nhóm | Token |
|---|---|
| **Spacing** | `panel-inline 14px` · `panel-block 14px` · `screen-inline 21px` · `head-height 35px` · `titlebar-height 42px` · `status-height 35px` · `gutter-width 21px`. 🔵 2026-10-29 (Ice): từ 16 · 12 · 36 · 40 · 34 · 22 |
| **Thước đọc** | `read-measure-lg 62ch` · `read-measure-md 68ch` · `read-measure-sm 76ch` |
| **Bo góc** | `none 0` · `sm 2px` · **mặc định `3px`** · `md 4px` · `window 9px` · `full 9999px` |

> 🔴 **TẦNG VỎ GIAO DIỆN NÂNG MỘT BẬC — Ice chốt 2026-08-12, và nó LẬT một quyết định của chính tài liệu này.**
>
> 🔵 2026-10-28 (Ice): bảng trên nay đọc **13 / 13 / 13 / 13px**, chữ nhỏ nhất toàn app là 13px. Bảng trên từng đọc **13 / 12 / 12 / 12px** (🔵 2026-10-26: `ui-label` và `ui-mono` nâng thêm một bậc); từ 2026-08-12 tới 2026-10-26 là **13 / 12 / 11 / 11,5px**; trước 2026-08-12 là **12 / 11,5 / 10 / 10,5px**. Ba
> thanh nâng theo để giữ tỉ lệ khoảng thở: `head` 34→**36**, `titlebar` 38→**40**, `status` 32→**34**.
>
> **Lý do, và nó là một phép đối chiếu chứ không một khẩu vị:** giao diện hệ thống macOS chạy ở
> **13px**. Tầng vỏ cũ ở 10–12px, tức **dưới mặc định của hệ điều hành** trên chính nền tảng Ice
> dùng hằng ngày. Ice đọc bằng mắt sau một lượt nghiệm thu A4 và chốt *"11,5px là quá nhỏ để đọc"*.
>
> ⚠️ **Câu ngay dưới đây — *"ghìm chặt … mật độ của một nhạc cụ nghề nghiệp"* — vẫn đứng, nhưng nó
> đã bị cân lại một lần và thua.** Mật độ là một giá trị thật của sản phẩm này; nó không thắng được
> việc chữ khó đọc với chính người dùng duy nhất. Ghi ra để lượt sau biết mệnh đề đó **đã được xét**,
> thay vì đọc bảng mới rồi tưởng ai đó quên mất lý lẽ cũ.
>
> 📌 Câu hỏi để ngỏ: một **hệ số scale giao diện** do người dùng chỉnh là câu trả lời đúng bản chất
> hơn — *"nhỏ quá"* là thuộc tính của từng người, không của một con số. Nó cần token chuyển từ `px`
> cứng sang đơn vị tương đối trước, nên nó là một story riêng, không một lượt sửa bảng.

### Giãn dòng 1.66 là sàn cứng — của chữ nội dung

> 🔵 2026-10-29 (Ice): giãn dòng nay là một bội 7px, không còn là tỉ số (§Lưới 7px). Chữ nội dung ngoài thân lưới Workspace lấy bội 7 nhỏ nhất không chật hơn giãn dòng cũ của nó, trừ `lookup-example` (21px, xem dưới). Chữ giao diện, một dòng lẫn xuống dòng, giãn 21px: ở 13px là 1.615. 🔵 2026-10-29 (Ice): **sàn của chữ xuống dòng hạ từ 1.66 xuống 1.6**, một sàn chung cho cả họ `read` lẫn `ui`; Ice chọn có biết con số, để `ui-md-wrap` và `lookup-example` cùng 21px ở 13px (chật hơn trước 0,58px mỗi dòng). Bốn token thân lưới (`source-cjk`, `source-latin`, `source-hanviet`, `editor`) giữ tỉ số cũ. 🔵 2026-10-30 (Ice): `source-cjk-strong` (chữ Hán đậm ở kiểu Song song) là token thân lưới thứ năm, giữ tỉ số của `source-cjk`. Phần dưới đây là lý lẽ của các sàn, vẫn dùng khi kiểm bằng mắt.

**Dấu tiếng Việt chồng cả trên lẫn dưới** (`ế` `ộ` `ữ` `ẳ` `ườ`). Giãn dòng 1.5 — mức mặc định quen thuộc và ổn thoả với chữ Latin — làm dấu `ườ` ở dòng trên chạm dấu `ộ` ở dòng dưới.

Không token họ `read` nào được xuống dưới **1.66**. Đây là ràng buộc của ngôn ngữ, không phải khẩu vị thiết kế, và nó vô hình nếu chỉ thử bằng chữ Latin. Mọi lần kiểm bằng mắt phải dùng chuỗi dày dấu.

**Họ `ui` được phép ở 1.4 và 1.5** *(quyết định 2026-08-03)*. Ranh giới không phải kích cỡ chữ mà là **chữ có chạy thành đoạn hay không**:

| Loại chuỗi | Sàn | Vì sao |
|---|---|---|
| Chữ nội dung — họ `read` | **1.66** | Chạy thành đoạn dài, xuống dòng liên tục. Đây là chỗ dấu chồng dấu gây mỏi tích luỹ |
| Nhãn giao diện một dòng — `ui-md` `ui-sm` `ui-label` `ui-mono` | **1.4** | Không xuống dòng thì không có dòng dưới để chạm. Ghìm chặt giúp thanh panel 36px và thanh trạng thái 34px giữ được mật độ của một nhạc cụ nghề nghiệp |
| Nhãn giao diện **có khả năng xuống dòng** — mô tả dưới ô thiết lập, câu trạng thái, hộp giải thích | **1.66** | Đã xuống dòng thì áp đúng ràng buộc của chữ nội dung, bất kể dùng họ `ui` |

**Phép thử khi không chắc:** chuỗi này có bao giờ dài quá một dòng không? Có thì 1.66. Không thì được ghìm xuống 1.4.

> **Vì sao phải viết ra.** Bản đầu phát biểu luật là *"không token văn bản nào dưới 1.66"* trong khi bảng token ngay phía trên đặt `ui-md` ở 1.5 và `ui-label` ở 1.4 — tài liệu tự mâu thuẫn với chính nó, và người dựng buộc phải đoán bên nào đúng. Phát hiện khi rà soát 2026-08-03.

Ba mức đọc: `read-lg` **Thoáng** (62ch) · `read-md` **Cân** (68ch, **mặc định**) · `read-sm` **Đặc** (76ch).

## Layout & Spacing

Đơn vị 7px (🔵 2026-10-29 (Ice): từ 4px, §Lưới 7px). Khoảng cách chặt ở phần bộ máy, rộng ở phần nội dung — panel chỉ cao 35px cho thanh tiêu đề, nhưng văn bản bên trong thở thoải mái.

Workspace là lưới **2×2** mặc định: hàng trên `Nguyên văn | Bản dịch`, hàng dưới `Tra cứu | Đề xuất AI`. Bố cục 4 cột là preset thay thế. Ranh giới giữa panel là một đường kẻ 1px `outline`, không phải khe hở, không phải bóng.

Chế độ đọc giới hạn chiều rộng bằng `ch` chứ không bằng `px` — thước đo đúng là số ký tự mỗi dòng, và nó phải giữ nguyên khi người dùng đổi cỡ chữ.

### Lưới 7px

*(🔵 2026-10-29, Ice chốt. Hình mẫu `mockups/luoi-7px.html`; lộ trình `plan-luoi-7px-va-canh-baseline.md`.)*

**Luật.** Mọi khoảng cách và mọi kích thước cố định là bội số của **7px**: 7 · 14 · 21 · 28 · 35 · 42 … Khoảng nhỏ nhất khác 0 là 7px. Luật áp cho **cả chiều dọc lẫn chiều ngang**: đệm trên/dưới và trái/phải, khe giữa các mục trong hàng, lề, thụt, chiều cao thanh và hàng, bề rộng cột và mọi bề rộng cố định.

**Ngoại lệ — thân lưới Workspace giữ nguyên số hiện tại:** chữ nguyên văn (cả dòng Hán Việt), chữ bản dịch, ô đang sửa, đệm của các ô đó và các cột trong hàng (`grid-*` ở front-matter). Khung quanh lưới theo luật: thanh panel, hàng công cụ, dải TM · Glossary · Soát, thanh trạng thái. Chế độ đọc và panel Tra cứu **không** được miễn. Ngoại lệ đi qua token có tên `--space-grid-*`, không qua danh sách tệp loại trừ.

**Thang.** Chuyển từ đơn vị 4px:

| Cũ | Mới |
|---|---|
| 2 · 4 · 6 · 8 | **7** |
| 12 · 16 | **14** |
| 20 · 24 | **21** |
| 28 | 28 |
| 32 · 36 | **35** |
| 40 | **42** |

**Trong nhóm 7, giữa nhóm 14.** Chỗ nào thang cũ dùng 4 để nối các dòng trong một nhóm và 8 để tách các nhóm (tên · tác giả, rồi tiến độ, rồi hàng nút), thang mới dùng **7 và 14**. Chuyển máy móc đưa cả hai về 7 và mắt mất ranh giới nhóm.

**Nét kẻ nằm trong ô lưới.** Viền và nét kẻ không cộng thêm vào lưới: khối có viền 1px trừ 1px ở đệm cùng phía, theo cả hai chiều. Hàng công cụ: đệm 7 + nút 28 + đệm 6 + nét đáy 1 = 42. Thẻ Library: viền 1 + đệm 13 mỗi phía.

**Bề rộng.**

- Bề rộng cố định bằng px là bội 7 (cột, ô vuông biểu tượng, lề cho ba nút cửa sổ).
- Bề rộng đo bằng `ch` (thước đọc, ô nhập ngắn 8ch và vừa 24ch) giữ nghĩa số ký tự nhưng làm tròn lên bội 7: `round(up, 24ch, 7px)`. ⚠️ Hàm `round()` của CSS phải được kiểm trên WKWebView của macOS và trên WebView2 trước khi dùng.
- `min-width: 8.5em` của cặp nút chân lớp phủ thành **112px**.
- Bề rộng ôm chữ (nút, nhãn, số) không ép được: CSS chưa làm tròn được kích thước theo nội dung trên WebKit. Mép trái của mục đầu hàng và mọi khe đều đúng lưới; mục đứng sau một mục ôm chữ lệch khỏi vạch lưới dọc, khe giữa hai mục vẫn là bội 7.

**Kích thước cố định.**

| Thành phần | Cũ | Mới |
|---|---|---|
| Thanh tiêu đề | 40 | **42** — đo lại `trafficLightPosition` trên máy Ice (AD-58 mục 6) |
| Lề cho ba nút cửa sổ (`traffic-light-inset`) | 88 | **91** |
| Thanh panel | 36 | **35** |
| Thanh trạng thái | 34 | **35** |
| Đệm panel ngang · dọc | 16 · 12 | **14 · 14** |
| Lề mép cửa sổ (`screen-inline`): thanh tiêu đề, thanh trạng thái, Library, Chế độ đọc, dải chân | 14 | **21**. 🔵 2026-10-30 (Ice): lớp phủ và cạnh panel giáp panel khác giữ `panel-inline`; panel Workspace chạm mép cửa sổ vẫn tràn sát mép, chỉ đệm phía mép đó là `screen-inline` |
| Điều khiển | 24 / 28 / 32 | **28 / 28 / 35** (§Điều khiển) |
| Biểu tượng · vùng bấm | 16 · 26 | **14 · 28** |
| Vùng bấm checkbox · công tắc | 24 · 28×16 | **28 · 28×14** |
| Vùng đầu mục Tra cứu · dải đầu nguồn | 40 · 32 | **42 · 35** |
| Vạch + thụt bản ghi từ điển | 2 + 13 | **2 + 12** |

**Giãn dòng.**

- Chữ giao diện (`ui-*`), một dòng lẫn xuống dòng: **21px**.
- Chữ nội dung ngoài thân lưới Workspace: bội 7 nhỏ nhất không chật hơn giãn dòng cũ — `read-lg` 42 · `read-md` 35 · `read-sm` 28 · `read-title` 35 · `lookup-headword` 35 · `lookup-gloss` 28 (🔵 2026-10-30 (Ice): nay 21, dưới sàn 1.6, ngoại lệ có tên). Ice chọn luật này thay cho luật "tỉ số ≥ 1.6", vốn làm mức Cân mặc định chật hơn hôm nay 11%.
- Ngoại lệ có tên của luật trên: `lookup-example` 21px (1.615), chật hơn giãn dòng cũ 21,58px; Ice chọn để ví dụ cùng nhịp với chữ giao diện 13px thay vì 28px (2.15).
- Ngoại lệ có tên của luật trên: `cover-name` 23px (≈ 1.45), không phải bội 7 và dưới sàn 1.6. 🔵 2026-11-01 (Ice): tên Tác phẩm trên bìa cắt ở ba dòng, bội 7 gần nhất là 21px (1.31) hoặc 28px (1.75).
- Sàn của mọi chữ xuống dòng là **1.6** (trước là 1.66), chung cho họ `read` và `ui`.
- Token lưu giãn dòng là **tỉ số không đơn vị** bằng bội 7 chia cỡ chữ (21/13 = 1.6153846…), không lưu `px`: tỉ số giữ nguyên khi phần tử con đổi cỡ chữ, còn `px` thừa hưởng nguyên giá trị. Đo trên Chrome: tỉ số đầy đủ cho đúng 21,000px mỗi dòng ở bảy cỡ chữ; WKWebView chưa đo.
- Thanh trượt giãn dòng của Chế độ đọc: giá trị lưu vẫn là tỉ số như trước, không đổi dữ liệu đã lưu; giãn dòng hiển thị là bội 7 nhỏ nhất ≥ cỡ chữ × tỉ số, nên thanh trượt nhảy theo bậc 7px và không bao giờ xuống dưới sàn của nó.
- Thân lưới Workspace giữ `source-cjk` · `source-latin` · `source-hanviet` · `editor` như cũ. 🔵 2026-10-30 (Ice): thêm `source-cjk-strong`, cùng tỉ số của `source-cjk`. 🔵 2026-11-04 (Ice): `source-cjk-strong` đã gỡ (§Bảng token typography hàng 16).

**Canh baseline.**

- Hàng ngang có chữ, số, biểu tượng hoặc điều khiển canh theo **baseline**. Hàng trong thanh cao cố định canh baseline giữa các mục, rồi cả nhóm nằm giữa thanh. Biểu tượng và chữ bên trong một nút cũng canh baseline, nhóm nằm giữa nút.
- Biểu tượng 14px: **đáy biểu tượng đặt trên baseline** của chữ cạnh nó, như một chữ hoa. 🔵 2026-10-30 (Ice): mũi tên xổ xuống (`chevron`) đứng cạnh chữ thì hạ 3px để tâm nét vẽ trùng giữa chiều cao chữ hoa, vì nét vẽ của nó nằm giữa ô chứ không lấp đầy ô.
- ⚠️ Nút chỉ có biểu tượng không có baseline, nên đáy cả nút rơi xuống baseline của hàng và nút nhô lên 7px so với nút chữ cùng cao. Nó phải mang một hộp dòng rỗng để có baseline đúng chỗ của nút chữ cùng bậc.
- Hàng baseline đứng tự do khai chiều cao bằng bội 7: 21 cho hàng chữ, 28 cho hàng điều khiển. Chữ `ui`, số `mono` và chip phím canh baseline thành hàng 21,5px, vì mỗi font nhô trên baseline một khác.
- Phần tử không mang chữ và không phải biểu tượng (nét ngăn ở thanh trạng thái, thanh tiến độ) canh giữa hàng.
- ⚠️ Cách flexbox tự sinh baseline cho phần tử không có chữ khác nhau giữa WebKit và WebView2. Phần Windows chỉ kiểm được ở `B7`.

**Không theo lưới.** Bề dày viền và vạch (1 · 2 · 3px), bo góc, cỡ chữ, hình học của ba nút cửa sổ do hệ điều hành vẽ, và thanh cuộn 10px. Thanh cuộn nằm ở mép panel, ngoài mọi đệm, nên nó không đẩy mép trái của nội dung nào; thu xuống 7px thì khó bắt chuột hơn. 🔵 2026-10-30 (Ice): vùng cuộn chạm mép cửa sổ giữ sẵn chỗ thanh cuộn (`scrollbar-gutter: stable`) và trừ 10px khỏi đệm phải, để lề trái và lề phải bằng nhau.

## Elevation & Depth

~~**Không có elevation.** Không bóng đổ, không lớp nổi, không z-index trang trí.~~ 🔵 2026-10-29 (Ice): **bóng đổ và lớp nổi được dùng ở nơi cần tạo chiều sâu.** Lớp nổi dùng khi nội dung phải bám ngay cạnh chỗ người dùng vừa chọn (popup tra cứu ở Chế độ đọc); bóng đổ chỉ lấy giá trị từ token `shadows` trong `tokens.json` (`box-shadow: var(--shadow-…)`, `check:tokens` Kiểm F), mỗi theme một giá trị. Gradient vẫn cấm; `z-index` vẫn cần miễn trừ có tên, không z-index trang trí. Chiều sâu của mặt phẳng làm việc vẫn là **sắc độ**: `surface-sunken` cho vùng lùi, `surface` cho mặt phẳng làm việc.

~~Ngoại lệ duy nhất là bóng của chính cửa sổ ứng dụng, do hệ điều hành vẽ.~~ 🔵 2026-10-29: không còn là ngoại lệ duy nhất (xem trên); bóng cửa sổ vẫn do hệ điều hành vẽ.

### Phân tách panel đảo ngược giữa hai theme

Đây là quy tắc phản trực giác, phát hiện khi dựng Workspace ở nền tối — **đừng thống nhất hai theme về một cách làm.**

| Theme | Cơ chế | Vì sao |
|---|---|---|
| **Sáng** | Đường kẻ 1px `outline` giữa các panel | Nét ấm trên mặt sáng đọc được, và giữ được vẻ liền mạch của trang giấy |
| **Tối** | **Khe 2px để `background` lộ ra** giữa các panel, panel bo `3px` | Đường kẻ `outline #3b382f` trên mặt `surface #26241f` chỉ đạt tỉ lệ tương phản **1,39** — gần như vô hình. Thêm sáng cho đường kẻ thì thành khung viền chói, phá hướng "Bàn viết" |

Nguyên tắc chung: **mặt sáng phân tách bằng nét, mặt tối phân tách bằng khe.** Bê nguyên cách của theme sáng sang theme tối làm bốn panel chìm thành một khối nâu.

## Motion

Chuyển động ở đây phục vụ **giảm giật thị giác trong thao tác lặp**, không phục vụ vẻ mượt. Auto-Lookup chạy hàng trăm lần mỗi Chương ở dưới 100ms đầu-cuối — thêm hiệu ứng dài là tự tay làm chậm thứ nhanh nhất trong sản phẩm.

**Thủ phạm gây giật là layout nhảy, không phải thiếu hiệu ứng.** Sửa cấu trúc trước, thêm hiệu ứng sau.

| Tình huống | Quy tắc |
|---|---|
| Vùng đầu mục Panel Lookup | Cao **cố định**. Đầu mục và thanh nhịp luôn ở cùng toạ độ; chỉ phần dưới thay đổi |
| Nội dung mới vào | **90ms**, opacity **0.4 → 1**, `ease-out`. Không `translate`, không `scale` |
| Nội dung cũ | Thay thẳng, **không có hiệu ứng ra** |
| Tra liên tiếp | Tra mới **huỷ** hiệu ứng đang chạy, đặt thẳng opacity 1. Hiệu ứng **không xếp hàng** |
| Vị trí cuộn | Về đầu **tức thì**. Không bao giờ cuộn có hiệu ứng |
| Bôi đen đang kéo | Chỉ tra khi vùng chọn **đã dừng** |
| Vượt 250ms | Vạch tiến trình mảnh ở đáy vùng đầu mục, **không spinner** |
| Không tìm thấy | Cùng 90ms — trạng thái rỗng không được hiện chậm hơn trạng thái có kết quả |
| `prefers-reduced-motion` | Bỏ **toàn bộ** hiệu ứng, đổi tức thì. Thuộc sàn khả năng tiếp cận, không phải tuỳ chọn |

Ba cấm chung cho toàn ứng dụng: **không `translate` trong thao tác lặp** (dịch chuyển hàng trăm lần gây mỏi rõ rệt), **không hiệu ứng nào vượt 150ms** trên đường nóng, **không hiệu ứng xếp hàng**.

## Shapes

Bo góc gần như không có: `3px` mặc định, `2px` cho vạch và chip. Cửa sổ `9px`. Hình dạng chủ đạo không phải hộp bo tròn mà là **vạch dọc** — vạch lề trạng thái segment, vạch tiêu điểm panel, vạch trái của mỗi bản ghi từ điển. Vạch chiếm ít không gian hơn khung viền và không cắt văn bản thành ô.

## Components

**Panel** — thanh tiêu đề 35px (🔵 2026-10-29 (Ice): từ 34px), tiêu đề `ui-md` màu `on-surface-variant`, tab bên phải. Panel có tiêu điểm: vạch dọc 2px `primary` ở mép trái + tiêu đề chuyển `primary` in đậm. Không dùng viền bao quanh để báo tiêu điểm. 🔵 2026-10-25 Ice bỏ vạch dọc 2px: panel sát mép cửa sổ khiến vạch chạy dọc cả cửa sổ như một đường kẻ thừa; còn tiêu đề tab `primary` in đậm (tab đang chọn, không phải focus DOM).

**Bản ghi từ điển** — vạch trái 2px, thụt 12px (🔵 2026-10-29 (Ice): từ 13px, để vạch + thụt = 14). Nhãn nguồn `ui-label` màu `primary`. Từ loại `read` in nghiêng màu `on-surface-variant`. Nghĩa `lookup-gloss`. Ví dụ in nghiêng; trích dẫn có vạch trái `primary` để phân biệt với ví dụ. **Nhiều nguồn xếp chồng dọc, mỗi nguồn một khối — không bao giờ gộp.**

**Vạch lề segment** — vạch dọc trong cột vạch bên trái lưới, cao đúng bằng **hàng** tương ứng. `confirmed` đã xác nhận · `primary` đang sửa · `draft` đã dịch tay chưa ký · `tm-rule` gợi ý TM chờ xác nhận · `ornament` đã về hưu. 🔵 **2026-09-24 (Ice chốt, phiếu quyết #65):** giá trị *về hưu* rút khỏi vạch trạng thái — hàng về hưu không còn hiện trong lưới (AD-5), nên không đường nào tới nó. Token màu `ornament` vẫn sống cho `draft` và cho khối bị loại ở màn xem trước nhập.

> 🔵 **CẬP NHẬT 2026-08-15 (Story 2.5b) — HAI mệnh đề của đoạn này đã hết đúng, sửa tại chỗ:**
> ① *"Đây là **cách duy nhất** trạng thái segment được hiển thị"* — **hết đúng**: lưới có thêm một **cột nhãn trạng thái** đọc được bằng chữ. Đó không phải một lượt nói hai lần: vạch là kênh **thị giác**, và một người đi bàn phím hoặc dùng trình đọc màn hình không có nó — cột nhãn là lý do vạch được phép `aria-hidden`.
> ② *"văn bản không bị chia khối"* — **hết đúng**: văn bản **có** chia ô, mỗi câu một hàng, mỗi hàng hai ô chữ. Tiền đề cũ *(trang văn liền mạch)* không còn tồn tại sau lượt correct-course 2026-08-14.
> ⚠️ Máng **22px** cũng hết đúng — cột vạch nay rộng **3px** và vạch lấy chiều cao từ **track hàng** của `subgrid`, không từ một phép đo `getClientRects()`.
> 🔵 **2026-10-13 (D3, `change-hoan-thien-giao-dien-theo-mockup`):** cột nhãn trạng thái không còn hiện. Chữ trạng thái ở lại trong hàng, ẩn khỏi mắt, và là lý do vạch vẫn được `aria-hidden`. Vì vạch thành kênh thị giác duy nhất, các trạng thái của nó phải khác nhau **ngoài màu** (WCAG 1.4.1): hình nét (liền · đứt · chấm) hoặc độ dày, chọn trong story dựng lưới.
> 🔵 **2026-10-14 (Ice chốt, `story-workspace-luoi-hai-cot-va-thanh-trang-thai`):** hình đã chọn — `confirmed` liền 2px · `draft` đứt 2px · `tm-rule` chấm 2px · `primary` liền 3px; `none` không vạch. Vạch dùng `border-left` (kiểu nét + màu token); 12.13 dùng lại bộ hình này.
> 🔵 **2026-10-23 (Ice chốt, Story 2.14):** mọi vạch cùng dày 3px, chỉ khác kiểu nét và màu — `primary` liền · `confirmed` đôi · `draft` đứt · `tm-rule` chấm; `none` vẫn không vạch (ô giữ viền 3px trong suốt). Lý do: đổi độ dày vạch khi dời con trỏ buộc WebKit tính lại bố cục cả lưới (~700 → ~160 ms ở 9.850 câu, bản debug).
> 🔵 **2026-10-25 (Ice chốt):** mọi vạch là nét liền 3px, chỉ khác màu; bỏ nét đôi, đứt, chấm. Vạch câu đang sửa dùng token riêng `caret-rule` thay cho `primary`. WCAG 1.4.1 giữ bằng bậc độ sáng: trên `surface`, nháp 1,99 · TM 3,41 · đã xác nhận 5,57 · đang sửa 9,52 (sáng) và 2,00 · 3,61 · 6,75 · 11,05 (tối); hai vạch bất kỳ cách nhau ít nhất 1,63:1, `check:commands` Kiểm I đòi tối thiểu 1,5:1. `draft` và `tm-rule` đổi giá trị nên mọi nét khác dùng hai token này cũng đổi theo.

**Nút — ba bậc** *(🔵 2026-10-26, Ice duyệt; hình mẫu `mockups/workspace-tinh-chinh.html`)*. Trước đó mọi nút là chữ 12px gạch chân bằng `outline` (1,31:1), nên mắt không tách được nút khỏi chữ và nút khoá trông như nút bấm được.

| Bậc | Dáng | Dùng khi |
|---|---|---|
| Chính | Nền `on-surface`, chữ `surface` 600, viền cùng màu nền (14,19:1) | Tối đa **một** mỗi panel: hành động người dùng đến panel để làm ("Dịch câu này") |
| Phụ | Nền `surface`, viền 1px `control-rule`, chữ `on-surface` | Hành động khác ("Đưa sang bản dịch", "Nguồn 8/10 ▾", "Bảng chờ Glossary") |
| Chìm | Không nền, không viền, chữ `on-surface-variant` | Lối mở phụ ít dùng ("Prompt", "Công cụ") |
| Nguy hiểm | Nền `surface`, viền 1px và chữ `error` (7,74:1 sáng · 5,92:1 tối) | Hành động có chữ **Xoá** làm mất thứ không lấy lại được trong app ("Xoá khoá"). 🔵 2026-10-28 (Ice duyệt): bậc thứ năm; `error` thêm đúng việc này, ngoài câu lỗi |
| Khoá | Không nền, viền **đứt** 1px `outline`, chữ `on-surface-variant` | Mọi bậc khi `disabled` — khác dáng, không chỉ khác màu |

Cao 28px (24px ở dạng nhỏ trong hàng điều khiển và thanh trạng thái), bo `3px`, chữ `ui-md`. 🔵 2026-10-29 (Ice): dạng nhỏ bỏ, mọi nút trong hàng điều khiển và thanh trạng thái cao 28px; form 35px. 🔵 2026-10-28: chiều cao theo thang ba bậc ở §Điều khiển (thêm 32px cho form). Nút chính **không** dùng `primary`: màu nhấn giữ đúng ba việc ở §Colors. 🔵 2026-10-28, Ice chọn giữ dáng mockup `library-luoi-rong.html` cho nút trên bìa thẻ Library: "Tiếp tục dịch" nền `primary` ở thẻ đang chọn (`ui-btn--cover-on`), nền `surface` chữ `primary` ở thẻ rê chuột (`ui-btn--cover`), `⋯` nền `surface` chữ `on-surface-variant` (`ui-btn--cover-plain`). Một hành động có cặp huỷ ("Dịch câu này" / "Huỷ dịch") hiện **một** nút ở **cùng một chỗ**, đổi theo trạng thái, để hàng nút không nhảy bố cục.

**Câu dịch tham khảo** *(🔵 2026-11-01, FR154–FR155)*. Nằm dưới chữ gốc, cùng cột với chữ gốc, không viền và không nền riêng. Chữ `read-sm`, màu `on-surface`, riêng khối đã loại và câu đang chảy dùng `on-surface-variant` để không nổi hơn chữ gốc. Dòng nguồn `ui-sm`, màu `on-surface-variant`. Không dùng `primary`, `tm-rule` hay `confirmed`, vì câu này không phải bản dịch đã qua ai. Khối dịch ở tầng khối giữ nguyên vạch lề và độ lùi của khối.

**Dấu quy ước** *(🔵 2026-11-03, Ice, FR159)*. Gạch chân `dotted` 2px, màu `on-surface-variant`, `text-underline-offset: 4px`, vẽ dưới đúng cụm cần tránh trong ô bản dịch và trên gợi ý ở panel AI Translation. Không thêm token: dấu quy ước và dấu Proofreader (lượn sóng 1px, `error` hoặc `tm-text`) phân biệt bằng **hình nét**, và Glossary đã chốt ở cột nguồn là màu chữ `primary`. Chấm 2px đã có ở vạch phân tách ô không liên kết (cạnh phải ô) và ở câu chưa xác nhận trong Chế độ đọc (viền đáy 1px `ornament`): khác bề mặt, khác loại vạch, nên không nhầm. Nền tô cụm bị loại, cùng lý do với Proofreader.

Tương phản của nét `on-surface-variant`, tính từ `tokens.json` (công thức WCAG), ngưỡng phi chữ 3:1 (WCAG 1.4.11):

| Nền | Sáng | Tối |
|---|---|---|
| `surface` | 5,60:1 | 5,56:1 |
| `surface-accent` (ô bản dịch đang sửa) | 4,96:1 | 4,51:1 |

Cả bốn cặp đạt 3:1, và đạt cả 4,5:1 của chữ. Hàng `surface-accent` tối dùng `#283637` của `tokens.json`; bảng token màu ở trên ghi `#2c3a3b` cho ô này, với giá trị đó cặp tối là 4,25:1, vẫn đạt 3:1. Hai giá trị lệch nhau cần được sửa ở một trong hai nơi.

**Dải quy ước** *(🔵 2026-11-03, Ice, FR159)* — một biến thể của dải Proofreader, không phải component thứ hai. Cùng vỏ: dải đẩy văn bản xuống, nền `surface`, viền trên 1px `outline`, vạch trái 3px, đệm 13px 14px 14px 11px, đầu mục `ui-md` đậm cùng chiều cao, nút `ui-btn` 28px. Khác ở ba chỗ: vạch trái `on-surface-variant` thay cho `error`; tên dải *Quy ước chữ Việt*; hành động chính **Nhận thay** là nút Chính, các nút còn lại Phụ, hai nút điều hướng và liên kết là Chìm. Luật không có cụm thay thế thì Nhận thay ở bậc Khoá (viền đứt `outline`, chữ `on-surface-variant`). Dòng trạng thái `ui-sm` `on-surface-variant`; lỗi `error`. Dải xếp ngay sau `proofreader` trong thứ tự ưu tiên dải.

**Lớp phủ Quản lý quy ước** *(🔵 2026-11-03, Ice, FR159)* — cùng vỏ lớp phủ với Quản lý Glossary: rộng tối đa 1099px, viền 1px `outline`, nền `surface`, đệm 13px. Thanh tầng là nút phân đoạn (ô đang chọn nền `on-surface`). Hàng tầng Tác phẩm có vạch trái 2px `confirmed`; hàng Toàn cục bị che thụt `calc(var(--space-unit) * 5)`, chữ `on-surface-variant`, nhãn viền đứt 1px *Đang bị che*; trạng thái *Đã tắt ở Tác phẩm này* cũng là nhãn viền đứt, nên trạng thái đọc được bằng chữ lẫn hình. Nút xoá bậc Nguy hiểm, hai bước. Nút điều khiển theo §Điều khiển; không thêm dáng nào.

**Màn xác nhận thay cả Chương** *(🔵 2026-11-03, Sally đặt mặc định, Ice xem qua mockup)* — lớp phủ rộng tối đa 917px như Cài đặt, danh sách cuộn tối đa 60vh. Bộ lọc luật là nút phân đoạn (ô đang chọn nền `on-surface`) đặt dưới đầu màn, mục đầu *Tất cả luật*. Nhóm theo câu, đường kẻ dưới `outline-faint`. Chữ cũ nền `diff-del-bg` chữ `diff-del-ink`, chữ mới nền `diff-add-bg` chữ `diff-add-ink` (cùng cặp với Review Mode). Nhãn câu nháp chữ `error` đậm, nhãn câu đã xác nhận chữ `on-surface-variant`; cả hai là chữ nên màu không là kênh duy nhất. Ô chọn là ô chọn của §Điều khiển (14px trong vùng bấm 28×28px). Chân màn nền `background`, nút chính bậc Chính, **Huỷ** bậc Phụ, chung `min-width: 8.5em`. Khung "chưa thay chỗ nào" viền trái 2px `error`, `role="alert"`.

### Điều khiển — một hệ cho mọi nút, ô nhập, ô chọn

*(🔵 2026-10-28, Ice duyệt. Ba điểm Ice chọn giữa hai phương án: hover nhẹ, focus đổi màu viền, thang 32/28/24. Phần còn lại Ice duyệt trên mockup `mockups/he-dieu-khien.html`. 🔵 2026-10-28, lượt soi lại: nút Chính rê chuột đổi nền sang `on-surface-variant` (Ice: viền trong 1px quá nhỏ, không thấy); nút Chìm rê chuột hiện viền thay cho nền 1,13:1; mục menu nền `outline`; vùng bấm checkbox 24px; dự phòng `forced-colors`.)*

**Vì sao.** Rà 2026-10-28 trên 66 tệp `.vue`: 356 `<button>`, chỉ 47 dùng lớp chung của `src/tokens/controls.css`; 309 nút còn lại tự vẽ bằng 148 lớp riêng trong 51 tệp. Chiều cao ước từ CSS rải 24 · 26 · 28 · ~29,5 · ~37,5px. 14 nút chính tô `primary`, 34 nút còn kiểu chữ gạch chân `outline` (1,31:1), 49 nút Phụ viền `outline` thay `control-rule`. Checkbox/radio chạy dáng gốc của hệ điều hành. Hover có ở 2/148 lớp nút tự vẽ.

**Thang kích thước.** Mọi control trên cùng một hàng dùng cùng một bậc, nên nút, ô nhập và ô chọn đứng thẳng hàng.

| Bậc | Cao | Đệm ngang (nút · ô nhập) | Dùng ở |
|---|---|---|---|
| `lg` | 35px | 14px · 7px | Form trong lớp phủ và Cài đặt |
| `md` | 28px | 14px · 7px | Thân panel (mặc định), hàng điều khiển, thanh trạng thái, dải gợi ý (TM, Glossary, Soát) |

🔵 2026-10-29 (Ice): thang 32/28/24 thành **28/28/35**: bậc `sm` 24px bỏ, chỗ nào dùng `sm` chuyển sang `md`; `lg` 32 → 35; đệm ngang 12 · 8 → 14 · 7. Chip lọc, `ui-seg--sm` và nút biểu tượng `--sm` cũng thành 28px. Bậc 28px trong thanh trạng thái 35px còn 3,5px mỗi bên; các mục canh baseline, cả nhóm nằm giữa thanh.

Chữ luôn `ui-md`, bo `3px`, viền 1px. Nút chỉ có biểu tượng là ô vuông cạnh bằng chiều cao của bậc (thay ô 26px cố định). Chip lọc dùng bậc `sm`, bo `2px` (§Shapes), không bo tròn viên thuốc.

🔵 2026-10-28, Ice chọn: thanh lọc Quản lý Glossary chạy trọn bậc `md` 28px, chip ở đó mang `ui-chip--md`. Nút phân đoạn `ui-seg` cao 28px tính cả viền; hàng toàn bậc `sm` (thanh tiêu đề, Chế độ đọc) dùng `ui-seg--sm` 24px. 🔵 2026-10-29 (Ice): `ui-seg--sm` cao 28px như bậc `md`.

**Chiều rộng.** 🔵 2026-10-29 (Ice): bề rộng theo §Lưới 7px — ô nhập đo bằng `ch` làm tròn lên bội 7, `min-width: 8.5em` thành 112px. Nút ôm chữ; riêng cặp nút ở chân lớp phủ dùng chung `min-width: 8.5em` để Huỷ và Xác nhận bằng nhau. Ô nhập đo bằng `ch` như thước đọc: **ngắn 8ch** (số, ngưỡng %), **vừa 24ch** (tên, khoá), **kín hàng 100%** (tìm, đường dẫn, mẫu luật). 🔵 2026-10-28, Ice chọn: ô Khoá API ở Cài đặt kín hàng như ô Endpoint ngay trên nó. Ô chọn dạng field rộng bằng nhãn dài nhất của nó, tối thiểu 12ch.

**Các loại.**

| Loại | Dáng mặc định |
|---|---|
| Nút | Năm bậc ở §Nút (Chính · Phụ · Chìm · Nguy hiểm · Khoá) |
| Ô nhập · ô nhiều dòng | Nền `surface-sunken`, viền `control-rule` (3,21:1 sáng · 3,98:1 tối — đạt sàn 3:1 của WCAG 1.4.11; viền `outline` 1,31:1 thì không), chữ `on-surface`, gợi ý `on-surface-variant`. Ô nhiều dòng giãn dòng 1.66 |
| Ô chọn (`SelectMenu`, `DropdownMenu` dạng field) | Dáng y như ô nhập, nhãn trái, dấu `▾` phải màu `on-surface-variant` |
| Nút mở menu (Công cụ ▾, Nguồn 8/10 ▾) | Một nút Phụ hoặc Chìm có dấu `▾` — không phải một dáng thứ ba |
| Mục trong menu | Cao bằng bậc `md`, đệm ngang 14px (🔵 2026-10-29 (Ice): từ 12px), không viền |
| Checkbox · radio | Vẽ bằng token, ô 14px trong vùng bấm 28×28px (🔵 2026-10-29 (Ice): từ 24×24px) (WCAG 2.5.8; đệm trong suốt khi ô đứng riêng không có nhãn bọc, như `gh-check`, `ip-cleanup-tick`): viền `control-rule`, checkbox bo `2px`, radio tròn. Đang chọn: nền `on-surface`, dấu tick/chấm `surface`, như nút Chính và ô phân đoạn đang chọn. Không dùng `primary`, không để màu nhấn của hệ điều hành. 🔵 2026-11-01 (Ice): ô 14px nằm giữa dòng chữ đầu của nhãn, nhãn nhiều dòng thì ô theo dòng đầu; hàng một dòng vẫn cao 28px |
| Công tắc (`role="switch"`) | Rãnh 28×14px (🔵 2026-10-29 (Ice): từ 28×16px) viền `control-rule`, núm `on-surface-variant`; bật: rãnh nền `on-surface`, núm `surface`. Chữ trạng thái vẫn đứng cạnh |
| Nút phân đoạn · tab con · chip phím | Như §Ba cấp điều khiển |

**Trạng thái.** Mỗi trạng thái đổi **một** thuộc tính và không đổi độ dày viền, đệm hay cỡ chữ, để không xô bố cục. 🔵 2026-10-29, Ice sau lượt nhìn trang Tác phẩm: hover và nhấn của nút Phụ quá nhạt (viền đổi 1,54:1, nền nhấn 1,13:1) nên nay đổi cả nền lẫn màu viền; vẫn không đổi độ dày viền, đệm hay cỡ chữ. Nút đang mở thứ nó điều khiển (`aria-expanded`) hay đang là giá trị hiện hành (`aria-pressed`) mang dáng Nhấn.

| Trạng thái | Nút Phụ · Nguy hiểm | Nút Chính | Nút Chìm · mục menu | Ô nhập · ô chọn · checkbox |
|---|---|---|---|---|
| Rê chuột | Viền `on-surface-variant` (3,63 → 5,60:1), nền `surface-sunken` (🔵 2026-10-29) | Nền và viền `on-surface-variant` (chênh 2,53:1 sáng · 2,18:1 tối so với nền mực), chữ `surface` 5,60 · 5,56:1. Token trung tính duy nhất giữ chữ ≥ 4,5:1: `control-rule` chỉ 3,63:1; `confirmed`, `tm-text`, `caret-rule` mang nghĩa trạng thái | Nút Chìm: viền `control-rule` (3,63 · 3,55:1), như `ui-icon-btn`. Mục menu: nền `outline` (1,31:1; chữ 10,85 · 9,15:1) | Viền `on-surface-variant` |
| Nhấn · đang mở · đang chọn (🔵 2026-10-29) | Nền `outline` (1,31:1 so với `surface`), viền `on-surface` | Như rê chuột | Nút Chìm và `ui-icon-btn`: nền `outline`, chữ `on-surface`; nút Chìm rê chuột vẫn chỉ hiện viền. Mục menu: như rê chuột | — |
| Focus bàn phím | Viền `primary`, không vòng ngoài | Giữ vòng 2px `primary` cách mép 1px | Mục menu: vòng 2px `primary` thụt vào trong (như ô phân đoạn); nút Chìm: viền trong suốt thành `primary` | Viền `primary`, không vòng ngoài |
| Khoá | §Nút bậc Khoá: viền đứt `outline`, chữ `on-surface-variant`, con trỏ mặc định, không hover | ← | ← | Viền đứt `outline`, nền không, chữ `on-surface-variant` |
| Lỗi | — | — | — | Viền `error`, câu lỗi `ui-sm` màu `error` ngay dưới, giãn 1.66 |

Focus thắng rê chuột: rule `:focus-visible` đứng sau `:hover` trong cùng tệp. ⚠️ Viền `primary` và viền rê chuột `on-surface-variant` chỉ cách nhau 1,25:1 (sáng) · 1,20:1 (tối) về độ sáng; hai màu khác nhau ở sắc độ, nên hai trạng thái không được đè nhau. Chênh lệch giữa viền thường `control-rule` và viền focus `primary` là 1,93:1 (sáng) · 1,89:1 (tối). Mức này đạt WCAG 2.4.11 (AA) và đạt 3:1 so với nền (7,01 · 6,69:1), nhưng dưới mức 3:1 mà 2.4.13 (AAA) đòi cho chính sự thay đổi. Ice đã chọn mức này có biết con số; ai muốn lên AAA thì đó là một lượt quyết mới, không phải một lỗi.

**Chế độ tương phản cao (Windows `forced-colors`).** Hệ điều hành ép mọi màu viền về màu hệ thống, nên viền focus không còn khác viền thường; vì outline đã tắt, phần tử mất hẳn dấu focus. `controls.css` trả lại `outline: 2px solid Highlight` cho `:focus-visible` dưới `@media (forced-colors: active)`. Chỉ kiểm được trên Windows (B7).

**Ô nhập trên nền trũng.** Nền `surface-sunken` của ô nhập trùng nền thanh trạng thái, thanh tab dock và dải đầu nguồn từ điển. Số ô đặt trên các nền đó chưa đếm; việc dựng `ui-field` đếm trước, có ô nào thì thêm bổ nghĩa `ui-field--on-sunken` nền `surface` thay vì đổi luật chung.

**Một nguồn.** Mọi dáng ở mục này nằm trong `src/tokens/controls.css` dưới dạng lớp `ui-*` (`ui-btn`, `ui-icon-btn`, `ui-seg`, `ui-field`, `ui-check`, `ui-switch`, `ui-menu-item`, bổ nghĩa `--lg`/`--sm`). Component không khai lại chiều cao, viền, hover, focus hay dáng khoá trong `<style scoped>`; nó chỉ đặt bố cục (khoảng cách, chiều rộng). Luật focus "có viền thì đổi màu viền" (Kiểm H) giữ nguyên, nay chỉ cần đúng ở một tệp.

**Ba cấp điều khiển, ba dáng** *(🔵 2026-10-26)*. Tab dock: nền `surface-sunken`, nét đáy `outline`, tab đang chọn nền `surface` "mở" xuống thân panel. Tab con trong panel (Từ điển · Concordance · Lịch sử): chữ `ui-md`, tab chọn gạch chân 2px `on-surface` và 600. Công tắc chế độ xem (Trung · Hán Việt; bố cục Workspace): **nút phân đoạn** viền `control-rule`, ô đang chọn nền `on-surface`. Chỉ dẫn phím tắt cạnh tên panel là **chip phím** (`ui-mono`, viền `outline`), không đứng trơn như một tab.

🔵 2026-10-27 (Ice): nhóm dock chỉ có một panel **ẩn thanh tab**; nhóm từ hai panel vẫn hiện để không panel nào bị giấu. "Sắp xếp panel" trong `Công cụ ▾` hiện lại mọi thanh tab để kéo thả và lưu bố cục, nhớ qua các phiên. Kéo thả dùng sự kiện con trỏ của dockview (`dndStrategy: 'pointer'`).

**Biểu tượng** *(🔵 2026-10-26)*. SVG vẽ tay trong mã, nét `currentColor`, ô 16px trong vùng bấm 26px — **không** thêm thư viện biểu tượng. 🔵 2026-10-29 (Ice): ô 14px trong vùng bấm 28px; trong hàng, đáy biểu tượng đặt trên baseline của chữ (§Lưới 7px). Mọi nút chỉ có biểu tượng mang `aria-label` và `title` lấy từ i18n. Trạng thái bật/tắt nói bằng **hình** (nét rỗng · tô đặc), không bằng màu (WCAG 1.4.1). Đã dùng: Ghim (rỗng = chưa ghim, đặc = đã ghim; thay chữ "Ghim" ở đầu mục và từng mục từ), thu gọn/mở nguồn, Nguồn dữ liệu (ⓘ), hai preset bố cục (sơ đồ "Lưới bên trái" và "Lưới trên đỉnh", thay nhãn `Ⓑ-2` · `Ⓑ-1`).

**Khối trong panel Tra cứu** *(🔵 2026-10-26)*. Mỗi nguồn từ điển mở bằng một dải đầu cao 35px (🔵 2026-10-29 (Ice): từ 32px) nền `surface-sunken`, nét đáy `outline`: nhãn nguồn `ui-label` `primary`, số nghĩa `ui-sm`, nút thu gọn bên phải. Mục từ trong nguồn cách nhau 14px (🔵 2026-10-29 (Ice): từ 12px) và nét `outline-faint`. **Khối kết quả AI** là một khung nền `surface-sunken`, viền `outline`, bo `3px`; dòng token nằm trong khung, dưới nét `outline`. Nhóm hành động cho cả Chương tách bằng nhãn nhóm `ui-sm` 600 và nét `outline-faint`.

🔵 2026-10-26, lượt hai (Ice: "tiết kiệm lại"): vùng đầu mục Tra cứu còn **một hàng**: đầu mục · âm đọc · "N nguồn · M nghĩa" · nút ghim. Vẫn cao cố định (§Motion), nay 40px thay 76px (🔵 2026-10-29 (Ice): 42px); dải nguồn đầu tiên nằm sát ngay dưới nét đáy, không khoảng trống.

**Thanh cuộn** *(🔵 2026-10-26, Ice)*. Vùng cuộn của panel kéo ra sát mép panel, nên thanh cuộn nằm sát lề, không thụt theo đệm panel. Thanh mảnh 10px, không rãnh; con chạy bo tròn màu `ornament` (nét, không chữ), rê chuột thì `control-rule`. Một kiểu chung cho mọi vùng cuộn của app.

**Thanh trạng thái** *(🔵 2026-10-26, Ice)*. Chỉ nói trạng thái, không giải thích. Nhóm trái: vị trí và tiến độ; nhóm phải: dịch vụ và lối mở; giữa các mục là nét `outline`. Câu giải thích auto-save **bỏ hẳn** (Ice: thừa). Câu TM rút thành "TM {total} · {threshold}%", câu đầy đủ ở `title`.

**Câu đang sửa** *(🔵 2026-10-26)*. Ngoài vạch `caret-rule`, **ô bản dịch** của câu đang sửa có nền `surface-accent`, như mã đã làm từ trước. Mockup tô cả hàng, nhưng Ice giữ phạm vi ô (plan 12.26): lưới không có phần tử "cả hàng", tô đủ các cột phải thêm điều kiện con trỏ vào từng cột.

**Chế độ đọc theo mockup** *(🔵 2026-11-02, Ice)*. Thanh công cụ 42px và dải vị trí 28px nằm trong `section.mode`, trôi cùng nội dung, không ghim. 🔵 2026-11-04 (Ice): thanh công cụ nay ghim ở mép trên khung cuộn khi cuộn xuống, dải vị trí vẫn trôi cùng nội dung. Lệch có chủ ý so với `reading-mode.html`: nút Tinh chỉnh thay bằng sáu icon cỡ chữ và giãn dòng ngay trên thanh công cụ; nguyên văn ở lề giữ cỡ và thẳng hàng hiện tại; chip Đánh dấu/Sửa giữ nguyên; mục chọn của nhóm Cỡ chữ dùng nền `surface-accent` chữ `primary` thay cho nền `on-surface` đặc của `ui-seg`; công tắc Song ngữ bật bằng `on-surface` (mockup dùng `primary`); nút đầu mốc biên dùng `ui-btn--primary` (nền `on-surface`) thay cho nền `primary`; cỡ chữ UI 13px, không 10.5px–12.5px; không dựng lại lớp phủ Mục lục và Danh sách đánh dấu.

**Ranh giới câu** — ký tự `⏐` màu `ornament`, `opacity: 0` mặc định, hiện ở `0.55` khi rê chuột hoặc khi con trỏ chạm.

**Công tắc và điều khiển đọc** — ba preset Thoáng/Cân/Đặc trên thanh công cụ; thanh trượt cỡ chữ và giãn dòng chi tiết nằm sau một lần bấm.

## Do's and Don'ts

**Nên**
- Dùng `primary` cho đúng ba việc: thuật ngữ Glossary, nhãn nguồn, tiêu điểm bàn phím.
- Báo trạng thái bằng vạch lề, giữ văn bản liền mạch.
- Kiểm mọi thay đổi typography bằng chuỗi dày dấu tiếng Việt.
- Giữ `read` cho nội dung và `ui` cho bộ máy, không trộn.
- Giữ ba hình gạch chân đúng nghĩa: lượn sóng 1px là Proofreader, chấm 2px là quy ước chữ Việt, liền 2px là Glossary chờ chốt.

**Không nên**
- Không viết giá trị màu thẳng trong component — mọi màu đến từ token đã kiểm tương phản (AD-34).
- Không dùng `ornament` hay `tm-rule` làm màu chữ.
- Không hạ giãn dòng văn bản xuống dưới 1.66.
- Không thêm gradient. 🔵 2026-10-29 (Ice): bóng đổ và lớp nổi không còn bị cấm; bóng chỉ lấy từ token `shadows`, không viết tay.
- Không dùng trắng tinh làm nền hay đen tuyền làm chữ, ở cả hai theme.
- Không chia Editor thành ô hay bảng — điều đó phá đúng thứ người dùng đã chọn.
- Không trộn hình nét giữa các họ dấu và không phân biệt hai họ chỉ bằng màu: dấu quy ước không được vẽ lượn sóng hay đổi sang màu `error` / `tm-text`, dấu Proofreader không được vẽ chấm.
