/**
 * `onCopy` (`src/panels/SourceHanViet.vue`) rebuilds the clipboard text in parallel view via
 * `resolveParallel`, since WKWebView copy ignores `user-select: none` on `<rt>` and would
 * otherwise leak Han readings through `Selection.toString()`. Switch view keeps the plain
 * `Selection.toString()` + `WORD_JOINER` gate.
 */
import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import SourceHanViet from '../../src/panels/SourceHanViet.vue'

/** Same reference text as `hanVietCutAnchors.test.ts`: one Han word, a non-Han fragment, another Han word. */
const NGUYEN_VAN = '京都」，春風'
//                  0 1 2 3 4 5   ⇐ chỉ số ký tự nguồn

function dung(viewMode: 'switch' | 'parallel') {
  return mount(SourceHanViet, {
    props: { sourceText: NGUYEN_VAN, viewMode, cuts: [], glossaryTerms: [], surfaceRole: 'cell' as const },
  })
}

beforeEach(() => {
  window.getSelection()?.removeAllRanges()
})

describe('SourceHanViet — onCopy', () => {
  it('kiểu song song: bôi đen một TỪ HÁN + mảnh liền sau ⇒ clipboard chỉ mang KÝ TỰ NGUỒN, không lẫn âm đọc', () => {
    const w = dung('parallel')

    const unit = w.find('.hv-unit')
    const rubyEl = unit.element.querySelector('ruby')
    const rubyBase = rubyEl === null ? undefined : Array.from(rubyEl.childNodes).find((n) => n.nodeType === Node.TEXT_NODE)
    if (rubyBase === undefined) throw new Error('không tìm thấy text node base của <ruby>')

    // Parallel view has no static `.hv-text` class on non-Han fragments; anchor via
    // `data-src-start` instead (same approach as `hanVietCutAnchors.test.ts`).
    const textEl = w.find('[data-src-start="2"]').element
    const textNode = textEl.firstChild
    if (textNode === null || textNode.nodeType !== Node.TEXT_NODE) {
      throw new Error('không tìm thấy text node của .hv-text')
    }

    const range = document.createRange()
    range.setStart(rubyBase, 0)
    range.setEnd(textNode, textNode.textContent?.length ?? 0)

    const selection = window.getSelection()
    if (selection === null) throw new Error('window.getSelection() trả null trong happy-dom')
    selection.removeAllRanges()
    selection.addRange(range)

    const clipboardData = new DataTransfer()
    const event = new ClipboardEvent('copy', { clipboardData, bubbles: true, cancelable: true })
    w.find('.hv-surface').element.dispatchEvent(event)

    expect(clipboardData.getData('text/plain')).toBe('京都」，')
    w.unmount()
  })

  it('kiểu chuyển đổi: bôi đen một TỪ nhiều âm tiết ⇒ clipboard mang âm đọc hiện trên bề mặt, cách nhau bằng dấu cách, không lẫn WORD_JOINER hay ký tự Hán', () => {
    const w = dung('switch')

    const word = w.find('.hv-word')
    const syllables = word.findAll('.hv-syl')
    expect(syllables.length).toBeGreaterThanOrEqual(2)

    const firstText = syllables.at(0)?.element.firstChild
    const lastText = syllables.at(-1)?.element.firstChild
    if (firstText === undefined || firstText === null || firstText.nodeType !== Node.TEXT_NODE) {
      throw new Error('không tìm thấy text node của âm tiết đầu')
    }
    if (lastText === undefined || lastText === null || lastText.nodeType !== Node.TEXT_NODE) {
      throw new Error('không tìm thấy text node của âm tiết cuối')
    }

    const range = document.createRange()
    range.setStart(firstText, 0)
    range.setEnd(lastText, lastText.textContent?.length ?? 0)

    const selection = window.getSelection()
    if (selection === null) throw new Error('window.getSelection() trả null trong happy-dom')
    selection.removeAllRanges()
    selection.addRange(range)

    const clipboardData = new DataTransfer()
    const event = new ClipboardEvent('copy', { clipboardData, bubbles: true, cancelable: true })
    w.find('.hv-surface').element.dispatchEvent(event)

    const copied = clipboardData.getData('text/plain')
    expect(copied).toBe(Array.from({ length: syllables.length }, () => '·').join(' '))
    expect(copied).not.toContain('⁠')
    expect(copied).not.toContain('京')
    expect(copied).not.toContain('都')
    w.unmount()
  })

  it('không có vùng chọn hợp lệ trên bề mặt ⇒ không ghi gì vào clipboard, không preventDefault', () => {
    const w = dung('parallel')
    window.getSelection()?.removeAllRanges()

    const clipboardData = new DataTransfer()
    const event = new ClipboardEvent('copy', { clipboardData, bubbles: true, cancelable: true })
    const prevented = !w.find('.hv-surface').element.dispatchEvent(event)

    expect(clipboardData.getData('text/plain')).toBe('')
    expect(prevented).toBe(false)
    w.unmount()
  })
})
