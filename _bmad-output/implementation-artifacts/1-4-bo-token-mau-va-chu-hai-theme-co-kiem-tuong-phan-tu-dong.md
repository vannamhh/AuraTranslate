---
baseline_commit: 0255163d7c2406c83cdfe83b4b05f1fa188bc0cb
---

# Story 1.4: Bộ token màu và chữ hai theme, có kiểm tương phản tự động

Status: ready-for-dev

Epic: 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
Covers: NFR17 · AD-34 · UX-DR1, UX-DR2, UX-DR3, UX-DR5, UX-DR6, UX-DR10, UX-DR11, UX-DR14, UX-DR16

> ⚠️ **Story 1.3 vẫn `in-progress` lúc story này được dựng** — bốn phép nghiệm thu của nó chờ một lượt runner thật (`deferred-work.md:28-32`). Điều đó **không chặn** story này: `ci.yml` đã tồn tại và chạy được ở máy, và Task 8 chỉ thêm **một** bước vào job đã có. Nhưng nếu lượt CI đầu tiên buộc `ci.yml` phải sửa cấu trúc, bước của Task 8 đi theo — ⛔ đừng dựng một workflow riêng để né va chạm.

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

> ⚠️ **16, không phải 17.** `DESIGN.md §Vì sao 16 chứ không phải 17` cảnh báo tường minh: `tm-rule` giữ **cùng một giá trị ở cả hai theme** nên dễ bị đếm thành hai. ⛔ **Không được bịa thêm một token thứ 17 để cho khớp một con số cũ** — con số 17 đã bị sửa thành 16 ở `UX-DR1` và ở AC của story trước ngày 2026-08-03.

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

- [ ] **Task 1 — Dựng nguồn sự thật một tệp cho toàn bộ token** (AC: 1)
  - [ ] Tạo `src/tokens/tokens.json` chứa: `colors.light` (16) · `colors.dark` (16) · `typography` (14) · `families` (4) · `spacing` · `rounded`. Giá trị chép từ **`DESIGN.md` §Bảng token màu / §Bảng token typography / §Bảng token khoảng cách và hình dạng**, không chép từ frontmatter YAML *(frontmatter và bảng khớp nhau hôm nay, nhưng bảng là thứ `DESIGN.md` tự khai là "nguồn sự thật")*
  - [ ] ⛔ **Đúng một nguồn sự thật.** Không tạo song song một tệp `.css` viết tay mang cùng giá trị — hai bản chép sẽ lệch nhau ở lần sửa thứ ba. `tsconfig.json` đã bật `resolveJsonModule` nên TS đọc trực tiếp, và `.mjs` đọc bằng `JSON.parse` — cùng một tệp, hai người tiêu thụ
  - [ ] Tạo `src/tokens/index.ts`: import typed + hàm `applyTheme(theme: 'light' | 'dark')` ghi CSS custom properties lên `document.documentElement`. Gọi **trước `mount()`** trong `main.ts` để không có nháy màu
  - [ ] Quy ước tên biến CSS: `--color-<token>` · `--font-<token>` · `--family-<token>` · `--space-<token>` · `--radius-<token>`. Chốt một lần, 127 story sau dùng lại

- [ ] **Task 2 — Cổng cưỡng chế `scripts/check-tokens.mjs`** (AC: 1, 2, 3, 4, 5, 7)
  - [ ] Kiểm A (AC1): đối chiếu `tokens.json` với một **bảng kỳ vọng đóng băng trong chính script** — hai bản chép độc lập phải khớp. Đếm đúng 16/16/14/4. Cùng khuôn với `BANNED_CRATES` của `check-deps.mjs` và allowlist của `config_invariants.rs`
  - [ ] Kiểm B (AC2): quét `src/**/*.{vue,ts,css}` tìm giá trị màu viết thẳng — `#rgb` · `#rrggbb` · `#rrggbbaa` · `rgb()` · `rgba()` · `hsl()` · `hsla()` · `color()` · **và tên màu CSS** (`red`, `white`, `black`, …). Miễn trừ: chính `src/tokens/**`. ⛔ Chỉ quét hex là bỏ lọt bốn cú pháp
  - [ ] Kiểm C (AC3): tính tỉ lệ tương phản WCAG 2.x cho **mọi cặp đã khai** ở §Bảng tương phản, cả hai theme. Sàn 4.5:1; 3:1 chỉ cho token cỡ ≥ 24px (`lookup-headword`). Thêm phép chặn: `#7d766c` vắng mặt hoàn toàn; `ornament` và `tm-rule` **không xuất hiện sau `color:`**
  - [ ] Kiểm D (AC4): quét `opacity:` trong `src/**` — giá trị **khác 0 và khác 1** trên một khai báo cùng khối với `color:` là FAIL. Miễn trừ có tên: khai báo mang comment `/* aura-allow-opacity: <lý do> */`
  - [ ] Kiểm E (AC5): mọi token họ `read` có `lineHeight` ≥ **1.66**. Token họ `ui` mang cờ `wraps: true` cũng ≥ 1.66
  - [ ] Kiểm F (AC7): quét `box-shadow` · `text-shadow` · `filter: drop-shadow` · `linear-gradient` · `radial-gradient` · `z-index` trong `src/**` — có là FAIL
  - [ ] ⚠️ **Mã thoát là phán quyết.** In cảnh báo rồi `exit 0` là một cổng không cưỡng chế được gì (§Testing standards)
  - [ ] Thêm `"check:tokens": "node scripts/check-tokens.mjs"` vào `package.json`

- [ ] **Task 3 — Nghiệm thu đỏ-rồi-xanh cho cả sáu phép kiểm** (AC: 1, 2, 3, 4, 5, 7)
  - [ ] Với **từng** kiểm A–F: cố ý tạo một vi phạm → chạy → phải **đỏ**; gỡ vi phạm → phải **xanh**. Ghi bảng kết quả vào §Debug Log References
  - [ ] ⛔ Một cổng chưa từng đỏ là một cổng chưa được chứng minh. Story 1.3 §Task 11 đã đặt tiền lệ này

- [ ] **Task 4 — Nạp font lúc chạy, và kiểm bốn nét của `Source Sans 3`** (AC: 1)
  - [ ] Tạo `src/tokens/fonts.ts`: đăng ký bốn `FontFace` từ `$RESOURCE/fonts/**` qua `resolveResource()` + `convertFileSrc()`, y hệt đường `scopeCheck.ts:220-242` đã chứng minh chạy được dưới CSP
  - [ ] ⛔ Descriptor **bắt buộc** cho hai tệp biến thiên: `{ weight: '200 900' }`. Thiếu nó thì `Source Sans 3` khoá ở `wght = 200` (ExtraLight) và `ui-label` 700 ra chữ mảnh — hoặc tệ hơn, trình duyệt tổng hợp nét đậm giả
  - [ ] `SourceSerif4[opsz,wght].ttf` + `SourceSerif4-Italic[opsz,wght].ttf` → cùng `family: 'Source Serif 4'`, khác `style`. `NotoSerifCJKtc-Regular.otf` → `'Noto Serif CJK TC'`, chỉ `weight: '400'`
  - [ ] **Dựng thật `Source Sans 3` ở 400 / 600 / 700** trên một trang thăm dò rồi chụp lại. Mệnh đề *"nét 600 và 700 là nét thật"* của `DESIGN.md` **chỉ mới được chứng minh cho `Source Serif 4`** — `Source Sans 3` chưa từng dựng quá một nét
  - [ ] Kiểm bằng **chuỗi dày dấu tiếng Việt** (`ế ộ ữ ẳ ườ` — UX-DR10) chứ không bằng chữ Latin

- [ ] **Task 5 — Xử lý chữ Hán nghiêng giả ở hai token** (AC: 1)
  - [ ] Hai token dính: `source-hanviet` (#6) và `lookup-example` (#10) — cả hai `italic`, cả hai họ `read`, mà `families.read` có `Noto Serif CJK TC` trong chuỗi dự phòng và tệp đó **chỉ có Regular**
  - [ ] Chọn một trong hai đường `DESIGN.md` đã liệt: (a) chấp nhận nghiêng giả cho phần Hán; (b) khai `font-style: normal` cho ký tự CJK trong hai token đó, qua `unicode-range` hoặc một `@font-face` riêng
  - [ ] ⛔ **Thêm một tệp nghiêng CJK KHÔNG phải phương án** — đó là ~23 MiB, một phần ba ngân sách font, và dư địa NFR6 chỉ còn ~47 MB
  - [ ] Ghi lựa chọn **và lý do** vào §Completion Notes. Đây là quyết định thị giác, không có đáp án máy chấm được

- [ ] **Task 6 — Reset CSS toàn cục** (AC: 7)
  - [ ] Tạo `src/tokens/reset.css`, import trong `main.ts`. Đóng mục `deferred-work.md:20`: `body` có margin 8px mặc định + `.shell { min-height: 100vh }` ⇒ **thanh cuộn ở một cửa sổ trống**
  - [ ] Tối thiểu: `box-sizing: border-box` toàn cục · `body { margin: 0 }` · `background`/`color` từ token · `-webkit-font-smoothing` nhất quán hai nền tảng
  - [ ] ⛔ Không kéo `normalize.css`/`modern-css-reset` về — mỗi phụ thuộc mới phải rà GPLv3 và vào bảng Stack **trước** (NFR15)
  - [ ] Sửa `src/App.vue`: bỏ `font-family` và `font-size` viết thẳng ở `.selftest` (`:37-38`) — chúng sẽ làm Kiểm B/E đỏ ngay lượt đầu. Chuyển sang `var(--family-mono)` + `var(--font-ui-mono)`

- [ ] **Task 7 — Token phân tách panel hai theme** (AC: 6)
  - [ ] Story này **không** dựng panel (đó là Story 1.14). Nó khai **cơ chế** ở tầng token: theme sáng → `--panel-separator: 1px solid var(--color-outline)`, khe 0; theme tối → khe `2px` lộ `background`, panel `border-radius: 3px`, không đường kẻ
  - [ ] ⛔ Không thống nhất hai theme về một cách làm. `outline #3b382f` trên `surface #26241f` chỉ đạt **1,32:1** — mình đã tính lại, khớp con số 1,39 mà `DESIGN.md` ghi ở dải làm tròn khác. Gần như vô hình ở cả hai cách tính
  - [ ] Thêm một phép kiểm vào `check-tokens.mjs`: hai theme phải khai **hai cơ chế khác nhau**, giống nhau là FAIL

- [ ] **Task 8 — Gắn vào pipeline đã có** (AC: 2, 3)
  - [ ] Thêm **một** bước `npm run check:tokens` vào `.github/workflows/ci.yml`, trong job `check` đã có
  - [ ] ⛔ **Không dựng pipeline thứ hai** — AC4 của Story 1.3 cấm tường minh, và §AC4 của tệp đó đã chừa sẵn chỗ móc mang tên *"lint cấm màu viết thẳng (AD-34)"* cho đúng story này
  - [ ] Đặt bước **trước** `npm run build` — nó chạy trong vài giây và không cần `dist/`, nên một lỗi token nên đỏ trước khi tốn một lượt biên dịch Rust
  - [ ] ⚠️ Bước này **không** cần cửa sổ đồ hoạ. Đừng đặt nó xuống cụm cuối cùng nơi hai phép kiểm cần webview đang đứng

- [ ] **Task 9 — Trang thăm dò thị giác, và gỡ nó đi** (AC: 1, 5)
  - [ ] Dựng tạm một trang bày cả 14 token typography trên cả hai theme, dùng **chuỗi dày dấu tiếng Việt** + một đoạn chữ Hán + một dòng Latin, để kiểm mắt ba hệ chữ có cân nhau không
  - [ ] Chụp lại làm bằng chứng, ghi vào §Completion Notes, rồi **gỡ trang đó khỏi cây nguồn**. Cùng khuôn với §Ranh giới phạm vi của mũi thăm dò Story 1.1: tài nguyên dùng một lần không vào repo

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
| Xử lý chữ Hán nghiêng giả (Task 5) | Thêm tệp font nghiêng CJK — ⛔ ~23 MiB |
| Công tắc theme ở tầng hàm (`applyTheme`) | Giao diện chọn theme, lưu lựa chọn xuống đĩa — **Story 1.8** |

⛔ **Không đụng tới:** `src-tauri/**` · `Cargo.toml` · `src-tauri/tauri.conf.json` · `_bmad-output/planning-artifacts/**` *(trừ khi Ice quyết theo §🔴 Phát hiện chặn, và khi đó ghi rõ trong File List — Story 1.3 đã bị bắt vì khai sai đúng dòng này)*.

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

> ⚠️ **Đường thứ ba, và nó cũng hợp lệ:** khai rằng cặp `on-surface-variant` × `surface-accent` **không bao giờ được dùng** trong ứng dụng, ghi mệnh đề đó thành một dòng trong `tokens.json` và cho `check-tokens.mjs` cưỡng chế nó. AC3 nói *"mọi cặp **dùng trong ứng dụng**"* — một cặp không dùng thì không phải cặp. ⛔ Nhưng phải **viết ra**, không được im lặng bỏ cặp đó khỏi danh sách kiểm: một danh sách kiểm tự rút gọn để cho xanh là đúng thứ AD-34 tồn tại để chặn.
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
Kiểm A (đếm 16/16/14/4) là phần dễ và cũng là phần yếu nhất. Thứ AC3 thật sự đòi là **cặp màu dùng trong ứng dụng** đạt AA — mà hôm nay ứng dụng chưa có component nào. Nên cổng phải kiểm **danh sách cặp đã khai** (bảng trên), và khi Story 1.14 dựng panel thật, chính cổng đó sẽ bắt cặp mới. ⛔ Đừng viết một cổng chỉ đúng cho hôm nay.

**2. 🔴 `Source Sans 3` khoá ở `wght = 200` nếu thiếu descriptor.**
`ARCHITECTURE-SPINE.md §Stack` ghi thẳng: tệp đó có `name ID 1 = Source Sans 3 ExtraLight` vì **mặc định trục `wght = 200`**. `ui-label` khai 700. Không có `{ weight: '200 900' }` thì hoặc ra chữ mảnh, hoặc trình duyệt tổng hợp nét đậm giả — và ở cỡ 10px với `letter-spacing 0.1em`, nét giả trông *gần đúng*, đủ gần để không ai nhận ra trong sáu tháng. `scopeCheck.ts:165,223` đã dùng đúng descriptor này; chép lại, đừng phát minh.

**3. 🔴 Font KHÔNG nằm trong `src/` — đừng `@font-face { src: url('./fonts/…') }`.**
Bốn tệp font sống ở `src-tauri/resources/fonts/`, đi vào bản cài qua `bundle.resources`, và tới được webview qua **asset protocol**. Đường duy nhất đã được chứng minh chạy dưới CSP là `resolveResource()` → `convertFileSrc()` → `new FontFace(...)`. Một `url()` tương đối trong CSS sẽ được Vite giải thành asset của bundle — nghĩa là **font bị nhân bản vào `dist/`**, cộng thẳng ~26 MiB vào payload mà NFR6 chỉ còn ~47 MB dư địa. Nó lại còn *chạy được* trên máy dev, nên bẫy này chỉ lộ ra ở phép đo dung lượng của Story 1.9.

**4. 🔴 `fetch()` tới asset protocol GÃY ở bản đóng gói.**
CSP hiện tại: `connect-src 'self' ipc: http://ipc.localhost` — **không có `asset:`**, trong khi `font-src` **có**. Đo thật trên bản `.app` debug ngày 2026-08-03, bốn sự kiện `securitypolicyviolation` nêu đích danh `connect-src`. Ice đã chốt **giữ nguyên CSP**. ⇒ `FontFace` chạy, `fetch()` không. Nếu bạn thấy mình cần `fetch` một tệp font để kiểm tra gì đó, bạn đang đi vào đúng đường đã đo là hỏng.

**5. 🔴 Kiểm màu viết thẳng chỉ quét hex là bỏ lọt bốn cú pháp.**
`rgb(43 39 35)` · `hsl(30 10% 15%)` · `color(display-p3 …)` · và **tên màu CSS** đều là màu viết thẳng. Kèm theo: quét phải **bỏ qua comment và chuỗi i18n** — nhưng ⛔ **không** bỏ qua bằng cách nới regex, mà bằng cách quét đúng vùng khai báo CSS. Một cổng dương tính giả sẽ bị người sau thêm ngoại lệ cho tới khi nó không bắt được gì.

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

### Debug Log References

<!-- Bảng nghiệm thu đỏ-rồi-xanh của Task 3 — sáu hàng, ghi ở đây -->

### Completion Notes List

<!--
Bắt buộc ghi:
  - Phương án đã chọn cho cặp trượt AA (Câu hỏi 1) và số đo sau khi sửa
  - Lựa chọn cho chữ Hán nghiêng giả (Task 5) và LÝ DO
  - Ảnh chụp `Source Sans 3` ở 400/600/700 — mệnh đề "nét thật" nay đã kiểm tới đâu
  - AC6 đóng được tới đâu, và giới hạn ghi thẳng
  - Đường nạp font đã chạy trên nền tảng nào — `deferred-work.md:7` đóng được một nửa hay trọn
-->

### File List

<!-- Điền khi cài đặt xong. Dự kiến:

Mới:  src/tokens/tokens.json · src/tokens/index.ts · src/tokens/fonts.ts ·
      src/tokens/reset.css · scripts/check-tokens.mjs
Sửa:  src/main.ts · src/App.vue · src/tokens/README.md · package.json ·
      .github/workflows/ci.yml · deferred-work.md · sprint-status.yaml · chính tệp này
⛔ Không đụng: src-tauri/** · Cargo.toml · _bmad-output/planning-artifacts/**
-->

---

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-03 | Story dựng bằng `bmad-create-story`. Phân tích `epics.md` §Story 1.4 + chín mục UX-DR được `Covers` trỏ tới · `DESIGN.md` trọn vẹn (bốn bảng token, sàn tương phản, luật `opacity`, luật giãn dòng, phân tách panel, elevation) · `ARCHITECTURE-SPINE.md` (AD-34, Consistency Conventions, Stack, Structural Seed) · Story 1.2 và 1.3 (Review Findings, File List, Testing standards) · `deferred-work.md` (55 dòng) · trạng thái repo thật (`src/**`, `tauri.conf.json`, `ci.yml`, `package.json`, `scopeCheck.ts`, `App.vue`, `tsconfig.json`). **Bốn phát hiện mà tài liệu nguồn chưa có:** (1) 🔴 **`on-surface-variant` trên `surface-accent` ở theme tối = 4,245:1 — TRƯỢT WCAG AA**, và `error` trên cùng nền chỉ 4,519:1; `DESIGN.md §Sàn tương phản` chỉ kiểm trên nền giấy nên chưa bắt được — hai phương án sửa đã tính sẵn kèm số; (2) AC5 đọc thẳng sẽ làm `read-title` và `lookup-headword` (cả hai `read`, cả hai 1.3) đỏ oan — ranh giới thật là *chuỗi có xuống dòng hay không*, nên `tokens.json` cần cờ `wraps` tường minh; (3) font nằm ở `src-tauri/resources/`, **không** ở `src/` — một `@font-face url()` tương đối sẽ chạy trên máy dev nhưng nhân bản ~26 MiB vào `dist/`, chỉ lộ ở phép đo NFR6 của Story 1.9; (4) AC2 nói *"lint"* nhưng ESLint không có trong bảng Stack và NFR15 đòi rà giấy phép từng gói — khuôn đúng là một cổng Node thuần như ba cổng đang chạy. Tự kiểm phương pháp tính tương phản bằng cách tái lập con số 5,2:1 mà `DESIGN.md` công bố → tính ra 5,18 |
