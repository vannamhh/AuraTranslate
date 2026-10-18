import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import type { SourceAttribution } from '../../src/config/dict'

const BASE: SourceAttribution = {
  code: 'src-a',
  display_name: 'Nguồn A',
  license_kind: 'open',
  license_id: 'CC-BY-SA-4.0',
  license_text_len: 12,
  attribution: 'Ghi công A',
  source_version: '2024-01',
  source_url: 'https://example.invalid/a',
  lang: 'zh',
  layer: 'layer-a',
  is_base: true,
}
const OTHER: SourceAttribution = {
  ...BASE,
  code: 'src-b',
  display_name: 'Nguồn B',
  license_kind: 'copyrighted',
  license_id: null,
  license_text_len: 0,
  lang: 'en',
  is_base: false,
}

let sourcesResult: SourceAttribution[] = [BASE, OTHER]
let licenseResult: { ok: string } | { err: object } = { ok: 'LICENSE BODY' }

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string, args?: { code?: string }) => {
    if (cmd === 'list_dict_sources') return { sources: sourcesResult, skipped: [] }
    if (cmd === 'read_dict_license_text') {
      if ('err' in licenseResult) throw licenseResult.err
      return `${licenseResult.ok} ${args?.code}`
    }
    throw new Error(`unknown command ${cmd}`)
  }),
}))

async function setup() {
  const state = await import('../../src/panels/dictSourcesState')
  const commands = await import('../../src/commands')
  const { default: AttributionOverlay } = await import('../../src/AttributionOverlay.vue')
  commands.installCommands({
    openAttribution: state.openAttribution,
    closeAttribution: state.closeAttribution,
    openLicenseText: () => void state.openAimedLicenseText(),
    copyCredit: () => void state.copyAimedCredit(),
    closeLicenseText: state.closeLicenseText,
  } as unknown as Parameters<typeof commands.installCommands>[0])
  await state.loadDictSources('')
  state.openAttribution()
  const w = mount(AttributionOverlay, { attachTo: document.body })
  await w.vm.$nextTick()
  return { state, commands, w }
}

describe('licenseKeyFor', () => {
  it('maps every known kind and falls back for an unknown one', async () => {
    const { licenseKeyFor, LICENSE_FALLBACK_KEY } = await import('../../src/panels/dictSourcesState')
    expect(licenseKeyFor('open')).toBe('attribution.license_open')
    expect(licenseKeyFor('public-domain')).toBe('attribution.license_public_domain')
    expect(licenseKeyFor('copyrighted')).toBe('attribution.license_copyrighted')
    expect(licenseKeyFor('unknown')).toBe('attribution.license_unknown')
    expect(licenseKeyFor('author-grant')).toBe('attribution.license_author_grant')
    expect(licenseKeyFor('something-new')).toBe(LICENSE_FALLBACK_KEY)
    expect(licenseKeyFor('')).toBe(LICENSE_FALLBACK_KEY)
  })
})

describe('conversionNoticeApplies', () => {
  it('keys on the licence id prefix only', async () => {
    const { conversionNoticeApplies } = await import('../../src/panels/dictSourcesState')
    expect(conversionNoticeApplies('CC-BY-SA-4.0')).toBe(true)
    expect(conversionNoticeApplies('CC-BY-SA-3.0')).toBe(true)
    expect(conversionNoticeApplies('CC-BY-4.0')).toBe(false)
    expect(conversionNoticeApplies('Unicode-3.0')).toBe(false)
    expect(conversionNoticeApplies(null)).toBe(false)
  })
})

describe('roleLangsFor', () => {
  it('maps languages to distinct keys, with a fallback keeping the raw value', async () => {
    const { roleLangsFor } = await import('../../src/panels/dictSourcesState')
    const [zh] = roleLangsFor('zh')
    const [en] = roleLangsFor('en')
    expect(zh.key).not.toBe(en.key)
    expect(roleLangsFor('xx')).toEqual([{ key: 'attribution.role_lang_other', lang: 'xx' }])
    expect(roleLangsFor('zh, en')).toHaveLength(2)
    expect(roleLangsFor('')).toEqual([])
  })
})

describe('AttributionOverlay actions', () => {
  beforeEach(() => {
    sourcesResult = [BASE, OTHER]
    licenseResult = { ok: 'LICENSE BODY' }
  })
  afterEach(() => {
    document.body.innerHTML = ''
    vi.resetModules()
    vi.unstubAllGlobals()
  })

  const footButtons = (w: Awaited<ReturnType<typeof setup>>['w']) => w.findAll('.attr-foot button')
  const settle = async (w: Awaited<ReturnType<typeof setup>>['w']) => {
    await w.vm.$nextTick()
    await w.vm.$nextTick()
  }

  it('shows the conversion notice only on the CC-BY-SA row, and no row carries an action button', async () => {
    const { w } = await setup()
    const rows = w.findAll('tbody tr')
    expect(rows[0].find('.attr-conversion').exists()).toBe(true)
    expect(rows[1].find('.attr-conversion').exists()).toBe(false)
    expect(w.find('tbody button').exists()).toBe(false)
    expect(w.find('.attr-actions').exists()).toBe(false)
    expect(rows[0].text()).toContain('https://example.invalid/a')
    expect(w.findAll('.attr-card')).toHaveLength(2)
    w.unmount()
  })

  it('is a screen, not a scrim: a non-modal labelled dialog with no aria-modal', async () => {
    const { w } = await setup()
    expect(w.find('.attr-scrim').exists()).toBe(false)
    expect(w.find('.attr-screen').exists()).toBe(true)
    const dialog = w.get('.attr-panel')
    expect(dialog.attributes('role')).toBe('dialog')
    expect(dialog.attributes('aria-modal')).toBeUndefined()
    expect(dialog.attributes('aria-labelledby')).toBe('attr-title')
    expect(w.find('#attr-title').exists()).toBe(true)
    w.unmount()
  })

  it('foot order: after .attr-legal, before .attr-note; primary first button; Tab keycap hint', async () => {
    const { w } = await setup()
    const legal = w.get('.attr-legal').element
    const foot = w.get('.attr-foot').element
    const note = w.get('.attr-note').element
    expect(legal.compareDocumentPosition(foot) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    expect(foot.compareDocumentPosition(note) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    const buttons = footButtons(w)
    expect(buttons).toHaveLength(2)
    expect(buttons[0].classes()).toContain('attr-action-primary')
    expect(buttons[1].classes()).not.toContain('attr-action-primary')
    expect(buttons[0].text()).toBe('Mở văn bản giấy phép')
    expect(buttons[1].text()).toBe('Sao chép bản ghi công')
    expect(w.get('.attr-foot kbd').text()).toBe('Tab')
    expect(w.get('.attr-foot .attr-hint').text()).toBe(
      'Toàn bộ giấy phép kèm trong bản phát hành · Tab để duyệt bảng',
    )
    w.unmount()
  })

  it('the open button follows the aimed row licence text', async () => {
    const { w } = await setup()
    expect(footButtons(w)[0].attributes('disabled')).toBeUndefined()
    await w.findAll('tbody tr')[1].trigger('mousedown')
    expect(footButtons(w)[0].attributes('disabled')).toBeDefined()
    expect(footButtons(w)[1].attributes('disabled')).toBeUndefined()
    w.unmount()
  })

  it('aims the first row on open: attr-aimed + aria-current, never aria-selected', async () => {
    const { w } = await setup()
    const rows = w.findAll('tbody tr')
    expect(rows[0].classes()).toContain('attr-aimed')
    expect(rows[0].attributes('aria-current')).toBe('true')
    expect(rows[1].classes()).not.toContain('attr-aimed')
    expect(rows[1].attributes('aria-current')).toBeUndefined()
    expect(w.findAll('[aria-selected]')).toHaveLength(0)
    expect(rows.every((r) => r.attributes('tabindex') === '0')).toBe(true)
    w.unmount()
  })

  it('aims the first row even when the sources load after the screen opened', async () => {
    const writeText = vi.fn(async () => {})
    vi.stubGlobal('navigator', { clipboard: { writeText } })
    const state = await import('../../src/panels/dictSourcesState')
    const commands = await import('../../src/commands')
    const { default: AttributionOverlay } = await import('../../src/AttributionOverlay.vue')
    commands.installCommands({
      openAttribution: state.openAttribution,
      closeAttribution: state.closeAttribution,
      openLicenseText: () => void state.openAimedLicenseText(),
      copyCredit: () => void state.copyAimedCredit(),
      closeLicenseText: state.closeLicenseText,
    } as unknown as Parameters<typeof commands.installCommands>[0])
    state.openAttribution()
    const w = mount(AttributionOverlay, { attachTo: document.body })
    await w.vm.$nextTick()
    expect(state.attributionAimedCode.value).toBeNull()
    expect(footButtons(w).every((b) => b.attributes('disabled') !== undefined)).toBe(true)

    await state.loadDictSources('')
    await settle(w)
    expect(state.attributionAimedCode.value).toBe('src-a')
    expect(w.findAll('tbody tr')[0].classes()).toContain('attr-aimed')
    await footButtons(w)[1].trigger('click')
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledTimes(1))
    expect(writeText).toHaveBeenCalledWith('Nguồn A\nGhi công A\nCC-BY-SA-4.0\nhttps://example.invalid/a')
    w.unmount()
  })

  it('re-aims the first row when the aimed code vanishes from the list', async () => {
    const { state, w } = await setup()
    await w.findAll('tbody tr')[0].trigger('mousedown')
    expect(state.attributionAimedCode.value).toBe('src-a')
    sourcesResult = [OTHER]
    await state.loadDictSources('')
    await settle(w)
    expect(state.attributionAimedCode.value).toBe('src-b')
    expect(w.findAll('tbody tr')).toHaveLength(1)
    expect(w.get('tbody tr').classes()).toContain('attr-aimed')
    w.unmount()
  })

  it('a source file removed from the list leaves no row and no error', async () => {
    sourcesResult = [BASE]
    const { w } = await setup()
    const rows = w.findAll('tbody tr')
    expect(rows).toHaveLength(1)
    expect(rows[0].text()).toContain('Nguồn A')
    expect(w.text()).not.toContain('Nguồn B')
    expect(w.find('.attr-error').exists()).toBe(false)
    w.unmount()
  })

  it('opens the licence text of the aimed row; Escape closes the viewer to the foot button, then the screen to the opener', async () => {
    const opener = document.createElement('button')
    opener.setAttribute('data-attribution-open', '')
    document.body.appendChild(opener)
    opener.focus()
    const { state, w } = await setup()
    const btn = w.get('.attr-foot button[data-license-open]')
    await btn.trigger('click')
    await vi.waitFor(() => expect(w.find('.attr-license-text').exists()).toBe(true))
    expect(w.get('.attr-license-text').text()).toBe('LICENSE BODY src-a')

    await w.get('.attr-screen').trigger('keydown', { key: 'Escape' })
    await settle(w)
    expect(w.find('.attr-license-text').exists()).toBe(false)
    expect(w.find('.attr-screen').exists()).toBe(true)
    expect(document.activeElement).toBe(btn.element)

    await w.get('.attr-screen').trigger('keydown', { key: 'Escape' })
    await settle(w)
    expect(state.attributionIsOpen.value).toBe(false)
    expect(document.activeElement).toBe(opener)
    w.unmount()
  })

  it('close button of the licence viewer returns focus to the foot open button', async () => {
    const { w } = await setup()
    const btn = w.get('.attr-foot button[data-license-open]')
    await btn.trigger('click')
    await vi.waitFor(() => expect(w.find('.attr-license-text').exists()).toBe(true))
    const close = w.findAll('.attr-license-view .attr-action').at(-1)!
    await close.trigger('click')
    await settle(w)
    expect(w.find('.attr-license-text').exists()).toBe(false)
    expect(document.activeElement).toBe(btn.element)
    w.unmount()
  })

  it('aiming another row with licence text while the viewer is open swaps the text and keeps focus on the row', async () => {
    sourcesResult = [BASE, { ...OTHER, license_text_len: 7 }]
    const { state, w } = await setup()
    await footButtons(w)[0].trigger('click')
    await vi.waitFor(() => expect(w.find('.attr-license-text').exists()).toBe(true))
    expect(w.get('.attr-license-view').element.contains(document.activeElement)).toBe(true)

    const rows = w.findAll('tbody tr')
    ;(rows[1].element as HTMLElement).focus()
    await rows[1].trigger('focusin')
    await vi.waitFor(() => expect(state.attributionLicenseView.value?.code).toBe('src-b'))
    await settle(w)
    expect(state.attributionLicenseView.value?.text).toBe('LICENSE BODY src-b')
    expect(w.get('.attr-license-text').text()).toBe('LICENSE BODY src-b')
    expect(w.findAll('tbody tr')[1].classes()).toContain('attr-aimed')
    expect(document.activeElement).toBe(rows[1].element)
    w.unmount()
  })

  it('aiming a row without licence text closes the viewer and focus lands on the aimed row, not the disabled button', async () => {
    const { state, w } = await setup()
    await footButtons(w)[0].trigger('click')
    await vi.waitFor(() => expect(w.find('.attr-license-text').exists()).toBe(true))
    const close = w.findAll('.attr-license-view .attr-action').at(-1)!
    ;(close.element as HTMLElement).focus()
    expect(w.get('.attr-license-view').element.contains(document.activeElement)).toBe(true)
    const rows = w.findAll('tbody tr')
    await rows[1].trigger('mousedown')
    await settle(w)
    await settle(w)
    expect(w.find('.attr-license-view').exists()).toBe(false)
    expect(state.attributionLicenseView.value).toBeNull()
    expect(footButtons(w)[0].attributes('disabled')).toBeDefined()
    expect((document.activeElement as HTMLElement).matches('tr.attr-aimed')).toBe(true)
    w.unmount()
  })

  it('renders the credit and the not-GPL sentence for an author-grant row', async () => {
    sourcesResult = [
      { ...OTHER, code: 'src-g', license_kind: 'author-grant', attribution: 'Tác giả Thử Nghiệm cấp phép riêng' },
    ]
    const { w } = await setup()
    const row = w.find('tbody tr')
    expect(row.text()).toContain('Tác giả Thử Nghiệm cấp phép riêng')
    expect(row.text()).toContain('không thuộc GPL v3')
    w.unmount()
  })

  it('Tab stops cover every row, the two foot buttons and the close button, and wrap both ways', async () => {
    const { w } = await setup()
    await footButtons(w)[0].trigger('click')
    await vi.waitFor(() => expect(w.find('.attr-license-text').exists()).toBe(true))
    const stops = Array.from(
      (w.element as HTMLElement).querySelectorAll<HTMLElement>('button:not([disabled]), [tabindex]:not([tabindex="-1"])'),
    )
    const labels = stops.map((el) => el.textContent.trim())
    expect(labels.filter((l) => l === 'Sao chép bản ghi công')).toHaveLength(1)
    expect(labels.filter((l) => l === 'Mở văn bản giấy phép')).toHaveLength(1)
    expect(labels).toContain('Đóng văn bản giấy phép')
    expect(stops.filter((el) => el.tagName === 'TR')).toHaveLength(2)

    const screen = w.get('.attr-screen')
    stops.at(-1)!.focus()
    await screen.trigger('keydown', { key: 'Tab' })
    expect(document.activeElement).toBe(stops[0])
    await screen.trigger('keydown', { key: 'Tab', shiftKey: true })
    expect(document.activeElement).toBe(stops.at(-1))
    w.unmount()
  })

  it('shows the named error when the code vanished', async () => {
    licenseResult = {
      err: {
        code: 'dict.unknown_source_code',
        message_key: 'err.dict.unknown_source_code',
        params: { code: 'src-a' },
        retryable: false,
      },
    }
    const { w } = await setup()
    await footButtons(w)[0].trigger('click')
    await vi.waitFor(() => expect(w.find('.attr-license-view .attr-error').exists()).toBe(true))
    expect(w.find('.attr-license-text').exists()).toBe(false)
    w.unmount()
  })

  it('copies name, credit, licence id and url of the aimed row', async () => {
    const writeText = vi.fn(async () => {})
    vi.stubGlobal('navigator', { clipboard: { writeText } })
    const { w } = await setup()
    await footButtons(w)[1].trigger('click')
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledTimes(1))
    expect(writeText).toHaveBeenCalledWith('Nguồn A\nGhi công A\nCC-BY-SA-4.0\nhttps://example.invalid/a')
    expect(w.find('.attr-error').exists()).toBe(false)
    w.unmount()
  })

  it('shows the error beside the foot buttons when the clipboard refuses, and drops it on aiming another row', async () => {
    vi.stubGlobal('navigator', {
      clipboard: { writeText: vi.fn(async () => Promise.reject(new Error('denied'))) },
    })
    const { w } = await setup()
    await w.findAll('tbody tr')[1].trigger('mousedown')
    await footButtons(w)[1].trigger('click')
    await vi.waitFor(() => expect(w.find('.attr-foot .attr-error').exists()).toBe(true))
    expect(w.find('tbody .attr-error').exists()).toBe(false)

    await w.findAll('tbody tr')[0].trigger('mousedown')
    await settle(w)
    expect(w.find('.attr-error').exists()).toBe(false)
    w.unmount()
  })

  it('a clipboard rejection that arrives after the aim moved shows no error', async () => {
    let reject!: (e: Error) => void
    const pending = new Promise<void>((_, rj) => {
      reject = rj
    })
    const writeText = vi.fn(() => pending)
    vi.stubGlobal('navigator', { clipboard: { writeText } })
    const { w } = await setup()
    await footButtons(w)[1].trigger('click')
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledTimes(1))
    await w.findAll('tbody tr')[1].trigger('mousedown')
    await settle(w)
    reject(new Error('denied'))
    await pending.catch(() => {})
    await settle(w)
    await settle(w)
    expect(w.find('.attr-error').exists()).toBe(false)
    w.unmount()
  })

  it('still opens when no source loaded: legal cards show, both foot buttons are disabled', async () => {
    sourcesResult = []
    const { w } = await setup()
    expect(w.find('.attr-screen').exists()).toBe(true)
    expect(w.find('.attr-empty').exists()).toBe(true)
    expect(w.findAll('.attr-card')).toHaveLength(2)
    const foot = footButtons(w)
    expect(foot).toHaveLength(2)
    expect(foot.every((b) => b.attributes('disabled') !== undefined)).toBe(true)
    w.unmount()
  })
})

describe('AttributionOverlay placement (source)', () => {
  const codeOnly = (path: string): string =>
    readFileSync(resolve(process.cwd(), path), 'utf8')
      .split('\n')
      .filter((line) => !/^\s*(\/\/|\/\*|\*|<!--)/.test(line))
      .join('\n')
  const cssOf = (path: string): string => /<style[^>]*>([\s\S]*?)<\/style>/.exec(codeOnly(path))?.[1] ?? ''
  const rule = (css: string, selector: string): string => {
    const esc = selector.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    return new RegExp(`(?:^|\\n)${esc}\\s*\\{([^}]*)\\}`).exec(css)?.[1] ?? ''
  }
  const overlayCss = (): string => cssOf('src/AttributionOverlay.vue')

  it('.attr-screen is a child of div.stage placed after .modeport', () => {
    const app = codeOnly('src/App.vue')
    expect(app).toMatch(
      /<div class="stage">\s*<div class="modeport">\s*<KeepAlive>[\s\S]*?<\/KeepAlive>\s*<\/div>[\s\S]*?<AttributionOverlay \/>\s*<\/div>/,
    )
  })

  it('.attr-screen CSS is position: absolute', () => {
    expect(rule(overlayCss(), '.attr-screen')).toMatch(/position:\s*absolute/)
  })

  it('.attr-screen is never position: fixed', () => {
    expect(overlayCss()).not.toMatch(/position:\s*fixed/)
  })

  it('.attr-screen does not depend on the titlebar height', () => {
    expect(overlayCss()).not.toContain('--space-titlebar-height')
  })

  it('.attr-screen declares no z-index', () => {
    expect(overlayCss()).not.toMatch(/z-index/)
  })

  it('the aimed row rule adds no padding and no box-shadow', () => {
    const aimed = rule(overlayCss(), '.attr-table tr.attr-aimed td:first-child')
    expect(aimed).not.toMatch(/padding/)
    expect(aimed).not.toMatch(/box-shadow/)
  })

  it('the first cell keeps one fixed border-left width; the aimed rule only recolours it', () => {
    const base = rule(overlayCss(), '.attr-table td:first-child')
    const aimed = rule(overlayCss(), '.attr-table tr.attr-aimed td:first-child')
    expect(base).toMatch(/border-left:\s*2px solid transparent/)
    expect(aimed).toMatch(/border-left-color:/)
    expect(aimed).not.toMatch(/border-left(?:-width)?:/)
  })
})

const APP_STUBS = {
  KeepAlive: false,
  LibraryMode: { template: '<div />' },
  WorkspaceMode: { template: '<div />' },
  ReadingMode: { template: '<div />' },
  LookupDrawer: { template: '<div data-stub="lookup-drawer" />' },
  GlossaryQuickAdd: { template: '<div data-stub="quick-add" />' },
  ProofreaderStrip: { template: '<div data-stub="proofreader" />' },
  StatusBar: { template: '<div data-stub="status" />' },
}

async function mountApp(opts: { open: boolean }) {
  const state = await import('../../src/panels/dictSourcesState')
  const modes = await import('../../src/modes/modeState')
  const commands = await import('../../src/commands')
  commands.installCommands({
    setMode: modes.setMode,
    openAttribution: state.openAttribution,
    closeAttribution: state.closeAttribution,
    openLicenseText: () => void state.openAimedLicenseText(),
    copyCredit: () => void state.copyAimedCredit(),
    closeLicenseText: state.closeLicenseText,
  } as unknown as Parameters<typeof commands.installCommands>[0])
  const { default: App } = await import('../../src/App.vue')
  await state.loadDictSources('')
  modes.setMode('workspace')
  if (opts.open) state.openAttribution()
  const w = mount(App, { attachTo: document.body, global: { stubs: { ...APP_STUBS } } })
  await w.vm.$nextTick()
  return { state, modes, w }
}

describe('AttributionOverlay with the title bar (App.vue)', { timeout: 60_000 }, () => {
  beforeEach(() => {
    sourcesResult = [BASE, OTHER]
    licenseResult = { ok: 'LICENSE BODY' }
  })
  afterEach(() => {
    document.body.innerHTML = ''
    vi.resetModules()
  })

  const openViewer = async (state: Awaited<ReturnType<typeof mountApp>>['state'], w: Awaited<ReturnType<typeof mountApp>>['w']) => {
    await w.get('.attr-foot button[data-license-open]').trigger('click')
    await vi.waitFor(() => expect(state.attributionLicenseView.value).not.toBeNull())
  }
  const settleApp = async (w: Awaited<ReturnType<typeof mountApp>>['w']) => {
    await w.vm.$nextTick()
    await w.vm.$nextTick()
  }

  it('the title bar shows the Attribution label and three enabled mode tabs while open', async () => {
    const { w } = await mountApp({ open: true })
    expect(w.get('[data-titlebar-attribution]').text()).toBe('Giới thiệu · Nguồn dữ liệu')
    const tabs = w.findAll('.mode-tab')
    expect(tabs).toHaveLength(3)
    expect(tabs.every((tab) => tab.attributes('disabled') === undefined)).toBe(true)
    w.unmount()
  })

  it('clicking another mode tab dispatches mode.* and closes the screen, the licence view and the aim', async () => {
    const { state, modes, w } = await mountApp({ open: true })
    await openViewer(state, w)
    await w.findAll('.mode-tab')[0].trigger('click')
    await settleApp(w)
    expect(modes.currentMode.value).toBe('library')
    expect(state.attributionIsOpen.value).toBe(false)
    expect(state.attributionLicenseView.value).toBeNull()
    expect(state.attributionAimedCode.value).toBeNull()
    expect(w.find('.attr-screen').exists()).toBe(false)
    expect(w.find('[data-titlebar-attribution]').exists()).toBe(false)
    w.unmount()
  })

  it('clicking the already-active Workspace tab closes the screen, the licence view and the aim', async () => {
    const { state, modes, w } = await mountApp({ open: true })
    await openViewer(state, w)
    const active = w.findAll('.mode-tab').find((tab) => tab.classes().includes('on'))
    expect(modes.currentMode.value).toBe('workspace')
    await (active ?? w.findAll('.mode-tab')[1]).trigger('click')
    await settleApp(w)
    expect(modes.currentMode.value).toBe('workspace')
    expect(state.attributionIsOpen.value).toBe(false)
    expect(state.attributionLicenseView.value).toBeNull()
    expect(state.attributionAimedCode.value).toBeNull()
    expect(w.find('.attr-screen').exists()).toBe(false)
    w.unmount()
  })

  it('the three bottom strips sit inside .stage before .attr-screen', async () => {
    const { w } = await mountApp({ open: true })
    const stage = w.get('.stage').element
    const screen = w.get('.attr-screen').element
    for (const name of ['quick-add', 'proofreader', 'status']) {
      const strip = w.get(`[data-stub="${name}"]`).element
      expect(stage.contains(strip)).toBe(true)
      expect(strip.compareDocumentPosition(screen) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
    }
    w.unmount()
  })

  it('LookupDrawer stays outside .stage', async () => {
    const { w } = await mountApp({ open: true })
    expect(w.get('.stage').element.contains(w.get('[data-stub="lookup-drawer"]').element)).toBe(false)
    w.unmount()
  })

  it('the layout switch buttons are disabled while open and enabled after', async () => {
    const { state, w } = await mountApp({ open: false })
    const switchDisabled = () => w.findAll('.layout-switch-btn').map((b) => b.attributes('disabled') !== undefined)
    expect(w.findAll('.layout-switch-btn')).toHaveLength(2)
    expect(switchDisabled()).toEqual([false, false])
    state.openAttribution()
    await w.vm.$nextTick()
    expect(switchDisabled()).toEqual([true, true])
    state.closeAttribution()
    await w.vm.$nextTick()
    expect(switchDisabled()).toEqual([false, false])
    w.unmount()
  })

  it('the Công cụ trigger is disabled while open and enabled after', async () => {
    const { state, w } = await mountApp({ open: false })
    const toolsDisabled = () => w.get('[data-tools-trigger]').attributes('disabled') !== undefined
    expect(toolsDisabled()).toBe(false)
    state.openAttribution()
    await w.vm.$nextTick()
    expect(toolsDisabled()).toBe(true)
    state.closeAttribution()
    await w.vm.$nextTick()
    expect(toolsDisabled()).toBe(false)
    w.unmount()
  })
})

describe('LookupDrawer while Attribution is open', () => {
  afterEach(() => {
    document.body.innerHTML = ''
    vi.resetModules()
  })

  it('hides the drawer while the screen is open and restores it, with focus on the opener, when it closes', async () => {
    sourcesResult = [BASE, OTHER]
    const drawerState = await import('../../src/layout/lookupDrawerState')
    const state = await import('../../src/panels/dictSourcesState')
    const { default: LookupDrawer } = await import('../../src/layout/LookupDrawer.vue')
    const { default: AttributionOverlay } = await import('../../src/AttributionOverlay.vue')
    const Host = {
      components: { LookupDrawer, AttributionOverlay },
      template: '<div><LookupDrawer /><AttributionOverlay /></div>',
    }
    const w = mount(Host, {
      attachTo: document.body,
      global: {
        stubs: { LookupPanel: { template: '<div><button data-attribution-open>open</button></div>' } },
      },
    })
    drawerState.openLookupDrawer()
    await w.vm.$nextTick()
    const opener = w.get('[data-attribution-open]').element as HTMLElement
    const scrim = () => w.get('.ld-scrim').element as HTMLElement
    expect(scrim().style.display).not.toBe('none')

    opener.focus()
    state.openAttribution()
    await w.vm.$nextTick()
    await w.vm.$nextTick()
    expect(scrim().style.display).toBe('none')

    state.closeAttribution()
    await w.vm.$nextTick()
    await w.vm.$nextTick()
    await w.vm.$nextTick()
    expect(scrim().style.display).not.toBe('none')
    expect(document.activeElement).toBe(opener)
    w.unmount()
  })
})

describe('LookupPanel opener', () => {
  afterEach(() => {
    vi.resetModules()
  })

  it('renders the attribution opener with zero sources', async () => {
    sourcesResult = []
    const state = await import('../../src/panels/dictSourcesState')
    const { default: LookupPanel } = await import('../../src/panels/LookupPanel.vue')
    await state.loadDictSources('')
    const w = mount(LookupPanel, { props: { params: { params: {} } } })
    expect(w.find('[data-attribution-open]').exists()).toBe(true)
    expect(w.find('.source-chip').exists()).toBe(false)
    w.unmount()
  })
})
