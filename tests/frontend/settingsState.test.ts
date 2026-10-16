/**
 * Lớp phủ **Cài đặt** — Story 6.8 (NFR19, AD-41).
 *
 * ⚠️ Khuôn `importPreviewUrls.test.ts`: `config/project.ts` là biên IPC, giả lập bằng
 * `vi.mock`, không gọi `@tauri-apps/api` thật. Nạp ĐỘNG cả state lẫn component trong cùng
 * một lượt (`freshOverlay()`), khuôn `importPreviewOverlayRender.test.ts:1-60`.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { DomainLogEntryWire, DomainLogOutcomeWire } from '../../src/config/project'

const listDomainLogMock = vi.fn()

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    listDomainLog: () => listDomainLogMock(),
  }
})

async function freshState() {
  vi.resetModules()
  listDomainLogMock.mockReset()
  const state = await import('../../src/settingsState')
  return { state }
}

async function freshOverlay() {
  const { state } = await freshState()
  const SettingsOverlay = (await import('../../src/SettingsOverlay.vue')).default
  return { state, SettingsOverlay }
}

function entry(
  domain: string,
  kind: DomainLogEntryWire['kind'],
  tier: DomainLogEntryWire['tier'],
  allowed: boolean,
  atEpochMs = 1,
  // 🔵 THÊM 2026-09-09 (Story 6.11, mục A vòng rà đối kháng 3 lớp) — mặc định `null`
  // (đúng giá trị Rust gửi khi `allowed === false`), giữ MỌI ca gọi cũ không cần sửa.
  outcome: DomainLogOutcomeWire | null = null,
): DomainLogEntryWire {
  return { at_epoch_ms: atEpochMs, domain, kind, allowed, tier, outcome }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

// ═════════════════════════════════════════════════════════════════════════════════
// groupDomainLogEntries — hàm thuần, gộp theo (domain, kind, tier)
// ═════════════════════════════════════════════════════════════════════════════════

describe('groupDomainLogEntries — gộp bản ghi THÔ theo (domain, kind, tier)', () => {
  it('nhiều bản ghi CÙNG (domain, kind, tier) gộp thành MỘT hàng, đếm đúng số lần', async () => {
    const { state } = await freshState()
    const rows = state.groupDomainLogEntries([
      entry('a.example', 'page', 'tier1', true, 3),
      entry('a.example', 'page', 'tier1', true, 1),
      entry('a.example', 'page', 'tier1', true, 2),
    ])

    expect(rows.length).toBe(1)
    expect(rows[0]?.count).toBe(3)
    // Thời điểm của hàng gộp là lượt gọi ĐẦU TIÊN (nhỏ nhất), không lượt cuối.
    expect(rows[0]?.firstAtEpochMs).toBe(1)
  })

  it('CÙNG domain nhưng KHÁC kind/tier ra HAI hàng riêng — không gộp lẫn hai lý do khác nhau', async () => {
    const { state } = await freshState()
    const rows = state.groupDomainLogEntries([
      entry('a.example', 'page', 'tier1', true),
      entry('a.example', 'image', 'tier2', true),
    ])

    expect(rows.length).toBe(2)
  })

  it('mảng rỗng ⇒ 0 hàng, không panic', async () => {
    const { state } = await freshState()
    expect(state.groupDomainLogEntries([])).toEqual([])
  })

  it('sắp theo lượt gọi ĐẦU TIÊN của mỗi nhóm, tăng dần', async () => {
    const { state } = await freshState()
    const rows = state.groupDomainLogEntries([
      entry('later.example', 'page', 'tier1', true, 500),
      entry('earlier.example', 'page', 'tier1', true, 100),
    ])
    expect(rows.map((r) => r.domain)).toEqual(['earlier.example', 'later.example'])
  })

  // 🔵 THÊM 2026-09-09 (Story 6.11, mục A vòng rà đối kháng 3 lớp) — `outcome` nay MỘT PHẦN
  // của khoá gộp: cùng (domain, kind, tier) nhưng khác outcome (một lượt tải xong, một lượt
  // bị `too_large` cắt) phải ra HAI hàng, mỗi hàng mang đúng MỘT outcome — không một hàng
  // "trộn" khai một kết quả không khớp một phần bản ghi của chính nó.
  it('CÙNG (domain, kind, tier) nhưng KHÁC outcome ra HAI hàng riêng', async () => {
    const { state } = await freshState()
    const rows = state.groupDomainLogEntries([
      entry('cdn.example', 'image', 'tier2', true, 1, 'fetched'),
      entry('cdn.example', 'image', 'tier2', true, 2, 'too_large'),
    ])

    expect(rows.length).toBe(2)
    expect(new Set(rows.map((r) => r.outcome))).toEqual(new Set(['fetched', 'too_large']))
  })

  it('CÙNG (domain, kind, tier, outcome) gộp thành MỘT hàng như trước', async () => {
    const { state } = await freshState()
    const rows = state.groupDomainLogEntries([
      entry('cdn.example', 'image', 'tier2', true, 1, 'fetched'),
      entry('cdn.example', 'image', 'tier2', true, 2, 'fetched'),
    ])

    expect(rows.length).toBe(1)
    expect(rows[0]?.count).toBe(2)
    expect(rows[0]?.outcome).toBe('fetched')
  })
})

describe('domainLogKindLabelKey / domainLogReasonKey / domainLogOutcomeLabelKey — khoá LITERAL, khuôn cleanupTierLabelKey', () => {
  it('hai kind ra hai khoá khác nhau, đều khác rỗng', async () => {
    const { state } = await freshState()
    expect(state.domainLogKindLabelKey('page')).not.toBe(state.domainLogKindLabelKey('image'))
    expect(state.domainLogKindLabelKey('page')).toBeTruthy()
    expect(state.domainLogKindLabelKey('image')).toBeTruthy()
  })

  it('ba tier ra BA khoá phân biệt', async () => {
    const { state } = await freshState()
    const keys = new Set([
      state.domainLogReasonKey('tier1'),
      state.domainLogReasonKey('tier2'),
      state.domainLogReasonKey('denied'),
    ])
    expect(keys.size).toBe(3)
  })

  it('tám outcome cộng `null` ra CHÍN khoá phân biệt, đều khác rỗng', async () => {
    const { state } = await freshState()
    const outcomes: Array<DomainLogOutcomeWire | null> = [
      null,
      'fetched',
      'redirected',
      'http_status',
      'timeout',
      'connect_failed',
      'too_large',
      'mime_rejected',
      'other',
    ]
    const keys = outcomes.map((o) => state.domainLogOutcomeLabelKey(o))
    expect(new Set(keys).size).toBe(9)
    for (const k of keys) expect(k).toBeTruthy()
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// SettingsOverlay.vue — mười một mục nav, đúng MỘT có thân
// ═════════════════════════════════════════════════════════════════════════════════

describe('SettingsOverlay.vue — nav lists only sections that have a body', () => {
  it('đóng ⇒ không dựng gì trong DOM', async () => {
    const { SettingsOverlay } = await freshOverlay()
    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.set-panel').exists()).toBe(false)
    wrapper.unmount()
  })

  it('openSettingsToPrivacy() mở THẲNG vào Quyền riêng tư bất kể mục trước đó', async () => {
    const { state } = await freshState()
    listDomainLogMock.mockResolvedValue({ entries: [], error: null })
    state.selectSettingsSection('shortcuts')
    state.openSettingsToPrivacy()
    expect(state.settingsActiveSection.value).toBe('privacy')
    await flushPromises()
  })
})

describe('SettingsOverlay.vue — Quyền riêng tư: lỗi đứng TRƯỚC rỗng, rỗng nói ra vì sao (I/O Matrix spec 6.8)', () => {
  it('nhật ký RỖNG (chưa gọi mạng lần nào) ⇒ câu nói ra, KHÔNG một bảng trắng im lặng', async () => {
    const { state, SettingsOverlay } = await freshOverlay()
    listDomainLogMock.mockResolvedValue({ entries: [], error: null })
    state.openSettingsToPrivacy()
    await flushPromises()

    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.set-table').exists()).toBe(false)
    expect(wrapper.find('.set-empty').exists()).toBe(true)
    wrapper.unmount()
  })

  it('lỗi đọc THẬT ⇒ hiện câu lỗi, KHÔNG câu "rỗng" (hai nguyên nhân khác nhau)', async () => {
    const { state, SettingsOverlay } = await freshOverlay()
    const ipcError = { code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false }
    listDomainLogMock.mockResolvedValue({ entries: null, error: ipcError })
    state.openSettingsToPrivacy()
    await flushPromises()

    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(state.settingsDomainLogError.value).not.toBeNull()
    expect(wrapper.find('.set-table').exists()).toBe(false)
    wrapper.unmount()
  })

  it('có bản ghi ⇒ bảng hiện GỘP theo domain, nhãn tầng là CHỮ (Tài liệu/Ảnh)', async () => {
    const { state, SettingsOverlay } = await freshOverlay()
    listDomainLogMock.mockResolvedValue({
      entries: [
        entry('truyen-example.com', 'page', 'tier1', true, 1),
        entry('truyen-example.com', 'page', 'tier1', true, 2),
        entry('img-cdn.example.net', 'image', 'tier2', true, 3),
      ],
      error: null,
    })
    state.openSettingsToPrivacy()
    await flushPromises()

    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('.set-table tbody tr')
    // Gộp: 3 bản ghi thô ⇒ 2 hàng (truyen-example.com gộp làm một, img-cdn riêng).
    expect(rows.length).toBe(2)
    wrapper.unmount()
  })
})

describe('mục nav mới và phạm vi', () => {
  it('Phạm vi: chỉ AI và mô hình có tầng Tác phẩm; Phím tắt, Bố cục, Chế độ đọc, Dữ liệu, Cập nhật khoá ở Toàn cục', async () => {
    const { state } = await freshState()
    const withWork = state.SETTINGS_SECTIONS.filter((s) => state.settingsSectionHasWorkTier(s))
    const locked = state.SETTINGS_SECTIONS.filter((s) => state.settingsSectionLocksScope(s))
    expect(withWork).toEqual(['ai_and_model'])
    expect(locked).toEqual(['shortcuts', 'layout', 'reading', 'data', 'update'])
  })

  it('openSettingsToAiConfig mở thẳng mục AI và mô hình bất kể mục trước đó', async () => {
    const { state } = await freshState()
    listDomainLogMock.mockResolvedValue({ entries: [], error: null })
    state.selectSettingsSection('privacy')
    state.openSettingsToAiConfig()
    expect(state.settingsOverlayIsOpen.value).toBe(true)
    expect(state.settingsActiveSection.value).toBe('ai_and_model')
  })
})

describe('settingsOwnsChord — cổng phím khi Cài đặt mở', () => {
  async function withCommands(deps: Record<string, unknown> = {}) {
    const { state } = await freshState()
    listDomainLogMock.mockResolvedValue({ entries: [], error: null })
    const commands = await import('../../src/commands')
    commands.installCommands({
      isMac: true,
      setMode: () => {},
      openSettingsToAiConfig: state.openSettingsToAiConfig,
      ...deps,
    })
    return { state, commands }
  }
  const ev = (code: string, init: Record<string, unknown> = {}) =>
    ({ code, metaKey: true, ...init }) as Parameters<
      (typeof import('../../src/settingsState'))['settingsOwnsChord']
    >[0]

  beforeEach(() => {
    Object.defineProperty(navigator, 'platform', { value: 'MacIntel', configurable: true })
  })

  it('⌘T và ⌘⇧R là của Cài đặt; ⌘M, ⌘, và phím không phải tổ hợp thì không', async () => {
    const { state } = await withCommands()
    expect(state.settingsOwnsChord(ev('KeyT'))).toBe(true)
    expect(state.settingsOwnsChord(ev('KeyR', { shiftKey: true }))).toBe(true)
    expect(state.settingsOwnsChord(ev('KeyM'))).toBe(false)
    expect(state.settingsOwnsChord(ev('Comma'))).toBe(false)
    expect(state.settingsOwnsChord(ev('ShiftLeft'))).toBe(false)
  })

  it('trong ô tìm phím tắt, ⌘T là tra ngược chứ không chạy lệnh', async () => {
    const { state } = await withCommands()
    const input = document.createElement('input')
    input.setAttribute('data-shortcut-search', '')
    document.body.appendChild(input)
    expect(state.settingsOwnsChord({ ...ev('KeyT'), target: input } as never)).toBe(false)
  })

  it('đi qua attachKeyboard như main.ts: Cài đặt mở thì ⌘T chạy, ⌘M bị chặn', async () => {
    const mergeSegments = vi.fn()
    const { state, commands } = await withCommands({ mergeSegments })
    state.openSettings()
    state.selectSettingsSection('privacy')
    const off = commands.attachKeyboard(document, {
      isBlocked: (event) => state.settingsOverlayIsOpen.value && !state.settingsOwnsChord(event),
    })
    document.body.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyM', metaKey: true, bubbles: true }))
    expect(mergeSegments).not.toHaveBeenCalled()
    expect(state.settingsActiveSection.value).toBe('privacy')
    document.body.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyT', metaKey: true, bubbles: true }))
    expect(state.settingsActiveSection.value).toBe('ai_and_model')

    state.closeSettings()
    document.body.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyM', metaKey: true, bubbles: true }))
    expect(mergeSegments).toHaveBeenCalledTimes(1)
    off()
  })
})

describe('settings.ai_config.clear_all_overrides ngoài mục AI', () => {
  it('không làm gì khi Cài đặt đang mở ở mục khác', async () => {
    const { state } = await freshState()
    const ai = await import('../../src/aiConfigState')
    const spy = vi.spyOn(ai, 'clearAllAiConfigOverrides')
    state.openSettingsToSection('shortcuts')
    state.clearAiOverridesFromSettings()
    expect(spy).not.toHaveBeenCalled()
  })
})
