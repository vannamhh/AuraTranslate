/**
 * State của dải "Thêm thuật ngữ" — Story 3.3, FR48.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ PHẠM VI — vế THUẦN và vế DOM tối thiểu happy-dom trả lời được; KHÔNG vế thị giác
 * ─────────────────────────────────────────────────────────────────────────────
 * `quickAddModeFor`/`quickAddLookupHasLoaded` là hai hàm THUẦN — kiểm trực tiếp, không
 * mount gì cả. Focus/`Selection` là DOM thật mà happy-dom mô phỏng được ở mức "phần tử nào
 * đang giữ tiêu điểm", nên vế "Esc trả lại focus đã lưu" kiểm được ở đây. Vế THỊ GIÁC (dải
 * đẩy nội dung lên, không phủ — xem CSS của `GlossaryQuickAdd.vue`) và vế VÙNG CHỌN trên
 * **engine thật** (WKWebView/Chromium, không phải mô phỏng `Selection` của happy-dom) thuộc
 * bàn đo tay — spec nói thẳng điều này ở §Verification.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

/** Bản giả của `config/glossary.ts` — biên IPC, cùng khuôn `statusBar.test.ts`. */
const lookupMock = vi.fn()
const addMock = vi.fn()
const updateMock = vi.fn()

vi.mock('../../src/config/glossary', () => ({
  lookupGlossaryTerm: (term: string) => lookupMock(term),
  addGlossaryTerm: (...args: unknown[]) => addMock(...args),
  updateGlossaryTerm: (...args: unknown[]) => updateMock(...args),
}))

/** Nạp lại module mỗi ca — state của dải là module-level singleton (cùng khuôn `statusBar.test.ts`). */
async function freshState() {
  vi.resetModules()
  lookupMock.mockReset()
  addMock.mockReset()
  updateMock.mockReset()
  return import('../../src/glossaryQuickAddState')
}

beforeEach(() => {
  document.body.innerHTML = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('quickAddModeFor — chế độ là hàm THUẦN của kết quả tra, không một cờ đặt lúc mở', () => {
  it('lookup === null (chưa từng tra) ⇒ unknown, KHÔNG "add"', async () => {
    const { quickAddModeFor } = await freshState()
    expect(quickAddModeFor(null)).toBe('unknown')
  })

  it("found === 'unknown' (đang bay HOẶC lượt tra trượt) ⇒ unknown", async () => {
    const { quickAddModeFor } = await freshState()
    expect(quickAddModeFor({ found: 'unknown', error: null })).toBe('unknown')
  })

  it("found === 'none' ⇒ add", async () => {
    const { quickAddModeFor } = await freshState()
    expect(quickAddModeFor({ found: 'none', workTierAvailable: true })).toBe('add')
  })

  it("found === 'entry' ⇒ edit, BẤT KỂ translation là null (chờ chốt vẫn mở SỬA)", async () => {
    const { quickAddModeFor } = await freshState()
    expect(
      quickAddModeFor({
        found: 'entry',
        workTierAvailable: true,
        entry: {
          tier: 'global',
          id: 1,
          source_term: '青丘',
          translation: null,
          note: '',
          category: 'place',
          term_origin: 'manual',
          created_at: '2026-08-20T00:00:00.000Z',
        },
      }),
    ).toBe('edit')
  })
})

describe('quickAddLookupHasLoaded — vị từ "…HasLoaded", BA trạng thái', () => {
  it('null (chưa tra lần nào) ⇒ false', async () => {
    const { quickAddLookupHasLoaded } = await freshState()
    expect(quickAddLookupHasLoaded(null)).toBe(false)
  })

  it("found === 'unknown' (đang bay/trượt) ⇒ false", async () => {
    const { quickAddLookupHasLoaded } = await freshState()
    expect(quickAddLookupHasLoaded({ found: 'unknown', error: null })).toBe(false)
  })

  it("found === 'none' HOẶC 'entry' (đã tra xong) ⇒ true", async () => {
    const { quickAddLookupHasLoaded } = await freshState()
    expect(quickAddLookupHasLoaded({ found: 'none', workTierAvailable: false })).toBe(true)
  })
})

describe('openGlossaryQuickAdd — ô nguồn RỖNG khi không có vùng chọn, và vẫn phát một lượt tra', () => {
  it('mở với chuỗi rỗng ⇒ ô nguồn rỗng, và lượt tra vẫn chạy (để biết work_tier_available)', async () => {
    const { openGlossaryQuickAdd, quickAddSourceTerm, quickAddIsOpen } = await freshState()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    openGlossaryQuickAdd('')

    expect(quickAddIsOpen.value).toBe(true)
    expect(quickAddSourceTerm.value).toBe('')
    expect(lookupMock).toHaveBeenCalledWith('')
  })

  it('mở với một cụm chưa có trong Glossary ⇒ chế độ THÊM sau khi lượt tra về', async () => {
    const { openGlossaryQuickAdd, quickAddMode } = await freshState()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    openGlossaryQuickAdd('慕容')
    // Ngay sau khi mở, lượt tra CHƯA về — mode phải là 'unknown', không "add" đoán trước.
    expect(quickAddMode.value).toBe('unknown')

    await Promise.resolve()
    await Promise.resolve()

    expect(quickAddMode.value).toBe('add')
  })

  it('mở với một cụm ĐÃ có trong Glossary ⇒ chế độ SỬA, tầng GHIM theo mục tìm được, các ô điền sẵn', async () => {
    const { openGlossaryQuickAdd, quickAddMode, quickAddEffectiveTier, quickAddTranslation, quickAddNote, quickAddCategory } =
      await freshState()
    lookupMock.mockResolvedValue({
      found: 'entry',
      workTierAvailable: true,
      entry: {
        tier: 'work',
        id: 7,
        source_term: '慕容',
        translation: 'Mộ Dung',
        note: 'ghi chu',
        category: 'person',
        term_origin: 'manual',
        created_at: '2026-08-20T00:00:00.000Z',
      },
    })

    openGlossaryQuickAdd('慕容')
    await Promise.resolve()
    await Promise.resolve()

    expect(quickAddMode.value).toBe('edit')
    expect(quickAddEffectiveTier.value).toBe('work')
    expect(quickAddTranslation.value).toBe('Mộ Dung')
    expect(quickAddNote.value).toBe('ghi chu')
    expect(quickAddCategory.value).toBe('person')
  })
})

describe('setQuickAddSourceTerm — đổi ô nguồn sau khi dải đã mở phát lại lượt tra (AC)', () => {
  it('lượt tra CHẬM của cụm CŨ không được ghi đè lên câu trả lời của cụm MỚI (đua round-trip IPC)', async () => {
    let resolveSlow: (v: unknown) => void = () => {}
    const slow = new Promise((resolve) => {
      resolveSlow = resolve
    })
    const { openGlossaryQuickAdd, setQuickAddSourceTerm, quickAddMode } = await freshState()

    lookupMock.mockImplementationOnce(() => slow) // lượt 1 — 'A', CHẬM.
    lookupMock.mockResolvedValueOnce({ found: 'none', workTierAvailable: true }) // lượt 2 — 'B', NHANH.

    openGlossaryQuickAdd('A')
    setQuickAddSourceTerm('B')

    await Promise.resolve()
    await Promise.resolve()
    expect(quickAddMode.value).toBe('add') // câu trả lời của 'B' đã về.

    // Lượt CHẬM của 'A' về muộn — không được ghi đè.
    resolveSlow({
      found: 'entry',
      workTierAvailable: true,
      entry: {
        tier: 'global',
        id: 1,
        source_term: 'A',
        translation: 'x',
        note: '',
        category: 'other',
        term_origin: 'manual',
        created_at: '2026-08-20T00:00:00.000Z',
      },
    })
    await Promise.resolve()
    await Promise.resolve()

    expect(quickAddMode.value).toBe('add') // vẫn là câu trả lời của 'B', không bị 'A' vượt mặt ngược.
  })
})

describe('đóng dải (Esc hoặc sau khi Lưu) trả lại focus đã lưu lúc mở', () => {
  it('closeGlossaryQuickAdd trả tiêu điểm về đúng phần tử đang giữ nó lúc mở', async () => {
    const anchor = document.createElement('button')
    document.body.appendChild(anchor)
    anchor.focus()
    expect(document.activeElement).toBe(anchor)

    const { openGlossaryQuickAdd, closeGlossaryQuickAdd, quickAddIsOpen } = await freshState()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    openGlossaryQuickAdd('') // lưu `anchor` làm phần tử focus TRƯỚC khi mở.
    closeGlossaryQuickAdd()

    expect(quickAddIsOpen.value).toBe(false)
    expect(document.activeElement).toBe(anchor)
  })

  it('Lưu THÀNH CÔNG cũng đóng dải và trả lại focus — cùng đường với Esc', async () => {
    const anchor = document.createElement('button')
    document.body.appendChild(anchor)
    anchor.focus()

    const { openGlossaryQuickAdd, saveGlossaryQuickAdd, quickAddIsOpen } = await freshState()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })
    addMock.mockResolvedValue({ value: 1, error: null })

    openGlossaryQuickAdd('慕容')
    await Promise.resolve()
    await Promise.resolve()

    const saved = await saveGlossaryQuickAdd()

    expect(saved).toBe(true)
    expect(quickAddIsOpen.value).toBe(false)
    expect(document.activeElement).toBe(anchor)
  })

  it('Lưu TRƯỢT giữ dải MỞ và hiện lỗi — không rỗng im lặng', async () => {
    const { openGlossaryQuickAdd, saveGlossaryQuickAdd, quickAddIsOpen, quickAddSaveError } = await freshState()
    const err = { code: 'store.write_failed', message_key: 'err.store.write_failed', params: {}, retryable: false }
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })
    addMock.mockResolvedValue({ value: null, error: err })

    openGlossaryQuickAdd('慕容')
    await Promise.resolve()
    await Promise.resolve()

    const saved = await saveGlossaryQuickAdd()

    expect(saved).toBe(false)
    expect(quickAddIsOpen.value).toBe(true)
    expect(quickAddSaveError.value).toEqual(err)
  })
})

describe('resetGlossaryQuickAdd — check:panel-refs đòi một đường reset* cho mọi ô nhớ module-level', () => {
  it('vứt toàn bộ state về giá trị khởi tạo', async () => {
    const { openGlossaryQuickAdd, resetGlossaryQuickAdd, quickAddIsOpen, quickAddSourceTerm, quickAddLookup } =
      await freshState()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    openGlossaryQuickAdd('慕容')
    resetGlossaryQuickAdd()

    expect(quickAddIsOpen.value).toBe(false)
    expect(quickAddSourceTerm.value).toBe('')
    expect(quickAddLookup.value).toBe(null)
  })
})

describe('đóng dải trả lại VÙNG CHỌN đã lưu lúc mở, không chỉ tiêu điểm', () => {
  /**
   * ⚠️ Vế thứ hai của AC "Đóng dải bằng Esc hoặc sau khi lưu — focus VÀ vùng chọn cũ được
   * trả lại". Hai `it` trên đã canh vế FOCUS; hai ca dưới đây canh vế `Range`
   * (`glossaryQuickAddState.ts::savedRange`/`restoreFocusAndSelection`), gồm cả nhánh
   * phòng thủ khi node đã rời DOM giữa chừng.
   */
  it('closeGlossaryQuickAdd khôi phục ĐÚNG đoạn văn bản đã bôi đen trước khi mở', async () => {
    const container = document.createElement('div')
    container.textContent = 'Mộ Dung Phục là một nhân vật.'
    document.body.appendChild(container)
    const otherFocusable = document.createElement('input')
    document.body.appendChild(otherFocusable)

    const text = container.firstChild
    if (text === null) throw new Error('câu không có text node')
    const range = document.createRange()
    range.setStart(text, 0)
    range.setEnd(text, 9) // "Mộ Dung P"
    const selection = window.getSelection()
    selection?.removeAllRanges()
    selection?.addRange(range)

    const { openGlossaryQuickAdd, closeGlossaryQuickAdd } = await freshState()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    openGlossaryQuickAdd('Mộ Dung P') // chụp Range HIỆN TẠI lúc mở.

    // Mô phỏng đúng thứ xảy ra thật khi dải mở: tiêu điểm rời khỏi bề mặt nguồn, đi vào ô
    // gõ của dải, và `Selection` của trình duyệt đi theo tiêu điểm mới — vùng chọn cũ
    // không còn hiện trên `window.getSelection()` nữa TRONG LÚC dải đang mở.
    otherFocusable.focus()
    selection?.removeAllRanges()
    expect(window.getSelection()?.toString()).toBe('')

    closeGlossaryQuickAdd()

    expect(window.getSelection()?.toString()).toBe('Mộ Dung P')
  })

  it('Range đã lưu trỏ vào một node RỜI DOM giữa chừng ⇒ đóng dải KHÔNG NÉM (nhánh phòng thủ)', async () => {
    const container = document.createElement('div')
    container.textContent = 'Thanh Khâu là một địa danh.'
    document.body.appendChild(container)

    const text = container.firstChild
    if (text === null) throw new Error('câu không có text node')
    const range = document.createRange()
    range.setStart(text, 0)
    range.setEnd(text, 10) // "Thanh Khâu"
    const selection = window.getSelection()
    selection?.removeAllRanges()
    selection?.addRange(range)

    const { openGlossaryQuickAdd, closeGlossaryQuickAdd, quickAddIsOpen } = await freshState()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    openGlossaryQuickAdd('Thanh Khâu')

    // Câu vừa bôi đen bị xoá khỏi DOM trong lúc dải còn mở (ví dụ: chuyển Chương) — `Range`
    // đã lưu nay trỏ vào một node MỒ CÔI.
    container.remove()

    expect(() => closeGlossaryQuickAdd()).not.toThrow()
    expect(quickAddIsOpen.value).toBe(false)
  })
})

describe('chốt tái nhập — Ice bắt 2026-08-20, ba lỗ cùng một gốc', () => {
  it('mở dải khi ĐANG MỞ ⇒ bỏ qua, KHÔNG ghi đè focus/vùng chọn đã lưu lúc mở lần ĐẦU', async () => {
    const original = document.createElement('button')
    document.body.appendChild(original)
    original.focus()

    const { openGlossaryQuickAdd, closeGlossaryQuickAdd, quickAddIsOpen } = await freshState()
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    openGlossaryQuickAdd('慕容') // lưu `original` làm focus TRƯỚC.
    expect(quickAddIsOpen.value).toBe(true)

    // Mô phỏng đúng thứ xảy ra thật: tiêu điểm nay ở TRONG dải (ô nguồn), rồi `Mod+Alt+G`
    // bị bấm lần NỮA trong khi dải còn mở.
    const insideDial = document.createElement('input')
    document.body.appendChild(insideDial)
    insideDial.focus()
    openGlossaryQuickAdd('青丘') // lượt mở THỨ HAI — phải bị bỏ qua hoàn toàn.

    closeGlossaryQuickAdd()

    // Nếu lượt mở thứ hai không bị chặn, `savedFocusEl` đã bị ghi đè thành `insideDial` —
    // một phần tử của chính dải vừa ẩn — và dòng dưới đây sẽ đỏ.
    expect(document.activeElement).toBe(original)
  })

  it('bấm Lưu hai lần liên tiếp (Enter + chuột gần như cùng lúc) ⇒ chỉ MỘT lượt IPC', async () => {
    const { openGlossaryQuickAdd, saveGlossaryQuickAdd } = await freshState()

    let resolveAdd: (v: unknown) => void = () => {}
    const pending = new Promise((resolve) => {
      resolveAdd = resolve
    })
    addMock.mockReturnValue(pending)
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    openGlossaryQuickAdd('慕容')
    await Promise.resolve()
    await Promise.resolve()

    const first = saveGlossaryQuickAdd() // KHÔNG await — mô phỏng lượt ghi đang bay.
    const second = saveGlossaryQuickAdd() // lượt THỨ HAI trong khi lượt đầu chưa về.

    expect(addMock).toHaveBeenCalledTimes(1)

    resolveAdd({ value: 1, error: null })
    const [firstResult, secondResult] = await Promise.all([first, second])

    expect(firstResult).toBe(true)
    expect(secondResult).toBe(false) // lượt thứ hai bị chốt từ chối, không phải một lượt ghi thật.
    expect(addMock).toHaveBeenCalledTimes(1)
  })

  it('Esc trong lúc một lượt ghi ĐANG BAY ⇒ dải KHÔNG đóng — màn hình không được nói "đã huỷ" trong khi đĩa vẫn đang ghi', async () => {
    const { openGlossaryQuickAdd, saveGlossaryQuickAdd, closeGlossaryQuickAdd, quickAddIsOpen } =
      await freshState()

    let resolveAdd: (v: unknown) => void = () => {}
    const pending = new Promise((resolve) => {
      resolveAdd = resolve
    })
    addMock.mockReturnValue(pending)
    lookupMock.mockResolvedValue({ found: 'none', workTierAvailable: true })

    openGlossaryQuickAdd('慕容')
    await Promise.resolve()
    await Promise.resolve()

    const saving = saveGlossaryQuickAdd() // lượt ghi ĐANG BAY.
    closeGlossaryQuickAdd() // `Esc` giữa lúc đang ghi — phải KHÔNG có hiệu lực.

    expect(quickAddIsOpen.value).toBe(true) // vẫn mở — chưa "huỷ" một lượt ghi đang chạy thật.

    resolveAdd({ value: 1, error: null })
    await saving

    // Sau khi lượt ghi THẬT SỰ xong, `saveGlossaryQuickAdd` tự đóng dải — `Esc` không cần
    // thắng nó, chỉ không được PHÉP thắng nó giữa chừng.
    expect(quickAddIsOpen.value).toBe(false)
  })
})
