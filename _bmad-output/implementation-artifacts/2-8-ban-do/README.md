# Bàn đo Story 2.8 — caret ở **cột nguyên văn** (Task 1, CHẶN task cài đặt của Quyết định #2)

Lưu ở đây để **không phải đo lại từ đầu**. Nó trả lời Task 1.1 và Task 1.3, và nó **lật một
chữ ký của Ice** — xem §Kết luận.

⚠️ Đây là **tạo tác của một lượt đo**, không phải công cụ của dự án. Không có gì ở đây vào
`package.json` — đúng khuôn `2-2-ban-do/` · `2-4-ban-do/` · `2-5-ban-do/` · `2-5b-ban-do/` ·
`2-5d-ban-do/`.

## Chạy

```sh
# 🔴 Cổng 4445 bị `gdrive-su` giữ (đo 2026-08-17, PID 91509) — PHẢI dời cổng.
TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-8-ban-do/tach-cot-nguon-wkwebview.e2e.mjs
TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-8-ban-do/tach-cot-nguon-vong3.e2e.mjs
```

Cần **cổng 1420 trống**. Ra: các khối JSON trên stdout, một khối một bước.

## Môi trường

WebKit **605.1.15** (`AppleWebKit/605.1.15`, WKWebView thật của Tauri), macOS 24.6.0, cửa sổ
debug `--features wdio`. Node 22.22.2 · rustc 1.97.1. Ba vòng, **1 passing** mỗi vòng.
Tự kiểm danh tính phiên mỗi vòng: `href = http://localhost:1420/`, `#app` có mặt ✅

---

## Vòng 1 — 🔵 **BỎ SỐ, GIỮ BÀI HỌC.** Thước hỏng, không phải số sai

Vòng 1 cho một bảng **tự mâu thuẫn**: bước *"bấm ô nguyên văn"* báo neo nằm ở cột **bản
dịch**, còn bước **đối chứng** *"bấm ô bản dịch"* báo neo ở cột **nguyên văn** — hai số hoán
vị nhau. Hai khuyết tật của bàn đo đứng sau:

- **Ⓐ Không bước nào dọn vùng chọn, và không bước nào tự khai nó vừa bấm vào cái gì.** Một
  lượt bấm **không đổi vùng chọn** để lại nguyên vùng chọn của bước trước — và bảng đọc ra
  **y hệt** một lượt bấm thành công vào chỗ khác.
  🔴 Đây đúng lớp *"rỗng im lặng"*: *"engine không làm gì"* và *"engine làm đúng"* cho **cùng
  một** bảng số. Vòng 2 dọn vùng chọn về `"None"` trước **mỗi** bước và khẳng định nó rỗng.
- **Ⓑ `TreeWalker` lấy text node ĐẦU TIÊN, và nó rỗng.** Ô nguyên văn mang hai `#comment`
  (`aura-allow-text`) nên `childNodes` là `[COMMENT, TEXT(0), COMMENT, TEXT(40), TEXT(0)]`.
  `setPosition(node, 0)` trên node dài 0 là một phép đo về hư không.

⚠️ **Hai lượt sửa THƯỚC, không hai vòng chẩn đoán bị bác** — LUẬT DỪNG (Task 1.4) đếm những
vòng mà một **giả thuyết về sản phẩm** bị phép đo bác. Phân biệt này có tiền lệ ở
`2-5d-ban-do/README.md` §Vòng 2 (*"số thật, trật câu hỏi"*).

---

## Vòng 2 — bốn mệnh đề, thước đã sửa

### ⓪ Kiểm kê một ô `[data-col="src"]`

| Trường | Số đo |
|---|---|
| `contenteditable` · `tabindex` | **`null`** · **`null`** — cột nguyên văn không có cả hai |
| `soPhanTuCon` | **0** *(Hán Việt tắt — Tác phẩm `zh` nhưng tab đang ở `original`)* |
| `childNodes` | `[COMMENT, TEXT(0), COMMENT, TEXT(40), TEXT(0)]` — **3 text node**, hai cái rỗng |
| `textContent` | `"Một câu nguồn để bộ nhập có việc mà làm."` (40 ký tự) |
| Tổng độ dài các text node **trước** node dài nhất | **0** |
| chiều cao ô | 71 px |

🔵 **Dòng cuối là câu trả lời cho một vế của Quyết định #2(a) mà story không hỏi thẳng:**
`anchorOffset` trong text node dài nhất **bằng đúng** chỉ số ký tự trong `source_text`, vì
mọi text node đứng trước nó cộng lại dài **0**. ⇒ Nếu có một vùng chọn, phép ánh xạ
offset → chỗ cắt là **tầm thường** *(hôm nay, với Hán Việt TẮT)*.

⚠️ **Và vế chưa đo, ghi ra thay vì để người sau tưởng đã xét:** với Hán Việt **BẬT**, ô mang
thêm một `SourceHanViet` với `<ruby>`/`<rt>`, và `<rt>` mang `user-select: none`
(`SourceHanViet.vue:980`). Lúc đó `soPhanTuCon > 0` và phép ánh xạ ở trên **hết đúng**. Chưa
đo — chủ: story cài đặt #2.

### ① Đối chứng — bấm ô `[data-col="tgt"]` *(chạy TRƯỚC mọi bước khác)*

| Trường | Số đo |
|---|---|
| dọn trước khi bấm | `"None"` ✅ |
| đích *(chụp ngay trước khi bấm)* | `cell cell-tgt empty`, `data-segment-id="1"`, hộp `239×71 @ (289,119)` |
| `elementFromPoint` ở tâm ô | **chính ô đó** ✅ *(không lớp phủ nào nuốt cú bấm)* |
| **sau khi bấm** | `selectionType = "Caret"` · `rangeCount = 1` · neo ở cột **`tgt`** · `activeCol = "tgt"` |

⇒ **Thước tốt.** Đối chứng chạy **đầu tiên** có chủ ý: ở vòng 1 nó chạy sau một lượt
`setPosition` bằng script và vì thế đọc lại đúng vùng chọn của bước đó.

### ② 🔴 Một cú bấm vào ô `[data-col="src"]`

| Trường | Số đo |
|---|---|
| dọn trước khi bấm | `"None"` ✅ |
| đích | `cell cell-src`, `data-segment-id="1"`, hộp `239×71 @ (50,119)`; `elementFromPoint` = **chính ô đó** ✅ |
| **sau khi bấm** | **`selectionType = "None"`** · **`rangeCount = 0`** · `activeElement` không đổi |
| lượt `lookup_dictionary` đã bay | **0** |

### ③ 🔴 Kéo chọn 20 % → 50 % chiều rộng trong ô nguyên văn

`selectionType = "None"` · `rangeCount = 0` · `chuDaChon = ""` · **0** lượt tra.

### ④ `setPosition` bằng script vào text node **dài nhất**

`datDuoc: true` · `selectionType = "Caret"` · `anchorOffset = 3` *(đúng thứ vừa đặt)* ·
`anchorLaChinhTextNode: true` · `chuTruocCaret = "Một"`.

⇒ **Engine GIỮ được một vùng chọn đặt bằng script trong vùng chỉ-đọc.** Chỉ đường **chuột**
là không cho gì.

---

## Vòng 3 — loại từng ứng viên của BÀN ĐO, một biến mỗi lượt

Bảng vòng 2 nói một điều **to hơn Story 2.8**: nếu cột nguyên văn không bôi đen được thì
**Auto-Lookup** (FR21, Story 1.18, **đã phát hành**) cũng chết — nó dựa toàn bộ vào một vùng
chọn ở cột nguồn (`GridPanel.vue:309`, `useSelectionSurface(colSrc, 'source', …)` →
`attachSelectionWatcher` bắt `mouseup` → `currentSelectionText()`).

Một mệnh đề hạng đó **không được ghi từ một lượt đo còn ứng viên chưa loại**. Hai ứng viên:

- **Ⓐ `blur()` của chính bàn đo** — vòng 2 gọi `document.activeElement.blur()` trước mỗi
  bước; WebKit từ chối dựng vùng chọn khi tài liệu mất tiêu điểm.
- **Ⓑ lượt kéo quá thô** — hai điểm, 30 ms; WebKit có thể cần nhiều bước trung gian.

| # | Biến thể | `selectionType` | `rangeCount` | `document.hasFocus()` |
|---|---|---|---|---|
| Ⓐ0 | nền, chưa chạm gì | `"None"` | 0 | **false** |
| ① | bấm đơn ở cột nguồn, **KHÔNG** `blur()` | **`"None"`** | 0 | false |
| ② | kéo **6 bước trung gian**, **KHÔNG** `blur()` | **`"None"`** | 0 | false |
| ③a | bấm ô **bản dịch** *(lấy tiêu điểm vào tài liệu)* | `"Caret"`, neo `tgt` | 1 | false |
| ③b | kéo ở cột nguồn **SAU** ③a | **`"None"`** | 0 | false |
| ④ | `setPosition` + `modify('extend','forward','word')` | **`"Range"`**, `chuDaChon = "Một"` | 1 | false |

**Ⓐ bị loại** *(① và ② không `blur()` mà vẫn `"None"`)*. **Ⓑ bị loại** *(② sáu bước vẫn
`"None"`)*.

🔵 **Và một số phụ đáng giá hơn cả hai:** `document.hasFocus()` là **`false` ở MỌI bước** —
kể cả ③a, nơi ô bản dịch **vẫn** nhận được caret. ⇒ *"tài liệu có tiêu điểm"* **không** là
điều kiện, và nó cũng không phải biến phân biệt hai cột. Thứ phân biệt là **`contenteditable`**.

⚠️ **③b còn XOÁ mất caret mà ③a vừa đặt** — một lượt kéo ở cột nguồn không tạo ra vùng chọn
nào **và** thu hồi vùng chọn đang có. Tức nó **không** phải *"engine bỏ qua cử chỉ"*; engine
có xử lý cú bấm, nó chỉ không dựng được vùng chọn.

---

## 🔴 Kết luận

### ① Đường (a) của Quyết định #2 **KHÔNG có đầu vào** trên engine thật

Chữ ký #2(a) đọc: *"lấy chỗ cắt từ **vùng chọn đang có** ở cột nguyên văn"*. Đo được: trên
WKWebView, **không cử chỉ chuột nào** tạo ra một vùng chọn ở cột nguyên văn — không caret từ
một cú bấm, không `Range` từ một lượt kéo, kể cả khi tài liệu đã có tiêu điểm và lượt kéo đi
sáu bước. ⇒ *"vùng chọn đang có"* là một tiền đề **không tồn tại**.

Chữ ký #2(a) ký **TRƯỚC** phép đo, và story ghi thẳng rằng **luật dừng thắng chữ ký nếu phép
đo trượt** — cùng khuôn chữ ký #6(a) của Story 2.5b và Quyết định #1 của Story 2.5d.

### ② Vế còn hở, và nó **không** loại trừ được bằng chính bộ đo này

Mọi cử chỉ ở trên đi qua **WebDriver pointer actions**, không phải chuột vật lý. Bộ đo này đã
có **một** giới hạn cùng hạng được ghi từ 2026-08-12: `browser.keys()` không đi vào đường
nhập văn bản gốc của WKWebView. ⇒ Ứng viên thứ ba — *"driver không lái được máy chọn văn bản
của WebKit trong nội dung không sửa được, dù lái được tiêu điểm"* — **không loại trừ được
bằng driver**, theo cấu tạo.

**Hệ quả phải nói ra, và nó lớn hơn story này:** nếu ứng viên thứ ba **sai** *(tức chuột thật
cũng không chọn được)* thì **Auto-Lookup bằng chuột đã chết trong bản đang chạy** và không
đường nghiệm thu nào của dự án bắt được — bộ e2e hôm nay **không có spec nào** cho vùng chọn
ở cột nguồn.

⇒ **Một lượt thử BẰNG TAY, chủ: Ice** — bôi đen một cụm ở cột nguyên văn bằng chuột thật,
xem Panel Lookup có tra không. Đúng khuôn Task 1.4 của 2.5b và Task 1.5 của 2.5d: một bất
biến mà không đường nghiệm thu nào của dự án mô phỏng được thì **chữ ký của Ice LÀ đường
nghiệm thu**.

### ③ Cái mà phép đo **cho phép**, và nó gợi một đường thứ năm

`setPosition` và `modify('extend', …)` **chạy được** ở cột nguyên văn, và `anchorOffset` ánh
xạ **thẳng** vào chỉ số ký tự của `source_text` (mọi text node đứng trước dài 0). Tức sản phẩm
**tự đặt** được caret ở cột nguồn từ toạ độ một cú bấm — đúng **khuôn đường chuột mà Story
2.5b đã phải dựng cho ô BẢN DỊCH** (`setPosition` ở `mouseup`, `GridPanel.vue` §AC3 đường
chuột), vì `contenteditable` trần cũng không đủ ở đó.

⇒ Đây là một đường **thứ năm** cho Quyết định #2, chưa có trong bảng bốn đường của story. Dev
đề xuất, **Ice chốt** — không tự chọn.
