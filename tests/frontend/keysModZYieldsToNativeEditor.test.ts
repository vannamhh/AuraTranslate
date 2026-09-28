/**
 * `commands/keys.ts::handle` — trong một vùng gõ, `Mod+Z`/`Mod+Shift+Z` LUÔN nhường trình
 * soạn thảo gốc, BẤT KỂ có command nào đăng ký hai hợp âm đó hay không — trình soạn thảo
 * `contenteditable` giữ ngăn xếp hoàn tác RIÊNG của nó.
 */
import { describe, expect, it, vi } from 'vitest'
import { createRegistry } from '../../src/commands/registry'
import { createKeymap } from '../../src/commands/keys'
import type { ChordEvent } from '../../src/commands/keys'

const TYPING_TARGET = { tagName: 'DIV', isContentEditable: true }

function registryWithUndoBoundToModZ() {
  const registry = createRegistry()
  const run = vi.fn()
  registry.register({ id: 'editor.some_future_undo', labelKey: 'command.editor.some_future_undo', run, keys: ['Mod+Z'] })
  return { registry, run }
}

describe('handle() — Mod+Z trong vùng gõ nhường trình soạn thảo gốc, kể cả khi một command ĐÃ đăng ký hợp âm đó', () => {
  it('target là vùng gõ ⇒ handle() trả `false`, command KHÔNG chạy, KHÔNG preventDefault', () => {
    const { registry, run } = registryWithUndoBoundToModZ()
    const keymap = createKeymap(registry, { isMac: true })
    const preventDefault = vi.fn()
    const event: ChordEvent = { code: 'KeyZ', metaKey: true, target: TYPING_TARGET, preventDefault }

    expect(keymap.handle(event)).toBe(false)
    expect(run).not.toHaveBeenCalled()
    expect(preventDefault).not.toHaveBeenCalled()
  })

  it('target KHÔNG phải vùng gõ ⇒ command đã đăng ký vẫn chạy bình thường', () => {
    const { registry, run } = registryWithUndoBoundToModZ()
    const keymap = createKeymap(registry, { isMac: true })
    const event: ChordEvent = { code: 'KeyZ', metaKey: true, target: { tagName: 'DIV' } }

    expect(keymap.handle(event)).toBe(true)
    expect(run).toHaveBeenCalledOnce()
  })

  it('ctrlKey (nền không phải Mac) ⇒ vẫn nhường, cùng luật với metaKey', () => {
    const { registry, run } = registryWithUndoBoundToModZ()
    const keymap = createKeymap(registry, { isMac: false })
    const preventDefault = vi.fn()
    const event: ChordEvent = { code: 'KeyZ', ctrlKey: true, target: TYPING_TARGET, preventDefault }

    expect(keymap.handle(event)).toBe(false)
    expect(run).not.toHaveBeenCalled()
    expect(preventDefault).not.toHaveBeenCalled()
  })

  it('Mod+Shift+Z ⇒ vẫn nhường trình soạn thảo gốc', () => {
    const { registry, run } = registryWithUndoBoundToModZ()
    const keymap = createKeymap(registry, { isMac: true })
    const preventDefault = vi.fn()
    const event: ChordEvent = {
      code: 'KeyZ',
      metaKey: true,
      shiftKey: true,
      target: TYPING_TARGET,
      preventDefault,
    }

    expect(keymap.handle(event)).toBe(false)
    expect(run).not.toHaveBeenCalled()
    expect(preventDefault).not.toHaveBeenCalled()
  })
})
