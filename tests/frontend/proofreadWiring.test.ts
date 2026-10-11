import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'
import type { ChapterSegment } from '../../src/config/segment'
import { FIXTURE_SEGMENTS } from './support/segmentFixture'

const invokeMock = vi.fn()

function codeLines(path: string): string[] {
  return readFileSync(resolve(process.cwd(), path), 'utf8')
    .split('\n')
    .filter((line) => !line.trim().startsWith('//'))
}

function seg(id: number, patch: Partial<ChapterSegment> = {}): ChapterSegment {
  return { ...FIXTURE_SEGMENTS[0], id, ord: id, source_text: `src ${String(id)}`, target_text: `dich ${String(id)}`, ...patch }
}

function proofreadCalls(): unknown[][] {
  return invokeMock.mock.calls.filter((c) => String(c[0]).startsWith('ai_proofread_'))
}

async function fresh() {
  vi.resetModules()
  invokeMock.mockReset()
  invokeMock.mockImplementation((cmd: string) =>
    Promise.resolve(cmd.startsWith('ai_translate_') ? { state: 'done', usage: null } : { state: 'done', not_scanned: 0 }),
  )
  const caret = ref<number | null>(4)
  const segments = ref<ChapterSegment[]>([seg(3), seg(4), seg(5, { target_text: '' }), seg(6, { is_omitted: true }), seg(7)])
  const selection = ref<number[]>([])
  const flush = vi.fn(() => Promise.resolve('clean'))
  vi.doMock('@tauri-apps/api/core', () => ({
    invoke: (...args: unknown[]) => invokeMock(...args),
    Channel: class {
      onmessage: (v: unknown) => void = () => {}
    },
  }))
  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: caret,
    editorSegments: segments,
    flushEditorBeforeDiscreteWrite: flush,
    promoteAiTranslationToEditor: vi.fn(),
  }))
  vi.doMock('../../src/panels/segmentSelectionState', () => ({ segmentSelectionIds: selection }))
  const commands = await import('../../src/commands')
  const { proofreadHandlers } = await import('../../src/proofreadHandlers')
  const { aiTranslateHandlers } = await import('../../src/aiTranslateHandlers')
  const state = await import('../../src/proofreadState')
  commands.installCommands({ ...proofreadHandlers, ...aiTranslateHandlers } as Parameters<typeof commands.installCommands>[0])
  return { commands, caret, segments, selection, flush, state }
}

describe('ai.proofread commands', () => {
  beforeEach(() => {
    vi.doUnmock('@tauri-apps/api/core')
  })

  it('ai.proofread.run scans only the segment that has the caret', async () => {
    const { commands } = await fresh()
    commands.dispatch('ai.proofread.run')
    await vi.waitFor(() => {
      expect(proofreadCalls()).toHaveLength(1)
    })
    expect(proofreadCalls()[0]?.[0]).toBe('ai_proofread_segments')
    expect((proofreadCalls()[0]?.[1] as { segmentIds: number[] }).segmentIds).toEqual([4])
  })

  it('ai.proofread.run_selection scans exactly the selected ids, in the order given', async () => {
    const { commands, selection } = await fresh()
    selection.value = [3, 4, 5, 6, 7]
    commands.dispatch('ai.proofread.run_selection')
    await vi.waitFor(() => {
      expect(proofreadCalls()).toHaveLength(1)
    })
    expect((proofreadCalls()[0]?.[1] as { segmentIds: number[] }).segmentIds).toEqual([3, 4, 5, 6, 7])
  })

  it('ai.proofread.run_chapter scans every segment of the open Chapter in document order', async () => {
    const { commands } = await fresh()
    commands.dispatch('ai.proofread.run_chapter')
    await vi.waitFor(() => {
      expect(proofreadCalls()).toHaveLength(1)
    })
    expect((proofreadCalls()[0]?.[1] as { segmentIds: number[] }).segmentIds).toEqual([3, 4, 5, 6, 7])
  })

  it('run_selection with nothing selected makes no call', async () => {
    const { commands, state } = await fresh()
    commands.dispatch('ai.proofread.run_selection')
    await Promise.resolve()
    expect(proofreadCalls()).toHaveLength(0)
    expect(state.proofreadStateValue.value).toBe('idle')
  })

  it('dispatching ai.proofread.cancel while scanning sends the Rust cancel', async () => {
    const { commands, state } = await fresh()
    invokeMock.mockImplementation((cmd: string) => (cmd === 'ai_proofread_segments' ? new Promise(() => {}) : Promise.resolve()))
    commands.dispatch('ai.proofread.run_chapter')
    await vi.waitFor(() => {
      expect(proofreadCalls()).toHaveLength(1)
    })
    commands.dispatch('ai.proofread.cancel')
    expect(proofreadCalls().map((c) => c[0])).toEqual(['ai_proofread_segments', 'ai_proofread_cancel'])
    state.resetProofread()
  })

  it('the chooser commands open and close it, and a scan closes it', async () => {
    const { commands, state } = await fresh()
    commands.dispatch('ai.proofread.scope.open')
    expect(state.proofreadScopeIsOpen.value).toBe(true)
    commands.dispatch('ai.proofread.scope.close')
    expect(state.proofreadScopeIsOpen.value).toBe(false)
    commands.dispatch('ai.proofread.scope.open')
    commands.dispatch('ai.proofread.run')
    expect(state.proofreadScopeIsOpen.value).toBe(false)
    await vi.waitFor(() => {
      expect(proofreadCalls()).toHaveLength(1)
    })
  })

  it('binds Mod+P to the segment, Mod+Shift+P to the Chapter, and no key to the selection', () => {
    const source = codeLines('src/commands/index.ts').join('\n')
    expect(source).toMatch(/id: 'ai\.proofread\.run',\s+labelKey: 'command\.ai\.proofread\.run',\s+keys: \['Mod\+P'\]/)
    expect(source).toMatch(/id: 'ai\.proofread\.run_chapter',\s+labelKey: 'command\.ai\.proofread\.run_chapter',\s+keys: \['Mod\+Shift\+P'\]/)
    expect(source).toMatch(/id: 'ai\.proofread\.run_selection',\s+labelKey: 'command\.ai\.proofread\.run_selection',\s+keys: undefined/)
  })
})

describe('a proofreader that is never called makes no call', () => {
  it('typing, flushing, changing Chapter, selecting and the chooser produce 0 ai_proofread_* invokes; one dispatch produces 1', async () => {
    const { commands, caret, segments, selection, flush, state } = await fresh()
    caret.value = 5
    selection.value = [3, 4]
    segments.value = [seg(10), seg(11)]
    state.resetProofread()
    commands.dispatch('ai.proofread.scope.open')
    commands.dispatch('ai.proofread.scope.close')
    commands.dispatch('ai.translate.run')
    await vi.waitFor(() => {
      expect(invokeMock.mock.calls.some((c) => c[0] === 'ai_translate_segment')).toBe(true)
    })
    await new Promise((r) => setTimeout(r, 0))
    selection.value = [10, 11]
    commands.dispatch('ai.translate.batch_run')
    await vi.waitFor(() => {
      expect(invokeMock.mock.calls.some((c) => c[0] === 'ai_translate_batch')).toBe(true)
    })
    state.clearProofreadFor(10)
    await flush()
    await new Promise((r) => setTimeout(r, 20))
    expect(proofreadCalls()).toHaveLength(0)

    caret.value = 10
    commands.dispatch('ai.proofread.run')
    await vi.waitFor(() => {
      expect(proofreadCalls()).toHaveLength(1)
    })
  })
})

describe('proofread wiring in source', () => {
  it('main.ts spreads proofreadHandlers and blocks chords while the chooser is open', () => {
    const main = codeLines('src/main.ts')
    expect(main.some((l) => l.trim() === '...proofreadHandlers,')).toBe(true)
    expect(main.some((l) => l.includes('proofreadScopeIsOpen.value'))).toBe(true)
  })

  it.each(['src/modes/libraryChapters.ts', 'src/modes/libraryImport.ts', 'src/panels/editorPanelState.ts'])('%s resets the proofread state', (path) => {
    expect(codeLines(path).some((l) => l.trim() === 'resetProofread()')).toBe(true)
  })

  it('GridPanel clears one cell on a typed edit and paints every scanned cell in one call', () => {
    const grid = codeLines('src/panels/GridPanel.vue').join('\n')
    expect(grid).toMatch(/noteEditorEdit\(id, cell\.textContent\)\s+clearProofreadFor\(id\)/)
    expect(grid).toContain('for (const [id, entry] of proofreadResults.value)')
    expect(grid).toContain('rangesForScan(cell, entry.scannedText, entry.findings.filter((f) => !isMeaningKind(f.kind)))')
    expect(grid).toContain('rangesForScan(cell, entry.scannedText, entry.findings.filter((f) => isMeaningKind(f.kind)))')
    expect(grid).toContain('paintProofreadRanges(errorRanges, meaningRanges)')
  })

  it('App.vue mounts the chooser behind a titlebar button that dispatches the open command', () => {
    const app = codeLines('src/App.vue').join('\n')
    expect(app).toContain('<ProofreadScopeOverlay />')
    expect(app).toMatch(/data-proofread-scope-open[\s\S]{0,200}dispatch\('ai\.proofread\.scope\.open'\)/)
  })
})
