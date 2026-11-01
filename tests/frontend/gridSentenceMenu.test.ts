import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

const omittedIds = new Set<number>()

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: async () => {
      const result = await readFixture()
      if (result.loaded === null || omittedIds.size === 0) return result
      const segments = result.loaded.segments.map((s) => (omittedIds.has(s.id) ? { ...s, is_omitted: true } : s))
      return { ...result, loaded: { ...result.loaded, segments } }
    },
    saveSegmentTargets: recordSave,
  }
})

const addTermMock = vi.fn()

vi.mock('../../src/config/glossary', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../src/config/glossary')>()),
  lookupGlossaryTerm: async () => ({ found: 'none', workTierAvailable: false, variantTerm: null }),
  addGlossaryTerm: (...args: unknown[]) => addTermMock(...args),
}))

const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

const mounted: { unmount: () => void }[] = []
const detachers: (() => void)[] = []

afterEach(() => {
  for (const w of mounted.splice(0)) w.unmount()
  for (const d of detachers.splice(0)) d()
  document.body.innerHTML = ''
  omittedIds.clear()
  vi.restoreAllMocks()
})

beforeEach(() => {
  resetRecorder()
})

async function mountGrid(options: { realStrip?: boolean } = {}) {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const commands = await import('../../src/commands')
  const menuState = await import('../../src/panels/sentenceMenuState')
  const strip = await import('../../src/glossaryQuickAddState')
  const selection = await import('../../src/panels/selectionContract')
  const calledOn: Record<string, (number | null)[]> = {}
  const spy = <A extends unknown[], R>(name: string, result: R) =>
    vi.fn((..._args: A): R => {
      ;(calledOn[name] ??= []).push(state.editorCaretSegmentId.value)
      return result
    })
  const deps = {
    confirmSegment: spy('confirmSegment', undefined),
    mergeSegments: spy('mergeSegments', undefined),
    splitSegment: spy('splitSegment', undefined),
    clearSourceCuts: spy('clearSourceCuts', undefined),
    goToNextUntranslated: spy('goToNextUntranslated', true),
    setSegmentOmitted: spy<[boolean], void>('setSegmentOmitted', undefined),
    setSegmentParagraphEnd: spy<[boolean], void>('setSegmentParagraphEnd', undefined),
    splitChapter: spy('splitChapter', undefined),
    openSegmentHistory: spy('openSegmentHistory', undefined),
    openGlossaryQuickAdd: spy<[string], void>('openGlossaryQuickAdd', undefined),
    currentSelectionForGlossary: spy('currentSelectionForGlossary', ''),
  }
  const realStrip =
    options.realStrip === true
      ? { openGlossaryQuickAdd: strip.openGlossaryQuickAdd, currentSelectionForGlossary: selection.currentSelectionTextForGlossaryQuickAdd }
      : {}
  commands.installCommands({
    isMac: true,
    setMode: () => {},
    ...deps,
    ...realStrip,
    openSentenceMenu: () => {
      menuState.openSentenceMenu()
    },
  })
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
  const wrapper = mount(GridPanel, {
    props: { params: {} } as never,
    global: { stubs: STUBS },
    attachTo: document.body,
  })
  mounted.push(wrapper)
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { state, commands, menuState, strip, deps, calledOn, wrapper }
}

const cell = (col: 'src' | 'tgt', id: number): HTMLElement => {
  const el = document.querySelector<HTMLElement>(`[data-col="${col}"][data-segment-id="${String(id)}"]`)
  if (el === null) throw new Error(`no ${col} cell for ${String(id)}`)
  return el
}
const numCell = (id: number): HTMLElement => {
  const el = document.querySelector<HTMLElement>(`[data-num-segment-id="${String(id)}"]`)
  if (el === null) throw new Error(`no number cell for ${String(id)}`)
  return el
}
const menuEl = (): HTMLElement | null => document.querySelector<HTMLElement>('[data-sentence-menu]')
const items = (): HTMLButtonElement[] => Array.from(document.querySelectorAll<HTMLButtonElement>('[data-sentence-menu] button'))
const itemByText = (text: string): HTMLButtonElement => {
  const found = items().find((b) => b.textContent.includes(text))
  if (found === undefined) throw new Error(`no menu item "${text}"`)
  return found
}

function press(el: Element, init: MouseEventInit = {}): MouseEvent {
  const base = { button: 2, bubbles: true, cancelable: true, clientX: 30, clientY: 40, ...init }
  const down = new MouseEvent('mousedown', base)
  el.dispatchEvent(down)
  el.dispatchEvent(new MouseEvent('contextmenu', base))
  el.dispatchEvent(new MouseEvent('mouseup', base))
  return down
}

function key(target: Element, init: KeyboardEventInit): KeyboardEvent {
  const event = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init })
  target.dispatchEvent(event)
  return event
}

describe('menu câu của Lưới — mỗi id một mục chạy trên câu được nhắm', () => {
  const CASES: readonly (readonly [id: string, label: string, dep: string, setup: 'cut' | 'omitted' | null])[] = [
    ['editor.confirm_segment', 'Xác nhận', 'confirmSegment', null],
    ['editor.merge_segments', 'Gộp lên', 'mergeSegments', null],
    ['editor.split_segment', 'Tách câu', 'splitSegment', null],
    ['editor.clear_source_cuts', 'Bỏ điểm cắt', 'clearSourceCuts', 'cut'],
    ['editor.next_untranslated', 'Câu chưa dịch kế', 'goToNextUntranslated', null],
    ['editor.omit_segment', 'Cắt bỏ câu', 'setSegmentOmitted', null],
    ['editor.restore_segment', 'Đưa câu trở lại', 'setSegmentOmitted', 'omitted'],
    ['editor.end_target_paragraph', 'Kết thúc đoạn', 'setSegmentParagraphEnd', null],
    ['editor.join_target_paragraph', 'Nối với đoạn tiếp theo', 'setSegmentParagraphEnd', null],
    ['editor.split_chapter', 'Tách Chương tại đây', 'splitChapter', null],
    ['history.open', 'Lịch sử phiên bản', 'openSegmentHistory', null],
    ['glossary.add_term', 'Thêm thuật ngữ', 'openGlossaryQuickAdd', null],
  ]

  const FLAG_ARG: Readonly<Record<string, boolean>> = {
    'editor.omit_segment': true,
    'editor.restore_segment': false,
    'editor.end_target_paragraph': true,
    'editor.join_target_paragraph': false,
  }

  it.each(CASES)('%s: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13', async (id: string, label: string, dep: string, setup: 'cut' | 'omitted' | null) => {
    if (setup === 'omitted') omittedIds.add(13)
    const { state, deps, calledOn } = await mountGrid()
    const target = id === 'editor.join_target_paragraph' ? 12 : 13
    state.setEditorCaret(11)
    if (setup === 'cut') state.setEditorSourceCut(target, 0)

    press(cell('src', target))
    await flushPromises()
    expect(menuEl()).not.toBeNull()

    itemByText(label).click()
    await flushPromises()

    expect((deps as Record<string, ReturnType<typeof vi.fn>>)[dep]).toHaveBeenCalledTimes(1)
    expect(calledOn[dep]).toEqual([target])
    if (id in FLAG_ARG) expect((deps as Record<string, ReturnType<typeof vi.fn>>)[dep].mock.calls[0]).toEqual([FLAG_ARG[id]])
    expect(menuEl()).toBeNull()
  })

  it('không mục nào nằm sai nhóm: Gộp lên tắt ở câu đầu Chương, Tách câu tắt khi điểm cắt ở câu khác và title nói câu nào', async () => {
    const { state } = await mountGrid()
    state.setEditorCaret(13)
    state.setEditorSourceCut(12, 1)

    press(cell('src', 11))
    await flushPromises()

    expect(itemByText('Gộp lên').disabled).toBe(true)
    const split = itemByText('Tách câu')
    expect(split.disabled).toBe(true)
    expect(split.title).toContain('câu 2')
    expect(itemByText('Bỏ điểm cắt').disabled).toBe(false)
  })

  it('Tách câu mở khi điểm cắt thuộc chính câu được nhắm', async () => {
    const { state } = await mountGrid()
    state.setEditorSourceCut(12, 1)
    press(cell('src', 12))
    await flushPromises()
    expect(itemByText('Tách câu').disabled).toBe(false)
  })

  it('tiêu đề là Câu N theo vị trí, và không có mục Bỏ điểm cắt khi chưa có điểm cắt', async () => {
    await mountGrid()
    press(cell('tgt', 12))
    await flushPromises()
    expect(menuEl()?.textContent).toContain('Câu 2')
    expect(items().some((b) => b.textContent.includes('Bỏ điểm cắt'))).toBe(false)
  })
})

describe('nhắm câu bằng nhấn phụ', () => {
  it.each([
    ['nguyên văn', (id: number) => cell('src', id)],
    ['số câu', (id: number) => numCell(id)],
  ] as const)('%s: mousedown bị chặn và caret sang câu bị nhấn ngay trước contextmenu', async (_name, pick) => {
    const { state } = await mountGrid()
    state.setEditorCaret(11)

    const el = pick(12)
    const down = new MouseEvent('mousedown', { button: 2, bubbles: true, cancelable: true })
    el.dispatchEvent(down)

    expect(down.defaultPrevented).toBe(true)
    expect(state.editorCaretSegmentId.value).toBe(12)
  })

  it('ô bản dịch: mousedown không bị chặn, lệnh vẫn chạy trên câu bị nhấn', async () => {
    const { state, calledOn } = await mountGrid()
    state.setEditorCaret(11)

    const down = press(cell('tgt', 12))
    await flushPromises()
    itemByText('Xác nhận').click()

    expect(down.defaultPrevented).toBe(false)
    expect(calledOn.confirmSegment).toEqual([12])
  })

  it('caret rơi về null giữa mousedown và contextmenu thì mở vẫn nhắm đúng câu', async () => {
    const { state, calledOn } = await mountGrid()
    state.setEditorCaret(11)

    const base = { button: 2, bubbles: true, cancelable: true }
    cell('src', 12).dispatchEvent(new MouseEvent('mousedown', base))
    state.setEditorCaret(null)
    cell('src', 12).dispatchEvent(new MouseEvent('contextmenu', base))
    await flushPromises()
    itemByText('Xác nhận').click()

    expect(calledOn.confirmSegment).toEqual([12])
  })

  it('không có câu dưới con trỏ: không mở, không nhắm', async () => {
    const { state } = await mountGrid()
    state.setEditorCaret(11)

    const rule = document.querySelector<HTMLElement>('.col-rule .cell')
    expect(rule).not.toBeNull()
    if (rule === null) return
    const base = { button: 2, bubbles: true, cancelable: true }
    const down = new MouseEvent('mousedown', base)
    rule.dispatchEvent(down)
    const menu = new MouseEvent('contextmenu', base)
    rule.dispatchEvent(menu)
    await flushPromises()

    expect(menuEl()).toBeNull()
    expect(menu.defaultPrevented).toBe(true)
    expect(down.defaultPrevented).toBe(false)
    expect(state.editorCaretSegmentId.value).toBe(11)
  })

  it('không có cú nhấn (phím Menu): mở tại câu có caret; caret null thì không mở', async () => {
    const { state, menuState } = await mountGrid()
    const none = new MouseEvent('contextmenu', { bubbles: true, cancelable: true })
    cell('tgt', 12).dispatchEvent(none)
    await flushPromises()
    expect(menuEl()).toBeNull()

    state.setEditorCaret(12)
    cell('tgt', 12).dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true }))
    await flushPromises()
    expect(menuState.sentenceMenu.value?.segmentId).toBe(12)
    expect(menuEl()).not.toBeNull()
  })

  it('một cú nhấn cũ không còn là đích: mở sau đó rơi về câu có caret, không dùng toạ độ cũ', async () => {
    const { state, menuState } = await mountGrid()
    state.setEditorCaret(11)
    cell('src', 12).dispatchEvent(new MouseEvent('mousedown', { button: 2, bubbles: true, cancelable: true, clientX: 30 }))
    state.setEditorCaret(11)
    const now = vi.spyOn(performance, 'now').mockReturnValue(performance.now() + 5000)

    expect(menuState.openSentenceMenu()).toBe(true)
    now.mockRestore()

    expect(menuState.sentenceMenu.value).toMatchObject({ segmentId: 11, x: 0 })
  })

  it('Ctrl+nút trái trên macOS đi đúng đường nhấn phụ', async () => {
    vi.spyOn(navigator, 'platform', 'get').mockReturnValue('MacIntel')
    const { state, calledOn } = await mountGrid()
    state.setEditorCaret(11)

    const down = press(cell('src', 12), { button: 0, ctrlKey: true })
    await flushPromises()
    itemByText('Xác nhận').click()

    expect(down.defaultPrevented).toBe(true)
    expect(calledOn.confirmSegment).toEqual([12])
  })

  it('isSecondaryPress: nút phải; Ctrl+nút trái chỉ trên macOS', async () => {
    const { menuState } = await mountGrid()
    const { isSecondaryPress } = menuState
    expect(isSecondaryPress({ button: 2, ctrlKey: false }, { isMac: false })).toBe(true)
    expect(isSecondaryPress({ button: 0, ctrlKey: true }, { isMac: true })).toBe(true)
    expect(isSecondaryPress({ button: 0, ctrlKey: true }, { isMac: false })).toBe(false)
    expect(isSecondaryPress({ button: 0, ctrlKey: false }, { isMac: true })).toBe(false)
  })
})

describe('đóng menu', () => {
  it('Esc: caret và vùng chọn về lúc trước, Esc bị nuốt, điểm cắt còn nguyên', async () => {
    const { state, menuState, deps } = await mountGrid()
    state.setEditorCaret(11)
    state.setEditorSourceCut(12, 1)
    const own = cell('tgt', 11)
    const selection = window.getSelection()
    selection?.collapse(own.firstChild ?? own, 3)

    press(cell('src', 12))
    await flushPromises()
    expect(state.editorCaretSegmentId.value).toBe(12)

    const esc = key(cell('tgt', 12), { key: 'Escape', code: 'Escape' })
    await flushPromises()

    expect(esc.defaultPrevented).toBe(true)
    expect(menuState.sentenceMenu.value).toBeNull()
    expect(state.editorCaretSegmentId.value).toBe(11)
    expect(own.contains(window.getSelection()?.anchorNode ?? null)).toBe(true)
    expect(window.getSelection()?.anchorOffset).toBe(3)
    expect(state.editorSourceCut.value?.segmentId).toBe(12)
    expect(deps.clearSourceCuts).not.toHaveBeenCalled()
  })

  it('caret null lúc mở: vùng chọn do WebKit bôi thu về caret', async () => {
    const { state } = await mountGrid()
    state.setEditorCaret(null)
    const target = cell('tgt', 12)
    press(cell('src', 12))
    await flushPromises()
    window.getSelection()?.setBaseAndExtent(target, 0, target, 1)
    expect(window.getSelection()?.isCollapsed).toBe(false)

    document.body.dispatchEvent(new MouseEvent('mousedown', { button: 0, bubbles: true }))
    await flushPromises()

    expect(menuEl()).toBeNull()
    expect(window.getSelection()?.isCollapsed).toBe(true)
  })

  it('bấm ngoài, cuộn Lưới và phím khác đều đóng; phím khác không bị nuốt', async () => {
    const { state, wrapper } = await mountGrid()
    state.setEditorCaret(11)

    press(cell('src', 12))
    await flushPromises()
    document.body.dispatchEvent(new MouseEvent('mousedown', { button: 0, bubbles: true }))
    await flushPromises()
    expect(menuEl()).toBeNull()
    expect(state.editorCaretSegmentId.value).toBe(11)

    press(cell('src', 12))
    await flushPromises()
    await wrapper.find('.grid-scroll').trigger('scroll')
    await flushPromises()
    expect(menuEl()).toBeNull()

    press(cell('src', 12))
    await flushPromises()
    const typed = key(cell('tgt', 11), { key: 'a', code: 'KeyA' })
    await flushPromises()
    expect(menuEl()).toBeNull()
    expect(typed.defaultPrevented).toBe(false)
    expect(state.editorCaretSegmentId.value).toBe(11)
  })

  it('bấm vào trong menu không đóng nó, và không lấy tiêu điểm', async () => {
    await mountGrid()
    press(cell('src', 12))
    await flushPromises()
    const down = new MouseEvent('mousedown', { button: 0, bubbles: true, cancelable: true })
    itemByText('Xác nhận').dispatchEvent(down)
    expect(down.defaultPrevented).toBe(true)
    expect(menuEl()).not.toBeNull()
  })

  it('mở chặn mọi hợp âm đã đăng ký, kể cả cú phím đóng nó', async () => {
    const { commands, menuState, deps } = await mountGrid()
    detachers.push(
      commands.attachKeyboard(window, { isBlocked: menuState.sentenceMenuIsOpen }),
    )
    press(cell('src', 12))
    await flushPromises()

    key(cell('tgt', 11), { key: 'm', code: 'KeyM', metaKey: true })
    await flushPromises()
    expect(deps.mergeSegments).not.toHaveBeenCalled()
    expect(menuEl()).toBeNull()

    key(cell('tgt', 11), { key: 'm', code: 'KeyM', metaKey: true })
    expect(deps.mergeSegments).toHaveBeenCalledTimes(1)
  })

  it('đổi Chương khi mở thì menu đóng', async () => {
    const { state, menuState } = await mountGrid()
    press(cell('src', 12))
    await flushPromises()
    expect(menuEl()).not.toBeNull()

    state.resetEditorPanel()
    await flushPromises()

    expect(menuState.sentenceMenu.value).toBeNull()
    expect(menuEl()).toBeNull()
  })
})

describe('sau khi chọn', () => {
  it('caret còn ở câu nhắm mà tiêu điểm ở ô khác: tiêu điểm vào ô câu đó', async () => {
    const { state } = await mountGrid()
    state.setEditorCaret(11)
    const focused: Element[] = []
    vi.spyOn(HTMLElement.prototype, 'focus').mockImplementation(function (this: HTMLElement) {
      focused.push(this)
    })

    press(cell('src', 13))
    await flushPromises()
    focused.length = 0
    itemByText('Cắt bỏ câu').click()
    await flushPromises()

    expect(focused).toContain(cell('tgt', 13))
  })

  it('trước lúc chọn không có focus() nào lên ô bản dịch của câu nhắm', async () => {
    const { state } = await mountGrid()
    state.setEditorCaret(11)
    const focused: Element[] = []
    vi.spyOn(HTMLElement.prototype, 'focus').mockImplementation(function (this: HTMLElement) {
      focused.push(this)
    })

    press(cell('src', 13))
    await flushPromises()

    expect(focused).not.toContain(cell('tgt', 13))
  })
})

describe('mở bằng hợp âm tự gán', () => {
  it('cú phím mở menu không bị chính menu đóng ngay trong cùng keydown', async () => {
    const { state, commands, menuState } = await mountGrid()
    expect(commands.applyBindings({ 'editor.sentence_menu.open': ['Mod+Alt+Shift+K'] })).toEqual({ ok: true })
    detachers.push(commands.attachKeyboard(window, { isBlocked: menuState.sentenceMenuIsOpen }))
    state.setEditorCaret(11)

    key(cell('tgt', 11), { key: 'k', code: 'KeyK', metaKey: true, altKey: true, shiftKey: true })
    await flushPromises()

    expect(menuState.sentenceMenu.value?.segmentId).toBe(11)
    expect(menuEl()).not.toBeNull()
  })
})

describe('sau khi chọn — vùng chọn', () => {
  it('tiêu điểm đã ở ô bản dịch của câu nhắm và có vùng bôi đen trong đó: thu về caret ở cuối vùng', async () => {
    const { state } = await mountGrid()
    const own = cell('tgt', 12)
    state.setEditorCaret(12)
    press(own)
    await flushPromises()
    own.focus()
    const text = own.firstChild
    expect(text).not.toBeNull()
    window.getSelection()?.setBaseAndExtent(text as Node, 2, text as Node, 6)
    expect(window.getSelection()?.isCollapsed).toBe(false)

    itemByText('Cắt bỏ câu').click()
    await flushPromises()

    expect(window.getSelection()?.isCollapsed).toBe(true)
    expect(window.getSelection()?.anchorOffset).toBe(6)
  })

  it('vùng chọn nằm ngoài ô câu nhắm (nguyên văn của chính câu đang có tiêu điểm): đặt lại caret vào ô bản dịch', async () => {
    const { state } = await mountGrid()
    const own = cell('tgt', 12)
    state.setEditorCaret(12)
    own.focus()
    press(cell('src', 12))
    await flushPromises()
    const src = cell('src', 12)
    window.getSelection()?.setBaseAndExtent(src, 0, src, 1)
    const focused: Element[] = []
    vi.spyOn(HTMLElement.prototype, 'focus').mockImplementation(function (this: HTMLElement) {
      focused.push(this)
    })

    itemByText('Cắt bỏ câu').click()
    await flushPromises()

    expect(focused).toContain(own)
  })
})

describe('mục Lịch sử phiên bản', () => {
  it('nguyên văn câu đang có tiêu điểm bị bôi đen: vùng chọn bị xoá và ô bản dịch không bị ép lấy tiêu điểm', async () => {
    const { state } = await mountGrid()
    const own = cell('tgt', 12)
    state.setEditorCaret(12)
    own.focus()
    press(cell('src', 12))
    await flushPromises()
    const src = cell('src', 12)
    window.getSelection()?.setBaseAndExtent(src, 0, src, 1)
    expect(window.getSelection()?.isCollapsed).toBe(false)
    const focused: Element[] = []
    vi.spyOn(HTMLElement.prototype, 'focus').mockImplementation(function (this: HTMLElement) {
      focused.push(this)
    })

    itemByText('Lịch sử phiên bản').click()
    expect(window.getSelection()?.rangeCount).toBe(0)
    await flushPromises()

    expect(focused).not.toContain(own)
    expect(menuEl()).toBeNull()
  })

  it('vùng chọn trong ô bản dịch bị xoá để phím gõ khi lớp phủ mở không rơi xuống ô bên dưới', async () => {
    const { state } = await mountGrid()
    const own = cell('tgt', 12)
    own.textContent = 'bản dịch'
    state.setEditorCaret(12)
    own.focus()
    press(own)
    await flushPromises()
    window.getSelection()?.collapse(own.firstChild as Node, 3)

    itemByText('Lịch sử phiên bản').click()

    expect(window.getSelection()?.rangeCount).toBe(0)
  })

  it('mục mang dấu mở lớp phủ, mục thường thì không', async () => {
    await mountGrid()
    press(cell('src', 13))
    await flushPromises()
    expect(itemByText('Lịch sử phiên bản').hasAttribute('data-opens-overlay')).toBe(true)
    expect(itemByText('Cắt bỏ câu').hasAttribute('data-opens-overlay')).toBe(false)
  })
})

describe('vị trí', () => {
  it('kẹp vào khung nhìn khi vị trí gốc nằm ngoài nó', async () => {
    const { menuState } = await mountGrid()
    const size = { width: 294, height: 300 }
    const viewport = { width: 1000, height: 800 }
    expect(menuState.placeMenu({ x: 1500, y: 100 }, size, viewport)).toEqual({ x: 706, y: 100 })
    expect(menuState.placeMenu({ x: 100, y: 5000 }, size, viewport)).toEqual({ x: 100, y: 500 })
    expect(menuState.placeMenu({ x: -50, y: -10 }, size, viewport)).toEqual({ x: 0, y: 0 })
  })


  it('lật vào trong khi sát mép phải và mép dưới', async () => {
    const { menuState } = await mountGrid()
    const size = { width: 294, height: 300 }
    const viewport = { width: 1000, height: 800 }
    expect(menuState.placeMenu({ x: 100, y: 100 }, size, viewport)).toEqual({ x: 100, y: 100 })
    expect(menuState.placeMenu({ x: 900, y: 100 }, size, viewport)).toEqual({ x: 606, y: 100 })
    expect(menuState.placeMenu({ x: 100, y: 700 }, size, viewport)).toEqual({ x: 100, y: 400 })
  })
})

function selectWord(el: Element): string {
  const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT)
  for (let node = walker.nextNode(); node !== null; node = walker.nextNode()) {
    const data = node.textContent ?? ''
    const start = data.search(/\S/)
    if (start < 0) continue
    window.getSelection()?.setBaseAndExtent(node, start, node, start + 1)
    return window.getSelection()?.toString() ?? ''
  }
  throw new Error('cell has no text')
}

describe('mục Thêm thuật ngữ', () => {
  it('nằm giữa nhóm 1 và nhóm 3, ngay trước Lịch sử phiên bản', async () => {
    await mountGrid()
    press(cell('src', 12))
    await flushPromises()
    const rows = Array.from(document.querySelectorAll('[data-sentence-menu] > li')).map((li) =>
      li.getAttribute('role') === 'separator' ? '|' : (li.querySelector('button span')?.textContent ?? ''),
    )
    const at = rows.indexOf('Thêm thuật ngữ')
    expect(rows[at - 1]).toBe('|')
    expect(rows[at + 1]).toBe('Lịch sử phiên bản')
    expect(rows[at + 2]).toBe('|')
    expect(rows[at + 3]).toBe('Cắt bỏ câu')
  })

  it.each(['src', 'tgt'] as const)('trong vùng bôi ở cột %s: ô thuật ngữ điền đúng từ bôi', async (col) => {
    const { strip } = await mountGrid({ realStrip: true })
    if (col === 'tgt') cell('tgt', 12).textContent = 'bản dịch'
    const picked = selectWord(cell(col, 12))
    expect(picked).not.toBe('')

    press(cell(col, 12))
    await flushPromises()
    itemByText('Thêm thuật ngữ').click()
    await flushPromises()

    expect(strip.quickAddIsOpen.value).toBe(true)
    expect(strip.quickAddSourceTerm.value).toBe(picked)
  })

  it('ô số câu khi có từ bôi ở câu khác: vùng chọn bị xoá và ô thuật ngữ trống', async () => {
    const { strip } = await mountGrid({ realStrip: true })
    selectWord(cell('src', 11))

    press(numCell(13))
    await flushPromises()
    expect(window.getSelection()?.rangeCount).toBe(0)
    itemByText('Thêm thuật ngữ').click()
    await flushPromises()

    expect(strip.quickAddIsOpen.value).toBe(true)
    expect(strip.quickAddSourceTerm.value).toBe('')
  })

  it('dải đang mở: mục tắt và title nói lý do', async () => {
    const { strip } = await mountGrid({ realStrip: true })
    strip.openGlossaryQuickAdd('từ')
    press(cell('src', 12))
    await flushPromises()

    const item = itemByText('Thêm thuật ngữ')
    expect(item.disabled).toBe(true)
    expect(item.title).toContain('đang mở')
  })

  it('dải mở ra trong lúc menu đang mở: mục vẫn bật nên cú bấm thật đóng được menu', async () => {
    const { strip } = await mountGrid({ realStrip: true })
    press(cell('src', 12))
    await flushPromises()
    const item = itemByText('Thêm thuật ngữ')

    strip.openGlossaryQuickAdd('từ')
    await flushPromises()
    expect(item.disabled).toBe(false)

    item.click()
    await flushPromises()
    expect(menuEl()).toBeNull()
  })

  it('mục mang dấu riêng của dải, không mang dấu mở lớp phủ', async () => {
    await mountGrid()
    press(cell('src', 13))
    await flushPromises()
    const item = itemByText('Thêm thuật ngữ')
    expect(item.hasAttribute('data-opens-glossary-strip')).toBe(true)
    expect(item.hasAttribute('data-opens-overlay')).toBe(false)
  })

  describe.each([
    ['nguyên văn', (id: number) => cell('src', id)],
    ['số câu', (id: number) => numCell(id)],
    ['bản dịch', (id: number) => cell('tgt', id)],
  ] as const)('đóng dải sau khi nhắm câu B ở cột %s', (_name, pick) => {
    const CLOSERS = [
      ['Esc', (strip: { closeGlossaryQuickAdd: () => void }) => strip.closeGlossaryQuickAdd()],
      ['Lưu', async (strip: { saveGlossaryQuickAdd: () => Promise<boolean> }) => void (await strip.saveGlossaryQuickAdd())],
    ] as const

    it.each(CLOSERS)('%s: caret về câu A đúng vị trí, tiêu điểm ở ô A, chữ của B nguyên vẹn', async (_closer, close) => {
      addTermMock.mockResolvedValue({ error: null })
      const { state, strip } = await mountGrid({ realStrip: true })
      const own = cell('tgt', 11)
      own.textContent = 'bản dịch của A'
      const other = cell('tgt', 12)
      const otherText = other.textContent
      state.setEditorCaret(11)
      own.focus()
      window.getSelection()?.collapse(own.firstChild as Node, 5)

      press(pick(12))
      await flushPromises()
      // A press re-asserts focus one frame later; a late frame would pull the caret back to B after the strip closes.
      await new Promise((done) => requestAnimationFrame(() => setTimeout(done, 0)))
      itemByText('Thêm thuật ngữ').click()
      await flushPromises()
      expect(strip.quickAddIsOpen.value).toBe(true)
      strip.setQuickAddSourceTerm('từ')
      await flushPromises()

      await (close as (s: typeof strip) => unknown)(strip)
      await flushPromises()

      expect(strip.quickAddIsOpen.value).toBe(false)
      expect(state.editorCaretSegmentId.value).toBe(11)
      expect(document.activeElement).toBe(own)
      expect(own.contains(window.getSelection()?.anchorNode ?? null)).toBe(true)
      expect(window.getSelection()?.anchorOffset).toBe(5)
      expect(other.textContent).toBe(otherText)
    })
  })

  it('Chương đổi khi dải mở: đóng dải không ném và caret không trỏ câu của Chương cũ', async () => {
    const { state, strip } = await mountGrid({ realStrip: true })
    state.setEditorCaret(11)
    press(cell('src', 12))
    await flushPromises()
    itemByText('Thêm thuật ngữ').click()
    await flushPromises()

    state.resetEditorPanel()
    expect(() => strip.closeGlossaryQuickAdd()).not.toThrow()

    expect(state.editorCaretSegmentId.value).toBeNull()
  })

  it('dải không mở được (lệnh không chạy): menu vẫn trả trạng thái như đóng không chọn', async () => {
    const { state } = await mountGrid()
    state.setEditorCaret(11)
    press(cell('src', 12))
    await flushPromises()
    expect(state.editorCaretSegmentId.value).toBe(12)

    itemByText('Thêm thuật ngữ').click()
    await flushPromises()

    expect(state.editorCaretSegmentId.value).toBe(11)
  })
})
