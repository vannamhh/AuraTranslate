/**
 * `editorPanelState.ts::dieuHuongVaBao` — trong khi một lượt xác nhận đang bay, một lệnh điều
 * hướng segment phải bị TỪ CHỐI VÀ KÊU, cùng khuôn `regroupInFlight`. Lượt xác nhận tự đọc
 * `caretSegmentId`/`segments` TRƯỚC lượt IPC rồi mới dời con trỏ khi nó về; một lệnh điều
 * hướng chen ngang sẽ bị lượt xác nhận ghi ĐÈ con trỏ khi nó về, xoá thao tác đó trong im lặng.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { FIXTURE_SEGMENTS, readFixture, recordSave, resetRecorder } from './support/segmentFixture'

/** Không bao giờ tự trả lời — giữ lượt xác nhận "đang bay" cho tới khi ca test gọi `resolve()`. */
let resolveConfirm: (() => void) | null = null
let resolveStarted: (() => void) | null = null
/** Ca test `await` cái này TRƯỚC khi gọi `resolveConfirm()` — bảo đảm `confirmSegment` đã thật
 * sự được gọi (không chỉ `confirmCurrentSegment()` đã trả về một promise đang chờ). */
let started = new Promise<void>((resolve) => {
  resolveStarted = resolve
})
async function confirmDangBay() {
  resolveStarted?.()
  await new Promise<void>((resolve) => {
    resolveConfirm = resolve
  })
  return { outcome: { id: 12, status: 'confirmed' }, error: null }
}

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture, saveSegmentTargets: recordSave, confirmSegment: confirmDangBay }
})

async function tuoi() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  await state.ensureSegmentsLoaded()
  return state
}

beforeEach(() => {
  resetRecorder()
  resolveConfirm = null
  started = new Promise((resolve) => {
    resolveStarted = resolve
  })
})

describe('lượt xác nhận đang bay chặn lệnh điều hướng segment', () => {
  it('gọi `editor.next_segment` trong lúc xác nhận đang bay ⇒ `false`, con trỏ KHÔNG dời, notice `confirm-in-flight`', async () => {
    const state = await tuoi()
    state.setEditorCaret(12)

    const dangKy = state.confirmCurrentSegment()

    // Lượt xác nhận đã bắt đầu bay (đã set `confirmInFlight`, còn đang chờ IPC) — CHƯA await.
    expect(state.goToNextSegmentCoBao()).toBe(false)
    expect(state.editorCaretSegmentId.value).toBe(12)
    expect(state.editorNavNotice.value).toBe('confirm-in-flight')

    await started
    resolveConfirm?.()
    await dangKy

    expect(FIXTURE_SEGMENTS.length).toBeGreaterThan(0)
  })
})
