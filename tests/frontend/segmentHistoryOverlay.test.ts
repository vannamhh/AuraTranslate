import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { readFileSync } from 'node:fs'

import { readFixture, recordSave, resetRecorder } from './support/segmentFixture'

type Origin = 'self' | 'other' | 'bilingual_import' | null
type Span = { kind: 'equal' | 'delete' | 'insert'; text: string }

const served: {
  current: number | null
  versions: { id: number; text: string; origin: Origin; spans: Span[] }[]
} = { current: null, versions: [] }

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: readFixture,
    saveSegmentTargets: recordSave,
    readSegmentHistory: async () => ({
      history: {
        current_version_id: served.current,
        source_text: 'Nguyen van cua cau.',
        versions: served.versions.map((v, index) => ({
          id: v.id,
          segment_id: 11,
          target_text: v.text,
          created_at: `2026-10-16T0${9 - index}:00:00.000Z`,
          translation_origin: v.origin,
          spans: v.spans,
        })),
      },
      error: null,
    }),
  }
})

const eq = (text: string): Span[] => [{ kind: 'equal', text }]

async function setup() {
  resetRecorder()
  const editor = await import('../../src/panels/editorPanelState')
  await editor.ensureSegmentsLoaded()
  const history = await import('../../src/panels/segmentHistoryState')
  const commands = await import('../../src/commands')
  commands.installCommands({
    closeSegmentHistory: history.closeSegmentHistory,
    toggleHistoryCompare: history.toggleHistoryCompare,
    restoreAimedVersion: history.restoreAimedVersion,
  } as unknown as Parameters<typeof commands.installCommands>[0])
  const { default: Overlay } = await import('../../src/SegmentHistoryOverlay.vue')
  editor.setEditorCaret(11)
  history.openSegmentHistory()
  await vi.waitFor(() => expect(history.historyPending.value).toBe(false))
  const w = mount(Overlay, { attachTo: document.body })
  await w.vm.$nextTick()
  return { w, history }
}

afterEach(() => {
  document.body.innerHTML = ''
  Reflect.deleteProperty(document, 'activeElement')
  vi.resetModules()
})

describe('SegmentHistoryOverlay hai ngăn', () => {
  it('danh sách bên trái, chi tiết bên phải với nguyên văn và toàn văn phiên bản đang nhắm', async () => {
    served.current = 203
    served.versions = [
      { id: 203, text: 'Ban A.', origin: 'other', spans: eq('Ban A.') },
      {
        id: 202,
        text: 'Ban B.',
        origin: 'self',
        spans: [
          { kind: 'delete', text: 'B.' },
          { kind: 'insert', text: 'A.' },
        ],
      },
      { id: 201, text: 'Ban A.', origin: null, spans: eq('Ban A.') },
    ]
    const { w } = await setup()

    expect(w.findAll('[data-history-row]')).toHaveLength(3)
    expect(w.find('[data-history-detail]').exists()).toBe(true)
    expect(w.find('[data-history-source]').text()).toBe('Nguyen van cua cau.')
    expect(w.find('[data-history-full]').text()).toBe('Ban A.')
    expect(w.find('[data-history-diff]').exists()).toBe(false)
    expect(w.findAll('[data-history-row]').map((r) => r.attributes('data-history-origin'))).toEqual(
      ['other', 'self', 'unknown'],
    )
  })

  it('hai hàng trùng chữ: chỉ hàng có `current_version_id` mang nhãn đang dùng, kể cả khi đó là hàng cũ hơn', async () => {
    served.current = 201
    served.versions = [
      { id: 203, text: 'Ban A.', origin: 'self', spans: eq('Ban A.') },
      { id: 202, text: 'Ban B.', origin: 'self', spans: eq('Ban B.') },
      { id: 201, text: 'Ban A.', origin: 'self', spans: eq('Ban A.') },
    ]
    const { w } = await setup()

    expect(
      w.findAll('[data-history-row]').map((r) => r.attributes('data-history-current')),
    ).toEqual(['false', 'false', 'true'])
  })

  it('không phiên bản nào đang dùng ⇒ không hàng nào có nhãn và có dòng "chưa ký"', async () => {
    served.current = null
    served.versions = [{ id: 201, text: 'Ban A.', origin: 'self', spans: eq('Ban A.') }]
    const { w } = await setup()

    expect(w.findAll('[data-history-current="true"]')).toHaveLength(0)
    expect(w.find('[data-history-unsigned]').exists()).toBe(true)
  })

  it('bấm "So với bản đang dùng" bật diff, bấm lần nữa trở lại toàn văn', async () => {
    served.current = 203
    served.versions = [
      { id: 203, text: 'Ban A.', origin: 'self', spans: eq('Ban A.') },
      {
        id: 202,
        text: 'Ban B.',
        origin: 'self',
        spans: [
          { kind: 'delete', text: 'B.' },
          { kind: 'insert', text: 'A.' },
        ],
      },
    ]
    const { w, history } = await setup()
    history.aimHistoryVersion(202)
    await w.vm.$nextTick()
    const compare = w
      .findAll('button.hist-act')
      .find(
        (b) => b.text() !== '' && b.attributes('disabled') === undefined && b.text().includes('So'),
      )
    expect(compare).toBeDefined()

    await compare!.trigger('click')
    expect(w.find('[data-history-diff]').exists()).toBe(true)
    expect(w.find('[data-history-diff] del').text()).toBe('B.')
    expect(w.find('[data-history-diff] ins').text()).toBe('A.')
    expect(w.find('[data-history-full]').exists()).toBe(false)

    await compare!.trigger('click')
    expect(w.find('[data-history-full]').text()).toBe('Ban B.')
    expect(w.find('[data-history-diff]').exists()).toBe(false)
  })

  it('↑↓ trên lớp phủ đổi hàng nhắm và chi tiết theo', async () => {
    served.current = 202
    served.versions = [
      { id: 202, text: 'Ban B.', origin: 'self', spans: eq('Ban B.') },
      { id: 201, text: 'Ban A.', origin: 'self', spans: eq('Ban A.') },
    ]
    const { w } = await setup()

    await w.find('.hist-scrim').trigger('keydown', { key: 'ArrowDown' })
    expect(w.find('[data-history-full]').text()).toBe('Ban A.')
    await w.find('.hist-scrim').trigger('keydown', { key: 'ArrowUp' })
    expect(w.find('[data-history-full]').text()).toBe('Ban B.')
  })

  it('không dùng `v-html` ở lớp phủ và ở `ReviewSpans`', () => {
    for (const file of ['src/SegmentHistoryOverlay.vue', 'src/panels/ReviewSpans.vue']) {
      expect(readFileSync(file, 'utf8')).not.toMatch(/v-html/)
    }
  })
})

describe('SegmentHistoryOverlay trả caret về chỗ cũ khi đóng', () => {
  async function mountThenOpen(setSelection: (cell: HTMLElement, text: Text) => void) {
    resetRecorder()
    served.current = null
    served.versions = []
    const editor = await import('../../src/panels/editorPanelState')
    await editor.ensureSegmentsLoaded()
    const history = await import('../../src/panels/segmentHistoryState')
    const { default: Overlay } = await import('../../src/SegmentHistoryOverlay.vue')

    const cell = document.createElement('div')
    cell.setAttribute('contenteditable', 'true')
    cell.setAttribute('tabindex', '0')
    cell.dataset.col = 'tgt'
    cell.dataset.segmentId = '11'
    const text = document.createTextNode('abcdef')
    cell.append(text)
    document.body.append(cell)
    const focusAtStart = vi.spyOn(cell, 'focus').mockImplementation(() => {
      const range = document.createRange()
      range.setStart(cell, 0)
      range.collapse(true)
      window.getSelection()?.removeAllRanges()
      window.getSelection()?.addRange(range)
      Object.defineProperty(document, 'activeElement', { value: cell, configurable: true })
    })
    Object.defineProperty(document, 'activeElement', { value: cell, configurable: true })
    editor.setEditorCaret(11)
    setSelection(cell, text)

    const w = mount(Overlay, { attachTo: document.body })
    history.openSegmentHistory()
    await w.vm.$nextTick()
    return { w, history, cell, text, focusAtStart }
  }

  const caretAt = (offset: number) => (_cell: HTMLElement, text: Text) => {
    const range = document.createRange()
    range.setStart(text, offset)
    range.collapse(true)
    window.getSelection()?.removeAllRanges()
    window.getSelection()?.addRange(range)
  }

  async function closeByEsc(w: Awaited<ReturnType<typeof mountThenOpen>>['w']) {
    await w.find('.hist-scrim').trigger('keydown', { key: 'Escape' })
    await w.vm.$nextTick()
    await w.vm.$nextTick()
  }

  it('Mod+H rồi Esc: caret về giữa chữ, không về đầu ô', async () => {
    const { w, text, focusAtStart } = await mountThenOpen(caretAt(1))
    await closeByEsc(w)
    expect(focusAtStart).toHaveBeenCalled()
    const selection = window.getSelection()
    expect(selection?.isCollapsed).toBe(true)
    expect(selection?.anchorNode).toBe(text)
    expect(selection?.anchorOffset).toBe(1)
  })

  it('mục menu Lịch sử: WebKit bôi một từ, xoá vùng chọn lúc đóng menu, Esc trả về một điểm sau từ đó', async () => {
    const { w, text } = await mountThenOpen((_cell, node) => {
      const range = document.createRange()
      range.setStart(node, 1)
      range.setEnd(node, 4)
      window.getSelection()?.removeAllRanges()
      window.getSelection()?.addRange(range)
    })
    window.getSelection()?.removeAllRanges()
    await closeByEsc(w)
    const selection = window.getSelection()
    expect(selection?.isCollapsed).toBe(true)
    expect(selection?.anchorNode).toBe(text)
    expect(selection?.anchorOffset).toBe(4)
  })

  it('khôi phục phiên bản làm nút văn bản cũ rời DOM: không ném, tiêu điểm vẫn về ô, caret ở đường hiện có', async () => {
    const { w, cell, focusAtStart } = await mountThenOpen(caretAt(2))
    cell.textContent = 'xyz'
    await closeByEsc(w)
    expect(focusAtStart).toHaveBeenCalled()
    const selection = window.getSelection()
    expect(selection?.anchorNode).toBe(cell)
    expect(selection?.anchorOffset).toBe(0)
  })

  it('không có Range trong ô lúc mở: đường đặt caret hiện có, không đụng vùng chọn', async () => {
    const { w, cell } = await mountThenOpen(() => {
      const other = document.createElement('p')
      other.textContent = 'ngoài ô'
      document.body.append(other)
      const range = document.createRange()
      range.setStart(other.firstChild as Text, 2)
      range.collapse(true)
      window.getSelection()?.removeAllRanges()
      window.getSelection()?.addRange(range)
    })
    await closeByEsc(w)
    const selection = window.getSelection()
    expect(selection?.anchorNode).toBe(cell)
    expect(selection?.anchorOffset).toBe(0)
  })

  it('Chương đổi trong lúc mở: Range đã lưu bị bỏ', async () => {
    const { w, history, cell } = await mountThenOpen(caretAt(3))
    history.resetSegmentHistory()
    await w.vm.$nextTick()
    await w.vm.$nextTick()
    expect(history.takeHistoryCaretRange()).toBeNull()
    expect(window.getSelection()?.anchorNode).toBe(cell)
    expect(window.getSelection()?.anchorOffset).toBe(0)
  })
})
