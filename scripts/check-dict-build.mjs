#!/usr/bin/env node
/**
 * Cổng `tools/dict-build` — Task 7 của Story 1.9, siết thêm ở Task 8 của Story 1.10.
 * Năm phép kiểm, đúng khuôn doctrine `check-deps.mjs`/`check-i18n.mjs`: script trả mã
 * thoát khác 0 khi thất bại, lỗi hạ tầng KHÔNG được báo thành "đạt", miễn trừ CÓ TÊN và
 * CÓ LÝ DO, in ra mỗi lượt chạy.
 *
 * Kiểm A — từ vựng HỢP NHẤT (AD-19, §Quyết định #4 của Story 1.9). Quét
 *          `tools/dict-build/src/**\/*.rs` tìm token cấm; miễn trừ khai bằng comment
 *          `// dict-build:allow <token> — <lý do>` NGAY DÒNG TRÊN dòng vi phạm.
 * Kiểm B — cách ly workspace (AC4). `tools/dict-build/Cargo.toml` phải có `[workspace]`
 *          và KHÔNG phụ thuộc nào trỏ `path` sang `src-tauri`.
 * Kiểm C — sàn số tệp (bài học `check-deps.mjs:15-17`, `store_boundary.rs:44`): cây
 *          rỗng ⛔ không được đọc thành sạch.
 * Kiểm D — chống TRÔI giữa `sources_meta.rs` (Rust) và `dict-manifest.toml` (Story
 *          1.10): mã lớp gỡ rời khai trong `DETACHABLE_ALL` phải khớp CHÍNH XÁC tập
 *          `name` mà `check-dict-manifest.mjs` đòi ở `[[detachable]]`.
 * Kiểm E — cách ly lớp (AD-19, §Bẫy 3 của Story 1.10): `src/sources/{thieu_chuu,
 *          vietphrase}.rs` và đường dựng lớp gỡ rời (`build.rs`) ⛔ không token
 *          `dict-core`/`dict_core` — đường dựng lớp gỡ rời không bao giờ được PHÉP mở
 *          `dict-core.db`. Cùng cơ chế miễn trừ với Kiểm A.
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
// 🔴 Sàn phải SÁT số thật, ⛔ không để hở đúng bằng số tệp mà story vừa thêm — sàn 18
// với số thật 20 cho phép xoá CẢ HAI parser lớp gỡ rời mà Kiểm C vẫn xanh (Review
// Findings 1.10). Thêm/bớt tệp .rs ⇒ cập nhật con số này cùng lượt.
const RS_FILE_FLOOR = 20 // số thật 2026-08-05 (Story 1.10): 20 tệp .rs dưới src/

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

// ═════════════════════════════════════════════════════════════════════════════════
// Kiểm D — chống trôi giữa Rust (`sources_meta.rs`) và `dict-manifest.toml` (Story 1.10)
// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm D — chống trôi giữa DETACHABLE_ALL (Rust) và [[detachable]] (manifest)')

const sourcesMetaPath = join(SRC_ROOT, 'sources_meta.rs')
let sourcesMetaText = ''
try {
  sourcesMetaText = readFileSync(sourcesMetaPath, 'utf8')
} catch (err) {
  abort(`${posix(sourcesMetaPath)}`, err)
}

// Bước 1 — lấy danh sách TÊN HẰNG trong mảng `DETACHABLE_ALL: [&SourceMeta; N] = [&X, &Y];`
const detachableArrayMatch = sourcesMetaText.match(/DETACHABLE_ALL\s*:\s*\[[^\]]*\]\s*=\s*\[([^\]]*)\]/)
let rustDetachableCodes = []
if (!detachableArrayMatch) {
  fail("không tìm thấy khai báo 'DETACHABLE_ALL' trong sources_meta.rs — Kiểm D không chạy được")
} else {
  const constNames = detachableArrayMatch[1]
    .split(',')
    .map((s) => s.trim().replace(/^&/, ''))
    .filter(Boolean)

  // Bước 2 — với MỖI tên hằng, tìm khối `pub const <TÊN>: SourceMeta = SourceMeta { ... };`
  // rồi lấy trường `code: "..."` bên trong.
  for (const name of constNames) {
    const constRe = new RegExp(`pub const ${name}\\s*:\\s*SourceMeta\\s*=\\s*SourceMeta\\s*\\{([\\s\\S]*?)\\};`)
    const constMatch = sourcesMetaText.match(constRe)
    if (!constMatch) {
      fail(`DETACHABLE_ALL khai '${name}' nhưng không tìm thấy khối 'pub const ${name}: SourceMeta { ... }'`)
      continue
    }
    const codeMatch = constMatch[1].match(/code\s*:\s*"([^"]*)"/)
    if (!codeMatch) {
      fail(`khối '${name}' không có trường 'code: "..."'`)
      continue
    }
    rustDetachableCodes.push(codeMatch[1])
  }
  pass(`DETACHABLE_ALL (Rust) khai ${rustDetachableCodes.length} mã lớp: ${JSON.stringify(rustDetachableCodes)}`)
}

const manifestPath = join(REPO_ROOT, 'dict-manifest.toml')
let manifestText = ''
try {
  manifestText = readFileSync(manifestPath, 'utf8')
} catch (err) {
  abort(`${posix(manifestPath)}`, err)
}

// Chỉ lấy `name = "..."` xuất hiện SAU một dòng `[[detachable]]` — tránh vô tình khớp
// `name` của một section khác nếu manifest mở rộng thêm bảng trong tương lai.
const manifestDetachableNames = []
{
  const lines = manifestText.split(/\r\n|\n/)
  let inDetachableBlock = false
  for (const line of lines) {
    const trimmed = line.trim()
    if (/^\[\[detachable\]\]$/.test(trimmed)) {
      inDetachableBlock = true
      continue
    }
    if (/^\[/.test(trimmed)) {
      inDetachableBlock = false
      continue
    }
    if (inDetachableBlock) {
      const m = trimmed.match(/^name\s*=\s*"([^"]*)"/)
      if (m) manifestDetachableNames.push(m[1])
    }
  }
}

const rustSet = new Set(rustDetachableCodes)
const manifestSet = new Set(manifestDetachableNames)
const missingFromManifest = rustDetachableCodes.filter((c) => !manifestSet.has(c))
const extraInManifest = manifestDetachableNames.filter((n) => !rustSet.has(n))

// 🔴 SÀN của Kiểm D — cùng doctrine với sàn số tệp của Kiểm C: một cây RỖNG ⛔ không
// được đọc thành sạch. Trước đây cả hai nhánh dưới đều bị chặn bởi
// `rustDetachableCodes.length > 0`, nên `DETACHABLE_ALL` rỗng cho ra 0 failure và cổng
// chống-trôi in "Tất cả phép kiểm đạt" đúng lúc cả hai lớp biến mất khỏi Rust.
if (rustDetachableCodes.length === 0) {
  fail(
    "DETACHABLE_ALL (Rust) khai 0 mã lớp gỡ rời — Kiểm D không có gì để đối chiếu. " +
      'Một danh sách rỗng ⛔ không phải "đạt": story hôm nay có HAI lớp gỡ rời.'
  )
} else if (missingFromManifest.length === 0 && extraInManifest.length === 0) {
  pass(`dict-manifest.toml [[detachable]] khớp CHÍNH XÁC DETACHABLE_ALL: ${JSON.stringify(manifestDetachableNames)}`)
} else {
  if (missingFromManifest.length) {
    fail(`DETACHABLE_ALL có mã lớp KHÔNG có mục [[detachable]] tương ứng trong manifest: ${JSON.stringify(missingFromManifest)}`)
  }
  if (extraInManifest.length) {
    fail(`dict-manifest.toml có [[detachable]] 'name' KHÔNG khớp mã lớp nào trong DETACHABLE_ALL: ${JSON.stringify(extraInManifest)}`)
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Kiểm E — cách ly lớp (§Bẫy 3 của Story 1.10): đường dựng lớp gỡ rời KHÔNG BAO GIỜ
// được phép mở dict-core.db. Cùng cơ chế miễn trừ với Kiểm A.
// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm E — cách ly lớp (§Bẫy 3: đường dựng lớp gỡ rời không mở dict-core.db)')

const ISOLATION_TOKENS = ['dict-core', 'dict_core']

// Phạm vi quét DẪN XUẤT, ⛔ không viết cứng từng đường dẫn: mọi parser lớp gỡ rời khai
// trong DETACHABLE_ALL (theo `code` → tên module `snake_case`) cộng đường điều phối
// `build.rs`. Đổi tên một parser hay thêm parser thứ ba ⇒ phạm vi tự đi theo.
const detachableModuleFiles = rustDetachableCodes.map(
  (code) => `tools/dict-build/src/sources/${code.replace(/-/g, '_')}.rs`
)
const ISOLATION_SCOPE = [...detachableModuleFiles, 'tools/dict-build/src/build.rs']
const isolationFiles = rsFiles.filter((f) => ISOLATION_SCOPE.includes(posix(f)))

// 🔴 SÀN của Kiểm E — cùng doctrine với Kiểm C và Kiểm D. Trước đây danh sách viết cứng
// ⛔ không có sàn: đổi tên `sources/thieu_chuu.rs` làm `isolationFiles` co về 0 phần tử,
// vòng lặp ⛔ không tìm thấy vi phạm nào, và cổng in OK đúng lúc §Bẫy 3 mất hiệu lực.
const missingIsolationFiles = ISOLATION_SCOPE.filter(
  (want) => !isolationFiles.some((f) => posix(f) === want)
)
if (missingIsolationFiles.length) {
  fail(
    `Kiểm E không quét được ${missingIsolationFiles.length} tệp thuộc phạm vi cách ly (đổi tên/xoá tệp?): ${JSON.stringify(missingIsolationFiles)}`
  )
}

let isolationExemptCount = 0
const isolationViolations = []
for (const file of isolationFiles) {
  const text = readFileSync(file, 'utf8')
  const lines = text.split(/\r\n|\n/)
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    if (/^\s*\/\//.test(line)) continue // comment thuần — không phải mã, chỉ dùng để tra miễn trừ
    const lowerLine = line.toLowerCase()
    for (const token of ISOLATION_TOKENS) {
      if (!lowerLine.includes(token.toLowerCase())) continue
      let exempted = false
      for (let j = i - 1; j >= 0 && /^\s*\/\//.test(lines[j]); j--) {
        const allowMatch = lines[j].match(ALLOW_RE)
        if (allowMatch && allowMatch[1] === token) {
          exempted = true
          break
        }
      }
      if (exempted) {
        isolationExemptCount += 1
        continue
      }
      isolationViolations.push({ file: posix(file), line: i + 1, token, text: line.trim() })
    }
  }
}

if (isolationViolations.length) {
  fail(`${isolationViolations.length} lần dùng token 'dict-core'/'dict_core' không có miễn trừ hợp lệ trong đường dựng lớp gỡ rời:`)
  for (const v of isolationViolations) {
    detail(`${v.file}:${v.line} — token '${v.token}' — ${v.text}`)
  }
} else {
  pass(`không token 'dict-core'/'dict_core' nào ngoài miễn trừ hợp lệ (${isolationFiles.length} tệp đã quét)`)
}
console.log(`       miễn trừ đã dùng: ${isolationExemptCount}`)

// ═════════════════════════════════════════════════════════════════════════════════
// Kiểm F — chống trôi `source_version` giữa Rust và manifest. Hằng `SOURCE_VERSION`
// trong mỗi module parser lớp gỡ rời và `source_version` của mục `[[detachable]]` tương
// ứng là HAI chuỗi chép tay: nâng nguồn lên @2.3 mà chỉ sửa một bên cho ra một release
// mang dữ liệu 2.3 nhưng metadata công bố 2.2 — đúng lớp lỗi mà `version_or_warn` đã
// được viết ra để chặn cho lớp nền.
// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm F — chống trôi source_version giữa Rust và dict-manifest.toml')

// `source_version` của từng mục `[[detachable]]`, khoá theo `name`.
const manifestSourceVersions = new Map()
{
  const lines = manifestText.split(/\r\n|\n/)
  let inBlock = false
  let currentName = null
  for (const line of lines) {
    const trimmed = line.trim()
    if (/^\[\[detachable\]\]$/.test(trimmed)) {
      inBlock = true
      currentName = null
      continue
    }
    if (/^\[/.test(trimmed)) {
      inBlock = false
      currentName = null
      continue
    }
    if (!inBlock) continue
    const nameMatch = trimmed.match(/^name\s*=\s*"([^"]*)"/)
    if (nameMatch) currentName = nameMatch[1]
    const versionMatch = trimmed.match(/^source_version\s*=\s*"([^"]*)"/)
    if (versionMatch && currentName) manifestSourceVersions.set(currentName, versionMatch[1])
  }
}

let sourceVersionChecked = 0
for (const code of rustDetachableCodes) {
  const modulePath = join(REPO_ROOT, 'tools', 'dict-build', 'src', 'sources', `${code.replace(/-/g, '_')}.rs`)
  let moduleText = ''
  try {
    moduleText = readFileSync(modulePath, 'utf8')
  } catch {
    fail(`không đọc được module parser của lớp '${code}': tools/dict-build/src/sources/${code.replace(/-/g, '_')}.rs`)
    continue
  }
  const constMatch = moduleText.match(/pub const SOURCE_VERSION\s*:\s*&str\s*=\s*"([^"]*)"\s*;/)
  if (!constMatch) {
    fail(`module của lớp '${code}' không khai 'pub const SOURCE_VERSION: &str = "…";'`)
    continue
  }
  const manifestValue = manifestSourceVersions.get(code)
  if (manifestValue === undefined) {
    fail(`lớp '${code}' không có 'source_version' trong mục [[detachable]] tương ứng`)
    continue
  }
  if (constMatch[1] !== manifestValue) {
    fail(`source_version của lớp '${code}' LỆCH giữa Rust và manifest`)
    detail(`Rust     : ${JSON.stringify(constMatch[1])}`)
    detail(`manifest : ${JSON.stringify(manifestValue)}`)
    continue
  }
  sourceVersionChecked += 1
}

if (sourceVersionChecked === rustDetachableCodes.length && rustDetachableCodes.length > 0) {
  pass(`source_version khớp giữa Rust và manifest cho cả ${sourceVersionChecked} lớp gỡ rời`)
}

// ─────────────────────────────────────────────────────────────────────────────────
console.log('')
if (failures !== 0) {
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  process.exit(1)
}
console.log('\x1b[32mTất cả phép kiểm tools/dict-build đạt.\x1b[0m')
process.exit(0)
