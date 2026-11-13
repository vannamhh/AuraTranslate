import { describe, expect, it, vi } from 'vitest'
import { createRegistry } from '../../src/commands/registry'
import { createKeymap } from '../../src/commands/keys'
import type { ChordEvent } from '../../src/commands/keys'

const TRANSLATION_CELL = { tagName: 'SPAN', isContentEditable: true }

function enterIn(target: object, mods: Partial<ChordEvent> = {}) {
  const preventDefault = vi.fn()
  const event: ChordEvent = { code: 'Enter', target, preventDefault, ...mods }
  return { event, preventDefault }
}

describe('Shift+Enter gán lại cho editor.confirm_segment', () => {
  it('xác nhận câu ngay trong ô bản dịch khi đi qua installCommands thật', async () => {
    vi.resetModules()
    const commands = await import('../../src/commands')
    const confirmSegment = vi.fn()
    const keymap = commands.installCommands({
      isMac: true,
      setMode: () => {},
      confirmSegment,
      bindings: { 'editor.confirm_segment': ['Shift+Enter'], 'library.continue_translation': [] },
    })
    const { event, preventDefault } = enterIn(TRANSLATION_CELL, { shiftKey: true })

    expect(keymap.handle(event)).toBe(true)
    expect(confirmSegment).toHaveBeenCalledOnce()
    expect(preventDefault).toHaveBeenCalledOnce()
  })

  it('không xác nhận khi bộ gõ đang dựng chữ', async () => {
    vi.resetModules()
    const commands = await import('../../src/commands')
    const confirmSegment = vi.fn()
    const keymap = commands.installCommands({
      isMac: true,
      setMode: () => {},
      confirmSegment,
      bindings: { 'editor.confirm_segment': ['Shift+Enter'], 'library.continue_translation': [] },
    })

    expect(keymap.handle(enterIn(TRANSLATION_CELL, { shiftKey: true, isComposing: true }).event)).toBe(false)
    expect(confirmSegment).not.toHaveBeenCalled()
  })

  it('không xác nhận khi đang gõ trong một ô input ngoài lưới, như ô thêm thuật ngữ', async () => {
    vi.resetModules()
    const commands = await import('../../src/commands')
    const confirmSegment = vi.fn()
    const keymap = commands.installCommands({
      isMac: true,
      setMode: () => {},
      confirmSegment,
      bindings: { 'editor.confirm_segment': ['Shift+Enter'], 'library.continue_translation': [] },
    })
    const { event, preventDefault } = enterIn({ tagName: 'INPUT', type: 'text' }, { shiftKey: true })

    expect(keymap.handle(event)).toBe(false)
    expect(confirmSegment).not.toHaveBeenCalled()
    expect(preventDefault).not.toHaveBeenCalled()
  })

  it('với bộ phím mặc định, Shift+Enter trong ô bản dịch vẫn để ô xuống dòng', async () => {
    vi.resetModules()
    const commands = await import('../../src/commands')
    const confirmSegment = vi.fn()
    const continueCurrentLibraryTranslation = vi.fn()
    const keymap = commands.installCommands({ isMac: true, setMode: () => {}, confirmSegment, continueCurrentLibraryTranslation })
    const { event, preventDefault } = enterIn(TRANSLATION_CELL, { shiftKey: true })

    expect(keymap.handle(event)).toBe(false)
    expect(confirmSegment).not.toHaveBeenCalled()
    expect(continueCurrentLibraryTranslation).not.toHaveBeenCalled()
    expect(preventDefault).not.toHaveBeenCalled()
  })
})

describe('chỉ đúng Shift+Enter của lệnh khai cờ mới vượt vùng gõ', () => {
  function keymapWith(chord: string, shiftEnterWhileTyping: boolean) {
    const registry = createRegistry()
    const run = vi.fn()
    registry.register({ id: 'x.confirm', labelKey: 'command.x.confirm', run, keys: [chord], shiftEnterWhileTyping })
    return { keymap: createKeymap(registry, { isMac: true }), run }
  }

  it('Enter trần của lệnh có cờ vẫn nhường ô bản dịch để xuống dòng', () => {
    const { keymap, run } = keymapWith('Enter', true)
    const { event, preventDefault } = enterIn(TRANSLATION_CELL)

    expect(keymap.handle(event)).toBe(false)
    expect(run).not.toHaveBeenCalled()
    expect(preventDefault).not.toHaveBeenCalled()
  })

  it('Alt+Enter của lệnh có cờ vẫn nhường ô bản dịch', () => {
    const { keymap, run } = keymapWith('Alt+Enter', true)

    expect(keymap.handle(enterIn(TRANSLATION_CELL, { altKey: true }).event)).toBe(false)
    expect(run).not.toHaveBeenCalled()
  })

  it('Alt+Shift+Enter của lệnh có cờ vẫn nhường ô bản dịch', () => {
    const { keymap, run } = keymapWith('Alt+Shift+Enter', true)

    expect(keymap.handle(enterIn(TRANSLATION_CELL, { altKey: true, shiftKey: true }).event)).toBe(false)
    expect(run).not.toHaveBeenCalled()
  })

  it('Shift+Enter của lệnh không khai cờ vẫn nhường ô bản dịch', () => {
    const { keymap, run } = keymapWith('Shift+Enter', false)

    expect(keymap.handle(enterIn(TRANSLATION_CELL, { shiftKey: true }).event)).toBe(false)
    expect(run).not.toHaveBeenCalled()
  })

  it('Shift+Enter của lệnh có cờ chạy trong ô bản dịch', () => {
    const { keymap, run } = keymapWith('Shift+Enter', true)

    expect(keymap.handle(enterIn(TRANSLATION_CELL, { shiftKey: true }).event)).toBe(true)
    expect(run).toHaveBeenCalledOnce()
  })
})
