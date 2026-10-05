import { beforeEach, describe, expect, it, vi } from 'vitest'
import { FIXTURE_CHAPTER_ID, FIXTURE_SEGMENTS, resetRecorder, recordSave } from './support/segmentFixture'
import vi_json from '../../src/i18n/vi.json'

const filledIds = { value: [12] as number[] }
const prefill = { value: { kind: 'ran' } as { kind: 'ran' } | { kind: 'skipped'; code: string } }

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
    confirmSegment: async (segmentId: number) => ({
      outcome: { segment_id: segmentId, status: 'confirmed', version_created: true },
      error: null,
    }),
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
  prefill.value = { kind: 'ran' }
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
    const wrapper = mount(StatusBar)

    expect(wrapper.find('.notice').text()).toBe((vi_json as Record<string, string>)['panel.grid.nav_tm_prefill_skipped'])
    wrapper.unmount()
  })
})
