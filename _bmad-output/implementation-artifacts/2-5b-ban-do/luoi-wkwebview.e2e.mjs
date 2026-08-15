/**
 * Bàn đo Story 2.5b · Task 1.2 — NĂM MỆNH ĐỀ TRÊN **WKWebView THẬT**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO TỆP NÀY KHÔNG NẰM TRONG `e2e/specs/`
 * ═════════════════════════════════════════════════════════════════════════════════
 * Nó **không kiểm sản phẩm** — sản phẩm chưa có lưới nào ở thời điểm chạy. Nó tiêm một
 * hình dạng DOM **giả** vào webview của app để hỏi engine năm câu hỏi. Đặt nó vào bộ e2e
 * thường trực là dựng một ca sẽ xanh mãi mãi mà không canh một mệnh đề sản phẩm nào —
 * đúng thứ §Testing Rules gọi là *"dựng nguồn sự thật thứ hai"*.
 * ⇒ Nó sống cạnh bàn đo, và chạy bằng `--spec`:
 *
 *     npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-5b-ban-do/luoi-wkwebview.e2e.mjs
 *
 * Ca **thường trực** cho cùng địa hạt là Task 12.2, và nó chạy trên `GridPanel.vue` thật.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO PHẢI LÀ WKWebView CHỨ KHÔNG PLAYWRIGHT-WEBKIT
 * ═════════════════════════════════════════════════════════════════════════════════
 * Story 2.3 trả giá đúng chỗ này: Playwright-WebKit **có** tạo vùng chọn ở lượt bấm vào
 * văn bản chỉ-đọc, WKWebView **không** (`EditorPanel.vue:464-479`). Một bàn đo Playwright
 * vì thế cho lượt XANH trên một sản phẩm mà chuột thật không dùng được. Bàn đo
 * `2-5b-ban-do-luoi.html` trả lời Task 1.3 (Chromium ≈ WebView2); tệp này trả lời 1.2.
 *
 * ⚠️ **GIỚI HẠN, ghi ra thay vì để người sau vấp:** `browser.keys()` chỉ bắn
 * `keydown`/`keyup` và **không** đi vào đường nhập văn bản gốc của WKWebView — đo
 * 2026-08-12, `EditorPanel.vue:492-493`. ⇒ mệnh đề ②③ ở đây đi qua `execCommand`, cùng
 * đường soạn thảo của engine nhưng **không** cùng đường phím vật lý. Vế phím vật lý là
 * Task 1.4, và **chủ của nó là Ice**.
 */
import { realClick } from '../../../e2e/support/pointer.mjs'

/**
 * Hình dạng tiêm vào: lưới chủ-cột `subgrid`, năm cột, ô bản dịch là editing host RIÊNG.
 *
 * 🔴 Chép **hình dạng**, không chép component — cùng giới hạn ① mà mọi bàn đo của kho
 * này mang: một lượt sửa `GridPanel.vue` sau này có thể làm hai bên lệch nhau mà không
 * cổng nào đỏ.
 */
const INJECT = `
  const old = document.getElementById('aura-bench-2-5b')
  if (old !== null) old.remove()

  const host = document.createElement('div')
  host.id = 'aura-bench-2-5b'
  host.setAttribute('style', 'position:fixed;inset:0;z-index:99999;background:#fbfaf6;color:#2b2723;overflow:auto;padding:16px;')

  const style = document.createElement('style')
  style.textContent = \`
    #aura-bench-2-5b .grid { display:grid; grid-template-columns:3px 30px minmax(0,1fr) minmax(0,1fr) 96px; align-items:stretch; }
    #aura-bench-2-5b .col { display:grid; grid-template-rows:subgrid; grid-row:1 / -1; }
    #aura-bench-2-5b .cell { padding:2px 8px; border-bottom:1px solid #e2dccf; font:15px/1.95 Georgia, serif; }
    #aura-bench-2-5b .col-src .cell { font:16.5px/2.05 'Songti SC', Georgia, serif; }
    #aura-bench-2-5b .col-tgt .cell.empty { min-height:1.95em; border-bottom:1px dashed #e2dccf; }
    #aura-bench-2-5b .col-tgt .cell:focus { outline:none; background:#efe9dc; }
  \`
  host.appendChild(style)

  const ROWS = [
    { n:1, src:'第一句话，用来占位。', tgt:'Cau thu nhat.' },
    { n:2, src:'第二句稍微长一点。', tgt:'Cau thu hai.' },
    { n:3, src:'第三句还没有翻译。', tgt:'' },
    { n:4, src:'第四句非常长，长到必须折行好几次，这样才能把两个单元格的高度差别做出来，用来检验 subgrid 是否真的让整行对齐。', tgt:'Ngan.' },
  ]

  const grid = document.createElement('div')
  grid.className = 'grid'
  grid.style.gridTemplateRows = 'repeat(' + ROWS.length + ', auto)'
  const COLS = ['col-rule','col-num','col-src','col-tgt','col-state']
  for (const cls of COLS) {
    const col = document.createElement('div')
    col.className = 'col ' + cls
    for (const r of ROWS) {
      const cell = document.createElement('div')
      cell.className = 'cell'
      cell.dataset.segmentId = String(r.n)
      cell.dataset.col = cls === 'col-src' ? 'src' : cls === 'col-tgt' ? 'tgt' : 'meta'
      if (cls === 'col-tgt') {
        cell.setAttribute('contenteditable', 'true')
        cell.textContent = r.tgt
        if (r.tgt === '') cell.classList.add('empty')
      } else if (cls === 'col-src') {
        cell.textContent = r.src
      } else if (cls === 'col-num') {
        cell.textContent = String(r.n)
      } else {
        cell.textContent = ''
      }
      col.appendChild(cell)
    }
    grid.appendChild(col)
  }
  host.appendChild(grid)
  document.body.appendChild(host)

  window.__auraBench = {
    log: [],
    cell: () => document.querySelector('#aura-bench-2-5b [data-col="tgt"][data-segment-id="3"]'),
  }
  const rec = (e) => {
    const t = e.target
    window.__auraBench.log.push({
      loai: e.type,
      inputType: e.inputType === undefined ? null : e.inputType,
      cancelable: e.cancelable,
      tren_o_rong: !!(t && t.dataset && t.dataset.col === 'tgt' && t.dataset.segmentId === '3'),
    })
  }
  document.addEventListener('beforeinput', rec, true)
  document.addEventListener('input', rec, true)

  window.__auraBench.chuot = []
  const desc = (t) => {
    if (!t || !t.tagName) return String(t)
    const d = t.dataset || {}
    return t.tagName + (t.className ? '.' + String(t.className).trim().split(/\\s+/).join('.') : '') +
      (d.col ? '[col=' + d.col + ' id=' + d.segmentId + ']' : '')
  }
  for (const type of ['pointerdown', 'mousedown', 'focusin', 'mouseup', 'click', 'selectstart', 'selectionchange']) {
    document.addEventListener(type, (e) => {
      window.__auraBench.chuot.push({ loai: type, target: desc(e.target), x: e.clientX === undefined ? null : e.clientX, y: e.clientY === undefined ? null : e.clientY })
    }, true)
  }
  return { subgrid_khai_bao_duoc: CSS.supports('grid-template-rows', 'subgrid') }
`

describe('Bàn đo 2.5b — lưới subgrid + ô gõ được, trong WKWebView thật', () => {
  it('trả lời năm mệnh đề của Task 1.2', async () => {
    /*
     * 🔴 TỰ KIỂM DANH TÍNH PHIÊN — vòng chẩn đoán 3, và nó phải chạy TRƯỚC mọi phép đo.
     *
     * Vòng 2 đọc được `document.activeElement = BUTTON.sidebar-folder-tree__chevron`, một
     * lớp CSS **không tồn tại trong kho này** (`grep` toàn cây: 0 kết quả). Một phép đo
     * chạy trong webview của một ứng dụng KHÁC là một **lỗi hạ tầng**, không một câu trả
     * lời về sản phẩm — và nó là hình dạng hỏng tệ nhất có thể ở chỗ này, vì mọi số vẫn
     * đọc ra như số thật.
     *
     * ⚠️ `wdio.conf.mjs` ghi rằng máy chủ nhúng **bám cổng cố định 4445**. Một ứng dụng
     * khác đang giữ cổng đó ⇒ phiên nối vào nó. Đó là mệnh đề phép kiểm dưới đây canh.
     */
    const danhTinh = await browser.execute(() => ({
      url: String(window.location.href),
      title: document.title,
      co_cau_ipc: window.__TAURI_INTERNALS__ !== undefined,
      co_goc_app: document.getElementById('app') !== null,
    }))
    console.log('\n[2.5b] danh tinh phien:', JSON.stringify(danhTinh))
    if (!danhTinh.url.includes('localhost:1420') || !danhTinh.co_goc_app) {
      throw new Error(
        '[ha tang] Phien KHONG noi vao webview cua AuraTranslate:\n' +
          JSON.stringify(danhTinh, null, 1) +
          '\n\nDay la LOI HA TANG, khong phai mot ket qua — dung doc bat ky so nao cua luot\n' +
          'chay nay. Nguyen nhan hay gap: mot ung dung khac dang giu cong 4445 (may chu\n' +
          'nhung cua `tauri-plugin-wdio-webdriver` bam cong co dinh — xem `wdio.conf.mjs`).',
      )
    }

    const setup = await browser.execute(new Function(INJECT))
    console.log('[2.5b] subgrid khai bao duoc:', setup.subgrid_khai_bao_duoc)

    // ── ⑤ `subgrid` giữ hàng thẳng khi hai ô cùng hàng lệch chiều cao ─────────────
    const hinhHoc = await browser.execute(() => {
      const cols = [...document.querySelectorAll('#aura-bench-2-5b .col')]
      const n = cols[0].children.length
      const hang = []
      for (let i = 0; i < n; i += 1) {
        const rects = cols.map((c) => c.children[i].getBoundingClientRect())
        const tops = rects.map((r) => r.top)
        const bots = rects.map((r) => r.bottom)
        hang.push({
          hang: i + 1,
          lech_top_px: +(Math.max(...tops) - Math.min(...tops)).toFixed(2),
          lech_bottom_px: +(Math.max(...bots) - Math.min(...bots)).toFixed(2),
          cao_px: +(Math.max(...bots) - Math.min(...tops)).toFixed(2),
        })
      }
      const empty = window.__auraBench.cell()
      const filled = document.querySelector('#aura-bench-2-5b [data-col="tgt"][data-segment-id="1"]')
      return {
        hang,
        lech_top_lon_nhat_px: +Math.max(...hang.map((h) => h.lech_top_px)).toFixed(2),
        cao_o_rong_px: +empty.getBoundingClientRect().height.toFixed(2),
        cao_o_co_chu_px: +filled.getBoundingClientRect().height.toFixed(2),
      }
    })
    console.log('[2.5b] (5) hang thang — lech top lon nhat:', hinhHoc.lech_top_lon_nhat_px, 'px')
    console.log('[2.5b]     chieu cao tung hang:', hinhHoc.hang.map((h) => h.cao_px).join(' / '))
    console.log(
      '[2.5b] (4) o RONG cao',
      hinhHoc.cao_o_rong_px,
      'px · o CO CHU cao',
      hinhHoc.cao_o_co_chu_px,
      'px',
    )

    // ── ① CHUỘT THẬT vào ô RỖNG ───────────────────────────────────────────────────
    //
    // 🔴 `realClick`, KHÔNG `.click()` của driver — `e2e/support/pointer.mjs`. Tệp này
    // nằm ngoài `e2e/**` nên `no-restricted-syntax` **không** phủ nó; luật thì vẫn phủ,
    // và lý do còn mạnh hơn ở đây: cả mệnh đề ① là một mệnh đề về **thứ tự sự kiện**.
    const emptyCell = await $('#aura-bench-2-5b [data-col="tgt"][data-segment-id="3"]')
    await realClick(emptyCell)

    const sauKhiBam = await browser.execute(() => {
      const cell = window.__auraBench.cell()
      const sel = window.getSelection()
      const ae = document.activeElement
      const r = cell.getBoundingClientRect()
      const giua = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2)
      return {
        active_la_o_rong: ae === cell,
        active_tag: ae ? ae.tagName + (ae.className ? '.' + String(ae.className).trim().split(/\s+/).join('.') : '') : null,
        selection_type: sel ? sel.type : null,
        range_count: sel ? sel.rangeCount : null,
        neo_trong_o: !!(sel && sel.anchorNode && cell.contains(sel.anchorNode)),
        // Vòng chẩn đoán 2 — ba dữ kiện phân biệt "engine không focus" với "bấm trượt ô".
        o_rect: { x: +r.x.toFixed(1), y: +r.y.toFixed(1), w: +r.width.toFixed(1), h: +r.height.toFixed(1) },
        phan_tu_o_giua_o: giua === cell ? '(chinh o)' : giua ? giua.tagName + '.' + String(giua.className || '') : null,
        cua_so_co_tieu_diem: document.hasFocus(),
        su_kien_chuot: window.__auraBench.chuot.slice(),
      }
    })
    console.log('[2.5b] (1a) CONTENTEDITABLE TRAN, sau CHUOT THAT:', JSON.stringify(sauKhiBam, null, 1))

    // ══════════════════════════════════════════════════════════════════════════════
    // ① vòng hai — CÙNG cú bấm, nhưng có ĐƯỜNG CHUỘT CỦA SẢN PHẨM
    // ══════════════════════════════════════════════════════════════════════════════
    //
    // 🔴 Vòng một ở trên là một **kết quả phải ở lại trong sổ**, không một bước hỏng:
    // `contenteditable` **trần** trong WKWebView không nhận tiêu điểm và không tạo vùng
    // chọn khi bấm chuột thật — cùng họ với Story 1.22-C2 (`<button>`) và Story 2.3
    // (`<span>`). ⇒ `GridPanel.vue` **KHÔNG** được bỏ đường chuột chỉ vì mỗi ô nay là một
    // editing host riêng.
    //
    // Nhưng vòng một **không** trả lời câu hỏi của Quyết định #6: đường (a) là
    // *"`contenteditable` trần **+ khuôn `EditorPanel.vue` đã chạy**"*, không phải *"không
    // handler nào"*. Vòng hai chép đúng ba mảnh đã thắng của Story 2.3:
    //   ① `setPosition`, KHÔNG `removeAllRanges()+addRange()`  (`EditorPanel.vue:539-556`)
    //   ② đặt caret ở `mouseup`, không ở `mousedown`           (`:674-682`)
    //   ③ vá lại ở frame kế tiếp nếu engine thu vùng chọn về 0  (`:708-724`)
    await browser.execute(() => {
      const grid = document.querySelector('#aura-bench-2-5b .grid')
      const setCaret = (node, offset) => {
        const sel = window.getSelection()
        if (sel === null) return false
        sel.setPosition(node, offset)
        return sel.type === 'Caret'
      }
      const placeCaretAtPoint = (cell, x, y) => {
        const range = (() => {
          const pos = document.caretPositionFromPoint ? document.caretPositionFromPoint(x, y) : null
          if (pos !== null && pos !== undefined) {
            const r = document.createRange()
            r.setStart(pos.offsetNode, pos.offset)
            r.collapse(true)
            return r
          }
          return document.caretRangeFromPoint ? document.caretRangeFromPoint(x, y) : null
        })()
        if (range === null || range === undefined) return false
        if (!cell.contains(range.startContainer)) return false
        return setCaret(range.startContainer, range.startOffset)
      }
      grid.addEventListener('mouseup', (e) => {
        const t = e.target
        const cell = t && t.closest ? t.closest('[data-col="tgt"]') : null
        if (cell === null) return
        if (placeCaretAtPoint(cell, e.clientX, e.clientY)) return
        // Không phân giải được điểm bấm vào TRONG ô ⇒ neo vào CUỐI ô. Ca thường nhất: ô rỗng.
        setCaret(cell, cell.childNodes.length)
        requestAnimationFrame(() => {
          if (!cell.isConnected) return
          if ((window.getSelection() || {}).type === 'Caret') return
          setCaret(cell, cell.childNodes.length)
        })
      })
      window.__auraBench.chuot = []
    })

    await realClick(emptyCell)
    const sauKhiBam2 = await browser.execute(() => {
      const cell = window.__auraBench.cell()
      const sel = window.getSelection()
      const ae = document.activeElement
      return {
        active_la_o_rong: ae === cell,
        active_tag: ae ? ae.tagName + (ae.className ? '.' + String(ae.className).trim().split(/\s+/).join('.') : '') : null,
        selection_type: sel ? sel.type : null,
        range_count: sel ? sel.rangeCount : null,
        neo_trong_o: !!(sel && sel.anchorNode && cell.contains(sel.anchorNode)),
        su_kien_chuot: window.__auraBench.chuot.map((r) => r.loai).join(' → '),
      }
    })
    console.log('[2.5b] (1b) CO DUONG CHUOT SAN PHAM:', JSON.stringify(sauKhiBam2, null, 1))

    // ── ② gõ một ký tự vào ô rỗng ────────────────────────────────────────────────
    //
    // ⚠️ `execCommand`, không phím vật lý — xem §Giới hạn ở đầu tệp.
    const go = await browser.execute(() => {
      const cell = window.__auraBench.cell()
      window.__auraBench.log = []
      const ok = document.execCommand('insertText', false, 'A')
      return {
        exec_tra_ve: ok,
        van_ban: cell.textContent,
        chu_ha_canh: cell.textContent === 'A',
        su_kien: window.__auraBench.log.slice(),
      }
    })
    console.log('[2.5b] (2) go mot ky tu:', JSON.stringify(go))

    // ── ③ `Backspace` ở offset 0 ─────────────────────────────────────────────────
    //
    // Hai ca, và chúng KHÁC nhau: ⓐ đầu một ô **CÓ CHỮ** — đây là cử chỉ gộp thật của
    // UX-DR32 / Story 2.9; ⓑ một ô **đã rỗng**.
    const xoa = await browser.execute(() => {
      const cell = window.__auraBench.cell()
      const sel = window.getSelection()

      cell.textContent = 'Xin'
      cell.focus()
      sel.setPosition(cell.firstChild, 0)
      window.__auraBench.log = []
      const a = document.execCommand('delete')
      const caA = {
        exec_tra_ve: a,
        van_ban: cell.textContent,
        su_kien: window.__auraBench.log.slice(),
        type: sel.type,
      }

      cell.textContent = ''
      cell.focus()
      sel.setPosition(cell, 0)
      window.__auraBench.log = []
      const b = document.execCommand('delete')
      const caB = {
        exec_tra_ve: b,
        van_ban: cell.textContent,
        su_kien: window.__auraBench.log.slice(),
        type: sel.type,
      }
      return { dau_o_co_chu: caA, o_da_rong: caB }
    })
    console.log('[2.5b] (3a) BACKSPACE dau mot o CO CHU:', JSON.stringify(xoa.dau_o_co_chu))
    console.log('[2.5b] (3b) BACKSPACE trong mot o DA RONG:', JSON.stringify(xoa.o_da_rong))

    // ── Dọn: gỡ lớp phủ để lượt sau (nếu có) thấy app thật ────────────────────────
    await browser.execute(() => {
      const el = document.getElementById('aura-bench-2-5b')
      if (el !== null) el.remove()
    })

    // 🔴 LUẬT DỪNG của story sống ở ĐÂY, không ở một bảng đọc bằng mắt: ① và ② là hai
    // mệnh đề mà nếu trượt thì 2.5b quay về `backlog`.
    if (!sauKhiBam2.neo_trong_o || sauKhiBam2.selection_type !== 'Caret') {
      throw new Error(
        'MENH DE (1) TRUOT — bam chuot that vao mot o RONG khong dat duoc con tro trong WKWebView,\n' +
          'KE CA voi duong chuot cua san pham (setPosition o mouseup + mot luot va lai).\n' +
          JSON.stringify(sauKhiBam2, null, 1) +
          '\n\nDay la dieu kien DUNG cua Story 2.5b (§Dieu kien khoi hanh): bao Ice, dua 2.5b ve `backlog`.',
      )
    }
    if (!go.chu_ha_canh) {
      throw new Error(
        'MENH DE (2) TRUOT — go mot ky tu vao o RONG khong ha canh trong WKWebView.\n' +
          JSON.stringify(go, null, 1),
      )
    }
  })
})
