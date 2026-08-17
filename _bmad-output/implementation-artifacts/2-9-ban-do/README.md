# Bàn đo Story 2.9 — `Backspace` ở đầu ô (Task 1, CHẶN Task 2)

Lưu ở đây để **không phải đo lại từ đầu**. Nó trả lời Task 1.2–1.5, và nó **xác nhận cạm bẫy
④ bằng một con số cụ thể** — xem §Kết luận.

⚠️ Đây là **tạo tác của một lượt đo**, không phải công cụ của dự án. Không có gì ở đây vào
`package.json` — đúng khuôn `2-2-ban-do/` · `2-4-ban-do/` · `2-5-ban-do/` · `2-5b-ban-do/` ·
`2-5d-ban-do/` · `2-8-ban-do/`.

## Chạy

```sh
TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-9-ban-do/caret-dau-o-vong2.e2e.mjs
```

Cần **cổng 1420 trống**. Ra: các khối JSON trên stdout, một khối một bước.

## Môi trường — Task 1.5

WebKit **605.1.15** (`AppleWebKit/605.1.15`, WKWebView thật của Tauri), macOS 24.6.0, cửa sổ
debug `--features wdio`. rustc/cargo **1.97.1** · Node **22.22.2** · vitest 4.1.10.
Đo **2026-08-17**. Vòng 2: **3 passing** (1m 06s).
Tự kiểm danh tính phiên: `href = http://localhost:1420/` · `#app` có mặt ✅ · 3 hàng ✅

---

## Vòng 1 — 🔵 **BỎ SỐ, GIỮ BÀI HỌC.** Thước hỏng, không phải engine hỏng

`backspace-dau-o-wkwebview.e2e.mjs`. Nó gửi `Backspace` bằng `browser.keys()` và
`browser.action('key')`, rồi đo được ở **cả năm** bước:

| Trường | Mọi bước |
|---|---|
| `isTrusted` | **`false`** |
| `document.hasFocus()` | **`false`** |
| `beforeinput` · `input` | **0** · **0** |
| `textContent` trước/sau | **không đổi một byte** |

🔴 **Đối chứng Ⓔ là chỗ bảng tự tố cáo.** Caret ở **GIỮA** ô (`startOffset: 3`) — một lượt xoá
lui tầm thường nhất trần đời — cũng **không xoá gì**. ⇒ Con số *"0 `beforeinput` ở offset 0"*
trả lời câu *"một phím KHÔNG TIN CẬY trong một tài liệu KHÔNG CÓ TIÊU ĐIỂM làm được gì"*,
**không** trả lời câu *"WebKit có phát `beforeinput` ở offset 0 không"*.

🔵 **Và giới hạn này ĐÃ CÓ CHỦ từ 2026-08-13** — `e2e/specs/editor-typing-flush.e2e.mjs:38-54`
ghi nguyên văn: *"`browser.keys()` KHÔNG GÕ ĐƯỢC CHỮ […] nó synthesize `keydown`/`keyup` và
không đi vào đường nhập văn bản gốc"*. Vòng 1 **đo lại một thứ đã ghi** thay vì đọc nó.
⇒ Bài học phương pháp: trước khi dựng một bàn đo, `grep` chính giới hạn mình sắp gặp.

⚠️ **Đây là một lượt sửa THƯỚC, không một vòng chẩn đoán bị bác** — LUẬT DỪNG (*"ba vòng chẩn
đoán bị phép đo bác ⇒ dừng"*) đếm những vòng mà một **giả thuyết về sản phẩm** bị bác. Tiền lệ:
`2-8-ban-do/` §Vòng 1 và `2-5d-ban-do/` §Debug Log Ⓐ.

**Ba số của vòng 1 SỐNG SÓT**, vì chúng là phép đọc JS thuần, không phụ thuộc độ tin cậy:

| Trường trên `keydown` | Giá trị | Vì sao nó quan trọng |
|---|---|---|
| `key` · `code` | `"Backspace"` · `"Backspace"` | Nhánh nhận diện được phím. **Khác** ca `⌘/` của 2.8, nơi driver giao `code: "/"` thay vì `"Slash"` |
| `cancelable` | **`true`** | `preventDefault()` **có nghĩa** trên sự kiện này |
| `repeat` | **`false`** ở một lượt bấm rời rạc | Chốt của chữ ký ③ (*"gộp một lần rồi dừng"*) **không phá ca thường** |

---

## Vòng 2 — `caret-dau-o-vong2.e2e.mjs`, và nó chỉ hỏi những câu ĐO ĐƯỢC

Caret đặt bằng **`document.caretRangeFromPoint`** — API mà 2.8 đã đo là **có thật** trên
WKWebView này *(khác `caretPositionFromPoint`, thứ NÉM `TypeError`)*. Nó cho đúng thứ một cú
bấm của người dùng cho, **không cần một sự kiện tin cậy nào**.

Xác nhận lại tại chỗ, vòng 2: `caretRangeFromPoint = "function"` · `caretPositionFromPoint =
"undefined"` ✅

### 🔴 Ⓐ+Ⓑ — engine biểu diễn *"đầu ô"* bằng `(node, offset)` nào

| Ca | `startContainer` | `startOffset` | `startOffset === 0`? | **ứng viên** | Đáp án đúng |
|---|---|---|---|---|---|
| Ⓐ① ô **RỖNG**, bấm giữa ô | `DIV` *(ELEMENT)* | 0 | ✅ true | **true** | đầu ô |
| Ⓐ② một dòng, mép **trái** | `#text`(11) | 0 | ✅ true | **true** | đầu ô |
| Ⓐ③ một dòng, **giữa chữ** | `#text`(11) | 2 | false | **false** | không |
| Ⓑ① hai dòng, mép trái **dòng 1** | `#text`(3) | 0 | ✅ true | **true** | đầu ô |
| 🔴 **Ⓑ② hai dòng, mép trái DÒNG 2** | **`#text`(3)** | **0** | 🔴 **true — SAI** | **false** | **không** |
| Ⓑ③ hai dòng, giữa chữ dòng 2 | `#text`(3) | 1 | false | **false** | không |
| Ⓒ vùng chọn **từ đầu ô**, không collapsed | `#text` | 0 | 🔴 **true — SAI** | **false** | **không** |

**Ứng viên đúng 7/7. Phép kiểm `startOffset === 0` sai 2/7.**

### 🔴 Ⓑ② là số quyết định, và nó xác nhận cạm bẫy ④ cụ thể hơn story đoán

Story viết *"caret ở đầu dòng thứ hai cũng cho `startOffset === 0`"* — **đúng**, và đây là cơ
chế: dưới `white-space: pre-line`, `execCommand('insertLineBreak')` của WebKit để lại **ba text
node** — `"AAA"`(3) · `"\n"`(1) · `"BBB"`(3), **không** một `<br>`, không một phần tử con nào
*(khớp Quyết định #1(d) của Story 2.5d)*.

Engine đặt caret ở đầu dòng 2 vào **offset 0 của text node THỨ BA**. ⇒ `startOffset === 0`
đúng, `startContainer` **không** phải node đầu, và một phép kiểm chỉ hỏi offset sẽ **gộp câu
khi người dùng chỉ muốn xoá một lần xuống dòng** — mất một segment vì một phép kiểm đọc thiếu
một node, trên một thao tác `AD-5` **không cho hoàn tác**.

Ứng viên trả `false` với `soKyTuPhiaTruoc = 4`, `chuPhiaTruoc = "AAA\n"`.

⚠️ **Và một phép kiểm ngược lại — *"startContainer là con ĐẦU của ô"* — cũng sai**, ở Ⓐ①: ô
rỗng cho `startContainer = chính ô` (`DIV`, 0 con). Không có phép kiểm nào **theo hình dạng**
đúng cả bốn; phép kiểm phải hỏi đúng câu định nghĩa.

### 🔴 Ⓓ — tiền đề ① xác nhận lại trên cây HÔM NAY, **có đối chứng dương**

| Lượt `execCommand('delete')` | trả về | `beforeinput` | `input` | `textContent` |
|---|---|---|---|---|
| caret **offset 0** | **`true`** | **`[]`** | **`[]`** | `"bốn năm sáu"` → **không đổi** |
| **đối chứng dương**, offset 3 | `true` | `["deleteContentBackward"]` | `["deleteContentBackward"]` | `"bốn năm sáu"` → `"bố năm sáu"` |

⇒ Thước **hoạt động**; con số ở offset 0 là mệnh đề về **engine**, không về bàn đo.
`onBeforeInput` (`GridPanel.vue:877`) **không dùng được** làm điểm móc, dù doc-comment của nó
gọi nó là *"cửa duy nhất mà một lượt sửa văn bản đi qua"* — câu đó vẫn đúng, chỉ không đúng cho
**ca này**. Đường còn lại là `keydown`.

🔴 **Một chi tiết đáng ghi riêng: `execCommand('delete')` trả `true` trong khi KHÔNG làm gì.**
Giá trị trả về của nó nói *"lệnh được nhận"*, không nói *"lệnh có tác dụng"*. Ai đọc nó thành
*"đã xoá"* sẽ có một lượt thành công không có thật — cùng lớp với *"rỗng IM LẶNG"*.

---

## Ba thứ bàn đo này **KHÔNG** đo được — ghi ra thay vì giả vờ

| Câu hỏi | Vì sao không đo được | Đi đâu |
|---|---|---|
| `preventDefault()` có chặn nổi lượt xoá của một phím **thật** không | Driver chỉ giao sự kiện `isTrusted: false`, và một sự kiện không tin cậy **không có default action** — nên phép kiểm sẽ trả **CÓ** trên mọi engine, kể cả engine không cho chặn | Món cho **Ice** *(kiểm tay)*. ⚠️ Rủi ro thật **thấp**: ở đúng offset 0 của một editing host, WebKit không có gì để xoá lui — Ⓓ đo được `textContent` không đổi. `preventDefault()` ở đây là lớp phòng thứ hai cho ca helper trả sai |
| Auto-repeat của hệ điều hành (`event.repeat === true`) | WebDriver `keyDown` giữ 600 ms qua Actions API cho **đúng một** `keydown`, `repeat: false`. Auto-repeat là hành vi tầng OS, driver không sinh ra | Món cho **Ice** *(kiểm tay)* — chữ ký ③ |
| `beforeinput` ở offset 0 với **phím vật lý thật** | Như trên | Đã có **hai** nguồn độc lập: `execCommand('delete')` ở Ⓓ *(hôm nay)* và Playwright-WebKit phím vật lý *(`deferred-work.md:3036-3061`, 2026-08-14)*. Không cần nguồn thứ ba |

⚠️ **Cả ba đều là giới hạn của BỘ ĐO, không của sản phẩm.** Chúng cùng lớp với *"không bộ chạy
test nào mô phỏng được một bộ gõ tiếng Việt thật"* — và cùng cách xử: một chữ ký của Ice **là**
đường nghiệm thu duy nhất.

---

## Kết luận — ba thứ Task 2 phải làm theo

1. **Móc ở `keydown`, không ở `beforeinput`** — Ⓓ, có đối chứng dương.
2. **Helper hỏi *"không còn ký tự nào phía trước caret trong cả ô"***, cài bằng một `Range` từ
   `(cell, 0)` tới caret rồi đo `toString().length`. **Không** hỏi `startOffset`, **không** hỏi
   node nào là con đầu — Ⓐ+Ⓑ, ứng viên 7/7 vs phép kiểm offset 2/7 sai.
3. **Vùng chọn không collapsed ⇒ trả `false` ngay** — Ⓒ, và nó là ca `startOffset === 0` sai
   thứ hai.
