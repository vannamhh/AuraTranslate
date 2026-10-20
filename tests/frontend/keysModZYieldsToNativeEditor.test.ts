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

describe('handle() — hợp âm trần trên input không phải văn bản', () => {
  function bare(target: object, code: string) {
    const registry = createRegistry()
    const run = vi.fn()
    registry.register({ id: 'x.bare', labelKey: 'command.x.bare', run, keys: ['D', 'ArrowRight', 'Escape'] })
    const keymap = createKeymap(registry, { isMac: true })
    const preventDefault = vi.fn()
    return { handled: keymap.handle({ code, target, preventDefault }), run, preventDefault }
  }

  it('range: chữ chạy lệnh, mũi tên về control', () => {
    const range = { tagName: 'INPUT', type: 'range' }
    const d = bare(range, 'KeyD')
    expect(d.handled).toBe(true)
    expect(d.run).toHaveBeenCalledOnce()
    const arrow = bare(range, 'ArrowRight')
    expect(arrow.handled).toBe(false)
    expect(arrow.run).not.toHaveBeenCalled()
    expect(arrow.preventDefault).not.toHaveBeenCalled()
  })

  it('checkbox: chữ chạy lệnh, Escape về control', () => {
    expect(bare({ tagName: 'INPUT', type: 'checkbox' }, 'KeyD').run).toHaveBeenCalledOnce()
    const esc = bare({ tagName: 'INPUT', type: 'checkbox' }, 'Escape')
    expect(esc.handled).toBe(false)
    expect(esc.run).not.toHaveBeenCalled()
  })

  it('text: không dispatch', () => {
    const r = bare({ tagName: 'INPUT', type: 'text' }, 'KeyD')
    expect(r.handled).toBe(false)
    expect(r.run).not.toHaveBeenCalled()
  })
})
