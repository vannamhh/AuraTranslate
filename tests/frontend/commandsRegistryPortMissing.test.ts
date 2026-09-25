import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { CommandDeps } from '../../src/commands'

const cleanups: Array<() => void> = []
let errSpy: ReturnType<typeof vi.spyOn>

beforeEach(() => {
  errSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
})

afterEach(() => {
  errSpy.mockRestore()
  for (const fn of cleanups.splice(0)) fn()
  document.body.innerHTML = ''
})

async function freshCommands(deps: CommandDeps) {
  vi.resetModules()
  const commands = await import('../../src/commands')
  commands.installCommands(deps)
  return commands
}

/** Có ít nhất một lượt gọi `console.error` nhắc đúng \`id\` command, dạng `portMissing`. */
function daBaoPortMissing(id: string): boolean {
  return (errSpy.mock.calls as unknown[][]).some((call) => String(call[0]).includes(`\`${id}\``))
}

describe('CommandRegistry — `portMissing` khi cổng vắng mặt', () => {
  it('mọi command trừ `mode.*` báo `portMissing`, nhắc đúng id của chính nó', async () => {
    const commands = await freshCommands({ isMac: true, setMode: () => {} })
    const modeIds = new Set(commands.MODE_IDS.map((mode) => `mode.${mode}`))
    const checked: string[] = []

    for (const spec of commands.commandRegistry.list()) {
      if (modeIds.has(spec.id)) continue
      errSpy.mockClear()
      expect(() => spec.run()).not.toThrow()
      expect(daBaoPortMissing(spec.id)).toBe(true)
      checked.push(spec.id)
    }

    // Cận dưới đo được (177 command đăng ký, trừ 3 `mode.*`): một con số rơi mạnh ở đây
    // báo một nhóm command đã ngừng đăng ký, không phải một lượt siết ngưỡng.
    expect(checked.length).toBeGreaterThanOrEqual(150)
  })

  it('`mode.*` không báo `portMissing` — `setMode` là cổng bắt buộc, luôn có mặt', async () => {
    const commands = await freshCommands({ isMac: true, setMode: () => {} })
    const specs = commands.commandRegistry.list()

    for (const mode of commands.MODE_IDS) {
      const spec = specs.find((s) => s.id === `mode.${mode}`)
      expect(spec).toBeDefined()
      spec?.run()
    }

    expect(errSpy).not.toHaveBeenCalled()
  })
})

describe('`Mod+Alt+C` qua keymap thật ⇒ `glossary.confirm.focus`', () => {
  function ganPhim(commands: Awaited<ReturnType<typeof freshCommands>>) {
    const host = document.createElement('div')
    document.body.appendChild(host)
    const detach = commands.attachKeyboard(host)
    cleanups.push(() => {
      detach()
      host.remove()
    })
    return host
  }

  function banModAltC(host: HTMLElement) {
    host.dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'c',
        code: 'KeyC',
        metaKey: true,
        altKey: true,
        bubbles: true,
        cancelable: true,
      }),
    )
  }

  it('cổng `focusGlossaryConfirmStrip` vắng mặt ⇒ `portMissing` gọi đúng id + cổng', async () => {
    const commands = await freshCommands({ isMac: true, setMode: () => {} })

    banModAltC(ganPhim(commands))

    expect(errSpy).toHaveBeenCalledTimes(1)
    const message = String(errSpy.mock.calls[0]?.[0])
    expect(message).toContain('`glossary.confirm.focus`')
    expect(message).toContain('`focusGlossaryConfirmStrip`')
  })

  it('cổng có mặt ⇒ hợp âm chạy THẬT, không `portMissing`', async () => {
    const goiVoi: string[] = []
    const commands = await freshCommands({
      isMac: true,
      setMode: () => {},
      focusGlossaryConfirmStrip: (prefill) => goiVoi.push(prefill),
    })

    banModAltC(ganPhim(commands))

    expect(errSpy).not.toHaveBeenCalled()
    expect(goiVoi).toEqual([''])
  })
})
