---
baseline_commit: 754f0f9a1a4f1da5b297cdbfa20bc9596a304139
---

# Story 1.1: Mũi thăm dò font — đo dung lượng thật và rà giấy phép

Status: done

Epic: 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
Covers: NFR6 · NFR15 · ~~mũi thăm dò bắt buộc — chặn mọi story khác của Epic 1~~ → ✅ **mệnh đề chặn đã gỡ 2026-08-03**

> **Loại story: MŨI THĂM DÒ (spike), không phải story cài đặt tính năng.**
> Sản phẩm bàn giao là **số đo thật + quyết định đã ghi vào tài liệu**, không phải mã nguồn sản phẩm.
> Không dòng mã nào của story này được đi vào cây nguồn thật. Xem §Ranh giới phạm vi.

## Story

As a **chủ dự án**,
I want **biết chắc bộ font nhúng nằm trong ngân sách NFR6 và giấy phép cho phép phân phối lại**,
So that **tôi không phải bóc font ra sau khi đã dựng nửa giao diện trên nó**.

## Acceptance Criteria

### AC1 — Đo dung lượng thật *(đã thu hẹp 2026-08-03)*

> ⚠️ **AC này đã thu hẹp trong lúc chạy story — Ice quyết 2026-08-03.** Bản gốc đòi đo **cả `.dmg` lẫn `.msi`**. `tauri-cli` 2.11.4 trên macOS từ chối target `msi` (*"possible values: ios, app, dmg"*) vì `.msi` dựng bằng WiX v3, mà `candle`/`light` là chương trình Windows — rào ở tầng **đóng gói**, không phải tầng biên dịch Rust. **Hai phép đo `.msi` chuyển sang Story 1.3**, đã ghi thành AC mới trong `epics.md`. Bản gốc của AC1 cũng giữ ở `epics.md` dưới dạng gạch ngang.
>
> **Vì sao chấp nhận đo muộn:** ước 16,0–20,3 MiB nằm gọn trong trần, và phương pháp ước đã tự kiểm sai số 0,1 % trên chính phép đo macOS; rủi ro Windows thật sự nằm ở **chế độ cài WebView2** chứ không ở font, mà thứ đó chỉ CI mới bắt được.

**Given** ba họ font đã chốt — `Source Serif 4`, `Source Han Serif` (chỉ Regular), `Source Sans 3`
**When** đóng gói thử một `.dmg` có nhúng font và một `.dmg` không nhúng
**Then** chênh lệch dung lượng được ghi lại thành số cụ thể
**And** con số đó cộng với 130 MB database đã đo phải nằm trong trần 150–200 MB của NFR6
**And** phép đo tương ứng cho `.msi` được **bàn giao sang Story 1.3** kèm công thức chạy, chứ không bị bỏ im lặng

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

- [x] **Task 1 — Dựng app thăm dò dùng một lần** (AC: 1, 3)
  - [x] Tạo app Tauri v2 tối thiểu **trong thư mục scratchpad của phiên, KHÔNG trong repo AuraTranslate**. `create-tauri-app` được phép ở đây và **chỉ** ở đây.
  - [x] App chỉ cần một cửa sổ hiển thị một đoạn văn bản thử — không cần panel, không cần IPC, không cần database.
  - [x] Ghim `tauri` 2.11.5 · Vue 3.5.40 · Vite 8.2.0 đúng bảng Stack, để số đo dùng lại được cho bản thật. *(Crate `tauri` = 2.11.5 đúng như ghim. `@tauri-apps/cli` dùng **2.11.4** vì bản 2.11.5 của gói npm không tồn tại — hai kênh đánh số riêng, xem Debug Log.)*
  - [x] **Không** commit app này. **Không** dùng nó làm scaffold cho Story 1.2 — xem §Ranh giới phạm vi. *(Đã xác minh: `git status` không có `src-tauri/`, `package.json`, `Cargo.toml`.)*

- [x] **Task 2 — Tải đúng tệp font từ kênh Google** (AC: 1, 3)
  - [x] **Kênh đã chốt: Google cho cả ba họ font** (Ice quyết 2026-08-03). Không phải việc của mũi thăm dò nữa — xem §Hai kênh phát hành để biết lý do và các ràng buộc kèm theo.
  - [x] Latin — tải từ `google/fonts`, **font biến thiên**: `ofl/sourceserif4/SourceSerif4[opsz,wght].ttf` · `ofl/sourceserif4/SourceSerif4-Italic[opsz,wght].ttf` · `ofl/sourcesans3/SourceSans3[wght].ttf`. **Ba tệp, phủ trọn dải nét 200–900** — không cần tệp riêng cho nét 600 và 700. *(Dựng thật xác nhận nét 600 và 700 là nét thật.)*
  - [x] CJK — tải từ release `notofonts/noto-cjk` tag **`Serif2.003`**, lấy **chỉ nét Regular** của biến thể vùng đầy đủ (`09_NotoSerifCJKsc.zip` hoặc `10_NotoSerifCJKtc.zip`, chốt ở Task 6). *(Tải cả hai để so ở Task 6; chỉ TC vào bộ phát hành.)*
  - [x] **Không** tải qua `fonts.googleapis.com`. Tải tệp về đóng gói; AD-15 cấm mọi origin từ xa lúc chạy.
  - [x] Ghi lại dung lượng **trên đĩa** của từng tệp trước khi đóng gói (số này khác với chênh lệch installer — xem §Bẫy đo lường). *(Kèm SHA-256 từng tệp.)*
  - [x] Ghi nhận trong báo cáo: `Noto Serif CJK` chỉ có Regular nên chữ Hán rơi vào token nét đậm hoặc nghiêng sẽ bị **tổng hợp giả**. Bảng token hiện không có token nào như vậy nên ca này chưa phát sinh — chỉ cần xác nhận lại là đúng, **không** tự ý thêm nét. *(Đã soát bảng 14 token, xác nhận đúng. Không thêm nét nào.)*

- [x] **Task 3 — Đo hai bản build macOS** (AC: 1) — *(phạm vi thu hẹp cùng AC1, Ice quyết 2026-08-03)*
  - [x] macOS: build `.dmg` **không** font → ghi số. Build `.dmg` **có** font → ghi số. *(1.402.311 B → 22.688.024 B; chênh lệch 21.285.713 B = 20,300 MiB.)*
  - [x] ~~Windows: build `.msi` **không** font → ghi số. Build `.msi` **có** font → ghi số.~~ — **CHUYỂN SANG STORY 1.3** (Ice quyết 2026-08-03). **Không đo ở story này.** Lý do: `tauri-cli` 2.11.4 trên macOS từ chối target `msi` — *"invalid value 'msi' for '--bundles', possible values: ios, app, dmg"*; `.msi` dựng bằng WiX v3 mà `candle`/`light` là chương trình Windows, nên rào ở tầng **đóng gói** chứ không ở tầng biên dịch Rust (target `x86_64-pc-windows-msvc` đã cài sẵn). Đã ghi thành **AC mới của Story 1.3** trong `epics.md`, kèm công thức chạy ở §Công thức đo trên Windows của báo cáo. **Bàn giao tường minh, không bỏ im lặng.**
  - [~] ~~Bốn bản build~~ → **hai bản đo** phải giống hệt nhau ở mọi thứ khác (cùng commit, cùng cấu hình, cùng chế độ release) — chỉ khác `bundle.resources`. *(Giữ đúng cho hai bản A/B đã chạy: bản có font chỉ chồng thêm một tệp `--config` chứa đúng một khoá, và cả hai mang **bốn** tệp font.)* ⚠️ **Sửa 2026-08-03 sau rà soát:** thực tế có **ba** bản build, không phải hai — phép so TC/SC ở AC3 cần thêm một bản mang **năm** tệp font, và **ảnh chụp đến từ bản thứ ba đó**. Bản đầu của báo cáo không ghi ra, khiến §Công thức đo trên Windows mô tả thiếu một bản. Đã sửa ở §Phép đo 1.
  - [~] Ghi lại **chế độ cài WebView2 đã dùng** trên Windows — **KHÔNG ÁP DỤNG ở story này**, vì không có bản build Windows nào để "đã dùng". Thứ đã làm là **khai** `downloadBootstrapper` tường minh trong `tauri.conf.json` của app thăm dò và chuyển cảnh báo sang Story 1.3, nơi nó thành một vế của AC. *(Sửa 2026-08-03: bản đầu tick `[x]` cho một việc chỉ có thể làm trên Windows.)*
  - [x] Ghi lại **dung lượng baseline** (bản không font) chứ không chỉ chênh lệch — cần cho phép tính tổng thật. *(1,337 MiB.)*

- [x] **Task 4 — Tính tổng và đối chiếu NFR6** (AC: 1, 4)
  - [x] Tính: `chênh lệch font (từng nền tảng) + 130 MB database` — đây là **phép tính nghiệm thu đúng nguyên văn AC1**. *(macOS: 21,29 + 130 = **151,29 MB**. Windows: số thật chưa có; ước **16,0–20,3 MiB** bằng phương pháp nén-thay-thế đã tự hiệu chuẩn trên chính phép đo macOS, sai số 0,1%.)*
  - [x] Ghi thêm phép tính đầy đủ hơn để Ice có bức tranh thật: `baseline + chênh lệch font + database`, kèm ghi chú rằng 130 MB là số **trên đĩa** với **ba nguồn đầu tiên**, còn ước tính đủ nguồn là 150–200 MB. *(151,64 MiB / 152,69 MB, kèm ghi chú rằng database thật cũng sẽ được nén nên số cuối sẽ thấp hơn.)*
  - [x] Đối chiếu với trần NFR6 = **150–200 MB**. *(🟢 ĐẠT trên macOS, nhưng ở **mép dưới**. Rủi ro còn mở đã ghi vào §Cần Ice quyết.)*

- [x] **Task 5 — Rà giấy phép theo NFR15** (AC: 2)
  - [x] Xác nhận cả ba họ font là **SIL OFL 1.1** bằng chính tệp `LICENSE` trong release đã tải (không tin trang web thứ ba). *(Mở và đọc từng tệp; SHA-256 ghi trong báo cáo.)*
  - [x] Ghi kết luận tương thích GPL v3, có nêu **ba ràng buộc kèm theo** của OFL: Reserved Font Name, đi kèm bản văn giấy phép, cấm bán riêng font. Chi tiết ở §Rà giấy phép.
  - [x] Ghi rõ **tình trạng RFN khác nhau giữa ba tệp**: `Noto Serif CJK` không khai · `Source Serif 4` bản Google Fonts không khai · `Source Sans 3` **có** khai `'Source'`. Mở từng tệp mà đọc, đừng suy từ tệp này sang tệp kia. *(Cả ba khớp dự đoán của Dev Notes. `LICENSE` của Noto Serif CJK không có cả dòng bản quyền.)*
  - [x] Thêm **ba hàng font** vào bảng Stack của `ARCHITECTURE-SPINE.md` — cột `Name` · `Version` · `Giấy phép`, đúng khuôn các hàng crate đang có. Ghi tên theo **kênh Google** (`Noto Serif CJK`, không phải `Source Han Serif`). *(Phiên bản đọc từ bảng `name` của tệp: Source Serif 4 là **4.004**, không phải 4.005R.)*
  - [x] Nếu kết luận là **không tương thích**: dừng lại và viết **đề xuất font thay thế** (ứng viên + lý do + hệ quả cho `DESIGN.md`). Không được kết thúc bằng "bỏ qua". *(Không kích hoạt — kết luận là **tương thích**.)*

- [x] **Task 6 — Chọn biến thể vùng TC hay SC** (AC: 3)
  - [x] Dựng **một đoạn văn bản thử duy nhất** chứa các mã Hán mà TC và SC vẽ khác nhau (ví dụ: 骨 · 直 · 房 · 令 · 兪 · 者 · 雇 — các mã có glyph vùng khác nhau trong Source Han). *(Dùng đúng bảy mã đó + 戶 · 產 · 為.)*
  - [x] Render đoạn đó **hai lần** trong app thăm dò, một lần với biến thể TC, một lần với SC. Chụp màn hình cả hai. *(Dựng cạnh nhau trong cùng một cửa sổ — bốn ảnh trong `research/font-spike-2026-08-03/`.)*
  - [x] Chọn một, ghi lý do. Bối cảnh đã có ở `DESIGN.md`: *TC hợp mạch cổ văn và Hán Việt; SC hợp truyện mạng đương đại.* *(Chốt **TC**. Lý do nặng nhất hoá ra là **vị trí dấu câu**, không phải dáng chữ.)*
  - [x] ⚠️ Dùng **biến thể vùng đầy đủ** — `SourceHanSerifTC` / `SourceHanSerifSC` (Adobe) hoặc `NotoSerifCJKtc` / `NotoSerifCJKsc` (Google). **Không** dùng bản subset theo ngôn ngữ: `SourceHanSerifTW`/`CN`, tức `NotoSerifTC`/`NotoSerifSC` — xem §Bẫy chọn tệp và cảnh báo tên gọi chéo kênh ở §Hai kênh phát hành. *(Đã kiểm dung lượng asset trên release API để chắc lấy đúng cặp.)*

- [x] **Task 7 — Ghi kết quả vào tài liệu** (AC: 1, 2, 3, 4)
  - [x] Viết `_bmad-output/planning-artifacts/research/font-spike-results-2026-08-03.md` theo **đúng khuôn** `phase-0-spike-results-2026-08-02.md` (frontmatter · bảng tóm tắt · từng phép đo · kết luận · việc cần Ice quyết).
  - [x] `ARCHITECTURE-SPINE.md` → bảng **Stack**: thêm ba hàng font (AC2).
  - [x] `ARCHITECTURE-SPINE.md` → bảng **Deferred**: đóng hai hàng *"Dung lượng và giấy phép font nhúng"* và *"Biến thể vùng cho Source Han Serif"*, dùng đúng khuôn gạch ngang + `✅ Đã đóng 2026-08-03` như hàng HVTĐTD và hàng FR115 đã dùng.
  - [x] `DESIGN.md` → frontmatter `fonts-bundled`: thay `license: 'SIL OFL — CẦN RÀ TƯỜNG MINH…'`, `size-budget: 'CHƯA ĐO…'`, `region-variant: 'CHƯA CHỐT…'` bằng kết quả thật.
  - [x] `DESIGN.md` → §Typography, mục *"Ba việc chưa xong"*: đánh dấu cả ba đã xong, giữ nguyên lý do.
  - [x] `EXPERIENCE.md` → §Còn thiếu, gạch mục *"Font thật chưa đo"* theo khuôn các mục đã đóng.
  - [x] `.memlog.md` của architecture: thêm một dòng `(version)` ghi số đo và một dòng `(decision)` ghi biến thể vùng đã chọn.
  - [x] `.memlog.md` của ux-designs: thêm một dòng ghi biến thể vùng đã chốt và dung lượng thật, vì `DESIGN.md` và `EXPERIENCE.md` cùng thư mục đều đổi theo.

- [x] **Task 8 — Nếu vượt trần NFR6** (AC: 4)
  - [x] **Không** tự ý subset font, **không** tự ý bỏ một họ font, **không** tự ý đổi sang font hệ điều hành. Cả ba đều là quyết định tầng PRD/thiết kế. *(Giữ đúng — không đụng vào bất kỳ thứ nào trong ba.)*
  - [x] Viết mục *"Cần Ice quyết"* nêu các đòn bẩy có thật kèm cái giá của từng đòn bẩy (§Đòn bẩy nếu vượt trần). *(Đã viết — nhưng cho **rủi ro tương lai** chứ không cho ca vượt trần hôm nay, vì hôm nay không vượt.)*
  - [~] Nêu rõ trong báo cáo: **Epic 1 dừng ở đây** cho tới khi Ice quyết — **KHÔNG ÁP DỤNG**. Điều kiện kích hoạt của Task 8 là *"nếu vượt trần NFR6"*, và nó **không xảy ra**: AC1 đạt ở 151,29 MB. Báo cáo vì thế ghi điều **ngược lại** — §Mệnh đề chặn Epic 1 — đã gỡ — kèm hai việc mang theo không chặn. *(Sửa 2026-08-03: bản đầu tick `[x]` cho một việc mà artifact ghi ngược lại nội dung subtask.)*

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

- **Cấm dùng `fonts.googleapis.com`.** AD-15 cấm mọi origin từ xa; đổi kênh lấy font là đổi chỗ **tải về**, không đổi luật **đóng gói**. Tệp vẫn phải nằm trong bản cài.
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

`claude-opus-5` (Claude Code, workflow `bmad-dev-story`) — 2026-08-03

### Debug Log References

Số liệu thô, lệnh chính xác và SHA-256 từng tệp: [`research/font-spike-results-2026-08-03.md`](../planning-artifacts/research/font-spike-results-2026-08-03.md).

Bốn chỗ vấp, ghi lại vì Story 1.2 và 1.3 sẽ gặp lại:

1. **`bundle_dmg.sh` chết ở bước AppleScript** khi không có phiên Finder tương tác — `Finder got an error: Can't set Finder window id … to 128. (-10006)`, rồi `exit 64`. Bước đó chỉ trang trí vị trí icon trong cửa sổ `.dmg`. Đặt `CI=true` là Tauri truyền `--skip-jenkins` xuống và bỏ hẳn bước này. **Runner GitHub Actions của Story 1.3 sẽ gặp đúng lỗi này.**
2. **Khai `[lib]` trong `Cargo.toml` mà không có `src/lib.rs`** → `cargo metadata` gãy, Tauri CLI dừng trước cả khi biên dịch: *"can't find library `font_spike_lib`"*. App một tệp `main.rs` thì bỏ hẳn khối `[lib]`.
3. **Số crate `tauri` và số CLI npm không đi cùng nhau.** Crate có 2.11.5; `@tauri-apps/cli` mới nhất là **2.11.4**. Bảng Stack ghim 2.11.5 là ghim **crate** — đúng. `npm i @tauri-apps/cli@2.11.5` sẽ 404.
4. **Bật `assetProtocol` thì `Cargo.toml` bắt buộc có feature `tauri/protocol-asset`** — `tauri-build` tự thêm vào và báo lỗi nếu thiếu.

### Completion Notes List

**Bàn giao chính:** [`_bmad-output/planning-artifacts/research/font-spike-results-2026-08-03.md`](../planning-artifacts/research/font-spike-results-2026-08-03.md) + bốn ảnh chụp trong `research/font-spike-2026-08-03/`. Không một dòng mã nào vào repo — đã xác minh bằng `git status`: không có `src-tauri/`, `package.json`, `Cargo.toml`.

**Kết quả từng AC:**

| AC | Trạng thái | Số / quyết định |
|---|---|---|
| **AC1** — đo dung lượng thật *(đã thu hẹp 2026-08-03)* | ✅ **Đạt theo phạm vi đã thu hẹp** | Chênh lệch `.dmg` **20,300 MiB**; tổng với database 130 MB = **151,29 MB**, **lọt trần NFR6** ở mép dưới. `.msi` bàn giao sang Story 1.3 kèm công thức — xem mục bàn giao dưới |
| **AC2** — rà giấy phép theo NFR15 | ✅ **Đạt** | **SIL OFL 1.1** cả ba, tương thích **GPL v3** theo diện gộp gói. Ba hàng font đã vào bảng Stack. Xác minh bằng chính tệp `LICENSE` trong release đã tải |
| **AC3** — chọn biến thể vùng | ✅ **Đạt** | Chốt **`Noto Serif CJK TC`**, lý do ghi kèm bốn ảnh chụp |
| **AC4** — vượt trần là quyết định tầng PRD | ⚪ **Không áp dụng** *(hành vi đã giữ đúng)* | Mệnh đề `Given` của AC4 là *"tổng dung lượng **vượt trần** NFR6"* — và nó **chưa bao giờ kích hoạt**: AC1 đạt ở 151,29 MB. Hành vi mà AC4 đòi vẫn được giữ đúng — không tự subset, không bỏ họ font, không đổi sang font hệ điều hành — và rủi ro dư địa đã viết thành mục *Cần Ice quyết* có bảng Được/Mất. Nhưng một AC có tiền đề không xảy ra thì **không thể "đạt"**; ghi ✅ làm bảng này nói rằng hành vi báo-cáo-lên-tầng-PRD đã được kiểm chứng, trong khi nó chưa. *(Sửa 2026-08-03 sau rà soát.)* |

**Ba phỏng đoán bị bác — ghi lại theo thói quen Giai đoạn 0:**

1. `DESIGN.md` ước bộ font ≈ 21,6 MB trên đĩa. Số thật **25,991 MiB**: phần CJK là **23,405 MiB** chứ không phải ≈19 MB. Phép chia zip 7 nét cho 7 giả định các nét bằng nhau, mà chúng không bằng. *(Ước 30–50 MB của bản trước nữa thì quá cao — cả hai ước đều trượt, theo hai hướng ngược nhau.)*
2. Tôi đoán `.dmg` của Tauri nén bằng **bzip2** (`UDBZ`). Thật ra là **`UDZO` — zlib/deflate**. Sai lầm này hoá ra có ích: nó là thứ cho phép **tự hiệu chuẩn** phép ước cho Windows — nén rời cùng payload bằng `zip -9` dự đoán tỉ lệ nén thật của `.dmg` lệch **0,1 %**.
3. `SourceSans3[wght].ttf` có mặc định trục `wght` = **200** và name ID 1 ghi `Source Sans 3 ExtraLight` — tôi tưởng mọi chữ giao diện không khai `font-weight` sẽ ra nét mảnh. **Bản dựng thật bác:** CSS luôn áp `font-weight: normal` (= 400) nên trục ghim ở 400. Mặc định 200 **vẫn** chạm tới thứ đọc font ngoài đường CSS — ghi để đường dựng `.docx` ở Epic 8 không vấp.

**Phát hiện ngoài dự kiến, đáng giá nhất:** khác biệt TC/SC **nặng nhất không phải dáng chữ** như mọi tài liệu bàn giao đều nói, mà là **vị trí dấu câu** — TC đặt 「，」「。」 giữa ô chữ, SC đặt góc dưới bên trái. Dáng chữ khác ở một nhúm mã; dấu câu khác ở **mọi câu**. Bài học đã ghi vào `EXPERIENCE.md`: so hai biến thể font thì phải dựng một **đoạn văn thật**, đừng chỉ so bảng ký tự.

**Lệch phiên bản cần biết:** `Source Serif 4` trên kênh Google là **4.004**, đi sau bản Adobe 4.005R một phát hành (commit cuối chạm tệp: 2021-11-17). Bảng Stack ghi 4.004 — đừng chép 4.005R từ Dev Notes.

---

#### Bàn giao sang Story 1.3 — hai phép đo `.msi`

**Ice quyết 2026-08-03: bỏ phép đo `.msi` ở story này, chuyển sang Story 1.3.** AC1 đã thu hẹp tương ứng ở **cả** `epics.md` **và** story file này, bản gốc giữ lại dưới dạng gạch ngang để đọc lại được. Story 1.3 nhận **một AC mới** trong `epics.md` mô tả đúng phép đo phải làm.

Rào chặn kỹ thuật, ghi lại nguyên văn:

```
error: invalid value 'msi' for '--bundles [<BUNDLES>...]'
  [possible values: ios, app, dmg]
```

`.msi` dựng bằng WiX v3, mà `candle`/`light` là chương trình Windows. Rào ở tầng **đóng gói**, không phải tầng biên dịch Rust — target `x86_64-pc-windows-msvc` đã cài sẵn.

**Ba lý do đo muộn là chấp nhận được:**

1. Ước **16,0–20,3 MiB** → tổng Windows ≈ **146,7–151,3 MB**, lọt trần với biên **rộng hơn** macOS. Phương pháp ước đã tự kiểm sai số **0,1 %** trên chính phép đo macOS (nén rời bằng `zip -9` dự đoán đúng tỉ lệ nén thật của `.dmg`, vốn là deflate).
2. Rủi ro Windows thật sự **không nằm ở font** mà ở **chế độ cài WebView2**: `embedBootstrapper` hay `offlineInstaller` làm `.msi` phình ~150 MB và vỡ NFR6 kể cả khi font bằng 0. Thứ đó chỉ CI bắt được, không phải phép đo một lần này.
3. Story 1.3 dựng CI hai nền tảng — chi phí thêm gần bằng **0**, và ghi lại **mỗi lần phát hành** nên còn bắt được cả hồi quy, thứ mà một phép đo một lần không làm được.

**Công thức chạy** đã chép vào [§Công thức đo trên Windows](../planning-artifacts/research/font-spike-results-2026-08-03.md) của báo cáo — đặt ở đó chứ không chỉ trong thư mục tạm, để nó **không mất khi scratchpad bị dọn**. Gói mang đi `font-spike-windows.zip` (SHA-256 `776e8d06dca6210fded432e7baec6505813571afabcf7c5435e09afda1b07af2`) vẫn còn trong scratchpad nếu Story 1.3 muốn dùng lại đúng bộ font đã đo, nhưng **không bắt buộc** — CI dựng từ cây nguồn thật.

**Epic 1 không bị chặn.** Mệnh đề chặn của AC4 là *"kết quả được ghi lại"*, và kết quả đã ghi vào **6 tệp quy hoạch** (`ARCHITECTURE-SPINE.md` · `.memlog.md` architecture · `DESIGN.md` · `EXPERIENCE.md` · `.memlog.md` ux · `epics.md`) **+ 1 báo cáo mới + 4 ảnh chụp**, ngoài ra story file và `sprint-status.yaml` là tài liệu triển khai chứ không phải tài liệu quy hoạch. *(Sửa 2026-08-03: ba chỗ trong chính tệp này từng ghi ba con số khác nhau — 7, 6 và 8.)*

**Việc thứ hai, không chặn:** dư địa dưới trần NFR6 mỏng hơn bản đầu tưởng. *(Bản đầu viết "200 MB database + 20,30 MiB font = 220,3 MB, vượt trần" — **đọc sai `[A2]`**: 150–200 MB là trần của **cả bản cài, đã gồm font**, không phải dự báo dung lượng database. Rà soát 2026-08-03 bắt được.)* Phép tính đúng là **trừ dư địa**: 200 − 21,29 (font) − 1,40 (baseline app **rỗng**, chưa có mã sản phẩm) − 130 (ba nguồn đầu) = **còn ~47 MB**, và 47 MB đó phải chứa các nguồn từ điển còn lại, chỉ mục FTS phụ (~17 MB), **và toàn bộ mã sản phẩm chưa viết**. Story 1.9 nay đã có **AC thật** cho phép đối chiếu này — trước đây nó chỉ là một câu khuyến nghị mà năm tài liệu chép lại như thể đã lên lịch.

### File List

Chỉ tài liệu quy hoạch — story này không sinh mã sản phẩm.

**Mới:**

- `_bmad-output/planning-artifacts/research/font-spike-results-2026-08-03.md`
- `_bmad-output/planning-artifacts/research/font-spike-2026-08-03/tc-vs-sc-glyphs.png`
- `_bmad-output/planning-artifacts/research/font-spike-2026-08-03/tc-vs-sc-paragraph-and-latin.png`
- `_bmad-output/planning-artifacts/research/font-spike-2026-08-03/zoom-glyphs-4-ma.png`
- `_bmad-output/planning-artifacts/research/font-spike-2026-08-03/zoom-dau-cau.png`

**Sửa:**

- `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md` — bảng Stack +3 hàng font; bảng Deferred đóng 2 hàng; `sources` +1
- `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/.memlog.md`
- `_bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/DESIGN.md` — `updated`; `families.read` / `families.read-cjk`; `fonts-bundled.license` / `.size-budget` / `.region-variant`; §Typography *"Ba việc chưa xong"*; ghi chú bốn họ chữ và ghi chú tổng hợp giả
- `_bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md` — §Còn thiếu, đóng mục *"Font thật chưa đo"*
- `_bmad-output/planning-artifacts/ux-designs/ux-AuraTranslate-2026-08-02/.memlog.md`
- `_bmad-output/planning-artifacts/epics.md` — **Story 1.1 AC1 thu hẹp** (bỏ `.msi`, bản gốc giữ dạng gạch ngang); **Story 1.3 nhận AC mới** đo `.msi`; ghi chú Epic 1 *"mũi thăm dò font phải chạy trước"* đánh dấu đã xong. ⚠️ Ngoài phạm vi sửa mặc định của workflow `dev-story` — **Ice chỉ đạo tường minh 2026-08-03**
- `_bmad-output/implementation-artifacts/1-1-mui-tham-do-font-do-dung-luong-that-va-ra-giay-phep.md` — story này *(gồm cả §AC1, sửa theo cùng chỉ đạo trên)*
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `ready-for-dev` → `in-progress` → `review`

**Không vào repo, đúng §Ranh giới phạm vi** — nằm trong scratchpad của phiên: `font-spike/app/` (app Tauri thăm dò) · `font-spike/downloads/` (zip gốc + tệp font) · `font-spike/measurements/` (hai bản `.dmg`, ảnh chụp, số liệu thô).

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-03 | Mũi thăm dò chạy xong phần macOS. AC2, AC3, AC4 đạt. AC1 đạt nửa: `.dmg` đo thật (chênh lệch 20,300 MiB, tổng 151,29 MB, lọt trần NFR6), `.msi` không đo được trên macOS. Chốt biến thể vùng **TC**. Kết quả ghi vào 6 tệp quy hoạch + 1 báo cáo mới + 4 ảnh chụp. `lint_spine.py` 0 findings |
| 2026-08-03 | Ice hỏi bản Adobe có đẹp hơn không. Trả lời: **không có gì để đổi** — `Source Han Serif` và `Noto Serif CJK` là cùng một font (23,4/26 MiB của bộ font), khác đúng cái tên trong bảng `name`. Kênh Google **giữ nguyên**, không sửa tài liệu nào. Bỏ luôn phép so Adobe/Google phần Latin |
| 2026-08-03 | **Code review ba lớp** (Blind Hunter · Edge Case Hunter · Acceptance Auditor) + kiểm số học và bốn ảnh chụp. 73 findings thô → 23 giữ lại: **5 quyết định + 12 patch áp xong**, 5 hoãn. Nặng nhất: báo cáo **đọc sai `[A2]`** — 150–200 MB là trần của **cả bản cài đã gồm font**, không phải dự báo database, nên rủi ro "220,3 MB" là giả; phép tính đúng là **trừ dư địa** và cho kết quả xấu hơn (~47 MB còn lại cho dữ liệu **và toàn bộ mã chưa viết**). Ba bàn giao treo thành **AC thật** ở Story 1.2 / 1.3 / 1.9. Hai token `italic` dựng chữ Hán **nghiêng giả** qua fallback — tiêu chí soát cũ sai điều kiện. Ảnh chụp đến từ **bản build thứ ba** (5 font), không phải bản đã đo (4 font). 29 mockup bỏ `Noto Serif CJK SC`. Con số **150,3 → 151,29 MB** ở 6 tài liệu. `lint_spine.py` 0 findings |
| 2026-08-03 | Ice quyết **bỏ phép đo `.msi` khỏi story này, chuyển sang Story 1.3**. AC1 thu hẹp ở **cả** `epics.md` **và** story file (bản gốc giữ dạng gạch ngang); Story 1.3 nhận **AC mới** đo `.msi` có/không font kèm chế độ cài WebView2, ghi lại mỗi lần phát hành. Công thức chạy chép vào §Công thức đo trên Windows của báo cáo để không mất khi scratchpad bị dọn. Task 3 thu hẹp phạm vi tương ứng. Story → `review` |

---

### Review Findings

*Rà soát 2026-08-03 bằng ba lớp song song (Blind Hunter · Edge Case Hunter · Acceptance Auditor) + kiểm chứng số học và bốn ảnh chụp. 73 findings thô → 41 sau khử trùng → 23 giữ lại.*

**Cần Ice quyết**

- [x] [Review][Decision] **Rủi ro 🔴 "220,3 MB vượt trần" dựng trên một cách đọc sai `[A2]`** — `prd.md:826` ghi *"NFR6 | Kích thước **bản cài** kèm toàn bộ từ điển | `[A2]` Ngân sách 150–200 MB"* và `prd.md:1020` ghi *"A2 | Ngân sách 150–200 MB **đủ cho** toàn bộ nguồn từ điển"*. 150–200 MB là **trần của cả bản cài, đã bao gồm font** — không phải dự báo dung lượng database. Báo cáo đọc thành *"ước tổng database khi đủ nguồn là 150–200 MB"* rồi cộng font lên trên chính cái trần đó. Rủi ro đỏ này đã lan vào 6 tài liệu. Rủi ro **thật** thì ngược lại và vẫn đáng lo: font ăn 20,30 MiB nên dư địa còn lại cho **toàn bộ** database là ~130–180 MB, mà hôm nay đã dùng 130 MB cho **3 trong 5** nguồn — và con số 150,3 MB còn **chưa gồm mã sản phẩm** (baseline 1,34 MiB là app một cửa sổ rỗng), cũng chưa rõ đã gồm chỉ mục FTS phụ (~17 MB theo `prd.md:832`) hay chưa.
- [x] [Review][Decision] **AC mới cấy vào Story 1.3 mâu thuẫn với hai AC sẵn có của chính nó** — AC mới (`epics.md:1107-1112`) đòi *"When dựng **bản phát hành**"* và *"chênh lệch cộng với **dung lượng database**"*. Nhưng Story 1.3 có sẵn *"đây **không phải** FR107 — **không build công khai**"* và *"**Then không tải dữ liệu từ điển**"*. Thêm nữa Story 1.3 chạy **trước** Story 1.9 nên chưa có `dict-core.db` để cộng. Dòng `Covers:` của Story 1.3 (`epics.md:1073`) cũng chưa thêm NFR6.
- [x] [Review][Decision] **Bốn tệp font chỉ tồn tại trong scratchpad — `epics.md` không có AC nào nhận việc đưa chúng vào repo** — đã thêm AC vào Story 1.2 trong `epics.md`: đặt bốn tệp vào `src-tauri/resources/fonts/`, đối chiếu SHA-256, kèm ba tệp giấy phép OFL.
  ⚠️ **Đính chính sau khi áp:** finding này ban đầu phát biểu quá rộng — *"không story nào nhận việc"*. Sai. **Story file `1-2-scaffold-...md` (viết 15:05 cùng ngày) ĐÃ có việc này ở Task 9**, đầy đủ cả bốn tên tệp, bước đối chiếu SHA-256, và ba tệp `LICENSE` trong cây nguồn. Lỗ hổng thật hẹp hơn: nó nằm ở **`epics.md`**, tức tầng đặc tả — nên yêu cầu này trước đó chỉ tồn tại nhờ người viết story 1.2 tự nhận ra, không nhờ một AC nào bắt buộc. Thêm AC vào `epics.md` vẫn đúng vì nó đóng lỗ hổng truy vết, nhưng **rủi ro mất tệp chưa bao giờ hiện hữu như finding mô tả**.
- [x] [Review][Decision] **Khuyến nghị "đối chiếu lại ở Story 1.9" chưa thành AC ở bất kỳ đâu** — 5 tài liệu đóng rủi ro NFR6 bằng cách trỏ tới một biện pháp không tồn tại. AC cuối của Story 1.9 (`epics.md:1348`) chỉ là *"đo kích thước `dict-core.db` và đối chiếu với ngân sách NFR6"* — không có bước cộng 20,30 MiB font, và cũng không gồm bốn lớp gỡ rời của Story 1.10. Ngoài ra danh sách nguồn trong báo cáo (*Unihan, Thiều Chửu, Cổ hán văn, VietPhrase, HVTĐTD*) khác PRD (4 nguồn, không có HVTĐTD) và khác hẳn Story 1.9 (*CVDICT, Unihan, CC-CEDICT, viwiktionary, en.wiktionary*).
- [x] [Review][Decision] **NFR6 chưa khai đơn vị và chưa khai là trần hay dải** — bảng *"hai cách đọc"* của AC1 thực chất là **một** phép cộng lệch đơn vị: 20,30 **MiB** + 130 **MB** rồi dán nhãn MiB. Đọc đúng theo MiB thì 130 MB = 123,98 MiB, tổng = 144,28 MiB = **151,29 MB** — chỉ có một đại lượng thật, không phải hai. Con số **150,3** đã lan vào 6 tài liệu. Ở mốc 200 thì MB/MiB lệch 7 %. Và AC1 viết *"nằm trong trần 150–200 MB"* khiến "nhỏ hơn 150" đọc thành trượt, điều vô lý với một ngân sách dung lượng.

**Sửa được ngay**

- [x] [Review][Patch] Hai token italic rơi vào Noto Serif CJK TC qua fallback → nghiêng giả; tiêu chí soát dùng sai điều kiện [DESIGN.md:256,264,269]
- [x] [Review][Patch] Ảnh chụp đến từ một bản build thứ ba (5 tệp font), không phải bản đã đo (4 tệp) — báo cáo mô tả chỉ hai bản build [font-spike-results-2026-08-03.md:42,181,290]
- [x] [Review][Patch] Mệnh đề chặn Epic 1 vẫn sống ở ba chỗ, kèm ước 30–50 MB đã bị chính story bác [epics.md:453,996]
- [x] [Review][Patch] Ba ô `[x]` tick cho việc không làm được hoặc làm ngược lại nội dung subtask [story:112,113,147]
- [x] [Review][Patch] AC4 ghi `✅ Đạt` trong khi mệnh đề `Given` chưa bao giờ kích hoạt — phải là *không áp dụng* [story:336]
- [x] [Review][Patch] Dải ước `.msi` "16–20,5 MiB" không dẫn ra từ bảng nào của chính báo cáo (bảng cho 16,0–20,3) [font-spike-results-2026-08-03.md:108,127,390]
- [x] [Review][Patch] "Sai số 0,1 %" chỉ kiểm nhánh deflate — cận dưới 16,0 (từ `xz -9`) chưa hiệu chuẩn, và cửa sổ LZX 2 MiB làm `xz -9` thành đại lượng thay thế quá lạc quan [font-spike-results-2026-08-03.md:120,122]
- [x] [Review][Patch] `ui-label` nét 700 (họ `ui` = Source Sans 3) chưa từng được kiểm — ảnh chỉ dựng Source Serif 4 ở bốn nét; phần Sans chỉ có một dòng một nét [tc-vs-sc-paragraph-and-latin.png]
- [x] [Review][Patch] 29 mockup vẫn khai SC đứng trước TC; `EXPERIENCE.md:377` vẫn nói *"cả hai biến thể cùng có mặt cho tới khi Story 1.1 chốt"*; ux `.memlog.md` lại ghi *"không đổi mockup nào"*
- [x] [Review][Patch] *"Không có ca ô vuông rỗng nào"* là tuyên bố tuyệt đối sai — mã Ext B/C/D và dị thể cổ văn nằm ngoài phủ mã của Noto Serif CJK [font-spike-results-2026-08-03.md:210]
- [x] [Review][Patch] §Công thức đo trên Windows giả định mặc định **không** font nên chỉ mô tả cách *chồng thêm*; bản thật sẽ có font sẵn trong `tauri.conf.json` nên Story 1.3 cần cách *gỡ ra* để dựng baseline [font-spike-results-2026-08-03.md:370]
- [x] [Review][Patch] Nhóm số lẻ: "7 tệp" vs "6 tệp" vs File List 8 tệp; Latin 2,586 vs 2,586 MiB; `status: complete` trong khi một phép đo trục còn treo; bảng Stack nên nói rõ đọc tên từ name ID nào (ID 1 của tệp Sans là `Source Sans 3 ExtraLight`); `read-cjk` sau khi bỏ SC không còn họ generic dự phòng

**Hoãn — có thật nhưng không thuộc lượt này**

- [x] [Review][Defer] `.memlog.md` của architecture còn `scope: 112 FR, 16 NFR` trong khi spine ghi 131 FR/19 NFR và PRD hiện có 132 FR — có sẵn từ trước, không do thay đổi này gây ra
- [x] [Review][Defer] Chưa đo trên Apple Silicon / universal binary / Windows ARM64 — báo cáo đã nêu ở §Việc chưa làm được nhưng không story nào nhận
- [x] [Review][Defer] Chưa khai artifact phát hành chính thức cho Windows (`.msi` hay NSIS) — thuộc Story 1.3 / 10.2
- [x] [Review][Defer] Đường nạp font (CSP + asset protocol + FontFace) chưa từng chạy trên Windows — thuộc Story 1.3 / 1.4
- [x] [Review][Defer] ~~Ba tệp giấy phép OFL chưa có AC nào đưa vào bundle~~ → ✅ **đã đóng trong lượt này**: thành một vế của AC mới cho Story 1.2. *(Phép đo 20,30 MiB vẫn chưa gồm ba tệp giấy phép — chúng nhỏ, nhưng Story 1.3 nên đo lại khi chúng đã vào bundle.)*
- [x] [Review][Defer] Rà NFR15 đọc `LICENSE` trong zip nhưng chưa đọc name ID 13/14 nhúng trong chính tệp font sẽ phát hành


> **Đã xử lý 2026-08-03.** 5 quyết định và 12 patch áp xong trong cùng phiên rà soát; 5 mục hoãn còn lại đã vào `deferred-work.md`. Chi tiết từng sửa nằm ngay tại chỗ được sửa, dưới dạng ghi chú *"bản đầu ... đã sửa"* — cố ý giữ lại lỗi cũ để đọc lại được, đúng thói quen Giai đoạn 0.
