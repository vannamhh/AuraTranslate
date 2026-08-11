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
 *    trên `layout.toggle_source`, và lượt sau đọc nó thành trạng thái đầu rồi ĐỎ với một
 *    câu đổ lỗi cho sản phẩm — một bộ đo tự làm hỏng phép đo của chính nó. Nút *"Về mặc
 *    định"* chỉ vá TRIỆU CHỨNG.
 *    Nay `onPrepare` dựng một thư mục tạm và truyền qua `AURATRANSLATE_E2E_DATA_DIR`;
 *    `onComplete` TỰ KIỂM rằng `global.db` nằm trong đó rồi mới xoá.
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
 * 3. ⚠️ **Một spec = một phiên app, và máy chủ nhúng bám cổng cố định 4445.** Chạy hai
 *    tệp spec trong cùng một lượt làm phiên thứ hai trượt, trong khi mỗi tệp chạy riêng
 *    đều xanh. Chưa đóng; đường ra là cổng theo worker (`TAURI_WEBDRIVER_PORT`) hoặc gộp
 *    các hàng vào ít tệp spec hơn. Tới lúc đó: chạy từng tệp bằng `--spec`.
 *
 * Chạy:  npm run test:e2e            (tất cả spec — xem giới hạn 3)
 *        npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/<tên>.e2e.mjs
 */
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { existsSync, mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { spawn } from 'node:child_process'

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

/** Tên tệp kho toàn cục — khớp `GLOBAL_DB_FILE` ở `src-tauri/src/lib.rs`. */
const GLOBAL_DB_FILE = 'global.db'

/** Thư mục tạm của lượt chạy này. `null` cho tới `onPrepare`. */
let dataDir = null

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

async function devServerIsUp() {
  try {
    const res = await fetch(DEV_URL, { signal: AbortSignal.timeout(1000) })
    return res.ok
  } catch {
    return false
  }
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

    if (await devServerIsUp()) {
      console.log(`[e2e] ${DEV_URL} đã có người phục vụ — dùng lại, KHÔNG dựng thêm.`)
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
      if (await devServerIsUp()) return
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
