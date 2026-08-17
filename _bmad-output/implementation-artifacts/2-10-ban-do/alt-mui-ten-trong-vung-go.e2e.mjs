/**
 * Bàn đo Story 2.10 · **Task 1.3** — CHẶN Quyết định #1.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 ĐỌC KHỐI NÀY TRƯỚC KHI ĐỌC MỘT CON SỐ NÀO — MỘT NỬA CÂU HỎI KHÔNG ĐO ĐƯỢC Ở ĐÂY
 * ═════════════════════════════════════════════════════════════════════════════════
 * Quyết định #1 hỏi hai câu, và chúng **không cùng một hạng**:
 *
 *   ① *"`keys.ts:510` có nuốt `⌥↓` khi caret ở trong ô không?"* — **ĐO ĐƯỢC**, và đo được ở
 *      đây. Luật đó là mã **của sản phẩm** chạy trên `event.target` và `event.code`; nó không
 *      hỏi `isTrusted`. Một sự kiện driver đi qua đúng nhánh đó như một phím thật.
 *
 *   ② *"`⌥↓` trong một ô văn bản trên macOS có bị hệ điều hành / WebKit dùng cho 'xuống cuối
 *      đoạn' không, và `preventDefault()` có chặn nổi nó không?"* — 🔴 **KHÔNG ĐO ĐƯỢC**, và
 *      không phải vì thiếu công. Mọi sự kiện `browser.keys()` giao đều `isTrusted: false`, và
 *      một sự kiện **không tin cậy KHÔNG CÓ default action** — đo ở Story 2.9, vòng 1
 *      (`2-9-ban-do/README.md` §Vòng 1): `Backspace` với caret ở **giữa** ô cũng không xoá gì.
 *
 * ⚠️ **Hệ quả phương pháp, và nó là chỗ một bàn đo dễ nói dối nhất:** nếu tệp này gắn một
 * listener `preventDefault()` rồi báo *"đã chặn thành công"*, con số đó sẽ **CÓ** trên mọi
 * engine — kể cả một engine không cho chặn — vì chẳng có default action nào để chặn ngay từ
 * đầu. ⇒ Câu ② đi thẳng vào §Chờ chữ ký Ice. **Đừng đọc bất kỳ số nào dưới đây thành một câu
 * trả lời cho nó.** *(Cạm bẫy ③ của story, và tiền lệ `2-9-ban-do` §Ba thứ KHÔNG đo được.)*
 *
 * Tệp này vẫn ghi `isTrusted` ở **mọi** bước — không phải thừa: nó là bằng chứng tại chỗ cho
 * người đọc sau rằng giới hạn trên đã được kiểm chứ không được giả định.
 *
 *     TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-10-ban-do/alt-mui-ten-trong-vung-go.e2e.mjs
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { doiChuongVaKiemDanhTinh, in_, VACH_PRIMARY } from './danh-tinh-phien.mjs'

/** Ba câu: hàng 0 để gõ vào, hàng 1 và 2 để `⌥↓` có đích mà nhảy tới. */
const CAU = (i) => `第${i}句子内容在这里。`
const SO_CAU = 3
const VAN_BAN = Array.from({ length: SO_CAU }, (_, i) => CAU(i + 1)).join('')
const CAU_DAU = CAU(1)

/**
 * Gắn một máy ghi sự kiện `keydown` ở tầng **capture của `document`** rồi trả về những gì nó
 * thấy. Capture trên `document` là chỗ `keys.ts` cũng nghe, nên nó thấy đúng thứ luật vùng gõ
 * thấy — không sớm hơn, không muộn hơn.
 */
const MAY_GHI = `
window.__banDo210 = { su: [] }
window.__banDo210.ghi = function (e) {
  window.__banDo210.su.push({
    key: e.key,
    code: e.code,
    altKey: e.altKey,
    metaKey: e.metaKey,
    ctrlKey: e.ctrlKey,
    isTrusted: e.isTrusted,
    cancelable: e.cancelable,
    repeat: e.repeat,
    targetTen: e.target && e.target.nodeName,
    targetCol: e.target && e.target.getAttribute ? e.target.getAttribute('data-col') : null,
    targetContentEditable: e.target ? e.target.isContentEditable === true : null,
    defaultPreventedNgaySauKhiBan: e.defaultPrevented,
  })
}
document.addEventListener('keydown', window.__banDo210.ghi, true)
`

describe('Bàn đo 2.10 — `⌥↓` khi caret ở TRONG ô bản dịch', () => {
  before(async () => {
    await openWorkspaceWithWork('Bàn đo 2.10 — alt mũi tên', VAN_BAN)
    in_('Task 1.1 · danh tính phiên', await doiChuongVaKiemDanhTinh(SO_CAU, CAU_DAU))
  })

  it('Ⓐ tiền đề — driver giao `code`/`key` nào cho `⌥↓`, và `isTrusted` bằng gì', async () => {
    await browser.execute((nguon) => {
      // eslint-disable-next-line no-eval
      eval(nguon)
      const cell = document.querySelectorAll('[data-col="tgt"]')[0]
      cell.focus()
      const sel = window.getSelection()
      sel.removeAllRanges()
      const r = document.createRange()
      r.setStart(cell, 0)
      r.collapse(true)
      sel.addRange(r)
    }, MAY_GHI)

    await browser.keys(['Alt', 'ArrowDown'])
    await browser.pause(300)

    const r = await browser.execute(() => ({
      suKien: window.__banDo210.su,
      hasFocus: document.hasFocus(),
      // 🔴 Ghi ra để không ai đọc nhầm bảng này thành một câu trả lời cho câu ②.
      ghiChu:
        'isTrusted=false ⇒ khong co default action ⇒ khong ket luan gi ve viec WebKit ' +
        'co dung ⌥↓ cho "xuong cuoi doan" hay khong. Xem khoi dau tep.',
    }))
    in_('Ⓐ driver giao gì cho ⌥↓ (tiền đề, KHÔNG phải kết luận)', r)
  })

  it('Ⓑ 🔴 CÂU ĐO ĐƯỢC — `keys.ts:510` có NUỐT `⌥↓` khi caret trong ô không', async () => {
    // Đo bằng **hành vi sản phẩm**, không bằng một listener của bàn đo: nếu lệnh chạy thì
    // caret dời sang câu chưa dịch kế tiếp và `[data-caret]` đổi hàng. Nếu bị nuốt thì không.
    const r = await browser.execute(async (nguon) => {
      const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
      try {
        // eslint-disable-next-line no-eval
        eval(nguon)
        const cell = document.querySelectorAll('[data-col="tgt"]')[0]
        cell.focus()
        const sel = window.getSelection()
        sel.removeAllRanges()
        const r0 = document.createRange()
        r0.setStart(cell, 0)
        r0.collapse(true)
        sel.addRange(r0)
        await doiKhungHinh()
        const neo = window.getSelection().anchorNode
        const oNeo = neo && neo.nodeType === 1 ? neo : neo && neo.parentElement
        return {
          caretDangONodeNao: oNeo === null ? null : oNeo.getAttribute('data-segment-id'),
          // eslint-disable-next-line no-undef
          vachPrimary: hangCoVachPrimary(),
          targetLaVungGo: document.activeElement
            ? document.activeElement.isContentEditable === true
            : null,
        }
      } catch (err) {
        return { loiChup: String(err) }
      }
    }, VACH_PRIMARY)
    in_('Ⓑ① trạng thái TRƯỚC khi bấm ⌥↓ (caret đặt ở hàng 0)', r)

    await browser.keys(['Alt', 'ArrowDown'])
    await browser.pause(500)

    const sau = await browser.execute((nguon) => {
      // eslint-disable-next-line no-eval
      eval(nguon)
      const neo = window.getSelection().anchorNode
      const oNeo = neo && neo.nodeType === 1 ? neo : neo && neo.parentElement
      return {
        caretDangONodeNao: oNeo === null ? null : oNeo.getAttribute('data-segment-id'),
        // eslint-disable-next-line no-undef
        vachPrimary: hangCoVachPrimary(),
        activeElementCol: document.activeElement
          ? document.activeElement.getAttribute('data-col')
          : null,
        activeElementSegmentId: document.activeElement
          ? document.activeElement.getAttribute('data-segment-id')
          : null,
      }
    }, VACH_PRIMARY)
    in_('Ⓑ② trạng thái SAU khi bấm ⌥↓ trong vùng gõ — dời ⇒ lệnh chạy; đứng yên ⇒ BỊ NUỐT', sau)
  })

  it('Ⓒ đối chứng DƯƠNG — cùng phím, caret NGOÀI vùng gõ ⇒ lệnh phải chạy', async () => {
    // 🔴 Không có ca này thì Ⓑ vô nghĩa: một lượt "đứng yên" ở Ⓑ có thể là vì luật vùng gõ,
    //    HOẶC vì lệnh `editor.next_untranslated` không chạy được từ driver ở bất kỳ đâu.
    //    Đối chứng này tách hai khả năng đó ra.
    const truoc = await browser.execute(async (nguon) => {
      const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
      // eslint-disable-next-line no-eval
      eval(nguon)
      // Nhả tiêu điểm khỏi mọi ô soạn thảo — bấm ra ngoài lưới bằng cách focus `body`.
      if (document.activeElement && document.activeElement.blur) document.activeElement.blur()
      window.getSelection().removeAllRanges()
      await doiKhungHinh()
      return {
        activeElementTen: document.activeElement ? document.activeElement.nodeName : null,
        activeElementLaVungGo: document.activeElement
          ? document.activeElement.isContentEditable === true
          : null,
        // eslint-disable-next-line no-undef
        vachPrimary: hangCoVachPrimary(),
      }
    }, VACH_PRIMARY)
    in_('Ⓒ① trạng thái TRƯỚC — tiêu điểm đã rời vùng gõ', truoc)

    await browser.keys(['Alt', 'ArrowDown'])
    await browser.pause(500)

    const sau = await browser.execute((nguon) => {
      // eslint-disable-next-line no-eval
      eval(nguon)
      return {
        // eslint-disable-next-line no-undef
        vachPrimary: hangCoVachPrimary(),
        activeElementCol: document.activeElement
          ? document.activeElement.getAttribute('data-col')
          : null,
        activeElementSegmentId: document.activeElement
          ? document.activeElement.getAttribute('data-segment-id')
          : null,
      }
    }, VACH_PRIMARY)
    in_('Ⓒ② trạng thái SAU — nếu ĐÂY cũng đứng yên thì Ⓑ không kết luận được gì', sau)
  })

  it('Ⓓ hợp âm mang `Mod` có đi qua luật vùng gõ không — tiền đề của Quyết định #2(b)', async () => {
    // `⌘Enter` (`editor.confirm_segment`) là hợp âm MANG phím bổ trợ chính **đã đăng ký** duy
    // nhất chạy trong ô. Nó là đối chứng cho mệnh đề *"một hợp âm có `Mod` đi qua `:510` sạch"*
    // — mệnh đề mà đường (b) của Quyết định #2 dựa vào. Đo nó ở đây thay vì suy từ mã.
    await browser.execute(async () => {
      const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
      const cell = document.querySelectorAll('[data-col="tgt"]')[0]
      cell.focus()
      const sel = window.getSelection()
      sel.removeAllRanges()
      const r = document.createRange()
      r.selectNodeContents(cell)
      sel.addRange(r)
      document.execCommand('insertText', false, 'một bản dịch thử')
      await doiKhungHinh()
    })
    await browser.pause(300)

    const truoc = await browser.execute((nguon) => {
      // eslint-disable-next-line no-eval
      eval(nguon)
      return {
        // eslint-disable-next-line no-undef
        vachPrimary: hangCoVachPrimary(),
        activeElementSegmentId: document.activeElement
          ? document.activeElement.getAttribute('data-segment-id')
          : null,
      }
    }, VACH_PRIMARY)
    in_('Ⓓ① trước ⌘Enter (caret trong ô, ô đã có chữ)', truoc)

    await browser.keys(['Meta', 'Enter'])
    await browser.pause(1500)

    const sau = await browser.execute((nguon) => {
      // eslint-disable-next-line no-eval
      eval(nguon)
      return {
        // eslint-disable-next-line no-undef
        vachPrimary: hangCoVachPrimary(),
        activeElementSegmentId: document.activeElement
          ? document.activeElement.getAttribute('data-segment-id')
          : null,
      }
    }, VACH_PRIMARY)
    in_('Ⓓ② sau ⌘Enter — dời sang câu kế ⇒ hợp âm có `Mod` ĐI QUA được luật vùng gõ', sau)
  })
})
