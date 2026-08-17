/**
 * Bàn đo Story 2.10 · Task 1.2 — **VÒNG 2**, và nó truy một con số vòng 1 tình cờ bắt được.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÒNG 1 TRẢ LỜI XONG BỐN CÂU, RỒI LÀM LỘ MỘT CÂU THỨ NĂM KHÔNG AI HỎI
 * ═════════════════════════════════════════════════════════════════════════════════
 * Bốn số của vòng 1 đều sạch: cuộn đúng track hàng · năm cột cùng `top` · 12/12 mẫu
 * `scrollTop` giống nhau *(không hiệu ứng)* · `nearest` đứng yên khi hàng đã trong vùng nhìn.
 *
 * Nhưng trường `hopCoTuDichKhong` — thêm vào như một phép đo phòng xa — trả **`true`**:
 * `.grid-scroll` **tự dịch 18 px** (`top` 119 → 101) trong chính lượt gọi. ⇒ `scrollIntoView`
 * đã cuộn **một tổ tiên** ngoài hộp cuộn của lưới. `document.scrollingElement.scrollTop` vẫn
 * là **0**, nên thủ phạm nằm **giữa** `.grid-scroll` và `documentElement`.
 *
 * ⚠️ **Vì sao 18 px không phải một chi tiết bỏ qua được:** đường nóng của story này là một phím
 * bấm **liên tục**. Nếu mỗi lượt điều hướng xô bố cục panel đi một quãng, người dùng thấy cả
 * khung chữ nhảy chứ không thấy một hàng được cuộn tới — và **không AC nào nêu**, **không cổng
 * nào bắt**, `happy-dom` thì không bố cục nên vitest cũng không. Đúng lớp lỗi mà cả story này
 * tồn tại để chống, chỉ khác là nó ở tầng thị giác.
 *
 * ⚠️ **Đây là một câu hỏi MỚI, không một giả thuyết bị bác.** LUẬT DỪNG đếm những vòng mà một
 * giả thuyết về **sản phẩm** bị phép đo bác *(tiền lệ: `2-9-ban-do` §Vòng 1)*. Vòng này hỏi
 * thêm, không hỏi lại.
 *
 * Ba câu:
 *   Ⓕ **Tổ tiên nào** cuộn, và bao nhiêu — đi ngược cây từ `.grid-scroll` lên `documentElement`,
 *     chụp `scrollTop`/`scrollLeft` của **từng** nút trước và sau.
 *   Ⓖ Đường **(b)** *(gán `scrollTop` bằng tay)* có gây cùng tác dụng phụ không — đây là vế
 *     phân định giữa hai đường, và vòng 1 **không** đo nó *(Ⓔ chỉ so `scrollTop` của hộp)*.
 *   Ⓗ Nó có **lặp lại** ở lượt gọi thứ hai, thứ ba không — một lượt dịch **một lần rồi thôi**
 *     *(vì trạng thái đầu chưa chuẩn)* khác hẳn một lượt dịch **mỗi lần bấm phím**.
 *
 *     TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-10-ban-do/cuon-vong2-to-tien.e2e.mjs
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { doiChuongVaKiemDanhTinh, in_ } from './danh-tinh-phien.mjs'

const SO_CAU = 60
const CAU = (i) => `第${i}句子内容在这里。`
const VAN_BAN = Array.from({ length: SO_CAU }, (_, i) => CAU(i + 1)).join('')
const CAU_DAU = CAU(1)
const HANG_DICH = 50

/** Chụp `scrollTop`/`scrollLeft` của **mọi** nút từ `.grid-scroll` lên tới `documentElement`. */
const CHUP_TO_TIEN = `
function chupToTien() {
  const ra = []
  let el = document.querySelector('.grid-scroll')
  while (el !== null) {
    ra.push({
      ten: el.nodeName,
      lop: el.className && el.className.baseVal === undefined ? String(el.className).slice(0, 60) : null,
      id: el.id || null,
      scrollTop: el.scrollTop,
      scrollLeft: el.scrollLeft,
      top: Math.round(el.getBoundingClientRect().top),
      overflowY: getComputedStyle(el).overflowY,
      cuonDuoc: el.scrollHeight > el.clientHeight,
    })
    el = el.parentElement
  }
  return ra
}
function lechToTien(a, b) {
  const ra = []
  for (let i = 0; i < Math.min(a.length, b.length); i += 1) {
    if (a[i].scrollTop !== b[i].scrollTop || a[i].top !== b[i].top) {
      ra.push({
        nut: a[i].ten + (a[i].id ? '#' + a[i].id : '') + (a[i].lop ? '.' + a[i].lop : ''),
        scrollTop: a[i].scrollTop + ' -> ' + b[i].scrollTop,
        top: a[i].top + ' -> ' + b[i].top,
        cuonDuoc: a[i].cuonDuoc,
        overflowY: a[i].overflowY,
      })
    }
  }
  return ra
}
`

describe('Bàn đo 2.10 vòng 2 — tổ tiên nào bị scrollIntoView cuộn theo', () => {
  before(async () => {
    await openWorkspaceWithWork('Bàn đo 2.10 v2 — tổ tiên', VAN_BAN)
    in_('Task 1.1 · danh tính phiên', await doiChuongVaKiemDanhTinh(SO_CAU, CAU_DAU))
  })

  it('Ⓕ đường (a) — chụp CẢ CHUỖI tổ tiên trước và sau một lượt scrollIntoView', async () => {
    const r = await browser.execute(
      async (idx, nguon) => {
        const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
        try {
          // eslint-disable-next-line no-eval
          eval(nguon)
          const hop = document.querySelector('.grid-scroll')
          const o = document.querySelectorAll('[data-col="tgt"]')[idx]
          if (hop === null || o === undefined) return { loi: 'thieu hop hoac o' }
          hop.scrollTop = 0
          await doiKhungHinh()
          // eslint-disable-next-line no-undef
          const truoc = chupToTien()
          o.scrollIntoView({ block: 'nearest', behavior: 'instant' })
          await doiKhungHinh()
          // eslint-disable-next-line no-undef
          const sau = chupToTien()
          return {
            // eslint-disable-next-line no-undef
            nutDaDoi: lechToTien(truoc, sau),
            chuoiToTien: truoc.map((x) => x.ten + (x.id ? '#' + x.id : '') + (x.lop ? '.' + x.lop : '')),
            soNutTrongChuoi: truoc.length,
          }
        } catch (err) {
          return { loiChup: String(err) }
        }
      },
      HANG_DICH,
      CHUP_TO_TIEN,
    )
    in_('Ⓕ đường (a) — nút nào trong chuỗi tổ tiên đã đổi', r)
  })

  it('Ⓖ đường (b) — cùng hàng, gán scrollTop bằng tay, chuỗi tổ tiên có đổi không', async () => {
    const r = await browser.execute(
      async (idx, nguon) => {
        const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
        try {
          // eslint-disable-next-line no-eval
          eval(nguon)
          const hop = document.querySelector('.grid-scroll')
          const o = document.querySelectorAll('[data-col="tgt"]')[idx]
          if (hop === null || o === undefined) return { loi: 'thieu hop hoac o' }
          hop.scrollTop = 0
          await doiKhungHinh()
          // eslint-disable-next-line no-undef
          const truoc = chupToTien()
          const hopBox = hop.getBoundingClientRect()
          const oBox = o.getBoundingClientRect()
          let dich = hop.scrollTop
          if (oBox.top < hopBox.top) dich = hop.scrollTop + (oBox.top - hopBox.top)
          else if (oBox.bottom > hopBox.bottom) dich = hop.scrollTop + (oBox.bottom - hopBox.bottom)
          hop.scrollTop = dich
          await doiKhungHinh()
          // eslint-disable-next-line no-undef
          const sau = chupToTien()
          const oSau = o.getBoundingClientRect()
          const hopSau = hop.getBoundingClientRect()
          return {
            // eslint-disable-next-line no-undef
            nutDaDoi: lechToTien(truoc, sau),
            scrollTopHopSau: hop.scrollTop,
            oNamTronTrongHop: oSau.top >= hopSau.top - 1 && oSau.bottom <= hopSau.bottom + 1,
          }
        } catch (err) {
          return { loiChup: String(err) }
        }
      },
      HANG_DICH,
      CHUP_TO_TIEN,
    )
    in_('Ⓖ đường (b) — nút nào trong chuỗi tổ tiên đã đổi', r)
  })

  it('Ⓗ đường (a) — tác dụng phụ có LẶP LẠI ở lượt gọi thứ hai và thứ ba không', async () => {
    const r = await browser.execute(
      async (idx, nguon) => {
        const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
        try {
          // eslint-disable-next-line no-eval
          eval(nguon)
          const hop = document.querySelector('.grid-scroll')
          const oA = document.querySelectorAll('[data-col="tgt"]')[idx]
          const oB = document.querySelectorAll('[data-col="tgt"]')[2]
          if (hop === null || oA === undefined || oB === undefined) return { loi: 'thieu phan tu' }
          hop.scrollTop = 0
          await doiKhungHinh()

          const luot = []
          // Ba lượt xen kẽ xa/gần — đúng hình dạng của một người bấm phím điều hướng liên tục.
          for (const o of [oA, oB, oA, oB]) {
            // eslint-disable-next-line no-undef
            const t = chupToTien()
            o.scrollIntoView({ block: 'nearest', behavior: 'instant' })
            // eslint-disable-next-line no-await-in-loop
            await doiKhungHinh()
            // eslint-disable-next-line no-undef
            const s = chupToTien()
            // eslint-disable-next-line no-undef
            luot.push({ topHopTruoc: t[0].top, topHopSau: s[0].top, nutDaDoi: lechToTien(t, s) })
          }
          return { luot }
        } catch (err) {
          return { loiChup: String(err) }
        }
      },
      HANG_DICH,
      CHUP_TO_TIEN,
    )
    in_('Ⓗ bốn lượt liên tiếp — tác dụng phụ một lần hay mỗi lần', r)
  })
})
