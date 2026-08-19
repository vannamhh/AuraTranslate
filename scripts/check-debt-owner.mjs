#!/usr/bin/env node
/**
 * Cổng SỔ NỢ — Story 2.13, Quyết định #3 đường (a) (Ice ký 2026-08-19).
 *
 * Cưỡng chế đúng MỘT luật, đã sống ở `project-context.md:447-448` từ trước Epic 1 và bị phá
 * **187 lần** tính tới `4b30199`, lúc story này bắt đầu — đo bằng CHÍNH lệnh này
 * (`--file <bản cũ> --report`), nên nó tái lập được.
 * 🔵 2026-08-19: con số `217` ở bản đầu của dòng này SAI — nó đến từ một bộ đếm tạm nhận
 * *"đóng"* bằng nguyên văn `ĐÃ ĐÓNG`, nên ~60 mục đóng bằng `→ ✅ ĐÓNG`/`ĐÃ SOÁT`/`ĐÃ KÝ`/
 * `ĐÃ GỠ` bị đếm thành *"mở"*. Một công cụ **không** được đóng đinh một con số mà chính nó
 * không tái lập được. Bộ đếm gốc của story mắc cùng lỗi: nó khai `37 đóng` ở chỗ lệnh này
 * cho `97`, và `199` mồ côi ở chỗ lệnh này cho `187`. Số ở đây đo ở Task 0.1:
 *
 *   "Mọi thứ không nghiệm thu được ở story hiện tại đi vào `deferred-work.md`, KÈM MỘT CHỦ.
 *    Không có mục nào mồ côi."
 *
 * Kiểm A — mọi mục MỞ trong `deferred-work.md` phải mang một `Chủ:` THẬT (không phải một
 *          dòng nói "chưa có chủ").
 * Kiểm B — TỰ KIỂM: chứng minh Kiểm A đỏ được, và không đỏ oan — trên chính hai lớp bẫy
 *          đã đo được lúc dựng bộ đếm này (Task 1.2 của story): một ngày bị đọc nhầm thành
 *          số Epic, và `Chủ:` viết nhiều dạng.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO LUẬT NHẬN DIỆN "MỘT MỤC" VÀ "CÓ CHỦ" LÀ DỤNG CỤ ĐO CỦA AC1 — VÀ CHƯA TỪNG ĐƯỢC
 * VIẾT RA TRƯỚC BỘ ĐẾM NÀY (Task 0.1 của Story 2.13)
 * ═════════════════════════════════════════════════════════════════════════════════
 * `deferred-work.md` là văn xuôi tiếng Việt, không phải dữ liệu có cấu trúc. Ba lượt đo thủ
 * công/bán tự động trên CÙNG một HEAD (`4b30199`) ra BA con số mồ côi khác nhau tuỳ luật nhận
 * diện "có chủ": 213 (rộng, nhận 5 dạng: `Chủ:` · `Chủ vẫn là` · `chủ sở hữu` · `giao cho` ·
 * `thuộc Story/Epic`) hay 198 (một luật khác). Story 2.13 CHỐT luật HẸP — chỉ `Chủ:` — vì luật
 * rộng đoán ý văn xuôi nên trượt theo thời gian. Bộ đếm này là LẦN ĐẦU luật đó được viết thành
 * mã thay vì sống trong đầu người đo.
 *
 * ⚠️ HAI BẪY ĐO ĐƯỢC làm bộ đếm cũ (thủ công, lúc soạn story) sai:
 *   1. Một regex vơ số 4 chữ số đọc nhầm ngày `2026-08-17` thành `Epic 2026`. ⇒ Bộ đếm này
 *      KHÔNG suy Epic từ số trong văn bản ở bất kỳ đâu — bốn con số AC5 đòi (mở · 🟡 · ✅ ·
 *      mở-không-chủ) không cần phân theo Epic, nên lớp bẫy đó bị loại bằng cách không dựng
 *      cơ chế sinh ra nó, không phải bằng cách vá một regex.
 *   2. `Chủ:` sống dưới nhiều dạng: `Chủ: chưa gán` (24 lần) và `Chủ: không ai` (1 lần) đều
 *      CHỨA literal `Chủ:` nhưng đều nói "chưa có chủ" — một `.includes('Chủ:')` ngây thơ sẽ
 *      đếm sai 25 mục thành "có chủ". Kiểm A dưới đây đọc NỘI DUNG sau `Chủ:`, không chỉ đếm
 *      sự có mặt của chuỗi.
 *
 * ⚠️ GIỚI HẠN THẬT, ghi ra thay vì giấu:
 *   - Trạng thái ĐÓNG một mục theo đúng luật ở `project-context.md:449` là "nối tiếp `→ …`
 *     NGAY TRONG mục đó". Nếu ai đó đóng một mục bằng cách viết một mục MỚI Ở CHỖ KHÁC (đã đo
 *     được ít nhất một ca thật: `deferred-work.md:303` bị "đóng" bởi một mục mới ở `:429` mà
 *     không nối tiếp tại `:303`), bộ đếm này sẽ KHÔNG thấy việc đóng đó — mục gốc vẫn đếm theo
 *     trạng thái cũ. Đây là lựa chọn CÓ CHỦ Ý: thà báo "còn mở/nửa" khi thật ra đã đóng ở chỗ
 *     khác (an toàn — không bao giờ tự nhận đạt bằng suy luận) còn hơn đoán một tham chiếu chéo
 *     bằng văn xuôi và đôi khi đoán sai theo chiều XẤU (báo đóng khi chưa đóng).
 *   - Chỉ trạng thái đặt trên (a) chính dòng bullet cấp một, hoặc (b) một dòng tiếp nối bắt đầu
 *     bằng `→` NẰM TRONG mục đó, được đọc. Một trạng thái ghi trên một bullet cấp hai (`  - `)
 *     lồng bên trong mục không đổi trạng thái của mục cha — cùng lý do: đoán cấu trúc lồng bằng
 *     văn xuôi tự do sẽ trượt.
 *
 * Chạy:
 *   npm run check:debt-owner            — Kiểm A + B, mã thoát là phán quyết (cổng)
 *   node scripts/check-debt-owner.mjs --report   — in bốn con số AC5, KHÔNG đổi mã thoát theo A
 *   node scripts/check-debt-owner.mjs --surface  — phân loại các mục mồ côi theo bề mặt (Q.định #5)
 *   node scripts/check-debt-owner.mjs --list     — liệt kê từng mục mồ côi kèm dòng
 */
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, isAbsolute, join } from 'node:path'

const REPO_ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')

/**
 * 🔵 **CODE REVIEW 2026-08-19 — `--file` là vế thứ HAI của AC5, và bản đầu chỉ có vế thứ nhất.**
 *
 * AC5 đòi hai bảng từ **một** lệnh: *"ra đúng bảng của §ĐỌC TRƯỚC **trước** lượt này, và bảng
 * mới **sau** lượt này"*. Một `DEBT_PATH` viết cứng chỉ trả lời được nửa **sau** — không đường
 * nào chĩa nó vào một bản lịch sử, nên vế *"dựng lại bảng trước"* **không nghiệm thu được**, và
 * đúng con số `199` của story vẫn là một con số không ai kiểm lại được.
 *
 * ⇒ `--file <đường dẫn>` để đo một bản bất kỳ, ví dụ:
 *   `git show 4b30199:_bmad-output/implementation-artifacts/deferred-work.md > /tmp/cu.md`
 *   `node scripts/check-debt-owner.mjs --file /tmp/cu.md --report`
 *
 * ⚠️ **Đường mặc định KHÔNG đổi**, có chủ ý: cổng chạy trong `pre-push`/CI phải phán quyết trên
 * sổ THẬT, và một cờ đường dẫn không được biến nó thành một cổng chĩa đi đâu cũng được.
 * `--file` chỉ có nghĩa cho `--report`/`--list`/`--surface`; Kiểm A vẫn đọc sổ thật.
 */
/**
 * 🔵 **HÀM THUẦN, có chủ ý — lượt rà 2026-08-19 (tầng Verification Gap).**
 *
 * Bản đầu phân giải `--file` **tại cấp module**, nên nó là một tác dụng lề: không phép kiểm nào
 * gọi được nó, và `pre-push`/CI chỉ chạy đường **mặc định** *(không cờ)* nên nhánh `--file`
 * **không bao giờ được thực thi trong đường nghiệm thu của kho**. ⇒ Ai đó đảo `!==` thành `===`
 * ở điều kiện dưới đây thì `--file` chết trong im lặng, vế *"TRƯỚC"* của AC5 mất khả năng tái
 * lập **lần thứ hai**, và không cổng nào đỏ. Đúng bài học mà chính lượt rà này vừa ghi cho bản
 * trước của tệp này.
 *
 * ⇒ Tách thành hàm thuần nhận `argv`/`repoRoot`/`cwd` qua **tham số**, và Kiểm B gọi CHÍNH nó.
 *
 * @param {string[]} argv tham số sau `node <script>`
 * @param {string} repoRoot
 * @param {string} cwd
 */
export function resolveDebtPath(argv, repoRoot, cwd) {
  const i = argv.indexOf('--file')
  const val = i !== -1 ? argv[i + 1] : undefined
  if (i !== -1 && val && !val.startsWith('--')) {
    return isAbsolute(val) ? val : join(cwd, val)
  }
  return join(repoRoot, '_bmad-output', 'implementation-artifacts', 'deferred-work.md')
}

/** Sổ nợ THẬT — đường duy nhất Kiểm A được phép đọc. */
const REAL_DEBT_PATH = join(REPO_ROOT, '_bmad-output', 'implementation-artifacts', 'deferred-work.md')

/**
 * 🔴 **LƯỢT RÀ 2026-08-19 — BẢN VÁ `--file` ĐẦU CỦA TÔI TỰ PHÁ CHÍNH LỜI HỨA CỦA NÓ.**
 *
 * Chú thích của `resolveDebtPath` viết *"đường mặc định KHÔNG đổi… Kiểm A vẫn đọc sổ THẬT"*.
 * Mã thì tính **một** `DEBT_PATH` rồi dùng cho **mọi** nhánh. Tầng Blind Hunter tái lập được:
 * `node scripts/check-debt-owner.mjs --file <tệp giả một dòng>` chạy **Kiểm A** trên tệp giả và
 * **exit 0** — tức một cổng CHẶN bị vô hiệu hoá bằng một cờ dòng lệnh, và nó báo *"tất cả đạt"*.
 *
 * ⇒ Đây đúng lớp lỗi `project-context.md` xếp cao nhất: *vi phạm được mà không cổng nào đỏ*, và
 * chú thích nói một đằng mã làm một nẻo. Hai đường tách hẳn:
 *   · **Kiểm A** *(phán quyết, chạy trong `pre-push`/CI)* đọc [`REAL_DEBT_PATH`], **luôn luôn**;
 *   · `--report`/`--list`/`--surface` *(chỉ ĐỌC, không phán quyết)* mới đọc đường của `--file`.
 *
 * ⚠️ Và `--file` **không có** một trong ba cờ đọc là một lượt DÙNG SAI, không một mặc định thầm
 * lặng: nó `abort()` thay vì chạy Kiểm A trên tệp lạ.
 */
const REPORT_PATH = resolveDebtPath(process.argv.slice(2), REPO_ROOT, process.cwd())
const READ_ONLY_FLAGS = ['--report', '--list', '--surface']
const argvTop = process.argv.slice(2)
if (argvTop.includes('--file') && !READ_ONLY_FLAGS.some((f) => argvTop.includes(f))) {
  abort(
    '`--file` dùng SAI',
    new Error(
      '`--file` chi co nghia cho `--report` / `--list` / `--surface`. Khong co mot trong ba co do,\n' +
        'Kiem A se phan quyet — va no BAT BUOC doc so no THAT, khong doc mot tep tuy y.',
    ),
  )
}
const DEBT_PATH = READ_ONLY_FLAGS.some((f) => argvTop.includes(f)) ? REPORT_PATH : REAL_DEBT_PATH

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
// Bộ phân tích — biến một tệp văn xuôi thành danh sách MỤC có trạng thái + chủ
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Luật nhận diện MỘT MỤC (Task 0.1 của Story 2.13, tái lập trên `4b30199`: 448 mục == số
 * dòng khớp `^- `): một dòng bắt đầu bằng `- ` ở CỘT 0 (không thụt lề) mở một mục mới; mục
 * kéo dài tới dòng `^- ` hoặc `^## ` kế tiếp, hoặc hết tệp.
 */
const ITEM_START_RE = /^- /
const BLOCK_START_RE = /^## /

/** Cụm phủ định — `Chủ:` chứa các cụm này KHÔNG phải một chủ thật. Đo được 2026-08-19: 24 ca
 *  `chưa gán`, 1 ca `không ai`, 1 ca `chưa cần` (trong một trích dẫn lịch sử — mục đó có MỘT
 *  `Chủ:` khác, thật, ở chỗ khác — xem `hasPositive` bên dưới, "có MỘT chủ thật ⇒ có chủ"). */
const NEGATIVE_OWNER_RE = /^(chưa gán|chưa có|chưa cần|không ai|chưa chốt|trống)\b/iu

/** Tìm mọi `Chủ:` (kể cả bọc `**`) trong văn bản một mục; trả về có ít nhất MỘT chủ THẬT. */
function detectOwner(text) {
  // 🔵 **LƯỢT RÀ 2026-08-19 — `i`: sổ đang có `CHỦ:` (2 ca) và `chủ:` (4 ca).** Không có cờ này,
  // một mục MỞ viết hoa bị chấm mồ côi ⇒ cổng ĐỎ OAN, và Task 4.3 nói một cổng đỏ oan trên sổ
  // nợ sẽ bị TẮT — lúc đó tệ hơn không có cổng.
  const re = /chủ:\**\s*/giu
  let m
  let any = false
  let positive = false
  while ((m = re.exec(text))) {
    any = true
    const after = text.slice(m.index + m[0].length, m.index + m[0].length + 60)
    if (!NEGATIVE_OWNER_RE.test(after)) positive = true
  }
  return { any, positive }
}

/** Emoji dẫn đầu của chính dòng bullet (sau khi gỡ `- ` và bọc `**`/`*`/`🔴`/`⚠️`/`🔵` mở đầu
 *  không mang nghĩa trạng thái — chỉ ✅/🟡 dẫn đầu MỚI là trạng thái tự khai của mục). */
function leadingStatus(firstLine) {
  // 🔵 **LƯỢT RÀ 2026-08-19 (tầng Edge Case) — CHÚ THÍCH TRÊN HỨA GỠ 🔴/⚠️/🔵, MÃ THÌ KHÔNG.**
  // Bản đầu chỉ gỡ `**`. Nên `- 🔴 ✅ ĐÃ ĐÓNG …` đọc ra `open`: một mục **tự khai đã đóng** bị
  // đếm thành mở, và nếu nó không có `Chủ:` thì cổng ĐỎ OAN — đúng điều Task 4.3 của story cấm.
  // ⇒ Sửa MÃ cho khớp lời hứa, không hạ lời hứa cho khớp mã. Vòng `while` vì một mục có thể mang
  // nhiều lớp bọc (`- **🔴 ✅ …`).
  let body = firstLine.replace(/^- /, '')
  for (let k = 0; k < 6; k += 1) {
    const truoc = body
    body = body.replace(/^\**/, '').replace(/^(?:🔴|⚠️|⚠|🔵|📝)\s*/u, '')
    if (body === truoc) break
  }
  body = body.trim()
  if (body.startsWith('✅')) return 'closed'
  if (body.startsWith('🟡')) return 'half'
  return null
}

/** Trạng thái ghi trên một dòng tiếp nối bắt đầu bằng `→` (đúng luật đóng mục ở
 *  `project-context.md:449`). Luật đo được 2026-08-19 (Task 1 của Story 2.13):
 *   - `→ ✅ …`                         ⇒ closed  (mọi ca `→ ✅` đã đọc đều là đóng thật, dù
 *                                        văn bản dùng "ĐÃ ĐÓNG"/"ĐÓNG"/"ĐÃ SOÁT"/"ĐÃ KÝ"/"ĐÃ GỠ")
 *   - `→ 🟡 …`                         ⇒ half    (mọi ca `→ 🟡` đã đọc đều là nửa thật)
 *   - `→ ⚠️ …một nửa|một phần…`        ⇒ half    (⚠️ dùng cho NHIỀU việc khác — bẫy, giới hạn,
 *                                        "CHƯA đóng" — nên chỉ đếm khi văn bản tự nói nửa/phần)
 *   - `→ KHÔNG LÀM <ngày> (`           ⇒ decided (trạng thái thứ tư, Quyết định #4 Ice ký) */
function continuationStatus(line) {
  const m = /^\s*→\s*(.*)$/.exec(line)
  if (!m) return null
  const rest = m[1]
  // 🔴 Bẫy đo được 2026-08-19 (Task 1 của Story 2.13): `deferred-work.md:20` mở bằng
  // `→ ✅ **Phần quyết định đã đóng…**` nhưng CHÍNH DÒNG ĐÓ kết bằng "**Phần phép đo vẫn
  // mở**" — một dòng mang HAI tín hiệu trái nhau. Tin ✅ mù quáng sẽ tự nhận đạt bằng suy
  // luận đúng thứ AC2 cấm. ⇒ nếu dòng đóng còn tự nói "vẫn mở/còn mở", hạ xuống `half` —
  // an toàn hơn `closed`, và đúng nghĩa: một phần đã quyết, một phần chưa xong.
  if (/vẫn (còn )?mở|còn mở/i.test(rest)) return 'half'
  if (/^\**\s*✅/.test(rest)) return 'closed'
  if (/^\**\s*🟡/.test(rest)) return 'half'
  if (/^\**\s*⚠️/.test(rest) && /một nửa|một phần/i.test(rest)) return 'half'
  if (/^\**\s*KHÔNG LÀM\s+\d{4}-\d{2}-\d{2}\s*\(/.test(rest)) return 'decided'
  return null
}

/** Phân tích toàn bộ tệp thành danh sách mục `{ line, status, owner, text }`. */
function parseItems(fileText) {
  const lines = fileText.split('\n')
  const items = []
  let cur = null
  const flush = () => {
    if (cur) items.push(cur)
    cur = null
  }
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    if (BLOCK_START_RE.test(line)) {
      flush()
      continue
    }
    // 🔵 **LƯỢT RÀ 2026-08-19 — `---` KẾT THÚC MỘT MỤC, đúng như `## ` kết thúc nó.**
    //
    // 🔴 Bản đầu chỉ coi `- ` và `## ` là ranh giới, nên **mọi** dòng khác bị nối vào mục đang mở
    // — kể cả một dòng phân cách `---`. Hệ quả đo được trong CHÍNH lượt phân loại này: lượt gắn
    // thẻ hàng loạt dán `**(Chủ: …)**` lên **hai** dòng `---` *(`deferred-work.md:494` và `:1057`)*,
    // và cổng **vẫn xanh** vì nó đọc thẻ ấy như chủ của mục phía trên. Một thẻ chủ nằm ngoài mọi
    // mục mà vẫn được tính là chủ ⇒ AC1 xanh trên một sổ có hai chỗ hỏng hình dạng markdown
    // *(dòng `---` mang nội dung không còn render `<hr>`)*.
    // ⇒ Cắt ở `---`: thẻ đặt sai chỗ **mất hiệu lực**, mục phía trên thành mồ côi, và cổng ĐỎ —
    // tức lớp lỗi này nay có người canh thay vì đi qua im lặng.
    if (/^-{3,}\s*$/.test(line) || /^-{3,}\s+\S/.test(line)) {
      flush()
      continue
    }
    if (ITEM_START_RE.test(line)) {
      flush()
      cur = { line: i + 1, text: line, status: leadingStatus(line) ?? 'open' }
      continue
    }
    if (cur) {
      cur.text += '\n' + line
      const s = continuationStatus(line)
      if (s) cur.status = s // dòng SAU thắng dòng TRƯỚC — trạng thái mới nhất theo thứ tự đọc
    }
  }
  flush()
  for (const it of items) {
    const { any, positive } = detectOwner(it.text)
    it.hasOwnerMention = any
    it.hasOwner = positive
  }
  return items
}

/** Bốn con số AC5 đòi. */
function summarize(items) {
  const open = items.filter((i) => i.status === 'open')
  const half = items.filter((i) => i.status === 'half')
  const closed = items.filter((i) => i.status === 'closed')
  const decided = items.filter((i) => i.status === 'decided')
  const orphans = open.filter((i) => !i.hasOwner)
  return {
    total: items.length,
    open: open.length,
    half: half.length,
    closed: closed.length,
    decided: decided.length,
    orphans,
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Kiểm B — TỰ KIỂM: chứng minh Kiểm A đỏ được, và không đỏ oan
// ═════════════════════════════════════════════════════════════════════════════════
/** [tên ca, mảnh văn bản MÔ PHỎNG MỘT MỤC (bắt đầu bằng `- `), mong: mồ côi hay không] */
const SELFTEST_CASES = [
  ['mục mở, không Chủ: nào — MỒ CÔI thật', '- Một việc chưa xong, không ai nhận.', true],
  ['mục mở, có Chủ: thật', '- Một việc chưa xong. **Chủ: Story 3.1.**', false],
  [
    '`Chủ: chưa gán` — PHỦ ĐỊNH, vẫn là mồ côi',
    '- Một việc chưa xong. **Chủ: chưa gán.**',
    true,
  ],
  ['`Chủ: không ai` — PHỦ ĐỊNH, vẫn là mồ côi', '- Một việc. **Chủ: không ai — luật đọc.**', true],
  [
    'đặt lại chủ: trích dẫn phủ định CŨ + một Chủ: thật MỚI trong cùng mục ⇒ có chủ',
    '- Việc X. 🔵 Bản đầu ghi *"Chủ: chưa cần"* — nay xét lại. **Chủ: Ice**',
    false,
  ],
  [
    'ngày bị đọc nhầm thành Epic — bộ đếm này KHÔNG suy Epic từ số, nên không có gì để trượt',
    '- Việc xảy ra ngày 2026-08-17, không phải Epic 2026. **Chủ: chưa gán.**',
    true,
  ],
  [
    '5 dạng "chủ" RỘNG bị luật HẸP loại — vẫn đếm là mồ côi theo luật đã ký',
    '- Việc Y. Chủ vẫn là đội cũ; chủ sở hữu là Story 4; giao cho Ice; thuộc Story 5.',
    true,
  ],
  [
    'đóng bằng nối tiếp → ✅, dù văn bản không có chữ "ĐÃ ĐÓNG"',
    '- Việc Z, không Chủ: nào ghi.\n  → ✅ **ĐÃ SOÁT 2026-08-06.** Xong.',
    false, // status = closed ⇒ không tính vào "mở", nên không mồ côi dù không có Chủ:
  ],
  [
    'đóng một nửa bằng → 🟡, không cần chữ "một nửa"',
    '- Việc W.\n  → 🟡 **Trạng thái sau Story 1.9: VẪN CHƯA ĐÓNG.**',
    false, // status = half ⇒ loại khỏi phép đếm mồ côi (AC1 chỉ nói "MỞ")
  ],
  [
    '⚠️ KHÔNG kèm "một nửa/phần" ⇒ KHÔNG phải trạng thái nửa — mục vẫn MỞ',
    '- Việc V, không chủ nào ghi.\n  → ⚠️ **VẪN CHƯA ĐÓNG sau Story 1.15 — ghi thẳng.**',
    true, // ⚠️ không tự đóng gì — mục vẫn mở, vẫn mồ côi
  ],
  [
    'dòng ✅ tự mâu thuẫn — "vẫn mở" trong CHÍNH dòng đóng ⇒ hạ xuống half, không tự nhận đóng',
    '- Việc S, không chủ nào ghi.\n  → ✅ **Phần quyết định đã đóng.** Phần phép đo **vẫn mở**.',
    false, // status = half (KHÔNG closed, KHÔNG open) ⇒ loại khỏi phép đếm mồ côi, đúng AC1 ("MỞ")
  ],
  [
    'trạng thái thứ tư — → KHÔNG LÀM <ngày> (…) ⇒ decided, không còn "mở"',
    '- Việc U, không Chủ: nào ghi.\n  → KHÔNG LÀM 2026-08-19 (Story 2.13) — lý do thật.',
    false,
  ],
  [
    'bullet cấp hai (`  - 🟡`) lồng bên trong KHÔNG đổi trạng thái mục cha',
    '- Việc T, không chủ nào ghi ở mục cha.\n  - 🟡 một ghi chú con, không phải trạng thái cha.',
    true, // mục cha vẫn "open" — bullet cấp hai không phải dòng `→`
  ],
]

function runSelftest() {
  console.log('Kiểm B — TỰ KIỂM: chứng minh Kiểm A đỏ được, và không đỏ oan\n')
  let bad = 0
  for (const [name, fragment, expectOrphan] of SELFTEST_CASES) {
    const items = parseItems(fragment)
    if (items.length !== 1) {
      fail(`tự kiểm — ca "${name}": mong đúng 1 mục phân tích được, nhận ${items.length}`)
      bad += 1
      continue
    }
    const it = items[0]
    const isOrphan = it.status === 'open' && !it.hasOwner
    if (isOrphan !== expectOrphan) {
      fail(
        `tự kiểm — ca "${name}": mong ${expectOrphan ? 'MỒ CÔI' : 'KHÔNG mồ côi'}, ` +
          `nhận status=${it.status} hasOwner=${it.hasOwner}`,
      )
      bad += 1
    }
  }
  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔵 NĂM CA CHO `--file` — lượt rà 2026-08-19, tầng Verification Gap
  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 Vì sao chúng ở ĐÂY chứ không ở một tệp test riêng: `pre-push` và `ci.yml` chỉ gọi
  // `npm run check:debt-owner` **không cờ**, nên nhánh `--file` không nằm trên đường nghiệm
  // thu nào của kho. Một ca trong Kiểm B là đường DUY NHẤT cho nó chạy mỗi lượt push.
  // ── Ca cho lớp lỗi VỪA XẢY RA trong chính lượt này: thẻ chủ dán lên một dòng `---` ──
  const theSaiCho = [
    '- 🔴 **Một mục không có chủ.** Nội dung nào đó.',
    '',
    '--- **(Chủ: một story nào đó.)**',
  ].join('\n')
  {
    const items = parseItems(theSaiCho)
    const orphan = items.length === 1 && items[0].status === 'open' && !items[0].hasOwner
    if (!orphan) {
      fail(
        'tự kiểm — thẻ `**(Chủ: …)**` dán trên một dòng `---` KHÔNG được tính là chủ của mục ' +
          `phía trên (nhận ${items.length} mục, hasOwner=${items[0]?.hasOwner}). Đây đúng lớp lỗi ` +
          'đã xảy ra HAI lần ở `deferred-work.md:494` và `:1057` trong lượt phân loại 2026-08-19.',
      )
      bad += 1
    }
  }

  const R = '/repo'
  const C = '/cwd'
  const MAC_DINH = '/repo/_bmad-output/implementation-artifacts/deferred-work.md'
  const caDuong = [
    ['--file tương đối ⇒ ghép với cwd', ['--file', 'tmp/cu.md', '--report'], '/cwd/tmp/cu.md'],
    ['--file tuyệt đối ⇒ dùng NGUYÊN VĂN', ['--file', '/x/cu.md'], '/x/cu.md'],
    ['không có --file ⇒ sổ THẬT', ['--report'], MAC_DINH],
    ['--file cuối dòng, thiếu giá trị ⇒ sổ THẬT', ['--file'], MAC_DINH],
    ['--file rồi một cờ khác ⇒ sổ THẬT, không nuốt cờ', ['--file', '--report'], MAC_DINH],
  ]
  for (const [name, argv, mong] of caDuong) {
    const got = resolveDebtPath(argv, R, C)
    if (got !== mong) {
      fail(`tự kiểm --file — ca "${name}": mong ${mong}, nhận ${got}`)
      bad += 1
    }
  }

  if (bad === 0) {
    pass(
      `${SELFTEST_CASES.length} ca tự kiểm mục + 1 ca thẻ-trên-\`---\` + ` +
        `${caDuong.length} ca đường dẫn \`--file\` (đối chứng dương + âm) đều đúng`,
    )
  }
  return bad
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bề mặt — phân loại các mục mồ côi (Quyết định #5, "bộ phân loại phải dựng lại và ghi vào
// kho cùng bộ đếm" — Ice ký 2026-08-19). Đây LÀ lượt dựng lại đó.
// ═════════════════════════════════════════════════════════════════════════════════
const SURFACES = [
  ['check-*.mjs', /\bscripts\/check-[\w.-]+\.mjs\b|`check:[\w-]+`/],
  ['test Rust/vitest', /\.rs::|tests\/[\w./-]+\.rs\b|\.test\.ts\b|vitest|cargo test/i],
  ['mã src/', /\bsrc\/[\w./-]+\.(vue|ts|tsx)\b/],
  ['tài liệu/spec', /\.memlog\.md|ARCHITECTURE-SPINE\.md|\bprd\.md\b|\bepics\.md\b|README\.md/],
  ['CI/workflow', /ci\.yml|workflow_dispatch|github\/workflows/],
  ['font/từ điển', /\bfont\b|Font|dict-manifest|dict-core|dict-build|OFL|từ điển|FontFace/i],
  ['Windows/nền tảng', /Windows|NSIS|\.msi\b|WebView2|win32|ARM64/i],
  ['nghiệm thu tay/bàn đo', /nghiệm thu tay|bàn đo|chạy tay|đo bằng mắt|Ice chạy tay/i],
]

function classifySurface(text) {
  const hit = []
  for (const [name, re] of SURFACES) if (re.test(text)) hit.push(name)
  return hit.length ? hit : ['(không bám bề mặt nào)']
}

// ═════════════════════════════════════════════════════════════════════════════════
let raw
try {
  raw = readFileSync(DEBT_PATH, 'utf8')
} catch (err) {
  abort(DEBT_PATH, err)
}

const items = parseItems(raw)
const summary = summarize(items)

/**
 * 🔴 **SÀN QUẦN THỂ — *"cây rỗng không phải cây sạch"*. Thêm ở lượt rà 2026-08-19 (tầng Edge Case).**
 *
 * Bản đầu **không có sàn**, và hậu quả đo được: chĩa cổng vào một sổ không còn dòng `^- ` nào thì
 * nó in `0/0 mục mở thiếu Chủ:` và **ĐẠT**. Một Kiểm A không quét gì cả là một Kiểm A luôn xanh —
 * đúng lớp lỗi mà `project-context.md` §Luật của một CỔNG gọi tên: *"Cổng đếm số tệp và `abort()`
 * khi dưới sàn"*. Một lượt sửa `ITEM_START_RE` hỏng, một lượt đổi hình dạng bullet của sổ, hay một
 * tệp bị ghi rỗng đều đi qua cửa đó.
 *
 * Số THẬT 2026-08-19: **467** mục. Sàn 397/467 = **85,0 %**, trong dải 80-85 % mà `project-context.md`
 * đặt — cùng khuôn `check-panel-refs.mjs` *(39 tệp, sàn 33 = 84,6 %)*.
 * ⚠️ Sàn là **cận dưới**: nó không đỏ oan khi sổ dài thêm, nhưng một sàn cũ là một sàn vô nghĩa —
 * sổ này chỉ dài ra *(AC4 cấm xoá mục)*, nên xét lại số này khi nó đã cách xa thực tế.
 * 🔴 Sàn chỉ áp cho **Kiểm A trên sổ THẬT**. `--report --file <bản cũ>` được phép nhỏ hơn: một bản
 * lịch sử **đúng là** có ít mục hơn, và chấm nó là lỗi hạ tầng thì cổng tự chặn vế TRƯỚC của AC5.
 */
const ITEM_FLOOR = 397
if (DEBT_PATH === REAL_DEBT_PATH && summary.total < ITEM_FLOOR) {
  abort(
    `so no THAT — chi ${summary.total} muc, duoi san ${ITEM_FLOOR}`,
    new Error(
      'Mot Kiem A khong quet gi ca la mot Kiem A luon xanh ("cay rong khong phai cay sach").\n' +
        'Hoac `ITEM_START_RE` da hong, hoac hinh dang bullet cua so da doi, hoac tep bi ghi rong.',
    ),
  )
}

const args = process.argv.slice(2)

if (args.includes('--report')) {
  console.log(`Bốn con số AC5 — ${DEBT_PATH.replace(REPO_ROOT + '/', '')}`)
  console.log(`  tổng mục            : ${summary.total}`)
  console.log(`  mở                  : ${summary.open}`)
  console.log(`  🟡 nửa               : ${summary.half}`)
  console.log(`  ✅ đóng              : ${summary.closed}`)
  console.log(`  KHÔNG LÀM (thứ tư)  : ${summary.decided}`)
  console.log(`  mở KHÔNG có Chủ:    : ${summary.orphans.length}`)
  process.exit(0)
}

if (args.includes('--surface')) {
  const only = args[args.indexOf('--surface') + 1]
  const buckets = new Map()
  for (const it of summary.orphans) {
    for (const s of classifySurface(it.text)) {
      if (!buckets.has(s)) buckets.set(s, [])
      buckets.get(s).push(it.line)
    }
  }
  if (only && !only.startsWith('--')) {
    const lines = buckets.get(only) || []
    for (const l of lines) console.log(`deferred-work.md:${l}`)
    process.exit(0)
  }
  console.log(`Phân loại ${summary.orphans.length} mục mồ côi theo bề mặt (đa nhãn — một mục có thể thuộc nhiều bề mặt):`)
  for (const [name, lines] of [...buckets.entries()].sort((a, b) => b[1].length - a[1].length)) {
    console.log(`  ${String(lines.length).padStart(3)}  ${name}`)
  }
  process.exit(0)
}

if (args.includes('--list')) {
  for (const it of summary.orphans) console.log(`deferred-work.md:${it.line}`)
  process.exit(0)
}

// Cờ chẩn đoán nội bộ, không thuộc AC5 — dùng lúc kiểm bộ đếm trước khi tin nó (Task 1.2).
if (args.includes('--list-status')) {
  // 🔵 2026-08-19: cùng guard với nhánh `--surface` — `--list-status --file x` không được đọc
  // `--file` thành một trạng thái muốn xem rồi in một danh sách RỖNG như thể không có mục nào.
  const wantRaw = args[args.indexOf('--list-status') + 1]
  const want = wantRaw && !wantRaw.startsWith('--') ? wantRaw : undefined
  for (const it of items) if (!want || it.status === want) console.log(`deferred-work.md:${it.line}\t${it.status}`)
  process.exit(0)
}

// ── Cổng thật ────────────────────────────────────────────────────────────────────
console.log(`Kiểm A — mọi mục MỞ trong deferred-work.md phải mang một Chủ: THẬT\n`)
if (summary.orphans.length === 0) {
  pass(`0/${summary.open} mục mở thiếu Chủ: — ${summary.total} mục tổng, ${summary.half} nửa, ${summary.closed} đóng`)
} else {
  fail(`${summary.orphans.length}/${summary.open} mục mở KHÔNG có Chủ:`)
  for (const it of summary.orphans.slice(0, 20)) {
    detail(`deferred-work.md:${it.line}`)
  }
  if (summary.orphans.length > 20) detail(`… và ${summary.orphans.length - 20} mục khác. Chạy --list để xem hết.`)
}

console.log('')
const selftestBad = runSelftest()
if (selftestBad > 0) failures += selftestBad

console.log('')
if (failures !== 0) {
  console.log(`\x1b[31m${failures} phép kiểm thất bại.\x1b[0m`)
  console.log('')
  console.log('project-context.md:447-448 — "Mọi thứ không nghiệm thu được đi vào đây, KÈM MỘT')
  console.log('CHỦ. Không có mục nào mồ côi." Thêm Chủ: <story/Epic/tên người> vào mục đỏ ở trên,')
  console.log('nối tiếp — đừng xoá, đừng viết lại nội dung.')
  process.exit(1)
}
console.log('\x1b[32mTất cả phép kiểm sổ nợ đạt.\x1b[0m')
process.exit(0)
