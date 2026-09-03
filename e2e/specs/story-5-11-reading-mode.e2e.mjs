/**
 * Bàn đo Story 5.11 — **Chế độ đọc: typography và bố cục đọc dài**, trong WKWebView THẬT.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO BÀN ĐO NÀY ĐO ĐƯỢC — khác hẳn giới hạn của Story 5.10
 * ═════════════════════════════════════════════════════════════════════════════════
 * §GIỚI HẠN của `5-10-hai-che-do-dau.md` ghi *"`create_work_from_text` chỉ dựng `source_text`;
 * không `target_text` nào tồn tại trong một bàn đo e2e"*. Câu đó đúng cho một bàn đo KHÔNG lái
 * Editor. Spec này lái Editor THẬT để dựng `target_text` — cùng công thức đã có ở
 * `editor-typing-flush.e2e.mjs:162-182`: `document.execCommand('insertText')` trên ô
 * `contenteditable`, KHÔNG `browser.keys()` — giới hạn đã đo và ghi ở
 * `editor-typing-flush.e2e.mjs:38-56` (`browser.keys()` chỉ bắn `keydown`, chữ không hạ cánh).
 *
 * ⚠️ Mọi lượt bấm chuột đi qua `realClick()` — ESLint cấm `.click()` của driver trong `e2e/**`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 MỆNH ĐỀ ③ — VÌ SAO ĐO BẰNG THANH TRƯỢT, KHÔNG BẰNG BA PRESET
 * ═════════════════════════════════════════════════════════════════════════════════
 * Ba mức Thoáng/Cân/Đặc mang BA con số `ch` KHÁC NHAU theo thiết kế (62/68/76) — đó chính là
 * điều làm mỗi mức đọc dễ chịu ở đúng cỡ chữ của nó. Đổi PRESET vì thế đổi CẢ HAI trục (cỡ chữ
 * VÀ số ký tự mỗi dòng) cùng lúc, và tỉ lệ giữa bề rộng cột (px) và cỡ chữ giữa hai preset khác
 * nhau KHÔNG bằng nhau (đo được: mức Cân→Thoáng cho tỉ lệ bề rộng ≈0,990 trong khi tỉ lệ cỡ chữ
 * ≈1,086 — hai con số không hề gần nhau). Mệnh đề *"bề rộng và cỡ chữ đổi cùng tỉ lệ"* — đúng
 * thứ AC5 của story và Đối chứng ĐỎ #4 đòi đo — chỉ có nghĩa trong CÙNG MỘT mức, nơi số `ch`
 * đứng yên và chỉ cỡ chữ đổi qua thanh trượt tinh chỉnh. Đây là đường đo bàn này dùng.
 */

import { realClick } from '../support/pointer.mjs'
import { markFlushBaseline, waitForFlushAfter } from '../support/flushWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const TYPED_ONE = 'Ban dich cau mot cho Story 5.11.'
const TYPED_TWO = 'Ban dich cau hai cho Story 5.11.'

async function typeInto(cell, text) {
  await realClick(cell)
  await browser.pause(200)
  const inserted = await browser.execute((t) => document.execCommand('insertText', false, t), text)
  return inserted
}

describe('Story 5.11 — Chế độ đọc: typography và bố cục đọc dài', () => {
  it('gõ bản dịch thật, vào Chế độ đọc, và năm mệnh đề hình học đo được trong WKWebView thật', async () => {
    await openWorkspaceWithWork('Story 5.11 — Che do doc', 'Cau mot。Cau hai。')

    const cells = await $$('[data-col="tgt"]')
    await expect(cells.length).toBe(2)

    const flushBaseline = await markFlushBaseline()

    const first = await typeInto(cells[0], TYPED_ONE)
    const second = await typeInto(cells[1], TYPED_TWO)
    await expect(first).toBe(true)
    await expect(second).toBe(true)

    // 🔴 **HAI lượt chờ, không một — và con số HAI là một mệnh đề về AD-35, không một sự
    // thận trọng.** Đo được ở lượt chạy thật đầu tiên (2026-08-30, WKWebView): một lượt chờ
    // duy nhất cho trang đọc chỉ có câu MỘT, và câu HAI vắng mặt —
    // `expect(pageText).toContain(TYPED_TWO)` ĐỎ với chuỗi nhận được
    // `"Chương 1Ban dich cau mot cho Story 5.11."`.
    //
    // Nguyên nhân KHÔNG phải sản phẩm: `typeInto(cells[1])` dời caret từ câu 1 sang câu 2, và
    // *"rời segment"* là một trong bốn cửa flush NGAY của AD-35 (`setEditorCaret`) — nên lượt
    // flush thứ nhất mang câu MỘT xuống đĩa **trước khi** câu HAI được gõ. `waitForFlushAfter`
    // trả về ngay tại mốc đó, đúng hợp đồng của nó (*"một lượt flush MỚI HƠN baseline"*), và
    // câu HAI lúc ấy còn nằm trong bộ đệm gõ chờ nhịp idle 2 s của riêng nó.
    //
    // ⇒ Chờ **nối tiếp**: mốc trả về của lượt một là baseline của lượt hai, nên lượt hai chỉ
    // xanh khi một flush **thật sự mới hơn** đã chạm WAL. Vẫn là chờ SỰ KIỆN, không một
    // `pause()` nào được thêm, và không hằng số nào bị nới.
    const afterFirstFlush = await waitForFlushAfter(flushBaseline, {
      what: 'lượt flush thứ NHẤT (câu một, kích hoạt bởi lượt rời segment)',
    })
    await waitForFlushAfter(afterFirstFlush, {
      what: 'lượt flush thứ HAI (câu hai, kích hoạt bởi nhịp idle của riêng nó)',
    })

    // Story 5.12: Chế độ đọc chỉ nạp Chương có trạng thái `done` (FR120).
    // Đặt Chương đang mở thành `done` qua IPC trước khi chuyển sang Chế độ đọc.
    await browser.execute(async () => {
      const internals = window.__TAURI_INTERNALS__
      const chapter = await internals.invoke('read_open_chapter')
      await internals.invoke('set_chapter_status', { chapterId: chapter.chapter_id, status: 'done' })
    })

    // `Mod+3` — `mode.reading`. `Mod` phân giải thành `Meta` trên macOS.
    await browser.keys(['Meta', '3'])

    // Mốc SẴN SÀNG: trang đọc đã dựng cột đọc — chờ TRẠNG THÁI, không một khoảng thời gian.
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('.column') !== null), {
      timeout: 15_000,
      timeoutMsg: 'trang doc khong len cot doc sau 15s',
    })

    // ── ① Chữ vừa gõ hiện trên trang đọc ─────────────────────────────────────────────
    const pageText = await browser.execute(() => document.querySelector('.column')?.textContent ?? '')
    await expect(pageText).toContain(TYPED_ONE)
    await expect(pageText).toContain(TYPED_TWO)

    // ── ② Không phần tử biên tập nào trong cây ───────────────────────────────────────
    const editingSurfaces = await browser.execute(() => ({
      dataCol: document.querySelectorAll('[data-col]').length,
      contentEditable: document.querySelectorAll('[contenteditable="true"]').length,
      confirmButtons: [...document.querySelectorAll('button')].filter((b) =>
        (b.textContent ?? '').includes('xác nhận'),
      ).length,
    }))
    await expect(editingSurfaces.dataCol).toBe(0)
    await expect(editingSurfaces.contentEditable).toBe(0)
    await expect(editingSurfaces.confirmButtons).toBe(0)

    // ── ③ Cỡ chữ và bề rộng cột đổi CÙNG TỈ LỆ khi CHỈ cỡ chữ đổi (cùng mức Cân) ─────
    //
    // `2` trần trước để tường minh đang ở mức Cân (mặc định lúc mở, nhưng nêu rõ chứ không
    // ngầm định) — xem §Design Notes ngay đầu tệp cho lý do KHÔNG so hai preset khác nhau.
    await browser.keys(['2'])
    await realClick(await $('[data-reading-toggle-tuner]'))
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('[data-reading-tuner-font-size]') !== null), {
      timeout: 5_000,
      timeoutMsg: 'khoi tinh chinh khong mo sau khi bam nut',
    })

    const probe = () =>
      browser.execute(() => {
        const col = document.querySelector('.column')
        const cs = getComputedStyle(col)
        return {
          width: col.getBoundingClientRect().width,
          fontSize: Number.parseFloat(cs.fontSize),
          cssWidth: cs.width,
          cssMaxWidth: cs.maxWidth,
          parentWidth: col.parentElement.getBoundingClientRect().width,
          viewport: window.innerWidth,
          chPx: (() => {
            const probeEl = document.createElement('div')
            probeEl.style.cssText = 'position:absolute;visibility:hidden;width:100ch'
            probeEl.style.font = cs.font
            col.appendChild(probeEl)
            const w = probeEl.getBoundingClientRect().width / 100
            probeEl.remove()
            return w
          })(),
        }
      })

    const beforeTune = await probe()

    // Kéo thanh trượt cỡ chữ lên 22px (khuôn ví dụ của Design Notes: "17,5px → 22px").
    await browser.execute(() => {
      const input = document.querySelector('[data-reading-tuner-font-size]')
      input.value = '22'
      input.dispatchEvent(new Event('input', { bubbles: true }))
    })

    const afterTune = await probe()
    console.log('[5.11 DIAG] before =', JSON.stringify(beforeTune))
    console.log('[5.11 DIAG] after  =', JSON.stringify(afterTune))

    await expect(afterTune.fontSize).toBeGreaterThan(beforeTune.fontSize)

    // 🔴 **ĐO SỐ KÝ TỰ MỖI DÒNG, KHÔNG ĐO "hai tỉ lệ bằng nhau" — 2026-08-30, sửa sau một lượt
    // ĐỎ đã đo được.** Bản đầu của bàn đo này khẳng định *"bề rộng và cỡ chữ đổi CÙNG TỈ LỆ"* và
    // nó ĐỎ hai lượt liên tiếp (lệch 3,72 % rồi 3,33 %). Số đo lấy ra từ chính WKWebView giải
    // thích vì sao, và nó KHÔNG phải một khuyết tật sản phẩm:
    //
    // | | cỡ chữ | bề rộng cột | `1ch` | bề rộng / `1ch` |
    // |---|---|---|---|---|
    // | trước | 17,5px | 609,375px | 8,96141px | **68,000** |
    // | sau   | 22px   | 745,750px | 10,96688px | **68,000** |
    //
    // ⇒ Số ký tự mỗi dòng đứng yên ở **68** — đúng nguyên văn AC. Thứ KHÔNG đứng yên là tỉ lệ
    // `ch`/`px`: **0,51208** rồi **0,49849**. WebKit làm tròn bề ngang glyph theo TỪNG cỡ chữ
    // (hinting), nên `1ch` không tuyến tính theo `font-size` — và vì thế *"hai tỉ lệ bằng nhau"*
    // là một mệnh đề SAI về engine, không một mệnh đề về sản phẩm. Bàn đo cũ đo nhầm bất biến.
    //
    // Bất biến ĐÚNG, và nó là chữ của AC: **bề rộng cột chia cho `1ch` giữ nguyên**.
    const charsPerLineBefore = beforeTune.width / beforeTune.chPx
    const charsPerLineAfter = afterTune.width / afterTune.chPx
    await expect(Math.abs(charsPerLineBefore - charsPerLineAfter)).toBeLessThan(0.5)
    // Và con số ấy phải là chính thước đo của mức Cân (68ch), không một con số tình cờ nào.
    await expect(Math.abs(charsPerLineBefore - 68)).toBeLessThan(0.5)

    // Đóng khối tinh chỉnh lại — không để nó che phần còn lại của trang cho các mệnh đề sau.
    await realClick(await $('[data-reading-toggle-tuner]'))

    // ── ④ Song ngữ: nguyên văn ở lề TRÁI, cột đọc GIỮ NGUYÊN bề rộng ─────────────────
    const widthBeforeBilingual = await browser.execute(() => document.querySelector('.column').getBoundingClientRect().width)

    await browser.keys(['b'])
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('.margin') !== null), {
      timeout: 5_000,
      timeoutMsg: 'le song ngu khong hien sau khi bam B',
    })

    const bilingual = await browser.execute(() => {
      const margin = document.querySelector('.margin')
      const col = document.querySelector('.column')
      return {
        marginLeft: margin.getBoundingClientRect().left,
        colLeft: col.getBoundingClientRect().left,
        colWidth: col.getBoundingClientRect().width,
      }
    })
    await expect(bilingual.marginLeft).toBeLessThan(bilingual.colLeft)
    await expect(bilingual.colWidth).toBe(widthBeforeBilingual)

    // Tắt song ngữ lại — dọn trạng thái cho spec kế tiếp trong cùng phiên app.
    await browser.keys(['b'])

    // ── ⑤ `D` đổi `document.documentElement.dataset.theme` ──────────────────────────
    const themeBefore = await browser.execute(() => document.documentElement.dataset.theme)
    await browser.keys(['d'])
    await browser.waitUntil(
      async () => {
        const now = await browser.execute(() => document.documentElement.dataset.theme)
        return now !== themeBefore
      },
      { timeout: 5_000, timeoutMsg: 'theme khong doi sau khi bam D' },
    )

    // 🔵 THÊM (rà 2026-08-30) — TRẢ theme về trạng thái ban đầu, cùng kỷ luật dọn dẹp mà
    // mệnh đề ④ đã áp cho công tắc song ngữ. Mọi spec trong một lượt chạy dùng CHUNG một
    // phiên app: để lại theme ngược lại là đẩy một biến môi trường không ai khai vào spec kế
    // tiếp — và một ca đỏ vì lý do đó sẽ đọc lên như một hồi quy sản phẩm.
    await browser.keys(['d'])
    await browser.waitUntil(
      async () => {
        const now = await browser.execute(() => document.documentElement.dataset.theme)
        return now === themeBefore
      },
      { timeout: 5_000, timeoutMsg: 'theme khong tro ve trang thai dau sau luot bam D thu hai' },
    )
  })
})
