import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

function vueFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name)
    if (statSync(path).isDirectory()) return name === 'selftest' ? [] : vueFiles(path)
    return name.endsWith('.vue') ? [path] : []
  })
}

function templateOf(source: string): string {
  const start = source.indexOf('\n<template>')
  const end = source.lastIndexOf('\n<style')
  return source.slice(start, end === -1 ? undefined : end)
}

function openTagOf(block: string): string {
  return block.match(/^<button\b(?:"[^"]*"|'[^']*'|[^>"'])*>/)![0]
}

function iconOnlyButtons(source: string): string[] {
  return (templateOf(source).match(/<button\b(?:"[^"]*"|'[^']*'|[^>"'])*>[\s\S]*?<\/button>/g) ?? []).filter((block) => {
    const inner = block.slice(openTagOf(block).length, block.lastIndexOf('</button>'))
    return /<AppIcon\b/.test(inner) && inner.replace(/<AppIcon\b[^>]*\/>/g, '').trim() === ''
  })
}

describe('buttons that carry only an icon', () => {
  const files = vueFiles(join(__dirname, '../../src'))

  it('every one names itself with an aria-label and a title', () => {
    let seen = 0
    for (const file of files) {
      for (const block of iconOnlyButtons(readFileSync(file, 'utf8'))) {
        seen += 1
        const open = openTagOf(block)
        expect(open, `${file}\n${block}`).toMatch(/:?aria-label=/)
        expect(open, `${file}\n${block}`).toMatch(/:?title=/)
      }
    }
    expect(seen).toBeGreaterThanOrEqual(7)
  })

  it('pin and layout buttons also state their pressed state', () => {
    for (const file of files) {
      for (const block of iconOnlyButtons(readFileSync(file, 'utf8'))) {
        if (!/name="pin|:name="[^"]*pin|name="layout-/.test(block)) continue
        expect(openTagOf(block), `${file}\n${block}`).toMatch(/aria-pressed=/)
      }
    }
  })
})
