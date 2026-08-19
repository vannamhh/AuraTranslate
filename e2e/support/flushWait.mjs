/**
 * Chờ một **SỰ KIỆN/trạng thái quan sát được**, không chờ một khoảng thời gian — **AC4 của
 * Story 2.12**, và nguyên văn món ④ của action item B2: *"chờ một SỰ KIỆN, không chờ một
 * khoảng thời gian — không nới hằng số"*.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO `pause(FLUSH_WAIT_MS)` LÀ MỘT BỘ ĐO NÓI DỐI
 * ═════════════════════════════════════════════════════════════════════════════════
 * `FLUSH_WAIT_MS = 3.500` so với `EDITOR_IDLE_MS = 2.000` cho biên đúng **1.500 ms**. Trong
 * biên ấy phải lọt: bộ hẹn idle + một lượt `invoke` + `Store::write` **nối tiếp** của AD-11
 * + một lượt `fsync` WAL. **Một máy đang biên dịch Rust ăn hết biên đó.** Phân xử bằng năm
 * lượt trọn bộ ở Story 2.7: máy **bận** cho 7/8 ba lần liên tiếp, máy **rảnh** cho 8/8.
 *
 * ⇒ Kết quả xanh/đỏ phụ thuộc **tải máy**, không phụ thuộc sản phẩm. Và đường vá rẻ —
 * nới `FLUSH_WAIT_MS` — bị B2 cấm đích danh, đúng lý do: nó làm mọi ca xanh **và** làm NFR18
 * hết được canh. Một cổng cho exit 0 trên một sản phẩm đang hỏng.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 ICE KÝ ĐƯỜNG (a′) — 2026-08-18, và nó KHÔNG mở cửa `AD`
 * ═════════════════════════════════════════════════════════════════════════════════
 * Ba đường trình ở Task 0 là: **(a)** khớp chuỗi *"Đã lưu N giây trước"* của `StatusBar`;
 * **(b)** thêm một `data-*`; **(c)** một sự kiện `window` phát khi flush vào WAL. Ice ký
 * **(a)**, loại (b) và (c) vì cả hai **thêm một bề mặt quan sát được mới** — đúng hình dạng
 * AD-2 mô tả cho một `AD` mới.
 *
 * Lượt dựng Task 2 lòi ra một biến thể mà lúc ký **chưa có trên bàn**, và Ice ký lại
 * **(a′)**: `editorLastSavedAt` *(`src/panels/editorPanelState.ts:246`)* **đã là một export
 * công khai** — `StatusBar.vue` đọc chính nó để dựng câu kia — nên nó đọc được qua đúng cầu
 * `import()` mà `support/panelReset.mjs` đã đo là sạch. **0 dòng mã sản phẩm mới**, y như
 * (a), và mất luôn cái giá duy nhất của (a): bộ đo không còn bám `vi.json`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * ⚠️ BẪY "ĐÃ CÓ SẴN" — vì sao phải CHỤP MỐC TRƯỚC, và nó là một bẫy ĐO ĐƯỢC
 * ═════════════════════════════════════════════════════════════════════════════════
 * Mọi spec trong một tệp dùng **chung một phiên app**. Ca thứ nhất của
 * `editor-typing-flush` đã flush, nên khi ca thứ hai bắt đầu thì `editorLastSavedAt`
 * **khác `null`** và `footer.status` **đã** mang câu *"Đã lưu N giây trước"*.
 *
 * ⇒ Một phép chờ *"tới khi có mốc lưu"* trả về **NGAY LẬP TỨC** ở ca thứ hai, trước lượt
 * flush mới một khoảng không xác định. Nó là một lượt xanh **không đo gì cả** — cùng hạng
 * với `pause()`, chỉ nhanh hơn.
 *
 * Nên hợp đồng của tệp này là **hai bước, không một**: [`markFlushBaseline`] trước khi gõ,
 * [`waitForFlushAfter`] sau khi gõ. Chờ một mốc **MỚI HƠN** mốc đã chụp là mệnh đề duy nhất
 * không thể thoả bằng quá khứ.
 */

/**
 * Trần chờ một lượt flush.
 *
 * 🔴 **30 giây, và nó KHÔNG phải `FLUSH_WAIT_MS` được nới.** Hai con số trả lời hai câu khác
 * hẳn nhau, và đó là cả điểm của AC4:
 *   · `FLUSH_WAIT_MS` là *"tôi tin flush xong sau chừng này"* — một lời **đoán** về nhịp
 *     AD-35, và nó **quyết định kết quả**: hết giờ là ca đọc đĩa cũ rồi đỏ.
 *   · Số này là *"quá chừng này thì có gì đó hỏng thật"* — một trần **an toàn**, và nó
 *     **không** quyết định kết quả: ca xanh ngay khi mốc đổi, dù đó là mili-giây thứ 200 hay
 *     thứ 12.000. Một máy bận chậm gấp năm lần vẫn cho **cùng một phán quyết**.
 *
 * ⇒ Nới số này không làm một ca đỏ thành xanh; nới `FLUSH_WAIT_MS` thì có. Đó là phép thử
 * phân biệt một trần an toàn với một ngưỡng bị nới.
 */
const FLUSH_TIMEOUT_MS = 30_000

/** Module mang mốc lưu. Cùng đường `import()` mà `support/panelReset.mjs` đã đo là sạch. */
const EDITOR_STATE = '/src/panels/editorPanelState.ts'

/**
 * Đọc `editorLastSavedAt` trong trang.
 *
 * @returns {Promise<number | null>}
 */
async function readLastSavedAt() {
  const out = await browser.execute(async (path) => {
    try {
      const mod = await import(/* @vite-ignore */ path)
      const ref = mod.editorLastSavedAt
      if (ref === undefined) return { ok: false, detail: `"${path}" không xuất \`editorLastSavedAt\`` }
      // 🔴 `.value`, và ô này là một `readonly(ref)` nên đọc được mà không ghi được. Đúng
      // thứ một bộ đo cần: quan sát mà không can thiệp.
      return { ok: true, value: ref.value }
    } catch (err) {
      return { ok: false, detail: String(err) }
    }
  }, EDITOR_STATE)

  if (!out.ok) {
    throw new Error(
      `Không đọc được mốc lưu của Editor: ${out.detail}\n\n` +
        '🔴 Lỗi HẠ TẦNG của bàn đo. `editorLastSavedAt` là dây giữa `StatusBar.vue` và\n' +
        '`editorPanelState.ts`; một lượt đổi tên export làm gãy cả hai, và cầu này kêu TRƯỚC.',
    )
  }
  return out.value
}

/**
 * Chụp mốc lưu hiện tại. Gọi **TRƯỚC** thao tác sẽ gây ra lượt flush.
 *
 * @returns {Promise<number | null>}
 */
export function markFlushBaseline() {
  return readLastSavedAt()
}

/**
 * Chờ tới khi một lượt flush **MỚI HƠN** `baseline` đã hoàn tất.
 *
 * ⚠️ *"Hoàn tất"* ở đây nghĩa là **đã vào WAL**, không phải *"đã vào hàng đợi"* — AD-35 vế
 * cuối. Mốc chỉ được đặt sau khi lượt `invoke` trả về, tức sau khi `store::Writer` nối tiếp
 * đã ghi xong. Đó chính là lý do chờ **mốc** đúng nghĩa hơn chờ **thời gian**.
 *
 * @param {number | null} baseline giá trị [`markFlushBaseline`] trả về
 * @param {{ timeout?: number, what?: string }} [opts]
 * @returns {Promise<number>} mốc mới
 */
export async function waitForFlushAfter(baseline, opts = {}) {
  const timeout = opts.timeout ?? FLUSH_TIMEOUT_MS
  const what = opts.what ?? 'một lượt flush'

  // 🔴 Đọc SAU vòng chờ, không nội suy vào `timeoutMsg` — xem khối bên dưới.
  let seen = baseline
  let lastErr = null

  try {
    await browser.waitUntil(
      async () => {
        try {
          seen = await readLastSavedAt()
          lastErr = null
        } catch (err) {
          lastErr = err
          return false
        }
        return seen !== null && (baseline === null || seen > baseline)
      },
      { timeout, timeoutMsg: 'hết giờ' },
    )
  } catch {
    throw new Error(
      `${what} không hoàn tất sau ${timeout} ms.\n` +
        `Mốc trước thao tác: ${baseline === null ? '`null`' : baseline}\n` +
        `Mốc lần đọc cuối:  ${lastErr !== null ? '(không đọc được)' : seen === null ? '`null` (CHƯA có lượt flush nào)' : seen}\n` +
        (lastErr !== null
          ? `\n🔴 NGUYÊN VĂN lỗi mà vòng chờ nuốt:\n${lastErr instanceof Error ? (lastErr.stack ?? lastErr.message) : String(lastErr)}\n`
          : '') +
        '\n🔴 Đây KHÔNG phải một lượt hết giờ có thể vá bằng cách chờ lâu hơn — trần này là một\n' +
        'hàng rào an toàn, không một ngưỡng. Mốc KHÔNG đổi sau chừng đó nghĩa là nhịp AD-35\n' +
        'thật sự không chạy: idle 2 s và trần cứng 5 s đều đã trôi qua nhiều lần.\n' +
        'Ứng viên: bộ đệm gõ không nhận ký tự nào · `flushEditorNow` ném · IPC không trả về.',
    )
  }
  return seen
}

/**
 * 🔴 **VÌ SAO CÂU BÁO DỰNG TRONG `catch`, KHÔNG TRONG THAM SỐ — 2026-08-19.**
 *
 * `timeoutMsg` của `browser.waitUntil` là một **chuỗi**, dựng lúc tạo object tham số, tức
 * **TRƯỚC** khi vòng chờ chạy một lần nào. Một `${seen}` đặt ở đó in ra giá trị **khởi tạo**
 * ở mọi lượt đỏ, bất kể vòng chờ đã đọc được gì.
 *
 * ⚠️ Đo được ở lượt trọn bộ thứ mười *(2026-08-18)*: `support/gridWait.mjs` mắc đúng lỗi này
 * và ba ca đỏ đều báo *"lần đọc cuối thấy -1"* — `-1` là giá trị khởi tạo, **không** một giá
 * trị đọc được. Nó đẩy lượt chẩn đoán đi sai hướng ngay câu đầu tiên.
 *
 * ⇒ Và vế thứ hai cũng bắt buộc: `waitUntil` coi một lượt **ném** trong hàm điều kiện là
 * *"chưa đúng"* rồi **nuốt** nó. Không giữ lại nguyên văn thì một lỗi thật *(cầu IPC chết,
 * export đổi tên)* biến thành một lượt hết giờ câm, và câu báo sẽ đổ lỗi cho nhịp AD-35.
 */
