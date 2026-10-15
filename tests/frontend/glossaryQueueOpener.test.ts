import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'

const pendingMock = vi.fn()

vi.mock('../../src/config/glossary', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/glossary')>()),
  pendingGlossaryCandidates: () => pendingMock(),
  lookupGlossaryTerm: async () => ({ found: 'none', workTierAvailable: true }),
  countGlossaryEntries: async () => null,
}))

beforeEach(() => {
  document.body.innerHTML = ''
  vi.resetModules()
  pendingMock.mockReset()
})

async function mountWorkspace() {
  const commands = await import('../../src/commands')
  const queue = await import('../../src/glossaryQueueState')
  commands.installCommands({
    openGlossaryQueue: () => void queue.openGlossaryQueue(),
    closeGlossaryQueue: () => queue.closeGlossaryQueue(),
  } as never)
  const StatusBar = (await import('../../src/StatusBar.vue')).default
  const Overlay = (await import('../../src/GlossaryQueueOverlay.vue')).default
  const Host = defineComponent({ render: () => [h(StatusBar), h(Overlay)] })
  const wrapper = mount(Host, { attachTo: document.body })
  return { queue, wrapper }
}

describe('Workspace opener of the Glossary queue', () => {
  it('exists with the data hook while nothing is pending, opens the queue on click, and Esc returns focus to it', async () => {
    pendingMock.mockResolvedValue({ candidates: [], error: null })
    const { queue, wrapper } = await mountWorkspace()
    const opener = wrapper.get('[data-glossary-queue-open]')
    expect(opener.element.tagName).toBe('BUTTON')

    await opener.trigger('mousedown')
    await opener.trigger('click')
    await vi.waitFor(() => expect(queue.queueOverlayIsOpen.value).toBe(true))
    await wrapper.vm.$nextTick()
    expect(document.querySelector('.gq-panel')?.contains(document.activeElement)).toBe(true)

    await wrapper.get('.gq-scrim').trigger('keydown', { key: 'Escape' })
    await wrapper.vm.$nextTick()
    expect(queue.queueOverlayIsOpen.value).toBe(false)
    expect(document.activeElement).toBe(opener.element)
    wrapper.unmount()
  })
})
