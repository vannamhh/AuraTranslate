/**
 * Bàn đo Story 2.9 — **chỗ cắt ở tab HÁN VIỆT: hỏng thế nào, và hỏng ÂM THẦM hay ồn ào?**
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 CÂU HỎI, VÀ VÌ SAO NÓ KHÔNG TRẢ LỜI ĐƯỢC BẰNG MỘT LƯỢT ĐỌC MÃ
 * ═════════════════════════════════════════════════════════════════════════════════
 * Ice báo 2026-08-17: *"ở phần Hán Việt vẫn chưa thấy điểm cắt, và chưa cắt được"*.
 *
 * Đọc mã cho **hai** nguyên nhân ứng viên, và chúng khác nhau về HẠNG:
 *
 *   Ⓐ **Dấu cắt không vẽ ra** — `GridPanel.vue` dựng ô nguyên văn bằng hai nhánh **loại trừ
 *      nhau**: `v-if="showHanViet"` → `<SourceHanViet>`, `v-else` → văn bản thuần **cộng**
 *      `.cut-mark`. Ở tab Hán Việt nhánh thứ hai **không chạy**, nên không dấu nào tồn tại.
 *      ⇒ Một khuyết tật **hiển thị**. Khó chịu, không nguy hiểm.
 *
 *   Ⓑ 🔴 **Chỗ cắt tính SAI, im lặng** — `sourceCutOffsetOf` đếm **text node** trong ô rồi
 *      trả một chỉ số ký tự của `source_text`. Nhưng ở tab Hán Việt cái nằm trong ô **không
 *      còn là `source_text`**:
 *        · kiểu `switch` — `.hv-word` render `seg.readings`, tức **ÂM Hán Việt**, cộng
 *          `WORD_SEPARATOR`/`WORD_JOINER`. Chữ Hán gốc **không có mặt trên màn hình**.
 *        · kiểu `parallel` — `<ruby>` mỗi TỪ, base mang chữ nguồn, `<rt>` mang âm.
 *          `duoiRt` đã bỏ `<rt>` *(vá ở code review 2.8)*, nhưng còn `WORD_SEPARATOR`.
 *      ⇒ Nếu Ⓑ đúng thì đây là **hỏng dữ liệu im lặng**: `⌘/` cắt đúng chỗ người dùng
 *        KHÔNG bấm, trên dữ liệu mà **AD-5 không cho hoàn tác**, và không cổng nào đỏ.
 *        Đúng lớp lỗi mà `sourceCutOffsetOf` đã sai HAI lần trước đó vì cùng một họ nguyên
 *        nhân *(đếm nhầm node · đếm nhầm đơn vị)*.
 *
 * ⚠️ **Ⓐ và Ⓑ KHÔNG loại trừ nhau**, và Ⓐ **che** Ⓑ: không thấy dấu cắt thì người dùng không
 * biết mình vừa đặt một chỗ cắt sai. Ice báo *"chưa cắt được"* — câu đó **không** phân biệt
 * *"không đặt được chỗ nào"* với *"đặt được nhưng ở chỗ sai rồi Rust từ chối"*.
 *
 * ⇒ Bàn đo này đọc **hình dạng DOM thật** ở cả hai kiểu xem và chạy **chính** phép ánh xạ của
 * sản phẩm trên đó, rồi đối chiếu với `source_text`. Không suy từ template.
 *
 *     TAURI_WEBDRIVER_PORT=4468 npm run test:e2e -- \
 *       --spec _bmad-output/implementation-artifacts/2-9-ban-do/han-viet-cho-cat.e2e.mjs
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'

/** Nguyên văn của bàn đo — chữ Hán có âm, cộng một mẩu KHÔNG-Hán xen giữa. */
const NGUYEN_VAN = '京都春風。'

function in_(nhan, v) {
  console.log(`\n[2.9·hv · ${nhan}] ` + JSON.stringify(v, null, 2))
}

/** Kiểm kê ô nguyên văn đầu: cây con, text node, và `textContent` so với `source_text`. */
async function kiemKe(nhan) {
  return await browser.execute((n) => {
    try {
      const cell = document.querySelector('[data-col="src"]')
      if (cell === null) return { buoc: n, loi: 'khong tim thay o nguyen van' }
      // Mọi text node của ô, kèm cha gần nhất — đủ để đọc ra cái gì đang được đếm.
      const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
      const nodes = []
      let x = walker.nextNode()
      while (x !== null) {
        nodes.push({
          cha: x.parentElement?.tagName ?? null,
          lop: x.parentElement?.className ?? null,
          chu: JSON.stringify(x.data),
          dai: [...x.data].length,
        })
        x = walker.nextNode()
      }
      return {
        buoc: n,
        // 🔴 Dấu cắt có tồn tại trong DOM không — vế Ⓐ.
        soDauCat: cell.querySelectorAll('.cut-mark').length,
        coHanViet: cell.querySelector('.hv-surface') !== null,
        kieuXem: cell.querySelector('.hv-switch')
          ? 'switch'
          : cell.querySelector('.hv-parallel')
            ? 'parallel'
            : null,
        soRuby: cell.querySelectorAll('ruby').length,
        soRt: cell.querySelectorAll('rt').length,
        textContentCuaO: JSON.stringify(cell.textContent),
        soKyTuTextContent: [...cell.textContent].length,
        soTextNode: nodes.length,
        textNode: nodes,
      }
    } catch (err) {
      return { buoc: n, loiChup: String(err) }
    }
  }, nhan)
}

/**
 * 🔴 Chạy **chính** phép ánh xạ của sản phẩm — chép nguyên `sourceCutOffsetOf` vào bàn đo —
 * trên một điểm bấm ở giữa ô, rồi đối chiếu với chỉ số ĐÚNG.
 *
 * ⚠️ Chép nguyên văn, không diễn giải lại: một bản viết gọn sẽ đo một hàm khác với hàm đang
 * chạy trong sản phẩm, và đó đúng là cách một bàn đo nói dối.
 *
 * 🔴 **VÀ MỘT BẢN CHÉP CŨ ĐI — đo được 2026-08-17, ngay trong chính story này.** Sau lượt vá
 * AC9, bàn đo chạy lại vẫn cho **17** và **19** y hệt lượt trước, trong khi DOM đã mang neo
 * *(`neoVao: "src-piece"` ở tab nguyên văn chứng minh điều đó)*. Bản chép ở đây là bản CŨ.
 * ⇒ Bài học: một bàn đo chép hàm sản phẩm phải được cập nhật **cùng lượt** với hàm ấy, nếu
 * không nó báo *"chưa vá"* trên một sản phẩm **đã vá** — cùng họ với *"một con số THẬT, trả
 * lời SAI câu hỏi"* mà `2-5d-ban-do` đã đặt tên.
 */
const ANH_XA = `
function duoiRt(cell, node) {
  let p = node.parentElement
  while (p !== null && cell.contains(p)) {
    if (p.tagName === 'RT') return true
    p = p.parentElement
  }
  return false
}
function demKyTu(s) { return s === null || s === undefined ? 0 : [...s].length }
function neoNguonCua(cell, node) {
  let p = node.nodeType === 1 ? node : node.parentElement
  while (p !== null && cell.contains(p)) {
    if (p.hasAttribute('data-src-start')) return p
    p = p.parentElement
  }
  return null
}
function sourceCutOffsetOf(cell, node, offsetInNode) {
  if (!cell.contains(node)) return null
  if (duoiRt(cell, node)) return null
  const neo = neoNguonCua(cell, node)
  if (neo === null) return null
  const batDau = Number(neo.getAttribute('data-src-start'))
  if (!Number.isFinite(batDau)) return null
  if (neo.hasAttribute('data-src-atomic')) return batDau
  let truoc = 0
  const walker = cell.ownerDocument.createTreeWalker(neo, NodeFilter.SHOW_TEXT)
  let n = walker.nextNode()
  while (n !== null && n !== node) {
    if (!duoiRt(cell, n)) truoc += demKyTu(n.textContent)
    n = walker.nextNode()
  }
  return batDau + truoc + (n === null ? 0 : demKyTu(n.textContent?.slice(0, offsetInNode)))
}
`

async function doChoCat(nhan, nguyenVan) {
  return await browser.execute(
    (n, src, nguon) => {
      try {
        // eslint-disable-next-line no-eval
        eval(nguon)
        const cell = document.querySelector('[data-col="src"]')
        const box = cell.getBoundingClientRect()
        // Bấm ở ~40 % chiều rộng, giữa dòng đầu — cùng khuôn ca e2e của sản phẩm.
        const x = box.left + box.width * 0.4
        const y = box.top + Math.min(20, box.height / 2)
        if (typeof document.caretRangeFromPoint !== 'function') {
          return { buoc: n, loi: 'caretRangeFromPoint vang mat' }
        }
        const r = document.caretRangeFromPoint(x, y)
        if (r === null) return { buoc: n, loi: 'caretRangeFromPoint tra null' }
        // eslint-disable-next-line no-undef
        const cut = sourceCutOffsetOf(cell, r.startContainer, r.startOffset)
        const sc = r.startContainer
        return {
          buoc: n,
          diem: { x: Math.round(x), y: Math.round(y) },
          neoVao: sc.parentElement?.className ?? sc.nodeName,
          neoLaChu: JSON.stringify(sc.nodeType === 3 ? sc.data : null),
          startOffset: r.startOffset,
          // 🔴 Con số sản phẩm SẼ GỬI cho Rust.
          choCat: cut,
          // 🔴 Con số ĐÚNG phải nằm trong [1, len-1] và ứng với chữ người dùng bấm.
          sourceText: JSON.stringify(src),
          doDaiNguon: [...src].length,
          trongBien: cut !== null && cut > 0 && cut < [...src].length,
        }
      } catch (err) {
        return { buoc: n, loiChup: String(err) }
      }
    },
    nhan,
    nguyenVan,
    ANH_XA,
  )
}

describe('Bàn đo 2.9 — chỗ cắt ở tab Hán Việt', () => {
  it('ba trạng thái: nguyên văn · Hán Việt switch · Hán Việt parallel', async () => {
    await openWorkspaceWithWork('Bàn đo 2.9 — Hán Việt', NGUYEN_VAN)
    await browser.execute(() => {
      window.location.reload()
    })
    await $('[data-col="src"]').waitForExist({ timeout: 30_000 })

    in_(
      'danh tính phiên',
      await browser.execute(() => ({
        href: window.location.href,
        soHang: document.querySelectorAll('[data-col="src"]').length,
        userAgent: navigator.userAgent,
      })),
    )

    // ── ⓪ ĐỐI CHỨNG DƯƠNG — tab NGUYÊN VĂN, đường đang chạy tốt ────────────────────
    in_('⓪ tab NGUYÊN VĂN · kiểm kê', await kiemKe('nguyên văn'))
    in_('⓪ tab NGUYÊN VĂN · chỗ cắt', await doChoCat('nguyên văn', NGUYEN_VAN))

    // ── ① Bật tab HÁN VIỆT ────────────────────────────────────────────────────────
    // Đi qua `dispatch` của sản phẩm, không sờ state trực tiếp.
    const bat = await browser.execute(() => {
      try {
        // eslint-disable-next-line no-undef
        const d = window.__AURA_DISPATCH__
        if (typeof d === 'function') {
          d('source.select_tab_han_viet')
          return { qua: 'dispatch' }
        }
        // Không có cửa nào từ ngoài ⇒ bấm chính tab, đúng đường người dùng.
        const tab = document.getElementById('grid-tab-han-viet')
        if (tab !== null) {
          tab.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
          return { qua: 'click tab' }
        }
        return { qua: null, lyDo: 'khong tim thay duong bat tab Han Viet' }
      } catch (err) {
        return { qua: null, loi: String(err) }
      }
    })
    in_('① bật tab Hán Việt', bat)
    await browser.pause(2500)

    in_('① tab HÁN VIỆT · kiểm kê', await kiemKe('hán việt (kiểu mặc định)'))
    in_('① tab HÁN VIỆT · chỗ cắt', await doChoCat('hán việt (kiểu mặc định)', NGUYEN_VAN))

    // ── ② Đổi kiểu xem sang kiểu còn lại ──────────────────────────────────────────
    const doi = await browser.execute(() => {
      try {
        // eslint-disable-next-line no-undef
        const d = window.__AURA_DISPATCH__
        if (typeof d === 'function') {
          d('source.toggle_han_viet_view')
          return { qua: 'dispatch' }
        }
        const nut = document.querySelector('.view-toggle')
        if (nut !== null) {
          nut.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }))
          return { qua: 'click nút' }
        }
        return { qua: null, lyDo: 'khong tim thay duong doi kieu xem' }
      } catch (err) {
        return { qua: null, loi: String(err) }
      }
    })
    in_('② đổi kiểu xem', doi)
    await browser.pause(2500)

    in_('② tab HÁN VIỆT · kiểm kê', await kiemKe('hán việt (kiểu còn lại)'))
    in_('② tab HÁN VIỆT · chỗ cắt', await doChoCat('hán việt (kiểu còn lại)', NGUYEN_VAN))
  })
})
