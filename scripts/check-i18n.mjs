#!/usr/bin/env node
/**
 * Cổng chuỗi giao diện của Story 1.5 — cưỡng chế NFR16 và AD-21 bằng lệnh, mã thoát
 * là phán quyết.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * VÌ SAO TỒN TẠI
 * ─────────────────────────────────────────────────────────────────────────────────
 * `deferred-work.md:19` ghi lại nghịch lý: *"NFR16 không có cơ chế cưỡng chế nào —
 * `src/App.vue:5` chỉ có một comment, trong khi thứ khó vi phạm hơn hẳn (lỡ cài
 * `tauri-plugin-fs`) thì có cả script lẫn mã thoát."* Quy tắc này vi phạm chỉ cần gõ
 * một nhãn button. Story 1.14 dựng bốn panel; Epic 3–9 thêm hàng trăm chuỗi. Cổng
 * phải đứng TRƯỚC chúng.
 *
 * ⚠️ Node chứ không bash (Ice chốt 2026-08-03, `check-deps.mjs:22-24`): `npm run` trên
 * Windows đi qua `cmd.exe`, không có bash. Một cổng chỉ canh nửa số nền tảng thì không
 * canh được NFR14.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * NĂM PHÉP KIỂM
 * ─────────────────────────────────────────────────────────────────────────────────
 *   A (AC2) Không ký tự có dấu tiếng Việt ở VỊ TRÍ MÃ của `src-tauri/**\/*.rs` và
 *           `src/**\/*.vue`. Quét CÓ TRẠNG THÁI — xem §Vì sao không `replace`.
 *   B (AC1) `vi.json` PHẲNG; khoá chấm có tiền tố miền; không giá trị rỗng.
 *   C (AC1) Placeholder khớp `{ten_tham_so}` — cùng dải mà `resolve.ts` nội suy.
 *   D (AC5) Giọng văn UX-DR47, phần máy chấm được: vô nhân xưng.
 *   E (AC4) HÀNH VI thật của `resolve.ts` — nạp và gọi hàm, cả hai chiều.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * BA NHÓM VI PHẠM, BA PHÁN QUYẾT KHÁC NHAU — đọc trước khi sửa quy tắc quét
 * ─────────────────────────────────────────────────────────────────────────────────
 * 1. **Comment tiếng Việt ở mọi nơi — KHÔNG phải vi phạm.** Toàn bộ dự án tự tài liệu
 *    hoá bằng tiếng Việt; đó là quy ước có chủ ý. NFR16 nói về CHUỖI HIỂN THỊ, không
 *    nói về comment. Một cổng bắt comment sẽ đỏ vĩnh viễn ở mọi tệp và sẽ bị gỡ
 *    trong tuần — đúng cách hỏng đắt hơn hẳn việc không có cổng (`ci.yml:410-418`).
 * 2. **Thông báo `assert!` trong `src-tauri/tests/**` — miễn trừ, nhưng KHAI RA.**
 *    Xem `EXEMPT` bên dưới.
 * 3. **Chuỗi trong `.vue` — vi phạm thật.** `src/App.vue:38,54` là hai chỗ duy nhất
 *    trong cây nguồn lúc Story 1.5 bắt đầu; Task 5 đã dời chúng sang
 *    `src/selftest/fallbackReport.ts`.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * VÌ SAO KHÔNG `replace(/\/\/.*$/gm, '')` — §Bốn thứ sẽ hỏng im lặng #4
 * ─────────────────────────────────────────────────────────────────────────────────
 * `"https://example.com"` có `//` bên trong dấu nháy. Một lượt bóc comment ngây thơ
 * biến nó thành `"https:` và mọi thứ sau đó lệch. Rust còn có raw string `r#"…"#`
 * (không escape), block comment LỒNG NHAU được, và `.vue` có ba vùng cú pháp khác
 * nhau. Hậu quả nếu làm ẩu đi cả hai chiều: bỏ sót vi phạm thật, VÀ báo động giả trên
 * một URL vô hại.
 *
 * ⚠️ Rủi ro ở đây NGƯỢC với `check-tokens.mjs`. Ở đó trạng thái mặc định là "cho qua"
 * nên một lượt che sai làm lọt vi phạm. Ở đây trạng thái mặc định (`code`/`string`) là
 * **FAIL**, nên chỉ có đúng một đường bỏ sót: nhận nhầm thứ gì đó thành COMMENT. Vì
 * vậy máy trạng thái dưới đây theo dõi chuỗi không phải để chấm chuỗi — nó chấm chuỗi
 * bằng cùng một luật với mã — mà để `//` bên trong chuỗi không mở được một comment giả.
 *
 * Chạy:  npm run check:i18n
 */
import { readFileSync, readdirSync, lstatSync, realpathSync, existsSync } from 'node:fs'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { dirname, join, relative, sep } from 'node:path'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const VI_JSON_PATH = join(REPO_ROOT, 'src', 'i18n', 'vi.json')
const RESOLVE_TS_PATH = join(REPO_ROOT, 'src', 'i18n', 'resolve.ts')

/**
 * Bộ dấu tiếng Việt — 67 chữ thường, và bản hoa sinh ra bằng `toUpperCase()` để hai
 * bản không trôi khỏi nhau khi ai đó thêm một ký tự.
 *
 * ⚠️ Một BỘ TƯỜNG MINH, không phải một dải `à-ỹ`. Dải đó nuốt trọn Hy Lạp,
 * Cyrillic, Do Thái, Ả Rập — và cổng sẽ đỏ trên một comment tiếng Nga hay một đường
 * dẫn test có ký tự Hy Lạp, vì một lý do không có thật. Đây đúng là loại dương tính
 * giả mà `check-tokens.mjs` cảnh báo: *"một cổng chỉ đường sai sẽ bị người sau thêm
 * ngoại lệ cho tới khi nó không bắt được gì"*.
 */
const VI_LOWER = 'àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđ'
const VI_CHARS = new Set([...VI_LOWER, ...VI_LOWER.toUpperCase()])
const isViChar = (ch) => VI_CHARS.has(ch)

/**
 * 🔴 CHUẨN HOÁ NFC TRƯỚC KHI QUÉT — bộ trên là chữ DỰNG SẴN, và đó là nửa câu chuyện.
 *
 * Unicode viết `ư` được hai cách: một điểm mã dựng sẵn (NFC, U+01B0) hoặc `u` + dấu tổ
 * hợp U+031B (NFD). Hai dạng hiện ra màn hình y hệt nhau, và macOS chuẩn hoá theo NFD ở
 * nhiều đường — nên một chuỗi dán từ Finder, từ một tệp cũ, hay từ một trình soạn thảo
 * khác đi qua cổng mà không chạm ký tự nào trong `VI_CHARS`. Đo được: `<button>Lưu</button>`
 * dạng NFD cho exit 0; đúng chuỗi đó dạng NFC cho exit 1.
 *
 * ⚠️ Chuẩn hoá làm CỘT lệch so với tệp gốc ở những dòng có dấu tổ hợp (mỗi dấu tổ hợp
 * gộp lại làm ngắn đi một ký tự). Dòng thì không lệch, và trích đoạn in kèm là thứ người
 * sửa thật sự đọc. Đổi lại là không còn một đường im lặng nào.
 */
const nfc = (s) => s.normalize('NFC')

/** Biên "tiếng" cho Kiểm D — chữ Latin, chữ Việt có dấu, chữ số, gạch dưới. */
const WORD_CHAR = new RegExp(`[0-9A-Za-z_${VI_LOWER}${VI_LOWER.toUpperCase()}]`)

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
// Miễn trừ KHÔNG được cài bằng cách thu hẹp glob quét. Glob quét cả cây; miễn trừ
// là một bước lọc CÓ TÊN và CÓ LÝ DO. Một danh sách kiểm tự rút gọn để cho xanh là
// đúng thứ cổng tồn tại để chặn (cùng luật mà Story 1.4 §Kiểm C áp cho cặp màu không
// dùng). Số tệp miễn trừ in ra để nó không lặng lẽ phình lên.
// ═════════════════════════════════════════════════════════════════════════════════
const EXEMPT = [
  [
    'src-tauri/tests/**',
    'thông báo `assert!` — không vượt IPC, không được render; người đọc chúng là người ' +
      'đang sửa test. Dịch sang tiếng Anh là mất giá trị tài liệu để đổi lấy con số không.',
  ],
  [
    'src/selftest/**',
    'chẩn đoán cho log CI — debug-only, `import()` động, không vào bundle release. ' +
      '`vi.json` là tài nguyên HIỂN THỊ; trộn hai thứ là hỏng chính ranh giới Story 1.5 dựng. ' +
      '⚠️ HÔM NAY KHỚP 0 TỆP và con số đó in ra có chủ ý: thư mục chỉ có `.ts`, mà AC2 ' +
      'phát biểu trên `.rs` và `.vue`. Mục này khai trước cho ngày một `.vue` chẩn đoán ' +
      'xuất hiện ở đó — và để lý do nằm cạnh chỗ cưỡng chế thay vì trong trí nhớ ai đó.',
  ],
  [
    'tools/**',
    'build tool (Story 1.9, `tools/dict-build`) — KHÔNG vào bản phát hành (AD-25), ' +
      'không có bề mặt giao diện để render, và thông báo lỗi của nó là CHẨN ĐOÁN cho ' +
      'người dựng trên máy Ice/CI, không phải chuỗi người dùng cuối thấy. Đóng ' +
      '`deferred-work.md:44` — "gốc quét cứng ở src/ và src-tauri/… mở lại khi cây mọc ' +
      'nhánh thứ ba": `tools/` LÀ nhánh thứ ba, và đây là lượt mở lại đó.',
  ],
]

/** `a/b/**` khớp mọi thứ dưới `a/b/`. Không hỗ trợ `*` giữa đường — không cần. */
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

/**
 * ⚠️ `lstatSync`, KHÔNG `statSync` — cùng bài học với `check-tokens.mjs`: `statSync`
 * giải symlink nên một liên kết trỏ về thư mục cha làm đệ quy không dừng, và một liên
 * kết gãy ném `ENOENT` bị `abort()` báo thành "cây nguồn không đọc được". Symlink bị
 * BỎ QUA và ghi tên ra, để việc bỏ qua không im lặng.
 */
const skippedLinks = []
const SKIP_DIRS = new Set(['target', 'node_modules', 'dist', '.git'])

function walk(dir, ext, out = [], seen = new Set()) {
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
    if (st.isDirectory()) walk(full, ext, out, seen)
    else if (name.toLowerCase().endsWith(ext)) out.push(full)
  }
  return out
}

let rsAll = []
let vueAll = []
try {
  // ⚠️ Quét CẢ `src-tauri/` chứ không chỉ `src-tauri/src/`. AC2 nói `.rs`, không nói
  // "`.rs` dưới `src/`" — và `tests/**` được miễn trừ CÓ TÊN ở `EXEMPT`, không phải
  // bằng một glob lặng lẽ hẹp lại. `target/` bị loại ở `SKIP_DIRS`: nó chứa mã sinh ra
  // và nguồn của crate bên thứ ba, không phải mã của dự án.
  //
  // 🔴 `tools/` là NHÁNH THỨ BA (Story 1.9, Task 9) — đóng `deferred-work.md:44`.
  // Miễn trừ TRỌN ở `EXEMPT` (`tools/**`), nên thêm gốc này KHÔNG được đổi quần thể
  // in ra sau miễn trừ; nếu số nhảy lên, miễn trừ chưa ăn — sửa miễn trừ, đừng chỉnh
  // sàn cho vừa (xem doc-comment `RS_FLOOR` bên dưới).
  rsAll = [...walk(join(REPO_ROOT, 'src-tauri'), '.rs'), ...walk(join(REPO_ROOT, 'tools'), '.rs')].sort()
  vueAll = walk(join(REPO_ROOT, 'src'), '.vue').sort()
} catch (err) {
  abort('cây nguồn (`src-tauri/**`, `tools/**` và `src/**`)', err)
}

const exemptedFiles = []
const keep = (files) =>
  files.filter((f) => {
    const hit = exemptionFor(f)
    if (hit) exemptedFiles.push([posix(f), hit[1]])
    return !hit
  })

const rsFiles = keep(rsAll)
const vueFiles = keep(vueAll)

/**
 * 🔴 NGƯỠNG SÀN, BẮT BUỘC — không phải nice-to-have.
 *
 * `check-deps.mjs:15-17` đã đâm vào đúng bẫy này một lần: *"cây rỗng đọc thành sạch"*.
 * Ở đây tương đương là một glob viết sai (`src/**.vue` thay vì `src/**\/*.vue`) khớp 0
 * tệp ⇒ script in "không tìm thấy vi phạm" ⇒ exit 0 ⇒ cổng chết im lặng ngay ngày nó
 * ra đời. Số thật lúc dựng: **18** tệp `.rs` sau miễn trừ (20 tệp đi qua `walk`, 2 tệp
 * `tests/**` miễn trừ) và 1 tệp `.vue`.
 *
 * ⚠️ Sàn áp lên quần thể SAU miễn trừ — đó mới là quần thể Kiểm A thật sự chạy trên.
 * Đo trước miễn trừ thì một `EXEMPT` phình ra tới mức nuốt cả `src/` vẫn qua sàn.
 *
 * ⚠️ Số cập nhật ở Story 1.7 (tầng ghi dữ liệu): **23** tệp `.rs` sau miễn trừ — 27 tệp
 * đi qua `walk`, 4 tệp `tests/**` miễn trừ. Cây mọc thêm 5 tệp dưới
 * `src-tauri/src/core/store/` và 2 tệp test. Sàn giữ nguyên tỷ lệ dư địa cũ (~78% số
 * thật): nó tồn tại để bắt một cây bị CẮT MẤT, không phải để đếm tệp mới.
 *
 * ⚠️ Số cập nhật ở Story 1.8 (phân giải cấu hình hai tầng): **27** tệp `.rs` sau miễn trừ
 * — 33 tệp đi qua `walk`, 6 tệp `tests/**` miễn trừ. Cây mọc thêm 3 tệp dưới
 * `src-tauri/src/core/scope/`, `src-tauri/src/commands/config.rs`, và 2 tệp test.
 *
 * 🔴 Sàn đặt ở **21** (~78% của 27), **không** đặt bằng 27. Story 1.7 §Completion Notes
 * #10 ghi lại nguyên văn vì sao: *"sàn tồn tại để bắt một cây bị cắt mất, không phải để
 * đếm tệp mới"* — đặt nó bằng số thật là tự tạo một cổng đỏ ở story sau, và cổng đỏ vì
 * một lý do không có thật là cổng bị gỡ.
 *
 * ⚠️ Story 1.9 (dữ liệu từ điển lớp nền) thêm gốc quét `tools/**` (nhánh thứ ba, đóng
 * `deferred-work.md:44`) VÀ miễn trừ nó TRỌN ở `EXEMPT`. Quần thể SAU miễn trừ vì vậy
 * **không đổi** — vẫn 27 tệp `.rs` + 5 tệp `.vue` (đã cập nhật sau Story 1.8; xem lịch
 * sử ở trên). Sàn `RS_FLOOR`/`VUE_FLOOR` giữ nguyên 21/1 — thêm một nhánh MIỄN TRỪ TRỌN
 * không phải lý do dời sàn.
 *
 * 🔴 NÂNG SÀN 2026-08-06 — Story 1.14 · AC11.1, đóng `deferred-work.md:48` và `:146`.
 *
 * Số THẬT sau Story 1.14: **32** tệp `.rs` sau miễn trừ · **11** tệp `.vue`. Quần thể
 * `.vue` nhảy từ 5 lên 11 vì bốn panel + `PanelTab` + `WorkspaceDock` ra đời.
 *
 * `VUE_FLOOR = 1` là con số **không còn canh được gì**: nó đúng ở ngày `PanelFrame` là
 * `.vue` duy nhất, và từ đó tới nay một lượt quét khớp 2 trong 11 tệp vẫn đi qua. Nay
 * nâng lên **9** (~82% của 11), cùng tỷ lệ dư địa mà `RS_FLOOR` đang giữ.
 *
 * ⚠️ `RS_FLOOR` lên **26** (~81% của 32). Nâng vì con số thật đã đi xa khỏi 21 sau các
 * story 1.9–1.13, không phải vì story này thêm tệp `.rs` nào — Story 1.14 thêm đúng
 * **không** tệp Rust mới, nó chỉ sửa hai tệp có sẵn.
 *
 * ⚠️ Và nhắc lại vì nó là lý do sàn này tồn tại ở dạng này: sàn ĐẾM TỆP thì một tệp RỖNG
 * vẫn qua. Sàn nội dung tương ứng của cổng này là Kiểm B (`16` khoá `vi.json`, object
 * phẳng) và Kiểm E (hành vi thật của `resolve.ts`).
 */
// 🔴 NÂNG 2026-08-07 (code review) — cùng lý do `CLICK_FLOOR` của `check-commands.mjs`:
// AC13 gọi đích danh sàn này (*"`RS_FLOOR` **32** vs 39"*) và bản đầu đánh dấu nó *"không
// đổi"* thay vì nâng. 32/40 = 80%, sát mép dưới; 34/40 = 85%, khớp doctrine.
const RS_FLOOR = 34 // số THẬT 2026-08-07: 40 tệp `.rs`
const VUE_FLOOR = 12 // số THẬT 2026-08-10 (sau Story 1.19): 14 tệp `.vue`
if (rsFiles.length < RS_FLOOR || vueFiles.length < VUE_FLOOR) {
  abort(
    `quần thể quét — ${rsFiles.length} tệp \`.rs\` (sàn ${RS_FLOOR}) · ` +
      `${vueFiles.length} tệp \`.vue\` (sàn ${VUE_FLOOR})`,
    new Error(
      'Cây quá nhỏ để là thật. Một danh sách rỗng làm Kiểm A xanh mà không kiểm gì cả.\n' +
        `Đã miễn trừ ${exemptedFiles.length} tệp — kiểm lại danh sách EXEMPT nếu con số đó bất thường.`,
    ),
  )
}

// ═════════════════════════════════════════════════════════════════════════════════
// Máy trạng thái — Rust
// ═════════════════════════════════════════════════════════════════════════════════
//
// Trạng thái nào CHE ký tự: chỉ `line_comment` và `block_comment`. Mọi trạng thái còn
// lại (`code`, `string`, `raw_string`) đều tính là VỊ TRÍ MÃ và ký tự có dấu ở đó là
// FAIL — đúng phát biểu AC2, và cũng là lý do máy này an toàn: sai sót nghiêng về báo
// thừa chứ không về bỏ sót.
//
// 🔴 `'…'` PHẢI được nuốt trọn, và lý do không phải là phán quyết — mà là ĐỒNG BỘ.
//
// Bản đầu bỏ qua `'` với lập luận *"char literal và mã cùng một phán quyết nên không
// cần phân biệt với lifetime"*. Phán quyết thì đúng, đồng bộ thì không: một
// `matches!(c, '"')` — Rust hoàn toàn bình thường, có trong mọi parser CSV/JSON — để
// dấu `"` bên trong mở một STRING MA. Từ đó trở đi máy lệch pha, và một `"/*"` trong
// một chuỗi thật sau đó bị đọc thành block comment không bao giờ đóng ⇒ phần còn lại
// của tệp im lặng. Đo được: `let q = '"'; let s = "/*"; let msg = "Đã lưu";` cho exit 0,
// trong khi riêng `let msg = "Đã lưu";` cho exit 1.
//
// Phân biệt được lifetime bằng HÌNH DẠNG, không bằng ngữ cảnh: một char literal luôn
// đóng bằng `'` ngay sau đúng MỘT ký tự (hoặc một escape). `'a` của `&'a str`, `'static`
// và `Foo<'a, 'b>` đều không khớp. Nội dung bên trong vẫn bị quét — phán quyết giữ
// nguyên như trước, chỉ khác là dấu nháy không còn làm lệch máy.

const IDENT_CHAR = /[A-Za-z0-9_]/

/** `'x'` · `'\n'` · `'\''` · `'\u{1F600}'`. Cờ `u` để một ký tự astral tính là MỘT. */
const CHAR_LIT_RE = /^'(?:\\u\{[0-9a-fA-F]{1,6}\}|\\.|[^'\\])'/u

function scanRust(text) {
  const hits = []
  let i = 0
  let depth = 0
  let hashes = 0
  let state = 'code'

  while (i < text.length) {
    const ch = text[i]
    if (state === 'code') {
      if (text.startsWith('/*', i)) {
        state = 'block_comment'
        depth = 1
        i += 2
        continue
      }
      if (text.startsWith('//', i)) {
        // Bắt cả `///` và `//!` — cùng là comment một dòng.
        state = 'line_comment'
        i += 2
        continue
      }
      // Raw string: `r"…"` · `r#"…"#` · `br##"…"##`. Không escape bên trong, nên chỗ
      // đóng là dấu nháy KÈM ĐÚNG số `#` đã mở.
      const prev = i > 0 ? text[i - 1] : ''
      if ((ch === 'r' || ch === 'b') && !IDENT_CHAR.test(prev)) {
        const m = /^b?r(#*)"/.exec(text.slice(i, i + 40))
        if (m) {
          state = 'raw_string'
          hashes = m[1].length
          i += m[0].length
          continue
        }
      }
      if (ch === '"') {
        state = 'string'
        i += 1
        continue
      }
      // Char literal: nuốt TRỌN, không để dấu nháy bên trong mở string ma. Nội dung vẫn
      // bị quét — `'Đ'` là vị trí mã, đúng phán quyết cũ.
      if (ch === "'") {
        const m = CHAR_LIT_RE.exec(text.slice(i, i + 12))
        if (m) {
          for (let k = i + 1; k < i + m[0].length - 1; k += 1) {
            if (isViChar(text[k])) hits.push(k)
          }
          i += m[0].length
          continue
        }
        // Không khớp ⇒ đây là một lifetime (`&'a str`, `'static`). Đi tiếp như mã.
      }
      if (isViChar(ch)) hits.push(i)
      i += 1
      continue
    }
    if (state === 'line_comment') {
      if (ch === '\n') state = 'code'
      i += 1
      continue
    }
    if (state === 'block_comment') {
      // ⚠️ Block comment của Rust LỒNG NHAU được — `/* /* */ */` là một comment, không
      // phải một comment cộng hai ký tự lạc. Đếm độ sâu, đừng tìm `*/` đầu tiên.
      if (text.startsWith('/*', i)) {
        depth += 1
        i += 2
        continue
      }
      if (text.startsWith('*/', i)) {
        depth -= 1
        i += 2
        if (depth === 0) state = 'code'
        continue
      }
      i += 1
      continue
    }
    if (state === 'string') {
      if (ch === '\\') {
        i += 2
        continue
      }
      if (ch === '"') {
        state = 'code'
        i += 1
        continue
      }
      if (isViChar(ch)) hits.push(i)
      i += 1
      continue
    }
    // raw_string
    if (ch === '"' && text.slice(i + 1, i + 1 + hashes) === '#'.repeat(hashes)) {
      state = 'code'
      i += 1 + hashes
      continue
    }
    if (isViChar(ch)) hits.push(i)
    i += 1
  }
  return hits
}

// ═════════════════════════════════════════════════════════════════════════════════
// Máy trạng thái — Vue (ba vùng cú pháp)
// ═════════════════════════════════════════════════════════════════════════════════

/** `<script>` và `<style>`: JS/CSS. Phần còn lại của tệp là template. */
function vueRegions(text) {
  const regions = []
  const open = /<(script|style)\b[^>]*>/gi
  let m
  while ((m = open.exec(text))) {
    const kind = m[1].toLowerCase()
    const start = m.index + m[0].length
    // ⚠️ Khớp thẻ đóng KHÔNG phân biệt hoa thường và cho phép khoảng trắng — cùng bài
    // học với `check-tokens.mjs`: `indexOf('</style>')` làm `</STYLE>` kéo vùng tới hết
    // tệp và một khối sau đó bị phân tích hai lần.
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
 * Sau ký tự nào thì một `/` mở REGEX chứ không phải phép chia.
 *
 * Cùng luật mà mọi lexer JS dùng: `/` sau một *toán tử* hoặc một *dấu mở* là regex;
 * sau một *giá trị* (định danh, số, `)`, `]`) là phép chia. `}` xếp vào nhóm regex —
 * `}` kết thúc một khối, và `a = b} / c` không phải JS hợp lệ.
 */
const REGEX_PRECEDERS = new Set([...'([{,;:=!&|?+-*%^~<>'])

/** Từ khoá mà sau nó `/` chắc chắn mở regex: `return /x/`, `typeof /x/`. */
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

/**
 * Vùng `<script>` — JS/TS.
 *
 * Template literal `` `…${expr}…` `` phải quay về trạng thái mã bên trong `${}`, nếu
 * không thì một `//` trong biểu thức mở một comment giả kéo dài tới hết dòng.
 *
 * 🔴 REGEX LITERAL là trạng thái thứ hai mà một lượt cài đặt vội bỏ sót, và nó hỏng
 * theo chiều ĐẮT NHẤT — báo sót, không báo thừa. `/^https?:\/\//` chứa `\/` rồi `/`;
 * một máy không biết regex đọc cặp đó thành `//` và che nốt dòng. Đo được:
 * `const _re = /^https?:\/\//; const _nhan = 'Đã lưu'` cho exit 0 trước khi có trạng
 * thái này. Header của tệp lập luận `//` trong string không đánh lừa được máy — đúng,
 * nhưng regex literal mở lại đúng cái lỗ đó bằng một cửa khác.
 */
function scanScript(text, from, to, hits) {
  let i = from
  let state = 'code'
  let quote = ''
  /** Ký tự có nghĩa gần nhất ở trạng thái mã — thứ quyết định `/` là regex hay phép chia. */
  let lastSig = ''
  /** Ngăn xếp `${}`: mỗi mục giữ độ sâu ngoặc nhọn của một biểu thức đang mở. */
  const interp = []

  while (i < to) {
    const ch = text[i]
    if (state === 'code') {
      if (text.startsWith('/*', i)) {
        state = 'block_comment'
        i += 2
        continue
      }
      if (text.startsWith('//', i)) {
        state = 'line_comment'
        i += 2
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
      // Regex literal. `//` và `/*` đã bị bắt ở trên, nên tới đây `/` chỉ còn hai nghĩa.
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
          // Regex không vượt dòng. Gặp `\n` trước khi đóng ⇒ đoán sai, đây là phép chia.
          if (c === '\n') break
          if (c === '[') inClass = true
          else if (c === ']') inClass = false
          else if (c === '/' && !inClass) {
            closed = true
            break
          }
          if (isViChar(c)) hits.push(j)
          j += 1
        }
        if (closed) {
          j += 1
          while (j < to && /[a-z]/.test(text[j])) j += 1
          lastSig = '/'
          i = j
          continue
        }
        // Không đóng được ⇒ bỏ mọi hit đã gom trong đoạn đoán nhầm, xử như phép chia.
        while (hits.length && hits[hits.length - 1] > i) hits.pop()
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
      if (isViChar(ch)) hits.push(i)
      if (!/\s/.test(ch)) lastSig = ch
      i += 1
      continue
    }
    if (state === 'line_comment') {
      if (ch === '\n') state = 'code'
      i += 1
      continue
    }
    if (state === 'block_comment') {
      if (text.startsWith('*/', i)) {
        state = 'code'
        i += 2
        continue
      }
      i += 1
      continue
    }
    if (state === 'string') {
      if (ch === '\\') {
        i += 2
        continue
      }
      // Một dấu nháy lẻ phải đóng TRONG CÙNG MỘT DÒNG — đúng ngữ nghĩa JS. Không có
      // luật này thì một dấu nháy trong văn xuôi (`don't`) nuốt phần còn lại của tệp.
      if (ch === '\n' || ch === quote) {
        state = 'code'
        // Một chuỗi vừa đóng là một GIÁ TRỊ ⇒ `/` ngay sau đó là phép chia, không phải regex.
        lastSig = quote
        i += 1
        continue
      }
      if (isViChar(ch)) hits.push(i)
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
    if (isViChar(ch)) hits.push(i)
    i += 1
  }
}

/** Vùng `<style>` — CSS. ⚠️ `//` KHÔNG phải comment: `url(//host/x.png)` là một URL. */
function scanStyle(text, from, to, hits) {
  let i = from
  let state = 'code'
  let quote = ''
  while (i < to) {
    const ch = text[i]
    if (state === 'code') {
      if (text.startsWith('/*', i)) {
        state = 'block_comment'
        i += 2
        continue
      }
      if (ch === '"' || ch === "'") {
        state = 'string'
        quote = ch
        i += 1
        continue
      }
      if (isViChar(ch)) hits.push(i)
      i += 1
      continue
    }
    if (state === 'block_comment') {
      if (text.startsWith('*/', i)) {
        state = 'code'
        i += 2
        continue
      }
      i += 1
      continue
    }
    if (ch === '\\') {
      i += 2
      continue
    }
    if (ch === '\n' || ch === quote) {
      state = 'code'
      i += 1
      continue
    }
    if (isViChar(ch)) hits.push(i)
    i += 1
  }
}

/**
 * Vùng template.
 *
 * 🔴 **`vue_template_text` là vế mà một lượt cài đặt vội sẽ bỏ sót.**
 * `<button>Lưu</button>` KHÔNG có dấu nháy nào — nó là vi phạm nặng nhất của AC2 và
 * một script chỉ soi string literal sẽ không thấy. Text node giữa hai thẻ phải bị quét
 * NHƯ MÃ.
 *
 * ⚠️ Một chỗ mạnh hơn khung ở story, ghi thẳng ra vì nó là một quyết định: nội dung
 * trong `{{ }}` cũng bị quét, không được cho qua. Story liệt kê `{{ }}` cùng nhóm với
 * "khoảng trắng, chữ số, dấu câu ASCII" — tức nhóm *không phải ký tự có dấu*, nên cho
 * qua hay không cho qua là như nhau ở mọi ca lành. Nhưng ở ca hỏng thì khác hẳn:
 * `{{ 'Đã lưu' }}` và `const x = 'Đã lưu'` là CÙNG một vi phạm trong CÙNG một tệp, và
 * một cổng bắt cái sau mà tha cái trước là một lỗ hổng mời gọi. Trong template chỉ
 * `<!-- -->` được che.
 *
 * 🔴 `<!--` KHÔNG mở comment ở mọi vị trí — chỉ ở VÙNG VĂN BẢN. Bản đầu tìm `<!--`
 * bằng một phép so chuỗi trần, nên `<div title="a <!-- b">` — HTML hợp lệ — mở một
 * comment không có thật và làm mù phần còn lại của vùng. Đo được:
 * `<div title="a <!-- b"></div>` + `<button>Lưu</button>` cho exit 0.
 * Kèm theo, `indexOf('-->')` không bị chặn bởi `to`, nên một `-->` nằm mãi trong
 * `<script>` phía sau cũng nuốt được text node. Ba trạng thái đóng cả hai đường:
 * `text` (quét) · `tag` (quét, `"`/`'` mở giá trị attribute) · `comment` (che, bị chặn
 * bởi `to`). Giá trị attribute VẪN bị quét: `title="Đã lưu"` là vi phạm AC2 y hệt.
 */
function scanTemplate(text, from, to, hits) {
  let i = from
  let state = 'text'
  let quote = ''
  while (i < to) {
    const ch = text[i]
    if (state === 'text') {
      if (text.startsWith('<!--', i)) {
        const end = text.indexOf('-->', i + 4)
        // Chặn bởi `to`: một `-->` ngoài vùng không đóng được comment trong vùng.
        i = end === -1 || end >= to ? to : end + 3
        continue
      }
      // `<` mở thẻ chỉ khi theo sau là tên thẻ hoặc `/`. `a < b` trong `{{ }}` thì không.
      if (ch === '<' && /[A-Za-z/]/.test(text[i + 1] ?? '')) {
        state = 'tag'
        i += 1
        continue
      }
      if (isViChar(ch)) hits.push(i)
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
      if (isViChar(ch)) hits.push(i)
      i += 1
      continue
    }
    // attr — giá trị attribute, quét như mã và kết thúc ở dấu nháy đóng.
    if (ch === quote) {
      state = 'tag'
      i += 1
      continue
    }
    if (isViChar(ch)) hits.push(i)
    i += 1
  }
}

/**
 * Thu **text node** của template — nguyên liệu của Kiểm A2 (Story 1.14 · §Quyết định #6).
 *
 * ⚠️ Cùng máy trạng thái với [`scanTemplate`], khác đúng một chỗ: nó ghi lại **đoạn văn
 * bản giữa hai thẻ** thay vì ghi ký tự có dấu. Dùng chung máy trạng thái là chủ ý — hai
 * bản chép sẽ lệch nhau ở lần sửa thứ ba, và lúc đó Kiểm A với Kiểm A2 nhìn hai template
 * khác nhau trong cùng một tệp.
 *
 * Giá trị attribute KHÔNG được thu: `title="Đã lưu"` đã là vi phạm của **Kiểm A**
 * (chuỗi có dấu ở vị trí mã). Thu nó lần nữa ở đây là báo một lỗi hai lần.
 */
function collectTextNodes(text, from, to, out) {
  let i = from
  let state = 'text'
  let quote = ''
  let runStart = from
  const flush = (end) => {
    if (end > runStart) out.push({ start: runStart, raw: text.slice(runStart, end) })
  }
  while (i < to) {
    const ch = text[i]
    if (state === 'text') {
      if (text.startsWith('<!--', i)) {
        flush(i)
        const end = text.indexOf('-->', i + 4)
        i = end === -1 || end >= to ? to : end + 3
        runStart = i
        continue
      }
      if (ch === '<' && /[A-Za-z/]/.test(text[i + 1] ?? '')) {
        flush(i)
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
        runStart = i
        continue
      }
      i += 1
      continue
    }
    if (ch === quote) state = 'tag'
    i += 1
  }
  if (state === 'text') flush(to)
}

function textNodesOf(text) {
  const out = []
  const regions = vueRegions(text)
  let cursor = 0
  for (const r of regions) {
    if (r.start > cursor) collectTextNodes(text, cursor, r.start, out)
    cursor = r.end
  }
  if (cursor < text.length) collectTextNodes(text, cursor, text.length, out)
  return out
}

function scanVue(text) {
  const hits = []
  const regions = vueRegions(text)
  let cursor = 0
  for (const r of regions) {
    if (r.start > cursor) scanTemplate(text, cursor, r.start, hits)
    if (r.kind === 'script') scanScript(text, r.start, r.end, hits)
    else scanStyle(text, r.start, r.end, hits)
    cursor = r.end
  }
  if (cursor < text.length) scanTemplate(text, cursor, text.length, hits)
  return hits.sort((a, b) => a - b)
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm A — không chuỗi tiếng Việt ở vị trí mã trong `.rs` và `.vue` (AC2)')
// ═════════════════════════════════════════════════════════════════════════════════

const positionOf = (text, index) => {
  const before = text.slice(0, index)
  const line = before.split('\n').length
  const col = index - (before.lastIndexOf('\n') + 1) + 1
  return { line, col }
}

/** Trích 60 ký tự quanh chỗ vi phạm, một dòng, để chẩn đoán đọc được ngay ở log CI. */
const excerpt = (text, index) => {
  const start = Math.max(0, index - 30)
  return text
    .slice(start, Math.min(text.length, index + 30))
    .replace(/\s+/g, ' ')
    .trim()
}

let aBad = 0
for (const [files, scan, label] of [
  [rsFiles, scanRust, '.rs'],
  [vueFiles, scanVue, '.vue'],
]) {
  for (const file of files) {
    let text
    try {
      text = nfc(readFileSync(file, 'utf8'))
    } catch (err) {
      abort(`tệp \`${posix(file)}\``, err)
    }
    const hits = scan(text)
    if (!hits.length) continue
    // Gom theo dòng: một câu tiếng Việt là hàng chục ký tự có dấu, in từng ký tự thì
    // log CI thành vô dụng.
    const byLine = new Map()
    for (const idx of hits) {
      const { line, col } = positionOf(text, idx)
      if (!byLine.has(line)) byLine.set(line, { col, idx })
    }
    for (const [line, { col, idx }] of byLine) {
      fail(`${posix(file)}:${line}:${col} — chuỗi tiếng Việt ở vị trí mã (${label})`)
      detail(`… ${excerpt(text, idx)} …`)
      aBad += 1
    }
  }
}

if (aBad === 0) {
  pass(
    `${rsFiles.length} tệp \`.rs\` + ${vueFiles.length} tệp \`.vue\` — không chuỗi hiển thị nào ở vị trí mã`,
  )
}
detail(`đã miễn trừ ${exemptedFiles.length} tệp:`)
for (const [pattern, why] of EXEMPT) {
  const n = exemptedFiles.filter(([, p]) => p === pattern).length
  detail(`  ${pattern} — ${n} tệp · ${why}`)
}
if (skippedLinks.length) detail(`symlink bỏ qua: ${skippedLinks.join(', ')}`)

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm A2 — mọi TEXT NODE của template phải đi qua `t()` (NFR16)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 ĐÓNG `deferred-work.md:36` — Story 1.14 · §Quyết định #6.
//
// Kiểm A đo **DẤU**, không đo **CHUỖI HIỂN THỊ**. Hệ quả đã ghi nguyên văn từ Story 1.5:
// `<button>Dong</button>` — một nhãn tiếng Việt viết không dấu, hiển thị ra màn hình,
// không đi qua `vi.json` — **đi qua Kiểm A xanh**. Cùng với nó: `<span>Save</span>`,
// `<p>3 results</p>`, mọi thứ không mang dấu.
//
// Ice chốt 2026-08-04: *"giữ nguyên cổng, không mở rộng phạm vi trong Story 1.5 […]
// **Mở lại ở Story 1.14**, khi bốn panel thật có nhãn thật để định nghĩa 'đúng' nghĩa là
// gì."* Nay chúng có nhãn thật, và "đúng" là: **mọi văn bản người dùng đọc được đến từ
// `vi.json`** (NFR16, AD-21).
//
// ─────────────────────────────────────────────────────────────────────────────────
// LUẬT
// ─────────────────────────────────────────────────────────────────────────────────
// Một text node HỢP LỆ khi, sau khi gỡ hết các khối `{{ … }}`, phần còn lại không có
// chữ cái hay chữ số — VÀ mọi khối `{{ … }}` đã gỡ đều mở đầu bằng `t(` hoặc `tError(`.
//
// ⚠️ Phần "còn lại không có chữ cái" cho phép dấu phân cách thị giác (`·`, `|`, `—`,
// dấu phẩy) đứng giữa hai lời gọi — chúng là **hình dạng**, không phải văn bản dịch
// được. Một chữ cái duy nhất lọt vào đó thì đã là một chuỗi hiển thị.
//
// Đường ra là **miễn trừ CÓ TÊN** `<!-- aura-allow-text: <lý do> -->` ngay trên node.
// ⚠️ Mọi miễn trừ được IN RA ở mỗi lượt chạy — cùng kỷ luật với `EXEMPT` ở đầu tệp: một
// miễn trừ không được soi là một chỗ mù.

const INTERPOLATION_RE = /\{\{([\s\S]*?)\}\}/g
const ALLOWED_CALL_RE = /^\s*(?:t|tError)\s*\(/
const HAS_WORD_RE = /[\p{L}\p{N}]/u

let a2Bad = 0
let a2Checked = 0
let a2Exempt = 0
const a2Files = new Set()

for (const file of vueFiles) {
  let text
  try {
    text = readFileSync(file, 'utf8')
  } catch (err) {
    abort(`tệp \`${posix(file)}\``, err)
  }
  for (const node of textNodesOf(text)) {
    // Gỡ các khối `{{ … }}` và giữ lại biểu thức để soi riêng.
    const exprs = []
    const rest = node.raw.replace(new RegExp(INTERPOLATION_RE.source, 'g'), (_, e) => {
      exprs.push(e)
      return ' '
    })
    const strayText = HAS_WORD_RE.test(rest)
    const badCalls = exprs.filter((e) => !ALLOWED_CALL_RE.test(e))
    if (!strayText && badCalls.length === 0) {
      if (exprs.length > 0) a2Checked += 1
      continue
    }
    a2Checked += 1
    const { line, col } = positionOf(text, node.start)
    /**
     * Miễn trừ có tên: comment HTML **đứng ngay trước** node.
     *
     * ⚠️ *"Ngay trước"* đo bằng CẤU TRÚC, không bằng SỐ DÒNG. Bản đầu nhìn lại hai dòng
     * và trượt ngay ở ca đầu tiên gặp thật (`src/App.vue`): một comment giải thích tử tế
     * chiếm sáu dòng, và cửa sổ hai dòng không với tới `aura-allow-text`. Một cổng có
     * đường thoát mà đường đó không dùng được thì đường thoát chỉ là trang trí.
     *
     * Luật: từ đầu node lùi ngược, bỏ qua khoảng trắng **và tối đa MỘT thẻ mở**; nếu chạm
     * `-->` thì lùi tới `<!--` khớp và soi TRỌN comment đó, dài bao nhiêu cũng được.
     *
     * ⚠️ *"tối đa một thẻ mở"* là vế thứ hai mà lượt dựng bỏ sót: comment giải thích đứng
     * trước **phần tử** (`<!-- … --> <p …>{{ x }}</p>`), còn text node thì bắt đầu sau
     * `>`. Bỏ vế này thì miễn trừ chỉ dùng được khi comment nằm BÊN TRONG thẻ — một chỗ
     * không ai viết comment cả.
     */
    let cut = node.start
    const skipSpace = () => {
      while (cut > 0 && /\s/.test(text[cut - 1])) cut -= 1
    }
    skipSpace()
    if (text[cut - 1] === '>' && !text.startsWith('-->', cut - 3)) {
      const open = text.lastIndexOf('<', cut - 1)
      if (open !== -1) {
        cut = open
        skipSpace()
      }
    }
    let window = ''
    if (text.startsWith('-->', cut - 3)) {
      const open = text.lastIndexOf('<!--', cut)
      if (open !== -1) window = text.slice(open, cut)
    }
    if (/aura-allow-text\s*:\s*\S/.test(window)) {
      pass(`${posix(file)}:${line}:${col} — text node có miễn trừ có tên`)
      a2Exempt += 1
      continue
    }
    a2Files.add(posix(file))
    if (strayText) {
      fail(`${posix(file)}:${line}:${col} — văn bản KHÔNG đi qua \`t()\`: ${JSON.stringify(rest.trim().slice(0, 60))}`)
      detail('NFR16: mọi văn bản hiển thị sống ở `src/i18n/vi.json` và CHỈ ở đó.')
    }
    for (const e of badCalls) {
      fail(`${posix(file)}:${line}:${col} — \`{{${e.trim().slice(0, 50)}}}\` không phải \`t()\`/\`tError()\``)
      detail('Nếu biểu thức này ĐÃ mang chuỗi đã dịch: `<!-- aura-allow-text: <lý do> -->`.')
    }
    a2Bad += 1
  }
}
if (a2Bad === 0) {
  pass(
    `${a2Checked} text node mang nội dung trên ${vueFiles.length} tệp \`.vue\` — tất cả đi qua ` +
      `\`t()\`/\`tError()\` (${a2Exempt} miễn trừ có tên)`,
  )
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm B — `vi.json` phẳng, khoá chấm có tiền tố miền (AC1)')
// ═════════════════════════════════════════════════════════════════════════════════

let catalog
let catalogRaw = ''
try {
  catalogRaw = readFileSync(VI_JSON_PATH, 'utf8')
  catalog = JSON.parse(catalogRaw)
} catch (err) {
  // ⚠️ `abort()`, không phải `fail()`. Một `vi.json` không parse được là lỗi hạ tầng
  // của cả bốn phép kiểm còn lại, không phải một phép kiểm đỏ.
  abort(`\`${posix(VI_JSON_PATH)}\``, err)
}
if (!catalog || typeof catalog !== 'object' || Array.isArray(catalog)) {
  abort(`\`${posix(VI_JSON_PATH)}\``, new Error('Gốc tệp phải là một object.'))
}

/** ≥ 1 dấu chấm bắt buộc: khoá phải có tiền tố miền (`err.…`, `lookup.…`). */
const KEY_RE = /^[a-z0-9]+(\.[a-z0-9_]+)+$/
const entries = Object.entries(catalog)
let bBad = 0

/**
 * 🔴 TRÙNG KHOÁ — thứ mà `JSON.parse` nuốt im lặng, và cả hai cổng cùng mù.
 *
 * `{"err.unknown": "A", …, "err.unknown": "B"}` là JSON hợp lệ; `JSON.parse` giữ lần
 * xuất hiện CUỐI và `serde_json` vào `BTreeMap` phía Rust cũng vậy. Một chuỗi đã soạn,
 * đã nghiệm thu theo năm quy tắc UX-DR47, biến mất mà không một dòng chẩn đoán nào.
 * Phía Rust đã có `message_key_catalog_has_no_duplicate_keys` canh đúng lỗ này cho
 * danh mục `MessageKey`; phía `vi.json` thì chưa — nên phải đọc VĂN BẢN THÔ, không đọc
 * kết quả `parse`.
 *
 * Bóc khoá bằng một lượt quét có trạng thái chuỗi (escape `\"` không được đọc thành
 * dấu đóng), chỉ lấy ở độ sâu 1 — cùng lý lẽ với Kiểm A: một `replace` trần trên văn
 * bản có chuỗi bên trong là sai từ đầu.
 */
function topLevelKeys(raw) {
  const keys = []
  let i = 0
  let depth = 0
  while (i < raw.length) {
    const ch = raw[i]
    if (ch === '"') {
      let j = i + 1
      while (j < raw.length) {
        if (raw[j] === '\\') {
          j += 2
          continue
        }
        if (raw[j] === '"') break
        j += 1
      }
      let k = j + 1
      while (k < raw.length && /\s/.test(raw[k])) k += 1
      if (depth === 1 && raw[k] === ':') keys.push(raw.slice(i + 1, j))
      i = j + 1
      continue
    }
    if (ch === '{' || ch === '[') depth += 1
    else if (ch === '}' || ch === ']') depth -= 1
    i += 1
  }
  return keys
}

const rawKeys = topLevelKeys(catalogRaw)
const seenKeys = new Set()
for (const k of rawKeys) {
  if (seenKeys.has(k)) {
    fail(`\`${k}\` khai TRÙNG trong \`vi.json\` — \`JSON.parse\` giữ bản cuối, bản trước biến mất`)
    detail('Một chuỗi đã soạn và đã nghiệm thu bị nuốt im lặng. Gộp hai dòng thành một.')
    bBad += 1
  }
  seenKeys.add(k)
}

for (const [key, value] of entries) {
  if (typeof value !== 'string') {
    const kind = value === null ? 'null' : Array.isArray(value) ? 'mảng' : typeof value
    fail(`\`${key}\` có giá trị kiểu ${kind} — \`vi.json\` phải PHẲNG, mọi giá trị là chuỗi`)
    detail('không `{"lookup": {"empty_result": "…"}}` là sai hình dạng. Khoá chấm, một tầng.')
    bBad += 1
    continue
  }
  if (!KEY_RE.test(key)) {
    fail(`khoá \`${key}\` sai hình dạng — phải khớp \`${KEY_RE.source}\``)
    detail('Chữ thường, gạch dưới, và BẮT BUỘC có tiền tố miền: `err.io.read_failed`.')
    bBad += 1
  }
  if (value.trim() === '') {
    fail(`\`${key}\` có giá trị rỗng — một khoá không có chuỗi là một khoá chưa viết xong`)
    bBad += 1
  }
}

if (entries.length === 0) {
  // Cùng luật ngưỡng sàn: một danh mục rỗng làm Kiểm B, C và D xanh mà không kiểm gì.
  fail('`vi.json` không có khoá nào — Kiểm B, C và D sẽ xanh rỗng')
  bBad += 1
} else if (bBad === 0) {
  pass(`${entries.length} khoá, object phẳng, mọi giá trị là chuỗi không rỗng`)
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm C — placeholder khớp dải mà `resolve.ts` nội suy (AC1)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// Cùng dải với `PLACEHOLDER_RE` của `src/i18n/resolve.ts`. Một placeholder ngoài dải
// KHÔNG ném lúc chạy — nó lặng lẽ không bao giờ khớp, và `{Path}` đi thẳng ra màn hình
// người dùng. Đó là lý do phép kiểm này ở cổng chứ không ở lúc chạy.

const PLACEHOLDER_NAME_RE = /^[a-z_][a-z0-9_]*$/
let cBad = 0
let placeholderCount = 0

for (const [key, value] of entries) {
  if (typeof value !== 'string') continue
  for (const m of value.matchAll(/\{([^{}]*)\}/g)) {
    placeholderCount += 1
    if (!PLACEHOLDER_NAME_RE.test(m[1])) {
      fail(`\`${key}\` — placeholder \`${m[0]}\` ngoài dải \`{ten_tham_so}\``)
      detail('Chữ thường, bắt đầu bằng chữ hoặc `_`. Bắt được: `{}`, `{Path}`, `{0}`, `{ path }`.')
      cBad += 1
    }
  }
  /**
   * ⚠️ ĐẾM NGOẶC CÂN BẰNG LÀ MỘT PHÉP KIỂM SAI, và nó sai theo cả hai chiều.
   * `"Xong } roi {"` có 1 `{` và 1 `}` nên cân bằng, mà `matchAll` không khớp gì ⇒ hai
   * ngoặc thô đi thẳng ra màn hình. `"{{path}}"` cũng cân bằng, `{path}` bên trong hợp
   * lệ ⇒ xanh, rồi `resolve.ts` in ra `{/tmp/a.txt}` với ngoặc thừa hai bên.
   * Phép kiểm đúng: BÓC hết placeholder hợp lệ, rồi phần dư không được còn ngoặc nào.
   */
  const leftover = value.replace(/\{[a-z_][a-z0-9_]*\}/g, '')
  if (leftover.includes('{') || leftover.includes('}')) {
    fail(`\`${key}\` còn ngoặc nhọn THỪA sau khi bóc placeholder hợp lệ`)
    detail(`phần dư: "${leftover}" — \`resolve.ts\` không nội suy chúng, chúng ra màn hình nguyên văn.`)
    cBad += 1
  }
}
if (cBad === 0) pass(`${placeholderCount} placeholder, tất cả khớp \`{ten_tham_so}\``)

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm D — giọng văn UX-DR47, phần máy chấm được (AC5)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// Hai vế của AC5 máy kiểm được: câu viết ở dạng VÔ NHÂN XƯNG — không xưng "chúng tôi",
// không gọi người dùng là "bạn". Ba quy tắc còn lại của UX-DR47 (*nói việc không nói
// cảm xúc · nêu hệ quả · số liệu là số liệu*) là quyết định biên tập, nghiệm thu bằng
// mắt và ghi vào Completion Notes của story.
//
// ⚠️ So theo BIÊN TIẾNG, không phải substring: "bạn" là vi phạm, "bạn bè" cũng vậy,
// nhưng "bạng" hay "hoạbạn" thì không phải chuyện của phép kiểm này.

const BANNED_WORDS = ['chúng tôi', 'bạn']

/** Ngoại lệ khai tường minh, mỗi mục một dòng lý do. Mặc định RỖNG. */
const VOICE_EXCEPTIONS = []

/**
 * ⚠️ So bằng REGEX `\s+` giữa các tiếng, không bằng `indexOf` một chuỗi liền.
 * `"Chúng  tôi không đọc được tệp."` — hai dấu cách, hoặc một lần xuống dòng giữa hai
 * tiếng — lọt qua `indexOf('chúng tôi')` sạch sẽ. Người soạn không cố tình; nó là thứ
 * xảy ra khi một câu được gói lại hay dán vào từ nơi khác.
 * Chuẩn hoá NFC cùng lý do như Kiểm A: `'bạn'` dạng NFD không khớp bộ dựng sẵn.
 */
const BANNED_RES = BANNED_WORDS.map((w) => [
  w,
  new RegExp(w.split(/\s+/).map((p) => p.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('\\s+'), 'g'),
])

let dBad = 0
for (const [key, value] of entries) {
  if (typeof value !== 'string') continue
  if (VOICE_EXCEPTIONS.some(([k]) => k === key)) continue
  const low = nfc(value).toLowerCase()
  for (const [word, re] of BANNED_RES) {
    re.lastIndex = 0
    let m
    while ((m = re.exec(low))) {
      const at = m.index
      const before = at > 0 ? low[at - 1] : ''
      const after = at + m[0].length < low.length ? low[at + m[0].length] : ''
      const isWord = (c) => c !== '' && WORD_CHAR.test(c)
      if (!isWord(before) && !isWord(after)) {
        fail(`\`${key}\` xưng hô: "${word}" — UX-DR47 đòi câu VÔ NHÂN XƯNG`)
        detail(`giá trị: "${value}"`)
        detail('Ví dụ đúng: "Không đọc được tệp tại {path}." · Sai: "Bạn đã chọn một tệp không đọc được."')
        dBad += 1
      }
    }
  }
}
if (dBad === 0) {
  pass(
    `${entries.length} chuỗi, không "chúng tôi", không "bạn"` +
      (VOICE_EXCEPTIONS.length ? ` (${VOICE_EXCEPTIONS.length} ngoại lệ đã khai)` : ''),
  )
}
for (const [key, why] of VOICE_EXCEPTIONS) detail(`ngoại lệ: ${key} — ${why}`)

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm E — hành vi thật của `resolve.ts`, cả hai chiều (AC4)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// Đây là phép kiểm duy nhất GỌI HÀM THẬT. AC4 là mệnh đề về hành vi lúc chạy, và một
// phép kiểm chỉ đọc tệp không nghiệm thu được nó.
//
// ⚠️ Đường đi này tồn tại nhờ Node ≥ 22.18 bóc kiểu TypeScript mặc định — và đó chính
// là lý do `resolve.ts` KHÔNG được `import` gì (doc-comment ở đầu tệp đó ghi đầy đủ).
//
// `import()` thất bại ⇒ `abort()` và exit 1, KHÔNG phải bỏ qua Kiểm E rồi exit 0.
// `check-deps.mjs:60-66`: *"Lỗi hạ tầng ≠ phép kiểm đỏ. Dừng ngay, đừng báo cáo một
// kết quả không có thật."*

if (!existsSync(RESOLVE_TS_PATH)) {
  abort(`\`${posix(RESOLVE_TS_PATH)}\``, new Error('Tệp không tồn tại — Kiểm E KHÔNG chạy được.'))
}

let createResolver
try {
  ;({ createResolver } = await import(pathToFileURL(RESOLVE_TS_PATH).href))
} catch (err) {
  abort(
    `\`${posix(RESOLVE_TS_PATH)}\` — Kiểm E KHÔNG chạy được`,
    new Error(
      `${err?.message || err}\n\n` +
        `Node đang chạy: ${process.version}. Kiểm E cần Node ≥ 22.18 (bóc kiểu TypeScript ` +
        'mặc định), và `resolve.ts` phải là cú pháp "erasable-only": không `enum`, ' +
        'không `namespace`, không parameter property, KHÔNG một dòng `import` nào.',
    ),
  )
}
if (typeof createResolver !== 'function') {
  abort(
    `\`${posix(RESOLVE_TS_PATH)}\``,
    new Error('không export `createResolver` — Kiểm E KHÔNG chạy được.'),
  )
}

/** Bắt `console.warn` để khẳng định AC4 có GHI CẢNH BÁO, không chỉ "không sập". */
const warnings = []
const realWarn = console.warn
console.warn = (...args) => warnings.push(args.join(' '))

let eBad = 0

/**
 * 🔴 MỌI lời gọi `t()` đi qua đây, và đó là hệ quả trực tiếp của một lượt nghiệm thu.
 *
 * Bản trước bọc cả khối trong một `try` duy nhất với `abort()` ở `catch`. Ca nghiệm
 * thu *"`resolve.ts` NÉM khi thiếu khoá"* — tức đúng vi phạm AC4 mà Kiểm E tồn tại để
 * bắt — chạy ra: hai dòng FAIL đúng, rồi lời gọi `t()` KẾ TIẾP ném ra ngoài và bị
 * `abort()` báo thành *"Kiểm E KHÔNG chạy được"*. Mã thoát vẫn 1 nên nó không lọt,
 * nhưng người đọc log được bảo là hạ tầng hỏng trong khi thứ hỏng là sản phẩm — và
 * `check-deps.mjs:60-66` đặt luật này theo đúng cả hai chiều: một phép kiểm đỏ không
 * bao giờ được mặc áo lỗi hạ tầng, cũng như ngược lại.
 *
 * Ném ⇒ FAIL có tên, rồi đi tiếp để các mệnh đề còn lại vẫn được chấm.
 */
const call = (what, fn) => {
  try {
    return { ok: true, value: fn() }
  } catch (err) {
    fail(`${what} làm \`t()\` NÉM — AC4 nói "không sập": ${err?.message || err}`)
    eBad += 1
    return { ok: false, value: undefined }
  }
}

const expect = (what, got, want) => {
  if (got === want) return
  fail(`${what} — nhận \`${String(got)}\`, phải là \`${String(want)}\``)
  eBad += 1
}

try {
  const t = createResolver({
    'co.mat': 'Đã lưu.',
    'co.tham_so': 'Không đọc được tệp tại {path}.',
    'hai.tham_so': '{mot} và {hai}.',
  })

  // Chiều dương — khoá có mặt.
  expect('khoá có mặt', call('khoá có mặt', () => t('co.mat')).value, 'Đã lưu.')

  // Chiều âm — khoá thiếu. ⚠️ Cả hai vế đều bắt buộc: một resolver luôn trả `''` cũng
  // "không ném", nên chỉ kiểm "không ném" thì phép kiểm này vô nghĩa.
  const before = warnings.length
  expect('khoá thiếu trả về', call('khoá thiếu', () => t('khong.co')).value, 'khong.co')
  if (warnings.length === before) {
    fail('khoá thiếu KHÔNG ghi cảnh báo nào — AC4 đòi cả hai: hiện khoá VÀ ghi cảnh báo')
    eBad += 1
  }

  // Dedupe — cùng một khoá thiếu lần thứ hai không được ghi thêm dòng nào.
  const afterFirst = warnings.length
  call('khoá thiếu lần hai', () => t('khong.co'))
  if (warnings.length !== afterFirst) {
    fail('khoá thiếu ghi cảnh báo LẶP LẠI — một khoá thiếu trong template Vue chạy lại mỗi lần render')
    eBad += 1
  }

  // Nội suy tham số.
  expect(
    'nội suy tham số',
    call('nội suy tham số', () => t('co.tham_so', { path: '/tmp/a.txt' })).value,
    'Không đọc được tệp tại /tmp/a.txt.',
  )
  expect(
    'nội suy nhiều tham số',
    call('nội suy nhiều tham số', () => t('hai.tham_so', { mot: 'A', hai: 'B' })).value,
    'A và B.',
  )

  // Tham số thiếu — giữ nguyên placeholder, không ném, không thay bằng `undefined`.
  expect(
    'tham số thiếu giữ nguyên placeholder',
    call('tham số thiếu', () => t('co.tham_so')).value,
    'Không đọc được tệp tại {path}.',
  )

  // Tham số CÓ MẶT nhưng không phải chuỗi (`null` qua dây JSON, `undefined` ở một chỗ
  // gọi ẩu) phải xử như tham số THIẾU. `has(params, name)` đúng nhưng giá trị vô nghĩa
  // là ca mà doc-comment `resolve.ts:61-63` cấm đích danh — `"… tại undefined."` là một
  // câu hoàn chỉnh về ngữ pháp và sẽ đi thẳng ra màn hình người dùng.
  for (const [what, bad] of [
    ['null', null],
    ['undefined', undefined],
    ['số', 42],
  ]) {
    expect(
      `tham số kiểu ${what} giữ nguyên placeholder`,
      call(`tham số kiểu ${what}`, () => t('co.tham_so', { path: bad })).value,
      'Không đọc được tệp tại {path}.',
    )
  }

  // Khoá không phải chuỗi vẫn phải TRẢ VỀ CHUỖI — `Translate` khai `=> string`, và
  // một binding Vue nhận `undefined` thì render rỗng thay vì hiện khoá như AC4 đòi.
  for (const bad of [undefined, null, 1]) {
    const got = call(`khoá kiểu ${typeof bad}`, () => t(bad)).value
    if (typeof got !== 'string') {
      fail(`khoá kiểu \`${String(bad)}\` trả về \`${typeof got}\`, không phải chuỗi`)
      eBad += 1
    }
  }

  /**
   * 🔴 CHIỀU CUỐI CÙNG, VÀ LÀ CHIỀU DUY NHẤT CHẠM DỮ LIỆU THẬT.
   *
   * Mọi mệnh đề ở trên chạy trên một catalog GIẢ dựng ngay tại đây — chúng nghiệm thu
   * `resolve.ts`, không nghiệm thu `vi.json`. Không có khối này thì AC1 (*"chuỗi phân
   * giải từ `vi.json` theo khoá chấm"*) không có một bằng chứng thực thi nào ở đâu cả:
   * `src/i18n/index.ts` không được test nào nạp, không được tệp nguồn nào import, và
   * thay `export const t = createResolver(catalog)` bằng `(k) => k` vẫn xanh mọi cổng.
   *
   * ⚠️ Kiểm E KHÔNG nạp được `index.ts` — tệp đó `import` `./vi.json`, mà Node không
   * phân giải JSON theo luật bundler của Vite. Đó chính là lý do ranh giới hai tệp tồn
   * tại. Nên chỗ này dựng lại đúng phép ghép mà `index.ts` làm — `createResolver` trên
   * `vi.json` thật — và khẳng định trên dữ liệu thật.
   */
  const real = createResolver(catalog)
  for (const [key, value] of entries) {
    if (typeof value !== 'string') continue
    const got = call(`\`${key}\` trên catalog thật`, () => real(key)).value
    if (got === key && value !== key) {
      fail(`\`${key}\` phân giải thành CHÍNH KHOÁ trên \`vi.json\` thật — nhánh khoá thiếu đã chạy`)
      eBad += 1
    }
  }
  // Đường nội suy thật, từ đầu này tới đầu kia: khoá của `MessageKey::IoReadFailed`.
  const PROBE_KEY = 'err.io.read_failed'
  if (Object.prototype.hasOwnProperty.call(catalog, PROBE_KEY)) {
    const got = call(`\`${PROBE_KEY}\` nội suy thật`, () =>
      real(PROBE_KEY, { path: '/tmp/a.txt' }),
    ).value
    if (typeof got !== 'string' || !got.includes('/tmp/a.txt') || got.includes('{path}')) {
      fail(`\`${PROBE_KEY}\` không nội suy \`{path}\` trên \`vi.json\` thật — nhận "${got}"`)
      eBad += 1
    }
  }
} catch (err) {
  // Chỉ còn đúng một đường tới đây: `createResolver` tự ném lúc DỰNG resolver — trước
  // khi có lời gọi `t()` nào. Đó thật sự là "phép kiểm không chạy được".
  console.warn = realWarn
  abort('Kiểm E — `createResolver` ném ngay khi dựng resolver', err)
} finally {
  console.warn = realWarn
}

if (eBad === 0) {
  pass(
    'hành vi thật, cả hai chiều — khoá có · khoá thiếu · dedupe · nội suy · tham số thiếu · ' +
      `tham số sai kiểu · khoá sai kiểu · ${entries.length} khoá của \`vi.json\` THẬT`,
  )
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('')
if (failures !== 0) {
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  console.log('')
  console.log('Chuỗi hiển thị sống ở `src/i18n/vi.json` và chỉ ở đó (NFR16). Lỗi qua IPC mang')
  console.log('hình dạng `{ code, message_key, params, retryable }` và KHÔNG mang văn bản (AD-21).')
  console.log('Comment tiếng Việt KHÔNG phải vi phạm — chỉ vị trí mã mới là.')
  process.exit(1)
}
console.log('\x1b[32mTất cả phép kiểm chuỗi giao diện đạt.\x1b[0m')
process.exit(0)
