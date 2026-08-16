/**
 * Vùng gõ của lưới — **MỖI Ô LÀ MỘT EDITING HOST RIÊNG**. Story 2.5b · AC3 · Quyết định #3(b).
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔵 **TỆP NÀY ĐỔI TIỀN ĐỀ 2026-08-14, và ba mệnh đề của nó đã BỊ LẬT — ghi ra, không xoá**
 * ═════════════════════════════════════════════════════════════════════════════════
 * Bản trước canh Quyết định #1 của Story 2.3 *(đường (c), Ice ký 2026-08-12)*: *"vùng gõ là
 * **MỘT câu tại một thời điểm**"*, cài bằng `contenteditable` trên **đúng một** `<span>`.
 * Story 2.5b lật mệnh đề đó bằng một chữ ký tường minh, và **lý do là tiền đề cũ không còn
 * tồn tại**, không phải nó sai lúc được ký: tiền đề là *"một dòng văn liên tục"*, và lưới
 * không còn dòng văn liên tục nào.
 *
 * | Mệnh đề cũ | Nay |
 * |---|---|
 * | `.doc` không mang `contenteditable`, **0** phần tử gõ được trước lượt bấm | **mọi ô** cột bản dịch gõ được từ đầu |
 * | `contenteditable` sống trên **đúng một** câu và **theo caret** | thuộc tính **không đổi** theo caret; caret chỉ đổi `data`-state |
 * | *"caret không nhảy khi vùng gõ được **lắp**"* — ca đắt nhất của 2.3 | **không còn lượt lắp nào** ⇒ mệnh đề mất đối tượng |
 *
 * 🔴 Cái mà cả ba mệnh đề cũ **mua** thì KHÔNG mất, nó đổi cơ chế: *"trình duyệt không bao giờ
 * được sửa cây `data-segment-id`"*. Trước: chỉ một span gõ được nên không có ranh giới nào để
 * gộp qua. Nay: mỗi ô là một editing host **riêng**, nên một `Range` soạn thảo **không bắc cầu
 * hai ô được** — cùng bảo đảm, tới từ cấu trúc thay vì từ một thuộc tính động.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ `happy-dom` KHÔNG PHẢI WebKit — vai của tệp này dừng ở đâu
 * ─────────────────────────────────────────────────────────────────────────────
 * Nó kiểm **hợp đồng của mã dự án**: mỗi ô là một editing host riêng, `Enter` bị chặn, một
 * lượt dán không tiêm cấu trúc, và văn bản đi vào tập chờ. Nó **không** —
 * và không được đọc thành — kiểm *hành vi chuẩn hoá DOM của một engine thật*: `happy-dom` là
 * một bản mô phỏng DOM trong Node. Vế đó thuộc **bàn đo** (`2-3-ban-do-vung-go.html`, mũi thăm
 * dò Task 0.1, hai engine thật) và **e2e trong WKWebView**. Bốn đường, bốn vai — AC25.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { FIXTURE_SEGMENTS, readFixture, recordSave, resetRecorder, saveCalls } from './support/segmentFixture'

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture, saveSegmentTargets: recordSave }
})

/** `PanelFrame` kéo theo hợp đồng focus của Story 1.6 — vai của nó không thuộc story này. */
const STUBS = { PanelFrame: { template: '<div class="panel-frame"><slot /></div>' } }

async function mountEditor() {
  vi.resetModules()
  const state = await import('../../src/panels/editorPanelState')
  const GridPanel = (await import('../../src/panels/GridPanel.vue')).default

  const wrapper = mount(GridPanel, {
    props: { params: {} } as never,
    global: { stubs: STUBS },
    attachTo: document.body,
  })
  await state.ensureSegmentsLoaded()
  await wrapper.vm.$nextTick()
  return { state, wrapper }
}

/**
 * **Ô BẢN DỊCH** của một câu, đọc từ DOM thật đã mount.
 *
 * 🔴 `[data-col="tgt"]` là **bắt buộc**, không một lượt viết cho rõ: từ Story 2.5b một câu có
 * **HAI** phần tử mang `data-segment-id` *(ô nguyên văn và ô bản dịch)*, nên một selector chỉ
 * theo id trả về **ô nguyên văn** — phần tử đầu tiên theo thứ tự tài liệu. Mọi phép kiểm dưới
 * đây sẽ đo nhầm cột, và phần lớn vẫn **xanh**.
 */
function sentence(id: number): HTMLElement {
  const el = document.querySelector<HTMLElement>(`[data-col="tgt"][data-segment-id="${id}"]`)
  if (el === null) throw new Error(`ô bản dịch của câu ${id} không có trong DOM`)
  return el
}

/**
 * Đặt caret vào một câu, rồi bắn `selectionchange` — đúng sự kiện mà sản phẩm nghe.
 *
 * 🔴 **Hai ca, không một** — và ca thứ hai là ca người dùng gặp ĐẦU TIÊN. Một câu **chưa dịch**
 * có `target_text === ''`, nên `<span>` của nó **không có text node nào** để mà neo caret vào;
 * neo rơi vào chính phần tử, ở `offset` 0. Story 2.2 đã ghi cùng sự thật này từ phía hình học:
 * *"câu 4 rỗng, và một `<span>` rỗng không có hình chữ nhật nào"*.
 *
 * ⇒ Sản phẩm phải chịu được **cả hai** hình dạng neo, và nó chịu: `onSelectionChange` đọc
 * `anchor instanceof Element ? anchor : anchor.parentElement`, còn đường trả caret dùng
 * `childNodes.length` làm trần khi node không phải text.
 */
function putCaretIn(id: number, offset: number): void {
  const el = sentence(id)
  const text = el.firstChild
  const range = document.createRange()
  if (text === null) {
    // Câu chưa dịch — neo vào chính phần tử. Đây là ca gõ ĐẦU TIÊN của mọi Chương mới.
    range.setStart(el, 0)
  } else {
    range.setStart(text, Math.min(offset, (text as Text).data.length))
  }
  range.collapse(true)
  const selection = window.getSelection()
  selection?.removeAllRanges()
  selection?.addRange(range)
  document.dispatchEvent(new Event('selectionchange'))
}

beforeEach(() => {
  resetRecorder()
  document.body.innerHTML = ''
})

describe('vùng gõ — MỖI Ô MỘT EDITING HOST RIÊNG (Quyết định #3, đường b)', () => {
  it('MỌI ô bản dịch gõ được từ đầu, và mỗi câu có ĐÚNG HAI neo `data-segment-id`', async () => {
    const { wrapper } = await mountEditor()

    // 🔵 ĐẢO mệnh đề của Story 2.3 (*"0 phần tử gõ được trước lượt bấm"*). Đây chính là thứ
    // đóng khuyết tật *"sập hố"* theo cấu trúc — không còn một lượt lắp thuộc tính nào để
    // engine thả vùng chọn, và không còn một `<span>` rỗng cao 0 px nào.
    const editable = document.querySelectorAll('[contenteditable="true"]')
    expect(editable.length).toBe(FIXTURE_SEGMENTS.length)
    for (const el of editable) expect(el.getAttribute('data-col')).toBe('tgt')

    // 🔴 B11 — `[data-segment-id]` nay đếm **2 × số câu**. Ca này khoá mệnh đề đó lại: hai
    // spec e2e và mọi phép đếm khác đều phải nói rõ CỘT nào.
    expect(document.querySelectorAll('[data-segment-id]').length).toBe(FIXTURE_SEGMENTS.length * 2)
    expect(document.querySelectorAll('[data-col="src"]').length).toBe(FIXTURE_SEGMENTS.length)
    expect(document.querySelectorAll('[data-col="tgt"]').length).toBe(FIXTURE_SEGMENTS.length)

    wrapper.unmount()
  })

  it('thuộc tính `contenteditable` KHÔNG đổi khi caret dời — nó không còn theo caret', async () => {
    const { wrapper } = await mountEditor()

    putCaretIn(12, 3)
    await wrapper.vm.$nextTick()
    expect(document.querySelectorAll('[contenteditable="true"]').length).toBe(FIXTURE_SEGMENTS.length)

    putCaretIn(11, 2)
    await wrapper.vm.$nextTick()
    // 🔵 Bản cũ đòi *"đúng MỘT"* và đòi thuộc tính **dời** theo caret. Nay lượt dời caret
    // không chạm một thuộc tính DOM nào — đó là toàn bộ điểm của Quyết định #3(b), và cũng là
    // lý do ca *"caret không nhảy khi vùng gõ được lắp"* của Story 2.3 **mất đối tượng**:
    // không còn lượt lắp nào để caret nhảy vì nó.
    expect(document.querySelectorAll('[contenteditable="true"]').length).toBe(FIXTURE_SEGMENTS.length)
    expect(sentence(11).getAttribute('contenteditable')).toBe('true')
    expect(sentence(12).getAttribute('contenteditable')).toBe('true')

    wrapper.unmount()
  })

  it('caret giữ nguyên vị trí qua một lượt dời — không lượt lắp nào đá nó', async () => {
    const { wrapper } = await mountEditor()

    // Người dùng bấm vào giữa câu 12, ở ký tự thứ 9.
    putCaretIn(12, 9)
    await wrapper.vm.$nextTick()

    const selection = window.getSelection()
    expect(selection).not.toBeNull()
    const range = selection!.getRangeAt(0)

    // 🔴 Không phải *"caret còn tồn tại"* mà là **caret còn ở ĐÚNG ký tự thứ 9 của ĐÚNG câu
    // 12**. Một lượt nhảy về đầu ô cho `startOffset === 0` và vẫn qua được một phép kiểm lỏng.
    expect(range.startOffset).toBe(9)
    expect(sentence(12).contains(range.startContainer)).toBe(true)
    expect(range.collapsed).toBe(true)

    wrapper.unmount()
  })

  /*
   * 🔵 **MỆNH ĐỀ CỦA CA NÀY BỊ LẬT 2026-08-16 (Story 2.5d, FR134/AD-46) — viết lại, không xoá.**
   *
   * Tên cũ: *"`Enter` bị CHẶN trong vùng gõ — cấu trúc đoạn là dữ liệu đã lưu (AD-37)"*, và nó
   * khẳng định **hai** lượt `preventDefault`. Cả hai đã hết đúng, nhưng **theo hai kiểu khác
   * nhau**, và trộn chúng lại là cách đọc sai cả hai:
   *
   * | Vế cũ | Nay | Vì sao |
   * |---|---|---|
   * | `keydown` `Enter` ⇒ `defaultPrevented` | **KHÔNG** còn chặn ở lớp phím | Vế *"`Enter` trần không ký câu nào"* chuyển sang `keys.ts::isTypingZone` — nó chặn theo **vùng gõ**, không theo tên phím |
   * | `beforeinput insertParagraph` ⇒ `defaultPrevented` | **VẪN** `defaultPrevented` | Nhưng vì một lý do **ngược hẳn**: không phải để chặn xuống dòng, mà để **thay** nó bằng `insertLineBreak` |
   *
   * 🔴 Vế được **GIỮ NGUYÊN**, và nó là thứ đắt nhất ở đây: *"không `data-segment-id` nào nhân
   * đôi"*. Xuống dòng trong một ô **không** tách câu. AD-37 vẫn sở hữu cờ nguồn, cấu trúc đoạn
   * của bản dịch sống ở cột `is_target_paragraph_end` (bước di trú 9) — **không** ở ký tự `\n`.
   *
   * ⚠️ `happy-dom` **không phải** WebKit: ca này canh **hợp đồng của mã dự án** *(chặn cái gì,
   * phát cái gì, cây neo có đổi không)*. Vế *"engine dựng text node hay `<br>`"* thuộc **bàn đo**
   * (`2-5d-ban-do/`, WKWebView thật), và vế *"`\n` tới được đĩa"* thuộc **e2e**.
   */
  it('`Enter` xuống dòng trong ô bản dịch mà KHÔNG tách câu (AD-46)', async () => {
    const { wrapper } = await mountEditor()
    putCaretIn(12, 4)
    await wrapper.vm.$nextTick()

    const before = document.querySelectorAll('[data-segment-id]').length

    // ── ① Lớp phím KHÔNG còn nuốt `Enter` ────────────────────────────────────────────
    // 🔴 Đây là vế bị lật. Một `preventDefault()` ở đây sẽ chặn luôn lượt `beforeinput` mà
    // engine phát ra sau đó ⇒ AC1 chết ở tầng phím, trước khi nhánh ① kịp chạy.
    const key = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true })
    sentence(12).dispatchEvent(key)
    expect(key.defaultPrevented).toBe(false)

    // ── ② Một lượt commit IME vẫn đi lọt nguyên vẹn ──────────────────────────────────
    // 🔴 `Enter` là **phím chốt dấu** của Telex; ăn nó là ăn mất chữ. Chốt `isComposing`
    // đứng trước mọi nhánh khác và ca này canh nó.
    const composing = new KeyboardEvent('keydown', {
      key: 'Enter',
      bubbles: true,
      cancelable: true,
      isComposing: true,
    })
    sentence(12).dispatchEvent(composing)
    expect(composing.defaultPrevented).toBe(false)

    // ── ③ `beforeinput insertParagraph` VẪN bị chặn — nhưng để ĐỔI, không để cấm ──────
    const para = new InputEvent('beforeinput', {
      inputType: 'insertParagraph',
      bubbles: true,
      cancelable: true,
    })
    sentence(12).dispatchEvent(para)
    expect(para.defaultPrevented).toBe(true)

    // ── ④ `insertLineBreak` ĐI QUA — đó là lượt mà nhánh ① tự phát ────────────────────
    // ⚠️ Nếu ca này đỏ vì `defaultPrevented === true`, nghĩa là ai đó đã gộp hai `inputType`
    // lại như bản trước 2.5d — và lượt xuống dòng sẽ tự chặn chính nó.
    const lineBreak = new InputEvent('beforeinput', {
      inputType: 'insertLineBreak',
      bubbles: true,
      cancelable: true,
    })
    sentence(12).dispatchEvent(lineBreak)
    expect(lineBreak.defaultPrevented).toBe(false)

    // ── ⑤ VẾ GIỮ NGUYÊN: không câu nào bị tách ⇒ không `data-segment-id` nào nhân đôi ──
    expect(document.querySelectorAll('[data-segment-id]').length).toBe(before)

    wrapper.unmount()
  })

  /*
   * 🔴 **Đường ghi nhận được `\n`** — Story 2.5d, AC1, Task 3.7.
   *
   * ⚠️ Ca này canh **đường ghi**, KHÔNG canh hành vi engine. `happy-dom` không dựng lượt
   * `insertLineBreak` nào; nó cũng không cần — mệnh đề ở đây là *"nếu DOM mang `\n` thì tập
   * chờ nhận đúng chuỗi có `\n`, không bị làm phẳng ở một chặng nào"*. Vế *"engine có dựng ra
   * `\n` không"* đã đo ở bàn đo Task 1, và vế *"`\n` tới được đĩa"* là e2e (Task 10.3).
   *
   * Ba chặng mà một `\n` có thể chết mà không ai thấy: `reportEdit` đọc `textContent` · lượt
   * vào `Map` của `noteEditorEdit` · và câu `UPDATE` ở Rust. Ca này canh chặng ĐẦU.
   */
  it('một lượt xuống dòng đưa chuỗi CÓ `\\n` vào tập chờ, không bị làm phẳng', async () => {
    const { state, wrapper } = await mountEditor()
    putCaretIn(13, 0)
    await wrapper.vm.$nextTick()

    // Engine đã hạ cánh `\n` vào DOM rồi mới phát `input` — mô phỏng đúng thứ tự đó, và mô
    // phỏng đúng **hình dạng** mà bàn đo đo được trên WKWebView: MỘT text node phẳng mang
    // `\n`, không `<br>`, không `<div>`.
    sentence(13).textContent = 'Dong mot.\nDong hai.'
    sentence(13).dispatchEvent(new InputEvent('input', { inputType: 'insertLineBreak', bubbles: true }))
    await wrapper.vm.$nextTick()

    const pending = state.editorEditedText.value.get(13)
    expect(pending).toBe('Dong mot.\nDong hai.')
    // 🔴 Mệnh đề trung tâm, viết tách ra để lượt đỏ nói đúng nguyên nhân: một chặng làm phẳng
    // sẽ cho `"Dong mot. Dong hai."` — một chuỗi **hợp lệ trông như thật**, và nó sẽ đi trọn
    // xuống đĩa mà không cổng nào đỏ.
    expect(pending?.includes('\n')).toBe(true)

    wrapper.unmount()
  })

  it('một lượt gõ đi vào tập chờ, đọc từ chính DOM', async () => {
    const { state, wrapper } = await mountEditor()
    putCaretIn(13, 0)
    await wrapper.vm.$nextTick()

    // Engine đã hạ cánh ký tự vào DOM rồi mới phát `input` — mô phỏng đúng thứ tự đó.
    sentence(13).textContent = 'Thiếu niên không quay đầu lại.'
    sentence(13).dispatchEvent(new Event('input', { bubbles: true }))

    expect(state.editorEditedText.value.get(13)).toBe('Thiếu niên không quay đầu lại.')

    wrapper.unmount()
  })

  it('dán nhiều dòng GIỮ `\n` vào `target_text`, và vẫn KHÔNG tiêm phần tử nào', async () => {
    // Mệnh đề này là hệ quả trực tiếp của mũi thăm dò Task 0.1: `contenteditable="true"` một
    // mình để **cả hai** engine tiêm markup và một `\n` thật vào trong một câu. Cái lọc là
    // đòn bẩy, và đây là lưới của nó ở tầng hợp đồng.
    //
    // 🔵 **VIẾT LẠI 2026-08-16 (code review, Ice ký đường (b)) — MỘT NỬA mệnh đề đã lật.**
    // Bản cũ đòi `\n` bị làm phẳng thành khoảng trắng và tên ca viết *"KHÔNG mang xuống
    // dòng"*. Mệnh đề đó đúng khi nó được viết, và nó đứng trên một tiền đề nay đã hết hiệu
    // lực: hồi đó `\n` **không thể** tồn tại trong `target_text`, nên giữ nó lại nghĩa là mất
    // nó. AC1 của chính Story 2.5d vừa làm `\n` thành một ký tự hợp lệ ⇒ một đường vào ô làm
    // phẳng còn đường kia thì không là **hai luật cho một ô**.
    //
    // 🔴 **Nửa KHÔNG lật, và nó mới là nửa đắt:** *"KHÔNG tiêm phần tử nào"*. Đó là mệnh đề
    // mũi thăm dò Task 0.1 đã trả tiền để biết, và lượt vá không được nới nó một ly — ba
    // `expect` cuối của ca này giữ nguyên từng chữ.
    const { state, wrapper } = await mountEditor()
    putCaretIn(13, 0)
    await wrapper.vm.$nextTick()

    const dt = new DataTransfer()
    dt.setData('text/plain', 'dòng một\ndòng hai\r\ndòng ba')
    const paste = new InputEvent('beforeinput', {
      inputType: 'insertFromPaste',
      dataTransfer: dt,
      bubbles: true,
      cancelable: true,
    })
    sentence(13).dispatchEvent(paste)

    expect(paste.defaultPrevented).toBe(true)
    const text = state.editorEditedText.value.get(13) ?? ''
    // 🔴 Ba dòng ở lại BA dòng — và `\r\n` chuẩn hoá về `\n`, không đi tiếp dưới dạng `\r`:
    //    một ô mang `\r` cho `textContent` lệch khỏi chuỗi trên đĩa ở lượt đọc lại, và
    //    `white-space: pre-line` không vẽ `\r` thành gì cả.
    expect(text).toBe('dòng một\ndòng hai\ndòng ba')
    expect(text.includes('\r')).toBe(false)
    // 🔴 KHÔNG phần tử nào bên trong câu — `<pre>`, `<span style>`, `<div>` đều là cấu trúc.
    expect(sentence(13).querySelectorAll('*').length).toBe(0)
    // 🔵 B11 (2026-08-14): `2 ×` số câu — một câu nay có **hai** neo *(ô nguyên văn + ô bản
    // dịch)*. Mệnh đề thật của ca này **không đổi**: một lượt dán **không được tách hay nhân
    // một `data-segment-id` nào**. Chỉ con số đổi.
    expect(document.querySelectorAll('[data-segment-id]').length).toBe(FIXTURE_SEGMENTS.length * 2)

    wrapper.unmount()
  })

  it('một ô chỉ chứa `\\n` vẫn được LƯỚI đọc là RỖNG — cùng định nghĩa với Rust', async () => {
    // 🔴 **Hai tầng, MỘT định nghĩa "rỗng"** — code review 2026-08-16.
    //
    // Lưới vẽ viền đứt `.cell-tgt.empty` cho câu **chưa dịch**; Rust từ chối ký một câu chưa
    // dịch bằng `target_text.trim().is_empty()` (`commands/segment.rs`, `confirm_segment`).
    // Trước lượt vá, lưới hỏi `=== ''` — và hai phép hỏi đó chỉ trùng nhau chừng nào ô KHÔNG
    // chứa được khoảng trắng. AC1 của chính story này vừa cho nó chứa `\n`.
    //
    // ⇒ Ca hỏng đo được: bấm `Enter` trong một ô rỗng cho `"\n"`. Ô **mất** viền đứt nên
    // trông đã dịch, mà `confirm_segment` vẫn từ chối — người dùng không có cách nào biết vì
    // sao. Đây là lưới giữ hai tầng nói cùng một thứ tiếng.
    const { state, wrapper } = await mountEditor()
    // 🔴 Câu **13** là câu duy nhất của fixture mang `target_text: ''` — ca này đứng trên
    //    đúng tiền đề đó, nên nó khẳng định tiền đề thay vì giả định.
    const id = 13
    expect(FIXTURE_SEGMENTS.find((s) => s.id === id)?.target_text).toBe('')
    putCaretIn(id, 0)
    await wrapper.vm.$nextTick()

    expect(sentence(id).classList.contains('empty')).toBe(true)

    // Engine đã hạ cánh ký tự vào DOM rồi mới phát `input` — mô phỏng đúng thứ tự đó.
    sentence(id).textContent = '\n'
    sentence(id).dispatchEvent(new Event('input', { bubbles: true }))
    await wrapper.vm.$nextTick()

    expect(state.editorEditedText.value.get(id)).toBe('\n')
    expect(sentence(id).classList.contains('empty')).toBe(true)

    // ── Đối chứng: một ký tự THẬT thì ô thôi rỗng — nếu không ca trên xanh vì lý do sai ──
    sentence(id).textContent = 'a'
    sentence(id).dispatchEvent(new Event('input', { bubbles: true }))
    await wrapper.vm.$nextTick()
    expect(sentence(id).classList.contains('empty')).toBe(false)

    wrapper.unmount()
  })

  it('một lượt commit IME tiếng Việt KHÔNG bị đường lọc hay lượt tháo cắt giữa chừng', async () => {
    // ═══════════════════════════════════════════════════════════════════════════════
    // 🔴 Task 3.6 — BA ĐƯỜNG CÓ THỂ CẮT MỘT LƯỢT COMPOSITION, và cả ba phải im
    // ═══════════════════════════════════════════════════════════════════════════════
    // ① `onEditKeydown` — một lượt commit của bộ gõ tiếng Việt phát `keydown` mang `code` vật
    //    lý, và `Enter` là phím commit thường dùng nhất. Ăn nó là ăn mất chữ vừa gõ. Cùng dòng
    //    và cùng lý do `keys.ts:504` đã đặt `isComposing` **đứng trước mọi thứ**.
    // ② `onBeforeInput` — `insertCompositionText` KHÔNG nằm trong danh sách *"chèn từ ngoài
    //    bàn phím"*, nên nó đi qua và engine tự hạ cánh chữ.
    // ③ lượt tháo `contenteditable` — composition xảy ra **trong một câu**, nên
    //    `editorCaretSegmentId` không đổi ⇒ không lượt tháo nào. Ca này khoá mệnh đề đó.
    const { state, wrapper } = await mountEditor()
    putCaretIn(13, 0)
    await wrapper.vm.$nextTick()
    const sent = sentence(13)

    sent.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true }))

    // ① `Enter` LÚC ĐANG COMPOSE phải đi tiếp — nó là phím commit, không phải một hợp âm.
    const commitKey = new KeyboardEvent('keydown', {
      key: 'Enter',
      isComposing: true,
      bubbles: true,
      cancelable: true,
    })
    sent.dispatchEvent(commitKey)
    expect(commitKey.defaultPrevented).toBe(false)

    // ② `insertCompositionText` không bị chặn.
    const compose = new InputEvent('beforeinput', {
      inputType: 'insertCompositionText',
      data: 'tiếng',
      bubbles: true,
      cancelable: true,
    })
    sent.dispatchEvent(compose)
    expect(compose.defaultPrevented).toBe(false)

    // Engine hạ cánh chữ rồi phát `input` — mô phỏng đúng thứ tự đó.
    sent.textContent = 'tiếng Việt có dấu'
    sent.dispatchEvent(new Event('input', { bubbles: true }))
    sent.dispatchEvent(new CompositionEvent('compositionend', { bubbles: true, data: 'tiếng' }))
    await wrapper.vm.$nextTick()

    expect(state.editorEditedText.value.get(13)).toBe('tiếng Việt có dấu')
    // ③ ô vẫn gõ được, và lượt commit IME KHÔNG làm mọc thêm hay mất đi một editing host nào.
    //
    // 🔵 Bản cũ đòi **đúng MỘT** vùng gõ; con số đó thuộc Quyết định #1 của Story 2.3 và đã
    // hết đúng. Mệnh đề thật ở đây là *"một lượt commit IME không đụng cấu trúc"* — nên nó
    // được viết lại thành một hằng đọc từ fixture, không một con số chép tay.
    expect(sent.getAttribute('contenteditable')).toBe('true')
    expect(document.querySelectorAll('[contenteditable="true"]').length).toBe(
      FIXTURE_SEGMENTS.length,
    )

    // 🔵 **VẾ NÀY BỊ LẬT 2026-08-16 (Story 2.5d, AD-46) — sửa tại chỗ, kèm lý do.**
    // Bản cũ đòi `Enter` **không** compose thì `defaultPrevented === true`, và nó dùng lượt
    // chặn đó để chứng minh chốt `isComposing` có tác dụng. Từ 2.5d **không lượt chặn nào ở
    // lớp phím nữa** ⇒ mệnh đề cũ mất đối tượng, và giữ nó là để một ca đỏ vì một hành vi
    // **đã cố ý bỏ**.
    //
    // 🔴 Nhưng thứ nó **mua** thì không được mất: *"chốt `isComposing` đứng trước và có tác
    // dụng"*. Nay đo bằng một mệnh đề đúng vai hơn — hàm phải trả về **sớm** ở lượt composing
    // và vì thế **không** phát lượt `execCommand` nào. Cách quan sát được từ ngoài: một lượt
    // `beforeinput insertParagraph` **có** bị chặn *(nhánh ① chạy)*, còn một `keydown` mang
    // `isComposing` thì **không** để lại dấu vết nào.
    const plainEnter = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true })
    sent.dispatchEvent(plainEnter)
    expect(plainEnter.defaultPrevented).toBe(false)

    const composingEnter = new KeyboardEvent('keydown', {
      key: 'Enter',
      bubbles: true,
      cancelable: true,
      isComposing: true,
    })
    sent.dispatchEvent(composingEnter)
    expect(composingEnter.defaultPrevented).toBe(false)
    // Cấu trúc neo KHÔNG đổi sau cả hai lượt — vế đắt nhất, giữ nguyên từ bản cũ.
    expect(document.querySelectorAll('[contenteditable="true"]').length).toBe(
      FIXTURE_SEGMENTS.length,
    )

    wrapper.unmount()
  })

  it('gõ nhiều ký tự KHÔNG dựng lại text node ⇒ caret không rơi về 0 (lỗi "gõ ngược")', async () => {
    // ═══════════════════════════════════════════════════════════════════════════════
    // 🔴 HỒI QUY — Ice bắt bằng mắt 2026-08-13: gõ `abc` ra `cba`
    // ═══════════════════════════════════════════════════════════════════════════════
    // Nguyên nhân: template từng render `editorEditedText.get(s.id) ?? s.target_text`, nên mỗi
    // phím gõ đổi state ⇒ Vue so vnode cũ (chuỗi TRƯỚC lượt gõ) với vnode mới ⇒ nó ghi
    // `textContent` ⇒ **text node bị dựng lại** ⇒ caret rơi về offset 0 ⇒ ký tự sau chèn vào đầu.
    //
    // ⚠️ Mệnh đề phải khẳng định là **danh tính của text node**, không phải chuỗi cuối cùng: một
    // phép kiểm `textContent === 'abc'` vẫn XANH dưới bản hỏng, vì mã test tự gán chuỗi đúng.
    // Thứ người dùng mất là caret, và dấu vết đo được của nó là node bị thay.
    const { state, wrapper } = await mountEditor()
    putCaretIn(12, 5)
    await wrapper.vm.$nextTick()

    const sent = sentence(12)
    const nodeAtStart = sent.firstChild
    expect(nodeAtStart).not.toBeNull()

    // Ba lượt gõ, mô phỏng đúng thứ tự thật: engine hạ cánh ký tự RỒI mới phát `input`.
    //
    // ⚠️ `appendData`, **không** `textContent = …`: gán `textContent` **tự nó** huỷ mọi node con
    // rồi dựng một text node MỚI, nên một ca viết như vậy đo chính mã test chứ không đo Vue.
    // Engine thật sửa `data` của node **có sẵn** — đó là lý do caret sống sót ở trình duyệt, và
    // là hình dạng ca này phải mô phỏng.
    for (const ch of ['a', 'b', 'c']) {
      ;(nodeAtStart as Text).appendData(ch)
      sent.dispatchEvent(new Event('input', { bubbles: true }))
      await wrapper.vm.$nextTick()
    }

    // 🔴 Vue KHÔNG được thay text node — nếu nó thay, caret của người dùng đã chết ba lần.
    expect(sent.firstChild).toBe(nodeAtStart)
    expect(state.editorEditedText.value.get(12)).toBe(sent.textContent)

    wrapper.unmount()
  })

  it('gõ vào MỘT câu không bao giờ đổi văn bản của câu KHÁC (lỗi "chữ biến mất")', async () => {
    // 🔴 HỒI QUY — Ice bắt bằng mắt 2026-08-13: bấm xuống dưới thì bản dịch đã có **biến mất
    // khỏi màn hình**. Nguyên nhân là bản vá đầu cho lỗi "gõ ngược": nó đóng băng chuỗi hiển thị
    // trong **một biến dùng chung cho mọi câu**, nên chuỗi của câu này bị áp lên câu khác.
    // Một bản vá chữa triệu chứng mà không chữa nguyên nhân đẻ ra một khuyết tật NẶNG HƠN.
    const { wrapper } = await mountEditor()

    const before = FIXTURE_SEGMENTS.map((s) => sentence(s.id).textContent)

    // Gõ vào câu RỖNG (13) — đúng ca Ice làm, và là ca vá đầu áp một chuỗi rỗng ra chỗ khác.
    putCaretIn(13, 0)
    await wrapper.vm.$nextTick()
    sentence(13).textContent = 'chữ mới'
    sentence(13).dispatchEvent(new Event('input', { bubbles: true }))
    await wrapper.vm.$nextTick()

    // Rồi bấm sang một câu khác — lượt chuyển là chỗ bản vá cũ làm chữ bốc hơi.
    putCaretIn(11, 3)
    await wrapper.vm.$nextTick()

    expect(sentence(11).textContent).toBe(before[0])
    expect(sentence(12).textContent).toBe(before[1])
    // Và câu vừa gõ giữ đúng chữ mới, không quay về bản lúc nạp.
    expect(sentence(13).textContent).toBe('chữ mới')

    wrapper.unmount()
  })

  it('văn bản đang gõ SỐNG SÓT một lượt tháo/mount lại panel (đổi preset bố cục)', async () => {
    // 🔴 Đây là lý do `editorEditedText` tồn tại ở tầng state chứ không trong `<script setup>`:
    // `WorkspaceDock.vue::applyPreset()` gọi `api.clear()` rồi dựng lại cả ba panel. Không có
    // nó, lượt mount lại vẽ chữ **lúc nạp** đè lên chữ người dùng vừa gõ.
    const { state, wrapper } = await mountEditor()
    putCaretIn(13, 0)
    await wrapper.vm.$nextTick()
    sentence(13).textContent = 'chữ vừa gõ'
    sentence(13).dispatchEvent(new Event('input', { bubbles: true }))
    wrapper.unmount()

    const GridPanel = (await import('../../src/panels/GridPanel.vue')).default
    const again = mount(GridPanel, {
      props: { params: {} } as never,
      global: { stubs: STUBS },
      attachTo: document.body,
    })
    await again.vm.$nextTick()

    expect(sentence(13).textContent).toBe('chữ vừa gõ')
    // Và tập chờ vẫn còn nguyên — lượt tháo panel không được làm mất một lượt ghi.
    expect(state.editorEditedText.value.get(13)).toBe('chữ vừa gõ')
    expect(saveCalls.length).toBe(0)

    again.unmount()
  })
})
