import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
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
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'list_dict_sources') return { sources: sourcesResult, skipped: [] }
    if (cmd === 'read_dict_license_text') {
      if ('err' in licenseResult) throw licenseResult.err
      return licenseResult.ok
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

  it('shows the conversion notice only on the CC-BY-SA row, and the open button only with licence text', async () => {
    const { w } = await setup()
    const rows = w.findAll('tbody tr')
    expect(rows[0].find('.attr-conversion').exists()).toBe(true)
    expect(rows[1].find('.attr-conversion').exists()).toBe(false)
    expect(rows[0].find('[data-license-open]').exists()).toBe(true)
    expect(rows[1].find('[data-license-open]').exists()).toBe(false)
    expect(rows[0].text()).toContain('https://example.invalid/a')
    expect(w.findAll('.attr-card')).toHaveLength(2)
    w.unmount()
  })

  it('opens the licence text of the clicked row, and Escape closes it back to the row button', async () => {
    const { w } = await setup()
    const btn = w.find('[data-license-open="src-a"]')
    await w.findAll('tbody tr')[0].trigger('mousedown')
    await btn.trigger('click')
    await vi.waitFor(() => expect(w.find('.attr-license-text').exists()).toBe(true))
    expect(w.find('.attr-license-text').text()).toBe('LICENSE BODY')

    await w.find('.attr-scrim').trigger('keydown', { key: 'Escape' })
    await w.vm.$nextTick()
    await w.vm.$nextTick()
    expect(w.find('.attr-license-text').exists()).toBe(false)
    expect(w.find('.attr-scrim').exists()).toBe(true)
    expect(document.activeElement).toBe(btn.element)
    w.unmount()
  })

  it('close button of the licence viewer returns focus to the row open button', async () => {
    const { w } = await setup()
    const btn = w.find('[data-license-open="src-a"]')
    await w.findAll('tbody tr')[0].trigger('mousedown')
    await btn.trigger('click')
    await vi.waitFor(() => expect(w.find('.attr-license-text').exists()).toBe(true))
    const close = w.findAll('.attr-license-view .attr-action').at(-1)!
    await close.trigger('click')
    await w.vm.$nextTick()
    await w.vm.$nextTick()
    expect(w.find('.attr-license-text').exists()).toBe(false)
    expect(document.activeElement).toBe(btn.element)
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

  it('Tab stops cover every row button and the close button, and wrap both ways', async () => {
    const { w } = await setup()
    await w.findAll('tbody tr')[0].trigger('mousedown')
    await w.find('[data-license-open="src-a"]').trigger('click')
    await vi.waitFor(() => expect(w.find('.attr-license-text').exists()).toBe(true))
    const stops = Array.from(
      (w.element as HTMLElement).querySelectorAll<HTMLElement>('button:not([disabled]), [tabindex]:not([tabindex="-1"])'),
    )
    const labels = stops.map((el) => el.textContent.trim())
    expect(labels.filter((l) => l === 'Sao chép bản ghi công')).toHaveLength(2)
    expect(labels.filter((l) => l === 'Mở văn bản giấy phép')).toHaveLength(1)
    expect(labels).toContain('Đóng văn bản giấy phép')

    const scrim = w.find('.attr-scrim')
    stops.at(-1)!.focus()
    await scrim.trigger('keydown', { key: 'Tab' })
    expect(document.activeElement).toBe(stops[0])
    await scrim.trigger('keydown', { key: 'Tab', shiftKey: true })
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
    await w.findAll('tbody tr')[0].trigger('mousedown')
    await w.find('[data-license-open="src-a"]').trigger('click')
    await vi.waitFor(() => expect(w.find('.attr-license-view .attr-error').exists()).toBe(true))
    expect(w.find('.attr-license-text').exists()).toBe(false)
    w.unmount()
  })

  it('copies name, credit, licence id and url of the aimed row', async () => {
    const writeText = vi.fn(async () => {})
    vi.stubGlobal('navigator', { clipboard: { writeText } })
    const { w } = await setup()
    await w.findAll('tbody tr')[0].trigger('mousedown')
    const copy = w.findAll('tbody tr')[0].findAll('.attr-action').at(-1)!
    await copy.trigger('click')
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledTimes(1))
    expect(writeText).toHaveBeenCalledWith(
      'Nguồn A\nGhi công A\nCC-BY-SA-4.0\nhttps://example.invalid/a',
    )
    expect(w.find('.attr-error').exists()).toBe(false)
    w.unmount()
  })

  it('shows an in-place error when the clipboard refuses', async () => {
    vi.stubGlobal('navigator', {
      clipboard: { writeText: vi.fn(async () => Promise.reject(new Error('denied'))) },
    })
    const { w } = await setup()
    await w.findAll('tbody tr')[1].trigger('mousedown')
    await w.findAll('tbody tr')[1].find('.attr-action').trigger('click')
    await vi.waitFor(() => expect(w.findAll('tbody tr')[1].find('.attr-error').exists()).toBe(true))
    expect(w.findAll('tbody tr')[0].find('.attr-error').exists()).toBe(false)
    w.unmount()
  })

  it('still opens the overlay when no source loaded', async () => {
    sourcesResult = []
    const { w } = await setup()
    expect(w.find('.attr-scrim').exists()).toBe(true)
    expect(w.find('.attr-empty').exists()).toBe(true)
    expect(w.findAll('.attr-card')).toHaveLength(2)
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
