/**
 * State + lớp phủ **Quản lý Glossary** — Story 3.9, FR49.
 *
 * ⚠️ Cùng khuôn `glossaryQueue.test.ts`: `config/glossary.ts` là biên IPC, giả lập bằng
 * `vi.mock`, không gọi `@tauri-apps/api` thật. `panels/editorPanelState.ts` ·
 * `panels/sourcePanelState.ts` · `panels/glossaryMarksState.ts` cũng giả lập — tệp này
 * kiểm STATE của lớp phủ, không kiểm lại toàn bộ chuỗi mở Chương/gộp segment mà
 * `glossaryMarksRefresh.test.ts` đã đóng.
 *
 * ⚠️ **Thứ tự bắt buộc trong mỗi ca**: `freshState()` TRƯỚC, cấu hình `mockResolvedValue`
 * SAU — `freshState()` tự `mockReset()` mọi mock.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { ref } from 'vue'
import type { GlossaryEntry } from '../../src/config/glossary'
import type { CommandDeps } from '../../src/commands'

const listMock = vi.fn()
const deleteMock = vi.fn()
const promoteMock = vi.fn()
const updateMock = vi.fn()
const lookupMock = vi.fn()
const refreshMarksMock = vi.fn()
const exportMock = vi.fn()

const fakeChapterId = ref<number | null>(42)
const fakeSegments = ref<readonly { id: number }[]>([{ id: 1 }])
const fakeSourceChapter = ref<{ chapter_id: number; source_lang: string } | null>({
  chapter_id: 42,
  source_lang: 'zh',
})

vi.mock('../../src/config/glossary', () => ({
  listGlossaryEntries: () => listMock(),
  deleteGlossaryTerm: (tier: string, id: number) => deleteMock(tier, id),
  promoteGlossaryTermToGlobal: (id: number) => promoteMock(id),
  updateGlossaryTerm: (...args: unknown[]) => updateMock(...args),
  lookupGlossaryTerm: (term: string) => lookupMock(term),
  exportGlossaryTier: (tier: string) => exportMock(tier),
}))

vi.mock('../../src/panels/editorPanelState', () => ({
  editorChapterId: fakeChapterId,
  editorSegments: fakeSegments,
}))

vi.mock('../../src/panels/sourcePanelState', () => ({
  sourceChapter: fakeSourceChapter,
}))

vi.mock('../../src/panels/glossaryMarksState', () => ({
  refreshGlossaryMarks: (chapterId: number, segments: unknown, sourceLang: string) =>
    refreshMarksMock(chapterId, segments, sourceLang),
}))

/** Nạp lại module mỗi ca — state của lớp phủ là module-level singleton. */
async function freshState() {
  vi.resetModules()
  listMock.mockReset()
  deleteMock.mockReset()
  promoteMock.mockReset()
  updateMock.mockReset()
  lookupMock.mockReset()
  refreshMarksMock.mockReset()
  exportMock.mockReset()
  fakeChapterId.value = 42
  fakeSegments.value = [{ id: 1 }]
  fakeSourceChapter.value = { chapter_id: 42, source_lang: 'zh' }
  return import('../../src/glossaryManageState')
}

function entry(over: Partial<GlossaryEntry> = {}): GlossaryEntry {
  return {
    tier: 'global',
    id: 1,
    source_term: '青丘',
    translation: 'Thanh Khâu',
    note: '',
    category: 'place',
    term_origin: 'manual',
    created_at: '2026-08-24T00:00:00.000Z',
    is_shadowed: false,
    ...over,
  }
}

const workOpenProbe = { found: 'none' as const, workTierAvailable: true }
const workClosedProbe = { found: 'none' as const, workTierAvailable: false }

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('openGlossaryManage — nạp cả hai tầng, giữ NGUYÊN thứ tự backend trả', () => {
  it('trạng thái "loaded", rows giữ đúng thứ tự và giữ nguyên cờ is_shadowed', async () => {
    const { openGlossaryManage, manageFilteredRows, manageStatus, manageWorkTierAvailable } = await freshState()
    const winner = entry({ tier: 'work', id: 2, source_term: '慕容', is_shadowed: false })
    const shadowed = entry({ tier: 'global', id: 1, source_term: '慕容', is_shadowed: true })
    listMock.mockResolvedValue({ entries: [winner, shadowed], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)

    await openGlossaryManage()

    expect(manageStatus.value).toBe('loaded')
    expect(manageFilteredRows.value.map((r) => [r.tier, r.id, r.is_shadowed])).toEqual([
      ['work', 2, false],
      ['global', 1, true],
    ])
    expect(manageWorkTierAvailable.value).toBe(true)
  })

  it('chưa mở Tác phẩm nào ⇒ chỉ mục tầng Global, KHÔNG một ca rỗng riêng — chỉ cờ workTierAvailable đổi', async () => {
    const { openGlossaryManage, manageFilteredRows, manageStatus, manageWorkTierAvailable } = await freshState()
    listMock.mockResolvedValue({ entries: [entry()], error: null })
    lookupMock.mockResolvedValue(workClosedProbe)

    await openGlossaryManage()

    expect(manageStatus.value).toBe('loaded')
    expect(manageFilteredRows.value).toHaveLength(1)
    expect(manageWorkTierAvailable.value).toBe(false)
  })
})

describe('BỐN ca rỗng khác nhau — §Always: "rỗng phải nói vì sao nó rỗng"', () => {
  it('cầu IPC vắng ⇒ status "ipc_unavailable"', async () => {
    const { openGlossaryManage, manageStatus, manageEmptyReasonFor } = await freshState()
    listMock.mockResolvedValue({ entries: null, error: null })
    lookupMock.mockResolvedValue({ found: 'unknown' as const, error: null })

    await openGlossaryManage()

    expect(manageStatus.value).toBe('ipc_unavailable')
    expect(manageEmptyReasonFor(manageStatus.value, 0, 0)).toBe('ipc_unavailable')
  })

  it('lỗi tải THẬT ⇒ status "error", khác hẳn "ipc_unavailable"', async () => {
    const { openGlossaryManage, manageStatus, manageLoadError } = await freshState()
    const err = { code: 'store.open_failed', message_key: 'err.store.open_failed', params: {}, retryable: true }
    listMock.mockResolvedValue({ entries: null, error: err })
    lookupMock.mockResolvedValue({ found: 'unknown' as const, error: null })

    await openGlossaryManage()

    expect(manageStatus.value).toBe('error')
    expect(manageLoadError.value).toEqual(err)
  })

  it('Glossary trống thật (0 mục ở cả hai tầng) ⇒ "glossary_empty", KHÁC "bộ lọc không khớp"', async () => {
    const { openGlossaryManage, manageStatus, manageFilteredRows, manageEmptyReasonFor } = await freshState()
    listMock.mockResolvedValue({ entries: [], error: null })
    lookupMock.mockResolvedValue(workClosedProbe)

    await openGlossaryManage()

    expect(manageEmptyReasonFor(manageStatus.value, 0, manageFilteredRows.value.length)).toBe('glossary_empty')
  })

  it('bộ lọc không khớp gì (danh sách đã nạp có hàng, filter loại hết) ⇒ "filter_no_match"', async () => {
    const { openGlossaryManage, setGlossaryManageCategoryFilter, manageStatus, manageFilteredRows, manageEmptyReasonFor } =
      await freshState()
    listMock.mockResolvedValue({ entries: [entry({ category: 'place' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()

    setGlossaryManageCategoryFilter('person')

    expect(manageFilteredRows.value).toHaveLength(0)
    expect(manageEmptyReasonFor(manageStatus.value, 1, manageFilteredRows.value.length)).toBe('filter_no_match')
  })
})

describe('Tìm và ba bộ lọc — chạy TRONG BỘ NHỚ, 0 lượt IPC', () => {
  it('tìm theo source_term VÀ translation, không phân biệt hoa/thường', async () => {
    const { openGlossaryManage, setGlossaryManageSearch, manageFilteredRows } = await freshState()
    listMock.mockResolvedValue({
      entries: [
        entry({ id: 1, source_term: '青丘', translation: 'Thanh Khâu' }),
        entry({ id: 2, source_term: '慕容', translation: 'Mộ Dung' }),
      ],
      error: null,
    })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()
    listMock.mockClear()

    setGlossaryManageSearch('khâu')
    expect(manageFilteredRows.value.map((r) => r.id)).toEqual([1])

    setGlossaryManageSearch('慕')
    expect(manageFilteredRows.value.map((r) => r.id)).toEqual([2])

    expect(listMock).not.toHaveBeenCalled()
  })

  it('giao của ba bộ lọc (phân loại · xuất xứ · trạng thái chốt) với ô tìm', async () => {
    const { openGlossaryManage, setGlossaryManageCategoryFilter, setGlossaryManageOriginFilter, setGlossaryManageConfirmedFilter, manageFilteredRows } =
      await freshState()
    listMock.mockResolvedValue({
      entries: [
        entry({ id: 1, category: 'person', term_origin: 'manual', translation: 'A' }),
        entry({ id: 2, category: 'person', term_origin: 'import_scan', translation: null }),
        entry({ id: 3, category: 'place', term_origin: 'manual', translation: 'A' }),
      ],
      error: null,
    })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()

    setGlossaryManageCategoryFilter('person')
    setGlossaryManageOriginFilter('manual')
    setGlossaryManageConfirmedFilter('confirmed')

    expect(manageFilteredRows.value.map((r) => r.id)).toEqual([1])
  })
})

describe('Sửa — update_manual_term(tier, id, …), hàng cập nhật TẠI CHỖ', () => {
  it('lưu thành công ⇒ hàng đổi giá trị, form đóng, KHÔNG gọi refreshGlossaryMarks', async () => {
    const { openGlossaryManage, beginGlossaryManageEdit, manageEditTranslation, saveGlossaryManageEdit, manageEditing, manageFilteredRows } =
      await freshState()
    listMock.mockResolvedValue({ entries: [entry({ id: 1, translation: 'Cu' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()
    updateMock.mockResolvedValue({ value: true, error: null })

    beginGlossaryManageEdit()
    expect(manageEditing.value).toBe(true)
    manageEditTranslation.value = 'Moi'
    await saveGlossaryManageEdit()

    expect(manageEditing.value).toBe(false)
    expect(manageFilteredRows.value[0]?.translation).toBe('Moi')
    // §Always: chỉ Xoá/Đẩy tầng gọi refresh — SỬA không nằm trong danh sách đó.
    expect(refreshMarksMock).not.toHaveBeenCalled()
  })

  it('lưu trượt (vd. trigger RAISE(ABORT)) ⇒ hàng giữ nguyên giá trị cũ, lỗi hiện qua manageActionError', async () => {
    const { openGlossaryManage, beginGlossaryManageEdit, manageEditTranslation, saveGlossaryManageEdit, manageActionError, manageFilteredRows } =
      await freshState()
    listMock.mockResolvedValue({ entries: [entry({ id: 1, translation: 'Cu' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()
    const err = { code: 'store.write_failed', message_key: 'err.store.write_failed', params: {}, retryable: false }
    updateMock.mockResolvedValue({ value: null, error: err })

    beginGlossaryManageEdit()
    manageEditTranslation.value = ''
    await saveGlossaryManageEdit()

    expect(manageActionError.value).toEqual(err)
    expect(manageFilteredRows.value[0]?.translation).toBe('Cu')
  })
})

describe('Xoá — kể cả một mục ĐÃ CHỐT là hợp lệ (nhịp HAI, sau khi nhịp MỘT đã qua)', () => {
  it('thành công ⇒ mục biến khỏi danh sách, refreshGlossaryMarks CHẠY với đúng tham số Chương đang mở', async () => {
    const { openGlossaryManage, deleteGlossaryManageEntry, manageFilteredRows, manageDeletePending } = await freshState()
    listMock.mockResolvedValue({ entries: [entry({ id: 1, translation: 'Đã chốt' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()
    deleteMock.mockResolvedValue({ value: true, error: null })
    listMock.mockResolvedValue({ entries: [], error: null }) // nạp lại sau xoá — danh sách rỗng.

    // Nhịp MỘT — chỉ đổi trạng thái, 0 lượt IPC (cụm D vá).
    await deleteGlossaryManageEntry()
    expect(deleteMock).not.toHaveBeenCalled()
    expect(manageDeletePending.value).toBe(true)

    // Nhịp HAI — cùng hàng ⇒ ghi thật.
    await deleteGlossaryManageEntry()

    expect(deleteMock).toHaveBeenCalledWith('global', 1)
    expect(manageFilteredRows.value).toHaveLength(0)
    // 🔴 ĐỐI CHỨNG "gỡ lời gọi thì đỏ": nếu `deleteGlossaryManageEntry` trong
    // `glossaryManageState.ts` không còn gọi `refreshGlossaryMarks(...)`, assertion dưới đây
    // trượt (spy không bao giờ được gọi) — ca này ĐỎ ngay, không cần đọc lại cài đặt.
    expect(refreshMarksMock).toHaveBeenCalledWith(42, fakeSegments.value, 'zh')
  })

  it('trượt (id đã biến mất) ⇒ hiện lỗi, KHÔNG gọi refreshGlossaryMarks, danh sách nạp lại từ Rust', async () => {
    const { openGlossaryManage, deleteGlossaryManageEntry, manageActionError } = await freshState()
    listMock.mockResolvedValue({ entries: [entry({ id: 1 })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()
    const err = { code: 'glossary.entry_missing', message_key: 'err.glossary.entry_missing', params: {}, retryable: false }
    deleteMock.mockResolvedValue({ value: null, error: err })

    await deleteGlossaryManageEntry() // nhịp MỘT.
    await deleteGlossaryManageEntry() // nhịp HAI — trượt.

    expect(manageActionError.value).toEqual(err)
    expect(refreshMarksMock).not.toHaveBeenCalled()
  })
})

describe('Nhịp xác nhận xoá — hai nhịp trong CÙNG lớp phủ (cụm D vá, §I/O Matrix ⑪⑫⑬)', () => {
  /**
   * 🔴 THÊM (vòng rà thứ hai, #2) — bản trước của bốn ca dưới đây chỉ khẳng định
   * `manageDeletePending.value` và số lần gọi mock, KHÔNG hề `mount`/đọc DOM (trừ ⑬b, và ca
   * đó cũng không đọc DOM). Một lỗi đảo `:class`, mất hẳn `<span>` badge, câu hint sai khoá,
   * hay nút không đổi nhãn sẽ SHIP mà cả bốn ca vẫn xanh — đúng lớp lỗi "một bộ test xanh
   * không chứng minh chỗ nối mới được canh". `freshOverlay` ở đây LUÔN mount component thật
   * và LUÔN đọc phần người dùng NHÌN THẤY: badge trên hàng, câu hint, nhãn nút, class nguy
   * hiểm — cả hai chiều (có mặt khi đang chờ, vắng mặt khi không).
   */
  async function freshOverlay(deps: Partial<CommandDeps> = {}) {
    vi.resetModules()
    listMock.mockReset()
    deleteMock.mockReset()
    promoteMock.mockReset()
    updateMock.mockReset()
    lookupMock.mockReset()
    refreshMarksMock.mockReset()
    fakeChapterId.value = 42
    fakeSegments.value = [{ id: 1 }]
    fakeSourceChapter.value = { chapter_id: 42, source_lang: 'zh' }

    const state = await import('../../src/glossaryManageState')
    const i18n = await import('../../src/i18n')
    const commands = await import('../../src/commands')
    const mergedDeps: Partial<CommandDeps> = {
      deleteGlossaryManageEntry: () => {
        void state.deleteGlossaryManageEntry()
      },
      nextGlossaryManageRow: state.nextGlossaryManageRow,
      beginGlossaryManageEdit: state.beginGlossaryManageEdit,
      ...deps,
    }
    commands.installCommands(mergedDeps as CommandDeps)
    const GlossaryManageOverlay = (await import('../../src/GlossaryManageOverlay.vue')).default
    return { state, i18n, GlossaryManageOverlay }
  }

  /** Khẳng định "chưa ở nhịp chờ" NHÌN THẤY được — không badge, không câu hint, nút mang
   * đúng nhãn/không class nguy hiểm "Xoá" thường. */
  function expectNoPendingUi(wrapper: ReturnType<typeof mount>, i18n: typeof import('../../src/i18n')): void {
    expect(wrapper.find('.gm-badge-delete-pending').exists()).toBe(false)
    expect(wrapper.text()).not.toContain(i18n.t('glossary.manage.delete_confirm_hint'))
    const deleteButton = wrapper.get('.gm-actions button:nth-of-type(2)')
    expect(deleteButton.text()).toBe(i18n.t('glossary.manage.delete'))
    expect(deleteButton.classes()).not.toContain('gm-act-danger')
  }

  /** Khẳng định "ĐANG ở nhịp chờ" NHÌN THẤY được — badge, câu hint, nhãn nút đổi, class nguy
   * hiểm có mặt. */
  function expectPendingUi(wrapper: ReturnType<typeof mount>, i18n: typeof import('../../src/i18n')): void {
    expect(wrapper.find('.gm-badge-delete-pending').exists()).toBe(true)
    expect(wrapper.find('.gm-badge-delete-pending').text()).toBe(i18n.t('glossary.manage.delete_confirm_badge'))
    expect(wrapper.text()).toContain(i18n.t('glossary.manage.delete_confirm_hint'))
    const deleteButton = wrapper.get('.gm-actions button:nth-of-type(2)')
    expect(deleteButton.text()).toBe(i18n.t('glossary.manage.delete_confirm_button'))
    expect(deleteButton.classes()).toContain('gm-act-danger')
  }

  it('⑪ nhịp MỘT ngay sau khi mở lớp phủ: hàng vào trạng thái chờ xác nhận, 0 lượt IPC, VÀ giao diện đổi đúng', async () => {
    const { state, i18n, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({ entries: [entry({ id: 1 })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()
    expect(state.manageCursor.value).toBe(0) // con trỏ ở hàng đầu, đúng mặc định của một lượt mở.

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    // TRƯỚC nhịp một — cả bốn thứ đều VẮNG MẶT.
    expectNoPendingUi(wrapper, i18n)

    await wrapper.get('.gm-scrim').trigger('keydown', { key: 'Backspace' })
    await wrapper.vm.$nextTick()

    expect(deleteMock).not.toHaveBeenCalled()
    expect(state.manageDeletePending.value).toBe(true)
    // SAU nhịp một — cả bốn thứ đều CÓ MẶT.
    expectPendingUi(wrapper, i18n)

    wrapper.unmount()
  })

  it('⑫ nhịp HAI trên ĐÚNG hàng đó ⇒ xoá thật, rồi nạp lại trọn danh sách, giao diện trở về trạng thái không-chờ', async () => {
    const { state, i18n, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({ entries: [entry({ id: 1 }), entry({ id: 2, source_term: '慕容' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()
    deleteMock.mockResolvedValue({ value: true, error: null })
    listMock.mockResolvedValue({ entries: [entry({ id: 2, source_term: '慕容' })], error: null })

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.gm-scrim')

    await scrim.trigger('keydown', { key: 'Backspace' }) // nhịp MỘT.
    await wrapper.vm.$nextTick()
    expectPendingUi(wrapper, i18n)

    await scrim.trigger('keydown', { key: 'Backspace' }) // nhịp HAI.
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    expect(deleteMock).toHaveBeenCalledWith('global', 1)
    // 🔴 ĐỐI CHỨNG "gỡ vé hai-nhịp thì đỏ": nếu `deleteGlossaryManageEntry` xoá NGAY ở lượt
    // gọi đầu (bản cũ), `listMock` thứ hai (đã đổi sang chỉ còn id=2) không kịp cấu hình
    // trước lượt xoá thật — ca dưới đây vẫn đúng ở CẢ HAI hình dạng, nên mệnh đề PHÂN BIỆT
    // là `deleteMock` chỉ được gọi sau đúng LẦN GỌI HÀM THỨ HAI (kiểm ở ca ⑪ riêng).
    expect(state.manageFilteredRows.value.map((r) => r.id)).toEqual([2])
    expectNoPendingUi(wrapper, i18n)

    wrapper.unmount()
  })

  it('⑬a ArrowDown (đổi con trỏ) trong lúc đang chờ xác nhận ⇒ trạng thái chờ TAN, không xoá gì, giao diện trở về bình thường', async () => {
    const { state, i18n, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({ entries: [entry({ id: 1 }), entry({ id: 2, source_term: '慕容' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.gm-scrim')

    await scrim.trigger('keydown', { key: 'Backspace' }) // nhịp MỘT.
    await wrapper.vm.$nextTick()
    expect(state.manageDeletePending.value).toBe(true)
    expectPendingUi(wrapper, i18n)

    await scrim.trigger('keydown', { key: 'ArrowDown' })
    await wrapper.vm.$nextTick()

    expect(state.manageDeletePending.value).toBe(false)
    expect(deleteMock).not.toHaveBeenCalled()
    expectNoPendingUi(wrapper, i18n)

    wrapper.unmount()
  })

  it('⑬b Escape trong lúc đang chờ xác nhận ⇒ trạng thái chờ TAN, KHÔNG đóng lớp phủ, không xoá gì, giao diện trở về bình thường', async () => {
    const { state, i18n, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({ entries: [entry({ id: 1 })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.gm-scrim')

    await scrim.trigger('keydown', { key: 'Backspace' }) // nhịp MỘT qua bàn phím thật.
    await wrapper.vm.$nextTick()
    expect(state.manageDeletePending.value).toBe(true)
    expectPendingUi(wrapper, i18n)

    await scrim.trigger('keydown', { key: 'Escape' })
    await wrapper.vm.$nextTick()

    expect(state.manageDeletePending.value).toBe(false)
    expect(deleteMock).not.toHaveBeenCalled()
    expect(state.manageOverlayIsOpen.value).toBe(true) // lớp phủ VẪN mở — Escape chỉ huỷ nhịp một.
    expectNoPendingUi(wrapper, i18n)

    wrapper.unmount()
  })

  it('⑬c vào chế độ Sửa trong lúc đang chờ xác nhận ⇒ trạng thái chờ TAN, badge trên hàng biến mất', async () => {
    const { state, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({ entries: [entry({ id: 1 })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.gm-scrim')

    await scrim.trigger('keydown', { key: 'Backspace' }) // nhịp MỘT.
    await wrapper.vm.$nextTick()
    expect(state.manageDeletePending.value).toBe(true)
    expect(wrapper.find('.gm-badge-delete-pending').exists()).toBe(true)

    await scrim.trigger('keydown', { key: 'Enter' }) // vào chế độ Sửa.
    await wrapper.vm.$nextTick()

    expect(state.manageDeletePending.value).toBe(false)
    expect(deleteMock).not.toHaveBeenCalled()
    // Form Sửa nay chiếm chỗ của `.gm-actions` (nút "Xoá" không còn trên màn hình lúc này —
    // đúng chủ ý của `v-if="!manageEditing"`), nên vế NHÌN THẤY được kiểm ở đây là badge trên
    // hàng, thứ vẫn render độc lập với chế độ sửa.
    expect(wrapper.find('.gm-badge-delete-pending').exists()).toBe(false)

    wrapper.unmount()
  })

  it('🔴 #4 (vòng rà thứ hai) — câu hint của nhịp chờ TRỎ ĐÚNG nhãn nút đang hiện thật (nhãn đã đổi thành "Xác nhận xoá vĩnh viễn")', async () => {
    // Đối chứng gỡ-chỗ-nối: đổi `glossary.manage.delete_confirm_hint` trong `vi.json` về lại
    // câu cũ ("Bấm Xoá (hoặc Backspace)…" — trỏ vào nhãn nút CŨ, thứ không còn hiện trên màn
    // hình lúc câu này hiện) ⇒ ca này ĐỎ.
    const { i18n } = await freshOverlay()
    const hint = i18n.t('glossary.manage.delete_confirm_hint')
    const buttonLabelWhilePending = i18n.t('glossary.manage.delete_confirm_button')

    expect(hint).toContain(buttonLabelWhilePending)
  })

  it('🔴 #5 (vòng rà thứ hai) — nhịp MỘT dọn `manageActionNotice` còn sót từ một lượt Đẩy tầng trước đó ("không áp dụng"), không để nó che câu xác nhận', async () => {
    // Đối chứng gỡ-chỗ-nối: gỡ hai dòng `actionError.value = null` / `actionNotice.value =
    // null` khỏi nhánh nhịp MỘT của `deleteGlossaryManageEntry` ⇒ ca này ĐỎ (câu
    // "không áp dụng" vẫn còn trên màn hình sau khi bấm Backspace).
    const { state, i18n, GlossaryManageOverlay } = await freshOverlay()
    // Hàng tầng `global` — Đẩy tầng lên nó là "không áp dụng", 0 lượt IPC (đường đồng bộ có
    // sẵn trong `promoteGlossaryManageEntry`, không cần mock một lượt IPC trượt).
    listMock.mockResolvedValue({ entries: [entry({ id: 1, tier: 'global' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    state.promoteGlossaryManageEntry() // Đồng bộ — đặt `manageActionNotice`, 0 lượt IPC.
    await wrapper.vm.$nextTick()
    expect(state.manageActionNotice.value).toBe('promote_not_applicable')
    expect(wrapper.text()).toContain(i18n.t('glossary.manage.promote_not_applicable'))

    await wrapper.get('.gm-scrim').trigger('keydown', { key: 'Backspace' }) // nhịp MỘT.
    await wrapper.vm.$nextTick()

    expect(state.manageDeletePending.value).toBe(true)
    // Mệnh đề trung tâm: câu "không áp dụng" CŨ đã biến mất, câu hint xác nhận xoá thế chỗ —
    // không phải cả hai cùng hiện, và không phải câu cũ che mất câu mới.
    expect(wrapper.text()).not.toContain(i18n.t('glossary.manage.promote_not_applicable'))
    expect(state.manageActionNotice.value).toBeNull()
    expect(wrapper.text()).toContain(i18n.t('glossary.manage.delete_confirm_hint'))

    wrapper.unmount()
  })

  it('🔴 #1 (vòng rà thứ hai, NẶNG NHẤT) — giữ phím (event.repeat === true) KHÔNG được phá cả hai nhịp trong MỘT cú nhấn-giữ', async () => {
    // Kịch bản: hệ điều hành tự lặp `keydown` khi người dùng GIỮ phím. Nhịp một trả về ĐỒNG
    // BỘ và không đặt cờ bận nào — nếu `onKeydown` không lọc `event.repeat`, hai sự kiện lặp
    // liên tiếp (giả lập ở đây bằng `repeat: true` ngay từ lần bắn ĐẦU) sẽ chạy nhịp một RỒI
    // nhịp hai trong đúng một thao tác người dùng cảm nhận là MỘT lần bấm — đúng kịch bản cả
    // tính năng hai-nhịp tồn tại để chặn. Đối chứng gỡ-chỗ-nối: gỡ `if (event.repeat) return`
    // khỏi nhánh `Backspace`/`Delete` của `GlossaryManageOverlay.vue::onKeydown` ⇒ ca này ĐỎ
    // (`deleteMock` bị gọi).
    const { state, i18n, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({ entries: [entry({ id: 1 })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()
    deleteMock.mockResolvedValue({ value: true, error: null })

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.gm-scrim')

    // Một cú nhấn-giữ: nhiều `keydown` liên tiếp đều mang `repeat: true` (đúng thứ trình
    // duyệt/OS phát ra sau lần đầu — kể cả lần "đầu tiên" người dùng CẢM NHẬN được ở đây vì
    // vitest không mô phỏng lần `repeat: false` khởi đầu, nên đây là ca XẤU NHẤT: mọi sự kiện
    // của cú giữ đều bị bỏ qua, không riêng "từ lần thứ hai trở đi").
    await scrim.trigger('keydown', { key: 'Backspace', repeat: true })
    await scrim.trigger('keydown', { key: 'Backspace', repeat: true })
    await scrim.trigger('keydown', { key: 'Backspace', repeat: true })
    await wrapper.vm.$nextTick()

    expect(deleteMock).not.toHaveBeenCalled()
    expect(state.manageDeletePending.value).toBe(false) // 0 nhịp nào chạy — repeat bị bỏ qua HOÀN TOÀN.
    expectNoPendingUi(wrapper, i18n)

    // Đối chứng DƯƠNG — cùng phím, KHÔNG `repeat`, vẫn hoạt động (chứng minh lượt lọc ở trên
    // chặn đúng ĐIỀU CẦN CHẶN, không chặn oan cả nhánh Backspace).
    await scrim.trigger('keydown', { key: 'Backspace' })
    await wrapper.vm.$nextTick()
    expect(state.manageDeletePending.value).toBe(true)

    wrapper.unmount()
  })

  it('🔴 #1 đối chứng — ArrowDown/ArrowUp VẪN tự lặp được (chỉ Backspace/Delete bị lọc `repeat`)', async () => {
    const nextMock = vi.fn()
    const prevMock = vi.fn()
    const { state, GlossaryManageOverlay } = await freshOverlay({
      nextGlossaryManageRow: nextMock,
      prevGlossaryManageRow: prevMock,
    })
    listMock.mockResolvedValue({ entries: [entry({ id: 1 }), entry({ id: 2, source_term: '慕容' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.gm-scrim')

    await scrim.trigger('keydown', { key: 'ArrowDown', repeat: true })
    await scrim.trigger('keydown', { key: 'ArrowDown', repeat: true })

    expect(nextMock).toHaveBeenCalledTimes(2) // KHÔNG bị lọc — giữ phím để lướt danh sách vẫn chạy.

    await scrim.trigger('keydown', { key: 'ArrowUp', repeat: true })
    expect(prevMock).toHaveBeenCalledTimes(1)

    wrapper.unmount()
  })
})

describe('Đẩy tầng (Work → Global)', () => {
  it('đích trống ⇒ thành công, danh sách nạp lại, refreshGlossaryMarks CHẠY', async () => {
    const { openGlossaryManage, promoteGlossaryManageEntry, manageFilteredRows } = await freshState()
    listMock.mockResolvedValue({ entries: [entry({ id: 5, tier: 'work' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()
    promoteMock.mockResolvedValue({ value: true, error: null })
    listMock.mockResolvedValue({ entries: [entry({ id: 99, tier: 'global' })], error: null })

    await promoteGlossaryManageEntry()

    expect(promoteMock).toHaveBeenCalledWith(5)
    expect(manageFilteredRows.value[0]?.tier).toBe('global')
    expect(refreshMarksMock).toHaveBeenCalledWith(42, fakeSegments.value, 'zh')
  })

  it('đích ĐÃ CÓ source_term ⇒ lỗi CÓ TÊN, cả hai mục giữ nguyên (KHÔNG nạp lại/refresh)', async () => {
    const { openGlossaryManage, promoteGlossaryManageEntry, manageActionError, manageFilteredRows } = await freshState()
    listMock.mockResolvedValue({ entries: [entry({ id: 5, tier: 'work' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()
    const err = {
      code: 'glossary.global_term_exists',
      message_key: 'err.glossary.global_term_exists',
      params: {},
      retryable: false,
    }
    promoteMock.mockResolvedValue({ value: null, error: err })

    await promoteGlossaryManageEntry()

    expect(manageActionError.value).toEqual(err)
    expect(manageFilteredRows.value[0]?.id).toBe(5)
    expect(refreshMarksMock).not.toHaveBeenCalled()
  })

  it('hàng tầng Global ⇒ "không áp dụng", NÓI RA, 0 lượt IPC promote', async () => {
    const { openGlossaryManage, promoteGlossaryManageEntry, manageActionNotice } = await freshState()
    listMock.mockResolvedValue({ entries: [entry({ id: 1, tier: 'global' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()

    await promoteGlossaryManageEntry()

    expect(promoteMock).not.toHaveBeenCalled()
    expect(manageActionNotice.value).toBe('promote_not_applicable')
  })
})

describe('Xuất CSV — P2/P3 (vòng rà ba lớp 2026-08-25)', () => {
  it('P3 — huỷ NGAY SAU một lượt xuất thành công không còn đọc đường dẫn CŨ', async () => {
    const { openGlossaryManage, exportGlossaryManageTier, manageExportedPath } = await freshState()
    listMock.mockResolvedValue({ entries: [], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()

    exportMock.mockResolvedValue({ outcome: 'done', path: '/tmp/a.csv' })
    await exportGlossaryManageTier()
    expect(manageExportedPath.value).toBe('/tmp/a.csv')

    // Mo lai roi HUY -- duong dan CU khong duoc doc thanh "da ghi" cua lot nay.
    exportMock.mockResolvedValue({ outcome: 'cancelled' })
    await exportGlossaryManageTier()
    expect(manageExportedPath.value).toBeNull()
  })

  it('P2 — không có cầu IPC hiện câu RIÊNG, khác hẳn huỷ hộp thoại (im lặng)', async () => {
    const { openGlossaryManage, exportGlossaryManageTier, manageExportedPath, manageExportIpcUnavailable } =
      await freshState()
    listMock.mockResolvedValue({ entries: [], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()

    exportMock.mockResolvedValue({ outcome: 'cancelled' })
    await exportGlossaryManageTier()
    expect(manageExportIpcUnavailable.value).toBe(false)
    expect(manageExportedPath.value).toBeNull()

    exportMock.mockResolvedValue({ outcome: 'ipc_unavailable' })
    await exportGlossaryManageTier()
    expect(manageExportIpcUnavailable.value).toBe(true)
  })

  it('lượt xuất trượt THẬT hiện lỗi và xoá đường dẫn cũ', async () => {
    const { openGlossaryManage, exportGlossaryManageTier, manageExportError, manageExportedPath } =
      await freshState()
    listMock.mockResolvedValue({ entries: [], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()

    exportMock.mockResolvedValue({ outcome: 'done', path: '/tmp/a.csv' })
    await exportGlossaryManageTier()

    const err = { code: 'x', message_key: 'err.unknown', params: {}, retryable: false }
    exportMock.mockResolvedValue({ outcome: 'error', error: err })
    await exportGlossaryManageTier()

    expect(manageExportError.value).toEqual(err)
    expect(manageExportedPath.value).toBeNull()
  })
})

describe('GlossaryManageOverlay.vue — bàn phím cục bộ, filter modifier VÀ filter ô gõ chữ', () => {
  async function freshOverlay(deps: Partial<CommandDeps> = {}) {
    vi.resetModules()
    listMock.mockReset()
    deleteMock.mockReset()
    promoteMock.mockReset()
    updateMock.mockReset()
    lookupMock.mockReset()
    refreshMarksMock.mockReset()
    fakeChapterId.value = 42
    fakeSegments.value = [{ id: 1 }]
    fakeSourceChapter.value = { chapter_id: 42, source_lang: 'zh' }

    const commands = await import('../../src/commands')
    commands.installCommands(deps as CommandDeps)

    const state = await import('../../src/glossaryManageState')
    const GlossaryManageOverlay = (await import('../../src/GlossaryManageOverlay.vue')).default
    return { state, GlossaryManageOverlay }
  }

  it('ArrowDown/ArrowUp/Backspace/Enter bắn ĐÚNG lệnh qua registry THẬT khi focus NGOÀI ô gõ', async () => {
    // 🔴 SỬA (cụm D vá D10) — bản trước khoá hành vi xoá-một-nhịp (`trigger Backspace` ⇒
    // `deleteHandler` gọi 1 lần, xoá NGAY). `deleteGlossaryManageEntry` dùng handler THẬT
    // (không một mock trần) để ca này kiểm CẢ HAI nhịp qua đúng đường bàn phím sản phẩm,
    // không chỉ đường dispatch → registry → deps() (điều đó vẫn đúng nhưng không còn đủ:
    // nhịp MỘT/HAI sống BÊN TRONG chính hàm `deleteGlossaryManageEntry`, một mock trần không
    // chạy qua nó).
    const nextMock = vi.fn()
    const prevMock = vi.fn()
    const editHandler = vi.fn()
    const { state, GlossaryManageOverlay } = await freshOverlay({
      nextGlossaryManageRow: nextMock,
      prevGlossaryManageRow: prevMock,
      deleteGlossaryManageEntry: () => {
        void state.deleteGlossaryManageEntry()
      },
      beginGlossaryManageEdit: editHandler,
    })
    listMock.mockResolvedValue({ entries: [entry({ id: 1 })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()
    deleteMock.mockResolvedValue({ value: true, error: null })
    listMock.mockResolvedValue({ entries: [], error: null })

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.gm-scrim')

    await scrim.trigger('keydown', { key: 'ArrowDown' })
    expect(nextMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 'ArrowUp' })
    expect(prevMock).toHaveBeenCalledTimes(1)

    // Nhịp MỘT — 0 lượt IPC, hàng vào trạng thái chờ xác nhận.
    await scrim.trigger('keydown', { key: 'Backspace' })
    expect(deleteMock).not.toHaveBeenCalled()
    expect(state.manageDeletePending.value).toBe(true)

    // Nhịp HAI — cùng phím, cùng hàng ⇒ xoá thật.
    await scrim.trigger('keydown', { key: 'Backspace' })
    await wrapper.vm.$nextTick()
    expect(deleteMock).toHaveBeenCalledWith('global', 1)

    await scrim.trigger('keydown', { key: 'Enter' })
    expect(editHandler).toHaveBeenCalledTimes(1)

    wrapper.unmount()
  })

  it('⚠️ §Always — ⌘Backspace KHÔNG kích hoạt Xoá (filter modifier trước mọi nhánh)', async () => {
    const deleteHandler = vi.fn()
    const { state, GlossaryManageOverlay } = await freshOverlay({ deleteGlossaryManageEntry: deleteHandler })
    listMock.mockResolvedValue({ entries: [entry({ id: 1 })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    await wrapper.get('.gm-scrim').trigger('keydown', { key: 'Backspace', metaKey: true })
    expect(deleteHandler).not.toHaveBeenCalled()

    // Đối chứng dương: cùng phím, KHÔNG bổ trợ, VẪN hoạt động.
    await wrapper.get('.gm-scrim').trigger('keydown', { key: 'Backspace' })
    expect(deleteHandler).toHaveBeenCalledTimes(1)

    wrapper.unmount()
  })

  it('🔵 gõ Backspace TRONG ô tìm KHÔNG kích hoạt Xoá — filter theo TARGET là ô gõ chữ', async () => {
    const deleteHandler = vi.fn()
    const { state, GlossaryManageOverlay } = await freshOverlay({ deleteGlossaryManageEntry: deleteHandler })
    listMock.mockResolvedValue({ entries: [entry({ id: 1 })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const searchInput = wrapper.get('.gm-toolbar input[type="text"]')
    await searchInput.trigger('keydown', { key: 'Backspace' })

    expect(deleteHandler).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})

/**
 * Ba ca dưới đây là ĐỐI CHỨNG cho vòng rà ba lớp 2026-08-24. Mỗi ca canh một chỗ mà cả
 * mười cổng, 446 ca vitest và toàn bộ `cargo test` đều XANH trong khi sản phẩm đang hỏng —
 * đúng khuôn *"một bộ test xanh không chứng minh chỗ nối mới được canh"* (`AGENTS.md`).
 */
describe('GlossaryManageOverlay.vue — đối chứng vòng rà: ca rỗng, phím trên nút, và reset', () => {
  async function freshOverlay(deps: Partial<CommandDeps> = {}) {
    vi.resetModules()
    listMock.mockReset()
    deleteMock.mockReset()
    promoteMock.mockReset()
    updateMock.mockReset()
    lookupMock.mockReset()
    refreshMarksMock.mockReset()
    fakeChapterId.value = 42
    fakeSegments.value = [{ id: 1 }]
    fakeSourceChapter.value = { chapter_id: 42, source_lang: 'zh' }

    const commands = await import('../../src/commands')
    commands.installCommands(deps as CommandDeps)

    const state = await import('../../src/glossaryManageState')
    const GlossaryManageOverlay = (await import('../../src/GlossaryManageOverlay.vue')).default
    return { state, GlossaryManageOverlay }
  }

  it('🔴 bộ lọc loại hết hàng trên Glossary CÓ dữ liệu ⇒ DOM nói "không khớp bộ lọc", KHÔNG nói "trống"', async () => {
    // Ca ở §I/O Matrix hàng "Bộ lọc không khớp". Ca cũ gọi THẲNG `manageEmptyReasonFor` với
    // `totalCount` tự chọn tay, nên nó đi vòng qua chính chỗ gọi hỏng trong template. Ca này
    // mount thật và đọc CHỮ TRÊN MÀN HÌNH — gỡ `manageTotalRows` khỏi chỗ gọi thì nó ĐỎ.
    const { state, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({ entries: [entry({ id: 1, category: 'place' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    state.setGlossaryManageCategoryFilter('person') // loại hết hàng, Glossary vẫn có 1 mục
    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(state.manageTotalRows.value).toBe(1)
    expect(state.manageFilteredRows.value).toHaveLength(0)
    const text = wrapper.get('.gm-scrim').text()
    expect(text).toContain('khớp bộ lọc')
    expect(text).not.toContain('Glossary đang trống')

    wrapper.unmount()
  })

  it('🔴 Enter khi tiêu điểm đang ở một NÚT không bị nuốt thành lệnh Sửa', async () => {
    // `onKeydown` gắn trên `.gm-scrim`, tức tổ tiên của mọi nút. Thiếu `HTMLButtonElement`
    // trong danh sách miễn, Enter trên nút "Xoá"/"Đẩy lên Toàn cục" gọi `preventDefault()`
    // rồi dispatch `glossary.manage.edit` — một phím LÀM VIỆC KHÁC, không phải một phím chết.
    const editHandler = vi.fn()
    const { state, GlossaryManageOverlay } = await freshOverlay({ beginGlossaryManageEdit: editHandler })
    listMock.mockResolvedValue({ entries: [entry({ id: 1 })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    // `trigger()` không đặt được `target`, nên bắn một `KeyboardEvent` THẬT từ chính cái nút —
    // nó nổi bọt lên `.gm-scrim` đúng như một lượt gõ thật, và `event.target` là cái nút.
    const button = wrapper.element.querySelector('button')
    expect(button).not.toBeNull()
    button?.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))
    await wrapper.vm.$nextTick()
    expect(editHandler).not.toHaveBeenCalled()

    // Đối chứng DƯƠNG — cùng phím, tiêu điểm ngoài nút, vẫn phải chạy.
    await wrapper.get('.gm-scrim').trigger('keydown', { key: 'Enter' })
    expect(editHandler).toHaveBeenCalledTimes(1)

    wrapper.unmount()
  })

  it('resetGlossaryManage đưa MỌI ô nhớ về trạng thái đầu — `check:panel-refs` chỉ thấy dòng gán, không thấy giá trị', async () => {
    // §GIỚI HẠN mục 2 của chính cổng: *"Một dòng gán trong hàm reset không chứng minh giá trị
    // gán là ĐÚNG. Mệnh đề ấy thuộc `tests/frontend/**`."* Đây là chỗ trả mệnh đề đó.
    const state = await freshState()
    listMock.mockResolvedValue({ entries: [entry({ id: 1 })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()
    state.setGlossaryManageSearch('青')
    state.setGlossaryManageCategoryFilter('place')

    state.resetGlossaryManage()

    expect(state.manageOverlayIsOpen.value).toBe(false)
    expect(state.manageStatus.value).toBe('unknown')
    expect(state.manageLoadError.value).toBeNull()
    expect(state.manageTotalRows.value).toBe(0)
    expect(state.manageFilteredRows.value).toHaveLength(0)
    expect(state.manageSearchQuery.value).toBe('')
    expect(state.manageCategoryFilter.value).toBe('all')
    expect(state.manageOriginFilter.value).toBe('all')
    expect(state.manageConfirmedFilter.value).toBe('all')
    expect(state.manageCursor.value).toBe(0)
    expect(state.manageEditing.value).toBe(false)
    expect(state.manageSaving.value).toBe(false)
    expect(state.manageSavingAction.value).toBe('save')
    expect(state.manageActionError.value).toBeNull()
    expect(state.manageActionNotice.value).toBeNull()
    expect(state.manageWorkTierAvailable.value).toBe(false)
  })
})

/**
 * Cụm F ⑦ (2026-08-26) — `aria-activedescendant` trên `<ul class="gm-list">`. Trước bản vá
 * này, `role="listbox"`/`aria-selected` tồn tại nhưng KHÔNG hàng nào bị mọi 37 ca hiện có
 * (grep `aria`/`activedescendant`/`listbox` cho 0 kết quả TRƯỚC lượt vá) chạm tới — chú
 * thích cạnh `<ul>` khai đã đóng lỗ hổng AC7 mà chưa hề có phép kiểm nào canh mệnh đề đó.
 */
describe('GlossaryManageOverlay.vue — `aria-activedescendant` trỏ đúng con trỏ (cụm F ⑤)', () => {
  async function freshOverlay(deps: Partial<CommandDeps> = {}) {
    vi.resetModules()
    listMock.mockReset()
    deleteMock.mockReset()
    promoteMock.mockReset()
    updateMock.mockReset()
    lookupMock.mockReset()
    refreshMarksMock.mockReset()
    fakeChapterId.value = 42
    fakeSegments.value = [{ id: 1 }]
    fakeSourceChapter.value = { chapter_id: 42, source_lang: 'zh' }

    const state = await import('../../src/glossaryManageState')
    const commands = await import('../../src/commands')
    const mergedDeps: Partial<CommandDeps> = {
      nextGlossaryManageRow: state.nextGlossaryManageRow,
      prevGlossaryManageRow: state.prevGlossaryManageRow,
      ...deps,
    }
    commands.installCommands(mergedDeps as CommandDeps)
    const GlossaryManageOverlay = (await import('../../src/GlossaryManageOverlay.vue')).default
    return { state, GlossaryManageOverlay }
  }

  it('⑤a con trỏ ở hàng thứ n ⇒ `<ul>` mang `aria-activedescendant` bằng ĐÚNG `id` của `<li>` thứ n', async () => {
    const { state, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({
      entries: [entry({ id: 1, source_term: 'một' }), entry({ id: 2, source_term: 'hai' })],
      error: null,
    })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()
    expect(state.manageCursor.value).toBe(0)

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const list = wrapper.get('.gm-list')
    const rows = wrapper.findAll('.gm-list li')
    expect(rows).toHaveLength(2)
    const activeId = list.attributes('aria-activedescendant')
    expect(activeId).toBeDefined()
    expect(activeId).toBe(rows[0]?.attributes('id'))
    expect(activeId).not.toBe(rows[1]?.attributes('id'))

    wrapper.unmount()
  })

  it('⑤b con trỏ di chuyển n → n+1 ⇒ `aria-activedescendant` đổi theo, trỏ `id` hàng mới', async () => {
    const { state, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({
      entries: [entry({ id: 1, source_term: 'một' }), entry({ id: 2, source_term: 'hai' })],
      error: null,
    })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const list = wrapper.get('.gm-list')
    const rows = wrapper.findAll('.gm-list li')
    const firstId = rows[0]?.attributes('id')
    const secondId = rows[1]?.attributes('id')
    expect(list.attributes('aria-activedescendant')).toBe(firstId)

    await wrapper.get('.gm-scrim').trigger('keydown', { key: 'ArrowDown' })
    await wrapper.vm.$nextTick()
    expect(state.manageCursor.value).toBe(1)
    expect(list.attributes('aria-activedescendant')).toBe(secondId)
    expect(list.attributes('aria-activedescendant')).not.toBe(firstId)

    wrapper.unmount()
  })

  it('⑤c danh sách rỗng (bộ lọc loại hết hàng) ⇒ `<ul>` KHÔNG mang `aria-activedescendant` trỏ một `id` không tồn tại', async () => {
    const { state, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({ entries: [entry({ id: 1, category: 'place' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()
    state.setGlossaryManageCategoryFilter('person') // loại hết hàng -- danh sách lọc rỗng.
    expect(state.manageFilteredRows.value).toHaveLength(0)

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    // Danh sách rỗng ⇒ nhánh `<p class="gm-empty">` render, KHÔNG `<ul class="gm-list">` --
    // đối chứng "thuộc tính vắng mặt" vì phần tử mang nó còn không tồn tại trong DOM.
    expect(wrapper.find('.gm-list').exists()).toBe(false)

    wrapper.unmount()
  })

  /**
   * ⑤d (vòng rà 1, 2026-08-26) — phần tử mang `aria-activedescendant` phải CHÍNH LÀ
   * `document.activeElement` khi danh sách có hàng. Đây là vế mà ⑤a/⑤b/⑤c KHÔNG canh: cả ba
   * ca đó gọi `state.openGlossaryManage()` RỒI MỚI `mount(...)`, nên `manageOverlayIsOpen`
   * đã `true` NGAY LÚC watcher (`watch(manageOverlayIsOpen, …)` trong `<script setup>`) được
   * dựng — watcher không thấy một lượt CHUYỂN `false → true` nào và KHÔNG BAO GIỜ chạy (Vue
   * `watch()` không tự chạy lúc mới đăng ký, khác `watchEffect`). Ba ca cũ vì thế chỉ đọc
   * GIÁ TRỊ `aria-activedescendant` do template tính sẵn, không đi qua đường `.focus()` sản
   * phẩm dùng thật — đúng cơ chế mà trình đọc màn hình đọc lệ thuộc vào, và đúng lỗ hổng mà
   * bản đầu của mục ⑦ tưởng đã đóng (WAI-ARIA chỉ tôn trọng `aria-activedescendant` trên
   * phần tử ĐANG GIỮ TIÊU ĐIỂM).
   */
  it('⑤d phần tử mang `aria-activedescendant` phải CHÍNH LÀ `document.activeElement` khi danh sách có hàng', async () => {
    const { state, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({
      entries: [entry({ id: 1, source_term: 'một' }), entry({ id: 2, source_term: 'hai' })],
      error: null,
    })
    lookupMock.mockResolvedValue(workOpenProbe)

    // 🔴 Thứ tự NGƯỢC ba ca ⑤a/⑤b/⑤c: mount TRƯỚC khi mở, để watcher focus-ban-đầu thấy
    // đúng một lượt chuyển `false → true` và THẬT SỰ chạy.
    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.gm-scrim').exists()).toBe(false) // chua mo -- doi chung khoi diem.

    await state.openGlossaryManage()
    // `openGlossaryManage()` đặt `manageOverlayIsOpen` NGAY (kích watcher focus-ban-đầu,
    // lúc đó `manageStatus` còn `'unknown'` ⇒ rơi về `panel`) RỒI MỚI `await` tải danh sách
    // (chuyển `manageStatus` sang `'loaded'`, kích watcher THỨ HAI dời tiêu điểm sang
    // `<ul>`). Hai `nextTick`: một cho DOM cập nhật sau khi mở, một cho DOM cập nhật sau khi
    // watcher thứ hai gọi `list.value?.focus()`.
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    const list = wrapper.get('.gm-list')
    const activeId = list.attributes('aria-activedescendant')
    expect(activeId).toBeDefined()

    // Mệnh đề TRUNG TÂM của ca này: KHÔNG chỉ giá trị thuộc tính đúng, mà phần tử MANG
    // thuộc tính đó (`<ul>`, CONTAINER) phải THẬT SỰ đang giữ tiêu điểm DOM -- WAI-ARIA chỉ
    // tôn trọng `aria-activedescendant` trên phần tử ĐANG giữ tiêu điểm. `activeId` là một
    // THAM CHIẾU tới `<li>` "đang chọn ảo" (kiểm ở ⑤a/⑤b) -- KHÔNG phải id của chính `<ul>`
    // (container không có `id`), nên hai giá trị này không bằng nhau, và không NÊN bằng nhau.
    expect(document.activeElement).toBe(list.element)
    expect(document.getElementById(activeId as string)).not.toBeNull() // tham chieu phai GIAI QUYET duoc

    // Đối chứng ÂM: `panel` (`role="dialog"`) — đích focus CŨ — KHÔNG còn giữ tiêu điểm.
    const panel = wrapper.get('.gm-panel')
    expect(document.activeElement).not.toBe(panel.element)

    wrapper.unmount()
  })

  /**
   * ⑤e (P1, vòng rà 2, 2026-08-26) — HỒI QUY do watcher 0→N của cụm F ⑦/vòng rà 1: chiều
   * NGƯỢC (N→0) thiếu, nên xoá hàng CUỐI CÙNG trong khi tiêu điểm đang ở `<ul>` làm `<ul>`
   * bị gỡ khỏi DOM CÙNG LÚC nó đang giữ tiêu điểm — trình duyệt đẩy tiêu điểm ra
   * `document.body`, NGOÀI `.gm-scrim`, nơi `trapTab`/`onEscape`/`onKeydown` nghe qua
   * bubbling — Tab/Escape/mũi tên/nhịp xoá NGỪNG PHẢN HỒI.
   *
   * 🔴 Dùng hai nhịp XOÁ THẬT (`state.deleteGlossaryManageEntry()`, gọi trực tiếp, KHÔNG
   * `scrim.trigger('keydown', …)`) — `.trigger()` bắn thẳng vào phần tử chỉ định, không
   * định tuyến theo `document.activeElement` thật, nên nó sẽ xanh giả trên đúng lỗi P1.
   */
  it('⑤e (P1) tiêu điểm dời VỀ `panel` khi danh sách cạn hẳn sau khi xoá hàng cuối cùng — không rơi ra `document.body`', async () => {
    const { state, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({ entries: [entry({ id: 1, source_term: 'một' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)

    // Mount TRƯỚC khi mở (khuôn ⑤d) để watcher focus-ban-đầu THẬT SỰ chạy và đưa tiêu điểm
    // lên `<ul>` một khi danh sách có đúng MỘT hàng.
    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    await state.openGlossaryManage()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    const list = wrapper.get('.gm-list')
    expect(document.activeElement).toBe(list.element) // khoi diem: tieu diem dang o <ul>.

    deleteMock.mockResolvedValue({ value: true, error: null })
    listMock.mockResolvedValue({ entries: [], error: null }) // sau khi xoa: danh sach rong.

    await state.deleteGlossaryManageEntry() // nhip MOT -- 0 luot IPC.
    expect(deleteMock).not.toHaveBeenCalled()
    await state.deleteGlossaryManageEntry() // nhip HAI -- ghi that, nap lai rong.
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    expect(state.manageFilteredRows.value).toHaveLength(0)
    expect(wrapper.find('.gm-list').exists()).toBe(false) // <ul> da bi go khoi DOM.

    // Mệnh đề TRUNG TÂM: tiêu điểm KHÔNG rơi ra `document.body` -- nó dời về `panel`, phần
    // tử còn SỐNG TRONG `.gm-scrim`, giữ Tab/Escape/mũi tên bubbling được.
    const panel = wrapper.get('.gm-panel')
    expect(document.activeElement).toBe(panel.element)
    expect(document.activeElement).not.toBe(document.body)

    wrapper.unmount()
  })

  /**
   * ⑤f (P2b, vòng rà 2, 2026-08-26) — mệnh đề *"không bao giờ cướp tiêu điểm khỏi một ô
   * người dùng đang thao tác"* mà chú thích của watcher 0→N tự khai, chưa từng có ca canh.
   * Đặt tiêu điểm THẬT vào ô tìm (không qua simulated keyboard event), đổi bộ lọc sao cho
   * danh sách VẪN còn hàng (không phải ca 0→N, mà một identity MỚI của `manageFilteredRows`
   * trong khi N vẫn > 0 suốt) — tiêu điểm phải giữ NGUYÊN ở ô tìm.
   */
  it('⑤f (P2b) đổi bộ lọc trong khi danh sách vẫn còn hàng KHÔNG cướp tiêu điểm khỏi ô tìm đang gõ', async () => {
    const { state, GlossaryManageOverlay } = await freshOverlay()
    listMock.mockResolvedValue({
      entries: [
        entry({ id: 1, source_term: 'một', category: 'place' }),
        entry({ id: 2, source_term: 'hai', category: 'person' }),
      ],
      error: null,
    })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    // Tự tay chuyển tiêu điểm sang ô tìm -- mô phỏng người dùng đang thao tác ở đó, KHÔNG
    // qua `.trigger('keydown', …)` trên `.gm-scrim` (không định tuyến theo tiêu điểm thật).
    const searchInput = wrapper.get('.gm-toolbar input[type="text"]').element as HTMLInputElement
    searchInput.focus()
    expect(document.activeElement).toBe(searchInput)

    // Đổi bộ lọc sao cho danh sách VẪN còn đúng MỘT hàng (không rỗng, không phải ca N→0).
    state.setGlossaryManageCategoryFilter('place')
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()
    expect(state.manageFilteredRows.value).toHaveLength(1)

    // Mệnh đề TRUNG TÂM: tiêu điểm KHÔNG bị cướp về `<ul>` hay `panel` -- nó vẫn ở ô tìm.
    expect(document.activeElement).toBe(searchInput)

    wrapper.unmount()
  })
})
