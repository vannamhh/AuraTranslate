# Story 1.1: Mũi thăm dò font — đo dung lượng thật và rà giấy phép

Status: ready-for-dev

Epic: 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
Covers: NFR6 · NFR15 · **mũi thăm dò bắt buộc — chặn mọi story khác của Epic 1**

> **Loại story: MŨI THĂM DÒ (spike), không phải story cài đặt tính năng.**
> Sản phẩm bàn giao là **số đo thật + quyết định đã ghi vào tài liệu**, không phải mã nguồn sản phẩm.
> Không dòng mã nào của story này được đi vào cây nguồn thật. Xem §Ranh giới phạm vi.

## Story

As a **chủ dự án**,
I want **biết chắc bộ font nhúng nằm trong ngân sách NFR6 và giấy phép cho phép phân phối lại**,
So that **tôi không phải bóc font ra sau khi đã dựng nửa giao diện trên nó**.

## Acceptance Criteria

### AC1 — Đo dung lượng thật trên cả hai nền tảng

**Given** ba họ font đã chốt — `Source Serif 4`, `Source Han Serif` (chỉ Regular), `Source Sans 3`
**When** đóng gói thử một `.dmg` và một `.msi` có nhúng font, và một cặp không nhúng
**Then** chênh lệch dung lượng được ghi lại thành số cụ thể cho từng nền tảng
**And** con số đó cộng với 130 MB database đã đo phải nằm trong trần 150–200 MB của NFR6

### AC2 — Rà giấy phép theo NFR15

**Given** giấy phép SIL OFL của ba họ font
**When** rà theo NFR15
**Then** kết luận tương thích GPL v3 được ghi vào bảng Stack của `ARCHITECTURE-SPINE.md`
**And** nếu không tương thích, mũi thăm dò kết thúc bằng một **đề xuất font thay thế** chứ không bằng việc bỏ qua

### AC3 — Chọn biến thể vùng cho Source Han Serif

**Given** hai biến thể vùng TC và SC của `Source Han Serif`
**When** dựng thử cùng một đoạn văn bản chứa các mã Hán dùng chung
**Then** biến thể được chọn và **lý do** được ghi lại

### AC4 — Vượt trần là quyết định tầng PRD, không phải tối ưu kiến trúc

**Given** tổng dung lượng vượt trần NFR6
**When** mũi thăm dò kết thúc
**Then** kết quả được báo cáo là **thay đổi tầng PRD cần chủ dự án quyết**, không phải một tối ưu ở tầng kiến trúc
**And** **không story nào của Epic 1 bắt đầu trước khi kết quả này được ghi lại**

---

## Tasks / Subtasks

- [ ] **Task 1 — Dựng app thăm dò dùng một lần** (AC: 1, 3)
  - [ ] Tạo app Tauri v2 tối thiểu **trong thư mục scratchpad của phiên, KHÔNG trong repo AuraTranslate**. `create-tauri-app` được phép ở đây và **chỉ** ở đây.
  - [ ] App chỉ cần một cửa sổ hiển thị một đoạn văn bản thử — không cần panel, không cần IPC, không cần database.
  - [ ] Ghim `tauri` 2.11.5 · Vue 3.5.40 · Vite 8.2.0 đúng bảng Stack, để số đo dùng lại được cho bản thật.
  - [ ] ⛔ **Không** commit app này. ⛔ **Không** dùng nó làm scaffold cho Story 1.2 — xem §Ranh giới phạm vi.

- [ ] **Task 2 — Tải đúng tệp font từ kênh Google** (AC: 1, 3)
  - [ ] **Kênh đã chốt: Google cho cả ba họ font** (Ice quyết 2026-08-03). Không phải việc của mũi thăm dò nữa — xem §Hai kênh phát hành để biết lý do và các ràng buộc kèm theo.
  - [ ] Latin — tải từ `google/fonts`, **font biến thiên**: `ofl/sourceserif4/SourceSerif4[opsz,wght].ttf` · `ofl/sourceserif4/SourceSerif4-Italic[opsz,wght].ttf` · `ofl/sourcesans3/SourceSans3[wght].ttf`. **Ba tệp, phủ trọn dải nét 200–900** — không cần tệp riêng cho nét 600 và 700.
  - [ ] CJK — tải từ release `notofonts/noto-cjk` tag **`Serif2.003`**, lấy **chỉ nét Regular** của biến thể vùng đầy đủ (`09_NotoSerifCJKsc.zip` hoặc `10_NotoSerifCJKtc.zip`, chốt ở Task 6).
  - [ ] ⛔ **Không** tải qua `fonts.googleapis.com`. Tải tệp về đóng gói; AD-15 cấm mọi origin từ xa lúc chạy.
  - [ ] Ghi lại dung lượng **trên đĩa** của từng tệp trước khi đóng gói (số này khác với chênh lệch installer — xem §Bẫy đo lường).
  - [ ] Ghi nhận trong báo cáo: `Noto Serif CJK` chỉ có Regular nên chữ Hán rơi vào token nét đậm hoặc nghiêng sẽ bị **tổng hợp giả**. Bảng token hiện không có token nào như vậy nên ca này chưa phát sinh — chỉ cần xác nhận lại là đúng, **không** tự ý thêm nét.

- [ ] **Task 3 — Đo bốn bản build** (AC: 1)
  - [ ] macOS: build `.dmg` **không** font → ghi số. Build `.dmg` **có** font → ghi số.
  - [ ] Windows: build `.msi` **không** font → ghi số. Build `.msi` **có** font → ghi số.
  - [ ] Bốn bản build phải **giống hệt nhau ở mọi thứ khác** (cùng commit, cùng cấu hình, cùng chế độ release) — chỉ khác `bundle.resources`.
  - [ ] Ghi lại **chế độ cài WebView2 đã dùng** trên Windows (`downloadBootstrapper` là mặc định) — nó chi phối con số tuyệt đối, xem §Bẫy đo lường.
  - [ ] Ghi lại **dung lượng baseline** (bản không font) chứ không chỉ chênh lệch — cần cho phép tính tổng thật.

- [ ] **Task 4 — Tính tổng và đối chiếu NFR6** (AC: 1, 4)
  - [ ] Tính: `chênh lệch font (từng nền tảng) + 130 MB database` — đây là **phép tính nghiệm thu đúng nguyên văn AC1**.
  - [ ] Ghi thêm phép tính đầy đủ hơn để Ice có bức tranh thật: `baseline + chênh lệch font + database`, kèm ghi chú rằng 130 MB là số **trên đĩa** với **ba nguồn đầu tiên**, còn ước tính đủ nguồn là 150–200 MB.
  - [ ] Đối chiếu với trần NFR6 = **150–200 MB**.

- [ ] **Task 5 — Rà giấy phép theo NFR15** (AC: 2)
  - [ ] Xác nhận cả ba họ font là **SIL OFL 1.1** bằng chính tệp `LICENSE` trong release đã tải (không tin trang web thứ ba).
  - [ ] Ghi kết luận tương thích GPL v3, có nêu **ba ràng buộc kèm theo** của OFL: Reserved Font Name, đi kèm bản văn giấy phép, cấm bán riêng font. Chi tiết ở §Rà giấy phép.
  - [ ] Ghi rõ **tình trạng RFN khác nhau giữa ba tệp**: `Noto Serif CJK` không khai · `Source Serif 4` bản Google Fonts không khai · `Source Sans 3` **có** khai `'Source'`. Mở từng tệp mà đọc, đừng suy từ tệp này sang tệp kia.
  - [ ] Thêm **ba hàng font** vào bảng Stack của `ARCHITECTURE-SPINE.md` — cột `Name` · `Version` · `Giấy phép`, đúng khuôn các hàng crate đang có. Ghi tên theo **kênh Google** (`Noto Serif CJK`, không phải `Source Han Serif`).
  - [ ] Nếu kết luận là **không tương thích**: dừng lại và viết **đề xuất font thay thế** (ứng viên + lý do + hệ quả cho `DESIGN.md`). Không được kết thúc bằng "bỏ qua".

- [ ] **Task 6 — Chọn biến thể vùng TC hay SC** (AC: 3)
  - [ ] Dựng **một đoạn văn bản thử duy nhất** chứa các mã Hán mà TC và SC vẽ khác nhau (ví dụ: 骨 · 直 · 房 · 令 · 兪 · 者 · 雇 — các mã có glyph vùng khác nhau trong Source Han).
  - [ ] Render đoạn đó **hai lần** trong app thăm dò, một lần với biến thể TC, một lần với SC. Chụp màn hình cả hai.
  - [ ] Chọn một, ghi lý do. Bối cảnh đã có ở `DESIGN.md`: *TC hợp mạch cổ văn và Hán Việt; SC hợp truyện mạng đương đại.*
  - [ ] ⚠️ Dùng **biến thể vùng đầy đủ** — `SourceHanSerifTC` / `SourceHanSerifSC` (Adobe) hoặc `NotoSerifCJKtc` / `NotoSerifCJKsc` (Google). **Không** dùng bản subset theo ngôn ngữ: `SourceHanSerifTW`/`CN`, tức `NotoSerifTC`/`NotoSerifSC` — xem §Bẫy chọn tệp và cảnh báo tên gọi chéo kênh ở §Hai kênh phát hành.

- [ ] **Task 7 — Ghi kết quả vào tài liệu** (AC: 1, 2, 3, 4)
  - [ ] Viết `_bmad-output/planning-artifacts/research/font-spike-results-2026-08-03.md` theo **đúng khuôn** `phase-0-spike-results-2026-08-02.md` (frontmatter · bảng tóm tắt · từng phép đo · kết luận · việc cần Ice quyết).
  - [ ] `ARCHITECTURE-SPINE.md` → bảng **Stack**: thêm ba hàng font (AC2).
  - [ ] `ARCHITECTURE-SPINE.md` → bảng **Deferred**: đóng hai hàng *"Dung lượng và giấy phép font nhúng"* và *"Biến thể vùng cho Source Han Serif"*, dùng đúng khuôn gạch ngang + `✅ Đã đóng 2026-08-03` như hàng HVTĐTD và hàng FR115 đã dùng.
  - [ ] `DESIGN.md` → frontmatter `fonts-bundled`: thay `license: 'SIL OFL — CẦN RÀ TƯỜNG MINH…'`, `size-budget: 'CHƯA ĐO…'`, `region-variant: 'CHƯA CHỐT…'` bằng kết quả thật.
  - [ ] `DESIGN.md` → §Typography, mục *"Ba việc chưa xong"*: đánh dấu cả ba đã xong, giữ nguyên lý do.
  - [ ] `EXPERIENCE.md` → §Còn thiếu, gạch mục *"Font thật chưa đo"* theo khuôn các mục đã đóng.
  - [ ] `.memlog.md` của architecture: thêm một dòng `(version)` ghi số đo và một dòng `(decision)` ghi biến thể vùng đã chọn.
  - [ ] `.memlog.md` của ux-designs: thêm một dòng ghi biến thể vùng đã chốt và dung lượng thật, vì `DESIGN.md` và `EXPERIENCE.md` cùng thư mục đều đổi theo.

- [ ] **Task 8 — Nếu vượt trần NFR6** (AC: 4)
  - [ ] **Không** tự ý subset font, **không** tự ý bỏ một họ font, **không** tự ý đổi sang font hệ điều hành. Cả ba đều là quyết định tầng PRD/thiết kế.
  - [ ] Viết mục *"Cần Ice quyết"* nêu các đòn bẩy có thật kèm cái giá của từng đòn bẩy (§Đòn bẩy nếu vượt trần).
  - [ ] Nêu rõ trong báo cáo: **Epic 1 dừng ở đây** cho tới khi Ice quyết.

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Story này **có** làm | Story này **KHÔNG** làm |
|---|---|
| Dựng app Tauri dùng một lần để đo | Dựng scaffold thật của dự án — **đó là Story 1.2** |
| Đo chênh lệch dung lượng installer | Cài đặt token màu/chữ — **đó là Story 1.4** |
| Rà giấy phép ba họ font | Rà giấy phép các crate — làm dần theo từng story, theo NFR15 |
| Chọn TC hay SC, ghi lý do | Dựng `src/tokens/`, `@font-face` thật cho sản phẩm |
| Ghi kết quả vào 4 tài liệu quy hoạch | Commit bất kỳ mã nguồn nào vào repo |

> **Vì sao app thăm dò phải nằm ngoài repo.** Story 1.2 mang AC nguyên văn: *"không dùng bất kỳ starter template cộng đồng nào"* và bắt cây nguồn đúng hình dạng đã chốt. Một app `create-tauri-app` để lại trong repo sẽ **hoặc** bị dùng làm scaffold và vi phạm AC đó, **hoặc** nằm đó gây nhầm lẫn. Giai đoạn 0 đã đặt tiền lệ đúng: *"Mã nguồn của các mũi thăm dò nằm trong thư mục scratchpad của phiên làm việc, không đưa vào repo (mã dùng một lần)."*

### Trạng thái repo hiện tại

Repo **chưa có một dòng mã nguồn nào** — chỉ có tài liệu quy hoạch (`_bmad-output/`), cấu hình BMad (`_bmad/`, `.claude/`, `.github/agents/`) và `design-artifacts/`. Không có `src-tauri/`, không có `package.json`, không có `Cargo.toml`, không có workflow CI. Bốn commit gần nhất đều là commit tài liệu (PRD, UX, epics, memlog). **Không có mã cũ để tham chiếu hay giữ tương thích** — nhưng cũng không có gì để tái dùng, nên mọi thứ trong app thăm dò đều dựng từ đầu.

### Số liệu đã tra (2026-08-03) — dùng làm điểm khởi hành, vẫn phải đo thật

Nguồn: GitHub Releases API của Adobe, tra ngày 2026-08-03.

| Họ font | Bản mới nhất | Ngày phát hành | Repo |
|---|---|---|---|
| `Source Serif 4` | **4.005R** | 2023-01-20 | `adobe-fonts/source-serif` |
| `Source Sans 3` | **3.052R** | 2023-04-04 | `adobe-fonts/source-sans` |
| `Source Han Serif` | **2.003R** | 2024-07-30 | `adobe-fonts/source-han-serif` |

Dung lượng **tệp zip release** (đủ mọi nét, chưa lọc):

| Asset | Zip | Nội dung |
|---|---|---|
| `source-serif-4.005_Desktop.zip` | 16,8 MB | OTF, đủ nét + italic |
| `OTF-source-sans-3.052R.zip` | 2,3 MB | OTF, đủ nét |
| `10_SourceHanSerifTC.zip` | 132,4 MB | **7 nét**, biến thể vùng TC đầy đủ |
| `09_SourceHanSerifSC.zip` | 132,2 MB | **7 nét**, biến thể vùng SC đầy đủ |
| `15_SourceHanSerifTW.zip` | 45,1 MB | 7 nét, **subset theo ngôn ngữ** (Adobe-CNS1) |
| `14_SourceHanSerifCN.zip` | 65,8 MB | 7 nét, **subset theo ngôn ngữ** (Adobe-GB1) |

**Ước tính chỉ-Regular** (chia 7, phải xác minh bằng số đo thật): biến thể vùng đầy đủ ≈ **19 MB**; bản subset theo ngôn ngữ ≈ 6,4 MB (TW) / 9,4 MB (CN).

Phần Latin — **kênh Google rẻ hơn rõ rệt**, số đo thật từ `google/fonts`:

| Tệp | Dung lượng | Phủ |
|---|---|---|
| `SourceSerif4[opsz,wght].ttf` | **1,15 MB** | toàn dải nét 200–900 + trục optical size |
| `SourceSerif4-Italic[opsz,wght].ttf` | **0,82 MB** | nghiêng, toàn dải nét |
| `SourceSans3[wght].ttf` | **0,62 MB** | toàn dải nét |

**Tổng Latin ≈ 2,6 MB** qua kênh Google, so với ≈ 4–6 MB nếu lấy 5 tệp OTF tĩnh từ Adobe — **và phủ mọi nét**, nên hai token nét 600/700 của `DESIGN.md` được lo trọn mà không đóng gói thêm tệp nào. *(Ba tệp biến thiên này cũng xoá luôn ghi chú synthetic bold ở Task 2 cho phần Latin — nhưng **không** cho phần Hán, vì Hán vẫn chỉ có Regular.)*

**Tổng ước ≈ 21,6 MB** (19 MB Hán + 2,6 MB Latin) **trên đĩa** — thấp hơn hẳn ước tính 30–50 MB ghi ở `DESIGN.md`. Nếu số đo thật lệch xa khỏi khoảng này, kiểm lại xem có vô tình đóng gói thừa nét hoặc lấy nhầm bản biến thể vùng đầy đủ cho cả 7 nét không.

### Hai kênh phát hành — cùng một font, hai bộ ràng buộc

> ✅ **Ice chốt 2026-08-03: dùng kênh Google cho cả ba họ font.** `DESIGN.md` đã cập nhật theo. Mục này giữ lại làm hồ sơ lý do — không phải câu hỏi để mở lại.

Xác minh 2026-08-03 qua GitHub API. Cả ba họ font đều có **hai kênh phát hành song song**, và chúng **không tương đương về giấy phép**:

| DESIGN.md chốt | Kênh Adobe | Kênh Google |
|---|---|---|
| `Source Serif 4` | `adobe-fonts/source-serif` 4.005R, OTF tĩnh | `google/fonts` → `ofl/sourceserif4`, **font biến thiên** |
| `Source Sans 3` | `adobe-fonts/source-sans` 3.052R, OTF tĩnh | `google/fonts` → `ofl/sourcesans3`, **font biến thiên** |
| `Source Han Serif` | `adobe-fonts/source-han-serif` 2.003R | `notofonts/noto-cjk` tag `Serif2.003` → **`Noto Serif CJK`** |

**Source Han Serif và Noto Serif CJK là cùng một font mang hai tên** — dự án chung Adobe + Google công bố 2017. Bằng chứng cơ học: hai release cùng phát hành **2024-07-30**, và asset trùng khít dung lượng (`10_SourceHanSerifTC.zip` = `10_NotoSerifCJKtc.zip` = 132,4 MB; `09_SourceHanSerifSC.zip` = `09_NotoSerifCJKsc.zip` = 132,2 MB). Bảng ánh xạ tên: `SourceHanSerifTC` ↔ `NotoSerifCJKtc` · `SourceHanSerifSC` ↔ `NotoSerifCJKsc` · `SourceHanSerifTW` ↔ `NotoSerifTC` · `SourceHanSerifCN` ↔ `NotoSerifSC`.

> ⚠️ **Cảnh báo tên gọi chéo kênh:** Adobe gọi bản subset theo ngôn ngữ là `TW`/`CN`; Google gọi **chính bản đó** là `NotoSerifTC`/`NotoSerifSC`. Nghĩa là `NotoSerifTC` (45,1 MB) **không phải** thứ tương đương `SourceHanSerifTC` (132,4 MB) — nó tương đương `SourceHanSerifTW`. Nhầm chỗ này là rơi thẳng vào bẫy ở mục dưới, chỉ khác đường vào.

**Hai lợi thế có thật của kênh Google:**

1. **Noto Serif CJK không khai Reserved Font Name.** Tệp `LICENSE` của `notofonts/noto-cjk` chỉ có bản văn OFL 1.1 trần, **không có dòng bản quyền khai tên dự trữ**; bản Adobe khai rõ *"with Reserved Font Name 'Source'"*. Hệ quả: **subset bản Noto không bắt buộc đổi tên font**, nên đòn bẩy giảm dung lượng ở §Đòn bẩy rẻ đi hẳn. *(Bản `Source Serif 4` trên Google Fonts cũng không khai RFN — ghi *"Copyright 2014 The Source Serif 4 Project Authors"*; riêng `Source Sans 3` trên Google Fonts **vẫn khai** RFN `'Source'`. Ba tệp, ba tình trạng khác nhau — phải mở từng tệp mà đọc, đừng suy từ tệp này sang tệp kia.)*
2. **Font biến thiên nhỏ hơn hẳn ở phần Latin** — xem bảng ở §Số liệu đã tra.

**Ba ràng buộc kèm theo, không được bỏ:**

- ⛔ **Cấm dùng `fonts.googleapis.com`.** AD-15 cấm mọi origin từ xa; đổi kênh lấy font là đổi chỗ **tải về**, không đổi luật **đóng gói**. Tệp vẫn phải nằm trong bản cài.
- `Noto Serif CJK` **không phục vụ qua Google Fonts API** (quá lớn) — chỉ tải được từ release `notofonts/noto-cjk`.
- **Font biến thiên không giúp cho CJK.** `02_NotoSerifCJK-OTF-VF.zip` nặng 214,5 MB. Lợi thế VF chỉ có ở phần Latin; phần Hán vẫn lấy tĩnh chỉ-Regular.
- Chọn kênh Google thì **tên họ font đổi** (`Noto Serif CJK TC` thay `Source Han Serif`) → phải sửa chuỗi trong bảng token và trong `families.read` / `families.read-cjk` của `DESIGN.md`. Sửa tài liệu, không đổi thiết kế.

### Bẫy chọn tệp — biến thể vùng ≠ subset theo ngôn ngữ

Adobe phát hành **hai loại tệp khác nhau** mà tên gọi rất dễ lẫn:

| Loại | Ví dụ | Phủ mã | Dùng được cho AuraTranslate? |
|---|---|---|---|
| **Biến thể vùng đầy đủ** | `SourceHanSerifTC` · `SourceHanSerifSC` | **Toàn bộ** kho mã CJK; chỉ khác **dáng chữ ưu tiên** ở mã dùng chung | ✅ **Đây là thứ AC3 nói tới** |
| **Subset theo ngôn ngữ** | `SourceHanSerifTW` · `SourceHanSerifCN` | Chỉ bộ mã của một vùng (Adobe-CNS1 / Adobe-GB1) | ⚠️ Chỉ là **đòn bẩy dung lượng**, không phải lựa chọn mặc định |

> **Vì sao phân biệt này quan trọng đến mức phải viết ra.** AC3 mô tả đúng loại thứ nhất: *"cùng một đoạn văn bản chứa các mã Hán **dùng chung**"* — chỉ có biến thể vùng đầy đủ mới có khái niệm "mã dùng chung vẽ khác nhau". Chọn nhầm loại thứ hai thì phép so sánh trở nên vô nghĩa, **và** sản phẩm sẽ hiện ô vuông rỗng khi người dùng nhập một văn bản phồn thể vào bản cài dùng subset giản thể — hỏng **im lặng**, đúng ở chỗ không ai kiểm, vì phần lớn ký tự vẫn hiện bình thường. Người dùng mục tiêu dịch **cả** truyện mạng đương đại (giản thể) **lẫn** cổ văn (phồn thể), nên phủ mã đầy đủ là yêu cầu, không phải tuỳ chọn.

### Bẫy đo lường — bốn chỗ số đo dễ sai

1. **Chênh lệch installer ≠ dung lượng trên đĩa.** `.dmg` và `.msi` đều nén nội dung. Một tệp OTF 19 MB có thể chỉ làm installer phình 12–15 MB. **Ghi cả hai con số** — số trên đĩa để suy luận, số chênh lệch installer để nghiệm thu AC1.

2. **Phép tính của AC1 trộn hai đơn vị.** AC1 nói *"chênh lệch dung lượng + 130 MB database"*, nhưng 130 MB là dung lượng SQLite **trên đĩa** (đo ở Giai đoạn 0), còn chênh lệch font là **sau nén**. Cứ thực hiện đúng nguyên văn AC1 làm phép nghiệm thu, nhưng **ghi chú rõ điều này trong báo cáo** để phép tính lại được khi database thật xuất hiện ở Story 1.9.

3. **WebView2 chi phối con số tuyệt đối trên Windows.** Tauri cho ba chế độ cài WebView2; `embedBootstrapper` / `offlineInstaller` làm `.msi` phình **~150 MB** vì nhúng luôn bản sao Chromium. Chế độ này **triệt tiêu trong phép trừ** (cả hai bản build đều có), nhưng nó quyết định liệu tổng thật có lọt trần NFR6 hay không. Dùng mặc định `downloadBootstrapper` và **ghi lại chế độ đã dùng**.

4. **Bốn bản build phải chỉ khác nhau đúng một biến.** Cùng commit, cùng chế độ release, cùng phiên bản toolchain. Khác một biến thứ hai thì chênh lệch đo được không quy về font nữa.

### Rà giấy phép — đã tra sẵn, việc còn lại là xác minh và ghi

Xác minh qua GitHub License API ngày 2026-08-03, cả ba repo:

| Họ font | SPDX | Dòng bản quyền |
|---|---|---|
| `source-serif` | `OFL-1.1` | *Copyright 2014–2023 Adobe, with **Reserved Font Name 'Source'*** |
| `source-sans` | `OFL-1.1` | *Copyright 2010–2024 Adobe, with **Reserved Font Name 'Source'*** |
| `source-han-serif` | OFL 1.1 *(GitHub gắn nhãn `NOASSERTION`; văn bản `LICENSE.txt` nói rõ OFL 1.1)* | *Copyright 2017–2022 Adobe, with **Reserved Font Name 'Source'*** |

**Kết luận tương thích GPL v3 — dự kiến ĐẠT**, với ba ràng buộc phải ghi kèm chứ không được lược:

1. **Đóng gói cùng phần mềm GPL là hợp lệ.** OFL-FAQ: *"Fonts licensed under the OFL can be freely included alongside other software under FLOSS licenses."* Quan hệ là **gộp gói (aggregation)**, không phải tác phẩm phái sinh — font không liên kết vào mã, chỉ nằm cạnh. FSF xếp OFL là giấy phép tự do và chấp nhận đóng gói cùng GPLv2/v3/LGPL/AGPL.
2. **Reserved Font Name — ràng buộc có răng thật, nhưng phụ thuộc kênh phát hành.** Bản Adobe của cả ba font giữ tên dự trữ `'Source'`. **Subset hoặc sửa font là tạo bản sửa đổi** → bắt buộc đổi tên nội bộ của font, kéo theo mọi khai báo `font-family: "Source Serif 4"` trong bảng token `DESIGN.md` phải đổi theo. **Bản Noto Serif CJK và bản Source Serif 4 trên Google Fonts không khai RFN** (xem §Hai kênh phát hành) — chọn kênh nào là quyết định có hệ quả giấy phép thật, phải ghi rõ trong báo cáo chứ không coi là chi tiết kỹ thuật.
3. **Bản văn giấy phép đi kèm.** Miễn trừ *"font nhúng trong chương trình"* của OFL có tồn tại, nhưng dự án này còn có màn hình Attribution (FR109) và ghi công trong bản phát hành (FR38) — cứ mang theo `LICENSE.txt` gốc của cả ba họ font. Rẻ, và bịt luôn câu hỏi ở Story 10.4/10.5.

> ⚠️ **Vẫn phải mở tệp `LICENSE` trong bản release đã tải và đọc.** Nhãn của GitHub là dẫn xuất, không phải nguồn sự thật — và riêng `source-han-serif` GitHub đã gắn `NOASSERTION`. NFR15 đòi *rà soát tường minh*; chép lại kết luận của người khác không phải là rà soát.

### Đòn bẩy nếu vượt trần — liệt kê để Ice quyết, KHÔNG tự áp dụng

| Đòn bẩy | Được | Mất |
|---|---|---|
| Dùng subset theo ngôn ngữ (`TW`/`CN`) | ~10–12 MB | Mất phủ mã vùng kia → tofu im lặng khi nhập văn bản khác hệ chữ |
| Bỏ nét SemiBold/Bold, mô phỏng bằng synthetic bold | vài MB | Nét giả xấu ở cỡ chữ nhỏ; lệch khỏi bảng token `DESIGN.md` |
| Chuyển OTF → WOFF2 | 20–40% cho phần Latin | Ít tác dụng sau khi installer đã nén; CJK vốn đã nén CFF |
| Lấy phần Latin từ kênh Google (font biến thiên) | ~2–3 MB, **và** phủ trọn mọi nét | Đổi định dạng TTF thay OTF; phải kiểm lại hình chữ ở cỡ nhỏ |
| Tự subset theo kho mã thật sự cần | nhiều nhất | Phải dựng và bảo trì pipeline subset; mất khả năng hiện mã hiếm trong cổ văn. **Đổi tên font chỉ bắt buộc nếu dùng bản Adobe** — bản Noto không khai Reserved Font Name |
| Nới trần NFR6 | không mất gì kỹ thuật | Đổi lời hứa sản phẩm — **đúng loại quyết định AC4 nói tới** |

### Chi tiết kỹ thuật Tauri cần cho Task 1 và Task 6

- **Nhúng font làm resource:** khai trong `tauri.conf.json` → `bundle.resources`, ánh xạ nguồn → đích. Kiến trúc đã chốt vị trí là `src-tauri/resources/fonts/` với scope `$RESOURCE/fonts/**` chỉ đọc (AD-23) — app thăm dò nên đi theo đúng đường này để số đo dùng lại được.
- **CSP:** mặc định của Tauri v2 có `default-src 'self' customprotocol: asset:`. Để `@font-face` nạp được font qua asset protocol, **`font-src` phải chứa `asset:`** (và `http://asset.localhost` trên Windows). Đây **không phải** nới CSP theo nghĩa AD-15 cấm — AD-15 cấm **origin từ xa** (CDN, font ngoài, ảnh ngoài); asset protocol là tài nguyên cục bộ đã đóng gói. Ghi lại cấu hình CSP đã dùng để Story 1.2 chép lại đúng.
- **Đường thay thế đơn giản hơn cho app thăm dò:** đặt font trong `src/assets/` và để Vite xử lý `url()` — Tauri sẽ đóng gói theo. Rẻ hơn cho việc đo, nhưng **khác đường đi thật**, nên nếu chọn cách này thì phải ghi rõ trong báo cáo rằng chênh lệch đo được là ở tầng frontend bundle, không ở tầng resource.
- **Lệnh build:** `tauri build --bundles dmg` trên macOS, `tauri build --bundles msi` trên Windows. AC1 nói `.msi` — dùng WiX v3, không dùng NSIS, để số đo khớp đúng thứ AC hỏi.

### Tiền lệ cần theo — `phase-0-spike-results-2026-08-02.md`

Giai đoạn 0 đã thiết lập khuôn mà mũi thăm dò này phải theo, và nó là khuôn tốt:

- **Frontmatter** có `title` · `status` · `created` · `updated` · `relates_to`.
- **Bảng tóm tắt ngay đầu** với dấu 🟢/🔴 cho từng phép đo — người đọc thấy kết luận trước khi thấy số.
- **Ghi rõ môi trường đo** (hệ điều hành, phiên bản toolchain) — nếu không, số đo không lặp lại được.
- **Ghi cả phỏng đoán bị bác**, không chỉ ghi kết quả. Giai đoạn 0 bác được hai phỏng đoán và phát hiện hai cái bẫy mới chính nhờ thói quen này.
- **Mục "Việc chưa làm được ở giai đoạn này"** — nêu thẳng giới hạn thay vì để người đọc tự phát hiện.
- **Mục "Cần Ice quyết"** tách riêng, có bảng phương án kèm *Được / Mất*.

### Testing standards

Story này không sinh mã sản phẩm nên **không có unit test**. Thứ thay thế cho test ở một mũi thăm dò là **tính lặp lại được**:

- Ghi lại **lệnh chính xác** đã chạy cho từng bản build, đủ để người khác chạy lại ra cùng số.
- Ghi lại **phiên bản toolchain** (Rust, Node, `tauri-cli`, hệ điều hành) — đúng như Giai đoạn 0 đã làm.
- Ghi lại **checksum hoặc tên tệp chính xác** của các asset font đã tải, để không ai lẫn giữa `TC` và `TW`.
- Chụp màn hình so sánh TC/SC và lưu kèm báo cáo — AC3 đòi *lý do được ghi lại*, và với một quyết định về dáng chữ thì bằng chứng thị giác chính là lý do.

### Project Structure Notes

- Tài liệu kết quả đi vào `_bmad-output/planning-artifacts/research/` — **cùng chỗ** với `phase-0-spike-results-2026-08-02.md`, không tạo thư mục mới.
- Bốn tệp quy hoạch bị sửa (`ARCHITECTURE-SPINE.md`, `DESIGN.md`, `EXPERIENCE.md`, `.memlog.md`) đều đã có **khuôn sửa sẵn** từ các lần đóng trước (hàng HVTĐTD, hàng FR115, mục "Còn thiếu") — theo đúng khuôn đó, đừng phát minh khuôn mới.
- Không tạo `src-tauri/`, không tạo `package.json`, không tạo `Cargo.toml` trong repo. Story 1.2 sở hữu việc đó.
- Không đụng `dict-manifest.toml` — chưa tồn tại, thuộc Story 1.9/10.1.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.1`] — AC nguyên văn, phạm vi, mệnh đề chặn Epic 1
- [Source: `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md#7.2`] — NFR6: ngân sách 150–200 MB, không tải thêm sau cài `[A2]`
- [Source: `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md#7.4`] — NFR15: mọi thư viện phải tương thích GPL v3, rà tường minh trước khi thêm
- [Source: `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md#Giả định A2`] — 150–200 MB chưa gồm Unihan, Thiều Chửu, Cổ hán văn, VietPhrase
- [Source: `ARCHITECTURE-SPINE.md#Stack`] — bảng phải bổ sung ba hàng font; quy ước ghi phiên bản và giấy phép
- [Source: `ARCHITECTURE-SPINE.md#Deferred`] — hai hàng phải đóng: *Dung lượng và giấy phép font nhúng* (điều kiện: mũi thăm dò trước Giai đoạn 1) · *Biến thể vùng cho Source Han Serif*
- [Source: `ARCHITECTURE-SPINE.md#AD-15`] — CSP giữ nguyên, cấm origin từ xa: không CDN, không font ngoài, không ảnh ngoài
- [Source: `ARCHITECTURE-SPINE.md#AD-23`] — scope tĩnh `$RESOURCE/fonts/**` chỉ đọc, dành sẵn cho font nhúng
- [Source: `ARCHITECTURE-SPINE.md#Consistency Conventions`] — *"Mỗi phụ thuộc mới phải rà tương thích GPLv3 trước khi thêm vào (NFR15) và ghi vào bảng Stack"*
- [Source: `DESIGN.md` frontmatter `fonts-bundled`] — ba trường phải cập nhật: `license` · `size-budget` · `region-variant`
- [Source: `DESIGN.md#Typography`] — bốn họ chữ; *"Ba việc chưa xong"*; lý do nhúng font thay vì dùng font hệ điều hành (NFR14)
- [Source: `DESIGN.md#Bảng token typography`] — 14 token, xác định đúng những nét cần đóng gói
- [Source: `EXPERIENCE.md#Còn thiếu`] — mục *"Font thật chưa đo"* phải đóng
- [Source: `_bmad-output/planning-artifacts/research/phase-0-spike-results-2026-08-02.md`] — khuôn báo cáo mũi thăm dò; quy ước mã dùng một lần không vào repo; số 130 MB
- [Source: `_bmad-output/planning-artifacts/implementation-readiness-report-2026-08-03.md:769,1090`] — Story 1.1 là đường đóng cho khoảng trống font; AC tự mang mệnh đề chặn
- [Web 2026-08-03] GitHub Releases API `adobe-fonts/source-han-serif` 2.003R · `adobe-fonts/source-serif` 4.005R · `adobe-fonts/source-sans` 3.052R — dung lượng asset
- [Web 2026-08-03] GitHub License API ba repo Adobe — OFL-1.1, Reserved Font Name `'Source'`
- [Web 2026-08-03] OFL-FAQ (openfontlicense.org) — đóng gói cùng phần mềm FLOSS; đổi tên khi sửa/subset; khi nào được lược bản văn giấy phép
- [Web 2026-08-03] `notofonts/noto-cjk` release `Serif2.003` + tệp `Serif/LICENSE` — Noto Serif CJK là Source Han Serif đổi tên (asset trùng dung lượng, cùng ngày phát hành); **LICENSE không khai Reserved Font Name**
- [Web 2026-08-03] `google/fonts` `ofl/sourceserif4` · `ofl/sourcesans3` — font biến thiên, dung lượng thật; `OFL.txt` của `sourceserif4` không khai RFN, của `sourcesans3` có khai
- [Web 2026-08-03] Google Open Source Blog / Adobe CCJK Type blog (2017-04-03) — Source Han Serif và Noto Serif CJK là cùng một dự án, hai nhãn
- [Web 2026-08-03] Tài liệu Tauri v2 — `bundle.resources`; CSP `asset:`; WebView2 `downloadBootstrapper` vs `embedBootstrapper` (~150 MB)

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
