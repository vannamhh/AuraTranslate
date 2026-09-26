/**
 * `AttributionOverlay.vue` distinguishes three states: `load_failed`, "no layer attached"
 * (`empty`), and "part of the dictionary unreadable" (`some_unreadable`) — the last must not
 * collapse into `empty` when `dictSources` is also empty.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { SourceAttribution } from '../../src/config/dict'

const FAKE_SOURCE: SourceAttribution = {
  code: 'src-doc-duoc',
  display_name: 'Nguồn Đọc Được',
  license_kind: 'open',
  license_id: null,
  license_text_len: 0,
  attribution: 'Thử nghiệm',
  source_version: '1',
  source_url: 'https://example.invalid',
  lang: 'zh',
  layer: 'layer-doc-duoc',
  is_base: true,
}

let listDictSourcesResult: { sources: SourceAttribution[]; skipped: string[] } = { sources: [], skipped: [] }

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'list_dict_sources') return listDictSourcesResult
    throw new Error(`attributionOverlayUnreadable.test.ts: invoke giả không biết lệnh "${cmd}"`)
  }),
}))

describe('AttributionOverlay — trạng thái thứ ba "một phần không đọc được"', () => {
  afterEach(() => {
    vi.resetModules()
  })

  beforeEach(() => {
    listDictSourcesResult = { sources: [], skipped: [] }
  })

  it('mọi lớp bị bỏ qua (dictSources RỖNG, skipped khác rỗng) ⇒ KHÔNG hiện câu "chưa gắn lớp từ điển nào"', async () => {
    listDictSourcesResult = { sources: [], skipped: ['open_failed', 'open_failed'] }
    const { loadDictSources, openAttribution } = await import('../../src/panels/dictSourcesState')
    const { default: AttributionOverlay } = await import('../../src/AttributionOverlay.vue')

    await loadDictSources('')
    openAttribution()
    const w = mount(AttributionOverlay)
    await w.vm.$nextTick()

    expect(w.find('.attr-empty').exists()).toBe(false)
    expect(w.find('.attr-partial').exists()).toBe(true)
    w.unmount()
  })

  it('một phần đọc được, một phần không (cả hai khác rỗng) ⇒ banner VÀ bảng cùng hiện, không loại trừ nhau', async () => {
    listDictSourcesResult = { sources: [FAKE_SOURCE], skipped: ['sources_unreadable'] }
    const { loadDictSources, openAttribution } = await import('../../src/panels/dictSourcesState')
    const { default: AttributionOverlay } = await import('../../src/AttributionOverlay.vue')

    await loadDictSources('')
    openAttribution()
    const w = mount(AttributionOverlay)
    await w.vm.$nextTick()

    expect(w.find('.attr-partial').exists()).toBe(true)
    expect(w.find('.attr-empty').exists()).toBe(false)
    expect(w.find('.attr-table').exists()).toBe(true)
    expect(w.findAll('tbody tr')).toHaveLength(1)
    w.unmount()
  })

  it('đối chứng: không lớp nào bị bỏ qua ⇒ hành vi CŨ đứng nguyên (empty khi rỗng, không banner)', async () => {
    listDictSourcesResult = { sources: [], skipped: [] }
    const { loadDictSources, openAttribution } = await import('../../src/panels/dictSourcesState')
    const { default: AttributionOverlay } = await import('../../src/AttributionOverlay.vue')

    await loadDictSources('')
    openAttribution()
    const w = mount(AttributionOverlay)
    await w.vm.$nextTick()

    expect(w.find('.attr-empty').exists()).toBe(true)
    expect(w.find('.attr-partial').exists()).toBe(false)
    w.unmount()
  })
})
