/**
 * Story 2.9 · AC1 · AC4 · AC6 — **cử chỉ `Backspace` ở đầu ô đi trọn đường dây, trong
 * WKWebView THẬT**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO CA NÀY PHẢI TỒN TẠI, TRONG KHI 2.8 ĐÃ CÓ MỘT CA GỘP
 * ═════════════════════════════════════════════════════════════════════════════════
 * Nghiệp vụ gộp dùng chung với `⌘M` và **đã có chủ** ở `segment_contract.rs` cộng
 * `segment-merge-split.e2e.mjs`. Thứ story này thêm là **cử chỉ**, và cử chỉ đi một đường
 * khác hẳn hợp âm:
 *
 * | | `⌘M` (Story 2.8) | `Backspace` (story này) |
 * |---|---|---|
 * | Vào đâu | `keys.ts` → bảng hợp âm → `CommandRegistry` | `GridPanel.vue::onEditKeydown` **trực tiếp** |
 * | Phép kiểm quyết định | hợp âm khớp | `caretAtCellStart(cell, selection)` |
 * | Cổng nào canh | `check:commands` Kiểm A/B | 🔴 **KHÔNG CỔNG NÀO** |
 *
 * ⇒ Hàng thứ ba là lý do tệp này tồn tại. `check:commands` Kiểm A **chỉ canh `@click`** —
 * `check-commands.mjs:2348-2349` in ra mỗi lượt chạy: *"ngày một `@keydown` mang thao tác
 * thật xuất hiện, luật phải được xem lại"*. Trước story này `onEditKeydown` **không mang
 * thao tác nào**; nay nó mang một thao tác **phá huỷ và không lui được**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * ⚠️ GIỚI HẠN THẬT CỦA BỘ ĐO — ghi ra thay vì để người sau tưởng đã phủ
 * ═════════════════════════════════════════════════════════════════════════════════
 * Mọi sự kiện driver giao đều mang **`isTrusted: false`** *(đo 2026-08-17,
 * `2-9-ban-do/README.md` §Vòng 1)*, và một sự kiện không tin cậy **không có default action**.
 * ⇒ Ba mệnh đề **KHÔNG** nghiệm thu được ở đây, và cả ba là món cho Ice:
 *   ① `preventDefault()` có chặn nổi lượt xoá của một phím **thật** không;
 *   ② auto-repeat của hệ điều hành *(chữ ký ③ — "gộp một lần rồi dừng")*;
 *   ③ một lượt chốt của **bộ gõ tiếng Việt** không bị ăn mất *(chốt `isComposing`)*.
 *
 * Thứ tệp này **CÓ** nghiệm thu được, và nó là phần quyết định: một `keydown` mang
 * `key: 'Backspace'` tới đúng ô, `caretAtCellStart` phân xử đúng trên `Selection` **do chính
 * engine dựng**, và cả chuỗi gộp chạy tới lưới.
 *
 * 🔴 **SỰ KIỆN TỔNG HỢP, KHÔNG `browser.keys()`** — cùng khuôn và cùng lý do với ca `⌘/` của
 * 2.8: driver giao sai `code` *(đo 2026-08-17: `browser.keys(['Meta','/'])` giao `code: "/"`
 * thay vì `"Slash"`, làm `⌘/` câm một lượt)*. Ở đây nhánh sản phẩm đọc `event.key`, và một sự
 * kiện tổng hợp mang đúng trường đó.
 */
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

/** Ảnh chụp lưới + thanh trạng thái, đọc THẲNG từ DOM người dùng nhìn thấy. */
async function chupLuoi() {
  return await browser.execute(() => {
    const src = [...document.querySelectorAll('[data-col="src"]')]
    return {
      soHang: src.length,
      hang: src.map((el) => ({
        id: el.getAttribute('data-segment-id'),
        nguyenVan: el.textContent,
      })),
      soThuTu: [...document.querySelectorAll('.cell-num')].map((el) => el.textContent?.trim()),
      // 🔴 AC4 — dòng báo hệ quả. `null` nghĩa là thanh **không nói gì**, và đó chính là
      // trạng thái story này tồn tại để chấm dứt.
      dongBao: document.querySelector('.status .notice')?.textContent?.trim() ?? null,
    }
  })
}

/**
 * Bắn một `keydown` **tổng hợp** mang `key: 'Backspace'` vào ô đang có caret.
 *
 * ⚠️ Bắn vào **chính ô**, không vào `document`: ba handler của lưới sống ở **cột**
 * (`.col-tgt`), và chúng dựa vào `event.target` để biết ô nào — một sự kiện bắn ở `document`
 * cho `targetCellOf` trả `null` và nhánh thoát im lặng.
 */
async function backspaceVao(idSegment) {
  await browser.execute((id) => {
    const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`)
    cell.dispatchEvent(
      new KeyboardEvent('keydown', {
        key: 'Backspace',
        code: 'Backspace',
        bubbles: true,
        cancelable: true,
      }),
    )
  }, idSegment)
}

/** Đặt caret ở **đầu** ô bản dịch của một segment, đi qua đường chuột sản phẩm trước. */
async function caretDauO(idSegment) {
  await realClick(await $(`[data-col="tgt"][data-segment-id="${idSegment}"]`))
  await browser.pause(250)
  return await browser.execute((id) => {
    const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`)
    cell.focus()
    const sel = window.getSelection()
    sel.removeAllRanges()
    const r = document.createRange()
    const dau = cell.firstChild
    if (dau && dau.nodeType === 3) r.setStart(dau, 0)
    else r.setStart(cell, 0)
    r.collapse(true)
    sel.addRange(r)
    return { type: sel.type, rangeCount: sel.rangeCount, offset: sel.getRangeAt(0).startOffset }
  }, idSegment)
}

/**
 * 🔴 **Đợi lưới hiện ĐÚNG Chương của Tác phẩm vừa dựng, không chỉ "có hàng nào đó".**
 *
 * ⚠️ Đây là một khuyết tật của **BÀN ĐO** đã trả giá để biết, 2026-08-17: `waitForExist` trên
 * `[data-col="src"]` **không phân biệt** *"Chương mới đã nạp"* với *"Chương CŨ còn nằm đó"*.
 * Ca đầu của tệp này gộp 3 hàng thành 2; ca sau dựng một Tác phẩm mới rồi đọc ngay và thấy
 * **2** — một lượt đỏ nói về bàn đo chứ không về sản phẩm *(nguyên văn: `Expected: 3,
 * Received: 2`)*.
 *
 * ⇒ Chờ **số hàng mong đợi**, không chờ "tồn tại". Đây là một **tiền đề** của ca, không phải
 * mệnh đề đang kiểm — nên chờ nó là hợp lệ, khác hẳn việc nới một ngưỡng cho hết đỏ.
 */
async function doiLuoiCo(soHang) {
  await $('[data-col="src"]').waitForExist({ timeout: 30_000 })
  await browser.waitUntil(async () => (await chupLuoi()).soHang === soHang, {
    timeout: 30_000,
    timeoutMsg:
      `Lưới không hiện đúng ${soHang} hàng sau 30 giây.\n` +
      'Ứng viên thường nhất: Chương của Tác phẩm TRƯỚC còn nằm trên lưới — fixture không\n' +
      'reset state panel (deferred-work.md, món "bộ e2e chập chờn").',
  })
}

describe('Story 2.9 — gộp bằng Backspace ở đầu ô, trong WKWebView thật', () => {
  it('🔴 AC1+AC4 — Backspace ở đầu ô gộp với câu trên, và thanh trạng thái NÓI RA hệ quả', async () => {
    await openWorkspaceWithWork('Story 2.9 — Backspace gộp', '一。二。三。')
    // Cùng lượt nạp lại của `grid-empty-cell.e2e.mjs`, cùng lý do (state module-level rò qua
    // các phiên app vì `$APPDATA` dùng chung). Vá của BÀN ĐO, không của sản phẩm.
    await browser.execute(() => {
      window.location.reload()
    })
    await doiLuoiCo(3)

    const truoc = await chupLuoi()
    await expect(truoc.dongBao).toBe(null)

    const idHai = truoc.hang[1].id
    const caret = await caretDauO(idHai)
    await expect(caret.rangeCount).toBe(1)

    await backspaceVao(idHai)

    await browser.waitUntil(async () => (await chupLuoi()).soHang === 2, {
      timeout: 15_000,
      timeoutMsg:
        'Sau `Backspace` ở đầu ô, lưới không rút xuống HAI hàng sau 15 giây.\n' +
        'Bốn ứng viên, theo thứ tự rẻ dần: ① nhánh trong `onEditKeydown` không nhận việc —\n' +
        'thường là `caretAtCellStart` trả `false`; ② `dispatch` không tới `mergeCurrentSegment`;\n' +
        '③ lệnh Rust từ chối — đọc dòng báo ở thanh trạng thái; ④ `applyRegroup` không vá\n' +
        'ảnh chụp.',
    })

    const sau = await chupLuoi()
    await expect(sau.soHang).toBe(2)
    await expect(sau.soThuTu).toEqual(['1', '2'])

    // Hàng mới mang trọn hình dạng dây, và nối KHÔNG khoảng trắng (nguồn `zh`).
    const idCu = new Set(truoc.hang.map((h) => h.id))
    const moi = sau.hang.filter((h) => !idCu.has(h.id))
    await expect(moi.length).toBe(1)
    await expect(moi[0].nguyenVan).toBe('一。二。')

    // 🔴 **AC4** — và đây là mệnh đề mà `⌘M` của Story 2.8 **không** có: một dòng báo nêu
    // HỆ QUẢ, không chỉ *"đã xong"*. Cả hai vế của AD-5 phải có mặt.
    await expect(sau.dongBao).toBe(
      'Đã gộp hai câu. Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được.',
    )
  })

  /**
   * 🔴 **HAI ĐỐI CHỨNG ÂM TRONG MỘT `it()`, và lượt gộp đó là một PHÉP ĐO chứ không một lượt
   * gom cho gọn.**
   *
   * Đo 2026-08-17: mỗi lệnh WebDriver của bộ đo này trả giá **~5 giây** — service in ra
   * `Tauri core.invoke not available after 5s timeout` ở **mỗi** lệnh, nên chi phí một ca tỉ
   * lệ **số lệnh**, không tỉ lệ việc nó làm. Ba ca rời chạy 1m40 · 1m48 · 1m40 trên trần
   * `mochaOpts.timeout = 120_000`, và ca thứ ba **trượt bằng timeout** khi đứng cuối chuỗi —
   * một lượt đỏ **không nói gì về sản phẩm** *(chạy riêng: 1/1 xanh, 1m40)*.
   *
   * ⚠️ **KHÔNG nới trần** — đó là vá triệu chứng, và `project-context.md` cấm bằng chữ. Đường
   * đúng là **giảm số lệnh**: hai đối chứng dưới đây đều **không phá lưới** *(một cái bị Rust
   * từ chối, một cái thoát trước `dispatch`)*, nên chúng dùng chung **một** lượt dựng Tác phẩm
   * — tiết kiệm trọn một `openWorkspaceWithWork` + `reload` + `waitForExist`.
   *
   * 🔵 Và chúng là **một** mệnh đề chứ không hai gộp lại: *"cử chỉ này KHÔNG được gộp khi
   * điều kiện chưa đủ"*, đo ở hai điều kiện khác nhau. Một ca đỏ vẫn chỉ đúng chỗ, vì hai
   * khối `expect` nêu đích danh điều kiện của mình.
   */
  it('🔴 AC4+AC6 — hai đối chứng ÂM: câu đầu Chương bị từ chối CÓ BÁO, và caret giữa ô KHÔNG gộp', async () => {
    // 🔴 Ca **thường nhất** của cử chỉ này — câu số 1 của mọi Chương — và ca **im lặng nhất**
    // trước story này: `editorRegroupError` được export mà không component nào đọc, nên người
    // dùng bấm `Backspace` và **không có gì xảy ra**.
    await openWorkspaceWithWork('Story 2.9 — đối chứng âm', '一。二。三。')
    await browser.execute(() => {
      window.location.reload()
    })
    await doiLuoiCo(3)

    const truoc = await chupLuoi()
    const idMot = truoc.hang[0].id
    await caretDauO(idMot)
    await backspaceVao(idMot)

    await browser.waitUntil(async () => (await chupLuoi()).dongBao !== null, {
      timeout: 15_000,
      timeoutMsg:
        'Sau `Backspace` ở câu ĐẦU Chương, thanh trạng thái vẫn im sau 15 giây.\n' +
        'Đây là ca "rỗng IM LẶNG" mà story này tồn tại để đóng — nếu nó tái hiện thì\n' +
        'hoặc nhánh không nhận việc, hoặc `regroupNotice` không được đặt ở đường từ chối.',
    })

    const sauMot = await chupLuoi()
    // Câu của **Rust**, qua `tError()` — không một câu thứ hai viết lại ở frontend.
    await expect(sauMot.dongBao).toContain('là câu đầu Chương')
    // 🔴 AC6 — *"không chặn và không hỏi lại"*: không hộp thoại nào, và **lưới không đổi**.
    await expect(sauMot.soHang).toBe(3)
    await expect(sauMot.hang.map((h) => h.id)).toEqual(truoc.hang.map((h) => h.id))

    // ── ② Caret ở GIỮA ô — nhánh phải thoát TRƯỚC khi chạm `dispatch` ─────────────
    //
    // Đối chứng này canh đúng chỗ `caretAtCellStart` có thể trả sai: một phép kiểm rộng tay
    // gộp mất một câu mà **AD-5 không cho hoàn tác**. Đây là ca mà phép kiểm
    // `startOffset === 0` *(đã bị bác bằng số đo)* vẫn đi qua được.
    const idHai = truoc.hang[1].id
    // ⚠️ Gõ chữ bằng `execCommand('insertText')` — `browser.keys()` không đi vào đường nhập
    // văn bản gốc của WKWebView (giới hạn đã ghi từ 2026-08-12).
    // 🔵 Một `execute` DUY NHẤT cho cả gõ lẫn đặt caret: mỗi lệnh WebDriver là ~5 giây, và
    // tách đôi ở đây không thêm một mệnh đề nào.
    await realClick(await $(`[data-col="tgt"][data-segment-id="${idHai}"]`))
    await browser.execute((id) => {
      const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`)
      cell.focus()
      const sel = window.getSelection()
      sel.removeAllRanges()
      const r = document.createRange()
      r.selectNodeContents(cell)
      sel.addRange(r)
      document.execCommand('insertText', false, 'bốn năm sáu')
      // Caret ở GIỮA — offset 3 của text node đầu.
      const sel2 = window.getSelection()
      sel2.removeAllRanges()
      const r2 = document.createRange()
      r2.setStart(cell.firstChild, 3)
      r2.collapse(true)
      sel2.addRange(r2)
    }, idHai)

    await backspaceVao(idHai)
    await browser.pause(1500)

    const sauHai = await chupLuoi()
    // 🔴 Lưới **không đổi** — nhánh phải thoát trước khi chạm `dispatch`.
    await expect(sauHai.soHang).toBe(3)
    // 🔵 **Và thanh trạng thái RỖNG, không mang câu từ chối của bước ①.** Bản đầu của ca này
    // khẳng định `.not.toContain('Đã gộp')` và **đỏ** với `Received has value: null` — một
    // lượt đỏ **đúng**, vì nó bắt chính chỗ tôi đọc sai sản phẩm.
    // Lý do thật: lượt `execCommand('insertText')` ở trên phát `input` → `onEditInput` →
    // `reportEdit` → `noteEditorEdit`, và hàm đó **dọn** `regroupNotice` (Task 3.6, dọn bằng
    // SỰ KIỆN). Tức hai tính năng của story này giao nhau, và ca dưới đây là chỗ duy nhất ghi
    // lại giao điểm đó: **người dùng gõ tiếp thì câu báo tắt, kể cả câu từ chối**.
    // ⇒ `toBe(null)` là mệnh đề MẠNH HƠN `.not.toContain`: nó khẳng định cả *"không câu gộp
    // nào"* lẫn *"câu cũ đã được dọn"*, trong một phép so.
    await expect(sauHai.dongBao).toBe(null)
  })

  /**
   * 🔴 **AC7 — cử chỉ đánh dấu chỗ cắt là `Mod`+click, và một cú bấm TRƠN không đánh dấu gì.**
   *
   * Ice tìm ra xung đột này bằng cách **DÙNG THẬT**, 2026-08-17 — khuôn đã lặp ở 2.5b *(hàng
   * về hưu trong lưới)* và 2.8 *(chữ ký #6 lật lần thứ ba)*: ba lý lẽ trên giấy đều đứng, và
   * cả ba cộng lại vẫn thua một lượt người thật nhìn vào một Chương thật.
   *
   * Cột nguyên văn mang **hai** cử chỉ chuột treo trên **cùng một** `mouseup`: đánh dấu chỗ
   * cắt *(2.8)* và **tra từ điển** *(FR21, Story 1.18, đã phát hành)*. ⇒ Mỗi lượt tra một từ
   * để **đọc** cũng rơi một dấu cắt; và một cú **double-click** bắn **HAI** `mouseup` nên nó
   * để lại **hai** dấu — cho một lượt `⌘/` người dùng không định gọi, trên dữ liệu mà AD-5
   * không cho hoàn tác.
   *
   * ⚠️ **GIỚI HẠN THẬT, giữ nguyên từ 2.8:** bộ đo **không giao được** một cú bấm chuột tới ô
   * `[data-col="src"]` qua ba cách nhắm *(toạ độ tuyệt đối · `origin` + lệch · `origin`
   * trần — `2-8-ban-do/tach-chan-doan.e2e.mjs`)*. Ca này vì thế bắn một `MouseEvent` **tổng
   * hợp**, thay **đúng một** mắt xích *(ai giao cú bấm)* và giữ nguyên phần còn lại:
   * `onSourceCellMouseUp` → `hasPrimaryModifier` → `caretRangeFromPoint` **thật của engine**
   * → `sourceCutOffsetOf` → `setEditorSourceCut`.
   *
   * ⚠️ **Vế KHÔNG phủ được, ghi ra thay vì để người sau tưởng đã xét:** một cú `⌘`+click
   * **CHUỘT THẬT** có tới được handler không, và double-click trơn có tra từ được không. Cả
   * hai là **món cho Ice** — và chữ ký 2026-08-17 của Ice đã đóng vế thứ hai *(double-click
   * TRA ĐƯỢC trên máy thật, đóng món nợ `deferred-work.md:4100`)*.
   */
  it('🔴 AC7 — bấm TRƠN không đánh dấu chỗ cắt; `Mod`+click thì có', async () => {
    // Câu đầu DÀI có chủ ý — cùng lý do 2.8 đã ghi: tâm ô phải rơi vào GIỮA CHỮ, không vào
    // khoảng trống sau chữ (chỗ đó cho chỗ cắt = cuối câu, và Rust từ chối đúng luật).
    await openWorkspaceWithWork('Story 2.9 — cử chỉ chỗ cắt', '一二三四五六七八九十甲乙丙丁戊己庚辛。五。')
    await browser.execute(() => {
      window.location.reload()
    })
    await doiLuoiCo(2)

    const truoc = await chupLuoi()
    const id = truoc.hang[0].id

    /**
     * Bắn `soLan` lượt `mouseup` vào **giữa chữ của dòng đầu**, có hoặc không có `Mod`.
     *
     * ⚠️ Hộp của **dòng đầu** (`getClientRects()[0]`), không hộp gộp — bài học của 2.8:
     * `getBoundingClientRect()` trả **hợp** của mọi dòng khi văn bản xuống dòng, nên "30 %
     * chiều rộng, giữa chiều cao" rơi vào **dòng thứ hai, sau chữ cuối**.
     *
     * ⚠️ `metaKey` viết thẳng vì runner **LÀ macOS**. Vế lái-hai-nền-tảng nằm ở
     * `hasPrimaryModifier` và có `tests/frontend/editorSourceCutGesture.test.ts` phủ cả hai
     * ca — một bộ e2e chỉ chạy trên macOS không phải chỗ diễn đạt mệnh đề đó.
     */
    const bam = async (segId, giuMod, soLan) =>
      await browser.execute(
        (sid, mod, lan) => {
          const cell = document.querySelector(`[data-col="src"][data-segment-id="${sid}"]`)
          const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
          let node = walker.nextNode()
          while (node !== null && node.data.length === 0) node = walker.nextNode()
          if (node === null) return { ok: false, lyDo: 'o nguyen van khong co text node nao' }
          const range = document.createRange()
          range.selectNodeContents(node)
          const rects = range.getClientRects()
          const r = rects.length > 0 ? rects[0] : range.getBoundingClientRect()
          const x = Math.round(r.x + r.width * 0.3)
          const y = Math.round(r.y + r.height / 2)
          for (let i = 0; i < lan; i++) {
            cell.dispatchEvent(
              new MouseEvent('mouseup', {
                clientX: x,
                clientY: y,
                bubbles: true,
                cancelable: true,
                metaKey: mod,
              }),
            )
          }
          return { ok: true, x, y }
        },
        segId,
        giuMod,
        soLan,
      )

    /** Đếm dấu cắt NGƯỜI DÙNG NHÌN THẤY, đọc thẳng từ DOM. */
    const demDauCat = async () =>
      await browser.execute(() => ({
        dauCat: document.querySelectorAll('[data-col="src"] .cut-mark').length,
        // 🔵 2026-08-17 — đọc `data-cut-count`, KHÔNG lớp `has-cuts`: lớp đó đã gỡ cùng
        // viền của nó (Ice chốt). Thuộc tính này chở một SỐ, tức chặt hơn một cờ.
        oCoDauCat: [...document.querySelectorAll('[data-col="src"]')].filter(
          (c) => Number(c.getAttribute('data-cut-count')) > 0,
        ).length,
      }))

    await expect((await demDauCat()).dauCat).toBe(0)

    // ── ① Bấm TRƠN ⇒ KHÔNG đánh dấu. Đây là vế Ice ký, và là toàn bộ điểm của AC7 ──
    //
    // 🔴 **HAI lượt** — đúng số `mouseup` mà một cú **double-click** bắn ra. Trước bản vá,
    // đây là chỗ để lại HAI dấu cắt cho một lượt người dùng chỉ muốn TRA TỪ.
    const bamTron = await bam(id, false, 2)
    await expect(bamTron.ok).toBe(true)
    await browser.pause(400)

    const sauTron = await demDauCat()
    await expect(sauTron.dauCat).toBe(0)
    await expect(sauTron.oCoDauCat).toBe(0)

    // ── ② `Mod`+click ⇒ CÓ đánh dấu. Đối chứng DƯƠNG — không có nó, ca ① xanh cả trên
    //    một sản phẩm mà `onSourceCellMouseUp` đã chết hẳn.
    const bamMod = await bam(id, true, 1)
    await expect(bamMod.ok).toBe(true)
    await browser.waitUntil(async () => (await demDauCat()).dauCat > 0, {
      timeout: 10_000,
      timeoutMsg:
        '`Mod`+click không đặt được chỗ cắt sau 10 giây.\n' +
        'Hai ứng viên: ① `hasPrimaryModifier` đọc sai cờ; ② `caretRangeFromPoint` không phân\n' +
        'giải được điểm bấm — ca sau thì `2-8-ban-do/` đã đo là CHẠY ĐƯỢC.',
    })
    await expect((await demDauCat()).oCoDauCat).toBe(1)
  })
})
