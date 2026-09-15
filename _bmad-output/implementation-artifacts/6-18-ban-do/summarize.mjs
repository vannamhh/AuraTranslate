import fs from 'node:fs'
import path from 'node:path'

const dir = path.dirname(new URL(import.meta.url).pathname)

function rows(file) {
  const raw = fs.readFileSync(path.join(dir, file), 'utf8').trim()
  if (!raw) return []
  const lines = raw.split(/\r?\n/)
  const header = lines.shift().split('\t')
  return lines.filter(Boolean).map((line) => Object.fromEntries(line.split('\t').map((v, i) => [header[i], v])))
}

function num(values) {
  return values.map(Number).filter(Number.isFinite)
}

function max(values) {
  const xs = num(values)
  return xs.length ? Math.max(...xs) : null
}

function median(values) {
  const xs = num(values).sort((a, b) => a - b)
  if (!xs.length) return null
  const middle = Math.floor(xs.length / 2)
  return xs.length % 2 ? xs[middle] : (xs[middle - 1] + xs[middle]) / 2
}

function ms(value) {
  return value === null ? 'unknown' : `${value.toFixed(3)} ms`
}

function memory(value) {
  if (value === null) return 'unknown'
  return `${value} byte · ${(value / 1_000_000).toFixed(3)} MB · ${(value / 1_048_576).toFixed(3)} MiB`
}

const WORKS = 50
const CHAPTERS_PER_WORK = 100
const SEGMENTS_PER_CHAPTER = 10
const TOTAL_CHAPTERS = WORKS * CHAPTERS_PER_WORK
const TOTAL_SEGMENTS = TOTAL_CHAPTERS * SEGMENTS_PER_CHAPTER
const NFR4_NFR5_SESSIONS = 10

const nfr3 = rows('nfr3-raw.tsv')
const startup = rows('startup-raw.tsv')
const memoryRows = rows('memory-raw.tsv')

// 🔴 Report không được tự suy "đủ" từ vài dòng còn sót sau một lượt chết giữa chừng — cùng
// kỷ luật `5-14-ban-do/summarize.mjs`. Ma trận CỐ ĐỊNH của spec 6.18: 3 session x 5 ca NFR3;
// 10 session x (1 cold + 2 warm) NFR4; 10 session x 2 fixture x 3 pha x 10 mẫu NFR5. Thiếu
// một ô là runner phải đỏ, không được in một verdict xanh từ phần còn lại. Một lượt chẩn
// đoán thu hẹp (`AURA_NFR_BENCH_SESSIONS`) sẽ làm đúng hàm này ném lỗi — CHỦ Ý, vì report của
// nó không phải bản đầy đủ spec đòi.
function requireMatrix(ok, message) {
  if (!ok) throw new Error(`Story 6.18: raw chưa đủ ma trận — ${message}`)
}
const nfr3Cases = nfr3.filter((r) => r.record === 'case')
requireMatrix(nfr3Cases.length === 15, `NFR3 cần 15 case rows, có ${nfr3Cases.length}`)
for (const session of ['1', '2', '3']) {
  requireMatrix(nfr3Cases.filter((r) => r.session === session).length === 5, `NFR3 session ${session} không đủ 5 ca`)
}
const startupExpected = NFR4_NFR5_SESSIONS * 3 // 1 cold + 2 warm mỗi session (full: cold+warm; frontier: warm)
requireMatrix(
  startup.length === startupExpected && startup.filter((r) => r.status === 'ok').length === startupExpected,
  `NFR4 cần đúng ${startupExpected} lần launch ok (${NFR4_NFR5_SESSIONS} session x 3), có ${startup.length}`,
)
requireMatrix(startup.filter((r) => r.temperature === 'cold').length === NFR4_NFR5_SESSIONS, `NFR4 cần ${NFR4_NFR5_SESSIONS} cold`)
requireMatrix(startup.filter((r) => r.temperature === 'warm').length === NFR4_NFR5_SESSIONS * 2, `NFR4 cần ${NFR4_NFR5_SESSIONS * 2} warm`)
const memoryExpected = NFR4_NFR5_SESSIONS * 2 * 3 * 10 // session x fixture x phase x sample
requireMatrix(
  memoryRows.length === memoryExpected && memoryRows.every((r) => r.status === 'ok'),
  `NFR5 cần ${memoryExpected} mẫu ok, có ${memoryRows.length}`,
)
for (const fixture of ['full', 'frontier']) {
  for (const phase of ['library', 'reading', 'back_library_keepalive']) {
    const expectedPerCell = NFR4_NFR5_SESSIONS * 10
    requireMatrix(
      memoryRows.filter((r) => r.fixture === fixture && r.phase === phase).length === expectedPerCell,
      `NFR5 ${fixture}/${phase} không đủ ${expectedPerCell} mẫu`,
    )
  }
}

const nfr3WorstP95 = max(nfr3.filter((r) => r.record === 'case').map((r) => r.p95_ms))
const cold = startup.filter((r) => r.temperature === 'cold' && r.status === 'ok')
const warm = startup.filter((r) => r.temperature === 'warm' && r.status === 'ok')
const memoryOk = memoryRows.filter((r) => r.status === 'ok')
const memoryErrors = memoryRows.filter((r) => r.status !== 'ok')
const maxPhys = max(memoryOk.map((r) => r.phys_footprint_bytes))
const maxRss = max(memoryOk.map((r) => r.rss_bytes))

const nfr3Verdict = nfr3WorstP95 === null ? 'chưa phân xử' : nfr3WorstP95 < 500 ? 'dưới ngưỡng' : 'vượt ngưỡng'
const startupMax = max(startup.filter((r) => r.status === 'ok').map((r) => r.elapsed_ms))
const nfr4Verdict = startupMax === null || startup.some((r) => r.status !== 'ok')
  ? 'chưa phân xử'
  : startupMax < 3000 ? 'dưới ngưỡng' : 'vượt ngưỡng'
let nfr5Verdict = 'chưa phân xử'
if (maxPhys !== null && memoryErrors.length === 0) {
  if (maxPhys >= 300_000_000 && maxPhys < 314_572_800) nfr5Verdict = 'chưa phân xử — nằm giữa 300 MB và 300 MiB, cần Ice chốt đơn vị (§Always spec 6.18)'
  else nfr5Verdict = maxPhys < 300_000_000 ? 'dưới ngưỡng (hiểu 300 MB là 300.000.000 byte)' : 'vượt ngưỡng (theo cả MB và MiB)'
}

const phaseNames = [...new Set(memoryOk.map((r) => `${r.fixture}/${r.phase}`))]
const phaseLines = phaseNames.map((name) => {
  const [fixture, phase] = name.split('/')
  const selected = memoryOk.filter((r) => r.fixture === fixture && r.phase === phase)
  return `| ${fixture} | ${phase} | ${selected.length} | ${memory(max(selected.map((r) => r.phys_footprint_bytes)))} | ${memory(max(selected.map((r) => r.rss_bytes)))} |`
}).join('\n')

const report = `# Story 6.18 — kết quả đo NFR3/NFR4/NFR5 trên thư viện 5.000 Chương thật

Ngày đo, commit, máy, OS, toolchain, profile và tải máy nằm trong \`environment.txt\`. Thư viện
(${WORKS} Tác phẩm × ${CHAPTERS_PER_WORK} Chương × ${SEGMENTS_PER_CHAPTER} segment =
${TOTAL_CHAPTERS} Chương / ${TOTAL_SEGMENTS} segment) được dựng qua \`confirm_bilingual_import\` +
\`lifecycle::set_chapter_status\` + \`Indexer::rebuild\` thật — KHÔNG một fixture SQL thô nào — rồi
xoá theo HOME nháp; chỉ mẫu TSV và báo cáo này được giữ lại. Đây **THAY THẾ**, không cộng thêm,
bản sơ bộ của Story 5.14 (một fixture tổng hợp/0 lớp từ điển). Con số này **THAY THẾ** bản sơ bộ
Story 5.14 — Epic 5 retro F5 gọi con số đó "không phải một con số về sản phẩm"; đây là con số về
sản phẩm.

## Phán quyết

| NFR | Phép đo quyết định | Ngưỡng | Phán quyết |
| --- | ---: | ---: | --- |
| NFR3 | p95 xấu nhất của từng ca = ${ms(nfr3WorstP95)} | p95 < 500 ms | ${nfr3Verdict} |
| NFR4 | cold median/max = ${ms(median(cold.map((r) => r.elapsed_ms)))} / ${ms(max(cold.map((r) => r.elapsed_ms)))}; warm median/max = ${ms(median(warm.map((r) => r.elapsed_ms)))} / ${ms(max(warm.map((r) => r.elapsed_ms)))} | < 3.000 ms | ${nfr4Verdict} |
| NFR5 | phys_footprint lớn nhất = ${memory(maxPhys)}; RSS đối chiếu lớn nhất = ${memory(maxRss)} | < 300 MB | ${nfr5Verdict} |

NFR4 kết thúc khi probe thấy \`[data-library-grid]\` mang đủ ${WORKS} \`[data-library-work-cell]\`
thật (kể cả Tác phẩm đích Reading sẽ mở) VÀ ít nhất một lớp từ điển đã nạp; mốc ngoài tiến trình
chạy từ trước spawn tới sau khi marker đã được ghi vào \`global.db\`, nên là cận trên nhỏ của mốc
DOM. NFR5 từ chối mọi mẫu chỉ có PID app: mỗi hàng \`ok\` có PID app cộng ít nhất một WebKit mới
sinh; hàng thiếu PID/footprint/RSS được giữ nguyên là \`error\` và làm phán quyết thành
\`chưa phân xử\`. Reading pha \`full\` mở ĐÚNG MỘT Tác phẩm (không toàn thư viện — Design Notes
spec 6.18: "Reading runs over the open Work only"), nên số NFR5 pha Reading KHÔNG so sánh được
với 894.570.496 byte của Story 5.14 (nơi Reading đọc cả 50.000 segment của một fixture tổng hợp).

## Bộ nhớ theo pha

| Fixture | Pha idle | Số mẫu hợp lệ | phys_footprint lớn nhất | RSS lớn nhất |
| --- | --- | ---: | ---: | ---: |
${phaseLines}

Mỗi session full đo Library (50 Tác phẩm) → Reading ${CHAPTERS_PER_WORK * SEGMENTS_PER_CHAPTER}
segment (một Tác phẩm, mọi Chương \`done\`) → quay lại Library. Mỗi session frontier đo Library →
Reading frontier-only trên Chương đầu Tác phẩm đích (100 Chương vừa \`not_started\` qua
\`lifecycle::set_chapter_status\`, không một câu SQL ghi nào) → quay lại Library. Dữ liệu thô:
\`nfr3-raw.tsv\`, \`startup-raw.tsv\`, \`memory-raw.tsv\`, \`transition-raw.tsv\`, \`fixture.txt\`.
`

fs.writeFileSync(path.join(dir, 'REPORT.md'), report)
process.stdout.write(report)
