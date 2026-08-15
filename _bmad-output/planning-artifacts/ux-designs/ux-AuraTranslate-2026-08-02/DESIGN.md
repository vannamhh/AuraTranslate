---
name: AuraTranslate
status: final
created: 2026-08-02
updated: 2026-08-14
sources:
  - _bmad-output/specs/spec-AuraTranslate/SPEC.md
  - _bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md
  - _bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md
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
  tm-rule: '#b99a5e'
  tm-text: '#7a5d25'
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
  tm-rule: '#b99a5e'
  tm-text: '#d3b276'
  error: '#e5867a'
typography:
  read-lg:
    fontFamily: read
    fontSize: 19px
    lineHeight: '1.95'
    letterSpacing: 0.004em
  read-md:
    fontFamily: read
    fontSize: 17.5px
    lineHeight: '1.8'
  read-sm:
    fontFamily: read
    fontSize: 16px
    lineHeight: '1.66'
  read-title:
    fontFamily: read
    fontSize: 23px
    fontWeight: '600'
    lineHeight: '1.3'
  source-cjk:
    fontFamily: read
    fontSize: 16.5px
    lineHeight: '2.05'
  source-hanviet:
    fontFamily: read
    fontSize: 12.5px
    fontStyle: italic
    lineHeight: '1.95'
  editor:
    fontFamily: read
    fontSize: 15px
    lineHeight: '1.95'
  lookup-headword:
    fontFamily: read
    fontSize: 24px
    lineHeight: '1.3'
  lookup-gloss:
    fontFamily: read
    fontSize: 14.5px
    lineHeight: '1.6'
  lookup-example:
    fontFamily: read
    fontSize: 12.5px
    fontStyle: italic
    lineHeight: '1.6'
  ui-md:
    fontFamily: ui
    fontSize: 12px
    lineHeight: '1.5'
  ui-sm:
    fontFamily: ui
    fontSize: 12px
    lineHeight: '1.5'
  ui-label:
    fontFamily: ui
    fontSize: 10px
    fontWeight: '700'
    lineHeight: '1.4'
    letterSpacing: 0.1em
  ui-mono:
    fontFamily: mono
    fontSize: 10.5px
    lineHeight: '1.4'
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
  unit: 4px
  panel-inline: 16px
  panel-block: 12px
  head-height: 34px
  titlebar-height: 38px
  status-height: 32px
  gutter-width: 22px
  read-measure-lg: 62ch
  read-measure-md: 68ch
  read-measure-sm: 76ch
components:
  panel-head: { height: 34px, typography: ui-md, color: on-surface-variant }
  panel-focus-rule: { width: 2px, color: primary }
  segment-gutter-rule: { width: 2px, radius: sm, inset-left: 8px }
  source-chip: { typography: ui-label, color: primary }
  record-rule: { width: 2px, color: outline-faint, padding-left: 13px }
  grid-row-divider: { height: 1px, color: outline-faint }
  grid-para-divider: { height: 1px, color: outline, space-block: 16px }
  grid-num-col: { width: 34px, typography: ui-mono, color: on-surface-variant, align: right }
  grid-state-col: { width: 108px, typography: ui-label, color: on-surface-variant }
  grid-empty-cell: { min-height: 1.95em, border-bottom: 1px dashed outline }
  grid-row-omitted: { color: on-surface-variant, decoration: line-through }
---

## Brand & Style

AuraTranslate là **bàn viết**, không phải bảng điều khiển. Người dùng ngồi với nó nửa buổi liền cho một Chương, nên phần mềm phải chịu lùi lại phía sau văn bản. Khung viền tan thành đường kẻ mảnh; thanh công cụ chỉ giữ những gì đang thật sự dùng; không có gradient, không có bóng đổ trang trí, không có màu nào chỉ để cho vui mắt.

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
| 14 | `tm-rule` | `#b99a5e` | `#b99a5e` | Vạch gợi ý TM — **là vạch, không phải chữ** |
| 15 | `tm-text` | `#7a5d25` | `#d3b276` | Chữ trong khối gợi ý TM |
| 16 | `error` | `#8f2f22` | `#e5867a` | Lỗi |
| 17 | `draft` | `#a9a196` | `#6a6459` | Vạch *đã dịch tay, chưa ai ký* — **là vạch, không phải chữ**. 🔵 Story 2.5b |

Sống ở `src/tokens/`. **Cấm giá trị màu viết thẳng trong component** (AD-34) — thứ cần kiểm tra tập trung thì không được rải rác.

> **Vì sao từng là 16, và vì sao nay là 17.** Một số bản nháp ghi *"17 token"* **vì một lý do sai** — `tm-rule` giữ **cùng một giá trị ở cả hai theme** (nó là vạch, không chịu ràng buộc tương phản chữ) nên nó dễ bị đếm thành hai. Cách đếm đó vẫn sai, và mệnh đề *"đừng thêm một token thứ 17 **để cho khớp một con số cũ**"* vẫn đúng từng chữ.
>
> 🔵 **2026-08-15 (Story 2.5b) — con số đổi vì một lý do ĐO ĐƯỢC, không vì một con số cũ.** UX-DR19 viết lại cùng lượt correct-course cấp cho *"đã dịch tay, chưa ai ký"* một giá trị vạch riêng, và `check-commands.mjs` Kiểm I đối chiếu **hai chiều** giữa `SEGMENT_RULE_VALUES` và các khối `.rule-<giá trị>` trong CSS — nó đòi đúng `var(--color-draft)`. Một `var(--color-ornament)` ở khối đó làm cổng **ĐỎ**, và một bảng alias để cho lọt là đúng thứ §Miễn trừ cấm bằng chữ.
>
> ⚠️ **`draft` MƯỢN ĐÚNG GIÁ TRỊ của `ornament` ở cả hai theme** *(Ice ký 2026-08-15)*, nên nó là **một cái tên mới cho một màu ĐÃ kiểm**, không một màu mới chưa ai đo — **0 cặp mới** cho `contrast.pairs`, và bảng cặp giữ nguyên **31** cặp mỗi theme. 🔴 Trùng giá trị **không** phải trùng nghĩa: `ornament` nói *đã về hưu*, `draft` nói *đã dịch tay, chưa ký*; hai vạch không bao giờ cùng xuất hiện trên một câu.

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

Ba việc chưa xong ở bản trước, **nay xong cả ba** *(Story 1.1, 2026-08-03 — số đo và bằng chứng ở [`research/font-spike-results-2026-08-03.md`](../../research/font-spike-results-2026-08-03.md))*:

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
| 1 | `read-lg` | 19px / 1.95 / `0.004em` | `read` | Chế độ đọc — mức **Thoáng** (62ch) |
| 2 | `read-md` | 17.5px / 1.8 | `read` | Chế độ đọc — mức **Cân** (68ch, mặc định) |
| 3 | `read-sm` | 16px / 1.66 | `read` | Chế độ đọc — mức **Đặc** (76ch) |
| 4 | `read-title` | 23px / 600 / 1.3 | `read` | Tiêu đề Chương |
| 5 | `source-cjk` | 16.5px / 2.05 | `read-cjk` | Nguyên văn tiếng Trung ở Panel Source |
| 6 | `source-hanviet` | 12.5px / italic / 1.95 | `read` | Âm Hán Việt |
| 7 | `editor` | 15px / 1.95 | `read` | Bản dịch trong Editor |
| 8 | `lookup-headword` | 24px / 1.3 | `read` | Đầu mục Panel Lookup |
| 9 | `lookup-gloss` | 14.5px / 1.6 | `read` | Nghĩa |
| 10 | `lookup-example` | 12.5px / italic / 1.6 | `read` | Ví dụ và trích dẫn |
| 11 | `ui-md` | 13px / 1.5 | `ui` | Tiêu đề panel |
| 12 | `ui-sm` | 12px / 1.5 | `ui` | Nhãn phụ |
| 13 | `ui-label` | 11px / 700 / 1.4 / `0.1em` | `ui` | Nhãn nguồn từ điển |
| 14 | `ui-mono` | 11.5px / 1.4 | `mono` | Phím tắt, số liệu |

### Bảng token khoảng cách và hình dạng

Đơn vị **4px**.

| Nhóm | Token |
|---|---|
| **Spacing** | `panel-inline 16px` · `panel-block 12px` · `head-height 36px` · `titlebar-height 40px` · `status-height 34px` · `gutter-width 22px` |
| **Thước đọc** | `read-measure-lg 62ch` · `read-measure-md 68ch` · `read-measure-sm 76ch` |
| **Bo góc** | `none 0` · `sm 2px` · **mặc định `3px`** · `md 4px` · `window 9px` · `full 9999px` |

> 🔴 **TẦNG VỎ GIAO DIỆN NÂNG MỘT BẬC — Ice chốt 2026-08-12, và nó LẬT một quyết định của chính tài liệu này.**
>
> Bảng trên đọc **13 / 12 / 11 / 11,5px**; trước 2026-08-12 nó là **12 / 11,5 / 10 / 10,5px**. Ba
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

Đơn vị 4px. Khoảng cách chặt ở phần bộ máy, rộng ở phần nội dung — panel chỉ cao 34px cho thanh tiêu đề, nhưng văn bản bên trong thở thoải mái.

Workspace là lưới **2×2** mặc định: hàng trên `Nguyên văn | Bản dịch`, hàng dưới `Tra cứu | Đề xuất AI`. Bố cục 4 cột là preset thay thế. Ranh giới giữa panel là một đường kẻ 1px `outline`, không phải khe hở, không phải bóng.

Chế độ đọc giới hạn chiều rộng bằng `ch` chứ không bằng `px` — thước đo đúng là số ký tự mỗi dòng, và nó phải giữ nguyên khi người dùng đổi cỡ chữ.

## Elevation & Depth

**Không có elevation.** Không bóng đổ, không lớp nổi, không z-index trang trí. Chiều sâu duy nhất là **sắc độ**: `surface-sunken` cho vùng lùi, `surface` cho mặt phẳng làm việc.

Ngoại lệ duy nhất là bóng của chính cửa sổ ứng dụng, do hệ điều hành vẽ.

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

**Panel** — thanh tiêu đề 34px, tiêu đề `ui-md` màu `on-surface-variant`, tab bên phải. Panel có tiêu điểm: vạch dọc 2px `primary` ở mép trái + tiêu đề chuyển `primary` in đậm. Không dùng viền bao quanh để báo tiêu điểm.

**Bản ghi từ điển** — vạch trái 2px, thụt 13px. Nhãn nguồn `ui-label` màu `primary`. Từ loại `read` in nghiêng màu `on-surface-variant`. Nghĩa `lookup-gloss`. Ví dụ in nghiêng; trích dẫn có vạch trái `primary` để phân biệt với ví dụ. **Nhiều nguồn xếp chồng dọc, mỗi nguồn một khối — không bao giờ gộp.**

**Vạch lề segment** — vạch dọc trong cột vạch bên trái lưới, cao đúng bằng **hàng** tương ứng. `confirmed` đã xác nhận · `primary` đang sửa · `draft` đã dịch tay chưa ký · `tm-rule` gợi ý TM chờ xác nhận · `ornament` đã về hưu.

> 🔵 **CẬP NHẬT 2026-08-15 (Story 2.5b) — HAI mệnh đề của đoạn này đã hết đúng, sửa tại chỗ:**
> ① *"Đây là **cách duy nhất** trạng thái segment được hiển thị"* — **hết đúng**: lưới có thêm một **cột nhãn trạng thái** đọc được bằng chữ. Đó không phải một lượt nói hai lần: vạch là kênh **thị giác**, và một người đi bàn phím hoặc dùng trình đọc màn hình không có nó — cột nhãn là lý do vạch được phép `aria-hidden`.
> ② *"văn bản không bị chia khối"* — **hết đúng**: văn bản **có** chia ô, mỗi câu một hàng, mỗi hàng hai ô chữ. Tiền đề cũ *(trang văn liền mạch)* không còn tồn tại sau lượt correct-course 2026-08-14.
> ⚠️ Máng **22px** cũng hết đúng — cột vạch nay rộng **3px** và vạch lấy chiều cao từ **track hàng** của `subgrid`, không từ một phép đo `getClientRects()`.

**Ranh giới câu** — ký tự `⏐` màu `ornament`, `opacity: 0` mặc định, hiện ở `0.55` khi rê chuột hoặc khi con trỏ chạm.

**Công tắc và điều khiển đọc** — ba preset Thoáng/Cân/Đặc trên thanh công cụ; thanh trượt cỡ chữ và giãn dòng chi tiết nằm sau một lần bấm.

## Do's and Don'ts

**Nên**
- Dùng `primary` cho đúng ba việc: thuật ngữ Glossary, nhãn nguồn, tiêu điểm bàn phím.
- Báo trạng thái bằng vạch lề, giữ văn bản liền mạch.
- Kiểm mọi thay đổi typography bằng chuỗi dày dấu tiếng Việt.
- Giữ `read` cho nội dung và `ui` cho bộ máy, không trộn.

**Không nên**
- Không viết giá trị màu thẳng trong component — mọi màu đến từ token đã kiểm tương phản (AD-34).
- Không dùng `ornament` hay `tm-rule` làm màu chữ.
- Không hạ giãn dòng văn bản xuống dưới 1.66.
- Không thêm bóng đổ, gradient hay lớp nổi.
- Không dùng trắng tinh làm nền hay đen tuyền làm chữ, ở cả hai theme.
- Không chia Editor thành ô hay bảng — điều đó phá đúng thứ người dùng đã chọn.
