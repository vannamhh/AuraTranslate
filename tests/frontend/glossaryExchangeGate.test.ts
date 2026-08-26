/**
 * Cửa loại trừ Xuất ↔ Nhập của Glossary — cụm D vá (vòng rà Epic 3, 2026-08-26).
 *
 * ⚠️ Tệp RIÊNG, không nhét vào `glossaryManage.test.ts`/`glossaryImportPreview.test.ts`: đây
 * là mệnh đề duy nhất cần MỘT LƯỢT `import` cả `glossaryManageState.ts` LẪN
 * `glossaryImportState.ts` trong CÙNG một ca — cả hai module đều mock `config/glossary.ts`
 * ở biên IPC (cùng khuôn hai tệp kia), gộp đủ adapter cả hai bên cần.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import type { GlossaryEntry } from '../../src/config/glossary'
import type { CommandDeps } from '../../src/commands'

const listMock = vi.fn()
const lookupMock = vi.fn()
const exportMock = vi.fn()
const openPreviewMock = vi.fn()

vi.mock('../../src/config/glossary', () => ({
  listGlossaryEntries: () => listMock(),
  lookupGlossaryTerm: (term: string) => lookupMock(term),
  exportGlossaryTier: (tier: string) => exportMock(tier),
  openGlossaryImportPreview: (tier: string) => openPreviewMock(tier),
}))

vi.mock('../../src/panels/editorPanelState', () => ({
  editorChapterId: { value: null },
  editorSegments: { value: [] },
}))
vi.mock('../../src/panels/sourcePanelState', () => ({ sourceChapter: { value: null } }))
vi.mock('../../src/panels/glossaryMarksState', () => ({ refreshGlossaryMarks: () => {} }))

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

async function freshGate() {
  vi.resetModules()
  listMock.mockReset()
  lookupMock.mockReset()
  exportMock.mockReset()
  openPreviewMock.mockReset()
  const manage = await import('../../src/glossaryManageState')
  const importState = await import('../../src/glossaryImportState')
  const gate = await import('../../src/glossaryExchangeGate')
  return { manage, importState, gate }
}

/**
 * 🔴 THÊM (vòng rà thứ hai, #3) — biến thể của `freshGate()` mà CŨNG cài bộ command thật và
 * `mount` `GlossaryManageOverlay.vue`. Ba ca ở trên gọi thẳng hàm state — chúng chứng minh
 * ĐÚNG cờ `glossaryExchangeBusy` đổi giá trị, nhưng KHÔNG chứng minh binding template của nút
 * còn trỏ vào cờ đó. Nếu một lượt sửa sau này lỡ trỏ `:disabled` của một trong hai nút về lại
 * `manageExportBusy`/`importOpening` RIÊNG (cờ cũ, trước cụm D), ba ca kia vẫn xanh trong khi
 * người dùng mở được HAI hộp thoại hệ điều hành cùng lúc — đúng lỗ mà mục nợ này tồn tại để
 * đóng.
 */
async function freshGateOverlay() {
  const { manage, importState, gate } = await freshGate()
  const commands = await import('../../src/commands')
  const deps: Partial<CommandDeps> = {
    exportGlossaryManageTier: () => {
      void manage.exportGlossaryManageTier()
    },
    // Cùng khuôn `main.ts:573` — handler zero-arg đọc `manageExchangeTier.value` chính nó.
    openGlossaryImportPreview: () => {
      void importState.openGlossaryImportPreviewOverlay(manage.manageExchangeTier.value)
    },
  }
  commands.installCommands(deps as CommandDeps)
  const i18n = await import('../../src/i18n')
  const GlossaryManageOverlay = (await import('../../src/GlossaryManageOverlay.vue')).default
  return { manage, importState, gate, i18n, GlossaryManageOverlay }
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('glossaryExchangeBusy — Xuất và Nhập loại trừ lẫn nhau (cụm D vá)', () => {
  it('một lượt Xuất đang bay ⇒ cờ dùng chung lên `true`, và `openGlossaryImportPreviewOverlay` KHÔNG khởi được lượt thứ hai', async () => {
    const { manage, importState, gate } = await freshGate()
    listMock.mockResolvedValue({ entries: [entry()], error: null })
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })
    await manage.openGlossaryManage()

    let resolveExport: (() => void) | undefined
    exportMock.mockReturnValue(
      new Promise((resolve) => {
        resolveExport = () => resolve({ outcome: 'done', path: '/tmp/a.csv' })
      }),
    )

    const exporting = manage.exportGlossaryManageTier() // KHÔNG await -- đang bay.
    expect(gate.glossaryExchangeBusy.value).toBe(true)

    await importState.openGlossaryImportPreviewOverlay('global') // Bấm "Nhập CSV" trong lúc Xuất đang bay.

    expect(openPreviewMock).not.toHaveBeenCalled() // 0 hộp thoại thứ hai được mở.
    expect(importState.importOpening.value).toBe(false) // KHÔNG kẹt `true` -- trả về ngay, không đổi state.

    resolveExport?.()
    await exporting
    expect(gate.glossaryExchangeBusy.value).toBe(false) // Xuất xong ⇒ cờ hạ, Nhập dùng được lại.
  })

  it('một lượt Nhập đang mở hộp thoại ⇒ cờ dùng chung lên `true`, và `exportGlossaryManageTier` KHÔNG khởi được lượt Xuất', async () => {
    const { manage, importState, gate } = await freshGate()
    listMock.mockResolvedValue({ entries: [entry()], error: null })
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })
    await manage.openGlossaryManage()

    let resolveOpen: (() => void) | undefined
    openPreviewMock.mockReturnValue(
      new Promise((resolve) => {
        resolveOpen = () => resolve({ outcome: 'cancelled' })
      }),
    )

    const opening = importState.openGlossaryImportPreviewOverlay('global') // KHÔNG await.
    expect(gate.glossaryExchangeBusy.value).toBe(true)

    await manage.exportGlossaryManageTier() // Bấm "Xuất CSV" trong lúc Nhập đang mở hộp thoại.

    expect(exportMock).not.toHaveBeenCalled()
    expect(manage.manageExportBusy.value).toBe(false)

    resolveOpen?.()
    await opening
    expect(gate.glossaryExchangeBusy.value).toBe(false)
  })

  it('mở lại lớp phủ Quản lý giữa chừng ⇒ cờ dùng chung hạ theo, không kẹt `true` mãi mãi', async () => {
    const { manage, gate } = await freshGate()
    listMock.mockResolvedValue({ entries: [entry()], error: null })
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })
    await manage.openGlossaryManage()

    exportMock.mockReturnValue(new Promise(() => {})) // KHÔNG BAO GIỜ về.
    void manage.exportGlossaryManageTier()
    expect(gate.glossaryExchangeBusy.value).toBe(true)

    manage.closeGlossaryManage()
    listMock.mockResolvedValue({ entries: [], error: null })
    await manage.openGlossaryManage() // Mở lại — bump `sequence`, phải tự dọn cờ dùng chung.

    expect(gate.glossaryExchangeBusy.value).toBe(false)
  })

  it('🔴 #3 (vòng rà thứ hai) — CẢ HAI nút "Xuất CSV"/"Nhập CSV" mang `disabled` khi glossaryExchangeBusy đúng, và nhả ra sau khi xong', async () => {
    const { manage, gate, GlossaryManageOverlay } = await freshGateOverlay()
    listMock.mockResolvedValue({ entries: [entry()], error: null })
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })
    await manage.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    const exportButton = wrapper.get('.gm-exchange-actions button:nth-of-type(1)')
    const importButton = wrapper.get('[data-glossary-import-open]')
    expect(exportButton.text()).toBe('Xuất CSV')

    // TRƯỚC lượt Xuất — cả hai nút BẬT.
    expect(exportButton.attributes('disabled')).toBeUndefined()
    expect(importButton.attributes('disabled')).toBeUndefined()

    let resolveExport: (() => void) | undefined
    exportMock.mockReturnValue(
      new Promise((resolve) => {
        resolveExport = () => resolve({ outcome: 'done', path: '/tmp/a.csv' })
      }),
    )

    await exportButton.trigger('click')
    await wrapper.vm.$nextTick()
    expect(gate.glossaryExchangeBusy.value).toBe(true)

    // 🔴 ĐỐI CHỨNG "gỡ chỗ nối template thì đỏ": nếu `:disabled` của một trong hai nút còn
    // trỏ vào cờ RIÊNG (`manageExportBusy`/`importOpening`) thay vì `glossaryExchangeBusy`
    // dùng chung, một trong hai assertion dưới đây trượt — nút kia vẫn bấm được trong lúc
    // hộp thoại hệ điều hành của nút này đang mở.
    expect(exportButton.attributes('disabled')).toBeDefined()
    expect(importButton.attributes('disabled')).toBeDefined()

    resolveExport?.()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    // SAU khi Xuất xong — cả hai nút BẬT lại.
    expect(gate.glossaryExchangeBusy.value).toBe(false)
    expect(exportButton.attributes('disabled')).toBeUndefined()
    expect(importButton.attributes('disabled')).toBeUndefined()

    wrapper.unmount()
  })

  it('🔴 #10 (vòng rà thứ hai) — Nhập đang mở hộp thoại ⇒ vùng Xuất hiện câu NÓI RÕ vì sao nút của nó xám (không chỉ khoá câm)', async () => {
    const { manage, importState, i18n, GlossaryManageOverlay } = await freshGateOverlay()
    listMock.mockResolvedValue({ entries: [entry()], error: null })
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })
    await manage.openGlossaryManage()

    const wrapper = mount(GlossaryManageOverlay, { attachTo: document.body })
    await wrapper.vm.$nextTick()

    expect(wrapper.text()).not.toContain(i18n.t('glossary.manage.exchange_busy_other'))

    let resolveOpen: (() => void) | undefined
    openPreviewMock.mockReturnValue(
      new Promise((resolve) => {
        resolveOpen = () => resolve({ outcome: 'cancelled' })
      }),
    )
    void importState.openGlossaryImportPreviewOverlay('global') // "bấm Nhập CSV".
    await wrapper.vm.$nextTick()

    // 🔴 ĐỐI CHỨNG "gỡ chỗ nối thì đỏ": nếu gỡ nhánh `v-if="glossaryExchangeBusy &&
    // !manageExportBusy"` khỏi template, câu này KHÔNG hiện dù nút Xuất vẫn xám.
    expect(manage.manageExportBusy.value).toBe(false) // KHÔNG phải chính Xuất đang bận.
    expect(wrapper.text()).toContain(i18n.t('glossary.manage.exchange_busy_other'))
    const exportButton = wrapper.get('.gm-exchange-actions button:nth-of-type(1)')
    expect(exportButton.attributes('disabled')).toBeDefined() // và nút vẫn xám, đúng lý do vừa nói.

    resolveOpen?.()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()
    expect(wrapper.text()).not.toContain(i18n.t('glossary.manage.exchange_busy_other')) // xong ⇒ câu biến mất.

    wrapper.unmount()
  })
})

/**
 * 🔴 #8 (vòng rà thứ hai) — `resetGlossaryExchangeGate()` phải là chỗ gọi THẬT của
 * `resetGlossaryManage`/`resetGlossaryImport`, không chỉ một câu khai trong doc-comment.
 *
 * ⚠️ Vì sao KHÔNG kiểm bằng `glossaryExchangeBusy.value === false` — bản trước
 * (`setGlossaryExchangeBusy(false)`) và bản sau (`resetGlossaryExchangeGate()`) cho ĐÚNG
 * CÙNG kết quả quan sát được từ bên ngoài (cờ hạ `false`), nên một phép kiểm chỉ đọc giá trị
 * cờ không phân biệt được hai đường — nó xanh ở CẢ HAI hình dạng. Test này `vi.doMock` module
 * cổng với một spy BỌC bản THẬT (hành vi giữ nguyên), để khẳng định đúng HÀM nào được gọi.
 */
describe('#8 (vòng rà thứ hai) — hai reset*() gọi ĐÚNG resetGlossaryExchangeGate()', () => {
  it('resetGlossaryManage() gọi resetGlossaryExchangeGate() đúng một lần', async () => {
    vi.resetModules()
    const actualGate =
      await vi.importActual<typeof import('../../src/glossaryExchangeGate')>('../../src/glossaryExchangeGate')
    const resetSpy = vi.fn(actualGate.resetGlossaryExchangeGate)
    vi.doMock('../../src/glossaryExchangeGate', () => ({ ...actualGate, resetGlossaryExchangeGate: resetSpy }))

    const manage = await import('../../src/glossaryManageState')
    manage.resetGlossaryManage()

    expect(resetSpy).toHaveBeenCalledTimes(1)

    vi.doUnmock('../../src/glossaryExchangeGate')
  })

  it('resetGlossaryImport() gọi resetGlossaryExchangeGate() đúng một lần', async () => {
    vi.resetModules()
    const actualGate =
      await vi.importActual<typeof import('../../src/glossaryExchangeGate')>('../../src/glossaryExchangeGate')
    const resetSpy = vi.fn(actualGate.resetGlossaryExchangeGate)
    vi.doMock('../../src/glossaryExchangeGate', () => ({ ...actualGate, resetGlossaryExchangeGate: resetSpy }))

    const importState = await import('../../src/glossaryImportState')
    importState.resetGlossaryImport()

    expect(resetSpy).toHaveBeenCalledTimes(1)

    vi.doUnmock('../../src/glossaryExchangeGate')
  })
})
