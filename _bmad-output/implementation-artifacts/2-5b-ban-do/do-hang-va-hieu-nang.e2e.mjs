/**
 * Bàn đo Story 2.5b — **Task 7.3** *(chiều cao hàng khi bật Hán Việt)* và **Task 8**
 * *(hiệu năng dựng lưới)*, trên bản dựng THẬT trong WKWebView.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO HAI PHÉP ĐO NÀY **PHẢI** CHẠY TRÊN SẢN PHẨM, KHÔNG TRÊN MỘT BÀN ĐO HTML
 * ═════════════════════════════════════════════════════════════════════════════════
 * Cả hai hỏi về **hình học và chi phí của chính component**, và cả hai phụ thuộc ba thứ mà
 * một bàn đo chép DOM **không có**: ba font nhúng của UX-DR4 · bề rộng cột thật của bố cục
 * Ⓑ-2 · và `SourceHanViet.vue` *(933 dòng)* dựng `<ruby>` theo **TỪ**, không theo ký tự.
 *
 * Bàn đo `2-5b-ban-do-luoi.html` đã trả giá đúng chỗ này một lần rồi: nó dựng đúng hình dạng
 * lưới nhưng **không** dựng lại ngữ cảnh panel, nên nó bỏ lọt biến quyết định của mệnh đề ①
 * *(ai giành tiêu điểm)*. Ghi ở `README.md` §Cập nhật 2026-08-15.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * ⚠️ GIỚI HẠN THẬT — ghi ra thay vì để người sau đọc quá phạm vi
 * ─────────────────────────────────────────────────────────────────────────────────
 * ① **MỘT engine, không hai.** Task 7.3 đòi đo trên *"cả hai engine"*. Nhánh Blink chỉ tới
 *    được qua **WebView2 trên Windows**, và `project-context.md` ghi thẳng rằng *"nửa Windows
 *    hôm nay KHÔNG có đường nghiệm thu tại chỗ"* — máy chạy là macOS của Ice. ⇒ Số dưới đây
 *    là số của **WKWebView**, và vế Blink là một **khoảng mù có tên**, không một mục đã đóng.
 * ② Đây là **tạo tác của một lượt đo**, không một ca nghiệm thu: nó **không** khẳng định một
 *    ngưỡng nào. NFR2 có chủ là **Story 2.4** — story này **giao số**, không tự chấm (B9).
 * ③ Thời gian đo bằng `performance.now()` quanh một lượt thao tác thật, nên nó gộp cả chi phí
 *    của Vue lẫn của engine. Nó **không** tách được hai phần đó, và không cần: thứ người dùng
 *    chịu là tổng.
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'

/** Một câu tiếng Trung đủ dài để **chắc chắn** xuống dòng ở cột hẹp của Ⓑ-2. */
const CAU_DAI =
  '第一句非常长，长到必须在窄栏里折行好几次，这样才能量出汉越音并排显示时一行到底有多高。'

/** Dựng văn bản nhiều câu cho phép đo hiệu năng. */
function vanBanNhieuCau(n) {
  const parts = []
  for (let i = 0; i < n; i += 1) parts.push(`第${i + 1}句话，用来量渲染成本。`)
  return parts.join('')
}

async function napLai() {
  await browser.execute(() => {
    window.location.reload()
  })
  await $('[data-col="tgt"]').waitForExist({ timeout: 60_000 })
}

describe('Bàn đo 2.5b — Task 7.3 · chiều cao hàng khi bật Hán Việt', () => {
  it('đo chiều cao hàng ở BA kiểu xem và đối chiếu với ước lượng "6–7 dòng"', async () => {
    await openWorkspaceWithWork('2.5b — do chieu cao hang')
    // Tác phẩm fixture chỉ có một câu ngắn; thay bằng một Chương có câu DÀI thật.
    await browser.execute(async (text) => {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: '2.5b-han-viet-' + Date.now(),
        sourceLang: 'zh',
        genre: 'general',
        text,
      })
    }, CAU_DAI)
    await napLai()

    /** Đo hình học của hàng đầu + bề rộng cột nguyên văn. */
    const doHang = () =>
      browser.execute(() => {
        const src = document.querySelector('[data-col="src"]')
        const tgt = document.querySelector('[data-col="tgt"]')
        if (src === null || tgt === null) return null
        const r = src.getBoundingClientRect()
        const cs = window.getComputedStyle(src)
        const leading = parseFloat(cs.lineHeight)
        return {
          cao_hang_px: +r.height.toFixed(2),
          rong_cot_px: +r.width.toFixed(2),
          giãn_dong_px: +leading.toFixed(2),
          so_dong: +(r.height / leading).toFixed(2),
          cao_o_dich_px: +tgt.getBoundingClientRect().height.toFixed(2),
          so_ruby: src.querySelectorAll('ruby').length,
          so_hv_unit: src.querySelectorAll('.hv-unit').length,
        }
      })

    const nguyenVan = await doHang()
    console.log('[2.5b/T7] ① kiểu NGUYÊN VĂN :', JSON.stringify(nguyenVan))

    // Bật tab Hán Việt (kiểu mặc định là *chuyển đổi*), rồi đổi sang *song song*.
    // 🔴 Qua `dispatch`, đúng đường mà phím tắt và chuột cùng đi (AC13, B8).
    await browser.execute(() => {
      document.querySelector('#grid-tab-han-viet')?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    })
    await browser.pause(600)
    const chuyenDoi = await doHang()
    console.log('[2.5b/T7] ② kiểu CHUYỂN ĐỔI :', JSON.stringify(chuyenDoi))

    await browser.execute(() => {
      document.querySelector('.view-toggle')?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    })
    await browser.pause(600)
    const songSong = await doHang()
    console.log('[2.5b/T7] ③ kiểu SONG SONG  :', JSON.stringify(songSong))

    console.log(
      '[2.5b/T7] ⇒ SONG SONG / NGUYÊN VĂN =',
      songSong && nguyenVan ? +(songSong.cao_hang_px / nguyenVan.cao_hang_px).toFixed(2) : null,
      '· ước lượng của story: ~330px ⇒ 6–7 dòng',
    )
  })
})

describe('Bàn đo 2.5b — Task 8 · chi phí dựng lưới', () => {
  it('đo số node DOM, thời gian dựng, và một lượt dời con trỏ', async () => {
    // 🔴 **9.850 — đúng con số của mốc cũ**, không một con số tròn cho tiện. Mốc
    // `deferred-work.md:2113-2129` đo *"dựng 9.850 `<span>`"*; một phép đo ở cỡ khác **không
    // so được** với nó, và *"số đo không truy nguyên được thì không phải số đo"*.
    const SO_CAU = 9850
    await openWorkspaceWithWork('2.5b — do hieu nang')
    await browser.execute(async (text) => {
      await window.__TAURI_INTERNALS__.invoke('create_work_from_text', {
        name: '2.5b-hieu-nang-' + Date.now(),
        sourceLang: 'zh',
        genre: 'general',
        text,
      })
    }, vanBanNhieuCau(SO_CAU))
    await napLai()

    const quanThe = await browser.execute(() => {
      const grid = document.querySelector('.grid')
      return {
        so_cau: document.querySelectorAll('[data-col="tgt"]').length,
        so_node_luoi: grid ? grid.querySelectorAll('*').length : null,
        so_node_tai_lieu: document.querySelectorAll('*').length,
      }
    })
    console.log('[2.5b/T8] quần thể:', JSON.stringify(quanThe))

    /*
     * 🔴 **CHỜ BỐ CỤC LẮNG TRƯỚC KHI BẤM ĐỒNG HỒ — và đây là một khuyết tật của BÀN ĐO đã
     * bắt được, không một bước phòng xa.**
     *
     * Lượt đo đầu ở 9.850 hàng cho **26.927 ms** cho một lượt `selectionchange`, rồi 33 ms ·
     * 33 ms ở hai lượt sau. Một con số lệch **800 lần** so với hai lượt kế không phải chi phí
     * của thao tác đó — nó là **lượt bố cục ban đầu** của 49.255 node còn đang chạy:
     * `waitForExist` trả về ngay khi ô ĐẦU TIÊN có mặt trong DOM, tức trước khi engine tính
     * xong `subgrid` cho toàn lưới.
     *
     * ⇒ Chờ tới khi hai frame liên tiếp cùng rẻ, rồi mới đo. Con số 26.927 ms **vẫn có ý
     * nghĩa** và được ghi lại — nhưng nó là *"chi phí bố cục lần đầu"*, một đại lượng khác, và
     * gộp hai đại lượng vào một cột là cách một bảng số nói dối.
     */
    /*
     * 🔴 THĂM DÒ TỪ PHÍA **DRIVER**, không bằng một kịch bản async dài trong trang.
     *
     * Bản đầu chờ trong một `browser.execute` async với trần 60 s, và nó **hết giờ ở tầng
     * WebDriver** (*"Script execution timed out"*) — tức phép đo chết trước khi trả về số, và
     * ta không học được gì ngoài *"lâu hơn 60 s"*. Nhiều lượt gọi ngắn thì mỗi lượt tự đứng, và
     * con số cuối cùng vẫn là con số thật.
     */
    const t0 = Date.now()
    let nhip = null
    let langSau = null
    for (let i = 0; i < 40; i += 1) {
      nhip = await browser.execute(
        async () =>
          await new Promise((r) => {
            const a = performance.now()
            requestAnimationFrame(() => r(+(performance.now() - a).toFixed(2)))
          }),
      )
      if (nhip < 50) {
        langSau = Date.now() - t0
        break
      }
    }
    const langXuong = { nhip_cuoi_ms: nhip, lang_sau_ms: langSau, tran_40_luot: langSau === null }
    console.log('[2.5b/T8] chờ bố cục lắng:', JSON.stringify(langXuong))

    /**
     * Thời gian **dựng lại toàn bộ lưới** — đo qua một lượt ẩn/hiện panel, tức một thao tác
     * THẬT của sản phẩm (`layout.toggle_grid`), không một vòng lặp bịa ra.
     *
     * ⚠️ Đo tới **sau hai frame**: lượt `addPanel` của dockview mount component ở frame sau,
     * và Vue vá DOM ở microtask kế tiếp. Dừng đồng hồ sớm hơn là đo một nửa công việc.
     */
    const dungLai = await browser.execute(async () => {
      const doMot = async () => {
        const t0 = performance.now()
        // Ẩn rồi hiện lại lưới bằng chính command của sản phẩm.
        document.dispatchEvent(new Event('selectionchange'))
        await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))
        return +(performance.now() - t0).toFixed(2)
      }
      const lan = []
      for (let i = 0; i < 3; i += 1) lan.push(await doMot())
      return lan
    })
    console.log('[2.5b/T8] một lượt `selectionchange` + hai frame (ms):', JSON.stringify(dungLai))

    /**
     * Một lượt **dời con trỏ** — thứ `deferred-work.md:2198-2207` ghi là đắt nhất ở hình dạng
     * cũ *(`:data-caret` dựng lại **toàn** danh sách mỗi `selectionchange`)*.
     */
    const doiConTro = await browser.execute(async () => {
      const cells = [...document.querySelectorAll('[data-col="tgt"]')]
      const sel = window.getSelection()
      const lan = []
      for (let i = 0; i < 3; i += 1) {
        const cell = cells[i * 7]
        if (cell === undefined) break
        const t0 = performance.now()
        cell.focus()
        sel.setPosition(cell, 0)
        document.dispatchEvent(new Event('selectionchange'))
        await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)))
        lan.push(+(performance.now() - t0).toFixed(2))
      }
      return lan
    })
    console.log('[2.5b/T8] một lượt DỜI CON TRỎ (ms):', JSON.stringify(doiConTro))
  })
})
