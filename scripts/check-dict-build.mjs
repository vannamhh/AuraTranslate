#!/usr/bin/env node
/**
 * Cổng `tools/dict-build` — Task 7 của Story 1.9. Ba phép kiểm, đúng khuôn doctrine
 * `check-deps.mjs`/`check-i18n.mjs`: script trả mã thoát khác 0 khi thất bại, lỗi hạ
 * tầng KHÔNG được báo thành "đạt", miễn trừ CÓ TÊN và CÓ LÝ DO, in ra mỗi lượt chạy.
 *
 * Kiểm A — từ vựng HỢP NHẤT (AD-19, §Quyết định #4 của Story 1.9). Quét
 *          `tools/dict-build/src/**\/*.rs` tìm token cấm; miễn trừ khai bằng comment
 *          `// dict-build:allow <token> — <lý do>` NGAY DÒNG TRÊN dòng vi phạm.
 * Kiểm B — cách ly workspace (AC4). `tools/dict-build/Cargo.toml` phải có `[workspace]`
 *          và KHÔNG phụ thuộc nào trỏ `path` sang `src-tauri`.
 * Kiểm C — sàn số tệp (bài học `check-deps.mjs:15-17`, `store_boundary.rs:44`): cây
 *          rỗng ⛔ không được đọc thành sạch.
 *
 * Chạy:  npm run check:dict
 */
import { readFileSync, readdirSync, lstatSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join, relative, sep } from 'node:path'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const CRATE_ROOT = join(REPO_ROOT, 'tools', 'dict-build')
const SRC_ROOT = join(CRATE_ROOT, 'src')

let failures = 0
const pass = (m) => console.log(`  \x1b[32mOK\x1b[0m   ${m}`)
const fail = (m) => {
  console.log(`  \x1b[31mFAIL\x1b[0m ${m}`)
  failures += 1
}
const detail = (m) => console.log(`       ${m}`)

function abort(what, err) {
  console.error(`\n\x1b[31mKhông đọc được ${what} — phép kiểm KHÔNG chạy được.\x1b[0m`)
  console.error('Đây là lỗi hạ tầng, không phải "đạt". Đọc lỗi dưới đây rồi chạy lại.\n')
  console.error(String(err?.message || err).trim())
  process.exit(1)
}

const posix = (p) => relative(REPO_ROOT, p).split(sep).join('/')

// ═════════════════════════════════════════════════════════════════════════════════
// Đọc cây `tools/dict-build/src/**/*.rs` — SÀN chống "cây rỗng đọc thành sạch"
// (Kiểm C).
// ═════════════════════════════════════════════════════════════════════════════════
const RS_FILE_FLOOR = 10 // số thật 2026-08-04: 18 tệp .rs dưới src/

function walkRs(dir, out = []) {
  let entries
  try {
    entries = readdirSync(dir)
  } catch (err) {
    abort(`thư mục ${posix(dir)}`, err)
  }
  for (const name of entries) {
    const full = join(dir, name)
    const st = lstatSync(full)
    if (st.isSymbolicLink()) {
      abort(
        `${posix(full)} là symlink`,
        new Error('cổng không quét được nội dung thật của một symlink — thay tệp thật vào chỗ này thay vì liên kết'),
      )
    }
    if (st.isDirectory()) walkRs(full, out)
    else if (name.endsWith('.rs')) out.push(full)
  }
  return out
}

let rsFiles = []
try {
  rsFiles = walkRs(SRC_ROOT)
} catch (err) {
  abort('cây nguồn tools/dict-build/src', err)
}

console.log('\nKiểm C — sàn số tệp (cây rỗng không được đọc thành sạch)')
if (rsFiles.length < RS_FILE_FLOOR) {
  fail(`chỉ ${rsFiles.length} tệp .rs dưới tools/dict-build/src/**, dưới sàn ${RS_FILE_FLOOR}`)
  detail('Cây quá nhỏ để là thật — có khả năng đường quét sai hoặc crate chưa dựng.')
} else {
  pass(`${rsFiles.length} tệp .rs đã quét dưới tools/dict-build/src/** (sàn ${RS_FILE_FLOOR})`)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Kiểm A — từ vựng HỢP NHẤT, hẹp có chủ ý (§Quyết định #4 của Story 1.9)
//
// Bốn tên cuối (`combine_senses` · `or_insert` · `or_insert_with` · `entry(`) là hình
// dạng THẬT của Bẫy 6 (`HashMap::entry(hw).or_insert(sense)` nuốt nguồn thứ hai),
// không phải từ khoá ngữ nghĩa suông. ⛔ Không cấm `distinct`/`DISTINCT` — dùng hợp lệ
// khi sinh `char_idx`.
// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm A — từ vựng hợp nhất (AD-19, Bẫy 6)')

// ⚠️ `entry(` (không chấm) khớp cả `dict_entry(id)` trong DDL và tên hàm hợp lệ như
// `insert_entry(`/`record_entry(` — không phải hình dạng Bẫy 6. Thu hẹp thành
// `.entry(` (có CHẤM ngay trước) — đúng hình dạng THẬT của lỗi:
// `by_headword.entry(hw).or_insert(sense)` là một lời gọi PHƯƠNG THỨC trên map, không
// phải một định danh có chứa "entry".
// So khớp KHÔNG phân biệt hoa/thường (xem lượt so `line`/`token` dưới) — một định danh
// PascalCase (`struct Merger`, `impl Combine for X`) trước đây lọt qua thuần vì so
// khớp có phân biệt hoa/thường. `::entry(` (không chấm, hai dấu hai chấm) bắt dạng gọi
// UFCS `HashMap::entry(&mut map, k)` — né được `.entry(` có chấm nhưng vẫn đúng API bị
// cấm (Review Findings Group C).
const BANNED_TOKENS = [
  'merge',
  'unify',
  'dedup',
  'dedupe',
  'coalesce',
  'consolidate',
  'combine_senses',
  'or_insert',
  'or_insert_with',
  '.entry(',
  '::entry(',
]
const ALLOW_RE = /\/\/\s*dict-build:allow\s+(\S+)\s+—\s+.+/

let exemptCount = 0
const violations = []

for (const file of rsFiles) {
  const text = readFileSync(file, 'utf8')
  const lines = text.split(/\r\n|\n/)
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    // Dòng comment thuần (kể cả chính comment `dict-build:allow` nêu tên token) không
    // phải MÃ — bỏ qua ở đây, chỉ dùng làm nguồn tra miễn trừ cho dòng mã bên dưới.
    if (/^\s*\/\//.test(line)) continue
    const lowerLine = line.toLowerCase()
    for (const token of BANNED_TOKENS) {
      if (!lowerLine.includes(token.toLowerCase())) continue
      // Một dòng vi phạm có thể cần NHIỀU miễn trừ (ví dụ cùng lúc `.entry(` và
      // `or_insert`) — quét NGƯỢC LÊN qua khối comment `//` liên tục ngay phía trên,
      // khớp token với BẤT KỲ dòng nào trong khối đó, không chỉ dòng sát nhất.
      let exempted = false
      for (let j = i - 1; j >= 0 && /^\s*\/\//.test(lines[j]); j--) {
        const allowMatch = lines[j].match(ALLOW_RE)
        if (allowMatch && allowMatch[1] === token) {
          exempted = true
          break
        }
      }
      if (exempted) {
        exemptCount += 1
        continue
      }
      violations.push({ file: posix(file), line: i + 1, token, text: line.trim() })
    }
  }
}

if (violations.length) {
  fail(`${violations.length} lần dùng từ vựng hợp nhất không có miễn trừ hợp lệ:`)
  for (const v of violations) {
    detail(`${v.file}:${v.line} — token '${v.token}' — ${v.text}`)
  }
} else {
  pass(`không có từ vựng hợp nhất nào ngoài miễn trừ hợp lệ (${rsFiles.length} tệp đã quét)`)
}
console.log(`       miễn trừ đã dùng: ${exemptCount}`)

// ═════════════════════════════════════════════════════════════════════════════════
// Kiểm B — cách ly workspace (AC4)
// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm B — cách ly workspace (AC4)')

const cargoTomlPath = join(CRATE_ROOT, 'Cargo.toml')
let cargoToml = ''
try {
  cargoToml = readFileSync(cargoTomlPath, 'utf8')
} catch (err) {
  abort(`${posix(cargoTomlPath)}`, err)
}

const cargoTomlLines = cargoToml.split(/\r\n|\n/)
const workspaceLineIdx = cargoTomlLines.findIndex((l) => /^\[workspace\]\s*$/.test(l))
if (workspaceLineIdx === -1) {
  fail('tools/dict-build/Cargo.toml KHÔNG khai [workspace] — AC4 mất chỗ bám cấu trúc')
} else {
  // Thân `[workspace]` phải RỖNG — `members = [...]` (kể cả trỏ ra ngoài crate này)
  // hút build tool vào một workspace khác, đúng thứ AC4 cấm, mà chỉ kiểm SỰ CÓ MẶT
  // của dòng `[workspace]` không bắt được (Review Findings Group C).
  let bodyEmpty = true
  for (let k = workspaceLineIdx + 1; k < cargoTomlLines.length; k++) {
    const bodyLine = cargoTomlLines[k].trim()
    if (bodyLine === '') continue
    if (/^\[/.test(bodyLine)) break // sang [section] hoặc [[array]] kế tiếp
    if (bodyLine.startsWith('#')) continue
    bodyEmpty = false
    break
  }
  if (bodyEmpty) {
    pass('tools/dict-build/Cargo.toml khai [workspace] rỗng của chính nó')
  } else {
    fail('[workspace] có khoá bên trong (vd. `members`) — không còn RỖNG, hai cây có thể đã giao nhau')
  }
}

const pathToSrcTauri = /path\s*=\s*["'][^"']*src-tauri[^"']*["']/
const workspaceTrue = /workspace\s*=\s*true/
if (pathToSrcTauri.test(cargoToml)) {
  fail('Cargo.toml có phụ thuộc `path =` trỏ sang src-tauri — hai cây đã giao nhau')
} else {
  pass('không phụ thuộc nào trỏ path sang src-tauri')
}
if (workspaceTrue.test(cargoToml)) {
  fail('Cargo.toml có `workspace = true` — dấu hiệu crate này đã bị kéo vào workspace khác')
} else {
  pass('không khai `workspace = true` cho phụ thuộc nào')
}

// ─────────────────────────────────────────────────────────────────────────────────
console.log('')
if (failures !== 0) {
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  process.exit(1)
}
console.log('\x1b[32mTất cả phép kiểm tools/dict-build đạt.\x1b[0m')
process.exit(0)
