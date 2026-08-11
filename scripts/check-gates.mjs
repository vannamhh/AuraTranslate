/**
 * Cổng thứ MƯỜI MỘT — BA danh sách cổng phải khai cùng một bộ.
 *
 * Năm phép kiểm:
 *
 *   A  mọi script `check:*` trong `package.json` được `ci.yml` GỌI.
 *   B  mọi `npm run <x>` trong `ci.yml` tồn tại trong `package.json`.
 *   C  TỰ KIỂM — chứng minh A, B và bộ đọc pre-push đỏ được, và không đỏ oan.
 *   D  mọi script `check:*` trong `package.json` được `.githooks/pre-push` CHẠY.
 *   E  mọi cổng `.githooks/pre-push` chạy tồn tại trong `package.json`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO CỔNG NÀY TỒN TẠI — một phép đo, không một lo xa
 * ═════════════════════════════════════════════════════════════════════════════════
 * Ngày 2026-08-11, lượt code review Story 1.21 dựng cổng thứ mười (`check:lint`) để đóng
 * đúng lớp lỗi mà chín cổng kia không thấy: `if (someRef)` trên một `Ref` của Vue là
 * TypeScript hợp lệ, nên `vue-tsc` im và nhánh bên trong thành mã chết. HAI trong bốn
 * phát hiện hạng cao của lượt đó mang đúng hình dạng ấy.
 *
 * Cổng ra đời ở `package.json`, và `ci.yml` KHÔNG được sửa cùng lượt. Lượt rà soát toàn
 * Epic 1 cùng ngày mới bắt được. Trong khoảng giữa, cổng thứ mười canh máy dev và
 * KHÔNG canh nhánh — tức nó là một cổng dựa vào trí nhớ, đúng thứ AC3 của Story 1.3
 * cấm bằng chữ.
 *
 * Nguyên nhân không phải một lượt quên: kho có **hai** danh sách cổng, và trước tệp này
 * không một phép kiểm nào buộc chúng khớp. Đây là cổng đóng chính nguyên nhân đó.
 *
 * ── Danh sách thứ BA, thêm 2026-08-11 (correct-course) ────────────────────────────
 * `.githooks/pre-push` ra đời cùng ngày để cưỡng chế tại chỗ trong lúc GitHub Actions
 * tạm dừng, và nó mang **danh sách cổng riêng**. Hai danh sách không ai buộc khớp là
 * nguyên nhân của sự cố `check:lint`; ba danh sách mà chỉ canh hai là **cùng một** lỗ
 * hổng dời sang một tệp khác. Kiểm D và Kiểm E đóng nó.
 *
 * ⚠️ Ghi thẳng chỗ căng, thay vì để người sau tưởng nó đã được xét: dòng kết của tệp
 * này in ra *"AC4 của Story 1.3 — MỘT pipeline duy nhất"*, và hook pre-push LÀ một
 * đường cưỡng chế thứ hai. AC4 cấm bằng chữ một **tệp workflow** thứ hai, nên hook
 * không phạm chữ; nhưng tinh thần của AC4 *(một danh sách, không dựa trí nhớ)* chỉ còn
 * đúng KHI có Kiểm D. Đây không phải một ngoại lệ xin khỏi AC4 — nó là điều kiện để
 * AC4 tiếp tục đúng dưới ba danh sách.
 *
 * ⚠️ GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện: tệp này đọc `ci.yml` bằng
 * BIỂU THỨC CHÍNH QUY trên văn bản thuần, không bằng một bộ phân tích YAML (một phụ
 * thuộc npm mới — NFR15). Hệ quả đo được:
 *   - một `npm run check:x` nằm trong một dòng **chú thích** của `ci.yml` vẫn được đếm là
 *     "có gọi". Đối chứng: hôm nay `ci.yml` có 0 chú thích dạng đó, và Kiểm C canh việc
 *     regex vẫn nhận đúng tên.
 *   - `if:` của một bước KHÔNG được đọc, nên một bước gắn `if: false` vẫn tính là gọi.
 * Cả hai hở đều đi theo hướng "rộng rãi hơn thực tế" — cổng này bắt việc QUÊN HẲN, không
 * bắt việc gọi có điều kiện. Đó là đúng lớp lỗi đã xảy ra.
 *
 * Chạy:  npm run check:gates
 */
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const PKG_PATH = join(REPO_ROOT, 'package.json')
const CI_PATH = join(REPO_ROOT, '.github', 'workflows', 'ci.yml')
const PREPUSH_PATH = join(REPO_ROOT, '.githooks', 'pre-push')

let failures = 0
const pass = (m) => console.log(`  \x1b[32mOK\x1b[0m   ${m}`)
const fail = (m) => {
  console.log(`  \x1b[31mFAIL\x1b[0m ${m}`)
  failures += 1
}
const detail = (m) => console.log(`       ${m}`)

/** Lỗi hạ tầng KHÔNG phải một phép kiểm đỏ. Dừng ngay, đừng báo một kết quả không có thật. */
function abort(what, err) {
  console.error(`\n\x1b[31mKhông đọc được ${what} — phép kiểm KHÔNG chạy được.\x1b[0m`)
  console.error('Đây là lỗi hạ tầng, không phải "đạt". Đọc lỗi dưới đây rồi chạy lại.\n')
  console.error(String(err?.message || err).trim())
  process.exit(1)
}

/**
 * Script `check:*` được phép VẮNG MẶT trong `ci.yml` — mỗi mục PHẢI kèm một lý do.
 *
 * Rỗng có chủ ý. Một cổng không chạy trên nhánh là một cổng dựa vào trí nhớ; nếu có ngày
 * một mục phải vào đây, lý do sẽ đọc được ngay tại chỗ chứ không nằm trong đầu ai.
 */
const CI_EXEMPT = new Map(/** @type {[string, string][]} */ ([]))

/**
 * Cổng được phép VẮNG MẶT trong `.githooks/pre-push` — mỗi mục PHẢI kèm một lý do.
 *
 * Hai mục dưới đây chép từ chính khối §Phạm vi của hook, không phải một phán đoán mới ở
 * đây. Nếu lý do đổi thì sửa cả hai chỗ — chúng nói về cùng một quyết định.
 */
const PREPUSH_EXEMPT = new Map(
  /** @type {[string, string][]} */ ([
    [
      'check:scope',
      'dựng cửa sổ Tauri thật, cần cổng 1420 trống — chặn nhầm khi Ice đang mở `tauri dev`',
    ],
    ['check:scope:bundled', 'cùng lý do, cộng một lượt `tauri build` đầy đủ'],
  ]),
)

// ═════════════════════════════════════════════════════════════════════════════════
let pkg
try {
  pkg = JSON.parse(readFileSync(PKG_PATH, 'utf8'))
} catch (err) {
  abort('package.json', err)
}

let ciText
try {
  ciText = readFileSync(CI_PATH, 'utf8')
} catch (err) {
  abort('.github/workflows/ci.yml', err)
}

let prepushText
try {
  prepushText = readFileSync(PREPUSH_PATH, 'utf8')
} catch (err) {
  abort('.githooks/pre-push', err)
}

/** @type {Record<string, string>} */
const scripts = pkg.scripts ?? {}
const checkScripts = Object.keys(scripts).filter((name) => name.startsWith('check:')).sort()

/**
 * Tên script mà một tệp workflow gọi qua `npm run`.
 *
 * ⚠️ Lớp ký tự PHẢI chứa `:` — `check:scope:bundled` và `check:scope` là HAI script khác
 * nhau, và một regex dừng ở dấu hai chấm đầu tiên sẽ trộn chúng làm một rồi báo đạt oan.
 */
const npmRunNames = (text) => {
  const found = new Set()
  for (const m of text.matchAll(/npm run ([A-Za-z0-9:._-]+)/g)) found.add(m[1])
  return found
}

const called = npmRunNames(ciText)

/**
 * Tên cổng mà `.githooks/pre-push` thật sự chạy, đọc từ vòng lặp
 * `for gate in deps tokens … ; do` — thân vòng lặp gọi `npm run check:$gate`.
 *
 * 🔴 Trả `null` khi KHÔNG đọc nổi vòng lặp, và đó là chỗ quan trọng nhất của hàm này.
 * Một bộ đọc trả về tập RỖNG sẽ làm Kiểm D xanh trong khi nó chẳng kiểm gì — đúng lớp
 * lỗi "rỗng im lặng" mà AD-26 và AD-44 ④ tồn tại để cấm. `null` buộc chỗ gọi phải
 * `abort`, tức một lỗi hạ tầng tường minh thay vì một phán quyết đạt không có thật.
 */
const prepushGateNames = (text) => {
  const m = text.match(/^for\s+gate\s+in\s+([^;]+);/m)
  return m ? new Set(m[1].trim().split(/\s+/).map((g) => `check:${g}`)) : null
}

const hooked = prepushGateNames(prepushText)
if (hooked === null || hooked.size === 0) {
  abort(
    'danh sách cổng trong .githooks/pre-push',
    'Không tìm được vòng lặp `for gate in … ; do`. Hook có thể đã đổi hình dạng.\n' +
      'Cổng này KHÔNG báo đạt khi nó không đọc nổi thứ nó phải đối chiếu — sửa biểu\n' +
      'thức chính quy `prepushGateNames` cho khớp hình dạng mới, đừng bỏ Kiểm D.',
  )
}

console.log('')
console.log('\x1b[1mCổng — ba danh sách cổng khai cùng một bộ\x1b[0m')
console.log('')

// ── Kiểm A ──────────────────────────────────────────────────────────────────────
{
  const missing = checkScripts.filter((name) => !called.has(name) && !CI_EXEMPT.has(name))
  if (missing.length === 0) {
    const exempt = checkScripts.filter((name) => CI_EXEMPT.has(name))
    pass(`Kiểm A — ${checkScripts.length} script \`check:*\` đều được ci.yml gọi`)
    for (const name of exempt) detail(`miễn trừ: ${name} — ${CI_EXEMPT.get(name)}`)
  } else {
    fail(`Kiểm A — ${missing.length} cổng sống trong package.json mà ci.yml KHÔNG gọi`)
    for (const name of missing) detail(`${name} — thêm một bước \`npm run ${name}\` vào ci.yml`)
    detail('Một cổng chỉ chạy trên máy dev là một cổng dựa vào trí nhớ (AC3, Story 1.3).')
  }
}

// ── Kiểm B ──────────────────────────────────────────────────────────────────────
{
  const orphans = [...called].filter((name) => !(name in scripts)).sort()
  if (orphans.length === 0) {
    pass(`Kiểm B — ${called.size} lời gọi \`npm run\` trong ci.yml đều có script thật`)
  } else {
    fail(`Kiểm B — ${orphans.length} lời gọi trỏ vào script KHÔNG tồn tại`)
    for (const name of orphans) detail(`npm run ${name} — không có trong package.json`)
    detail('Đổi tên một script mà quên ci.yml thì job đỏ ở runner, muộn hơn ở đây một vòng.')
  }
}

// ── Kiểm D ──────────────────────────────────────────────────────────────────────
{
  const missing = checkScripts.filter((name) => !hooked.has(name) && !PREPUSH_EXEMPT.has(name))
  if (missing.length === 0) {
    const exempt = checkScripts.filter((name) => PREPUSH_EXEMPT.has(name))
    pass(`Kiểm D — ${hooked.size} cổng của pre-push phủ đúng bộ \`check:*\` phải phủ`)
    for (const name of exempt) detail(`miễn trừ: ${name} — ${PREPUSH_EXEMPT.get(name)}`)
  } else {
    fail(`Kiểm D — ${missing.length} cổng sống trong package.json mà pre-push KHÔNG chạy`)
    for (const name of missing) {
      detail(`${name} — thêm tên (bỏ tiền tố \`check:\`) vào vòng lặp của .githooks/pre-push`)
    }
    detail('Trong lúc GitHub Actions tạm dừng, pre-push là đường cưỡng chế DUY NHẤT chạy')
    detail('mỗi lượt. Một cổng vắng mặt ở đó là một cổng dựa vào trí nhớ (AC3, Story 1.3).')
  }
}

// ── Kiểm E ──────────────────────────────────────────────────────────────────────
{
  const orphans = [...hooked].filter((name) => !(name in scripts)).sort()
  if (orphans.length === 0) {
    pass(`Kiểm E — ${hooked.size} cổng pre-push gọi đều có script thật`)
  } else {
    fail(`Kiểm E — ${orphans.length} cổng pre-push gọi trỏ vào script KHÔNG tồn tại`)
    for (const name of orphans) detail(`${name} — không có trong package.json`)
    detail('`npm run` một script vắng mặt làm hook đỏ với một thông báo khó đọc hơn hẳn.')
  }
}

// ── Kiểm C — TỰ KIỂM ─────────────────────────────────────────────────────────────
{
  const CASES = [
    {
      why: 'Kiểm A đỏ được: một cổng vắng mặt trong ci.yml',
      names: ['check:deps', 'check:ao-tuong'],
      ci: 'run: npm run check:deps',
      expect: 'A-red',
    },
    {
      why: 'Kiểm A không đỏ oan: đủ bộ thì xanh',
      names: ['check:deps'],
      ci: 'run: npm run check:deps',
      expect: 'A-green',
    },
    {
      why: 'Kiểm B đỏ được: ci.yml gọi một script đã bị xoá',
      names: ['check:deps'],
      ci: 'run: npm run check:deps\nrun: npm run check:da-xoa',
      expect: 'B-red',
    },
    {
      why: '`check:scope` và `check:scope:bundled` KHÔNG bị trộn làm một',
      names: ['check:scope', 'check:scope:bundled'],
      ci: 'run: npm run check:scope:bundled',
      expect: 'A-red',
    },
  ]

  /**
   * Ca cho bộ đọc pre-push. Ca thứ hai là ca ĐẮT NHẤT của cả tệp: nó chứng minh một
   * hook đổi hình dạng cho ra `null` — tức `abort` — chứ không cho ra một tập rỗng làm
   * Kiểm D xanh oan.
   */
  const HOOK_CASES = [
    {
      why: 'bộ đọc lấy đúng tên từ vòng lặp thật',
      hook: 'for gate in deps tokens lint; do\n  npm run --silent "check:$gate"\ndone',
      expect: 'check:deps,check:lint,check:tokens',
    },
    {
      why: 'vòng lặp đổi hình dạng cho ra `null`, KHÔNG cho ra tập rỗng',
      hook: 'for g in deps tokens; do\n  npm run --silent "check:$g"\ndone',
      expect: 'null',
    },
  ]

  let wrong = 0
  for (const c of CASES) {
    const seen = npmRunNames(c.ci)
    const aRed = c.names.some((n) => !seen.has(n))
    const bRed = [...seen].some((n) => !c.names.includes(n))
    const got = aRed ? 'A-red' : bRed ? 'B-red' : 'A-green'
    if (got !== c.expect) {
      wrong += 1
      fail(`Kiểm C — ca "${c.why}" cho ${got}, mong đợi ${c.expect}`)
    }
  }
  for (const c of HOOK_CASES) {
    const seen = prepushGateNames(c.hook)
    const got = seen === null ? 'null' : [...seen].sort().join(',')
    if (got !== c.expect) {
      wrong += 1
      fail(`Kiểm C — ca "${c.why}" cho ${got}, mong đợi ${c.expect}`)
    }
  }
  if (wrong === 0) {
    pass(`Kiểm C — ${CASES.length + HOOK_CASES.length} ca tự kiểm đúng chiều`)
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('')
if (failures !== 0) {
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  console.log('')
  console.log('AC3 của Story 1.3: NFR14 cưỡng chế bằng CI, KHÔNG bằng trí nhớ.')
  console.log('AC4 của Story 1.3: MỘT pipeline duy nhất — thêm bước vào ci.yml, đừng dựng')
  console.log('tệp workflow thứ hai.')
  process.exit(1)
}
console.log('\x1b[32mBa danh sách cổng khớp nhau.\x1b[0m')
console.log('')
console.log(
  `Tầm quét: ${checkScripts.length} script \`check:*\` · ${called.size} lời gọi \`npm run\` trong ` +
    `ci.yml · ${hooked.size} cổng trong .githooks/pre-push.`,
)
console.log('')
console.log('Ghi chú cho người rà soát — ba giới hạn, ghi thẳng:')
console.log('  1. `ci.yml` đọc bằng regex trên văn bản thuần, không bằng bộ phân tích YAML')
console.log('     (một phụ thuộc npm mới — NFR15). Một lời gọi nằm trong CHÚ THÍCH vẫn tính.')
console.log('  2. `if:` của bước KHÔNG được đọc — cổng này bắt việc QUÊN HẲN, không bắt việc')
console.log('     gọi có điều kiện.')
console.log('  3. `.githooks/pre-push` đọc bằng một regex bám hình dạng `for gate in … ;`.')
console.log('     Viết lại vòng lặp theo hình dạng khác làm cổng DỪNG bằng abort, không làm')
console.log('     nó xanh oan — Kiểm C ca 6 giữ mệnh đề đó.')
process.exit(0)
