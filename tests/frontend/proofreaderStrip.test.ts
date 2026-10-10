import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { flushPromises } from './support/flushMicrotasks'

const runMock = vi.fn()

vi.mock('../../src/config/proofread', () => ({
  runProofreadSegment: (...args: unknown[]) => runMock(...args),
  cancelProofreadCall: () => Promise.resolve(),
}))
vi.mock('../../src/panels/editorPanelState', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/panels/editorPanelState')>()),
  flushEditorBeforeDiscreteWrite: () => Promise.resolve('clean'),
}))

const { setEditorCaret } = await import('../../src/panels/editorPanelState')
const { runProofread, resetProofread } = await import('../../src/proofreadState')
const { t } = await import('../../src/i18n')
const ProofreaderStrip = (await import('../../src/ProofreaderStrip.vue')).default

afterEach(() => {
  resetProofread()
})

describe('ProofreaderStrip', () => {
  it('invites to configure AI when the scan reports not_configured', async () => {
    setEditorCaret(5)
    runMock.mockResolvedValue({ value: { state: 'not_configured' }, error: null })
    const wrapper = mount(ProofreaderStrip, { attachTo: document.body })
    await runProofread(5)
    await flushPromises()

    expect(wrapper.find('[role="status"]').text()).toBe(t('panel.ai_translation.status'))
    wrapper.unmount()
  })
})
