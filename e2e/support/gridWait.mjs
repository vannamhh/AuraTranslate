/**
 * Chờ **trạng thái ĐÍCH** của lưới, không chờ *"phần tử tồn tại"* — **AC3 của Story 2.12**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 KHUYẾT TẬT NÀY ĐÃ TRẢ GIÁ ĐỂ BIẾT — 2026-08-17, Story 2.9
 * ═════════════════════════════════════════════════════════════════════════════════
 * `waitForExist('[data-col="src"]')` rồi đọc ngay **không phân biệt** *"Chương MỚI đã nạp"*
 * với *"Chương CŨ còn nằm đó"* — hai trạng thái có **cùng hình dạng DOM**, chỉ khác **số
 * lượng** và **nội dung**. Ca đầu của `segment-backspace-merge` gộp 3 hàng thành 2; ca sau
 * dựng một Tác phẩm mới rồi đọc ngay và thấy **2**. Nguyên văn lượt đỏ: `Expected: 3,
 * Received: 2` — một lượt đỏ nói về **bàn đo**, không về sản phẩm.
 *
 * Ba spec đã tự vá tại chỗ, mỗi chỗ một kiểu *(`segment-navigation:116` ·
 * `segment-backspace-merge:106` · `segment-merge-split:87`)*. Tệp này rút khuôn chung để
 * chỗ thứ tư không phải phát hiện lại cùng một điều.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * ⚠️ VÌ SAO CHỜ THẾ NÀY **KHÔNG** LÀ NỚI MỘT NGƯỠNG CHO HẾT ĐỎ
 * ═════════════════════════════════════════════════════════════════════════════════
 * Số hàng mong đợi ở đây là một **TIỀN ĐỀ** của ca — *"Tác phẩm tôi vừa dựng có N câu"* —
 * chứ không phải **mệnh đề đang kiểm**. Chờ một tiền đề thành hình là hợp lệ; chờ chính
 * mệnh đề mình sắp khẳng định thì là một ca không bao giờ đỏ được.
 *
 * 🔴 Ranh giới đó phải giữ ở mọi chỗ dùng: gọi hàm này với con số của **đầu vào fixture**,
 * đừng bao giờ gọi nó với con số mà ca sắp `expect`.
 *
 * ⚠️ **GIỚI HẠN THẬT:** *"đúng N hàng"* không chứng minh *"đúng Chương ĐÓ"* — hai Chương
 * khác nhau cùng có N câu vẫn khớp. Ca nào cần danh tính phải kèm một vế **nội dung**;
 * [`waitForGridText`] dưới đây là chỗ cho vế ấy. `segment-navigation` gọi đó là *"vế duy
 * nhất bắt được ca 2.9"*.
 */

/**
 * 🔴 **CHÍNH TỆP NÀY ĐÃ NÓI DỐI MỘT LẦN — đọc trước khi sửa nó, 2026-08-19.**
 *
 * Bản đầu đưa giá trị đọc được vào `timeoutMsg` của `browser.waitUntil`:
 *
 * ```js
 * let seen = -1
 * await browser.waitUntil(async () => { seen = await countGridCells(col); … },
 *                         { timeoutMsg: `… lần đọc cuối thấy ${seen}.` })
 * ```
 *
 * ⚠️ **`timeoutMsg` là một CHUỖI, dựng lúc tạo object tham số — tức TRƯỚC khi vòng chờ chạy
 * một lần nào.** `${seen}` bị nội suy khi `seen` vẫn còn là `-1`, và nó in `-1` **ở mọi lượt
 * đỏ, bất kể lưới thật có bao nhiêu hàng**.
 *
 * Hệ quả đo được ở lượt trọn bộ thứ mười *(2026-08-18, 5 passed / 6 failed)*: ba ca đỏ đều báo
 * *"lần đọc cuối thấy -1"*, và `-1` **không phải một giá trị đọc được** — nó là giá trị khởi
 * tạo. Một người đọc log *(kể cả người viết nó)* sẽ kết luận `browser.execute` ném mọi vòng,
 * và đi chẩn đoán sai hướng.
 *
 * 🔴 ⇒ Đây đúng lớp *"một cổng cho exit 0 trên một sản phẩm đang hỏng"*, chỉ đảo chiều: một
 * bộ đo cho một **câu chẩn đoán không có thật** trên một lượt đỏ thật. Và nó lọt qua vì
 * `timeoutMsg` **trông** như một template literal được đánh giá lười.
 *
 * ⇒ Luật của tệp này từ nay: **mọi con số trong một câu báo lỗi phải được đọc SAU khi lượt
 * chờ kết thúc**, tức câu báo dựng trong `catch`, không trong tham số. Và lỗi mà vòng chờ nuốt
 * phải được **giữ nguyên văn** rồi in kèm — `waitUntil` coi một lượt ném là *"chưa đúng"*, nên
 * không có vế này thì một lỗi thật biến thành một lượt hết giờ câm.
 */

/** Trần chờ mặc định. Bằng đúng số ba spec đã tự dùng — không nới, không siết. */
const DEFAULT_TIMEOUT_MS = 30_000

/**
 * Đếm ô của một cột trên lưới, trong trang.
 *
 * @param {'src' | 'tgt'} col
 * @returns {Promise<number>}
 */
export function countGridCells(col) {
  return browser.execute((c) => document.querySelectorAll(`[data-col="${c}"]`).length, col)
}

/**
 * Chờ tới khi lưới mang **đúng** `expected` hàng ở cột `col`.
 *
 * @param {number} expected số hàng của **đầu vào fixture** — xem ranh giới ở đầu tệp
 * @param {{ col?: 'src' | 'tgt', timeout?: number, what?: string }} [opts]
 */
export async function waitForGridRows(expected, opts = {}) {
  const col = opts.col ?? 'src'
  const timeout = opts.timeout ?? DEFAULT_TIMEOUT_MS
  const what = opts.what ?? 'lưới'

  // 🔴 Cả hai ô nhớ này chỉ đọc được SAU vòng chờ — xem khối lý do ở đầu tệp.
  let seen = null
  let lastErr = null

  try {
    await browser.waitUntil(
      async () => {
        try {
          seen = await countGridCells(col)
          lastErr = null
        } catch (err) {
          // ⚠️ `waitUntil` coi một lượt ném là *"chưa đúng"* và **nuốt nó**. Giữ nguyên văn
          // lại, nếu không một lỗi thật thành một lượt hết giờ câm.
          lastErr = err
          return false
        }
        return seen === expected
      },
      { timeout, timeoutMsg: 'hết giờ' },
    )
  } catch {
    throw new Error(
      `${what} không hiện đúng ${expected} hàng \`[data-col="${col}"]\` sau ${timeout} ms.\n` +
        `Lần đọc cuối: ${lastErr !== null ? '(không đọc được)' : seen}\n` +
        (lastErr !== null
          ? `\n🔴 NGUYÊN VĂN lỗi mà vòng chờ nuốt:\n${lastErr instanceof Error ? (lastErr.stack ?? lastErr.message) : String(lastErr)}\n`
          : '') +
        '\n🔴 Đây là một lỗi HẠ TẦNG của bàn đo, KHÔNG một hồi quy sản phẩm.\n' +
        'Ứng viên thường nhất, theo thứ tự đã gặp thật:\n' +
        '  ① Chương của Tác phẩm TRƯỚC còn nằm trên lưới — fixture không dọn state panel.\n' +
        '     (Đóng ở Story 2.12 bằng `support/panelReset.mjs`; nếu lại thấy, cầu reset đã hỏng.)\n' +
        '  ② Tác phẩm dựng ra ít/nhiều câu hơn ta tưởng — bộ tách nhìn `。！？；` cho `zh`,\n' +
        '     KHÔNG nhìn dấu chấm tiếng Anh. Một chuỗi tiếng Việt có `.` vẫn cho MỘT câu.\n' +
        '  ③ Lưới chưa render xong trên một máy đang bận.\n' +
        '  ④ Lưới KHÔNG BAO GIỜ nạp — đọc được 0 ở mọi vòng. Khác hẳn ba ca trên: nó nói\n' +
        '     Chương không tới được webview, không phải nó tới chậm.',
    )
  }
}

/**
 * Chờ tới khi ô thứ `index` của cột `col` mang đúng `text`.
 *
 * 🔴 Đây là vế **DANH TÍNH** mà [`waitForGridRows`] một mình không có. Dùng nó ở mọi chỗ hai
 * Chương có thể cùng số hàng — tức gần như mọi chỗ dựng một Tác phẩm thứ hai trong một spec.
 *
 * @param {number} index
 * @param {string} text nguyên văn mong đợi, đã `trim()`
 * @param {{ col?: 'src' | 'tgt', timeout?: number }} [opts]
 */
export async function waitForGridText(index, text, opts = {}) {
  const col = opts.col ?? 'src'
  const timeout = opts.timeout ?? DEFAULT_TIMEOUT_MS
  let seen = null
  let lastErr = null

  try {
    await browser.waitUntil(
      async () => {
        try {
          seen = await browser.execute(
            (c, i) => document.querySelectorAll(`[data-col="${c}"]`)[i]?.textContent?.trim() ?? null,
            col,
            index,
          )
          lastErr = null
        } catch (err) {
          lastErr = err
          return false
        }
        return seen === text
      },
      { timeout, timeoutMsg: 'hết giờ' },
    )
  } catch {
    throw new Error(
      `Ô ${index} của cột "${col}" không mang "${text}" sau ${timeout} ms.\n` +
        `Lần đọc cuối: ${lastErr !== null ? '(không đọc được)' : seen === null ? '(không có ô đó)' : `"${seen}"`}\n` +
        (lastErr !== null
          ? `\n🔴 NGUYÊN VĂN lỗi mà vòng chờ nuốt:\n${lastErr instanceof Error ? (lastErr.stack ?? lastErr.message) : String(lastErr)}\n`
          : '') +
        '\n🔴 Lỗi HẠ TẦNG của bàn đo. `null` nghĩa là lưới chưa đủ hàng; một chuỗi KHÁC nghĩa là\n' +
        'lưới đang hiện một Chương khác — đúng lớp lỗi mà đếm hàng một mình không bắt được.',
    )
  }
}
