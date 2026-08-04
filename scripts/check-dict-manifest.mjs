#!/usr/bin/env node
/**
 * Cổng `dict-manifest.toml` — Task 8 của Story 1.9 (AC3, đóng `deferred-work.md:79`:
 * *"đặt ra luật ba trường rồi không cưỡng chế bằng gì cả"*).
 *
 * 🔴 Cổng này KHÔNG đọc tệp `.db` và KHÔNG tải gì từ mạng — nó phải xanh trên một
 * runner CI không có byte dữ liệu từ điển nào (AC cuối của Story 1.3: CI ⛔ không tải
 * dữ liệu từ điển). Nó kiểm HÌNH DẠNG manifest, không kiểm nội dung tệp.
 *
 * Parser TOML ở đây là TẬP CON NGHIÊM NGẶT, tự viết — tiền lệ `parseCssBlocks` của
 * `check-tokens.mjs`: đủ cho ĐÚNG những gì `dict-manifest.toml` cần, và không hơn.
 * Cú pháp NGOÀI tập con (mảng, bảng inline, chuỗi nhiều dòng, số, boolean, khoá không
 * nháy) ⇒ FAIL, ⛔ không bỏ qua. NFR15 đòi rà GPLv3 + vào bảng Stack trước khi thêm một
 * phụ thuộc npm; một tệp 40 dòng không đáng một lượt rà.
 *
 * Chạy:  npm run check:dict-manifest
 */
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const MANIFEST_PATH = join(REPO_ROOT, 'dict-manifest.toml')

let failures = 0
const pass = (m) => console.log(`  \x1b[32mOK\x1b[0m   ${m}`)
const fail = (m) => {
  console.log(`  \x1b[31mFAIL\x1b[0m ${m}`)
  failures += 1
}

// ═════════════════════════════════════════════════════════════════════════════════
// Parser TOML — TẬP CON NGHIÊM NGẶT:
//   - dòng trống, dòng `# comment` (bắt đầu bằng #, sau khi trim)
//   - `[section]`
//   - `[[array_of_tables]]`
//   - `key = "chuỗi nháy kép"` (không escape ngoài `\"` và `\\`, không đa dòng)
// Bất kỳ dòng nào khác ⇒ lỗi cú pháp, dừng và báo — KHÔNG cố đoán.
// ═════════════════════════════════════════════════════════════════════════════════
class TomlSyntaxError extends Error {
  constructor(lineNo, line, why) {
    super(`dòng ${lineNo}: ${why}\n       > ${line}`)
  }
}

const SECTION_RE = /^\[([A-Za-z_][A-Za-z0-9_.-]*)\]$/
const ARRAY_SECTION_RE = /^\[\[([A-Za-z_][A-Za-z0-9_.-]*)\]\]$/
const KV_RE = /^([A-Za-z_][A-Za-z0-9_.-]*)\s*=\s*"((?:\\.|[^"\\])*)"$/

const SUPPORTED_ESCAPES = new Set(['\\', '"', 'n'])

function unescape(s, lineNo, rawLine) {
  return s.replace(/\\(.)/g, (whole, c) => {
    if (!SUPPORTED_ESCAPES.has(c)) {
      // ⛔ Không đoán — trước đây `\t`/`A`/bất kỳ escape lạ nào bị nuốt âm thầm
      // thành chỉ ký tự cuối (mất luôn dấu `\`), mâu thuẫn với lời hứa "ngoài tập con
      // ⇒ FAIL" ở đầu tệp (Review Findings Group C).
      throw new TomlSyntaxError(lineNo, rawLine, `escape '\\${c}' ngoài tập con hỗ trợ (chỉ \\\\ · \\" · \\n)`)
    }
    return c === 'n' ? '\n' : c
  })
}

/**
 * Trả về `{ tables: Map<sectionPath, object[]>, forms: Map<sectionPath, 'single'|'array'> }`
 * — `[base]` là một bảng ĐƠN (`forms.get('base') === 'single'`), khoá trong `tables`
 * là `'base'` với mảng ĐÚNG MỘT phần tử; `[[detachable]]` là một mảng bảng
 * (`forms.get('detachable') === 'array'`), khoá `'detachable'` với N phần tử theo thứ
 * tự xuất hiện. `forms` tồn tại vì `[base]` và `[[base]]` xuất hiện ĐÚNG MỘT LẦN không
 * thể phân biệt chỉ bằng độ dài mảng — dạng ngoặc SAI (mảng bảng cho một bảng đơn, hay
 * ngược lại) phải bị bắt riêng (Review Findings Group C).
 */
function parseStrictToml(text) {
  const tables = new Map()
  const forms = new Map()
  let current = null // { path, obj }
  const lines = text.split(/\r\n|\n/)

  for (let i = 0; i < lines.length; i++) {
    const rawLine = lines[i]
    const line = rawLine.trim()
    const lineNo = i + 1
    if (line === '' || line.startsWith('#')) continue

    const arrSection = line.match(ARRAY_SECTION_RE)
    if (arrSection) {
      const name = arrSection[1]
      if (forms.has(name) && forms.get(name) !== 'array') {
        throw new TomlSyntaxError(
          lineNo,
          rawLine,
          `'${name}' trước đó khai dạng [${name}] (bảng đơn) — không thể lại khai [[${name}]] (mảng bảng)`,
        )
      }
      forms.set(name, 'array')
      const obj = {}
      if (!tables.has(name)) tables.set(name, [])
      tables.get(name).push(obj)
      current = obj
      continue
    }

    const section = line.match(SECTION_RE)
    if (section) {
      const name = section[1]
      const obj = {}
      if (tables.has(name)) {
        const why =
          forms.get(name) === 'array'
            ? `'${name}' trước đó khai dạng [[${name}]] (mảng bảng) — không thể lại khai [${name}] (bảng đơn)`
            : `bảng '${name}' khai lại (không phải mảng bảng)`
        throw new TomlSyntaxError(lineNo, rawLine, why)
      }
      forms.set(name, 'single')
      tables.set(name, [obj])
      current = obj
      continue
    }

    const kv = line.match(KV_RE)
    if (kv) {
      if (!current) {
        throw new TomlSyntaxError(lineNo, rawLine, 'gán khoá/giá trị trước khi có bất kỳ [section] nào')
      }
      // Khoá trùng TRONG một bảng — trước đây ghi đè âm thầm (giá trị sau thắng),
      // không lỗi; giờ FAIL rõ để bắt lỗi copy-paste (Review Findings Group C).
      if (Object.prototype.hasOwnProperty.call(current, kv[1])) {
        throw new TomlSyntaxError(lineNo, rawLine, `khoá '${kv[1]}' đã khai trong bảng này rồi (khai lại)`)
      }
      current[kv[1]] = unescape(kv[2], lineNo, rawLine)
      continue
    }

    throw new TomlSyntaxError(
      lineNo,
      rawLine,
      'ngoài tập con hỗ trợ (chỉ [section] · [[array]] · key = "chuỗi nháy kép" · comment `#`)',
    )
  }

  return { tables, forms }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm cú pháp — tập con TOML nghiêm ngặt')

let text
try {
  text = readFileSync(MANIFEST_PATH, 'utf8')
} catch (err) {
  console.error(`\n\x1b[31mKhông đọc được ${MANIFEST_PATH}.\x1b[0m`)
  console.error(String(err?.message || err).trim())
  process.exit(1)
}

let tables
let forms
try {
  ;({ tables, forms } = parseStrictToml(text))
  pass('cú pháp nằm trọn trong tập con hỗ trợ')
} catch (err) {
  if (err instanceof TomlSyntaxError) {
    fail(`cú pháp TOML ngoài tập con hỗ trợ — ${err.message}`)
    console.log('')
    console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
    process.exit(1)
  }
  throw err
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm luật — [base] bắt buộc, mỗi mục đủ ba trường đúng hình dạng')

const SHA256_RE = /^[0-9a-f]{64}$/
// Ghim ĐÚNG host + tổ chức + repo — trước đây `.*` giữa `https://` và
// `/releases/download/dict-v` chấp nhận BẤT KỲ domain HTTPS nào chứa đúng chuỗi con
// đó (vd. `https://evil.example.com/x/releases/download/dict-v1/...`). URL này là thứ
// sẽ được TẢI XUỐNG máy người dùng thật (AC3), nên phải ghim đúng repo đã ghi trong
// §Trạng thái repo hiện tại của story (`git remote origin`) — Review Findings Group C.
const URL_RE = /^https:\/\/github\.com\/vannamhh\/AuraTranslate\/releases\/download\/dict-v/

function validateEntry(label, obj, requireName) {
  if (requireName) {
    if (typeof obj.name !== 'string' || obj.name === '') {
      fail(`${label}: thiếu trường 'name'`)
    } else {
      pass(`${label}: 'name' = '${obj.name}'`)
    }
  }
  if (typeof obj.url !== 'string' || obj.url === '') {
    fail(`${label}: thiếu trường 'url'`)
  } else if (!obj.url.startsWith('https://')) {
    fail(`${label}: 'url' không bắt đầu bằng https:// — '${obj.url}'`)
  } else if (!URL_RE.test(obj.url)) {
    fail(`${label}: 'url' không trỏ vào tag dạng .../releases/download/dict-v... — '${obj.url}'`)
  } else {
    pass(`${label}: 'url' đúng hình dạng`)
  }

  if (typeof obj.sha256 !== 'string' || obj.sha256 === '') {
    fail(`${label}: thiếu trường 'sha256'`)
  } else if (!SHA256_RE.test(obj.sha256)) {
    fail(`${label}: 'sha256' không phải 64 ký tự hex thường — '${obj.sha256}' (dài ${obj.sha256.length})`)
  } else {
    pass(`${label}: 'sha256' đúng hình dạng (64 hex thường)`)
  }

  if (typeof obj.source_version !== 'string' || obj.source_version === '') {
    fail(`${label}: thiếu hoặc rỗng 'source_version'`)
  } else {
    pass(`${label}: 'source_version' = '${obj.source_version}'`)
  }
}

const baseList = tables.get('base')
if (!baseList) {
  fail("[base] KHÔNG có mặt trong dict-manifest.toml — bắt buộc phải có")
} else if (forms.get('base') !== 'single') {
  // `[[base]]` xuất hiện đúng MỘT lần trông giống `[base]` nếu chỉ đếm độ dài mảng —
  // phải phân biệt bằng DẠNG NGOẶC, không chỉ bằng số lượng (Review Findings Group C).
  fail(`[base] khai sai dạng ngoặc — phải là bảng ĐƠN [base], không phải mảng bảng [[base]]`)
} else if (baseList.length !== 1) {
  fail(`[base] xuất hiện ${baseList.length} lần — phải là một bảng ĐƠN, không phải mảng`)
} else {
  validateEntry('[base]', baseList[0], false)
}

const detachableList = tables.get('detachable') ?? []
if (detachableList.length === 0) {
  pass('[[detachable]] — 0 mục hôm nay, hợp lệ (Story 1.10 sẽ thêm)')
} else if (forms.get('detachable') !== 'array') {
  fail(`[[detachable]] khai sai dạng ngoặc — phải là mảng bảng [[detachable]], không phải bảng đơn [detachable]`)
} else {
  detachableList.forEach((obj, idx) => {
    validateEntry(`[[detachable]] #${idx + 1}`, obj, true)
  })
}

// ─────────────────────────────────────────────────────────────────────────────────
console.log('')
if (failures !== 0) {
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  process.exit(1)
}
console.log('\x1b[32mTất cả phép kiểm dict-manifest.toml đạt.\x1b[0m')
process.exit(0)
