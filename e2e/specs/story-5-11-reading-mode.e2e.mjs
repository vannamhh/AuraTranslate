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
const SOURCE_TEXT = '他控制了局面。我们走吧。'
const LOOKUP_WORD = '控制'

async function typeInto(cell, text) {
  await realClick(cell)
  await browser.pause(200)
  const inserted = await browser.execute((t) => document.execCommand('insertText', false, t), text)
  return inserted
}

describe('Story 5.11 — Chế độ đọc: typography và bố cục đọc dài', () => {
  it('gõ bản dịch thật, vào Chế độ đọc, và năm mệnh đề hình học đo được trong WKWebView thật', async () => {
    await openWorkspaceWithWork('Story 5.11 — Che do doc', SOURCE_TEXT)

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

    // Chars per line (column width / 1ch) stay in [68, 68 + 7/1ch) at each font size: the
    // column is round(up, 68ch, 7px) and 1ch is not linear in font-size.
    const charsPerLineBefore = beforeTune.width / beforeTune.chPx
    const charsPerLineAfter = afterTune.width / afterTune.chPx
    await expect(Math.abs(charsPerLineBefore - charsPerLineAfter)).toBeLessThan(7 / Math.min(beforeTune.chPx, afterTune.chPx))
    await expect(charsPerLineBefore).toBeGreaterThanOrEqual(68 - 0.01)
    await expect(charsPerLineBefore).toBeLessThan(68 + 7 / beforeTune.chPx)
    await expect(charsPerLineAfter).toBeGreaterThanOrEqual(68 - 0.01)
    await expect(charsPerLineAfter).toBeLessThan(68 + 7 / afterTune.chPx)

    // Đóng khối tinh chỉnh lại — không để nó che phần còn lại của trang cho các mệnh đề sau.
    await realClick(await $('[data-reading-toggle-tuner]'))

    // ── ④ Song ngữ: nguyên văn ở lề TRÁI, cột đọc GIỮ NGUYÊN bề rộng khi đủ chỗ ─────────────────
    const widthBeforeBilingual = await browser.execute(() => document.querySelector('.column').getBoundingClientRect().width)
    const sidesOf = () => browser.execute(() => {
      const col = document.querySelector('.column').getBoundingClientRect()
      const page = document.querySelector('.page').getBoundingClientRect()
      return { left: col.left - page.left, right: page.right - col.right }
    })
    const sidesPlain = await sidesOf()
    await expect(Math.abs(sidesPlain.left - sidesPlain.right)).toBeLessThan(2)

    await browser.keys(['b'])
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('.source-note') !== null), {
      timeout: 5_000,
      timeoutMsg: 'le song ngu khong hien sau khi bam B',
    })

    const bilingual = await browser.execute(() => {
      const note = document.querySelector('.source-note')
      const col = document.querySelector('.column')
      const paragraph = note.closest('.reading-paragraph')
      const noteBox = note.getBoundingClientRect()
      const text = paragraph.querySelector('.paragraph').getBoundingClientRect()
      // The fixture has one paragraph; a cloned second one shows that a note taller than its
      // paragraph stays inside it and the next paragraph and its note start below, still level.
      const original = note.textContent
      const next = paragraph.cloneNode(true)
      paragraph.after(next)
      note.textContent = original.repeat(40)
      const nextNote = next.querySelector('.source-note').getBoundingClientRect()
      const nextText = next.querySelector('.paragraph').getBoundingClientRect()
      const tall = { note: note.getBoundingClientRect().bottom, paragraph: paragraph.getBoundingClientRect().bottom }
      note.textContent = original
      next.remove()
      return {
        noteLeft: noteBox.left,
        noteRight: noteBox.right,
        noteTopToText: noteBox.top - text.top,
        tallNoteOverhang: tall.note - tall.paragraph,
        nextTopBelowTallNote: nextText.top - tall.note,
        nextNoteTopToText: nextNote.top - nextText.top,
        pageLeft: document.querySelector('.page').getBoundingClientRect().left,
        colLeft: col.getBoundingClientRect().left,
        colWidth: col.getBoundingClientRect().width,
      }
    })
    await expect(bilingual.noteRight).toBeLessThanOrEqual(bilingual.colLeft)
    await expect(bilingual.noteLeft).toBeGreaterThanOrEqual(bilingual.pageLeft)
    await expect(Math.abs(bilingual.noteTopToText)).toBeLessThan(1)
    await expect(bilingual.tallNoteOverhang).toBeLessThanOrEqual(0)
    await expect(bilingual.nextTopBelowTallNote).toBeGreaterThanOrEqual(0)
    await expect(Math.abs(bilingual.nextNoteTopToText)).toBeLessThan(1)
    await expect(bilingual.colWidth).toBe(widthBeforeBilingual)
    // Centred when both sides can hold the note track; otherwise the left side keeps exactly the
    // note track and the column moves right (the tuner above has widened the column).
    const sidesBilingual = await sidesOf()
    const noteTrack = bilingual.noteRight - bilingual.noteLeft + (bilingual.colLeft - bilingual.noteRight)
    const halfFree = (sidesBilingual.left + sidesBilingual.right) / 2
    await expect(Math.abs(sidesBilingual.left - Math.max(noteTrack, halfFree))).toBeLessThan(2)

    // Too narrow even for the shifted column: the left side keeps the note track, the column shrinks.
    const narrow = await browser.execute((track) => {
      const page = document.querySelector('.page')
      const col = document.querySelector('.column')
      page.style.width = `${String(col.getBoundingClientRect().width + track - 100)}px`
      const result = {
        colWidth: col.getBoundingClientRect().width,
        colLeft: col.getBoundingClientRect().left - page.getBoundingClientRect().left,
        noteLeft: document.querySelector('.source-note').getBoundingClientRect().left - page.getBoundingClientRect().left,
      }
      page.style.width = ''
      return result
    }, noteTrack)
    await expect(Math.abs(narrow.colWidth - (widthBeforeBilingual - 100))).toBeLessThan(2)
    await expect(Math.abs(narrow.colLeft - noteTrack)).toBeLessThan(2)
    await expect(narrow.noteLeft).toBeGreaterThanOrEqual(0)

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

  // Runs after the case above, which leaves Reading mode open on the same Work with bilingual off.
  it('double-click on a source-note word opens the lookup popup beside it with real results; Esc closes it', async function () {
    const sourceCount = await browser.execute(async () => {
      try {
        const res = await window.__TAURI_INTERNALS__.invoke('list_dict_sources')
        return Array.isArray(res?.sources) ? res.sources.length : -1
      } catch {
        return -1
      }
    })
    if (sourceCount < 0) throw new Error('[BÀN ĐO HỎNG] `list_dict_sources` ném hoặc trả hình dạng lạ; mệnh đề chưa được đo.')
    if (sourceCount === 0) {
      console.warn('[BÀN ĐO KHÔNG DỰNG ĐƯỢC] 0 nguồn từ điển: popup không có kết quả thật để đo. Dựng dữ liệu rồi chạy lại.')
      this.skip()
    }

    const bilingualOn = await browser.execute(() => document.querySelector('[data-reading-source-note]') !== null)
    if (!bilingualOn) await browser.keys(['b'])
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('[data-reading-source-note]') !== null), {
      timeout: 5_000,
      timeoutMsg: 'le song ngu khong hien sau khi bam B',
    })

    // Driver mouse events carry `isTrusted: false` and select nothing, so the word is selected
    // the way WebKit does on a real double-click before `dblclick` is fired.
    const range = await browser.execute((word) => {
      const note = document.querySelector('[data-reading-source-note]')
      const text = note.firstChild
      const at = text.data.indexOf(word)
      if (at === -1) return null
      const r = document.createRange()
      r.setStart(text, at)
      r.setEnd(text, at + word.length)
      const selection = window.getSelection()
      selection.removeAllRanges()
      selection.addRange(r)
      note.dispatchEvent(new MouseEvent('dblclick', { bubbles: true }))
      const box = r.getBoundingClientRect()
      return { top: box.top, bottom: box.bottom, left: box.left, right: box.right }
    }, LOOKUP_WORD)
    await expect(range).not.toBe(null)

    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('[data-reading-lookup] .lookup-source') !== null),
      { timeout: 10_000, timeoutMsg: 'popup tra cuu khong hien ket qua that sau double-click' },
    )

    const shown = await browser.execute(() => {
      const box = document.querySelector('[data-reading-lookup]')
      const host = document.querySelector('section.mode').getBoundingClientRect()
      const r = box.getBoundingClientRect()
      return {
        headword: box.querySelector('[data-reading-lookup-headword]')?.textContent ?? '',
        records: box.querySelectorAll('.lookup-source').length,
        top: r.top,
        bottom: r.bottom,
        left: r.left,
        right: r.right,
        host: { top: host.top, bottom: host.bottom, left: host.left, right: host.right },
        shadow: getComputedStyle(box).boxShadow,
      }
    })
    console.log('[reading lookup DIAG] range =', JSON.stringify(range), 'popup =', JSON.stringify(shown))
    await expect(shown.headword).toBe(LOOKUP_WORD)
    await expect(shown.records).toBeGreaterThan(0)
    await expect(shown.shadow).not.toBe('none')
    const below = Math.abs(shown.top - range.bottom)
    const above = Math.abs(range.top - shown.bottom)
    await expect(Math.min(below, above)).toBeLessThan(8)
    await expect(shown.top).toBeGreaterThanOrEqual(shown.host.top - 1)
    await expect(shown.bottom).toBeLessThanOrEqual(shown.host.bottom + 1)
    await expect(shown.left).toBeGreaterThanOrEqual(shown.host.left - 1)
    await expect(shown.right).toBeLessThanOrEqual(shown.host.right + 1)

    await browser.keys(['Escape'])
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('[data-reading-lookup]') === null), {
      timeout: 5_000,
      timeoutMsg: 'Esc khong dong popup tra cuu',
    })

    await browser.keys(['b'])
  })
})
