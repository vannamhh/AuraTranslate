/**
 * Cổng BỐ CỤC — Story 1.14 · AC1 · AC4 · AC7 · AC12.
 *
 * Năm phép kiểm, và cả năm đều chạy trên **mã của sản phẩm**, không trên một bản chép:
 *
 *   A (AC7)  thứ tự hy sinh của UX-DR15 — ba mệnh đề, gọi `nextToSacrifice()` THẬT.
 *   B (AC4)  nhịp ghi bố cục — ĐẾM số lượt `putConfig` trên một dòng sự kiện dày.
 *   C (AC1 · AC12) bề mặt cấm: không cửa sổ OS thứ hai, không kho lưu trữ thứ hai.
 *   D        TỰ KIỂM — chứng minh Kiểm C **đỏ được**, và không đỏ oan.
 *   E (Story 4.12) bốn ngưỡng bố cục theo preset — toàn phần (cả vế hữu hạn lẫn vế
 *     `null` khi không đo được), biên bao gồm cả hai đầu theo TỪNG preset, ưu tiên
 *     trên-xuống khi hai điều kiện cùng khớp, và `NEVER_SACRIFICED` không bị đụng —
 *     KHÔNG chứng minh "lưới còn hiện ở mọi tầng", việc đó thuộc Phase 4 trên dock thật.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO KIỂM C VIẾT DẠNG **DANH SÁCH CHO PHÉP**, KHÔNG PHẢI MỘT DANH SÁCH CẤM DÀI
 * ═════════════════════════════════════════════════════════════════════════════════
 * `src-tauri/src/core/config_invariants.rs:92-94` lập luận thẳng: *"một danh sách cấm chỉ
 * chặn được những hình dạng ai đó đã nghĩ ra"*. Một cổng cấm `window.open` không chặn
 * được `globalThis.open`, `window['op'+'en']`, hay `Window.prototype.open.call(...)`.
 *
 * ⇒ Kiểm C hỏi ngược lại: **mọi thành viên của `window` và `document` mà `src/**` chạm
 * tới phải nằm trong một danh sách CHO PHÉP**. Thêm một cái mới là một quyết định phải
 * viết ra, không phải một dòng lọt qua.
 *
 * ⚠️ Và một GIỚI HẠN THẬT, ghi ra thay vì giấu: `localStorage` gọi trần *(không có tiền
 * tố `window.`)* là một **định danh tự do**, và liệt kê hết định danh tự do đòi một bộ
 * phân tích cú pháp thật — thứ story này không dựng (nó sẽ là một phụ thuộc npm mới,
 * NFR15). Nên hai cái tên đó vẫn đi qua một mệnh đề CẤM hẹp, và cái giới hạn đó nằm ở đây
 * chứ không nằm trong trí nhớ ai.
 *
 * Chạy:  npm run check:layout
 */
import { readFileSync, readdirSync, lstatSync, realpathSync } from 'node:fs'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { dirname, join, relative, sep } from 'node:path'
import { functionBodyRange, balancedBraceBody, splitTopLevel } from './lib/commands-scan.mjs'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const SRC_ROOT = join(REPO_ROOT, 'src')

let failures = 0
const pass = (m) => console.log(`  \x1b[32mOK\x1b[0m   ${m}`)
const fail = (m) => {
  console.log(`  \x1b[31mFAIL\x1b[0m ${m}`)
  failures += 1
}
const detail = (m) => console.log(`       ${m}`)

/** Lỗi hạ tầng ≠ phép kiểm đỏ. Dừng ngay, đừng báo cáo một kết quả không có thật. */
function abort(what, err) {
  console.error(`\n\x1b[31mKhông đọc được ${what} — phép kiểm KHÔNG chạy được.\x1b[0m`)
  console.error('Đây là lỗi hạ tầng, không phải "đạt". Đọc lỗi dưới đây rồi chạy lại.\n')
  console.error(String(err?.message || err).trim())
  process.exit(1)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đọc cây nguồn
//
// ⚠️ `lstatSync`, KHÔNG `statSync` — cùng bài học với ba cổng trước: `statSync` giải
// symlink nên một liên kết trỏ về thư mục cha làm đệ quy không dừng, và một liên kết gãy
// ném `ENOENT` bị `abort()` báo thành "cây nguồn không đọc được".
// ═════════════════════════════════════════════════════════════════════════════════
const SCAN_EXT = ['.ts', '.tsx', '.mts', '.cts', '.js', '.mjs', '.cjs', '.vue']
const SKIP_DIRS = new Set(['node_modules', 'dist', 'target', '.git'])
const skippedLinks = []
const posix = (p) => relative(REPO_ROOT, p).split(sep).join('/')

function walk(dir, out = [], seen = new Set()) {
  let key
  try {
    key = realpathSync(dir)
  } catch {
    key = dir
  }
  if (seen.has(key)) return out
  seen.add(key)
  for (const name of readdirSync(dir)) {
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
 * 🔴 SÀN QUẦN THỂ — *"cây rỗng không phải cây sạch"*, thừa kế từ `check-deps.mjs`.
 *
 * Số THẬT lúc dựng cổng (Story 1.14): **11** tệp `.vue` + **18** tệp `.ts` = 29. Sàn đặt
 * dưới số thật một khoảng nhỏ để một lượt xoá tệp có chủ ý không làm cổng `abort()`,
 * nhưng một lượt quét hỏng thì có.
 */
// 🔴 NÂNG 2026-08-12 — Story 2.2 · AC16. Số thật là **50** tệp `src/**`, nên sàn 35 đã tụt
// xuống **70,0%**; ba story (1.20 · 1.21 · 2.1) thêm tệp mà không ai nâng sàn. Đo chứ không ước.
// 🔵 ĐẾM LẠI 2026-08-14 (Story 2.5b) — quần thể **không đổi**: gỡ ba tệp
// (`SourcePanel.vue` · `EditorPanel.vue` · `editorGutter.ts`), thêm ba
// (`GridPanel.vue` · `hanVietSurfaces.ts` · `segmentNavigation.ts`). Một lượt lật hình dạng
// **cân bằng theo số tệp** là chuyện tình cờ, không một mệnh đề — nên nó được đếm, không suy.
// 🔴 NÂNG 2026-08-18 (Story 2.12 · Task 7.5) — số thật lên **55**, nên sàn 43 tụt xuống
// **78,2%**, DƯỚI dải 80-85% mà `project-context.md` đặt. Ba story (2.5c · 2.5d · 2.10) thêm
// tệp mà không ai đếm lại; chú thích cũ còn ghi "52" trong khi `walk()` đếm được 55.
// ⚠️ Đây là hình dạng hỏng ÊM nhất của một sàn: nó KHÔNG đỏ oan bao giờ, nó chỉ lặng lẽ thôi
// canh. Một sàn dưới dải là một sàn đã tắt mà không ai biết — cùng lớp nợ với một miễn trừ
// hết cần mà ở lại.
// 🔴 Và đây chính là ràng buộc Ice ký kèm quyết định #7: *"cổng nào đọc `src/**` thì phải
// xét lại sàn quần thể"*. Đo chứ không ước: `find src -type f \( -name '*.ts' -o -name
// '*.vue' … \) | wc -l` = **55** (39 `.ts` + 16 `.vue`), 2026-08-18.
const FILE_FLOOR = 56 // 🔵 NÂNG 2026-08-22 (Story 3.6): số THẬT 66 tệp `src/**`
// (+glossaryConfirmStripState.ts +panels/inlineStripPriority.ts +GlossaryConfirmStrip.vue)
// — 56/66 = 84,8%
// (trước đó) 🔵 NÂNG 2026-08-22 (Story 3.5): số THẬT 63 tệp `src/**` (+glossarySettingsState.ts
// +GlossarySettingsOverlay.vue) — 52/63 = 82,5%

let files = []
try {
  files = walk(SRC_ROOT).sort()
} catch (err) {
  abort('cây nguồn `src/**`', err)
}
if (files.length < FILE_FLOOR) {
  abort(
    `cây nguồn \`src/**\` — chỉ ${files.length} tệp, dưới sàn ${FILE_FLOOR}`,
    new Error('Một danh sách rỗng làm Kiểm C xanh mà không quét gì cả.'),
  )
}

// ═════════════════════════════════════════════════════════════════════════════════
// CHE COMMENT, GIỮ NGUYÊN OFFSET
//
// ⚠️ Che chứ không xoá: mọi số dòng báo lỗi bên dưới tính từ offset trong văn bản gốc.
//
// 🔴 CHIỀU HỎNG CỦA CỔNG NÀY LÀ **CHE THỪA**, không phải che thiếu. Che thiếu ⇒ một
// comment bị đọc thành mã ⇒ một FAIL giả — ồn ào, nhìn thấy ngay, sửa được. Che thừa ⇒
// một `window.open` thật biến mất ⇒ exit 0 im lặng. Nên luật ở đây theo `check-tokens.mjs`:
// một chuỗi `'`/`"` phải đóng TRONG CÙNG MỘT DÒNG mới được coi là chuỗi; `/* */`, `` ` ``
// và `<!-- -->` phải có chỗ đóng. Không đóng ⇒ ký tự đó là ký tự thường, đi tiếp một bước.
//
// ⚠️ `text.split('')` chứ KHÔNG `[...text]`: spread đánh chỉ số theo CODE POINT trong
// khi mọi chỉ số nạp vào nó là UTF-16 — một emoji trong comment (tệp này có nhiều) làm
// lệch toàn bộ offset từ đó trở đi.
// ═════════════════════════════════════════════════════════════════════════════════
const blank = (chars, s, e) => {
  for (let i = s; i < e && i < chars.length; i += 1) if (chars[i] !== '\n') chars[i] = ' '
}

function maskComments(text) {
  const chars = text.split('')
  let i = 0
  while (i < text.length) {
    // Comment HTML của `<template>`. Đứng trước mọi thứ: `<!-- // -->` là một comment HTML.
    if (text.startsWith('<!--', i)) {
      const end = text.indexOf('-->', i + 4)
      if (end === -1) {
        i += 1
        continue
      }
      blank(chars, i, end + 3)
      i = end + 3
      continue
    }
    if (text.startsWith('/*', i)) {
      const end = text.indexOf('*/', i + 2)
      if (end === -1) {
        i += 1
        continue
      }
      blank(chars, i, end + 2)
      i = end + 2
      continue
    }
    if (text.startsWith('//', i)) {
      const end = text.indexOf('\n', i)
      const stop = end === -1 ? text.length : end
      blank(chars, i, stop)
      i = stop
      continue
    }
    // Chuỗi và template: KHÔNG che nội dung (Kiểm C phải đọc được `'localStorage'` viết
    // trong một chuỗi — đó vẫn là một lượt chạm tới kho lưu trữ qua `globalThis[...]`).
    // Chỉ NHẢY QUA chúng để một `//` bên trong dấu nháy không mở một comment giả.
    const ch = text[i]
    if (ch === "'" || ch === '"') {
      const nl = text.indexOf('\n', i + 1)
      const limit = nl === -1 ? text.length : nl
      let j = i + 1
      while (j < limit && text[j] !== ch) j += text[j] === '\\' ? 2 : 1
      // Không đóng trong cùng dòng ⇒ đây là một dấu nháy trong văn xuôi, không phải chuỗi.
      i = j < limit && text[j] === ch ? j + 1 : i + 1
      continue
    }
    if (ch === '`') {
      let j = i + 1
      while (j < text.length && text[j] !== '`') j += text[j] === '\\' ? 2 : 1
      i = j < text.length ? j + 1 : i + 1
      continue
    }
    i += 1
  }
  return chars.join('')
}

const lineOf = (text, index) => text.slice(0, index).split('\n').length

const sources = files.map((file) => {
  let text
  try {
    text = readFileSync(file, 'utf8')
  } catch (err) {
    abort(`tệp \`${posix(file)}\``, err)
  }
  return { file, text, masked: maskComments(text) }
})

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm A — thứ tự hy sinh của UX-DR15 (AC7)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// Ba mệnh đề dưới đây là **QUYẾT ĐỊNH**, không phải số hiệu chỉnh được. Bốn ngưỡng
// kích thước màn hình là **Story 4.12**, và UX-DR15 cấm tường minh việc đóng chúng
// ở story này. Cổng này canh CƠ CHẾ, không canh ngưỡng.

const layoutMod = await import(pathToFileURL(join(SRC_ROOT, 'layout', 'workspaceLayout.ts')).href).catch(
  (err) => abort('`src/layout/workspaceLayout.ts` — Kiểm A KHÔNG chạy được', err),
)

for (const name of ['PANEL_IDS', 'SACRIFICE_ORDER', 'NEVER_SACRIFICED', 'nextToSacrifice']) {
  if (layoutMod[name] === undefined) {
    abort('`src/layout/workspaceLayout.ts`', new Error(`không export \`${name}\`.`))
  }
}

const { PANEL_IDS, SACRIFICE_ORDER, NEVER_SACRIFICED, nextToSacrifice, nextToRestore } = layoutMod

// Mệnh đề 1 — hai tập RỜI NHAU và hợp lại đúng `PANEL_IDS`.
//
// 🔵 2026-08-14 (Story 2.5b): tiêu đề cũ viết "đúng BỐN panel" — con số đó đã hết đúng
// (bốn → ba). Phép kiểm thì **không đổi một dòng**: nó luôn so với `PANEL_IDS` thật, nên
// nó là một trong số ít chỗ của lượt lật này KHÔNG phải sửa logic.
{
  const overlap = SACRIFICE_ORDER.filter((id) => NEVER_SACRIFICED.includes(id))
  const union = [...new Set([...SACRIFICE_ORDER, ...NEVER_SACRIFICED])].sort()
  const all = [...PANEL_IDS].sort()
  if (overlap.length > 0) {
    fail(`\`SACRIFICE_ORDER\` và \`NEVER_SACRIFICED\` giao nhau ở: ${overlap.join(', ')}`)
    detail('Một panel vừa được phép nhường vừa không bao giờ nhường là một mệnh đề vô nghĩa.')
  } else if (union.join('|') !== all.join('|')) {
    fail(`hai tập hợp lại là [${union.join(', ')}], nhưng \`PANEL_IDS\` là [${all.join(', ')}]`)
    detail('Một panel không thuộc tập nào thì UX-DR15 KHÔNG nói gì về nó — và Story 4.12 sẽ đoán.')
  } else {
    pass(`hai tập rời nhau và hợp lại đúng ${all.length} panel`)
  }
}

// Mệnh đề 2 — mọi phần tử của `NEVER_SACRIFICED` KHÔNG BAO GIỜ là đầu ra.
//
// ⚠️ Duyệt TOÀN BỘ `2^n` tập con, không chỉ vài ca lấy mẫu: mệnh đề là
// *"không bao giờ"*, và một phép kiểm lấy mẫu chứng minh được ít hơn hẳn thứ nó tuyên bố.
{
  const bad = []
  const n = PANEL_IDS.length
  for (let mask = 0; mask < 1 << n; mask += 1) {
    const visible = PANEL_IDS.filter((_, i) => (mask >> i) & 1)
    const out = nextToSacrifice(visible)
    if (out !== null && NEVER_SACRIFICED.includes(out)) bad.push(`[${visible.join(', ')}] ⇒ ${out}`)
  }
  if (bad.length > 0) {
    fail(`\`nextToSacrifice\` trả về một panel KHÔNG BAO GIỜ được nhường: ${bad.join(' · ')}`)
    detail('UX-DR15: cặp `Nguyên văn | Bản dịch` không bao giờ nhường. Đó là một quyết định.')
  } else {
    pass(`${1 << n} tập con: \`nextToSacrifice\` không bao giờ trả về ${NEVER_SACRIFICED.join(' hay ')}`)
  }
}

// Mệnh đề 3 — `panel.ai_translation` đứng TRƯỚC `panel.lookup`.
{
  const ai = SACRIFICE_ORDER.indexOf('panel.ai_translation')
  const lookup = SACRIFICE_ORDER.indexOf('panel.lookup')
  if (ai === -1 || lookup === -1) {
    fail(`thứ tự hy sinh thiếu một trong hai panel — đang là [${SACRIFICE_ORDER.join(', ')}]`)
  } else if (ai >= lookup) {
    fail('`panel.ai_translation` phải nhường TRƯỚC `panel.lookup` (UX-DR15)')
  } else if (nextToSacrifice([...PANEL_IDS]) !== 'panel.ai_translation') {
    fail(`đủ ba panel ⇒ cái nhường đầu tiên phải là \`panel.ai_translation\``)
  } else if (nextToSacrifice(['panel.grid', 'panel.lookup']) !== 'panel.lookup') {
    fail('sau khi `panel.ai_translation` đã nhường, cái kế tiếp phải là `panel.lookup`')
  } else if (nextToSacrifice(['panel.grid']) !== null) {
    fail('chỉ còn `panel.grid` ⇒ phải trả `null`, không phải hy sinh nó')
  } else {
    pass('`panel.ai_translation` nhường trước · `panel.lookup` nhường sau · `panel.grid` ⇒ `null`')
  }
}

// Nghịch đảo: trả panel về theo thứ tự NGƯỢC. Không có nó thì một lượt nới cửa sổ trả
// `panel.ai_translation` về trước `panel.lookup`, tức đảo đúng ưu tiên vừa phát biểu.
if (typeof nextToRestore === 'function') {
  const a = nextToRestore(['panel.grid'])
  const b = nextToRestore(['panel.grid', 'panel.lookup'])
  if (a !== 'panel.lookup' || b !== 'panel.ai_translation') {
    fail(`\`nextToRestore\` sai thứ tự — nhận được \`${a}\` rồi \`${b}\``)
    detail('Cái nhường SAU CÙNG phải được lấy lại TRƯỚC.')
  } else {
    pass('`nextToRestore` là nghịch đảo đúng: `panel.lookup` về trước, `panel.ai_translation` về sau')
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm B — nhịp ghi bố cục: idle + TRẦN CỨNG không reset (AC4)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 §Bẫy 3 của story: ghi một lượt `putConfig` ở mỗi `onDidLayoutChange` thì một cú kéo
// sash 3 giây là hàng trăm job xếp hàng qua `store::Writer` nối tiếp — đúng thứ AD-11/AD-12
// tồn tại để chặn, và không cổng nào đỏ. Nay có một, và nó ĐẾM.

const scheduleMod = await import(pathToFileURL(join(SRC_ROOT, 'layout', 'writeSchedule.ts')).href).catch(
  (err) => abort('`src/layout/writeSchedule.ts` — Kiểm B KHÔNG chạy được', err),
)
const { IDLE_MS, HARD_CAP_MS, simulateWrites, createWriteSchedule } = scheduleMod

{
  // Một cú kéo sash 3 giây: sự kiện mỗi 16 ms (≈ một frame) — 188 sự kiện.
  const drag = []
  for (let t = 0; t <= 3000; t += 16) drag.push(t)
  const writes = simulateWrites(drag)
  // Trần 5 s ⇒ trong 3 s không mốc trần nào tới; idle 500 ms ⇒ đúng MỘT lượt ghi, sau
  // khi người dùng buông tay.
  if (writes.length !== 1) {
    fail(`kéo sash 3 s (${drag.length} sự kiện) ⇒ ${writes.length} lượt ghi, phải là 1`)
    detail(`Mốc ghi: ${writes.join(', ')}. Một lượt ghi mỗi sự kiện là §Bẫy 3 của Story 1.14.`)
  } else if (writes[0] !== drag[drag.length - 1] + IDLE_MS) {
    fail(
      `lượt ghi phải rơi vào ${drag[drag.length - 1] + IDLE_MS} ms (sự kiện CUỐI + idle), ` +
        `đang là ${writes[0]}`,
    )
  } else {
    pass(`kéo sash 3 s · ${drag.length} sự kiện ⇒ ĐÚNG 1 lượt ghi, ở ${writes[0]} ms`)
  }
}

{
  // 🔴 TRẦN CỨNG. Kéo LIÊN TỤC 20 giây — một debounce thuần cho ra 0 lượt ghi, tức mất
  // trọn 20 giây thao tác nếu máy tắt đột ngột.
  const long = []
  for (let t = 0; t <= 20000; t += 16) long.push(t)
  const writes = simulateWrites(long)
  const expected = Math.floor(20000 / HARD_CAP_MS)
  if (writes.length < expected) {
    fail(`kéo liên tục 20 s ⇒ chỉ ${writes.length} lượt ghi, trần ${HARD_CAP_MS} ms đòi ít nhất ${expected}`)
    detail('Trần bị RESET bởi sự kiện kế tiếp — đó là một debounce thuần, không phải trần cứng.')
  } else if (writes.length > expected + 1) {
    fail(`kéo liên tục 20 s ⇒ ${writes.length} lượt ghi, nhiều hơn cần thiết (${expected}+1)`)
  } else {
    /**
     * 🔴 BẤT BIẾN ĐÚNG LÀ **TUỔI CỦA MỘT THAY ĐỔI CHƯA GHI**, KHÔNG phải khoảng cách
     * giữa hai lượt ghi — và lượt dựng cổng này bắt được đúng chỗ đó.
     *
     * Bản đầu khẳng định *"khoảng cách giữa hai lượt ghi ≤ trần"* và cổng đỏ với `5008 ms`.
     * Con số đó ĐÚNG và mệnh đề thì sai: trần nổ ở mốc 5000, còn chu kỳ kế tiếp chỉ bắt
     * đầu ở **sự kiện tiếp theo** (5008) chứ không phải ở chính mốc 5000 — giữa hai mốc
     * đó không có gì chưa ghi cả. Thứ người dùng mất khi máy tắt đột ngột là *"thay đổi
     * cũ nhất còn chưa chạm đĩa"*, và đó mới là thứ trần cứng hứa chặn.
     *
     * ⚠️ Giữ lại con số 5008 trong comment này có chủ ý: một cổng chỉ đo được thứ nó phát
     * biểu, và ghi ra lần phát biểu sai là cách người sau không "sửa" nó ngược lại.
     */
    let worstStaleness = 0
    let cursor = 0
    for (const at of long) {
      while (cursor < writes.length && writes[cursor] < at) cursor += 1
      if (cursor >= writes.length) {
        fail(`sự kiện ở ${at} ms KHÔNG bao giờ được ghi — một thay đổi mất hẳn`)
        worstStaleness = Number.POSITIVE_INFINITY
        break
      }
      worstStaleness = Math.max(worstStaleness, writes[cursor] - at)
    }
    if (worstStaleness > HARD_CAP_MS) {
      fail(
        `thay đổi cũ nhất chưa ghi đạt tới ${worstStaleness} ms, vượt trần ${HARD_CAP_MS} ms — ` +
          'trần đang BỊ RESET bởi sự kiện kế tiếp, tức đây là một debounce thuần',
      )
    } else {
      pass(
        `kéo liên tục 20 s · ${long.length} sự kiện ⇒ ${writes.length} lượt ghi; không thay ` +
          `đổi nào chờ quá ${worstStaleness} ms (≤ trần ${HARD_CAP_MS} ms) — trần KHÔNG bị reset`,
      )
    }
  }
}

{
  // Sạch thì không ghi. Một lượt rời chế độ khi không có gì đổi phải là một no-op —
  // nếu không, mỗi lần bấm qua lại giữa ba chế độ là một lượt chạm đĩa.
  const s = createWriteSchedule()
  if (s.isDirty() || s.deadline() !== null) {
    fail('lịch ghi mới dựng đã "bẩn" — một lượt rời chế độ sẽ ghi khi không có gì đổi')
  } else {
    s.onChange(1000)
    if (!s.isDirty()) fail('`onChange` không đánh dấu bẩn — lượt ghi lúc rời chế độ sẽ bị bỏ')
    s.onWrite(1500)
    if (s.isDirty() || s.deadline() !== null) fail('`onWrite` không dọn trạng thái')
    else pass('sạch ⇒ không ghi · một thay đổi ⇒ bẩn · sau khi ghi ⇒ sạch lại')
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm C — bề mặt CẤM: không cửa sổ OS thứ hai, không kho lưu trữ thứ hai (AC1, AC12)')
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * 🔴 DANH SÁCH **CHO PHÉP** cho mọi thành viên của `window` / `document`.
 *
 * Thêm một mục vào đây là một quyết định phải viết ra. `window.open` không có mặt, và
 * đó chính là mệnh đề của AD-24: **một cửa sổ hệ điều hành**.
 */
const ALLOWED_GLOBAL_MEMBERS = new Set([
  // Bàn phím toàn ứng dụng (FR22) và lượt ghi bố cục cuối cùng (AC4).
  'window.addEventListener',
  'window.removeEventListener',
  // Story 1.17, Quyết định #1a — vùng chọn cho `lookup.lookup_selection` (dep TỐI THIỂU,
  // không hợp đồng vùng chọn dùng chung của Story 1.18). API DOM chuẩn, không mở cửa sổ/kho thứ
  // hai — AC1/AC12 của story này canh đúng hai thứ đó, không canh API đọc vùng chọn văn bản.
  'window.getSelection',
  // Story 1.18, AC11 — `document.createRange()` dựng một `Range` rỗng ở đầu bề mặt chữ để
  // `Selection.modify()` có chỗ bám (`selectionContract.ts::focusSelectionSource`). Đó là
  // đường DUY NHẤT đóng được `deferred-work.md §*Deferred from: 1-13-duong-tra-cuu-giu-nguyen-bat-dong-giua-cac-nguon (2026-08-05)*` (bôi đen bằng bàn phím) mà không phải bật
  // caret browsing (không bật được bằng mã) hay `contenteditable` (AD-1: nguyên văn là dữ liệu
  // không sửa được). API DOM chuẩn, không mở cửa sổ/kho thứ hai — AC1/AC12 canh đúng hai thứ đó.
  'document.createRange',
  // Token ghi lên `:root` (Story 1.4) · sổ điểm vào focus (AD-34 §2) · hộp chẩn đoán khởi
  // động (`main.ts`) · nạp font (Story 1.4).
  'document.documentElement',
  'document.activeElement',
  'document.body',
  'document.fonts',
  'document.getElementById',
  'document.createElement',
  'document.addEventListener',
  'document.removeEventListener',
  // Story 1.19, AC11 · UX-DR17 (thêm ở code review 2026-08-10) — `AttributionOverlay.vue`
  // tìm lại nút ĐÃ MỞ lớp phủ (`[data-attribution-open]`) để trả tiêu điểm về khi node giữ
  // tiêu điểm lúc mở đã rời DOM. Một `ref` không dùng được: nút sống trong `LookupPanel.vue`,
  // một component KHÁC, và một lượt đổi preset bố cục dựng lại cả ba panel. API DOM chuẩn,
  // không mở cửa sổ/kho thứ hai — AC1/AC12 canh đúng hai thứ đó.
  'document.querySelector',
  // Story 2.3, AC8 — `GridPanel.vue::onBeforeInput` chèn văn bản THUẦN đã làm phẳng sau khi
  // `preventDefault()` một lượt dán/kéo-thả. Đó là đường DUY NHẤT giữ được mệnh đề của AD-37
  // (*cấu trúc đoạn là dữ liệu ĐÃ LƯU*), và nó là một PHÉP ĐO chứ không một lượt phòng xa: mũi
  // thăm dò Task 0.1 đo được rằng không có nhánh này thì **cả hai** engine tiêm markup vào
  // trong một câu — `<pre>`, `<span style>`, và trên WebKit cả `<div>` khối — cộng một `\n`
  // thật vào `target_text` của MỘT câu. `contenteditable="plaintext-only"` KHÔNG thay được:
  // Blink vẫn giữ `\n`, WebKit tự tạo `<div>`.
  // API DOM chuẩn, không mở cửa sổ/kho thứ hai — AC1/AC12 canh đúng hai thứ đó.
  'document.createTextNode',
  // Story 2.3, AC22 — `GridPanel.vue::placeCaretAtPoint`. ĐÂY LÀ MỘT PHÉP ĐO, không một lượt
  // phòng xa: đo trong cửa sổ Tauri thật (WKWebView 605.1.15, chuột THẬT) rằng một cú bấm vào
  // văn bản chỉ-đọc cho `getSelection().type === 'None'`, `rangeCount = 0`, và `.doc` KHÔNG nhận
  // tiêu điểm dù có `tabindex="0"`. Đường `Selection.anchorNode` của Story 2.2 vì thế không bao
  // giờ chạy tới trên engine mà sản phẩm thật sự chạy, và người dùng bấm vào một câu thì KHÔNG
  // GÌ xảy ra. Hai API này là đường duy nhất phân giải điểm bấm thành một vị trí caret.
  // API DOM chuẩn, không mở cửa sổ/kho thứ hai — AC1/AC12 canh đúng hai thứ đó.
  'document.caretPositionFromPoint',
  'document.caretRangeFromPoint',
  // Story 2.8, AC2 — `GridPanel.vue::onSourceCellMouseUp` đổi một điểm bấm ở CỘT NGUYÊN VĂN
  // thành một **chỉ số ký tự trong `source_text`**, tức chỗ cắt của `⌘/`.
  //
  // 🔴 Vì sao phải duyệt text node thay vì lấy `offset` trần: một ô nguyên văn chứa NHIỀU text
  // node *(hai `#comment` `aura-allow-text` chia chuỗi làm ba — đo trên WKWebView thật ngày
  // 2026-08-17, `childNodes = [COMMENT, TEXT(0), COMMENT, TEXT(40), TEXT(0)]`)*, nên
  // `anchorOffset` là offset **trong một node**, không phải trong ô. Hôm nay tổng độ dài các
  // node đứng trước là **0** nên hai cách cho cùng kết quả; bật Hán Việt lên thì ô mang thêm
  // `<ruby>`/`<rt>` và phép lấy trần **sai im lặng** — cắt nhầm chỗ trên dữ liệu người dùng,
  // mà AD-5 không cho hoàn tác.
  //
  // API DOM chuẩn, chỉ ĐỌC, không mở cửa sổ/kho thứ hai — AC1/AC12 canh đúng hai thứ đó.
  'document.createTreeWalker',
  // Story 2.5d, AC1 · AC6 — `GridPanel.vue::onBeforeInput` chặn `insertParagraph` rồi phát
  // `insertLineBreak` để engine tự dựng lượt xuống dòng. ĐÂY LÀ MỘT PHÉP ĐO trên WKWebView
  // 605.1.15 thật (bàn đo Task 1, 2026-08-15), không một lượt tiện tay:
  //   • thả `insertParagraph` chạy ⇒ engine dựng `A<div>B</div>` và `cell.textContent` đọc
  //     ra `"AB"` — DOM hai dòng, đĩa MỘT chuỗi. AC1 hỏng mà không cổng nào đỏ;
  //   • `insertLineBreak` dưới `white-space: pre-line` ⇒ text node `"\n"`, 0 phần tử con,
  //     `textContent === "A\nB"`, và ở cuối nội dung engine tự thêm `\n` canh chót;
  //   • đường tự chèn bằng `Range` (không cần API này) cũng ĐẠT, nhưng phải tự xử lý ca
  //     cuối-nội-dung mà engine đang làm hộ — Ice ký đường engine 2026-08-16.
  // ⚠️ `execCommand` là API bị khai tử trong đặc tả, và điều đó được cân nhắc: nó là đường
  // DUY NHẤT gọi được lượt soạn thảo gốc của engine kèm undo-stack thật. Không có nó, mọi
  // lượt sửa phải tự dựng bằng `Range`, và `⌘Z` của người dùng mất lịch sử.
  // API DOM chuẩn, không mở cửa sổ/kho thứ hai — AC1/AC12 canh đúng hai thứ đó.
  'document.execCommand',
  // Story 4.12, Task 3-4 (Phase 2) — `WorkspaceDock.vue::computeWorkArea` đọc kích thước cửa
  // sổ THẬT để tính tầng bố cục qua `layoutTierFor`. ĐÂY LÀ LƯỢT ĐỌC `window.inner*` ĐẦU
  // TIÊN trong `src/**` — Story 1.14 · AC1 giữ mệnh đề "không một chỗ đọc kích thước cửa sổ
  // nào tồn tại" suốt tới tận đây, và spec 4.12 §Always nói đường kết thúc mệnh đề đó phải đi
  // qua CHÍNH danh sách cho phép này, không phải một `ResizeObserver` — thứ Kiểm C này không
  // thấy được (`workspaceLayout.ts` §"vì sao window reads rather than ResizeObserver" giải
  // thích đầy đủ). API DOM chuẩn, chỉ ĐỌC, không mở cửa sổ/kho thứ hai — AC1/AC12 canh đúng
  // hai thứ đó.
  'window.innerWidth',
  'window.innerHeight',
  // Cùng chỗ, cùng lý do — `getComputedStyle(document.documentElement)` đọc hai token chrome
  // do `tokens/index.ts::applyTheme` ghi LÚC CHẠY (`--space-titlebar-height` ·
  // `--space-status-height`, Story 1.4), vì spec 4.12 §Always cấm viết hai con số đó thành
  // literal ở `computeWorkArea`. Trả về một `CSSStyleDeclaration` CHỈ-ĐỌC; không mở cửa
  // sổ/kho thứ hai — AC1/AC12 canh đúng hai thứ đó.
  'window.getComputedStyle',
])

const GLOBAL_MEMBER_RE = /\b(window|document|globalThis|self|top|parent)\s*\.\s*([A-Za-z_$][A-Za-z0-9_$]*)/g

/**
 * ⚠️ Truy cập bằng CHỈ SỐ (`window['open']`, `globalThis[x]`) không đọc tĩnh được thành
 * một tên. Nó bị đếm và IN RA — cùng kỷ luật với `nonLiteralOwnerCalls` của
 * `check-commands.mjs`. Một con số khác 0 ở đây là chỗ người rà soát phải nhìn bằng mắt.
 */
const GLOBAL_INDEX_RE = /\b(window|document|globalThis|self|top|parent)\s*\[/g

/**
 * Mệnh đề CẤM **hẹp**, và mỗi cái gắn với một sự thật đã ĐO, không phải một danh sách
 * dài những thứ nghe có vẻ nguy hiểm:
 *
 *   `addPopoutGroup` — đo trên `dockview-core/dist/package/main.esm.mjs`: đường DUY NHẤT
 *     trong thư viện gọi `window.open`, và đường DUY NHẤT tạo `<style>` lúc chạy. ⇒ cửa sổ
 *     OS thứ hai (vi phạm AD-24) và một lượt đụng CSP `style-src 'self'`.
 *   `localStorage` · `sessionStorage` — `kinds.rs:212` gọi tên chúng là đường SAI cho bố
 *     cục. Chúng là **định danh tự do**, nên chúng không đóng được bằng danh sách cho
 *     phép ở trên; giới hạn đó ghi ở đầu tệp.
 *   `document.write` — nó không có trong danh sách cho phép nên đã bị chặn; không lặp lại.
 */
const NARROW_BANS = [
  ['addPopoutGroup', 'cửa sổ OS thứ hai (AD-24) + `<style>` lúc chạy (CSP `style-src \'self\'`)'],
  ['localStorage', 'kho lưu trữ thứ hai — bố cục đi qua `putConfig` → `store::Writer` (AD-11)'],
  ['sessionStorage', 'cùng lý do với `localStorage`'],
]

let cBad = 0
let indexedAccess = 0
const seenMembers = new Map()

for (const s of sources) {
  const rel = posix(s.file)
  let m
  const member = new RegExp(GLOBAL_MEMBER_RE.source, 'g')
  while ((m = member.exec(s.masked))) {
    const name = `${m[1]}.${m[2]}`
    const where = `${rel}:${lineOf(s.text, m.index)}`
    if (!seenMembers.has(name)) seenMembers.set(name, where)
    if (!ALLOWED_GLOBAL_MEMBERS.has(name)) {
      fail(`${where} — \`${name}\` KHÔNG có trong danh sách cho phép`)
      detail('Nếu đây là một nhu cầu thật: thêm nó vào `ALLOWED_GLOBAL_MEMBERS` KÈM một dòng')
      detail('nói nó phục vụ AC nào. Đừng nới regex, và đừng bỏ tệp ra khỏi tầm quét.')
      cBad += 1
    }
  }
  const indexed = new RegExp(GLOBAL_INDEX_RE.source, 'g')
  while ((m = indexed.exec(s.masked))) indexedAccess += 1

  for (const [needle, why] of NARROW_BANS) {
    let at = s.masked.indexOf(needle)
    while (at !== -1) {
      fail(`${rel}:${lineOf(s.text, at)} — \`${needle}\` bị cấm: ${why}`)
      cBad += 1
      at = s.masked.indexOf(needle, at + needle.length)
    }
  }
}

if (cBad === 0) {
  pass(
    `${seenMembers.size} thành viên \`window\`/\`document\` được chạm tới trên ${files.length} tệp — ` +
      'tất cả đều trong danh sách cho phép',
  )
  pass(`không \`${NARROW_BANS.map(([n]) => n).join('\` · \`')}\` ở bất kỳ đâu trong \`src/**\``)
  if (indexedAccess > 0) {
    detail(
      `⚠️ ${indexedAccess} lượt truy cập global bằng CHỈ SỐ (\`window[…]\`) — không đọc ` +
        'tĩnh được thành tên. Người rà soát phải nhìn chúng bằng mắt.',
    )
  } else {
    pass('không lượt truy cập global nào bằng chỉ số — danh sách cho phép không có chỗ mù')
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm D — TỰ KIỂM: chứng minh Kiểm C đỏ được, và không đỏ oan')
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 *"Một cổng chưa từng đỏ là một cổng chưa từng canh."* Khuôn: Task 3 của Story 1.4,
// Task 10 của Story 1.6. Khác biệt ở đây: bảng ca chạy **mỗi lượt CI**, không phải một
// lượt chạy tay được chép vào §Debug Log References rồi thôi đúng trong im lặng.

/** Chạy lại đúng logic quét của Kiểm C trên một mẩu mã, trả về số vi phạm. */
function scanFragment(text) {
  const masked = maskComments(text)
  let hits = 0
  let m
  const member = new RegExp(GLOBAL_MEMBER_RE.source, 'g')
  while ((m = member.exec(masked))) {
    if (!ALLOWED_GLOBAL_MEMBERS.has(`${m[1]}.${m[2]}`)) hits += 1
  }
  for (const [needle] of NARROW_BANS) {
    let at = masked.indexOf(needle)
    while (at !== -1) {
      hits += 1
      at = masked.indexOf(needle, at + needle.length)
    }
  }
  return hits
}

/** [tên ca, mã, có phải vi phạm không] */
const CASES = [
  ['popout của dockview', 'api.addPopoutGroup({ position: {} })', true],
  ['cửa sổ thứ hai bằng tay', 'window.open("/x")', true],
  ['cửa sổ thứ hai qua `globalThis`', 'globalThis.open("/x")', true],
  ['cửa sổ thứ hai qua `self`', 'self.open("/x")', true],
  ['kho thứ hai', 'localStorage.setItem("layout", j)', true],
  ['kho thứ hai, phiên', 'sessionStorage.setItem("layout", j)', true],
  ['kho thứ hai qua `window.`', 'window.localStorage.clear()', true],
  ['`document.write`', 'document.write("<b>x</b>")', true],
  ['`document.cookie`', 'document.cookie = "a=b"', true],
  ['khoảng trắng chen giữa', 'window . open ( "/x" )', true],
  // ── Đối chứng ÂM: những thứ KHÔNG được đỏ ──────────────────────────────────────
  ['comment dòng nhắc tên', '// đường duy nhất là addPopoutGroup — đừng gọi', false],
  ['comment khối nhắc tên', '/* localStorage bị cấm ở đây */', false],
  ['comment HTML nhắc tên', '<!-- không window.open -->', false],
  ['dấu nháy lẻ trong văn xuôi', "// don't call window.open\nconst a = 1", false],
  ['thành viên hợp lệ', 'window.addEventListener("keydown", h)', false],
  ['thành viên hợp lệ #2', 'document.activeElement === document.body', false],
  ['tên chỉ GIỐNG chứ không phải', 'const localStorageNote = 1', true],
]

let dBad = 0
for (const [name, code, shouldFail] of CASES) {
  const hits = scanFragment(code)
  const caught = hits > 0
  if (caught !== shouldFail) {
    fail(`tự kiểm — ca \`${name}\`: mong ${shouldFail ? 'ĐỎ' : 'XANH'}, nhận ${caught ? 'ĐỎ' : 'XANH'}`)
    dBad += 1
  }
}
if (dBad === 0) {
  const red = CASES.filter(([, , f]) => f).length
  pass(`${CASES.length} ca tự kiểm — ${red} ca ĐỎ đúng, ${CASES.length - red} đối chứng âm XANH đúng`)
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm E — bốn ngưỡng bố cục theo preset (Story 4.12, Task 1-2)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// Gọi thẳng `layoutTierFor()` thật từ `layoutMod` (đã `import()` ở Kiểm A) — không một
// bản chép logic. Bốn mệnh đề, đúng thứ Task 2 của Phase 1 giao:
//   1. TOÀN PHẦN, HAI VẾ — mọi (width, height, preset) HỮU HẠN trả về một tầng hợp lệ,
//      không ném lỗi; mọi (width, height) có ít nhất một chiều KHÔNG hữu hạn (`NaN`,
//      `±Infinity`, `undefined`) trả ĐÚNG `null`, không bao giờ một chuỗi tầng.
//   2. BIÊN THEO TỪNG PRESET — mỗi ngưỡng đóng cả hai phía (đúng số ⇒ tầng cao hơn; kém
//      1 ⇒ tầng thấp hơn), số kỳ vọng ghim RIÊNG cho Ⓑ-1 và Ⓑ-2 để một lượt hiệu chỉnh
//      chỉ đổi MỘT preset không làm phép kiểm đọc sai preset kia.
//   3. ƯU TIÊN TRÊN-XUỐNG — khi chiều rộng đã `unsupported`, nó thắng cả một chiều cao
//      lẽ ra rơi vào `narrow`; khi chiều cao quá thấp, nó thắng cả một chiều rộng đủ cho
//      `full`/`short`.
//   4. `NEVER_SACRIFICED` KHÔNG BỊ ĐỤNG, HAI TỪ VỰNG RỜI NHAU — không chứng minh "lưới
//      còn hiện ở mọi tầng"; thang bậc thuần không biết panel nào đang mount. Mệnh đề
//      đó là Phase 4, trên `WorkspaceDock.vue` thật.

for (const name of ['LAYOUT_TIERS', 'LAYOUT_THRESHOLDS', 'layoutTierFor']) {
  if (layoutMod[name] === undefined) {
    abort('`src/layout/workspaceLayout.ts`', new Error(`không export \`${name}\` — Kiểm E cần nó.`))
  }
}
const { LAYOUT_TIERS, LAYOUT_THRESHOLDS, layoutTierFor } = layoutMod

const PRESET_IDS = ['layout.preset_grid', 'layout.preset_columns']
for (const id of PRESET_IDS) {
  if (LAYOUT_THRESHOLDS[id] === undefined) {
    abort('`src/layout/workspaceLayout.ts`', new Error(`\`LAYOUT_THRESHOLDS\` thiếu khoá \`${id}\`.`))
  }
}

// Mệnh đề 1 — TOÀN PHẦN, HAI VẾ.
//
// 🔴 Vế thứ hai tồn tại vì một cái bẫy ĐO ĐƯỢC, không phải phòng xa: mọi phép so sánh
// với `NaN` trả `false`, nên một chuỗi `if (x < ngưỡng)` viết ngây thơ rơi qua HẾT bốn
// nhánh và trả `'full'` — tức chính lượt "không đo được kích thước" lại báo bố cục
// THOẢI MÁI NHẤT. Phase 2 đọc `getComputedStyle(...)` rồi `parseFloat`; một token rỗng
// cho `NaN` theo đúng cách này. `layoutTierFor` phải trả `null` — *"không biết"*, không
// phải *"đủ chỗ"* — và Kiểm E ghim cả hai vế để không ai âm thầm quay lại `'full'`.
{
  const widths = [-100, 0, 1, 500, 837.5, 859, 860, 861, 1099, 1100, 1101, 1500, 5000, 100000]
  const heights = [-100, 0, 1, 500, 650.25, 699, 700, 701, 819, 820, 821, 1000, 5000, 100000]
  const known = new Set(LAYOUT_TIERS)
  let checked = 0
  let bad = 0
  for (const id of PRESET_IDS) {
    for (const width of widths) {
      for (const height of heights) {
        checked += 1
        let out
        try {
          out = layoutTierFor({ width, height }, id)
        } catch (err) {
          fail(`toàn phần (hữu hạn) — (${width}×${height}, ${id}) NÉM LỖI: ${err?.message || err}`)
          bad += 1
          continue
        }
        if (typeof out !== 'string' || !known.has(out)) {
          fail(`toàn phần (hữu hạn) — (${width}×${height}, ${id}) trả về \`${String(out)}\`, không thuộc bốn tầng`)
          bad += 1
        }
      }
    }
  }

  // Vế KHÔNG HỮU HẠN — mọi tổ hợp có ít nhất một chiều không phải số hữu hạn phải trả
  // ĐÚNG `null`, không phải một chuỗi tầng nào (kể cả `'full'`).
  const nonFinite = [NaN, Infinity, -Infinity, undefined]
  const FIXED = 999 // một chiều "bình thường" để cô lập chiều đang bị làm hỏng
  let checkedNonFinite = 0
  for (const id of PRESET_IDS) {
    for (const bad_ of nonFinite) {
      const cases = [
        ['width', { width: bad_, height: FIXED }],
        ['height', { width: FIXED, height: bad_ }],
        ['cả hai', { width: bad_, height: bad_ }],
      ]
      for (const [label, workArea] of cases) {
        checkedNonFinite += 1
        let out
        try {
          out = layoutTierFor(workArea, id)
        } catch (err) {
          fail(`toàn phần (không hữu hạn) — ${label}=\`${String(bad_)}\` (${id}) NÉM LỖI: ${err?.message || err}`)
          bad += 1
          continue
        }
        if (out !== null) {
          fail(
            `toàn phần (không hữu hạn) — ${label}=\`${String(bad_)}\` (${id}) trả về \`${String(out)}\`, ` +
              'phải là `null` — không đo được KHÔNG được đọc thành một tầng, nhất là không phải `full`',
          )
          bad += 1
        }
      }
    }
  }

  if (bad === 0) {
    pass(
      `toàn phần — ${checked} tổ hợp hữu hạn luôn ra một trong bốn tầng; ` +
        `${checkedNonFinite} tổ hợp không hữu hạn (NaN/±Infinity/undefined) luôn ra \`null\``,
    )
  }
}

// Mệnh đề 2 — BIÊN, đóng cả hai phía, GHIM RIÊNG THEO PRESET. Số kỳ vọng ở đây ĐỘC LẬP
// với `LAYOUT_THRESHOLDS` (không đọc lại từ module) — nếu Task 1 (hoặc một lượt hiệu
// chỉnh sau này) dời một ngưỡng mà quên sửa cổng, phép kiểm này đỏ và NÊU TÊN đúng
// ngưỡng VÀ đúng preset đã dời.
//
// 🔴 Khoá theo `PresetId`, giống hệt cách `LAYOUT_THRESHOLDS` khoá — KHÔNG một `SEED`
// dùng chung lặp qua cả hai preset. Spec và UX-DR15 đòi Ⓑ-1/Ⓑ-2 mang hai bộ số RIÊNG
// sau khi hiệu chỉnh (Phase 5); một `SEED` chung sẽ ĐÚNG hôm nay (cả hai preset cùng hạt
// giống) nhưng SAI HÌNH DẠNG ngay khi Phase 5 dời một preset — nó sẽ đỏ cả hai preset dù
// chỉ một preset đổi, hoặc tệ hơn là đọc số của preset kia. Khoá theo preset thì một lượt
// hiệu chỉnh chỉ làm đỏ đúng preset đó, và chỉ hàng ứng với đúng ngưỡng đã dời.
//
// 🔴 Đây chính là móc cho counter-check 1 của Phase 4: dời MỘT ngưỡng của MỘT preset —
// hoặc số biên (khớp) hoặc số kém-1 (lệch) sẽ lệch tầng mong đợi, ở CẢ HAI HƯỚNG dời,
// và chỉ ở preset đó.
{
  const EXPECTED = {
    'layout.preset_grid': { minFullWidth: 1100, minFullHeight: 820, minShortHeight: 700, minSupportedWidth: 860 },
    'layout.preset_columns': { minFullWidth: 1100, minFullHeight: 820, minShortHeight: 700, minSupportedWidth: 860 },
  }
  let bad = 0
  const expect = (label, id, width, height, wantTier) => {
    const got = layoutTierFor({ width, height }, id)
    if (got !== wantTier) {
      fail(`biên — ${label} (${id}, ${width}×${height}) mong \`${wantTier}\`, được \`${got}\``)
      bad += 1
    }
  }
  for (const id of PRESET_IDS) {
    const seed = EXPECTED[id]
    if (seed === undefined) {
      abort('`scripts/check-layout.mjs`', new Error(`Kiểm E — \`EXPECTED\` thiếu khoá \`${id}\`.`))
    }
    // minSupportedWidth — chiều cao giữ cố định cao (999) để không chen vào.
    expect('minSupportedWidth đúng biên ⇒ narrow', id, seed.minSupportedWidth, 999, 'narrow')
    expect('minSupportedWidth kém 1 ⇒ unsupported', id, seed.minSupportedWidth - 1, 999, 'unsupported')
    // minFullWidth — cùng lý do, chiều cao cố định cao.
    expect('minFullWidth đúng biên ⇒ không còn narrow-vì-rộng', id, seed.minFullWidth, 999, 'full')
    expect('minFullWidth kém 1 ⇒ narrow', id, seed.minFullWidth - 1, 999, 'narrow')
    // minFullHeight — chiều rộng cố định rộng (5000) để không chen vào.
    expect('minFullHeight đúng biên ⇒ full', id, 5000, seed.minFullHeight, 'full')
    expect('minFullHeight kém 1 ⇒ short', id, 5000, seed.minFullHeight - 1, 'short')
    // minShortHeight — cùng lý do, chiều rộng cố định rộng.
    expect('minShortHeight đúng biên ⇒ short', id, 5000, seed.minShortHeight, 'short')
    expect('minShortHeight kém 1 ⇒ narrow (quá thấp)', id, 5000, seed.minShortHeight - 1, 'narrow')
  }
  if (bad === 0) pass(`biên — cả 4 ngưỡng × 2 phía × ${PRESET_IDS.length} preset (ghim RIÊNG) đều đóng đúng`)
}

// Mệnh đề 3 — ƯU TIÊN TRÊN-XUỐNG: khi hai điều kiện cùng khớp, cái đứng TRƯỚC thắng.
{
  let bad = 0
  const expect = (label, id, width, height, wantTier) => {
    const got = layoutTierFor({ width, height }, id)
    if (got !== wantTier) {
      fail(`ưu tiên — ${label} (${id}, ${width}×${height}) mong \`${wantTier}\`, được \`${got}\``)
      bad += 1
    }
  }
  for (const id of PRESET_IDS) {
    // Rộng đã unsupported (800 < 860) VÀ cao đã đủ điều kiện narrow-vì-thấp (650 < 700):
    // unsupported phải thắng — chiều rộng được xét TRƯỚC chiều cao.
    expect('unsupported thắng narrow-vì-thấp', id, 800, 650, 'unsupported')
    // Rộng đủ full (1500 ≥ 1100) nhưng cao quá thấp (650 < 700): vẫn narrow, không phải
    // full/short — chiều cao "quá thấp" thắng một chiều rộng tốt.
    expect('narrow-vì-thấp thắng full/short dù rộng thoải mái', id, 1500, 650, 'narrow')
    // Rộng hẹp (900, trong dải 860-1099) nhưng cao rất tốt (2000): vẫn narrow — dải hẹp
    // của chiều rộng thắng một chiều cao thoải mái.
    expect('narrow-vì-hẹp thắng dù cao thoải mái', id, 900, 2000, 'narrow')
  }
  if (bad === 0) pass('ưu tiên trên-xuống — chiều rộng unsupported/hẹp thắng mọi mệnh đề về chiều cao')
}

// Mệnh đề 4 — `NEVER_SACRIFICED` KHÔNG BỊ ĐỤNG, VÀ HAI TỪ VỰNG RỜI NHAU.
//
// ⚠️ Tên cũ của mệnh đề này ("LƯỚI KHÔNG BAO GIỜ BỊ NHƯỜNG") hứa NHIỀU hơn thân nó
// chứng minh — thang bậc thuần ở đây chỉ trả về một TẦNG, nó không biết panel nào đang
// mount, nên nó KHÔNG THỂ tự chứng minh "lưới còn hiện". Cái nó chứng minh được, và chỉ
// chừng đó: Task 1/2 không đụng vào `NEVER_SACRIFICED` (spec §Always), và tên bốn tầng
// không trùng với một `PanelId` — hai từ vựng RỜI NHAU để không ai lẫn "trả về một tầng"
// với "trả về một panel". Mệnh đề THẬT "lưới không bao giờ bị nhường ở bất kỳ tầng nào"
// chỉ chứng minh được khi tầng được nối vào `hidePanel`/`showPanel` thật — đó là
// Phase 4, chạy trên `WorkspaceDock.vue`, không phải ở đây.
{
  let bad = 0
  if (NEVER_SACRIFICED.length !== 1 || NEVER_SACRIFICED[0] !== 'panel.grid') {
    fail(`\`NEVER_SACRIFICED\` phải là ĐÚNG \`['panel.grid']\`, đang là [${NEVER_SACRIFICED.join(', ')}]`)
    detail('Task 1/2 của Story 4.12 không được đụng vào tập này — spec §Always.')
    bad += 1
  }
  const overlap = LAYOUT_TIERS.filter((t) => PANEL_IDS.includes(t))
  if (overlap.length > 0) {
    fail(`tên tầng trùng với \`PanelId\`: ${overlap.join(', ')}`)
    detail('Hai từ vựng phải RỜI NHAU — nhầm lẫn "tầng" với "panel" là chỗ nối dễ hỏng nhất.')
    bad += 1
  }
  if (bad === 0) {
    pass('`NEVER_SACRIFICED` vẫn đúng `[\'panel.grid\']`, và tên tầng không trùng `PanelId` nào')
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm F — ba khớp nối bỏ ngỏ giữa WorkspaceDock.vue và workspaceLayout.ts/App.vue')
// ═════════════════════════════════════════════════════════════════════════════════
//
// Ba mệnh đề, mỗi mệnh đề canh một khớp nối mà Kiểm A-E ở trên không chạm tới —
// `deferred-work.md` L631, L639, L1233:
//   F.1  `WorkspaceDock.vue` ghi bố cục xuống đĩa CHỈ qua `flush()`, và `flush()` CHỈ ghi
//        khi `schedule` báo bẩn — Kiểm B đo nhịp ghi của `writeSchedule.ts` một mình, nó
//        không thấy một `emit('persist', …)` đứng thẳng ở `onLayoutChange`.
//   F.2  Mọi giá trị của `PANEL_COMPONENTS` (`workspaceLayout.ts`) là một khoá của
//        `components` (`WorkspaceDock.vue`) — lệch một tên là một panel TRẮNG kèm
//        `console.error` của chính dockview, và không cổng nào thấy được hôm nay.
//   F.3  Phần tử host cây dockview (`.modeport`, `App.vue`) còn giữ `isolation: isolate`
//        — cơ chế đã vá lỗi sash vẽ đè lớp phủ (Ice bắt bằng mắt 2026-08-10). Đây KHÔNG so
//        hai số `z-index`, nó canh đúng CƠ CHẾ vừa chọn — z-index của dockview đổi bao
//        nhiêu cũng không chạm phép kiểm này.

const dockVueSource = sources.find((s) => posix(s.file) === 'src/layout/WorkspaceDock.vue')
if (dockVueSource === undefined) {
  abort('`src/layout/WorkspaceDock.vue`', new Error('không có trong quần thể quét — Kiểm F cần nó.'))
}
const appVueSource = sources.find((s) => posix(s.file) === 'src/App.vue')
if (appVueSource === undefined) {
  abort('`src/App.vue`', new Error('không có trong quần thể quét — Kiểm F cần nó.'))
}

// ───────────────────────────────────────────────────────────────────────────────────
// F.1 — persist chỉ qua `flush()`, và `flush()` canh `schedule` trước khi ghi.
// ───────────────────────────────────────────────────────────────────────────────────
const EMIT_PERSIST_RE = /\bemit\(\s*(['"])persist\1/g

/**
 * @param {string} masked
 * @returns {{hasFlush: boolean, emitCount: number, outsideCount: number, guarded: boolean}}
 */
function scanPersistViaSchedule(masked) {
  const body = functionBodyRange(masked, 'flush')
  const emitAt = []
  const re = new RegExp(EMIT_PERSIST_RE.source, 'g')
  let m
  while ((m = re.exec(masked))) emitAt.push(m.index)
  const outsideCount = emitAt.filter((i) => body === null || i < body.start || i >= body.end).length
  const inner = body === null ? '' : masked.slice(body.start, body.end)
  const guarded = /schedule\s*\.\s*isDirty\s*\(/.test(inner) && /schedule\s*\.\s*onWrite\s*\(/.test(inner)
  return { hasFlush: body !== null, emitCount: emitAt.length, outsideCount, guarded }
}

{
  const r = scanPersistViaSchedule(dockVueSource.masked)
  if (!r.hasFlush) {
    fail('`WorkspaceDock.vue` — không tìm thấy `function flush(...)`, F.1 không đối chiếu được')
  } else if (r.emitCount === 0) {
    fail("`WorkspaceDock.vue` — không còn lời gọi `emit('persist', …)` nào, AC4 mất đường ghi")
  } else if (r.outsideCount > 0) {
    fail(
      `\`WorkspaceDock.vue\` — ${r.outsideCount} lời gọi \`emit('persist', …)\` đứng NGOÀI ` +
        '`flush()` — đi tắt qua mặt nhịp ghi mà Kiểm B đo',
    )
  } else if (!r.guarded) {
    fail('`WorkspaceDock.vue` — `flush()` không còn canh `schedule.isDirty()`/`schedule.onWrite(...)` trước khi ghi')
  } else {
    pass("F.1 — `emit('persist', …)` chỉ đứng trong `flush()`, và `flush()` canh `schedule` trước khi ghi")
  }
}

// Tự kiểm F.1 — mã tổng hợp, không phải `WorkspaceDock.vue` thật.
const F1_CASES = [
  [
    'đúng: emit trong flush, có canh schedule',
    "function flush(): void {\n  if (!schedule.isDirty()) return\n  schedule.onWrite(now)\n  emit('persist', json)\n}",
    true,
  ],
  [
    'sai: emit thẳng ở onLayoutChange, ngoài flush',
    "function flush(): void {\n  if (!schedule.isDirty()) return\n  schedule.onWrite(now)\n}\nfunction onLayoutChange(): void {\n  emit('persist', json)\n}",
    false,
  ],
  [
    'sai: flush() ghi thẳng, không canh schedule',
    "function flush(): void {\n  emit('persist', json)\n}",
    false,
  ],
  [
    'sai: không còn function flush nào',
    "function save(): void {\n  emit('persist', json)\n}",
    false,
  ],
]
let f1Bad = 0
for (const [name, code, shouldPass] of F1_CASES) {
  const r = scanPersistViaSchedule(maskComments(code))
  const ok = r.hasFlush && r.emitCount > 0 && r.outsideCount === 0 && r.guarded
  if (ok !== shouldPass) {
    fail(`tự kiểm F.1 — ca \`${name}\`: mong ${shouldPass ? 'XANH' : 'ĐỎ'}, nhận ${ok ? 'XANH' : 'ĐỎ'}`)
    f1Bad += 1
  }
}
if (f1Bad === 0) pass(`tự kiểm F.1 — ${F1_CASES.length} ca (1 xanh thật · 3 đỏ đúng lý do)`)

// ───────────────────────────────────────────────────────────────────────────────────
// F.2 — mọi giá trị của `PANEL_COMPONENTS` là một khoá của `components`.
// ───────────────────────────────────────────────────────────────────────────────────
const COMPONENTS_HEAD_RE = /\bconst\s+components(?::[^=]*)?\s*=\s*\{/

/**
 * @param {string} masked
 * @returns {string[] | null}
 */
function componentKeysOf(masked) {
  const body = balancedBraceBody(masked, new RegExp(COMPONENTS_HEAD_RE.source))
  if (body === null) return null
  const keys = []
  for (const raw of splitTopLevel(body)) {
    const entry = raw.trim()
    if (entry === '') continue
    const m = /^([A-Za-z_$][A-Za-z0-9_$]*)\s*:/.exec(entry)
    if (m) keys.push(m[1])
  }
  return keys
}

if (layoutMod.PANEL_COMPONENTS === undefined) {
  abort('`src/layout/workspaceLayout.ts`', new Error('không export `PANEL_COMPONENTS` — F.2 cần nó.'))
}

{
  const keys = componentKeysOf(dockVueSource.masked)
  if (keys === null) {
    fail('`WorkspaceDock.vue` — không tìm thấy `const components = { … }`, F.2 không đối chiếu được')
  } else {
    const values = Object.values(layoutMod.PANEL_COMPONENTS)
    const missing = values.filter((v) => !keys.includes(v))
    if (missing.length > 0) {
      fail(
        `\`PANEL_COMPONENTS\` (workspaceLayout.ts) khai [${missing.join(', ')}] — không phải khoá ` +
          `nào của \`components\` ([${keys.join(', ')}]) trong WorkspaceDock.vue — panel TRẮNG`,
      )
      detail('`PANEL_COMPONENTS` sống ở workspaceLayout.ts, `components` sống ở WorkspaceDock.vue —')
      detail('một tên lệch giữa hai bảng chỉ hiện ra lúc chạy, bằng console.error của dockview.')
    } else {
      pass(`F.2 — ${values.length} giá trị của \`PANEL_COMPONENTS\` đều là khoá của \`components\``)
    }
  }
}

// Tự kiểm F.2 — mã tổng hợp, không phải các tệp thật.
const F2_CASES = [
  ['đúng: mọi giá trị đều là khoá', ['grid', 'lookup'], 'const components = { grid: GridPanel, lookup: LookupPanel }', true],
  [
    'sai: một giá trị lệch tên',
    ['grid', 'lookupX'],
    'const components = { grid: GridPanel, lookup: LookupPanel }',
    false,
  ],
  [
    'comment nhắc tên không tính',
    ['grid'],
    '// const components = { grid: GridPanel }\nconst components = { lookup: LookupPanel }',
    false,
  ],
]
let f2Bad = 0
for (const [name, values, code, shouldPass] of F2_CASES) {
  const keys = componentKeysOf(maskComments(code))
  const ok = keys !== null && values.every((v) => keys.includes(v))
  if (ok !== shouldPass) {
    fail(`tự kiểm F.2 — ca \`${name}\`: mong ${shouldPass ? 'XANH' : 'ĐỎ'}, nhận ${ok ? 'XANH' : 'ĐỎ'}`)
    f2Bad += 1
  }
}
if (f2Bad === 0) pass(`tự kiểm F.2 — ${F2_CASES.length} ca (1 xanh thật · 2 đỏ đúng lý do)`)

// ───────────────────────────────────────────────────────────────────────────────────
// F.3 — phần tử host cây dockview (`.modeport`) còn giữ `isolation: isolate`.
// ───────────────────────────────────────────────────────────────────────────────────
const MODEPORT_HEAD_RE = /\.modeport\s*\{/
const ISOLATION_RE = /\bisolation\s*:\s*isolate\b/

/**
 * @param {string} masked
 * @param {RegExp} selectorHead
 * @returns {boolean | null}
 */
function hostKeepsIsolation(masked, selectorHead) {
  const body = balancedBraceBody(masked, new RegExp(selectorHead.source))
  if (body === null) return null
  return ISOLATION_RE.test(body)
}

{
  const kept = hostKeepsIsolation(appVueSource.masked, MODEPORT_HEAD_RE)
  if (kept === null) {
    fail('`App.vue` — không tìm thấy khối `.modeport { … }`, F.3 không canh được `isolation: isolate`')
  } else if (!kept) {
    fail('`App.vue` — `.modeport` KHÔNG còn `isolation: isolate` — sash lại vẽ đè lớp phủ (Ice bắt bằng mắt 2026-08-10)')
  } else {
    pass('F.3 — `.modeport` (App.vue) vẫn giữ `isolation: isolate`, ngữ cảnh xếp lớp của cây dockview còn cô lập')
  }
}

// Tự kiểm F.3 — mã tổng hợp, không phải App.vue thật.
const F3_CASES = [
  ['đúng: còn isolation: isolate', '.modeport {\n  flex: 1;\n  isolation: isolate;\n}', true],
  ['sai: isolation bị gỡ', '.modeport {\n  flex: 1;\n}', false],
  ['comment nhắc tên không tính', '/* .modeport { isolation: isolate; } */\n.modeport {\n  flex: 1;\n}', false],
  ['sai: selector .modeport mất hẳn', '.otherport {\n  isolation: isolate;\n}', false],
]
let f3Bad = 0
for (const [name, code, shouldPass] of F3_CASES) {
  const kept = hostKeepsIsolation(maskComments(code), MODEPORT_HEAD_RE)
  const ok = kept === true
  if (ok !== shouldPass) {
    fail(`tự kiểm F.3 — ca \`${name}\`: mong ${shouldPass ? 'XANH' : 'ĐỎ'}, nhận ${ok ? 'XANH' : 'ĐỎ'}`)
    f3Bad += 1
  }
}
if (f3Bad === 0) pass(`tự kiểm F.3 — ${F3_CASES.length} ca (1 xanh thật · 3 đỏ đúng lý do)`)

// ═════════════════════════════════════════════════════════════════════════════════
console.log('')
if (skippedLinks.length) {
  console.log(`\x1b[33mĐã BỎ QUA ${skippedLinks.length} symlink:\x1b[0m ${skippedLinks.join(' · ')}`)
  console.log('')
}
if (failures !== 0) {
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  console.log('')
  console.log('AD-24: MỘT cửa sổ hệ điều hành, ba chế độ ngang hàng. Undock = `addFloatingGroup`.')
  console.log('AD-11: mọi lượt ghi đi qua `store::Writer` nối tiếp — không kho thứ hai.')
  console.log('UX-DR15: thứ tự hy sinh là một QUYẾT ĐỊNH; ngưỡng kích thước là Story 4.12.')
  process.exit(1)
}
console.log('\x1b[32mTất cả phép kiểm bố cục đạt.\x1b[0m')
console.log('')
console.log(`Tầm quét: ${files.length} tệp dưới \`src/**\` · ${seenMembers.size} thành viên global.`)
console.log('')
console.log('Ghi chú cho người rà soát — ba giới hạn, ghi thẳng thay vì để người sau tự phát hiện:')
console.log('  1. `localStorage`/`sessionStorage` gọi TRẦN vẫn đi qua một mệnh đề CẤM, không')
console.log('     qua danh sách cho phép — chúng là định danh tự do, và liệt kê hết định danh')
console.log('     tự do đòi một bộ phân tích cú pháp thật (một phụ thuộc npm mới — NFR15).')
console.log('  2. Kiểm B đo NHỊP, không đo rằng `WorkspaceDock.vue` thật sự dùng lịch đó.')
console.log('     Vế đó là một lượt đếm tay trong DevTools — §Debug Log References của story.')
console.log('  3. AC7 khai CƠ CHẾ. Bốn ngưỡng màn hình hẹp là Story 4.12, và UX-DR15')
console.log('     cấm tường minh việc đóng chúng ở Story 1.14. Đừng thêm `matchMedia` vào đây.')
process.exit(0)
