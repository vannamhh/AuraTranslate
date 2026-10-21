/** Settings → Cập nhật (FR111): version line, switch, manual check and its three outcomes. */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { readFileSync, readdirSync } from 'node:fs'
import { join, relative, sep } from 'node:path'

const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mockInvoke(...args) }))

const wire = (extra: Record<string, unknown> = {}) => ({
  theme: 'light',
  mode: 'library',
  shortcuts: {},
  layout_presets: {},
  workspace_layout: '',
  dict_sources_disabled: '',
  update_check_enabled: true,
  app_version: '0.1.0',
  ...extra,
})

async function mounted(bootstrap: Record<string, unknown> = {}) {
  vi.resetModules()
  mockInvoke.mockReset()
  Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  mockInvoke.mockResolvedValueOnce(wire(bootstrap))
  const boot = await import('../../src/config/bootstrap')
  await boot.loadBootstrapConfig()
  const { default: Section } = await import('../../src/SettingsUpdateSection.vue')
  const { t } = await import('../../src/i18n')
  const wrapper = mount(Section)
  return { wrapper, t, state: await import('../../src/updateCheckState') }
}

function callsTo(cmd: string): unknown[][] {
  return mockInvoke.mock.calls.filter((c) => c[0] === cmd)
}

beforeEach(() => {
  vi.spyOn(console, 'info').mockImplementation(() => undefined)
  vi.spyOn(console, 'error').mockImplementation(() => undefined)
})

describe('SettingsUpdateSection', () => {
  it('shows the running version and sends nothing until the button is pressed', async () => {
    const { wrapper, t } = await mounted()
    expect(wrapper.find('[data-update-version]').text()).toBe(t('settings.update.version', { version: '0.1.0' }))
    expect(callsTo('update_check')).toHaveLength(0)
    expect(wrapper.find('[data-update-outcome]').exists()).toBe(false)
  })

  it('says so when the running version is unknown instead of rendering an empty value', async () => {
    const { wrapper, t } = await mounted({ app_version: undefined })
    expect(wrapper.find('[data-update-version]').text()).toBe(t('settings.update.version_unknown'))
  })

  it('up to date', async () => {
    const { wrapper, t } = await mounted()
    mockInvoke.mockResolvedValueOnce({ kind: 'up_to_date', current: '0.1.0' })
    await wrapper.find('form.su-row').trigger('submit')
    await flushPromises()
    expect(callsTo('update_check')).toHaveLength(1)
    expect(wrapper.find('[data-update-outcome]').text()).toBe(t('settings.update.up_to_date', { version: '0.1.0' }))
    expect(wrapper.find('[data-update-open]').exists()).toBe(false)
  })

  it('no app release is its own state, never "up to date"', async () => {
    const { wrapper, t } = await mounted()
    mockInvoke.mockResolvedValueOnce({ kind: 'no_app_release' })
    await wrapper.find('form.su-row').trigger('submit')
    await flushPromises()
    const text = wrapper.find('[data-update-outcome]').text()
    expect(text).toBe(t('settings.update.no_app_release'))
    expect(text).not.toContain(t('settings.update.up_to_date', { version: '0.1.0' }))
  })

  it('update available shows the checksum reminder and opens the page with the bare version', async () => {
    const { wrapper, t } = await mounted()
    mockInvoke.mockResolvedValueOnce({ kind: 'update_available', latest: '0.2.0' })
    await wrapper.find('form.su-row').trigger('submit')
    await flushPromises()
    expect(wrapper.find('[data-update-outcome]').text()).toContain(t('settings.update.available', { version: '0.2.0' }))
    expect(wrapper.find('[data-update-checksum]').text()).toBe(t('settings.update.checksum_hint'))
    mockInvoke.mockResolvedValueOnce(undefined)
    await wrapper.find('[data-update-open]').trigger('submit')
    await flushPromises()
    expect(callsTo('update_open_release_page')).toEqual([['update_open_release_page', { version: '0.2.0' }]])
  })

  it('a failed check shows the error and keeps the previous outcome', async () => {
    const { wrapper } = await mounted()
    mockInvoke.mockResolvedValueOnce({ kind: 'up_to_date', current: '0.1.0' })
    await wrapper.find('form.su-row').trigger('submit')
    await flushPromises()
    mockInvoke.mockRejectedValueOnce({
      code: 'update.rate_limited',
      message_key: 'err.update.rate_limited',
      params: {},
      retryable: true,
    })
    await wrapper.find('form.su-row').trigger('submit')
    await flushPromises()
    expect(wrapper.find('[data-update-error]').exists()).toBe(true)
    expect(wrapper.find('[data-update-outcome]').exists()).toBe(true)
  })

  it('switch off disables the check button and writes "off" to the global key', async () => {
    const { wrapper } = await mounted()
    mockInvoke.mockResolvedValue(undefined)
    expect(wrapper.find('[data-update-check]').attributes('disabled')).toBeUndefined()
    await wrapper.find('[data-update-toggle]').trigger('submit')
    await flushPromises()
    expect(wrapper.find('[data-update-check]').attributes('disabled')).toBeDefined()
    const [, args] = callsTo('put_config').at(-1) as [string, Record<string, unknown>]
    expect(args).toMatchObject({ kind: 'app_config', key: 'update_check_enabled', value: 'off' })
    await wrapper.find('form.su-row').trigger('submit')
    await flushPromises()
    expect(callsTo('update_check')).toHaveLength(0)
  })

  it('starts disabled when the bootstrap says the switch is off', async () => {
    const { wrapper } = await mounted({ update_check_enabled: false })
    expect(wrapper.find('[data-update-check]').attributes('disabled')).toBeDefined()
  })
})

function sourceFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const full = join(dir, e.name)
    if (e.isDirectory()) return sourceFiles(full)
    return /\.(ts|vue)$/.test(e.name) ? [full] : []
  })
}

function codeLines(file: string): string {
  return readFileSync(file, 'utf8')
    .split('\n')
    .filter((l) => !/^\s*(\/\/|\*|\/\*|<!--)/.test(l))
    .join('\n')
}

describe('no startup path reaches the update host', () => {
  const root = join(__dirname, '../../src')
  const files = sourceFiles(root).map((f) => ({ rel: relative(root, f).split(sep).join('/'), code: codeLines(f) }))
  const where = (re: RegExp) => files.filter((f) => re.test(f.code)).map((f) => f.rel).sort()

  it('update_check is named only by the adapter, updateCheck( is called only by the state, runUpdateCheck only by the section', () => {
    expect(where(/update_check\b(?!_)|CMD_CHECK/)).toEqual(['config/update.ts'])
    expect(where(/\bupdateCheck\(/)).toEqual(['config/update.ts', 'updateCheckState.ts'])
    expect(where(/\brunUpdateCheck\b/)).toEqual(['SettingsUpdateSection.vue', 'updateCheckState.ts'])
  })
})
