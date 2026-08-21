/**
 * Fixture — đưa app vào chế độ `workspace` với một Tác phẩm đang mở.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO CẦN FIXTURE NÀY
 * ═════════════════════════════════════════════════════════════════════════════════
 * App khởi động ở chế độ `library` (`modes/modeState.ts`), và **phần lớn bề mặt của Epic
 * 1 sống trong `workspace`**: ba panel *(🔵 bốn → ba ở Story 2.5b)*, lưới đối chiếu, Panel Lookup, và nút mở màn hình
 * Attribution. Mọi hàng bàn đo chạm tới chúng đều bắt đầu bằng đúng hai bước dưới đây.
 *
 * 🔴 Fixture này chỉ dựng được SAU khi hai bề mặt dữ liệu thật đã bị chuyển hướng —
 * `$APPDATA` **và** thư mục gốc Library. Trước đó, một fixture tạo Tác phẩm sẽ ghi vào
 * `~/Documents/AuraTranslate/` của người chạy mỗi lượt. Thứ tự đó không phải tình cờ: bề
 * mặt thứ hai được tìm ra **trong lúc chuẩn bị chính fixture này**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * HAI LỰA CHỌN CÓ CHỦ Ý
 * ═════════════════════════════════════════════════════════════════════════════════
 * ① **Tạo Tác phẩm bằng IPC, không bằng form Library.** Form đó không có một mối nối
 *    `data-` nào (`LibraryMode.vue` — `v-model` trên `<input>` trần), nên một fixture đi
 *    qua nó phải chọn phần tử theo cấu trúc DOM và sẽ vỡ ở lượt đổi bố cục đầu tiên. Quan
 *    trọng hơn: mọi hàng dùng fixture này đo **một thứ khác** — Panel Lookup, tiêu điểm,
 *    Attribution — nên một fixture giòn sẽ làm chúng đỏ vì lý do không liên quan.
 *    ⚠️ Đánh đổi phải nói ra: fixture này **không** đo đường nhập của người dùng thật.
 *    Ngày có một hàng bàn đo cho chính form Library, nó phải đi qua giao diện, không qua
 *    đây.
 *
 * ② **Đổi chế độ bằng BÀN PHÍM (`Mod+2`), không bằng cách bấm tab.** Ba tab chế độ ở
 *    `App.vue` không mang mối nối `data-` nào, và thêm một mối nối vào mã sản phẩm **chỉ
 *    để test chọn được** là mở một tiền lệ mà kho này cố ý chưa mở: `data-shortcuts-open`
 *    và `data-attribution-open` tồn tại vì **sản phẩm** cần chúng (đường lui của tiêu
 *    điểm), không vì bàn đo. Bàn phím tránh hẳn câu hỏi đó và đi đúng đường NFR17 hứa.
 */

import { resetPanelState } from './panelReset.mjs'

/** Khớp `DOCUMENTS_SUBFOLDER` ở `src-tauri/src/commands/project.rs`. */
export const WORK_SUFFIX = '.atproj'

/**
 * Tạo một Tác phẩm rồi vào `workspace`, đợi tới khi Panel Lookup thật sự có mặt.
 *
 * @param {string} name tên Tác phẩm — nên mang dấu của lượt chạy để đọc ra được nếu nó
 *   rơi nhầm vào thư mục thật
 * @param {string} [text] văn bản nguồn. Mặc định là **một** câu.
 *
 *   🔵 Tham số này thêm ở Story 2.8, và nó là một tham số **TUỲ CHỌN có giá trị mặc định y
 *   nguyên chuỗi cũ** — có chủ ý. `2-5b-ban-do` đã ghi bằng chữ rằng *"đổi một fixture dùng
 *   chung để chữa một ca là cách rẻ nhất để làm đỏ năm ca khác"*; một tham số tuỳ chọn không
 *   đổi một byte nào cho bảy spec đang gọi nó không tham số. Lý do cần nó: gộp/tách đòi **ít
 *   nhất hai** segment, và chuỗi mặc định cho đúng **một**.
 *
 *   ⚠️ Tác phẩm tạo với `sourceLang: 'zh'`, nên bộ tách nhìn `。！？；` — không dấu chấm
 *   tiếng Anh. Một chuỗi tiếng Việt có dấu `.` vẫn cho **một** segment.
 * @returns {Promise<void>}
 */
export async function openWorkspaceWithWork(
  name,
  text = 'Một câu nguồn để bộ nhập có việc mà làm.',
) {
  // ── 🔴 CHỜ CẦU IPC TRƯỚC KHI DÙNG NÓ — thêm 2026-08-20, sau lượt CI đầu tiên ──────
  //
  // Bản trước hỏi `window.__TAURI_INTERNALS__` NGAY, một lần, và trả *"không có cầu IPC"*
  // nếu nó chưa có. Trên máy Ice không ai thấy nhánh đó bao giờ; trên runner GitHub nó nổ
  // ở đúng spec ĐẦU TIÊN của lô (`attribution-focus`, đỏ sau **73 ms** — quá nhanh để là
  // bất cứ thứ gì trừ một lượt hỏi trước khi trang kịp dựng), rồi mười một spec sau đó
  // chạy bình thường. Phiên WebDriver mở xong KHÔNG đồng nghĩa `main.ts` đã chạy: bộ lái
  // nối vào webview ngay khi cửa sổ có mặt.
  //
  // ⚠️ Con số trong câu báo lỗi đọc SAU vòng chờ, không nội suy vào tham số — cùng luật
  // mà `support/gridWait.mjs` rút ra từ lượt 11 (`timeoutMsg` dựng lúc tạo object cho một
  // câu chẩn đoán KHÔNG có thật trên một lượt đỏ thật).
  try {
    await browser.waitUntil(
      async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined),
      { timeout: 30_000, interval: 250 },
    )
  } catch {
    const url = await browser.execute(() => window.location.href)
    throw new Error(
      `Không thấy cầu IPC (\`window.__TAURI_INTERNALS__\`) sau 30 giây. URL đang mở: ${url}\n\n` +
        'Đây là lỗi HẠ TẦNG của bàn đo — app chưa nạp xong, hoặc webview đang ở\n' +
        '`about:blank` (nhị phân debug nạp `devUrl`, nên Vite phải sống trước). KHÔNG\n' +
        'một hồi quy sản phẩm; đừng đọc ca đỏ phía sau nó thành một khuyết tật.',
    )
  }

  const created = await browser.execute(async (workName, sourceText) => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) return { ok: false, detail: 'không có cầu IPC' }
    try {
      await internals.invoke('create_work_from_text', {
        name: workName,
        sourceLang: 'zh',
        genre: 'general',
        text: sourceText,
      })
      return { ok: true, detail: '' }
    } catch (err) {
      return { ok: false, detail: String(err && err.code ? err.code : err) }
    }
  }, name, text)

  if (!created.ok) {
    throw new Error(
      `Fixture không tạo được Tác phẩm "${name}": ${created.detail}\n\n` +
        'Đây là lỗi HẠ TẦNG của bàn đo, không một hồi quy giao diện — đừng đọc một ca đỏ\n' +
        'phía sau nó thành một khuyết tật sản phẩm.',
    )
  }

  // ── 🔴 AC2 (Story 2.12) — SOI ĐÚNG `finishSubmit`, VÀ ĐÚNG CHỖ NÓ CHẠY ────────────
  //
  // Fixture tạo Tác phẩm qua IPC nên nó **đi vòng qua** `modes/libraryImport.ts::finishSubmit`
  // — điểm nghẽn DUY NHẤT trong sản phẩm vừa dọn state panel vừa phát lại lượt nạp.
  //
  // 🔴 **SAU lượt tạo, không TRƯỚC.** `finishSubmit` chạy sau khi `create_work_from_*` trả về,
  // tức sau khi `replace_open_work` phía Rust đã trỏ `OpenWorkState` sang Tác phẩm MỚI. Một
  // lượt nạp phát trước lời gọi đó sẽ nạp Tác phẩm **cũ**.
  //
  // ⚠️ **Bản đầu của story này đặt nó ở ĐẦU fixture và chỉ dọn, không nạp** — lý lẽ khi đó là
  // *"một spec đỏ giữa chừng không chạy tới phần dọn của chính nó"*. Lý lẽ ấy đúng về vế dọn
  // và **sai về vế nạp**, và phép đo đã bác nó: lượt trọn bộ thứ mười cho 5 passed / 6 failed
  // với `Lần đọc cuối: 0` ở mọi ca đỏ — lưới không bao giờ nạp. Xem `support/panelReset.mjs`.
  await resetPanelState()

  // `Mod+2` — `mode.workspace`. `Mod` phân giải thành `Meta` trên macOS.
  await browser.keys(['Meta', '2'])

  // ── 🔴 MỐC SẴN SÀNG: LƯỚI ĐÃ NẠP — đổi 2026-08-20, sau lượt CI đầu tiên ──────────
  //
  // Bản trước đợi `[data-attribution-open]`. Lý lẽ của nó đúng và vẫn đúng — *"đợi Panel
  // Lookup có mặt THẬT, không chỉ đợi chế độ đổi ... một ca không đợi vế đó sẽ đỏ bằng
  // 'không tìm thấy phần tử' trên một máy chậm"* — nhưng nó mua vế đó bằng một sự phụ
  // thuộc mà **tám trong mười hai spec không cần**: nút ấy nằm trong dải chip nguồn, và
  // dải chỉ render khi `dictSources.length > 0`, tức khi `src-tauri/target/debug/dict/*.db`
  // có mặt. Trên máy Ice thư mục đó **356 MB**, do `tools/dict-build` sinh ra; CI không có
  // bước nào dựng nó và AC cuối của Story 1.3 cấm CI tải dữ liệu từ điển.
  // ⇒ Lượt CI đầu tiên (run 32393425715, 2026-08-20): **3 xanh / 9 đỏ**, tám lượt đỏ chết
  // ở ĐÚNG dòng này sau 30 giây, không một spec nào chạm tới mệnh đề của mình.
  //
  // Mốc mới là **lưới đã nạp ít nhất một hàng** — mạnh HƠN cho việc mọi chỗ gọi thật sự
  // cần (không spec nào dùng fixture này mà không đọc lưới), và độc lập với từ điển.
  // ⚠️ *"Ít nhất một"*, không *"đúng N"*: fixture nhận `text` tuỳ chọn nên số segment khác
  // nhau theo chỗ gọi. Spec nào cần một con số CHÍNH XÁC vẫn gọi `waitForGridRows(n)` của
  // `support/gridWait.mjs` như trước — mốc này không thay nó, nó chỉ dựng tiền đề.
  //
  // ⚠️ Số trong câu báo lỗi đọc SAU vòng chờ — cùng luật `support/gridWait.mjs`.
  let seenRows = null
  try {
    await browser.waitUntil(
      async () => {
        seenRows = await browser.execute(
          () => document.querySelectorAll('[data-col="src"]').length,
        )
        return seenRows > 0
      },
      { timeout: 30_000, interval: 250 },
    )
  } catch {
    throw new Error(
      `Vào \`workspace\` rồi mà lưới KHÔNG nạp một hàng nào sau 30 giây ` +
        `(lần đọc cuối: ${seenRows === null ? 'chưa đọc được lần nào' : seenRows} ô ` +
        `\`[data-col="src"]\`).\n\n` +
        'Đây là lỗi HẠ TẦNG của bàn đo — Tác phẩm đã tạo qua IPC nhưng lượt nạp lưới\n' +
        'không tới nơi. Nghi phạm theo thứ tự: `resetPanelState()` không phát lại lượt\n' +
        'nạp (xem `support/panelReset.mjs`), hoặc `Mod+2` không vào được `workspace`.\n' +
        'KHÔNG một hồi quy sản phẩm; đừng đọc ca đỏ phía sau nó thành một khuyết tật.',
    )
  }
}
