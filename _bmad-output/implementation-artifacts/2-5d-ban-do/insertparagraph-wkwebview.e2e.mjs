/**
 * Bàn đo Story 2.5d · Task 1.2 — SÁU MỆNH ĐỀ TRÊN **WKWebView THẬT**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 BÀN ĐO NÀY CHẠY TRÊN LƯỚI THẬT — và đó là một lượt GỠ giới hạn, không một tiện tay
 * ═════════════════════════════════════════════════════════════════════════════════
 * Bốn bàn đo trước của kho (`2-2` · `2-4` · `2-5` · `2-5b`) đều **tiêm một hình dạng DOM
 * chép tay** vào webview, và cả bốn mang cùng một giới hạn viết ra ở
 * `2-5b-ban-do/README.md` §Giới hạn ①: *"một lượt sửa template sau này làm bàn đo và sản
 * phẩm lệch nhau **mà không cổng nào đỏ**"*.
 *
 * Lượt này **không** cần chép: `GridPanel.vue` đã ở trong sản phẩm từ Story 2.5b, với
 * `contenteditable` tĩnh trên mọi ô bản dịch. ⇒ Bàn đo mở Tác phẩm thật, gõ vào ô thật,
 * và hỏi engine sáu câu hỏi **trên chính DOM mà người dùng chạm**. Số đọc ra vì thế mang
 * cả `<style scoped>` thật, cả `subgrid` thật, cả hợp đồng tiêu điểm thật.
 *
 * ⚠️ Giới hạn **mới** thay vào chỗ đó, và nó nhẹ hơn nhưng có thật: bàn đo phải **vô hiệu
 * hoá hai lớp chặn `Enter` đang sống trong sản phẩm** để nhìn được engine làm gì khi không
 * ai chặn. Cách làm ghi ở §VÔ HIỆU HOÁ dưới đây, và nó **không sửa một dòng mã sản phẩm**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO TỆP NÀY KHÔNG NẰM TRONG `e2e/specs/`
 * ═════════════════════════════════════════════════════════════════════════════════
 * Nó hỏi **engine**, không kiểm **sản phẩm** — và nó cố ý gỡ hai lớp bảo vệ của sản phẩm
 * để hỏi. Một tệp như vậy nằm trong bộ thường trực là dựng một ca xanh mãi mãi mà không
 * canh mệnh đề nào, đúng thứ §Testing Rules gọi là *"nguồn sự thật thứ hai"*.
 * ⇒ Chạy bằng `--spec`:
 *
 *     TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-5d-ban-do/insertparagraph-wkwebview.e2e.mjs
 *
 * Ca **thường trực** cho địa hạt này là Task 10.3, và nó chạy trên đường ghi thật.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÔ HIỆU HOÁ HAI LỚP CHẶN — bằng thứ tự sự kiện, không bằng một lượt sửa mã
 * ═════════════════════════════════════════════════════════════════════════════════
 * Hai lớp chặn hôm nay đăng ký trên **cột bản dịch** (`GridPanel.vue:955-957`), tức pha
 * **nổi bọt**. Một listener ở `document` pha **capture** chạy **trước** chúng, và
 * `stopPropagation()` ở đó cắt trọn đường đi xuống ⇒ handler của sản phẩm **không chạy**,
 * còn **hành vi mặc định của engine thì vẫn chạy** *(vì không ai gọi `preventDefault`)*.
 *
 * 🔴 `stopPropagation`, **KHÔNG** `stopImmediatePropagation`: bàn đo cần chính listener
 * ghi chép của mình chạy tiếp trên cùng nút. Và **KHÔNG** một lượt gỡ listener của sản
 * phẩm: không đường nào lấy được tham chiếu tới chúng, và một lượt `cloneNode` để gỡ sẽ
 * dựng lại đúng cái hình dạng chép tay mà bàn đo này tồn tại để tránh.
 *
 * ⚠️ **GIỚI HẠN CỦA BỘ ĐO, chép nguyên từ 2.5b vì nó vẫn đứng:** `browser.keys()` chỉ bắn
 * `keydown`/`keyup` và **không** đi vào đường nhập văn bản gốc của WKWebView *(đo
 * 2026-08-12)*. ⇒ Mệnh đề ① đo bằng **cả hai** đường và ghi **hai** số riêng; mệnh đề
 * ②–⑥ đi qua `execCommand`/`Range`, cùng đường soạn thảo của engine. Vế **bộ gõ tiếng
 * Việt thật** thì không đường nghiệm thu nào của dự án mô phỏng được — Task 1.5, **chủ:
 * Ice**.
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { realClick } from '../../../e2e/support/pointer.mjs'

/**
 * Cài bộ ghi chép + bộ cắt đường ở `document`, pha capture.
 *
 * Trả về qua `window.__aura25d` để các lượt `execute` sau đọc lại được.
 */
const ARM = `
  const W = (window.__aura25d = window.__aura25d || {})
  if (W.armed !== true) {
    W.log = []
    W.chan = true          // bật/tắt lượt cắt đường mà không phải gỡ listener
    const rec = (e) => {
      const t = e.target
      W.log.push({
        loai: e.type,
        inputType: e.inputType === undefined ? null : e.inputType,
        key: e.key === undefined ? null : e.key,
        cancelable: e.cancelable,
        tren_o_dich: !!(t && t.dataset && t.dataset.col === 'tgt'),
      })
      if (W.chan === true) e.stopPropagation()
    }
    for (const type of ['beforeinput', 'keydown', 'input']) {
      document.addEventListener(type, rec, true)
    }
    W.armed = true
  }
  W.log.length = 0
  return true
`

/** Đặt ô về đúng MỘT text node `text`, caret ở `offset`. Trả về trạng thái đọc lại được. */
const SEED = `
  const [sel_id, text, offset] = arguments
  const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + sel_id + '"]')
  if (cell === null) return { ok: false, detail: 'không thấy ô' }
  cell.textContent = text
  cell.focus()
  const node = cell.firstChild
  const sel = window.getSelection()
  if (sel === null || node === null) return { ok: false, detail: 'không có selection hoặc text node' }
  sel.setPosition(node, offset)
  return {
    ok: true,
    detail: '',
    soNodeCon: cell.childNodes.length,
    selectionType: sel.type,
    anchorOffset: sel.anchorOffset,
  }
`

/** Mô tả DOM bên trong ô, đủ chi tiết để đọc ra engine đã dựng cái gì. */
const DUMP = `
  const [sel_id] = arguments
  const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + sel_id + '"]')
  if (cell === null) return { ok: false }
  const the = [...cell.childNodes].map((n) =>
    n.nodeType === 3 ? 'TEXT(' + JSON.stringify(n.nodeValue) + ')' : n.nodeName,
  )
  const sel = window.getSelection()
  return {
    ok: true,
    innerHTML: cell.innerHTML,
    textContent: cell.textContent,
    coXuongDongTrongTextContent: cell.textContent.includes('\\n'),
    soNodeCon: cell.childNodes.length,
    soPhanTuCon: cell.querySelectorAll('*').length,
    theCon: the,
    caoOPx: +cell.getBoundingClientRect().height.toFixed(2),
    selectionType: sel ? sel.type : null,
    neoTrongO: !!(sel && sel.anchorNode && cell.contains(sel.anchorNode)),
    anchorLaText: !!(sel && sel.anchorNode && sel.anchorNode.nodeType === 3),
    anchorOffset: sel ? sel.anchorOffset : null,
  }
`

/** Chiều cao **track hàng** và độ lệch `top` giữa năm ô cùng một hàng — vế `subgrid`. */
const HANG = `
  const [sel_id] = arguments
  const o = [...document.querySelectorAll('[data-segment-id="' + sel_id + '"]')]
  if (o.length === 0) return { ok: false }
  const r = o.map((el) => {
    const b = el.getBoundingClientRect()
    return { col: el.dataset.col || 'meta', top: +b.top.toFixed(2), cao: +b.height.toFixed(2) }
  })
  const tops = r.map((x) => x.top)
  return {
    ok: true,
    o: r,
    soO: r.length,
    lechTopPx: +(Math.max(...tops) - Math.min(...tops)).toFixed(2),
    caoHangPx: +Math.max(...r.map((x) => x.cao)).toFixed(2),
  }
`

describe('Bàn đo 2.5d — `insertParagraph` và đường `<br>` trên WKWebView thật', () => {
  it('sáu mệnh đề, mỗi mệnh đề một số', async () => {
    const bao = { ngay: new Date().toISOString(), menhDe: {} }

    await openWorkspaceWithWork('Story 2.5d — bàn đo ngắt đoạn')

    /*
     * Nạp lại webview — cùng lý do và cùng phép đo với `grid-empty-cell.e2e.mjs:41-64`:
     * `$APPDATA` dùng chung cho cả lượt chạy nên lưới có thể còn mang Tác phẩm của spec
     * trước. Vá của **bàn đo**, không của sản phẩm.
     */
    await browser.execute(() => {
      window.location.reload()
    })
    await $('[data-col="tgt"]').waitForExist({
      timeout: 30_000,
      timeoutMsg:
        'Nạp lại webview rồi mà không thấy ô `[data-col="tgt"]` nào sau 30 giây — ' +
        'lưới không dựng được, hoặc Tác phẩm đang mở không có segment nào.',
    })

    /*
     * ═══════════════════════════════════════════════════════════════════════════════
     * 🔴 TỰ KIỂM DANH TÍNH PHIÊN — chạy TRƯỚC mọi phép đo
     * ═══════════════════════════════════════════════════════════════════════════════
     * Máy chủ WebDriver nhúng bám cổng cố định 4445. Nếu cổng đó bị một tiến trình khác
     * giữ, phiên nối vào **webview của ứng dụng khác** và **mọi phép đo vẫn trả về số**
     * (`2-5b-ban-do/README.md`, đo 2026-08-14). Đó là hình dạng hỏng tệ nhất ở chỗ này.
     * Bộ e2e thường trực **chưa có** phép kiểm này — món nợ có chủ (Story 1.22).
     */
    const danhTinh = await browser.execute(() => ({
      href: String(window.location.href),
      coApp: document.getElementById('app') !== null,
      soODich: document.querySelectorAll('[data-col="tgt"]').length,
      ua: String(navigator.userAgent),
    }))
    if (!danhTinh.href.includes('localhost:1420') || !danhTinh.coApp) {
      throw new Error(
        'LỖI HẠ TẦNG, không phải một phép kiểm đỏ: phiên nối vào một webview lạ ' +
          `(href=${danhTinh.href}, #app=${danhTinh.coApp}). Đặt TAURI_WEBDRIVER_PORT và chạy lại.`,
      )
    }
    bao.danhTinh = danhTinh

    // Ô đo: ô bản dịch đầu tiên. Bấm CHUỘT THẬT để đi đúng đường tiêu điểm của sản phẩm.
    const segId = await browser.execute(
      () => document.querySelector('[data-col="tgt"]').getAttribute('data-segment-id'),
    )
    await realClick(await $(`[data-col="tgt"][data-segment-id="${segId}"]`))
    await browser.pause(200)

    // ══════════════════════════════════════════════════════════════════════════════
    // ① `Enter` phát `beforeinput` gì? — đo bằng HAI đường, ghi HAI số
    // ══════════════════════════════════════════════════════════════════════════════
    await browser.execute(ARM)
    await browser.execute(SEED, segId, 'AB', 1)

    // ①a — phím qua driver. Giới hạn của bộ đo: chỉ `keydown`/`keyup`.
    await browser.keys('Enter')
    await browser.pause(150)
    bao.menhDe['1a_phim_driver'] = {
      log: await browser.execute(() => window.__aura25d.log.slice()),
      dom: await browser.execute(DUMP, segId),
    }

    // ①b — `execCommand('insertParagraph')`: cùng đường soạn thảo của engine.
    await browser.execute(ARM)
    await browser.execute(SEED, segId, 'AB', 1)
    const exec = await browser.execute(() => {
      const ok = document.execCommand('insertParagraph')
      return { ok }
    })
    await browser.pause(150)
    bao.menhDe['1b_execCommand_insertParagraph'] = {
      execTraVe: exec.ok,
      log: await browser.execute(() => window.__aura25d.log.slice()),
    }

    // ══════════════════════════════════════════════════════════════════════════════
    // ② engine dựng CÁI GÌ trong DOM · ③ `textContent` đọc ra gì
    //    ⇒ đây là bẫy trung tâm của Quyết định #1, đo bằng một lượt duy nhất
    // ══════════════════════════════════════════════════════════════════════════════
    const sauInsertParagraph = await browser.execute(DUMP, segId)
    bao.menhDe['2_dom_sau_insertParagraph'] = sauInsertParagraph
    bao.menhDe['3_textContent_sau_insertParagraph'] = {
      textContent: sauInsertParagraph.textContent,
      coXuongDong: sauInsertParagraph.coXuongDongTrongTextContent,
    }

    // ══════════════════════════════════════════════════════════════════════════════
    // ④ tự chèn một text node `"\n"` rồi `setPosition` sau nó — vế đo cho Quyết định #1(b)
    //    (Ice ký (c), nhưng số này vẫn phải có: nó là thứ nói (b) đúng hay sai, và nó là
    //     đường lui nếu (c) trượt ở mệnh đề ⑥.)
    // ══════════════════════════════════════════════════════════════════════════════
    await browser.execute(ARM)
    await browser.execute(SEED, segId, 'AB', 1)
    const chenTextNode = await browser.execute(() => {
      const cell = document.activeElement
      const sel = window.getSelection()
      if (sel === null || sel.rangeCount === 0) return { ok: false, detail: 'không có range' }
      const range = sel.getRangeAt(0)
      range.deleteContents()
      const node = document.createTextNode('\n')
      range.insertNode(node)
      // 🔴 `setPosition`, KHÔNG `removeAllRanges()`+`addRange()` — phép đo ở
      // `GridPanel.vue:328-336`: WebKit BỎ QUA `addRange` với editing host rỗng.
      const dat = sel.setPosition(node, 1)
      return { ok: true, detail: String(dat), soNodeCon: cell.childNodes.length }
    })
    await browser.pause(100)
    bao.menhDe['4_tu_chen_text_node'] = {
      ...chenTextNode,
      dom: await browser.execute(DUMP, segId),
    }

    // ══════════════════════════════════════════════════════════════════════════════
    // ⑥ ĐƯỜNG ICE ĐÃ KÝ — chèn `<br>`, và ba câu hỏi mà chỉ engine thật trả lời được
    // ══════════════════════════════════════════════════════════════════════════════
    // 🔴 Ba thứ phải đo, và cả ba đều có thể LẬT chữ ký #1(c):
    //    (i)  `cell.textContent` sau lượt chèn `<br>` — `textContent` **không thấy** phần
    //         tử, nên nó đọc ra `"AB"` chứ không `"A\nB"`. Nếu đúng vậy thì `reportEdit`
    //         **phải** đổi, và đó là chỗ đường (c) chạm vào đường ghi.
    //    (ii) caret sau `<br>`: đặt được không, và đặt vào đâu.
    //    (iii) một `<br>` **cuối nội dung** có vẽ ra dòng thứ hai không — bài toán
    //          "trailing `<br>`" kinh điển, và nó quyết định phép chuyển `\n` → DOM có
    //          phải thêm một `<br>` canh chót hay không.
    await browser.execute(ARM)
    await browser.execute(SEED, segId, 'AB', 1)
    const truocBr = await browser.execute(HANG, segId)
    const chenBr = await browser.execute(() => {
      const cell = document.activeElement
      const sel = window.getSelection()
      if (sel === null || sel.rangeCount === 0) return { ok: false, detail: 'không có range' }
      const range = sel.getRangeAt(0)
      range.deleteContents()
      const br = document.createElement('br')
      range.insertNode(br)
      const sau = br.nextSibling
      const dat = sau === null ? sel.setPosition(cell, 2) : sel.setPosition(sau, 0)
      return {
        ok: true,
        detail: String(dat),
        coNodeSauBr: sau !== null,
        loaiNodeSauBr: sau === null ? null : sau.nodeName,
      }
    })
    await browser.pause(120)
    bao.menhDe['6_duong_br_ICE_KY'] = {
      ...chenBr,
      dom: await browser.execute(DUMP, segId),
      hangTruoc: truocBr,
      hangSau: await browser.execute(HANG, segId),
    }

    // ⑥bis — `<br>` ở CUỐI nội dung: có vẽ ra dòng thứ hai không?
    await browser.execute(SEED, segId, 'AB', 2)
    const brCuoi = await browser.execute(() => {
      const cell = document.activeElement
      const truoc = +cell.getBoundingClientRect().height.toFixed(2)
      cell.appendChild(document.createElement('br'))
      const mot = +cell.getBoundingClientRect().height.toFixed(2)
      cell.appendChild(document.createElement('br'))
      const hai = +cell.getBoundingClientRect().height.toFixed(2)
      return { caoTruocPx: truoc, caoMotBrPx: mot, caoHaiBrPx: hai }
    })
    bao.menhDe['6bis_br_cuoi_noi_dung'] = brCuoi

    // ══════════════════════════════════════════════════════════════════════════════
    // ⑤ `white-space` — đo CẢ HAI giá trị, dù Ice đã ký `pre-line`
    // ══════════════════════════════════════════════════════════════════════════════
    // Lý do đo cả hai: chữ ký #2 chọn giữa hai **ngữ nghĩa**, không giữa hai con số. Nếu
    // hai giá trị cho hình học khác nhau thì đó là một dữ kiện Ice chưa có lúc ký.
    const ws = {}
    for (const value of ['normal', 'pre-line', 'pre-wrap']) {
      await browser.execute(SEED, segId, 'A\nB', 1)
      const doc = await browser.execute((v, id) => {
        const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
        cell.style.whiteSpace = v
        // đọc lại sau một lượt reflow cưỡng bức
        const cao = +cell.getBoundingClientRect().height.toFixed(2)
        return {
          whiteSpaceThucTe: window.getComputedStyle(cell).whiteSpace,
          caoOPx: cao,
          textContent: cell.textContent,
        }
      }, value, segId)
      ws[value] = { ...doc, hang: await browser.execute(HANG, segId) }
    }
    // Trả lại nguyên trạng — bàn đo không để lại một style nào trên sản phẩm.
    await browser.execute((id) => {
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      cell.style.whiteSpace = ''
    }, segId)
    bao.menhDe['5_white_space'] = ws

    // ══════════════════════════════════════════════════════════════════════════════
    // Bật lại hai lớp chặn của sản phẩm, rồi khẳng định chúng CÒN SỐNG
    // ══════════════════════════════════════════════════════════════════════════════
    // ⚠️ Một bàn đo tắt bảo vệ của sản phẩm mà quên bật lại là một bàn đo cho số của một
    // sản phẩm không tồn tại. Lượt kiểm này cũng là số **đối chứng** cho mệnh đề ①: cùng
    // một thao tác, một lượt có chặn và một lượt không.
    await browser.execute(() => {
      window.__aura25d.chan = false
    })
    await browser.execute(SEED, segId, 'AB', 1)
    await browser.execute(() => {
      document.execCommand('insertParagraph')
    })
    await browser.pause(120)
    bao.doiChung_conChan = await browser.execute(DUMP, segId)

    // eslint-disable-next-line no-console -- bàn đo: số đo là sản phẩm giao ra, không phải log
    console.log('\n===== BÁO CÁO BÀN ĐO 2.5D =====\n' + JSON.stringify(bao, null, 2) + '\n')

    // Bàn đo **không** phán quyết — nó chỉ phải chạy tới cuối và đưa ra số.
    await expect(bao.menhDe['2_dom_sau_insertParagraph'].ok).toBe(true)
  })

  /*
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 VÒNG 2 — VÌ THƯỚC CỦA VÒNG 1 BỊ NHIỄU, KHÔNG VÌ SỐ CỦA NÓ SAI
   * ═══════════════════════════════════════════════════════════════════════════════
   * Vòng 1 đo *"ô có vẽ ra hai dòng không"* bằng **chiều cao ô** — và cho `71,00 px` ở
   * **mọi** lượt: trước `<br>`, sau một `<br>`, sau hai `<br>`, và cả ba giá trị
   * `white-space`. Con số đó **không sai**, nó chỉ **không trả lời câu hỏi**: `subgrid`
   * đang ghim ô bản dịch theo track hàng mà **cột nguyên văn** dựng ra *(71 px ≈ 2,4 dòng
   * của font editor)*, nên một ô bản dịch hai dòng *(≈ 58 px)* vẫn **nằm gọn dưới trần**
   * và không đẩy được gì cả.
   *
   * ⇒ Đây đúng lớp bẫy mà `2-5b-ban-do/README.md` §Giới hạn ⑤ đã ghi tên một lần
   * *(`Range.getBoundingClientRect()` của một range thu gọn trả 0×0, và bản đầu đọc số 0
   * đó thành "caret sập hố")*: **một con số thật, đọc sai câu hỏi**. Vòng này đổi thước,
   * không đổi mệnh đề.
   *
   * Hai thước mới:
   *   ⓐ **số dòng hiển thị thật** = `Range.getClientRects().length` cho một Range bao trọn
   *      nội dung ô. Nó đếm **hộp dòng**, và nó **không** bị `stretch` của lưới làm nhiễu.
   *   ⓑ **áp lực ngược chiều**: rút ô nguyên văn xuống một chữ để hạ trần track, rồi đo
   *      track hàng khi ô bản dịch mang 1 · 2 · 5 dòng. Đây là chiều đo mà
   *      `deferred-work.md:3131-3162` **chưa tính**, và là chỗ Quyết định #4 phải nhìn.
   *
   * ⚠️ ⓑ **sửa nội dung ô nguyên văn trong DOM** để hạ trần. Đó là một thao tác của **bàn
   * đo trên cơ chế**, không một mệnh đề về sản phẩm — nó không ghi xuống đĩa, và lượt nạp
   * lại sau đó trả mọi thứ về chỗ cũ.
   */
  it('vòng 2 — số DÒNG hiển thị và áp lực ngược chiều lên track hàng', async () => {
    const bao = { ngay: new Date().toISOString(), menhDe: {} }

    await $('[data-col="tgt"]').waitForExist({ timeout: 30_000 })
    const segId = await browser.execute(
      () => document.querySelector('[data-col="tgt"]').getAttribute('data-segment-id'),
    )

    /** Đếm hộp dòng của nội dung một ô — thước KHÔNG bị `stretch` làm nhiễu. */
    const DONG = `
      const [id] = arguments
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      if (cell === null || cell.firstChild === null) return { ok: false }
      const r = document.createRange()
      r.selectNodeContents(cell)
      const rects = [...r.getClientRects()].map((b) => ({
        top: +b.top.toFixed(2),
        cao: +b.height.toFixed(2),
      }))
      // Hai hộp cùng một hàng chữ (một text node bị <br> cắt) vẫn là MỘT dòng nếu trùng
      // \`top\` — đếm số \`top\` KHÁC NHAU mới là số dòng.
      const tops = [...new Set(rects.map((x) => x.top))]
      return {
        ok: true,
        soHopDong: rects.length,
        soDong: tops.length,
        rects,
        whiteSpace: window.getComputedStyle(cell).whiteSpace,
        innerHTML: cell.innerHTML,
        textContent: cell.textContent,
      }
    `

    // ── ⓐ số dòng thật, ba giá trị `white-space` × hai hình dạng DOM ────────────────
    const dong = {}
    for (const [ten, dungBr] of [
      ['text_node_\\n', false],
      ['phan_tu_br', true],
    ]) {
      dong[ten] = {}
      for (const ws of ['normal', 'pre-line', 'pre-wrap']) {
        await browser.execute(
          (id, v, br) => {
            const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
            cell.style.whiteSpace = v
            if (br) {
              cell.textContent = ''
              cell.appendChild(document.createTextNode('A'))
              cell.appendChild(document.createElement('br'))
              cell.appendChild(document.createTextNode('B'))
            } else {
              cell.textContent = 'A\nB'
            }
          },
          segId,
          ws,
          dungBr,
        )
        await browser.pause(60)
        dong[ten][ws] = await browser.execute(DONG, segId)
      }
    }
    bao.menhDe['5_so_dong_hien_thi'] = dong

    // ── ⓑ áp lực NGƯỢC CHIỀU lên track hàng ────────────────────────────────────────
    const HANG2 = `
      const [id] = arguments
      const o = [...document.querySelectorAll('[data-segment-id="' + id + '"]')]
      const r = o.map((el) => ({
        col: el.dataset.col || 'meta',
        top: +el.getBoundingClientRect().top.toFixed(2),
        cao: +el.getBoundingClientRect().height.toFixed(2),
      }))
      const tops = r.map((x) => x.top)
      return {
        o: r,
        lechTopPx: +(Math.max(...tops) - Math.min(...tops)).toFixed(2),
        caoTrackPx: +Math.max(...r.map((x) => x.cao)).toFixed(2),
      }
    `

    // Hạ trần: ô nguyên văn còn một chữ.
    const nguonCu = await browser.execute((id) => {
      const src = document.querySelector('[data-col="src"][data-segment-id="' + id + '"]')
      const cu = src === null ? null : src.innerHTML
      if (src !== null) src.textContent = '一'
      return cu
    }, segId)

    const apLuc = {}
    for (const [ten, soDong] of [
      ['dich_1_dong', 1],
      ['dich_2_dong', 2],
      ['dich_5_dong', 5],
    ]) {
      await browser.execute(
        (id, n) => {
          const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
          cell.style.whiteSpace = 'pre-line'
          cell.textContent = Array.from({ length: n }, (_, i) => 'Dong ' + (i + 1)).join('\n')
        },
        segId,
        soDong,
      )
      await browser.pause(80)
      apLuc[ten] = {
        ...(await browser.execute(HANG2, segId)),
        dong: await browser.execute(DONG, segId),
      }
    }
    bao.menhDe['7_ap_luc_nguoc_chieu'] = { nguonRutXuong: '一', ketQua: apLuc }

    // Trả ô nguyên văn về nguyên trạng.
    await browser.execute(
      (id, html) => {
        const src = document.querySelector('[data-col="src"][data-segment-id="' + id + '"]')
        if (src !== null && html !== null) src.innerHTML = html
        const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
        if (cell !== null) cell.style.whiteSpace = ''
      },
      segId,
      nguonCu,
    )

    // eslint-disable-next-line no-console -- bàn đo: số đo là sản phẩm giao ra
    console.log('\n===== BÁO CÁO BÀN ĐO 2.5D · VÒNG 2 =====\n' + JSON.stringify(bao, null, 2) + '\n')

    await expect(bao.menhDe['5_so_dong_hien_thi']['phan_tu_br']['pre-line'].ok).toBe(true)
  })

  /*
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 VÒNG 3 — `<br>` Ở CUỐI NỘI DUNG, tức thao tác THƯỜNG NHẤT của tính năng
   * ═══════════════════════════════════════════════════════════════════════════════
   * Vòng 1 hỏi câu này rồi, và trả lời bằng **chiều cao ô** — cùng cái thước mà vòng 2 vừa
   * chứng minh là bị `subgrid` ghim. ⇒ Con số `caoMotBrPx = caoHaiBrPx = 71,00` của vòng 1
   * **không có nội dung**, và nó phải bị rút chứ không được đọc thành *"trailing `<br>` không
   * vẽ ra dòng nào"*.
   *
   * Vì sao câu này đáng một vòng riêng: bấm `Enter` **ở cuối câu** là thao tác thường nhất
   * của FR134 *(viết xong một đoạn rồi xuống đoạn mới)*. Nếu một `<br>` cuối **không** vẽ ra
   * dòng thứ hai thì người dùng bấm `Enter` và **màn hình không đổi gì cả** — cùng lớp lỗi
   * mà Quyết định #2 tồn tại để chặn, chỉ khác chỗ phát.
   *
   * Ba hình dạng phải phân biệt, và chúng quyết định phép chuyển `\n` → DOM:
   *   ⓐ `A<br>`      — một `<br>` trần ở cuối
   *   ⓑ `A<br><br>`  — thêm một `<br>` canh chót *(khuôn mà nhiều editor dùng)*
   *   ⓒ engine tự làm — thả `insertParagraph`/`insertLineBreak` chạy ở **cuối** nội dung và
   *      xem WebKit dựng cái gì. Đây là số đối chứng: nếu engine tự thêm một node canh chót
   *      thì phép chuyển của ta phải làm y hệt.
   */
  it('vòng 3 — `<br>` cuối nội dung có vẽ ra dòng thứ hai không', async () => {
    const bao = { ngay: new Date().toISOString(), menhDe: {} }

    await $('[data-col="tgt"]').waitForExist({ timeout: 30_000 })
    const segId = await browser.execute(
      () => document.querySelector('[data-col="tgt"]').getAttribute('data-segment-id'),
    )

    const DEM = `
      const [id] = arguments
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      const r = document.createRange()
      r.selectNodeContents(cell)
      const tops = [...new Set([...r.getClientRects()].map((b) => +b.top.toFixed(2)))]
      const sel = window.getSelection()
      return {
        soDong: tops.length,
        tops,
        innerHTML: cell.innerHTML,
        textContent: cell.textContent,
        soPhanTuCon: cell.querySelectorAll('*').length,
        theCon: [...cell.childNodes].map((n) =>
          n.nodeType === 3 ? 'TEXT(' + JSON.stringify(n.nodeValue) + ')' : n.nodeName,
        ),
        caretDong: (() => {
          if (sel === null || sel.rangeCount === 0) return null
          const c = sel.getRangeAt(0).cloneRange()
          const b = [...c.getClientRects()]
          return b.length > 0 ? +b[0].top.toFixed(2) : null
        })(),
        selectionType: sel ? sel.type : null,
      }
    `

    // Hạ trần track để số dòng đọc ra được, đúng cách vòng 2 đã làm.
    const nguonCu = await browser.execute((id) => {
      const src = document.querySelector('[data-col="src"][data-segment-id="' + id + '"]')
      const cu = src === null ? null : src.innerHTML
      if (src !== null) src.textContent = '一'
      return cu
    }, segId)

    // ⓐ một `<br>` trần ở cuối.
    await browser.execute((id) => {
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      cell.style.whiteSpace = 'pre-line'
      cell.textContent = ''
      cell.appendChild(document.createTextNode('A'))
      cell.appendChild(document.createElement('br'))
      cell.focus()
      window.getSelection().setPosition(cell, cell.childNodes.length)
    }, segId)
    await browser.pause(80)
    bao.menhDe['a_mot_br_cuoi'] = await browser.execute(DEM, segId)

    // ⓑ hai `<br>` — một canh chót.
    await browser.execute((id) => {
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      cell.textContent = ''
      cell.appendChild(document.createTextNode('A'))
      cell.appendChild(document.createElement('br'))
      cell.appendChild(document.createElement('br'))
      cell.focus()
      window.getSelection().setPosition(cell, 2)
    }, segId)
    await browser.pause(80)
    bao.menhDe['b_hai_br_cuoi'] = await browser.execute(DEM, segId)

    // ⓒ số ĐỐI CHỨNG — engine tự làm ở cuối nội dung.
    await browser.execute((id) => {
      const W = (window.__aura25d = window.__aura25d || {})
      W.chan = true
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      cell.textContent = 'A'
      cell.focus()
      window.getSelection().setPosition(cell.firstChild, 1)
    }, segId)
    const execCuoi = await browser.execute(() => ({
      ok: document.execCommand('insertParagraph'),
    }))
    await browser.pause(100)
    bao.menhDe['c_engine_tu_lam_o_cuoi'] = {
      execTraVe: execCuoi.ok,
      ...(await browser.execute(DEM, segId)),
    }

    // ⓓ và `insertLineBreak` — WebKit có thể dựng hình dạng khác hẳn `insertParagraph`.
    await browser.execute((id) => {
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      cell.textContent = 'A'
      cell.focus()
      window.getSelection().setPosition(cell.firstChild, 1)
    }, segId)
    const execLb = await browser.execute(() => ({
      ok: document.execCommand('insertLineBreak'),
    }))
    await browser.pause(100)
    bao.menhDe['d_insertLineBreak_o_cuoi'] = {
      execTraVe: execLb.ok,
      ...(await browser.execute(DEM, segId)),
    }

    // Trả nguyên trạng.
    await browser.execute(
      (id, html) => {
        const src = document.querySelector('[data-col="src"][data-segment-id="' + id + '"]')
        if (src !== null && html !== null) src.innerHTML = html
        const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
        if (cell !== null) cell.style.whiteSpace = ''
      },
      segId,
      nguonCu,
    )

    // eslint-disable-next-line no-console -- bàn đo: số đo là sản phẩm giao ra
    console.log('\n===== BÁO CÁO BÀN ĐO 2.5D · VÒNG 3 =====\n' + JSON.stringify(bao, null, 2) + '\n')

    await expect(bao.menhDe['a_mot_br_cuoi'].soDong).toBeGreaterThan(0)
  })

  /*
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 VÒNG 4 — MỘT ĐƯỜNG THỨ TƯ MÀ STORY KHÔNG LIỆT, và vòng 3 vừa đo ra nó
   * ═══════════════════════════════════════════════════════════════════════════════
   * Vòng 3 ⓓ đo được: `execCommand('insertLineBreak')` trên WKWebView, trong một ô có
   * `white-space: pre-line`, dựng **text node `\n`** — `soPhanTuCon = 0`, `textContent =
   * "A\n\n"`, **không một mảnh markup nào**. Tức chính engine làm ra đúng hình dạng mà
   * Quyết định #1 **đường (b)** muốn, và làm hộ luôn phần caret.
   *
   * Story liệt ba đường *(thả engine · tự chèn text node · `<br>` + hai phép chuyển)*. Đây
   * là **đường thứ tư**: chặn `insertParagraph` rồi **gọi `insertLineBreak` của chính
   * engine**. Nó không nằm trong bảng vì bảng được viết **trước** khi có số này.
   *
   * Vòng này đo bốn thứ, và bất kỳ thứ nào trượt cũng loại đường đó:
   *   E1 giữa nội dung, có `pre-line`  — DOM · `textContent` · caret
   *   E2 **cuối** nội dung, có `pre-line` — chỗ `<br>` trần đã trượt ở vòng 3 ⓐ
   *   E3 **không** `pre-line` — hình dạng `\n` có phụ thuộc `white-space` không?
   *      *(Nếu có: hai chữ ký #1 và #2 ràng nhau, và điều đó phải nói ra.)*
   *   E4 một vòng đĩa mô phỏng: gán `el.textContent = "A\nB"` như `restoreEditedText` làm,
   *      rồi đọc lại — vế **ngược** của phép chuyển.
   */
  it('vòng 4 — đường thứ tư: chặn `insertParagraph`, gọi `insertLineBreak` của engine', async () => {
    const bao = { ngay: new Date().toISOString(), menhDe: {} }

    await $('[data-col="tgt"]').waitForExist({ timeout: 30_000 })
    const segId = await browser.execute(
      () => document.querySelector('[data-col="tgt"]').getAttribute('data-segment-id'),
    )

    const DEM = `
      const [id] = arguments
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      const r = document.createRange()
      r.selectNodeContents(cell)
      const tops = [...new Set([...r.getClientRects()].map((b) => +b.top.toFixed(2)))]
      const sel = window.getSelection()
      return {
        soDong: tops.length,
        innerHTML: cell.innerHTML,
        textContent: cell.textContent,
        coXuongDong: cell.textContent.includes('\\n'),
        soPhanTuCon: cell.querySelectorAll('*').length,
        theCon: [...cell.childNodes].map((n) =>
          n.nodeType === 3 ? 'TEXT(' + JSON.stringify(n.nodeValue) + ')' : n.nodeName,
        ),
        selectionType: sel ? sel.type : null,
        neoTrongO: !!(sel && sel.anchorNode && cell.contains(sel.anchorNode)),
        anchorLaText: !!(sel && sel.anchorNode && sel.anchorNode.nodeType === 3),
        anchorOffset: sel ? sel.anchorOffset : null,
      }
    `

    // Hạ trần track để đọc được số dòng, đúng cách vòng 2 và 3 đã làm.
    const nguonCu = await browser.execute((id) => {
      const src = document.querySelector('[data-col="src"][data-segment-id="' + id + '"]')
      const cu = src === null ? null : src.innerHTML
      if (src !== null) src.textContent = '一'
      return cu
    }, segId)

    /**
     * Mô phỏng ĐÚNG thứ `onBeforeInput` sẽ làm ở đường thứ tư: chặn `insertParagraph`, rồi
     * gọi `insertLineBreak`. Ghi lại **mọi** `beforeinput` phát ra trong lượt đó — vì lượt
     * gọi thứ hai tự phát một sự kiện nữa, và handler thật sẽ nhìn thấy nó.
     */
    const CHAY = `
      const [id, ws, text, offset] = arguments
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      cell.style.whiteSpace = ws
      cell.textContent = text
      cell.focus()
      window.getSelection().setPosition(cell.firstChild, offset)

      const seen = []
      const on = (e) => { seen.push({ inputType: e.inputType, cancelable: e.cancelable }) }
      document.addEventListener('beforeinput', on, true)
      const ok = document.execCommand('insertLineBreak')
      document.removeEventListener('beforeinput', on, true)
      return { execTraVe: ok, suKien: seen }
    `

    // E1 — giữa nội dung, có `pre-line`.
    bao.menhDe['E1_giua_noi_dung_pre_line'] = {
      ...(await browser.execute(CHAY, segId, 'pre-line', 'AB', 1)),
      dom: await browser.execute(DEM, segId),
    }

    // E2 — CUỐI nội dung, có `pre-line`. Chỗ `<br>` trần trượt ở vòng 3 ⓐ.
    bao.menhDe['E2_cuoi_noi_dung_pre_line'] = {
      ...(await browser.execute(CHAY, segId, 'pre-line', 'A', 1)),
      dom: await browser.execute(DEM, segId),
    }

    // E3 — KHÔNG `pre-line`: hình dạng `\n` có phụ thuộc `white-space` không?
    bao.menhDe['E3_giua_noi_dung_normal'] = {
      ...(await browser.execute(CHAY, segId, 'normal', 'AB', 1)),
      dom: await browser.execute(DEM, segId),
    }

    // E4 — vế NGƯỢC: `el.textContent = "A\nB"` như `restoreEditedText` làm.
    await browser.execute(
      (id) => {
        const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
        cell.style.whiteSpace = 'pre-line'
        cell.textContent = 'A\nB'
      },
      segId,
    )
    await browser.pause(80)
    bao.menhDe['E4_vong_nguoc_restoreEditedText'] = await browser.execute(DEM, segId)

    // Trả nguyên trạng.
    await browser.execute(
      (id, html) => {
        const src = document.querySelector('[data-col="src"][data-segment-id="' + id + '"]')
        if (src !== null && html !== null) src.innerHTML = html
        const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
        if (cell !== null) cell.style.whiteSpace = ''
      },
      segId,
      nguonCu,
    )

    // eslint-disable-next-line no-console -- bàn đo: số đo là sản phẩm giao ra
    console.log('\n===== BÁO CÁO BÀN ĐO 2.5D · VÒNG 4 =====\n' + JSON.stringify(bao, null, 2) + '\n')

    await expect(bao.menhDe['E1_giua_noi_dung_pre_line'].dom.selectionType).toBe('Caret')
  })

  /*
   * ═══════════════════════════════════════════════════════════════════════════════
   * VÒNG 5 — BA HÌNH DẠNG HIỂN THỊ CỜ ĐÍCH (Quyết định #4 đường (c): ĐO TRƯỚC, QUYẾT SAU)
   * ═══════════════════════════════════════════════════════════════════════════════
   * Ice ký #4(c) ngày 2026-08-15: **không** chọn hình dạng, mà **đo ba hình dạng rồi mới
   * hỏi lại**. Vòng này là phép đo đó.
   *
   * Ràng buộc đã đo ở vòng 2 ⓑ và nó là thứ loại hình dạng: năm cột là năm `subgrid` chia
   * **chung một tập track hàng**, `.cell` mặc định `align-self: stretch`, nên
   * **track = max(chiều cao các ô cùng hàng)**. Vòng 2 đo bằng **nội dung** (38 → 63 → 150
   * px). Vòng này đo bằng **padding**, tức đúng cơ chế mà `.cell.para-end` dùng — và đó là
   * bước suy một-bậc mà README vòng 2 đã ghi ra là **chưa đo**.
   *
   * Ba hình dạng:
   *   Ⓐ `padding-bottom: 14px` **chỉ** ở ô bản dịch — khuôn `.cell.para-end` của cờ nguồn
   *   Ⓑ đổi **kiểu đường kẻ đáy** ở ô bản dịch (phi hình học)
   *   Ⓒ một **ký tự** ở cột nhãn trạng thái (phi hình học, ngoài ô bản dịch)
   *
   * Số cần đọc ở mỗi hình dạng: chiều cao **track hàng** và **lệch `top`** giữa các cột.
   */
  it('vòng 5 — ba hình dạng hiển thị cờ đích, và cái giá hình học của từng cái', async () => {
    const bao = { ngay: new Date().toISOString(), hinhDang: {} }

    await $('[data-col="tgt"]').waitForExist({ timeout: 30_000 })
    const segId = await browser.execute(
      () => document.querySelector('[data-col="tgt"]').getAttribute('data-segment-id'),
    )

    const DO = `
      const [id] = arguments
      const o = [...document.querySelectorAll('[data-segment-id="' + id + '"]')]
      const r = o.map((el) => ({
        col: el.dataset.col || 'meta',
        top: +el.getBoundingClientRect().top.toFixed(2),
        cao: +el.getBoundingClientRect().height.toFixed(2),
      }))
      const tops = r.map((x) => x.top)
      return {
        caoTrackPx: +Math.max(...r.map((x) => x.cao)).toFixed(2),
        lechTopPx: +(Math.max(...tops) - Math.min(...tops)).toFixed(2),
        o: r,
      }
    `

    // Nen chung: o nguyen van rut ngan de tran track thap, o ban dich mot dong.
    const nguonCu = await browser.execute((id) => {
      const src = document.querySelector('[data-col="src"][data-segment-id="' + id + '"]')
      const cu = src === null ? null : src.innerHTML
      if (src !== null) src.textContent = '一'
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      cell.textContent = 'Mot dong.'
      return cu
    }, segId)
    await browser.pause(80)
    bao.nen = await browser.execute(DO, segId)

    // ── Ⓐ `padding-bottom` chỉ ở ô bản dịch — khuôn `.cell.para-end` ────────────────
    await browser.execute((id) => {
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      cell.style.paddingBottom = '14px'
    }, segId)
    await browser.pause(80)
    bao.hinhDang['A_padding_bottom_chi_o_ban_dich'] = await browser.execute(DO, segId)
    await browser.execute((id) => {
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      cell.style.paddingBottom = ''
    }, segId)

    // ── Ⓑ đổi KIỂU đường kẻ đáy ở ô bản dịch (phi hình học) ────────────────────────
    await browser.execute((id) => {
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      cell.style.borderBottomStyle = 'double'
      cell.style.borderBottomWidth = '1px'
    }, segId)
    await browser.pause(80)
    bao.hinhDang['B_doi_kieu_duong_ke_day'] = await browser.execute(DO, segId)
    await browser.execute((id) => {
      const cell = document.querySelector('[data-col="tgt"][data-segment-id="' + id + '"]')
      cell.style.borderBottomStyle = ''
      cell.style.borderBottomWidth = ''
    }, segId)

    // ── Ⓒ một KÝ TỰ ở cột nhãn trạng thái ─────────────────────────────────────────
    const daDatKyTu = await browser.execute((id) => {
      // Cot nhan trang thai la cot cuoi; no khong mang `data-segment-id`, nen lay theo vi tri
      // hang trong chinh cot do.
      const tgt = [...document.querySelectorAll('[data-col="tgt"]')]
      const index = tgt.findIndex((el) => el.getAttribute('data-segment-id') === String(id))
      const state = document.querySelector('.col-state')
      const cell = state === null ? null : state.children[index]
      if (cell === undefined || cell === null) return false
      cell.dataset.auraBenchOld = cell.textContent
      cell.textContent = '¶'
      return true
    }, segId)
    await browser.pause(80)
    bao.hinhDang['C_ky_tu_o_cot_nhan_trang_thai'] = {
      datDuoc: daDatKyTu,
      ...(await browser.execute(DO, segId)),
    }

    // Tra nguyen trang.
    await browser.execute(
      (id, html) => {
        const src = document.querySelector('[data-col="src"][data-segment-id="' + id + '"]')
        if (src !== null && html !== null) src.innerHTML = html
        const state = document.querySelector('.col-state')
        if (state !== null) {
          for (const c of state.children) {
            if (c.dataset.auraBenchOld !== undefined) {
              c.textContent = c.dataset.auraBenchOld
              delete c.dataset.auraBenchOld
            }
          }
        }
      },
      segId,
      nguonCu,
    )

    // eslint-disable-next-line no-console -- bàn đo: số đo là sản phẩm giao ra
    console.log('\n===== BÁO CÁO BÀN ĐO 2.5D · VÒNG 5 =====\n' + JSON.stringify(bao, null, 2) + '\n')

    await expect(bao.nen.caoTrackPx).toBeGreaterThan(0)
  })
})
