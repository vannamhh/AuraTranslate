import { afterEach, describe, expect, it, vi } from 'vitest'
import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, resolve } from 'node:path'

const SRC = resolve(__dirname, '../../src')
const app = readFileSync(join(SRC, 'App.vue'), 'utf8')
const mainTs = readFileSync(join(SRC, 'main.ts'), 'utf8')

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name)
    if (statSync(path).isDirectory()) return sourceFiles(path)
    return /\.(vue|ts)$/.test(name) ? [path] : []
  })
}

describe('title bar drag region (AD-58)', () => {
  it('App.vue carries exactly one drag attribute, on header.titlebar, valued by the macOS-only binding', () => {
    expect(app.match(/data-tauri-drag-region/g) ?? []).toHaveLength(1)
    const header = /<header class="titlebar"([^>]*)>/.exec(app)?.[1] ?? ''
    expect(header).toContain(':data-tauri-drag-region="titlebarDragRegion"')
    expect(app).toMatch(/const titlebarDragRegion = detectIsMac\(\) \? 'deep' : undefined/)
  })

  it('src/ never imports the window API or calls plugin:window|', () => {
    const banned = [/@tauri-apps\/api\/window/, /@tauri-apps\/api\/webviewWindow/, /plugin:window\|/]
    const hits = sourceFiles(SRC).flatMap((file) => {
      const text = readFileSync(file, 'utf8')
      return banned.filter((re) => re.test(text)).map((re) => `${file}: ${re}`)
    })
    expect(hits).toEqual([])
  })

  it('main.ts sets the platform marker only on macOS and before mount', () => {
    const marker = mainTs.indexOf("if (detectIsMac()) document.documentElement.dataset.platform = 'mac'")
    expect(marker).toBeGreaterThan(-1)
    expect(marker).toBeLessThan(mainTs.indexOf("createApp(App).mount('#app')"))
  })

  describe('platform detection that gates marker and drag region', () => {
    afterEach(() => {
      vi.unstubAllGlobals()
      vi.resetModules()
    })

    async function detect(platform: string): Promise<boolean> {
      vi.resetModules()
      vi.stubGlobal('navigator', { platform })
      const { detectIsMac } = await import('../../src/commands')
      return detectIsMac()
    }

    it('is true on MacIntel and false on Win32', async () => {
      expect(await detect('MacIntel')).toBe(true)
      expect(await detect('Win32')).toBe(false)
    })
  })

  it('the macOS inset applies only under the platform marker', () => {
    expect(app).toMatch(/:root\[data-platform='mac'\] \.titlebar \{\s*padding-left: var\(--space-traffic-light-inset\);/)
    const base = /\n\.titlebar \{[^}]*\}/.exec(app)?.[0] ?? ''
    expect(base).not.toContain('traffic-light-inset')
  })
})
