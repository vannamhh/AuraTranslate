import fs from 'node:fs'

const [, , rawPath, populationLine] = process.argv
const result = JSON.parse(fs.readFileSync(rawPath, 'utf8'))
const failures = []
const ms = (us) => (us / 1000).toFixed(3)
const pct = (sorted, p) => sorted[Math.max(0, Math.ceil(p * sorted.length) - 1)]
const stats = (values) => {
  const sorted = [...values].sort((a, b) => a - b)
  return { p50: pct(sorted, 0.5), p95: pct(sorted, 0.95), max: sorted[sorted.length - 1], n: sorted.length }
}

console.log(populationLine)
console.log(
  `probe_population segments=${result.population.segments} distinct_sources=${result.population.distinct_sources} ` +
    `empty_targets_before_prefill=${result.population.empty_targets_before_prefill} rounds=${result.rounds} batch_size=${result.batch_size}`,
)
if (result.population.segments !== 300 || result.population.distinct_sources !== 300 || result.population.empty_targets_before_prefill !== 300) {
  failures.push('probe-side Chapter population differs from the declaration')
}
if (!result.prompt_has_tm) failures.push('prompt assembled without a TM search (ledger.tm.kind != searched)')
console.log(`prompt_tm kind=searched similar_segments_first_segment=${result.prompt_tm_similar_count}`)
console.log(`details ${JSON.stringify(result.details)}`)
if (result.details.prefill_filled_after_round_1 !== 150) {
  failures.push(`prefill filled ${result.details.prefill_filled_after_round_1} segments, declared 150`)
}

const control = result.control
console.log(`control windows=${control.w} hold_ms=${ms(control.h)} max_poll_gap_ms=${ms(control.g)}`)
if (control.w !== 0) failures.push(`control round recorded ${control.w} held windows`)

const line = (label, rounds, tag) => {
  const hold = stats(rounds.map((r) => r.h))
  const sum = stats(rounds.map((r) => r.s))
  const wall = stats(rounds.map((r) => r.t))
  const zero = rounds.filter((r) => r.w === 0).length
  const windows = rounds.map((r) => r.w)
  const gap = Math.max(...rounds.map((r) => r.g))
  console.log(
    `RESULT ${label}${tag} n=${hold.n} hold_ms(p50/p95/max)=${ms(hold.p50)}/${ms(hold.p95)}/${ms(hold.max)} ` +
      `sum_ms(p50/p95/max)=${ms(sum.p50)}/${ms(sum.p95)}/${ms(sum.max)} wall_ms(p50/p95/max)=${wall.p50}/${wall.p95}/${wall.max} ` +
      `windows(min/max)=${Math.min(...windows)}/${Math.max(...windows)} rounds_without_window=${zero} max_poll_gap_ms=${ms(gap)}`,
  )
  return { hold, zero }
}

const verdicts = []
for (const [label, rounds] of Object.entries(result.commands)) {
  if (rounds.length !== result.rounds) failures.push(`${label}: ${rounds.length} rounds, expected ${result.rounds}`)
  const outcomes = {}
  for (const r of rounds) outcomes[r.o] = (outcomes[r.o] ?? 0) + 1
  console.log(`OUTCOMES ${label} ${JSON.stringify(outcomes)}`)
  const coarse = rounds.filter((r) => r.h / 1000 > r.t + 5).length
  if (coarse > 0) failures.push(`${label}: ${coarse} rounds report a hold longer than the call itself (watcher resolution too coarse)`)
  if (rounds.some((r) => r.f & 4)) failures.push(`${label}: OpenWorkState was not managed at some poll`)
  if (rounds.some((r) => r.f & 2)) failures.push(`${label}: the lock was still held when the watcher stopped`)
  const expected = label.startsWith('ai_translate_') ? 'rejected:ai_translate.provider_unreachable' : 'ok'
  const early = rounds.filter((r) => r.o !== expected).length
  if (early > 0) failures.push(`${label}: ${early} rounds ended with an outcome other than ${expected}`)
  if (rounds.some((r) => r.f & 1)) failures.push(`${label}: held windows were truncated`)
  if (rounds.every((r) => r.w === 0)) failures.push(`${label}: zero held windows in all rounds`)
  if (label === 'read_open_chapter_segments') {
    line(label, [rounds[0]], ' round_1_lookup+write')
    const rest = line(label, rounds.slice(1), ' rounds_2_to_20_lookup_only')
    verdicts.push([label, rest.hold.p95, Math.max(rounds[0].h, rest.hold.p95)])
  } else {
    const all = line(label, rounds, '')
    verdicts.push([label, all.hold.p95, all.hold.p95])
  }
}
for (const [label, , worst] of verdicts) {
  console.log(`VERDICT ${label} verdict_ms=${ms(worst)} ${worst < 100000 ? 'under_100ms_KHONG_LAM' : 'at_or_over_100ms_keep_open'}`)
}
if (failures.length) {
  for (const f of failures) console.error(`FAIL: ${f}`)
  process.exit(1)
}
console.log('ALL_CHECKS_PASSED')
