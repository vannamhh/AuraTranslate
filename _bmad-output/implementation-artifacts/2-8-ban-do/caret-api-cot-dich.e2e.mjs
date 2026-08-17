/**
 * Bàn đo Story 2.8 — **`placeCaretAtPoint` của Story 2.5b có đang NÉM trên mỗi cú bấm không?**
 *
 * Vòng 4 của `tach-chan-doan.e2e.mjs` đo được: `document.caretPositionFromPoint` là
 * **`undefined`** trên WKWebView này, và một lời gọi trần **NÉM `TypeError`**.
 *
 * `GridPanel.vue::placeCaretAtPoint` — viết ở Story 2.5b, **đang chạy trong bản phát hành** —
 * gọi đúng API đó, trần, ở **dòng đầu**:
 *
 *     const pos = document.caretPositionFromPoint(x, y)
 *
 * ⇒ Nếu suy luận đúng thì `onCellMouseUp` chết ngay tại đó ở **mọi** cú bấm vào ô bản dịch,
 * và lượt vá `ensureCaretNextFrame` — thứ mà doc-comment của chính nó gọi là *"đường duy
 * nhất chạy được khi engine không làm"* — **chưa bao giờ chạy**.
 *
 * ⚠️ **Đây là một SUY LUẬN cho tới khi có số.** Ca `grid-empty-cell.e2e.mjs` vẫn xanh, tức
 * caret **có** hiện — nên hoặc suy luận sai, hoặc caret đến từ `cell.focus()` cộng hành vi
 * mặc định của engine chứ không từ đường vá. Bàn đo này phân biệt hai khả năng đó.
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { realClick } from '../../../e2e/support/pointer.mjs'

describe('Bàn đo 2.8 — API caret ở cột BẢN DỊCH', () => {
  it('một cú bấm thật vào ô bản dịch có ném không', async () => {
    await openWorkspaceWithWork('Bàn đo 2.8 — caret cột dịch', '一。二。')
    await browser.execute(() => {
      window.location.reload()
    })
    await $('[data-col="tgt"]').waitForExist({ timeout: 30_000 })

    await browser.execute(() => {
      window.__c28 = { loi: [], canhBao: [] }
      window.onerror = (msg) => {
        window.__c28.loi.push(String(msg))
      }
      const goc = console.warn.bind(console)
      console.warn = (...args) => {
        window.__c28.canhBao.push(args.map(String).join(' ').slice(0, 120))
        goc(...args)
      }
      return {
        caretPositionFromPoint: typeof document.caretPositionFromPoint,
        caretRangeFromPoint: typeof document.caretRangeFromPoint,
      }
    })

    await realClick(await $('[data-col="tgt"]'))
    await browser.pause(400)

    const ketQua = await browser.execute(() => ({
      loi: window.__c28.loi,
      canhBao: window.__c28.canhBao,
      // Caret có hiện không — nếu CÓ mà vẫn ném thì nó đến từ `cell.focus()` + engine, KHÔNG
      // từ đường vá của Story 2.5b.
      selectionType: window.getSelection()?.type ?? null,
      rangeCount: window.getSelection()?.rangeCount ?? 0,
      activeCol: document.activeElement?.getAttribute?.('data-col') ?? null,
    }))
    console.log('\n[c28 · bấm ô bản dịch] ' + JSON.stringify(ketQua, null, 2))
  })
})
