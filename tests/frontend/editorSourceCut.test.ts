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

/**
 * 🔵 **2026-08-17, STORY 2.9 · AC9 — HÌNH DẠNG Ô ĐỔI, và mọi ca dưới đây đổi theo.**
 *
 * `sourceCutOffsetOf` **thôi đếm mù** mọi text node của ô; nó đọc một **NEO** `data-src-start`
 * trên phần tử mang ký tự nguồn. Lý do đầy đủ và bảng số ở khối §AC9 cuối tệp.
 *
 * ⇒ Mỗi ca ở đây nay bọc mảnh nguồn trong một `<span class="src-piece" data-src-start="…">`,
 * đúng hình dạng template sinh ra sau AC9. **Mệnh đề của từng ca KHÔNG đổi một chữ** — bài
 * học `<rt>` và bài học code-point vẫn nguyên, chỉ khác cơ chế thi hành: trước là hai chốt
 * viết tay, nay là hệ quả của cấu tạo.
 */
describe('Story 2.8 — chỗ cắt ở cột nguyên văn', () => {
  /** Bọc một mảnh nguồn đúng khuôn template sau AC9, rồi trả text node của nó. */
  function neo(cell: Element, text: string, start: number): Text {
    const span = document.createElement('span')
    span.className = 'src-piece'
    span.setAttribute('data-src-start', String(start))
    const t = document.createTextNode(text)
    span.appendChild(t)
    cell.appendChild(span)
    return t
  }

  it('① một text node duy nhất ⇒ offset đi thẳng', () => {
    const cell = oNguon('')
    const text = neo(cell, 'Mr. Smith đến.', 0)
    expect(sourceCutOffsetOf(cell, text, 3)).toBe(3)
  })

  it('② hình dạng THẬT của template — hai `#comment` chia chuỗi, mảnh trước RỖNG', () => {
    // Đúng `childNodes` đo được trên WKWebView 605.1.15 ngày 2026-08-17:
    // `[COMMENT, TEXT(0), COMMENT, TEXT(40), TEXT(0)]`.
    const cell = oNguon('<!-- a -->')
    cell.appendChild(document.createTextNode(''))
    cell.appendChild(document.createComment(' b '))
    const chinh = neo(cell, 'Một câu nguồn để bộ nhập có việc mà làm.', 0)
    cell.appendChild(document.createTextNode(''))

    // ⚠️ Tổng độ dài các text node đứng trước là **0** ⇒ hai cách cho cùng số. Ca này ghim
    // **tiền đề** của phép đo, không ghim phép ánh xạ — ca ③ mới làm việc đó.
    expect(sourceCutOffsetOf(cell, chinh, 7)).toBe(7)
  })

  it('🔴 ③ ca PHÂN BIỆT hai cách — có chữ đứng trước thì `offset` trần SAI', () => {
    const cell = oNguon('')
    neo(cell, '東京', 0)
    const unit = document.createElement('span')
    unit.setAttribute('data-src-start', '2')
    const ruby = document.createElement('ruby')
    ruby.appendChild(document.createTextNode('京都'))
    unit.appendChild(ruby)
    cell.appendChild(unit)
    const sau = neo(cell, 'です。', 4)

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
    const unit = document.createElement('span')
    unit.setAttribute('data-src-start', '0')
    const ruby = document.createElement('ruby')
    ruby.appendChild(document.createTextNode('京'))
    const rt = document.createElement('rt')
    rt.appendChild(document.createTextNode('kinh'))
    ruby.appendChild(rt)
    unit.appendChild(ruby)
    cell.appendChild(unit)
    const sau = neo(cell, '都です。', 1)

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
    const truoc = neo(cell, `${astral}${astral}`, 0)
    const sau = neo(cell, '走了。', 2)

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

  it('🔵 ⑤ bấm vào khoảng trống của một ô ⇒ `null` — ĐỔI HÀNH VI ở AC9, và đổi theo chiều tốt', () => {
    // `caretRangeFromPoint` trả chính `cell` khi điểm bấm không rơi vào text node nào, và
    // `cell` **không mang neo** ⇒ hàm từ chối.
    //
    // 🔵 **Bản trước trả `'Một câu.'.length`, tức CUỐI ô.** Con số đó luôn bị
    // `regroup.rs::split_at` từ chối bằng `segment.cut_leaves_empty_piece` — một chỗ cắt ở
    // cuối câu để lại một mảnh rỗng. ⇒ Đường cũ đổi một cú bấm vô nghĩa thành một dấu cắt
    // trông như thật, rồi để `⌘/` báo lỗi ở một bước sau, xa chỗ người dùng vừa bấm.
    // Đường mới **không ghi nhận gì** — đúng chính sách `onSourceCellMouseUp` đã khai bằng
    // chữ: *"một điểm cắt đoán bừa là một lượt tách sai chỗ trên dữ liệu người dùng"*.
    const cell = oNguon('')
    neo(cell, 'Một câu.', 0)
    expect(sourceCutOffsetOf(cell, cell, 0)).toBe(null)
  })
})

/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔵 STORY 2.9 · AC9 — PHÉP ÁNH XẠ ĐỔI TỪ *ĐẾM MÙ* SANG *ĐỌC NEO*
 * ═══════════════════════════════════════════════════════════════════════════════
 * Ice báo 2026-08-17: *"ở phần Hán Việt vẫn chưa thấy điểm cắt, và chưa cắt được"*.
 * Bàn đo `2-9-ban-do/han-viet-cho-cat.e2e.mjs` trên WKWebView thật, nguyên văn `京都春風。`
 * (**5 ký tự**):
 *
 * | Tab | `sourceCutOffsetOf` trả | Neo vào | Trong biên? |
 * |---|---|---|---|
 * | Nguyên văn *(đối chứng)* | 5 | text node của ô | — |
 * | Hán Việt **switch** | **17** | `.hv-syl` = `"kinh"` — **ÂM**, không phải chữ Hán | ❌ |
 * | Hán Việt **parallel** | **19** | base `<ruby>` = `"京都"`, đúng phải là **2** | ❌ |
 *
 * 🔴 **Nguyên nhân lớn nhất là thứ không ai nêu trong ba giả thuyết ban đầu:** dòng
 * `Nguồn: thieu-chuu` (`.hv-sources`, **17 ký tự**) nằm **trong ô** và bị đếm.
 *
 * 🔴 **Và hôm nay nó KHÔNG hỏng im lặng chỉ vì MAY:** `17`/`19` tình cờ vượt biên một câu 5
 * chữ nên Rust từ chối. Trên một câu Chương thật *(40–60 chữ)*, `19` nằm **trong biên** và
 * `⌘/` cắt **sai chỗ, im lặng**, trên dữ liệu mà AD-5 không cho hoàn tác.
 *
 * ⇒ Phép đếm mù bị thay bằng một **NEO tường minh**: `data-src-start` trên đúng những phần tử
 * mang ký tự nguồn. Không neo ⇒ **`null`**, tức từ chối — `.hv-sources`, `.hv-notice`, `<rt>`
 * đều rơi vào đó theo **cấu tạo**, không nhờ một danh sách loại trừ phải bảo trì.
 */
describe('Story 2.9 · AC9 — chỗ cắt đọc NEO `data-src-start`', () => {
  /** Ô nguyên văn ở nhánh VĂN BẢN THUẦN, đúng hình dạng template sau AC9. */
  function oThuan(text: string, start = 0): { cell: Element; manh: Text } {
    const cell = oNguon('')
    const span = document.createElement('span')
    span.className = 'src-piece'
    span.setAttribute('data-src-start', String(start))
    const manh = document.createTextNode(text)
    span.appendChild(manh)
    cell.appendChild(span)
    return { cell, manh }
  }

  it('① mảnh văn bản thuần ⇒ neo + offset trong mảnh', () => {
    const { cell, manh } = oThuan('Mr. Smith đến.')
    expect(sourceCutOffsetOf(cell, manh, 3)).toBe(3)
  })

  it('🔴 ② mảnh THỨ HAI của một ô đã có điểm cắt ⇒ cộng NEO, không đếm lại từ đầu', () => {
    const cell = oNguon('')
    for (const [text, start] of [
      ['京都', 0],
      ['春風。', 2],
    ] as const) {
      const span = document.createElement('span')
      span.className = 'src-piece'
      span.setAttribute('data-src-start', String(start))
      span.appendChild(document.createTextNode(text))
      cell.appendChild(span)
    }
    const manhHai = cell.children[1].firstChild as Node
    expect(sourceCutOffsetOf(cell, manhHai, 1)).toBe(3)
  })

  it('🔴 ③ DÒNG NGUỒN của Hán Việt ⇒ `null`, KHÔNG một con số', () => {
    // Đây là con số 17 của bàn đo. Không neo ⇒ từ chối, theo cấu tạo.
    const cell = oNguon('')
    const p = document.createElement('p')
    p.className = 'hv-sources'
    const chu = document.createTextNode('Nguồn: thieu-chuu')
    p.appendChild(chu)
    cell.appendChild(p)
    expect(sourceCutOffsetOf(cell, chu, 5)).toBe(null)
  })

  it('🔴 ④ ÂM Hán Việt ở kiểu `switch` ⇒ cắt theo RANH GIỚI TỪ (Ice ký)', () => {
    // `.hv-word` mang neo VÀ cờ nguyên khối: chữ Hán gốc không có trên màn hình, nên một cú
    // bấm vào âm chỉ nói được *"từ nào"*, không nói được *"ký tự nào trong từ"*.
    const cell = oNguon('')
    const word = document.createElement('span')
    word.className = 'hv-word'
    word.setAttribute('data-src-start', '2')
    word.setAttribute('data-src-atomic', '1')
    const syl = document.createElement('span')
    syl.className = 'hv-syl'
    const am = document.createTextNode('xuân')
    syl.appendChild(am)
    word.appendChild(syl)
    cell.appendChild(word)

    // Bấm giữa chữ "xuân" (offset 2) vẫn cho **đầu từ** = 2, không 2 + 2.
    expect(sourceCutOffsetOf(cell, am, 2)).toBe(2)
    expect(sourceCutOffsetOf(cell, am, 0)).toBe(2)
  })

  it('🔵 ④b mảnh `.hv-text` NGUYÊN KHỐI ⇒ neo về ĐẦU mảnh, không giữa mảnh (code review 2026-08-17)', () => {
    // ═══════════════════════════════════════════════════════════════════════════════
    // 🔴 Ca này canh một lượt đổi HÀNH VI, và trước nó không phép kiểm nào canh.
    // ═══════════════════════════════════════════════════════════════════════════════
    // Tầng Edge Case Hunter đo được: mảnh `.hv-text` *(dấu câu, số, chữ Latin)* **mang** chính ký
    // tự nguồn, nên phép đếm cho một offset **chính xác từng ký tự** — và đó lại là vấn đề, vì
    // `SourceHanViet.vue` vẽ dấu cắt bằng `cutSet.has(seg.srcStart)`, tức so với **ĐẦU** mảnh.
    // ⇒ Một offset ở giữa mảnh là một chỗ cắt **có hiệu lực mà không hiện ở đâu cả**: Rust thực
    //   thi nó đúng, trên dữ liệu AD-5 không cho hoàn tác, còn người dùng không thấy nó ở đâu.
    //
    // Ice ký 2026-08-17: **TỪ CHỐI offset giữa mảnh**. Bất biến — mọi offset trong một mảnh
    // KHÔNG-Hán đều vẽ được dấu. Cái giá là mất độ chính xác bên trong `」，`, và nó đọc được.
    //
    // ⚠️ Chuỗi `」，` dài 2 ký tự là chuyện **thường** trong tiểu thuyết, không một ca hiếm — đó
    // là vì sao nó không đi đường ghi nợ.
    const cell = oNguon('')
    const manh = document.createElement('span')
    manh.className = 'hv-text'
    manh.setAttribute('data-src-start', '7')
    manh.setAttribute('data-src-atomic', '1')
    const chu = document.createTextNode('」，')
    manh.appendChild(chu)
    cell.appendChild(manh)

    // Bấm ở giữa `」，` (offset 1) cho **đầu mảnh** = 7, không 8.
    expect(sourceCutOffsetOf(cell, chu, 1)).toBe(7)
    expect(sourceCutOffsetOf(cell, chu, 2)).toBe(7)
    expect(sourceCutOffsetOf(cell, chu, 0)).toBe(7)
  })

  it('🔴 ⑤ base `<ruby>` ở kiểu `parallel` ⇒ chính xác từng CHỮ', () => {
    const cell = oNguon('')
    const unit = document.createElement('span')
    unit.className = 'hv-unit'
    unit.setAttribute('data-src-start', '0')
    const ruby = document.createElement('ruby')
    const base = document.createTextNode('京都')
    const rt = document.createElement('rt')
    rt.appendChild(document.createTextNode('kinh đô'))
    ruby.append(base, rt)
    unit.appendChild(ruby)
    cell.appendChild(unit)

    // Đây là con số **2** mà bàn đo nói đúng phải là, thay cho **19**.
    expect(sourceCutOffsetOf(cell, base, 2)).toBe(2)
    expect(sourceCutOffsetOf(cell, base, 1)).toBe(1)
  })

  it('🔴 ⑥ `<rt>` ⇒ `null`, và nay nó là hệ quả của CẤU TẠO chứ không một chốt riêng', () => {
    const cell = oNguon('')
    const unit = document.createElement('span')
    unit.className = 'hv-unit'
    unit.setAttribute('data-src-start', '0')
    const ruby = document.createElement('ruby')
    const rt = document.createElement('rt')
    const am = document.createTextNode('kinh đô')
    rt.appendChild(am)
    ruby.append(document.createTextNode('京都'), rt)
    unit.appendChild(ruby)
    cell.appendChild(unit)
    expect(sourceCutOffsetOf(cell, am, 3)).toBe(null)
  })

  it('⑦ node ngoài ô ⇒ `null`', () => {
    const { cell } = oThuan('京都春風。')
    const ngoai = document.createTextNode('ở ngoài')
    document.body.appendChild(ngoai)
    expect(sourceCutOffsetOf(cell, ngoai, 2)).toBe(null)
  })

  it('🔴 ⑧ ký tự NGOÀI BMP đếm bằng điểm mã, không code unit', () => {
    // Bài học ② của code review 2.8, phải sống sót qua lượt đổi cấu tạo này.
    const { cell, manh } = oThuan('𠀀𠀁春')
    // Hai ký tự astral = 4 code unit JS, nhưng 2 `chars()` của Rust.
    expect(sourceCutOffsetOf(cell, manh, 4)).toBe(2)
  })
})
