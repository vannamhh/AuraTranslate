/**
 * AC1 of the standalone harvest: after the import overlay closes with harvested candidates, the
 * pending queue opens and focus sits inside its panel; closing it returns focus to a live element.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'

const pendingMock = vi.fn()
const lookupMock = vi.fn()

vi.mock('../../src/config/glossary', () => ({
  pendingGlossaryCandidates: () => pendingMock(),
  lookupGlossaryTerm: (term: string) => lookupMock(term),
  approveGlossaryCandidate: vi.fn(),
  rejectGlossaryCandidate: vi.fn(),
}))

vi.mock('../../src/config/reviewerImport', () => ({
  reviewerImportOpenPreview: async () => ({
    outcome: 'loaded',
    preview: { file_name: 'r.docx', file_kind: 'docx', chapters: [], skipped: [], image_rows_ignored: 0 },
  }),
  reviewerImportConfirm: async () => ({
    summary: { chapter_count: 1, row_count: 3, replaced_count: 0, harvest_candidate_count: 2, harvest_error: null },
    error: null,
  }),
  reviewerImportCancel: async () => ({ ok: true, error: null }),
}))

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  document.body.innerHTML = ''
})

function harvestCandidate(id: number) {
  return {
    id,
    source_term: '北凉王',
    candidate_origin: 'review_harvest',
    resolution: null,
    created_at: '2026-10-09T00:00:00.000Z',
    occurrence_count: 3,
    context_example: 'ví dụ',
    han_viet_suggestion: null,
    han_viet_status: 'not_chinese',
    chapter_span_count: 0,
    replaced_translation: 'Bắc Lương vương',
    proposed_translation: 'vương Bắc Lương',
    changed_count: 3,
    seen_count: 5,
  }
}

describe('focus after a reviewer import with harvested candidates', () => {
  it('lands inside the queue panel, then returns to a connected element on close', async () => {
    vi.resetModules()
    pendingMock.mockReset()
    lookupMock.mockReset()
    pendingMock.mockResolvedValue({ candidates: [harvestCandidate(1), harvestCandidate(2)], error: null })
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    const commands = await import('../../src/commands')
    commands.installCommands({} as never)
    const queue = await import('../../src/glossaryQueueState')
    const importState = await import('../../src/reviewerImportState')
    const GlossaryQueueOverlay = (await import('../../src/GlossaryQueueOverlay.vue')).default
    const ReviewerImportOverlay = (await import('../../src/ReviewerImportOverlay.vue')).default

    const opener = document.createElement('button')
    opener.setAttribute('data-glossary-queue-open', '')
    document.body.appendChild(opener)

    const Host = defineComponent({ render: () => [h(GlossaryQueueOverlay), h(ReviewerImportOverlay)] })
    const wrapper = mount(Host, { attachTo: document.body })

    importState.installReviewerImportHooks({
      afterClosedWithHarvest: () => {
        void queue.openGlossaryQueue()
      },
    })

    await importState.openReviewerImportPreviewOverlay()
    await importState.confirmReviewerImportPreview()
    await wrapper.vm.$nextTick()
    await importState.cancelReviewerImportPreview()
    await vi.waitFor(() => expect(queue.queueOverlayIsOpen.value).toBe(true))
    await new Promise((resolve) => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()

    const panel = document.querySelector('.gq-panel')
    expect(panel).not.toBeNull()
    expect(panel?.contains(document.activeElement)).toBe(true)

    queue.closeGlossaryQueue()
    await wrapper.vm.$nextTick()
    expect(document.activeElement?.isConnected).toBe(true)
    expect(document.activeElement).not.toBe(document.body)
    wrapper.unmount()
  })
})
