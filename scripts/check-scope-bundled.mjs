#!/usr/bin/env node
/**
 * Kiểm 3 NGOÀI chế độ dev — Story 1.3, AC8.
 *
 * `npm run check:scope` chạy qua `tauri dev`, nơi Tauri **không** áp CSP (webview nạp
 * HTML từ Vite qua `devUrl`). Tổ hợp **CSP + asset protocol** vì thế chưa phép kiểm nào
 * chạm tới — đó là mục `deferred-work.md:13` mà story này nhận về. Script này đóng nó.
 *
 * ⚠️ **Vì sao `tauri build --debug` chứ không phải bản release.** Móc self-check phía
 * Rust là `#[cfg(debug_assertions)]` (`src-tauri/src/lib.rs:31,37,55`) nên **không tồn
 * tại trong bản release**. Profile `dev` giữ `debug_assertions` bật ⇒ móc còn đó; nhưng
 * webview vẫn nạp HTML từ `frontendDist` qua asset protocol ⇒ Tauri **có** chèn CSP.
 * Đó đúng là tổ hợp mà `tauri dev` không bao giờ chạm tới.
 *
 * ⛔ Và đó cũng là **giới hạn** của phép kiểm này, ghi thẳng ra đây để không ai đọc quá:
 * nó chứng minh **tổ hợp CSP + asset protocol**. Nó **KHÔNG** chứng minh hành vi của
 * nhị phân profile **release**. Hai đường không được nhập làm một —
 *   ⛔ đừng bật `debug-assertions = true` trong `[profile.release]` để "làm cho đúng
 *      hơn": profile đó đang được cố ý đóng băng để số đo NFR6 của Story 1.1 còn so
 *      sánh được (`Cargo.toml:56-61`), và đổi nó là làm hỏng chính AC6 của story này;
 *   ⛔ đừng gỡ `#[cfg(debug_assertions)]` khỏi móc self-check — Story 1.2 đặt nó ở đó
 *      có lý do đã ghi thành chữ: *"một móc như vậy không có việc gì trong bản phát hành"*.
 *
 * Vì sao là Node chứ không phải bash: `npm run` trên Windows chạy qua `cmd.exe` — không
 * có bash. Cùng lý do với `check-deps.mjs` và `check-scope.mjs` (Ice chốt 2026-08-03).
 *
 * Story 1.3 gắn thẳng script này vào `.github/workflows/ci.yml` — KHÔNG dựng pipeline thứ hai.
 *
 * Chạy:  npm run check:scope:bundled
 */
import { spawn, spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { copyFileSync, mkdirSync, readdirSync, readFileSync, existsSync } from 'node:fs'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const TIMEOUT_MS = Number(process.env.AURA_SCOPE_TIMEOUT_MS ?? 300_000)
const IS_WIN = process.platform === 'win32'

const die = (msg) => {
  console.error(`\n\x1b[31m${msg}\x1b[0m`)
  process.exit(1)
}

// ── Tên nhị phân: đọc từ Cargo.toml, KHÔNG từ `productName` ──────────────────────
// Story 1.2 đã vấp đúng chỗ này: `productName` là "AuraTranslate" nhưng tên tiến trình
// và tên nhị phân lấy từ `package.name` của Cargo — `auratranslate` chữ thường. Công
// thức `pgrep -n AuraTranslate` của bản nháp trả RỖNG vì lý do đó. Đọc, đừng đoán.
const cargoToml = readFileSync(join(REPO_ROOT, 'src-tauri', 'Cargo.toml'), 'utf8')
const binName = cargoToml.match(/^\s*name\s*=\s*"([^"]+)"/m)?.[1]
if (!binName) die('Không đọc được `package.name` từ src-tauri/Cargo.toml')

const conf = JSON.parse(readFileSync(join(REPO_ROOT, 'src-tauri', 'tauri.conf.json'), 'utf8'))
const productName = conf.productName
if (!productName) die('Không đọc được `productName` từ src-tauri/tauri.conf.json')

// ── 1. Dựng bản debug đã đóng gói ────────────────────────────────────────────────
// `--bundles app` trên macOS: `.app` mang sẵn `Contents/Resources/fonts/`, đúng hình
// dạng `bundle.resources` khai. `--no-bundle` ở nơi khác: rẻ hơn, và ta tự chép font.
const buildArgs = IS_WIN
  ? ['tauri', 'build', '--debug', '--no-bundle']
  : ['tauri', 'build', '--debug', '--bundles', 'app']

console.log('Kiểm 3 ngoài chế độ dev — CSP + asset protocol (Story 1.3, AC8)')
console.log(`\n$ npx ${buildArgs.join(' ')}\n`)

const build = spawnSync('npx', buildArgs, {
  cwd: REPO_ROOT,
  // ⚠️ CẢ HAI cờ. `VITE_SCOPE_SELFTEST` phải có LÚC BUILD — `App.vue:14` chỉ `import()`
  // động mã self-check khi cờ này bằng '1', nên thiếu nó là self-check không vào bundle
  // và lượt chạy treo chờ một event không bao giờ tới.
  env: { ...process.env, VITE_SCOPE_SELFTEST: '1' },
  stdio: 'inherit',
  shell: IS_WIN,
})
if (build.status !== 0) die(`\`tauri build --debug\` trả mã ${build.status}.`)

// ── 2. Định vị nhị phân, và bảo đảm nó THẤY được thư mục resource ────────────────
const debugDir = join(REPO_ROOT, 'src-tauri', 'target', 'debug')
const binPath = IS_WIN
  ? join(debugDir, `${binName}.exe`)
  : join(debugDir, 'bundle', 'macos', `${productName}.app`, 'Contents', 'MacOS', binName)

if (!existsSync(binPath)) die(`Không tìm thấy nhị phân đã dựng ở ${binPath}`)

if (IS_WIN) {
  // ⚠️ `--no-bundle` KHÔNG chép resource. Không có bước này thì chiều DƯƠNG trả 404 và
  // lượt chạy tuy FAIL đúng (self-check phân biệt được "thiếu tệp" với "scope chặn")
  // nhưng vẫn vô nghĩa — ta sẽ đi sửa nhầm chỗ. Chép đúng hình dạng `bundle.resources`
  // khai: `resources/fonts/*.{otf,ttf,txt}` → `fonts/`.
  const src = join(REPO_ROOT, 'src-tauri', 'resources', 'fonts')
  const dst = join(debugDir, 'fonts')
  mkdirSync(dst, { recursive: true })
  const copied = readdirSync(src).filter((f) => /\.(otf|ttf|txt)$/i.test(f))
  if (copied.length === 0) die(`Không có tệp font nào ở ${src} để chép — chiều dương sẽ vô nghĩa.`)
  for (const f of copied) copyFileSync(join(src, f), join(dst, f))
  console.log(`\nĐã chép ${copied.length} tệp resource sang ${dst} (cạnh nhị phân).`)
}

// ── 3. Chạy, đọc VERDICT, và TIMEOUT CỨNG ────────────────────────────────────────
// Bài học Story 1.2: một phép kiểm không bao giờ trả gì cũng tệ như một phép kiểm luôn
// trả 0. Nếu webview không mở được trên runner (không phiên đồ hoạ, WebView2 vắng mặt)
// thì `app.exit()` không bao giờ được gọi, và job chạy tới hạn mức rồi bị huỷ.
console.log(`\n$ AURA_SCOPE_SELFTEST=1 ${binPath}\n`)

const child = spawn(binPath, [], {
  cwd: REPO_ROOT,
  env: { ...process.env, AURA_SCOPE_SELFTEST: '1' },
  stdio: ['ignore', 'pipe', 'pipe'],
})

let log = ''
let timedOut = false

const capture = (stream) => {
  stream.setEncoding('utf8')
  stream.on('data', (chunk) => {
    log += chunk
    process.stdout.write(chunk)
  })
}
capture(child.stdout)
capture(child.stderr)

const timer = setTimeout(() => {
  timedOut = true
  child.kill('SIGKILL')
}, TIMEOUT_MS)

child.on('error', (err) => {
  clearTimeout(timer)
  die(`Không chạy được nhị phân: ${err.message}`)
})

child.on('close', () => {
  clearTimeout(timer)
  console.log('')

  if (timedOut) {
    console.log(`\x1b[31mHết ${TIMEOUT_MS / 1000}s mà self-check chưa phát VERDICT — đã giết tiến trình.\x1b[0m`)
    console.log('Treo, không phải đạt. Nhiều khả năng webview không mở được trên máy này.')
    console.log('⛔ Theo AC8: ghi rõ lý do và trả lại cho Ice. Đừng đánh dấu đạt, đừng lặng lẽ bỏ.')
    process.exit(1)
  }

  const verdict = log
    .split('\n')
    .map((l) => l.trim())
    .filter((l) => /^VERDICT: (PASS|FAIL)$/.test(l))
    .at(-1)

  if (!verdict) {
    console.log('\x1b[31mKhông tìm thấy dòng VERDICT trong log.\x1b[0m')
    console.log("Self-check chưa chạy tới nơi. Đọc log ở trên; đừng coi đây là 'đạt'.")
    process.exit(1)
  }

  const mode = log.split('\n').find((l) => l.trim().startsWith('mode:'))?.trim() ?? '(không rõ chế độ)'

  if (verdict === 'VERDICT: PASS') {
    console.log(`\x1b[32mKiểm 3 ngoài chế độ dev: ĐẠT.\x1b[0m  ${mode}`)
    console.log('')
    console.log('⚠️ Đọc đúng phạm vi của kết quả này:')
    console.log('   ✅ CHỨNG MINH: tổ hợp CSP + asset protocol — tài nguyên trong scope nạp được')
    console.log('      dưới CSP, qua `font-src` (đúng đường Story 1.4 sẽ dùng).')
    console.log('   ⛔ KHÔNG chứng minh: hành vi của nhị phân profile RELEASE.')
    console.log('   ⛔ KHÔNG chứng minh: chiều âm dưới CSP — xem dòng [----] ở trên và')
    console.log('      doc-comment của src/selftest/scopeCheck.ts.')
    process.exit(0)
  }

  console.log('\x1b[31mKiểm 3 ngoài chế độ dev THẤT BẠI.\x1b[0m')
  log
    .split('\n')
    .filter((l) => /^\[(PASS|FAIL|----)\]/.test(l.trim()))
    .forEach((l) => console.log(`  ${l.trim()}`))
  process.exit(1)
})
