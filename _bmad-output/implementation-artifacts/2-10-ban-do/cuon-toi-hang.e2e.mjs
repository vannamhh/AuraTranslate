/**
 * Bàn đo Story 2.10 · **Task 1.2** — CHẶN Quyết định #7 và Task 4.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * CÂU HỎI TRUNG TÂM, VÀ VÌ SAO KHÔNG ĐƯỜNG NÀO KHÁC TRẢ LỜI ĐƯỢC
 * ═════════════════════════════════════════════════════════════════════════════════
 * AC8 đòi *"vùng nhìn cuộn tới nó **tức thì**, không có hiệu ứng cuộn"*. Hai đường ứng viên:
 *
 *   (a) `scrollIntoView({ block: 'nearest', behavior: 'instant' })` trên **một ô** của hàng đích
 *   (b) tính `scrollTop` bằng tay từ `getBoundingClientRect()`
 *
 * 🔴 **`happy-dom` không trả lời được, và không phải vì nó thiếu một API — vì nó KHÔNG BỐ CỤC.**
 * Mọi `getBoundingClientRect()` ở đó trả số 0. Một ca vitest cho hai đường này sẽ **xanh trên cả
 * hai** và không phân biệt được gì.
 *
 * ⚠️ **Và câu hỏi có một vế mà kho này chưa ai đo:** hàng trong lưới **không phải một phần tử
 * DOM**. `GridPanel.vue` dựng năm cột, mỗi cột là một `subgrid` chiếm trọn tập track hàng của
 * cha (`:1544`), và hộp cuộn là `.grid-scroll` bọc **cả năm** (`:1518-1528`). ⇒ Gọi
 * `scrollIntoView` trên một ô của cột 4 có cuộn đúng **track hàng** không, hay nó cuộn theo
 * **hộp con** của chính cột đó? `subgrid` là chỗ ba engine đã bất đồng ít nhất một lần trong
 * epic này — nên câu này phải hỏi engine, không hỏi một bảng tương thích.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * NĂM PHÉP ĐO, và mỗi phép có một đối chứng để nó không tự xác nhận mình
 * ═════════════════════════════════════════════════════════════════════════════════
 *   Ⓐ Hộp cuộn có **tràn** thật không — nếu không tràn thì mọi số sau vô nghĩa (đối chứng
 *     tiền đề: một bàn đo cuộn trên một lưới không cuộn được là một bàn đo tự lừa).
 *   Ⓑ Đường (a) trên hàng **xa dưới**: `scrollTop` đổi bao nhiêu, hàng đích có nằm TRỌN trong
 *     hộp không, và **năm cột có cùng một `top`** không *(vế `subgrid`)*.
 *   Ⓒ **Có hiệu ứng không** — lấy mẫu `scrollTop` qua 12 khung hình liên tiếp. Một lượt cuộn
 *     tức thì cho **một** giá trị suốt 12 mẫu; một lượt có hiệu ứng cho một dãy tăng dần.
 *   Ⓓ Gọi **lại** khi hàng đã trong vùng nhìn ⇒ `block:'nearest'` phải **không cuộn gì**. Đây
 *     là mệnh đề mà một phím bấm liên tục sống nhờ vào.
 *   Ⓔ Đối chứng đường (b) trên **cùng** hàng: `scrollTop` tính tay có ra cùng con số không.
 *
 * ⚠️ Cộng một phép đo mà story **không** yêu cầu nhưng đường (a) sống chết vì nó: `scrollIntoView`
 * cuộn **mọi tổ tiên cuộn được**, không chỉ `.grid-scroll`. Nếu nó cũng dịch `documentElement`
 * hay khung dockview thì đường (a) làm xô cả bố cục — một tác dụng phụ không AC nào nêu và
 * không cổng nào bắt.
 *
 *     TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-10-ban-do/cuon-toi-hang.e2e.mjs
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { doiChuongVaKiemDanhTinh, in_ } from './danh-tinh-phien.mjs'

/**
 * 60 câu — đủ để `.grid-scroll` tràn trên mọi kích thước cửa sổ hợp lý, và đủ nhỏ để lượt tạo
 * Tác phẩm không ăn hết trần `mochaOpts.timeout` 120 s.
 *
 * ⚠️ Mỗi câu mang **số thứ tự của chính nó** — không phải để đẹp: nó là thứ làm vế Ⓑ của phép
 * kiểm danh tính có nghĩa *(một Chương khác cùng độ dài vẫn cho câu đầu khác)*, và nó làm mọi
 * khối JSON dưới đây tự nói ra nó đang đứng ở hàng nào.
 *
 * ⚠️ `sourceLang: 'zh'` ⇒ bộ tách nhìn `。`, không nhìn dấu chấm tiếng Anh
 * (`workspace.mjs:51-52`).
 */
const SO_CAU = 60
const CAU = (i) => `第${i}句子内容在这里。`
const VAN_BAN = Array.from({ length: SO_CAU }, (_, i) => CAU(i + 1)).join('')
const CAU_DAU = CAU(1)

/** Hàng đích: xa dưới đáy vùng nhìn ở mọi cửa sổ hợp lý, nhưng không phải hàng cuối. */
const HANG_DICH = 50

describe('Bàn đo 2.10 — cuộn tới một hàng của lưới subgrid', () => {
  before(async () => {
    await openWorkspaceWithWork('Bàn đo 2.10 — cuộn tới hàng', VAN_BAN)
    in_('Task 1.1 · danh tính phiên', await doiChuongVaKiemDanhTinh(SO_CAU, CAU_DAU))
  })

  it('Ⓐ hộp cuộn có TRÀN thật không — tiền đề của mọi số phía sau', async () => {
    const r = await browser.execute(() => {
      const hop = document.querySelector('.grid-scroll')
      if (hop === null) return { loi: 'khong tim thay .grid-scroll' }
      const o = document.querySelectorAll('[data-col="tgt"]')[0]
      return {
        scrollHeight: hop.scrollHeight,
        clientHeight: hop.clientHeight,
        traoRa: hop.scrollHeight - hop.clientHeight,
        coTran: hop.scrollHeight > hop.clientHeight,
        scrollTopBanDau: hop.scrollTop,
        overflowY: getComputedStyle(hop).overflowY,
        // 🔴 Giá trị CSS `scroll-behavior` đang hiệu lực trên hộp cuộn. Quyết định #7 dựa
        // trên mệnh đề "không luật nào canh giá trị này" — đây là lượt đọc nó từ engine
        // thay vì từ một `grep`.
        scrollBehaviorHop: getComputedStyle(hop).scrollBehavior,
        scrollBehaviorRoot: getComputedStyle(document.documentElement).scrollBehavior,
        caoMotHang: o === undefined ? null : Math.round(o.getBoundingClientRect().height),
      }
    })
    in_('Ⓐ hộp cuộn', r)
  })

  it('Ⓑ+Ⓒ đường (a) scrollIntoView({block:"nearest", behavior:"instant"}) tới hàng xa', async () => {
    const r = await browser.execute(async (idx) => {
      const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
      try {
        const hop = document.querySelector('.grid-scroll')
        const o = document.querySelectorAll('[data-col="tgt"]')[idx]
        if (hop === null || o === undefined) return { loi: 'thieu hop cuon hoac o dich' }

        // Về đầu trước, để lượt đo bắt đầu từ một trạng thái biết trước.
        hop.scrollTop = 0
        await doiKhungHinh()

        const hopTruoc = hop.getBoundingClientRect()
        const oTruoc = o.getBoundingClientRect()
        const truoc = {
          scrollTopHop: hop.scrollTop,
          scrollTopRoot: document.scrollingElement === null ? null : document.scrollingElement.scrollTop,
          topO: Math.round(oTruoc.top),
          topHop: Math.round(hopTruoc.top),
          oTrongVungNhin: oTruoc.top >= hopTruoc.top && oTruoc.bottom <= hopTruoc.bottom,
        }

        o.scrollIntoView({ block: 'nearest', behavior: 'instant' })

        // ── Ⓒ 12 mẫu liên tiếp. Cuộn tức thì ⇒ 12 giá trị GIỐNG NHAU ────────────────
        const mau = [hop.scrollTop]
        for (let i = 0; i < 12; i += 1) {
          await doiKhungHinh()
          mau.push(hop.scrollTop)
        }

        const hopSau = hop.getBoundingClientRect()
        const oSau = o.getBoundingClientRect()

        // ── Vế `subgrid`: năm cột của CÙNG hàng có cùng `top` không ─────────────────
        const cotCungHang = ['src', 'tgt']
          .map((c) => {
            const el = document.querySelectorAll(`[data-col="${c}"]`)[idx]
            return el === undefined ? null : { cot: c, top: Math.round(el.getBoundingClientRect().top) }
          })
          .filter((x) => x !== null)

        return {
          truoc,
          sau: {
            scrollTopHop: hop.scrollTop,
            scrollTopRoot:
              document.scrollingElement === null ? null : document.scrollingElement.scrollTop,
            topO: Math.round(oSau.top),
            topHop: Math.round(hopSau.top),
            // 🔴 Mệnh đề của AC8: hàng đích nằm TRỌN trong hộp.
            oNamTronTrongHop: oSau.top >= hopSau.top - 1 && oSau.bottom <= hopSau.bottom + 1,
          },
          // 🔴 Tác dụng phụ mà story không nêu: hộp cha có bị dịch theo không.
          hopCoTuDichKhong: Math.round(hopSau.top) !== Math.round(hopTruoc.top),
          cotCungHang,
          namCotCungTop: new Set(cotCungHang.map((x) => x.top)).size === 1,
          // ── Ⓒ ────────────────────────────────────────────────────────────────────
          mauScrollTop: mau,
          soGiaTriKhacNhau: new Set(mau).size,
          coHieuUng: new Set(mau).size > 1,
        }
      } catch (err) {
        return { loiChup: String(err) }
      }
    }, HANG_DICH)
    in_('Ⓑ+Ⓒ scrollIntoView tới hàng 50 — vị trí, subgrid, và 12 mẫu khung hình', r)
  })

  it('Ⓓ gọi LẠI khi hàng đã trong vùng nhìn ⇒ block:"nearest" phải đứng yên', async () => {
    const r = await browser.execute(async (idx) => {
      const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
      try {
        const hop = document.querySelector('.grid-scroll')
        const o = document.querySelectorAll('[data-col="tgt"]')[idx]
        if (hop === null || o === undefined) return { loi: 'thieu hop cuon hoac o dich' }

        const truoc = hop.scrollTop
        o.scrollIntoView({ block: 'nearest', behavior: 'instant' })
        await doiKhungHinh()
        const sau1 = hop.scrollTop
        // Gọi ba lượt nữa — mô phỏng đúng ca "giữ phím điều hướng".
        o.scrollIntoView({ block: 'nearest', behavior: 'instant' })
        o.scrollIntoView({ block: 'nearest', behavior: 'instant' })
        o.scrollIntoView({ block: 'nearest', behavior: 'instant' })
        await doiKhungHinh()
        const sau4 = hop.scrollTop

        // ── Đối chứng ÂM: `block:'center'` trên CÙNG ô PHẢI dịch, nếu không thì phép
        //    kiểm trên chỉ đang đo một ô `scrollIntoView` không hoạt động chút nào.
        o.scrollIntoView({ block: 'center', behavior: 'instant' })
        await doiKhungHinh()
        const sauCenter = hop.scrollTop

        return {
          truoc,
          sauMotLuot: sau1,
          sauBonLuot: sau4,
          nearestDungYen: truoc === sau1 && sau1 === sau4,
          doiChungAm_sauBlockCenter: sauCenter,
          doiChungAm_centerCoDich: sauCenter !== sau4,
        }
      } catch (err) {
        return { loiChup: String(err) }
      }
    }, HANG_DICH)
    in_('Ⓓ nearest khi đã trong vùng nhìn + đối chứng âm block:"center"', r)
  })

  it('Ⓔ đối chứng đường (b) — scrollTop tính tay trên CÙNG hàng', async () => {
    const r = await browser.execute(async (idx) => {
      const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
      try {
        const hop = document.querySelector('.grid-scroll')
        const o = document.querySelectorAll('[data-col="tgt"]')[idx]
        if (hop === null || o === undefined) return { loi: 'thieu hop cuon hoac o dich' }

        // ── Đường (a), để lấy số đối chiếu ────────────────────────────────────────
        hop.scrollTop = 0
        await doiKhungHinh()
        o.scrollIntoView({ block: 'nearest', behavior: 'instant' })
        await doiKhungHinh()
        const cuaDuongA = hop.scrollTop

        // ── Đường (b), tính tay đúng ngữ nghĩa `nearest` ──────────────────────────
        hop.scrollTop = 0
        await doiKhungHinh()
        const hopBox = hop.getBoundingClientRect()
        const oBox = o.getBoundingClientRect()
        let dich = hop.scrollTop
        if (oBox.top < hopBox.top) dich = hop.scrollTop + (oBox.top - hopBox.top)
        else if (oBox.bottom > hopBox.bottom) dich = hop.scrollTop + (oBox.bottom - hopBox.bottom)
        hop.scrollTop = dich
        await doiKhungHinh()
        const cuaDuongB = hop.scrollTop

        // Đường (b) có đọc CSS `scroll-behavior` khi gán `scrollTop` không — 12 mẫu.
        const mau = [hop.scrollTop]
        for (let i = 0; i < 12; i += 1) {
          await doiKhungHinh()
          mau.push(hop.scrollTop)
        }

        const oSau = o.getBoundingClientRect()
        const hopSau = hop.getBoundingClientRect()
        return {
          duongA_scrollTop: cuaDuongA,
          duongB_scrollTopTinhTay: Math.round(dich),
          duongB_scrollTopThucTe: cuaDuongB,
          haiDuongCungSo: Math.abs(cuaDuongA - cuaDuongB) <= 1,
          duongB_oNamTronTrongHop: oSau.top >= hopSau.top - 1 && oSau.bottom <= hopSau.bottom + 1,
          duongB_mauScrollTop: mau,
          duongB_coHieuUng: new Set(mau).size > 1,
        }
      } catch (err) {
        return { loiChup: String(err) }
      }
    }, HANG_DICH)
    in_('Ⓔ đối chứng đường (b) scrollTop tính tay', r)
  })
})
