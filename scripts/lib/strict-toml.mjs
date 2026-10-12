/**
 * Strict TOML subset shared by `check-dict-manifest.mjs` and `fetch-dicts.mjs`:
 * blank lines, `# comment` lines, `[section]`, `[[array_of_tables]]` and
 * `key = "double-quoted string"` (escapes `\\` `\"` `\n` only, one line). Any other line throws
 * `TomlSyntaxError`; the parser never guesses.
 */

export class TomlSyntaxError extends Error {
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
      // Không đoán — trước đây `\t`/`A`/bất kỳ escape lạ nào bị nuốt âm thầm
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
export function parseStrictToml(text) {
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
