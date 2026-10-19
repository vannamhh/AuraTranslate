---
ticket: 22
status: done
type: chore
---

# Plan: Bộ chạy e2e trong webview thật

Bản ghi build v6 của story này bị thiếu: `sprint-status.yaml` ghi `done` nhưng không có tệp `1-22-*.md` hay `spec-1-22-*.md` trong `implementation-artifacts`. Plan tối thiểu này giữ trạng thái đã ghi; không có baseline vì không có bản ghi để lấy baseline đáng tin (không suy từ HEAD).

## Acceptance Criteria (epics.md, Story 1.22)

**Covers:** đường đóng cho **nợ nghiệm thu thị giác** của Epic 1 *(không FR mới)* · **AD-45** · liên đới action item **A4** và **A5**

> ⚠️ **Story này dựng SAU khi mã đã viết — ghi thẳng thay vì để nó trông bình thường.** Bộ chạy ra đời ngày 2026-08-11 từ một đề xuất được Ice ký, và Bước 0 tới Bước 2 đã chạy xong. Lý do vẫn dựng story: bộ chạy đang mang **ba khuyết tật có tên** mà không tạo tác nào ở tầng quy hoạch chịu trách nhiệm cho chúng. Không có story thì ba mục đó sống trong một tệp đề xuất, và Epic 2 sẽ dựng Panel Editor — bề mặt thị giác lớn nhất dự án — lên trên một nền như vậy. Kết quả đo và ba giả định bị lật: `initiative-auratranslate/epic-nen-mong-ung-dung-va-tra-cuu-ngoai-tuyen-tuc-thi/proposal-tauri-window-automation/proposal-tauri-window-automation.md` §8.

As a chủ dự án,
I want một bộ chạy lái được cửa sổ Tauri thật,
So that món nợ nghiệm thu thị giác — món **duy nhất có hệ số nhân** — có một đường đóng bằng máy thay vì chỉ bằng mắt.

**Acceptance Criteria:**

**Given** một hàng bàn đo thị giác viết thành kịch bản
**When** nó chạy
**Then** nó chạy trong **webview THẬT của sản phẩm** — WKWebView trên macOS, WebView2 trên Windows — không trong Chrome
**And** mệnh đề này **đã đo, không suy đoán**: hoàn nguyên `keyCellOf(id)?.focus()` trong `config/shortcutsState.ts` — đúng bản vá WKWebView của lượt code review Story 1.21 — làm ca **ĐỎ** đúng triệu chứng *"ô phím không đổi"*; khôi phục ⇒ **XANH**
**And** một bộ chạy trong Chrome đóng được lớp DOM trung tính và **KHÔNG** đóng được lớp lỗi đặc thù engine — đừng mua sự yên tâm sai ở đây

**Given** bộ chạy dựng một cửa sổ thật trên máy người chạy
**When** một ca sửa cấu hình
**Then** 🔴 nó ghi vào một `$APPDATA` **TẠM của riêng lượt chạy**
**And** nó **KHÔNG** đụng `global.db` thật của Ice — hôm nay mỗi ca tự dọn bằng nút *"Về mặc định"*, đó là **vá triệu chứng**
**And** đây là việc phải đóng **TRƯỚC** khi dựng thêm bất kỳ hàng bàn đo nào

**Given** `element.click()` của driver bắn `click` **trước** `focusin`
**When** một ca phụ thuộc thứ tự sự kiện
**Then** ca đó đi qua **Actions API**, không qua `element.click()`
**And** lý do ghi ra tại chỗ, vì một ca xanh nhờ thứ tự sai là một ca nói dối

**Given** nhiều tệp spec trong cùng một lượt chạy
**When** chạy `npm run test:e2e` không kèm `--spec`
**Then** cả bộ chạy hết trong **một** lượt, tuần tự
**And** ⚠️ **AC này đã đổi 2026-08-12 sau một phép đo.** Bản gốc giả định máy chủ nhúng bám cổng cố định **4445** làm phiên thứ hai trượt, và đòi *"cổng cấp theo worker, hoặc chạy từng tệp bằng `--spec`"*. Đo lại khi bộ có **bốn** spec: **4/4 xanh**, hai lượt liên tiếp, **3m07** và **3m04** — triệu chứng **không còn tái lập được**, nên phần "cổng theo worker" là công việc cho một vấn đề đã biến mất
**And** 🔴 nguyên nhân lượt trượt cũ **không được chẩn đoán**, và không bản vá nào được nhận công mà không có phép đo nói thế
**And** bộ chạy **tuần tự** (`maxInstances: 1`) là một **quyết định**, không một chỗ chưa làm tới: `onPrepare` cấp **một** `$APPDATA` tạm và **một** thư mục Library tạm cho cả lượt, nên chạy song song sẽ để hai app dùng chung chúng — đúng trạng thái AC2 vừa đóng, chỉ đổi từ *"e2e đụng dữ liệu người dùng"* thành *"hai ca e2e đụng nhau"*
**And** điều kiện mở lại nếu có ngày cần song song: cấp thư mục tạm **theo worker** trước, rồi đổi phép tự kiểm ở `onComplete` theo

**Given** bản phát hành
**When** kiểm
**Then** `tauri-plugin-wdio-webdriver` và `axum` **VẮNG MẶT** khỏi cây phụ thuộc mặc định — **AD-45**, canh bởi `check-deps.mjs` **Kiểm 1b**
**And** hai lớp chặn *(feature ngoài `default` **và** `debug_assertions`)* đều phải còn nguyên; gỡ một lớp là mở lại đúng cái cửa AD-45 vừa đóng

**Given** phạm vi story này
**When** so với **28** hàng bàn đo treo của Story 1.20 + 1.21
**Then** story này dựng **ĐƯỜNG**, không viết trọn 28 hàng
**And** 28 hàng thuộc action item **A4**, và vế **thẩm mỹ** của chúng vẫn cần mắt Ice — WebDriver chụp được ảnh, nó không **phán xét** ảnh
