/**
 * Cấu hình bộ lái cửa sổ Tauri THẬT — Ice chốt 2026-08-11.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO THƯ MỤC NÀY TỒN TẠI
 * ═════════════════════════════════════════════════════════════════════════════════
 * Epic 1 để lại 28 hàng bàn đo chạy tay trên hai story (1.20 · 1.21) cộng vế thị giác
 * "kế thừa không đóng" của bảy story khác. Món nợ đó là món DUY NHẤT có hệ số nhân:
 * mọi bản vá tầng DOM đều nằm ngoài tầm của cả mười một cổng — chúng nạp mã bằng Node
 * thuần, không `window`, không DOM — nên mỗi lượt code review chạm DOM lại SINH THÊM
 * hàng bàn đo. Story 1.21 đi từ 12 hàng treo lên 19 SAU khi vá mười phát hiện.
 *
 * 🔴 Và đây là lý do phải là WKWebView chứ không phải Chrome: khuyết tật hạng cao nhất
 * của lượt review Story 1.21 là *"đường chuột của AC2 chết hoàn toàn trên macOS vì
 * WKWebView không đặt tiêu điểm cho `<button>`"*. Một bộ chạy trong Chrome cho ta một
 * bảng xanh và KHÔNG chạm tới đúng lớp lỗi đắt nhất. `driverProvider: 'embedded'` chạy
 * máy chủ WebDriver TRONG chính webview của sản phẩm — WKWebView ở macOS, WebView2 ở
 * Windows.
 *
 * ⚠️ Viết bằng `.mjs`, KHÔNG `.ts`, và đó là một lựa chọn chứ không phải lười:
 * `tsconfig.json` chỉ `include` `src/**`, và `check:lint` chỉ chạy `eslint src`. Một tệp
 * `.ts` ở đây sẽ là TypeScript mà KHÔNG cổng nào type-check — đúng khoản nợ mà
 * `deferred-work.md` đã ghi tên cho `scripts/*.mjs`. Thà cùng hình dạng với `scripts/`
 * còn hơn mọc thêm một bề mặt không ai canh.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * §Giới hạn — ba thứ ĐO ĐƯỢC ở lượt dựng, ghi thẳng thay vì để người sau vấp
 * ═════════════════════════════════════════════════════════════════════════════════
 * 1. ✅ **ĐÃ ĐÓNG 2026-08-11 — `$APPDATA` của app con trỏ sang một thư mục tạm mỗi lượt.**
 *    Trước bản vá: Story 1.21 ghi phím tắt xuống `global.db` (`ScopeKind::Shortcut`), nên
 *    một ca gán phím SỬA cấu hình thật của Ice. Đo được: một lượt chẩn đoán để lại `⌥⌘K`
 *    trên `layout.toggle_source` *(🔵 command đó đổi tên thành `layout.toggle_grid` ở Story
 *    2.5b; bản ghi lịch sử này giữ nguyên tên cũ vì nó kể một lượt chạy đã xảy ra)*, và lượt
 *    sau đọc nó thành trạng thái đầu rồi ĐỎ với một
 *    câu đổ lỗi cho sản phẩm — một bộ đo tự làm hỏng phép đo của chính nó. Nút *"Về mặc
 *    định"* chỉ vá TRIỆU CHỨNG.
 *    Nay `onPrepare` dựng một thư mục tạm và truyền qua `AURATRANSLATE_E2E_DATA_DIR`;
 *    `onComplete` TỰ KIỂM rằng `global.db` nằm trong đó rồi mới xoá.
 *
 *    🔴 **Và `$APPDATA` KHÔNG phải bề mặt duy nhất — bề mặt thứ hai đóng cùng ngày.** Thư
 *    mục gốc Library đi một đường hoàn toàn khác (`document_dir()` ⇒
 *    `~/Documents/AuraTranslate/`, phân giải ở `commands::project::default_library_root`),
 *    nên một bàn đo tạo Tác phẩm sẽ ghi vào Documents THẬT. Đóng bằng
 *    `AURATRANSLATE_E2E_LIBRARY_ROOT`, cộng **hai** hàng rào: `library-root-redirect.e2e.mjs`
 *    đi chiều dương *(`.atproj` phải nằm trong thư mục tạm)*, và `onComplete` đi chiều âm
 *    *(thư mục thật phải y nguyên)*. Bề mặt này tìm ra bằng cách **đọc mã** lúc chuẩn bị
 *    fixture, không bằng cách mất dữ liệu thêm một lần — nên đừng chờ một bề mặt thứ ba tự
 *    lộ ra: mỗi đường ghi mới của sản phẩm là một câu hỏi *"nó rơi vào đâu khi e2e chạy"*.
 *    🔴 Vì sao một biến môi trường đọc trong Rust chứ không phải chỉ đổi `HOME`: đo trên
 *    `dirs-6.0.0`/`dirs-sys-0.5.0` đang ghim — macOS phân giải qua `$HOME`, **Windows đi
 *    Known Folder API và bỏ qua `%APPDATA%`**. Đổi `HOME` là một bản vá chạy trên macOS
 *    và hỏng IM LẶNG trên Windows. Lý do đầy đủ ở doc-comment `E2E_DATA_DIR_ENV` trong
 *    `src-tauri/src/lib.rs`.
 *
 * 2. 🔴 **`element.click()` của driver KHÔNG trung thực về thứ tự sự kiện** — nó bắn
 *    `click` TRƯỚC `focusin`, ngược chuột thật (`mousedown -> focusin -> mouseup ->
 *    click`). Hệ quả đo được: `shortcuts.capture` chạy lúc `aimedRow` còn rỗng và màn
 *    hình trả về *"Chưa nhắm được thao tác nào"*, tức một lượt ĐỎ nói SAI nguyên nhân.
 *    ⇒ Mọi tương tác mà thứ tự sự kiện có nghĩa phải đi qua Actions API
 *    (`browser.action('pointer')…`), xem `realClick()` trong spec.
 *
 * 3. ✅ **HẾT HIỆU LỰC 2026-08-12 — chạy cả bộ trong MỘT lượt được.** Bản ghi cũ nói
 *    *"một spec = một phiên app, máy chủ nhúng bám cổng cố định 4445, chạy hai tệp trong
 *    cùng một lượt làm phiên thứ hai trượt"* và khuyên chạy từng tệp bằng `--spec`. Đo lại
 *    khi bộ có **bốn** spec: **4/4 xanh**, hai lượt liên tiếp, **3m07** và **3m04**.
 *    ⚠️ Nguyên nhân lượt trượt cũ **không được chẩn đoán** — nó biến mất trong lúc C1/C2 đi
 *    qua, và tôi không gán công cho một bản vá nào mà không có phép đo nói thế. Ghi ra để
 *    ai gặp lại triệu chứng đó biết nó **từng** có thật.
 *
 * 🔴 **ĐÍNH CHÍNH 2026-08-12 — bộ này CHẬP CHỜN, và bản ghi trước đó nói "ổn định" trên
 *    một cỡ mẫu quá nhỏ.** Lượt chốt C3 chạy **hai** lượt xanh rồi kết luận ổn định. Tám
 *    lượt tính tới hôm nay: **6 xanh · 2 đỏ**.
 *      - Lần đỏ ①: `shortcuts-capture-mouse` — **đã chẩn đoán và vá**. `cell` lấy TRƯỚC
 *        `resetRowToDefault()`, mà lượt reset dựng lại hàng ⇒ tham chiếu chết ⇒
 *        `"element wasn't found"`. Một lỗi hạ tầng của bàn đo đội lốt hồi quy sản phẩm.
 *      - Lần đỏ ②: `attribution-focus` — **CHƯA chẩn đoán**, nguyên văn lỗi không kịp bắt.
 *        Nó xanh khi chạy một mình và xanh ở mọi lượt cả-bộ khác.
 *    ⚠️ Hai lượt xanh sau bản vá **không** chứng minh bộ đã hết chập chờn — đó đúng là cỡ
 *    mẫu đã lừa một lần. Ai gặp một lượt đỏ không tái lập được: **bắt nguyên văn trước**,
 *    đừng chạy lại cho tới khi xanh rồi đi tiếp.
 *
 * 🔵 **CẬP NHẬT 2026-08-18 (Story 2.12) — bản ghi "8 lượt = 6 xanh · 2 đỏ" ở trên ĐÃ HẾT
 *    ĐÚNG.** Có một lượt trọn bộ **THỨ CHÍN**, ghi ở `deferred-work.md`: **8 passed / 3
 *    failed, 18m51s**. Số thật hôm nay: **9 lượt = 6 xanh · 3 đỏ**.
 *      - `editor-typing-flush` — xanh ở lượt chạy lại.
 *      - `attribution-focus` *(lần đỏ ② ở trên)* — nay có **thêm một vế chẩn đoán, chưa phải
 *        một nguyên nhân**: nó xanh **4/4 khi chạy MỘT MÌNH trên CẢ HAI cây** *(cây story và
 *        baseline `5d94ba1`)*, tức nó đỏ **chỉ trong lô**. Nguyên nhân **vẫn chưa ai đặt tên**.
 *      - `segment-navigation` — đỏ trong lô **trên cả BASELINE** *(before-hook hết 60 s chờ 40
 *        hàng)*. Chạy một mình: **9/10** trên cây story so với **5/5** trên baseline.
 *        🔴 `1/10` so với `0/5` **không phân biệt được hai cây** — không chứng minh có hồi quy,
 *        và cũng không chứng minh không có.
 *
 * 🔵 **VÀ BỐN NGUỒN NHIỄU ĐÃ CÓ BẢN VÁ, 2026-08-18 (Story 2.12 · AC1-AC4):**
 *    `devServerIsUp` nay đi trọn module graph *(`support/devServerHealth.mjs`)* · fixture dọn
 *    state panel bằng cầu `import()` *(`support/panelReset.mjs`)* · khuôn chờ trạng thái đích
 *    *(`support/gridWait.mjs`)* · chờ mốc lưu thay vì `pause(FLUSH_WAIT_MS)`
 *    *(`support/flushWait.mjs`)*.
 *    ⚠️ Câu *"phép đo đó chưa chạy một lượt nào"* của bản ghi này **hết đúng ngày 2026-08-19**
 *    — ba lượt đã chạy, xem ngay dưới. Giữ nguyên câu cũ vì nó đúng **lúc viết**.
 *
 * 🔵 **BA LƯỢT NỮA, 2026-08-18 → 19 (Story 2.12). Tổng: 12 lượt.**
 *
 *    | # | Kết quả | Thời gian | Điều lượt đó DẠY |
 *    |---|---|---|---|
 *    | **10** | 5 passed · **6 failed** | 16m18 | 🔴 Bản vá AC2 làm bộ **XẤU ĐI ba spec** so với mốc 8/3 |
 *    | **11** | 5 passed · **6 failed** | 14m12 | Chẩn đoán sửa cho nói thật ⇒ `Lần đọc cuối: 0` mọi ca |
 *    | **12** | **11 passed · 0 failed** | 13m01 | Sau khi cầu reset soi **cả hai** nửa của `finishSubmit` |
 *
 *    🔴 **Lượt 10 là bài học đắt nhất của story, và nó là một lỗi của DEV, không của hồ sơ.**
 *    Chín lượt `window.location.reload()` bị **cả ba tài liệu** *(hồ sơ story · `deferred-work.md`
 *    · chính tệp này)* mô tả là *"vá của BÀN ĐO cho state cấp module rò"*. **Mô tả đó thiếu một
 *    nửa:** `reload()` dựng lại webview ⇒ chạy lại `main.ts` ⇒ `GridPanel.vue::onMounted` ⇒
 *    `ensureChapterLoaded()`. Nó mang **HAI** vai — dọn state **và** phát một lượt nạp.
 *    Gỡ nó mà chỉ thay vai thứ nhất ⇒ lưới **không bao giờ nạp**, 6 spec đỏ.
 *    ⇒ Đúng khuôn *"chữ ký thi hành đúng MỘT NỬA"* mà retro Epic 2 gọi tên **năm** lần; đây là
 *    lần thứ sáu. Và `libraryImport.ts:173` **đã viết sẵn câu trả lời từ 2026-08-07**:
 *    *"VỨT state cũ là CHƯA ĐỦ — phải NẠP LẠI ngay tại đây."*
 *
 *    🔴 **Lượt 11 dạy một thứ khác, về chính bộ đo:** ba ca đỏ của lượt 10 đều báo *"lần đọc
 *    cuối thấy -1"*, và `-1` **không phải một giá trị đọc được** — nó là giá trị **khởi tạo**.
 *    `timeoutMsg` của `waitUntil` là một **chuỗi dựng lúc tạo object tham số**, nên `${seen}`
 *    bị nội suy **trước** khi vòng chờ chạy. ⇒ Một bộ đo cho một **câu chẩn đoán không có
 *    thật** trên một lượt đỏ thật, và nó đẩy lượt chẩn đoán đi sai hướng ngay câu đầu tiên.
 *    Luật rút ra, nay ghi trong `support/gridWait.mjs`: **mọi con số trong một câu báo lỗi
 *    phải đọc SAU vòng chờ** — dựng câu trong `catch`, không trong tham số.
 *
 * 🔴 **AC7 ĐẠT theo chữ ký #8 của Ice (2026-08-19): `n = 1` lượt, và phải XANH 11/11.**
 *    Lượt 12 thoả: 11/11, 0 đỏ, máy loadavg 4,19 → 3,23 trên 16 nhân.
 *    ⚠️ **VÀ GIỚI HẠN CÓ TÊN của chữ ký ấy, nêu TRƯỚC khi ký và Ice giữ nguyên:** `n=1` đúng
 *    bằng thứ khối *"ĐÍNH CHÍNH 2026-08-12"* ở trên đã đính chính — lượt chốt C3 kết luận
 *    *"ổn định"* trên `n=2` và **sai**. ⇒ Lượt 12 chứng minh bộ **XANH ĐƯỢC**; nó **không**
 *    chứng minh bộ hết chập chờn. Hai mệnh đề khác nhau, và chỉ mệnh đề thứ nhất được mua.
 *    Vế còn lại là một món nợ **có chủ** trong `deferred-work.md`, không một dấu ✅.
 *
 * 🔴 **KHÔNG chạy song song (`maxInstances: 1`), và đó là một quyết định, không một chỗ
 *    chưa làm tới.** Hai lý do, lý do đầu là một hồi quy **đúng theo cấu tạo** chứ không
 *    một rủi ro cần đo:
 *      ① `onPrepare` cấp **một** `$APPDATA` tạm và **một** thư mục Library tạm cho cả lượt.
 *         Hai app chạy song song sẽ dùng chung chúng — đúng trạng thái mà AC2 vừa đóng, chỉ
 *         đổi từ *"e2e đụng dữ liệu người dùng"* thành *"hai ca e2e đụng nhau"*. Muốn song
 *         song thì phải cấp thư mục **theo worker** trước, và phép tự kiểm ở `onComplete`
 *         phải đổi theo.
 *      ② **Mọi** spec trong bộ này khẳng định trên `document.activeElement`. Hai cửa sổ
 *         thật trên cùng một desktop macOS tranh tiêu điểm ở tầng hệ điều hành — một ca có
 *         thể đỏ vì cửa sổ kia vừa được kích hoạt. Đây là rủi ro **chưa đo**, ghi đúng mức
 *         độ chắc chắn của nó; ① một mình đã đủ để không đi đường này hôm nay.
 *    Đổi lại: 3 phút cho cả bộ, tuần tự, và không một lớp đỏ giả nào.
 *
 * 🔴 **SỬA 2026-09-13 — câu "không một lớp đỏ giả nào" ngay trên đã bị chính phép đo LẬT, và
 *    một mệnh đề đã lật không được phép đứng nguyên trong im lặng.** Lý do ① và ② ở trên vẫn
 *    ĐÚNG — cả hai chỉ nói về HAI APP chạy SONG SONG đụng nhau. Chúng không nói gì về, và
 *    không hề loại trừ, một lớp đỏ giả HOÀN TOÀN khác: MỘT app, chạy TUẦN TỰ, mang state cấp
 *    module SỐNG SANG tệp spec kế tiếp theo THỜI GIAN.
 *
 *    Đo 2026-09-12: `npm run test:e2e` — 8 tệp spec, 15 ca đỏ trong lượt trọn bộ, và **12/15
 *    ca đó XANH khi chạy MỘT MÌNH** (`story-5-3-rescan`: 7 đỏ trong lô, 7 xanh một mình,
 *    2,8 s). Không một selector nào đổi, không một hồi quy sản phẩm nào. Nguyên nhân: một
 *    `pid` duy nhất phục vụ cả 24 spec (đo trên hai phiên chạy) — trạng thái Chế độ đang mở,
 *    lượt đọc đang dở, các singleton Library (works/rescan/search) đều là state cấp module
 *    KHÔNG bị dọn giữa hai tệp spec, trừ năm module Panel mà `support/panelReset.mjs` đã đóng
 *    từ Story 2.12.
 *
 *    ⚠️ **MỘT BẢN VÁ ĐÃ ĐƯỢC THỬ VÀ ĐÃ BỊ GỠ — đọc trước khi thử lại đúng đường ấy.** Ngày
 *    2026-09-13 đã dựng: thêm bốn module Chế độ đọc/Library vào `PANEL_MODULES` của
 *    `support/panelReset.mjs`, một lượt chuẩn hoá `setMode('library')`, và một lời gọi
 *    `resetPanelState()` từ `before` hook dưới đây (mỗi tệp spec, không chỉ những spec đi qua
 *    `openWorkspaceWithWork()`). Đo trên máy Ice, cây đứng yên, mỗi lượt một lần:
 *      · mốc `f5feca0`         — 16 tệp xanh /  8 đỏ · 15 ca đỏ · 131 s
 *      · có bản vá             — 12 tệp xanh / 12 đỏ · 20 ca đỏ · 389 s
 *      · bản vá + `LOAD_CALLS` — 12 tệp xanh / 12 đỏ · 20 ca đỏ · 372 s
 *    Vế chuẩn hoá chế độ CÓ chạy đúng (`story-5-6` hết câu *"khối Tác phẩm không có mặt"*),
 *    nhưng bốn tệp đang xanh hoá đỏ và khoảng cách đầy-đủ/lẻ KHÔNG đóng: trên chính cây đã vá,
 *    `story-5-6` và `story-5-11` vẫn XANH khi chạy một mình. Thêm `LOAD_CALLS` cho bốn module
 *    ấy đổi đúng 17 giây và không một ca nào — tức giả thuyết *"dọn mà không nạp lại"* đã bị
 *    bác cho lớp này, dù nó từng đúng cho năm module Panel năm 2026-08-18.
 *    ⇒ Ice chốt 2026-09-13: trả cây về mốc, giữ lại phát hiện. Khuyết tật còn nguyên, có chủ
 *    trong `deferred-work.md`. Đừng dựng lại bản vá TRÊN (dọn state trong CÙNG một app) mà
 *    không có một cơ chế MỚI đo được — mục ngay dưới đây LÀ cơ chế mới đó, và nó đi một
 *    đường khác hẳn: không dọn state, mà GIẾT app cũ và dựng app mới trên thư mục trắng.
 *
 * 🔴 **SỬA 2026-09-14 (`spec-e2e-cach-ly-trang-thai-giua-cac-spec.md`, quyết định 1a-4a của
 *    Ice) — con đường trên (dọn state cấp module) bị BỎ HẲN, không chỉ hoãn.** Ba lớp state
 *    sống sót qua một tệp spec đo được cùng ngày: state module frontend + `<KeepAlive>`,
 *    state Rust trong tiến trình (`OpenWorkState`, chỉ mục Library), và state trên đĩa
 *    trong MỘT `$APPDATA`/gốc Library dùng chung. Dọn state JS (con đường 2026-09-13) không
 *    chạm được hai lớp sau — đúng lý do `story-5-6`/`story-5-11` vẫn xanh một mình trong khi
 *    bản vá đó không đóng khoảng cách đầy-đủ/lẻ.
 *
 *    Cơ chế mới, cắm ở `onWorkerEnd` dưới đây: sau MỖI tệp spec, giết đúng tiến trình app
 *    đang nghe cổng WebDriver nhúng, đợi cổng đóng, rồi cấp một `$APPDATA` và gốc Library
 *    MỚI cho lượt kế — `@wdio/tauri-service@1.3.0` tự hồi sinh app khi thấy cổng chết
 *    (`ensureEmbeddedServersHealthy` → `restartEmbeddedServer` → `startEmbeddedDriver`, env
 *    con dựng bằng `{ ...process.env, … }` NGAY LÚC SPAWN — nên ghi `process.env` trước khi
 *    worker kế mở là đủ, không cần vá hay fork chính tauri-service). Kết quả: mỗi tệp spec
 *    chạy trong một TIẾN TRÌNH MỚI trên hai THƯ MỤC TRẮNG, đúng như khi chạy một mình.
 *
 *    Số đo dựng quyết định (máy Ice, cây đứng yên tại `574c869`, các cấu hình thử là bản
 *    sao NGOÀI kho, không đổi tệp này):
 *      · cấu hình gốc, cả bộ (D)                       — 16 xanh /  8 đỏ ·          · 155 s
 *      · relaunch mỗi tệp spec, DÙNG CHUNG thư mục (C) — 17 xanh /  7 đỏ ·          · 204 s
 *      · relaunch + THƯ MỤC TRẮNG mỗi tệp spec (E)     — 22 xanh /  2 đỏ (đúng G2) · 254 s
 *      · lặp lại E (E2)                                — 21 xanh /  3 đỏ (G2+`story-5-7`) · 280 s
 *    Giết app giải phóng cổng 4445 trong khoảng 1 giây. Relaunch dùng-chung-thư-mục đóng
 *    được state JS/Rust trong tiến trình nhưng KHÔNG đóng lớp đĩa — `story-5-6` vẫn đỏ. Chỉ
 *    thư mục trắng, không relaunch, thì không dọn được state trong tiến trình (`OpenWorkState`,
 *    chỉ mục Library dựng một lần lúc khởi động). Cần CẢ HAI.
 *
 *    ⚠️ **Quyết định 3a — cổng thứ mười bốn (gác reset) bị bỏ, không thay bằng một gác khác
 *    canh chuyện dọn state.** Cơ chế reset không còn tồn tại ở lớp module nữa nên không có
 *    gì để một gác kiểu đó canh; một relaunch mà NGỪNG relaunch thất bại LOUD (`onWorkerEnd`
 *    ném `SevereServiceError` — loại lỗi DUY NHẤT mà `@wdio/cli` không nuốt, xem
 *    `node_modules/@wdio/cli/build/index.js:354-371`), và hàng rào real-Library/`global.db`
 *    ở `onComplete` (quyết định 2a) nay chạy trên MỌI cặp thư mục một app đã dùng, không chỉ
 *    cặp đầu tiên — hai lớp đó cùng thay vai của cổng thứ mười bốn.
 *
 *    ⚠️ **Quyết định 4a — "xong" là BA lượt liên tiếp đúng 22/2 trên cây đứng yên.**
 *    `story-5-4-lifecycle` (2 ca) và `story-5-5-progress` (1 ca) là G2 — đỏ khi chạy MỘT
 *    MÌNH, không phải đỏ giả của bộ; `deferred-work.md` ghi nguyên nhân (nút submit đổi vai
 *    ở Story 6.3) và có chủ. Số đo ba lượt liên tiếp của LƯỢT DỰNG này nằm ở §Verification
 *    cuối tệp spec — đọc ở đó, đừng suy diễn từ bảng số 2026-09-14 phía trên: bảng đó đo
 *    các CẤU HÌNH THỬ ngoài kho, không đo chính tệp này.
 *
 * Chạy:  npm run test:e2e                                          (cả bộ, ~4 phút)
 *        npm run test:e2e -- --spec e2e/specs/<tên>.e2e.mjs        (một tệp, khi đang vá)
 *
 * 🔵 **THÊM 2026-08-20 (lượt rà soát Story 3.3) — BỘ NÀY NAY CÓ MỘT CHỖ CHẠY TỰ ĐỘNG.**
 * Job `e2e` trong `.github/workflows/ci.yml` chạy trọn bộ theo **nhịp đêm** (`cron` 18:00
 * UTC = 01:00 UTC+7) và khi bấm tay, trên `macos-26`. Nó **không** chạy ở `push`, và điều
 * kiện để nâng lên `push` viết thành số ngay trong doc-comment của job đó: mục nợ chập
 * chờn phải ĐÓNG và bảng nightly phải xanh 11/11 mười lượt liên tiếp.
 * ⇒ Bảng nightly là chỗ chuỗi số đo mà §Giới hạn ở trên nói là còn thiếu được tích lại.
 * Đọc nó bằng `gh run list --workflow=CI`, đừng suy từ việc không thấy thông báo nào.
 */
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { existsSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync } from 'node:fs'
import { homedir, tmpdir } from 'node:os'
import { spawn, spawnSync } from 'node:child_process'
import { SevereServiceError } from 'webdriverio'
import {
  crawlModuleGraph,
  describeBrokenGraph,
  describeTruncatedGraph,
  selfCheckDevServerHealth,
} from './support/devServerHealth.mjs'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')

/**
 * Tên biến chỉ `$APPDATA` của app con sang thư mục tạm.
 *
 * 🔴 Phải khớp TỪNG KÝ TỰ với `E2E_DATA_DIR_ENV` ở `src-tauri/src/lib.rs`, và
 * `config_invariants::the_e2e_runner_and_the_rust_side_name_the_same_variable` canh vế
 * đó. Không có cổng ấy thì một lượt đổi tên bên Rust làm móc ngừng có tác dụng, bộ e2e
 * quay lại ghi vào `global.db` THẬT của người chạy, và **mọi ca vẫn xanh** — vì một kho
 * thật cũng là một kho mở được. Đó là hình dạng hỏng tệ nhất có thể ở chỗ này.
 */
const DATA_DIR_ENV = 'AURATRANSLATE_E2E_DATA_DIR'

/**
 * Tên biến chỉ **thư mục gốc Library** sang thư mục tạm.
 *
 * 🔴 Bề mặt dữ liệu thật THỨ HAI, và nó đi một đường hoàn toàn khác `$APPDATA`:
 * `document_dir()` ⇒ `~/Documents/AuraTranslate/`, phân giải ở
 * `commands::project::default_library_root`. Đóng nó **trước** khi tồn tại một bàn đo nào
 * tạo Tác phẩm — bề mặt này tìm ra bằng cách đọc mã, không bằng cách mất dữ liệu thêm một
 * lần nữa.
 */
const LIBRARY_ROOT_ENV = 'AURATRANSLATE_E2E_LIBRARY_ROOT'

/** Tên tệp kho toàn cục — khớp `GLOBAL_DB_FILE` ở `src-tauri/src/lib.rs`. */
const GLOBAL_DB_FILE = 'global.db'

/**
 * Tên tệp chỉ mục Library dẫn xuất — khớp `LIBRARY_INDEX_DB_FILE` ở `src-tauri/src/lib.rs`.
 * Nằm CÙNG thư mục với `GLOBAL_DB_FILE` ở trên (`$APPDATA`, không phải thư mục gốc Library).
 */
const LIBRARY_INDEX_DB_FILE = 'library-index.db'

/** Thư mục con dưới `~/Documents/` — khớp `DOCUMENTS_SUBFOLDER` ở `commands/project.rs`. */
const DOCUMENTS_SUBFOLDER = 'AuraTranslate'

/**
 * Cặp thư mục ĐANG được app con dùng ngay bây giờ. `null` cho tới `onPrepare`.
 *
 * 🔴 **Quyết định 1a — mỗi tệp spec chạy trên một cặp MỚI, không dùng chung một cặp cho cả
 * lượt.** `onPrepare` cấp cặp đầu tiên; mỗi lần `onWorkerEnd` chạy (tức mỗi khi một tệp spec
 * vừa xong), nó giết app đang dùng `currentPair`, đẩy `currentPair` đã xong vào `usedPairs`
 * kèm `exitCode` của worker đó, rồi cấp một `currentPair` MỚI. `onComplete` xử lý cả
 * `usedPairs` (mọi cặp một app THẬT SỰ đã dùng) lẫn `currentPair` còn sót lại lúc lượt chạy
 * kết thúc (cặp cuối, `onWorkerEnd` đã cấp nhưng không tệp spec nào kịp dùng).
 */
let currentPair = null

/**
 * Dấu vân thư mục Library THẬT tại thời điểm `currentPair` BẮT ĐẦU được dùng — chụp lúc
 * `onPrepare` cho cặp đầu, và lúc `onWorkerEnd` (NGAY SAU khi giết app cũ) cho mọi cặp sau.
 * Xem `realLibrarySignature`.
 */
let currentRealLibraryBefore = null

/**
 * Mọi cặp thư mục một app THẬT SỰ đã chạy (spec đã thi hành xong trên đó), theo thứ tự dùng.
 * Mỗi phần tử: `{ dataDir, libraryDir, realLibraryBefore, realLibraryAfter, exitCode }`.
 *
 * 🔴 **Quyết định 2a — `onComplete` canh MỌI phần tử ở đây, không chỉ cặp đầu tiên.** Bản
 * trước-1a chỉ có một cặp cho cả lượt nên một hàng rào canh một lần là đủ; nay một lượt
 * `npm run test:e2e` đi qua tới 24 cặp, và một cặp giữa lô lỡ rò ra Library thật (hay
 * "xanh giả" vào `$APPDATA` thật) sẽ KHÔNG bị bắt nếu hàng rào chỉ nhìn cặp đầu hay cặp cuối.
 */
let usedPairs = []

/** Đường dẫn thư mục Library THẬT của người chạy — thứ lượt e2e KHÔNG được chạm. */
function realLibraryPath() {
  return join(homedir(), 'Documents', DOCUMENTS_SUBFOLDER)
}

/**
 * Dấu vân đủ để phát hiện *"lượt e2e vừa ghi vào thư mục Library thật"*.
 *
 * 🔴 Vì sao cần hàng rào NÀY chứ không chỉ tin biến môi trường: móc `$APPDATA` có một
 * phép tự kiểm dương tính — `global.db` phải NẰM trong thư mục tạm. Móc Library **không
 * có** đối ứng như vậy hôm nay, vì chưa bàn đo nào tạo Tác phẩm, nên thư mục tạm rỗng dù
 * móc chạy đúng hay sai. Một phép kiểm dương tính bịa ra ở đây sẽ luôn xanh và không canh
 * gì.
 *
 * Nên hàng rào đi chiều ÂM: thư mục thật phải **y nguyên**. Nó đúng một cách tầm thường
 * hôm nay, và nó **tự có răng** vào đúng ngày fixture đầu tiên xuất hiện — kể cả khi
 * người viết fixture quên đọc tệp này.
 *
 * ⚠️ Giới hạn: `mtimeMs` của thư mục chỉ đổi khi có mục được thêm hay xoá, nên một lượt
 * ghi ĐÈ lên một `.atproj` sẵn có sẽ lọt. Đóng nốt vế đó cần quét đệ quy cả cây — đắt và
 * chưa cần, vì hôm nay không đường mã nào của bộ e2e mở được một Tác phẩm có sẵn.
 */
function realLibrarySignature() {
  const path = realLibraryPath()
  if (!existsSync(path)) return 'absent'
  const st = statSync(path)
  return `${st.mtimeMs}|${readdirSync(path).length}`
}

/**
 * Cổng của Vite. `tauri.conf.json::build.devUrl` trỏ vào đây và `vite.config.ts` khai
 * `strictPort: true` — hai chỗ phải cùng một số.
 */
const DEV_PORT = 1420
const DEV_URL = `http://localhost:${DEV_PORT}`

/**
 * 🔴 Nhị phân **debug** nạp `devUrl`, KHÔNG nạp `frontendDist` — đo được, không suy đoán:
 * lượt chạy đầu tiên của bàn đo này cho `url: "about:blank"` với `document.body` rỗng, và
 * nó XANH ở mọi khẳng định "không tìm thấy" nếu ca test viết cẩu thả. Một cửa sổ trắng
 * trông giống hệt một ứng dụng chưa kịp render.
 *
 * Nên bộ chạy tự dựng Vite, và tự **tắt** nó. Không có vế tắt thì mỗi lượt e2e để lại
 * một tiến trình giữ cổng 1420, và lượt sau thấy cổng bận rồi tin rằng có người phục vụ.
 */
let viteProcess = null

/**
 * 🔵 **CODE REVIEW 2026-08-19 — ĐỌC CÙNG VỚI CHỮ ĐÃ SỬA CỦA AC1.**
 *
 * Hàm này **cố ý** chỉ trả lời *"có ai đang nghe cổng không"*, và nó **không** phải bản vá của
 * AC1. Câu AC1 bản đầu viết *"`devServerIsUp()` trả `false`"*; chữ ấy đã được sửa tại chỗ ở
 * story *(§Acceptance Criteria, mục 1)* vì vai hẹp này là một **quyết định đo được**: vòng chờ
 * 60 giây dưới kia hỏi **mỗi 500 ms**, nên nó cần một phép hỏi rẻ. Bản vá thật là
 * [`assertModuleGraphHealthy`] — nó **NÉM** và chạy **đúng một lần**.
 * ⇒ Đừng "cải thiện" hàm này bằng cách nhét phép kiểm graph vào đây.
 */
async function devServerIsUp() {
  try {
    const res = await fetch(DEV_URL, { signal: AbortSignal.timeout(1000) })
    return res.ok
  } catch {
    return false
  }
}

/**
 * Một module qua dây, ở hình dạng mà [`crawlModuleGraph`] nhận.
 *
 * ⚠️ **20 giây, không 1 giây.** Đo 2026-08-18: lượt biến đổi đầu tiên của `/src/main.ts`
 * trên một Vite nguội **vượt 3 giây** và trượt timeout — tức một trần chật biến một Vite
 * hoàn toàn lành thành *"hấp hối"*. Đó đúng là chiều đỏ oan, và một phép kiểm đỏ oan sẽ
 * bị nới cho hết đỏ.
 */
async function fetchDevModule(path) {
  const res = await fetch(`${DEV_URL}${path}`, { signal: AbortSignal.timeout(20_000) })
  return {
    status: res.status,
    contentType: res.headers.get('content-type'),
    body: await res.text(),
  }
}

/**
 * 🔴 **AC1 — Vite ĐANG CHẠY không đồng nghĩa app NẠP ĐƯỢC.**
 *
 * [`devServerIsUp`] ngay trên chỉ trả lời *"có ai đang nghe cổng không"*, và nó phải giữ
 * đúng vai hẹp đó: vòng chờ 60 giây dưới kia hỏi **mỗi 500 ms**, nên nó cần một phép hỏi
 * rẻ. Phép kiểm ĐẮT — đi trọn module graph — chạy **đúng một lần**, sau khi đã có người
 * phục vụ, và nó là thứ quyết định bộ có được chạy tiếp hay không.
 *
 * Số đo dựng nên quyết định này nằm ở doc-comment của `support/devServerHealth.mjs`; hai
 * dòng đáng nhắc lại tại chỗ:
 *   · `/` giống nhau **tới từng byte** giữa Vite lành và Vite hấp hối ⇒ `res.ok` không thể
 *     biết gì, và bản cũ chỉ có đúng `res.ok`;
 *   · lượt duyệt tốn **270 ms** (ấm) / **4.129 ms** (nguội) — và khoản nguội là chi phí
 *     **dời chỗ**, không chi phí thêm: nó làm ấm Vite trước khi app mở.
 *
 * @throws {Error} kèm tên module gãy — AC1 vế *"nói ĐÚNG nguyên nhân"*
 */
async function assertModuleGraphHealthy() {
  // Phán quyết phải chứng minh nó đỏ được TRƯỚC khi ai tin một lượt xanh của nó.
  await selfCheckDevServerHealth()

  const started = Date.now()
  const { visited, bad, truncated } = await crawlModuleGraph(fetchDevModule)
  const ms = Date.now() - started

  if (bad.length > 0) throw new Error(describeBrokenGraph(bad, visited.length))
  // 🔵 **CODE REVIEW BA TẦNG 2026-08-19** — vế `truncated` phải được đọc, và đọc SAU `bad`:
  // một graph vừa vỡ vừa bị cắt thì nguyên nhân đáng nói là **vỡ**. Bản đầu không có vế này,
  // nên một lượt duyệt bị cắt cho đúng câu *"module graph lành"* dưới kia — một lời khai về
  // một thứ chưa được kiểm.
  if (truncated) throw new Error(describeTruncatedGraph(visited.length))
  console.log(`[e2e] module graph lành — ${visited.length} module, ${ms} ms.`)
}

/**
 * Nhị phân được lái.
 *
 * 🔴 `debug`, KHÔNG `release`, và hai lớp gác nói cùng một câu: plugin WebDriver đứng
 * sau `#[cfg(all(debug_assertions, feature = "wdio"))]` (`src-tauri/src/lib.rs`) **và**
 * sau một feature không nằm trong `default` (`src-tauri/Cargo.toml`). Một bản `release`
 * KHÔNG có máy chủ nào để nối vào — có chủ ý, và `check-deps.mjs` Kiểm 1b canh vế đó.
 */
const APP_BIN = join(REPO_ROOT, 'src-tauri', 'target', 'debug', 'auratranslate')

if (!existsSync(APP_BIN)) {
  throw new Error(
    `Không thấy nhị phân ${APP_BIN}.\n\n` +
      'Dựng nó trước — và PHẢI có feature `wdio`, nếu không app chạy nhưng không có máy\n' +
      'chủ WebDriver nào để nối vào và lỗi sẽ đội lốt một lượt timeout:\n\n' +
      '  cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml\n\n' +
      '`npm run test:e2e` đã làm việc này cho bạn; lỗi này nghĩa là lượt dựng đó trượt.',
  )
}

/**
 * Cổng máy chủ WebDriver NHÚNG mà `@wdio/tauri-service@1.3.0` chạy trong chính webview.
 *
 * 🔴 Parse GIỐNG HỆT `getEmbeddedPort()` của chính tauri-service
 * (`node_modules/@wdio/tauri-service/dist/esm/index.js:1811-1823`): `parseInt(envPort, 10)`
 * rồi kiểm `Number.isNaN`, KHÔNG `Number(envPort)` — `Number()` đòi TOÀN BỘ chuỗi là số,
 * còn `parseInt` dừng ở ký tự không-số đầu tiên, nên một giá trị dị dạng như `4445abc` cho
 * hai kết quả khác nhau giữa hai cách parse (`parseInt` → 4445, `Number` → `NaN` → rơi về
 * mặc định). Hàm gốc còn có một tầng ƯU TIÊN CAO HƠN — `options.embeddedPort` — nhưng
 * `capabilities` ở tệp này (dưới `export const config`) không truyền `wdio:tauriServiceOptions`
 * nào cả, nên tầng đó không bao giờ khớp ở đây; chỉ còn tầng biến môi trường rồi tới 4445.
 * Đọc sai cổng thì `onWorkerEnd` giết NHẦM cổng — hoặc không giết được gì.
 */
function embeddedPort() {
  const envPort = process.env.TAURI_WEBDRIVER_PORT
  if (envPort) {
    const port = parseInt(envPort, 10)
    if (!Number.isNaN(port)) return port
  }
  return 4445
}

/**
 * Danh sách pid đang NGHE (LISTEN) một cổng TCP, dùng `lsof`.
 *
 * 🔴 **`lsof` thoát mã 1 nghĩa là "không ai nghe"**, KHÔNG phải lỗi — đây là quy ước chuẩn
 * của chính `lsof`, không phải suy đoán. Chỉ mã thoát khác 0/1, hay `spawnSync` không khởi
 * chạy được tiến trình con, mới là lỗi HẠ TẦNG thật.
 *
 * @throws {SevereServiceError} khi `lsof` lỗi hạ tầng — cổng chờ tiến trình chết ở
 *   `waitPortClosed` không được phép đọc nhầm một lỗi hạ tầng thành "cổng vẫn còn bận".
 */
function pidsListeningOnPort(port) {
  const result = spawnSync('lsof', ['-ti', `tcp:${port}`, '-sTCP:LISTEN'], { encoding: 'utf8' })
  if (result.error) {
    throw new SevereServiceError(
      `lsof lỗi hạ tầng khi tìm pid nghe cổng ${port}: ${result.error.message}`,
    )
  }
  if (result.status === 1) return [] // không ai nghe cổng này — quy ước của lsof, không phải lỗi
  if (result.status !== 0) {
    throw new SevereServiceError(
      `lsof thoát mã ${result.status} khi tìm pid nghe cổng ${port} ` +
        `(stderr: ${result.stderr?.trim() || '(rỗng)'}).`,
    )
  }
  return result.stdout
    .split('\n')
    .map((s) => s.trim())
    .filter(Boolean)
    .map(Number)
}

/**
 * Dòng lệnh đầy đủ của một pid, dùng `ps -ww` để KHÔNG bị cắt ở cột hẹp mặc định.
 *
 * @returns {string | null} `null` nếu pid đã biến mất giữa `lsof` và `ps` (cửa sổ đua ở
 *   ranh giới một lượt spec — thấp, không đáng một hàng rào riêng, xem Review Triage Log #3
 *   của spec cách ly trạng thái).
 * @throws {SevereServiceError} cho MỌI lỗi khác của `ps`.
 */
function processCommand(pid) {
  const result = spawnSync('ps', ['-p', String(pid), '-ww', '-o', 'command='], {
    encoding: 'utf8',
  })
  if (result.error) {
    throw new SevereServiceError(`ps lỗi hạ tầng khi đọc lệnh của pid ${pid}: ${result.error.message}`)
  }
  if (result.status !== 0) return null // pid đã thoát giữa lsof và ps — coi như đã dọn xong
  return result.stdout.trim()
}

/**
 * Đợi cổng `port` hết ai nghe, tối đa `timeoutMs`.
 *
 * @throws {SevereServiceError} nếu cổng vẫn còn bị giữ sau `timeoutMs` — `SevereServiceError`
 *   là loại lỗi DUY NHẤT mà `@wdio/cli` không nuốt trong một launcher hook
 *   (`node_modules/@wdio/cli/build/index.js:354-371`), nên đây là cách DUY NHẤT một relaunch
 *   ngừng có tác dụng sẽ dừng cả lượt chạy thay vì để spec kế tiếp lặng lẽ chạy trên app cũ.
 */
async function waitPortClosed(port, timeoutMs) {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    if (pidsListeningOnPort(port).length === 0) return
    await new Promise((r) => setTimeout(r, 200))
  }
  throw new SevereServiceError(
    `Cổng ${port} vẫn còn bị giữ sau ${timeoutMs / 1000} giây — không thể chuyển sang tệp ` +
      'spec kế tiếp một cách an toàn (nó sẽ tranh cổng với app vừa được hồi sinh, hoặc nối ' +
      'nhầm vào app CŨ). Xem `onWorkerEnd` — đây KHÔNG phải một lỗi hạ tầng nên bỏ qua được.',
  )
}

/**
 * Giết đúng tiến trình app con đang nghe `port`, rồi đợi cổng đóng hẳn.
 *
 * 🔴 **Chỉ giết một pid mà cả HAI điều kiện đều đúng: nó đang LISTEN trên `port`, VÀ dòng
 * lệnh của nó khớp `APP_BIN` chính xác hoặc `APP_BIN` theo sau bởi một khoảng trắng** (Review
 * Triage Log #9: `startsWith(APP_BIN)` không biên, một tiến trình tên `APP_BIN` cộng hậu tố
 * sẽ bị giết nhầm). Không có điều kiện thứ hai, `onWorkerEnd` có thể giết bất cứ tiến trình
 * nào (kể cả của người dùng) tình cờ đang nghe đúng cổng đó — vi phạm thẳng §Always của spec:
 * "never a process this run did not launch".
 */
async function killAppOnPort(port) {
  const pids = pidsListeningOnPort(port)
  for (const pid of pids) {
    const command = processCommand(pid)
    if (command === null) continue // đã thoát giữa lsof và ps
    if (command !== APP_BIN && !command.startsWith(`${APP_BIN} `)) continue
    try {
      process.kill(pid, 'SIGTERM')
    } catch (err) {
      if (err.code !== 'ESRCH') {
        throw new SevereServiceError(
          `Không SIGTERM được pid ${pid} (đang nghe cổng ${port}, lệnh: ${command}): ${err.message}`,
        )
      }
      // ESRCH — pid đã tự thoát trước khi kill tới nơi. Không phải lỗi.
    }
  }
  await waitPortClosed(port, 15_000)
}

/**
 * Cấp một `$APPDATA` và một gốc Library MỚI vào `process.env`, ghi đè cặp cũ.
 *
 * 🔴 Đủ chỉ đặt vào `process.env`: `startEmbeddedDriver` của tauri-service dựng env con bằng
 * `{ ...process.env, … }` NGAY LÚC SPAWN (`node_modules/@wdio/tauri-service/dist/esm/index.js:1626`),
 * nên biến mới chỉ có tác dụng cho tiến trình app SAU lần này — đúng lý do `killAppOnPort`
 * phải chạy XONG (cổng đã đóng) trước khi hàm này được gọi, không sau.
 *
 * @returns {{ dataDir: string, libraryDir: string }} cặp thư mục vừa cấp
 */
function allocateFreshPair() {
  const newDataDir = mkdtempSync(join(tmpdir(), 'auratranslate-e2e-'))
  process.env[DATA_DIR_ENV] = newDataDir
  const newLibraryDir = mkdtempSync(join(tmpdir(), 'auratranslate-e2e-library-'))
  process.env[LIBRARY_ROOT_ENV] = newLibraryDir
  return { dataDir: newDataDir, libraryDir: newLibraryDir }
}

export const config = {
  runner: 'local',
  specs: [join(REPO_ROOT, 'e2e', 'specs', '**', '*.e2e.mjs')],
  maxInstances: 1,
  logLevel: 'warn',
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: { ui: 'bdd', timeout: 120_000 },

  /**
   * Dựng Vite nếu chưa có ai phục vụ cổng 1420.
   *
   * ⚠️ Nhánh *"đã có người phục vụ"* tồn tại cho ca Ice đang mở sẵn `npm run tauri dev`
   * — cướp cổng của phiên đó rồi tắt nó ở `onComplete` là làm hỏng việc người khác.
   */
  onPrepare: async () => {
    // ── Thư mục dữ liệu riêng của lượt chạy này ────────────────────────────────────
    //
    // Đặt vào `process.env` là đủ: bộ lái dựng env của app con bằng
    // `{ ...process.env, ...options.env, … }` (`@wdio/tauri-service`,
    // `startEmbeddedDriver`), nên biến này đi thẳng xuống tiến trình con.
    currentPair = allocateFreshPair()
    currentRealLibraryBefore = realLibrarySignature()
    console.log(`[e2e] $APPDATA của app con → ${currentPair.dataDir}`)
    console.log(`[e2e] thư mục gốc Library → ${currentPair.libraryDir}`)

    if (await devServerIsUp()) {
      console.log(`[e2e] ${DEV_URL} đã có người phục vụ — dùng lại, KHÔNG dựng thêm.`)
      // 🔴 Dùng lại KHÔNG có nghĩa là tin. Một Vite của ai đó để mở từ trước là đúng chỗ
      // graph vỡ hay bị bỏ quên nhất — nó không được dựng bởi lượt chạy này nên không ai
      // vừa nhìn thấy nó lành.
      await assertModuleGraphHealthy()
      return
    }
    console.log(`[e2e] dựng Vite ở ${DEV_URL}…`)
    viteProcess = spawn('npm', ['run', 'dev'], {
      cwd: REPO_ROOT,
      stdio: 'ignore',
      detached: false,
    })
    const deadline = Date.now() + 60_000
    while (Date.now() < deadline) {
      if (await devServerIsUp()) {
        await assertModuleGraphHealthy()
        return
      }
      await new Promise((r) => setTimeout(r, 500))
    }
    throw new Error(
      `Vite không lên ở ${DEV_URL} sau 60 giây.\n\n` +
        'Không bỏ qua bước này: nhị phân debug nạp `devUrl`, nên thiếu Vite thì webview\n' +
        'hiện `about:blank` và MỌI ca đợi phần tử sẽ trượt bằng timeout — một lỗi hạ tầng\n' +
        'đội lốt một hồi quy giao diện.',
    )
  },

  /**
   * Giết app con NGAY SAU khi một tệp spec xong, đợi cổng đóng, rồi cấp một `$APPDATA` và
   * một gốc Library MỚI cho tệp spec kế — **Quyết định 1a** của spec cách ly trạng thái.
   *
   * 🔴 `@wdio/cli` chạy hook này qua `runLauncherHook`, hàm NUỐT mọi lỗi trừ
   * `SevereServiceError` (`node_modules/@wdio/cli/build/index.js:354-371`) — một `throw new
   * Error(...)` bình thường ở đây chỉ được LOG rồi lượt chạy tiếp tục như không có gì, tệp
   * spec kế tiếp lặng lẽ chạy trên app CŨ và cặp thư mục CŨ. `killAppOnPort`/`waitPortClosed`
   * tự ném đúng `SevereServiceError`, nhưng `realLibrarySignature()` (`existsSync`/`statSync`/
   * `readdirSync`) và `allocateFreshPair()` (`mkdtempSync`) KHÔNG — cả hai có thể ném lỗi hạ
   * tầng thô (quyền, đĩa đầy, một cuộc đua xoá thư mục). Một lỗi thô ở ĐÚNG chỗ này nguy hiểm
   * hơn ở chỗ khác: nếu nó ném SAU khi cặp vừa xong đã vào `usedPairs` nhưng TRƯỚC khi
   * `currentPair`/`process.env` được ghi đè, `process.env` đứng yên trên cặp ĐÃ ghi nhận là
   * "đã dùng" — app hồi sinh sẽ chạy tiếp trên một cặp thư mục coi như đã nghỉ hưu. ⇒ BỌC
   * TOÀN THÂN hàm trong try/catch, ném lại MỌI lỗi không phải `SevereServiceError` (giữ
   * nguyên thông điệp gốc) — Task 1 đòi "mọi lỗi trong hook này", không chỉ lỗi của hai hàm
   * `kill*`/`wait*`.
   *
   * Thứ tự bắt buộc, Review Triage Log #6: giết App → ĐỢI cổng đóng → CHỤP dấu vân biên
   * (`realLibrarySignature`) → mới cấp cặp mới. Chụp dấu vân TRƯỚC khi giết sẽ gán nhầm một
   * lượt ghi lúc app đang tắt cho cặp KẾ TIẾP.
   */
  onWorkerEnd: async (cid, exitCode) => {
    if (currentPair === null) return // onPrepare chưa từng chạy — không có gì để đóng cặp

    try {
      await killAppOnPort(embeddedPort())

      const finishedPair = currentPair
      const realLibraryAfter = realLibrarySignature()
      usedPairs.push({
        dataDir: finishedPair.dataDir,
        libraryDir: finishedPair.libraryDir,
        realLibraryBefore: currentRealLibraryBefore,
        realLibraryAfter,
        exitCode,
      })

      currentPair = allocateFreshPair()
      currentRealLibraryBefore = realLibraryAfter
      console.log(`[e2e] app hồi sinh — $APPDATA → ${currentPair.dataDir}`)
    } catch (err) {
      if (err instanceof SevereServiceError) throw err
      throw new SevereServiceError(err instanceof Error ? err.message : String(err))
    }
  },

  /**
   * Tắt Vite, rồi chạy MỌI hàng rào dữ liệu — TỰ KIỂM real-Library và `global.db` — trên
   * MỌI cặp thư mục một app đã dùng, không chỉ cặp đầu (**Quyết định 2a**), rồi mới xoá.
   *
   * 🔴 Vì sao phải tự kiểm chứ không chỉ xoá: nếu móc chuyển hướng ngừng có tác dụng —
   * đổi tên biến, quên `--features wdio`, hay một bản `Cargo.toml` bỏ feature — thì app
   * lặng lẽ quay về `$APPDATA` THẬT và **mọi ca vẫn xanh**, vì một kho thật cũng là một
   * kho mở được. Hình dạng hỏng đó không có triệu chứng nào ngoài một thư mục tạm rỗng.
   * Nên thư mục rỗng là một lượt ĐỎ, không phải một chi tiết bỏ qua được.
   *
   * ⚠️ Phép kiểm `global.db` chỉ khẳng định trên một cặp khi WORKER CỦA CHÍNH CẶP ĐÓ đã
   * xanh (`pair.exitCode === 0`), không theo `exitCode` của cả lượt chạy — bộ này CÓ THỂ đỏ
   * cấu trúc (G2, `story-5-4-lifecycle`/`story-5-5-progress`) trong khi từng cặp riêng lẻ
   * vẫn ghi đúng chỗ; gác theo exitCode của cả lượt sẽ khiến phép kiểm này KHÔNG BAO GIỜ
   * chạy trong khi G2 còn mở (Review Triage Log #1).
   *
   * ⚠️ Vòng lặp KHÔNG dừng ở lỗi đầu tiên: gom hết thất bại, xoá hết thư mục của MỌI cặp
   * (dùng hay không), rồi mới ném một lỗi duy nhất nêu tên từng cặp lỗi — dừng sớm sẽ bỏ
   * qua các cặp sau và làm rò thư mục của chúng (Review Triage Log #7).
   */
  onComplete: () => {
    if (viteProcess !== null) {
      viteProcess.kill('SIGTERM')
      viteProcess = null
    }

    // Cặp CUỐI CÙNG: `onWorkerEnd` đã cấp nó nhưng lượt chạy kết thúc trước khi một tệp spec
    // nào kịp dùng nó (hoặc — lượt chạy có đúng MỘT tệp spec — nó là cặp `onPrepare` cấp và
    // `onWorkerEnd` của tệp đó đã đóng nó vào `usedPairs` rồi; trường hợp đó `currentPair`
    // vẫn khác `null` vì `onWorkerEnd` luôn cấp cặp KẾ TIẾP). Không worker nào chạy trên nó
    // nên không có `exitCode` — hàng rào real-Library vẫn chạy, hàng rào `global.db` bỏ qua.
    if (currentPair !== null) {
      usedPairs.push({
        dataDir: currentPair.dataDir,
        libraryDir: currentPair.libraryDir,
        realLibraryBefore: currentRealLibraryBefore,
        realLibraryAfter: realLibrarySignature(),
        exitCode: undefined,
      })
      currentPair = null
    }

    const failures = []

    for (const pair of usedPairs) {
      // ── Hàng rào chiều ÂM: thư mục Library THẬT phải y nguyên ────────────────────
      if (pair.realLibraryBefore !== pair.realLibraryAfter) {
        failures.push(
          `Thư mục Library THẬT của bạn đã ĐỔI trong lượt e2e này, khi app chạy trên cặp\n` +
            `  ${pair.dataDir}\n  ${pair.libraryDir}\n` +
            `Đường dẫn thật: ${realLibraryPath()}\n` +
            `  trước: ${pair.realLibraryBefore}\n  sau:   ${pair.realLibraryAfter}\n\n` +
            'Bộ e2e không được chạm vào đó. Nguyên nhân hay gặp:\n' +
            `  1. nhị phân dựng thiếu \`--features wdio\` ⇒ \`${LIBRARY_ROOT_ENV}\` không được đọc;\n` +
            '  2. tên biến ở `src-tauri/src/lib.rs` đã đổi mà tệp này chưa đổi theo;\n' +
            '  3. một đường ghi mới không đi qua `default_library_root()` — đó là một bề\n' +
            '     mặt THỨ BA, và nó cần một móc riêng chứ không một ngoại lệ ở đây.\n\n' +
            'Nếu bạn vừa mở ứng dụng thật song song với lượt chạy này thì đây là báo động\n' +
            'giả — chạy lại khi app đã đóng, đừng gỡ phép kiểm.',
        )
      }

      // ── Hàng rào chiều ĐỌC: `library-index.db` không được nhắc đường dẫn Library THẬT ──
      //
      // 🔴 PHÁN QUYẾT Ice 2026-08-27 — hàng rào ÂM ở trên (`realLibrarySignature`) chỉ canh
      // chiều GHI (thư mục thật có mọc/mất mục hay không); nó KHÔNG canh chiều ĐỌC. Một lượt
      // chạy đã lọt qua nó trong khi vẫn ĐỌC `~/Documents/AuraTranslate` thật và lập chỉ mục
      // các Tác phẩm ở đó (xem mục nợ "Một lượt e2e ĐỎ chưa chẩn đoán được",
      // `deferred-work.md`) — dấu vết mà hàng rào GHI không để lại, vì không byte nào bị ghi
      // vào chính thư mục thật đó.
      //
      // Hàng rào DƯƠNG ở đây: đọc `library-index.db` (nằm trong `$APPDATA` tạm, CÙNG thư mục
      // với `global.db`) DẠNG BYTE — không phân tích SQLite, không thêm phụ thuộc npm
      // (`scripts/AGENTS.md`) — và ghi nhận thất bại nếu nội dung chứa chuỗi con đúng đường
      // dẫn Library THẬT. SQLite lưu một cột `TEXT` dưới dạng UTF-8 thô ngay trong trang dữ
      // liệu của tệp `.db`, nên một chuỗi con khớp byte-cho-byte là bằng chứng THẬT, không
      // suy luận — đúng cách `atproj_path`/`library_orphan.atproj_path` (phán quyết Ice #1)
      // sẽ mang nguyên văn đường dẫn nếu ứng dụng lỡ lập chỉ mục thư viện thật.
      //
      // ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì giấu:** hàng rào này chỉ bắt được đường dẫn ĐÃ ĐI
      // VÀO chỉ mục. Một lượt chỉ ĐỌC thư mục thật mà không lập chỉ mục được gì vẫn LỌT qua
      // đây — hàng rào canh DẤU VẾT còn lại trên đĩa, không canh hành vi ĐỌC tại đúng thời
      // điểm nó xảy ra.
      const indexPath = join(pair.dataDir, LIBRARY_INDEX_DB_FILE)
      let indexBytes = null
      try {
        indexBytes = readFileSync(indexPath)
      } catch (err) {
        if (err.code === 'ENOENT') {
          // Chưa từng mở/lập chỉ mục trên cặp này -- KHÔNG phải lỗi, bỏ qua êm.
          indexBytes = null
        } else {
          // Lỗi HẠ TẦNG (quyền, đĩa hỏng, …) -- KHÔNG phải một phép kiểm ĐỎ.
          console.warn(
            `[e2e] không đọc được ${indexPath} để kiểm hàng rào chiều ĐỌC (${err.code}) -- ` +
              'bỏ qua phép kiểm này, đây là lỗi HẠ TẦNG, không phải một phát hiện.',
          )
          indexBytes = null
        }
      }
      if (indexBytes !== null) {
        const needle = Buffer.from(realLibraryPath(), 'utf8')
        if (indexBytes.includes(needle)) {
          failures.push(
            `${indexPath} (${LIBRARY_INDEX_DB_FILE}) chứa đường dẫn Library THẬT của bạn:\n` +
              `  ${realLibraryPath()}\n\n` +
              'Nghĩa là ứng dụng đã ĐỌC và lập chỉ mục thư viện thật trong lượt e2e này, dù\n' +
              'không byte nào bị GHI vào thư mục đó (hàng rào chữ ký ở trên không bắt được\n' +
              'chiều này). Đây chính là dấu vết của "Một lượt e2e ĐỎ chưa chẩn đoán được"\n' +
              '(`deferred-work.md`) — đọc mục đó trước khi sửa bất cứ dòng nào.',
          )
        }
      }

      // ── Hàng rào chiều DƯƠNG: `global.db` phải NẰM trong `$APPDATA` tạm ───────────
      //
      // ⚠️ KHÔNG nối định danh bundle vào đây. `app_data_dir()` của Tauri là
      // `data_dir()/<identifier>`, nhưng biến môi trường THAY THẾ TRỌN kết quả đó — nên kho
      // nằm thẳng trong `dataDir`. Chỉ khẳng định khi WORKER CỦA CHÍNH CẶP NÀY đã xanh — một
      // spec đỏ sớm có thể dừng trước khi app kịp tạo kho.
      if (pair.exitCode === 0) {
        const storePath = join(pair.dataDir, GLOBAL_DB_FILE)
        if (!existsSync(storePath)) {
          failures.push(
            `Tệp spec chạy trên cặp ${pair.dataDir} xanh nhưng KHÔNG thấy ${GLOBAL_DB_FILE}\n` +
              'trong đó.\n\n' +
              'Phần lớn nguyên nhân nghĩa là app con đã ghi vào `$APPDATA` THẬT của bạn, không\n' +
              'vào thư mục tạm — một lượt xanh ở đây là một lượt xanh giả. Theo thứ tự hay gặp:\n' +
              `  1. nhị phân dựng THIẾU \`--features wdio\` ⇒ \`${DATA_DIR_ENV}\` không được đọc\n` +
              '     (`npm run test:e2e` truyền sẵn; một lượt `cargo build` tay thì không);\n' +
              '  2. tên biến ở `src-tauri/src/lib.rs` đã đổi mà tệp này chưa đổi theo;\n' +
              '  3. `open_global_store` thôi không đi qua `data_dir_override()` nữa;\n' +
              '  4. `onWorkerEnd` không kịp giết app cũ trước khi ghi biến môi trường mới,\n' +
              '     nên tệp spec này chạy trên `$APPDATA` của cặp TRƯỚC;\n' +
              '  5. KHÔNG kho nào được mở cả — `open_global_store` (`src-tauri/src/lib.rs:899-\n' +
              '     940`) chỉ `eprintln!` rồi `return` khi `create_dir_all` trên thư mục tạm\n' +
              '     hay `Store::open` thất bại, nên app tiếp tục chạy KHÔNG store, không panic,\n' +
              '     không rơi vào nhánh dữ liệu thật; xem log stderr của app cho dòng\n' +
              '     `store[global] …`.\n\n' +
              'Đừng bỏ phép kiểm này để cho xanh — nó là thứ duy nhất đứng giữa bộ đo và\n' +
              'cấu hình thật của bạn.',
          )
        }
      }

      // Xoá dù cặp này có lỗi hay không — một cặp lỗi không được phép rò thư mục
      // (Review Triage Log #7).
      rmSync(pair.dataDir, { recursive: true, force: true })
      rmSync(pair.libraryDir, { recursive: true, force: true })
    }

    usedPairs = []

    if (failures.length > 0) {
      throw new Error(
        `${failures.length} cặp thư mục thất bại hàng rào dữ liệu thật:\n\n` +
          failures.join('\n\n═══════════════════════════════════════\n\n'),
      )
    }
  },

  before: async () => {
    // Cầu nối __wdio_original_core__ cho @wdio/tauri-service: tránh mỗi lệnh WebDriver bị delay timeout 5 giây
    await browser.execute(() => {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        window.__wdio_original_core__ = {
          invoke: window.__TAURI_INTERNALS__.invoke.bind(window.__TAURI_INTERNALS__),
        }
      }
    })
  },

  services: ['@wdio/tauri-service'],
  capabilities: [
    {
      browserName: 'tauri',
      'tauri:options': {
        application: APP_BIN,
        // `embedded` là mặc định và là đường DUY NHẤT chạy được trên macOS: Apple không
        // cung cấp WebDriver cho WKWebView, và `tauri-driver` chính thức vẫn Windows +
        // Linux (issue `tauri-apps/tauri#7068`, mở từ 2023).
        driverProvider: 'embedded',
      },
    },
  ],
}
