/**
 * `GridPanel.vue::rowErrorLabelById` — khi hai trong ba nguồn lỗi (xác nhận · khôi phục · flush)
 * cùng trỏ vào một hàng, hàng đó phải hiện `message_key` của nguồn ƯU TIÊN cao hơn (thứ tự đã
 * khai ở doc-comment của `rowErrorLabelById`: xác nhận > khôi phục > flush). Khuôn mount THẬT
 * `gridPanelImages.test.ts`, giả ở biên IPC (`config/segment.ts`) như `editorConfirmSegment.test.ts`.
 *
 * 🔴 Ca đầu (xác nhận > flush) là bản gốc. Hai ca sau thêm cho hai chân ƯU TIÊN mà nó không đi
 * qua — Phase 3 của Story 11.5 tự ghi lại là chưa canh riêng: xác nhận > khôi phục, khôi phục >
 * flush (không có lỗi xác nhận nào ở chân này, để chứng minh khôi phục tự nó khoá được flush,
 * không chỉ ăn theo cùng guard mà chân xác nhận > flush đã khoá).
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { failNextSave, readFixture, recordSave, resetRecorder } from './support/segmentFixture'

vi.mock('@tauri-apps/api/core', () => ({
  convertFileSrc: (path: string) => `mock-asset://${path}`,
  invoke: () => Promise.reject(new Error('invoke khong duoc goi truc tiep trong ca nay')),
}))

const failNextConfirm = { value: false }

async function recordConfirm(segmentId: number, _textAtLoad: string, _originAtLoad: string) {
  if (failNextConfirm.value) {
    failNextConfirm.value = false
    return {
      outcome: null,
      error: {
        code: 'segment.nothing_to_confirm',
        message_key: 'err.segment.nothing_to_confirm',
        params: { segment_id: String(segmentId) },
        retryable: false,
      },
    }
  }
  return { outcome: { segment_id: segmentId, status: 'confirmed', version_created: true }, error: null }
}

/** `null` ⇒ lượt khôi phục kế tiếp THÀNH CÔNG; khác `null` ⇒ nó bị từ chối với đúng khoá này. */
const failNextRestoreKey: { value: 'err.segment.not_found' | 'err.segment.retired' | null } = { value: null }

async function recordRestore(segmentId: number, _versionId: number, _force: boolean) {
  const key = failNextRestoreKey.value
  if (key !== null) {
    failNextRestoreKey.value = null
    return {
      outcome: null,
      error: {
        code: key === 'err.segment.not_found' ? 'segment.not_found' : 'segment.retired',
        message_key: key,
        params: { segment_id: String(segmentId) },
        retryable: false,
      },
    }
  }
  return {
    outcome: { segment_id: segmentId, status: 'draft', restored: true, needs_confirmation: false, unsigned_draft: null },
    error: null,
  }
}

async function recordHistory(_segmentId: number) {
  return { versions: [], error: null }
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSave,
    confirmSegment: recordConfirm,
    restoreSegmentVersion: recordRestore,
    readSegmentHistory: recordHistory,
  }
})

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

async function mountGrid() {
  vi.resetModules()
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const state = await import('../../src/panels/editorPanelState')
  const history = await import('../../src/panels/segmentHistoryState')
  const wrapper = mount(GridPanel, { props: { params: {} } as never, global: { stubs: STUBS }, attachTo: document.body })
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { wrapper, state, history }
}

function stateCellTextFor(wrapper: ReturnType<typeof mount>, index: number): string | null {
  const cells = Array.from((wrapper.element as unknown as Element).querySelectorAll('.col-state .cell-state'))
  const cell = cells.at(index)
  return cell === undefined ? null : cell.textContent.trim()
}

beforeEach(() => {
  document.body.innerHTML = ''
  resetRecorder()
  failNextConfirm.value = false
  failNextRestoreKey.value = null
})

describe('GridPanel.vue — nhãn hàng ưu tiên lỗi xác nhận trên lỗi flush', () => {
  it('cùng một segment mang cả hai lỗi ⇒ hàng hiện lỗi XÁC NHẬN, không lỗi flush', async () => {
    const { wrapper, state } = await mountGrid()
    const { t } = await import('../../src/i18n')

    // Câu 12 (chỉ số 1, thứ hai trong fixture): dịch rồi, chưa ký — hợp lệ để ký.
    state.setEditorCaret(12)
    failNextConfirm.value = true
    expect(await state.confirmCurrentSegment()).toBe('refused')
    expect(state.editorConfirmError.value?.message_key).toBe('err.segment.nothing_to_confirm')

    // Cùng câu 12: gõ rồi flush trượt — lỗi flush chở CHÍNH `id` này trong `segmentIds`.
    state.noteEditorEdit(12, 'Bản mới, sắp mất khi flush trượt.')
    failNextSave.value = true
    expect(await state.flushEditorNow()).toBe('failed')
    expect(state.editorFlushError.value?.segmentIds).toEqual([12])

    await wrapper.vm.$nextTick()

    expect(stateCellTextFor(wrapper, 1)).toBe(t('err.segment.nothing_to_confirm', { segment_id: '12' }))
    expect(stateCellTextFor(wrapper, 1)).not.toBe(t('err.store.write_failed', {}))
  })

  it('cùng một segment mang cả lỗi xác nhận VÀ lỗi khôi phục ⇒ hàng hiện lỗi XÁC NHẬN, không lỗi khôi phục', async () => {
    const { wrapper, state, history } = await mountGrid()
    const { t } = await import('../../src/i18n')

    // Câu 12 (chỉ số 1): xác nhận trượt trước.
    state.setEditorCaret(12)
    failNextConfirm.value = true
    expect(await state.confirmCurrentSegment()).toBe('refused')
    expect(state.editorConfirmError.value?.message_key).toBe('err.segment.nothing_to_confirm')

    // Cùng câu 12: mở lịch sử (nhắm đúng câu đang có caret) rồi khôi phục trượt.
    history.openSegmentHistory()
    failNextRestoreKey.value = 'err.segment.not_found'
    expect(await history.restoreVersion(999, false)).toBe('refused')
    expect(history.historyRestoreError.value?.message_key).toBe('err.segment.not_found')

    await wrapper.vm.$nextTick()

    expect(stateCellTextFor(wrapper, 1)).toBe(t('err.segment.nothing_to_confirm', { segment_id: '12' }))
    expect(stateCellTextFor(wrapper, 1)).not.toBe(t('err.segment.not_found', { segment_id: '12' }))
  })

  it('cùng một segment mang cả lỗi khôi phục VÀ lỗi flush, KHÔNG lỗi xác nhận ⇒ hàng hiện lỗi KHÔI PHỤC, không lỗi flush', async () => {
    const { wrapper, state, history } = await mountGrid()
    const { t } = await import('../../src/i18n')

    // Câu 11 (chỉ số 0): đã ký, có văn bản — không chạm lỗi xác nhận ở chân này.
    state.setEditorCaret(11)
    history.openSegmentHistory()
    failNextRestoreKey.value = 'err.segment.retired'
    expect(await history.restoreVersion(999, false)).toBe('refused')
    expect(history.historyRestoreError.value?.message_key).toBe('err.segment.retired')

    // Cùng câu 11: gõ rồi flush trượt.
    state.noteEditorEdit(11, 'Bản mới, sắp mất khi flush trượt.')
    failNextSave.value = true
    expect(await state.flushEditorNow()).toBe('failed')
    expect(state.editorFlushError.value?.segmentIds).toEqual([11])

    await wrapper.vm.$nextTick()

    expect(stateCellTextFor(wrapper, 0)).toBe(t('err.segment.retired', { segment_id: '11' }))
    expect(stateCellTextFor(wrapper, 0)).not.toBe(t('err.store.write_failed', {}))
  })
})
