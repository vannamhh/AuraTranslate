/**
 * Cổng BỐ CỤC — Story 1.14 · AC1 · AC4 · AC7 · AC12.
 *
 * Bốn phép kiểm, và cả bốn đều chạy trên **mã của sản phẩm**, không trên một bản chép:
 *
 *   A (AC7)  thứ tự hy sinh của UX-DR15 — ba mệnh đề, gọi `nextToSacrifice()` THẬT.
 *   B (AC4)  nhịp ghi bố cục — ĐẾM số lượt `putConfig` trên một dòng sự kiện dày.
 *   C (AC1 · AC12) bề mặt cấm: không cửa sổ OS thứ hai, không kho lưu trữ thứ hai.
 *   D        TỰ KIỂM — chứng minh Kiểm C **đỏ được**, và không đỏ oan.
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
const FILE_FLOOR = 40 // số THẬT 2026-08-12 (sau Story 2.2): 50 tệp `src/**` — 40/50 = 80,0%

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
// kích thước màn hình là **Story 4.12**, và `epics.md:1617` cấm tường minh việc đóng chúng
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

// Mệnh đề 1 — hai tập RỜI NHAU và hợp lại đúng bốn panel.
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

// Mệnh đề 2 — `panel.source` / `panel.editor` KHÔNG BAO GIỜ là đầu ra.
//
// ⚠️ Duyệt TOÀN BỘ 16 tập con của bốn panel, không chỉ vài ca lấy mẫu: mệnh đề là
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
    fail('`panel.ai_translation` phải nhường TRƯỚC `panel.lookup` (epics.md:1616)')
  } else if (nextToSacrifice([...PANEL_IDS]) !== 'panel.ai_translation') {
    fail(`đủ bốn panel ⇒ cái nhường đầu tiên phải là \`panel.ai_translation\``)
  } else if (nextToSacrifice(['panel.source', 'panel.lookup', 'panel.editor']) !== 'panel.lookup') {
    fail('sau khi `panel.ai_translation` đã nhường, cái kế tiếp phải là `panel.lookup`')
  } else if (nextToSacrifice(['panel.source', 'panel.editor']) !== null) {
    fail('chỉ còn cặp không nhường ⇒ phải trả `null`, không phải hy sinh một trong hai')
  } else {
    pass('`panel.ai_translation` nhường trước · `panel.lookup` nhường sau · cặp còn lại ⇒ `null`')
  }
}

// Nghịch đảo: trả panel về theo thứ tự NGƯỢC. Không có nó thì một lượt nới cửa sổ trả
// `panel.ai_translation` về trước `panel.lookup`, tức đảo đúng ưu tiên vừa phát biểu.
if (typeof nextToRestore === 'function') {
  const a = nextToRestore(['panel.source', 'panel.editor'])
  const b = nextToRestore(['panel.source', 'panel.editor', 'panel.lookup'])
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
  // đường DUY NHẤT đóng được `deferred-work.md:608` (bôi đen bằng bàn phím) mà không phải bật
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
  // một component KHÁC, và một lượt đổi preset bố cục dựng lại cả bốn panel. API DOM chuẩn,
  // không mở cửa sổ/kho thứ hai — AC1/AC12 canh đúng hai thứ đó.
  'document.querySelector',
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
console.log('  3. AC7 khai CƠ CHẾ. Bốn ngưỡng màn hình hẹp là Story 4.12, và `epics.md:1617`')
console.log('     cấm tường minh việc đóng chúng ở Story 1.14. Đừng thêm `matchMedia` vào đây.')
process.exit(0)
