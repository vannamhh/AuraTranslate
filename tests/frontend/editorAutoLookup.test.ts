/**
 * **AC23 — PHÉP ĐO:** Auto-Lookup còn chạy trên bề mặt Editor sau khi nó thành vùng gõ không?
 * Story 2.3 · Task 6.2.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO CÂU HỎI NÀY CÓ THẬT, VÀ NÓ KHÔNG ĐƯỢC TRẢ LỜI BẰNG LÝ LẼ
 * ─────────────────────────────────────────────────────────────────────────────
 * `keys.ts:510` — `if (lacksPrimaryMod(entry.mods) && isTypingZone(event.target)) return false` —
 * làm bốn command `selection.extend_*` của Story 1.18 *(`Shift+Mũi tên`)* **thôi dispatch** trên
 * bề mặt Editor, vì `Shift` không phải phím bổ trợ chính. Trong khi `epics.md:1762` hứa Editor
 * *"nhận được cùng hành vi khi chúng có nội dung ở các epic sau, **không cần cài lại**"*, và
 * `useSelectionSurface(surface, 'source')` vẫn đang cắm ở `EditorPanel.vue:67`.
 *
 * AC23 nói **hai kết quả đều hợp lệ**, nên đây là một phép **đo**, không một lượt xác nhận một
 * giả định. Kết quả ghi ở §Debug Log References của story.
 *
 * ⚠️ Đường **phím** của mệnh đề — *"luật vùng gõ không `preventDefault`, nên native
 * `contenteditable` vẫn mở rộng vùng chọn"* — là một mệnh đề về `keys.ts`, và nó đo ở
 * **Kiểm D** của `scripts/check-commands.mjs`. Tệp này đo vế **DOM**: một vùng chọn thật trong
 * Editor có phát `lookup.lookup_selection` hay không. Một mệnh đề, một đường (AC25).
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  attachSelectionWatcher,
  currentSelectionText,
  registerSelectionSurface,
} from '../../src/panels/selectionContract'

beforeEach(() => {
  document.body.innerHTML = ''
  window.getSelection()?.removeAllRanges()
})

/**
 * Dựng đúng hình dạng DOM mà `EditorPanel.vue` dựng: một `.doc` chứa các `<span>` mang
 * `data-segment-id`, và **một** trong số đó đang `contenteditable="true"`.
 */
function mountEditorSurface(): { doc: HTMLElement; sentence: HTMLElement; release: () => void } {
  const doc = document.createElement('div')
  doc.className = 'doc'
  doc.tabIndex = 0
  const sentence = document.createElement('span')
  sentence.className = 'sent'
  sentence.setAttribute('data-segment-id', '12')
  sentence.setAttribute('contenteditable', 'true')
  sentence.textContent = 'Gió thổi tới từ cuối hành lang.'
  doc.append(sentence)
  document.body.append(doc)

  // Cùng vai và cùng tham số với `EditorPanel.vue:67`.
  const release = registerSelectionSurface(doc, 'source')
  return { doc, sentence, release }
}

/** Bôi đen một khoảng chữ THẬT trong câu — neo vào một **text node**, đúng vị từ `surfaceFor`. */
function selectInside(sentence: HTMLElement, start: number, end: number): void {
  const text = sentence.firstChild
  if (text === null) throw new Error('câu không có text node')
  const range = document.createRange()
  range.setStart(text, start)
  range.setEnd(text, end)
  const selection = window.getSelection()
  selection?.removeAllRanges()
  selection?.addRange(range)
}

describe('AC23 — Auto-Lookup trên bề mặt Editor GÕ ĐƯỢC', () => {
  it('vùng chọn trong một câu đang `contenteditable` VẪN đọc ra được (hợp đồng vùng chọn)', () => {
    const { sentence, release } = mountEditorSurface()

    selectInside(sentence, 4, 10)
    // 🔴 Vị từ của `surfaceFor` là `anchorNode.nodeType === TEXT_NODE`. Một `contenteditable`
    // KHÔNG đổi điều đó — khác `<input>`/`<textarea>`, nơi vùng chọn neo vào phần tử CHA và bị
    // loại trừ cơ học (§②/③ của `selectionContract.ts`).
    expect(currentSelectionText()).toBe('thổi t')

    release()
  })

  it('`mouseup` sau một lượt bôi đen bằng CHUỘT vẫn phát lượt tra — đường chuột CÒN CHẠY', () => {
    const { sentence, release } = mountEditorSurface()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)

    selectInside(sentence, 0, 3)
    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))

    expect(dispatched).toHaveBeenCalledTimes(1)

    detach()
    release()
  })

  it('`keyup` của `Shift` vẫn phát lượt tra — đường BÀN PHÍM còn chạy qua hành vi native', () => {
    // 🔴 Đây là nửa thứ hai của phép đo AC23, và nó là nửa quan trọng: bốn command
    // `selection.extend_*` **thôi dispatch** trong vùng gõ. Nhưng luật vùng gõ của `keys.ts`
    // `return false` **TRƯỚC** `preventDefault()` (đo ở Kiểm D), nên engine tự mở rộng vùng
    // chọn — và lượt `keyup` của `Shift` mà bộ theo dõi này nghe vẫn phát lượt tra như thường.
    const { sentence, release } = mountEditorSurface()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)

    selectInside(sentence, 4, 11)
    document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Shift', bubbles: true }))

    expect(dispatched).toHaveBeenCalledTimes(1)

    detach()
    release()
  })

  it('một `keyup` KHÔNG phải `Shift` không phát gì — giữ nguyên hợp đồng của Story 1.18', () => {
    const { sentence, release } = mountEditorSurface()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)

    selectInside(sentence, 0, 5)
    document.dispatchEvent(new KeyboardEvent('keyup', { key: 'a', bubbles: true }))

    expect(dispatched).not.toHaveBeenCalled()

    detach()
    release()
  })

  it('một caret THU GỌN (bấm để gõ, không bôi đen) KHÔNG phát lượt tra nào', () => {
    // Đây là ca thường xuyên nhất của story này: người dùng bấm vào một câu để **gõ**. Một lượt
    // tra ở đó là một lượt IPC cộng một lượt đổi nội dung Panel Lookup ở mỗi cú bấm — đúng Bẫy 5
    // mà `selectionContract.ts:249-253` đã ghi, nay chạy trên một bề mặt được bấm liên tục.
    const { sentence, release } = mountEditorSurface()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)

    selectInside(sentence, 6, 6)
    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))

    expect(dispatched).not.toHaveBeenCalled()

    detach()
    release()
  })
})
