#!/usr/bin/env node
/**
 * Cổng — cấm tham chiếu `epics.md` kèm số dòng trong mã nguồn.
 *
 * `epics.md` trôi ở mọi lượt sửa; một trích dẫn `epics.md` + dấu hai chấm + chữ số neo
 * vào một số dòng cụ thể, và số đó không còn đúng sau lượt trôi kế tiếp — kể cả trong một
 * tệp `src-tauri/` đã commit từ lâu. Trích dẫn đúng là một mã FR/NFR/AD (ổn định), hoặc bị
 * xoá nếu chỉ là lịch sử không còn giá trị mã.
 *
 * Quét văn bản THÔ, không tách chú thích/chuỗi: hình dạng bị cấm chỉ từng xuất hiện bên
 * trong chú thích hoặc chuỗi (nó không phải cú pháp mã hợp lệ ở bất kỳ ngôn ngữ nào trong
 * cây này), nên việc tách không đổi tập hợp bị bắt — và bỏ tách tránh một tầng phân tích
 * `.rs`/`.ts`/`.vue`/`.mjs` bốn hình dạng khác nhau cho một cổng chỉ cần khớp chuỗi.
 *
 * Chạy:  npm run check:doc-refs
 */
import { lstatSync, readdirSync, readFileSync, realpathSync } from 'node:fs'
import { dirname, join, relative, sep } from 'node:path'
import { fileURLToPath } from 'node:url'
import { judgeFloor } from './lib/floor-judge.mjs'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const posix = (p) => relative(REPO_ROOT, p).split(sep).join('/')

let failures = 0
const pass = (m) => console.log(`  \x1b[32mOK\x1b[0m   ${m}`)
const fail = (m) => {
  console.log(`  \x1b[31mFAIL\x1b[0m ${m}`)
  failures += 1
}
const detail = (m) => console.log(`       ${m}`)

/** Lỗi hạ tầng — KHÔNG phải một phép kiểm đỏ. */
function abort(what, err) {
  console.error(`\n\x1b[31mKhông đọc được ${what} — phép kiểm KHÔNG chạy được.\x1b[0m`)
  console.error(String(err?.message || err).trim())
  process.exit(1)
}

const SKIP_DIRS = new Set(['target', 'node_modules', 'dist', '.git'])
const SCAN_EXT = ['.rs', '.ts', '.tsx', '.vue', '.mjs', '.cjs', '.js', '.md']
const ROOTS = ['src', 'src-tauri/src', 'src-tauri/tests', 'scripts', 'tests', 'e2e']

/**
 * @param {string} dir
 * @param {string[]} out
 * @param {Set<string>} seen
 * @returns {string[]}
 */
/** @type {string[]} */
const skippedLinks = []

function walk(dir, out = [], seen = new Set()) {
  let key
  try {
    key = realpathSync(dir)
  } catch {
    key = dir
  }
  if (seen.has(key)) return out
  seen.add(key)
  let names
  try {
    names = readdirSync(dir)
  } catch (err) {
    abort(posix(dir), err)
  }
  for (const name of names) {
    if (SKIP_DIRS.has(name)) continue
    const full = join(dir, name)
    const st = lstatSync(full)
    if (st.isSymbolicLink()) {
      skippedLinks.push(posix(full))
      continue
    }
    if (st.isDirectory()) walk(full, out, seen)
    else if (SCAN_EXT.some((e) => name.toLowerCase().endsWith(e))) out.push(full)
  }
  return out
}

/**
 * Hình dạng bị cấm: `epics.md` theo sau bởi dấu hai chấm và một hoặc nhiều chữ số — một
 * mốc dòng cụ thể trong `epics.md`. `epics.md` một mình (không neo dòng), hay một mã
 * FR/NFR/AD, không khớp.
 *
 * @param {string} text
 * @returns {{ index: number, match: string }[]}
 */
function violationsIn(text) {
  const out = []
  for (const m of text.matchAll(/\bepics\.md:\d+/g)) out.push({ index: m.index, match: m[0] })
  return out
}

/** @param {string} text @param {number} index @returns {number} dòng 1-based */
function lineOf(text, index) {
  let line = 1
  for (let i = 0; i < index; i += 1) if (text[i] === '\n') line += 1
  return line
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nCổng doc-refs — cấm tham chiếu epics.md kèm số dòng trong mã nguồn')

let files = []
for (const root of ROOTS) {
  const abs = join(REPO_ROOT, root)
  let st
  try {
    st = lstatSync(abs)
  } catch (err) {
    abort(root, err)
  }
  if (!st.isDirectory()) abort(root, new Error('không phải một thư mục'))
  walk(abs, files)
}
files = files.sort()

/** An empty tree must not read as clean; judged by `judgeFloor`. */
const FILE_FLOOR = 369
{
  const v = judgeFloor(FILE_FLOOR, files.length, 'FILE_FLOOR', 'tệp trong tầm quét doc-refs')
  if (!v.ok) abort('quần thể tệp', new Error(v.message))
}

let violationCount = 0
for (const file of files) {
  let text
  try {
    text = readFileSync(file, 'utf8')
  } catch (err) {
    abort(posix(file), err)
  }
  const rel = posix(file)
  for (const v of violationsIn(text)) {
    violationCount += 1
    fail(
      `${rel}:${lineOf(text, v.index)} — \`${v.match}\` neo vào một số dòng của epics.md.\n` +
        `        Đổi thành mã FR/NFR/AD mà dòng đó nói, hoặc xoá nếu chỉ là lịch sử.`,
    )
  }
}
if (violationCount === 0) {
  pass(`0 tham chiếu epics.md kèm số dòng trong ${files.length} tệp quét`)
  if (skippedLinks.length) detail(`symlink bỏ qua: ${skippedLinks.join(', ')}`)
}

// ═════════════════════════════════════════════════════════════════════════════════
// TỰ KIỂM — gọi CHÍNH violationsIn, không một bản chép
// ═════════════════════════════════════════════════════════════════════════════════
{
  const cases = [
    {
      why: 'trích dẫn thật trong một chú thích JS phải bị bắt',
      // Ghép chuỗi rời — nếu viết liền, cổng này sẽ tự bắt chính nó khi quét scripts/**.
      text: '// ' + 'epics.md' + ':1234' + ' nói X',
      wantCount: 1,
    },
    {
      why: 'trích dẫn thật trong một chuỗi Rust phải bị bắt',
      text: '"xem ' + 'epics.md' + ':42' + ' cho ngữ cảnh"',
      wantCount: 1,
    },
    {
      why: 'hai trích dẫn trên hai dòng đều bị bắt, không chỉ dòng đầu',
      text: ['a ' + 'epics.md' + ':1', 'b ' + 'epics.md' + ':2'].join('\n'),
      wantCount: 2,
    },
    {
      why: '`epics.md` một mình (không neo dòng) không bị tính',
      text: 'xem file epics.md để biết thêm',
      wantCount: 0,
    },
    {
      why: 'một mã FR/NFR/AD hợp lệ không bị tính',
      text: '// FR30, NFR15, AD-25 đều hợp lệ',
      wantCount: 0,
    },
  ]
  let wrong = 0
  for (const c of cases) {
    const got = violationsIn(c.text).length
    if (got !== c.wantCount) {
      wrong += 1
      fail(`tự kiểm — ca "${c.why}": bắt được ${got}, mong ${c.wantCount}`)
    }
  }
  if (wrong === 0) pass(`tự kiểm — ${cases.length} ca đúng chiều`)
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('')
if (failures !== 0) {
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  process.exit(1)
}
console.log('\x1b[32mcheck:doc-refs OK.\x1b[0m')
process.exit(0)
