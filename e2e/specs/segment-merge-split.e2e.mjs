/**
 * Story 2.8 · AC1 · AC2 · AC8 — **gộp và tách đi trọn đường dây, trong WKWebView THẬT**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO CA NÀY PHẢI TỒN TẠI — nó là lưới DUY NHẤT cho một lớp lỗi ĐÃ LỌT HAI LẦN
 * ═════════════════════════════════════════════════════════════════════════════════
 * Hình dạng **dây** không có đường nghiệm thu nào khác, và kho này đã trả giá đúng hai lần:
 *
 * | Story | Thứ lọt | Ai bắt |
 * |---|---|---|
 * | 2.5 | `read_open_chapter_segments` không gửi cột `status` ⇒ `isConfirmed` LUÔN `false` trong app thật | **e2e** — 74/74 vitest vẫn xanh |
 * | 2.7 | `confirm_segment` đổi hình dạng tham số (`textAtLoad`) | **e2e** — 382 ca Rust + 133 ca vitest đều xanh |
 *
 * Cả hai lần, nguyên nhân giống hệt nhau: **fixture chép tay luôn có sẵn trường**. Một ca
 * vitest dựng `{ id, ord, source_text, … }` bằng tay không bao giờ phát hiện được rằng Rust
 * **không gửi** một trong số đó.
 *
 * Story 2.8 thêm hai lệnh với một hình dạng trả về **mới** (`RegroupOutcome`), và webview
 * **chèn thẳng** `new_segments` vào mảng nguồn sự thật của lưới. Một trường thiếu ở đó không
 * làm gì đỏ — nó làm một hàng trên màn hình mang `undefined` ở một cột.
 *
 * ⚠️ **Ca này đi ĐƯỜNG NGƯỜI DÙNG, không `invoke` thẳng** — khác spec của 2.7 *(spec đó gọi
 * `invoke('confirm_segment')` trực tiếp và vì thế đi vòng qua adapter)*. Đường phím phủ trọn
 * chuỗi: keymap → command → `editorPanelState` → adapter → Rust → vá ảnh chụp → lưới. Một
 * mắt xích đứt ở **bất kỳ** đâu đều lộ ra ở đây.
 *
 * ⚠️ **GIỚI HẠN THẬT:** `browser.keys()` chỉ bắn `keydown`/`keyup` và **không** đi vào đường
 * nhập văn bản gốc của WKWebView *(đo 2026-08-12)*. Điều đó **không** chạm ca này — một hợp
 * âm phím tắt **là** một `keydown`, và đó chính xác là thứ `CommandRegistry` nghe. Vế bộ gõ
 * tiếng Việt thì không đường nào của dự án mô phỏng được; nó không thuộc phạm vi story này.
 */
import { waitForGridRows, waitForGridText } from '../support/gridWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

/** Ảnh chụp lưới: mỗi hàng một bộ trường, đọc THẲNG từ DOM người dùng nhìn thấy. */
async function chupLuoi() {
  return await browser.execute(() => {
    const src = [...document.querySelectorAll('[data-col="src"]')]
    return {
      soHang: src.length,
      hang: src.map((el) => ({
        id: el.getAttribute('data-segment-id'),
        nguyenVan: el.textContent,
      })),
      // 🔵 2026-08-17 — chữ ký #6(b) ĐÃ BỊ LẬT sau một lượt Ice dùng thật: hàng về hưu
      // **không** ở lại lưới nữa. Trường này vì thế đổi vai — nó thôi là *"đếm cái phải
      // có"* và thành *"chốt canh cái KHÔNG được có"*: một số khác 0 nghĩa là bộ lọc
      // `retired_at IS NULL` đã bị gỡ ở đâu đó, và triệu chứng ở người dùng là nguyên văn
      // báo cáo hôm đó — *"câu cũ vẫn tồn tại và số thứ tự vẫn chiếm, gây rối nội dung"*.
      soVachVeHuu: document.querySelectorAll('.rule-ornament').length,
      // Số thứ tự NGƯỜI DÙNG NHÌN THẤY, đọc thẳng từ cột số — không suy từ chỉ số mảng.
      soThuTu: [...document.querySelectorAll('.cell-num')].map((el) => el.textContent?.trim()),
    }
  })
}

describe('Story 2.8 — gộp và tách segment trong WKWebView thật', () => {
  it('⌘M gộp hai câu: hai hàng cũ biến khỏi lưới, một hàng mới mang trọn hình dạng dây', async () => {
    // 🔴 Ba câu tiếng Trung, không chuỗi mặc định một câu: gộp cần **ít nhất hai** segment,
    // và bộ tách nhìn `。` vì Tác phẩm khai `sourceLang: 'zh'`.
    await openWorkspaceWithWork('Story 2.8 — gộp', '一。二。三。')

    // 🔵 **STORY 2.12 · AC2 — lượt `reload()` ở đây ĐÃ GỠ 2026-08-18.**
    // Fixture `openWorkspaceWithWork` nay tự dọn state panel bằng `resetPanelState()` (cầu
    // `import()` gọi thẳng năm hàm `reset*`, quyết định #5(b) Ice ký). Vá tại chỗ hết việc.
    // 🔵 **STORY 2.12 · AC3** — chờ TRẠNG THÁI ĐÍCH, không chờ *"phần tử tồn tại"*.
    // `waitForExist` không phân biệt *"Chương MỚI đã nạp"* với *"Chương CŨ còn đó"* — hai
    // trạng thái cùng hình dạng DOM. Vế đếm hàng dưới đây nay là **tiền đề đã được chờ**,
    // nên `expect(truoc.soHang)` cũ đã thành một phép kiểm không bao giờ đỏ được và bị gỡ.
    await waitForGridRows(3, { what: 'Tác phẩm "Story 2.8 — gộp"' })
    await waitForGridText(0, '一。')

    const truoc = await chupLuoi()
    await expect(truoc.soVachVeHuu).toBe(0)

    // ── ① Đặt caret vào ô bản dịch của câu THỨ HAI ────────────────────────────────
    //
    // 🔴 `realClick`, KHÔNG `.click()` của driver — lệnh `click` bắn `click` trước `focusin`,
    // ngược chuột thật; cưỡng chế bằng `no-restricted-syntax`.
    const idHai = truoc.hang[1].id
    await realClick(await $(`[data-col="tgt"][data-segment-id="${idHai}"]`))
    await browser.pause(250)

    // ── ② `⌘M` — AC8: một command ĐÃ ĐĂNG KÝ, không một hệ quả phụ của việc gõ ────
    await browser.keys(['Meta', 'm'])
    // Lượt này đi qua: flush tập chờ (hai vòng) → IPC → `Store::write` NỐI TIẾP của AD-11 →
    // fsync WAL → vá ảnh chụp → Vue render. Đợi rộng tay, vì câu hỏi là *"có xảy ra không"*
    // chứ không *"nhanh không"* — một ngưỡng chặt ở đây đo bàn đo chứ không đo sản phẩm.
    await browser.waitUntil(async () => (await chupLuoi()).soHang === 2, {
      timeout: 15_000,
      timeoutMsg:
        'Sau `⌘M`, lưới không rút xuống HAI hàng sau 15 giây.\n' +
        'Ba ứng viên, theo thứ tự rẻ dần: ① command chưa đăng ký / hợp âm không phân giải\n' +
        '(xem `check:commands`); ② lệnh Rust từ chối — đọc `editorRegroupError`;\n' +
        '③ `applyRegroup` không vá ảnh chụp.',
    })

    const sau = await chupLuoi()

    // 🔴 **AC1 + lượt LẬT chữ ký #6(b), cùng lúc:** hai hàng cũ về hưu và **BIẾN KHỎI LƯỚI**,
    // một hàng mới thế chỗ ⇒ ba hàng thành **HAI**. Trước lượt lật, chỗ này khẳng định `4`.
    await expect(sau.soHang).toBe(2)
    await expect(sau.soVachVeHuu).toBe(0)
    // 🔴 Và SỐ THỨ TỰ đọc lại liên tục từ 1 — đúng vế thứ hai của báo cáo 2026-08-17.
    await expect(sau.soThuTu).toEqual(['1', '2'])

    // ── ③ Hàng MỚI mang trọn hình dạng dây ────────────────────────────────────────
    //
    // 🔴 Đây là mệnh đề mà cả 391 ca Rust lẫn 138 ca vitest đều **mù**: chúng kiểm dữ liệu
    // Rust *trả về*, không kiểm dữ liệu webview *nhận được rồi render*.
    const idCu = new Set(truoc.hang.map((h) => h.id))
    const moi = sau.hang.filter((h) => !idCu.has(h.id))
    await expect(moi.length).toBe(1)
    await expect(moi[0].nguyenVan).toBe(
      '一。二。',
      // Hai câu tiếng Trung nối nhau KHÔNG khoảng trắng — chữ ký #3(b). Một chuỗi
      // `"一。 二。"` ở đây nghĩa là dấu nối đi nhánh sai.
    )

    // Và hàng mới đứng **đúng chỗ** nhóm cũ đang đứng, không bị đẩy xuống cuối lưới.
    const viTriMoi = sau.hang.findIndex((h) => h.id === moi[0].id)
    await expect(viTriMoi).toBe(0)
  })

  it('⌘/ tách một câu tại chỗ vừa bấm ở CỘT NGUYÊN VĂN', async () => {
    // 🔴 **Câu đầu DÀI có chủ ý — 18 ký tự, không 5.** Bộ đo chỉ giao được một cú bấm khi
    // đích là `origin: element` **trần**, tức **tâm ô** *(đo 2026-08-17: `origin` + một lệch
    // toạ độ, và cả toạ độ tuyệt đối, đều cho **0** sự kiện chuột — xem
    // `2-8-ban-do/tach-chan-doan.e2e.mjs`)*. Ô rộng 239 px, nên một câu ngắn để tâm ô rơi vào
    // **khoảng trống sau chữ** ⇒ chỗ cắt = cuối ô ⇒ Rust từ chối bằng
    // `segment.cut_leaves_empty_piece`, **đúng hành vi sản phẩm**. Một câu đủ dài đưa tâm ô
    // vào giữa chữ.
    await openWorkspaceWithWork('Story 2.8 — tách', '一二三四五六七八九十甲乙丙丁戊己庚辛。五。')

    // 🔵 **STORY 2.12 · AC2 — lượt `reload()` ở đây ĐÃ GỠ 2026-08-18.**
    // Fixture `openWorkspaceWithWork` nay tự dọn state panel bằng `resetPanelState()` (cầu
    // `import()` gọi thẳng năm hàm `reset*`, quyết định #5(b) Ice ký). Vá tại chỗ hết việc.
    // 🔵 **STORY 2.12 · AC3** — chờ TRẠNG THÁI ĐÍCH, không chờ *"phần tử tồn tại"*.
    // `waitForExist` không phân biệt *"Chương MỚI đã nạp"* với *"Chương CŨ còn đó"* — hai
    // trạng thái cùng hình dạng DOM. Vế đếm hàng dưới đây nay là **tiền đề đã được chờ**,
    // nên `expect(truoc.soHang)` cũ đã thành một phép kiểm không bao giờ đỏ được và bị gỡ.
    await waitForGridRows(2, { what: 'Tác phẩm tách' })

    const truoc = await chupLuoi()

    // ── ① Bấm vào GIỮA ô nguyên văn của câu đầu ───────────────────────────────────
    //
    // 🔴 Đây là đường (e) của Quyết định #2 chạy thật: WKWebView **không** tạo vùng chọn nào
    // ở cột nguyên văn *(đo ở `2-8-ban-do/`: `selectionType = "None"`, `rangeCount = 0` cho
    // cả bấm lẫn kéo)*, nên sản phẩm phải **tự** đổi toạ độ cú bấm thành một chỗ cắt.
    // ⇒ Nếu `onSourceCellMouseUp` chết, ca này đỏ ở bước `⌘/` với `'no-cut'`.
    // 🔴 **SỰ KIỆN CHUỘT TỔNG HỢP, và đây là GIỚI HẠN THỨ HAI của bộ đo — đo được, không
    // suy.** Ba cách nhắm đều cho **0** `mouseup` tới `document` khi đích là một ô
    // `[data-col="src"]`: toạ độ tuyệt đối · `origin` + lệch · `origin` trần
    // (`2-8-ban-do/tach-chan-doan.e2e.mjs`, ba vòng ngày 2026-08-17). Cùng lệnh đó trên ô
    // `[data-col="tgt"]` thì **ăn** — ca gộp ở trên chạy trọn đường chuột thật.
    //
    // ⇒ Sự kiện dưới đây thay **đúng một** mắt xích *(ai giao cú bấm)* và giữ nguyên phần
    // còn lại: `onSourceCellMouseUp` → `caretPositionFromPoint` **thật của engine** →
    // `sourceCutOffsetOf` → `setEditorSourceCut`. Toạ độ lấy từ hộp chữ **thật** trên màn
    // hình, không một con số dựng tay.
    //
    // ⚠️ **Vế KHÔNG được phủ, ghi ra thay vì để người sau tưởng đã xét:** *"một cú bấm CHUỘT
    // THẬT vào cột nguyên văn có tới được `onSourceCellMouseUp` không"*. Bộ đo không trả lời
    // được — món nợ có chủ **Ice**, đóng bằng một lượt bấm tay. Xem `deferred-work.md`.
    const daBam = await browser.execute((id) => {
      const cell = document.querySelector(`[data-col="src"][data-segment-id="${id}"]`)
      const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
      let node = walker.nextNode()
      while (node !== null && node.data.length === 0) node = walker.nextNode()
      if (node === null) return { ok: false, lyDo: 'ô nguyên văn không có text node nào' }
      const range = document.createRange()
      range.selectNodeContents(node)
      // 🔴 **HỘP DÒNG ĐẦU, không hộp GỘP** — và đây là phép đo cuối của chuỗi chẩn đoán.
      // `getBoundingClientRect()` trả **hợp** của mọi dòng khi văn bản xuống dòng, nên một
      // điểm ở "45 % chiều rộng, giữa chiều cao" của hộp đó rơi vào **dòng thứ hai, sau chữ
      // cuối** — engine phân giải ra `offset = 19` trên một câu **19 ký tự**, tức CUỐI câu,
      // và Rust từ chối đúng luật bằng `segment.cut_leaves_empty_piece`.
      // ⇒ `getClientRects()[0]` là hộp của **dòng đầu**; 30 % chiều rộng của nó nằm chắc
      // trong chữ.
      const rects = range.getClientRects()
      const r = rects.length > 0 ? rects[0] : range.getBoundingClientRect()
      const x = Math.round(r.x + r.width * 0.3)
      const y = Math.round(r.y + r.height / 2)
      cell.dispatchEvent(
        new MouseEvent('mouseup', {
          clientX: x,
          clientY: y,
          bubbles: true,
          cancelable: true,
          // 🔵 2026-08-17 (Story 2.9, AC7) — cử chỉ đánh dấu chỗ cắt nay đòi `Mod`.
          // Một cú bấm TRƠN không đánh dấu gì nữa: cột này dùng chung `mouseup` với
          // Auto-Lookup (FR21), và mỗi lượt tra một từ để ĐỌC cũng rơi một dấu cắt.
          // ⚠️ `metaKey`, viết thẳng, vì runner LÀ macOS. Vế lái-hai-nền-tảng nằm ở
          //    `hasPrimaryModifier` và có `editorSourceCutGesture.test.ts` phủ cả hai ca —
          //    một bộ e2e chỉ chạy trên macOS không phải chỗ diễn đạt mệnh đề đó.
          metaKey: true,
        }),
      )
      return { ok: true, x, y, chu: node.data }
    }, truoc.hang[0].id)
    await expect(daBam.ok).toBe(true)
    await browser.pause(250)

    // ── ② `⌘/` ────────────────────────────────────────────────────────────────────
    //
    // 🔴 **SỰ KIỆN TỔNG HỢP, KHÔNG `browser.keys()` — và đây là một GIỚI HẠN CỦA BỘ ĐO đã
    // đo được, không một lối tắt.**
    //
    // Đo 2026-08-17 (`2-8-ban-do/tach-chan-doan.e2e.mjs`): `browser.keys(['Meta', '/'])` giao
    // một `keydown` mang **`code: "/"`**, trong khi bàn phím thật trên WebKit mang
    // **`code: "Slash"`**. Hợp âm `Mod+Slash` vì thế không khớp — `defaultPrevented: false`,
    // **0** command chạy. Đối chứng cùng lượt: `⌘M` giao `code: "KeyM"` và
    // `defaultPrevented: true` ⇒ khớp, và ca gộp ở trên **đi trọn đường phím thật**.
    //
    // ⇒ Sự kiện dưới đây thay **đúng một** mắt xích *(ai sinh ra `keydown`)* và giữ nguyên cả
    // chuỗi còn lại: keymap → command → `editorPanelState` → adapter → Rust → vá ảnh chụp →
    // lưới. Đo được rằng nó chạy: cùng lượt bàn đo, sự kiện này làm sản phẩm ghi
    // `[grid] khong tach duoc segment: no-cut` — tức lệnh **đã tới nơi**.
    //
    // 🔴 **MỘT CÂU HỎI CÒN MỞ, KHÔNG ĐÓNG ĐƯỢC BẰNG BỘ ĐO NÀY:** nếu WKWebView **thật** cũng
    // báo `code: "/"` cho phím gạch chéo thì `⌘/` là một **phím tắt chết im lặng** trong sản
    // phẩm. Bộ đo không phân biệt được hai khả năng đó — theo cấu tạo. Món nợ có chủ **Ice**,
    // đóng bằng một lượt gõ tay. Xem `deferred-work.md`.
    await browser.execute(() => {
      document.dispatchEvent(
        new KeyboardEvent('keydown', {
          key: '/',
          code: 'Slash',
          metaKey: true,
          bubbles: true,
          cancelable: true,
        }),
      )
    })
    await browser.waitUntil(async () => (await chupLuoi()).soHang === 3, {
      timeout: 15_000,
      timeoutMsg:
        'Sau `⌘/`, lưới không lên BA hàng sau 15 giây.\n' +
        'Ứng viên thêm so với ca gộp: chỗ cắt chưa được ghi nhận — `onSourceCellMouseUp`\n' +
        'không phân giải được điểm bấm, hoặc `⌘/` chạy trước khi nó chạy.',
    })

    const sau = await chupLuoi()
    // Một hàng về hưu **biến khỏi lưới**, HAI mảnh mới thế chỗ ⇒ 2 thành **3**.
    await expect(sau.soHang).toBe(3)
    await expect(sau.soVachVeHuu).toBe(0)
    await expect(sau.soThuTu).toEqual(['1', '2', '3'])

    const idCu = new Set(truoc.hang.map((h) => h.id))
    const manh = sau.hang.filter((h) => !idCu.has(h.id))
    await expect(manh.length).toBe(2)

    // 🔴 Hai mảnh nối lại phải bằng ĐÚNG văn bản gốc — không một ký tự rơi, không một ký tự
    // thêm. Đây là mệnh đề mà một lượt lấy `offset` trần (thay vì cộng dồn text node) vẫn
    // giữ được; nó canh vế **không mất chữ**, còn vế **đúng chỗ** thuộc vitest ca ③.
    await expect(manh[0].nguyenVan + manh[1].nguyenVan).toBe(truoc.hang[0].nguyenVan)
    await expect(manh[0].nguyenVan.length).toBeGreaterThan(0)
    await expect(manh[1].nguyenVan.length).toBeGreaterThan(0)
  })

  /**
   * 🔴 **HAI điểm cắt trong MỘT lượt `⌘/` ⇒ BA mảnh** — AC7 vế *"nhiều mảnh"*, chữ ký của Ice
   * ngày 2026-08-17 sau code review.
   *
   * 🔴 **Vì sao ca này BẮT BUỘC ở e2e và không ở đâu khác:** lượt đa-mảnh **đổi hình dạng
   * dây** — `cut: number` thành `cuts: number[]`. Kho này đã để lọt đúng lớp lỗi ấy **hai
   * lần** *(cột `status` ở Story 2.5, tham số `textAtLoad` ở Story 2.7)*, và cả hai lần
   * **toàn bộ** test Rust cộng vitest đều xanh, vì fixture chép tay luôn có sẵn trường. Chỉ
   * một lượt `invoke` thật qua `tauri-macros` mới phát hiện một tên tham số sai.
   *
   * ⚠️ Cùng **hai** giới hạn của bộ đo như ca một-điểm ở trên *(cú bấm tổng hợp vào cột
   * nguyên văn · `code: "Slash"`)*, cùng lý do, và cùng hai món nợ có chủ. Ca này **không**
   * thêm một giới hạn nào — nó chỉ bấm hai lần thay vì một.
   */
  it('⌘/ tách BA mảnh từ HAI điểm cắt tích luỹ, trong một lượt ghi', async () => {
    // 20 ký tự ở câu đầu — đủ dài để hai điểm ở 25 % và 60 % của hộp dòng đầu rơi vào chữ và
    // cách nhau chắc chắn hơn một ký tự.
    await openWorkspaceWithWork(
      'Story 2.8 — tách ba mảnh',
      '一二三四五六七八九十甲乙丙丁戊己庚辛壬癸。五。',
    )

    // 🔵 **STORY 2.12 · AC2 — lượt `reload()` ở đây ĐÃ GỠ 2026-08-18.**
    // Fixture `openWorkspaceWithWork` nay tự dọn state panel bằng `resetPanelState()` (cầu
    // `import()` gọi thẳng năm hàm `reset*`, quyết định #5(b) Ice ký). Vá tại chỗ hết việc.
    // 🔵 **STORY 2.12 · AC3** — chờ TRẠNG THÁI ĐÍCH, không chờ *"phần tử tồn tại"*.
    // `waitForExist` không phân biệt *"Chương MỚI đã nạp"* với *"Chương CŨ còn đó"* — hai
    // trạng thái cùng hình dạng DOM. Vế đếm hàng dưới đây nay là **tiền đề đã được chờ**,
    // nên `expect(truoc.soHang)` cũ đã thành một phép kiểm không bao giờ đỏ được và bị gỡ.
    await waitForGridRows(2, { what: 'Tác phẩm tách' })

    const truoc = await chupLuoi()

    // ── ① HAI cú bấm, hai vị trí khác nhau, cùng một ô ────────────────────────────
    //
    // 🔴 Mỗi cú bấm **thêm** một điểm vào tập — đó là cơ chế tích luỹ mà Ice ký. Ca này đo
    // chính mệnh đề đó: nếu cú bấm thứ hai **thay** cú thứ nhất *(hành vi của bản một-điểm)*
    // thì lưới lên **3** hàng, không 4, và ca đỏ ngay ở `waitUntil`.
    const daBam = await browser.execute((id) => {
      const cell = document.querySelector(`[data-col="src"][data-segment-id="${id}"]`)
      const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
      let node = walker.nextNode()
      while (node !== null && node.data.length === 0) node = walker.nextNode()
      if (node === null) return { ok: false, lyDo: 'ô nguyên văn không có text node nào' }
      const range = document.createRange()
      range.selectNodeContents(node)
      // Cùng lý do `getClientRects()[0]` với ca một-điểm: hộp GỘP của văn bản xuống dòng đưa
      // một tỷ lệ chiều rộng vào **dòng thứ hai**, tức sau chữ cuối.
      const rects = range.getClientRects()
      const r = rects.length > 0 ? rects[0] : range.getBoundingClientRect()
      const y = Math.round(r.y + r.height / 2)
      const diem = [0.25, 0.6].map((ty) => Math.round(r.x + r.width * ty))
      for (const x of diem) {
        cell.dispatchEvent(
          new MouseEvent('mouseup', {
          clientX: x,
          clientY: y,
          bubbles: true,
          cancelable: true,
          // 🔵 2026-08-17 (Story 2.9, AC7) — cử chỉ đánh dấu chỗ cắt nay đòi `Mod`.
          // Một cú bấm TRƠN không đánh dấu gì nữa: cột này dùng chung `mouseup` với
          // Auto-Lookup (FR21), và mỗi lượt tra một từ để ĐỌC cũng rơi một dấu cắt.
          // ⚠️ `metaKey`, viết thẳng, vì runner LÀ macOS. Vế lái-hai-nền-tảng nằm ở
          //    `hasPrimaryModifier` và có `editorSourceCutGesture.test.ts` phủ cả hai ca —
          //    một bộ e2e chỉ chạy trên macOS không phải chỗ diễn đạt mệnh đề đó.
          metaKey: true,
        }),
        )
      }
      return { ok: true, diem, chu: node.data }
    }, truoc.hang[0].id)
    await expect(daBam.ok).toBe(true)
    await browser.pause(250)

    // 🔴 **Kênh thị giác phải THẤY ĐƯỢC hai điểm** — một trạng thái giữa hai thao tác mà
    // người dùng không nhìn thấy là đúng lớp *"im lặng"* mà `project-context.md` cấm. Đọc
    // thẳng thuộc tính sản phẩm ghi ra, không suy từ một biến nội bộ.
    const soDiem = await browser.execute((id) => {
      const cell = document.querySelector(`[data-col="src"][data-segment-id="${id}"]`)
      return {
        dem: cell?.getAttribute('data-cut-count') ?? null,
        // 🔵 2026-08-17 — trường `coVien` ĐÃ GỠ cùng lớp `has-cuts` (Ice chốt: *"bỏ dấu gạch
        // đứng ở trước câu đi, nó không cần thiết"*). Lớp đó là một kênh THAY THẾ cho ngày
        // dấu cắt chưa vẽ được ở tab Hán Việt; Story 2.9 · AC9 làm nó vẽ được ⇒ kênh hết việc.
        // Mệnh đề *"kênh thị giác phải THẤY ĐƯỢC hai điểm"* không đổi một chữ — `soDauCat` là
        // vế mang nó, và nó CHẶT hơn một cờ.
        soDauCat: cell?.querySelectorAll('.cut-mark').length ?? -1,
      }
    }, truoc.hang[0].id)
    await expect(soDiem.dem).toBe('2')
    await expect(soDiem.soDauCat).toBe(2)

    // ── ② MỘT lượt `⌘/` ───────────────────────────────────────────────────────────
    await browser.execute(() => {
      document.dispatchEvent(
        new KeyboardEvent('keydown', {
          key: '/',
          code: 'Slash',
          metaKey: true,
          bubbles: true,
          cancelable: true,
        }),
      )
    })
    await browser.waitUntil(async () => (await chupLuoi()).soHang === 4, {
      timeout: 15_000,
      timeoutMsg:
        'Sau MỘT lượt `⌘/` với hai điểm cắt, lưới không lên BỐN hàng sau 15 giây.\n' +
        'Ứng viên ĐẦU TIÊN phải loại: tham số trên dây. `cuts` là một MẢNG kể từ 2026-08-17;\n' +
        'một tên sai hay một kiểu sai ở đó là đúng lớp lỗi đã lọt hai lần (`status` 2.5,\n' +
        '`textAtLoad` 2.7) mà toàn bộ test Rust + vitest đều mù.\n' +
        'Ứng viên thứ hai: cú bấm thứ hai THAY vì THÊM ⇒ lưới lên 3, không 4.',
    })

    const sau = await chupLuoi()
    await expect(sau.soHang).toBe(4)
    await expect(sau.soVachVeHuu).toBe(0)
    await expect(sau.soThuTu).toEqual(['1', '2', '3', '4'])

    const idCu = new Set(truoc.hang.map((h) => h.id))
    const manh = sau.hang.filter((h) => !idCu.has(h.id))
    await expect(manh.length).toBe(3)

    // 🔴 **MỘT lượt về hưu, không hai.** Đây là mệnh đề phân biệt lượt đa-mảnh thật với hai
    // lượt `⌘/` nối nhau — đường (c) của Quyết định #1, thứ đã bị bác vì nó cho **5** hàng về
    // hưu thay vì 3 cộng một segment trung gian mang một `id` không ai từng thấy.
    await expect(manh.map((m) => m.nguyenVan).join('')).toBe(truoc.hang[0].nguyenVan)
    for (const m of manh) await expect(m.nguyenVan.length).toBeGreaterThan(0)

    // Tập điểm cắt đã tiêu thụ ⇒ chết cùng lượt, và ô không còn dấu nào.
    const conDau = await browser.execute(
      () => document.querySelectorAll('[data-col="src"] .cut-mark').length,
    )
    await expect(conDau).toBe(0)
  })
})
