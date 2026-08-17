/**
 * Story 2.9 · AC1 — **phép kiểm *"caret ở ĐẦU ô bản dịch"***.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO KHÔNG HỎI `startOffset === 0` — ĐO ĐƯỢC, KHÔNG SUY
 * ═════════════════════════════════════════════════════════════════════════════════
 * Bàn đo `2-9-ban-do/` chạy trên WKWebView 605.1.15 thật ngày 2026-08-17, caret đặt bằng
 * `document.caretRangeFromPoint` *(tức đúng thứ một cú bấm của người dùng cho)*. Bảy hình
 * dạng, và phép kiểm `startOffset === 0` **sai hai**:
 *
 * | Ca | `startContainer` | `startOffset` | `startOffset === 0` | đáp án đúng |
 * |---|---|---|---|---|
 * | ô RỖNG | `DIV` *(chính ô)* | 0 | true | đầu ô ✅ |
 * | một dòng, mép trái | `#text`(11) | 0 | true | đầu ô ✅ |
 * | một dòng, giữa chữ | `#text`(11) | 2 | false | không ✅ |
 * | hai dòng, mép trái dòng 1 | `#text`(3) | 0 | true | đầu ô ✅ |
 * | 🔴 **hai dòng, mép trái DÒNG 2** | **`#text`(3)** | **0** | 🔴 **true** | **không** ❌ |
 * | hai dòng, giữa chữ dòng 2 | `#text`(3) | 1 | false | không ✅ |
 * | 🔴 **vùng chọn TỪ đầu ô, không collapsed** | `#text` | 0 | 🔴 **true** | **không** ❌ |
 *
 * Cơ chế của ca sai thứ nhất: dưới `white-space: pre-line` (`GridPanel.vue`,
 * Story 2.5d/AD-46), `insertLineBreak` của WebKit để lại **ba text node** —
 * `"AAA"` · `"\n"` · `"BBB"` — **không** một `<br>`. Engine đặt caret ở đầu dòng 2 vào
 * **offset 0 của node THỨ BA**. ⇒ Một phép kiểm hỏi offset sẽ **gộp câu khi người dùng chỉ
 * muốn xoá một lần xuống dòng** — mất một segment, trên một thao tác `AD-5` **không cho hoàn
 * tác**, và **không cổng nào đỏ**.
 *
 * ⚠️ Phép kiểm ngược lại — *"`startContainer` là con ĐẦU của ô"* — **cũng sai**, ở ca ô rỗng:
 * `startContainer` là **chính ô** (0 con). Không hình dạng nào đúng cả bốn; phải hỏi đúng câu
 * định nghĩa: *"không còn ký tự nào phía trước caret trong cả ô"*.
 *
 * ⚠️ `happy-dom` **không phải WebKit**, và tệp này không giả vờ ngược lại: nó kiểm **phép
 * kiểm**, một mệnh đề thuần về cây DOM và `Range`. Vế *"engine phân giải một cú bấm thành
 * `(node, offset)` nào"* là mệnh đề của **bàn đo/e2e** — `2-9-ban-do/README.md`.
 */
import { describe, expect, it } from 'vitest'

import { caretAtCellStart } from '../../src/panels/editorSegments'

/** Dựng một ô bản dịch đúng hình dạng template sinh ra — MỘT mustache, không `#comment`. */
function oDich(text: string): HTMLElement {
  const cell = document.createElement('div')
  cell.setAttribute('data-col', 'tgt')
  cell.setAttribute('contenteditable', 'true')
  if (text !== '') cell.appendChild(document.createTextNode(text))
  document.body.appendChild(cell)
  return cell
}

/**
 * Ô hai dòng đúng hình dạng WebKit để lại sau `insertLineBreak` dưới `pre-line` —
 * **ba** text node, đo 2026-08-17 (`2-9-ban-do/README.md` §Ⓑ⓪).
 */
function oHaiDong(): { cell: HTMLElement; dau: Text; xuongDong: Text; sau: Text } {
  const cell = document.createElement('div')
  cell.setAttribute('data-col', 'tgt')
  cell.setAttribute('contenteditable', 'true')
  const dau = document.createTextNode('AAA')
  const xuongDong = document.createTextNode('\n')
  const sau = document.createTextNode('BBB')
  cell.append(dau, xuongDong, sau)
  document.body.appendChild(cell)
  return { cell, dau, xuongDong, sau }
}

/** Một `Selection` giả mang đúng một `Range` — hình dạng duy nhất phép kiểm đọc. */
function selTai(node: Node, offset: number, ketThuc?: { node: Node; offset: number }): Selection {
  const r = document.createRange()
  r.setStart(node, offset)
  if (ketThuc === undefined) r.collapse(true)
  else r.setEnd(ketThuc.node, ketThuc.offset)
  return {
    rangeCount: 1,
    getRangeAt: () => r,
  } as unknown as Selection
}

describe('Story 2.9 — caret ở ĐẦU ô bản dịch', () => {
  it('① ô RỖNG, caret neo vào CHÍNH ô ⇒ đầu ô', () => {
    // Hình dạng đo được: `startContainer` = DIV, `childNodes.length` = 0.
    const cell = oDich('')
    expect(cell.childNodes.length).toBe(0)
    expect(caretAtCellStart(cell, selTai(cell, 0))).toBe(true)
  })

  it('② ô một dòng, caret ở offset 0 của text node ⇒ đầu ô', () => {
    const cell = oDich('bốn năm sáu')
    expect(caretAtCellStart(cell, selTai(cell.firstChild as Node, 0))).toBe(true)
  })

  it('③ ô một dòng, caret GIỮA chữ ⇒ KHÔNG phải đầu ô', () => {
    const cell = oDich('bốn năm sáu')
    expect(caretAtCellStart(cell, selTai(cell.firstChild as Node, 2))).toBe(false)
  })

  it('④ ô hai dòng, caret đầu DÒNG 1 ⇒ đầu ô', () => {
    const { cell, dau } = oHaiDong()
    expect(caretAtCellStart(cell, selTai(dau, 0))).toBe(true)
  })

  it('🔴 ⑤ CA PHÂN BIỆT — caret đầu DÒNG THỨ HAI cũng cho `startOffset === 0`', () => {
    const { cell, sau } = oHaiDong()
    // 🔴 Chính hình dạng engine dựng: offset 0 của text node THỨ BA. Một phép kiểm hỏi
    // offset trả `true` ở đây và gộp mất một câu.
    const sel = selTai(sau, 0)
    expect(sel.getRangeAt(0).startOffset).toBe(0)
    expect(caretAtCellStart(cell, sel)).toBe(false)
  })

  it('⑥ ô hai dòng, caret giữa chữ dòng 2 ⇒ KHÔNG phải đầu ô', () => {
    const { cell, sau } = oHaiDong()
    expect(caretAtCellStart(cell, selTai(sau, 1))).toBe(false)
  })

  it('🔴 ⑦ CA PHÂN BIỆT THỨ HAI — vùng chọn BẮT ĐẦU từ đầu ô nhưng KHÔNG collapsed', () => {
    // AC6 nói *không chặn*, không nói *cướp phím*. `Backspace` khi đang bôi đen là "xoá vùng
    // chọn" — cướp nó làm người dùng mất cả đoạn vừa bôi đen VÀ một ranh giới câu, cùng lúc.
    const cell = oDich('bốn năm sáu')
    const text = cell.firstChild as Node
    const sel = selTai(text, 0, { node: text, offset: 4 })
    expect(sel.getRangeAt(0).startOffset).toBe(0)
    expect(sel.getRangeAt(0).collapsed).toBe(false)
    expect(caretAtCellStart(cell, sel)).toBe(false)
  })

  it('⑧ không có `Selection` nào ⇒ false, không ném', () => {
    const cell = oDich('bốn năm sáu')
    expect(caretAtCellStart(cell, null)).toBe(false)
    expect(caretAtCellStart(cell, { rangeCount: 0 } as unknown as Selection)).toBe(false)
  })

  it('⑨ caret ở một ô SAU trong cây ⇒ false', () => {
    // Ba handler của cột sống ở CỘT, không ở từng ô (`GridPanel.vue`), nên một sự kiện có
    // thể tới từ một ô trong khi `Selection` còn neo ở ô khác.
    const a = oDich('câu A')
    const b = oDich('câu B')
    expect(caretAtCellStart(a, selTai(b.firstChild as Node, 0))).toBe(false)
  })

  it('🔴 ⑨b CA PHÂN BIỆT THỨ BA — caret ở một ô ĐỨNG TRƯỚC trong cây', () => {
    // 🔴 Ca này tìm ra bằng một lượt **đột biến mã sản phẩm**: gỡ chốt `cell.contains(...)`
    // mà ca ⑨ vẫn XANH. Ca ⑨ sống sót nhờ tình cờ — `Range` từ `(a, 0)` tới một điểm trong
    // `b` *đứng sau* là một `Range` **xuôi**, `toString()` trả `"câu A"` nên phép đo dài ≠ 0.
    //
    // Chiều ngược lại thì không: theo DOM, `setEnd` với một biên **đứng trước** `start` sẽ
    // **thu `Range` về điểm cuối** ⇒ `toString()` rỗng ⇒ hàm trả `true` cho một caret **không
    // nằm trong ô**. Đó là một lượt gộp sai câu, im lặng.
    // ⇒ Chốt `contains` là hàng rào **duy nhất** cho ca này, và ca này là bằng chứng của nó.
    const truoc = oDich('câu TRƯỚC')
    const sau = oDich('câu SAU')
    expect(caretAtCellStart(sau, selTai(truoc.firstChild as Node, 0))).toBe(false)
  })

  it('⑩ ô chỉ chứa một `\\n` — "rỗng" theo mắt người dùng nhưng có một ký tự', () => {
    // 🔴 Ca này có tiền lệ đắt: code review 2026-08-16 tìm ra rằng bấm `Enter` trong một ô
    // rỗng cho `"\n"`, ô MẤT viền đứt nên trông đã dịch, mà `confirm_segment` vẫn từ chối ký.
    // Ở đây phép kiểm phải theo **ký tự**, không theo vẻ ngoài: sau `\n` là KHÔNG phải đầu ô.
    const cell = oDich('\n')
    const text = cell.firstChild as Node
    expect(caretAtCellStart(cell, selTai(text, 0))).toBe(true)
    expect(caretAtCellStart(cell, selTai(text, 1))).toBe(false)
  })
})
