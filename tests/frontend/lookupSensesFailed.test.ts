/**
 * A layer named in `LookupResponse.senses_failed` must show a "senses failed to load" state,
 * not silently render an empty sense list indistinguishable from "this entry has no senses".
 */
import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import LookupRecord from '../../src/panels/LookupRecord.vue'
import type { GroupedLookup, LookupResponse, SourceGroup } from '../../src/config/dict'

const GROUP: SourceGroup = {
  layer: 'layer-hong',
  source: { code: 'src-hong', display_name: 'Nguồn Hỏng' },
  entries: [{ entry_id: 1, source_code: 'src-hong', lang: 'zh', headword: '測試', headword_simp: null }],
  total_entries: null,
}

const GROUPED: GroupedLookup = {
  route: 'zh',
  branch: 'exact_btree',
  groups: [GROUP],
  skipped: [],
  truncated_layers: [],
  hidden_sources: [],
  layers_loaded: true,
}

const RESPONSE_WITH_FAILURE: LookupResponse = {
  grouped: GROUPED,
  // `layer-hong` has no key here (hydrate failure), not an empty array, matching the shape
  // `commands::dict::lookup` (Rust) produces.
  senses_by_layer: {},
  query_truncated: false,
  senses_failed: ['layer-hong'],
}

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'lookup_dictionary') return RESPONSE_WITH_FAILURE
    throw new Error(`lookupSensesFailed.test.ts: invoke giả không biết lệnh "${cmd}"`)
  }),
}))

describe('lookupPanelState — layerSensesFailed đọc đúng senses_failed', () => {
  afterEach(() => {
    vi.resetModules()
  })

  it('sau một lượt tra mà pha hai hỏng, layerSensesFailed(layer) đúng — và layer đó KHÔNG có khoá trong sensesByLayer', async () => {
    const { layerSensesFailed, runLookup, sensesByLayer } = await import('../../src/panels/lookupPanelState')

    expect(layerSensesFailed('layer-hong')).toBe(false)
    await runLookup('測試')

    expect(layerSensesFailed('layer-hong')).toBe(true)
    // `undefined`, not `[]` — see doc-comment on `LookupResponse.senses_failed`.
    expect(sensesByLayer.value['layer-hong']).toBeUndefined()
    expect(layerSensesFailed('mot-lop-khac')).toBe(false)
  })
})

describe('LookupRecord — sensesFailed hiện banner, KHÔNG một khối nghĩa trống câm lặng', () => {
  it('sensesFailed=true ⇒ hiện thông báo hydrate hỏng, không lặp cluster.senses (rỗng)', () => {
    const w = mount(LookupRecord, {
      props: { group: GROUP, senses: [], sensesFailed: true },
    })
    expect(w.find('.lookup-senses-failed').exists()).toBe(true)
    // Entry chrome (headword/pin button) still renders — phase one matched for real.
    expect(w.find('.lookup-entry').exists()).toBe(true)
    w.unmount()
  })

  it('sensesFailed=false + senses rỗng ⇒ KHÔNG hiện banner (đây là "đầu mục không có nghĩa", một hình dạng hợp lệ khác)', () => {
    const w = mount(LookupRecord, {
      props: { group: GROUP, senses: [], sensesFailed: false },
    })
    expect(w.find('.lookup-senses-failed').exists()).toBe(false)
    w.unmount()
  })
})
