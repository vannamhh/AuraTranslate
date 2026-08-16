# Bàn đo Story 2.5d — `Enter` xuống dòng trong ô bản dịch (Task 1, CHẶN Task 3)

Lưu ở đây để **không phải đo lại từ đầu**. Nó trả lời Task 1.2 *(năm mệnh đề)*, và nó đã
**lật một chữ ký của Ice** — xem §Kết luận ③.

⚠️ Đây là **tạo tác của một lượt đo**, không phải công cụ của dự án. Không có gì ở đây vào
`package.json` — đúng khuôn `2-2-ban-do/` · `2-4-ban-do/` · `2-5-ban-do/` · `2-5b-ban-do/`.

## 🔵 Bàn đo này chạy trên LƯỚI THẬT — một giới hạn của bốn bàn đo trước đã được GỠ

Bốn bàn đo trước đều **tiêm một hình dạng DOM chép tay**, và cả bốn mang cùng một giới hạn
(`2-5b-ban-do/README.md` §Giới hạn ①): *"một lượt sửa template sau này làm bàn đo và sản
phẩm lệch nhau **mà không cổng nào đỏ**"*.

Lượt này không cần chép: `GridPanel.vue` đã ở trong sản phẩm từ Story 2.5b. Bàn đo mở Tác
phẩm thật, bấm **chuột thật** vào ô thật, và hỏi engine trên **chính DOM người dùng chạm** —
mang cả `<style scoped>` thật, cả `subgrid` thật, cả hợp đồng tiêu điểm thật.

**Giới hạn mới thay vào chỗ đó, nhẹ hơn nhưng có thật:** để nhìn được engine làm gì khi
không ai chặn, bàn đo phải **vô hiệu hoá hai lớp chặn `Enter` đang sống trong sản phẩm**. Nó
làm bằng **thứ tự sự kiện**, không bằng một lượt sửa mã: hai lớp chặn đăng ký trên **cột**
(`GridPanel.vue:955-957`, pha nổi bọt), nên một listener ở `document` pha **capture** chạy
trước và `stopPropagation()` cắt trọn đường xuống — handler sản phẩm không chạy, còn hành vi
mặc định của engine **vẫn chạy**. Lượt chạy kết thúc bằng một số **đối chứng** với chặn bật
lại.

## Chạy

```sh
# 🔴 Cổng 4445 trên máy Ice bị `gdrive-su` giữ (đo 2026-08-15, PID 48486) — PHẢI dời cổng.
TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-5d-ban-do/insertparagraph-wkwebview.e2e.mjs
```

Ra: bốn khối JSON trên stdout, một khối một vòng. Cần **cổng 1420 trống**.

## Số đo — 2026-08-15

WebKit **605.1.15** *(`AppleWebKit/605.1.15`, WKWebView thật của Tauri)*, macOS, cửa sổ debug
`--features wdio`. Node 22.22.2. Bốn vòng, **1 passing** mỗi vòng.
Tự kiểm danh tính phiên: `href = http://localhost:1420/`, `#app` có mặt ✅

### Vòng 1 — năm mệnh đề của Task 1.2

| # | Mệnh đề | Số đo |
|---|---|---|
| ① | `Enter` qua `browser.keys()` | **0** `beforeinput`; chỉ một `keydown key="Enter"`. *(Giới hạn đã biết của bộ đo, không phải của engine — xem §Giới hạn ①.)* |
| ① | `execCommand('insertParagraph')` | `beforeinput` `inputType="insertParagraph"`, **`cancelable: true`**, rồi `input` không huỷ được |
| ② | engine dựng gì trong DOM | **`innerHTML = "A<div>B</div>"`** — một `<div>` **khối**. `soPhanTuCon = 1` |
| ③ | `cell.textContent` sau lượt đó | 🔴 **`"AB"`** — **không một `\n` nào** |
| ④ | tự chèn text node `"\n"` + `setPosition` | `theCon = [TEXT("A"), TEXT("\n"), TEXT("B")]` · `textContent = "A\nB"` ✅ · `selectionType = "Caret"`, neo **trong ô**, anchor là text node, offset 1 |
| ⑥ | chèn `<br>` *(đường Ice ký lúc đó)* | `innerHTML = "A<br>B"` · 🔴 **`textContent = "AB"`** · caret đặt được |
| — | đối chứng, chặn **bật lại** | `innerHTML = "AB"`, không đổi — hai lớp chặn của sản phẩm **còn sống** ✅ |

🔴 **Mệnh đề ③ là bẫy trung tâm của Quyết định #1, và nó TÁI LẬP ĐƯỢC trên engine thật.**
DOM có hai dòng, `textContent` có một chuỗi liền. Đường (a) *(thả engine chạy)* vì thế hỏng
AC1 ở vế *"ký tự lưu vào `target_text`"* **mà không cổng nào đỏ** — đúng như story dự đoán,
nay có số.

### 🔴 Vòng 2 — vì THƯỚC của vòng 1 bị nhiễu, không vì số của nó sai

Vòng 1 đo *"ô có vẽ ra hai dòng không"* bằng **chiều cao ô**, và cho **`71,00 px` ở MỌI
lượt**: trước `<br>`, sau một `<br>`, sau hai `<br>`, và cả ba giá trị `white-space`. Con số
đó không sai — nó **không trả lời câu hỏi**: `subgrid` đang ghim ô bản dịch theo track mà
**cột nguyên văn** dựng ra *(71 px ≈ 2,4 dòng)*, nên một ô hai dòng *(≈ 58 px)* nằm gọn dưới
trần.

⇒ Cùng lớp bẫy mà `2-5b-ban-do/README.md` §Giới hạn ⑤ đã ghi tên một lần: **một con số thật,
đọc sai câu hỏi**. Vòng 2 đổi thước, không đổi mệnh đề.

**Thước mới ⓐ — số dòng thật** = số `top` khác nhau trong `Range.getClientRects()` của một
Range bao trọn nội dung ô. Nó đếm **hộp dòng** và **không** bị `stretch` làm nhiễu.

| hình dạng DOM | `white-space: normal` | `pre-line` | `pre-wrap` |
|---|---|---|---|
| text node `"A\nB"` | 🔴 **1 dòng** | **2 dòng** | **2 dòng** |
| `A<br>B` | 2 dòng | 2 dòng | 2 dòng |

⇒ **Điểm chặn của Quyết định #2 là THẬT và đo được:** một `\n` trên đĩa, dưới `white-space`
mặc định, hiển thị thành **một dòng**. Và nó chỉ chạm đường **text node** — `<br>` xuống dòng
bất kể `white-space`.

**Thước mới ⓑ — áp lực NGƯỢC CHIỀU lên track hàng.** Hạ trần bằng cách rút ô nguyên văn
xuống một chữ (`一`), rồi cho ô bản dịch mang 1 · 2 · 5 dòng:

| ô bản dịch | track hàng | lệch `top` giữa các cột |
|---|---|---|
| 1 dòng | **38,00 px** | 0 px |
| 2 dòng | **63,00 px** | 0 px |
| 5 dòng | **150,00 px** | 0 px |

🔴 **Đây là chiều đo mà `deferred-work.md:3131-3162` CHƯA TÍNH, nay có số.** Ô bản dịch nhiều
dòng **đẩy track hàng cao lên**, và vì lệch `top` = 0 ở mọi lượt, **cả năm cột giãn theo**.
⇒ Suy luận hình học của Quyết định #4 được xác nhận ở vế cơ chế: track = max chiều cao các ô
cùng hàng, nên **bất kỳ** chiều cao thêm vào ô bản dịch — nội dung hay `padding` — đều nâng
track và kéo bốn ô kia.
⚠️ Đo bằng **nội dung**, không bằng `padding`. Hai thứ đi vào cùng một phép `max`, nhưng
đó là một suy luận một bước — ghi ra thay vì để người sau tưởng đã đo cả hai.

### 🔴 Vòng 3 — `<br>` cuối nội dung, tức thao tác THƯỜNG NHẤT

Vòng 1 đã hỏi câu này và trả lời bằng cái thước bị ghim. Số `caoMotBrPx = caoHaiBrPx =
71,00` của vòng 1 vì thế **không có nội dung** và bị **rút**, không được đọc thành *"trailing
`<br>` không vẽ ra dòng nào"*.

| hình dạng | số dòng | `textContent` |
|---|---|---|
| `A<br>` | 🔴 **1** | `"A"` |
| `A<br><br>` | **2** | `"A"` |
| `insertParagraph` ở cuối *(engine tự làm)* | 2 | `"A"` — `innerHTML = "A<div><br></div>"` |
| `insertLineBreak` ở cuối *(engine tự làm)* | 2 | 🔵 **`"A\n\n"`** — `innerHTML = "A\n\n"`, **0 phần tử con** |

🔴 **`A<br>` = MỘT dòng.** Bấm `Enter` ở cuối câu là thao tác thường nhất của FR134; nếu
màn hình không đổi gì thì đó đúng lớp lỗi mà Quyết định #2 tồn tại để chặn, chỉ khác chỗ
phát. ⇒ Đường `<br>` **buộc** hai phép chuyển phải thoả thuận về một `<br>` **canh chót** —
đúng chỗ mong manh nhất mà bảng quyết định đã cảnh báo.

🔵 Và hàng cuối bảng là thứ vòng 3 **không đi tìm**: `insertLineBreak` của chính WebKit dựng
**text node `\n`**, không markup.

### 🔵 Vòng 4 — một ĐƯỜNG THỨ TƯ mà story không liệt

Story liệt ba đường *(thả engine · tự chèn text node · `<br>` + hai phép chuyển)*. Đường thứ
tư: **chặn `insertParagraph`, rồi gọi `insertLineBreak` của chính engine**. Nó không có trong
bảng vì bảng được viết **trước** khi có số của vòng 3.

| | E1 giữa nội dung, `pre-line` | E2 **cuối** nội dung, `pre-line` | E3 giữa nội dung, `normal` *(đối chứng)* |
|---|---|---|---|
| `beforeinput` phát ra | **1**, `insertLineBreak`, cancelable | 1, như trái | 1, như trái |
| DOM | `TEXT("A") · TEXT("\n") · TEXT("B")` | `TEXT("A") · TEXT("\n") · TEXT("\n")` | `A<br>B` |
| phần tử con | **0** | **0** | 1 |
| `textContent` | ✅ **`"A\nB"`** | ✅ **`"A\n\n"`** | 🔴 `"AB"` |
| số dòng | 2 | **2** | 2 |
| caret | `Caret`, neo trong ô, anchor **là text node** | như trái | như trái |

**E4 — vế NGƯỢC** *(`el.textContent = "A\nB"`, đúng thứ `restoreEditedText` làm)*: cho **một**
text node `TEXT("A\nB")`, `textContent = "A\nB"`, **2 dòng**, 0 phần tử con.

## 🔴 Ba kết luận

**① Đường (a) *(thả engine)* bị bác bằng số.** `insertParagraph` dựng `<div>` khối và
`textContent` đọc ra `"AB"`. AC1 hỏng ở vế đĩa, im lặng.

**② Hai chữ ký RÀNG NHAU, và điều đó chưa ai viết ra.** Đối chứng E3 cho thấy hình dạng
engine dựng **phụ thuộc `white-space`**: `normal` ⇒ `<br>`; `pre-line` ⇒ text node `\n`.
⇒ Chữ ký #2(b) *(`pre-line`)* **là điều kiện** để đường thứ tư chạy. Không phải hai quyết
định độc lập, và một lượt đổi `white-space` sau này **lật cả Quyết định #1**. Chú thích tại
chỗ phải nói điều đó.

**③ 🔵 CHỮ KÝ #1 ĐÃ LẬT — (c) → (d), bằng phép đo, ngày 2026-08-16.**
Ice ký (c) ngày 2026-08-15, **trước** khi có vòng 3 và vòng 4. Hai số lật nó:
- `A<br>` = **1 dòng** ⇒ đường (c) phải mang một sentinel `<br>` canh chót, tức hai phép
  chuyển phải thoả thuận về một quy ước **không** có trong dữ liệu;
- `insertLineBreak` dưới `pre-line` cho **0 phần tử con** và `textContent` mang `\n` thật,
  **kể cả ở ca cuối nội dung** *(engine tự thêm `\n` canh chót — đúng thứ đường (c) phải tự
  làm bằng tay)*.

⇒ Đường (d) đóng AC1 **không cần một phép chuyển nào**: `reportEdit` giữ nguyên
`cell.textContent`, `restoreEditedText` giữ nguyên `el.textContent =` *(E4)*. Ice ký (d)
2026-08-16.

⚠️ **Cái giá của (d), ghi ra để không ai tưởng nó miễn phí:** `execCommand('insertLineBreak')`
tự phát một `beforeinput` `insertLineBreak` — mà nhánh ① đang chặn **đúng** inputType đó.
⇒ Cài đặt cần một **chốt chống đệ quy**, và chốt đó là một chỗ dễ viết sai. Số E1 nói rõ:
**đúng một** sự kiện phát ra mỗi lượt.

## Giới hạn thật — ghi ra thay vì để người sau tự phát hiện

1. ⚠️ **`browser.keys()` không đi vào đường nhập văn bản gốc của WKWebView** — mệnh đề ① đo
   được **0** `beforeinput` cho một `Enter` qua driver. Đây là giới hạn của **bộ đo**, không
   của engine *(cùng số với 2.5b, chủ: Story 1.22)*. Mọi mệnh đề ②–⑦ vì thế đi qua
   `execCommand`/`Range` — **cùng đường soạn thảo** của engine, **không** cùng đường phím vật lý.
2. 🔴 **Vế bộ gõ tiếng Việt thật không đường nghiệm thu nào của dự án mô phỏng được.**
   `Enter` là **phím chốt dấu** của Telex, nên nó là ca nặng nhất của story này. Task 1.5,
   **chủ: ICE**.
3. ⚠️ **Vế Blink chưa đo** — chỉ tới được qua WebView2/Windows. **Khoảng mù có tên**, chủ
   Story 1.22. Đừng suy hành vi Blink từ bảng này: tiền lệ `Backspace` offset 0
   (`deferred-work.md:3012-3037`) cho thấy hai engine **nói ngược nhau** ở đúng địa hạt này.
4. ⚠️ Vòng 2 ⓑ và vòng 3–4 **rút ô nguyên văn xuống một chữ** để hạ trần track. Đó là thao
   tác trên **cơ chế**, không ghi xuống đĩa, và được trả nguyên trạng ở cuối mỗi vòng. Nhưng
   nó nghĩa là các số **hình học** ở đây là số của một hàng **không có thật trong dữ liệu**.
5. ⚠️ Fixture chỉ có **một** segment, nên vế `subgrid` chỉ đo được trên **hai** ô cùng hàng
   *(`src` + `tgt`)*; ba cột kia không mang `data-segment-id`. Mệnh đề *"cả năm cột giãn
   theo"* đứng trên `lechTopPx = 0` cộng cấu trúc `grid-template-rows: subgrid` — **suy một
   bước**, không đo trực tiếp.
6. ⚠️ Bàn đo **không** đo hiệu năng trên 9.850 hàng *(chủ: Story 2.4)*, **không** đo tương
   phản, và **không** trả lời *"ca này thường gặp tới đâu trên dữ liệu thật"*.
7. ⚠️ Số của vòng 1 về **chiều cao ô** (`71,00 px` khắp nơi) đã bị **rút** ở vòng 2 vì thước
   sai câu hỏi. Chúng còn nằm trong log để truy nguyên được; **đừng** trích chúng.
