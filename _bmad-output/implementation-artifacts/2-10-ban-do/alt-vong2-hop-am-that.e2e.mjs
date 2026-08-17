/**
 * Bàn đo Story 2.10 · Task 1.3 — **VÒNG 2**. Vòng 1 hỏng THƯỚC, và đối chứng của nó tự tố cáo.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÒNG 1 KHÔNG ĐO ĐƯỢC GÌ, VÀ Ⓐ NÓI RA CHÍNH XÁC VÌ SAO
 * ═════════════════════════════════════════════════════════════════════════════════
 * `browser.keys(['Alt', 'ArrowDown'])` giao **hai** `keydown` rời, và cái thứ hai mang:
 *
 *     { key: "ArrowDown", code: "ArrowDown", altKey: FALSE }
 *
 * ⇒ Hợp âm `Alt+ArrowDown` **chưa từng được gửi**. `keys.ts:509` so `sameMods` trước mọi thứ
 * khác, nên nó `continue` và vòng lặp thoát **trước** cả dòng `isTypingZone` — đúng cơ chế mà
 * `GridPanel.vue:1084-1092` đã ghi cho ca `Enter` trần. Con số *"caret không dời"* của Ⓑ vì thế
 * trả lời câu *"một `ArrowDown` KHÔNG có `Alt` có kích hoạt `editor.next_untranslated` không"* —
 * một câu không ai hỏi, và đáp án hiển nhiên là không.
 *
 * ✅ **Đối chứng dương Ⓒ đã làm đúng việc của nó:** caret **ngoài** vùng gõ cũng đứng yên. Một
 * bàn đo không có Ⓒ sẽ đọc vòng 1 thành *"đã xác nhận: luật vùng gõ nuốt `⌥↓`"* và ký một
 * quyết định lên một con số rỗng. Đây là lần thứ hai trong epic một đối chứng dương cứu một
 * lượt kết luận sai *(lần đầu: `2-9-ban-do` §Vòng 1, đối chứng Ⓔ)*.
 *
 * ⚠️ **Đây là lượt sửa THƯỚC thứ nhất, không một vòng chẩn đoán bị bác** — LUẬT DỪNG đếm những
 * vòng mà một giả thuyết về **sản phẩm** bị phép đo bác. Chưa vòng nào.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * THƯỚC MỚI: DỰNG SỰ KIỆN BẰNG `new KeyboardEvent`, KHÔNG QUA DRIVER
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 **Và nó hợp lệ cho đúng câu hỏi đang hỏi, không phải một lượt hạ chuẩn.** Câu ① của
 * Quyết định #1 là *"`keys.ts:510` — mã **JavaScript của sản phẩm** — có nuốt hợp âm này khi
 * `event.target` là một vùng gõ không?"*. Luật đó đọc **đúng ba** thứ: `event.code`,
 * `modsOf(event)`, và `event.target`. Nó **không** đọc `isTrusted`. ⇒ Một `KeyboardEvent` dựng
 * bằng tay với ba trường đó đúng chạy qua **chính** nhánh mà một phím thật chạy qua, và nó có
 * thêm một ưu điểm quyết định: hình dạng hợp âm **do bàn đo kiểm soát**, không do driver đoán.
 *
 * 🔴 **Cái nó vẫn KHÔNG trả lời, y nguyên vòng 1 — câu ②** *(`⌥↓` có bị WebKit/macOS dùng cho
 * "xuống cuối đoạn" không, và `preventDefault()` có chặn nổi nó không)*. `isTrusted` vẫn
 * `false`, và một sự kiện không tin cậy **không có default action**. ⇒ Câu ② đi thẳng vào
 * §Chờ chữ ký Ice. Đừng đọc một số nào dưới đây thành đáp án cho nó.
 *
 *     TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-10-ban-do/alt-vong2-hop-am-that.e2e.mjs
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { doiChuongVaKiemDanhTinh, in_, VACH_PRIMARY } from './danh-tinh-phien.mjs'

const CAU = (i) => `第${i}句子内容在这里。`
const SO_CAU = 3
const VAN_BAN = Array.from({ length: SO_CAU }, (_, i) => CAU(i + 1)).join('')
const CAU_DAU = CAU(1)

/**
 * Bắn một hợp âm **đúng hình dạng** vào một target cho trước, và **tự kiểm hình dạng đó** ngay
 * trong cùng lời gọi.
 *
 * 🔴 `tuKiem` không phải một lượt cẩn thận thừa: nó là thứ làm vòng 1 lộ ra. Một bàn đo in ra
 * kết quả mà **không** in ra hình dạng đầu vào là một bàn đo không kiểm chứng lại được.
 */
const BAN_HOP_AM = `
function banHopAm(target, opt) {
  const nhan = { batDuoc: null }
  const ghi = (e) => {
    if (nhan.batDuoc === null) {
      nhan.batDuoc = {
        code: e.code, key: e.key, altKey: e.altKey, metaKey: e.metaKey,
        ctrlKey: e.ctrlKey, shiftKey: e.shiftKey, isTrusted: e.isTrusted,
        targetCol: e.target && e.target.getAttribute ? e.target.getAttribute('data-col') : null,
        targetLaVungGo: e.target ? e.target.isContentEditable === true : null,
      }
    }
  }
  document.addEventListener('keydown', ghi, true)
  const ev = new KeyboardEvent('keydown', {
    key: opt.key, code: opt.code,
    altKey: opt.altKey === true, metaKey: opt.metaKey === true,
    ctrlKey: opt.ctrlKey === true, shiftKey: opt.shiftKey === true,
    bubbles: true, cancelable: true, composed: true,
  })
  target.dispatchEvent(ev)
  document.removeEventListener('keydown', ghi, true)
  return { tuKiem: nhan.batDuoc, defaultPreventedSauKhiBan: ev.defaultPrevented }
}
`

describe('Bàn đo 2.10 vòng 2 — hợp âm ĐÚNG HÌNH DẠNG vào luật vùng gõ', () => {
  before(async () => {
    await openWorkspaceWithWork('Bàn đo 2.10 v2 — hợp âm', VAN_BAN)
    in_('Task 1.1 · danh tính phiên', await doiChuongVaKiemDanhTinh(SO_CAU, CAU_DAU))
  })

  it('Ⓘ 🔴 CÂU ① — `⌥↓` với target LÀ ô đang gõ: `keys.ts:510` có nuốt không', async () => {
    const r = await browser.execute(
      async (nguonBan, nguonVach) => {
        const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
        try {
          // eslint-disable-next-line no-eval
          eval(nguonBan)
          // eslint-disable-next-line no-eval
          eval(nguonVach)
          const cell = document.querySelectorAll('[data-col="tgt"]')[0]
          cell.focus()
          const sel = window.getSelection()
          sel.removeAllRanges()
          const r0 = document.createRange()
          r0.setStart(cell, 0)
          r0.collapse(true)
          sel.addRange(r0)
          await doiKhungHinh()
          // eslint-disable-next-line no-undef
          const truoc = hangCoVachPrimary()

          // eslint-disable-next-line no-undef
          const ban = banHopAm(cell, { key: 'ArrowDown', code: 'ArrowDown', altKey: true })
          await doiKhungHinh()
          await new Promise((res) => setTimeout(res, 200))
          // eslint-disable-next-line no-undef
          const sau = hangCoVachPrimary()

          return {
            ...ban,
            vachTruoc: truoc,
            vachSau: sau,
            // 🔴 Mệnh đề: nếu vạch KHÔNG dời thì lệnh không chạy ⇒ luật vùng gõ ĐÃ nuốt.
            lenhDaChay: truoc.hang !== sau.hang,
          }
        } catch (err) {
          return { loiChup: String(err) }
        }
      },
      BAN_HOP_AM,
      VACH_PRIMARY,
    )
    in_('Ⓘ ⌥↓ với target = ô đang gõ', r)
  })

  it('Ⓙ 🔴 ĐỐI CHỨNG DƯƠNG — cùng hợp âm, target NGOÀI vùng gõ ⇒ lệnh PHẢI chạy', async () => {
    const r = await browser.execute(
      async (nguonBan, nguonVach) => {
        const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
        try {
          // eslint-disable-next-line no-eval
          eval(nguonBan)
          // eslint-disable-next-line no-eval
          eval(nguonVach)
          // Caret state của Editor vẫn ở hàng 0 từ ca trước; chỉ đổi TARGET của sự kiện.
          // Đây là biến duy nhất khác giữa Ⓘ và Ⓙ — đúng thứ một đối chứng cần.
          // eslint-disable-next-line no-undef
          const truoc = hangCoVachPrimary()
          // eslint-disable-next-line no-undef
          const ban = banHopAm(document.body, { key: 'ArrowDown', code: 'ArrowDown', altKey: true })
          await doiKhungHinh()
          await new Promise((res) => setTimeout(res, 200))
          // eslint-disable-next-line no-undef
          const sau = hangCoVachPrimary()
          return { ...ban, vachTruoc: truoc, vachSau: sau, lenhDaChay: truoc.hang !== sau.hang }
        } catch (err) {
          return { loiChup: String(err) }
        }
      },
      BAN_HOP_AM,
      VACH_PRIMARY,
    )
    in_('Ⓙ ⌥↓ với target = body (ngoài vùng gõ) — ĐỐI CHỨNG DƯƠNG', r)
  })

  it('Ⓚ tiền đề của Quyết định #2(b) — hợp âm mang `Mod` có đi qua luật vùng gõ không', async () => {
    const r = await browser.execute(
      async (nguonBan, nguonVach) => {
        const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
        try {
          // eslint-disable-next-line no-eval
          eval(nguonBan)
          // eslint-disable-next-line no-eval
          eval(nguonVach)
          const cell = document.querySelectorAll('[data-col="tgt"]')[0]
          cell.focus()
          const sel = window.getSelection()
          sel.removeAllRanges()
          const r0 = document.createRange()
          r0.setStart(cell, 0)
          r0.collapse(true)
          sel.addRange(r0)
          await doiKhungHinh()

          // `Mod+Enter` = `editor.confirm_segment`, hợp âm MANG phím bổ trợ chính đã đăng ký.
          // 🔴 Phép đo ở đây là `defaultPrevented`, KHÔNG phải kết quả xác nhận: `keys.ts:513`
          //    gọi `preventDefault()` **ngay khi hợp âm khớp và đi qua luật vùng gõ**, trước
          //    cả `dispatch`. ⇒ Nó là tín hiệu trực tiếp và tất định cho câu "có đi qua không",
          //    trong khi lượt xác nhận còn phụ thuộc IPC, flush và trạng thái ô.
          // eslint-disable-next-line no-undef
          const modEnter = banHopAm(cell, { key: 'Enter', code: 'Enter', metaKey: true })

          // Cùng phép đo cho `⌥↓` trong ô — hai số đặt cạnh nhau đọc ra ngay chỗ chúng lệch.
          // eslint-disable-next-line no-undef
          const altXuong = banHopAm(cell, { key: 'ArrowDown', code: 'ArrowDown', altKey: true })

          // Và một hợp âm KHÔNG đăng ký, để biết `defaultPrevented` không phải luôn `true`.
          // eslint-disable-next-line no-undef
          const khongDangKy = banHopAm(cell, { key: 'F9', code: 'F9' })

          return {
            'Mod+Enter (co Mod, DA dang ky)': modEnter,
            'Alt+ArrowDown (khong Mod, DA dang ky)': altXuong,
            'F9 (doi chung AM — chua dang ky)': khongDangKy,
          }
        } catch (err) {
          return { loiChup: String(err) }
        }
      },
      BAN_HOP_AM,
      VACH_PRIMARY,
    )
    in_('Ⓚ `defaultPrevented` — ba hợp âm trong CÙNG một ô đang gõ', r)
  })
})
