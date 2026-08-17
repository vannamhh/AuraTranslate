/**
 * Bàn đo Story 2.8 · Task 1 — **VÒNG 3**, và nó chỉ hỏi một câu: *"số của vòng 2 là mệnh
 * đề về ENGINE, hay là tạo tác của BÀN ĐO?"*
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO VÒNG NÀY PHẢI CHẠY TRƯỚC KHI GHI MỘT KẾT LUẬN
 * ═════════════════════════════════════════════════════════════════════════════════
 * Vòng 2 đo được, trên WKWebView 605.1.15, cửa sổ Tauri thật:
 *
 * | Cử chỉ trên ô `[data-col="src"]` | `selectionType` | `rangeCount` |
 * |---|---|---|
 * | một cú bấm | **`"None"`** | **0** |
 * | kéo chọn 20 % → 50 % chiều rộng | **`"None"`** | **0** |
 * | đối chứng: bấm ô `[data-col="tgt"]` | `"Caret"` | 1 |
 *
 * Đọc thẳng, bảng đó nói *"cột nguyên văn không bôi đen được trên engine thật"* — và mệnh
 * đề ấy **to hơn Story 2.8**: nó giết luôn **Auto-Lookup** (FR21, Story 1.18, đã phát
 * hành), thứ dựa **toàn bộ** vào một vùng chọn ở cột nguồn (`GridPanel.vue:309`,
 * `useSelectionSurface(colSrc, 'source', …)`).
 *
 * ⇒ Một mệnh đề hạng đó **không được ghi từ một lượt đo có hai ứng viên chưa loại trừ**,
 * và bàn đo vòng 2 có đúng hai:
 *
 *   Ⓐ **`blur()` của chính bàn đo.** `donVungChon()` gọi `document.activeElement.blur()`
 *      trước mỗi bước. WebKit **từ chối** dựng vùng chọn khi tài liệu không có tiêu điểm.
 *      Đối chứng ① sống sót được vì ô bản dịch là `contenteditable` — nó tự giành tiêu
 *      điểm ở `mousedown`, còn ô nguyên văn thì **không có gì để giành**.
 *   Ⓑ **Lượt kéo quá thô.** Hai điểm và 30 ms giữa chúng; WebKit có thể cần nhiều bước di
 *      chuyển trung gian mới khởi động một lượt chọn văn bản.
 *
 * Vòng này chạy **bốn** biến thể trên **cùng một ô**, khác nhau đúng một biến mỗi lượt, và
 * ghi `document.hasFocus()` ở mọi bước — thứ vòng 2 không hỏi.
 *
 * ⚠️ **Đây là vòng chẩn đoán THỨ HAI theo nghĩa của LUẬT DỪNG** (Task 1.4 — *"ba vòng chẩn
 * đoán bị phép đo bác ⇒ dừng"*). Vòng 1 không tính: nó không bác một giả thuyết nào về sản
 * phẩm, nó chỉ hỏng thước (hai số hoán vị nhau vì bàn đo không dọn vùng chọn giữa các
 * bước). Phân biệt này đã có tiền lệ ở `2-5d-ban-do/` §Debug Log Ⓐ.
 *
 *     TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-8-ban-do/tach-cot-nguon-vong3.e2e.mjs
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'

/** Chụp vùng chọn + tiêu điểm + **`document.hasFocus()`** — trường mới của vòng này. */
async function chup(nhan) {
  return await browser.execute((n) => {
    const sel = window.getSelection()
    const active = document.activeElement
    const anchorEl =
      sel?.anchorNode instanceof Element ? sel.anchorNode : (sel?.anchorNode?.parentElement ?? null)
    const cell = anchorEl?.closest('[data-col]') ?? null
    return {
      buoc: n,
      taiLieuCoTieuDiem: document.hasFocus(),
      selectionType: sel ? sel.type : null,
      rangeCount: sel ? sel.rangeCount : 0,
      chuDaChon: sel ? sel.toString() : null,
      anchorOffset: sel?.anchorOffset ?? null,
      focusOffset: sel?.focusOffset ?? null,
      neoOCot: cell?.getAttribute('data-col') ?? null,
      activeCol: active?.getAttribute?.('data-col') ?? null,
      activeTag: active?.tagName ?? null,
    }
  }, nhan)
}

/** Dọn vùng chọn **KHÔNG** `blur()` — biến Ⓐ được gỡ khỏi ba biến thể sau. */
async function donNhe() {
  await browser.execute(() => {
    window.getSelection()?.removeAllRanges()
  })
}

/** Hộp của ô nguyên văn đầu tiên. */
async function hopONguon() {
  return await browser.execute(() => {
    const cell = document.querySelector('[data-col="src"]')
    const r = cell.getBoundingClientRect()
    return {
      x: Math.round(r.x),
      y: Math.round(r.y),
      w: Math.round(r.width),
      h: Math.round(r.height),
      text: cell.textContent,
    }
  })
}

describe('Bàn đo 2.8 vòng 3 — hai ứng viên của bàn đo, loại từng cái một', () => {
  it('bốn biến thể trên cùng một ô nguyên văn', async () => {
    await openWorkspaceWithWork('Bàn đo 2.8 vòng 3')
    await browser.execute(() => {
      window.location.reload()
    })
    await $('[data-col="src"]').waitForExist({ timeout: 30_000 })

    const danhTinh = await browser.execute(() => ({
      href: window.location.href,
      coApp: !!document.querySelector('#app'),
      userAgent: navigator.userAgent,
    }))
    console.log('\n[2.8·v3 · danh tính phiên] ' + JSON.stringify(danhTinh, null, 2))

    const hop = await hopONguon()
    const giua = { x: hop.x + Math.round(hop.w / 2), y: hop.y + Math.round(hop.h / 2) }
    console.log('\n[2.8·v3 · hộp ô nguyên văn] ' + JSON.stringify(hop, null, 2))

    // ── Ⓐ0 — trạng thái NỀN, chưa ai chạm gì ────────────────────────────────────────
    console.log('\n[2.8·v3 · Ⓐ0 nền] ' + JSON.stringify(await chup('nền'), null, 2))

    // ── ① Bấm đơn, KHÔNG `blur()` trước ─────────────────────────────────────────────
    //
    // Khác vòng 2 đúng **một** biến: bỏ `blur()`. Một `"Caret"` ở đây ⇒ ứng viên Ⓐ là thủ
    // phạm, và số của vòng 2 là tạo tác của bàn đo.
    await donNhe()
    await browser
      .action('pointer')
      .move({ x: giua.x, y: giua.y })
      .down()
      .pause(40)
      .up()
      .pause(150)
      .perform()
    await browser.pause(300)
    console.log(
      '\n[2.8·v3 · ① bấm đơn, KHÔNG blur] ' + JSON.stringify(await chup('bấm đơn'), null, 2),
    )

    // ── ② Kéo NHIỀU BƯỚC, không `blur()` ────────────────────────────────────────────
    //
    // Gỡ ứng viên Ⓑ: sáu điểm trung gian thay vì một, mỗi điểm cách nhau 40 ms.
    await donNhe()
    const keo = browser
      .action('pointer')
      .move({ x: hop.x + Math.round(hop.w * 0.15), y: giua.y })
      .down()
      .pause(60)
    for (const p of [0.25, 0.35, 0.45, 0.55, 0.65, 0.75]) {
      keo.move({ x: hop.x + Math.round(hop.w * p), y: giua.y }).pause(40)
    }
    await keo.up().pause(200).perform()
    await browser.pause(600)
    console.log(
      '\n[2.8·v3 · ② kéo nhiều bước, KHÔNG blur] ' +
        JSON.stringify(await chup('kéo nhiều bước'), null, 2),
    )

    // ── ③ Bấm ô BẢN DỊCH trước (lấy tiêu điểm vào tài liệu), RỒI kéo ở cột nguồn ────
    //
    // Biến thể này tách hẳn *"tài liệu có tiêu điểm"* khỏi *"cột nguồn nhận được vùng
    // chọn"*: nếu ② vẫn `"None"` mà ③ ra `"Range"` thì điều kiện là **tiêu điểm tài
    // liệu**, không phải bản thân cột nguồn — và đó là một mệnh đề khác hẳn.
    const oDich = await $('[data-col="tgt"]')
    await browser.action('pointer').move({ origin: oDich }).down().pause(40).up().pause(150).perform()
    await browser.pause(250)
    console.log(
      '\n[2.8·v3 · ③a sau khi bấm ô bản dịch] ' +
        JSON.stringify(await chup('sau bấm ô dịch'), null, 2),
    )
    await donNhe()
    const keo2 = browser
      .action('pointer')
      .move({ x: hop.x + Math.round(hop.w * 0.15), y: giua.y })
      .down()
      .pause(60)
    for (const p of [0.3, 0.45, 0.6, 0.75]) {
      keo2.move({ x: hop.x + Math.round(hop.w * p), y: giua.y }).pause(40)
    }
    await keo2.up().pause(200).perform()
    await browser.pause(600)
    console.log(
      '\n[2.8·v3 · ③b kéo cột nguồn SAU khi tài liệu đã có tiêu điểm] ' +
        JSON.stringify(await chup('kéo sau khi có tiêu điểm'), null, 2),
    )

    // ── ④ `Selection.modify()` — đường mà `SourceHanViet.vue:317` đã đo một lần ─────
    //
    // Nếu ba biến thể trên đều `"None"`, câu hỏi cuối là: engine có giữ nổi một vùng chọn
    // **KHÔNG do chuột tạo ra** ở cột nguồn không. Vòng 2 đã trả lời `setPosition` thì có;
    // đây hỏi thêm `modify('extend')`, tức đường mở rộng thành một `Range` thật.
    const modify = await browser.execute(() => {
      const cell = document.querySelector('[data-col="src"]')
      const texts = [...cell.childNodes].filter((n) => n.nodeType === 3)
      const text = texts.reduce((a, b) => (b.data.length > a.data.length ? b : a))
      const sel = window.getSelection()
      sel.setPosition(text, 0)
      sel.modify('extend', 'forward', 'word')
      return {
        selectionType: sel.type,
        chuDaChon: sel.toString(),
        anchorOffset: sel.anchorOffset,
        focusOffset: sel.focusOffset,
        taiLieuCoTieuDiem: document.hasFocus(),
      }
    })
    console.log('\n[2.8·v3 · ④ Selection.modify] ' + JSON.stringify(modify, null, 2))
  })
})
