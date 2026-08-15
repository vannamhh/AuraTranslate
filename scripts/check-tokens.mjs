#!/usr/bin/env node
/**
 * Cổng token của Story 1.4 — cưỡng chế AC1..AC7 bằng lệnh, mã thoát là phán quyết.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * VÌ SAO ĐÂY LÀ NODE THUẦN CHỨ KHÔNG PHẢI ESLINT
 * ─────────────────────────────────────────────────────────────────────────────────
 * AC2 nói "khi **lint** chạy". Đọc thẳng ra sẽ là ESLint + một rule tự viết. Ba lý do
 * đo được nói không:
 *
 *   1. `ARCHITECTURE-SPINE.md §Consistency Conventions` chốt: mỗi phụ thuộc mới phải rà
 *      tương thích GPLv3 TRƯỚC khi thêm (NFR15), bằng cách MỞ TỆP GIẤY PHÉP trong nguồn
 *      đã tải. ESLint + `eslint-plugin-vue` + `typescript-eslint` kéo theo hàng trăm gói.
 *   2. Cây npm hiện là 59 gói. `check-deps.mjs` có ngưỡng sàn (`NPM_TREE_FLOOR = 30`) và
 *      quét mẫu trên TÊN GÓI; một cú nhảy lên vài trăm gói làm mọi con số nghiệm thu của
 *      Story 1.2 mất chỗ bám.
 *   3. Dự án đã có khuôn đang chạy tốt cho đúng loại việc này: `check-deps.mjs` (13 phép
 *      kiểm) · `check-scope.mjs` · `check-scope-bundled.mjs` · `config_invariants.rs`.
 *
 * `ARCHITECTURE-SPINE.md` gọi luật này là "lint cấm màu viết thẳng (AD-34)" — đó là TÊN
 * GỌI, không phải chỉ định công cụ. Script này *là* lint theo nghĩa AC2 dùng.
 *
 * ⚠️ Node, không phải bash: `npm run` trên Windows đi qua `cmd.exe`. Một cổng chỉ canh
 * được một nửa số nền tảng thì không canh được NFR14 (Ice chốt 2026-08-03).
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * BẢY PHÉP KIỂM
 * ─────────────────────────────────────────────────────────────────────────────────
 *   A (AC1) đối chiếu `tokens.json` với một bảng kỳ vọng ĐÓNG BĂNG trong chính script.
 *           Hai bản chép độc lập phải khớp; mọi chỗ lệch phải có mục ở `deviations`
 *           KÈM `question` và `reason` không rỗng.
 *   B (AC2) màu viết thẳng trong component — sáu cú pháp, không chỉ hex.
 *           Kèm B2: CỠ CHỮ viết thẳng. Câu chuyện của story là "mọi màu VÀ MỌI CỠ CHỮ
 *           đến từ một bộ token"; một cổng chỉ canh màu để lọt đúng nửa còn lại, và
 *           `App.vue:37-38` là bằng chứng có sẵn trong cây nguồn lúc story bắt đầu.
 *   C (AC3) tương phản WCAG 2.x cho MỌI cặp đã khai, cả hai theme. Kèm kiểm ĐẦY ĐỦ.
 *   D (AC4) MỌI `opacity` trung gian trong `src/**`, có miễn trừ có tên.
 *   E (AC5) sàn giãn dòng 1.66 cho token `wraps: true`.
 *   F (AC7) không bóng đổ, không gradient, không lớp nổi.
 *   G (AC6) hai theme phải khai HAI cơ chế phân tách panel khác nhau.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * NGUYÊN TẮC XƯƠNG SỐNG, thêm sau lượt rà soát 2026-08-03
 * ─────────────────────────────────────────────────────────────────────────────────
 * **KHÔNG một phán quyết nào của cổng được đọc tham số từ `tokens.json`.** Sàn WCAG,
 * danh sách vai, danh sách cặp loại trừ, danh sách màu đã loại — tất cả đóng băng ở đây.
 * Lượt rà soát chạy thật ba đường thoát và cả ba đều cho exit 0 trong khi sản phẩm mang
 * một cặp 4,245:1: hạ `contrast.floors` từ chính tệp bị kiểm · CHUYỂN cặp trượt sang
 * `excluded` với một chuỗi lý do bất kỳ · thêm một mục `deviations` không có lý do.
 * Một cổng đọc ngưỡng của mình từ thứ nó đang kiểm thì không phải cổng.
 *
 * Chạy:  npm run check:tokens
 */
import { readFileSync, readdirSync, lstatSync, existsSync, realpathSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join, relative, sep } from 'node:path'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const SRC_ROOT = join(REPO_ROOT, 'src')
const TOKENS_PATH = join(SRC_ROOT, 'tokens', 'tokens.json')

/**
 * "Cây rỗng không phải cây sạch" — thừa kế nguyên từ `check-deps.mjs`. Một lượt quét
 * KHÔNG tìm thấy tệp nào phải là LỖI QUÉT, không phải "đạt": mọi phép kiểm dưới đây
 * đều xanh trên một danh sách rỗng.
 *
 * ⚠️ Hai sàn, không phải một. `FILE_FLOOR` canh cây nguồn; `COMPONENT_FILE_FLOOR` canh
 * đúng quần thể mà Kiểm B/B2 cưỡng chế trên đó — tệp KHÔNG thuộc `src/tokens/**`. Sàn
 * đầu không thay được sàn sau: khi `src/tokens/` mọc thêm tệp, `files.length` vẫn qua
 * sàn trong khi số component có thể về 0, và toàn bộ AD-34 xanh rỗng.
 *
 * 🔴 NÂNG SÀN 2026-08-06 — Story 1.14 · AC11.1, đóng `deferred-work.md:48` và `:146`.
 * NÂNG LẠI 2026-08-06 — Story 1.17 · Task 10 (AC13): sàn 1.14 (26/23) đã tụt xuống
 * ~62–65% số thật sau ba story liên tiếp (1.15/1.16/1.17) — dưới hẳn tỷ lệ ~81% mà chính
 * comment này đặt ra, tức sàn đã "canh không được gì" đúng như cảnh báo ở dưới.
 *
 * Số THẬT sau Story 1.17: **40** tệp trong tầm quét, trong đó **37** là component (ngoài
 * `src/tokens/**`). Sau Story 1.14 là 32/29. Cây mọc thêm `LookupPanel.vue` (nội dung
 * thật), `LookupRecord.vue`, `lookupPanelState.ts`, cộng các tệp trước đó của 1.15/1.16.
 *
 * ⚠️ Sàn đặt ở ~81% số thật — cùng tỷ lệ dư địa mà `RS_FLOOR` của `check-i18n.mjs` giữ,
 * và cùng lý lẽ: sàn tồn tại để bắt một cây bị **CẮT MẤT**, không phải để đếm tệp mới.
 * Đặt nó bằng số thật là tự tạo một cổng đỏ ở story sau, và một cổng đỏ vì một lý do
 * không có thật là một cổng sắp bị gỡ.
 */
// 🔴 NÂNG LẠI 2026-08-12 — Story 2.2 · AC16, và lượt này là một lượt **bắt kịp**, không chỉ
// một lượt cộng thêm. Đo ngày 2026-08-12: **53** tệp trong tầm quét, **50** là component.
// Sàn cũ (37/35, đặt theo số của Story 1.19) đã tụt xuống **69,8% / 70,0%** — dưới hẳn dải
// ~81% mà doc-comment ngay trên đặt ra, tức đúng trạng thái *"canh không được gì"* mà chính
// nó cảnh báo. Ba story (1.20 · 1.21 · 2.1) thêm tệp mà không ai nâng sàn.
const FILE_FLOOR = 45 // số THẬT 2026-08-12 (sau Story 2.3): 55 tệp — 45/55 = 81,8%
const COMPONENT_FILE_FLOOR = 43 // số THẬT 2026-08-12 (sau Story 2.3): 52 tệp component — 43/52 = 82,7%

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
// BẢNG KỲ VỌNG ĐÓNG BĂNG — bản chép ĐỘC LẬP thứ hai của DESIGN.md
//
// Đây KHÔNG phải chỗ để "sửa cho khớp" khi Kiểm A đỏ. Nó tồn tại chính vì hai bản
// chép độc lập bắt được lỗi mà một bản không bắt được. Nếu Kiểm A đỏ, một trong hai
// bản sai — mở `DESIGN.md §Bảng token màu` ra mà phân xử, đừng chép bản này sang bản kia.
// Cùng khuôn với `BANNED_CRATES` của `check-deps.mjs` và allowlist của
// `config_invariants.rs`.
// ═════════════════════════════════════════════════════════════════════════════════

const EXPECTED_COLORS_LIGHT = {
  background: '#f4f1ea',
  surface: '#fbfaf6',
  'surface-sunken': '#f0ece1',
  'surface-accent': '#e6eeee',
  'surface-tm': '#faf6ee',
  'on-surface': '#2b2723',
  'on-surface-variant': '#6b6459',
  outline: '#e2dccf',
  'outline-faint': '#efeade',
  ornament: '#a9a196',
  // 🔵 2026-08-14 (Story 2.5b) — TOKEN THỨ 17, và nó MƯỢN ĐÚNG GIÁ TRỊ của `ornament`.
  //
  // 🔴 Vì sao một cái tên thứ hai cho một màu đã có: `check-commands.mjs` Kiểm I đối chiếu
  // HAI CHIỀU giữa `SEGMENT_RULE_VALUES` và các khối `.rule-<giá trị>` trong CSS, và nó đòi
  // đúng `background-color: var(--color-<giá trị>)`. Giá trị vạch thứ sáu (`draft`, UX-DR19)
  // vì thế **phải** có một token mang đúng tên đó — một `var(--color-ornament)` ở khối
  // `.rule-draft` làm cổng ĐỎ, và một bảng alias để cho lọt là đúng thứ §Miễn trừ cấm.
  //
  // ⚠️ Trùng giá trị KHÔNG phải trùng nghĩa: `ornament` = *đã về hưu* (Story 2.8),
  // `draft` = *đã dịch tay, chưa ai ký*. Hai vạch không bao giờ cùng xuất hiện trên một câu.
  // ⇒ Vì trùng giá trị, nó **không** mang một cặp tương phản mới nào vào bảng.
  draft: '#a9a196',
  primary: '#2f5d63',
  'on-primary': '#fbfaf6',
  confirmed: '#5a6b3f',
  'tm-rule': '#b99a5e',
  'tm-text': '#7a5d25',
  error: '#8f2f22',
}

const EXPECTED_COLORS_DARK = {
  background: '#201e1b',
  surface: '#26241f',
  'surface-sunken': '#1b1a17',
  'surface-accent': '#2c3a3b',
  'surface-tm': '#302b21',
  'on-surface': '#e8e3d8',
  'on-surface-variant': '#a29a8c',
  outline: '#3b382f',
  'outline-faint': '#302d26',
  ornament: '#6a6459',
  // 🔵 Story 2.5b — xem khối lý do ở `EXPECTED_COLORS_LIGHT`.
  draft: '#6a6459',
  primary: '#7fb3ba',
  'on-primary': '#1b1a17',
  confirmed: '#9cb37a',
  'tm-rule': '#b99a5e',
  'tm-text': '#d3b276',
  error: '#e5867a',
}

/** Cỡ / giãn dòng / nét / kiểu / giãn chữ — nguyên văn §Bảng token typography. */
const EXPECTED_TYPOGRAPHY = {
  'read-lg': { family: 'read', fontSize: '19px', lineHeight: '1.95', letterSpacing: '0.004em' },
  'read-md': { family: 'read', fontSize: '17.5px', lineHeight: '1.8' },
  'read-sm': { family: 'read', fontSize: '16px', lineHeight: '1.66' },
  'read-title': { family: 'read', fontSize: '23px', lineHeight: '1.3', fontWeight: '600' },
  'source-cjk': { family: 'read-cjk', fontSize: '16.5px', lineHeight: '2.05' },
  'source-hanviet': {
    family: 'read',
    fontSize: '12.5px',
    lineHeight: '1.95',
    fontStyle: 'italic',
  },
  editor: { family: 'read', fontSize: '15px', lineHeight: '1.95' },
  'lookup-headword': { family: 'read', fontSize: '24px', lineHeight: '1.3' },
  'lookup-gloss': { family: 'read', fontSize: '14.5px', lineHeight: '1.6' },
  'lookup-example': { family: 'read', fontSize: '12.5px', lineHeight: '1.6', fontStyle: 'italic' },
  'ui-md': { family: 'ui', fontSize: '13px', lineHeight: '1.5' },
  /*
   * ⚠️ `ui-md-strong` (Story 1.14 · AC10) CỐ Ý VẮNG MẶT ở đây.
   *
   * Nó tồn tại trong `tokens.json` nhưng KHÔNG có trong §Bảng token typography của
   * `DESIGN.md`, và bảng dưới đây là bản chép ĐỘC LẬP của bảng đó — 14 hàng, đúng 14.
   * Chữ ký cho hàng thứ 15 sống ở `tokens.deviations`, và `compare()` cưỡng chế rằng nó
   * phải có `question` + `reason` không rỗng. Xem lý lẽ đầy đủ ở phần `extra` của
   * `compare()`. Đừng "sửa" bằng cách thêm một hàng vào đây.
   */
  'ui-sm': { family: 'ui', fontSize: '12px', lineHeight: '1.5' },
  'ui-label': {
    family: 'ui',
    fontSize: '11px',
    lineHeight: '1.4',
    fontWeight: '700',
    letterSpacing: '0.1em',
  },
  'ui-mono': { family: 'mono', fontSize: '11.5px', lineHeight: '1.4' },
}

const EXPECTED_FAMILIES = {
  read: '"Source Serif 4", "Noto Serif CJK TC", serif',
  'read-cjk': '"Noto Serif CJK TC", serif',
  ui: '"Source Sans 3", ui-sans-serif, -apple-system, "Segoe UI", system-ui, sans-serif',
  mono: 'ui-monospace, SFMono-Regular, Consolas, monospace',
}

const EXPECTED_SPACING = {
  unit: '4px',
  'panel-inline': '16px',
  'panel-block': '12px',
  'head-height': '36px',
  'titlebar-height': '40px',
  'status-height': '34px',
  'gutter-width': '22px',
  'read-measure-lg': '62ch',
  'read-measure-md': '68ch',
  'read-measure-sm': '76ch',
}

const EXPECTED_ROUNDED = {
  none: '0',
  sm: '2px',
  DEFAULT: '3px',
  md: '4px',
  window: '9px',
  full: '9999px',
}

/**
 * Đếm bắt buộc — **17** / 17 / 16 / 4.
 *
 * 🔵 2026-08-14 (Story 2.5b): số màu mỗi theme **16 → 17**. Mệnh đề cũ — *"KHÔNG phải 17:
 * `tm-rule` cùng giá trị hai theme"* — nói về một cách đếm SAI (đếm `tm-rule` thành hai vì nó
 * trùng giá trị), và nó vẫn đúng phần của nó. Cái đổi là một token **thật** được thêm:
 * `draft`, giá trị vạch thứ sáu của UX-DR19. `DESIGN.md:196` cấm thêm token *"cho khớp một
 * con số cũ"* — đây là chiều ngược lại: con số đổi vì có một lý do đo được.
 *
 * ⚠️ `typography` là **17** kể từ Story 1.17 (`ui-md-wrap`, Quyết định #7) — trước đó đã
 * là **16** kể từ Story 1.16 (`source-latin`, Quyết định #6), và **15** kể từ Story 1.14
 * (`ui-md-strong`, AC10), không phải 14 như §Bảng token typography của `DESIGN.md` còn
 * ghi. Cả ba lệch đó CÓ CHỦ Ý và có chữ ký — xem `deviations` trong `tokens.json`. Sửa
 * `DESIGN.md` cho khớp là một lượt riêng của Ice.
 *
 * ⚠️ Con số này đã LÊN 18 rồi XUỐNG lại 17 trong ngày 2026-08-07: token
 * `source-cjk-parallel` được thêm để vá lỗi chồng chữ ở kiểu song song, rồi bị GỠ khi
 * `<ruby>` thay `position: absolute` làm cơ chế âm đọc — ruby chiếm chỗ thật nên không
 * cần một giãn dòng riêng. Ghi lại để lần sau không ai dựng lại token đó.
 */
const EXPECTED_COUNTS = { colorsPerTheme: 17, typography: 17, families: 4 }

// ─────────────────────────────────────────────────────────────────────────────────
// Hằng số của phép kiểm tương phản — ĐÓNG BĂNG, không đọc từ `tokens.json`
//
// 🔴 Lượt rà soát 2026-08-03 chạy thật: khôi phục `colors.dark.surface-accent` về
// `#2c3a3b` (khớp bảng đóng băng nên KHÔNG cần một mục `deviations` nào) rồi hạ
// `tokens.contrast.floors.normal` xuống 3.0 ⇒ cổng in "[dark] 31 cặp đạt AA · thấp nhất
// 4.245:1" và exit 0. 4,5 và 3,0 là hằng số WCAG 2.x, không phải cấu hình dự án.
// ─────────────────────────────────────────────────────────────────────────────────
const CONTRAST_FLOORS = { normal: 4.5, large: 3.0 }
const LARGE_TEXT_MIN_PX = 24

/** Vai quyết định token được phép làm gì. Vai KHÔNG đổi theo theme, và KHÔNG đổi theo tệp. */
const EXPECTED_ROLES = {
  text: ['on-surface', 'on-surface-variant', 'primary', 'on-primary', 'confirmed', 'tm-text', 'error'],
  surface: ['background', 'surface', 'surface-sunken', 'surface-accent', 'surface-tm', 'primary'],
  stroke: ['outline', 'outline-faint', 'ornament', 'draft', 'tm-rule'],
}

/**
 * 🔴 Danh sách cặp được LOẠI TRỪ, đóng băng.
 *
 * Story chừa sẵn đường thoát này ("một cặp không dùng thì không phải cặp") với điều kiện
 * phải VIẾT RA. Nhưng "viết ra" mà cổng chỉ đòi một chuỗi không rỗng thì nó là van xả áp:
 * lượt rà soát chạy thật — khôi phục `#2c3a3b` rồi CHUYỂN cặp `on-surface-variant` ×
 * `surface-accent` từ `pairs` sang `excluded` với lý do ba chữ "khong dung" ⇒ exit 0, cặp
 * 4,245:1 quay lại sản phẩm. XOÁ thì đỏ, CHUYỂN thì xanh — cùng một hậu quả.
 *
 * Thêm một hàng vào đây là một quyết định thiết kế phải qua rà soát, không phải một
 * lần sửa JSON. Mọi hàng dưới đây đều là hệ quả của cùng một mệnh đề: nền `primary` chỉ
 * mang đúng một màu chữ hợp lệ là `on-primary`, và `on-primary` chỉ đứng trên `primary`.
 */
const EXPECTED_EXCLUDED = new Set([
  'on-primary|background',
  'on-primary|surface',
  'on-primary|surface-sunken',
  'on-primary|surface-accent',
  'on-primary|surface-tm',
  'on-surface|primary',
  'on-surface-variant|primary',
  'primary|primary',
  'confirmed|primary',
  'tm-text|primary',
  'error|primary',
])

/** Màu đã bị loại khỏi vai chữ — phải vắng mặt hoàn toàn. Khoá đóng băng, lý do đọc từ JSON. */
const EXPECTED_BANNED_VALUES = ['#7d766c']

/** Token là màu của NÉT, không bao giờ là màu của chữ — ở cả hai theme. */
const EXPECTED_NEVER_TEXT = ['ornament', 'tm-rule']

// ═════════════════════════════════════════════════════════════════════════════════
// Nạp tokens.json
// ═════════════════════════════════════════════════════════════════════════════════
let tokens
let tokensText = ''
try {
  tokensText = readFileSync(TOKENS_PATH, 'utf8')
  tokens = JSON.parse(tokensText)
} catch (err) {
  abort(`\`${relative(REPO_ROOT, TOKENS_PATH)}\``, err)
}
if (!tokens || typeof tokens !== 'object' || Array.isArray(tokens)) {
  abort(`\`${relative(REPO_ROOT, TOKENS_PATH)}\``, new Error('Gốc tệp phải là một object.'))
}

const isToken = (k) => k !== '$doc'
const entriesOf = (obj) => Object.entries(obj ?? {}).filter(([k]) => isToken(k))
const keysOf = (obj) => entriesOf(obj).map(([k]) => k)
const asArray = (v, what) => {
  if (v === undefined || v === null) return []
  if (!Array.isArray(v)) abort(`\`${what}\` trong tokens.json`, new Error('Phải là một mảng.'))
  return v
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đọc cây nguồn MỘT LẦN
//
// ⚠️ Tầm quét là một phần của phán quyết. Trước lượt rà soát 2026-08-03 nó chỉ có
// `.vue`/`.ts`/`.css` dưới `src/`, nên `index.html` ở gốc repo — HÔM NAY đang là vỏ ứng
// dụng và là chỗ tự nhiên nhất để một `<style>` lọt vào — nằm ngoài mọi phép kiểm, cùng
// với `.svg`, `.js`, `.tsx`, `.scss`. Một cổng không nhìn thấy tệp thì không canh được nó.
// ═════════════════════════════════════════════════════════════════════════════════
const CSS_EXT = ['.css', '.scss']
const MARKUP_EXT = ['.vue', '.html', '.htm', '.svg']
const CODE_EXT = ['.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs']
const SCAN_EXT = [...CSS_EXT, ...MARKUP_EXT, ...CODE_EXT]

/** Tệp ngoài `src/**` vẫn thuộc bề mặt giao diện. `index.html` là vỏ ứng dụng thật. */
const EXTRA_FILES = [join(REPO_ROOT, 'index.html')]

const skippedLinks = []

/**
 * ⚠️ `lstatSync`, KHÔNG phải `statSync`: `statSync` giải symlink, nên một liên kết trỏ về
 * thư mục cha làm đệ quy không dừng, và một liên kết gãy ném `ENOENT` bị `abort()` báo
 * thành "cây nguồn không đọc được" — một liên kết hỏng làm sập cả cổng dưới danh nghĩa
 * lỗi hạ tầng. Symlink bị BỎ QUA và ghi tên ra, để việc bỏ qua không im lặng.
 */
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
    const full = join(dir, name)
    const st = lstatSync(full)
    if (st.isSymbolicLink()) {
      skippedLinks.push(relative(REPO_ROOT, full))
      continue
    }
    if (st.isDirectory()) walk(full, out, seen)
    else if (SCAN_EXT.some((e) => name.toLowerCase().endsWith(e))) out.push(full)
  }
  return out
}

let files = []
try {
  files = walk(SRC_ROOT).sort()
  for (const extra of EXTRA_FILES) if (existsSync(extra)) files.push(extra)
} catch (err) {
  abort('cây nguồn `src/**`', err)
}
if (files.length < FILE_FLOOR) {
  abort(
    `cây nguồn \`src/**\` — chỉ ${files.length} tệp, dưới sàn ${FILE_FLOOR}`,
    new Error('Cây quá nhỏ để là thật. Một danh sách rỗng làm MỌI phép kiểm dưới đây xanh.'),
  )
}

/** `src/tokens/**` là nơi màu ĐƯỢC PHÉP viết thẳng — đó là định nghĩa của chúng. */
const TOKENS_DIR_PREFIX = `src${sep}tokens${sep}`
const rel = (f) => relative(REPO_ROOT, f)
const isTokenSource = (f) => rel(f).startsWith(TOKENS_DIR_PREFIX)

const componentFiles = files.filter((f) => !isTokenSource(f))
if (componentFiles.length < COMPONENT_FILE_FLOOR) {
  abort(
    `quần thể component — chỉ ${componentFiles.length} tệp ngoài \`src/tokens/**\`, dưới sàn ${COMPONENT_FILE_FLOOR}`,
    new Error(
      'Kiểm B và B2 chỉ cưỡng chế trên quần thể này. `FILE_FLOOR` không thay được sàn này: ' +
        'khi `src/tokens/` mọc thêm tệp thì cây vẫn qua sàn trong khi số component về 0.',
    ),
  )
}

// ── Che comment và chuỗi, GIỮ NGUYÊN offset ──────────────────────────────────────
//
// ⚠️ Che chứ không xoá: mọi số dòng báo lỗi bên dưới tính từ offset trong văn bản gốc.
// Xoá đi thì mọi chẩn đoán trỏ sai dòng, và một cổng chỉ đường sai sẽ bị người sau
// thêm ngoại lệ cho tới khi nó không bắt được gì.
//
// 🔴 `text.split('')` chứ KHÔNG phải `[...text]`. Spread đánh chỉ số theo CODE POINT
// trong khi mọi chỉ số nạp vào nó (`i`, `indexOf`, `blankRange`) là UTF-16: một ký tự
// ngoài BMP — một emoji trong comment, một chữ Hán mở rộng — làm lệch toàn bộ offset từ
// đó trở đi, và cổng che nhầm vùng mà không có gì báo.
const blankRange = (chars, s, e) => {
  for (let i = s; i < e && i < chars.length; i++) if (chars[i] !== '\n') chars[i] = ' '
}

/**
 * 🔴 Chuỗi và comment KHÔNG ĐÓNG không được che.
 *
 * Bản trước quét `while (j < text.length && text[j] !== ch) j++` rồi che tới `j` — với
 * một dấu nháy lẻ thì `j` là hết tệp và toàn bộ phần còn lại bị xoá trắng. Lượt rà soát
 * chạy thật: `<p>don't</p>` trong template, cộng một khối mang `opacity: 0.4` trên chữ +
 * `box-shadow` + `z-index` trong `<style>` ⇒ **0 FAIL, exit 0**; đúng khối đó mà không có
 * dấu nháy lẻ ⇒ **3 FAIL, exit 1**. Một dấu nháy trong văn xuôi tiếng Việt hay tiếng Anh
 * tắt được Kiểm B, D và F cùng lúc.
 *
 * Luật: `'` và `"` phải đóng TRONG CÙNG MỘT DÒNG mới được coi là chuỗi (đúng ngữ nghĩa
 * của cả JS lẫn CSS); `` ` `` và `/* *\/` được phép nhiều dòng nhưng phải có chỗ đóng.
 * Không đóng ⇒ ký tự đó là ký tự thường, đi tiếp một bước.
 *
 * `cssRanges` tắt comment `//` bên trong vùng CSS: `background: url(//host/x.png)` không
 * phải một comment, và che nó nuốt luôn dấu `;` làm hai khai báo dính vào nhau.
 */
function maskCommentsAndStrings(text, { cssRanges = [] } = {}) {
  const chars = text.split('')
  const comments = []
  const inCss = (i) => cssRanges.some((r) => i >= r.start && i < r.end)
  let i = 0
  while (i < text.length) {
    const two = text.slice(i, i + 2)
    if (two === '/*') {
      const end = text.indexOf('*/', i + 2)
      if (end === -1) {
        i += 1
        continue
      }
      const stop = end + 2
      comments.push({ index: i, text: text.slice(i, stop) })
      blankRange(chars, i, stop)
      i = stop
      continue
    }
    if (two === '//' && !inCss(i)) {
      let end = text.indexOf('\n', i)
      if (end === -1) end = text.length
      comments.push({ index: i, text: text.slice(i, end) })
      blankRange(chars, i, end)
      i = end
      continue
    }
    const ch = text[i]
    if (ch === '"' || ch === "'" || ch === '`') {
      let j = i + 1
      let closed = false
      while (j < text.length) {
        if (text[j] === '\\') {
          j += 2
          continue
        }
        if (text[j] === ch) {
          closed = true
          break
        }
        if (text[j] === '\n' && ch !== '`') break
        j += 1
      }
      if (!closed) {
        i += 1
        continue
      }
      blankRange(chars, i + 1, j)
      i = j + 1
      continue
    }
    i += 1
  }
  return { masked: chars.join(''), comments }
}

const lineOf = (text, index) => text.slice(0, index).split('\n').length

// ── Phân tích khối CSS ───────────────────────────────────────────────────────────
//
// Đủ để trả lời hai câu hỏi mà Kiểm B và Kiểm D cần, và không hơn: (1) khai báo nào
// mang giá trị gì, (2) khai báo nằm ở đâu. Giới hạn của bộ phân tích này được ghi thẳng
// ở `deferred-work.md` — khi Story 1.14 dựng CSS thật và nhiều, soát lại SỐ KHAI BÁO mà
// cổng báo đã quét: con số tụt bất thường là dấu hiệu nó bỏ sót cả vùng.
function parseCssBlocks(masked, source) {
  const blocks = []
  const stack = []
  let buf = ''

  const flush = (block, endIndex) => {
    const raw = buf.trim()
    buf = ''
    if (!block || !raw) return
    const colon = raw.indexOf(':')
    if (colon <= 0) return
    const prop = raw.slice(0, colon).trim().toLowerCase()
    const value = raw.slice(colon + 1).trim()
    if (!prop || prop.startsWith('@')) return
    block.decls.push({ prop, value, index: endIndex, source })
  }

  for (let i = 0; i < masked.length; i++) {
    const ch = masked[i]
    if (ch === '{') {
      stack.push({ prelude: buf.trim(), decls: [], source })
      buf = ''
    } else if (ch === '}') {
      const block = stack.pop()
      flush(block, i)
      if (block) blocks.push(block)
      buf = ''
    } else if (ch === ';') {
      flush(stack[stack.length - 1], i)
    } else {
      buf += ch
    }
  }
  return blocks
}

// ── `style` trong markup ─────────────────────────────────────────────────────────
//
// 🔴 Bản trước chỉ bắt `style="…"` nháy kép TĨNH. Lượt rà soát chạy thật:
// `:style="{ color: 'red', fontSize: '13px' }"` và `style='color: rebeccapurple'` ⇒
// **exit 0**. `:style` là chính cách Vue khai style động — bỏ nó là bỏ đường dễ nhất.
const STYLE_ATTR_RE = /(?:^|[\s])(:?|v-bind:)style\s*=\s*("([^"]*)"|'([^']*)')/gi
const kebab = (s) => s.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase()

function inlineStyleBlocks(text, file) {
  const blocks = []
  let m
  const re = new RegExp(STYLE_ATTR_RE.source, 'gi')
  while ((m = re.exec(text))) {
    const bound = m[1] !== '' // `:style` / `v-bind:style` → giá trị là biểu thức JS
    const body = m[3] !== undefined ? m[3] : m[4]
    const base = m.index + m[0].indexOf(body)
    const decls = []
    if (!bound) {
      let off = 0
      for (const piece of body.split(';')) {
        const colon = piece.indexOf(':')
        if (colon > 0) {
          decls.push({
            prop: piece.slice(0, colon).trim().toLowerCase(),
            value: piece.slice(colon + 1).trim(),
            index: base + off,
            source: file,
          })
        }
        off += piece.length + 1
      }
    } else {
      // Hai hình dạng: một chuỗi (`:style="'color: red'"`) hoặc một object literal.
      const objRe = /(['"]?)([A-Za-z-]+)\1\s*:\s*(?:(['"])(.*?)\3|([^,}]+))/g
      let o
      while ((o = objRe.exec(body))) {
        const value = (o[4] !== undefined ? o[4] : o[5] || '').trim()
        if (!value) continue
        decls.push({ prop: kebab(o[2]).toLowerCase(), value, index: base + o.index, source: file })
      }
    }
    // ⚠️ MỘT khối cho MỖI thuộc tính, không gộp cả tệp. Gộp thì Kiểm D nhiễu chéo giữa
    // các thẻ không liên quan, và mọi khai báo báo về cùng một số dòng.
    if (decls.length) blocks.push({ prelude: 'style=""', decls, source: file })
  }
  return blocks
}

/** Một tệp đã đọc, đã che, đã phân tích. */
const parsed = files.map((file) => {
  const low = file.toLowerCase()
  const isCss = CSS_EXT.some((e) => low.endsWith(e))
  const isMarkup = MARKUP_EXT.some((e) => low.endsWith(e))
  let text
  try {
    text = readFileSync(file, 'utf8')
  } catch (err) {
    abort(`tệp \`${rel(file)}\``, err)
  }

  /** Vùng CSS thật của tệp: cả tệp `.css`, hoặc từng khối `<style>` của markup. */
  const cssRegions = []
  if (isCss) cssRegions.push({ start: 0, end: text.length })
  else if (isMarkup) {
    // ⚠️ `</style>` khớp KHÔNG phân biệt hoa thường và cho phép khoảng trắng — bản trước
    // dùng `indexOf('</style>')`, nên `</STYLE>` làm vùng kéo tới hết tệp và một `<style>`
    // sau đó mở một vùng chồng lấn được phân tích hai lần.
    const open = /<style[^>]*>/gi
    let m
    while ((m = open.exec(text))) {
      const start = m.index + m[0].length
      const close = /<\/\s*style\s*>/gi
      close.lastIndex = start
      const c = close.exec(text)
      const end = c ? c.index : text.length
      cssRegions.push({ start, end })
      open.lastIndex = end
    }
  }

  const { masked, comments } = maskCommentsAndStrings(text, { cssRanges: cssRegions })

  const blocks = []
  for (const r of cssRegions) {
    // Giữ offset tuyệt đối bằng cách đệm khoảng trắng phía trước vùng.
    const padded = ' '.repeat(r.start) + masked.slice(r.start, r.end)
    blocks.push(...parseCssBlocks(padded, file))
  }
  if (isMarkup) blocks.push(...inlineStyleBlocks(text, file))

  return { file, text, masked, comments, blocks, isMarkup, isCss, cssRegions }
})

const allDecls = parsed.flatMap((p) =>
  p.blocks.flatMap((b) => b.decls.map((d) => ({ ...d, file: p.file, text: p.text }))),
)
const where = (d) => `${rel(d.file)}:${lineOf(d.text, d.index)}`

/** Miễn trừ có tên: một comment `/* aura-allow-<gì>: <lý do> *\/` trong phạm vi một dòng. */
const exemptAt = (p, index, kind) => {
  const line = lineOf(p.text, index)
  const re = new RegExp(`aura-allow-${kind}\\s*:\\s*\\S`)
  return p.comments.some((c) => re.test(c.text) && Math.abs(lineOf(p.text, c.index) - line) <= 1)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Tương phản WCAG 2.x
// ═════════════════════════════════════════════════════════════════════════════════
const HEX6_RE = /^#[0-9a-fA-F]{6}$/
const srgbChannel = (c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4)

function luminance(hex) {
  const n = parseInt(hex.slice(1), 16)
  const [r, g, b] = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map((v) => srgbChannel(v / 255))
  return 0.2126 * r + 0.7152 * g + 0.0722 * b
}

/**
 * ⚠️ Ném thay vì trả `NaN`. `parseInt('#abc'.slice(1), 16)` cho 2748 — một số HỢP LỆ cho
 * một giá trị SAI; với `rgb(…)` hay tên màu thì ra `NaN`, và `NaN < 4.5` là `false`, tức
 * cặp được tuyên bố ĐẠT. Một phép so sánh im lặng nhận `NaN` là chỗ mà mọi phán quyết
 * của Kiểm C thành rác kèm hai chữ số thập phân trông rất thuyết phục.
 */
function contrast(a, b) {
  if (!HEX6_RE.test(a) || !HEX6_RE.test(b)) {
    throw new Error(`giá trị màu phải là \`#rrggbb\`; nhận \`${a}\` và \`${b}\``)
  }
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x)
  return (hi + 0.05) / (lo + 0.05)
}

// ⚠️ Tự kiểm công thức TRƯỚC khi tin bất cứ con số nào nó phát ra. `DESIGN.md §Sàn
// tương phản` công bố 5,2:1 cho `on-surface-variant #6b6459` trên nền giấy `#f4f1ea`;
// một cài đặt đúng phải tái lập được 5,18. Sai ở đây thì mọi phán quyết của Kiểm C là
// rác — và rác đi kèm hai chữ số thập phân trông rất thuyết phục.
{
  const probe = contrast('#6b6459', '#f4f1ea')
  if (Math.abs(probe - 5.18) > 0.02) {
    abort(
      'phép tự kiểm công thức tương phản',
      new Error(
        `contrast('#6b6459', '#f4f1ea') = ${probe.toFixed(4)}, kỳ vọng ≈ 5,18 ` +
          '(con số DESIGN.md công bố là 5,2). Công thức luminance đã hỏng.',
      ),
    )
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm A — đủ token, đúng giá trị (AC1)')
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Mọi chỗ lệch khỏi bảng đóng băng phải có mục ở `deviations`, KÈM LÝ DO.
 *
 * 🔴 Bản trước chỉ đối chiếu `designValue`/`value` — `question` và `reason` không hề được
 * đọc. Lượt rà soát chạy thật: đổi `colors.light.primary` rồi thêm một mục deviation
 * KHÔNG có trường `reason` nào ⇒ exit 0. Sổ deviation vừa *ghi nhận* vừa *cấp phép*, và
 * phần cấp phép không đòi gì cả.
 */
const deviations = new Map(asArray(tokens.deviations, 'deviations').map((d) => [d?.path, d]))
const deviationsUsed = new Set()

const deviationOk = (path, wantV, gotV) => {
  const dev = deviations.get(path)
  if (!dev) return false
  if (String(dev.designValue) !== String(wantV) || String(dev.value) !== String(gotV)) return false
  const reason = String(dev.reason ?? '').trim()
  const question = String(dev.question ?? '').trim()
  if (!reason || !question) {
    fail(`deviation \`${path}\` thiếu \`${!question ? 'question' : 'reason'}\` — một chỗ lệch không có lý do là một chỗ lệch không được phép`)
    detail('Sổ `deviations` là bằng chứng, không phải công tắc. Mỗi mục phải nêu ai quyết và vì sao.')
    return false
  }
  deviationsUsed.add(path)
  return true
}

function compare(group, expected, actual, pick = (v) => v) {
  const expKeys = Object.keys(expected)
  const actKeys = keysOf(actual)
  const missing = expKeys.filter((k) => !actKeys.includes(k))
  const extra = actKeys.filter((k) => !expKeys.includes(k))
  let bad = 0

  if (missing.length) {
    fail(`${group}: thiếu ${missing.length} token — ${missing.join(', ')}`)
    bad += missing.length
  }
  /**
   * 🔴 TOKEN THỪA — token có trong `tokens.json` mà bảng của `DESIGN.md` KHÔNG có.
   *
   * ⚠️ Đây là một chỗ lệch THẬT và nó phải đi qua sổ `deviations` như mọi chỗ lệch khác.
   * Story 1.14 · AC10 là lần đầu tiên ca này xảy ra (`typography.ui-md-strong`), và
   * đường thoát dễ mà nó mở ra rất đắt: **thêm hàng vào bảng đóng băng ngay trên**. Làm
   * vậy là để bảng — thứ khối chú thích ở đầu tệp gọi là *"bản chép ĐỘC LẬP thứ hai của
   * DESIGN.md"* — lặng lẽ trôi khỏi `DESIGN.md`, và người sau đối chiếu hai bên sẽ thấy
   * lệch mà không có gì nói vì sao. Hai bản chép chỉ bắt được lỗi khi cả hai còn chép
   * cùng một thứ.
   *
   * ⇒ Bảng đóng băng ở lại đúng 14 hàng của `DESIGN.md`. Hàng thứ 15 sống trong
   * `tokens.json` và **được cấp phép bằng một mục `deviations` có `question` và `reason`
   * không rỗng** — tức một chữ ký, đúng như khối chú thích đầu bảng đòi.
   *
   * Quy ước `designValue`/`value` cho ca này, để mục deviation đọc được thành câu:
   *   designValue: "(không có trong bảng)"   value: "(token mới)"
   */
  const EXTRA_WANT = '(không có trong bảng)'
  const EXTRA_GOT = '(token mới)'
  const unsigned = extra.filter((k) => !deviationOk(`${group}.${k}`, EXTRA_WANT, EXTRA_GOT))
  for (const k of extra.filter((x) => !unsigned.includes(x))) {
    pass(`${group}.${k} — token NGOÀI bảng DESIGN.md, có deviation ký tên`)
  }
  if (unsigned.length) {
    fail(`${group}: thừa ${unsigned.length} token KHÔNG có chữ ký — ${unsigned.join(', ')}`)
    detail('Đừng thêm token để cho khớp một con số cũ. Mọi token mới phải qua Kiểm C.')
    detail(
      `Và đừng thêm hàng vào bảng đóng băng ở đầu tệp này: khai một mục \`deviations\` ` +
        `\`{ path: "${group}.${unsigned[0]}", designValue: "${EXTRA_WANT}", value: "${EXTRA_GOT}", question, reason }\`.`,
    )
    bad += unsigned.length
  }

  for (const key of expKeys) {
    if (missing.includes(key)) continue
    const want = expected[key]
    const got = pick(actual[key])
    if (typeof want === 'object') {
      for (const [field, wantV] of Object.entries(want)) {
        const gotV = got?.[field] === undefined ? undefined : String(got[field])
        if (gotV === String(wantV)) continue
        if (deviationOk(`${group}.${key}.${field}`, wantV, gotV)) continue
        fail(`${group}.${key}.${field} = ${gotV} — bảng DESIGN.md ghi ${wantV}`)
        bad += 1
      }
    } else if (String(got) !== String(want)) {
      if (deviationOk(`${group}.${key}`, want, got)) continue
      fail(`${group}.${key} = ${got} — bảng DESIGN.md ghi ${want}`)
      bad += 1
    }
  }
  if (bad === 0) pass(`${group}: ${expKeys.length} token, khớp bảng DESIGN.md`)
  return bad
}

compare('colors.light', EXPECTED_COLORS_LIGHT, tokens.colors?.light)
compare('colors.dark', EXPECTED_COLORS_DARK, tokens.colors?.dark)
compare('typography', EXPECTED_TYPOGRAPHY, tokens.typography)
compare('families', EXPECTED_FAMILIES, tokens.families)
compare('spacing', EXPECTED_SPACING, tokens.spacing)
compare('rounded', EXPECTED_ROUNDED, tokens.rounded)

// Đếm tường minh — mệnh đề 16/16/14/4 của AC1.
const counts = [
  ['colors.light', keysOf(tokens.colors?.light).length, EXPECTED_COUNTS.colorsPerTheme],
  ['colors.dark', keysOf(tokens.colors?.dark).length, EXPECTED_COUNTS.colorsPerTheme],
  ['typography', keysOf(tokens.typography).length, EXPECTED_COUNTS.typography],
  ['families', keysOf(tokens.families).length, EXPECTED_COUNTS.families],
]
for (const [name, got, want] of counts) {
  if (got === want) pass(`đếm ${name} = ${got}`)
  else fail(`đếm ${name} = ${got}, phải là ${want}`)
}

// Hình dạng giá trị màu — điều kiện để Kiểm C tính được gì đó có thật.
let shapeBad = 0
for (const theme of ['light', 'dark']) {
  for (const [name, value] of entriesOf(tokens.colors?.[theme])) {
    if (!HEX6_RE.test(String(value))) {
      fail(`colors.${theme}.${name} = \`${value}\` — phải là \`#rrggbb\` sáu chữ số`)
      detail('`#abc`, `rgb(…)` và tên màu làm phép tính tương phản ra `NaN`, và `NaN < 4.5` là `false`.')
      shapeBad += 1
    }
  }
}
if (shapeBad === 0) pass('32 giá trị màu đều đúng hình dạng `#rrggbb`')

// Mọi deviation đã khai phải THẬT SỰ được dùng. Một mục thừa nghĩa là hoặc ai đó đã
// lặng lẽ khôi phục giá trị gốc mà quên gỡ mục, hoặc mục được viết ra để chừa sẵn chỗ
// cho một lần lệch tương lai — cả hai đều làm sổ deviation mất giá trị làm bằng chứng.
for (const [path, dev] of deviations) {
  if (deviationsUsed.has(path)) {
    pass(`deviation đã khai và đang áp: ${path} (${dev.designValue} → ${dev.value})`)
  } else {
    fail(`deviation \`${path}\` khai trong tokens.json nhưng KHÔNG khớp chỗ lệch nào`)
    detail('Gỡ mục thừa, hoặc sửa `designValue`/`value` cho đúng chỗ lệch thật.')
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm B — màu viết thẳng trong component bị từ chối (AC2)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// Chỉ quét hex là bỏ lọt bốn cú pháp: `rgb(43 39 35)` · `hsl(30 10% 15%)` ·
// `color(display-p3 …)` · và TÊN MÀU CSS. Cả bốn đều là màu viết thẳng.

const NAMED_COLORS = new Set(
  ('aliceblue antiquewhite aqua aquamarine azure beige bisque black blanchedalmond blue ' +
    'blueviolet brown burlywood cadetblue chartreuse chocolate coral cornflowerblue cornsilk ' +
    'crimson cyan darkblue darkcyan darkgoldenrod darkgray darkgreen darkgrey darkkhaki ' +
    'darkmagenta darkolivegreen darkorange darkorchid darkred darksalmon darkseagreen ' +
    'darkslateblue darkslategray darkslategrey darkturquoise darkviolet deeppink deepskyblue ' +
    'dimgray dimgrey dodgerblue firebrick floralwhite forestgreen fuchsia gainsboro ghostwhite ' +
    'gold goldenrod gray green greenyellow grey honeydew hotpink indianred indigo ivory khaki ' +
    'lavender lavenderblush lawngreen lemonchiffon lightblue lightcoral lightcyan ' +
    'lightgoldenrodyellow lightgray lightgreen lightgrey lightpink lightsalmon lightseagreen ' +
    'lightskyblue lightslategray lightslategrey lightsteelblue lightyellow lime limegreen linen ' +
    'magenta maroon mediumaquamarine mediumblue mediumorchid mediumpurple mediumseagreen ' +
    'mediumslateblue mediumspringgreen mediumturquoise mediumvioletred midnightblue mintcream ' +
    'mistyrose moccasin navajowhite navy oldlace olive olivedrab orange orangered orchid ' +
    'palegoldenrod palegreen paleturquoise palevioletred papayawhip peachpuff peru pink plum ' +
    'powderblue purple rebeccapurple red rosybrown royalblue saddlebrown salmon sandybrown ' +
    'seagreen seashell sienna silver skyblue slateblue slategray slategrey snow springgreen ' +
    'steelblue tan teal thistle tomato turquoise violet wheat white whitesmoke yellow ' +
    'yellowgreen ' +
    // Màu HỆ THỐNG CSS — cũng là màu viết thẳng, và tệ hơn: chúng phớt lờ theme của
    // ứng dụng và đi theo theme của hệ điều hành, nên hai người dùng thấy hai giao diện.
    'canvas canvastext linktext visitedtext activetext buttonface buttontext buttonborder ' +
    'field fieldtext highlight highlighttext selecteditem selecteditemtext mark marktext ' +
    'graytext accentcolor accentcolortext').split(' '),
)

/**
 * Cú pháp màu chữ-và-số.
 *
 * ⚠️ Độ dài hex là 3, 4, 6 hoặc 8 — KHÔNG phải `{3,8}`. Bản trước bắt mọi dãy 3–8 ký tự
 * hex, nên `document.querySelector('#faded')` bị báo là màu viết thẳng (đã chạy thật).
 * Comment ở đầu tệp cảnh báo *"một cổng chỉ đường sai sẽ bị người sau thêm ngoại lệ cho
 * tới khi nó không bắt được gì"* — dương tính giả là cái đòn bẩy đó.
 *
 * `#dad` và `#decade` vẫn là hex hợp lệ nên vẫn khớp; đường ra cho chúng là miễn trừ CÓ
 * TÊN `/* aura-allow-literal: <lý do> *\/`, không phải nới regex.
 */
const LITERAL_COLOR_RE =
  /#(?:[0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{4}|[0-9a-fA-F]{3})(?![0-9a-zA-Z_-])|\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch|color|color-mix|device-cmyk)\s*\(/g

/** Thuộc tính mà giá trị BẮT BUỘC là một tham chiếu token — dạng danh sách CHO PHÉP. */
const PURE_COLOR_PROPS = new Set([
  'color',
  'background-color',
  'border-color',
  'border-top-color',
  'border-right-color',
  'border-bottom-color',
  'border-left-color',
  'outline-color',
  'fill',
  'stroke',
  'caret-color',
  'accent-color',
  'text-decoration-color',
  'column-rule-color',
  '-webkit-text-fill-color',
  '-webkit-text-stroke-color',
])

/** Thuộc tính có thể CHỨA màu trong một giá trị ghép. Chỉ áp phần danh sách CẤM. */
const COMPOSITE_COLOR_PROPS = new Set([
  'background',
  'border',
  'border-top',
  'border-right',
  'border-bottom',
  'border-left',
  'outline',
  'column-rule',
  'text-decoration',
  'scrollbar-color',
  'background-image',
])

const ALLOWED_COLOR_KEYWORDS = new Set([
  'inherit',
  'initial',
  'unset',
  'revert',
  'revert-layer',
  'currentcolor',
  'transparent',
  'none',
])

/** `!important` là chỉ thị ưu tiên, không phải một phần của giá trị. */
const stripImportant = (v) => v.replace(/\s*!\s*important\s*$/i, '').trim()

/** Tách giá trị thành các phần ở TẦNG NGOÀI — `border-color` hợp lệ nhận 1..4 giá trị. */
function topLevelParts(value) {
  const parts = []
  let depth = 0
  let buf = ''
  for (const ch of value) {
    if (ch === '(') depth += 1
    if (ch === ')') depth -= 1
    if (depth === 0 && /\s/.test(ch)) {
      if (buf.trim()) parts.push(buf.trim())
      buf = ''
    } else buf += ch
  }
  if (buf.trim()) parts.push(buf.trim())
  return parts
}

const COLOR_VAR_RE = /^var\(\s*--(?:color|panel-border-color)[a-z0-9-]*\s*(?:,[\s\S]*)?\)$/

let bLiteral = 0
let bAllowlist = 0
const componentDecls = allDecls.filter((d) => !isTokenSource(d.file))

for (const d of componentDecls) {
  const value = stripImportant(d.value)
  // Danh sách CẤM — sáu cú pháp chữ-và-số, ở bất kỳ thuộc tính nào.
  const hits = value.match(LITERAL_COLOR_RE)
  if (hits) {
    fail(`${where(d)} — màu viết thẳng trong \`${d.prop}: ${d.value}\` (${[...new Set(hits)].join(', ')})`)
    bLiteral += 1
  }
  // Tên màu CSS: chỉ soi giá trị của thuộc tính màu, và bỏ qua phần trong `var(…)`.
  if (PURE_COLOR_PROPS.has(d.prop) || COMPOSITE_COLOR_PROPS.has(d.prop)) {
    const outsideVar = value.replace(/var\([^)]*\)/g, ' ')
    for (const word of outsideVar.toLowerCase().match(/[a-z-]+/g) ?? []) {
      if (NAMED_COLORS.has(word) && !ALLOWED_COLOR_KEYWORDS.has(word)) {
        fail(`${where(d)} — tên màu CSS viết thẳng: \`${d.prop}: ${d.value}\` (\`${word}\`)`)
        bLiteral += 1
      }
    }
  }
  // Danh sách CHO PHÉP — `config_invariants.rs:92-94`: "một danh sách cấm chỉ chặn được
  // những hình dạng ai đó đã nghĩ ra". Ở thuộc tính màu thuần thì hỏi ngược lại được:
  // token nào đang được dùng ở đây?
  if (PURE_COLOR_PROPS.has(d.prop)) {
    const parts = topLevelParts(value.toLowerCase())
    const ok =
      parts.length > 0 &&
      parts.length <= 4 &&
      parts.every((p) => COLOR_VAR_RE.test(p) || ALLOWED_COLOR_KEYWORDS.has(p))
    if (!ok) {
      fail(`${where(d)} — \`${d.prop}\` phải là \`var(--color-…)\` hoặc từ khoá cho phép, đang là \`${d.value}\``)
      bAllowlist += 1
    }
  }
}

// Mã (`.ts`, `<script>` của markup) không có khối CSS nhưng vẫn dựng được màu bằng chuỗi.
// ⚠️ Vùng CSS bị LOẠI khỏi lượt quét này — nó đã được quét ở tầng khai báo phía trên.
// Bản trước dùng `scanned[m.index] === undefined` để làm việc đó, nhưng vùng `<style>`
// được thay bằng khoảng trắng CÙNG ĐỘ DÀI nên điều kiện đó không bao giờ đúng: một màu
// trong `<style>` của `.vue` bị báo BA lần và `failures` đếm vi phạm mà gọi là "phép kiểm".
let bTs = 0
for (const p of parsed) {
  if (isTokenSource(p.file) || p.isCss) continue
  const inCss = (i) => p.cssRegions.some((r) => i >= r.start && i < r.end)
  const re = new RegExp(LITERAL_COLOR_RE.source, 'g')
  let m
  while ((m = re.exec(p.text))) {
    if (inCss(m.index)) continue
    const inComment = p.comments.some((c) => m.index >= c.index && m.index < c.index + c.text.length)
    if (inComment) continue
    if (exemptAt(p, m.index, 'literal')) {
      pass(`${rel(p.file)}:${lineOf(p.text, m.index)} — \`${m[0]}\` có miễn trừ có tên`)
      continue
    }
    fail(`${rel(p.file)}:${lineOf(p.text, m.index)} — màu viết thẳng trong mã: \`${m[0]}\``)
    detail('Nếu đây là một selector/anchor chứ không phải màu: thêm `/* aura-allow-literal: <lý do> */`.')
    bTs += 1
  }
}

// B2 — CỠ CHỮ viết thẳng. Cùng lập luận với màu: thứ cần kiểm tập trung thì không được
// rải rác. Không có phép kiểm này thì `App.vue:37-38` (`font-family: ui-monospace,
// monospace` + `font-size: 0.8125rem`) đứng yên trong cây nguồn qua chín epic.
const TYPE_PROPS = new Set([
  'font',
  'font-family',
  'font-size',
  'line-height',
  'font-weight',
  'font-style',
  'letter-spacing',
  'font-synthesis',
  'font-stretch',
  'font-variant',
  'font-size-adjust',
  'font-variation-settings',
])
const ALLOWED_TYPE_KEYWORDS = new Set(['inherit', 'initial', 'unset', 'revert', 'revert-layer', 'normal'])
let bType = 0
for (const d of componentDecls) {
  if (!TYPE_PROPS.has(d.prop)) continue
  if (d.prop === 'font') {
    fail(`${where(d)} — \`font\` shorthand không chở được \`letter-spacing\`/\`font-synthesis\`; dùng bảy biến của token`)
    bType += 1
    continue
  }
  const v = stripImportant(d.value).toLowerCase()
  const ok = /^var\(\s*--[a-z0-9-]+\s*(?:,[\s\S]*)?\)$/.test(v) || ALLOWED_TYPE_KEYWORDS.has(v)
  if (!ok) {
    fail(`${where(d)} — cỡ/họ chữ viết thẳng: \`${d.prop}: ${d.value}\``)
    detail('Mọi cỡ chữ đến từ token: `var(--font-…)` · `var(--face-…)` · `var(--leading-…)` …')
    bType += 1
  }
}

// Miễn trừ `src/tokens/**` là điều kiện của Kiểm B, nhưng một miễn trừ không được soi
// là một chỗ mù. Hôm nay nó RỖNG — giá trị màu sống trong `tokens.json` (không phải tệp
// được quét), còn `.ts`/`.css` của tầng token chỉ đọc lại chúng. Khẳng định điều đó
// thành một phép kiểm để nó không lặng lẽ thôi đúng.
const tokenLayerLiterals = []
for (const p of parsed) {
  if (!isTokenSource(p.file)) continue
  const re = new RegExp(LITERAL_COLOR_RE.source, 'g')
  let m
  while ((m = re.exec(p.text))) {
    const inComment = p.comments.some((c) => m.index >= c.index && m.index < c.index + c.text.length)
    if (!inComment) tokenLayerLiterals.push(`${rel(p.file)}:${lineOf(p.text, m.index)} \`${m[0]}\``)
  }
}

if (bLiteral + bAllowlist + bTs + bType === 0) {
  pass(
    `không màu viết thẳng — ${componentDecls.length} khai báo CSS trên ${componentFiles.length} tệp component`,
  )
  pass('không cỡ/họ chữ viết thẳng trong component (B2)')
  if (tokenLayerLiterals.length) {
    detail(`miễn trừ \`src/tokens/**\` đang che ${tokenLayerLiterals.length} giá trị: ${tokenLayerLiterals.join(' · ')}`)
  } else {
    pass('miễn trừ `src/tokens/**` hiện RỖNG — giá trị màu chỉ sống trong `tokens.json`')
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm C — tương phản WCAG AA, cả hai theme (AC3)')
// ═════════════════════════════════════════════════════════════════════════════════

const cfg = tokens.contrast ?? {}
let cBad = 0

// C0 — vai và ngưỡng phải khớp bảng đóng băng. `tokens.json` được phép NHẮC LẠI chúng cho
// người đọc, nhưng không được là nơi phán quyết đọc chúng ra.
{
  for (const [name, want] of Object.entries(EXPECTED_ROLES)) {
    const got = asArray(tokens.roles?.[name], `roles.${name}`).map(String)
    if (got.join('|') !== want.join('|')) {
      fail(`roles.${name} lệch bảng đóng băng — đang là [${got.join(', ')}], phải là [${want.join(', ')}]`)
      cBad += 1
    }
  }
  const fl = cfg.floors ?? {}
  if (Number(fl.normal) !== CONTRAST_FLOORS.normal || Number(fl.large) !== CONTRAST_FLOORS.large) {
    fail(
      `contrast.floors trong tokens.json (${JSON.stringify(fl)}) không khớp hằng số WCAG đóng băng ` +
        `(${JSON.stringify(CONTRAST_FLOORS)})`,
    )
    detail('Sàn AA không phải cấu hình dự án. Sửa cho khớp, hoặc gỡ khỏi tokens.json.')
    cBad += 1
  }
  if (Number(cfg.largeTextMinPx) !== LARGE_TEXT_MIN_PX) {
    fail(`contrast.largeTextMinPx = ${cfg.largeTextMinPx}, hằng số đóng băng là ${LARGE_TEXT_MIN_PX}`)
    cBad += 1
  }
  if (cBad === 0) pass(`vai và ngưỡng khớp bảng đóng băng — sàn ${CONTRAST_FLOORS.normal}:1 / ${CONTRAST_FLOORS.large}:1`)
}

// C1 — ĐẦY ĐỦ, và danh sách LOẠI TRỪ là danh sách ĐÓNG BĂNG.
// Đây là phép kiểm quan trọng nhất của Kiểm C: một danh sách tự rút gọn để cho xanh
// là đúng thứ AD-34 tồn tại để chặn — và "rút gọn" gồm cả việc CHUYỂN sang `excluded`.
{
  const pairs = asArray(cfg.pairs, 'contrast.pairs')
  const excluded = asArray(cfg.excluded, 'contrast.excluded')
  const declared = new Map()
  for (const p of pairs) declared.set(`${p.fg}|${p.bg}`, 'checked')

  const seenExcluded = new Set()
  for (const p of excluded) {
    const key = `${p.fg}|${p.bg}`
    seenExcluded.add(key)
    if (!EXPECTED_EXCLUDED.has(key)) {
      fail(`cặp loại trừ \`${p.fg}\` × \`${p.bg}\` KHÔNG có trong danh sách đóng băng của script`)
      detail('Loại một cặp là quyết định thiết kế, không phải một lần sửa JSON. Đưa qua rà soát trước.')
      cBad += 1
    }
    if (!String(p.reason ?? '').trim()) {
      fail(`cặp loại trừ \`${p.fg}\` × \`${p.bg}\` KHÔNG có lý do — im lặng bỏ một cặp là FAIL`)
      cBad += 1
    }
    declared.set(key, 'excluded')
  }
  for (const key of EXPECTED_EXCLUDED) {
    if (!seenExcluded.has(key)) {
      fail(`cặp loại trừ \`${key.replace('|', '` × `')}\` có trong script nhưng biến mất khỏi tokens.json`)
      cBad += 1
    }
  }

  const missing = []
  for (const fg of EXPECTED_ROLES.text) {
    for (const bg of EXPECTED_ROLES.surface) {
      if (!declared.has(`${fg}|${bg}`)) missing.push(`${fg} × ${bg}`)
    }
  }
  if (missing.length) {
    fail(`${missing.length} tổ hợp (chữ × nền) chưa khai ở \`pairs\` cũng chưa ở \`excluded\``)
    missing.forEach((m) => detail(m))
    cBad += missing.length
  } else {
    const n = EXPECTED_ROLES.text.length * EXPECTED_ROLES.surface.length
    pass(`đầy đủ: ${n} tổ hợp = ${pairs.length} cặp kiểm + ${excluded.length} cặp loại trừ đóng băng`)
  }
}

// C2 — tỉ lệ thật, cả hai theme.
for (const theme of ['light', 'dark']) {
  const palette = tokens.colors?.[theme] ?? {}
  let worst = { ratio: Infinity, label: '' }
  let themeBad = 0
  const pairs = asArray(cfg.pairs, 'contrast.pairs')
  for (const pair of pairs) {
    const fg = palette[pair.fg]
    const bg = palette[pair.bg]
    if (!fg || !bg) {
      fail(`cặp \`${pair.fg}\` × \`${pair.bg}\` trỏ tới token không tồn tại ở theme ${theme}`)
      themeBad += 1
      continue
    }
    // Sàn 3:1 chỉ dành cho chữ lớn, và một cặp muốn dùng nó phải NÊU TÊN token cỡ ≥ 24px.
    // Không có điều kiện này thì "large" trở thành cái van xả áp cho mọi cặp trượt.
    let floor = CONTRAST_FLOORS.normal
    if (pair.floor === 'large') {
      const t = tokens.typography?.[pair.token ?? '']
      const px = parseFloat(String(t?.fontSize ?? '0'))
      const bold = parseInt(String(t?.fontWeight ?? '400'), 10) >= 700
      const qualifies = px >= LARGE_TEXT_MIN_PX || (bold && px >= 18.66)
      if (!qualifies) {
        fail(
          `cặp \`${pair.fg}\` × \`${pair.bg}\` khai sàn "large" nhưng không nêu được token ` +
            `cỡ ≥ ${LARGE_TEXT_MIN_PX}px (đang là \`${pair.token ?? 'không khai'}\`)`,
        )
        themeBad += 1
      } else floor = CONTRAST_FLOORS.large
    }
    let r
    try {
      r = contrast(fg, bg)
    } catch (err) {
      fail(`cặp \`${pair.fg}\` × \`${pair.bg}\` [${theme}] không tính được: ${err.message}`)
      themeBad += 1
      continue
    }
    if (r < worst.ratio) worst = { ratio: r, label: `${pair.fg} × ${pair.bg}` }
    if (r < floor) {
      fail(`[${theme}] ${pair.fg} (${fg}) trên ${pair.bg} (${bg}) = ${r.toFixed(3)}:1 — dưới sàn ${floor}:1`)
      themeBad += 1
    }
  }
  if (themeBad === 0) {
    pass(`[${theme}] ${pairs.length} cặp đạt AA · thấp nhất ${worst.ratio.toFixed(3)}:1 (${worst.label})`)
  }
  cBad += themeBad
}

// C3 — màu đã loại phải VẮNG MẶT hoàn toàn khỏi bề mặt giao diện.
//
// ⚠️ `tokens.json` NẰM TRONG tầm quét ở đây, dù nó không phải tệp `.css`/`.ts`. Mệnh đề
// của AC3 là "vắng mặt hoàn toàn"; một màu đã loại quay lại làm GIÁ TRỊ của một token là
// đúng chỗ tệ nhất, và đó là chỗ duy nhất lượt quét cũ không nhìn thấy.
const escapeRe = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

/**
 * Che ĐÚNG khối `"bannedColorValues": { … }` trong văn bản `tokens.json`.
 *
 * Chính chỗ khai lệnh cấm là nơi duy nhất được phép viết ra giá trị bị cấm — nếu không
 * thì lệnh cấm tự làm mình đỏ. Đừng nới bằng cách bỏ `tokens.json` khỏi tầm quét:
 * một màu đã loại quay lại làm GIÁ TRỊ của một token là đúng chỗ tệ nhất, và đó là chỗ
 * duy nhất lượt quét cũ không nhìn thấy. Che một khối tên rõ ràng, không che cả tệp.
 */
function maskBannedDeclaration(text) {
  const key = '"bannedColorValues"'
  const at = text.indexOf(key)
  if (at === -1) return text
  const open = text.indexOf('{', at)
  if (open === -1) return text
  let depth = 0
  let end = open
  for (let i = open; i < text.length; i++) {
    if (text[i] === '{') depth += 1
    else if (text[i] === '}') {
      depth -= 1
      if (depth === 0) {
        end = i + 1
        break
      }
    }
  }
  const region = text.slice(at, end)
  return text.slice(0, at) + region.replace(/[^\n]/g, ' ') + text.slice(end)
}
const tokensScanText = maskBannedDeclaration(tokensText)

for (const value of EXPECTED_BANNED_VALUES) {
  const why = String(cfg.bannedColorValues?.[value] ?? '').trim()
  if (!why) {
    fail(`màu đã loại \`${value}\` không có lý do trong \`contrast.bannedColorValues\``)
    cBad += 1
  }
  const hits = []
  const targets = [
    ...parsed.map((p) => ({ file: p.file, text: p.text })),
    { file: TOKENS_PATH, text: tokensScanText },
  ]
  for (const t of targets) {
    const re = new RegExp(escapeRe(value), 'gi')
    let m
    while ((m = re.exec(t.text))) {
      hits.push(`${rel(t.file)}:${lineOf(t.text, m.index)}`)
      if (m.index === re.lastIndex) re.lastIndex += 1
    }
  }
  if (hits.length) {
    fail(`màu đã loại \`${value}\` CÓ MẶT — ${hits.join(' · ')}`)
    if (why) detail(why)
    cBad += hits.length
  } else pass(`màu đã loại \`${value}\` vắng mặt hoàn toàn (kể cả trong \`tokens.json\`)`)
}

// C4 — `ornament` và `tm-rule` không bao giờ là màu chữ.
const TEXT_COLOR_PROPS = new Set(['color', '-webkit-text-fill-color'])

/**
 * Miễn trừ CÓ TÊN **kèm tham số là tên token** — Story 2.2 · Quyết định #5 · AC10.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO THAM SỐ, TRONG KHI `aura-allow-opacity`/`aura-allow-z-index` KHÔNG CÓ
 * ═════════════════════════════════════════════════════════════════════════════════
 * Hai miễn trừ kia canh **một** thuộc tính, nên khai báo mà chúng che là đúng khai báo mà
 * người viết đang nhìn. Miễn trừ này canh **một tập token dùng chung**
 * (`EXPECTED_NEVER_TEXT` = `ornament` · `tm-rule`), và một dấu miễn trừ không tham số sẽ
 * cấp cho **cả hai** cùng lúc. `tokens.json:99` đặc tả ngoại lệ cho **đúng một** ký tự —
 * ranh giới câu `⏐` màu `ornament` — và `:100` nói thẳng rằng `tm-rule` **không** có vai
 * chữ ở bất kỳ theme nào. Một miễn trừ cấp nhầm là hai mệnh đề tương phản mất cùng lúc, im
 * lặng.
 *
 * ⚠️ Hai đường tắt bị CẤM đích danh, và cả hai đều "làm cổng xanh":
 *   ① gỡ `ornament` khỏi `EXPECTED_NEVER_TEXT` ⇒ mất luôn vế `tm-rule` dùng chung tập đó;
 *   ② khai một biến CSS cục bộ chép giá trị hex ⇒ đúng thứ AD-34 tồn tại để chặn.
 *
 * Khuôn: `/* aura-allow-never-text: <tên token> — <lý do> *​/` **ngay trên** khai báo (cùng
 * luật khoảng cách một dòng của [`exemptAt`], nên một dấu đặt ở một `color:` KHÁC không cấp
 * cho khai báo này).
 */
const neverTextExemptAt = (p, index, token) => {
  const line = lineOf(p.text, index)
  // `\\b` sau tên token: một miễn trừ cho `tm-rule` KHÔNG được khớp khi token là `tm`.
  // `\\S` cuối: tên token một mình chưa đủ — phải có LÝ DO viết ra sau nó.
  const re = new RegExp(`aura-allow-never-text\\s*:\\s*${escapeRe(token)}\\b\\s*\\S`)
  return p.comments.some((c) => re.test(c.text) && Math.abs(lineOf(p.text, c.index) - line) <= 1)
}

for (const token of EXPECTED_NEVER_TEXT) {
  // ⚠️ Đếm theo TỪNG token, không một biến chung: một bộ đếm dùng chung sẽ báo số miễn trừ
  // của `ornament` trong dòng kết luận của `tm-rule` — một con số đúng ở chỗ sai.
  let cExempt = 0
  const why = String(cfg.neverTextTokens?.[token] ?? '').trim()
  if (!why) {
    fail(`token \`${token}\` không có lý do trong \`contrast.neverTextTokens\``)
    cBad += 1
  }
  const candidates = allDecls.filter(
    (d) => TEXT_COLOR_PROPS.has(d.prop) && new RegExp(`--color-${escapeRe(token)}\\b`).test(d.value),
  )
  const hits = []
  for (const d of candidates) {
    const p = parsed.find((x) => x.file === d.file)
    if (p && neverTextExemptAt(p, d.index, token)) {
      pass(`${where(d)} — \`${token}\` làm màu chữ, có miễn trừ có tên cho ĐÚNG token này`)
      cExempt += 1
      continue
    }
    hits.push(d)
  }
  if (hits.length) {
    hits.forEach((d) => fail(`${where(d)} — \`${token}\` dùng làm màu chữ: \`${d.prop}: ${d.value}\``))
    if (why) detail(why)
    detail(
      `Nếu đây là ngoại lệ ĐÃ ĐẶC TẢ: thêm \`/* aura-allow-never-text: ${token} — <lý do> */\` ngay trên khai báo.`,
    )
    cBad += hits.length
  } else if (cExempt) {
    pass(`\`${token}\` sau \`color:\` chỉ ở ${cExempt} chỗ, và mỗi chỗ có miễn trừ có tên`)
  } else {
    pass(`\`${token}\` không xuất hiện sau \`color:\` ở bất kỳ đâu`)
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm D — lùi chữ bằng token, không bằng `opacity` (AC4)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// Luật màu của Kiểm B KHÔNG bắt được luật này, và đó là lý do nó là một phép kiểm riêng.
// `DESIGN.md §Opacity không được dùng để làm mờ chữ` có số đo: `opacity: 0.4` trên chữ
// `on-surface-variant` (5,2:1) ra màn hình ở ≈ 2,3:1 — kiểm token vẫn xanh, mắt vẫn
// không đọc được. Phát hiện gốc: kiểm toán bảng chờ Glossary 2026-08-03.
//
// 🔴 PHẠM VI ĐÃ NỚI RỘNG (Ice chốt 2026-08-03, lượt rà soát). Bản trước chỉ đỏ khi
// `opacity` nằm CÙNG KHỐI với `color:` — đúng chữ của Task 2, nhưng lượt rà soát chạy
// thật: `.dimmed { opacity: 0.4 }` trên một thẻ bọc mà chữ con kế thừa màu ⇒ exit 0. Bọc
// rồi làm mờ là cách làm thông thường hơn trong Vue, và AC4 nói "`opacity` … không áp cho
// chữ", không nói "cùng khối với `color:`". Nên: MỌI giá trị trung gian là FAIL, và nét
// với nền hợp lệ đi qua miễn trừ CÓ TÊN — đường thoát đã có sẵn và đã nghiệm thu.

/** `50%` là 0.5. `var(--x)` và `calc(…)` không chứng minh được ⇒ `NaN` ⇒ phải khai miễn trừ. */
function parseOpacity(v) {
  const s = stripImportant(v).toLowerCase()
  if (/^[+-]?(?:\d+\.?\d*|\.\d+)%$/.test(s)) return parseFloat(s) / 100
  if (/^[+-]?(?:\d+\.?\d*|\.\d+)$/.test(s)) return parseFloat(s)
  return NaN
}

let dBad = 0
let dChecked = 0
let dExempt = 0
for (const p of parsed) {
  for (const block of p.blocks) {
    for (const d of block.decls) {
      if (d.prop !== 'opacity') continue
      dChecked += 1
      const o = parseOpacity(d.value)
      if (o === 0 || o === 1) continue
      const line = lineOf(p.text, d.index)
      if (exemptAt(p, d.index, 'opacity')) {
        pass(`${rel(p.file)}:${line} — \`opacity: ${d.value}\`, có miễn trừ có tên`)
        dExempt += 1
        continue
      }
      const why = Number.isFinite(o)
        ? `\`opacity: ${d.value}\` là giá trị trung gian`
        : `\`opacity: ${d.value}\` không tĩnh — cổng không chứng minh được nó là 0 hay 1`
      fail(`${rel(p.file)}:${line} — ${why}`)
      detail('Lùi chữ bằng cách đổi sang `var(--color-on-surface-variant)`.')
      detail('Nếu đây thật sự là nét/nền: thêm `/* aura-allow-opacity: <lý do> */` ngay trên khai báo.')
      dBad += 1
    }
  }
}
if (dBad === 0) {
  pass(`không \`opacity\` trung gian nào trong \`src/**\` (${dChecked} khai báo đã soi, ${dExempt} có miễn trừ)`)
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm E — sàn giãn dòng 1.66 cho chữ có xuống dòng (AC5)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 AC5 KHÔNG được cài thành "mọi token họ `read` ≥ 1.66". `read-title` và
// `lookup-headword` đều họ `read` và đều ở 1.3 ĐÚNG ĐẶC TẢ — một cổng cài theo chữ của
// AC5 sẽ làm hai token đúng đỏ oan, và phản ứng tự nhiên là sửa bảng token cho khớp
// cổng: hỏng đúng chiều tệ nhất.
//
// Ranh giới thật (`DESIGN.md §Giãn dòng 1.66 là sàn cứng`) là chữ có CHẠY THÀNH ĐOẠN
// hay không — tiêu đề một dòng không có dòng dưới để dấu `ườ` chạm vào dấu `ộ`. Nên sàn
// áp theo cờ `wraps`, bất kể họ. Đó cũng là cách phủ được mệnh đề thứ hai của AC5.

const LINE_HEIGHT_FLOOR = 1.66
let eBad = 0
for (const [name, t] of entriesOf(tokens.typography)) {
  // ⚠️ Giãn dòng phải KHÔNG ĐƠN VỊ. `parseFloat('20px')` là 20, vượt sàn 1.66 trong khi
  // 20px trên chữ 16px là tỉ lệ 1,25 — con số qua sàn mà chữ vẫn chật.
  const lhRaw = String(t.lineHeight)
  if (!/^\d+(?:\.\d+)?$/.test(lhRaw)) {
    fail(`typography.${name}.lineHeight = \`${lhRaw}\` — phải là một tỉ lệ KHÔNG ĐƠN VỊ`)
    detail('Giãn dòng có đơn vị không co theo cỡ chữ, và `parseFloat` nuốt đơn vị nên sàn 1.66 mất nghĩa.')
    eBad += 1
    continue
  }
  if (typeof t.wraps !== 'boolean') {
    fail(`typography.${name} thiếu cờ \`wraps\` — không quyết được sàn giãn dòng`)
    detail('Mọi token mới phải trả lời: chuỗi này có bao giờ dài quá một dòng không?')
    eBad += 1
    continue
  }
  if (!t.wraps) continue
  if (!(parseFloat(lhRaw) >= LINE_HEIGHT_FLOOR)) {
    fail(`typography.${name} — \`wraps: true\` nhưng lineHeight ${lhRaw} < ${LINE_HEIGHT_FLOOR}`)
    eBad += 1
  }
}
if (eBad === 0) {
  const wrapping = entriesOf(tokens.typography).filter(([, t]) => t.wraps)
  const single = entriesOf(tokens.typography).filter(([, t]) => !t.wraps)
  pass(`${wrapping.length} token \`wraps: true\` đều ≥ ${LINE_HEIGHT_FLOOR}, và đều không đơn vị`)
  pass(`${single.length} token nhãn một dòng được ghìm dưới sàn hợp lệ: ${single.map(([n]) => n).join(', ')}`)
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm F — không elevation (AC7)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// Chiều sâu duy nhất của sản phẩm là SẮC ĐỘ (`surface-sunken`). Ngoại lệ duy nhất là
// bóng của chính cửa sổ ứng dụng, do hệ điều hành vẽ — không mã nào ở đây vẽ nó.
//
// ⚠️ `z-index` có miễn trừ CÓ TÊN, `box-shadow`/`text-shadow` thì không. Lý do bất đối
// xứng: bóng đổ là quyết định thị giác mà AC7 cấm thẳng, còn ngữ cảnh xếp lớp là nhu cầu
// CƠ HỌC — panel của Story 1.14, dropdown, tooltip và chính dockview đều cần. Không có
// đường thoát có tên thì cái `z-index` hợp lệ đầu tiên sẽ được "sửa" bằng cách xoá nó
// khỏi `BANNED_PROPS`, và hai lệnh cấm bóng đổ dùng chung tập đó mất theo.
const BANNED_PROPS = new Set(['box-shadow', 'text-shadow'])
const EXEMPTABLE_PROPS = new Map([['z-index', 'z-index']])
const BANNED_VALUE_RE =
  /\b(?:drop-shadow|linear-gradient|radial-gradient|conic-gradient|repeating-linear-gradient|repeating-radial-gradient|repeating-conic-gradient)\s*\(/
let fBad = 0
let fExempt = 0
for (const d of allDecls) {
  const p = parsed.find((x) => x.file === d.file)
  if (BANNED_PROPS.has(d.prop)) {
    fail(`${where(d)} — \`${d.prop}\` bị cấm (không bóng đổ, không lớp nổi)`)
    fBad += 1
  }
  if (EXEMPTABLE_PROPS.has(d.prop)) {
    if (p && exemptAt(p, d.index, EXEMPTABLE_PROPS.get(d.prop))) {
      pass(`${where(d)} — \`${d.prop}\` có miễn trừ có tên`)
      fExempt += 1
    } else {
      fail(`${where(d)} — \`${d.prop}\` bị cấm (không lớp nổi)`)
      detail(`Nếu đây là ngữ cảnh xếp lớp cơ học: thêm \`/* aura-allow-${EXEMPTABLE_PROPS.get(d.prop)}: <lý do> */\`.`)
      fBad += 1
    }
  }
  if (BANNED_VALUE_RE.test(d.value)) {
    fail(`${where(d)} — gradient/bóng trong \`${d.prop}: ${d.value}\``)
    fBad += 1
  }
}
if (fBad === 0) {
  pass(`không \`box-shadow\` · \`text-shadow\` · \`drop-shadow\` · gradient · \`z-index\` (${fExempt} miễn trừ có tên)`)
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm H — focus ring: `outline: none` CHỈ trên gốc `tabindex="-1"` (NFR17)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 ĐÓNG `deferred-work.md:140` — Story 1.14 · AC11.2.
//
// Ghi chú của chính cổng này *(và của `check-commands.mjs`)* đã nêu tên lỗ suốt bốn story:
// *"Một `*:focus { outline: none }` phá NFR17 mà vẫn qua được cả cổng này lẫn
// `check-commands.mjs`"* (§Trap 4 của Story 1.6). Nó là một dòng CSS, nó xoá đường đi bàn
// phím của MỌI nút và ô nhập trong sản phẩm, và không một phép kiểm nào nhìn thấy.
//
// ─────────────────────────────────────────────────────────────────────────────────
// LUẬT — hẹp có chủ ý, và không phải một danh sách cấm
// ─────────────────────────────────────────────────────────────────────────────────
// `outline: none` *(hoặc `outline: 0`, hoặc `outline-style: none`)* HỢP LỆ khi và chỉ khi
// selector của nó chọn **gốc `tabindex="-1"` của một chế độ hoặc một panel** — tức những
// phần tử KHÔNG nằm trong thứ tự Tab của trình duyệt và chỉ nhận focus qua `el.focus()`
// của `focus.ts`. Vẽ một vòng focus quanh cả một chế độ là nhiễu thị giác cho một lượt dời
// focus mà chính ứng dụng vừa thực hiện.
//
// Mọi ca khác đi qua **miễn trừ CÓ TÊN** `/* aura-allow-outline-none: <lý do> */`, cùng
// khuôn `aura-allow-z-index` mà Kiểm F đã dùng và đã nghiệm thu.
//
// ⚠️ Cổng đọc SELECTOR, không đọc HTML — nó không chứng minh được rằng phần tử khớp
// selector đó THẬT SỰ mang `tabindex="-1"`. Nó chứng minh được điều quan trọng hơn và
// kiểm được: selector không quét rộng. `*:focus`, `:focus`, `button:focus`,
// `.panel *:focus` đều đỏ. Giới hạn này in ra ở cuối lượt chạy.

/** Lớp gốc của chế độ/panel — chúng mang `tabindex="-1"` và không vào thứ tự Tab. */
const FOCUS_ROOT_CLASSES = ['.mode', '.panel', '.dock']
const OUTLINE_OFF_RE = /^(?:none|0|0px)$/

/** Selector có chọn ĐÚNG một gốc chế độ/panel, không quét rộng hơn? */
function isFocusRootSelector(prelude) {
  const parts = prelude
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s !== '')
  if (parts.length === 0) return false
  return parts.every((sel) => {
    // Bộ chọn hậu duệ / anh em: `.panel *:focus`, `.mode > button:focus` — chúng chạm
    // tới phần tử CON, và con thì có nút, có ô nhập, có tab. Đó là đúng ca §Trap 4.
    if (/[\s>+~]/.test(sel)) return false
    if (!sel.endsWith(':focus') && !sel.endsWith(':focus-visible')) return false
    const base = sel.replace(/:focus(-visible)?$/, '')
    return FOCUS_ROOT_CLASSES.includes(base)
  })
}

let hBad = 0
let hExempt = 0
let hOk = 0
for (const p of parsed) {
  for (const block of p.blocks) {
    for (const d of block.decls) {
      const isOutlineOff =
        (d.prop === 'outline' || d.prop === 'outline-style') &&
        OUTLINE_OFF_RE.test(stripImportant(d.value).toLowerCase())
      if (!isOutlineOff) continue
      const line = lineOf(p.text, d.index)
      if (isFocusRootSelector(block.prelude)) {
        hOk += 1
        continue
      }
      if (exemptAt(p, d.index, 'outline-none')) {
        pass(`${rel(p.file)}:${line} — \`${d.prop}: ${d.value}\` có miễn trừ có tên`)
        hExempt += 1
        continue
      }
      fail(`${rel(p.file)}:${line} — \`${d.prop}: ${d.value}\` trên \`${block.prelude}\``)
      detail(`Chỉ ${FOCUS_ROOT_CLASSES.join(' · ')} ở dạng \`<lớp>:focus\` được tắt focus ring.`)
      detail('Nút, ô nhập và tab PHẢI giữ vòng focus của trình duyệt — đó là nửa còn lại của NFR17.')
      detail('Nếu đây thật sự là một gốc `tabindex="-1"`: thêm `/* aura-allow-outline-none: <lý do> */`.')
      hBad += 1
    }
  }
}
if (hBad === 0) {
  pass(
    `${hOk} lượt tắt focus ring, tất cả trên gốc chế độ/panel (${hExempt} miễn trừ có tên) — ` +
      'không `*:focus`, không bộ chọn hậu duệ',
  )
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('\nKiểm G — phân tách panel ĐẢO NGƯỢC giữa hai theme (AC6)')
// ═════════════════════════════════════════════════════════════════════════════════
//
// Không thống nhất hai theme về một cách làm. `outline #3b382f` trên `surface #26241f`
// chỉ đạt 1,32:1 — gần như vô hình. Bê cách của theme sáng sang theme tối làm ba panel
// chìm thành một khối nâu.
//
// ⚠️ Giới hạn của phép kiểm này, ghi thẳng: nó nghiệm thu Ở TẦNG TOKEN. Không panel nào
// tồn tại hôm nay (Story 1.14 mới dựng), nên "nhìn thấy khe 2px" là chưa đo được.

const PX_RE = /^(\d+(?:\.\d+)?)px$/
const px = (v) => (PX_RE.test(String(v)) ? parseFloat(String(v)) : NaN)
const sepL = tokens.panelSeparator?.light
const sepD = tokens.panelSeparator?.dark
let gBad = 0
if (!sepL || !sepD) {
  fail('`panelSeparator` thiếu một trong hai theme')
  gBad += 1
} else {
  if (sepL.mechanism === sepD.mechanism) {
    fail(`hai theme khai CÙNG một cơ chế \`${sepL.mechanism}\` — AC6 đòi hai cơ chế khác nhau`)
    gBad += 1
  }
  if (sepL.mechanism !== 'rule') {
    fail(`theme sáng phải phân tách bằng NÉT (\`rule\`), đang là \`${sepL.mechanism}\``)
    gBad += 1
  }
  if (sepD.mechanism !== 'gap') {
    fail(`theme tối phải phân tách bằng KHE (\`gap\`), đang là \`${sepD.mechanism}\``)
    gBad += 1
  }
  // ⚠️ Bốn trường, không phải hai — và ĐƠN VỊ được kiểm. Bản trước chỉ soi `light.borderWidth`
  // và `dark.gap` bằng `parseFloat`, nên `light.gap: 12px` (theme sáng vừa có nét vừa có khe,
  // tức hai cơ chế cộng lại) và `dark.gap: 2rem` (`parseFloat` cho 2, qua) đều lọt.
  if (px(sepL.borderWidth) !== 1) {
    fail(`theme sáng phải là đường kẻ 1px, đang là \`${sepL.borderWidth}\``)
    gBad += 1
  }
  if (px(sepL.gap) !== 0) {
    fail(`theme sáng phân tách bằng NÉT nên khe phải là 0px, đang là \`${sepL.gap}\``)
    gBad += 1
  }
  if (sepL.borderColor !== 'outline') {
    fail(`đường kẻ theme sáng phải dùng token \`outline\`, đang là \`${sepL.borderColor}\``)
    gBad += 1
  }
  if (px(sepD.gap) !== 2) {
    fail(`khe theme tối phải là 2px để \`background\` lộ ra, đang là \`${sepD.gap}\``)
    gBad += 1
  }
  if (px(sepD.borderWidth) !== 0) {
    fail(`theme tối phân tách bằng KHE nên KHÔNG có đường kẻ, đang là \`${sepD.borderWidth}\``)
    gBad += 1
  }
  if (sepD.borderColor !== null) {
    fail(`theme tối không có đường kẻ nên \`borderColor\` phải là \`null\`, đang là \`${sepD.borderColor}\``)
    gBad += 1
  }
  if (String(tokens.rounded?.[sepD.radius]) !== '3px') {
    fail(`panel theme tối phải bo 3px (\`rounded.DEFAULT\`), đang trỏ \`${sepD.radius}\``)
    gBad += 1
  }
  if (gBad === 0) {
    pass(`sáng: nét ${sepL.borderWidth} \`${sepL.borderColor}\`, khe ${sepL.gap}`)
    pass(`tối: khe ${sepD.gap} lộ \`background\`, panel bo ${tokens.rounded?.[sepD.radius]}, không đường kẻ`)
    pass('hai cơ chế khác nhau — `rule` ≠ `gap`')
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
console.log('')
if (skippedLinks.length) {
  console.log(`\x1b[33mĐã BỎ QUA ${skippedLinks.length} symlink trong cây nguồn:\x1b[0m ${skippedLinks.join(' · ')}`)
  console.log('')
}
if (failures !== 0) {
  console.log(`\x1b[31m${failures} vi phạm.\x1b[0m`)
  process.exit(1)
}
console.log('\x1b[32mTất cả phép kiểm token đạt.\x1b[0m')
console.log('')
console.log(`Tầm quét: ${files.length} tệp (${componentFiles.length} component) · ${allDecls.length} khai báo CSS.`)
console.log('')
console.log('Ghi chú cho người rà soát — bốn giới hạn, ghi thẳng thay vì để người sau tự phát hiện:')
console.log('  1. AC6 nghiệm thu Ở TẦNG TOKEN. Không panel nào tồn tại hôm nay (Story 1.14).')
console.log('     Kiểm G chứng minh hai cơ chế đã khai và không bị thống nhất; nó KHÔNG chứng')
console.log('     minh khe 2px hiện ra đúng trên màn hình.')
console.log('  2. Kiểm C kiểm DANH SÁCH ĐÃ KHAI, không phải những cặp tình cờ tồn tại trong mã.')
console.log('     Khi Story 1.14 dựng panel thật, cặp mới phải được thêm vào `contrast.pairs` —')
console.log('     phép kiểm đầy đủ (C1) là thứ bắt việc quên thêm.')
console.log('  3. TÁM deviation khỏi bảng DESIGN.md đang được áp — ba cái Ice PHÊ CHUẨN 2026-08-03,')
console.log('     cái thứ tư (`typography.ui-md-strong`, token NGOÀI bảng) là Story 1.14 · AC10,')
console.log('     cái thứ năm (`typography.source-latin`, token NGOÀI bảng) là Story 1.16 · QĐ #6,')
console.log('     cái thứ sáu (`typography.ui-md-wrap`, token NGOÀI bảng) là Story 1.17 · QĐ #7,')
console.log('     hai cái cuối (`source-hanviet.fontSize` 12,5→14,5px · `.fontStyle` nghiêng→thường)')
console.log('     là Ice chốt trực tiếp 2026-08-07 — âm Hán Việt quá nhỏ và khó đọc khi in nghiêng.')
console.log('     `DESIGN.md` chưa được sửa cho khớp — đó là một lượt riêng của Ice, không phải')
console.log('     của dev. Xem `deviations` trong `tokens.json`.')
console.log('  4. Cờ `wraps` là mệnh đề về NỘI DUNG sẽ chạy qua token; không phép kiểm tĩnh nào')
console.log('     phân xử được nó khi chưa có component. Cổng chỉ bắt được việc THIẾU cờ.')
process.exit(0)
