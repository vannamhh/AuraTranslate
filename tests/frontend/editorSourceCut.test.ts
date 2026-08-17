/**
 * Story 2.8 · AC2 — **phép ánh xạ một điểm bấm ở cột nguyên văn thành chỗ cắt**.
 *
 * 🔴 Mệnh đề trung tâm: `offset` mà `caretPositionFromPoint` trả về đếm **trong một text
 * node**, không trong ô — và một ô nguyên văn có **nhiều** text node vì template chèn hai
 * `#comment` (`aura-allow-text`). Hôm nay tổng độ dài các node đứng trước là **0** nên lấy
 * `offset` trần cho **cùng một số**; ca ③ dưới đây là ca phân biệt được hai cách, và nó là
 * hình dạng ô sẽ có ngay khi người dùng bật Hán Việt.
 *
 * ⚠️ `happy-dom` **không phải WebKit**, và ca này không giả vờ ngược lại: nó kiểm **phép
 * ánh xạ**, một mệnh đề thuần về cây DOM. Vế *"engine phân giải được điểm bấm thành
 * `(node, offset)` nào"* là mệnh đề của **bàn đo/e2e** — `2-8-ban-do/README.md`.
 */
import { describe, expect, it } from 'vitest'

import { sourceCutOffsetOf } from '../../src/panels/editorSegments'

/** Dựng một ô nguyên văn đúng hình dạng template sinh ra. */
function oNguon(html: string): Element {
  const cell = document.createElement('div')
  cell.setAttribute('data-col', 'src')
  cell.innerHTML = html
  document.body.appendChild(cell)
  return cell
}

describe('Story 2.8 — chỗ cắt ở cột nguyên văn', () => {
  it('① một text node duy nhất ⇒ offset đi thẳng', () => {
    const cell = oNguon('Mr. Smith đến.')
    const text = cell.firstChild
    expect(text).not.toBeNull()
    expect(sourceCutOffsetOf(cell, text as Node, 3)).toBe(3)
  })

  it('② hình dạng THẬT của template — hai `#comment` chia chuỗi, mảnh trước RỖNG', () => {
    // Đúng `childNodes` đo được trên WKWebView 605.1.15 ngày 2026-08-17:
    // `[COMMENT, TEXT(0), COMMENT, TEXT(40), TEXT(0)]`.
    const cell = oNguon('<!-- a -->')
    cell.appendChild(document.createTextNode(''))
    cell.appendChild(document.createComment(' b '))
    const chinh = document.createTextNode('Một câu nguồn để bộ nhập có việc mà làm.')
    cell.appendChild(chinh)
    cell.appendChild(document.createTextNode(''))

    // ⚠️ Tổng độ dài các text node đứng trước là **0** ⇒ hai cách cho cùng số. Ca này ghim
    // **tiền đề** của phép đo, không ghim phép ánh xạ — ca ③ mới làm việc đó.
    expect(sourceCutOffsetOf(cell, chinh, 7)).toBe(7)
  })

  it('🔴 ③ ca PHÂN BIỆT hai cách — có chữ đứng trước thì `offset` trần SAI', () => {
    const cell = oNguon('')
    cell.appendChild(document.createTextNode('東京'))
    const ruby = document.createElement('ruby')
    ruby.appendChild(document.createTextNode('京都'))
    cell.appendChild(ruby)
    const sau = document.createTextNode('です。')
    cell.appendChild(sau)

    // `sau` đứng sau 2 + 2 = 4 ký tự.
    expect(sourceCutOffsetOf(cell, sau, 1)).toBe(5)
    // 🔴 Và đây là con số mà một lượt lấy `offset` trần sẽ cho — lệch 4 ký tự, trên dữ liệu
    // người dùng, ở một thao tác AD-5 không cho hoàn tác.
    expect(sourceCutOffsetOf(cell, sau, 1)).not.toBe(1)
  })

  /**
   * 🔵 **2026-08-17, code review — ca ③ ở trên dựng một `<ruby>` KHÔNG CÓ `<rt>`.**
   *
   * Đó không phải hình dạng sản phẩm: `SourceHanViet.vue` dựng
   * `<ruby>{chữ Hán}<rt>{âm Hán Việt}</rt></ruby>`, và `<rt>` **là một text node** mà
   * `TreeWalker(SHOW_TEXT)` đếm vào. ⇒ ca ③ cho một lượt **tin cậy giả**: nó xanh trên đúng
   * cái nó tồn tại để bắt.
   *
   * 🔴 Kho đã đo và ghi ra cái bẫy này từ **2026-08-07** *(`SourceHanViet.vue` §"`ruby.
   * textContent` GỘP CẢ `<rt>`")* — một mệnh đề có sẵn mà không ai đọc lại lúc viết hàm mới.
   */
  it('🔴 ③b hình dạng THẬT của Hán Việt — `<rt>` KHÔNG được đếm là ký tự nguồn', () => {
    const cell = oNguon('')
    const ruby = document.createElement('ruby')
    ruby.appendChild(document.createTextNode('京'))
    const rt = document.createElement('rt')
    rt.appendChild(document.createTextNode('kinh'))
    ruby.appendChild(rt)
    cell.appendChild(ruby)
    const sau = document.createTextNode('都です。')
    cell.appendChild(sau)

    // `source_text` là `京都です。` ⇒ ký tự ngay sau `京` là chỉ số **1**.
    expect(sourceCutOffsetOf(cell, sau, 0)).toBe(1)
    // 🔴 Con số mà bản trước lượt vá cho: 1 (`京`) + 4 (`kinh`) = **5**. Cắt ở 5 là cắt sau
    // `です` — bốn ký tự lệch, trên dữ liệu người dùng, không hoàn tác được.
    expect(sourceCutOffsetOf(cell, sau, 0)).not.toBe(5)
  })

  it('🔴 ③c bấm vào chính `<rt>` ⇒ `null` — âm Hán Việt không phải một chỗ cắt', () => {
    const cell = oNguon('')
    const ruby = document.createElement('ruby')
    ruby.appendChild(document.createTextNode('京'))
    const rt = document.createElement('rt')
    const am = document.createTextNode('kinh')
    rt.appendChild(am)
    ruby.appendChild(rt)
    cell.appendChild(ruby)

    // Không ghi nhận gì còn hơn ghi nhận một chỗ cắt ở giữa một chuỗi KHÔNG có trong
    // `source_text` — cùng chính sách mà `onSourceCellMouseUp` đã khai cho ca không phân
    // giải được.
    expect(sourceCutOffsetOf(cell, am, 2)).toBeNull()
  })

  /**
   * 🔴 **Đơn vị đếm: code point, KHÔNG code unit UTF-16.**
   *
   * `regroup.rs::split_at` cắt bằng `chars()` — code point. `String.length` và mọi offset của
   * DOM Range đếm code unit. Hai đơn vị chỉ trùng trong BMP; một ký tự CJK Extension B
   * (U+20000+) là **2** code unit nhưng **1** `char`, nên mỗi ký tự astral đứng trước chỗ cắt
   * đẩy chỗ cắt lệch thêm một — và thường **vẫn trong biên** nên không lỗi nào ném.
   */
  it('🔴 ③d ký tự NGOÀI BMP — đếm code point, không code unit', () => {
    // U+20000 (𠀀) là một ký tự CJK Extension B: 1 code point, 2 code unit UTF-16.
    const astral = '\u{20000}'
    expect(astral.length).toBe(2)
    expect([...astral].length).toBe(1)

    const cell = oNguon('')
    const truoc = document.createTextNode(`${astral}${astral}`)
    cell.appendChild(truoc)
    const sau = document.createTextNode('走了。')
    cell.appendChild(sau)

    // `source_text` = `𠀀𠀀走了。` ⇒ Rust thấy 5 ký tự, và `走` bắt đầu ở chỉ số **2**.
    expect(sourceCutOffsetOf(cell, sau, 0)).toBe(2)
    // 🔴 Con số mà bản trước lượt vá cho: `.length` = **4**. Cắt ở 4 là cắt sau `了`.
    expect(sourceCutOffsetOf(cell, sau, 0)).not.toBe(4)

    // Và cùng luật bên TRONG một node: offset UTF-16 giữa hai ký tự astral là 2, không 1.
    expect(sourceCutOffsetOf(cell, truoc, 2)).toBe(1)
  })

  it('④ node NGOÀI ô ⇒ `null`, không một con số đoán bừa', () => {
    const cell = oNguon('trong ô')
    const ngoai = document.createElement('div')
    ngoai.appendChild(document.createTextNode('ô khác'))
    document.body.appendChild(ngoai)

    expect(sourceCutOffsetOf(cell, ngoai.firstChild as Node, 2)).toBeNull()
  })

  it('⑤ bấm vào khoảng trống của một ô ⇒ CUỐI ô, không đầu ô', () => {
    const cell = oNguon('Một câu.')
    // `caretPositionFromPoint` trả chính `cell` khi điểm bấm không rơi vào text node nào.
    expect(sourceCutOffsetOf(cell, cell, 0)).toBe('Một câu.'.length)
  })
})
