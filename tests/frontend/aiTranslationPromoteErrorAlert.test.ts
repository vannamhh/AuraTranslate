/**
 * `AiTranslationPanel.vue` — `editorPromoteAiTranslationError` phải hiện qua `tError()`, cùng
 * bề mặt `editorConfirmError`/`aiTranslateError`, không chỉ được đọc mà không component nào
 * render. Khuôn mount THẬT `aiTranslate.test.ts::freshPanel()`, thu gọn — chỉ cần panel mount
 * và đọc đúng ba export của `editorPanelState.ts`.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { Ref } from 'vue'
import type { IpcError } from '../../src/i18n'

vi.mock('../../src/config/aitranslate', () => ({
  runAiTranslateSegment: vi.fn(),
  cancelAiTranslateCall: vi.fn(),
  promoteAiTranslation: vi.fn(),
}))

const promoteError: Ref<IpcError | null> = ref(null)

vi.mock('../../src/panels/editorPanelState', () => ({
  editorCaretSegmentId: ref<number | null>(null),
  editorPendingPromote: ref(null),
  editorPromoteAiTranslationError: promoteError,
  promoteAiTranslationToEditor: vi.fn(),
}))

async function mountPanel() {
  vi.resetModules()
  promoteError.value = null
  const AiTranslationPanel = (await import('../../src/panels/AiTranslationPanel.vue')).default
  const i18n = await import('../../src/i18n')
  return { wrapper: mount(AiTranslationPanel, { props: { params: { params: {} } }, attachTo: document.body }), i18n }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('AiTranslationPanel.vue — lỗi PROMOTE hiện qua tError(), cùng bề mặt các lỗi khác', () => {
  it('`editorPromoteAiTranslationError` khác null ⇒ `.ai-translate-alert` hiện đúng câu tError()', async () => {
    const { wrapper, i18n } = await mountPanel()

    promoteError.value = {
      code: 'segment.not_found',
      message_key: 'err.segment.not_found',
      params: { segment_id: '9' },
      retryable: false,
    }
    await wrapper.vm.$nextTick()

    const alert = wrapper.find('[data-ai-translate-promote-alert]')
    expect(alert.exists()).toBe(true)
    expect(alert.text()).toBe(i18n.tError(promoteError.value))
  })

  it('`editorPromoteAiTranslationError` là null ⇒ không alert nào hiện', async () => {
    const { wrapper } = await mountPanel()
    expect(wrapper.find('[data-ai-translate-promote-alert]').exists()).toBe(false)
  })
})
