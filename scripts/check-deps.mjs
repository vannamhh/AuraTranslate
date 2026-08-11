#!/usr/bin/env node
/**
 * Kiểm 1 + Kiểm 2 của Story 1.2 — cưỡng chế AC2 và nửa "không crash reporter,
 * không analytics" của AC5, bằng lệnh, trên CẢ HAI cây phụ thuộc.
 *
 * Vì sao là script chứ không phải soát bằng mắt: hai điều kiện này chỉ đúng vào ngày
 * ai đó nhìn. Một story sau cài `tauri-plugin-fs` "cho tiện" thì không có gì báo.
 *
 * ⚠️ Script này PHẢI trả mã thoát khác 0 khi thất bại. Một script in ra cảnh báo rồi
 * trả 0 là script không cưỡng chế được gì. Ba cách nó từng làm đúng thứ đó, đã sửa:
 *
 *   1. `cargo tree -i <crate>` trả **exit 101** cho CẢ "crate vắng mặt" LẪN "manifest
 *      hỏng / offline / thiếu cargo". Bản bash cũ đọc mọi lỗi hạ tầng thành "vắng
 *      mặt" → sáu dòng OK rồi exit 0. Nay: đọc cây MỘT LẦN, lỗi đọc cây là lỗi cứng.
 *   2. Cây rỗng đọc thành "sạch". `npm ls --all` trên checkout chưa `npm ci` trả đúng
 *      một mục và exit 0 → "cây npm sạch (1 mục)". Nay: có NGƯỠNG SÀN, dưới sàn là
 *      lỗi quét chứ không phải đạt.
 *   3. Mẫu quét chạy trên **đường dẫn tuyệt đối** của `npm ls --parseable`, nên một
 *      repo nằm trong `~/work/analytics/` hay `~/Dropbox/` là FAIL vĩnh viễn. Nay:
 *      chỉ so trên TÊN GÓI, không bao giờ so trên đường dẫn.
 *
 * Vì sao là Node chứ không phải bash (Ice chốt 2026-08-03): AC6 đòi hành vi tương
 * đương hai nền tảng, mà `npm run` trên Windows chạy qua `cmd.exe` — không có bash.
 * Một cổng chỉ canh được một nửa số nền tảng thì không canh được AC6.
 *
 * Story 1.3 gắn thẳng script này vào pipeline — KHÔNG dựng pipeline thứ hai.
 *
 * Chạy:  npm run check:deps
 */
import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const CARGO_MANIFEST = join(REPO_ROOT, 'src-tauri', 'Cargo.toml')

// ⚠️ BẮT BUỘC cho mọi lệnh `npm`/`npx` trên Windows. `npm` ở đó là `npm.cmd`, mà
// `libuv` chỉ dò `.com`/`.exe` khi tìm PATH ⇒ spawn không shell trả **ENOENT**; và từ
// bản vá CVE-2024-27980, Node còn từ chối thẳng việc spawn `.cmd`/`.bat` khi
// `shell: false`. Không có cờ này thì cả cổng phụ thuộc chết ở dòng đầu trên Windows —
// và vì `abort()` gọi `exit 1`, JOB WINDOWS DỪNG TẠI ĐÂY: `cargo test`, AC8 và cả ba
// phép đo `.msi` của AC6 không bao giờ chạy. `check-scope.mjs` và
// `check-scope-bundled.mjs` đã có cờ này từ đầu; tệp này bị sót.
// ⚠️ `cargo` là `cargo.exe` nên KHÔNG cần shell — và không truyền là đúng hơn.
const IS_WIN = process.platform === 'win32'

// Ngưỡng sàn — số thật đo 2026-08-03 là 343 (Rust) và 59 (npm). Sàn đặt thấp hơn hẳn
// để một lần thêm/bớt phụ thuộc bình thường không làm đỏ, nhưng một CÂY RỖNG (chưa
// `npm ci`, cargo không resolve được) thì không thể lọt qua thành "sạch".
const RUST_TREE_FLOOR = 200
const NPM_TREE_FLOOR = 30

let failures = 0
const pass = (m) => console.log(`  \x1b[32mOK\x1b[0m   ${m}`)
const fail = (m) => {
  console.log(`  \x1b[31mFAIL\x1b[0m ${m}`)
  failures += 1
}

/** Lỗi hạ tầng ≠ phép kiểm đỏ. Dừng ngay, đừng báo cáo một kết quả không có thật. */
function abort(what, err) {
  console.error(`\n\x1b[31mKhông đọc được ${what} — phép kiểm KHÔNG chạy được.\x1b[0m`)
  console.error('Đây là lỗi hạ tầng, không phải "đạt". Đọc lỗi dưới đây rồi chạy lại.\n')
  console.error(String(err?.stderr || err?.message || err).trim())
  process.exit(1)
}

// ── Đọc cây Rust MỘT LẦN ─────────────────────────────────────────────────────────
let rustCrates = []
try {
  // ⚠️ `--locked` KHÔNG phải trang trí. `cargo tree` được phép giải lại cây và GHI LẠI
  // `Cargo.lock`. Thiếu cờ này thì (a) cây được quét ở đây có thể không phải cây được
  // `cargo test` biên dịch, và (b) một lượt ghi lại lock làm `cargo test --locked` ở
  // bước SAU đỏ vì một lý do không liên quan tới commit — trong khi comment của
  // `ci.yml` khẳng định `--locked` là nửa Rust của NFR15.
  const out = execFileSync(
    'cargo',
    ['tree', '--locked', '--manifest-path', CARGO_MANIFEST, '--prefix', 'none', '--no-dedupe'],
    { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
  )
  rustCrates = [...new Set(out.split('\n').map((l) => l.trim()).filter(Boolean))]
} catch (err) {
  abort('cây phụ thuộc Rust (`cargo tree`)', err)
}
if (rustCrates.length < RUST_TREE_FLOOR) {
  abort(
    `cây phụ thuộc Rust — chỉ ${rustCrates.length} mục, dưới sàn ${RUST_TREE_FLOOR}`,
    new Error('Cây quá nhỏ để là thật. Nhiều khả năng cargo không resolve được.'),
  )
}
/** Tên crate đứng đầu mỗi dòng `cargo tree --prefix none`: `name vX.Y.Z (path)`. */
const rustNames = new Set(rustCrates.map((l) => l.replace(/^[├└─│\s]+/, '').split(' ')[0]))

// ── Đọc cây npm MỘT LẦN, lấy TÊN GÓI (không lấy đường dẫn) ───────────────────────
let npmNames = new Set()
try {
  // `npm ls` trả exit != 0 khi cây có vấn đề nhưng vẫn in JSON — bắt cả hai đường.
  let raw
  try {
    raw = execFileSync('npm', ['ls', '--all', '--json'], {
      cwd: REPO_ROOT,
      encoding: 'utf8',
      maxBuffer: 64 * 1024 * 1024,
      stdio: ['ignore', 'pipe', 'ignore'],
      shell: IS_WIN, // xem chú thích ở khai báo IS_WIN — thiếu cờ này là ENOENT trên Windows
    })
  } catch (err) {
    raw = err.stdout
    if (!raw) throw err
  }
  const walk = (node) => {
    for (const [name, child] of Object.entries(node?.dependencies ?? {})) {
      npmNames.add(name)
      walk(child)
    }
  }
  walk(JSON.parse(raw))
} catch (err) {
  abort('cây phụ thuộc npm (`npm ls --all --json`)', err)
}
if (npmNames.size < NPM_TREE_FLOOR) {
  abort(
    `cây phụ thuộc npm — chỉ ${npmNames.size} gói, dưới sàn ${NPM_TREE_FLOOR}`,
    new Error('Cây quá nhỏ để là thật. Nhiều khả năng chưa chạy `npm ci`.'),
  )
}

// ─────────────────────────────────────────────────────────────────────────────────
// Kiểm 1 — phụ thuộc đã loại phải VẮNG MẶT trong cây phụ thuộc (AC2)
//
// Ba tên đầu là nguyên văn AC2. `tauri-plugin-fs` là quyết định của Ice 2026-08-03,
// cùng hạng lý do (AD-1 + AD-29): plugin tồn tại để phơi API ra JavaScript, mà
// frontend chỉ render và giữ state UI — nó không có việc gì với hệ thống file.
// ─────────────────────────────────────────────────────────────────────────────────
console.log('\nKiểm 1 — phụ thuộc đã loại phải vắng mặt (AC2)')

const BANNED_CRATES = [
  ['tauri-plugin-stronghold', 'đã khai tử'],
  ['tauri-plugin-keyring', 'AD-29 — dùng crate `keyring` trực tiếp'],
  ['tauri-wire', 'payload 679 byte'],
  ['tauri-plugin-fs', 'AD-1 + AD-29 — Ice chốt 2026-08-03'],
  ['tauri-plugin-sql', 'AD-11 — dùng `rusqlite` trực tiếp'],
  ['tauri-plugin-dialog', 'cùng lý do: không phơi filesystem ra JS'],
]

for (const [crate, why] of BANNED_CRATES) {
  if (rustNames.has(crate)) fail(`crate \`${crate}\` CÓ MẶT trong cây phụ thuộc Rust (${why})`)
  else pass(`crate \`${crate}\` vắng mặt`)
}

// ─────────────────────────────────────────────────────────────────────────────────
// Kiểm 1b — bộ lái WebDriver phải vắng mặt khỏi bộ feature MẶC ĐỊNH (Ice chốt 2026-08-11)
//
// 🔴 Vì sao đây là một cổng chứ không một dòng chú thích trong `Cargo.toml`:
// `tauri-plugin-wdio-webdriver` kéo `axum` + `tokio`, tức một **máy chủ HTTP lắng nghe**
// trên `localhost:4445`. AD-15 đếm đúng BA điểm RA mạng và không khai điểm nào LẮNG
// NGHE — nên một listener trong bản người dùng cài là một bề mặt không có luật nào cho nó.
//
// `#[cfg(debug_assertions)]` KHÔNG đủ: nó loại **mã**, không loại **phụ thuộc**. Thứ duy
// nhất giữ `axum` khỏi nhị phân phát hành là `optional = true` cộng một feature không
// nằm trong `default`. Một lượt sửa `Cargo.toml` thêm `default = ["wdio"]` đi qua trọn
// `cargo test`, trọn `npm run build`, và trọn mười cổng còn lại mà không gì đỏ.
//
// ⚠️ Danh sách KHÔNG gồm `tokio`: đo được nó đã nằm trong cây mặc định từ trước qua
// `tauri` (`tokio v1.53.1`). Canh nó là một cổng đỏ oan ngay lượt chạy đầu.
//
// ⚠️ Phép kiểm này đọc `cargo tree` của bộ feature **mặc định** — cùng lượt gọi mà
// `rustNames` ở trên đã dùng. Chạy `cargo tree --features wdio` thì cả hai tên CÓ mặt,
// và đó là hành vi đúng, không phải một vi phạm.
// ─────────────────────────────────────────────────────────────────────────────────
const DEV_ONLY_CRATES = [
  ['tauri-plugin-wdio-webdriver', 'bộ lái e2e — chỉ sau `--features wdio`'],
  ['axum', 'máy chủ HTTP mà bộ lái e2e kéo theo — AD-15 không khai điểm LẮNG NGHE nào'],
]

for (const [crate, why] of DEV_ONLY_CRATES) {
  if (rustNames.has(crate)) {
    fail(`crate \`${crate}\` có mặt trong cây feature MẶC ĐỊNH (${why})`)
    console.log('       Nhiều khả năng `Cargo.toml` vừa mọc `default = ["wdio"]`, hoặc một')
    console.log('       phụ thuộc khác vừa kéo nó vào. Bộ feature mặc định phải RỖNG — xem')
    console.log('       khối chú thích ở `[dependencies]` của `tauri-plugin-wdio-webdriver`.')
  } else {
    pass(`crate \`${crate}\` vắng mặt khỏi bộ feature mặc định`)
  }
}

// Quét trên TÊN GÓI của cả cây, không chỉ `node_modules` tầng đỉnh: một gói bị cài
// lồng do xung đột phiên bản vẫn phải bắt được.
const BANNED_NPM = [
  '@tauri-apps/plugin-fs',
  '@tauri-apps/plugin-sql',
  '@tauri-apps/plugin-dialog',
  '@tauri-apps/plugin-stronghold',
]

for (const pkg of BANNED_NPM) {
  if (npmNames.has(pkg)) fail(`gói npm \`${pkg}\` CÓ MẶT trong cây phụ thuộc`)
  else pass(`gói npm \`${pkg}\` vắng mặt`)
}

// ─────────────────────────────────────────────────────────────────────────────────
// Kiểm 2 — không crash reporter, không analytics, trên CẢ HAI cây (AC5)
//
// ⚠️ `segment-io` khác `segment`. Module Rust `core/segment/` của chính dự án tên là
// *segment* — mẫu quét phải bắt thư viện thật mà không tự báo động vào chính mình.
// ─────────────────────────────────────────────────────────────────────────────────
console.log('\nKiểm 2 — không crash reporter, không analytics (AC5)')

const PATTERN =
  /sentry|bugsnag|rollbar|crashlytics|datadog|newrelic|posthog|amplitude|mixpanel|segment-io|telemetry|analytics|opentelemetry|google-analytics|firebase/i

const rustHits = [...rustNames].filter((n) => PATTERN.test(n))
if (rustHits.length) {
  fail('cây Rust có thư viện thu thập dữ liệu:')
  rustHits.forEach((h) => console.log(`       ${h}`))
} else {
  pass(`cây Rust sạch (${rustNames.size} crate đã quét)`)
}

const npmHits = [...npmNames].filter((n) => PATTERN.test(n))
if (npmHits.length) {
  fail('cây npm có thư viện thu thập dữ liệu:')
  npmHits.forEach((h) => console.log(`       ${h}`))
} else {
  pass(`cây npm sạch (${npmNames.size} gói đã quét)`)
}

// NFR13 — không tài khoản, không đăng nhập, không đồng bộ đám mây. Cùng cách nghiệm
// thu: bằng VẮNG MẶT. Không SDK auth, không client đồng bộ trong cả hai cây.
const AUTH_PATTERN =
  /auth0|okta|firebase-auth|supabase|clerk|cognito|oauth-client|dropbox|googleapis|onedrive|icloud/i
const authHits = [...rustNames, ...npmNames].filter((n) => AUTH_PATTERN.test(n))
if (authHits.length) {
  fail('có SDK tài khoản / đồng bộ đám mây (NFR13):')
  authHits.forEach((h) => console.log(`       ${h}`))
} else {
  pass('không SDK tài khoản, không client đồng bộ đám mây (NFR13)')
}

// ─────────────────────────────────────────────────────────────────────────────────
console.log('')
if (failures !== 0) {
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  process.exit(1)
}
console.log('\x1b[32mTất cả phép kiểm phụ thuộc đạt.\x1b[0m')
console.log('')
console.log('Ghi chú cho người rà soát: `reqwest` CÓ trong cây phụ thuộc và đó KHÔNG phải')
console.log('vi phạm AC5. Bảng Stack cài trọn ở Story 1.2, nhưng chưa một dòng mã nào gọi tới.')
console.log("AC5 nói 'không có LỜI GỌI ra ngoài nào' — một crate không được gọi thì không gọi")
console.log('đi đâu cả. Ba điểm ra mạng của AD-15 mở ở Story 4.x, 6.7, 10.7.')
process.exit(0)
