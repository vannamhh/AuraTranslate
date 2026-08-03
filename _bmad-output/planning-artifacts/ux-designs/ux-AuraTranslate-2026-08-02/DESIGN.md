---
name: AuraTranslate
status: final
created: 2026-08-02
updated: 2026-08-02
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
    fontSize: 11.5px
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
  read: '"Source Serif 4", "Noto Serif CJK SC", "Noto Serif CJK TC", serif'
  read-cjk: '"Noto Serif CJK SC", "Noto Serif CJK TC"'
  ui: '"Source Sans 3", ui-sans-serif, -apple-system, "Segoe UI", system-ui, sans-serif'
  mono: 'ui-monospace, SFMono-Regular, Consolas, monospace'
fonts-bundled:
  policy: 'Nhúng trong bản cài. Không CDN, không tải sau khi cài (AD-15, FR27). Tải tệp về rồi đóng gói — CẤM fonts.googleapis.com, đó vẫn là origin từ xa.'
  channel: 'Kênh Google cho cả ba họ (Ice chốt 2026-08-03). Latin: google/fonts ofl/sourceserif4 + ofl/sourcesans3, font biến thiên. CJK: notofonts/noto-cjk release Serif2.003 — cùng một font với Source Han Serif 2.003R, khác nhãn.'
  license: 'SIL OFL 1.1 — tương thích GPL v3 theo diện gộp gói. Noto Serif CJK và Source Serif 4 (bản Google Fonts) KHÔNG khai Reserved Font Name; Source Sans 3 CÓ khai RFN "Source" nên subset tệp đó phải đổi tên font. Kết luận rà theo NFR15 chốt ở Story 1.1.'
  weights: 'Latin dùng font biến thiên nên phủ trọn dải nét 200–900 chỉ với 3 tệp — nét 600 và 700 trong bảng token được lo trọn, không cần đóng gói thêm. CJK chỉ Regular, để ghìm dung lượng.'
  size-budget: 'CHƯA ĐO — ước ≈21,6 MB trên đĩa (≈19 MB CJK + ≈2,6 MB Latin biến thiên). Chênh lệch installer thật chốt ở Story 1.1.'
  region-variant: 'CHƯA CHỐT — Noto Serif CJK SC hay TC; coverage như nhau, chỉ khác dáng chữ ưu tiên ở mã dùng chung. Bắt buộc dùng bản vùng ĐẦY ĐỦ (NotoSerifCJKsc / NotoSerifCJKtc), KHÔNG dùng bản subset theo ngôn ngữ (NotoSerifSC / NotoSerifTC).'
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

### Bảng token màu — **16 token mỗi theme**

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

Sống ở `src/tokens/`. **Cấm giá trị màu viết thẳng trong component** (AD-34) — thứ cần kiểm tra tập trung thì không được rải rác.

> **Vì sao 16 chứ không phải 17.** Một số bản nháp ghi *"17 token"*. Con số đúng là **16** — đếm trên chính bảng này. `tm-rule` giữ **cùng một giá trị ở cả hai theme** (nó là vạch, không chịu ràng buộc tương phản chữ), nên nó dễ bị đếm thành hai. Đừng thêm một token thứ 17 để cho khớp một con số cũ: mọi token mới đều phải qua vòng kiểm tương phản ở mục dưới.

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

Ba việc chưa xong ở bản trước, nay còn hai:

1. ~~**Rà giấy phép tường minh** theo `NFR15`~~ — ✅ **đã rà 2026-08-03.** Cả ba là **SIL OFL 1.1**, tương thích `GPL v3` theo diện **gộp gói** (font nằm cạnh mã, không liên kết vào mã). Một khác biệt phải nhớ: `Noto Serif CJK` và `Source Serif 4` bản Google Fonts **không khai** Reserved Font Name, nhưng `Source Sans 3` **có khai** RFN `'Source'` — nên nếu subset riêng tệp Sans thì phải đổi tên font. Kết luận ghi vào bảng Stack của `ARCHITECTURE-SPINE.md` ở Story 1.1.
2. **Đo dung lượng thật.** Ước **≈21,6 MB** trên đĩa (≈19 MB CJK + ≈2,6 MB Latin biến thiên) — thấp hơn hẳn ước tính 30–50 MB của bản trước, nhờ font biến thiên. Vẫn phải đo **chênh lệch installer** thật vì `.dmg` và `.msi` đều nén. Ngân sách `NFR6` là 150–200 MB trên nền database đã 130 MB. Nếu vượt trần thì đây là thay đổi ở tầng PRD, không phải tầng thiết kế.
3. **Chọn biến thể vùng** cho Han: `Noto Serif CJK SC` hay `Noto Serif CJK TC`. Coverage như nhau — chỉ khác **dáng chữ ưu tiên** ở các mã chung. TC hợp mạch cổ văn và Hán Việt; SC hợp truyện mạng đương đại.

> ⚠️ **Một cái bẫy đặt tên phải viết ra, vì nó hỏng im lặng.** Google phát hành **hai loại tệp khác nhau** với tên gần giống: `NotoSerifCJKsc` / `NotoSerifCJKtc` là **biến thể vùng đầy đủ** — phủ trọn kho mã CJK, chỉ khác dáng chữ ưu tiên; còn `NotoSerifSC` / `NotoSerifTC` là **subset theo ngôn ngữ**, chỉ phủ bộ mã một vùng. Mục 3 ở trên nói tới loại thứ nhất. Lấy nhầm loại thứ hai thì phần lớn chữ vẫn hiện bình thường, chỉ **ô vuông rỗng** ở những mã thuộc hệ chữ kia — mà người dùng mục tiêu dịch **cả** truyện mạng giản thể **lẫn** cổ văn phồn thể. *(Bẫy này còn nhân đôi khi đối chiếu chéo với nhãn Adobe: `NotoSerifTC` tương đương `SourceHanSerifTW`, **không** tương đương `SourceHanSerifTC`.)*

### Bảng token typography — **14 token, bốn họ chữ**

> **Nguồn sự thật của bộ token chữ.** *(Bổ sung 2026-08-03 cùng lý do với bảng token màu.)*

Bốn họ chữ: **`read`** (`Source Serif 4`, biến thiên) · **`read-cjk`** (`Noto Serif CJK`, chỉ Regular) · **`ui`** (`Source Sans 3`, biến thiên) · **`mono`**.

> **Nét 600 và 700 trong bảng dưới không cần tệp riêng.** `read-title` dùng nét 600 và `ui-label` dùng nét 700 — cả hai đều nằm trong dải 200–900 của font biến thiên, nên chúng là nét **thật**, không phải nét tổng hợp giả. Riêng `read-cjk` chỉ có Regular: chữ Hán rơi vào token nét 600 hoặc nghiêng sẽ bị webview tổng hợp giả. Bảng token hiện **không có** token nào vừa là `read-cjk` vừa đòi nét đậm hay nghiêng, nên ca này chưa phát sinh — nhưng thêm một token như vậy về sau là thêm một tệp font, không phải thêm một dòng CSS.

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
| 11 | `ui-md` | 12px / 1.5 | `ui` | Tiêu đề panel |
| 12 | `ui-sm` | 11.5px / 1.5 | `ui` | Nhãn phụ |
| 13 | `ui-label` | 10px / 700 / 1.4 / `0.1em` | `ui` | Nhãn nguồn từ điển |
| 14 | `ui-mono` | 10.5px / 1.4 | `mono` | Phím tắt, số liệu |

### Bảng token khoảng cách và hình dạng

Đơn vị **4px**.

| Nhóm | Token |
|---|---|
| **Spacing** | `panel-inline 16px` · `panel-block 12px` · `head-height 34px` · `titlebar-height 38px` · `status-height 32px` · `gutter-width 22px` |
| **Thước đọc** | `read-measure-lg 62ch` · `read-measure-md 68ch` · `read-measure-sm 76ch` |
| **Bo góc** | `none 0` · `sm 2px` · **mặc định `3px`** · `md 4px` · `window 9px` · `full 9999px` |

### Giãn dòng 1.66 là sàn cứng — của chữ nội dung

**Dấu tiếng Việt chồng cả trên lẫn dưới** (`ế` `ộ` `ữ` `ẳ` `ườ`). Giãn dòng 1.5 — mức mặc định quen thuộc và ổn thoả với chữ Latin — làm dấu `ườ` ở dòng trên chạm dấu `ộ` ở dòng dưới.

Không token họ `read` nào được xuống dưới **1.66**. Đây là ràng buộc của ngôn ngữ, không phải khẩu vị thiết kế, và nó vô hình nếu chỉ thử bằng chữ Latin. Mọi lần kiểm bằng mắt phải dùng chuỗi dày dấu.

**Họ `ui` được phép ở 1.4 và 1.5** *(quyết định 2026-08-03)*. Ranh giới không phải kích cỡ chữ mà là **chữ có chạy thành đoạn hay không**:

| Loại chuỗi | Sàn | Vì sao |
|---|---|---|
| Chữ nội dung — họ `read` | **1.66** | Chạy thành đoạn dài, xuống dòng liên tục. Đây là chỗ dấu chồng dấu gây mỏi tích luỹ |
| Nhãn giao diện một dòng — `ui-md` `ui-sm` `ui-label` `ui-mono` | **1.4** | Không xuống dòng thì không có dòng dưới để chạm. Ghìm chặt giúp thanh panel 34px và thanh trạng thái 32px giữ được mật độ của một nhạc cụ nghề nghiệp |
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

**Vạch lề segment** — vạch dọc 2px trong máng rộng 22px bên trái Editor, cao đúng bằng câu tương ứng. `confirmed` đã xác nhận · `primary` đang sửa · `tm-rule` gợi ý TM chờ xác nhận. Đây là **cách duy nhất** trạng thái segment được hiển thị; văn bản không bị chia khối.

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
