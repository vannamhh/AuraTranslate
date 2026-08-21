/**
 * Bàn đo Story 1.19 — AC11: `Escape` đóng lớp phủ Attribution và **trả tiêu điểm về chỗ cũ**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY RA ĐỜI — nó là một PHÉP ĐO, không một lượt mở rộng bao phủ
 * ═════════════════════════════════════════════════════════════════════════════════
 * Story 1.22 C2 đổi bàn đo sang chuột THẬT (Actions API), và `shortcuts-focus` lập tức ĐỎ.
 * `AttributionOverlay.vue` mang **khuôn giống hệt** `ShortcutsOverlay.vue` — chính doc-comment
 * của tệp sau ghi *"khuôn và lý lẽ chép từ `AttributionOverlay.vue`, cả hai vế"* — nên nó nhận
 * cùng một bản vá.
 *
 * 🔴 Nhưng "giống khuôn" là một suy luận, không một phép đo. Ice chốt 2026-08-11: dựng spec
 * này để **BIẾT** thay vì suy. Và phép đo đã trả công: hai lớp phủ hoá ra **không** hỏng cùng
 * một kiểu, vì nút mở của chúng ngồi ở hai chỗ khác nhau.
 *
 * ⚠️ **Bản vá cho `ShortcutsOverlay` KHÔNG chuyển được sang đây, và đó là phát hiện của spec
 * này.** Nút mở phím tắt nằm ở **titlebar** — không tổ tiên nào focusable — nên ép tiêu điểm
 * lên nó là đủ. Nút mở Attribution nằm trong **panel Lookup**, tức trong một
 * `section.panel[tabindex="-1"]`, và ở đó tiêu điểm **không giữ được trên nút** dù đặt bằng
 * cách nào. Mệnh đề AC11 vì thế đã được LÀM RÕ, không bị bỏ — xem khối khẳng định ở cuối ca.
 *
 * ⚠️ Mọi lượt bấm đi qua `realClick()`. Lý do và số đo: `e2e/support/pointer.mjs`.
 */

import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

/** Mối nối `data-`, không tên lớp CSS — cùng doctrine với `data-shortcuts-open`. */
const OPENER = '[data-attribution-open]'
const PANEL = '.attr-panel'

/** Node đang giữ tiêu điểm, đọc TRONG webview thật. */
async function activeElementSignature() {
  return browser.execute(() => {
    const el = document.activeElement
    if (el === null) return 'null'
    return [
      el.tagName.toLowerCase(),
      el.hasAttribute('data-attribution-open') ? 'data-attribution-open' : '',
      el.className || '',
    ]
      .filter((s) => s !== '')
      .join('|')
  })
}

describe('Story 1.19 · AC11 — Escape trả tiêu điểm về nút mở Attribution hoặc khung chứa nó', () => {
  /**
   * 🔴 ĐIỀU KIỆN CHẠY — nút `[data-attribution-open]` sống trong **panel Lookup**, panel
   * đó chỉ được dựng trong `WorkspaceDock` của chế độ **`workspace`**, và app khởi động ở
   * **`library`** (`modes/modeState.ts:33`). Nên ca này bắt đầu bằng một fixture.
   *
   * ⚠️ Fixture đó chỉ dựng được sau khi **hai** bề mặt dữ liệu thật đã bị chuyển hướng
   * (`$APPDATA` **và** thư mục gốc Library) — trước đó nó ghi một Tác phẩm vào
   * `~/Documents/AuraTranslate/` của người chạy mỗi lượt. Lịch sử đó nằm ở
   * `deferred-work.md`, và nó là lý do ca này `skip` suốt một ngày trước khi chạy được.
   */
  it('đóng lớp phủ bằng Escape thì tiêu điểm về nút mở hoặc khung chứa nó, không rơi về <body>', async function () {
    await openWorkspaceWithWork(`e2e-attribution-focus-${Date.now()}`)

    // ── 🔴 TIỀN ĐỀ CÓ ĐIỀU KIỆN — thêm 2026-08-20, sau lượt CI đầu tiên ─────────────
    //
    // Ca này là spec DUY NHẤT của bộ thật sự cần dữ liệu từ điển: nút `[data-attribution-
    // open]` nằm trong dải chip nguồn, và dải chỉ render khi `dictSources.length > 0`.
    // Trên runner GitHub `src-tauri/target/debug/dict/*.db` không tồn tại (356 MB, do
    // `tools/dict-build` sinh ra; AC cuối của Story 1.3 cấm CI tải dữ liệu từ điển), nên
    // ca này KHÔNG chạy được ở đó. `openWorkspaceWithWork` thôi đợi nút này từ cùng ngày —
    // nó đợi lưới — nên tiền đề phải được hỏi TẠI ĐÂY.
    //
    // 🔴 VÀ NÓ HỎI ĐÚNG CÂU, KHÔNG HỎI CÂU DỄ. Hỏi *"nút có mặt không"* rồi bỏ qua khi
    // vắng sẽ nuốt luôn một hồi quy thật — ngày nút biến mất trên máy CÓ từ điển, ca này
    // im lặng chuyển sang `skip` thay vì đỏ, tức đúng loại cổng không bao giờ đỏ mà kho
    // này cấm. Nên nó hỏi NGUỒN TỪ ĐIỂN qua IPC, không hỏi cái nút:
    //   · 0 nguồn  ⇒ tiền đề bàn đo không dựng được ⇒ `skip`, in ra lý do;
    //   · có nguồn ⇒ nút PHẢI có mặt, và vắng mặt là một lượt ĐỎ thật.
    const dictSourceCount = await browser.execute(async () => {
      try {
        const sources = await window.__TAURI_INTERNALS__.invoke('list_dict_sources')
        return Array.isArray(sources) ? sources.length : -1
      } catch {
        return -1
      }
    })

    if (dictSourceCount < 0) {
      throw new Error(
        '[BÀN ĐO HỎNG] `list_dict_sources` ném hoặc trả về một hình dạng không phải mảng.\n' +
          'Tiền đề chưa dựng được, nên mệnh đề AC11 CHƯA ĐƯỢC ĐO — lượt đỏ này không đọc ' +
          'thành "sản phẩm đạt" hay "sản phẩm hỏng", cả hai đều là kết quả không có thật.',
      )
    }

    if (dictSourceCount === 0) {
      console.warn(
        '[BÀN ĐO KHÔNG DỰNG ĐƯỢC] `list_dict_sources` trả 0 nguồn, nên dải chip nguồn ' +
          'không render và nút `[data-attribution-open]` không tồn tại.\n' +
          'Mệnh đề AC11 của Story 1.19 CHƯA ĐƯỢC ĐO ở lượt này — đừng đọc lượt `skip` ' +
          'này thành "đạt". Dựng dữ liệu từ điển (`src-tauri/target/debug/dict/*.db`) rồi ' +
          'chạy lại. Nợ có chủ ở `deferred-work.md`.',
      )
      this.skip()
    }

    const opener = await $(OPENER)

    // ── Mở bằng CHUỘT THẬT ───────────────────────────────────────────────────────
    // Đây là đường mà khuyết tật WKWebView sống. Một ca mở bằng bàn phím sẽ XANH kể cả
    // khi đường lui hỏng, vì bàn phím ĐẶT tiêu điểm lên nút còn chuột thì không.
    await realClick(opener)

    const panel = await $(PANEL)
    await panel.waitForDisplayed({ timeout: 10_000 })

    // Bẫy tiêu điểm của `aria-modal="true"` phải THẬT: tiêu điểm nằm trong lớp phủ.
    const insideOverlay = await browser.execute(
      (sel) => document.querySelector(sel)?.contains(document.activeElement) ?? false,
      PANEL,
    )
    expect(insideOverlay).toBe(true)

    // ── Đóng bằng Escape ─────────────────────────────────────────────────────────
    // ⚠️ `@keydown.esc` gắn trên `.attr-scrim`, nên `Escape` chỉ tới được handler khi tiêu
    // điểm còn TRONG lớp phủ — khẳng định ngay trên là điều kiện để bước này có nghĩa.
    await browser.keys(['Escape'])
    await panel.waitForExist({ reverse: true, timeout: 10_000 })

    // ── Mệnh đề của AC11, ĐÃ LÀM RÕ 2026-08-12 (Ice chốt) ────────────────────────
    //
    // 🔴 Đích là **nút mở HOẶC một tổ tiên của nó**, không phải riêng nút. Đây là một lần
    // nới AC có chữ ký, không một lượt hạ chuẩn cho dễ xanh — lý do đo được:
    //
    // Nút mở của lớp phủ này nằm TRONG một panel dockview, và trên WKWebView tiêu điểm
    // **không giữ được trên nó**: đặt lên nút thì nó rơi lên `section.panel[tabindex="-1"]`
    // ngay trong cùng một tick (`focusout ← button` rồi `focusin → section.panel`). Bốn giả
    // thuyết đã bị BÁC BẰNG ĐO, không bằng lập luận: hành vi mặc định của `mousedown`
    // (`preventDefault()` không đổi gì) · dockview (`dockview-core` chỉ dùng `focusin` ở
    // `popupService`) · mã ứng dụng (liệt kê trọn `.focus()` trong `src/`) · nút không phải
    // tab stop (`tabindex="0"` không đổi gì).
    //
    // ⚠️ Nút mở của `ShortcutsOverlay` nằm ở titlebar, KHÔNG tổ tiên nào focusable, và ở đó
    // tiêu điểm dính đúng vào nút — `shortcuts-focus.e2e.mjs` vẫn đòi đúng nút. Hai lớp phủ
    // khai hai mệnh đề khác nhau vì chúng ngồi ở hai chỗ khác nhau, không vì một cái được
    // châm chước.
    //
    // 🔴 Khẳng định này VẪN CÓ RĂNG: nó đỏ khi tiêu điểm về `<body>`, và đỏ khi nó về một
    // panel KHÁC — đúng hai hình dạng hỏng đã đo được trong lúc dựng ca này
    // (`div.original tok-source-cjk` của Panel Source là một trong hai).
    const returnedNearOpener = await browser.execute((sel) => {
      const opener = document.querySelector(sel)
      const active = document.activeElement
      if (opener === null || active === null) return false
      return active === opener || active.contains(opener)
    }, OPENER)
    expect(returnedNearOpener).toBe(true)

    const signature = await activeElementSignature()
    expect(signature).not.toContain('body')
  })
})
