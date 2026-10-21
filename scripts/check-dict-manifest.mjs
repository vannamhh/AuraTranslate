#!/usr/bin/env node
/**
 * Cổng `dict-manifest.toml` — Task 8 của Story 1.9 (AC3, đóng `deferred-work.md §*Deferred from: code review of 1-3-ci-toi-thieu-hai-nen-tang-moi-lan-push (2026-08-03)*`:
 * *"đặt ra luật ba trường rồi không cưỡng chế bằng gì cả"*), siết thêm ở Task 7 của
 * Story 1.10 (AC5): `DETACHABLE_ALL` (Rust) = `[[detachable]]` ∪ `[[removed]]` (FR112), disjoint.
 * A dropped layer fails; an intentional removal needs no code change.
 *
 * 🔴 Cổng này KHÔNG đọc tệp `.db` và KHÔNG tải gì từ mạng — nó phải xanh trên một
 * runner CI không có byte dữ liệu từ điển nào (AC cuối của Story 1.3: CI không tải
 * dữ liệu từ điển). Nó kiểm HÌNH DẠNG manifest, không kiểm nội dung tệp.
 *
 * Parser TOML ở đây là TẬP CON NGHIÊM NGẶT, tự viết — tiền lệ `parseCssBlocks` của
 * `check-tokens.mjs`: đủ cho ĐÚNG những gì `dict-manifest.toml` cần, và không hơn.
 * Cú pháp NGOÀI tập con (mảng, bảng inline, chuỗi nhiều dòng, số, boolean, khoá không
 * nháy) ⇒ FAIL, không bỏ qua. NFR15 đòi rà GPLv3 + vào bảng Stack trước khi thêm một
 * phụ thuộc npm; một tệp 40 dòng không đáng một lượt rà.
 *
 * Chạy:  npm run check:dict-manifest
 */
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { checkLayerPartition, readDetachableAllCodes } from './lib/detachable-all.mjs'
import { TomlSyntaxError, parseStrictToml } from './lib/strict-toml.mjs'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const MANIFEST_PATH = join(REPO_ROOT, 'dict-manifest.toml')

let failures = 0
/** @param {string} m */
const pass = (m) => console.log(`  \x1b[32mOK\x1b[0m   ${m}`)
/** @param {string} m */
const fail = (m) => {
  console.log(`  \x1b[31mFAIL\x1b[0m ${m}`)
  failures += 1
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm cú pháp — tập con TOML nghiêm ngặt')

let text
try {
  text = readFileSync(MANIFEST_PATH, 'utf8')
} catch (err) {
  console.error(`\n\x1b[31mKhông đọc được ${MANIFEST_PATH}.\x1b[0m`)
  console.error(String((/** @type {Error | undefined} */ (err))?.message || err).trim())
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

/**
 * @param {string} label
 * @param {Record<string, string>} obj
 * @param {boolean} requireName
 */
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

// Layer list comes from DETACHABLE_ALL; each code must be in [[detachable]] or [[removed]].
const SOURCES_META_PATH = join(REPO_ROOT, 'tools', 'dict-build', 'src', 'sources_meta.rs')
let detachableCodes = []
{
  let metaText = ''
  try {
    metaText = readFileSync(SOURCES_META_PATH, 'utf8')
  } catch (err) {
    fail(`không đọc được ${SOURCES_META_PATH}: ${String((/** @type {Error | undefined} */ (err))?.message || err).trim()}`)
  }
  const read = readDetachableAllCodes(metaText)
  read.problems.forEach(fail)
  detachableCodes = read.codes
  if (read.problems.length === 0 && detachableCodes.length === 0) {
    fail('DETACHABLE_ALL (Rust) khai 0 mã lớp — một danh sách rỗng không phải "đạt"')
  }
}

const detachableList = tables.get('detachable') ?? []
const removedList = tables.get('removed') ?? []
if (forms.get('detachable') === 'single') {
  fail(`[[detachable]] khai sai dạng ngoặc — phải là mảng bảng [[detachable]], không phải bảng đơn [detachable]`)
} else if (forms.get('removed') === 'single') {
  fail(`[[removed]] khai sai dạng ngoặc — phải là mảng bảng [[removed]], không phải bảng đơn [removed]`)
} else if (detachableCodes.length > 0) {
  detachableList.forEach((obj, idx) => {
    validateEntry(`[[detachable]] #${idx + 1}`, obj, true)
  })
  const partition = checkLayerPartition({ codes: detachableCodes, detachable: detachableList, removed: removedList })
  if (partition.length === 0) {
    pass(
      `[[detachable]] ${JSON.stringify(detachableList.map((o) => o.name))} ∪ [[removed]] ${JSON.stringify(removedList.map((o) => o.name))} = DETACHABLE_ALL ${JSON.stringify(detachableCodes)}`,
    )
  } else {
    partition.forEach(fail)
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 `url` phải khớp `name` của CHÍNH mục đó, và không mục nào dùng chung `url`/
// `sha256` với mục khác.
//
// URL_RE chỉ ghim host/org/repo/tiền-tố-tag rồi DỪNG, còn Kiểm D của check-dict-build
// chỉ so tập `name` — nên hoán đổi `url` giữa hai mục `[[detachable]]` đi qua MỌI phép
// kiểm hôm nay, và người dùng tải `dict-thieu-chuu.db` sẽ nhận nội dung VietPhrase.
// Quy tắc tên tệp là hàm XÁC ĐỊNH ở phía Rust (`build::output_file_name` → `dict-<code>.db`,
// §Quyết định #3), nên nó kiểm được từ đây mà không cần đọc một byte `.db` nào.
// ═════════════════════════════════════════════════════════════════════════════════
{
  const all = [
    ...(baseList && baseList.length === 1 ? [{ label: '[base]', obj: baseList[0], fileName: 'dict-core.db' }] : []),
    ...detachableList
      .filter((o) => typeof o.name === 'string' && o.name !== '')
      .map((o) => ({ label: `[[detachable]] '${o.name}'`, obj: o, fileName: `dict-${o.name}.db` })),
  ]

  let fileNameMismatch = 0
  for (const { label, obj, fileName } of all) {
    if (typeof obj.url !== 'string') continue
    if (!obj.url.endsWith(`/${fileName}`)) {
      fail(`${label}: 'url' phải kết thúc bằng '/${fileName}' (quy tắc dict-<code>.db của build::output_file_name) — '${obj.url}'`)
      fileNameMismatch += 1
    }
  }
  if (fileNameMismatch === 0 && all.length > 0) {
    pass(`mọi 'url' kết thúc đúng tên tệp suy ra từ 'name' (${all.length} mục)`)
  }

  for (const field of ['url', 'sha256']) {
    const seen = new Map()
    const dupes = []
    for (const { label, obj } of all) {
      const value = obj[field]
      if (typeof value !== 'string' || value === '') continue
      if (seen.has(value)) dupes.push(`${seen.get(value)} ↔ ${label}`)
      else seen.set(value, label)
    }
    if (dupes.length) {
      fail(`hai mục dùng CHUNG '${field}' — một lớp sẽ không bao giờ được tải thật: ${JSON.stringify(dupes)}`)
    } else if (all.length > 0) {
      pass(`mọi '${field}' là duy nhất giữa các mục (${all.length} mục)`)
    }
  }
}

// ─────────────────────────────────────────────────────────────────────────────────
console.log('')
if (failures !== 0) {
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  process.exit(1)
}
console.log('\x1b[32mTất cả phép kiểm dict-manifest.toml đạt.\x1b[0m')
process.exit(0)
