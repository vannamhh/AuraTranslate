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
 * Chạy:  npm run test:e2e                                          (cả bộ, ~3 phút)
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
import { existsSync, mkdtempSync, readdirSync, rmSync, statSync } from 'node:fs'
import { homedir, tmpdir } from 'node:os'
import { spawn } from 'node:child_process'
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

/** Thư mục con dưới `~/Documents/` — khớp `DOCUMENTS_SUBFOLDER` ở `commands/project.rs`. */
const DOCUMENTS_SUBFOLDER = 'AuraTranslate'

/** Thư mục tạm của lượt chạy này. `null` cho tới `onPrepare`. */
let dataDir = null

/** Thư mục gốc Library tạm của lượt chạy này. `null` cho tới `onPrepare`. */
let libraryDir = null

/** Dấu vân của thư mục Library THẬT, chụp lúc `onPrepare`. Xem `realLibrarySignature`. */
let realLibraryBefore = null

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
    dataDir = mkdtempSync(join(tmpdir(), 'auratranslate-e2e-'))
    process.env[DATA_DIR_ENV] = dataDir
    console.log(`[e2e] $APPDATA của app con → ${dataDir}`)

    // Bề mặt dữ liệu thật THỨ HAI — xem `LIBRARY_ROOT_ENV`.
    libraryDir = mkdtempSync(join(tmpdir(), 'auratranslate-e2e-library-'))
    process.env[LIBRARY_ROOT_ENV] = libraryDir
    realLibraryBefore = realLibrarySignature()
    console.log(`[e2e] thư mục gốc Library → ${libraryDir}`)

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
   * Tắt Vite, TỰ KIỂM thư mục dữ liệu, rồi mới xoá nó.
   *
   * 🔴 Vì sao phải tự kiểm chứ không chỉ xoá: nếu móc chuyển hướng ngừng có tác dụng —
   * đổi tên biến, quên `--features wdio`, hay một bản `Cargo.toml` bỏ feature — thì app
   * lặng lẽ quay về `$APPDATA` THẬT và **mọi ca vẫn xanh**, vì một kho thật cũng là một
   * kho mở được. Hình dạng hỏng đó không có triệu chứng nào ngoài một thư mục tạm rỗng.
   * Nên thư mục rỗng là một lượt ĐỎ, không phải một chi tiết bỏ qua được.
   *
   * ⚠️ Chỉ khẳng định khi lượt chạy đã xanh (`exitCode === 0`). Một spec đỏ sớm có thể
   * dừng trước khi app kịp tạo kho, và ném thêm một lỗi thứ hai ở đây chỉ che mất lỗi
   * thật đầu tiên.
   */
  onComplete: (exitCode) => {
    if (viteProcess !== null) {
      viteProcess.kill('SIGTERM')
      viteProcess = null
    }

    // ── Hàng rào chiều ÂM: thư mục Library THẬT phải y nguyên ──────────────────────
    if (realLibraryBefore !== null) {
      const after = realLibrarySignature()
      const before = realLibraryBefore
      realLibraryBefore = null
      if (libraryDir !== null) {
        rmSync(libraryDir, { recursive: true, force: true })
        libraryDir = null
      }
      if (after !== before) {
        throw new Error(
          `Thư mục Library THẬT của bạn đã ĐỔI trong lượt e2e này:\n  ${realLibraryPath()}\n` +
            `  trước: ${before}\n  sau:   ${after}\n\n` +
            'Bộ e2e không được chạm vào đó. Nguyên nhân hay gặp:\n' +
            `  1. nhị phân dựng thiếu \`--features wdio\` ⇒ \`${LIBRARY_ROOT_ENV}\` không được đọc;\n` +
            '  2. tên biến ở `src-tauri/src/lib.rs` đã đổi mà tệp này chưa đổi theo;\n' +
            '  3. một đường ghi mới không đi qua `default_library_root()` — đó là một bề\n' +
            '     mặt THỨ BA, và nó cần một móc riêng chứ không một ngoại lệ ở đây.\n\n' +
            'Nếu bạn vừa mở ứng dụng thật song song với lượt chạy này thì đây là báo động\n' +
            'giả — chạy lại khi app đã đóng, đừng gỡ phép kiểm.',
        )
      }
    }

    if (dataDir === null) return

    // ⚠️ KHÔNG nối định danh bundle vào đây. `app_data_dir()` của Tauri là
    // `data_dir()/<identifier>`, nhưng biến môi trường THAY THẾ TRỌN kết quả đó — nên kho
    // nằm thẳng trong `dataDir`. Bản đầu của phép kiểm này nối `com.auratranslate.desktop`
    // vào và ĐỎ ở lượt chạy thật đầu tiên, dù móc chuyển hướng hoạt động đúng: băm của
    // `global.db` thật giống hệt nhau trước và sau lượt chạy.
    const storePath = join(dataDir, GLOBAL_DB_FILE)
    const redirected = existsSync(storePath)
    rmSync(dataDir, { recursive: true, force: true })
    const usedDir = dataDir
    dataDir = null

    if (exitCode === 0 && !redirected) {
      throw new Error(
        `Bộ e2e chạy xanh nhưng KHÔNG thấy ${GLOBAL_DB_FILE} trong ${usedDir}.\n\n` +
          'Nghĩa là app con đã ghi vào `$APPDATA` THẬT của bạn, không vào thư mục tạm —\n' +
          'và một lượt xanh ở đây là một lượt xanh giả. Ba nguyên nhân, theo thứ tự\n' +
          'hay gặp:\n' +
          `  1. nhị phân dựng THIẾU \`--features wdio\` ⇒ \`${DATA_DIR_ENV}\` không được đọc\n` +
          '     (`npm run test:e2e` truyền sẵn; một lượt `cargo build` tay thì không);\n' +
          '  2. tên biến ở `src-tauri/src/lib.rs` đã đổi mà tệp này chưa đổi theo;\n' +
          '  3. `open_global_store` thôi không đi qua `data_dir_override()` nữa.\n\n' +
          'Đừng bỏ phép kiểm này để cho xanh — nó là thứ duy nhất đứng giữa bộ đo và\n' +
          'cấu hình thật của bạn.',
      )
    }
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
