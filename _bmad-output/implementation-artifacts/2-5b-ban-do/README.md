# Bàn đo Story 2.5b — lưới `subgrid` năm cột, ô bản dịch gõ được (Task 1, CỬA CHẶN)

Lưu ở đây để **không phải đo lại từ đầu**. Món nợ gốc: `deferred-work.md:2896-2908` (chủ 2.5b).

⚠️ Đây là **tạo tác của một lượt đo**, không phải công cụ của dự án. Không có gì ở đây vào
`package.json` — đúng khuôn `2-2-ban-do/` · `2-3-ban-do/` · `2-4-ban-do/` · `2-5-ban-do/`.

## Hai nhánh, hai vai KHÔNG thay nhau được

| Nhánh | Engine | Trả lời | Chạy |
|---|---|---|---|
| `../2-5b-ban-do-luoi.html` + `chup.mjs` | Chromium ≈ WebView2 · Playwright-WebKit | **Task 1.3** + vế bàn phím **vật lý** | Playwright |
| `luoi-wkwebview.e2e.mjs` | **WKWebView 605.1.15 THẬT** của Tauri | **Task 1.2** | bộ e2e, `--spec` |

🔴 **Playwright-WebKit KHÔNG thay được WKWebView.** Story 2.3 trả giá đúng chỗ này:
Playwright-WebKit **có** tạo vùng chọn ở lượt bấm, WKWebView **không**
(`EditorPanel.vue:464-479`) — nên một bàn đo Playwright cho lượt XANH trên một sản phẩm mà
chuột thật không dùng được.

🔴 Chiều ngược lại cũng đúng: **bộ e2e không thay được Playwright ở vế phím vật lý.**
`browser.keys()` chỉ bắn `keydown`/`keyup`, không đi vào đường nhập văn bản gốc
(`EditorPanel.vue:492-493`). Mệnh đề ③ vì thế có **hai** số, đo bằng **hai** đường, và
bảng dưới nói rõ số nào tới từ đường nào.

## Chạy

```sh
# ── Nhánh Playwright (Task 1.3) ────────────────────────────────────────────────
PW=/tmp/pw-aura; mkdir -p $PW && (cd $PW && npm init -y && npm i playwright@1.62.1)
(cd $PW && npx playwright install chromium webkit)          # một lần
AURA_PW=$PW/node_modules/playwright/index.mjs \
  node _bmad-output/implementation-artifacts/2-5b-ban-do/chup.mjs

# ── Nhánh WKWebView THẬT (Task 1.2) ────────────────────────────────────────────
TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-5b-ban-do/luoi-wkwebview.e2e.mjs
```

Ra: `2-5b-luoi-{blink,webkit}-{light,dark}.png` · `bao-cao.json` · log của spec.

## 🔴 `TAURI_WEBDRIVER_PORT=4467` KHÔNG phải trang trí — một lỗi hạ tầng ĐO ĐƯỢC

Lượt chạy đầu của nhánh WKWebView cho `document.activeElement =
BUTTON.sidebar-folder-tree__chevron` — một lớp CSS **không tồn tại trong kho này**
(`grep` toàn cây: 0 kết quả).

Nguyên nhân: máy chủ WebDriver nhúng **bám cổng cố định 4445** (`wdio.conf.mjs`), và trên
máy Ice cổng đó đang bị một tiến trình khác giữ (`gdrive-su`, PID 19811, `lsof -nP
-iTCP:4445 -sTCP:LISTEN`). ⇒ Phiên nối vào **webview của một ứng dụng khác**, và **mọi
phép đo vẫn chạy, vẫn trả về số**.

🔴 Đó là hình dạng hỏng tệ nhất có thể ở chỗ này: không một ca nào đỏ vì lý do đúng, và
mọi con số đọc ra như số thật. Cả hai phía đọc `TAURI_WEBDRIVER_PORT` (`wdio` service
`getEmbeddedPort` · crate `tauri-plugin-wdio-webdriver-1.3.0/src/lib.rs:24`), nên đường ra
là **dời cổng**, không phải giết tiến trình của người dùng.

⇒ Spec này mang một **phép tự kiểm danh tính phiên** chạy TRƯỚC mọi phép đo (`location.href`
phải là `localhost:1420` và `#app` phải có mặt). Bộ e2e thường trực **chưa có** phép kiểm
đó — món nợ có chủ, ghi ở `deferred-work.md`.

## Số đo — 2026-08-14

- **Nhánh WKWebView:** WebKit **605.1.15**, macOS 15.6, cửa sổ Tauri debug + `--features wdio`.
- **Nhánh Playwright:** Playwright **1.62.1** (Chromium · WebKit), macOS 15.6, Node 22.22.2.

### Năm mệnh đề của Task 1

| # | Mệnh đề | WKWebView 605.1.15 | Blink (Playwright) | Playwright-WebKit |
|---|---|---|---|---|
| ① | bấm chuột thật vào ô **rỗng** ⇒ caret | `contenteditable` **trần: KHÔNG** — `activeElement = SECTION.mode`, `selection = None`, `rangeCount 0`, **0 lượt `focusin`** *(cú bấm **có** trúng ô: `mousedown`/`mouseup`/`click` đều target đúng ô)*.<br>**Có đường chuột sản phẩm: ĐẠT** — `Caret`, `activeElement` = chính ô, thứ tự `mousedown → mouseup → focusin → selectionchange → click` | ĐẠT | ĐẠT |
| ② | gõ một ký tự vào ô rỗng | **ĐẠT** — `exec=true`, `beforeinput insertText` **huỷ được**, `textContent === 'A'` | ĐẠT | ĐẠT |
| ③ | `Backspace` ở offset 0 ⇒ `beforeinput deleteContentBackward` | 🔴 **KHÔNG** — **0** `beforeinput`, cả ở **đầu một ô CÓ CHỮ** lẫn ở **một ô đã rỗng** *(caret xác nhận `type = "Caret"`, `neo_trong_o = true`)* | ✅ **CÓ**, `deleteContentBackward` **huỷ được**, cả hai ca | 🔴 **KHÔNG** — khớp WKWebView |
| ④ | *"sập hố"* — ô rỗng có chiều cao thật | **ĐẠT** — ô rỗng **38,00 px** = ô có chữ **38,00 px** | ĐẠT — **38,81 px** = **38,81 px** | ĐẠT |
| ⑤ | `subgrid` giữ hàng thẳng khi hai ô lệch chiều cao | **ĐẠT** — lệch `top` lớn nhất **0 px**; chiều cao hàng `38 / 38 / 38 / 71` | **ĐẠT** — **0 px**; `38,81 / 50,81 / 38,81 / 106,44` | **ĐẠT** — **0 px**, số **khớp Blink từng chữ số** |

### 🔴 Ba kết luận, và một trong ba là một BẤT ĐỒNG ENGINE

**① `contenteditable` trần KHÔNG đủ trên WKWebView — đường chuột phải đi cùng.**
Đây là kết quả đắt nhất của lượt đo, và nó **không** hiển nhiên: mỗi ô nay là một editing
host **riêng**, nên cám dỗ là bỏ đường chuột *(«engine tự lo»)*. Số nói ngược: cùng họ với
Story 1.22-C2 *(WKWebView không focus `<button>`)* và Story 2.3 *(không focus `<span>`)*.
⇒ `GridPanel.vue` **giữ nguyên ba mảnh đã thắng của Story 2.3**: `setPosition` *(không
`addRange`)* · đặt caret ở `mouseup` *(không `mousedown`)* · một lượt vá ở frame kế tiếp.

**② Quyết định #6 *(không thư viện editor)* được phép đứng.** Ice ký nó **trước** khi có số
này; số này **không bác** nó. Ghi rõ phạm vi chữ ký: nó đứng cho *"`contenteditable` trần
**+ khuôn `EditorPanel.vue` đã chạy**"*, **không** cho *"không handler nào"*.

**③ 🔴 `Backspace` ở offset 0: HAI ENGINE NÓI NGƯỢC NHAU, và WebKit nói KHÔNG.**
Blink phát `deleteContentBackward` huỷ được kể cả khi **không có gì để xoá**; WebKit
*(cả WKWebView lẫn Playwright-WebKit)* **không phát gì cả**.

⚠️ Story 2.9 *(`Backspace` đầu ô = gộp với câu trên, UX-DR32)* dự định bắt cử chỉ đó ở
`beforeinput`. **Trên macOS đường đó KHÔNG tồn tại.** 2.5b chỉ dựng **tiền đề** *(ô là
editing host riêng)* nên nó không chặn story này, nhưng nó **lật một tiền đề** mà Quyết
định #3 của Task 0 đã viết ra bằng chữ *(«`Backspace` ở offset 0 sinh một `beforeinput`
`deleteContentBackward` bắt được ⇒ Story 2.9 có tiền đề»)*. ⇒ Món nợ **có chủ: Story 2.9**,
và đường còn lại là `keydown` **kèm chốt `event.isComposing`** — cùng lý do `EditorPanel.vue:841`.

## Giới hạn thật — ghi ra thay vì để người sau tự phát hiện

1. Cả hai nhánh **CHÉP** hình dạng DOM/CSS mà `GridPanel.vue` sẽ mang; không nhánh nào
   **mount** component thật. ⇒ một lượt sửa template sau này làm bàn đo và sản phẩm lệch
   nhau **mà không cổng nào đỏ**. Ca thường trực cho cùng địa hạt là **Task 12.2**.
2. Ba font nhúng của UX-DR4 **vắng mặt** ⇒ hình học rơi về `serif`/`Songti SC` hệ thống.
   Số hàng ở đây là số của **cơ chế**; chiều cao hàng thật của sản phẩm khác. Thứ **không**
   phụ thuộc font: `subgrid` có chia chung tập track hay không *(lệch 0 px)*, và ô rỗng có
   hộp hay không.
3. Mệnh đề ③ ở nhánh WKWebView đo bằng `execCommand('delete')`, **không** bằng phím vật lý —
   giới hạn của `browser.keys()`. Nhánh Playwright đo bằng **phím thật** và cho **cùng** câu
   trả lời trên WebKit *(0 `beforeinput`)*, nên hai đường **đồng ý**; nhưng vế *"bộ gõ tiếng
   Việt thật"* thì **không đường nghiệm thu nào của dự án mô phỏng được** — đó là **Task 1.4,
   chủ: ICE**.
4. Bàn đo **không** đo tương phản, **không** đo hiệu năng trên 9.850 hàng *(Task 8)*, và
   **không** trả lời *"ca này thường gặp tới đâu trên dữ liệu thật"*.
5. ⚠️ `Range.getBoundingClientRect()` của một range **thu gọn** trong một editing host
   **rỗng** trả **0×0** trên cả hai engine. Đó là hành vi của **API**, không phải chiều cao
   caret người dùng nhìn thấy — đừng đọc số 0 đó thành *"caret sập hố"*. Mệnh đề ④ vì thế đo
   bằng **chiều cao của chính ô**, thứ đo được và cũng chính là thứ chữa khuyết tật.
   *(Bản đầu của bàn đo đọc nhầm đúng chỗ này, và nó suýt bác oan Quyết định #3.)*
6. ⚠️ Bản đầu còn cắm listener `beforeinput` trên **ô** và bao quanh một lời gọi
   `execCommand`; nó cho `beforeinput: null` trên Blink **kể cả khi chữ đã hạ cánh**. Sửa:
   ghi ở `document` + pha **capture**. Đúng lớp lỗi *"bàn đo cũng sai được"* mà bàn đo Story
   2.5 đã ghi tên.

## 🔵 CẬP NHẬT 2026-08-15 — bàn đo này NÓI THIẾU một biến, và e2e bắt được

Bàn đo dựng đúng hình dạng lưới, nhưng nó dựng trong một **lớp phủ không có tổ tiên nào focus
được**. Sản phẩm thì có: `PanelFrame` dựng một `section[tabindex="-1"]` cho AD-34 §2, và
`WorkspaceDock` gọi `enterFocus()` khi panel được kích hoạt bằng chuột.

⇒ Mệnh đề ① của bàn đo — *"có đường chuột sản phẩm thì caret ĐẠT"* — **đúng trong bàn đo và
KHÔNG đủ trong sản phẩm**. Ca thật đo được ở `e2e/specs/grid-empty-cell.e2e.mjs`: **cú bấm đầu
tiên** vào panel mất caret *(`activeElement = SECTION.panel.focused`, `type = "None"`)*, cú thứ
hai thì ăn.

🔴 **Giới hạn ⑥ của bàn đo, thêm vào danh sách trên:** nó **không** dựng lại ngữ cảnh panel
*(dockview + `PanelFrame` + hợp đồng tiêu điểm)*. Mọi mệnh đề về **tiêu điểm** đo ở đây là mệnh
đề về **engine**, không về **sản phẩm**. Vế sản phẩm chỉ e2e trả lời được.

⚠️ Đây là cùng lớp bài học *"trúng tiền đề chưa phải trúng cơ chế"* mà §Bài học của story ghi —
lần này bàn đo trúng **tiền đề** *(hình dạng lưới cho caret một hộp thật)*, và e2e bắt được
**cơ chế** *(ai giành tiêu điểm, và lúc nào)*.
