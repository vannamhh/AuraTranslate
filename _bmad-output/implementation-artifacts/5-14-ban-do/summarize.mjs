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

const nfr3 = rows('nfr3-raw.tsv')
const startup = rows('startup-raw.tsv')
const memoryRows = rows('memory-raw.tsv')
const reading = rows('reading-run-raw.tsv')

// 🔴 Report không được tự suy "đủ" từ vài dòng còn sót sau một lượt chết giữa chừng. Đây là
// ma trận cố định của spec: 3 phiên × 5 ca NFR3; 3 cold + 6 warm; 3 phiên × 2 fixture × 3 pha
// × 10 mẫu NFR5; và đúng hai hình dạng read_reading_run. Thiếu một ô là runner phải đỏ, không
// được in một verdict xanh từ phần còn lại.
function requireMatrix(ok, message) {
  if (!ok) throw new Error(`Story 5.14: raw chưa đủ ma trận — ${message}`)
}
const nfr3Cases = nfr3.filter((r) => r.record === 'case')
requireMatrix(nfr3Cases.length === 15, `NFR3 cần 15 case rows, có ${nfr3Cases.length}`)
for (const session of ['1', '2', '3']) {
  requireMatrix(nfr3Cases.filter((r) => r.session === session).length === 5, `NFR3 session ${session} không đủ 5 ca`)
}
requireMatrix(startup.length === 9 && startup.filter((r) => r.status === 'ok').length === 9, 'NFR4 cần đúng 9 lần launch ok')
requireMatrix(startup.filter((r) => r.temperature === 'cold').length === 3, 'NFR4 cần 3 cold')
requireMatrix(startup.filter((r) => r.temperature === 'warm').length === 6, 'NFR4 cần 6 warm')
requireMatrix(reading.length === 2 && new Set(reading.map((r) => r.case)).size === 2, 'cần đúng hai hình dạng read_reading_run')
requireMatrix(memoryRows.length === 180 && memoryRows.every((r) => r.status === 'ok'), 'NFR5 cần 180 mẫu ok')
for (const fixture of ['full', 'frontier']) {
  for (const phase of ['library', 'reading', 'back_library_keepalive']) {
    requireMatrix(memoryRows.filter((r) => r.fixture === fixture && r.phase === phase).length === 30, `NFR5 ${fixture}/${phase} không đủ 30 mẫu`)
  }
}

const nfr3WorstP95 = max(nfr3.filter((r) => r.record === 'case').map((r) => r.p95_ms))
const cold = startup.filter((r) => r.temperature === 'cold' && r.status === 'ok')
const warm = startup.filter((r) => r.temperature === 'warm' && r.status === 'ok')
const memoryOk = memoryRows.filter((r) => r.status === 'ok')
const memoryErrors = memoryRows.filter((r) => r.status !== 'ok')
const maxPhys = max(memoryOk.map((r) => r.phys_footprint_bytes))
const maxRss = max(memoryOk.map((r) => r.rss_bytes))

const nfr3Verdict = nfr3WorstP95 === null ? 'chưa phân xử' : nfr3WorstP95 < 500 ? 'dưới ngưỡng (sơ bộ)' : 'vượt ngưỡng (sơ bộ)'
const startupMax = max(startup.filter((r) => r.status === 'ok').map((r) => r.elapsed_ms))
const nfr4Verdict = startupMax === null || startup.some((r) => r.status !== 'ok')
  ? 'chưa phân xử'
  : startupMax < 3000 ? 'dưới ngưỡng (sơ bộ)' : 'vượt ngưỡng (sơ bộ)'
let nfr5Verdict = 'chưa phân xử'
if (maxPhys !== null && memoryErrors.length === 0) {
  if (maxPhys >= 300_000_000 && maxPhys < 314_572_800) nfr5Verdict = 'chưa phân xử — nằm giữa 300 MB và 300 MiB, cần Ice chốt đơn vị'
  else nfr5Verdict = maxPhys < 300_000_000 ? 'dưới ngưỡng (sơ bộ, hiểu 300 MB là 300.000.000 byte)' : 'vượt ngưỡng (sơ bộ theo cả MB và MiB)'
}

const readingLines = reading.map((r) => `| ${r.case} | ${r.samples} | ${ms(Number(r.p50_ms))} | ${ms(Number(r.p95_ms))} | ${ms(Number(r.p99_ms))} | ${ms(Number(r.worst_ms))} |`).join('\n')
const phaseNames = [...new Set(memoryOk.map((r) => `${r.fixture}/${r.phase}`))]
const phaseLines = phaseNames.map((name) => {
  const [fixture, phase] = name.split('/')
  const selected = memoryOk.filter((r) => r.fixture === fixture && r.phase === phase)
  return `| ${fixture} | ${phase} | ${selected.length} | ${memory(max(selected.map((r) => r.phys_footprint_bytes)))} | ${memory(max(selected.map((r) => r.rss_bytes)))} |`
}).join('\n')

const report = `# Story 5.14 — kết quả đo sơ bộ NFR3/NFR4/NFR5

Ngày đo, commit, máy, OS, toolchain, profile và tải máy nằm trong \`environment.txt\`. Fixture nằm trong HOME nháp và bị xoá bởi trap; chỉ mẫu TSV và báo cáo này được giữ lại. Đây là fixture tổng hợp một Work/5.000 Chương, chưa phải thư viện 5.000 Chương tạo qua FR14, nên mọi phán quyết đều **sơ bộ**; A6–A8/Q4 vẫn mở tới Story 6.18.

## Phán quyết

| NFR | Phép đo quyết định | Ngưỡng tạm | Phán quyết |
| --- | ---: | ---: | --- |
| NFR3 | p95 xấu nhất của từng ca = ${ms(nfr3WorstP95)} | p95 < 500 ms | ${nfr3Verdict} |
| NFR4 | cold median/max = ${ms(median(cold.map((r) => r.elapsed_ms)))} / ${ms(max(cold.map((r) => r.elapsed_ms)))}; warm median/max = ${ms(median(warm.map((r) => r.elapsed_ms)))} / ${ms(max(warm.map((r) => r.elapsed_ms)))} | < 3.000 ms | ${nfr4Verdict} |
| NFR5 | phys_footprint lớn nhất = ${memory(maxPhys)}; RSS đối chiếu lớn nhất = ${memory(maxRss)} | < 300 MB | ${nfr5Verdict} |

NFR4 kết thúc khi probe thấy \`[data-library-grid]\` mang đúng một \`[data-library-work-cell]\` tên “5.14 Fixture”; mốc ngoài tiến trình chạy từ trước spawn tới sau khi marker đã được ghi vào \`global.db\`, nên là cận trên nhỏ của mốc DOM. NFR5 từ chối mọi mẫu chỉ có PID app: mỗi hàng \`ok\` có PID app cộng ít nhất một WebKit mới sinh; hàng thiếu PID/footprint/RSS được giữ nguyên là \`error\` và làm phán quyết thành \`chưa phân xử\`.

## Hai chi phí \`read_reading_run\`

| Hình dạng | Mẫu | p50 | p95 | p99 | Xấu nhất |
| --- | ---: | ---: | ---: | ---: | ---: |
${readingLines}

## Bộ nhớ theo pha

| Fixture | Pha idle | Số mẫu hợp lệ | phys_footprint lớn nhất | RSS lớn nhất |
| --- | --- | ---: | ---: | ---: |
${phaseLines}

Mỗi session full đo Library → Reading 50.000 segment → quay lại Library (Reading component nằm trong KeepAlive). Mỗi session frontier đo Library → Reading frontier-only trên Chương đầu chưa \`done\` → quay lại Library. Dữ liệu thô: \`nfr3-raw.tsv\`, \`reading-run-raw.tsv\`, \`startup-raw.tsv\`, \`memory-raw.tsv\`.
`

fs.writeFileSync(path.join(dir, 'REPORT.md'), report)
process.stdout.write(report)
