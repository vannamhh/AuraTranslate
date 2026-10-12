import { spawnSync } from 'node:child_process'
import { mkdtempSync, rmSync } from 'node:fs'
import { createRequire } from 'node:module'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterAll, beforeAll, describe, expect, it } from 'vitest'
import { openReadOnly, wilson } from '../../scripts/lib/falseAlarm.mjs'

const { DatabaseSync } = createRequire(import.meta.url)('node:sqlite')
const SCRIPT = join(__dirname, '../../scripts/measure-proofread-false-alarm.mjs')

const SCAN_AT = '2026-10-12T10:00:00.000Z'
const AFTER = '2026-10-12T10:05:00.000Z'
const BEFORE = '2026-10-12T09:59:00.000Z'

interface Fixture {
  scans: Array<{ scope?: string; at?: string; reported: number; filtered?: number }>
  ignores: Array<{ at: string; kind?: string; phrase?: string; before?: string; after?: string }>
}

let dir: string
let counter = 0

beforeAll(() => {
  dir = mkdtempSync(join(tmpdir(), 'false-alarm-'))
})
afterAll(() => rmSync(dir, { recursive: true, force: true }))

function build(f: Fixture): string {
  const path = join(dir, `p${counter++}.db`)
  const db = new DatabaseSync(path)
  db.exec(`
    CREATE TABLE work (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
    CREATE TABLE chapter (id INTEGER PRIMARY KEY, ord INTEGER NOT NULL, title TEXT);
    CREATE TABLE proof_ignore (id INTEGER PRIMARY KEY, kind TEXT NOT NULL, phrase TEXT NOT NULL,
      context_before TEXT NOT NULL, context_after TEXT NOT NULL, created_at TEXT NOT NULL);
    CREATE TABLE proof_scan (id INTEGER PRIMARY KEY, scope TEXT NOT NULL, scanned_at TEXT NOT NULL,
      reported INTEGER NOT NULL, filtered INTEGER NOT NULL);
    INSERT INTO work VALUES (1, 'Truyện thử');
    INSERT INTO chapter VALUES (7, 3, 'Chương ba');
  `)
  for (const s of f.scans) {
    db.prepare('INSERT INTO proof_scan (scope, scanned_at, reported, filtered) VALUES (?,?,?,?)').run(
      s.scope ?? 'chapter',
      s.at ?? SCAN_AT,
      s.reported,
      s.filtered ?? 0,
    )
  }
  f.ignores.forEach((r, i) => {
    db.prepare(
      'INSERT INTO proof_ignore (kind, phrase, context_before, context_after, created_at) VALUES (?,?,?,?,?)',
    ).run(r.kind ?? 'spelling', r.phrase ?? `cụm${i}`, r.before ?? '', r.after ?? '', r.at)
  })
  db.close()
  return path
}

const rows = (n: number, at = AFTER) => Array.from({ length: n }, () => ({ at }))

function run(args: string[]) {
  const r = spawnSync(process.execPath, ['--no-warnings', SCRIPT, ...args], { encoding: 'utf8' })
  return { code: r.status, out: r.stdout, err: r.stderr }
}

describe('đo tỷ lệ báo động giả', () => {
  it('đủ mẫu, đỗ', () => {
    const p = build({ scans: [{ reported: 40 }], ignores: rows(6) })
    const r = run([p, '--chapter', '7'])
    expect(r.code).toBe(0)
    expect(r.out).toContain('15,0%')
    expect(r.out).toContain('Wilson')
    expect(r.out).toContain('kết luận: đỗ')
    expect(r.out).toContain('Truyện thử')
    expect(r.out).toContain('ord=3')
    expect(r.out).toContain('Chương ba')
  })

  it('đủ mẫu, vượt kèm ba đề xuất', () => {
    const p = build({ scans: [{ reported: 40 }], ignores: rows(12) })
    const r = run(['--chapter', '7', p])
    expect(r.out).toContain('kết luận: vượt')
    expect(r.out).toContain('chỉnh prompt')
    expect(r.out).toContain('hạ độ nhạy')
    expect(r.out).toContain('tách riêng hai loại')
  })

  it('thiếu mẫu: chưa đủ mẫu dù tỷ lệ thấp', () => {
    const p = build({ scans: [{ reported: 12 }], ignores: rows(2) })
    const r = run([p, '--chapter', '7'])
    expect(r.out).toContain('16,7%')
    expect(r.out).toContain('kết luận: chưa đủ mẫu')
  })

  it('filtered > 0: kết luận theo reported, in thêm cách tính khác, có cảnh báo', () => {
    const p = build({ scans: [{ reported: 40, filtered: 10 }], ignores: rows(6) })
    const r = run([p, '--chapter', '7'])
    expect(r.out).toContain('kết luận: đỗ')
    expect(r.out).toContain('tham khảo, trên reported + filtered: 12,0%')
    expect(r.out).toContain('CẢNH BÁO')
  })

  it('nhiều lượt quét: liệt kê và đòi --scan', () => {
    const p = build({
      scans: [{ reported: 40 }, { reported: 50, at: '2026-10-12T11:00:00.000Z' }],
      ignores: rows(6),
    })
    const r = run([p, '--chapter', '7'])
    expect(r.code).toBe(2)
    expect(r.err).toContain('--scan')
    expect(r.err).toContain('id=2')
    expect(r.out).not.toContain('tỷ lệ')
  })

  it('--scan đứng trước đường dẫn và chọn đúng lượt; cửa sổ dừng ở lượt kế', () => {
    const p = build({
      scans: [{ reported: 40 }, { reported: 50, at: '2026-10-12T11:00:00.000Z' }],
      ignores: [...rows(6), ...rows(4, '2026-10-12T11:10:00.000Z')],
    })
    const r = run(['--scan', '1', p, '--chapter', '7'])
    expect(r.code).toBe(0)
    expect(r.out).toContain('tử số: 6')
  })

  it('--scan không nguyên hoặc không có id', () => {
    const p = build({ scans: [{ reported: 40 }], ignores: [] })
    expect(run([p, '--chapter', '7', '--scan', '1.5']).code).toBe(2)
    const r = run([p, '--chapter', '7', '--scan', '9'])
    expect(r.code).toBe(2)
    expect(r.err).toContain('--scan 9')
  })

  it('không có proof_scan: thoát khác 0, không phải 0%', () => {
    const p = build({ scans: [], ignores: [] })
    const r = run([p, '--chapter', '7'])
    expect(r.code).toBe(2)
    expect(r.err).toContain('proof_scan rỗng')
    expect(r.out).not.toContain('%')
  })

  it('DB không phải DB của dự án: thoát khác 0', () => {
    const path = join(dir, 'empty.db')
    new DatabaseSync(path).close()
    const r = run([path, '--chapter', '7'])
    expect(r.code).toBe(2)
    expect(r.out).toBe('')
  })

  it('bấm giữa lượt quét: in số hàng, không kết luận đỗ', () => {
    const p = build({
      scans: [{ at: '2026-10-12T09:00:00.000Z', reported: 5 }, { reported: 40 }],
      ignores: [...rows(6), ...rows(3, BEFORE)],
    })
    const r = run([p, '--chapter', '7', '--scan', '2'])
    expect(r.out).toContain('3 hàng ignore tạo trước scanned_at')
    expect(r.out).not.toContain('kết luận: đỗ')
    expect(r.out).toContain('tử số: 6')
  })

  it('bấm giữa lượt quét nhưng tỷ lệ vượt: vẫn vượt, kèm cảnh báo', () => {
    const p = build({
      scans: [{ at: '2026-10-12T09:00:00.000Z', reported: 5 }, { reported: 40 }],
      ignores: [...rows(12), ...rows(3, BEFORE)],
    })
    const r = run([p, '--chapter', '7', '--scan', '2'])
    expect(r.out).toContain('kết luận: vượt')
    expect(r.out).toContain('3 hàng ignore tạo trước scanned_at')
  })

  it('tử số > mẫu số: thoát khác 0, không NaN', () => {
    const p = build({ scans: [{ reported: 5 }], ignores: rows(8) })
    const r = run([p, '--chapter', '7'])
    expect(r.code).toBe(2)
    expect(r.out + r.err).not.toContain('NaN')
    expect(r.out).not.toContain('Wilson')
  })

  it('scope khác chapter: in scope, không kết luận đỗ', () => {
    const p = build({ scans: [{ scope: 'selection', reported: 40 }], ignores: rows(6) })
    const r = run([p, '--chapter', '7'])
    expect(r.out).toContain('scope = selection')
    expect(r.out).not.toContain('kết luận: đỗ')
  })

  it('--chapter sai hoặc thiếu', () => {
    const p = build({ scans: [{ reported: 40 }], ignores: rows(6) })
    const bad = run([p, '--chapter', '99'])
    expect(bad.code).toBe(2)
    expect(bad.out).not.toContain('%')
    const missing = run([p])
    expect(missing.code).toBe(2)
    expect(missing.err).toContain('--chapter')
  })

  it('tử số tách theo loại', () => {
    const p = build({
      scans: [{ reported: 40 }],
      ignores: [{ at: AFTER, kind: 'spelling' }, { at: AFTER, kind: 'grammar' }, { at: AFTER, kind: 'grammar' }],
    })
    expect(run([p, '--chapter', '7']).out).toContain('grammar=2, spelling=1')
  })
})

describe('openReadOnly', () => {
  it('từ chối lượt ghi', () => {
    const p = build({ scans: [{ reported: 1 }], ignores: [] })
    const db = openReadOnly(p)
    expect(() => db.exec("INSERT INTO proof_scan VALUES (9,'chapter','x',1,0)")).toThrow(/readonly/i)
    db.close()
  })
})

describe('wilson', () => {
  it('khoảng quanh 15% với n = 40', () => {
    const w = wilson(6, 40)
    expect(w.low).toBeCloseTo(0.0705, 3)
    expect(w.high).toBeCloseTo(0.2907, 3)
  })
})
