/**
 * State + lớp phủ **Xem trước lượt nhập Glossary** — Story 3.10b, AD-48.
 *
 * ⚠️ Cùng khuôn `glossaryManage.test.ts`: `config/glossary.ts` là biên IPC, giả lập bằng
 * `vi.mock`, không gọi `@tauri-apps/api` thật.
 *
 * ⚠️ **Thứ tự bắt buộc trong mỗi ca**: `freshState()` TRƯỚC, cấu hình `mockResolvedValue`
 * SAU — `freshState()` tự `mockReset()` mọi mock.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { GlossaryImportPreview } from '../../src/config/glossary'

const openPreviewMock = vi.fn()
const confirmMock = vi.fn()
const cancelMock = vi.fn()

vi.mock('../../src/config/glossary', () => ({
  openGlossaryImportPreview: (tier: string) => openPreviewMock(tier),
  confirmGlossaryImport: (decisions: Record<string, string>) => confirmMock(decisions),
  cancelGlossaryImport: () => cancelMock(),
}))

/** Nạp lại module mỗi ca — state của lớp phủ là module-level singleton. */
async function freshState() {
  vi.resetModules()
  openPreviewMock.mockReset()
  confirmMock.mockReset()
  cancelMock.mockReset()
  return import('../../src/glossaryImportState')
}

/** Cùng `freshState()`, kèm module cổng dùng chung — cùng thể hiện (`vi.resetModules()` một
 * lần, cả hai import ngay sau nó) mà `glossaryImportState.ts` bên trong đang cầm. */
async function freshStateWithGate() {
  vi.resetModules()
  openPreviewMock.mockReset()
  confirmMock.mockReset()
  cancelMock.mockReset()
  const state = await import('../../src/glossaryImportState')
  const gate = await import('../../src/glossaryExchangeGate')
  return { state, gate }
}

function preview(over: Partial<GlossaryImportPreview> = {}): GlossaryImportPreview {
  return {
    file_name: 'glossary.csv',
    tier: 'global',
    row_count: 3,
    recognized_column_count: 2,
    ignored_columns: [],
    term_origin_column_present: false,
    new_count: 1,
    identical_count: 0,
    conflicts: [{ source_term: '慕容', existing_translation: 'Mộ Dung', file_translation: 'Mo Dung' }],
    ...over,
  }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('glossaryImportState — nhịp một, quyết định, dọn lô', () => {
  it('mở thành công nạp preview và mở lớp phủ; mặc định MỌI hàng bất đồng là giữ của tôi', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })

    await state.openGlossaryImportPreviewOverlay('global')

    expect(state.importOverlayIsOpen.value).toBe(true)
    expect(state.importStatus.value).toBe('loaded')
    expect(state.importPreview.value?.conflicts).toHaveLength(1)
    expect(state.importDecisionFor('慕容')).toBe('keep_mine')
  })

  it("huỷ hộp thoại chọn tệp (outcome: 'cancelled') KHÔNG mở lớp phủ — im lặng", async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'cancelled' })

    await state.openGlossaryImportPreviewOverlay('global')

    expect(state.importOverlayIsOpen.value).toBe(false)
    expect(state.importStatus.value).toBe('unknown')
  })

  it('🔴 P2 (vòng rà ba lớp 2026-08-25) — không có cầu IPC MỞ lớp phủ và NÓI RA, KHÁC hẳn huỷ', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'ipc_unavailable' })

    await state.openGlossaryImportPreviewOverlay('global')

    // Đối chứng với ca "huỷ hộp thoại" ngay trên: outcome khác nhau ⇒ hành vi khác nhau.
    // Gộp lại (đọc `ipc_unavailable` như `cancelled`) sẽ làm ca này ĐỎ vì overlay không mở.
    expect(state.importOverlayIsOpen.value).toBe(true)
    expect(state.importStatus.value).toBe('ipc_unavailable')
    expect(state.importLoadError.value).toBeNull()
  })

  it('lượt mở trượt THẬT (outcome error) mở lớp phủ và hiện IpcError, KHÁC "ipc_unavailable"', async () => {
    const state = await freshState()
    const err = { code: 'x', message_key: 'err.unknown', params: {}, retryable: false }
    openPreviewMock.mockResolvedValue({ outcome: 'error', error: err })

    await state.openGlossaryImportPreviewOverlay('global')

    expect(state.importOverlayIsOpen.value).toBe(true)
    expect(state.importStatus.value).toBe('error')
    expect(state.importLoadError.value).toEqual(err)
  })

  it('setImportDecision đổi một hàng sang lấy của file, rồi đổi lại giữ của tôi thì XOÁ khoá', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openGlossaryImportPreviewOverlay('global')

    state.setImportDecision('慕容', 'take_theirs')
    expect(state.importDecisionFor('慕容')).toBe('take_theirs')

    state.setImportDecision('慕容', 'keep_mine')
    expect(state.importDecisionFor('慕容')).toBe('keep_mine')
  })

  it('importConfirmSummary đếm mới + lấy-của-file làm "mới", còn lại làm "giữ"', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({
      outcome: 'loaded',
      preview: preview({
        new_count: 2,
        conflicts: [
          { source_term: 'a', existing_translation: 'A', file_translation: 'A2' },
          { source_term: 'b', existing_translation: 'B', file_translation: 'B2' },
        ],
      }),
    })
    await state.openGlossaryImportPreviewOverlay('global')

    expect(state.importConfirmSummary.value).toEqual({ newCount: 2, keepCount: 2 })

    state.setImportDecision('a', 'take_theirs')
    expect(state.importConfirmSummary.value).toEqual({ newCount: 3, keepCount: 1 })
  })

  it('§Always — BỐN nhánh lý-do-rỗng phân biệt được', async () => {
    const state = await freshState()

    expect(state.importEmptyReasonFor('unknown', 0, 0)).toBe('not_loaded')
    expect(state.importEmptyReasonFor('ipc_unavailable', 0, 0)).toBe('ipc_unavailable')
    expect(state.importEmptyReasonFor('loaded', 0, 0)).toBe('file_has_no_rows')
    expect(state.importEmptyReasonFor('loaded', 5, 0)).toBe('no_conflicts')
    expect(state.importEmptyReasonFor('loaded', 5, 2)).toBeNull()
    expect(state.importEmptyReasonFor('error', 5, 0)).toBeNull()
  })

  it('xác nhận thành công đóng lớp phủ; xác nhận trượt GIỮ lớp phủ mở và hiện lỗi', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openGlossaryImportPreviewOverlay('global')

    confirmMock.mockResolvedValue({
      summary: null,
      error: { code: 'x', message_key: 'err.unknown', params: {}, retryable: false },
    })
    await state.confirmGlossaryImportPreview()
    expect(state.importOverlayIsOpen.value).toBe(true)
    expect(state.importConfirmError.value).not.toBeNull()

    confirmMock.mockResolvedValue({ summary: { inserted: 1, updated: 0, identical: 0 }, error: null })
    await state.confirmGlossaryImportPreview()
    expect(state.importOverlayIsOpen.value).toBe(false)
  })

  it('huỷ dọn lô ở Rust và đóng lớp phủ NGAY, không đợi round-trip', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openGlossaryImportPreviewOverlay('global')
    expect(state.importOverlayIsOpen.value).toBe(true)

    let resolveCancel: (() => void) | undefined
    cancelMock.mockReturnValue(
      new Promise((resolve) => {
        resolveCancel = () => resolve({ value: true, error: null })
      }),
    )

    const done = state.cancelGlossaryImportPreview()
    expect(state.importOverlayIsOpen.value).toBe(false) // Dong NGAY, truoc khi Rust tra ve.
    resolveCancel?.()
    await done
    expect(cancelMock).toHaveBeenCalledTimes(1)
  })

  it('P9 (vòng rà ba lớp 2026-08-25) — bấm chồng KHÔNG xếp hai hộp thoại: cửa `opening` chặn lượt gọi thứ hai', async () => {
    const state = await freshState()
    let resolveFirst: (() => void) | undefined
    openPreviewMock.mockReturnValue(
      new Promise((resolve) => {
        resolveFirst = () => resolve({ outcome: 'loaded', preview: preview() })
      }),
    )

    const first = state.openGlossaryImportPreviewOverlay('global')
    const second = state.openGlossaryImportPreviewOverlay('global') // Bam lan hai TRUOC khi lan dau xong.

    resolveFirst?.()
    await Promise.all([first, second])

    expect(openPreviewMock).toHaveBeenCalledTimes(1)
  })

  it('⑦ (cụm D vá) — mở lại xem-trước rồi HUỶ hộp thoại trong lúc một lượt xác nhận ĐANG BAY ⇒ `confirming` hạ về `false`', async () => {
    // Bản trước: `openGlossaryImportPreviewOverlay` chỉ hạ `confirming`/`confirmError` ở
    // nhánh KHÔNG `'cancelled'`. Một lượt `confirmGlossaryImportPreview` cũ (sequence CŨ)
    // đang bay khi lượt mở lại này chạy sẽ tự bỏ qua `confirming.value = false` của CHÍNH
    // NÓ (điều kiện `mySequence !== sequence` bên trong nó) — nên nếu lượt mở lại HUỶ hộp
    // thoại (return sớm TRƯỚC khi chạm `confirming`), cờ kẹt `true` mãi mãi, nút xác nhận
    // khoá vĩnh viễn dù dữ liệu chưa từng đổi.
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openGlossaryImportPreviewOverlay('global')

    let resolveConfirm: ((v: unknown) => void) | undefined
    confirmMock.mockReturnValue(
      new Promise((resolve) => {
        resolveConfirm = resolve
      }),
    )
    const confirming = state.confirmGlossaryImportPreview() // KHÔNG await -- đang bay.
    expect(state.importConfirming.value).toBe(true)

    // Mở lại (vd. bấm lại "Nhập CSV") TRONG LÚC lượt xác nhận ở trên còn bay, rồi HUỶ hộp
    // thoại chọn tệp của lượt mở lại đó.
    openPreviewMock.mockResolvedValue({ outcome: 'cancelled' })
    await state.openGlossaryImportPreviewOverlay('global')

    expect(state.importConfirming.value).toBe(false) // Mệnh đề trung tâm của ⑦.

    // Dọn lượt xác nhận cũ để không rò một promise treo sang ca sau.
    resolveConfirm?.({ summary: { inserted: 0, updated: 0, identical: 0 }, error: null })
    await confirming
  })

  it('⑧ (cụm D vá) — `cancelGlossaryImportPreview` ĐỌC `result.error` và ghi một chẩn đoán nêu đích danh, KHÔNG nuốt im lặng', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openGlossaryImportPreviewOverlay('global')

    const err = { code: 'x', message_key: 'err.unknown', params: {}, retryable: false }
    cancelMock.mockResolvedValue({ value: null, error: err })
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    await state.cancelGlossaryImportPreview()

    expect(state.importOverlayIsOpen.value).toBe(false) // Lớp phủ vẫn đóng NGAY (không đổi).
    expect(errorSpy).toHaveBeenCalledTimes(1)
    expect(String(errorSpy.mock.calls[0]?.[0])).toContain('glossary_cancel_import')

    errorSpy.mockRestore()
  })

  it('🔴 #12 (vòng rà thứ hai) — một lượt mở CŨ (bị `resetGlossaryImport()` vượt mặt trong lúc bay) KHÔNG được hạ nhầm cờ dùng chung của một thao tác MỚI đã bắt đầu', async () => {
    // Kịch bản: lượt mở A đang bay (`opening`/`glossaryExchangeBusy` đều `true`). Một
    // `resetGlossaryImport()` chạy giữa chừng (bump `sequence`, tự dọn cả hai cờ CỦA CHÍNH
    // NÓ). Trước khi A kịp `await` xong, một thao tác THẬT SỰ MỚI (ở đây mô phỏng bằng cách
    // đặt thẳng cờ dùng chung — đại diện cho `exportGlossaryManageTier` đã kịp giành cờ)
    // giành `glossaryExchangeBusy`. Đối chứng gỡ-chỗ-nối: đổi lại thứ tự (hạ cờ TRƯỚC khi
    // kiểm `mySequence !== sequence`, bản trước bản vá) ⇒ ca này ĐỎ (`glossaryExchangeBusy`
    // bị lượt A hạ nhầm về `false` dù thao tác MỚI vẫn đang thật sự chạy).
    const { state, gate } = await freshStateWithGate()

    let resolveOpen: ((v: unknown) => void) | undefined
    openPreviewMock.mockReturnValue(
      new Promise((resolve) => {
        resolveOpen = resolve
      }),
    )

    const opening = state.openGlossaryImportPreviewOverlay('global') // Lượt A — KHÔNG await.
    expect(state.importOpening.value).toBe(true)
    expect(gate.glossaryExchangeBusy.value).toBe(true)

    state.resetGlossaryImport() // Vượt mặt lượt A — bump sequence, tự dọn cờ CỦA CHÍNH NÓ.
    expect(state.importOpening.value).toBe(false)
    expect(gate.glossaryExchangeBusy.value).toBe(false)

    // Một thao tác MỚI THẬT SỰ giành cờ (đại diện cho một lượt Xuất mới bắt đầu).
    gate.setGlossaryExchangeBusy(true)

    resolveOpen?.({ outcome: 'cancelled' }) // Lượt A cũ về MUỘN.
    await opening

    // Mệnh đề trung tâm: cờ dùng chung VẪN `true` — lượt A (đã stale) không được chạm vào nó.
    expect(gate.glossaryExchangeBusy.value).toBe(true)

    gate.setGlossaryExchangeBusy(false) // dọn cho ca sau.
  })

  it('resetGlossaryImport vứt toàn bộ state', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({ outcome: 'loaded', preview: preview() })
    await state.openGlossaryImportPreviewOverlay('global')
    state.setImportDecision('慕容', 'take_theirs')

    state.resetGlossaryImport()

    expect(state.importOverlayIsOpen.value).toBe(false)
    expect(state.importStatus.value).toBe('unknown')
    expect(state.importPreview.value).toBeNull()
    expect(state.importDecisionFor('慕容')).toBe('keep_mine')
  })
})

describe('GlossaryImportOverlay.vue — vẽ từ mô hình, đổi quyết định qua radiogroup', () => {
  it('hiện tên tệp/số hàng/cột bỏ qua và một radiogroup mỗi hàng bất đồng', async () => {
    const state = await freshState()
    openPreviewMock.mockResolvedValue({
      outcome: 'loaded',
      preview: preview({ ignored_columns: ['usage_count'], term_origin_column_present: true }),
    })
    await state.openGlossaryImportPreviewOverlay('global')

    const GlossaryImportOverlay = (await import('../../src/GlossaryImportOverlay.vue')).default
    const wrapper = mount(GlossaryImportOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.gi-scrim').exists()).toBe(true)
    expect(wrapper.text()).toContain('glossary.csv')
    expect(wrapper.findAll('[role="radiogroup"]')).toHaveLength(1)

    const radios = wrapper.findAll('input[type="radio"]')
    expect(radios).toHaveLength(2)
    // Mac dinh: radio dau (giu cua toi) da checked.
    expect((radios[0]?.element as HTMLInputElement).checked).toBe(true)
    expect((radios[1]?.element as HTMLInputElement).checked).toBe(false)

    await radios[1]?.setValue(true)
    expect(state.importDecisionFor('慕容')).toBe('take_theirs')

    wrapper.unmount()
  })
})
