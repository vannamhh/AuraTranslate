/**
 * Bàn đo Story 1.21 — hàng **17** (UX-DR17): tiêu điểm quay đúng về nút đã mở lớp phủ.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO HÀNG NÀY ĐI ĐẦU
 * ═════════════════════════════════════════════════════════════════════════════════
 * Nó là hàng RẺ NHẤT trong 28 hàng treo mà vẫn nằm đúng lớp lỗi đắt nhất: thuần DOM +
 * tiêu điểm, không cần Tác phẩm, không cần từ điển, không cần một tiến trình khởi động
 * lại. Và §Testing của story ghi nó thành *"hàng canh HỒI QUY"* sau lượt code review —
 * mười bản vá đi qua đúng đường của nó, vì cú ép tiêu điểm mới lên ô phím đổi *"node nào
 * đang giữ tiêu điểm lúc đóng"*, tức đổi đúng đầu vào của đường trả tiêu điểm.
 *
 * ⚠️ Đây là bàn đo của MỘT hàng, không phải một bộ e2e. Bước 2 của đề xuất mới là phép
 * thử thật của cả phương án: lái hàng 15 (vòng gán phím không chạm chuột) rồi hoàn nguyên
 * bản vá *"ép tiêu điểm lúc arming"* và đòi ca đó ĐỎ **vì đúng lý do WKWebView**. Nếu vế
 * đó không tái lập được, dừng ở Bước 2 — Ice đã đọc mệnh đề này ở §7 của đề xuất.
 */

import { realClick } from '../support/pointer.mjs'

/** Selector là các mối nối `data-`, không tên lớp CSS — xem chú thích ở `App.vue`. */
const OPENER = '[data-shortcuts-open]'
const PANEL = '.sc-panel'

/** Node đang giữ tiêu điểm, đọc TRONG webview thật. */
async function activeElementSignature() {
  return browser.execute(() => {
    const el = document.activeElement
    if (el === null) return 'null'
    return [
      el.tagName.toLowerCase(),
      el.hasAttribute('data-shortcuts-open') ? 'data-shortcuts-open' : '',
      el.className || '',
    ]
      .filter((s) => s !== '')
      .join('|')
  })
}

describe('Story 1.21 · hàng 17 — UX-DR17: tiêu điểm quay về nút đã mở', () => {
  it('đóng lớp phủ bằng Escape thì tiêu điểm về đúng nút mở, không rơi về <body>', async () => {
    const opener = await $(OPENER)
    await opener.waitForExist({ timeout: 30_000 })

    // ── Mở bằng CHUỘT ────────────────────────────────────────────────────────────
    // 🔴 Cố ý đi đường chuột: đây là đúng đường mà lượt code review phát hiện chết hoàn
    // toàn trên macOS (WKWebView không đặt tiêu điểm cho `<button>` khi bấm). Một ca mở
    // bằng bàn phím sẽ XANH kể cả khi bản vá `@focusin` bị hoàn nguyên.
    //
    // ⚠️ Sửa 2026-08-11 (Story 1.22, AC3), và ghi ra vì nó là một mâu thuẫn THẬT chứ
    // không một lượt dọn hình thức: dòng dưới đây trước là `opener.click()`. Tức ca này
    // khai bằng chữ rằng nó *"cố ý đi đường chuột"* trong khi gọi đúng lệnh mà spec bên
    // cạnh vừa đo được là KHÔNG trung thực về thứ tự sự kiện (`click` trước `focusin`).
    // Chú thích nói một đằng, mã đi một nẻo — và một bàn đo như vậy tự làm hỏng lý do
    // nó tồn tại. Nay đi qua `realClick()`; lý do đầy đủ ở `e2e/support/pointer.mjs`.
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
    await browser.keys(['Escape'])
    await panel.waitForExist({ reverse: true, timeout: 10_000 })

    // ── Mệnh đề của UX-DR17 ──────────────────────────────────────────────────────
    // Vế ÂM viết ra tường minh: `<body>` là đúng thứ tiêu điểm rơi về khi đường lui hỏng,
    // và một khẳng định chỉ nói "không phải body" sẽ xanh với một nút BẤT KỲ khác. Nên
    // hỏi cả hai chiều.
    const signature = await activeElementSignature()
    expect(signature).toContain('data-shortcuts-open')
    expect(signature).not.toContain('body')
  })
})
