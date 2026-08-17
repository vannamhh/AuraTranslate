/**
 * Bàn đo Story 2.8 · Task 1 — caret trong **cột NGUYÊN VĂN**, trên WKWebView THẬT.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 CÂU HỎI BÀN ĐO NÀY TRẢ LỜI, VÀ VÌ SAO NÓ CHẶN MỘT TASK
 * ═════════════════════════════════════════════════════════════════════════════════
 * Ice ký Quyết định #2 **đường (a)**: *"lấy chỗ cắt từ vùng chọn đang có ở cột nguyên
 * văn"*. Chữ ký đó ký **TRƯỚC** phép đo, đúng khuôn Quyết định #6(a) của Story 2.5b — và
 * story ghi thẳng rằng **luật dừng thắng chữ ký nếu phép đo trượt**.
 *
 * Có một số đo CŨ đứng ngay chỗ này, và nó nghiêng về phía **bác** đường (a):
 * `2-5b-ban-do/chup.mjs:11-14` ghi *"Playwright-WebKit **có** tạo vùng chọn ở lượt bấm vào
 * văn bản chỉ-đọc, WKWebView **không**"*, và trỏ về `EditorPanel.vue:474-479`.
 *
 * ⚠️ **Không đọc số cũ đó thành kết luận của hôm nay**, vì hai tiền đề của nó đã đổi:
 * ① `EditorPanel.vue` **không còn tồn tại** (xoá ở Story 2.5b, commit `ca33072`); ② hình
 * dạng DOM đã đổi hẳn — từ *"N `<span>` trong một dòng văn liền tục"* sang **lưới
 * `subgrid`, mỗi ô một `<div>` khối có `min-height`**. Một ô có hộp thật là **đúng thứ**
 * đã lật ca *"ô rỗng không neo được caret"* ở 2.5b. ⇒ Đo lại, đừng suy.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔵 VÒNG 2 — VÌ **THƯỚC** CỦA VÒNG 1 HỎNG, KHÔNG VÌ SỐ CỦA NÓ SAI
 * ═════════════════════════════════════════════════════════════════════════════════
 * Vòng 1 (2026-08-17, cùng tệp này) cho một bảng **tự mâu thuẫn**: phép đo *"bấm vào ô
 * nguyên văn"* báo neo nằm trong cột **bản dịch**, còn phép **đối chứng** *"bấm vào ô bản
 * dịch"* lại báo neo trong cột **nguyên văn** — hai số hoán vị nhau. Hai khuyết tật của
 * BÀN ĐO đứng sau, cả hai đã sửa ở vòng này:
 *
 * ① **Không bước nào tự khai nó vừa bấm vào cái gì.** Một lượt bấm không đổi vùng chọn để
 *    lại nguyên vùng chọn của bước TRƯỚC, và bảng đọc ra y hệt một lượt bấm thành công vào
 *    chỗ khác. ⇒ Vòng này **dọn vùng chọn về rỗng trước mỗi bước** và **chụp lại đích ngay
 *    trước khi bấm** (`data-col` + `data-segment-id` + hộp). Không dọn thì *"engine không
 *    làm gì"* và *"engine làm đúng"* cho **cùng một** bảng số — đúng lớp *"rỗng im lặng"*.
 * ② **`TreeWalker` lấy text node ĐẦU TIÊN, và nó rỗng.** Ô nguyên văn mang hai `#comment`
 *    (`aura-allow-text`) nên `childNodes` là `[#comment, TEXT, #comment, TEXT, TEXT]` —
 *    text node đầu là khoảng trắng của template, `data.length = 0`. `setPosition(node, 0)`
 *    trên nó là một phép đo về **hư không**. ⇒ Vòng này chọn text node **DÀI NHẤT**.
 *
 * ⚠️ Đây là **hai lượt sửa thước, không hai vòng chẩn đoán bị bác** — LUẬT DỪNG (Task 1.4)
 * đếm những vòng mà một **giả thuyết về sản phẩm** bị phép đo bác. Phân biệt này đã được
 * ghi một lần ở 2.5d §Debug Log Ⓐ (*"số thật, trật câu hỏi"*), và đây là cùng loại.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO TỆP NÀY KHÔNG NẰM TRONG `e2e/specs/`
 * ═════════════════════════════════════════════════════════════════════════════════
 * Nó hỏi **engine**, không kiểm **sản phẩm**: mọi mệnh đề ở đây là *"engine làm gì"*, và
 * hôm nay chưa có bề mặt sản phẩm nào để kiểm. Một tệp như vậy nằm trong bộ thường trực là
 * dựng một ca xanh mãi mãi mà không canh mệnh đề nào — đúng thứ §Testing Rules gọi là
 * *"nguồn sự thật thứ hai"*. Cùng khuôn `2-2-ban-do/` · `2-4-ban-do/` · `2-5-ban-do/` ·
 * `2-5b-ban-do/` · `2-5d-ban-do/`; không có gì ở đây vào `package.json`.
 *
 *     TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-8-ban-do/tach-cot-nguon-wkwebview.e2e.mjs
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * ⚠️ GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện
 * ═════════════════════════════════════════════════════════════════════════════════
 * ① **Một engine, không hai.** Task 1.1 hỏi *"trên cả hai engine"*, và bàn đo này chỉ chạy
 *    WKWebView. Lý do là chính câu `chup.mjs:11-14` đã ghi: ở **đúng** mệnh đề này —
 *    vùng chọn trong văn bản chỉ-đọc — Playwright-WebKit và WKWebView **đã bất đồng một
 *    lần**, nên một số Blink/Playwright ở đây không **cộng** thêm gì; nó chỉ cho một con số
 *    thứ hai dễ đọc nhầm thành xác nhận. Nửa Blink có chủ: nó chỉ có nghĩa cho vế **hình
 *    học**, và story này không hỏi hình học.
 * ② `browser.keys()` chỉ bắn `keydown`/`keyup`, **không** đi vào đường nhập văn bản gốc của
 *    WKWebView *(đo 2026-08-12, giới hạn của bộ đo)*. Mọi mệnh đề ở đây đi qua **chuột
 *    thật** hoặc `Range`/`Selection` — không mệnh đề nào dựa vào phím.
 * ③ Bàn đo **không sửa một dòng mã sản phẩm** và **không** vô hiệu hoá lớp nào. Khác
 *    `2-5d-ban-do/`: ở đó phải cắt hai lớp chặn `Enter` để nhìn engine; ở đây cột nguyên
 *    văn **không có** lớp chặn nào, nên hỏi thẳng.
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { realClick } from '../../../e2e/support/pointer.mjs'

/**
 * Bọc cầu IPC để **đếm** lượt tra từ điển — mệnh đề ④.
 *
 * 🔴 Đếm ở **cầu IPC**, không ở DOM của Panel Lookup: một lượt tra có thể bay đi rồi trả
 * **0 hàng**, và panel khi đó không đổi một pixel. Đo ở DOM là đo *"kết quả có hiện
 * không"*, còn câu hỏi của Task 1.3 là *"có một lượt tra được PHÁT ra không"* — hai mệnh
 * đề khác nhau, và cái sai là cái im lặng.
 *
 * Bọc, **không** thay: hàm gốc vẫn được gọi, nên sản phẩm chạy y nguyên trong lúc đo.
 */
async function catIpc() {
  await browser.execute(() => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined || internals.__aura28Goc !== undefined) return
    internals.__aura28Goc = internals.invoke
    window.__aura28 = { loiGoi: [] }
    internals.invoke = function (cmd, ...rest) {
      window.__aura28.loiGoi.push(String(cmd))
      return internals.__aura28Goc.call(this, cmd, ...rest)
    }
  })
}

/**
 * 🔴 Dọn vùng chọn về **rỗng thật** và khẳng định nó rỗng — khuyết tật ① của vòng 1.
 *
 * Trả `selectionType` **sau** lượt dọn. Một giá trị khác `"None"` ở đây nghĩa là chính
 * phép dọn không ăn, và mọi số của bước kế tiếp phải đọc là **không kết luận được**.
 */
async function donVungChon() {
  return await browser.execute(() => {
    const sel = window.getSelection()
    sel?.removeAllRanges()
    if (document.activeElement instanceof HTMLElement) document.activeElement.blur()
    return { sauKhiDon: window.getSelection()?.type ?? null }
  })
}

/** Chụp trạng thái vùng chọn + tiêu điểm, cùng một bộ trường cho mọi mệnh đề. */
async function chupVungChon() {
  return await browser.execute(() => {
    const sel = window.getSelection()
    const active = document.activeElement
    const anchorEl =
      sel?.anchorNode instanceof Element ? sel.anchorNode : (sel?.anchorNode?.parentElement ?? null)
    const anchorCell = anchorEl?.closest('[data-col]') ?? null
    return {
      selectionType: sel ? sel.type : null,
      rangeCount: sel ? sel.rangeCount : 0,
      anchorLaTextNode: sel?.anchorNode ? sel.anchorNode.nodeType === 3 : null,
      anchorOffset: sel?.anchorOffset ?? null,
      focusOffset: sel?.focusOffset ?? null,
      chuDaChon: sel ? sel.toString() : null,
      // 🔴 Đọc THẲNG thuộc tính của ô mang neo, không hai cờ boolean rời — vòng 1 cho hai
      // cờ đó hoán vị nhau và không cách nào biết cái nào đúng.
      neoOCot: anchorCell?.getAttribute('data-col') ?? null,
      neoOSegment: anchorCell?.getAttribute('data-segment-id') ?? null,
      activeCol: active?.getAttribute?.('data-col') ?? null,
      activeSegment: active?.getAttribute?.('data-segment-id') ?? null,
      activeClass: active?.getAttribute?.('class') ?? null,
    }
  })
}

/** Số lượt `lookup_dictionary` đã bay kể từ lượt gọi trước, rồi dọn sổ. */
async function demLuotTra() {
  return await browser.execute(() => {
    const all = window.__aura28?.loiGoi ?? []
    const n = all.filter((c) => c === 'lookup_dictionary').length
    window.__aura28.loiGoi = []
    return n
  })
}

/** Chụp đích **ngay trước** lượt bấm — khuyết tật ① của vòng 1. */
async function chupDich(col) {
  return await browser.execute((c) => {
    const cell = document.querySelector(`[data-col="${c}"]`)
    if (cell === null) return null
    const r = cell.getBoundingClientRect()
    return {
      dataCol: cell.getAttribute('data-col'),
      dataSegmentId: cell.getAttribute('data-segment-id'),
      class: cell.getAttribute('class'),
      textContent: cell.textContent,
      hop: {
        x: Math.round(r.x),
        y: Math.round(r.y),
        w: Math.round(r.width),
        h: Math.round(r.height),
      },
      // Cái gì thật sự nằm dưới tâm ô — bắt trọn ca "một lớp phủ nuốt cú bấm".
      duoiTamO: (() => {
        const el = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2)
        return el === null
          ? null
          : {
              tag: el.tagName,
              col: el.getAttribute?.('data-col') ?? null,
              class: el.getAttribute?.('class') ?? null,
              laChinhO: el === cell || cell.contains(el),
            }
      })(),
    }
  }, col)
}

describe('Bàn đo 2.8 — caret trong cột NGUYÊN VĂN trên WKWebView thật', () => {
  it('bốn mệnh đề của Task 1', async () => {
    await openWorkspaceWithWork('Bàn đo 2.8 — tách ở cột nguồn')

    // Cùng lượt nạp lại của `grid-empty-cell.e2e.mjs`, cùng lý do (state module-level rò
    // qua các phiên app vì `$APPDATA` dùng chung). Vá của BÀN ĐO, không của sản phẩm.
    await browser.execute(() => {
      window.location.reload()
    })
    await $('[data-col="src"]').waitForExist({
      timeout: 30_000,
      timeoutMsg:
        'Nạp lại webview rồi mà không thấy một ô `[data-col="src"]` nào sau 30 giây — ' +
        'lưới không dựng được, hoặc Tác phẩm đang mở không có segment nào.',
    })

    // 🔴 Tự kiểm danh tính phiên TRƯỚC mọi số đo — khuôn 2.5d. Một bàn đo đọc nhầm cửa sổ
    // cho ra một bảng số thật về một ứng dụng khác.
    const danhTinh = await browser.execute(() => ({
      href: window.location.href,
      coApp: !!document.querySelector('#app'),
      userAgent: navigator.userAgent,
    }))
    console.log('\n[2.8 · danh tính phiên] ' + JSON.stringify(danhTinh, null, 2))

    await catIpc()

    // ── ⓪ Kiểm kê DOM của một ô nguyên văn ─────────────────────────────────────────
    //
    // 🔴 Hỏi TRƯỚC, vì mọi mệnh đề sau đọc `anchorOffset` — và một `anchorOffset` chỉ dùng
    // được nếu ta biết nó đếm trong CÁI GÌ. Vòng 1 trượt đúng ở đây: nó lấy text node đầu
    // tiên mà không hỏi node đó dài bao nhiêu.
    const kiemKe = await browser.execute(() => {
      const cell = document.querySelector('[data-col="src"]')
      if (cell === null) return null
      const con = [...cell.childNodes].map((n, i) => ({
        i,
        kieu: n.nodeType === 3 ? 'TEXT' : n.nodeType === 8 ? 'COMMENT' : n.nodeName,
        doDai: n.nodeType === 3 ? n.data.length : null,
        xemTruoc: n.nodeType === 3 ? JSON.stringify(n.data.slice(0, 24)) : null,
      }))
      return {
        idSegment: cell.getAttribute('data-segment-id'),
        soPhanTuCon: cell.querySelectorAll('*').length,
        contentEditable: cell.getAttribute('contenteditable'),
        tabIndex: cell.getAttribute('tabindex'),
        caoPx: +cell.getBoundingClientRect().height.toFixed(2),
        textContent: cell.textContent,
        doDaiTextContent: cell.textContent?.length ?? null,
        con,
        // 🔴 Câu hỏi thật của Quyết định #2(a): một `anchorOffset` trong text node DÀI NHẤT
        // có bằng chỉ số ký tự trong `source_text` không? Bằng ⇔ mọi text node đứng trước
        // nó cộng lại dài 0.
        chiSoTextNodeDaiNhat: (() => {
          const texts = [...cell.childNodes].filter((n) => n.nodeType === 3)
          let best = -1
          let bestLen = -1
          texts.forEach((n, i) => {
            if (n.data.length > bestLen) {
              bestLen = n.data.length
              best = i
            }
          })
          return { chiSo: best, doDai: bestLen, soTextNode: texts.length }
        })(),
        tongDoDaiTruocTextNodeDaiNhat: (() => {
          const texts = [...cell.childNodes].filter((n) => n.nodeType === 3)
          let bestLen = -1
          let best = -1
          texts.forEach((n, i) => {
            if (n.data.length > bestLen) {
              bestLen = n.data.length
              best = i
            }
          })
          return texts.slice(0, best).reduce((s, n) => s + n.data.length, 0)
        })(),
      }
    })
    console.log('\n[2.8 · ⓪ kiểm kê ô nguyên văn] ' + JSON.stringify(kiemKe, null, 2))

    // ── ① ĐỐI CHỨNG TRƯỚC — một cú bấm vào ô BẢN DỊCH ──────────────────────────────
    //
    // 🔴 Đối chứng chạy **đầu tiên**, không cuối cùng: ở vòng 1 nó chạy sau một lượt
    // `setPosition` bằng script và vì thế đọc lại đúng vùng chọn của bước đó. Ô bản dịch
    // **đã biết** là đặt được caret (`grid-empty-cell.e2e.mjs` xanh) ⇒ nó là thước đo
    // chính bàn đo này, và một thước phải được đọc trước khi bị chạm vào.
    const donTruocDich = await donVungChon()
    const dichTruocBam = await chupDich('tgt')
    await realClick(await $('[data-col="tgt"]'))
    await browser.pause(250)
    const sauBamDich = await chupVungChon()
    console.log(
      '\n[2.8 · ① ĐỐI CHỨNG — bấm ô BẢN DỊCH] ' +
        JSON.stringify(
          { don: donTruocDich, dich: dichTruocBam, sauBam: sauBamDich },
          null,
          2,
        ),
    )

    // ── ② MỘT CÚ BẤM vào ô NGUYÊN VĂN ──────────────────────────────────────────────
    //
    // 🔴 Mệnh đề trung tâm. Đường (a) của Quyết định #2 cần một **offset**, và một cú bấm
    // đơn là cử chỉ rẻ nhất cho nó. Vùng chọn dọn về rỗng ngay trước, nên một `"None"` ở
    // đây là *"engine KHÔNG đặt caret"*, không phải *"số cũ còn sót"*.
    await demLuotTra()
    const donTruocNguon = await donVungChon()
    const nguonTruocBam = await chupDich('src')
    await realClick(await $('[data-col="src"]'))
    await browser.pause(250)
    const sauBamNguon = await chupVungChon()
    const traSauBamNguon = await demLuotTra()
    console.log(
      '\n[2.8 · ② một cú bấm vào ô NGUYÊN VĂN] ' +
        JSON.stringify(
          {
            don: donTruocNguon,
            dich: nguonTruocBam,
            sauBam: sauBamNguon,
            luotTraDaBay: traSauBamNguon,
          },
          null,
          2,
        ),
    )

    // ── ③ KÉO CHỌN THẬT trong ô nguyên văn + Auto-Lookup ───────────────────────────
    //
    // Task 1.3: vùng chọn cột nguồn hôm nay **đang phục vụ Auto-Lookup** (vai `'source'`,
    // `GridPanel.vue:309`). Một lượt bôi đen **để tách** có bắn một lượt tra không?
    await demLuotTra()
    const donTruocKeo = await donVungChon()
    const hopKeo = nguonTruocBam.hop
    await browser
      .action('pointer')
      .move({ x: hopKeo.x + Math.round(hopKeo.w * 0.2), y: hopKeo.y + Math.round(hopKeo.h / 2) })
      .down()
      .pause(30)
      .move({ x: hopKeo.x + Math.round(hopKeo.w * 0.5), y: hopKeo.y + Math.round(hopKeo.h / 2) })
      .pause(30)
      .up()
      .pause(150)
      .perform()
    // Đợi rộng tay: một lượt tra đi qua IPC, và câu hỏi là *"có bay không"*, không phải
    // *"bay nhanh không"* — một ngưỡng chặt ở đây đo bàn đo chứ không đo sản phẩm.
    await browser.pause(900)
    const sauKeo = await chupVungChon()
    const traSauKeo = await demLuotTra()
    console.log(
      '\n[2.8 · ③ kéo chọn trong ô NGUYÊN VĂN] ' +
        JSON.stringify(
          { don: donTruocKeo, sauKeo, luotTraDaBay: traSauKeo },
          null,
          2,
        ),
    )

    // ── ④ `setPosition` bằng script vào text node DÀI NHẤT của ô nguyên văn ────────
    //
    // Đường dự phòng: nếu ② trượt, sản phẩm vẫn đặt được caret bằng tay ở `mouseup` — đúng
    // khuôn đường chuột mà 2.5b phải dựng cho ô BẢN DỊCH. Mệnh đề: engine có **giữ** một
    // vùng chọn đặt bằng script trong vùng chỉ-đọc không, và `anchorOffset` đọc lại có
    // đúng thứ vừa đặt không.
    const donTruocSet = await donVungChon()
    const sauSetPosition = await browser.execute(() => {
      const cell = document.querySelector('[data-col="src"]')
      const texts = [...cell.childNodes].filter((n) => n.nodeType === 3)
      if (texts.length === 0) return { datDuoc: false, lyDo: 'ô không có text node nào' }
      // 🔵 Text node DÀI NHẤT, không node đầu tiên — khuyết tật ② của vòng 1.
      const text = texts.reduce((a, b) => (b.data.length > a.data.length ? b : a))
      if (text.data.length === 0) return { datDuoc: false, lyDo: 'mọi text node đều rỗng' }
      const sel = window.getSelection()
      const viTri = Math.min(3, text.data.length)
      sel.setPosition(text, viTri)
      const doc = sel.anchorNode
      return {
        datDuoc: true,
        doDaiTextNode: text.data.length,
        viTriYeuCau: viTri,
        selectionType: sel.type,
        anchorOffset: sel.anchorOffset,
        anchorLaChinhTextNode: doc === text,
        // Ánh xạ offset → chỉ số trong `source_text`: chỉ bằng nhau nếu mọi text node
        // đứng trước cộng lại dài 0.
        chuTruocCaret: text.data.slice(0, viTri),
      }
    })
    console.log(
      '\n[2.8 · ④ setPosition vào text node dài nhất] ' +
        JSON.stringify({ don: donTruocSet, ...sauSetPosition }, null, 2),
    )

    // Khẳng định DUY NHẤT của bàn đo, và nó canh **BÀN ĐO**, không canh sản phẩm: nếu đối
    // chứng ① không có caret thì thước hỏng và mọi số ở trên phải đọc là không kết luận
    // được — chứ không đọc thành một mệnh đề về cột nguyên văn.
    await expect(sauBamDich.selectionType).toBe('Caret')
    await expect(sauBamDich.activeCol).toBe('tgt')
  })
})
