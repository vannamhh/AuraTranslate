/**
 * `keymap` is a plain module variable, so Vue learns of a keymap change only through
 * `bindingsEpoch`. The invariant: `applyBindings(` is called from `commitBindings` alone, and
 * `commitBindings` is the only place the epoch is bumped.
 *
 * Reads source text, comments stripped, so a commented-out call never counts.
 */
import { describe, expect, it } from 'vitest'
import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, relative, resolve } from 'node:path'

const SRC = resolve(process.cwd(), 'src')

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name)
    if (statSync(path).isDirectory()) return sourceFiles(path)
    return /\.(ts|vue)$/.test(name) ? [path] : []
  })
}

function codeOnly(text: string): string {
  return text
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/'(?:[^'\\\n]|\\.)*'/g, "''")
    .replace(/^\s*\/\/.*$/gm, '')
    .replace(/\s\/\/.*$/gm, '')
}

function sites(needle: RegExp): Array<{ file: string; text: string; index: number }> {
  const found: Array<{ file: string; text: string; index: number }> = []
  for (const path of sourceFiles(SRC)) {
    const text = codeOnly(readFileSync(path, 'utf8'))
    for (const m of text.matchAll(needle)) {
      found.push({ file: relative(SRC, path), text, index: m.index })
    }
  }
  return found
}

function enclosingFunction(text: string, index: number): string | null {
  const before = text.slice(0, index)
  const matches = [...before.matchAll(/(?:^|\n)(?:export )?(?:async )?function (\w+)\(/g)]
  return matches.at(-1)?.[1] ?? null
}

describe('bindingsEpoch is bumped on the only path that rewrites the keymap', () => {
  it('applyBindings( has exactly one caller, and it is commitBindings in shortcutsState.ts', () => {
    const callers = sites(/(?<!function )\bapplyBindings\(/g)

    expect(callers.map((c) => `${c.file}::${enclosingFunction(c.text, c.index)}`)).toEqual([
      'config/shortcutsState.ts::commitBindings',
    ])
  })

  it('the epoch is incremented only inside commitBindings, and there it follows the apply', () => {
    const bumps = sites(/bindingsEpoch\.value\s*(?:\+\+|--|[-+*/]?=(?!=))|(?:\+\+|--)\s*bindingsEpoch\.value/g)

    expect(bumps.map((b) => `${b.file}::${enclosingFunction(b.text, b.index)}`)).toEqual([
      'config/shortcutsState.ts::commitBindings',
    ])
    const { text } = bumps[0]
    const body = text.slice(text.indexOf('function commitBindings('))
    expect(body.indexOf('applyBindings(')).toBeGreaterThan(-1)
    expect(body.indexOf('applyBindings(')).toBeLessThan(body.indexOf('bindingsEpoch.value'))
  })
})
