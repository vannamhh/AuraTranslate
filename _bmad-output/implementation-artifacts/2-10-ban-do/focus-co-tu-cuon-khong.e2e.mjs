/**
 * Bàn đo Story 2.10 · **VÒNG 3** — `focus()` có tự cuộn không, và `cuonToiHang` có phải MÃ CHẾT?
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO VÒNG NÀY TỒN TẠI — một ĐỘT BIẾN không giết được ca nào
 * ═════════════════════════════════════════════════════════════════════════════════
 * `e2e/specs/segment-navigation.e2e.mjs` §Ⓒ khẳng định *"vùng nhìn đã cuộn, và hàng đích nằm
 * trọn trong hộp"* và nó **xanh**. Rồi Task 7.3 gỡ lời gọi `cuonToiHang(target)` khỏi mã sản
 * phẩm và chạy lại: **vẫn 3 passing**.
 *
 * ⇒ Ca Ⓒ **không đo `cuonToiHang`**. Nó đo một thứ khác cũng đang cuộn, và ứng viên hiển nhiên
 * là `target.focus()` ngay dòng trên.
 *
 * 🔴 Đây là lý do luật *"đột biến mã sản phẩm cho mỗi ca mới"* tồn tại. Không có lượt đột biến
 * đó, story sẽ giao một hàm **có thể là mã chết** kèm một ca test tự xưng là canh nó.
 *
 * ⚠️ **VÒNG 3A ĐÃ HỎNG THƯỚC — bỏ số, giữ bài học.** Bản đầu gộp ba phép đo vào một
 * `browser.execute` async có `o.blur()` + nhiều `requestAnimationFrame`, và **cả ba ca đều
 * `Script execution timed out`** *(6 phút 23, 3 failing)*. Không một con số nào ra. Nguyên nhân
 * không được chẩn thêm — thước bị **thay**, không bị vá: bản này bỏ `blur()`, bỏ vòng rAF lồng
 * nhau, và hỏi **một** câu trong **một** lời gọi đồng bộ.
 * ⇒ Vẫn là một lượt sửa **THƯỚC**, không một vòng chẩn đoán sản phẩm bị bác.
 *
 *     TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-10-ban-do/focus-co-tu-cuon-khong.e2e.mjs
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { doiChuongVaKiemDanhTinh, in_ } from './danh-tinh-phien.mjs'

const SO_CAU = 60
const CAU = (i) => `第${i}句子内容在这里。`
const VAN_BAN = Array.from({ length: SO_CAU }, (_, i) => CAU(i + 1)).join('')
const HANG_DICH = 50

/**
 * Một lượt đo, **đồng bộ**. Đặt `scrollTop = 0`, gọi `focus()` theo kiểu được chỉ định, rồi đọc
 * hình học **ngay** — `focus()` cuộn **đồng bộ** *(nó không phải một hoạt ảnh)*, nên không cần
 * chờ khung hình nào, và mọi lượt chờ đã được đo là chỗ thước treo.
 */
async function doMotKieu(idx, preventScroll, dungCongThuc) {
  return browser.execute(
    (i, ps, ct) => {
      const hop = document.querySelector('.grid-scroll')
      const panel = document.querySelector('.panel')
      const o = document.querySelectorAll('[data-col="tgt"]')[i]
      hop.scrollTop = 0
      const truoc = hop.scrollTop

      o.focus(ps ? { preventScroll: true } : undefined)
      const sauFocus = hop.scrollTop

      if (ct) {
        const hb = hop.getBoundingClientRect()
        const ob = o.getBoundingClientRect()
        if (ob.top < hb.top) hop.scrollTop += ob.top - hb.top
        else if (ob.bottom > hb.bottom) hop.scrollTop += ob.bottom - hb.bottom
      }
      const sauCongThuc = hop.scrollTop

      const hb2 = hop.getBoundingClientRect()
      const ob2 = o.getBoundingClientRect()
      return {
        truoc,
        sauFocus,
        sauCongThuc,
        hangNamTronTrongHop: ob2.top >= hb2.top - 1 && ob2.bottom <= hb2.bottom + 1,
        scrollTopPanel: panel === null ? null : panel.scrollTop,
      }
    },
    idx,
    preventScroll,
    dungCongThuc,
  )
}

describe('Bàn đo 2.10 vòng 3 — focus() có tự cuộn không', () => {
  before(async () => {
    await openWorkspaceWithWork('Bàn đo 2.10 v3 — focus tự cuộn', VAN_BAN)
    in_('Task 1.1 · danh tính phiên', await doiChuongVaKiemDanhTinh(SO_CAU, CAU(1)))
  })

  it('Ⓘ `focus()` một mình — có tự cuộn không', async () => {
    in_('Ⓘ focus() một mình', await doMotKieu(HANG_DICH, false, false))
  })

  it('Ⓙ đối chứng — `focus({preventScroll:true})` phải KHÔNG cuộn', async () => {
    in_('Ⓙ focus({preventScroll:true}) — đối chứng ÂM', await doMotKieu(HANG_DICH, true, false))
  })

  it('Ⓚ `focus({preventScroll:true})` + công thức `cuonToiHang`', async () => {
    in_('Ⓚ preventScroll rồi chạy công thức', await doMotKieu(HANG_DICH, true, true))
  })

  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔴 Ⓜ — CA NÀY ĐÃ BỊ GỠ, và câu hỏi của nó ĐÃ CÓ ĐÁP ÁN SẠCH HƠN Ở Ⓘ
   * ═══════════════════════════════════════════════════════════════════════════════
   * Câu hỏi: *"`focus()` cuộn CÓ HIỆU ỨNG không"* — vế mà AC8 nói bằng chữ, và là nửa **đo
   * được** của Task 4.5. Vòng 1 đã đo nó cho `scrollIntoView` và cho công thức tự cài bằng cách
   * lấy 12 mẫu `scrollTop` qua 12 khung hình *(12/12 bằng nhau)*; ca này định làm y vậy cho
   * `focus()`.
   *
   * **HAI lượt dựng thước, hai lượt hỏng, và cả hai chỉ hỏng ở ĐÚNG ca này:**
   *   ① `browser.executeAsync(...)` ⇒ spec chết ngay tại ca đó *(`1 failing`)*. Driver này không
   *      chạy được nó.
   *   ② `browser.execute(async …)` + `await new Promise(requestAnimationFrame)` — **đúng khuôn
   *      đã chạy 12 mẫu ở vòng 1** — ⇒ treo, `4 passing / 1 failing`, riêng ca này hết giờ. Bốn
   *      ca đồng bộ trong cùng tệp thì xanh. Khác biệt duy nhất: vòng lặp rAF chạy **sau một
   *      lượt `focus()`**, không sau một lượt `scrollIntoView`.
   *
   * 🔴 **DỪNG Ở ĐÂY, và không phải vì bỏ cuộc — vì câu hỏi ĐÃ ĐƯỢC TRẢ LỜI bởi một phép đo
   * KHÁC, sạch hơn, đã nằm sẵn ở ca Ⓘ:** Ⓘ đọc `hop.scrollTop` **ĐỒNG BỘ**, ngay dòng sau
   * `o.focus()`, không chờ một khung hình nào — và nó đọc được **1569**, tức **giá trị cuối
   * cùng**. Một lượt cuộn **có hiệu ứng** không thể đã tới đích trong cùng một lượt thực thi
   * đồng bộ; nó sẽ đọc ra ~0 rồi bò dần qua các khung hình.
   * ⇒ `focus()` cuộn **tức thì**. Mệnh đề đóng, bằng một phép đo **mạnh hơn** phép lấy mẫu 12
   *   khung hình chứ không yếu hơn.
   *
   * ⚠️ Ghi lại thay vì xoá, vì hai lý do: ca này **sẽ được nghĩ ra lại** bởi người sau *(nó là
   * phép đo hiển nhiên cho câu hỏi ấy)*, và giới hạn *"rAF sau `focus()` treo trên driver này"*
   * là một dữ kiện về **bộ đo** mà bàn đo kế tiếp sẽ cần.
   */

  it('Ⓛ `focus()` rồi công thức — công thức có gán thêm gì không (ca CHỒNG NHAU)', async () => {
    // 🔴 Đây là hình dạng **THẬT** của mã sản phẩm hôm nay: `focus()` rồi `cuonToiHang()`.
    //    Nếu `sauCongThuc === sauFocus` thì công thức **không làm gì** ở ca này.
    in_('Ⓛ focus() rồi công thức — đúng hình dạng mã sản phẩm', await doMotKieu(HANG_DICH, false, true))
  })
})
