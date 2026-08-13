/**
 * **AC23 — MỆNH ĐỀ:** bề mặt Editor **KHÔNG** phát lượt tra từ điển.
 * Story 2.3 · Task 6.2 · Sprint Change Proposal 2026-08-13.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 2026-08-13 — TỆP NÀY ĐỔI TỪ MỘT PHÉP ĐO THÀNH MỘT MỆNH ĐỀ
 * ─────────────────────────────────────────────────────────────────────────────
 * Bản trước hỏi *"Auto-Lookup còn chạy trên bề mặt Editor sau khi nó thành vùng gõ không?"* và
 * ghi rằng **hai kết quả đều hợp lệ**. Nó đo ra **còn chạy**, và đóng theo đúng nhánh đó.
 *
 * Câu hỏi ấy **không bao giờ hỏi "có NÊN chạy không?"** — Ice trả lời ngày 2026-08-13: **không**.
 * Bề mặt Editor chứa **tiếng Việt đã dịch**, còn từ điển nhúng là zh→vi / en→vi ⇒ một lượt tra
 * ở đó trả **0 hàng, 0 lỗi, 0 ms** rồi **thay mất** kết quả người dùng vừa tra từ Panel Source.
 * Đúng vòng tự thay thế mà `selectionContract.ts:11-17` đã bác cho Panel Lookup, chỉ tệ hơn
 * một bậc vì thứ thay vào là **rỗng**. ⇒ `EditorPanel.vue` nay đăng ký vai **`'display'`**.
 *
 * Phép đo cũ **không sai** — nó chỉ không phủ câu hỏi này. Xem
 * `planning-artifacts/sprint-change-proposal-2026-08-13.md` §1.2.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY BẮT BUỘC MANG MỘT ĐỐI CHỨNG DƯƠNG
 * ─────────────────────────────────────────────────────────────────────────────
 * Sau lượt đảo, **năm** ca dưới đây đều khẳng định *"không có gì xảy ra"*. Một tệp toàn mệnh
 * đề âm tính là một tệp **xanh kể cả khi `attachSelectionWatcher` chết hoàn toàn** — và lúc
 * đó nó không còn nói gì về **vai**, nó chỉ nói rằng một cơ chế đã hỏng vẫn im lặng.
 * `project-context.md` §Luật của một CỔNG cấm đúng hình dạng đó: *"Một cổng chưa bao giờ đỏ
 * là một cổng chưa ai biết nó có chạy không."*
 * ⇒ Ca cuối cùng đăng ký một bề mặt vai `'source'` và đòi lượt tra **PHẢI** phát.
 *
 * ⚠️ Đường **phím** của mệnh đề — *"luật vùng gõ không `preventDefault`, nên native
 * `contenteditable` vẫn mở rộng vùng chọn"* — là một mệnh đề về `keys.ts`, và nó đo ở
 * **Kiểm D** của `scripts/check-commands.mjs`. Tệp này đo vế **DOM**. Một mệnh đề, một đường
 * (AC25). Vế **khai báo** — `EditorPanel.vue`/`AiTranslationPanel.vue` mang đúng vai
 * `'display'` trong mã sản phẩm — sống ở **Kiểm F ③**, không ở đây: tệp này dựng bề mặt giả
 * nên nó **không đọc được** vai thật trong `.vue`.
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

  // 🔵 Cùng vai và cùng tham số với `EditorPanel.vue` — `'display'` từ 2026-08-13.
  const release = registerSelectionSurface(doc, 'display')
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

describe('AC23 — bề mặt Editor KHÔNG phát lượt tra từ điển', () => {
  it('vùng chọn trong một câu đang `contenteditable` đọc ra RỖNG — vai `display`', () => {
    const { sentence, release } = mountEditorSurface()

    selectInside(sentence, 4, 10)
    // 🔴 Vùng chọn vẫn được hợp đồng NHẬN (bề mặt có đăng ký, `anchorNode` là text node) —
    // thứ đổi là `currentSelectionText()` trả `''` cho mọi vai khác `'source'`
    // (`selectionContract.ts:203`). Đây chính là chỗ FR48/FR60 sẽ KHÔNG đi qua: chúng đọc
    // `Selection` bằng đường của riêng chúng, không qua hàm này.
    expect(currentSelectionText()).toBe('')

    release()
  })

  it('`mouseup` sau một lượt bôi đen bằng CHUỘT không phát gì — đường chuột đã TẮT', () => {
    const { sentence, release } = mountEditorSurface()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)

    selectInside(sentence, 0, 3)
    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))

    expect(dispatched).not.toHaveBeenCalled()

    detach()
    release()
  })

  it('`keyup` của `Shift` không phát gì — đường BÀN PHÍM cũng đã TẮT', () => {
    // 🔴 Nửa thứ hai của mệnh đề. Luật vùng gõ của `keys.ts` `return false` TRƯỚC
    // `preventDefault()` (đo ở Kiểm D), nên engine vẫn tự mở rộng vùng chọn và lượt `keyup`
    // của `Shift` vẫn tới bộ theo dõi này. Thứ chặn lượt tra là **vai**, không phải sự vắng
    // mặt của sự kiện — và đó là lý do ca này tồn tại riêng khỏi ca `mouseup`.
    const { sentence, release } = mountEditorSurface()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)

    selectInside(sentence, 4, 11)
    document.dispatchEvent(new KeyboardEvent('keyup', { key: 'Shift', bubbles: true }))

    expect(dispatched).not.toHaveBeenCalled()

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
    // Ca thường xuyên nhất của story này: người dùng bấm vào một câu để **gõ**. Bẫy 5 của
    // `selectionContract.ts:249-253` chặn nó ở tầng "vùng chọn rỗng", tức **độc lập** với vai
    // — nên ca này vẫn có nghĩa sau lượt đảo: nó canh một hàng rào KHÁC.
    const { sentence, release } = mountEditorSurface()
    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)

    selectInside(sentence, 6, 6)
    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))

    expect(dispatched).not.toHaveBeenCalled()

    detach()
    release()
  })

  it('🔴 ĐỐI CHỨNG DƯƠNG — cùng bộ theo dõi VẪN phát trên một bề mặt vai `source`', () => {
    // 🔴 Không có ca này, năm ca trên là năm mệnh đề "không có gì xảy ra" và tệp xanh kể cả
    // khi `attachSelectionWatcher` chết hoàn toàn. Ca này chứng minh **cơ chế còn sống**, nên
    // năm ca âm tính ở trên nói về **VAI**, không về một bộ theo dõi hỏng.
    //
    // Hình dạng mô phỏng `SourcePanel.vue`: một bề mặt nguyên văn, vai `'source'`.
    const original = document.createElement('div')
    original.className = 'original'
    const line = document.createElement('p')
    line.textContent = '他打開了那扇門，走進了黑暗之中。'
    original.append(line)
    document.body.append(original)
    const release = registerSelectionSurface(original, 'source')

    const dispatched = vi.fn()
    const detach = attachSelectionWatcher(document, dispatched)

    selectInside(line, 5, 7)
    document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }))

    expect(dispatched).toHaveBeenCalledTimes(1)

    detach()
    release()
  })
})
