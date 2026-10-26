import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

function codeLines(path: string): string[] {
  return readFileSync(resolve(process.cwd(), path), 'utf8')
    .split('\n')
    .filter((line) => !line.trim().startsWith('//'))
}

describe('main.ts nối các handler AI từ một module dùng chung', () => {
  const main = codeLines('src/main.ts')

  it('spread `aiTranslateHandlers` vào `installCommands`', () => {
    expect(main.some((l) => l.trim() === '...aiTranslateHandlers,')).toBe(true)
  })

  it('spread `aiPromptInspectorHandlers` vào `installCommands`', () => {
    expect(main.some((l) => l.trim() === '...aiPromptInspectorHandlers,')).toBe(true)
  })

  it('không định nghĩa lại handler nào trong main.ts', () => {
    for (const name of [
      'runAiTranslate',
      'cancelAiTranslate',
      'promoteAiTranslate',
      'runAiTranslateBatch',
      'runAiTranslateChapter',
      'retryAiTranslate',
      'retryAiTranslateBatch',
      'resendAiPrompt',
      'copyAiPrompt',
      'openAiPromptTmConcordance',
      'confirmAiPromptTermNow',
    ]) {
      expect(main.some((l) => new RegExp(`^\\s*${name}:`).test(l)), name).toBe(false)
    }
  })

  it('module xuất đúng mười handler', async () => {
    const { aiTranslateHandlers } = await import('../../src/aiTranslateHandlers')
    expect(Object.keys(aiTranslateHandlers).sort()).toEqual([
      'cancelAiTranslate',
      'discardAiTranslateBatch',
      'promoteAiTranslate',
      'promoteAiTranslateBatch',
      'resendAiPrompt',
      'retryAiTranslate',
      'retryAiTranslateBatch',
      'runAiTranslate',
      'runAiTranslateBatch',
      'runAiTranslateChapter',
    ])
  })

  it('module `aiPromptInspectorHandlers` xuất đúng bảy handler', async () => {
    const { aiPromptInspectorHandlers } = await import('../../src/aiPromptInspectorHandlers')
    expect(Object.keys(aiPromptInspectorHandlers).sort()).toEqual([
      'confirmAiPromptTermNow',
      'copyAiPrompt',
      'openAiPromptGlossaryEntry',
      'openAiPromptTmConcordance',
      'openAiPromptWorkPrompts',
      'selectAiPromptPromptTab',
      'selectAiPromptResultTab',
    ])
  })
})

describe('luồng AI đọc câu hiện tại từ một nguồn: editorActiveSegmentId, không phải con trỏ trần', () => {
  const nonComment = (path: string): string[] =>
    codeLines(path).filter((line) => !/^\s*(\*|\/\*)/.test(line))

  it.each([
    'src/aiTranslateHandlers.ts',
    'src/panels/AiTranslationPanel.vue',
    'src/AiPromptInspectorOverlay.vue',
    'src/aiPromptInspectorState.ts',
  ])('%s không đọc editorCaretSegmentId trong dòng mã', (path) => {
    expect(nonComment(path).filter((line) => line.includes('editorCaretSegmentId'))).toEqual([])
  })

  it('lệnh ai.prompt.assemble của main.ts lắp cho editorActiveSegmentId', () => {
    const lines = nonComment('src/main.ts').filter((line) => line.includes('assembleCurrentAiPrompt('))
    expect(lines.length).toBe(1)
    expect(lines[0]).toContain('editorActiveSegmentId.value')
  })
})
