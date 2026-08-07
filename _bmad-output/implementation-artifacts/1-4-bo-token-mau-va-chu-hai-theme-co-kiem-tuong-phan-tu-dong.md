---
baseline_commit: 0255163d7c2406c83cdfe83b4b05f1fa188bc0cb
---

# Story 1.4: Bộ token màu và chữ hai theme, có kiểm tương phản tự động

Status: done

Epic: 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
Covers: NFR17 · AD-34 · UX-DR1, UX-DR2, UX-DR3, UX-DR5, UX-DR6, UX-DR10, UX-DR11, UX-DR14, UX-DR16

> ⚠️ **Story 1.3 vẫn `in-progress` lúc story này được dựng** — bốn phép nghiệm thu của nó chờ một lượt runner thật (`deferred-work.md:28-32`). Điều đó **không chặn** story này: `ci.yml` đã tồn tại và chạy được ở máy, và Task 8 chỉ thêm **một** bước vào job đã có. Nhưng nếu lượt CI đầu tiên buộc `ci.yml` phải sửa cấu trúc, bước của Task 8 đi theo — đừng dựng một workflow riêng để né va chạm.

> **Story này là nền của 127 story còn lại.** Mọi màu, mọi cỡ chữ, mọi khoảng cách của chín epic sau đều đọc từ thứ dựng ở đây. Một giá trị sai ở story này không hỏng ngay — nó hỏng ở Epic 5 khi ai đó đọc ba tiếng liền và thấy mỏi mắt mà không chỉ ra được nguyên nhân.
>
> **Phần khó nhất không phải gõ 16 + 14 giá trị vào một tệp.** Nó là ba thứ: (1) **một cặp màu trong bảng token đang trượt WCAG AA ở theme tối** — số thật ở §🔴 Phát hiện chặn, chưa tài liệu nào bắt được; (2) `Source Sans 3` là font biến thiên có **mặc định trục `wght = 200`**, nên `ui-label` (700) sẽ ra chữ mảnh nếu thiếu descriptor — và nó **chưa từng được kiểm**; (3) **chữ Hán nghiêng giả CÓ phát sinh** ở hai token, `DESIGN.md` giao đích danh cho story này xử lý.

---

## Story

As a **người dựng**,
I want **mọi màu và mọi cỡ chữ đến từ một bộ token đã kiểm tương phản**,
So that **một lần đổi nhầm không thể âm thầm đẩy chữ xuống dưới WCAG AA**.

---

## Acceptance Criteria

### AC1 — Đủ token, đúng giá trị

**Given** bốn bảng token ở `DESIGN.md` — *Bảng token màu*, *Bảng token typography*, *Bảng token khoảng cách và hình dạng*
**When** `src/tokens/` dựng xong
**Then** có đủ **16 token màu cho theme sáng và 16 cho theme tối**, **14 token typography**, **bốn họ chữ** (`read` · `read-cjk` · `ui` · `mono`), bộ spacing và bộ rounded
**And** mỗi token khớp **đúng giá trị** ở bảng, không phải một giá trị gần đúng

> ⚠️ **16, không phải 17.** `DESIGN.md §Vì sao 16 chứ không phải 17` cảnh báo tường minh: `tm-rule` giữ **cùng một giá trị ở cả hai theme** nên dễ bị đếm thành hai. **Không được bịa thêm một token thứ 17 để cho khớp một con số cũ** — con số 17 đã bị sửa thành 16 ở `UX-DR1` và ở AC của story trước ngày 2026-08-03.

### AC2 — Màu viết thẳng trong component bị từ chối bằng lệnh

**Given** một component bất kỳ
**When** lint chạy
**Then** giá trị màu viết thẳng trong component bị từ chối

### AC3 — Kiểm tương phản tự động, cả hai theme

**Given** mọi cặp (màu chữ, màu nền) dùng trong ứng dụng
**When** chạy kiểm tương phản tự động
**Then** tất cả đạt WCAG AA ở **cả hai** theme
**And** `#7d766c`, `ornament` và `tm-rule` không xuất hiện làm màu chữ ở bất kỳ đâu

### AC4 — Lùi chữ bằng token, không bằng `opacity`

**Given** một khối chữ cần lùi ra sau
**When** cài đặt
**Then** lùi bằng cách đổi sang token `on-surface-variant`
**And** `opacity` ở trạng thái nghỉ chỉ áp cho nét và nền, không áp cho chữ

### AC5 — Sàn giãn dòng

**Given** mọi token họ `read`
**When** kiểm
**Then** không token nào có `line-height` dưới **1.66**
**And** token họ `ui` dùng cho nhãn có khả năng xuống dòng cũng ở 1.66

### AC6 — Phân tách panel đảo ngược giữa hai theme

**Given** theme tối
**When** hai panel đứng cạnh nhau
**Then** phân tách bằng **khe 2px** để `background` lộ ra, panel bo `3px`
**And** ở theme sáng phân tách bằng **đường kẻ 1px** `outline`

### AC7 — Không elevation

**Given** toàn bộ giao diện
**When** kiểm
**Then** không có bóng đổ, gradient hay lớp nổi nào ngoài bóng cửa sổ do hệ điều hành vẽ

---

## Tasks / Subtasks

- [x] **Task 1 — Dựng nguồn sự thật một tệp cho toàn bộ token** (AC: 1)
  - [x] Tạo `src/tokens/tokens.json` chứa: `colors.light` (16) · `colors.dark` (16) · `typography` (14) · `families` (4) · `spacing` · `rounded`. Giá trị chép từ **`DESIGN.md` §Bảng token màu / §Bảng token typography / §Bảng token khoảng cách và hình dạng**, không chép từ frontmatter YAML *(frontmatter và bảng khớp nhau hôm nay, nhưng bảng là thứ `DESIGN.md` tự khai là "nguồn sự thật")*
  - [x] **Đúng một nguồn sự thật.** Không tạo song song một tệp `.css` viết tay mang cùng giá trị — hai bản chép sẽ lệch nhau ở lần sửa thứ ba. `tsconfig.json` đã bật `resolveJsonModule` nên TS đọc trực tiếp, và `.mjs` đọc bằng `JSON.parse` — cùng một tệp, hai người tiêu thụ
  - [x] Tạo `src/tokens/index.ts`: import typed + hàm `applyTheme(theme: 'light' | 'dark')` ghi CSS custom properties lên `document.documentElement`. Gọi **trước `mount()`** trong `main.ts` để không có nháy màu
  - [x] Quy ước tên biến CSS: `--color-<token>` · `--font-<token>` · `--family-<token>` · `--space-<token>` · `--radius-<token>`. Chốt một lần, 127 story sau dùng lại

- [x] **Task 2 — Cổng cưỡng chế `scripts/check-tokens.mjs`** (AC: 1, 2, 3, 4, 5, 7)
  - [x] Kiểm A (AC1): đối chiếu `tokens.json` với một **bảng kỳ vọng đóng băng trong chính script** — hai bản chép độc lập phải khớp. Đếm đúng 16/16/14/4. Cùng khuôn với `BANNED_CRATES` của `check-deps.mjs` và allowlist của `config_invariants.rs`
  - [x] Kiểm B (AC2): quét `src/**/*.{vue,ts,css}` tìm giá trị màu viết thẳng — `#rgb` · `#rrggbb` · `#rrggbbaa` · `rgb()` · `rgba()` · `hsl()` · `hsla()` · `color()` · **và tên màu CSS** (`red`, `white`, `black`, …). Miễn trừ: chính `src/tokens/**`. Chỉ quét hex là bỏ lọt bốn cú pháp
  - [x] Kiểm C (AC3): tính tỉ lệ tương phản WCAG 2.x cho **mọi cặp đã khai** ở §Bảng tương phản, cả hai theme. Sàn 4.5:1; 3:1 chỉ cho token cỡ ≥ 24px (`lookup-headword`). Thêm phép chặn: `#7d766c` vắng mặt hoàn toàn; `ornament` và `tm-rule` **không xuất hiện sau `color:`**
  - [x] Kiểm D (AC4): quét `opacity:` trong `src/**` — giá trị **khác 0 và khác 1** trên một khai báo cùng khối với `color:` là FAIL. Miễn trừ có tên: khai báo mang comment `/* aura-allow-opacity: <lý do> */`
  - [x] Kiểm E (AC5): mọi token họ `read` có `lineHeight` ≥ **1.66**. Token họ `ui` mang cờ `wraps: true` cũng ≥ 1.66
  - [x] Kiểm F (AC7): quét `box-shadow` · `text-shadow` · `filter: drop-shadow` · `linear-gradient` · `radial-gradient` · `z-index` trong `src/**` — có là FAIL
  - [x] ⚠️ **Mã thoát là phán quyết.** In cảnh báo rồi `exit 0` là một cổng không cưỡng chế được gì (§Testing standards)
  - [x] Thêm `"check:tokens": "node scripts/check-tokens.mjs"` vào `package.json`

- [x] **Task 3 — Nghiệm thu đỏ-rồi-xanh cho cả sáu phép kiểm** (AC: 1, 2, 3, 4, 5, 7)
  - [x] Với **từng** kiểm A–F: cố ý tạo một vi phạm → chạy → phải **đỏ**; gỡ vi phạm → phải **xanh**. Ghi bảng kết quả vào §Debug Log References
  - [x] không Một cổng chưa từng đỏ là một cổng chưa được chứng minh. Story 1.3 §Task 11 đã đặt tiền lệ này

- [x] **Task 4 — Nạp font lúc chạy, và kiểm bốn nét của `Source Sans 3`** (AC: 1)
  - [x] Tạo `src/tokens/fonts.ts`: đăng ký bốn `FontFace` từ `$RESOURCE/fonts/**` qua `resolveResource()` + `convertFileSrc()`, y hệt đường `scopeCheck.ts:220-242` đã chứng minh chạy được dưới CSP
  - [x] không Descriptor **bắt buộc** cho hai tệp biến thiên: `{ weight: '200 900' }`. Thiếu nó thì `Source Sans 3` khoá ở `wght = 200` (ExtraLight) và `ui-label` 700 ra chữ mảnh — hoặc tệ hơn, trình duyệt tổng hợp nét đậm giả
  - [x] `SourceSerif4[opsz,wght].ttf` + `SourceSerif4-Italic[opsz,wght].ttf` → cùng `family: 'Source Serif 4'`, khác `style`. `NotoSerifCJKtc-Regular.otf` → `'Noto Serif CJK TC'`, chỉ `weight: '400'`
  - [x] **Dựng thật `Source Sans 3` ở 400 / 600 / 700** trên một trang thăm dò rồi chụp lại. Mệnh đề *"nét 600 và 700 là nét thật"* của `DESIGN.md` **chỉ mới được chứng minh cho `Source Serif 4`** — `Source Sans 3` chưa từng dựng quá một nét
  - [x] Kiểm bằng **chuỗi dày dấu tiếng Việt** (`ế ộ ữ ẳ ườ` — UX-DR10) chứ không bằng chữ Latin

- [x] **Task 5 — Xử lý chữ Hán nghiêng giả ở hai token** (AC: 1)
  - [x] Hai token dính: `source-hanviet` (#6) và `lookup-example` (#10) — cả hai `italic`, cả hai họ `read`, mà `families.read` có `Noto Serif CJK TC` trong chuỗi dự phòng và tệp đó **chỉ có Regular**
  - [x] Chọn một trong hai đường `DESIGN.md` đã liệt: (a) chấp nhận nghiêng giả cho phần Hán; (b) khai `font-style: normal` cho ký tự CJK trong hai token đó, qua `unicode-range` hoặc một `@font-face` riêng
  - [x] **Thêm một tệp nghiêng CJK KHÔNG phải phương án** — đó là ~23 MiB, một phần ba ngân sách font, và dư địa NFR6 chỉ còn ~47 MB
  - [x] Ghi lựa chọn **và lý do** vào §Completion Notes. Đây là quyết định thị giác, không có đáp án máy chấm được

- [x] **Task 6 — Reset CSS toàn cục** (AC: 7)
  - [x] Tạo `src/tokens/reset.css`, import trong `main.ts`. Đóng mục `deferred-work.md:20`: `body` có margin 8px mặc định + `.shell { min-height: 100vh }` ⇒ **thanh cuộn ở một cửa sổ trống**
  - [x] Tối thiểu: `box-sizing: border-box` toàn cục · `body { margin: 0 }` · `background`/`color` từ token · `-webkit-font-smoothing` nhất quán hai nền tảng
  - [x] Không kéo `normalize.css`/`modern-css-reset` về — mỗi phụ thuộc mới phải rà GPLv3 và vào bảng Stack **trước** (NFR15)
  - [x] Sửa `src/App.vue`: bỏ `font-family` và `font-size` viết thẳng ở `.selftest` (`:37-38`) — chúng sẽ làm Kiểm B/E đỏ ngay lượt đầu. Chuyển sang `var(--family-mono)` + `var(--font-ui-mono)`

- [x] **Task 7 — Token phân tách panel hai theme** (AC: 6)
  - [x] Story này **không** dựng panel (đó là Story 1.14). Nó khai **cơ chế** ở tầng token: theme sáng → `--panel-separator: 1px solid var(--color-outline)`, khe 0; theme tối → khe `2px` lộ `background`, panel `border-radius: 3px`, không đường kẻ
  - [x] Không thống nhất hai theme về một cách làm. `outline #3b382f` trên `surface #26241f` chỉ đạt **1,32:1** — mình đã tính lại, khớp con số 1,39 mà `DESIGN.md` ghi ở dải làm tròn khác. Gần như vô hình ở cả hai cách tính
  - [x] Thêm một phép kiểm vào `check-tokens.mjs`: hai theme phải khai **hai cơ chế khác nhau**, giống nhau là FAIL

- [x] **Task 8 — Gắn vào pipeline đã có** (AC: 2, 3)
  - [x] Thêm **một** bước `npm run check:tokens` vào `.github/workflows/ci.yml`, trong job `check` đã có
  - [x] **Không dựng pipeline thứ hai** — AC4 của Story 1.3 cấm tường minh, và §AC4 của tệp đó đã chừa sẵn chỗ móc mang tên *"lint cấm màu viết thẳng (AD-34)"* cho đúng story này
  - [x] Đặt bước **trước** `npm run build` — nó chạy trong vài giây và không cần `dist/`, nên một lỗi token nên đỏ trước khi tốn một lượt biên dịch Rust
  - [x] ⚠️ Bước này **không** cần cửa sổ đồ hoạ. Đừng đặt nó xuống cụm cuối cùng nơi hai phép kiểm cần webview đang đứng

- [x] **Task 9 — Trang thăm dò thị giác, và gỡ nó đi** (AC: 1, 5)
  - [x] Dựng tạm một trang bày cả 14 token typography trên cả hai theme, dùng **chuỗi dày dấu tiếng Việt** + một đoạn chữ Hán + một dòng Latin, để kiểm mắt ba hệ chữ có cân nhau không
  - [x] Chụp lại làm bằng chứng, ghi vào §Completion Notes, rồi **gỡ trang đó khỏi cây nguồn**. Cùng khuôn với §Ranh giới phạm vi của mũi thăm dò Story 1.1: tài nguyên dùng một lần không vào repo

---

### Review Findings

*Lượt rà soát `bmad-code-review` ngày 2026-08-03. Ba lớp song song (Blind Hunter · Edge Case Hunter · Acceptance Auditor), không lớp nào thất bại. Số học tương phản và phép đếm 16/16/14/4 đã được lớp thứ ba **tự tái lập độc lập và khớp**; bảy giả thuyết nặng nhất đã được **chạy thật trong sandbox**, kết quả ghi kèm.*

**Quyết định của Ice — đã chốt 2026-08-03**

- [x] [Review][Decision] **Ba deviation khỏi `DESIGN.md`** — `colors.dark.surface-accent` `#2c3a3b` → `#283637`; `typography.lookup-gloss.lineHeight` và `typography.lookup-example.lineHeight` `1.6` → `1.66`. → **Ice PHÊ CHUẨN cả ba, `DESIGN.md` chưa sửa.** Giữ tiền lệ *"dev không tự sửa tài liệu quy hoạch"* (quyết định #3 của Story 1.3). Việc chỉnh `DESIGN.md` cho khớp là một lượt riêng của Ice → ghi vào `deferred-work.md`.
- [x] [Review][Decision] **Phạm vi Kiểm D so với AC4** — cổng chỉ đỏ khi `opacity` trung gian nằm cùng khối với `color:`; `.dimmed { opacity: 0.4 }` trên thẻ bọc lọt qua (đã chạy thật, exit 0). → **Ice chốt NỚI RỘNG:** mọi `opacity` trung gian trong `src/**` là FAIL trừ khi mang miễn trừ có tên `/* aura-allow-opacity: <lý do> */`. Hợp đồng mới của cổng, áp cho 127 story sau.
- [x] [Review][Decision] **Bằng chứng thị giác của Task 4 · 5 · 9** — trang thăm dò, bốn ảnh chụp và bộ đọc `fvar` nằm ngoài repo có chủ ý (tiền lệ Story 1.1). → **Ice CHẤP NHẬN văn xuôi làm bằng chứng**, kèm điều kiện ghi rõ ba mệnh đề đang đứng bằng văn xuôi vào `deferred-work.md` để lượt đo NFR của Story 1.9 / 10.9 nhặt lại.

**Vá được — ĐÃ ÁP TOÀN BỘ 21/21 ngày 2026-08-03**

> Hồi quy sau lượt vá: `check:tokens` **exit 0** · `check:deps` **exit 0** · `npm run build` (vue-tsc ×2 + vite) **exit 0** · nghiệm thu đỏ-rồi-xanh **52/52 ca đạt** (bảng ở §Debug Log References). Cây nguồn thật không bị đụng trong lúc nghiệm thu — mỗi ca dựng lại một sandbox sạch ngoài repo.

- [x] [Review][Patch] **Nới rộng Kiểm D theo quyết định của Ice** [scripts/check-tokens.mjs:819] — bỏ điều kiện `if (!hasColor) continue`; mọi `opacity` khác 0 và khác 1 trong `src/**` là FAIL trừ khi mang `/* aura-allow-opacity: <lý do> */` trong phạm vi một dòng. Nghiệm thu đỏ-rồi-xanh lại cho cả hai chiều (thẻ bọc → đỏ; có miễn trừ → xanh), và cập nhật §Debug Log References.
- [x] [Review][Patch] **Ghi ba mục vào `deferred-work.md`** theo hai quyết định của Ice — (a) `DESIGN.md` cần một lượt chỉnh của Ice cho ba giá trị đã phê chuẩn (`surface-accent` tối, `lookup-gloss`, `lookup-example`), tới lúc đó sổ `deviations` là chỗ giữ sự thật; (b) ba mệnh đề thị giác của Task 4/5 đang đứng bằng văn xuôi — *"bốn nét `Source Sans 3` phân biệt rõ"*, *"`ui-label` 700 là nét thật"*, *"chữ Hán đứng thẳng dưới `font-synthesis: none` trong khi Latin vẫn nghiêng thật"* — chưa có bằng chứng tái lập được từ cây nguồn, và **chưa đo trên WKWebView/Windows**; nhặt lại ở lượt đo NFR của Story 1.9 / 10.9.
- [x] [Review][Patch] 🔴 Một dấu nháy lẻ trong template làm Kiểm B/D/F **im lặng xanh** [scripts/check-tokens.mjs:262] — `maskCommentsAndStrings` không có chốt "chuỗi chưa đóng": gặp `'` không có cặp thì `blankRange` xoá trắng tới cuối tệp. **Đã chạy thật:** `<p>don't</p>` trong template + `.a { color: …; opacity: 0.4; box-shadow: …; z-index: 5 }` trong `<style>` → **0 FAIL, exit 0**; cùng khối đó mà không có dấu nháy lẻ → **3 FAIL, exit 1**. Cùng lỗ hổng với `/*` không đóng.
- [x] [Review][Patch] 🔴 Sàn WCAG đọc từ chính tệp bị kiểm [scripts/check-tokens.mjs:701] — `cfg.floors ?? {…}` lấy từ `tokens.contrast.floors`, không nằm trong bảng đóng băng. **Đã chạy thật:** khôi phục `colors.dark.surface-accent` về `#2c3a3b` (khớp bảng đóng băng nên **không cần deviation nào**) + hạ `floors.normal` xuống 3.0 → in `[dark] 31 cặp đạt AA · thấp nhất 4.245:1`, **exit 0**. 4,5 và 3,0 là hằng số WCAG, phải đóng băng trong script. Cùng vấn đề: `largeTextMinPx` [:753].
- [x] [Review][Patch] 🔴 **Chuyển** một cặp trượt sang `excluded` là đường thoát [scripts/check-tokens.mjs:710] — `excluded` chỉ đòi một chuỗi `reason` không rỗng. **Đã chạy thật:** khôi phục `#2c3a3b`, chuyển cặp `on-surface-variant × surface-accent` sang `excluded` với lý do ba chữ *"khong dung"* → **exit 0**, cặp 4,245:1 quay lại sản phẩm. §Completion Notes #9 khẳng định *"Xoá một cặp trượt khỏi danh sách kiểm giờ là FAIL, không phải một đường thoát"* — đúng cho **xoá**, sai cho **chuyển**. Đóng băng danh sách `excluded` trong script như `EXPECTED_*`.
- [x] [Review][Patch] `deviations` không đòi lý do [scripts/check-tokens.mjs:446] — chỉ đối chiếu `designValue`/`value`; `reason` và `question` **không hề được đọc**. Đã chạy thật: đổi `colors.light.primary` + thêm một mục deviation **không có trường `reason`** → exit 0. Đòi `reason` và `question` không rỗng.
- [x] [Review][Patch] Danh sách rỗng làm Kiểm C xanh rỗng [scripts/check-tokens.mjs:707,734,778,794] — đã chạy thật: `roles.text`/`roles.surface`/`pairs`/`excluded` rỗng → in `đầy đủ: 0 tổ hợp` · `0 cặp đạt AA · thấp nhất Infinity:1`, **exit 0**. `bannedColorValues`/`neverTextTokens` xoá đi thì cũng im lặng biến mất. Chính script đã có khuôn đúng (`FILE_FLOOR`, *"cây rỗng không phải cây sạch"*) — áp cho các tập này. Kèm: khoá `bannedColorValues` đang dùng làm regex **chưa thoát ký tự** [:781], `rgb(255, 0, 0)` sẽ không bao giờ khớp và khoá rỗng làm `re.exec` lặp vô hạn.
- [x] [Review][Patch] `:style` binding và `style='…'` nháy đơn lọt hoàn toàn [scripts/check-tokens.mjs:354] — regex chỉ bắt `style="…"` nháy kép tĩnh. **Đã chạy thật:** `:style="{ color: 'red', fontSize: '13px' }"` + `style='color: rebeccapurple; font-size: 22px'` → **exit 0**. `:style` là chính cách Vue khai style động.
- [x] [Review][Patch] `#[0-9a-fA-F]{3,8}\b` là máy sinh dương tính giả [scripts/check-tokens.mjs:532] — **đã chạy thật:** `document.querySelector('#faded')` trong `.ts` → `FAIL … màu viết thẳng trong mã: '#faded'`. Cùng số phận: `#dad`, `#face`, `#decade`, `href="#…"`. Comment ở `:233` của chính tệp cảnh báo *"một cổng chỉ đường sai sẽ bị người sau thêm ngoại lệ cho tới khi nó không bắt được gì"* — đây là cái đòn bẩy đó.
- [x] [Review][Patch] Tầm quét quá hẹp [scripts/check-tokens.mjs:200,214] — `SCAN_EXT` chỉ `.vue/.ts/.css` dưới `src/`. Ngoài tầm: `index.html` ở gốc repo (**hôm nay đang là vỏ ứng dụng**, chỗ tự nhiên nhất để một `<style>` lọt vào), `.svg` · `.js/.jsx/.tsx` · `.scss`, và **chính `tokens.json`** — nên mệnh đề *"`#7d766c` vắng mặt hoàn toàn"* không phủ nguồn sự thật token.
- [x] [Review][Patch] `FILE_FLOOR` canh nhầm quần thể [scripts/check-tokens.mjs:56,583] — sàn áp lên `files.length` (7 tệp hôm nay), nhưng Kiểm B/B2 chạy trên `componentDecls`, tức đã lọc bỏ 3 tệp tầng token. Khi `src/tokens/` lên 5 tệp thì toàn bộ cưỡng chế màu/cỡ chữ ở tầng component xanh rỗng với 0 component trong cây.
- [x] [Review][Patch] `applyTheme()` ném lỗi trước `mount()` với theme lạ [src/tokens/index.ts:61] — `tokens.colors[theme]` `undefined` → `Object.entries(undefined)` ném `TypeError` ở tầng module, ứng dụng **không mount**: cửa sổ trắng, đúng thứ comment đầu tệp nói mình tồn tại để chặn. `THEMES` đã export ở `:48` nhưng không ai kiểm. Story 1.8 sẽ nạp giá trị từ đĩa. Kèm: `reset.css:32-40` dùng `var(--color-…)` không có giá trị dự phòng.
- [x] [Review][Patch] `contrast()` trả `NaN` mà im lặng đạt [scripts/check-tokens.mjs:383,764] — `parseInt(hex.slice(1),16)` sai với `#abc` (3 chữ số), `NaN` với `rgb()`/tên màu; `r < floor` với `NaN` là `false` ⇒ cặp được tuyên bố đạt. Phép tự kiểm ở `:398` chỉ phủ đúng một cặp 6 chữ số. Chặn tường minh: giá trị token phải khớp `/^#[0-9a-f]{6}$/i`.
- [x] [Review][Patch] Gói vá bộ phân tích [scripts/check-tokens.mjs:235,337,370,622] — (a) `[...text]` đánh chỉ số theo **code point** trong khi mọi chỉ số nạp vào nó là **UTF-16**: một emoji trong comment làm lệch toàn bộ offset (hôm nay `src/**` chưa có ký tự nào ngoài BMP — đã kiểm, nên là bẫy ngủ); (b) `</style>` khớp phân biệt hoa thường trong khi `<style>` thì không; (c) regex `<style>` chạy trên `text` thô chứ không trên `masked`; (d) mọi `style="…"` của cả tệp gộp thành **một** khối giả ⇒ Kiểm D nhiễu chéo giữa các thẻ không liên quan và số dòng trỏ vào đầu thuộc tính; (e) `scanned[m.index] === undefined` là mã chết (thay bằng khoảng trắng **cùng độ dài**) nên một màu trong `<style>` của `.vue` bị báo **ba lần** và dòng tổng kết đếm *vi phạm* mà gọi là *"phép kiểm"*.
- [x] [Review][Patch] Đọc số thiếu chặt [scripts/check-tokens.mjs:823,866] — `line-height: 20px` qua sàn 1.66 vì `parseFloat` nuốt đơn vị (chỉ với tính từ token mới, token hiện có bị Kiểm A chặn trước); `opacity: 100%` bị báo sai là trung gian; `opacity: var(--x)` và `calc()` bị bỏ qua hoàn toàn.
- [x] [Review][Patch] Danh sách cho phép/cấm còn hở [scripts/check-tokens.mjs:510,594,608,661] — màu hệ thống CSS (`Canvas`, `ButtonFace`, `LinkText`) không có trong `NAMED_COLORS` và thuộc tính ghép không có danh sách cho phép; `color: var(--color-x) !important` và `border-color: var(--a) var(--b)` (hợp lệ, 1–4 giá trị) bị regex neo hai đầu từ chối oan.
- [x] [Review][Patch] `loadFonts()` nối tiếp, không có hạn giờ, không luỹ đẳng [src/tokens/fonts.ts:96,102] — `for … await` bốn tệp một hàng: nếu tệp đầu **treo** thay vì reject thì ba tệp sau không bao giờ được dựng, promise không bao giờ settle, và `.then` ở `main.ts:26` không chạy ⇒ **treo không phân biệt được với thành công trong log**. Gọi lại lần hai (HMR) thêm bốn `FontFace` trùng vào `document.fonts`, không có chốt.
- [x] [Review][Patch] Kiểm F cấm `z-index` mà không có đường miễn trừ có tên [scripts/check-tokens.mjs:886] — trong khi Kiểm D có `/* aura-allow-opacity: … */`. Panel của Story 1.14, dropdown, tooltip và chính dockview đều cần ngữ cảnh xếp lớp; cái `z-index` hợp lệ đầu tiên không có chỗ để tự biện minh, nên phản ứng tự nhiên là **xoá nó khỏi `BANNED_PROPS`** — mất luôn hai lệnh cấm `box-shadow`/`text-shadow` dùng chung tập đó.
- [x] [Review][Patch] `walk()` đi theo symlink không có chốt vòng [scripts/check-tokens.mjs:202] — `statSync` giải symlink ⇒ một liên kết trỏ về thư mục cha làm đệ quy không dừng; một symlink gãy thì ném `ENOENT` và bị báo thành *"cây nguồn không đọc được"*, tức một liên kết hỏng làm sập cả cổng dưới danh nghĩa lỗi hạ tầng. Dùng `lstatSync` + tập đã thăm.
- [x] [Review][Patch] §File List thiếu một thay đổi [_bmad-output/implementation-artifacts/sprint-status.yaml:57] — dòng `1-5-…: backlog → ready-for-dev` nằm trong change set nhưng File List chỉ khai đường đi trạng thái của story 1.4. Bổ sung một dòng, hoặc tách nó khỏi lượt commit này.
- [x] [Review][Patch] `spacing.unit` lấy từ **frontmatter**, không từ bảng [src/tokens/tokens.json:284] — `DESIGN.md:127` (frontmatter YAML) có `unit: 4px`; §Bảng token khoảng cách và hình dạng ở `:283` thì **không**. Task 1 dặn tường minh *"chép từ bảng, không chép từ frontmatter YAML"*. Giá trị vô hại nhưng nó là token thứ mười đi qua Kiểm A từ một nguồn story đã loại — hoặc gỡ, hoặc ghi vào `notes` như trường hợp `source-cjk.family`.

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Story này **có** làm | Story này **KHÔNG** làm |
|---|---|
| `src/tokens/**` — nguồn sự thật token, áp theme, nạp font | Bất kỳ panel, mode hay layout nào — **Story 1.14** |
| `scripts/check-tokens.mjs` + **một** bước trong `ci.yml` đã có | `CommandRegistry`, focus, phím tắt — **Story 1.6** |
| Reset CSS toàn cục, sửa `.selftest` trong `App.vue` | Chuỗi giao diện, `vi.json`, hình dạng lỗi IPC — **Story 1.5** |
| Cơ chế phân tách panel **ở tầng token** (AC6) | Dựng panel thật để nhìn thấy khe 2px — **Story 1.14** |
| Đăng ký `FontFace` và kiểm bốn nét | Đổi tệp font, subset font, đụng `bundle.resources` |
| Xử lý chữ Hán nghiêng giả (Task 5) | Thêm tệp font nghiêng CJK — không ~23 MiB |
| Công tắc theme ở tầng hàm (`applyTheme`) | Giao diện chọn theme, lưu lựa chọn xuống đĩa — **Story 1.8** |

**Không đụng tới:** `src-tauri/**` · `Cargo.toml` · `src-tauri/tauri.conf.json` · `_bmad-output/planning-artifacts/**` *(trừ khi Ice quyết theo §🔴 Phát hiện chặn, và khi đó ghi rõ trong File List — Story 1.3 đã bị bắt vì khai sai đúng dòng này)*.

### 🔴 Phát hiện chặn — một cặp màu trong bảng token TRƯỢT WCAG AA ở theme tối

Tính lúc dựng story, bằng công thức WCAG 2.x (relative luminance, sRGB), trên chính 16 giá trị của `DESIGN.md`:

```
on-surface-variant #a29a8c  trên  surface-accent #2c3a3b  =  4,245 : 1     ← DƯỚI sàn 4,5
error              #e5867a  trên  surface-accent #2c3a3b  =  4,519 : 1     ← sát mép
```

**Vì sao chưa ai bắt được:** `DESIGN.md §Sàn tương phản` kiểm ba màu **trên nền giấy** rồi kết luận *"đã kiểm"*. `surface-accent` ở theme tối chưa từng được đối chiếu với `on-surface-variant` — mà AC3 đòi **"mọi cặp (màu chữ, màu nền) dùng trong ứng dụng"**, không phải mọi cặp trên nền chính. Cổng của Task 2 Kiểm C **sẽ đỏ ngay lượt đầu** nếu cặp này nằm trong danh sách khai báo.

Hai phương án, cả hai đã tính sẵn:

| | Đổi gì | Kết quả | Đánh đổi |
|---|---|---|---|
| **A** *(khuyến nghị)* | `surface-accent` tối: `#2c3a3b` → **`#283637`** | `on-surface-variant` **4,505** · `error` **4,795** · `primary` 5,55 · `on-surface` 9,81 | Đổi **một** token nền, không token chữ nào lay chuyển. `surface-accent` chưa được story nào tiêu thụ nên chi phí đổi ý bằng 0 |
| **B** | `on-surface-variant` tối sáng lên: `#a29a8c` → **`#a79f91`** | trên `surface-accent` **4,512** · `background` 6,34 · `surface` 5,91 · `surface-sunken` 6,64 · `surface-tm` 5,37 | Lan ra **cả năm** mặt nền và đổi sắc độ chữ phụ ở mọi panel — đúng thứ `DESIGN.md` gọi là *"đừng hạ"* |

> ⚠️ **Đường thứ ba, và nó cũng hợp lệ:** khai rằng cặp `on-surface-variant` × `surface-accent` **không bao giờ được dùng** trong ứng dụng, ghi mệnh đề đó thành một dòng trong `tokens.json` và cho `check-tokens.mjs` cưỡng chế nó. AC3 nói *"mọi cặp **dùng trong ứng dụng**"* — một cặp không dùng thì không phải cặp. Nhưng phải **viết ra**, không được im lặng bỏ cặp đó khỏi danh sách kiểm: một danh sách kiểm tự rút gọn để cho xanh là đúng thứ AD-34 tồn tại để chặn.
>
> **Câu hỏi cho Ice ở §Câu hỏi cho Ice.** Nếu Ice chưa trả lời khi dev bắt đầu: **đi theo phương án A**, ghi rõ trong Completion Notes, và **không** sửa `DESIGN.md` — sửa tài liệu quy hoạch là quyết định của Ice, không phải hệ quả phụ của một lượt cài đặt (tiền lệ: quyết định #3 của Ice ở Story 1.3).

### Bảng tương phản đã tính sẵn — dùng làm kỳ vọng của Kiểm C

Sàn AA: **4,5:1** chữ thường · **3:1** chữ lớn (≥ 24px hoặc ≥ 18,66px bold) — trong bảng token chỉ `lookup-headword` (24px) đủ điều kiện chữ lớn.

**Theme sáng** — không cặp nào trượt:

| chữ \ nền | `background` | `surface` | `surface-sunken` | `surface-accent` | `surface-tm` |
|---|---|---|---|---|---|
| `on-surface` | 13,14 | 14,19 | 12,55 | 12,58 | 13,75 |
| `on-surface-variant` | 5,18 | 5,60 | 4,95 | 4,96 | 5,42 |
| `primary` | 6,49 | 7,01 | 6,21 | 6,22 | 6,80 |
| `confirmed` | 5,16 | 5,57 | 4,93 | 4,94 | 5,40 |
| `tm-text` | 5,44 | 5,88 | 5,20 | 5,21 | 5,70 |
| `error` | 7,17 | 7,74 | 6,85 | 6,86 | 7,50 |

`on-primary` trên `primary` = **7,01**.

**Theme tối** — một ô trượt, một ô sát mép:

| chữ \ nền | `background` | `surface` | `surface-sunken` | `surface-accent` | `surface-tm` |
|---|---|---|---|---|---|
| `on-surface` | 12,99 | 12,11 | 13,60 | 9,24 | 10,99 |
| `on-surface-variant` | 5,97 | 5,56 | 6,25 | **4,25 🔴** | 5,05 |
| `primary` | 7,17 | 6,69 | 7,51 | 5,10 | 6,07 |
| `confirmed` | 7,24 | 6,75 | 7,58 | 5,15 | 6,12 |
| `tm-text` | 8,23 | 7,68 | 8,62 | 5,86 | 6,96 |
| `error` | 6,35 | 5,92 | 6,65 | **4,52 ⚠️** | 5,37 |

`on-primary` trên `primary` = **7,51**.

**Ba token là nét, không phải chữ** — chúng **phải** nằm ngoài phép kiểm tương phản chữ, và phải bị chặn khỏi mọi khai báo `color:` (AC3):

| Token | Sáng / trên `surface` | Tối / trên `surface` | Vai |
|---|---|---|---|
| `ornament` | 2,44 | 2,64 | nét vẽ · ký tự `⏐` |
| `tm-rule` | 2,48 *(trên `surface-tm`)* | 5,26 *(trên `surface-tm`)* | vạch lề TM |
| `outline` | 1,31 | 1,32 | nét phân tách |
| `outline-faint` | 1,15 | 1,13 | nét phân tách mờ |

> ⚠️ **`tm-rule` đạt 5,26 ở theme tối — và đó là một cái bẫy.** Một con số qua sàn sẽ cám dỗ ai đó dùng nó làm màu chữ ở theme tối "vì nó đạt mà". Nhưng ở theme **sáng** nó là 2,48, và cùng một token không được đổi vai theo theme. Luật là **vai**, không phải con số: `ornament` và `tm-rule` là màu của nét, ở cả hai theme, không ngoại lệ *(trừ ký tự ranh giới câu `⏐` — ngoại lệ duy nhất đã đặc tả, và nó thuộc Story 2.x)*.

### "Lint" ở dự án này là gì — và vì sao KHÔNG phải ESLint

AC2 nói *"khi **lint** chạy"*. Đọc thẳng ra sẽ là ESLint + một rule tự viết. **Đừng đi đường đó**, ba lý do đo được:

1. **Bảng Stack không có ESLint.** `ARCHITECTURE-SPINE.md §Consistency Conventions` chốt: *"Mỗi phụ thuộc mới phải rà tương thích GPLv3 **trước khi** thêm vào (NFR15) và ghi vào bảng Stack"*. ESLint + `eslint-plugin-vue` + `typescript-eslint` kéo theo hàng trăm gói, mỗi gói một lượt rà bằng cách **mở tệp giấy phép trong nguồn đã tải** — đó là phương pháp Story 1.1 và 1.2 đã thiết lập, không phải đọc nhãn registry.
2. **Cây npm hiện là 59 gói.** `check-deps.mjs` có ngưỡng sàn và quét mẫu trên **tên gói**; một cú nhảy lên vài trăm gói làm mọi con số nghiệm thu của Story 1.2 mất chỗ bám.
3. **Dự án đã có một khuôn đang chạy tốt cho đúng loại việc này**: `check-deps.mjs` (13 phép kiểm) · `check-scope.mjs` · `check-scope-bundled.mjs` · `config_invariants.rs` (15 test). Tất cả là cổng bằng lệnh, mã thoát là phán quyết, không phụ thuộc mới.

⇒ **`scripts/check-tokens.mjs`, Node thuần, không phụ thuộc mới.** Nó *là* lint theo nghĩa AC2 dùng: một phép kiểm tĩnh chạy bằng lệnh, từ chối bằng mã thoát khác 0. `ARCHITECTURE-SPINE.md` gọi luật này là *"lint cấm màu viết thẳng (AD-34)"* — tên gọi, không phải chỉ định công cụ.

⚠️ **Node, không phải bash.** `npm run` trên Windows đi qua `cmd.exe`, không có bash — một cổng chỉ canh được một nửa số nền tảng thì không canh được NFR14. Ice đã chốt điều này ngày 2026-08-03 khi `check-deps.sh` được viết lại thành `.mjs`.

### Bảy thứ sẽ hỏng im lặng

**1. 🔴 Một cổng chỉ đếm token chứng minh được rất ít.**
Kiểm A (đếm 16/16/14/4) là phần dễ và cũng là phần yếu nhất. Thứ AC3 thật sự đòi là **cặp màu dùng trong ứng dụng** đạt AA — mà hôm nay ứng dụng chưa có component nào. Nên cổng phải kiểm **danh sách cặp đã khai** (bảng trên), và khi Story 1.14 dựng panel thật, chính cổng đó sẽ bắt cặp mới. Đừng viết một cổng chỉ đúng cho hôm nay.

**2. 🔴 `Source Sans 3` khoá ở `wght = 200` nếu thiếu descriptor.**
`ARCHITECTURE-SPINE.md §Stack` ghi thẳng: tệp đó có `name ID 1 = Source Sans 3 ExtraLight` vì **mặc định trục `wght = 200`**. `ui-label` khai 700. Không có `{ weight: '200 900' }` thì hoặc ra chữ mảnh, hoặc trình duyệt tổng hợp nét đậm giả — và ở cỡ 10px với `letter-spacing 0.1em`, nét giả trông *gần đúng*, đủ gần để không ai nhận ra trong sáu tháng. `scopeCheck.ts:165,223` đã dùng đúng descriptor này; chép lại, đừng phát minh.

**3. 🔴 Font KHÔNG nằm trong `src/` — đừng `@font-face { src: url('./fonts/…') }`.**
Bốn tệp font sống ở `src-tauri/resources/fonts/`, đi vào bản cài qua `bundle.resources`, và tới được webview qua **asset protocol**. Đường duy nhất đã được chứng minh chạy dưới CSP là `resolveResource()` → `convertFileSrc()` → `new FontFace(...)`. Một `url()` tương đối trong CSS sẽ được Vite giải thành asset của bundle — nghĩa là **font bị nhân bản vào `dist/`**, cộng thẳng ~26 MiB vào payload mà NFR6 chỉ còn ~47 MB dư địa. Nó lại còn *chạy được* trên máy dev, nên bẫy này chỉ lộ ra ở phép đo dung lượng của Story 1.9.

**4. 🔴 `fetch()` tới asset protocol GÃY ở bản đóng gói.**
CSP hiện tại: `connect-src 'self' ipc: http://ipc.localhost` — **không có `asset:`**, trong khi `font-src` **có**. Đo thật trên bản `.app` debug ngày 2026-08-03, bốn sự kiện `securitypolicyviolation` nêu đích danh `connect-src`. Ice đã chốt **giữ nguyên CSP**. ⇒ `FontFace` chạy, `fetch()` không. Nếu bạn thấy mình cần `fetch` một tệp font để kiểm tra gì đó, bạn đang đi vào đúng đường đã đo là hỏng.

**5. 🔴 Kiểm màu viết thẳng chỉ quét hex là bỏ lọt bốn cú pháp.**
`rgb(43 39 35)` · `hsl(30 10% 15%)` · `color(display-p3 …)` · và **tên màu CSS** đều là màu viết thẳng. Kèm theo: quét phải **bỏ qua comment và chuỗi i18n** — nhưng **không** bỏ qua bằng cách nới regex, mà bằng cách quét đúng vùng khai báo CSS. Một cổng dương tính giả sẽ bị người sau thêm ngoại lệ cho tới khi nó không bắt được gì.

**6. 🔴 Luật `opacity` (AC4) KHÔNG bắt được bằng luật màu.**
`DESIGN.md §Opacity không được dùng để làm mờ chữ` có số đo: `opacity: 0.4` trên chữ `on-surface-variant` (5,2:1) ra màn hình ở **≈ 2,3:1** — *"kiểm token vẫn xanh, mắt vẫn không đọc được"*. Đây là phép kiểm **riêng** (Kiểm D), không phải một nhánh của Kiểm B. Phát hiện gốc: kiểm toán bảng chờ Glossary ngày 2026-08-03, hai hàng cùng trượt AA.

**7. ⚠️ AC6 nghiệm thu được ở story này tới đâu.**
Không có panel nào tồn tại hôm nay (Story 1.14 mới dựng). Nếu bạn cố "nhìn thấy khe 2px" thì bạn đang dựng panel — vượt phạm vi. Nghiệm thu đúng ở đây là: **hai theme khai hai cơ chế phân tách khác nhau ở tầng token**, và một phép kiểm chặn việc chúng bị thống nhất về một cách làm. Ghi rõ giới hạn này vào Completion Notes thay vì đánh dấu đạt trọn — tiền lệ `unmeasured` của Story 1.3.

### Trạng thái repo hiện tại — số, không phải mô tả

Đọc lúc dựng story, `HEAD = 0255163`:

| | |
|---|---|
| Nhánh | `master` *(không phải `main`)*, repo **private** |
| `src/tokens/` | **chỉ có `README.md`** — bàn giao từ Story 1.1, đã ghi sẵn cảnh báo `wght = 200` |
| `src/` đang có | `App.vue` · `main.ts` · `i18n/vi.json` · `selftest/{eventName,scopeCheck}.ts` · 5 thư mục chỉ có `README.md` |
| CSS toàn cục | **không có tệp nào.** `index.html` không nạp stylesheet; `App.vue` có `<style scoped>` với 2 quy tắc |
| Màu viết thẳng trong `src/` | **0** *(đã grep)* — cổng Kiểm B sẽ xanh ngay, và đó là lý do Task 3 bắt buộc |
| Font đã ở đúng chỗ | `src-tauri/resources/fonts/` — 4 tệp + 3 tệp OFL, đã vào `bundle.resources` |
| npm deps | 59 gói · 3 dependencies · 5 devDependencies |
| Cổng đang chạy | `check:deps` · `check:scope` · `check:scope:bundled` · `cargo test` (15 test) |
| `ci.yml` | một job `check`, matrix `macos-26` + `windows-2025`, `fail-fast: false` |

**Hai chỗ trong `App.vue` sẽ làm cổng mới đỏ ngay lượt đầu** — sửa ở Task 6, đừng nới cổng để tránh:

```
src/App.vue:37   font-family: ui-monospace, monospace;   → var(--family-mono)
src/App.vue:38   font-size: 0.8125rem;                   → var(--font-ui-mono)
```

### 16 token màu — giá trị đầy đủ, hai theme

Chép từ `DESIGN.md §Bảng token màu`. Đây là thứ Kiểm A đối chiếu; **hai bản chép độc lập phải khớp** (bảng này ↔ `tokens.json`).

| # | Token | Sáng | Tối | Vai |
|---|---|---|---|---|
| 1 | `background` | `#f4f1ea` | `#201e1b` | Nền ngoài cùng; ở theme tối còn là **khe phân tách panel** |
| 2 | `surface` | `#fbfaf6` | `#26241f` | Mặt phẳng làm việc của panel |
| 3 | `surface-sunken` | `#f0ece1` | `#1b1a17` | Vùng lùi — chiều sâu duy nhất của sản phẩm |
| 4 | `surface-accent` | `#e6eeee` | `#2c3a3b` 🔴 | Nền nhấn nhẹ — *xem §Phát hiện chặn* |
| 5 | `surface-tm` | `#faf6ee` | `#302b21` | Nền gợi ý Translation Memory |
| 6 | `on-surface` | `#2b2723` | `#e8e3d8` | Chữ chính |
| 7 | `on-surface-variant` | `#6b6459` | `#a29a8c` | Chữ phụ — **sàn thấp nhất cho mọi chữ** |
| 8 | `outline` | `#e2dccf` | `#3b382f` | Nét phân tách |
| 9 | `outline-faint` | `#efeade` | `#302d26` | Nét phân tách mờ |
| 10 | `ornament` | `#a9a196` | `#6a6459` | **Màu của nét, không bao giờ là màu của chữ** |
| 11 | `primary` | `#2f5d63` | `#7fb3ba` | Màu nhấn **duy nhất** |
| 12 | `on-primary` | `#fbfaf6` | `#1b1a17` | Chữ trên nền `primary` |
| 13 | `confirmed` | `#5a6b3f` | `#9cb37a` | Câu đã xác nhận |
| 14 | `tm-rule` | `#b99a5e` | `#b99a5e` | Vạch gợi ý TM — **cùng giá trị hai theme**, là vạch không phải chữ |
| 15 | `tm-text` | `#7a5d25` | `#d3b276` | Chữ trong khối gợi ý TM |
| 16 | `error` | `#8f2f22` | `#e5867a` | Lỗi |

⚠️ **Hàng 14 là chỗ con số 17 sinh ra.** `tm-rule` có một giá trị dùng cho cả hai theme, không phải hai token. Đếm trên bảng: **16 hàng**.

### Spacing và rounded — đơn vị 4px

```
spacing   panel-inline 16px · panel-block 12px · head-height 34px ·
          titlebar-height 38px · status-height 32px · gutter-width 22px
thước đọc read-measure-lg 62ch · read-measure-md 68ch · read-measure-sm 76ch
rounded   none 0 · sm 2px · DEFAULT 3px · md 4px · window 9px · full 9999px
```

⚠️ **Thước đọc dùng `ch`, không phải `px`** — thước đo đúng là số ký tự mỗi dòng, và nó phải giữ nguyên khi người dùng đổi cỡ chữ. Đổi sang `px` là phá đúng thứ ba mức Thoáng/Cân/Đặc tồn tại để làm.

### Bốn họ chữ và 14 token — chép đúng, đừng làm tròn

Bốn họ *(`DESIGN.md` frontmatter `families`)*:

```
read      "Source Serif 4", "Noto Serif CJK TC", serif
read-cjk  "Noto Serif CJK TC", serif
ui        "Source Sans 3", ui-sans-serif, -apple-system, "Segoe UI", system-ui, sans-serif
mono      ui-monospace, SFMono-Regular, Consolas, monospace
```

⚠️ **`families.read` có `Noto Serif CJK TC` trong chuỗi dự phòng** — đây chính là cơ chế sinh ra vấn đề nghiêng giả ở Task 5. Chữ Hán rơi vào tệp CJK qua fallback dù token khai họ `read`, và tệp CJK chỉ có Regular.

14 token typography *(cỡ / giãn dòng / khác — họ)*: `read-lg` 19px/1.95/`0.004em` · `read-md` 17.5px/1.8 · `read-sm` 16px/1.66 · `read-title` 23px/600/1.3 · `source-cjk` 16.5px/2.05 **(họ `read-cjk`)** · `source-hanviet` 12.5px/italic/1.95 · `editor` 15px/1.95 · `lookup-headword` 24px/1.3 · `lookup-gloss` 14.5px/1.6 · `lookup-example` 12.5px/italic/1.6 · `ui-md` 12px/1.5 · `ui-sm` 11.5px/1.5 · `ui-label` 10px/700/1.4/`0.1em` · `ui-mono` 10.5px/1.4 **(họ `mono`)**.

> 🔴 **Đọc kỹ AC5, nó KHÔNG nói "không token nào dưới 1.66".** Bảng ngay trên có `read-title` ở **1.3** và `lookup-headword` ở **1.3** — cả hai họ `read`. Nếu bạn cài Kiểm E thành *"mọi token họ `read` ≥ 1.66"* thì **hai token đúng đặc tả sẽ làm cổng đỏ**, và phản ứng tự nhiên là sửa bảng token cho khớp cổng — hỏng đúng chiều tệ nhất.
>
> Ranh giới thật, `DESIGN.md §Giãn dòng 1.66 là sàn cứng` phát biểu rõ: **chữ có chạy thành đoạn hay không.** Tiêu đề một dòng (`read-title`, `lookup-headword`) không có dòng dưới để dấu chạm vào. Nên `tokens.json` phải mang một cờ tường minh cho từng token — ví dụ `wraps: true | false` — và Kiểm E áp sàn 1.66 **cho các token `wraps: true`**, bất kể họ. Đây là lý do `ui-md`/`ui-sm`/`ui-label`/`ui-mono` được ở 1.4–1.5: chúng là nhãn một dòng.
>
> *(Bản đầu của luật này trong `DESIGN.md` phát biểu là "không token văn bản nào dưới 1.66" trong khi bảng ngay trên đặt `ui-md` ở 1.5 — tài liệu tự mâu thuẫn, đã sửa 2026-08-03. Đừng khôi phục cách phát biểu cũ vào mã.)*

### Ba nguyên tắc thị giác mà cổng không bắt được — vẫn phải giữ

- **Màu là thông tin, không phải trang trí.** Một màu nhấn duy nhất (`primary`), dùng cho đúng ba việc: thuật ngữ Glossary đã chốt · nhãn nguồn từ điển · tiêu điểm bàn phím. Không cho nút bấm, không cho tiêu đề.
- **Không trắng tinh, không đen tuyền**, ở cả hai theme. Nền tối `#26241f` là nâu rất tối; chữ `#e8e3d8` là ngà. Tương phản tuyệt đối gây loá sau vài chương.
- **Hình dạng chủ đạo là vạch dọc, không phải hộp bo tròn.** Bo góc mặc định `3px`, `2px` cho vạch và chip, `9px` cho cửa sổ.

### Testing standards — thừa kế nguyên từ Story 1.2 và 1.3

- **Mã thoát là phán quyết.** Script in cảnh báo rồi `exit 0` không cưỡng chế được gì.
- **Nghiệm thu bằng đỏ trước, xanh sau.** Task 3 là bắt buộc, không phải nice-to-have — sáu phép kiểm, sáu lần.
- **Cây rỗng không phải cây sạch.** Áp vào story này: một lượt quét **không tìm thấy tệp nào** trong `src/**` phải là **lỗi quét**, không phải "đạt". Đặt ngưỡng sàn số tệp, cùng khuôn `RUST_TREE_FLOOR`/`NPM_TREE_FLOOR`.
- **Danh sách CHO PHÉP, không phải danh sách CẤM.** `config_invariants.rs:92-94` phát biểu nguyên tắc này; Kiểm B nên hỏi *"token nào được dùng ở đây"* thay vì *"cú pháp màu nào bị cấm"* nếu làm được — một danh sách cấm chỉ chặn được những hình dạng ai đó đã nghĩ ra.
- **Số, không phải tính từ.** *"Tương phản tốt"* không nghiệm thu được AC3; `4,505:1 ≥ 4,5` thì có.
- **Không đo được ≠ đạt.** Story 1.3 dựng trạng thái `unmeasured` cho đúng việc này; AC6 của story này là ứng viên đầu tiên dùng lại nó.

### Bốn mục `deferred-work.md` chạm tới story này

| Mục | Đóng ở đây? |
|---|---|
| `:20` — `.shell{min-height:100vh}` + margin 8px sinh thanh cuộn, **không có reset CSS toàn cục** | ✅ **Có** — Task 6 |
| `:7` — *"đường nạp font chưa từng chạy trên Windows"* | ⚠️ **Một nửa.** Story 1.3 đã đưa chiều dương qua `font-src` vào CI trên cả hai nền tảng (`ci.yml:370`). Story này đưa **đường nạp thật của sản phẩm** vào cùng pipeline. Nhưng *nhìn thấy chữ hiện đúng nét trên Windows* vẫn cần một lượt runner có ảnh chụp — ghi lại, đừng đánh dấu đạt trọn |
| `:47` — không ESLint / Prettier / test runner frontend | ❌ **Không** — §"Lint" ở dự án này là gì giải thích vì sao. Story này thêm **một** cổng chuyên trách, không mở cánh cửa công cụ |
| `:26` — `connect-src` thiếu `asset:` | ❌ **Không** — thuộc Story 1.9/10.1. Story này chỉ cần **biết** để không đi vào đường `fetch` (Bẫy #4) |

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.4` — bảy AC nguyên văn, `:1134-1176`]
- [Source: `_bmad-output/planning-artifacts/epics.md#UX-DR1` — 16 token màu, hai theme, liệt kê tường minh, `:493`]
- [Source: `_bmad-output/planning-artifacts/epics.md#UX-DR2, UX-DR3` — 14 token typography, bốn họ, spacing/rounded, `:495-497`]
- [Source: `_bmad-output/planning-artifacts/epics.md#UX-DR5, UX-DR6` — ba màu đã loại; luật `opacity`, `:503-505`]
- [Source: `_bmad-output/planning-artifacts/epics.md#UX-DR10, UX-DR11` — sàn 1.66 và ngoại lệ nhãn một dòng, `:515-517`]
- [Source: `_bmad-output/planning-artifacts/epics.md#UX-DR14, UX-DR16` — phân tách panel đảo ngược; không elevation, `:525-529`]
- [Source: `.../DESIGN.md#Bảng token màu` — **nguồn sự thật**, 16 hàng hai theme, `:165-190`]
- [Source: `.../DESIGN.md#Sàn tương phản — đã kiểm, đừng hạ` — ba màu đã loại kèm tỉ lệ, `:192-204`]
- [Source: `.../DESIGN.md#Opacity không được dùng để làm mờ chữ` — số đo 5,2 → 2,3, `:206-219`]
- [Source: `.../DESIGN.md#Bảng token typography` — 14 hàng + hai ghi chú giao việc cho story này, `:250-275`]
- [Source: `.../DESIGN.md#Giãn dòng 1.66 là sàn cứng` — bảng ba loại chuỗi, phép thử "có xuống dòng không", `:287-303`]
- [Source: `.../DESIGN.md#Phân tách panel đảo ngược giữa hai theme` — 1,39:1, `:321-330`]
- [Source: `.../ARCHITECTURE-SPINE.md#AD-34` — ba mệnh đề của sàn a11y, `:369-380`]
- [Source: `.../ARCHITECTURE-SPINE.md#Consistency Conventions` — hàng **Màu**; hàng **Giấy phép**, `:548`, `:557`]
- [Source: `.../ARCHITECTURE-SPINE.md#Stack` — `Source Sans 3` `name ID 1 = ExtraLight`, mặc định `wght = 200`, RFN `'Source'`, `:586-590`]
- [Source: `.../ARCHITECTURE-SPINE.md#Structural Seed` — `src/tokens/` giữ token đã kiểm tương phản, `:709`]
- [Source: `_bmad-output/implementation-artifacts/1-3-…-moi-lan-push.md#AC4` — chỗ móc mang tên *"lint cấm màu viết thẳng (AD-34)"*, `:52-58`]
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md` — `:7`, `:20`, `:26`, `:47`]
- [Source: `src/tokens/README.md` — bàn giao Story 1.1: descriptor `{ weight: "200 900" }`]
- [Source: `src/selftest/scopeCheck.ts:165,220-242` — đường `FontFace` đã chứng minh chạy dưới CSP]
- [Source: `src-tauri/tauri.conf.json:25` — CSP: `font-src` **có** `asset:`, `connect-src` **không**]
- [Source: `src-tauri/tauri.conf.json:35-40` — `bundle.resources`, bốn tệp font + ba OFL + `license/`]
- [Source: `src/App.vue:31,37-38` — `min-height: 100vh`; hai giá trị viết thẳng phải chuyển sang token]
- [Source: `.github/workflows/ci.yml:41-107` — job `check`, thứ tự bước, `npm run build` phải trước `cargo test`]
- [Đo lúc dựng story, 2026-08-03] Toàn bộ §Bảng tương phản — WCAG 2.x relative luminance trên chính 16 giá trị của `DESIGN.md`; phương pháp tự kiểm bằng cách tái lập con số **5,2:1** mà `DESIGN.md` công bố cho `on-surface-variant` trên nền giấy → tính ra **5,18**, khớp

---

## Câu hỏi cho Ice

| # | Câu hỏi | Vì sao cần Ice | Mặc định nếu chưa trả lời |
|---|---|---|---|
| 1 | Cặp `on-surface-variant` × `surface-accent` ở theme tối = **4,25:1**, trượt AA. Sửa `surface-accent` → `#283637` (A), sửa `on-surface-variant` → `#a79f91` (B), hay khai cặp này **không dùng** (C)? | Đổi một giá trị trong bảng token là chạm vào `DESIGN.md` — tầng quy hoạch. Tiền lệ Story 1.3: dev **không** tự sửa tài liệu quy hoạch | **Phương án A**, ghi vào Completion Notes, **không** sửa `DESIGN.md` |
| 2 | Chữ Hán nghiêng giả ở `source-hanviet` và `lookup-example`: chấp nhận, hay khai `font-style: normal` cho ký tự CJK? | Quyết định thị giác, không có đáp án máy chấm được. Ảnh hưởng đến Panel Lookup — nơi ví dụ từ điển Trung–Việt **chắc chắn** có chữ Hán ở cỡ 12,5px | Khai `font-style: normal` cho CJK — nghiêng giả ở 12,5px là chỗ nó xấu nhất |
| 3 | AC6 nghiệm thu ở tầng token là **đủ**, hay Ice muốn thấy hai panel thật? | Dựng panel là Story 1.14; làm ở đây là vượt phạm vi và dựng thứ Story 1.14 sẽ vứt đi | Nghiệm thu ở tầng token, ghi giới hạn vào Completion Notes theo tiền lệ `unmeasured` |

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, dev-story workflow)

### Debug Log References

**Nghiệm thu đỏ-rồi-xanh — 28/28 ca đạt.** Bộ điều khiển tiêm từng vi phạm vào cây nguồn thật, chạy `node scripts/check-tokens.mjs`, đọc mã thoát, khôi phục tệp, chạy lại. Nó sống ở thư mục scratchpad ngoài repo và luôn khôi phục trong `finally` — một lượt nghiệm thu để lại vi phạm trong repo còn tệ hơn không nghiệm thu. Nền được kiểm trước: cây sạch → exit 0.

| Kiểm | AC | Vi phạm cố ý | Tiêm vào → mã thoát | Gỡ ra → mã thoát |
|---|---|---|---|---|
| A | AC1 | đổi `colors.light.background` thành `#f4f1eb` (lệch 1 bit) | **1** 🔴 | 0 ✅ |
| A | AC1 | xoá token `error` khỏi theme sáng (16 → 15) | **1** 🔴 | 0 ✅ |
| A | AC1 | thêm token thứ 17 `accent-2` cho khớp một con số cũ | **1** 🔴 | 0 ✅ |
| A | AC1 | gỡ mục `deviations` nhưng giữ giá trị đã lệch | **1** 🔴 | 0 ✅ |
| B | AC2 | hex viết thẳng — `color: #ff0000` | **1** 🔴 | 0 ✅ |
| B | AC2 | cú pháp hàm — `background-color: rgb(43 39 35)` | **1** 🔴 | 0 ✅ |
| B | AC2 | không gian màu mới — `color: oklch(0.6 0.1 40)` | **1** 🔴 | 0 ✅ |
| B | AC2 | **tên màu CSS** — `border: 1px solid red` | **1** 🔴 | 0 ✅ |
| B | AC2 | giá trị không phải token màu — `color: var(--space-panel-block)` | **1** 🔴 | 0 ✅ |
| B2 | AC1 | cỡ chữ viết thẳng — `font-size: 13px` | **1** 🔴 | 0 ✅ |
| B2 | AC1 | họ chữ viết thẳng — `font-family: Georgia, serif` | **1** 🔴 | 0 ✅ |
| C | AC3 | khôi phục `surface-accent` tối về `#2c3a3b` (cặp trượt 4,245:1) | **1** 🔴 | 0 ✅ |
| C | AC3 | rút gọn danh sách kiểm — xoá cặp `on-surface-variant × surface-accent` | **1** 🔴 | 0 ✅ |
| C | AC3 | loại một cặp mà KHÔNG ghi lý do | **1** 🔴 | 0 ✅ |
| C | AC3 | né sàn bằng cách khai `"large"` mà không nêu token ≥ 24px | **1** 🔴 | 0 ✅ |
| C | AC3 | màu đã loại `#7d766c` quay lại `src/**` | **1** 🔴 | 0 ✅ |
| C | AC3 | `ornament` dùng làm màu chữ | **1** 🔴 | 0 ✅ |
| C | AC3 | `tm-rule` dùng làm màu chữ *(con số 5,26 ở theme tối là cái bẫy)* | **1** 🔴 | 0 ✅ |
| D | AC4 | `opacity: 0.4` cùng khối với `color:` | **1** 🔴 | 0 ✅ |
| D | AC4 | *chiều ngược:* miễn trừ `/* aura-allow-opacity: … */` thì KHÔNG được đỏ | **0** *(kỳ vọng 0)* | 0 ✅ |
| E | AC5 | khôi phục `lookup-gloss` về 1.6 trong khi `wraps: true` | **1** 🔴 | 0 ✅ |
| E | AC5 | token mới thiếu cờ `wraps` | **1** 🔴 | 0 ✅ |
| E | AC5 | *giới hạn đã biết:* khai `wraps: false` sai sự thật cho `read-lg` | **0** *(kỳ vọng 0)* | 0 ✅ |
| F | AC7 | `box-shadow` — kể cả khi dùng token màu hợp lệ | **1** 🔴 | 0 ✅ |
| F | AC7 | `linear-gradient` trong `background-image` | **1** 🔴 | 0 ✅ |
| F | AC7 | `z-index` trang trí | **1** 🔴 | 0 ✅ |
| G | AC6 | thống nhất hai theme về một cơ chế (`rule`) | **1** 🔴 | 0 ✅ |
| G | AC6 | khe theme tối tụt xuống 1px | **1** 🔴 | 0 ✅ |

#### Lượt hai — sau rà soát mã 2026-08-03: **52/52 ca đạt**

Bảng 28 hàng ở trên nghiệm thu *cổng như nó được đặc tả*. Lượt rà soát chỉ ra ba đường thoát và một loạt chỗ mù mà 28 hàng đó **không chạm tới** — vì chúng tiêm vi phạm vào đúng những chỗ cổng đang nhìn. Bộ nghiệm thu mới dựng lại một **sandbox sạch từ repo ở mỗi ca** (thay vì tiêm-rồi-khôi-phục tại chỗ), nên không ca nào để lại gì trong cây nguồn thật kể cả khi nó gãy giữa chừng. Bộ điều khiển sống ở thư mục scratchpad, ngoài repo.

Mười tám hàng dưới đây là **hàng mới**; 34 hàng còn lại là bảng cũ chạy lại trên cổng đã vá và vẫn đúng phán quyết.

| Kiểm | AC | Vi phạm cố ý — hàng MỚI | Trước lượt vá | Sau lượt vá |
|---|---|---|---|---|
| B/D/F | AC2·4·7 | 🔴 dấu nháy lẻ `don't` trong template + `opacity: 0.4` trên chữ + `box-shadow` trong `<style>` | **0** ⚠️ *(0 FAIL)* | **1** 🔴 |
| B | AC2 | 🔴 `/*` không đóng, rồi một màu viết thẳng sau đó | **0** ⚠️ | **1** 🔴 |
| B | AC2 | 🔴 `:style="{ color: 'red' }"` — binding động của Vue | **0** ⚠️ | **1** 🔴 |
| B2 | AC1 | 🔴 `:style="{ fontSize: '13px' }"` | **0** ⚠️ | **1** 🔴 |
| B | AC2 | 🔴 `style='color: rebeccapurple'` — nháy đơn | **0** ⚠️ | **1** 🔴 |
| B | AC2 | 🔴 màu hệ thống CSS — `background: Canvas` | **0** ⚠️ | **1** 🔴 |
| B | AC2 | 🔴 `<style>` trong `index.html` ở gốc repo | **0** ⚠️ *(ngoài tầm quét)* | **1** 🔴 |
| B | AC2 | 🔴 màu viết thẳng trong `src/**/*.js` | **0** ⚠️ | **1** 🔴 |
| B | AC2 | 🔴 màu viết thẳng trong `src/**/*.svg` | **0** ⚠️ | **1** 🔴 |
| B | AC2 | 🔴 ký tự ngoài BMP (emoji) làm lệch offset, rồi một màu viết thẳng | **1** *(may)* | **1** 🔴 |
| A | AC1 | 🔴 deviation **không có `reason`** *(và biến thể: không có `question`, `reason` rỗng)* | **0** ⚠️ | **1** 🔴 |
| A | AC1 | 🔴 giá trị màu ba chữ số `#abc` — `contrast()` ra `NaN`, `NaN < 4.5` là `false` | **0** ⚠️ | **1** 🔴 |
| C | AC3 | 🔴 hạ `contrast.floors.normal` xuống 3.0 **từ chính tệp bị kiểm** | **0** ⚠️ | **1** 🔴 |
| C | AC3 | 🔴 khôi phục `#2c3a3b` *(khớp bảng đóng băng ⇒ không cần deviation)* + hạ sàn | **0** ⚠️ *(in "31 cặp đạt AA · thấp nhất 4.245:1")* | **1** 🔴 |
| C | AC3 | 🔴 **CHUYỂN** cặp trượt sang `excluded` với lý do ba chữ | **0** ⚠️ | **1** 🔴 |
| C | AC3 | 🔴 `roles` và `pairs` rỗng — danh sách rỗng làm C1/C2 xanh rỗng | **0** ⚠️ *(in "đầy đủ: 0 tổ hợp · thấp nhất Infinity:1")* | **1** 🔴 |
| C | AC3 | 🔴 xoá `bannedColorValues` / xoá `neverTextTokens` | **0** ⚠️ | **1** 🔴 |
| C | AC3 | 🔴 `#7d766c` quay lại làm **GIÁ TRỊ của một token** trong `tokens.json` | **0** ⚠️ *(`.json` ngoài tầm quét)* | **1** 🔴 |
| D | AC4 | 🔴 `opacity: 0.4` trên thẻ **BỌC**, chữ con kế thừa màu | **0** ⚠️ | **1** 🔴 |
| D | AC4 | 🔴 `opacity: var(--dim)` — cổng không chứng minh được là 0 hay 1 | **0** ⚠️ | **1** 🔴 |
| E | AC5 | 🔴 `lineHeight: "20px"` — `parseFloat` nuốt đơn vị, 20 ≥ 1.66 | **0** ⚠️ *(khi đi qua deviation)* | **1** 🔴 |
| F | AC7 | 🔴 `z-index` không có miễn trừ **·** và chiều ngược: có miễn trừ → **0** ✅ | 1 🔴 / *(không có đường thoát)* | **1** 🔴 / **0** ✅ |
| G | AC6 | 🔴 `light.gap: 12px` — theme sáng có CẢ nét CẢ khe | **0** ⚠️ *(trường không được soi)* | **1** 🔴 |
| G | AC6 | 🔴 `dark.gap: 2rem` — đơn vị bị đánh tráo, `parseFloat` cho 2 | **0** ⚠️ | **1** 🔴 |
| G | AC6 | 🔴 `dark.borderWidth: 1px` + `borderColor: outline` — theme tối vẫn khai đường kẻ | **0** ⚠️ | **1** 🔴 |
| sàn | — | 🔴 `src/tokens/` phình lên 6 tệp trong khi component về 0 | **0** ⚠️ *(`FILE_FLOOR` vẫn qua)* | **1** 🔴 |

**Sáu hàng kỳ vọng exit 0 là dương tính giả ĐÃ SỬA, và chúng đắt ngang các hàng đỏ** — một cổng chỉ đường sai sẽ bị người sau thêm ngoại lệ cho tới khi nó không bắt được gì:

| Kiểm | Đầu vào hợp lệ | Trước | Sau |
|---|---|---|---|
| B | `document.querySelector('#faded')` trong `.ts` | **1** ⚠️ *(báo là màu viết thẳng)* | **0** ✅ |
| B | `color: var(--color-on-surface) !important` | **1** ⚠️ | **0** ✅ |
| B | `border-color: var(--a) var(--b)` *(CSS hợp lệ nhận 1–4 giá trị)* | **1** ⚠️ | **0** ✅ |
| D | `opacity: 100%` *(= hoàn toàn đục)* | **1** ⚠️ | **0** ✅ |
| hạ tầng | symlink trong `src/` | **1** ⚠️ *(`ENOENT` báo thành "cây nguồn không đọc được")* | **0** ✅ *(bỏ qua + ghi tên)* |
| B | `#dad` có `/* aura-allow-literal: … */` | *(chưa có kênh)* | **0** ✅ |

**Hai hàng kỳ vọng exit 0 ở bảng lượt một là cố ý, và chúng đo thứ mà 26 hàng còn lại không đo được:**
hàng D thứ hai chứng minh miễn trừ có tên **hoạt động** (một cổng không có đường thoát hợp lệ sẽ bị nới cho tới khi không bắt được gì); hàng E thứ ba ghi thẳng **giới hạn** — cờ `wraps` là một mệnh đề về nội dung sẽ chạy qua token, và không phép kiểm tĩnh nào phân xử được nó khi chưa có component.

**Đọc thẳng bảng `fvar` và `name` của bốn tệp font** *(Task 4 — bằng chứng loại khác với ảnh chụp)*:

```
SourceSans3[wght].ttf         name ID 1  = "Source Sans 3 ExtraLight"
                              name ID 16 = "Source Sans 3"
                              fvar wght: min 200 · MẶC ĐỊNH 200 · max 900
SourceSerif4[opsz,wght].ttf   fvar wght: min 200 · mặc định 400 · max 900
                              fvar opsz: min 8 · mặc định 20 · max 60
SourceSerif4-Italic[…].ttf    name ID 2 = "Italic", cùng family "Source Serif 4"
NotoSerifCJKtc-Regular.otf    KHÔNG có fvar — tệp tĩnh, một nét duy nhất
```

Hồi quy đầy đủ ở lượt giao: `check:deps` exit 0 · `check:tokens` exit 0 · `npm run build` (vue-tsc ×2 + vite) exit 0 · `cargo test --locked` **15/15 đạt**.

### Completion Notes List

#### 1. Cặp trượt AA — đi theo mặc định của story (phương án A), và Ice vẫn phải phê chuẩn

Đã **tự tính lại toàn bộ bảng tương phản** trước khi tin nó, bằng WCAG 2.x trên chính 16 giá trị của `DESIGN.md`. **Khớp story tới ba chữ số thập phân**, kể cả phát hiện chặn: `on-surface-variant #a29a8c` trên `surface-accent #2c3a3b` = **4,245:1**, dưới sàn 4,5.

Ice chưa trả lời khi dev bắt đầu ⇒ **phương án A**: `colors.dark.surface-accent` `#2c3a3b` → **`#283637`**. Số đo sau khi đổi:

| chữ trên `surface-accent` tối | trước | sau |
|---|---|---|
| `on-surface-variant` | 4,245 🔴 | **4,505** ✅ |
| `error` | 4,519 ⚠️ | **4,795** ✅ |
| `primary` | 5,103 | 5,415 |
| `confirmed` | 5,149 | 5,465 |
| `tm-text` | 5,857 | 6,216 |
| `on-surface` | 9,243 | 9,809 |

*(Story ước `primary` sau khi đổi là 5,55; số tính lại là **5,415**. Chênh lệch không đổi kết luận — cả hai đều qua sàn 4,5 — nhưng ghi ra để lượt rà soát sau không phải tự hỏi.)*

**`DESIGN.md` KHÔNG bị sửa.** Sửa tài liệu quy hoạch là quyết định của Ice, không phải hệ quả phụ của một lượt cài đặt (tiền lệ: quyết định #3 của Ice ở Story 1.3).

#### 2. 🔴 PHÁT HIỆN MỚI — AC5 còn hai token nữa trượt, và cờ `wraps` KHÔNG giải được

Story bắt được `DESIGN.md` tự mâu thuẫn ở `read-title` và `lookup-headword` (họ `read`, ở 1.3) và giải đúng bằng cờ `wraps` — tiêu đề một dòng không có dòng dưới để dấu chạm vào.

**Nhưng bảng token còn HAI token nữa dưới sàn 1.66, và cả hai thật sự xuống dòng:**

| Token | Vai | `lineHeight` bảng | `wraps` | Phán quyết |
|---|---|---|---|---|
| `lookup-gloss` | *Nghĩa* của Panel Lookup | **1.6** | `true` | trượt sàn |
| `lookup-example` | *Ví dụ và trích dẫn* | **1.6** | `true` | trượt sàn |

Đây là lần **thứ hai** `DESIGN.md` tự mâu thuẫn: §Giãn dòng phát biểu *"không token họ `read` nào được xuống dưới 1.66"* trong khi chính bảng ngay trên đặt hai token này ở 1.6.

**Đã nâng cả hai lên 1.66** và ghi vào `deviations`. Chi phí thị giác: 0,87px và 0,75px mỗi dòng. Đường thay thế duy nhất là khai `wraps: false` — tức nói dối cổng để cho xanh, đúng thứ AD-34 tồn tại để chặn. **Cần Ice phê chuẩn**, và mục này **chưa có trong §Câu hỏi cho Ice** của story.

#### 3. Chữ Hán nghiêng giả — `font-synthesis: none`, và nó đã được dựng thật

Chọn **`fontSynthesis: 'none'`** khai ở chính hai token `source-hanviet` và `lookup-example` (phát ra `--synthesis-<token>`).

**Lý do chọn đường này chứ không phải ba đường kia:** thêm tệp nghiêng CJK là ~23 MiB (không một phần ba ngân sách font, dư địa NFR6 còn ~47 MB); chấp nhận nghiêng giả thì `lookup-example` là ví dụ từ điển Trung–Việt ở 12,5px — đúng cỡ nó xấu nhất; `unicode-range` + `@font-face` riêng làm được nhưng phải bảo trì một dải mã CJK viết tay và không phủ được ký tự ngoài dải mình nghĩ ra. `font-synthesis: none` là **một thuộc tính, 0 byte, 0 bộ nhớ thêm, 0 dải mã phải bảo trì**.

**Đã dựng thật và chụp lại:** ở `auto`, 橫看成嶺側成峰 nghiêng rõ và nét bị méo; ở `none`, chữ Hán **đứng thẳng** trong khi phần Latin *"nghiêng Latin thật"* **vẫn nghiêng thật** — vì `Source Serif 4` có tệp Italic riêng nên không cần tổng hợp, và `font-synthesis` chỉ tắt phần *tổng hợp*.

⚠️ Nó cũng tắt tổng hợp **nét đậm** cho hai token đó. Tác dụng phụ này *mong muốn*: nếu descriptor nét sai ở đâu đó, thấy chữ mảnh (lỗi hiện ra) tốt hơn nét đậm giả (lỗi ẩn đi).

⚠️ **Lời giải này chưa có người tiêu thụ.** Nó chỉ có hiệu lực trong sản phẩm khi Story 1.16/1.17 áp `font-synthesis: var(--synthesis-<token>)` ở chính chỗ dựng hai token. Bỏ sót dòng đó là cách nó chết im lặng — đã ghi vào `deferred-work.md`.

#### 4. `Source Sans 3` bốn nét — mệnh đề "nét thật" nay đã kiểm tới đâu

✅ **Đã chứng minh:** dựng thật `Source Sans 3` ở **200 / 400 / 600 / 700** trên chuỗi dày dấu tiếng Việt (`Nghiêng ngửa ườ ộ ế`), bốn nét **phân biệt rõ**, `ui-label` (700) là **nét thật**. Mệnh đề của `DESIGN.md` trước đó chỉ mới phủ `Source Serif 4`.

✅ **Bằng chứng ở tầng tệp:** `fvar` trục `wght` = min 200 · **mặc định 200** · max 900, `name ID 1 = "Source Sans 3 ExtraLight"` — đọc thẳng từ tệp, khớp đúng thứ `ARCHITECTURE-SPINE.md §Stack` cảnh báo.

⚠️ **Nhưng một mệnh đề của tài liệu KHÔNG tái lập được, và mình ghi thẳng thay vì lờ đi:** đối chứng *thiếu* descriptor `{ weight: '200 900' }` — cùng một tệp, `@font-face` không khai dải nét — **vẫn ra nét đúng ở cả 400/600/700 trên Blink**. Blink đọc `fvar` và nội suy trục dù descriptor không khai. Nên:
- Descriptor **vẫn bắt buộc** — nó là thứ đặc tả CSS dựa vào, và `scopeCheck.ts:165,223` đã dùng.
- Nhưng mệnh đề *"thiếu nó thì chắc chắn ra chữ mảnh hoặc nét đậm giả"* **chưa đúng cho mọi engine**. WKWebView chưa đo.

#### 5. AC6 đóng được tới đâu — dùng lại tiền lệ `unmeasured`

**Đóng ở tầng token, KHÔNG ở tầng màn hình.** Story này không dựng panel (Story 1.14), nên *"nhìn thấy khe 2px"* là chưa đo được. Cái đã có:
- `panelSeparator.light` = `rule` (nét 1px `outline`, khe 0) · `panelSeparator.dark` = `gap` (khe 2px lộ `background`, panel bo 3px, **không** đường kẻ).
- `applyTheme` phát `--panel-separator-mechanism` · `--panel-gap` · `--panel-border-width` · `--panel-border-color` · `--panel-radius`.
- **Kiểm G** chặn việc hai theme bị thống nhất về một cách làm — đã nghiệm thu đỏ (2 ca).
- Trang thăm dò có dựng **hai panel giả** cạnh nhau ở cả hai theme để nhìn cơ chế; đó là minh hoạ, **không phải nghiệm thu** — panel thật thuộc Story 1.14.

#### 6. Đường nạp font — `deferred-work.md:7` đóng được MỘT NỬA

Story này đưa **đường nạp thật của sản phẩm** vào cây nguồn (`src/tokens/fonts.ts`: bốn `FontFace` qua `resolveResource()` → `convertFileSrc()`, đúng đường `check:scope:bundled` đã đo `LOADED` dưới CSP). Nhưng phần *nhìn thấy chữ hiện đúng nét trên Windows* vẫn cần một lượt runner có ảnh chụp — **bốn nét mới chỉ dựng trên Blink/macOS**. Đã ghi lại, **không đánh dấu đạt trọn**.

#### 7. Hai chỗ đọc `DESIGN.md` phải chọn bên — chọn theo BẢNG, đã ghi ra

- `source-cjk.family`: **bảng** ghi họ `read-cjk`, **frontmatter YAML** ghi `read`. Chọn **bảng** — `DESIGN.md` tự khai bảng là nguồn sự thật, và `read-cjk` mới đúng vai (nguyên văn tiếng Trung ở Panel Source, nơi không có chữ Latin để cần `Source Serif 4` đứng trước). Ghi ở `notes` của `tokens.json`; **không** phải deviation vì nó khớp bảng.
- `rounded.DEFAULT` giữ nguyên tên khoá trong JSON (khớp `DESIGN.md`) nhưng phát ra biến `--radius-default` viết thường cho khớp phần còn lại của quy ước.

#### 8. Một phép kiểm mà story không kê ra, và vì sao nó phải có

Story khẳng định `src/App.vue:37-38` *"sẽ làm Kiểm B/E đỏ ngay lượt đầu"*. **Không đúng như đặc tả:** hai dòng đó là `font-family` và `font-size` — Kiểm B kiểm màu, Kiểm E kiểm giãn dòng trong `tokens.json`. Cả hai đều không chạm tới cỡ chữ viết thẳng trong component.

Nhưng câu chuyện của story là *"mọi màu **và mọi cỡ chữ** đến từ một bộ token"*, nên **cổng thiếu đúng một nửa**. Đã thêm **Kiểm B2** — cùng lập luận AD-34, áp cho `font-family` · `font-size` · `line-height` · `font-weight` · `font-style` · `letter-spacing` · `font-synthesis`, và cấm thẳng shorthand `font` (nó không chở được `letter-spacing` với `font-synthesis`, nên một token đi qua nó chỉ chở nửa nghĩa mà bị dùng như thể chở đủ). B2 làm cây nguồn **đỏ ngay lượt đầu đúng như story dự đoán**, rồi Task 6 đưa về xanh.

#### 9. Hai chỗ cổng được làm chặt hơn đặc tả, có chủ ý

- **Kiểm C có phép kiểm ĐẦY ĐỦ (C1).** Story chỉ đòi tính tỉ lệ cho các cặp đã khai. Nhưng *"một danh sách kiểm tự rút gọn để cho xanh là đúng thứ AD-34 tồn tại để chặn"* — nên cổng cưỡng chế: mọi tổ hợp (7 chữ × 6 nền = **42**) phải nằm ở `pairs` (**31**) hoặc ở `excluded` **kèm lý do** (**11**). Xoá một cặp trượt khỏi danh sách kiểm giờ là FAIL, không phải một đường thoát.
- **Sàn 3:1 tồn tại nhưng bị khoá.** Một cặp muốn dùng sàn chữ lớn phải **nêu tên** một token cỡ ≥ 24px. Hôm nay **không cặp nào** dùng nó — tất cả 31 cặp đều qua ở sàn 4,5. Cơ chế có mặt và đã nghiệm thu đỏ, nhưng nó không phải cái van xả áp cho một cặp trượt.

### File List

**Mới**

| Đường dẫn | |
|---|---|
| `src/tokens/tokens.json` | Nguồn sự thật: 16×2 màu · 14 typography · 4 họ · spacing · rounded · 42 cặp tương phản · `panelSeparator` · sổ `deviations` |
| `src/tokens/index.ts` | `applyTheme()`, kiểu `Theme`/`ColorToken`/`TypographyToken`/`FamilyToken` |
| `src/tokens/fonts.ts` | `loadFonts()` — bốn `FontFace` qua asset protocol |
| `src/tokens/reset.css` | Reset toàn cục |
| `scripts/check-tokens.mjs` | Cổng bảy phép kiểm (A · B+B2 · C · D · E · F · G) |

**Sửa**

| Đường dẫn | |
|---|---|
| `src/main.ts` | `applyTheme('light')` **trước** `mount()` · import `reset.css` · gọi `loadFonts()` không chặn |
| `src/App.vue` | `.selftest` — ba giá trị viết thẳng → token (`:74-78`) |
| `src/tokens/README.md` | Viết lại: bốn tệp, quy ước biến, ba bẫy, số đo `fvar`. **Rà soát:** thêm §tầm quét, §"không phán quyết nào đọc tham số từ `tokens.json`", bảng **ba đường miễn trừ có tên** |
| `package.json` | thêm script `check:tokens` |
| `.github/workflows/ci.yml` | thêm **một** bước `check design tokens`, đặt **trước** `npm run build`; đánh dấu ✅ hàng Story 1.4 ở khối *CHỖ MÓC CHO EPIC SAU* |
| `_bmad-output/implementation-artifacts/deferred-work.md` | đóng mục reset CSS · hạ mục "đường nạp font trên Windows" xuống *đóng một nửa* · thêm mục của story này. **Rà soát:** đánh dấu Ice đã phê chuẩn ba deviation *(còn mở: `DESIGN.md` chưa sửa)* · thêm mục **ba mệnh đề thị giác đứng bằng văn xuôi** · thêm mục **`body` chạy ở giãn dòng 1.5** |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | `ready-for-dev` → `in-progress` → `review` → `done`. ⚠️ **Cùng tệp, cùng change set:** `1-5-tai-nguyen-chuoi-giao-dien-va-hinh-dang-loi-qua-ipc` `backlog` → `ready-for-dev` — do lượt `bmad-create-story` riêng, không thuộc story này; tệp story 1.5 tương ứng nằm ngoài File List này |
| chính tệp story này | |

**Lượt rà soát mã 2026-08-03 — 21 bản vá, đã áp toàn bộ**

| Đường dẫn | |
|---|---|
| `scripts/check-tokens.mjs` | Viết lại phần lớn. Đóng băng sàn WCAG · vai · danh sách `excluded` · danh sách màu đã loại vào chính script. Bộ che comment/chuỗi có chốt "chưa đóng" và đánh chỉ số UTF-16. Tầm quét lên 11 đuôi tệp + `index.html` ở gốc repo + `tokens.json` cho C3. Sàn thứ hai cho quần thể component. `deviations` đòi `question` + `reason`. Kiểm D nới rộng theo quyết định của Ice. Ba đường miễn trừ có tên. `walk()` bỏ qua symlink. Sáu dương tính giả đã sửa |
| `src/tokens/index.ts` | `applyTheme` kiểm tham số **lúc chạy** và rơi về `light` kèm cảnh báo, thay vì ném `TypeError` ở tầng module trước `mount()`; thêm `DEFAULT_THEME` và `isTheme()` |
| `src/tokens/fonts.ts` | `loadFonts()` chạy **song song** qua `allSettled`, hạn giờ 20 s mỗi tệp *(một promise treo và một promise thành công trông giống hệt nhau trong log)*, và **luỹ đẳng** bằng chốt ở tầng module |
| `src/tokens/reset.css` | Bốn khai báo tầng `body` có giá trị dự phòng trong `var()`, kèm lý do vì sao đây là chỗ **duy nhất** được phép; ghi thẳng giới hạn của mặc định `ui-md` (giãn dòng 1.5) |
| `src/tokens/tokens.json` | Thêm `notes`: `spacing.unit` đến từ **frontmatter** `DESIGN.md:127`, không từ bảng `:283` — ngoại lệ có chủ ý, không phải deviation |

**Không đụng, và đã kiểm bằng `git status`:** `src-tauri/**` · `Cargo.toml` · `src-tauri/tauri.conf.json` · `_bmad-output/planning-artifacts/**`.

**Không vào repo, có chủ ý** *(§Ranh giới phạm vi của mũi thăm dò Story 1.1: tài nguyên dùng một lần không vào cây nguồn)* — trang thăm dò thị giác, bộ sinh trang, bộ điều khiển nghiệm thu đỏ-rồi-xanh, bộ đọc `fvar`, và bốn ảnh chụp. Tất cả sống ở thư mục scratchpad của phiên làm việc; kết quả của chúng đã được chép vào §Debug Log References và §Completion Notes ở trên.

---

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-03 | **Rà soát mã (`bmad-code-review`) — Status → `done`.** Ba lớp song song, không lớp nào thất bại; 22 phát hiện còn lại sau triage, 7 bị loại là nhiễu. **Phần số học của story đứng vững:** lớp Acceptance Auditor tự tính lại toàn bộ WCAG 2.x trên chính giá trị trong `tokens.json` và **khớp tới ba chữ số thập phân**, phép đếm 16/16/14/4 đúng, 42 = 31 + 11 tổ hợp không sót cái nào, và cổng thật sự đỏ trên mọi vi phạm tiêm theo bảng Task 3. **Vấn đề nằm chỗ khác — cổng có ba đường thoát, và một dấu nháy lẻ tắt được nó.** Đã chạy thật, không suy luận: `<p>don't</p>` trong template làm `opacity: 0.4` trên chữ + `box-shadow` + `z-index` cùng lúc đi qua (3 FAIL → **0 FAIL**); hạ `contrast.floors` từ chính tệp bị kiểm + khôi phục `#2c3a3b` *(khớp bảng đóng băng nên không cần deviation nào)* cho ra `[dark] 31 cặp đạt AA · thấp nhất **4.245:1**` và exit 0; **CHUYỂN** cặp trượt sang `excluded` với lý do ba chữ cũng exit 0; một mục `deviations` không có trường `reason` cũng exit 0. **Ba quyết định của Ice:** (1) phê chuẩn cả ba deviation, `DESIGN.md` chưa sửa — ghi vào `deferred-work.md` cho một lượt riêng; (2) **nới rộng Kiểm D** — mọi `opacity` trung gian trong `src/**` là FAIL trừ khi có miễn trừ có tên, vì `.dimmed { opacity: .4 }` trên thẻ bọc lọt qua và đó là cách làm thông thường hơn trong Vue; (3) chấp nhận văn xuôi làm bằng chứng cho Task 4/5/9, kèm điều kiện ghi ba mệnh đề thị giác chưa tái lập được vào `deferred-work.md`. **21 bản vá đã áp toàn bộ.** Nguyên tắc xương sống mới: *không một phán quyết nào của cổng được đọc tham số từ `tokens.json`* — sàn WCAG, vai, danh sách loại trừ, danh sách màu đã loại đều đóng băng trong script. Kèm: bộ che comment/chuỗi có chốt "chưa đóng" và đánh chỉ số UTF-16; tầm quét lên 11 đuôi tệp + `index.html` + `tokens.json`; sàn thứ hai cho quần thể component; ba đường **miễn trừ có tên** (`aura-allow-opacity` · `aura-allow-z-index` · `aura-allow-literal`); `applyTheme` kiểm tham số lúc chạy; `loadFonts` song song + hạn giờ + luỹ đẳng; **sáu dương tính giả đã sửa** (`querySelector('#faded')`, `!important`, `border-color` nhiều giá trị, `opacity: 100%`, symlink, hex-hình-dạng có miễn trừ). Nghiệm thu **52/52 ca** *(18 hàng mới)*. Hồi quy: `check:tokens` 0 · `check:deps` 0 · `build` 0 |
| 2026-08-03 | **Cài đặt xong (`bmad-dev-story`) — chín Task, 45 subtask, Status → `review`.** Dựng `src/tokens/{tokens.json,index.ts,fonts.ts,reset.css}` + `scripts/check-tokens.mjs` (bảy phép kiểm, Node thuần, **không phụ thuộc mới**), gắn **một** bước vào `ci.yml` đã có. Nghiệm thu **đỏ-rồi-xanh 28/28 ca**. Hồi quy: `check:deps` 0 · `check:tokens` 0 · `build` 0 · `cargo test` 15/15. **Ba việc đáng chú ý nhất:** (1) 🔴 **phát hiện mới ngoài story** — `lookup-gloss` và `lookup-example` (cả hai `1.6`, cả hai thật sự xuống dòng) cũng trượt sàn AC5, và cờ `wraps` KHÔNG giải được vì chúng đúng là chữ chạy thành đoạn; đã nâng lên `1.66` và ghi vào `deviations` **chờ Ice phê chuẩn** — đây là lần **thứ hai** `DESIGN.md` tự mâu thuẫn với bảng của chính nó; (2) chữ Hán nghiêng giả giải bằng `font-synthesis: none` khai ở hai token — **0 byte**, và đã **dựng thật + chụp lại**: chữ Hán đứng thẳng trong khi Latin vẫn nghiêng thật; (3) `Source Sans 3` đã dựng ở 200/400/600/700 — `ui-label` là **nét thật**, mệnh đề của `DESIGN.md` nay đã kiểm, **nhưng** đối chứng thiếu descriptor **vẫn ra nét đúng trên Blink** nên bẫy `wght = 200` chưa tái lập được trên engine nào (ghi vào `deferred-work.md`, WKWebView chưa đo). Cặp trượt AA đi theo **mặc định phương án A** của story (`surface-accent` tối → `#283637`, đo lại 4,505); bảng tương phản của story đã **tự tính lại và khớp tới ba chữ số thập phân**. không `DESIGN.md` không bị sửa. Thêm **Kiểm B2** (cỡ chữ viết thẳng) vì hai dòng ở `App.vue:37-38` mà story nói *"sẽ làm Kiểm B/E đỏ"* thật ra không chạm phép kiểm nào như đặc tả — mà câu chuyện của story là *"mọi màu **và mọi cỡ chữ**"*. AC6 đóng ở **tầng token**, giới hạn ghi thẳng theo tiền lệ `unmeasured` |
| 2026-08-03 | Story dựng bằng `bmad-create-story`. Phân tích `epics.md` §Story 1.4 + chín mục UX-DR được `Covers` trỏ tới · `DESIGN.md` trọn vẹn (bốn bảng token, sàn tương phản, luật `opacity`, luật giãn dòng, phân tách panel, elevation) · `ARCHITECTURE-SPINE.md` (AD-34, Consistency Conventions, Stack, Structural Seed) · Story 1.2 và 1.3 (Review Findings, File List, Testing standards) · `deferred-work.md` (55 dòng) · trạng thái repo thật (`src/**`, `tauri.conf.json`, `ci.yml`, `package.json`, `scopeCheck.ts`, `App.vue`, `tsconfig.json`). **Bốn phát hiện mà tài liệu nguồn chưa có:** (1) 🔴 **`on-surface-variant` trên `surface-accent` ở theme tối = 4,245:1 — TRƯỢT WCAG AA**, và `error` trên cùng nền chỉ 4,519:1; `DESIGN.md §Sàn tương phản` chỉ kiểm trên nền giấy nên chưa bắt được — hai phương án sửa đã tính sẵn kèm số; (2) AC5 đọc thẳng sẽ làm `read-title` và `lookup-headword` (cả hai `read`, cả hai 1.3) đỏ oan — ranh giới thật là *chuỗi có xuống dòng hay không*, nên `tokens.json` cần cờ `wraps` tường minh; (3) font nằm ở `src-tauri/resources/`, **không** ở `src/` — một `@font-face url()` tương đối sẽ chạy trên máy dev nhưng nhân bản ~26 MiB vào `dist/`, chỉ lộ ở phép đo NFR6 của Story 1.9; (4) AC2 nói *"lint"* nhưng ESLint không có trong bảng Stack và NFR15 đòi rà giấy phép từng gói — khuôn đúng là một cổng Node thuần như ba cổng đang chạy. Tự kiểm phương pháp tính tương phản bằng cách tái lập con số 5,2:1 mà `DESIGN.md` công bố → tính ra 5,18 |
