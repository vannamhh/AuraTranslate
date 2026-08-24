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

describe('Xoá — kể cả một mục ĐÃ CHỐT là hợp lệ', () => {
  it('thành công ⇒ mục biến khỏi danh sách, refreshGlossaryMarks CHẠY với đúng tham số Chương đang mở', async () => {
    const { openGlossaryManage, deleteGlossaryManageEntry, manageFilteredRows } = await freshState()
    listMock.mockResolvedValue({ entries: [entry({ id: 1, translation: 'Đã chốt' })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await openGlossaryManage()
    deleteMock.mockResolvedValue({ value: true, error: null })
    listMock.mockResolvedValue({ entries: [], error: null }) // nạp lại sau xoá — danh sách rỗng.

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

    await deleteGlossaryManageEntry()

    expect(manageActionError.value).toEqual(err)
    expect(refreshMarksMock).not.toHaveBeenCalled()
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
    const nextMock = vi.fn()
    const prevMock = vi.fn()
    const deleteHandler = vi.fn()
    const editHandler = vi.fn()
    const { state, GlossaryManageOverlay } = await freshOverlay({
      nextGlossaryManageRow: nextMock,
      prevGlossaryManageRow: prevMock,
      deleteGlossaryManageEntry: deleteHandler,
      beginGlossaryManageEdit: editHandler,
    })
    listMock.mockResolvedValue({ entries: [entry({ id: 1 })], error: null })
    lookupMock.mockResolvedValue(workOpenProbe)
    await state.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()
    const scrim = wrapper.get('.gm-scrim')

    await scrim.trigger('keydown', { key: 'ArrowDown' })
    expect(nextMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 'ArrowUp' })
    expect(prevMock).toHaveBeenCalledTimes(1)
    await scrim.trigger('keydown', { key: 'Backspace' })
    expect(deleteHandler).toHaveBeenCalledTimes(1)
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
