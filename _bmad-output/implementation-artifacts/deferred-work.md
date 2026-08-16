
## Deferred from: code review of 1-1-mui-tham-do-font-do-dung-luong-that-va-ra-giay-phep (2026-08-03)

- **`.memlog.md` của architecture còn `scope: 112 FR, 16 NFR`** — spine ghi 131 FR / 19 NFR, PRD hiện có 132 FR. Tệp memlog đã bị chạm trong lượt này (bump `updated`) nhưng dòng `scope` lỗi thời vẫn để nguyên. Có sẵn từ trước, không do Story 1.1 gây ra.
- **Chưa đo trên Apple Silicon / universal binary / Windows ARM64** — máy đo là Intel x86_64. Chênh lệch font gần như chắc chắn không đổi theo kiến trúc nhưng baseline thì có, và universal binary nhân đôi baseline. Báo cáo đã nêu ở §Việc chưa làm được nhưng không story nào nhận việc.
- **Chưa khai artifact phát hành chính thức cho Windows** — Tauri dựng được cả `.msi` lẫn NSIS. AC1 đòi `.msi`, nhưng nếu bản phát hành thật là NSIS thì con số NFR6 không áp cho thứ người dùng tải về. Thuộc Story 1.3 / 10.2.
- **Đường nạp font chưa từng chạy trên Windows** — cấu hình CSP + `assetProtocol` scope + `FontFace` API mới chỉ kiểm chứng trên macOS. ~~CI của Story 1.3 chỉ `cargo test` và build, không xác minh font nạp được lúc chạy~~ *(mệnh đề này đã SAI kể từ `ci.yml` có `check:scope:bundled` — chiều dương đi qua `font-src`)*. Thuộc Story 1.3 / 1.4.
  → ⚠️ **ĐÓNG MỘT NỬA 2026-08-03 (Story 1.4, Task 4).** Story 1.4 đưa **đường nạp thật của sản phẩm** (`src/tokens/fonts.ts`, bốn `FontFace` qua `resolveResource` + `convertFileSrc`) vào cùng pipeline. **Còn mở, và ghi thẳng thay vì đánh dấu đạt:**
  - *Nhìn thấy chữ hiện đúng nét trên Windows* vẫn cần một lượt runner có ảnh chụp. Bốn nét của `Source Sans 3` mới chỉ được dựng trên **Blink/macOS** (Chrome headless 2×).
  - **WKWebView chưa đo.** Đối chứng *thiếu* descriptor `{ weight: '200 900' }` **vẫn ra nét đúng trên Blink** — Blink đọc `fvar` và nội suy trục dù `@font-face` không khai dải nét. Nên bẫy "khoá ở `wght = 200`" mà `ARCHITECTURE-SPINE.md` cảnh báo **chưa tái lập được trên engine nào**, và mức độ nghiêm trọng thật của nó vẫn là ẩn số. Descriptor vẫn bắt buộc (đặc tả dựa vào nó), chỉ là *lý do* bắt buộc yếu hơn tài liệu đang khẳng định.
  - **`-webkit-font-smoothing: antialiased` trong `reset.css` là LÝ LẼ, chưa phải PHÉP ĐO** — chưa có ảnh chụp cạnh nhau của cùng một chuỗi trên hai nền tảng để chứng minh nó thu hẹp khoảng cách độ đậm nét thay vì nới rộng.
- ✅ **ĐÃ ĐÓNG 2026-08-03 ngay trong lượt rà soát** — ~~Ba tệp giấy phép OFL chưa có AC nào đưa vào bundle~~ — báo cáo kết luận *"cả ba tệp giấy phép gốc phải đi kèm bản phát hành (FR38, FR109)"* nhưng không story nào cưỡng chế điều đó, và phép đo 20,30 MiB cũng chưa gồm chúng. Thuộc Story 1.2 / 10.5.
- **Rà NFR15 chưa đọc name ID 13/14 của tệp font phát hành** — đã mở `LICENSE` / `OFL.txt` trong zip mà đọc (đúng yêu cầu "rà tường minh"), nhưng chưa đối chiếu với trường License Description nhúng trong chính tệp `.otf`/`.ttf` sẽ được đóng gói.

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

- **Tệp nguồn tới qua symlink bị loại khỏi Kiểm A và không tính vào sàn** — `scripts/check-i18n.mjs:162-165` đẩy symlink vào `skippedLinks` rồi `continue`; nó chỉ hiện ra như một dòng `detail(...)` ở `:603`, **không bao giờ là `fail`**. Với 18 tệp `.rs` trên sàn 14 có đủ dư địa để giấu tệp bằng đường này mà sàn vẫn qua. Hoãn: cây hiện không có symlink nào. Mở lại nếu một `.vue`/`.rs` symlink xuất hiện.
- **Gốc quét cứng ở `src/` và `src-tauri/`** — `scripts/check-i18n.mjs:179-180`. Comment `:175-178` lập luận glob được cố ý **nới rộng** và mọi thu hẹp phải đi qua `EXEMPT` có tên; nhưng chính hai gốc này là một lần thu hẹp lặng lẽ. Một `packages/`, `examples/` hay `e2e/` về sau vô hình với cổng trong khi `vueFiles.length >= 1` vẫn đúng. Hoãn: chưa có thư mục nào ngoài hai gốc. Mở lại khi cây mọc nhánh thứ ba.
  → ✅ **ĐÃ ĐÓNG 2026-08-04 (Story 1.9, Task 9).** `tools/` (chứa `tools/dict-build`) là nhánh thứ ba. Đã thêm vào gốc quét của `check-i18n.mjs` **và** miễn trừ TRỌN nó ở `EXEMPT` với tên + lý do (build tool không vào bản phát hành — AD-25, không có bề mặt giao diện, chuỗi của nó là chẩn đoán cho người dựng). Quần thể in ra sau miễn trừ **không đổi** — vẫn 27 `.rs` + 5 `.vue` — đúng như doctrine đòi: thêm gốc quét không phải cái cớ để quần thể phình lên trong im lặng.
- **`scanStyle` không có trạng thái `line_comment`** — `scripts/check-i18n.mjs:455-499`. Doc `:455` biện minh đúng cho CSS thuần (`url(//host/x.png)` là URL, không phải comment), nhưng trong `<style lang="scss">` thì `//` **là** comment và một comment tiếng Việt ở đó sẽ bị báo là vi phạm. Đúng kiểu hỏng đắt nhất — cổng đỏ trên comment thì bị gỡ trong tuần. Hoãn: chưa có `.scss` nào và không gì trong repo cấm dùng. Mở lại ngày đầu tiên có `lang="scss"`.
- **Sàn đếm tệp, không đếm nội dung** — `scripts/check-i18n.mjs:207-218`. `VUE_FLOOR = 1` được thoả bởi một `src/App.vue` chỉ có khoảng trắng. Sàn đóng được *"cây rỗng đọc thành sạch"* nhưng không đóng *"tệp rỗng đọc thành sạch"*. Hoãn: rủi ro thấp khi cây `.vue` còn đúng một tệp; mở lại khi Story 1.14 dựng bốn panel.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, AC11.1).** `VUE_FLOOR` 1 → **9** (thật: 11), `RS_FLOOR` 21 → **26** (thật: 32) ở `check-i18n.mjs`; `VUE_FLOOR` 4 → **9**, `TS_FLOOR` 10 → **16**, `COMMAND_FLOOR` 4 → **10** ở `check-commands.mjs`; `FILE_FLOOR` 5 → **26**, `COMPONENT_FILE_FLOOR` 4 → **23** ở `check-tokens.mjs`. Con số THẬT ghi vào comment cạnh từng hằng số, đúng khuôn `RS_FLOOR` đang có. Nghiệm thu: di dời `src/panels/` ⇒ `check-i18n` · `check-commands` · `check-layout` đều `abort()` kèm số thật; thêm `src/layout/` ⇒ `check-tokens` cũng `abort()`. ⚠️ Sàn ĐẾM TỆP thì một tệp RỖNG vẫn qua — giới hạn đó **không** đóng ở đây; nó được bù bằng sàn NỘI DUNG (`CLICK_FLOOR`/`DISPATCH_FLOOR`/`COMMAND_FLOOR`, Kiểm B của `check-i18n`).
- **`ipc_error_wire_shape` assert "không ký tự có dấu nào trên dây" là một mệnh đề vòng** — `src-tauri/tests/ipc_contract.rs:128-137` quét bản serialize của chính literal `IpcError` mà test tự dựng ở `:73-78`. Nó chỉ đỏ khi ai đó sửa fixture, và không quan sát đường sản phẩm nào — vì chưa có đường nào (`commands/mod.rs` mới chỉ có doc-comment). Doc `:121-127` và §Completion Notes gọi nó là *"mệnh đề trung tâm của AD-21, kiểm được bằng máy"*, rộng hơn thứ nó làm được. ~~Hoãn tới **Story 1.6**, khi `#[tauri::command]` thật đầu tiên cho một đường thật để quan sát.~~
  → 🔴 **CHỦ SỞ HỮU ĐÃ SỬA 2026-08-04 (Story 1.6): KHÔNG phải Story 1.6.** Câu trên được viết lúc chưa ai đọc kỹ AC của Story 1.6; đọc rồi thì thấy **không AC nào của story đó cần Rust**. Chuyển chế độ, tiêu điểm bàn phím và bố cục panel là **state UI**, và AD-1 nói thẳng đó là phần frontend được phép sở hữu (*"frontend chỉ render và giữ state UI (focus, cuộn, vùng chọn, bố cục panel)"*). Story 1.6 giao **0 dòng Rust** và không từ chối dựng một `#[tauri::command]` giả chỉ để đóng mục này — cùng ba lý do Story 1.5 đã từ chối: nó là mã sản phẩm không ai gọi, chạy nó cần webview cộng một lượt biên dịch profile `dev` riêng (đắt nhất trên macOS, hệ số ×10), và vòng chạy thật đến **miễn phí** ở story đầu tiên có nhu cầu IPC thật.
  → ~~**Nhận lại ở: Story 1.8** *(phân giải cấu hình hai tầng — đường IPC thật đầu tiên có nhu cầu đọc/ghi qua ranh giới)*, hoặc **1.9/1.11** nếu đường tra cứu chạm Rust trước.~~
  → ✅ **ĐÓNG 2026-08-04 (Story 1.8).** `ipc_error_wire_shape` nay serialize giá trị mà `commands::config::bootstrap_config(None)` trả về — **đường sản phẩm thật**, đúng hàm mà `#[tauri::command] wire::bootstrap_config` bọc lại, chạy đúng nhánh mà một `$APPDATA` không ghi được sẽ chạy trên máy người dùng. Và **không** phải một command giả: hàm nhận `Option<&Store>` để test gọi được mà không cần webview *(§Quyết định #6)*, chứ không phải để test có một thứ riêng để gọi. Đối chứng âm N14 *(cho `bootstrap_config(None)` trả `Ok` với cấu hình mặc định)* làm ca này đỏ.
- **`process.exit()` ngay sau `console.log` có thể cắt cụt chẩn đoán trên pipe Windows** — `scripts/check-i18n.mjs:873,876`. Mã thoát — tức phán quyết — vẫn nguyên; thứ mất là các dòng `file:dòng:cột` làm cổng dùng được, đúng trên nền tảng mà cổng được viết bằng Node để có mặt. Hoãn tới lượt runner thật của Story 1.3; xác nhận trong cùng lượt đó.

## Deferred from: 1-3-ci-toi-thieu-hai-nen-tang-moi-lan-push (2026-08-03)

- 🔴 **`connect-src` thiếu `asset:` ⇒ `fetch()`/`XHR` tới asset protocol CHẠY ở dev, GÃY ở bản đã đóng gói — im lặng.** Đo thật trên bản `.app` debug 2026-08-03: bốn sự kiện `securitypolicyviolation` nêu đích danh `connect-src`, `blockedURI` là chính URL `asset://`. CSP hiện là `connect-src 'self' ipc: http://ipc.localhost` — `font-src` và `img-src` **có** `asset:`, riêng `connect-src` thì không.
  - **Hệ quả 1 (đã xử lý):** chiều ÂM của Kiểm 3 không đo được ngoài chế độ dev. Kênh duy nhất đọc được mã HTTP đã bị chặn, còn `FontFace` trả **cùng một** `NetworkError` cho "403 scope chặn", cho "tệp có thật nhưng không phải font" (đo thật với `OFL-sourcesans3.txt`), và cho "404" — nên nó cho cùng kết quả dù hàng rào còn hay mất. Story 1.3 ghi `unmeasured` thay vì đoán. **Ice chốt 2026-08-03: giữ nguyên CSP, không nới `connect-src` chỉ để một phép kiểm đo được.**
  - **Hệ quả 2 (CÒN MỞ, và đây mới là phần đắt):** story đầu tiên `fetch` một tài nguyên `$RESOURCE/**` từ webview sẽ chạy tốt suốt lúc phát triển rồi hỏng ở bản người dùng cài — đúng lớp lỗi mà dự án này liên tục đi săn. Đáng ngờ nhất: `$RESOURCE/dict/**` **đang nằm trong** `assetProtocol.scope` (tức là webview *được phép* đọc từ điển) nhưng CSP thì cấm `fetch` nó. **Hai khai báo đang mâu thuẫn nhau** — hoặc bỏ `$RESOURCE/dict/**` khỏi scope (nếu chỉ Rust đọc từ điển, đúng AD-11), hoặc thêm `asset:` vào `connect-src`. Nối thẳng với mục *"`$RESOURCE/dict/**` nằm trong scope nhưng không nằm trong `bundle.resources`"* ở trên. Thuộc **Story 1.9 / 10.1**, story nào chốt trước thì giải luôn cả hai.
    → ✅ **ĐÃ ĐÓNG 2026-08-04 (Story 1.9, Task 10).** Gỡ `$RESOURCE/dict/**` khỏi scope — mâu thuẫn biến mất theo chiều SIẾT, đúng tiền lệ Ice 2026-08-03 *"giữ nguyên CSP, không nới `connect-src` chỉ để một phép kiểm đo được"*. `connect-src` **không đổi** (không thêm `asset:`) — CSP vẫn siết y hệt trước.

- **Bốn phép nghiệm thu của Story 1.3 CHƯA chạy vì chưa có lượt CI thật** (Ice chốt 2026-08-03: chưa đẩy lên remote). Pipeline đã dựng và kiểm chứng hết phần chạy được ở máy, nhưng những thứ sau **chỉ runner mới trả lời được** — xem §Completion Notes của story để biết chính xác cái gì còn thiếu:
  - **AC6** — ba số `.msi` (có font · không font · `downloadBootstrapper`) và hai dòng nghiệm thu NFR6. `.msi` **chỉ dựng được trên Windows**.
  - **AC7** — thời gian tường + phút tính phí, cache lạnh và cache nóng, cả hai nền tảng. ⚠️ Rủi ro đã biết: job biên dịch Rust **hai profile** (`dev` cho AC8, `release` cho AC1/AC6) và dựng **ba** bản `.msi`; trên macOS hệ số ×10.
  - **Task 11 hàng 4** — `#[cfg(windows)] compile_error!` phải làm **chỉ** job Windows đỏ trong khi macOS **vẫn xanh** (phép kiểm của `fail-fast: false` và của AC1 *"tách bạch"*). Ba hàng còn lại đã nghiệm thu tại chỗ.
  - **AC3 / Task 4** — mệnh đề *rào biên dịch C của `zstd-sys` · `libsqlite3-sys` · `aws-lc-sys` biến mất trên `windows-2025`* mới là **kỳ vọng đọc từ tài liệu**, chưa ai đo. Cùng chỗ: WiX v3 — mũi thăm dò nói *"Tauri CLI tự tải lần build đầu"*, tài liệu Tauri nói phải cài sẵn; **hai nguồn nói khác nhau**, lượt chạy đầu phân xử.

## Deferred from: code review of 1-3-ci-toi-thieu-hai-nen-tang-moi-lan-push (2026-08-03)

*Lượt rà soát ba lớp song song trên dải `847e933..HEAD` (gồm cả code Story 1.2). Các mục dưới đây là **hoãn** — thật nhưng không thuộc phạm vi sửa ngay. Phần cần vá nằm ở §Review Findings của story 1.3.*

- **[D2] Trạng thái AC8 chốt sau lượt CI đầu tiên có `check:scope`** — Ice chốt 2026-08-03 trong lượt rà soát. Hôm nay `deferred-work.md:14` (mục của Story 1.2) ghi *"✅ ĐÃ ĐÓNG (Story 1.3, AC8)"* trong khi chiều **âm** là `unmeasured`, và AC8 không có trong danh sách *"Còn thiếu gì để đóng story — đúng bốn thứ"*. AC8 đòi *"**cả hai** chiều"*, và mệnh đề không của nó cấm *"đánh dấu đạt"*. Quyết định phụ thuộc D1: nếu `npm run check:scope` chạy được trên runner thì chiều âm có lưới tự động và AC8 **đóng trọn**; nếu runner không mở được webview thì **hạ `:14` xuống "đóng một nửa, đã trả lại cho Ice"** và thêm AC8 vào danh sách còn thiếu thành mục thứ năm. Không đánh dấu đạt trước khi có lượt chạy đó.
- **[D3] `on: push` (mọi nhánh) + `on: pull_request` ⇒ ma trận chạy HAI lần cho mỗi commit trên nhánh có PR** — `ci.yml:26-27`. `concurrency.group: ci-${{ github.ref }}` (`:34`) không gộp được: push là `refs/heads/x`, PR là `refs/pull/N/merge` — hai group khác nhau nên `cancel-in-progress` không huỷ chéo. Repo **private**, macOS hệ số **×10**. **Ice chốt giữ cả hai trigger** (Task 2 yêu cầu tường minh cả hai): AC7 nghiệm thu bằng **số thật**, nên để lượt CI đầu đo đúng giá của việc nhân đôi rồi mới quyết — đúng §Ngân sách CI *"ghi số và dừng, không tự cắt"*. Ba đường xử nếu số không chịu nổi: bỏ qua loạt `pull_request` khi PR đến từ cùng repo · khoá `concurrency.group` theo `head_ref || ref` để hai loạt huỷ chéo được · giảm tần suất job nặng. *(Kèm theo và chưa giải: `cancel-in-progress: true` xoá luôn bảng số đo AC6 của lượt bị huỷ, trong khi AC6 đòi ghi số ở MỖI lần chạy.)*
- **[D4] Hai khoản chi biên dịch trong `Cargo.toml` đánh thẳng vào AC7** — **Ice chốt không đổi** (§File List không cấm đụng `Cargo.toml`; bảng Stack được cài trọn có chủ ý ở Story 1.2). Ghi lại để lượt tối ưu AC7 sau có chỗ bám: (a) `reqwest = "=0.13.4"` (`:52`) để nguyên default features nên kéo `aws-lc-sys` — biên dịch từ nguồn C — vào **mọi** lượt cache lạnh, trong khi chính manifest tự khai *"chưa có một dòng mã nào gọi tới"*; `default-features = false` bỏ được cả một stack TLS. (b) `crate-type = ["staticlib", "cdylib", "rlib"]` (`:16`) là để phục vụ iOS/Android của template Tauri, nhưng `bundle.targets` chỉ có `["dmg","msi"]` — hai artifact thừa được link ở mọi `cargo test` và cả **ba** lượt build release Windows dưới `lto = true` + `codegen-units = 1`. ⚠️ Sửa hai chỗ này làm số `.dmg`/`.msi` khác đi, nên nếu làm thì phải làm **trước** khi chốt baseline NFR6, không phải sau.
  → 🟡 **Trạng thái sau Story 1.9 (2026-08-04, §Quyết định của Ice #3): VẪN CHƯA ĐÓNG, chốt lần thứ ba là KHÔNG ĐỤNG.** Story 1.9 đo baseline NFR6 (`.dmg` cây nguồn hôm nay, không font/license: **2.334.696 byte**) TRÊN HIỆN TRẠNG hai khoản này — số đó **chưa phản ánh** khoản tiết kiệm nếu (a)/(b) được cắt. Tổng payload sản phẩm sau khi cộng `dict-core.db` là **178.492.550 byte**, còn cách trần 200.000.000 đúng **21.507.450 byte** — nếu Story 1.10 (bốn lớp gỡ rời) đẩy tổng sát trần, đây vẫn là hai đòn bẩy đầu tiên nên thử, và bây giờ đã có SỐ THẬT để cân nhắc thay vì một tối ưu mù.
  → 🔴 **Trạng thái sau Story 1.10 (2026-08-05): NFR6 đã VƯỢT trần THẬT, KHÔNG ĐỤNG vẫn chốt.** Story 1.10 giao hai lớp gỡ rời (Thiều Chửu + VietPhrase, phạm vi thu hẹp — Ice chốt 2026-08-05) và đo thật: tổng payload hôm nay **343.991.430 byte**, VƯỢT trần 200.000.000 byte đúng **143.991.430 byte**. VietPhrase một mình **160.083.968 byte**. Hai khoản (a)/(b) ở mục này **chưa đo tác động thật** — cần đo TRƯỚC khi dùng làm đòn bẩy, không suy đoán. Dev **không đụng** `Cargo.toml` ở story này (chốt lần thứ tư). Phán quyết đầy đủ + bảng kế toán: §Debug Log References Task 11 của story `1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap.md`. Quyết định xử lý VƯỢT: §Câu hỏi cho Ice #1 của story đó.
  → ✅ **CHỐT 2026-08-05 (Ice): CHẤP NHẬN VƯỢT TRẦN.** Không bỏ nguồn nào, không bỏ `sense_fts_nd` của lớp nào *(phá AC4)*, không đụng hai khoản (a)/(b) ở mục này. Payload sản phẩm **343.991.430 byte** trên trần **200.000.000** — vượt **143.991.430 byte**, và con số đó được **chấp nhận có ý thức** trên số ĐO THẬT, không phải bỏ sót. Mục [D4] này vì vậy **KHÔNG còn là đòn bẩy đang chờ** — nó trở lại đúng bản chất ban đầu: một khoản tối ưu **AC7 (thời gian build)**, không phải AC6 (dung lượng). 🔴 **Hệ quả cần Ice xử lý ở tầng PRD:** trần 200.000.000 byte của **NFR6 giờ mâu thuẫn với sản phẩm thật** — hoặc nâng trần, hoặc ghi rằng NFR6 không tính lớp gỡ rời *(VietPhrase 160.083.968 byte là lớp **gỡ rời**, mà FR36 nói sản phẩm phải chạy đầy đủ khi **không có** nó — đây là cách diễn giải tự nhiên nhất)*. Dev không sửa `prd.md`.

- **`timeout-minutes: 60` nhiều khả năng không đủ cho nhánh Windows ở lượt cache lạnh** — `ci.yml:59`. Nhánh Windows phải làm tuần tự: `npm ci` → `cargo tree` (giải toàn cây) → `npm run build` (vue-tsc ×2 + vite) → `cargo test` (biên dịch **profile dev** toàn cây gồm `aws-lc-sys`, `libsqlite3-sys`, `zstd-sys` từ nguồn C) → `tauri build --debug` → **ba** lượt `tauri build --bundles msi` **profile release** với `lto = true` + `codegen-units = 1`, mỗi lượt chạy lại `beforeBuildCommand`, và hai trong ba lượt tải ~127 MB WebView2 lúc build. Vượt 60 phút ⇒ job bị giết ⇒ mất cả số AC6 lẫn số AC7, và cái đỏ đó **trông giống hệt** một lỗi thật. Con số 60 là phỏng đoán, không phải đo — §Ngân sách CI nói **ghi số rồi để Ice quyết**, nên lượt chạy thật đầu tiên phân xử.
- **`--config` vô hiệu hoá mọi bất biến trong `config_invariants.rs`, và CI đang dùng nó ba lần** — `config_invariants.rs:166-190` chốt `devCsp` và cấm `tauri.<platform>.conf.json`, nhưng không có gì chặn `--config <file>` hay biến `TAURI_CONFIG`. Hai lớp phủ hôm nay (`tauri.nofonts.conf.json`, `bootstrapper.conf.json`) vô hại và đều có cổng riêng, nhưng lối đó mở toang: một lớp phủ tương lai đặt `app.security.csp` hay nới `assetProtocol.scope` cho bản build THẬT sẽ không làm test nào đỏ. Kèm theo, danh sách chặn ở `:178` chỉ liệt kê biến thể `.json` — Tauri còn nhận `tauri.macos.conf.json5` và `Tauri.macos.toml`.
- **Cổng phụ thuộc dùng DANH SÁCH CẤM, trong khi chính repo lập luận danh sách cấm là sai** — `check-deps.mjs:121-142` vs `config_invariants.rs:92-94` (*"Danh sách CHO PHÉP, không phải danh sách CẤM. Một danh sách cấm chỉ chặn được những hình dạng ai đó đã nghĩ ra"*). `BANNED_CRATES`/`BANNED_NPM` thiếu `tauri-plugin-shell`, `tauri-plugin-http`, `tauri-plugin-process`, `tauri-plugin-opener`, `@tauri-apps/plugin-http`, `@tauri-apps/plugin-shell`. `plugin-http` phá cả AD-1/AD-29 lẫn AD-15; `plugin-shell` nguy hiểm hơn hẳn `plugin-dialog` đang bị cấm. Hai phương pháp trái ngược nhau trong cùng một lượt giao hàng.
- **`walk()` đệ quy trong cổng phụ thuộc không có bộ nhớ đã-thăm** — `check-deps.mjs:95-99` duyệt `node.dependencies` đệ quy, thêm tên vào `npmNames` nhưng không dùng nó để chặn lặp. Với cây `npm ls --all --json` sâu/lặp (peer-dep, workspace) đây là công đệ quy mũ và có thể tràn stack — khi tràn, `abort()` in ra *"không đọc được cây npm"*, tức một lỗi công cụ đội lốt lỗi hạ tầng. Không tới hạn hôm nay (59 gói).
- **Hai chỗ tài liệu nội bộ đã lệch khỏi sự thật** — (a) `deferred-work.md:7` vẫn ghi *"CI của Story 1.3 chỉ `cargo test` và build, không xác minh font nạp được lúc chạy"*; mệnh đề này **đã sai** kể từ khi `check:scope:bundled` (chiều dương qua `font-src`) vào `ci.yml:124`. (b) §File List của story 1.3 khai không *"Không đụng `_bmad-output/planning-artifacts/**`"* nhưng dải commit của story có sửa `epics.md` (+14/−) và `prd.md` (+6/−) — nội dung sửa đúng với quyết định của Ice, chỉ là dòng không và bảng "Sửa" nói sai sự thật, nên một lượt rà soát sau sẽ không biết tầng PRD đã đổi.
- **Không có clippy · rustfmt · ESLint · Prettier · test runner frontend · quét CVE** — `ci.yml:88-130` chạy `cargo test` nhưng không `cargo clippy -- -D warnings`, không `cargo fmt --check`; không có `.eslintrc*`, `.prettierrc*`, `vitest.config.*`, `rustfmt.toml`, `clippy.toml`, `dependabot.yml`. Nặng hơn: `scripts/*.mjs` — **chính tầng cưỡng chế** — không được type-check (`tsconfig.json` chỉ include `src/**` + `env.d.ts`) và không có một test nào. Kèm theo: mọi crate ghim `=` vĩnh viễn **cộng** lệnh cấm tường minh `cargo-deny`/`cargo-audit` (`ci.yml:95-97`) ⇒ không có đường nào để biết một CVE xuất hiện trong cây.
- **`dict-manifest.toml` đặt ra một luật ba trường bắt buộc rồi không cưỡng chế bằng gì cả** — `dict-manifest.toml:9-18` viết *"Mỗi mục PHẢI có đủ ba trường"* và cảnh báo checksum sai *"hỏng im lặng đúng kiểu tệ nhất"*, nhưng tệp giao ở trạng thái comment toàn bộ và không parser/test nào đọc nó. Bất đối xứng ngược chiều với mức rủi ro: repo dựng cả một script Node + mã thoát để canh việc ai lỡ cài `tauri-plugin-fs`, còn tệp sắp mang SHA-256 của ~130 MB dữ liệu tải về thì không có cổng nào. Chủ sở hữu: Story 1.9 / 10.1.
  → ✅ **ĐÃ ĐÓNG 2026-08-04 (Story 1.9, Task 8/13).** `scripts/check-dict-manifest.mjs` — parser TOML tập con tự viết, đọc + phán quyết `[base]`/`[[detachable]]`, gắn vào `ci.yml` job `check`. `[base]` đã điền THẬT (`sha256` của `dict-core.db` 154.836.992 byte dựng ở Task 11, `source_version` ghép năm nguồn). Nghiệm thu đỏ-rồi-xanh 11 ca ghi ở Debug Log References của Story 1.9.
- **Trích dẫn dòng trong comment cưỡng chế đã rữa trước cả khi commit** — `check-scope-bundled.mjs:20` trích `Cargo.toml:56-61` cho khối `[profile.release]`, nhưng khối đó thật sự ở **`Cargo.toml:61-66`** (lệch 5 dòng). Cùng tệp `:7`, `:20` trỏ `deferred-work.md:13`/`:5` như thể chúng ở gốc repo; đường thật là `_bmad-output/implementation-artifacts/deferred-work.md`. Các comment này là cơ chế truyền tri thức duy nhất giữa chín epic, mà trích dẫn dòng cứng vào tệp còn đang sửa sẽ rữa nhanh hơn tốc độ ai đó đọc lại.
- **Nhánh nền tảng chỉ có `win32` / không-`win32`; và không bước build nào khớp một OS thứ ba** — `check-scope-bundled.mjs:60-62,80-82`: trên Linux `IS_WIN === false` ⇒ build với `--bundles app` (target chỉ hợp lệ cho macOS/iOS) và `binPath` trỏ vào `bundle/macos/…app/Contents/MacOS/`; cùng nhánh ẩn đó ở `scopeCheck.ts:93-95`. Song song: `ci.yml:138` và `:204` gác bằng `if: runner.os == 'macOS'` / `== 'Windows'` không có nhánh mặc định ⇒ thêm `ubuntu-*` vào matrix cho một job **xanh mà không dựng ứng dụng và không ghi phép đo AC6 nào**. Khối *"CHỖ MÓC CHO EPIC SAU"* mời gọi đúng việc mở rộng matrix này.
- **Action ghim bằng tag major trôi trong khi header khẳng định đã kiểm chứng phiên bản chính xác** — `ci.yml:15-17` ghi *"kiểm chứng qua GitHub API ngày 2026-08-03: `actions/checkout` v7.0.1 · `actions/setup-node` v7.0.0 · `Swatinem/rust-cache` v2.9.1"*, nhưng `:62,64,75` dùng `@v7`, `@v7`, `@v2`. Chính tệp này cấm `-latest` cho ảnh runner với lý do *"ảnh runner đổi dưới chân là một hồi quy giả"*; action còn nguy hơn ảnh runner vì nó **thực thi mã** trong job. Ghim theo SHA là hình dạng khớp với lời văn đang có.
- **`rust-version = "1.85"` là số trang trí** — `Cargo.toml:7` vs `ci.yml:70` (`dtolnay/rust-toolchain@1.97.1`). CI chỉ chạy một toolchain, cách MSRV khai báo 12 phiên bản. Một crate phụ thuộc nâng sàn thật, hay một cú pháp chỉ có từ 1.9x lọt vào, đều không làm gì đỏ.
- **`vite.config.ts` không nối `build.target`/`minify`/`sourcemap` với `TAURI_ENV_*`** — `vite.config.ts:9-19`. Hệ quả cụ thể: (a) esbuild dùng target `modules` mặc định, có thể phát cú pháp mà WebView2/WKWebView ở nền tảng tối thiểu chưa hỗ trợ, và không phép kiểm nào trong lượt này phát hiện được; (b) bản `tauri build --debug` mà `check-scope-bundled.mjs` chẩn đoán bằng `String(err)` được build **không sourcemap**, nên mọi lỗi frontend trong CI là stack đã minify. Comment `:5-8` liệt kê *"bốn thiết lập Tauri bắt buộc"* nhưng bỏ đúng nhóm này.
- **Gộp `stdout` + `stderr` vào cùng một chuỗi `log` có thể phá anchor `^VERDICT: …$`** — `check-scope-bundled.mjs:112-123` và `check-scope.mjs:52-60` nối hai `capture()` vào cùng biến `log` không phân tách theo dòng. Một chunk stderr (log WebView2, warning của Tauri) tới xen giữa lúc dòng `VERDICT: PASS` đang được ghi ⇒ regex không khớp ⇒ rơi vào nhánh *"Self-check chưa chạy tới nơi"* và exit 1 trong khi self-check đã PASS. Không tái lập được, sẽ bị đọc thành flaky rồi bị bỏ qua.

## Deferred from: 1-4-bo-token-mau-va-chu-hai-theme-co-kiem-tuong-phan-tu-dong (2026-08-03)

- ✅ **ICE ĐÃ PHÊ CHUẨN cả ba 2026-08-03** (lượt rà soát mã của story). ⚠️ **Còn mở: `DESIGN.md` chưa được sửa cho khớp.** Tới lúc đó, sổ `deviations` trong `src/tokens/tokens.json` là chỗ giữ sự thật, và `check-tokens.mjs` cưỡng chế rằng không có chỗ lệch nào khác **cộng thêm** rằng mỗi mục phải có `question` và `reason` không rỗng. Việc chỉnh `DESIGN.md` là **một lượt riêng của Ice** — giữ tiền lệ quyết định #3 ở Story 1.3, dev không tự sửa tài liệu quy hoạch. Ai làm lượt đó nhớ gỡ ba mục `deviations` trong cùng một commit, nếu không Kiểm A sẽ đỏ vì "deviation khai nhưng không khớp chỗ lệch nào".

- 🔴 **BA giá trị trong bảng token đang lệch khỏi `DESIGN.md`.**
  - `colors.dark.surface-accent` `#2c3a3b` → **`#283637`** *(§Câu hỏi cho Ice #1, đi theo mặc định của story là phương án A)*. Cặp `on-surface-variant` × `surface-accent` ở theme tối là **4,245:1 — trượt AA**; sau khi đổi là **4,505**. `error` trên cùng nền từ 4,519 lên 4,795.
  - `typography.lookup-gloss.lineHeight` `1.6` → **`1.66`** — **PHÁT HIỆN MỚI của lượt cài đặt**, chưa có trong §Câu hỏi cho Ice của story.
  - `typography.lookup-example.lineHeight` `1.6` → **`1.66`** — cùng lý do.
  - *Vì sao hai mục sau là phát hiện mới:* story bắt được `DESIGN.md` tự mâu thuẫn ở `read-title`/`lookup-headword` (họ `read` mà ở 1.3) và giải bằng cờ `wraps`. Nhưng `lookup-gloss` ("Nghĩa") và `lookup-example` ("Ví dụ và trích dẫn") ở **1.6** thì cờ `wraps` **không** giải được — chúng thật sự chạy thành đoạn, nên sàn 1.66 áp cho chúng. Đường thay thế duy nhất là khai `wraps: false`, tức nói dối cổng để cho xanh — đúng thứ AD-34 tồn tại để chặn. Chi phí thị giác: 0,87px và 0,75px mỗi dòng.

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

- **`scripts/check-tokens.mjs` không được type-check và không có test** — cùng hạng với mục *"không có clippy · rustfmt · ESLint · test runner frontend"* ở trên: `tsconfig.json` chỉ include `src/**` + `env.d.ts`, nên **cả tầng cưỡng chế** nằm ngoài mọi phép kiểm tĩnh. Bù lại một phần bằng nghiệm thu đỏ-rồi-xanh 28 ca (Task 3) — nhưng đó là test của *hành vi cổng*, chạy tay, không nằm trong CI. Một hồi quy trong chính script sẽ đi qua CI mà không ai biết. Thuộc lượt bổ sung công cụ frontend.

- **Bộ phân tích CSS của cổng là "đủ dùng", không phải một parser CSS thật** — `parseCssBlocks` bám dấu `{}` `;` trên văn bản đã che comment/chuỗi. Nó đúng cho CSS mà dự án đang viết, nhưng chưa xử `@supports` lồng sâu, CSS nesting của Vue SFC ở dạng lạ, hay `url()` chứa dấu `;`. Khi Story 1.14 dựng CSS thật và nhiều, **soát lại số khai báo mà cổng báo đã quét** — con số đó tụt xuống bất thường là dấu hiệu parser bỏ sót cả vùng, và một cổng bỏ sót im lặng thì xanh y hệt một cổng đang canh.
  → ✅ **ĐÃ SOÁT 2026-08-06 (Story 1.14, AC11 ⚠️(a)) — con số đi ĐÚNG CHIỀU.** Trước: *21 tệp (18 component) · 116 khai báo*. Sau: *32 tệp (29 component) · **195** khai báo*. **+79** khai báo, phần lớn từ `src/layout/dockview-theme.css`. Không dấu hiệu bỏ sót vùng: mọi khai báo của tệp đó **đi qua Kiểm B** *(chúng phải là `var(--color-*)`; một hex viết thẳng ở đó sẽ đỏ)*. ⚠️ Cây vẫn chưa có `@supports` lồng sâu hay `url()` chứa `;`, nên hai lỗ đó của parser vẫn **chưa được thử**.

- ⚠️ **BA MỆNH ĐỀ THỊ GIÁC của Task 4/5 đang đứng bằng VĂN XUÔI, không bằng bằng chứng tái lập được** *(Ice chấp nhận 2026-08-03 với điều kiện ghi ra đây)*. Trang thăm dò, bốn ảnh chụp và bộ đọc `fvar` sống ngoài repo có chủ ý (tiền lệ §Ranh giới phạm vi của mũi thăm dò Story 1.1: tài nguyên dùng một lần không vào cây nguồn). Hệ quả là không lượt rà soát nào sau này tái lập lại được ba mệnh đề sau từ cây nguồn:
  - *"Bốn nét `Source Sans 3` (200/400/600/700) phân biệt rõ trên chuỗi dày dấu tiếng Việt"* — dựng trên **Blink/macOS**, chưa đo trên WKWebView, chưa đo trên Windows.
  - *"`ui-label` (700) là nét THẬT, không phải nét đậm tổng hợp"* — cùng giới hạn engine.
  - *"Dưới `font-synthesis: none`, chữ Hán đứng thẳng trong khi phần Latin vẫn nghiêng thật"* — cùng giới hạn engine, và lời giải này **chưa có người tiêu thụ** (xem mục ngay dưới).
  - Số đo `fvar`/`name` thì **đọc lại được từ chính tệp font** nên đứng vững hơn hẳn ba mệnh đề trên; chúng đã được chép vào `src/tokens/README.md`.
  - **Nhặt lại ở đâu:** lượt đo NFR của **Story 1.9** và nghiệm thu cuối của **Story 10.9**, nơi đã có sẵn một lượt runner hai nền tảng để bấu vào.

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
  - Lưới thật là lượt rà soát khi **Story 1.14** dựng bốn panel trong `dockview`, nơi thứ tự focus thật sự phức tạp. Cùng hạng với mục *"`check-tokens.mjs` không được type-check và không có test"*.

- 🔴 **Nghiệm thu DOM chạy trên Blink (Chrome), KHÔNG phải WKWebView, và KHÔNG qua `tauri dev`.** Lý do đo được, không phải quên: cổng `1420` mà `vite.config.ts` ghim (`strictPort: true`) đang bị **một dự án khác của Ice** (`gdrive_suite_manager`) chiếm lúc đo, và `devUrl` trong `tauri.conf.json` trỏ cứng vào đó — mà §Ranh giới phạm vi của story không cấm đụng `tauri.conf.json`. Lượt đo chạy qua `npx vite --port 1431` rồi lái bằng Chrome. **Chưa đo:** `⌘1 ⌘2 ⌘3` đi qua **WKWebView** thật; `⌘1..3` đi qua **tầng OS** *(Chrome nuốt `⌘2` để chuyển tab — sự kiện được dựng trên `window` thay thế, tức tầng ứng dụng đã đo, tầng phân phối phím của OS thì chưa)*. Đừng viết "tương đương" bằng suy luận. **Nhặt lại:** một lượt `npm run tauri dev` khi cổng 1420 rảnh, hoặc lượt runner của Story 1.9 / 10.9.

- 🔴 **Ca Windows CHƯA ĐO** — không có máy Windows. Kiểm D của `npm run check:commands` chứng minh **tầng phân giải hợp âm** đúng ở cả hai nhánh `Mod → ⌘ | Ctrl` (nền tảng là một tham số tiêm vào, nên phép kiểm chạy được trên một nền tảng), nhưng nó **không** chứng minh `Ctrl+1` tới được webview trên Windows. Đúng tiền lệ bàn giao phép đo của Story 1.1 → 1.3. Thuộc **Story 1.3 / 10.9**, nơi đã có sẵn một lượt runner hai nền tảng để bấu vào.

- ⚠️ **`focus.next_panel` chưa có phím, nên hôm nay KHÔNG có đường bàn phím nào vào panel** — cố ý (§Quyết định thiết kế #5 của story: bốn panel chưa tồn tại; mọi phím ứng cử đều đang hoặc sắp có chủ; và AC6 cần một phần tử thật để `unbound()` có nhánh chạy). Hệ quả phải nói ra: trạng thái tiêu điểm của AC5 hôm nay **chỉ đến được bằng chuột**, và đó là một lỗ trong NFR17 cho tới khi phím được gán. Nhận ở **Story 1.14** *(thứ tự vòng xoay khi có `dockview`)* và **Story 1.21** *(màn hình gán phím)*.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, §Quyết định #2).** `focus.next_panel` = `Mod+Alt+→`, `focus.prev_panel` = `Mod+Alt+←`. Không đụng `Tab`, không đụng `⌥←` `⌥→` trần *(Chương trước/sau — `EXPERIENCE.md:148`, Story 2.11)*, không đụng `⌘⇧…` (không gian của UX-DR35).

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

- ⚠️ **Kiểm A chỉ canh `@click`** — `@keydown`, `@input`, `@change`, `@submit` **không** thuộc luật *"phải là đúng một `dispatch('<id>')`"*. Có chủ ý: chúng không phải "thao tác" theo nghĩa AD-34 §1 (một `@input` là dòng dữ liệu). Nhưng ngày **Epic 2** dựng Editor với `@keydown` mang thao tác thật, luật phải được xem lại — không phải nới regex một cách lặng lẽ.

- ⚠️ **`scripts/check-commands.mjs` không được type-check và không có test tự động** — cùng hạng với ba mục đã ghi cho `check-deps.mjs` · `check-tokens.mjs` · `check-i18n.mjs`: `tsconfig.json` chỉ include `src/**` + `env.d.ts`, nên cả tầng cưỡng chế nằm ngoài mọi phép kiểm tĩnh. Bù lại một phần bằng nghiệm thu đỏ-rồi-xanh **28 ca** (Task 10) — nhưng đó là test của *hành vi cổng*, chạy tay, không nằm trong CI.

- ⚠️ **Sàn của cổng đếm TỆP, không đếm nội dung** — `VUE_FLOOR = 4` (thật: 5) và `TS_FLOOR = 10` (thật: 13) đóng được *"cây rỗng đọc thành sạch"* nhưng không đóng *"tệp rỗng đọc thành sạch"*. Cùng mục đã ghi cho `check-i18n.mjs:207-218`. Mở lại khi Story 1.14 dựng bốn panel.

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

- ⚠️ **Chốt chống rơi `body` bắn-và-quên: `rAF` không chạy khi cửa sổ ẩn, và blur cho cáo buộc sai** — `src/commands/focus.ts:103-113`. Đây là **chuông báo tự động duy nhất** cho AC4, và nó có hai lỗ không canh gác: (1) `requestAnimationFrame` không chạy khi cửa sổ bị ẩn/thu nhỏ, nên chốt bị **bỏ qua đúng trên đường khởi động nền** — chỗ nó cần kêu nhất; (2) nếu người dùng bấm ra ngoài hoặc cửa sổ mất focus trong khoảng giữa `enter()` và callback, `document.activeElement` đọc ra `body` và chốt in một **cáo buộc sai** nêu đích danh một owner đã focus hoàn toàn đúng. Không có đường huỷ. Hoãn vì đây là chuông báo chứ không phải cơ chế — cả hai lỗ làm chuông kém tin, không làm focus hỏng. **Nhặt lại cùng lượt** dựng nghiệm thu DOM tự động *(cùng mục với "Nghiệm thu DOM chạy trên Blink" ở trên)*.

- 🔴 **AC4 của Story 1.6 ĐẠT MỘT PHẦN — vế panel chưa có đường dời focus tường minh nào chạy được** *(Ice chốt 2026-08-04 trong lượt code review)*. `src/panels/PanelFrame.vue:50-52` chỉ gọi `declareFocus`, không có `onActivated`/`enterFocus`. Đường duy nhất dời focus vào một panel là `focus.next()` qua command `focus.next_panel` — mà command đó **cố ý không gán phím** (§Quyết định thiết kế #5) **và cũng không có `@click` nào dispatch nó**: grep toàn `src/` cho đúng 3 lời gọi `dispatch()`, cả ba là `mode.*`. Hệ quả phải nói thẳng: handler của `focus.next_panel` là **mã sống nhưng bất khả đạt**, và vế *"mỗi chế độ và mỗi panel dời focus DOM tường minh tới điểm vào đã khai"* của AC4 hôm nay chỉ được thoả cho **chế độ** *(qua `onActivated` → `enterFocus`, đã đo)*; với **panel** nó chỉ được thoả bằng hành vi focus mặc định của trình duyệt khi bấm chuột vào một phần tử `tabindex="-1"` — **không** phải bằng `el.focus()` của ứng dụng. Đây là giới hạn của chính AC4, không chỉ của NFR17 *(mục "focus.next_panel chưa có phím" ở trên xếp nó dưới NFR17 — chưa đủ)*. **Lý do hoãn:** giữ §Quyết định #5 nguyên vẹn — gán phím hôm nay làm `unbound()` trả mảng rỗng và AC6 mất bằng chứng; còn cho `PanelFrame` tự `enterFocus` là thêm một hành vi focus tự động mà Story 1.14 có thể phải tháo ra khi `dockview` quyết thứ tự vòng xoay. **Nhặt lại ở Story 1.14** *(thứ tự vòng xoay panel)* và **Story 1.21** *(màn hình gán phím)*. Không đánh dấu AC4 đạt trọn cho tới lúc đó.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, AC9).** `focus.next_panel` **nay có phím** (`Mod+Alt+→`) và có cả `focus.prev_panel` (`Mod+Alt+←`), nên handler không còn là mã bất khả đạt. Vòng xoay đi theo **thứ tự bố cục thật** *(`visiblePanelsInLayoutOrder()` sắp theo `group.api.boundingBox`: trên→dưới rồi trái→phải)*, không theo thứ tự `declare()`; panel đã ẩn không có trong vòng. Và `onDidActivePanelChange` của dockview gọi `enterFocus(owner)` **tường minh** — nhưng CHỈ khi `origin === 'user'` *(xem mục mới bên dưới)*. Đo được trên Blink: bốn lần `Mod+Alt+→` đi hết bốn panel theo đúng thứ tự lưới rồi quay lại; ẩn một panel ⇒ vòng còn ba và panel đã ẩn không xuất hiện; rời Workspace rồi quay lại ⇒ không vạch tiêu điểm nào nói dối.
  ⚠️ **AC6 của Story 1.6 GIỮ ĐƯỢC bằng chứng:** `unbound()` nay trả về **bốn** `layout.toggle_*` thay vì `focus.next_panel`.

- ⚠️ **Bộ lọc phần mở rộng của cổng bỏ qua `.tsx` · `.mts` · `.cts`** — `scripts/check-commands.mjs:122,130-131`. `name.toLowerCase().endsWith('.ts')` sai với cả ba. Một tệp như vậy **không đóng góp gì** vào `tsFiles`, nên mọi `dispatch('…')` và `declareFocus('…')` trong đó vô hình với Kiểm B và Kiểm E, và nó cũng **không tính vào `TS_FLOOR`** — tức sàn không phát hiện được việc mất tệp. Hoãn vì dự án không dùng ba phần mở rộng đó và `tsconfig.json` không bật `jsx`. **Nhặt lại** nếu có story nào thêm `.tsx`, hoặc gộp vào lượt rà soát sàn cổng ở Story 1.14.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.14, AC11.3).** `walk(SRC_ROOT, ['.ts', '.tsx', '.mts', '.cts'])`. Nghiệm thu: một `src/layout/__probe.mts` và một `__probe.cts` mang `dispatch("khong.ton_tai")` **nay bị Kiểm B bắt** (trước lượt sửa: vô hình). ⚠️ `.d.ts` cố ý KHÔNG bị loại — một tệp khai báo không chở `dispatch()` nào nên nó chỉ làm quần thể to thêm, và một luật thừa là một chỗ để sai.

---

## Deferred from: 1-7-tang-ghi-du-lieu-mot-writer-noi-tiep-va-luoc-do-co-phien-ban (2026-08-04)

- 🔴 **SÁU con số `Tuning` là TẠM và CHƯA CÁI NÀO ĐƯỢC ĐO — chủ sở hữu là Story 2.4.** `src-tauri/src/core/store/mod.rs`, `impl Default for Tuning`. Chúng không đo được ở story này vì phép đo cần **Editor thật**: `wal_threshold_bytes` (AD-12) và nhịp flush (AD-35) **đánh đổi lẫn nhau** — phải đạt NFR18 *(mất ≤ 5 s)* mà không phạm NFR2 *(không frame nào vượt 50 ms)*. `ARCHITECTURE-SPINE.md#Deferred` và `epics.md:454` đã xếp cả cặp vào **Giai đoạn 2**.

  | Tham số | Giá trị tạm | Lý lẽ *(lý lẽ, không phải phép đo)* |
  |---|---|---|
  | `pool_size` | 4 | Đủ để quan sát được đọc chồng nhau; nhỏ để TRUNCATE không phải chờ nhiều reader |
  | `busy_timeout` | 5 000 ms | Dài hơn một lượt checkpoint bình thường, ngắn hơn ngưỡng người dùng cho là treo |
  | `checkpoint_tick` | 1 s | Độ phân giải của cả hai điều kiện kích hoạt |
  | `idle_before_passive` | 5 s | **Cố ý dài hơn** nhịp flush 2 s của AD-35, để checkpoint không đánh nhau với đường gõ |
  | `wal_threshold_bytes` | 4 MiB | Bằng đúng ngưỡng autocheckpoint mặc định của SQLite *(1000 trang × 4096 B)* mà AC3 vừa tắt — lấy lại đúng số nó bỏ lại, tức không đổi hành vi theo một hướng chưa ai đo |
  | `close_truncate_budget` | 2 s | Trần để `close()` không làm `check:scope` / `check:scope:bundled` đỏ |

  **Đừng đọc chúng như đã hiệu chỉnh**, và đừng để một story sau tưởng chúng đã qua một lượt đo. Doc-comment của `Tuning` và của module đều khai là tạm. **Story 2.4** đo lại cả sáu trên Editor thật.

- ⚠️ **AC5 nghiệm thu CƠ CHẾ, không nghiệm thu NGƯỠNG THẬT.** `store_contract.rs::the_wal_stops_growing_once_it_crosses_the_threshold` chạy với `wal_threshold_bytes = 64 KiB` và blob 32 KiB — số thu nhỏ để ca chạy trong dưới một giây trên cả hai nền tảng *(§Testing standards cấm `sleep` dài)*. Nó chứng minh **vế (b) tồn tại và kích hoạt được khi chưa rảnh**; nó **không** chứng minh 4 MiB là con số đúng cho một phiên gõ thật. Cùng chủ sở hữu: **Story 2.4**.

- ⚠️ **Ca AC5 phụ thuộc vào nhịp tương đối giữa `checkpoint_tick` và khoảng cách hai lượt ghi.** SQLite chỉ quay `.db-wal` về đầu tệp khi một giao dịch ghi bắt đầu đúng lúc `nBackfill == mxFrame` (`walRestartLog`). Ca test vì thế đặt tick 3 ms / gap 10 ms và ghi lý do ngay tại chỗ. Trên một runner chậm hơn hẳn, tỷ lệ đó có thể lệch. Đã chạy 5 lượt liên tiếp trên máy dev không dao động; **chưa chạy trên runner CI lần nào** — cùng danh sách với bốn phép nghiệm thu của Story 1.3 đang chờ lượt CI thật.

- ~~⚠️ **Lỗi mở kho hôm nay chỉ ra `stderr`, không tới người dùng.**~~ `src-tauri/src/lib.rs::open_global_store` ghi chẩn đoán rồi **đi tiếp** thay vì chặn khởi động; lúc viết, story đó không dựng `#[tauri::command]` nào nên **không có bề mặt để nói**.
  → ✅ **ĐÓNG 2026-08-04 (Story 1.8).** Đường đã nối trọn: `try_state::<Store>()` rỗng ⇒ `bootstrap_config` trả `IpcError` mang `code = "store.open_failed"` · `MessageKey::StoreOpenFailed` · `params = {"store": "global"}` · `retryable = false` ⇒ `src/config/bootstrap.ts` bắt và đặt vào `configError` ⇒ `src/App.vue` vẽ một dải báo lỗi **không chặn**, nội dung qua `tError(err)`. Không khoá `MessageKey` mới nào và không chuỗi `vi.json` mới nào — năm khoá kho của Story 1.7 đã đủ.
  → ⚠️ **Và `open_global_store` vẫn đi tiếp — nay đó là quyết định đúng chứ không còn là ít tệ nhất.** Ứng dụng lên bằng cấu hình mặc định và **nói ra** rằng nó không đọc được kho, thay vì không lên. Doc-comment của hàm đã được sửa cho khớp.
  → ⚠️ **Một hở còn lại, ghi ra thay vì đánh dấu đạt:** dải báo lỗi chỉ hiện khi Rust **trả lời**. Một phiên `npm run dev` không có cầu IPC cho `configError = null` có chủ ý *(dựng một `IpcError` giả ở đó làm mọi lần chạy dev mọc một dải "Không mở được kho dữ liệu" — một câu sai, và một câu sẽ dạy người đọc bỏ qua đúng dải đó)*. Hệ quả: đường hiển thị này **chưa từng chạy trong một webview thật** — nghiệm thu nó cần một `$APPDATA` chỉ-đọc, và đó là một bảng nghiệm thu tay. Giao lại **Story 1.15** *(story tiếp theo mở một kho thứ hai, tức story tiếp theo có lý do thật để chạy bảng đó)*.
  → ⚠️ **VẪN CHƯA ĐÓNG sau Story 1.15 — ghi thẳng thay vì đánh dấu đạt.** Story 1.15 mở kho thứ hai thật (`project.db` qua `commands::project::create_work`) nên lý do kỹ thuật để chạy bảng này nay đã có, nhưng phiên triển khai của Story 1.15 là một agent CLI **không có công cụ điều khiển GUI desktop** (không dựng được cửa sổ Tauri thật rồi đọc màn hình bằng mắt, không có cầu debug-protocol tới WKWebView như Chrome DevTools). Bảng nghiệm thu tay này **vẫn** cần một người vận hành thật, hoặc một bộ tự động hoá desktop mới (`cliclick`/tương đương) chưa có trong môi trường build. Giao tiếp: **QA người trước khi phát hành**, hoặc story kế tiếp có công cụ GUI automation.

- ⚠️ **`tests/**` được miễn trừ khỏi phép quét ranh giới của AC2** — `src-tauri/tests/store_boundary.rs` quét `src-tauri/src/**` và **không** quét `tests/**`. Miễn trừ có tên và có lý do *(ba ca của AC6/AC7 phải dựng một database ở một phiên bản lược đồ và một chế độ journal cho trước — đúng thứ `core::store` tồn tại để mã sản phẩm không làm được)*, nhưng nó là một miễn trừ thật: một test tương lai **có thể** mở kết nối ghi thứ hai vào một kho thật mà không cổng nào báo. Cùng hạng với miễn trừ `src-tauri/tests/**` của `check-i18n.mjs`. Mở lại nếu số tệp test chạm `rusqlite` vượt quá `store_contract.rs`.

- ⚠️ **AC7 nghiệm thu trên một fixture ở chế độ `delete`, không phải trên một `global.db` WAL thật của một bản tương lai.** `a_newer_schema_is_refused_without_touching_a_single_byte` dựng fixture ở `journal_mode = delete` để khẳng định *"`.db-wal`/`.db-shm` không được tạo"* một cách sạch sẽ. Một database WAL thật do một bản sau viết ra **sẽ** làm SQLite tạo `-shm` ngay khi mở — tệp `.db` vẫn không đổi một byte *(mệnh đề trung tâm của AC7 vẫn giữ, và hợp đồng thứ tự trong `Store::open` là thứ giữ nó)*, nhưng hai tệp sidecar xuất hiện rồi biến mất khi kết nối đóng. Ghi ra để lượt sau không tưởng phép kiểm rộng hơn thứ nó đo.

## Deferred from: code review of 1-7-tang-ghi-du-lieu-mot-writer-noi-tiep-va-luoc-do-co-phien-ban (2026-08-04)

- **Lỗi checkpoint/backup lúc chạy đều gắn nhãn `StoreError::OpenFailed`** — `src-tauri/src/core/store/pragmas.rs:249`, `wal_checkpoint()` luôn map lỗi SQLite thành `OpenFailed`, dùng cả lúc `Store::open()` (sao lưu) lẫn liên tục sau đó từ luồng checkpoint nền (đã chạy nhiều giờ). Vô hại hôm nay vì các lỗi này chỉ đi qua `Display` rồi log qua `shared.note()`, chưa bao giờ đi qua `IpcError::from`. Nếu một story sau đưa checkpoint diagnostics lên UI qua `message_key()`, một checkpoint lỗi sau nhiều giờ chạy sẽ hiển thị nhầm "Không mở được kho dữ liệu — dữ liệu chưa được nạp". Giao lại cho story nào nối chẩn đoán checkpoint lên giao diện.
- **Sao lưu bằng `fs::copy` không nguyên tử, không xác minh sau khi chép** — `src-tauri/src/core/store/schema.rs:137`. Nếu hết đĩa giữa chừng, tệp `.bak-v{from}` có thể bị cắt cụt; `open()` vẫn đúng đắn dừng lại (trả `Err`) nếu chính lệnh copy thất bại nên dữ liệu sống không gặp rủi ro, nhưng nếu copy "thành công" mà bị cắt cụt do lỗi hệ thống tệp không báo qua `Result`, tệp sao lưu trông hợp lệ mà thực ra thiếu. Đáng làm cứng hơn (copy-rồi-rename nguyên tử, hoặc so kích thước) nhưng không chặn Story 1.7.
- **`GLOBAL_TARGET_VERSION` nêu ở Task 6 chưa từng được tạo** — thay bằng hàm `pub(crate) target_version(migrations) -> u32` tính động ở `src-tauri/src/core/store/schema.rs:82`. Hợp lý hơn vì `StoreSpec.migrations` đã trở thành trường theo từng instance, nhưng là một sai khác so với hạng mục đã liệt trong story, và tầm nhìn `pub(crate)` nghĩa là không chỗ gọi bên ngoài nào (vd. IPC chẩn đoán tương lai) truy vấn được phiên bản target mà không mở `Store`. Không ảnh hưởng AC nào; ghi lại cho minh bạch.
- 🔴 **`Checkpointer::shutdown()` có thể để luồng nền treo lửng sau khi `close()` đã trả về** — `src-tauri/src/core/store/checkpoint.rs:228`. Đây là đánh đổi CÓ CHỦ Ý và đã ghi rõ trong doc-comment (hết ngân sách ⇒ ghi chẩn đoán rồi thoát, không join, không treo tiến trình). Rủi ro còn lại: luồng nền có thể vẫn đang chạy TRUNCATE (có thể bị chặn tới `busy_timeout` ~5s) sau khi `shutdown()`/`close()` đã trả về ở phía gọi. ~~Hôm nay vô hại vì chỗ gọi DUY NHẤT là `close_global_store()` ở `RunEvent::Exit` (`lib.rs:128`), ngay sau đó tiến trình thoát. Chỉ trở thành rủi ro thật nếu một story sau này thêm luồng "khởi động lại kho mà không thoát tiến trình". Ghi lại cho story đó.~~
  → 🔴 **ĐỔI TRẠNG THÁI 2026-08-06 (Story 1.15): TỪ "VÔ HẠI" SANG "RỦI RO THẬT".** Story 1.15 là chính story đã được cảnh báo trước: `commands::project::replace_open_work` (`lib.rs`) thay `Option<OpenWork>` trong state mỗi khi một Tác phẩm mới được tạo trong CÙNG một phiên — `Store` cũ bị `Drop` (gọi `close()`, tức `Checkpointer::shutdown()`) **giữa chừng tiến trình**, **không** thoát tiến trình. Đây đúng là "luồng khởi động lại kho mà không thoát tiến trình" mà mục này chờ. Rủi ro cụ thể: tạo hai Tác phẩm liên tiếp nhanh trong cùng phiên có thể để lại một luồng checkpoint của Tác phẩm THỨ NHẤT còn chạy TRUNCATE (tới ~5s `busy_timeout`) trong khi Tác phẩm thứ hai đã bắt đầu ghi — hai luồng checkpoint của hai kho KHÁC NHAU nên không tranh chấp dữ liệu, nhưng CPU/I/O chồng lấn chưa được đo. Chưa có test nào ép được ca này (cần dựng đúng nhịp thời gian giữa hai `create_work` liên tiếp). Giao lại cho story đo hiệu năng multi-Work (chưa có chủ) hoặc Story 2.4 (đo `Tuning`).
- **`Writer::shutdown()` không có trần thời gian cho `handle.join()`** — `src-tauri/src/core/store/writer.rs:159`, dựa trên giả định trong doc-comment rằng job ghi "không chặn/không gọi ra ngoài", giả định này không được kiểu hay runtime cưỡng chế. Một job chặn do bug tương lai (9 epic còn lại ghi qua tầng này) sẽ treo `RunEvent::Exit` vô thời hạn. **Ice chốt 2026-08-04 (lượt code review):** chấp nhận rủi ro — giữ kỷ luật "không bỏ dở một giao dịch đang commit để tiết kiệm mili-giây trên đường thoát" cho v1; giám sát bằng review thủ công mỗi khi một story mới ghi qua tầng này thay vì cưỡng chế bằng cơ chế.
  → ⚠️ **Story 1.15 (2026-08-06) là story ĐẦU TIÊN có job ghi kèm việc tạo dữ liệu I/O SONG SONG (ghi `meta.json`) — đã soát và giữ TÁCH RIÊNG có ý thức.** `commands::project::create_work`'s `store.write(move |tx| {...})` chạy **đúng hai** `tx.execute` (INSERT `work`, INSERT `chapter`), cả hai SQL là **hằng `&'static str`**, tham số ràng buộc qua **tuple** *(⚠️ sửa ở lượt code review 2026-08-06 — mục này trước đó ghi `rusqlite::params`, và đó là tên sai: `store_boundary.rs::FORBIDDEN` cấm token `rusqlite` ngoài `core/store`, nên `commands::project` **buộc phải** dùng tuple. Bất biến được ghi nhận — tham số **ràng buộc**, không `format!` chèn dữ liệu người dùng vào SQL — thì **đúng**; chỉ cơ chế bị gọi sai tên)*, không I/O, không gọi ra ngoài, không `Store::write` lồng nhau. `meta.json` (bao gồm `fs::write`/`sync_all`/`rename`) chạy **SAU KHI** `store.write(...)` đã trả về `Ok`, ở tầng THAO TÁC — Quyết định #3 của story, chính vì lý do này. Giả định *"job ghi không chặn, không I/O"* vẫn đúng sau story này.
- **`ReaderPool::acquire()` chờ `Condvar` không trần** — `src-tauri/src/core/store/reader.rs:107`, không đối xứng với bảo đảm hữu hạn (`StoreError::WriterGone`) của đường ghi; có thể chờ mãi nếu pool cạn kiệt vì một `Lease` rò rỉ hoặc một read job bị chặn. **Ice chốt 2026-08-04 (lượt code review):** chấp nhận như hiện tại — đường đọc không có tác dụng phụ chờ đợi bên ngoài giống job ghi; rủi ro rò rỉ `Lease` thấp hơn rủi ro một job ghi bị chặn.

## Deferred from: 1-8-phan-giai-cau-hinh-hai-tang (2026-08-04)

- 🔴 **`ScopeResolver` chưa cache gì, và đó là một quyết định.** Consumer đường nóng duy nhất là khớp Glossary khi gõ — **Story 3.4**, dưới trần NFR2 *"không frame nào vượt 50 ms"* — và hôm nay nó chưa tồn tại. Dựng cache bây giờ là dựng một cơ chế vô hiệu hoá mà **không có gì để vô hiệu hoá**, và một cơ chế như vậy sẽ sai theo đúng cách mà không test nào bắt. Ba hàm phân giải là **thuần** nên thêm cache về sau là một lượt sửa cục bộ, không phải một lượt mổ. **Chủ sở hữu: Story 3.4** — và nó phải **đo trước** khi cache.

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

- ⚠️ **`src/config/` là một thư mục frontend NGOÀI cây nguồn đã khai.** `ARCHITECTURE-SPINE.md#Structural Seed` chỉ liệt kê `modes/ panels/ layout/ commands/ tokens/ i18n/`. Lý do chấp nhận, ghi ra để lượt review phân xử chứ không tự coi là đã duyệt: nó **không phải một khái niệm miền mới** mà là **adapter IPC phía webview** *(một `invoke`, một `try/catch`, không quy tắc nào)*; đặt nó vào `src/commands/` sẽ kéo `@tauri-apps/api` vào một thư mục mà ba phép kiểm hành vi nạp bằng **Node thuần** (§Bẫy 6), đặt vào `src/modes/` thì sai khái niệm. *(Mặc định của story, Ice ký 2026-08-04.)*

- ⚠️ **Story này ghi qua tầng `store::Writer` — nên nó kích hoạt điều kiện mà Ice đã đặt cho `writer.rs:159`.** Mục *"`Writer::shutdown()` không có trần cho `handle.join()`"* được Ice chấp nhận 2026-08-04 **với điều kiện review tay mỗi khi một story mới ghi qua tầng này**. Đã soát: `save_value` chạy **đúng một** `tx.execute` với SQL hằng, không I/O, không gọi ra ngoài, không `Store::write` lồng nhau *(`writer.rs:104` trả `WriteFailed` chứ không xếp hàng, và story này không đi vào đường đó)*. Giả định *"job ghi không chặn"* vẫn đúng sau story này.

- ⚠️ **Đây là lượt di trú THẬT đầu tiên trên một `global.db` đã có dữ liệu, tức lần đầu đường sao lưu `fs::copy` chạy trên máy người dùng.** `schema.rs:137` đã ghi nhận rằng bản sao đó **không nguyên tử và không xác minh lại**. Không sửa ở story này *(ngoài phạm vi, và mục đã có chủ sở hữu)*, nhưng ghi lại quan sát: từ hôm nay mục đó không còn là lý thuyết — mọi người dùng đã chạy một bản có `user_version = 1` sẽ đi qua nó đúng một lần khi nâng cấp.

- ⚠️ **Đường hiển thị lỗi kho chưa chạy trong webview thật** — xem mục đã cập nhật ở §*Deferred from: code review of 1-7* (*"Lỗi mở kho hôm nay chỉ ra `stderr`"*). Nghiệm thu cần một `$APPDATA` chỉ-đọc; **Story 1.15 vẫn KHÔNG đóng được mục này** — môi trường triển khai của nó không có công cụ GUI automation, xem ghi chú 2026-08-06 ở mục gốc.

- ✅ **`tests/**` miễn trừ khỏi phép quét ranh giới: mục ở §1-7 **KHÔNG** phải mở lại.** Hai tệp test mới của story này **không** gõ tên crate SQLite: `Store::write` nhận một closure lấy `&Transaction` — kiểu **tái xuất** từ `core::store` — nên ca ghi thẳng một hàng vào `global.db` viết được mà không chạm `rusqlite`. Số tệp test chạm crate đó vẫn đúng bằng `store_contract.rs`.

- ⚠️ **Sàn quần thể vẫn đếm TỆP, không đếm nội dung** — `scope_boundary.rs::RS_FLOOR = 20` (thật: 26) và `check-i18n.mjs::RS_FLOOR = 21` (thật: 27). Cùng mục đã ghi ba lần trước cho `check-i18n.mjs:207-218` và `check-commands.mjs`. ⚠️ **Hai quần thể này KHÁC nhau** — `src-tauri/src/**` so với `src-tauri/**` sau miễn trừ `tests/**` *(gồm `build.rs`)* — và chép số của tệp này sang tệp kia là đặt một cái sàn cho một cây khác. Đã ghi vào doc-comment của cả hai.

## Deferred from: code review of 1-8-phan-giai-cau-hinh-hai-tang (2026-08-04)

- **`"ScopeKind"` vẫn còn nửa bẫy sau khi `ScopeResolver::resolve_override`/`resolve_merge` đã đổi tên thành `apply_override`/`apply_merge`** (`src-tauri/tests/scope_boundary.rs:62-67`, `src-tauri/src/core/scope/mod.rs:201,218`) — lượt sửa hôm nay xoá đúng hai token đụng độ (`resolve_override`/`resolve_merge`), nhưng lời gọi hợp lệ tương lai vẫn phải viết `ScopeKind::Glossary` (hay tương đương) để truyền tham số, và `"ScopeKind"` vẫn bị cấm ngoài `core/scope/**`. Cổng AC1 sẽ vẫn đỏ ở token này ngay lần đầu Epic 3/4/6/7 gọi `apply_override`/`apply_merge` từ module của họ — dù đường gọi hoàn toàn đúng. Xoá triệt để đòi đổi chữ ký `apply_override`/`apply_merge` sang nhận `kind: &str` thay vì `kind: ScopeKind` (giống khuôn `save_value` ở ranh giới IPC), để domain module không cần gõ tên kiểu `ScopeKind` trong mã của họ. Ice chọn KHÔNG làm việc đó ở lượt review này (2026-08-04) — giao cho story đầu tiên thật sự trở thành consumer.
- `resolve_one` trong `load_global_config` nuốt lỗi `WrongSemantics` bằng `debug_assert!` rồi `unwrap_or_default()` (`src-tauri/src/core/scope/store.rs:135-147`) — trong build release, một thay đổi ngữ nghĩa tương lai cho `AppConfig`/`Shortcut`/`LayoutPreset` mà quên sửa chỗ gọi này sẽ rơi về map rỗng im lặng thay vì lỗi. Rủi ro thấp vì `cargo test` bắt buộc trước khi merge sẽ đỏ ở debug build, nhưng ghi lại cho lượt sau.
- `watch(currentMode)` gọi `put_config` không có khoá thứ tự (`src/main.ts:178-184`) — đổi chế độ liên tiếp rất nhanh có thể khiến một giá trị trung gian được ghi cuối cùng xuống đĩa do các lời gọi `invoke` hoàn tất không đúng thứ tự gọi. Tự phục hồi ở lượt chuyển chế độ kế tiếp.
- Nhánh lỗi `store.read_failed` của `bootstrap_config` chưa có test ép đường đọc thật trượt (`src-tauri/tests/scope_contract.rs:700`) — `every_command_error_comes_from_the_store_vocabulary` chỉ ép các nhánh `OpenFailed`/`WriteFailed`. Đường lan `?` và phép chuyển `From<StoreError>` đã được kiểm ở tầng `store` (Story 1.7); thiếu một ca tích hợp trực tiếp qua `bootstrap_config`/`load_global_config`.
- `save_value` không giới hạn độ dài `key`/`value` trước khi ghi vào `config_value` (`src-tauri/src/core/scope/store.rs:203`) — mọi lời gọi hôm nay đến từ frontend tin cậy của chính ứng dụng, không phải một biên tin cậy với dữ liệu ngoài.
- `rel_posix(...).starts_with(SCOPE_DIR)` không có dấu `/` đuôi (`src-tauri/tests/scope_boundary.rs:40`, chép nguyên khuôn `store_boundary.rs` của Story 1.7) — một thư mục anh em tên `core/scope_legacy` sẽ khớp nhầm miễn trừ. Xác suất gần như bằng không.
- `code_lines()` trong `scope_boundary.rs` (và bản gốc `store_boundary.rs`) chỉ miễn trừ dòng bắt đầu bằng `//`, không bóc comment khối `/* … */` (`src-tauri/tests/scope_boundary.rs:133`) — một token cấm nằm trong comment khối sẽ báo vi phạm giả. Codebase không dùng comment khối nên rủi ro thực tế thấp.

## Deferred from: 1-9-dung-du-lieu-tu-dien-lop-nen (2026-08-04)

- 🔴 **Lưới thay thế cho `bundle.resources`/`dict/*.db` — chủ sở hữu: Story 10.1.** Task 10 của story này gỡ `$RESOURCE/dict/**` khỏi `assetProtocol.scope` (Ice chốt 2026-08-04, đóng `deferred-work.md:21`+`:57`) vì webview không bao giờ đọc tệp từ điển — `rusqlite` mở tệp bằng đường dẫn hệ thống, không qua asset protocol. Hệ quả: từ hôm nay đến Story 10.1, **không còn dòng nào trong `tauri.conf.json` nhắc tới `dict`**, tức lưới cũ *"ship một bản không có byte từ điển nào thì phải lộ ra ở đâu đó"* mất chỗ bấu — `config_invariants.rs` không còn gì để kiểm về việc `dict-core.db` có được đóng gói hay không. **Story 10.1 phải làm hai việc cùng lúc, không phải một:** (1) thêm `dict/*.db` vào `bundle.resources` của `tauri.conf.json`; (2) thêm một test khẳng định nó **có mặt** trong cấu hình đóng gói — nếu chỉ làm (1) mà không làm (2), lỗ hổng lưới lặp lại y hệt mục đã đóng ở đây.
  → 🔄 **Cập nhật sau Story 1.10 (2026-08-05): phạm vi giờ là BA tệp, không phải một.** `dict-core.db` + `dict-thieu-chuu.db` + `dict-vietphrase.db` — cả ba phải vào `bundle.resources`, và test khẳng định "có mặt" (2) phải khẳng định cả ba, không chỉ base. **Đánh dấu KHÔNG đóng** — vẫn là việc của Story 10.1.
- ⚠️ **`ARCHITECTURE-SPINE.md` (AD-23, dòng ~316) còn liệt kê `$RESOURCE/dict/**` bằng chữ — ĐANG LỆCH khỏi `tauri.conf.json` kể từ Task 10 của story này.** Dev không sửa tài liệu quy hoạch (tiền lệ quyết định #3 của Ice ở Story 1.3); sửa AD-23 là việc riêng của Ice. Ai đọc AD-23 trước khi đọc cấu hình thật sẽ hiểu sai rằng `dict` vẫn còn trong scope.

## Deferred from: 1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap (2026-08-05)

- 🔴 **HVTĐTD + Cổ hán văn — hai lớp gỡ rời còn lại, chưa có nguồn thô. Chủ sở hữu: story nối tiếp Story 1.10.** Ice chốt 2026-08-05 thu hẹp phạm vi Story 1.10 từ bốn lớp xuống hai (Thiều Chửu + VietPhrase) vì hai lớp này chưa có nguồn thô và không thể tự tìm thay thế:
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

## Deferred from: correct-course — đường tiếng Anh (2026-08-05)

- 🔴 **Dư địa NFR6 nay là 15.474.554 byte ĐO THẬT — HVTĐTD và Cổ hán văn phải được ĐO trước khi hứa đóng gói. Chủ sở hữu: story nối tiếp của Story 1.10.**
  🔄 **CẬP NHẬT 2026-08-05 sau Story 1.10b — số DỰ PHÓNG thay bằng số ĐO ĐƯỢC.** Lớp tiếng Anh đã dựng thật; `dict-core.db` đi từ `154.464.256` → **`194.998.272` byte** *(**+40.534.016**, dự phóng là +40.333.312 ⇒ hụt **200.704** byte)*. Kế toán thật: baseline `.dmg` **2.334.696** + license **35.149** + font **21.285.713** + `dict-core.db` **194.998.272** + Thiều Chửu **5.787.648** + VietPhrase **160.083.968** = **384.525.446 byte** *(384,53 MB thập phân)*. **ĐẠT** trần **400.000.000**, **dư địa thật 15.474.554 byte** *(dự phóng 15.675.258 ⇒ **hẹp hơn 200.704**; dự phóng chính xác 99,95%)*.
  Thang so sánh **không đổi**: Thiều Chửu **5.787.648** *(vừa)* · VietPhrase **160.083.968** *(không vừa, gấp **10,3 lần** dư địa)*. Nếu HVTĐTD giàu ví dụ + trích dẫn như `prd.md` §8.3 mô tả, **trần 400 MB sẽ vượt lần thứ hai** — và lúc đó đường ra không còn là nâng trần tiếp: phải cân nhắc lại chính lời hứa *"không tải thêm sau khi cài"*, tức chạm **NFR7** và **NFR12**. **Không dựng HVTĐTD rồi mới đo** — đo trước, báo số cho Ice, rồi mới quyết đóng gói.
- ~~🔴 **AD mới cho đường tra cứu tiếng Anh — chủ sở hữu: Winston (`bmad-architecture`). 🔴 VẪN CHẶN Story 1.11b.**~~
  → ✅ **ĐÃ ĐÓNG 2026-08-05 — `AD-44` đã vào `ARCHITECTURE-SPINE.md`. Story 1.11b KHÔNG còn bị chặn bởi mục này.** Sáu mệnh đề của AD-44: ① vị từ điều phối là **hình dạng chuỗi truy vấn** *(có ký tự Hán ⇒ đường zh; ngược lại ⇒ đường en)*, không phải ngôn ngữ của Tác phẩm — nên ca *"bôi đen `API` trong truyện Trung"* mà chính mục này nêu **ra kết quả đúng** thay vì rỗng; vị từ chạy **một lần mỗi lượt tra, TRÊN tầng gom**, và không tồn tại sổ đăng ký *"tệp nào chứa ngôn ngữ nào"*. ② đường en có **hai** nhánh *(exact B-tree · FTS5 trigram ≥ 3)*, không có `char_idx`. ③ tập khoá tra chính xác = `{nguyên văn, hạ chữ thường}` trong **một** truy vấn `IN (?1,?2)`, không fallback dây chuyền — và 🔴 **stemming KHÔNG nằm trên đường nóng tra từ điển**, xem mục kế tiếp. ④ chuỗi con **1–2 ký tự** tiếng Anh khai là **không hỗ trợ**, trả trạng thái phân biệt được với *"không có kết quả"*. ⑤ ranh giới mã-riêng-theo-ngôn-ngữ: được phép **đúng** ở chiến lược truy vấn trong `core/dict/`; không cấm ở cổng `DictionarySource` *(adapter theo **tệp**, không theo **ngôn ngữ**)*, ở hình dạng bản ghi kết quả *(`lang` là **trường**, không phải **kiểu**)*, và không cấm mọi bước hợp nhất zh với en. ⑥ **NFR1 đo TRÊN đường tiếng Anh**, không suy từ số tiếng Trung. Kèm theo: **AD-26 sửa Rule tại chỗ** — phạm vi *"tiếng Trung"* đưa vào **thân** Rule chứ không chỉ ở tiêu đề, và dải hiệu năng công bố *(0,15–4,5 ms)* đánh dấu **LỖI THỜI**, thay bằng số đo 2026-08-05 mà Story 1.11 đã bàn giao ở §Completion Notes ②. Reviewer Gate: `lint_spine.py` 0 findings, ba lens bắt 5 phát hiện *(2 nghiêm trọng)* — tất cả đã vá; báo cáo ở `architecture/architecture-AuraTranslate-2026-08-02/reviews/review-ad-44-2026-08-05.md`.

- 🔵 **PHÁT HIỆN MỚI của lượt AD-44 (2026-08-05) — đo thật, không suy luận: `FR40` trên đường TỪ ĐIỂN không cần stemming, và chữ HOA mới là lỗ thật.**
  - **Stemming mua được ~0 recall.** Thứ phủ FR40 là một **tính chất của corpus**: Wiktionary đã có sẵn mọi dạng biến thể làm **đầu mục riêng** — mẫu **16/16** có mặt, **gồm cả bất quy tắc** `went` · `gone` · `children` · `happiest`, thứ stemming về nguyên tắc **không bao giờ** làm được. Quy mô: **7.656** đầu mục `-ing` · **8.855** `-ed` · **19.616** `-s` · **228** `-est` trên **119.039**. *(Dữ kiện phụ, yếu hơn: ba dạng stem Porter kinh điển tra vào `dict-core.db` cho **0** hàng — `dictionari` · `studi` · `happi`; `run` cho 1. ⚠️ **Số hàng là đo thật, nhưng ba CHUỖI stem đó chưa chạy qua stemmer mà sản phẩm sẽ dùng** — ghi rõ trong AD-44 ③.)*
  - 🔴 **Lỗ chữ HOA, chưa tài liệu nào ghi:** `headword='running'` ⇒ **1** hàng nhưng `headword='Running'` ⇒ **0**. Bôi đen một từ ở **đầu câu** là thao tác thường ngày và nó trả rỗng không báo gì — đúng lớp lỗi FR39/AD-26 tồn tại để chặn. **1.635** đầu mục en mang chữ hoa có nghĩa (`API` · `Wikipedia` · `English`) và **184** nhóm chỉ khác nhau ở chữ hoa ⇒ hạ chữ thường phải là **THÊM** khoá, không phải **THAY**.
  - ⇒ **Hệ quả cho Story 1.11b:** `Matcher` của 1.12 **không còn là điều kiện chặn** — 1.11b không gọi nó. **Story 1.12 vẫn dựng Matcher đầy đủ** cho Glossary (FR51) và TM (FR61), nơi thuật ngữ do **người dùng tự viết** nên corpus không mang tính chất trên và stemming thật sự đáng tiền *(ranh giới này đã ghi vào Rule của AD-17)*.
  - 🟡 **`epics.md` ĐANG LỆCH khỏi AD-44 ở hai chỗ — chủ sở hữu: John (PM), Winston không sửa `epics.md`.** *(a)* Mục Story 1.11b `:1478` ghi *"biến thể hình thái **dùng `Matcher` của AD-17**, không cài riêng một bản thứ hai"* — AD-44 ③ nay nói đường **từ điển** không gọi Matcher, và ghi số đo làm lý do. Câu đó nên đổi thành *"tập khoá `{nguyên văn, hạ chữ thường}`"* cộng một AC cho lỗ chữ HOA *(`Running` ⇒ 0 hàng)*. *(b)* Cùng mục còn ghi 🔴 *"CHẶN: cần một AD mới… chủ sở hữu Winston"* — **nay đã giao**, dòng chặn nên gỡ. **Hệ quả thứ tự:** lý do đảo `1.12` lên trước `1.11b` *(Ice chốt 2026-08-05, vì 1.11b cần Matcher)* nay **không còn**; thứ tự trong `sprint-status.yaml` và `epics.md` chưa ai đổi, nên `bmad-create-story` sẽ tự chọn `1-11b` — và **đó nay là lựa chọn đúng**.
  - 🟡 **Việc tầng PRD, không tự sửa:** `prd.md` FR40 phát biểu yêu cầu **bằng cơ chế** *(*"nhận diện biến thể hình thái"* + ghi chú stemming/lemmatization)*. Trên đường **từ điển**, cơ chế thật là *"corpus có sẵn mọi dạng biến thể"*, và nó phủ **rộng hơn** stemming. Chủ sở hữu: **John (PM)** — cân nhắc tách FR40 thành *(a)* tra cứu từ điển và *(b)* khớp Glossary/TM, vì hai vế nay có hai cơ chế và hai giới hạn khác nhau. `AD-26` tên đầy đủ là *"Ba nhánh truy vấn **tiếng Trung**"* và cả ba nhánh đều là cơ chế cho chữ Hán: tra chính xác đầu mục *(dùng được cho EN)* · chuỗi con 1–2 ký tự qua `char_idx` *(**vô dụng** với tiếng Anh)* · chuỗi con 3+ ký tự qua FTS5 `trigram` *(chạy được nhưng không phải hình dạng đúng)*. Tiếng Anh cần **exact + stemming** *(FR40, `Matcher` của AD-17)*, không phải truy vấn chuỗi con ký tự. Cần một AD nêu rõ chiến lược truy vấn theo ngôn ngữ, và nêu rõ **ranh giới**: chỗ nào là "mã riêng cho từng ngôn ngữ" **được phép** *(chiến lược truy vấn)* và chỗ nào **cấm** *(cổng `DictionarySource`, AD-10)*.
  🔄 **CẬP NHẬT 2026-08-05 sau Story 1.10b — con số nay là ĐO ĐƯỢC trên `dict-core.db` thật, không còn là mũi thăm dò:** nguồn `viwiktionary-en` đóng góp đúng **9** cặp `char_idx` trên **119.039** đầu mục = **0,0076%**. `char_idx` tổng của tệp là **1.341.179**, tức lớp tiếng Anh chiếm **0,00067%** của chỉ mục đó. ⇒ **`AD-26` nhánh 2 KHÔNG áp được cho tiếng Anh** — đây là dữ kiện, không phải suy đoán. *(SQL tái lập: `SELECT COUNT(*) FROM char_idx c JOIN dict_entry e ON e.id=c.entry_id JOIN dict_source s ON s.id=e.source_id WHERE s.code='viwiktionary-en';`)*
- 🔴 **Đường tra cứu PHẢI lọc theo `dict_entry.lang` — KHÔNG được giả định mọi hàng là `zh`. Chủ sở hữu: Story 1.11b và 1.13.** *(MỚI — Story 1.10b, 2026-08-05.)*
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
  ⇒ Story 1.11 lọc `lang='zh'` ở **cả ba** nhánh, và `tests/dict_lookup.rs::every_branch_filters_out_english_entries` cưỡng chế bằng đúng hai truy vấn Latin đó. **Trạng thái: ĐÃ ĐÓNG cho 1.11; vẫn mở cho 1.11b và 1.13.**

- 🟡 **VietPhrase: 18 đầu mục trùng — VẪN MỞ, chủ sở hữu là 1.13, không phải 1.11.** Story 1.11 chạy trên **MỘT** tệp `.db` mỗi lượt và AD-19 cấm hợp nhất nguồn, nên nó không gộp trùng và không được phép gộp. Hậu quả *"UI hiện hai khối VietPhrase giống hệt nhau"* chỉ **quan sát được** khi gom nhiều nguồn — tức nó là quyết định của **1.13**, và quyết định đó phải chọn giữa *"gộp lúc đọc"* và *"quyết lại mô hình lúc dựng"*.

- 🔵 **Khoá theo `code` chứ không theo `id` — ĐÃ ĐÓNG MỘT NỬA.** `EntryHit` của 1.11 mang `source_code: String`, không có trường `source_id` nào, và `tests/dict_lookup.rs::results_carry_the_source_code_not_the_id` khoá mệnh đề đó trên một fixture hai nguồn. **Nửa còn lại là của 1.13:** lúc gom nhiều tệp, khoá gom **phải** là `code`, và không được phép dựng lại một bảng tra `id → nguồn` ở tầng gom.

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

- **NFR1 nhánh 2 (1 ký tự) còn 27% dư địa tới trần — quyết định lúc review: chấp nhận nguyên trạng.** Xác nhận lại mục đã có ở trên (§Deferred from: 1-11-…, mục NFR1): người dùng, khi review, chọn không sửa gì bây giờ và để 1.13/1.17 xử lý bằng phân trang thật khi Panel Lookup tồn tại.
- **Kế hoạch truy vấn của nhánh 1/2 chỉ được xác nhận bằng `EXPLAIN QUERY PLAN` chạy tay, không có cổng CI tự động** (`src-tauri/src/core/dict/query.rs`) — nếu một di trú lược đồ tương lai xoá `idx_entry_headword`/`idx_entry_headword_simp` hay đổi khoá chính của `char_idx`, nhánh 1/2 có thể âm thầm suy biến thành quét toàn bảng mà không test hành vi nào đỏ. Đây là cùng ràng buộc mà AC9 đã chấp nhận (không có tệp `.db` thật trong CI), nên không tự động hoá được trong story này.
- **Nhánh `char_idx` 1 ký tự bỏ qua xác minh chuỗi con ở Rust, dựa hoàn toàn vào bất biến của `tools/dict-build`** (`src-tauri/src/core/dict/query.rs:121`) — đúng của tối ưu này phụ thuộc việc `char_idx` không bao giờ sinh một cặp `(ký tự, entry_id)` sai, một bất biến chỉ được cưỡng chế ở workspace `tools/dict-build`, không có cổng nào kiểm chéo hai workspace. Ranh giới hai workspace tách rời đã chốt từ Story 1.9 (AC4); story này kế thừa quyết định đó chứ không tạo ra nó.
- **Không có giới hạn trên cho độ dài truy vấn** trước khi đưa vào `chars()`, cấp phát chuỗi lặp lại, và dựng cụm FTS (`src-tauri/src/core/dict/query.rs`) — thật với một đầu vào cực dài, nhưng story 1.11 tường minh cấm dựng IPC command hay chạm frontend nên chưa có bên gọi không tin cậy nào tồn tại. Validate độ dài đầu vào thuộc về tầng IPC/UI của Story 1.13/1.17.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.17, Task 2).** `commands::dict::lookup()` cắt truy vấn ở `QUERY_LENGTH_CEILING = 200` ký tự TRƯỚC khi vào đường tra — một sàn TRÊN có tên, không một `panic`. Ca test `a_query_past_the_length_ceiling_is_truncated_before_it_reaches_the_lookup` chứng minh việc cắt xảy ra trước `pick_route` (201 ký tự Latin+Hán ⇒ route `En`, không `Zh`).

## Deferred from: code review of 1-11b-duong-tra-cuu-tieng-anh (2026-08-05)

- **Điều kiện `≤ 2 ký tự` của `char_idx()` chỉ cưỡng chế bằng `debug_assert!`** (`src-tauri/src/core/dict/query.rs:138`) — vô tác dụng ở bản release; một lượt gọi trực tiếp trong tương lai (vd. từ tầng gom Story 1.13, nếu nó bỏ qua `lookup()` và gọi thẳng `query::char_idx`) với truy vấn dài hơn sẽ âm thầm cắt còn hai ký tự đầu thay vì báo lỗi. Kế thừa từ Story 1.11 — story 1.11b không chạm hàm này, chỉ mở rộng phạm vi tiếp xúc của nó qua module dùng chung.
- **Ngưỡng độ dài nhánh (`chars().count()`) đếm code point Unicode, không đếm cụm ký tự hiển thị (grapheme cluster)** (`src-tauri/src/core/dict/mod.rs:242`) — văn bản chuẩn hoá NFD (vd. clipboard macOS với dấu tổ hợp) có thể đẩy một truy vấn qua lằn ranh nhánh (`CharIdx`/`FtsTrigram` ở đường zh, `NoBranchQueryTooShort`/`FtsTrigram` ở đường en) sai lệch so với số ký tự người dùng cảm nhận đã gõ. Phép đo kế thừa nguyên xi từ ngưỡng zh của Story 1.11; story 1.11b áp dụng cùng cách đo cho ngưỡng en mới chứ không tạo ra vấn đề. Sửa đòi một quyết định chuẩn hoá Unicode chung cho cả hai đường — thuộc tầng kiến trúc, không phải một lượt vá cục bộ.

## Deferred from: 1-12-matcher-dung-chung (2026-08-05)

- 🔵 **Sơ đồ mermaid của AD-13 còn cạnh `dict --> matching` — LỆCH khỏi thân Rule của AD-17. Chủ sở hữu: Winston (architect).** `ARCHITECTURE-SPINE.md:189` vẽ một cạnh phụ thuộc từ `dict` sang `matching`. Sơ đồ đó vẽ **trước** lượt sửa Rule của AD-17 ngày 2026-08-05, và nay mâu thuẫn với chính **thân Rule** ở `:236`: *"AD này nói mọi nơi cần khớp ngôn ngữ dùng chung MỘT cài đặt — nó KHÔNG nói mọi đường đều phải gọi Matcher. Đường tra cứu **từ điển** tiếng Anh không gọi."* **Không chặn Story 1.12** *(dev theo thân Rule, và Story 1.11b đã giao xong đường tra cứu tiếng Anh mà không gọi Matcher một lần nào)*, nhưng nó sẽ làm lệch **mọi lượt đọc kiến trúc sau** — sơ đồ là thứ người ta đọc trước, thân Rule là thứ người ta đọc sau. Mệnh đề đúng nay được cưỡng chế bằng cổng `tests/matching_boundary.rs::the_dictionary_lookup_path_never_calls_the_matcher`, kèm thông báo assert nêu đích danh AD-17 `:236` và AD-44 ③.

- 🟡 **`epics.md:1510` còn vế *"`dict/` dùng nó"* — chủ sở hữu: John (PM).** Mục Story 1.12 vẫn viết *"**And** `dict/` **dùng nó**; `glossary/` và `tm/` sẽ dùng chính nó ở các epic sau"*. Vế đầu đã bị AD-17 lật. Cùng lượt sửa với `:1491` *(mục 1.11b)* mà mục *"`epics.md` ĐANG LỆCH khỏi AD-44 ở hai chỗ"* ở trên đã ghi. Story 1.12 giao `core/matching/` cho Glossary và TM và **cưỡng chế bằng cổng** rằng `core/dict/**` không gọi nó — nên nếu vế cũ được ai đó thi hành, cổng sẽ đỏ **có tên** thay vì hồi quy im lặng.

- 🔵 **AD-44 ③: bảng phỏng đoán Porter NAY CÓ SỐ ĐO THẬT — chủ sở hữu: Winston (architect), dev không sửa `ARCHITECTURE-SPINE.md`.** `:616` ghi ⚠️ *"ba chuỗi stem đó lấy từ hành vi kinh điển của Porter chứ **chưa chạy qua stemmer mà sản phẩm sẽ dùng** […] ai muốn mở lại câu hỏi stemming thì việc đầu tiên là **chạy stemmer thật và thay bảng này bằng số đo**"*. Story 1.12 là lượt đầu tiên có stemmer thật trong cây mã, và đã chạy `tantivy_stemmers::algorithms::english_porter_2` trên đúng bốn chuỗi AD-44 nêu:

  | Đầu vào | AD-44 ③ phỏng đoán | **Đo thật** | Trùng? |
  |---|---|---|---|
  | `dictionary` | `dictionari` | `dictionari` | ✅ |
  | `study` | `studi` | `studi` | ✅ |
  | `happy` | `happi` | `happi` | ✅ |
  | `run` | *(1 hàng, không nêu chuỗi)* | `run` | ✅ |

  ⇒ **Phỏng đoán của AD-44 ③ ĐÚNG 3/3 chuỗi nó nêu.** Kết luận của ③ *(stemming không nằm trên đường nóng tra từ điển)* **không** bị lật bởi số đo — nó **được củng cố**: ba chuỗi stem đó đúng là thứ sẽ được tra, và chúng đúng là cho **0** hàng trên `dict-core.db`. Món nợ đo đạc ở `:616` nay **đóng được**; ⚠️ nội dung sửa là của Winston.

- 🔴 **`Jieba` khởi tạo tốn ~180–330 ms bản RELEASE — VƯỢT NFR2 (50 ms) từ 3,6× đến 6,6×. Chủ sở hữu: Story 3.4** *(story đầu tiên gọi Matcher trên một đường gõ thật)*. Đo thật, `[profile.release]` không đổi một dòng, 6 lượt chạy trên máy dev *(macOS, darwin 24.6.0)*, mỗi lượt một tiến trình mới:

  | Lượt | 1 | 2 | 3 | 4 | 5 | 6 |
  |---|---:|---:|---:|---:|---:|---:|
  | Khởi tạo lạnh **(ms)** | 328,588 | 244,444 | 224,407 | 179,161 | 242,224 | 255,437 |

  Trung vị **~243 ms**, thấp nhất **179 ms**, cao nhất **329 ms**. Lượt gọi **ấm** kế tiếp: **1 µs** *(dưới ngưỡng đo)*. Chi phí là giải nén `dict.txt` *(**5.071.843 byte** thô, nhúng qua `include_flate::flate!`)* cộng nạp từng dòng vào một cây `cedar` — công việc **chạy lúc chạy**, không phải một hằng số biên dịch, và nó rơi vào **lần gọi đầu tiên**, tức có thể rơi đúng vào phím đầu tiên người dùng gõ.
  **Đường ra là hâm nóng `LazyLock` NGOÀI đường gõ** *(một lượt `tokenize` giả lúc mở Tác phẩm, hoặc trên một luồng nền lúc khởi động)*. **Story 1.12 cố ý KHÔNG dựng cơ chế hâm nóng** — chưa có đường gõ nào tồn tại để hâm nóng vào, và một cơ chế dựng trước người tiêu thụ là một phỏng đoán về chỗ gọi. Cổng `tests/matching_boundary.rs::the_jieba_dictionary_is_constructed_at_exactly_one_place` + `…::the_single_jieba_instance_is_actually_lazily_initialised_once` giữ cho chi phí này không nhân lên khi ai đó chuyển lời gọi vào thân một hàm bị gọi lặp.

- 🔵 **PHÁT HIỆN MỚI: Porter2 KHÔNG có luật cho hậu tố so sánh/cực cấp (`-er` · `-est`) — `happiest` không về được `happy`.** AC7 của story liệt kê `happiest` là một *"biến thể hình thái"* mà Matcher phải nhận diện được về dạng gốc. **Đo thật lật vế đó:** `happiest` ⇒ `happiest`, trong khi `happy` ⇒ `happi` — hai vế **không** gặp nhau. Ba biến thể còn lại của AC7 thì đạt *(`running`⇒`run` · `dogs`⇒`dog` · `studies`⇒`studi`=`study`)*. Đây **không** phải lỗi cài đặt: Porter2 theo định nghĩa không xử lý `-er`/`-est`, nên một biến thể **có quy tắc** cũng rơi vào đúng giới hạn mà FR40 đã tuyên bố cho dạng **bất quy tắc**. Story đóng nó bằng cách đưa `happiest` vào ca test giới hạn có tên *(`stemming_is_not_lemmatization_irregular_and_comparative_forms_never_reach_their_lemma`)* thay vì vào ca AC7. ⚠️ **Hệ quả cần biết cho Epic 3:** một người dịch thêm thuật ngữ `happy` vào Glossary sẽ **không** thấy `happiest` được tô màu. Nếu đó là mức phủ không chấp nhận được thì đường ra là một lemmatizer — và NFR15 đòi rà giấy phép **trước** khi thêm phụ thuộc. Chủ sở hữu quyết định: **Ice / John (PM)**, ứng viên là Story 3.4.

- 🟡 **`find_terms` là O(số thuật ngữ × độ dài văn bản) — không chỉ mục ngược, không cache. Chủ sở hữu: Story 3.4 / 7.5.** Story 1.12 cố ý không dựng cả hai *(§Ranh giới phạm vi: chúng thuộc 7.5/7.6 và phụ thuộc dữ liệu thật)*. Với một Glossary vài trăm thuật ngữ trên một segment vài trăm ký tự thì hình dạng hiện tại thừa đủ; với một Glossary vài nghìn thuật ngữ trên **cả chương**, nó cần đo lại trước khi đặt lên đường gõ *(NFR2 = 50 ms mỗi frame)*. **Chưa đo** — chưa có người tiêu thụ nào để đo trên đó, và một con số đo trên đầu vào tự bịa là một con số không dùng được.

## Deferred from: code review of 1-12-matcher-dung-chung (2026-08-05)

- **Các cổng ranh giới trong `matching_boundary.rs` (và `dict_boundary.rs`/`store_boundary.rs` trước đó) là phép quét CHỮ trên mã nguồn, không phải phân tích đồ thị gọi hàm ngữ nghĩa** — một lớp bọc re-export dưới tên khác (vd. `pub use matching::find_terms as glossary_probe;` đặt trong `core/mod.rs`) có thể để `core/dict/**` gọi vào Matcher mà không chạm bất kỳ token cấm nào (`matching`/`jieba`/`stemmer`/`stem(`). Đây là giới hạn có sẵn của cả khuôn "cổng quét chữ" dùng xuyên dự án từ Story 1.9, không phải do Story 1.12 gây ra hay có thể sửa cục bộ trong một story — sửa đòi thiết kế lại triết lý cổng ranh giới trên toàn dự án.
- **Không có test nào cưỡng chế lời hứa "không chạm filesystem/database/mạng" (AD-15) mà doc-comment của `core/matching/mod.rs` tuyên bố** — đúng hôm nay qua rà tay thủ công (không có lời gọi I/O nào trong mã story 1.12), nhưng không gì bắt được nếu một lượt sửa tương lai âm thầm thêm I/O vào module lá này. Khuôn cổng ranh giới hiện tại (`matching_boundary.rs`, `dict_boundary.rs`, `store_boundary.rs`) chưa có tiền lệ kiểm loại forbidden-token này cho `fs`/`net`/`rusqlite`.
- **Một số con số "đo được" gắn cứng trong doc-comment của `core/matching/mod.rs` và trong mục review trước ở tệp này (`dict.txt` = 5.071.843 byte thô; khởi tạo `Jieba` 179–329 ms bản release) không được một test nào khẳng định** — sẽ lặng lẽ lạc hậu khi phiên bản `jieba-rs` hoặc dữ liệu dict đổi, vì không cổng nào đỏ khi điều đó xảy ra. Rủi ro tài liệu, không phải rủi ro đúng/sai của mã.
- **`ngrams` và `find_terms` mỗi hàm tự tokenize/normalize lại toàn bộ văn bản đầu vào — không có bề mặt API nào để tái dùng token đã tính giữa hai lời gọi trên cùng một đoạn văn bản.** Một người tiêu thụ tương lai (Story 7.6) cần cả n-gram lẫn tìm thuật ngữ trên cùng một segment sẽ trả giá tokenize/normalize hai lần. Chủ sở hữu quyết định hình dạng API: Story 3.4/7.6, khi có người tiêu thụ thật.
- **Văn bản chuẩn hoá NFD (dấu tổ hợp, vd. một số nguồn clipboard macOS) tokenize/stem khác với văn bản NFC cùng nội dung ở đường `En` của `core/matching/`** (`char::is_alphanumeric` trong `tokenize` và `to_lowercase` trong `normalize` đều không chuẩn hoá NFC/NFD trước khi xử lý) — cùng lớp vấn đề chuẩn hoá Unicode đã ghi nhận cho một module khác ở mục *§Deferred from: code review of 1-11b-duong-tra-cuu-tieng-anh* phía trên (`core/dict/mod.rs:242`, kế thừa từ Story 1.11). Sửa đòi một quyết định chuẩn hoá Unicode chung cho toàn dự án — thuộc tầng kiến trúc, không phải một lượt vá cục bộ ở module này.

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

- **`SchemaTooNew` chỉ từ chối tệp mới hơn `SUPPORTED_SCHEMA_VERSION`, chưa bắt tệp cũ hơn** — `src-tauri/src/core/dict/layer.rs:229` chỉ kiểm `file_version > SUPPORTED_SCHEMA_VERSION`. Một tệp mang `PRAGMA user_version` và `dict_meta('schema_version')` **nhất quán với nhau nhưng thấp hơn** phiên bản ứng dụng hiện tại sẽ lọt qua cổng phiên bản. Deferred, pre-existing shape of AC4 — spec Story 1.13 chỉ đòi hỏi bắt ca "quá mới" (`epics.md`), và chưa có tệp schema version 0 nào từng tồn tại nên chưa có ca thật để nghiệm thu. Chủ sở hữu: story tiếp theo nào bump `SUPPORTED_SCHEMA_VERSION`/`tools/dict-build/src/schema.rs::SCHEMA_VERSION` lên 2.

## Deferred from: code review of 1-13-duong-tra-cuu-giu-nguyen-bat-dong-giua-cac-nguon — lượt 2, tests (2026-08-06)

- **`ordering_lacks_a_tiebreaker` (`tests/dict_boundary.rs:807`, AC7 gate) quét theo từng dòng vật lý** — một câu `ORDER BY` bị tách dòng lọt qua cổng hoàn toàn im lặng (không đỏ, không xanh). Rủi ro thật thấp hôm nay: mọi SQL trong `core/dict/**` viết một dòng theo đúng quy ước đã có. Một bản sửa an toàn cần phân tích ranh giới chuỗi ký tự Rust thật, không phải ghép cặp dòng kề nhau (rủi ro làm mất phát hiện ca một-dòng nếu dòng kế cận tình cờ chứa token `id`).
- **`mentions_a_dict_db_file` (`tests/dict_boundary.rs:540`, AC2 gate) đòi literal `"dict-"` có gạch nối** — một tên tệp viết cứng không mang tiền tố đó (vd. `"hvtdtd.db"`) lọt qua. Mọi tên tệp thật trong `dict-manifest.toml` hôm nay theo đúng quy ước `dict-*.db`, nên chưa có rủi ro thật; xem lại nếu quy ước đặt tên đổi.
- **Sáu trong mười biến thể `SkipReason` chưa có ca hành vi gọi tên trực tiếp trong `dict_sources.rs`** — `OpenFailed`, hai khoá của `MetaRowMissing`, `SourcesUnreadable`, `DuplicateLayer`, `LookupFailed`. Chỉ `MetaUnreadable`/`SchemaTooNew`/`SchemaVersionDisagrees`/`DuplicateSourceCode` được test trực tiếp. Mỗi ca cần dựng fixture hỏng riêng — khối lượng thuộc một lượt hardening test riêng.
- **Nhóm khoảng trống độ phủ nhỏ hơn, gộp lại:** (a) `.DB` viết hoa chưa có ca chứng minh được nạp như `.db` (NFR14); (b) lỗi `read_dir` khác `NotFound` (vd. từ chối quyền) chưa có ca chứng minh vẫn trả tập lớp rỗng không panic — đúng đường vừa sửa ở lượt 1; (c) chưa có ca xoá tệp của một lớp bị `conflict_with` từ chối để chứng minh nó không còn bị khoá; (d) chưa có ca gọi `senses()` (chỉ `lookup()`) trên hai lớp khác nhau cùng `entry_id` để chứng minh không trộn dữ liệu; (e) chưa có ca ép `layer.lookup()` hỏng giữa chừng để chứng minh `SkipReason::LookupFailed`; (f) chưa có ca dựng một **thư mục** tên `*.db` để chứng minh bị từ chối an toàn; (g) nhánh `FtsTrigram`/`CharIdx` 2-ký-tự chỉ xuất hiện trong bench `#[ignore]`, chưa có ca khẳng định ở tầng gom.

---

## Deferred from: 1-14-khung-bon-panel (2026-08-06)

*Mọi mục dưới đây là thứ Story 1.14 **cố ý KHÔNG làm**, hoặc chưa đo được. Mười hai mục mà `deferred-work.md` giao đích danh story này đã được đánh dấu đóng **tại chỗ** ở các mục phía trên, mỗi cái kèm tên phép kiểm và số đo.*

- 🔴 **BỐN NGƯỠNG MÀN HÌNH HẸP của UX-DR15 KHÔNG đóng ở đây — chủ sở hữu là Story 4.12.** `epics.md:1617` cấm tường minh: *"ngưỡng kích thước cụ thể đóng ở Story 4.12, không đóng ở đây"* và *"không được cài cơ chế ẩn theo cách khiến Story 4.12 phải mổ lại bố cục"*. Story này giao **CƠ CHẾ**: `SACRIFICE_ORDER` · `NEVER_SACRIFICED` · `nextToSacrifice()` · `nextToRestore()` ở `src/layout/workspaceLayout.ts` — **hàm thuần**, không đọc `window.innerWidth`, không một `matchMedia` nào trong toàn `src/**`. `scripts/check-layout.mjs` Kiểm A cưỡng chế ba mệnh đề của AC7 trên **cả 16 tập con** của bốn panel. ⇒ 4.12 **chỉ phải nối ngưỡng vào**.
  ⚠️ Vế *"Tra cứu rút về THANH TRẠNG THÁI, không bao giờ mất hẳn"* **chưa cài** — `panel.lookup` hôm nay chỉ **nhường**. Đừng đọc `SACRIFICE_ORDER` thành *"Tra cứu được phép biến mất"*. Cũng thuộc **Story 4.12**; ngăn kéo cũng vậy.
  ⚠️ **Sự thật đã có mà 4.12 sẽ đụng:** `tauri.conf.json:19-20` khai `minWidth: 960` · `minHeight: 600`, nên ngưỡng *"< 860 rộng ⇒ báo không hỗ trợ"* của UX-DR15 **không đến được bằng cách kéo cửa sổ** trên cấu hình hôm nay. Story này không sửa `tauri.conf.json` *(`deferred-work.md` [D4], Ice chốt lần thứ tư)* — ghi ra để 4.12 quyết **một lần**.

- ⚠️ **LỖ NFR17 MỞ RA CÓ Ý THỨC: bốn `layout.toggle_*` không có phím.** Ẩn/hiện panel hôm nay **chỉ tới được bằng chuột** *(qua menu ngữ cảnh của dockview)*. Đổi lại: `unbound()` giữ được **bốn** phần tử thật, nên **AC6 của Story 1.6** *(*"liệt kê được thao tác chưa gán phím"*)* không mất bằng chứng — gán phím cho cả bốn sẽ làm `unbound()` trả mảng rỗng và **không cổng nào đỏ** *(§Bẫy 5 của story)*. Một lỗ **có tên và có chủ** tốt hơn một bằng chứng bị xoá. Chủ: **Story 1.21** *(màn hình gán phím)*. ⚠️ Handler thì **chạy thật** — `registry.ts` ném với một `run` thiếu, nên không có command rỗng nào ở đây.
  → ⚠️ **ĐÓNG MỘT NỬA 2026-08-11 (Story 1.21), và nửa còn lại KHÔNG được đóng — mệnh đề ghi ra bằng chữ thay vì gạch mục.**
  **Đã đóng, theo nghĩa của FR22:** từ hôm nay người dùng **gán được** phím cho cả bốn `layout.toggle_*` ở màn hình phím tắt, và lựa chọn đó sống qua các phiên. Ẩn/hiện panel không còn *"chỉ tới được bằng chuột"* đối với người dùng chịu gán một phím.
  **KHÔNG đóng, và không được đóng:** *"bộ MẶC ĐỊNH của sản phẩm có phím cho bốn thao tác này"*. Gán hợp âm mặc định cho chúng làm `unbound()` trả mảng rỗng và `check-commands.mjs` **đỏ** — AC7 của Story 1.21 nói đích danh điều đó. Số thật sau story: `unbound()` giữ **16** phần tử *(bốn `layout.toggle_*` · hai `library.import_*` · ba của 1.19 · ba của 1.20 · bốn của 1.21)*. Một lỗ có tên vẫn tốt hơn một bằng chứng bị xoá — nay nó còn có một **đường ra** cho người dùng. **Chủ của nửa còn lại: chưa gán**, và nó chỉ mở lại nếu một story sau tìm được một hợp âm mặc định có nghĩa mà không giết bằng chứng của AC6/1.6.

- 🔴 **Vế THỊ GIÁC của story CHƯA đo trên WKWebView, và ca Windows chưa đo.** Bảng 35 ca của §Debug Log References chạy trên **Blink/Chromium (Playwright headless), macOS 24.6 arm64**. Lượt `npm run tauri dev` **có chạy** và nghiệm thu **AC4** *(vòng lưu → đóng → mở lại → khôi phục, trong WKWebView thật với IPC thật)* — nhưng nó **không** nghiệm thu bố cục, khe 2px, kéo–thả hay vòng xoay focus, vì không có đường lái cửa sổ native. **Đừng viết "tương đương" bằng suy luận.** Bàn giao **Story 1.3 / 10.9**, nơi đã có lượt runner hai nền tảng để bấu vào. *(Tiến bộ so với Story 1.6: cổng 1420 lần này **rảnh**, nên `tauri dev` chạy được — giới hạn còn lại là lái GUI, không phải hạ tầng.)*

- ⚠️ **Preset `Review Mode` chưa dựng — Story 8.11.** `LAYOUT_PRESETS` hôm nay có **hai**: `layout.preset_grid` *(2×2, mặc định)* và `layout.preset_columns` *(bốn cột)*. Hợp âm `Mod+Alt+3` **để trống có chủ ý** cho preset thứ ba *(`Bản dịch của tôi` cạnh `Bản Reviewer đã sửa`)*, đúng thứ tự mockup.

- ⚠️ **Preset do NGƯỜI DÙNG đặt tên chưa có đường vào — Story 1.21.** `ScopeKind::LayoutPreset` *(`GlobalOnly`)* và `BootstrapConfig.layout_presets` đã có từ Story 1.8 và story này **không** ghi vào chúng: hai preset trên là hằng số ở frontend. 🔴 Và **KHÔNG dựng thanh chuyển phạm vi Toàn cục/Tác phẩm cho preset** — `kinds.rs:36` gọi tên đích danh cái bẫy đó.
  → 🔴 **ĐỔI CHỦ 2026-08-11 — Story 1.21 TRẢ LẠI món nợ này, và Ice ký.** Lý do đo được: `epics.md:1579-1581` giao FR17/FR18 cho **Story 1.14**, còn `epics.md:1883` giao Story 1.21 **đúng FR22**; một màn quản lý preset đặt tên có **0 AC** ở cả hai chỗ. Dựng một bề mặt cho `ScopeKind::LayoutPreset` trong story phím tắt là thêm một năng lực không AC nào yêu cầu — đúng thứ §KHÔNG-LÀM của mọi story lớn trong dự án này từ chối.
  ✅ Vế *"không dựng thanh chuyển phạm vi"* thì Story 1.21 **có** tuân, và tuân cho chính bề mặt của nó: mockup `settings.html:243-248` vẽ hai nút `Toàn cục`/`Tác phẩm` cho **phím tắt**, và story thay chúng bằng đúng một câu (`shortcuts.scope_note`, nguyên văn `settings.html:246`). Tiền lệ đã có; story sau của preset chép nó.
  **Chủ mới: chưa gán — nêu ở retrospective Epic 1.**

- ⚠️ **Kiểm B của `check-layout.mjs` đo NHỊP GHI, không đo rằng `WorkspaceDock.vue` thật sự dùng lịch đó.** Nó `import()` `src/layout/writeSchedule.ts` và đẩy 1.251 sự kiện qua `simulateWrites()` — kéo sash 3 s ⇒ **1** lượt ghi; kéo liên tục 20 s ⇒ **4** lượt ghi với không thay đổi nào chờ quá **5.000 ms**. Nhưng một lượt sửa `WorkspaceDock.vue` gọi `emit('persist')` thẳng ở mỗi `onDidLayoutChange` sẽ **đi qua cổng** — cổng không thấy chỗ nối. Lưới còn lại là một lượt đếm tay trong DevTools. Cùng hạng với *"cổng không được type-check"*.

- ⚠️ **`localStorage`/`sessionStorage` gọi TRẦN vẫn đi qua một mệnh đề CẤM, không qua danh sách cho phép.** Kiểm C của `check-layout.mjs` hỏi ngược *"mọi thành viên `window.`/`document.` phải nằm trong danh sách CHO PHÉP"* — đúng lập luận của `config_invariants.rs:92-94`. Nhưng `localStorage` không tiền tố là một **định danh tự do**, và liệt kê hết định danh tự do đòi một bộ phân tích cú pháp thật *(một phụ thuộc npm mới — NFR15)*. Ba cái tên đó vì vậy vẫn nằm trong một danh sách cấm hẹp. **Mở lại** khi dự án có lý do độc lập để thêm một parser.

- ⚠️ **Bảng `PANEL_COMPONENTS` và map `components` phải khớp nhau, và không cổng nào canh.** `src/layout/workspaceLayout.ts` khai tên component dạng chuỗi; `WorkspaceDock.vue` khai map thật. Một tên lệch cho ra **panel trắng** kèm `console.error` của chính dockview — không cổng nào đỏ. Rẻ nhất để đóng: cho `check-layout.mjs` đọc luôn map trong `.vue`. Không làm ở story này vì nó đòi một bộ phân tích `.vue` thứ ba trong cây script.

- ⚠️ **Chín biến `--dv-tab-group-color-*` cố ý ĐỂ TRỐNG.** Chúng phục vụ tính năng "tab group có màu" mà sản phẩm không dùng ở đâu. Khai chúng đòi **chín màu MỚI** phải qua Kiểm C của `check-tokens.mjs` — tức mở một bảng màu thứ hai để phục vụ một tính năng không dùng. Ngày nào sản phẩm dùng tới, đó là một quyết định thiết kế **có chữ ký**.

- ⚠️ **Ba biến `--dv-*` mang tên KHÔNG khớp thuộc tính CSS mà cổng đọc.** `--dv-floating-box-shadow` không khớp `box-shadow` của Kiểm F; `--dv-overlay-z-index` không khớp `z-index`; `--dv-floating-group-dragging-opacity` không khớp `opacity` của Kiểm D. Cả ba **đã được đặt đúng luật bằng tay** *(`none` · một ngữ cảnh xếp lớp cơ học có ghi lý do · `1`)* và lý do viết ngay cạnh — nhưng đó là **kỷ luật, không phải cưỡng chế**. Ngày dockview thêm một biến kiểu này, không gì báo. `epics.md:381` nói ranh giới kiến trúc phải cưỡng chế **bằng test**; đây là một chỗ nó chưa được.

- ⚠️ **`ui-md` chạy giãn dòng 1.5 nhưng câu trạng thái panel AI XUỐNG DÒNG THẬT** — xem mục *"Kiểm E không phát hiện được một cờ `wraps` khai sai"* ở trên. **Chưa chốt, quyết định của Ice**, và nó chạm `DESIGN.md`.

- ⚠️ **Chuỗi chẩn đoán trong `.vue` phải viết KHÔNG DẤU.** `WorkspaceDock.vue` và `WorkspaceMode.vue` mang ~7 lời gọi `console.error`/`console.warn` viết tiếng Việt **không dấu**, theo tiền lệ `src-tauri/src/commands/config.rs:36`. Lý do: Kiểm A của `check-i18n.mjs` đo **DẤU** và không phân biệt được *chuỗi hiển thị* với *chẩn đoán ra console*. Đường thoát dễ — dời khối logic sang một `.ts` — là **đúng đường mà `deferred-work.md:35` cấm bằng chữ**, nên không dùng. Lời giải đúng là cho cổng một khái niệm *"chẩn đoán"* *(ví dụ: chuỗi nằm trong đối số của `console.*` được miễn trừ có tên)*. Thuộc **Story 10.9**.

- ⚠️ **`as unknown as Record<string, VueComponent>` ở `WorkspaceDock.vue`.** `dockview-vue` khai `VueComponent<T = any> = DefineComponent<T>`, và prop là vị trí **nghịch biến** nên `DefineComponent<DockviewPanelProps>` không gán được. Đường thay thế *(khai `params?:` ở cả năm component)* qua được kiểm tra kiểu **bằng cách nói dối**: dockview LUÔN truyền `params`, và `PanelTab.vue` không chạy được nếu thiếu. Ép kiểu **một lần ở đúng ranh giới thư viện** rẻ hơn năm lời nói dối rải trong mã. **Mở lại** nếu `dockview-vue` siết kiểu ở một bản sau.

## Deferred from: code review of 1-14-khung-bon-panel (2026-08-06)

- **Khoá tiêu đề panel chảy qua một lời gọi `t()` KHÔNG literal, ngoài tầm quét của `check-i18n.mjs`.** `PANEL_TITLE_KEYS` sống ở `src/layout/workspaceLayout.ts` (một tệp `.ts`, Kiểm A2 chỉ quét `.vue`) và đổ vào `PanelTab.vue:80` qua `t(props.params.params.titleKey ?? '')` — một biểu thức, không một literal. Giá trị hôm nay đều khớp `vi.json` (xác minh trực tiếp), nhưng một lỗi gõ tương lai trong bốn khoá đó sẽ không bị cổng nào bắt — `resolve.ts` cố ý không sập với khoá thiếu, nên hậu quả là khoá thô hiện ra màn hình. Cùng lớp rủi ro với mục *"Bảng `PANEL_COMPONENTS` và map `components` phải khớp nhau"* ở trên.
- **`PANEL_SUFFIXES` ở `src/commands/index.ts:172-173` là bản chép tay của `PANEL_IDS`** (`src/layout/workspaceLayout.ts`), chỉ có một dòng comment "chép từ", không cổng nào đối chiếu hai bảng. Thêm/đổi tên/xoá một panel sau này có thể làm bốn `layout.toggle_*` trôi khỏi `PANEL_IDS` mà không cổng nào đỏ.
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
    `libraryImport.ts::finishSubmit` khi Tác phẩm đổi. ⚠️ **Vẫn MỞ cho Editor/AI (Epic 2/4)**.

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

- ⚠️ **Sáu số `Tuning` của Story 2.4 nay càng khó đo hơn: một phiên có thể chạy HAI kho cùng lúc** (`global.db` + `project.db` của Tác phẩm đang mở) — mỗi kho tự mang luồng checkpoint + pool 4 kết nối riêng. Chưa có phép đo nào về tranh chấp CPU/I/O giữa hai luồng checkpoint chạy song song trên cùng một tiến trình. **Story 2.4** đo lại cả sáu số trên Editor thật, nay nên đo trong ĐÚNG kịch bản hai kho, không phải một kho đơn lẻ.

- ⚠️ **`ports::ProjectStore` được khai (AD-2, Task 3) nhưng CHƯA CÓ CÀI ĐẶT nào** — cùng hoàn cảnh `TranslationProvider` (Epic 4). `commands::project` gọi thẳng `Option<&Store>`/`&Path` (khuôn `commands::config`), không qua cổng này. Cắm một `impl ProjectStore` thật là việc của epic đầu tiên cần trừu tượng hoá trên "một Tác phẩm đã mở" (ứng viên: Epic 2 Editor, Epic 3 Glossary).

- ⚠️ **Tên Tác phẩm rỗng rơi về `"Untitled"` (tiếng Anh, không dịch)** — `core::library::atproj::sanitize_name`. Đây là một tên **thư mục hồi phòng**, không phải văn bản hiển thị (NFR16 áp cho UI, không áp cho tên tệp hệ thống), nhưng nó là quyết định thẩm mỹ chưa ai duyệt. Không có AC nào của story đòi validate trường "Tên" ở tầng giao diện trước khi nộp — form hôm nay cho phép nộp tên rỗng. **Story nào dựng màn hình gán tên Tác phẩm tử tế hơn** (nếu có) nên xét lại.

- ⚠️ **Đường "Dán văn bản" và ba điểm vào của Quyết định #1 (ô nhập đường dẫn, vùng kéo-thả) KHÔNG có mockup nào trong 29 tệp quy hoạch.** Giao diện `LibraryMode.vue` của story này được suy ra từ `.field`/`.dlg` của `mockups/library-and-import.html` + §Voice and Tone, không sao chép một thiết kế đã duyệt. Cùng khoảng trống mà story đã nêu cho Sally ở §Câu hỏi cho Ice — chưa có lượt thiết kế thị giác chính thức cho ba điểm vào này.

## Deferred from: code review of 1-15-tac-pham-tren-dia-va-duong-vao-van-ban-toi-thieu (2026-08-06)

- ⚠️ **`replace_open_work` thả `Store` cũ TRONG vùng khoá mutex** (`src-tauri/src/commands/project.rs:204-211`). `*guard = Some(new_work)` chạy `Drop` của `OpenWork` cũ ngay tại chỗ, mà `Drop` đó gọi `Store::close()` — join luồng writer + một lượt checkpoint TRUNCATE có trần — **trong khi vẫn giữ `OpenWorkState`**. Hôm nay chưa phải lỗi đang sống: chỉ có hai chỗ chạm khoá này (`replace_open_work` và `close_open_work`), và chỗ thứ hai chỉ chạy lúc `RunEvent::Exit`, nên không có tranh chấp thật. Nó trở thành rủi ro thật khi một story sau thêm **bất kỳ command nào đọc `OpenWorkState`** (Epic 2 Editor, Epic 3 Glossary là ứng viên gần nhất) — khi đó một lượt "mở Tác phẩm khác" sẽ chặn mọi lượt đọc state trong suốt thời gian đóng kho cũ. Khuôn sửa rẻ và đã biết: `let old = { let mut g = state.lock()…; g.replace(new_work) }; drop(old);` — thả kho cũ **ngoài** vùng khoá. Cùng họ với mục sáu số `Tuning` chưa đo ở trên.

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

- ⚠️ **`err.project.meta_too_new` và `MessageKey::ProjectMetaTooNew` ĐÃ BỊ GỠ ở lượt code review 2026-08-06** — Ice chốt. Cơ chế từ chối một `meta.json` phiên bản mới hơn **vẫn còn nguyên và vẫn có test** (`MetaError::SchemaTooNew` + `WorkMeta::read` + `project_contract.rs::a_newer_meta_schema_is_refused_without_touching_a_single_byte`); thứ bị gỡ là **bề mặt hiển thị** của nó. Lý do: Story 1.15 không dựng màn hình *"mở lại một `.atproj` đã có"*, nên `WorkMeta::read` **không có một chỗ gọi sản phẩm nào**, nên một `MessageKey` + một khoá `vi.json` cho nó là **một khoá cho tính năng chưa tồn tại** — đúng thứ Story 1.7 §Completion Notes #3 cấm và `scope_contract.rs` trích lại nguyên văn. 🔴 **Story nào dựng đường mở lại một `.atproj`** *(ứng viên: Epic 5, lưới Tác phẩm)* **thêm lại cả ba thứ — biến thể `ProjectError`, `MessageKey`, khoá `vi.json` — CÙNG MỘT LƯỢT với màn hình.**

- ⚠️ **AC2 đếm "đúng ba thành phần", nhưng một `.atproj` đang SỐNG có NĂM mục trên đĩa** — `project.db-wal` và `project.db-shm` là sidecar của chế độ WAL. `project_contract.rs::creating_a_work_lays_down_exactly_three_things_on_disk` lọc hai tệp đó ra trước khi so, và bộ lọc đó **hợp lý** *(chúng là một phần của chính `project.db`, không phải một tệp lạc)* — nhưng nó là một cách **diễn giải lại** AC2 mà story không khai trong năm độ lệch. 🔴 **Quan trọng cho Epic 5:** `Indexer` quét thư mục sẽ gặp **năm** mục, không phải ba, và một lượt quét cho rằng `.atproj` chỉ chứa đúng ba tên sẽ sai. `close()` chạy TRUNCATE nên `-wal` co về 0 byte, **không** biến mất.

- 🔴 **NGHIỆM THU TRÊN WINDOWS CHƯA TỪNG CHẠY — mở mới 2026-08-06, sau khi lượt nghiệm thu tay macOS đóng hai món nợ ở trên.** Cả năm bảng nghiệm thu của Story 1.15 *(trùng tên không xoá dữ liệu · form không biến mất · kéo-thả điền vào ô · ba ca biên `.docx`/không đuôi/BOM · AC10(a) đường hiển thị lỗi kho)* chạy **chỉ trên macOS/WKWebView**. CI dựng **cả `macos-26` lẫn `windows-2025`** và **NFR14 là một mệnh đề hai nền tảng**, nên **đừng đọc hai mục đã đóng ở trên thành "đã xong mọi nền tảng"**. Bốn đường sau là **Windows-only theo bản chất** ⇒ **không** phép kiểm nào chạy trên macOS có thể làm chúng đỏ, kể cả khi chúng hỏng:
  1. **Tên thiết bị dành riêng** — `CON.txt`/`NUL.md`/`COM1` chỉ bị NTFS từ chối; trên macOS chúng là tên thư mục hợp lệ. `sanitize_name` thêm hậu tố `_` và có test đơn vị (`a_folder_name_survives_both_platforms_rules`), nhưng chưa ai xác nhận một thư mục `CON_.atproj` **thật sự tạo được** trên NTFS.
  2. **`remove_dir_all` với một tệp đang mở** — đúng bài học NFR14 mà `close_open_work`/`Store::close` tồn tại để chống. Trên macOS xoá một tệp đang mở là **hợp lệ**, nên đường này **không thể** đỏ ở đó.
  3. **Trần độ dài đường dẫn** — `MAX_FOLDER_NAME_BYTES = 180` nhắm `NAME_MAX` 255 byte của APFS/ext4, nhưng Windows còn có trần **`MAX_PATH` 260 ký tự cho CẢ đường dẫn**, mà `C:\Users\<tên>\Documents\AuraTranslate\` đã ăn một phần đáng kể. Con số 180 chưa được đo với ràng buộc đó.
  4. **Kéo-thả trên Windows** — `WindowEvent::DragDrop` đi qua **WebView2/Win32**, một cài đặt runtime khác hẳn WKWebView/AppKit; và ba event `DRAG_ENTER_EVENT`/`DRAG_LEAVE_EVENT`/`DRAG_DROP_EVENT` là **đường mã mới** sinh ra ở lượt code review 2026-08-06, chưa chạy trên nền tảng nào ngoài macOS.

  **Chủ: lượt QA trước khi phát hành**, hoặc story đầu tiên có một máy Windows thật trong tay. Đừng đóng mục này bằng một lượt chạy CI xanh — CI chạy `cargo test`, không dựng cửa sổ và không kéo-thả.

## Deferred from: 1-10c-am-han-viet-dung-nguon-va-dung-nhan (2026-08-06)

- ✅ **Story 1.16 (tab Hán Việt) HẾT CHẶN.** Lý do chặn — `dict_entry.han_viet` của lớp nền mang âm NÔM (`Unihan kVietnamese`) thay vì âm Hán Việt — đã đóng: AC1/AC2 đổi vai `kVietnamese` sang `nom_reading`, `han_viet` giờ chỉ nhận âm gắn nhãn tường minh (Thiều Chửu · en-wiktionary-vi · Trần Văn Chánh). Bốn tệp `.db` đã dựng lại (`SCHEMA_VERSION` v2), manifest đã cập nhật, mọi cổng xanh.
  ⚠️ **Bẫy 4 của story này — Story 1.16 PHẢI đọc trước khi viết đường tách âm đọc:** ba quy ước phân tách nhiều-âm-trong-một-chuỗi tồn tại song song trong dữ liệu thật — Thiều Chửu dùng `|` (`"đinh|chênh"`), Trần Văn Chánh **và** en-wiktionary-vi dùng `,` (`"đáng, đương"`, có khoảng trắng sau dấu phẩy ở TVC), Unihan/`nom_reading` cũ dùng khoảng trắng (`"tợ tử"`). Story 1.10c KHÔNG chuẩn hoá bốn quy ước này về một — `nom_guard::split_readings` (`tools/dict-build/src/nom_guard.rs`) đã viết luật "cắt trên cả `|`, `,`, khoảng trắng" cho MỤC ĐÍCH ĐỐI CHIẾU AC5 (build-time only, không ghi lại `.db`), và đó chính là luật Story 1.16 nên tái dùng ở đường ĐỌC — xem module đó làm tham khảo trước khi viết một bộ tách thứ hai.

- ⚠️ **Nguồn kaikki.org khai DEPRECATED trên trang tải** (`raw/en_wiktionary_vi/`, ghim 2026-08-06, `Last-Modified: 2026-08-02`). Story này ghim đúng bản đã tải, không có đường thay ổn định hơn tại thời điểm khảo sát. **Câu hỏi mở cho Ice** (chưa trả lời — story 10.1 hoặc lượt làm mới dữ liệu kế tiếp phải quyết): ai/khi nào làm mới `dict-core.db` theo một dump kaikki mới hơn, và làm gì nếu kaikki ngừng phục vụ hẳn (sáu trong bảy nguồn nền hôm nay đi qua `wiktextract_common.rs`, tức phụ thuộc CÙNG một nhà cung cấp trích xuất).

- 🔴 **Dư địa NFR6 còn lại sau story này: 3.104.634 byte (0,78% trần)** — **SỬA ở lượt code review 2026-08-06**: bản ghi gốc của story ("26.760.192 byte còn lại") dùng baseline "trước story" SAI (343.991.430, số CŨ của `epics.md:336` từ TRƯỚC Story 1.10b, KHÔNG cộng font+baseline app+license). Baseline ĐÚNG là số Story 1.10b tự đo (`1-10b-...md:934,963,1087`) = **384.525.446**. Payload THẬT sau story 1-10c = **396.895.366 / trần 400.000.000**. `prd.md:946` đã cảnh báo dư địa này vốn dành cho **HVTĐTD + Cổ hán văn** — với chỉ **3,1 MB** còn lại, 🔴 **hai lớp đó gần như CHẮC CHẮN không còn vừa** trừ khi cực nhỏ. **Chưa đo HVTĐTD/Cổ hán văn thật** — quyết định tầng PRD (nâng trần, hoãn một lớp, hoặc bỏ `sense_fts_nd`) cần cân nhắc SỚM hơn dự tính ban đầu, không phải quyết định của story dựng dữ liệu tiếp theo. **Đo TRƯỚC khi hứa đóng gói** — đúng bài học `prd.md §8.2` đã ghi cho chính hai lớp này, giờ càng cấp thiết hơn.

- ✅ **Lưới AC5 (`nom_guard`) — sửa lỗ hổng cấu trúc + dương tính giả, cả hai phát hiện ở lượt code review 2026-08-06 (SAU khi story đã ở trạng thái `review`).** (1) Bản gốc: `LABELED_NOM_SOURCE` chỉ có ở `dict-core.db`, nên AC5 vĩnh viễn `0/0` cho ba tệp gỡ rời (AD-10: một tệp một `dict_source`) — sửa bằng nạp nhãn Nôm từ raw `en-wiktionary-vi` cho MỌI lớp gỡ rời (`build.rs::load_en_wiktionary_vi_labeled_nom`, không thêm mã riêng-từng-nguồn). (2) Sửa xong lộ dương tính giả THẬT: Thiều Chửu (nguồn chuẩn) bị gắn cờ 63,4% — nguyên nhân là `en-wiktionary-vi` tự gắn cả hai nhãn HV/Nôm cho cùng âm khá thường xuyên. Sửa bằng `nom_guard::nom_only_readings` (loại âm tự-trùng-vai khỏi vế đối chứng). Số cuối trên dữ liệu thật: thieu-chuu 5,2%, tran-van-chanh 6,5%, cả hai an toàn; mệnh đề "đỏ được" của AC5 đo lại 79,5% (từ 92,4% gốc, do siết phép lọc) — vẫn cách xa ngưỡng. Bốn SHA-256 `.db` **không đổi** — bản vá chỉ đổi phép kiểm, không đụng dữ liệu ghi ra. **Story 1.16** nên đọc `nom_guard.rs` (cả `split_readings` VÀ `nom_only_readings`) trước khi viết logic liên quan tới HV/Nôm.

- ⚠️ **`dict-tran-van-chanh.db` mang rủi ro pháp lý CHƯA ĐÓNG, có chủ ý** — Trần Văn Chánh (1999) còn trong bản quyền, tác giả còn sống, dự án CHƯA xin phép trực tiếp. Giảm thiểu: đóng gói làm lớp gỡ rời (FR112 = xoá một tệp), `license_kind = "copyrighted"`, rủi ro ghi thẳng vào `dict_source.attribution` + `assets/licenses/tran-van-chanh.txt`. **Chủ: lượt rà pháp lý trước khi phát hành công khai** (cùng nhóm với rủi ro VietPhrase/Cổ hán văn đã ghi ở `prd.md §8.6`) — xin phép tác giả hoặc chấp nhận rủi ro có ý thức là quyết định tầng dự án, không phải quyết định kỹ thuật.

- 📝 **Lệch giữa story này và `prd.md §8.2` — ghi ra, KHÔNG sửa file quy hoạch (đúng ranh giới story đã khai).** `prd.md:922` liệt Trần Văn Chánh với trạng thái *"Còn bản quyền · Đã loại"*. Story 1.10c (§Năm quyết định #1, Ice chốt 2026-08-06) đảo quyết định đó: TVC ĐƯỢC dựng, làm lớp gỡ rời thứ ba, với rủi ro pháp lý ghi thẳng vào `attribution` thay vì tránh né bằng cách loại bỏ. `docs/dics/README.md §_khong-dung` cũng còn ghi TVC ở nhóm "đừng dùng" — lý do gốc (`Pleco`/`.xlsx` TRỘN hai từ điển không tách được nguồn) vẫn ĐÚNG cho HAI tệp đó, nhưng KHÔNG áp cho tệp `.tab` chuyên biệt mà story này thực sự dùng (`catusf/tudien` → `dict/Tu-dien-ThienChuu-TranVanChanh.tab`, tự nó là một nguồn ghi công được, kiểm chứng qua `Tu-dien-ThienChuu-TranVanChanh.toml` cạnh nó). **Ai sở hữu đồng bộ lại `prd.md`/`docs/dics/README.md`:** lượt quy hoạch kế tiếp chạm §8.2, hoặc Story 10.4 (màn hình Attribution) khi nó cần bảng nguồn khớp thực tế.

- ⚠️ **Tệp thô `Tu-dien-ThienChuu-TranVanChanh.tab` (`catusf/tudien`) thực chất TRỘN hai phong cách nội dung** — một số dòng mang văn phong/cách đánh số Thiều-Chửu-cũ (số khoanh tròn ①②③, từ vựng cổ), một số dòng khác mang văn phong TVC hiện đại (nghĩa tiếng Trung giản thể, ví dụ đương đại) — cùng một ký tự có thể xuất hiện ở CẢ HAI phong cách trên các dòng KHÁC nhau (ca thật: `長`/`行` — xem test `duplicate_headword_lines_stay_as_separate_entries` của `tran_van_chanh.rs`). Story này KHÔNG cố tách hai phong cách đó thành hai nguồn — toàn bộ tệp được ghi công như MỘT nguồn `tran-van-chanh` (đúng thực tế phân phối của `catusf/tudien`, đúng tinh thần tiền lệ `thieu_chuu.rs` không tự suy đoán cấu trúc nội bộ của một tệp thô). Ghi ra để người đọc dữ liệu sau này không ngạc nhiên khi thấy hai văn phong khác hẳn nhau dưới cùng một `source_id`.

## Deferred from: code review of 1-10c-am-han-viet-dung-nguon-va-dung-nhan (2026-08-06)

- `nom_guard::split_readings` cắt đồng thời trên cả ba quy ước phân tách (`|`, `,`, khoảng trắng) dù chúng là quy ước RIÊNG của ba nguồn khác nhau — deferred, pre-existing design tradeoff đã ghi rõ trong doc-comment (Bẫy 4) của chính module; phạm vi sửa đường ĐỌC thuộc Story 1.16, xem module này làm tham khảo. [`tools/dict-build/src/nom_guard.rs:46-56`]
- So sánh âm đọc xuyên nguồn (`nom_guard`) và các parser mới dùng so khớp chuỗi thô, không chuẩn hoá Unicode (NFC/NFD) — rủi ro lý thuyết, chưa có bằng chứng xảy ra trên dữ liệu thật hôm nay; cân nhắc cùng lượt Story 1.16 chuẩn hoá đường đọc âm đọc. [`tools/dict-build/src/nom_guard.rs:108`]
- Thiều Chửu và Trần Văn Chánh cùng lấy từ `catusf/tudien` nhưng ghim cách nhau ba năm (tag `2.2`/2022-10-10 so với commit 2025-12-19), chưa đối chiếu lại xem hai bản có còn nhất quán — câu hỏi về tính toàn vẹn nguồn, không chặn story 1-10c, cân nhắc khi có lượt làm mới dữ liệu tiếp theo. [`tools/dict-build/src/sources/thieu_chuu.rs:12`; `tools/dict-build/src/sources/tran_van_chanh.rs:37`]

## Deferred from: 1-16-panel-source-va-tab-han-viet (2026-08-06)

- 🔴 **Vế thị giác hai nền tảng thật (WKWebView macOS · WebView2 Windows) CHƯA đo được** — dải tab, bề mặt song song (`position: absolute` cho `.hv-reading`, xem Debug Log References của story), và `font-synthesis` chữ Hán nghiêng giả chỉ được xác nhận đúng CƠ CHẾ qua Playwright/**headless Chromium** — một engine THỨ BA, không phải một trong hai engine mục tiêu. Dự án `không có runner đo được vế đó` — món nợ cũ (`deferred-work.md:478`, Story 1.6/1.14), story này KHÔNG đóng nó, chỉ kế thừa. Nghiệm thu mắt trên máy thật là bước còn thiếu trước khi đóng dấu "đã kiểm hai nền tảng".
- ⚠️ **AC9 (đổi preset ⇒ không gọi lại IPC) đúng CẤU TRÚC MÃ, chưa đo bằng webview đang chạy.** `ensureChapterLoaded`/`ensureHanVietLoaded` (`src/panels/sourcePanelState.ts`) dùng cờ module-level nên về logic KHÔNG THỂ gọi lại `read_open_chapter`/`read_han_viet` ở lượt mount thứ hai — nhưng phiên dev-story không có một instance `tauri dev` rảnh để tạo Tác phẩm, bấm `Mod+Alt+1`↔`Mod+Alt+2`, và đọc DevTools Network thật. Nghiệm thu tay còn nợ.
- ⚠️ **Trần render kiểu song song (50.000 ký tự Hán) đo trên headless Chromium, không phải WKWebView/WebView2, và không đi qua bộ máy reactivity của Vue** (DOM dựng thẳng `document.createElement`, rẻ hơn Vue một chút vì bỏ VDOM diff). Bảng số ở Completion Notes của story là **cận dưới hợp lý**, không phải con số cuối cùng đã đóng dấu trên hai nền tảng thật — nếu đo lại cho ra số khác đáng kể, hằng `PARALLEL_VIEW_RENDER_CEILING` (`sourcePanelState.ts`) là chỗ sửa.
- 📝 **`HanVietLookup.sources_used` mang `dict_source.code` thô** (`fx-hv`, `thieu-chuu`, …), không `display_name` đẹp ("Thiều Chửu"). FR31 (nhãn nguồn bắt buộc) thoả bằng `code`; ánh xạ sang tên hiển thị là việc của màn hình Attribution — **Story 10.4** (đã ghi rõ trong Ranh giới phạm vi của chính story 1.16). Nếu 10.4 cần `display_name` ở đây sớm hơn dự tính, cách rẻ nhất là thêm nó vào `HanVietReading`/`sources_used` qua `layer.source(code)` — hạ tầng đã sẵn (`DictLayer::source`), chỉ chưa nối.
- 📝 **§Câu hỏi cho Ice #2 (báo hay không báo ký tự nhiều âm) và #3 (hình dạng placeholder ký tự không âm) — dùng MẶC ĐỊNH ĐỀ XUẤT của story, CHƯA được Ice xác nhận lại trong phiên dev-story này.** #2: không đánh dấu gì cho ca nhiều âm (danh sách đầy đủ vẫn đi qua IPC qua `HanVietReading.all`, sẵn cho Story 1.17/3.7). #3: hai chuỗi `vi.json` riêng theo `layersLoaded` (`panel.source.han_viet_unknown`/`han_viet_unavailable`), không dùng `ornament`/`opacity`. Nếu Ice muốn một hướng khác, cả hai đổi được mà không đụng tầng dữ liệu.

- 🔴 **Kiểu song song CHỒNG CHỮ thật — claim "giãn dòng 2.05 đủ chỗ cho âm đọc" của Task 8 SAI, đo lại lật.** Ice báo lỗi trực tiếp 2026-08-07 (`.hv-parallel` đọc không được, âm đọc đè lên dòng Hán kế tiếp) sau khi story đã Status `done`. Đo lại bằng `getBoundingClientRect`: ở `line-height: 2.05` (token `source-cjk`), `.hv-reading` (`position: absolute; top: 100%`) đè **19,8px** vào dòng sau — chiều cao hộp dòng chỉ do KÝ TỰ quyết, âm đọc không góp một pixel nào nên toàn bộ chiều cao của nó ăn vào dòng kế tiếp, không nằm "trong phần leading" như comment gốc khẳng định.
  → ⚠️ **VÁ LẦN 1 (4.8, neo `top:100%` vào `.hv-unit`) SAI THEO CÁCH KHÁC — Ice bắt lại bằng ảnh chụp thật cùng ngày.** `.hv-unit` kế thừa chính line-height đã giãn (4.8), nên `top:100%` đẩy âm đọc xuống ĐÁY một hộp CAO — âm đọc trôi XA khỏi ký tự của nó, trôi GẦN dòng SAU hơn, đọc như thể thuộc dòng sau. Phép đo `getBoundingClientRect` (chỉ đo độ đè giữa hai dòng) không lộ ra lỗi này.
  → ⚠️ **VÁ LẦN 2 (3.2, neo vào một `.hv-char` mang `line-height: normal`) VẪN CÒN BA LỖI.** Neo đúng dòng rồi, nhưng Ice báo tiếp: ① **vùng tô khi bôi đen trùm cả hộp dòng cao** (trình duyệt tô selection theo hộp dòng, không theo glyph — line-height 3.2 nghĩa là vệt tô cao gấp ba chữ), trải nghiệm rất khó chịu; ② **âm đọc dòng CUỐI bị cắt, cuộn không tới** (`position: absolute` không đẩy `scrollHeight` của `.hv-surface`); ③ `min-width` giãn ô theo độ dài âm làm **chữ Hán rời rạc**, kéo chọn một từ ghép rất khó nhắm.
  → ✅ **ĐÓNG 2026-08-07 (VÁ LẦN 3 — ĐỔI CƠ CHẾ, Ice chốt).** Âm đọc nay đi bằng `<ruby>`/`<rt>` + `ruby-position: under`, bỏ hẳn `position: absolute`. 🔴 **Ràng buộc gốc buộc phải dùng `absolute` đã HẾT HIỆU LỰC**: Story 1.16 chọn nó vì mọi thứ tạo một hộp dòng mới làm Chromium chèn `\n` vào `Selection.toString()` — nhưng `resolveParallel()` nay đọc thẳng node DOM thay vì tin `toString()` (lượt sửa AC12 của story 1.18), nên chuỗi truy vấn không còn phụ thuộc điều đó. Đo lại (Chromium): âm đọc dòng cuối **cách đáy vùng cuộn 31px và cuộn tới được** (② xong) · vùng tô **ôm sát glyph** (① xong) · chữ Hán **liền nhau tự nhiên** (③ xong) · **không đè ở MỌI mức line-height đã thử, kể cả `normal`** ⇒ token `source-cjk-parallel` của hai lượt vá trước trở nên THỪA và **đã được gỡ** — bộ token về lại **17**, `.hv-parallel` dùng chung `source-cjk` với tab thuần. Hai hàng rào AC6/AC12 nay TÁCH ĐÔI, cần cả hai: `<rt>` mang `user-select: none` giữ cho **copy/paste của người dùng** sạch (đo: thiếu ⇒ `"台đài北"`, có ⇒ `"台北"`), còn **truy vấn tra cứu** đi đường `resolveParallel()` đọc node văn bản trực tiếp của `<ruby>` (không `textContent` — nó gộp cả `<rt>`). Khoảng cách giữa hai âm đọc do `padding-inline: var(--space-unit)` trên `<rt>` (dùng lại token 4px sẵn có, KHÔNG thêm hàng vào bảng đóng băng `EXPECTED_SPACING`) — Ice chốt *"chữ hán xa nhau cũng được, âm hán việt không được đè lên nhau"*. ⚠️ **Mọi số đo trên font hệ thống thay thế, chỉ Chromium** (môi trường đo không có `Noto Serif CJK TC`/`Source Serif 4` thật, không có WKWebView) — cần Ice xác nhận bằng mắt trên `tauri dev` thật, cả hai nền tảng. ⚠️ **`ruby-position: under` là thuộc tính có khác biệt engine đã biết** (WebKit từng cần `-webkit-ruby-position`); chưa đo được trên WKWebView.

---

## Deferred from: code review of 1-16-panel-source-va-tab-han-viet (2026-08-06)

- ✅ **ĐÓNG cùng lượt code review — đã đo bổ sung, kết luận không lật.** `buildSegments`+`switchText` đo được **2,5 / 17,5 / 237,5 ms** ở 5k/50k/500k ⇒ kiểu chuyển đổi ở 500k là **~460 ms** *(bảng cũ ghi 222,4 ms — bỏ sót quá nửa)*, vẫn rẻ cho một thao tác chạy MỘT LẦN ⇒ *"chuyển đổi không có trần"* **vẫn đúng**, trần song song **50.000 giữ nguyên**. Bản vá `min-width` chỉ tốn **+2,7 %** ở 50k. Số đầy đủ ở §Review Findings của story 1.16. ~~Phép đo Task 8 không đo đường mã thật.~~ Bảng số trần render dựng DOM bằng `document.createElement`, không đi qua `buildSegments()` (một object JS cho **mỗi** ký tự Hán) lẫn `switchText` (`.join()` trên toàn bộ mẩu) — mà cả hai **luôn chạy ở CẢ HAI kiểu xem**. ⇒ mệnh đề *"kiểu chuyển đổi không có trần"* (222,4 ms ở 500k) và hằng `PARALLEL_VIEW_RENDER_CEILING = 50_000` đứng trên một phép đo **sai đối tượng**. Cần đo lại trên component Vue thật trước khi tin con số trần. *(Khác với món nợ engine WKWebView/WebView2 — món đó dev đã ghi rõ và trung thực.)*
- **Trần render chỉ đếm ký tự Hán, bỏ qua node của mẩu không-Hán.** `buildSegments` `flush()` mỗi lần gặp một ký tự Hán ⇒ mỗi mẩu không-Hán xen giữa cũng sinh một `<span>` riêng. Văn bản xen kẽ `漢a漢a…` với 49.999 ký tự Hán lọt qua trần nhưng dựng ~100.000 node, trong khi bảng đo chỉ đo văn bản Hán liền mạch. Nhặt lại cùng lượt đo lại ở mục trên.
- **`source_lang` không được validate ở tầng ghi.** `create_work_from_text`/`create_work_from_file` chèn giá trị nguyên văn vào `work`, không một phép kiểm nào; `SourcePanel.vue` so `=== 'zh'` chính xác từng byte. Bất kỳ đường ghi nào khác (`"ZH"`, `"zh-Hans"`, `"cmn"`, hay một `.atproj` chép từ máy khác) cho một Tác phẩm tiếng Trung **không có tab Hán Việt**, không lỗi, không cách nào biết vì sao. Guard đúng nằm ở tầng ghi (Story 1.15), không ở so sánh chuỗi phía UI. **Có sẵn từ trước Story 1.16.**
- **`read_open_chapter` với 0 Chương trả lỗi KHO thay vì lỗi có tên.** `conn.query_row(…)` ném `QueryReturnedNoRows` khi bảng `chapter` rỗng ⇒ qua `From<StoreError>` thành `store.read_failed` ⇒ người dùng đọc *"không mở được kho dữ liệu"* cho một Tác phẩm hoàn toàn lành lặn. AC8 dựng riêng `project.no_work_open` để không trộn trạng thái sản phẩm vào từ vựng `store.*`, nhưng chỉ phủ nhánh `open == None`, không phủ nhánh `open == Some` mà 0 hàng. 🔴 **Epic 1 luôn ghi đúng một Chương nên hôm nay chưa chạm tới; Story 2.x (chọn/chuyển Chương) mở đúng nhánh này.** Guard: `query_map().next()` + một `MessageKey` riêng. *(`src-tauri/src/commands/chapter.rs:60-71`)*

- **Bôi đen nguyên văn bằng BÀN PHÍM — điều kiện tiên quyết của Story 1.18, CHƯA CÀI.** §KHÔNG-LÀM ① của Story 1.16 giao cho story đó **đúng một** nghĩa vụ: nguyên văn phải bôi đen được *bằng chuột **và bằng bàn phím***. Vế chuột đã đo thật bằng Playwright (`Selection.toString()`); vế bàn phím không xuất hiện một lần nào trong Tasks/AC/Completion Notes của 1.16. 🔴 **Ice chốt 2026-08-06 ở lượt code review: ghi nợ cho 1.18** — lý do: thêm `tabindex` lên bề mặt văn bản đụng hợp đồng tiêu điểm mà Story 1.14 dặn không chạm, nên vế bàn phím phải đóng **cùng lượt** với hợp đồng vùng chọn dùng chung cho bốn panel. ⚠️ Một `<div>` không sửa được **không** hỗ trợ Shift+Mũi tên nếu không bật caret browsing — Story 1.18 phải giải bài này, không giả định trình duyệt cho không.
  → ✅ **ĐÓNG 2026-08-07 (Story 1.18, AC11 · Quyết định #2a).** Năm command mới trong `CommandRegistry` (`selection.focus_source` · `extend_left` · `extend_right` · `extend_word_left` · `extend_word_right`), cài bằng `Selection.modify()` trên bề mặt mang `tabindex="0"`. 🔴 **ĐO THẬT trên CẢ HAI engine trước khi chốt** (Task 0, WKWebView qua một bộ đo Swift + Chromium headless): `modify()` chạy trên `<p>` không sửa được ở cả hai, và **Bẫy 9 của story không CÓ THẬT** — `'word'` trên văn xuôi tiếng Trung phân đoạn ĐÚNG (`他` / `打開`), không nuốt cả câu. Giá đã trả và đã ghi: `Tab` nay dừng ở thân Panel Source (Ice chốt chấp nhận 2026-08-07). ⚠️ **Một món nợ MỚI sinh ra**, xem mục `repeat` ở §1.18 bên dưới.


- **Không token nào đỡ được một câu GIAO DIỆN xuống dòng** — cả sáu token `ui-*` *(`ui-md` · `ui-md-strong` · `ui-sm` · `ui-label` · `ui-mono` · `read-title`)* đều khai `wraps: false`, giãn dòng 1.4–1.5, dưới sàn 1.66; còn `check-tokens.mjs` chỉ áp `LINE_HEIGHT_FLOOR` cho token khai `wraps: true` nên cổng **mù hoàn toàn**. Ba chỗ đang chịu: `.parallel-note` *(có sẵn)*, `.load-error` và `.hv-notice` *(thêm ở lượt code review 2026-08-06)* — cả ba là câu đầy đủ, chắc chắn xuống dòng trong một panel hẹp. 🔴 **Đây là lỗ hổng của BẢNG TOKEN, không phải của chỗ dùng** — cùng hạng với hàng `source-latin` còn thiếu mà Quyết định #6 của Story 1.16 vừa vá. Đóng nó là **quyết định của Ice**: đổi cờ `wraps` của `ui-md` *(mục `:115` ngay trên — đây là lần thứ BA nó bị nhắc tên)* hoặc thêm một token thứ 17 qua sổ `deviations`. Lượt code review GHI RA thay vì tự chế một token.
  → ✅ **ĐÓNG 2026-08-06 (Story 1.17, Quyết định #7) — lần thứ TƯ bị gọi tên, lần này Ice chốt hẳn.** Token thứ 17 `ui-md-wrap` (12px/1.66/`wraps:true`), áp cho cả ba chỗ liệt kê ở đây. Xem mục `:115` để có chi tiết đầy đủ.

## Deferred from: 1-17-panel-lookup-ban-ghi-co-cau-truc (2026-08-06)

- 🔴 **`QueryBranch::NoBranchQueryTooShort`/`"query_too_short"` KHÔNG thể xảy ra qua đường sản phẩm thật của chính story vừa dựng nó.** AC6 đòi Panel Lookup hiện chuỗi *"truy vấn quá ngắn"* khi `branch == query_too_short`, và bề mặt (`LookupPanel.vue::queryTooShort`) render đúng nhánh đó — nhưng `commands::dict::lookup()` cố định `LookupMode::Exact` (Quyết định #3), và `pick_branch` cho `Exact` **luôn luôn** trả `ExactBtree` bất kể độ dài truy vấn, bất kể route. Nhánh `query_too_short` chỉ sinh ra khi `mode = Substring` **và** route `En` **và** độ dài < 3 — tổ hợp đó không tồn tại trong bất kỳ lời gọi nào của 1.17. ⇒ Bề mặt UI đúng, đã viết, đã kiểu-khớp với wire — nhưng **chưa từng và không thể được thực thi bằng dữ liệu thật cho tới khi có một chỗ gọi dùng `Substring`**. **Chủ: Story 1.18** (Auto-Lookup, dùng `Substring` khi bôi đen ngắn) hoặc **7.7** (Concordance) — story đầu tiên gọi `LookupMode::Substring` qua IPC phải verify bằng mắt chuỗi "truy vấn quá ngắn" thật sự hiện ra, đừng giả định 1.17 đã làm việc đó.
  → ✅ **ĐÓNG 2026-08-07 (Story 1.18).** Ice chốt bật `Substring` (§Câu hỏi #1) — cài như một **ĐƯỜNG LUI**, không một phép thay thế: tra `Exact` trước, **rỗng** *và* truy vấn ≤ 4 ký tự ⇒ tra lại `Substring`. Tổ hợp `mode = Substring` + route `En` + độ dài < 3 nay **tồn tại trên đường sản phẩm**. Nghiệm thu: `dict_sources.rs::a_short_latin_selection_now_reaches_the_query_too_short_state` (`"zz"` ⇒ `NoBranchQueryTooShort`), **cộng** phép đo trên bốn lớp THẬT — bench `bench_the_auto_lookup_path_on_distinct_queries` đếm **2 lượt `query_too_short`** trên 166 truy vấn khác nhau. không còn là một nhánh chỉ tồn tại trên giấy.


- 🔴 **Vòng IPC Tauri THẬT (serialize Rust → cầu JS → deserialize → Vue reactivity → paint) CHƯA được đo** — cùng hạng món nợ *"vế thị giác hai nền tảng thật"* mà Story 1.6/1.14/1.16 đã để lại, story này **KHÔNG đóng, chỉ kế thừa**. Số đo NFR1 của story dựa trên: (a) backend Rust trên dữ liệu thật (`--release`, đáng tin — p95 6,535 ms), (b) webview render qua Playwright/**headless Chromium** với `invoke` **giả lập trả lời tức thời** (không đo độ trễ round-trip IPC thật). Kết luận NFR1 ĐẠT có cơ sở mạnh (tổng ước tính < 40 ms, cách trần 100 ms một biên độ lớn) nhưng **KHÔNG phải một phép đo đầu-cuối 100% trên WKWebView/WebView2 qua `tauri dev`/bản đóng gói thật**. Xem §Debug Log References của story để có bảng đầy đủ + giới hạn phép đo ghi thẳng.

- ⚠️ **`.parallel-note` (Panel Source, Story 1.16) đổi cỡ chữ 11,5px → 12px** khi chuyển từ `ui-sm` sang `ui-md-wrap` (Quyết định #7). Đây là một thay đổi THỊ GIÁC trên một bề mặt đã ship từ Story 1.16, không chỉ thêm token cho chuỗi mới — Ice đã chốt chấp nhận đổi cỡ để đóng dứt điểm `deferred-work.md:115` thay vì rải thêm một token `ui-sm-wrap`. **Chưa nghiệm thu bằng mắt trên máy thật** (chỉ Playwright headless) rằng 0,5px đổi cỡ không làm vỡ bố cục dải tab của `SourcePanel.vue` ở màn hình hẹp — nhặt lại nếu Story 4.12 (bố cục màn hình hẹp) phát hiện vấn đề.

- 📝 **Mục từ TIẾNG ANH của Panel Lookup dùng hình dạng TẠM** — nhắc lại mục `:317` (chủ sở hữu Sally, `bmad-ux`): `LookupRecord.vue` dùng **cùng cấu trúc khối** cho tiếng Anh và tiếng Trung (chỉ khác token đầu mục), một lựa chọn tự chế ở tầng story mà mục `:317` tự cảnh báo là "đúng cách một bất nhất giao diện ra đời". **KHÔNG ĐÓNG** — chữ ký UX chính thức vẫn thiếu.

- 📝 **`commands::dict::lookup()` không có nhánh lỗi riêng khi `layer.senses(&entry_ids)` trượt** — nếu pha hai hydrate thất bại cho một lớp (lỗi đọc SQLite giữa chừng, hiếm), `unwrap_or_default()` âm thầm trả danh sách nghĩa RỖNG cho lớp đó thay vì báo lỗi hay đưa vào `skipped`/`truncated_layers`. Pha một của lớp đó đã THÀNH CÔNG (nó nằm trong `groups`), nên panel sẽ hiện đúng nhóm nguồn nhưng KHÔNG nghĩa nào — trông giống một đầu mục "chỉ có âm đọc, không nghĩa" (trạng thái hợp lệ theo `senses.rs`) chứ không giống một lỗi. Rủi ro thấp (pha một vừa đọc được từ đúng tệp đó) nhưng chưa có tín hiệu phân biệt hai ca. Nhặt lại nếu có báo cáo thật về hiện tượng "nguồn hiện tên mà không có nghĩa nào".

## Deferred from: code review of 1-17-panel-lookup-ban-ghi-co-cau-truc (2026-08-06)

- 📝 **Chip thanh nhịp bị `overflow: hidden` cắt CÂM khi nhiều nguồn** (`src/panels/LookupPanel.vue:109-113,131-137`) — `.lookup-head` khoá `height: 76px` + `overflow: hidden` để giữ bất biến AC7, còn `.lookup-spine` là `flex-wrap: wrap`. Đo thật cho `山` ra **7–8 nhóm** ⇒ chip tràn sang dòng thứ ba trở đi bị cắt mất hoàn toàn: không dấu hiệu, không cuộn, không chỉ báo `+N`. Đây là đánh đổi CÓ CHỦ ĐÍCH đã ghi trong chú thích tại chỗ (giữ chiều cao bất biến quan trọng hơn), nhưng hệ quả "tên nguồn biến mất" thì chưa ai quyết. Nhặt lại cùng Story 4.12 (bố cục màn hình hẹp) hoặc khi thanh nhịp có chủ sở hữu UX thật.

- 📝 **`layers_loaded = false` khi MỌI tệp `.db` hỏng ⇒ panel hiện "chưa gắn lớp từ điển nào" — một chẩn đoán SAI** (`src-tauri/src/core/dict/layer.rs:460-493`) — `DictLayers::new` đẩy mọi lớp mở-hỏng vào `skipped` chứ không vào `layers`, nên `layers_loaded: !layers.layers().is_empty()` cho `false` cả khi thư mục ĐẦY tệp `.db` hỏng. AC6 dựng chuỗi đó riêng cho ca "thư mục rỗng" (AD-25). Rủi ro thấp: banner `someLayerFailed` vẫn hiện song song nên người dùng không bị bỏ câm, chỉ đọc được hai câu hơi lệch nhau. Sửa gọn: `!layers().is_empty() || !skipped().is_empty()`.

- 📝 **Nhánh `Substring`/`fts_trigram` nạp TOÀN BỘ hàng khớp vào RAM trước khi cắt** (`src-tauri/src/core/dict/query.rs:203-221,238-254,321-333`) — ba nhánh cần `verify_substring` cố tình fetch không giới hạn rồi `cap()` ở Rust để tránh Bẫy 11, nên `limit` không chặn được bộ nhớ lẫn độ trễ, chỉ chặn băng thông IPC. Hôm nay là **latent**: đường sản phẩm 1.17 là `Exact`-only nên không chạm tới ba nhánh này. Trở thành thật khi **Story 1.18/7.7** bật `Substring`. Hướng sửa giữ đúng thứ tự "verify rồi mới `cap`": thêm một trần AN TOÀN ở SQL (vd `LIMIT limit * 50`) làm cận trên cho tập ứng viên.
  → ✅ **ĐÓNG 2026-08-07 (Story 1.18).** `candidate_ceiling(limit) = limit * 50` đặt vào SQL ở cả **ba** nhánh có xác minh. 🔴 Phần tinh tế mà mục này không nêu: khi trần chạm, `verify_substring` có thể loại đủ nhiều dương tính giả để phần còn lại **ít hơn** `limit`, và khi đó `cap()` một mình báo `truncated = false` — đúng câu *"danh sách này đầy đủ"*, và nó SAI. ⇒ `cap_verified()` OR cờ trần vào. Nghiệm thu đỏ-rồi-xanh: `dict_lookup.rs::the_candidate_ceiling_keeps_the_truncated_flag_honest` (60 ứng viên, **0** qua được verify ⇒ `truncated` phải `true`), chứng minh ĐỎ bằng cách nâng hệ số lên 100.000.


- 📝 **Chuỗi `query_too_short` chỉ dẫn một thao tác không TỒN TẠI trong panel** (`src/i18n/vi.json:65`) — *"gõ thêm ít nhất ba ký tự"*, nhưng Panel Lookup không có ô nhập nào: truy vấn chỉ đến từ vùng chọn (`main.ts:182`). Cộng với việc chính story tự khai nhánh `query_too_short` không thực thi được qua đường sản phẩm `Exact`-only hôm nay, đây là một chuỗi vừa không hiện được, vừa vô nghĩa nếu hiện. **Story 1.18/7.7** sẽ kế thừa nguyên văn nó — sửa lúc bật `Substring`, cùng lượt với hợp đồng vùng chọn.
  → ✅ **ĐÓNG 2026-08-07 (Story 1.18).** Chuỗi đổi thành *"Đoạn đang chọn quá ngắn để tra chuỗi con — **chọn** thêm ít nhất ba ký tự."* — nay chỉ một thao tác CÓ THẬT (bôi đen thêm), không một ô nhập không tồn tại.


- 📝 **`window.getSelection()` mù với `<input>`/`<textarea>`; vùng chọn rỗng = im lặng tuyệt đối** (`src/main.ts:182`, `src/commands/index.ts:475-477`) — trên Chromium/WebKit, `window.getSelection().toString()` trả `''` cho vùng chọn BÊN TRONG một ô nhập, nên bấm `Mod+Alt+L` khi con trỏ ở ô nhập của Library trông y hệt một phím tắt hỏng; và `text.trim() === ''` trả về sớm không phản hồi nào. **Story 1.18** sở hữu hợp đồng vùng chọn dùng chung — dep hôm nay là dep TỐI THIỂU theo đúng Quyết định #1a, nên hai vế này đóng cùng lượt đó chứ không ở đây.
  → ✅ **ĐÓNG 2026-08-07 (Story 1.18, AC3 · Task 1).** `src/panels/selectionContract.ts` — sổ đăng ký **OPT-IN** theo phần tử, một listener trên `document`. 🔴 **VÀ PHÉP ĐO LẬT LÝ DO CỦA CHÍNH MỤC NÀY:** `getSelection().toString()` trong một `<input>` trả **`"nội "`** (văn bản THẬT) trên **cả** Chromium **lẫn** WKWebView — không phải `''`. Lời khuyên của mục này đúng, lý do thì sai. Phép loại trừ đã cài không dựa vào chuỗi rỗng lẫn `document.activeElement` (đo được: `activeElement` cho **âm tính giả** khi tiêu điểm ở ô nhập mà vùng chọn nằm nơi khác) — nó đọc **`anchorNode.nodeType`**: một vùng chọn chữ thật luôn neo vào node VĂN BẢN, vùng chọn trong ô nhập neo vào node PHẦN TỬ. Phân biệt sạch trên cả hai engine.


## Deferred from: 1-18-auto-lookup (2026-08-07)

- 🔴 **`Selection.modify()` đi XUYÊN QUA `user-select: none` trên WKWebView — và story này đã vá chỗ dùng, không vá được nguyên nhân.** Đo 2026-08-07, vùng chọn cả đoạn ở kiểu song song: `Selection.toString()` cho `他打開了那扇門，走進了黑暗之中。` trên Chromium *(đúng)* nhưng `他tha打đả開khai了liễu…` trên WKWebView — tức **rò âm Hán Việt vào truy vấn**. `user-select: none` chi phối vùng chọn do **chuột kéo** (số đo Playwright của Story 1.16 vẫn đúng); nó **không ràng buộc** `Selection.modify()`, mà `modify()` chính là đường bàn phím AC11 vừa dựng. ⇒ `SourceHanViet.vue::resolveParallel` nay đọc thẳng node `.hv-char` thay vì tin `toString()` — đúng trên cả hai engine. ⚠️ **Cái không đóng:** mọi bề mặt TƯƠNG LAI dùng `user-select: none` để loại chữ khỏi vùng chọn *(Story 3.4 — đánh dấu thuật ngữ Glossary; Epic 2 — Editor)* thừa hưởng nguyên cái bẫy này, và không cổng nào canh. Cân nhắc một vị từ dùng chung ở `selectionContract.ts` khi bề mặt thứ hai xuất hiện.

- ⚠️ **Giữ phím không mở rộng vùng chọn liên tục — phải bấm lặp** (`src/commands/keys.ts:295`). `handle()` trả sớm khi `event.repeat === true`, một luật đúng cho 17 command cũ (*lặp lại "đổi chế độ" là vô nghĩa*) và sai cho đúng **bốn** command `selection.extend_*` của story này. Nới nó cần một cờ `repeatable` trên `CommandSpec`, chạm `registry.ts` + `keys.ts` + **mọi** command đang có ⇒ không thuộc phạm vi 1.18. Hai command **theo TỪ** (`Alt+Shift+←/→`) bù phần lớn chi phí thao tác. **Chủ: Story 1.21** (màn hình gán phím — nó vốn phải mổ tầng này).
  → ✅ **ĐÃ ĐÓNG 2026-08-11 (Story 1.21).** Cờ `repeatable?: boolean` trên `CommandSpec`, mặc định **không**; `frozen()` chuẩn hoá nó về `boolean` ở cửa vào; `keys.ts::handle` đọc `event.repeat === true && !entry.repeatable`. **Bốn** chỗ khai `true` — đúng bốn `selection.extend_*` — và không chỗ nào khác.
  🔴 **Ice ký NHẬN món nợ này 2026-08-11, và việc đó là một quyết định có chủ, không một sự trôi phạm vi.** Story 1.21 đề xuất **trả lại** cả ba món nợ mang tên nó vì cả ba có **0 AC** ở `epics.md`; Ice lật một phần, nhận đúng cái này. Cái giá đã nói trước và Ice ký nhận: nó chạm `registry.ts` + `keys.ts` + mọi command đang có, cho một thay đổi không AC nào yêu cầu.
  ⚠️ Lưới: Kiểm D của `check-commands.mjs` có **hai** khẳng định — nhánh dương *(`repeatable: true` ⇒ keydown lặp VẪN dispatch)* và **đối chứng âm** trên **cùng một keymap** *(command không khai cờ ⇒ keydown lặp vẫn bị chặn)*. Không có vế thứ hai thì một bản cài đặt bỏ quên cờ hoàn toàn vẫn xanh.

- 🔴 **NFR1 đo được ĐẦU-CUỐI ở tầng Rust, không qua vòng IPC Tauri thật lẫn lượt VẼ của webview** — món nợ Story 1.17 để lại, story này **KẾ THỪA, không ĐÓNG**. Số đã đo (4 lượt độc lập, `--release`, 4 lớp `.db` thật, 166 truy vấn KHÁC NHAU, đường sản phẩm `commands::dict::lookup` gồm cả đường lui `Substring`): trạng thái ổn định **p50 ~1,0 ms · p95 1,8–2,4 ms · p99 5,5–9,8 ms · max 20–25 ms**. Cơ chế đo đầu-cuối THẬT đã cài và bật được tay (`src/panels/lookupTiming.ts`, cờ mặc định TẮT, `__auraLookupTiming.enable()`), mốc cuối sau `requestAnimationFrame` — nhưng nó **chưa được chạy trong một bản Tauri đóng gói**, nên vế *"từ lúc thả chuột tới lúc hiển thị"* vẫn là **ước lượng**, không một phép đo. Đánh dấu AC4 đạt trọn.

- ⚠️ **Vế thị giác của story không nghiệm thu trên hai nền tảng thật** — cùng hạng món nợ mà 1.6/1.14/1.16/1.17 để lại, **kế thừa không đóng**: hiệu ứng 90 ms, vạch tiến trình 250 ms, `prefers-reduced-motion`, và điểm dừng `Tab` mới đều mới chỉ chạy qua `vue-tsc` + cổng tĩnh + hai bộ đo engine rời, không qua mắt người trên `tauri dev` (macOS) lẫn một máy Windows.

- 📝 **Phép kiểm AC12 chạy bằng `Selection.modify()`, không bằng một cú KÉO CHUỘT thật.** `modify()` là thuật toán chọn của chính trình duyệt nên nó là bản mô phỏng gần nhất mà một trang tĩnh dựng được, nhưng nó không **là** một lượt kéo. Story 1.16 đo vế chuột bằng Playwright; story này không thêm phụ thuộc nào (NFR15) nên không chạy lại được vế đó. Hai vế cộng lại phủ đủ ý định của AC12, và khoảng trống ghi ở đây thay vì để người sau tự phát hiện.

- 📝 **Trần đường lui `SUBSTRING_FALLBACK_CEILING = 4` chưa có số đo hành vi người dùng đỡ lưng** (`src-tauri/src/commands/dict.rs`). Con số dựng trên một lý lẽ ngôn ngữ (*một thành ngữ tiếng Trung là bốn ký tự — đơn vị dài nhất còn đáng tra như chuỗi con*), không trên nhật ký bôi đen thật. Bench đo được **78/166** truy vấn đi qua đường lui, tức nó không phải một nhánh hiếm. Nhặt lại khi có dữ liệu dùng thật, hoặc ở **Story 7.7** (Concordance — chủ thật sự của `Substring`).

## Deferred from: code review of 1-18-auto-lookup (2026-08-07)

- 📝 **Bộ đếm Kiểm F (`scripts/check-commands.mjs`, AC2) đọc `p.masked`, và `maskScript`/`maskTemplate` chỉ che comment (`//`, `/* */`, `<!-- -->`), KHÔNG che nội dung chuỗi literal/template literal.** Một chuỗi giả dạng lời gọi (vd một dòng văn xuôi/chuỗi lỗi chứa nguyên văn `"useSelectionSurface(original, 'source')"`) vẫn bị đếm là một lượt đăng ký thật, dù không có lời gọi nào. Đây là đặc tính CHUNG của mọi cổng regex trong tệp này (không riêng Kiểm F) — vá đúng nghĩa là đổi hành vi `maskScript`/`maskTemplate` toàn cục, ngoài phạm vi story 1.18. Nhặt lại nếu có một lượt hardening riêng cho `check-commands.mjs`, hoặc nếu một ca thật (không phải giả định) từng lọt qua cổng theo đường này.

- 📝 **`SURFACE_CALL_RE` không khớp dạng gọi thay thế** — gọi trực tiếp `registerSelectionSurface(...)` thay vì qua `useSelectionSurface(...)`, đối số đầu chứa dấu phẩy (vd `pick(a, b)`), hoặc `role` viết sai hoa/thường (`'Source'`) đều không được đếm đúng. Chưa có ca thật nào trong mã hôm nay dùng các dạng đó — mọi panel đều gọi `useSelectionSurface(ref, 'source'|'display')` literal, đúng quy ước. Cùng lớp giới hạn với các cổng regex khác trong tệp (NFR15 cấm phụ thuộc một bộ phân tích cú pháp thật). Nhặt lại nếu một story sau đổi cách gọi.

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
  **Chủ: chưa gán — nợ chung với món "hai nền tảng" của 1.6/1.14/1.16/1.17/1.18/1.18b.**

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
  **Chủ: chưa gán — mở lại khi có một Chương thật vượt 50.000 và người dùng báo giật.**

## Deferred from: 1-19-bat-tat-nguon-tu-dien-va-ghi-cong (2026-08-10)

- 🔴 **Vế DOM của Story 1.19 chưa có bộ chạy test — nghiệm thu bằng BÀN ĐO CHẠY TAY.** Không
  bộ chạy test frontend (NFR15, Ice chốt ở 1.5, giữ qua mười story), nên bốn mệnh đề dưới đây
  **chưa có lưới tự động**: ① dải chip vẽ lại tức thì khi tắt/bật *(AC2)*; ② chip tắt phân biệt
  được bằng mắt mà **không** dùng `opacity` *(UX-DR6 — cài bằng màu + `line-through`, cổng
  `check:tokens` Kiểm D chỉ canh `opacity`, không canh việc nó **có** phân biệt được)*;
  ③ `Escape` đóng lớp phủ và **trả tiêu điểm về chỗ cũ** *(UX-DR17)*; ④ dải chip + bảng
  Attribution duyệt hết được bằng `Tab`. Vế KHAI BÁO thì có cổng: `check:commands` Kiểm A
  *(mọi `@click` là một `dispatch`)* và `SELECTION_SURFACE_FLOOR = 6`.
  **Chủ: chưa gán — nợ chung với món "không bộ chạy test frontend" của 1.16/1.17/1.18/1.18b.**

- 🔴 **`AttributionOverlay.vue` chưa đo trên WKWebView.** Lớp phủ dùng `position: fixed` +
  `inset` + một `z-index` có miễn trừ có tên, và nó nằm **trên** lưới `dockview` — mà dockview
  tự dựng ngữ cảnh xếp lớp riêng cho mỗi nhóm panel. Đo mới chạy trên Chromium.
  **Chủ: chưa gán — nợ chung với món "hai nền tảng" (NFR14) của 1.6/1.14/1.16/1.17/1.18/1.18b.**

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
  **Chủ: chưa gán — mở lại nếu người dùng thật báo trang vơi khi tắt nhiều nguồn.**

- ⚠️ **`max` một lượt đo vượt trần 100 ms — ghi ra chứ không làm tròn xuống.** Cùng bàn đo:
  cấu hình *9/10 nguồn tắt*, **lượt 1**, `max` **150,628 ms** *(lượt 2: 48,856 ms)*. NFR1 phát
  biểu trên **p95**, và p95 của chính cấu hình đó là **3,891 / 2,369 ms** — dưới trần 26–42 lần.
  Đây là nhiễu page-cache của lượt đầu, đúng **Bẫy 8** mà Story 1.18 đã ghi *(1.17 đo p99
  70,742 ms ở lượt đầu và không tái lập được)*. Không kết luận trên một lượt đo.
  **Chủ: chưa gán — theo dõi cùng món "đo NFR1 đầu-cuối gồm vòng IPC" của 1.17.**

- ⚠️ **Số đo NFR1 vẫn là ĐƯỜNG RUST, không đầu-cuối.** Bàn đo mới
  *(`bench_the_source_filter_on_the_real_dictionaries`)* thừa hưởng nguyên giới hạn của 1.17/1.18:
  nó không gồm vòng IPC Tauri lẫn lượt vẽ của webview, **và** không gồm lượt đọc `global.db` mà
  `commands::dict::wire` chạy ở **mỗi** lượt tra để lấy tập bị tắt.
  🔴 Lượt đọc đó là **mới của story này**, nên nó là phần chưa ai đo bao giờ: một `load_global_config`
  cho **mỗi** lượt Auto-Lookup. Nó rẻ *(một `SELECT` trên `config_value` của `global.db`, ba
  loại `GlobalOnly`)*, nhưng *"rẻ"* ở đây là một suy luận, không một số đo.
  **Chủ: chưa gán — đóng cùng món "đo đầu-cuối" của Story 1.17.**

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
  đọc, hoặc một test canh `code` duy nhất trên bốn tệp thật.

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
  nguồn `zh` rồi mở tab Hán Việt.

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
  **Chủ: chưa gán.**

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
  *0 tệp* · *có tệp nhưng k lớp bị bỏ* · *gọi trượt*. **Chủ: chưa gán.**

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
  ghim từ một bản dữ liệu cũ hơn"*. **Chủ: chưa gán.**

- ⚠️ **Số lần tra trên hàng ghim thuộc PHIÊN, không bền vững** (§Dev Notes ⑨ của story).
  Một cột `lookup_count` bền vững đòi một lượt `Store::write` **mỗi lượt tra** — tức mỗi lần
  bôi đen chữ — đưa ghi đĩa vào đúng đường nóng của Auto-Lookup và cho nó cạnh tranh hàng đợi
  ghi nối tiếp với auto-save Editor (NFR2, AD-11/AD-12). Không AC nào đòi nó sống qua phiên.
  Nếu về sau muốn thật: **đo chi phí ghi TRƯỚC**, đừng thêm cột rồi mới đo. **Chủ: chưa gán.**

- ⚠️ **Nhãn thời gian tương đối KHÔNG có đồng hồ riêng.** `relativeTimeKey`/`relativeTimeParams`
  tính lại ở mỗi lượt render, nên *"vừa xong"* chỉ thành *"1 ph"* ở **lượt tra kế tiếp** (hoặc
  một lượt render khác), không tự trôi theo thời gian. Một `setInterval` cho việc này là một
  hẹn giờ chạy suốt phiên chỉ để sửa một nhãn phụ; chưa đáng. **Chủ: chưa gán.**

- ⚠️ **Trần lịch sử 200 hàng là một con số CHƯA ĐO** (`lookupHistoryState.ts::HISTORY_CEILING`).
  AC7 (dedupe) chặn *"hàng trăm dòng giống nhau"*, nó **không** chặn hàng trăm dòng KHÁC nhau
  — một Chương dài có thừa từ khác nhau để tra. 200 là một phỏng đoán có tên, cắt ở ĐUÔI (cũ
  nhất). Nếu người dùng thật chạm trần, con số phải đến từ một lượt đo. **Chủ: chưa gán.**

- ⚠️ **Hàng lịch sử KHÔNG bấm để tra lại được.** Mockup không hứa điều đó, và Kiểm A của
  `check:commands` đòi mọi `@click` là đúng một `dispatch('<id>')` — nên một hàng bấm được
  cần một command thứ năm mang **mục tiêu**, thứ §KHÔNG-LÀM ⑤ và Quyết định #7 đều không cấp.
  Nếu về sau muốn: một `lookup.lookup_history_row` đọc mục tiêu từ `@mousedown` uỷ quyền, đúng
  khuôn `lookup.toggle_pin` — và nó **phải** đi qua `runLookup` chứ không một đường gọi song
  song (Bẫy 3: một đường thứ hai bỏ qua `sequence` là dựng lại đúng lỗi đã vá). **Chủ: chưa gán.**

- ⚠️ **Ba lệch mockup của story này, ghi lại thay vì sửa tài liệu quy hoạch** (Quyết định #3
  của Story 1.3): ① `Concordance` bị loại khỏi dải tab (FR64/Story 7.7, đo được **0** lần
  trong `src/`); ② thanh lọc ba chip bị loại (*"Cả Tác phẩm"* mâu thuẫn AC4); ③ `⌘⌫` bị bác
  cho `lookup.clear_history`. Thêm hai lệch nhỏ phát hiện lúc cài đặt: ④ **cột âm đọc** của
  hàng ghim/lịch sử **không có** — âm Hán Việt là một lượt tra RIÊNG (`read_han_viet`) và
  `pinned_entry` không lưu nó; dựng nó ở đây là một vòng IPC thứ hai cho mỗi hàng;
  ⑤ `pinned_empty_note` viết lại thành một câu đủ (mockup ghi `⌘D khi đang xem một mục từ`,
  mà một hợp âm viết cứng theo nền tảng là đúng thứ Kiểm D/NFR14 tồn tại để chặn).

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
  chọn thay vì một con số. **Chủ: chưa gán.**

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
  hôm nay; ③ chuyển thanh nhịp ra một hàng riêng như dải chip. **Chủ: chưa gán.**

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
  một câu có tên, không hỏng im lặng. **Chủ: chưa gán.**

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
  chế. **Chủ: chưa gán.**

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
  nó cần một sổ đếm tách khỏi danh sách lịch sử. **Chủ: chưa gán.**
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
  **Chủ: chưa gán.**

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
  `check-commands.mjs` cấm `unbound()` xuất hiện trong `src/**/*.vue`. **Chủ: chưa gán** —
  nhặt lại khi bề mặt thứ hai đọc bảng phím xuất hiện (ứng viên: Story 10.4).

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
  xác nhận)*, lúc đó `Escape` mới có chỗ. **Chủ: chưa gán.**

- 📝 **LỆCH MOCKUP — bốn chỗ, ghi ra thay vì dựng theo.** `mockups/settings.html`:
  ① `:243-248` vẽ thanh chuyển phạm vi `Toàn cục`/`Tác phẩm` ⇒ **KHÔNG dựng**, thay bằng một
  câu (`shortcuts.scope_note`); `kinds.rs:29-37` cấm bằng chữ và gọi đích danh story này.
  ② `:251-262` vẽ khung điều hướng chín mục Cài đặt ⇒ **KHÔNG dựng**, chín mục đó thuộc Epic
  4/5/6/10 và trỏ tới năng lực chưa tồn tại. ③ `:291-292` Xuất/Nhập bộ phím tắt ⇒ **KHÔNG
  dựng**, 0 AC và một định dạng trao đổi là một hợp đồng phải bảo trì. ④ `:269` ô tìm kiếm /
  tra ngược hợp âm ⇒ **KHÔNG dựng**, 0 AC. Cả bốn giữ nguyên trong mockup — Quyết định #3 của
  Story 1.3: lệch thì **ghi ra**, không sửa mockup.

- 📝 **Câu `shortcuts.gesture` diễn giải `⌫` bằng CHỮ (*"phím xoá lùi"*), không bằng ký hiệu
  như mockup.** `settings.html:294` viết *"`⌫` để bỏ gán"*. Màn hình thật viết cả câu ra vì
  cử chỉ này có **hai** trạng thái và ký hiệu trần không nói được trạng thái nào — đúng cái
  bẫy mà Bẫy 5 của story mô tả. Không một AC nào đòi ký hiệu; ghi ra để lượt review không đọc
  nó thành một lượt bỏ sót.

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

- 📝 **Số đo của story file khớp thực tế.** Lượt rà soát chạy lại tám cổng, `npm run build`,
  và `cargo test --locked` (**264 xanh · 0 đỏ · 5 ignored** — khớp đúng số Story 1.21 khai).
  Không tìm được một khai sai nào trong các bảng số. Cùng lượt, `tauri build` dựng `.dmg`
  và cả hai cổng `check:scope*` XANH trên runner macOS.

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

- 🔴 **Ba khuyết tật của bộ e2e, nay có chủ là Story 1.22** *(trước lượt này chúng chỉ sống
  trong `proposal-tauri-window-automation-2026-08-11.md` §8, không tạo tác nào chịu trách
  nhiệm)*: ① bộ e2e dùng chung `$APPDATA` với ứng dụng **thật** của người chạy — ca gán phím
  **sửa cấu hình thật của Ice**, và cách dọn hôm nay là bấm nút *"Về mặc định"*, tức vá triệu
  chứng; ② `element.click()` bắn `click` **trước** `focusin` nên mọi tương tác có thứ tự phải
  đi Actions API; ③ máy chủ nhúng bám cổng cố định **4445** nên hai tệp spec cùng lượt làm
  phiên thứ hai trượt. Mục ① phải đóng **trước** khi dựng thêm bất kỳ hàng bàn đo nào.

- ⚠️ **Ba món nợ tài liệu cũ, xác nhận VẪN MỞ trong lượt này** *(không phải phát hiện mới —
  ghi lại để chúng không trôi thêm một epic)*: AD-23 còn liệt kê `$RESOURCE/dict/**` trong
  khi `tauri.conf.json` chỉ khai `$RESOURCE/fonts/**` *(chủ: **Ice**)* · sơ đồ mermaid của
  AD-13 còn cạnh `dict --> matching`, lệch khỏi thân Rule của AD-17 *(chủ: **Winston**)* ·
  bảng phỏng đoán Porter ở AD-44 ③ nay đã có số đo thật từ Story 1.12 mà chưa thay vào
  *(chủ: **Winston**)*.

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
  vẫn xanh (không hồi quy); chín cổng · `npm run build` · `cargo test --locked` đều xanh.

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
  ngày cần: dựng một trang tối giản NGOÀI kho để cô lập hành vi WKWebView.

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

- 📝 **Tài liệu đã dọn ở ba chỗ**, vì lời khuyên cũ nay **tốn tiền của người đọc**: nó bảo
  chạy từng tệp bằng `--spec`, tức bốn lượt khởi động app thay vì một. `e2e/wdio.conf.mjs`
  §Giới hạn 3 + khối `Chạy:` · AC tương ứng của Story 1.22 ở `epics.md`.

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

- `focus.ts::enter()` chỉ chạy lúc **đổi chế độ** *(`WorkspaceMode.vue::onMounted`, `modeState.ts`)*, không chạy ở một cú bấm;
- chốt chống-rơi-về-`body` của nó **chỉ `console.error`**, doc-comment của chính nó ghi *"để KÊU, không để VÁ"*;
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

**Hai ca hồi quy khoá cả hai khuyết tật** (`tests/frontend/editorTypingZone.test.ts`):
- *gõ nhiều ký tự KHÔNG dựng lại text node* — khẳng định **danh tính** của text node, không phải
  chuỗi cuối. Một phép kiểm `textContent === 'abc'` vẫn **xanh dưới bản hỏng**, vì mã test tự gán
  chuỗi đúng; thứ người dùng mất là **caret**, và dấu vết đo được của nó là node bị thay.
  ⚠️ Ca này phải mô phỏng bằng `Text.appendData()`, **không** `textContent = …`: gán `textContent`
  tự nó huỷ node con rồi dựng node mới, nên một ca viết vậy đo chính mã test. Nghiệm thu
  **đỏ-rồi-xanh**: trả template về binding phản ứng ⇒ ca đỏ.
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
  điều thứ hai nữa.

- ⚠️ **KHÔNG di chuyển các phép kiểm HÀNH VI từ cổng tĩnh sang vitest** — `check-layout.mjs`
  Kiểm B *(chạy `simulateWrites`)* và `check-commands.mjs` Kiểm C/D/E *(`import()` thẳng
  `src/commands/*.ts`)* **ở nguyên chỗ**. Đó là một lượt tái cấu trúc có rủi ro riêng: bốn phép
  kiểm đó là lưới **hai nền tảng** duy nhất của tầng bàn phím, và một lượt chuyển làm chúng phụ
  thuộc vào một bộ chạy mới thay vì Node thuần. **Chủ: chưa gán — cần một story riêng.**

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

- Văn bản có **dấu tiếng Việt** trên đường gõ — mọi số của 2.4 đọc ra từ văn bản **ASCII**.
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

- ⚠️ **CHỦ: Story 3.3 (FR48) và Story 7.7 (FR60) — điều kiện khởi hành.** Vai `'display'` của
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

- ⚠️ **`updated_at` của `segment` KHÔNG đổi ở lượt xác nhận.** Cột đó mang nghĩa *"mốc sửa **văn
  bản**"* — nó do `save_segment_targets` sinh, và `SEGMENT_DDL` phân biệt nó với `created_at`
  (*"mốc TẠO, không phải mốc sửa"*). Một lượt ký không sửa một ký tự nào, và thời điểm ký có chỗ
  ghi riêng chính xác hơn: `segment_version.created_at`. ⚠️ Ghi ra vì **Story 2.6 sẽ đọc cả hai
  mốc** và phải biết chúng nói hai chuyện khác nhau. **Chủ: Story 2.6** *(xác nhận lại mệnh đề này
  khi dựng màn hình lịch sử)*.

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
  `editorPanelState.ts` · `segmentNavigation.ts` · `selectionContract.ts`.
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

### 🟡 AC5 vế "ẩn hoàn toàn ở đầu ra" — hai bề mặt tiêu thụ CHƯA TỒN TẠI

**Số đo 2026-08-15 (mở tệp ra đọc, không suy từ tên):**
- `src-tauri/src/core/export/mod.rs` — **6 dòng, toàn bộ là doc-comment, không một dòng mã**.
  `docx-rs` khai ở `Cargo.toml` nhưng `grep docx_rs` trong `src-tauri/src/**/*.rs` chỉ trúng chính
  dòng comment đó. *(Story ghi "7 dòng"; số thật là 6 — đính chính tại chỗ, không đổi kết luận.)*
- `src/modes/ReadingMode.vue` — template chỉ có một `<p>` chở `t('mode.reading.status')`.
  Doc-comment tự ghi *"KHUNG RỖNG có chủ ý… toàn bộ thuộc Epic 5"*; `modeState.ts:30` xác nhận
  *"cả ba chế độ đều rỗng"*.

**Ice ký Quyết định #2 đường (b):** story này dựng **hàm thuần lọc ở Rust** + test hợp đồng khẳng
định câu đã cắt bỏ không xuất hiện. Đó là **cái chốt**, không phải bề mặt.

⇒ Vế còn hở là **hai lượt CẮM VÀO chốt đó**:
- Chế độ đọc → **Epic 5** *(Story 5.11 · 5.12 · 5.13)*
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
