#!/usr/bin/env node
/**
 * Says whether every file changed between two commits is documentation, so `pre-push`
 * and `ci.yml` can skip vitest, build and `cargo test` while still running the gates.
 * Not a gate: no `check:*` name, stays out of the three gate lists.
 *
 * Documentation is `*.md` and `_bmad-output/**`, except the root `AGENTS.md`
 * (`naming_boundary.rs` reads it) and `*-ban-do/` folders (Rust tests read them).
 *
 * Run:  node scripts/docs-only.mjs <from> <to>
 * Exit 0 = documentation only · 1 = something else changed · 2 = could not tell.
 */
import { spawnSync } from 'node:child_process'
import { pathToFileURL } from 'node:url'

/**
 * @param {string} path
 * @returns {boolean}
 */
export function isDocumentation(path) {
  if (path === 'AGENTS.md' || path.includes('-ban-do/')) return false
  return path.startsWith('_bmad-output/') || path.endsWith('.md')
}

/**
 * An empty list is not documentation-only: a diff that names no file proves nothing.
 * @param {string[]} paths
 * @returns {boolean}
 */
export function isDocsOnly(paths) {
  const named = paths.filter((p) => p !== '')
  return named.length > 0 && named.every(isDocumentation)
}

function main() {
  const [from, to] = process.argv.slice(2)
  if (!from || !to) {
    console.error('usage: node scripts/docs-only.mjs <from> <to>')
    process.exit(2)
  }
  const r = spawnSync('git', ['diff', '--name-only', from, to], { encoding: 'utf8' })
  if (r.error || r.status !== 0) {
    console.error(`git diff ${from} ${to} failed: ${r.error?.message ?? r.stderr}`)
    process.exit(2)
  }
  process.exit(isDocsOnly(r.stdout.split('\n')) ? 0 : 1)
}

const isMain = process.argv[1] !== undefined && import.meta.url === pathToFileURL(process.argv[1]).href
if (isMain) main()
