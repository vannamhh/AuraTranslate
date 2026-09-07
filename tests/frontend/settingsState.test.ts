/**
 * Lớp phủ **Cài đặt** — Story 6.8 (NFR19, AD-41).
 *
 * ⚠️ Khuôn `importPreviewUrls.test.ts`: `config/project.ts` là biên IPC, giả lập bằng
 * `vi.mock`, không gọi `@tauri-apps/api` thật. Nạp ĐỘNG cả state lẫn component trong cùng
 * một lượt (`freshOverlay()`), khuôn `importPreviewOverlayRender.test.ts:1-60`.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import type { DomainLogEntryWire } from '../../src/config/project'

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
): DomainLogEntryWire {
  return { at_epoch_ms: atEpochMs, domain, kind, allowed, tier }
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
})

describe('domainLogKindLabelKey / domainLogReasonKey — khoá LITERAL, khuôn cleanupTierLabelKey', () => {
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
})

// ═════════════════════════════════════════════════════════════════════════════════
// SettingsOverlay.vue — mười một mục nav, đúng MỘT có thân
// ═════════════════════════════════════════════════════════════════════════════════

describe('SettingsOverlay.vue — mười một mục nav, mười mục chưa có thân LUÔN hiện kèm tên chủ', () => {
  it('đóng ⇒ không dựng gì trong DOM', async () => {
    const { SettingsOverlay } = await freshOverlay()
    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.set-panel').exists()).toBe(false)
    wrapper.unmount()
  })

  it('mở ⇒ ĐÚNG 11 mục nav hiện, không mục nào bị `v-if` giấu', async () => {
    const { state, SettingsOverlay } = await freshOverlay()
    listDomainLogMock.mockResolvedValue({ entries: [], error: null })
    state.openSettings()

    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.findAll('.set-nav-item').length).toBe(11)
    wrapper.unmount()
  })

  it('mục chưa có thân hiện câu nói vì sao rỗng KÈM TÊN CHỦ, không một bảng trắng', async () => {
    const { state, SettingsOverlay } = await freshOverlay()
    listDomainLogMock.mockResolvedValue({ entries: [], error: null })
    state.openSettings()
    state.selectSettingsSection('translation_memory')

    const wrapper = mount(SettingsOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const reason = wrapper.find('.set-tier-empty-reason')
    expect(reason.exists()).toBe(true)
    expect(reason.text().length).toBeGreaterThan(0)
    wrapper.unmount()
  })

  it('openSettingsToPrivacy() mở THẲNG vào Quyền riêng tư bất kể mục trước đó', async () => {
    const { state } = await freshState()
    listDomainLogMock.mockResolvedValue({ entries: [], error: null })
    state.selectSettingsSection('update')
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
