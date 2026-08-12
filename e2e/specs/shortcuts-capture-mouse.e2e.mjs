/**
 * Bàn đo Story 1.21 — vòng gán phím qua **đường CHUỘT** (AC2), trên WKWebView thật.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * ĐÂY LÀ PHÉP THỬ GO/NO-GO CỦA CẢ PHƯƠNG ÁN e2e
 * ═════════════════════════════════════════════════════════════════════════════════
 * Câu hỏi mà §5 của báo cáo retrospective đặt ra, và §7 của đề xuất đòi trả lời trước khi
 * dựng tiếp 28 hàng: **bộ chạy có tái lập được lớp lỗi ĐẶC THÙ ENGINE hay không?**
 *
 * Khuyết tật thật, tìm ra ở lượt code review Story 1.21 (hạng cao nhất trong mười phát
 * hiện): *WKWebView không đặt tiêu điểm cho `<button>` khi bấm chuột*. Hệ quả là ô phím
 * không giữ tiêu điểm, `@keydown` gắn trên nó không bao giờ nổ, và **đường chuột của AC2
 * chết hoàn toàn trên macOS** — trong khi chín cổng, `cargo test`, `npm run build` và
 * toàn bộ bộ đo 24 phép trên đường sản phẩm đều XANH. Bản vá là cú ép tiêu điểm ở
 * `config/shortcutsState.ts::captureShortcut`.
 *
 * 🔴 Một bộ chạy trong Chrome KHÔNG bắt được ca này: Blink đặt tiêu điểm cho `<button>`
 * khi bấm, nên ca sẽ XANH kể cả khi bản vá bị hoàn nguyên. Đó là lý do phương án Chrome
 * bị loại và phương án webview nhúng được chọn.
 *
 * Nghiệm thu của chính ca này là ĐỎ-RỒI-XANH: hoàn nguyên `keyCellOf(id)?.focus()` thì nó
 * phải đỏ. Nếu nó vẫn xanh, bộ chạy KHÔNG thay được mắt người và ta dừng ở đây — mệnh đề
 * đó đã ghi ở §7 đề xuất trước khi chạy dòng đầu tiên.
 */

import { realClick } from '../support/pointer.mjs'

const OPENER = '[data-shortcuts-open]'
const PANEL = '.sc-panel'

/** Thao tác chưa gán phím mặc định — AC7 liệt kê nó trong nhóm `unbound()`. */
const TARGET_COMMAND = 'layout.toggle_source'
const ROW = `[data-command-id="${TARGET_COMMAND}"]`
const KEY_CELL = `${ROW} [data-key-cell]`

/** Đưa hàng đích về hợp âm mặc định của sản phẩm qua nút *"Về mặc định"* (AC8). */
async function resetRowToDefault() {
  const actions = await $$(`${ROW} .sc-act`)
  // Hai nút một hàng, theo thứ tự khai ở `ShortcutsOverlay.vue`: `unassign` rồi `reset`.
  const reset = actions[actions.length - 1]
  await realClick(reset)
  await browser.pause(300)
}

describe('Story 1.21 · AC2 đường CHUỘT — vòng gán phím trên WKWebView', () => {
  it('bấm chuột vào ô phím rồi gõ hợp âm thì hàng đó nhận phím mới', async () => {
    const opener = await $(OPENER)
    await opener.waitForExist({ timeout: 30_000 })
    await realClick(opener)
    await $(PANEL).waitForDisplayed({ timeout: 10_000 })

    await $(KEY_CELL).waitForDisplayed({ timeout: 10_000 })

    // ── Đưa hàng về MẶC ĐỊNH trước khi đo ────────────────────────────────────────
    // 🔴 Ca này KHÔNG được giả định trạng thái đầu. Story 1.21 ghi phím tắt **xuống đĩa**
    // (`ScopeKind::Shortcut`, `global.db`), nên một lượt e2e trước để lại hợp âm của nó và
    // lượt sau đọc `before` = giá trị cũ rồi gán đúng hợp âm đó ⇒ *"không có gì đổi"* ⇒ ĐỎ
    // với một câu đổ lỗi cho sản phẩm. Đo được ở lượt dựng ca này: ô phím vào ca với
    // `⌥⌘K` còn sót từ một lượt chẩn đoán.
    //
    // ⚠️ Vế *"dùng chung `$APPDATA`"* của chú thích này ĐÃ ĐÓNG 2026-08-11 (Story 1.22 AC2):
    // mỗi lượt chạy nay có kho riêng. Giữ lượt reset vì nó vẫn cô lập giữa các ca TRONG
    // cùng một spec, thứ thư mục tạm theo lượt chạy không cho.
    await resetRowToDefault()

    // 🔴 LẤY LẠI handle SAU lượt reset, không dùng lại handle lấy trước đó.
    //
    // Đo được 2026-08-12: lượt reset dựng lại hàng, nên một handle giữ từ trước thành
    // **tham chiếu chết** và ca đỏ bằng `"element wasn't found"` — một lỗi HẠ TẦNG của bàn
    // đo đội lốt một hồi quy sản phẩm. Nó **chập chờn** vì nó phụ thuộc Vue có thật sự tái
    // tạo node ở lượt đó hay không: bốn lượt chạy cả bộ đầu tiên đi qua sạch, lượt thứ năm
    // mới lộ ra. Đúng lớp lỗi mà một bàn đo chạy hiếm sẽ giấu được rất lâu.
    const cell = await $(KEY_CELL)
    await cell.waitForDisplayed({ timeout: 10_000 })
    const before = (await cell.getText()).trim()

    // ── Đường CHUỘT, và chỉ chuột ────────────────────────────────────────────────
    // 🔴 Không `Tab`, không `browser.execute(el => el.focus())`. Cả hai đường đó đặt tiêu
    // điểm bằng cách KHÁC và sẽ XANH kể cả khi khuyết tật WKWebView quay lại — tức chúng
    // biến ca này thành một bảng xanh trang trí.
    //
    // 🔴 VÀ KHÔNG `cell.click()` — đo được ở lượt dựng ca này (2026-08-11): lệnh `click`
    // của driver KHÔNG trung thực về thứ tự sự kiện. Nó bắn `click` TRƯỚC `focusin`, nên
    // `shortcuts.capture` chạy lúc `aimedRow` còn rỗng và màn hình trả về câu *"Chưa nhắm
    // được thao tác nào"* — một lượt ĐỎ nói sai nguyên nhân, đổ lỗi cho sản phẩm trong
    // khi lỗi ở bộ lái. Chuột thật đi `mousedown -> focusin -> mouseup -> click`.
    //
    // ⇒ Actions API dựng đúng chuỗi đó. Ghi ra đây vì mọi hàng bàn đo còn lại mà thứ tự
    // sự kiện có nghĩa đều phải đi đường này, không đường `click()`.
    await realClick(cell)

    // Vào trạng thái *đang bắt* — đọc bằng chính câu `shortcuts.capturing` của màn hình.
    // ⚠️ Đừng chỉ đếm `.sc-alert`: dải đó cũng chở `shortcuts.disk_rejected` và câu lỗi,
    // nên một phép đếm sẽ XANH oan ngay cả khi lượt bắt không hề bật.
    await browser.waitUntil(
      async () => {
        const texts = await browser.execute(() =>
          Array.from(document.querySelectorAll('.sc-alert')).map((e) => e.textContent.trim()),
        )
        return texts.some((t) => t.includes('Đang chờ một tổ hợp phím'))
      },
      { timeout: 5_000, timeoutMsg: 'không vào được trạng thái đang bắt sau một lượt bấm chuột thật' },
    )

    // ── Gõ hợp âm ────────────────────────────────────────────────────────────────
    // `Mod+Alt+K`: trống trên toàn bộ bộ mặc định, nên nó không đụng AC3 (xung đột).
    await browser.keys(['Meta', 'Alt', 'k'])

    await browser.waitUntil(async () => (await cell.getText()).trim() !== before, {
      timeout: 5_000,
      timeoutMsg:
        `ô phím của \`${TARGET_COMMAND}\` KHÔNG đổi sau lượt gán bằng chuột (vẫn "${before}").\n` +
        'Đây là đúng hình dạng khuyết tật WKWebView mà `captureShortcut()` ép tiêu điểm để sửa.',
    })

    const after = (await cell.getText()).trim()
    expect(after).not.toBe(before)
    expect(after.toLowerCase()).toContain('k')

    // Trả hàng về mặc định — cô lập giữa các ca trong cùng spec (kho nay là kho TẠM).
    await resetRowToDefault()
  })
})
