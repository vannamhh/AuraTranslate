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
