import { execFileSync } from 'node:child_process'
import { dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import {
  SUGGESTIONS,
  crossKindPairs,
  ignoreWindow,
  judgeRate,
  listScans,
  openReadOnly,
} from './lib/falseAlarm.mjs'

const repo = dirname(fileURLToPath(import.meta.url))

function fail(message) {
  process.stderr.write(`${message}\n`)
  process.exit(2)
}

function parseArgs(argv) {
  const out = { path: null, chapter: null, scan: null }
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a === '--chapter' || a === '--scan') {
      const v = argv[++i]
      if (v === undefined || !/^\d+$/.test(v)) fail(`${a} cần một số nguyên không âm`)
      out[a.slice(2)] = Number(v)
    } else if (a.startsWith('--')) {
      fail(`cờ không biết: ${a}`)
    } else if (out.path === null) {
      out.path = a
    } else {
      fail(`thừa đối số: ${a}`)
    }
  }
  if (out.path === null) fail('dùng: <project.db> --chapter <id> [--scan <id>]')
  if (out.chapter === null) fail('thiếu --chapter <id>')
  return out
}

function git(args) {
  try {
    return execFileSync('git', args, { cwd: repo, encoding: 'utf8' }).trim()
  } catch {
    return null
  }
}

function pct(x) {
  return `${(x * 100).toFixed(1).replace('.', ',')}%`
}

const args = parseArgs(process.argv.slice(2))
let db
try {
  db = openReadOnly(args.path)
} catch (e) {
  fail(`không mở được ${args.path}: ${e.message}`)
}

let scans, chapter, work
try {
  scans = listScans(db)
  chapter = db.prepare('SELECT id, ord, title FROM chapter WHERE id = ?').get(args.chapter)
  work = db.prepare('SELECT name FROM work WHERE id = 1').get()
} catch (e) {
  fail(`không đọc được lược đồ proofreader: ${e.message}`)
}
if (!chapter) fail(`Chương ${args.chapter} không có trong DB`)
if (scans.length === 0) fail('proof_scan rỗng: chưa có lượt quét nào, không có tỷ lệ')

let scan
if (args.scan !== null) {
  scan = scans.find((s) => s.id === args.scan)
  if (!scan) fail(`--scan ${args.scan} không có trong proof_scan`)
} else if (scans.length > 1) {
  const list = scans.map((s) => `  id=${s.id} scope=${s.scope} scanned_at=${s.scanned_at} reported=${s.reported}`)
  fail(`proof_scan có ${scans.length} hàng, truyền --scan <id>:\n${list.join('\n')}`)
} else {
  scan = scans[0]
}

const win = ignoreWindow(db, scans, scan)
const judged = judgeRate({
  reported: Number(scan.reported),
  filtered: Number(scan.filtered),
  ignored: win.ignored,
  beforeScan: win.beforeScan,
  scope: scan.scope,
})
if (judged.error) fail(judged.error)

const head = git(['rev-parse', 'HEAD'])
const dirty = git(['status', '--porcelain'])
const lines = [
  `git HEAD: ${head ?? 'không đọc được'} (${dirty === null ? 'không rõ' : dirty === '' ? 'sạch' : 'bẩn'})`,
  `Tác phẩm: ${work?.name ?? '(không có)'}`,
  `Chương (lời khai --chapter): id=${chapter.id} ord=${chapter.ord} title=${chapter.title ?? '(không tên)'}`,
  `scan.id: ${scan.id} (scope=${scan.scope}, scanned_at=${scan.scanned_at})`,
  `n (reported): ${scan.reported}   filtered: ${scan.filtered}`,
  `tử số: ${win.ignored} (${Object.entries(win.byKind).map(([k, v]) => `${k}=${v}`).join(', ') || 'không có'})`,
  `tỷ lệ: ${pct(judged.rate)}   Wilson 95%: ${pct(judged.interval.low)} đến ${pct(judged.interval.high)}`,
]
if (judged.reference !== null) lines.push(`tham khảo, trên reported + filtered: ${pct(judged.reference)}`)
lines.push(`cặp (cụm, ngữ cảnh) có hai loại: ${crossKindPairs(db)}`)
for (const w of judged.warnings) lines.push(`CẢNH BÁO: ${w}`)
lines.push(`kết luận: ${judged.verdict}`)
if (judged.verdict === 'vượt') {
  lines.push('đề xuất (FR81):', ...SUGGESTIONS.map((s) => `  - ${s}`))
}
process.stdout.write(`${lines.join('\n')}\n`)
