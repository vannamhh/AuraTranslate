/**
 * Bàn đo Story 2.8 — **chẩn đoán vì sao `⌘/` không tách**, trong khi `⌘M` gộp được.
 *
 * Vòng chẩn đoán ① *(giả thuyết "cú bấm rơi sau chữ nên chỗ cắt = cuối ô")* đã bị phép đo
 * **bác**: dời cú bấm vào giữa hộp chữ, ca vẫn đỏ y nguyên.
 *
 * ⇒ Vòng này **đo** thay vì đoán tiếp. Bốn thứ được ghi lại, mỗi thứ loại một mắt xích:
 *   ① `keydown` có tới `document` không, và nó mang `code`/`key` gì — loại *"driver không
 *      gửi được `/` cùng `Meta`"*;
 *   ② `console.warn`/`error` của sản phẩm — `main.ts` ghi `[grid] khong tach duoc segment:
 *      <result>`, và chuỗi đó **nói thẳng** mắt xích nào đứt (`no-cut` · `refused` · …);
 *   ③ `mouseup` trên cột nguồn có chạy không, và `caretPositionFromPoint` phân giải ra gì;
 *   ④ đối chứng: `⌘M` trên cùng phiên — nếu nó cũng câm thì vấn đề là **bàn phím**, không
 *      phải đường tách.
 */
import { openWorkspaceWithWork } from '../../../e2e/support/workspace.mjs'

describe('Bàn đo 2.8 — chẩn đoán đường tách', () => {
  it('bốn mắt xích, đo từng cái', async () => {
    await openWorkspaceWithWork(
      'Bàn đo 2.8 — chẩn đoán tách',
      '一二三四五六七八九十甲乙丙丁戊己庚辛。五。',
    )
    await browser.execute(() => {
      window.location.reload()
    })
    await $('[data-col="src"]').waitForExist({ timeout: 30_000 })

    // Cài bộ ghi chép: console + keydown + mouseup.
    await browser.execute(() => {
      window.__t28 = { log: [], phim: [], chuot: [] }
      for (const muc of ['warn', 'error', 'info']) {
        const goc = console[muc].bind(console)
        console[muc] = (...args) => {
          window.__t28.log.push(`${muc}: ${args.map(String).join(' ')}`)
          goc(...args)
        }
      }
      document.addEventListener(
        'keydown',
        (e) => {
          window.__t28.phim.push({
            key: e.key,
            code: e.code,
            meta: e.metaKey,
            ctrl: e.ctrlKey,
            defaultPrevented: e.defaultPrevented,
            target: e.target?.getAttribute?.('data-col') ?? e.target?.tagName ?? null,
          })
        },
        true,
      )
      document.addEventListener(
        'mouseup',
        (e) => {
          const el = e.target instanceof Element ? e.target : e.target?.parentElement
          const cell = el?.closest('[data-col="src"]') ?? null
          // 🔴 **BẢN ĐẦU CỦA CHÍNH LISTENER NÀY LÀ MỘT KHUYẾT TẬT CỦA BÀN ĐO, và nó đã
          // nói dối suốt ba vòng.** Nó gọi `document.caretPositionFromPoint` **trần**; API đó
          // `undefined` trên WKWebView này nên lời gọi **NÉM** — trước dòng `push`. Kết quả:
          // `chuot` rỗng ở mọi vòng, và tôi đã đọc nó thành *"không một `mouseup` nào tới
          // `document`"*, một mệnh đề về **engine** dựng từ một cú ném của **bàn đo**.
          // ⇒ Cùng lớp bẫy mà vòng 1 đã mắc một lần: một con số thật, trả lời sai câu hỏi.
          const range =
            typeof document.caretRangeFromPoint === 'function'
              ? document.caretRangeFromPoint(e.clientX, e.clientY)
              : null
          window.__t28.chuot.push({
            trongCotNguon: cell !== null,
            segmentId: cell?.getAttribute('data-segment-id') ?? null,
            caretPhanGiaiDuoc: range !== null,
            offset: range?.startOffset ?? null,
            nodeLaText: range?.startContainer ? range.startContainer.nodeType === 3 : null,
            nodeTrongO: range?.startContainer && cell ? cell.contains(range.startContainer) : null,
          })
        },
        true,
      )
    })

    // ── ③ Bấm vào giữa hộp chữ của ô nguyên văn đầu tiên ──────────────────────────
    const hop = await browser.execute(() => {
      const cell = document.querySelector('[data-col="src"]')
      const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
      let node = walker.nextNode()
      while (node !== null && node.data.length === 0) node = walker.nextNode()
      const range = document.createRange()
      range.selectNodeContents(node)
      const r = range.getBoundingClientRect()
      return { x: r.x, y: r.y, w: r.width, h: r.height, chu: node.data }
    })
    console.log('\n[t28 · hộp chữ] ' + JSON.stringify(hop, null, 2))

    // ═══════════════════════════════════════════════════════════════════════════════
    // VÒNG 4 — Ice cấp phép một vòng nữa. Đo BA thứ, mỗi thứ cắt một nửa không gian còn lại
    // ═══════════════════════════════════════════════════════════════════════════════
    //
    // 🔴 Ứng viên MỚI, tìm ra lúc ĐỌC LẠI MÃ chứ không lúc chạy: `onSourceCellMouseUp` gọi
    // `document.caretPositionFromPoint(...)` **trần**. Nếu API đó **không tồn tại** trên
    // engine này thì đó là một `TypeError`, **không** một `null` — handler chết ngay dòng
    // đầu, không ghi được điểm cắt nào, và triệu chứng đúng là `no-cut`. WebKit lịch sử chỉ
    // có `caretRangeFromPoint`.
    const co = await browser.execute(() => ({
      caretPositionFromPoint: typeof document.caretPositionFromPoint,
      caretRangeFromPoint: typeof document.caretRangeFromPoint,
    }))
    console.log('\n[t28 · ⑦ hai API có tồn tại không] ' + JSON.stringify(co, null, 2))

    // Bắn `mouseup` TỔNG HỢP thẳng lên ô, toạ độ lấy từ hộp chữ thật. Bọc `try/catch` để một
    // `TypeError` trong handler lộ ra thay vì biến mất vào bộ điều phối sự kiện.
    const banChuot = await browser.execute(() => {
      const cell = document.querySelector('[data-col="src"]')
      const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
      let node = walker.nextNode()
      while (node !== null && node.data.length === 0) node = walker.nextNode()
      const range = document.createRange()
      range.selectNodeContents(node)
      const r = range.getBoundingClientRect()
      const x = Math.round(r.x + r.width * 0.45)
      const y = Math.round(r.y + r.height / 2)

      // Cùng toạ độ, hỏi thẳng engine — tách "API trượt" khỏi "handler trượt".
      let hoiThang = null
      try {
        const pos = document.caretPositionFromPoint(x, y)
        hoiThang = {
          duong: 'caretPositionFromPoint',
          co: pos !== null,
          offset: pos?.offset ?? null,
          trongO: pos?.offsetNode ? cell.contains(pos.offsetNode) : null,
        }
      } catch (err) {
        hoiThang = { duong: 'caretPositionFromPoint', nem: String(err) }
      }
      let duongHai = null
      try {
        const rr = document.caretRangeFromPoint(x, y)
        duongHai = {
          duong: 'caretRangeFromPoint',
          co: rr !== null,
          offset: rr?.startOffset ?? null,
          trongO: rr?.startContainer ? cell.contains(rr.startContainer) : null,
        }
      } catch (err) {
        duongHai = { duong: 'caretRangeFromPoint', nem: String(err) }
      }

      window.onerror = (msg) => {
        window.__t28.log.push(`onerror: ${String(msg)}`)
      }
      cell.dispatchEvent(
        new MouseEvent('mouseup', { clientX: x, clientY: y, bubbles: true, cancelable: true }),
      )
      return { x, y, chu: node.data, hoiThang, duongHai }
    })
    console.log('\n[t28 · ⑧ bắn mouseup tổng hợp] ' + JSON.stringify(banChuot, null, 2))
    await browser.pause(300)
    console.log(
      '\n[t28 · ⑨ console NGAY SAU mouseup] ' +
        JSON.stringify(await browser.execute(() => window.__t28.log), null, 2),
    )

    console.log(
      '\n[t28 · ③ mouseup ở cột nguồn] ' +
        JSON.stringify(await browser.execute(() => window.__t28.chuot), null, 2),
    )

    // ── ① + ② `⌘/` ───────────────────────────────────────────────────────────────
    await browser.execute(() => {
      window.__t28.phim = []
      window.__t28.log = []
    })
    await browser.keys(['Meta', '/'])
    await browser.pause(2500)
    console.log(
      '\n[t28 · ⑤ số hàng sau ⌘/ của DRIVER] ' +
        JSON.stringify(
          await browser.execute(() => ({
            soHang: document.querySelectorAll('[data-col="src"]').length,
            soVachVeHuu: document.querySelectorAll('.rule-ornament').length,
          })),
          null,
          2,
        ),
    )
    console.log(
      '\n[t28 · ① phím của ⌘/] ' +
        JSON.stringify(await browser.execute(() => window.__t28.phim), null, 2),
    )
    console.log(
      '\n[t28 · ② console sau ⌘/] ' +
        JSON.stringify(await browser.execute(() => window.__t28.log), null, 2),
    )

    // ── ⑥ Cùng hợp âm, nhưng `code: "Slash"` — TÁCH hai câu hỏi ra khỏi nhau ──────
    //
    // Driver báo `code: "/"`. Bàn phím thật trên WebKit báo `"Slash"`. Sự kiện tổng hợp dưới
    // đây đi qua **đúng** đường của sản phẩm *(keymap → command → adapter → Rust → lưới)* và
    // chỉ thay đúng một thứ: ai sinh ra `keydown`. ⇒ Nó phân biệt *"dây tách hỏng"* với
    // *"driver gửi sai `code`"*.
    await browser.execute(() => {
      window.__t28.phim = []
      window.__t28.log = []
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
    await browser.pause(2500)
    console.log(
      '\n[t28 · ⑥ ⌘/ với code="Slash" — console] ' +
        JSON.stringify(await browser.execute(() => window.__t28.log), null, 2),
    )
    console.log(
      '\n[t28 · ⑥ số hàng sau đó] ' +
        JSON.stringify(
          await browser.execute(() => ({
            soHang: document.querySelectorAll('[data-col="src"]').length,
            soVachVeHuu: document.querySelectorAll('.rule-ornament').length,
          })),
          null,
          2,
        ),
    )

    // ── ④ Đối chứng: `⌘M` trên cùng phiên ─────────────────────────────────────────
    await browser.execute(() => {
      window.__t28.phim = []
      window.__t28.log = []
    })
    await browser.keys(['Meta', 'm'])
    await browser.pause(2500)
    console.log(
      '\n[t28 · ④ đối chứng ⌘M — phím] ' +
        JSON.stringify(await browser.execute(() => window.__t28.phim), null, 2),
    )
    console.log(
      '\n[t28 · ④ đối chứng ⌘M — console] ' +
        JSON.stringify(await browser.execute(() => window.__t28.log), null, 2),
    )
    console.log(
      '\n[t28 · ④ số hàng sau ⌘M] ' +
        JSON.stringify(
          await browser.execute(() => ({
            soHang: document.querySelectorAll('[data-col="src"]').length,
            soVachVeHuu: document.querySelectorAll('.rule-ornament').length,
          })),
          null,
          2,
        ),
    )
  })
})
