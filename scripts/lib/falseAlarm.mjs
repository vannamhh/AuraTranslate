import { createRequire } from 'node:module'

export const MIN_SAMPLE = 30
export const PASS_RATE = 0.2
const Z = 1.96

export function openReadOnly(path) {
  const { DatabaseSync } = createRequire(import.meta.url)('node:sqlite')
  return new DatabaseSync(path, { readOnly: true })
}

export function wilson(k, n) {
  const p = k / n
  const z2 = Z * Z
  const denom = 1 + z2 / n
  const centre = (p + z2 / (2 * n)) / denom
  const half = (Z * Math.sqrt((p * (1 - p)) / n + z2 / (4 * n * n))) / denom
  return { low: Math.max(0, centre - half), high: Math.min(1, centre + half) }
}

export function listScans(db) {
  return db
    .prepare('SELECT id, scope, scanned_at, reported, filtered FROM proof_scan ORDER BY scanned_at, id')
    .all()
}

/**
 * `ignored` counts rows created at or after this scan's `scanned_at` and strictly before the next
 * scan's. `beforeScan` counts rows created at or after the previous scan's `scanned_at` and strictly
 * before this one's: marks made while the scan was still running.
 */
export function ignoreWindow(db, scans, scan) {
  const at = scans.findIndex((s) => s.id === scan.id)
  const prev = at > 0 ? scans[at - 1] : null
  const next = at < scans.length - 1 ? scans[at + 1] : null
  const count = (from, to) => {
    const rows = db
      .prepare(
        `SELECT kind, COUNT(*) AS n FROM proof_ignore
         WHERE (?1 IS NULL OR created_at >= ?1) AND (?2 IS NULL OR created_at < ?2)
         GROUP BY kind`,
      )
      .all(from, to)
    const byKind = Object.fromEntries(rows.map((r) => [r.kind, Number(r.n)]))
    return { byKind, total: rows.reduce((a, r) => a + Number(r.n), 0) }
  }
  const inside = count(scan.scanned_at, next ? next.scanned_at : null)
  const before = count(prev ? prev.scanned_at : null, scan.scanned_at)
  return { ignored: inside.total, byKind: inside.byKind, beforeScan: before.total }
}

export function crossKindPairs(db) {
  return Number(
    db
      .prepare(
        `SELECT COUNT(*) AS n FROM (
           SELECT 1 FROM proof_ignore GROUP BY phrase, context_before, context_after
           HAVING COUNT(DISTINCT kind) > 1)`,
      )
      .get().n,
  )
}

export const SUGGESTIONS = [
  'chỉnh prompt của proofreader để bớt báo những chỗ không phải lỗi',
  'hạ độ nhạy của proofreader (chỉ báo khi chắc hơn)',
  'tách riêng hai loại phát hiện (chính tả/ngữ pháp và nghi về nghĩa) để đo và chỉnh từng loại',
]

export function judgeRate({ reported, filtered, ignored, beforeScan, scope }) {
  if (ignored > reported) {
    return { error: `số hàng ignore trong cửa sổ (${ignored}) lớn hơn reported (${reported}); không tính tỷ lệ` }
  }
  if (reported === 0) {
    return { error: 'reported = 0: không có mẫu số, không tính tỷ lệ' }
  }
  const rate = ignored / reported
  const interval = wilson(ignored, reported)
  const warnings = []
  if (scope !== 'chapter') warnings.push(`lượt quét có scope = ${scope}, không phải chapter`)
  if (beforeScan > 0) {
    warnings.push(`${beforeScan} hàng ignore tạo trước scanned_at của lượt này (bấm giữa lượt quét), không nằm trong tử số`)
  }
  if (filtered > 0) {
    warnings.push(`filtered = ${filtered} > 0: Chương đã có ghi nhớ, vi phạm quy trình quét một lần`)
  }
  let verdict
  if (reported < MIN_SAMPLE) verdict = 'chưa đủ mẫu'
  else if (rate > PASS_RATE) verdict = 'vượt'
  else if (scope !== 'chapter' || beforeScan > 0) verdict = 'không kết luận'
  else verdict = 'đỗ'
  return { rate, interval, warnings, verdict, reference: filtered > 0 ? ignored / (reported + filtered) : null }
}
