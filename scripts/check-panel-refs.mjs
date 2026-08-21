#!/usr/bin/env node
/**
 * CỔNG — **mọi ô nhớ cấp module trong `src/**\/*.ts` phải đi qua một hàm reset, hoặc mang
 * một miễn trừ CÓ TÊN kèm lý do đọc được tại chỗ.** Story 2.12 · AC5.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO CỔNG NÀY TỒN TẠI — một luật đã bị bỏ sót HAI STORY LIÊN TIẾP
 * ═════════════════════════════════════════════════════════════════════════════════
 * `resetEditorPanel()` là đường duy nhất dọn state Panel Editor khi Tác phẩm đổi. Hai ô nhớ
 * mới thêm vào tệp đó **không** được nối vào nó:
 *   · `sourceCut`  — Story 2.8
 *   · `omitError`  — Story 2.9
 * Cả hai lượt đều đi qua **chín cổng, `cargo test`, và vitest** mà không một lượt đỏ nào.
 * Chú thích trong chính `resetEditorPanel` đã viết trước điều đó: *"một luật chỉ sống trong
 * một khối chú thích là một luật sẽ bị quên lần thứ ba"*.
 *
 * Hình dạng hỏng, và nó không ném lỗi nào: đổi Tác phẩm ⇒ ô còn giữ dữ liệu của Tác phẩm
 * CŨ ⇒ màn hình khẳng định một điều thuộc về một Tác phẩm khác. `segment.id` không tái dùng
 * **trong một kho**, nhưng hai kho đánh số **độc lập** — nên một id cũ tồn tại thật ở kho
 * mới và trỏ vào một câu khác hẳn.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 PHẠM VI — Ice ký 2026-08-18 (quyết định #2b): **toàn `src/**\/*.ts`**
 * ═════════════════════════════════════════════════════════════════════════════════
 * Hai đường bị loại, mỗi đường một lý do đo được:
 *
 * - *"chỉ `src/panels/**`"* — một tệp state cấp module đặt ngoài `panels/` không được canh,
 *   và quần thể đo 2026-08-18 cho **30 `let` cấp module ở NGOÀI `src/panels/`**.
 * - *"kèm `.vue` có luật phân biệt"* — 🔴 **cạm bẫy đỏ oan.** `const x = ref()` ở đầu
 *   `<script setup>` là state cấp **COMPONENT-INSTANCE**: nó dựng lại mỗi lần component
 *   mount, nên nó **không** cần một hàm reset. Một cổng không phân biệt sẽ đỏ ở
 *   `LookupPanel.vue` ×2 · `PanelTab.vue` · `PanelFrame.vue` · `App.vue` · `StatusBar.vue`
 *   — đo được **6** chỗ — rồi bị nới cho hết đỏ.
 *
 * ⇒ Loại `.vue` **theo đuôi tệp**, không theo một parser SFC tập con tự viết. Đuôi tệp là
 * một mệnh đề không thể đọc sai; một parser SFC là chính chỗ cổng có thể sai.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * §GIỚI HẠN THẬT — ghi ra thay vì để người sau tự phát hiện
 * ═════════════════════════════════════════════════════════════════════════════════
 * 1. **Cổng đi theo TÊN, không theo luồng dữ liệu.** Một ô được dọn bằng một con đường
 *    không nhắc tên nó *(một `Object.assign`, một vòng lặp trên `Object.keys`)* sẽ bị chấm
 *    là chưa dọn. Đó là chiều đỏ **oan**, và nó vá bằng một miễn trừ có tên — không bằng
 *    việc nới luật.
 * 2. **Một dòng gán trong hàm reset không chứng minh giá trị gán là ĐÚNG.** Cổng thấy
 *    `x.value = null`; nó không biết `null` có phải trạng thái đầu hay không. Mệnh đề ấy
 *    thuộc `tests/frontend/**`, không thuộc đây.
 * 3. **Chỉ theo lời gọi hàm SÂU MỘT TẦNG** từ thân hàm reset. `resetEditorPanel()` gọi
 *    `clearFlushTimer()`, và tầng một là đủ cho mọi chỗ dùng hôm nay. Một chuỗi hai tầng sẽ
 *    bị chấm là chưa dọn — lại là chiều đỏ oan, vá bằng miễn trừ.
 * 4. 🔵 **Tầng một ấy bị chặn ở ranh giới TỆP** *(code review 2026-08-19 — chỗ duy nhất trong
 *    tệp này chưa tự khai chỗ yếu)*. Vòng tìm hàm phụ chỉ quét trên `source` của **chính tệp
 *    đang xét**, nên một hàm dọn được `import` từ tệp khác rồi gọi trong `reset*()` là **vô
 *    hình** ⇒ ĐỎ OAN. Chiều an toàn, nhưng phải viết ra: không cổng nào canh việc người sau
 *    tưởng *"sâu một tầng"* đã bao gồm cả tầng liên-tệp.
 * 5. 🔵 **Cổng nhận diện ô nhớ trên MỘT DÒNG, và tập cú pháp là một TẬP CON nghiêm ngặt**
 *    *(code review 2026-08-19)*. `const a = ref(0), b = ref(0)` và một khai báo trải hai dòng
 *    nằm **ngoài** tập con. Chúng **không** bị bỏ qua trong im lặng — [`failOutOfSubset`] cho
 *    ĐỎ kèm câu *"cú pháp ngoài tập con"*, đúng khuôn mà `project-context.md` đặt cho các
 *    parser TOML/CSS tự viết trong `scripts/`: *"cú pháp ngoài tập con ⇒ FAIL, không bỏ qua"*.
 *    Một lượt bỏ qua im lặng ở đây là một ô nhớ cấp module **cổng không bao giờ thấy**.
 */

import { lstatSync, readdirSync, readFileSync, realpathSync } from 'node:fs'
import { dirname, join, relative, sep } from 'node:path'
import { fileURLToPath } from 'node:url'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const SRC_ROOT = join(REPO_ROOT, 'src')
const posix = (p) => relative(REPO_ROOT, p).split(sep).join('/')

let failures = 0

/**
 * 🔴 Lỗi HẠ TẦNG — **KHÔNG** một phép kiểm đỏ. *"Đừng bao giờ báo một kết quả không có thật."*
 * Thoát khác 0, và nói thẳng rằng đây không phải một lượt đạt.
 */
function abort(what, err) {
  console.error(`\n  LOI HA TANG — day KHONG phai mot ket qua dat.`)
  console.error(`  Khong doc duoc: ${what}`)
  console.error(`  ${err instanceof Error ? err.message : String(err)}\n`)
  process.exit(2)
}

function fail(msg) {
  failures += 1
  console.error(`  FAIL  ${msg}`)
}

function walk(dir, out = [], seen = new Set()) {
  // 🔵 **CODE REVIEW BA TẦNG 2026-08-19** — bản đầu dùng `statSync` *(THEO liên kết)* và không
  // mang `seen`. Một symlink trỏ ngược vào một thư mục tổ tiên trong `src/**` cho **đệ quy vô
  // hạn** ⇒ stack overflow, tức tiến trình Node chết mà **không** một câu chẩn đoán nào —
  // ngược đúng luật *"lỗi hạ tầng KHÔNG phải một phép kiểm đỏ; nó `abort()` kèm một câu"*.
  // ⇒ Ba thứ dưới đây chép **đúng** khuôn `check-layout.mjs:65-77` đã thêm cho cùng nhu cầu:
  // `realpathSync` làm khoá danh tính · `seen` chặn vòng · `lstatSync` **không** theo liên kết.
  let key
  try {
    key = realpathSync(dir)
  } catch {
    key = dir
  }
  if (seen.has(key)) return out
  seen.add(key)
  for (const name of readdirSync(dir)) {
    const full = join(dir, name)
    if (lstatSync(full).isDirectory()) walk(full, out, seen)
    // 🔴 `.ts` thôi — xem §Phạm vi. `.vue` loại theo ĐUÔI TỆP, không theo một parser.
    else if (name.endsWith('.ts')) out.push(full)
  }
  return out
}

// ═══════════════════════════════════════════════════════════════════════════════════
// MIỄN TRỪ CÓ TÊN — khuôn `Map<khoá, lý do>` của `check-gates.mjs`
// ═══════════════════════════════════════════════════════════════════════════════════
/**
 * 🔴 **Ba luật của một miễn trừ ở kho này, và cả ba đều cưỡng chế được:**
 *   ① **có tên** — khoá là `đường/tệp.ts::tênÔ`, không một mẫu glob;
 *   ② **có lý do đọc được tại chỗ** — cổng đỏ nếu lý do rỗng;
 *   ③ **chết được** — một miễn trừ cho một ô **không còn tồn tại** làm cổng ĐỎ. Đó đúng lớp
 *      nợ mà `reportUnusedDisableDirectives: 'error'` của `eslint.config.js` tồn tại để
 *      chống: một miễn trừ hết cần mà ở lại là một luật đã tắt mà không ai biết.
 */
const EXEMPT = new Map([
  [
    'src/panels/lookupTiming.ts::enabled',
    'Công tắc chẩn đoán do NGƯỜI ĐO bật, không một mẩu state của phiên. `resetLookupTiming()` ' +
      'cố ý không chạm nó: dọn nó là tự tắt bàn đo giữa một lượt đo.',
  ],
  [
    'src/panels/wordBoundary.ts::segmenter',
    '`Intl.Segmenter` dựng một lần rồi dùng mãi — một bộ nhớ đệm THUẦN, không mang một byte ' +
      'dữ liệu người dùng nào. Dựng lại nó khi đổi Tác phẩm là trả tiền cho cùng một kết quả.',
  ],
  [
    'src/i18n/index.ts::warnedErrors',
    'Tập khoá đã cảnh báo MỘT LẦN, tồn tại đúng để không lặp lại một dòng log. Dọn nó khi đổi ' +
      'Tác phẩm làm mỗi lượt đổi in lại trọn bộ cảnh báo cũ — ngược hẳn mục đích của nó.',
  ],
  [
    'src/commands/keys.ts::attached',
    'Cờ "đã gắn listener vào `window`" — vòng đời của nó là vòng đời TIẾN TRÌNH, không vòng ' +
      'đời Tác phẩm. Dọn nó cho phép `attachKeyboard` gắn listener lần thứ hai (AD-34).',
  ],
  [
    'src/modes/libraryImport.ts::dragDropWired',
    'Cùng hạng với `keys.ts::attached`: cờ một-lần cho lượt nối kéo-thả ở tầng cửa sổ.',
  ],
  [
    'src/modes/libraryImport.ts::unlisteners',
    'Các hàm nhả của listener cấp cửa sổ, ghép cặp với `dragDropWired`. Dọn nó là rò rỉ ' +
      'listener: hàm nhả mất, listener ở lại.',
  ],
  [
    'src/tokens/fonts.ts::inflight',
    'Chống nạp font hai lần trong MỘT phiên. Font là tài nguyên cấp ứng dụng, không theo Tác ' +
      'phẩm — CSP ghim `$RESOURCE/fonts/**`, không có đường nào tải font theo Tác phẩm.',
  ],
  [
    'src/layout/dockController.ts::live',
    'Các hàm do `main.ts` TIÊM vào (`src/layout/README`: `main.ts` tiêm, không import) — một ' +
      'bảng nối dây cấp ứng dụng. Dọn nó làm mọi lệnh bố cục ném vì chưa được nối.',
  ],
  // ── Phím tắt + bố cục: BA loại "chỉ toàn cục" của AD-18 ──────────────────────────
  //
  // 🔴 `ScopeResolver` **trả lỗi** khi có ai ghi ba loại này ở tầng Tác phẩm — phím tắt,
  // preset bố cục, lựa chọn ứng dụng. Một ô nhớ chỉ-toàn-cục mà đi qua một hàm reset theo
  // Tác phẩm là mâu thuẫn thẳng với AD-18: nó sẽ dọn một giá trị mà tầng Tác phẩm không
  // được phép có ý kiến, rồi không ai đọc lại nó từ đĩa.
  [
    'src/commands/index.ts::keymap',
    'Bảng phím tắt đang hiệu lực — loại "chỉ toàn cục" của AD-18. Nạp một lần lúc khởi động; ' +
      'dọn nó khi đổi Tác phẩm làm mọi hợp âm chết cho tới lần khởi động sau.',
  ],
  [
    'src/commands/index.ts::liveOverrides',
    'Các lượt gán đè phím tắt của người dùng, ghép cặp với `keymap`. Cùng tầng, cùng lý do.',
  ],
  [
    'src/commands/index.ts::diskRejection',
    'Câu từ chối của lượt đọc phím tắt từ đĩa — một chẩn đoán của lượt KHỞI ĐỘNG. Dọn nó là ' +
      'xoá bằng chứng về một lượt đọc hỏng mà không có lượt đọc thứ hai nào để dựng lại nó.',
  ],
  [
    'src/config/shortcutsState.ts::overlayOpen',
    'Màn hình gán phím đang mở hay không — state của MÀN HÌNH, không của Tác phẩm. ' +
      '`resetShortcut()` cùng tệp là "trả MỘT phím tắt về mặc định", một khái niệm khác hẳn.',
  ],
  [
    'src/config/shortcutsState.ts::aimedRow',
    'Hàng đang ngắm trong màn hình gán phím. Cùng vòng đời với `overlayOpen`.',
  ],
  [
    'src/config/shortcutsState.ts::capturing',
    'Đang bắt một hợp âm hay không. Cùng vòng đời với `overlayOpen` — và dọn nó giữa một ' +
      'lượt bắt sẽ bỏ rơi lượt bắt đó mà không ai báo.',
  ],

  // ── Ảnh chụp lúc KHỞI ĐỘNG — không có lượt đọc thứ hai để dựng lại ───────────────
  [
    'src/config/bootstrap.ts::lastError',
    'Lỗi của lượt nạp cấu hình KHỞI ĐỘNG. Nó có đúng một lượt ghi, ở một thời điểm trước khi ' +
      'Tác phẩm đầu tiên tồn tại — nên "đổi Tác phẩm" không phải một sự kiện của nó.',
  ],
  [
    'src/config/bootstrap.ts::layout',
    'Preset bố cục đã nạp — loại "chỉ toàn cục" thứ hai của AD-18, cùng lý do với `keymap`.',
  ],
  [
    'src/modes/modeState.ts::mode',
    'Chế độ đang mở (library/workspace). Dọn nó khi đổi Tác phẩm sẽ ném người dùng về màn ' +
      'hình Library ngay giữa lượt họ vừa mở một Tác phẩm — nó là state của PHIÊN.',
  ],

  // ── SỔ ĐĂNG KÝ phần tử DOM — Vue đã sở hữu vòng đời, đừng dựng chủ thứ hai ───────
  //
  // ⚠️ Cả hai ô đều nhận mục lúc `mount` và nhả lúc `unmount`, qua hàm nhả mà chính chúng
  // trả về. Dọn chúng từ bên ngoài **không** gỡ được listener nào — nó chỉ làm sổ trống
  // trong khi phần tử vẫn sống, và hàm nhả sau đó tìm một mục không còn ở đó.
  [
    'src/panels/hanVietSurfaces.ts::entries',
    'Sổ đăng ký ô Hán Việt đang mount. Vòng đời do Vue sở hữu (mount/unmount), không do lượt ' +
      'đổi Tác phẩm — dọn từ ngoài là dựng một chủ thứ hai cho cùng một vòng đời.',
  ],
  [
    'src/panels/selectionContract.ts::surfaces',
    'Sổ đăng ký bề mặt vùng chọn. Cùng hình dạng và cùng lý do với `hanVietSurfaces::entries`.',
  ],

  // ── Bộ ghim: dữ liệu tầng GLOBAL, nạp MỘT LẦN lúc khởi động ─────────────────────
  //
  // 🔴 Cùng hình dạng hỏng với `dictSourcesState::disabled`, và nó đáng ghi lại ở đây thay
  // vì trỏ đi chỗ khác: `loadPinnedEntries()` được gọi **một lần**, từ `src/main.ts`. Dọn
  // các ô này khi đổi Tác phẩm sẽ xoá bộ ghim trong bộ nhớ mà **không ai đọc lại nó từ
  // đĩa** ⇒ tab Ghim rỗng cho tới lần khởi động sau, trong khi đĩa vẫn đủ.
  [
    'src/panels/lookupHistoryState.ts::pinnedRaw',
    'Bộ ghim đọc từ tầng Global, nạp một lần từ `main.ts`. `null` ⇔ CHƯA có câu trả lời — ' +
      'khác hẳn `[]`. Dọn nó là dựng lại đúng trạng thái "chưa hỏi" mà không phát lượt hỏi nào.',
  ],
  [
    'src/panels/lookupHistoryState.ts::pinnedErr',
    'Lỗi của lượt ĐỌC bộ ghim, ghép cặp với `pinnedRaw`. Cùng vòng đời, cùng lý do.',
  ],
  [
    'src/panels/lookupHistoryState.ts::loadSequence',
    'Số thứ tự của lượt nạp bộ ghim, ghép cặp với `pinnedRaw`. Lùi nó về 0 mà `pinnedRaw` ' +
      'vẫn còn là mở lại đúng ca "lượt cũ về sau" mà nó tồn tại để chặn.',
  ],
  [
    'src/panels/lookupHistoryState.ts::pinWriteQueue',
    'Hàng đợi ghi bộ ghim — NỐI TIẾP, và bộ ghim là dữ liệu tầng Global nên một lượt ghi đang ' +
      'bay vẫn hợp lệ sau khi Tác phẩm đổi. Khác `editorPanelState::inFlight`, ô kia mang một ' +
      'lượt bay THEO Tác phẩm nên nó BẮT BUỘC bị dọn.',
  ],
  [
    'src/panels/lookupHistoryState.ts::tab',
    'Tab đang mở của Panel Lookup (Bản ghi / Lịch sử / Ghim) — một lựa chọn của người dùng về ' +
      'cách họ muốn nhìn, không một mẩu dữ liệu của Tác phẩm. Dọn nó là tự ý bẻ lái mắt họ về ' +
      'tab đầu mỗi lượt đổi Tác phẩm.',
  ],

  [
    'src/commands/index.ts::installedIsMac',
    'Nền tảng của MÁY đang chạy. Nó không đổi giữa hai Tác phẩm, và cũng không đổi giữa hai ' +
      'lần khởi động trên cùng một máy.',
  ],
])

/**
 * 🔵 **CODE REVIEW BA TẦNG 2026-08-19 — HAI HÌNH DẠNG KHAI BÁO NGOÀI TẬP CON, và vì sao chúng
 * phải cho ĐỎ chứ không được bỏ qua.**
 *
 * Ba biểu thức dưới đây nhận diện ô nhớ **trên MỘT DÒNG** và mỗi biểu thức khớp **một lần**
 * mỗi dòng. Hai hình dạng hợp lệ của TypeScript nằm ngoài tập ấy:
 *   ⒜ `const a = ref(0), b = ref(0)` — nhiều declarator một dòng ⇒ chỉ `a` được thấy;
 *   ⒝ `const x =` rồi `  ref(0)` ở dòng sau ⇒ **không** dòng nào khớp, ô **vô hình**.
 * Cả hai cho một ô nhớ cấp module mà cổng **không bao giờ thấy** — tức đúng lớp xanh-oan mà
 * cổng này tồn tại để chống, chỉ đi vào bằng một cửa khác.
 *
 * 🔴 ⇒ Không nới luật, không im lặng: **FAIL kèm câu *"cú pháp ngoài tập con"***, đúng khuôn
 * `project-context.md` đặt cho mọi parser tự viết trong `scripts/` — *"cú pháp ngoài tập con
 * ⇒ FAIL, không bỏ qua"*. Người viết chọn: tách thành hai dòng, hoặc thêm một miễn trừ có tên.
 *
 * ⚠️ Chạy trên dòng ĐÃ tách chú thích/chuỗi *(xem [`stripNonCode`])*, và dấu phẩy chỉ tính ở
 * **độ sâu 0** của `(`/`[`/`{`. Đó là điều kiện để `new Map<string, number>()` và
 * `const BANG = { a: 1, b: 2 }` **không** bị chấm oan — hai hình dạng có thật trong `src/**`.
 *
 * @param {string} codeLine dòng đã qua `stripNonCode`
 * @returns {string | null} lý do, hoặc `null` nếu trong tập con
 */
function outOfSubset(codeLine) {
  if (!/^(?:const|let)\s+[A-Za-z_$][\w$]*/.test(codeLine)) return null
  if (/^(?:const|let)\s+[A-Za-z_$][\w$]*\s*(?::[^=]*)?=\s*$/.test(codeLine)) {
    return 'khai bao trai HAI DONG (dong ket thuc bang `=`)'
  }
  let depth = 0
  for (let i = 0; i < codeLine.length; i += 1) {
    const c = codeLine[i]
    if (c === '(' || c === '[' || c === '{') depth += 1
    else if (c === ')' || c === ']' || c === '}') depth -= 1
    else if (c === ',' && depth === 0) {
      // Chỉ là một declarator thứ hai nếu sau dấu phẩy là `ten =` hoặc `ten:`.
      if (/^\s*[A-Za-z_$][\w$]*\s*(?::|=(?!=))/.test(codeLine.slice(i + 1))) {
        return 'NHIEU declarator tren mot dong (`const a = …, b = …`)'
      }
    }
  }
  return null
}

/** Ô nhớ cấp module: `ref`/`shallowRef`/`reactive`, `let`, và `const` gán một hộp RỖNG. */
const RE_REACTIVE = /^(?:const|let)\s+([A-Za-z_$][\w$]*)\s*(?::[^=]*)?=\s*(?:ref|shallowRef|reactive|shallowReactive)\s*[(<]/
const RE_LET = /^let\s+([A-Za-z_$][\w$]*)\b/
const RE_EMPTY_BOX = /^const\s+([A-Za-z_$][\w$]*)\s*(?::[^=]*)?=\s*(?:\[\s*\]|\{\s*\}|new\s+(?:Map|Set|WeakMap|WeakSet)\s*[(<])/

/**
 * 🔵 **CODE REVIEW BA TẦNG 2026-08-19 — CỬA XANH-OAN THỨ HAI CỦA CỔNG NÀY, cùng hạng với
 * lượt đột biến Task 5.5 nhưng ở một cơ chế KHÁC.**
 *
 * Lượt đột biến 5.5 đóng câu hỏi *"một lượt ĐỌC có thành bằng chứng cho một lượt DỌN không"*.
 * Nó **không** đóng câu hỏi *"văn bản mà cổng đang đọc có phải MÃ không"*. Hai cửa còn hở:
 *
 * 1. [`isAssignedIn`] khớp trên văn bản thô ⇒ một dòng chú thích `// sourceCut.value = null`
 *    hay một chuỗi lỗi chứa nguyên văn ấy **thành bằng chứng đã dọn**. Một hàm reset *"trông
 *    như"* dọn một ô mà không gán gì vẫn đi qua Kiểm A.
 * 2. [`bodyOf`] cân dấu ngoặc trên MỌI ký tự `{}()` ⇒ một `)` trong một chuỗi
 *    *(`function reset(msg = ")")`)*, một `}` trong một template, hay một `{` lệch cặp trong
 *    một chú thích **cắt thân hàm sai chỗ**: cắt sớm cho ĐỎ OAN, đọc tràn sang mã kế tiếp cho
 *    **XANH OAN** *(một lượt gán KHÔNG thuộc hàm reset bị tính là bằng chứng)*.
 *
 * ⇒ Hàm này xoá **nội dung** của chú thích · chuỗi · template · regex literal, và **giữ đúng
 * độ dài** *(thay bằng dấu cách, giữ nguyên `\n`)*. Giữ độ dài là điều kiện bắt buộc: `bodyOf`
 * dùng `m.index` của một `matchAll` chạy trên **cùng** chuỗi này, nên một lượt xoá làm lệch
 * chỉ số sẽ cắt thân hàm ở một chỗ khác hẳn.
 *
 * ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì giấu:** phép nhận diện regex literal là một **suy đoán**
 * theo ký tự đứng trước *(khuôn quen của mọi bộ tách JS không có parser)*. Một `/` chia đứng
 * sau một trong các ký tự ấy sẽ bị đọc thành mở regex. Hình dạng đó **không tồn tại** trong
 * `src/**` hôm nay và nó ở chiều ĐỎ OAN, không chiều xanh oan — nên nó là một giới hạn được
 * chọn, không một chỗ bỏ sót. Không thêm một phụ thuộc npm cho một cổng *(NFR15)*.
 *
 * @param {string} src
 * @returns {string} cùng độ dài, cùng số dòng, chỉ còn MÃ
 */
function stripNonCode(src) {
  const out = src.split('')
  const blank = (from, to) => {
    for (let k = from; k < to && k < out.length; k += 1) if (out[k] !== '\n') out[k] = ' '
  }
  // Ký tự có nghĩa gần nhất trước `i` — dùng để phân biệt `/` mở regex với `/` chia.
  const prevCode = (i) => {
    for (let k = i - 1; k >= 0; k -= 1) if (!/\s/.test(out[k])) return out[k]
    return null
  }
  let i = 0
  while (i < src.length) {
    const c = src[i]
    const d = src[i + 1]
    if (c === '/' && d === '/') {
      let j = i + 2
      while (j < src.length && src[j] !== '\n') j += 1
      blank(i, j)
      i = j
    } else if (c === '/' && d === '*') {
      const end = src.indexOf('*/', i + 2)
      const j = end === -1 ? src.length : end + 2
      blank(i, j)
      i = j
    } else if (c === '"' || c === "'" || c === '`') {
      let j = i + 1
      while (j < src.length) {
        if (src[j] === '\\') {
          j += 2
          continue
        }
        if (src[j] === c) break
        // Một chuỗi `'`/`"` không qua được dòng mới — chốt ở đó thay vì nuốt cả tệp.
        if (c !== '`' && src[j] === '\n') break
        j += 1
      }
      blank(i + 1, j)
      i = j + 1
    } else if (c === '/' && '(,=:[!&|?{};+*%^~'.includes(prevCode(i) ?? '(')) {
      let j = i + 1
      let inClass = false
      while (j < src.length && src[j] !== '\n') {
        if (src[j] === '\\') {
          j += 2
          continue
        }
        if (src[j] === '[') inClass = true
        else if (src[j] === ']') inClass = false
        else if (src[j] === '/' && !inClass) break
        j += 1
      }
      blank(i + 1, j)
      i = j + 1
    } else {
      i += 1
    }
  }
  return out.join('')
}

/**
 * Thân của mọi hàm `reset*` trong một tệp, CỘNG thân của mọi hàm cùng tệp được gọi từ đó.
 *
 * ⚠️ Sâu **một tầng** — xem §Giới hạn thật, mục 3, và mục 4 cho ranh giới TỆP.
 * 🔵 2026-08-19: chạy trên [`stripNonCode`], không trên văn bản thô — xem doc-comment ở đó.
 */
function resetScope(rawSource) {
  const source = stripNonCode(rawSource)
  const bodies = []
  const bodyOf = (startIdx) => {
    // 🔴 **KHÔNG lấy dấu `{` đầu tiên** — bẫy đã cắn thật ở lượt dựng cổng này, 2026-08-18:
    // `function datThongBao(o: { confirm?: …; regroup?: …; nav?: … })` mở một `{` **trong
    // kiểu tham số**, và một phép lấy-`{`-đầu-tiên trả về đúng khối kiểu đó rồi coi nó là
    // thân hàm. Hệ quả đo được: `confirmNotice` và `navNotice` bị chấm là *chưa dọn* trong
    // khi `datThongBao` dọn cả ba — một lượt ĐỎ OAN, và nó sẽ được vá bằng hai miễn trừ sai.
    // ⇒ Nhảy qua danh sách tham số bằng cách cân dấu ngoặc TRÒN trước, rồi mới tìm `{`.
    const lparen = source.indexOf('(', startIdx)
    if (lparen === -1) return ''
    let round = 0
    let afterParams = -1
    for (let i = lparen; i < source.length; i += 1) {
      if (source[i] === '(') round += 1
      else if (source[i] === ')') {
        round -= 1
        if (round === 0) {
          afterParams = i + 1
          break
        }
      }
    }
    if (afterParams === -1) return ''
    const open = source.indexOf('{', afterParams)
    if (open === -1) return ''
    let depth = 0
    for (let i = open; i < source.length; i += 1) {
      if (source[i] === '{') depth += 1
      else if (source[i] === '}') {
        depth -= 1
        if (depth === 0) return source.slice(open, i + 1)
      }
    }
    return ''
  }

  const direct = []
  // 🔵 2026-08-19: `(?:async )?` — bản đầu bỏ nó ở ĐÂY nhưng có nó ở vòng tìm hàm phụ ngay
  // dưới, tức hai regex trong cùng một hàm không khai cùng một tập cú pháp.
  for (const m of source.matchAll(/^export (?:async )?function (reset[A-Za-z0-9_$]*)\s*\(/gm)) {
    direct.push(bodyOf(m.index))
  }
  bodies.push(...direct)

  // Tầng một: mọi hàm cùng tệp có tên xuất hiện như một lời gọi trong thân hàm reset.
  const joined = direct.join('\n')
  for (const m of source.matchAll(/^(?:export )?(?:async )?function ([A-Za-z_$][\w$]*)\s*\(/gm)) {
    const name = m[1]
    if (name.startsWith('reset')) continue
    if (new RegExp(`\\b${name}\\s*\\(`).test(joined)) bodies.push(bodyOf(m.index))
  }
  return bodies.join('\n')
}

/**
 * Ô `name` có bị **GÁN** trong `scope` không.
 *
 * 🔴 **Đây là chỗ cổng này suýt trở nên vô nghĩa, và nó bị bắt bằng một ĐỘT BIẾN — ghi ra
 * thay vì để nó trông như một lựa chọn hiển nhiên.**
 *
 * Bản đầu hỏi *"tên có xuất hiện trong thân hàm reset không"* (`\bname\b`). Lượt đột biến
 * của Task 5.5 hoàn nguyên **đúng** hai dòng đã lọt qua chín cổng ở Story 2.8 và 2.9 —
 * `sourceCut.value = null` và `omitError.value = null` — và cổng vẫn **XANH**. Lý do:
 * `resetEditorPanel` gọi mấy hàm phụ, và tầng-một kéo theo thân của chúng, trong đó có
 * những dòng chỉ **ĐỌC** `sourceCut`/`omitError`. Một lượt đọc trở thành bằng chứng cho một
 * lượt dọn.
 *
 * ⇒ Một cổng xanh trên **chính khuyết tật nó sinh ra để bắt**. Nếu lượt đột biến ấy không
 * chạy, cổng đã vào kho ở trạng thái không bao giờ đỏ được.
 *
 * Bốn hình dạng gán được chấp nhận, và chỉ bốn:
 *   `x = …` · `x.value = …` · `x.length = 0` · `x.clear()`
 * Toán tử gán kép (`+=`, `??=`, `||=`) tính; so sánh (`===`, `==`, `!=`) **không** tính.
 */
function isAssignedIn(name, scope) {
  const n = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
  return (
    // `x = …` hoặc `x.value = …` hoặc `x.length = …` — nhưng KHÔNG `x === …`
    new RegExp(`\\b${n}\\b(?:\\.(?:value|length))?\\s*(?:\\+|-|\\*|\\?\\?|\\|\\||&&)?=(?!=)`).test(scope) ||
    // `x.clear()` — hình dạng dọn của `Map`/`Set`
    new RegExp(`\\b${n}\\b\\.clear\\s*\\(`).test(scope)
  )
}

// ═══════════════════════════════════════════════════════════════════════════════════
// KIỂM A — mọi ô nhớ cấp module đi qua một hàm reset, hoặc có miễn trừ CÓ TÊN
// ═══════════════════════════════════════════════════════════════════════════════════
let files = []
try {
  files = walk(SRC_ROOT).sort()
} catch (err) {
  abort('cây nguồn `src/**`', err)
}

/**
 * 🔴 SÀN QUẦN THỂ — *"cây rỗng không phải cây sạch"*.
 *
 * Số THẬT 2026-08-18 (Story 2.12, đo chứ không ước): **39** tệp `.ts` dưới `src/**`.
 * Sàn 33/39 = **84,6 %**, trong dải 80-85 % mà `project-context.md` đặt.
 * ⚠️ Sàn là **cận dưới**: nó không đỏ oan khi thêm tệp, nhưng một sàn cũ là một sàn vô nghĩa
 * — thêm tệp vào `src/**` thì xét lại số này.
 */
const FILE_FLOOR = 36 // 🔵 NÂNG 2026-08-21 (Story 3.4b): số THẬT 44 tệp `.ts` dưới `src/**`
// (+glossaryMarksMap.ts +glossaryMarksState.ts +glossaryTermHoverState.ts) — 36/44 = 81,8%,
// giữa dải 80-85%. Đo bằng `find src -name '*.ts' | wc -l`.
if (files.length < FILE_FLOOR) {
  abort(
    `cây nguồn \`src/**\` — chỉ ${files.length} tệp \`.ts\`, dưới sàn ${FILE_FLOOR}`,
    new Error('Mot danh sach rong lam Kiem A xanh ma khong quet gi ca.'),
  )
}

const seenKeys = new Set()
let slotCount = 0

for (const file of files) {
  let source
  try {
    source = readFileSync(file, 'utf8')
  } catch (err) {
    abort(posix(file), err)
  }
  const rel = posix(file)
  const scope = resetScope(source)
  // 🔵 2026-08-19: quét trên dòng ĐÃ tách chú thích/chuỗi. Bản đầu quét văn bản thô, nên một
  // `const x = ref(0)` nằm ở cột 0 **trong một khối chú thích** đếm thành một ô nhớ có thật.
  const codeLines = stripNonCode(source).split('\n')
  // 🔵 2026-08-19: nhận `export async function reset…` — xem `resetScope`, cùng lý do.
  const hasReset = /^export (?:async )?function reset[A-Za-z0-9_$]*\s*\(/m.test(source)

  codeLines.forEach((line, i) => {
    const ngoaiTap = outOfSubset(line)
    if (ngoaiTap !== null) {
      fail(
        `${rel}:${i + 1} — CU PHAP NGOAI TAP CON: ${ngoaiTap}.\n` +
          `        Cong nhan dien o nho tren MOT dong, mot lan moi dong. Hinh dang nay cho mot\n` +
          `        o nho cap module ma cong KHONG BAO GIO thay — mot xanh oan, khong mot cho\n` +
          `        bo sot. Tach thanh hai dong, hoac them mot mien tru CO TEN kem ly do.`,
      )
      return
    }
    const m = RE_REACTIVE.exec(line) ?? RE_LET.exec(line) ?? RE_EMPTY_BOX.exec(line)
    if (m === null) return
    const name = m[1]
    const key = `${rel}::${name}`
    slotCount += 1
    seenKeys.add(key)

    if (EXEMPT.has(key)) {
      const reason = EXEMPT.get(key)
      if (typeof reason !== 'string' || reason.trim().length === 0) {
        fail(`${key} — mien tru KHONG co ly do. Mot mien tru khong ly do la mot luat da tat.`)
      }
      return
    }
    if (!hasReset) {
      fail(
        `${rel}:${i + 1} \`${name}\` — o nho cap module trong mot tep KHONG co ham \`reset*\` nao.\n` +
          `        Dung mot trong hai: dung mot ham \`export function reset…()\`, hoac them\n` +
          `        "${key}" vao EXEMPT o \`scripts/check-panel-refs.mjs\` KEM LY DO.`,
      )
      return
    }
    if (!isAssignedIn(name, scope)) {
      fail(
        `${rel}:${i + 1} \`${name}\` — KHONG di qua mot ham \`reset*\` nao cua chinh tep do.\n` +
          `        Day dung lop loi ma \`sourceCut\` (2.8) va \`omitError\` (2.9) da lot qua.\n` +
          `        Dung mot trong hai: dat lai no trong ham reset, hoac them "${key}" vao\n` +
          `        EXEMPT KEM LY DO.`,
      )
    }
  })
}

// ═══════════════════════════════════════════════════════════════════════════════════
// KIỂM B — mọi miễn trừ phải CHẾT ĐƯỢC: một miễn trừ trỏ vào một ô không còn tồn tại là ĐỎ
// ═══════════════════════════════════════════════════════════════════════════════════
for (const key of EXEMPT.keys()) {
  if (!seenKeys.has(key)) {
    fail(
      `mien tru "${key}" tro vao mot o KHONG CON TON TAI.\n` +
        `        Mot mien tru het can ma o lai la mot luat da tat ma khong ai biet — go no.`,
    )
  }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// KIỂM C — 🔴 TỰ KIỂM: cổng phải ĐỎ ĐƯỢC, và KHÔNG ĐỎ OAN
// ═══════════════════════════════════════════════════════════════════════════════════
/**
 * *"Một cổng chưa bao giờ đỏ là một cổng chưa ai biết nó có chạy không."*
 *
 * 🔴 Gọi **CHÍNH** [`resetScope`] và **CHÍNH** ba biểu thức chính quy đang chạy thật —
 * không một bản chép. Bài học F4 ② của retro Epic 2: Story 2.9 dựng một bản đồ **chép** hàm
 * sản phẩm, và nó cho `17`/`19` y hệt lượt trước **sau khi đã vá**, tức nó báo *"chưa vá"*
 * trên một sản phẩm ĐÃ VÁ.
 */
function selfCheck() {
  const problems = []
  const slotsOf = (src) => {
    const out = []
    for (const line of src.split('\n')) {
      const m = RE_REACTIVE.exec(line) ?? RE_LET.exec(line) ?? RE_EMPTY_BOX.exec(line)
      if (m !== null) out.push(m[1])
    }
    return out
  }

  // ── Ca DƯƠNG ①: một ô KHÔNG đi qua reset ⇒ phải bị bắt ────────────────────────────
  const duong = [
    "const daDon = shallowRef<number | null>(null)",
    "const chuaDon = shallowRef<number | null>(null)",
    'export function resetVi(): void {',
    '  daDon.value = null',
    '}',
  ].join('\n')
  const scopeDuong = resetScope(duong)
  if (!/\bdaDon\b/.test(scopeDuong)) problems.push('ca duong: `daDon` PHAI bi thay trong than reset')
  if (/\bchuaDon\b/.test(scopeDuong)) {
    problems.push('ca duong: `chuaDon` KHONG di qua reset ma cong lai thay no — cong nay khong do duoc')
  }

  // ── Ca ÂM: tên chỉ GIỐNG, không được tính là đã dọn ───────────────────────────────
  //
  // Đây là đối chứng cho một luật lỏng lẻo: nếu cổng khớp bằng `includes()` thay vì bằng
  // ranh giới từ, thì `caretSegmentId` sẽ "được dọn" bởi một dòng nhắc `caretSegment`.
  const am = ['export function resetVi(): void {', '  caretSegment.value = null', '}'].join('\n')
  if (new RegExp('\\bcaretSegmentId\\b').test(resetScope(am))) {
    problems.push('ca am: `caretSegmentId` KHONG duoc coi la da don chi vi co `caretSegment`')
  }

  // ── Ca DƯƠNG ②: theo lời gọi hàm SÂU MỘT TẦNG (khuôn `clearFlushTimer`) ──────────
  const motTang = [
    'let boHen: number | null = null',
    'function donBoHen(): void {',
    '  boHen = null',
    '}',
    'export function resetVi(): void {',
    '  donBoHen()',
    '}',
  ].join('\n')
  if (!/\bboHen\b/.test(resetScope(motTang))) {
    problems.push('ca duong ②: khong theo duoc mot tang loi goi — khuon `clearFlushTimer` se do oan')
  }

  // ── Ca ÂM ③: một lượt ĐỌC KHÔNG được tính là một lượt dọn ────────────────────────
  //
  // 🔴 Đây là ca khoá đúng lỗ mà lượt đột biến Task 5.5 đã lộ ra. Giữ nó: nếu ai đó nới
  // `isAssignedIn` về lại `\bname\b`, ca này đỏ ngay chứ không chờ một story nữa.
  const chiDoc = [
    'export function resetVi(): void {',
    '  donPhu()',
    '}',
    'function donPhu(): void {',
    '  if (sourceCut.value !== null) console.log(sourceCut.value)', // ĐỌC, không gán
    '  omitError.value = null', // GÁN
    '}',
  ].join('\n')
  const scopeChiDoc = resetScope(chiDoc)
  if (isAssignedIn('sourceCut', scopeChiDoc)) {
    problems.push(
      'ca am ③: `sourceCut` chi duoc DOC ma cong cham la da don — day dung lop loi da lot ' +
        'qua chin cong o Story 2.8 va 2.9',
    )
  }
  if (!isAssignedIn('omitError', scopeChiDoc)) {
    problems.push('ca am ③: `omitError` CO mot luot gan that ma cong khong thay')
  }

  // ── Ca ÂM ②: ba biểu thức chính quy phải bắt ĐÚNG ba hình dạng, không hơn ────────
  const hinhDang = [
    'const a = ref(0)',
    'let b = false',
    'const c = new Map<string, number>()',
    'const D_HANG = { x: 1 }', // bảng hằng — KHÔNG được bắt
    'const e = tinhCaiGiDo()', // lời gọi hàm — KHÔNG được bắt
    '  const f = ref(0)', // thụt lề ⇒ cấp HÀM, KHÔNG cấp module
  ].join('\n')
  const bat = slotsOf(hinhDang)
  const mong = ['a', 'b', 'c']
  if (bat.join(',') !== mong.join(',')) {
    problems.push(`ca am ②: cho [${mong}], bat duoc [${bat}] — hinh dang nhan dien o nho da lech`)
  }

  // ═══════════════════════════════════════════════════════════════════════════════════
  // 🔵 CODE REVIEW BA TẦNG 2026-08-19 — BỐN CA CHO BỐN CƠ CHẾ MỚI
  // ═══════════════════════════════════════════════════════════════════════════════════
  // 🔴 Một cơ chế mới không có phép tự kiểm là một cơ chế **chưa ai biết nó đỏ được** — đúng
  // bài học mà chính cổng này học bằng lượt đột biến Task 5.5. Bốn ca dưới đây gọi **CHÍNH**
  // [`stripNonCode`] · [`outOfSubset`] · [`resetScope`] đang chạy thật, không một bản chép
  // *(bài học F4 ② của retro Epic 2)*.

  // ── Ca ÂM ④: một lượt gán trong CHÚ THÍCH không được tính là đã dọn ──────────────
  const trongChuThich = [
    'export function resetVi(): void {',
    '  // sourceCut.value = null   ← chỉ là một dòng chú thích, KHÔNG một lượt gán',
    '  omitError.value = null',
    '}',
  ].join('\n')
  const scopeChuThich = resetScope(trongChuThich)
  if (isAssignedIn('sourceCut', scopeChuThich)) {
    problems.push(
      'ca am ④: `sourceCut` chi xuat hien trong mot CHU THICH ma cong cham la da don — ' +
        '`stripNonCode` khong chay, hoac no khong xoa chu thich',
    )
  }
  if (!isAssignedIn('omitError', scopeChuThich)) {
    problems.push('ca am ④: `omitError` co mot luot gan THAT ma cong khong thay — da xoa qua tay')
  }

  // ── Ca ÂM ⑤: một `)` trong CHUỖI không được cắt danh sách tham số sớm ───────────
  //
  // Nếu `bodyOf` cân dấu ngoặc trên văn bản thô, `")"` đóng sớm và `open` trỏ vào một `{`
  // khác — thân hàm đọc ra sai chỗ, và `daDon` biến mất.
  const ngoacTrongChuoi = [
    'export function resetVi(msg = ")"): void {',
    '  daDon.value = null',
    '}',
  ].join('\n')
  if (!isAssignedIn('daDon', resetScope(ngoacTrongChuoi))) {
    problems.push(
      'ca am ⑤: mot `)` trong CHUOI da cat danh sach tham so som — `bodyOf` dang can dau ' +
        'ngoac tren van ban tho',
    )
  }

  // ── Ca DƯƠNG ③: `export async function reset…` phải được nhận ───────────────────
  const resetAsync = [
    'export async function resetVi(): Promise<void> {',
    '  daDon.value = null',
    '}',
  ].join('\n')
  if (!isAssignedIn('daDon', resetScope(resetAsync))) {
    problems.push('ca duong ③: `export async function reset…` khong duoc nhan — hai regex lech nhau')
  }
  if (!/^export (?:async )?function reset[A-Za-z0-9_$]*\s*\(/m.test(resetAsync)) {
    problems.push('ca duong ③: regex `hasReset` khong nhan `async` — mot tep nhu the se DO OAN')
  }

  // ── Ca ⑥: tập con cú pháp — hai hình dạng PHẢI đỏ, ba hình dạng KHÔNG được đỏ oan ─
  const phaiDo = [
    'const a = ref(0), b = ref(0)',
    'let x =',
  ]
  for (const line of phaiDo) {
    if (outOfSubset(stripNonCode(line)) === null) {
      problems.push(`ca ⑥: "${line}" nam NGOAI tap con ma cong khong bat — mot o nho vo hinh`)
    }
  }
  const khongDuocDo = [
    'const cache = new Map<string, number>()', // phẩy trong `<>`
    'const BANG = { a: 1, b: 2 }', // phẩy trong `{}`
    'const nhan = ref(0) // ghi chú, b = 1', // phẩy trong chú thích
    "const s = ref('a, b = 1')", // phẩy trong chuỗi
  ]
  for (const line of khongDuocDo) {
    const ly = outOfSubset(stripNonCode(line))
    if (ly !== null) {
      problems.push(`ca ⑥: "${line}" nam TRONG tap con ma cong cham la ngoai ("${ly}") — do oan`)
    }
  }

  return problems
}

const selfProblems = selfCheck()
for (const p of selfProblems) {
  console.error(`  FAIL  TU KIEM: ${p}`)
  failures += 1
}

// ═══════════════════════════════════════════════════════════════════════════════════
// PHÁN QUYẾT — mã thoát, không một dòng log rồi đi tiếp
// ═══════════════════════════════════════════════════════════════════════════════════
if (failures > 0) {
  console.error(
    `\ncheck:panel-refs — ${failures} loi.\n` +
      `Quet ${files.length} tep \`.ts\`, ${slotCount} o nho cap module, ${EXEMPT.size} mien tru co ten.\n`,
  )
  process.exit(1)
}
console.log(
  `check:panel-refs OK — ${files.length} tep \`.ts\`, ${slotCount} o nho cap module, ` +
    `${EXEMPT.size} mien tru co ten, tu kiem xanh.`,
)
