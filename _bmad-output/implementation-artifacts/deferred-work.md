
## Deferred from: code review of 1-1-mui-tham-do-font-do-dung-luong-that-va-ra-giay-phep (2026-08-03)

- **`.memlog.md` của architecture còn `scope: 112 FR, 16 NFR`** — spine ghi 131 FR / 19 NFR, PRD hiện có 132 FR. Tệp memlog đã bị chạm trong lượt này (bump `updated`) nhưng dòng `scope` lỗi thời vẫn để nguyên. Có sẵn từ trước, không do Story 1.1 gây ra. **(Chủ: Winston — architect.)**
- **Chưa đo trên Apple Silicon / universal binary / Windows ARM64** — máy đo là Intel x86_64. Chênh lệch font gần như chắc chắn không đổi theo kiến trúc nhưng baseline thì có, và universal binary nhân đôi baseline. Báo cáo đã nêu ở §Việc chưa làm được nhưng không story nào nhận việc. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**
- **Chưa khai artifact phát hành chính thức cho Windows** — Tauri dựng được cả `.msi` lẫn NSIS. AC1 đòi `.msi`, nhưng nếu bản phát hành thật là NSIS thì con số NFR6 không áp cho thứ người dùng tải về. Thuộc Story 1.3 / 10.2. **(Chủ: Story 1.3 / 10.2.)**
- **Đường nạp font chưa từng chạy trên Windows** — cấu hình CSP + `assetProtocol` scope + `FontFace` API mới chỉ kiểm chứng trên macOS. ~~CI của Story 1.3 chỉ `cargo test` và build, không xác minh font nạp được lúc chạy~~ *(mệnh đề này đã SAI kể từ `ci.yml` có `check:scope:bundled` — chiều dương đi qua `font-src`)*. Thuộc Story 1.3 / 1.4.
  → ⚠️ **ĐÓNG MỘT NỬA 2026-08-03 (Story 1.4, Task 4).** Story 1.4 đưa **đường nạp thật của sản phẩm** (`src/tokens/fonts.ts`, bốn `FontFace` qua `resolveResource` + `convertFileSrc`) vào cùng pipeline. **Còn mở, và ghi thẳng thay vì đánh dấu đạt:**
  - *Nhìn thấy chữ hiện đúng nét trên Windows* vẫn cần một lượt runner có ảnh chụp. Bốn nét của `Source Sans 3` mới chỉ được dựng trên **Blink/macOS** (Chrome headless 2×).
  - **WKWebView chưa đo.** Đối chứng *thiếu* descriptor `{ weight: '200 900' }` **vẫn ra nét đúng trên Blink** — Blink đọc `fvar` và nội suy trục dù `@font-face` không khai dải nét. Nên bẫy "khoá ở `wght = 200`" mà `ARCHITECTURE-SPINE.md` cảnh báo **chưa tái lập được trên engine nào**, và mức độ nghiêm trọng thật của nó vẫn là ẩn số. Descriptor vẫn bắt buộc (đặc tả dựa vào nó), chỉ là *lý do* bắt buộc yếu hơn tài liệu đang khẳng định.
  - **`-webkit-font-smoothing: antialiased` trong `reset.css` là LÝ LẼ, chưa phải PHÉP ĐO** — chưa có ảnh chụp cạnh nhau của cùng một chuỗi trên hai nền tảng để chứng minh nó thu hẹp khoảng cách độ đậm nét thay vì nới rộng.
- ✅ **ĐÃ ĐÓNG 2026-08-03 ngay trong lượt rà soát** — ~~Ba tệp giấy phép OFL chưa có AC nào đưa vào bundle~~ — báo cáo kết luận *"cả ba tệp giấy phép gốc phải đi kèm bản phát hành (FR38, FR109)"* nhưng không story nào cưỡng chế điều đó, và phép đo 20,30 MiB cũng chưa gồm chúng. Thuộc Story 1.2 / 10.5.
- **Rà NFR15 chưa đọc name ID 13/14 của tệp font phát hành** — đã mở `LICENSE` / `OFL.txt` trong zip mà đọc (đúng yêu cầu "rà tường minh"), nhưng chưa đối chiếu với trường License Description nhúng trong chính tệp `.otf`/`.ttf` sẽ được đóng gói. **(Chủ: Story 1.2 / 10.5.)**

## Deferred from: code review of 1-2-scaffold-du-an-va-khoa-pham-vi-filesystem-pham-vi-mang (2026-08-03)

- **Tổ hợp CSP + asset protocol của bản RELEASE chưa phép kiểm nào chạm tới** — `check-scope` chạy `tauri dev`, nơi Tauri **không** áp CSP (webview nạp HTML từ Vite qua `devUrl`; Tauri chỉ chèn header CSP cho HTML nó tự phục vụ qua asset protocol — `tauri-2.11.5/src/manager/mod.rs:438-452`). Ở release thì có CSP, và `fetch` tới asset protocol do `connect-src` quyết — tổ hợp đó chưa ai đo. **Lý do hoãn (Ice chốt 2026-08-03):** kiểm trên release đòi một bản build release, mà Story 1.3 đã nhận sẵn việc dựng bản đó trên cả hai nền tảng — dựng riêng ở 1.2 là pipeline thứ hai mà AC của 1.3 cấm tường minh. Thuộc Story 1.3.
  → ✅ **ĐÃ ĐÓNG 2026-08-03 (Story 1.3, AC8)** — `npm run check:scope:bundled` dựng `tauri build --debug` (giữ `debug_assertions` ⇒ móc self-check còn, nhưng webview nạp HTML qua asset protocol ⇒ **CSP có áp**) rồi chạy nhị phân với timeout cứng. Đo thật trên macOS: chiều **dương** ĐẠT qua `font-src` — đúng đường Story 1.4 sẽ dùng. Chiều **âm** ghi `[----] unmeasured` kèm lý do, xem mục `connect-src` bên dưới. Đã gắn vào `.github/workflows/ci.yml`, chạy trên cả hai nền tảng. ⚠️ **Giới hạn ghi thẳng:** phép kiểm này chứng minh **tổ hợp CSP + asset protocol**, KHÔNG chứng minh hành vi của nhị phân profile **release** — móc self-check là `#[cfg(debug_assertions)]` và profile release đang bị cố ý đóng băng để giữ số đo NFR6 so sánh được.
- **NFR6 phải đo lại: `webviewInstallMode` đổi sang `offlineInstaller`** — Ice chốt 2026-08-03 ưu tiên lời hứa *"fully offline"* hơn ngưỡng dung lượng. Story 1.1 đã cảnh báo một mình chế độ này đủ làm `.msi` phình ~150 MB và vỡ NFR6. Con số NFR6 cũ **không còn áp dụng** cho bản Windows. Thuộc Story 1.3 / 10.2, đi cùng hai phép đo `.msi` đã bàn giao từ Story 1.1.
  → ✅ **Phần quyết định đã đóng 2026-08-03 (Ice, lúc dựng Story 1.3).** Trần 150–200 MB của NFR6 nay là trần của **payload sản phẩm**; **bản WebView2 Runtime nhúng nằm NGOÀI ngân sách** và ghi thành dòng riêng trong mọi phép đo. Đã sửa `prd.md` §7.2 (NFR6 + giả định A2) và `epics.md` (§NFR6 · bản đồ NFR · ghi chú Epic 1 · AC6 Story 1.3 · AC Story 1.9 · AC Story 10.9). **Phần phép đo vẫn mở** — Story 1.3 Task 5/6 đo hai dòng và ghi cả số tổng; đường quay lui (`downloadBootstrapper` · NSIS thay `.msi`) giữ mở ở Story 10.2.
- **`$RESOURCE/dict/**` nằm trong `assetProtocol.scope` nhưng không nằm trong `bundle.resources`** — `tauri.conf.json:28` khai scope hai mục, `:36` chỉ đóng gói `resources/fonts/*`. Có chủ ý và story đã ghi lý do (thư mục còn rỗng; glob không khớp tệp nào có thể làm `tauri build` gãy). Nhưng **không phép kiểm nào nối scope với `bundle.resources`**, nên Story 10.1 có thể ship một bản không có byte từ điển nào mà test `asset_protocol_scope_has_exactly_the_two_readonly_resource_areas` vẫn xanh — lỗi chỉ lộ ở lần tra cứu đầu tiên của người dùng thật. Thuộc Story 1.9 / 10.1.
  → ✅ **ĐÃ ĐÓNG 2026-08-04 (Story 1.9, Task 10 — Ice phê chuẩn).** Giải theo hướng SIẾT, không NỚI: gỡ `$RESOURCE/dict/**` khỏi `assetProtocol.scope` (webview không bao giờ đọc tệp từ điển — AD-1/AD-11, `rusqlite` mở tệp qua đường dẫn hệ thống). `scope` giờ đúng MỘT mục `["$RESOURCE/fonts/**"]`; test đổi tên thành `asset_protocol_scope_has_exactly_the_one_readonly_resource_area`. Mâu thuẫn scope/`bundle.resources` **biến mất theo cấu trúc** — không còn gì để nối vì không còn `dict` trong scope. Lưới thay thế cho Story 10.1: xem mục mới `## Deferred from: 1-9-dung-du-lieu-tu-dien-lop-nen`.
- **`panic = "abort"` + `strip = true` + không crash reporter → crash release là hộp đen, và giết đường checkpoint của AD-12** — `src-tauri/Cargo.toml:56-61`. Profile được cố ý đóng băng để giữ số đo NFR6 của Story 1.1 so sánh được. Nhưng `core::store` có một writer nối tiếp (AD-11) và tự quyết checkpoint (AD-12); một `panic!` trong luồng writer với `panic = "abort"` chấm dứt tiến trình ngay — không unwind, không `Drop`, không cơ hội flush WAL. Trên Windows release `windows_subsystem = "windows"` khiến `.expect()` ở `lib.rs:36` cũng không in ra đâu. Thuộc Story 1.7 + lượt đo lại NFR6.
  → 🟡 **ĐÓNG MỘT NỬA 2026-08-04 (Story 1.7).** Ice chốt: **không đụng `[profile.release]`** — cùng tệp này *(mục [D4])* ghi quyết định không sửa `Cargo.toml`, và sửa profile làm số `.dmg`/`.msi` khác đi, nên nếu làm thì phải làm **trước** khi chốt baseline NFR6, tức thuộc **Story 1.9 / 10.9**, không phải story tầng dữ liệu.
  **Story 1.7 đã đóng phần đóng được bằng THIẾT KẾ, không bằng cấu hình:**
  - **Luồng writer không panic.** Lỗi là **giá trị** đi ngược qua kênh phản hồi (`core/store/writer.rs`). Không `unwrap()`/`expect()` nào trong `core::store`; mutex khoá qua `unwrap_or_else(|e| e.into_inner())`; `let _ = reply_tx.send(…)` là cố ý *(chỗ gọi bỏ đi thì im lặng, không giết luồng)*. `catch_unwind` **vô dụng** ở đây và không được dùng — không có unwind để bắt.
  - **`Store::write` không bao giờ treo.** Writer chết hoặc kênh đứt ⇒ `StoreError::WriterGone` trong thời gian hữu hạn *(nghiệm thu: `store_contract.rs::write_after_close_fails_instead_of_hanging`, có `recv_timeout` nên một bản treo ra test ĐỎ chứ không ra một CI đứng)*.
  - **`wal_checkpoint(TRUNCATE)` lúc thoát** qua `RunEvent::Exit` của `lib.rs`, có **trần thời gian** *(`Tuning::close_truncate_budget`)* để không làm `check:scope` / `check:scope:bundled` đỏ.
  **VẪN CÒN HỞ, ghi thẳng thay vì đánh dấu đạt:**
  - **Thoát cứng thì KHÔNG có lần flush cuối.** `panic = "abort"` ở bất kỳ đâu trong tiến trình *(kể cả ngoài `core::store`)*, `SIGKILL`, mất điện — cả ba không đi qua `RunEvent::Exit`, nên `.db-wal` ở lại nguyên cỡ. ⚠️ Dữ liệu **không mất** *(đó chính là điều WAL bảo đảm — lần mở sau SQLite chép lại từ WAL)*; thứ mất là lượt dọn dẹp, và hệ quả là một `.db-wal` lớn tồn tại giữa hai phiên. AC5 giữ nó có trần khi ứng dụng đang chạy; không cơ chế nào cắt nó khi ứng dụng chết đột ngột.
  - **Crash release vẫn là hộp đen** — `strip = true`, không crash reporter, `windows_subsystem = "windows"`. Story 1.7 không chạm tới vế này.
  **Giao lại: Story 1.9 / 10.9** *(quyết định `[profile.release]` + lượt đo lại NFR6)*. Không phải story tầng dữ liệu nào nữa.
- ~~**NFR16 ("không chuỗi tiếng Việt trong `.vue`") không có cơ chế cưỡng chế nào**~~ — `src/App.vue:5` chỉ có một comment, trong khi thứ khó vi phạm hơn hẳn (lỡ cài `tauri-plugin-fs`) thì có cả script + mã thoát. Quy tắc này vi phạm chỉ cần gõ một nhãn button; đến Story 1.14 sẽ có hàng chục chuỗi lọt vào trước khi ai nhớ ra. Thuộc Story 1.5 (NFR16, AD-21).
  → 🟡 **ĐÓNG MỘT PHẦN 2026-08-04 (Story 1.5; phạm vi thu hẹp lại sau lượt code review cùng ngày — Ice chốt).** Cơ chế: `scripts/check-i18n.mjs` — năm phép kiểm, mã thoát là phán quyết — gắn thành **một** bước `npm run check:i18n` trong job `check` đã có của `ci.yml`, kề `check:tokens` và **trước** `npm run build`. Kiểm A quét **có trạng thái** trên `src-tauri/**/*.rs` và `src/**/*.vue`; comment tiếng Việt **không** phải vi phạm, chuỗi và text node thì có. Miễn trừ **có tên, có lý do, in ra số tệp mỗi lượt chạy**: `src-tauri/tests/**` và `src/selftest/**`. Nghiệm thu đỏ-rồi-xanh: 16 ca lúc dựng, cộng **23 ca của lượt code review** (13 ca hỏng + 10 đối chứng âm — lifetime `&'a str`, `'\''`, phép chia `a / b`, `{{ a < b }}`, attribute chứa `>`, thẻ tự đóng, `url(//host)` …).
  ⚠️ **Giới hạn 1 — phạm vi TỆP.** Quét `.rs` và `.vue`, **đúng phát biểu AC2 và Story 10.9, không hơn**. Tệp `.ts` (`src/tokens/fonts.ts`, `src/selftest/scopeCheck.ts`, `src/i18n/resolve.ts`) mang chuỗi tiếng Việt ở vị trí mã và **không** bị cổng nào canh. Hôm nay tất cả là chẩn đoán/log chứ không phải chuỗi hiển thị, nhưng mệnh đề đó do **người đọc** giữ, không do máy giữ. Hệ quả cụ thể: "dời một chuỗi từ `.vue` sang một tệp `.ts`" là một cách hợp lệ về mặt cổng để cho xanh mà không đổi gì về bản chất — đừng dùng. Thuộc **Story 1.14 / 10.9**.
  ⚠️ **Giới hạn 2 — cổng đo DẤU, không đo CHUỖI HIỂN THỊ** *(phát hiện ở lượt code review 2026-08-04)*. Kiểm A nhận dạng 134 ký tự có dấu tiếng Việt. `<button>Xem</button>`, `<button>Dong</button>`, `<button>Save</button>` **đều xanh** — chúng là nhãn giao diện thật, chỉ tình cờ không có dấu. AC2 phát biểu nguyên văn *"grep chuỗi tiếng Việt"* nên cài đặt đúng phát biểu, nhưng NFR16 rộng hơn: *"chuỗi hiển thị sống ở `vi.json` và chỉ ở đó"*. **Ice chốt: giữ nguyên cổng, không mở rộng phạm vi trong Story 1.5** — một phép kiểm cấu trúc (`text node phải là {{ t('…') }}`) là phạm vi mới và sẽ báo thừa trên `App.vue` hiện tại. Mở lại ở **Story 1.14**, khi bốn panel thật có nhãn thật để định nghĩa "đúng" nghĩa là gì.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, Task 13 · §Quyết định #6).** `scripts/check-i18n.mjs` **Kiểm A2**: mọi **text node** của template `.vue` phải, sau khi gỡ hết khối `{{ … }}`, không còn chữ cái/chữ số — và mọi khối đã gỡ phải mở đầu bằng `t(` hoặc `tError(`. Dùng lại chính máy trạng thái template của Kiểm A (không dựng bản chép thứ hai). `<button>Dong</button>`, `<span>Save</span>`, `<i>3 muc</i>`, `{{ label }}` — cả bốn **nay ĐỎ**. Miễn trừ **có tên** `<!-- aura-allow-text: <lý do> -->`, tìm được cả khi comment đứng trước THẺ và khi nó trải nhiều dòng; mọi miễn trừ **in ra mỗi lượt chạy** (hôm nay: **2** — `announcedConfigError` và `report.text` ở `App.vue`, cả hai có lý do viết tại chỗ). Nghiệm thu đỏ-rồi-xanh **14 ca** (7 đỏ + 7 đối chứng âm, gồm hai ca miễn trừ và một ca miễn trừ ĐẶT SAI CHỖ vẫn phải đỏ).
- **`.shell { min-height: 100vh }` + margin 8px mặc định của `<body>` sinh thanh cuộn ở cửa sổ trống** — `src/App.vue:31`. Không có reset CSS toàn cục, `index.html` không nạp stylesheet nào. Thuộc Story 1.4.
  → ✅ **ĐÃ ĐÓNG 2026-08-03 (Story 1.4, Task 6)** — `src/tokens/reset.css` (`box-sizing` toàn cục · `html, body { margin: 0; padding: 0 }` · nền/chữ từ token · `-webkit-font-smoothing`), import trong `main.ts`. Không kéo `normalize.css` về: mỗi phụ thuộc mới phải rà GPLv3 và vào bảng Stack trước (NFR15), và bốn quy tắc không đáng một lượt rà.

## Deferred from: code review of 1-5-tai-nguyen-chuoi-giao-dien-va-hinh-dang-loi-qua-ipc (2026-08-04)

*Sáu mục dưới đây là phát hiện của lượt code review Story 1.5 được xếp **hoãn** — thật nhưng chưa tới lúc hành động. Mọi phát hiện `patch` và `decision` của cùng lượt nằm ở §Review Findings của story file.*

- **Tệp nguồn tới qua symlink bị loại khỏi Kiểm A và không tính vào sàn** — `scripts/check-i18n.mjs:162-165` đẩy symlink vào `skippedLinks` rồi `continue`; nó chỉ hiện ra như một dòng `detail(...)` ở `:603`, **không bao giờ là `fail`**. Với 18 tệp `.rs` trên sàn 14 có đủ dư địa để giấu tệp bằng đường này mà sàn vẫn qua. Hoãn: cây hiện không có symlink nào. Mở lại nếu một `.vue`/`.rs` symlink xuất hiện. **(Chủ: một story hạ tầng cổng kế tiếp.)**
- **Gốc quét cứng ở `src/` và `src-tauri/`** — `scripts/check-i18n.mjs:179-180`. Comment `:175-178` lập luận glob được cố ý **nới rộng** và mọi thu hẹp phải đi qua `EXEMPT` có tên; nhưng chính hai gốc này là một lần thu hẹp lặng lẽ. Một `packages/`, `examples/` hay `e2e/` về sau vô hình với cổng trong khi `vueFiles.length >= 1` vẫn đúng. Hoãn: chưa có thư mục nào ngoài hai gốc. Mở lại khi cây mọc nhánh thứ ba.
  → ✅ **ĐÃ ĐÓNG 2026-08-04 (Story 1.9, Task 9).** `tools/` (chứa `tools/dict-build`) là nhánh thứ ba. Đã thêm vào gốc quét của `check-i18n.mjs` **và** miễn trừ TRỌN nó ở `EXEMPT` với tên + lý do (build tool không vào bản phát hành — AD-25, không có bề mặt giao diện, chuỗi của nó là chẩn đoán cho người dựng). Quần thể in ra sau miễn trừ **không đổi** — vẫn 27 `.rs` + 5 `.vue` — đúng như doctrine đòi: thêm gốc quét không phải cái cớ để quần thể phình lên trong im lặng.
- **`scanStyle` không có trạng thái `line_comment`** — `scripts/check-i18n.mjs:455-499`. Doc `:455` biện minh đúng cho CSS thuần (`url(//host/x.png)` là URL, không phải comment), nhưng trong `<style lang="scss">` thì `//` **là** comment và một comment tiếng Việt ở đó sẽ bị báo là vi phạm. Đúng kiểu hỏng đắt nhất — cổng đỏ trên comment thì bị gỡ trong tuần. Hoãn: chưa có `.scss` nào và không gì trong repo cấm dùng. Mở lại ngày đầu tiên có `lang="scss"`. **(Chủ: một story hạ tầng cổng kế tiếp.)**
- **Sàn đếm tệp, không đếm nội dung** — `scripts/check-i18n.mjs:207-218`. `VUE_FLOOR = 1` được thoả bởi một `src/App.vue` chỉ có khoảng trắng. Sàn đóng được *"cây rỗng đọc thành sạch"* nhưng không đóng *"tệp rỗng đọc thành sạch"*. Hoãn: rủi ro thấp khi cây `.vue` còn đúng một tệp; mở lại khi Story 1.14 dựng bốn panel.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, AC11.1).** `VUE_FLOOR` 1 → **9** (thật: 11), `RS_FLOOR` 21 → **26** (thật: 32) ở `check-i18n.mjs`; `VUE_FLOOR` 4 → **9**, `TS_FLOOR` 10 → **16**, `COMMAND_FLOOR` 4 → **10** ở `check-commands.mjs`; `FILE_FLOOR` 5 → **26**, `COMPONENT_FILE_FLOOR` 4 → **23** ở `check-tokens.mjs`. Con số THẬT ghi vào comment cạnh từng hằng số, đúng khuôn `RS_FLOOR` đang có. Nghiệm thu: di dời `src/panels/` ⇒ `check-i18n` · `check-commands` · `check-layout` đều `abort()` kèm số thật; thêm `src/layout/` ⇒ `check-tokens` cũng `abort()`. ⚠️ Sàn ĐẾM TỆP thì một tệp RỖNG vẫn qua — giới hạn đó **không** đóng ở đây; nó được bù bằng sàn NỘI DUNG (`CLICK_FLOOR`/`DISPATCH_FLOOR`/`COMMAND_FLOOR`, Kiểm B của `check-i18n`).
- **`ipc_error_wire_shape` assert "không ký tự có dấu nào trên dây" là một mệnh đề vòng** — `src-tauri/tests/ipc_contract.rs:128-137` quét bản serialize của chính literal `IpcError` mà test tự dựng ở `:73-78`. Nó chỉ đỏ khi ai đó sửa fixture, và không quan sát đường sản phẩm nào — vì chưa có đường nào (`commands/mod.rs` mới chỉ có doc-comment). Doc `:121-127` và §Completion Notes gọi nó là *"mệnh đề trung tâm của AD-21, kiểm được bằng máy"*, rộng hơn thứ nó làm được. ~~Hoãn tới **Story 1.6**, khi `#[tauri::command]` thật đầu tiên cho một đường thật để quan sát.~~
  → 🔴 **CHỦ SỞ HỮU ĐÃ SỬA 2026-08-04 (Story 1.6): KHÔNG phải Story 1.6.** Câu trên được viết lúc chưa ai đọc kỹ AC của Story 1.6; đọc rồi thì thấy **không AC nào của story đó cần Rust**. Chuyển chế độ, tiêu điểm bàn phím và bố cục panel là **state UI**, và AD-1 nói thẳng đó là phần frontend được phép sở hữu (*"frontend chỉ render và giữ state UI (focus, cuộn, vùng chọn, bố cục panel)"*). Story 1.6 giao **0 dòng Rust** và không từ chối dựng một `#[tauri::command]` giả chỉ để đóng mục này — cùng ba lý do Story 1.5 đã từ chối: nó là mã sản phẩm không ai gọi, chạy nó cần webview cộng một lượt biên dịch profile `dev` riêng (đắt nhất trên macOS, hệ số ×10), và vòng chạy thật đến **miễn phí** ở story đầu tiên có nhu cầu IPC thật.
  → ~~**Nhận lại ở: Story 1.8** *(phân giải cấu hình hai tầng — đường IPC thật đầu tiên có nhu cầu đọc/ghi qua ranh giới)*, hoặc **1.9/1.11** nếu đường tra cứu chạm Rust trước.~~
  → ✅ **ĐÓNG 2026-08-04 (Story 1.8).** `ipc_error_wire_shape` nay serialize giá trị mà `commands::config::bootstrap_config(None)` trả về — **đường sản phẩm thật**, đúng hàm mà `#[tauri::command] wire::bootstrap_config` bọc lại, chạy đúng nhánh mà một `$APPDATA` không ghi được sẽ chạy trên máy người dùng. Và **không** phải một command giả: hàm nhận `Option<&Store>` để test gọi được mà không cần webview *(§Quyết định #6)*, chứ không phải để test có một thứ riêng để gọi. Đối chứng âm N14 *(cho `bootstrap_config(None)` trả `Ok` với cấu hình mặc định)* làm ca này đỏ.
- **`process.exit()` ngay sau `console.log` có thể cắt cụt chẩn đoán trên pipe Windows** — `scripts/check-i18n.mjs:873,876`. Mã thoát — tức phán quyết — vẫn nguyên; thứ mất là các dòng `file:dòng:cột` làm cổng dùng được, đúng trên nền tảng mà cổng được viết bằng Node để có mặt. Hoãn tới lượt runner thật của Story 1.3; xác nhận trong cùng lượt đó. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**

## Deferred from: 1-3-ci-toi-thieu-hai-nen-tang-moi-lan-push (2026-08-03)

- 🔴 **`connect-src` thiếu `asset:` ⇒ `fetch()`/`XHR` tới asset protocol CHẠY ở dev, GÃY ở bản đã đóng gói — im lặng.** Đo thật trên bản `.app` debug 2026-08-03: bốn sự kiện `securitypolicyviolation` nêu đích danh `connect-src`, `blockedURI` là chính URL `asset://`. CSP hiện là `connect-src 'self' ipc: http://ipc.localhost` — `font-src` và `img-src` **có** `asset:`, riêng `connect-src` thì không.
  - **Hệ quả 1 (đã xử lý):** chiều ÂM của Kiểm 3 không đo được ngoài chế độ dev. Kênh duy nhất đọc được mã HTTP đã bị chặn, còn `FontFace` trả **cùng một** `NetworkError` cho "403 scope chặn", cho "tệp có thật nhưng không phải font" (đo thật với `OFL-sourcesans3.txt`), và cho "404" — nên nó cho cùng kết quả dù hàng rào còn hay mất. Story 1.3 ghi `unmeasured` thay vì đoán. **Ice chốt 2026-08-03: giữ nguyên CSP, không nới `connect-src` chỉ để một phép kiểm đo được.**
  - **Hệ quả 2 (CÒN MỞ, và đây mới là phần đắt):** story đầu tiên `fetch` một tài nguyên `$RESOURCE/**` từ webview sẽ chạy tốt suốt lúc phát triển rồi hỏng ở bản người dùng cài — đúng lớp lỗi mà dự án này liên tục đi săn. Đáng ngờ nhất: `$RESOURCE/dict/**` **đang nằm trong** `assetProtocol.scope` (tức là webview *được phép* đọc từ điển) nhưng CSP thì cấm `fetch` nó. **Hai khai báo đang mâu thuẫn nhau** — hoặc bỏ `$RESOURCE/dict/**` khỏi scope (nếu chỉ Rust đọc từ điển, đúng AD-11), hoặc thêm `asset:` vào `connect-src`. Nối thẳng với mục *"`$RESOURCE/dict/**` nằm trong scope nhưng không nằm trong `bundle.resources`"* ở trên. Thuộc **Story 1.9 / 10.1**, story nào chốt trước thì giải luôn cả hai.
    → ✅ **ĐÃ ĐÓNG 2026-08-04 (Story 1.9, Task 10).** Gỡ `$RESOURCE/dict/**` khỏi scope — mâu thuẫn biến mất theo chiều SIẾT, đúng tiền lệ Ice 2026-08-03 *"giữ nguyên CSP, không nới `connect-src` chỉ để một phép kiểm đo được"*. `connect-src` **không đổi** (không thêm `asset:`) — CSP vẫn siết y hệt trước.

- **Bốn phép nghiệm thu của Story 1.3 CHƯA chạy vì chưa có lượt CI thật** (Ice chốt 2026-08-03: chưa đẩy lên remote). Pipeline đã dựng và kiểm chứng hết phần chạy được ở máy, nhưng những thứ sau **chỉ runner mới trả lời được** — xem §Completion Notes của story để biết chính xác cái gì còn thiếu:
  - **AC6** — ba số `.msi` (có font · không font · `downloadBootstrapper`) và hai dòng nghiệm thu NFR6. `.msi` **chỉ dựng được trên Windows**.
  - **AC7** — thời gian tường + phút tính phí, cache lạnh và cache nóng, cả hai nền tảng. ⚠️ Rủi ro đã biết: job biên dịch Rust **hai profile** (`dev` cho AC8, `release` cho AC1/AC6) và dựng **ba** bản `.msi`; trên macOS hệ số ×10.
  - **Task 11 hàng 4** — `#[cfg(windows)] compile_error!` phải làm **chỉ** job Windows đỏ trong khi macOS **vẫn xanh** (phép kiểm của `fail-fast: false` và của AC1 *"tách bạch"*). Ba hàng còn lại đã nghiệm thu tại chỗ.
  - **AC3 / Task 4** — mệnh đề *rào biên dịch C của `zstd-sys` · `libsqlite3-sys` · `aws-lc-sys` biến mất trên `windows-2025`* mới là **kỳ vọng đọc từ tài liệu**, chưa ai đo. Cùng chỗ: WiX v3 — mũi thăm dò nói *"Tauri CLI tự tải lần build đầu"*, tài liệu Tauri nói phải cài sẵn; **hai nguồn nói khác nhau**, lượt chạy đầu phân xử. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**

## Deferred from: code review of 1-3-ci-toi-thieu-hai-nen-tang-moi-lan-push (2026-08-03)

*Lượt rà soát ba lớp song song trên dải `847e933..HEAD` (gồm cả code Story 1.2). Các mục dưới đây là **hoãn** — thật nhưng không thuộc phạm vi sửa ngay. Phần cần vá nằm ở §Review Findings của story 1.3.*

- **[D2] Trạng thái AC8 chốt sau lượt CI đầu tiên có `check:scope`** — Ice chốt 2026-08-03 trong lượt rà soát. Hôm nay `deferred-work.md:14` (mục của Story 1.2) ghi *"✅ ĐÃ ĐÓNG (Story 1.3, AC8)"* trong khi chiều **âm** là `unmeasured`, và AC8 không có trong danh sách *"Còn thiếu gì để đóng story — đúng bốn thứ"*. AC8 đòi *"**cả hai** chiều"*, và mệnh đề không của nó cấm *"đánh dấu đạt"*. Quyết định phụ thuộc D1: nếu `npm run check:scope` chạy được trên runner thì chiều âm có lưới tự động và AC8 **đóng trọn**; nếu runner không mở được webview thì **hạ `:14` xuống "đóng một nửa, đã trả lại cho Ice"** và thêm AC8 vào danh sách còn thiếu thành mục thứ năm. Không đánh dấu đạt trước khi có lượt chạy đó. **(Chủ: một story hạ tầng cổng kế tiếp — AC8 của Story 1.3 dường như đã đóng ở `:18`, chưa xác minh lại toàn bộ D1/D2.)**
- **[D3] `on: push` (mọi nhánh) + `on: pull_request` ⇒ ma trận chạy HAI lần cho mỗi commit trên nhánh có PR** — `ci.yml:26-27`. `concurrency.group: ci-${{ github.ref }}` (`:34`) không gộp được: push là `refs/heads/x`, PR là `refs/pull/N/merge` — hai group khác nhau nên `cancel-in-progress` không huỷ chéo. Repo **private**, macOS hệ số **×10**. **Ice chốt giữ cả hai trigger** (Task 2 yêu cầu tường minh cả hai): AC7 nghiệm thu bằng **số thật**, nên để lượt CI đầu đo đúng giá của việc nhân đôi rồi mới quyết — đúng §Ngân sách CI *"ghi số và dừng, không tự cắt"*. Ba đường xử nếu số không chịu nổi: bỏ qua loạt `pull_request` khi PR đến từ cùng repo · khoá `concurrency.group` theo `head_ref || ref` để hai loạt huỷ chéo được · giảm tần suất job nặng. *(Kèm theo và chưa giải: `cancel-in-progress: true` xoá luôn bảng số đo AC6 của lượt bị huỷ, trong khi AC6 đòi ghi số ở MỖI lần chạy.)* **(Chủ: một story hạ tầng CI kế tiếp.)**
- **[D4] Hai khoản chi biên dịch trong `Cargo.toml` đánh thẳng vào AC7** — **Ice chốt không đổi** (§File List không cấm đụng `Cargo.toml`; bảng Stack được cài trọn có chủ ý ở Story 1.2). Ghi lại để lượt tối ưu AC7 sau có chỗ bám: (a) `reqwest = "=0.13.4"` (`:52`) để nguyên default features nên kéo `aws-lc-sys` — biên dịch từ nguồn C — vào **mọi** lượt cache lạnh, trong khi chính manifest tự khai *"chưa có một dòng mã nào gọi tới"*; `default-features = false` bỏ được cả một stack TLS. (b) `crate-type = ["staticlib", "cdylib", "rlib"]` (`:16`) là để phục vụ iOS/Android của template Tauri, nhưng `bundle.targets` chỉ có `["dmg","msi"]` — hai artifact thừa được link ở mọi `cargo test` và cả **ba** lượt build release Windows dưới `lto = true` + `codegen-units = 1`. ⚠️ Sửa hai chỗ này làm số `.dmg`/`.msi` khác đi, nên nếu làm thì phải làm **trước** khi chốt baseline NFR6, không phải sau.
  → 🟡 **Trạng thái sau Story 1.9 (2026-08-04, §Quyết định của Ice #3): VẪN CHƯA ĐÓNG, chốt lần thứ ba là KHÔNG ĐỤNG.** Story 1.9 đo baseline NFR6 (`.dmg` cây nguồn hôm nay, không font/license: **2.334.696 byte**) TRÊN HIỆN TRẠNG hai khoản này — số đó **chưa phản ánh** khoản tiết kiệm nếu (a)/(b) được cắt. Tổng payload sản phẩm sau khi cộng `dict-core.db` là **178.492.550 byte**, còn cách trần 200.000.000 đúng **21.507.450 byte** — nếu Story 1.10 (bốn lớp gỡ rời) đẩy tổng sát trần, đây vẫn là hai đòn bẩy đầu tiên nên thử, và bây giờ đã có SỐ THẬT để cân nhắc thay vì một tối ưu mù.
  → 🔴 **Trạng thái sau Story 1.10 (2026-08-05): NFR6 đã VƯỢT trần THẬT, KHÔNG ĐỤNG vẫn chốt.** Story 1.10 giao hai lớp gỡ rời (Thiều Chửu + VietPhrase, phạm vi thu hẹp — Ice chốt 2026-08-05) và đo thật: tổng payload hôm nay **343.991.430 byte**, VƯỢT trần 200.000.000 byte đúng **143.991.430 byte**. VietPhrase một mình **160.083.968 byte**. Hai khoản (a)/(b) ở mục này **chưa đo tác động thật** — cần đo TRƯỚC khi dùng làm đòn bẩy, không suy đoán. Dev **không đụng** `Cargo.toml` ở story này (chốt lần thứ tư). Phán quyết đầy đủ + bảng kế toán: §Debug Log References Task 11 của story `1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap.md`. Quyết định xử lý VƯỢT: §Câu hỏi cho Ice #1 của story đó.
  → ✅ **CHỐT 2026-08-05 (Ice): CHẤP NHẬN VƯỢT TRẦN.** Không bỏ nguồn nào, không bỏ `sense_fts_nd` của lớp nào *(phá AC4)*, không đụng hai khoản (a)/(b) ở mục này. Payload sản phẩm **343.991.430 byte** trên trần **200.000.000** — vượt **143.991.430 byte**, và con số đó được **chấp nhận có ý thức** trên số ĐO THẬT, không phải bỏ sót. Mục [D4] này vì vậy **KHÔNG còn là đòn bẩy đang chờ** — nó trở lại đúng bản chất ban đầu: một khoản tối ưu **AC7 (thời gian build)**, không phải AC6 (dung lượng). 🔴 **Hệ quả cần Ice xử lý ở tầng PRD:** trần 200.000.000 byte của **NFR6 giờ mâu thuẫn với sản phẩm thật** — hoặc nâng trần, hoặc ghi rằng NFR6 không tính lớp gỡ rời *(VietPhrase 160.083.968 byte là lớp **gỡ rời**, mà FR36 nói sản phẩm phải chạy đầy đủ khi **không có** nó — đây là cách diễn giải tự nhiên nhất)*. Dev không sửa `prd.md`.

- **`timeout-minutes: 60` nhiều khả năng không đủ cho nhánh Windows ở lượt cache lạnh** — `ci.yml:59`. Nhánh Windows phải làm tuần tự: `npm ci` → `cargo tree` (giải toàn cây) → `npm run build` (vue-tsc ×2 + vite) → `cargo test` (biên dịch **profile dev** toàn cây gồm `aws-lc-sys`, `libsqlite3-sys`, `zstd-sys` từ nguồn C) → `tauri build --debug` → **ba** lượt `tauri build --bundles msi` **profile release** với `lto = true` + `codegen-units = 1`, mỗi lượt chạy lại `beforeBuildCommand`, và hai trong ba lượt tải ~127 MB WebView2 lúc build. Vượt 60 phút ⇒ job bị giết ⇒ mất cả số AC6 lẫn số AC7, và cái đỏ đó **trông giống hệt** một lỗi thật. Con số 60 là phỏng đoán, không phải đo — §Ngân sách CI nói **ghi số rồi để Ice quyết**, nên lượt chạy thật đầu tiên phân xử. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**
- **`--config` vô hiệu hoá mọi bất biến trong `config_invariants.rs`, và CI đang dùng nó ba lần** — `config_invariants.rs:166-190` chốt `devCsp` và cấm `tauri.<platform>.conf.json`, nhưng không có gì chặn `--config <file>` hay biến `TAURI_CONFIG`. Hai lớp phủ hôm nay (`tauri.nofonts.conf.json`, `bootstrapper.conf.json`) vô hại và đều có cổng riêng, nhưng lối đó mở toang: một lớp phủ tương lai đặt `app.security.csp` hay nới `assetProtocol.scope` cho bản build THẬT sẽ không làm test nào đỏ. Kèm theo, danh sách chặn ở `:178` chỉ liệt kê biến thể `.json` — Tauri còn nhận `tauri.macos.conf.json5` và `Tauri.macos.toml`. **(Chủ: một story hạ tầng cổng kế tiếp.)**
- **Cổng phụ thuộc dùng DANH SÁCH CẤM, trong khi chính repo lập luận danh sách cấm là sai** — `check-deps.mjs:121-142` vs `config_invariants.rs:92-94` (*"Danh sách CHO PHÉP, không phải danh sách CẤM. Một danh sách cấm chỉ chặn được những hình dạng ai đó đã nghĩ ra"*). `BANNED_CRATES`/`BANNED_NPM` thiếu `tauri-plugin-shell`, `tauri-plugin-http`, `tauri-plugin-process`, `tauri-plugin-opener`, `@tauri-apps/plugin-http`, `@tauri-apps/plugin-shell`. `plugin-http` phá cả AD-1/AD-29 lẫn AD-15; `plugin-shell` nguy hiểm hơn hẳn `plugin-dialog` đang bị cấm. Hai phương pháp trái ngược nhau trong cùng một lượt giao hàng. **(Chủ: một story hạ tầng cổng kế tiếp.)**
- **`walk()` đệ quy trong cổng phụ thuộc không có bộ nhớ đã-thăm** — `check-deps.mjs:95-99` duyệt `node.dependencies` đệ quy, thêm tên vào `npmNames` nhưng không dùng nó để chặn lặp. Với cây `npm ls --all --json` sâu/lặp (peer-dep, workspace) đây là công đệ quy mũ và có thể tràn stack — khi tràn, `abort()` in ra *"không đọc được cây npm"*, tức một lỗi công cụ đội lốt lỗi hạ tầng. Không tới hạn hôm nay (59 gói). **(Chủ: một story hạ tầng cổng kế tiếp.)**
- **Hai chỗ tài liệu nội bộ đã lệch khỏi sự thật** — (a) `deferred-work.md:7` vẫn ghi *"CI của Story 1.3 chỉ `cargo test` và build, không xác minh font nạp được lúc chạy"*; mệnh đề này **đã sai** kể từ khi `check:scope:bundled` (chiều dương qua `font-src`) vào `ci.yml:124`. (b) §File List của story 1.3 khai không *"Không đụng `_bmad-output/planning-artifacts/**`"* nhưng dải commit của story có sửa `epics.md` (+14/−) và `prd.md` (+6/−) — nội dung sửa đúng với quyết định của Ice, chỉ là dòng không và bảng "Sửa" nói sai sự thật, nên một lượt rà soát sau sẽ không biết tầng PRD đã đổi. **(Chủ: một story kế tiếp rà soát tài liệu quy hoạch.)**
- **Không có clippy · rustfmt · ESLint · Prettier · test runner frontend · quét CVE** — `ci.yml:88-130` chạy `cargo test` nhưng không `cargo clippy -- -D warnings`, không `cargo fmt --check`; không có `.eslintrc*`, `.prettierrc*`, `vitest.config.*`, `rustfmt.toml`, `clippy.toml`, `dependabot.yml`. Nặng hơn: `scripts/*.mjs` — **chính tầng cưỡng chế** — không được type-check (`tsconfig.json` chỉ include `src/**` + `env.d.ts`) và không có một test nào. Kèm theo: mọi crate ghim `=` vĩnh viễn **cộng** lệnh cấm tường minh `cargo-deny`/`cargo-audit` (`ci.yml:95-97`) ⇒ không có đường nào để biết một CVE xuất hiện trong cây. **(Chủ: một story hạ tầng CI kế tiếp — lưu ý ESLint đã có qua `check:lint`, chưa kiểm phần còn lại.)**
- **`dict-manifest.toml` đặt ra một luật ba trường bắt buộc rồi không cưỡng chế bằng gì cả** — `dict-manifest.toml:9-18` viết *"Mỗi mục PHẢI có đủ ba trường"* và cảnh báo checksum sai *"hỏng im lặng đúng kiểu tệ nhất"*, nhưng tệp giao ở trạng thái comment toàn bộ và không parser/test nào đọc nó. Bất đối xứng ngược chiều với mức rủi ro: repo dựng cả một script Node + mã thoát để canh việc ai lỡ cài `tauri-plugin-fs`, còn tệp sắp mang SHA-256 của ~130 MB dữ liệu tải về thì không có cổng nào. Chủ sở hữu: Story 1.9 / 10.1.
  → ✅ **ĐÃ ĐÓNG 2026-08-04 (Story 1.9, Task 8/13).** `scripts/check-dict-manifest.mjs` — parser TOML tập con tự viết, đọc + phán quyết `[base]`/`[[detachable]]`, gắn vào `ci.yml` job `check`. `[base]` đã điền THẬT (`sha256` của `dict-core.db` 154.836.992 byte dựng ở Task 11, `source_version` ghép năm nguồn). Nghiệm thu đỏ-rồi-xanh 11 ca ghi ở Debug Log References của Story 1.9.
- **Trích dẫn dòng trong comment cưỡng chế đã rữa trước cả khi commit** — `check-scope-bundled.mjs:20` trích `Cargo.toml:56-61` cho khối `[profile.release]`, nhưng khối đó thật sự ở **`Cargo.toml:61-66`** (lệch 5 dòng). Cùng tệp `:7`, `:20` trỏ `deferred-work.md:13`/`:5` như thể chúng ở gốc repo; đường thật là `_bmad-output/implementation-artifacts/deferred-work.md`. Các comment này là cơ chế truyền tri thức duy nhất giữa chín epic, mà trích dẫn dòng cứng vào tệp còn đang sửa sẽ rữa nhanh hơn tốc độ ai đó đọc lại. **(Chủ: một story hạ tầng cổng kế tiếp.)**
- **Nhánh nền tảng chỉ có `win32` / không-`win32`; và không bước build nào khớp một OS thứ ba** — `check-scope-bundled.mjs:60-62,80-82`: trên Linux `IS_WIN === false` ⇒ build với `--bundles app` (target chỉ hợp lệ cho macOS/iOS) và `binPath` trỏ vào `bundle/macos/…app/Contents/MacOS/`; cùng nhánh ẩn đó ở `scopeCheck.ts:93-95`. Song song: `ci.yml:138` và `:204` gác bằng `if: runner.os == 'macOS'` / `== 'Windows'` không có nhánh mặc định ⇒ thêm `ubuntu-*` vào matrix cho một job **xanh mà không dựng ứng dụng và không ghi phép đo AC6 nào**. Khối *"CHỖ MÓC CHO EPIC SAU"* mời gọi đúng việc mở rộng matrix này. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**
- **Action ghim bằng tag major trôi trong khi header khẳng định đã kiểm chứng phiên bản chính xác** — `ci.yml:15-17` ghi *"kiểm chứng qua GitHub API ngày 2026-08-03: `actions/checkout` v7.0.1 · `actions/setup-node` v7.0.0 · `Swatinem/rust-cache` v2.9.1"*, nhưng `:62,64,75` dùng `@v7`, `@v7`, `@v2`. Chính tệp này cấm `-latest` cho ảnh runner với lý do *"ảnh runner đổi dưới chân là một hồi quy giả"*; action còn nguy hơn ảnh runner vì nó **thực thi mã** trong job. Ghim theo SHA là hình dạng khớp với lời văn đang có. **(Chủ: một story hạ tầng CI kế tiếp.)**
- **`rust-version = "1.85"` là số trang trí** — `Cargo.toml:7` vs `ci.yml:70` (`dtolnay/rust-toolchain@1.97.1`). CI chỉ chạy một toolchain, cách MSRV khai báo 12 phiên bản. Một crate phụ thuộc nâng sàn thật, hay một cú pháp chỉ có từ 1.9x lọt vào, đều không làm gì đỏ. **(Chủ: một story hạ tầng CI kế tiếp.)**
- **`vite.config.ts` không nối `build.target`/`minify`/`sourcemap` với `TAURI_ENV_*`** — `vite.config.ts:9-19`. Hệ quả cụ thể: (a) esbuild dùng target `modules` mặc định, có thể phát cú pháp mà WebView2/WKWebView ở nền tảng tối thiểu chưa hỗ trợ, và không phép kiểm nào trong lượt này phát hiện được; (b) bản `tauri build --debug` mà `check-scope-bundled.mjs` chẩn đoán bằng `String(err)` được build **không sourcemap**, nên mọi lỗi frontend trong CI là stack đã minify. Comment `:5-8` liệt kê *"bốn thiết lập Tauri bắt buộc"* nhưng bỏ đúng nhóm này. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**
- **Gộp `stdout` + `stderr` vào cùng một chuỗi `log` có thể phá anchor `^VERDICT: …$`** — `check-scope-bundled.mjs:112-123` và `check-scope.mjs:52-60` nối hai `capture()` vào cùng biến `log` không phân tách theo dòng. Một chunk stderr (log WebView2, warning của Tauri) tới xen giữa lúc dòng `VERDICT: PASS` đang được ghi ⇒ regex không khớp ⇒ rơi vào nhánh *"Self-check chưa chạy tới nơi"* và exit 1 trong khi self-check đã PASS. Không tái lập được, sẽ bị đọc thành flaky rồi bị bỏ qua. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**

## Deferred from: 1-4-bo-token-mau-va-chu-hai-theme-co-kiem-tuong-phan-tu-dong (2026-08-03)

- ✅ **ICE ĐÃ PHÊ CHUẨN cả ba 2026-08-03** (lượt rà soát mã của story). ⚠️ **Còn mở: `DESIGN.md` chưa được sửa cho khớp.** Tới lúc đó, sổ `deviations` trong `src/tokens/tokens.json` là chỗ giữ sự thật, và `check-tokens.mjs` cưỡng chế rằng không có chỗ lệch nào khác **cộng thêm** rằng mỗi mục phải có `question` và `reason` không rỗng. Việc chỉnh `DESIGN.md` là **một lượt riêng của Ice** — giữ tiền lệ quyết định #3 ở Story 1.3, dev không tự sửa tài liệu quy hoạch. Ai làm lượt đó nhớ gỡ ba mục `deviations` trong cùng một commit, nếu không Kiểm A sẽ đỏ vì "deviation khai nhưng không khớp chỗ lệch nào".

- 🔴 **BA giá trị trong bảng token đang lệch khỏi `DESIGN.md`.**
  - `colors.dark.surface-accent` `#2c3a3b` → **`#283637`** *(§Câu hỏi cho Ice #1, đi theo mặc định của story là phương án A)*. Cặp `on-surface-variant` × `surface-accent` ở theme tối là **4,245:1 — trượt AA**; sau khi đổi là **4,505**. `error` trên cùng nền từ 4,519 lên 4,795.
  - `typography.lookup-gloss.lineHeight` `1.6` → **`1.66`** — **PHÁT HIỆN MỚI của lượt cài đặt**, chưa có trong §Câu hỏi cho Ice của story.
  - `typography.lookup-example.lineHeight` `1.6` → **`1.66`** — cùng lý do.
  - *Vì sao hai mục sau là phát hiện mới:* story bắt được `DESIGN.md` tự mâu thuẫn ở `read-title`/`lookup-headword` (họ `read` mà ở 1.3) và giải bằng cờ `wraps`. Nhưng `lookup-gloss` ("Nghĩa") và `lookup-example` ("Ví dụ và trích dẫn") ở **1.6** thì cờ `wraps` **không** giải được — chúng thật sự chạy thành đoạn, nên sàn 1.66 áp cho chúng. Đường thay thế duy nhất là khai `wraps: false`, tức nói dối cổng để cho xanh — đúng thứ AD-34 tồn tại để chặn. Chi phí thị giác: 0,87px và 0,75px mỗi dòng. **(Chủ: Sally — bmad-ux.)**

- **AC6 nghiệm thu ở TẦNG TOKEN, không phải trên màn hình** — không panel nào tồn tại hôm nay (Story 1.14 mới dựng). Kiểm G chứng minh hai theme khai hai cơ chế khác nhau và chặn việc chúng bị thống nhất; nó **không** chứng minh khe 2px hiện ra đúng. Dùng lại tiền lệ `unmeasured` của Story 1.3. Thuộc **Story 1.14**.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, AC11.4) — đo TRÊN MÀN HÌNH, không suy luận.** Engine **Blink/Chromium (Playwright headless)**, nền tảng **macOS 24.6 arm64**, cửa sổ 1400×900, bốn panel thật:
  - **theme sáng** — `.dv-groupview` `border-top-width: 1px`, `border-top-left-radius: 0px`; `.dv-view:has(> .dv-groupview)` `padding-top: 0px`; vỏ `.dv-dockview` nền `rgb(244, 241, 234)` (`--color-background`).
  - **theme tối** — `border-top-width: 0px`, `border-top-left-radius: 3px`, `padding-top: **1px**`.
  - **khe THẬT đo giữa hai panel cạnh nhau ở theme tối: 2px**. Nửa khe mỗi bên là số học có chủ ý: đệm cả khe mỗi bên cho ra 4px — gấp đôi `panelSeparator.dark.gap` — và **Kiểm G không bắt được** vì nó đọc `tokens.json` chứ không đọc CSS.
  ⚠️ **WKWebView CHƯA ĐO cho vế thị giác này**; lượt `npm run tauri dev` của story chỉ nghiệm thu vòng lưu/khôi phục bố cục (AC4). Ca **Windows** chưa đo. Bàn giao **Story 1.3 / 10.9**.

- **Kiểm E không phát hiện được một cờ `wraps` khai sai** — đã nghiệm thu tường minh: đặt `read-lg.wraps = false` thì cổng **vẫn xanh** (ca 23/28 của Task 3, kỳ vọng exit 0). Cờ `wraps` là một mệnh đề về *nội dung sẽ chạy qua token*, và không phép kiểm tĩnh nào phân xử được nó khi chưa có component. Lưới duy nhất hôm nay: cổng bắt buộc **phải có** cờ (thiếu là FAIL), nên một token mới không lặng lẽ trốn được sàn. Lưới thật là lượt rà soát khi Story 1.14/1.17 dựng panel — **đối chiếu lại từng cờ với chuỗi thật chạy qua nó**.
  → 🟡 **SOÁT MỘT PHẦN 2026-08-06 (Story 1.14, AC11 ⚠️(b)) — và nó tìm ra MỘT CHỖ LỆCH THẬT.** Bốn chuỗi đầu tiên chạy qua `ui-md` là bốn câu trạng thái panel, và **một trong bốn XUỐNG DÒNG THẬT**: `panel.ai_translation.status` dài 96 ký tự, chạy hai dòng ở panel rộng 700px. `ui-md` khai `wraps: false` với giãn dòng **1.5** — **dưới sàn 1.66**.
  🔴 **Ghi ra thay vì lặng lẽ sửa cờ.** `DESIGN.md` khai `ui-md` là *"Tiêu đề panel — nhãn một dòng"*; đổi `wraps` thành `true` sẽ đòi nâng giãn dòng lên 1.66 cho **mọi** nhãn giao diện — một quyết định thị giác toàn ứng dụng, không phải một lần sửa JSON, và nó chạm `DESIGN.md`. Ba đường: *(a)* rút ngắn câu trạng thái về một dòng; *(b)* thêm token `ui-md-wrap` ở 1.66 cho câu trạng thái; *(c)* nâng `ui-md` lên 1.66. **CHƯA CHỐT — quyết định của Ice.** Nhặt lại ở **Story 1.16/1.17**, nơi bề mặt đọc thật buộc phải mở lại chính bảng này.
  → ✅ **ĐÓNG MỘT PHẦN 2026-08-06 (Story 1.17, Quyết định #7).** Ice chốt đường **(b)** — token thứ 17 `ui-md-wrap` (họ `ui`, 12px/1.66, `wraps: true`) vào `tokens.json` + sổ `deviations` + `EXPECTED_COUNTS.typography` (16→17) của `check-tokens.mjs`. Áp cho **ba** chỗ dùng cũ nêu ở mục `:129` (`.load-error`/`.parallel-note` của `SourcePanel.vue`, `.hv-notice` của `SourceHanViet.vue`) **và** mọi câu trạng thái mới của Panel Lookup. `.parallel-note` đổi cỡ 11,5px→12px (chấp nhận, ghi ra ở `tokens.json` deviations). ⚠️ **`PanelFrame.vue .status`** (câu trạng thái MẶC ĐỊNH dùng chung bởi mọi panel — kể cả `panel.ai_translation.status` dài 96 ký tự xuống dòng thật đã nêu ở mục này) **VẪN ở `ui-md`, chưa đổi** — ngoài phạm vi story 1.17 (component đó không phải nội dung Panel Lookup); nhặt lại khi Epic 4 (AI Translation) dựng nội dung thật.

- **`scripts/check-tokens.mjs` không được type-check và không có test** — cùng hạng với mục *"không có clippy · rustfmt · ESLint · test runner frontend"* ở trên: `tsconfig.json` chỉ include `src/**` + `env.d.ts`, nên **cả tầng cưỡng chế** nằm ngoài mọi phép kiểm tĩnh. Bù lại một phần bằng nghiệm thu đỏ-rồi-xanh 28 ca (Task 3) — nhưng đó là test của *hành vi cổng*, chạy tay, không nằm trong CI. Một hồi quy trong chính script sẽ đi qua CI mà không ai biết. Thuộc lượt bổ sung công cụ frontend. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- **Bộ phân tích CSS của cổng là "đủ dùng", không phải một parser CSS thật** — `parseCssBlocks` bám dấu `{}` `;` trên văn bản đã che comment/chuỗi. Nó đúng cho CSS mà dự án đang viết, nhưng chưa xử `@supports` lồng sâu, CSS nesting của Vue SFC ở dạng lạ, hay `url()` chứa dấu `;`. Khi Story 1.14 dựng CSS thật và nhiều, **soát lại số khai báo mà cổng báo đã quét** — con số đó tụt xuống bất thường là dấu hiệu parser bỏ sót cả vùng, và một cổng bỏ sót im lặng thì xanh y hệt một cổng đang canh.
  → ✅ **ĐÃ SOÁT 2026-08-06 (Story 1.14, AC11 ⚠️(a)) — con số đi ĐÚNG CHIỀU.** Trước: *21 tệp (18 component) · 116 khai báo*. Sau: *32 tệp (29 component) · **195** khai báo*. **+79** khai báo, phần lớn từ `src/layout/dockview-theme.css`. Không dấu hiệu bỏ sót vùng: mọi khai báo của tệp đó **đi qua Kiểm B** *(chúng phải là `var(--color-*)`; một hex viết thẳng ở đó sẽ đỏ)*. ⚠️ Cây vẫn chưa có `@supports` lồng sâu hay `url()` chứa `;`, nên hai lỗ đó của parser vẫn **chưa được thử**.

- ⚠️ **BA MỆNH ĐỀ THỊ GIÁC của Task 4/5 đang đứng bằng VĂN XUÔI, không bằng bằng chứng tái lập được** *(Ice chấp nhận 2026-08-03 với điều kiện ghi ra đây)*. Trang thăm dò, bốn ảnh chụp và bộ đọc `fvar` sống ngoài repo có chủ ý (tiền lệ §Ranh giới phạm vi của mũi thăm dò Story 1.1: tài nguyên dùng một lần không vào cây nguồn). Hệ quả là không lượt rà soát nào sau này tái lập lại được ba mệnh đề sau từ cây nguồn:
  - *"Bốn nét `Source Sans 3` (200/400/600/700) phân biệt rõ trên chuỗi dày dấu tiếng Việt"* — dựng trên **Blink/macOS**, chưa đo trên WKWebView, chưa đo trên Windows.
  - *"`ui-label` (700) là nét THẬT, không phải nét đậm tổng hợp"* — cùng giới hạn engine.
  - *"Dưới `font-synthesis: none`, chữ Hán đứng thẳng trong khi phần Latin vẫn nghiêng thật"* — cùng giới hạn engine, và lời giải này **chưa có người tiêu thụ** (xem mục ngay dưới).
  - Số đo `fvar`/`name` thì **đọc lại được từ chính tệp font** nên đứng vững hơn hẳn ba mệnh đề trên; chúng đã được chép vào `src/tokens/README.md`.
  - **Nhặt lại ở đâu:** lượt đo NFR của **Story 1.9** và nghiệm thu cuối của **Story 10.9**, nơi đã có sẵn một lượt runner hai nền tảng để bấu vào. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**

- ⚠️ **`body` chạy ở giãn dòng 1.5 (`ui-md`, `wraps: false`) và không phép kiểm nào canh được chỗ đó** — `src/tokens/reset.css` đặt `--face-ui-md`/`--font-ui-md`/`--leading-ui-md` làm mặc định toàn ứng dụng. Giả định đằng sau: *mọi thứ chạy qua `body` mà chưa khai token riêng đều là nhãn giao diện*. Giả định đó đúng hôm nay và **không có gì cưỡng chế nó** — Kiểm E chỉ đọc `tokens.json`, hoàn toàn mù với việc component nào đang kế thừa gì. Một bề mặt đọc quên khai token `read-*` của chính nó sẽ chạy chữ chạy thành đoạn ở 1.5, đúng mức mà `DESIGN.md §Giãn dòng` nói làm dấu `ườ` chạm dấu `ộ`, và cổng vẫn xanh. **Lưới thật là lượt rà soát khi Story 1.14/1.16/1.17 dựng panel** — đối chiếu từng bề mặt chữ với token nó khai, cùng lượt với việc đối chiếu lại cờ `wraps` (mục ngay trên).
  → 🟡 **SOÁT MỘT PHẦN 2026-08-06 (Story 1.14, AC11 ⚠️(c)).** Ba bề mặt chữ mới của story **đều tự khai token**, không cái nào kế thừa `ui-md` của `body`: `PanelTab.vue .tab` khai `--face/--font/--leading/--weight-ui-md` *(và `ui-md-strong` khi có tiêu điểm)*; `PanelFrame.vue .status` khai `--face/--font/--leading-ui-md`. Cả ba là nhãn giao diện, nên `ui-md` đúng vai.
  ⚠️ **Vế đầy đủ VẪN MỞ:** bề mặt ĐỌC *(nguyên văn tiếng Trung, âm Hán Việt, bản ghi từ điển, Editor)* chưa tồn tại — thân panel còn trống. Chúng phải khai `read-*` / `source-*` / `lookup-*` của chính chúng: **Story 1.16 / 1.17 / Epic 2**. Doc-comment đầu `PanelFrame.vue` ghi đúng cảnh báo đó tại chỗ.
  → ✅ **ĐÓNG NỬA LOOKUP 2026-08-06 (Story 1.17, AC9).** Bản ghi từ điển (`LookupPanel.vue`/`LookupRecord.vue`) tự khai `lookup-headword`/`lookup-gloss`/`lookup-example`/`ui-label` — đối chiếu từng bề mặt chữ với token nó khai, không một class nào kế thừa `ui-md` của `body`. Nửa **Editor** (Epic 2) vẫn mở.
  → ✅ **ĐÓNG NỬA EDITOR 2026-08-12 (Story 2.2 · AC6 · Task 3.2).** Bề mặt đọc của Panel Editor (`.doc.tok-editor` trong `EditorPanel.vue`) tự khai token `editor` của chính nó — `var(--face-editor)` · `var(--font-editor)` · `var(--leading-editor)` · `var(--color-on-surface)` — họ `read`, 15px, giãn dòng **1.95**, tức trên hẳn sàn cứng 1.66 mà mặc định `ui-md` của `body` (1.5) vi phạm. Ba bề mặt chữ khác của story cũng tự khai: `.load-error` và `.untranslated-note` đều dùng `ui-md-wrap` (1.66), cùng token và cùng lý do với `SourcePanel.vue::.load-error`. ⚠️ Mục gốc ở trên vẫn đúng ở vế **cơ chế**: Kiểm E vẫn chỉ đọc `tokens.json` và vẫn mù với việc component nào kế thừa gì — lưới duy nhất vẫn là một lượt rà soát khi một bề mặt chữ mới ra đời. Bề mặt đọc còn lại chưa dựng: **Panel AI Translation (Epic 4)**.

- **`--synthesis-*` và `--tracking-*` chưa có người tiêu thụ** — `applyTheme` phát đủ bảy biến cho cả 14 token, nhưng hôm nay chỉ `App.vue` dùng ba biến của `ui-mono`. Lời giải chữ Hán nghiêng giả (`fontSynthesis: 'none'` ở `source-hanviet` và `lookup-example`) đã được **dựng thật và chụp lại** trên trang thăm dò, nhưng nó chỉ có hiệu lực trong sản phẩm khi Story 1.16/1.17 áp `font-synthesis: var(--synthesis-<token>)` ở chính chỗ dựng hai token đó. **Bỏ sót dòng đó là cách lời giải này chết im lặng.**
  → ✅ **ĐÓNG 2026-08-06 (Story 1.17, AC9).** `lookup-example` (Panel Lookup: từ loại, ví dụ, trích dẫn, ghi chú) là **người tiêu thụ thứ hai** của `font-synthesis`, sau `source-hanviet` (Story 1.16). `LookupRecord.vue` khai `font-synthesis: var(--synthesis-lookup-example)` ở mọi lớp dùng token đó.

## Deferred from: 1-6-commandregistry-ba-che-do-va-tieu-diem-ban-phim (2026-08-04)

- 🔴 **Vế DOM của AC4 KHÔNG có phép kiểm tự động nào** — *"focus không bao giờ rơi về `body`"* là hành vi lúc chạy trong một webview, và dự án **không có bộ chạy test frontend** (và không được thêm — NFR15). Hai thứ đang canh nó, cả hai đều có giới hạn ghi thẳng:
  - **Chốt tự kêu** ở `src/commands/focus.ts` — `console.error` ở frame kế tiếp nếu `document.activeElement` là `body`. Nó chỉ **kêu**, không vá (một vòng focus tự phục hồi sẽ đánh nhau với người dùng đang Tab và với hộp thoại của OS). Không ai đọc log thì không ai biết.
  - **Nghiệm thu tay** 2026-08-04 — bảng ở §Debug Log References của story. Chạy tay nghĩa là **không nằm trong CI**: một hồi quy đi qua CI mà không ai biết.
  - Lưới thật là lượt rà soát khi **Story 1.14** dựng bốn panel trong `dockview`, nơi thứ tự focus thật sự phức tạp. Cùng hạng với mục *"`check-tokens.mjs` không được type-check và không có test"*. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- 🔴 **Nghiệm thu DOM chạy trên Blink (Chrome), KHÔNG phải WKWebView, và KHÔNG qua `tauri dev`.** Lý do đo được, không phải quên: cổng `1420` mà `vite.config.ts` ghim (`strictPort: true`) đang bị **một dự án khác của Ice** (`gdrive_suite_manager`) chiếm lúc đo, và `devUrl` trong `tauri.conf.json` trỏ cứng vào đó — mà §Ranh giới phạm vi của story không cấm đụng `tauri.conf.json`. Lượt đo chạy qua `npx vite --port 1431` rồi lái bằng Chrome. **Chưa đo:** `⌘1 ⌘2 ⌘3` đi qua **WKWebView** thật; `⌘1..3` đi qua **tầng OS** *(Chrome nuốt `⌘2` để chuyển tab — sự kiện được dựng trên `window` thay thế, tức tầng ứng dụng đã đo, tầng phân phối phím của OS thì chưa)*. Đừng viết "tương đương" bằng suy luận. **Nhặt lại:** một lượt `npm run tauri dev` khi cổng 1420 rảnh, hoặc lượt runner của Story 1.9 / 10.9. **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

- 🔴 **Ca Windows CHƯA ĐO** — không có máy Windows. Kiểm D của `npm run check:commands` chứng minh **tầng phân giải hợp âm** đúng ở cả hai nhánh `Mod → ⌘ | Ctrl` (nền tảng là một tham số tiêm vào, nên phép kiểm chạy được trên một nền tảng), nhưng nó **không** chứng minh `Ctrl+1` tới được webview trên Windows. Đúng tiền lệ bàn giao phép đo của Story 1.1 → 1.3. Thuộc **Story 1.3 / 10.9**, nơi đã có sẵn một lượt runner hai nền tảng để bấu vào. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**

- ⚠️ **`focus.next_panel` chưa có phím, nên hôm nay KHÔNG có đường bàn phím nào vào panel** — cố ý (§Quyết định thiết kế #5 của story: bốn panel chưa tồn tại; mọi phím ứng cử đều đang hoặc sắp có chủ; và AC6 cần một phần tử thật để `unbound()` có nhánh chạy). Hệ quả phải nói ra: trạng thái tiêu điểm của AC5 hôm nay **chỉ đến được bằng chuột**, và đó là một lỗ trong NFR17 cho tới khi phím được gán. Nhận ở **Story 1.14** *(thứ tự vòng xoay khi có `dockview`)* và **Story 1.21** *(màn hình gán phím)*.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, §Quyết định #2).** `focus.next_panel` = `Mod+Alt+→`, `focus.prev_panel` = `Mod+Alt+←`. Không đụng `Tab`, không đụng `⌥←` `⌥→` trần *(Chương trước/sau — `EXPERIENCE.md:148`, Story 2.11)*, không đụng `⌘⇧…` (không gian của UX-DR35).
  → 🔵 **ĐỊNH CHÍNH 2026-08-18 (Story 2.11) — vế *"chỗ đã đặt trước cho `⌥←`/`⌥→`"* ở dòng trên đứng trên MỘT LƯỢT ĐỌC NHẦM.** *(Định chính, **không** xoá: quyết định `Mod+Alt+→`/`Mod+Alt+←` cho `focus.*_panel` vẫn đúng nguyên vẹn, chỉ **lý do phụ** dẫn kèm là sai.)* Đo lại từ nguồn hôm nay: dòng **148** của `EXPERIENCE.md` nay là đoạn **Auto-Lookup** — số dòng đã trôi; hàng thật `| ⌥← ⌥→ | Chương trước / sau trong cùng lần nhập |` nằm ở **`EXPERIENCE.md:184`** và nó thuộc bảng *"**Sửa ranh giới bóc** — bàn phím là đường chính"* (`:174-186`), tức **màn xem trước NHẬP**, xác nhận bằng `epics.md:599` = **UX-DR33**. Bảng Phím của **Workspace** (`EXPERIENCE.md:261-269`) **không một hàng nào** cho chuyển Chương. ⇒ ~~`⌥←`/`⌥→` là chỗ đã đặt trước cho Story 2.11~~ — nó **chưa bao giờ** được đặt chỗ ở Workspace. Và độc lập với chuyện đặt chỗ, cặp phím ấy **không dùng được** cho FR26: `keys.ts:510` nuốt mọi hợp âm `lacksPrimaryMod` khi caret đang trong vùng gõ, tức đúng ca thường nhất *(người dùng vừa gõ xong câu cuối)* — cùng phép đo đã lật một chữ ký ở Story 2.10 (`⌥↓` → `⌘⌥↓`). Story 2.11 chốt **`Mod+Alt+]`** / **`Mod+Alt+[`** (Ice ký Quyết định #6, 2026-08-18).
  🔴 **Bài học, và nó lớn hơn cặp phím:** một mục sổ nợ dẫn `tệp:dòng` sẽ **trôi** cùng tệp, và ở đây nó trôi từ một bảng này sang một bảng khác **của một màn hình khác** mà vẫn đọc trơn tru. Dẫn kèm **tên bảng** hoặc **nguyên văn hàng**, đừng chỉ số dòng.

- ⚠️ **Xung đột `⌘1` `⌘2` giữa mockup và UX-DR34 — đã phân xử, nhưng mockup CHƯA sửa.** `mockups/key-screen-workspace.html:89` vẽ `Bố cục 2×2 nguồn–đích ⌘1 · 4 cột ⌘2` là **preset bố cục**, trong khi AC3 của Story 1.6, UX-DR34 và `EXPERIENCE.md:49` đều nói `⌘1 ⌘2 ⌘3` là **ba chế độ**. Phân xử: **chế độ thắng** — AC của epic là hợp đồng nghiệm thu còn mockup là bản phác; UX-DR34 là một mục đánh số còn dòng trong mockup thì không; và ba chế độ là cấu trúc toàn ứng dụng (AD-24) còn preset bố cục chỉ sống trong Workspace. **Việc còn lại: Story 1.14 phải chọn phím KHÁC cho preset bố cục (FR18).** Dev không sửa mockup — giữ tiền lệ quyết định #3 của Ice ở Story 1.3; lượt chỉnh tài liệu quy hoạch là một lượt riêng của Ice.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, §Quyết định #1).** Preset bố cục nhận `Mod+Alt+1` *(lưới 2×2)* và `Mod+Alt+2` *(bốn cột)*. Giữ nguyên "số thứ tự preset" mà mockup dạy, chỉ thêm một phím bổ trợ; `Mod+Alt+3` **để trống** cho **Review Mode** ở Story 8.11 — đúng thứ tự mockup. Khớp bằng `event.code` (`Digit1`) nên `⌥1` sinh ký tự `¡` trên macOS không thành vấn đề. Đo được: `⌘1` **vẫn** chuyển sang Library, không đổi preset.
  ⚠️ **`mockups/key-screen-workspace.html:89` VẪN chưa sửa** và dev vẫn không sửa nó — sửa mockup là **một lượt riêng của Ice**.

- ⚠️ **Bộ token thiếu một biến trọng lượng cho nhãn giao diện ĐẬM** — AC5 và UX-DR8 đòi tiêu đề panel *"`primary` in đậm"*, `DESIGN.md §Components` và `mockups/key-screen-workspace.html:34` ghi **600**. Nhưng `ui-md` khai `400` và `ui-label` khai `700`; viết thẳng `600` thì Kiểm B2 của `check-tokens.mjs` đỏ (đúng), và không khai một biến CSS cục bộ `--weight-…: 600` để lách cổng là đúng thứ AD-34 tồn tại để chặn. Đang **mượn `var(--weight-read-title)`** ở `src/panels/PanelFrame.vue` và `src/App.vue`, kèm comment nêu lý do ở cả hai chỗ. Hai đường ra cho **Story 1.14**: thêm một token typography cho nhãn đậm *(phải qua Kiểm A và Kiểm C của `check-tokens.mjs`)*, hoặc chốt rằng mượn là đúng và ghi vào `DESIGN.md`. ⚠️ Đường thứ hai vẫn còn nợ: `--weight-read-title` đổi giá trị thì hai chỗ này đổi theo mà không ai biết.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, AC10 — đường A).** Token typography thứ **15**: `ui-md-strong` *(họ `ui` · 12px · **600** · 1.5 · `wraps: false`)*. `PanelTab.vue` dùng `var(--weight-ui-md-strong)`; hai chỗ mượn cũ *(`PanelFrame` — nay là `PanelTab` sau §Quyết định #4A — và `App.vue:288`)* không còn mượn.
  🔴 **Đường B bị loại có lý do:** nó đòi một lượt sửa `DESIGN.md`, mà dev không sửa tài liệu quy hoạch *(tiền lệ quyết định #3 của Ice ở Story 1.3)* — nên nó để món nợ mở tiếp **và** để nguyên rủi ro đã ghi.
  ⚠️ **Cách chữ ký được cưỡng chế, và nó KHÔNG phải cách story đề xuất.** Story bảo thêm hàng vào `EXPECTED_TYPOGRAPHY` của cổng. Làm vậy là để **bản chép độc lập thứ hai của `DESIGN.md`** lặng lẽ trôi khỏi `DESIGN.md` — hai bản chép chỉ bắt được lỗi khi cả hai còn chép cùng một thứ. Nên: bảng đóng băng **ở lại đúng 14 hàng của `DESIGN.md`**, và `compare()` được mở rộng để coi **token THỪA** là một chỗ lệch phải có mục `deviations` với `question` + `reason` không rỗng. Nghiệm thu: gỡ mục deviation ⇒ `FAIL typography: thừa 1 token KHÔNG có chữ ký — ui-md-strong`.
  ⚠️ **`DESIGN.md` vẫn ghi 14 token** và việc sửa nó là **một lượt riêng của Ice** — mục này đóng phần *cưỡng chế*, không đóng phần *tài liệu*.

- ⚠️ **Không cổng nào canh focus ring** — một `*:focus { outline: none }` phá đúng nửa NFR17 (*"trạng thái focus luôn nhìn thấy rõ"*) mà **qua được cả `check-commands.mjs` lẫn `check-tokens.mjs`** (cổng token canh màu, cỡ chữ, tương phản, opacity, elevation — không canh focus ring). Luật đang do người viết giữ: `outline: none` **chỉ** ở gốc `tabindex="-1"` của chế độ và panel, kèm lý do ngay cạnh dòng CSS. Đóng được rẻ nhất ở **Story 1.14**, cùng lượt rà soát khi bốn panel thật có điều khiển tương tác thật.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, AC11.2).** `scripts/check-tokens.mjs` **Kiểm H**: `outline: none` / `outline: 0` / `outline-style: none` chỉ hợp lệ khi selector là **đúng một** gốc chế độ/panel ở dạng `<lớp>:focus` (`.mode` · `.panel` · `.dock`). Mọi bộ chọn hậu duệ hay anh em (`.panel *:focus`, `.mode > a:focus`) và mọi `:focus` trần đều đỏ; đường ra là miễn trừ **có tên** `/* aura-allow-outline-none: <lý do> */`, cùng khuôn `aura-allow-z-index` của Kiểm F. Nghiệm thu đỏ-rồi-xanh **13 ca** (9 đỏ — gồm `*:focus`, `:focus`, `button:focus`, `outline: 0`, `outline-style: none`, `!important`, và một selector NHÓM có một vế hợp lệ một vế không — 4 đối chứng âm). ⚠️ Giới hạn ghi thẳng: cổng đọc **selector**, không đọc HTML, nên nó không chứng minh được phần tử khớp selector thật sự mang `tabindex="-1"`; nó chứng minh được điều kiểm được — selector không quét rộng.

- ⚠️ **Kiểm A chỉ canh `@click`** — `@keydown`, `@input`, `@change`, `@submit` **không** thuộc luật *"phải là đúng một `dispatch('<id>')`"*. Có chủ ý: chúng không phải "thao tác" theo nghĩa AD-34 §1 (một `@input` là dòng dữ liệu). Nhưng ngày **Epic 2** dựng Editor với `@keydown` mang thao tác thật, luật phải được xem lại — không phải nới regex một cách lặng lẽ. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- ⚠️ **`scripts/check-commands.mjs` không được type-check và không có test tự động** — cùng hạng với ba mục đã ghi cho `check-deps.mjs` · `check-tokens.mjs` · `check-i18n.mjs`: `tsconfig.json` chỉ include `src/**` + `env.d.ts`, nên cả tầng cưỡng chế nằm ngoài mọi phép kiểm tĩnh. Bù lại một phần bằng nghiệm thu đỏ-rồi-xanh **28 ca** (Task 10) — nhưng đó là test của *hành vi cổng*, chạy tay, không nằm trong CI. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- ⚠️ **Sàn của cổng đếm TỆP, không đếm nội dung** — `VUE_FLOOR = 4` (thật: 5) và `TS_FLOOR = 10` (thật: 13) đóng được *"cây rỗng đọc thành sạch"* nhưng không đóng *"tệp rỗng đọc thành sạch"*. Cùng mục đã ghi cho `check-i18n.mjs:207-218`. Mở lại khi Story 1.14 dựng bốn panel. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- ~~⚠️ **Chế độ mặc định lúc khởi động là `library` và không phép kiểm nào canh**~~ — PRD §5.2 gọi Library là *"điểm vào ứng dụng"*, nên lựa chọn có lý do; nhưng lúc viết, cả ba chế độ đều rỗng nên nó không quan sát được ở đâu ngoài tab nào đang sáng.
  → ✅ **ĐÓNG 2026-08-04 (Story 1.8).** Chế độ cuối cùng nay được **lưu** (`watch(currentMode)` → `put_config('app_config', 'mode', …)`) và **nạp lại** lúc khởi động, và `scope_contract.rs::the_last_mode_survives_a_write_and_a_reopen` canh cả vòng ghi → đóng kho → mở lại → đọc, cộng mệnh đề *"kho rỗng ⇒ `library`"*. `setMode()` giữ nguyên chốt lúc chạy cho giá trị không hợp lệ — không và không cố ý **không** thêm một chốt thứ hai ở Rust: hai danh sách chép tay ở hai tầng là hai danh sách sẽ trôi khỏi nhau.

---

## Deferred from: code review of 1-6-commandregistry-ba-che-do-va-tieu-diem-ban-phim (2026-08-04)

Ba mục dưới đây là phát hiện **có thật** của lượt review ba lớp, được xếp hoãn vì đường dẫn tới chúng chưa tồn tại trong sản phẩm hôm nay. Không mục nào đã đóng.

- ⚠️ **`isTypingZone` mù với shadow DOM, và chặn nhầm input phi văn bản** — `src/commands/keys.ts:207-212`. Hai chiều hỏng ngược nhau: (1) `event.target` trên một listener gắn ở `window` bị **retarget về shadow host**, nên một custom element bọc `<input>` đọc ra `tagName: 'MY-EDITOR'` và `isContentEditable: false` ⇒ hàm trả `false` ⇒ hợp âm trần dispatch **trong khi người dùng đang gõ**; `composedPath()[0]` không bao giờ được hỏi. (2) `type="checkbox"` · `radio` · `button` · `range` và input `disabled`/`readonly` đều báo `tagName === 'INPUT'` ⇒ focus vào một checkbox làm **mọi hợp âm trần chết im lặng**, `handle()` trả `false` không một dòng chẩn đoán. Hoãn vì hôm nay sản phẩm chưa có shadow DOM lẫn input phi văn bản nào. **Nhặt lại ở Epic 2** *(Editor là vùng gõ tự do đầu tiên)* hoặc bất kỳ story nào dựng điều khiển form thật.
  → 🔁 **CHUYỂN CHỦ 2026-08-12 (Story 2.2 · Task 10.1) — từ *"Epic 2"* sang **Story 2.3** đích danh.** Món này **đi qua** Story 2.2 mà **KHÔNG đóng**, và nay lý do là dứt khoát chứ không phải một lượt hoãn nữa: Ice chốt Quyết định #1 đường (b) ngày 2026-08-12, nên bề mặt Editor của 2.2 là **chỉ-đọc** — không `contenteditable`, không `<textarea>`/`<input>`, và một cổng tĩnh (`check-commands.mjs` Kiểm J) cưỡng chế điều đó. Vùng gõ tự do đầu tiên của dự án vì thế sinh ra ở **Story 2.3**, cùng lượt với hợp đồng flush AD-35 — đó mới là lượt đầu tiên `isTypingZone` có một vùng gõ thật để trả lời đúng hay sai về nó. **Chủ: Story 2.3.**
  → ✅ **ĐÓNG MỘT NỬA 2026-08-12 (Story 2.3)** — chiều *thật sự chạm tới* đã đóng và **có lưới**
  (Kiểm D lái nhánh `isContentEditable === true`, nghiệm thu đỏ-rồi-xanh). Hai chiều **shadow DOM**
  và **input phi văn bản** vẫn hở, và vẫn **không chỗ gọi nào đi qua** — chủ chuyển sang story nào
  dựng điều khiển form thật hoặc một custom element. Số đo ở §Deferred from: 2-3-hop-dong-flush.

- ⚠️ **Chốt chống rơi `body` bắn-và-quên: `rAF` không chạy khi cửa sổ ẩn, và blur cho cáo buộc sai** — `src/commands/focus.ts:103-113`. Đây là **chuông báo tự động duy nhất** cho AC4, và nó có hai lỗ không canh gác: (1) `requestAnimationFrame` không chạy khi cửa sổ bị ẩn/thu nhỏ, nên chốt bị **bỏ qua đúng trên đường khởi động nền** — chỗ nó cần kêu nhất; (2) nếu người dùng bấm ra ngoài hoặc cửa sổ mất focus trong khoảng giữa `enter()` và callback, `document.activeElement` đọc ra `body` và chốt in một **cáo buộc sai** nêu đích danh một owner đã focus hoàn toàn đúng. Không có đường huỷ. Hoãn vì đây là chuông báo chứ không phải cơ chế — cả hai lỗ làm chuông kém tin, không làm focus hỏng. **Nhặt lại cùng lượt** dựng nghiệm thu DOM tự động *(cùng mục với "Nghiệm thu DOM chạy trên Blink" ở trên)*. **(Chủ: story kế tiếp chạm `src/commands/focus.ts`.)**

- 🔴 **AC4 của Story 1.6 ĐẠT MỘT PHẦN — vế panel chưa có đường dời focus tường minh nào chạy được** *(Ice chốt 2026-08-04 trong lượt code review)*. `src/panels/PanelFrame.vue:50-52` chỉ gọi `declareFocus`, không có `onActivated`/`enterFocus`. Đường duy nhất dời focus vào một panel là `focus.next()` qua command `focus.next_panel` — mà command đó **cố ý không gán phím** (§Quyết định thiết kế #5) **và cũng không có `@click` nào dispatch nó**: grep toàn `src/` cho đúng 3 lời gọi `dispatch()`, cả ba là `mode.*`. Hệ quả phải nói thẳng: handler của `focus.next_panel` là **mã sống nhưng bất khả đạt**, và vế *"mỗi chế độ và mỗi panel dời focus DOM tường minh tới điểm vào đã khai"* của AC4 hôm nay chỉ được thoả cho **chế độ** *(qua `onActivated` → `enterFocus`, đã đo)*; với **panel** nó chỉ được thoả bằng hành vi focus mặc định của trình duyệt khi bấm chuột vào một phần tử `tabindex="-1"` — **không** phải bằng `el.focus()` của ứng dụng. Đây là giới hạn của chính AC4, không chỉ của NFR17 *(mục "focus.next_panel chưa có phím" ở trên xếp nó dưới NFR17 — chưa đủ)*. **Lý do hoãn:** giữ §Quyết định #5 nguyên vẹn — gán phím hôm nay làm `unbound()` trả mảng rỗng và AC6 mất bằng chứng; còn cho `PanelFrame` tự `enterFocus` là thêm một hành vi focus tự động mà Story 1.14 có thể phải tháo ra khi `dockview` quyết thứ tự vòng xoay. **Nhặt lại ở Story 1.14** *(thứ tự vòng xoay panel)* và **Story 1.21** *(màn hình gán phím)*. Không đánh dấu AC4 đạt trọn cho tới lúc đó.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, AC9).** `focus.next_panel` **nay có phím** (`Mod+Alt+→`) và có cả `focus.prev_panel` (`Mod+Alt+←`), nên handler không còn là mã bất khả đạt. Vòng xoay đi theo **thứ tự bố cục thật** *(`visiblePanelsInLayoutOrder()` sắp theo `group.api.boundingBox`: trên→dưới rồi trái→phải)*, không theo thứ tự `declare()`; panel đã ẩn không có trong vòng. Và `onDidActivePanelChange` của dockview gọi `enterFocus(owner)` **tường minh** — nhưng CHỈ khi `origin === 'user'` *(xem mục mới bên dưới)*. Đo được trên Blink: bốn lần `Mod+Alt+→` đi hết bốn panel theo đúng thứ tự lưới rồi quay lại; ẩn một panel ⇒ vòng còn ba và panel đã ẩn không xuất hiện; rời Workspace rồi quay lại ⇒ không vạch tiêu điểm nào nói dối.
  ⚠️ **AC6 của Story 1.6 GIỮ ĐƯỢC bằng chứng:** `unbound()` nay trả về **bốn** `layout.toggle_*` thay vì `focus.next_panel`.

- ⚠️ **Bộ lọc phần mở rộng của cổng bỏ qua `.tsx` · `.mts` · `.cts`** — `scripts/check-commands.mjs:122,130-131`. `name.toLowerCase().endsWith('.ts')` sai với cả ba. Một tệp như vậy **không đóng góp gì** vào `tsFiles`, nên mọi `dispatch('…')` và `declareFocus('…')` trong đó vô hình với Kiểm B và Kiểm E, và nó cũng **không tính vào `TS_FLOOR`** — tức sàn không phát hiện được việc mất tệp. Hoãn vì dự án không dùng ba phần mở rộng đó và `tsconfig.json` không bật `jsx`. **Nhặt lại** nếu có story nào thêm `.tsx`, hoặc gộp vào lượt rà soát sàn cổng ở Story 1.14.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, AC11.3).** `walk(SRC_ROOT, ['.ts', '.tsx', '.mts', '.cts'])`. Nghiệm thu: một `src/layout/__probe.mts` và một `__probe.cts` mang `dispatch("khong.ton_tai")` **nay bị Kiểm B bắt** (trước lượt sửa: vô hình). ⚠️ `.d.ts` cố ý KHÔNG bị loại — một tệp khai báo không chở `dispatch()` nào nên nó chỉ làm quần thể to thêm, và một luật thừa là một chỗ để sai.

---

## Deferred from: 1-7-tang-ghi-du-lieu-mot-writer-noi-tiep-va-luoc-do-co-phien-ban (2026-08-04)

- 🔴 **SÁU con số `Tuning` là TẠM và CHƯA CÁI NÀO ĐƯỢC ĐO — chủ sở hữu là Story 2.4. **(Chủ: Story 2.4.)** `src-tauri/src/core/store/mod.rs`, `impl Default for Tuning`. Chúng không đo được ở story này vì phép đo cần **Editor thật**: `wal_threshold_bytes` (AD-12) và nhịp flush (AD-35) **đánh đổi lẫn nhau** — phải đạt NFR18 *(mất ≤ 5 s)* mà không phạm NFR2 *(không frame nào vượt 50 ms)*. `ARCHITECTURE-SPINE.md#Deferred` và `epics.md:454` đã xếp cả cặp vào **Giai đoạn 2**.

  | Tham số | Giá trị tạm | Lý lẽ *(lý lẽ, không phải phép đo)* |
  |---|---|---|
  | `pool_size` | 4 | Đủ để quan sát được đọc chồng nhau; nhỏ để TRUNCATE không phải chờ nhiều reader |
  | `busy_timeout` | 5 000 ms | Dài hơn một lượt checkpoint bình thường, ngắn hơn ngưỡng người dùng cho là treo |
  | `checkpoint_tick` | 1 s | Độ phân giải của cả hai điều kiện kích hoạt |
  | `idle_before_passive` | 5 s | **Cố ý dài hơn** nhịp flush 2 s của AD-35, để checkpoint không đánh nhau với đường gõ |
  | `wal_threshold_bytes` | 4 MiB | Bằng đúng ngưỡng autocheckpoint mặc định của SQLite *(1000 trang × 4096 B)* mà AC3 vừa tắt — lấy lại đúng số nó bỏ lại, tức không đổi hành vi theo một hướng chưa ai đo |
  | `close_truncate_budget` | 2 s | Trần để `close()` không làm `check:scope` / `check:scope:bundled` đỏ |

  **Đừng đọc chúng như đã hiệu chỉnh**, và đừng để một story sau tưởng chúng đã qua một lượt đo. Doc-comment của `Tuning` và của module đều khai là tạm. **Story 2.4** đo lại cả sáu trên Editor thật.

- ⚠️ **AC5 nghiệm thu CƠ CHẾ, không nghiệm thu NGƯỠNG THẬT.** `store_contract.rs::the_wal_stops_growing_once_it_crosses_the_threshold` chạy với `wal_threshold_bytes = 64 KiB` và blob 32 KiB — số thu nhỏ để ca chạy trong dưới một giây trên cả hai nền tảng *(§Testing standards cấm `sleep` dài)*. Nó chứng minh **vế (b) tồn tại và kích hoạt được khi chưa rảnh**; nó **không** chứng minh 4 MiB là con số đúng cho một phiên gõ thật. Cùng chủ sở hữu: **Story 2.4**. **(Chủ: Story 2.4.)**

- ⚠️ **Ca AC5 phụ thuộc vào nhịp tương đối giữa `checkpoint_tick` và khoảng cách hai lượt ghi.** SQLite chỉ quay `.db-wal` về đầu tệp khi một giao dịch ghi bắt đầu đúng lúc `nBackfill == mxFrame` (`walRestartLog`). Ca test vì thế đặt tick 3 ms / gap 10 ms và ghi lý do ngay tại chỗ. Trên một runner chậm hơn hẳn, tỷ lệ đó có thể lệch. Đã chạy 5 lượt liên tiếp trên máy dev không dao động; **chưa chạy trên runner CI lần nào** — cùng danh sách với bốn phép nghiệm thu của Story 1.3 đang chờ lượt CI thật. **(Chủ: một story kế tiếp chạm `core/store`.)**

- ~~⚠️ **Lỗi mở kho hôm nay chỉ ra `stderr`, không tới người dùng.**~~ `src-tauri/src/lib.rs::open_global_store` ghi chẩn đoán rồi **đi tiếp** thay vì chặn khởi động; lúc viết, story đó không dựng `#[tauri::command]` nào nên **không có bề mặt để nói**.
  → ✅ **ĐÓNG 2026-08-04 (Story 1.8).** Đường đã nối trọn: `try_state::<Store>()` rỗng ⇒ `bootstrap_config` trả `IpcError` mang `code = "store.open_failed"` · `MessageKey::StoreOpenFailed` · `params = {"store": "global"}` · `retryable = false` ⇒ `src/config/bootstrap.ts` bắt và đặt vào `configError` ⇒ `src/App.vue` vẽ một dải báo lỗi **không chặn**, nội dung qua `tError(err)`. Không khoá `MessageKey` mới nào và không chuỗi `vi.json` mới nào — năm khoá kho của Story 1.7 đã đủ.
  → ⚠️ **Và `open_global_store` vẫn đi tiếp — nay đó là quyết định đúng chứ không còn là ít tệ nhất.** Ứng dụng lên bằng cấu hình mặc định và **nói ra** rằng nó không đọc được kho, thay vì không lên. Doc-comment của hàm đã được sửa cho khớp.
  → ⚠️ **Một hở còn lại, ghi ra thay vì đánh dấu đạt:** dải báo lỗi chỉ hiện khi Rust **trả lời**. Một phiên `npm run dev` không có cầu IPC cho `configError = null` có chủ ý *(dựng một `IpcError` giả ở đó làm mọi lần chạy dev mọc một dải "Không mở được kho dữ liệu" — một câu sai, và một câu sẽ dạy người đọc bỏ qua đúng dải đó)*. Hệ quả: đường hiển thị này **chưa từng chạy trong một webview thật** — nghiệm thu nó cần một `$APPDATA` chỉ-đọc, và đó là một bảng nghiệm thu tay. Giao lại **Story 1.15** *(story tiếp theo mở một kho thứ hai, tức story tiếp theo có lý do thật để chạy bảng đó)*.
  → ⚠️ **VẪN CHƯA ĐÓNG sau Story 1.15 — ghi thẳng thay vì đánh dấu đạt.** Story 1.15 mở kho thứ hai thật (`project.db` qua `commands::project::create_work`) nên lý do kỹ thuật để chạy bảng này nay đã có, nhưng phiên triển khai của Story 1.15 là một agent CLI **không có công cụ điều khiển GUI desktop** (không dựng được cửa sổ Tauri thật rồi đọc màn hình bằng mắt, không có cầu debug-protocol tới WKWebView như Chrome DevTools). Bảng nghiệm thu tay này **vẫn** cần một người vận hành thật, hoặc một bộ tự động hoá desktop mới (`cliclick`/tương đương) chưa có trong môi trường build. Giao tiếp: **QA người trước khi phát hành**, hoặc story kế tiếp có công cụ GUI automation.

- ⚠️ **`tests/**` được miễn trừ khỏi phép quét ranh giới của AC2** — `src-tauri/tests/store_boundary.rs` quét `src-tauri/src/**` và **không** quét `tests/**`. Miễn trừ có tên và có lý do *(ba ca của AC6/AC7 phải dựng một database ở một phiên bản lược đồ và một chế độ journal cho trước — đúng thứ `core::store` tồn tại để mã sản phẩm không làm được)*, nhưng nó là một miễn trừ thật: một test tương lai **có thể** mở kết nối ghi thứ hai vào một kho thật mà không cổng nào báo. Cùng hạng với miễn trừ `src-tauri/tests/**` của `check-i18n.mjs`. Mở lại nếu số tệp test chạm `rusqlite` vượt quá `store_contract.rs`. **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**

- ⚠️ **AC7 nghiệm thu trên một fixture ở chế độ `delete`, không phải trên một `global.db` WAL thật của một bản tương lai.** `a_newer_schema_is_refused_without_touching_a_single_byte` dựng fixture ở `journal_mode = delete` để khẳng định *"`.db-wal`/`.db-shm` không được tạo"* một cách sạch sẽ. Một database WAL thật do một bản sau viết ra **sẽ** làm SQLite tạo `-shm` ngay khi mở — tệp `.db` vẫn không đổi một byte *(mệnh đề trung tâm của AC7 vẫn giữ, và hợp đồng thứ tự trong `Store::open` là thứ giữ nó)*, nhưng hai tệp sidecar xuất hiện rồi biến mất khi kết nối đóng. Ghi ra để lượt sau không tưởng phép kiểm rộng hơn thứ nó đo. **(Chủ: một story kế tiếp chạm `core/store`.)**

## Deferred from: code review of 1-7-tang-ghi-du-lieu-mot-writer-noi-tiep-va-luoc-do-co-phien-ban (2026-08-04)

- **Lỗi checkpoint/backup lúc chạy đều gắn nhãn `StoreError::OpenFailed`** — `src-tauri/src/core/store/pragmas.rs:249`, `wal_checkpoint()` luôn map lỗi SQLite thành `OpenFailed`, dùng cả lúc `Store::open()` (sao lưu) lẫn liên tục sau đó từ luồng checkpoint nền (đã chạy nhiều giờ). Vô hại hôm nay vì các lỗi này chỉ đi qua `Display` rồi log qua `shared.note()`, chưa bao giờ đi qua `IpcError::from`. Nếu một story sau đưa checkpoint diagnostics lên UI qua `message_key()`, một checkpoint lỗi sau nhiều giờ chạy sẽ hiển thị nhầm "Không mở được kho dữ liệu — dữ liệu chưa được nạp". Giao lại cho story nào nối chẩn đoán checkpoint lên giao diện. **(Chủ: một story kế tiếp chạm `core/store`.)**
- **Sao lưu bằng `fs::copy` không nguyên tử, không xác minh sau khi chép** — `src-tauri/src/core/store/schema.rs:137`. Nếu hết đĩa giữa chừng, tệp `.bak-v{from}` có thể bị cắt cụt; `open()` vẫn đúng đắn dừng lại (trả `Err`) nếu chính lệnh copy thất bại nên dữ liệu sống không gặp rủi ro, nhưng nếu copy "thành công" mà bị cắt cụt do lỗi hệ thống tệp không báo qua `Result`, tệp sao lưu trông hợp lệ mà thực ra thiếu. Đáng làm cứng hơn (copy-rồi-rename nguyên tử, hoặc so kích thước) nhưng không chặn Story 1.7. **(Chủ: một story kế tiếp chạm `core/store`.)**
- **`GLOBAL_TARGET_VERSION` nêu ở Task 6 chưa từng được tạo** — thay bằng hàm `pub(crate) target_version(migrations) -> u32` tính động ở `src-tauri/src/core/store/schema.rs:82`. Hợp lý hơn vì `StoreSpec.migrations` đã trở thành trường theo từng instance, nhưng là một sai khác so với hạng mục đã liệt trong story, và tầm nhìn `pub(crate)` nghĩa là không chỗ gọi bên ngoài nào (vd. IPC chẩn đoán tương lai) truy vấn được phiên bản target mà không mở `Store`. Không ảnh hưởng AC nào; ghi lại cho minh bạch. **(Chủ: một story kế tiếp chạm `core/store`.)**
- 🔴 **`Checkpointer::shutdown()` có thể để luồng nền treo lửng sau khi `close()` đã trả về** — `src-tauri/src/core/store/checkpoint.rs:228`. Đây là đánh đổi CÓ CHỦ Ý và đã ghi rõ trong doc-comment (hết ngân sách ⇒ ghi chẩn đoán rồi thoát, không join, không treo tiến trình). Rủi ro còn lại: luồng nền có thể vẫn đang chạy TRUNCATE (có thể bị chặn tới `busy_timeout` ~5s) sau khi `shutdown()`/`close()` đã trả về ở phía gọi. ~~Hôm nay vô hại vì chỗ gọi DUY NHẤT là `close_global_store()` ở `RunEvent::Exit` (`lib.rs:128`), ngay sau đó tiến trình thoát. Chỉ trở thành rủi ro thật nếu một story sau này thêm luồng "khởi động lại kho mà không thoát tiến trình". Ghi lại cho story đó.~~
  → 🔴 **ĐỔI TRẠNG THÁI 2026-08-06 (Story 1.15): TỪ "VÔ HẠI" SANG "RỦI RO THẬT".** Story 1.15 là chính story đã được cảnh báo trước: `commands::project::replace_open_work` (`lib.rs`) thay `Option<OpenWork>` trong state mỗi khi một Tác phẩm mới được tạo trong CÙNG một phiên — `Store` cũ bị `Drop` (gọi `close()`, tức `Checkpointer::shutdown()`) **giữa chừng tiến trình**, **không** thoát tiến trình. Đây đúng là "luồng khởi động lại kho mà không thoát tiến trình" mà mục này chờ. Rủi ro cụ thể: tạo hai Tác phẩm liên tiếp nhanh trong cùng phiên có thể để lại một luồng checkpoint của Tác phẩm THỨ NHẤT còn chạy TRUNCATE (tới ~5s `busy_timeout`) trong khi Tác phẩm thứ hai đã bắt đầu ghi — hai luồng checkpoint của hai kho KHÁC NHAU nên không tranh chấp dữ liệu, nhưng CPU/I/O chồng lấn chưa được đo. Chưa có test nào ép được ca này (cần dựng đúng nhịp thời gian giữa hai `create_work` liên tiếp). Giao lại cho story đo hiệu năng multi-Work (chưa có chủ) hoặc Story 2.4 (đo `Tuning`). **(Chủ: một story kế tiếp chạm `core/store`.)**
- **`Writer::shutdown()` không có trần thời gian cho `handle.join()`** — `src-tauri/src/core/store/writer.rs:159`, dựa trên giả định trong doc-comment rằng job ghi "không chặn/không gọi ra ngoài", giả định này không được kiểu hay runtime cưỡng chế. Một job chặn do bug tương lai (9 epic còn lại ghi qua tầng này) sẽ treo `RunEvent::Exit` vô thời hạn. **Ice chốt 2026-08-04 (lượt code review):** chấp nhận rủi ro — giữ kỷ luật "không bỏ dở một giao dịch đang commit để tiết kiệm mili-giây trên đường thoát" cho v1; giám sát bằng review thủ công mỗi khi một story mới ghi qua tầng này thay vì cưỡng chế bằng cơ chế.
  → ⚠️ **Story 1.15 (2026-08-06) là story ĐẦU TIÊN có job ghi kèm việc tạo dữ liệu I/O SONG SONG (ghi `meta.json`) — đã soát và giữ TÁCH RIÊNG có ý thức.** `commands::project::create_work`'s `store.write(move |tx| {...})` chạy **đúng hai** `tx.execute` (INSERT `work`, INSERT `chapter`), cả hai SQL là **hằng `&'static str`**, tham số ràng buộc qua **tuple** *(⚠️ sửa ở lượt code review 2026-08-06 — mục này trước đó ghi `rusqlite::params`, và đó là tên sai: `store_boundary.rs::FORBIDDEN` cấm token `rusqlite` ngoài `core/store`, nên `commands::project` **buộc phải** dùng tuple. Bất biến được ghi nhận — tham số **ràng buộc**, không `format!` chèn dữ liệu người dùng vào SQL — thì **đúng**; chỉ cơ chế bị gọi sai tên)*, không I/O, không gọi ra ngoài, không `Store::write` lồng nhau. `meta.json` (bao gồm `fs::write`/`sync_all`/`rename`) chạy **SAU KHI** `store.write(...)` đã trả về `Ok`, ở tầng THAO TÁC — Quyết định #3 của story, chính vì lý do này. Giả định *"job ghi không chặn, không I/O"* vẫn đúng sau story này. **(Chủ: một story kế tiếp chạm `core/store`.)**
- **`ReaderPool::acquire()` chờ `Condvar` không trần** — `src-tauri/src/core/store/reader.rs:107`, không đối xứng với bảo đảm hữu hạn (`StoreError::WriterGone`) của đường ghi; có thể chờ mãi nếu pool cạn kiệt vì một `Lease` rò rỉ hoặc một read job bị chặn. **Ice chốt 2026-08-04 (lượt code review):** chấp nhận như hiện tại — đường đọc không có tác dụng phụ chờ đợi bên ngoài giống job ghi; rủi ro rò rỉ `Lease` thấp hơn rủi ro một job ghi bị chặn. **(Chủ: một story kế tiếp chạm `core/store`.)**

## Deferred from: 1-8-phan-giai-cau-hinh-hai-tang (2026-08-04)

- 🔴 **`ScopeResolver` chưa cache gì, và đó là một quyết định.** Consumer đường nóng duy nhất là khớp Glossary khi gõ — **Story 3.4**, dưới trần NFR2 *"không frame nào vượt 50 ms"* — và hôm nay nó chưa tồn tại. Dựng cache bây giờ là dựng một cơ chế vô hiệu hoá mà **không có gì để vô hiệu hoá**, và một cơ chế như vậy sẽ sai theo đúng cách mà không test nào bắt. Ba hàm phân giải là **thuần** nên thêm cache về sau là một lượt sửa cục bộ, không phải một lượt mổ. **Chủ sở hữu: Story 3.4** — và nó phải **đo trước** khi cache. **(Chủ: Story 3.4.)**

- ~~⚠️ **Tầng Tác phẩm chưa từng chạy trên dữ liệu thật.**~~ Nhánh `Some(..)` của cả ba hàm phân giải **có test đầy đủ** *(`scope_contract.rs` cấp dữ liệu tầng Work bằng tay)*, nhưng đường sản phẩm hôm nay **luôn** truyền `None`: `.atproj` và `project.db` là **Story 1.15**, `StoreKind::Project` chưa có `StoreSpec` nào. `ScopeResolver::global_only()` là hàm dựng duy nhất tồn tại và `WorkScope` là một struct rỗng đánh dấu chỗ. **Story 1.15** cắm tầng thật vào; không ba chữ ký không phải đổi.
  → ✅ **ĐÓNG MỘT PHẦN 2026-08-06 (Story 1.15).** `WorkScope` nay mang `work_id` thật; `ScopeResolver::with_work(WorkScope)` là hàm dựng thứ hai, không ba chữ ký `apply_override`/`apply_merge`/`resolve_global_only` không đổi. Đường sản phẩm (`commands::project::create_work`) dựng một `ScopeResolver::with_work(...)` thật mỗi khi một Tác phẩm được tạo — `has_work_tier()` không còn luôn `false` trên đường sản phẩm.
  → ⚠️ **Vẫn còn hở, ghi ra thay vì đóng trọn:** chưa có method phân giải nào (`apply_override`/`apply_merge`/`resolve_global_only`) thật sự được GỌI với dữ liệu tầng Work — Story 1.15 không có bảng nào ở tầng Work để tra (Glossary/TM/Prompt là các epic sau). Resolver tồn tại và phản ánh đúng trạng thái *"đang mở"*, nhưng "phân giải hai tầng trên dữ liệu Work thật" vẫn chờ **Epic 3+**.

- ⚠️ **Mã hoá hợp âm trên đĩa là TẠM.** `config_value` lưu hợp âm dưới dạng **một chuỗi**, và `src/main.ts::toBindings` tách bằng dấu phẩy. Nó đủ cho AC5 *(đọc lại được đúng hàng đã ghi)* và nó phân biệt được *"cố ý không có phím"* `""` với *"chưa ai đặt gì"* (khoá vắng mặt), nhưng nó **không** phải một mô hình: không escape, nên một hợp âm chứa dấu phẩy là không biểu diễn được. **Chủ sở hữu: Story 1.21** *(màn hình gán phím)* — story đó có mô hình thật thì thêm bước di trú của **chính nó**.
  → ✅ **ĐÃ ĐÓNG 2026-08-11 (Story 1.21) — bằng một PHÉP ĐO rồi một CƠ CHẾ, không bằng một mô hình mới và không một bước di trú nào.** Nỗi lo ở đây là *"một hợp âm chứa dấu phẩy là không biểu diễn được"*. Đo trên `src/commands/keys.ts`: phím dấu phẩy viết là **`Comma`** — một tên chữ cái — nên hợp âm của nó là `'Mod+Comma'`, **không** `'Mod+,'`; và `keyToCode` chỉ nhận `[0-9]`, `[A-Za-z]` và các khoá của `NAMED_CODES`, không khoá nào chứa `,`. ⇒ **không hợp âm hợp lệ nào chứa dấu phẩy**, và mã hoá hiện tại an toàn **theo cấu trúc**, không do tình cờ.
  🔴 Và phép đo đó đã thành **cơ chế**, vì *"đúng do tình cờ"* và *"đúng có lưới"* là hai thứ khác nhau: Kiểm D của `scripts/check-commands.mjs` nay đọc bảng `NAMED_CODES` **từ chính mã nguồn** *(không một bản chép trong script — một bản chép sẽ trôi khỏi sự thật trong hai story)*, lái cả **61** phím mà `keyToCode` có thể sinh ra, và khẳng định không hợp âm nào chứa `,`. Một hàng mới thêm vào bảng ngày mai **tự động** bị kiểm.

- ⚠️ **Xung đột hợp âm từ đĩa chỉ được *sống sót*, chưa được *giải quyết*.** `installCommands` thử dựng keymap trên một registry **nháp** trước; xung đột ⇒ ghi chẩn đoán rồi **rơi về hợp âm mặc định**, nên một `global.db` sửa tay không cho ra cửa sổ trắng (§Bẫy 5). Nhưng người dùng chỉ biết nếu họ mở console, và lựa chọn của họ im lặng không được áp. **Màn giải quyết xung đột là Story 1.21**; đừng đọc chốt hiện tại rộng hơn thứ nó làm.
  → ✅ **ĐÃ ĐÓNG 2026-08-11 (Story 1.21, AC13).** Chẩn đoán không còn dừng ở `console.error`: `installCommands` ghi lý do trượt vào một biến module đọc được qua `shortcutsDiskRejection()`, và `ShortcutsOverlay.vue` hiện câu `shortcuts.disk_rejected` — *"Bộ phím tắt đã lưu có hai thao tác giành cùng một phím nên chưa được áp — ứng dụng đang chạy bằng phím mặc định. Lựa chọn cũ chưa bị xoá."* Người dùng biết mà không phải mở console, và họ có một màn hình để sửa.
  ⚠️ **Registry nháp đã BIẾN MẤT cùng lượt này, và đó là một sự đơn giản hoá chứ không một sự đánh đổi:** lớp hợp âm của đĩa nay là tham số `overrides` của `createKeymap`, mà `createKeymap` **chỉ đọc** registry — nên lượt thử chạy thẳng trên registry thật. Một biến thể ít hơn để hai đường trôi khỏi nhau.
  🔴 **Vế NGHIỆM THU vẫn mở:** hàng 14 của bàn đo *(sửa tay `global.db` cho hai command cùng một hợp âm → mở app → câu đó phải hiện)* **chưa chạy** — nó cần một tiến trình Tauri thật. Xem mục Story 1.21 ở cuối tệp.

- ⚠️ **`src/config/` là một thư mục frontend NGOÀI cây nguồn đã khai.** `ARCHITECTURE-SPINE.md#Structural Seed` chỉ liệt kê `modes/ panels/ layout/ commands/ tokens/ i18n/`. Lý do chấp nhận, ghi ra để lượt review phân xử chứ không tự coi là đã duyệt: nó **không phải một khái niệm miền mới** mà là **adapter IPC phía webview** *(một `invoke`, một `try/catch`, không quy tắc nào)*; đặt nó vào `src/commands/` sẽ kéo `@tauri-apps/api` vào một thư mục mà ba phép kiểm hành vi nạp bằng **Node thuần** (§Bẫy 6), đặt vào `src/modes/` thì sai khái niệm. *(Mặc định của story, Ice ký 2026-08-04.)* **(Chủ: Winston — architect.)**

- ⚠️ **Story này ghi qua tầng `store::Writer` — nên nó kích hoạt điều kiện mà Ice đã đặt cho `writer.rs:159`.** Mục *"`Writer::shutdown()` không có trần cho `handle.join()`"* được Ice chấp nhận 2026-08-04 **với điều kiện review tay mỗi khi một story mới ghi qua tầng này**. Đã soát: `save_value` chạy **đúng một** `tx.execute` với SQL hằng, không I/O, không gọi ra ngoài, không `Store::write` lồng nhau *(`writer.rs:104` trả `WriteFailed` chứ không xếp hàng, và story này không đi vào đường đó)*. Giả định *"job ghi không chặn"* vẫn đúng sau story này. **(Chủ: một story kế tiếp chạm `core/store`.)**

- ⚠️ **Đây là lượt di trú THẬT đầu tiên trên một `global.db` đã có dữ liệu, tức lần đầu đường sao lưu `fs::copy` chạy trên máy người dùng.** `schema.rs:137` đã ghi nhận rằng bản sao đó **không nguyên tử và không xác minh lại**. Không sửa ở story này *(ngoài phạm vi, và mục đã có chủ sở hữu)*, nhưng ghi lại quan sát: từ hôm nay mục đó không còn là lý thuyết — mọi người dùng đã chạy một bản có `user_version = 1` sẽ đi qua nó đúng một lần khi nâng cấp. **(Chủ: một story kế tiếp chạm `core/store`.)**

- ⚠️ **Đường hiển thị lỗi kho chưa chạy trong webview thật** — xem mục đã cập nhật ở §*Deferred from: code review of 1-7* (*"Lỗi mở kho hôm nay chỉ ra `stderr`"*). Nghiệm thu cần một `$APPDATA` chỉ-đọc; **Story 1.15 vẫn KHÔNG đóng được mục này** — môi trường triển khai của nó không có công cụ GUI automation, xem ghi chú 2026-08-06 ở mục gốc. **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

- ✅ **`tests/**` miễn trừ khỏi phép quét ranh giới: mục ở §1-7 **KHÔNG** phải mở lại.** Hai tệp test mới của story này **không** gõ tên crate SQLite: `Store::write` nhận một closure lấy `&Transaction` — kiểu **tái xuất** từ `core::store` — nên ca ghi thẳng một hàng vào `global.db` viết được mà không chạm `rusqlite`. Số tệp test chạm crate đó vẫn đúng bằng `store_contract.rs`.

- ⚠️ **Sàn quần thể vẫn đếm TỆP, không đếm nội dung** — `scope_boundary.rs::RS_FLOOR = 20` (thật: 26) và `check-i18n.mjs::RS_FLOOR = 21` (thật: 27). Cùng mục đã ghi ba lần trước cho `check-i18n.mjs:207-218` và `check-commands.mjs`. ⚠️ **Hai quần thể này KHÁC nhau** — `src-tauri/src/**` so với `src-tauri/**` sau miễn trừ `tests/**` *(gồm `build.rs`)* — và chép số của tệp này sang tệp kia là đặt một cái sàn cho một cây khác. Đã ghi vào doc-comment của cả hai. **(Chủ: một story hạ tầng cổng kế tiếp.)**

## Deferred from: code review of 1-8-phan-giai-cau-hinh-hai-tang (2026-08-04)

- **`"ScopeKind"` vẫn còn nửa bẫy sau khi `ScopeResolver::resolve_override`/`resolve_merge` đã đổi tên thành `apply_override`/`apply_merge`** (`src-tauri/tests/scope_boundary.rs:62-67`, `src-tauri/src/core/scope/mod.rs:201,218`) — lượt sửa hôm nay xoá đúng hai token đụng độ (`resolve_override`/`resolve_merge`), nhưng lời gọi hợp lệ tương lai vẫn phải viết `ScopeKind::Glossary` (hay tương đương) để truyền tham số, và `"ScopeKind"` vẫn bị cấm ngoài `core/scope/**`. Cổng AC1 sẽ vẫn đỏ ở token này ngay lần đầu Epic 3/4/6/7 gọi `apply_override`/`apply_merge` từ module của họ — dù đường gọi hoàn toàn đúng. Xoá triệt để đòi đổi chữ ký `apply_override`/`apply_merge` sang nhận `kind: &str` thay vì `kind: ScopeKind` (giống khuôn `save_value` ở ranh giới IPC), để domain module không cần gõ tên kiểu `ScopeKind` trong mã của họ. Ice chọn KHÔNG làm việc đó ở lượt review này (2026-08-04) — giao cho story đầu tiên thật sự trở thành consumer. **(Chủ: story đầu tiên thật sự trở thành consumer của `ScopeKind`.)**
  → ✅ **ĐÃ ĐÓNG 2026-08-19 (Story 3.1).** `ScopeResolver::apply_override`/`apply_merge`/`resolve_global_only` nay nhận `kind: &str` và phân giải nội bộ bằng `ScopeKind::from_wire` (`src-tauri/src/core/scope/mod.rs`), đúng khuôn `save_value`/`delete_value`. `core/glossary/store.rs` gọi `apply_override("glossary", ..)` bằng một hằng literal và không `use` `ScopeKind` ở đâu cả. Chữ ký sai kiểu giờ là một lỗi lúc chạy (`ScopeError::UnknownKind`) thay vì lỗi biên dịch — cổng `scope_boundary.rs::only_core_scope_may_name_the_two_tier_vocabulary` xanh **mà danh sách `FORBIDDEN_OUTSIDE_SCOPE` không đổi một dòng**, đúng như AC của story này đòi.
- `resolve_one` trong `load_global_config` nuốt lỗi `WrongSemantics` bằng `debug_assert!` rồi `unwrap_or_default()` (`src-tauri/src/core/scope/store.rs:135-147`) — trong build release, một thay đổi ngữ nghĩa tương lai cho `AppConfig`/`Shortcut`/`LayoutPreset` mà quên sửa chỗ gọi này sẽ rơi về map rỗng im lặng thay vì lỗi. Rủi ro thấp vì `cargo test` bắt buộc trước khi merge sẽ đỏ ở debug build, nhưng ghi lại cho lượt sau. **(Chủ: một story kế tiếp chạm `core/scope`.)**
- `watch(currentMode)` gọi `put_config` không có khoá thứ tự (`src/main.ts:178-184`) — đổi chế độ liên tiếp rất nhanh có thể khiến một giá trị trung gian được ghi cuối cùng xuống đĩa do các lời gọi `invoke` hoàn tất không đúng thứ tự gọi. Tự phục hồi ở lượt chuyển chế độ kế tiếp. **(Chủ: story kế tiếp chạm `src/main.ts` (đổi chế độ).)**
- Nhánh lỗi `store.read_failed` của `bootstrap_config` chưa có test ép đường đọc thật trượt (`src-tauri/tests/scope_contract.rs:700`) — `every_command_error_comes_from_the_store_vocabulary` chỉ ép các nhánh `OpenFailed`/`WriteFailed`. Đường lan `?` và phép chuyển `From<StoreError>` đã được kiểm ở tầng `store` (Story 1.7); thiếu một ca tích hợp trực tiếp qua `bootstrap_config`/`load_global_config`. **(Chủ: một story kế tiếp chạm `core/scope`.)**
- `save_value` không giới hạn độ dài `key`/`value` trước khi ghi vào `config_value` (`src-tauri/src/core/scope/store.rs:203`) — mọi lời gọi hôm nay đến từ frontend tin cậy của chính ứng dụng, không phải một biên tin cậy với dữ liệu ngoài. **(Chủ: một story kế tiếp chạm `core/scope`.)**
- `rel_posix(...).starts_with(SCOPE_DIR)` không có dấu `/` đuôi (`src-tauri/tests/scope_boundary.rs:40`, chép nguyên khuôn `store_boundary.rs` của Story 1.7) — một thư mục anh em tên `core/scope_legacy` sẽ khớp nhầm miễn trừ. Xác suất gần như bằng không. **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**
- `code_lines()` trong `scope_boundary.rs` (và bản gốc `store_boundary.rs`) chỉ miễn trừ dòng bắt đầu bằng `//`, không bóc comment khối `/* … */` (`src-tauri/tests/scope_boundary.rs:133`) — một token cấm nằm trong comment khối sẽ báo vi phạm giả. Codebase không dùng comment khối nên rủi ro thực tế thấp. **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**

## Deferred from: 1-9-dung-du-lieu-tu-dien-lop-nen (2026-08-04)

- 🔴 **Lưới thay thế cho `bundle.resources`/`dict/*.db` — chủ sở hữu: Story 10.1.** **(Chủ: Story 10.1.)** Task 10 của story này gỡ `$RESOURCE/dict/**` khỏi `assetProtocol.scope` (Ice chốt 2026-08-04, đóng `deferred-work.md:21`+`:57`) vì webview không bao giờ đọc tệp từ điển — `rusqlite` mở tệp bằng đường dẫn hệ thống, không qua asset protocol. Hệ quả: từ hôm nay đến Story 10.1, **không còn dòng nào trong `tauri.conf.json` nhắc tới `dict`**, tức lưới cũ *"ship một bản không có byte từ điển nào thì phải lộ ra ở đâu đó"* mất chỗ bấu — `config_invariants.rs` không còn gì để kiểm về việc `dict-core.db` có được đóng gói hay không. **Story 10.1 phải làm hai việc cùng lúc, không phải một:** (1) thêm `dict/*.db` vào `bundle.resources` của `tauri.conf.json`; (2) thêm một test khẳng định nó **có mặt** trong cấu hình đóng gói — nếu chỉ làm (1) mà không làm (2), lỗ hổng lưới lặp lại y hệt mục đã đóng ở đây.
  → 🔄 **Cập nhật sau Story 1.10 (2026-08-05): phạm vi giờ là BA tệp, không phải một.** `dict-core.db` + `dict-thieu-chuu.db` + `dict-vietphrase.db` — cả ba phải vào `bundle.resources`, và test khẳng định "có mặt" (2) phải khẳng định cả ba, không chỉ base. **Đánh dấu KHÔNG đóng** — vẫn là việc của Story 10.1.
- ⚠️ **`ARCHITECTURE-SPINE.md` (AD-23, dòng ~316) còn liệt kê `$RESOURCE/dict/**` bằng chữ — ĐANG LỆCH khỏi `tauri.conf.json` kể từ Task 10 của story này.** Dev không sửa tài liệu quy hoạch (tiền lệ quyết định #3 của Ice ở Story 1.3); sửa AD-23 là việc riêng của Ice. Ai đọc AD-23 trước khi đọc cấu hình thật sẽ hiểu sai rằng `dict` vẫn còn trong scope. **(Chủ: Winston — architect.)**

## Deferred from: 1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap (2026-08-05)

- 🔴 **HVTĐTD + Cổ hán văn — hai lớp gỡ rời còn lại, chưa có nguồn thô. Chủ sở hữu: story nối tiếp Story 1.10.** **(Chủ: story nối tiếp Story 1.10.)** Ice chốt 2026-08-05 thu hẹp phạm vi Story 1.10 từ bốn lớp xuống hai (Thiều Chửu + VietPhrase) vì hai lớp này chưa có nguồn thô và không thể tự tìm thay thế:
  - **HVTĐTD** *(Hán Việt Từ Điển Trích Dẫn, tác giả Đặng Thế Kiệt)* — tác giả **đã đồng ý bằng văn bản 2026-08-02** (PRD §8.5), nhưng **không tồn tại bản tải hàng loạt công khai**: `winvnkey.sourceforge.net/hanviet` là *"work in progress"*, `vietnamtudien.org/hanviet` chỉ là cổng tra cứu (`e-hvtd v2.5`). **Phải xin trực tiếp tác giả** một bản tải hàng loạt trước khi story nối tiếp có thể bắt đầu.
  - **Cổ hán văn** — chưa có nguồn nào. Tam tự kinh / Thiên tự văn / Bách gia tính là **văn bản**, không phải từ điển — cần **quyết lại "nó là lớp gì"** (một dạng `dict_source` khác? một loại tài liệu riêng?) trước khi đi tìm tệp.
  - Hạ tầng đa lớp (CLI `--layer`, `finalize` dùng chung, bảng phân phối, parity `sqlite_master`, hai cổng `.mjs`) đã dựng ở Story 1.10 và dùng lại được nguyên vẹn — **không sửa** hạ tầng đã có (§Quyết định #2 của Story 1.10). 🔄 **Nhưng "chỉ là thêm ba chỗ" là SAI** *(sửa sau lượt code review 2026-08-05 — người làm story nối tiếp tin con số đó sẽ mất nửa buổi truy `cargo test` đỏ)*. Danh sách ĐẦY ĐỦ các điểm phải sửa cho MỘT lớp mới:
    1. `sources_meta.rs` — một biến thể `LicenseRef` · một hằng `SourceMeta` · phần tử trong `DETACHABLE_ALL` · **và** test `exactly_two_detachable_sources_in_scope_today` *(đang hardcode `2` + vector mã)*
    2. `licenses.rs` — một `include_str!` + hàm dựng văn bản, cộng tệp trong `assets/licenses/`
    3. `sources/mod.rs` — khai module parser mới
    4. `build.rs` — phần tử trong `DETACHABLE_LAYERS` *(bảng phân phối THẬT; test `distribution_table_matches_detachable_all_exactly` sẽ đỏ nếu quên — đó là chủ ý)*
    5. `dict-manifest.toml` — một khối `[[detachable]]` đủ bốn trường
    6. `scripts/check-dict-manifest.mjs` — `EXPECTED_DETACHABLE_NAMES`
    7. `scripts/check-dict-build.mjs` — `RS_FILE_FLOOR` *(sàn đi theo số tệp `.rs`)*
    8. `tests/layers.rs` — `assert_eq!(report.detachable.len(), 2)` và các cặp lớp viết cứng
    9. `main.rs` — chuỗi usage `[--layer <base|…|all>]`
    10. `tools/dict-build/README.md` — bảng nguồn + quy ước `raw/`
    ⚠️ Kiểm D/E/F của `check-dict-build.mjs` **tự đi theo** `DETACHABLE_ALL`, không phải sửa tay — đó là phần thật sự "miễn phí".
- 🟡 **FR36 nghiệm thu HÀNH VI ("xoá file, chạy lại bộ test tra cứu") — chủ sở hữu: Story 1.13.** Story 1.10 giao **điều kiện CẤU TRÚC** của FR36 (mỗi lớp một tệp `.db` độc lập — AC1; lược đồ đồng nhất giữa các tệp — AC4), nhưng KHÔNG viết đường tra cứu (đó là 1.11/1.13) nên KHÔNG thể nghiệm thu hành vi thật của AD-10 *"xoá file → chạy lại bộ test tra cứu → hệ thống vẫn hoạt động đầy đủ với các nguồn còn lại"*. Không đánh dấu FR36 là "đã nghiệm thu" cho tới khi 1.13 viết phép thử này.
- 🟡 **`dict_source.id` KHÔNG toàn cục giữa các tệp — chủ sở hữu: Story 1.11/1.13.** Mỗi tệp `.db` (base, `dict-thieu-chuu.db`, `dict-vietphrase.db`, …) có bảng `dict_source` RIÊNG của chính nó, nên `id = 1` xuất hiện ở CẢ BA tệp hôm nay, trỏ tới BA nguồn khác nhau. Trong phạm vi MỘT tệp, FK vẫn đúng tuyệt đối. Nhưng đường đọc của 1.11/1.13 khi GOM kết quả từ nhiều tệp **phải khoá theo `code` (chuỗi), không theo `id` (số)** — gộp theo `id` sẽ dán nhãn sai nguồn cho một nghĩa (vd. gán "Thiều Chửu" cho một nghĩa thật ra từ CVDICT), tức FR31 vỡ theo cách thầm lặng nhất có thể. Không phải việc của Story 1.10, nhưng tình huống này ra đời chính từ kiến trúc "một tệp một `dict_source`" mà story này dựng.
- 🟡 **Nghĩa vụ thông báo tác giả HVTĐTD khi công cụ hoàn thành (PRD §8.5) — chủ sở hữu: Story 10.4.** Tác giả Đặng Thế Kiệt đề nghị được thông báo khi AuraTranslate hoàn thành/phát hành. Nghĩa vụ này không mang số FR nên rất dễ rơi mất khỏi dòng chảy story — ghi rõ ở đây để nó còn xuất hiện ở ĐÂU ĐÓ. Màn hình Attribution (Story 10.4) là nơi tự nhiên nhất để không quên: khi HVTĐTD được đóng gói thật (story nối tiếp của 1.10), nhắc Ice gửi thông báo cho tác giả.

## Deferred from: code review of 1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap (2026-08-05)

- 🟡 **`require_nonempty` chỉ chặn ĐÚNG mốc 0 entry — không có ngưỡng tỉ lệ bỏ dòng** (`tools/dict-build/src/build.rs:66`). Hạ tầng Story 1.9, Story 1.10 chỉ tái dùng. Kịch bản chưa được chặn: một nguồn bị đọc sai mã hoá (đúng ca UTF-16LE mà `tools/dict-build/README.md` mới thêm cảnh báo) khiến 679.310/679.311 dòng thành `ParseIssue` còn 1 dòng may mắn giải mã được ⇒ `entries == 1 > 0` ⇒ build **THÀNH CÔNG**, `ExitCode::SUCCESS`, sinh một `.db` gần rỗng và in SHA-256 để chép vào `dict-manifest.toml`. `require_nonempty` là chốt chặn duy nhất theo chính lời README, và nó không bắt được ca 99,9997% hỏng. Cần một ngưỡng tỉ lệ (`skipped / (read)` vượt N% ⇒ lỗi) — **chọn N là quyết định, không phải mặc định hiển nhiên**, nên không vá trong lượt review này.
- 🟡 **VietPhrase: đầu mục TRÙNG không được gộp — 46 trong nguồn thô, 18 trong `dict-vietphrase.db` đã dựng** (`tools/dict-build/src/sources/vietphrase.rs:19`). Mỗi dòng → một `RawEntry` → một `dict_entry`; lược đồ không có UNIQUE trên `dict_entry(source_id, headword)` (`src/schema.rs:44-58`) nên build không hề đỏ. Ngược khuôn Group A của Story 1.9, nơi `sources/wiktextract_common.rs` được sửa đúng vì lỗi *"nhiều dòng cùng headword ⇒ nhiều `dict_entry`"* và có hẳn test khoá (`en_wiktionary_same_headword_ma_merges_into_one_entry_with_multiple_senses`). **Không vá vì spec Story 1.10 chốt mô hình khác:** §Thông tin kỹ thuật ghi *"Mục hợp lệ: **679.311** — mọi dòng đều là mục"*, và dev đo được 679.302 entry, khớp. Hậu quả thật: tra `不是他的对手` trong `dict-vietphrase.db` trả về HAI `dict_entry` từ CÙNG một nguồn ⇒ UI của **1.13** hiện hai khối "VietPhrase" giống hệt nhau. → **Chủ sở hữu: Story 1.11/1.13** — hoặc gộp lúc đọc, hoặc quyết lại mô hình lúc dựng.
- 🟡 **VietPhrase tách `/` vô điều kiện, không một ngoại lệ** (`tools/dict-build/src/sources/vietphrase.rs:77`). Bất kỳ nghĩa hợp lệ nào chứa `/` — `và/hoặc`, `24/7`, `n/a`, một URL trong nghĩa — bị bẻ thành nhiều `dict_sense` giả với `ord` 0/1, trong khi `ord` được cả story tuyên bố là *"thứ tự ƯU TIÊN của bản dịch"*. **Không vá vì Task 6 chốt thẳng luật tách `/`**, và không có luật rõ nào phân biệt dấu `/` phân tách với dấu `/` trong nội dung. Ghi lại để lượt rà chất lượng dữ liệu sau này (hoặc story harvest thuật ngữ 8.14/8.15) biết nó tồn tại.
- 🟢 **§Bẫy 8 — điều kiện "điền manifest + siết cổng cùng MỘT commit" còn TREO.** Cả hai vế đang nằm trong cây làm việc chưa commit (`dict-manifest.toml` hai khối `[[detachable]]` + `scripts/check-dict-manifest.mjs` đòi đúng hai mục), nên một commit là đủ để thoả — nhưng chưa xác minh được cho tới khi commit thật diễn ra. Nếu tách thành hai commit, `check:dict-manifest` sẽ ĐỎ ở mọi lượt push trên cả hai nền tảng cho tới commit thứ hai — đúng thứ §Bẫy 8 tồn tại để tránh.
  → ✅ **ĐÃ ĐÓNG 2026-08-19 (Story 2.13) — phép kiểm đã CHẠY, không suy luận.** `dict-manifest.toml`
  gốc kho hôm nay mang `[[detachable]]` đúng **3** mục (`thieu-chuu` · `vietphrase` · `tran-van-chanh`),
  cả ba có `sha256`/`url`/`source_version` THẬT (không placeholder). `npm run check:dict-manifest`
  chạy tại chỗ 2026-08-19: **tất cả phép kiểm đạt**, gồm câu *"[[detachable]] có đúng 3 mục, đúng
  tên"*. Cả hai vế đã committed từ lâu (cây làm việc sạch ở đầu story này) — điều kiện "cùng MỘT
  commit" mà §Bẫy 8 lo ngại đã trôi qua an toàn; không còn gì TREO.

## Deferred from: correct-course — đường tiếng Anh (2026-08-05)

- 🔴 **Dư địa NFR6 nay là 15.474.554 byte ĐO THẬT — HVTĐTD và Cổ hán văn phải được ĐO trước khi hứa đóng gói. Chủ sở hữu: story nối tiếp của Story 1.10.** **(Chủ: story nối tiếp của Story 1.10.)**
  🔄 **CẬP NHẬT 2026-08-05 sau Story 1.10b — số DỰ PHÓNG thay bằng số ĐO ĐƯỢC.** Lớp tiếng Anh đã dựng thật; `dict-core.db` đi từ `154.464.256` → **`194.998.272` byte** *(**+40.534.016**, dự phóng là +40.333.312 ⇒ hụt **200.704** byte)*. Kế toán thật: baseline `.dmg` **2.334.696** + license **35.149** + font **21.285.713** + `dict-core.db` **194.998.272** + Thiều Chửu **5.787.648** + VietPhrase **160.083.968** = **384.525.446 byte** *(384,53 MB thập phân)*. **ĐẠT** trần **400.000.000**, **dư địa thật 15.474.554 byte** *(dự phóng 15.675.258 ⇒ **hẹp hơn 200.704**; dự phóng chính xác 99,95%)*.
  Thang so sánh **không đổi**: Thiều Chửu **5.787.648** *(vừa)* · VietPhrase **160.083.968** *(không vừa, gấp **10,3 lần** dư địa)*. Nếu HVTĐTD giàu ví dụ + trích dẫn như `prd.md` §8.3 mô tả, **trần 400 MB sẽ vượt lần thứ hai** — và lúc đó đường ra không còn là nâng trần tiếp: phải cân nhắc lại chính lời hứa *"không tải thêm sau khi cài"*, tức chạm **NFR7** và **NFR12**. **Không dựng HVTĐTD rồi mới đo** — đo trước, báo số cho Ice, rồi mới quyết đóng gói.
- ~~🔴 **AD mới cho đường tra cứu tiếng Anh — chủ sở hữu: Winston (`bmad-architecture`). 🔴 VẪN CHẶN Story 1.11b.**~~
  → ✅ **ĐÃ ĐÓNG 2026-08-05 — `AD-44` đã vào `ARCHITECTURE-SPINE.md`. Story 1.11b KHÔNG còn bị chặn bởi mục này.** Sáu mệnh đề của AD-44: ① vị từ điều phối là **hình dạng chuỗi truy vấn** *(có ký tự Hán ⇒ đường zh; ngược lại ⇒ đường en)*, không phải ngôn ngữ của Tác phẩm — nên ca *"bôi đen `API` trong truyện Trung"* mà chính mục này nêu **ra kết quả đúng** thay vì rỗng; vị từ chạy **một lần mỗi lượt tra, TRÊN tầng gom**, và không tồn tại sổ đăng ký *"tệp nào chứa ngôn ngữ nào"*. ② đường en có **hai** nhánh *(exact B-tree · FTS5 trigram ≥ 3)*, không có `char_idx`. ③ tập khoá tra chính xác = `{nguyên văn, hạ chữ thường}` trong **một** truy vấn `IN (?1,?2)`, không fallback dây chuyền — và 🔴 **stemming KHÔNG nằm trên đường nóng tra từ điển**, xem mục kế tiếp. ④ chuỗi con **1–2 ký tự** tiếng Anh khai là **không hỗ trợ**, trả trạng thái phân biệt được với *"không có kết quả"*. ⑤ ranh giới mã-riêng-theo-ngôn-ngữ: được phép **đúng** ở chiến lược truy vấn trong `core/dict/`; không cấm ở cổng `DictionarySource` *(adapter theo **tệp**, không theo **ngôn ngữ**)*, ở hình dạng bản ghi kết quả *(`lang` là **trường**, không phải **kiểu**)*, và không cấm mọi bước hợp nhất zh với en. ⑥ **NFR1 đo TRÊN đường tiếng Anh**, không suy từ số tiếng Trung. Kèm theo: **AD-26 sửa Rule tại chỗ** — phạm vi *"tiếng Trung"* đưa vào **thân** Rule chứ không chỉ ở tiêu đề, và dải hiệu năng công bố *(0,15–4,5 ms)* đánh dấu **LỖI THỜI**, thay bằng số đo 2026-08-05 mà Story 1.11 đã bàn giao ở §Completion Notes ②. Reviewer Gate: `lint_spine.py` 0 findings, ba lens bắt 5 phát hiện *(2 nghiêm trọng)* — tất cả đã vá; báo cáo ở `architecture/architecture-AuraTranslate-2026-08-02/reviews/review-ad-44-2026-08-05.md`.

- 🔵 **PHÁT HIỆN MỚI của lượt AD-44 (2026-08-05) — đo thật, không suy luận: `FR40` trên đường TỪ ĐIỂN không cần stemming, và chữ HOA mới là lỗ thật.**
  - **Stemming mua được ~0 recall.** Thứ phủ FR40 là một **tính chất của corpus**: Wiktionary đã có sẵn mọi dạng biến thể làm **đầu mục riêng** — mẫu **16/16** có mặt, **gồm cả bất quy tắc** `went` · `gone` · `children` · `happiest`, thứ stemming về nguyên tắc **không bao giờ** làm được. Quy mô: **7.656** đầu mục `-ing` · **8.855** `-ed` · **19.616** `-s` · **228** `-est` trên **119.039**. *(Dữ kiện phụ, yếu hơn: ba dạng stem Porter kinh điển tra vào `dict-core.db` cho **0** hàng — `dictionari` · `studi` · `happi`; `run` cho 1. ⚠️ **Số hàng là đo thật, nhưng ba CHUỖI stem đó chưa chạy qua stemmer mà sản phẩm sẽ dùng** — ghi rõ trong AD-44 ③.)*
  - 🔴 **Lỗ chữ HOA, chưa tài liệu nào ghi:** `headword='running'` ⇒ **1** hàng nhưng `headword='Running'` ⇒ **0**. Bôi đen một từ ở **đầu câu** là thao tác thường ngày và nó trả rỗng không báo gì — đúng lớp lỗi FR39/AD-26 tồn tại để chặn. **1.635** đầu mục en mang chữ hoa có nghĩa (`API` · `Wikipedia` · `English`) và **184** nhóm chỉ khác nhau ở chữ hoa ⇒ hạ chữ thường phải là **THÊM** khoá, không phải **THAY**.
  - ⇒ **Hệ quả cho Story 1.11b:** `Matcher` của 1.12 **không còn là điều kiện chặn** — 1.11b không gọi nó. **Story 1.12 vẫn dựng Matcher đầy đủ** cho Glossary (FR51) và TM (FR61), nơi thuật ngữ do **người dùng tự viết** nên corpus không mang tính chất trên và stemming thật sự đáng tiền *(ranh giới này đã ghi vào Rule của AD-17)*.
  - 🟡 **`epics.md` ĐANG LỆCH khỏi AD-44 ở hai chỗ — chủ sở hữu: John (PM), Winston không sửa `epics.md`.** **(Chủ: John — PM.)** *(a)* Mục Story 1.11b `:1478` ghi *"biến thể hình thái **dùng `Matcher` của AD-17**, không cài riêng một bản thứ hai"* — AD-44 ③ nay nói đường **từ điển** không gọi Matcher, và ghi số đo làm lý do. Câu đó nên đổi thành *"tập khoá `{nguyên văn, hạ chữ thường}`"* cộng một AC cho lỗ chữ HOA *(`Running` ⇒ 0 hàng)*. *(b)* Cùng mục còn ghi 🔴 *"CHẶN: cần một AD mới… chủ sở hữu Winston"* — **nay đã giao**, dòng chặn nên gỡ. **Hệ quả thứ tự:** lý do đảo `1.12` lên trước `1.11b` *(Ice chốt 2026-08-05, vì 1.11b cần Matcher)* nay **không còn**; thứ tự trong `sprint-status.yaml` và `epics.md` chưa ai đổi, nên `bmad-create-story` sẽ tự chọn `1-11b` — và **đó nay là lựa chọn đúng**.
  - 🟡 **Việc tầng PRD, không tự sửa:** `prd.md` FR40 phát biểu yêu cầu **bằng cơ chế** *(*"nhận diện biến thể hình thái"* + ghi chú stemming/lemmatization)*. Trên đường **từ điển**, cơ chế thật là *"corpus có sẵn mọi dạng biến thể"*, và nó phủ **rộng hơn** stemming. Chủ sở hữu: **John (PM)** — cân nhắc tách FR40 thành *(a)* tra cứu từ điển và *(b)* khớp Glossary/TM, vì hai vế nay có hai cơ chế và hai giới hạn khác nhau. `AD-26` tên đầy đủ là *"Ba nhánh truy vấn **tiếng Trung**"* và cả ba nhánh đều là cơ chế cho chữ Hán: tra chính xác đầu mục *(dùng được cho EN)* · chuỗi con 1–2 ký tự qua `char_idx` *(**vô dụng** với tiếng Anh)* · chuỗi con 3+ ký tự qua FTS5 `trigram` *(chạy được nhưng không phải hình dạng đúng)*. Tiếng Anh cần **exact + stemming** *(FR40, `Matcher` của AD-17)*, không phải truy vấn chuỗi con ký tự. Cần một AD nêu rõ chiến lược truy vấn theo ngôn ngữ, và nêu rõ **ranh giới**: chỗ nào là "mã riêng cho từng ngôn ngữ" **được phép** *(chiến lược truy vấn)* và chỗ nào **cấm** *(cổng `DictionarySource`, AD-10)*.
  🔄 **CẬP NHẬT 2026-08-05 sau Story 1.10b — con số nay là ĐO ĐƯỢC trên `dict-core.db` thật, không còn là mũi thăm dò:** nguồn `viwiktionary-en` đóng góp đúng **9** cặp `char_idx` trên **119.039** đầu mục = **0,0076%**. `char_idx` tổng của tệp là **1.341.179**, tức lớp tiếng Anh chiếm **0,00067%** của chỉ mục đó. ⇒ **`AD-26` nhánh 2 KHÔNG áp được cho tiếng Anh** — đây là dữ kiện, không phải suy đoán. *(SQL tái lập: `SELECT COUNT(*) FROM char_idx c JOIN dict_entry e ON e.id=c.entry_id JOIN dict_source s ON s.id=e.source_id WHERE s.code='viwiktionary-en';`)*
- 🔴 **Đường tra cứu PHẢI lọc theo `dict_entry.lang` — KHÔNG được giả định mọi hàng là `zh`. Chủ sở hữu: Story 1.11b và 1.13.** **(Chủ: Story 1.11b và 1.13.)** *(MỚI — Story 1.10b, 2026-08-05.)*
  Cho tới trước story này, `dict-core.db` có **473.499 đầu mục, 100% `lang='zh'`**, nên một đường đọc không lọc `lang` vẫn cho kết quả đúng **do may mắn**. Điều đó **đã hết đúng**: tệp nay có **119.039 hàng `lang='en'`** *(20,1% tổng số 592.538 đầu mục)*.
  **Hậu quả cụ thể nếu bỏ sót:** đường tra cứu tiếng Trung của Story 1.11 tra một chữ Hán sẽ nhận về `dictionary`, `lock`, `API`, `Wikipedia` — vì `entry_fts` *(trigram trên headword)* lập chỉ mục **cả** đầu mục tiếng Anh, đó là hành vi sẵn có không phải quyết định của Story 1.10b.
  Phân bố thật để đối chiếu: `cvdict|zh|122.596` · `cc-cedict|zh|124.758` · `unihan|zh|49.870` · `viwiktionary|zh|1.598` · `en-wiktionary|zh|174.677` · **`viwiktionary-en|en|119.039`**.
- 🟡 **Kiểm Panel Lookup có hình dạng hiển thị cho mục từ TIẾNG ANH chưa — chủ sở hữu: Sally (`bmad-ux`).** `EXPERIENCE.md` và `DESIGN.md` dựng quanh mục từ tiếng Trung *(tab Hán Việt, âm đọc, bộ thủ)*. Mục tiếng Anh có hình dạng khác: từ loại + nghĩa tiếng Việt + ví dụ, không có Hán Việt, không có tab chữ Hán. Nếu Panel Lookup chưa có biến thể cho hình dạng đó, Story 1.11b sẽ phải tự chế — và tự chế ở tầng story là đúng cách một bất nhất giao diện ra đời.
  → ⚠️ **KHÔNG ĐÓNG — Ice chốt 2026-08-06 (Story 1.17): TẠM dùng mặc định của story, không phải chữ ký UX.** `LookupRecord.vue` dùng **cùng một cấu trúc khối** cho mục tiếng Anh và tiếng Trung — chỉ khác token đầu mục (`lookup-headword` họ `read`, không chữ Hán 34px của mockup). Đây là một lựa chọn **tự chế ở tầng story**, đúng thứ mục này cảnh báo là "cách một bất nhất giao diện ra đời" — chủ sở hữu **vẫn là Sally**, mục này **vẫn mở**.
- 🟡 **Kiểm SPEC có bản sao FR34 cần đồng bộ không** — `bmad-spec`. FR34 nay có hai story mang nó *(1.10b dữ liệu · 1.11b tra cứu)*.

## Deferred from: code review of 1-10b-dung-du-lieu-tu-dien-tieng-anh (2026-08-05)

- 🟡 **Commit `dd7af61` gộp mã story 1.10b với sửa đổi tài liệu quy hoạch (`prd.md`, `epics.md`, `ARCHITECTURE-SPINE.md`, `sprint-change-proposal-2026-08-05.md` mới) vào chung MỘT commit** — File List xác nhận các tệp này "đã bị sửa TRƯỚC khi story bắt đầu" (thuộc gói `correct-course` Ice đã duyệt cùng ngày), dev không đụng nội dung, nhưng khi commit tất cả bị gộp chung, mất tính nguyên tử. **Ice chốt 2026-08-05: giữ nguyên, không tách commit** — chưa push lên `origin/master`, không ai khác đụng vào giữa chừng, tách lúc này chỉ thêm rủi ro thao tác git mà không đổi nội dung. Quy ước cho lần sau: commit riêng tài liệu quy hoạch trước khi bắt đầu code của story.
- 🟡 **`vi-extract.jsonl` (273 MB) bị đọc và parse lại từ đầu HAI LẦN** (`tools/dict-build/src/build.rs:253-283`), một lần mỗi vai (`viwiktionary` vai B, `viwiktionary-en` vai A), thay vì một lượt đọc duy nhất với hai bộ tích luỹ song song. Đánh đổi có chủ ý và có lý do ghi rõ trong code: gộp một lượt `parse()` sẽ hợp nhất headword xuyên nguồn, đúng thứ AD-19 cấm. Nhưng chưa đo chi phí thời gian build tăng thêm, và không có bước đối chiếu nội dung/hash giữa hai lần `File::open` nếu tệp thô đổi giữa chừng (rủi ro thấp cho một lượt build cục bộ, đơn tiến trình).
- 🟡 **Ngưỡng định lượng AC2 không có cưỡng chế tự động** — con số 119.039 đầu mục / 190.543 nghĩa / 27.396 ví dụ (lệch ≤1%) chỉ được đối chiếu một lần thủ công ở một mũi thăm dò đã bị revert khỏi cây mã. Test CI đóng gói (`tools/dict-build/tests/parse.rs`) chỉ khẳng định `count > 0` cộng hai headword mẫu cố định (`dictionary`, `lock`). Một hồi quy tương lai làm rơi một phần lớn đầu mục tiếng Anh thật (ví dụ đổi tham số lọc sai) sẽ không bị bắt tự động.
- 🟡 **`epics.md` — lệch tài liệu/mã thứ TƯ, chưa nằm trong "ba lệch" của §Completion Notes ②** — mục Story 1.10b vẫn ghi quyết định base-vs-detachable là "🔴 Quyết định phải chốt TRONG story", dù chính Completion Notes của story này đã đánh dấu AC5 **ĐẠT**. Theo đúng tiền lệ (dev không sửa `epics.md`), nên bổ sung vào danh sách lệch cho Ice cập nhật.
- 🟡 **Dual-license (CC-BY-SA-4.0 + GFDL-1.3) chỉ biểu diễn được bằng MỘT trường `license_id`** (`tools/dict-build/src/sources_meta.rs`, `LicenseRef::CcBySaAndGfdl` gộp cả hai vào một enum) — nguyên trạng kế thừa từ `viwiktionary`/`en_wiktionary` ở Story 1.9; `viwiktionary-en` chỉ tái dùng cùng khuôn mẫu. Story này bị cấm sửa `schema.rs` nên không vá ở đây.

## Deferred from: 1-11-ba-nhanh-truy-van-tieng-trung (2026-08-05)

- 🔵 **ĐÍNH CHÍNH mệnh đề "tra một chữ Hán sẽ nhận về `dictionary`, `lock`, `API`, `Wikipedia`" (mục `Đường tra cứu PHẢI lọc theo dict_entry.lang`, ở trên).** Mệnh đề đó **SAI với truy vấn thuần Hán**, và đây là số ĐO ĐƯỢC trên `dict-core.db` thật *(194.998.272 byte, 2026-08-05)*, không phải suy luận:
  - `entry_fts MATCH '"中國人"'` ⇒ **33** hàng, giao với `lang='en'` ⇒ **0**. Trigram Latin không khớp trigram Hán.
  - `char_idx` thuộc lớp tiếng Anh: **9** cặp trên tổng **1.341.179** *(0,00067%)*.
  ⇒ Rò rỉ với truy vấn **thuần Hán** đo được là **0**.
  **NHƯNG mệnh lệnh lọc `lang` KHÔNG đổi**, vì rò rỉ là **thật và lớn** với truy vấn **Latin** — chuyện thường khi người dùng bôi đen một chữ Latin trong văn bản tiếng Trung:
  - `headword = 'lock'` ⇒ **1** hàng không lọc → **0** khi lọc `lang='zh'`
  - `entry_fts MATCH '"dic"'` ⇒ **572** hàng, **100%** `lang='en'` → **0** khi lọc
  ⇒ Story 1.11 lọc `lang='zh'` ở **cả ba** nhánh, và `tests/dict_lookup.rs::every_branch_filters_out_english_entries` cưỡng chế bằng đúng hai truy vấn Latin đó. **Trạng thái: ĐÃ ĐÓNG cho 1.11; vẫn mở cho 1.11b và 1.13. **(Chủ: Story 1.11b / 1.13.)**

- 🟡 **VietPhrase: 18 đầu mục trùng — VẪN MỞ, chủ sở hữu là 1.13, không phải 1.11.** Story 1.11 chạy trên **MỘT** tệp `.db` mỗi lượt và AD-19 cấm hợp nhất nguồn, nên nó không gộp trùng và không được phép gộp. Hậu quả *"UI hiện hai khối VietPhrase giống hệt nhau"* chỉ **quan sát được** khi gom nhiều nguồn — tức nó là quyết định của **1.13**, và quyết định đó phải chọn giữa *"gộp lúc đọc"* và *"quyết lại mô hình lúc dựng"*.

- 🔵 **Khoá theo `code` chứ không theo `id` — ĐÃ ĐÓNG MỘT NỬA.** `EntryHit` của 1.11 mang `source_code: String`, không có trường `source_id` nào, và `tests/dict_lookup.rs::results_carry_the_source_code_not_the_id` khoá mệnh đề đó trên một fixture hai nguồn. **Nửa còn lại là của 1.13:** lúc gom nhiều tệp, khoá gom **phải** là `code`, và không được phép dựng lại một bảng tra `id → nguồn` ở tầng gom. **(Chủ: Story 1.13.)**

- 🔴 **NFR1: nhánh 2 với truy vấn MỘT ký tự đã VƯỢT dải công bố của AD-26, và trần 10 ms chỉ còn 27% dư địa. Chủ sở hữu: Ice quyết, ứng viên là 1.13/1.17.** Đo thật trên `dict-core.db`, 200 lượt, bỏ 10 lượt làm nóng:
  | Nhánh | AD-26 công bố | Trần story | p95 **release** | p95 **debug** |
  |---|---|---:|---:|---:|
  | 1 — B-tree chính xác (`山`) | 0,02 ms | 1 ms | **0,083 ms** ĐẠT | 0,133 ms |
  | 2 — `char_idx` **1 ký tự** (`山`, 3.177 hàng) | 0,15–4,5 ms | 10 ms | 🔴 **7,324 ms** ĐẠT *(sát)* | 🔴 **15,045 ms** VƯỢT |
  | 2 — `char_idx` **2 ký tự** (`中國`, 350 hàng) | 0,15–4,5 ms | 10 ms | **1,039 ms** ĐẠT | 2,566 ms |
  | 3 — FTS5 trigram (`中國人`, 33 hàng) | 0,13–0,19 ms | 1 ms | **0,448 ms** ĐẠT | 0,768 ms |
  **Phán quyết:** ĐẠT trên bản **release** — đó là bản người dùng chạy, nên đó là số nghiệm thu. Nhưng ba dữ kiện phải đi cùng nó:
  1. **7,324 ms vượt hẳn dải 0,15–4,5 ms mà AD-26 công bố.** Số của AD-26 đo ở Giai đoạn 0 trên một database **ba** nguồn; `dict-core.db` hôm nay có **sáu**, và `char_idx` của `山` đi từ ~2.576 lên **3.177** hàng. Dải công bố của AD-26 nên được đo lại, không nên được trích tiếp như số hiện hành.
  2. **Chi phí nằm ở số HÀNG, không ở chỉ mục.** Kế hoạch truy vấn đúng (`SEARCH char_idx USING PRIMARY KEY`); 3.177 hàng × 4 chuỗi cấp phát mỗi hàng là toàn bộ chi phí. Nên nó không sửa được bằng một chỉ mục mới, và story này bị cấm thêm chỉ mục — đúng lý do.
  3. **Đường ra là một quyết định sản phẩm, không phải một lượt tối ưu:** giới hạn số hàng trả về *(phân trang / `LIMIT` + đếm)* là hình dạng tự nhiên nhất, nhưng nó chạm hợp đồng của Panel Lookup **(1.17)** và tầng gom **(1.13)** — cả hai đều chưa tồn tại. Không tự chọn ở 1.11.
  ⚠️ Bản **debug** VƯỢT trần *(15,045 ms)*. Không phải số nghiệm thu, nhưng nó là số mà mọi dev chạy `cargo test` sẽ thấy — ghi ra để lượt sau không đọc nó thành một hồi quy.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.17, Quyết định #4).** Ice chốt: có `LIMIT` pha một. Đo lại trên bốn lớp thật (`--release`): char_idx 1 ký tự p95 **20,836 ms** (không `LIMIT`) → **5,109 ms** (`LIMIT 20`) — đường sản phẩm thật (`commands::dict::lookup`, `LookupMode::Exact` cố định) đo p95 **6,535 ms** cho ca xấu nhất. 🔴 **Phát hiện mới**: `Exact` luôn đi nhánh `ExactBtree`, không bao giờ đi `char_idx` — nhánh đắt của mục này **không** được đường sản phẩm 1.17 tự nó chạm tới; `LIMIT` vẫn giữ giá trị cho FR31 (AC12) trên `ExactBtree` và cho ngày `Substring` được dùng (1.18/7.7). Xem story `1-17-panel-lookup-ban-ghi-co-cau-truc.md` §Debug Log References.

- 🟡 **`EXPLAIN QUERY PLAN` của nhánh 3 chứa chữ `SCAN`, và đó là hành vi ĐÚNG.** Story yêu cầu *"phải thấy `VIRTUAL TABLE INDEX`, không thấy `SCAN`"*. Kế hoạch thật là `SCAN f VIRTUAL TABLE INDEX 0:M1` — SQLite luôn dùng từ `SCAN` cho một bảng ảo; phần mang nghĩa là hậu tố **`:M1`**, nó nói ràng buộc `MATCH` **đã** được đẩy xuống mô-đun FTS5. Một kế hoạch hỏng sẽ là `VIRTUAL TABLE INDEX 0:` **không** có `M`. Nêu ra để lượt rà sau không đọc chữ `SCAN` thành một vi phạm.

## Deferred from: code review of 1-11-ba-nhanh-truy-van-tieng-trung (2026-08-05)

- **NFR1 nhánh 2 (1 ký tự) còn 27% dư địa tới trần — quyết định lúc review: chấp nhận nguyên trạng.** Xác nhận lại mục đã có ở trên (§Deferred from: 1-11-…, mục NFR1): người dùng, khi review, chọn không sửa gì bây giờ và để 1.13/1.17 xử lý bằng phân trang thật khi Panel Lookup tồn tại. **(Chủ: một story kế tiếp chạm `core/dict`.)**
- **Kế hoạch truy vấn của nhánh 1/2 chỉ được xác nhận bằng `EXPLAIN QUERY PLAN` chạy tay, không có cổng CI tự động** (`src-tauri/src/core/dict/query.rs`) — nếu một di trú lược đồ tương lai xoá `idx_entry_headword`/`idx_entry_headword_simp` hay đổi khoá chính của `char_idx`, nhánh 1/2 có thể âm thầm suy biến thành quét toàn bảng mà không test hành vi nào đỏ. Đây là cùng ràng buộc mà AC9 đã chấp nhận (không có tệp `.db` thật trong CI), nên không tự động hoá được trong story này. **(Chủ: một story hạ tầng cổng kế tiếp.)**
- **Nhánh `char_idx` 1 ký tự bỏ qua xác minh chuỗi con ở Rust, dựa hoàn toàn vào bất biến của `tools/dict-build`** (`src-tauri/src/core/dict/query.rs:121`) — đúng của tối ưu này phụ thuộc việc `char_idx` không bao giờ sinh một cặp `(ký tự, entry_id)` sai, một bất biến chỉ được cưỡng chế ở workspace `tools/dict-build`, không có cổng nào kiểm chéo hai workspace. Ranh giới hai workspace tách rời đã chốt từ Story 1.9 (AC4); story này kế thừa quyết định đó chứ không tạo ra nó. **(Chủ: Story 1.11b / 1.13.)**
- **Không có giới hạn trên cho độ dài truy vấn** trước khi đưa vào `chars()`, cấp phát chuỗi lặp lại, và dựng cụm FTS (`src-tauri/src/core/dict/query.rs`) — thật với một đầu vào cực dài, nhưng story 1.11 tường minh cấm dựng IPC command hay chạm frontend nên chưa có bên gọi không tin cậy nào tồn tại. Validate độ dài đầu vào thuộc về tầng IPC/UI của Story 1.13/1.17.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.17, Task 2).** `commands::dict::lookup()` cắt truy vấn ở `QUERY_LENGTH_CEILING = 200` ký tự TRƯỚC khi vào đường tra — một sàn TRÊN có tên, không một `panic`. Ca test `a_query_past_the_length_ceiling_is_truncated_before_it_reaches_the_lookup` chứng minh việc cắt xảy ra trước `pick_route` (201 ký tự Latin+Hán ⇒ route `En`, không `Zh`).

## Deferred from: code review of 1-11b-duong-tra-cuu-tieng-anh (2026-08-05)

- **Điều kiện `≤ 2 ký tự` của `char_idx()` chỉ cưỡng chế bằng `debug_assert!`** (`src-tauri/src/core/dict/query.rs:138`) — vô tác dụng ở bản release; một lượt gọi trực tiếp trong tương lai (vd. từ tầng gom Story 1.13, nếu nó bỏ qua `lookup()` và gọi thẳng `query::char_idx`) với truy vấn dài hơn sẽ âm thầm cắt còn hai ký tự đầu thay vì báo lỗi. Kế thừa từ Story 1.11 — story 1.11b không chạm hàm này, chỉ mở rộng phạm vi tiếp xúc của nó qua module dùng chung. **(Chủ: một story kế tiếp chạm `core/dict`.)**
- **Ngưỡng độ dài nhánh (`chars().count()`) đếm code point Unicode, không đếm cụm ký tự hiển thị (grapheme cluster)** (`src-tauri/src/core/dict/mod.rs:242`) — văn bản chuẩn hoá NFD (vd. clipboard macOS với dấu tổ hợp) có thể đẩy một truy vấn qua lằn ranh nhánh (`CharIdx`/`FtsTrigram` ở đường zh, `NoBranchQueryTooShort`/`FtsTrigram` ở đường en) sai lệch so với số ký tự người dùng cảm nhận đã gõ. Phép đo kế thừa nguyên xi từ ngưỡng zh của Story 1.11; story 1.11b áp dụng cùng cách đo cho ngưỡng en mới chứ không tạo ra vấn đề. Sửa đòi một quyết định chuẩn hoá Unicode chung cho cả hai đường — thuộc tầng kiến trúc, không phải một lượt vá cục bộ. **(Chủ: một story kế tiếp chạm `core/dict`.)**

## Deferred from: 1-12-matcher-dung-chung (2026-08-05)

- 🔵 **Sơ đồ mermaid của AD-13 còn cạnh `dict --> matching` — LỆCH khỏi thân Rule của AD-17. Chủ sở hữu: Winston (architect).** **(Chủ: Winston — architect.)** `ARCHITECTURE-SPINE.md:189` vẽ một cạnh phụ thuộc từ `dict` sang `matching`. Sơ đồ đó vẽ **trước** lượt sửa Rule của AD-17 ngày 2026-08-05, và nay mâu thuẫn với chính **thân Rule** ở `:236`: *"AD này nói mọi nơi cần khớp ngôn ngữ dùng chung MỘT cài đặt — nó KHÔNG nói mọi đường đều phải gọi Matcher. Đường tra cứu **từ điển** tiếng Anh không gọi."* **Không chặn Story 1.12** *(dev theo thân Rule, và Story 1.11b đã giao xong đường tra cứu tiếng Anh mà không gọi Matcher một lần nào)*, nhưng nó sẽ làm lệch **mọi lượt đọc kiến trúc sau** — sơ đồ là thứ người ta đọc trước, thân Rule là thứ người ta đọc sau. Mệnh đề đúng nay được cưỡng chế bằng cổng `tests/matching_boundary.rs::the_dictionary_lookup_path_never_calls_the_matcher`, kèm thông báo assert nêu đích danh AD-17 `:236` và AD-44 ③.

- 🟡 **`epics.md:1510` còn vế *"`dict/` dùng nó"* — chủ sở hữu: John (PM).** Mục Story 1.12 vẫn viết *"**And** `dict/` **dùng nó**; `glossary/` và `tm/` sẽ dùng chính nó ở các epic sau"*. Vế đầu đã bị AD-17 lật. Cùng lượt sửa với `:1491` *(mục 1.11b)* mà mục *"`epics.md` ĐANG LỆCH khỏi AD-44 ở hai chỗ"* ở trên đã ghi. Story 1.12 giao `core/matching/` cho Glossary và TM và **cưỡng chế bằng cổng** rằng `core/dict/**` không gọi nó — nên nếu vế cũ được ai đó thi hành, cổng sẽ đỏ **có tên** thay vì hồi quy im lặng.

- 🔵 **AD-44 ③: bảng phỏng đoán Porter NAY CÓ SỐ ĐO THẬT — chủ sở hữu: Winston (architect), dev không sửa `ARCHITECTURE-SPINE.md`.** **(Chủ: Winston — architect.)** `:616` ghi ⚠️ *"ba chuỗi stem đó lấy từ hành vi kinh điển của Porter chứ **chưa chạy qua stemmer mà sản phẩm sẽ dùng** […] ai muốn mở lại câu hỏi stemming thì việc đầu tiên là **chạy stemmer thật và thay bảng này bằng số đo**"*. Story 1.12 là lượt đầu tiên có stemmer thật trong cây mã, và đã chạy `tantivy_stemmers::algorithms::english_porter_2` trên đúng bốn chuỗi AD-44 nêu:

  | Đầu vào | AD-44 ③ phỏng đoán | **Đo thật** | Trùng? |
  |---|---|---|---|
  | `dictionary` | `dictionari` | `dictionari` | ✅ |
  | `study` | `studi` | `studi` | ✅ |
  | `happy` | `happi` | `happi` | ✅ |
  | `run` | *(1 hàng, không nêu chuỗi)* | `run` | ✅ |

  ⇒ **Phỏng đoán của AD-44 ③ ĐÚNG 3/3 chuỗi nó nêu.** Kết luận của ③ *(stemming không nằm trên đường nóng tra từ điển)* **không** bị lật bởi số đo — nó **được củng cố**: ba chuỗi stem đó đúng là thứ sẽ được tra, và chúng đúng là cho **0** hàng trên `dict-core.db`. Món nợ đo đạc ở `:616` nay **đóng được**; ⚠️ nội dung sửa là của Winston.

- 🔴 **`Jieba` khởi tạo tốn ~180–330 ms bản RELEASE — VƯỢT NFR2 (50 ms) từ 3,6× đến 6,6×. Chủ sở hữu: Story 3.4** **(Chủ: Story 3.4.)** *(story đầu tiên gọi Matcher trên một đường gõ thật)*. Đo thật, `[profile.release]` không đổi một dòng, 6 lượt chạy trên máy dev *(macOS, darwin 24.6.0)*, mỗi lượt một tiến trình mới:

  | Lượt | 1 | 2 | 3 | 4 | 5 | 6 |
  |---|---:|---:|---:|---:|---:|---:|
  | Khởi tạo lạnh **(ms)** | 328,588 | 244,444 | 224,407 | 179,161 | 242,224 | 255,437 |

  Trung vị **~243 ms**, thấp nhất **179 ms**, cao nhất **329 ms**. Lượt gọi **ấm** kế tiếp: **1 µs** *(dưới ngưỡng đo)*. Chi phí là giải nén `dict.txt` *(**5.071.843 byte** thô, nhúng qua `include_flate::flate!`)* cộng nạp từng dòng vào một cây `cedar` — công việc **chạy lúc chạy**, không phải một hằng số biên dịch, và nó rơi vào **lần gọi đầu tiên**, tức có thể rơi đúng vào phím đầu tiên người dùng gõ.
  **Đường ra là hâm nóng `LazyLock` NGOÀI đường gõ** *(một lượt `tokenize` giả lúc mở Tác phẩm, hoặc trên một luồng nền lúc khởi động)*. **Story 1.12 cố ý KHÔNG dựng cơ chế hâm nóng** — chưa có đường gõ nào tồn tại để hâm nóng vào, và một cơ chế dựng trước người tiêu thụ là một phỏng đoán về chỗ gọi. Cổng `tests/matching_boundary.rs::the_jieba_dictionary_is_constructed_at_exactly_one_place` + `…::the_single_jieba_instance_is_actually_lazily_initialised_once` giữ cho chi phí này không nhân lên khi ai đó chuyển lời gọi vào thân một hàm bị gọi lặp.

- 🔵 **PHÁT HIỆN MỚI: Porter2 KHÔNG có luật cho hậu tố so sánh/cực cấp (`-er` · `-est`) — `happiest` không về được `happy`.** AC7 của story liệt kê `happiest` là một *"biến thể hình thái"* mà Matcher phải nhận diện được về dạng gốc. **Đo thật lật vế đó:** `happiest` ⇒ `happiest`, trong khi `happy` ⇒ `happi` — hai vế **không** gặp nhau. Ba biến thể còn lại của AC7 thì đạt *(`running`⇒`run` · `dogs`⇒`dog` · `studies`⇒`studi`=`study`)*. Đây **không** phải lỗi cài đặt: Porter2 theo định nghĩa không xử lý `-er`/`-est`, nên một biến thể **có quy tắc** cũng rơi vào đúng giới hạn mà FR40 đã tuyên bố cho dạng **bất quy tắc**. Story đóng nó bằng cách đưa `happiest` vào ca test giới hạn có tên *(`stemming_is_not_lemmatization_irregular_and_comparative_forms_never_reach_their_lemma`)* thay vì vào ca AC7. ⚠️ **Hệ quả cần biết cho Epic 3:** một người dịch thêm thuật ngữ `happy` vào Glossary sẽ **không** thấy `happiest` được tô màu. Nếu đó là mức phủ không chấp nhận được thì đường ra là một lemmatizer — và NFR15 đòi rà giấy phép **trước** khi thêm phụ thuộc. Chủ sở hữu quyết định: **Ice / John (PM)**, ứng viên là Story 3.4. **(Chủ: Story 3.4.)**

- 🟡 **`find_terms` là O(số thuật ngữ × độ dài văn bản) — không chỉ mục ngược, không cache. Chủ sở hữu: Story 3.4 / 7.5.** Story 1.12 cố ý không dựng cả hai *(§Ranh giới phạm vi: chúng thuộc 7.5/7.6 và phụ thuộc dữ liệu thật)*. Với một Glossary vài trăm thuật ngữ trên một segment vài trăm ký tự thì hình dạng hiện tại thừa đủ; với một Glossary vài nghìn thuật ngữ trên **cả chương**, nó cần đo lại trước khi đặt lên đường gõ *(NFR2 = 50 ms mỗi frame)*. **Chưa đo** — chưa có người tiêu thụ nào để đo trên đó, và một con số đo trên đầu vào tự bịa là một con số không dùng được.

## Deferred from: code review of 1-12-matcher-dung-chung (2026-08-05)

- **Các cổng ranh giới trong `matching_boundary.rs` (và `dict_boundary.rs`/`store_boundary.rs` trước đó) là phép quét CHỮ trên mã nguồn, không phải phân tích đồ thị gọi hàm ngữ nghĩa** — một lớp bọc re-export dưới tên khác (vd. `pub use matching::find_terms as glossary_probe;` đặt trong `core/mod.rs`) có thể để `core/dict/**` gọi vào Matcher mà không chạm bất kỳ token cấm nào (`matching`/`jieba`/`stemmer`/`stem(`). Đây là giới hạn có sẵn của cả khuôn "cổng quét chữ" dùng xuyên dự án từ Story 1.9, không phải do Story 1.12 gây ra hay có thể sửa cục bộ trong một story — sửa đòi thiết kế lại triết lý cổng ranh giới trên toàn dự án. **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**
- **Không có test nào cưỡng chế lời hứa "không chạm filesystem/database/mạng" (AD-15) mà doc-comment của `core/matching/mod.rs` tuyên bố** — đúng hôm nay qua rà tay thủ công (không có lời gọi I/O nào trong mã story 1.12), nhưng không gì bắt được nếu một lượt sửa tương lai âm thầm thêm I/O vào module lá này. Khuôn cổng ranh giới hiện tại (`matching_boundary.rs`, `dict_boundary.rs`, `store_boundary.rs`) chưa có tiền lệ kiểm loại forbidden-token này cho `fs`/`net`/`rusqlite`. **(Chủ: một story kế tiếp chạm `core/matching`.)**
- **Một số con số "đo được" gắn cứng trong doc-comment của `core/matching/mod.rs` và trong mục review trước ở tệp này (`dict.txt` = 5.071.843 byte thô; khởi tạo `Jieba` 179–329 ms bản release) không được một test nào khẳng định** — sẽ lặng lẽ lạc hậu khi phiên bản `jieba-rs` hoặc dữ liệu dict đổi, vì không cổng nào đỏ khi điều đó xảy ra. Rủi ro tài liệu, không phải rủi ro đúng/sai của mã. **(Chủ: một story kế tiếp chạm `core/matching`.)**
- **`ngrams` và `find_terms` mỗi hàm tự tokenize/normalize lại toàn bộ văn bản đầu vào — không có bề mặt API nào để tái dùng token đã tính giữa hai lời gọi trên cùng một đoạn văn bản.** Một người tiêu thụ tương lai (Story 7.6) cần cả n-gram lẫn tìm thuật ngữ trên cùng một segment sẽ trả giá tokenize/normalize hai lần. Chủ sở hữu quyết định hình dạng API: Story 3.4/7.6, khi có người tiêu thụ thật. **(Chủ: Story 3.4/7.6.)**
- **Văn bản chuẩn hoá NFD (dấu tổ hợp, vd. một số nguồn clipboard macOS) tokenize/stem khác với văn bản NFC cùng nội dung ở đường `En` của `core/matching/`** (`char::is_alphanumeric` trong `tokenize` và `to_lowercase` trong `normalize` đều không chuẩn hoá NFC/NFD trước khi xử lý) — cùng lớp vấn đề chuẩn hoá Unicode đã ghi nhận cho một module khác ở mục *§Deferred from: code review of 1-11b-duong-tra-cuu-tieng-anh* phía trên (`core/dict/mod.rs:242`, kế thừa từ Story 1.11). Sửa đòi một quyết định chuẩn hoá Unicode chung cho toàn dự án — thuộc tầng kiến trúc, không phải một lượt vá cục bộ ở module này. **(Chủ: một story kế tiếp chạm `core/matching`.)**

## Deferred from: 1-13-duong-tra-cuu-giu-nguyen-bat-dong-giua-cac-nguon (2026-08-05)

- ✅ **FR36 nghiệm thu HÀNH VI — ĐÓNG. Món nợ mở từ Story 1.10 kết thúc ở đây.** Mục 🟡 *"FR36 nghiệm thu HÀNH VI"* ở §Deferred from 1-10 *(dòng ghi *"Không đánh dấu FR36 là 'đã nghiệm thu' cho tới khi 1.13 viết phép thử này"*)* nay đã có phép thử: `tests/dict_sources.rs::deleting_any_detachable_layer_keeps_the_whole_lookup_suite_green`. Nó **xoá tệp `.db` thật** rồi **mở lại tập lớp**, và chạy **cùng một** hàm mệnh đề (`the_layer_independent_lookups_still_hold`) trước và sau — không một nhánh `#[cfg]`, không một ca nào phải sửa. Danh sách lớp gỡ rời **dẫn xuất từ chính tập lớp** chứ không viết cứng, nên *"một lớp gỡ rời **bất kỳ**"* (`epics.md:1572`) được nghiệm thu đúng nghĩa. Kèm **đối chứng dương**: trước khi xoá, ca khẳng định lớp đó **thật sự** đóng góp một nhóm — không có vế đó thì *"xoá xong vẫn xanh"* và *"lớp đó chưa bao giờ được nạp"* đọc giống hệt nhau.
  ⚠️ **Vế còn mở, và nó không phải FR36:** phép thử chạy trên **fixture ba tệp** do test dựng, không trên ba tệp `.db` thật *(195 MB — `.gitignore: *.db`, AD-25; CI không có tệp nào)*. Đó là cùng ràng buộc mọi story từ 1.9 tới nay đã chấp nhận có tên, không phải một chỗ bỏ sót của story này.

- 🟡 **VietPhrase 18 đầu mục trùng — PHÁN QUYẾT: ĐỂ NGUYÊN. Bàn giao trình bày cho Story 1.17.** Mục 🟡 *"VietPhrase: 18 đầu mục trùng — VẪN MỞ, chủ sở hữu là 1.13"* ở §Deferred from 1-11 nay **đã được quyết** (Story 1.13 §Quyết định #2, phương án **A**):
  - **Dữ liệu giữ đúng như nguồn ghi.** Tra `不是他的对手` vẫn trả **HAI** `dict_entry` từ **cùng** nguồn `vietphrase`, và tầng gom đưa cả hai vào **một** nhóm `vietphrase` *(khoá gom là `code`, nên chúng không thành hai khối)*.
  - **Vì sao không gộp lúc đọc:** AD-19 cấm hợp nhất **GIỮA** các nguồn; đây là trùng **TRONG** một nguồn — một câu hỏi khác. Nhưng một hàm gộp đặt ở đúng module mà AC6 cấm có hàm gộp buộc cổng `no_function_merges_meanings_across_sources` phải mang **ngoại lệ đầu tiên**, và một cổng có ngoại lệ thứ nhất sẽ có ngoại lệ thứ hai. Cái giá của việc để nguyên là một dòng trình bày ở 1.17; cái giá của việc gộp là một cổng kiến trúc bị thủng.
  - 🔴 **Bàn giao cho Story 1.17:** một nhóm nguồn có thể chứa **nhiều đầu mục cùng `headword`**. Panel Lookup phải trình bày được ca đó *(gộp hiển thị, hoặc đánh số, hoặc hiện liền nhau)* mà **không** đổi dữ liệu. Con số hôm nay: **18** trong `dict-vietphrase.db` *(46 trong nguồn thô)*.
  - Phương án C *(quyết lại mô hình lúc dựng)* vẫn ngoài phạm vi mọi story cho tới khi ai đó chấp nhận dựng lại `dict-vietphrase.db`, điền lại `sha256`, và đo lại NFR6.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.17, Quyết định #5).** Nhìn bằng mắt 18 hàng trùng — CHỐT đường **(a) hiện LIỀN NHAU**, không đánh số, không gộp: `senses` truyền vào `LookupRecord.vue` là danh sách PHẲNG (mọi nghĩa của mọi đầu mục trong nguồn, đúng thứ tự `entry_id → ord → sense_id`), nên component không cần biết ranh giới đầu mục — dữ liệu không đổi, đúng ràng buộc.

- 🔴 **NFR1 trên ĐƯỜNG GOM: nhánh 2 một ký tự VƯỢT trần 10 ms — 12,569 ms bản release. Chủ sở hữu: Ice quyết, ứng viên là 1.17.** Đo thật 2026-08-05 trên **ba tệp `.db` thật** *(`dict-core.db` 194.998.272 · `dict-thieu-chuu.db` 5.787.648 · `dict-vietphrase.db` 160.083.968 byte)*, bản **release**, 200 lượt, bỏ 10 lượt làm nóng, qua `tests/dict_sources.rs::bench_the_grouped_path_on_the_real_dictionaries`:

  **Pha một — `lookup_grouped` trên 3 lớp:**

  | Nhánh | Truy vấn | Nhóm | Hàng | p50 | p95 | p99 | Trần | |
  |---|---|---:|---:|---:|---:|---:|---:|---|
  | 1 — B-tree chính xác | `山` | 6 | 8 | 0,196 | **0,223** | 0,268 | 1 ms | ĐẠT |
  | 2 — `char_idx` 1 ký tự | `山` | 7 | **6.563** | 11,059 | 🔴 **12,569** | 13,368 | 10 ms | **VƯỢT** |
  | 2 — `char_idx` 2 ký tự | `中國` | 5 | 354 | 2,423 | **3,608** | 4,318 | 10 ms | ĐẠT |
  | 3 — FTS5 trigram | `中國人` | 4 | 35 | 0,454 | **0,563** | 0,644 | 1 ms | ĐẠT |
  | en-1 B-tree (thường) | `running` | 1 | 1 | 0,133 | **0,144** | 0,320 | 1 ms | ĐẠT |
  | en-1 B-tree (HOA) | `Running` | 1 | 1 | 0,144 | **0,239** | 0,323 | 1 ms | ĐẠT |
  | en-2 trigram | `dic` | 1 | 572 | 1,057 | **1,575** | 1,842 | 10 ms | ĐẠT |

  **Pha hai — `senses()` theo lô (`SENSE_BATCH = 64`):**

  | Ca | Lớp | Đầu mục | p50 | p95 | p99 |
  |---|---|---:|---:|---:|---:|
  | `山` substring — **một trang** | `vietphrase` | 20 | 0,151 | **0,294** | 0,369 |
  | `山` substring — **tất cả** | `vietphrase` | 3.385 | 12,244 | 🔴 **13,015** | 13,380 |
  | `中國` substring — một trang | `base` | 20 | 0,223 | **0,299** | 0,352 |
  | `中國` substring — tất cả | `base` | 147 | 0,748 | **0,972** | 1,309 |
  | `dic` substring — một trang | `base` | 20 | 0,229 | **0,315** | 0,422 |
  | `dic` substring — tất cả | `base` | 572 | 3,732 | **5,110** | 5,785 |

  **Bốn dữ kiện phải đi cùng bảng trên:**
  1. **Chi phí nằm ở SỐ HÀNG, không ở số tệp.** Story 1.11 đo `山` trên **một** tệp: 3.177 hàng, 7,324 ms. Nay **6.563** hàng qua ba tệp cho 12,569 ms — **2,07× hàng ⇒ 1,72× thời gian**, tức gần tuyến tính theo hàng. Gom nhiều tệp **không** thêm một chi phí cố định đáng kể; nó chỉ cộng thêm hàng. ⇒ Đường ra **không** phải "mở ít tệp hơn".
  2. 🔴 **§Quyết định #1B được số đo XÁC NHẬN, không phải chỉ được lý luận.** Một trang 20 đầu mục hydrate hết **0,29–0,32 ms** ở **mọi** ca — thừa sức trong ngân sách. Hydrate **cả** 3.385 đầu mục hết **13,015 ms**. Đó **chính xác** là chi phí mà phương án A *(một pha)* sẽ buộc phải trả **bên trong** `lookup_grouped`, **cộng dồn cho cả ba tệp** — tức pha một sẽ là ~12,6 + ~13,0 + … ms thay vì 12,6 ms. Hai pha đẩy đúng chi phí đó sang chỗ **quyết định được**, và chỗ đó là 1.17.
  3. **KHÔNG tự thêm `LIMIT`, và story này không thêm.** `deferred-work.md` §1-11 đã chốt: *"đường ra là một **quyết định sản phẩm** […] nó chạm hợp đồng của Panel Lookup **(1.17)**"*, và Ice **đã** chọn *"chấp nhận nguyên trạng"* một lần cho cùng câu hỏi ở lượt review 1.11. Story 1.13 giữ nguyên lựa chọn đó và **ghi số**.
  4. ⚠️ **Ngân sách NFR1 đầu-cuối là 100 ms; 12,569 ms là phần backend.** Trần 10 ms là một con số **dẫn xuất** *(PRD dành ~99,95 ms cho vòng IPC Tauri + render, giả định `[A1]` — thứ **chưa ai đo**)*. Vượt trần backend **không** đồng nghĩa vượt NFR1; nó nghĩa là dư địa cho hai thứ chưa đo còn **87,4 ms** thay vì 90 ms. Con số nghiệm thu thật của NFR1 chỉ có sau khi Panel Lookup tồn tại.
  ⇒ 🔴 **Hình dạng đường ra cho 1.17, không phải một mệnh lệnh:** giới hạn số hàng **pha một** *(phân trang + đếm)* là thứ duy nhất chạm được vào 12,569 ms. Pha hai **không cần** làm gì — 1.17 chỉ hydrate đúng trang nó hiện, và số đo nói giá của việc đó là 0,3 ms.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.17, Quyết định #4 + Task 8).** `LIMIT` pha một đã cài (`LOOKUP_PAGE_LIMIT = 20`, chốt từ số đo Task 8). Đo lại trên 4 tệp thật (`vietphrase` tách lớp riêng từ 1.10c/1.16, khác 3 tệp thời điểm ghi mục này): char_idx 1 ký tự p95 **20,836 ms** không `LIMIT` → **5,109 ms** có `LIMIT`. 🔴 Phát hiện Task 8: đường sản phẩm thật của 1.17 (`commands::dict::lookup`, `Exact` cố định) **không đi nhánh `char_idx`** — nó luôn đi `ExactBtree` (p95 đo được **6,535 ms**, xa dưới trần 100ms NFR1). `LIMIT` vẫn cần cho AC12/FR31 trên `ExactBtree` và cho ngày `Substring` được dùng (1.18/7.7).

- 🟡 **HVTĐTD nghiệm thu trên FIXTURE, không trên dữ liệu thật — VẪN MỞ. Chủ sở hữu: story dựng lớp HVTĐTD.** AC11 của Story 1.13 *(từ loại · ví dụ · trích dẫn **tiếng Việt**, và rơi về nhãn tiếng Anh khi gỡ)* nghiệm thu bằng một fixture mang **đúng hình dạng** HVTĐTD *(`pos_lang = 'vi'`, ví dụ + trích dẫn tiếng Việt)*, vì `dict-hvtdtd.db` **không tồn tại** — chưa có nguồn thô *(`src-tauri/resources/dict/README.md:13`; `prd.md:856` [A2]; xem mục 🔴 HVTĐTD ở §Deferred from 1-10)*. Thứ đã nghiệm thu: **đường mã phân biệt được nhãn tiếng Việt với nhãn ngoại ngữ, và không đánh mất trường nào trên đường đi**. Thứ **chưa** nghiệm thu: hình dạng đó **trên dữ liệu HVTĐTD thật**. **Đừng đánh dấu FR35/FR36 là *"đã nghiệm thu trên dữ liệu thật"***.

- 🟢 **`app.manage(DictLayers)` chạy ở `setup()` nhưng CHƯA có người tiêu thụ nào đọc nó.** Story 1.13 mở tập lớp lúc khởi động *(§Quyết định #3A — NFR14/FR112 đòi một vòng đời tài nguyên có `close()` ở `RunEvent::Exit`)*, nhưng chưa có `#[tauri::command]` nào lấy nó ra: đó là **Story 1.17**. Hệ quả có ý thức: đường `resource_dir()/dict/` **chưa bao giờ chạy trên một bản dựng có tệp `.db` thật** — `bundle.resources` chưa mang thư mục đó *(**Story 10.1**)*, nên hôm nay mọi bản dựng lên với **0 lớp** và dòng `dict[layers] 0 layer(s) loaded from …` ra stderr. Trạng thái đó **đúng theo thiết kế** *(AC3: thư mục không tồn tại ⇒ tập lớp rỗng, không lỗi)*, nhưng nó cũng nghĩa là **tên thư mục con `dict/` chưa được một lượt chạy thật nào xác nhận**. 🔴 Story 10.1 phải khớp hai đầu: khoá ánh xạ trong `bundle.resources` và hằng `DICT_RESOURCE_DIR` của `lib.rs:36`.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.17, AC8/Task 2).** `commands::dict::wire::lookup_dictionary` (`#[tauri::command]`, `try_state::<DictLayers>()`) là người tiêu thụ đầu tiên. ⚠️ Vế **`bundle.resources`/Story 10.1** ở lại mở nguyên — mọi bản dựng hôm nay vẫn lên với 0 lớp; đây là món nợ RIÊNG, không phải món nợ story này đóng (Task 11 nhắc lại ở cuối story 1.17).

## Deferred from: code review of 1-13-duong-tra-cuu-giu-nguyen-bat-dong-giua-cac-nguon (2026-08-05)

- **`SchemaTooNew` chỉ từ chối tệp mới hơn `SUPPORTED_SCHEMA_VERSION`, chưa bắt tệp cũ hơn** — `src-tauri/src/core/dict/layer.rs:229` chỉ kiểm `file_version > SUPPORTED_SCHEMA_VERSION`. Một tệp mang `PRAGMA user_version` và `dict_meta('schema_version')` **nhất quán với nhau nhưng thấp hơn** phiên bản ứng dụng hiện tại sẽ lọt qua cổng phiên bản. Deferred, pre-existing shape of AC4 — spec Story 1.13 chỉ đòi hỏi bắt ca "quá mới" (`epics.md`), và chưa có tệp schema version 0 nào từng tồn tại nên chưa có ca thật để nghiệm thu. Chủ sở hữu: story tiếp theo nào bump `SUPPORTED_SCHEMA_VERSION`/`tools/dict-build/src/schema.rs::SCHEMA_VERSION` lên 2. **(Chủ: story tiếp theo nào bump `SUPPORTED_SCHEMA_VERSION`.)**

## Deferred from: code review of 1-13-duong-tra-cuu-giu-nguyen-bat-dong-giua-cac-nguon — lượt 2, tests (2026-08-06)

- **`ordering_lacks_a_tiebreaker` (`tests/dict_boundary.rs:807`, AC7 gate) quét theo từng dòng vật lý** — một câu `ORDER BY` bị tách dòng lọt qua cổng hoàn toàn im lặng (không đỏ, không xanh). Rủi ro thật thấp hôm nay: mọi SQL trong `core/dict/**` viết một dòng theo đúng quy ước đã có. Một bản sửa an toàn cần phân tích ranh giới chuỗi ký tự Rust thật, không phải ghép cặp dòng kề nhau (rủi ro làm mất phát hiện ca một-dòng nếu dòng kế cận tình cờ chứa token `id`). **(Chủ: một story hạ tầng cổng kế tiếp.)**
- **`mentions_a_dict_db_file` (`tests/dict_boundary.rs:540`, AC2 gate) đòi literal `"dict-"` có gạch nối** — một tên tệp viết cứng không mang tiền tố đó (vd. `"hvtdtd.db"`) lọt qua. Mọi tên tệp thật trong `dict-manifest.toml` hôm nay theo đúng quy ước `dict-*.db`, nên chưa có rủi ro thật; xem lại nếu quy ước đặt tên đổi. **(Chủ: một story hạ tầng cổng kế tiếp.)**
- **Sáu trong mười biến thể `SkipReason` chưa có ca hành vi gọi tên trực tiếp trong `dict_sources.rs`** — `OpenFailed`, hai khoá của `MetaRowMissing`, `SourcesUnreadable`, `DuplicateLayer`, `LookupFailed`. Chỉ `MetaUnreadable`/`SchemaTooNew`/`SchemaVersionDisagrees`/`DuplicateSourceCode` được test trực tiếp. Mỗi ca cần dựng fixture hỏng riêng — khối lượng thuộc một lượt hardening test riêng. **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**
- **Nhóm khoảng trống độ phủ nhỏ hơn, gộp lại:** (a) `.DB` viết hoa chưa có ca chứng minh được nạp như `.db` (NFR14); (b) lỗi `read_dir` khác `NotFound` (vd. từ chối quyền) chưa có ca chứng minh vẫn trả tập lớp rỗng không panic — đúng đường vừa sửa ở lượt 1; (c) chưa có ca xoá tệp của một lớp bị `conflict_with` từ chối để chứng minh nó không còn bị khoá; (d) chưa có ca gọi `senses()` (chỉ `lookup()`) trên hai lớp khác nhau cùng `entry_id` để chứng minh không trộn dữ liệu; (e) chưa có ca ép `layer.lookup()` hỏng giữa chừng để chứng minh `SkipReason::LookupFailed`; (f) chưa có ca dựng một **thư mục** tên `*.db` để chứng minh bị từ chối an toàn; (g) nhánh `FtsTrigram`/`CharIdx` 2-ký-tự chỉ xuất hiện trong bench `#[ignore]`, chưa có ca khẳng định ở tầng gom. **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**

---

## Deferred from: 1-14-khung-bon-panel (2026-08-06)

*Mọi mục dưới đây là thứ Story 1.14 **cố ý KHÔNG làm**, hoặc chưa đo được. Mười hai mục mà `deferred-work.md` giao đích danh story này đã được đánh dấu đóng **tại chỗ** ở các mục phía trên, mỗi cái kèm tên phép kiểm và số đo.*

- 🔴 **BỐN NGƯỠNG MÀN HÌNH HẸP của UX-DR15 KHÔNG đóng ở đây — chủ sở hữu là Story 4.12. **(Chủ: Story 4.12.)** `epics.md:1617` cấm tường minh: *"ngưỡng kích thước cụ thể đóng ở Story 4.12, không đóng ở đây"* và *"không được cài cơ chế ẩn theo cách khiến Story 4.12 phải mổ lại bố cục"*. Story này giao **CƠ CHẾ**: `SACRIFICE_ORDER` · `NEVER_SACRIFICED` · `nextToSacrifice()` · `nextToRestore()` ở `src/layout/workspaceLayout.ts` — **hàm thuần**, không đọc `window.innerWidth`, không một `matchMedia` nào trong toàn `src/**`. `scripts/check-layout.mjs` Kiểm A cưỡng chế ba mệnh đề của AC7 trên **cả 16 tập con** của bốn panel. ⇒ 4.12 **chỉ phải nối ngưỡng vào**.
  ⚠️ Vế *"Tra cứu rút về THANH TRẠNG THÁI, không bao giờ mất hẳn"* **chưa cài** — `panel.lookup` hôm nay chỉ **nhường**. Đừng đọc `SACRIFICE_ORDER` thành *"Tra cứu được phép biến mất"*. Cũng thuộc **Story 4.12**; ngăn kéo cũng vậy.
  ⚠️ **Sự thật đã có mà 4.12 sẽ đụng:** `tauri.conf.json:19-20` khai `minWidth: 960` · `minHeight: 600`, nên ngưỡng *"< 860 rộng ⇒ báo không hỗ trợ"* của UX-DR15 **không đến được bằng cách kéo cửa sổ** trên cấu hình hôm nay. Story này không sửa `tauri.conf.json` *(`deferred-work.md` [D4], Ice chốt lần thứ tư)* — ghi ra để 4.12 quyết **một lần**.

- ⚠️ **LỖ NFR17 MỞ RA CÓ Ý THỨC: bốn `layout.toggle_*` không có phím.** Ẩn/hiện panel hôm nay **chỉ tới được bằng chuột** *(qua menu ngữ cảnh của dockview)*. Đổi lại: `unbound()` giữ được **bốn** phần tử thật, nên **AC6 của Story 1.6** *(*"liệt kê được thao tác chưa gán phím"*)* không mất bằng chứng — gán phím cho cả bốn sẽ làm `unbound()` trả mảng rỗng và **không cổng nào đỏ** *(§Bẫy 5 của story)*. Một lỗ **có tên và có chủ** tốt hơn một bằng chứng bị xoá. Chủ: **Story 1.21** *(màn hình gán phím)*. ⚠️ Handler thì **chạy thật** — `registry.ts` ném với một `run` thiếu, nên không có command rỗng nào ở đây.
  → ⚠️ **ĐÓNG MỘT NỬA 2026-08-11 (Story 1.21), và nửa còn lại KHÔNG được đóng — mệnh đề ghi ra bằng chữ thay vì gạch mục.**
  **Đã đóng, theo nghĩa của FR22:** từ hôm nay người dùng **gán được** phím cho cả bốn `layout.toggle_*` ở màn hình phím tắt, và lựa chọn đó sống qua các phiên. Ẩn/hiện panel không còn *"chỉ tới được bằng chuột"* đối với người dùng chịu gán một phím.
  **KHÔNG đóng, và không được đóng:** *"bộ MẶC ĐỊNH của sản phẩm có phím cho bốn thao tác này"*. Gán hợp âm mặc định cho chúng làm `unbound()` trả mảng rỗng và `check-commands.mjs` **đỏ** — AC7 của Story 1.21 nói đích danh điều đó. Số thật sau story: `unbound()` giữ **16** phần tử *(bốn `layout.toggle_*` · hai `library.import_*` · ba của 1.19 · ba của 1.20 · bốn của 1.21)*. Một lỗ có tên vẫn tốt hơn một bằng chứng bị xoá — nay nó còn có một **đường ra** cho người dùng. **Chủ của nửa còn lại: chưa gán**, và nó chỉ mở lại nếu một story sau tìm được một hợp âm mặc định có nghĩa mà không giết bằng chứng của AC6/1.6.

- 🔴 **Vế THỊ GIÁC của story CHƯA đo trên WKWebView, và ca Windows chưa đo.** Bảng 35 ca của §Debug Log References chạy trên **Blink/Chromium (Playwright headless), macOS 24.6 arm64**. Lượt `npm run tauri dev` **có chạy** và nghiệm thu **AC4** *(vòng lưu → đóng → mở lại → khôi phục, trong WKWebView thật với IPC thật)* — nhưng nó **không** nghiệm thu bố cục, khe 2px, kéo–thả hay vòng xoay focus, vì không có đường lái cửa sổ native. **Đừng viết "tương đương" bằng suy luận.** Bàn giao **Story 1.3 / 10.9**, nơi đã có lượt runner hai nền tảng để bấu vào. *(Tiến bộ so với Story 1.6: cổng 1420 lần này **rảnh**, nên `tauri dev` chạy được — giới hạn còn lại là lái GUI, không phải hạ tầng.)* **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**

- ⚠️ **Preset `Review Mode` chưa dựng — Story 8.11.** `LAYOUT_PRESETS` hôm nay có **hai**: `layout.preset_grid` *(2×2, mặc định)* và `layout.preset_columns` *(bốn cột)*. Hợp âm `Mod+Alt+3` **để trống có chủ ý** cho preset thứ ba *(`Bản dịch của tôi` cạnh `Bản Reviewer đã sửa`)*, đúng thứ tự mockup. **(Chủ: Story 8.11.)**

- ⚠️ **Preset do NGƯỜI DÙNG đặt tên chưa có đường vào — Story 1.21.** `ScopeKind::LayoutPreset` *(`GlobalOnly`)* và `BootstrapConfig.layout_presets` đã có từ Story 1.8 và story này **không** ghi vào chúng: hai preset trên là hằng số ở frontend. 🔴 Và **KHÔNG dựng thanh chuyển phạm vi Toàn cục/Tác phẩm cho preset** — `kinds.rs:36` gọi tên đích danh cái bẫy đó.
  → 🔴 **ĐỔI CHỦ 2026-08-11 — Story 1.21 TRẢ LẠI món nợ này, và Ice ký.** Lý do đo được: `epics.md:1579-1581` giao FR17/FR18 cho **Story 1.14**, còn `epics.md:1883` giao Story 1.21 **đúng FR22**; một màn quản lý preset đặt tên có **0 AC** ở cả hai chỗ. Dựng một bề mặt cho `ScopeKind::LayoutPreset` trong story phím tắt là thêm một năng lực không AC nào yêu cầu — đúng thứ §KHÔNG-LÀM của mọi story lớn trong dự án này từ chối.
  ✅ Vế *"không dựng thanh chuyển phạm vi"* thì Story 1.21 **có** tuân, và tuân cho chính bề mặt của nó: mockup `settings.html:243-248` vẽ hai nút `Toàn cục`/`Tác phẩm` cho **phím tắt**, và story thay chúng bằng đúng một câu (`shortcuts.scope_note`, nguyên văn `settings.html:246`). Tiền lệ đã có; story sau của preset chép nó.
  **Chủ mới: chưa gán — nêu ở retrospective Epic 1.** **(Chủ: một story quản lý preset đặt tên tiếp theo.)**

- ⚠️ **Kiểm B của `check-layout.mjs` đo NHỊP GHI, không đo rằng `WorkspaceDock.vue` thật sự dùng lịch đó.** Nó `import()` `src/layout/writeSchedule.ts` và đẩy 1.251 sự kiện qua `simulateWrites()` — kéo sash 3 s ⇒ **1** lượt ghi; kéo liên tục 20 s ⇒ **4** lượt ghi với không thay đổi nào chờ quá **5.000 ms**. Nhưng một lượt sửa `WorkspaceDock.vue` gọi `emit('persist')` thẳng ở mỗi `onDidLayoutChange` sẽ **đi qua cổng** — cổng không thấy chỗ nối. Lưới còn lại là một lượt đếm tay trong DevTools. Cùng hạng với *"cổng không được type-check"*. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- ⚠️ **`localStorage`/`sessionStorage` gọi TRẦN vẫn đi qua một mệnh đề CẤM, không qua danh sách cho phép.** Kiểm C của `check-layout.mjs` hỏi ngược *"mọi thành viên `window.`/`document.` phải nằm trong danh sách CHO PHÉP"* — đúng lập luận của `config_invariants.rs:92-94`. Nhưng `localStorage` không tiền tố là một **định danh tự do**, và liệt kê hết định danh tự do đòi một bộ phân tích cú pháp thật *(một phụ thuộc npm mới — NFR15)*. Ba cái tên đó vì vậy vẫn nằm trong một danh sách cấm hẹp. **Mở lại** khi dự án có lý do độc lập để thêm một parser. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- ⚠️ **Bảng `PANEL_COMPONENTS` và map `components` phải khớp nhau, và không cổng nào canh.** `src/layout/workspaceLayout.ts` khai tên component dạng chuỗi; `WorkspaceDock.vue` khai map thật. Một tên lệch cho ra **panel trắng** kèm `console.error` của chính dockview — không cổng nào đỏ. Rẻ nhất để đóng: cho `check-layout.mjs` đọc luôn map trong `.vue`. Không làm ở story này vì nó đòi một bộ phân tích `.vue` thứ ba trong cây script. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- ⚠️ **Chín biến `--dv-tab-group-color-*` cố ý ĐỂ TRỐNG.** Chúng phục vụ tính năng "tab group có màu" mà sản phẩm không dùng ở đâu. Khai chúng đòi **chín màu MỚI** phải qua Kiểm C của `check-tokens.mjs` — tức mở một bảng màu thứ hai để phục vụ một tính năng không dùng. Ngày nào sản phẩm dùng tới, đó là một quyết định thiết kế **có chữ ký**. **(Chủ: story kế tiếp nếu tab-group màu được dùng thật.)**

- ⚠️ **Ba biến `--dv-*` mang tên KHÔNG khớp thuộc tính CSS mà cổng đọc.** `--dv-floating-box-shadow` không khớp `box-shadow` của Kiểm F; `--dv-overlay-z-index` không khớp `z-index`; `--dv-floating-group-dragging-opacity` không khớp `opacity` của Kiểm D. Cả ba **đã được đặt đúng luật bằng tay** *(`none` · một ngữ cảnh xếp lớp cơ học có ghi lý do · `1`)* và lý do viết ngay cạnh — nhưng đó là **kỷ luật, không phải cưỡng chế**. Ngày dockview thêm một biến kiểu này, không gì báo. `epics.md:381` nói ranh giới kiến trúc phải cưỡng chế **bằng test**; đây là một chỗ nó chưa được. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- ⚠️ **`ui-md` chạy giãn dòng 1.5 nhưng câu trạng thái panel AI XUỐNG DÒNG THẬT** — xem mục *"Kiểm E không phát hiện được một cờ `wraps` khai sai"* ở trên. **Chưa chốt, quyết định của Ice**, và nó chạm `DESIGN.md`. **(Chủ: story kế tiếp dựng panel chạm `ui-md`.)**

- ⚠️ **Chuỗi chẩn đoán trong `.vue` phải viết KHÔNG DẤU.** `WorkspaceDock.vue` và `WorkspaceMode.vue` mang ~7 lời gọi `console.error`/`console.warn` viết tiếng Việt **không dấu**, theo tiền lệ `src-tauri/src/commands/config.rs:36`. Lý do: Kiểm A của `check-i18n.mjs` đo **DẤU** và không phân biệt được *chuỗi hiển thị* với *chẩn đoán ra console*. Đường thoát dễ — dời khối logic sang một `.ts` — là **đúng đường mà `deferred-work.md:35` cấm bằng chữ**, nên không dùng. Lời giải đúng là cho cổng một khái niệm *"chẩn đoán"* *(ví dụ: chuỗi nằm trong đối số của `console.*` được miễn trừ có tên)*. Thuộc **Story 10.9**. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- ⚠️ **`as unknown as Record<string, VueComponent>` ở `WorkspaceDock.vue`.** `dockview-vue` khai `VueComponent<T = any> = DefineComponent<T>`, và prop là vị trí **nghịch biến** nên `DefineComponent<DockviewPanelProps>` không gán được. Đường thay thế *(khai `params?:` ở cả năm component)* qua được kiểm tra kiểu **bằng cách nói dối**: dockview LUÔN truyền `params`, và `PanelTab.vue` không chạy được nếu thiếu. Ép kiểu **một lần ở đúng ranh giới thư viện** rẻ hơn năm lời nói dối rải trong mã. **Mở lại** nếu `dockview-vue` siết kiểu ở một bản sau. **(Chủ: story kế tiếp chạm `WorkspaceDock.vue`.)**

## Deferred from: code review of 1-14-khung-bon-panel (2026-08-06)

- **Khoá tiêu đề panel chảy qua một lời gọi `t()` KHÔNG literal, ngoài tầm quét của `check-i18n.mjs`.** `PANEL_TITLE_KEYS` sống ở `src/layout/workspaceLayout.ts` (một tệp `.ts`, Kiểm A2 chỉ quét `.vue`) và đổ vào `PanelTab.vue:80` qua `t(props.params.params.titleKey ?? '')` — một biểu thức, không một literal. Giá trị hôm nay đều khớp `vi.json` (xác minh trực tiếp), nhưng một lỗi gõ tương lai trong bốn khoá đó sẽ không bị cổng nào bắt — `resolve.ts` cố ý không sập với khoá thiếu, nên hậu quả là khoá thô hiện ra màn hình. Cùng lớp rủi ro với mục *"Bảng `PANEL_COMPONENTS` và map `components` phải khớp nhau"* ở trên. **(Chủ: một story hạ tầng cổng kế tiếp.)**
- **`PANEL_SUFFIXES` ở `src/commands/index.ts:172-173` là bản chép tay của `PANEL_IDS`** (`src/layout/workspaceLayout.ts`), chỉ có một dòng comment "chép từ", không cổng nào đối chiếu hai bảng. Thêm/đổi tên/xoá một panel sau này có thể làm bốn `layout.toggle_*` trôi khỏi `PANEL_IDS` mà không cổng nào đỏ. **(Chủ: một story hạ tầng cổng kế tiếp.)**
- **`applyPreset()` luôn `api.clear()` rồi dựng lại TOÀN BỘ bốn panel**, kể cả khi preset yêu cầu đã là preset đang áp — vô hại hôm nay vì bốn panel là khung rỗng, nhưng sẽ mất trạng thái thật (cuộn, nội dung đang gõ, state AI) một khi panel có nội dung thật. **Nhặt lại ở Story 1.16 / 1.17 / Epic 2**, nơi panel lần đầu có trạng thái đáng giữ.
  - ✅ **ĐÓNG cho Panel Source — Story 1.16, AC9 (2026-08-06).** State (Chương đã nạp, âm Hán
    Việt đã tra, tab/kiểu xem đang chọn) chuyển ra `src/panels/sourcePanelState.ts` —
    module-level singleton, không `ref` cục bộ của `SourcePanel.vue`. `ensureChapterLoaded`/
    `ensureHanVietLoaded` idempotent qua cờ module-level, nên một lượt `applyPreset()` tháo
    và dựng lại instance chỉ đọc lại state cũ, không gọi lại `read_open_chapter`/
    `read_han_viet`. ⚠️ **Vẫn MỞ cho Panel Lookup (1.17), Editor/AI (Epic 2/4)** — mỗi panel
    tự chuyển state ra ngoài khi nó lần đầu có nội dung thật, đúng khuôn vừa dựng ở đây.
  - ✅ **ĐÓNG cho Panel Lookup — Story 1.17, AC10 (2026-08-06).** State (truy vấn, kết quả
    pha một+hai, lỗi, pending) chuyển ra `src/panels/lookupPanelState.ts` — module-level
    singleton, không `ref` cục bộ của `LookupPanel.vue`. `runLookup()` không idempotent theo thiết
    kế (mỗi lượt gọi là một thao tác tường minh mới của người dùng, không như
    `ensureChapterLoaded`), nhưng state vẫn sống sót qua `applyPreset()` vì nó ở ngoài
    component. `resetLookupPanel()` gọi CÙNG điểm nghẽn `resetSourcePanel()` ở
    `libraryImport.ts::finishSubmit` khi Tác phẩm đổi. ⚠️ **Vẫn MỞ cho Editor/AI (Epic 2/4)**. **(Chủ: story kế tiếp áp preset có trạng thái thật.)**

## Deferred from: 1-15-tac-pham-tren-dia-va-duong-vao-van-ban-toi-thieu (2026-08-06)

- 🔴 **Mọi Chương nhập ở Epic 1 có `segment_count = 0`, và Story 2.1 phải xử lý bằng một thao tác tách TƯỜNG MINH.** Quyết định #4 của story: AD-4 đóng băng ranh giới segment tính một lần lúc nhập; cài một bộ tách "tạm" ở Story 1.15 là đóng băng vĩnh viễn những ranh giới sai (id đã "về hưu" không bao giờ được tái dùng — AD-3). `chapter.source_text` mang **nguyên khối** văn bản; **không** bảng `segment`, **không** dòng `segment` nào tồn tại. **Story 2.1** sở hữu bước di trú thêm bảng `segment` VÀ một thao tác "tách lần đầu" tường minh trong giao diện (hoặc một bước di trú dữ liệu), **không** một đường tính ngầm lúc nạp Chương — đường đó là vi phạm AD-4 trực tiếp.
  → ✅ **ĐÓNG 2026-08-12 — Story 2.1.** Bước di trú **5** (`SEGMENT_DDL`; số 4 đã cháy) cộng
  **hai** đường tách, và không đường nào tính ngầm: Chương **mới** tách trong `create_work`,
  cùng giao dịch với hàng `chapter` (AC13); Chương **cũ** đi qua lệnh IPC tường minh
  `split_chapter_into_segments`, một Chương một lượt, **từ chối** một Chương đã có segment
  thay vì ghi đè. Bước di trú cố ý chỉ làm một việc — `CREATE TABLE segment` — vì nhét phép
  tách vào đó trộn DDL với nghiệp vụ và chạy im lặng lúc mở Tác phẩm, đúng thứ AC3 cấm; và
  vì bản sao lưu trước di trú vẫn *"không nguyên tử, không xác minh lại"* (mục nợ ở trên,
  chưa ai vá) và đây sẽ là lượt di trú thật đầu tiên trên một `project.db` có dữ liệu người
  dùng. `segment_boundary.rs::the_splitter_has_exactly_two_product_call_sites` khoá con số
  hai đó lại.
  ⚠️ **21 Chương, không phải 25** — đo 2026-08-12 trên `~/Documents/AuraTranslate/`: 21 thư
  mục `.atproj`, mỗi thư mục đúng 1 Chương, tất cả ở `user_version = 3`. Con số 25 trong
  story là một ước, không một phép đếm.
  🔴 **Chúng CHƯA được tách** — story này dựng *đường*, và bấm nó trên 21 Chương thật là một
  thao tác ghi vào dữ liệu thật của Ice, cần Ice bấm. **Chủ: Ice**, một lượt nghiệm thu tay.

- ⚠️ **Đường kéo-thả tệp thật (Quyết định #1(b)) chỉ được verify bằng ĐỌC MÃ NGUỒN `tauri-runtime`/`tauri` đã ghim, chưa bằng một lượt kéo tay thật trên máy có GUI.** Task 0 của story kết luận `WindowEvent::DragDrop` nhận qua `on_window_event` cần **0 permission** (không đi qua ACL/capabilities) và `drag_drop_enabled` mặc định `true` — kết luận mạnh và có cơ sở (đọc trực tiếp `tauri-runtime-2.11.3`/`tauri-2.11.5`), nhưng môi trường triển khai (agent CLI) không có công cụ điều khiển GUI desktop để thật sự thả một tệp bằng chuột. Rủi ro thấp — cơ chế là API ổn định, dùng rộng rãi trong hệ sinh thái Tauri — nhưng câu này chưa có bằng chứng thực nghiệm. ~~**Nghiệm thu tay trước khi phát hành**, hoặc story kế tiếp có công cụ GUI automation.~~
  → ✅ **ĐÓNG 2026-08-06** — Ice kéo-thả một tệp thật bằng chuột trên macOS/WKWebView: viền vùng kéo-thả đổi màu lúc đang kéo, thả ⇒ đường dẫn điền vào ô, chưa ghi gì xuống đĩa cho tới khi bấm nút. Xem §Nghiệm thu tay của story `1-15…md` (bảng 3). ⚠️ **Chỉ macOS** — đường Windows đi qua WebView2/Win32, một cài đặt runtime khác hẳn, và ba event `Enter`/`Leave`/`Drop` là mã mới của lượt code review. Xem mục "nghiệm thu Windows" bên dưới.

- ✅ **Đường hiển thị lỗi kho trong webview thật (nợ giao lại từ Story 1.7/1.8) ĐÃ ĐÓNG 2026-08-06** — Ice dựng `$APPDATA` chỉ-đọc (`chmod 555`), mở lại app, và **đọc dải báo lỗi kho bằng mắt** trên macOS/WKWebView. Đây là món nợ đã treo qua ba story (1.7 → 1.8 → 1.15) vì mọi lượt triển khai đều chạy ở môi trường agent CLI không có công cụ dựng/đọc một cửa sổ desktop thật. AC10(a) của Story 1.15 nay **đạt**. ⚠️ **Chỉ macOS.** ~~cùng lý do ở trên: agent CLI không dựng được cửa sổ Tauri thật rồi đọc màn hình.~~ Xem mục đã cập nhật ở §*Deferred from: 1-7* và §*1-8* (2026-08-06).

- ⚠️ **Sáu số `Tuning` của Story 2.4 nay càng khó đo hơn: một phiên có thể chạy HAI kho cùng lúc** (`global.db` + `project.db` của Tác phẩm đang mở) — mỗi kho tự mang luồng checkpoint + pool 4 kết nối riêng. Chưa có phép đo nào về tranh chấp CPU/I/O giữa hai luồng checkpoint chạy song song trên cùng một tiến trình. **Story 2.4** đo lại cả sáu số trên Editor thật, nay nên đo trong ĐÚNG kịch bản hai kho, không phải một kho đơn lẻ. **(Chủ: Story 2.4.)**

- ⚠️ **`ports::ProjectStore` được khai (AD-2, Task 3) nhưng CHƯA CÓ CÀI ĐẶT nào** — cùng hoàn cảnh `TranslationProvider` (Epic 4). `commands::project` gọi thẳng `Option<&Store>`/`&Path` (khuôn `commands::config`), không qua cổng này. Cắm một `impl ProjectStore` thật là việc của epic đầu tiên cần trừu tượng hoá trên "một Tác phẩm đã mở" (ứng viên: Epic 2 Editor, Epic 3 Glossary). **(Chủ: Epic 4.)**

- ⚠️ **Tên Tác phẩm rỗng rơi về `"Untitled"` (tiếng Anh, không dịch)** — `core::library::atproj::sanitize_name`. Đây là một tên **thư mục hồi phòng**, không phải văn bản hiển thị (NFR16 áp cho UI, không áp cho tên tệp hệ thống), nhưng nó là quyết định thẩm mỹ chưa ai duyệt. Không có AC nào của story đòi validate trường "Tên" ở tầng giao diện trước khi nộp — form hôm nay cho phép nộp tên rỗng. **Story nào dựng màn hình gán tên Tác phẩm tử tế hơn** (nếu có) nên xét lại. **(Chủ: story kế tiếp chạm `atproj::sanitize_name`.)**

- ⚠️ **Đường "Dán văn bản" và ba điểm vào của Quyết định #1 (ô nhập đường dẫn, vùng kéo-thả) KHÔNG có mockup nào trong 29 tệp quy hoạch.** Giao diện `LibraryMode.vue` của story này được suy ra từ `.field`/`.dlg` của `mockups/library-and-import.html` + §Voice and Tone, không sao chép một thiết kế đã duyệt. Cùng khoảng trống mà story đã nêu cho Sally ở §Câu hỏi cho Ice — chưa có lượt thiết kế thị giác chính thức cho ba điểm vào này. **(Chủ: story kế tiếp dựng "Dán văn bản"/kéo-thả.)**

## Deferred from: code review of 1-15-tac-pham-tren-dia-va-duong-vao-van-ban-toi-thieu (2026-08-06)

- ⚠️ **`replace_open_work` thả `Store` cũ TRONG vùng khoá mutex** (`src-tauri/src/commands/project.rs:204-211`). `*guard = Some(new_work)` chạy `Drop` của `OpenWork` cũ ngay tại chỗ, mà `Drop` đó gọi `Store::close()` — join luồng writer + một lượt checkpoint TRUNCATE có trần — **trong khi vẫn giữ `OpenWorkState`**. Hôm nay chưa phải lỗi đang sống: chỉ có hai chỗ chạm khoá này (`replace_open_work` và `close_open_work`), và chỗ thứ hai chỉ chạy lúc `RunEvent::Exit`, nên không có tranh chấp thật. Nó trở thành rủi ro thật khi một story sau thêm **bất kỳ command nào đọc `OpenWorkState`** (Epic 2 Editor, Epic 3 Glossary là ứng viên gần nhất) — khi đó một lượt "mở Tác phẩm khác" sẽ chặn mọi lượt đọc state trong suốt thời gian đóng kho cũ. Khuôn sửa rẻ và đã biết: `let old = { let mut g = state.lock()…; g.replace(new_work) }; drop(old);` — thả kho cũ **ngoài** vùng khoá. Cùng họ với mục sáu số `Tuning` chưa đo ở trên. **(Chủ: story kế tiếp chạm `commands/project.rs`.)**

- ⚠️ **Chuẩn hoá xuống dòng (CRLF → LF) và khoảng trắng CỐ Ý KHÔNG làm ở Story 1.15** — `core::segment::import::import_text` giữ nguyên byte văn bản sau khi giải mã, và bước *"chuẩn hoá tối thiểu"* của chuỗi AD-39 vẫn **rỗng có chủ ý**. Ranh giới Ice chốt ở lượt code review 2026-08-06: **BOM là tạo tác của bước GIẢI MÃ** *(cắt ngay ở story này — `EF BB BF` là UTF-8 hợp lệ nên nó đi lọt `String::from_utf8`, và AD-4 đóng băng ranh giới segment lúc nhập ⇒ Epic 6 không sửa lại được)*; **CRLF là bước CHUẨN HOÁ** *(FR124/125 — Epic 6)*, nó đổi chỗ ngắt đoạn, tức đụng thẳng vào thứ Story 2.1 và Epic 6 sở hữu, nên sửa nó ở Story 1.15 là đúng cái bẫy *"bộ tách tạm"* mà story tự cấm. ⚠️ **Hệ quả phải biết:** mọi Chương nhập từ một tệp Windows ở Epic 1 mang `\r\n` trong `chapter.source_text`, và **Story 2.1** *(tách câu)* phải xử lý `\r` như khoảng trắng, không để nó dính vào cuối segment. **Chủ: Story 2.1 + Story 6.4/6.5.**
  → ✅ **PHẦN CỦA STORY 2.1 ĐÓNG 2026-08-12.** Bộ tách coi `\r` là khoảng trắng **và** là
  ranh giới cứng, nên bất biến thu được mạnh hơn vế mà mục nợ này đòi: **không segment nào
  chứa `\r` hay `\n`**, bất kể đầu vào và bất kể nhánh ngôn ngữ
  (`segment_contract.rs::no_segment_ever_carries_a_line_break`, chạy trên 5 đầu vào × 2 nhánh).
  ⚠️ **Vế Epic 6 VẪN MỞ:** `chapter.source_text` trên đĩa **không** được chuẩn hoá — story
  này cố ý không chạm nó (chuẩn hoá thật là FR124/FR125). Bộ tách chỉ **tự phòng thủ**.
  **Chủ phần còn lại: Story 6.4/6.5.**

- ⚠️ **Trần nhập 100 MB là một con số TẠM, chưa ai đo** — `core::segment::import::MAX_IMPORT_BYTES`, Ice chốt 2026-08-06 ở lượt code review. Nó tồn tại để một tệp bệnh hoạn không giết tiến trình (`fs::read` trọn tệp + `String` + bind SQLite ≈ 3 bản trong bộ nhớ, trên **luồng invoke đồng bộ**, và `panic = "abort"` biến cạn bộ nhớ thành giết cả tiến trình). **Chưa ai đo đỉnh RSS thật** cho một tệp 100 MB đi hết chuỗi, và nó **không** phải một phép đo về *"bao nhiêu thì Editor còn dùng được"*. Còn hai lỗ nữa ghi ra thay vì giấu: ① một **cửa sổ đua** giữa `metadata()` và `read()` *(tệp phình ra ở giữa)* — đóng nó đòi đọc theo khối có trần, mà nhập theo khối là Epic 6; ② lượt nhập vẫn **chặn luồng invoke**, không có tiến độ nào ngoài cờ `busy`. **Chủ: Story 2.4** *(đo `Tuning`)* cho con số, **Epic 6** cho đường đọc theo khối.

- 🔴 **Tầng Tác phẩm của `ScopeResolver` ĐÃ được cắm nhưng CHƯA CÓ MỘT CONSUMER NÀO** — sửa lại phạm vi của mục này ở lượt code review 2026-08-06, vì §Completion Notes của Story 1.15 khai **hẹp hơn thực tế**. Story đó viết *"chưa có method phân giải nào thực sự chạy với dữ liệu tầng Work"*, đúng nhưng thiếu: thực tế là **cả cái slot Tác phẩm chưa có ai đọc**. Cụ thể — `ScopeResolver::with_work` được dựng ở `commands::project::create_work` và cất vào `OpenWork.scope`, và `OpenWork.scope` **không được đọc ở đâu trong `src-tauri/src/**`**; `OpenWorkState` chỉ có đúng hai chỗ chạm (`lib.rs::open_work_slot` đăng ký, `lib.rs::close_open_work` đóng lúc thoát); và đường phân giải sản phẩm thật (`core::scope::store`) vẫn dựng `ScopeResolver::global_only()`. ⇒ **AC9 đạt về CHỮ** *(có hàm dựng thứ hai, ba chữ ký không đổi, đường sản phẩm không còn luôn truyền `None`)* **nhưng CHƯA đạt về MỤC ĐÍCH**, và Ice đã chấm như vậy ở lượt review. ⚠️ Lý do **không** vá được ở Story 1.15: `project.db` chưa có bảng nào ở tầng Tác phẩm để tra *(Glossary → Epic 3, TM → Epic 7, prompt → Epic 4)*, và thêm một bảng như thế hôm nay vi phạm luật `store::schema`: *"Không thêm bước cho một lược đồ chưa tồn tại"*. **Chủ: epic đầu tiên mang dữ liệu tầng Tác phẩm** *(ứng viên gần nhất: Epic 3, Glossary)* — story đó nối `OpenWork.scope` vào đường phân giải thật CÙNG LƯỢT với bảng đầu tiên. Hàm dựng nay đã có test (`scope_contract.rs::the_second_constructor_carries_a_work_tier_and_the_first_one_does_not`), trước lượt review nó ship với **0 test**.
  → 🟡 **Story 3.1 (2026-08-19): hàm TIÊU THỤ đã có, CHƯA có chỗ gọi sản phẩm.** `core::glossary::entries_eligible_for_injection(resolver, global, work)` là hàm đầu tiên trong kho THẬT SỰ nhận một `ScopeResolver` mang tầng Work và phân giải nó (`scope_contract.rs`/`glossary_contract.rs` canh bằng test, gồm ca "tầng Tác phẩm chờ chốt che tầng Global đã chốt"). Nhưng **không có command IPC hay đường sản phẩm nào gọi nó với `OpenWork.scope` thật** ở story này — §Never của Story 3.1 cấm mọi bề mặt IPC/màn hình (`epics.md`: "Không màn hình ⇒ không khoá chuỗi"). ⇒ Mục nợ gốc **chưa đóng hẳn**: `OpenWork.scope` vẫn không được đọc ở đâu trong `src-tauri/src/**` ngoài test. **Chủ chuyển sang Story 3.3** (Thêm nhanh thuật ngữ từ bất kỳ panel nào) — story đầu tiên của Epic 3 dựng một bề mặt IPC thật, nên là story đầu tiên có lý do nối `OpenWork.scope` vào một lời gọi `entries_eligible_for_injection`/`load_tier` sản phẩm.
  → ✅ **ĐÃ ĐÓNG 2026-08-20 (Story 3.3).** `commands::glossary::work_context` (`src-tauri/src/commands/glossary.rs`) là chỗ ĐẦU TIÊN trong mã sản phẩm đọc `&open.scope` — nó nạp `(&Store, &ScopeResolver)` từ `OpenWork` rồi truyền cho `core::glossary::store::resolve_term_for_quick_add`, chỗ này tự gọi `ScopeResolver::apply_override` với dữ liệu THẬT ở cả hai tầng khi một Tác phẩm đang mở. Ba command `glossary_lookup_term`/`glossary_add_term`/`glossary_update_term` là bề mặt IPC đầu tiên của module này (`lib.rs::generate_handler!`). `OpenWork.scope` không còn là một trường chỉ được ĐẶT mà không ai ĐỌC.

- ⚠️ **`err.project.meta_too_new` và `MessageKey::ProjectMetaTooNew` ĐÃ BỊ GỠ ở lượt code review 2026-08-06** — Ice chốt. Cơ chế từ chối một `meta.json` phiên bản mới hơn **vẫn còn nguyên và vẫn có test** (`MetaError::SchemaTooNew` + `WorkMeta::read` + `project_contract.rs::a_newer_meta_schema_is_refused_without_touching_a_single_byte`); thứ bị gỡ là **bề mặt hiển thị** của nó. Lý do: Story 1.15 không dựng màn hình *"mở lại một `.atproj` đã có"*, nên `WorkMeta::read` **không có một chỗ gọi sản phẩm nào**, nên một `MessageKey` + một khoá `vi.json` cho nó là **một khoá cho tính năng chưa tồn tại** — đúng thứ Story 1.7 §Completion Notes #3 cấm và `scope_contract.rs` trích lại nguyên văn. 🔴 **Story nào dựng đường mở lại một `.atproj`** *(ứng viên: Epic 5, lưới Tác phẩm)* **thêm lại cả ba thứ — biến thể `ProjectError`, `MessageKey`, khoá `vi.json` — CÙNG MỘT LƯỢT với màn hình.** **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**

- ⚠️ **AC2 đếm "đúng ba thành phần", nhưng một `.atproj` đang SỐNG có NĂM mục trên đĩa** — `project.db-wal` và `project.db-shm` là sidecar của chế độ WAL. `project_contract.rs::creating_a_work_lays_down_exactly_three_things_on_disk` lọc hai tệp đó ra trước khi so, và bộ lọc đó **hợp lý** *(chúng là một phần của chính `project.db`, không phải một tệp lạc)* — nhưng nó là một cách **diễn giải lại** AC2 mà story không khai trong năm độ lệch. 🔴 **Quan trọng cho Epic 5:** `Indexer` quét thư mục sẽ gặp **năm** mục, không phải ba, và một lượt quét cho rằng `.atproj` chỉ chứa đúng ba tên sẽ sai. `close()` chạy TRUNCATE nên `-wal` co về 0 byte, **không** biến mất. **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**

- 🔴 **NGHIỆM THU TRÊN WINDOWS CHƯA TỪNG CHẠY — mở mới 2026-08-06, sau khi lượt nghiệm thu tay macOS đóng hai món nợ ở trên.** Cả năm bảng nghiệm thu của Story 1.15 *(trùng tên không xoá dữ liệu · form không biến mất · kéo-thả điền vào ô · ba ca biên `.docx`/không đuôi/BOM · AC10(a) đường hiển thị lỗi kho)* chạy **chỉ trên macOS/WKWebView**. CI dựng **cả `macos-26` lẫn `windows-2025`** và **NFR14 là một mệnh đề hai nền tảng**, nên **đừng đọc hai mục đã đóng ở trên thành "đã xong mọi nền tảng"**. Bốn đường sau là **Windows-only theo bản chất** ⇒ **không** phép kiểm nào chạy trên macOS có thể làm chúng đỏ, kể cả khi chúng hỏng:
  1. **Tên thiết bị dành riêng** — `CON.txt`/`NUL.md`/`COM1` chỉ bị NTFS từ chối; trên macOS chúng là tên thư mục hợp lệ. `sanitize_name` thêm hậu tố `_` và có test đơn vị (`a_folder_name_survives_both_platforms_rules`), nhưng chưa ai xác nhận một thư mục `CON_.atproj` **thật sự tạo được** trên NTFS.
  2. **`remove_dir_all` với một tệp đang mở** — đúng bài học NFR14 mà `close_open_work`/`Store::close` tồn tại để chống. Trên macOS xoá một tệp đang mở là **hợp lệ**, nên đường này **không thể** đỏ ở đó.
  3. **Trần độ dài đường dẫn** — `MAX_FOLDER_NAME_BYTES = 180` nhắm `NAME_MAX` 255 byte của APFS/ext4, nhưng Windows còn có trần **`MAX_PATH` 260 ký tự cho CẢ đường dẫn**, mà `C:\Users\<tên>\Documents\AuraTranslate\` đã ăn một phần đáng kể. Con số 180 chưa được đo với ràng buộc đó.
  4. **Kéo-thả trên Windows** — `WindowEvent::DragDrop` đi qua **WebView2/Win32**, một cài đặt runtime khác hẳn WKWebView/AppKit; và ba event `DRAG_ENTER_EVENT`/`DRAG_LEAVE_EVENT`/`DRAG_DROP_EVENT` là **đường mã mới** sinh ra ở lượt code review 2026-08-06, chưa chạy trên nền tảng nào ngoài macOS.

  **Chủ: lượt QA trước khi phát hành**, hoặc story đầu tiên có một máy Windows thật trong tay. Đừng đóng mục này bằng một lượt chạy CI xanh — CI chạy `cargo test`, không dựng cửa sổ và không kéo-thả.

## Deferred from: 1-10c-am-han-viet-dung-nguon-va-dung-nhan (2026-08-06)

- ✅ **Story 1.16 (tab Hán Việt) HẾT CHẶN.** Lý do chặn — `dict_entry.han_viet` của lớp nền mang âm NÔM (`Unihan kVietnamese`) thay vì âm Hán Việt — đã đóng: AC1/AC2 đổi vai `kVietnamese` sang `nom_reading`, `han_viet` giờ chỉ nhận âm gắn nhãn tường minh (Thiều Chửu · en-wiktionary-vi · Trần Văn Chánh). Bốn tệp `.db` đã dựng lại (`SCHEMA_VERSION` v2), manifest đã cập nhật, mọi cổng xanh.
  ⚠️ **Bẫy 4 của story này — Story 1.16 PHẢI đọc trước khi viết đường tách âm đọc:** ba quy ước phân tách nhiều-âm-trong-một-chuỗi tồn tại song song trong dữ liệu thật — Thiều Chửu dùng `|` (`"đinh|chênh"`), Trần Văn Chánh **và** en-wiktionary-vi dùng `,` (`"đáng, đương"`, có khoảng trắng sau dấu phẩy ở TVC), Unihan/`nom_reading` cũ dùng khoảng trắng (`"tợ tử"`). Story 1.10c KHÔNG chuẩn hoá bốn quy ước này về một — `nom_guard::split_readings` (`tools/dict-build/src/nom_guard.rs`) đã viết luật "cắt trên cả `|`, `,`, khoảng trắng" cho MỤC ĐÍCH ĐỐI CHIẾU AC5 (build-time only, không ghi lại `.db`), và đó chính là luật Story 1.16 nên tái dùng ở đường ĐỌC — xem module đó làm tham khảo trước khi viết một bộ tách thứ hai.

- ⚠️ **Nguồn kaikki.org khai DEPRECATED trên trang tải** (`raw/en_wiktionary_vi/`, ghim 2026-08-06, `Last-Modified: 2026-08-02`). Story này ghim đúng bản đã tải, không có đường thay ổn định hơn tại thời điểm khảo sát. **Câu hỏi mở cho Ice** (chưa trả lời — story 10.1 hoặc lượt làm mới dữ liệu kế tiếp phải quyết): ai/khi nào làm mới `dict-core.db` theo một dump kaikki mới hơn, và làm gì nếu kaikki ngừng phục vụ hẳn (sáu trong bảy nguồn nền hôm nay đi qua `wiktextract_common.rs`, tức phụ thuộc CÙNG một nhà cung cấp trích xuất). **(Chủ: Ice — câu hỏi mở ghi ngay trong mục này.)**

- 🔴 **Dư địa NFR6 còn lại sau story này: 3.104.634 byte (0,78% trần)** — **SỬA ở lượt code review 2026-08-06**: bản ghi gốc của story ("26.760.192 byte còn lại") dùng baseline "trước story" SAI (343.991.430, số CŨ của `epics.md:336` từ TRƯỚC Story 1.10b, KHÔNG cộng font+baseline app+license). Baseline ĐÚNG là số Story 1.10b tự đo (`1-10b-...md:934,963,1087`) = **384.525.446**. Payload THẬT sau story 1-10c = **396.895.366 / trần 400.000.000**. `prd.md:946` đã cảnh báo dư địa này vốn dành cho **HVTĐTD + Cổ hán văn** — với chỉ **3,1 MB** còn lại, 🔴 **hai lớp đó gần như CHẮC CHẮN không còn vừa** trừ khi cực nhỏ. **Chưa đo HVTĐTD/Cổ hán văn thật** — quyết định tầng PRD (nâng trần, hoãn một lớp, hoặc bỏ `sense_fts_nd`) cần cân nhắc SỚM hơn dự tính ban đầu, không phải quyết định của story dựng dữ liệu tiếp theo. **Đo TRƯỚC khi hứa đóng gói** — đúng bài học `prd.md §8.2` đã ghi cho chính hai lớp này, giờ càng cấp thiết hơn. **(Chủ: Story 10.1.)**

- ✅ **Lưới AC5 (`nom_guard`) — sửa lỗ hổng cấu trúc + dương tính giả, cả hai phát hiện ở lượt code review 2026-08-06 (SAU khi story đã ở trạng thái `review`).** (1) Bản gốc: `LABELED_NOM_SOURCE` chỉ có ở `dict-core.db`, nên AC5 vĩnh viễn `0/0` cho ba tệp gỡ rời (AD-10: một tệp một `dict_source`) — sửa bằng nạp nhãn Nôm từ raw `en-wiktionary-vi` cho MỌI lớp gỡ rời (`build.rs::load_en_wiktionary_vi_labeled_nom`, không thêm mã riêng-từng-nguồn). (2) Sửa xong lộ dương tính giả THẬT: Thiều Chửu (nguồn chuẩn) bị gắn cờ 63,4% — nguyên nhân là `en-wiktionary-vi` tự gắn cả hai nhãn HV/Nôm cho cùng âm khá thường xuyên. Sửa bằng `nom_guard::nom_only_readings` (loại âm tự-trùng-vai khỏi vế đối chứng). Số cuối trên dữ liệu thật: thieu-chuu 5,2%, tran-van-chanh 6,5%, cả hai an toàn; mệnh đề "đỏ được" của AC5 đo lại 79,5% (từ 92,4% gốc, do siết phép lọc) — vẫn cách xa ngưỡng. Bốn SHA-256 `.db` **không đổi** — bản vá chỉ đổi phép kiểm, không đụng dữ liệu ghi ra. **Story 1.16** nên đọc `nom_guard.rs` (cả `split_readings` VÀ `nom_only_readings`) trước khi viết logic liên quan tới HV/Nôm.

- ⚠️ **`dict-tran-van-chanh.db` mang rủi ro pháp lý CHƯA ĐÓNG, có chủ ý** — Trần Văn Chánh (1999) còn trong bản quyền, tác giả còn sống, dự án CHƯA xin phép trực tiếp. Giảm thiểu: đóng gói làm lớp gỡ rời (FR112 = xoá một tệp), `license_kind = "copyrighted"`, rủi ro ghi thẳng vào `dict_source.attribution` + `assets/licenses/tran-van-chanh.txt`. **Chủ: lượt rà pháp lý trước khi phát hành công khai** (cùng nhóm với rủi ro VietPhrase/Cổ hán văn đã ghi ở `prd.md §8.6`) — xin phép tác giả hoặc chấp nhận rủi ro có ý thức là quyết định tầng dự án, không phải quyết định kỹ thuật.

- 📝 **Lệch giữa story này và `prd.md §8.2` — ghi ra, KHÔNG sửa file quy hoạch (đúng ranh giới story đã khai).** `prd.md:922` liệt Trần Văn Chánh với trạng thái *"Còn bản quyền · Đã loại"*. Story 1.10c (§Năm quyết định #1, Ice chốt 2026-08-06) đảo quyết định đó: TVC ĐƯỢC dựng, làm lớp gỡ rời thứ ba, với rủi ro pháp lý ghi thẳng vào `attribution` thay vì tránh né bằng cách loại bỏ. `docs/dics/README.md §_khong-dung` cũng còn ghi TVC ở nhóm "đừng dùng" — lý do gốc (`Pleco`/`.xlsx` TRỘN hai từ điển không tách được nguồn) vẫn ĐÚNG cho HAI tệp đó, nhưng KHÔNG áp cho tệp `.tab` chuyên biệt mà story này thực sự dùng (`catusf/tudien` → `dict/Tu-dien-ThienChuu-TranVanChanh.tab`, tự nó là một nguồn ghi công được, kiểm chứng qua `Tu-dien-ThienChuu-TranVanChanh.toml` cạnh nó). **Ai sở hữu đồng bộ lại `prd.md`/`docs/dics/README.md`:** lượt quy hoạch kế tiếp chạm §8.2, hoặc Story 10.4 (màn hình Attribution) khi nó cần bảng nguồn khớp thực tế. **(Chủ: John — PM.)**

- ⚠️ **Tệp thô `Tu-dien-ThienChuu-TranVanChanh.tab` (`catusf/tudien`) thực chất TRỘN hai phong cách nội dung** — một số dòng mang văn phong/cách đánh số Thiều-Chửu-cũ (số khoanh tròn ①②③, từ vựng cổ), một số dòng khác mang văn phong TVC hiện đại (nghĩa tiếng Trung giản thể, ví dụ đương đại) — cùng một ký tự có thể xuất hiện ở CẢ HAI phong cách trên các dòng KHÁC nhau (ca thật: `長`/`行` — xem test `duplicate_headword_lines_stay_as_separate_entries` của `tran_van_chanh.rs`). Story này KHÔNG cố tách hai phong cách đó thành hai nguồn — toàn bộ tệp được ghi công như MỘT nguồn `tran-van-chanh` (đúng thực tế phân phối của `catusf/tudien`, đúng tinh thần tiền lệ `thieu_chuu.rs` không tự suy đoán cấu trúc nội bộ của một tệp thô). Ghi ra để người đọc dữ liệu sau này không ngạc nhiên khi thấy hai văn phong khác hẳn nhau dưới cùng một `source_id`. **(Chủ: story làm mới dữ liệu từ điển tiếp theo.)**

## Deferred from: code review of 1-10c-am-han-viet-dung-nguon-va-dung-nhan (2026-08-06)

- `nom_guard::split_readings` cắt đồng thời trên cả ba quy ước phân tách (`|`, `,`, khoảng trắng) dù chúng là quy ước RIÊNG của ba nguồn khác nhau — deferred, pre-existing design tradeoff đã ghi rõ trong doc-comment (Bẫy 4) của chính module; phạm vi sửa đường ĐỌC thuộc Story 1.16 **(Chủ: Story 1.16.)**, xem module này làm tham khảo. [`tools/dict-build/src/nom_guard.rs:46-56`]
- So sánh âm đọc xuyên nguồn (`nom_guard`) và các parser mới dùng so khớp chuỗi thô, không chuẩn hoá Unicode (NFC/NFD) — rủi ro lý thuyết, chưa có bằng chứng xảy ra trên dữ liệu thật hôm nay; cân nhắc cùng lượt Story 1.16 chuẩn hoá đường đọc âm đọc. [`tools/dict-build/src/nom_guard.rs:108`] **(Chủ: Story 1.16.)**
- Thiều Chửu và Trần Văn Chánh cùng lấy từ `catusf/tudien` nhưng ghim cách nhau ba năm (tag `2.2`/2022-10-10 so với commit 2025-12-19), chưa đối chiếu lại xem hai bản có còn nhất quán — câu hỏi về tính toàn vẹn nguồn, không chặn story 1-10c, cân nhắc khi có lượt làm mới dữ liệu tiếp theo. [`tools/dict-build/src/sources/thieu_chuu.rs:12`; `tools/dict-build/src/sources/tran_van_chanh.rs:37`] **(Chủ: story làm mới dữ liệu từ điển tiếp theo.)**

## Deferred from: 1-16-panel-source-va-tab-han-viet (2026-08-06)

- 🔴 **Vế thị giác hai nền tảng thật (WKWebView macOS · WebView2 Windows) CHƯA đo được** — dải tab, bề mặt song song (`position: absolute` cho `.hv-reading`, xem Debug Log References của story), và `font-synthesis` chữ Hán nghiêng giả chỉ được xác nhận đúng CƠ CHẾ qua Playwright/**headless Chromium** — một engine THỨ BA, không phải một trong hai engine mục tiêu. Dự án `không có runner đo được vế đó` — món nợ cũ (`deferred-work.md:478`, Story 1.6/1.14), story này KHÔNG đóng nó, chỉ kế thừa. Nghiệm thu mắt trên máy thật là bước còn thiếu trước khi đóng dấu "đã kiểm hai nền tảng". **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**
- ⚠️ **AC9 (đổi preset ⇒ không gọi lại IPC) đúng CẤU TRÚC MÃ, chưa đo bằng webview đang chạy.** `ensureChapterLoaded`/`ensureHanVietLoaded` (`src/panels/sourcePanelState.ts`) dùng cờ module-level nên về logic KHÔNG THỂ gọi lại `read_open_chapter`/`read_han_viet` ở lượt mount thứ hai — nhưng phiên dev-story không có một instance `tauri dev` rảnh để tạo Tác phẩm, bấm `Mod+Alt+1`↔`Mod+Alt+2`, và đọc DevTools Network thật. Nghiệm thu tay còn nợ. **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**
- ⚠️ **Trần render kiểu song song (50.000 ký tự Hán) đo trên headless Chromium, không phải WKWebView/WebView2, và không đi qua bộ máy reactivity của Vue** (DOM dựng thẳng `document.createElement`, rẻ hơn Vue một chút vì bỏ VDOM diff). Bảng số ở Completion Notes của story là **cận dưới hợp lý**, không phải con số cuối cùng đã đóng dấu trên hai nền tảng thật — nếu đo lại cho ra số khác đáng kể, hằng `PARALLEL_VIEW_RENDER_CEILING` (`sourcePanelState.ts`) là chỗ sửa. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**
- 📝 **`HanVietLookup.sources_used` mang `dict_source.code` thô** (`fx-hv`, `thieu-chuu`, …), không `display_name` đẹp ("Thiều Chửu"). FR31 (nhãn nguồn bắt buộc) thoả bằng `code`; ánh xạ sang tên hiển thị là việc của màn hình Attribution — **Story 10.4** (đã ghi rõ trong Ranh giới phạm vi của chính story 1.16). Nếu 10.4 cần `display_name` ở đây sớm hơn dự tính, cách rẻ nhất là thêm nó vào `HanVietReading`/`sources_used` qua `layer.source(code)` — hạ tầng đã sẵn (`DictLayer::source`), chỉ chưa nối. **(Chủ: story kế tiếp chạm nhãn nguồn hiển thị (FR31).)**
- 📝 **§Câu hỏi cho Ice #2 (báo hay không báo ký tự nhiều âm) và #3 (hình dạng placeholder ký tự không âm) — dùng MẶC ĐỊNH ĐỀ XUẤT của story, CHƯA được Ice xác nhận lại trong phiên dev-story này.** #2: không đánh dấu gì cho ca nhiều âm (danh sách đầy đủ vẫn đi qua IPC qua `HanVietReading.all`, sẵn cho Story 1.17/3.7). #3: hai chuỗi `vi.json` riêng theo `layersLoaded` (`panel.source.han_viet_unknown`/`han_viet_unavailable`), không dùng `ornament`/`opacity`. Nếu Ice muốn một hướng khác, cả hai đổi được mà không đụng tầng dữ liệu. **(Chủ: Ice — câu hỏi mở #2/#3 ghi ngay trong mục này, chưa được xác nhận.)**

- 🔴 **Kiểu song song CHỒNG CHỮ thật — claim "giãn dòng 2.05 đủ chỗ cho âm đọc" của Task 8 SAI, đo lại lật.** Ice báo lỗi trực tiếp 2026-08-07 (`.hv-parallel` đọc không được, âm đọc đè lên dòng Hán kế tiếp) sau khi story đã Status `done`. Đo lại bằng `getBoundingClientRect`: ở `line-height: 2.05` (token `source-cjk`), `.hv-reading` (`position: absolute; top: 100%`) đè **19,8px** vào dòng sau — chiều cao hộp dòng chỉ do KÝ TỰ quyết, âm đọc không góp một pixel nào nên toàn bộ chiều cao của nó ăn vào dòng kế tiếp, không nằm "trong phần leading" như comment gốc khẳng định.
  → ⚠️ **VÁ LẦN 1 (4.8, neo `top:100%` vào `.hv-unit`) SAI THEO CÁCH KHÁC — Ice bắt lại bằng ảnh chụp thật cùng ngày.** `.hv-unit` kế thừa chính line-height đã giãn (4.8), nên `top:100%` đẩy âm đọc xuống ĐÁY một hộp CAO — âm đọc trôi XA khỏi ký tự của nó, trôi GẦN dòng SAU hơn, đọc như thể thuộc dòng sau. Phép đo `getBoundingClientRect` (chỉ đo độ đè giữa hai dòng) không lộ ra lỗi này.
  → ⚠️ **VÁ LẦN 2 (3.2, neo vào một `.hv-char` mang `line-height: normal`) VẪN CÒN BA LỖI.** Neo đúng dòng rồi, nhưng Ice báo tiếp: ① **vùng tô khi bôi đen trùm cả hộp dòng cao** (trình duyệt tô selection theo hộp dòng, không theo glyph — line-height 3.2 nghĩa là vệt tô cao gấp ba chữ), trải nghiệm rất khó chịu; ② **âm đọc dòng CUỐI bị cắt, cuộn không tới** (`position: absolute` không đẩy `scrollHeight` của `.hv-surface`); ③ `min-width` giãn ô theo độ dài âm làm **chữ Hán rời rạc**, kéo chọn một từ ghép rất khó nhắm.
  → ✅ **ĐÓNG 2026-08-07 (VÁ LẦN 3 — ĐỔI CƠ CHẾ, Ice chốt).** Âm đọc nay đi bằng `<ruby>`/`<rt>` + `ruby-position: under`, bỏ hẳn `position: absolute`. 🔴 **Ràng buộc gốc buộc phải dùng `absolute` đã HẾT HIỆU LỰC**: Story 1.16 chọn nó vì mọi thứ tạo một hộp dòng mới làm Chromium chèn `\n` vào `Selection.toString()` — nhưng `resolveParallel()` nay đọc thẳng node DOM thay vì tin `toString()` (lượt sửa AC12 của story 1.18), nên chuỗi truy vấn không còn phụ thuộc điều đó. Đo lại (Chromium): âm đọc dòng cuối **cách đáy vùng cuộn 31px và cuộn tới được** (② xong) · vùng tô **ôm sát glyph** (① xong) · chữ Hán **liền nhau tự nhiên** (③ xong) · **không đè ở MỌI mức line-height đã thử, kể cả `normal`** ⇒ token `source-cjk-parallel` của hai lượt vá trước trở nên THỪA và **đã được gỡ** — bộ token về lại **17**, `.hv-parallel` dùng chung `source-cjk` với tab thuần. Hai hàng rào AC6/AC12 nay TÁCH ĐÔI, cần cả hai: `<rt>` mang `user-select: none` giữ cho **copy/paste của người dùng** sạch (đo: thiếu ⇒ `"台đài北"`, có ⇒ `"台北"`), còn **truy vấn tra cứu** đi đường `resolveParallel()` đọc node văn bản trực tiếp của `<ruby>` (không `textContent` — nó gộp cả `<rt>`). Khoảng cách giữa hai âm đọc do `padding-inline: var(--space-unit)` trên `<rt>` (dùng lại token 4px sẵn có, KHÔNG thêm hàng vào bảng đóng băng `EXPECTED_SPACING`) — Ice chốt *"chữ hán xa nhau cũng được, âm hán việt không được đè lên nhau"*. ⚠️ **Mọi số đo trên font hệ thống thay thế, chỉ Chromium** (môi trường đo không có `Noto Serif CJK TC`/`Source Serif 4` thật, không có WKWebView) — cần Ice xác nhận bằng mắt trên `tauri dev` thật, cả hai nền tảng. ⚠️ **`ruby-position: under` là thuộc tính có khác biệt engine đã biết** (WebKit từng cần `-webkit-ruby-position`); chưa đo được trên WKWebView.

---

## Deferred from: code review of 1-16-panel-source-va-tab-han-viet (2026-08-06)

- ✅ **ĐÓNG cùng lượt code review — đã đo bổ sung, kết luận không lật.** `buildSegments`+`switchText` đo được **2,5 / 17,5 / 237,5 ms** ở 5k/50k/500k ⇒ kiểu chuyển đổi ở 500k là **~460 ms** *(bảng cũ ghi 222,4 ms — bỏ sót quá nửa)*, vẫn rẻ cho một thao tác chạy MỘT LẦN ⇒ *"chuyển đổi không có trần"* **vẫn đúng**, trần song song **50.000 giữ nguyên**. Bản vá `min-width` chỉ tốn **+2,7 %** ở 50k. Số đầy đủ ở §Review Findings của story 1.16. ~~Phép đo Task 8 không đo đường mã thật.~~ Bảng số trần render dựng DOM bằng `document.createElement`, không đi qua `buildSegments()` (một object JS cho **mỗi** ký tự Hán) lẫn `switchText` (`.join()` trên toàn bộ mẩu) — mà cả hai **luôn chạy ở CẢ HAI kiểu xem**. ⇒ mệnh đề *"kiểu chuyển đổi không có trần"* (222,4 ms ở 500k) và hằng `PARALLEL_VIEW_RENDER_CEILING = 50_000` đứng trên một phép đo **sai đối tượng**. Cần đo lại trên component Vue thật trước khi tin con số trần. *(Khác với món nợ engine WKWebView/WebView2 — món đó dev đã ghi rõ và trung thực.)*
- **Trần render chỉ đếm ký tự Hán, bỏ qua node của mẩu không-Hán.** `buildSegments` `flush()` mỗi lần gặp một ký tự Hán ⇒ mỗi mẩu không-Hán xen giữa cũng sinh một `<span>` riêng. Văn bản xen kẽ `漢a漢a…` với 49.999 ký tự Hán lọt qua trần nhưng dựng ~100.000 node, trong khi bảng đo chỉ đo văn bản Hán liền mạch. Nhặt lại cùng lượt đo lại ở mục trên. **(Chủ: story kế tiếp chạm `buildSegments`.)**
- **`source_lang` không được validate ở tầng ghi.** `create_work_from_text`/`create_work_from_file` chèn giá trị nguyên văn vào `work`, không một phép kiểm nào; `SourcePanel.vue` so `=== 'zh'` chính xác từng byte. Bất kỳ đường ghi nào khác (`"ZH"`, `"zh-Hans"`, `"cmn"`, hay một `.atproj` chép từ máy khác) cho một Tác phẩm tiếng Trung **không có tab Hán Việt**, không lỗi, không cách nào biết vì sao. Guard đúng nằm ở tầng ghi (Story 1.15), không ở so sánh chuỗi phía UI. **Có sẵn từ trước Story 1.16.** **(Chủ: story kế tiếp chạm `create_work_from_text`/`create_work_from_file`.)**
- **`read_open_chapter` với 0 Chương trả lỗi KHO thay vì lỗi có tên.** `conn.query_row(…)` ném `QueryReturnedNoRows` khi bảng `chapter` rỗng ⇒ qua `From<StoreError>` thành `store.read_failed` ⇒ người dùng đọc *"không mở được kho dữ liệu"* cho một Tác phẩm hoàn toàn lành lặn. AC8 dựng riêng `project.no_work_open` để không trộn trạng thái sản phẩm vào từ vựng `store.*`, nhưng chỉ phủ nhánh `open == None`, không phủ nhánh `open == Some` mà 0 hàng. 🔴 **Epic 1 luôn ghi đúng một Chương nên hôm nay chưa chạm tới; Story 2.x (chọn/chuyển Chương) mở đúng nhánh này.** Guard: `query_map().next()` + một `MessageKey` riêng. *(`src-tauri/src/commands/chapter.rs:60-71`)*
  → ✅ **ĐÓNG 2026-08-18 (Story 2.11, Task 1.3)** — đúng story được giao đích danh. Vá bằng `query_map().next().transpose()` rồi đổi `None` thành một lỗi **có tên**: `chapter_not_found(chapter_id)` với `MessageKey::SegmentChapterNotFound` *(`commands/chapter.rs`)*. 🔵 **KHÔNG một khoá thứ hai, và đó là kết luận chứ không một lượt bỏ qua:** khoá ấy đã khai đúng tham số `["chapter_id"]` *(`core/i18n/mod.rs:174`)* và nói **cùng câu, cùng nghĩa** — hai khoá cho một câu là hai chuỗi phải giữ khớp bằng kỷ luật, đúng lập luận mà `no_work_open` đã đi qua hai lần. Ca nghiệm thu: `a_missing_chapter_row_is_a_named_error_not_a_store_error` *(`project_contract.rs`)*, đã chạy **đỏ-rồi-xanh** bằng một phép đột biến trả `query_row` về chỗ cũ.
  ⚠️ **Ghi ra thay vì để người sau tưởng đã phủ hết:** cùng lượt này, `read_open_chapter` bỏ hẳn câu `ORDER BY ord LIMIT 1` và đọc `OpenWork::chapter_id`, nên nhánh *"bảng `chapter` rỗng"* nay đi vào **cùng** một guard với nhánh *"hàng được chỉ đã bị xoá"* — hai hoàn cảnh, **một** câu. Chấp nhận vì cả hai đều là *"Chương này không có ở đây"* theo nghĩa đen của người dùng; tách chúng ra đòi một khoá thứ hai cho một phân biệt mà màn hình không dùng được.

- **Bôi đen nguyên văn bằng BÀN PHÍM — điều kiện tiên quyết của Story 1.18, CHƯA CÀI.** §KHÔNG-LÀM ① của Story 1.16 giao cho story đó **đúng một** nghĩa vụ: nguyên văn phải bôi đen được *bằng chuột **và bằng bàn phím***. Vế chuột đã đo thật bằng Playwright (`Selection.toString()`); vế bàn phím không xuất hiện một lần nào trong Tasks/AC/Completion Notes của 1.16. 🔴 **Ice chốt 2026-08-06 ở lượt code review: ghi nợ cho 1.18** — lý do: thêm `tabindex` lên bề mặt văn bản đụng hợp đồng tiêu điểm mà Story 1.14 dặn không chạm, nên vế bàn phím phải đóng **cùng lượt** với hợp đồng vùng chọn dùng chung cho bốn panel. ⚠️ Một `<div>` không sửa được **không** hỗ trợ Shift+Mũi tên nếu không bật caret browsing — Story 1.18 phải giải bài này, không giả định trình duyệt cho không.
  → ✅ **ĐÓNG 2026-08-07 (Story 1.18, AC11 · Quyết định #2a).** Năm command mới trong `CommandRegistry` (`selection.focus_source` · `extend_left` · `extend_right` · `extend_word_left` · `extend_word_right`), cài bằng `Selection.modify()` trên bề mặt mang `tabindex="0"`. 🔴 **ĐO THẬT trên CẢ HAI engine trước khi chốt** (Task 0, WKWebView qua một bộ đo Swift + Chromium headless): `modify()` chạy trên `<p>` không sửa được ở cả hai, và **Bẫy 9 của story không CÓ THẬT** — `'word'` trên văn xuôi tiếng Trung phân đoạn ĐÚNG (`他` / `打開`), không nuốt cả câu. Giá đã trả và đã ghi: `Tab` nay dừng ở thân Panel Source (Ice chốt chấp nhận 2026-08-07). ⚠️ **Một món nợ MỚI sinh ra**, xem mục `repeat` ở §1.18 bên dưới.


- **Không token nào đỡ được một câu GIAO DIỆN xuống dòng** — cả sáu token `ui-*` *(`ui-md` · `ui-md-strong` · `ui-sm` · `ui-label` · `ui-mono` · `read-title`)* đều khai `wraps: false`, giãn dòng 1.4–1.5, dưới sàn 1.66; còn `check-tokens.mjs` chỉ áp `LINE_HEIGHT_FLOOR` cho token khai `wraps: true` nên cổng **mù hoàn toàn**. Ba chỗ đang chịu: `.parallel-note` *(có sẵn)*, `.load-error` và `.hv-notice` *(thêm ở lượt code review 2026-08-06)* — cả ba là câu đầy đủ, chắc chắn xuống dòng trong một panel hẹp. 🔴 **Đây là lỗ hổng của BẢNG TOKEN, không phải của chỗ dùng** — cùng hạng với hàng `source-latin` còn thiếu mà Quyết định #6 của Story 1.16 vừa vá. Đóng nó là **quyết định của Ice**: đổi cờ `wraps` của `ui-md` *(mục `:115` ngay trên — đây là lần thứ BA nó bị nhắc tên)* hoặc thêm một token thứ 17 qua sổ `deviations`. Lượt code review GHI RA thay vì tự chế một token.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.17, Quyết định #7) — lần thứ TƯ bị gọi tên, lần này Ice chốt hẳn.** Token thứ 17 `ui-md-wrap` (12px/1.66/`wraps:true`), áp cho cả ba chỗ liệt kê ở đây. Xem mục `:115` để có chi tiết đầy đủ.

## Deferred from: 1-17-panel-lookup-ban-ghi-co-cau-truc (2026-08-06)

- 🔴 **`QueryBranch::NoBranchQueryTooShort`/`"query_too_short"` KHÔNG thể xảy ra qua đường sản phẩm thật của chính story vừa dựng nó.** AC6 đòi Panel Lookup hiện chuỗi *"truy vấn quá ngắn"* khi `branch == query_too_short`, và bề mặt (`LookupPanel.vue::queryTooShort`) render đúng nhánh đó — nhưng `commands::dict::lookup()` cố định `LookupMode::Exact` (Quyết định #3), và `pick_branch` cho `Exact` **luôn luôn** trả `ExactBtree` bất kể độ dài truy vấn, bất kể route. Nhánh `query_too_short` chỉ sinh ra khi `mode = Substring` **và** route `En` **và** độ dài < 3 — tổ hợp đó không tồn tại trong bất kỳ lời gọi nào của 1.17. ⇒ Bề mặt UI đúng, đã viết, đã kiểu-khớp với wire — nhưng **chưa từng và không thể được thực thi bằng dữ liệu thật cho tới khi có một chỗ gọi dùng `Substring`**. **Chủ: Story 1.18** (Auto-Lookup, dùng `Substring` khi bôi đen ngắn) hoặc **7.7** (Concordance) — story đầu tiên gọi `LookupMode::Substring` qua IPC phải verify bằng mắt chuỗi "truy vấn quá ngắn" thật sự hiện ra, đừng giả định 1.17 đã làm việc đó.
  → ✅ **ĐÓNG 2026-08-07 (Story 1.18).** Ice chốt bật `Substring` (§Câu hỏi #1) — cài như một **ĐƯỜNG LUI**, không một phép thay thế: tra `Exact` trước, **rỗng** *và* truy vấn ≤ 4 ký tự ⇒ tra lại `Substring`. Tổ hợp `mode = Substring` + route `En` + độ dài < 3 nay **tồn tại trên đường sản phẩm**. Nghiệm thu: `dict_sources.rs::a_short_latin_selection_now_reaches_the_query_too_short_state` (`"zz"` ⇒ `NoBranchQueryTooShort`), **cộng** phép đo trên bốn lớp THẬT — bench `bench_the_auto_lookup_path_on_distinct_queries` đếm **2 lượt `query_too_short`** trên 166 truy vấn khác nhau. không còn là một nhánh chỉ tồn tại trên giấy.


- 🔴 **Vòng IPC Tauri THẬT (serialize Rust → cầu JS → deserialize → Vue reactivity → paint) CHƯA được đo** — cùng hạng món nợ *"vế thị giác hai nền tảng thật"* mà Story 1.6/1.14/1.16 đã để lại, story này **KHÔNG đóng, chỉ kế thừa**. Số đo NFR1 của story dựa trên: (a) backend Rust trên dữ liệu thật (`--release`, đáng tin — p95 6,535 ms), (b) webview render qua Playwright/**headless Chromium** với `invoke` **giả lập trả lời tức thời** (không đo độ trễ round-trip IPC thật). Kết luận NFR1 ĐẠT có cơ sở mạnh (tổng ước tính < 40 ms, cách trần 100 ms một biên độ lớn) nhưng **KHÔNG phải một phép đo đầu-cuối 100% trên WKWebView/WebView2 qua `tauri dev`/bản đóng gói thật**. Xem §Debug Log References của story để có bảng đầy đủ + giới hạn phép đo ghi thẳng. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**

- ⚠️ **`.parallel-note` (Panel Source, Story 1.16) đổi cỡ chữ 11,5px → 12px** khi chuyển từ `ui-sm` sang `ui-md-wrap` (Quyết định #7). Đây là một thay đổi THỊ GIÁC trên một bề mặt đã ship từ Story 1.16, không chỉ thêm token cho chuỗi mới — Ice đã chốt chấp nhận đổi cỡ để đóng dứt điểm `deferred-work.md:115` thay vì rải thêm một token `ui-sm-wrap`. **Chưa nghiệm thu bằng mắt trên máy thật** (chỉ Playwright headless) rằng 0,5px đổi cỡ không làm vỡ bố cục dải tab của `SourcePanel.vue` ở màn hình hẹp — nhặt lại nếu Story 4.12 (bố cục màn hình hẹp) phát hiện vấn đề. **(Chủ: story kế tiếp dựng panel chạm `.parallel-note`.)**

- 📝 **Mục từ TIẾNG ANH của Panel Lookup dùng hình dạng TẠM** — nhắc lại mục `:317` (chủ sở hữu Sally, `bmad-ux`): `LookupRecord.vue` dùng **cùng cấu trúc khối** cho tiếng Anh và tiếng Trung (chỉ khác token đầu mục), một lựa chọn tự chế ở tầng story mà mục `:317` tự cảnh báo là "đúng cách một bất nhất giao diện ra đời". **KHÔNG ĐÓNG** — chữ ký UX chính thức vẫn thiếu. **(Chủ: Sally — bmad-ux, xem mục `:317`.)**

- 📝 **`commands::dict::lookup()` không có nhánh lỗi riêng khi `layer.senses(&entry_ids)` trượt** — nếu pha hai hydrate thất bại cho một lớp (lỗi đọc SQLite giữa chừng, hiếm), `unwrap_or_default()` âm thầm trả danh sách nghĩa RỖNG cho lớp đó thay vì báo lỗi hay đưa vào `skipped`/`truncated_layers`. Pha một của lớp đó đã THÀNH CÔNG (nó nằm trong `groups`), nên panel sẽ hiện đúng nhóm nguồn nhưng KHÔNG nghĩa nào — trông giống một đầu mục "chỉ có âm đọc, không nghĩa" (trạng thái hợp lệ theo `senses.rs`) chứ không giống một lỗi. Rủi ro thấp (pha một vừa đọc được từ đúng tệp đó) nhưng chưa có tín hiệu phân biệt hai ca. Nhặt lại nếu có báo cáo thật về hiện tượng "nguồn hiện tên mà không có nghĩa nào". **(Chủ: một story kế tiếp chạm `core/dict` (`commands::dict::lookup()`).)**

## Deferred from: code review of 1-17-panel-lookup-ban-ghi-co-cau-truc (2026-08-06)

- 📝 **Chip thanh nhịp bị `overflow: hidden` cắt CÂM khi nhiều nguồn** (`src/panels/LookupPanel.vue:109-113,131-137`) — `.lookup-head` khoá `height: 76px` + `overflow: hidden` để giữ bất biến AC7, còn `.lookup-spine` là `flex-wrap: wrap`. Đo thật cho `山` ra **7–8 nhóm** ⇒ chip tràn sang dòng thứ ba trở đi bị cắt mất hoàn toàn: không dấu hiệu, không cuộn, không chỉ báo `+N`. Đây là đánh đổi CÓ CHỦ ĐÍCH đã ghi trong chú thích tại chỗ (giữ chiều cao bất biến quan trọng hơn), nhưng hệ quả "tên nguồn biến mất" thì chưa ai quyết. Nhặt lại cùng Story 4.12 (bố cục màn hình hẹp) hoặc khi thanh nhịp có chủ sở hữu UX thật. **(Chủ: story kế tiếp chạm `LookupPanel.vue` (thanh nhịp).)**

- 📝 **`layers_loaded = false` khi MỌI tệp `.db` hỏng ⇒ panel hiện "chưa gắn lớp từ điển nào" — một chẩn đoán SAI** (`src-tauri/src/core/dict/layer.rs:460-493`) — `DictLayers::new` đẩy mọi lớp mở-hỏng vào `skipped` chứ không vào `layers`, nên `layers_loaded: !layers.layers().is_empty()` cho `false` cả khi thư mục ĐẦY tệp `.db` hỏng. AC6 dựng chuỗi đó riêng cho ca "thư mục rỗng" (AD-25). Rủi ro thấp: banner `someLayerFailed` vẫn hiện song song nên người dùng không bị bỏ câm, chỉ đọc được hai câu hơi lệch nhau. Sửa gọn: `!layers().is_empty() || !skipped().is_empty()`. **(Chủ: story kế tiếp chạm `core/dict/layer.rs`.)**

- 📝 **Nhánh `Substring`/`fts_trigram` nạp TOÀN BỘ hàng khớp vào RAM trước khi cắt** (`src-tauri/src/core/dict/query.rs:203-221,238-254,321-333`) — ba nhánh cần `verify_substring` cố tình fetch không giới hạn rồi `cap()` ở Rust để tránh Bẫy 11, nên `limit` không chặn được bộ nhớ lẫn độ trễ, chỉ chặn băng thông IPC. Hôm nay là **latent**: đường sản phẩm 1.17 là `Exact`-only nên không chạm tới ba nhánh này. Trở thành thật khi **Story 1.18/7.7** bật `Substring`. Hướng sửa giữ đúng thứ tự "verify rồi mới `cap`": thêm một trần AN TOÀN ở SQL (vd `LIMIT limit * 50`) làm cận trên cho tập ứng viên.
  → ✅ **ĐÓNG 2026-08-07 (Story 1.18).** `candidate_ceiling(limit) = limit * 50` đặt vào SQL ở cả **ba** nhánh có xác minh. 🔴 Phần tinh tế mà mục này không nêu: khi trần chạm, `verify_substring` có thể loại đủ nhiều dương tính giả để phần còn lại **ít hơn** `limit`, và khi đó `cap()` một mình báo `truncated = false` — đúng câu *"danh sách này đầy đủ"*, và nó SAI. ⇒ `cap_verified()` OR cờ trần vào. Nghiệm thu đỏ-rồi-xanh: `dict_lookup.rs::the_candidate_ceiling_keeps_the_truncated_flag_honest` (60 ứng viên, **0** qua được verify ⇒ `truncated` phải `true`), chứng minh ĐỎ bằng cách nâng hệ số lên 100.000.


- 📝 **Chuỗi `query_too_short` chỉ dẫn một thao tác không TỒN TẠI trong panel** (`src/i18n/vi.json:65`) — *"gõ thêm ít nhất ba ký tự"*, nhưng Panel Lookup không có ô nhập nào: truy vấn chỉ đến từ vùng chọn (`main.ts:182`). Cộng với việc chính story tự khai nhánh `query_too_short` không thực thi được qua đường sản phẩm `Exact`-only hôm nay, đây là một chuỗi vừa không hiện được, vừa vô nghĩa nếu hiện. **Story 1.18/7.7** sẽ kế thừa nguyên văn nó — sửa lúc bật `Substring`, cùng lượt với hợp đồng vùng chọn.
  → ✅ **ĐÓNG 2026-08-07 (Story 1.18).** Chuỗi đổi thành *"Đoạn đang chọn quá ngắn để tra chuỗi con — **chọn** thêm ít nhất ba ký tự."* — nay chỉ một thao tác CÓ THẬT (bôi đen thêm), không một ô nhập không tồn tại.


- 📝 **`window.getSelection()` mù với `<input>`/`<textarea>`; vùng chọn rỗng = im lặng tuyệt đối** (`src/main.ts:182`, `src/commands/index.ts:475-477`) — trên Chromium/WebKit, `window.getSelection().toString()` trả `''` cho vùng chọn BÊN TRONG một ô nhập, nên bấm `Mod+Alt+L` khi con trỏ ở ô nhập của Library trông y hệt một phím tắt hỏng; và `text.trim() === ''` trả về sớm không phản hồi nào. **Story 1.18** sở hữu hợp đồng vùng chọn dùng chung — dep hôm nay là dep TỐI THIỂU theo đúng Quyết định #1a, nên hai vế này đóng cùng lượt đó chứ không ở đây.
  → ✅ **ĐÓNG 2026-08-07 (Story 1.18, AC3 · Task 1).** `src/panels/selectionContract.ts` — sổ đăng ký **OPT-IN** theo phần tử, một listener trên `document`. 🔴 **VÀ PHÉP ĐO LẬT LÝ DO CỦA CHÍNH MỤC NÀY:** `getSelection().toString()` trong một `<input>` trả **`"nội "`** (văn bản THẬT) trên **cả** Chromium **lẫn** WKWebView — không phải `''`. Lời khuyên của mục này đúng, lý do thì sai. Phép loại trừ đã cài không dựa vào chuỗi rỗng lẫn `document.activeElement` (đo được: `activeElement` cho **âm tính giả** khi tiêu điểm ở ô nhập mà vùng chọn nằm nơi khác) — nó đọc **`anchorNode.nodeType`**: một vùng chọn chữ thật luôn neo vào node VĂN BẢN, vùng chọn trong ô nhập neo vào node PHẦN TỬ. Phân biệt sạch trên cả hai engine.


## Deferred from: 1-18-auto-lookup (2026-08-07)

- 🔴 **`Selection.modify()` đi XUYÊN QUA `user-select: none` trên WKWebView — và story này đã vá chỗ dùng, không vá được nguyên nhân.** Đo 2026-08-07, vùng chọn cả đoạn ở kiểu song song: `Selection.toString()` cho `他打開了那扇門，走進了黑暗之中。` trên Chromium *(đúng)* nhưng `他tha打đả開khai了liễu…` trên WKWebView — tức **rò âm Hán Việt vào truy vấn**. `user-select: none` chi phối vùng chọn do **chuột kéo** (số đo Playwright của Story 1.16 vẫn đúng); nó **không ràng buộc** `Selection.modify()`, mà `modify()` chính là đường bàn phím AC11 vừa dựng. ⇒ `SourceHanViet.vue::resolveParallel` nay đọc thẳng node `.hv-char` thay vì tin `toString()` — đúng trên cả hai engine. ⚠️ **Cái không đóng:** mọi bề mặt TƯƠNG LAI dùng `user-select: none` để loại chữ khỏi vùng chọn *(Story 3.4 — đánh dấu thuật ngữ Glossary; Epic 2 — Editor)* thừa hưởng nguyên cái bẫy này, và không cổng nào canh. Cân nhắc một vị từ dùng chung ở `selectionContract.ts` khi bề mặt thứ hai xuất hiện. **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

- ⚠️ **Giữ phím không mở rộng vùng chọn liên tục — phải bấm lặp** (`src/commands/keys.ts:295`). `handle()` trả sớm khi `event.repeat === true`, một luật đúng cho 17 command cũ (*lặp lại "đổi chế độ" là vô nghĩa*) và sai cho đúng **bốn** command `selection.extend_*` của story này. Nới nó cần một cờ `repeatable` trên `CommandSpec`, chạm `registry.ts` + `keys.ts` + **mọi** command đang có ⇒ không thuộc phạm vi 1.18. Hai command **theo TỪ** (`Alt+Shift+←/→`) bù phần lớn chi phí thao tác. **Chủ: Story 1.21** (màn hình gán phím — nó vốn phải mổ tầng này).
  → ✅ **ĐÃ ĐÓNG 2026-08-11 (Story 1.21).** Cờ `repeatable?: boolean` trên `CommandSpec`, mặc định **không**; `frozen()` chuẩn hoá nó về `boolean` ở cửa vào; `keys.ts::handle` đọc `event.repeat === true && !entry.repeatable`. **Bốn** chỗ khai `true` — đúng bốn `selection.extend_*` — và không chỗ nào khác.
  🔴 **Ice ký NHẬN món nợ này 2026-08-11, và việc đó là một quyết định có chủ, không một sự trôi phạm vi.** Story 1.21 đề xuất **trả lại** cả ba món nợ mang tên nó vì cả ba có **0 AC** ở `epics.md`; Ice lật một phần, nhận đúng cái này. Cái giá đã nói trước và Ice ký nhận: nó chạm `registry.ts` + `keys.ts` + mọi command đang có, cho một thay đổi không AC nào yêu cầu.
  ⚠️ Lưới: Kiểm D của `check-commands.mjs` có **hai** khẳng định — nhánh dương *(`repeatable: true` ⇒ keydown lặp VẪN dispatch)* và **đối chứng âm** trên **cùng một keymap** *(command không khai cờ ⇒ keydown lặp vẫn bị chặn)*. Không có vế thứ hai thì một bản cài đặt bỏ quên cờ hoàn toàn vẫn xanh.

- 🔴 **NFR1 đo được ĐẦU-CUỐI ở tầng Rust, không qua vòng IPC Tauri thật lẫn lượt VẼ của webview** — món nợ Story 1.17 để lại, story này **KẾ THỪA, không ĐÓNG**. Số đã đo (4 lượt độc lập, `--release`, 4 lớp `.db` thật, 166 truy vấn KHÁC NHAU, đường sản phẩm `commands::dict::lookup` gồm cả đường lui `Substring`): trạng thái ổn định **p50 ~1,0 ms · p95 1,8–2,4 ms · p99 5,5–9,8 ms · max 20–25 ms**. Cơ chế đo đầu-cuối THẬT đã cài và bật được tay (`src/panels/lookupTiming.ts`, cờ mặc định TẮT, `__auraLookupTiming.enable()`), mốc cuối sau `requestAnimationFrame` — nhưng nó **chưa được chạy trong một bản Tauri đóng gói**, nên vế *"từ lúc thả chuột tới lúc hiển thị"* vẫn là **ước lượng**, không một phép đo. Đánh dấu AC4 đạt trọn. **(Chủ: story kế tiếp đo NFR1 đầu-cuối qua IPC Tauri thật.)**

- ⚠️ **Vế thị giác của story không nghiệm thu trên hai nền tảng thật** — cùng hạng món nợ mà 1.6/1.14/1.16/1.17 để lại, **kế thừa không đóng**: hiệu ứng 90 ms, vạch tiến trình 250 ms, `prefers-reduced-motion`, và điểm dừng `Tab` mới đều mới chỉ chạy qua `vue-tsc` + cổng tĩnh + hai bộ đo engine rời, không qua mắt người trên `tauri dev` (macOS) lẫn một máy Windows. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**

- 📝 **Phép kiểm AC12 chạy bằng `Selection.modify()`, không bằng một cú KÉO CHUỘT thật.** `modify()` là thuật toán chọn của chính trình duyệt nên nó là bản mô phỏng gần nhất mà một trang tĩnh dựng được, nhưng nó không **là** một lượt kéo. Story 1.16 đo vế chuột bằng Playwright; story này không thêm phụ thuộc nào (NFR15) nên không chạy lại được vế đó. Hai vế cộng lại phủ đủ ý định của AC12, và khoảng trống ghi ở đây thay vì để người sau tự phát hiện. **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**

- 📝 **Trần đường lui `SUBSTRING_FALLBACK_CEILING = 4` chưa có số đo hành vi người dùng đỡ lưng** (`src-tauri/src/commands/dict.rs`). Con số dựng trên một lý lẽ ngôn ngữ (*một thành ngữ tiếng Trung là bốn ký tự — đơn vị dài nhất còn đáng tra như chuỗi con*), không trên nhật ký bôi đen thật. Bench đo được **78/166** truy vấn đi qua đường lui, tức nó không phải một nhánh hiếm. Nhặt lại khi có dữ liệu dùng thật, hoặc ở **Story 7.7** (Concordance — chủ thật sự của `Substring`). **(Chủ: story kế tiếp đo hành vi người dùng thật trên `SUBSTRING_FALLBACK_CEILING`.)**

## Deferred from: code review of 1-18-auto-lookup (2026-08-07)

- 📝 **Bộ đếm Kiểm F (`scripts/check-commands.mjs`, AC2) đọc `p.masked`, và `maskScript`/`maskTemplate` chỉ che comment (`//`, `/* */`, `<!-- -->`), KHÔNG che nội dung chuỗi literal/template literal.** Một chuỗi giả dạng lời gọi (vd một dòng văn xuôi/chuỗi lỗi chứa nguyên văn `"useSelectionSurface(original, 'source')"`) vẫn bị đếm là một lượt đăng ký thật, dù không có lời gọi nào. Đây là đặc tính CHUNG của mọi cổng regex trong tệp này (không riêng Kiểm F) — vá đúng nghĩa là đổi hành vi `maskScript`/`maskTemplate` toàn cục, ngoài phạm vi story 1.18. Nhặt lại nếu có một lượt hardening riêng cho `check-commands.mjs`, hoặc nếu một ca thật (không phải giả định) từng lọt qua cổng theo đường này. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- 📝 **`SURFACE_CALL_RE` không khớp dạng gọi thay thế** — gọi trực tiếp `registerSelectionSurface(...)` thay vì qua `useSelectionSurface(...)`, đối số đầu chứa dấu phẩy (vd `pick(a, b)`), hoặc `role` viết sai hoa/thường (`'Source'`) đều không được đếm đúng. Chưa có ca thật nào trong mã hôm nay dùng các dạng đó — mọi panel đều gọi `useSelectionSurface(ref, 'source'|'display')` literal, đúng quy ước. Cùng lớp giới hạn với các cổng regex khác trong tệp (NFR15 cấm phụ thuộc một bộ phân tích cú pháp thật). Nhặt lại nếu một story sau đổi cách gọi. **(Chủ: một story hạ tầng cổng kế tiếp.)**

- 📝 **`SELECTION_PANEL_FILES` (`scripts/check-commands.mjs:1627`) là danh sách chép tay từ `src/layout/workspaceLayout.ts`, không tự đồng bộ khi Workspace có panel văn bản mới.** Một panel mới (vd Story 3.4 — Glossary) mà không được thêm tay vào danh sách này sẽ không bị Kiểm F đòi đăng ký — cùng lớp lỗi "sổ đăng ký không tự cập nhật" mà AD-34 §2 (`FOCUS_OWNERS`) tồn tại để chặn ở chỗ khác, nhưng ở đây chưa có một đối chiếu hai chiều. Cùng khuôn với `PANEL_SUFFIXES` đã dùng nơi khác trong tệp — không phải một quy ước mới của story này. Chủ: story nào thêm panel văn bản mới tiếp theo.

## Deferred from: nghiệm thu tay tab Hán Việt (Ice, 2026-08-07)

> Bối cảnh: sau khi Story 1.18 đóng, Ice chạy thật và báo một chuỗi lỗi thị giác ở tab Hán
> Việt. Phần lớn đã VÁ XONG trong cùng phiên *(âm đọc chuyển sang `<ruby>`; số dính liền âm;
> cỡ chữ 12,5→14,5px; bỏ nghiêng — xem mục §1-16 ở trên và `deviations` trong `tokens.json`)*.
> Mục dưới đây là phần **KHÔNG vá trong phiên**, đã đo đủ để giao thành một story riêng.

- 🔴 **DOUBLE-CLICK ở tab Hán Việt chỉ chọn được MỘT ký tự / MỘT âm, trong khi ở tab nguyên
  văn tiếng Trung nó chọn đúng CẢ CỤM TỪ.** Ice báo 2026-08-07: *"ở phần văn bản gốc thì
  double click sẽ chọn được cả cụm từ, vậy tại sao khi chuyển đổi sang phần hán việt lại
  không chọn được, nó phải nên được xử lý theo phần văn bản gốc chứ"*. Hệ quả thật: tra một
  từ ghép ở tab Hán Việt phải KÉO CHỌN thủ công từng lần, trong khi tab bên cạnh chỉ cần bấm
  hai phát — một bất đối xứng người dùng không có lý do gì để chấp nhận.

  🔴 **PHÂN TÍCH ĐẦU TIÊN CỦA DEV SAI, VÀ ICE LẬT NÓ.** Dev kết luận *"phải tự xây tách từ,
  một story lớn, cần bộ tách từ ở Rust"*. Sai: bộ tách từ **đã có sẵn trong engine** — nó
  chính là thứ làm double-click chạy đúng ở tab nguyên văn. Truy được qua **`Intl.Segmenter`**,
  nội tại của JS engine ⇒ **0 phụ thuộc mới, NFR15 giữ nguyên**, và **0 dòng Rust**.

  **SỐ ĐO ĐÃ CÓ — story sau đừng đo lại từ đầu** *(Chromium, 2026-08-07)*:

  | Đo | Kết quả |
  |---|---|
  | `Intl.Segmenter('zh',{granularity:'word'})` có mặt | ✅ có |
  | Tách câu mẫu | `台湾 / 地方 / 议会 / 接连 / 通过 / 提案 / ， / 反对 / 中共 / 跨 / 境 / 镇压 / 。 / 北市 / 议会 / 8 / 月 / 5 / 日` — ĐÚNG |
  | Kiểu **song song**: một `<ruby>` mỗi KÝ TỰ *(hiện tại)* | double-click ⇒ `""` — **hỏng hoàn toàn** |
  | Kiểu **song song**: một `<ruby>` mỗi **TỪ** (`台湾`+`thai loan`) | double-click ⇒ `台湾` ✅ |
  | Kiểu **chuyển đổi**: âm cách nhau bằng dấu cách *(hiện tại)* | double-click ⇒ `thai` — một âm |
  | Kiểu **chuyển đổi**: nối âm trong cùng từ bằng `U+2060`, khe hở vẽ bằng CSS `margin` | double-click ⇒ `thai⁠loan` ✅ |
  | `U+00A0` · `U+2009` *(thin space)* làm chất nối | ❌ vẫn cắt ở dấu cách |

  ⚠️ **Chi tiết dễ cài sai:** `U+2060` rộng **bằng 0**, nên dùng nó THAY dấu cách sẽ làm chữ
  dính (`thailoan`). Đường chạy được là **tách hai vai**: `U+2060` giữ *tính liền từ* cho
  trình duyệt, còn *khoảng cách nhìn thấy* do CSS vẽ (`margin` trên từng âm). Đã đo đúng.

  **Phạm vi thật của story — đây KHÔNG phải một lượt sửa CSS:**
  - `.hv-switch` từ **một text node thuần** thành có cấu trúc span ⇒ **`resolveSelection()`
    phải viết lại**: nó đang đòi `range.startContainer` chính là text node duy nhất, và đang
    ánh xạ ngược bằng bảng `switchView.map`/`starts` dựng theo offset của chuỗi phẳng đó.
  - `.hv-parallel` từ một `<ruby>` mỗi ký tự sang một `<ruby>` mỗi **từ** ⇒ `buildSegments()`
    và `resolveParallel()` đổi theo *(segment `han` nay mang một CỤM, không một ký tự)*.
  - 🔴 **AC6 (Story 1.16) và AC12 (Story 1.18) canh đúng bề mặt này** — cả hai phải **đo
    LẠI** sau khi đổi, không suy từ số đo cũ: chuỗi copy/paste của người dùng (`<rt>` +
    `user-select:none`) và chuỗi truy vấn tra cứu (`resolveParallel`/`resolveSelection` đọc
    node) là **hai đường riêng**, cần kiểm cả hai.
  - Lợi ích kèm theo, không phải mục tiêu chính: gom âm theo TỪ làm âm đọc bám đúng cụm
    (`thai loan` nằm dưới `台湾`), đọc tự nhiên hơn hẳn so với rải đều theo ký tự.

  ⚠️ **HAI THỨ CHƯA ĐO ĐƯỢC, đừng khai đạt nếu chưa làm:** ① `Intl.Segmenter` trên
  **WKWebView** *(có từ Safari 14.1 nên nhiều khả năng có, nhưng môi trường đo chỉ có
  Chromium — cùng món nợ hai nền tảng mà 1.6/1.14/1.16/1.17/1.18 đã để lại)*; ② **chất lượng
  tách từ trên văn bản TIỂU THUYẾT thật** — mẫu đã đo là văn bản tin tức, và tiểu thuyết mang
  tên riêng, từ cổ, thành ngữ mà ICU có thể cắt khác.

  **Chủ: một story riêng — Ice chốt 2026-08-07** *(*"đồng ý làm thành một lượt riêng có đo
  lại AC6/AC12 đàng hoàng"*)*.

  ✅ **ĐÓNG 2026-08-07 bởi Story 1.18b** (`1-18b-tach-tu-tieng-trung-tab-han-viet`).
  `src/panels/wordBoundary.ts` mới · `buildSegments`/`resolveParallel`/`resolveSelection` viết
  lại · **0** phụ thuộc mới, **0** dòng Rust. Nghiệm thu: bảng đối chiếu **26 vị trí** tab
  Trung ↔ tab Hán Việt bằng **double-click THẬT**, **26/26 khớp** ở *cả hai* kiểu xem.
  Ba mệnh đề sống đã đo LẠI trên cấu trúc mới (AC6/1.16 · AC11+AC12/1.18) — xem §Debug Log
  References của story.

---

## Deferred from: 1-18b-tach-tu-tieng-trung-tab-han-viet (2026-08-07)

- 🔴 **`Intl.Segmenter` trên WKWebView vẫn CHƯA đo** — cùng món nợ hai nền tảng mà
  1.6/1.14/1.16/1.17/1.18 để lại, nay thêm một mặt hàng. Story 1.18b đo trọn trên **Chromium**
  và **không** dựng được bản Tauri thật. Ba thứ phải đo lại khi có máy macOS dựng được:
  ① `Intl.Segmenter` có mặt và cắt **cùng ranh giới** với Chromium *(nếu lệch, hai nền tảng
  chọn hai cụm khác nhau cho cùng một cú double-click — không cổng nào bắt)*;
  ② `U+2060` có giữ được tính liền từ với ICU của WebKit không;
  ③ `Selection.modify('extend','right','word')` trên cấu trúc `.hv-unit` `display: inline`
  *(số đo Chromium: một lần bấm = một TỪ; `inline-block` thì **kẹt hẳn** — xem doc-comment
  `.hv-unit` trong `SourceHanViet.vue`)*.
  **Chủ: món nợ hai nền tảng chung, đóng khi CI macOS dựng được bản thật.**

- ⚠️ **ICU cắt SAI ở một tỉ lệ có thật trên văn xuôi TIỂU THUYẾT — danh sách ca sai đã đo,
  không phải một lo xa.** Story 1.18b chạy `Intl.Segmenter('zh')` trên bốn đoạn mở đầu của
  bốn bộ tiểu thuyết cổ điển *(công hữu)* và ghi lại các lượt cắt sai **có thật**:

  | Nguồn | ICU cho ra | Đúng phải là | Loại lỗi |
  |---|---|---|---|
  | 三國演義 | `周末` | `周` + `末` *(cuối đời Chu)* | từ hiện đại đè nghĩa cổ |
  | 三國演義 | `分` + `爭` | `分爭` | cắt vụn một từ |
  | 西遊記 | `有一` + `國土` | `有` + `一` + `國土` | ghép sai qua ranh giới |
  | 西遊記 | `海` + `中有` + `一座` | `海中` + `有` + `一座` | ghép sai qua ranh giới |
  | 西遊記 | `傲` + `來` + `國` | `傲來國` *(tên nước)* | tên riêng bị xé |
  | 西遊記 | `正當` + `頂上` | `正` + `當頂上` | từ hiện đại đè nghĩa cổ |
  | 紅樓夢 | `姑` + `蘇` · `閶` + `門` | `姑蘇` · `閶門` *(địa danh)* | tên riêng bị xé |
  | 紅樓夢 | `一` + `二等` | `一二等` | ghép sai qua ranh giới |
  | 水滸傳 | `大` + `宋` | `大宋` *(quốc hiệu)* | tên riêng bị xé |
  | 水滸傳 | `在` + `位` | `在位` | cắt vụn một từ |

  ⚠️ **Đây KHÔNG phải một lỗi phải sửa ở 1.18b**, và lý do đã ghi thành chữ trong
  `wordBoundary.ts`: một lượt cắt sai ở đây chỉ khiến người dùng **kéo chọn lại**, không làm
  lệch một điểm khớp nào *(so với `mockups/tm-fuzzy-match.html:267-269`, nơi cùng tỉ lệ sai đó
  bị từ chối vì nó làm **điểm khớp** lệch không giải thích được)*. Nó đáng ghi vì **Story 3.4**
  *(đánh dấu thuật ngữ Glossary trong Panel Source)* và **Story 3.7/FR113** sẽ gặp lại đúng
  các ca này, và vì chúng cho thấy **tên riêng** là lớp sai lớn nhất — đúng thứ Glossary tồn
  tại để giải.
  **Chủ: dữ kiện cho Story 3.4 · 3.7. Không hành động ở Epic 1.**

- ⚠️ **Story 3.4 KHÔNG bị chặn, nhưng nó phải TỰ CẮT `.hv-unit`.** Rà 2026-08-07: 3.4 đánh
  dấu thuật ngữ Glossary trong Panel Source bằng ranh giới do **`Matcher` (Rust)** trả về, và
  ranh giới đó **không nhất thiết trùng** ranh giới TỪ của ICU — một thuật ngữ có thể phủ *một
  phần* một `.hv-unit`, hoặc *bắc cầu* hai `.hv-unit`. Trước 1.18b *(một ký tự một node)* việc
  đánh dấu chỉ là gắn class cho các node trong khoảng; nay 3.4 phải **tách `.hv-unit` tại biên
  thuật ngữ**. ⇒ mệnh đề cho 3.4: **ranh giới của `Matcher` thắng ranh giới của ICU**; ICU chỉ
  quyết *"double-click phủ tới đâu"*.
  ⚠️ Và khi 3.4 tách node, nó phải giữ đúng bất biến mà `resolveSwitch()` đứng lên:
  `host.children[i]` ứng **một-một** với `segments.value[i]`.
  **Chủ: Story 3.4.**

- ⚠️ **Bàn đo vùng chọn là một tệp DÙNG MỘT LẦN, không phải một lưới tự động.** Toàn bộ vế DOM
  của 1.18b *(double-click, vùng chọn, clipboard, bàn phím)* nghiệm thu bằng một trang HTML
  chạy tay trong trình duyệt — nó **chép** DOM/CSS của `SourceHanViet.vue` chứ **không mount**
  component thật *(component cần cầu IPC Tauri cho `hanVietByChar`)*. Nghĩa là: một lượt sửa
  template sau này có thể làm bàn đo và sản phẩm **lệch nhau mà không cổng nào đỏ**. Dự án cố
  ý **không có bộ chạy test frontend** *(NFR15, Ice chốt ở 1.5, giữ qua tám story)*, nên đây là
  một giới hạn **đã biết và đã chọn**, không một thiếu sót.
  **Chủ: treo cho tới khi có quyết định về một bộ chạy test frontend.**

---

## Deferred from: code review of 1-18b-tach-tu-tieng-trung-tab-han-viet (2026-08-08)

- ⚠️ **`onCopy` không chặn rò âm Hán Việt qua `Selection.modify()` trên WKWebView, kiểu SONG
  SONG.** `SourceHanViet.vue:571-582` chỉ vào cuộc khi chuỗi chứa `WORD_JOINER`; kiểu song song
  không sinh ký tự đó nên hàm thoát sớm và lượt copy đi đường mặc định của trình duyệt. Nhưng
  doc-comment `:409-412` **đã đo** rằng trên WKWebView đường bàn phím cho ra `他tha打đả開khai…`
  — tức bôi đen bằng phím rồi `⌘C` ở kiểu song song dán ra chuỗi **lẫn âm**. AC5 của 1.18b chỉ
  đòi **`U+2060`**, và lớp lỗi này có **trước** 1.18b *(`<rt>` đã tồn tại từ lượt vá `51132cb`)*
  ⇒ ngoài phạm vi story. Sửa được bằng cách cho `onCopy` dựng lại chuỗi copy từ chính đường đọc
  DOM *(`resolveParallel`)* thay vì tin `Selection.toString()`.
  **Chủ: chưa gán — nợ chung với món "hai nền tảng" của 1.6/1.14/1.16/1.17/1.18/1.18b.** **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

- 🔴 **Kiểu CHUYỂN ĐỔI mất tính "rẻ" — và nó là đường lui KHÔNG có trần.** Trước 1.18b
  `.hv-switch` là `<p>{{ switchText }}</p>`, **đúng MỘT** text node; đó chính là lý do bảng đo
  của `PARALLEL_VIEW_RENDER_CEILING` ghi **222,4 ms ở 500.000 ký tự** và gọi nó là van an toàn.
  1.18b thay bằng một `.hv-word` mỗi TỪ + một `.hv-syl` mỗi KÝ TỰ. Đo lại 2026-08-08 *(bàn đo
  tái lập được số cũ: 23,6 ms vs 24,2 ms ở 50.000 ⇒ so sánh được)*:

  | ký tự Hán | trước 1.18b | sau 1.18b |
  |---|---|---|
  | 5.000   | 3,1 ms · **1** node  | 41,5 ms · 13.728 node |
  | 50.000  | 23,6 ms · **1** node | **532,9 ms** · 136.864 node *(gấp 23)* |
  | 100.000 | 62,1 ms · **1** node | 1.038,5 ms · 273.728 node ⇒ ngoại suy **~5 s** ở 500.000 |

  ⚠️ `canUseParallelView` chỉ khoá **kiểu song song** ⇒ trên 50.000, người dùng bị ép vào đúng
  bề mặt vừa nặng lên, **không trần nào che**. Bảng AC9 của story chỉ đo song song-mới vs
  song song-cũ; nhánh chuyển đổi chưa đo lần nào.
  ⚠️ Và cột "node" của bảng AC9 đếm **con trực tiếp** của host, không phải node DOM — đo cùng
  bàn: song song mới ở 50.000 là **142.128** node DOM, không phải 34.714. Hai cột đó **không
  so được với nhau**; đừng trộn.
  🔴 **Ice chốt 2026-08-08: CHẤP NHẬN có ý thức.** Không đặt trần cho kiểu chuyển đổi *(Chương
  lớn sẽ không còn đường xem nào)*, không dựng `.hv-syl` theo yêu cầu *(một cơ chế mới, đắt hơn
  khoản nó vá)*. Số đã ghi cả vào `sourcePanelState.ts` để lượt sau không đo lại từ đầu.
  **Chủ: chưa gán — mở lại khi có một Chương thật vượt 50.000 và người dùng báo giật.** **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

## Deferred from: 1-19-bat-tat-nguon-tu-dien-va-ghi-cong (2026-08-10)

- 🔴 **Vế DOM của Story 1.19 chưa có bộ chạy test — nghiệm thu bằng BÀN ĐO CHẠY TAY.** Không
  bộ chạy test frontend (NFR15, Ice chốt ở 1.5, giữ qua mười story), nên bốn mệnh đề dưới đây
  **chưa có lưới tự động**: ① dải chip vẽ lại tức thì khi tắt/bật *(AC2)*; ② chip tắt phân biệt
  được bằng mắt mà **không** dùng `opacity` *(UX-DR6 — cài bằng màu + `line-through`, cổng
  `check:tokens` Kiểm D chỉ canh `opacity`, không canh việc nó **có** phân biệt được)*;
  ③ `Escape` đóng lớp phủ và **trả tiêu điểm về chỗ cũ** *(UX-DR17)*; ④ dải chip + bảng
  Attribution duyệt hết được bằng `Tab`. Vế KHAI BÁO thì có cổng: `check:commands` Kiểm A
  *(mọi `@click` là một `dispatch`)* và `SELECTION_SURFACE_FLOOR = 6`.
  **Chủ: chưa gán — nợ chung với món "không bộ chạy test frontend" của 1.16/1.17/1.18/1.18b.** **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

- 🔴 **`AttributionOverlay.vue` chưa đo trên WKWebView.** Lớp phủ dùng `position: fixed` +
  `inset` + một `z-index` có miễn trừ có tên, và nó nằm **trên** lưới `dockview` — mà dockview
  tự dựng ngữ cảnh xếp lớp riêng cho mỗi nhóm panel. Đo mới chạy trên Chromium.
  **Chủ: chưa gán — nợ chung với món "hai nền tảng" (NFR14) của 1.6/1.14/1.16/1.17/1.18/1.18b.** **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

- ✅ **ĐÃ ĐÓNG 2026-08-10 ở lượt code review Story 1.19 — Ice chốt vá thật thay vì để nợ.**
  ~~`viwiktionary-en` là nguồn DUY NHẤT của đường tiếng Anh — vị từ "mọi nguồn đều tắt"
  KHÔNG hỏi theo đường đang tra.~~ Đo trên bốn tệp `.db` thật 2026-08-08: đúng **một** nguồn
  mang `lang = 'en'`. Tắt riêng nó ⇒ **mọi** truy vấn tiếng Anh trả rỗng trong khi bảy nguồn
  tiếng Trung vẫn bật, và `everySourceIsOff` *(hỏi trên **toàn tập**)* trả `false` ⇒ panel nói
  *"không tìm thấy trong từ điển"* — một câu **SAI**, hệ thống không hề tra.
  🔴 **Lý lẽ "chưa sửa được" đã SAI ở một vế, và đó là chỗ đáng ghi lại.** Bản đầu lập luận:
  *"webview không biết nguồn nào phục vụ đường nào, và dựng bảng tra `code → lang` ở webview
  là dựng đúng sổ đăng ký AD-44 ① vá A2 cấm"*. Vế sau đúng; vế trước sai. Câu trả lời không
  phải một bảng tra ở webview, cũng không phải một vị từ Rust nhét vào `GroupedLookup` — mà
  là **để dữ liệu tự khai**: `dict_source.lang` nay **ĐO lúc dựng** từ `dict_entry` của chính
  tệp *(`insert.rs::backfill_source_langs`)*, đúng cùng đường mà `is_base` đọc `dict_meta`.
  Webview chỉ **đọc một trường**, không suy luận gì cả, và AD-44 không bị chạm tới.
  **Cách cài:** `SCHEMA_VERSION` 2→3 · cột `dict_source.lang` *(tập, quy ước
  `parse_disabled_sources`)* · `SourceAttribution.lang` · `everySourceOffForRoute(route)` với
  `route` đọc từ `grouped.route` Rust đã trả về. **Đo được:** 10/10 nguồn cho đúng một `lang`;
  giá đọc **~480 ms → ~16 ms** mỗi lượt khởi động *(dẫn xuất lúc đọc bằng `SELECT DISTINCT`
  tốn 374 ms + 97 ms vì `dict_entry` không có index trên `source_id`)*; bốn tệp `.db` dựng
  lại, **ba lượt dựng độc lập cho cùng bốn `sha256`**.

- ⚠️ **§Quyết định #2b *(lọc thẳng trong câu SQL)* nay là một MÓN NỢ CÓ SỐ.** Đo 2026-08-10,
  `--release`, bốn tệp `.db` thật, 130 truy vấn khác nhau, hai lượt mỗi cấu hình — **tỉ lệ lượt
  tra chạm trần `LIMIT`**: **1,8 %** *(0 nguồn tắt)* · **1,8 %** *(1 nguồn tắt)* · **4,8 %**
  *(9/10 nguồn tắt)*.
  🔴 **Nguyên nhân KHÔNG phải bộ lọc**, và đọc nhầm chỗ này là đi sửa nhầm chỗ: con số **không
  đổi** khi tắt một nguồn — đúng như §Quyết định #2a tiên liệu *(trần chạy TRƯỚC phép lọc, nên
  các nguồn còn lại giữ nguyên tập đầu mục)*. Nó tăng ở cấu hình 9/10 vì lượt `Exact` rỗng
  nhiều hơn ⇒ **đường lui `Substring` của Story 1.18** chạy nhiều hơn, và chính nó mới là thứ
  chạm trần. ⇒ lọc trong SQL sẽ làm **trang đầy hơn** ở cấu hình nhiều nguồn tắt, nhưng nó đụng
  **sáu** bề mặt *(`exact` · `exact_en` · `char_idx` · `fts_trigram` · `fts_trigram_en` ·
  `count_by_source`)*, mỗi câu một `IN` sinh động.
  **Chủ: chưa gán — mở lại nếu người dùng thật báo trang vơi khi tắt nhiều nguồn.** **(Chủ: một story kế tiếp chạm `core/dict`.)**

- ⚠️ **`max` một lượt đo vượt trần 100 ms — ghi ra chứ không làm tròn xuống.** Cùng bàn đo:
  cấu hình *9/10 nguồn tắt*, **lượt 1**, `max` **150,628 ms** *(lượt 2: 48,856 ms)*. NFR1 phát
  biểu trên **p95**, và p95 của chính cấu hình đó là **3,891 / 2,369 ms** — dưới trần 26–42 lần.
  Đây là nhiễu page-cache của lượt đầu, đúng **Bẫy 8** mà Story 1.18 đã ghi *(1.17 đo p99
  70,742 ms ở lượt đầu và không tái lập được)*. Không kết luận trên một lượt đo.
  **Chủ: chưa gán — theo dõi cùng món "đo NFR1 đầu-cuối gồm vòng IPC" của 1.17.** **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

- ⚠️ **Số đo NFR1 vẫn là ĐƯỜNG RUST, không đầu-cuối.** Bàn đo mới
  *(`bench_the_source_filter_on_the_real_dictionaries`)* thừa hưởng nguyên giới hạn của 1.17/1.18:
  nó không gồm vòng IPC Tauri lẫn lượt vẽ của webview, **và** không gồm lượt đọc `global.db` mà
  `commands::dict::wire` chạy ở **mỗi** lượt tra để lấy tập bị tắt.
  🔴 Lượt đọc đó là **mới của story này**, nên nó là phần chưa ai đo bao giờ: một `load_global_config`
  cho **mỗi** lượt Auto-Lookup. Nó rẻ *(một `SELECT` trên `config_value` của `global.db`, ba
  loại `GlobalOnly`)*, nhưng *"rẻ"* ở đây là một suy luận, không một số đo.
  **Chủ: chưa gán — đóng cùng món "đo đầu-cuối" của Story 1.17.** **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

- ⚠️ **`prd.md §8.2` và `docs/dics/README.md` vẫn xếp Trần Văn Chánh vào nhóm *"đã loại"*.**
  Đo trên tệp thật: `dict-tran-van-chanh.db` **đã dựng**, `license_kind = "copyrighted"`, và
  `attribution` của nó mang một **cảnh báo pháp lý** *(nay hiện nguyên văn trên màn hình
  Attribution)*. `deferred-work.md:581` đã treo việc đồng bộ đó cho *"lượt quy hoạch kế tiếp
  hoặc Story 10.4"*; story này **không sửa file quy hoạch** *(tiền lệ 1.10c)*.
  **Chủ: Story 10.4, hoặc một lượt quy hoạch của Ice.**

- ⚠️ **Chỗ giữ `author-grant` chưa từng chạy trên dữ liệu thật, và sẽ không.** Ice chốt
  2026-08-08: **HVTĐTD không tìm được nguồn dữ liệu**. AC9 đã được neo lại vào **cơ chế** và
  nghiệm thu bằng **fixture** *(`the_author_grant_placeholder_lands_and_leaves_with_its_file`)*.
  **0** nguồn thật nào mang `license_kind = "author-grant"` hôm nay. Ba chỗ xuôi dòng vẫn mang
  mệnh đề cũ: `epics.md:1839-1841` *(AC cuối của chính story 1.19)* · `epics.md:6202-6204`
  *(Story 10.4, **nguyên văn giống hệt** ⇒ cùng số phận)* · `mockups/sources-attribution.html`
  *(vẽ HVTĐTD như một hàng có thật)*.
  🔴 Và `deferred-work.md:292` *(nghĩa vụ thông báo tác giả HVTĐTD)* **mất điều kiện kích
  hoạt** — không đóng gói dữ liệu ⇒ không có phép sử dụng nào để thực hiện. Đóng hay giữ là
  quyết định của **Story 10.4**, không của story này.
  **Chủ: Story 10.4.**

## Deferred from: code review of 1-19-bat-tat-nguon-tu-dien-va-ghi-cong (2026-08-10)

- ✅ **ĐÃ ĐÓNG 2026-08-10 — và nó KHÔNG phải một lo ngại lý thuyết như lượt triage đã xếp.**
  ~~Dải chip có thể cắt hàng thứ ba khi đủ 10 nguồn thật.~~ Lượt triage xếp mức **low** và
  `defer` với lý do *"quyết định CÓ CHỦ Ý, đã ghi trong chú thích CSS, cần đo pixel thật"*.
  **Ice chạy app thật và ảnh chụp bác lại ngay**: với mười nguồn, `max-height: 52px;
  overflow: hidden` cắt mất **ba** thứ, và chú thích CSS biện hộ *"hàng thứ ba tới được bằng
  bàn phím"* đã bỏ sót cả ba:
  ① **nút "Nguồn dữ liệu"** — con CUỐI trong dải, tức **đường chuột DUY NHẤT** vào màn hình
     Attribution (AC11) biến mất. Hai phép thử bẫy tiêu điểm và cửa nuốt hợp âm **không bắt
     đầu được bằng chuột**, đúng như Ice gặp;
  ② chip **Trần Văn Chánh** *(`license_kind = "copyrighted"`, cảnh báo pháp lý trong
     `attribution`)* — **không tắt được bằng chuột**, trong khi FR112 dựng cả cơ chế lớp gỡ
     rời chính vì rủi ro của nguồn này;
  ③ chip **VietPhrase**.
  **Cách vá (cầm máu, không phải thiết kế cuối):** tách vùng chip thành một con flex riêng
  `.lookup-sources-chips` *(`flex: 1 1 auto; min-width: 0; overflow-y: auto`)*, còn nhãn và
  nút *"Nguồn dữ liệu"* để `flex: none` ở hộp ngoài — hộp ngoài **không cắt gì cả**. Chip
  tràn thì **cuộn**, không biến mất không dấu vết.
  🔴 **Bài học cho lượt triage sau:** mức `low` ở đây đến từ việc đọc chú thích CSS và tin lý
  lẽ của nó. Lý lẽ ấy đúng về cơ chế *(chip vẫn trong thứ tự Tab)* và sai về hậu quả *(thứ bị
  cắt không phải một chip bất kỳ)*. **Một câu văn xuôi giải thích một quyết định không thay
  được một lượt render.**

- ⚠️ **`list_source_attributions` không loại trùng `code` giữa các lớp.**
  `src-tauri/src/core/dict/mod.rs:940-952` nối `rows` từ mọi lớp, không dedupe. Bất biến
  *"`code` duy nhất trong toàn tập lớp"* chỉ được ghi bằng **doc-comment**, không cổng nào ép.
  Nếu bất biến đó vỡ (hai tệp `.db` cùng mang một `dict_source.code`), hai hàng trùng
  `:key="src.code"` (`AttributionOverlay.vue:119`, `LookupPanel.vue:263`) phá giả định duy nhất
  của Vue, và bật/tắt một chip sẽ bật/tắt **CẢ HAI** nguồn vì chúng chia một mục trong tập
  `disabled`. Không với tới được bằng dữ liệu hiện có — phía dựng (`tools/dict-build`) giữ bất
  biến này — nên là **độ bền**, không một lỗi đang chạy. Chỗ vá rẻ nhất: một `assert` ở đường
  đọc, hoặc một test canh `code` duy nhất trên bốn tệp thật. **(Chủ: story dựng thêm lớp từ điển tiếp theo — `tools/dict-build`.)**

- 🔴 **Vế DOM của chính lượt code review 2026-08-10 chưa chạy trên app thật.** Lượt vá thêm
  **mã DOM mới** mà không một phép thử tự động nào chạm tới, nên nó **mở rộng** đúng lỗ hổng
  mà món nợ *"vế DOM của Story 1.19 chưa có bộ chạy test"* ở §1-19 đã khai, chứ không nằm
  trong phạm vi cũ. Ba thứ cần rà tay trên **cả hai** nền *(và WKWebView là nền đáng ngờ
  nhất, đúng như món nợ `AttributionOverlay.vue` đã ghi)*:
  ① **Bẫy tiêu điểm** (`AttributionOverlay.vue::trapTab`) — Tab và Shift+Tab vòng đúng bên
     trong hộp thoại, không thoát ra nền; `Escape` vẫn đóng được ở **mọi** vị trí tiêu điểm.
  ② **Cửa nuốt hợp âm** (`main.ts` → `KeymapGate`) — mọi hợp âm toàn cục **im** khi lớp phủ
     mở, và `Escape` **vẫn** đi lọt *(nó thoát sớm chứ không `preventDefault`, nhưng điều đó
     mới chỉ đúng trên giấy)*.
  ③ **Đường lui `[data-attribution-open]`** — chưa từng chạy: nó chỉ kích hoạt khi node giữ
     tiêu điểm lúc mở đã rời DOM, mà D4 vừa bịt gần hết nguyên nhân gây ra ca đó.
  ⚠️ Và hai câu `vi.json` mới (`attribution.load_failed` · `panel.source.han_viet_all_sources_off`)
  chưa ai nhìn thấy hiện ra: câu đầu cần ép `list_dict_sources` trượt, câu sau cần tắt hết
  nguồn `zh` rồi mở tab Hán Việt. **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

- 🔴 **Bờ đọc từ điển KHÔNG có sàn phiên bản — một tệp `.db` QUÁ CŨ hỏng NỬA VỜI thay vì bị
  từ chối có tên.** Phát hiện khi soạn bàn đo chạy tay ngay sau lượt vá, 2026-08-10.
  `layer.rs:292` chỉ hỏi `if file_version > SUPPORTED_SCHEMA_VERSION`, tức chỉ chặn tệp
  **MỚI hơn**. Sau lượt nâng `SCHEMA_VERSION` 2→3, một tệp **v2** vẫn được **NHẬN** *(2 > 3
  là sai)*, rồi mới gãy ở `DictLayer::attributions` bằng `no such column: lang`.
  ⇒ `list_source_attributions` bỏ **im lặng cả lớp** kèm một dòng `stderr`, nên **bảng ghi
  công rỗng và dải chip biến mất**, trong khi **tra cứu vẫn chạy bình thường** *(đường tra
  không đọc cột `lang`)*. Người dùng thấy một ứng dụng tra được từ nhưng khai *"chưa gắn lớp
  từ điển nào"* — đúng hình dạng *"hỏng nửa vời, không ai biết"* mà `SkipReason` sinh ra để
  chống.
  **Đường bịt:** một hằng `MINIMUM_SCHEMA_VERSION` cạnh `SUPPORTED_SCHEMA_VERSION`, cộng một
  nhánh `SkipReason::SchemaTooOld { file_version, minimum }`. Rẻ, và nó biến một lượt hỏng im
  lặng thành một câu đọc được trên màn hình.
  ⚠️ **Phạm vi chạm:** bản phát hành **không** chạm — bốn tệp đi cùng một release và `sha256`
  trong `dict-manifest.toml` ràng chúng lại. Ca thật là một **máy dev** chưa chép lại bốn tệp
  sau lượt dựng, và nó xảy ra **ngay lập tức** với bất kỳ ai đang giữ bản `.db` ngày 2026-08-07
  ở `src-tauri/resources/dict/`.
  ~~**Chủ: chưa gán.**~~ **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

- ⚠️ **D3 mới bịt được NỬA lỗ: một tệp `.db` đọc KHÔNG được vẫn nói *"chưa gắn lớp nào"*.**
  Phát hiện khi soạn bàn đo chạy tay, 2026-08-10. `attribution.load_failed` chỉ hiện khi
  `dictSourcesError !== null`, mà `list_dict_sources` phía Rust trả `Vec<SourceAttribution>`
  **không** `Result` — nó không có đường sinh lỗi. Nhánh đó vì thế chỉ với tới được bằng một
  lỗi **tầng invoke** *(command chưa đăng ký, lỗi tuần tự hoá, hỏng đường IPC)* ⇒
  `UNKNOWN_IPC_ERROR`. Không phải mã chết, nhưng hẹp.
  🔴 **Ca thật vẫn hở:** một lớp mà `dict_source` đọc không được *(tệp hỏng, quyền sai, tệp bị
  thay dưới chân tiến trình)* bị `list_source_attributions` **nuốt** kèm một dòng `stderr`, và
  hàm trả về một `Vec` **rỗng hoặc thiếu hàng** — tức `error` vẫn `null`, và màn hình vẫn nói
  `attribution.empty` *"Chưa gắn lớp từ điển nào"*, đúng câu SAI mà D3 dựng ra để chặn. Cùng
  hình dạng với ca sàn phiên bản ở mục trên, và hai mục nên vá **cùng một lượt**.
  **Đường bịt:** cho `list_dict_sources` trả về cả **danh sách lớp bị bỏ** *(`skipped`, kiểu
  đã có sẵn — `lookup_grouped` đang dùng)*, rồi bảng phân biệt ba trạng thái thay vì hai:
  *0 tệp* · *có tệp nhưng k lớp bị bỏ* · *gọi trượt*. ~~**Chủ: chưa gán.**~~ **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

---

## Story 1.20 — lịch sử tra cứu và mục đã ghim (2026-08-10)

- 🔴 **Cạnh phụ thuộc MỚI chưa có trong Capability Map: Panel Lookup (C3) → `core/store/`.**
  `ARCHITECTURE-SPINE.md:823` liệt kê C3 là `core/dict/`, `ports/DictionarySource`,
  `resources/dict/` — **không** có `core/store/`. Mục ghim sống ở `global.db` (Ice ký lại
  2026-08-11 — bản đầu chốt `project.db`), nên `commands/pinned.rs` là một cạnh C3 → tầng
  ghi dữ liệu mà bản đồ chưa mang. Lượt đổi phạm vi **không** làm cạnh này biến mất, chỉ đổi
  kho ở đầu kia. Cùng loại với cách AD-36 phải thêm cạnh `glossary/ → dict/` (`:435`).
  **Chủ: lượt cập nhật kiến trúc kế tiếp.**

- ⚠️ **`headword`/`gloss` của một mục ghim là ẢNH CHỤP, và ảnh chụp thì cũ đi.**
  `pinned_entry` lưu chữ, không một khoá ngoại vào từ điển — có chủ ý: một hàng ghim phải
  hiện ra được **không cần một lượt tra thứ hai**, và `entry_id` chỉ duy nhất TRONG một tệp
  `.db`. Giá phải trả: thay tệp `.db` nguồn ở một bản phát hành sau có thể làm
  `(source_code, entry_id)` trỏ vào một đầu mục **khác**, trong khi hàng vẫn hiện chữ cũ.
  Hôm nay vô hại (chưa bản phát hành nào), nhưng nó thành thật ngay lượt thay dữ liệu đầu
  tiên. **Đường bịt:** một cột `source_version` trên `pinned_entry` cộng một câu *"mục này
  ghim từ một bản dữ liệu cũ hơn"*. ~~**Chủ: chưa gán.**~~ **(Chủ: Story 1.20.)**

- ⚠️ **Số lần tra trên hàng ghim thuộc PHIÊN, không bền vững** (§Dev Notes ⑨ của story).
  Một cột `lookup_count` bền vững đòi một lượt `Store::write` **mỗi lượt tra** — tức mỗi lần
  bôi đen chữ — đưa ghi đĩa vào đúng đường nóng của Auto-Lookup và cho nó cạnh tranh hàng đợi
  ghi nối tiếp với auto-save Editor (NFR2, AD-11/AD-12). Không AC nào đòi nó sống qua phiên.
  Nếu về sau muốn thật: **đo chi phí ghi TRƯỚC**, đừng thêm cột rồi mới đo. ~~**Chủ: chưa gán.**~~ **(Chủ: story kế tiếp quản lý mục ghim.)**

- ⚠️ **Nhãn thời gian tương đối KHÔNG có đồng hồ riêng.** `relativeTimeKey`/`relativeTimeParams`
  tính lại ở mỗi lượt render, nên *"vừa xong"* chỉ thành *"1 ph"* ở **lượt tra kế tiếp** (hoặc
  một lượt render khác), không tự trôi theo thời gian. Một `setInterval` cho việc này là một
  hẹn giờ chạy suốt phiên chỉ để sửa một nhãn phụ; chưa đáng. ~~**Chủ: chưa gán.**~~ **(Chủ: story kế tiếp quản lý nhãn thời gian tương đối.)**

- ⚠️ **Trần lịch sử 200 hàng là một con số CHƯA ĐO** (`lookupHistoryState.ts::HISTORY_CEILING`).
  AC7 (dedupe) chặn *"hàng trăm dòng giống nhau"*, nó **không** chặn hàng trăm dòng KHÁC nhau
  — một Chương dài có thừa từ khác nhau để tra. 200 là một phỏng đoán có tên, cắt ở ĐUÔI (cũ
  nhất). Nếu người dùng thật chạm trần, con số phải đến từ một lượt đo. ~~**Chủ: chưa gán.**~~ **(Chủ: story kế tiếp đo trần lịch sử tra cứu.)**

- ⚠️ **Hàng lịch sử KHÔNG bấm để tra lại được.** Mockup không hứa điều đó, và Kiểm A của
  `check:commands` đòi mọi `@click` là đúng một `dispatch('<id>')` — nên một hàng bấm được
  cần một command thứ năm mang **mục tiêu**, thứ §KHÔNG-LÀM ⑤ và Quyết định #7 đều không cấp.
  Nếu về sau muốn: một `lookup.lookup_history_row` đọc mục tiêu từ `@mousedown` uỷ quyền, đúng
  khuôn `lookup.toggle_pin` — và nó **phải** đi qua `runLookup` chứ không một đường gọi song
  song (Bẫy 3: một đường thứ hai bỏ qua `sequence` là dựng lại đúng lỗi đã vá). ~~**Chủ: chưa gán.**~~ **(Chủ: story kế tiếp cải thiện lịch sử tra cứu.)**

- ⚠️ **Ba lệch mockup của story này, ghi lại thay vì sửa tài liệu quy hoạch** (Quyết định #3
  của Story 1.3): ① `Concordance` bị loại khỏi dải tab (FR64/Story 7.7, đo được **0** lần
  trong `src/`); ② thanh lọc ba chip bị loại (*"Cả Tác phẩm"* mâu thuẫn AC4); ③ `⌘⌫` bị bác
  cho `lookup.clear_history`. Thêm hai lệch nhỏ phát hiện lúc cài đặt: ④ **cột âm đọc** của
  hàng ghim/lịch sử **không có** — âm Hán Việt là một lượt tra RIÊNG (`read_han_viet`) và
  `pinned_entry` không lưu nó; dựng nó ở đây là một vòng IPC thứ hai cho mỗi hàng;
  ⑤ `pinned_empty_note` viết lại thành một câu đủ (mockup ghi `⌘D khi đang xem một mục từ`,
  mà một hợp âm viết cứng theo nền tảng là đúng thứ Kiểm D/NFR14 tồn tại để chặn). **(Chủ: Winston — architect.)**

- 🔴 **Bàn đo chạy tay của Story 1.20 CHƯA CHẠY** — 18 hàng ở §Testing của story, gồm mọi đối
  chứng âm cho AC3/AC4/AC7/AC9/AC12 và hai hàng NFR14 (`Mod+D` trên cả macOS lẫn Windows).
  Vế DOM không có bộ chạy test frontend (§KHÔNG-LÀM ⑥, nợ `:836-846` nối dài). **Chủ: Ice**,
  và story không được đánh dấu `done` trước lượt đó.

- ⚠️ **Không cổng nào canh được thứ tự XẾP LỚP giữa lớp phủ và cây dockview.** Lỗi *"sash vẽ
  đè lớp phủ Attribution"* (Ice bắt bằng mắt 2026-08-10, vá bằng `isolation: isolate` trên
  `.modeport`) đi qua **cả chín cổng XANH** ở cả hai lượt — trước lẫn sau lượt vá. Kiểm F của
  `check-tokens.mjs` đọc `z-index` như một **chuỗi ký tự cần miễn trừ**, nó không so hai giá
  trị với nhau và hoàn toàn không biết `node_modules/dockview-vue/dist/styles/dockview.css`
  tồn tại. ⇒ mọi mệnh đề *"cái này nằm trên cái kia"* trong dự án hôm nay chỉ được canh bằng
  **mắt người**, và cùng lớp lỗi sẽ tái phát ở mỗi lượt nâng `dockview-vue`.
  **Đường bịt rẻ nhất:** một ca trong `check-layout.mjs` đọc `z-index` cao nhất mà
  `dockview.css` khai *(hôm nay 9999)* rồi đòi mọi lớp phủ của `src/**` hoặc đứng trên số đó,
  hoặc nằm ngoài một khối mang `isolation`/`contain` — tức cưỡng chế chính **cơ chế** vừa
  chọn thay vì một con số. ~~**Chủ: chưa gán.**~~ **(Chủ: một story hạ tầng cổng kế tiếp.)**

- 🔴 **`.lookup-head` CẮT NỘI DUNG khi thanh nhịp xuống dòng thứ hai — đo được, không suy
  đoán.** Ice nghi ngờ từ ảnh chụp 2026-08-10; đo bằng CDP trên app thật xác nhận.

  Số đo (`.lookup-head`, 6 nguồn, thanh nhịp 2 dòng):
  ```
  offsetHeight  76   (border-box — đúng --lookup-head-height, KHÔNG vỡ)
  clientHeight  75   (content 63 + padding-bottom 12)
  scrollHeight  89   ⇒ nội dung vượt hộp NỘI DUNG 14px
  ```
  Phân rã: đầu mục `24px × 1.3` = **31,19** + `.lookup-spine` `margin-top` **7** +
  thanh nhịp **39,25** *(dòng 1 mang `.lookup-spine-count` `ui-sm` 11,5×1,5 = 17,25 · khe
  `gap` 8 · dòng 2 chỉ có chip `ui-label` 10×1,4 = 14)* = **77,44px** nội dung, trong khi
  `overflow: hidden` cắt ở **mép padding = 75px**.
  ⇒ **~2,4px cuối của dòng nhịp thứ hai bị cắt, và toàn bộ 12px khoảng thở trước nét ngăn
  bị nuốt** — dòng nhịp thứ hai dính sát viền dưới. Ở 3 dòng nhịp thì mất hẳn một dòng.

  ⚠️ **KHÔNG phải do Story 1.20.** Dải tab của story này là một hàng RIÊNG, `flex: none`,
  **ngoài** `.lookup-head` (AC10 đo được: `offsetHeight` = 76 ở **cả hai** tab). Thủ phạm là
  chính thanh nhịp — Story 1.17 dựng nó, Story 1.19 làm nó dài ra bằng cách thêm nguồn.
  Ngưỡng vỡ là **số dòng nhịp ≥ 2**, tức phụ thuộc số nguồn đang bật **và** bề rộng panel.
  Với 4 tệp `.db` thật hôm nay và panel ~593px, Ice đang ở đúng ngưỡng đó.

  🔴 **Cố ý KHÔNG vá ở story này.** `--lookup-head-height: 76px` là hằng mà bốn story đã
  phải tránh (1.17 thanh nhịp · 1.18 vạch tiến trình · 1.19 dải chip · 1.20 dải tab), và
  AC10 nói thẳng *"nếu không vừa bố cục thì **nói ra và đo** — đừng nới hằng trong im
  lặng"*. Nới nó ở cuối một story không sở hữu nó là đúng cách một bất biến chết.
  **Ba đường vá, đo trước khi chọn:** ① cho `.lookup-spine` `flex-wrap: nowrap` +
  `overflow-x: auto` *(một dòng, cuộn ngang — cùng thuốc mà dải chip nguồn đã dùng ở 1.19)*;
  ② nâng `--lookup-head-height` theo một phép đo ở số nguồn **tối đa**, không ở số nguồn
  hôm nay; ③ chuyển thanh nhịp ra một hàng riêng như dải chip. ~~**Chủ: chưa gán.**~~ **(Chủ: story kế tiếp chạm `LookupPanel.vue` (thanh nhịp).)**

- 🔴 **`user_version = 4` của `project.db` là một số ĐÃ CHÁY.** Story 1.20 bản đầu
  (2026-08-10) thêm bước di trú 4 đặt `pinned_entry` vào `PROJECT_MIGRATIONS`; lượt Ice ký
  lại (2026-08-11) chuyển bảng sang `global.db` và **gỡ** bước đó. `PROJECT_MIGRATIONS` về
  đúng ba bước, nhưng con số 4 đã từng tồn tại trên đĩa thật.
  ⇒ **Bước di trú kế tiếp của `project.db` phải đánh số 5**, không được tái dùng 4 — doc-comment
  đầu `schema.rs` nói vì sao bằng chữ: *"một bước như vậy là hai đường lược đồ khác nhau cho
  cùng một số, và chúng sẽ rẽ nhau ở máy người dùng chứ không ở đây"*. Ràng buộc này ghi
  trong doc-comment của `PROJECT_MIGRATIONS`; **không** có cổng nào canh nó.
  ⚠️ Sáu `.atproj` ở `user_version = 4` đã được xoá sau khi Ice ký (2026-08-11, đo lại từng
  tệp trước khi xoá; còn 21 tệp, tất cả ở v3). Nhưng nếu một bản sao nào còn ở máy khác thì
  nó sẽ bị `store.schema_too_new` từ chối mở vào ngày Epic 5 dựng đường mở lại `.atproj` —
  một câu có tên, không hỏng im lặng. ~~**Chủ: chưa gán.**~~ **(Chủ: một story kế tiếp chạm `core/store`.)**

- 🔴 **KHÔNG cổng nào bắt được một chuỗi `vi.json` NÓI DỐI về hành vi.** Lượt đổi phạm vi
  ghim (2026-08-11) làm hai chuỗi sai nghĩa — `pinned_empty_body` hứa *"các Chương của **Tác
  phẩm này**"* và `history_hint` hứa *"Mục ghim sống qua các phiên và **theo Tác phẩm**"* —
  trong khi ghim đã chuyển sang phạm vi toàn ứng dụng. **Cả chín cổng vẫn XANH** với hai câu
  đó: `check-i18n.mjs` kiểm khoá có tồn tại, placeholder có khớp, giọng văn có vô nhân xưng
  — nó **không** đọc nghĩa, và không có gì để đối chiếu nghĩa với. Bắt được bằng một lượt rà
  tay `grep "Tác phẩm này"` sau khi đổi phạm vi.
  ⚠️ Đây là một lỗ **cấu trúc**, không một lượt sơ ý: mọi story đổi hành vi mà quên sửa chuỗi
  mô tả hành vi đó đều rơi vào nó, và triệu chứng là ứng dụng **nói một đằng làm một nẻo** —
  đúng loại lỗi mà UX-DR27/AD-44 ④ tồn tại để chặn ở phía ngược lại. **Đường bịt khả dĩ:**
  một danh sách *"chuỗi mô tả hành vi"* trong chính `vi.json` (một tiền tố, hoặc một tệp
  cạnh) mà mỗi story đổi hành vi phải đọc lại — rẻ, nhưng nó là kỷ luật chứ không phải cơ
  chế. ~~**Chủ: chưa gán.**~~ **(Chủ: một story hạ tầng cổng kế tiếp.)**

## Deferred from: code review of 1-20-lich-su-tra-cuu-va-muc-da-ghim (2026-08-11)

- **Không token thứ tự giữa lượt NẠP và lượt GHI bộ ghim.** `loadPinnedEntries()`
  (`lookupHistoryState.ts:365`) canh mình bằng `loadSequence`, và `pinWriteQueue`
  (`:439`) canh mình bằng một chuỗi promise nối tiếp — nhưng **không** cơ chế nào canh
  giữa **hai** đường đó. Nếu một phản hồi nạp về **sau** một phản hồi ghi, nó đè
  `pinnedRaw` về bản cũ hơn và mục vừa ghim biến khỏi màn hình dù đĩa đã giữ.
  ⚠️ **Hôm nay không đường nào tới được**, và đó là lý do nó nằm ở đây chứ không ở bảng
  vá: `loadPinnedEntries()` có **đúng một** chỗ gọi (`main.ts:325`, lúc khởi động), còn
  một lượt ghi chỉ enqueue được **sau** khi một lượt tra đã xong — tức sau một cử chỉ
  bôi đen của người dùng, chậm hơn một vòng IPC nhiều bậc. Lỗ mở ra ngay khi có đường
  nạp lại thứ hai. **Chủ: Story 1.21** (màn hình gán phím) hoặc story đầu tiên thêm một
  lượt `loadPinnedEntries()` thứ hai — bất kỳ cái nào tới trước.
  → ⚠️ **KHÔNG ĐÓNG, và điều kiện kích hoạt giữ nguyên — Story 1.21 trả lại, Ice ký 2026-08-11.** Điều kiện là *"story đầu tiên thêm một lượt `loadPinnedEntries()` thứ hai"*, và Story 1.21 **không thêm lượt nào**: nó đọc `config.shortcuts` từ chính lượt `await loadBootstrapConfig()` đã có ở `main.ts`, không mở một vòng IPC thứ hai nào. Một story có tên trong mục nợ mà không thoả điều kiện của chính mục đó thì không phải chủ của nó. **Chủ: story đầu tiên thêm một lượt `loadPinnedEntries()` thứ hai — vẫn treo.**
- **`sessionLookupCount` nói dối theo hướng THẤP khi lịch sử chạm trần.**
  `HISTORY_CEILING = 200` (`lookupHistoryState.ts:105`) cắt đuôi lịch sử, và
  `sessionLookupCount` (`:227`) cộng dồn từ **chính** danh sách đó. Ghim một mục rồi tra
  200+ truy vấn **khác** trong cùng phiên ⇒ hàng chở số đếm của mục đó bị đẩy ra, và số
  trên hàng ghim về **0** dù nó thật sự đã được tra N lần. Không AC nào đòi số đếm chính
  xác (§Dev Notes ⑨ chốt nó thuộc phiên và cosmetic), nên đây là một sai lệch **có
  hướng** đã biết, không một khuyết tật. Nếu về sau số đếm thành một thứ người dùng tin,
  nó cần một sổ đếm tách khỏi danh sách lịch sử. ~~**Chủ: chưa gán.**~~ **(Chủ: story kế tiếp quản lý lịch sử tra cứu.)**
- ***"Số 4 đã cháy"* của `PROJECT_MIGRATIONS` chỉ sống bằng văn xuôi.** Doc-comment
  `schema.rs:277-289` ghi rõ số **4** không được tái dùng (bản đầu của Story 1.20 đã đốt
  nó, rồi bước đó bị gỡ ở lượt Ice ký lại). Nhưng `validate_strictly_increasing`
  (`:321`) chỉ kiểm **tăng dần nghiêm ngặt** — `3 → 4` vẫn hợp lệ, nên một story sau
  thêm `Migration { to_version: 4, … }` sẽ **không** làm cổng nào đỏ, và hai lược đồ
  khác nhau mang cùng một số sẽ rẽ nhau ở máy người dùng.
  ✅ **Hành vi hôm nay AN TOÀN, không giấu:** một `.atproj` còn ở `user_version = 4` bị
  `Store::open` **từ chối** bằng `store.schema_too_new` (AC7 của Story 1.7) — hỏng ồn
  ào, không hỏng im lặng. **Đường bịt rẻ:** một ca ở `pinned_contract.rs` khẳng định
  `PROJECT_MIGRATIONS` không chứa `to_version == 4`. Ba dòng, và nó biến một kỷ luật
  thành một cơ chế — đúng thứ dự án này vẫn đòi ở mọi chỗ khác. **Chủ: story đầu tiên
  thêm bước di trú cho `project.db`** (dự kiến Epic 5).
  → ✅ **ĐÓNG 2026-08-12 — Story 2.1**, và chủ hoá ra tới sớm hơn dự kiến (Epic 2, không
  Epic 5). Cổng là `segment_contract.rs::the_project_migration_set_never_reuses_the_burned_number_four`
  *(đặt ở tệp của story sở hữu bước di trú, không ở `pinned_contract.rs` như đường bịt gợi ý
  — `pinned_contract.rs` nói về **phạm vi bảng ghim**, và một mệnh đề về đánh số di trú nằm
  ở đó là một tệp mang hai mối quan tâm)*. Kèm một ca thứ hai khẳng định bộ di trú đọc đúng
  `[1, 2, 3, 5]`. **Đỏ-rồi-xanh đã chạy thật:** đổi `to_version` thành `4` ⇒ **cả hai** ca đỏ
  với đúng thông điệp; đổi lại ⇒ xanh.
  ⚠️ **Một hệ quả mới, ghi ra vì nó đổi hành vi trên dữ liệu có thật:** nâng target lên 5 làm
  một `project.db` mang `user_version = 4` chuyển từ *"bị từ chối mở"* sang *"mở được và di
  trú lên 5"*, mang theo một bảng `pinned_entry` mồ côi. Vô hại về dữ liệu, và cả 6 thư mục ở
  trạng thái đó đã bị Ice xoá 2026-08-11 — nên đây là ghi chép, không phải nợ. Đã vào
  doc-comment của `PROJECT_MIGRATIONS`.
- **`SourcePanel.vue` mang cùng khuyết tật tiêu điểm dải tab.** `@keydown.right/left`
  (`SourcePanel.vue:113-114,126-127`) đổi tab nhưng không gọi `.focus()` trên tab mới,
  nên tiêu điểm DOM ở lại nút vừa nhận `tabindex="-1"`. Với **hai** tab, hệ quả là người
  dùng bàn phím đi được một chiều rồi kẹt: lượt bấm mũi tên thứ hai vẫn phát từ nút cũ
  và dispatch đúng cái id vừa chạy, tức một no-op. Ra được bằng `Shift+Tab` rồi `Tab`
  lại, nên nó **khó chịu chứ không chặn đường**.
  ⚠️ **Có từ Story 1.18**, không phải Story 1.20 — 1.20 chỉ chép lại đúng khuôn đó cho
  `LookupPanel.vue` (chỗ đó được vá trong lượt review này). Hai panel phải vá **cùng một
  cách**, nếu không dự án có hai hợp đồng `tablist` khác nhau cho cùng một cử chỉ.
  ~~**Chủ: chưa gán.**~~ **(Chủ: story kế tiếp chạm `SourcePanel.vue`.)**

## Ràng buộc để lại cho Story 7.7 (Concordance) — từ Story 1.20

- 🔴 **Concordance phải chèn vào GIỮA dải tab của Panel Lookup, không nối vào đuôi.**
  `epics.md:1871-1873` (AC5 của Story 1.20) đòi lịch sử và mục ghim là **tab thứ ba** của
  Panel Lookup. Story 1.20 dựng **hai** tab — `Từ điển` · `Lịch sử` — vì Concordance là
  **FR64, Story 7.7**, một năng lực chưa tồn tại (đo 2026-08-10: `grep -rn "Concordance"
  src/` trả **0** lần trong `src/`, đúng hai doc-comment ở `commands/dict.rs:119,173`).
  ⚠️ **Đây KHÔNG phải một lệch tài liệu** — Ice chốt 2026-08-11 khi một lượt code review
  định mở `correct-course` cho nó. AC5 mô tả **trạng thái cuối** và nó đúng: tab Lịch sử
  hôm nay tạm đứng thứ hai, và nó **thành** thứ ba đúng lúc Concordance vào giữa.
  🔴 **Chèn sai chỗ là một lỗi IM LẶNG tuyệt đối:** nối Concordance vào đuôi cho ra thứ tự
  `Từ điển · Lịch sử · Concordance`, tức AC5 của Story 1.20 **vĩnh viễn không thoả** trong
  khi cả chín cổng vẫn xanh — không cổng nào đọc thứ tự tab. Thứ tự đúng:
  **`Từ điển` · `Concordance` · `Lịch sử`**, khớp `lookup-history-pins.html:103`.
  **Chủ: Story 7.7.**

## Deferred from: 1-21-phim-tat-cau-hinh-lai-duoc (2026-08-11)

- 🔴 **MƯỜI HAI trên hai mươi hàng bàn đo CHƯA CHẠY — vế DOM và vế hai nền tảng của story
  chưa có một bằng chứng runtime nào.** Tám hàng đã đo **bằng máy** trên đường sản phẩm thật
  *(một bộ đo nạp chính `src/commands/index.ts` bằng Node thuần, 24 phép đo, tất cả đạt)*:
  hàng **2** (AC1) · **3** (AC5) · **4**/**5** (AC2 + AC12, có đối chứng `registry.unbound()`
  vẫn nói cũ) · **7** (AC3) · **8** (AC9, có đối chứng âm) · **13** (AC8, hai chiều) ·
  **20** (Bẫy 9). Mười hai hàng còn lại **cần một cửa sổ Tauri thật**, và ghi rõ vì sao:
  **1**/**19** (thị giác — bảng đầy đủ, không thanh phạm vi) · **6** (đóng rồi mở lại
  **tiến trình**) · **9**/**18** (cửa nuốt hợp âm — đòi một listener `window` sống) ·
  **10** (AC11 — câu `shortcuts.key_unknown` hiện ra) · **11** (Bẫy 4 — `Escape` hai nghĩa) ·
  **12** (Bẫy 5 — `⌫` gán được) · **14** (AC13 — sửa tay `global.db` rồi mở app) ·
  **15**/**16** (AC6 — vòng đầy đủ không chạm chuột, trên **cả** macOS lẫn Windows, NFR14) ·
  **17** (UX-DR17 — tiêu điểm quay đúng về nút đã mở). Cộng **ảnh chụp màn hình thật** cho
  mỗi AC thị giác. **Chủ: Ice** *(hàng 16 đòi một máy Windows — cùng hạng món nợ mà
  1.6/1.14/1.16/1.17/1.18/1.19/1.20 để lại và chưa story nào đóng được)*.

- ⚠️ **`spec.keys` và `registry.unbound()` nay ĐÚNG TRONG MỘT NGHĨA HẸP, và cả hai chỉ được
  giữ đúng bằng doc-comment.** Kể từ story này, một lượt gán phím **không** đi qua
  `register()` — nó dựng một `Keymap` mới với một lớp `overrides` — nên hai bề mặt đó trả
  lời **thời điểm cài đặt**, mãi mãi. Chúng vẫn đúng cho mục đích của chúng
  (`check-commands.mjs:1399` đọc `unbound()` để chứng minh AC6 của Story 1.6 trên **bộ mặc
  định của sản phẩm**, và bộ đó không đổi lúc chạy), nhưng một bề mặt đọc chúng để hiển thị
  sẽ **sai im lặng sau lượt gán đầu tiên**.
  🔴 **Cưỡng chế hôm nay là VĂN XUÔI**, không một cơ chế: hai doc-comment ở
  `src/commands/registry.ts` cộng một khối ở `src/commands/index.ts` nói *"màn hình đọc
  `effectiveBindings()`/`effectiveUnbound()`"*. Không cổng nào chặn một `.vue` tương lai
  `import { commandRegistry }` rồi đọc `.unbound()`. **Đường bịt rẻ:** một phép kiểm ở
  `check-commands.mjs` cấm `unbound()` xuất hiện trong `src/**/*.vue`. ~~**Chủ: chưa gán**~~ —
  nhặt lại khi bề mặt thứ hai đọc bảng phím xuất hiện (ứng viên: Story 10.4). **(Chủ: story kế tiếp chạm `spec.keys`/`registry.unbound()`.)**

- ⚠️ **`bindingsEpoch` là một chốt phản ứng THỦ CÔNG, và nó đúng chỉ nhờ kỷ luật.**
  `keymap` là một biến module **thường** ở `src/commands/index.ts` và phải như vậy — tệp đó
  nạp bằng Node thuần ở ba phép kiểm của cổng, nên một `import { ref } from 'vue'` ở đó giết
  cả ba cùng lúc. Vue không có cách nào biết nó vừa đổi, nên `src/config/shortcutsState.ts`
  tăng một bộ đếm ở **mọi** đường ghi và mọi `computed` chạm vào bộ đếm đó. Quên một đường
  ghi ⇒ bảng hiện số cũ, và **không cổng nào đỏ**. Hôm nay có đúng **một** đường ghi
  (`commitBindings`), nên bề mặt sai là nhỏ nhất có thể; nó lớn lên ngay khi có đường thứ hai.
  **Chủ: story đầu tiên thêm một đường ghi keymap thứ hai.**

- ⚠️ **`Escape` KHÔNG gán được làm phím tắt, và đó là một đánh đổi có chủ, không một thiếu
  sót.** Ở trạng thái *đang bắt*, `Escape` là **huỷ lượt bắt** (Bẫy 4 của story) — nên không
  đường nào gán `Escape` hay `Mod+Escape` cho một thao tác. `Escape` có trong `NAMED_CODES`,
  tức tầng dưới **biểu diễn được** nó; chỉ màn hình là không cho. Đánh đổi ngược lại — `⌫` —
  đã được xử đúng chiều kia *(nó bỏ gán ở trạng thái **nghỉ**, và là một hợp âm thường lúc
  đang bắt)*, nên sự bất đối xứng là có thật. Lý do chấp nhận: `Escape` là lối thoát cuối
  cùng của một hộp thoại modal, và mất nó là nhốt người dùng bàn phím trong đúng thứ vừa mở.
  **Đường ra nếu ai đó cần `Escape`:** một cử chỉ thứ hai để chốt lượt bắt *(ví dụ `Enter`
  xác nhận)*, lúc đó `Escape` mới có chỗ. ~~**Chủ: chưa gán.**~~ **(Chủ: story kế tiếp cân nhắc lại phím tắt Escape.)**

- 📝 **LỆCH MOCKUP — bốn chỗ, ghi ra thay vì dựng theo.** `mockups/settings.html`:
  ① `:243-248` vẽ thanh chuyển phạm vi `Toàn cục`/`Tác phẩm` ⇒ **KHÔNG dựng**, thay bằng một
  câu (`shortcuts.scope_note`); `kinds.rs:29-37` cấm bằng chữ và gọi đích danh story này.
  ② `:251-262` vẽ khung điều hướng chín mục Cài đặt ⇒ **KHÔNG dựng**, chín mục đó thuộc Epic
  4/5/6/10 và trỏ tới năng lực chưa tồn tại. ③ `:291-292` Xuất/Nhập bộ phím tắt ⇒ **KHÔNG
  dựng**, 0 AC và một định dạng trao đổi là một hợp đồng phải bảo trì. ④ `:269` ô tìm kiếm /
  tra ngược hợp âm ⇒ **KHÔNG dựng**, 0 AC. Cả bốn giữ nguyên trong mockup — Quyết định #3 của
  Story 1.3: lệch thì **ghi ra**, không sửa mockup.
  → KHÔNG LÀM 2026-08-19 (Story 2.13) — cả bốn lệch đã QUYẾT theo Quyết định #3 của Story 1.3
  ("lệch thì ghi ra, không sửa mockup"): mục này TỰ THÂN là bản ghi của quyết định đó, không
  phải một việc còn chờ làm.

- 📝 **Câu `shortcuts.gesture` diễn giải `⌫` bằng CHỮ (*"phím xoá lùi"*), không bằng ký hiệu
  như mockup.** `settings.html:294` viết *"`⌫` để bỏ gán"*. Màn hình thật viết cả câu ra vì
  cử chỉ này có **hai** trạng thái và ký hiệu trần không nói được trạng thái nào — đúng cái
  bẫy mà Bẫy 5 của story mô tả. Không một AC nào đòi ký hiệu; ghi ra để lượt review không đọc
  nó thành một lượt bỏ sót. **(Chủ: story kế tiếp chạm nhãn phím tắt hiển thị.)**

## Deferred from: rà soát toàn Epic 1 — retrospective (2026-08-11)

*Lượt rà soát tự động toàn epic. Mọi số đo chạy lại trên cây làm việc hôm nay, không lấy
một khẳng định nào của story file làm đúng sẵn. Báo cáo đầy đủ: `epic-1-retro-2026-08-11.md`.*

- ✅ **ĐÓNG — CI Windows đỏ 12 trên 12 lượt, và không story nào biết.** Repo đã đẩy lên
  remote từ 2026-08-05 *(trái với dòng "chưa đẩy lên remote" của Story 1.3, 2026-08-03)*
  và CI đã chạy **12** lần: `macos-26` **XANH** ở mọi lượt hoàn tất, `windows-2025` **ĐỎ**
  ở mọi lượt, luôn ở bước `cargo test`, luôn với `0xc0000139`
  `STATUS_ENTRYPOINT_NOT_FOUND`.
  **Nguyên nhân đo được:** `tauri-build` nhét app manifest qua `tauri-winres` ->
  `embed_resource::compile()`, và hàm đó phát `cargo:rustc-link-arg-BINS`
  (`embed-resource-3.0.11/src/lib.rs:443`) — nhị phân **sản phẩm** có manifest, nhị phân
  **test** thì không. Thiếu manifest, trình nạp gắn `comctl32.dll` **v5** và entry point
  mà tầng Win32 của `tauri` nhập không tồn tại ở phiên bản đó.
  Nhị phân unittest của `src/lib.rs` sống vì nó không chạm `run()` nên cây `tauri` bị loại;
  `tests/config_invariants.rs:105` lấy **địa chỉ hàm** `auratranslate_lib::run` nên ép trình
  liên kết giữ trọn cây đó. Nó là nhị phân đầu tiên đủ nặng để lộ ra, không phải nhị phân
  có lỗi.
  🔴 **Hệ quả lớn hơn một job đỏ:** `cargo test` dừng ở nhị phân tích hợp ĐẦU TIÊN theo thứ
  tự chữ cái ⇒ **12 tệp `tests/**` còn lại chưa từng chạy một lần nào trên Windows** suốt
  Epic 1. Nửa Windows của NFR14 chưa từng có bằng chứng, trong khi cả epic tin rằng nó có.
  **Vá:** `src-tauri/build.rs` phát `/MANIFEST:EMBED` + `/MANIFESTINPUT` qua
  `cargo:rustc-link-arg-`**`tests`** + `src-tauri/windows-app-manifest.xml`. Hẹp hơn bản
  thượng nguồn (`rustc-link-arg` trần) có chủ ý — bản trần nhét manifest hai lần vào nhị
  phân phát hành. Lý do từng lựa chọn ở doc-comment của `build.rs`.

- ✅ **ĐÓNG — cổng thứ mười canh máy dev, KHÔNG canh nhánh.** `check:lint` ra đời ở
  `01be1c2` (2026-08-11) nhưng `ci.yml` không gọi nó. Đã thêm. **Nguyên nhân không phải
  một lượt quên:** kho có HAI danh sách cổng (`package.json` và `ci.yml`) và trước hôm nay
  không phép kiểm nào buộc chúng khớp ⇒ dựng **cổng thứ mười một** `check:gates`
  (`scripts/check-gates.mjs`), ba phép kiểm, có tự kiểm. Nó bắt chính mình ở lượt chạy đầu
  tiên — `check:gates` có trong `package.json` mà chưa có trong `ci.yml` ⇒ ĐỎ đúng chiều.

- ⚠️ **[D2] của lượt review Story 1.3 — chiều ÂM của AC8 nay CÓ lưới tự động.** Điều kiện
  mà [D2] đặt ra (*"nếu `npm run check:scope` chạy được trên runner thì chiều âm có lưới tự
  động và AC8 đóng trọn"*) đã thoả: run `31467748678` và `31468807121`, job `macos-26`,
  bước *"check scope chiều âm (chế độ dev — 403 thật)"* **XANH**. 🔴 **Chưa đóng trọn:**
  cùng bước đó trên `windows-2025` chưa từng chạy tới (job chết ở `cargo test` trước đó).
  Đóng khi có một lượt Windows xanh. **Chủ: Story 1.3.**

- 🔴 **Bốn phép nghiệm thu runner của Story 1.3 vẫn CHƯA ĐỌC, và nay chúng đã hết lý do.**
  AC6 (ba số `.msi` + hai dòng NFR6) · AC7 (thời gian tường + phút tính phí, cache lạnh và
  nóng) · Task 11 hàng 4 (`#[cfg(windows)] compile_error!` làm CHỈ job Windows đỏ) ·
  AC3/Task 4 (rào biên dịch C và WiX v3 trên `windows-2025` — hai nguồn tài liệu nói khác
  nhau). Cả bốn nằm sau đúng một lượt Windows xanh. **Chủ: Story 1.3.**

- ⚠️ **Nợ nghiệm thu thị giác của Epic 1 có HỆ SỐ NHÂN, không phải hằng số.** Mọi bản vá
  tầng DOM đều không đo được bằng bộ cổng hiện có (cổng nạp mã bằng Node thuần — không
  `window`, không DOM), nên mỗi lượt code review chạm DOM lại **sinh thêm** hàng bàn đo:
  Story 1.21 đi từ 12 hàng treo lên **19** SAU khi vá mười phát hiện. Tổng hôm nay:
  **9** hàng của 1.20 + **19** hàng của 1.21, cộng vế thị giác *"kế thừa không đóng"* của
  1.6 · 1.14 · 1.15 · 1.16 · 1.17 · 1.18 · 1.19.
  🔴 **Một dữ kiện phải cân trước khi ai đó đề xuất một bộ chạy test trong trình duyệt:**
  khuyết tật hạng cao nhất của lượt review 1.21 là *"đường chuột của AC2 chết hoàn toàn
  trên macOS vì WKWebView không đặt tiêu điểm cho `<button>`"* — một khác biệt **engine**.
  Một bộ nghiệm thu chạy trong Chrome đóng được lớp DOM trung tính và **KHÔNG** đóng được
  đúng lớp lỗi đắt nhất. Đừng mua sự yên tâm sai ở đây. **Chủ: Ice** *(quyết định về bộ
  chạy test frontend — món nợ này đang treo chờ đúng quyết định đó)*.

- 📝 **Bao phủ FR của Epic 1 ĐỦ.** Đối chiếu bản đồ FR ↔ `**Covers:**` của 25 story:
  **27/27** FR mà bản đồ gán cho Epic 1 đều có story nhận. Khoảng trống của epic là **bằng
  chứng**, không phải **phạm vi**.
  → KHÔNG LÀM 2026-08-19 (Story 2.13) — mục tự thân là một BIÊN BẢN xác nhận bao phủ đủ
  (27/27), không phải một việc chờ làm. Không có chủ vì không có việc để giao chủ.

- 📝 **Số đo của story file khớp thực tế.** Lượt rà soát chạy lại tám cổng, `npm run build`,
  và `cargo test --locked` (**264 xanh · 0 đỏ · 5 ignored** — khớp đúng số Story 1.21 khai).
  Không tìm được một khai sai nào trong các bảng số. Cùng lượt, `tauri build` dựng `.dmg`
  và cả hai cổng `check:scope*` XANH trên runner macOS.
  → KHÔNG LÀM 2026-08-19 (Story 2.13) — mục tự thân là một BIÊN BẢN xác nhận số khớp thực tế
  ("không tìm được một khai sai nào"), không phải một việc chờ làm. Không có chủ vì không có
  việc để giao chủ.

- ✅ **ĐÓNG — khuyết tật thứ HAI, lộ ra ngay khi `cargo test` trên Windows chạy được.**
  Bản vá manifest cho `cargo test` chạy tới `tests/dict_lookup.rs` lần đầu tiên trong đời
  dự án, và cổng parity lược đồ ở đó **ĐỎ**: `fixture_ddl_is_verbatim_from_dict_build_schema`
  so một hằng chuỗi Rust (LF) bằng `contains` với `tools/dict-build/src/schema.rs` đọc từ
  đĩa, mà ảnh `windows-2025` đặt `core.autocrlf=true` nên tệp đó tới nơi mang **CRLF**.
  Cổng đỏ với đúng câu *"lược đồ hai cây đã trôi khỏi nhau"* trong khi hai cây giống hệt
  nhau — thứ trôi là ký tự xuống dòng. Đo ở run `31468807121`.
  **Vá:** `.gitattributes` với `* -text` — cây làm việc bằng đúng byte trong index ở mọi
  nền tảng. 🔴 **KHÔNG dùng `text=auto eol=lf`**: kho có fixture mang CRLF **có chủ ý**
  (`tools/dict-build/tests/fixtures/raw/cc_cedict/cedict.txt`), và `project_contract.rs:490`
  khẳng định bằng chữ *"CRLF phải được GIỮ NGUYÊN"* — một lượt chuẩn hoá lúc commit sẽ đổi
  dữ liệu dưới chân hai phép kiểm cùng lúc. Cộng một dòng chuẩn hoá trong chính hai cổng
  parity (`dict_lookup.rs` · `dict_sources.rs`, cái sau mang **cùng** lỗ ở
  `read_dict_build_schema()`) để chúng nói đúng thứ chúng định nói dưới mọi cấu hình
  checkout. Cổng KHÔNG bị nới.
  ⚠️ **Bài học chung, và nó lớn hơn hai tệp này:** mọi cổng so văn bản nguồn TỪNG BYTE đều
  mang lỗ này, và không lỗ nào trong số đó lộ ra được chừng nào nửa Windows còn chết. Mười
  tệp `tests/**` vẫn chưa từng chạy trên Windows tính tới lượt vá này; mỗi lượt CI xanh
  thêm một bậc sẽ lộ ra lớp tiếp theo. **Chủ: Story 1.3**, cho tới khi có một lượt Windows
  xanh trọn vẹn.

- 🔴 **CÒN MỞ — khuyết tật thứ BA, và nó chạm một bất biến SẢN PHẨM chứ không một tạo tác
  test.** Sau hai bản vá ở trên, **15 trên 15** nhị phân test chạy được trên Windows và
  **263 trên 264** ca XANH *(run `31469843146`)*. Ca duy nhất còn đỏ:
  `store_contract::the_wal_stops_growing_once_it_crosses_the_threshold` — **AC5 của
  Story 1.7**.
  **Số thật** *(`THRESHOLD` 65.536 B · `ROUNDS` 20 · `BLOB` 32 KiB)*: `.db-wal` =
  **889.952 B**, tổng đã ghi = 1.310.720 B, trần của assert = 327.680 B. Stats:
  `threshold_triggered: 51 · frames_checkpointed: 6392 · passive_busy: 0 ·
  idle_triggered: 0 · errors: 0`.
  **Hai mệnh đề của ca này nói hai chuyện khác nhau, và chúng KHÔNG cùng phán quyết:**
  mệnh đề 1 *("chững lại", `after_second <= after_first * 2`)* **ĐẠT** — cơ chế AC5 có
  chạy, 51 lượt theo ngưỡng, 0 lượt bị chặn, 0 lỗi, `idle_triggered = 0` chứng minh vế (a)
  không hề kích hoạt; mệnh đề 2 *("có trần", `< written / 4`)* **TRƯỢT** — WAL đứng ở
  **13,6 lần** ngưỡng. Doc-comment của chính ca đó viết *"Kỳ vọng là **chững lại**, không
  phải co lại"*. Khác biệt nằm ở `walRestartLog` của SQLite: nó chỉ quay WAL về đầu tệp khi
  một giao dịch ghi bắt đầu đúng lúc `nBackfill == mxFrame`, và trên Windows nhịp đó không
  rơi vào nhau — frame vẫn được chép, tệp không bao giờ quay đầu.
  🔴 **KHÔNG vá ở lượt rà soát này, có lý do:** ba đường đi được và cả ba đều cần một thứ
  không có trong tay. ① Nới mệnh đề 2 hoặc gắn `#[cfg(windows)]` cho nó là một **miễn trừ**
  — Ice đã bác thẳng lối này ở lượt review 1.21. ② Sửa tầng `Store` cho WAL quay đầu được
  trên Windows là chạm bất biến AD-11 dựa trên **một** con số từ một runner, không có máy
  Windows để đo lại — đó là đoán. ③ Đo thật trên một máy Windows rồi mới chốt là đường
  đúng, và nó chính là món nợ *"cần một máy Windows"* đang chờ chủ.
  **Câu hỏi cho Ice, đúng một câu:** AC5 nói *"chững lại"* hay nói *"có trần tuyệt đối"*?
  Câu trả lời quyết định đây là **một khuyết tật sản phẩm trên Windows** hay **một assert
  được hiệu chuẩn trên một nền tảng**. Tới lúc đó để nó **ĐỎ** — một pipeline đỏ vì một câu
  hỏi thật tốt hơn một pipeline xanh vì một câu hỏi bị nới. **Chủ: Ice** · liên đới
  **Story 1.7**.

- ✅ **ĐÓNG bằng một QUYẾT ĐỊNH của Ice, 2026-08-11: AC5 nói *"CHỮNG LẠI"*, không nói *"có
  trần tuyệt đối"* — chấp nhận nới trần.** Trần của mệnh đề 2 nới **theo nền tảng**, KHÔNG
  nới toàn cục: Windows `3/4`, macOS giữ nguyên `1/4`. Lý do không nới chung: hạ trần chung
  vứt luôn bảo đảm chặt của nền tảng Ice phát triển hằng ngày cho một khác biệt chỉ tồn tại
  ở nền tảng kia.
  **Hai điểm đo đứng sau con số:** macOS **94.792 B = 7,2%** lượng đã ghi, và **bằng nhau ở
  cả hai đợt** *(WAL quay đầu mỗi lượt)*; Windows **889.952 B = 67,9%**. Chênh **9,4 lần**.
  Trần 3/4 nằm giữa số đo Windows (67,9%) và ngưỡng của *"cơ chế vắng mặt"* (≈100%), đặt
  gần số đo hơn để còn bắt được hồi quy; trần 1/4 của macOS có dư địa 3,5 lần.
  ⚠️ **Trần Windows hiệu chuẩn trên ĐÚNG MỘT phép đo (n = 1).** Điểm đo thứ hai lấy được
  mà không cần một lượt đỏ: `cargo test --test store_contract -- --nocapture` in cả hai số
  cộng tỷ lệ phần trăm. Thông điệp của assert nay mang **cả** `after_first` lẫn một câu
  nhắc phân biệt *một hồi quy tầng Store* với *một trần hiệu chuẩn sai* — mệnh đề 1 và
  `threshold_triggered`/`frames_checkpointed` là hai câu trả lời đó. Đường đóng thật sự vẫn
  là đo trên một máy Windows — **món nợ A5**.

## Deferred from: correct-course — rà soát tài liệu vs mã nguồn (2026-08-11)

*Ice yêu cầu một lượt đối chiếu tài liệu với mã đã triển khai, không nêu trước chỗ nghi ngờ.
Lượt này ĐO trước rồi mới đề xuất: đọc trọn `ARCHITECTURE-SPINE.md` (857 dòng, 44 AD lúc bắt
đầu), quét `epics.md`, chạy lại chín cổng, đối chiếu `package.json` · `Cargo.toml` ·
`capabilities/` · `tauri.conf.json` · `ci.yml` · `.githooks/pre-push` với thứ tài liệu khai.*

**Nền đo được, ghi trước vì nó là thứ giữ lượt rà soát này trung thực:** chín cổng **9/9
XANH**, cây git **sạch**, `capabilities/` đúng **một** tệp `main.json` với **0** permission
plugin, `Cargo.toml` **không** có `default = [...]`, và trạng thái ba story `in-progress`
**khớp** `sprint-status.yaml`. Không tìm được một khai sai nào trong các bảng số của story
file — cùng kết quả với lượt retrospective sáu ngày trước.

### ĐÃ ĐÓNG trong chính lượt này

- ✅ **Hồ sơ chép sai quyết định của Ice về GitHub Actions — bốn chỗ.** Bốn tạo tác ghi
  *"Ice chốt BỎ QUA GitHub Actions"*, đọc như một quyết định kiến trúc vĩnh viễn. Ice đính
  chính 2026-08-11: đó là **TẠM DỪNG**, và lý do là **không có máy Windows để đối chiếu kết
  quả runner**. Khác biệt không phải chữ nghĩa — §10 của retrospective đang giao cho Epic 2
  một điều kiện khởi hành mang chữ *"nay KHÔNG có đường nghiệm thu nào"* như một trạng thái
  đã chốt, trong khi đúng ra nó là một khoảng mù **có điều kiện mở lại**, và điều kiện đó
  chính là món nợ **A5**. Đã sửa: `sprint-status.yaml` (A2) · `epic-1-retro-2026-08-11.md`
  (§9 hàng A2, §10 mục 2) · `.githooks/pre-push` (§Giới hạn).

- ✅ **Cổng thứ mười một canh HAI trong BA danh sách cổng.** `check:gates` ra đời ngày
  2026-08-11 để đóng lỗ *"hai danh sách không ai buộc khớp"*, và **cùng ngày** hook
  `pre-push` sinh ra một danh sách **thứ ba** mà không phép kiểm nào canh. Đo: `package.json`
  khai **11** script `check:*`, hook chạy **9**, chênh 2 (`check:scope`,
  `check:scope:bundled`) có lý do thật nhưng lý do đó chỉ nằm trong một khối chú thích.
  Ngày mai thêm cổng thứ mười hai mà quên hook là **lặp lại nguyên vẹn** sự cố `check:lint`,
  chỉ đổi tệp bị quên.
  **Vá:** `scripts/check-gates.mjs` thêm **Kiểm D** *(cổng thiếu trong hook)* và **Kiểm E**
  *(hook gọi cổng không tồn tại)*, đối xứng đúng cặp A/B sẵn có, cộng `PREPUSH_EXEMPT` — mỗi
  miễn trừ kèm lý do, chép từ chính §Phạm vi của hook.
  🔴 **Chi tiết đắt nhất của bản vá:** bộ đọc trả `null` khi không phân giải nổi vòng lặp
  `for gate in … ;`, **không** trả tập rỗng. Một bộ đọc trả rỗng làm Kiểm D xanh trong khi
  nó chẳng kiểm gì — đúng lớp lỗi *"rỗng im lặng"* mà AD-26 và AD-44 ④ tồn tại để cấm.
  `null` buộc `abort`, tức một lỗi hạ tầng tường minh.
  **Nghiệm thu đỏ-rồi-xanh, bốn ca chạy thật trên bản sao ngoài kho:** ① thêm một cổng thứ
  mười hai vào `package.json` + `ci.yml` mà quên hook ⇒ **A và B XANH, D ĐỎ** *(đúng hình
  dạng sự cố đã xảy ra)* · ② đổi `for gate in` thành `for g in` ⇒ **abort, exit 1**, không
  xanh oan · ③ hook gọi `check:da-bi-xoa` ⇒ **E ĐỎ** · ④ khôi phục ⇒ **exit 0**. Sau bản vá:
  chín cổng vẫn **9/9 XANH**.
  ⚠️ **Chỗ căng đã ghi vào chính tệp thay vì giấu:** dòng kết của `check-gates.mjs` in ra
  *"AC4 của Story 1.3 — MỘT pipeline duy nhất"*, và hook `pre-push` **LÀ** một đường cưỡng
  chế thứ hai. AC4 cấm bằng chữ một **tệp workflow** thứ hai nên hook không phạm chữ; nhưng
  tinh thần AC4 *(một danh sách, không dựa trí nhớ)* chỉ còn đúng **KHI có Kiểm D**. Bản vá
  này không xin ngoại lệ khỏi AC4 — nó là điều kiện để AC4 tiếp tục đúng dưới ba danh sách.

- ✅ **`ARCHITECTURE-SPINE.md` lỗi thời so với mã — tám chỗ, đã đồng bộ.**
  ① **Bảng Stack thiếu 10 phụ thuộc** trong khi chính spine đặt luật *"mỗi phụ thuộc mới
  phải rà GPLv3 và **ghi vào bảng Stack**"* (§Consistency Conventions, hàng *Giấy phép*).
  Bảy trong mười sinh ra rồi mới được ghi — `uuid` từ Story 1.15, ba hàng ESLint từ cổng thứ
  mười, năm gói WebdriverIO cùng plugin từ bộ lái e2e. Quy ước đó bị bỏ lỡ **ba lần liên
  tiếp**. Rà lượt ba theo đúng phương pháp hai lượt trước — **mở tệp `LICENSE` trong nguồn
  đã tải mà đọc**, không tin nhãn registry: **10/10 mang ✓**, thân tệp đều có mệnh đề
  *"Permission is hereby granted, free of charge"*; `uuid` là MIT OR Apache-2.0.
  ② **AD-45 mới — bản phát hành không mở một cổng LẮNG NGHE nào.** AD-15 đếm điểm **RA** và
  không nói gì về chiều ngược lại, nên một máy chủ nghe trên `localhost` đi vào bản người
  dùng cài mà **không phạm một chữ nào** của AD-15. Cơ chế **đã có thật trong mã** (hai lớp
  chặn) và **đã có cổng canh** (`check-deps.mjs` Kiểm 1b) — AD-45 chỉ đặt tên cho một luật
  đang chạy, không đặt việc mới.
  ③ §*"Không dùng, đã loại có lý do"* còn khai kho có **0** plugin Tauri. ④ tên tệp cưỡng
  chế ghi `check-deps.sh`, tệp thật là `.mjs`. ⑤ **cây nguồn thiếu 5 nhánh thật** —
  `src/config/` · `src/selftest/` · `scripts/` · `e2e/` · `.githooks/`; hai cái đầu có lý do
  ghi ở sổ nợ này, nhưng một lý do nằm trong sổ nợ **không thay được một dòng trong cây
  nguồn**. ⑥ hàng mới *Cổng lắng nghe* trong bảng Consistency Conventions. ⑦ `updated` sang
  `2026-08-11`. ⑧ đoạn *Rà NFR15 lượt ba*.
  **Nghiệm thu:** `lint_spine.py` → **0 findings**, 45 AD, bảng Stack 31 hàng.
  ⚠️ **Hai mục giấy phép phải nói thẳng, cả hai ở phần BẮC CẦU chứ không phải hàng Stack:**
  cây npm đi **194 → 530** gói; `@promptbook/utils` mang **CC-BY-4.0** *(đòi ghi công)* và
  `css-value@0.0.1` **không khai giấy phép**. Cả hai chỉ devDependency, không vào sản phẩm —
  nhưng chúng là hai mục duy nhất trong 530 gói không thuộc nhóm dễ dãi.

- ✅ **Bốn năng lực đã dựng mà KHÔNG tạo tác quy hoạch nào nhận.**
  `grep -ni "e2e|webdriver|eslint|wdio" epics.md` cho **0 kết quả**, trong khi bốn thứ sau
  sống trong mã: cổng thứ mười `check:lint` (`01be1c2`) · cổng thứ mười một `check:gates`
  (`b53002f`) · bộ lái e2e (`3a54628`, `7127f5f`) · hook `pre-push` (`8a9992b`).
  **Cách xử, và lý do không dựng bốn story:** ba trong bốn nằm gọn trong hiến chương sẵn có
  của Story 1.3 — AC4 của nó viết bằng chữ *"các luật cưỡng chế bằng test sinh ra ở epic
  sau… gắn vào **chính pipeline này**"*. Hai cổng mới chỉ là hai **thể hiện** của luật đó.
  Thứ AC4 chưa phủ là **số lượng danh sách cổng** và **đường cưỡng chế lúc CI vắng mặt**, nên
  Story 1.3 nhận **hai AC mới** đúng hai điểm ấy — cả hai chép lại thứ đã chạy thật, không
  đặt việc mới. Bộ e2e thì khác bản chất *(một năng lực nghiệm thu, phục vụ món nợ xuyên
  chín story, và **chưa xong**)* ⇒ **Story 1.22** mới.

### CÒN MỞ

- 🔴 **FR107 dựng trên GitHub Actions — Ice chốt 2026-08-11: ghi nợ, hoãn tới Epic 10.**
  FR107 hứa *"build công khai qua GitHub Actions, để bất kỳ ai cũng kiểm chứng được binary
  khớp với mã nguồn"*, và Story 10.1 nhận nguyên phạm vi đó. Lượt tạm dừng hôm nay chưa chạm
  Epic 10 vì Epic 10 còn cách **chín epic**.
  **Vì sao KHÔNG sửa PRD hôm nay, và đây là một lựa chọn chứ không một lượt bỏ qua:** tạm
  dừng không phải bỏ, và sửa một FR dựa trên một ràng buộc có thể đã biến mất trước lúc
  Epic 10 tới là đổi tài liệu bằng **phỏng đoán** — đúng thứ doctrine *"đo trước khi tin"*
  cấm. Thay vào đó, một **điều kiện khởi hành** đã ghi thẳng vào Story 10.1 với hai câu hỏi
  phải trả lời trước khi dựng: (1) GitHub Actions quay lại chưa, nếu chưa thì FR107 còn
  đường nào khác; (2) nợ **A5** có chủ chưa, vì FR105/FR106 hứa cả `.dmg` lẫn `.msi`.
  **Chủ: Ice** · mở lại ở **Story 10.1**.

- 🔴 **Ba khuyết tật của bộ e2e, nay có chủ là Story 1.22 **(Chủ: Story 1.22.)** *(trước lượt này chúng chỉ sống
  trong `proposal-tauri-window-automation-2026-08-11.md` §8, không tạo tác nào chịu trách
  nhiệm)*: ① bộ e2e dùng chung `$APPDATA` với ứng dụng **thật** của người chạy — ca gán phím
  **sửa cấu hình thật của Ice**, và cách dọn hôm nay là bấm nút *"Về mặc định"*, tức vá triệu
  chứng; ② `element.click()` bắn `click` **trước** `focusin` nên mọi tương tác có thứ tự phải
  đi Actions API; ③ máy chủ nhúng bám cổng cố định **4445** nên hai tệp spec cùng lượt làm
  phiên thứ hai trượt. Mục ① phải đóng **trước** khi dựng thêm bất kỳ hàng bàn đo nào.

- ⚠️ **Ba món nợ tài liệu cũ, xác nhận VẪN MỞ trong lượt này** *(không phải phát hiện mới —
  ghi lại để chúng không trôi thêm một epic)*: AD-23 còn liệt kê `$RESOURCE/dict/**` trong
  khi `tauri.conf.json` chỉ khai `$RESOURCE/fonts/**` *(Chủ: Ice.)* · sơ đồ mermaid của
  AD-13 còn cạnh `dict --> matching`, lệch khỏi thân Rule của AD-17 *(Chủ: Winston.)* ·
  bảng phỏng đoán Porter ở AD-44 ③ nay đã có số đo thật từ Story 1.12 mà chưa thay vào
  *(Chủ: Winston.)*.

### Ngoài phạm vi lượt này, ghi thẳng

Lượt rà soát này đối chiếu **tài liệu quy hoạch với hình dạng mã** — bảng Stack, bất biến
kiến trúc, cây nguồn, danh sách cổng, bao phủ story. Nó **KHÔNG** đối chiếu từng AC của 25
story với hành vi thật của mã; phép đó cần chạy lại 28 hàng bàn đo thị giác và một máy
Windows, tức đúng hai món nợ **A4** và **A5** đang chờ chủ. Không lượt đọc tài liệu nào thay
được hai món đó, và lượt này không giả vờ thay.

## Deferred from: Story 1.22 — C1, chuyển hướng `$APPDATA` của bộ e2e (2026-08-11)

- ✅ **ĐÓNG — bộ e2e thôi dùng chung `$APPDATA` với ứng dụng thật của người chạy.** Đây là
  AC2 của Story 1.22 và là món chặn mọi hàng bàn đo còn lại.

  **🔴 Phép đo quyết định hình dạng bản vá — và nó lật phương án rẻ.** Cách rẻ là đổi `HOME`
  của tiến trình con. Đo trên chính cây đang ghim (`dirs-6.0.0` · `dirs-sys-0.5.0`):

  | Nền tảng | `dirs::data_dir()` đi qua | Đổi được bằng biến môi trường? |
  |---|---|---|
  | macOS | `home_dir()/Library/Application Support`, `home_dir()` đọc `$HOME` trước | **CÓ** |
  | Windows | `known_folder(FOLDERID_RoamingAppData)` — Known Folder API | **KHÔNG** |

  `dirs-sys` gọi thẳng Shell API trên Windows và **bỏ qua** `%APPDATA%`. Nên phương án
  `HOME` là một bản vá **chạy trên macOS và hỏng im lặng trên Windows** — đúng lớp lệch nền
  tảng NFR14 tồn tại để chặn, và hôm nay nửa Windows **không có đường nghiệm thu nào** để
  phát hiện ra. Đó là hai khuyết tật chồng lên nhau, nên phương án rẻ bị loại.

  **Bản vá:** `AURATRANSLATE_E2E_DATA_DIR` đọc trong Rust, chặn bằng **đúng hai lớp của
  AD-45** — `debug_assertions` **và** `feature = "wdio"`, cùng khuôn với chính plugin
  WebDriver. Bản phát hành không có một dòng mã nào đọc biến đó: nhánh
  `not(all(debug_assertions, feature = "wdio"))` của `data_dir_override()` trả `None` **theo
  kiểu**. Phân giải giống hệt nhau trên hai nền tảng.

  **Chính sách tách khỏi phép đọc** (`data_dir_override_from_raw`, **không** bị `cfg` gác):
  `std::env::set_var` là `unsafe` từ edition 2024 và một ca đặt biến môi trường còn đua với
  các ca chạy song song; hàm thuần thì test được mà không chạm tiến trình, và luật được kiểm
  ở **mọi** bộ feature — kể cả bộ mặc định mà hook `pre-push` chạy. Đường dẫn **tương đối bị
  TỪ CHỐI**, không được phân giải: nó phân giải theo thư mục làm việc của tiến trình con và
  sẽ đẻ một `global.db` ở một chỗ bất kỳ trong kho mà không ai báo.

  **Ba bất biến mới** ở `tests/config_invariants.rs`: chính sách từ chối rỗng/tương đối ·
  phép đọc chỉ sống sau hai lớp gác *(ca đọc mã nguồn, vì hai lớp gác là tính chất **lúc
  biên dịch** và một nhị phân test chỉ quan sát được đúng một bộ feature mỗi lượt)* · **tên
  biến ở Rust và ở `wdio.conf.mjs` phải khớp từng ký tự**.

  **Cộng một phép TỰ KIỂM lúc chạy** ở `onComplete`: `global.db` phải nằm trong thư mục tạm,
  nếu không thì lượt chạy ĐỎ. 🔴 Vì sao cần cả hai lớp canh: hình dạng hỏng ở đây **không có
  triệu chứng** — app lặng lẽ quay về `$APPDATA` thật và **mọi ca vẫn xanh**, vì một kho thật
  cũng là một kho mở được.

  **Nghiệm thu — bốn phép đo chạy thật, không suy đoán:**

  | Phép đo | Kết quả |
  |---|---|
  | Hai spec, mỗi cái một lượt riêng | **xanh** *(`webkit 605.1.15 macos`)* |
  | Băm SHA-256 của `global.db` thật, trước và sau hai lượt | **không một byte nào đổi** |
  | Ca ĐỎ — mô phỏng trôi tên biến ở phía JS | spec **xanh** mà tự kiểm **chặn**; kho thật **bị động** đúng như dự đoán, rồi khôi phục từ sao lưu **khớp từng byte** |
  | Chín cổng · `npm run build` · `cargo test --locked` | 9/9 xanh · xanh · **267 xanh · 0 đỏ · 5 ignored** *(264 → 267, cộng đúng ba ca mới)* |

  ⚠️ **Bản vá tự bắt được lỗi của chính nó, ghi lại vì nó là bằng chứng phép tự kiểm có
  răng:** bản đầu của `onComplete` tìm kho ở `<tạm>/com.auratranslate.desktop/global.db`.
  Sai — `app_data_dir()` của Tauri là `data_dir()/<định danh>`, còn biến môi trường **thay
  thế trọn** kết quả đó, nên kho nằm thẳng trong thư mục tạm. Lượt chạy thật đầu tiên ĐỎ
  đúng vào chỗ ấy, dù móc chuyển hướng hoạt động đúng. Cả hai chỗ nay ghi mệnh đề đó ra chữ.

- 📝 **Nút *"Về mặc định"* trong hai spec nay là dư, KHÔNG gỡ ở lượt này.** Nó ra đời để dọn
  `global.db` thật giữa các ca; sau bản vá mỗi lượt đã có kho riêng. Giữ lại có chủ ý: gỡ nó
  là đổi hành vi của hai ca đang xanh trong cùng một lượt vá hạ tầng, và nó vẫn giữ một
  nghĩa thật — cô lập **giữa các ca trong cùng một spec**, thứ thư mục tạm theo **lượt chạy**
  không cho. Mở lại khi có ca thứ ba trong một spec. **Chủ: Story 1.22.**

## Deferred from: Story 1.22 — C2, chuột thật thay `element.click()` (2026-08-11)

- ✅ **ĐÓNG — luật *"thứ tự sự kiện đi Actions API"* nay là một CỔNG, không một quy ước.**
  `realClick()` tách thành `e2e/support/pointer.mjs` *(một cài đặt dùng chung)*;
  `eslint.config.js` cấm mọi lời gọi `.click()` trong `e2e/**` bằng `no-restricted-syntax`;
  `check:lint` chạy `eslint src e2e` từ hôm nay — trước đó **`e2e/` không được cổng nào canh**.
  Lệnh cấm **toàn phần**, rộng hơn phát biểu AC3 *(chỉ đòi ở nơi thứ tự có nghĩa)*, có chủ ý:
  câu hỏi *"hàng này có phụ thuộc thứ tự không"* đã bị trả lời sai một lần rồi — xem mục kế
  tiếp. Ngoại lệ thật đi qua `eslint-disable-next-line` kèm lý do, và
  `reportUnusedDisableDirectives: 'error'` sẵn có bắt được ngoại lệ hết cần.
  **Nghiệm thu đỏ-rồi-xanh:** cấy lại `.click()` ⇒ ĐỎ kèm câu chỉ thẳng sang `realClick()`;
  cấy một `eslint-disable` **hết cần** ⇒ cũng ĐỎ; gỡ ⇒ XANH.

- 🔴 **KHUYẾT TẬT SẢN PHẨM lộ ra ngay khi bàn đo đi chuột thật — UX-DR17 hỏng trên WKWebView.**
  Đây là phát hiện đắt nhất của lượt này, và nó là **lý do cả phương án e2e tồn tại**.

  Đổi `shortcuts-focus` sang `realClick()` ⇒ ca ĐỎ ngay lượt đầu:
  `Expected substring: "data-shortcuts-open"` · `Received string: "section|mode"`.

  **Cơ chế, đọc từ `ShortcutsOverlay.vue:58-80`:** đường lui lưu `document.activeElement`
  **lúc mở** rồi trả về đúng node đó. Trên WKWebView, nút `<button>` **không nhận tiêu điểm
  khi bấm**, nên node đã lưu là điểm vào focus của chế độ (`section.mode`), không phải nút mở.
  ⚠️ **Nhánh dự phòng `querySelector('[data-…-open]')` KHÔNG cứu được ca này** — nó chỉ chạy
  khi node đã lưu **rời DOM**, mà `section.mode` thì vẫn ở nguyên đó. Một đường lui không bao
  giờ chạy tới không phải một đường lui.

  🔴 **Vì sao nó sống sót suốt Epic 1:** bàn đo cũ bấm bằng `element.click()`, và lệnh đó
  **có** đặt tiêu điểm — tức bàn đo XANH nhờ một hành vi mà chuột thật không có. Đúng hình
  dạng *"xanh trên một sản phẩm đang hỏng"* mà lệnh cấm ở trên vừa dựng để chặn, và nó đã xảy
  ra thật trước khi ai kịp lo xa về nó.

  **Ice chốt 2026-08-11: vá GỐC, không vá ở lớp phủ.** `@mousedown="focusOnPointerDown($event)"`
  trên nút mở, cùng khuôn `@mousedown` mà `config/shortcutsState.ts` đã dùng cho đúng khuyết
  tật engine này. Vá gốc làm `activeElement` lúc mở thành đúng nút, nên đường lui **sẵn có**
  tự đúng — thay vì thêm một nhánh thứ hai chỉ chữa cho một lớp phủ.
  **Nghiệm thu:** trước vá ĐỎ với `section|mode` → sau vá **XANH**; `shortcuts-capture-mouse`
  vẫn xanh (không hồi quy); chín cổng · `npm run build` · `cargo test --locked` đều xanh. **(Chủ: Ice — quyết định hình dạng nghiệm thu tay B10/F8, `epic-2-retro-2026-08-18.md:381`; mục này chờ B10.)**

- ⚠️ **`AttributionOverlay.vue` (Story 1.19, đang `done`) đã VÁ THEO CÙNG NGUYÊN NHÂN, nhưng
  mệnh đề của nó CHƯA ĐO ĐƯỢC.** Tệp đó mang khuôn giống hệt (`:57-70`) — chính doc-comment
  của `ShortcutsOverlay` ghi *"khuôn và lý lẽ chép từ `AttributionOverlay.vue`, cả hai vế"* —
  nên nút mở của nó nhận cùng bản vá.
  🔴 Ghi ra thay vì để nó trông như đã nghiệm thu: đây là một bản vá **theo nguyên nhân đã đo
  ở chỗ khác**, không theo một phép đo của chính nó. Spec đã dựng
  (`e2e/specs/attribution-focus.e2e.mjs`) nhưng **`skip` có lý do in ra màn hình**.
  **Chủ: Story 1.22.**

- 🔴 **Hai món chặn đã đo được, và món thứ hai LỚN HƠN nó trông.**
  ① Nút `[data-attribution-open]` sống trong panel Lookup ⇒ chỉ tồn tại ở chế độ `workspace`,
  mà app khởi động ở `library` (`modes/modeState.ts:33`). Cần một **fixture mở Tác phẩm**.
  ② 🔴 **`$APPDATA` không phải bề mặt dữ liệu thật duy nhất mà bộ e2e chạm tới.** AC2 của
  Story 1.22 chuyển hướng `$APPDATA`; thư mục gốc Library đi đường **khác hẳn** —
  `document_dir()` ⇒ `~/Documents/AuraTranslate/` (`commands/project.rs:60`). Nên một fixture
  tạo Tác phẩm hôm nay sẽ ghi vào thư mục **Documents THẬT** của người chạy, tức tái lập đúng
  lớp lỗi mà C1 vừa đóng, chỉ ở một thư mục khác. **Chuyển hướng Library root phải làm TRƯỚC
  fixture**, không sau. **Chủ: Story 1.22.**

## Deferred from: Story 1.22 — bề mặt dữ liệu thật THỨ HAI (2026-08-11)

- ✅ **ĐÓNG — thư mục gốc Library đi vào thư mục tạm, không vào `~/Documents` thật.**

  **Bề mặt này tìm ra bằng cách ĐỌC MÃ, không bằng cách mất dữ liệu thêm một lần.** AC2 đóng
  `$APPDATA`; lượt chuẩn bị fixture cho spec Attribution mới lộ ra rằng thư mục gốc Library
  đi một đường **hoàn toàn khác** — `app.path().document_dir()` ⇒ `~/Documents/AuraTranslate/`,
  phân giải ở `commands::project::default_library_root` (AD-23, scope động). Một bàn đo tạo
  Tác phẩm sẽ ghi vào Documents THẬT của người chạy, tức tái lập nguyên vẹn lớp lỗi mà AC2
  vừa đóng, chỉ ở một thư mục khác.

  **Bản vá:** `AURATRANSLATE_E2E_LIBRARY_ROOT`, cùng hai lớp gác AD-45 và **cùng một chính
  sách** với móc thứ nhất — hàm thuần đổi tên `data_dir_override_from_raw` →
  `absolute_dir_override_from_raw`, vì hai móc khác nhau ở **cái gì bị chuyển hướng**, không
  ở **giá trị nào hợp lệ**. Hai biến chứ không một: `global.db` và `.atproj/` là **hai vai
  khác nhau** trong AD-7 với ranh giới sở hữu **cứng**; gộp chúng vào một biến là dạy người
  đọc rằng chúng cùng một chỗ.

  **Ba bất biến mở rộng để canh CẢ HAI tên** *(danh sách `ENV_NAMES`, phải mọc theo mọi móc
  mới — một móc thứ ba không có tên ở đó là một đường ghi không ai canh, đúng cách bề mặt thứ
  hai đã lọt qua AC2)*.

  🔴 **Hai hàng rào, hai chiều, và lý do cần cả hai:** móc `$APPDATA` có một phép tự kiểm
  **dương tính** *(`global.db` phải nằm trong thư mục tạm)*. Móc Library thoạt nhìn không có
  đối ứng — chưa bàn đo nào tạo Tác phẩm, nên thư mục tạm rỗng dù móc chạy đúng hay sai, và
  một phép kiểm dương tính bịa ra sẽ **luôn xanh mà không canh gì**. Đóng bằng hai thứ:
  ① `onComplete` đi chiều **ÂM** — thư mục Documents thật phải y nguyên; đúng một cách tầm
  thường hôm nay, và **tự có răng** vào ngày fixture đầu tiên xuất hiện, kể cả khi người viết
  fixture quên đọc `wdio.conf.mjs`;
  ② `e2e/specs/library-root-redirect.e2e.mjs` đi chiều **DƯƠNG** — nó **thật sự tạo một Tác
  phẩm** bằng cách gọi thẳng `create_work_from_text` qua IPC, rồi đọc đĩa bằng `node:fs`.
  Gọi IPC chứ không đi giao diện có lý do: câu hỏi là *"`.atproj` rơi vào thư mục nào"*, và
  một đường đi qua form sẽ đo **giao diện nhập** thay vì đo **đường ghi**.

  **Nghiệm thu — đỏ-rồi-xanh chạy thật:**

  | Phép đo | Kết quả |
  |---|---|
  | Spec chiều dương | **XANH** — `.atproj` nằm trong thư mục tạm, vắng mặt ở Documents thật |
  | Ca ĐỎ — biến đặt sai *(đường tương đối, thứ chính sách từ chối)* | **cả hai** hàng rào nổ: khẳng định của spec, và hàng rào âm *(Documents thật 21 → 22 mục)* |
  | Dọn Tác phẩm lạc rồi đối chiếu | Documents thật **khớp ảnh chụp ban đầu, 21 mục** |
  | Chín cổng · `npm run build` · `cargo test --locked` | 9/9 xanh · xanh · xanh |

  ⚠️ **Ca đỏ đầu tiên SAI hình dạng, ghi lại vì bài học dùng được:** lượt đầu mô phỏng bằng
  cách đổi **tên hằng** trong `wdio.conf.mjs`. Nhưng tên đó là thứ **chính spec** đọc để biết
  thư mục tạm, nên spec trượt **trước khi** tạo Tác phẩm và hàng rào âm không hề bị chạm — một
  ca đỏ **không chứng minh gì**. Ca đúng là mô phỏng một biến **đặt sai giá trị**, vì đó mới
  là hình dạng hỏng thật.

  ⚠️ **Giới hạn của hàng rào âm, ghi thẳng:** `mtimeMs` của thư mục chỉ đổi khi có mục được
  **thêm hay xoá**, nên một lượt ghi **đè** lên `.atproj` sẵn có sẽ lọt. Đóng nốt vế đó cần
  quét đệ quy cả cây — đắt, và chưa cần vì hôm nay không đường mã nào của bộ e2e mở được một
  Tác phẩm có sẵn. Mở lại khi có fixture mở Tác phẩm.

- 📝 **Fixture cho spec Attribution nay KHÔNG còn bị chặn bởi dữ liệu thật.** Hai bề mặt đã
  đóng, và `library-root-redirect.e2e.mjs` vừa chứng minh một Tác phẩm tạo được từ trong bàn
  đo. Việc còn lại là thuần giao diện: từ chế độ `library` sang `workspace` với Tác phẩm vừa
  tạo, để `[data-attribution-open]` tồn tại. **Chủ: Story 1.22.**

## Deferred from: Story 1.22 — fixture workspace, và một AC được làm rõ (2026-08-12)

- ✅ **ĐÓNG — fixture `workspace` chạy được.** `e2e/support/workspace.mjs`: tạo Tác phẩm qua
  IPC rồi `Mod+2` vào chế độ `workspace`, đợi tới khi `[data-attribution-open]` có mặt THẬT
  *(dải chip nguồn chỉ render khi `dictSources.length > 0`)*. Hai lựa chọn có chủ ý:
  **không** đi qua form Library *(nó không có một mối nối `data-` nào — `v-model` trên
  `<input>` trần — nên một fixture bám cấu trúc DOM sẽ vỡ ở lượt đổi bố cục đầu tiên, và
  làm MỌI hàng dùng nó đỏ vì lý do không liên quan)*; **không** thêm `data-` vào ba tab chế
  độ chỉ để bàn đo chọn được *(`data-shortcuts-open` và `data-attribution-open` tồn tại vì
  **sản phẩm** cần chúng, không vì bàn đo — bàn phím tránh hẳn câu hỏi tiền lệ đó)*.

- 🔴 **PHÁT HIỆN — bản vá `@mousedown` KHÔNG chuyển được sang nút trong panel, và tôi đã
  khuyến nghị sai.** Lượt trước tôi đề xuất Ice vá gốc bằng `@mousedown` cho **cả hai** nút
  mở. Nó **có tác dụng thật** ở nút titlebar *(`shortcuts-focus` xanh)* và **bị vô hiệu** ở
  nút Attribution — tức với `AttributionOverlay` đó là một bản vá **không chạy**. Đã gỡ khỏi
  `LookupPanel.vue`; giữ ở `App.vue` vì chỗ đó đo được là có tác dụng.

  **Triệu chứng đo được:** trên WKWebView, nút mở nằm trong `section.panel[tabindex="-1"]`
  **không giữ nổi tiêu điểm** — đặt lên nút thì nó rơi lên khung panel **đồng bộ, ngay trong
  lời gọi `focus()`**: `focusout ← button` rồi `focusin → section.panel`.

  🔴 **Bốn giả thuyết bị BÁC BẰNG ĐO, không bằng lập luận:** ① hành vi mặc định của
  `mousedown` — `preventDefault()` không đổi gì; ② dockview cướp tiêu điểm — `dockview-core`
  chỉ dùng `focusin` ở `popupService`; ③ mã ứng dụng — liệt kê trọn `.focus()` trong `src/`,
  không chỗ nào chạy trên đường này; ④ nút không phải tab stop — `tabindex="0"` không đổi gì.
  ⚠️ **Nguyên nhân KHÔNG được đặt tên.** Còn lại là engine, và tôi dừng ở đó thay vì đoán
  tiếp — khác biệt duy nhất giữa hai nút là tổ tiên `tabindex="-1"`. Đường đào tiếp, nếu có
  ngày cần: dựng một trang tối giản NGOÀI kho để cô lập hành vi WKWebView. **(Chủ: một story kế tiếp — điều tra tiêu điểm WKWebView, nguyên nhân chưa đặt tên.)**

- ✅ **ĐÓNG — `focusReturnTargetOnOpen` ở `src/commands/focus.ts`, luật CHUNG cho hai lớp phủ.**
  UX-DR17 hứa tiêu điểm về **nút đã mở**; lưu `document.activeElement` lúc mở chỉ là một phép
  **xấp xỉ** của lời hứa đó, đúng khi engine chịu focus nút và sai khi không.
  🔴 Luật **hẹp hơn** *"luôn ưu tiên nút mở"*: chỉ ưu tiên khi tiêu điểm đang ở chính nút hoặc
  ở một **tổ tiên** của nó — tức đúng hình dạng *"cú bấm đã rơi vào nút, engine đỗ tiêu điểm ở
  khung ngoài"*. Ưu tiên vô điều kiện sẽ hỏng đường **bàn phím**: cả hai lớp phủ là command
  gán phím được (Story 1.21), và khi người dùng mở bằng phím lúc đang gõ ở panel khác thì
  *"nút đã mở"* không tồn tại — trả tiêu điểm về nút khi đó là ném họ ra khỏi chỗ đang làm.
  Nhà của nó là module tự khai *"nửa thứ hai của AD-34"*, không một tệp tiện ích mới.

- ⚠️ **AC11 của Story 1.19 được LÀM RÕ — Ice chốt 2026-08-12.** Đích là **nút mở HOẶC một tổ
  tiên của nó**. Đây là một lần **nới có chữ ký**, phạm vi hẹp: UX-DR17 của Story 1.21 giữ
  nguyên mệnh đề chặt vì nút của nó ở titlebar. Toàn văn cùng lý do và số đo: §AC11 của
  `1-19-bat-tat-nguon-tu-dien-va-ghi-cong.md`.
  **Khẳng định thay thế vẫn có răng, nghiệm thu đỏ-rồi-xanh:** cắt đường trả tiêu điểm ⇒ ĐỎ
  với `body`; khôi phục ⇒ XANH. Nó cũng đỏ khi tiêu điểm về một panel **khác** — hình dạng
  `div.original tok-source-cjk` đã quan sát được thật trong lúc dựng ca.
  → KHÔNG LÀM 2026-08-19 (Story 2.13) — mục tự thân là bản ghi một quyết định ĐÃ CHỐT (Ice,
  2026-08-12) kèm phép kiểm đỏ-rồi-xanh đã có, không phải một việc chờ làm.

- 📌 **Câu hỏi để ngỏ cho Story 10.4** *(sở hữu nửa còn lại của màn Attribution)*: dời nút mở
  ra **titlebar**, cạnh nút phím tắt? Chỗ đó đo được là tiêu điểm dính, nên nó đóng luôn mệnh
  đề chặt. Chạm UX và `mockups/`, nên không quyết trong lượt này. **Chủ: Ice.**

## Deferred from: Story 1.22 — C3, và một giới hạn hoá ra đã tự hết hiệu lực (2026-08-12)

- ✅ **ĐÓNG bằng một PHÉP ĐO, không bằng một bản vá.** C3 sinh ra để sửa giới hạn số 3 của bộ
  e2e: *"một spec = một phiên app, máy chủ nhúng bám cổng cố định 4445, chạy hai tệp trong
  cùng một lượt làm phiên thứ hai trượt"*, với đường ra đề xuất là **cổng cấp theo worker**.

  **Việc đầu tiên là chạy thử, không phải viết mã.** Bộ nay có **bốn** spec:
  `npm run test:e2e` không kèm `--spec` ⇒ **4/4 XANH**, hai lượt liên tiếp, **3m07** và
  **3m04**. Triệu chứng **không còn tái lập được**.

  ⇒ *"Cổng theo worker"* là công việc cho một vấn đề **đã biến mất**. Nếu viết nó ra hôm nay,
  ta có thêm một cơ chế không ai kiểm chứng được là cần thiết, cộng một lượt tự khen đã sửa
  một thứ chưa chắc từng hỏng vì lý do ta nghĩ.

  🔴 **Nguyên nhân lượt trượt cũ KHÔNG được chẩn đoán.** Nó biến mất đâu đó trong lúc C1 và
  C2 đi qua, và tôi **không gán công** cho một bản vá nào mà không có phép đo nói thế. Ghi ra
  để ai gặp lại triệu chứng biết nó **từng** có thật và biết nó đã tự hết ở đâu.

- 🔴 **QUYẾT ĐỊNH: bộ e2e chạy TUẦN TỰ (`maxInstances: 1`), không song song.** Đây là một
  lựa chọn có lý do, không một chỗ chưa làm tới — và lý do đầu **không phải một rủi ro cần
  đo** mà là một hồi quy **đúng theo cấu tạo**:

  ① `onPrepare` cấp **một** `$APPDATA` tạm và **một** thư mục Library tạm cho **cả lượt**.
  Hai app chạy song song dùng chung chúng ⇒ đúng trạng thái mà AC2 vừa đóng, chỉ đổi từ
  *"e2e đụng dữ liệu người dùng"* thành *"hai ca e2e đụng nhau"*. Và phép tự kiểm ở
  `onComplete` *(`global.db` phải nằm trong thư mục tạm)* trở thành **mơ hồ**: nó không phân
  biệt được app nào đã ghi.

  ② Mọi spec trong bộ khẳng định trên `document.activeElement`. Hai cửa sổ **thật** trên cùng
  một desktop macOS tranh tiêu điểm ở tầng hệ điều hành. ⚠️ Đây là rủi ro **CHƯA ĐO** — ghi
  đúng mức độ chắc chắn của nó, không dựng nó thành một dữ kiện. Lý do ① một mình đã đủ.

  **Cái giá đã biết:** 3 phút cho cả bộ. **Điều kiện mở lại:** cấp thư mục tạm **theo
  worker** trước, rồi đổi phép tự kiểm ở `onComplete` theo — đúng thứ tự đó, không ngược lại.
  → KHÔNG LÀM 2026-08-19 (Story 2.13) — một quyết định có lý do cấu tạo, không một chỗ chưa
  làm tới (mục tự khai). Điều kiện mở lại đã ghi ngay trong mục: cấp thư mục tạm theo worker
  trước khi đổi phép tự kiểm ở `onComplete`.

- 📝 **Tài liệu đã dọn ở ba chỗ**, vì lời khuyên cũ nay **tốn tiền của người đọc**: nó bảo
  chạy từng tệp bằng `--spec`, tức bốn lượt khởi động app thay vì một. `e2e/wdio.conf.mjs`
  §Giới hạn 3 + khối `Chạy:` · AC tương ứng của Story 1.22 ở `epics.md`.
  → ✅ **ĐÃ ĐÓNG 2026-08-19 (Story 2.13) — phép kiểm đã chạy: đối chiếu với `e2e/wdio.conf.mjs`
  hôm nay.** `:27` mang `§Giới hạn — ba thứ ĐO ĐƯỢC`; `:150` mang khối `Chạy:` khuyên
  `npm run test:e2e` (cả bộ, một lượt khởi động), không còn khuyên chạy từng tệp bằng `--spec`.
  Ba chỗ dọn tài liệu mà mục này khai đã tồn tại thật trên cây nguồn hôm nay.

## Deferred from: A4 — cỡ chữ vỏ giao diện, và một đính chính về độ ổn định của bộ e2e (2026-08-12)

- ✅ **ĐÓNG — tầng vỏ giao diện nâng một bậc lên mốc macOS 13px. Ice chốt sau nghiệm thu A4.**
  Ice xem bằng mắt và chốt: *"phần thẩm mỹ tốt rồi, nhưng 11,5px là quá nhỏ để đọc"*.

  **Mốc đối chiếu, không phải khẩu vị:** giao diện hệ thống macOS chạy ở **13px**; tầng vỏ cũ
  ở **10–12px**, tức **dưới mặc định của hệ điều hành** trên chính nền tảng Ice dùng hằng ngày.

  | Token | Cũ | Mới | Số chỗ dùng |
  |---|---|---|---|
  | `ui-md` · `ui-md-strong` · `ui-md-wrap` | 12px | **13px** | 29 · 3 · 11 |
  | `ui-sm` | 11,5px | **12px** | 16 |
  | `ui-label` | 10px | **11px** | 11 |
  | `ui-mono` | 10,5px | **11,5px** | 4 |
  | `head-height` · `titlebar-height` · `status-height` | 34 · 38 · 32px | **36 · 40 · 34px** | — |

  Tầng **nội dung** (14,5–24px) **không đụng tới**. Ba thanh nâng theo để giữ tỉ lệ khoảng
  thở — Ice chốt phương án đó thay vì ép chữ to vào chiều cao cũ.

  🔴 **Vì sao không nâng riêng `ui-sm` như câu hỏi của Ice:** lên 12px thì nó **bằng** `ui-md`
  ⇒ thừa một token; lên 12,5px thì **nhãn phụ to hơn tiêu đề panel** ⇒ đảo trật tự phân cấp.
  Câu trả lời phải áp cho cả tầng.

  **Ba nơi khai cùng một con số, sửa cả ba** *(thiếu một là cổng đỏ — đó là thiết kế đúng, nó
  buộc lượt đổi phải có chữ ký)*: `src/tokens/tokens.json` · **bảng đóng băng viết cứng trong
  chính cổng** `scripts/check-tokens.mjs` · `DESIGN.md` §Bảng token typography + §Spacing.

  ⚠️ **Lật một quyết định có lý do, ghi ra chứ không lặng lẽ đè.** `DESIGN.md` bảo vệ tầng vỏ
  nhỏ bằng chữ: *"ghìm chặt … giữ được mật độ của một nhạc cụ nghề nghiệp"*. Câu đó **vẫn
  đứng** và nay mang một khối ghi rõ nó **đã được cân một lần và thua**: mật độ là một giá trị
  thật, nhưng nó không thắng được việc chữ khó đọc với chính người dùng duy nhất.

  **Nghiệm thu bằng phép đo trên app thật, không bằng suy luận:** titlebar render **40px**,
  thanh tab panel **36px** — khớp đúng token mới, **không tràn**. Dải chip nguồn báo "tràn"
  nhưng đó là **dương tính giả theo cấu tạo**: `.lookup-sources-chips` khai
  `max-height: 52px; overflow-y: auto` và chú thích tại chỗ nói *"nơi DUY NHẤT được phép
  tràn, và nó cuộn chứ không nuốt"*. Đo lại số hàng chip hiện được: **2,64 hàng** ở cỡ mới so
  với ~2,76 ở cỡ cũ ⇒ vẫn giữ *"hai hàng trọn + một vệt hàng thứ ba"* làm dấu hiệu còn cuộn
  được. **Không phải chỉnh `max-height`.**

  📌 **Câu hỏi để ngỏ:** một **hệ số scale giao diện** do người dùng chỉnh là câu trả lời đúng
  bản chất hơn — *"nhỏ quá"* là thuộc tính của từng người, không của một con số. Token đang là
  `px` cứng nên nó cần một lượt chuyển sang đơn vị tương đối trước ⇒ **một story riêng**, đã
  ghi vào `DESIGN.md`. **Chủ: Ice.**

- 🔴 **ĐÍNH CHÍNH — bộ e2e CHẬP CHỜN, và bản ghi C3 hôm qua nói "ổn định" trên cỡ mẫu quá nhỏ.**
  Lượt chốt C3 chạy **hai** lượt xanh rồi kết luận ổn định. **Tám** lượt tính tới hôm nay:
  **6 xanh · 2 đỏ**.

  - **Lần đỏ ① — `shortcuts-capture-mouse`: đã chẩn đoán và VÁ.** `cell` lấy **trước**
    `resetRowToDefault()`, mà lượt reset dựng lại hàng ⇒ **tham chiếu chết** ⇒ ca đỏ bằng
    `"element wasn't found"`. Một lỗi **hạ tầng của bàn đo đội lốt hồi quy sản phẩm** — đúng
    lớp lỗi đắt nhất ở một bàn đo. Vá: lấy lại handle **sau** lượt reset.
    ⚠️ Nó chập chờn vì phụ thuộc Vue có thật sự tái tạo node ở lượt đó hay không, nên **bốn
    lượt đầu đi qua sạch**.
  - **Lần đỏ ② — `attribution-focus`: CHƯA chẩn đoán.** Nguyên văn lỗi không kịp bắt *(lượt
    chạy lại đã xanh)*. Nó xanh khi chạy một mình và xanh ở mọi lượt cả-bộ khác. **Còn mở.**

  ⚠️ **Hai lượt xanh sau bản vá KHÔNG chứng minh bộ đã hết chập chờn** — đó đúng là cỡ mẫu đã
  lừa một lần. Luật cho lượt sau, đã ghi vào `wdio.conf.mjs`: gặp một lượt đỏ không tái lập
  được thì **bắt nguyên văn TRƯỚC**, đừng chạy lại cho tới khi xanh rồi đi tiếp. **Chủ: Dev.**

## Deferred from: hai quyết định của Ice về CI và Windows (2026-08-12)

- ✅ **ĐÓNG — CI thôi tự chạy lúc push. Ice chốt: hạn mức tài khoản miễn phí để dành cho một
  dự án khác.**

  🔴 **Và lượt này lộ ra một khoảng cách giữa hồ sơ và thực tế.** Ice chốt *"tạm dừng GitHub
  Actions"* ngày **2026-08-11**, bốn tạo tác đã ghi lại quyết định đó, và lượt correct-course
  còn đính chính từ *"BỎ QUA"* thành *"TẠM DỪNG"* cho đúng ý. **Nhưng khối `on:` của
  `ci.yml` vẫn khai `push:` + `pull_request:`** — nên mọi lượt push từ đó tới nay **vẫn khởi
  động cả hai job và vẫn tiêu phút**. Một quyết định không được cài vào tệp thì không phải
  một quyết định; nó là một ghi chú. Cùng lớp lỗi với *"cổng thứ mười canh máy dev, không
  canh nhánh"* — khoảng cách giữa thứ ta tin và thứ máy làm.

  **Vá:** `on: workflow_dispatch:` — pipeline còn **sống** và tiêu **0 phút** cho tới khi có
  người bấm *"Run workflow"*.

  🔴 **Vì sao KHÔNG xoá tệp, ba lý do và cả ba đều cứng:**
  ① AC4 của Story 1.3 cấm dựng một tệp workflow **thứ hai**; xoá rồi dựng lại sau là đúng
  thứ AC4 chặn. ② Cổng thứ mười một `check:gates` **đọc chính tệp này** (Kiểm A/B) — xoá nó
  làm cổng `abort`, và ta mất phép kiểm buộc **ba** danh sách cổng khai cùng một bộ.
  ③ Bốn phép nghiệm thu runner còn nợ của Story 1.3 (AC6 · AC7 · Task 11 hàng 4 · AC3/Task 4)
  **chỉ đo được ở đây** — chúng chờ, không mất.

  **Nghiệm thu:** `check:gates` vẫn XANH sau lượt đổi ⇒ ba danh sách cổng vẫn khớp, tức
  pipeline vẫn là một tạo tác sống chứ không một tệp chết.

- 📌 **Phương án khôi phục CI miễn phí, để Ice cân khi tới lúc — KHÔNG quyết trong lượt này.**

  | Đường | Cái giá | Ghi chú |
  |---|---|---|
  | **Bấm tay khi cần** *(đang dùng)* | 0 phút cho tới lượt bấm | Đủ cho bốn phép nghiệm thu runner của Story 1.3 — chúng chỉ cần **một** lượt xanh, không cần chạy mỗi push |
  | **Repo công khai** | 0đ, Actions **không giới hạn** cho repo public | 🔴 Đáng cân nhất, vì nó **phục vụ luôn FR107** — *"build công khai để bất kỳ ai kiểm chứng được binary khớp mã nguồn"*, tức đúng thứ Story 10.1 phải làm. Dự án đã là **GPL-3.0-or-later**, nên mã sẽ công khai ở một thời điểm nào đó. Đây là một quyết định của Ice về **thời điểm**, không về nguyên tắc |
  | **Runner tự quản trên máy Ice** | 0 phút hạn mức | Chỉ cho macOS — mà `pre-push` đã canh macOS mỗi lượt rồi ⇒ **giá trị thêm gần bằng 0**. Với Windows thì nó cần đúng cái máy Windows đang chờ |
  | Chạy Actions cục bộ (`act`) | 0đ | Cần Docker, và **không** dựng được job macOS hay Windows ⇒ không trả lời được câu hỏi duy nhất mà CI còn nợ |

  ⇒ Nếu mục tiêu là **nghiệm thu bốn món của Story 1.3**, đường rẻ nhất là **bấm tay đúng
  một lượt** khi Ice sẵn sàng. Nếu mục tiêu là **CI thường trực**, đường duy nhất miễn phí là
  **repo công khai**, và nó trùng với FR107. **Chủ: Ice.**

- 📌 **Trọn phần Windows dời về CUỐI dự án — Ice chốt 2026-08-12, và Ice sẽ tự dựng máy để chạy.**

  Món nợ **A5** vì thế đổi hình dạng: từ *"chờ một điều kiện chưa biết"* thành **một món có
  lịch và có chủ**. Những gì sẽ chờ tới lượt đó, gom lại một chỗ để lượt sau không phải đi
  tìm:
  - **Story 1.3** — AC6 (ba số `.msi` + hai dòng NFR6) · AC7 (thời gian tường, phút tính phí,
    cache lạnh/nóng) · Task 11 hàng 4 (`#[cfg(windows)] compile_error!` làm **chỉ** job Windows
    đỏ) · AC3/Task 4 (rào biên dịch C và WiX v3) · chiều âm của AC8 trên `windows-2025`;
  - **Story 1.7 AC5** — trần WAL nới theo nền tảng, hiệu chuẩn trên **n = 1** điểm đo Windows;
  - **AD-45 và hai móc chuyển hướng** (`$APPDATA`, thư mục gốc Library) — cả ba là mệnh đề
    **hai nền tảng** mới đo được một nửa; đường Windows đi Known Folder API, khác hẳn macOS;
  - **Bốn spec e2e** — chưa từng chạy trên WebView2 một lần nào;
  - **Nợ Windows-only** của 1.6 · 1.14 · 1.15 · 1.16 · 1.17 · 1.18 · 1.19 · 1.20 · 1.21.

  ⚠️ **Hệ quả phải nói thẳng:** mọi thứ Epic 2 → Epic 9 thêm vào sẽ chạy **chỉ trên macOS**
  cho tới lượt đó. Khoảng mù không đứng yên — nó **dày lên theo từng epic**, và lượt Windows
  cuối cùng sẽ phải trả một lần cho tất cả. Đó là cái giá của lựa chọn này, và Ice chọn nó
  với thông tin đó trước mắt. **Chủ: Ice** · mở ở **cuối dự án**.

## Deferred from: 2-1-tach-segment-cap-cau-va-co-ket-doan (2026-08-12)

- 🔴 **AC8 vế MỘT chưa dựng, và đó không phải lệch spec.** AC8 đòi *"tái tách chủ động kèm
  cảnh báo về dữ liệu sẽ về hưu"*. Story 2.1 giao **nửa cưỡng chế được ngay** — đường tự động
  tách lại **không tồn tại**, và `segment_boundary.rs::the_splitter_has_exactly_two_product_call_sites`
  khẳng định điều đó bằng một phép đếm chỗ gọi *(hai, cả hai đều có tên)*. Nửa còn lại — nút
  tái tách kèm cảnh báo — cần ngữ nghĩa **về hưu** của AD-5 *(segment cũ thành tombstone,
  lịch sử vẫn tra được, segment mới bắt đầu với lịch sử rỗng)*, mà hôm nay **chưa có
  `SegmentVersion`** để mà giữ lại. Cột `retired_at` đã có sẵn trong `SEGMENT_DDL` để 2.8
  không phải mở một bước di trú thứ hai chỉ để thêm một cột. **Chủ: Story 2.8.**

- ⚠️ **Tỷ lệ ranh giới sai của giả định A4 đo được `0,47%` — nhưng trên một mẫu văn xuôi
  MỎNG.** Đo 2026-08-12 trên 21 Chương thật của Epic 1 (`127.940` ký tự ⇒ `10.477` segment),
  rà tay **211** ranh giới: **1 sai**. Con số đẹp, và nó **đọc quá tốt so với thứ nó chứng
  minh được** — vì bộ dữ liệu Epic 1 gần như không có văn xuôi thật:
  - `17.zh` (72.862 ký tự, **94%** tổng số segment) là một **bảng dữ liệu TSV** Hán Việt, một
    dòng một mục. Mọi ranh giới của nó là một xuống dòng — nó không kiểm một luật tách câu nào.
  - `01.en` (48.640 ký tự) là một tài liệu **Markdown** tiếng Việt khai `source_lang = "en"`.
  - Mẫu văn xuôi tiếng Trung **thật** lớn nhất là `12.zh`: **351 ký tự**, 7 segment.
  ⇒ A4 (*"tách câu tự động đúng ở tỷ lệ chấp nhận được"*) **chưa được kiểm trên một chương
  tiểu thuyết tiếng Trung thật**, và đó chính là ca sản phẩm chính. **Chủ: Ice** — một lượt
  nhập một chương truyện thật rồi rà tay, trước khi Epic 2 đi xa hơn. AD-4 đóng băng ranh
  giới **vĩnh viễn**, nên phép đo này rẻ nhất khi làm sớm.

- ⚠️ **Ca sai duy nhất còn lại: một hàng bảng Markdown bị cắt giữa ô.** Một ô chứa hai câu
  (`| 2\. CHIẾN LƯỢC … phục vụ ai. Chiến lược là sự tập trung… |`) bị cắt tại dấu chấm giữa
  ô, cho hai segment mà segment sau mang một `|` mồ côi ở cuối. Đây là cấu trúc **bảng**, và
  bộ tách cấp câu không biết bảng — biết bảng là việc của đường nhập (FR124/FR125, luật làm
  sạch của Story 6.5). **Không vá ở đây**, và cố ý không: một luật *"đừng cắt trong một hàng
  bảng"* nhét vào `core/segment/split.rs` là đưa kiến thức Markdown xuống một tầng không biết
  định dạng nào. **Chủ: Story 6.5** (luật làm sạch) hoặc **6.12** (đọc `.docx` có bảng).

- 🔴 **Luật thứ NĂM của bộ tách do một phép đo dựng ra, ngoài bốn luật của Quyết định #5 —
  Ice có thể lật.** *"Một câu phải có ít nhất một chữ"*: một dấu kết câu không chốt ranh giới
  nếu phần văn bản trước nó không chứa ký tự `char::is_alphabetic` nào. Số dẫn tới nó: mục lục
  đánh số của `01.en` (`* 0\. Triết Lý Nền Tảng…` trên **một** dòng) bị cắt ngay tại dấu chấm
  của mốc danh sách — **26 ranh giới sai trên 99** trong 100 segment đầu, **tất cả cùng một
  nguyên nhân**. Sau luật: **0/99**; tổng segment toàn bộ 21 Chương `10.556 → 10.477`.
  Luật áp cho ranh giới **dấu kết câu**, KHÔNG cho ranh giới **xuống dòng** *(một dòng không
  có chữ — `---` của Markdown, một hàng số — vẫn là một segment riêng)*. Nó **không** phải một
  luật Markdown và cố ý không phải: mệnh đề thuần về kiểu chữ, không biết định dạng nào.
  ⚠️ Ghi ra ở đây vì nó là một luật **story không đặt hàng**, và tỷ lệ 12,8% → 0,47% là toàn bộ
  lý lẽ của nó. Nếu Ice thấy nó quá rộng, chỗ lật là một dòng ở `split_source_text`. **Chủ: Ice.**

- ⚠️ **`work.source_lang` trong dữ liệu thật đang SAI ở ít nhất 3/21 Chương, và nó đổi kết quả
  tách.** Đo 2026-08-12: `Truyện Kiều.atproj` và `Thieu Chuu 3.atproj` khai `zh` nhưng chứa
  **tiếng Việt**; `Russia is considering using.atproj` khai `zh` nhưng chứa **tiếng Anh** *(cùng
  nội dung với `Russia is considering.atproj` khai `en` — chúng cho **3** và **4** segment)*.
  Bộ tách chọn nhánh theo `source_lang` chứ **không** đoán từ nội dung, đúng AD-18 và đúng
  `segment_contract.rs::the_language_branch_comes_from_source_lang_not_from_the_content` — nên
  hệ quả là một Chương khai sai ngôn ngữ được tách theo luật của ngôn ngữ **khác**, và với văn
  bản tiếng Việt khai `zh` thì **cả Chương thành một segment duy nhất**. AD-18 nói `source_lang`
  là trường **bất biến**, nên đây không sửa được bằng một lượt sửa nhãn. Đây là dữ liệu thử
  nghiệm của Epic 1 nên rủi ro thấp — nhưng nó là bằng chứng rằng **màn hình tạo Tác phẩm để
  người dùng chọn sai ngôn ngữ quá dễ**. **Chủ: Ice** — quyết định xem một lượt xác nhận ngôn
  ngữ lúc nhập (đối chiếu nội dung với nhãn, cảnh báo chứ không tự đổi) có đáng một story không.

## Deferred from: code review of 2-1-tach-segment-cap-cau-va-co-ket-doan (2026-08-12)

- ⚠️ **`insert_segments` chuẩn bị lại statement SQL cho mỗi hàng segment.**
  `src-tauri/src/commands/segment.rs:74-88` gọi `tx.execute` với một chuỗi SQL literal bên
  trong vòng lặp, nên `rusqlite` parse lại câu lệnh **mỗi hàng** thay vì chuẩn bị một lần rồi
  tái dùng (`prepare`/`prepare_cached`). Toàn bộ N lượt đó chạy trong **một** closure của
  `Store::write`, tức trên writer **duy nhất, nối tiếp** của AD-11 — cùng điểm nghẽn mà
  `commands/project.rs:120-127` đã kéo `split_source_text` ra ngoài để né. Quy mô thật đo được
  ở Task 8: một Chương chiếm 94% của 10.477 segment ⇒ ~9.850 lượt parse trong một giao dịch.
  🔴 **Hoãn vì chưa ai đo, không phải vì nó nhỏ.** Story 2.1 đặt chuẩn *"đo chứ không ước"*
  (AC15), và đề xuất một tối ưu chưa có số là tự phá chuẩn đó — SQLite parse rất nhanh và chi
  phí thật có thể nằm dưới ngưỡng đáng sửa. Việc cần làm là **một phép đo** trên Chương lớn
  nhất có thật, rồi mới quyết vá hay đóng.
  **Chủ: Story 2.2** *(story đầu tiên tải segment lên giao diện, tức chỗ đầu tiên chi phí này
  chạm một thao tác người dùng nhìn thấy)*.
  → ✅ **ĐÓNG 2026-08-12 (Story 2.2 · AC17 · Task 8).** Đã đo, rồi mới vá. `cargo test --release`
  trên macOS, **9.850 hàng** — quy mô thật của Chương lớn nhất, ba lượt: `tx.execute` literal mỗi
  hàng cho **105,51 / 106,90 / 112,47 ms**; `prepare_cached` một lần cho **44,76 / 49,75 / 48,28 ms**.
  Chênh **57–64 ms** (53,5–57,6 %). Vá vì con số chứ không vì linh cảm: khoản tiết kiệm nằm **trên**
  trần một frame của NFR2 (50 ms) chỉ bằng một mình nó, và nó nằm trong closure của `Store::write`,
  tức trên writer duy nhất nối tiếp của AD-11. Dùng `prepare_cached` (không `prepare`) để Chương
  **thứ hai** trở đi không phải parse lại lần nào — bộ nhớ đệm sống trên kết nối ghi dài hạn.
  Bảng số đầy đủ ở doc-comment của `insert_segments`.

## Deferred from: 2-2-panel-editor-lien-mach (2026-08-12)

- 🔴 **Ba trong năm giá trị vạch lề KHÔNG có nguồn dữ liệu, và mỗi giá trị có chủ riêng.**
  `src/panels/editorSegments.ts::resolveSegmentRule` cài **cả năm** nhánh ở một hàm duy nhất
  (Quyết định #4(b) — bảng ánh xạ *trạng thái → vạch* là một **hợp đồng**, và một hợp đồng cài
  nửa vời là chỗ để story sau chép sai). Ba nhánh đọc từ hai trường mà hôm nay **không đường nào
  bật lên được**: `isConfirmed` ← cột `segment.status` chưa tồn tại (**Chủ: Story 2.5**);
  `isTmFilled` ← chưa tầng TM nào, FR58 (**Chủ: Epic 7**); `retiredAt` ← cột **đã có** từ Story
  2.1 nhưng chưa đường nào cho segment về hưu (**Chủ: Story 2.8**). Mỗi story chủ chỉ phải nối
  nguồn — **không** phải sửa tầng hiển thị. Ba nhánh nghiệm thu được ở bàn đo
  (`2-2-ban-do-editor.html`) và ở `segment_contract.rs::a_chapter_with_real_translations_round_trips_through_the_load_command`.
  → 🟡 **ĐÓNG MỘT PHẦN 2026-08-14 (Story 2.5).** Nhánh `isConfirmed` **đã có nguồn thật**: cột
  `segment.status` tới bằng bước di trú **7**, và `segmentRuleInputOf` đọc nó thay cho hằng
  `false` — đúng **một** trong hai dòng mà doc-comment tại chỗ đã hẹn. Lưới:
  `tests/frontend/editorSegmentRule.test.ts`.
  **PHẦN CÒN HỞ, và mỗi phần giữ nguyên chủ cũ:** `isTmFilled` ← FR58, **Chủ: Epic 7** *(hằng
  `false` ở `editorSegments.ts` **ở lại**)*; `retiredAt` ← chưa đường sản phẩm nào cho segment về
  hưu, **Chủ: Story 2.8** *(Story 2.5 chỉ dựng một **hàng rào từ chối** —
  `MessageKey::SegmentRetired` — và test của nó dựng trạng thái về hưu bằng SQL trực tiếp)*.

- 🔴 **HAI CÂU CÙNG MỘT DÒNG CHO HAI VẠCH LỀ CHỒNG LÊN NHAU — phát hiện của bàn đo, chưa vá.**
  Vạch được đặt `position: absolute; left: 8px` trong máng, chiều cao đo từ `getClientRects()`
  của chính câu. Văn bản chảy **inline** (AC1 cấm chia khối), nên hai câu ngắn nằm cùng một dòng
  cho hai vạch **cùng `top`, cùng `left`** — vạch vẽ sau che vạch vẽ trước. Đo được ở bàn đo,
  cả Blink lẫn WebKit: fixture 5 câu vẽ **4** vạch nhưng chỉ nhìn thấy **2** vị trí (câu 1
  `confirmed` bị câu 2 `primary` che; câu 3 `tm-rule` bị câu 5 `ornament` che).
  ⚠️ **Hôm nay KHÔNG chạm tới được trong sản phẩm**: chỉ `primary` có nguồn dữ liệu, và caret chỉ
  có **một**, nên nhiều nhất một vạch tồn tại cùng lúc. Nó thành thật ngay lượt Story 2.5 nối
  `segment.status`. `DESIGN.md:380` và `EXPERIENCE.md:105-113` **không** phân xử ca này — máng
  rộng 22px, vạch thụt 8px, còn 12px trống, nên xếp cạnh nhau là **khả thi** nhưng là một quyết
  định thiết kế, không phải một bản vá kỹ thuật.
  **Chủ: Story 2.5** *(story đầu tiên làm hai vạch cùng tồn tại)* — **và một lượt ký của Ice** cho
  hình dạng lời giải.
  → ✅ **ĐÃ ĐÓNG 2026-08-14 (Story 2.5, Quyết định #2 đường (a) — Ice ký).** Cách đóng: `left` rời
  khỏi CSS và nay do `editorGutter.ts::assignGutterLanes` tính, đi qua `:style` cùng đường với
  `top`/`height` *(hình học bind bằng style, màu bind bằng lớp — điều kiện để Kiểm B của
  `check-tokens.mjs` còn đọc được bốn màu vạch từ CSS)*.
  **Phép phát làn là TÔ MÀU ĐỒ THỊ KHOẢNG**, không gom bắc cầu: mỗi vạch nhận làn nhỏ nhất chưa bị
  một vạch chồng nào chiếm. **Bước làn CO cho vừa máng** — `bước = clamp(2, 5, ⌊12/(N-1)⌋)`.
  ⚠️ **Bản đầu bị PHÉP ĐO bác, ghi lại thay vì sửa im lặng:** bước cố định 5px *(đúng hình dạng
  §Quyết định #2 của story mô tả — "làn trong 8px, làn ngoài 13px")* chỉ chứa nổi **3** làn, và
  fixture **đối thoại** của bàn đo đòi **5** ⇒ làn ngoài ở `left: 28px`, mép phải **30px**, tràn
  khỏi máng 22px đúng chỗ chữ bắt đầu. Với bước co thì 5 làn nằm gọn ở mép phải **22px**, **0 vạch
  bị che**. Số đo hai engine × hai theme: `2-5-ban-do/README.md`; ảnh: `2-5-ban-do/*.png`.
  Lưới: `tests/frontend/editorGutterLanes.test.ts` *(hai đường sai — bước cố định, gom bắc cầu —
  đều đã chạy đỏ-rồi-xanh 2026-08-14)*.

- ⚠️ **GIỚI HẠN CÒN LẠI của lời giải vừa ký: từ 8 LÀN trở lên máng 22px hết chỗ.** Bước tối thiểu
  là 2px *(bằng bề rộng vạch)*, nên máng chứa nhiều nhất **7** làn; từ làn thứ tám luật là **dồn
  về làn cuối**, tức chấp nhận che — có chủ ý, thay vì tràn ra đè lên chữ. Nó đòi **8 câu cùng một
  dòng**; fixture đối thoại dày nhất của bàn đo mới cho **5**. Lời giải nếu ngày đó tới: nới token
  `gutter-width` *(một lượt sửa `DESIGN.md`, tầng token)*. **Chủ: Ice.**

  → ✅ **ĐÃ ĐÓNG 2026-08-15 (Story 2.5b) — BIẾN MẤT THEO CẤU TRÚC, KHÔNG ĐƯỢC VÁ.** Ghi rõ cách
  đóng vì hai cách đó **không** đổi cho nhau được: không ai nới `gutter-width`, không ai sửa
  thuật toán chia làn. **Khái niệm "làn" thôi tồn tại**: lưới cho **một câu một HÀNG**, nên hai
  vạch không bao giờ còn trùng `top`, nên không còn gì để xếp làn. Vạch nay lấy chiều cao từ
  **track hàng** của `subgrid` *(`GridPanel.vue::.rule { height: 100% }`)*, và cột vạch rộng
  **3 px** — token `gutter-width` **không còn người đọc** ở bề mặt này.
  ⚠️ Điều kiện để mục này ở lại đóng: **một câu vẫn là một hàng**. Ngày nào một story cho hai
  câu chung một hàng *(gộp hiển thị, xuống dòng mềm…)*, bài toán quay lại **nguyên vẹn** — và
  phép đo của `assignGutterLanes` vẫn nằm trong sổ, ngay dưới đây.

- ⚠️ **Bảng năm giá trị vạch có một HÀNG CÒN THIẾU: "đã dịch bằng tay, chưa xác nhận, con trỏ ở
  chỗ khác".** `confirmed` sai *(chưa ai ký)*, `tm-rule` sai *(không phải máy điền)*, *không vạch*
  sai *(nó đã có bản dịch)*. `EXPERIENCE.md:105-113` đơn giản không có hàng đó. Hôm nay khe hở
  **không chạm tới được** — `target_text` chỉ nhận giá trị qua đường gõ, mà đường gõ là Story 2.3.
  Nhánh hiện rơi về *không vạch* và ghi lại điều đó tại chỗ (doc-comment `resolveSegmentRule`).
  **Chủ: Story 2.5** *(nó mang `segment.status`, tức chỗ duy nhất phân xử được)* — và một lượt ký
  của Ice nếu lời giải là sửa `EXPERIENCE.md`.
  → ✅ **ĐÃ ĐÓNG 2026-08-14 (Story 2.5, Quyết định #3 đường (a) — Ice ký).** Cách đóng: giữ *không
  vạch*, và **viết mệnh đề đó ra** thay vì để nhánh rơi vào đó không lời giải thích — *"vạch chỉ
  nói **ai đã ký**, không nói **có chữ hay chưa**"*, khớp `DESIGN.md:380` nơi vạch lề được định
  nghĩa là chỗ đọc **trạng thái xác nhận**. Cái mất *(không phân biệt được **chưa dịch** với **đã
  dịch chưa ký** bằng vạch)* chấp nhận được vì đã có kênh khác chở: **văn bản có chữ**, nằm ngay
  cạnh. Hai đường kia bị loại — mượn `tm-rule` phá nghĩa cố định của nó *(và làm hỏng cả FR81 lẫn
  ranh giới bóc)*; một giá trị **thứ sáu** phá `EXPERIENCE.md:99` và làm Kiểm I đỏ.
  Doc-comment `editorSegments.ts::resolveSegmentRule` mang mệnh đề đã ký; lưới:
  `tests/frontend/editorSegmentRule.test.ts`.
  ⚠️ **Món con còn hở, có chủ:** `EXPERIENCE.md:105-113` **chưa** được sửa cho có hàng đó — sửa một
  tài liệu tầng nguyên tắc là **một lượt riêng của Ice**, dev không sửa tài liệu quy hoạch.
  **Chủ: Ice.**

- 🔴 **NFR2: dựng 9.850 `<span>` câu vượt trần 50 ms/frame — 6× trên Blink, 26× trên WebKit.**
  Đo ở bàn đo, 2026-08-12, Chương lớn nhất có thật (9.850 câu):

  | | dựng DOM + bố cục | đo + vẽ **1** vạch (ca THẬT hôm nay) | đo + vẽ **9.850** vạch (ca trần) |
  |---|---|---|---|
  | Blink (HeadlessChrome 151) | **300,1 ms** | 8,5 ms | 63,1 ms |
  | WebKit (605.1.15 / Safari 26) | **1.308,0 ms** | 5,0 ms | 64,0 ms |

  ⇒ **Cơ chế đo của Quyết định #2 KHÔNG phải chỗ đắt** — nó tốn 5–9 ms ở ca thật và 63–64 ms ở ca
  trần. Chỗ đắt là **dựng 9.850 phần tử DOM**, tức đúng hàng Deferred *"ảo hoá danh sách dài"*
  (`ARCHITECTURE-SPINE.md:888`, Giai đoạn 3). AC14 nói thẳng: *"Nếu vượt, đó là số của Story 2.4,
  ghi lại và báo, đừng tối ưu mù"* — nên story này **không** dựng ảo hoá.
  ⚠️ Lượt dựng là **một lần mỗi Chương**, không phải đường nóng NFR1 — cùng hạng với trần render
  của kiểu song song ở Story 1.16, nơi Ice đã chốt 1,4 s là *"còn chấp nhận được"* cho một thao
  tác chạy một lần. 1,3 s ở đây nằm ngay dưới mốc đó, nhưng nó là số của **hôm nay**, khi mọi
  `target_text` còn RỖNG; chữ thật sẽ làm nó tăng.
  **Chủ: Story 2.4** *(story mang AC ghi lại lựa chọn thư viện editor — `epics.md:2142-2145`)*.

- ⚠️ **Bàn đo của story này là một tệp DÙNG MỘT LẦN, và nó CHÉP chứ không mount.**
  `_bmad-output/implementation-artifacts/2-2-ban-do-editor.html` chép CSS và cấu trúc DOM của
  `src/panels/EditorPanel.vue` cộng bản chép JS của `editorSegments.ts`/`editorGutter.ts` —
  component thật cần cầu IPC của Tauri. ⇒ một lượt sửa template/CSS sau này có thể làm bàn đo và
  sản phẩm **lệch nhau mà không cổng nào đỏ**. Cộng hai giới hạn nữa: **ba font nhúng của UX-DR4
  vắng mặt** *(bàn đo rơi về `serif` hệ thống, nên số chiều cao vạch là số của **cơ chế**, không
  phải của **sản phẩm**)*, và **`⏐` là pseudo-element nên nó không hiện trong một bàn đo chép DOM**.
  Cùng lớp nợ `deferred-work.md:826`. **Chủ: treo cho tới khi có quyết định về một bộ chạy test
  frontend (NFR15).**

- 🔴 **WKWebView THẬT (trong cửa sổ Tauri) vẫn CHƯA ĐO — bàn đo chạy WebKit của Playwright.**
  Đây là lượt đầu tiên của dự án có bằng chứng **WebKit** cho một bề mặt DOM *(mọi story trước đo
  trên Blink — `deferred-work.md:145`)*, và nó trả lời được câu hỏi nóng nhất: hình học
  `getClientRects()` với chữ dày dấu tiếng Việt **khớp giữa hai engine** *(46,25 px Blink vs
  46,00 px WebKit; 64,50 vs 64,00 — lệch dưới một pixel)*, và `innerText` **không** rò ký tự `⏐`
  trên **cả hai** *(tức Quyết định #3 — pseudo-element thay vì `<span>` thật — đã đóng vết sẹo
  `WORD_JOINER` của Story 1.18b trên đúng engine đã sinh ra nó)*.
  ⚠️ Nhưng WebKit-của-Playwright **không phải** WKWebView-của-Tauri: khác phiên bản, khác lượt
  nhúng font, khác tầng phân phối sự kiện của OS. Đừng viết *"tương đương"*.
  **Nhặt lại:** một lượt `npm run tauri dev`, hoặc một spec e2e WebdriverIO khi bộ đó hết chập chờn.
  **Chủ: nợ chung "hai nền tảng" của 1.6/1.14/1.16/1.17/1.18/1.18b — nay thêm 2.2.**

- ⚠️ **`data-caret` đọc từ NEO VÙNG CHỌN DOM, không từ một caret thật — một luật ngoài bảy AC.**
  AC5 đòi *"tiêu điểm bàn phím chạm tới một câu"*, nhưng một bề mặt không `contenteditable`
  **không có caret** (AC18 cấm gõ ở lượt này). Lời giải: `Selection.anchorNode`, cộng
  `tabindex="0"` trên `.doc` — **đúng cơ chế** mà Story 1.18 đã dựng cho Panel Source để đóng
  `deferred-work.md:608`, và một cú bấm chuột cũng đặt một vùng chọn thu gọn.
  🔴 Cái giá phải nói ra: `tabindex="0"` **bên trong** một `PanelFrame` mang `tabindex="-1"` làm
  phím `Tab` nay dừng ở thân Panel Editor — đúng hệ quả mà Story 1.18 đã ghi và Ice đã ký cho
  Panel Source ngày 2026-08-07. Story này áp cùng đánh đổi cho panel thứ hai **mà chưa có một
  lượt ký riêng**.
  → ✅ **ICE ĐÃ KÝ 2026-08-12** *(lượt code review của Story 2.2)*. Tiền lệ Panel Source mở rộng
  sang panel **thứ hai**: `tabindex="0"` trên `.doc` giữ nguyên, và cái giá — `Tab` dừng ở thân
  Panel Editor — được chấp nhận có chủ ý. Lý lẽ đã cân trước khi ký: gỡ nó thì vế **bàn phím** của
  AC5 mất nguồn dữ liệu *(`Shift+Mũi tên` không tới, `data-caret` không bao giờ bật)* và hợp đồng
  vùng chọn đăng ký từ Story 1.18 nằm chết trên bề mặt này — tức AC5 sẽ **không giao đủ**.
  ⚠️ Chữ ký này phủ **cơ chế hôm nay**, không phủ Story 2.3: khi caret thật xuất hiện cùng
  `contenteditable`, **Story 2.3** vẫn phải xét lại toàn bộ đường `Selection.anchorNode` này.

- ⚠️ **Một luật hiển thị ngoài bảy AC: dòng *"Chương này đã tách thành câu, chưa câu nào có bản
  dịch"*.** Bảy AC không nói gì về ca *"đã tách, chưa câu nào có bản dịch"* — trước Quyết định #1
  nó không tồn tại; sau phán quyết đường (b) nó là trạng thái **thường trực** của mọi Tác phẩm cho
  tới Story 2.3. UX-DR27 nói cái giá của việc im: *"một khung trống câm là thứ người dùng đọc thành
  hỏng"*, và `SourcePanel.vue::panel.source.empty_chapter` đã đặt tiền lệ đúng ca này. Dòng hiện
  **phía trên** trang văn, không thay nó. **Chủ: Ice** — chỗ lật là một `v-if` trong
  `EditorPanel.vue`, và khoá `panel.editor.nothing_translated` trong `vi.json`.
  → ✅ **ĐÃ ĐÓNG 2026-08-15 (Story 2.5b, chốt ở lượt code review).** Đóng **theo cấu trúc**, không
  bằng một câu: lưới hai cột cho **mỗi hàng chưa dịch** một nhãn `panel.grid.state_untranslated`
  *("chưa dịch")* ở cột ⑤, nên ca *"đã tách, chưa câu nào có bản dịch"* hiện ra thành **N dòng nói
  rõ**, không một khung trống. Cái mà UX-DR27 cấm — *"một khung trống câm"* — **thôi dựng được**
  trong hình dạng lưới, chứ không phải bị một câu che đi.
  ⚠️ **Hai con trỏ trong mục này đã chết từ 2026-08-15 và cố ý GIỮ NGUYÊN làm lịch sử:**
  ~~`EditorPanel.vue`~~ *(xoá trọn ở Story 2.5b)* và ~~`panel.editor.nothing_translated`~~ *(gỡ
  khỏi `vi.json` cùng lượt)*. Lượt rà bắt được mục này đang trỏ vào cả hai — một món nợ trỏ vào
  hư không thì không ai lần lại được, và đó là lý do dòng đóng này phải nói ra chỗ nó đã trỏ.
  ⚠️ **Vế còn hở, ghi ra thay vì làm tròn lên:** ca *"đã tách, chưa câu nào có bản dịch"* nay đọc
  được **theo từng hàng**, chưa có một câu tổng ở tầng panel. Nếu một Chương 9.850 câu làm người
  dùng phải cuộn mới thấy tình trạng chung thì đó là một mệnh đề MỚI, cần một phép đo, và nó
  **không** thuộc món nợ này — mở một mục mới, đừng mở lại mục đã đóng.

- ⚠️ **Cổng Kiểm J của `check-commands.mjs` HẾT HẠN ở Story 2.3, và nó phải được gỡ ĐÚNG LÚC.**
  Cổng khẳng định `EditorPanel.vue` không mang `contenteditable`/`<textarea>`/`<input>`/`v-model`/
  handler sửa văn bản (AC18 — hệ quả phán quyết Quyết định #1). Story 2.3 dựng vùng gõ, nên nó
  phải gỡ cổng **cùng lượt** với hợp đồng flush AD-35 — **không sớm hơn**. Gỡ sớm là mở lại đúng
  cửa sổ mất dữ liệu im lặng mà cổng tồn tại để đóng. **Chủ: Story 2.3.**
  → ✅ **ĐÓNG 2026-08-12 (Story 2.3).** Gỡ **cả khối** — `TYPING_BANS`, sàn nội dung, tiêu đề in
  ra. Vế *"đúng lúc, không sớm hơn"* có bằng chứng: 8 ca mới ở `segment_contract.rs` xanh **trước**
  khi `contenteditable` chạm `EditorPanel.vue`. Chi tiết ở §Deferred from: 2-3-hop-dong-flush.
  ⚠️ Giới hạn đã ghi tại chỗ: cổng đọc bản **đã che** (bỏ chú thích và chuỗi), nên một
  `el.setAttribute('contenteditable', 'true')` trong một chuỗi JavaScript đi lọt.

- ⚠️ **Nhãn `Covers:` của Story 2.2 trong `epics.md:2036` TRỎ SAI NGUỒN.** Nó ghi
  `**Covers:** UX-DR13 · AD-1`, nhưng UX-DR13 (`epics.md:523`) là *"Workspace là lưới 2×2 mặc
  định"* — một quyết định **bố cục lưới** mà Story 1.14 đã dựng xong, và không một chữ nào của nó
  nói về nội thất Panel Editor. Đặc tả thật của bảy AC là **UX-DR19** (AC1·AC2·AC3) · **UX-DR20**
  (AC4·AC5) · **UX-DR2 + UX-DR12** (AC6) · **UX-DR7 + AD-34 §2** (AC7). Dev **không** sửa
  `epics.md` — tiền lệ quyết định #3 của Ice ở Story 1.3, giữ qua toàn Epic 1. **Chủ: Ice.**

## Deferred from: code review of 2-2-panel-editor-lien-mach (2026-08-12)

- ⚠️ **`:data-caret` buộc Vue dựng lại TOÀN BỘ trang văn mỗi lượt `selectionchange`**
  (`src/panels/EditorPanel.vue:286`). `editorCaretSegmentId` là một ref phản ứng được đọc trong
  hàm render, nên mỗi lượt đổi caret — và `selectionchange` bắn **liên tục** trong lúc kéo chọn —
  chạy lại `v-for` trên tới **9.850** `<span>`. ⚠️ Đổi sang tra `Map` như `ruleClassById`
  **KHÔNG** vá được: chi phí nằm ở lượt duyệt danh sách của Vue, không ở phép so sánh. Lời giải
  thật là **ảo hoá danh sách dài** — hàng Deferred Giai đoạn 3 (`ARCHITECTURE-SPINE.md:888`).
  Story 2.2 **đã đo và đã báo** đúng trần này (dựng DOM + bố cục: **300,1 ms** Blink ·
  **1.308,0 ms** WebKit cho 9.850 câu, so với trần 50 ms/frame của NFR2) và giao số cho
  **Story 2.4**. Ghi ở đây để lượt tối ưu của 2.4 biết rằng caret là **nguồn kích hoạt thứ hai**
  của cùng chi phí đó, không chỉ lượt dựng đầu tiên. **Chủ: Story 2.4.**

## Deferred from: 2-3-hop-dong-flush-va-trang-thai-da-luu (2026-08-12)

### 🔴 ĐÃ ĐÓNG ở story này — hai món có chủ đích danh

- ✅ **ĐÓNG — cổng Kiểm J của `check-commands.mjs`** *(mục `:2135-2139` ở trên)*. Gỡ **cả khối**:
  bảng `TYPING_BANS`, sàn nội dung `data-segment-id`, và tiêu đề in ra — không để lại một cổng
  xanh rỗng. **Bằng chứng thứ tự làm việc** *(vế "đúng lúc, không sớm hơn" của món nợ)*: đường
  flush của AD-35 nghiệm thu xanh ở `src-tauri/tests/segment_contract.rs` — 8 ca mới, gồm lượt
  round-trip *gõ → flush → nạp lại* và ca *lô mang một id lạ bị từ chối TRỌN* — **trước** khi dòng
  `contenteditable` đầu tiên chạm `EditorPanel.vue`. Sàn nội dung không mồ côi: Kiểm I vẫn đọc
  `editorVue.masked` và vẫn đối chiếu năm giá trị vạch hai chiều.

- ✅ **ĐÓNG MỘT NỬA — `isTypingZone`, chiều thật sự chạm tới** *(mục `:180-182` ở trên)*. Vùng gõ
  Editor được nhận **đúng**, và nó có lưới: Kiểm D của `check-commands.mjs` nay lái nhánh
  `isContentEditable === true` bằng `{ tagName: 'SPAN', isContentEditable: true }` — nhánh đó có
  trong mã từ Story 1.6 mà **chưa phép kiểm nào đi qua**, vì tới hết 2.2 kho không có một
  `contenteditable` nào. Nghiệm thu **đỏ-rồi-xanh**: gỡ nhánh `isContentEditable` ⇒ ca đỏ với
  *"nhận `true`, phải là `false`"*; trả lại ⇒ xanh. Cộng một đối chứng âm *(một `<span>` KHÔNG gõ
  được thì hợp âm trần VẪN khớp)*, không có nó thì một `isTypingZone` luôn trả `true` vẫn đi qua.
  🔵 **Và một lo ngại đã bị LOẠI bằng phép đo, không bằng một bản vá:** `isContentEditable` đọc ra
  `true` cho **cả** `"true"` lẫn `"plaintext-only"` trên **cả hai** engine ⇒ `isTypingZone` không
  cần sửa một dòng. Một lượt đo trung gian từng đọc ra `false` trên WebKit và đó là số của một
  **cây DOM đã tháo** *(`page.setContent` không thực thi lại `<script>` nội tuyến)* — nếu tin nó,
  story đã đi vá một khuyết tật không tồn tại.

- ⚠️ **CHIỀU CÒN LẠI của `isTypingZone` KHÔNG đóng, và lý do là một phép đo chứ không một lượt
  hoãn.** Hai chiều còn hở — **mù shadow DOM** *(`composedPath()[0]` không bao giờ được hỏi)* và
  **chặn nhầm input phi văn bản** *(`checkbox`/`radio`/`button`/`range`, input `disabled`/`readonly`)*
  — vẫn **không có chỗ gọi nào đi qua**: kho hôm nay có **0** shadow DOM và **0** input phi văn
  bản *(ô phím của `ShortcutsOverlay.vue:159` là một `<button>` có chủ ý)*. Viết mã cho một nhánh
  không chỗ gọi nào đi qua là đúng thứ danh mục `MessageKey` của `core::i18n` đã cấm bằng chữ, và
  đúng thứ Story 1.7 §Completion Notes #3 từ chối. **Chủ: story nào dựng điều khiển form thật
  HOẶC một custom element** — ứng viên gần nhất là Epic 3 *(bảng glossary)* và Story 4.2 *(cấu
  hình nhà cung cấp AI, có ô nhập thật)*.

### 🔴 MÓN MỚI — và món thứ nhất là một KHUYẾT TẬT SẢN PHẨM đã đo, chưa vá

### 🔵 ĐÍNH CHÍNH 2026-08-13 — CHẨN ĐOÁN "AD-34 GIÀNH TIÊU ĐIỂM" LÀ **SAI**

Ice ký đường ① *(cho `section.mode`/`section.panel` thôi giành tiêu điểm)* ngày 2026-08-13. **Bản vá đó KHÔNG được thi hành, vì phép đo tiếp theo cho thấy nó chữa một nguyên nhân không tồn tại** — và ghi lại ở đây thay vì im lặng bỏ qua một lượt ký.

**Không ai giành tiêu điểm cả.** Đo thẳng vào từng ứng viên:

- `focus.ts::enter()` chỉ chạy lúc **đổi chế độ** *(`WorkspaceMode.vue::onMounted`, `modeState.ts`)*, không chạy ở một cú bấm; **(Chủ: story kế tiếp chạm `focus.ts`.)**
- chốt chống-rơi-về-`body` của nó **chỉ `console.error`**, doc-comment của chính nó ghi *"để KÊU, không để VÁ"*; **(Chủ: story kế tiếp chạm `focus.ts`.)**
- `PanelFrame.vue` chỉ **nghe** `focusin`/`focusout`, không gọi `focus()` một lần nào.

Thứ đặt tiêu điểm lên `section.panel` là **hành vi mặc định của trình duyệt**: khi chỗ bấm không soạn thảo được, engine đi ngược cây tìm tổ tiên **focus được gần nhất** — đúng gốc `tabindex="-1"` mà `PanelFrame` dựng cho AD-34 §2. Nó chọn như vậy chỉ vì **`<span>` chưa `contenteditable` tại thời điểm engine ra quyết định**: `onDocMouseDown` chỉ gọi `setEditorCaret()`, và Vue vá DOM ở một **microtask sau**.

⇒ **Nguyên nhân thật: một lượt đặt thuộc tính BẤT ĐỒNG BỘ.** Bản vá: `sent.setAttribute('contenteditable', 'true')` **đồng bộ ngay trong `mousedown`**, trước khi handler trả về. Vue render cùng giá trị ngay sau ⇒ hai đường hội tụ, không tranh nhau.

**Đo lại sau bản vá, WKWebView thật, chuột thật, trên một câu ĐÃ CÓ CHỮ:**

| | Trước | Sau |
|---|---|---|
| `contenteditable` lúc engine xử lý cú bấm | **vắng** | **`"true"`** |
| `caretRangeFromPoint` | phân giải **ngoài** câu | **`#text@18`**, `pointInsideSent: true` |
| `getSelection().type` | `"None"` | **`"Caret"`** |
| `document.activeElement` | `SECTION.panel` / `SECTION.mode` | **`SPAN.sent`** |

🔴 **AD-34 KHÔNG cần đổi một dòng, và `focus.ts` không bị chạm.** Đây là một ca mẫu của bài học *"kiểm điều kiện đo trước khi lật một quyết định"*: một số đọc đúng (`activeElement = SECTION.panel`) dẫn tới một chẩn đoán sai, và chẩn đoán đó suýt lật một hợp đồng đang hoàn toàn đúng.

### 🔴 CÒN LẠI SAU ĐÍNH CHÍNH — Ice gõ tay 2026-08-13, ba triệu chứng, ba nguyên nhân khác nhau

Ảnh chụp của Ice cũng cho một **bằng chứng dương** quan trọng: thanh trạng thái ghi *"Đã lưu 4 giây trước"* và chữ đã vào `project.db` ⇒ **đường flush AD-35 chạy thật với người dùng thật**.

**① ĐÃ VÁ — gõ ngược từ phải sang trái.** Nguyên nhân là mã của tôi, và nó lật một câu tôi từng
viết trong chính `EditorPanel.vue`: mỗi phím gõ đổi `editedText` ⇒ Vue render lại; bản trước lý
luận *"giá trị render bằng đúng thứ đang có trong DOM nên Vue không ghi"* — **sai**, Vue so
**vnode cũ với vnode mới**, không so với DOM. Vnode cũ mang chuỗi **trước** lượt gõ ⇒ Vue ghi
`textContent`, **dựng lại text node**, caret rơi về **offset 0**, ký tự sau chèn vào đầu.
⇒ Vá bằng `frozenText`: trong lúc một câu đang gõ, **DOM sở hữu văn bản của nó**, giá trị render
bị đóng băng nên Vue không chạm text node đó nữa. Đặt trong `watch(flush:'post')` chứ không ở
`mousedown` — chính lượt watch sau đó sẽ xoá mất giá trị vừa đặt.

**② ĐÃ VÁ — khung viền quanh câu đang sửa, và chữ trông lệch so với bản gốc.** Đó là focus ring
mặc định của trình duyệt trên `contenteditable`; nó chiếm chỗ và đẩy dòng văn vào trong. Sản phẩm
**đã có** chỉ báo *"câu nào đang sửa"* khác hẳn: vạch lề `primary` (UX-DR19) + `⏐` sáng ở
`[data-caret]`. Một vòng focus nữa là cùng một thông tin nói hai lần. Tắt kèm **miễn trừ có tên**
của Kiểm H *(và miễn trừ phải nằm ngay trên dòng khai báo — `exemptAt` đòi khoảng cách ≤ 1 dòng)*.

**④ ĐÃ VÁ — CHỮ ĐÃ DỊCH BIẾN MẤT KHỎI MÀN HÌNH khi bấm xuống dưới** *(Ice, ảnh chụp 2026-08-13)*.
Đây là khuyết tật **nặng nhất** trong ba cái, và nó **do chính bản vá cho ① đẻ ra**: bản vá đó
đóng băng chuỗi hiển thị trong **một biến dùng chung cho mọi câu** (`frozenText`), nên chuỗi của
câu này bị áp lên câu khác. Người dùng thấy bản dịch của mình **bốc hơi** *(dù trên đĩa vẫn còn)*.

🔴 **Bài học, và nó đắt:** ① và ④ có **cùng một nguyên nhân gốc** — **hai chủ sở hữu cho một text
node**. Bản vá cho ① chữa **triệu chứng** *(giữ chuỗi đứng yên)* mà không chữa **nguyên nhân**
*(Vue vẫn điều khiển text node đang được gõ)*, và một bản vá như vậy không trung tính: nó đẻ ra
một khuyết tật nặng hơn thứ nó chữa.

⇒ Lời giải cuối là một **doctrine**, không một bản vá: **DOM sở hữu văn bản bản dịch, Vue không.**
Template render `s.target_text` — **bản LÚC NẠP**, một giá trị chỉ đổi ở lượt nạp lại — nên vnode
của câu **không bao giờ đổi** trong một phiên gõ và Vue **không bao giờ** chạm text node đó. Vế
*"văn bản sống sót một lượt tháo panel"* đổi đường: `restoreEditedText()` chép `editedText` ngược
vào DOM **một lần** sau mỗi lượt dựng lại, bỏ qua câu đang **giữ tiêu điểm thật**
*(`document.activeElement`, **không** phải thuộc tính `contenteditable` — sau một lượt mount lại
thuộc tính còn mà tiêu điểm thì không, và đó đúng là lượt cần khôi phục nhất)*.

**Hai ca hồi quy khoá cả hai khuyết tật** (`tests/frontend/editorTypingZone.test.ts`): **(Chủ: story kế tiếp chạm `PanelFrame.vue`.)**
- *gõ nhiều ký tự KHÔNG dựng lại text node* — khẳng định **danh tính** của text node, không phải
  chuỗi cuối. Một phép kiểm `textContent === 'abc'` vẫn **xanh dưới bản hỏng**, vì mã test tự gán
  chuỗi đúng; thứ người dùng mất là **caret**, và dấu vết đo được của nó là node bị thay.
  ⚠️ Ca này phải mô phỏng bằng `Text.appendData()`, **không** `textContent = …`: gán `textContent`
  tự nó huỷ node con rồi dựng node mới, nên một ca viết vậy đo chính mã test. Nghiệm thu
  **đỏ-rồi-xanh**: trả template về binding phản ứng ⇒ ca đỏ. **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**
- *gõ vào MỘT câu không bao giờ đổi văn bản của câu KHÁC*.

**③ VÁ MỘT PHẦN — *"rất khó click để focus"*.** Ba lượt vá, mỗi lượt đóng một nguyên nhân **thật
và khác nhau**, cả ba cần thiết:
  1. `setAttribute('contenteditable','true')` **đồng bộ** trong `mousedown` — trước đó engine
     quyết định lúc `<span>` chưa soạn thảo được *(xem §ĐÍNH CHÍNH)*;
  2. `nearestSentenceTo()` — bấm vào **khoảng trống** của `.doc` *(dưới câu cuối, cuối dòng, và
     chính chỗ của mọi câu chưa dịch — `<span>` rỗng rộng 0 px)* nay chọn câu gần nhất. Đây là
     thứ Ice yêu cầu đích danh: *"click vào vùng dịch là nó focus để nhập được ngay"*;
  3. `Selection.setPosition()` thay `removeAllRanges()`+`addRange()` — đo được: trên một phần tử
     soạn thảo được **rỗng**, WebKit **bỏ qua** `addRange` *(`type` ở lại `"None"`,
     `execCommand` trả `false`)* nhưng **nhận** `setPosition` *(`type` thành `"Caret"`,
     `execCommand` trả `true`, chữ hạ cánh)*.

🔴 **Chưa đóng:** trong bộ e2e, ca *gõ vào một câu **chưa dịch*** vẫn đỏ — `execCommand` trả
`false` sau một cú bấm do WebDriver lái, dù đúng chuỗi thao tác ấy chạy được khi gọi từ một lượt
`browser.execute` **riêng**. ⚠️ Và bộ đo **đã tự chứng minh nó bóp méo vế này**: một lượt đo đọc
`document.hasFocus() === false` — cửa sổ không được hệ điều hành focus, trạng thái mà một người
dùng thật không bao giờ ở trong. Nên câu hỏi *"còn hỏng với tay người hay không"* **chưa trả lời
được bằng bộ e2e**, và lượt nghiệm thu tiếp theo phải là **Ice gõ tay**.
⚠️ `ensureCaretNextFrame()` giữ lại theo **lý lẽ, chưa theo số đo** *(nó có guard, chỉ chạy khi
tới frame sau vẫn chưa có caret nào)* — đánh dấu như vậy để không ai đọc nó thành đã nghiệm thu.
**Chủ: Story 2.3 (tiếp) — chờ lượt gõ tay của Ice.**

- 🔴 **BẤM CHUỘT VÀO MỘT CÂU KHÔNG ĐẶT ĐƯỢC CARET TRONG WKWebView — vùng gõ lên đúng chỗ, nhưng
  chữ không hạ cánh được.** Đây là phát hiện nặng nhất của Story 2.3, và nó chỉ lộ ra vì story
  dựng một spec e2e chạy trong **cửa sổ Tauri thật**. Chuỗi phép đo *(2026-08-12, WebKit
  605.1.15, chuột thật qua `realClick()`, `document.hasFocus() === true`)*:

  | # | Đo được | Hệ quả |
  |---|---|---|
  | ① | `mousedown`/`mouseup`/`click` tới đúng `<span>`; `pointerdown` **không** tới | bản vá `@mousedown` dời vùng gõ **đúng** — e2e xanh cho vế này |
  | ② | bấm vào văn bản chỉ-đọc ⇒ `getSelection().type === "None"`, `rangeCount = 0`, `activeElement = SECTION.mode` *(không phải `.doc`, dù nó có `tabindex="0"`)* | đường `Selection.anchorNode` của Story 2.2 **không bao giờ chạy tới** trên engine sản phẩm |
  | ③ | `el.focus()` trên `<span contenteditable>` ⇒ `activeElement` vẫn `SECTION.panel` | `focus()` **không** dùng được; chỉ `selection.addRange(...)` đặt được tiêu điểm |
  | ④ | đặt caret trong `nextTick` sau `mousedown` ⇒ engine **thu vùng chọn về không** *(microtask chạy trước `mouseup`)* | lượt đặt caret phải sau `mouseup` |
  | ⑤ | đặt ở `mouseup` ⇒ caret **có thật** nhưng neo **ngoài** câu | một câu **chưa dịch** là `<span>` rỗng, rộng **0 px** ⇒ hit-test theo toạ độ rơi sang phần tử liền kề |
  | ⑥ | siết *"neo phải NẰM TRONG câu"* ⇒ nhánh dự phòng chạy, và caret nó đặt **cũng bị xoá**: `rangeCount = 0`, `activeElement = SECTION.mode` | **còn một lượt dời tiêu điểm nữa** đang xoá vùng chọn |

  ⇒ Thứ xoá vùng chọn là một lượt **dời tiêu điểm về điểm vào của chế độ**, tức hợp đồng focus
  **AD-34 §2** (`src/commands/focus.ts`, Story 1.6). Nó gặp đúng bức tường mà
  `attribution-focus.e2e.mjs` đã đo và đã ghi: *"trong một `section.panel[tabindex="-1"]`, tiêu
  điểm KHÔNG giữ được trên phần tử con dù đặt bằng cách nào"*.

  ⚠️ **Vế *"gõ được"* của engine KHÔNG hỏng** — đo riêng: `document.execCommand('insertText', …)`
  cho `beforeinput` → `input` → chữ hạ cánh. Và `browser.keys()` của bộ e2e **không** gõ được chữ
  *(chỉ `keydown`, không `beforeinput`)* — một **giới hạn của bộ đo**, không một khuyết tật sản
  phẩm; ghi ra để không ai chẩn đoán lại.

  🔴 **Vì sao Story 2.3 KHÔNG tự vá:** lời giải chạm hợp đồng **AD-34** — hoặc `section.mode`/
  `section.panel` thôi giành tiêu điểm khi tiêu điểm đang ở trong một vùng gõ, hoặc vùng gõ khai
  một điểm vào focus riêng ở `FOCUS_OWNERS`. Cả hai là quyết định về một AD mà **Story 1.6 sở
  hữu**, và AC19 của story này nói thẳng: gặp một quyết định story khác sở hữu thì **dừng và báo**.
  **Chủ: Ice** *(phán quyết về AD-34), rồi story thi hành.*
  ⚠️ **Trạng thái hôm nay, nói thẳng:** bề mặt Editor **gõ được trên Blink** *(bàn đo + vitest +
  e2e vế vùng gõ đều xanh)* nhưng **chưa gõ được bằng chuột trên macOS/WKWebView** — tức trên đúng
  nền tảng duy nhất dự án đang chạy. **AC8 không được đánh dấu đạt trọn vẹn.**

- 🔴 **KÝ TỰ RANH GIỚI `⏐` CHIẾM CHỖ Ở MỌI CÂU CHƯA DỊCH, và nó đẩy chữ lệch dần** — Ice bắt
  bằng mắt 2026-08-13 *(hai lần: *"chữ hiển thị lệch so với bản gốc"*, rồi *"chữ 'a' thụt vào"*)*.

  **Đo được, 2026-08-13:** `.sent::after { content: '⏐'; opacity: 0 }` của UX-DR20 vẫn **chiếm
  bề rộng** — `opacity` không gỡ chỗ, khác `display: none`. Mỗi câu **chưa dịch** *(một `<span>`
  rỗng, nhưng `::after` của nó thì không rỗng)* đẩy văn bản phía sau sang phải **9,05 px**:

  | | mép trái câu cuối |
  |---|---|
  | không câu rỗng xen giữa | **72,0 px** |
  | **bốn** câu rỗng xen giữa | **108,2 px** |

  ⇒ Một Chương **mới** có hàng chục câu chưa dịch liên tiếp, nên khoảng thụt cộng dồn tới hàng
  trăm pixel — và nó **co lại dần** khi người dùng dịch xong từng câu, tức bố cục nhảy trong lúc
  làm việc. Đây là lớp khuyết tật chỉ lộ ra ở Story 2.3, vì tới hết 2.2 **mọi** câu đều rỗng nên
  không có gì để so lệch với.

  ⚠️ **Chẩn đoán trước của Dev chỉ đúng MỘT NỬA** — lượt đầu đổ trọn cho focus ring của
  `contenteditable`. Focus ring có thật và đã tắt, nhưng nó chỉ giải thích khung viền; **vệt
  thụt** thì tới từ đây.

  **Bản vá, MỘT dòng:** `.sent:empty::after { content: none }` — một câu rỗng không có chữ nào để
  mà đánh dấu ranh giới. Nó giữ nguyên UX-DR20 cho mọi câu **có chữ**.
  🔴 Nó **thu hẹp một quyết định UX (UX-DR20)** chứ không sửa một lỗi cài đặt — UX-DR20 nói `⏐`
  đánh dấu **ranh giới câu**, và ranh giới giữa hai câu rỗng vẫn là một ranh giới — nên nó cần
  một chữ ký.

  → ✅ **ICE KÝ 2026-08-13, ĐÃ ÁP, ĐÃ ĐO trên cả hai engine:**

  | | 0 câu rỗng | 4 câu rỗng | 40 câu rỗng | đẩy/câu |
  |---|---|---|---|---|
  | **trước** | 72,0 px | 108,2 px | **433,9 px** | **9,05 px** |
  | **sau** | 72,0 px | 72,0 px | **72,0 px** | **0,00 px** |

  *(Blink 151 và WebKit 26.5 cho **cùng** con số tới từng chữ số.)* Và `⏐` của câu **có chữ** đọc
  lại vẫn là `"⏐"` — mệnh đề *"chỉ thu hẹp cho câu rỗng"* được kiểm ở chính lượt đo đó.
  ⚠️ Con số 433,9 px cho 40 câu **lớn hơn nửa bề rộng panel** — nặng hơn hẳn ước lượng ban đầu.

  **Phép đo này nay SỐNG:** `2-3-ban-do-vung-go.html` tự dựng hai dòng văn tạm rồi in
  `Câu chưa dịch đẩy chữ: … px/câu — phải là 0,00` ở mỗi lượt chạy bàn đo. `happy-dom` **không**
  giữ được mệnh đề này *(nó không có bố cục)*, nên đây là đường nghiệm thu đúng của nó (AC25).

- ⚠️ **Vế *"đóng app → mở lại → chữ còn đó"* của Task 7.2 KHÔNG chạm tới được, và không vì bộ đo
  thiếu sức: KHÔNG tồn tại đường mở lại một `.atproj`.** `OpenWorkState` khởi tạo `None` mỗi lượt
  chạy, và cách duy nhất một Tác phẩm được mở là **tạo mới** nó. Màn hình mở lại thuộc **Epic 5**.
  Vế *"chữ còn đó sau khi nạp lại"* nghiệm thu ở
  `segment_contract.rs::typed_text_round_trips_through_the_flush_and_the_load_command` — ghi rồi
  đọc lại qua đúng hai lệnh IPC của sản phẩm. **Chủ: Epic 5** *(nhặt lại cùng lượt dựng đường mở
  lại một Tác phẩm)*.

- ⚠️ **`panic = "abort"` khiến một lần thoát CỨNG không đi qua đường flush lúc thoát** — món nợ
  **kế thừa** từ `close_global_store`/`close_open_work`, story này **không** đóng nó.
  `wire_exit_flush` phủ lượt đóng **bình thường** *(`WindowEvent::CloseRequested`)*, và đó là thao
  tác người dùng chắc chắn nhất trong danh sách của AC3. **Chủ: cùng chủ với món gốc.**

- ⚠️ **Vế *"xác nhận segment"* của AC3 chưa có đường nào chạm tới** — nó cần cột `segment.status`
  và một máy trạng thái, cả hai thuộc **Story 2.5**. AC3 vì thế **không** được đánh dấu đạt trọn
  vẹn ở story này. Ba đường còn lại *(nhịp 2 s · rời segment · đóng Tác phẩm)* và vế thoát ứng
  dụng thì có. **Chủ: Story 2.5.**
  → ✅ **ĐÃ ĐÓNG 2026-08-14 (Story 2.5).** Cách đóng: `editorPanelState.ts::confirmCurrentSegment`
  **`await flushEditorNow()` TRƯỚC** rồi mới `invoke('confirm_segment')`, và một lượt flush trượt
  ⇒ **DỪNG**, không xác nhận *(ký một câu mà lượt lưu vừa trượt là ghi chữ ký cho một văn bản không
  tồn tại trên đĩa)*. Lưới: `tests/frontend/editorConfirmSegment.test.ts` §① — đảo hai dòng đó làm
  hai ca ĐỎ *(đã chạy 2026-08-14)*.
  ⚠️ **GIỚI HẠN THẬT:** mệnh đề này **không** cưỡng chế được ở tầng Rust — `confirm_segment` chỉ
  đọc thứ đã ở trên đĩa và không biết gì về văn bản đang gõ. Một bề mặt tương lai `invoke` thẳng
  lệnh đó sẽ đi vòng qua cả hàm lẫn ca test mà **không cổng nào đỏ**. Ghi ở doc-comment của
  `wire::confirm_segment` và của `config/segment.ts::confirmSegment`.

- ⚠️ **Lệch `32px` / `34px` của chiều cao thanh trạng thái.** `tokens.json:480` và
  `DESIGN.md:283`/`:316` ghi **34px**; `DESIGN.md:132` còn một khối bảng cũ ghi `32px`, và mockup
  `key-screen-workspace.html:73` dựng `.status{height:32px}`. `StatusBar.vue` đọc
  `var(--space-status-height)` = **34px** — số trong bảng token, và `EXPERIENCE.md:312` phân xử
  rằng tài liệu thắng bản dựng. Dev **không** sửa `DESIGN.md` *(tiền lệ quyết định #3 của Ice ở
  Story 1.3)*. **Chủ: Ice.**

- ⚠️ **PHỦ TEST HỒI TỐ cho mã Story 1.x / 2.1 / 2.2 — cố ý KHÔNG làm ở đây.** Bộ chạy test
  frontend ra đời ở story này *(Quyết định #6)*, và một bộ chạy mới luôn mời gọi phủ ngược. Trộn
  nó vào đây làm diff của 2.3 không đọc được một mình *(tiền lệ Ice: cây bẩn trước story đi commit
  riêng)*. **Chủ: chưa gán — cần một story riêng.**
  🔴 **Và lý do của mọi hàng nợ cũ mang câu *"dự án không có bộ chạy test frontend"* nay ĐÃ SAI**
  *(`:141` · `:833-835` · `:877-884` · `:1098`)*. Bản thân các **mệnh đề** ở đó vẫn chưa được phủ,
  nhưng lý do nay là *"chưa ai viết"*, **không** phải *"không chạy được"*. Đừng đọc chúng thành
  điều thứ hai nữa. **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**

- ⚠️ **KHÔNG di chuyển các phép kiểm HÀNH VI từ cổng tĩnh sang vitest** — `check-layout.mjs`
  Kiểm B *(chạy `simulateWrites`)* và `check-commands.mjs` Kiểm C/D/E *(`import()` thẳng
  `src/commands/*.ts`)* **ở nguyên chỗ**. Đó là một lượt tái cấu trúc có rủi ro riêng: bốn phép
  kiểm đó là lưới **hai nền tảng** duy nhất của tầng bàn phím, và một lượt chuyển làm chúng phụ
  thuộc vào một bộ chạy mới thay vì Node thuần. **Chủ: chưa gán — cần một story riêng.** **(Chủ: một story hạ tầng cổng kế tiếp.)**

- ⚠️ **Bàn đo `2-2-ban-do-editor.html:11` còn khai *"Dự án CỐ Ý không có bộ chạy test frontend"*.**
  Lời khai đó **hết đúng** từ 2026-08-12. Ba chỗ mà Task 0b.7 nêu đích danh
  *(`src/commands/registry.ts` · `src/commands/README.md` · `src/i18n/README.md`)* đã được sửa;
  bàn đo của story trước **không** nằm trong danh sách đó nên nó giữ nguyên, và đây là hàng ghi
  lại điều đó. **Chủ: Story 2.4** *(story đó đã nhận món "chụp lại ba ảnh bàn đo 2.2")*.

- ⚠️ **`happy-dom` thiếu ba thứ so với một DOM thật, và danh sách đó là một món nợ ĐO ĐƯỢC.**
  `tests/frontend/support/setup.ts` vá `document.fonts` *(không cài FontFaceSet)* và
  `ResizeObserver` *(có mặt nhưng không bao giờ bắn — không có bố cục thật)*. Hệ quả: mọi mệnh đề
  về **hình học** vạch lề **không** nghiệm thu được ở cây test đó; chúng thuộc bàn đo. Danh sách
  càng dài thì khoảng cách giữa bản mô phỏng và WKWebView càng lớn — đọc nó như một chỉ số, không
  như một danh sách tiện tay. **Chủ: Dev** *(giữ danh sách ngắn, mỗi mục một dòng lý do)*.

---

## Deferred from: code review of 2-3-hop-dong-flush-va-trang-thai-da-luu (2026-08-13)

Lượt review ba tầng song song trên `git diff HEAD` + tệp mới, mốc gốc `6a9777b`. 18 phát hiện sau
gộp trùng; 3 đưa lên Ice, 8 vá được, và **năm** món dưới đây hoãn — mỗi món kèm chủ và kèm lý do
hoãn, không gom thành một câu *(retro §5)*.

- **`restoreEditedText()` quét toàn bộ `.doc` thay vì duyệt theo tập đã sửa.**
  `src/panels/EditorPanel.vue:294` gọi `querySelectorAll('[data-segment-id]')` trên **cả Chương**
  mỗi lượt dựng lại trang, trong khi `editedText` thường chỉ mang vài mục. O(cả Chương) thay cho
  O(số câu đang gõ dở). ⚠️ Trần đã đo của Story 2.2 là **9 850** câu, nên đây cùng một hàng nợ với
  *"ảo hoá danh sách dài"* (`ARCHITECTURE-SPINE.md:888`) và phải được hiệu chỉnh **cùng lượt** với
  nó — vá lẻ ở đây là tối ưu một hằng số trước khi biết bậc độ lớn có đổi không. **Chủ: Story 2.4.**

- **`nearestSentenceTo()` ép bố cục lại trên mỗi cú bấm hụt.**
  `src/panels/EditorPanel.vue:565`, gọi từ `onDocMouseDown`/`onDocMouseUp`. Mỗi cú bấm rơi vào
  khoảng trống của `.doc` duyệt **từng** câu và gọi `getClientRects()`/`getBoundingClientRect()`.
  🔴 Điểm đáng ghi không phải chi phí — nó là chỗ **duy nhất** của lượt này không kèm số đo, giữa
  một story mà mọi quyết định khác đều có bảng đo chống lưng. Hàm này ra đời ở lượt vá 2026-08-13
  *(một trong ba nguyên nhân thật của "khó click để focus")*, tức nó **chưa từng đi qua** bàn đo
  hai engine của Task 0.1. **Chủ: Story 2.4** *(đo cùng lượt với trần NFR2)*.

- **Cửa rà giấy phép NFR15 không có một cổng máy nào.**
  `src/commands/registry.ts` · `src/commands/README.md` · `src/i18n/README.md` · `vitest.config.ts`
  cùng khai *"ba gói đã đi qua đúng cửa rà giấy phép"*, nhưng không `check-*.mjs` nào xác minh lượt
  rà đã xảy ra — `check-deps.mjs` canh thư viện thu thập dữ liệu, không canh tương thích GPLv3.
  ⚠️ Món này **có sẵn từ trước lượt này**: NFR15 xưa nay là một quy trình người, và Task 0b đã đi
  qua nó đúng cách *(ba tệp giấy phép thật đã mở, 811 dòng của `vitest` đọc ra 27 gói vendor)*.
  Ghi ra vì gói **thứ tư** sẽ gặp lại đúng cửa này, và lúc đó trí nhớ người là thứ duy nhất canh.
  **Chủ: Ice** *(quyết định có biến cửa này thành cổng máy hay không)*.

- **Tiêu điểm/caret không khôi phục sau một lượt dựng lại component.**
  Người dùng đang gõ dở một câu, một lượt đổi preset bố cục tháo và dựng lại `EditorPanel.vue`:
  `restoreEditedText()` chép đúng chữ trở lại, nhưng watcher khôi phục caret
  (`src/panels/EditorPanel.vue:363`) **không chạy** — nó nghe **giá trị** `editorCaretSegmentId`,
  mà giá trị đó không đổi qua một lượt remount; và `savedCaret` là biến của component nên đã về
  `null`. Câu hiện đúng chữ nhưng mất tiêu điểm bàn phím; phím kế tiếp rơi vào chỗ trình duyệt tự
  chọn. 🔴 **Vì sao hoãn chứ không vá:** lời giải là **giành** tiêu điểm lúc mount, và đó đúng là
  thứ `PanelFrame.vue::focused` cùng chốt chống-rơi-`body` của `focus.ts` tồn tại để **không** làm
  — cùng doctrine mà §ĐÍNH CHÍNH 2026-08-13 vừa xác lập lại bằng phép đo. **Chủ: Ice.**

- **Chưa đo caret có NHÌN THẤY trên một câu rỗng sau khi bỏ `min-width`.**
  Khối CSS *"KHÔNG ép câu rỗng chiếm chỗ"* cộng `.sent[contenteditable='true']{outline:none}` để
  lại một `<span>` rộng **0 px** không viền. e2e và bàn đo đều chỉ khẳng định chữ **hạ cánh được**
  (`execCommand('insertText')` trả `true`), không khẳng định caret **hiện ra** trước khi gõ. ⚠️ Đây
  là cùng vùng với ca đỏ đã công bố *(lượt gõ ĐẦU TIÊN vào một câu chưa dịch)*, và nó là ca
  **thường nhất** của tính năng — mọi Chương mới mở ra đều toàn câu rỗng. Đo nó cùng lượt với ca
  đỏ đó, đừng đo riêng. **Chủ: cùng chủ với ca đỏ** *(chờ phán quyết Ice)*.

---

## Đo thêm về ca đỏ `<span>` rỗng — code review 2026-08-13, **KHÔNG kết luận được**

🔴 **Đọc mục này trước khi ai đó lại đi chẩn đoán ca đỏ ấy.** Lượt đo dưới đây tốn tám lượt chạy
e2e và nó **không** đóng được câu hỏi — nhưng nó thu hẹp được, và nó để lại một cái bẫy đã sập
một lần.

### Cái ĐO ĐƯỢC và tái lập được

**Chuỗi hành động `pointer` của WebDriver để cửa sổ mất tiêu điểm TẦNG HỆ ĐIỀU HÀNH.** Đo ngay
trước lượt gõ, nhiều lượt, nhất quán:

```
hasFocus: false · selType: "None" · rangeCount: 0 · activeElement: SECTION.mode · zoneWidth: 0
```

`document.hasFocus() === false` là một trạng thái **người dùng thật không bao giờ ở trong**, và
WebKit **không giữ vùng chọn** trong một cửa sổ như vậy. ⇒ ca đỏ hiện nay **không phân biệt được**
*"sản phẩm hỏng"* với *"bộ đo hỏng"*. Đây là vế mà `deferred-work.md` đã ngờ từ 2026-08-12; nay nó
có số.

Ép cửa sổ lên trước bằng `osascript` *(System Events, `set frontmost of process "AuraTranslate"`)*
**đưa `hasFocus` về `true`** — tái lập được. Đặt lời gọi ở cuối `realClick` thì `hasFocus` giữ
`true` suốt, nhưng tiêu điểm **DOM** vẫn trôi khỏi `SPAN.sent` sang `SECTION` trước lượt gõ
*(`selType` về `"None"`)*. Nên hai thứ bị mất là **hai thứ khác nhau**, và mới đóng được một.

### 🔴 Cái KHÔNG kết luận được, và vì sao phải nói ra

**Đúng MỘT lượt chạy** cho kết quả xanh trọn vẹn: với `osascript` gọi ngay trước lượt gõ,
`activeElement` = `SPAN.sent`, `selType` = `"Caret"`, `rangeCount` = 1, `execCommand` trả `true`,
chữ hạ cánh và đi vào `project.db` — **2/2 xanh**, trên đúng một `<span>` rỗng rộng 0 px.

⚠️ **Lượt đó KHÔNG tái lập được.** Bảy lượt còn lại đỏ, kể cả các lượt có `osascript` đặt đúng
chỗ, chạy một spec, và nhịp chờ dài hơn. Tổng: **1 xanh / 7 đỏ**.

⇒ Câu *"sản phẩm không hỏng, ca đỏ chỉ là dấu vết bộ đo"* **đã được viết ra trong lượt review này
rồi RÚT LẠI**: nó rút từ **một** quan sát, trên đúng bộ đo mà `sprint-status.yaml` đã ghi là chập
chờn *(8 lượt gần nhất 6 xanh / 2 đỏ)*. Một mẫu bằng 1 trên một bộ đo nhiễu không phải một phép đo
— đó là lớp lỗi mà luật *"đo trước khi tin"* của kho tồn tại để chặn, và lượt này đã mắc nó.
Mã dựng trên kết luận đó *(một helper `restoreOsFocus` trong `e2e/support/pointer.mjs`)* **đã được
gỡ**, vì một doc-comment khai *"sản phẩm không hỏng"* dựa trên một lượt chạy là một lời khai sai
nằm vĩnh viễn trong kho.

### Ba việc kế tiếp, theo thứ tự giá trị

1. **Ice gõ tay** — vẫn là lượt nghiệm thu rẻ nhất và dứt khoát nhất, đúng như bản ghi 2026-08-12
   đã kết luận. Một câu **chưa dịch**, bấm rồi gõ. Kết quả trả lời trọn câu hỏi mà tám lượt e2e
   không trả lời được.
2. **Chạy ca đỏ ~10 lượt liên tiếp và đếm** — với `osascript` đặt ngay trước lượt gõ. Nếu tỷ lệ
   xanh ≫ 1/8 thì bộ đo là nguyên nhân chính; nếu nó ở lại quanh 1/8 thì lượt xanh kia là nhiễu và
   sản phẩm **thật sự** có một khuyết tật. **Đừng kết luận trước khi có phân phối**, không phải
   trước khi có một lượt.
3. **Đo vì sao tiêu điểm DOM trôi `SPAN.sent` → `SECTION`** giữa cú bấm và lượt gõ. Câu hỏi này
   **chưa ai đặt** trước lượt review, và nó độc lập với vế `hasFocus`. Nếu lượt trôi ấy cũng xảy
   ra với tay người thì nó là khuyết tật sản phẩm thật, và nó **không** phải ca `<span>` rỗng.

**Chủ: Story 2.3 (tiếp) — chờ lượt gõ tay của Ice.**

---

## Bàn đo Story 2.4 chỉ gõ được ASCII — NFR2/NFR18 chưa chạm tiếng Việt có dấu và chưa chạm IME

**Đo được 2026-08-13**, không suy: gõ `⟦42⟧ Trời hôm nay trong xanh lạ thường.` bằng đường của
Quyết định #2(c) *(`osascript ... keystroke`, sự kiện phím ở tầng hệ điều hành)*, kho nhận
`a42a Trai ham nay trong xanh la thaang.` — `osascript` đi qua **bố cục bàn phím hiện hành** nên mọi
ký tự ngoài ASCII bị bẻ.

**Ice chốt 2026-08-13:** giữ **phím thật** *(để `keydown → beforeinput → input` chạy trọn — đó là
thứ NFR2 đo)*, đổi chỉ số truy vết sang ASCII `[n]`, và **ghi nợ phần tiếng Việt** thay vì đổi sang
clipboard *(dán là một sự kiện `paste`, không phải chuỗi phím người dùng đi qua)*.

**Cái CHƯA đo, và không được đọc bảng NFR2/NFR18 của story 2.4 rộng hơn chỗ này:**

- Văn bản có **dấu tiếng Việt** trên đường gõ — mọi số của 2.4 đọc ra từ văn bản **ASCII**. **(Chủ: Story 2.4.)**
- Đường **IME**, và cùng với nó là ca *"xoá lùi qua đầu câu"* — ca mà chính tệp này xếp là **ca
  thủng cao nhất theo dự đoán**, và là lý do `type-driver.sh` cố ý đòi tiếng Việt có dấu ngay từ đầu.

**Ba đường mở lại, mỗi đường một cái giá đã cân:**

| Đường | Được | Mất |
| --- | --- | --- |
| Bộ gõ tiếng Việt tầng OS *(VietTelex…)* | phím thật **và** dấu thật | thêm một phần mềm bên thứ ba vào chính máy đo — một biến chưa đo |
| `pbcopy` + `⌘V` | nội dung thật | đo `paste`, không đo gõ ⇒ đổi **nghĩa** con số NFR2 |
| Bộ bơm phím phát Unicode ở tầng CoreGraphics *(`CGEventKeyboardSetUnicodeString`)* | phím thật **và** Unicode thật | phải viết, và phải tự nghiệm thu là nó thật sự đi qua `beforeinput` |

🔵 Đường thứ ba là đường sạch nhất về mặt phương pháp và **chưa ai thử**.

**Chủ: Ice** · mở khi cần một bảng NFR2 nói được về tiếng Việt.

---

## Deferred from: Sprint Change Proposal 2026-08-13 (FR21 thu hẹp)

- 🔵 **AC23 của Story 2.3 ĐỔI TỪ MỘT PHÉP ĐO THÀNH MỘT MỆNH ĐỀ.** AC23 hỏi *"Auto-Lookup còn
  chạy trên bề mặt Editor không?"* và đo ra **còn chạy**, đóng theo nhánh hợp lệ. Nó **không
  bao giờ hỏi "có NÊN chạy không?"** — Ice đặt câu hỏi đó ngày 2026-08-13 và trả lời:
  **không**, vì bề mặt Editor chứa tiếng Việt đã dịch còn từ điển nhúng là zh→vi / en→vi, nên
  một lượt tra ở đó trả **0 hàng, 0 lỗi, 0 ms** rồi **thay mất** kết quả vừa tra từ Panel
  Source. Phép đo cũ **không sai**; nó không phủ câu hỏi này.
  ⇒ AC23 nay đọc: *"Editor KHÔNG phát lượt tra từ điển"*. Nghiệm thu: `EditorPanel.vue` +
  `AiTranslationPanel.vue` mang vai `'display'` · `tests/frontend/editorAutoLookup.test.ts`
  (đã đảo, **kèm một ca đối chứng dương**) · `check-commands.mjs` Kiểm F ③.
  **Không món nợ nào mở ra từ mục này** — ghi để retro Epic 2 thấy được vì sao một AC đã đóng
  lại đổi nghĩa. Chi tiết: `planning-artifacts/sprint-change-proposal-2026-08-13.md`.
  → KHÔNG LÀM 2026-08-19 (Story 2.13) — mục tự khai "Không món nợ nào mở ra từ mục này": đây là
  một ghi chú lịch sử giải thích vì sao AC23 đổi nghĩa, không phải một việc chờ làm. Không có
  chủ vì không có việc để giao chủ.

- ⚠️ **Chủ: Story 3.3 (FR48) và Story 7.7 (FR60) — điều kiện khởi hành.** Vai `'display'` của
  `AiTranslationPanel.vue` / `EditorPanel.vue` tắt **đúng một** đường: `currentSelectionText()`,
  tức tra từ điển. Nó **KHÔNG** tắt việc bề mặt được đăng ký. Hai story trên đọc vùng chọn ở
  hai panel đó bằng đường của **riêng chúng** *(cả hai là lệnh người dùng gọi —
  `epics.md:2554` "gọi lệnh thêm thuật ngữ" · `epics.md:5034` "người dùng gọi lệnh
  Concordance")*, **không** qua `currentSelectionText()`.
  🔴 Đọc nhầm `'display'` thành *"không lấy được chữ"* sẽ dẫn tới một lượt "sửa" gỡ đăng ký
  hoặc lật vai — và `epics.md:2553` đã liệt kê **Panel Lookup** (vai `display` từ 1.18) trong
  chính danh sách FR48, nên tiền lệ đã có sẵn. Gỡ đăng ký ⇒ `SELECTION_SURFACE_FLOOR = 7` đỏ;
  lật vai ⇒ Kiểm F ③ đỏ. Cả hai đường đều có lưới, nhưng lưới không giải thích được **vì sao**
  — mục này làm việc đó.
  → 🟡 **Vế Story 3.3 ĐÃ ĐÓNG 2026-08-20; vế Story 7.7 (FR60, Concordance) VẪN MỞ.**
  `src/panels/selectionContract.ts::currentSelectionTextForGlossaryQuickAdd` là đường RIÊNG
  mà mục này dự đoán trước — nó dùng lại `surfaceFor()` (đã hỏi `'display'` trước `'source'`)
  nhưng KHÔNG lọc theo `role`, nên cả bốn bề mặt FR48 đều lấy được chữ, kể cả hai bề mặt vai
  `'display'` mà mục này nói tới. `glossary.add_term` (`commands/index.ts`) gọi đường đó, không
  qua `currentSelectionText()`. **Chủ phần còn lại: Story 7.7** — dựng đường đọc riêng tương tự
  cho lệnh Concordance khi story đó tới lượt.

- 📝 **Vế bằng MẮT của lượt sửa này chưa chạy — CHỦ: Ice.** Ba mệnh đề phải xác nhận trên
  `tauri dev` THẬT *(mọi bằng chứng ở trên là vitest trên happy-dom + cổng tĩnh, không phải
  WKWebView)*: ① bôi đen tiếng Việt trong Editor ⇒ Panel Lookup **giữ nguyên** nội dung;
  ② `Mod+Alt+L` trong Editor ⇒ **không** lượt tra nào *(Ice chốt 2026-08-13 bỏ luôn cả đường
  thủ công)*; ③ bôi đen ở Panel Source ⇒ tra bình thường, **không hồi quy**.
  ⚠️ Cùng món nợ hai nền tảng mà 1.6/1.14/1.16/1.17/1.18/2.2/2.3 đã để lại — chưa đo trên
  Windows.
  → 🟡 **ĐÓNG MỘT NỬA 2026-08-13.** Ice chạy tay và báo **pass** cả ba mệnh đề ① ② ③ trên
  `tauri dev` thật. **Nửa còn HỞ: Windows/WebView2 — chưa chạy một lần nào.** Không làm tròn
  lên ✅: `EXPERIENCE.md`/`deferred-work.md:145` đã ghi tên đúng lỗi *"nghiệm thu trên một
  engine rồi viết «tương đương»"*, và lượt sửa này chạm **vùng chọn trong `contenteditable`**
  — đúng một trong hai chỗ có tiền sử lệch engine mà Story 2.3 §9 gọi đích danh. Nửa Windows
  nhập vào lượt rà hai nền tảng cuối dự án *(Ice chốt 2026-08-12: trọn phần Windows dời về
  cuối)*, **không mở một món nợ mới**.

- ⚠️ **BỐN trích dẫn trong story file còn chép nguyên văn mệnh đề đã sửa — CHỦ: Ice, quyết
  định còn MỞ.** `grep` nghiệm thu của proposal tìm ra bốn chỗ mà §4.10 **không liệt kê**:
  `1-18-auto-lookup.md:37` và `1-18b-tach-tu-tieng-trung-tab-han-viet.md:40` *(**hai hàng
  Change Log**, cùng ngày 2026-08-07)* · `1-18-auto-lookup.md:989` và `1-18b-…md:622`
  *(**hai danh sách `[Source: …]`**, trỏ đích danh `EXPERIENCE.md:131`)*. Cả bốn chép
  *"không được thiết kế lại cho khác đi"*.
  🔵 **Đính chính lúc đóng:** bản đầu của mục này viết **BA** và bỏ sót `1-18b:40`. Lỗi đếm
  đến từ việc `grep` cụm đầy đủ **không khớp** `1-18:989` *(cụm bị ngắt dòng ở đó)*, nên hai
  lượt quét khác nhau cho hai danh sách khác nhau và mình cộng nhầm. Phân loại **không đổi**
  — vẫn 2 ảnh chụp + 2 con trỏ — nên phán quyết của Ice áp y nguyên. Ghi ra vì một con số sai
  trong sổ làm hỏng lòng tin vào cả sổ.

  🔴 **Hai loại khác nhau, và chỉ một loại đáng lo.** Hàng Change Log là một **ảnh chụp có
  ngày** — nó ghi đúng thứ dev đọc hôm 2026-08-07, sửa nó là viết lại lịch sử. Nhưng hai dòng
  `[Source: …]` là **con trỏ tới một tài liệu SỐNG**: ai theo con trỏ đó ở Epic 3/4 sẽ thấy
  một văn bản khác hẳn, và rủi ro thật là họ trích **bản trong story** như thể nó còn hiệu lực
  — đúng lớp "chú thích cũ hơn mã" mà `project-context.md` §Bẫy tài liệu vừa đặt tên.

  Hai phương án, **cả hai đều hợp lệ**, Amelia không tự chọn:
  **(a)** nối một dòng 🔵 vào hai dòng `[Source: …]` *(«mệnh đề này đã sửa 2026-08-13 — xem
  sprint-change-proposal»)*, để nguyên hàng Change Log. Rẻ, chặn đúng chỗ rủi ro.
  **(b)** để nguyên cả ba, coi story file là ảnh chụp tuyệt đối; dựa vào việc `EXPERIENCE.md`
  **giữ nguyên bản cũ trong câu giải thích 🔵** nên người theo con trỏ vẫn tìm thấy mệnh đề cũ
  và đọc được vì sao nó đổi.

  ⚠️ Chưa làm gì cả — mục này ghi để nó không mồ côi. Không chặn lượt vá 2026-08-13.
  → ✅ **ĐÃ ĐÓNG 2026-08-13 — Ice chốt phương án (a).** Nối một khối 🔵 vào **hai** dòng
  `[Source: …]` *(`1-18-auto-lookup.md:989` · `1-18b-…md:622`)*: nói rõ con trỏ nay trỏ vào một
  văn bản khác, mệnh đề cũ đã được **sửa chứ không bị vượt**, và câu trích tại chỗ là **ảnh
  chụp 2026-08-07** — đừng trích lại như một mệnh đề đang hiệu lực. **HAI hàng Change Log
  (`1-18:37` · `1-18b:40`) để NGUYÊN**, đúng lý do phân loại ở trên: chúng là ảnh chụp có
  ngày, không phải con trỏ. 🔴 Vì sao không xoá câu trích cũ ở cả hai chỗ: một con trỏ mất câu
  trích thì người sau không đối chiếu được **cái gì đã đổi**, và lịch sử của một mệnh đề là
  bằng chứng cho mệnh đề kế tiếp.

---

## Deferred from: 2-5-xac-nhan-segment-va-may-trang-thai (2026-08-14)

- 🔴 **Hợp âm `⌘↵` (`Mod+Enter`) SẼ xung đột với lệnh *"xác nhận nhập"* của Epic 6.**
  Story 2.5 đăng ký `editor.confirm_segment` với `Mod+Enter` — hôm nay hợp âm đó **chưa ai
  chiếm**, và `EXPERIENCE.md:169` đã hẹn đúng `⌘↵` cho *"xác nhận nhập"* ở màn xem trước của
  đường nhập. Hai thao tác **cùng ngữ nghĩa "ký duyệt"**, khác bề mặt.
  ⚠️ `check:commands` kiểm trùng hợp âm **trên TOÀN BỘ registry, không theo chế độ**. Ngày Epic 6
  đăng ký lệnh kia cũng bằng `⌘↵`, cổng sẽ **ĐỎ** và một trong hai phải nhường. Ghi ở đây để nó
  **không lộ ra dưới dạng một cổng đỏ không ai hiểu**. Ba đường sẽ có lúc đó: đổi hợp âm của lệnh
  nhập · đổi hợp âm của lệnh xác nhận · hoặc dựng khái niệm *"hợp âm theo chế độ"* trong registry
  *(lượt đắt nhất, và nó chạm AD-34)*. **Chủ: Story 6.2.**

- ⚠️ **AD-35 vế (c) — *"xác nhận ⇒ flush trước"* — KHÔNG cưỡng chế được ở tầng Rust.**
  `commands::segment::wire::confirm_segment` chỉ đọc thứ **đã ở trên đĩa**; nó không biết gì về
  văn bản đang gõ trong webview. Thứ tự *flush → confirm* vì thế sống ở
  `editorPanelState.ts::confirmCurrentSegment`, và lưới duy nhất là
  `tests/frontend/editorConfirmSegment.test.ts` §①. Nó canh **chỗ gọi đó**, không canh mọi chỗ gọi
  tương lai: một bề mặt mới `invoke('confirm_segment')` thẳng sẽ ký một văn bản **cũ hơn** thứ
  người dùng đang nhìn, và **không cổng nào đỏ**. Lời giải nếu ngày đó tới: một cổng tĩnh cấm
  `invoke('confirm_segment')` ngoài `src/config/segment.ts` — cùng khuôn `no-restricted-syntax` đã
  dùng cho `.click()` trong `e2e/**`. **Chủ: story nào dựng bề mặt xác nhận thứ hai.**

- ⚠️ **Từ 8 LÀN trở lên, máng vạch lề 22px hết chỗ** — xem mục đầy đủ ở §Deferred from 2-2, ngay
  dưới hàng *"hai câu cùng một dòng"* vừa đóng. **Chủ: Ice.**
  → ✅ **ĐÃ ĐÓNG 2026-08-15 (Story 2.5b) — biến mất theo cấu trúc.** Lý do đầy đủ ở mục gốc.

- 🔴 **`browser.keys()` ĐÁNH RƠI `Meta` đúng ở phím `Enter`, và CHỈ ở đó — giới hạn của BỘ ĐO.**
  Đo 2026-08-14 trong chính cửa sổ e2e, listener `keydown` pha capture trên `window`, một lượt chạy:

  | Lượt gọi | `code` nhận được | `event.metaKey` |
  |---|---|---|
  | `browser.keys(['Meta', '1'])` | `Digit1` | **`true`** |
  | `browser.keys(['Meta', '2'])` | `Digit2` | **`true`** |
  | `browser.keys(['Meta', 'Enter'])` | `Enter` | 🔴 **`false`** |

  ⇒ Hợp âm `Mod+Enter` **không bao giờ khớp** qua `browser.keys`: `sameMods` thấy một `Enter` trần.
  Hai hợp âm đối chứng đi qua **cùng đường mã, cùng cửa sổ, cùng lượt chạy** và mang đủ phím bổ
  trợ, nên đây là **bộ đo**, không phải sản phẩm — mệnh đề đó được phân xử bằng đối chứng, không
  bằng suy luận.
  ⚠️ Hệ quả: `e2e/specs/editor-confirm-segment.e2e.mjs` phát một `KeyboardEvent` **tổng hợp**. Nó
  đo trọn chuỗi *keymap → registry → command → IPC → `project.db`* trong WKWebView thật, nhưng
  **không** đo được rằng một phím **vật lý** `⌘↵` sinh ra đúng sự kiện đó — cùng hạng với vế *"một
  phím vật lý sinh ra `beforeinput`"* mà spec của Story 2.3 đã ghi.
  **Chủ: Story 1.22** *(nó sở hữu bộ chạy e2e và ba giới hạn chưa đóng của nó)*.

- ⚠️ **Một `[Vue warn] Unhandled error during execution of native event handler` ở `EditorPanel`,
  BẮT NGUYÊN VĂN, CHƯA CHẨN ĐOÁN.** Xuất hiện trong mọi lượt chạy e2e của Story 2.5 *(kể cả lượt
  chẩn đoán không chạm gì tới đường xác nhận)*, ngay sau một lượt `realClick` vào một câu. Cây
  component: `PanelFrame(panel.editor) → EditorPanel → DockviewPortals → …`.
  🔴 **Chưa đặt tên nguyên nhân, và cố ý không đoán** — nó **không** làm ca nào đỏ, và ba ứng viên
  hiển nhiên *(`onDocMouseDown`, `onEditKeydown`, `onEditInput` — cả ba là native handler trên
  đường chuột của Story 2.3)* chưa cái nào được đo. Ghi ở đây theo đúng luật *"gặp một lượt lạ thì
  BẮT NGUYÊN VĂN TRƯỚC, đừng chẩn đoán từ trí nhớ"*.
  ⚠️ **Chưa xác định nó có TRƯỚC Story 2.5 hay không** — lượt đo đầu tiên nhìn thấy nó là lượt của
  story này. **Chủ: Story 2.3** *(chủ của đường chuột và của ba handler đó)*.

- ⚠️ **`EXPERIENCE.md:105-113` chưa có hàng *"đã dịch, chưa xác nhận"*.** Quyết định #3 đã được Ice
  ký và mệnh đề đã viết vào doc-comment của `resolveSegmentRule` cùng một ca vitest, nhưng **tài
  liệu quy hoạch chưa được sửa** — sửa một tài liệu tầng nguyên tắc là một lượt riêng của Ice, dev
  không sửa `EXPERIENCE.md`/`epics.md`. **Chủ: Ice.**

- ⚠️ **Bảng `segment_version` KHÔNG có index, và đó là một quyết định chứ không một lượt quên.**
  Story 2.5 **chỉ ghi**, không đọc — không đường sản phẩm nào truy vấn bảng đó ở story này, nên
  một index ở đây là tối ưu cho một đường đọc **chưa ai đo**. Cùng luật mà `SEGMENT_TARGET_TEXT_DDL`
  đã ghi cho `target_text`. Story 2.6 mang đường đọc *(lịch sử theo `segment_id`, sắp theo thời
  điểm)*, nên nó mang index **cùng lượt** — đúng cách bước 5 mang `idx_segment_chapter_ord` cùng
  lúc với đường đọc cần nó. **Chủ: Story 2.6.**
  → ✅ **ĐÃ ĐÓNG 2026-08-16 (Story 2.6).** Bước di trú **10** mang
  `CREATE INDEX idx_segment_version_segment_created ON segment_version (segment_id, created_at DESC)`
  — Quyết định #7 đường (a), Ice ký. Index tới **cùng lượt** với đường đọc biện minh cho nó
  (`commands/segment.rs::read_segment_history`), đúng như món nợ này đòi.
  🔴 Và nó tới bằng một **bước mới** chứ không bằng một dòng thêm vào hằng của bước 7: một
  `project.db` đã ở v7 không bao giờ chạy lại hằng đó, nên sửa tại chỗ cho ra **hai lược đồ
  khác nhau mang cùng số 7** — vết sẹo số 4 ở một hình dạng êm hơn *(vết sẹo cũ ít nhất còn làm
  `Store::open` từ chối; lượt này thì im lặng)*.
  Hai ca hợp đồng đứng canh, cả hai đã chạy đỏ-rồi-xanh trên hai đòn bẩy *(đổi tên index · đổi
  chỗ hai cột)*. ⚠️ Ca hình dạng đọc `pragma_index_info` chứ **không** so chuỗi DDL — đo được:
  một index sai thứ tự cột vẫn chứa cả `ON SEGMENT_VERSION` lẫn `CREATED_AT DESC`, nên một phép
  `contains()` sẽ **xanh trên đúng thứ nó tồn tại để bắt**.

- ⚠️ **`updated_at` của `segment` KHÔNG đổi ở lượt xác nhận.** Cột đó mang nghĩa *"mốc sửa **văn
  bản**"* — nó do `save_segment_targets` sinh, và `SEGMENT_DDL` phân biệt nó với `created_at`
  (*"mốc TẠO, không phải mốc sửa"*). Một lượt ký không sửa một ký tự nào, và thời điểm ký có chỗ
  ghi riêng chính xác hơn: `segment_version.created_at`. ⚠️ Ghi ra vì **Story 2.6 sẽ đọc cả hai
  mốc** và phải biết chúng nói hai chuyện khác nhau. **Chủ: Story 2.6** *(xác nhận lại mệnh đề này
  khi dựng màn hình lịch sử)*.
  → ✅ **ĐÃ ĐÓNG 2026-08-16 (Story 2.6).** Mệnh đề **đã xác nhận lại bằng cách đọc mã**, không
  chép: câu ghi của `confirm_segment` là `UPDATE segment SET status = ?1 WHERE id = ?2` —
  **không** có `updated_at`. ⇒ *"một lượt ký không sửa một ký tự nào nên nó không đụng
  `updated_at`"* vẫn đúng nguyên văn.
  Màn hình lịch sử đọc `segment_version.created_at`, và mệnh đề được ghi vào doc-comment của
  chính đường đọc kèm hệ quả đo được của việc dùng nhầm: `updated_at` sẽ cho một danh sách mà
  **mọi hàng mang cùng một mốc**, và mốc đó là lần gõ cuối chứ không phải lần ký.
  🔵 **Một vế MỚI mà món nợ gốc chưa nói tới, phát hiện lúc dựng đường ghi:** lượt **khôi phục**
  thì **CÓ** đụng `updated_at`, và nó phải đụng — khôi phục **sửa văn bản thật**, nên nó rơi
  đúng vào nghĩa *"mốc sửa văn bản"* mà `SEGMENT_DDL` khai cho cột đó. Hai lệnh ghi, hai hành vi
  ngược nhau trên cùng một cột, và cả hai đều đúng: `confirm_segment` **không** đụng vì nó không
  sửa chữ; `restore_segment_version` **có** đụng vì nó sửa.

- 🔴 **Story 2.5 phá một giả định hiệu năng của cả Epic 2 — đã ĐO và đã VÁ trong story, ghi lại
  vì nó đổi cách đọc mọi số cũ.** Tới hết Story 2.3, `wanted` của `measureGutterRules` có **nhiều
  nhất MỘT** phần tử *(chỉ `primary` có nguồn dữ liệu, và caret chỉ có một)*, nên mọi số đo vạch
  lề của bàn đo 2.2 đọc ở cột *"1 vạch"* **5,0–8,5 ms**. Story 2.5 cho `confirmed` một nguồn ⇒
  `wanted` nay chứa **mọi câu đã xác nhận**, tức lớn theo tiến độ dịch của chính người dùng.
  ⚠️ **Đo 2026-08-14** (Node 22.22.2, 9.850 vạch — Chương lớn nhất có thật, ~3 câu mỗi dòng, ba
  lượt): bản đầu của `assignGutterLanes` là `O(n²)` và tốn **482,4 / 254,5 / 261,6 ms**, tức vượt
  trần một frame của NFR2 *(50 ms)* **5–10 lần**, chỉ riêng CPU. Đã vá bằng **quét đường**:
  **8,3 / 5,2 / 4,3 ms**, kết quả **giống hệt** trên cùng bộ dữ liệu.
  🔴 **PHẦN CÒN HỞ:** số trên đo bằng **Node**, không phải WKWebView, và nó **không** gồm lượt
  `getClientRects()` lẫn lượt bố cục đi kèm. Vế *"một frame thật có vượt 50 ms không, trên một
  Chương đã dịch xong"* **chưa ai đo**, và Story 2.5 **không tự chấm NFR2 đạt**.
  **Chủ: Story 2.4** *(nó sở hữu bộ đo NFR2/NFR18 — đừng dựng bộ đo thứ hai)*.

## Deferred from: code review of 2-5-xac-nhan-segment-va-may-trang-thai (2026-08-14)

_Lượt rà ba lớp song song trên `git diff HEAD` (baseline `8245a17`). Bảy phát hiện, năm đã vá
trong chính lượt rà; hai món dưới đây **không** nghiệm thu được ở Story 2.5 nên đi vào đây kèm chủ._

- ⚠️ **AC1 KHÔNG đạt ở câu cuối mỗi Chương — và nó không có lời giải trong phạm vi 2.5.** AC1 đòi
  *"trạng thái chuyển sang đã xác nhận **và vạch lề chuyển `confirmed`**"*. Lời giải đã ký (Quyết
  định #1, đường (a)) là **dời con trỏ sang câu kế tiếp**, vì `resolveSegmentRule` cho `primary`
  thắng `confirmed` và thứ tự đó là một quyết định 🔴 không được đảo. Ở **câu cuối Chương không có
  câu kế** ⇒ `confirmCurrentSegment` không dời được ⇒ vạch **ở lại `primary`** cho tới khi người
  dùng tự đi chỗ khác. `segment.status` trong CSDL thì **đúng**; chỉ vế thị giác của AC1 hụt.
  ⚠️ Không phải ca hiếm: nó xảy ra **đúng một lần mỗi Chương**, ở đúng câu cuối.
  ⚠️ Khuyết tật đã được ghi bằng chữ tại `src/panels/editorPanelState.ts:503-504` từ lượt dựng,
  nhưng **chưa vào sổ này** — tức nó là một mệnh đề không có chủ, đúng thứ sổ này tồn tại để chặn.
  **Chủ: Story 2.10** *(điều hướng segment — nó **dùng lại** đường dời con trỏ tối thiểu của 2.5
  chứ không dựng đường thứ hai, nên nó là chỗ duy nhất trả lời được câu "đi đâu khi hết Chương")*.

  → 🟡 **ĐÓNG MỘT NỬA 2026-08-18 (Story 2.10, Quyết định #6 đường (c) — Ice ký).**

  **Cách đóng:** `confirmCurrentSegmentUnguarded` nay ghi cờ `kyTrungCauCuoi` ở đúng nhánh biết
  sự thật (`following === undefined`), và cửa có khoá của `confirmCurrentSegment` biến nó thành
  `editorNavNotice = 'confirmed-last'` ⇒ thanh trạng thái nói *"Đã xác nhận câu cuối Chương. Con
  trỏ ở nguyên vì không còn câu nào phía dưới."*. ⇒ Người dùng **biết** chuyện gì vừa xảy ra.

  🔴 **PHẦN CÒN HỞ, và nó là đúng phần AC1 đòi:** vạch lề **vẫn `primary`**, không `confirmed`.
  Đóng bằng **thông tin**, không bằng **màu**. Hai đường đóng trọn đều bị loại, mỗi đường một lý
  do đã ghi:
  - **Đảo thứ tự `primary`/`confirmed` trong `resolveSegmentRule`** — 🔴 quyết định có chữ ký, không đảo.
  - **`⌘Enter` ở câu cuối nhả caret (`setEditorCaret(null)`)** — lo ngại *"bỏ rơi bộ đệm gõ"* mà
    story nêu đã bị **BÁC bằng phép đo** *(bước ① của `confirmCurrentSegmentUnguarded` đã
    `flushEditorBeforeDiscreteWrite()` trước lượt IPC, nên tập chờ sạch theo cấu tạo tại điểm
    gọi)*. Nhưng một rủi ro **khác** lộ ra khi đọc `onSelectionChange` (`GridPanel.vue:875-882`):
    đường đó dựng trạng thái `caretSegmentId === null` **trong khi DOM focus vẫn trong ô** — hai
    nguồn sự thật nói ngược nhau, **không cổng nào canh** — và `onSelectionChange` đặt lại id ở
    lượt dịch caret kế tiếp, nên hiệu lực thị giác chỉ **tạm**. Nó mua một vế thị giác tạm thời
    bằng một trạng thái lệch thường trực.

  ⚠️ Ca *"ký câu cuối ⇒ thanh nói ra"* ở `tests/frontend/editorNavNotice.test.ts` **cố ý KHÔNG
  kiểm vạch lề** — vạch vẫn `primary`, và một ca khẳng định ngược lại sẽ là một lời hứa sai nằm
  trong chính bộ test.

  **Chủ của phần còn hở: chưa có — nó cần một AD.** Vế thị giác chỉ đóng trọn được bằng cách đổi
  ngữ nghĩa của `resolveSegmentRule` *(một bất biến có chữ ký)* hoặc bằng một khái niệm caret thứ
  hai *("caret logic" tách khỏi "DOM focus")*. Cả hai là **quyết định kiến trúc**, và luật kho
  viết bằng chữ: *"đổi một bất biến kiến trúc là một `AD` MỚI, không phải một dòng mã"*.
  ⇒ **Chủ: Ice phân định → Winston soạn AD.** Story 2.10 dừng đúng ở đây thay vì tự chọn.

  🔵 **HỒ SƠ BÀN GIAO ĐÃ SOẠN 2026-08-17** (lượt code review ba tầng của 2.10, Ice ký đường 1):
  `planning-artifacts/ad-brief-2026-08-17-vach-le-cau-cuoi-chuong.md` — đúng khuôn hai tiền lệ
  (`ad-brief-2026-08-16-xuat-xu-ban-dich.md` · `ad-brief-2026-08-17-mo-hinh-hoan-tac.md`). Bốn
  đường trình kèm cái giá, không kèm khuyến nghị.
  🔴 **Và hồ sơ đó mang một phép đo MỚI làm khoảng hở rộng hơn mục này mô tả:** cột nhãn trạng
  thái (`GridPanel.vue:244-251`) khoá theo **cùng** `SegmentRuleValue`, nên ở câu cuối Chương nó
  đọc ra **"đang sửa"**, không *"đã xác nhận"*. Cột ấy **là** kênh khả năng tiếp cận *(vạch mang
  `aria-hidden="true"`, `:1368`, và doc-comment `:240-242` khai cột ⑤ là lý do vạch được phép
  ẩn)*. ⇒ Vế còn hở **không** chỉ là *"bằng màu"* như dòng trên viết — nó chạm **AD-34 §2** và
  **NFR17**. Mệnh đề *"đóng bằng thông tin, không bằng màu"* vì thế đúng một nửa: thanh trạng thái
  nói được, nhưng **hàng** thì không, và trình đọc màn hình chỉ có hàng.

- 🔴 **Ba khoá lỗi `err.segment.*` vừa dựng KHÔNG có đường ra màn hình.** `confirm_segment` phía
  Rust trả đúng ba `IpcError` phân biệt được (`not_found` · `retired` · `nothing_to_confirm`), và
  `vi.json` có đủ ba câu tiếng Việt. Nhưng ở tầng giao diện: `editorConfirmError` được **export mà
  không component nào đọc**, và `ConfirmResult` bị **vứt** ở chỗ gọi duy nhất (`src/main.ts:229` —
  `void confirmCurrentSegment()`). ⇒ Hôm nay bấm `Mod+Enter` trên một câu **chưa dịch** thì **không
  gì xảy ra trên màn hình**: người dùng không biết mình vừa bị từ chối hay vừa ký thành công.
  ⚠️ **AC14 vẫn ĐẠT và món này KHÔNG phải lệch spec** — AC14 nói về **hợp đồng IPC** (*"trả một
  `IpcError` có `message_key` riêng"*), và Rust làm đúng. Đây là *"năng lực chưa dựng"*, đúng
  hạng mục mà luật **"năng lực chưa dựng ≠ lệch spec"** mô tả — nên nó ghi nợ, không sửa `epics.md`.
  ⚠️ Sau lượt rà này danh sách còn dài thêm một: `'still-dirty'` (Quyết định #8) cũng chỉ để lại
  một dòng `console.error`, cùng cảnh ngộ.
  ⚠️ Cùng đường hỏng và cùng chủ với vế *"báo lỗi ghi ra màn hình"* của lượt flush, đã ghi tại
  `src/panels/editorPanelState.ts:269-271`: cả hai là một lượt **thu hẹp UX-DR30** (*"không hộp
  thoại, không dấu chấm chưa lưu"*), và hợp đồng đó **có chủ là Ice**.
  **Chủ: Ice** *(hợp đồng UX-DR30 — cần chốt bề mặt báo lỗi của Editor **trước** khi story nào
  cài nó; hai món này phải đóng **cùng một** bề mặt, không hai bề mặt rời)*.

  → 🟡 **ĐÓNG MỘT NỬA 2026-08-15 (Story 2.5b, AC14 · Quyết định #8 đường (a), Ice ký).** Điều
  kiện chặn của mục này — *"chốt hợp đồng UX-DR30 TRƯỚC khi story nào cài"* — **đã đạt**: chữ ký
  của Ice cho Quyết định #8 **LÀ** hợp đồng đó, ở **phạm vi tối thiểu**.

  **Đã đóng:**
  - `src/main.ts` **đọc** `ConfirmResult` thay vì `void` — bốn giá trị không đi qua `IpcError`
    nay để lại một dòng chẩn đoán nêu đích danh *(**kêu**, không ném — hàm chạy từ một hợp âm
    bàn phím không bao giờ ném)*.
  - `GridPanel.vue` **đọc** `editorConfirmError` và hiện nhãn ở **cột nhãn trạng thái của chính
    hàng bị từ chối**, màu `error`. Không hộp thoại, không lớp nổi (UX-DR16).
  - 🔴 Hàng nào mang lỗi đọc `segment_id` từ **`params` của chính lỗi**, **không** từ *"câu đang
    có con trỏ"*: lượt xác nhận **dời con trỏ sang câu kế** khi nó thành công, nên gắn lỗi vào
    con trỏ sẽ dán nó lên **hàng sai** ngay ở ca thường nhất.
  - `'still-dirty'` đi **cùng đường**, không một bề mặt thứ hai — đúng điều kiện *"hai món này
    phải đóng cùng một bề mặt"* mà mục này viết ra.

  **CÒN HỞ, và ghi ra thay vì làm tròn lên:**
  - ⚠️ Vế *"báo lỗi ghi ra màn hình"* của lượt **flush** *(`editorPanelState.ts`)* **chưa** nối
    vào bề mặt này. Một lượt ghi trượt vẫn chỉ để lại `console.error` cộng một con số ngừng tăng
    ở `StatusBar`. Cùng hợp đồng, khác đường gọi.
  - ⚠️ Nhãn là **một chuỗi cố định** (`panel.grid.state_refused`), **không** phải câu tiếng Việt
    của chính `message_key`. Ba câu trong `vi.json` vẫn chưa tới màn hình — chỉ *sự kiện bị từ
    chối* tới. Thu hẹp có chủ ý: cột nhãn rộng 96px, và `tError()` ở đó sẽ vỡ bố cục.
  - ⚠️ **Không đường nghiệm thu tự động nào** canh vế hiển thị này. Không cổng nào đọc được
    *"một lượt từ chối có đổi pixel nào không"*, và ca e2e cho nó chưa dựng.

  **Chủ phần còn hở: Ice** *(mở rộng UX-DR30 quá phạm vi tối thiểu — hai gạch đầu dòng trên)*.

- 🔴 **Bộ e2e ĐỎ OAN khi chạy cả bộ — hai tệp xanh khi chạy riêng, đỏ khi chạy nối tiếp.**
  **Đo 2026-08-14** (macOS 15.6, sau lượt code review Story 2.5):

  | Cách chạy | `attribution-focus` | `editor-confirm-segment` | `editor-typing-flush` |
  |---|---|---|---|
  | `npm run test:e2e` (cả bộ, 6 tệp) | **ĐỎ** | **ĐỎ** | ĐỎ (2 ca) |
  | từng tệp một | XANH | XANH 2/2 | 1 xanh / 1 đỏ *(ca có chủ)* |

  ⚠️ **Không phải chạy song song.** `wdio.conf.mjs:221` khai `maxInstances: 1` và log lượt chạy
  cho thấy các spec đi nối tiếp. ⇒ Nguyên nhân là **rò trạng thái giữa các spec**: `attribution-focus`
  chết ngay ở `openWorkspaceWithWork` (`e2e/support/workspace.mjs:76`), tức fixture không mở nổi
  workspace vì thứ spec trước để lại — Tác phẩm còn mở, hoặc thư mục gốc Library tạm còn dữ liệu.

  🔴 **Vì sao đây là khuyết tật của CỔNG, không phải một phiền toái:** luật của kho ghi *"mỗi cổng
  phải có phép TỰ KIỂM chứng minh nó ĐỎ ĐƯỢC — **và không đỏ oan**"*. Một bộ đỏ oan dạy người chạy
  bỏ qua nó, và ngày nó đỏ thật thì không ai tin. Nó cũng làm mọi con số *"e2e N/N"* trong sổ story
  **không so sánh được**, vì kết quả phụ thuộc cách gọi.

  ⚠️ **Hệ quả ngay:** con số e2e ghi trong Dev Agent Record của các story trước đó được đo bằng
  **cách chạy nào** thì không tệp nào ghi lại. Đừng đối chiếu chúng với nhau cho tới khi món này đóng.

  **Chủ: Story 2.4** *(nó sở hữu hạ tầng đo và bàn đo; đừng dựng đường chẩn đoán thứ hai)*.

---

## Lượt Correct Course 2026-08-14 — bề mặt nhập lật sang lưới hai cột

*(Nguồn: `_bmad-output/planning-artifacts/sprint-change-proposal-2026-08-14.md`, Ice duyệt trọn gói.)*

- ⚠️ **FR20 (Sync Scrolling) đã RÚT, và cặp panel thứ ba là thứ bị BỎ chứ không phải không tồn tại.**

  FR20 đồng bộ **ba** panel: `Source`, `AI Translation`, `Editor`. Lưới đối chiếu nuốt **hai**
  trong ba — `Source` và `Editor` nay là **cùng một hàng**. Cặp còn lại, `lưới ↔ AI Translation`,
  **vẫn còn thật**: panel Đề xuất AI là một cột riêng ở **cả** Ⓑ-1 lẫn Ⓑ-2.

  🔴 **Rủi ro cụ thể, nêu trước khi Ice chốt và Ice vẫn chốt rút:** Epic 4 cho gọi AI *"từng
  segment **hoặc theo lô**"*. Ở chế độ **theo lô**, panel Đề xuất AI mang nội dung **cả Chương**
  và nhu cầu cuộn cùng lưới **quay lại** — lúc đó **không FR nào chứa nó**, vì FR20 đã rút và
  Story 2.12 đã xoá.

  ⇒ Epic 4 phải **quyết lại có cần hay không**, và nếu cần thì nó là một FR mới, không phải
  một lượt "khôi phục FR20".

  **Chủ: Epic 4.**

- ⚠️ **Chiều cao hàng khi bật Hán Việt *song song* ở cột hẹp của Ⓑ-2 — CHƯA ĐO.**

  Âm Hán Việt là chữ Latin, nên một câu Hán ~30 ký tự nở ra ~150 ký tự — dài gấp 4–5 lần dòng
  gốc. Ở cột nguyên văn của Ⓑ-2 *(ước ~330 px)*, một hàng bật *song song* có thể cao **6–7 dòng**,
  **ăn mất chính thứ Ⓑ-2 được chọn để có** *(nhiều câu trong tầm mắt nhất)*.

  🔴 **Con số này là ƯỚC LƯỢNG HÌNH HỌC, không phải một phép đo.** Nó là một **cận để cảnh báo**.
  Ai dựng thì **đo trên bản dựng thật và ghi số kèm ngày** — *"số đo không truy nguyên được thì
  không phải số đo"*.

  **Chủ: Story 2.5b.**

- ⚠️ **`editorGutter.ts` (273 dòng, 31 chỗ nhắc "làn") và `tests/frontend/editorGutterLanes.test.ts`
  (140 dòng) mất lý do tồn tại — KHÔNG xoá im lặng.**

  Làn ngang tồn tại **chỉ vì** nhiều câu chung một dòng thị giác nên các vạch phải xếp thành làn.
  Trong lưới, **một câu một vạch** ⇒ bài toán xếp làn biến mất.

  ⚠️ Nhưng `assignGutterLanes` mang một **phép đo thật** *(bản đầu O(n²) = 254–482 ms trên 9.850
  vạch ⇒ quét đường 4–8 ms)*. Gỡ nó là gỡ luôn bằng chứng đó. ⇒ Story 2.5b phải **nói rõ nó gỡ
  cái gì và vì sao**, và giữ lại phép đo trong sổ nếu còn có ai cần so sánh.

  **Chủ: Story 2.5b.**

  → ✅ **ĐÃ GỠ 2026-08-15, và BẰNG CHỨNG Ở LẠI ĐÂY** *(Quyết định #4 đường (a), Ice ký)*.
  Gỡ `src/panels/editorGutter.ts` *(273 dòng)* và `tests/frontend/editorGutterLanes.test.ts`.

  **Phép đo được chép nguyên vào sổ trước khi mã rời cây nguồn** — đây là toàn bộ lý do đường
  (a) được chọn thay vì `@deprecated`:

  | `assignGutterLanes`, 9.850 vạch | Ba lượt |
  |---|---|
  | bản đầu, **O(n²)** | **482,4 / 254,5 / 261,6 ms** |
  | quét đường *(tô màu đồ thị khoảng)* | **8,3 / 5,2 / 4,3 ms** |

  *(2026-08-14, Node 22.22.2, macOS 15.6.)*

  🔴 **Lý do gỡ, viết ra:** bài toán *"nhiều câu trên cùng một dòng ⇒ vạch chồng nhau"* **biến
  mất theo cấu trúc** khi một câu là một hàng. Mã giải một bài toán không còn tồn tại là mã sẽ
  bị story sau đọc nhầm thành *"chỗ này có một vấn đề chưa xong"*.
  ⚠️ Bằng chứng thuộc về **sổ**, không thuộc cây nguồn — nên nó ở đây, đầy đủ, thay vì trong một
  tệp `@deprecated` không ai chạy.
  ⚠️ `FILE_FLOOR`/`TS_FLOOR` đã **đếm lại** cùng lượt: quần thể `src/**` **không đổi** *(gỡ ba
  tệp, thêm ba)*, `.ts` đi từ 36 lên **37**. Sàn giữ nguyên và vẫn trong dải ~81–85 %.

- ⚠️ **Ngưỡng bố cục màn hình hẹp nay phải hiệu chỉnh cho HAI bố cục, không phải một.**

  UX-DR15 giữ **nguyên bốn con số** và **nguyên thứ tự hy sinh** — cả hai không đổi một chữ.
  Cái đổi là **số biến thể phải đo**: Ⓑ-1 *(lưới trên, hai panel dưới)* và Ⓑ-2 *(lưới trái toàn
  chiều cao)* co giãn khác nhau, nên một bộ số đúng cho một bố cục **không suy ra được** cho
  bố cục kia.

  **Chủ: Story 4.12** *(chủ cũ, chỉ mở rộng phạm vi)*.

- 🔵 **CHUYỂN CHỦ 2026-08-14 — hàng *"thư viện editor"* từ Story 2.4 sang Story 2.5b.**

  Lưới **đổi bài toán**: `contenteditable` trên **một ô mỗi hàng** khác hẳn `contenteditable`
  trên **một dòng văn liên tục**. Khuyết tật *"sập hố"* Ice báo 2026-08-14 *(xoá lui tới khi
  câu rỗng thì con trỏ thấp xuống và `Backspace` chết)* đến thẳng từ hình dạng cũ — nguyên nhân
  là span rỗng 0 px cộng `contenteditable` đặt trên **đúng một** span, nên `Backspace` ở offset 0
  không có chỗ nào để xoá lui vào.

  ⇒ **Kết luận cũ của Story 2.4 về thư viện editor KHÔNG được mặc nhiên giữ** — nó được đưa ra
  cho một hình dạng không còn tồn tại. 2.5b **đọc lại** món này cùng lượt dựng lưới, vì chính
  nó quyết định `contenteditable` đặt ở đâu.

  **Chủ: Story 2.5b** *(chuyển từ Story 2.4, Ice ký 2026-08-14)*.

- 🔴 **BỘ E2E KHÔNG TỰ KIỂM DANH TÍNH PHIÊN — một lượt chạy có thể đo NHẦM ỨNG DỤNG.**
  *(Tìm ra 2026-08-14, Story 2.5b Task 1.2, vòng chẩn đoán 3.)*

  Máy chủ WebDriver nhúng **bám cổng cố định 4445** (`e2e/wdio.conf.mjs`). Trên máy Ice cổng
  đó đang bị một tiến trình khác giữ — `gdrive-su`, PID 19811, đo bằng
  `lsof -nP -iTCP:4445 -sTCP:LISTEN`. ⇒ Phiên nối vào **webview của ứng dụng đó**, và **mọi
  phép đo vẫn chạy, vẫn trả về số**.

  Đo được: lượt chạy đầu của bàn đo 2.5b tiêm DOM thành công, đọc hình học thành công, rồi trả
  `document.activeElement = BUTTON.sidebar-folder-tree__chevron` — một lớp CSS mà `grep` toàn
  kho cho **0 kết quả**. Nếu bàn đo không tình cờ in `activeElement` ra thì lượt đó đã đẻ ra một
  bảng số trông hoàn toàn hợp lệ **về một ứng dụng khác**.

  ⚠️ Đây đúng hạng với hình dạng hỏng mà `onComplete` của `wdio.conf.mjs` đã dựng hàng rào
  (*"mọi ca vẫn xanh, vì một kho thật cũng là một kho mở được"*) — chỉ khác chỗ hỏng.

  **Đường ra đã đo:** cả hai phía đọc `TAURI_WEBDRIVER_PORT` — `getEmbeddedPort` của
  `@wdio/tauri-service` và crate `tauri-plugin-wdio-webdriver-1.3.0/src/lib.rs:24`. Chạy với
  `TAURI_WEBDRIVER_PORT=4467` cho phiên đúng (`location.href = http://localhost:1420/`,
  `#app` có mặt). ⇒ **Không** cần giết tiến trình của người dùng.

  **Còn hở:** bộ e2e thường trực vẫn **không** có phép kiểm nào. Bản vá đúng hình dạng là một
  `before` hook khẳng định `location.href` + sự có mặt của `#app` **trước** ca đầu tiên, cộng
  một lượt chọn cổng trống thay vì hằng số. 2.5b **không** nhận việc này: nó nằm ngoài phạm vi
  story và chạm vào hạ tầng của cả bộ.

  **Chủ: Story 1.22** *(bộ chạy e2e trong webview thật — cùng chủ với ba giới hạn đã ghi ở
  `wdio.conf.mjs`)*.

- 🔴 **`Backspace` ở offset 0 KHÔNG phát `beforeinput` trên WebKit — tiền đề của Story 2.9 đã
  LẬT.** *(Đo 2026-08-14, Story 2.5b Task 1.2/1.3.)*

  Quyết định #3 của Story 2.5b viết ra bằng chữ: *"`Backspace` ở offset 0 sinh một `beforeinput`
  `deleteContentBackward` **bắt được** ⇒ Story 2.9 có tiền đề"*. Phép đo bác vế đó trên WebKit.

  | Engine | đầu một ô **CÓ CHỮ** | một ô **đã rỗng** |
  |---|---|---|
  | WKWebView 605.1.15 *(`execCommand('delete')`)* | **0** `beforeinput` | **0** `beforeinput` |
  | Playwright-WebKit *(phím **vật lý**)* | **0** `beforeinput` | **0** `beforeinput` |
  | Blink *(phím **vật lý**)* | `deleteContentBackward`, **huỷ được** | `deleteContentBackward`, **huỷ được** |

  Caret đã xác nhận đúng chỗ ở cả ba (`type = "Caret"`, neo nằm trong ô), nên đây **không** phải
  một lượt đo hỏng: WebKit đơn giản **không phát sự kiện cho một lượt xoá không có gì để xoá**.

  ⇒ Story 2.9 *(`Backspace` đầu ô = gộp với câu trên, UX-DR32)* **không** cài được ở
  `beforeinput` trên macOS. Đường còn lại là `keydown`, và nó **bắt buộc** mang chốt
  `event.isComposing` **trước mọi nhánh khác** — cùng dòng và cùng lý do `EditorPanel.vue:841`
  (*"một lượt commit composition của bộ gõ tiếng Việt phát `keydown` mang code vật lý; ăn nó là
  ăn mất chữ"*).

  ⚠️ 2.5b **không** bị chặn bởi món này — nó chỉ được giao dựng **tiền đề cấu trúc** *(mỗi ô là
  một editing host riêng)*, và tiền đề đó đứng. Cái lật là **đường bắt sự kiện**, không phải
  hình dạng DOM.

  **Chủ: Story 2.9.**

  → ✅ **ĐÃ ĐÓNG 2026-08-17 (Story 2.9).** Nhánh cắm ở `keydown` (`GridPanel.vue::onEditKeydown`),
  **sau** chốt `event.isComposing` và không chạm một dòng nào của nó. Tiền đề đo lại từ nguồn
  trên cây hôm nay, **có đối chứng dương** — thứ bảng ở trên thiếu: `2-9-ban-do/` §Ⓓ chạy
  `execCommand('delete')` hai lượt trên **cùng một ô**, caret ở offset 0 và caret ở offset 3.
  Offset 0 cho **0** `beforeinput` / **0** `input` / `textContent` không đổi; offset 3 cho
  `["deleteContentBackward"]` ở cả hai sự kiện và `"bốn năm sáu"` → `"bố năm sáu"`.
  ⇒ Thước hoạt động, và con số ở offset 0 là mệnh đề về **engine**, không về bàn đo.
  🔴 **Một chi tiết mới, đáng ghi riêng:** `execCommand('delete')` trả **`true`** trong khi
  KHÔNG làm gì. Giá trị trả về của nó nói *"lệnh được nhận"*, không *"lệnh có tác dụng"* — ai
  đọc nó thành *"đã xoá"* sẽ có một lượt thành công không có thật.

- 🔴 **CÚ BẤM ĐẦU TIÊN VÀO MỘT PANEL GIẾT CARET VỪA ĐẶT — hợp đồng tiêu điểm AD-34 va vào
  hợp đồng vùng gõ.** *(Đo 2026-08-15, Story 2.5b Task 12.2, trong WKWebView 605.1.15 thật.)*

  `WorkspaceDock.vue:591-611` nghe `onDidActivePanelChange` và, với `origin === 'user'`, gọi
  `enterFocus(id)`. `focus.ts::enter()` chạy `el.focus()` **vô điều kiện** trên gốc panel —
  **kể cả khi tiêu điểm ĐÃ nằm trong panel đó**. Cú bấm **đầu tiên** vào lưới kích hoạt panel,
  nên lượt dời ấy chạy **sau** handler `mouseup` và **sau** cả hai lượt vá của
  `ensureCaretNextFrame`.

  | Cùng một ô, chuột thật | `document.activeElement` | `getSelection().type` |
  |---|---|---|
  | cú bấm **thứ nhất** | `SECTION.panel.focused` | **`"None"`**, `rangeCount 0` |
  | cú bấm **thứ hai** | `DIV` *(chính ô)* | **`"Caret"`** |

  ⇒ Khuyết tật gói gọn ở **cú bấm đầu tiên vào panel**; mọi cú bấm sau đều ăn. Người dùng đọc
  nó thành *"phải bấm hai lần mới gõ được"*.

  🔴 **Bốn lượt vá đã thử và bị bác BẰNG PHÉP ĐO**, ghi ra để không ai đi lại: ①
  `contenteditable` trần *(engine không focus)* → ② `cell.focus()` trong `mouseup` *(gốc panel
  vẫn giành)* → ③ `requestAnimationFrame` → ④ một lượt `setTimeout(0)` nữa. Cả bốn đều chạy
  **trước** lượt `enterFocus`.

  **Đường sửa là MỘT ĐIỀU KIỆN, và có hai chỗ đặt được nó — Ice chốt:**
  - **(A)** `focus.ts::enter()` bỏ qua khi `el.contains(document.activeElement)`. Nguyên tắc:
    AD-34 §2 nói *"CHUYỂN panel phải dời focus DOM tường minh"*, và khi tiêu điểm đã ở trong
    panel thì **không có lượt chuyển nào**. Sửa một chỗ, phủ **cả sáu** điểm vào focus.
  - **(B)** Chỗ gọi ở `WorkspaceDock` tự kiểm cùng điều kiện trước khi gọi `enterFocus`. Bán
    kính hẹp hơn, nhưng cùng một phép kiểm viết ở một tầng trên — và nó **không** đóng ca
    tương tự cho panel khác.

  ⚠️ Cả hai chạm hợp đồng tiêu điểm, nên **không** được vá bằng một lượt thứ năm chồng lên
  trong `GridPanel.vue`. Story 2.3 đã trả giá một lần cho đúng lớp lỗi này *(chẩn đoán "AD-34
  giành tiêu điểm" khi **không ai giành cả**)*; lần này phép đo nói **có**, và nó nói đích
  danh dòng nào.

  ⚠️ Ca nghiệm thu **đang ĐỎ có chủ**: `e2e/specs/grid-empty-cell.e2e.mjs`. Đừng nới mệnh đề
  của nó cho xanh — nó đang nói đúng sự thật.

  ✅ **ĐÃ ĐÓNG 2026-08-15 — Ice ký đường (A).** `focus.ts::enter()` nay bỏ qua lượt `focus()`
  khi `el.contains(document.activeElement)`, tức khi tiêu điểm **đã** ở trong owner đó. AD-34 §2
  **không sửa một chữ**: mệnh đề của nó nói về một lượt **CHUYỂN**, và khi tiêu điểm đã ở trong
  panel thì không có lượt chuyển nào.

  Nghiệm thu: `e2e/specs/grid-empty-cell.e2e.mjs` **XANH** trên WKWebView thật, và **cả bộ e2e
  7/7 xanh** — tức lượt sửa một hợp đồng dùng chung không làm đỏ một điểm vào focus nào khác.

  ⚠️ **Một hệ quả ĐO ĐƯỢC, ghi ra vì nó lật một giả thuyết hợp lý:** lượt vá thứ hai của
  `GridPanel.vue::ensureCaretNextFrame` *(`setTimeout(…, 0)`)* **KHÔNG** trở thành mã chết sau
  bản vá này. Gỡ nó ⇒ ca e2e **đỏ trở lại**; trả lại ⇒ **xanh**. ⇒ Còn **một** nguồn thu vùng
  chọn nữa ngoài `enterFocus`, chạy ngoài vòng `requestAnimationFrame`, và nó **chưa được đặt
  tên**. Ứng viên chưa loại trừ: lượt xử lý cú bấm của chính WKWebView.

  **Chủ: Ice** *(quyết định đã ký)* → **đóng**.

- ⚠️ **FIXTURE `workspace.mjs` KHÔNG reset state của panel — spec sau đọc Tác phẩm của spec
  trước.** *(Đo 2026-08-15, Story 2.5b Task 12.2.)*

  `e2e/specs/grid-empty-cell.e2e.mjs` XANH khi chạy một mình *(ba lượt)* và ĐỎ khi chạy cả bộ
  *(hai lượt liên tiếp)*: lưới hiện một Chương **đã có bản dịch** thay vì Tác phẩm fixture vừa
  tạo — `soCauDich = 1`, `soORong = 0`.

  Cơ chế: mọi spec dùng chung **một** `$APPDATA` tạm cho cả lượt chạy (`onPrepare`), nên
  `app_config` — gồm **chế độ đang mở** — sống sót qua từng phiên app. App khởi động **thẳng
  vào `workspace`** với Tác phẩm spec trước để lại; lưới mount và nạp segment của Tác phẩm đó.
  Rồi `create_work_from_text` của fixture đi **đường IPC**, đường **không** gọi
  `resetEditorPanel()` — chỗ gọi duy nhất là `libraryImport.ts::finishSubmit`, tức đường **giao
  diện**, thứ fixture cố ý không đi (`workspace.mjs` §Lựa chọn ①).

  ⇒ Vá tạm **trong spec đó**: nạp lại webview sau lượt tạo Tác phẩm. Vá đúng chỗ là ở
  `workspace.mjs`, nhưng đổi một fixture **dùng chung** để chữa một ca là cách rẻ nhất làm đỏ
  sáu ca đang xanh — nên nó không được làm trong story này.

  ⚠️ Đây là một khuyết tật của **bàn đo**, không của sản phẩm: trên đường người dùng thật,
  `finishSubmit` đã reset. Nhưng nó là một **cửa xanh giả**: một spec đọc nhầm Tác phẩm vẫn
  chạy trọn và vẫn khẳng định được nhiều mệnh đề.

  **Chủ: Story 1.22** *(bộ chạy e2e — cùng chủ với ba giới hạn đã ghi ở `wdio.conf.mjs`)*.

- ⚠️ **Command id nằm CỨNG trong spec e2e, và không cổng nào canh mối nối đó.**
  *(Tìm ra 2026-08-15, Story 2.5b.)*

  `shortcuts-capture-mouse.e2e.mjs` ghi `TARGET_COMMAND = 'layout.toggle_source'`. Story 2.5b
  đổi `PANEL_SUFFIXES` *(bốn → ba)* nên command đó **thôi tồn tại**, và lượt đổi tên đi qua
  sạch **chín cổng, `npm run build`, và cả vitest** — `check:commands` đọc `src/**`, không đọc
  `e2e/**`. Nó chỉ lộ ra ở lượt chạy e2e **bằng tay**, dưới dạng một timeout 10 giây nói *"phần
  tử không hiện"*: một câu đúng về triệu chứng và **câm về nguyên nhân**.

  ⇒ Đã sửa tại chỗ. Còn hở: **cơ chế**. Một cổng đối chiếu id trong `e2e/**` với bộ đăng ký
  thật sẽ đóng nó, và nó rẻ — `check:commands` đã nạp bộ đăng ký sẵn.

  **Chủ: Story 1.22.**

- 🟡 **TASK 7.3 ĐÃ ĐO — chiều cao hàng khi bật Hán Việt SONG SONG ở Ⓑ-2. Số XẤU HƠN ước
  lượng, và mối lo của `epics.md:2329` được XÁC NHẬN.** *(2026-08-15, WKWebView 605.1.15,
  macOS 15.6, bản dựng thật, `2-5b-ban-do/do-hang-va-hieu-nang.e2e.mjs`.)*

  Nợ gốc `:2863-2873` ước *"cột ~330 px ⇒ một hàng có thể cao **6–7 dòng**"*. Đo trên một câu
  tiếng Trung dài ở bố cục **Ⓑ-2**, giãn dòng `source-cjk` **33,83 px**:

  | Kiểu xem | Chiều cao hàng | Số dòng | Cột nguyên văn |
  |---|---|---|---|
  | Nguyên văn | **137 px** | **4,05** | 238,5 px |
  | Hán Việt — *chuyển đổi* | **228 px** | **6,74** | 238,5 px |
  | Hán Việt — **song song** | **388 px** | **11,47** | 238,5 px |

  ⇒ **Cả hai vế của ước lượng đều thấp hơn thực tế:** 388 px *(không ~330)* và **11,5 dòng**
  *(không 6–7)*. Lý do đọc được từ chính bảng: cột thật của Ⓑ-2 chỉ **238,5 px**, hẹp hơn con
  số ước ~330 px, nên chữ xuống dòng nhiều hơn — và `<ruby>` nhân đôi chiều cao mỗi dòng.
  *(28 `<ruby>` cho một câu, tức một `<ruby>` mỗi TỪ, đúng Quyết định #2a của Story 1.18b.)*

  🔴 **Và một cái giá thứ hai mà nợ gốc KHÔNG nêu:** `subgrid` giữ hàng thẳng, nên **ô bản
  dịch cũng cao 388 px** — đo được `cao_o_dich_px = 388`. Một câu **chưa dịch** vì thế hiện ra
  thành một ô rỗng cao gần **12 dòng**. Đây là hệ quả trực tiếp của Quyết định #1(b) và nó
  **không** sửa được bằng CSS của riêng cột: hàng thẳng là thứ AC2 đòi.

  ⚠️ **Vế Blink CHƯA ĐO** — Task 7.3 đòi *"cả hai engine"*, nhưng Blink chỉ tới được qua
  **WebView2 trên Windows**, và `project-context.md` ghi *"nửa Windows hôm nay KHÔNG có đường
  nghiệm thu tại chỗ"*. Khoảng mù **có tên**, không một mục đã đóng.

  ⚠️ Story 2.5b **không** tự chấm mục này đạt: nó đo xong và **giao lại**. Ai quyết cái giá
  này là một quyết định UX *(giữ nguyên · giới hạn số dòng của `<rt>` · chỉ mở song song ở
  Ⓑ-1)* — cả ba đều chưa được nêu ra bao giờ.

  **Chủ: Ice** *(quyết định UX)*, kèm **Story 4.12** cho vế ngưỡng bố cục.

- 🔴 **TASK 8 ĐÃ ĐO — SỐ GIAO CHO STORY 2.4. MỘT ĐƯỜNG VƯỢT TRẦN NFR2 15 LẦN.**
  *(2026-08-15, WKWebView 605.1.15, macOS 15.6, bản dựng thật.)*

  | Phép đo | 2.000 câu | **9.850 câu** *(mốc cũ)* |
  |---|---|---|
  | node DOM trong lưới | 10.005 | **49.256** *(5 node/câu — đúng năm cột)* |
  | một lượt `selectionchange` + 2 frame | 12 / 34 / 34 ms | **24 / 33 / 33 ms** |
  | một lượt **DỜI CON TRỎ** | 226 / 173 / 195 / 189 / 161 ms | 🔴 **770 / 706 / 767 ms** |

  ⇒ **Đường dời con trỏ vượt trần 50 ms/frame của NFR2 khoảng 15 lần** ở 9.850 câu, và ~4 lần
  ở 2.000 câu. Đây là đường **thường nhất** của tính năng: mỗi lần người dùng bấm sang câu khác.

  ⚠️ **Mốc cũ mất hiệu lực THEO CẤU TRÚC, không bị đóng** — `:2113-2129` đo *"dựng 9.850
  `<span>`"* (300,1 ms Blink · 1.308,0 ms WebKit). Lưới không dựng `<span>` nào; nó dựng
  **49.256 node** trong năm cột `subgrid`. Hai con số **không so được với nhau**, và ghi chúng
  cạnh nhau như một lượt "cải thiện" là nói dối.

  ⚠️ `:2198-2207` *(`:data-caret` dựng lại **toàn** danh sách mỗi `selectionchange`)* thì
  **vẫn còn hiệu lực về cơ chế**: lưới vẫn tính lại `ruleById` trên **toàn** danh sách mỗi lượt
  dời con trỏ, cộng một lượt Vue vá lớp trên `N × 2` ô. Hình dạng đổi, điểm nghẽn thì không.

  🔴 **KHÔNG tối ưu mù** *(Quyết định #7, Ice ký)*: số này được **báo**, không được vá vội. Ba
  đường đã thấy — ảo hoá hàng *(spine Giai đoạn 3)* · tính `rule` tại chỗ thay vì một `Map`
  toàn danh sách · tách lớp `editing` khỏi lượt tính lại — chưa đường nào được **đo**.

  ⚠️ Một khuyết tật của **bàn đo** đã bắt và ghi ra: lượt đo đầu ở 9.850 hàng cho **26.927 ms**
  cho một lượt `selectionchange`, rồi 33 ms ở hai lượt kế. Con số đó **không** phải chi phí
  thao tác — nó là **lượt bố cục lần đầu** của 49.256 node còn đang chạy, vì `waitForExist` trả
  về ngay khi ô ĐẦU TIÊN có mặt. Bản sửa chờ nhịp frame rẻ trước khi bấm đồng hồ.

  **Chủ: Story 2.4** *(sở hữu bộ đo NFR2/NFR18 — 2.5b **giao số**, không tự chấm, B9)*.

- 🔵 **NGUYÊN VĂN của lượt đỏ chập chờn `attribution-focus` — ĐÃ BẮT ĐƯỢC 2026-08-15.**

  `wdio.conf.mjs:68-78` ghi món này từ 2026-08-12 và nói thẳng nó còn hở: *"Lần đỏ ②:
  `attribution-focus` — **CHƯA chẩn đoán**, nguyên văn lỗi không kịp bắt."* Lượt nghiệm thu
  cuối của Story 2.5b bắt được nó:

  > Vào `workspace` rồi mà không thấy `[data-attribution-open]` sau 30 giây.

  ⇒ Lượt đỏ nằm ở **fixture**, không ở một khẳng định nào về sản phẩm: `openWorkspaceWithWork`
  hết giờ chờ dải chip nguồn của Panel Lookup render. Nó **không** phải một hồi quy tiêu điểm,
  và **không** phải ca `Escape` mà spec được dựng để canh.

  **Cỡ mẫu của ngày 2026-08-15** *(Story 2.5b, cùng máy, cùng nhị phân)*: **bốn** lượt chạy cả
  bộ — spec này **3 xanh · 1 đỏ**; chạy một mình ngay sau lượt đỏ: **xanh**.

  ⚠️ **Vẫn CHƯA có nguyên nhân**, chỉ có triệu chứng. Ứng viên chưa loại trừ: dải chip chỉ
  render khi `dictSources.length > 0`, tức sau **một lượt IPC nữa** sau khi đổi chế độ — một
  máy đang bận *(lượt chạy cả bộ vừa dựng xong một Chương 9.850 câu ở spec trước)* có thể vượt
  trần 30 giây. Đó là một giả thuyết **có thể đo**: nới trần rồi đếm lại.

  ⚠️ Đừng đọc mục này thành *"đã đóng"*. Thứ đóng được hôm nay là **vế nguyên văn**, đúng thứ
  bản ghi cũ nói là còn thiếu.

  **Chủ: Story 1.22.**

---

## Deferred from: code review of 2-5b-luoi-hai-cot-doi-chieu (2026-08-15)

> Lượt rà ba tầng trên `f990dd5..HEAD -- src/ scripts/`. **13 phát hiện, 13 bản vá, 0 hoãn.**
> Mục dưới đây **không** phải một phát hiện bị hoãn — nó là khoảng **CHƯA NGHIỆM THU ĐƯỢC** của
> hai bản vá **đã va**, ghi ra vì luật đo của kho cấm đánh dấu đạt bằng suy luận.

- 🟡 **HAI bản vá của lượt rà 2026-08-15 đi qua sạch mười một cổng mà KHÔNG cổng nào thật sự
  nhìn thấy chúng.** Cả hai sống ở **tầng vẽ** — tầng mà chính Story 2.5b tự khai là *"ít cổng
  canh nhất trong kho, không một test mount component nào tồn tại"*.

  **① Guard `caretTarget` trong `GridPanel.vue::ensureCaretNextFrame`.** Nó chặn lượt vá của một
  ô cũ kéo tiêu điểm về sau khi người dùng đã bấm sang ô khác. Mệnh đề *"lượt vá vẫn đặt được
  caret ở ô rỗng"* thuộc **e2e trong WKWebView thật** (`e2e/specs/grid-empty-cell.e2e.mjs`) —
  đúng bộ mà `§Debug Log` ghi là đã dùng để **bác** một giả thuyết *"mã chết"* bằng hai lượt
  chạy liên tiếp. Cổng tĩnh và vitest **không** thay được nó: `happy-dom` không phải WebKit, và
  mệnh đề ở đây là về **thời điểm** giữa `requestAnimationFrame` và `setTimeout(0)`.
  ⚠️ Rủi ro hồi quy có thật, không lý thuyết: điều kiện thoát nay hỏi **ba** vế thay vì hai. Một
  vế thừa ở đó làm lượt vá im **đúng ca nó tồn tại để chữa**, và biểu hiện là *"bấm vào ô rỗng
  không có caret"* — đúng triệu chứng mà LUẬT DỪNG của Task 12.2 đã kích hoạt một lần.

  **② Đường ra thanh trạng thái cho ba `ConfirmResult`** (`StatusBar.vue` · `editorConfirmNotice`
  · ba khoá `panel.grid.confirm_*`). `check:i18n` xác nhận **khoá tồn tại và đúng hình dạng**;
  `check:tokens` xác nhận **màu và cỡ chữ đến từ token**. Không cổng nào trả lời được câu hỏi
  thật: *"bấm `⌘Enter` khi chưa đặt con trỏ thì câu ĐÓ có hiện lên không, và nó có tắt khi gõ
  tiếp không"*. Ba đường vào (`'no-caret'` · `'flush-failed'` · `'still-dirty'`) và hai đường dọn
  (`noteEditorEdit` · `resetEditorPanel`) hôm nay **chưa có một phép kiểm nào**.

  🔴 **Đây là chỗ rẻ nhất để đóng nợ này, ghi ra để lần sau không phải tìm lại:** vế ② nghiệm thu
  được bằng **vitest** *(bộ chạy frontend đã có từ 2026-08-12)* — `editorConfirmNotice` là state
  module-level thuần, và cả năm đường vào/ra đều gọi được **không cần webview**. Chỉ vế ① mới
  buộc phải đi qua e2e.

  → 🟡 **ĐÓNG MỘT NỬA 2026-08-15 (cùng lượt code review, Ice chốt đường 1).**

  **Vế ② ĐÃ ĐÓNG ✅** — `tests/frontend/statusBar.test.ts` §nhóm ⑤, **sáu** ca mới: ba ca cho ba
  giá trị *(`'no-caret'` · `'flush-failed'` · `'still-dirty'`)*, một ca cho luật *"câu báo THAY
  mốc «Đã lưu», hai câu không bao giờ cùng lúc"*, một ca cho *"gõ tiếp ⇒ tắt"* **dọn bằng sự
  kiện, không bằng hẹn giờ**, và một ca cho *"lượt thành công không để lại câu nào"*. Bộ vitest
  **83 → 89**.
  🔴 **Phép TỰ KIỂM đã chạy, không phải một lời hứa:** đặt `confirmNotice.value = null` vô điều
  kiện ⇒ **5/6 ca ĐỎ**; ca thứ sáu ở lại xanh vì nó khẳng định đúng `null` — tức bộ này **đỏ
  được** và **không đỏ oan**, cả hai vế của luật *"một cổng chưa bao giờ đỏ là một cổng chưa ai
  biết nó có chạy không"*.
  🔵 Nhân đây, một mệnh đề của story hết đúng: `§Cổng nào sẽ nhìn story này` viết *"không một
  test mount component nào tồn tại"*. Sai từ Story 2.3 — `statusBar.test.ts` đã `mount(StatusBar)`
  bằng `@vue/test-utils` từ 2026-08-12. Vế ② đóng được rẻ **chính vì** nhà đã dựng sẵn.

  **Vế ① CÒN HỞ** — guard `caretTarget` vẫn chỉ nghiệm thu được bằng `npm run test:e2e` chạy tay
  *(`e2e/specs/grid-empty-cell.e2e.mjs`)*. Không hạ mức mục này xuống ✅ khi mới đóng một nửa.
  **Chủ: Ice.**

  🔴 **ĐÃ THỬ CHẠY 2026-08-15, 07:17Z — và bộ e2e KHÔNG CHẠY ĐƯỢC. Đây là lỗi HẠ TẦNG, không một
  ca đỏ.** Ghi ra vì một lượt thử thất bại là dữ kiện, và vì thông báo lỗi to nhất ở đó **dẫn sai
  đường**.

  **Triệu chứng:** cả **7/7** spec đỏ tại cùng một điểm — `openWorkspaceWithWork` không dựng được
  fixture. Chính bàn đo in ra câu phân biệt của nó: *"Fixture không tạo được Tác phẩm …: không có
  cầu IPC. Đây là lỗi HẠ TẦNG của bàn đo, không một hồi quy giao diện."* Không một assertion nào
  của `grid-empty-cell.e2e.mjs` được chạy tới.

  ⚠️ **BẪY, và nó tốn thời gian nếu không ghi ra:** service in đậm và lặp bảy lần
  `❌ Tauri Driver: tauri-driver not found. Install it with: cargo install tauri-driver`.
  **Đừng cài.** `wdio.conf.mjs:351-355` khai chính xác lý do: `tauri-driver` chính thức vẫn chỉ
  Windows + Linux *(`tauri-apps/tauri#7068`, mở từ 2023)*, nên trên macOS
  `driverProvider: 'embedded'` là đường **DUY NHẤT** chạy được. Dòng đó là chẩn đoán chung của
  service, sai nền tảng, và nó **át** dòng thật.

  **Dòng thật:** `WARN tauri-service:launcher: Embedded WebDriver on port 4445 (instance: 0) is
  unreachable — restarting...` — lặp sau mỗi worker.

  **Đã loại trừ:** `~/.cargo/bin` không có `tauri-driver` *(đúng như thiết kế — chỉ có
  `cargo-tauri`)*; nhị phân `src-tauri/target/debug/auratranslate` **có** và vừa dựng lại cùng
  lượt (`--features wdio`, 49,6 MB, 14:16 giờ máy); sau lượt chạy **không** tiến trình app/driver
  nào còn sót; cổng 1420 và 4445 đều **trống**.

  **Manh mối chưa lần hết:** đầu lượt chạy có dòng
  `http://localhost:1420 đã có người phục vụ — dùng lại, KHÔNG dựng thêm` — tức conf **tái dùng**
  một máy chủ 1420 có sẵn thay vì tự dựng, và cổng đó nay trống. Một máy chủ 1420 đã chết giữa
  chừng giải thích được cả chuỗi *(webview nạp hụt ⇒ embedded WebDriver không bao giờ lên ⇒ không
  cầu IPC)*, nhưng **chưa đo**, nên nó là một giả thuyết, không một kết luận.

  🔴 **MÂU THUẪN PHẢI GIẢI, không được để trôi:** `§Change Log` của story ghi **cùng ngày**
  *"e2e 3/4 lượt cả bộ 7/7"*. Hai mệnh đề đó không thể cùng đúng trừ khi môi trường đổi trong
  vài giờ. ⇒ Hoặc bộ e2e phụ thuộc một điều kiện môi trường **chưa được khai ở đâu cả**
  *(không có `e2e/README.md`; `grep` toàn kho chỉ thấy `tauri-driver` ở đúng một dòng chú thích)*,
  hoặc con số 7/7 cần xem lại. Đây là đường nghiệm thu mà story dựa vào **nặng nhất** cho các
  mệnh đề khó nhất của nó — một đường nghiệm thu không dựng lại được là một đường nghiệm thu chưa
  ai kiểm chứng.

  **Chủ: Ice** — và món này nay **to hơn** vế ①: nó chặn mọi story sau có động tới WKWebView.

  → ✅ **NGUYÊN NHÂN TÌM RA VÀ ĐO ĐƯỢC 2026-08-15. Giả thuyết cổng 1420 ĐÚNG.**

  Chạy lại **một** spec với cổng 1420 **trống**: conf tự dựng Vite, app lên, phiên WebKit
  **605.1.15** thiết lập được *(`Session ID` có thật)*, embedded WebDriver trên 4445 chạy. ⇒ Cả
  7/7 spec đỏ lúc 07:17Z là do nhánh *"đã có người phục vụ — dùng lại"* ở `onPrepare` tin một
  máy chủ 1420 **hấp hối**.

  🔴 **Khuyết tật thật nằm ở `devServerIsUp()`, và nó đã được TIÊN ĐOÁN ngay trong tệp:**
  doc-comment ở `wdio.conf.mjs:186` viết *"không có vế tắt thì mỗi lượt e2e để lại một tiến
  trình giữ cổng 1420, và lượt sau thấy cổng bận rồi tin rằng có người phục vụ"*. Vị từ đó hỏi
  **một** câu — `fetch(DEV_URL)` có `res.ok` không — rồi `return` mà không dựng gì. Một Vite
  đang chết vẫn trả `200` cho `/` trong khi module graph của nó đã hỏng ⇒ webview nạp một trang
  không chạy ⇒ không cầu IPC ⇒ **mọi** spec đỏ với một lý do không liên quan.
  ⚠️ **Chủ:** một story hạ tầng e2e. Vá hướng nào cũng được, nhưng phải **đo** chứ đừng đoán:
  ứng viên là hỏi một tài nguyên **thật** của app *(không phải `/`)*, hoặc bỏ hẳn nhánh tái dùng
  và đòi cổng trống — nhánh đó tồn tại cho ca *"Ice đang mở sẵn `npm run tauri dev`"*, nên bỏ nó
  là một đánh đổi có người trả giá.
  → ✅ **ĐÃ ĐÓNG 2026-08-18 (Story 2.12 · AC1).** Vá theo đúng yêu cầu *"phải ĐO chứ đừng đoán"*,
  và **hai** giả thuyết rẻ đã bị chính phép đo BÁC trước khi có bản vá:
  ① *"kiểm `/` kỹ hơn"* — vô vọng: `/` là `index.html` phục vụ **tĩnh**, giống nhau **tới từng
  byte** giữa Vite lành và Vite hấp hối; ② *"nạp module entry là đủ"* — **SAI**: Vite biến đổi
  module **lười, theo từng yêu cầu**, nên `/src/main.ts` vẫn 200 sạch (88.494 B) trong khi
  `/src/App.vue` đã trả **500**.
  ⇒ Bản vá đi **theo graph**: `e2e/support/devServerHealth.mjs` duyệt BFS từ entry qua mọi đường
  `/src/…`. Đối chứng dương là một lượt phá **thật** ở `src/App.vue`, chạy ba lượt: lành → **xanh
  (58 module, 190 ms)** · hấp hối → **đỏ, nêu đích danh** `/src/App.vue` HTTP 500 · lành trở lại →
  xanh. Giá: **270 ms** (Vite ấm) / **4.129 ms** (nguội) — và khoản nguội là chi phí **dời chỗ**,
  không chi phí thêm: trình duyệt trả đúng khoản ấy ở lượt nạp đầu, nên lượt duyệt làm ấm sẵn Vite.
  ⚠️ **Nhánh tái dùng KHÔNG bị bỏ** *(đường đánh đổi mà mục này cảnh báo có người trả giá)* — nó ở
  lại, và nay lượt tái dùng **cũng** phải qua phép kiểm graph: một Vite ai đó để mở từ trước đúng là
  chỗ graph vỡ mà không ai vừa nhìn thấy nó lành.
  🔴 **GIỚI HẠN THẬT, mở một món nợ mới có chủ:** phép duyệt chỉ chạm những gì entry **với tới**.
  Một tệp `src/**` không module nào import sẽ không được kiểm. Đó đúng theo cấu tạo *(một module
  không ai import cũng không làm app chết)*, và mệnh đề *"mọi tệp `src/**` phải lành"* có chủ
  riêng: `npm run build` (`vue-tsc`).

  ⚠️ **BẪY THỨ HAI, và nó cũng dẫn sai đường:** khi fixture trượt, spec khuyên *"kiểm
  `src-tauri/target/debug/dict/*.db` có mặt chưa"*. Đã kiểm: **cả bốn tệp có đủ**
  *(`dict-core` 195 MB · `vietphrase` 160 MB · `tran-van-chanh` 10,8 MB · `thieu-chuu` 5,8 MB)*.
  Gợi ý đó đúng cho một ca khác và sai cho ca này. ⇒ Trong một ngày, **hai** thông báo lỗi to và
  tự tin đã trỏ nhầm chỗ *(`cargo install tauri-driver`, và `dict/*.db`)*. Bài học rẻ: ở bộ e2e
  này, **đo trước khi tin thông báo lỗi** — kể cả thông báo do chính kho viết ra.

  → ✅ **VẾ ① ĐÓNG 2026-08-15 — rủi ro hồi quy của guard `caretTarget` đã đo, không suy luận.**

  `e2e/specs/grid-empty-cell.e2e.mjs` trên cây **đã vá**, WKWebView **605.1.15**, tám lượt:
  **5 ĐẠT / 3 đỏ**. 🔴 **Không một lượt đỏ nào rơi vào phép khẳng định caret.** Hai lượt đỏ có
  chi tiết đều chặn ở **đường khởi động/fixture** — một ở `[data-attribution-open]` sau 30 s,
  một ở `Error: Timeout` của mocha sau `core.invoke not available` — tức **trước** khi lưới được
  dựng, và guard chỉ chạy trong `onCellMouseUp` trên một ô của lưới.
  ⚠️ **Ghi đúng mức, đừng đọc quá:** năm lượt đạt chứng minh guard **không phá** ca nó bảo vệ.
  Chúng **không** chứng minh guard *chữa* được cuộc đua — cuộc đua cần **hai cú bấm nhanh vào
  hai ô khác nhau trong một khung hình**, và không spec nào làm thế. Vế đó vẫn **không có phép
  kiểm**, và nó ở mức **thấp** *(biểu hiện là một nháy tiêu điểm ~1 frame, tự chữa ở frame sau)*.
  ⚠️ Baseline chỉ chạy **1** lượt *(đạt)*, nên bộ số này **không** đủ để nói lượt vá có làm tăng
  hay giảm độ chập chờn. Mệnh đề đo được chỉ là *"đường caret không đỏ lần nào"*.

  ⚠️ **Độ chập chờn ~3/8 là CÓ THẬT và nó đã có tiền lệ**: `§Change Log` của story ghi *"e2e 3/4
  lượt cả bộ 7/7"*, và `wdio.conf.mjs` mang một ghi chú về lượt đỏ chập chờn `attribution-focus`
  chưa chẩn đoán từ 2026-08-12. ⇒ Mâu thuẫn nêu ở trên **đã giải**: con số 7/7 không sai, nó chỉ
  là một lượt may trong một bộ chập chờn cộng một cổng 1420 sạch. **Cả hai mệnh đề cùng đúng.**

  ⚠️ Đừng đọc mục này thành *"hai bản vá đáng ngờ"*. `vue-tsc` · `eslint` · 11 cổng · build ·
  vitest 83/83 · cargo 338/0/5 đều xanh sau lượt vá. Thứ còn thiếu là một phép kiểm **của riêng
  hai mệnh đề trên**, không phải một nghi ngờ về mã.

  **Chủ: Ice** — chọn giữa (a) chạy tay `npm run test:e2e` một lượt rồi ghi số vào §Debug Log của
  story, hay (b) giao vế ② cho một lượt vitest và vế ① cho story kế tiếp có động tới lưới.

---

## Deferred from: 2-5c-cat-bo-cau-khoi-ban-dich (2026-08-15) — ghi LÚC KÝ, không lúc nghiệm thu

🔵 Mục này viết **ngay khi Ice ký năm quyết định mở của Task 0**, trước dòng mã đầu tiên — đúng
Task 0.3 của story. Ghi lúc ký chứ không lúc nghiệm thu là có chủ ý: một phần phạm vi bị thu hẹp
**bởi một chữ ký** là thứ dễ trôi nhất, vì lúc nghiệm thu nó không biểu hiện thành một ca đỏ nào.

### 🟡 AC1 vế "một dải câu" — Quyết định #1, Ice ký đường (b) ngày 2026-08-15

**Đặc tả đòi:** *"Given một câu hoặc **một dải câu** đang chọn / When gọi thao tác cắt bỏ / Then cờ
đặt trên câu nguồn / And phạm vi do người dùng chọn **từng lần**, không định trước"*
(`epics.md:2341-2348` · FR133 · `EXPERIENCE.md:132`).

**Story 2.5c dựng:** vế **một câu** — câu đang có caret. Vế **dải câu** KHÔNG dựng.

**Vì sao, kèm số đo (2026-08-15):**
- `editorPanelState.ts:51` khai `const caretSegmentId = ref<number | null>(null)` — **một số duy
  nhất**, không mảng, không `Set`. `setEditorCaret(id: number | null)` nhận đúng một `id`. Không
  `selectedIds` / `Set<number>` / `{from,to}` ở bất kỳ đâu trong `GridPanel.vue` ·
  `editorPanelState.ts` · `segmentNavigation.ts` · `selectionContract.ts`. **(Chủ: story kế tiếp chạm `editorPanelState.ts`.)**
- **Và đặc tả không mô tả CƠ CHẾ.** `prd.md:458` · `epics.md:130,2343` · `EXPERIENCE.md:132` đều
  chỉ mô tả **kết quả** *("phạm vi do người dùng chọn từng lần")*. Không tài liệu nào của dự án
  nói Shift+click, kéo chọn, hay Shift+mũi tên. ⇒ Dựng nó trong story này là **tự thiết kế một
  tương tác chưa ai đặc tả**, không phải thi hành một đặc tả đã có.

⚠️ **Bẫy đọc, ghi ra vì hai khái niệm trùng tên:** `selectionContract.ts` phục vụ **chọn văn bản
trong một cột** cho Auto-Lookup. Nó **không phải** cơ chế chọn nhiều **hàng**. Ai đóng món này
đừng cắm vào đó — ngoài chuyện sai khái niệm, `check-commands.mjs:1876-2113` Kiểm F đếm **tĩnh**
số lời gọi `useSelectionSurface(...)` literal *(sàn 6, bảng theo tệp `GridPanel.vue: 2`)*, nên một
lượt cắm nhầm vừa sai ngữ nghĩa vừa làm cổng đỏ.

**Chủ: một story sau của Epic 2 có động tới lưới** — ứng viên tự nhiên là **2.8** *(gộp và tách
segment tường minh)*, vì nó là story đầu tiên trong epic **buộc** phải có một khái niệm "nhiều
segment cùng lúc" ở tầng UI. 🔴 Ai nhận: **đừng** sửa `epics.md` cho khớp mã đã viết
(`project-context.md:456-458`) — AC1 vẫn đúng, chỉ là đường đi chưa tới.

→ 🟡 **2.8 ĐÃ XÉT VÀ TỪ CHỐI — món này VẪN HỞ, chủ chuyển đi.** Quyết định #1, Ice ký đường
**(a)** ngày 2026-08-17: gộp đúng **hai** câu *(câu có caret + câu liền trên)*, không dựng
chọn nhiều hàng. Lý do y hệt lý do 2.5c đã ghi và nó **vẫn đứng**: không tài liệu nào của dự
án mô tả **cơ chế** chọn. ⇒ Ứng viên *"2.8"* ở trên **hết đúng**; chủ mới: một story sau của
Epic 2, hoặc một story UX dựng cơ chế chọn nhiều hàng trước.
⚠️ **Đọc kèm:** AC7 vế *"nhiều mảnh"* — khoảng hở song sinh của AC6 — thì 2.8 **đã đóng** ở
lượt code review 2026-08-17 bằng cơ chế **tích luỹ điểm cắt**, một tương tác Ice ký tại chỗ.
Hai khoảng hở cùng hình dạng, hai số phận khác nhau, và cái khác nhau là: tách chỉ cần nhiều
**điểm trong một câu**, còn gộp cần nhiều **câu** — chỉ vế sau mới đụng khái niệm chọn hàng.

### 🟡 AC5 vế "ẩn hoàn toàn ở đầu ra" — hai bề mặt tiêu thụ CHƯA TỒN TẠI

**Số đo 2026-08-15 (mở tệp ra đọc, không suy từ tên):**
- `src-tauri/src/core/export/mod.rs` — **6 dòng, toàn bộ là doc-comment, không một dòng mã**.
  `docx-rs` khai ở `Cargo.toml` nhưng `grep docx_rs` trong `src-tauri/src/**/*.rs` chỉ trúng chính
  dòng comment đó. *(Story ghi "7 dòng"; số thật là 6 — đính chính tại chỗ, không đổi kết luận.)* **(Chủ: story kế tiếp dựng export DOCX.)**
- `src/modes/ReadingMode.vue` — template chỉ có một `<p>` chở `t('mode.reading.status')`.
  Doc-comment tự ghi *"KHUNG RỖNG có chủ ý… toàn bộ thuộc Epic 5"*; `modeState.ts:30` xác nhận
  *"cả ba chế độ đều rỗng"*.

**Ice ký Quyết định #2 đường (b):** story này dựng **hàm thuần lọc ở Rust** + test hợp đồng khẳng
định câu đã cắt bỏ không xuất hiện. Đó là **cái chốt**, không phải bề mặt.

⇒ Vế còn hở là **hai lượt CẮM VÀO chốt đó**: **(Chủ: Epic 5.)**
- Chế độ đọc → **Epic 5** *(Story 5.11 · 5.12 · 5.13)* **(Chủ: Epic 5 — Story 5.11/5.12/5.13.)**
- Bản xuất → **Epic 8** *(Story 8.3 · 8.4 · 8.6)*

### 🔴 CÒN HỞ, và đây là món lớn hơn cả hai mục trên: nghĩa vụ FR133 chỉ phát biểu MỘT CHIỀU

**Phát hiện, đã grep để chắc (2026-08-15):** nghĩa vụ *"ẩn hoàn toàn câu đã cắt bỏ"* chỉ được phát
biểu **từ FR133 áp xuống**. Các FR xuất bản *(FR87 · FR88 · FR89 · FR121 · FR130 · FR131)* và các
Story của Epic 5 / Epic 8 **không AC nào tham chiếu ngược lại FR133**.

🔴 **Vì sao đây là một lớp lỗi chứ không một chỗ thiếu chữ:** một nghĩa vụ chỉ có chiều đi xuống
thì **không có ai canh ở phía tiêu thụ**. Người viết Story 8.3 đọc AC của chính nó, thấy đủ, và
xuất ra một tệp `.docx` có nguyên câu người dùng đã quyết định bỏ. Không cổng nào đỏ, không test
nào đỏ — đúng tiêu chí của §Critical Don't-Miss Rules: *"vi phạm được mà không cổng nào đỏ"*.

⚠️ Và loại suy trong chính đặc tả **dẫn sai đường ở đúng vế này**: `epics.md` và
`EXPERIENCE.md:126` viết *"đúng khuôn `translate="no"` của XLIFF"*. Trong XLIFF 2.0,
`translate="no"` **khoá** một unit và **GIỮ NGUYÊN nội dung trong bản xuất** — nó là *"đừng dịch
cái này"*, không phải *"bỏ cái này đi"*. Loại suy đúng ở vế *"trục độc lập"*, **sai** ở vế hành vi
đầu ra. Người đọc `EXPERIENCE.md:126` rồi đi viết phần xuất bản sẽ làm **ngược** AC5.

**Chủ: Ice** — quyết định: có thêm một AC tham chiếu FR133 vào Story 5.11–5.13 và 8.3 · 8.4 · 8.6
không. Đây là một quyết định về **quy hoạch**, không phải một dòng mã, nên nó không tự đóng được ở
tầng dev.

### 🔴 MÓN MỚI — `ornament` làm màu chữ: đặc tả nói một đằng, cổng cưỡng chế một nẻo

**Phát hiện giữa Task 4 của Story 2.5c, và nó không phải chỗ dev đọc sót.** `DESIGN.md:148`
khai `grid-row-omitted: { color: **ornament**, decoration: line-through }`. Vế **màu** của
dòng đó **không thi hành được**: `tokens.json` `contrast.neverTextTokens` ghi cho `ornament`
câu *"KHÔNG một ngoại lệ nào — token này không bao giờ là màu chữ"*, và
`check-tokens.mjs:1300-1334` cưỡng chế nó với một bảng miễn trừ **cố ý rỗng** *(chính Ice dọn
nó rỗng ở lượt ra mã 2.5b, gỡ một miễn trừ chết cho ký tự `⏐`)*.

**Đo 2026-08-15 trên nền `surface`, sàn AA = 4,5:**

| Token | Sáng | Tối | |
|---|---|---|---|
| `ornament` | **2,44** | **2,64** | trượt |
| `on-surface-variant` | **5,60** | **5,56** | đạt |

🔴 **Phép tự kiểm đã CHẠY, không phải một lời khẳng định:** đặt
`.cell.omitted { color: var(--color-ornament) }` ⇒ `check:tokens` **ĐỎ** với
`FAIL src/panels/GridPanel.vue:1321 — `ornament` dùng làm màu chữ`. Trả lại ⇒ **XANH**. Cổng
này bít thật, không chỉ khai trên giấy.

⇒ **Quyết định #6, Ice ký 2026-08-15: đường (a)** — `on-surface-variant` + `line-through`.
`DESIGN.md:148` đã **sửa tại chỗ** cho khớp.

**⚠️ Và lượt này lộ ra một món CŨ, cùng lớp, chưa ai đóng:** `DESIGN.md:145` và `:146` vẫn
khai `color: ornament` cho `grid-num-col` và `grid-state-col`, trong khi
① mã đã dùng `on-surface-variant` cho cả hai từ Story 2.5b *(`GridPanel.vue`, Quyết định
#9(a) — cùng lý do nguyên văn: **"đây là chữ thật, không phải nét"**)*, và
② **chính `DESIGN.md:213` tự mâu thuẫn với chúng**: *"`ornament` và `tm-rule` là màu của nét,
không bao giờ là màu của chữ. Mọi chữ, kể cả nhãn 10px, tối thiểu phải là
`on-surface-variant`"*.
⇒ Hai dòng đó **đã sửa cùng lượt** — chúng là đúng cái *"nửa là MỘT DÒNG CHUỖI thì rơi"* mà
lượt ra mã 2.5b đã gọi tên thành một **khuôn lặp lại ba lần**. 🔴 Đây là lần **thứ tư**.

**Chủ: Ice** — món còn lại **không** phải một dòng chữ nữa mà là một câu hỏi về quy trình:
`DESIGN.md` §components hôm nay **không có cổng nào canh**, nên nó trôi khỏi mã mỗi story và
mỗi lần đều được phát hiện bằng mắt của một lượt sau. Ba ứng viên, chưa đường nào được đo:
một cổng đối chiếu §components với CSS · gộp §components vào `tokens.json` để cổng sẵn có
nhìn thấy · hoặc chấp nhận nó là tài liệu-người-đọc và **gỡ** các giá trị màu khỏi đó.

### 🟡 Task 7.4 — độ trễ dời con trỏ: KHÔNG đo lại ở story này, và lý do là một phép đo

Story 2.5c thêm **một `:class` boolean** (`omitted: s.is_omitted`) vào **bốn** trong năm
`v-for` của lưới. Nó **không** đổi cấu trúc DOM: không node mới, không phần tử bọc, số node
mỗi hàng vẫn là **5**.

⚠️ Nhưng nó **không phải zero**: mỗi lượt Vue vá lớp nay tính thêm bốn biểu thức boolean trên
mỗi hàng. Ở 9.850 câu đó là 39.400 phép đọc thuộc tính mỗi lượt dời con trỏ — trên một đường
2.5b đã đo **706–770 ms**, tức **vượt trần 50 ms/frame của NFR2 khoảng 15 lần** *(mốc
`:3164-3194`)*.

🔴 **Không tự chấm "không ảnh hưởng đáng kể"** — đó là một suy luận, và luật đo của dự án cấm
đánh dấu đạt bằng suy luận. Số này **giao lại**, không tự đóng.
**Chủ: Story 2.4** *(sở hữu bộ đo NFR2/NFR18)* — khi bộ đo chạy lại, chạy trên cây **sau**
2.5c và so với 706–770 ms, chứ đừng so với một mốc trước lưới.

### 🟡 AC3 — vế "hàng vẫn thẳng hàng trong subgrid" chưa có phép đo của riêng nó

Task 4.5 đòi kiểm bằng mắt rằng hàng đã cắt bỏ vẫn **thẳng hàng** với các hàng khác và khoảng
thở `is_paragraph_end` không vỡ. Lượt cài đặt **không** đụng chiều cao: nó thêm `color` +
`text-decoration`, hai thuộc tính không tham gia bố cục.

⚠️ Vẫn ghi ra vì `happy-dom` **không phải WebKit** — mọi mệnh đề hình học thuộc bàn đo hoặc
e2e, và **không đường nghiệm thu nào của story này khẳng định nó**. Một `line-through` trên
một ô có `SourceHanViet` bên trong *(cột nguyên văn, chế độ song song)* là chỗ chưa ai nhìn:
gạch ngang **kế thừa** xuống mọi con, nên phần Hán Việt cũng bị gạch — đúng hay không thì
chưa có ai phán.
**Chủ: Ice** *(một lượt nhìn bằng mắt)* — hoặc một story sau có động tới cột Hán Việt.

---

## Story 2.5d — Ngắt đoạn của bản dịch (2026-08-16)

### 🟡 AC3 vế **gộp** và vế **tách do người dùng gọi** — bảng ba ca chưa có chỗ áp

AC3 đòi ba ca biên của AD-37 áp **y nguyên** cho cờ đích. Đo lại 2026-08-16:
`grep "fn merge_segments\|merge_segment\|MergeSegment"` trên `src-tauri/src/**` ⇒ **0** kết
quả; Story 2.8 *(gộp/tách tường minh)* là `backlog`.

**Đã đóng được vế nào:** ca *"segment cuối Chương ⇒ cờ tắt, luôn luôn"* có mã thi hành ở
đường nhập, và vì cờ đích **bằng** cờ nguồn lúc nhập (AC2), ba ca đúng cho cờ đích **theo
dẫn xuất**. Cưỡng chế bằng
`segment_contract.rs::a_freshly_imported_chapter_mirrors_the_source_flag_into_the_target_flag_row_by_row`.

**Còn hở:** ngày người dùng đã **đổi** cờ đích rồi mới gộp/tách — lúc đó hai cờ khác nhau và
bảng phải chạy **hai lần, độc lập**. Quyết định #6 đường (b) *(Ice ký 2026-08-15)* dựng sẵn
hàm thuần `core::segment::paragraph` *(`at_end_of_chapter` · `merged` · `split_into`)* cộng
bốn ca hợp đồng, để Story 2.8 chỉ việc gọi.
🔴 **Cái bẫy đã ghi thành test:** cách viết tự nhiên ở 2.8 là lấy cờ của câu cuối rồi coi cờ
đích *"chắc cũng vậy"* — lượt suy đó **xoá quyết định ngắt đoạn của người dùng**, và không
cổng nào đỏ.
**Chủ: Story 2.8.**

→ ✅ **ĐÃ ĐÓNG 2026-08-17 (Story 2.8).** `core::segment::regroup::merge` gọi `merged()` và
`split_at` gọi `split_into()` — hai bề mặt tiêu thụ mà Quyết định #6(b) đã dựng sẵn hàm thuần
để chờ. **Cái bẫy đã ghi thành test thì KHÔNG sập:** Task 3.2 đọc hai cờ **riêng từng cột** ở
`load_segment_for_write`, và ca hợp đồng dùng một cặp cờ **lệch nhau** nên một lượt chép cờ
nguồn sang cờ đích cho đỏ. ⚠️ Món này được đóng ở lượt **code review** ngày 2026-08-17, không
ở lượt dev — diff của story chỉ nối thêm ở cuối tệp và bỏ sót cả bốn món có chủ 2.8.

### 🟡 AC4 vế *"đường xuất đọc cả hai nguồn"* — bề mặt tiêu thụ là khung rỗng

AC4 nói *"đường mã nào cần cấu trúc đoạn của bản dịch thì đọc dữ liệu đã lưu"*. Cột đã có,
dây IPC đã chở nó, lưới đã đọc nó. Nhưng bề mặt **tiêu thụ thật** là đường xuất, và
`core/export/mod.rs` vẫn là **6 dòng toàn doc-comment, 0 dòng mã** *(đo lại 2026-08-16)*.
⚠️ Cùng khuôn lỗi mà 2.5c tìm ra cho FR133: nghĩa vụ phát biểu **một chiều**, không AC nào
của Epic 8 tham chiếu ngược lại AD-46 ⇒ người viết Story 8.3 đọc AC của chính nó, thấy đủ,
và xuất ra một tệp mang nhịp của **bản gốc**.
**Chủ: Epic 8** *(Story 8.3 · 8.4 · 8.6)*.

### 🟡 Lượt đổi cờ đích bị TỪ CHỐI không có đường ra màn hình

`setCurrentSegmentParagraphEnd` trả `'refused'` và `main.ts` ghi một `console.warn`. Không ô
lỗi riêng, **có chủ ý**: kho hôm nay đã có hai ô lỗi Editor và **một** trong hai
(`editorOmitError`) được export mà **không component nào đọc** — thêm ô thứ ba là nhân một bề
mặt chết. ⇒ Người dùng bấm `Mod+Alt+P` trên một segment đã về hưu thì **không thấy gì**.
**Chủ: một story dựng đường báo lỗi dùng chung cho lệnh Editor** *(gộp luôn `editorOmitError`)*.

### 🔴 `check-i18n.mjs` Kiểm A: một tên thẻ nhắc trong COMMENT của template làm hỏng bộ quét

Đo được 2026-08-16, tái lập chắc chắn: viết `` `<style>` `` **bên trong** một `<!-- … -->` của
template `.vue` làm Kiểm A báo FAIL ở **một comment khác, cách đó 20 dòng**
(`GridPanel.vue:1071`). Nguyên nhân: `scanTemplate` gặp `<` + chữ cái thì chuyển sang state
`tag` — và nó làm vậy **kể cả khi đang ở trong một comment** mà nó vừa nhảy qua, vì lượt nhảy
kết thúc ở `-->` đầu tiên còn phần văn bản sau đó được đọc lại từ đầu ở state `text`.
⚠️ Hậu quả **không phải** một lượt bỏ lọt *(cổng đỏ chứ không xanh)*, nhưng nó **đỏ sai chỗ**,
và người sửa sẽ đi tìm ở dòng cổng chỉ vào — cách xa nguyên nhân. Lượt này mất một vòng chẩn
đoán để lần ra.
**Vá tạm đang dùng:** không nhắc tên thẻ trong comment của template. **Chủ: một story hạ tầng
cổng.**

### ⚠️ Vế **Blink** của mọi phép đo hình học và hành vi engine — khoảng mù có tên

Bàn đo `2-5d-ban-do/` chạy **chỉ trên WKWebView 605.1.15**. Vế Blink chỉ tới được qua
WebView2/Windows, và kho **không có đường nghiệm thu tại chỗ** cho nửa đó.
🔴 Đây **không** phải một lo xa: tiền lệ trực tiếp là `Backspace` offset 0 — WebKit phát **0**
`beforeinput`, Blink thì có *(deferred-work, Story 2.5b)*. Hai engine **đã** nói ngược nhau ở
đúng địa hạt này một lần.
Cụ thể chưa đo trên Blink: `insertLineBreak` dưới `pre-line` có dựng text node `\n` không, và
`A<br>` có vẽ ra một dòng hay hai.
**Chủ: Story 1.22** *(hạ tầng e2e hai nền tảng)*.
→ ⚠️ **KHÔNG ĐỔI CHỦ 2026-08-18 — Ice ký đường (c) của quyết định #1 (Story 2.12).** Story 2.12 chạy
**chỉ trên macOS/WKWebView**; đường (b) *(ôm luôn vế Windows)* bị loại vì không có máy Windows, và
ôm nó sẽ chặn cửa chặn ② vô hạn. Món này **giữ nguyên chủ Story 1.22**, không chuyển sang 2.12.
🔴 Và đường (c) khác đường (a) đúng một dòng, dòng đó là lý do Ice chọn nó: xem món mới
*"AC7 của Story 2.12 là một mệnh đề MỘT NỀN TẢNG"* ở cuối tệp này.

### ⚠️ Độ trễ dời con trỏ sau khi Task 8 thêm một nhánh vào `:class` — GIAO LẠI, không tự chấm

Story 2.5d thêm `'tgt-para-end': s.is_target_paragraph_end` vào biểu thức `:class` của ô bản
dịch — tức một thuộc tính nữa được tính **cho mỗi hàng** ở mỗi lượt render.

🔴 **Không đo được ở story này, và không được suy ra:** phép đo độ trễ dời con trỏ cần bộ đo
9.850 câu, và bộ đo đó là của **Story 2.4** *(đang treo ở chỗ chưa tiêm được `bench.js` vào
webview bản release)*. Số gần nhất còn hiệu lực: **706–770 ms** trên 9.850 câu, **vượt trần
NFR2 (50 ms/frame) ~15 lần** — đo ở Story 2.5b, còn hở, chủ **Story 2.4**.
⇒ Story này **không** làm số đó tốt lên và cũng **không** khẳng định nó không xấu đi. Một
thuộc tính boolean thêm vào một object literal đã có bốn nhánh là **nhỏ so với 706 ms**, nhưng
đó là một **suy luận**, và luật của kho cấm đánh dấu đạt bằng suy luận.
**Chủ: Story 2.4** *(bộ đo NFR2)*.

---

## Deferred from: code review of 2-5d-ngat-doan-ban-dich (2026-08-16)

### 🟡 AC6 vế *"giữ nguyên ở mọi nơi khác"* — không có bề mặt thứ hai để đối chứng

AC6 đòi gỡ hai lớp chặn `Enter` **ở ô bản dịch**, và **giữ nguyên ở mọi nơi khác**. Vế gỡ
đóng và đo được. Vế **giữ nguyên** thì chỉ đúng **theo CẤU TRÚC**: `@beforeinput`/`@keydown`
gắn đúng trên `div.col-tgt`, cột nguyên văn không mang listener nào. Đó là một lập luận về
hình dạng mã, **không** một phép đo trên một bề mặt thứ hai — vì hôm nay **không có** bề mặt
soạn thảo nào khác trong sản phẩm để `Enter` được bấm thử ở đó.

🔴 **Vì sao món này phải nằm ở đây thay vì được chấm đạt:** *"không đánh dấu đạt bằng suy
luận"*. Cấu trúc đúng hôm nay không chứng minh cấu trúc đúng vào ngày bề mặt thứ hai ra đời —
và đúng vào ngày đó, người viết nó sẽ đọc AC6 thấy chữ *"đã đóng"* rồi không kiểm gì cả.

⇒ **Đóng bằng cách:** story nào dựng bề mặt soạn thảo thứ hai chạy lại đúng hai mệnh đề của
AC6 trên bề mặt đó — `Enter` làm gì, và `Mod+Enter` có còn ký được không.

**Chủ: Story 8.11** *(`8-11-review-mode-bo-cuc-hai-cua-so-side-by-side` — bề mặt soạn thảo thứ
hai gần nhất trong sổ sprint; nếu một story sớm hơn dựng bề mặt trước thì món này theo về đó)*.

### 🟡 Lượt DÁN giữ `\n` — vế DỮ LIỆU đã đo, vế THỊ GIÁC thì chưa

Code review 2026-08-16, Ice ký đường (b): nhánh ② của `onBeforeInput` (`GridPanel.vue`) thôi
làm phẳng `\n` thành khoảng trắng, để một đường vào ô mang **một** luật với `Enter` gõ tay.

Vế **dữ liệu** có lưới: `\n` dán vào đi tới `target_text` nguyên vẹn (vitest). Vế **thị giác**
— *"`\n` dán vào hiện ra hai dòng thật dưới `white-space: pre-line` trên WKWebView"* — **chưa
đo**. Bàn đo `2-5d-ban-do/` đo `insertLineBreak` của engine; đường này chèn một **text node do
chính mã dựng**, và bàn đo chưa chạy vòng nào cho nó.

⚠️ Suy luận *"cùng là text node mang `\n`, cùng `pre-line`, nên cùng hiện hai dòng"* rất
mạnh — và luật của kho vẫn cấm chấm đạt bằng nó.

**Chủ: Story 1.22** *(hạ tầng e2e hai nền tảng — gộp cùng vế Blink đã ghi ở trên, một lượt
chạy trả lời cả hai)*.
→ ⚠️ **KHÔNG ĐỔI CHỦ 2026-08-18** — cùng lý do và cùng chữ ký với món Blink ngay trên (Ice ký #1(c)).

- **Ba chú thích ở `src/main.ts:248,264,281` khẳng định sai phạm vi Kiểm A của `check:i18n`** — cả
  ba viết *"Chẩn đoán viết KHÔNG DẤU — Kiểm A của `check:i18n`"*, nhưng `check-i18n.mjs:839,860-861`
  cho thấy Kiểm A quét đúng hai quần thể: `rsFiles` (`.rs`) và `vueFiles` (`.vue`). `src/main.ts` là
  `.ts` ⇒ **không cổng nào canh những dòng đó**; chúng có dấu cũng không ai đỏ. Quy ước viết không
  dấu ở đây vẫn đáng giữ (nhất quán phong cách với `.rs`/`.vue`), chỉ **lý do** ghi ra là sai — và
  một lý do sai là thứ người sau sẽ tin thay vì đo lại.
  🔴 **Có sẵn từ trước, không do Story 2.5d gây ra** — ba dòng này đến từ Story 2.5 và 2.5c. Lượt
  rà 2.5d bắt được vì story lặp lại đúng mẫu đó ở một dòng thứ tư (`main.ts:293`); dòng thứ tư được
  vá trong chính lượt rà, ba dòng cũ để lại đây.
  ⚠️ Món này **không** phải "sửa ba chú thích". Câu hỏi thật đứng sau nó: quy ước *"chẩn đoán viết
  không dấu"* trong `src/**/*.ts` hôm nay **không có cổng nào canh** — nên hoặc nó được nới thành
  một luật có cổng (mở rộng quần thể Kiểm A sang `.ts`, kèm xét lại sàn quần thể), hoặc ba chú
  thích phải nói đúng rằng đây là **quy ước tay**, không phải một cổng.
  **Chủ: một story hạ tầng cổng** *(gộp cùng khuyết tật `check-i18n` Kiểm A mà chính Story 2.5d đã
  ghi ở trên — hai món cùng chạm một tệp)*.

---

## Story 2.6 — Lịch sử phiên bản segment và khôi phục (2026-08-16)

Sáu món dưới đây ghi **lúc Ice ký tám quyết định mở của Task 0**, không phải lúc nghiệm thu —
đúng luật *"phần lệch ghi vào `deferred-work.md` kèm chủ ngay lúc ký"* (story §Task 0.3). Không
mục nào mồ côi.

- 🟡 **AC4 đóng MỘT NỬA: vế "bề mặt vào" cho một segment đã về hưu không đối chứng được.**
  Chữ ký **#8(a)** đóng đúng mệnh đề mà AC4 phát biểu — *"lịch sử vẫn tra lại được"* — bằng một
  test hợp đồng dựng `retired_at` bằng SQL trực tiếp. Vế còn hở là vế **người dùng**: hôm nay
  **không đường mã nào cho một segment về hưu**. Đo lại 2026-08-16 từ nguồn: `retired_at` là
  `None` cho mọi segment, `grep merge_segment` trên `src-tauri/src/**` cho **0 đường mã**
  *(xem bẫy grep ngay dưới)*, và Story 2.8 là `backlog`. `schema.rs:296-298` ghi thẳng rằng cột
  có mặt sớm để 2.8 không phải mở một bước di trú thứ hai.
  ⇒ Khi 2.8 dựng gộp/tách, **nghiệm thu lại AC4 trên một segment về hưu THẬT**, không trên một
  hàng dựng bằng SQL. **Chủ: Story 2.8.**

  → ✅ **ĐÃ ĐÓNG 2026-08-17 (Story 2.8).** Ca
  `the_history_of_a_genuinely_retired_segment_still_reads_back_after_a_real_merge` chạy một
  lượt `merge_segments` **thật** rồi tra lịch sử của hàng vừa về hưu — không một `retired_at`
  nào dựng bằng SQL. Vế *"bề mặt vào"* vì thế có mã sản phẩm sinh ra nó lần đầu.
  ⚠️ Đóng ở lượt **code review** 2026-08-17, không ở lượt dev.

- 🟡 **Bốn nhãn của mockup không được dựng — chữ ký #5(a), và chúng có BỐN chủ tách rời.**
  `data-integrity.html` vẽ mỗi hàng phiên bản kèm một nhãn; bảng `segment_version` có **đúng bốn
  cột** (`id` · `segment_id` · `target_text` · `created_at`, `schema.rs:460-467`) và
  doc-comment ngay trên khai bằng chữ rằng cột xuất xứ thuộc 2.7 và cột cặp TM thuộc Epic 7.
  | Nhãn | Cần năng lực | Chủ |
  | --- | --- | --- |
  | `đang dùng` + dòng *"bạn sửa · đã xác nhận"* | xuất xứ FR117 | **Story 2.7** |
  | `từ bản review` | Review Mode FR94 | **Epic 8** |
  | `từ AI` | — | **Epic 4** |
  | `từ TM` | FR58 | **Epic 7** |
  ⚠️ Nhãn `đang dùng` mang một cái bẫy riêng đã đo: so theo **nội dung** thì hai phiên bản trùng
  văn bản làm nhãn khớp **nhiều** hàng. Story nào dựng nó phải nhớ theo **id**, không theo nội
  dung — tức nó cần một cột hoặc một quy ước mới, không chỉ một lượt render.

- 🟡 **Vế DIFF (`So với phiên bản trước`) không được dựng — chữ ký #4(a).**
  Mockup vẽ `<del>`/`<ins>`; **không AC nào của Story 2.6 đòi diff**. `src-tauri/Cargo.toml:86-89`
  ghi sẵn cả hai số — `similar` 3.1.1 · `dissimilar` 1.0.11 — và **cố ý không cài cái nào**.
  🔵 **Chủ cụ thể hơn thứ story ghi:** chú thích tại chỗ nói chốt ở **Story 8.1**, không chỉ
  *"Giai đoạn 5"* — *"sau khi thử cả hai trên bản review thật"*. Ghi số cụ thể vì một món nợ chủ
  *"một giai đoạn"* là một món nợ không ai nhận.
  🔴 Khi diff được dựng: **không** `v-html`. Rust phân tích thành mô hình dữ liệu có cấu trúc,
  Vue render từ mô hình đó, và mô hình **không có nhánh nào mang HTML** (AD-16).
  **Chủ: Story 8.1.**

- 🔴 **HỞ THẬT: `is_target_paragraph_end` KHÔNG được khôi phục cùng `target_text`, và bảng không
  có chỗ nào lưu nó.** Cờ ngắt đoạn của bản dịch (bước di trú 9, Story 2.5d, AD-46) là **dữ liệu
  riêng của bản dịch** — nó sống trên `segment`, không trên `segment_version`. AC2 nói khôi phục
  *"văn bản đích"* và **không nói cờ đích**. ⇒ Một lượt khôi phục trả `target_text` về bản cũ
  nhưng **giữ nguyên cấu trúc đoạn hiện tại**, và với một bản dịch từng ngắt đoạn khác đi thì hai
  thứ đó **không còn nói cùng một chuyện**.
  ⚠️ Đây **không** phải một lỗi cài đặt — nó là một khoảng hở **ngữ nghĩa** mà cả AD-31, AD-46 lẫn
  năm AC của story này đều không nói tới. Ba đường thoát, cả ba đòi một quyết định chứ không một
  dòng mã: ① thêm cột cờ vào `segment_version` *(một bước di trú, và làm "phiên bản" mang nghĩa
  rộng hơn "văn bản")* · ② khai bằng chữ rằng khôi phục **chỉ** đụng văn bản · ③ khôi phục cũng
  hạ cờ về một giá trị mặc định *(mất dữ liệu, đường tệ nhất)*.
  Story 2.6 làm ①-không, ②-có: ghi mệnh đề vào doc-comment tại chỗ và ghi món nợ này.
  **Chủ: Ice** *(quyết định ngữ nghĩa, không phải một lượt cài đặt)*.

- 🟡 **Lượt từ chối khôi phục thừa hưởng một bề mặt báo lỗi đang dở.** `editorConfirmError` hiện
  một chuỗi **cố định**, không đọc `message_key` thật — món nợ đã ghi ở `:2825-2840` (🟡, chủ Ice).
  Đường khôi phục của story này dựng thêm ba nhánh từ chối *(không tìm thấy · đã về hưu · phiên
  bản không thuộc segment đó)* và cả ba đi vào đúng bề mặt dở đó.
  ⚠️ Ghi ra vì nó làm món nợ cũ **nặng thêm**, không phải vì nó là món mới: trước story này có
  hai lệnh Editor dùng ô lỗi chung, nay là ba. **Chủ: Ice** *(gộp vào món `:2825-2840` đã có)*.

- ⚠️ **Mockup nói phiên bản "thứ sáu" xuất hiện NGAY lúc khôi phục; chữ ký #1(a) làm nó xuất hiện
  MUỘN HƠN MỘT NHỊP.** `data-integrity.html:226-229` viết đậm *"Khôi phục là tạo phiên bản
  mới… đẩy nó lên thành phiên bản thứ sáu"*. Bảng Rule của AD-31 (`ARCHITECTURE-SPINE.md:374-381`)
  có **đúng sáu hàng và không hàng nào là "khôi phục"** — đây là một **mâu thuẫn đo được giữa hai
  tài liệu quy hoạch**, không một chỗ đọc nhầm.
  Ice ký **#1(a)**: AD-31 không sửa một chữ, AC2 đúng nguyên văn, và lời hứa *"lịch sử chỉ dài
  thêm, không bao giờ ngắn đi"* **vẫn giữ** — nó chỉ dài thêm ở **lượt xác nhận kế tiếp**, do
  chính hàng 2 của AD-31 sinh ra.
  ⇒ Phần lệch còn lại là **một câu trong mockup nói sai về THỜI ĐIỂM**, không về cơ chế. Dev
  **không** sửa mockup và **không** sửa `epics.md` — sửa một tài liệu tầng quy hoạch là một lượt
  riêng của Ice. **Chủ: Ice.**

- ⚠️ **Bẫy đo: một phép `grep` tiền đề có thể tự bắt chính câu nói về nó.**
  Task 0.6 của Story 2.6 đòi đo lại `grep "merge_segment" src-tauri/src`. Kết quả thô là **1**,
  không phải 0 — và dòng khớp duy nhất là `core/segment/paragraph.rs:10`, một **doc-comment** viết
  nguyên văn *"`grep …` trên `src-tauri/src/**` cho **0**"*. Số thật vẫn là **0 đường mã**, nhưng
  một lượt đọc số thô kết luận ngược, và kết luận ngược đó chặn đúng một quyết định (#8).
  ⇒ Luật rút ra, áp cho mọi lượt đo tiền đề về sau: **đọc NỘI DUNG dòng khớp, đừng đếm.** Kho này
  ghi kết quả đo vào chú thích rất dày *(đó là văn hoá có chủ ý)*, nên lớp bẫy này sẽ **gặp lại**.
  Không có cổng nào canh được nó. **Chủ: không ai — đây là một luật đọc, ghi ra để người sau khỏi
  vấp lại.**
  → KHÔNG LÀM 2026-08-19 (Story 2.13) — mục tự khai "Chủ: không ai — đây là một luật đọc": không
  có cổng nào canh được nó và không có việc để giao chủ, đúng bản chất một bài học ghi lại, không
  một việc chờ làm.

- ⚠️ **Ca `toISOString()` của `historyTimeLabel` RỖNG NGHĨA trên CI, và CI là nơi duy nhất chạy
  tự động.** Story 2.6 dựng quy ước định dạng thời gian đầu tiên của kho
  (`src/panels/segmentHistoryTime.ts`), và phép so ngày của nó phải đọc theo **giờ địa phương**
  chứ không `toISOString()` — cái sau trả về theo UTC nên nó rẽ sai ở hai chiều ngược nhau tuỳ
  dấu offset *(offset dương: mốc sáng sớm hôm nay đọc thành "hôm qua"; offset âm: mốc tối muộn
  hôm qua thôi đọc thành "hôm qua")*.
  **Đo 2026-08-16:** ca `tests/frontend/segmentHistoryTime.test.ts` bắt được cái bẫy này **trên
  máy của Ice (UTC+7)** — đỏ-rồi-xanh đã chạy hai lượt. Ca đã được viết để **tự chọn chiều** theo
  offset đang chạy, nên nó cũng bắt được ở múi giờ âm.
  🔴 Nhưng ở **UTC đúng** (offset 0) cả hai chiều biến mất và ca **xanh kể cả trên một hàm dùng
  `toISOString()`**. Runner GitHub Actions chạy **UTC**. ⇒ Mệnh đề này hôm nay được canh **chỉ ở
  lượt `pre-push` trên máy người chạy**, không ở CI. Ca tự khai điều đó bằng một nhánh
  `expect(offsetMin).toBe(0)` thay vì giả vờ đã đo.
  ⚠️ Đường đóng: đặt `TZ` tường minh cho một tệp test *(vitest `environmentOptions` hoặc một
  `process.env.TZ` đặt trước khi nạp module)*. Chưa làm vì nó là một quyết định về **cấu hình bộ
  chạy test**, không một dòng trong story này — và đặt `TZ` toàn cục sẽ chạm mọi tệp test khác.
  **Chủ: một story hạ tầng cổng** *(gộp cùng hai món `check-i18n` đã ghi ở trên)*.

- ⚠️ **Sau Story 2.6, `src/config/segment.ts` có HAI loại adapter, và một kho nửa này nửa kia là
  một kho không đoán được luật.** Sáu adapter cũ *(`splitChapterIntoSegments` ·
  `readOpenChapterSegments` · `saveSegmentTargets` · `confirmSegment` · `setSegmentOmitted` ·
  `setSegmentParagraphEnd`)* **tin** payload thành công và chỉ kiểm hình dạng của **lỗi**
  (`isIpcError`). Adapter `readSegmentHistory` của story này **kiểm cả payload** lúc chạy
  (`isSegmentVersionArray`), và biến một hình dạng sai thành một trạng thái **phân biệt được**.
  Lý do lệch: nó đóng một lớp lỗi **đã xảy ra thật** — bản đầu Story 2.5 quên thêm `status` vào
  struct Rust và vào câu `SELECT` ⇒ `undefined` phía webview ⇒ `isConfirmed` luôn `false` **trên
  sản phẩm thật**, trong khi 74/74 test frontend vẫn xanh vì fixture chép tay có sẵn cột.
  ⚠️ Nhưng lý do đó áp cho **cả sáu** adapter kia y hệt, nên lượt này đóng lỗ ở **hai** chỗ và để
  hở ở **sáu**. Câu hỏi thật là một câu hỏi **quy ước**: nâng cả sáu lên, hay hạ hai cái này
  xuống và tin vào lưới Rust + e2e. Không cổng nào canh sự nhất quán này. **Chủ: Ice.**

## Deferred from: code review of 2-6-lich-su-phien-ban-segment-va-khoi-phuc (2026-08-16)

- 🟡 **Không chỉ dấu nào nói *"phiên bản nào đang được dùng"* — và đường suy từ nội dung đã bị
  BÁC, không phải chưa nghĩ tới.** Bản dựng đầu của Story 2.6 gắn một viền trái
  `--color-primary` cho hàng có `target_text` trùng văn bản đang dùng (`hist-current`), kèm một
  lời tự biện minh rằng *"nó cố ý không mang chữ"*. Code review 2026-08-16 bác lập luận đó và
  **Ice chốt gỡ**: đây **chính là** phép so mà bảng của Quyết định #5 gọi tên là *"suy được,
  nhưng KHÔNG AN TOÀN — hai phiên bản trùng văn bản thì nhãn khớp NHIỀU hàng"*, và chữ ký (a) là
  *"không nhãn nào"*. Lập luận *chữ thì cấm, màu thì không* không đứng được: chuỗi thao tác
  **ký → sửa → hoàn tác về bản cũ** là chuyện thường ngày và nó sinh hai hàng trùng văn bản ⇒ cả
  hai cùng sáng viền, tức chỉ dấu **nói dối** ở đúng ca nó được dựng để phục vụ.
  ⇒ Vế này đóng được ở **Story 2.7**: cột xuất xứ (FR117) làm câu hỏi *"hàng nào đang dùng"* trả
  lời được theo **`id`** chứ không theo **nội dung** — tức đúng đường (b) mà Quyết định #5 mô tả
  là an toàn nhưng chưa dựng được ở 2.6. **Chủ: Story 2.7** *(cùng chủ với bốn nhãn kia)*.

- ⚠️ **`restore_segment_version` KHÔNG hỏi lại khi văn bản hiện tại là chuỗi RỖNG**, và vế miễn
  trừ đó (`!current_text.is_empty()`, `segment.rs`) nay đã có lý do viết ra tại chỗ nhưng
  **chưa có một ca hợp đồng nào** dựng tình huống. Hôm nay vô hại — ô rỗng không có chữ nào của
  người dùng để mất, và bỏ vế này đi thì mọi câu **chưa dịch** đều bị hỏi một câu vô nghĩa ở
  lượt khôi phục đầu tiên *(mà một hộp thoại hỏi thừa là thứ làm người dùng bấm "đồng ý" theo
  phản xạ, tức làm chốt THẬT mất tác dụng)*. 🔴 Nhưng mệnh đề *"rỗng = không có gì để mất"* hết
  đúng vào ngày **"rỗng" mang một nghĩa RIÊNG** khác *"chưa dịch"* — ví dụ một lượt xoá sạch có
  chủ ý ở Review Mode, hay một segment về hưu do gộp/tách. **Chủ: Story 2.8** *(gộp/tách segment
  — chỗ gần nhất mà "rỗng" có thể tách nghĩa)*.

## Deferred from: 2-7-xuat-xu-ban-dich-cap-segment (2026-08-16)

- 🔴 **ĐÍNH CHÍNH món nợ ngay trên (*"phiên bản nào đang được dùng"*) — tiền đề của nó KHÔNG
  ĐỨNG ĐƯỢC, đo lại 2026-08-16 lúc dựng Story 2.7.** Món đó viết *"cột xuất xứ (FR117) làm câu
  hỏi 'hàng nào đang dùng' trả lời được theo `id` chứ không theo nội dung"*. Sai, và sai đúng
  cái lớp mà chính nó tồn tại để chống: một cột **xuất xứ** không nói hàng nào đang dùng — hai
  phiên bản cùng mang *tôi dịch* thì nhãn lại khớp **nhiều hàng**, y hệt phép so nội dung mà
  Quyết định #5 của 2.6 gọi là *"KHÔNG AN TOÀN"*. Thứ trả lời được câu đó là một **con trỏ**
  *(ví dụ `segment.current_version_id`)*, không một cột xuất xứ.
  🔴 Và chữ ký **#1(a)** *(Ice ký 2026-08-16: cột xuất xứ **chỉ** trên `segment`)* làm nó xa
  thêm một bậc: `segment_version` nay **không mang xuất xứ**, nên không có gì ở cấp phiên bản
  để đọc cả. ⇒ Món nợ **KHÔNG đóng ở Story 2.7**; nó **đổi chủ và đổi lý do**, không đổi trạng
  thái. Đóng nó là một **cột thứ ba** *(con trỏ)*, tức một quyết định lược đồ mới.
  **Chủ: Ice** *(cùng chủ với bốn nhãn kia)*.

- 🟡 **AC3 chỉ đối chứng được bằng fixture SQL — không đường sản phẩm nào sinh ra một xuất xứ
  khác mặc định hôm nay.** Ba ca `reviewing_a_sentence_word_for_word_...`,
  `typing_and_undoing_back_to_the_mark_...` và `..._claims_it_as_their_own` dựng câu *"sẵn có"*
  bằng một `UPDATE` trực tiếp, đúng tiền lệ chữ ký #8(a) của Story 2.6 cho `retired_at`.
  ⚠️ **Cái mất, ghi ra thay vì giấu:** phép phân xử được nghiệm thu, còn **đường sinh ra dữ liệu
  vào** thì chưa — một cơ chế tương lai quên vế (b) của AD-47 ① *(đặt xuất xứ cùng lúc với mốc)*
  sẽ không làm ca nào ở đây đỏ. **Chủ: Epic 6 (FR115) · Epic 7 (FR58)** — hai cơ chế đầu tiên
  thật sự ghi một xuất xứ khác `self`.

- 🔴 **Ba hàng còn lại của bảng AD-47 ③ chưa có mã, mỗi hàng một chủ.** Story 2.7 khai **giá
  trị** chúng sẽ dùng (`TRANSLATION_ORIGIN_*`) để bốn Epic sau không tự đặt tên riêng, nhưng nó
  **không cài** một đường ghi nào trong số đó:
  | Cơ chế | Xuất xứ AD-47 ③ giao | Chủ |
  |---|---|---|
  | Nhập song ngữ (FR115) | *nhập từ tài liệu song ngữ* | Epic 6 |
  | Chấp nhận thay đổi Review Mode (FR94) | *người khác dịch* | Epic 8 |
  | Điền sẵn từ TM khớp 100% (FR58) | xuất xứ của cặp TM nguồn | Story 7.4 |
  | Đưa đề xuất AI sang Editor | *người khác dịch* | Epic 4 |
  | Gộp/tách segment (AD-47 ④) | đồng ý ⇒ giữ · bất đồng ⇒ *người khác dịch* | ✅ Story 2.8 |

  → ✅ **HÀNG "Gộp/tách segment" ĐÃ ĐÓNG 2026-08-17 (Story 2.8).** `regroup::merged_origin`
  cài AD-47 ④ nguyên văn *(đồng ý ⇒ giữ · bất đồng ⇒ `other`)*, và `split_at` cho mọi mảnh
  **không phải mảnh đầu** một xuất xứ rỗng — một suy dẫn, không một luật mới: mảnh đó chưa có
  bản dịch nào để khai. Bốn hàng còn lại của bảng **vẫn hở**, giữ nguyên chủ.
  ⚠️ Đóng ở lượt **code review** 2026-08-17, không ở lượt dev.
  🔴 Mỗi chủ phải làm **cả hai** vế của AD-47 ①: đặt **mốc** *và* đặt **xuất xứ**, trong cùng
  một thao tác. Quên vế xuất xứ ⇒ lượt xác nhận kế tiếp ghi *tôi dịch* cho chữ người dùng chưa
  gõ, và **không cổng nào đỏ**.

- 🟡 **Khôi phục (FR101) trả văn bản về mà KHÔNG trả xuất xứ về — AD-47 ⑤, ngoại lệ CÓ TÊN.**
  Hệ quả bắt buộc của chữ ký #1(a): `segment_version` không mang xuất xứ nên **không có gì để
  trả về**. ⇒ Khôi phục văn bản của một phiên bản cũ rồi xác nhận **không sửa** ⇒ giữ nguyên
  xuất xứ **hiện tại**, thứ có thể thuộc về một phiên bản khác.
  ⚠️ **Cùng gốc** với món nợ bốn nhãn của Story 2.6 và đóng **cùng lúc** với nó. **Chủ: story
  nào cho `segment_version` một cột xuất xứ.** 🔴 Story 2.7 **không** được chấm đạt vế này —
  AD-47 ⑤ nói bằng chữ.

- ⚠️ **Rust TIN mốc do webview khai, và không cổng nào ở tầng Rust bắt được một mốc sai.** Đây
  là cái giá tường minh của chữ ký #2(b), và nó là **cùng một lỗ** với món nợ đã ghi ở
  *"story nào dựng bề mặt xác nhận thứ hai"* — nhưng **nặng thêm một bậc**: bản trước, một bề
  mặt tương lai gọi `invoke('confirm_segment')` thẳng chỉ ký một **văn bản cũ**; nay nó ký thêm
  một **xuất xứ sai**, và xuất xứ sai đi vĩnh viễn vào kho TM của Epic 7 chứ không sửa được ở
  lượt gõ kế tiếp. Lưới duy nhất là ba ca ④ của `tests/frontend/editorConfirmSegment.test.ts`,
  và chúng canh **một chỗ gọi**, không canh mọi chỗ gọi tương lai.
  **Chủ: story nào dựng bề mặt xác nhận thứ hai** *(nối tiếp món nợ cũ, không mở món mới)*.
  🔵 **ĐO ĐƯỢC 2026-08-16, và phép đo THU HẸP món nợ này thay vì nới nó.** Bề mặt thứ hai đó
  **đã tồn tại**: `e2e/specs/segment-history-restore.e2e.mjs` gọi `invoke('confirm_segment')`
  thẳng. Lượt đổi hình dạng dây của story này làm nó **ĐỎ** — nguyên văn *"invalid args
  `textAtLoad` for command `confirm_segment`: … missing required key textAtLoad"* — trong khi
  **382 ca Rust và 133 ca vitest đều xanh**. ⇒ Hai mệnh đề đọc ra được: (1) đường e2e là lưới
  **duy nhất** cho hình dạng dây, đúng khuôn vụ cột `status` của Story 2.5; (2) Tauri từ chối
  một tham số **thiếu** một cách ồn ào chứ **không** âm thầm cấp giá trị mặc định — nếu nó im
  lặng cấp `""` thì lượt này đã **XANH** trong khi mọi câu duyệt-nguyên-văn bị gắn nhãn *tôi
  dịch*. ⚠️ Vế **vẫn hở** là một chỗ gọi truyền một mốc **SAI KIỂU ĐÚNG** *(một chuỗi hợp lệ
  nhưng không phải bản lúc nạp)* — Tauri không có gì để nói ở đó, và không cổng nào bắt được.

- ⚠️ **AC7 là một mệnh đề PHỦ ĐỊNH và nó được giữ bằng KỶ LUẬT, không bằng một cổng.** *"Không
  thao tác nào thêm — hệ thống không hỏi"*: Story 2.7 không thêm command, không khoá `vi.json`,
  không hàng nào vào bảng phím `EXPERIENCE.md`, và `COMMAND_FLOOR` **không đổi** *(sàn 37, cổng
  in 44 — y baseline)*. Nhưng không cổng nào phát biểu *"xuất xứ không được có bề mặt"*: một
  story sau thêm một hộp thoại hỏi xuất xứ sẽ đi qua cả mười một cổng.
  ⚠️ Ghi ra thay vì dựng một cổng cho nó: một cổng canh *"không có X"* trên một khái niệm chưa
  có tên trong mã là một cổng không đỏ được, và luật của kho là **mỗi cổng phải đỏ được**.
  **Chủ: Ice** *(một câu hỏi quy ước, cùng hạng với món adapter ở trên)*.

- 🔵 **ĐÃ CHẨN ĐOÁN 2026-08-16 (Story 2.7) — và phép đo ĐẶT TÊN cho một phần của hai món nợ
  *"bộ e2e chập chờn"* ở trên.** Năm lượt trọn bộ, ghi cả năm:

  | # | Cây nguồn | Máy | Kết quả |
  |---|---|---|---|
  | ① | Story 2.7 | **bận** *(`cargo test` song song)* | 7/8 — `segment-history-restore`, **đỏ THẬT**, đã vá |
  | ② ③ | Story 2.7 | **bận** | 7/8 — `editor-typing-flush` `:184` rồi `:293` |
  | ④ | baseline `5a7e007` | **rảnh** | **8/8** (10m30) |
  | ⑤ | Story 2.7 | **rảnh** | **8/8** (9m28) |

  ⇒ **Biến là TẢI MÁY, không phải cây nguồn.** `FLUSH_WAIT_MS = 3.500 ms` trong khi idle của
  AD-35 là **2.000 ms** — biên chỉ **1.500 ms**, và trong biên đó phải lọt: timer idle · một
  lượt `invoke` · `Store::write` **nối tiếp** của AD-11 · một lượt fsync WAL. Một máy đang biên
  dịch Rust ăn hết biên đó. Hai lượt đỏ rơi vào hai phép khẳng định khác nhau nhưng **cùng một
  triệu chứng**: *"chữ chưa tới đĩa sau `browser.pause`"*.

  🔴 **Đây cũng là lý do những lượt đỏ cũ *"không tái lập được"*:** người chẩn đoán chạy lại
  trên một máy đã rảnh. Món nợ `:3345-3354` và `:3117-3139` vì thế **không sai**, nhưng chúng
  thiếu đúng biến này.

  ⚠️ **Bản vá KHÔNG phải nới hằng số** — nới một ngưỡng cho hết đỏ là đúng thứ
  `project-context.md` §*Miễn trừ và cảnh báo* cấm, và nó chỉ dời điểm gãy. Đường đúng: **chờ
  một SỰ KIỆN** *(mốc `editorLastFlushAt` đổi, hoặc một `waitUntil` trên chính đĩa)* thay vì chờ
  một khoảng thời gian — cùng luật *"hàm nhịp ghi không tự đọc `Date.now()`"* mà `writeSchedule`
  đã đi qua một lần. **Chủ: một story hạ tầng e2e** *(nối tiếp hai món cũ, không mở món thứ ba)*.
  → ✅ **ĐÃ ĐÓNG 2026-08-18 (Story 2.12 · AC4), và đóng đúng đường mục này đã chỉ.** Mục viết
  *"chờ một SỰ KIỆN (mốc `editorLastFlushAt` đổi …) thay vì chờ một khoảng thời gian"* — bản vá đi
  đúng đó, chỉ khác tên ô: mốc thật tên `editorLastSavedAt` (`editorPanelState.ts:246`), **đã là
  một export công khai** vì `StatusBar.vue` đọc chính nó để dựng câu *"Đã lưu N giây trước"*.
  ⇒ `e2e/support/flushWait.mjs`, hợp đồng **hai bước**: `markFlushBaseline()` trước khi gõ,
  `waitForFlushAfter(baseline)` sau. Hai chỗ `pause(FLUSH_WAIT_MS)` ở `editor-typing-flush` đã
  thay; hằng `FLUSH_WAIT_MS = 3_500` **đã gỡ khỏi mã**, không nới.
  🔴 **Và hằng AD-35 KHÔNG đổi một chữ số** — `git diff` trên `src/panels/editorFlush.ts` và
  `src/layout/writeSchedule.ts` là **RỖNG**: `EDITOR_IDLE_MS = 2000`, `EDITOR_HARD_CAP_MS = 5000`.
  ⚠️ **Bẫy đã cắn ở lượt dựng, ghi ra:** mọi ca trong một tệp spec dùng **chung một phiên app**,
  nên ca thứ hai bắt đầu với mốc đã khác `null` — một phép chờ *"tới khi có mốc lưu"* trả về **ngay
  lập tức** và không đo gì cả. Vế `markFlushBaseline()` tồn tại đúng vì thế.
  🔴 **CÒN HỞ, chuyển thành nợ mới có chủ:** vế *"đo lại trên MÁY BẬN"* (Task 4.4) **chưa chạy** —
  xem món `AC7 · Task 8.4` mới ở cuối tệp này.

  ⚠️ **Bài học phương pháp, ghi vì dev vừa mắc:** đừng chạy việc nặng song song với một bộ đo
  **thời gian thực trên engine thật**. Bộ đo đó không phân biệt được *"sản phẩm hỏng"* với *"máy
  bận"*, nên mọi lượt đỏ nó cho đều dẫn tới một cuộc chẩn đoán vô ích.

## Deferred from: code review of 2-7-xuat-xu-ban-dich-cap-segment (2026-08-16)

- ⚠️ **Không cổng TỰ ĐỘNG nào canh hình dạng dây `textAtLoad` ↔ `text_at_load`** — Blind Hunter
  nêu độc lập, và phép rà xác nhận: khuôn hai lớp làm hàm thuần `confirm_segment`
  (`segment.rs:1407`) được `cargo test` gọi thẳng, **bỏ qua** `wire::confirm_segment`
  (`:2021`, cần `AppHandle` nên `tests/**` không gọi được). ⇒ Lượt đổi hình dạng dây của story
  này đi qua **382 ca Rust + 133 ca vitest đều xanh** và chỉ đỏ ở một spec e2e — mà e2e **cố ý
  nằm ngoài `pre-push`** (`project-context.md` §*Hai thứ CỐ Ý nằm ngoài pre-push*).
  🔵 **KHÔNG mở món mới** — đây là cùng một lỗ với món *"Rust TIN mốc do webview khai"* ngay
  trên (mục Story 2.7) và với vụ cột `status` của Story 2.5. Ghi ở đây vì lượt rà làm rõ **hình
  dạng** của nó: cái hở không phải *"thiếu một ca test"* mà là *"tầng `wire` không có đường
  nghiệm thu tự động nào"*, nên một ca test thêm vào `segment_contract.rs` **không đóng được**.
  ⚠️ **Chưa đo:** liệu một ca ở tầng `tests/**` có đọc được danh sách tham số của
  `#[tauri::command]` mà không dựng webview hay không. Đó là phép đo đầu tiên chủ nó phải chạy.
  **Chủ: story nào dựng bề mặt xác nhận thứ hai** *(nối tiếp món cũ, không mở món thứ ba)*.

- 🔴 **Mốc ghim theo PHIÊN panel, xuất xứ đọc SỐNG từ đĩa — hai thứ lệch pha, và vòng ký thứ
  hai trong cùng phiên không trả lại được xuất xứ gốc.** Edge Case Hunter nêu; phép rà xác nhận
  đường đi **có thật về cấu trúc**: `editorPanelState.ts:795` lấy mốc từ `segments.value`, thứ
  ghim **lúc nạp** và không đường flush nào chạm — còn bước ③ ngay dưới **cố ý** chỉ vá `status`
  vào ảnh chụp, **không** chạm `target_text`. Trong khi đó `segment.rs:1493` đọc
  `translation_origin` **sống** trong chính giao dịch.
  ⇒ Đường đi: nạp một câu `target = A · origin = other` → sửa `A→B` *(flush hạ về `draft` ở
  `segment.rs:1873`)* → ký ⇒ `self` **đúng**, đĩa nay `B · self` → sửa ngược `B→A` → ký lại.
  Mốc **vẫn là `A`** *(chưa ai làm mới nó)*, văn bản **là `A`** ⇒ *"y hệt"* ⇒ giữ nguyên ⇒ giữ
  `self`. Nhưng đọc theo AC3 + AC5 *(hoàn tác về nguyên trạng ⇒ coi như không sửa ⇒ giữ xuất xứ
  **lúc nạp**)* thì câu trả lời đúng là `other`. Xuất xứ gốc bị xoá ở vòng ký thứ nhất và
  **không vòng hoàn tác nào lấy lại được**.
  ⚠️ **Đo được, và nó hạ mức món này xuống:** đường **chưa tới được hôm nay** — tập giá trị thật
  trên đĩa là `{'', 'self'}`, mà cả hai đều cho `self` ở mọi nhánh. Nó kích hoạt câm lặng đúng
  vào Epic đầu tiên ghi `other`/`bilingual_import`.
  🔴 **Câu hỏi phải trả lời trước khi sửa, không phải một dòng mã:** *"xuất xứ lúc nạp"* nghĩa
  là lúc nạp **phiên panel**, hay lúc bắt đầu **vòng draft hiện tại**? Hai cách đọc đều đứng
  được, và chúng cho hai kết quả khác nhau ở đúng ca trên. **Chủ: Epic đầu tiên sinh ra một xuất
  xứ phi-`self`** *(Epic 6 FR115 hoặc Epic 4, tuỳ cái nào tới trước)*.

- ⚠️ **Vế ĐỌC của danh mục đóng `translation_origin` không tồn tại** — phát hiện ở lượt rà
  2026-08-16, cả Blind Hunter lẫn Acceptance Auditor cùng chỉ vào nó độc lập. Doc-comment của
  `TRANSLATION_ORIGIN_BILINGUAL_IMPORT` khai một hàm `is_translation_origin` *"phải nhận"* giá
  trị lạ; `grep` toàn kho: **0 kết quả**, hàm đó chưa từng được viết. Chú thích **đã sửa tại
  chỗ** (`segment.rs:1204`), nhưng khoảng hở nó mô tả thì còn: `TRANSLATION_ORIGINS` chỉ có một
  chỗ đọc là **một ca test**, nên nó canh *giá trị khai trong mã nguồn*, không canh *giá trị đi
  vào cột*. Một `.atproj` do một bản tương lai ghi mang giá trị thứ năm đi qua sạch.
  🔴 Đóng nó **không** phải thêm một hàm: một phép kiểm lúc chạy phải khai trước **nó làm gì khi
  gặp giá trị lạ** — từ chối mở *(cùng khuôn "lược đồ mới hơn ⇒ không bao giờ ghi vào")*, hạ về
  `''`, hay báo lỗi. Đó là một quyết định lược đồ. **Chủ: Ice.**
  ⚠️ Kèm một mệnh đề đo được, ghi vì nó rộng hơn món này: kho **không** chạy `cargo doc` với
  `rustdoc::broken_intra_doc_links` ở bất kỳ đâu trong 11 cổng · CI · `pre-push` — nên **mọi**
  liên kết intra-doc gãy trong kho hôm nay đều im lặng. Chính lỗi vừa sửa là ca đầu tiên.

- 🟡 **Chuẩn hoá Unicode (NFC/NFD) chưa phủ ở phép so mốc FR117.** Chữ ký thứ mười của Ice
  (2026-08-16) đưa `trim()` vào cả hai vế của `segment.rs:1493`, và nó phủ khoảng trắng bao
  ngoài — **không** phủ hai chuỗi trông giống hệt nhau trên màn hình nhưng khác nhau từng byte
  vì một bên dùng ký tự dựng sẵn còn bên kia dùng dấu kết hợp. Phủ nốt vế đó cần một phụ thuộc
  **MỚI** (`unicode-normalization`), nên nó phải đi qua **cửa rà giấy phép NFR15 ba bước** trước
  — không tiện tay cài. **Chủ: Ice** *(cùng hạng với các quyết định phụ thuộc khác)*.

---

## Story 2.8 — gộp và tách segment tường minh (2026-08-17)

- 🟡 **AC6 vế *"gộp một NHÓM"* đóng một nửa** — chữ ký #1(a) của Ice (2026-08-17) chốt gộp
  **đúng hai**: câu đang có caret và câu liền trên. Tầng thuần đã sẵn sàng cho `n` bất kỳ
  (`core::segment::regroup::merge` nhận một lát cắt, `paragraph::merged` cũng vậy), nên phần
  còn thiếu là **bề mặt chọn nhiều hàng** — thứ không tồn tại trong kho
  (`editorPanelState.ts:57` là một `Ref<number | null>`) và **không tài liệu nào của dự án mô
  tả cơ chế chọn**: không Shift+click, không kéo chọn, không Shift+mũi tên ở PRD, epics,
  `EXPERIENCE.md` hay `DESIGN.md`.
  ⚠️ Đây là **lượt lặp lại thứ hai** của đúng một câu hỏi — Quyết định #1 của Story 2.5c hỏi y
  hệt cho *"một dải câu"* của FR133 và Ice cũng ký *"một câu"*, ghi nợ với chủ là *"ứng viên tự
  nhiên là 2.8"* (`:3397-3424`). Nay 2.8 đã đi qua và **không** mở nó ⇒ món nợ **đổi chủ**, chứ
  không đóng. **Chủ: một story sau của Epic 2 dựng cơ chế chọn nhiều hàng** — và story đó phải
  đóng **cả hai** món cùng lượt.

- 🔴 **`⌘/` có thể là một phím tắt CHẾT trên bàn phím thật, và bộ đo không phân biệt được.**
  Đo 2026-08-17 trong cửa sổ Tauri thật: `browser.keys(['Meta', '/'])` giao một `keydown` mang
  **`code: "/"`**, không `"Slash"` ⇒ hợp âm `Mod+Slash` **không khớp**, `defaultPrevented:
  false`, **0** command chạy. Đối chứng cùng lượt: `⌘M` giao `code: "KeyM"` ⇒ khớp.
  ⚠️ **Ice thử tay 2026-08-17 và báo *"đã thử ⌘/ nhưng không có gì xảy ra"*.** Số đó **không
  tách được hai khả năng**, và phải ghi ra đúng mức: lượt thử ấy diễn ra khi mã còn mang khuyết
  tật `caretPositionFromPoint` (xem mục dưới), nên *"không có gì xảy ra"* cũng là triệu chứng
  đúng của **thiếu điểm cắt**, không riêng của một hợp âm chết.
  🔴 Hai đường đóng, và **không** đường nào là "nới hằng cho hết đỏ": ① một lượt gõ `⌘/` bằng
  tay **sau bản vá này**, trên một câu đã bấm vào cột nguyên văn; ② nếu vẫn câm thì `keys.ts`
  phải chấp cả hai `code` — và đó là một lượt nới **danh mục đóng**, tức một quyết định.
  **Chủ: Ice.**

- 🟡 **AC5 (*"cặp TM đã ghi ở lại nguyên"*) đóng bằng CẤU TRÚC, không bằng một phép đo.**
  Bảng TM chưa tồn tại trong lược đồ, nên không đường sản phẩm nào đối chứng được. Thứ nói được
  hôm nay: **không câu SQL nào** của `merge_segments`/`split_segment` chạm một bảng ngoài
  `segment`. **Chủ: Epic 7** — nghiệm thu lại cùng lượt bảng TM ra đời.

- 🟡 **Luật `is_omitted` khi gộp (chữ ký #5(a)) chưa có chỗ đứng trong spine.** Ice phán định
  2026-08-17 rằng *"bất kỳ mảnh nào đã cắt ⇒ segment mới đã cắt"* nằm **trong biên độ AD-5** và
  **không** cần một `AD` mới ⇒ cửa chặn Task 0.4 không kích hoạt. Nhưng hôm nay nguồn **duy
  nhất** phát biểu luật ấy là một ca test
  (`merging_carries_the_omitted_flag_from_any_piece_not_from_all_of_them`) cộng một doc-comment.
  ⚠️ Và nó **ngược chiều** AD-47 ④ ở ca bất đồng — 47 ④ chọn chiều bi quan cho một **nhãn**, luật
  này chọn chiều an toàn cho một **quyết định của người dùng**. Hai cột cùng một ca mà khác
  chiều là đúng thứ một story sau sẽ đọc nhầm. **Chủ: Ice** *(một dòng trong AD-5, hoặc một mục
  của AD kế tiếp chạm segment)*.

- ✅ **~~Lưới phình theo số lần sửa, VĨNH VIỄN~~ → ĐÃ ĐÓNG 2026-08-17 (Story 2.8), bằng một
  lượt DÙNG THẬT.** Chữ ký #6(b) giữ hàng về hưu **ở lại trong lưới** với vạch `ornament`, và
  món nợ này ghi đúng cái giá của nó. Ice **lật** chữ ký ấy cùng ngày, sau khi dùng: *"đã tách
  ra 2 câu, nhưng câu cũ vẫn tồn tại và số thứ tự vẫn chiếm, gây rối nội dung"*.
  ⇒ Đóng bằng `WHERE retired_at IS NULL` ở `read_open_chapter_segments` + `applyRegroup` gỡ
  hàng về hưu khỏi ảnh chụp. **Lọc khỏi LƯỚI, không xoá khỏi ĐĨA** — AC4 còn nguyên, và một ca
  hợp đồng khoá cả hai vế (lưới 3 → 2, đĩa 3 → 4).
  🔴 **Bài học giữ lại, vì nó rộng hơn món nợ:** cái giá này **đã được viết ra bằng chữ TRƯỚC
  KHI KÝ** và vẫn không đủ để thấy. Ba lý lẽ đứng sau #6(b) đều **vẫn đúng** hôm nay; cả ba
  cộng lại thua một lượt người thật nhìn vào một Chương thật.

- 🟡 **Nhánh `'ornament'` của `resolveSegmentRule` KHÔNG CÒN ĐƯỜNG TỚI** — hệ quả trực tiếp của
  lượt lật ngay trên. Nó **không** bị gỡ, và đó là một lựa chọn có lý do: `ornament` *"mờ đã về
  hưu"* là **một trong sáu** giá trị vạch mà UX-DR19 (`epics.md:555`) khai, nên gỡ nó khỏi mã
  là làm mã lệch một UX-DR **đang đứng** — `project-context.md:456-458` cấm sửa spec cho khớp
  mã.
  ⇒ Hai đường đóng, và cả hai là **một quyết định**, không một lượt dọn: ① một bề mặt nào đó
  *(lịch sử? điều hướng tới chỗ đánh dấu FR119?)* cho hàng về hưu một chỗ hiện, và nhánh này
  sống lại; ② UX-DR19 rút xuống năm giá trị, và **đó là một lượt sửa spec** phải đi qua thủ tục
  của nó. **Chủ: Ice.**

- ⚠️ **`⌘M` sẽ va Quản lý TM ở Epic 7.** `mockups/tm-manage.html:128` dùng `⌘M` mở màn hình
  Quản lý TM; Story 2.8 vừa đăng ký `⌘M` cho `editor.merge_segments`. Va chạm **chưa xảy ra**
  *(Quản lý TM là Epic 7)*, và tài liệu **chưa từng gọi tên nó** — trong khi xung đột `⌘⇧T` thì
  `mockups/settings.html:274-275` đã đánh dấu bằng `class="conflict"`. `conflictFor` chạy trên
  **toàn registry** nên nó sẽ đỏ ở `register()` chứ không im lặng — nhưng nó đỏ ở một story
  không hiểu vì sao. **Chủ: Epic 7.**

- ⚠️ **Phép ánh xạ *"offset → chỉ số ký tự"* chưa đo với Hán Việt BẬT.**
  `editorSegments.ts::sourceCutOffsetOf` cộng dồn độ dài mọi text node đứng trước, nên nó
  **đúng theo cấu trúc** cho cả hai trạng thái. Nhưng số đo nền
  *(`2-8-ban-do/README.md` bước ⓪: `soPhanTuCon = 0`, ba text node `[0, 40, 0]`)* lấy với Hán
  Việt **TẮT**. Bật lên thì ô mang thêm `<ruby>`/`<rt>`, và `<rt>` mang `user-select: none`
  (`SourceHanViet.vue:980`) — một biến chưa ai đo ở đường **caret**, khác đường **vùng chọn**.
  **Chủ: story đầu tiên chạm lại đường tách**, hoặc một lượt đo tay của Ice.

- 🔴 **Bộ e2e KHÔNG giao được một cú bấm chuột tới cột nguyên văn** — đo 2026-08-17, ba cách
  nhắm *(toạ độ tuyệt đối · `origin` + lệch · `origin` trần)*, và **cùng lệnh đó trên ô
  `[data-col="tgt"]` thì ăn**. ⇒ Spec `segment-merge-split.e2e.mjs` phải bắn một `MouseEvent`
  **tổng hợp** lên ô, với toạ độ lấy từ hộp dòng thật.
  ⚠️ **Vế KHÔNG được phủ, và phải nói ra:** *"một cú bấm CHUỘT THẬT vào cột nguyên văn có tới
  được `onSourceCellMouseUp` không"*. **Chủ: Ice** *(một lượt bấm tay)*, hoặc **story hạ tầng
  e2e** nếu nó tìm ra cách lái con trỏ tới một vùng không soạn thảo được.
  🔵 Gộp vào cùng chủ với hai món đã có: `devServerIsUp()` tin một Vite hấp hối (`:3345-3354`)
  và `FLUSH_WAIT_MS` thua một máy đang biên dịch (`:3902-3906`).

- 🔴 **Auto-Lookup bằng chuột ở cột nguồn CHƯA CÓ đường nghiệm thu, và có thể đang chết.**
  Đo 2026-08-17 (`2-8-ban-do/README.md` vòng 2–3): trên WKWebView, **không cử chỉ chuột nào**
  tạo ra một vùng chọn ở cột nguyên văn — không caret từ một cú bấm, không `Range` từ một lượt
  kéo, kể cả sau khi tài liệu đã có tiêu điểm. Đối chứng ô bản dịch cho `"Caret"` ⇒ thước tốt.
  FR21 (Story 1.18, **đã phát hành**) dựa **toàn bộ** vào vùng chọn đó
  (`GridPanel.vue:309` → `attachSelectionWatcher` bắt `mouseup` → `currentSelectionText()`), và
  bộ e2e hôm nay **không có spec nào** cho nó.
  ⚠️ Ứng viên còn lại — *"driver không lái được máy chọn văn bản của WebKit trong nội dung
  không sửa được"* — **không loại trừ được bằng chính driver đó**, theo cấu tạo. Và món ngay
  trên vừa cho nó một chỗ dựa: driver cũng không giao được một cú bấm tới cột đó.
  🔴 **Ice chốt 2026-08-17: tách hẳn thành một story hạ tầng**, gộp với hai món *"bộ e2e chập
  chờn"*. **Chủ: story hạ tầng e2e** — và một lượt bôi đen bằng tay của Ice đóng được vế *"sản
  phẩm có hỏng không"* ngay hôm nay.

  → ✅ **VẾ *"SẢN PHẨM CÓ HỎNG KHÔNG"* ĐÃ ĐÓNG 2026-08-17 (Story 2.9) — bằng chữ ký của Ice,
  đúng cách mục này đã hẹn.** Ice xác nhận trên máy thật: **double-click vào một từ ở cột
  nguyên văn TRA ĐƯỢC**, kết quả hiện ở Panel Lookup.
  ⇒ **FR21 SỐNG.** Mệnh đề *"có thể đang chết"* hết đúng, và ứng viên còn lại được xác nhận:
  *"driver không lái được máy chọn văn bản của WebKit trong nội dung không sửa được"* là một
  giới hạn của **BỘ ĐO**, không của sản phẩm. Đây là lớp mệnh đề mà chữ ký của Ice **là** đường
  nghiệm thu duy nhất — cùng hạng với *"gõ tiếng Việt bằng bộ gõ"*.
  🟡 **Vế *"chưa có đường nghiệm thu"* thì CÒN MỞ** và giữ nguyên chủ *(story hạ tầng e2e)*:
  hôm nay vẫn **không spec nào** canh FR21 trên lưới, nên nó có thể chết lại trong im lặng.
  🔵 **Và món này vừa đẻ ra một khuyết tật THẬT, đã vá cùng ngày:** chính vì cột đó dùng chung
  `mouseup` với đường đánh dấu chỗ cắt của Story 2.8, **mỗi lượt tra một từ để ĐỌC cũng rơi một
  dấu cắt** — và một cú double-click *(hai `mouseup`)* rơi **hai** dấu. Ice tìm ra bằng cách
  dùng thật và ký lượt đổi cử chỉ sang `Mod`+click (Story 2.9, AC7).

- 🟡 **Khe thông điệp của `StatusBar` vẫn đóng, và `⌘Z` vẫn chưa có mô hình.** Chữ ký #9(a)
  (Ice, 2026-08-17) giữ 2.8 đúng phạm vi tám AC: **không** dòng báo hệ quả, **không** hoàn tác.
  ⚠️ Cái giá, ghi ra thay vì giấu: gộp là một thao tác **phá huỷ** *(hai câu biến khỏi chỗ cũ)*
  chạy **im lặng và không lui được**. `editorRegroupError` có tồn tại nhưng **chưa component
  nào đọc** — cùng khuôn `editorOmitError` đã ghi từ 2.5c.
  **Chủ: Story 2.9** cho dòng báo; **Ice** cho `⌘Z` *(chưa FR/AD/UX-DR nào chốt mô hình undo, và
  chọn một mô hình là một `AD` MỚI)*.

  → 🟡 **VẾ "DÒNG BÁO" ĐÃ ĐÓNG 2026-08-17 (Story 2.9); vế `⌘Z` CÒN MỞ.**
  Khe mở bằng một ô nhớ **thứ hai** (`regroupNotice`) cộng một `Record` **thứ hai**
  (`StatusBar.vue::REGROUP_NOTICE_KEYS`, sáu khoá) — **không** nới `CONFIRM_NOTICE_KEYS` thành
  `string`. Lý do viết tại chỗ: toàn bộ giá trị của bảng cũ là `vue-tsc` đỏ khi ai đó thêm một
  kết quả mà quên bảng, và nới nó gỡ đúng cái chốt ấy cho **cả hai** lượt.
  `editorRegroupError` nay **có người đọc** — nhánh `'refused'` đi qua `tError()`, nên câu từ
  chối là câu của **Rust**, không một bản chép ở frontend. Ca **thường nhất** của cử chỉ mới
  *(`Backspace` ở câu đầu Chương)* vì thế thôi im lặng; e2e khoá cả hai chiều
  (`segment-backspace-merge.e2e.mjs`).
  🔴 **Vế `⌘Z` giữ nguyên chủ là Ice**, và Story 2.9 đã soạn hồ sơ bàn giao cho nó:
  `planning-artifacts/ad-brief-2026-08-17-mo-hinh-hoan-tac.md` — hai đường (A)/(B) kèm hệ quả
  trên đĩa, ba mức phạm vi, tám ràng buộc cứng, sáu điều kiện nghiệm thu.
  → ✅ **ĐÃ ĐÓNG 2026-08-18 (SCP 2026-08-18b — AC5 RÚT).** Ice ký đường **(C)**: không dựng mô
  hình hoàn tác cho gộp/tách; đường quay lại là **gọi lại chính lệnh gộp/tách**. Cách đóng: đo
  được rằng **không FR nào đòi hoàn tác** *(`grep` trên `prd.md` chỉ trúng `undock`)*, và lời hứa
  `⌘Z` tồn tại ở **đúng một câu văn xuôi** *(`EXPERIENCE.md:171`)* vốn là **mẩu sót** của lượt sửa
  2026-08-17 — mâu thuẫn với `:169` trong cùng tệp. `EXPERIENCE.md:171` và AC5 ở `epics.md` đã sửa.
  ⚠️ **Chỗ hở mới, đã mở một mục riêng ở cuối tệp:** bấm `⌘Z` không phản hồi gì.

- ⚠️ **`ord` trong ảnh chụp webview thành CŨ sau một lượt gộp/tách.** Chữ ký #7(a) đánh lại
  `ord` **liên tục 1..N cho cả Chương** trong Rust, còn `applyRegroup` chỉ vá những hàng bị
  chạm. Hôm nay vô hại **theo một phép đo, không theo một lập luận**: `grep '\.ord'` trên
  `src/**` cho **0** chỗ đọc — lưới đánh số hàng bằng **chỉ số mảng** và thứ tự đọc là thứ tự
  mảng. **Chủ: story đầu tiên đọc `segment.ord` ở webview** — nó phải chọn giữa vá đủ hoặc nạp
  lại Chương (đường #4(b), đã bị loại vì nạp 9.850 hàng cho một thao tác sửa một chỗ).

---

## Deferred from: code review of story-2.8 (2026-08-17)

*(Lượt rà ba tầng — Blind Hunter · Edge Case Hunter · Acceptance Auditor. Mười phát hiện còn
lại sau phân loại: hai quyết định Ice chốt tại chỗ, tám bản vá. Các món dưới đây là thứ **lượt
vá sinh ra hoặc không đóng được**, mỗi món một chủ.)*

- 🟡 **Dấu điểm cắt KHÔNG vẽ được ở chế độ Hán Việt.** Cơ chế tích luỹ *(Ice ký 2026-08-17 cho
  AC7)* vẽ mỗi ranh giới một dấu ở đường **chữ trần** — `GridPanel.vue::sourcePiecesOf` cộng
  `.cut-mark`. Ở chế độ Hán Việt ô do `SourceHanViet.vue` dựng và chỗ cắt rơi vào **giữa các
  `<ruby>`**; cắm dấu vào đó là chẻ một `<ruby>` làm đôi, thứ vừa sai ngữ nghĩa vừa đụng hợp
  đồng vùng chọn của Auto-Lookup (`hanVietSurfaces.ts`). ⇒ Ở chế độ đó ô chỉ nhận **viền
  `has-cuts`**: người dùng biết *"câu này đang có điểm chờ"* và biết **bao nhiêu điểm**
  (`data-cut-count`), nhưng **không thấy chúng ở đâu**. 🔴 Phép **ánh xạ** thì đã đúng ở cả hai
  chế độ *(`<rt>` bị loại khỏi phép đếm, hai ca vitest ③b/③d khoá)* — hở là vế **hiển thị**,
  không vế đúng-sai. **Chủ: một story sau của Epic 2 có động tới lưới Hán Việt.**

- 🟡 **`⌘Z` cho một lượt tách ĐA-MẢNH — cùng món nợ cũ, nhưng cái giá đã lớn hơn.** Quyết định
  #9(a) *(Ice ký 2026-08-17)* chốt 2.8 không dựng undo, chủ **Story 2.9**. Lượt đa-mảnh
  **không đổi** quyết định đó nhưng đổi hậu quả: một cú `⌘/` nay phá **một** hàng thành `n`
  hàng, `n` không chặn trên. Cộng với việc không có dòng báo hệ quả *(cũng #9(a))*, một lượt
  bấm nhầm bốn chỗ rồi `⌘/` là **năm** hàng mới, im lặng, không lui được. ⚠️ Đường lui **duy
  nhất** hôm nay: bấm trùng một điểm đã có thì **gỡ** nó ra — đã cài, nhưng nó chỉ lui được
  **trước** khi bấm `⌘/`. **Chủ: Story 2.9** *(cùng chủ với dòng báo và `⌘Z`)*.
  → 🟡 **ĐÓNG MỘT NỬA 2026-08-18 (SCP 2026-08-18b).** Vế *"chờ `⌘Z` gỡ giúp"* **CHẾT** — Ice ký
  đường (C), không có `⌘Z`. Đường lui chính thức nay là **gọi lại lệnh gộp**, và với một lượt tách
  `n` mảnh đó là **n−1 lượt gộp bằng tay** — đắt, nhưng đo được và **không mất dữ liệu**.
  🔴 **CÒN HỞ:** một lượt `⌘/` đa-mảnh vẫn **im lặng về cái giá đó TRƯỚC khi chạy**. **Chủ: Ice**
  *(cùng chủ với chỗ hở `⌘Z` ở mục cuối tệp — một quyết định về dòng báo phủ được cả hai)*.

- 🔴 **Luật `is_omitted` khi gộp (chữ ký #5(a)) VẪN chưa có chỗ đứng trong spine.** Món này đã
  ghi ở lượt dev; nhắc lại ở đây vì lượt rà xác nhận nó là mệnh đề **duy nhất** của story mà
  nguồn phát biểu là một ca test, không một `AD`. Ice phán định 2026-08-17 rằng nó nằm trong
  biên độ AD-5 và Task 0.4 không kích hoạt. ⚠️ Và lượt rà thêm một số cho hồ sơ: `is_omitted`
  *(bất kỳ ⇒ cắt)* và `translation_origin` *(bất đồng ⇒ `other`)* là **hai trường cạnh nhau
  trong cùng một struct giải "bất đồng" NGƯỢC CHIỀU nhau**. Cả hai đều đúng theo chữ ký; cái
  thiếu là một chỗ viết ra **vì sao** hai chiều khác nhau, ở nơi story sau sẽ đọc. **Chủ: Ice**
  *(một `AD` mới, hoặc một dòng trong AD-5)*.

- ⚠️ **Ca e2e đa-mảnh mang y nguyên HAI giới hạn của bàn đo, không thêm và không bớt.**
  `segment-merge-split.e2e.mjs` ca *"tách BA mảnh"* dùng một `MouseEvent` tổng hợp *(bộ đo
  không giao được cú bấm tới cột nguyên văn qua **ba** cách nhắm)* và một `KeyboardEvent` mang
  `code: 'Slash'` *(`browser.keys(['Meta','/'])` giao `code: "/"`)*. ⇒ Hai vế **chưa được phủ**
  vẫn là hai vế cũ: *"chuột thật có tới `onSourceCellMouseUp` không"* và *"WKWebView thật báo
  `code` gì cho phím gạch chéo"*. **Chủ: Ice** *(một lượt bấm và gõ tay)* — không phải một món
  mới, nhưng ca mới **không** thu hẹp nó.

- ⚠️ **`split_at` cắt theo code point, không theo CỤM CHỮ CÁI.** Một chỗ cắt giữa một ký tự cơ
  sở và một dấu tổ hợp *(chuỗi NFD)* cho hai mảnh "hợp lệ" mà mảnh sau mở đầu bằng một dấu mồ
  côi. **Không với tới hôm nay**: nguồn duy nhất của `cut` là caret của WebKit, và caret không
  đậu giữa một cụm. Ghi ra vì `split_at` là `pub` và một chỗ gọi thứ hai sẽ không có hàng rào
  đó. **Chủ: story nào cho `split_at` một chỗ gọi không đến từ caret.**

- ⚠️ **`ORDER BY ord` thiếu khoá phụ: đã vá MỘT chỗ, chưa rà HẾT kho.** Lượt rà vá
  `read_open_chapter_segments` (`ORDER BY ord, id`) sau khi thấy hai truy vấn khác của cùng
  story đã có khoá phụ còn nó thì không. **Chưa ai đếm** còn bao nhiêu `ORDER BY` trên một cột
  không `UNIQUE` ở phần còn lại của `commands/**`. `ord` cố ý không `UNIQUE` (`schema.rs:279-282`)
  nên đây là một lớp, không một ca. **Chủ: một story hạ tầng, hoặc một cổng tĩnh mới.**

- 🔵 **Quan sát MỚI cho món nợ *"bộ e2e chập chờn"* (2026-08-17, lượt code review 2.8).**
  `editor-typing-flush.e2e.mjs:184` đỏ trong bộ **trọn bộ** *(nhận `""` thay vì chữ vừa gõ)*
  và xanh **2/2** khi chạy riêng — khuôn đã biết. Cái **mới**: trước ca đỏ có **hơn 20** dòng
  `WARN tauri-service:window: Failed to get window states: Error: Tauri core.invoke not
  available after 5s timeout`. ⇒ Cầu IPC của **bàn đo** không lên trong suốt spec đó, một ứng
  viên chưa từng nêu trong hai món nợ cũ *(`devServerIsUp` tin một Vite hấp hối · `FLUSH_WAIT_MS`
  thua một máy đang biên dịch)* và cũng khác biểu hiện của lượt ② lượt dev
  *(`Couldn't find element for "pointerMove"`)*. 🔴 **Ba biểu hiện, một khuôn *"xanh riêng, đỏ
  trong bộ"*** — chưa ai đặt tên nguyên nhân, và luật sau Story 1.22 cấm chấm "đã chẩn đoán"
  khi mới có triệu chứng. **Chủ: story hạ tầng e2e** *(cùng chủ với hai món trên)*.
  → 🟡 **ĐÓNG MỘT NỬA 2026-08-18 (Story 2.12).** ~~Hai trong bốn nguyên nhân đã nêu tên trong món này
  còn là giả thuyết~~ — nay **hai** đã có bản vá đo được: `devServerIsUp` tin một Vite hấp hối
  *(AC1)* và fixture không dọn state panel *(AC2)*. Cộng hai nguyên nhân nữa của F3: khuôn
  `waitForExist` *(AC3)* và biên `FLUSH_WAIT_MS` *(AC4)*.
  🔴 **PHẦN CÒN HỞ, và nó đúng là phần mục này nói tới:** *"vì sao `core.invoke` của **bàn đo** không
  lên trong suốt một spec"* — hơn 20 dòng `Failed to get window states … timeout 5s` — **vẫn chưa ai
  đặt tên nguyên nhân**. Bốn bản vá trên **không** chạm giả thuyết đó, và luật sau Story 1.22 vẫn
  cấm chấm *"đã chẩn đoán"* khi mới có triệu chứng.
  ⚠️ Vế nghiệm thu *(một lượt trọn bộ tái lập được)* chưa chạy — xem món `AC7 · Task 8.4` ở cuối tệp.
  **Chủ: giữ nguyên — story hạ tầng e2e kế tiếp.**

## Deferred from: 2-9-gop-bang-backspace-dau-o (2026-08-17)

- 🔴 **`AC5` (`⌘Z`) chưa có mô hình, và chọn một mô hình là `AD-48`.** Ice ký 2026-08-17 chữ
  ký ① *(giao 5/6 AC, ghi nợ AC5)* — nên story này đóng AC1·AC2·AC3·AC4·AC6 và **không viết một
  dòng mã nào** của AC5. Đo lại từ nguồn cùng ngày: `grep -rniE "undo|redo|UndoManager"` trên
  `src/` + `src-tauri/src/` cho **0 cơ chế** *(8 dòng trúng đều là chữ `dock`/`undock` của
  dockview cộng một chú thích)*; `grep -rn "KeyZ" src/commands/` cho **0**; bảng Phím của
  `EXPERIENCE.md:261-268` **không có hàng `⌘Z`** *(chữ "hoàn tác bằng `⌘Z`" chỉ có trong VĂN
  XUÔI của UX-DR32 — một **lời hứa**, không một mô hình)*; `grep -c "^### AD-"` = **47**.
  🔴 Hai đường cài đặt đều **hỏng vĩnh viễn dữ liệu người dùng theo hai kiểu khác nhau**, và
  cả hai biên dịch sạch qua mười một cổng: **(A)** gỡ `retired_at` + xoá hàng mới ⇒ `segment.id`
  cũ **sống lại**, đụng thẳng AD-3 *("bất biến, không tái dùng sau khi về hưu")* và đòi một
  **năng lực ghi chưa từng tồn tại** *(đo: `retired_at` đọc ở sáu chỗ, ĐẶT ở đúng một —
  `write_regroup`; `core/i18n/mod.rs:215` ghi bằng chữ "chỉ đặt được bằng SQL")*; **(B)** một
  lượt tách mới ⇒ tuân thủ AD-5 hoàn hảo **và chính vì thế** mất dữ liệu: một `⌘Z` biến **một**
  chỗ đánh dấu FR119 thành **hai** chỗ mang ghi chú *"câu này đã đổi"*, cho một thao tác người
  dùng vừa **huỷ bỏ**.
  ⇒ Hồ sơ bàn giao đã soạn: `planning-artifacts/ad-brief-2026-08-17-mo-hinh-hoan-tac.md`.
  **Chủ: Ice** *(phán định phạm vi)* → **Winston** *(soạn `AD-48`)*.
  → 🟡 **ĐÓNG MỘT NỬA 2026-08-18 (SCP 2026-08-18b).** ✅ **Vế "Ice phán định" ĐÃ ĐÓNG:** Ice ký một
  đường **THỨ BA** mà hồ sơ chưa nêu — **(C) rút `⌘Z` cho gộp/tách**, AC5 rút khỏi `epics.md`. Cả
  (A) lẫn (B) bị loại; chi tiết và bốn phép đo ở §11 của hồ sơ.
  ⇒ **Không còn cần:** ngoại lệ AD-3 · đường `DELETE` đầu tiên trên nội dung người dùng · năng lực
  `retired_at → NULL` · bảng nhật ký · bước di trú 12. AD-3, AD-5, AD-31 **không đổi một chữ**.
  🔴 **CÒN HỞ — vế Winston:** `AD-48` **vẫn phải được soạn**, nhưng nay chỉ khai một mệnh đề *(Epic 2
  không có mô hình hoàn tác, và đây là lý do)*. Cần thế vì Epic 3+ còn thêm thao tác rời rạc; không
  viết ra thì câu hỏi quay lại mỗi epic. **Chủ: Winston.** 🔴 Dev **không** tự soạn `AD`.

- 🔴 **`@keydown` NAY MANG MỘT THAO TÁC THẬT ⇒ luật của `check:commands` Kiểm A phải được xem
  lại — và chính cổng đó đã dặn trước ngày này.** `scripts/check-commands.mjs:2348-2349` in ra
  **mỗi lượt chạy**: *"Kiểm A chỉ canh `@click`. `@keydown`/`@input`/`@submit` KHÔNG thuộc luật
  này; **ngày một `@keydown` mang thao tác thật xuất hiện, luật phải được xem lại**."*
  Trước story này `onEditKeydown` **không mang thao tác nào** — chỉ một chốt `isComposing`. Nay
  nó mang một thao tác **phá huỷ và không lui được** *(gộp segment, AD-5, và `⌘Z` đang là món nợ
  ngay trên)*.
  🔵 **Lượt cài đã giảm bề mặt xuống mức nhỏ nhất có thể mà không đợi một cổng mới:** nhánh
  `Backspace` **không** gọi thẳng `mergeCurrentSegment()` mà `dispatch('editor.merge_segments')`
  — một command **đã đăng ký**, tức cùng đường với `⌘M` ở một bậc **cao hơn** một lời gọi thẳng,
  và đúng AD-34 §1 cộng `project-context.md` *("một lời gọi thẳng dựng một đường thứ hai mà
  `check:commands` KHÔNG nhìn thấy")*. `COMMAND_FLOOR` **không đổi** — story không thêm command
  nào; nếu nó đổi thì đó là dấu hiệu đã đi sai đường.
  ⚠️ Nhưng **vẫn không cổng nào canh** rằng nhánh ấy `dispatch` chứ không tự cài đặt lại, và
  cũng không cổng nào đếm được *"còn `@keydown` nào khác mang thao tác"*. **Chủ: một story hạ
  tầng cổng** *(hoặc Ice, nếu muốn đóng ngay bằng một Kiểm mới)*.

- ⚠️ **Ba mệnh đề của cử chỉ `Backspace` KHÔNG đường nghiệm thu nào của dự án mô phỏng được —
  chữ ký của Ice là đường nghiệm thu duy nhất.** Đo 2026-08-17 (`2-9-ban-do/` §Vòng 1): **mọi**
  sự kiện WebDriver giao đều mang `isTrusted: false` *(cả `browser.keys` lẫn Actions API)*, và
  một sự kiện không tin cậy **không có default action** — đối chứng: caret ở **GIỮA** ô,
  `startOffset: 3`, `Backspace` qua driver cũng **không xoá một ký tự nào**.
  ⇒ ① `preventDefault()` có chặn nổi lượt xoá của một phím **thật** không *(rủi ro **thấp**: ở
  đúng offset 0 của một editing host WebKit không có gì để xoá lui — `2-9-ban-do/` §Ⓓ đo được
  `textContent` không đổi; dòng đó là lớp phòng **thứ hai**)*; ② auto-repeat của hệ điều hành
  *(chữ ký ③ — WebDriver `keyDown` giữ 600 ms cho **đúng một** `keydown`, `repeat: false`)*;
  ③ một lượt chốt của **bộ gõ tiếng Việt** không bị nhánh mới ăn mất.
  **Chủ: Ice** *(một lượt kiểm tay trên máy thật, cùng lớp với Task 1.4/1.5 của các story trước)*.

- 🔵 **Món `restore_segment_version` khi văn bản RỖNG (`:3821-3829`) rà lại 2026-08-17 —
  KHÔNG chạm, và lý do đáng ghi.** Story 2.9 **không** sinh thêm một đường tạo segment rỗng:
  nghiệp vụ gộp dùng nguyên `regroup::merge` của Story 2.8, và `regroup.rs:121` **lọc bỏ mảnh
  rỗng trước khi nối** — một lượt gộp hai câu chưa dịch cho `target_text` rỗng đúng như hai câu
  nguồn, không một chuỗi `" "` mới. Cử chỉ mới chỉ đổi **cách gọi**, không đổi **cái được ghi**.
  ⇒ Món giữ nguyên chủ **Story 2.8** đã ghi; mệnh đề *"rỗng có thể tách nghĩa ở gộp/tách"* vẫn
  chưa có ca hợp đồng nào. **(Chủ: story kế tiếp rà `restore_segment_version`.)**

- ⚠️ **Ba ca e2e của story này ĐỤNG TRẦN `mochaOpts.timeout` 120 s, và trần đó nay còn rất ít
  biên.** Đo 2026-08-17: mỗi lệnh WebDriver của bộ đo trả giá **~5 giây** — service in
  `Tauri core.invoke not available after 5s timeout` ở **mỗi** lệnh — nên chi phí một ca tỉ lệ
  **số lệnh**, không tỉ lệ việc nó làm. Ba ca rời chạy 1m40 · 1m48 · 1m40; ca thứ ba **trượt
  bằng timeout** khi đứng cuối chuỗi, và **xanh 1/1 khi chạy riêng**.
  🔴 **Ban vá KHÔNG phải nới trần** *(vá triệu chứng, `project-context.md` cấm bằng chữ)* — hai
  đối chứng âm được gộp vào **một** `it()` dùng chung một lượt dựng Tác phẩm, còn 2m28 cho cả
  spec. Nhưng đó là một lượt mua biên, không một lượt sửa nguyên nhân: story sau thêm **một**
  ca vào spec này sẽ đụng trần lần nữa.
  ⇒ Nguyên nhân thật *(vì sao `core.invoke` của **bàn đo** không lên)* trùng với quan sát mới
  ghi ở món *"bộ e2e chập chờn"* ngay trên. **Chủ: story hạ tầng e2e.**
  → 🟡 **ĐÓNG MỘT NỬA 2026-08-18 (Story 2.12).** Story này **giảm** áp lực lên trần chứ không dời nó:
  chín lượt `window.location.reload()` trong sáu tệp spec đã còn **một** *(lượt ở
  `segment-navigation:172` cố ý ở lại — nó đồng bộ webview sau một lượt ghi **ngoài luồng**, không
  vá một rò rỉ state)*, và mỗi lượt `reload()` gỡ đi là một lượt dựng lại trang không phải trả nữa.
  🔴 **CÒN HỞ:** trần `mochaOpts.timeout` **vẫn là 120 s** và **không được nới** — đúng luật *"vá
  triệu chứng bị cấm"*. Nguyên nhân thật *(vì sao `core.invoke` của bàn đo không lên)* trùng với món
  ngay trên và **vẫn chưa ai đặt tên**. ⚠️ Và cạm bẫy 6 của hồ sơ story cảnh báo đúng chiều ngược
  lại: một fixture **nặng thêm** có thể đẩy ca thứ tư qua trần. Fixture nay gọi thêm
  `resetPanelState()` — một lượt `browser.execute` với năm `import()` — nên chi phí đó **phải được
  đo** ở lượt chạy trọn bộ đầu tiên. **Chủ: giữ nguyên.**

- 🔴 **Cử chỉ chuột của lưới KHÔNG CỔNG NÀO CANH, và nay chúng đã có ba.** Story 2.9 thêm cái
  thứ ba *(`Mod`+click đánh dấu chỗ cắt)* bên cạnh hai cái sẵn có ở cột nguyên văn — bấm trơn
  *(nay để trống)* và vùng chọn cho Auto-Lookup. `check:commands` Kiểm A **chỉ canh `@click`**,
  và cả ba đường này đi qua `@mouseup`. ⇒ Không phép kiểm tĩnh nào trả lời được câu *"còn cử chỉ
  chuột nào đang giẫm lên nhau"* — đúng câu hỏi mà một lượt dùng thật của Ice vừa phải trả lời
  thay. Cùng lớp với món *"`@keydown` nay mang một thao tác thật"* ngay trên.
  **Chủ: một story hạ tầng cổng.**
  → ⚠️ **NGOÀI PHẠM VI Story 2.12 — giữ nguyên chủ, tường minh 2026-08-18 (Task 9.2).** Story 2.12
  dựng **một** cổng mới (`check:panel-refs`, AC5) và nó canh **ô nhớ cấp module**, không canh **cử
  chỉ chuột**. Hai mệnh đề khác miền; gộp chúng vào một cổng là đúng thứ Ice loại ở quyết định #7(b)
  *(*"một cổng mang hai mệnh đề khác miền là chỗ mệnh đề yếu bị mệnh đề mạnh che"*)*.
  🔵 Ghi ra vì hồ sơ story nêu đích danh mục này ở Task 9.2 — nó **được xét và được giữ**, không bị
  bỏ quên. **Chủ: giữ nguyên — story hạ tầng cổng kế tiếp.**

- ⚠️ **`PLATFORM` của `GridPanel.vue` KHÔNG tiêm được, khác `installCommands`.** `hasPrimaryModifier`
  nhận nền tảng qua **tham số** và có `tests/frontend/editorSourceCutGesture.test.ts` lái cả hai
  ca *(bốn lượt đột biến mã sản phẩm, mỗi lượt đỏ đúng chỗ)*, nhưng **dây nối** ở component thì
  gọi thẳng `detectIsMac()` một lần lúc dựng. ⇒ Không đường nghiệm thu nào chứng minh **component**
  truyền đúng nền tảng xuống vị từ. Hôm nay vô hại *(một dòng, không nhánh)*; nó thành một khoảng
  hở vào ngày có cử chỉ chuột thứ hai cần `Mod`. **Chủ: story đầu tiên thêm một cử chỉ chuột có
  phím bổ trợ.**

- 🔴 **Bàn đo của chính Story 2.9 mang một khuyết tật đã vá, và bài học đáng giữ hơn bản vá:**
  `waitForExist('[data-col="src"]')` **không phân biệt** *"Chương mới đã nạp"* với *"Chương CŨ
  còn nằm đó"*. Ca đầu của spec gộp 3 hàng thành 2; ca sau dựng một Tác phẩm mới, đọc ngay và
  thấy **2** — một lượt đỏ nói về **bàn đo** chứ không về sản phẩm *(nguyên văn: `Expected: 3,
  Received: 2`)*. Đã vá bằng `doiLuoiCo(n)` — chờ **số hàng mong đợi**, không chờ "tồn tại".
  ⚠️ Vá này chỉ đóng cho **spec của 2.9**. Khuôn `waitForExist` rồi đọc ngay còn nguyên ở các
  spec khác, và nó là một **ứng viên chưa ai xét** cho món *"bộ e2e chập chờn"*.
  **Chủ: story hạ tầng e2e.**
  → ✅ **ĐÃ ĐÓNG 2026-08-18 (Story 2.12 · AC3).** Khuôn rút thành `e2e/support/gridWait.mjs`
  (`waitForGridRows` + `waitForGridText`), và **bốn** chỗ còn lại đã chuyển: `segment-merge-split`
  ×3 · `grid-empty-cell` · `editor-typing-flush` · cộng hai khuôn tự vá (`doiLuoiCo` của
  `segment-backspace-merge`, `doiChuong` của `segment-navigation`) nay đi qua helper chung.
  ⚠️ Bốn lượt `waitForExist` **CỐ Ý ở lại** — `attribution-focus:82` · `shortcuts-focus:43,70` ·
  `shortcuts-capture-mouse:61`: ở đó *"phần tử tồn tại"* **chính là** mệnh đề đang kiểm (panel mở
  hay đóng), không một tiền đề cần chờ. Chuyển chúng là đổi thứ ca đang đo.
  🔴 Và helper mang một ranh giới viết thẳng vào doc-comment: gọi nó với con số của **đầu vào
  fixture**, đừng bao giờ gọi nó với con số mà ca sắp `expect` — nếu không thì phép chờ nuốt chính
  phép kiểm. Một `expect(truoc.soHang).toBe(3)` đã thành vô nghĩa vì thế và **bị gỡ**, không giữ lại
  làm một dòng xanh không bao giờ đỏ được.
  🔵 **ĐÍNH CHÍNH 2026-08-19 — hai mệnh đề của lượt đóng hôm qua đã hết đúng, sửa tại chỗ.**
  ⓵ **Câu *"chín lượt `reload()` là vá của BÀN ĐO"* SAI MỘT NỬA** — và cả ba tài liệu đều chép
  cùng cái sai đó *(hồ sơ story · mục này · `wdio.conf.mjs`)*. `reload()` dựng lại webview ⇒ chạy
  lại `main.ts` ⇒ `GridPanel.vue::onMounted` ⇒ `ensureChapterLoaded()`. Nó mang **HAI** vai: dọn
  state **và** phát một lượt nạp. Bản vá hôm qua chỉ thay vai thứ nhất ⇒ lượt trọn bộ **thứ mười**
  cho **5 passed / 6 failed** *(xấu hơn mốc 8/3)*, `Lần đọc cuối: 0` ở mọi ca — lưới **không bao
  giờ nạp**. ⇒ Đúng khuôn *"chữ ký thi hành đúng MỘT NỬA"*, **lần thứ sáu** của Epic 2. Và
  `libraryImport.ts:173` đã viết sẵn câu trả lời từ 2026-08-07: *"VỨT state cũ là CHƯA ĐỦ — phải
  NẠP LẠI ngay tại đây"*.
  ⓶ **Bản vá thật:** `support/panelReset.mjs` nay soi **cả hai** nửa của `finishSubmit` *(năm hàm
  `reset*` → đọc lưới phải rỗng → `ensureChapterLoaded()` + `ensureSegmentsLoaded()`)*, và nó chuyển
  về **SAU** lượt tạo Tác phẩm — đúng chỗ `finishSubmit` chạy, tức sau khi `replace_open_work` đã
  trỏ `OpenWorkState` sang Tác phẩm mới. Nghiệm thu: lượt trọn bộ **thứ mười hai** = **11/11 xanh**.
  🔴 ⇒ Và quyết định #5 phải đọc lại kèm dòng này: đường (a) bị loại vì *"giết cả webview state"* —
  lý do vẫn đúng, nhưng lúc ký **không ai biết** thứ nó "giết" bao gồm một vế **bắt buộc**.

- 🔴 **HỎNG DỮ LIỆU IM LẶNG ở tab Hán Việt — TÌM RA và ĐÃ VÁ cùng ngày (Story 2.9, AC9).**
  Ice báo *"chưa thấy điểm cắt, và chưa cắt được"*; bàn đo cho một bảng nặng hơn hẳn triệu
  chứng. Trên `京都春風。` (**5 ký tự**), `sourceCutOffsetOf` trả **17** ở kiểu `switch` và
  **19** ở `parallel`. Nguyên nhân lớn nhất **không nằm trong ba giả thuyết ban đầu**: dòng
  `Nguồn: thieu-chuu` (`.hv-sources`, 17 ký tự) nằm **trong ô** và bị phép đếm mù cộng vào.
  🔴 **Hôm nay nó chưa hỏng im lặng chỉ vì MAY** — hai con số tình cờ vượt biên một câu 5 chữ
  nên Rust từ chối. Trên một câu Chương thật (40–60 chữ), `19` nằm **trong biên** và `⌘/` cắt
  **sai chỗ, im lặng**, trên dữ liệu mà AD-5 không cho hoàn tác.
  ✅ Đã vá: phép **đếm mù** thay bằng **đọc neo** `data-src-start`; không neo ⇒ `null`.
  Đo lại sau vá, cùng bàn đo: `switch` **0** *(đầu từ được bấm)* · `parallel` **2** ✅
  → ✅ **ĐÃ ĐÓNG 2026-08-17 (Story 2.9, AC9)** — phép kiểm đã chạy VÀ ĐO LẠI ngay trong mục
  này (không suy luận): trước vá `switch`=17 `parallel`=19 trên câu 5 ký tự đã đo được là hỏng;
  sau vá (đọc neo `data-src-start` thay đếm mù) đo lại `switch`=0 `parallel`=2, đúng.

- ⚠️ **Ở kiểu `parallel`, một chỗ cắt nằm GIỮA một từ KHÔNG vẽ được dấu.** Chữ ký của Ice
  (2026-08-17) cho `parallel` cắt **chính xác từng chữ**, và phép ánh xạ làm đúng thế *(đo:
  offset 2 trong base `京都`)*. Nhưng dấu cắt vẽ bằng **`::before`** trên phần tử mang neo, nên
  nó chỉ đặt được ở **đầu** phần tử — một chỗ cắt giữa `京` và `都` không có chỗ bám.
  🔴 **Vì sao không chen một `<span class="cut-mark">` vào giữa:** `resolveSwitch()` ánh xạ
  ngược bằng **CHỈ SỐ** (`host.children[i]` ↔ `segments[i]`), và doc-comment của template ghi
  thẳng *"thêm/bớt/đổi thứ tự một phần tử ở đây là làm truy vấn tra cứu sai im lặng"*. Một dấu
  cắt bằng **text node** thì đi vào `Selection.toString()` của Auto-Lookup.
  ⇒ Hai đường rẻ đều phá một thứ đang chạy. **Chủ: Ice** — chọn giữa (a) cho `.hv-unit` nguyên
  khối luôn *(cắt theo ranh giới từ ở CẢ hai kiểu xem; mọi dấu cắt vẽ được, mất độ chính xác
  giữa từ)*, hay (b) giữ độ chính xác và nhận một dấu cắt vô hình ở ca giữa từ.
  ⚠️ Ghi ra vì im lặng ở đây đúng bằng khuyết tật vừa vá: một chỗ cắt **không nhìn thấy**.

- 🔴 **Một bàn đo CHÉP hàm sản phẩm sẽ đo BẢN CHÉP, và bản chép cũ đi — đo được trong chính
  story này.** Sau lượt vá AC9, `2-9-ban-do/han-viet-cho-cat.e2e.mjs` chạy lại vẫn cho **17**
  và **19** y hệt lượt trước, trong khi DOM đã mang neo *(`neoVao: "src-piece"` chứng minh)*.
  ⇒ Nó báo *"chưa vá"* trên một sản phẩm **đã vá**. Cùng họ với *"một con số THẬT, trả lời SAI
  câu hỏi"* mà `2-5d-ban-do` đã đặt tên, nhưng ở một cơ chế mới chưa ai ghi.
  **Chủ: một luật cho bàn đo** — hoặc cấm chép, hoặc buộc cập nhật cùng lượt với hàm gốc.

- ⚠️ **`.cell-src.has-cuts` nay KHÔNG còn là "kênh duy nhất ở chế độ Hán Việt".** Chú thích ở
  `GridPanel.vue` khai nó bằng chữ như thế *(và trỏ về một món nợ có chủ)*; sau AC9 dấu cắt
  **vẽ được** ở cả hai kiểu xem qua `::before`. Đã sửa chú thích tại chỗ kèm 🔵.
  Món còn lại là một câu hỏi **thẩm mỹ**: giữ cả hai kênh *(viền ô + dấu cắt)* hay bỏ một.
  **Chủ: Ice.**

  → ✅ **ĐÃ ĐÓNG 2026-08-17 — Ice dùng thật rồi chốt: BỎ.** Nguyên văn: *"bỏ dấu gạch đứng ở
  trước câu đi, nó không cần thiết"*. Viền `has-cuts` **và chính lớp đó** đã gỡ; hai spec e2e
  đọc nó chuyển sang `data-cut-count` *(chở một SỐ, chặt hơn một cờ)*. Cùng lượt: dấu cắt cao
  `1em` → **`1,3em`** và đổi `ornament` → **`primary`** — `ornament` đo được **2,44/2,64** trên
  `surface`, tức mờ đến mức `check:tokens` cấm nó làm màu chữ.
  🔴 Chiều cao hàng **đã đo, không suy**: 71px → **71px**, chênh 0/0 *(`2-9-ban-do/
  dau-cat-chieu-cao.e2e.mjs`)*. `subgrid` làm một phần tử inline cao hơn line box đẩy cả track
  và kéo ô bản dịch theo — cái giá đó đã đo một lần ở 2.5b (388px), nên nó không được tin bằng mắt.

- ⚠️ **`hasPrimaryModifier` và `caretAtCellStart` nay sống cạnh `sourceCutOffsetOf` trong
  `editorSegments.ts`, và tệp đó khai bằng chữ *"KHÔNG `import` giá trị nào, KHÔNG Vue, KHÔNG
  DOM"*.** Vế "không DOM" đã **hết đúng theo chữ** từ Story 2.8 *(`sourceCutOffsetOf` gọi
  `createTreeWalker`)*, và Story 2.9 thêm hai hàm nữa đụng DOM. Điều kiện THẬT mà cổng
  `check:commands` cần là *"nạp được bằng Node thuần"* — thân hàm đụng DOM thì không sao, chỉ
  `import` giá trị mới giết cổng. ⇒ Chú thích đang mô tả một luật **chặt hơn** luật thật, và
  người sau sẽ hoặc tin nhầm hoặc phá nhầm. **Chủ: một story hạ tầng** *(sửa chú thích cho
  đúng điều kiện, hoặc tách tệp)*.

- ⚠️ **Hai khối CSS của dấu cắt phải đổi CÙNG LÚC, và KHÔNG cổng nào canh việc chúng khớp.**
  `GridPanel.vue::.cut-mark` *(nhánh văn bản thuần)* và `SourceHanViet.vue::.cut-here::before`
  *(cả hai kiểu xem Hán Việt)* vẽ **cùng một** khái niệm bằng **hai** khối rời — chúng không
  dùng chung được vì một cái là phần tử còn cái kia là pseudo-element, và `SourceHanViet` có
  `<style scoped>` riêng. Lượt 2026-08-17 sửa cả hai *(cao 1,3em · `primary`)* và ghi cảnh báo
  tại chỗ ở cả hai đầu, nhưng một lượt sau vẫn có thể sửa một nửa: người dùng sẽ thấy dấu cắt
  đổi hình khi bật tab Hán Việt, và **không phép kiểm nào đỏ**.
  **Chủ: một story hạ tầng cổng** *(hoặc một token dùng chung cho hình dạng dấu cắt)*.

---

## Deferred from: code review of 2-9-gop-bang-backspace-dau-o (2026-08-17)

- 🟡 **`sourceCut` (Story 2.8) không được dọn ở `resetEditorPanel()`** — `editorPanelState.ts:443-492`
  dọn `confirmError` · `caretPlacement` · `confirmNotice` (thêm ở code review 2026-08-15) nhưng
  **không** dọn `sourceCut`, một ô nhớ chở `segment.id` + offset của Tác phẩm đang mở. Nằm ngoài
  diff của 2.9 nên không vá ở lượt này.
  ⚠️ **Quan sát rộng hơn, và nó mới là món thật:** luật viết bằng chữ ở `:479-487` — *"áp cho mọi
  ô nhớ THÊM VÀO TỆP NÀY sau này: hỏi 'ô này thuộc Tác phẩm hay thuộc ứng dụng?'"* — **không có
  cổng nào canh**. Bằng chứng: nó đã bị bỏ sót ở **hai story liên tiếp** (`sourceCut` ở 2.8,
  `regroupNotice`/`regroupError` ở 2.9), cả hai đi qua trọn mười một cổng. Một luật chỉ sống trong
  một khối chú thích là một luật sẽ bị quên lần thứ ba.
  **Chủ: một story hạ tầng cổng** *(một phép kiểm đếm `shallowRef`/`ref` cấp module trong
  `editorPanelState.ts` và đối chiếu với thân `resetEditorPanel`)*.
  → ✅ **ĐÃ ĐÓNG 2026-08-18 (Story 2.12 · AC5) — cổng `check:panel-refs`.** Và nó đóng **RỘNG HƠN**
  hình dạng mục này đề nghị: không chỉ `editorPanelState.ts` mà **toàn `src/**/*.ts`** *(Ice ký #2b;
  đường hẹp *"chỉ `src/panels/**`"* bị loại vì 30 `let` cấp module sống **ngoài** `panels/`)*.
  ⚠️ Và nó **không** đếm — nó đối chiếu **từng ô một** với thân hàm reset, đòi một lượt **GÁN** chứ
  không một lượt nhắc tên. Một phép **đếm** sẽ xanh ở đúng ca `sourceCut`: số ô và số dòng reset
  vẫn khớp nếu một ô mới thay chỗ một ô cũ.
  🔴 Chi tiết đột biến, giới hạn thật, và ba tệp được dựng hàm reset mới: xem mục
  *"`resetEditorPanel()` nay có HAI chỗ gọi…"* ở khối 2-11 phía dưới.


---

## Deferred from: 2-10-dieu-huong-segment (2026-08-18)

- 🔴 **`SECTION.panel` cuộn được dù mang `overflow-y: hidden`, và không ai lấy lại được 18 px
  đó.** Đo trên WKWebView 605.1.15 thật, bàn đo `2-10-ban-do/cuon-vong2-to-tien.e2e.mjs` §Ⓕ: một
  lượt `el.scrollIntoView()` trên một ô của lưới cuộn **cả** `SECTION.panel` từ `scrollTop` 0 →
  **18**, kéo `.panel-body` lên 18 px. `overflow: hidden` nghĩa *"không vẽ thanh cuộn"*, **không**
  nghĩa *"không cuộn được"* — nên người dùng không có thanh để kéo về và không cử chỉ nào đưa nó
  lại. Xảy ra **một lần**, ở **lượt điều hướng đầu tiên** của phiên.
  🔵 **SỬA 2026-08-17 (code review ba tầng) — hai mệnh đề của đoạn này đã hết đúng, và cái sai
  là ở chỗ GÁN CÔNG TRẠNG.** Bản đầu viết: *"Story 2.10 tránh được nó bằng chữ ký #7(b) (gán
  `scrollTop` bằng tay…)"*. Mệnh đề ấy **hết đúng trong cùng story**: `cuonToiHang` — toàn bộ mã
  của đường #7(b) — **đã bị GỠ** ở vòng 3 sau ba lượt đột biến, Ice ký. ⇒ Sản phẩm hôm nay
  **không có một dòng mã cuộn nào**, nên nó không thể được bảo vệ bởi một cơ chế không tồn tại.
  ⚠️ Và bản đầu còn nêu **`focus()`** là tác nhân tái kích hoạt — trong khi `focus()` là **đúng
  thứ sản phẩm đang dùng**, tức đoạn này tự nói ngược: *"không phải lỗi đang sống"* cộng *"`focus()`
  kích hoạt lại nó"* với một sản phẩm mà `focus()` là cơ chế duy nhất.

  🔴 **Lý do THẬT khiến sản phẩm hôm nay an toàn, và nó là một phép đo, không một suy luận:**
  `focus()` **không** cuộn tổ tiên. Bàn đo `2-10-ban-do/focus-co-tu-cuon-khong.e2e.mjs` đọc
  `scrollTopPanel` ở **cả bốn** ca và `SECTION.panel` = **0** ở mọi ca *(Ⓘ `focus()` một mình ·
  Ⓙ `preventScroll` đối chứng âm · Ⓚ `preventScroll`+công thức · Ⓛ ô đã có tiêu điểm)* — xem bảng
  §vòng 3 của `2-10-ban-do/README.md`. ⇒ Tác nhân đã đo được là **`scrollIntoView`**, và chỉ nó.
  Đừng thừa kế câu *"`focus()` cũng kích hoạt"* — nó chưa từng có số đo đỡ lưng.

  Câu hỏi gốc còn nguyên: **vì sao `SECTION.panel` có `scrollHeight > clientHeight`?** Đó là
  18 px nội dung đang bị cắt mà không ai khai. Bất kỳ ai gọi **`scrollIntoView`** trên một phần tử
  trong panel sau này đều kích hoạt lại nó.
  **Chủ: một story bố cục của Epic 1** *(hoặc lượt rà `.panel`/`.panel-body` kế tiếp)*.

  → 🟡 **NGUYÊN NHÂN ĐÃ TÌM RA VÀ ĐÃ VÁ 2026-08-17 — nhưng CHƯA ĐO LẠI, nên đóng một nửa.**

  **Câu hỏi mở của mục này** *("vì sao `SECTION.panel` có `scrollHeight > clientHeight`?")* **nay
  có câu trả lời**, và nó đến từ **mắt Ice**, không từ một cổng: Task 4.5 của Story 2.10 phát hiện
  *"hàng cuối bị che mất chữ"* trên một Chương tiếng Trung.

  **Nguyên nhân:** `.panel-body` (`PanelFrame.vue`) là một **khối thường**, còn `.grid-scroll` xin
  `height: 100%`. `GridPanel` đặt **hai** con vào slot — dải tab Hán Việt rồi hộp cuộn — nên hộp
  cuộn lấy *toàn bộ* chiều cao trong khi đã bị đẩy xuống **30 px** *(tab `ui-sm` 12px × 1,5 = 18px
  + `--space-panel-block` 12px)*. Tràn 30 px, `.panel` mang `overflow: hidden` ⇒ cắt, không thanh
  cuộn nào lấy lại. Khi đã cuộn tới đáy, đáy **hàng cuối** trùng đúng vùng bị cắt.
  ⇒ Con số **18** của bàn đo không phải tổng tràn: `scrollIntoView` cuộn mỗi tổ tiên **đúng lượng
  cần**, không tới đáy. 18 ≤ 30, hai số không mâu thuẫn.

  **Cách vá — ở NGUYÊN NHÂN, vì đây là lần THỨ HAI cùng lớp lỗi:** `.panel-body` nay khai
  `display: flex; flex-direction: column`, và `.grid-scroll` đổi `height: 100%` ⇒ `flex: 1;
  min-height: 0`. Lần đầu là `SourceHanViet.vue:853-861` *(code review 2026-08-06, cả ba tầng)* —
  lần đó vá bằng cách dạy **một** panel viết `flex: 1`, và luật chỉ sống trong một khối chú thích
  nên panel kế tiếp lặp lại y nguyên. Đã rà cả ba panel tiêu thụ trước khi đổi; `AiTranslationPanel`
  và `LookupPanel` mỗi cái có **đúng một** con trong slot khai `height: 100%` ⇒ hình dạng không đổi.

  🔴 **PHẦN CÒN HỞ, và nó là vế nghiệm thu:** không đường **tự động** nào của dự án thấy được lượt
  vá này. 225 ca vitest và mười cổng đều xanh **cả trước lẫn sau** — `happy-dom` **không bố cục**.
  ⇒ Mệnh đề *"`SECTION.panel` nay có `scrollHeight === clientHeight`"* và *"hàng cuối nằm trọn"*
  **chưa được đo**. Đóng trọn cần một trong hai: một spec bàn đo đọc hai số đó trên WKWebView thật,
  hoặc mắt Ice lần thứ hai.
  ⚠️ Và cái bẫy mới do chính lượt vá mở ra, ghi ra thay vì để người sau gặp: flex column cho mọi
  con `flex-shrink: 1` mặc định ⇒ một con không khai `flex: none` bị **co** thay vì làm tràn.
  `.tabs` và `.load-error` đã khai; **không cổng nào canh câu này** cho con thứ ba.
  **Chủ phần còn hở: Ice** *(một lượt nhìn, hoặc một lượt cho phép chạy bàn đo)*.

  → ✅ **ĐÃ ĐÓNG 2026-08-17 — Ice nghiệm thu bằng mắt trên bản dựng thật:** *"đã pass, không bị ẩn
  nữa"*. Hàng cuối Chương tiếng Trung nay hiện đủ chữ. ⇒ Cả hai vế của mục này đóng: nguyên nhân
  *(vá ở `PanelFrame.vue::.panel-body` + `GridPanel.vue::.grid-scroll`)* và nghiệm thu *(mắt Ice —
  đúng đường mà Task 4.5 khai từ đầu là đường duy nhất cho vế này)*.
  ⚠️ **Cái bẫy do lượt vá mở ra thì KHÔNG đóng theo:** flex column cho mọi con `flex-shrink: 1`
  mặc định ⇒ một con không khai `flex: none` bị **co** thay vì làm tràn, và **không cổng nào canh
  câu đó**. Hôm nay `.tabs` và `.load-error` đã khai. **Chủ: panel nào đặt con thứ ba vào slot của
  `PanelFrame`** — luật đã ghi tại chỗ ở `PanelFrame.vue::.panel-body` §GIỚI HẠN THẬT.

- 🔴 **Không cổng nào canh `scroll-behavior`, và AC8 của story này phụ thuộc vào nó.** Đo
  2026-08-18: `grep -rn "scroll-behavior" _bmad-output/` ⇒ **0 kết quả**; trên `src/` ⇒ đúng **2**
  kết quả, cả hai là **chú thích** (`LookupPanel.vue`), không một dòng CSS. Giá trị đang hiệu lực
  lúc chạy là `auto` trên cả hộp cuộn lẫn `documentElement` — mà `auto` **uỷ quyền cho CSS**.
  ⇒ Mệnh đề *"không `scroll-behavior: smooth` ở bất kỳ đâu trong `src/**`"* là một **quy ước sống
  trong hai khối chú thích**, không một luật cưỡng chế được. Ngày ai đó thêm nó, **AC7 của Panel
  Lookup và AC8 của Panel Editor hỏng CÙNG LÚC và IM LẶNG**, và cả mười một cổng vẫn xanh.
  ⚠️ 🔵 Hai chú thích ấy trích `DESIGN.md:342` cho một luật **chưa từng tồn tại** — đã **sửa tại
  chỗ** ở lượt này *(dòng 342 nói về chiều rộng `ch` của Chế độ đọc)*. Nguồn thật gần nhất là
  `DESIGN.md:373`, và nó thuộc phạm vi **Panel Lookup**, không toàn ứng dụng.
  **Chủ: một story hạ tầng cổng** *(một phép kiểm chuỗi trên `src/**/*.vue` là đủ — cùng họ với
  Kiểm B của `check-tokens`)*. Hoặc: Ice cho `DESIGN.md` một mệnh đề toàn ứng dụng thật, rồi cổng
  cưỡng chế nó.

- 🟡 **`goToNextSegment`/`goToPrevSegment` đăng ký mà KHÔNG có phím mặc định** — Quyết định #2
  đường (c), Ice ký. Đủ chữ AC9 *("command đăng ký, gán phím được")* và chúng có mặt trong bảng
  phím của Story 1.21 để người dùng tự gán. ⚠️ Hệ quả thật, ghi ra thay vì để người sau tự phát
  hiện: **hai lệnh này vô hình cho tới khi người dùng tự gán phím**, nên AC1/AC2 hôm nay chỉ chạy
  được qua `dispatch()`.
  **Chủ: Ice** *(một lượt xem lại bảng Phím khi Epic 2 xong và có đủ ngữ cảnh về bảng phím tổng)*.

- ⚠️ **Vế *"`⌥↓` thật có bị macOS nuốt không, và `preventDefault()` có chặn nổi không"* vẫn chưa
  đóng** — mọi sự kiện driver mang `isTrusted: false`, và một sự kiện không tin cậy **không có
  default action**, nên một phép kiểm sẽ trả *"chặn được"* trên **mọi** engine kể cả engine không
  cho chặn. Cùng lớp với *"không bộ chạy test nào mô phỏng được một bộ gõ tiếng Việt thật"*.
  ⚠️ Chữ ký #1(c) *(đổi sang `Mod+Alt+ArrowDown`)* làm nó **hết chặn** — đường đã ký không cướp
  `⌥↓` nữa. Nó ở lại đây như một mệnh đề chưa đóng cho bất kỳ ai sau này muốn đường *"cửa thứ hai
  ở `onEditKeydown`"*.
  🔵 **CHỦ ĐẶT LẠI 2026-08-17 (code review).** Bản đầu ghi *"Chủ: chưa cần"* — theo chữ luật của
  `project-context.md` §Sổ nợ *("Mọi thứ… KÈM MỘT CHỦ. Không có mục nào mồ côi")* thì đó là một mục
  **mồ côi**, và *"chưa cần"* là đúng cái hình dạng mà một mục không chủ mang khi nó trông vô hại.
  **Chủ: Ice** — và chủ ấy có một **điều kiện kích hoạt viết ra**: bất kỳ lượt nào đề xuất đường
  *"cửa thứ hai ở `onEditKeydown`"* cho một hợp âm **không** mang `Mod` *(tức bất kỳ lượt nào lật
  chữ ký #1(c) của Story 2.10)* thì mục này **chặn** lượt đó cho tới khi có một phép đo trên máy
  thật. Không lượt nào như thế được đi bằng suy luận từ mã: thước của dự án **không đo được** vế
  này *(`isTrusted: false` ⇒ không default action ⇒ mọi phép kiểm trả "chặn được" trên mọi engine)*.

- 🔴 **AC8 nửa sau dựa vào HÀNH VI ENGINE, không vào một dòng mã đọc được** — và đó là một
  quyết định có chữ ký, không một chỗ bỏ sót. Story 2.10 giao Task 4 *"cuộn tới hàng — CHƯA CÓ"*;
  ba phép đo bác tiền đề đó *(xem `GridPanel.vue` §"AC8 NỬA SAU")*: `target.focus()` đã tự cuộn
  hàng vào vùng nhìn, **không** đụng một tổ tiên nào, và nó **khéo hơn** một công thức tự cài —
  **căn giữa** khi hàng ở xa *(`scrollTop` 1569)*, **nearest** khi nó chỉ vừa ló khỏi mép *(dịch
  đúng 38 px = một chiều cao hàng)*. Một công thức ép nearest ở mọi ca dán hàng đích vào sát mép
  dưới sau lượt nhảy xa, tức **xấu hơn**. ⇒ Ice ký **gỡ** `cuonToiHang` 2026-08-18; không mã cuộn
  nào ở `GridPanel.vue`.
  ⚠️ **Cái giá:** không chuẩn nào bảo đảm hành vi ấy và **không cổng nào canh nó**. Một bản WebKit
  sau đổi cách cuộn khi nhận tiêu điểm ⇒ AC8 hỏng **im lặng**, và cả mười một cổng vẫn xanh.
  Lưới duy nhất là ca Ⓒ + Ⓔ của `e2e/specs/segment-navigation.e2e.mjs` — **chạy tay**.
  ⚠️ Cùng lớp: `happy-dom` **không bố cục**, nên vitest không bao giờ thay được lưới đó.
  **Chủ: bộ e2e trong CI** *(hôm nay e2e không chạy trên runner nào — action item A5 của retro
  Epic 1 đã ghi vế Windows của cùng khoảng trống này)*.

- ⚠️ **Ba lượt đột biến của Story 2.10 cho một bài học phương pháp, ghi lại vì nó sẽ lặp.** Ca
  e2e §Ⓒ *("đã cuộn" + "hàng nằm trọn")* **xanh ở CẢ HAI** thế giới — có `cuonToiHang` và không.
  Nó là một ca **tự xưng** là canh một hàm mà thật ra đo một hàm khác. Chỉ một phép so về **độ
  dịch** *(ca Ⓔ)* mới phân biệt được *nearest* với *căn giữa*.
  ⇒ **Một ca test khẳng định "X đã xảy ra" không chứng minh "mã CỦA TÔI làm X".** Đường phân biệt
  duy nhất là gỡ mã ra và chạy lại — đúng thứ Task 7.3 đòi, và ở story này nó bắt được **hai** ca
  vô hiệu liên tiếp. **(Chủ: một story hạ tầng kiểm thử kế tiếp.)**

## Deferred from: code review of 2-10-dieu-huong-segment (2026-08-17)

- ⚠️ **Một lượt điều hướng đồng bộ trong lúc `confirmCurrentSegment` đang bay kéo caret về chỗ
  `confirm` đã chọn, ghi đè chỗ người dùng vừa tới.** Đường đua: `⌘Enter` chạy tới
  `await flushEditorBeforeDiscreteWrite()` / `await confirmSegment(...)`; trong lúc chờ, người
  dùng bấm một lệnh điều hướng — chúng **đồng bộ** và **không** bị `confirmInFlight` khoá *(khoá
  đó chỉ nối tiếp các lượt `confirm` với nhau)*. Khi lượt xác nhận resume, nó tính `following` từ
  vị trí của câu **cũ** trong ảnh chụp rồi gọi `setEditorCaret(following.id)` +
  `caretPlacement.value` (`editorPanelState.ts:921-924`), và watcher đường lệnh ở
  `GridPanel.vue` kéo caret DOM theo.
  ⚠️ **Không mất dữ liệu** — lượt `setEditorCaret` ấy vẫn mang vế (d) của AD-35 nên bộ đệm của
  câu người dùng vừa gõ được flush đúng về câu đó. Hệ quả là **caret nhảy**, không phải chữ sai chỗ.
  🔵 **Lớp lỗi có TRƯỚC Story 2.10** *(`editor.next_untranslated` của 2.5b đã là một đường điều
  hướng đồng bộ)*, nên đây **không** phải khuyết tật do 2.10 tạo ra. Nhưng 2.10 thêm **hai** lệnh
  nữa đi đúng đường đó ⇒ bề mặt chạm tới rộng hơn hẳn, và không test nào của kho dựng kịch bản
  xen kẽ này.
  **Chủ: story đầu tiên phân xử "khoá bàn phím trong lúc một lượt ghi rời rạc đang bay"** *(cùng
  họ với `regroupInFlight` ở 2.8 — chỗ đó đã chọn **từ chối và kêu** thay vì nhập vào lượt đang
  bay; câu hỏi ở đây là ba lệnh điều hướng có phải theo cùng luật đó không)*.

- 🔴 **Không cổng nào đối chiếu tên `var(--<token>)` với bảng token — một tham chiếu KHÔNG TỒN TẠI
  là CSS chết im lặng.** Đo 2026-08-18 (lượt code review 2.10, sau khi Ice tìm ra bằng mắt): dải
  tab Hán Việt của `GridPanel.vue` viết `gap: var(--space-inline-sm)`, và `inline-sm` **chưa bao
  giờ** có trong khối `spacing` của `tokens.json` *(đúng chín khoá)*. `applyTheme` phát
  `--space-<tên>` từ **chính** khối đó (`tokens/index.ts:106-107`) ⇒ biến không được đặt ⇒ **cả**
  khai báo `gap` không hợp lệ ⇒ `gap` về `normal` = **0** ⇒ hai tab *"Trung"*/*"Hán Việt"* dính
  nhau. Sống từ `ca33072` (Story 2.5b, 2026-08-15) tới 2026-08-18: **ba story, mười một cổng, một
  lượt code review ba tầng — tất cả xanh.**
  ⚠️ Vì sao `check:tokens` không thấy: Kiểm B đọc CSS để bắt **màu viết thẳng** *(hướng ngược —
  giá trị đáng lẽ phải là token)*. Không phép kiểm nào đi hướng còn lại *(tên token phải có thật)*.
  ⇒ Hai hướng là **hai** mệnh đề, và hôm nay chỉ một hướng có chủ.
  🔵 Ca cụ thể **đã vá** *(hoàn nguyên về `--space-panel-inline`, giá trị chạy được từ Story 1.16 —
  xem khối lý do ở `GridPanel.vue::.tabs`)*, nhưng **lớp lỗi thì chưa có lưới nào.**
  ⚠️ Đo kèm, vì nó nói phạm vi thật của món nợ: đối chiếu chín khoá `spacing` với mọi `--space-*`
  trong `src/**` ⇒ `inline-sm` là ca **duy nhất** của toàn cây hôm nay. Bốn khoá khai mà không dùng
  (`gutter-width` · `read-measure-lg/md/sm`) — **không** phải lỗi, nhưng phép kiểm mới nên báo
  chúng ở mức ghi chú chứ đừng làm đỏ, kẻo nó thành một cổng ai cũng học cách bỏ qua.
  🔴 **Đây là một phép kiểm THÊM VÀO `check-tokens.mjs`, KHÔNG một cổng thứ mười hai** — thêm cổng
  là sửa **ba** danh sách (`package.json` · `ci.yml` · `.githooks/pre-push`) và `check:gates` canh
  cả ba. Phạm vi đủ nhỏ: đọc bảng token, quét `--<tiền tố>-<tên>` trong `src/**/*.vue` + `*.css`,
  đỏ ở một tên không có khai báo. Cộng một phép **tự kiểm chứng minh nó đỏ được** (luật của một cổng).
  **Chủ: một story hạ tầng cổng** *(cùng chỗ với món nợ `scroll-behavior` ở khối 2-10 phía trên —
  hai món nợ này là **cùng một** hình dạng: một quy ước không ai cưỡng chế, và cả hai đỏ được bằng
  một phép quét chuỗi trên `src/**`)*.

## Deferred from: code review of 2-10-dieu-huong-segment, lượt HAI (2026-08-18)

- 🔴 **Lượt vá selector `.panel` trong `e2e/specs/segment-navigation.e2e.mjs:242` KHÔNG có đường
  nghiệm thu nào — nó đúng theo suy luận cấu trúc, không theo một phép đo.** Thước cũ
  (`document.querySelector('.panel')`) là một thước **mù**: `PanelFrame.vue:144` là chỗ duy nhất
  khai `class="panel"`, nhưng cả ba panel dựng component ấy (`GridPanel` · `LookupPanel` ·
  `AiTranslationPanel`) và preset mặc định `B2_GRID_LEFT` đặt cả ba **cạnh nhau**, nên
  `querySelector` lấy phần tử đầu theo thứ tự DOM — thứ tự do cây split của dockview quyết. Trúng
  một panel không bị đụng thì `scrollTop` **luôn** 0 và `toBe(0)` xanh **vô điều kiện**.
  ⇒ Đã neo lại bằng `.grid-scroll` rồi `closest('.panel')`.
  ⚠️ **Vế còn hở:** mệnh đề *"thước cũ THẬT SỰ đo nhầm panel trên WKWebView"* chưa ai đo. Bộ e2e
  chạy tay và chưa vào CI, nên cả thước cũ lẫn thước mới đều **chưa chạy một lượt nào** kể từ lượt
  sửa. Có khả năng thứ tự DOM của dockview vẫn luôn cho `panel.grid` đứng đầu, tức thước cũ **vô
  tình** đúng — nhưng *"vô tình đúng"* không phải một lưới.
  🔴 Điều đo được ngay khi bộ e2e chạy: chạy ca Ⓓ **trước** và **sau** lượt sửa selector; nếu hai
  lượt cho cùng một số thì thước cũ vô tình đúng và món nợ này đóng bằng một ghi chú, nếu khác
  nhau thì nó đóng bằng một lượt xanh-giả vừa bị bắt.
  **Chủ: bộ e2e trong CI** *(cùng chủ với món nợ AC8 nửa sau ở khối 2-10 phía trên — cả hai chờ
  đúng một điều kiện khởi hành)*.

- 🟡 **Nhánh `if (s.retiredAt !== null) continue` trong `segmentNavigation.ts::buocTu` là mã phòng
  thủ KHÔNG ĐO ĐƯỢC ở sản phẩm.** Hai hàng rào đứng trước, mỗi hàng đủ một mình:
  `src-tauri/src/commands/segment.rs:840` lọc `WHERE retired_at IS NULL`, và
  `editorPanelState.ts::applyRegroup` (`:1508-1517`) gỡ hẳn hàng về hưu khỏi `segments.value`.
  ⇒ Đổi `continue` thành `break` đi qua **mọi** cổng và **không một Chương thật nào** lộ triệu
  chứng. Lưới duy nhất là ba ca vitest.
  🔵 **Đóng một nửa 2026-08-18:** chỗ yếu **đã được ghi ra** thành §GIỚI HẠN THẬT trong
  doc-comment của `buocTu`, cộng một lượt sửa mệnh đề hết đúng ở `NavigationSegment.retiredAt`
  *(dòng cũ viết `null` "cho tới Story 2.8", ngụ ý sau 2.8 thì khác — đo lại: vẫn luôn `null`)*.
  🔴 **Phần CÒN HỞ:** không cổng nào kiểm chéo hai workspace. Một lượt nới `WHERE` ở Rust cho một
  tính năng *"xem lịch sử gộp"* sẽ đưa hàng về hưu ra webview mà không ai nhớ tới hàm này — cùng
  hình dạng với món nợ `is_han` *(hai định nghĩa, hai workspace, không cổng kiểm chéo)*.
  **Chủ: story đầu tiên đưa hàng về hưu ra khỏi Rust** *(hôm nay chưa story nào định làm thế; nếu
  Epic 5 dựng màn hình lịch sử gộp thì nó là chủ)*.

- ⚠️ **`editorHasLoaded()` là một vị từ mà KHÔNG CỔNG NÀO canh việc nó được dùng.** Nó tồn tại từ
  Story 1.16 chính vì lớp lỗi *"`segments` rỗng vì ba lý do khác hẳn nhau"*, nhưng nó là một hàm
  export mà **chỗ quên gọi vẫn biên dịch sạch** — và đường điều hướng của Story 2.10 đã quên đúng
  một lượt *(vá 2026-08-18: `dieuHuongVaBao` nay chặn trước bằng `NavNotice` `'loading'`)*.
  🔴 Đây là **lần thứ hai** cùng lớp: `hanVietPending` của 1.16 là lần thứ nhất, và nó cũng được
  dựng sau một lượt màn hình khẳng định điều nó chưa biết.
  ⚠️ **Chưa rõ phép kiểm đúng hình dạng gì** — *"mọi bề mặt đọc `segments` phải đi qua
  `editorHasLoaded`"* khó diễn đạt bằng một phép quét chuỗi mà không đỏ oan. Ghi ra để lần thứ ba
  không phải phát hiện lại từ đầu.
  ~~**Chủ: Ice phân định** — nó có đáng một phép kiểm, hay đáng một dòng trong `project-context.md`
  §Critical Don't-Miss Rules và thế là đủ.~~
  → ✅ **ĐÃ ĐÓNG 2026-08-18 — Ice chốt: một dòng, KHÔNG một cổng.** Đã thêm vào
  `project-context.md` §Critical Don't-Miss Rules ▸ *"Rỗng IM LẶNG bị cấm"* — đúng mục, vì đây là
  một biến thể của lớp lỗi trung tâm chứ không một lớp mới.
  🔴 **Ghi thẳng cái mà lượt đóng này KHÔNG mua được:** một dòng trong `project-context.md` là một
  luật **agent đọc**, không một cổng **cưỡng chế**. Lần thứ ba vẫn vi phạm được mà không lượt CI
  nào đỏ — khác với `check:commands` hay `check:layout`. Điều nó mua: lần thứ ba sẽ **nhận ra**
  thay vì phát hiện lại từ đầu, và đó là thứ hai lần trước đều thiếu.
  ⚠️ **Điều kiện mở lại:** nếu lớp lỗi này hụt **lần thứ ba**, thì bằng chứng đã đủ để không cần
  bàn nữa — dựng phép kiểm, và dựng như một phép kiểm THÊM vào một cổng có sẵn, không một cổng
  thứ mười hai *(cùng hình dạng với món nợ `--space-inline-sm` và `scroll-behavior`)*.

## Deferred from: 2-11-chuyen-chuong-trong-workspace (2026-08-18)

- 🔴 **KHÔNG ĐƯỜNG SẢN PHẨM NÀO SINH RA CHƯƠNG THỨ HAI, nên AC1/AC2 xanh mà chưa ai bấm được.** Đo từ nguồn 2026-08-18: `grep -rn "INSERT INTO chapter" src-tauri/src` = **1** kết quả (`commands/project.rs:138`), và hàng đó chèn `ord = 1` **viết cứng**, một lượt, không vòng lặp; `grep -rn "list_chapters\|read_chapters" src src-tauri/src` = **0**. Trên **mọi** `.atproj` tồn tại hôm nay *(21 Tác phẩm thật, mỗi cái đúng 1 Chương — mục `:559-560`)* không có Chương thứ hai để mở. ⇒ Cơ chế của Story 2.11 **đã dựng trọn** và nghiệm thu bằng **hợp đồng dữ liệu** *(8 ca trong `project_contract.rs` chèn Chương thứ hai bằng SQL trực tiếp — chữ ký #1(a) của Ice)*, nhưng **không đường e2e nào** với tới được một lượt chuyển thành công. 🔴 **Đây là một món nợ, KHÔNG một ca đã xanh** — đừng đọc *"409/0/5 cargo xanh"* thành *"người dùng chuyển Chương được"*. **Chủ: Epic 6** *(FR14 — nhập hàng loạt + mẫu phân tách ⇒ nhiều Chương, `epics.md:662`)*; **Epic 5** cũng mở nhánh này ở FR15 *(gộp/tách/sắp lại Chương, `:663`)*. Khuôn ghi nợ này đã có chữ ký hai lần: #8(a) của Story 2.6 và AC3 của Story 2.7.

- 🔴 **AC5 *(mở lại một Chương ⇒ khôi phục đúng segment và vị trí cuộn)* GIAO TRỌN CHO EPIC 5 — Ice ký Quyết định #4 đường (c), 2026-08-18.** Story 2.11 giao **5/6 AC**, đúng khuôn chữ ký ① của Story 2.9. Ba lý do, cả ba đo được: **①** AC5 phát biểu **nguyên văn FR12**, mà bảng ánh xạ giao FR12 cho **Epic 5** (`epics.md:660`), còn story này khai `Covers: FR26` và chỉ FR26; **②** hôm nay **0** mảnh hạ tầng tồn tại — 0/9 `ScopeKind` cho vị trí đọc (`core/scope/kinds.rs:157-219`) · `config_value` nằm ở `global.db` với cột `value TEXT` phẳng chỉ phục vụ ba loại `GlobalOnly` (`schema.rs:98-105`) · `grep -rn "scroll" src-tauri/src` = **0** · lưới **không** một dòng cuộn tường minh nào *(cuộn đến từ hành vi engine sau `target.focus()`, `GridPanel.vue:903-923`, và đường đó chỉ chạy khi `editorCaretPlacement` được đặt — tức **không** chạy ở luồng mở Chương)*; **③** hạ tầng này có **BA** chỗ tiêu thụ, không riêng 2.11 — **UX-DR34** (`epics.md:601`) đòi y hệt cho lượt **đổi chế độ** (*"rời Workspace sang Chế độ đọc rồi quay lại thì vẫn đúng Chương, đúng câu, đúng vị trí cuộn"*). Chọn hình dạng ở đây là chọn cho cả ba, nên nó thuộc story sở hữu cả ba. ⚠️ **Và khi Epic 5 dựng: `segment.id`, KHÔNG pixel.** AD-3 (`SPINE:93`) nói bằng chữ *"mọi dữ liệu gắn theo segment tham chiếu `id`, không bao giờ tham chiếu vị trí"*; một `scrollTop` pixel vô nghĩa ngay khi người dùng đổi cỡ chữ hoặc kéo sash. Đường pixel đã được trình cho Ice và **bị loại**; chọn lại nó là một **AD mới**. **Chủ: Epic 5.**

- 🟡 **Tiêu điểm sau một lượt chuyển Chương THÀNH CÔNG — cơ chế đã cài, vế nghiệm thu còn HỞ.** `switchChapter` gọi `await nextTick()` rồi `enterFocus('panel.grid')` *(`editorPanelState.ts`)*, và lý do có bằng chứng: lượt chuyển thay **toàn bộ** hàng của `v-for`, `segment.id` là `AUTOINCREMENT` **theo Tác phẩm** nên Chương mới gần như chắc chắn mang tập khoá khác ⇒ Vue **gỡ** đúng ô `contenteditable` đang giữ tiêu điểm ⇒ trình duyệt trả nó về `document.body`, thứ AD-34 §2 cấm thẳng. 🔴 **Nhưng mệnh đề *"tiêu điểm KHÔNG rơi về `body`"* chưa có đường nghiệm thu nào:** `happy-dom` **không phải** WebKit *(và không bố cục)*, còn e2e thì **không tới được** một lượt chuyển thành công — cùng món nợ với mục thứ nhất ở trên. ⇒ Đã cài, **không** tự chấm đạt. **Chủ: cùng story mở đường sinh Chương thứ hai (Epic 6/FR14)** — nghiệm thu vế này **cùng lượt** với AC1/AC2.

- ⚠️ **`resetEditorPanel()` nay có HAI chỗ gọi, và luật *"mọi ô nhớ mới phải qua nó"* vẫn KHÔNG có cổng nào canh.** Story này vá **hai** ô sót còn lại — `sourceCut` *(nợ ghi bằng chữ từ Story 2.8, hở hai story)* và `omitError` *(**chưa ai nêu** trước lượt này)* — nên tính tới hôm nay hàm ấy dọn **đủ**. 🔴 Nhưng lượt vá không đóng được món nợ **cổng**: nó chỉ làm sổ sạch tại một thời điểm. Bằng chứng rằng cơ chế vẫn hở: `omitError` **đi qua trọn lượt rà ba tầng của Story 2.9** — lượt vốn vá hai ô **cùng hạng** (`confirmError` · `regroupError`) — và nó lọt **chính vì** nó là ô duy nhất trong hạng đó chưa component nào đọc, tức biểu hiện của nó là **0 pixel**. ⇒ Một cổng canh luật này phải hỏi *"ô nhớ nào thuộc Tác phẩm/Chương"*, không hỏi *"ô nhớ nào nhìn thấy được"*. **Chủ: story hạ tầng cổng** *(cùng chủ với món nợ đã ghi ở lượt rà Story 2.9 — không mở một mục thứ hai cho cùng một cổng)*.
  → ✅ **ĐÃ ĐÓNG 2026-08-18 (Story 2.12 · AC5) — cổng `check:panel-refs`, cổng thứ MƯỜI.** Nó hỏi
  đúng câu mục này đòi (*"ô nhớ nào thuộc Tác phẩm/Chương"*, không *"ô nhớ nào nhìn thấy được"*):
  mọi ô cấp module trong **toàn `src/**/*.ts`** phải đi qua một hàm `reset*` của chính tệp đó, hoặc
  mang một miễn trừ **CÓ TÊN kèm lý do**. Quét **39** tệp · **91** ô · **25** miễn trừ có tên.
  Có mặt ở cả ba danh sách (`package.json` · `ci.yml` · `pre-push`); `check:gates` Kiểm D/E xanh.
  🔴 **Đột biến chứng minh nó không rỗng:** hoàn nguyên **đúng** hai dòng đã lọt qua chín cổng —
  `sourceCut.value = null` (2.8) và `omitError.value = null` (2.9) — ⇒ cổng **ĐỎ, nêu đích danh cả
  hai**; khôi phục ⇒ xanh.
  ⚠️ **Và lượt đột biến ấy suýt không xảy ra, ghi ra vì nó là bài học chứ không một chi tiết.** Bản
  cổng ĐẦU hỏi *"tên có xuất hiện trong thân hàm reset không"*, và nó **XANH** trên chính đột biến
  đó: `resetEditorPanel` gọi vài hàm phụ, và tầng-một kéo theo những dòng chỉ **ĐỌC** hai ô ấy — một
  lượt đọc trở thành bằng chứng cho một lượt dọn. Tức cổng đã suýt vào kho ở trạng thái **không bao
  giờ đỏ được**. Đã siết thành *"phải là một lượt GÁN"*, và một ca tự kiểm khoá đúng lỗ đó.
  🔵 **Kéo theo, cùng lượt:** năm cờ/mutex tiến trình vào `resetEditorPanel` (`inFlight` ·
  `confirmInFlight` · `regroupInFlight` · `kyTrungCauCuoi` · `dangChuyenChuong` — Ice ký #2a), và
  **ba** tệp chưa từng có hàm reset nay có (`resetDictSources` · `resetSegmentHistory` ·
  `resetLookupTiming` — Ice ký #2c). `lookupTiming.ts` là ứng viên thứ ba mà **hồ sơ story không
  biết tới**; nó lòi ra ở lượt đo lại Task 0.1.
  🔴 **GIỚI HẠN THẬT của cổng, ghi thay vì để người sau tưởng đã phủ:** nó đi theo **TÊN**, không
  theo luồng dữ liệu, và chỉ theo lời gọi hàm **sâu một tầng**. Một ô dọn bằng `Object.assign` hay
  một chuỗi hai tầng sẽ bị chấm là chưa dọn — chiều đỏ **oan**, vá bằng miễn trừ có tên chứ không
  bằng việc nới luật.
  🔵 **THÊM 2026-08-19 — một lỗi thứ BA của chính cổng/bộ đo, tìm ra ở lượt chạy thật.**
  `support/gridWait.mjs` đưa giá trị đọc được vào `timeoutMsg` của `browser.waitUntil`. **`timeoutMsg`
  là một CHUỖI dựng lúc tạo object tham số**, nên `${seen}` bị nội suy **trước** khi vòng chờ chạy một
  lần nào ⇒ nó in giá trị **khởi tạo** *(`-1`)* ở mọi lượt đỏ, bất kể lưới thật có bao nhiêu hàng.
  🔴 Hệ quả: ba ca đỏ của lượt trọn bộ thứ mười đều báo *"lần đọc cuối thấy -1"*, và lượt chẩn đoán
  **đi sai hướng ngay câu đầu tiên** — người đọc *(kể cả người viết)* kết luận `browser.execute` ném
  mọi vòng. ⇒ Một bộ đo cho một **câu chẩn đoán không có thật** trên một lượt đỏ **thật**.
  ⚠️ Cùng lỗi có ở `support/flushWait.mjs`; cả hai đã sửa. Luật rút ra, ghi trong cả hai tệp: **mọi
  con số trong một câu báo lỗi phải đọc SAU vòng chờ** *(dựng câu trong `catch`)*, và **lỗi mà
  `waitUntil` nuốt phải giữ nguyên văn rồi in kèm** — `waitUntil` coi một lượt ném là *"chưa đúng"*.

- ⚠️ **`'still-dirty'` nay có nơi gọi đầu tiên phán quyết nó, và nó KHÔNG có ca nghiệm thu riêng.** `flushEditorBeforeDiscreteWrite()` trả ba giá trị; đường đổi **Tác phẩm** (`libraryImport.ts:145`) chỉ gọi `flushEditorNow()` **một** lượt nên nó chưa bao giờ thấy `'still-dirty'`. `switchChapter` là nơi gọi đầu tiên chặn cả `'failed'` **lẫn** `'still-dirty'`. 🔴 Ca vitest hiện có đo được nhánh `'failed'` *(qua `failNextSave`)*; nhánh `'still-dirty'` đòi dựng một cuộc đua — một ký tự gõ **trong lúc** lô đầu đang bay — mà fixture hôm nay chưa có cách bắn. ⇒ Nhánh ấy **đúng theo cấu tạo** *(cùng một `if (flushed !== 'clean')`)* nhưng **chưa có ca nào đi qua**. Ghi ra vì *"một nhánh chưa bao giờ chạy là một nhánh chưa ai biết nó có chạy không"*. **Chủ: story kế tiếp chạm đường flush rời rạc.**

- ⚠️ **NFR2 KHÔNG được story này chấm.** Một lượt đổi con trỏ trên 9.850 câu đo được **706–770 ms** *(trần 50 ms)*, và Story 2.11 thêm một đường **nạp lại toàn bộ lưới** vào cùng bề mặt đó. Story này **không đo và không vá** — nó không chạm một dòng nào của đường render. **Chủ vẫn là Story 2.4** **(Chủ: Story 2.4.)**, đúng như §Ranh giới phạm vi của story ghi.

- ⚠️ **Bộ e2e KHÔNG cho một lượt xanh trọn bộ, và chế độ LÔ là nơi cái đỏ tập trung — dữ kiện MỚI cho món nợ bàn đo của Story 2.5b.** Đo 2026-08-18, trọn bộ 11 spec: **8 passed / 3 failed** (18m51s). Ba ca đỏ đã chạy đối chứng trên **cả hai** cây *(cây story và baseline `5d94ba1`)*: `editor-typing-flush` xanh ở lượt chạy lại; `attribution-focus` **4/4 xanh khi chạy một mình** trên **cả hai** cây *(tức nó chỉ đỏ trong lô)*; `segment-navigation` đỏ trong lô **trên cả baseline** *(before-hook hết 60 s chờ 40 hàng)*, và chạy một mình cho **9/10 xanh** trên cây story so với **5/5 xanh** trên baseline. 🔴 **1/10 so với 0/5 không phân biệt được hai cây** — nó không chứng minh có hồi quy, và **cũng không chứng minh không có**. ⇒ Món nợ không phải một ca đỏ cụ thể mà là **bàn đo không cho một phán quyết đọc được**: một bộ mà chế độ lô đỏ trên baseline thì mọi lượt story sau phải trả cùng cái giá phân xử này bằng tay *(lượt này tốn ~25 phút chạy đối chứng)*. **Chủ: một story hạ tầng e2e** *(cùng chủ với món `devServerIsUp()` đã ghi ở lượt rà 2.5b — không mở một mục thứ hai cho cùng một bàn đo)*.
  ⚠️ **Và giới hạn của chính lượt phân xử, ghi ra thay vì để người sau tưởng đã phủ:** e2e **không chạm một dòng nào** của Story 2.11 — không spec nào gọi `open_adjacent_chapter`, vì không đường sản phẩm nào sinh Chương thứ hai. Ba ca đỏ ở trên nói về **hồi quy**, không về **tính năng mới**.
  → 🟡 **ĐÓNG MỘT NỬA 2026-08-18 (Story 2.12).** Bốn nguyên nhân đã có bản vá *(AC1-AC4)*, và ba ca
  đỏ của lượt thứ chín có ứng viên trực tiếp trong số đó: `segment-navigation` đỏ vì before-hook hết
  60 s chờ 40 hàng ⇒ đúng khuôn `waitForExist`/đếm hàng mà AC3 đóng; `editor-typing-flush` xanh ở
  lượt chạy lại ⇒ đúng khuôn biên `FLUSH_WAIT_MS` mà AC4 đóng.
  🔴 **NHƯNG PHẦN NẶNG NHẤT CÒN NGUYÊN, và nó là chính mệnh đề của mục này:** *"bàn đo không cho một
  phán quyết đọc được"*. Lượt chạy trọn bộ nghiệm thu **CHƯA CHẠY** — xem món `AC7 · Task 8.4` ngay
  dưới. Và `attribution-focus` *(lần đỏ ② của `wdio.conf.mjs`, xanh 4/4 khi chạy một mình trên **cả
  hai** cây)* **vẫn chưa được chẩn đoán**: không bản vá nào ở trên nêu tên nguyên nhân của nó.
  ⚠️ ⇒ **Không được đọc bốn bản vá này thành "bộ đo đã nói thật".** Chúng gỡ bốn nguồn nhiễu đã đặt
  tên được; mệnh đề *"kết quả tái lập được"* là một phép **ĐO**, và phép đo đó chưa chạy.
  **Chủ: giữ nguyên.**

## Deferred from: SCP 2026-08-18b — rút `⌘Z` cho gộp/tách (2026-08-18)

- 🔴 **`⌘Z` bấm vào KHÔNG PHẢN HỒI GÌ — và đó là "rỗng IM LẶNG", đúng lớp lỗi trung tâm của dự án.** Đo: `grep -rn "KeyZ" src/commands/` = **0**, `grep -rniE "undo|redo" src/ src-tauri/src/` = **0 cơ chế**. Ice ký đường **(C)** ngày 2026-08-18 *(SCP 2026-08-18b)*: không dựng mô hình hoàn tác cho gộp/tách, đường quay lại là **gọi lại chính lệnh gộp/tách**. Quyết định ấy **đúng và đã đo**, nhưng nó để lại một chỗ hở mà chính nó không đóng: `⌘Z` là phím mà **cả thế giới phần mềm gán nghĩa "hoàn tác"**, và bấm vào một phím như thế mà màn hình không nói gì là **một màn hình im lặng về một điều nó biết** — `project-context.md:473-499` cấm đúng hình dạng này *(*"một danh sách rỗng không tự nói vì sao nó rỗng"*)*.
  ⚠️ **Vì sao nó KHÔNG tự đóng bằng dòng báo đã có:** `vi.json:101` *("Đã gộp hai câu…")* chỉ hiện **sau** một lượt gộp thành công. Nó không nói gì cho một lượt `⌘Z`, và cũng không nói gì **trước** một lượt `⌘/` đa-mảnh *(xem món nợ 🟡 ở lượt rà Story 2.9)*.
  **Hai đường đóng, chưa chốt:** ① một dòng báo ở `StatusBar` — *"Gộp/tách không hoàn tác được — tách lại rồi chuyển chữ sang ô dưới"*; ② quyết định để nguyên, **viết ra bằng chữ** kèm lý do. 🔴 Đường ② là một quyết định hợp lệ, **không** phải một lượt bỏ qua — nhưng nó phải được viết, không được im lặng.
  ⚠️ **Ràng buộc nếu chọn ①:** `⌘Z` có `primaryMod` nên nó **không** bị `keys.ts:510` chặn trong vùng gõ *(khác `Backspace`)* ⇒ nó bắn **cả khi con trỏ đang ở trong ô bản dịch**. Một dòng báo gắn vào đó sẽ hiện giữa lúc người dùng đang gõ — đúng chỗ dễ thành phiền. Và command mới phải đi qua `CommandRegistry` *(AD-34 §1)*, không cài thẳng trong `GridPanel.vue`.
  **Chủ: Ice** *(chốt hình dạng — một dòng báo, hay một quyết định để nguyên viết ra)*. Một quyết định phủ được **cả hai** chỗ hở: `⌘Z` và lượt `⌘/` đa-mảnh.

- ⚠️ **`AD-48` chưa được soạn, và nó KHÔNG chết theo AC5.** Ice rút AC5 nhưng câu hỏi *"`⌘Z` làm gì trong ứng dụng này"* vẫn phải có một chỗ đứng: Epic 3 trở đi còn thêm thao tác rời rạc *(duyệt glossary hàng loạt FR53, điền sẵn từ TM FR58, đề xuất AI)*, và mỗi thao tác ấy sẽ hỏi lại đúng câu này. Không viết ra thì mỗi epic phải đo lại từ đầu — đúng chi phí mà 47 `AD` kia tồn tại để tránh. Nội dung nay **nhỏ hơn nhiều** so với hồ sơ gốc: một mệnh đề *(Epic 2 không có mô hình hoàn tác; đường quay lại là gọi lại chính lệnh; và đây là lý do)*, cộng một câu khai rằng **AD-3, AD-5, AD-31 không đổi một chữ** *(khuôn AD-47 đã dùng)*. Hồ sơ: `planning-artifacts/ad-brief-2026-08-17-mo-hinh-hoan-tac.md` §11.4.
  **Chủ: Winston.** 🔴 Dev **không** tự soạn `AD` *(`project-context.md:461-463`)*.

---

## 🔵 2026-08-18 — Sprint Change Proposal 2026-08-18c: nửa NFR2 của Story 2.4 đã có AC SỐNG trở lại

Ba mục dưới đây ghi **"Chủ: Story 2.4"** nhưng từ 2026-08-14 chúng trỏ vào **AC12/AC13 bản chết** —
hai AC đó gọi tên `EditorPanel.vue` · `.doc` · `.sent` · `nearestSentenceTo()`, và cả bốn bị lượt
correct-course hôm đó xoá. Retro Epic 2 §F6 gọi đúng tên tình trạng: *"không phải giao nợ, đó là
**xếp nợ vào một cái hộp khoá**"*.

Ice ký ngày 2026-08-18: AC12 và AC13 viết lại trên bề mặt lưới, và **AC1 mở rộng phủ cả đường dời
con trỏ**. ⇒ Ba mục này nay trỏ được vào một AC **đang hoạt động**:

| Mục | Nợ | Nay thuộc AC |
| --- | --- | --- |
| `:3245-3262` | 2.5b đo dời con trỏ **706–770 ms** ở 9.850 câu, vượt trần NFR2 ~15× | **AC1** *(vế mở rộng)* + **AC12** hàng ① + **AC13** *(mốc so)* |
| `:3572-3583` | 2.5c thêm 39.400 phép đọc thuộc tính mỗi lượt dời con trỏ | **AC12** — đo trên cây **sau** 2.5c, so với 706–770 ms |
| `:4707` | 2.11 thêm một đường nạp lại toàn bộ lưới vào cùng bề mặt | **AC12** hàng ② |

🔴 **Và một hệ quả phải nói ra:** vì AC1 nay phủ đường dời con trỏ, nếu đường đó **không** hạ được
xuống 50 ms trong phạm vi hằng số mà Story 2.4 được phép chạm, đó là ca *"một ngưỡng trượt một
mình"* của **AC5** ⇒ dừng, báo Ice, **và Epic 2 dừng theo**. Cái giá đó đã được cân và ký.

### Bản ghi lịch sử — ba ảnh bàn đo của Story 2.2

`2-2-ban-do/ban-do-blink-light.png` · `ban-do-webkit-dark.png` · `ban-do-webkit-light.png` chụp bề
mặt `EditorPanel.vue`, mà Story 2.5b khai `Supersedes:` **4/8 AC của Story 2.2**.

⇒ Vế ① của **AC17** *(«chụp lại ba ảnh»)* được **BỎ** ngày 2026-08-18: chụp lại ảnh của một bề mặt
đã bị thay là sản xuất bằng chứng cho một thứ không còn tồn tại. Ba tệp **giữ nguyên**, khai bằng
chữ là **bản ghi lịch sử** — đúng khuôn action item **B5** của retro Epic 2.
⚠️ Vế ② của AC17 *(lời khai NFR15 sai ở `2-2-ban-do-editor.html:11`)* thì **vẫn đứng** — nó không
phụ thuộc bề mặt, và đã kiểm 2026-08-18 là còn nguyên.
**Chủ: Story 2.4** *(vế ②)*.

### 🟡 Hai tên panel chết NGOÀI phạm vi lượt này — giao cho B5

Lượt rà 2026-08-18 sửa `epics.md:2204` *(AC4 của Story 2.4)* và hàng Deferred *"Thư viện editor"*
của `ARCHITECTURE-SPINE.md`. Nhưng còn **hai** chỗ nữa gọi *"Panel Editor"* trong một mệnh đề
**đang hoạt động**, và cả hai nằm ngoài phạm vi Ice ký *(rà 22 AC của Story 2.4)*:

- `epics.md:477` — bảng giai đoạn, *"Panel Editor + AI Translation (BYOK/local) + Glossary…"* **(Chủ: John — PM.)**
- `epics.md:1833` — một AC của story khác: *"bôi đen một cụm từ ở Panel AI Translation hoặc **Panel Editor**"*

⚠️ Ghi ra thay vì tự sửa: `epics.md` là lượt riêng của Ice, và action item **B4** của retro Epic 2
đã đóng vế Epic 3 + Epic 6 nhưng **sót Epic 2**. Hai chỗ này thuộc đúng lượt rà đó.
**Chủ: Winston** *(action item **B5** — rà tồn dư tài liệu quy hoạch sau correct-course)*.

## Deferred from: 2-12-ha-tang-e2e-va-cong-con-thieu (2026-08-18)

- 🔴 **`AC7` · Task 8.4 · quyết định #8 — HOÃN, và đây là vế NẶNG NHẤT của story không đóng được.**
  AC7 đòi bộ e2e trọn bộ cho một kết quả **tái lập được**, với ngưỡng do quyết định #8 định nghĩa
  *(bao nhiêu lượt liên tiếp · máy rảnh hay máy bận · một ca đỏ có nguyên văn thì tính đạt hay
  không)*. Ice ký **chữ ký #0 ngày 2026-08-18: làm phần độc lập với Story 2.4, hoãn Task 8.4 và
  quyết định #8** — vì bảng §Điều kiện khởi hành chụp một cây **trước** bản vá NFR2, và một bản vá
  NFR2 chạm đúng đường nóng mà chữ ký #8 đang đo.
  ⚠️ **Điều kiện gỡ:** Story 2.4 đóng. Nó đang `in-progress` và **chủ là ICE** — bản ghi
  `sprint-status.yaml` lượt 18/8 (b): *"loadavg 162,88 → 111,35 trên 16 nhân, so với 7,19 mà AC22 đã
  gắn cờ ⇒ lưới 6 điểm và phiên NFR2 KHÔNG chạy được hôm nay"*.
  🔴 **Và ghi thẳng cái mà story này KHÔNG mua được:** bốn bản vá *(AC1-AC4)* gỡ bốn **nguồn nhiễu
  đã đặt tên được**. Chúng **không** chứng minh bộ đo đã nói thật — mệnh đề đó là một phép **ĐO**, và
  phép đo đó chưa chạy một lượt nào. Cửa chặn ② vì thế **chưa đóng trọn**.
  **Chủ: Ice** *(gỡ điều kiện)* → **story hạ tầng e2e kế tiếp** *(chạy phép đo)*.

  → ✅ **ĐÃ ĐÓNG 2026-08-19 (Story 2.12 · AC7).** Ice báo Story 2.4 đã chạy xong phép đo, và điều
  kiện gỡ được kiểm chứ không tin suông: **cây sạch tại `8457bf3`, `EDITOR_IDLE_MS`/`EDITOR_HARD_CAP_MS`
  nguyên vẹn** ⇒ 2.4 sinh ra **số đo, không sinh ra bản vá** ⇒ lo ngại phía sau chữ ký #0 *(biên
  1.500 ms tự nới sau khi 2.4 vá)* **không thành hiện thực**. Và AC4 vốn đã xoá hẳn phụ thuộc ấy.
  **Chữ ký #8, Ice 2026-08-19: `n = 1` lượt trọn bộ, và phải XANH 11/11.** Lượt **thứ mười hai**
  thoả: **11 passed / 0 failed / 13m01s / exit 0**, không một dòng `Error in`; máy loadavg
  **4,19 → 3,23** trên 16 nhân. Task 4.4 *(vế "máy bận")* đi cùng phán quyết này theo chữ ký #0b.

- 🔴 **`n = 1` KHÔNG chứng minh bộ e2e hết chập chờn — nó chứng minh bộ XANH ĐƯỢC. Hai mệnh đề
  khác nhau, và chỉ mệnh đề thứ nhất đã được mua.**
  Giới hạn này được **nêu ra trước khi Ice ký** và Ice **giữ nguyên** — ghi ở đây thay vì để nó tan
  vào một dấu ✅. Lý do nó đáng một mục riêng: `n=1` đúng bằng thứ retro Epic 2 đã **đính chính** —
  lượt chốt C3 kết luận *"ổn định"* trên `n=2` và sai; `wdio.conf.mjs:70-80` còn giữ bản ghi ấy.
  ⚠️ Và bằng chứng còn nóng: **cùng cây, cùng máy**, lượt 10 và 11 cho **6 đỏ**, lượt 12 cho **0 đỏ**.
  Khác biệt giữa chúng là một bản vá thật — nhưng nó cho thấy khoảng dao động của bàn đo này rộng
  đến đâu khi một biến đổi.
  🔵 Ba ứng viên gây chập chờn **đã có tên và đã vá** *(AC1-AC4)*, nên cỡ mẫu cần để tin đã nhỏ đi
  thật — đó là lý do `n=1` không vô lý. Nhưng `attribution-focus` *(lần đỏ ② của `wdio.conf.mjs`)*
  **chưa bao giờ được chẩn đoán**: nó xanh ở lượt 12, và không bản vá nào nêu tên nguyên nhân của nó.
  **Chủ: story hạ tầng e2e kế tiếp** *(hoặc một lượt `n≥3` khi có máy rảnh — rẻ hơn nhiều so với
  phân xử bằng tay một ca đỏ trong lô, như lượt 2.11 đã tốn ~25 phút)*.

- 🔴 **`AC7` của Story 2.12 là một mệnh đề MỘT NỀN TẢNG — Ice ký đường (c) của quyết định #1.**
  Mọi phép đo, mọi bản vá và mọi lượt nghiệm thu của story này chạy trên **macOS/WKWebView**. Không
  một byte bằng chứng nào cho Windows/WebView2 — cộng dồn với bản ghi của retro Epic 2 *(B7: Epic 2
  khép lại với **0** bằng chứng Windows cho 145 ca Rust mới + 249 vitest + 11 spec e2e)* và action
  item **A5** của retro Epic 1 *(đã lỡ mốc **lần thứ hai**)*.
  ⚠️ Ice chọn (c) thay vì (a) đúng vì một dòng: (a) im lặng, (c) **nói ra** rằng khoảng mù dày thêm
  một epic nữa. Cùng chi phí, trung thực hơn.
  🔴 Cụ thể **chưa đo trên Blink** ở riêng story này: `import()` động qua Vite dev có trả về **cùng
  một module record** trong WebView2 không *(tiền đề của `support/panelReset.mjs` và
  `support/flushWait.mjs`, đo được là đúng trên WKWebView)*. Đây là ngữ nghĩa ES module chuẩn nên
  rủi ro thấp — **và luật của kho vẫn cấm chấm đạt bằng suy luận đó**.
  **Chủ: Story 1.22** *(hạ tầng e2e hai nền tảng — gộp cùng hai vế Blink đã ghi ở trên, một lượt
  chạy trả lời cả ba)*.

- ⚠️ **Cạm bẫy 8 — vế phía CLIENT của AC1 chưa có phép kiểm nào, và Task 1.5 nói đừng lặng lẽ gộp
  nó vào AC1.** `devServerIsUp` + `assertModuleGraphHealthy` canh phía **máy chủ**; một cửa sổ
  `about:blank` với `document.body` **rỗng** là phía **client**, và nó **XANH ở mọi khẳng định
  *"không tìm thấy"***. Đo được ở lượt chạy đầu của chính bàn đo này *(`wdio.conf.mjs:181-187`)*.
  🔵 **Story 2.12 thu hẹp nó chứ không đóng:** `onPrepare` nay chỉ cho bộ chạy tiếp sau khi graph
  module đã chứng minh nạp được, nên **nguyên nhân phía máy chủ** của một cửa sổ trắng đã bị chặn từ
  gốc. Còn hở là nguyên nhân phía client *(webview nạp trước khi Vite sẵn sàng, một lượt điều hướng
  trượt)*.
  🔴 Và fixture nay có một hàng rào **gián tiếp** cho ca đó: `resetPanelState()` khẳng định lưới còn
  **0** hàng sau reset, tức nó chạm DOM thật — một cửa sổ trắng đi qua nó sạch, nên hàng rào ấy
  **không** thay được một phép kiểm riêng. Ghi ra thay vì để người sau tưởng đã phủ.
  ⚠️ **Chưa đo lại xem nó còn tái lập được không** — đòi một lượt chạy e2e thật, tức cùng điều kiện
  với món `AC7 · Task 8.4` ở trên. **Chủ: story hạ tầng e2e kế tiếp.**

- ⚠️ **Bộ đo nay bám `editorLastSavedAt` — một export của mã sản phẩm, và không cổng nào canh dây
  đó.** Ice ký **(a′)** của quyết định #4 *(2026-08-18)*: `e2e/support/flushWait.mjs` đọc
  `editorLastSavedAt` (`src/panels/editorPanelState.ts:246`) qua cầu `import()`. Đó là **0 dòng mã
  sản phẩm mới** — ô ấy đã tồn tại vì `StatusBar.vue` đọc chính nó — nhưng nó biến một export thành
  một **dây**: đổi tên nó làm bộ e2e đỏ, và `check:lint`/`vue-tsc` **không** thấy vì `e2e/**` không
  nằm trong `tsconfig`.
  🔵 Cầu đó **tự kêu**: `readLastSavedAt` ném một câu nêu đích danh nếu export vắng mặt, nên hỏng
  này không im lặng. Nhưng nó kêu ở **lượt chạy e2e tay**, không ở một cổng.
  ⚠️ Cùng hạng và cùng chủ: năm tên hàm `reset*` mà `support/panelReset.mjs` gọi, và đường
  `/src/panels/*.ts` mà nó `import()`.
  **Chủ: story hạ tầng cổng kế tiếp** *(một cổng tĩnh đối chiếu tên trong `e2e/support/**` với export
  thật của `src/**` — hoặc một chữ ký của Ice rằng vế "tự kêu" là đủ)*.

- ⚠️ **`resetDictSources()` không nằm trên đường gọi nào của sản phẩm.** Ice ký #2c *(dựng reset thay
  vì miễn trừ)*, và hàm ra đời đúng thế. Nhưng `disabled` là cấu hình tầng **Global**, chỉ nạp lại
  **một lần lúc khởi động** qua `loadDictSources` — nên nối nó vào đường đổi Tác phẩm sẽ xoá tập
  nguồn đã tắt trong bộ nhớ mà **không ai đọc lại từ đĩa**. Doc-comment tại chỗ ghi đủ lý do.
  🔴 ⇒ Hôm nay nó có **0 chỗ gọi**. Đó là một hàm đúng đắn chờ một chủ gọi hợp lệ *(một lượt dựng lại
  phiên có kèm `loadDictSources`)*, **không** một hàm thừa — nhưng khoảng cách ấy phải có tên, vì một
  hàm không ai gọi là một hàm chưa ai biết nó có chạy không.
  **Chủ: story kế tiếp chạm đường dựng lại phiên** *(hoặc Epic 3, nếu Glossary thêm một lượt nạp
  nguồn theo Tác phẩm)*.

- ⚠️ **Sàn quần thể: HAI sàn đã tụt dưới dải, và story chỉ được giao MỘT.** Task 7.5 nêu đích danh
  `check-layout.mjs` *(43 vs số thật 55 = 78,2%)* — đã nâng lên **46** (83,6%). Lượt đo lại đếm cả
  hai sàn đọc `src/**` và tìm ra sàn thứ hai: `check-commands.mjs::TS_FLOOR` *(30 vs số thật 39 =
  76,9%)* — đã nâng lên **33** (84,6%).
  🔴 **Ghi ra vì hình dạng hỏng của một sàn là hình dạng ÊM nhất trong kho:** nó **không bao giờ đỏ
  oan**, nó chỉ lặng lẽ thôi canh. Không lượt CI nào đỏ, không ai nhận ra.
  ⚠️ **CÒN HỞ — chưa ai kiểm:** `check-tokens.mjs::COMPONENT_FILE_FLOOR`,
  `check-commands.mjs::CLICK_FLOOR`/`DISPATCH_FLOOR`/`COMMAND_FLOOR`/`SELECTION_SURFACE_FLOOR`,
  `check-i18n.mjs::RS_FLOOR`, `check-dict-build.mjs::RS_FILE_FLOOR`. Chúng đếm theo **hình dạng riêng
  của từng cổng**, không theo một phép `find` trần, nên đo chúng đòi chạy chính bộ đếm của từng cổng
  — và một phép đếm bằng tay ở đây là đúng thứ *"số chép sẽ lệch trong im lặng"*.
  🔴 **Và món này sẽ tái phát**, vì luật *"thêm tệp thì xét lại sàn"* hôm nay là một lời dặn trong
  chú thích, không một cổng. **Chủ: story hạ tầng cổng kế tiếp** *(một Kiểm trong `check:gates` đối
  chiếu mỗi sàn với quần thể thật của chính cổng đó, và đỏ khi tỷ lệ rơi dưới 80%)*.

---

## Deferred from: code review of 2-12-ha-tang-e2e-va-cong-con-thieu (2026-08-19)

Lượt rà **ba tầng** trên dải `6931e87..HEAD` *(hai commit mang scope `story-2.12`)*. Ba mục dưới đây
là thứ **không** nghiệm thu được ở story hiện tại — cả ba đều **latent**, tức đo được là **chưa sống
hôm nay**, và chính vì thế chúng đi vào sổ chứ không thành một patch. Mỗi mục có một chủ.

- 🟡 **`judgeModuleResponse` chỉ đặc cách đuôi `.json`** — `e2e/support/devServerHealth.mjs:78-100`.
  Mọi URL khác dưới `/src/**` bị đòi `content-type` chứa `"javascript"`, không thì chấm `bad` và cả
  bộ e2e dừng ở `onPrepare` với chẩn đoán *"Vite ĐANG CHẠY nhưng module graph đã VỠ"*.
  ⚠️ **Đo 2026-08-19:** `.css` dưới Vite dev trả `content-type: text/javascript` ⇒ **chưa trúng bẫy**.
  Ngày một `import icon from './x.svg'` hay một `.wasm` vào `src/**`, Vite trả content-type gốc và
  cổng tự sinh ra đúng loại **dương tính giả** mà nó viết cả một bảng số đo để chống.
  🔴 Và giới hạn này **không** có trong mục §GIỚI HẠN THẬT ở đầu tệp — mục đó chỉ nói về tệp mồ côi.
  **Chủ:** story đầu tiên thêm một asset non-JS/non-JSON vào `src/**` *(chưa có lịch)*.

- 🟡 **`extractSrcImports` bỏ template literal, và một chuỗi `"/src/…"` trong chú thích thành cạnh
  giả** — `e2e/support/devServerHealth.mjs:113-116`. Regex chỉ khớp `'…'`/`"…"`, không backtick.
  Hai chiều hỏng: ⒜ `` import(`/src/${x}.ts`) `` không sinh cạnh nào ⇒ module đó không bao giờ được
  thăm ⇒ **xanh oan** nếu nó vỡ; ⒝ một chú thích chứa `"/src/ghi-chu.ts"` thành một cạnh thật, và
  nếu URL đó không tồn tại thì cả bộ e2e **đỏ oan** vì một dòng chú thích.
  ⚠️ **Đo 2026-08-19:** `grep -rn "['\"]/src/" src/` cho **0** ca ⇒ chiều ⒝ chưa sống.
  **Chủ:** story đầu tiên dựng một `import()` động dưới `src/**` *(chưa có lịch)*.

- 🟡 **Test AC6 chỉ phủ nhánh GỘP của đường `INSERT` thứ hai, không phủ nhánh TÁCH** —
  `src-tauri/tests/segment_contract.rs:6324`
  *(`a_row_born_from_regroup_has_every_column_set_on_purpose_not_by_default`)*. Test chỉ `use`
  `merge_segments`; doc-comment của chính nó khai phạm vi là *"đường ghi `INSERT` thứ hai"*, mà câu
  `INSERT` ấy phục vụ **cả gộp lẫn tách** *(Story 2.8)*. ⇒ Một cột bị bỏ sót ở một lượt di trú mà
  chỉ ảnh hưởng hàng do **TÁCH** sinh ra sẽ không bị test này bắt.
  **Chủ:** Story 2.8 *(chủ gốc của gộp/tách)* — bù một ca `split_segment` cùng khuôn.

### Và một mục KHÔNG vào sổ, ghi ra để không ai tưởng nó đã có chủ

🔴 **`resetSegmentHistory()` chưa nối vào đường đổi Tác phẩm** *(`segmentHistoryState.ts:387`, 0 lời
gọi trong `src/**`)* **không** phải một món nợ — nó là một **quyết định đang chờ chữ ký của Ice**,
ghi ở §Review Findings của story. Lý do phân loại: cửa sổ hỏng dữ liệu **VĨNH VIỄN** mà doc-comment
của chính hàm ấy mô tả vẫn dựng được hôm nay, và `project-context.md` xếp lớp đó cao nhất. Một món
nợ có chủ là chỗ cho thứ **không nghiệm thu được**; đây là thứ nghiệm thu được và **chưa làm**.

### 🔴 Một vế nghiệm thu CÒN LẠI của lượt code review — có chủ, có lệnh, không một dấu ✅

**Lượt e2e trọn bộ thứ MƯỜI BA** → ✅ **ĐÃ ĐÓNG 2026-08-19 (Story 2.12)** — Ice chạy
`npm run test:e2e`: **11/11 xanh, 0 đỏ, 12m46s**, và ba đường mã mới **đã được kiểm là thật sự
chạy** *(hàng rào `resetPanelState` qua năm spec · `truncated` + hai ca tự kiểm qua `onPrepare` ·
`waitForGridText` hai lần trong `doiChuong`, một lần sau `reload()`)*. ⇒ Vế *"chưa nghiệm thu ở tầng
engine thật"* dưới đây **hết đúng**, ~~gạch ngang~~ thay vì xoá. Nguyên văn lúc mở:
**Chủ: Ice.** Lý do là một phép đo, không một thủ tục: lượt vá 2026-08-19 chạm **ba** chỗ trong
`e2e/**` — vế danh tính của `doiChuong` *(`segment-navigation.e2e.mjs`)* · hàng rào chờ-trạng-thái
của `resetPanelState` *(`panelReset.mjs`)* · vế `truncated` *(`devServerHealth.mjs` + `wdio.conf.mjs`)*
— và e2e là đường nghiệm thu **DUY NHẤT** cho hình dạng dây. Lượt 12 *(11/11 xanh, 13m01s)* đo trên
cây **trước** lượt vá, nên nó **không** là bằng chứng cho cây hôm nay.

⚠️ Ba cơ chế mới trong `e2e/**` đã qua đột biến ở tầng **hàm thuần** *(`selfCheckDevServerHealth`
đỏ được trên cả ba vị từ sai)*, nhưng **không** ở tầng **engine thật** — `happy-dom` không phải
WKWebView, và vế `requestAnimationFrame` của hàng rào `resetPanelState` chỉ chạy thật trong webview.
⇒ Đây là chỗ *"không nghiệm thu được ở tầng đang làm"*, đúng định nghĩa một món nợ.

---

## Deferred from: lượt push + kiểm tra CI (2026-08-19)

✅ **CHUỖI BẢY LƯỢT ĐỎ ĐÃ DỨT** — lượt `32215808717` trên `8a4a060`: **`completed/success`**,
`check (macos-26)` **6m28s** · `check (windows-2025)` **18m16s**. Lượt xanh trước đó là `64cf7cb`
*(2026-08-16)*, tức master đỏ **ba ngày** qua bảy lượt push liên tiếp: `440c6d5` · `4d72cd4` ·
`a664dac` · `8457bf3` · `0a03c68` · `d339257` · `fa70fe3`.

⚠️ **Hai khuyết tật XẾP CHỒNG, và cái thứ hai chỉ lộ ra sau khi cái thứ nhất được vá.** Ghi ra vì
nó là hình dạng đáng nhớ, không một chi tiết: `npm test` đứng **trước** `cargo test` trong job, nên
cái bẫy `-0` *(`fa70fe3`)* làm `cargo test` **không chạy lần nào** suốt bảy lượt — và nó che một ca
Rust đỏ ở dưới. ⇒ *"Vá cái đỏ đầu tiên"* **không** đồng nghĩa *"CI sẽ xanh"*; một job dừng ở bước
thứ k không nói gì về bước k+1. Mỗi lượt vá phải chờ một phán quyết CI mới, không suy ra.

⚠️ Số đo phụ, ghi để truy nguyên được: `29m31s` cho Windows trong `project-context.md` là **lượt
xanh ĐẦU TIÊN** *(cache lạnh)* — nó **vẫn đúng như một mốc lịch sử**, và `18m16s` hôm nay là cùng
job trên `Swatinem/rust-cache` đã ấm. Hai số không mâu thuẫn; đừng sửa số cũ.


🔴 **MỘT MÓN NỢ MỚI, VÀ NÓ KHÔNG PHẢI BẢN VÁ VỪA GIAO — bản vá đã xong, chỗ hở là CÁI CANH.**

Lượt kiểm CI sau khi push tìm ra: **CI đỏ NĂM lượt push liên tiếp**, sớm nhất 2026-08-17, gồm **cả
hai commit của Story 2.12** — story mà hồ sơ ghi *"7/7 AC đóng, cửa chặn ② đóng"*. Nguyên nhân đã vá
*(`fa70fe3`: `-x` với `x === 0` cho `-0`, và `Object.is(-0, 0)` là `false`)*. Nhưng vá xong lại để hở
đúng cái đã cho phép nó sống năm lượt:

- 🟡 **Không đường nào chạy vitest dưới một múi giờ KHÁC múi giờ người chạy.** `pre-push` chạy trên
  máy Ice *(UTC+7)*; runner CI chạy **UTC**. Ca `segmentHistoryTime` rẽ **ba nhánh theo dấu của
  offset**, nên hai môi trường đi hai đường mã khác nhau — và `pre-push` xanh trong khi CI đỏ **cùng
  đúng một lúc**, không một cái nào nói dối.
  ⚠️ **Đo 2026-08-19:** `TZ=UTC npx vitest run` tái lập **đúng nguyên văn** câu CI báo. Tức chỗ mù
  đóng được bằng **một biến môi trường**, không cần một runner thứ hai.
  🔴 Nhưng thêm một đường cưỡng chế là **sửa BA danh sách** *(`package.json` · `ci.yml` ·
  `.githooks/pre-push`)* và `check:gates` Kiểm D/E/F canh cả ba — nó là một **quyết định**, không
  một dòng cấu hình. **Chủ: Ice** *(chưa có lịch)*.
  ⚠️ Và ghi thẳng chỗ yếu của chính đề xuất đó: chạy trọn bộ hai lần cho **hai** múi giờ làm
  `pre-push` dài thêm ~4 s *(đo: trọn bộ 250 ca chạy 4,35 s)*. Rẻ — nhưng nó chỉ canh **hai** điểm
  trên một trục liên tục, nên nó **không** là *"đã canh mọi múi giờ"*.

- 🔴 **Và món nặng hơn, không phải chuyện kỹ thuật: KHÔNG AI ĐỌC KẾT QUẢ CI.** Kho có CI chạy mỗi
  push *(`ci.yml`, repo công khai)*, và nó đỏ **năm lượt** mà không lượt nào bị chặn, không lượt nào
  bị nêu. Action item **A5** của retro Epic 1 ghi *"nửa Windows không có đường nghiệm thu tại chỗ —
  263 ca xanh trên runner Windows là một ẢNH CHỤP, không một trạng thái được canh"*. Thực tế **nặng
  hơn mệnh đề đó**: đỏ trên **CẢ HAI** nửa, và cái được canh không phải *"nửa Windows"* mà là
  *"có ai đọc phán quyết không"*.
  ⇒ Đây **không** đóng được bằng một cổng trong kho — một cổng cục bộ không đọc được kết quả CI.
  Ba đường có thật, đều cần Ice chốt: ⒜ bật thông báo GitHub cho lượt đỏ trên `master`; ⒝ một bước
  `pre-push` hỏi `gh run list` về phán quyết của lượt push **trước** và KÊU nếu nó đỏ *(cảnh báo,
  không chặn — không để mạng chập làm một lượt push chết)*; ⒞ một mục trong khung retro/sprint-status
  đọc CI. **Chủ: Ice.**
  ⚠️ Nợ này **không** ghi là 🟡 vì chưa có gì được đóng một nửa — nó mở nguyên.

### Và một mục thứ hai, lộ ra CHỈ SAU KHI mục trên được vá — nó bị che 6 lượt

🔴 **`store_contract::the_wal_stops_growing_once_it_crosses_the_threshold`: *"hồi quy của tầng
Store, hay biến động runner?"* — CHƯA ĐO ĐƯỢC.**

Bản vá `fa70fe3` *(cái bẫy `-0`)* là thứ cho `cargo test` chạy được **lần đầu sau 7 lượt push** —
trước đó `npm test` đứng **trước** nó trong job và chết sớm, nên khâu Rust bị bỏ qua hoàn toàn.
Ngay lượt đầu chạy tới, `macos-26` đỏ ở ca WAL. Hai lượt sửa hình dạng đã giao ở `8a4a060`
*(đảo thứ tự hai mệnh đề · gỡ phép so tự tham chiếu)*, và cả hai **không** trả lời câu dưới đây:

- **Cửa sổ hồi quy:** `cargo test` **XANH** trên `macos-26` ở `64cf7cb` *(2026-08-16)*, rồi
  **không chạy trong CI lần nào** cho tới `fa70fe3` *(2026-08-19)*. Trong khoảng ấy:
  · `tests/store_contract.rs` — **0 dòng** đổi;
  · `core/store/mod.rs` — 3 dòng, **trơ** *(một `pub use` thêm `SEGMENT_TRANSLATION_ORIGIN_DDL`)*;
  · `core/store/schema.rs` — **+208 dòng di trú** *(story 2.7 `translation_origin`, và `440c6d5`)*,
    **và di trú chạy lúc `Store::open`**, tức nó đổi nội dung WAL ở thời điểm ca test bắt đầu. **(Chủ: một story hạ tầng kiểm thử kế tiếp — điều tra cửa sổ hồi quy CI.)**
- ⚠️ **Số đo đã có, và nó KHÔNG kết luận được:** `after_first` bằng nhau **từng byte** trên hai
  máy *(94.792 B)*, nên phần mở kho là **tất định**. Khác biệt nằm trọn ở đợt hai — máy Ice lớn
  thêm **0 B**, `macos-26` lớn thêm **115.360 B**. Điều đó **tương thích với cả hai** giả thuyết:
  ⒜ `walRestartLog` không rơi nhịp trên runner *(biến động máy — đúng hiện tượng chú thích
  2026-08-11 đã ghi cho Windows)*; ⒝ 208 dòng di trú mới dịch nhịp `nBackfill == mxFrame`.
  ⇒ **Hai điểm đo trên hai máy khác nhau không tách được hai giả thuyết ấy.** Đường tách duy
  nhất là **cùng một máy, hai cây nguồn**: dựng lại `schema.rs` ở trạng thái `64cf7cb` rồi đo lại
  trên **cùng** runner. **(Chủ: B7 — bảng nghiệm thu Windows, chủ Ice, `epic-2-retro-2026-08-18.md:378`.)**
- 🔴 **Vì sao nó là một món nợ chứ không một mục đã đóng:** `8a4a060` sửa một phép so **sai hình
  dạng** — nó đúng bất kể câu trên trả lời thế nào. Nhưng nếu câu trả lời là ⒝, thì có một hiệu
  ứng thật của lượt di trú lên nhịp WAL mà **không ai đo**, và nó sẽ lớn dần theo mỗi lượt thêm
  di trú. Đóng im lặng là để một hồi quy có thật đi qua dưới một bản vá bộ đo.
  **Chủ: Ice** *(chưa có lịch)*. Ràng buộc: cần **một** runner, hai cây nguồn — không phải một
  máy Windows, nên nó **không** nằm sau món nợ A5.

⚠️ **Và ghi thẳng một chỗ yếu của chính bản vá `8a4a060`:** trần `1/4` cho *mức lớn thêm* hiệu
chuẩn trên **n = 2 máy**. Hai điểm đo không vẽ được một phân bố. Ba lượt đột biến chứng minh nó
**phân biệt được** *(115.360 xanh · 655.360 đỏ 4× · 163.841 đỏ ở đúng biên)*, nhưng *"phân biệt
được"* khác *"hiệu chuẩn đúng"*. Một lượt CI sau vượt `1/4` thì **đọc hai câu in trong thông báo
trước khi nới** — chúng có mặt để lượt đó có dữ liệu thật mà cãi.

---

## Deferred from: 2-13-phan-loai-so-no-va-luat-khong-mo-coi (2026-08-19)

- 🔴 **Quyết định #5 của Story 2.13 CHƯA CÓ CHỮ KÝ, và cho tới lượt rà này nó chỉ sống trong văn
  xuôi rải rác** *(tầng Blind Hunter bắt được: *"không có mục nợ nào, không cờ trạng thái nào ở đầu
  story, để đảm bảo việc còn treo một chữ ký không bị quên"*)*. Nội dung: nhóm *"không bám bề mặt
  nào"* — **đọc HẾT** hay **lấy mẫu**. ⚠️ Vế **thực chất** đã được thi hành theo đường chặt nhất
  *(đọc hết 83 mục, không lấy mẫu)*, nên đây **không** phải một khoảng trống trong sản phẩm; nó là
  một **chữ ký còn thiếu** trên một quyết định mà cấu trúc Task 0 của story dành riêng cho Ice.
  🔴 Và con số của nhóm ấy **khác 36** như story gốc khai — bộ phân loại gốc không được ghi lại, nên
  `36` không tái lập được. **Chủ: Ice** — ký hoặc rút #5 trước khi story chuyển `done`.
  → ✅ **ĐÃ ĐÓNG 2026-08-19 (Story 2.13).** Ice ký: **XÁC NHẬN** đường đã thi hành *(đọc hết, không
  lấy mẫu)*. Cách đóng: một chữ ký, không một lượt làm thêm — vế thực chất đã đúng từ trước, mục nợ
  này chỉ tồn tại vì **chữ ký** còn thiếu. ⚠️ Vế *"con số nhóm khác 36 vì bộ phân loại gốc không
  được ghi"* **vẫn đúng** và ở lại trong story, không bị lượt ký này làm tròn lên.

- ⚠️ **Sàn quần thể `ITEM_FLOOR = 397` của `check:debt-owner` hiệu chuẩn trên MỘT phép đo** *(467 mục,
  2026-08-19)*. Sổ này chỉ dài ra *(AC4 cấm xoá mục)*, nên sàn sẽ ngày càng xa thực tế và ngày càng
  ít nghĩa — đúng cái bẫy mà `project-context.md` gọi là *"một sàn cũ là một sàn vô nghĩa"*.
  **Chủ: story kế tiếp thêm một khối `## Deferred from:` khiến tổng vượt 550** — xét lại sàn cùng lượt.

---

## Deferred from: 2-4-mui-tham-do-do-nfr18-va-nfr2-dong-thoi (2026-08-19) — lượt gỡ cổng Epic 3

- 🔴 **NFR18 TRƯỢT 120/120 mẫu, và con số đó CHƯA có bản vá.** Lưới sáu điểm chạy 2026-08-18
  19:45–22:22 *(121 lượt bắn · 120 `VALID` · 1 `RIG_FAIL`)*: cửa sổ mất dữ liệu tốt nhất của
  **toàn phiên** là **7,747 s** so với trần **5 s**, trung vị 13,4–16,6 s — vượt khoảng **ba lần**.
  Mọi điểm bị gắn cờ **BẤT ỔN** theo cả hai vế dung sai AC2.
  ⚠️ Và điều tìm ra quan trọng hơn con số: **lưới PHẲNG vì `wal_threshold_bytes` là một biến TRƠ** —
  đỉnh `.db-wal` không bao giờ vượt ~740 KB, nên từ 1 MiB trở lên ngưỡng chưa từng kích hoạt
  *(ở 16 MiB mới chạm 1/22)*, và `busy = 0` ở cả sáu điểm. ⇒ Cặp đánh đổi
  `wal_threshold_bytes` ⟷ NFR18 mà `ARCHITECTURE-SPINE.md:990` treo lên **không tồn tại trong tải
  này**; NFR18 hỏng ở **đường flush**. **Chủ: Ice** — hàng SPINE `:990` chưa được đóng, và dòng đóng
  nó bắt buộc phải khai sự thu hẹp đó kèm số (AC3).

- 🔴 **Phiên NFR2 thật (30 phút × n=3) CHƯA CHẠY — Ice chốt 2026-08-19 KHÔNG chặn Epic 3 vì nó.**
  Bàn đo nay chạy trọn vòng và lặp lại được **5/5 phiên**, nên thứ còn thiếu là **thời gian máy**,
  không phải một khuyết tật chưa biết. Số smoke *(30 s, `n=1`, thang `m` 122 segment)*: **9 %** frame
  vượt trần 50 ms ở hai phiên *(max 101–113 ms)* và **39 %** ở hai phiên khác *(max 313–321 ms)*.
  ⚠️ Hai cụm tách bạch trên cùng tham số **không** giải thích được bằng nhiễu đo — nó đòi một biến
  chưa kiểm soát, nghi can đầu là tải nền. **Chủ: Ice** — mở lại khi có một máy rảnh ~2 giờ 20 phút
  và màn hình mở khoá.

- 🔴 **App tụt lại ~15 giây sau một phiên gõ 30 giây, và đây là một mệnh đề về NFR2 chứ không về
  bàn đo.** Đo được qua cổng `settle_keys`: bộ đếm phím nóng nhảy **3 → 13** *(và 3 → 17 trên thang
  `l`)* — các phím gửi trong lúc chờ **không bị mất, chúng bị XẾP HÀNG** rồi xử lý cùng lúc khi app
  đuổi kịp. Biểu hiện cho người dùng: gõ liên tục rồi bấm một phím tắt thì **không có gì xảy ra
  trong nhiều giây**. Chưa có FR nào chở mệnh đề này. **Chủ: phiên đo NFR2 thật** *(mục ngay trên)*.

- ⚠️ **Story 3.4 là story rủi ro nhất của Epic 3, và nó vào với sổ nợ này còn mở.** Nó thêm một
  kênh trang trí lên **cột nguyên văn của lưới** — đúng đường nóng mà cả ba mục trên đang nói tới.
  **Chủ: Story 3.4** — đọc ba mục này trước khi viết dòng mã đầu tiên, và nếu số NFR2 vẫn chưa có
  thì nói ra trong story thay vì giả định nó đã an toàn.

## Deferred from: 3-1-mo-hinh-glossary-hai-tang-va-vong-doi-ba-trang-thai (2026-08-19)

- 🔴 **Mục Glossary tầng Tác phẩm của một `.atproj` đã đóng rồi mở lại KHÔNG phân giải được
  qua đường sản phẩm — dữ liệu vẫn nguyên vẹn trên đĩa, chỉ là không đường Rust nào nạp lại
  nó.** Story 3.1 dựng `core::glossary::entries_eligible_for_injection(resolver, global,
  work)` — hàm đầu tiên thật sự tiêu thụ tầng Work của `ScopeResolver` (xem mục 🟡 mới ở
  `deferred-work.md:602`). Nhưng `ScopeResolver::with_work` chỉ được dựng ở
  `commands::project::create_work`, tức lúc **TẠO MỚI** một Tác phẩm trong phiên hiện tại —
  không tồn tại đường mở lại một `.atproj` đã có trên đĩa (`OpenWorkState` khởi động luôn
  `None`, không command IPC nào ngoài `create_work_*` đặt được giá trị vào đó — cùng mệnh đề
  đã ghi ở `deferred-work.md:2465` cho Editor). Hệ quả riêng cho Glossary: người dùng thêm
  một mục tầng Tác phẩm, đóng ứng dụng, mở lại **cùng** `.atproj` đó — mục vẫn nằm trong
  `project.db`, nhưng không có `ScopeResolver::with_work` nào được dựng lại cho phiên mới để
  đọc nó, nên Epic 4 (`RagInjector`) sẽ không thấy mục đó cho tới khi đường mở lại tồn tại.
  Không vá được ở Story 3.1: mở một đường "mở lại `.atproj`" tạm bợ chỉ để phục vụ Glossary
  là đúng bẫy *"bộ tách tạm"* mà nhiều story trước đã tự cấm cho chính miền của chúng — đường
  mở lại là một quyết định kiến trúc của toàn Tác phẩm (menu Thư viện, `Indexer`,
  `library-index.db`), không phải một chi tiết của riêng một bảng. **Chủ: Epic 5** (đường mở
  lại `.atproj`) — nhặt món nợ này cùng lượt với món đã ghi ở `deferred-work.md:2465`.

- ⚠️ **`entries_eligible_for_injection(resolver, global, work)` nhận `global`/`work` là
  `BTreeMap` ĐÃ NẠP, còn `load_tier`/`insert_entry`/`confirm_translation` bị cấm gọi ngoài
  `core/glossary/**` (`glossary_boundary.rs::only_entries_eligible_for_injection_may_be_called_from_outside_glossary`,
  thêm ở lượt rà soát ba lớp 2026-08-19).** Hai mệnh đề này CĂNG với nhau: chỗ gọi hợp lệ
  đầu tiên ngoài module (Epic 4) cần một `BTreeMap<String, GlossaryEntry>` cho mỗi tầng để
  truyền vào, và cách duy nhất dựng nó hôm nay là `load_tier` — thứ cổng vừa cấm gọi từ bên
  ngoài. Story 3.1 cố ý KHÔNG giải bài đó ("vá tại chỗ, đừng dựng lại" — chỉ đạo của lượt rà
  soát): `entries_eligible_for_injection` có thể cần đổi chữ ký để nhận `&Store`/`Option<&Store>`
  thay vì `BTreeMap` (đúng khuôn `core::scope::store::load_global_config`), kéo theo một kiểu
  lỗi hợp nhất `StoreError`+`ScopeError` mà hàm chưa có hôm nay — hoặc một đường khác mà
  story đó tự quyết với đủ bối cảnh của Epic 4. **Chủ: story đầu tiên gọi
  `entries_eligible_for_injection` từ ngoài `core/glossary/**`** (ứng viên gần nhất: Epic 4,
  `RagInjector`) — đọc doc-comment của chính hàm đó (`core/glossary/store.rs`) trước khi gõ
  dòng đầu tiên.
  → ✅ **ĐÃ ĐÓNG 2026-08-19 (Story 3.1) — cùng ngày, lượt vá cuối do Ice ký.** Cổng vừa dựng
  ĐANG cấm đúng con đường duy nhất tới thứ nó bảo vệ — không phải một món nợ để chuyển giao
  cho Epic 4, mà một lỗi trong chính chỉ thị vá vừa ban hành, phải đóng ngay. Đóng bằng cách
  đổi chữ ký: `entries_eligible_for_injection(resolver: &ScopeResolver, global: &Store, work:
  Option<&Store>) -> Result<Vec<GlossaryEntry>, GlossaryError>` — hàm tự gọi `load_tier` cho
  từng tầng RỒI MỚI phân giải, đúng khuôn `core::scope::store::load_global_config(store:
  &Store)`. `GlossaryError` (mới, `core/glossary/store.rs`) là enum hai biến thể `Store(StoreError)`
  · `Scope(ScopeError)`, mỗi biến thể một `From` — không nuốt một họ lỗi vào họ kia, không
  `unwrap`. `load_tier` ở lại `pub` (vẫn cần cho `glossary_contract.rs` dựng fixture và
  canh mệnh đề "có mặt khi liệt kê" mà hàm phơi ra không trả lời được) nhưng không còn ai
  NGOÀI `entries_eligible_for_injection` gọi nó — `tests/**` không nằm trong phạm vi quét
  của `glossary_boundary.rs` (chỉ quét `src-tauri/src/**`) nên việc test gọi thẳng không
  đụng cổng. Ca `a_pending_work_tier_entry_shadows_and_disqualifies_a_confirmed_global_entry`
  giữ nguyên sức: gọi qua đúng chữ ký mới, và bằng chứng "lọc sau khi phân giải" vẫn đo được
  từ NGOÀI (đầu vào → đầu ra), không phụ thuộc cách tham số được dựng.

## Deferred from: 3-1-mo-hinh-glossary-hai-tang-va-vong-doi-ba-trang-thai (rà soát 2026-08-19)

- source_spec: `_bmad-output/implementation-artifacts/3-1-mo-hinh-glossary-hai-tang-va-vong-doi-ba-trang-thai.md`
  summary: Chính sách chuẩn hoá `source_term` — hạ chữ thường và chuẩn hoá Unicode — chưa
    được quyết, nên hai thuật ngữ khác nhau CHỈ ở chữ hoa (`Fire` / `fire`) hay ở dạng
    Unicode vẫn là hai hàng riêng dưới `idx_glossary_entry_source_term`.
  evidence: Story 3.1 đóng vế RỖNG (`CHECK` khoảng trắng) và vế KHOẢNG TRẮNG THỪA (cắt lúc
    ghi), nhưng cố ý dừng trước vế chuẩn hoá — đó là quyết định của đường KHỚP, không phải
    của bảng. Ba dữ kiện đã đo ở Epic 1 chi phối nó và cả ba nằm ở miền Story 3.4: chữ hoa
    có nghĩa với 1.635 đầu mục tiếng Anh và 184 nhóm chỉ phân biệt nhau bằng chữ hoa
    (`project-context.md` §Rỗng im lặng); luật của kho là *"hạ chữ thường là THÊM một khoá,
    không THAY khoá gốc"*; và `Matcher` của Story 1.12 — thứ Story 3.4 phải dùng lại theo
    AD-17 — đã mang sẵn ranh giới stemming cho tiếng Anh. Chốt chuẩn hoá ở tầng bảng hôm
    nay là chốt thay cho một story có nhiều bối cảnh hơn, và làm nó bằng một chỉ mục UNIQUE
    thì KHÔNG lùi được sau khi dữ liệu người dùng đã nằm trên đĩa.
    **(Chủ: Story 3.4 — khớp thuật ngữ theo ngôn ngữ.)**

## Deferred from: 3-1-mo-hinh-glossary-hai-tang-va-vong-doi-ba-trang-thai (vòng rà soát #2, 2026-08-19)

*Lượt rà soát này ĐÃ ĐÓNG tại chỗ: bảng ký tự khoảng trắng (bảy → 25 điểm mã, kèm ca đo đi
thẳng vào SQL), comment sai quan hệ hai lớp ở `insert_entry`, `Category` "ba giá trị" → bốn,
hai chuỗi `assert!` mất dấu nối dòng, và năm khoảng trống nghiệm thu. Sáu mục dưới đây là
những mục CÒN LẠI, không mục nào mồ côi.*

- 🔴 **`GlossaryEntry.translation` là trường `pub`, nên mệnh đề *"`is_confirmed()` là vị từ
  DUY NHẤT định nghĩa đã chốt"* (AD-36) không có gì cưỡng chế.** `entry.rs` viết mệnh đề đó
  ở doc-comment đầu module, nhưng struct phơi trường ra công khai và
  `glossary_boundary.rs::GLOSSARY_ONLY_SURFACE` không canh nó — một module Epic 4 gõ
  `entry.translation.is_some()` biên dịch sạch và qua cả mười ba cổng. Tức luật quay về
  **kỷ luật**, đúng thứ mà chính story này từ chối ở mọi chỗ khác. Hai đường sửa, cả hai
  đổi bề mặt công khai nên cần chữ ký: ① `translation` thành riêng tư + một accessor;
  ② thêm `".translation"` vào `GLOSSARY_ONLY_SURFACE`. **Chủ: story đầu tiên gọi
  `entries_eligible_for_injection` từ ngoài `core/glossary/**`** (ứng viên: Epic 4).

- 🔴 **`entries_eligible_for_injection(resolver, global, work)` có HAI nguồn độc lập trả
  lời cùng một câu *"đã mở Tác phẩm nào chưa"*, và không gì bắt chúng khớp nhau.**
  `resolver.has_work_tier()` là một; `work.is_some()` là hai. `apply_override`
  (`scope/mod.rs`) **không bao giờ đọc `self.work`** — nó chỉ dùng dữ liệu được truyền — nên
  một `with_work(..)` đi cùng `work: None` phân giải bằng nguyên tầng Global **trong im
  lặng**, và mục *chờ chốt* ở tầng Tác phẩm thôi không che mục Global nữa. Đó đúng là kết
  quả sai mà hàng 3 của I/O Matrix sinh ra để chặn, tới bằng một cửa khác. Không ca nào ghép
  một resolver mang tầng Work với `work: None`: `with_no_work_open_resolution_is_the_whole_global_tier`
  khẳng định `!has_work_tier()` rồi truyền `None`, tức hai vế luôn khớp trong mọi ca đang có.
  ⇒ Cần một `debug_assert_eq!(resolver.has_work_tier(), work.is_some(), …)`, hoặc một chữ ký
  không cho hai vế lệch được. **Chủ: Story 3.3** — story đầu tiên gọi hàm này với một
  `OpenWork.scope` thật, tức chỗ đầu tiên hai vế có thể lệch ngoài test.
  → 🟡 **ĐÓNG MỘT PHẦN 2026-08-20 (Story 3.3).** `debug_assert_eq!` đã thêm ở CẢ hai hàm
  đọc hai tầng (`entries_eligible_for_injection` và `resolve_term_for_quick_add` mới) —
  lệch nhau giờ nổ ngay trong debug/`cargo test`, đúng lưới mà mục này đòi. ⚠️ **Nhưng
  `debug_assert!` không bắn ở bản release** (`Cargo.toml` không đặt `debug-assertions =
  true` cho `[profile.release]`), nên trên đường sản phẩm PHÁT HÀNH, hai vế vẫn có thể lệch
  trong im lặng nếu một chỗ gọi tương lai tách rời `(&open.store, &open.scope)` ra khỏi
  nhau. **Chủ phần còn lại: Story 3.9** — chữ ký không cho hai vế lệch được (ví dụ một kiểu
  `WorkContext<'a> { store: &'a Store, resolver: &'a ScopeResolver }` gói cặp này thành MỘT
  tham số) vẫn chưa dựng.

- ⚠️ **`Vec<GlossaryEntry>` trả ra đánh rơi nhãn tầng, và `id` chỉ duy nhất TRONG một
  `Store`.** `entries_eligible_for_injection` gọi `resolved_entry.value().clone()` rồi bỏ
  `Resolved::tier()`. Hệ quả: một mục Global `id = 1` và một mục Tác phẩm `id = 1` cùng nằm
  trong kết quả, không trường nào phân biệt được — chỗ gọi nào khoá theo `id` sẽ đụng nhau,
  và không ai đọc được một thuật ngữ đến từ tầng nào (thứ `mockups/glossary-manage.html:169`
  đã vẽ). **Chủ: Epic 4** — đọc mục này trước khi thiết kế hình dạng dữ liệu vào `RagInjector`.

- ⚠️ **`GLOSSARY_ONLY_SURFACE` khớp định danh TRẦN (`insert_entry` · `confirm_translation`
  · `load_tier`) như chuỗi con, trên toàn `src-tauri/src/**`.** Chính doc-comment của
  `core/glossary/store.rs` dự báo TM/Prompt/Cấu hình AI sẽ lấy đúng khuôn module này —
  nghĩa là một `core/tm/store.rs::load_tier` là chuyện gần như chắc chắn xảy ra, và ngày nó
  ra đời cổng Glossary sẽ ĐỎ cho một tệp không hề chạm Glossary. Sửa bằng cách khớp đường
  dẫn có định tính (`glossary::load_tier`) thay vì tên trần. **Chủ: epic đầu tiên dựng module
  miền hai tầng thứ hai** (ứng viên: Epic 7, TM).
  → 🔵 **CẬP NHẬT 2026-08-20 (Story 3.2):** đổi tên hàm chạm đúng mục này, đúng như Code Map
  của story đã dự đoán. `insert_entry` đổi tên `insert_manual_entry` (mất tham số
  `term_origin`); `GLOSSARY_ONLY_SURFACE` nay là `insert_manual_entry` ·
  `confirm_translation` · `load_tier` — ba chuỗi con khác, cùng lỗ hổng. Mệnh đề CHÍNH (so
  chuỗi con trên toàn `src-tauri/src/**`, không so đường dẫn có định tính) KHÔNG đổi — mục
  này vẫn MỞ, vẫn cùng chủ.

- ⚠️ **Trigger một chiều chỉ canh `BEFORE UPDATE OF translation`.** `INSERT OR REPLACE`, hay
  `DELETE` rồi `INSERT` lại, đưa một mục đã chốt về *chờ chốt* mà trigger không hề nổ. Hôm
  nay không đường ghi nào của module đi lối đó, nên đây là một lỗ hổng của **mệnh đề**
  (*"vòng đời khoá bằng cấu trúc, không bằng kỷ luật"*) chứ chưa phải của hành vi. Đóng bằng
  một trigger `BEFORE DELETE … WHEN OLD.translation IS NOT NULL` — nhưng chỉ khi Story 3.9
  (quản lý Glossary, có xoá mục) đã quyết xoá một mục đã chốt là hợp lệ hay không, vì hai
  quyết định đó ngược chiều nhau. **Chủ: Story 3.9.**

- ⚠️ **`note` không được cắt khoảng trắng biên, khác `source_term` và `translation`.**
  `insert_entry` cắt hai cột kia rồi ghi `note` nguyên văn (`store.rs`), nên `"   "` thành
  một cách biểu diễn THỨ BA của "không có ghi chú" — trong khi doc-comment của
  `GLOSSARY_ENTRY_DDL` khẳng định vắng mặt và rỗng là CÙNG một điều. Một dòng `.trim()` là
  đủ, nhưng nó đổi dữ liệu người dùng gõ nên không tự quyết ở lượt rà soát. **Chủ: Story 3.3**
  (bề mặt đầu tiên cho người dùng gõ `note`).
  → ✅ **ĐÃ ĐÓNG 2026-08-20 (Story 3.3), Ice ký 2026-08-20.** `insert_manual_entry` và
  `update_manual_term` (mới) đều `.trim()` `note` trước khi ghi — cùng khuôn
  `source_term`/`translation`. `add_manual_term_trims_a_whitespace_only_note_down_to_the_empty_string`
  (`glossary_contract.rs`) khoá hành vi: `note = "   \u{3000}  "` ghi xuống `""`.

- ⚠️ **`entries_eligible_for_injection` quét trọn hai bảng và nhân bản mọi hàng, HAI lần,
  mỗi lượt gọi.** `load_tier` dựng một `BTreeMap` chứa bản sao của từng hàng; rồi vòng lọc
  `clone()` thêm lần nữa kể cả với mục sắp bị loại. Với 412 mục toàn cục (con số
  `resolve.rs` lấy làm ví dụ) và một lượt gọi cho MỖI câu được dịch, đây là đường nóng của
  Epic 4. Chưa đo, nên chưa gọi nó là vấn đề — nhưng phải đo trước khi `RagInjector` chạy
  thật. **Chủ: Epic 4** — đo trước, rồi quyết cache theo phiên hay đổi chữ ký.

- ⚠️ **`pinned_contract.rs::a_fresh_global_database_ends_at_the_pinned_entry_step` nay
  khẳng định phiên bản 4, tức bước `glossary_entry`, không phải bước `pinned_entry` mà tên
  nó nói.** Doc-comment đã ghi nhận việc giữ tên là có chủ ý, nhưng mệnh đề *"pinned_entry
  là bước cuối cùng đã có mặt lúc story đó chạy"* nay **không được ca nào kiểm**. Thêm một
  dòng `assert_eq!(GLOBAL_MIGRATIONS[2].sql, PINNED_ENTRY_DDL)` là đủ để tên hàm nói thật
  trở lại. **Chủ: story kế tiếp thêm một bước vào `GLOBAL_MIGRATIONS`.**

## Deferred from: 3-2-bang-cho-ung-vien-tach-han-khoi-glossary (thực thi 2026-08-20)

- source_spec: `_bmad-output/implementation-artifacts/3-2-bang-cho-ung-vien-tach-han-khoi-glossary.md`
  summary: Trigger `glossary_candidate_resolution_is_one_way` chỉ canh
    `BEFORE UPDATE OF resolution` — đúng lỗ mà mục ngay trên đã ghi cho
    `glossary_entry_lifecycle_is_one_way`, nay lặp lại ở bảng ứng viên. Một `DELETE` rồi
    `INSERT` lại cùng `source_term` không đi qua `UPDATE` nào cả, nên trigger không hề nổ —
    hàng mới sinh ra mang `resolution = NULL` (chờ duyệt), tức một ứng viên đã bị bỏ/duyệt
    "sống lại" mà không phạm `UNIQUE (source_term)` (hàng cũ đã bị xoá, ô chuỗi đó đang
    trống).
  evidence: Hôm nay không đường ghi nào của `core::glossary::candidate_store` đi lối
    `DELETE` + `INSERT` — `insert_candidate`/`approve_candidate`/`reject_candidate` đều chỉ
    `INSERT` (một lần, lúc sinh ứng viên) hoặc `UPDATE resolution` — nên đây là một lỗ hổng
    của MỆNH ĐỀ ("vòng đời khoá bằng cấu trúc, không bằng kỷ luật") chứ chưa phải của hành
    vi hiện có. Đóng bằng một trigger `BEFORE DELETE ON glossary_candidate WHEN OLD.resolution
    IS NOT NULL` (hoặc tương đương) — nhưng chỉ khi story sở hữu một đường ghi thật sự cần
    `DELETE`/`INSERT OR REPLACE` đã quyết ngữ nghĩa xoá một ứng viên đã quyết là hợp lệ hay
    không, đúng khuôn quyết định mà mục `glossary_entry` tương ứng chờ Story 3.9.
    **(Chủ: Story 3.9 — quản lý Glossary, chủ tự nhiên của mọi quyết định xoá/tái sinh.)**

- source_spec: `_bmad-output/implementation-artifacts/3-2-bang-cho-ung-vien-tach-han-khoi-glossary.md`
  summary: Một ứng viên có `source_term` trùng một `glossary_entry` ĐÃ CÓ SẴN (ví dụ mục đó
    đến từ nhập tay trước khi ứng viên được quét ra) thì `approve_candidate` sẽ luôn thất
    bại ở `UNIQUE INDEX idx_glossary_entry_source_term` — và vì `approve_candidate` không
    phân biệt lỗi đó với bất kỳ `WriteFailed` nào khác, ứng viên nằm lại bảng chờ VĨNH VIỄN,
    không đường nào tự thoát.
  evidence: `epics.md:2984-2985` đặt chỗ chặn đúng lỗ này ở LƯỢT QUÉT (Story 3.5) — quét
    không được sinh ứng viên cho một chuỗi đã có mục Glossary, chứ không phải để
    `approve_candidate` phát hiện muộn. Story 3.2 không có lượt quét nào để áp luật đó
    (`insert_candidate` là API thuần, không tự tra `glossary_entry` trước khi chèn — làm
    vậy là đặt một quyết định nghiệp vụ của Story 3.5 vào một hàm mà story đó chưa tồn tại).
    **(Chủ: Story 3.5 — quét ứng viên khi nhập tài liệu.)**

- source_spec: `_bmad-output/implementation-artifacts/3-2-bang-cho-ung-vien-tach-han-khoi-glossary.md`
  summary: `pending_candidates` sắp theo `ORDER BY source_term` — đối chiếu BYTE của
    SQLite, vô nghĩa cho chữ Hán (không theo âm/nét) và tiếng Việt (không theo bảng chữ cái
    có dấu) — và mệnh đề `WHERE resolution IS NULL` chưa có chỉ mục riêng, nên mỗi lượt gọi
    quét toàn bảng.
  evidence: Story 3.2 chưa có bề mặt duyệt hàng loạt nào cần một thứ tự CÓ NGHĨA với người
    dùng (tần suất giảm dần, theo AD của epic-3-context.md), và bảng chờ hôm nay quá nhỏ để
    đo được chi phí thiếu chỉ mục — cả hai là quyết định của Story 3.8 (duyệt hàng loạt một
    phím: "sắp theo tần suất giảm dần"), không phải của bảng.
    **(Chủ: Story 3.8 — duyệt hàng loạt một phím.)**

- source_spec: `_bmad-output/implementation-artifacts/3-2-bang-cho-ung-vien-tach-han-khoi-glossary.md`
  summary: Bốn hàm của `candidate_store` (`insert_candidate` · `pending_candidates` ·
    `approve_candidate` · `reject_candidate`) CHƯA vào `GLOSSARY_ONLY_SURFACE` của
    `glossary_boundary.rs` — hôm nay chỉ một doc-comment nói ra rằng chúng phơi dữ liệu
    THÔ/ghi thẳng và "nên" chỉ gọi được trong `core/glossary/**`, không cổng nào canh.
  evidence: Story 3.2 cố ý không thêm bốn tên này vào cổng vì CHƯA có chỗ gọi sản phẩm nào
    ngoài `core/glossary/**` để nghiệm thu quyết định đó — có nên hạn chế
    `pending_candidates`/`approve_candidate` triệt để như `load_tier`/`insert_manual_entry`
    hay không phụ thuộc vào hình dạng bề mặt IPC mà Story 3.3/3.5/3.8 dựng (ví dụ:
    `pending_candidates` rất có thể cần gọi được từ một `#[tauri::command]` mỏng của Story
    3.8, trong khi `load_tier` thì không — hai hàm không nhất thiết cùng một luật).
    **(Chủ: story dựng chỗ gọi sản phẩm đầu tiên cho bốn hàm này** — ứng viên gần nhất:
    Story 3.5 (`insert_candidate`) hoặc Story 3.8 (`pending_candidates`/`approve_candidate`/
    `reject_candidate`).**)**

- source_spec: `_bmad-output/implementation-artifacts/3-2-bang-cho-ung-vien-tach-han-khoi-glossary.md`
  summary: Khoá `UNIQUE (source_term)` của `glossary_candidate` chặn TRÙNG CHUỖI — hẹp hơn
    luật "cùng một cặp nguồn→đích" mà `epics.md:6115-6117` đòi cho đề xuất thu hoạch từ bản
    review (FR54/FR95, Epic 8). Hệ quả: nếu một `source_term` đã bị BỎ (`resolution =
    'rejected'`) với một đề xuất dịch A, và sau đó bản review phát hiện cùng chuỗi nguồn nên
    dịch thành B (một cặp KHÁC A), `UNIQUE (source_term)` vẫn chặn đứng — không phân biệt
    được "cùng chuỗi, đề xuất khác" với "cùng chuỗi, đề xuất y hệt đã bị bỏ".
  evidence: Bảng `glossary_candidate` của Story 3.2 không có cột "bản dịch đề xuất" (cố ý —
    xem §Never: đó là Story 3.7/Epic 8), nên khoá duy nhất hôm nay CHỈ CÓ THỂ dựa trên
    `source_term`. Việc phân biệt theo cặp X→Y đòi một cột mới cộng một chỉ mục UNIQUE mới —
    quyết định thuộc về story dựng chính cột đó.
    **(Chủ: Story 8.14 — hoặc epic sở hữu FR54/FR95, tuỳ số hiệu story cuối cùng khớp
    `epics.md:6115-6117`.)**

- source_spec: `_bmad-output/implementation-artifacts/3-2-bang-cho-ung-vien-tach-han-khoi-glossary.md`
  summary: Cột `resolution` của `glossary_candidate` không mang thời điểm quyết định — khác
    `glossary_entry`/`glossary_candidate.created_at`, không có `resolved_at`. Duyệt hàng
    loạt (Story 3.8) hay một báo cáo "đã xử lý bao nhiêu ứng viên hôm nay" không đọc được
    mốc thời gian đó từ chính bảng.
  evidence: I/O Matrix của Story 3.2 không đòi một AC nào cần thời điểm quyết định — chỉ cần
    `resolution` đúng ba trạng thái. Thêm `resolved_at` hôm nay là đoán trước một nhu cầu
    chưa ai xin, đúng luật `AGENTS.md`: "Năng lực chưa dựng không phải lệch spec, ghi nợ có
    chủ thay vì đoán trước".
    **(Chủ: Story 3.8 — duyệt hàng loạt một phím, story đầu tiên có khả năng cần hiển thị
    "đã xử lý lúc nào".)**

## Deferred from: 3-2-bang-cho-ung-vien-tach-han-khoi-glossary (rà soát ba lớp, 2026-08-20)

- source_spec: `_bmad-output/implementation-artifacts/3-2-bang-cho-ung-vien-tach-han-khoi-glossary.md`
  summary: `approve_candidate` viết cứng `note = ""` khi chèn `glossary_entry` qua
    `insert_entry_row`, và không có tham số nào cho người duyệt tự đính ghi chú lúc duyệt —
    trong khi `insert_manual_entry` (đường nhập tay) CÓ tham số `note`.
  evidence: Đây là một lược bỏ CÓ CHỦ Ý của story (§Never liệt các cột/tham số bị hoãn kèm
    chủ), nhưng riêng vế này chưa từng vào sổ. `candidate_store.rs::approve_candidate` không
    có đường nào khác gán `note` — mọi mục Glossary sinh từ một ứng viên đã duyệt LUÔN có
    `note` rỗng, kể cả khi người duyệt muốn ghi lại lý do/ngữ cảnh ngay lúc quyết định.
    **(Chủ: Story 3.8 — duyệt hàng loạt một phím, bề mặt đầu tiên cho người dùng thao tác
    trên `approve_candidate` và có khả năng cần một ô ghi chú nhanh.)**

- source_spec: `_bmad-output/implementation-artifacts/3-2-bang-cho-ung-vien-tach-han-khoi-glossary.md`
  summary: `already_decided_error` (`candidate_store.rs`) mượn hình dạng lỗi
    `SqlError::FromSqlConversionFailure` — ĐÚNG hình dạng mà `decode_candidate_origin`/
    `decode_resolution` dùng cho "dữ liệu trên đĩa đã trôi khỏi `CHECK`" (một lỗi HỎNG DỮ
    LIỆU, không nên xảy ra trên đường ghi đúng). "Ứng viên đã quyết, không quyết lại được"
    là một LỖI NGHIỆP VỤ BÌNH THƯỜNG (người dùng bấm đúp, một lượt duyệt hàng loạt đụng một
    `id` đã xử lý) — hai loại lỗi khác hẳn nhau về mức độ nghiêm trọng và cách xử lý ở tầng
    trên, nhưng ở `StoreError` cả hai đều chỉ là `WriteFailed { detail: String }`, không
    phân biệt được bằng kiểu.
  evidence: `StoreError::WriteFailed` không mang đủ cấu trúc để tầng gọi rẽ nhánh — `detail`
    là chuỗi chẩn đoán, không dùng để `match`. Bề mặt IPC đầu tiên chạm `approve_candidate`/
    `reject_candidate` (Story 3.3, theo Code Map của story: "story đầu tiên dựng bề mặt IPC
    chạm tới hàm này") phải quyết cách phân biệt hai lớp lỗi này trước khi ánh xạ sang
    `message_key` — ví dụ một biến thể `GlossaryError`/`CandidateError` mới bọc
    `StoreError` cộng một nhánh "đã quyết" riêng, cùng khuôn `GlossaryError::Store`/
    `GlossaryError::Scope` mà `store.rs::entries_eligible_for_injection` đã dựng cho một cặp
    lỗi khác biệt tương tự.
    **(Chủ: Story 3.3 — thêm nhanh thuật ngữ từ bất kỳ panel nào, story đầu tiên dựng
    `#[tauri::command]` chạm `candidate_store`.)**
  → 🔵 **CẬP NHẬT 2026-08-20 (Story 3.3) — TIỀN ĐỀ CỦA MỤC NÀY ĐO LẠI LÀ SAI, CHUYỂN CHỦ
    SANG STORY 3.8.** Story 3.3 xây xong bề mặt IPC đầu tiên của Epic 3
    (`commands::glossary`), và nó KHÔNG gọi `approve_candidate`/`reject_candidate`/
    `insert_candidate`/`pending_candidates` một lần nào — đúng §Never của chính story 3.3:
    *"Story này không chạm `candidate_store`."* `grep -c "candidate_store\|approve_candidate\|reject_candidate"
    src-tauri/src/commands/glossary.rs` = 0. ⇒ Mệnh đề *"bề mặt IPC đầu tiên chạm
    `candidate_store` là Story 3.3"* đã HẾT ĐÚNG trước khi kịp đúng — nó là một dự đoán ở
    Code Map của Story 3.2, không phải một sự thật đã xảy ra. **Chủ nay là Story 3.8**
    (duyệt hàng loạt một phím) — story ĐẦU TIÊN thật sự dựng `#[tauri::command]` gọi
    `approve_candidate`/`reject_candidate`, đúng ứng viên mà mục *"Bốn hàm của
    `candidate_store` CHƯA vào `GLOSSARY_ONLY_SURFACE`"` (mục ngay phía trên trong tệp này)
    đã nêu tên từ trước.

## Deferred from: 3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao (2026-08-20)

- source_spec: `_bmad-output/implementation-artifacts/3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao.md`
  summary: `impl From<ScopeError> for IpcError` đứng RIÊNG vẫn không tồn tại. Cầu nối duy
    nhất từ `ScopeError` qua biên IPC hôm nay đi qua `core::glossary::GlossaryError::Scope`
    (`impl From<GlossaryError> for IpcError`, `core/glossary/store.rs`), một cầu nối GIÁN
    TIẾP và CHỈ dành riêng cho module Glossary.
  evidence: `core/scope/mod.rs` doc-comment của `ScopeError` đã sửa tại chỗ (🔵 2026-08-20)
    để không còn khẳng định sai "không bao giờ vượt ranh giới IPC" — nhưng module tiếp theo
    cần một cầu nối tương tự (TM, Prompt, Cấu hình AI ở Epic 4/7) sẽ phải TỰ dựng một
    `GlossaryError`-style wrapper của riêng nó thay vì tái dùng, vì `ScopeError` cố ý không
    `Copy`/không mang `impl From` chung — mỗi module domain hai tầng tự quyết cách nó lộ một
    `ScopeError` ra ngoài (nếu có).
    **(Chủ: Epic 7 — TM, module hai tầng kế tiếp dùng `ScopeResolver`; hoặc bất kỳ module
    nào tới trước và cần một `impl From<ScopeError> for IpcError` đứng riêng thật sự.)**

- source_spec: `_bmad-output/implementation-artifacts/3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao.md`
  summary: `debug_assert_eq!(resolver.has_work_tier(), work.is_some(), …)` mới thêm ở
    `entries_eligible_for_injection`/`resolve_term_for_quick_add` KHÔNG bắn ở bản phát hành
    (`[profile.release]` không đặt `debug-assertions = true`, và `panic = "abort"` khiến
    một `debug_assert!` bật ở release là một quyết định phải cân nhắc riêng, không phải một
    lượt bật cờ). Trên bản đã đóng gói, hai vế `resolver`/`work` vẫn có thể lệch nhau trong
    im lặng nếu một chỗ gọi tương lai tách rời cặp `(&OpenWork.store, &OpenWork.scope)`.
  evidence: Task của Story 3.3 chỉ đòi "một `debug_assert_eq!`", không đòi một chữ ký cưỡng
    chế bằng kiểu — và một kiểu gói cặp `(&Store, &ScopeResolver)` thành MỘT tham số (ví dụ
    `WorkContext<'a>`) là một thay đổi chữ ký chạm mọi chỗ gọi `entries_eligible_for_injection`
    hiện có, ngoài phạm vi story này.
    **(Chủ: Story 3.9 — quản lý Glossary, lượt tiếp theo chạm cùng hai hàm này qua nghiệp vụ
    xoá/đẩy tầng, tự nhiên đọc lại chữ ký trước khi thêm quyền mới.)**

- source_spec: `_bmad-output/implementation-artifacts/3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao.md`
  summary: Đổi TẦNG của một mục Glossary đã có (chuyển một mục từ `project.db` lên
    `global.db` hoặc ngược lại) không có đường nào — `add_manual_term`/`update_manual_term`
    (Story 3.3) chỉ THÊM mới hoặc SỬA tại chỗ trong đúng một `Store`; không hàm nào đọc một
    hàng ở tầng này rồi ghi nó sang tầng kia.
  evidence: §Ask First của Story 3.3 liệt đích danh "Đổi tầng của một mục đã có... chủ Story
    3.9" — dải "Thêm thuật ngữ" cố ý KHÔNG dựng năng lực này, đúng ranh giới đã ký trước khi
    viết dòng mã đầu tiên. `mockups/glossary-manage.html` (nếu có) là màn hình quản lý đầy
    đủ, nơi thao tác "đẩy một mục từ Tác phẩm lên Global bằng một thao tác" thuộc về.
    **(Chủ: Story 3.9 — quản lý Glossary.)**

- source_spec: `_bmad-output/implementation-artifacts/3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao.md`
  summary: Một ứng viên (`glossary_candidate`) trùng `source_term` với một mục vừa được
    người dùng thêm TAY qua dải "Thêm thuật ngữ" (Story 3.3) sẽ nằm lại bảng chờ VĨNH VIỄN —
    `approve_candidate` chèn qua `insert_entry_row` dùng chung, và `UNIQUE INDEX
    idx_glossary_entry_source_term` chặn đứng lượt duyệt vì `glossary_entry` đã có hàng đó.
  evidence: Đây là chính lỗ mà Story 3.2 đã ghi nợ cho "Story 3.5 (lượt quét) hoặc chủ sở
    hữu FR54/FR95" — Story 3.3 KHÔNG đóng nó, mà làm nó DỄ XẢY RA HƠN: trước 3.3, đường ghi
    tay DUY NHẤT vào `glossary_entry` là qua test; nay người dùng thật có một dải bàn phím
    để thêm bất kỳ lúc nào, kể cả đúng lúc một ứng viên cùng chuỗi đang chờ duyệt. Chỗ chặn
    đúng vẫn là LƯỢT QUÉT (`epics.md:2984-2985`) — quét không được sinh ứng viên trùng
    `source_term` với một `glossary_entry` đã có — không phải `approve_candidate`.
    **(Chủ: Story 3.5 — quét ứng viên khi nhập tài liệu, đúng chủ mà Story 3.2 đã ghi; mục
    này chỉ nối thêm bằng chứng rằng Story 3.3 làm lỗ đó DỄ CHẠM hơn, không phải chủ mới.)**

- source_spec: `_bmad-output/implementation-artifacts/3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao.md`
  summary: Ba vỏ `#[tauri::command]` của `commands/glossary.rs` giữ khoá `OpenWorkState`
    suốt cả lượt `Store::write` — tức một giao dịch SQLite chặn — nên một lượt ghi Glossary
    chậm có thể xếp hàng mọi lời gọi IPC khác cũng cần `OpenWorkState`, gồm cả lượt đóng
    Tác phẩm.
  evidence: Lớp rà soát mù ngữ cảnh nêu, và tôi CHƯA đo. Nó kiểm được: so phạm vi giữ khoá
    ở `commands/glossary.rs` với `commands/chapter.rs` và `commands/segment.rs`. Nếu ba
    module lệch nhau thì một trong ba đang sai, và không cổng nào canh phạm vi giữ khoá.
    NFR2 (không frame nào vượt 50 ms) là mệnh đề duy nhất có thể đỏ vì chuyện này, mà nó
    chỉ đo được ở bàn đo tay. **(Chủ: Story 3.9 — quản lý Glossary, story kế tiếp thêm
    lượt ghi Glossary qua cùng ba vỏ đó, tức chỗ đầu tiên có lý do đo phạm vi giữ khoá.)**

- source_spec: `_bmad-output/implementation-artifacts/3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao.md`
  summary: `openGlossaryQuickAdd` luôn đặt `tierChoice = 'global'` kể cả khi đang có một
    Tác phẩm mở và `workTierAvailable` là `true` — người dùng vội bấm `↵` sẽ ghi vào tầng
    Global một thuật ngữ chỉ thuộc về Tác phẩm đang dịch.
  evidence: Hành vi này ĐÚNG spec (spec chỉ đòi "người dùng chọn tầng", không nói mặc định
    nào) nên không phải lỗi — nhưng nó là một quyết định sản phẩm chưa ai chốt, và hậu quả
    không đối xứng: ghi nhầm lên Global thì thuật ngữ riêng của một truyện rò sang mọi Tác
    phẩm khác, còn ghi nhầm xuống Tác phẩm thì chỉ thiếu ở nơi khác. Đẩy một mục từ Tác
    phẩm lên Global là thao tác một bước của Story 3.9; chiều ngược lại thì không.
    **(Chủ: Ice — đây là một lựa chọn sản phẩm, không phải một lỗi kỹ thuật; nêu lại ở
    Story 3.9 khi màn hình quản lý cho thấy hậu quả thật.)**

- source_spec: `_bmad-output/implementation-artifacts/3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao.md`
  summary: `<form>` của dải "Thêm thuật ngữ" không có tên khả truy cập nối với tiêu đề
    đang hiện (`.gqa-title`), nên trình đọc màn hình đọc lên một biểu mẫu không tên.
  evidence: Chưa đo bằng công cụ nào — kho có cổng tương phản (`check:tokens`) nhưng KHÔNG
    có cổng nào canh vai trò hay tên khả truy cập, nên cả dải này lẫn ba lớp phủ có sẵn
    (`ShortcutsOverlay`/`AttributionOverlay`/`SegmentHistoryOverlay`) đều chưa ai kiểm.
    ⇒ Đây là một khoảng trống CỦA CẢ KHO mà story này chỉ làm lộ ra, không phải một khuyết
    tật riêng của dải. **(Chủ: Story 3.9 — story kế tiếp dựng một màn hình Glossary đầy đủ
    bằng bàn phím, tức chỗ rẻ nhất để đặt luật tên khả truy cập một lần cho cả bốn bề mặt.)**

- source_spec: `_bmad-output/implementation-artifacts/3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao.md`
  summary: `editor-confirm-segment.e2e.mjs` ĐỎ trong lượt chạy cả bộ 2026-08-20 nhưng XANH
    khi chạy một mình và XANH khi chạy đúng cặp `attribution-focus` → nó. Spec chập chờn,
    và cả bộ e2e dùng CHUNG một thư mục Library tạm cho cả lượt (`wdio.conf.mjs:139`).
  evidence: Đo 2026-08-20 trên commit `3322399`. Lượt đầy đủ: 10 xanh / 2 đỏ trong 15m13s;
    `target_text` đọc lên là `"test"` thay vì `"Một lần ký là đủ."`, mà chuỗi `test` KHÔNG
    phải literal trong spec nào — tức rác trạng thái. Đã loại trừ hồi quy của Story 3.3 bằng
    ba chân: xanh khi chạy riêng (1m54s), xanh khi chạy đúng cặp đúng thứ tự (2m53s), và
    `maxInstances: 1` với dấu thời gian bốn worker đầu không chồng lấn (08:52:29 · 08:53:21 ·
    08:55:17 · 08:56:49). Spec đó dựa vào `browser.pause()` cố định — khuôn chập chờn kinh
    điển. 🔴 Hệ quả rộng hơn con số: một bộ e2e cho kết quả khác nhau giữa lượt đầy đủ và
    lượt lẻ là một bộ KHÔNG dùng làm cổng được, và nó đang là đường nghiệm thu DUY NHẤT cho
    mọi mệnh đề về webview thật. **(Chủ: Story 3.9 — story kế tiếp thêm spec e2e cho màn hình
    Glossary, tức chỗ đầu tiên chi phí của sự chập chờn này rơi vào một lượt phát triển thật.)**

- source_spec: `_bmad-output/implementation-artifacts/3-3-them-nhanh-thuat-ngu-tu-bat-ky-panel-nao.md`
  summary: Không spec e2e nào bôi đen được chữ bằng **chuột thật** — WebDriver pointer action
    trong WKWebView không sinh ra một `Selection`, nên `glossary-quick-add.e2e.mjs` phải đặt
    `Range` bằng mã trong trang sau một `realClick()`.
  evidence: Đo 2026-08-20, hai lượt liên tiếp: kéo bằng `browser.action('pointer')` cho
    `selection.toString() === ''` cả trước lẫn sau khi sửa hộp dòng đầu (`getClientRects()[0]`
    thay `getBoundingClientRect()`). Chọn được đúng chuỗi RỖNG chứ không phải chọn thiếu một
    phần là dấu hiệu không có vùng chọn nào được tạo, không phải sai số toạ độ. ⇒ Chặng
    *"chuột kéo ⇒ trình duyệt dựng Selection"* hôm nay KHÔNG có đường nghiệm thu nào trong kho.
    Đây là hành vi của WebKit chứ không phải mã dự án, nên mức ưu tiên thấp — nhưng nó có
    nghĩa là FR21 (Auto-Lookup bôi đen bằng chuột) cũng chưa từng được canh đầu-cuối bằng
    chuột thật. **(Chủ: Story 3.4 — khớp thuật ngữ và đánh dấu ở cột nguyên văn, story đầu
    tiên mà một vùng chọn SAI sẽ hiện thành đánh dấu sai trên màn hình.)**
