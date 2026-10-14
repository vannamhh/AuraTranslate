/** Han Viet cell, 12.7 (D4): the "Nguồn: …" line is gone even when sources contributed readings. */
import { describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import vi_json from '../../src/i18n/vi.json'

vi.mock('../../src/config/chapter', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/chapter')>()),
  readOpenChapter: async () => ({
    chapter: { chapter_id: 7, source_lang: 'zh', source_text: '京都' },
    error: null,
  }),
}))
vi.mock('../../src/config/dict', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/dict')>()),
  readHanViet: async () => ({
    lookup: {
      characters: [
        { character: '京', reading: null },
        { character: '都', reading: null },
      ],
      sources_used: ['src-a'],
      layers_loaded: true,
    },
    error: null,
  }),
}))

describe('SourceHanViet — không còn dòng "Nguồn: …"', () => {
  it('nguồn đã đóng góp âm mà ô vẫn không in dòng nguồn', async () => {
    vi.resetModules()
    const panel = await import('../../src/panels/sourcePanelState')
    await panel.ensureChapterLoaded()
    await flushPromises()
    expect(panel.sourcesUsed.value).toEqual(['src-a'])

    const SourceHanViet = (await import('../../src/panels/SourceHanViet.vue')).default
    const wrapper = mount(SourceHanViet, {
      props: { sourceText: '京都', viewMode: 'switch' as const, cuts: [], glossaryTerms: [], surfaceRole: 'cell' as const },
    })
    await flushPromises()

    expect(wrapper.find('.hv-sources').exists()).toBe(false)
    expect(wrapper.text()).not.toMatch(/Nguồn\s*:/)
    expect('panel.source.han_viet_sources_prefix' in (vi_json as Record<string, string>)).toBe(false)
    wrapper.unmount()
  })
})
