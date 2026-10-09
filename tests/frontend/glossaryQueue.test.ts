/**
 * State của lớp phủ **Duyệt hàng loạt một phím** — Story 3.8, FR53/FR55.
 *
 * ⚠️ Cùng khuôn `glossaryQuickAdd.test.ts`/`glossarySettings.test.ts`: `config/glossary.ts`
 * là biên IPC, giả lập bằng `vi.mock`, không gọi `@tauri-apps/api` thật.
 *
 * ⚠️ **Thứ tự bắt buộc trong mỗi ca**: `freshState()` TRƯỚC, cấu hình `mockResolvedValue`
 * SAU — `freshState()` tự `mockReset()` bốn mock, nên đặt cấu hình trước nó sẽ bị chính nó
 * xoá mất (cùng khuôn `glossaryQuickAdd.test.ts`/`glossarySettings.test.ts`).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { GlossaryCandidate, GlossaryCategory, GlossaryLookupResult } from '../../src/config/glossary'
import type { CommandDeps } from '../../src/commands'

const pendingMock = vi.fn()
const lookupMock = vi.fn()
const approveMock = vi.fn()
const rejectMock = vi.fn()

vi.mock('../../src/config/glossary', () => ({
  pendingGlossaryCandidates: () => pendingMock(),
  lookupGlossaryTerm: (term: string) => lookupMock(term),
  approveGlossaryCandidate: (...args: unknown[]) => approveMock(...args),
  rejectGlossaryCandidate: (id: number) => rejectMock(id),
}))

/** Nạp lại module mỗi ca — state của lớp phủ là module-level singleton. */
async function freshState() {
  vi.resetModules()
  pendingMock.mockReset()
  lookupMock.mockReset()
  approveMock.mockReset()
  rejectMock.mockReset()
  return import('../../src/glossaryQueueState')
}

function candidate(over: Partial<GlossaryCandidate> = {}): GlossaryCandidate {
  return {
    id: 1,
    source_term: '萧炎',
    candidate_origin: 'import_scan',
    resolution: null,
    created_at: '2026-08-24T00:00:00.000Z',
    occurrence_count: 10,
    context_example: 'ví dụ',
    han_viet_suggestion: null,
    han_viet_status: 'not_chinese',
    chapter_span_count: 0,
    replaced_translation: null,
    proposed_translation: null,
    changed_count: null,
    seen_count: null,
    ...over,
  }
}

const workOpenLookup: GlossaryLookupResult = { found: 'none', workTierAvailable: true }
const workClosedLookup: GlossaryLookupResult = { found: 'none', workTierAvailable: false }

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('openGlossaryQueue — nạp danh sách, giữ NGUYÊN thứ tự backend đã trả', () => {
  it('con trỏ ở hàng đầu, mọi hàng bắt đầu category "other" và outcome null', async () => {
    const { openGlossaryQueue, queueRows, queueCursor, queueStatus } = await freshState()
    const a = candidate({ id: 1, source_term: '甲', occurrence_count: 10 })
    const b = candidate({ id: 2, source_term: '乙', occurrence_count: 9 })
    pendingMock.mockResolvedValue({ candidates: [a, b], error: null })

    await openGlossaryQueue()

    expect(queueStatus.value).toBe('loaded')
    expect(queueRows.value.map((r) => r.candidate.id)).toEqual([1, 2])
    expect(queueRows.value.every((r) => r.category === 'other' && r.outcome === null)).toBe(true)
    expect(queueCursor.value).toBe(0)
  })

  it('KHÔNG tự sắp lại — thứ tự backend trả là thứ tự hiển thị, kể cả khi "trông" không sắp', async () => {
    const { openGlossaryQueue, queueRows } = await freshState()
    // Cố ý ĐẢO thứ tự tần suất để chứng minh state không tự ý sort lại — đúng thứ tự
    // Rust trả (occurrence_count DESC, id ASC) là thứ tự DUY NHẤT được tin.
    const rows = [
      candidate({ id: 3, source_term: '丙', occurrence_count: 1 }),
      candidate({ id: 1, source_term: '甲', occurrence_count: 5 }),
      candidate({ id: 2, source_term: '乙', occurrence_count: 3 }),
    ]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })

    await openGlossaryQueue()

    expect(queueRows.value.map((r) => r.candidate.id)).toEqual([3, 1, 2])
  })
})

describe('Ba ca RỖNG khác nhau — §Always: "rỗng phải nói vì sao nó rỗng"', () => {
  it('chưa mở Tác phẩm nào ⇒ status "no_work" (KHÔNG "loaded" rỗng)', async () => {
    const { openGlossaryQueue, queueStatus, queueRows } = await freshState()
    pendingMock.mockResolvedValue({ candidates: [], error: null })
    lookupMock.mockResolvedValue(workClosedLookup)

    await openGlossaryQueue()

    expect(queueStatus.value).toBe('no_work')
    expect(queueRows.value).toEqual([])
    expect(lookupMock).toHaveBeenCalledTimes(1)
  })

  it('đã mở, bảng chờ sạch (đã duyệt hết) ⇒ status "loaded" với 0 hàng', async () => {
    const { openGlossaryQueue, queueStatus, queueRows } = await freshState()
    pendingMock.mockResolvedValue({ candidates: [], error: null })
    lookupMock.mockResolvedValue(workOpenLookup)

    await openGlossaryQueue()

    expect(queueStatus.value).toBe('loaded')
    expect(queueRows.value).toEqual([])
  })

  it('cầu IPC vắng (chạy ngoài Tauri) ⇒ status "ipc_unavailable", KHÔNG một lượt lookup thêm', async () => {
    const { openGlossaryQueue, queueStatus } = await freshState()
    pendingMock.mockResolvedValue({ candidates: null, error: null })

    await openGlossaryQueue()

    expect(queueStatus.value).toBe('ipc_unavailable')
    expect(lookupMock).not.toHaveBeenCalled()
  })

  it('lượt tải trượt THẬT (IpcError) ⇒ status "error", lỗi đọc được qua queueLoadError', async () => {
    const { openGlossaryQueue, queueStatus, queueLoadError } = await freshState()
    const err = { code: 'store.read_failed', message_key: 'err.store.read_failed', params: {}, retryable: false }
    pendingMock.mockResolvedValue({ candidates: null, error: err })

    await openGlossaryQueue()

    expect(queueStatus.value).toBe('error')
    expect(queueLoadError.value).toEqual(err)
  })
})

describe('Nhận — hai nhánh CÓ/KHÔNG đề xuất, con trỏ TỰ TIẾN', () => {
  it('han_viet_status === "ok" ⇒ approve(id, han_viet_suggestion, category)', async () => {
    const { openGlossaryQueue, acceptGlossaryQueueCandidate, queueRows } = await freshState()
    const row = candidate({ id: 1, han_viet_status: 'ok', han_viet_suggestion: 'Bắc Lương' })
    pendingMock.mockResolvedValue({ candidates: [row], error: null })
    approveMock.mockResolvedValue({ value: 99, error: null })

    await openGlossaryQueue()
    await acceptGlossaryQueueCandidate()

    expect(approveMock).toHaveBeenCalledWith(1, 'Bắc Lương', 'other')
    expect(queueRows.value[0]?.outcome).toBe('accepted')
  })

  it.each(['not_chinese', 'no_reading', 'dict_unavailable', 'not_requested'] as const)(
    'han_viet_status === "%s" (KHÔNG đề xuất) ⇒ approve(id, null, category)',
    async (statusValue) => {
      const { openGlossaryQueue, acceptGlossaryQueueCandidate } = await freshState()
      const row = candidate({ id: 2, han_viet_status: statusValue, han_viet_suggestion: null })
      pendingMock.mockResolvedValue({ candidates: [row], error: null })
      approveMock.mockResolvedValue({ value: 100, error: null })

      await openGlossaryQueue()
      await acceptGlossaryQueueCandidate()

      expect(approveMock).toHaveBeenCalledWith(2, null, 'other')
    },
  )

  it('con trỏ tự tiến tới hàng CHƯA XỬ LÝ kế tiếp, bỏ qua hàng đã xử lý', async () => {
    const { openGlossaryQueue, acceptGlossaryQueueCandidate, queueCursor } = await freshState()
    const rows = [candidate({ id: 1 }), candidate({ id: 2 }), candidate({ id: 3 })]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })
    approveMock.mockResolvedValue({ value: 1, error: null })

    await openGlossaryQueue()
    await acceptGlossaryQueueCandidate() // hàng 0 -> con trỏ sang 1
    expect(queueCursor.value).toBe(1)
    await acceptGlossaryQueueCandidate() // hàng 1 -> con trỏ sang 2
    expect(queueCursor.value).toBe(2)
  })

  it('lỗi ⇒ hàng KHÔNG đổi, con trỏ KHÔNG tiến, lỗi hiện qua queueActionError', async () => {
    const { openGlossaryQueue, acceptGlossaryQueueCandidate, queueCursor, queueRows, queueActionError } =
      await freshState()
    const rows = [candidate({ id: 1 }), candidate({ id: 2 })]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })
    const err = { code: 'store.write_failed', message_key: 'err.store.write_failed', params: {}, retryable: false }
    approveMock.mockResolvedValue({ value: null, error: err })

    await openGlossaryQueue()
    await acceptGlossaryQueueCandidate()

    expect(queueCursor.value).toBe(0)
    expect(queueRows.value[0]?.outcome).toBeNull()
    expect(queueActionError.value).toEqual(err)
  })

  it('🔴 clicking Accept a SECOND time on an already-decided row -> local error, no second IPC call', async () => {
    // Counter-check: removing `if (row.outcome !== null) { actionError.value =
    // ALREADY_DECIDED_ERROR; return }` turns this red (approveMock called twice, wrong error).
    const { openGlossaryQueue, acceptGlossaryQueueCandidate, queueRows, queueActionError } = await freshState()
    const rows = [candidate({ id: 1 })]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })
    approveMock.mockResolvedValue({ value: 1, error: null })

    await openGlossaryQueue()
    await acceptGlossaryQueueCandidate() // hàng DUY NHẤT -> 'accepted'; 0 hàng chưa xử lý còn lại, con trỏ đứng yên ở 0.
    expect(queueRows.value[0]?.outcome).toBe('accepted')
    expect(approveMock).toHaveBeenCalledTimes(1)

    await acceptGlossaryQueueCandidate() // bấm Nhận LẦN HAI, đúng hàng đã quyết.

    expect(queueActionError.value?.code).toBe('glossary.queue_already_decided')
    expect(queueActionError.value?.message_key).toBe('err.glossary.queue_already_decided')
    expect(approveMock).toHaveBeenCalledTimes(1) // KHÔNG một lượt ghi thứ hai nào phát ra.
    expect(queueRows.value[0]?.outcome).toBe('accepted') // hàng KHÔNG đổi.
  })
})

describe('Bỏ — hàng lùi màu + "rejected", con trỏ tự tiến, 0 tham số translation/category', () => {
  it('rejectGlossaryCandidate(id) — không translation, không category', async () => {
    const { openGlossaryQueue, rejectGlossaryQueueCandidate, queueRows, queueCursor } = await freshState()
    const rows = [candidate({ id: 5 }), candidate({ id: 6 })]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })
    rejectMock.mockResolvedValue({ value: true, error: null })

    await openGlossaryQueue()
    await rejectGlossaryQueueCandidate()

    expect(rejectMock).toHaveBeenCalledWith(5)
    expect(queueRows.value[0]?.outcome).toBe('rejected')
    expect(queueCursor.value).toBe(1)
  })

  it('lỗi ⇒ hàng KHÔNG đổi, con trỏ KHÔNG tiến', async () => {
    const { openGlossaryQueue, rejectGlossaryQueueCandidate, queueCursor, queueRows, queueActionError } =
      await freshState()
    const rows = [candidate({ id: 7 })]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })
    const err = { code: 'store.write_failed', message_key: 'err.store.write_failed', params: {}, retryable: false }
    rejectMock.mockResolvedValue({ value: null, error: err })

    await openGlossaryQueue()
    await rejectGlossaryQueueCandidate()

    expect(queueCursor.value).toBe(0)
    expect(queueRows.value[0]?.outcome).toBeNull()
    expect(queueActionError.value).toEqual(err)
  })

  it('🔴 clicking Reject a SECOND time on an already-decided row -> local error, no second IPC call', async () => {
    const { openGlossaryQueue, rejectGlossaryQueueCandidate, queueRows, queueActionError } = await freshState()
    const rows = [candidate({ id: 7 })]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })
    rejectMock.mockResolvedValue({ value: true, error: null })

    await openGlossaryQueue()
    await rejectGlossaryQueueCandidate()
    expect(queueRows.value[0]?.outcome).toBe('rejected')
    expect(rejectMock).toHaveBeenCalledTimes(1)

    await rejectGlossaryQueueCandidate() // bấm Bỏ LẦN HAI, đúng hàng đã quyết.

    expect(queueActionError.value?.code).toBe('glossary.queue_already_decided')
    expect(rejectMock).toHaveBeenCalledTimes(1) // KHÔNG một lượt ghi thứ hai nào phát ra.
    expect(queueRows.value[0]?.outcome).toBe('rejected') // hàng KHÔNG đổi.
  })
})

describe('setGlossaryQueueCategory — phím 1-4, CỤC BỘ, 0 lượt ghi cho tới khi Nhận', () => {
  it('đổi category của hàng ĐANG CHỌN mà KHÔNG gọi approve/reject', async () => {
    const { openGlossaryQueue, setGlossaryQueueCategory, queueRows } = await freshState()
    const rows = [candidate({ id: 1 })]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })

    await openGlossaryQueue()

    const next: GlossaryCategory = 'person'
    setGlossaryQueueCategory(next)

    expect(queueRows.value[0]?.category).toBe('person')
    expect(approveMock).not.toHaveBeenCalled()
    expect(rejectMock).not.toHaveBeenCalled()
  })

  it('hàng đã xử lý ⇒ đổi category không có tác dụng', async () => {
    const {
      openGlossaryQueue,
      acceptGlossaryQueueCandidate,
      setGlossaryQueueCategory,
      prevGlossaryQueueCandidate,
      queueRows,
    } = await freshState()
    const rows = [candidate({ id: 1 }), candidate({ id: 2 })]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })
    approveMock.mockResolvedValue({ value: 1, error: null })

    await openGlossaryQueue()
    await acceptGlossaryQueueCandidate() // hàng 0 đã xử lý, con trỏ sang 1
    prevGlossaryQueueCandidate() // quay lại hàng 0 (đã xử lý)

    setGlossaryQueueCategory('place')
    expect(queueRows.value[0]?.category).toBe('other') // KHÔNG đổi.
  })
})

describe('nextGlossaryQueueCandidate/prevGlossaryQueueCandidate — di chuyển con trỏ, không vòng', () => {
  it('next dừng ở hàng cuối, prev dừng ở hàng đầu', async () => {
    const { openGlossaryQueue, nextGlossaryQueueCandidate, prevGlossaryQueueCandidate, queueCursor } =
      await freshState()
    const rows = [candidate({ id: 1 }), candidate({ id: 2 })]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })

    await openGlossaryQueue()

    expect(queueCursor.value).toBe(0)
    prevGlossaryQueueCandidate()
    expect(queueCursor.value).toBe(0)

    nextGlossaryQueueCandidate()
    expect(queueCursor.value).toBe(1)
    nextGlossaryQueueCandidate()
    expect(queueCursor.value).toBe(1)
  })
})

describe('closeGlossaryQueue — KHÔNG dọn state', () => {
  it('đóng rồi state (hàng, con trỏ) vẫn còn nguyên trong bộ nhớ', async () => {
    const { openGlossaryQueue, closeGlossaryQueue, queueOverlayIsOpen, queueRows } = await freshState()
    const rows = [candidate({ id: 1 }), candidate({ id: 2 })]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })

    await openGlossaryQueue()
    closeGlossaryQueue()

    expect(queueOverlayIsOpen.value).toBe(false)
    expect(queueRows.value.length).toBe(2)
  })
})

describe('AC — đóng rồi mở lại: k mục đã quyết không còn, con trỏ ở tần suất cao nhất', () => {
  it('lượt mở lại gọi FỚI danh sách backend (đã bớt k hàng), không phải một phép lọc phía client', async () => {
    const { openGlossaryQueue, closeGlossaryQueue, acceptGlossaryQueueCandidate, queueRows, queueCursor } =
      await freshState()
    const first = [candidate({ id: 1, occurrence_count: 10 }), candidate({ id: 2, occurrence_count: 5 })]
    pendingMock.mockResolvedValue({ candidates: first, error: null })
    approveMock.mockResolvedValue({ value: 1, error: null })

    await openGlossaryQueue()
    await acceptGlossaryQueueCandidate() // quyết hàng id=1.
    closeGlossaryQueue()

    // Mô phỏng Rust: hàng id=1 nay có resolution, KHÔNG còn trong pending_candidates.
    const second = [candidate({ id: 2, occurrence_count: 5 })]
    pendingMock.mockResolvedValue({ candidates: second, error: null })

    await openGlossaryQueue()

    expect(queueRows.value.map((r) => r.candidate.id)).toEqual([2])
    expect(queueCursor.value).toBe(0)
  })
})

describe('resetGlossaryQueue — cổng check:panel-refs', () => {
  it('vứt toàn bộ state', async () => {
    const { openGlossaryQueue, resetGlossaryQueue, queueOverlayIsOpen, queueRows, queueStatus } = await freshState()
    const rows = [candidate({ id: 1 })]
    pendingMock.mockResolvedValue({ candidates: rows, error: null })

    await openGlossaryQueue()
    resetGlossaryQueue()

    expect(queueOverlayIsOpen.value).toBe(false)
    expect(queueRows.value).toEqual([])
    expect(queueStatus.value).toBe('unknown')
  })
})

// ═════════════════════════════════════════════════════════════════════════════════
// Vòng rà ba lớp (P2) — vế MOUNT: `onKeydown`/`trapTab`/focus-return/dấu ✓/✕ chưa từng
// chạy qua một component thật trước lượt vá này. Khuôn chép: `glossarySettings.test.ts`
// (mount thật + vùng chọn DOM thật) — cùng file, không tách riêng, đúng tiền lệ đó.
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Cài BỘ COMMAND THẬT (`installCommands`, không một bản giả) với `deps` là các mock cho
 * SÁU handler của lớp phủ — dispatch một hợp âm/phím phải đi qua ĐÚNG đường sản phẩm
 * (`GlossaryQueueOverlay.vue` → `dispatch('<id>')` → `CommandRegistry` thật → `run()` của
 * chính `commands/index.ts` → gọi `deps.xxx()`), không một đường tắt tự dựng trong test.
 *
 * ⚠️ Các dep KHÔNG liên quan (mode.*, editor.*, …) cố ý ĐỂ TRỐNG — `registerAll` đăng ký
 * được toàn bộ 66 lệnh mà không cần deps có mặt (chỉ `run()` mới đọc `deps[port]`, và đọc
 * LÚC CHẠY, không lúc đăng ký) — cùng cơ chế mà `check:commands` Kiểm E đã dùng.
 */
async function freshOverlay(deps: Partial<CommandDeps> = {}) {
  vi.resetModules()
  pendingMock.mockReset()
  lookupMock.mockReset()
  approveMock.mockReset()
  rejectMock.mockReset()

  const commands = await import('../../src/commands')
  commands.installCommands(deps as CommandDeps)

  const state = await import('../../src/glossaryQueueState')
  const i18n = await import('../../src/i18n')
  const selectionContract = await import('../../src/panels/selectionContract')
  const GlossaryQueueOverlay = (await import('../../src/GlossaryQueueOverlay.vue')).default
  return { commands, state, i18n, selectionContract, GlossaryQueueOverlay }
}

describe('GlossaryQueueOverlay.vue — bề mặt selection thật của modal', () => {
  it('vùng chọn chữ thật trong modal vai display không trở thành nguồn Auto-Lookup', async () => {
    const { state, selectionContract, GlossaryQueueOverlay } = await freshOverlay()
    pendingMock.mockResolvedValue({ candidates: [], error: null })
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: false }) // ⇒ status 'no_work'.
    await state.openGlossaryQueue()

    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const text = wrapper.get('.gq-empty').element.firstChild
    if (!(text instanceof Text) || text.data.length === 0) {
      wrapper.unmount()
      throw new Error('modal thật phải render một text node để dựng vùng chọn')
    }
    const range = document.createRange()
    range.setStart(text, 0)
    range.setEnd(text, Math.min(8, text.data.length))
    const selection = window.getSelection()
    selection?.removeAllRanges()
    selection?.addRange(range)
    const selectedText = selection?.toString() ?? ''

    expect(selectedText.length).toBeGreaterThan(0)
    expect(selectionContract.currentSelectionText()).toBe('')
    expect(selectionContract.currentSelectionTextForGlossaryQuickAdd()).toBe(selectedText)

    wrapper.unmount()
    selection?.removeAllRanges()
  })
})

describe('GlossaryQueueOverlay.vue — bàn phím cục bộ dispatch ĐÚNG lệnh qua registry THẬT', () => {
  it('n/b/mũi tên bắn đúng bốn lệnh glossary.queue.*', async () => {
    const acceptMock = vi.fn()
    const rejectHandlerMock = vi.fn()
    const nextMock = vi.fn()
    const prevMock = vi.fn()
    const { state, GlossaryQueueOverlay } = await freshOverlay({
      acceptGlossaryQueueCandidate: acceptMock,
      rejectGlossaryQueueCandidate: rejectHandlerMock,
      nextGlossaryQueueCandidate: nextMock,
      prevGlossaryQueueCandidate: prevMock,
    })
    pendingMock.mockResolvedValue({ candidates: [candidate({ id: 1 })], error: null })
    await state.openGlossaryQueue()

    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.gq-scrim')

    await scrim.trigger('keydown', { key: 'n' })
    expect(acceptMock).toHaveBeenCalledTimes(1)

    await scrim.trigger('keydown', { key: 'b' })
    expect(rejectHandlerMock).toHaveBeenCalledTimes(1)

    await scrim.trigger('keydown', { key: 'ArrowDown' })
    expect(nextMock).toHaveBeenCalledTimes(1)

    await scrim.trigger('keydown', { key: 'ArrowUp' })
    expect(prevMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
  })

  it('phím số (không bổ trợ) đổi phân loại của hàng đang chọn, KHÔNG một lệnh nào bắn', async () => {
    const acceptMock = vi.fn()
    const { state, GlossaryQueueOverlay } = await freshOverlay({
      acceptGlossaryQueueCandidate: acceptMock,
    })
    pendingMock.mockResolvedValue({ candidates: [candidate({ id: 1 })], error: null })
    await state.openGlossaryQueue()

    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    await wrapper.get('.gq-scrim').trigger('keydown', { key: '1' })

    expect(state.queueRows.value[0]?.category).toBe('person')
    expect(acceptMock).not.toHaveBeenCalled()

    wrapper.unmount()
  })

  it('⚠️ P1/P9 — hợp âm CÓ bổ trợ (⌘N / ⌘1) KHÔNG kích hoạt gì trong lớp phủ', async () => {
    const acceptMock = vi.fn()
    const { state, GlossaryQueueOverlay } = await freshOverlay({
      acceptGlossaryQueueCandidate: acceptMock,
    })
    pendingMock.mockResolvedValue({ candidates: [candidate({ id: 1 })], error: null })
    await state.openGlossaryQueue()

    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.gq-scrim')

    // ⌘N — KHÔNG được Nhận một ứng viên chỉ vì người dùng gõ một hợp âm quen tay khác
    // (AD-20/FR55: máy đề xuất, NGƯỜI bấm).
    await scrim.trigger('keydown', { key: 'n', metaKey: true })
    expect(acceptMock).not.toHaveBeenCalled()

    // ⌘1 — KHÔNG được đổi phân loại (đường DOM cục bộ, xem doc-comment `onKeydown`).
    await scrim.trigger('keydown', { key: '1', metaKey: true })
    expect(state.queueRows.value[0]?.category).toBe('other')

    // Đối chứng: cùng phím, KHÔNG bổ trợ, VẪN hoạt động bình thường — chứng minh lượt lọc
    // ở trên chặn đúng ĐIỀU CẦN CHẶN, không chặn oan cả nhánh.
    await scrim.trigger('keydown', { key: 'n' })
    expect(acceptMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
  })
})

describe('GlossaryQueueOverlay.vue — dấu ✓/✕ mang tín hiệu ĐỌC ĐƯỢC, không chỉ màu', () => {
  it('hàng đã Nhận render span sr-only "Đã nhận." ngoài dấu ✓ (aria-hidden)', async () => {
    // 🔴 SỬA (cụm D vá) — HAI hàng, không MỘT: sau khi `queueEmptyReasonFor` đo "đã duyệt
    // hết" bằng SỐ HÀNG CHƯA XỬ LÝ (không còn `rows.length === 0`, mã chết cho mục đích đó),
    // Nhận hàng DUY NHẤT của một bảng chờ một-hàng làm `unprocessedCount` về 0 ⇒ nhánh
    // "bảng chờ đã sạch" SỐNG (đúng mục đích của bản vá, §I/O Matrix ⑩) và danh sách (cùng
    // dấu ✓/✕ của nó) không còn render. Ca này giữ MỘT hàng CÒN chờ để danh sách vẫn render.
    const { state, i18n, GlossaryQueueOverlay } = await freshOverlay()
    pendingMock.mockResolvedValue({
      candidates: [candidate({ id: 1 }), candidate({ id: 2, source_term: '乙' })],
      error: null,
    })
    approveMock.mockResolvedValue({ value: 1, error: null })
    await state.openGlossaryQueue()
    // Đi qua đường Nhận THẬT (cùng adapter giả `config/glossary.ts` mà mọi ca khác của tệp
    // này dùng) — không ép state nội bộ, giữ mount test này là một lượt tích hợp thật.
    await state.acceptGlossaryQueueCandidate()
    expect(state.queueRows.value[0]?.outcome).toBe('accepted')
    expect(state.queueRows.value[1]?.outcome).toBeNull() // hàng thứ hai còn chờ ⇒ danh sách còn sống.

    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const mark = wrapper.get('.gq-mark')
    expect(mark.attributes('aria-hidden')).toBe('true')
    expect(wrapper.text()).toContain(i18n.t('glossary.queue.row_status_accepted'))

    wrapper.unmount()
  })
})

describe('GlossaryQueueOverlay.vue — số Chương khác nhau render trên mỗi hàng', () => {
  it('`.gq-chapter-span` hiện đúng `chapter_span_count` của hàng, đã nội suy vào câu `vi.json`', async () => {
    const { state, i18n, GlossaryQueueOverlay } = await freshOverlay()
    pendingMock.mockResolvedValue({
      candidates: [candidate({ id: 1, chapter_span_count: 3 })],
      error: null,
    })
    await state.openGlossaryQueue()

    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.get('.gq-chapter-span').text()).toBe(
      i18n.t('glossary.queue.chapter_span_count', { count: '3' }),
    )

    wrapper.unmount()
  })

  it('`chapter_span_count: 0` vẫn hiện được — `0` là một câu trả lời thật, không phải "không biết"', async () => {
    const { state, i18n, GlossaryQueueOverlay } = await freshOverlay()
    pendingMock.mockResolvedValue({
      candidates: [candidate({ id: 1, chapter_span_count: 0 })],
      error: null,
    })
    await state.openGlossaryQueue()

    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.get('.gq-chapter-span').text()).toBe(
      i18n.t('glossary.queue.chapter_span_count', { count: '0' }),
    )

    wrapper.unmount()
  })
})

describe('GlossaryQueueOverlay.vue — chọn hàng bằng CHUỘT đi qua ĐÚNG `setGlossaryQueueCursor`', () => {
  it('bấm radio ẩn của hàng thứ hai đổi `queueCursor` — cùng hàm bàn phím dùng, không một cơ chế song song', async () => {
    // Counter-check: turning `cursorModel`'s setter into a no-op in GlossaryQueueOverlay.vue
    // turns this red (`queueCursor.value` stays at 0).
    const { state, GlossaryQueueOverlay } = await freshOverlay()
    pendingMock.mockResolvedValue({
      candidates: [candidate({ id: 1, source_term: '甲' }), candidate({ id: 2, source_term: '乙' })],
      error: null,
    })
    await state.openGlossaryQueue()
    expect(state.queueCursor.value).toBe(0)

    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const rowRadios = wrapper.findAll<HTMLInputElement>('input[name="gq-row-cursor"]')
    expect(rowRadios).toHaveLength(2)
    await rowRadios[1]?.setValue(true)

    expect(state.queueCursor.value).toBe(1)
    expect(rowRadios[1]?.element.checked).toBe(true)
    expect(rowRadios[0]?.element.checked).toBe(false)

    wrapper.unmount()
  })
})

describe('GlossaryQueueOverlay.vue — chip phân loại là radio THẬT', () => {
  it('bấm chip "Người" (radio ẩn) đổi phân loại của hàng ĐANG CHỌN — cùng hàm phím số dùng', async () => {
    // Counter-check: removing `v-model="currentCategoryModel"` from the radio (chip writes no
    // state) turns this red (`category` stays 'other').
    const { state, GlossaryQueueOverlay } = await freshOverlay()
    pendingMock.mockResolvedValue({ candidates: [candidate({ id: 1 })], error: null })
    await state.openGlossaryQueue()
    expect(state.queueRows.value[0]?.category).toBe('other')

    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const personRadio = wrapper.get<HTMLInputElement>('input[name="gq-category"][value="person"]')
    await personRadio.setValue(true)

    expect(state.queueRows.value[0]?.category).toBe('person')
    expect(personRadio.element.checked).toBe(true)
    expect(personRadio.element.closest('label.gq-chip')?.classList.contains('on')).toBe(true)

    wrapper.unmount()
  })
})

describe('queueEmptyReasonFor — cụm D vá: "loading" có tên riêng, "all_reviewed" đo bằng số hàng CHƯA xử lý', () => {
  it('"unknown" trả "loading" (KHÔNG null) — vị từ tự khai, template không cần một v-if riêng canh cùng mệnh đề', async () => {
    const { queueEmptyReasonFor } = await freshState()
    expect(queueEmptyReasonFor('unknown', 0)).toBe('loading')
  })

  it('🔴 ĐỐI CHỨNG mã-chết-cũ: "loaded" với rows.length > 0 nhưng MỌI hàng đã xử lý ⇒ "all_reviewed" (bản cũ đo bằng rowCount===0 không bao giờ đúng ở đây)', async () => {
    const { openGlossaryQueue, acceptGlossaryQueueCandidate, queueEmptyReasonFor, queueStatus, queueUnprocessedCount } =
      await freshState()
    pendingMock.mockResolvedValue({ candidates: [candidate({ id: 1 })], error: null })
    approveMock.mockResolvedValue({ value: 1, error: null })
    await openGlossaryQueue()
    await acceptGlossaryQueueCandidate()

    expect(queueUnprocessedCount.value).toBe(0)
    expect(queueEmptyReasonFor(queueStatus.value, queueUnprocessedCount.value)).toBe('all_reviewed')
  })

  it('"loaded" còn hàng CHƯA xử lý ⇒ null (danh sách vẫn hiện, không phải câu rỗng nào)', async () => {
    const { openGlossaryQueue, acceptGlossaryQueueCandidate, queueEmptyReasonFor, queueStatus, queueUnprocessedCount } =
      await freshState()
    pendingMock.mockResolvedValue({ candidates: [candidate({ id: 1 }), candidate({ id: 2 })], error: null })
    approveMock.mockResolvedValue({ value: 1, error: null })
    await openGlossaryQueue()
    await acceptGlossaryQueueCandidate() // chỉ hàng ĐẦU được xử lý.

    expect(queueUnprocessedCount.value).toBe(1)
    expect(queueEmptyReasonFor(queueStatus.value, queueUnprocessedCount.value)).toBeNull()
  })

  it('GlossaryQueueOverlay.vue — Nhận hết mọi hàng ⇒ DOM chuyển sang câu "đã sạch", danh sách biến mất', async () => {
    const { state, i18n, GlossaryQueueOverlay } = await freshOverlay()
    pendingMock.mockResolvedValue({ candidates: [candidate({ id: 1 })], error: null })
    approveMock.mockResolvedValue({ value: 1, error: null })
    await state.openGlossaryQueue()
    await state.acceptGlossaryQueueCandidate()

    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.gq-list').exists()).toBe(false)
    expect(wrapper.text()).toContain(i18n.t('glossary.queue.empty_all_reviewed'))

    wrapper.unmount()
  })

  it('🔴 #13 (vòng rà thứ hai) — GlossaryQueueOverlay.vue gọi queueEmptyReasonFor(...) ĐÚNG MỘT LẦN mỗi lượt render, không bốn', async () => {
    // Đối chứng gỡ-chỗ-nối: rải lại bốn lời gọi trực tiếp `queueEmptyReasonFor(queueStatus,
    // queueUnprocessedCount)` vào bốn nhánh `v-if`/`v-else-if` của template (bỏ `computed`
    // `queueEmptyReason`) ⇒ ca này ĐỎ (spy đếm được 4 lượt gọi thay vì 1).
    const { state, GlossaryQueueOverlay } = await freshOverlay()
    pendingMock.mockResolvedValue({ candidates: [], error: null })
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: false }) // ⇒ status 'no_work'.
    await state.openGlossaryQueue()

    const spy = vi.spyOn(state, 'queueEmptyReasonFor')

    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(spy).toHaveBeenCalledTimes(1)

    wrapper.unmount()
    spy.mockRestore()
  })
})

describe('review_harvest — hàng bảng chờ và phím N', () => {
  const harvest = (over: Partial<GlossaryCandidate> = {}) =>
    candidate({
      id: 7,
      source_term: '北凉王',
      candidate_origin: 'review_harvest',
      han_viet_status: 'ok',
      han_viet_suggestion: 'Bắc Lương vương',
      replaced_translation: 'Bắc Lương vương',
      proposed_translation: 'vương Bắc Lương',
      changed_count: 3,
      seen_count: 5,
      ...over,
    })

  it('phím N gửi proposed_translation, không gợi ý Hán Việt', async () => {
    const state = await freshState()
    pendingMock.mockResolvedValue({ candidates: [harvest()], error: null })
    approveMock.mockResolvedValue({ value: 1, error: null })
    await state.openGlossaryQueue()
    await state.acceptGlossaryQueueCandidate()
    expect(approveMock).toHaveBeenCalledWith(7, 'vương Bắc Lương', 'other')
  })

  it.each([
    [2, 5, true],
    [3, 6, true],
    [3, 5, false],
    [4, 4, false],
  ])('isInconsistentHarvest(%i/%i) = %s', async (changed, seen, expected) => {
    const state = await freshState()
    expect(state.isInconsistentHarvest(harvest({ changed_count: changed, seen_count: seen }))).toBe(expected)
    expect(
      state.isInconsistentHarvest(
        harvest({ candidate_origin: 'import_scan', changed_count: changed, seen_count: seen }),
      ),
    ).toBe(false)
  })

  it('hàng hiện X → Y, N/M, và nhãn không nhất quán chỉ khi N/M <= 50%', async () => {
    const { state, GlossaryQueueOverlay } = await freshOverlay()
    pendingMock.mockResolvedValue({
      candidates: [harvest({ id: 1, changed_count: 3, seen_count: 5 }), harvest({ id: 2, changed_count: 2, seen_count: 5 })],
      error: null,
    })
    lookupMock.mockResolvedValue(workOpenLookup)
    await state.openGlossaryQueue()
    const wrapper = mount(GlossaryQueueOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const rows = wrapper.findAll('.gq-row')
    expect(rows).toHaveLength(2)
    expect(rows[0]?.get('.gq-harvest').text()).toBe('Bắc Lương vương → vương Bắc Lương · đổi ở 3/5 lần')
    expect(rows[0]?.find('.gq-inconsistent').exists()).toBe(false)
    expect(rows[1]?.get('.gq-harvest').text()).toContain('2/5')
    expect(rows[1]?.find('.gq-inconsistent').exists()).toBe(true)
    wrapper.unmount()
  })
})
