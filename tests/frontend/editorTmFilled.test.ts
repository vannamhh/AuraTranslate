import { beforeEach, describe, expect, it, vi } from 'vitest'
import { FIXTURE_CHAPTER_ID, FIXTURE_SEGMENTS, resetRecorder, recordSave } from './support/segmentFixture'
import vi_json from '../../src/i18n/vi.json'

const filledIds = { value: [12] as number[] }
const prefill = { value: { kind: 'ran', hidden_count: 0 } as { kind: 'ran'; hidden_count: number } | { kind: 'skipped'; code: string } }

const confirmFilled = { value: [] as { segment_id: number; target_text: string; translation_origin: string }[] }
const confirmPrefill = { value: { kind: 'not_asked' } as { kind: string; hidden_count?: number; code?: string } }
const confirmHook = { value: null as (() => void) | null }

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: async () => ({
      loaded: {
        chapter_id: FIXTURE_CHAPTER_ID,
        segments: FIXTURE_SEGMENTS.map((s) => ({ ...s })),
        caret_segment_id: null,
        assets: [],
        assets_dir: '',
        tm_filled_segment_ids: filledIds.value,
        tm_prefill: prefill.value,
      },
      error: null,
    }),
    saveSegmentTargets: recordSave,
    confirmSegment: async (segmentId: number) => {
      confirmHook.value?.()
      return {
        outcome: {
          segment_id: segmentId,
          status: 'confirmed',
          version_created: true,
          chapter_started: false,
          tm_filled: confirmFilled.value,
          tm_prefill: confirmPrefill.value,
        },
        error: null,
      }
    },
  }
})

async function freshState() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  await state.ensureSegmentsLoaded()
  return state
}

beforeEach(() => {
  resetRecorder()
  filledIds.value = [12]
  prefill.value = { kind: 'ran', hidden_count: 0 }
  confirmFilled.value = []
  confirmPrefill.value = { kind: 'not_asked' }
  confirmHook.value = null
})

describe('dấu TM điền sẵn sống trong phiên', () => {
  it('nạp Chương ⇒ tập mang đúng các id Rust báo', async () => {
    const state = await freshState()
    expect([...state.editorTmFilledSegmentIds.value]).toEqual([12])
  })

  it('gõ vào segment ⇒ dấu rơi', async () => {
    const state = await freshState()
    state.noteEditorEdit(12, 'Đã sửa.')
    expect(state.editorTmFilledSegmentIds.value.has(12)).toBe(false)
  })

  it('xác nhận segment ⇒ dấu rơi', async () => {
    const state = await freshState()
    state.setEditorCaret(12)
    expect(await state.confirmCurrentSegment()).toBe('confirmed')
    expect(state.editorTmFilledSegmentIds.value.has(12)).toBe(false)
  })

  it('nạp lại mà Rust không báo id nào ⇒ tập rỗng, segment chỉ còn `draft`', async () => {
    const state = await freshState()
    filledIds.value = []
    state.resetEditorPanel()
    await state.ensureSegmentsLoaded()
    expect(state.editorTmFilledSegmentIds.value.size).toBe(0)
  })

  it('nhãn trạng thái nói rõ cần xác nhận', () => {
    expect((vi_json as Record<string, string>)['panel.grid.state_tm']).toContain('cần xác nhận')
  })
})

describe('a skipped TM pre-fill is a non-fatal notice', () => {
  it('skipped => the Chapter still loads and the StatusBar notice is set', async () => {
    prefill.value = { kind: 'skipped', code: 'store.open_failed' }
    filledIds.value = []
    vi.spyOn(console, 'error').mockImplementation(() => {})
    const state = await freshState()

    expect(state.editorLoadError.value).toBeNull()
    expect(state.editorSegments.value.length).toBeGreaterThan(0)
    expect(state.editorNavNotice.value).toBe('tm-prefill-skipped')
  })

  it('ran with hidden pairs => a hidden-count notice carrying the count', async () => {
    prefill.value = { kind: 'ran', hidden_count: 6 }
    const state = await freshState()
    expect(state.editorNavNotice.value).toBe('tm-prefill-hidden')
    expect(state.editorTmPrefillHiddenCount.value).toBe(6)
  })

  it('the StatusBar names the hidden count after a pre-fill that left pairs out', async () => {
    prefill.value = { kind: 'ran', hidden_count: 6 }
    await freshState()
    const { mount } = await import('@vue/test-utils')
    const StatusBar = (await import('../../src/StatusBar.vue')).default
    ;(await import('../../src/modes/modeState')).setMode('workspace')
    const wrapper = mount(StatusBar)
    expect(wrapper.find('.notice').text()).toContain('6')
    wrapper.unmount()
  })

  it('ran => no notice', async () => {
    const state = await freshState()
    expect(state.editorNavNotice.value).toBeNull()
  })

  it('the StatusBar shows the mapped text', async () => {
    prefill.value = { kind: 'skipped', code: 'tm.lookup_failed' }
    vi.spyOn(console, 'error').mockImplementation(() => {})
    await freshState()
    const { mount } = await import('@vue/test-utils')
    const StatusBar = (await import('../../src/StatusBar.vue')).default
    ;(await import('../../src/modes/modeState')).setMode('workspace')
    const wrapper = mount(StatusBar)

    expect(wrapper.find('.notice').text()).toBe((vi_json as Record<string, string>)['panel.grid.nav_tm_prefill_skipped'])
    wrapper.unmount()
  })
})

describe('same-source fill returned by a confirm', () => {
  const filledRow = { segment_id: 13, target_text: 'Bản vừa ký.', translation_origin: 'self' }

  it('patches the snapshot and marks the Segment as TM-filled, still unconfirmed', async () => {
    filledIds.value = []
    const state = await freshState()
    confirmFilled.value = [filledRow]
    confirmPrefill.value = { kind: 'ran', hidden_count: 0 }
    state.setEditorCaret(11)
    expect(await state.confirmCurrentSegment()).toBe('confirmed')

    const row = state.editorSegments.value.find((s) => s.id === 13)
    expect(row?.target_text).toBe('Bản vừa ký.')
    expect(row?.translation_origin).toBe('self')
    expect(row?.status).not.toBe('confirmed')
    expect(state.editorTmFilledSegmentIds.value.has(13)).toBe(true)
  })

  it('unflushed typing in that Segment wins over the filled text', async () => {
    filledIds.value = []
    const state = await freshState()
    confirmFilled.value = [filledRow]
    confirmPrefill.value = { kind: 'ran', hidden_count: 0 }
    confirmHook.value = () => state.noteEditorEdit(13, 'gõ dở')
    state.setEditorCaret(11)
    await state.confirmCurrentSegment()

    expect(state.editorSegments.value.find((s) => s.id === 13)?.target_text).toBe('')
    expect(state.editorTmFilledSegmentIds.value.has(13)).toBe(false)
  })

  it('a skipped fill sets the new notice and logs, the confirm stays confirmed', async () => {
    filledIds.value = []
    const err = vi.spyOn(console, 'error').mockImplementation(() => {})
    const state = await freshState()
    confirmPrefill.value = { kind: 'skipped', code: 'store.open_failed' }
    state.setEditorCaret(11)
    expect(await state.confirmCurrentSegment()).toBe('confirmed')

    expect(state.editorNavNotice.value).toBe('tm-prefill-skipped-after-confirm')
    expect(err).toHaveBeenCalled()
  })

  it('ran with hidden pairs only logs, no notice', async () => {
    filledIds.value = []
    const err = vi.spyOn(console, 'error').mockImplementation(() => {})
    const state = await freshState()
    confirmPrefill.value = { kind: 'ran', hidden_count: 2 }
    state.setEditorCaret(11)
    await state.confirmCurrentSegment()

    expect(state.editorNavNotice.value).toBeNull()
    expect(err).toHaveBeenCalled()
  })
})
