import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'

import { resetHeldCaret, focusReturnTargetOnOpen, holdTargetCaret } from '../../src/commands/focus'
import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSave,
    readSegmentHistory: async () => ({
      history: { current_version_id: null, source_text: 'Nguyen van cua cau.', versions: [] },
      error: null,
    }),
  }
})

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue({ state: 'done', not_scanned: 0, filtered: 0 }),
  Channel: class {
    onmessage: (v: unknown) => void = () => {}
  },
}))

let closeToolsMenu: () => void
let toggleToolsMenu: () => void
let closeSourcesMenu: () => void
let toggleSourcesMenu: () => void

beforeAll(async () => {
  ;({ closeToolsMenu, toggleToolsMenu } = await import('../../src/titlebarToolsState'))
  ;({ closeSourcesMenu, toggleSourcesMenu } = await import('../../src/panels/dictSourcesState'))
})

function makeCell() {
  const cell = document.createElement('div')
  cell.setAttribute('contenteditable', 'true')
  cell.setAttribute('tabindex', '0')
  cell.dataset.col = 'tgt'
  cell.dataset.segmentId = '11'
  const text = document.createTextNode('abcdef')
  cell.append(text)
  document.body.append(cell)
  vi.spyOn(cell, 'focus').mockImplementation(() => {
    const range = document.createRange()
    range.setStart(cell, 0)
    range.collapse(true)
    window.getSelection()?.removeAllRanges()
    window.getSelection()?.addRange(range)
    Object.defineProperty(document, 'activeElement', { value: cell, configurable: true })
    cell.dispatchEvent(new FocusEvent('focusin', { bubbles: true }))
  })
  Object.defineProperty(document, 'activeElement', { value: cell, configurable: true })
  return { cell, text }
}

function select(node: Text, from: number, to = from) {
  const range = document.createRange()
  range.setStart(node, from)
  range.setEnd(node, to)
  window.getSelection()?.removeAllRanges()
  window.getSelection()?.addRange(range)
}

function overlayDialog() {
  const dialog = document.createElement('section')
  dialog.setAttribute('role', 'dialog')
  dialog.setAttribute('tabindex', '-1')
  document.body.append(dialog)
  return dialog
}

afterEach(() => {
  resetHeldCaret()
  document.body.innerHTML = ''
  Reflect.deleteProperty(document, 'activeElement')
  vi.restoreAllMocks()
})

function opener(selector = '[data-x-open]') {
  const button = document.createElement('button')
  button.setAttribute(selector.slice(1, -1), '')
  document.body.append(button)
  return button
}

describe('giữ caret khi lớp phủ mở', () => {
  it('mở bằng chuột: vùng chọn bị xoá nên phím không còn đích trong ô', () => {
    const { text } = makeCell()
    select(text, 2)
    focusReturnTargetOnOpen('[data-x-open]')
    expect(window.getSelection()?.rangeCount).toBe(0)
    expect(text.data).toBe('abcdef')
  })

  it('đóng rồi gõ: đích trả tiêu điểm là ô, caret về điểm cũ dù tiêu điểm đang ở nút mở', () => {
    const { cell, text } = makeCell()
    const button = opener()
    select(text, 2)
    Object.defineProperty(document, 'activeElement', { value: button, configurable: true })
    const back = focusReturnTargetOnOpen('[data-x-open]')
    expect(back).toBe(cell)
    back?.focus()
    const selection = window.getSelection()
    expect(selection?.anchorNode).toBe(text)
    expect(selection?.anchorOffset).toBe(2)
  })

  it('mở bằng phím: tiêu điểm đang ở ô, cùng kết quả đường chuột', () => {
    const { cell, text } = makeCell()
    select(text, 5)
    const back = focusReturnTargetOnOpen('[data-x-open]')
    expect(back).toBe(cell)
    back?.focus()
    expect(window.getSelection()?.anchorOffset).toBe(5)
  })

  it('WebKit tự bôi một từ: trả về một điểm sau từ, không đè', () => {
    const { cell, text } = makeCell()
    select(text, 1, 4)
    focusReturnTargetOnOpen('[data-x-open]')
    cell.focus()
    const selection = window.getSelection()
    expect(selection?.isCollapsed).toBe(true)
    expect(selection?.anchorOffset).toBe(4)
  })

  it('mục menu câu: vùng chọn đã bị xoá trước lượt mở, điểm đã giữ đồng bộ vẫn trả', () => {
    const { cell, text } = makeCell()
    select(text, 3)
    holdTargetCaret()
    const back = focusReturnTargetOnOpen('[data-x-open]')
    expect(back).toBe(cell)
    back?.focus()
    expect(window.getSelection()?.anchorOffset).toBe(3)
  })

  it('lồng: lớp phủ trong lớp phủ trả về lớp cha, lớp cha trả về ô', () => {
    const { cell, text } = makeCell()
    select(text, 3)
    const outerBack = focusReturnTargetOnOpen('[data-outer-open]')
    const outer = overlayDialog()
    Object.defineProperty(document, 'activeElement', { value: outer, configurable: true })
    const innerBack = focusReturnTargetOnOpen('[data-inner-open]')
    expect(innerBack).toBe(outer)
    expect(outerBack).toBe(cell)
    outerBack?.focus()
    expect(window.getSelection()?.anchorOffset).toBe(3)
  })

  it('hộp thoại đang mở và tiêu điểm rơi về body: lớp phủ lồng không trả ô nằm sau lớp ngoài', () => {
    const { text } = makeCell()
    const button = opener()
    select(text, 3)
    overlayDialog()
    Object.defineProperty(document, 'activeElement', { value: document.body, configurable: true })
    expect(focusReturnTargetOnOpen('[data-x-open]')).toBe(button)
  })

  it('không có caret trong ô: không giữ gì, trả về nút mở như UX-DR17', () => {
    makeCell()
    const button = opener()
    const other = document.createElement('p')
    other.textContent = 'ngoài ô'
    document.body.append(other)
    select(other.firstChild as Text, 2)
    Object.defineProperty(document, 'activeElement', { value: button, configurable: true })
    expect(focusReturnTargetOnOpen('[data-x-open]')).toBe(button)
    expect(window.getSelection()?.anchorNode).toBe(other.firstChild)
  })

  it('ô đã rời DOM: không trả caret, không ném, rơi về hành vi cũ', () => {
    const { cell, text } = makeCell()
    const button = opener()
    select(text, 2)
    Object.defineProperty(document, 'activeElement', { value: button, configurable: true })
    focusReturnTargetOnOpen('[data-x-open]')
    cell.dataset.col = 'moved'
    const removed = vi.spyOn(document, 'removeEventListener')
    expect(focusReturnTargetOnOpen('[data-x-open]')).toBe(button)
    expect(removed).toHaveBeenCalledWith('focusin', expect.any(Function), true)
    const second = makeCell()
    expect(second.cell.isConnected).toBe(true)
    window.getSelection()?.removeAllRanges()
    second.cell.focus()
    expect(window.getSelection()?.anchorOffset).toBe(0)
  })

  it('tiêu điểm sang ô khác: điểm giữ bỏ, ô khác không bị đặt caret', () => {
    const { text } = makeCell()
    const second = document.createElement('div')
    second.setAttribute('contenteditable', 'true')
    second.dataset.col = 'tgt'
    second.dataset.segmentId = '12'
    second.textContent = 'xyz'
    document.body.append(second)
    select(text, 2)
    focusReturnTargetOnOpen('[data-x-open]')
    second.dispatchEvent(new FocusEvent('focusin', { bubbles: true }))
    expect(window.getSelection()?.rangeCount).toBe(0)
  })
})

describe('menu không phải lớp phủ', () => {
  function menuCell(offset = 3) {
    const made = makeCell()
    const trigger = document.createElement('button')
    trigger.setAttribute('aria-haspopup', 'menu')
    document.body.append(trigger)
    select(made.text, offset)
    return { ...made, trigger }
  }

  it('mở xoá vùng chọn, đóng bằng cú bấm ra ngoài trả điểm mà không đụng tiêu điểm', () => {
    vi.useFakeTimers()
    const { cell, text, trigger } = menuCell()
    toggleToolsMenu()
    expect(window.getSelection()?.rangeCount).toBe(0)
    Object.defineProperty(document, 'activeElement', { value: trigger, configurable: true })
    closeToolsMenu()
    vi.runAllTimers()
    vi.useRealTimers()
    expect(cell.focus).not.toHaveBeenCalled()
    expect(window.getSelection()?.anchorNode).toBe(text)
    expect(window.getSelection()?.anchorOffset).toBe(3)
  })

  it('mở và đóng bằng hợp âm khi ô giữ tiêu điểm: caret về, tiêu điểm vẫn ở ô', () => {
    vi.useFakeTimers()
    const { cell, text } = menuCell(2)
    toggleToolsMenu()
    expect(document.activeElement).toBe(cell)
    toggleToolsMenu()
    vi.runAllTimers()
    vi.useRealTimers()
    expect(document.activeElement).toBe(cell)
    expect(window.getSelection()?.anchorNode).toBe(text)
    expect(window.getSelection()?.anchorOffset).toBe(2)
  })

  it('menu sang nút khác: không giành tiêu điểm khỏi nút đó', () => {
    vi.useFakeTimers()
    const { cell, text } = menuCell(2)
    const other = document.createElement('button')
    document.body.append(other)
    toggleToolsMenu()
    Object.defineProperty(document, 'activeElement', { value: other, configurable: true })
    closeToolsMenu()
    vi.runAllTimers()
    vi.useRealTimers()
    expect(cell.focus).not.toHaveBeenCalled()
    expect(document.activeElement).toBe(other)
    expect(window.getSelection()?.anchorNode).toBe(text)
  })

  it('bấm vào nguồn đặt vùng chọn mới: không trả điểm cũ', () => {
    vi.useFakeTimers()
    const { text } = menuCell(2)
    toggleToolsMenu()
    select(text, 5)
    closeToolsMenu()
    vi.runAllTimers()
    vi.useRealTimers()
    expect(window.getSelection()?.anchorOffset).toBe(5)
  })

  it('nút Đã xong thật trên StatusBar: vùng chọn trong ô sống qua mousedown và click', async () => {
    const editor = await import('../../src/panels/editorPanelState')
    await editor.ensureSegmentsLoaded()
    const commands = await import('../../src/commands')
    commands.installCommands({ setOpenChapterDone: () => {} } as unknown as Parameters<typeof commands.installCommands>[0])
    ;(await import('../../src/modes/modeState')).setMode('workspace')
    const StatusBar = (await import('../../src/StatusBar.vue')).default
    const { text } = makeCell()
    const w = mount(StatusBar, { attachTo: document.body })
    await nextTick()
    const done = w.find('[data-chapter-done]')
    expect(done.exists()).toBe(true)
    select(text, 2)
    await done.trigger('mousedown')
    await done.trigger('click')
    expect(window.getSelection()?.anchorNode).toBe(text)
    expect(window.getSelection()?.anchorOffset).toBe(2)
    w.unmount()
  })

  it('hộp thoại đã mở khi menu đóng: không trả, lớp phủ tự trả khi đóng', () => {
    vi.useFakeTimers()
    const { cell } = menuCell(3)
    toggleToolsMenu()
    closeToolsMenu()
    overlayDialog()
    vi.runAllTimers()
    vi.useRealTimers()
    expect(window.getSelection()?.rangeCount).toBe(0)
    const back = focusReturnTargetOnOpen('[data-x-open]')
    expect(back).toBe(cell)
    back?.focus()
    expect(window.getSelection()?.anchorOffset).toBe(3)
  })

  it('menu nguồn Tra cứu đang gắn: mở xoá vùng chọn, đóng thì trả', async () => {
    vi.useFakeTimers()
    const Menu = (await import('../../src/panels/LookupSourcesMenu.vue')).default
    const { text } = menuCell(2)
    const w = mount(Menu, { attachTo: document.body })
    toggleSourcesMenu()
    expect(window.getSelection()?.rangeCount).toBe(0)
    closeSourcesMenu()
    vi.runAllTimers()
    vi.useRealTimers()
    expect(window.getSelection()?.anchorNode).toBe(text)
    expect(window.getSelection()?.anchorOffset).toBe(2)
    w.unmount()
  })

  it('menu nguồn Tra cứu chưa gắn (hợp âm khi panel không có): vùng chọn nguyên vẹn', () => {
    const { text } = menuCell(2)
    toggleSourcesMenu()
    expect(window.getSelection()?.anchorNode).toBe(text)
    expect(window.getSelection()?.anchorOffset).toBe(2)
    closeSourcesMenu()
  })
})

describe('qua hai lớp phủ thật', () => {
  it('Phạm vi soát: mở xoá vùng chọn, đóng trả điểm', async () => {
    const state = await import('../../src/proofreadState')
    const Scope = (await import('../../src/ProofreadScopeOverlay.vue')).default
    const { cell, text } = makeCell()
    const w = mount(Scope, { attachTo: document.body })
    select(text, 4)
    state.openProofreadScope()
    await nextTick()
    expect(window.getSelection()?.rangeCount).toBe(0)
    state.closeProofreadScope()
    await nextTick()
    await nextTick()
    expect(cell.focus).toHaveBeenCalled()
    expect(window.getSelection()?.anchorNode).toBe(text)
    expect(window.getSelection()?.anchorOffset).toBe(4)
    w.unmount()
  })

  it('Lịch sử đi hết đường đóng bằng Esc: caret về điểm cũ', async () => {
    resetRecorder()
    const editor = await import('../../src/panels/editorPanelState')
    await editor.ensureSegmentsLoaded()
    const history = await import('../../src/panels/segmentHistoryState')
    const { default: Overlay } = await import('../../src/SegmentHistoryOverlay.vue')
    const { cell, text } = makeCell()
    editor.setEditorCaret(11)
    select(text, 1)
    const w = mount(Overlay, { attachTo: document.body })
    history.openSegmentHistory()
    await w.vm.$nextTick()
    expect(window.getSelection()?.rangeCount).toBe(0)
    await w.find('.hist-scrim').trigger('keydown', { key: 'Escape' })
    await w.vm.$nextTick()
    await w.vm.$nextTick()
    expect(cell.focus).toHaveBeenCalled()
    expect(window.getSelection()?.anchorNode).toBe(text)
    expect(window.getSelection()?.anchorOffset).toBe(1)
    w.unmount()
  })
})

function stripComments(source: string): string {
  return source
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/(^|[^:'"`])\/\/.*$/gm, '$1')
}

function callsReturnTarget(source: string): boolean {
  return /focusReturnTargetOnOpen\(/.test(stripComments(source))
}

describe('mọi lớp phủ gọi chỗ giữ caret', () => {
  const dir = join(process.cwd(), 'src')
  const overlays = readdirSync(dir).filter((f) => f.endsWith('Overlay.vue'))

  it('có lớp phủ để kiểm', () => {
    expect(overlays.length).toBeGreaterThan(0)
  })

  it('bộ quét bỏ mọi dạng chú thích: lời gọi chỉ nằm trong chú thích thì không tính', () => {
    const real = 'const back = focusReturnTargetOnOpen(\'[x]\')'
    expect(callsReturnTarget(real)).toBe(true)
    for (const only of [
      '// focusReturnTargetOnOpen(',
      '/** focusReturnTargetOnOpen( */',
      '/* focusReturnTargetOnOpen( */ const a = 1',
      '<!-- focusReturnTargetOnOpen( -->',
      'const a = 1 // focusReturnTargetOnOpen(',
      '/**\n * focusReturnTargetOnOpen(\n */',
    ]) {
      expect(callsReturnTarget(only)).toBe(false)
    }
  })

  it('mỗi *Overlay.vue có một dòng mã gọi focusReturnTargetOnOpen', () => {
    const missing = overlays.filter((f) => !callsReturnTarget(readFileSync(join(dir, f), 'utf8')))
    expect(missing).toEqual([])
  })
})
