#!/usr/bin/env node
/**
 * Cổng CommandRegistry của Story 1.6 — cưỡng chế AD-34 · FR22 · NFR17 bằng lệnh, mã
 * thoát là phán quyết.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * VÌ SAO TỒN TẠI
 * ─────────────────────────────────────────────────────────────────────────────────
 * AC1 (*"mọi thao tác đăng ký ở `CommandRegistry` TRƯỚC khi bind; handler chuột chỉ
 * `dispatch`"*) là mệnh đề trung tâm của AD-34 §1, và nó thuộc đúng loại quy tắc thoái
 * hoá thành kỷ luật cá nhân qua bảy giai đoạn. Ba quy tắc cùng hạng đã lần lượt được
 * đóng bằng một cổng: `check-deps.mjs` (phụ thuộc) · `check-tokens.mjs` (màu và cỡ chữ)
 * · `check-i18n.mjs` (chuỗi giao diện). Đây là cổng thứ tư, và nó nợ từ Story 1.6.
 *
 * ⚠️ Node chứ không bash (Ice chốt 2026-08-03, `check-deps.mjs:22-24`): `npm run` trên
 * Windows đi qua `cmd.exe`, không có bash. Một cổng chỉ canh nửa số nền tảng thì không
 * canh được NFR14 — và Kiểm D dưới đây tồn tại ĐÚNG vì NFR14.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * NĂM PHÉP KIỂM
 * ─────────────────────────────────────────────────────────────────────────────────
 *   A (AC1)          mọi `@click` / `v-on:click` trong `src/**\/*.vue` là ĐÚNG MỘT lời
 *                    gọi `dispatch('<id>')` — không hàm khác, không mã nội tuyến.
 *   B (AC2)          mọi id trong `dispatch('…')` khớp văn phạm khoá chấm VÀ có mặt
 *                    trong bộ đã đăng ký.
 *   C (AC1,2,6)      HÀNH VI thật của `src/commands/registry.ts` — nạp và gọi hàm.
 *   D (AC3)          HÀNH VI thật của `src/commands/keys.ts` trên CẢ HAI nền tảng.
 *   E (AC4)          nhãn có trong `vi.json`; sổ điểm vào focus khớp mã nguồn hai chiều.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * GIỚI HẠN — ghi thẳng thay vì để người sau tự phát hiện
 * ─────────────────────────────────────────────────────────────────────────────────
 * 1. **Chỉ `@click`.** `@keydown`, `@input`, `@change`, `@submit` KHÔNG thuộc luật Kiểm A
 *    — chúng không phải "thao tác" theo nghĩa AD-34 §1 (một `@input` là dòng dữ liệu, một
 *    `@click` là một thao tác người dùng chủ động). Ngày một `@keydown` mang thao tác thật
 *    xuất hiện, luật này phải được xem lại — chứ không phải mở rộng regex một cách lặng lẽ.
 * 2. **Vế DOM của AC4 KHÔNG kiểm được ở đây.** *"Focus không bao giờ rơi về `body`"* là
 *    hành vi lúc chạy trong một webview thật. Cổng canh vế KHAI BÁO; vế hành vi có một
 *    chốt tự kêu ở `focus.ts` cộng một lượt nghiệm thu tay có bảng, và giới hạn đó ghi ở
 *    `deferred-work.md`. Không đánh dấu đạt bằng suy luận.
 * 3. **Không canh focus ring.** `outline: none` trải toàn ứng dụng vẫn qua được cổng này
 *    và cả `check-tokens.mjs` (§Trap 4 của story). Luật đang do người viết giữ.
 * 4. **Chuỗi ký tự KHÔNG được che** — cố ý, vì Kiểm B phải đọc được nội dung
 *    `dispatch('mode.library')`. Hệ quả phải nói ra: một `dispatch('…')` nằm TRONG một
 *    chuỗi hay một ví dụ trong doc-comment vẫn bị Kiểm B chấm. Đó là dương tính giả — tức
 *    hướng an toàn — nhưng nó là **hành vi**, không phải tai nạn, nên nó được khai ở đây.
 *    *(Comment thì CÓ được che; máy trạng thái vẫn theo dõi chuỗi để một `//` trong dấu
 *    nháy không mở một comment giả.)*
 * 5. **Khối `<script>`/`<style>` phải ở ĐẦU DÒNG.** `vueRegions` neo `^` (cờ `m`) vì bản
 *    quét không neo đọc một `<style>` nằm trong mustache hay attribute thành một vùng
 *    thật, rồi nuốt mọi `@click` phía sau. Thẻ không ở đầu dòng được ĐẾM và IN RA ở Kiểm A
 *    — không bỏ qua im lặng.
 * 6. **`t(<biến>)` và `:title-key="<biểu thức>"` không đọc tĩnh được.** Kiểm E xác nhận
 *    khoá `t('…')` literal và `title-key="…"` literal; phần còn lại được đếm và in ra.
 *    `PanelFrame` nhận khoá qua prop, nên đường đó chỉ kiểm được ở chỗ GỌI component.
 * 7. **Ba phần mở rộng `.tsx` · `.mts` · `.cts` KHÔNG được quét** — dự án không dùng chúng
 *    và `tsconfig.json` không bật `jsx`. Mục đã mở ở `deferred-work.md`.
 *
 * Chạy:  npm run check:commands
 */
import { readFileSync, readdirSync, lstatSync, realpathSync, existsSync } from 'node:fs'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { basename, dirname, join, relative, sep } from 'node:path'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const SRC_ROOT = join(REPO_ROOT, 'src')
const REGISTRY_TS = join(SRC_ROOT, 'commands', 'registry.ts')
const KEYS_TS = join(SRC_ROOT, 'commands', 'keys.ts')
const COMMANDS_INDEX_TS = join(SRC_ROOT, 'commands', 'index.ts')
const VI_JSON = join(SRC_ROOT, 'i18n', 'vi.json')

let failures = 0
const pass = (m) => console.log(`  \x1b[32mOK\x1b[0m   ${m}`)
const fail = (m) => {
  console.log(`  \x1b[31mFAIL\x1b[0m ${m}`)
  failures += 1
}
const detail = (m) => console.log(`       ${m}`)

/**
 * Lỗi hạ tầng ≠ phép kiểm đỏ. Dừng ngay, đừng báo cáo một kết quả không có thật.
 * (`check-deps.mjs:60-66`)
 */
function abort(what, err) {
  console.error(`\n\x1b[31mKhông đọc được ${what} — phép kiểm KHÔNG chạy được.\x1b[0m`)
  console.error('Đây là lỗi hạ tầng, không phải "đạt". Đọc lỗi dưới đây rồi chạy lại.\n')
  console.error(String(err?.message || err).trim())
  process.exit(1)
}

// ═════════════════════════════════════════════════════════════════════════════════
// MIỄN TRỪ — mỗi mục một câu lý do, và cổng IN RA số tệp đã miễn trừ ở mỗi lượt chạy
//
// Miễn trừ KHÔNG được cài bằng cách thu hẹp glob quét (cùng luật với `check-i18n.mjs`).
// Hôm nay danh sách RỖNG, và con số 0 in ra có chủ ý: khối này tồn tại để lần đầu ai đó
// cần một ngoại lệ thì phải viết lý do ra đây, chứ không phải sửa một dấu sao.
// ═════════════════════════════════════════════════════════════════════════════════
const EXEMPT = []

function globToRe(pattern) {
  const escaped = pattern.replace(/[.+^${}()|[\]\\]/g, '\\$&')
  return new RegExp(`^${escaped.replace(/\*\*/g, '.*').replace(/(?<!\.)\*/g, '[^/]*')}$`)
}
const EXEMPT_RES = EXEMPT.map(([pattern, why]) => [globToRe(pattern), pattern, why])
const posix = (p) => relative(REPO_ROOT, p).split(sep).join('/')
const exemptionFor = (file) => EXEMPT_RES.find(([re]) => re.test(posix(file)))

// ═════════════════════════════════════════════════════════════════════════════════
// Đọc cây nguồn
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ `lstatSync`, KHÔNG `statSync` — cùng bài học với hai cổng trước: `statSync` giải
// symlink nên một liên kết trỏ về thư mục cha làm đệ quy không dừng, và một liên kết gãy
// ném `ENOENT` bị `abort()` báo thành "cây nguồn không đọc được". Symlink bị BỎ QUA và
// ghi tên ra, để việc bỏ qua không im lặng.
const skippedLinks = []
const SKIP_DIRS = new Set(['target', 'node_modules', 'dist', '.git'])

function walk(dir, exts, out = [], seen = new Set()) {
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
    if (st.isDirectory()) walk(full, exts, out, seen)
    else if (exts.some((e) => name.toLowerCase().endsWith(e))) out.push(full)
  }
  return out
}

let vueAll = []
let tsAll = []
try {
  vueAll = walk(SRC_ROOT, ['.vue']).sort()
  /**
   * 🔴 BỐN PHẦN MỞ RỘNG, KHÔNG PHẢI MỘT — đóng `deferred-work.md:163` (Story 1.14 · AC11.3).
   *
   * `endsWith('.ts')` là `false` với `.tsx`, `.mts` VÀ `.cts` — cả ba đều là TypeScript và
   * cả ba đều `import()` được bằng Node. Hôm nay cây không có tệp nào như vậy, nên lỗ
   * này chưa từng để lọt gì; nhưng ngày đầu tiên một `.mts` xuất hiện, nó rơi ra khỏi Kiểm
   * A/B *(quét `dispatch()`)* và khỏi sổ điểm vào focus **mà không một dòng nào báo** —
   * đúng hình dạng im lặng mà mọi cổng ở đây tồn tại để chặn.
   *
   * ⚠️ `.d.ts` KHÔNG bị loại: một tệp khai báo không chở `dispatch()` nào, nên nó chỉ
   * làm quần thể to thêm mà không làm phán quyết sai. Loại nó đòi thêm một luật, và một
   * luật thừa là một chỗ để sai.
   */
  tsAll = walk(SRC_ROOT, ['.ts', '.tsx', '.mts', '.cts']).sort()
} catch (err) {
  abort('cây nguồn `src/**`', err)
}

const exemptedFiles = []
const keep = (files) =>
  files.filter((f) => {
    const hit = exemptionFor(f)
    if (hit) exemptedFiles.push([posix(f), hit[1]])
    return !hit
  })

const vueFiles = keep(vueAll)
const tsFiles = keep(tsAll)

/**
 * 🔴 NGƯỠNG SÀN, BẮT BUỘC — không phải nice-to-have.
 *
 * `check-deps.mjs:15-17` đã đâm vào đúng bẫy này một lần (*"cây rỗng đọc thành sạch"*) và
 * `check-i18n.mjs:212-222` phải dựng lại nó lần nữa. Ở đây tương đương là một glob viết
 * sai khớp 0 tệp `.vue` ⇒ Kiểm A và Kiểm B in "không tìm thấy vi phạm" ⇒ exit 0 ⇒ cổng
 * chết im lặng ngay ngày nó ra đời.
 *
 * Số THẬT lúc dựng (2026-08-04): **5** tệp `.vue` (`App` · ba chế độ · `PanelFrame`) ·
 * **13** tệp `.ts` · **4** command. Sàn đặt dưới số thật một khoảng nhỏ để một lần xoá
 * tệp có chủ ý không làm cổng `abort()`, nhưng một lượt quét hỏng thì có.
 *
 * 🔴 NÂNG SÀN 2026-08-06 — Story 1.14 · AC11.1, đóng `deferred-work.md:48` và `:146`.
 *
 * Số THẬT sau Story 1.14: **11** tệp `.vue` *(`App` · ba chế độ · `PanelFrame` ·
 * `PanelTab` · bốn panel · `WorkspaceDock`)* · **18** tệp `.ts` · **11** command.
 *
 * ⚠️ Nâng sàn **KHÔNG** phải "sửa cho vừa". Ba con số trên là quần thể ĐO ĐƯỢC hôm nay
 * và chúng nằm trong comment này chính để lượt nâng sau đối chiếu được — cùng khuôn mà
 * `RS_FLOOR` của `check-i18n.mjs` đã dùng. Sàn thấp hơn số thật một khoảng nhỏ; một lượt
 * quét hỏng (glob sai, `SKIP_DIRS` nuốt nhầm) tụt sâu hơn khoảng đó rất nhiều.
 *
 * ⚠️ Và sàn ĐẾM TỆP thì một tệp RỖNG vẫn qua — đó là giới hạn thật của cơ chế này, và nó
 * được bù bằng `CLICK_FLOOR`/`DISPATCH_FLOOR`/`COMMAND_FLOOR` ngay dưới (sàn NỘI DUNG).
 */
// 🔴 NÂNG SÀN 2026-08-06 — Story 1.17 · Task 10 (AC13). Số THẬT sau story: 13 tệp `.vue` ·
// 24 tệp `.ts` · 17 command · 8 `@click` · 12 lời gọi `dispatch()` (trước story: 12/23/16/8/12).
//
// ⚠️ **Sửa sổ sách 2026-08-07 (code review).** Bản đầu ghi *"trước story: … 8"* cho
// `dispatch()` và dùng con số đó để biện minh cho một lượt nâng sàn 6 → 10. Đếm lại bằng
// CHÍNH `DISPATCH_CALL_RE` của cổng: **12 trước, 12 sau** — Story 1.17 không thêm hay bớt một
// lời gọi `dispatch()` nào (`git diff` trên `src/**` không một dòng `dispatch(` nào). Số "8"
// là ghi chép cũ chưa cập nhật từ 1.16, và nó đã bị chép lại thành một mệnh đề nhân quả
// SAI. Sàn 10 vẫn đúng theo số thật 12 nên không hạ lại; chỉ **lý do** được sửa cho khớp sự
// thật — một con số bịa trong đúng tệp mà cả kiến trúc dựa vào để tin các con số là chính
// thứ rot mà AC13 tồn tại để chặn.
const VUE_FLOOR = 13 // số THẬT 2026-08-11 (sau Story 1.21): 15 tệp `.vue` — 13/15 = 86,7%
// 🔴 NÂNG 2026-08-12 (Story 2.1) — số thật lên **32**: `src/config/segment.ts`, wrapper IPC
// của lệnh tách tường minh. Sàn 26 trên 32 là 81,3%; lên **27** để giữ dải ~84% của lượt
// trước. `VUE_FLOOR`/`COMMAND_FLOOR`/`CLICK_FLOOR`/`DISPATCH_FLOOR` KHÔNG đổi — story này
// thêm 0 tệp `.vue`, 0 command của `CommandRegistry`, 0 `@click`, 0 lời gọi `dispatch()`.
// 🔴 NÂNG 2026-08-12 (Story 2.2 · AC16) — số thật lên **35** (thêm `editorSegments.ts`,
// `editorGutter.ts`, `editorPanelState.ts`), nên sàn 27 tụt xuống 77,1%, dưới dải ~81–85%
// mà chính doc-comment ở trên đặt ra. Đo chứ không ước.
// 🔵 ĐẾM LẠI 2026-08-14 (Story 2.5b) — **KHÔNG đổi số, và đó là một kết quả chứ không một
// lượt bỏ qua.** Story gỡ `editorGutter.ts` và thêm `hanVietSurfaces.ts` + `segmentNavigation.ts`
// ⇒ 36 → **37**. Sàn 30 nay là 81,1%, vẫn trong dải ~81–85% mà doc-comment trên đặt ra.
// ⚠️ Sàn là **cận dưới**: bớt tệp không làm cổng đỏ, nó chỉ làm sàn vô nghĩa — nên lượt đếm
// lại này là bắt buộc kể cả khi kết luận là "giữ nguyên".
const TS_FLOOR = 30 // số THẬT 2026-08-14 (sau Story 2.5b): 37 tệp `.ts` — 30/37 = 81,1%
/**
 * ⚠️ Sàn command: **17** hôm nay — ba chế độ · `focus.next_panel` · `focus.prev_panel` ·
 * hai `layout.preset_*` · **ba** `layout.toggle_*` *(🔵 bốn → ba, Story 2.5b)* · hai
 * `library.import_*` · ba
 * `source.select_tab_*`/`toggle_han_viet_view` · `lookup.lookup_selection` (Story 1.17) ·
 * `editor.confirm_segment` (2.5) · `editor.next_untranslated` (2.5b).
 * Một bộ đăng ký rỗng làm Kiểm B, D và E xanh mà không kiểm gì.
 */
// 🔵 ĐẾM LẠI 2026-08-14 (Story 2.5b): **+1** (`editor.next_untranslated`) và **−1**
// (`layout.toggle_*` từ bốn xuống ba, theo `PANEL_SUFFIXES`) ⇒ 34 → **35**. Giữ sàn 29.
// 🔵 ĐO LẠI 2026-08-16 (Story 2.6), không chép: **41** command thật — 35/41 = 85,4 %.
// Dòng cũ ghi *"ĐO LẠI 2026-08-16 (Story 2.5d): 39 command — 33/39 = 84,6 %"*, và nó hết đúng
// khi 2.6 thêm `history.open`/`history.close` (→ 41).
// 🔵 **SỬA 2026-08-16 (code review Story 2.6): con số 41 ở dòng trên SAI, số thật là 44.**
// Story 2.6 đăng ký **năm** command `history.*` (`commands/index.ts:827-834`), không hai —
// dòng trên chỉ đếm `open`/`close` rồi bỏ quên `restore`/`confirm_restore`/`cancel_restore`.
// Đo bằng cách chạy chính cổng này: `OK   44 command`. ⇒ 35/44 = **79,5 %**, tức sàn đã rơi
// **dưới** dải 80–85 % và chín command có thể biến mất mà cổng vẫn xanh. Sàn nay là **37**
// (37/44 = 84,1 %).
// 🔴 Bài học đắt hơn con số: dòng sai nằm ngay dưới một dòng tự xưng *"không chép"*, trong một
// story mà luật đo của nó là *"đo lại, đừng chép"*. Một phép đo **tự khai** là đã đo vẫn phải
// đối chứng bằng cách CHẠY thứ nó đo — `npm run check:commands` in ra con số thật, và nó rẻ.
// ⚠️ Một sàn không được nâng **không làm cổng đỏ** — nó chỉ lặng lẽ mất ý nghĩa, vì sàn là
// **cận dưới**.
// 🔴 Khuôn này đã lặp lại **ba** lượt liên tiếp (2.5c · 2.5d · 2.6) và mỗi lượt phải sửa bằng
// tay. Không cổng nào canh chính cái sàn này — nó là một con số người phải nâng, và cái duy
// nhất nhắc là dòng chú thích đang đọc.
// 🔵 2026-08-17, Story 2.8 — 37 → 38. Số THẬT do lại **từ chính cổng này in ra**, không từ
// một lượt đếm bằng mắt: `npm run check:commands` báo **46 command** sau khi thêm
// `editor.merge_segments` và `editor.split_segment` (baseline 44). 38 / 46 = **82,6 %**, nằm
// giữa dải 80–85 % mà luật sàn quần thể đặt ra.
// ⚠️ Đây đúng lớp lỗi mà code review Story 2.6 bắt được: chú thích ở đó tự khai *"đo lại,
// không chép"* rồi ghi **41** trong khi cổng in **44** — nó chỉ đếm 2 trong 5 command mới của
// chính story đó. ⇒ Lượt này chạy cổng trước, đọc số, rồi mới sửa dòng dưới.
const COMMAND_FLOOR = 38

/**
 * 🔴 SÀN NỘI DUNG — tầng thứ hai của cùng một cái bẫy, và tầng này từng để lọt thật.
 *
 * Sàn tệp ở trên đóng được *"cây rỗng đọc thành sạch"*. Nhưng Kiểm A và Kiểm B vẫn `pass`
 * trên một danh sách **thuộc tính** rỗng: `aBad === 0` đúng khi không có `@click` nào để
 * kiểm. Đó chính là thứ làm cho lỗ `vueRegions` *(vùng `<style>` giả nuốt mọi `@click`
 * phía sau)* trở nên **im lặng** — cổng vẫn in `OK` và vẫn exit 0.
 *
 * Số THẬT hôm nay: **3** `@click` (ba tab chế độ ở `App.vue`) · **3** lời gọi `dispatch()`
 * literal. Sàn đặt đúng bằng số thật: hôm nay không có lý do chính đáng nào để một trong
 * hai con số đó giảm, và ngày Story 1.14 dựng panel thật thì chúng chỉ tăng.
 */
// 🔴 NÂNG 2026-08-07 (code review) — AC13 gọi ĐÍCH DANH sàn này (*"`CLICK/DISPATCH_FLOOR`
// **6** vs 8"*) và đòi *"**mọi** hằng `*_FLOOR` bị vượt được nâng theo số thật"*. Bản đầu
// đánh dấu nó *"không đổi ở Story 1.17"* thay vì nâng — 6/8 = 75%, dưới hẳn doctrine
// ~81-85% mà **mọi** sàn khác trong cùng lượt tuân theo. Đúng cách 1.16 để lọt và bị bắt.
const CLICK_FLOOR = 17 // số THẬT 2026-08-11 (sau Story 1.21): 21 thuộc tính `@click` — 17/21 = 81,0%
const DISPATCH_FLOOR = 23 // số THẬT 2026-08-11 (sau Story 1.21): 28 lời gọi `dispatch()` — 23/28 = 82,1%

if (vueFiles.length < VUE_FLOOR || tsFiles.length < TS_FLOOR) {
  abort(
    `quần thể quét — ${vueFiles.length} tệp \`.vue\` (sàn ${VUE_FLOOR}) · ` +
      `${tsFiles.length} tệp \`.ts\` (sàn ${TS_FLOOR})`,
    new Error(
      'Cây quá nhỏ để là thật. Một danh sách rỗng làm Kiểm A và Kiểm B xanh mà không kiểm gì cả.\n' +
        `Đã miễn trừ ${exemptedFiles.length} tệp — kiểm lại danh sách EXEMPT nếu con số đó bất thường.`,
    ),
  )
}

// ═════════════════════════════════════════════════════════════════════════════════
// CHE COMMENT, GIỮ NGUYÊN OFFSET
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ Che chứ không xoá: mọi số dòng báo lỗi bên dưới tính từ offset trong văn bản gốc.
// Xoá đi thì mọi chẩn đoán trỏ sai dòng, và một cổng chỉ đường sai sẽ bị người sau thêm
// ngoại lệ cho tới khi nó không bắt được gì (`check-tokens.mjs:351-355`).
//
// 🔴 CHỈ CHE COMMENT, KHÔNG CHE CHUỖI — ngược với `check-tokens.mjs`. Cổng này cần đọc
// NỘI DUNG chuỗi (`dispatch('mode.library')`), nên chuỗi phải sống sót. Nhưng máy trạng
// thái vẫn phải THEO DÕI chuỗi, vì một `//` hay `/*` bên trong dấu nháy không được mở một
// comment giả — đúng bài học `check-i18n.mjs:43-55`.
//
// 🔴 REGEX LITERAL là trạng thái mà một lượt cài đặt vội bỏ sót, và nó hỏng theo chiều
// ĐẮT NHẤT: `/^https?:\/\//` chứa `\/` rồi `/`; một máy không biết regex đọc cặp đó thành
// `//` và che nốt dòng — tức nuốt một `dispatch(` thật ngay sau đó.

const IDENT_CHAR = /[A-Za-z0-9_]/

/**
 * Sau ký tự nào thì một `/` mở REGEX chứ không phải phép chia. (`check-i18n.mjs:394-415`)
 *
 * ⚠️ `}` PHẢI có mặt. Thiếu nó thì một regex mở đầu câu lệnh ngay sau một block —
 * `function f() {}` xuống dòng `/^https?:\/\//.test(x)` — bị đọc thành phép chia; máy
 * trạng thái gặp cặp `\/` + `/` liền đó và che nốt dòng, tức nuốt một `dispatch(` thật
 * nằm sau nó. Đúng chiều hỏng mà header ở trên gọi là ĐẮT NHẤT.
 */
const REGEX_PRECEDERS = new Set([...'([{},;:=!&|?+-*%^~<>'])
const REGEX_KEYWORDS = new Set([
  'return', 'typeof', 'instanceof', 'in', 'of', 'case', 'do', 'else',
  'yield', 'await', 'new', 'delete', 'void', 'throw',
])

function regexAllowed(lastSig, text, at) {
  if (lastSig === '') return true
  if (REGEX_PRECEDERS.has(lastSig)) return true
  if (!IDENT_CHAR.test(lastSig)) return false
  const m = /[A-Za-z_$][A-Za-z0-9_$]*$/.exec(text.slice(Math.max(0, at - 24), at).trimEnd())
  return m ? REGEX_KEYWORDS.has(m[0]) : false
}

const blank = (chars, s, e) => {
  for (let i = s; i < e && i < chars.length; i += 1) if (chars[i] !== '\n') chars[i] = ' '
}

/** Vùng JS/TS — che `//` và `/* *\/`, theo dõi chuỗi · template literal · regex literal. */
function maskScript(text, from, to, chars) {
  let i = from
  let state = 'code'
  let quote = ''
  let lastSig = ''
  const interp = []

  while (i < to) {
    const ch = text[i]
    if (state === 'code') {
      if (text.startsWith('/*', i)) {
        let end = text.indexOf('*/', i + 2)
        end = end === -1 || end + 2 > to ? to : end + 2
        blank(chars, i, end)
        i = end
        continue
      }
      if (text.startsWith('//', i)) {
        let end = text.indexOf('\n', i)
        if (end === -1 || end > to) end = to
        blank(chars, i, end)
        i = end
        continue
      }
      if (ch === '"' || ch === "'") {
        state = 'string'
        quote = ch
        i += 1
        continue
      }
      if (ch === '`') {
        state = 'template'
        i += 1
        continue
      }
      if (ch === '/' && regexAllowed(lastSig, text, i)) {
        let j = i + 1
        let inClass = false
        let closed = false
        while (j < to) {
          const c = text[j]
          if (c === '\\') {
            j += 2
            continue
          }
          if (c === '\n') break
          if (c === '[') inClass = true
          else if (c === ']') inClass = false
          else if (c === '/' && !inClass) {
            closed = true
            break
          }
          j += 1
        }
        if (closed) {
          j += 1
          while (j < to && /[a-z]/.test(text[j])) j += 1
          lastSig = '/'
          i = j
          continue
        }
        // Không đóng được ⇒ đoán sai, đây là phép chia. Đi tiếp một bước.
      }
      if (interp.length) {
        if (ch === '{') interp[interp.length - 1] += 1
        else if (ch === '}') {
          if (interp[interp.length - 1] === 0) {
            interp.pop()
            state = 'template'
            i += 1
            continue
          }
          interp[interp.length - 1] -= 1
        }
      }
      if (!/\s/.test(ch)) lastSig = ch
      i += 1
      continue
    }
    if (state === 'string') {
      if (ch === '\\') {
        i += 2
        continue
      }
      // Một dấu nháy lẻ phải đóng TRONG CÙNG MỘT DÒNG — đúng ngữ nghĩa JS. Không có luật
      // này thì một dấu nháy trong văn xuôi (`don't`) nuốt phần còn lại của tệp.
      if (ch === '\n' || ch === quote) {
        state = 'code'
        lastSig = quote
        i += 1
        continue
      }
      i += 1
      continue
    }
    // template literal
    if (ch === '\\') {
      i += 2
      continue
    }
    if (text.startsWith('${', i)) {
      interp.push(0)
      state = 'code'
      i += 2
      continue
    }
    if (ch === '`') {
      state = 'code'
      lastSig = '`'
      i += 1
      continue
    }
    i += 1
  }
}

/** Vùng CSS — chỉ `/* *\/`. ⚠️ `//` KHÔNG phải comment: `url(//host/x.png)` là một URL. */
function maskStyle(text, from, to, chars) {
  let i = from
  while (i < to) {
    if (text.startsWith('/*', i)) {
      let end = text.indexOf('*/', i + 2)
      end = end === -1 || end + 2 > to ? to : end + 2
      blank(chars, i, end)
      i = end
      continue
    }
    i += 1
  }
}

/**
 * Vùng template — che `<!-- -->`, và CHỈ ở vùng văn bản.
 *
 * 🔴 `<!--` KHÔNG mở comment ở mọi vị trí. `<div title="a <!-- b">` là HTML hợp lệ, và
 * một phép so chuỗi trần ở đó mở một comment không có thật rồi làm mù phần còn lại của
 * vùng (`check-i18n.mjs:628-635`). Kèm theo, `indexOf('-->')` phải bị chặn bởi `to`.
 */
function maskTemplate(text, from, to, chars) {
  let i = from
  let state = 'text'
  let quote = ''
  while (i < to) {
    const ch = text[i]
    if (state === 'text') {
      if (text.startsWith('<!--', i)) {
        const end = text.indexOf('-->', i + 4)
        const stop = end === -1 || end + 3 > to ? to : end + 3
        blank(chars, i, stop)
        i = stop
        continue
      }
      if (ch === '<' && /[A-Za-z/]/.test(text[i + 1] ?? '')) {
        state = 'tag'
        i += 1
        continue
      }
      i += 1
      continue
    }
    if (state === 'tag') {
      if (ch === '"' || ch === "'") {
        state = 'attr'
        quote = ch
        i += 1
        continue
      }
      if (ch === '>') {
        state = 'text'
        i += 1
        continue
      }
      i += 1
      continue
    }
    if (ch === quote) {
      state = 'tag'
      i += 1
      continue
    }
    i += 1
  }
}

/**
 * Số lần thấy một `<script`/`<style` KHÔNG ở đầu dòng. Đếm và in ra, không bỏ im lặng.
 * Xem lý do ở `vueRegions`.
 */
let looseBlockTags = 0

/**
 * `<script>` và `<style>` của một SFC; phần còn lại là template. (`check-i18n.mjs:374-392`)
 *
 * 🔴 NEO Ở ĐẦU DÒNG — và đây là một lỗ đã bị khai thác, không phải một lo xa.
 *
 * Bản đầu quét `/<(script|style)\b[^>]*>/gi` trên văn bản **THÔ**, không phải văn bản đã
 * che. Hệ quả: một `<style>` nằm trong một mustache hay trong giá trị attribute — ví dụ
 * `<p>{{ '<style>' }}</p>` — mở một vùng CSS **giả** chạy tới thẻ đóng THẬT, và mọi
 * `@click` nằm giữa hai chỗ đó biến mất khỏi `templates`, tức khỏi Kiểm A. Đúng loại lỗi
 * không-trạng-thái mà tệp này tự hào đã tránh cho `<!--` ở `maskTemplate`.
 *
 * Khối top-level của một SFC luôn ở cột 0 — đó là hình dạng mà `@vue/compiler-sfc` và
 * `vite` đọc. Neo `^` (cờ `m`) đóng lỗ mà không cần một lượt phân tích hai pha.
 *
 * ⚠️ Cái giá: một `<script>` thật viết thụt đầu dòng sẽ bị bỏ qua. Nên nó được **ĐẾM và
 * IN RA** thay vì bỏ im lặng — cùng kỷ luật với `nonLiteralOwnerCalls` ở Kiểm E.
 */
function vueRegions(text) {
  const regions = []
  const anchored = /^<(script|style)\b[^>]*>/gim
  const anyTag = /<(script|style)\b[^>]*>/gi
  let loose
  while ((loose = anyTag.exec(text))) {
    const atLineStart = loose.index === 0 || text[loose.index - 1] === '\n'
    if (!atLineStart) looseBlockTags += 1
  }
  const open = anchored
  let m
  while ((m = open.exec(text))) {
    const kind = m[1].toLowerCase()
    const start = m.index + m[0].length
    // ⚠️ Thẻ đóng khớp KHÔNG phân biệt hoa thường và cho phép khoảng trắng: `</STYLE>`
    // kéo vùng tới hết tệp nếu dùng `indexOf('</style>')`.
    const close = new RegExp(`</\\s*${kind}\\s*>`, 'gi')
    close.lastIndex = start
    const c = close.exec(text)
    const end = c ? c.index : text.length
    regions.push({ kind, start, end })
    open.lastIndex = end
  }
  return regions.sort((a, b) => a.start - b.start)
}

function maskFile(text, isVue) {
  const chars = text.split('')
  if (!isVue) {
    maskScript(text, 0, text.length, chars)
    return { masked: chars.join(''), templates: [] }
  }
  const regions = vueRegions(text)
  const templates = []
  let cursor = 0
  for (const r of regions) {
    if (r.start > cursor) {
      maskTemplate(text, cursor, r.start, chars)
      templates.push({ start: cursor, end: r.start })
    }
    if (r.kind === 'script') maskScript(text, r.start, r.end, chars)
    else maskStyle(text, r.start, r.end, chars)
    cursor = r.end
  }
  if (cursor < text.length) {
    maskTemplate(text, cursor, text.length, chars)
    templates.push({ start: cursor, end: text.length })
  }
  return { masked: chars.join(''), templates }
}

const parsed = []
for (const file of [...vueFiles, ...tsFiles]) {
  let text
  try {
    text = readFileSync(file, 'utf8')
  } catch (err) {
    abort(`tệp \`${posix(file)}\``, err)
  }
  const isVue = file.toLowerCase().endsWith('.vue')
  const { masked, templates } = maskFile(text, isVue)
  parsed.push({ file, text, masked, templates, isVue })
}

const positionOf = (text, index) => {
  const before = text.slice(0, index)
  const line = before.split('\n').length
  const col = index - (before.lastIndexOf('\n') + 1) + 1
  return { line, col }
}
const at = (p, index) => {
  const { line, col } = positionOf(p.text, index)
  return `${posix(p.file)}:${line}:${col}`
}
/** Trích 60 ký tự quanh chỗ vi phạm, một dòng, để chẩn đoán đọc được ngay ở log CI. */
const excerpt = (text, index) =>
  text
    .slice(Math.max(0, index - 30), Math.min(text.length, index + 30))
    .replace(/\s+/g, ' ')
    .trim()

// ═════════════════════════════════════════════════════════════════════════════════
// Bóc thuộc tính trong vùng template — CÓ TRẠNG THÁI
// ═════════════════════════════════════════════════════════════════════════════════
//
// Không `replace` ngây thơ. Lượt review Story 1.5 đã dựng lại được ba lỗ thủng của một
// bộ quét không trạng thái (char literal chứa `"`, regex literal, `<!--` trong giá trị
// attribute). Giá trị attribute được phép chứa `>` — `@click="a > b ? f() : g()"` là
// template Vue hợp lệ — nên chỗ đóng của một giá trị là DẤU NHÁY, không phải `>`.
const ATTR_NAME_START = /[@:A-Za-z_]/
const ATTR_NAME_CHAR = /[@:A-Za-z0-9_.\-[\]]/

function attributesIn(masked, from, to) {
  const out = []
  let i = from
  let state = 'text'
  while (i < to) {
    const ch = masked[i]
    if (state === 'text') {
      if (ch === '<' && /[A-Za-z/]/.test(masked[i + 1] ?? '')) {
        state = 'tag'
        i += 1
        continue
      }
      i += 1
      continue
    }
    // state === 'tag'
    if (ch === '>') {
      state = 'text'
      i += 1
      continue
    }
    if (!ATTR_NAME_START.test(ch)) {
      i += 1
      continue
    }
    let j = i
    while (j < to && ATTR_NAME_CHAR.test(masked[j])) j += 1
    const name = masked.slice(i, j)
    let k = j
    while (k < to && /\s/.test(masked[k])) k += 1
    if (masked[k] !== '=') {
      i = j
      continue
    }
    k += 1
    while (k < to && /\s/.test(masked[k])) k += 1
    const q = masked[k]
    if (q === '"' || q === "'") {
      let e = k + 1
      while (e < to && masked[e] !== q) e += 1
      out.push({ name, value: masked.slice(k + 1, e), index: k + 1 })
      i = Math.min(e + 1, to)
      continue
    }
    let e = k
    while (e < to && !/[\s>]/.test(masked[e])) e += 1
    out.push({ name, value: masked.slice(k, e), index: k })
    i = e
  }
  return out
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm A — `@click` chỉ được là `dispatch(\'<id>\')` (AC1)')
// ═════════════════════════════════════════════════════════════════════════════════

/** ⚠️ Neo hai đầu (`^…$`): "chứa một lời gọi dispatch" KHÁC "là một lời gọi dispatch". */
const DISPATCH_ONLY_RE = /^\s*dispatch\(\s*(['"])([^'"]*)\1\s*\)\s*$/
const CLICK_ATTR_RE = /^(@|v-on:)click(\.[A-Za-z0-9.\-]+)?$/

/**
 * 🔴 CÁC CÁCH VIẾT KHÁC CỦA MỘT LISTENER CLICK — và cả ba đều từng đi qua cổng XANH.
 *
 * `CLICK_ATTR_RE` chỉ biết đúng hai cách đánh vần. Nhưng Vue 3 cài listener click thật
 * qua ít nhất ba đường nữa, và cả ba đều cài được thao tác TẠI CHỖ — đúng thứ AC1 tồn
 * tại để cấm:
 *   - `:onClick="() => {…}"` / `onClick="…"` — prop `on*` được runtime coi là listener;
 *   - `v-on="{ click: … }"` — dạng object, một handler click hạng nhất;
 *   - `@[evtName]="…"` / `v-on:[evtName]="…"` — tên sự kiện động, không đọc tĩnh được.
 *
 * Cả ba KHÔNG thể chứng minh tĩnh là "đúng một `dispatch()`", nên chúng bị TỪ CHỐI
 * chứ không được suy đoán. Đây là chỗ §Task 9 nói *"miễn trừ — nếu có — viết ngay trong
 * script"*: không có miễn trừ nào ở đây, có một lời từ chối.
 */
const OPAQUE_CLICK_RES = [
  [/^:?on[Cc]lick$/, '`:onClick` / `onClick` là một listener click của Vue 3'],
  [/^v-on$/, '`v-on="{ click: … }"` là dạng object của một listener click'],
  [/^(@|v-on:)\[/, 'tên sự kiện ĐỘNG — không đọc tĩnh được, nên không chứng minh được'],
]

const clickAttrs = []
let aBad = 0
for (const p of parsed) {
  if (!p.isVue) continue
  for (const region of p.templates) {
    for (const a of attributesIn(p.masked, region.start, region.end)) {
      if (CLICK_ATTR_RE.test(a.name)) {
        clickAttrs.push({ p, a })
        continue
      }
      const opaque = OPAQUE_CLICK_RES.find(([re]) => re.test(a.name))
      if (opaque) {
        fail(`${at(p, a.index)} — \`${a.name}\` là một thao tác click KHÔNG kiểm được tĩnh`)
        detail(`… ${excerpt(p.text, a.index)} …`)
        detail(`${opaque[1]}, nên nó lách được luật "\`@click\` phải là đúng một \`dispatch()\`".`)
        detail('Viết lại thành `@click="dispatch(\'<id>\')"`. AD-34 §1: handler chuột chỉ được')
        detail('`dispatch` một command ĐÃ ĐĂNG KÝ, không tự cài đặt thao tác tại chỗ.')
        aBad += 1
      }
    }
  }
}

for (const { p, a } of clickAttrs) {
  const m = DISPATCH_ONLY_RE.exec(a.value)
  if (m) continue
  fail(`${at(p, a.index)} — \`${a.name}\` không phải một lời gọi \`dispatch('<id>')\``)
  detail(`… ${excerpt(p.text, a.index)} …`)
  detail('AD-34 §1: handler chuột chỉ được `dispatch` một command ĐÃ ĐĂNG KÝ, không tự cài')
  detail('đặt thao tác tại chỗ. Đăng ký thao tác ở `src/commands/index.ts` rồi gọi id của nó.')
  aBad += 1
}
// 🔴 SÀN NỘI DUNG. `aBad === 0` trên một danh sách RỖNG là một lượt "đạt" không kiểm gì
// cả — và nó là thứ làm cho một lỗ ở tầng quét (vùng `<style>` giả) trở nên im lặng.
if (clickAttrs.length < CLICK_FLOOR) {
  abort(
    `thuộc tính \`@click\` quét được — ${clickAttrs.length} (sàn ${CLICK_FLOOR})`,
    new Error(
      'Ba tab chế độ ở `App.vue` phải luôn có mặt. Ít hơn sàn nghĩa là tầng quét đã mất\n' +
        'một vùng template — kiểm `vueRegions` và `maskTemplate` trước khi hạ sàn.',
    ),
  )
}
if (aBad === 0) {
  pass(
    `${clickAttrs.length} thuộc tính \`@click\` trên ${vueFiles.length} tệp \`.vue\` — ` +
      'tất cả là một lời gọi `dispatch()` đơn',
  )
}
detail('giới hạn đã khai: chỉ `@click`; `@keydown`/`@input` KHÔNG thuộc luật này (xem đầu tệp)')
detail(
  `\`:onClick\` · \`v-on="{click}"\` · \`@[dyn]\` bị TỪ CHỐI (không kiểm được tĩnh) · ` +
    `${looseBlockTags} thẻ \`<script/style>\` không ở đầu dòng đã bỏ qua`,
)

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm B — văn phạm id, và id phải TỒN TẠI trong bộ đăng ký (AC2)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 CHÉP ĐÚNG `KEY_RE` của `check-i18n.mjs:781` — không phải một biến thể. AD-34 nói
// command id *"cùng hình dạng khoá `vi.json`"*, và "cùng hình dạng" nghĩa là CÙNG MỘT
// BIỂU THỨC. Lượt review Story 1.5 đã bắt một ca hai phép kiểm cưỡng chế hai văn phạm
// khoá khác nhau cho cùng một thứ; đừng tạo ca thứ hai.
const KEY_RE = /^[a-z0-9]+(\.[a-z0-9_]+)+$/

const DISPATCH_CALL_RE = /\bdispatch\(\s*(['"])([^'"]*)\1\s*\)/g
const dispatched = []
for (const p of parsed) {
  const re = new RegExp(DISPATCH_CALL_RE.source, 'g')
  let m
  while ((m = re.exec(p.masked))) dispatched.push({ p, id: m[2], index: m.index })
}

/**
 * ⚠️ `dispatch(<biến>)` và `` dispatch(`mode.${x}`) `` KHÔNG đọc tĩnh được — nhưng chúng
 * phải được ĐẾM và IN RA, không bỏ qua im lặng. Đây đúng là kỷ luật mà bộ quét owner
 * của Kiểm E đã áp cho `nonLiteralOwnerCalls`; Kiểm B thiếu nó, nên một lượt chuyển sang
 * id động làm cả phép kiểm này rỗng đi mà không ai thấy.
 *
 * Không FAIL: một `dispatch` id động là hợp lệ về nguyên tắc (Story 1.21 sẽ gọi từ màn
 * hình gán phím). Lưới cho ca đó là `dispatch()` NÉM lúc chạy với id chưa đăng ký.
 */
const DISPATCH_ANY_RE = /\bdispatch\(\s*(['"`]?)/g
let nonLiteralDispatchCalls = 0
for (const p of parsed) {
  const re = new RegExp(DISPATCH_ANY_RE.source, 'g')
  let m
  while ((m = re.exec(p.masked))) {
    const literal = /^\s*dispatch\(\s*(['"])([^'"]*)\1\s*\)/.test(p.masked.slice(m.index))
    if (!literal) nonLiteralDispatchCalls += 1
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm C — hành vi thật của `src/commands/registry.ts` (AC1, AC2, AC6)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ Đường đi này tồn tại nhờ Node ≥ 22.18 bóc kiểu TypeScript mặc định — và đó chính là
// lý do `registry.ts` KHÔNG được `import` gì (doc-comment ở đầu tệp đó ghi đầy đủ).
//
// `import()` thất bại ⇒ `abort()` và exit 1, KHÔNG phải bỏ qua rồi exit 0.

if (!existsSync(REGISTRY_TS)) {
  abort(`\`${posix(REGISTRY_TS)}\``, new Error('Tệp không tồn tại — Kiểm C KHÔNG chạy được.'))
}

const loadTs = async (path, which) => {
  try {
    return await import(pathToFileURL(path).href)
  } catch (err) {
    abort(
      `\`${posix(path)}\` — ${which} KHÔNG chạy được`,
      new Error(
        `${err?.message || err}\n\n` +
          `Node đang chạy: ${process.version}. Phép kiểm này cần Node ≥ 22.18 (bóc kiểu ` +
          'TypeScript mặc định), và tệp phải là cú pháp "erasable-only": không `enum`, ' +
          'không `namespace`, không parameter property, không `import` một module ' +
          'cần bundler (`.vue`, `.json`, `vue`).',
      ),
    )
  }
}

const registryMod = await loadTs(REGISTRY_TS, 'Kiểm C')
if (typeof registryMod.createRegistry !== 'function') {
  abort(`\`${posix(REGISTRY_TS)}\``, new Error('không export `createRegistry` — Kiểm C KHÔNG chạy được.'))
}
const { createRegistry } = registryMod

let cBad = 0
/**
 * 🔴 MỌI khẳng định đi qua hai helper này, và đó là hệ quả trực tiếp của một lượt nghiệm
 * thu ở Story 1.5: một `try` bọc cả khối làm một phép kiểm ĐỎ mặc áo LỖI HẠ TẦNG. Ở đây
 * `abort()` chỉ dành cho ca `import()` gãy; mọi thứ khác là `fail` CÓ TÊN rồi đi tiếp để
 * các mệnh đề còn lại vẫn được chấm.
 */
/**
 * ⚠️ NHẬN DIỆN LỖI, không chỉ đếm "có ném hay không".
 *
 * Bản đầu `catch {}` trần: bất kỳ giá trị nào ném ra cũng tính là đạt. Hệ quả là một hồi
 * quy làm `register()` ném VÔ ĐIỀU KIỆN biến cả bảy mệnh đề *"⇒ ném"* thành xanh, và bộ
 * kiểm chỉ đỏ sau đó vì một lời gọi hợp lệ ở dưới làm sập script — báo dưới dạng stack
 * trace chứ không phải một FAIL có tên.
 *
 * Hai lớp: lỗi phải là `Error` có thông báo thật *(một `TypeError` trần từ một dòng sập
 * ngẫu nhiên vẫn qua được lớp này, nên nó không đủ một mình)*, và `needle` — khi có —
 * phải nằm trong thông báo. Lớp thật sự đóng ca "ném vô điều kiện" là `expectNoThrow`
 * ngay dưới.
 */
const expectThrow = (what, fn, needle) => {
  try {
    fn()
  } catch (err) {
    if (!(err instanceof Error) || err.message.trim() === '') {
      fail(`${what} — có ném, nhưng KHÔNG phải \`Error\` có thông báo: \`${String(err)}\``)
      cBad += 1
      return
    }
    if (needle !== undefined && !err.message.includes(needle)) {
      fail(`${what} — ném đúng, nhưng thông báo không nhắc \`${needle}\`: \`${err.message}\``)
      cBad += 1
      return
    }
    pass(what)
    return
  }
  fail(`${what} — KHÔNG ném`)
  cBad += 1
}
/**
 * 🔴 ĐỐI CHỨNG DƯƠNG — đây mới là thứ bắt được "ném vô điều kiện".
 *
 * Một bộ kiểm chỉ gồm các mệnh đề *"⇒ ném"* là một bộ kiểm mà cài đặt `throw new
 * Error('x')` ở dòng đầu tiên sẽ vượt qua trọn vẹn. Đường hợp lệ phải được khẳng định
 * tường minh, và một lần ném ở đó phải là FAIL **CÓ TÊN**, không phải một lượt sập script.
 */
const expectNoThrow = (what, fn) => {
  try {
    fn()
    pass(what)
  } catch (err) {
    fail(`${what} — NÉM ở đường hợp lệ: ${err?.message || String(err)}`)
    cBad += 1
  }
}
const expectEq = (what, got, want) => {
  if (got === want) return
  fail(`${what} — nhận \`${String(got)}\`, phải là \`${String(want)}\``)
  cBad += 1
}

{
  const noop = () => {}
  const spec = (id, extra = {}) => ({ id, labelKey: `command.${id}`, run: noop, ...extra })

  // 🔴 ĐỐI CHỨNG DƯƠNG, đứng TRƯỚC mọi mệnh đề "⇒ ném". Nếu `register()` ném vô điều
  // kiện thì dòng này đỏ CÓ TÊN, thay vì cả bộ kiểm xanh rồi script sập ở đâu đó dưới.
  expectNoThrow('đường HỢP LỆ: `register()` một spec đúng ⇒ KHÔNG ném', () => {
    createRegistry().register(spec('mode.library'))
  })

  expectThrow(
    'id trùng ⇒ ném (AC2)',
    () => {
      const r = createRegistry()
      r.register(spec('mode.library'))
      r.register(spec('mode.library'))
    },
    'đã đăng ký rồi',
  )
  expectThrow('id sai văn phạm (`mode_library`) ⇒ ném (AC2)', () => {
    createRegistry().register(spec('mode_library'))
  })
  expectThrow('id không có tiền tố miền (`library`) ⇒ ném (AC2)', () => {
    createRegistry().register(spec('library'))
  })
  expectThrow('id viết hoa (`Mode.Library`) ⇒ ném (AC2)', () => {
    createRegistry().register(spec('Mode.Library'))
  })
  expectThrow('`labelKey` rỗng ⇒ ném', () => {
    createRegistry().register({ id: 'a.b', labelKey: '  ', run: noop })
  })
  expectThrow('thiếu `run` ⇒ ném (không đăng ký command rỗng cho đủ số)', () => {
    createRegistry().register({ id: 'a.b', labelKey: 'command.a.b' })
  })
  expectThrow('`dispatch` một id LẠ ⇒ ném (AC1, nửa cưỡng chế lúc chạy)', () => {
    createRegistry().dispatch('khong.co')
  })

  const r = createRegistry()
  let fired = 0
  r.register(spec('mode.library', { keys: ['Mod+1'] }))
  r.register(spec('mode.workspace', { keys: ['Mod+2'] }))
  r.register(spec('focus.next_panel', { run: () => { fired += 1 } }))
  r.register(spec('demo.keys_rong', { keys: [] }))

  expectEq('`has()` đúng với id đã đăng ký', r.has('mode.library'), true)
  expectEq('`has()` đúng với id lạ', r.has('khong.co'), false)

  r.dispatch('focus.next_panel')
  expectEq('`dispatch` chạy đúng handler', fired, 1)

  expectEq(
    '`list()` giữ THỨ TỰ ĐĂNG KÝ',
    r.list().map((s) => s.id).join(' → '),
    'mode.library → mode.workspace → focus.next_panel → demo.keys_rong',
  )

  // AC6 — và ⚠️ ca `keys: []` phải nằm trong tập, không chỉ ca `keys` vắng mặt.
  expectEq(
    '`unbound()` trả ĐÚNG tập command thiếu phím (AC6)',
    r.unbound().map((s) => s.id).sort().join(' · '),
    'demo.keys_rong · focus.next_panel',
  )

  // Story 1.21 dựng màn hình gán phím trên chính hai hàm này — một tham chiếu vào kho nội
  // bộ nghĩa là màn hình đó sửa được registry mà không đi qua `register()`.
  //
  // ⚠️ Hai mệnh đề, không phải một, và mệnh đề thứ hai là thứ lượt nghiệm thu Task 10 tìm
  // ra: một cài đặt trả về CÙNG MỘT mảng đệm ở mọi lời gọi vẫn qua được phép so độ dài
  // (đệm được dựng lại mỗi lần), nhưng nó vẫn là một tham chiếu dùng chung — hai chỗ gọi
  // `list()` giẫm lên nhau. So ĐỊNH DANH là phép kiểm bắt được ca đó.
  const snapshot = r.list()
  snapshot.length = 0
  expectEq('`list()` trả BẢN SAO, không phải kho nội bộ', r.list().length, 4)
  expectEq('`list()` trả một mảng MỚI ở mỗi lời gọi', r.list() !== r.list(), true)
  expectEq('`unbound()` trả một mảng MỚI ở mỗi lời gọi', r.unbound() !== r.unbound(), true)
  const one = r.list()[0]
  try {
    one.id = 'da.bi.doi'
  } catch {
    /* strict mode ném khi ghi vào object đã đóng băng — đúng ý đồ */
  }
  expectEq('spec đã đăng ký là BẤT BIẾN', r.list()[0].id, 'mode.library')
}
// ── C phần hai: `src/commands/focus.ts` — nửa KHAI BÁO của AC4 ───────────────────
//
// ⚠️ GIỚI HẠN, ghi ngay cạnh chỗ cưỡng chế: khối này nghiệm thu CƠ CHẾ (khai trùng ném ·
// owner rỗng ném · `enter` dời focus tường minh · vòng xoay đúng thứ tự). Nó KHÔNG
// nghiệm thu mệnh đề *"`document.activeElement` không bao giờ là `body`"* — đó là hành vi
// của một webview thật, và chốt tự kêu ở `focus.ts` cộng nghiệm thu tay là thứ canh nó.
// Không đọc khối này thành "AC4 đã đạt".
const FOCUS_TS = join(SRC_ROOT, 'commands', 'focus.ts')
if (!existsSync(FOCUS_TS)) {
  abort(`\`${posix(FOCUS_TS)}\``, new Error('Tệp không tồn tại — Kiểm C KHÔNG chạy được.'))
}
const focusMod = await loadTs(FOCUS_TS, 'Kiểm C')
if (typeof focusMod.createFocusRegistry !== 'function') {
  abort(`\`${posix(FOCUS_TS)}\``, new Error('không export `createFocusRegistry` — Kiểm C KHÔNG chạy được.'))
}
{
  const { createFocusRegistry } = focusMod
  /** Phần tử giả: `enter()` chỉ cần một thứ có `focus()`. */
  const fakeEl = (name) => ({ name, focused: 0, focus() { this.focused += 1 } })

  expectThrow('owner rỗng ⇒ ném (AD-34 §2)', () => createFocusRegistry().declare('', () => null))
  expectThrow('owner sai văn phạm (`modeLibrary`) ⇒ ném', () =>
    createFocusRegistry().declare('modeLibrary', () => null))
  expectThrow('owner khai TRÙNG ⇒ ném', () => {
    const f = createFocusRegistry()
    f.declare('mode.library', () => null)
    f.declare('mode.library', () => null)
  })

  const f = createFocusRegistry()
  const source = fakeEl('source')
  const editor = fakeEl('editor')
  f.declare('mode.library', () => null)
  f.declare('panel.source', () => source)
  f.declare('panel.editor', () => editor)

  expectEq('`owners()` giữ thứ tự KHAI BÁO', f.owners().join(' → '), 'mode.library → panel.source → panel.editor')

  // Cửa vào một owner LẠ phải trả `false` và không ném — một chốt chống rơi focus không
  // được tự nó làm sập ứng dụng.
  const realError = console.error
  const errors = []
  console.error = (...a) => errors.push(a.join(' '))
  try {
    expectEq('`enter()` owner lạ ⇒ `false`', f.enter('mode.khong_co'), false)
    expectEq('`enter()` owner lạ ⇒ ghi `console.error` nêu đích danh owner', errors.length >= 1, true)
    expectEq(
      'thông báo nêu ĐÍCH DANH owner',
      errors.some((e) => e.includes('mode.khong_co')),
      true,
    )
    // Điểm vào khai rồi nhưng phần tử chưa có trong DOM — cũng phải kêu, không im lặng.
    expectEq('`enter()` khi phần tử chưa dựng ⇒ `false`', f.enter('mode.library'), false)

    expectEq('`enter()` gọi `el.focus()` TƯỜNG MINH', f.enter('panel.source'), true)
    expectEq('phần tử đã thật sự nhận `focus()`', source.focused, 1)
    expectEq('`current()` theo dõi owner vào gần nhất', f.current(), 'panel.source')

    // AC6 — handler thật của `focus.next_panel`: xoay vòng, và CHỈ trong nhóm `panel.`.
    expectEq('`next()` xoay sang panel kế tiếp', f.next('panel.'), true)
    expectEq('`next()` dừng đúng ở panel thứ hai', f.current(), 'panel.editor')
    f.next('panel.')
    expectEq('`next()` quay vòng về panel đầu', f.current(), 'panel.source')

    f.release('panel.editor')
    expectEq('`release()` gỡ owner khỏi sổ', f.has('panel.editor'), false)
    expectEq('`owners()` sau `release`', f.owners().join(' → '), 'mode.library → panel.source')
  } finally {
    console.error = realError
  }
}

if (cBad === 0) {
  pass('registry: bảy ca ném · thứ tự · unbound · bản sao · bất biến — tất cả đúng')
  pass('focus: ba ca ném · thứ tự khai · `enter` dời focus tường minh · vòng xoay · `release`')
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm D — HAI NỀN TẢNG, cùng một hợp âm (AC3, NFR14)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 ĐÂY LÀ PHÉP KIỂM DUY NHẤT TRONG TOÀN BỘ DỰ ÁN ĐỨNG GIỮA §Trap 1 VÀ NGƯỜI DÙNG
// WINDOWS. `⌘1` là ký hiệu macOS của một phím TRỪU TƯỢNG; trên Windows nó là `Ctrl+1`.
// Một `if (e.metaKey && e.key === '1')` đi qua CẢ HAI nền tảng của CI (không test nào
// chạm tầng bàn phím) rồi hỏng ở tay người dùng. Đừng rút gọn xuống một ca.

if (!existsSync(KEYS_TS)) {
  abort(`\`${posix(KEYS_TS)}\``, new Error('Tệp không tồn tại — Kiểm D KHÔNG chạy được.'))
}
const keysMod = await loadTs(KEYS_TS, 'Kiểm D')
if (typeof keysMod.createKeymap !== 'function') {
  abort(`\`${posix(KEYS_TS)}\``, new Error('không export `createKeymap` — Kiểm D KHÔNG chạy được.'))
}

let dBad = 0
{
  const fired = []
  /** Registry giả — Kiểm D nghiệm thu TẦNG BÀN PHÍM, không nghiệm thu lại registry. */
  const fakeRegistry = {
    list: () => [
      { id: 'mode.library', labelKey: 'command.mode.library', run: () => {}, keys: ['Mod+1'] },
      { id: 'ai.send', labelKey: 'command.ai.send', run: () => {}, keys: ['Mod+Shift+Enter'] },
      { id: 'read.bilingual', labelKey: 'command.read.bilingual', run: () => {}, keys: ['B'] },
    ],
    dispatch: (id) => fired.push(id),
    register: () => {},
    has: () => true,
    unbound: () => [],
  }

  const ev = (over) => {
    let prevented = 0
    return {
      event: {
        code: 'Digit1',
        metaKey: false,
        ctrlKey: false,
        shiftKey: false,
        altKey: false,
        preventDefault: () => {
          prevented += 1
        },
        ...over,
      },
      prevented: () => prevented,
    }
  }

  const check = (what, got, want) => {
    if (got === want) {
      pass(what)
      return
    }
    fail(`${what} — nhận \`${String(got)}\`, phải là \`${String(want)}\``)
    dBad += 1
  }

  const mac = keysMod.createKeymap(fakeRegistry, { isMac: true })
  const win = keysMod.createKeymap(fakeRegistry, { isMac: false })

  // Bốn mệnh đề của §Khung `check-commands.mjs`, không rút gọn.
  const a = ev({ metaKey: true })
  check('[macOS] `Mod+1` KHỚP khi `metaKey`', mac.handle(a.event), true)
  check('[macOS] khớp rồi thì `preventDefault()` đã gọi', a.prevented(), 1)
  check('[macOS] `Mod+1` KHÔNG khớp khi `ctrlKey`', mac.handle(ev({ ctrlKey: true }).event), false)
  const b = ev({ ctrlKey: true })
  check('[Windows] `Mod+1` KHỚP khi `ctrlKey`', win.handle(b.event), true)
  check('[Windows] khớp rồi thì `preventDefault()` đã gọi', b.prevented(), 1)
  const c = ev({ metaKey: true })
  check('[Windows] `Mod+1` KHÔNG khớp khi `metaKey`', win.handle(c.event), false)
  check('[Windows] không khớp thì KHÔNG đụng vào event', c.prevented(), 0)

  check('cả hai nền tảng cùng dispatch đúng một id', fired.join('|'), 'mode.library|mode.library')

  // Khớp bằng `event.code`, KHÔNG bằng `event.key`: trên bố cục không phải US, phím vật
  // lý `1` cho `event.key === '&'` (AZERTY). Một cài đặt đọc `key` trượt ca này.
  check(
    'khớp theo `event.code` kể cả khi `event.key` trôi theo bố cục',
    mac.handle(ev({ metaKey: true, key: '&' }).event),
    true,
  )

  // So khớp TUYỆT ĐỐI cả bốn cờ — `⌘⇧1` không được kích hoạt `Mod+1`.
  check('`Mod+Shift+1` KHÔNG kích hoạt `Mod+1`', mac.handle(ev({ metaKey: true, shiftKey: true }).event), false)
  check(
    '`Mod+Shift+Enter` khớp đúng ba cờ',
    mac.handle(ev({ code: 'Enter', metaKey: true, shiftKey: true }).event),
    true,
  )

  // 🔴 LUẬT VÙNG GÕ — chốt từ hôm nay dù chưa có ô nhập nào. Chế độ đọc dùng `M`, `B`,
  // `1 2 3` TRẦN (UX-DR46) và Editor của Epic 2 là một vùng gõ tự do.
  check('hợp âm TRẦN (`B`) khớp khi focus ngoài vùng gõ', mac.handle(ev({ code: 'KeyB' }).event), true)
  const typing = { tagName: 'TEXTAREA', isContentEditable: false }
  check(
    'hợp âm TRẦN (`B`) KHÔNG khớp khi focus trong vùng gõ',
    mac.handle(ev({ code: 'KeyB', target: typing }).event),
    false,
  )
  check(
    'hợp âm CÓ bổ trợ (`Mod+1`) VẪN khớp trong vùng gõ — đó là điều NFR17 hứa',
    mac.handle(ev({ metaKey: true, target: typing }).event),
    true,
  )

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 2.3 · AC21 — NHÁNH `isContentEditable` CHƯA TỪNG ĐƯỢC LÁI TỚI TRƯỚC ĐÂY
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // Ba ca trên lái vùng gõ bằng `{ tagName: 'TEXTAREA' }`, tức **nhánh thứ hai** của
  // `isTypingZone`. Nhánh **thứ nhất** — `el.isContentEditable === true` — có mặt trong mã từ
  // Story 1.6 mà **không một phép kiểm nào đi qua**, vì tới hết Story 2.2 kho không có một
  // `contenteditable` nào: Kiểm J cấm nó bằng máy, và không `<div contenteditable>` nào tồn tại
  // ở đâu khác.
  //
  // Story 2.3 làm nó thành thật: vùng gõ Editor là một `<span class="sent" contenteditable="true">`
  // — **một `<span>`**, nên nhánh `tagName` KHÔNG cứu được nó. Nếu nhánh thứ nhất hỏng, gõ chữ
  // `b` trong một bản dịch sẽ **bật chế độ song ngữ** (UX-DR46), và gõ `1` sẽ đổi chế độ.
  //
  // ⚠️ Hình dạng dưới đây là hình dạng THẬT, không một object tiện tay: `tagName: 'SPAN'` cộng
  // `isContentEditable: true` là đúng thứ `event.target` mang khi caret ở trong vùng gõ.
  //
  // 🔴 Và đây là chỗ luật *"đọc HÌNH DẠNG, không `instanceof HTMLElement`"* của `keys.ts` trả
  // công: nhánh này lái được bằng một object giả, nên nó có **lưới** thay vì chỉ có lời hứa.
  const editorZone = { tagName: 'SPAN', isContentEditable: true }
  check(
    'hợp âm TRẦN (`B`) KHÔNG khớp trong vùng gõ `contenteditable` của Editor (AC21)',
    mac.handle(ev({ code: 'KeyB', target: editorZone }).event),
    false,
  )
  check(
    'hợp âm CÓ bổ trợ (`Mod+1`) VẪN khớp trong vùng gõ `contenteditable` — NFR17',
    mac.handle(ev({ metaKey: true, target: editorZone }).event),
    true,
  )
  // Đối chứng ÂM: một `<span>` KHÔNG gõ được thì hợp âm trần phải khớp bình thường. Không có ca
  // này, một `isTypingZone` luôn trả `true` vẫn đi qua hai ca trên.
  check(
    'hợp âm TRẦN (`B`) VẪN khớp trên một `<span>` KHÔNG gõ được',
    mac.handle(ev({ code: 'KeyB', target: { tagName: 'SPAN', isContentEditable: false } }).event),
    true,
  )

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 2.3 · AC23 — LUẬT VÙNG GÕ NHƯỜNG ĐƯỜNG, NÓ KHÔNG *CHẶN* ĐƯỜNG
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // Đây là mệnh đề mà **cả AC23 đứng trên**, và nó là một hệ quả của **thứ tự hai dòng** trong
  // `keys.ts::handle`: phép kiểm vùng gõ `return false` **TRƯỚC** `event.preventDefault()`.
  //
  // Vì sao nó quyết định AC23: bốn command `selection.extend_*` dùng `Shift+Mũi tên`, và `Shift`
  // **không** phải phím bổ trợ chính ⇒ chúng thôi dispatch trong vùng gõ Editor. Nếu luật vùng
  // gõ cũng `preventDefault()`, hành vi **native** của `contenteditable` *(mở rộng vùng chọn)*
  // sẽ bị ăn theo — và lúc đó bôi đen bằng bàn phím trong Editor **chết hoàn toàn**: không
  // command, cũng không native. Vì nó KHÔNG `preventDefault`, engine tự mở rộng vùng chọn, rồi
  // lượt `keyup` của `Shift` mà `selectionContract.ts::attachSelectionWatcher` đang nghe vẫn
  // phát `lookup.lookup_selection` như thường.
  //
  // ⇒ Auto-Lookup **CÒN CHẠY** trên bề mặt Editor sau khi nó thành vùng gõ. Vế DOM của mệnh đề
  // đó đo ở `tests/frontend/editorAutoLookup.test.ts`; vế **này** — *"không nuốt sự kiện"* — là
  // một mệnh đề về `keys.ts`, nên nó ở đây, cùng chỗ với luật nó nói về (AC25).
  {
    const probe = ev({ code: 'KeyB', target: editorZone })
    check('hợp âm TRẦN trong vùng gõ: KHÔNG khớp', mac.handle(probe.event), false)
    check(
      'hợp âm TRẦN trong vùng gõ: KHÔNG `preventDefault` — native `contenteditable` phải chạy tiếp',
      probe.prevented(),
      0,
    )
    // Đối chứng: khi hợp âm THẬT SỰ khớp thì nó PHẢI nuốt sự kiện.
    const swallowed = ev({ metaKey: true })
    check('hợp âm khớp: VẪN `preventDefault` đúng một lần', mac.handle(swallowed.event), true)
    check('hợp âm khớp: đếm `preventDefault` = 1', swallowed.prevented(), 1)
  }

  // 🔴 GIỮ PHÍM — và phép kiểm này đo THAO TÁC, không đo giá trị trả về.
  //
  // Bản đầu khẳng định `handle()` trả `false` khi `repeat: true`. Sai thuộc tính: cái phải
  // giữ là *"không dispatch lần hai"*, còn *"hợp âm đã khớp không rơi xuống webview"* thì
  // vẫn phải đúng — tức `preventDefault()` VẪN phải chạy. Đo bằng giá trị trả về gộp hai
  // mệnh đề đó thành một và ép cài đặt phải bỏ một trong hai.
  const beforeRepeat = fired.length
  const rep = ev({ metaKey: true, repeat: true })
  check('`repeat: true` vẫn là một hợp âm KHỚP', mac.handle(rep.event), true)
  check('`repeat: true` vẫn chặn hành vi mặc định (hợp âm đã khớp không rơi xuống webview)', rep.prevented(), 1)
  check('`repeat: true` KHÔNG lặp lại thao tác — không dispatch thêm', fired.length, beforeRepeat)

  // 🔴 IME — ứng dụng dịch tiếng Việt, đường này đi hằng ngày từ Epic 2. Một lượt commit
  // composition phát `keydown` mang `code` vật lý; ăn nó như một hợp âm là ăn mất chữ.
  // ⚠️ Luật vùng gõ KHÔNG cứu được ca này: nó chỉ áp cho hợp âm thiếu bổ trợ chính.
  const beforeIme = fired.length
  const ime = ev({ metaKey: true, isComposing: true })
  check('`isComposing: true` ⇒ KHÔNG khớp', mac.handle(ime.event), false)
  check('`isComposing: true` ⇒ KHÔNG đụng vào event', ime.prevented(), 0)
  check('`isComposing: true` ⇒ không dispatch', fired.length, beforeIme)

  // 🔴 Luật vùng gõ hỏi phím bổ trợ CHÍNH (`⌘`/`Ctrl`), không hỏi "có bổ trợ nào không".
  // `Shift+B` khớp đúng keydown mà người dùng tạo ra khi gõ chữ "B" hoa; `Alt+M` là
  // Option+M (gõ `µ` trên macOS). Cả hai từng bắn giữa câu và nuốt luôn ký tự.
  const shiftMap = keysMod.createKeymap(
    {
      list: () => [{ id: 'demo.shift', labelKey: 'command.demo.shift', keys: ['Shift+B'], run: () => {} }],
      dispatch: () => {},
    },
    { isMac: true },
  )
  check(
    '`Shift+B` KHÔNG khớp trong vùng gõ (thiếu bổ trợ chính)',
    shiftMap.handle(ev({ code: 'KeyB', shiftKey: true, target: typing }).event),
    false,
  )
  check(
    '`Shift+B` VẪN khớp ngoài vùng gõ',
    shiftMap.handle(ev({ code: 'KeyB', shiftKey: true }).event),
    true,
  )

  // Hợp âm viết lặp phím bổ trợ ⇒ ném. `'Mod+Mod+1'` biên dịch ra CÙNG `resolved` với
  // `'Mod+1'`, nên lỗi gõ chỉ lộ nếu hợp âm đúng tình cờ cũng được đăng ký.
  let dupMod = false
  try {
    keysMod.createKeymap(
      {
        list: () => [{ id: 'demo.dup', labelKey: 'command.demo.dup', keys: ['Mod+Mod+1'], run: () => {} }],
        dispatch: () => {},
      },
      { isMac: true },
    )
  } catch {
    dupMod = true
  }
  check('phím bổ trợ viết LẶP (`Mod+Mod+1`) ⇒ ném', dupMod, true)

  // Gắn keymap hai lần vào cùng một target ⇒ ném. Không có canh gác thì hai listener
  // capture cùng nghe và MỌI hợp âm dispatch hai lần — `setMode` idempotent nên ca đó ẩn.
  const fakeTarget = { addEventListener() {}, removeEventListener() {} }
  const detach = keysMod.attachKeymap(mac, fakeTarget)
  let doubleAttach = false
  try {
    keysMod.attachKeymap(mac, fakeTarget)
  } catch {
    doubleAttach = true
  }
  check('gắn keymap HAI LẦN vào một target ⇒ ném', doubleAttach, true)
  detach()
  let reattached = true
  try {
    keysMod.attachKeymap(mac, fakeTarget)()
  } catch {
    reattached = false
  }
  check('gỡ rồi thì gắn lại được', reattached, true)

  // Hai command giành một phím ⇒ ném lúc dựng keymap, không im lặng.
  let clashed = false
  try {
    keysMod.createKeymap(
      {
        ...fakeRegistry,
        list: () => [
          { id: 'a.mot', labelKey: 'command.a.mot', run: () => {}, keys: ['Mod+1'] },
          { id: 'a.hai', labelKey: 'command.a.hai', run: () => {}, keys: ['Mod+1'] },
        ],
      },
      { isMac: true },
    )
  } catch {
    clashed = true
  }
  check('hai command giành một hợp âm ⇒ ném', clashed, true)

  // Một tên phím ngoài bảng phải NÉM, không phải lặng lẽ không bao giờ khớp.
  let unknown = false
  try {
    keysMod.createKeymap(
      { ...fakeRegistry, list: () => [{ id: 'a.b', labelKey: 'command.a.b', run: () => {}, keys: ['Mod+Khong'] }] },
      { isMac: true },
    )
  } catch {
    unknown = true
  }
  check('tên phím không phân giải được ⇒ ném', unknown, true)

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 1.21 — CỜ `repeatable` (`deferred-work.md:656`, Ice ký nhận 2026-08-11)
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // Ba ca ngay trên khẳng định `repeat: true` **không** lặp thao tác — đúng, và đó là mặc
  // định. Nhánh còn lại phải có lưới riêng, nếu không một bản cài đặt bỏ quên `repeatable`
  // hoàn toàn vẫn xanh: giữ `Shift+→` sẽ mở rộng vùng chọn đúng một ký tự rồi đứng im, và
  // không cổng nào đỏ.
  {
    const fired = []
    const map = keysMod.createKeymap(
      {
        ...fakeRegistry,
        list: () => [
          { id: 'a.once', labelKey: 'command.a.once', run: () => {}, keys: ['Mod+1'] },
          {
            id: 'a.again',
            labelKey: 'command.a.again',
            run: () => {},
            keys: ['Shift+ArrowRight'],
            repeatable: true,
          },
        ],
        dispatch: (id) => fired.push(id),
      },
      { isMac: true },
    )
    map.handle({ code: 'ArrowRight', shiftKey: true, repeat: true, preventDefault: () => {} })
    check('`repeatable: true` ⇒ keydown lặp VẪN dispatch (giữ `Shift+→` bôi đen được)', fired.join(''), 'a.again')

    fired.length = 0
    map.handle({ code: 'Digit1', metaKey: true, repeat: true, preventDefault: () => {} })
    check('cùng keymap, command KHÔNG khai cờ ⇒ keydown lặp vẫn bị chặn', fired.length, 0)
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 1.21 — `overrides`, VÀ BA TRẠNG THÁI MÀ AC8 ĐỨNG LÊN
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // Phép phân biệt sống ở **sự có mặt của khoá**, không ở giá trị. Không có lưới ở đây thì
  // một bản cài đặt viết `overrides[id] ?? spec.keys` vẫn xanh trên hai trạng thái đầu và
  // sai im lặng ở trạng thái thứ ba — nút *"bỏ gán"* sẽ lặng lẽ dựng lại hợp âm mặc định.
  {
    const base = {
      ...fakeRegistry,
      list: () => [{ id: 'a.b', labelKey: 'command.a.b', run: () => {}, keys: ['Mod+1'] }],
    }
    const chordsOf = (overrides) =>
      keysMod
        .createKeymap(base, { isMac: true }, overrides)
        .bindings()
        .map((b) => b.chord)
        .join(' · ')

    check('`overrides` VẮNG MẶT ⇒ hành vi cũ từng dòng một (tương thích ngược)', chordsOf(undefined), 'Mod+1')
    check('khoá vắng mặt trong `overrides` ⇒ rơi về `spec.keys` (= "trả về mặc định")', chordsOf({}), 'Mod+1')
    check('khoá có, MẢNG RỖNG ⇒ KHÔNG hợp âm nào (= "cố ý không có phím")', chordsOf({ 'a.b': [] }), '')
    check('khoá có, có phần tử ⇒ hợp âm của người dùng thắng', chordsOf({ 'a.b': ['Mod+K'] }), 'Mod+K')
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 1.21 — `chordFromEvent` / `formatChord`, VÀ ĐÂY LÀ LƯỚI NFR14 DUY NHẤT
  //    CỦA CHÚNG
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // Story 1.21 thêm hai hàm **phụ thuộc nền tảng** vào tầng này. Cùng lý lẽ mở đầu Kiểm D:
  // không test nào khác trong dự án chạm tầng bàn phím, và CI hai nền tảng của Story 1.3
  // chỉ `cargo test` + build. Một `chordFromEvent` đọc `metaKey` thẳng tay đi qua **cả
  // hai** nhánh CI rồi hỏng ở tay người dùng Windows.

  if (typeof keysMod.chordFromEvent !== 'function' || typeof keysMod.formatChord !== 'function') {
    abort(
      `\`${posix(KEYS_TS)}\``,
      new Error('không export `chordFromEvent`/`formatChord` — vế Story 1.21 của Kiểm D KHÔNG chạy được.'),
    )
  }

  // ① VÒNG KHỨ HỒI, trên CẢ HAI nền tảng. `parseChord` không export được (nó là chi tiết
  //    của `createKeymap`), nên vòng khứ hồi đo qua chính `createKeymap`: dựng một keymap
  //    từ hợp âm mà `chordFromEvent` vừa sinh ra, rồi bắn lại đúng sự kiện gốc vào nó.
  //    Khớp ⇒ hai hàm là nghịch đảo của nhau **cho nền tảng đó**.
  const roundTripEvents = [
    { code: 'KeyD', metaKey: true },
    { code: 'Digit1', ctrlKey: true, altKey: true },
    { code: 'Comma', metaKey: true },
    { code: 'ArrowLeft', shiftKey: true, altKey: true },
    { code: 'Enter', metaKey: true, shiftKey: true },
  ]
  for (const isMac of [true, false]) {
    for (const event of roundTripEvents) {
      const chord = keysMod.chordFromEvent(event, { isMac })
      if (typeof chord !== 'string') {
        check(`vòng khứ hồi (isMac=${isMac}) — \`${event.code}\` phải cho ra một hợp âm`, typeof chord, 'string')
        continue
      }
      const seen = []
      const map = keysMod.createKeymap(
        {
          ...fakeRegistry,
          list: () => [{ id: 'x.y', labelKey: 'command.x.y', run: () => {}, keys: [chord] }],
          dispatch: (id) => seen.push(id),
        },
        { isMac },
      )
      check(
        `vòng khứ hồi (isMac=${isMac}): \`${event.code}\` ⇒ \`${chord}\` ⇒ khớp lại chính sự kiện đó`,
        map.handle({ ...event, preventDefault: () => {} }) && seen.length === 1,
        true,
      )
    }
  }

  // ⚠️ Và cùng một sự kiện phải cho HAI hợp âm khác nhau giữa hai nền tảng — nếu không thì
  //    vòng khứ hồi ở trên vẫn xanh với một bản cài đặt bỏ qua `isMac` hoàn toàn.
  check(
    '`Mod` phụ thuộc nền tảng — `⌘D` là `Mod+D` trên macOS, `Meta+D` ở nơi khác',
    `${keysMod.chordFromEvent({ code: 'KeyD', metaKey: true }, { isMac: true })} / ` +
      `${keysMod.chordFromEvent({ code: 'KeyD', metaKey: true }, { isMac: false })}`,
    'Mod+D / Meta+D',
  )

  // ② PHÍM NGOÀI BẢNG ⇒ `null`, KHÔNG ném (AC11). Chỗ gọi là một cử chỉ người dùng, không
  //    một lỗi lập trình — ném ở đó biến một lượt bấm phím thành một sự cố.
  check('`F1` ⇒ `null` (phím ngoài bảng, không ném)', keysMod.chordFromEvent({ code: 'F1' }, { isMac: true }), null)
  check(
    'keydown CHỈ CÓ phím bổ trợ ⇒ `null` (chưa gõ xong, đừng chốt)',
    keysMod.chordFromEvent({ code: 'MetaLeft', metaKey: true }, { isMac: true }),
    null,
  )
  check(
    'lượt commit của bộ gõ ⇒ `null` (đây là ứng dụng dịch tiếng Việt)',
    keysMod.chordFromEvent({ code: 'KeyD', metaKey: true, isComposing: true }, { isMac: true }),
    null,
  )

  // ③ `formatChord` — chuỗi ĐỌC, và nó cũng phải đổi theo nền tảng.
  check('`formatChord` trên macOS', keysMod.formatChord('Mod+Alt+ArrowRight', { isMac: true }), '⌥⌘→')
  check('`formatChord` ngoài macOS', keysMod.formatChord('Mod+Alt+ArrowRight', { isMac: false }), 'Ctrl+Alt+→')

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 1.21 · Quyết định #3 — MÃ HOÁ TRÊN ĐĨA AN TOÀN THEO CẤU TRÚC
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // `src/main.ts::toBindings` tách hợp âm bằng **dấu phẩy, không escape**, và
  // `deferred-work.md:241` ghi mã hoá đó là TẠM vì một hợp âm chứa dấu phẩy sẽ vỡ nó.
  // Phép đo đóng mục nợ đó: phím dấu phẩy viết là `Comma` — một **tên chữ cái** — nên
  // không hợp âm hợp lệ nào chứa `,`. Ba dòng dưới đây biến phép đo thành **cơ chế**; nếu
  // không, mệnh đề *"an toàn theo cấu trúc"* chỉ đúng cho tới ngày ai đó thêm một tên phím
  // chứa dấu phẩy vào `NAMED_CODES` và không cổng nào đỏ.
  //
  // ⚠️ Bảng `NAMED_CODES` **không** export và không nên export — nó là cửa duy nhất vào
  // tầng phím. Nên phép kiểm đọc nó **từ chính mã nguồn** rồi lái từng `code` qua
  // `chordFromEvent`, và khẳng định trên KẾT QUẢ. Đọc từ nguồn thay vì chép một danh sách
  // vào script là cả điểm: một hàng mới thêm vào bảng ngày mai **tự động** bị kiểm, còn
  // một bản chép sẽ trôi khỏi sự thật trong đúng hai story.
  {
    const src = readFileSync(KEYS_TS, 'utf8')
    const table = /const NAMED_CODES\s*:[^=]*=\s*\{([\s\S]*?)\n\}/.exec(src)
    if (table === null) {
      abort(`\`${posix(KEYS_TS)}\``, new Error('không đọc được `NAMED_CODES` từ mã nguồn — phép kiểm dấu phẩy KHÔNG chạy được.'))
    }
    const codes = [...table[1].matchAll(/^\s*([A-Za-z][A-Za-z0-9]*)\s*:/gm)].map((m) => m[1])
    check('đọc được bảng `NAMED_CODES` từ mã nguồn (≥ 20 hàng)', codes.length >= 20, true)

    const commaCarrying = []
    // Cộng cả hai nhánh sinh mã của `keyToCode` — chữ và số — vì một dấu phẩy lọt vào qua
    // đó cũng phá cùng một mã hoá.
    const probe = [
      ...codes,
      ...'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('').map((c) => `Key${c}`),
      ...'0123456789'.split('').map((d) => `Digit${d}`),
    ]
    for (const code of probe) {
      const chord = keysMod.chordFromEvent({ code }, { isMac: true })
      if (typeof chord === 'string' && chord.includes(',')) commaCarrying.push(code)
    }
    check(
      `không tên phím nào trong ${probe.length} phím chứa \`,\` — mã hoá "ngăn nhau bằng dấu phẩy" an toàn THEO CẤU TRÚC`,
      commaCarrying.join(' · '),
      '',
    )
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm E — nhãn có trong `vi.json`, sổ điểm vào focus khớp mã nguồn (AC4)')
// ═════════════════════════════════════════════════════════════════════════════════

let catalog
try {
  catalog = JSON.parse(readFileSync(VI_JSON, 'utf8'))
} catch (err) {
  // `abort()`, không `fail()`: một `vi.json` không parse được là lỗi hạ tầng của Kiểm E.
  abort(`\`${posix(VI_JSON)}\``, err)
}

const indexMod = await loadTs(COMMANDS_INDEX_TS, 'Kiểm E')
for (const name of ['installCommands', 'commandRegistry', 'FOCUS_OWNERS']) {
  if (indexMod[name] === undefined) {
    abort(`\`${posix(COMMANDS_INDEX_TS)}\``, new Error(`không export \`${name}\` — Kiểm E KHÔNG chạy được.`))
  }
}

let eBad = 0
const eFail = (m) => {
  fail(m)
  eBad += 1
}

let registered = []
try {
  // ⚠️ Nạp BỘ COMMAND THẬT CỦA SẢN PHẨM, không một bản chép trong script. `setMode` được
  // tiêm vào chính vì lý do này: `src/commands/index.ts` phải nạp được bằng Node thuần.
  indexMod.installCommands({ setMode: () => {}, isMac: true })
  registered = indexMod.commandRegistry.list()
} catch (err) {
  abort('Kiểm E — `installCommands()` ném ngay khi đăng ký bộ command khởi động', err)
}

/**
 * 🔴 BỘ COMMAND THẬT PHẢI DỰNG ĐƯỢC TRÊN CẢ HAI NỀN TẢNG — và đây là lỗ mà Kiểm D KHÔNG
 * đóng, dù comment ở `ci.yml` khẳng định ngược lại.
 *
 * Kiểm D lái hai nhánh `isMac`, nhưng trên `fakeRegistry` — bộ command của SẢN PHẨM chỉ
 * được phân giải với `isMac: true` *(và `installCommands()` chỉ gọi được một lần, theo
 * đúng thiết kế)*. Vì `claimed` khoá theo hợp âm ĐÃ PHÂN GIẢI, một xung đột có thể tồn
 * tại trên đúng MỘT nhánh: `Mod+1` là `Meta+Digit1` trên macOS và `Ctrl+Digit1` ở nơi
 * khác, nên thêm một command mang `keys: ['Ctrl+1']` cho ra cổng XANH trên cả hai nền
 * tảng CI và một lần NÉM lúc khởi động **chỉ trên Windows** — tức cửa sổ trắng, vì lượt
 * ném đó xảy ra trước `mount()`.
 *
 * ⚠️ Dựng lại keymap trực tiếp từ `commandRegistry` (không gọi lại `installCommands`) là
 * cách duy nhất chạm được nhánh kia mà không phá luật "đăng ký đúng một lần".
 */
for (const isMac of [true, false]) {
  const nen = isMac ? 'macOS' : 'Windows/Linux'
  try {
    keysMod.createKeymap(indexMod.commandRegistry, { isMac })
    pass(`bộ command THẬT dựng được keymap trên ${nen} — không hợp âm nào giành nhau`)
  } catch (err) {
    eFail(`bộ command THẬT KHÔNG dựng được keymap trên ${nen}: ${err?.message || String(err)}`)
    detail('Một xung đột hợp âm chỉ tồn tại trên MỘT nền tảng vẫn là một cửa sổ trắng ở nền tảng đó:')
    detail('`installCommands()` chạy TRƯỚC `mount()` trong `src/main.ts`.')
  }
}

/**
 * 🔴 KHOÁ `t()` Ở CHỖ GỌI — không cổng nào canh chỗ này trước lượt review 2026-08-04.
 *
 * Kiểm E chỉ duyệt `labelKey` của command ĐÃ ĐĂNG KÝ, còn `check-i18n.mjs` chỉ kiểm hình
 * dạng catalog và hành vi `resolve.ts`. Nên 5 trong 9 khoá mà Story 1.6 thêm — ba câu
 * trạng thái chế độ và hai tiêu đề panel — không có lưới nào: đổi `t('mode.library.
 * status')` thành `t('mode.libary.status')` và cả hai cổng vẫn xanh, người dùng thấy
 * khoá thô hiện ra màn hình *(`resolve.ts` cố ý không sập với khoá thiếu — AC4 Story 1.5,
 * nên chỗ DUY NHẤT bắt được là ở đây)*.
 *
 * ⚠️ Giới hạn thật, ghi ra thay vì để im: `t(props.titleKey)` KHÔNG đọc tĩnh được. Những
 * lời gọi như vậy được ĐẾM và IN RA — cùng kỷ luật với `nonLiteralOwnerCalls`. Thuộc tính
 * `title-key="…"` literal thì đọc được, và đó là đường mà `PanelFrame` thật sự nhận khoá.
 */
const T_LITERAL_RE = /\bt\(\s*(['"])([^'"]*)\1\s*\)/g
const T_ANY_RE = /\bt\(\s*/g
/**
 * Thuộc tính mang một KHOÁ `vi.json` literal xuống một component vỏ.
 *
 * ⚠️ Danh sách này phải mọc theo vỏ. Story 1.14 gỡ `title-key` khỏi `PanelFrame` (tiêu đề
 * chuyển lên tab — §Quyết định #4A) và thêm `status-key`; nếu chỉ đổi tên prop mà không
 * đổi ở đây thì khoá trạng thái của **bốn** panel mới rơi ra khỏi mọi lưới, và một
 * `status-key="panel.lokup.status"` gõ sai sẽ hiện khoá thô ra màn hình với cả bốn cổng
 * xanh (`resolve.ts` cố ý không sập với khoá thiếu — AC4 Story 1.5).
 */
const KEY_ATTR_RE = /^:?title-key$|^:?titleKey$|^:?status-key$|^:?statusKey$/
let nonLiteralTCalls = 0
const callSiteKeys = []

for (const p of parsed) {
  const lit = new RegExp(T_LITERAL_RE.source, 'g')
  let m
  while ((m = lit.exec(p.masked))) callSiteKeys.push({ p, key: m[2], index: m.index, how: "t('…')" })
  const any = new RegExp(T_ANY_RE.source, 'g')
  while ((m = any.exec(p.masked))) {
    if (!/^\bt\(\s*(['"])([^'"]*)\1\s*\)/.test(p.masked.slice(m.index))) nonLiteralTCalls += 1
  }
  if (!p.isVue) continue
  for (const region of p.templates) {
    for (const a of attributesIn(p.masked, region.start, region.end)) {
      if (!KEY_ATTR_RE.test(a.name)) continue
      // `:title-key="…"` là một biểu thức, không phải một khoá — đếm, đừng đoán.
      if (a.name.startsWith(':')) {
        nonLiteralTCalls += 1
        continue
      }
      callSiteKeys.push({ p, key: a.value.trim(), index: a.index, how: `\`${a.name}\`` })
    }
  }
}

for (const c of callSiteKeys) {
  if (!Object.prototype.hasOwnProperty.call(catalog, c.key)) {
    eFail(`${at(c.p, c.index)} — khoá \`${c.key}\` (qua ${c.how}) KHÔNG có trong \`src/i18n/vi.json\``)
    detail('`resolve.ts` không sập với khoá thiếu — nó hiện KHOÁ NGUYÊN VĂN ra màn hình.')
    detail('Thêm khoá vào `vi.json`, hoặc sửa chỗ gõ sai.')
    oBad += 1
  }
}
if (callSiteKeys.length > 0) {
  pass(
    `${callSiteKeys.length} khoá \`t()\` ở chỗ gọi — đều có trong \`vi.json\` ` +
      `(${nonLiteralTCalls} lời gọi truyền biến: không đọc tĩnh được)`,
  )
}

if (registered.length < COMMAND_FLOOR) {
  abort(
    `bộ command đã đăng ký — ${registered.length} command (sàn ${COMMAND_FLOOR})`,
    new Error('Một bộ đăng ký rỗng làm Kiểm B, D và E xanh mà không kiểm gì cả.'),
  )
}

const registeredIds = new Set(registered.map((s) => s.id))

for (const spec of registered) {
  if (!KEY_RE.test(spec.id)) eFail(`command \`${spec.id}\` sai văn phạm id — phải khớp \`${KEY_RE.source}\``)
  // §Quyết định thiết kế #4 — tiền tố `command.` là quy ước, không phải sở thích: nó chừa
  // chỗ cho `command.<id>.hint` ở màn hình gán phím của Story 1.21.
  if (spec.labelKey !== `command.${spec.id}`) {
    eFail(`command \`${spec.id}\` có \`labelKey\` là \`${spec.labelKey}\`, quy ước là \`command.${spec.id}\``)
  }
  if (!Object.prototype.hasOwnProperty.call(catalog, spec.labelKey)) {
    eFail(`\`${spec.labelKey}\` KHÔNG có trong \`src/i18n/vi.json\` — nhãn sẽ hiện ra khoá nguyên văn`)
    detail('`resolve.ts` không sập với khoá thiếu (AC4 Story 1.5), nên chỗ duy nhất bắt được nó là đây.')
  }
}
if (eBad === 0) {
  pass(`${registered.length} command — id đúng văn phạm, \`labelKey\` đúng quy ước và có mặt trong \`vi.json\``)
}

// AC6 — `unbound()` phải có phần tử THẬT, nếu không nhánh có nghĩa của nó không bao giờ chạy.
const unboundIds = indexMod.commandRegistry.unbound().map((s) => s.id)
if (unboundIds.length === 0) {
  eFail('`unbound()` trả MẢNG RỖNG — AC6 chưa được chứng minh trên bộ command thật')
  detail('§Quyết định thiết kế #5: `focus.next_panel` cố ý không gán phím, và handler của nó CHẠY THẬT.')
} else {
  pass(`\`unbound()\` trên bộ command thật: ${unboundIds.join(' · ')} (AC6 có nhánh chạy thật)`)
}

// ── Sổ điểm vào focus, đối chiếu HAI CHIỀU với mã nguồn ──────────────────────────
const owners = indexMod.FOCUS_OWNERS
let oBad = 0
if (!Array.isArray(owners) || owners.length === 0) {
  eFail('`FOCUS_OWNERS` rỗng — mỗi chế độ và mỗi panel phải khai một điểm vào (AD-34 §2)')
  oBad += 1
} else {
  const seen = new Set()
  for (const owner of owners) {
    if (typeof owner !== 'string' || owner.trim() === '') {
      eFail(`owner rỗng trong \`FOCUS_OWNERS\``)
      oBad += 1
      continue
    }
    if (!KEY_RE.test(owner)) {
      eFail(`owner \`${owner}\` sai văn phạm — phải khớp \`${KEY_RE.source}\``)
      oBad += 1
    }
    if (seen.has(owner)) {
      eFail(`owner \`${owner}\` khai TRÙNG trong \`FOCUS_OWNERS\``)
      oBad += 1
    }
    seen.add(owner)
  }
}

/**
 * Bóc owner từ mã nguồn, và **phân biệt KHAI BÁO với THAM CHIẾU**.
 *
 * 🔴 Sự phân biệt này là kết quả của lượt nghiệm thu Task 10, ca E5. Bản trước gom cả
 * `declareFocus` · `enterFocus` · `releaseFocus` vào một rổ "đã dùng", nên một chế độ
 * QUÊN gọi `declareFocus()` mà vẫn còn `enterFocus()`/`releaseFocus()` thì cổng **xanh**
 * — đúng ca mà AD-34 §2 (*"mỗi chế độ và mỗi panel KHAI BÁO điểm vào focus"*) tồn tại để
 * chặn, và đúng ca dẫn thẳng tới focus rơi về `body`.
 *
 * Hai rổ:
 *   - **khai báo** — `declareFocus('x', …)`, hoặc `owner="x"` trong template (vỏ
 *     `PanelFrame` nhận owner qua prop rồi tự `declareFocus(props.owner, …)`);
 *   - **tham chiếu** — thêm `enterFocus('x')` và `releaseFocus('x')`.
 *
 * ⚠️ `declareFocus(props.owner, …)` KHÔNG khớp regex và đó là đúng: một vỏ dùng lại được
 * thì owner phải là biến. Số lời gọi không-literal được ĐẾM và in ra, để việc bỏ qua
 * chúng không im lặng.
 */
const FOCUS_CALL_RE = /\b(declare|enter|release)Focus\(\s*(?:(['"])([^'"]*)\2|([A-Za-z_$][\w$.]*))/g
const OWNER_ATTR_RE = /\bowner\s*=\s*(['"])([^'"]*)\1/g
const declaredOwners = new Map()
const referencedOwners = new Map()
let nonLiteralOwnerCalls = 0
const noteOwner = (map, owner, where) => {
  if (!map.has(owner)) map.set(owner, where)
}
/**
 * Component nào tự khai điểm vào bằng một BIẾN — tức `declareFocus(props.owner, …)`.
 * Đây là nửa còn thiếu của phép nối attribute ↔ component ở dưới.
 */
const declaresViaVariable = new Map()

/** ⚠️ Loại trừ ĐỊNH NGHĨA hàm. Xem lý do ở khối `FOCUS_CALL_RE` bên trên. */
const isFunctionDefinition = (text, index) => /\bfunction\s+$/.test(text.slice(Math.max(0, index - 32), index))

for (const p of parsed) {
  const calls = new RegExp(FOCUS_CALL_RE.source, 'g')
  let m
  while ((m = calls.exec(p.masked))) {
    // 🔴 `export function declareFocus(owner: FocusOwner, …)` KHỚP regex này, và bản đầu
    // đếm cả ba chữ ký hàm ở `src/commands/index.ts` là "lời gọi truyền biến". Con số in
    // ra tồn tại đúng để việc bỏ qua chúng KHÔNG im lặng — đếm sai thì nó không phục vụ
    // được mục đích đó nữa (5 in ra, thật chỉ có 2).
    if (isFunctionDefinition(p.masked, m.index)) continue
    if (m[3] === undefined) {
      nonLiteralOwnerCalls += 1
      if (m[1] === 'declare') declaresViaVariable.set(p.file, at(p, m.index))
      continue
    }
    noteOwner(referencedOwners, m[3], at(p, m.index))
    if (m[1] === 'declare') noteOwner(declaredOwners, m[3], at(p, m.index))
  }
}

/**
 * 🔴 THUỘC TÍNH `owner=` KHÔNG PHẢI MỘT KHAI BÁO — và đây là lỗ nghiêm trọng nhất của
 * bản đầu, đóng theo hướng CHẶT mà Ice chốt ở lượt review 2026-08-04.
 *
 * Bản đầu đẩy `owner="x"` vào **cả** `referencedOwners` **lẫn** `declaredOwners`, nên
 * chiều ngược lại ở dưới — chiều tồn tại để bắt một chế độ/panel QUÊN khai điểm vào — bị
 * chính thuộc tính đó thoả mãn. Dựng lại được: xoá hẳn `declareFocus(props.owner, …)`
 * khỏi `PanelFrame.vue` và cổng vẫn in `OK 5 điểm vào focus … đều được declareFocus()`,
 * trong khi thực tế KHÔNG panel nào khai, vòng xoay rỗng, và focus không bao giờ tới
 * được một panel. Thêm một `<div owner="panel.ghost" />` trần cũng qua.
 *
 * Luật mới: `owner="x"` chỉ là một THAM CHIẾU. Nó được coi là khai báo **khi và chỉ khi**
 * component mang thuộc tính đó tự gọi `declareFocus(<biến>, …)` trong tệp của chính nó.
 */
const COMPONENT_TAG_RE = /<([A-Z][A-Za-z0-9]*)\b([^>]*)>/g
const byComponentName = new Map()
for (const p of parsed) {
  if (!p.isVue) continue
  byComponentName.set(basename(p.file, '.vue'), p)
}

for (const p of parsed) {
  if (!p.isVue) continue
  for (const region of p.templates) {
    const slice = p.masked.slice(region.start, region.end)
    const tags = new RegExp(COMPONENT_TAG_RE.source, 'g')
    let g
    while ((g = tags.exec(slice))) {
      const [, tagName, attrText] = g
      const attrs = new RegExp(OWNER_ATTR_RE.source, 'g')
      let a
      while ((a = attrs.exec(attrText))) {
        const owner = a[2]
        const where = at(p, region.start + g.index)
        noteOwner(referencedOwners, owner, where)
        const target = byComponentName.get(tagName)
        if (target === undefined) {
          eFail(`${where} — \`owner="${owner}"\` đặt trên \`<${tagName}>\`, không tìm thấy tệp component`)
          detail(`Cổng cần \`src/**/${tagName}.vue\` để xác nhận component đó TỰ khai điểm vào.`)
          oBad += 1
          continue
        }
        if (!declaresViaVariable.has(target.file)) {
          eFail(`${where} — \`<${tagName}>\` nhận \`owner="${owner}"\` nhưng KHÔNG tự \`declareFocus()\``)
          detail(`\`${posix(target.file)}\` phải gọi \`declareFocus(<biến owner>, …)\` — một thuộc tính`)
          detail('`owner=` một mình KHÔNG phải một khai báo. AD-34 §2 đòi mỗi panel KHAI điểm vào.')
          oBad += 1
          continue
        }
        noteOwner(declaredOwners, owner, where)
      }
    }
  }
}

for (const [owner, where] of referencedOwners) {
  if (!owners.includes(owner)) {
    eFail(`${where} — owner \`${owner}\` dùng trong mã nhưng KHÔNG có trong \`FOCUS_OWNERS\``)
    detail('Thêm nó vào `FOCUS_OWNERS` ở `src/commands/index.ts`, hoặc sửa chỗ gõ sai.')
    oBad += 1
  }
}
// 🔴 CHIỀU NGƯỢC LẠI, và nó đòi KHAI BÁO chứ không chỉ "có xuất hiện đâu đó". Đây là
// chiều bắt được một chế độ quên khai điểm vào — không con mắt nào bắt được nó sau khi
// có mười panel, và hậu quả của nó chính là focus rơi về `body` (AC4).
for (const owner of owners) {
  if (declaredOwners.has(owner)) continue
  if (referencedOwners.has(owner)) {
    eFail(`owner \`${owner}\` được dùng nhưng KHÔNG chỗ nào gọi \`declareFocus()\` cho nó`)
    detail(`thấy ở ${referencedOwners.get(owner)} — AD-34 §2 đòi mỗi chế độ và mỗi panel KHAI BÁO điểm vào.`)
  } else {
    eFail(`owner \`${owner}\` khai trong \`FOCUS_OWNERS\` nhưng KHÔNG chế độ/panel nào dùng`)
    detail('Hoặc một chế độ quên gọi `declareFocus()`, hoặc mục này đã chết. Cả hai đều phải sửa.')
  }
  oBad += 1
}
if (oBad === 0) {
  pass(
    `${owners.length} điểm vào focus — không rỗng, không trùng, đúng văn phạm, đều được ` +
      `\`declareFocus()\` và không có mục thừa (${nonLiteralOwnerCalls} lời gọi truyền biến: ` +
      'vỏ dùng lại được)',
  )
}

// ── Kiểm B, phần phán quyết (cần bộ đăng ký của Kiểm E) ──────────────────────────
// 🔴 SÀN NỘI DUNG, cùng lý lẽ với `CLICK_FLOOR`: `bBad === 0` trên danh sách rỗng là một
// lượt "đạt" không kiểm gì. Ba lời gọi `dispatch()` literal của ba tab chế độ là số thật.
if (dispatched.length < DISPATCH_FLOOR) {
  abort(
    `lời gọi \`dispatch()\` literal quét được — ${dispatched.length} (sàn ${DISPATCH_FLOOR})`,
    new Error(
      `Ngoài ra thấy ${nonLiteralDispatchCalls} lời gọi truyền biến (không đọc tĩnh được).\n` +
        'Ít hơn sàn nghĩa là tầng quét hỏng, hoặc ba tab chế độ đã mất — kiểm trước khi hạ sàn.',
    ),
  )
}
let bBad = 0
for (const d of dispatched) {
  if (!KEY_RE.test(d.id)) {
    fail(`${at(d.p, d.index)} — id \`${d.id}\` sai văn phạm, phải khớp \`${KEY_RE.source}\``)
    detail('Khoá chấm có tiền tố miền, chữ thường và gạch dưới — CÙNG hình dạng khoá `vi.json`.')
    bBad += 1
    continue
  }
  if (!registeredIds.has(d.id)) {
    fail(`${at(d.p, d.index)} — \`dispatch('${d.id}')\` gọi một command CHƯA ĐĂNG KÝ`)
    detail(`Bộ đã đăng ký: ${[...registeredIds].join(' · ')}`)
    detail('`dispatch` ném lúc chạy, nhưng chỉ khi có người bấm đúng nút đó — đây là lưới bắt lỗi gõ sai.')
    bBad += 1
  }
}
if (bBad === 0) {
  console.log('')
  pass(
    `${dispatched.length} lời gọi \`dispatch()\` trên ${parsed.length} tệp — id đúng văn phạm và ` +
      `đều có trong bộ đăng ký (Kiểm B) · ${nonLiteralDispatchCalls} lời gọi truyền biến`,
  )
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm F — BA panel đăng ký hợp đồng vùng chọn (Story 1.18, AC2)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 VÌ SAO AC2 ĐÒI MỘT CỔNG, KHÔNG CHỈ ĐÒI MÃ
//
// `epics.md:1762` nói Auto-Lookup gắn vào *"một hợp đồng vùng chọn dùng chung cho **mọi**
// panel văn bản"*, và AI Translation + Editor *"nhận được cùng hành vi khi chúng có nội
// dung ở các epic sau, **không cần cài lại**"*. Một cài đặt chỉ chạy cho `SourcePanel`
// **đạt AC1 và trượt AC2**.
//
// Và khác biệt đó không để lại **triệu chứng nào**: Panel AI Translation và Editor hôm nay không
// có chữ, nên một lượt đăng ký thiếu ở đó im lặng tuyệt đối cho tới **Epic 2 / Epic 4** —
// hai epic sau, và tới lúc đó không ai nhớ AC này tồn tại. Đây chính xác là lớp lỗi mà AD-34
// §2 dựng sổ `FOCUS_OWNERS` đối chiếu HAI CHIỀU để chặn; cổng này áp cùng khuôn.
//
// ⚠️ **Đếm lời gọi LITERAL** — cùng luật `owner`/`status-key` của Kiểm E: cổng đọc TĨNH,
// nên một `useSelectionSurface(el, role)` với `role` là biến bị đếm rồi **bỏ qua**, tức
// mất lưới. Vai phải là một chuỗi viết thẳng.

/**
 * **Ba** panel của Workspace, và **SỐ LỜI GỌI MONG ĐỢI CỦA TỪNG TỆP**.
 * ⚠️ Chép từ `src/layout/workspaceLayout.ts`, cùng khuôn `PANEL_SUFFIXES`.
 *
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔵 2026-08-14 (Story 2.5b) — MỘT LƯỢT NỚI **CÓ CHỦ**, VÀ LÝ DO PHẢI ĐỌC ĐƯỢC
 * ═══════════════════════════════════════════════════════════════════════════════
 * Bản trước là một mảng tên tệp, và mệnh đề ① đọc *"mỗi panel **ĐÚNG MỘT** lời gọi"*. Mệnh
 * đề đó **hết đúng** ở `GridPanel.vue`: AC7 của Story 2.5b đòi **hai** bề mặt trong **một**
 * tệp — cột nguyên văn vai `'source'`, cột bản dịch vai `'display'` — vì lưới gộp hai panel
 * cũ thành một.
 *
 * 🔴 **Nới một con số KHÔNG được phép là nới một mệnh đề.** Nếu chỉ đổi ① thành *"ít nhất
 * một"* thì cổng mất luôn khả năng bắt một lời gọi **thừa** *(một bề mặt thứ ba lọt vào lưới)*
 * — và đó là đúng thứ AC7 cấm bằng chữ. ⇒ Số mong đợi ghi **theo từng tệp**, và mệnh đề ⑤
 * mới ở dưới canh **cả hai vai** bên trong `GridPanel.vue`.
 *
 * ⚠️ Vì sao ⑤ phải tồn tại: mệnh đề ③ đối chiếu theo **TỆP**, mà hai vai nay sống trong
 * **cùng một** tệp. Đảo vai giữa hai cột trong `GridPanel.vue` vì thế **đi lọt** ③ — cổng
 * canh yếu hẳn đi so với lúc hai panel còn rời nhau, và khoảng hở đó phải được bịt chứ không
 * được ghi rồi bỏ đấy.
 */
const SELECTION_PANEL_FILES = {
  'src/panels/GridPanel.vue': 2,
  'src/panels/LookupPanel.vue': 1,
  'src/panels/AiTranslationPanel.vue': 1,
}

/**
 * Sàn = 5, không 4 — lượt review 2026-08-07 bắt được rằng sàn cũ (4, đúng số panel trong
 * `SELECTION_PANEL_FILES`) không canh được gì cho bề mặt THỨ NĂM: `SourceHanViet.vue` cũng
 * gọi `useSelectionSurface` (AC11/AC12), nhưng nó KHÔNG nằm trong các panel Workspace nên
 * không được kiểm ① canh riêng. Với sàn cũ, xoá đúng lời gọi đó vẫn để lại 4 lời gọi —
 * ĐÚNG sàn, cổng xanh, và mất lưới cho toàn bộ đường bàn phím Hán Việt mà AC11/AC12 vừa
 * đóng. Sàn = SỐ THẬT hôm nay (AC13); Story 1.20/3.4 sẽ THÊM bề mặt, không bớt.
 *
 * 🔴 **NÂNG 6 — Story 1.19, và lời hứa ngay trên vừa được thu.** Bề mặt thứ sáu là bảng
 * Attribution (`src/AttributionOverlay.vue`): nó chứa **chữ thật** (ghi công, tên giấy
 * phép), nên nó rơi vào đúng lớp câu hỏi mà hợp đồng vùng chọn tồn tại để trả lời — một bề
 * mặt văn bản im lặng đứng ngoài sổ là đúng thứ AC2 của Story 1.18 dựng ra để chặn.
 * Vai `'display'`, KHÔNG `'source'`, cùng lý do Bẫy 1 đã bắt ở Panel Lookup.
 */
/*
 * 🔵 **HẠ 7 → 6, 2026-08-14 (Story 2.5b) — và đây là một lượt ĐẾM LẠI QUẦN THỂ, không một
 * lượt nới cổng cho mã đi lọt.**
 *
 * Story viết sẵn *"sàn KHÔNG đổi (hai lời gọi thay hai lời gọi)"*. **Phép đếm bác câu đó.**
 * Đếm thật trước lượt lật — **bảy** lời gọi:
 *   `AttributionOverlay` · `ShortcutsOverlay` · `SourcePanel` · **`SourceHanViet`** ·
 *   `AiTranslationPanel` · `EditorPanel` · `LookupPanel`
 * Lưới thay **BA** trong số đó *(`SourcePanel` + `SourceHanViet` + `EditorPanel`)* bằng
 * **HAI** *(hai cột của `GridPanel.vue`)* ⇒ **6**.
 *
 * 🔴 Vế `SourceHanViet` là chỗ dễ đếm sót nhất, và nó có lý do cấu trúc: AC7 đòi cột là **một**
 * bề mặt, nên bề mặt Hán Việt **nhượng** lượt đăng ký của nó cho cột và chỉ ghi tên vào
 * `panels/hanVietSurfaces.ts`. Nó **không biến mất** — nó đổi cửa. Xem doc-comment tệp đó.
 *
 * 🔵 **2026-08-15 (code review) — con số 6 ĐÚNG, nhưng nó chỉ vừa mới đúng.**
 *
 * Lượt rà đo được: cho tới 2026-08-15, phép đếm trên **sai một đơn vị**. `SourceHanViet.vue`
 * vẫn mang lời gọi `useSelectionSurface` của nó ở mặt chữ — nó nằm trong nhánh
 * `if (props.surfaceRole === 'own')`, và `SURFACE_CALL_RE` là **regex quét tĩnh**, không phân
 * tích `if`. Bằng chứng là chính cổng này: nó in ra `7 bề mặt đăng ký` trong khi sàn là 6.
 *
 * ⇒ Cổng mang đúng **một đơn vị dư**: bớt một bề mặt THẬT vẫn còn 6, **đúng sàn, vẫn xanh** —
 * tái diễn nguyên hình dạng cái lỗ mà đoạn ngay trên kể lại từ thời sàn = 4.
 *
 * Đóng bằng cách **gỡ nhánh `'own'`** *(mã chết: chỗ mount duy nhất là `GridPanel.vue:848` và
 * nó luôn khai `surface-role="cell"`)*, **không** bằng cách nâng sàn lên 7. Kèm theo, prop
 * `surfaceRole` bỏ giá trị mặc định — nếu không, một chỗ mount quên khai nó sẽ rơi vào một vai
 * không làm gì cả, im lặng, và phép đếm tĩnh **vẫn** cho 6. Lý do đầy đủ ở
 * `src/panels/SourceHanViet.vue` §prop `surfaceRole`.
 *
 * 🔴 **Bài học cho người sửa sàn sau:** con số ở đây phải đến từ một lượt **CHẠY CỔNG**, không
 * từ một phép trừ trên giấy. Phép trừ *"ba thay bằng hai"* đọc rất thuyết phục và nó sai.
 *
 * ⚠️ Sàn là **cận dưới**: nó canh chính CỔNG *(regex thôi khớp ⇒ mọi phép kiểm trên xanh
 * rỗng)*, không canh số bề mặt đúng. Story 1.20/3.4 sẽ THÊM bề mặt, không bớt.
 */
const SELECTION_SURFACE_FLOOR = 6

const SURFACE_CALL_RE = /useSelectionSurface\s*\(\s*[^,)]+,\s*'(source|display)'/g

/**
 * MỌI lời gọi, bất kể hình dạng đối số.
 *
 * ⚠️ Số *"truyền vai bằng biến"* tính bằng **phép TRỪ**, không bằng một regex phủ định thứ hai:
 * bản đầu viết `,\s*(?!['"])` và nó khớp **mọi** lời gọi — `\s*` lùi được về rỗng, nên phủ
 * định nhìn vào dấu **cách** thay vì vào dấu nháy, và cổng báo cả 5 lời gọi literal là
 * phi-literal (bắt lúc chạy cổng, 2026-08-07). Một phép trừ không có chỗ để trôi như vậy.
 */
const SURFACE_ANY_CALL_RE = /useSelectionSurface\s*\(/g

let fBad = 0
const surfaceCalls = []
let anySurfaceCalls = 0

for (const p of parsed) {
  if (!p.file.endsWith('.vue')) continue
  let m
  const re = new RegExp(SURFACE_CALL_RE.source, 'g')
  while ((m = re.exec(p.masked ?? p.text))) {
    surfaceCalls.push({ file: posix(p.file), role: m[1], index: m.index })
  }
  const any = new RegExp(SURFACE_ANY_CALL_RE.source, 'g')
  while ((m = any.exec(p.masked ?? p.text))) anySurfaceCalls += 1
}
const nonLiteralSurfaceCalls = anySurfaceCalls - surfaceCalls.length

// ① Mỗi panel trong sổ phải có ĐÚNG số lời gọi đã khai — chiều thứ nhất.
for (const [want, expected] of Object.entries(SELECTION_PANEL_FILES)) {
  const hits = surfaceCalls.filter((c) => c.file.endsWith(want))
  if (hits.length === 0) {
    fail(`${want} — không đăng ký hợp đồng vùng chọn (AC2)`)
    detail('Thêm `useSelectionSurface(ref, \'source\')` — hoặc `\'display\'` nếu bề mặt này CỐ Ý')
    detail('không được là nguồn (Panel Lookup, Bẫy 1). Một panel văn bản đứng ngoài sổ mà không ai')
    detail('giải thích là đúng thứ AC2 tồn tại để chặn.')
    fBad += 1
  } else if (hits.length !== expected) {
    fail(`${want} — ${hits.length} lời gọi đăng ký, phải đúng ${expected}`)
    detail('Số mong đợi khai theo TỪNG TỆP ở `SELECTION_PANEL_FILES`. Một lời gọi thừa là một')
    detail('bề mặt không ai xét vai; một lời gọi thiếu là một bề mặt chữ đứng ngoài sổ (AC2).')
    fBad += 1
  }
}

// ② Panel Lookup phải mang vai `display`, không `source` — AC3 / Bẫy 1.
const lookupCall = surfaceCalls.find((c) => c.file.endsWith('src/panels/LookupPanel.vue'))
if (lookupCall !== undefined && lookupCall.role !== 'display') {
  fail(`src/panels/LookupPanel.vue — đăng ký vai \`${lookupCall.role}\`, phải là \`display\``)
  detail('🔴 Bẫy 1 — VÒNG TỰ THAY THẾ. Panel Lookup tự nó chứa chữ (nghĩa, ví dụ, trích dẫn),')
  detail('nên làm nguồn nghĩa là bôi đen một nghĩa để đọc kỹ sẽ phát một lượt tra mới THAY')
  detail('CHÍNH đoạn đang đọc, cộng một hiệu ứng, cộng một lượt cuộn về đầu. AC3.')
  fBad += 1
}

// ③ Panel AI Translation và Panel Editor phải mang vai `display`, KHÔNG `source`.
//    🔴 Sprint Change Proposal 2026-08-13 (Ice ký) — FR21 thu hẹp. Hai panel này chứa TIẾNG
//    VIỆT ĐÃ DỊCH, còn từ điển nhúng là zh→vi / en→vi ⇒ một lượt tra ở đó trả 0 hàng, 0 lỗi,
//    0 ms rồi THAY MẤT kết quả đang hiện ở Panel Lookup. Cùng Bẫy 1 mà ② canh cho Panel
//    Lookup, chỉ tệ hơn một bậc vì thứ thay vào là RỖNG.
//
//    ⚠️ VÌ SAO MỆNH ĐỀ NÀY CẦN MỘT CỔNG: lật ngược về `'source'` là ĐÚNG MỘT TỪ, và nó đi qua
//    sạch mười một cổng — đo được lúc dựng lượt sửa này: đổi vai xong, `check:commands` XANH
//    ngay khi chỉ có ① và ②. Cộng thêm việc Panel AI Translation hôm nay KHÔNG CÓ CHỮ, nên
//    triệu chứng chỉ lộ ở Epic 4 — hai epic sau, lúc không ai còn nhớ proposal này. Đúng tiêu
//    chí §Critical Don't-Miss của `project-context.md`: "vi phạm được mà không cổng nào đỏ".
//
//    ⚠️ GIỚI HẠN THẬT: cổng này canh vai KHAI BÁO trong `.vue`, không canh hành vi lúc chạy.
//    Vế hành vi sống ở `tests/frontend/editorAutoLookup.test.ts` (đường vitest). Một mệnh đề,
//    một đường — AC25. Đừng nhân đôi.
const DISPLAY_ONLY_FILES = ['src/panels/AiTranslationPanel.vue']

for (const want of DISPLAY_ONLY_FILES) {
  const call = surfaceCalls.find((c) => c.file.endsWith(want))
  if (call !== undefined && call.role !== 'display') {
    fail(`${want} — đăng ký vai \`${call.role}\`, phải là \`display\` (FR21, 2026-08-13)`)
    detail('🔴 Panel này chứa TIẾNG VIỆT ĐÃ DỊCH. Từ điển nhúng là zh→vi / en→vi, nên một lượt')
    detail('tra ở đây trả 0 hàng — 0 lỗi, 0 ms — rồi THAY MẤT kết quả người dùng vừa tra từ')
    detail('Panel Source. Rỗng im lặng, đúng lớp lỗi trung tâm của dự án.')
    detail('⚠️ Sửa bằng cách đổi vai, KHÔNG bằng cách gỡ lời gọi: FR48 (Story 3.3) và FR60')
    detail('(Story 7.7) đọc vùng chọn ở đây bằng đường của riêng chúng.')
    fBad += 1
  }
}

// ④ Sàn NỘI DUNG — cùng lý lẽ `CLICK_FLOOR`: `fBad === 0` trên một danh sách rỗng là một
//    lượt xanh vô nghĩa (một lượt đổi tên hàm làm regex không khớp gì nữa).
if (surfaceCalls.length < SELECTION_SURFACE_FLOOR) {
  fail(`lời gọi đăng ký vùng chọn quét được — ${surfaceCalls.length} (sàn ${SELECTION_SURFACE_FLOOR})`)
  detail('Sàn này canh chính CỔNG: nếu regex thôi khớp thì mọi phép kiểm trên đều xanh rỗng.')
  fBad += 1
}

// ⑤ 🔴 `GridPanel.vue` phải có ĐÚNG MỘT `'source'` VÀ ĐÚNG MỘT `'display'` — Story 2.5b.
//
//    Đây là chỗ bịt khoảng hở mà lượt gộp panel vừa mở ra: ③ canh theo **tệp**, và hai vai
//    nay ở **cùng một** tệp nên ③ không nói được gì về việc vai nào thuộc cột nào.
//
//    ⚠️ Cổng vẫn **không** đọc được *"cột nào là cột nào"* — nó chỉ đọc được rằng có đúng một
//    lời gọi mỗi vai. Một lượt đảo `colSrc`/`colTgt` trong template đi qua được. Vế đó thuộc
//    đường **e2e** (bấm vào cột bản dịch không được phát lượt tra) và
//    `tests/frontend/editorAutoLookup.test.ts`. Ghi ra thay vì để người sau tưởng cổng phủ hết.
const GRID_VUE = 'src/panels/GridPanel.vue'
const gridCalls = surfaceCalls.filter((c) => c.file.endsWith(GRID_VUE))
if (gridCalls.length > 0) {
  for (const role of ['source', 'display']) {
    const n = gridCalls.filter((c) => c.role === role).length
    if (n !== 1) {
      fail(`${GRID_VUE} — ${n} lời gọi vai \`${role}\`, phải đúng MỘT (AC7 của Story 2.5b)`)
      detail('🔴 Cột nguyên văn là `source`, cột bản dịch là `display`. Đảo hai vai mở lại đúng')
      detail('lỗi commit `1c7658d`: tra trong bản dịch tiếng Việt cho 0 hàng rồi THAY MẤT kết')
      detail('quả người dùng vừa tra từ cột nguyên văn — rỗng im lặng.')
      fBad += 1
    }
  }
}

if (nonLiteralSurfaceCalls > 0) {
  console.log(
    `\x1b[33m⚠️  ${nonLiteralSurfaceCalls} lời gọi \`useSelectionSurface\` truyền vai bằng BIẾN\x1b[0m — ` +
      'cổng đọc tĩnh nên chúng bị đếm rồi BỎ QUA. Viết vai thành chuỗi literal.',
  )
}

if (fBad === 0) {
  pass(
    // 🔵 `Object.keys(...).length`, KHÔNG `.length` — 2026-08-15 (code review). `SELECTION_PANEL_FILES`
    // đổi từ MẢNG sang OBJECT ở Story 2.5b để chở số lời gọi mong đợi theo từng tệp; vòng lặp phán
    // quyết được sửa theo (`Object.entries`) nhưng dòng bằng chứng này thì không, nên cổng in ra
    // nguyên văn `trên undefined panel` ở mọi lượt xanh. Phán quyết không sai — nhưng
    // `project-context.md` §Luật đo: *"số đo không truy nguyên được thì không phải số đo"*, và một
    // cổng tự in `undefined` vào chính dòng bằng chứng của nó là một cổng thôi tự mô tả đúng.
    `${surfaceCalls.length} bề mặt đăng ký hợp đồng vùng chọn trên ${Object.keys(SELECTION_PANEL_FILES).length} panel ` +
      `(sàn ${SELECTION_SURFACE_FLOOR}) — ` +
      `${surfaceCalls.filter((c) => c.role === 'source').length} nguồn · ` +
      `${surfaceCalls.filter((c) => c.role === 'display').length} hiển thị`,
  )
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm I — vạch lề segment: ĐÚNG NĂM giá trị, không một giá trị thứ sáu (Story 2.2, AC12)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// AC2 của Story 2.2 nói vạch lề là **cách DUY NHẤT** trạng thái segment được hiển thị;
// `DESIGN.md:380` lặp lại; và `EXPERIENCE.md:99` giải thích cái giá đã trả cho mệnh đề đó:
// *"vạch lề **đã dùng hết năm giá trị**"* — chính vì thế UX-DR22 buộc phát hiện Proofreader
// phải đi đường **gạch chân lượn sóng** thay vì xin một màu vạch nữa.
//
// ⇒ Con số **năm** là một tài nguyên đã tiêu hết, không một chi tiết cài đặt. Một giá trị
// thứ sáu thêm vào *"tạm để phân biệt"* sẽ không làm gì đỏ ở bất kỳ cổng nào khác, và nó
// tiêu mất chỗ mà một epic sau đang trông vào. Cổng này là chỗ duy nhất nói không.
//
// Ba mệnh đề, và mệnh đề ③ là lý do cổng này không chỉ là một phép đếm:
//   ① `SEGMENT_RULE_VALUES` có ĐÚNG năm phần tử, và đúng năm cái tên đó;
//   ② phép phân giải THẬT trả đúng giá trị cho từng ca — cổng `import()` và **chạy** nó;
//   ③ đối chiếu HAI CHIỀU với CSS của `GridPanel.vue`: mỗi giá trị (trừ *không vạch*) có
//      đúng một khối `.rule-<giá trị>` khai `background-color: var(--color-<giá trị>)`, và
//      không khối `.rule-*` nào tồn tại ngoài danh sách. Không có chiều thứ hai thì một
//      giá trị đổi tên trong TS mà quên CSS cho ra một vạch **vô hình** — trạng thái mất
//      im lặng, đúng lớp lỗi tệ nhất trên một panel mà trạng thái là toàn bộ nội dung.

const EDITOR_SEGMENTS_TS = join(SRC_ROOT, 'panels', 'editorSegments.ts')
// 🔵 2026-08-14 (Story 2.5b): `EditorPanel.vue` → `GridPanel.vue`. Bốn màu vạch chuyển sang
// `<style scoped>` của lưới cùng lượt gộp hai panel; tên hằng giữ nguyên để diff đọc được.
const EDITOR_PANEL_VUE = join(SRC_ROOT, 'panels', 'GridPanel.vue')

/** ⚠️ Bản chép ĐỘC LẬP của `DESIGN.md:380` — không `import` từ tệp đang bị kiểm. */
// 🔵 2026-08-14 (Story 2.5b) — NĂM → SÁU. `draft` lấp một hàng vốn đã THIẾU trong bảng
// (*"đã dịch tay, chưa xác nhận, con trỏ ở chỗ khác"*), không xin một kênh thị giác cho một
// trạng thái mới ⇒ UX-DR22 không bị đụng. Lý lẽ đầy đủ + hai lượt ký theo thứ tự ở
// doc-comment của `src/panels/editorSegments.ts::resolveSegmentRule`.
const EXPECTED_RULE_VALUES = ['confirmed', 'primary', 'tm-rule', 'draft', 'none', 'ornament']

if (!existsSync(EDITOR_SEGMENTS_TS)) {
  abort(`\`${posix(EDITOR_SEGMENTS_TS)}\``, new Error('Tệp không tồn tại — Kiểm I KHÔNG chạy được.'))
}
const segmentsMod = await loadTs(EDITOR_SEGMENTS_TS, 'Kiểm I')

let iBad = 0
const ruleValues = segmentsMod.SEGMENT_RULE_VALUES
if (!Array.isArray(ruleValues)) {
  abort(
    `\`${posix(EDITOR_SEGMENTS_TS)}\``,
    new Error('không export mảng `SEGMENT_RULE_VALUES` — Kiểm I KHÔNG chạy được.'),
  )
}

// ① Đếm và đối chiếu tên.
if ([...ruleValues].sort().join('|') !== [...EXPECTED_RULE_VALUES].sort().join('|')) {
  fail(`bộ giá trị vạch lề lệch bản đặc tả — mã khai [${ruleValues.join(' · ')}]`)
  detail(`DESIGN.md:380 · EXPERIENCE.md:105-113 khai [${EXPECTED_RULE_VALUES.join(' · ')}]`)
  detail('Năm giá trị là tài nguyên ĐÃ TIÊU HẾT. Kênh thị giác kế tiếp là gạch chân lượn sóng')
  detail('(UX-DR22), không phải một màu vạch nữa. Sửa `DESIGN.md` là một lượt riêng của Ice.')
  iBad += 1
}

// ② Hành vi THẬT của phép phân giải — năm ca, mỗi ca một giá trị.
if (typeof segmentsMod.resolveSegmentRule !== 'function') {
  fail(`\`${posix(EDITOR_SEGMENTS_TS)}\` không export \`resolveSegmentRule\``)
  iBad += 1
} else {
  const base = {
    retiredAt: null,
    hasCaret: false,
    isConfirmed: false,
    isTmFilled: false,
    targetText: '',
  }
  const cases = [
    ['ornament', { ...base, retiredAt: '2026-08-12T00:00:00.000Z', hasCaret: true, isConfirmed: true }],
    ['primary', { ...base, hasCaret: true, isConfirmed: true, isTmFilled: true }],
    ['confirmed', { ...base, isConfirmed: true, isTmFilled: true }],
    ['tm-rule', { ...base, isTmFilled: true, targetText: 'ban dich' }],
    ['none', { ...base }],
  ]
  for (const [want, input] of cases) {
    const got = segmentsMod.resolveSegmentRule(input)
    if (got !== want) {
      fail(`\`resolveSegmentRule\` trả \`${got}\`, phải là \`${want}\` — thứ tự ưu tiên năm nhánh đã đổi`)
      detail('Thứ tự là một quyết định, không phải thứ tự gõ ra — xem doc-comment của hàm đó.')
      iBad += 1
    }
  }
}

// ③ Đối chiếu HAI CHIỀU với CSS của `GridPanel.vue`.
const editorVue = parsed.find((p) => p.file === EDITOR_PANEL_VUE)
if (editorVue === undefined) {
  fail(`\`${posix(EDITOR_PANEL_VUE)}\` không nằm trong quần thể quét — Kiểm I mất chiều thứ hai`)
  iBad += 1
} else {
  /**
   * 🔴 Đọc bản **ĐÃ CHE**, không nguyên văn — cùng lý do Kiểm J ngay dưới, và lý do đó là một
   * phép đo chứ không một phòng hờ: bản đầu của Kiểm J quét `p.text` và **đỏ ngay trên chính
   * tệp nó canh**, vì doc-comment gọi tên đủ thứ bị cấm để giải thích vì sao chúng bị cấm.
   *
   * Kiểm I ăn đúng rủi ro đó ở **chiều ngược**: một chú thích sau này viết ví dụ
   * `.rule-<gì đó> { … }` sẽ nhập vào `declaredClasses` và làm cổng đỏ oan *(bắt ở code review
   * 2026-08-12; hôm nay chưa chú thích nào chứa chuỗi đó nên cổng còn xanh)*.
   *
   * ⚠️ `maskStyle` chỉ xoá `/* *​/`, nên **mọi khai báo CSS sống nguyên** — bốn khối
   * `.rule-*` và `background-color: var(--color-*)` vẫn đọc được từng chữ.
   */
  const editorCss = editorVue.masked
  const declaredClasses = new Set()
  const classRe = /\.rule-([a-z0-9-]+)\b/g
  let m
  while ((m = classRe.exec(editorCss))) declaredClasses.add(m[1])

  for (const value of ruleValues) {
    if (value === 'none') {
      // *Không vạch* CỐ Ý không có khối CSS — nó không vẽ gì. Một `.rule-none` tồn tại
      // nghĩa là ai đó đã vẽ một vạch cho trạng thái "chưa dịch".
      if (declaredClasses.has('none')) {
        fail(`\`${posix(EDITOR_PANEL_VUE)}\` khai \`.rule-none\` — *không vạch* phải KHÔNG vẽ gì`)
        iBad += 1
      }
      continue
    }
    if (!declaredClasses.has(value)) {
      fail(`\`${posix(EDITOR_PANEL_VUE)}\` thiếu khối \`.rule-${value}\` — vạch \`${value}\` sẽ VÔ HÌNH`)
      iBad += 1
      continue
    }
    // ⚠️ Khuôn thoát chép từ `globToRe` (`:101`) — bản trước ở đây HỎNG và phép thoát là một
    //    lượt no-op: lớp ký tự `[.*+?^${}()|[\\]` **đóng sớm** ở `]` sau `\\`, nên regex thật
    //    đòi thêm hai dấu `\` và một `]` ở sau; đo được, `'a.b*c'` đi qua nguyên vẹn. Chuỗi
    //    thay thế cũng chèn HAI dấu `\` chứ không một. Vô hại hôm nay *(năm giá trị vạch không
    //    chứa ký tự đặc biệt nào, và AC12 khoá con số năm lại)*, nhưng một hàm thoát không thoát
    //    gì là thứ story sau tin nhầm. Bắt ở code review 2026-08-12.
    const escaped = value.replace(/[.+^${}()|[\]\\]/g, '\\$&')
    const wantDecl = new RegExp(
      `\\.rule-${escaped}\\s*\\{[^}]*background-color:\\s*var\\(\\s*--color-${escaped}\\s*\\)`,
    )
    if (!wantDecl.test(editorCss)) {
      fail(`\`.rule-${value}\` không khai \`background-color: var(--color-${value})\``)
      detail('Màu vạch phải đến từ token và phải nằm trong CSS — `check-tokens.mjs` không đọc TypeScript.')
      iBad += 1
    }
  }
  for (const cls of declaredClasses) {
    if (!ruleValues.includes(cls)) {
      fail(`\`${posix(EDITOR_PANEL_VUE)}\` khai \`.rule-${cls}\` — không phải một trong sáu giá trị`)
      iBad += 1
    }
  }
}

if (iBad === 0) {
  pass(
    `vạch lề segment khai ĐÚNG ${ruleValues.length} giá trị [${ruleValues.join(' · ')}] — ` +
      'phép phân giải chạy đúng cả năm ca, CSS khớp hai chiều',
  )
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 KIỂM J ĐÃ ĐƯỢC GỠ — Story 2.3, và nó được gỡ ĐÚNG LÚC, không sớm hơn.
// ═════════════════════════════════════════════════════════════════════════════════
//
// Kiểm J tồn tại từ Story 2.2 tới Story 2.3 và nó khẳng định `EditorPanel.vue` *(tệp đó gỡ
// ở Story 2.5b)* KHÔNG mang
// năm thứ: `contenteditable` · `<textarea>` · `<input>` · `v-model` ·
// `@input`/`@beforeinput`/`@paste`/`@cut`. Lý do nó tồn tại: Ice chốt Quyết định #1 của
// Story 2.2 đường **(b)** — bề mặt CHỈ-ĐỌC — và một bề mặt gõ được mà **chưa có đường lưu**
// là một cửa sổ người dùng gõ rồi mất trắng khi đóng app, không một dấu hiệu nào (NFR18).
//
// `deferred-work.md` ghi hạn của nó bằng chữ: *"cổng này hết hạn ở Story 2.3, và nó phải
// được gỡ ĐÚNG LÚC — không sớm hơn. Gỡ sớm là mở lại đúng cửa sổ mất dữ liệu im lặng mà cổng
// tồn tại để đóng."*
//
// ⚠️ **Thứ tự làm việc đã giữ, và đây là bằng chứng chứ không một lời khai:** đường flush của
// AD-35 nghiệm thu XANH ở `src-tauri/tests/segment_contract.rs` — TÁM ca mới, gồm lượt
// round-trip *gõ → flush → nạp lại* và ca *lô mang một id lạ bị từ chối TRỌN* — **trước** khi
// dòng `contenteditable` đầu tiên chạm `EditorPanel.vue`.
//
// 🔴 Gỡ **CẢ KHỐI**: bảng `TYPING_BANS`, sàn nội dung `data-segment-id`, và tiêu đề in ra. Một
// cổng xanh RỖNG — năm phép cấm không còn gì để cấm — là một dòng OK dạy người đọc rằng có
// một lưới ở đây, trong khi không còn lưới nào. Sàn nội dung `data-segment-id` KHÔNG mồ côi:
// Kiểm I ngay trên vẫn đọc `editorVue.masked` và vẫn đối chiếu năm giá trị vạch hai chiều, nên
// một `EditorPanel.vue` bị đổi tên hay bị làm rỗng vẫn làm cổng này đỏ.
//
// ⚠️ `@keydown` **chưa từng** nằm trong danh sách cấm (làm rõ ở code review 2026-08-12), nên
// story 2.3 không phải "mở khoá" nó.

// ═════════════════════════════════════════════════════════════════════════════════
console.log('')
if (skippedLinks.length) {
  console.log(`\x1b[33mĐã BỎ QUA ${skippedLinks.length} symlink:\x1b[0m ${skippedLinks.join(' · ')}`)
  console.log('')
}
if (failures !== 0) {
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  console.log('')
  console.log('AD-34 §1: mọi thao tác đăng ký ở `CommandRegistry` TRƯỚC khi bind vào chuột hoặc')
  console.log('phím; handler chuột chỉ được `dispatch` một command đã đăng ký.')
  console.log('AD-34 §2: mỗi chế độ và mỗi panel khai điểm vào focus; focus không rơi về `body`.')
  process.exit(1)
}
console.log('\x1b[32mTất cả phép kiểm CommandRegistry đạt.\x1b[0m')
console.log('')
console.log(
  `Tầm quét: ${vueFiles.length} tệp \`.vue\` + ${tsFiles.length} tệp \`.ts\` · ` +
    `${clickAttrs.length} \`@click\` · ${dispatched.length} lời gọi \`dispatch()\` · ` +
    `${registered.length} command · ${owners.length} điểm vào focus.`,
)
console.log(`Đã miễn trừ ${exemptedFiles.length} tệp.`)
console.log('')
console.log('Ghi chú cho người rà soát — ba giới hạn, ghi thẳng thay vì để người sau tự phát hiện:')
console.log('  1. Kiểm A chỉ canh `@click`. `@keydown`/`@input`/`@submit` KHÔNG thuộc luật này;')
console.log('     ngày một `@keydown` mang thao tác thật xuất hiện, luật phải được xem lại.')
console.log('  2. Vế DOM của AC4 (*"focus không rơi về `body`"*) KHÔNG kiểm được ở đây — nó là')
console.log('     hành vi lúc chạy trong một webview thật. Chốt tự kêu ở `src/commands/focus.ts`')
console.log('     cộng nghiệm thu tay; giới hạn ghi ở `deferred-work.md`. Không đánh dấu đạt.')
console.log('  3. Cổng KHÔNG canh focus ring. Một `*:focus { outline: none }` phá NFR17 mà vẫn')
console.log('     qua được cả cổng này lẫn `check-tokens.mjs` (§Trap 4 của Story 1.6).')
process.exit(0)
