import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

const runMock = vi.fn()
const cancelMock = vi.fn()

function codeLines(path: string): string[] {
  return readFileSync(resolve(process.cwd(), path), 'utf8')
    .split('\n')
    .filter((line) => !line.trim().startsWith('//'))
}

async function fresh() {
  vi.resetModules()
  runMock.mockReset()
  cancelMock.mockReset()
  runMock.mockResolvedValue({ value: { state: 'cancelled' }, error: null })
  const caret = ref<number | null>(4)
  vi.doMock('../../src/config/proofread', () => ({
    runProofreadSegment: (...args: unknown[]) => runMock(...args),
    cancelProofreadCall: (...args: unknown[]) => cancelMock(...args),
  }))
  vi.doMock('../../src/panels/editorPanelState', () => ({
    editorCaretSegmentId: caret,
    flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
  }))
  const commands = await import('../../src/commands')
  const { proofreadHandlers } = await import('../../src/proofreadHandlers')
  const state = await import('../../src/proofreadState')
  commands.installCommands({ ...proofreadHandlers } as Parameters<typeof commands.installCommands>[0])
  return { commands, caret, state }
}

describe('ai.proofread commands', () => {
  beforeEach(() => {
    vi.doUnmock('../../src/config/proofread')
  })

  it('dispatching ai.proofread.run scans the segment that has the caret', async () => {
    const { commands, state } = await fresh()
    commands.dispatch('ai.proofread.run')
    await vi.waitFor(() => {
      expect(state.proofreadStateValue.value).toBe('cancelled')
    })
    expect(runMock.mock.calls[0]?.[0]).toBe(4)
  })

  it('dispatching ai.proofread.cancel while scanning sends the Rust cancel', async () => {
    const { commands, state } = await fresh()
    runMock.mockReturnValue(new Promise(() => {}))
    commands.dispatch('ai.proofread.run')
    await vi.waitFor(() => {
      expect(runMock).toHaveBeenCalled()
    })
    commands.dispatch('ai.proofread.cancel')
    expect(cancelMock).toHaveBeenCalledTimes(1)
    state.resetProofread()
  })

  it('Mod+P is bound to ai.proofread.run', () => {
    const source = codeLines('src/commands/index.ts').join('\n')
    expect(source).toMatch(/id: 'ai\.proofread\.run',\s+labelKey: 'command\.ai\.proofread\.run',\s+keys: \['Mod\+P'\]/)
  })
})

describe('proofread wiring in source', () => {
  it('main.ts spreads proofreadHandlers', () => {
    expect(codeLines('src/main.ts').some((l) => l.trim() === '...proofreadHandlers,')).toBe(true)
  })

  it.each(['src/modes/libraryChapters.ts', 'src/modes/libraryImport.ts', 'src/panels/editorPanelState.ts'])('%s resets the proofread state', (path) => {
    expect(codeLines(path).some((l) => l.trim() === 'resetProofread()')).toBe(true)
  })

  it('GridPanel clears underlines on a typed edit and repaints through the text guard', () => {
    const grid = codeLines('src/panels/GridPanel.vue').join('\n')
    expect(grid).toMatch(/noteEditorEdit\(id, cell\.textContent\)\s+clearProofreadFor\(id\)/)
    expect(grid).toContain('rangesForScan(cell, proofreadScannedText.value, proofreadFindings.value)')
  })
})
