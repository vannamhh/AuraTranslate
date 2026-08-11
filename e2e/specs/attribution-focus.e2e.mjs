/**
 * Bàn đo Story 1.19 — AC11: `Escape` đóng lớp phủ Attribution và **trả tiêu điểm về chỗ cũ**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY RA ĐỜI — nó là một PHÉP ĐO, không một lượt mở rộng bao phủ
 * ═════════════════════════════════════════════════════════════════════════════════
 * Story 1.22 C2 đổi bàn đo sang chuột THẬT (Actions API), và `shortcuts-focus` lập tức ĐỎ:
 * trên WKWebView, nút `<button>` **không nhận tiêu điểm khi bấm**, nên `document.activeElement`
 * lúc mở lớp phủ là node đang giữ tiêu điểm từ trước — không phải nút mở. Đường lui của
 * UX-DR17 lưu node đó rồi trả về đúng nó, và nhánh dự phòng `[data-…-open]` **không bao giờ
 * chạy tới** vì `activeElement` gần như không bao giờ rỗng.
 *
 * `AttributionOverlay.vue:57-70` mang **khuôn giống hệt** `ShortcutsOverlay.vue:56-80` — chính
 * doc-comment của tệp sau ghi *"khuôn và lý lẽ chép từ `AttributionOverlay.vue`, cả hai vế"*.
 *
 * 🔴 Nhưng "giống khuôn" là một suy luận, không một phép đo. Ice chốt 2026-08-11: dựng spec
 * này để **BIẾT** thay vì suy. Hai lớp phủ khác nhau ở một chỗ có thể đổi kết quả — nút mở
 * của Attribution nằm trong **panel Lookup** (`LookupPanel.vue`), còn nút mở phím tắt nằm ở
 * **titlebar** — nên node giữ tiêu điểm trước lượt bấm không nhất thiết là cùng một node.
 *
 * ⚠️ Mọi lượt bấm đi qua `realClick()`. Lý do và số đo: `e2e/support/pointer.mjs`.
 */

import { realClick } from '../support/pointer.mjs'

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

describe('Story 1.19 · AC11 — Escape trả tiêu điểm về nút đã mở Attribution', () => {
  /**
   * 🔴 ĐIỀU KIỆN CHẠY, và vì sao nó là một `skip` CÓ TÊN chứ không một lượt trượt.
   *
   * Đo 2026-08-11: nút `[data-attribution-open]` sống trong **panel Lookup**
   * (`LookupPanel.vue`), panel đó chỉ được dựng trong `WorkspaceDock` của chế độ
   * **`workspace`**, và app khởi động ở chế độ **`library`** (`modes/modeState.ts:33`).
   * Vào được `workspace` cần một **Tác phẩm đang mở** — tức một fixture mà bộ e2e chưa có.
   *
   * Để nguyên thì ca này trượt sau 30 giây với câu *"element still not existing"* — một
   * lỗi HẠ TẦNG đội lốt một hồi quy giao diện, đúng lớp nhầm lẫn mà `wdio.conf.mjs` đã
   * phải ghi ra một lần cho ca `about:blank`. Nên nó `skip` kèm lý do in ra màn hình:
   * **rỗng im lặng bị cấm, rỗng có lý do thì không** — cùng luật AD-44 ④ áp cho truy vấn
   * quá ngắn.
   *
   * ⚠️ Fixture đó cần một việc khác làm trước: bộ e2e mới chỉ chuyển hướng `$APPDATA`
   * (Story 1.22 AC2). Thư mục gốc Library đi đường **khác** — `document_dir()`, tức
   * `~/Documents/AuraTranslate/` (`commands/project.rs:60`) — nên một fixture tạo Tác phẩm
   * hôm nay sẽ ghi vào thư mục Documents THẬT của người chạy. Xem mục nợ ở
   * `deferred-work.md`.
   */
  it('đóng lớp phủ bằng Escape thì tiêu điểm về đúng nút mở, không rơi về <body>', async function () {
    const opener = await $(OPENER)
    if (!(await opener.isExisting())) {
      console.log(
        `[e2e] BỎ QUA có lý do: không thấy ${OPENER}.\n` +
          '      Nút này sống trong panel Lookup ⇒ chỉ có ở chế độ `workspace`, mà app\n' +
          '      khởi động ở `library`. Cần một fixture mở Tác phẩm — chưa dựng, và nó\n' +
          '      phụ thuộc việc chuyển hướng thư mục gốc Library (`document_dir()`), thứ\n' +
          '      AC2 của Story 1.22 CHƯA phủ. Xem `deferred-work.md`.',
      )
      this.skip()
      return
    }

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

    // ── Mệnh đề của AC11 ─────────────────────────────────────────────────────────
    // Hỏi cả hai chiều: một khẳng định chỉ nói "không phải body" sẽ XANH với một nút BẤT
    // KỲ khác, và đó đúng là hình dạng hỏng đã đo được ở `shortcuts-focus`.
    const signature = await activeElementSignature()
    expect(signature).toContain('data-attribution-open')
    expect(signature).not.toContain('body')
  })
})
