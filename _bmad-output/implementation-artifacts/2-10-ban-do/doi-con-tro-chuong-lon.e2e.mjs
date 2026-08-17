/**
 * Bàn đo Story 2.10 · **Task 1.4** — ĐO VÀ GHI SỐ. **KHÔNG tự chấm NFR2.**
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VAI CỦA TỆP NÀY, VÀ CÁI NÓ KHÔNG ĐƯỢC LÀM
 * ═════════════════════════════════════════════════════════════════════════════════
 * Cạm bẫy ④ của story: một lượt **đổi con trỏ** trên 9.850 câu tốn **706–770 ms** (đo ở Story
 * 2.5b trên WKWebView thật), trong khi trần một khung hình của NFR2 là **50 ms** — tức đường
 * nóng này đang vượt trần **~15 lần**, và Story 2.10 thêm **hai** lệnh nữa đi đúng đường đó.
 *
 * 🔴 **CHỦ CỦA NFR2 LÀ STORY 2.4** *(nó sở hữu bộ đo NFR2/NFR18 — dựng bộ đo thứ hai là dựng
 * nguồn sự thật thứ hai)*. Tệp này vì thế làm **đúng hai việc**:
 *   ① ghi một con số **đối chiếu được** với mốc 706–770 ms, để biết story này có làm nó **tệ
 *     hơn** không;
 *   ② ghi **điều kiện đo** đủ chi tiết để con số đọc lại được sau nhiều tháng.
 *
 * ⚠️ Nó **không** kết luận *"đạt"* hay *"không đạt"*, và nó **không** vá. Cả hai đều thuộc 2.4.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * ⚠️ HAI GIỚI HẠN CỦA CHÍNH PHÉP ĐO NÀY — ghi ra thay vì để người sau tự phát hiện
 * ═════════════════════════════════════════════════════════════════════════════════
 * ① **Chương ở đây là chương DỰNG, không phải "Chương lớn nhất có thật".** Story viết *"trên
 *    Chương lớn nhất có thật"*; kho không có một Tác phẩm thật cố định để mọi lượt chạy dùng
 *    chung, nên bàn đo dựng một Chương **9.850 câu** — đúng con số mà mốc 706–770 ms được đo
 *    trên đó. Chọn cùng số hàng là điều kiện để hai con số **so sánh được**; một con số đo trên
 *    số hàng khác là một con số không đối chiếu được với mốc, tức vô dụng cho câu hỏi đang hỏi.
 * ② **Cửa sổ debug (`--features wdio`) không phải bản phát hành.** `[profile.release]` của kho
 *    đóng băng `opt-level`/`lto`, và bản debug chạy chậm hơn ở tầng Rust. Vế đo ở đây là **tầng
 *    webview** *(Vue tính lại `ruleById`)* nên chênh lệch đó ảnh hưởng ít — nhưng "ít" không
 *    phải "không", và mốc 706–770 ms cũng đo trên cùng loại cửa sổ.
 *
 *     TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-10-ban-do/doi-con-tro-chuong-lon.e2e.mjs
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'
import { doiChuongVaKiemDanhTinh, in_ } from './danh-tinh-phien.mjs'

/** 🔴 Khớp **đúng** số hàng của mốc 706–770 ms (Story 2.5b) — xem giới hạn ① ở đầu tệp. */
const SO_CAU = 9_850
const CAU = (i) => `第${i}句。`
const VAN_BAN = Array.from({ length: SO_CAU }, (_, i) => CAU(i + 1)).join('')
const CAU_DAU = CAU(1)

/**
 * Đo **một** lượt đổi con trỏ, và đo nó bằng hai thước đặt cạnh nhau.
 *
 * 🔴 Hai thước vì chúng trả lời hai câu khác nhau, và trộn chúng là cách đọc sai cả hai:
 *   • `khungHinhDaiNhat` — mệnh đề của **NFR2** *(không khung hình nào vượt 50 ms)*. Đo bằng
 *     khoảng cách giữa hai `requestAnimationFrame` liên tiếp trong lúc lượt đổi diễn ra.
 *   • `toiKhiVachDoi` — thời gian tới khi **người dùng thấy** vạch lề dời chỗ. Đây là con số
 *     đối chiếu với mốc 706–770 ms của 2.5b.
 */
const DO_MOT_LUOT = `
async function doMotLuot(idxDich) {
  const oDich = document.querySelectorAll('[data-col="tgt"]')[idxDich]
  if (oDich === undefined) return { loi: 'khong co hang dich' }
  const idDich = oDich.getAttribute('data-segment-id')
  // 🔴 Vạch lề là một CLASS ở cột riêng, không một \`data-\` attribute trên ô — xem khối lý do
  // của \`VACH_PRIMARY\` ở \`danh-tinh-phien.mjs\`. Một phép hỏi \`hasAttribute('data-caret')\`
  // trả \`false\` ở MỌI trạng thái, tức thước sẽ báo "không bao giờ dời" và con số vô nghĩa.
  const oVachDich = document.querySelectorAll('.col-rule .cell-rule')[idxDich]
  if (oVachDich === undefined) return { loi: 'khong co o vach cho hang dich' }
  const daToi = () => oVachDich.querySelector('.rule-primary') !== null

  // Ghi mốc mọi khung hình cho tới khi vạch lề đã ở hàng đích (hoặc hết trần).
  const moc = []
  let xong = null
  const bd = performance.now()
  const dem = (t) => {
    moc.push(t)
    if (daToi() && xong === null) xong = performance.now()
    if (xong === null && performance.now() - bd < 5000) requestAnimationFrame(dem)
    else if (xong !== null && moc.length < 4) requestAnimationFrame(dem)
  }
  requestAnimationFrame(dem)

  // Lượt đổi con trỏ THẬT: đặt vùng chọn vào ô đích ⇒ \`selectionchange\` ⇒ \`setEditorCaret\`.
  // Đây là đúng đường mà cả chuột lẫn ba lệnh điều hướng của story này đi qua.
  oDich.focus()
  const sel = window.getSelection()
  sel.removeAllRanges()
  const r = document.createRange()
  r.setStart(oDich, 0)
  r.collapse(true)
  sel.addRange(r)

  await new Promise((res) => setTimeout(res, 5200))

  let daiNhat = 0
  for (let i = 1; i < moc.length; i += 1) {
    const d = moc[i] - moc[i - 1]
    if (d > daiNhat) daiNhat = d
  }
  return {
    idDich: idDich,
    soKhungHinhLayMau: moc.length,
    khungHinhDaiNhat_ms: Math.round(daiNhat),
    toiKhiVachDoi_ms: xong === null ? null : Math.round(xong - bd),
    vachDaToiHangDich: daToi(),
  }
}
`

describe('Bàn đo 2.10 — một lượt đổi con trỏ trên Chương 9.850 câu', function () {
  // Lượt tạo Tác phẩm phải tách 9.850 câu rồi ghi xuống SQLite — trần 120 s của
  // `mochaOpts` là quá ngắn, và một lượt trượt vì trần sẽ đội lốt một khuyết tật sản phẩm.
  this.timeout(600_000)

  before(async () => {
    await openWorkspaceWithWork('Bàn đo 2.10 — Chương lớn', VAN_BAN)
    in_('Task 1.1 · danh tính phiên', await doiChuongVaKiemDanhTinh(SO_CAU, CAU_DAU))
  })

  it('Ⓐ môi trường — Task 1.5, ghi TRƯỚC số để số đọc lại được', async () => {
    const r = await browser.execute(() => ({
      userAgent: navigator.userAgent,
      soHang: document.querySelectorAll('[data-col="tgt"]').length,
      // Số phần tử DOM là thứ giải thích con số phía sau — `ruleById` chạy trên toàn danh sách.
      soPhanTuTrongLuoi: document.querySelectorAll('.grid-scroll *').length,
      kichThuocCuaSo: { w: window.innerWidth, h: window.innerHeight },
      devicePixelRatio: window.devicePixelRatio,
    }))
    in_('Ⓐ môi trường của lượt đo', r)
  })

  it('Ⓑ 🔴 SỐ CHÍNH — ba lượt đổi con trỏ liên tiếp, đối chiếu mốc 706–770 ms của 2.5b', async () => {
    const ketQua = []
    for (const idx of [9_000, 500, 5_000]) {
      // eslint-disable-next-line no-await-in-loop
      const r = await browser.execute(
        async (i, nguon) => {
          try {
            // eslint-disable-next-line no-eval
            eval(nguon)
            // eslint-disable-next-line no-undef
            return await doMotLuot(i)
          } catch (err) {
            return { loiChup: String(err) }
          }
        },
        idx,
        DO_MOT_LUOT,
      )
      ketQua.push({ hang: idx, ...r })
    }
    in_('Ⓑ ba lượt đổi con trỏ — GHI SỐ, không chấm đạt (chủ NFR2 là Story 2.4)', {
      mocDoiChieu_2_5b: '706–770 ms trên 9.850 câu · trần một khung hình NFR2 = 50 ms',
      ketQua,
    })
  })

  it('Ⓒ đối chứng — một lượt CUỘN trên cùng Chương, để tách chi phí cuộn khỏi chi phí đổi caret', async () => {
    // 🔴 Không có ca này thì số ở Ⓑ không tách được: nếu chính lượt **cuộn** tới hàng 9.000
    //    đã tốn hàng trăm ms thì Task 4 mới là chỗ trả giá, không phải `ruleById`.
    const r = await browser.execute(async (idx) => {
      const doiKhungHinh = () => new Promise((res) => requestAnimationFrame(res))
      try {
        const hop = document.querySelector('.grid-scroll')
        const o = document.querySelectorAll('[data-col="tgt"]')[idx]
        if (hop === null || o === undefined) return { loi: 'thieu hop hoac o' }
        hop.scrollTop = 0
        await doiKhungHinh()

        const moc = []
        let dung = false
        const dem = (t) => {
          moc.push(t)
          if (!dung) requestAnimationFrame(dem)
        }
        requestAnimationFrame(dem)

        const bd = performance.now()
        o.scrollIntoView({ block: 'nearest', behavior: 'instant' })
        const sau = performance.now()
        await new Promise((res) => setTimeout(res, 500))
        dung = true

        let daiNhat = 0
        for (let i = 1; i < moc.length; i += 1) {
          const d = moc[i] - moc[i - 1]
          if (d > daiNhat) daiNhat = d
        }
        return {
          loiGoiScrollIntoView_ms: Math.round((sau - bd) * 100) / 100,
          khungHinhDaiNhat_ms: Math.round(daiNhat),
          soKhungHinhLayMau: moc.length,
          scrollTopSau: hop.scrollTop,
        }
      } catch (err) {
        return { loiChup: String(err) }
      }
    }, 9_000)
    in_('Ⓒ chi phí của MỘT lượt cuộn tới hàng 9.000 (tách khỏi chi phí đổi caret)', r)
  })
})
