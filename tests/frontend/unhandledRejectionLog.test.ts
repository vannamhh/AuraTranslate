import { afterEach, describe, expect, it, vi } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { createRegistry } from '../../src/commands/registry'
import { installUnhandledRejectionLog } from '../../src/unhandledRejectionLog'

/** What a browser fires when a promise rejects with no handler. */
function rejectionEvent(reason: unknown): Event {
  const event = new Event('unhandledrejection')
  Object.defineProperty(event, 'reason', { value: reason })
  return event
}

const disposers: Array<() => void> = []

afterEach(() => {
  while (disposers.length > 0) disposers.pop()?.()
  vi.restoreAllMocks()
})

describe('installUnhandledRejectionLog', () => {
  it('logs the cause of a command handler that rejects after an await', async () => {
    const logged = vi.spyOn(console, 'error').mockImplementation(() => {})
    disposers.push(installUnhandledRejectionLog())
    const cause = new Error('boom after await')
    const leaked: Promise<void>[] = []
    const registry = createRegistry()
    registry.register({
      id: 'test.async_throw',
      labelKey: 'command.test.async_throw',
      keys: undefined,
      run: () => {
        const work = (async () => {
          await Promise.resolve()
          throw cause
        })()
        leaked.push(work)
      },
    })

    expect(() => registry.dispatch('test.async_throw')).not.toThrow()
    // The runtime turns the rejected, unhandled promise into this window event.
    await leaked[0].catch((reason) => window.dispatchEvent(rejectionEvent(reason)))

    expect(logged).toHaveBeenCalledTimes(1)
    expect(logged.mock.calls[0]).toContain(cause)
  })

  it('stops logging once disposed', () => {
    const logged = vi.spyOn(console, 'error').mockImplementation(() => {})
    const dispose = installUnhandledRejectionLog()
    dispose()

    window.dispatchEvent(rejectionEvent(new Error('late')))

    expect(logged).not.toHaveBeenCalled()
  })
})

describe('main.ts wiring', () => {
  it('boot() installs the listener before anything can reject', () => {
    const source = readFileSync(resolve(process.cwd(), 'src/main.ts'), 'utf8')
      .replace(/\/\*[\s\S]*?\*\//g, '')
      .replace(/^\s*\/\/.*$/gm, '')
    const header = 'async function boot(): Promise<void> {'
    const body = source.slice(source.indexOf(header) + header.length)

    expect(body.trimStart().startsWith('installUnhandledRejectionLog()')).toBe(true)
  })
})
