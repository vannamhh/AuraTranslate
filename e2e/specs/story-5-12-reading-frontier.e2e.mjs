/**
 * Bàn đo Story 5.12 — "Chế độ đọc đọc Chương đã xong, đang dịch và tạm ngưng" (FR120)
 * trong **WKWebView THẬT**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY TỒN TẠI
 * ═════════════════════════════════════════════════════════════════════════════════
 * Mệnh đề trung tâm của story — *"nguyên văn của Chương chưa bắt đầu và của câu chưa dịch
 * KHÔNG BAO GIỜ xuất hiện trong Chế độ đọc, kể cả khi bật song ngữ"* — là một mệnh đề về **cây DOM trong engine
 * thật**. `happy-dom` (`tests/frontend/readingFrontier.test.ts`) chạy trên `invoke` GIẢ,
 * nên nó chỉ chứng minh được rằng DỮ LIỆU GIẢ không có mặt — không chứng minh được rằng
 * `read_reading_run` thật sự lọc trước khi gửi. Spec này lái toàn bộ đường thật: tạo Tác
 * phẩm → tách Chương → gõ bản dịch → đặt trạng thái → `⌘3` → đo DOM → bấm *Dịch tiếp*.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * §Giới hạn — ghi ra thay vì để người sau tưởng đã phủ
 * ═════════════════════════════════════════════════════════════════════════════════
 * 1. Chương 2 (đang dịch) và Chương 3 (chưa bắt đầu) KHÔNG được dịch (giữ `target_text = ''`) —
 *    mỗi Chương mang một `source_text` PHÂN BIỆT được để đo "không có mặt".
 * 2. Lượt bấm *Dịch tiếp* đi qua `realClick()` (chuột thật) — cùng giới hạn "làm được bằng
 *    bàn phím" mà `story-5-8-reorganise-chapters.e2e.mjs` đã ghi cho `Tab`/`Enter` (chủ Ice).
 * 3. Trạng thái Chương đặt qua cầu IPC trần (`set_chapter_status`) cho nhanh: Chương 1 không phải
 *    Chương đang mở nên nút Workspace `[data-chapter-done]` không áp dụng cho nó.
 */

import { realClick } from '../support/pointer.mjs'
import { markFlushBaseline, waitForFlushAfter } from '../support/flushWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_NAME = `e2e-frontier-${RUN_TAG}`
// zh — bộ tách nhìn `。！？`. Hai câu, tách ở câu thứ hai ⇒ hai Chương một-câu-mỗi-Chương
// sau lượt `split_chapter_at_segment`.
const SOURCE_ONE = 'Cau mot cua Chuong mot.'
const SOURCE_TWO = 'Cau rieng cua Chuong hai chua duoc dich.'
const SOURCE_THREE = 'Cau rieng cua Chuong ba chua bat dau.'
const SOURCE_TEXT = `${SOURCE_ONE}。${SOURCE_TWO}。${SOURCE_THREE}。`
const TRANSLATED_ONE = 'Ban dich cau mot cho Story 5.12.'

/** Đọc segment của Chương ĐANG MỞ qua ĐÚNG lệnh IPC của sản phẩm — cùng khuôn Story 5.8. */
async function readSegmentsFromDisk() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('read_open_chapter_segments', {})
  })
}

/** Tách Chương đang mở tại một câu — xem §Giới hạn của `story-5-8-reorganise-chapters`. */
async function splitAtSegmentViaIpc(segmentId) {
  return browser.execute(async (sid) => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    // ⚠️ Tên THAM SỐ lệnh đi camelCase — `invoke()` chỉ đổi tên ở cấp đỉnh.
    return internals.invoke('split_chapter_at_segment', { segmentId: sid })
  }, segmentId)
}

/** Danh sách Chương của Tác phẩm đang mở — dùng để định vị `chapter_id` của Chương MỚI mà
 * lượt tách vừa sinh ra (con trỏ `OpenWork::chapter_id` giữ nguyên Chương CŨ sau tách). */
async function listChaptersViaIpc() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('list_chapters')
  })
}

/** Đổi trạng thái một Chương qua cầu IPC trần — cùng khuôn `story-5-5-progress.e2e.mjs`. */
async function setChapterStatusViaIpc(chapterId, status) {
  return browser.execute(
    async (id, st) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) return { ok: false, detail: 'khong co cau IPC' }
      try {
        await internals.invoke('set_chapter_status', { chapterId: id, status: st })
        return { ok: true, detail: '' }
      } catch (err) {
        return { ok: false, detail: String(err && err.code ? err.code : err) }
      }
    },
    chapterId,
    status,
  )
}

async function typeInto(cell, text) {
  await realClick(cell)
  await browser.pause(200)
  return browser.execute((t) => document.execCommand('insertText', false, t), text)
}

describe('Story 5.12 — Chế độ đọc đọc qua Chương đang dịch, dừng ở Chương chưa bắt đầu', () => {
  it('Chương 1 done + dịch, Chương 2 in_progress chưa dịch, Chương 3 not_started ⇒ trang đọc chở Chương 1 và một khối chưa dịch của Chương 2, mốc biên là Chương 3; bấm khối mở đúng câu, "Dịch tiếp" mở Chương 3', async () => {
    await openWorkspaceWithWork(WORK_NAME, SOURCE_TEXT)

    // ── Fixture: ba câu trong MỘT Chương lúc mới tạo ─────────────────────────────────
    const initialSegments = await readSegmentsFromDisk()
    await expect(initialSegments.segments.length).toBe(3)
    const [, segmentTwo, segmentThree] = initialSegments.segments
    const chapterOneId = initialSegments.chapter_id

    // ── Gõ bản dịch cho câu MỘT trong khi còn là Chương đang mở, chờ flush chạm đĩa ──
    const cellsBeforeSplit = await $$('[data-col="tgt"]')
    await expect(cellsBeforeSplit.length).toBe(3)
    const flushBaseline = await markFlushBaseline()
    const inserted = await typeInto(cellsBeforeSplit[0], TRANSLATED_ONE)
    await expect(inserted).toBe(true)
    await waitForFlushAfter(flushBaseline, { what: 'lượt flush bản dịch câu một' })

    // ── Tách tại câu BA rồi tại câu HAI ⇒ Chương 1 = câu một, Chương 2 = câu hai, Chương 3 = câu ba ──
    await splitAtSegmentViaIpc(segmentThree.id)
    await splitAtSegmentViaIpc(segmentTwo.id)
    const chapters = (await listChaptersViaIpc()).sort((x, y) => x.ord - y.ord)
    await expect(chapters.length).toBe(3)
    await expect(chapters[0].chapter_id).toBe(chapterOneId)
    const chapterTwoId = chapters[1].chapter_id
    const chapterThreeId = chapters[2].chapter_id

    // ── Trạng thái: Chương 1 done, Chương 2 in_progress, Chương 3 mặc định not_started ──
    const doneResult = await setChapterStatusViaIpc(chapterOneId, 'done')
    await expect(doneResult.ok).toBe(true)
    const inProgressResult = await setChapterStatusViaIpc(chapterTwoId, 'in_progress')
    await expect(inProgressResult.ok).toBe(true)
    const notStartedResult = await setChapterStatusViaIpc(chapterThreeId, 'not_started')
    await expect(notStartedResult.ok).toBe(true)

    // ── `Mod+3` — vào Chế độ đọc. `Mod` phân giải thành `Meta` trên macOS. ────────────
    await browser.keys(['Meta', '3'])
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('.column') !== null), {
      timeout: 15_000,
      timeoutMsg: 'trang doc khong len cot doc sau 15s',
    })

    // ── ① Chữ của Chương 1 có mặt trên trang ─────────────────────────────────────────
    const columnText = await browser.execute(() => document.querySelector('.column')?.textContent ?? '')
    await expect(columnText).toContain(TRANSLATED_ONE)

    // ── ② Chương 2 hiện đúng MỘT khối "chưa dịch" 1 câu; không `source_text` của Chương 2
    //     hay Chương 3 có trong `document.body`, kể cả khi bật song ngữ (`B`) ──────────
    const untranslated = await browser.execute(() => {
      const nodes = [...document.querySelectorAll('[data-reading-untranslated]')]
      return nodes.map((n) => ({ count: n.getAttribute('data-reading-untranslated-count'), first: n.getAttribute('data-reading-untranslated-first') }))
    })
    await expect(untranslated.length).toBe(1)
    await expect(untranslated[0].count).toBe('1')
    await expect(untranslated[0].first).toBe(String(segmentTwo.id))

    const bodyTextClosed = await browser.execute(() => document.body.textContent ?? '')
    await expect(bodyTextClosed).not.toContain(SOURCE_TWO)
    await expect(bodyTextClosed).not.toContain(SOURCE_THREE)

    await browser.keys(['b'])
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('.source-note') !== null), {
      timeout: 5_000,
      timeoutMsg: 'le song ngu khong hien sau khi bam B',
    })
    const bodyTextBilingual = await browser.execute(() => document.body.textContent ?? '')
    await expect(bodyTextBilingual).not.toContain(SOURCE_TWO)
    await expect(bodyTextBilingual).not.toContain(SOURCE_THREE)
    // Tắt song ngữ lại — dọn trạng thái cho phần còn lại của ca.
    await browser.keys(['b'])

    // ── ③ Khối `.frontier` có mặt và nêu đích danh Chương 3 ──────────────────────────
    const frontierText = await browser.execute(() => document.querySelector('.frontier')?.textContent ?? '')
    await expect(frontierText).not.toBe('')
    await expect(frontierText).toContain('Chương 3')

    // ── ④ Bấm khối chưa dịch ⇒ Workspace mở Chương 2, lưới mang câu của nó ──────────
    //     `realClick` đưa chuột vào nút trước (mouseenter đặt aim của khối).
    const block = await $('[data-reading-untranslated]')
    await realClick(block)
    await browser.waitUntil(async () => browser.execute(() => document.querySelectorAll('[data-col="src"]').length > 0), {
      timeout: 15_000,
      timeoutMsg: 'khong vao duoc Workspace sau khi bam khoi chua dich',
    })
    const blockSourceText = await browser.execute(() =>
      [...document.querySelectorAll('[data-col="src"]')].map((el) => el.textContent ?? '').join(' '),
    )
    await expect(blockSourceText).toContain(SOURCE_TWO)
    await expect(blockSourceText).not.toContain(SOURCE_THREE)

    // ── ⑤ Quay lại Chế độ đọc, bấm "Dịch tiếp" ⇒ Workspace mở Chương 3 ──────────────
    await browser.keys(['Meta', '3'])
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('[data-reading-frontier-continue]') !== null), {
      timeout: 15_000,
      timeoutMsg: 'trang doc khong hien nut Dich tiep sau khi quay lai',
    })
    const continueBtn = await $('[data-reading-frontier-continue]')
    await expect(await continueBtn.isExisting()).toBe(true)
    await realClick(continueBtn)

    await browser.waitUntil(async () => browser.execute(() => document.querySelectorAll('[data-col="src"]').length > 0), {
      timeout: 15_000,
      timeoutMsg: 'khong quay lai duoc Workspace sau khi bam Dich tiep',
    })

    const workspaceSourceText = await browser.execute(() =>
      [...document.querySelectorAll('[data-col="src"]')].map((el) => el.textContent ?? '').join(' '),
    )
    await expect(workspaceSourceText).toContain(SOURCE_THREE)

    // Trang đọc phải KHÔNG còn hiện ra — xác nhận state đọc đã bị vứt và chế độ đã đổi
    // hẳn, không phải một lớp phủ nổi trên trang đọc.
    const readingColumnStillThere = await browser.execute(() => document.querySelector('.column') !== null)
    await expect(readingColumnStillThere).toBe(false)
  })
})
