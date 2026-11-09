#!/usr/bin/env node
/**
 * Cổng — xoá `core/ai/` cùng ba seam đã duyệt vẫn để phần còn lại của cây biên dịch (FR77, AD-13).
 *
 * Ca thật nằm ở `src-tauri/tests/ai_boundary.rs` và mang `#[ignore]` có chủ ý: nó CHÉP
 * toàn bộ `src-tauri/` sang một thư mục tạm rồi chạy một `cargo check` THỨ HAI trên bản
 * chép đã xoá `core/ai/`. Đó là một phép kiểm dựa-trên-build, đắt hơn hẳn một `assert!`,
 * nên nó sống ngoài đường `cargo test --locked` mặc định — đúng khuôn
 * `check:scope`/`check:scope:bundled`.
 *
 * 🔴 Vì sao cổng này KHÔNG tin mã thoát của `cargo test`: `cargo test … -- --ignored
 * --exact <tên>` khi bộ lọc khớp 0 ca thì in "running 0 tests" và THOÁT 0. Đổi tên hoặc
 * xoá ca đó, hay gỡ `#[ignore]`, làm đối chứng dương duy nhất của FR77 tắt tiếng trong im
 * lặng mà cổng vẫn xanh. Nên cổng đọc log của chính nó và đòi đúng MỘT ca chạy.
 *
 * ⚠️ Chạy SAU `cargo test` trong cả `ci.yml` lẫn `.githooks/pre-push`: nó trỏ `--target-dir`
 * thẳng vào `target/` thật để dùng lại các crate phụ thuộc đã biên dịch, nên một `target/`
 * nguội biến nó từ một bước mười tám giây thành một lượt build đầy đủ. Nó cũng cần `dist/`,
 * do `build.rs` của Tauri nhúng webview — tức `npm run build` phải đi trước.
 *
 * Chạy:  npm run check:fr77
 */
import { spawnSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const MANIFEST = join(REPO_ROOT, 'src-tauri', 'Cargo.toml')
const AI_BOUNDARY = join(REPO_ROOT, 'src-tauri', 'tests', 'ai_boundary.rs')
/** Dòng `cargo test` in khi bộ lọc khớp đúng một ca — vế mà mã thoát không nói được. */
const ONE_CASE_LINE = 'running 1 test'
/** `#[ignore …]` rồi (các attribute khác rồi) `fn <tên>(` — hình dạng ca duy nhất phải tìm. */
const IGNORED_CASE_RE = /#\[ignore[^\]]*\]\s*(?:#\[[^\]]*\]\s*)*fn\s+([a-z0-9_]+)\s*\(/g

let failures = 0
/** @param {string} m */
const pass = (m) => console.log(`  \x1b[32mOK\x1b[0m   ${m}`)
/** @param {string} m */
const fail = (m) => {
  console.log(`  \x1b[31mFAIL\x1b[0m ${m}`)
  failures += 1
}
/** @param {string} m */
const detail = (m) => console.log(`       ${m}`)

/** Lỗi hạ tầng KHÔNG phải một phép kiểm đỏ. Dừng ngay, đừng báo một kết quả không có thật. */
/** @param {string} what @param {unknown} err @returns {never} */
function abort(what, err) {
  console.error(`\n\x1b[31mKhông chạy được ${what} — phép kiểm KHÔNG chạy được.\x1b[0m`)
  console.error('Đây là lỗi hạ tầng, không phải "đạt". Đọc lỗi dưới đây rồi chạy lại.\n')
  console.error(String(/** @type {{ message?: unknown } | null | undefined} */ (err)?.message || err).trim())
  process.exit(1)
}

console.log('')
console.log('\x1b[1mFR77 — xoá core/ai/ + ba seam đã duyệt vẫn để phần còn lại của cây biên dịch\x1b[0m')
console.log('')

// ── Tên ca đọc TỪ NGUỒN, không viết lại ở đây ───────────────────────────────────
// 🔴 Trước lượt này tên ca là một chuỗi trần trong `ci.yml`, và nó đã đứt đúng một lần:
// một lượt đổi tên hàm test để `--exact` khớp 0 ca, mà `cargo test` vẫn thoát 0. Nay tên
// đến từ `ai_boundary.rs` — một nguồn duy nhất, nên một lượt đổi tên không cắt được dây.
let rsText
try {
  rsText = readFileSync(AI_BOUNDARY, 'utf8')
} catch (err) {
  abort('src-tauri/tests/ai_boundary.rs', err)
}
const ignoredCases = [...rsText.matchAll(IGNORED_CASE_RE)].map((m) => m[1])
if (ignoredCases.length !== 1) {
  abort(
    'bộ ca `#[ignore]` của ai_boundary.rs',
    `Tìm thấy ${ignoredCases.length} ca \`#[ignore]\`, cần đúng MỘT.\n` +
      (ignoredCases.length === 0
        ? 'Ca đối chứng dương của FR77 đã bị xoá, hay `#[ignore]` đã bị gỡ?'
        : `Thấy: ${ignoredCases.join(', ')}. Cổng này không tự chọn một trong nhiều ca — ` +
          'đặt ca mới ở tệp khác, hoặc sửa cổng kèm lý do.'),
  )
}
const CASE = ignoredCases[0]

const args = [
  'test',
  '--test',
  'ai_boundary',
  '--locked',
  '--manifest-path',
  MANIFEST,
  '--',
  '--ignored',
  '--exact',
  CASE,
]
const run = spawnSync('cargo', args, { cwd: REPO_ROOT, encoding: 'utf8' })
if (run.error) abort('`cargo test --test ai_boundary`', run.error)
const out = `${run.stdout ?? ''}${run.stderr ?? ''}`

// ── Phép kiểm 1 — ca có THẬT SỰ chạy ────────────────────────────────────────────
const ranOneCase = out.split('\n').some((line) => line.trim() === ONE_CASE_LINE)
if (ranOneCase) {
  pass(`ca \`${CASE}\` đọc từ ai_boundary.rs đã CHẠY ("${ONE_CASE_LINE}")`)
} else {
  fail(`bộ lọc --exact KHÔNG khớp ca nào — log không có dòng "${ONE_CASE_LINE}"`)
  detail(`Ca phải chạy: ${CASE}`)
  detail('Tên ca đã đổi, ca bị xoá, hay `#[ignore]` đã bị gỡ? `cargo test` THOÁT 0 khi bộ')
  detail('lọc khớp 0 ca, nên vắng dòng này là cách duy nhất thấy cổng đang tắt tiếng.')
}

// ── Phép kiểm 2 — và ca đó XANH ─────────────────────────────────────────────────
// 🔴 Treo vào Phép kiểm 1: `status === 0` khi 0 ca chạy KHÔNG nói gì về biên giới, và
// in "OK" ở đó là đánh dấu đạt bằng suy luận — đúng thứ cổng này ra đời để chặn.
if (!ranOneCase) {
  detail('biên giới: KHÔNG đo được lượt này — không ca nào chạy để mà xanh hay đỏ.')
} else if (run.status === 0) {
  pass('bản chép đã xoá core/ai/ + ba seam vẫn biên dịch')
} else {
  fail(`\`cargo test --test ai_boundary\` thoát ${run.status}`)
  detail('Một module ngoài `core/ai/` đã mọc thêm phụ thuộc vào `core/ai/` — đó là AD-13.')
  detail('Seam mới phải được duyệt và ghi vào `ai_boundary.rs` TRƯỚC khi mã vào nhánh.')
}

console.log('')
if (failures !== 0) {
  console.log(out.trimEnd())
  console.log('')
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  process.exit(1)
}
console.log('\x1b[32mcheck:fr77 OK.\x1b[0m')
process.exit(0)
