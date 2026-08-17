/**
 * Bàn đo Story 2.9 · Task 1 — **`Backspace` ở đầu ô bản dịch trên WKWebView thật**
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO BÀN ĐO NÀY CHẶN TASK 2
 * ═════════════════════════════════════════════════════════════════════════════════
 * Story 2.9 cắm một thao tác **phá huỷ và không lui được** (`AD-5`: gộp = về hưu + tạo mới,
 * và `AC5`/`⌘Z` đang là **món nợ** chờ `AD-48`) vào một phím người dùng bấm **hàng trăm lần
 * mỗi Chương**. Ba mệnh đề dưới đây quyết định hình dạng của nhánh đó, và **cả ba đều đã có
 * một lượt đo bác một giả định "hiển nhiên"** ở story trước:
 *
 *   ① `Backspace` ở offset 0 **KHÔNG phát `beforeinput`** trên WebKit — 0 sự kiện, cả
 *      WKWebView lẫn Playwright-WebKit *(`deferred-work.md:3036-3061`, đo 2026-08-14)*.
 *      Blink thì **có**. ⇒ `onBeforeInput` không dùng được làm điểm móc; đường còn lại là
 *      `keydown`. **Bàn đo này xác nhận lại vế đó trên cây HÔM NAY**, không tin ảnh chụp.
 *   ② `document.caretPositionFromPoint` **NÉM `TypeError`** trên WKWebView này — một khuyết
 *      tật của mã **đã phát hành** mà ba vòng chẩn đoán của 2.3 và 2.5b không thấy, vì caret
 *      **vẫn hiện** *(từ `cell.focus()` + hành vi mặc định của engine)*.
 *   ③ `browser.keys(['Meta','/'])` giao `code: "/"`, **không** `"Slash"` — một giới hạn của
 *      **bàn đo** đã nói dối dev ba vòng liền ở 2.8.
 *
 * ⚠️ **BÀI HỌC ③ CỦA 2.8 ÁP THẲNG VÀO ĐÂY, và nó đổi thiết kế của bàn đo này:** *"một
 * listener chẩn đoán phải CHỊU ĐƯỢC engine mà nó đang đo"*. Ở 2.8, listener của chính bàn đo
 * gọi `caretPositionFromPoint` trần ⇒ nó **ném trước dòng `push`** ⇒ mảng rỗng ở mọi vòng, và
 * dev đọc con số rỗng ấy thành một mệnh đề **về engine**. ⇒ Mọi listener dưới đây bọc
 * `try/catch` và **không gọi một API nào có thể vắng mặt**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 MỘT PHÂN BIỆT LÀ TRỤC CỦA CẢ BÀN ĐO: sự kiện TỔNG HỢP không có DEFAULT ACTION
 * ═════════════════════════════════════════════════════════════════════════════════
 * `new KeyboardEvent('keydown', …)` bắn bằng `dispatchEvent` mang `isTrusted === false`, và
 * một sự kiện không tin cậy **không sinh ra hành vi mặc định của engine** — nó không xoá một
 * ký tự nào. ⇒ `preventDefault()` trên nó là một lượt gọi **vô nghĩa**, và một bàn đo hỏi
 * *"preventDefault có chặn được lượt xoá không"* bằng sự kiện tổng hợp sẽ trả lời **CÓ** trên
 * mọi engine, kể cả engine không cho chặn.
 *
 * ⇒ Ⓒ và Ⓓ dùng **phím VẬT LÝ** (`browser.keys`) — đường duy nhất có default action.
 *   Ⓗ dùng sự kiện tổng hợp, và nó ở đây **để đo chính chỗ lệch đó**, vì spec sản phẩm
 *   (`segment-merge-split.e2e.mjs`) buộc phải dùng sự kiện tổng hợp *(giới hạn ③)*.
 *
 *     TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-9-ban-do/backspace-dau-o-wkwebview.e2e.mjs
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { realClick } from '../../../e2e/support/pointer.mjs'

/** Mã phím `Backspace` của WebDriver (`U+E003`). */
const BACKSPACE = ''

/**
 * Cắm một listener `keydown` ghi **mọi trường quyết định nhánh**, và tuỳ chọn
 * `preventDefault()`.
 *
 * 🔴 Bọc `try/catch` và **không** gọi API nào có thể vắng — bài học ③ của 2.8.
 *
 * @param {boolean} chan `true` ⇒ listener gọi `preventDefault()`
 */
async function camListener(chan) {
  await browser.execute((coChan) => {
    window.__b29 = { keydown: [], beforeinput: [], input: [], loi: [] }
    const col = document.querySelector('.col-tgt')
    if (col === null) {
      window.__b29.loi.push('khong tim thay .col-tgt')
      return
    }
    window.__b29Go = (e) => {
      try {
        window.__b29.keydown.push({
          key: e.key,
          code: e.code,
          isComposing: e.isComposing,
          repeat: e.repeat,
          isTrusted: e.isTrusted,
          // 🔴 `cancelable` là thứ nói preventDefault CÓ NGHĨA hay không. Một sự kiện
          // `cancelable: false` nuốt lượt gọi trong im lặng.
          cancelable: e.cancelable,
          t: Math.round(performance.now()),
        })
        if (coChan && e.key === 'Backspace') e.preventDefault()
      } catch (err) {
        window.__b29.loi.push('keydown: ' + String(err))
      }
    }
    window.__b29Bi = (e) => {
      try {
        window.__b29.beforeinput.push({ inputType: e.inputType, cancelable: e.cancelable })
      } catch (err) {
        window.__b29.loi.push('beforeinput: ' + String(err))
      }
    }
    window.__b29In = (e) => {
      try {
        window.__b29.input.push({ inputType: e.inputType ?? null })
      } catch (err) {
        window.__b29.loi.push('input: ' + String(err))
      }
    }
    // 🔴 `capture: true` — bàn đo phải thấy sự kiện KỂ CẢ khi một handler sản phẩm ở
    // giai đoạn bubble gọi `stopPropagation()`. Đo trước, kết luận sau.
    col.addEventListener('keydown', window.__b29Go, true)
    col.addEventListener('beforeinput', window.__b29Bi, true)
    col.addEventListener('input', window.__b29In, true)
  }, chan)
}

/** Gỡ listener để bước sau không đọc nhầm số của bước trước. */
async function goListener() {
  await browser.execute(() => {
    const col = document.querySelector('.col-tgt')
    if (col === null) return
    if (window.__b29Go) col.removeEventListener('keydown', window.__b29Go, true)
    if (window.__b29Bi) col.removeEventListener('beforeinput', window.__b29Bi, true)
    if (window.__b29In) col.removeEventListener('input', window.__b29In, true)
  })
}

/**
 * Chụp hình dạng `Selection` **và** hình dạng DOM của ô đang mang caret.
 *
 * Đây là trả lời cho Task 1.3, và nó phải trả về `startContainer` **theo loại node**, không
 * theo tham chiếu — một tham chiếu DOM không đi qua được ranh giới `browser.execute`.
 */
async function chupCaret(nhan) {
  return await browser.execute((n) => {
    try {
      const sel = window.getSelection()
      const r = sel && sel.rangeCount > 0 ? sel.getRangeAt(0) : null
      const sc = r ? r.startContainer : null
      const cellCuaNeo =
        sc instanceof Element ? sc.closest('[data-col]') : (sc?.parentElement?.closest('[data-col]') ?? null)
      const active = document.activeElement
      return {
        buoc: n,
        taiLieuCoTieuDiem: document.hasFocus(),
        selectionType: sel ? sel.type : null,
        rangeCount: sel ? sel.rangeCount : 0,
        collapsed: r ? r.collapsed : null,
        startOffset: r ? r.startOffset : null,
        // 3 = TEXT_NODE, 1 = ELEMENT_NODE. Ca ô RỖNG cho ELEMENT (chính ô).
        startContainerType: sc ? sc.nodeType : null,
        startContainerLen: sc && sc.nodeType === 3 ? sc.textContent.length : null,
        neoOCot: cellCuaNeo?.getAttribute('data-col') ?? null,
        activeCol: active?.getAttribute?.('data-col') ?? null,
        // Hình dạng DOM của chính ô — Task 1.3
        soPhanTuCon: cellCuaNeo ? cellCuaNeo.childNodes.length : null,
        loaiPhanTuCon: cellCuaNeo ? [...cellCuaNeo.childNodes].map((c) => c.nodeType) : null,
        textContent: cellCuaNeo ? JSON.stringify(cellCuaNeo.textContent) : null,
      }
    } catch (err) {
      return { buoc: n, loiChup: String(err) }
    }
  }, nhan)
}

/** Đặt caret vào offset 0 của ô bản dịch thứ `i`, đi qua đường chuột SẢN PHẨM rồi mới thu hẹp. */
async function caretDauO(i) {
  const o = await $$('[data-col="tgt"]')[i]
  await realClick(o)
  await browser.pause(200)
  return await browser.execute((idx) => {
    try {
      const cell = document.querySelectorAll('[data-col="tgt"]')[idx]
      cell.focus()
      const sel = window.getSelection()
      sel.removeAllRanges()
      const r = document.createRange()
      // 🔴 Neo vào **text node đầu** nếu có, vào **chính ô** nếu ô rỗng — đúng hai hình
      // dạng mà Task 1.3 phải phủ. Ô bản dịch render MỘT mustache nên không có `#comment`
      // chen giữa như cột nguyên văn (`GridPanel.vue:1303`).
      const dau = cell.firstChild
      if (dau && dau.nodeType === 3) r.setStart(dau, 0)
      else r.setStart(cell, 0)
      r.collapse(true)
      sel.addRange(r)
      return { dat: true, coTextNode: !!(dau && dau.nodeType === 3) }
    } catch (err) {
      return { dat: false, loi: String(err) }
    }
  }, i)
}

/** Đọc `textContent` của mọi ô bản dịch — thước cho *"engine có xoá một ký tự không"*. */
async function chuTrongOto() {
  return await browser.execute(() =>
    [...document.querySelectorAll('[data-col="tgt"]')].map((c) => c.textContent),
  )
}

/** Đọc số hàng đang hiện — thước cho *"có lượt gộp nào chạy không"*. */
async function soHang() {
  return await browser.execute(() => document.querySelectorAll('[data-col="tgt"]').length)
}

async function doc(nhan) {
  const r = await browser.execute(() => window.__b29 ?? null)
  console.log(`\n[2.9 · ${nhan}] ` + JSON.stringify(r, null, 2))
  return r
}

describe('Bàn đo 2.9 — Backspace ở đầu ô bản dịch', () => {
  it('tám bước trên WKWebView thật', async () => {
    // Ba câu ⇒ có câu ĐẦU Chương (ca từ chối) và hai câu sau (ca gộp).
    await openWorkspaceWithWork('Bàn đo 2.9 — Backspace đầu ô', '一二三。四五六。七八九。')
    await browser.execute(() => {
      window.location.reload()
    })
    await $('[data-col="tgt"]').waitForExist({ timeout: 30_000 })

    // ── Tự kiểm danh tính phiên — luật sau 1.22 ────────────────────────────────────
    const danhTinh = await browser.execute(() => ({
      href: window.location.href,
      coApp: !!document.querySelector('#app'),
      userAgent: navigator.userAgent,
      soHang: document.querySelectorAll('[data-col="tgt"]').length,
    }))
    console.log('\n[2.9 · danh tính phiên] ' + JSON.stringify(danhTinh, null, 2))

    // ── Ⓐ Kiểm kê một ô bản dịch (Task 1.3, ca ô RỖNG) ────────────────────────────
    const kiemKe = await browser.execute(() => {
      const cell = document.querySelector('[data-col="tgt"]')
      return {
        contenteditable: cell.getAttribute('contenteditable'),
        tabindex: cell.getAttribute('tabindex'),
        soPhanTuCon: cell.childNodes.length,
        loaiPhanTuCon: [...cell.childNodes].map((c) => c.nodeType),
        textContent: JSON.stringify(cell.textContent),
        whiteSpace: getComputedStyle(cell).whiteSpace,
      }
    })
    console.log('\n[2.9 · Ⓐ kiểm kê ô bản dịch (RỖNG, chưa dịch)] ' + JSON.stringify(kiemKe, null, 2))

    // ── Ⓑ Ô RỖNG · phím VẬT LÝ · KHÔNG chặn ───────────────────────────────────────
    // Ca thường nhất của tính năng: xoá lui hết chữ rồi bấm thêm một lần.
    await camListener(false)
    console.log('\n[2.9 · Ⓑ caret] ' + JSON.stringify(await caretDauO(1), null, 2))
    console.log('\n[2.9 · Ⓑ trước] ' + JSON.stringify(await chupCaret('ô rỗng, offset 0'), null, 2))
    const truocB = await chuTrongOto()
    await browser.keys([BACKSPACE])
    await browser.pause(400)
    await doc('Ⓑ ô RỖNG · vật lý · không chặn')
    console.log('\n[2.9 · Ⓑ chữ trước/sau] ' + JSON.stringify({ truoc: truocB, sau: await chuTrongOto() }))
    console.log('\n[2.9 · Ⓑ sau] ' + JSON.stringify(await chupCaret('sau Backspace'), null, 2))
    await goListener()

    // ── Ⓒ Ô CÓ CHỮ · phím VẬT LÝ · KHÔNG chặn ─────────────────────────────────────
    // Gõ chữ vào ô 1 rồi đưa caret về offset 0. Đây là ca AC1 thật.
    await browser.execute(() => {
      const cell = document.querySelectorAll('[data-col="tgt"]')[1]
      cell.focus()
      const sel = window.getSelection()
      sel.removeAllRanges()
      const r = document.createRange()
      r.selectNodeContents(cell)
      sel.addRange(r)
      document.execCommand('insertText', false, 'bốn năm sáu')
    })
    await browser.pause(400)
    await camListener(false)
    console.log('\n[2.9 · Ⓒ caret] ' + JSON.stringify(await caretDauO(1), null, 2))
    console.log('\n[2.9 · Ⓒ trước] ' + JSON.stringify(await chupCaret('ô có chữ, offset 0'), null, 2))
    const truocC = await chuTrongOto()
    await browser.keys([BACKSPACE])
    await browser.pause(400)
    await doc('Ⓒ ô CÓ CHỮ · vật lý · không chặn')
    console.log('\n[2.9 · Ⓒ chữ trước/sau] ' + JSON.stringify({ truoc: truocC, sau: await chuTrongOto() }))
    await goListener()

    // ── Ⓓ Ô CÓ CHỮ · phím VẬT LÝ · CÓ preventDefault ──────────────────────────────
    // 🔴 Đây là mệnh đề Task 1.2 hỏi, và là mệnh đề mà một sự kiện TỔNG HỢP không trả lời
    // được: `preventDefault()` có chặn nổi lượt xoá của engine ở offset 0 không.
    await browser.execute(() => {
      const cell = document.querySelectorAll('[data-col="tgt"]')[1]
      cell.focus()
      const sel = window.getSelection()
      sel.removeAllRanges()
      const r = document.createRange()
      r.selectNodeContents(cell)
      sel.addRange(r)
      document.execCommand('insertText', false, 'bốn năm sáu')
    })
    await browser.pause(400)
    await camListener(true)
    await caretDauO(1)
    const truocD = await chuTrongOto()
    await browser.keys([BACKSPACE])
    await browser.pause(400)
    await doc('Ⓓ ô CÓ CHỮ · vật lý · CÓ preventDefault')
    console.log('\n[2.9 · Ⓓ chữ trước/sau] ' + JSON.stringify({ truoc: truocD, sau: await chuTrongOto() }))
    await goListener()

    // ── Ⓔ Caret ở GIỮA ô · phím VẬT LÝ · không chặn (đối chứng ÂM của cạm bẫy ④) ───
    // Phải xoá bình thường và KHÔNG được coi là "đầu ô".
    await camListener(false)
    await browser.execute(() => {
      const cell = document.querySelectorAll('[data-col="tgt"]')[1]
      cell.focus()
      const sel = window.getSelection()
      sel.removeAllRanges()
      const r = document.createRange()
      const dau = cell.firstChild
      if (dau && dau.nodeType === 3) r.setStart(dau, Math.min(3, dau.textContent.length))
      else r.setStart(cell, 0)
      r.collapse(true)
      sel.addRange(r)
    })
    console.log('\n[2.9 · Ⓔ trước] ' + JSON.stringify(await chupCaret('caret GIỮA ô'), null, 2))
    const truocE = await chuTrongOto()
    await browser.keys([BACKSPACE])
    await browser.pause(400)
    await doc('Ⓔ caret GIỮA ô · vật lý')
    console.log('\n[2.9 · Ⓔ chữ trước/sau] ' + JSON.stringify({ truoc: truocE, sau: await chuTrongOto() }))
    await goListener()

    // ── Ⓕ Ô có `\n` (Story 2.5d) · caret đầu DÒNG THỨ HAI ─────────────────────────
    // 🔴 Cạm bẫy ④: `startOffset === 0` ở đây KHÔNG có nghĩa "đầu ô". Bước này đo hình
    // dạng DOM mà `white-space: pre-line` để lại — một text node `\n` hay một `<br>`.
    const hinhDangXuongDong = await browser.execute(() => {
      try {
        const cell = document.querySelectorAll('[data-col="tgt"]')[1]
        cell.focus()
        const sel = window.getSelection()
        sel.removeAllRanges()
        const r = document.createRange()
        r.selectNodeContents(cell)
        sel.addRange(r)
        document.execCommand('insertText', false, 'AAA')
        document.execCommand('insertLineBreak')
        document.execCommand('insertText', false, 'BBB')
        return {
          soPhanTuCon: cell.childNodes.length,
          loaiPhanTuCon: [...cell.childNodes].map((c) => c.nodeType),
          tenThe: [...cell.childNodes].map((c) => c.nodeName),
          textContent: JSON.stringify(cell.textContent),
        }
      } catch (err) {
        return { loi: String(err) }
      }
    })
    await browser.pause(300)
    console.log('\n[2.9 · Ⓕ hình dạng sau insertLineBreak] ' + JSON.stringify(hinhDangXuongDong, null, 2))

    await camListener(false)
    const caretDongHai = await browser.execute(() => {
      try {
        const cell = document.querySelectorAll('[data-col="tgt"]')[1]
        const txt = cell.textContent
        const viTri = txt.indexOf('\n')
        if (viTri < 0) return { dat: false, lyDo: 'khong tim thay \\n trong textContent' }
        // Đi tìm đúng node + offset ứng với ký tự NGAY SAU `\n`.
        let conLai = viTri + 1
        for (const con of cell.childNodes) {
          const len = con.nodeType === 3 ? con.textContent.length : 1
          if (conLai <= len) {
            const sel = window.getSelection()
            sel.removeAllRanges()
            const r = document.createRange()
            if (con.nodeType === 3) r.setStart(con, conLai)
            else r.setStartAfter(con)
            r.collapse(true)
            sel.addRange(r)
            return { dat: true, node: con.nodeName, offset: conLai }
          }
          conLai -= len
        }
        return { dat: false, lyDo: 'khong lan duoc toi vi tri' }
      } catch (err) {
        return { dat: false, loi: String(err) }
      }
    })
    console.log('\n[2.9 · Ⓕ đặt caret đầu dòng 2] ' + JSON.stringify(caretDongHai, null, 2))
    console.log('\n[2.9 · Ⓕ caret] ' + JSON.stringify(await chupCaret('đầu DÒNG THỨ HAI'), null, 2))
    await goListener()

    // ── Ⓖ event.repeat — Ice ký ③ "gộp một lần rồi dừng", chốt bằng chính cờ này ───
    // ⚠️ GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện: WebDriver `keyDown`
    // KHÔNG sinh auto-repeat của hệ điều hành. Bước này đo được `repeat` có MẶT và bằng
    // `false` ở một lượt bấm rời rạc — tức chốt không phá ca thường. Vế "giữ phím thật"
    // là một món cho Ice, cùng lớp với "gõ tiếng Việt bằng bộ gõ".
    await camListener(false)
    await caretDauO(2)
    await browser.action('key').down(BACKSPACE).pause(600).up(BACKSPACE).perform()
    await browser.pause(400)
    await doc('Ⓖ giữ phím 600ms qua Actions API')
    await goListener()

    // ── Ⓗ Sự kiện TỔNG HỢP — đo chính chỗ nó lệch khỏi phím vật lý ────────────────
    // Spec sản phẩm buộc phải dùng đường này (giới hạn ③ của 2.8). Bước này nói nó đo
    // được cái gì và KHÔNG đo được cái gì.
    await camListener(false)
    await caretDauO(2)
    const truocH = await chuTrongOto()
    await browser.execute(() => {
      const cell = document.querySelectorAll('[data-col="tgt"]')[2]
      cell.dispatchEvent(
        new KeyboardEvent('keydown', {
          key: 'Backspace',
          code: 'Backspace',
          bubbles: true,
          cancelable: true,
        }),
      )
    })
    await browser.pause(400)
    await doc('Ⓗ sự kiện TỔNG HỢP')
    console.log('\n[2.9 · Ⓗ chữ trước/sau] ' + JSON.stringify({ truoc: truocH, sau: await chuTrongOto() }))
    await goListener()

    console.log('\n[2.9 · số hàng cuối lượt] ' + (await soHang()))
  })
})
