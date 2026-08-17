/**
 * Bàn đo Story 2.9 — **dấu cắt cao 1,3em có đẩy CHIỀU CAO HÀNG không?**
 *
 * 🔴 Lưới dùng `subgrid`, nên một phần tử inline cao hơn line box của ô nguyên văn sẽ đẩy
 * **cả track hàng** — và kéo theo **ô bản dịch** ở cột bên cạnh. Cái giá đó đã đo một lần ở
 * Story 2.5b: hàng Hán Việt song song cao **388px** và `subgrid` ép ô bản dịch cao y hệt.
 *
 * ⇒ Một lượt đổi `height: 1em` → `1.3em` **không được** tin bằng mắt. Bàn đo này đo chiều cao
 * hàng TRƯỚC và SAU khi đặt một điểm cắt, trên cùng một Chương.
 *
 *     TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-9-ban-do/dau-cat-chieu-cao.e2e.mjs
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'

async function doCao() {
  return await browser.execute(() => {
    const src = document.querySelector('[data-col="src"]')
    const tgt = document.querySelector('[data-col="tgt"]')
    return {
      caoNguonPx: Math.round(src.getBoundingClientRect().height * 100) / 100,
      caoDichPx: Math.round(tgt.getBoundingClientRect().height * 100) / 100,
      soDauCat: document.querySelectorAll('.cut-mark').length,
    }
  })
}

describe('Bàn đo 2.9 — dấu cắt và chiều cao hàng', () => {
  it('đặt một điểm cắt KHÔNG được đổi chiều cao hàng', async () => {
    await openWorkspaceWithWork('Bàn đo 2.9 — chiều cao', '一二三四五六七八九十甲乙丙丁戊己庚辛。五。')
    await browser.execute(() => {
      window.location.reload()
    })
    await $('[data-col="src"]').waitForExist({ timeout: 30_000 })

    const truoc = await doCao()
    console.log('\n[2.9·cao · TRƯỚC khi đặt điểm cắt] ' + JSON.stringify(truoc, null, 2))

    const bam = await browser.execute(() => {
      const cell = document.querySelector('[data-col="src"]')
      const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
      let node = walker.nextNode()
      while (node !== null && node.data.length === 0) node = walker.nextNode()
      if (node === null) return { ok: false }
      const range = document.createRange()
      range.selectNodeContents(node)
      const rects = range.getClientRects()
      const r = rects.length > 0 ? rects[0] : range.getBoundingClientRect()
      cell.dispatchEvent(
        new MouseEvent('mouseup', {
          clientX: Math.round(r.x + r.width * 0.3),
          clientY: Math.round(r.y + r.height / 2),
          bubbles: true,
          cancelable: true,
          metaKey: true,
        }),
      )
      return { ok: true }
    })
    await expect(bam.ok).toBe(true)
    await browser.pause(600)

    const sau = await doCao()
    console.log('\n[2.9·cao · SAU khi đặt điểm cắt] ' + JSON.stringify(sau, null, 2))
    console.log(
      '\n[2.9·cao · CHÊNH] ' +
        JSON.stringify({
          nguon: Math.round((sau.caoNguonPx - truoc.caoNguonPx) * 100) / 100,
          dich: Math.round((sau.caoDichPx - truoc.caoDichPx) * 100) / 100,
        }),
    )

    // 🔴 Dấu cắt phải CÓ, nếu không phép đo chiều cao là phép đo về hư không.
    await expect(sau.soDauCat).toBeGreaterThan(0)
    await expect(sau.caoNguonPx).toBe(truoc.caoNguonPx)
    await expect(sau.caoDichPx).toBe(truoc.caoDichPx)
  })
})
