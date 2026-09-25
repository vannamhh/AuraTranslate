#!/usr/bin/env node
/**
 * Warns when the previous push's CI run failed. Not a gate: no `check:*` name, never
 * blocks a push, stays out of the three gate lists (`package.json` / `ci.yml` /
 * `.githooks/pre-push`'s `for gate in … ; do` loop) and out of `check:gates`.
 *
 * Reads `git rev-parse` for the upstream ref, `gh auth status` and `gh run list`
 * through `spawnSync` with an 8s timeout each — no shell, no `perl`, no `/dev/null`, so
 * this runs the same on Windows as on macOS/Linux. `ciPreviousVerdict({ run })` is the
 * pure, injectable core; `spawnRun` is the only impure edge.
 *
 * Run:  node scripts/ci-previous-verdict.mjs
 */
import { spawnSync } from 'node:child_process'
import { pathToFileURL } from 'node:url'

const TIMEOUT_MS = 8000
const WORKFLOW = 'ci.yml'
const BRANCH = 'master'

/**
 * @typedef {object} RunResult
 * @property {number | null} status
 * @property {string} stdout
 * @property {Error} [error]
 */

/**
 * @param {string} cmd
 * @param {string[]} args
 * @returns {RunResult}
 */
export function spawnRun(cmd, args) {
  const r = spawnSync(cmd, args, { encoding: 'utf8', timeout: TIMEOUT_MS })
  const error = r.error ?? (r.signal ? new Error(`spawnSync ${cmd} killed by signal ${r.signal}`) : undefined)
  return { status: r.status, stdout: r.stdout ?? '', error }
}

/**
 * @typedef {{ kind: 'failure', sha: string } | { kind: 'success' } | { kind: 'in_progress' } | { kind: 'silent' }} Verdict
 */

/**
 * Pure core: no `fs`, no `process`, no timers. Every early return is a documented
 * "silent skip" branch of the spec's I/O matrix.
 * @param {{ run: (cmd: string, args: string[]) => RunResult }} deps
 * @returns {Verdict}
 */
export function ciPreviousVerdict({ run }) {
  const auth = run('gh', ['auth', 'status'])
  if (auth.error || auth.status !== 0) return { kind: 'silent' }

  const upstream = run('git', ['rev-parse', '@{u}'])
  const sha = upstream.stdout.trim()
  if (upstream.error || upstream.status !== 0 || !sha) return { kind: 'silent' }

  const list = run('gh', [
    'run',
    'list',
    '--workflow',
    WORKFLOW,
    '--branch',
    BRANCH,
    '--event',
    'push',
    '--json',
    'headSha,conclusion,status',
    '-L',
    '30',
  ])
  if (list.error || list.status !== 0) return { kind: 'silent' }

  /** @type {unknown} */
  let runs
  try {
    runs = JSON.parse(list.stdout)
  } catch {
    return { kind: 'silent' }
  }
  if (!Array.isArray(runs)) return { kind: 'silent' }

  const match = runs.find((r) => r && typeof r === 'object' && r.headSha === sha)
  if (!match) return { kind: 'silent' }

  const outcome = match.status === 'completed' ? match.conclusion : match.status
  if (outcome === 'failure') return { kind: 'failure', sha }
  if (outcome === 'success') return { kind: 'success' }
  if (outcome === 'in_progress' || outcome === 'queued') return { kind: 'in_progress' }
  return { kind: 'silent' }
}

/** @param {string} s */
const yellow = (s) => `\x1b[33m${s}\x1b[0m`
/** @param {string} s */
const green = (s) => `\x1b[32m${s}\x1b[0m`
/** @param {string} s */
const grey = (s) => `\x1b[90m${s}\x1b[0m`

/** @param {Verdict} v */
export function printVerdict(v) {
  process.stdout.write(`  ${'ci (trước)'.padEnd(14)}`)
  switch (v.kind) {
    case 'failure':
      process.stdout.write('\n')
      console.log(
        yellow(`CẢNH BÁO — CI của lượt push trước (${v.sha}) ĐỎ. Xem: gh run list --branch master --workflow ci.yml`),
      )
      break
    case 'success':
      console.log(green('OK'))
      break
    case 'in_progress':
      console.log(grey('đang chạy'))
      break
    default:
      console.log(grey('—'))
  }
}

/** Never throws, never sets a non-zero exit code — this step must never fail the hook. */
function main() {
  try {
    printVerdict(ciPreviousVerdict({ run: spawnRun }))
  } catch {
    process.stdout.write(`  ${'ci (trước)'.padEnd(14)}`)
    console.log(grey('—'))
  }
}

const isMain = process.argv[1] !== undefined && import.meta.url === pathToFileURL(process.argv[1]).href
if (isMain) main()
