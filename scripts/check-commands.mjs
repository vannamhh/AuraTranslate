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
 * 1. **Kiểm A chỉ canh `@click`.** `@input`/`@change` KHÔNG thuộc luật này — chúng là dòng DỮ
 *    LIỆU theo nghĩa AD-34 §1, không phải một thao tác người dùng chủ động.
 *    `@keydown`/`@keyup`/`@mouseup`/`@mousedown`/`@submit` are checked by Kiểm K (end of
 *    file) against a frozen two-way table, not by widening Kiểm A's regex.
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
import {
  blank,
  maskScript,
  maskTemplate,
  attributesIn,
  scanVueAttrs,
  functionBodyRange,
  balancedBraceBody,
  splitTopLevel,
} from './lib/commands-scan.mjs'

/** @typedef {import('./lib/commands-scan.mjs').ParsedFile} ParsedFile */
/** @typedef {import('./lib/commands-scan.mjs').TemplateRegion} TemplateRegion */
/** @typedef {import('./lib/commands-scan.mjs').TemplateAttr} TemplateAttr */

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const SRC_ROOT = join(REPO_ROOT, 'src')
const REGISTRY_TS = join(SRC_ROOT, 'commands', 'registry.ts')
const KEYS_TS = join(SRC_ROOT, 'commands', 'keys.ts')
const COMMANDS_INDEX_TS = join(SRC_ROOT, 'commands', 'index.ts')
const VI_JSON = join(SRC_ROOT, 'i18n', 'vi.json')

let failures = 0
/** @param {string} m */
const pass = (m) => console.log(`  \x1b[32mOK\x1b[0m   ${m}`)
/** @param {string} m */
const fail = (m) => {
  console.log(`  \x1b[31mFAIL\x1b[0m ${m}`)
  failures += 1
}
/** @param {string} m */
const detail = (m) => console.log(`       ${m}`)

/**
 * @param {unknown} err
 * @returns {string}
 */
const errorMessage = (err) => (err instanceof Error && err.message ? err.message : String(err))

/**
 * Lỗi hạ tầng ≠ phép kiểm đỏ. Dừng ngay, đừng báo cáo một kết quả không có thật.
 * (`check-deps.mjs:60-66`)
 * @param {string} what
 * @param {unknown} err
 * @returns {never}
 */
function abort(what, err) {
  console.error(`\n\x1b[31mKhông đọc được ${what} — phép kiểm KHÔNG chạy được.\x1b[0m`)
  console.error('Đây là lỗi hạ tầng, không phải "đạt". Đọc lỗi dưới đây rồi chạy lại.\n')
  console.error(errorMessage(err).trim())
  process.exit(1)
}

// ═════════════════════════════════════════════════════════════════════════════════
// MIỄN TRỪ — mỗi mục một câu lý do, và cổng IN RA số tệp đã miễn trừ ở mỗi lượt chạy
//
// Miễn trừ KHÔNG được cài bằng cách thu hẹp glob quét (cùng luật với `check-i18n.mjs`).
// Hôm nay danh sách RỖNG, và con số 0 in ra có chủ ý: khối này tồn tại để lần đầu ai đó
// cần một ngoại lệ thì phải viết lý do ra đây, chứ không phải sửa một dấu sao.
// ═════════════════════════════════════════════════════════════════════════════════
/** @type {[string, string][]} */
const EXEMPT = []

/**
 * @param {string} pattern
 * @returns {RegExp}
 */
function globToRe(pattern) {
  const escaped = pattern.replace(/[.+^${}()|[\]\\]/g, '\\$&')
  return new RegExp(`^${escaped.replace(/\*\*/g, '.*').replace(/(?<!\.)\*/g, '[^/]*')}$`)
}
const EXEMPT_RES = EXEMPT.map(
  ([pattern, why]) => /** @type {[RegExp, string, string]} */ ([globToRe(pattern), pattern, why]),
)
/** @param {string} p */
const posix = (p) => relative(REPO_ROOT, p).split(sep).join('/')
/** @param {string} file */
const exemptionFor = (file) => EXEMPT_RES.find(([re]) => re.test(posix(file)))

// ═════════════════════════════════════════════════════════════════════════════════
// Đọc cây nguồn
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ `lstatSync`, KHÔNG `statSync` — cùng bài học với hai cổng trước: `statSync` giải
// symlink nên một liên kết trỏ về thư mục cha làm đệ quy không dừng, và một liên kết gãy
// ném `ENOENT` bị `abort()` báo thành "cây nguồn không đọc được". Symlink bị BỎ QUA và
// ghi tên ra, để việc bỏ qua không im lặng.
/** @type {string[]} */
const skippedLinks = []
const SKIP_DIRS = new Set(['target', 'node_modules', 'dist', '.git'])

/**
 * @param {string} dir
 * @param {string[]} exts
 * @param {string[]} out
 * @param {Set<string>} seen
 * @returns {string[]}
 */
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
   * 🔴 BỐN PHẦN MỞ RỘNG, KHÔNG PHẢI MỘT — đóng `deferred-work.md §*Deferred from: 1-6-commandregistry-ba-che-do-va-tieu-diem-ban-phim (2026-08-04)*` (Story 1.14 · AC11.3).
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

/** @type {[string, string][]} */
const exemptedFiles = []
/** @param {string[]} files */
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
 * 🔴 NÂNG SÀN 2026-08-06 — Story 1.14 · AC11.1, đóng `deferred-work.md §*Deferred from: code review of 1-5-tai-nguyen-chuoi-giao-dien-va-hinh-dang-loi-qua-ipc (2026-08-04)*` và `:146`.
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
const VUE_FLOOR = 16 // 🔵 NÂNG 2026-08-22 (Story 3.6): số THẬT 19 tệp `.vue`
// (+GlossaryConfirmStrip.vue) — 16/19 = 84,2%
// (trước đó) 🔵 NÂNG 2026-08-22 (Story 3.5): số THẬT 18 tệp `.vue`
// (+GlossarySettingsOverlay.vue) — 15/18 = 83,3%
// ⚠️ 15 → 17 KHÔNG chỉ từ story này: sàn cũ (13) đặt từ số thật 15 hồi Story 1.21. Story
// 3.3 tự nó thêm ĐÚNG MỘT tệp `.vue` (`GlossaryQuickAdd.vue`); tệp thứ hai là nợ đo lại
// tồn đọng từ một story giữa 1.21 và 3.3 không ai nâng sàn lại — cùng bài học đã ghi ở
// `RS_FLOOR`/`VUE_FLOOR` của `check-i18n.mjs`.
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
// 🔴 NÂNG 2026-08-18 (Story 2.12 · Task 7.5) — số thật lên **39**, nên sàn 30 tụt xuống
// **76,9%**, dưới dải ~81–85% mà chính doc-comment trên đặt ra.
// ⚠️ **Sàn này KHÔNG nằm trong việc story 2.12 được giao** — Task 7.5 chỉ nêu đích danh
// `check-layout.mjs`. Nó lòi ra vì lượt đo lại của Task 7.5 đếm CẢ HAI sàn đọc `src/**` thay
// vì đúng một, và bỏ qua nó sau khi đã thấy thì đúng bằng việc biết một cổng đã tắt mà im.
const TS_FLOOR = 39 // 🔵 NÂNG 2026-08-22 (Story 3.6): số THẬT 47 tệp `.ts`
// (+glossaryConfirmStripState.ts, +panels/inlineStripPriority.ts) — 39/47 = 83,0%.
// (trước đó) 🔵 NÂNG 2026-08-22 (Story 3.5): số THẬT 45 tệp `.ts`
// (+glossarySettingsState.ts) — 37/45 = 82,2%.
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
// 🔵 **38 → 39, Story 2.9 (2026-08-17).** Cổng in **47** command sau khi thêm
// `editor.clear_source_cuts` *(Story 2.8 ghi 46 — đúng +1, không hơn)*. Đo lại bằng cách chạy
// cổng RỒI sửa dòng này, đúng thứ tự mà chú thích ngay trên đòi. 39/47 = **83 %**, giữa dải
// 80–85 %; để nguyên 38 thì sàn tụt xuống 81 % và mất dần ý nghĩa qua từng story.
// 🔵 **39 → 41, Story 2.10 (2026-08-18).** Cổng in **49** command sau khi thêm
// `editor.next_segment` và `editor.prev_segment` *(Story 2.9 ghi 47 — đúng +2, không hơn)*.
// Chạy cổng, đọc số, rồi mới sửa dòng này. 41/49 = **83,7 %**, giữa dải 80–85 %; để nguyên 39
// thì tụt xuống 79,6 %, tức **ra khỏi dải** — đúng thứ ba story liên tiếp trước đây phải sửa.
//
// ⚠️ **Một lượt đọc sai của chính lượt này, ghi lại vì nó là bài học chứ không vì thủ tục:**
// Task 0.1 đo được *"sàn 39, số thật 47"* và đọc nó thành *"sàn thấp hơn thực tế 8 đơn vị ⇒ nó
// không canh được gì"*, rồi đề xuất nâng thẳng lên **49**. Sai, và sai vì **chưa đọc doc-comment
// ngay trên đây**: sàn là **cận dưới có chủ ý**, đặt ở ~80–85 % số thật, chính vì *"một lượt
// quét hỏng (glob sai, `SKIP_DIRS` nuốt nhầm) tụt sâu hơn khoảng đó rất nhiều"*. 39/47 = 83 % là
// **đúng thiết kế**, không một khuyết tật. Một sàn đặt **bằng** số thật thì mọi lượt thêm
// command đều làm cổng đỏ oan — nó đổi một cận dưới thành một phép so bằng.
// ⇒ Cùng lớp với luật đã ghi ở `project-context.md`: *"cây nguồn thắng"*, và ở đây cây nguồn là
//   doc-comment của chính cơ chế mình đang sửa.
// 🔵 **41 → 43, Story 2.11 (2026-08-18).** Cổng in **51** command sau khi thêm
// `editor.next_chapter` và `editor.prev_chapter` *(Story 2.10 ghi 49 — đúng +2, không hơn)*.
// Chạy cổng, đọc số, rồi mới sửa dòng này — đúng thứ tự mà lượt code review Story 2.6 đã trả
// giá để dựng ra. 43/51 = **84,3 %**, giữa dải 80–85 %; để nguyên 41 thì tụt xuống 80,4 %,
// tức chạm mép dưới và mất ý nghĩa ở story kế tiếp.
// ⚠️ **Ba sàn kia KHÔNG đổi, và đó là một kết luận đã đo chứ không một lượt bỏ qua:** story
// này thêm **0** tệp `.vue`, **0** tệp `.ts` *(mọi thay đổi nằm trong tệp đã có)*, **0**
// `@click` và **0** lời gọi `dispatch()` — hai lệnh mới tới được bằng **phím**, không bằng một
// bề mặt bấm. Cổng in lại đúng 16 `.vue` · 39 `.ts` · 25 `@click` · 34 `dispatch()`.
// 🔵 NÂNG 2026-08-20 (Story 3.3): cổng in 54 command sau khi thêm
// `glossary.add_term`/`glossary.save_term`/`glossary.close_quick_add` (51 → 54, đúng +3,
// không hơn — chạy cổng, đọc số, rồi mới sửa dòng này, đúng thứ tự luật đã đúc từ Story
// 2.6). 44/54 = 81,5%, giữa dải 80–85%.
// 🔵 NÂNG 2026-08-22 (Story 3.5): cổng in 57 command sau khi thêm `glossary.settings.open`/
// `glossary.settings.close`/`glossary.settings.save` (54 → 57, đúng +3, không hơn). Sàn
// 44 → 47 (47/57 = 82,5%, giữa dải 80–85%).
// 🔵 NÂNG 2026-08-22 (Story 3.6): cổng in 60 command sau khi thêm `glossary.confirm.focus`/
// `glossary.confirm.save`/`glossary.confirm.defer` (57 → 60, đúng +3). Sàn 47 → 50
// (50/60 = 83,3%, giữa dải 80–85%).
// 🔵 NÂNG 2026-08-28 (Story 5.6): cổng in **95** command sau khi thêm `library.work_next`/
// `library.work_prev` (chỉ +2 — bàn phím di chuyển con trỏ lưới, chép khuôn
// `library.orphan_next`/`orphan_prev`). Sàn +2 theo đúng con số vừa thêm (50 → 52).
// ⚠️ 52/95 = 54,7% — DƯỚI hẳn dải 80–85% mà doctrine này đặt ra: khoảng cách đó đã tồn tại
// TỪ TRƯỚC story này (95 − 60 = 35 command thêm ở các story giữa 3.6 và 5.6 mà không ai
// nâng sàn theo), không phải một khoản nợ do story này để lại. Đóng dứt điểm khoảng cách
// đó cần đọc lại TOÀN BỘ lịch sử các story ở giữa — ngoài phạm vi của story này, ghi ra để
// người sau không tưởng nhầm 52 là con số "đã canh sát".
const COMMAND_FLOOR = 52

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
// 🔵 NÂNG 2026-08-20 (Story 3.3): số THẬT 26 thuộc tính `@click`
// — 21/26 = 80,8%. ⚠️ **Chỉ MỘT** `@click` mới, không hai: `GlossaryQuickAdd.vue` có hai
// nút, nhưng nút Lưu là `type="submit"` đi qua `@submit.prevent` (Kiểm A KHÔNG canh
// `@submit`) — chỉ nút Huỷ (`type="button"`) mang `@click="dispatch('glossary.close_
// quick_add')"`. Đo lại bằng cách đọc chính tệp, không suy từ số nút bấm trên màn hình.
// 🔵 NÂNG 2026-08-22 (Story 3.5): số THẬT 29 thuộc tính `@click` — ba `@click` mới
// (`App.vue` nút mở lớp phủ, `GlossarySettingsOverlay.vue` nút đóng + nút Huỷ; nút Lưu là
// `type="submit"` đi qua `@submit.prevent`, cùng lý do `GlossaryQuickAdd.vue`). 24/29 = 82,8%.
// 🔵 NÂNG 2026-08-22 (Story 3.6): số THẬT 30 thuộc tính `@click` — MỘT `@click` mới
// (`GlossaryConfirmStrip.vue` nút "Để sau"; nút Lưu là `type="submit"` đi qua
// `@submit.prevent`, cùng lý do hai dải kia). 25/30 = 83,3%.
// 🔵 NÂNG 2026-08-28 (Story 5.6): số THẬT **65** — hai `@click` mới (`LibraryMode.vue`, nút
// `‹`/`›` con trỏ lưới, `dispatch('library.work_prev'/'work_next')`). Sàn +2 theo đúng số
// vừa thêm (25 → 27) — cùng ghi chú về khoảng cách 80–85% đã ghi ở `COMMAND_FLOOR`, khoảng
// cách đó không phải nợ của story này.
const CLICK_FLOOR = 27
// 🔵 NÂNG 2026-08-20 (Story 3.3): số THẬT 37 lời gọi `dispatch()` — 30/37 = 81,1%
// 🔵 NÂNG 2026-08-22 (Story 3.5): số THẬT 42 lời gọi `dispatch()` (+5: nút mở ở `App.vue`,
// `@keydown.esc`/nút đóng/nút Huỷ và `@submit` của `GlossarySettingsOverlay.vue`, mỗi
// lời gọi `dispatch('glossary.settings.*')` một chỗ). 34/42 = 81,0%.
// 🔵 NÂNG 2026-08-22 (Story 3.6): số THẬT 45 lời gọi `dispatch()` (+3: `@keydown.esc`/nút
// "Để sau" và `@submit` của `GlossaryConfirmStrip.vue`, mỗi lời gọi
// `dispatch('glossary.confirm.*')` một chỗ). 38/45 = 84,4%.
// 🔵 NÂNG 2026-08-28 (Story 5.6): số THẬT **93** lời gọi `dispatch()` — hai lời gọi mới
// (`dispatch('library.work_prev')`/`dispatch('library.work_next')`). Sàn +2 theo đúng số
// vừa thêm (38 → 40) — cùng ghi chú khoảng cách 80–85% đã ghi ở `COMMAND_FLOOR`.
const DISPATCH_FLOOR = 40

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
// `maskScript`/`maskTemplate`/`attributesIn` moved to `./lib/commands-scan.mjs`.
// `maskStyle`/`vueRegions`/`maskFile` stay here: `vueRegions` owns the impure
// module-level `looseBlockTags` counter this gate reports on.

/**
 * Vùng CSS — chỉ `/* *\/`. ⚠️ `//` KHÔNG phải comment: `url(//host/x.png)` là một URL.
 * @param {string} text
 * @param {number} from
 * @param {number} to
 * @param {string[]} chars
 */
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
 * @param {string} text
 * @returns {(TemplateRegion & { kind: string })[]}
 */
function vueRegions(text) {
  /** @type {(TemplateRegion & { kind: string })[]} */
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

/**
 * `code` is a second, additive masking pass over the same text (also blanks string/
 * template-literal content) — used only to locate a real call, never to read an id.
 * @param {string} text
 * @param {boolean} isVue
 * @returns {{ masked: string, code: string, templates: TemplateRegion[] }}
 */
function maskFile(text, isVue) {
  const chars = text.split('')
  const codeChars = text.split('')
  if (!isVue) {
    maskScript(text, 0, text.length, chars)
    maskScript(text, 0, text.length, codeChars, { blankLiterals: true })
    return { masked: chars.join(''), code: codeChars.join(''), templates: [] }
  }
  const regions = vueRegions(text)
  /** @type {TemplateRegion[]} */
  const templates = []
  let cursor = 0
  for (const r of regions) {
    if (r.start > cursor) {
      maskTemplate(text, cursor, r.start, chars)
      maskTemplate(text, cursor, r.start, codeChars)
      templates.push({ start: cursor, end: r.start })
    }
    if (r.kind === 'script') {
      maskScript(text, r.start, r.end, chars)
      maskScript(text, r.start, r.end, codeChars, { blankLiterals: true })
    } else {
      maskStyle(text, r.start, r.end, chars)
      maskStyle(text, r.start, r.end, codeChars)
    }
    cursor = r.end
  }
  if (cursor < text.length) {
    maskTemplate(text, cursor, text.length, chars)
    maskTemplate(text, cursor, text.length, codeChars)
    templates.push({ start: cursor, end: text.length })
  }
  return { masked: chars.join(''), code: codeChars.join(''), templates }
}

/**
 * Builds a `ParsedFile` — shared by the real scan below and Kiểm K's self-check, so the
 * self-check parses its fake fixture with the real function, not a copy.
 * @param {string} file
 * @param {string} text
 * @returns {ParsedFile}
 */
function buildParsedEntry(file, text) {
  const isVue = file.toLowerCase().endsWith('.vue')
  const { masked, code, templates } = maskFile(text, isVue)
  return { file, text, masked, code, templates, isVue }
}

/** @type {ParsedFile[]} */
const parsed = []
for (const file of [...vueFiles, ...tsFiles]) {
  /** @type {string} */
  let text
  try {
    text = readFileSync(file, 'utf8')
  } catch (err) {
    abort(`tệp \`${posix(file)}\``, err)
  }
  parsed.push(buildParsedEntry(file, text))
}

/**
 * @param {string} text
 * @param {number} index
 */
const positionOf = (text, index) => {
  const before = text.slice(0, index)
  const line = before.split('\n').length
  const col = index - (before.lastIndexOf('\n') + 1) + 1
  return { line, col }
}
/**
 * @param {ParsedFile} p
 * @param {number} index
 */
const at = (p, index) => {
  const { line, col } = positionOf(p.text, index)
  return `${posix(p.file)}:${line}:${col}`
}
/**
 * Trích 60 ký tự quanh chỗ vi phạm, một dòng, để chẩn đoán đọc được ngay ở log CI.
 * @param {string} text
 * @param {number} index
 */
const excerpt = (text, index) =>
  text
    .slice(Math.max(0, index - 30), Math.min(text.length, index + 30))
    .replace(/\s+/g, ' ')
    .trim()

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
/** @type {[RegExp, string][]} */
const OPAQUE_CLICK_RES = [
  [/^:?on[Cc]lick$/, '`:onClick` / `onClick` là một listener click của Vue 3'],
  [/^v-on$/, '`v-on="{ click: … }"` là dạng object của một listener click'],
  [/^(@|v-on:)\[/, 'tên sự kiện ĐỘNG — không đọc tĩnh được, nên không chứng minh được'],
]

/** @type {{ p: ParsedFile, a: TemplateAttr }[]} */
const clickAttrs = []
let aBad = 0
for (const { p, a } of scanVueAttrs(parsed)) {
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
detail('giới hạn đã khai: chỉ `@click`; `@input`/`@change` KHÔNG thuộc luật này (xem đầu tệp).')
detail('`@keydown`/`@keyup`/`@mouseup`/`@mousedown`/`@submit` có Kiểm K riêng, cuối tệp.')
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

/**
 * @param {string} path
 * @param {string} which
 * @returns {Promise<any>}
 */
const loadTs = async (path, which) => {
  try {
    return await import(pathToFileURL(path).href)
  } catch (err) {
    abort(
      `\`${posix(path)}\` — ${which} KHÔNG chạy được`,
      new Error(
        `${errorMessage(err)}\n\n` +
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
/**
 * @param {string} what
 * @param {() => void} fn
 * @param {string} [needle]
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
/**
 * @param {string} what
 * @param {() => void} fn
 */
const expectNoThrow = (what, fn) => {
  try {
    fn()
    pass(what)
  } catch (err) {
    fail(`${what} — NÉM ở đường hợp lệ: ${errorMessage(err)}`)
    cBad += 1
  }
}
/**
 * @param {string} what
 * @param {unknown} got
 * @param {unknown} want
 */
const expectEq = (what, got, want) => {
  if (got === want) return
  fail(`${what} — nhận \`${String(got)}\`, phải là \`${String(want)}\``)
  cBad += 1
}

{
  const noop = () => {}
  /**
   * @param {string} id
   * @param {Record<string, unknown>} [extra]
   */
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
    r.list().map((/** @type {any} */ s) => s.id).join(' → '),
    'mode.library → mode.workspace → focus.next_panel → demo.keys_rong',
  )

  // AC6 — và ⚠️ ca `keys: []` phải nằm trong tập, không chỉ ca `keys` vắng mặt.
  expectEq(
    '`unbound()` trả ĐÚNG tập command thiếu phím (AC6)',
    r.unbound().map((/** @type {any} */ s) => s.id).sort().join(' · '),
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
  /**
   * Phần tử giả: `enter()` chỉ cần một thứ có `focus()`.
   * @param {string} name
   */
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
  /** @type {string[]} */
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
  /** @type {string[]} */
  const fired = []
  /** Registry giả — Kiểm D nghiệm thu TẦNG BÀN PHÍM, không nghiệm thu lại registry. */
  const fakeRegistry = {
    list: () => [
      { id: 'mode.library', labelKey: 'command.mode.library', run: () => {}, keys: ['Mod+1'] },
      { id: 'ai.send', labelKey: 'command.ai.send', run: () => {}, keys: ['Mod+Shift+Enter'] },
      { id: 'read.bilingual', labelKey: 'command.read.bilingual', run: () => {}, keys: ['B'] },
    ],
    /** @param {string} id */
    dispatch: (id) => fired.push(id),
    register: () => {},
    has: () => true,
    unbound: () => [],
  }

  /** @param {Partial<Record<string, unknown>>} over */
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

  /**
   * @param {string} what
   * @param {unknown} got
   * @param {unknown} want
   */
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
  // 🔴 STORY 1.21 — CỜ `repeatable` (`deferred-work.md §*Deferred from: 1-14-khung-bon-panel (2026-08-06)*`, Ice ký nhận 2026-08-11)
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // Ba ca ngay trên khẳng định `repeat: true` **không** lặp thao tác — đúng, và đó là mặc
  // định. Nhánh còn lại phải có lưới riêng, nếu không một bản cài đặt bỏ quên `repeatable`
  // hoàn toàn vẫn xanh: giữ `Shift+→` sẽ mở rộng vùng chọn đúng một ký tự rồi đứng im, và
  // không cổng nào đỏ.
  {
    /** @type {string[]} */
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
        /** @param {string} id */
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
    /** @param {Record<string, string[]>} [overrides] */
    const chordsOf = (overrides) =>
      keysMod
        .createKeymap(base, { isMac: true }, overrides)
        .bindings()
        .map((/** @type {any} */ b) => b.chord)
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
      /** @type {string[]} */
      const seen = []
      const map = keysMod.createKeymap(
        {
          ...fakeRegistry,
          list: () => [{ id: 'x.y', labelKey: 'command.x.y', run: () => {}, keys: [chord] }],
          /** @param {string} id */
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
  // `deferred-work.md §*Deferred from: 1-7-tang-ghi-du-lieu-mot-writer-noi-tiep-va-luoc-do-co-phien-ban (2026-08-04)*` ghi mã hoá đó là TẠM vì một hợp âm chứa dấu phẩy sẽ vỡ nó.
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

/** @type {any} */
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
/** @param {string} m */
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
    eFail(`bộ command THẬT KHÔNG dựng được keymap trên ${nen}: ${errorMessage(err)}`)
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

const registeredIds = new Set(registered.map((/** @type {any} */ s) => s.id))

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
const unboundIds = indexMod.commandRegistry.unbound().map((/** @type {any} */ s) => s.id)
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
/**
 * @param {Map<string, string>} map
 * @param {string} owner
 * @param {string} where
 */
const noteOwner = (map, owner, where) => {
  if (!map.has(owner)) map.set(owner, where)
}
/**
 * Component nào tự khai điểm vào bằng một BIẾN — tức `declareFocus(props.owner, …)`.
 * Đây là nửa còn thiếu của phép nối attribute ↔ component ở dưới.
 */
const declaresViaVariable = new Map()

/**
 * ⚠️ Loại trừ ĐỊNH NGHĨA hàm. Xem lý do ở khối `FOCUS_CALL_RE` bên trên.
 * @param {string} text
 * @param {number} index
 */
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
// FR21 nói Auto-Lookup gắn vào *"một hợp đồng vùng chọn dùng chung cho **mọi**
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
// 🔵 2026-08-22 (Story 3.5 review) — đếm lại bằng chính cổng: trước lượt này có 7 lời gọi
// thật (sàn 6 đã thấp một đơn vị); `GlossarySettingsOverlay` thêm bề mặt `display` thứ tám.
// Nâng thẳng lên số thật, không giữ phần dư khiến xoá một bề mặt mà cổng vẫn xanh.
const SELECTION_SURFACE_FLOOR = 8

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

// The call-head is located on `code` (a fake call inside a prose string can't match there),
// then the role is read from `masked` at that same offset, since `code` also blanks the
// `'source'`/`'display'` literal — same split as Kiểm B's `DISPATCH_ANY_RE` + masked-slice.
const SURFACE_CALL_AT_RE = new RegExp(`^${SURFACE_CALL_RE.source}`)
for (const p of parsed) {
  if (!p.file.endsWith('.vue')) continue
  const code = p.code ?? '' // `code` is optional only for vitest fixtures; always set here
  let m
  const any = new RegExp(SURFACE_ANY_CALL_RE.source, 'g')
  while ((m = any.exec(code))) {
    anySurfaceCalls += 1
    const roleMatch = SURFACE_CALL_AT_RE.exec(p.masked.slice(m.index))
    if (roleMatch) surfaceCalls.push({ file: posix(p.file), role: roleMatch[1], index: m.index })
  }
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

// ⑥ `SELECTION_PANEL_FILES` is cross-checked two-way against the real `components` map of
// `WorkspaceDock.vue`, instead of being a hand-copied table nothing keeps in sync.
const WORKSPACE_DOCK_VUE = 'src/layout/WorkspaceDock.vue'
const dockPanel = parsed.find((p) => posix(p.file).endsWith(WORKSPACE_DOCK_VUE))
if (dockPanel === undefined) {
  fail(`\`${WORKSPACE_DOCK_VUE}\` — không tìm thấy trong quần thể quét, không đối chiếu được`)
  fBad += 1
} else {
  // Brace-BALANCED body, not `{([^}]*)}`: a value containing its own `{`/`}` (or a nested
  // object) would otherwise truncate the match at the first inner `}`.
  const objBody = balancedBraceBody(dockPanel.masked, /\bconst\s+components(?::[^=]*)?\s*=\s*\{/)
  /** @type {string[]} */
  const dockImportedFiles = []
  if (objBody === null) {
    fail(`\`${WORKSPACE_DOCK_VUE}\` — không tìm thấy \`const components = { … }\`, không đối chiếu được`)
    fBad += 1
  } else {
    const ENTRY_RE = /^([A-Za-z_$][A-Za-z0-9_$]*)\s*:\s*([A-Za-z_$][A-Za-z0-9_$]*)$/
    for (const raw of splitTopLevel(objBody)) {
      const entryText = raw.trim()
      if (entryText === '') continue
      const e = ENTRY_RE.exec(entryText)
      if (e === null) {
        // Fail-closed: a shorthand entry (`{ NewPanel }`), a spread, or anything else that
        // isn't `key: Ident` must name itself instead of being silently skipped.
        fail(`\`${WORKSPACE_DOCK_VUE}\` — mục \`${entryText}\` trong \`components\` không đọc được tĩnh (không phải \`key: Ident\`)`)
        fBad += 1
        continue
      }
      const ident = e[2]
      const importRe = new RegExp(`import\\s+${ident}\\s+from\\s+(['"])([^'"]+)\\1`)
      const im = importRe.exec(dockPanel.masked)
      if (im === null) {
        fail(
          `\`${WORKSPACE_DOCK_VUE}\` — \`components.${e[1]}\` là \`${ident}\`, không tìm thấy ` +
            `\`import ${ident} from …\` để tra ra tệp`,
        )
        fBad += 1
        continue
      }
      dockImportedFiles.push(posix(join(dirname(dockPanel.file), im[2])))
    }
  }
  for (const file of Object.keys(SELECTION_PANEL_FILES)) {
    if (!dockImportedFiles.includes(file)) {
      fail(`SELECTION_PANEL_FILES khai \`${file}\`, nhưng \`${WORKSPACE_DOCK_VUE}\` không đăng ký nó vào \`components\``)
      detail('Hai bảng đã lệch nhau — panel này không còn là một panel Workspace thật, hoặc')
      detail('SELECTION_PANEL_FILES chưa theo kịp một lượt đổi tên/gỡ panel.')
      fBad += 1
    }
  }
  for (const file of dockImportedFiles) {
    if (!(file in SELECTION_PANEL_FILES)) {
      fail(
        `\`${WORKSPACE_DOCK_VUE}\` đăng ký panel \`${file}\` vào \`components\`, nhưng ` +
          'SELECTION_PANEL_FILES không khai nó',
      )
      detail('Một panel Workspace mới đứng ngoài sổ hợp đồng vùng chọn — đúng thứ AC2 tồn tại')
      detail('để chặn (§Kiểm F đầu tệp).')
      fBad += 1
    }
  }
}

// ⑦ `registerSelectionSurface(` may only be called from `src/panels/selectionContract.ts`
// — it is the selection contract's own internal, idempotent registration function. Reads
// `p.code`, not `p.masked`, so a fake mention inside a prose string doesn't count as a call.
const REGISTER_SURFACE_CALL_RE = /\bregisterSelectionSurface\s*\(/g
const REGISTER_SURFACE_HOME = 'src/panels/selectionContract.ts'
for (const p of parsed) {
  const home = posix(p.file).endsWith(REGISTER_SURFACE_HOME)
  const code = p.code ?? '' // optional only for vitest fixtures; always set here
  const re = new RegExp(REGISTER_SURFACE_CALL_RE.source, 'g')
  let m
  while ((m = re.exec(code))) {
    if (isFunctionDefinition(code, m.index)) continue
    if (home) continue
    fail(`${at(p, m.index)} — gọi \`registerSelectionSurface(\` ngoài \`${REGISTER_SURFACE_HOME}\``)
    detail(`… ${excerpt(p.text, m.index)} …`)
    detail('Hàm này là NỘI BỘ của hợp đồng vùng chọn — dùng `useSelectionSurface` (bề mặt công')
    detail('khai) thay vì gọi thẳng, hoặc chuyển logic cần nó vào chính `selectionContract.ts`.')
    fBad += 1
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
console.log(
  '\nKiểm K — `@keydown`/`@keyup`/`@mouseup`/`@mousedown`/`@submit`: bảng đông cứng ' +
    'HAI CHIỀU (Story 11.1 lot A, Quyết định 1)',
)
// ═════════════════════════════════════════════════════════════════════════════════
//
// Every matching attribute must resolve to a `file::handler` entry in HANDLER_TABLE: the
// exact set of ids its own function body dispatches, or `nonCommand: <reason>`. A literal
// `dispatch('<id>')` value passes silently, same as Kiểm A. `@input`/`@change` stay out
// (data flow, not a user action).

/** @type {RegExp} */
const HANDLER_ATTR_RE = /^(@|v-on:)(keydown|keyup|mouseup|mousedown|submit)(\.[A-Za-z0-9.\-]+)?$/

/**
 * A handler value must be a bare identifier or exactly one call `ident(<args>)` spanning
 * the whole string; anything else (two statements, `||`, a trailing call) is unreadable.
 * @param {string} value
 * @returns {{ ident: string } | null}
 */
function parseHandlerValue(value) {
  if (/^[A-Za-z_$][A-Za-z0-9_$]*$/.test(value)) return { ident: value }
  const head = /^([A-Za-z_$][A-Za-z0-9_$]*)\(/.exec(value)
  if (!head || !value.endsWith(')')) return null
  let depth = 0
  for (let i = head[1].length; i < value.length; i += 1) {
    if (value[i] === '(') depth += 1
    else if (value[i] === ')') {
      depth -= 1
      if (depth === 0) return i === value.length - 1 ? { ident: head[1] } : null
    }
  }
  return null
}

/**
 * Whether an import statement in `code` brings `name` into scope (named, aliased via
 * `real as name`, or default).
 * @param {string} code
 * @param {string} name
 * @returns {boolean}
 */
function isImportedIdentifier(code, name) {
  const named = /import\s*\{([^}]*)\}\s*from\s*['"][^'"]*['"]/g
  let m
  while ((m = named.exec(code))) {
    for (const raw of m[1].split(',')) {
      const spec = raw.trim().replace(/^type\s+/, '')
      if (spec === '') continue
      const asMatch = /^[\w$]+\s+as\s+([\w$]+)$/.exec(spec)
      if ((asMatch ? asMatch[1] : spec) === name) return true
    }
  }
  return new RegExp(`import\\s+${name}\\s*(?:,|from)`).test(code)
}

const R_FOCUS_TRAP = 'Tab focus trap inside an overlay — moves focus in place, no dispatch'
const R_POINTER_FOCUS = 'moves focus on a pointer click — no dispatch'
const R_SCRIM_CLOSE = 'closes an overlay on a scrim click/key in place — no dispatch'
const R_CLOSE_IMPORTED =
  'closes an overlay/drawer via a state-module import — its body cannot be opened from ' +
  'this file, so the import is machine-verified but its dispatch behaviour is not'
const R_CURSOR_LOCAL = 'sets caret / grid-cell cursor memory in place — no dispatch'
const R_CURSOR_IMPORTED = 'sets grid-cell cursor memory via a state-module import — same limit as R_CLOSE_IMPORTED'
const R_SUBMIT_DIRECT = '`@submit` acting directly on local form state — a named exemption, not a command'
const R_CATEGORY_NAV = 'in-place list navigation (arrow/Enter selects a row) — no dispatch'
const R_DISPATCH_VIA_PARAM =
  'dispatches an id received as a parameter, not a literal — already counted by ' +
  "Kiểm B's nonLiteralDispatchCalls"
const R_SOURCE_CUT_EXEMPT = 'calls `setEditorSourceCut(...)` directly — a named exemption, not a command'

/**
 * @typedef {{ ids: string[] }} HandlerDispatches
 * @typedef {{ nonCommand: string }} HandlerNonCommand
 * @typedef {HandlerDispatches | HandlerNonCommand} HandlerExpectation
 */

/** @type {Record<string, HandlerExpectation>} */
const HANDLER_TABLE = {
  'src/AiPromptInspectorOverlay.vue::onEscape': { ids: ['ai.prompt_inspector.close'] },
  'src/AiPromptInspectorOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/App.vue::focusOnPointerDown': { nonCommand: R_POINTER_FOCUS },
  'src/AttributionOverlay.vue::closeAttribution': { nonCommand: R_CLOSE_IMPORTED },
  'src/AttributionOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/BilingualImportPreviewOverlay.vue::onEscapeCancel': { ids: ['import.preview.bilingual_cancel'] },
  'src/BilingualImportPreviewOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/BilingualImportPreviewOverlay.vue::onScrimKeydown': { nonCommand: R_SCRIM_CLOSE },
  'src/GlossaryImportOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/GlossaryManageOverlay.vue::onEscape': { ids: ['glossary.manage.close'] },
  'src/GlossaryManageOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/GlossaryManageOverlay.vue::onKeydown': {
    ids: ['glossary.manage.next', 'glossary.manage.prev', 'glossary.manage.edit', 'glossary.manage.delete'],
  },
  'src/GlossaryQueueOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/GlossaryQueueOverlay.vue::onKeydown': {
    ids: ['glossary.queue.next', 'glossary.queue.prev', 'glossary.queue.accept', 'glossary.queue.reject'],
  },
  'src/GlossaryQuickAdd.vue::onCategoryKeydown': { nonCommand: R_CATEGORY_NAV },
  'src/GlossarySettingsOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/ImportPreviewOverlay.vue::onEscapeCancel': { ids: ['import.preview.cancel'] },
  'src/ImportPreviewOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/ImportPreviewOverlay.vue::onScrimKeydown': { nonCommand: R_SCRIM_CLOSE },
  'src/ImportPreviewOverlay.vue::onReloadUrlItem': { nonCommand: R_SUBMIT_DIRECT },
  'src/ImportPreviewOverlay.vue::onRemoveUrlItem': { nonCommand: R_SUBMIT_DIRECT },
  'src/ImportPreviewOverlay.vue::onRemoveFileItem': { nonCommand: R_SUBMIT_DIRECT },
  'src/ImportPreviewOverlay.vue::onSaveEditCleanupRule': { nonCommand: R_SUBMIT_DIRECT },
  'src/ImportPreviewOverlay.vue::onStartEditCleanupRule': { nonCommand: R_SUBMIT_DIRECT },
  'src/ImportPreviewOverlay.vue::onDeleteCleanupRule': { nonCommand: R_SUBMIT_DIRECT },
  'src/ImportPreviewOverlay.vue::onAddCleanupRule': { nonCommand: R_SUBMIT_DIRECT },
  'src/PromptImportOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/PromptLibraryOverlay.vue::onEscape': { ids: ['prompt.library.close'] },
  'src/PromptLibraryOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/PromptLibraryOverlay.vue::selectRow': { nonCommand: R_SUBMIT_DIRECT },
  'src/PromptLibraryOverlay.vue::onOpenCreate': { nonCommand: R_SUBMIT_DIRECT },
  'src/PromptLibraryOverlay.vue::onOpenImport': { ids: ['prompt.import.open'] },
  'src/PromptLibraryOverlay.vue::onSubmitCreate': { nonCommand: R_SUBMIT_DIRECT },
  'src/PromptLibraryOverlay.vue::onCancelCreate': { nonCommand: R_SUBMIT_DIRECT },
  'src/PromptLibraryOverlay.vue::onSubmitRename': { nonCommand: R_SUBMIT_DIRECT },
  'src/PromptLibraryOverlay.vue::onSubmitBody': { nonCommand: R_SUBMIT_DIRECT },
  'src/PromptLibraryOverlay.vue::onUseSelected': { nonCommand: R_SUBMIT_DIRECT },
  'src/PromptLibraryOverlay.vue::onExportSelected': { nonCommand: R_SUBMIT_DIRECT },
  'src/PromptLibraryOverlay.vue::onDeleteSubmit': { nonCommand: R_SUBMIT_DIRECT },
  'src/SegmentHistoryOverlay.vue::closeSegmentHistory': { nonCommand: R_CLOSE_IMPORTED },
  'src/SegmentHistoryOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/SegmentHistoryOverlay.vue::aimRow': { nonCommand: R_CURSOR_LOCAL },
  'src/SettingsOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/SettingsOverlay.vue::onSelectSection': { nonCommand: R_SUBMIT_DIRECT },
  'src/SettingsOverlay.vue::onSaveAiConfigField': { nonCommand: R_SUBMIT_DIRECT },
  'src/SettingsOverlay.vue::onClearAiConfigOverride': { nonCommand: R_SUBMIT_DIRECT },
  'src/SettingsOverlay.vue::onSaveAiConfigKey': { nonCommand: R_SUBMIT_DIRECT },
  'src/SettingsOverlay.vue::onDeleteAiConfigKey': { nonCommand: R_SUBMIT_DIRECT },
  'src/ShortcutsOverlay.vue::onEscape': { ids: ['shortcuts.close'] },
  'src/ShortcutsOverlay.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/ShortcutsOverlay.vue::aimRowFrom': { nonCommand: R_CURSOR_IMPORTED },
  'src/ShortcutsOverlay.vue::onKeyCellKeydown': { ids: ['shortcuts.unassign'] },
  'src/layout/LookupDrawer.vue::closeLookupDrawer': { nonCommand: R_CLOSE_IMPORTED },
  'src/layout/LookupDrawer.vue::trapTab': { nonCommand: R_FOCUS_TRAP },
  'src/modes/ReadingMode.vue::onReadingSegmentEnter': { ids: ['reading.open_aimed'] },
  'src/modes/ReadingMode.vue::trapOverlayTab': { nonCommand: R_FOCUS_TRAP },
  'src/panels/GridPanel.vue::onSourceCellMouseUp': { nonCommand: R_SOURCE_CUT_EXEMPT },
  'src/panels/GridPanel.vue::onCellMouseDown': { nonCommand: R_CURSOR_LOCAL },
  'src/panels/GridPanel.vue::onCellMouseUp': { nonCommand: R_CURSOR_LOCAL },
  'src/panels/GridPanel.vue::onEditKeydown': { ids: ['editor.clear_source_cuts', 'editor.merge_segments'] },
  'src/panels/LookupPanel.vue::aimDictSourceFrom': { nonCommand: R_CURSOR_IMPORTED },
  'src/panels/LookupPanel.vue::moveTabFocus': { nonCommand: R_DISPATCH_VIA_PARAM },
  'src/panels/LookupPanel.vue::aimLookupEntryFrom': { nonCommand: R_CURSOR_IMPORTED },
}

/**
 * Judges a single attribute — shared by the real scan below and the self-check, so the
 * self-check always exercises the real function, not a copy.
 * @param {ParsedFile} p
 * @param {TemplateAttr} a
 * @param {Record<string, HandlerExpectation>} table
 * @returns {{ key: string|null, problems: string[] }}
 */
function judgeHandlerInventory(p, a, table) {
  const value = a.value.trim()
  if (DISPATCH_ONLY_RE.test(value)) return { key: null, problems: [] }
  const handlerValue = parseHandlerValue(value)
  if (handlerValue === null) {
    return {
      key: null,
      problems: [
        `${at(p, a.index)} — \`${a.name}="${value}"\` không đọc được tĩnh (không phải ` +
          "`dispatch('<id>')`, một tên hàm trần, hay đúng MỘT lời gọi `ident(...)`)",
      ],
    }
  }
  const key = `${posix(p.file)}::${handlerValue.ident}`
  const entry = table[key]
  if (entry === undefined) {
    return {
      key,
      problems: [
        `${at(p, a.index)} — \`${key}\` chưa có trong HANDLER_TABLE (handler mới hoặc đổi tên — ` +
          'xếp nó vào một bộ id nó dispatch, hoặc `nonCommand: <lý do>`)',
      ],
    }
  }
  const code = p.code ?? ''
  const range = functionBodyRange(code, handlerValue.ident)
  /** @type {string[]|null} */
  let actualIds = null
  if (range !== null) {
    const body = p.masked.slice(range.start, range.end)
    const ids = new Set()
    const re = new RegExp(DISPATCH_CALL_RE.source, 'g')
    let m
    while ((m = re.exec(body))) ids.add(m[2])
    actualIds = [...ids].sort()
  }
  if ('nonCommand' in entry) {
    if (actualIds !== null && actualIds.length > 0) {
      return {
        key,
        problems: [`${key} — khai \`nonCommand\`, nhưng thân hàm THẬT dispatch ${JSON.stringify(actualIds)}`],
      }
    }
    // `masked`, not `code`: `code` blanks the quote characters of the import path too.
    if (actualIds === null && !isImportedIdentifier(p.masked, handlerValue.ident)) {
      return {
        key,
        problems: [
          `${key} — khai \`nonCommand\`, nhưng không mở được thân hàm cục bộ VÀ không thấy ` +
            `import nào đưa \`${handlerValue.ident}\` vào tệp này — không đọc được tĩnh`,
        ],
      }
    }
    return { key, problems: [] }
  }
  if (actualIds === null) {
    return {
      key,
      problems: [
        `${key} — khai bộ id ${JSON.stringify(entry.ids)}, nhưng không tìm thấy định nghĩa hàm ` +
          'cục bộ để đối chiếu',
      ],
    }
  }
  const want = [...entry.ids].sort()
  if (want.join('\u0000') !== actualIds.join('\u0000')) {
    return {
      key,
      problems: [
        `${key} — khai dispatch ${JSON.stringify(want)}, thân hàm THẬT dispatch ` +
          `${JSON.stringify(actualIds)} (bộ id đã TRÔI)`,
      ],
    }
  }
  return { key, problems: [] }
}

/**
 * Which HANDLER_TABLE keys matched no scanned attribute — shared by the real run and the
 * self-check, so a dangling entry is caught by the same code path in both.
 * @param {Record<string, HandlerExpectation>} table
 * @param {Set<string>} seenKeys
 * @returns {string[]}
 */
function danglingHandlerKeys(table, seenKeys) {
  return Object.keys(table).filter((k) => !seenKeys.has(k))
}

/**
 * Self-check: the gate must go red on a bad case and not go red on a good one. Calls the
 * real buildParsedEntry/judgeHandlerInventory/danglingHandlerKeys on a fake fixture.
 * @returns {string[]}
 */
function selfCheckHandlerInventory() {
  /** @type {string[]} */
  const problems = []
  const fixtureFile = join(REPO_ROOT, '__selfcheck__', 'Fixture.vue')
  const fixtureText = [
    '<template>',
    '  <button @keydown="onFakeDispatch" />',
    '  <button @keydown="onFakeQuiet" />',
    '  <button @keydown="onFakeUnlisted" />',
    '</template>',
    '<script setup lang="ts">',
    "import { onFakeImported } from './fakeModule'",
    'function onFakeDispatch(event: KeyboardEvent): void {',
    "  dispatch('fake.command')",
    '}',
    'function onFakeQuiet(event: KeyboardEvent): void {',
    '  closeSomething()',
    '}',
    '</script>',
  ].join('\n')
  const p = buildParsedEntry(fixtureFile, fixtureText)
  const attrs = scanVueAttrs([p]).filter(({ a }) => HANDLER_ATTR_RE.test(a.name))
  if (attrs.length !== 3) {
    problems.push('fixture phải quét đúng BA thuộc tính @keydown — tự kiểm không dựng đúng ca thử')
    return problems
  }
  const byValue = new Map(attrs.map(({ a }) => [a.value.trim(), a]))
  const dispatchAttr = byValue.get('onFakeDispatch')
  const quietAttr = byValue.get('onFakeQuiet')
  const unlistedAttr = byValue.get('onFakeUnlisted')
  if (dispatchAttr === undefined || quietAttr === undefined || unlistedAttr === undefined) {
    problems.push('fixture thiếu một trong ba thuộc tính mong đợi — tự kiểm không dựng đúng ca thử')
    return problems
  }

  // Case 1: unlisted handler must be caught.
  if (judgeHandlerInventory(p, unlistedAttr, {}).problems.length === 0) {
    problems.push('ca ①: handler chưa khai trong bảng PHẢI bị bắt (unlisted) — cổng không đỏ')
  }

  // Case 2: a correctly declared ids/nonCommand entry must not be flagged.
  const tableDung = {
    [`${posix(fixtureFile)}::onFakeDispatch`]: { ids: ['fake.command'] },
    [`${posix(fixtureFile)}::onFakeQuiet`]: { nonCommand: 'ca tự kiểm — không dispatch' },
  }
  const r2a = judgeHandlerInventory(p, dispatchAttr, tableDung)
  const r2b = judgeHandlerInventory(p, quietAttr, tableDung)
  if (r2a.problems.length > 0) problems.push(`ca ②a: khai ĐÚNG bộ id vẫn bị đỏ oan — ${r2a.problems.join('; ')}`)
  if (r2b.problems.length > 0) problems.push(`ca ②b: khai ĐÚNG nonCommand vẫn bị đỏ oan — ${r2b.problems.join('; ')}`)

  // Case 3: a wrong id set (dispatch-set drift) must be caught.
  const tableTroi = { [`${posix(fixtureFile)}::onFakeDispatch`]: { ids: ['fake.other_command'] } }
  if (judgeHandlerInventory(p, dispatchAttr, tableTroi).problems.length === 0) {
    problems.push('ca ③: bộ id đã khai SAI với thân hàm thật (trôi) PHẢI bị bắt — cổng không đỏ')
  }

  // Case 4: `nonCommand` declared while the real body dispatches must be caught (reverse drift).
  const tableNguoc = { [`${posix(fixtureFile)}::onFakeDispatch`]: { nonCommand: 'sai — hàm này CÓ dispatch' } }
  if (judgeHandlerInventory(p, dispatchAttr, tableNguoc).problems.length === 0) {
    problems.push('ca ④: khai `nonCommand` trong khi thân hàm THẬT dispatch PHẢI bị bắt — cổng không đỏ')
  }

  // Case 5: a table key matching no scanned attribute must be caught (dangling entry).
  const tableTreo = {
    [`${posix(fixtureFile)}::onFakeDispatch`]: { ids: ['fake.command'] },
    [`${posix(fixtureFile)}::onGoneHandler`]: { nonCommand: 'ca tự kiểm — mục treo' },
  }
  const seenTreo = new Set()
  const rTreo = judgeHandlerInventory(p, dispatchAttr, tableTreo)
  if (rTreo.key !== null) seenTreo.add(rTreo.key)
  if (danglingHandlerKeys(tableTreo, seenTreo).length === 0) {
    problems.push('ca ⑤: một mục HANDLER_TABLE không còn khớp gì PHẢI bị bắt (mục treo) — cổng không đỏ')
  }

  // Case 6: `nonCommand` on a handler that is neither locally openable nor imported must be
  // caught — a null body alone (arrow function, typo, renamed import) is not proof enough.
  const ghostAttr = { name: '@keydown', value: 'onFakeGhost', index: 0 }
  const tableGhost = { [`${posix(fixtureFile)}::onFakeGhost`]: { nonCommand: 'ca tự kiểm — không mở được' } }
  if (judgeHandlerInventory(p, ghostAttr, tableGhost).problems.length === 0) {
    problems.push('ca ⑥: `nonCommand` không mở được thân VÀ không thấy import PHẢI bị bắt — cổng không đỏ')
  }

  // Case 6b: the same shape, but the identifier IS imported, must NOT be flagged.
  const importedAttr = { name: '@keydown', value: 'onFakeImported', index: 0 }
  const tableImported = { [`${posix(fixtureFile)}::onFakeImported`]: { nonCommand: 'ca tự kiểm — qua import' } }
  const rImported = judgeHandlerInventory(p, importedAttr, tableImported)
  if (rImported.problems.length > 0) {
    problems.push(`ca ⑥b: một handler ĐÃ import vẫn bị đỏ oan — ${rImported.problems.join('; ')}`)
  }

  // Case 7: a value that is not a bare identifier nor a single whole-value call (two
  // statements, an operator) must FAIL as unreadable, not silently key on the leading name.
  const multiStmtAttr = { name: '@keydown', value: "onFakeQuiet(); dispatch('sneaky.id')", index: 0 }
  const rMulti = judgeHandlerInventory(p, multiStmtAttr, {})
  if (rMulti.key !== null || rMulti.problems.length === 0) {
    problems.push('ca ⑦: một giá trị nhiều lệnh PHẢI bị bắt là không đọc được tĩnh — cổng không đỏ')
  }

  return problems
}

for (const problem of selfCheckHandlerInventory()) fail(`TỰ KIỂM Kiểm K: ${problem}`)

let kBad = 0
let handlerAttrCount = 0
const seenHandlerKeys = new Set()
for (const { p, a } of scanVueAttrs(parsed)) {
  if (!HANDLER_ATTR_RE.test(a.name)) continue
  handlerAttrCount += 1
  const { key, problems } = judgeHandlerInventory(p, a, HANDLER_TABLE)
  if (key !== null) seenHandlerKeys.add(key)
  for (const msg of problems) {
    fail(msg)
    kBad += 1
  }
}
for (const tableKey of danglingHandlerKeys(HANDLER_TABLE, seenHandlerKeys)) {
  fail(`HANDLER_TABLE khai \`${tableKey}\`, nhưng không còn thuộc tính nào khớp — mục TREO`)
  detail('Handler đã đổi tên, tệp đã gỡ, hoặc thuộc tính đã đổi loại sự kiện. Xoá mục này khỏi')
  detail('HANDLER_TABLE hoặc sửa lại tên cho khớp mã nguồn hiện tại.')
  kBad += 1
}

// Population floor, same reasoning as CLICK_FLOOR: an empty scan must not read as a pass.
const HANDLER_ATTR_FLOOR = 75
if (handlerAttrCount < HANDLER_ATTR_FLOOR) {
  abort(
    `thuộc tính @keydown/@keyup/@mouseup/@mousedown/@submit quét được — ${handlerAttrCount} ` +
      `(sàn ${HANDLER_ATTR_FLOOR})`,
    new Error('Ít hơn sàn nghĩa là tầng quét đã mất một vùng template — kiểm `vueRegions` trước khi hạ sàn.'),
  )
}
if (kBad === 0) {
  pass(
    `${handlerAttrCount} thuộc tính @keydown/@keyup/@mouseup/@mousedown/@submit trên ` +
      `${Object.keys(HANDLER_TABLE).length} handler đã khai — đúng bảng, không mục treo`,
  )
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
console.log('  1. Kiểm A chỉ canh `@click`. `@input`/`@change` KHÔNG thuộc luật này (dòng dữ liệu,')
console.log('     AD-34 §1); `@keydown`/`@keyup`/`@mouseup`/`@mousedown`/`@submit` có Kiểm K riêng.')
console.log('  2. Vế DOM của AC4 (*"focus không rơi về `body`"*) KHÔNG kiểm được ở đây — nó là')
console.log('     hành vi lúc chạy trong một webview thật. Chốt tự kêu ở `src/commands/focus.ts`')
console.log('     cộng nghiệm thu tay; giới hạn ghi ở `deferred-work.md`. Không đánh dấu đạt.')
console.log('  3. Cổng KHÔNG canh focus ring. Một `*:focus { outline: none }` phá NFR17 mà vẫn')
console.log('     qua được cả cổng này lẫn `check-tokens.mjs` (§Trap 4 của Story 1.6).')
process.exit(0)
