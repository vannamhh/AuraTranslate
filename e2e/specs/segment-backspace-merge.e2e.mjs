/**
 * Backspace at the start of a target cell does not merge; merging goes through `⌘M` and the sentence menu.
 * Driver events are `isTrusted: false`, so the native no-op of a real key is measured in
 * `_bmad-output/implementation-artifacts/bo-cu-chi-backspace-ban-do/`.
 */
import { waitForGridRows } from '../support/gridWait.mjs'
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

/** Dispatches on the cell: the grid handlers live on the column and read `event.target`. */
async function backspaceVao(idSegment, soLan) {
  await browser.execute(
    (id, lan) => {
      const cell = document.querySelector(`[data-col="tgt"][data-segment-id="${id}"]`)
      for (let i = 0; i < lan; i += 1) {
        cell.dispatchEvent(
          new KeyboardEvent('keydown', {
            key: 'Backspace',
            code: 'Backspace',
            bubbles: true,
            cancelable: true,
          }),
        )
      }
    },
    idSegment,
    soLan,
  )
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
  // 🔵 **STORY 2.12 · AC3** — khuôn tự vá tại chỗ nay đi qua helper dùng chung
  // `support/gridWait.mjs`, để chỗ thứ tư không phải phát hiện lại cùng một điều.
  await waitForGridRows(soHang)
}

describe('Backspace ở đầu ô không gộp câu, trong WKWebView thật', () => {
  it('🔴 ô rỗng, caret đầu ô, bấm năm cú: lưới và thanh trạng thái không đổi', async () => {
    await openWorkspaceWithWork('Backspace không gộp', '一。二。三。')
    await doiLuoiCo(3)

    const truoc = await chupLuoi()
    await expect(truoc.dongBao).toBe(null)

    const idHai = truoc.hang[1].id
    const caret = await caretDauO(idHai)
    await expect(caret.rangeCount).toBe(1)

    await backspaceVao(idHai, 5)
    await browser.pause(1500)

    const sau = await chupLuoi()
    await expect(sau.hang).toEqual(truoc.hang)
    await expect(sau.dongBao).toBe(null)
  })

  it('🔴 `⌘M` ở câu đầu Chương bị từ chối có báo, ở câu sau thì gộp và nói ra hệ quả', async () => {
    await openWorkspaceWithWork('Gộp bằng hợp âm', '一。二。三。')
    await doiLuoiCo(3)

    const truoc = await chupLuoi()
    await caretDauO(truoc.hang[0].id)
    await browser.keys(['Meta', 'm'])

    await browser.waitUntil(async () => (await chupLuoi()).dongBao !== null, {
      timeout: 15_000,
      timeoutMsg: 'Sau `⌘M` ở câu ĐẦU Chương, thanh trạng thái vẫn im sau 15 giây.',
    })
    const tuChoi = await chupLuoi()
    await expect(tuChoi.dongBao).toContain('là câu đầu Chương')
    await expect(tuChoi.hang.map((h) => h.id)).toEqual(truoc.hang.map((h) => h.id))

    await caretDauO(truoc.hang[1].id)
    await browser.keys(['Meta', 'm'])
    await browser.waitUntil(async () => (await chupLuoi()).soHang === 2, {
      timeout: 15_000,
      timeoutMsg: 'Sau `⌘M`, lưới không rút xuống HAI hàng sau 15 giây.',
    })

    const sau = await chupLuoi()
    await expect(sau.soThuTu).toEqual(['1', '2'])
    await expect(sau.hang[0].nguyenVan).toBe('一。二。')
    await expect(sau.dongBao).toBe(
      'Đã gộp hai câu. Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được.',
    )
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
   * TRA ĐƯỢC trên máy thật, đóng món nợ `deferred-work.md §*Deferred from: code review of 2-5d-ngat-doan-ban-dich (2026-08-16)*`)*.
   */
  it('🔴 AC7 — bấm TRƠN không đánh dấu chỗ cắt; `Mod`+click thì có', async () => {
    // Câu đầu DÀI có chủ ý — cùng lý do 2.8 đã ghi: tâm ô phải rơi vào GIỮA CHỮ, không vào
    // khoảng trống sau chữ (chỗ đó cho chỗ cắt = cuối câu, và Rust từ chối đúng luật).
    await openWorkspaceWithWork('Story 2.9 — cử chỉ chỗ cắt', '一二三四五六七八九十甲乙丙丁戊己庚辛。五。')
    // 🔵 **STORY 2.12 · AC2 — lượt `reload()` ở đây ĐÃ GỠ 2026-08-18.**
    // Fixture `openWorkspaceWithWork` nay tự dọn state panel bằng `resetPanelState()` (cầu
    // `import()` gọi thẳng năm hàm `reset*`, quyết định #5(b) Ice ký). Vá tại chỗ hết việc.
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
