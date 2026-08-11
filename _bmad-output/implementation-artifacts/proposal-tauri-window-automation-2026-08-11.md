# Đề xuất — tự động đọc và lái CỬA SỔ TAURI THẬT

**Ngày:** 2026-08-11 · **Cho:** Ice · **Sinh ra từ:** retrospective Epic 1, action item **A4/A5/A7**
**Trạng thái:** đề xuất, chưa cài một dòng nào. Cần Ice ký trước Bước 1.

---

## 1. Câu trả lời ngắn

**Có.** Và nó khác hẳn phương án Chrome mà chính tôi đã can ở §5 báo cáo retrospective:
công cụ này chạy **bên trong webview thật của sản phẩm** — WKWebView trên macOS, WebView2
trên Windows — nên nó **tái lập được** đúng lớp lỗi đắt nhất của Epic 1.

Nhắc lại phát hiện hạng cao của lượt review Story 1.21: *"đường chuột của AC2 chết hoàn
toàn trên macOS vì WKWebView không đặt tiêu điểm cho `<button>`"*. Một bộ chạy trong Chrome
**không** bắt được nó. Một bộ chạy trong WKWebView **có**. Đó là toàn bộ khác biệt giữa
"mua sự yên tâm sai" và "đóng được món nợ".

---

## 2. Ba lựa chọn có thật — số đọc từ crates.io ngày 2026-08-11

| | `tauri-driver` | `tauri-plugin-wdio-webdriver` | `tauri-plugin-webdriver` |
|---|---|---|---|
| Chủ | chính thức, `tauri-apps/tauri` | tổ chức **WebdriverIO** | cá nhân (Choochmeque) |
| Bản mới nhất | `2.0.6` | **`1.3.0`** | `0.2.1` |
| Cập nhật lần cuối | 2026-05-06 | **2026-08-03** *(8 ngày)* | 2026-02-17 *(6 tháng)* |
| Lượt tải | 269.037 | 37.052 | 83.537 |
| **macOS / WKWebView** | 🔴 **KHÔNG** | ✅ **CÓ** | ✅ có |
| Windows / WebView2 | ✅ | ✅ | ✅ |
| Kiến trúc | tiến trình ngoài, cần Edge WebDriver | **máy chủ W3C nhúng trong app**, `debug_assertions` | máy chủ nhúng + proxy ngoài |

`tauri-driver` **không dùng được cho nửa việc quan trọng nhất**: Apple không cung cấp
WebDriver cho WKWebView, và issue `tauri-apps/tauri#7068` mở từ 2023 vẫn chưa đóng. Tài
liệu chính thức Tauri v2 nay ghi thẳng rằng macOS chỉ được hỗ trợ qua **máy chủ nhúng**.

**Đề xuất: `tauri-plugin-wdio-webdriver` + `@wdio/tauri-service`.** Lý do chọn nó thay vì
bản của Choochmeque: cùng năng lực, nhưng nó do chính tổ chức WebdriverIO bảo trì và mới
được cập nhật tuần trước, trong khi bản kia đứng yên nửa năm. Với một kho ghim phiên bản
bằng `=` và cấm `cargo-audit`, một phụ thuộc bỏ hoang là một khoản nợ chứ không phải một
món hời.

---

## 3. Bốn ràng buộc của CHÍNH kho này mà đề xuất phải đi qua

Đây là phần quyết định đề xuất này sống hay chết, và cả bốn đều **đo được**, không suy đoán.

### ① `capabilities/` chỉ được có ĐÚNG một tệp — và nó có một cổng canh

`src-tauri/tests/config_invariants.rs` mang hai bất biến:

- `capabilities_directory_holds_exactly_the_one_reviewed_file` — quét **đệ quy**, **mọi**
  phần mở rộng, và đòi thư mục chứa đúng `main.json`.
- `main_capability_grants_the_minimum_and_no_plugin_permission` — cấm **mọi** permission
  của plugin.

Tài liệu WebdriverIO bảo thêm `"wdio-webdriver:default"` vào `capabilities/default.json`.
Làm đúng như thế sẽ **phá cả hai** bất biến cùng lúc.

🔴 **Đường đi mà không phá cái nào — đã kiểm chứng trong mã của `tauri` 2.11.5 đang ghim:**
`Manager::add_capability` (`tauri-2.11.5/src/lib.rs:813`, sau feature `dynamic-acl` **bật
mặc định**) nạp một capability **lúc chạy** từ một chuỗi. Nên:

```rust
#[cfg(debug_assertions)]
{
    app.add_capability(include_str!("../capabilities-dev/wdio.json"))?;
    builder = builder.plugin(tauri_plugin_wdio_webdriver::init());
}
```

Thư mục `capabilities-dev/` **không phải** `capabilities/`, nên cổng đệ quy không thấy nó,
và `main.json` không mọc thêm một dòng permission nào. Hai bất biến giữ nguyên **nghĩa gốc**
thay vì bị nới để lọt một ca.

⚠️ Kèm theo: phải thêm **một bất biến thứ ba** khẳng định `capabilities-dev/` chỉ tới được
qua `#[cfg(debug_assertions)]`. Không có nó thì ta vừa mở đúng cái cửa mà hai cổng kia đóng.

### ② `check-deps.mjs` — danh sách cấm, và doctrine đằng sau nó

`BANNED_CRATES` cấm đích danh sáu crate, trong đó **bốn** là `tauri-plugin-*`
(`fs` · `sql` · `dialog` · `stronghold`). Lý do ghi tại chỗ: AD-1 + AD-29 — *không phơi
filesystem ra JS*. `tauri-plugin-wdio-webdriver` **không** phơi filesystem, nhưng nó là một
plugin Tauri thứ nhất trong một kho tới nay có **0** plugin. Đó là một tiền lệ, và tiền lệ
cần chữ ký chứ không cần một lượt `cargo add`.

### ③ AD-15 — nó là một socket LẮNG NGHE, và AD-15 chỉ nói về ĐIỂM RA

Kiến trúc khai **ba điểm ra mạng**. Một máy chủ WebDriver không phải điểm ra — nó là một
cổng **lắng nghe** trên `localhost`. AD-15 không nói gì về chiều đó, nghĩa là hôm nay
**không có luật nào** cho nó. Một bề mặt mới không có luật là đúng thứ kho này săn.

Đề xuất: bản build phát hành phải **chứng minh được** không có listener, không phải hứa.
`scripts/check-scope-bundled.mjs` đã dựng sẵn khuôn *"chạy bản đóng gói thật rồi đọc
`VERDICT:`"* — thêm một mệnh đề vào đúng harness đó, chứ đừng dựng harness thứ hai.

### ④ NFR15 và cái giá npm — rủi ro đo được TRƯỚC khi cài

`check-deps.mjs` chạy `npm ls --all --json`, tức **gồm cả `devDependencies`**, rồi quét
`PATTERN = /sentry|…|telemetry|analytics|…/i` trên **toàn cây**. Kho hôm nay có **3** dependency
và **8** devDependency. Cây WebdriverIO là hàng trăm gói.

🔴 **Nếu một gói trong cây đó mang chữ `telemetry` hay `analytics` trong TÊN, cổng đỏ ngay
và cả đề xuất này dừng.** Đó là phép đo **Bước 0** ở dưới — chạy trước khi tiêu một giờ nào.

---

## 4. Kế hoạch bốn bước, mỗi bước có nghiệm thu đo được

### Bước 0 — phép đo rẻ nhất, chạy TRƯỚC mọi thứ

Trong một thư mục nháp **ngoài** kho:

```
npm i -D @wdio/cli @wdio/tauri-service
npm ls --all --parseable | grep -iE "sentry|bugsnag|rollbar|crashlytics|datadog|newrelic|posthog|amplitude|mixpanel|segment-io|telemetry|analytics|opentelemetry|firebase"
npm ls --all --json | node -e "…" # đếm tổng số gói, và liệt kê giấy phép không phải MIT/Apache-2.0/BSD
```

**Nghiệm thu:** grep trả **rỗng**, và bảng giấy phép không có mục nào ngoài danh sách NFR15
cho phép. Trả về hai con số: tổng gói thêm vào, và tổng byte `node_modules`.

**Nếu grep có kết quả: DỪNG.** Báo số cho Ice; đừng nới `PATTERN` — nó là một cổng của AC5
Story 1.2, và nới nó để lọt một công cụ test là đúng thứ lượt review 1.21 đã bác.

### Bước 1 — một hàng, một ảnh chụp, trên macOS

Chỉ dựng đủ để lái **đúng một** hàng bàn đo và chụp lại: hàng **17** của Story 1.21
(UX-DR17 — tiêu điểm quay đúng về nút đã mở). Chọn nó vì nó **thuần DOM + tiêu điểm**, tức
đúng lớp mà bản vá code review đã chạm và không đo được bằng máy.

**Nghiệm thu:** một ảnh `.png` thật, cộng một khẳng định WebDriver đỏ-rồi-xanh *(hoàn nguyên
bản vá `@focusin` ⇒ ca ĐỎ; khôi phục ⇒ XANH)*. Không có vế đỏ thì ta chỉ chứng minh bộ chạy
khởi động được, không chứng minh nó **đo** được.

### Bước 2 — hàng đắt nhất, và là lý do cả đề xuất này tồn tại

Lái hàng **15** của Story 1.21: vòng gán phím **không chạm chuột**, trên WKWebView.

**Nghiệm thu:** hoàn nguyên bản vá *"ép tiêu điểm lúc arming"* ⇒ ca phải ĐỎ **vì đúng lý do
WKWebView**, không vì một timeout. Đây là phép thử của cả phương án: nếu bộ chạy không tái
lập được khuyết tật WKWebView đã biết, nó **không** thay được mắt người và ta dừng ở Bước 2
thay vì dựng tiếp 28 hàng.

### Bước 3 — nối vào `ci.yml`, và đây là chỗ nợ Windows tự đóng

Thêm **một** bước vào job `check` đang có *(AC4 của Story 1.3 cấm tệp workflow thứ hai)*.
Ma trận đã sẵn hai nền tảng, nên cùng một bộ kịch bản chạy trên **cả** `macos-26` lẫn
`windows-2025` mà không khai thêm gì.

🔴 **Đây là điểm ăn tiền lớn nhất và nó dễ bị bỏ sót:** món nợ *"cần một máy Windows"* mà
1.6 · 1.14 · 1.15 · 1.16 · 1.17 · 1.18 · 1.19 · 1.20 · 1.21 đều để lại và **chưa story nào
đóng được** — CI **chính là** máy Windows đó. Hàng 16 của Story 1.21 không cần Ice mua máy;
nó cần một bộ chạy đặt đúng chỗ.

**Nghiệm thu:** hàng 16 chạy XANH trên `windows-2025` với ảnh chụp làm tạo tác.

---

## 5. Cái nó KHÔNG đóng được — ghi thẳng

- **Thẩm mỹ.** *"Nét chữ trông đúng chưa"*, *"hai vệt sash có xấu không"* — WebDriver chụp
  được ảnh, nó không **phán xét** ảnh. Mọi hàng dạng *"Ice nhìn và thấy sai"* — như lượt bắt
  sash vẽ đè lớp phủ Attribution ở bàn đo 1.20 — vẫn cần mắt Ice.
- **Ba lớp hệ điều hành:** menu Tauri gốc, hộp thoại hệ thống, và IME. WebDriver dừng ở mép
  webview.
- **Bản phát hành.** Máy chủ chỉ sống trong build debug — có chủ ý. Nghiệm thu bản `release`
  thật vẫn là một lượt riêng.
- **Nó không rẻ ở lượt đầu.** 28 hàng treo phải được **viết lại thành kịch bản**; đó là một
  lượt dịch từ văn xuôi sang mã, không phải một lượt bật công tắc.

---

## 6. Cái giá, và vì sao tôi vẫn đề xuất

Giá: một plugin Tauri đầu tiên trong kho · một cây npm lớn hơn hẳn · một bất biến mới phải
viết · một tiền lệ cần Ice ký.

Đổi lại: món nợ nghiệm thu thị giác của Epic 1 là món **duy nhất có hệ số nhân** — Story
1.21 đi từ 12 hàng treo lên 19 **sau** khi vá mười phát hiện, vì mọi bản vá tầng DOM đều
nằm ngoài tầm của cả mười một cổng. Epic 2 dựng Panel Editor, bề mặt thị giác lớn nhất dự
án tới nay. Món nợ này sẽ không nhỏ đi.

Và có một dữ kiện vừa xảy ra hôm nay đáng cân: nửa Windows của `cargo test` **chết im lặng
sáu ngày** và không ai biết, vì không có gì tự nói. Bàn đo chạy tay có đúng tính chất đó —
nó chỉ chạy khi có người nhớ ra.

---

## 7. Ice cần ký ba câu

1. **Chạy Bước 0 chứ?** Nó rẻ, chạy ngoài kho, và có thể giết cả đề xuất trong một lượt grep.
2. **Chấp nhận tiền lệ "plugin Tauri đầu tiên"** — với ràng buộc `debug_assertions` +
   `capabilities-dev/` + một bất biến mới, hay giữ luật *"0 plugin"* tuyệt đối?
3. **Dừng ở Bước 2 nếu WKWebView không tái lập được khuyết tật đã biết** — tôi đề xuất
   *có*, và ghi ra ở đây để lượt sau không lặng lẽ đi tiếp.
