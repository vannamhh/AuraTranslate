/**
 * Bàn đo Story 2.10 · phụ lục — **driver giao nổi hợp âm nào?**
 *
 * Vòng 1 của Task 1.3 đo được `browser.keys(['Alt','ArrowDown'])` giao hai `keydown` rời, cái
 * thứ hai mang `altKey: false`. Câu hỏi còn lại trước khi viết spec e2e: `['Meta','Alt',
 * 'ArrowDown']` có khá hơn không, và `['Meta','Enter']` *(đã dùng ở một spec đang chạy)* có
 * thật sự ghép không.
 *
 * ⚠️ Đây là một phép đo về **THƯỚC**, không về sản phẩm. Kết quả quyết định hình dạng spec e2e
 * chứ không quyết định một dòng mã sản phẩm nào.
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { doiChuongVaKiemDanhTinh, in_ } from './danh-tinh-phien.mjs'

const CAU = (i) => `第${i}句子内容在这里。`
const SO_CAU = 3
const VAN_BAN = Array.from({ length: SO_CAU }, (_, i) => CAU(i + 1)).join('')

async function thu(nhan, phim) {
  await browser.execute(() => {
    window.__probe = []
    window.__probeGhi = (e) => {
      window.__probe.push({
        key: e.key,
        code: e.code,
        metaKey: e.metaKey,
        altKey: e.altKey,
        ctrlKey: e.ctrlKey,
        shiftKey: e.shiftKey,
      })
    }
    document.addEventListener('keydown', window.__probeGhi, true)
  })
  await browser.keys(phim)
  await browser.pause(300)
  const r = await browser.execute(() => {
    document.removeEventListener('keydown', window.__probeGhi, true)
    return window.__probe
  })
  in_(nhan, r)
}

describe('Bàn đo 2.10 phụ lục — driver ghép phím bổ trợ tới đâu', () => {
  before(async () => {
    await openWorkspaceWithWork('Bàn đo 2.10 — probe hợp âm', VAN_BAN)
    await doiChuongVaKiemDanhTinh(SO_CAU, CAU(1))
    // Caret vào một ô đang gõ — đúng ngữ cảnh mà spec e2e sẽ chạy.
    await browser.execute(() => {
      const cell = document.querySelectorAll('[data-col="tgt"]')[0]
      cell.focus()
      const sel = window.getSelection()
      sel.removeAllRanges()
      const r = document.createRange()
      r.setStart(cell, 0)
      r.collapse(true)
      sel.addRange(r)
    })
  })

  it('bốn hình dạng lời gọi `browser.keys`', async () => {
    await thu('① [Meta, Alt, ArrowDown]', ['Meta', 'Alt', 'ArrowDown'])
    await thu('② [Meta, Enter] — đối chứng, hợp âm một mod đang được dùng thật', ['Meta', 'Enter'])
    await thu('③ chuỗi ghép "\\uE03D\\uE00A\\uE015" (Meta+Alt+ArrowDown, WebDriver keys)', '')
    await thu('④ [Meta, "2"] — đúng lời gọi mà `openWorkspaceWithWork` dùng', ['Meta', '2'])
  })
})
