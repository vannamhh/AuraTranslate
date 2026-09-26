/**
 * Calls `toggleDictSource` directly since `CommandRegistry` is wired only in `src/main.ts`,
 * which the test tree cannot import; this is the real handler behind `lookup.toggle_source`.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import LookupPanel from '../../src/panels/LookupPanel.vue'
import { loadDictSources, resetDictSources, toggleDictSource } from '../../src/panels/dictSourcesState'
import type { SourceAttribution } from '../../src/config/dict'

const FAKE_SOURCE: SourceAttribution = {
  code: 'test-src',
  display_name: 'Nguồn Thử',
  license_kind: 'open',
  license_id: null,
  license_text_len: 0,
  attribution: 'Thử nghiệm',
  source_version: '1',
  source_url: 'https://example.invalid',
  lang: 'zh',
  layer: 'test-layer',
  is_base: true,
}

// `happy-dom` has no `__TAURI_INTERNALS__` bridge, so `loadDictSources` would otherwise
// fall into the "running outside Tauri" branch instead of loading a source for this test.
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === 'list_dict_sources') return { sources: [FAKE_SOURCE], skipped: [] }
    if (cmd === 'put_config') return undefined
    throw new Error(`sourceChipToggle.test.ts: invoke giả không biết lệnh "${cmd}"`)
  }),
}))

function dungPanel() {
  return mount(LookupPanel, {
    props: { params: { params: {} } },
  })
}

beforeEach(async () => {
  await loadDictSources('')
})

afterEach(() => {
  resetDictSources()
})

describe('LookupPanel — chip nguồn vẽ lại khi bật/tắt', () => {
  it('tắt một nguồn ⇒ chip mang lớp `off`, KHÔNG một `opacity` viết tay nào; bật lại ⇒ lớp mất', () => {
    const w = dungPanel()

    const chip = () => w.find('[data-source-code="test-src"]')
    expect(chip().exists()).toBe(true)

    // ① — trạng thái đầu: đang BẬT, không mang lớp `off`.
    expect(chip().classes()).not.toContain('off')

    // ② — tắt qua handler THẬT của `lookup.toggle_source` ⇒ chip VẼ LẠI thành trạng thái tắt.
    toggleDictSource('test-src')
    return w.vm.$nextTick().then(() => {
      expect(chip().classes()).toContain('off')
      // Mệnh đề trung tâm của ②: không một `opacity` viết tay nào đè lên phần tử — tín hiệu
      // tắt là `color` + `text-decoration` (CSS lớp `.source-chip.off`), không độ đục.
      const styleAttr = chip().attributes('style')
      expect(styleAttr === undefined || !styleAttr.includes('opacity')).toBe(true)

      // Đối chứng hai chiều — bật lại xoá đúng lớp đó, không để lại tàn dư.
      toggleDictSource('test-src')
      return w.vm.$nextTick().then(() => {
        expect(chip().classes()).not.toContain('off')
        w.unmount()
      })
    })
  })
})
