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

/**
 * Thao tác chưa gán phím mặc định — AC7 liệt kê nó trong nhóm `unbound()`.
 *
 * 🔵 **2026-08-14 (Story 2.5b): `layout.toggle_source` → `layout.toggle_grid`.** Hai panel
 * `Nguyên văn` + `Bản dịch` gộp thành `panel.grid`, nên `PANEL_SUFFIXES` còn ba và command
 * `layout.toggle_source` **thôi tồn tại**.
 *
 * 🔴 **Chỗ này là một khoảng mù ĐO ĐƯỢC, ghi ra thay vì vá im lặng:** command id nằm **cứng**
 * trong một spec e2e, và **không cổng nào canh mối nối đó** — `check:commands` đọc `src/**`,
 * không đọc `e2e/**`. Lượt đổi tên đi qua sạch **chín cổng, build, và cả vitest**; nó chỉ lộ
 * ra ở lượt chạy e2e **bằng tay**, dưới dạng một timeout 10 giây nói *"phần tử không hiện"* —
 * một câu đúng về triệu chứng và **câm về nguyên nhân**. Bản đồ tệp của Story 2.5b liệt kê
 * `e2e/specs/editor-*.e2e.mjs` mà **bỏ sót** tệp này.
 */
const TARGET_COMMAND = 'layout.toggle_grid'
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
    //
    // 🔵 SỬA 2026-08-28: `Mod+Alt+K` → `Mod+Alt+Y`, và mệnh đề cũ đã HẾT ĐÚNG.
    //
    // ⚠️ Bản trước gõ `Mod+Alt+K` kèm chú thích *"trống trên toàn bộ bộ mặc định"*. Story 5.3
    // (2026-08-27) cấp ĐÚNG hợp âm đó cho `library.rescan` (`src/commands/index.ts`), nên lượt
    // gán ở đây đâm vào nhánh XUNG ĐỘT (AC3) và bị từ chối — ca đỏ, và thông điệp đổ lỗi cho
    // một khuyết tật WKWebView không liên quan. Story 5.3 CÓ đo trước khi cấp phím, nhưng nó
    // `grep` trên `src/commands/index.ts`; hợp âm này neo ở ĐÂY, ngoài tầm phép đo đó.
    //
    // 🔴 Ca này chỉ cần MỘT hợp âm còn trống, không cần hợp âm nào cụ thể. Đo 2026-08-28 bằng
    // `grep -ohE "Mod\+Alt\+[A-Za-z0-9]+" src/commands/index.ts | sort -u`: còn trống
    // `A B D E F I N Y Z`. Loại `D` (macOS: ẩn/hiện Dock) và `I` (WebKit: Web Inspector) vì cả
    // hai bị hệ điều hành/engine nuốt trước khi tới webview. Chọn `Y`.
    await browser.keys(['Meta', 'Alt', 'y'])

    // 🔴 Bắt lượt chờ để một lần va chạm SAU NÀY tự nói ra nguyên nhân. Không có khối này,
    // mọi lý do khiến ô phím không đổi — xung đột hợp âm, kho từ chối, tiêu điểm sai — đều
    // đọc lên thành cùng một câu đổ lỗi cho WKWebView, đúng thứ "một lượt ĐỎ nói sai nguyên
    // nhân" mà chính spec này cảnh báo ở khối `realClick` phía trên.
    try {
      await browser.waitUntil(async () => (await cell.getText()).trim() !== before, { timeout: 5_000 })
    } catch {
      const alerts = await browser.execute(() =>
        Array.from(document.querySelectorAll('.sc-alert')).map((node) => (node.textContent || '').trim()),
      )
      throw new Error(
        `ô phím của \`${TARGET_COMMAND}\` KHÔNG đổi sau lượt gán bằng chuột (vẫn "${before}").\n` +
          `Dải cảnh báo của màn hình đang nói: ${JSON.stringify(alerts)}\n` +
          'Nếu ở đó có câu XUNG ĐỘT thì nguyên nhân là hợp âm này đã bị một lệnh khác chiếm — ' +
          'đo lại bằng `grep -ohE "Mod\\+Alt\\+[A-Za-z0-9]+" src/commands/index.ts | sort -u` rồi ' +
          'chọn một hợp âm còn trống, ĐỪNG sửa sản phẩm.\n' +
          'Nếu dải rỗng thì mới là khuyết tật WKWebView mà `captureShortcut()` ép tiêu điểm để sửa.',
      )
    }

    const after = (await cell.getText()).trim()
    expect(after).not.toBe(before)
    expect(after.toLowerCase()).toContain('y')

    // Trả hàng về mặc định — cô lập giữa các ca trong cùng spec (kho nay là kho TẠM).
    await resetRowToDefault()
  })
})
