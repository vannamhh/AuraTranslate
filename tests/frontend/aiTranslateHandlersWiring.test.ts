import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

function codeLines(path: string): string[] {
  return readFileSync(resolve(process.cwd(), path), 'utf8')
    .split('\n')
    .filter((line) => !line.trim().startsWith('//'))
}

describe('main.ts nối sáu handler AI từ một module dùng chung', () => {
  const main = codeLines('src/main.ts')

  it('spread `aiTranslateHandlers` vào `installCommands`', () => {
    expect(main.some((l) => l.trim() === '...aiTranslateHandlers,')).toBe(true)
  })

  it('không định nghĩa lại handler nào trong main.ts', () => {
    for (const name of [
      'runAiTranslate',
      'cancelAiTranslate',
      'promoteAiTranslate',
      'runAiTranslateBatch',
      'retryAiTranslate',
      'retryAiTranslateBatch',
    ]) {
      expect(main.some((l) => new RegExp(`^\\s*${name}:`).test(l)), name).toBe(false)
    }
  })

  it('module xuất đúng sáu handler', async () => {
    const { aiTranslateHandlers } = await import('../../src/aiTranslateHandlers')
    expect(Object.keys(aiTranslateHandlers).sort()).toEqual([
      'cancelAiTranslate',
      'promoteAiTranslate',
      'retryAiTranslate',
      'retryAiTranslateBatch',
      'runAiTranslate',
      'runAiTranslateBatch',
    ])
  })
})
