# Bàn đo Story 2.10 — điều hướng segment (Task 1; CHẶN Quyết định #1, #7 và Task 4)

Lưu ở đây để **không phải đo lại từ đầu**. Nó trả lời Task 1.1–1.5, nó **lật một lo ngại của
story bằng số**, và nó **tìm ra một tác dụng phụ không đường nghiệm thu nào khác của dự án bắt
được** — xem §Kết luận.

⚠️ Đây là **tạo tác của một lượt đo**, không phải công cụ của dự án. Không có gì ở đây vào
`package.json` — đúng khuôn `2-2-ban-do/` · `2-4-ban-do/` · `2-5-ban-do/` · `2-5b-ban-do/` ·
`2-5d-ban-do/` · `2-8-ban-do/` · `2-9-ban-do/`.

## Chạy

```sh
# Quyết định #7 — cuộn (vòng 1, rồi vòng 2 truy tổ tiên)
TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-10-ban-do/cuon-toi-hang.e2e.mjs
TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-10-ban-do/cuon-vong2-to-tien.e2e.mjs

# Quyết định #1 — ⌥↓ trong vùng gõ (vòng 1 HỎNG THƯỚC, vòng 2 là số dùng được)
TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-10-ban-do/alt-mui-ten-trong-vung-go.e2e.mjs
TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-10-ban-do/alt-vong2-hop-am-that.e2e.mjs

# Task 1.4 — NFR2, Chương 9.850 câu (~30 s chỉ riêng lượt tạo Tác phẩm)
TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-10-ban-do/doi-con-tro-chuong-lon.e2e.mjs

# VÒNG 3 — `focus()` có tự cuộn không (lật Task 4; chạy SAU khi đọc §Vòng 3)
TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-10-ban-do/focus-co-tu-cuon-khong.e2e.mjs

# Phụ lục — driver ghép phím bổ trợ tới đâu (quyết định hình dạng spec e2e)
TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-10-ban-do/probe-hop-am-driver.e2e.mjs
```

Cần **cổng 1420 trống**. Ra: các khối JSON trên stdout, một khối một bước.

`danh-tinh-phien.mjs` **không** phải một spec — nó là phép tự kiểm Task 1.1 dùng chung, cộng
thước `VACH_PRIMARY`.

## Môi trường — Task 1.5

WebKit **605.1.15** (`AppleWebKit/605.1.15`, WKWebView thật của Tauri) · macOS **15.7.9**
(kernel 24.6.0) · rustc/cargo **1.97.1** · Node **22.22.2** · cửa sổ debug `--features wdio`.
Cửa sổ **1280×832**, `devicePixelRatio` **2**. Đo **2026-08-18**.
Sáu spec: **4 + 3 + 4 + 3 + 3 + 4 passing** *(vòng 1 của Task 1.3 xanh nhưng **bỏ số** — thước
hỏng; vòng 3A `timeout` toàn bộ và cũng bỏ số)*.

### Phụ lục — driver ghép phím bổ trợ tới đâu (`probe-hop-am-driver.e2e.mjs`)

Đo trong chính cửa sổ này. Nó quyết định hình dạng của **cả spec e2e lẫn hai vòng bàn đo**:

| Lượt gọi | `code` nhận được | phím bổ trợ trên sự kiện |
|---|---|---|
| `browser.keys(['Meta', '2'])` | `Digit2` | ✅ `metaKey: true` |
| `browser.keys(['Meta', 'Enter'])` | `Enter` | 🔴 `metaKey: false` |
| `browser.keys(['Meta', 'Alt', 'ArrowDown'])` | `ArrowDown` | 🔴 cả hai `false` |
| chuỗi WebDriver keys ghép sẵn | `ArrowDown` | 🔴 cả hai `false` |

⇒ **Driver chỉ ghép phím bổ trợ cho phím IN ĐƯỢC.** Hàng `Digit2` là đối chứng dương: cùng đường
mã, cùng cửa sổ, cùng lượt chạy. ⇒ Mọi mệnh đề về hợp âm không-in-được phải đi bằng một
`KeyboardEvent` **tổng hợp** — tiền lệ đã ghi và đang chạy ở `editor-confirm-segment.e2e.mjs`.

---

## Task 1.1 — phép tự kiểm danh tính, và **khuyết tật của bàn đo 2.9 mà nó đóng**

Bàn đo 2.9 chỉ `waitForExist('[data-col="tgt"]')`. Phép đợi đó **không phân biệt** *"Chương mới
đã nạp"* với *"Chương cũ còn đó"* — giữa lượt `create_work_from_text` và lượt lưới nạp segment,
hàng của Chương trước **vẫn tồn tại**. `doiChuongVaKiemDanhTinh()` hỏi **ba** vế và **ném** khi
lệch: số hàng · **nội dung câu đầu** · `data-chapter-id`.

🔴 **Vế "nội dung câu đầu" là vế 2.9 không có, và nó là vế duy nhất bắt được ca đó.** Số hàng
một mình không đủ: hai lượt chạy của cùng một bàn đo dùng **cùng** fixture, nên số hàng khớp là
chuyện đương nhiên chứ không phải bằng chứng.

Cả năm lượt chạy đều xanh vế này: `chapterId: "1"` · `cauDau` khớp fixture từng chữ.

---

## 🔴 Task 1.2 / Quyết định #7 — CUỘN. Bốn số sạch, rồi một số thứ năm không ai hỏi

### Vòng 1 — bốn câu, cả bốn có đáp án dứt khoát

Lưới **60 hàng**, hộp cuộn `scrollHeight` **2280** / `clientHeight` **696** ⇒ tràn **1584 px**.
Hàng đích: **50**.

| Câu | Đo được | Đọc ra |
|---|---|---|
| Ⓐ hộp cuộn có tràn thật? | `coTran: true`, `overflowY: auto` | tiền đề đứng |
| Ⓐ `scroll-behavior` đang hiệu lực? | hộp **`auto`** · root **`auto`** | 🔵 khớp phép `grep`: **không** ai đặt `smooth`. Nhưng `auto` **uỷ quyền cho CSS**, nên `'instant'` tường minh vẫn là chốt duy nhất |
| Ⓑ cuộn đúng **track hàng** dù là `subgrid`? | `scrollTop` 0 → **1242** · `oNamTronTrongHop: true` · `namCotCungTop: true` *(cả `src` lẫn `tgt` đều `top: 759`)* | ✅ `subgrid` **không** bất đồng ở ca này |
| Ⓒ có hiệu ứng không? | **12/12** mẫu `scrollTop` đều **1242**, `soGiaTriKhacNhau: 1` | ✅ tức thì |
| Ⓓ `nearest` khi đã trong vùng nhìn | `truoc === sauMotLuot === sauBonLuot === 1242`. **Đối chứng âm** `block:'center'` → **1571** *(có dịch)* | ✅ đứng yên, và đối chứng chứng minh phép kiểm không rỗng |
| Ⓔ đối chứng đường (b) | `scrollTop` tính tay **1242**, trùng đường (a). 12/12 mẫu giống nhau | hai đường **cùng đích** |

### 🔴 Vòng 2 — trường `hopCoTuDichKhong` trả `true`, và đó là phát hiện của cả bàn đo này

Vòng 1 thêm một phép đo **phòng xa**: hộp cuộn có tự dịch không. Nó trả **`true`** —
`.grid-scroll` dịch **18 px** (`top` 119 → 101) trong chính lượt gọi, dù
`document.scrollingElement.scrollTop` vẫn **0**. Vòng 2 đi ngược **28 nút** tổ tiên để tìm thủ
phạm:

| Nút | `scrollTop` | `top` | `overflow-y` |
|---|---|---|---|
| `DIV#grid-tabpanel.grid-scroll` | 0 → **1242** | 119 → 101 | `auto` — *đúng việc của nó* |
| `DIV.panel-body` | 0 → 0 | 89 → **71** | `visible` — *bị đẩy theo* |
| 🔴 **`SECTION.panel`** | **0 → 18** | 77 → 77 | 🔴 **`hidden`** |

🔴 **`overflow: hidden` KHÔNG có nghĩa "không cuộn được".** Nó chỉ có nghĩa *"không vẽ thanh
cuộn"*. `scrollIntoView` cuộn **mọi tổ tiên cuộn được**, và `SECTION.panel` có
`scrollHeight > clientHeight` nên nó **là** một trong số đó. Hệ quả: thân panel dịch lên **18 px
và ở lại đó** — người dùng **không có đường nào** đưa nó về, vì một hộp `overflow: hidden` không
có thanh cuộn để kéo và không nhận cử chỉ cuộn.

| Câu vòng 2 | Đo được |
|---|---|
| Ⓖ **đường (b)** có gây cùng chuyện không? | 🔵 **KHÔNG.** `nutDaDoi` chỉ có **một** phần tử: chính `.grid-scroll`. `top` của nó **101 → 101**. Không một tổ tiên nào đổi |
| Ⓗ tác dụng phụ lặp lại mỗi lượt hay một lần? | **Một lần.** Bốn lượt tiếp theo *(xen kẽ hàng 50 / hàng 2)*: `topHop` **101 → 101** ở cả bốn. `scrollTop` của hộp đi đúng: `0→1242`, `1242→76`, `76→1242`, `1242→76` |

⚠️ *"Chỉ một lần"* **không** làm nó thành vô hại: 18 px mất đi là mất **vĩnh viễn** trong phiên
đó, và nó xảy ra ở **lượt điều hướng đầu tiên** của mỗi phiên — tức mọi người dùng đều gặp.

**⇒ Ice ký đường (b), 2026-08-18.** Lý do là bảng Ⓖ: nó cho **đúng cùng một `scrollTop`** với
đường (a), **cùng** kết quả *"hàng nằm trọn trong hộp"*, **cùng** 12/12 mẫu không hiệu ứng, và
**không** tác dụng phụ nào. Cái giá — tự cài *"đã trong vùng nhìn thì đừng cuộn"* và tự xử lề
trên/dưới — là **sáu dòng đã chạy thật** trong `cuon-toi-hang.e2e.mjs` §Ⓔ.

🔵 **Và một lo ngại của story bị bác:** story viết *"đường (b) **cũng** đọc CSS `scroll-behavior`
khi gán `scrollTop` — nó không miễn nhiễm; nó chỉ đổi chỗ rủi ro"*. Đo được: `duongB_coHieuUng:
false`, 12/12 mẫu bằng nhau. Mệnh đề ấy **đúng về nguyên tắc** *(gán `scrollTop` có đi qua
`scroll-behavior`)* nhưng **không phải một rủi ro phân biệt được hai đường** — cả hai đường đều
phụ thuộc nó y như nhau, nên nó không phải một lý lẽ chống (b).

---

## 🔴 Task 1.3 / Quyết định #1 — `⌥↓` TRONG VÙNG GÕ

### Vòng 1 — **BỎ SỐ, GIỮ BÀI HỌC.** Thước hỏng, và đối chứng dương tự tố cáo

`browser.keys(['Alt', 'ArrowDown'])` giao **hai `keydown` rời**, và cái thứ hai mang:

```
{ key: "ArrowDown", code: "ArrowDown", altKey: FALSE }
```

⇒ Hợp âm `Alt+ArrowDown` **chưa từng được gửi**. `keys.ts:509` so `sameMods` **trước** mọi thứ
khác nên nó `continue`, và vòng lặp thoát **trước** cả dòng `isTypingZone` — đúng cơ chế mà
`GridPanel.vue:1084-1092` đã ghi cho ca `Enter` trần.

✅ **Đối chứng dương Ⓒ đã làm đúng việc của nó:** caret **ngoài** vùng gõ cũng đứng yên. Một bàn
đo không có Ⓒ sẽ đọc vòng 1 thành *"đã xác nhận: luật vùng gõ nuốt `⌥↓`"* — một kết luận **tình
cờ đúng** rút ra từ một con số **rỗng**, tức đúng thứ nguy hiểm hơn một kết luận sai.
*(Lần thứ hai trong epic một đối chứng dương cứu một lượt kết luận sai; lần đầu là `2-9-ban-do`
§Vòng 1 đối chứng Ⓔ.)*

⚠️ **Lượt sửa THƯỚC thứ nhất, không một vòng chẩn đoán bị bác** — LUẬT DỪNG đếm những vòng mà
một giả thuyết về **sản phẩm** bị phép đo bác. Đếm hôm nay: **0**.

**Một số của vòng 1 sống sót**, và nó có ích: driver giao `code: "AltLeft"` rồi `code:
"ArrowDown"` — tức phép khớp bằng `event.code` của kho **nhận diện đúng phím**, khác ca `⌘/` của
2.8 nơi driver giao `code: "/"` thay vì `"Slash"`.

### 🔴 Vòng 2 — sự kiện dựng bằng `new KeyboardEvent`, và ba số dứt khoát

Thước mới **hợp lệ cho đúng câu đang hỏi**: câu ① hỏi về `keys.ts:510` — **mã JavaScript của
sản phẩm** — và luật đó đọc **đúng ba** thứ (`event.code` · `modsOf(event)` · `event.target`).
Nó **không** đọc `isTrusted`. Hình dạng hợp âm nay do bàn đo kiểm soát, không do driver đoán.

| Ca | target | hợp âm tự kiểm | `defaultPrevented` | vạch lề |
|---|---|---|---|---|
| Ⓘ | ô `[data-col="tgt"]`, `isContentEditable: true` | `code: ArrowDown, altKey: true` ✅ | **`false`** | 0 → 0 — **không dời** |
| 🔴 **Ⓙ đối chứng DƯƠNG** | `body`, ngoài vùng gõ | `code: ArrowDown, altKey: true` ✅ | **`true`** | **0 → 1** *(`segmentId` 1 → 2)* |

⇒ **Biến duy nhất khác nhau giữa hai ca là `event.target`.** Ⓙ chứng minh lệnh chạy được và
thước hoạt động; Ⓘ vì thế là một mệnh đề về **sản phẩm**: 🔴 **`keys.ts:510` NUỐT `⌥↓` khi caret
ở trong ô — xác nhận, không còn là suy luận từ mã.**

### Ⓚ — tiền đề của đường (c), đo trong **cùng một ô**

| Hợp âm | Đã đăng ký? | Mang `Mod`? | `defaultPrevented` |
|---|---|---|---|
| `Mod+Enter` | ✅ | ✅ | **`true`** — đi qua |
| `Alt+ArrowDown` | ✅ | ❌ | **`false`** — bị nuốt |
| `F9` *(đối chứng ÂM)* | ❌ | — | **`false`** |

🔴 Đối chứng âm `F9` là thứ làm bảng này đọc được: nó chứng minh `defaultPrevented` **không
phải luôn `true`**, nên giá trị `true` ở hàng đầu là một tín hiệu thật.

**⇒ Ice ký đường (c), 2026-08-18** — đổi `editor.next_untranslated` sang một hợp âm mang `Mod`.

🔵 **Và rủi ro *"mồ côi binding người dùng đã gán"* mà story nêu là NHỎ HƠN story tưởng — đo từ
kiểu, 2026-08-18.** `ChordOverrides` là `Readonly<Record<CommandId, readonly string[]>>`
(`keys.ts:457`) — khoá theo **command id**, không theo hợp âm. Story này **không đổi id**, chỉ
đổi `spec.keys`. ⇒ Người dùng đã tự gán phím **giữ nguyên** phím của họ *(`createKeymap` ưu tiên
`overrides` qua `hasOwnProperty`, `:474-477`)*; chỉ **mặc định** đổi, và nó chỉ chạm những ai
chưa từng tuỳ chỉnh.

### ⚠️ Một vế vẫn KHÔNG đo được ở đây — ghi ra thay vì giả vờ

| Câu | Vì sao không đo được | Đi đâu |
|---|---|---|
| `⌥↓` trong ô văn bản trên macOS **có** là *"xuống cuối đoạn"* của hệ điều hành không, và `preventDefault()` có chặn nổi nó không | Mọi sự kiện của bàn đo `isTrusted: false`, và một sự kiện không tin cậy **không có default action** ⇒ một phép kiểm sẽ trả **CÓ CHẶN ĐƯỢC** trên mọi engine, kể cả engine không cho chặn | **Chữ ký tay của Ice** *(kiểm trên máy thật)*. ⚠️ Chữ ký #1(c) làm câu này **hết chặn**: đường (c) không cướp `⌥↓`, nên vế này không còn là cửa của Story 2.10. Nó ở lại như một mệnh đề chưa đóng nếu ai đó sau này muốn đường (b) |

---

## Task 1.4 — NFR2. **GHI SỐ, KHÔNG CHẤM ĐẠT** *(chủ là Story 2.4)*

Chương **9.850 câu** — cùng số hàng với mốc của Story 2.5b, để hai con số **so sánh được**.
`59.106` phần tử DOM trong lưới.

| Hàng đích | Khung hình dài nhất | Tới khi vạch lề dời |
|---|---|---|
| 9.000 | **107 ms** | **1.248 ms** |
| 500 | **104 ms** | **848 ms** |
| 5.000 | **85 ms** | **884 ms** |

Mốc đối chiếu: **706–770 ms** (2.5b) · trần một khung hình NFR2 = **50 ms**.

🔴 **KHÔNG kết luận *"Story 2.10 làm nó tệ hơn"* từ bảng này, và lý do là phương pháp chứ không
lịch sự:** phép đo ở đây tính từ **trước** lượt đăng ký `requestAnimationFrame` tới khung hình
**đầu tiên** thấy `.rule-primary` ở hàng đích, còn phép đo của 2.5b chưa được đọc lại để biết nó
tính từ mốc nào. Hai thước khác nhau cho hai con số không trừ được cho nhau. Cái bảng này nói
được, và chỉ nói được: **cùng một bậc độ lớn, vẫn vượt trần một khung hình nhiều lần.**

⚠️ `soKhungHinhLayMau: 4` ở cả ba lượt — vòng lấy mẫu dừng ngay khi vạch tới nơi, nên
`khungHinhDaiNhat_ms` là khoảng cách lớn nhất **giữa bốn khung hình đó**, không phải khung hình
dài nhất của toàn lượt. Một giới hạn thật của thước, ghi ra.

### 🔴 Ⓒ — và đây là số làm việc cho Task 4: **cuộn KHÔNG phải chỗ tốn tiền**

| Phép đo, hàng 9.000 trên 9.850 hàng | Số |
|---|---|
| Lời gọi `scrollIntoView` mất bao lâu | **0 ms** |
| Khung hình dài nhất quanh lượt cuộn | **18 ms** |
| `scrollTop` sau | 341.342 |
| Số khung hình lấy mẫu | 27 |

⇒ Một lượt **cuộn** tốn **18 ms** trong khi một lượt **đổi caret** tốn **85–107 ms** ở cùng
Chương. Chi phí nằm ở `ruleById` *(Cạm bẫy ④)*, **không** ở Task 4. Task 4 vì thế **không** làm
đường nóng xấu thêm đáng kể, và nó **không** phải chỗ để tối ưu.

---

## Bốn thứ bàn đo này **KHÔNG** đo được — ghi ra thay vì giả vờ

| Câu hỏi | Vì sao không đo được | Đi đâu |
|---|---|---|
| `preventDefault()` có chặn nổi default action của một phím **thật** không | `isTrusted: false` ⇒ không có default action để chặn | Chữ ký Ice *(nay không còn chặn — xem #1(c))* |
| Lượt cuộn **trông** thế nào bằng mắt | Không thước nào của dự án nhìn thay được | 🔴 **Task 4.5 — món cho Ice** |
| 18 px của `SECTION.panel` **che mất cái gì** | Bàn đo đọc được `top`, không đọc được *"người dùng mất nhìn thấy phần nào"* | Đã tránh bằng chữ ký #7(b); nếu ai lật sang (a) thì phải đo lại vế này |
| Một khung hình có vượt 50 ms không, **theo thước của dự án** | Bộ đo NFR2/NFR18 thuộc **Story 2.4**; dựng bộ thứ hai là dựng nguồn sự thật thứ hai | **Story 2.4** |

---

## 🔴 VÒNG 3 — `focus()` ĐÃ CUỘN SẴN, và nó lật Task 4

`focus-co-tu-cuon-khong.e2e.mjs`. Vòng này sinh ra từ một **đột biến không giết được ca nào**:
ca e2e §Ⓒ *("vùng nhìn đã cuộn, hàng đích nằm trọn trong hộp")* xanh, rồi Task 7.3 **gỡ** lời gọi
`cuonToiHang` khỏi mã sản phẩm và chạy lại — **4/4 vẫn xanh**.

⚠️ **VÒNG 3A HỎNG THƯỚC, bỏ số giữ bài học.** Bản đầu gộp ba phép đo vào một `browser.execute`
async có `o.blur()` cộng nhiều `requestAnimationFrame` lồng nhau: **cả ba ca `Script execution
timed out`**, 6 phút 23, không một con số. Thước bị **thay** chứ không được vá — bản dùng được
hỏi **một** câu trong **một** lời gọi **đồng bộ** *(`focus()` cuộn đồng bộ, nó không phải một
hoạt ảnh)*. Vẫn là sửa **thước**, không một vòng chẩn đoán sản phẩm bị bác.

Hàng 50/60, hộp cuộn `scrollTop` tối đa 1584, mỗi hàng 38 px:

| Lượt | `scrollTop` sau | hàng nằm trọn | `SECTION.panel` |
|---|---|---|---|
| Ⓘ `focus()` một mình | **1569** | ✅ | 0 |
| Ⓙ `focus({preventScroll:true})` — **đối chứng ÂM** | 0 | ❌ | 0 |
| Ⓚ `focus({preventScroll:true})` + công thức tự cài | **1242** | ✅ | 0 |
| Ⓛ `focus()` khi ô **đã có tiêu điểm** rồi + công thức | 0 → 1242 | ✅ | 0 |

🔴 **Ⓙ là ca làm bảng này đọc được.** Không có nó, ai đó sẽ đọc Ⓘ thành *"một thứ nào đó trong
lượt chạy đã cuộn"*. Với nó, biến duy nhất khác nhau là `preventScroll`.

🔴 **1569 vs 1242 KHÔNG phải sai số — chúng là hai NGỮ NGHĨA.** 1242 = mép dưới (*nearest*);
1569 ≈ **căn giữa** *(đối chứng độc lập: `block:'center'` ở vòng 1 cho **1571**)*. ⇒ WebKit
**căn giữa khi hàng ở xa** và dùng **nearest khi nó chỉ vừa ló khỏi mép** — vế sau đo được ở ca
e2e Ⓔ: đi xuống một hàng dịch **đúng 38 px**, không phải hàng trăm.

⇒ **Hành vi sẵn có TỐT HƠN công thức tự cài.** Nhảy xa thì người dùng có ngữ cảnh trên dưới; bấm
liên tục thì vùng nhìn không giật. Công thức ép *nearest* ở **mọi** ca, tức dán hàng đích vào sát
mép dưới sau một lượt nhảy xa.

**⇒ Ice ký GỠ `cuonToiHang` và `preventScroll` 2026-08-18.** AC8 nửa sau **đã đạt sẵn**, đúng
cách AC8 **nửa đầu** đã đạt sẵn nhờ `ruleById` — hai vế, cùng một hình dạng, và story đoán sai vế
thứ hai.

⚠️ **Cái giá đã nhận, không giấu:** AC8 nửa sau nay dựa vào hành vi engine, **không chuẩn nào bảo
đảm** và **không cổng nào canh**. Ghi trong `deferred-work.md` kèm chủ, và ghi thẳng trong
`GridPanel.vue` §"AC8 NỬA SAU" — cộng một dòng cấm: **đừng thêm `preventScroll`**, làm thế là tắt
vế cuộn duy nhất đang có mà mọi ca tự động vẫn xanh.

---

## Kết luận — sáu thứ Task 2–4 phải làm theo

1. 🔵 **KHÔNG cuộn bằng tay, và KHÔNG `scrollIntoView` — `focus()` đã lo cả vế này** (vòng 3,
   Ice ký). Hai kết luận đầu của bản README này là kết luận **của vòng 1–2**, và chúng đã bị
   vòng 3 thay: lúc đó câu hỏi còn là *"đường (a) hay đường (b)"*, chưa ai hỏi *"có cần đường nào
   không"*. Giữ lại chúng ở dưới vì chúng vẫn đúng **nếu** một ngày phải tự cuộn.
2. 🔴 Nếu **phải** tự cuộn lần nữa: dùng `scrollTop` tính tay, **KHÔNG** `scrollIntoView` — vòng 2
   Ⓕ/Ⓖ: đường (a) cuộn `SECTION.panel` (`overflow: hidden`) 18 px và người dùng không lấy lại
   được. Công thức đúng ngữ nghĩa `nearest` đã chạy thật ở §Ⓔ: `oBox.top < hopBox.top` ⇒ trừ;
   `oBox.bottom > hopBox.bottom` ⇒ cộng; **không thoả cả hai ⇒ không gán gì**. Và nhớ:
   `focus()` cuộn **trước**, nên công thức chỉ có tác dụng nếu đi kèm `preventScroll`.
3. 🔴 **Đổi `editor.next_untranslated` sang hợp âm mang `Mod`** — Ⓘ/Ⓙ/Ⓚ. Giữ **nguyên id**, nên
   binding người dùng đã gán không mồ côi.
4. **Không** thêm `scroll-behavior` vào CSS ở bất kỳ đâu, và **không** dựa vào việc *"kho không
   có `smooth`"* — Ⓐ đo được cả hai đều `auto`, tức **uỷ quyền**, tức chưa ai canh.
5. **Task 4 không phải chỗ tối ưu hiệu năng** — 18 ms so với 85–107 ms của một lượt đổi caret.
6. **Vạch lề là một `class` ở cột riêng, không một `data-` attribute** — mọi phép kiểm tương lai
   hỏi `.col-rule .cell-rule > .rule-primary` theo **chỉ số hàng**, không hỏi `[data-caret]`
   *(thuộc tính đó không tồn tại; xem `danh-tinh-phien.mjs` §`VACH_PRIMARY`)*.
