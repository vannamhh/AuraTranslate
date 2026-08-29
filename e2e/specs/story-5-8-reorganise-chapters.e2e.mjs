/**
 * Bàn đo Story 5.8 — "Tổ chức lại Chương sau khi nhập" (FR15 · AD-32) trong **WKWebView
 * thật**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY TỒN TẠI
 * ═════════════════════════════════════════════════════════════════════════════════
 * `project_contract.rs` gọi hàm THUẦN và là đường nghiệm thu của AD-32 (đúng hai cột đổi);
 * `tests/frontend/libraryChapters.test.ts` chạy trên `happy-dom` với `invoke` GIẢ. Bốn vỏ IPC
 * mới (`rename_chapter` · `move_chapter` · `merge_chapter_into_previous` ·
 * `split_chapter_at_segment`) và năm id lệnh mới của `CommandRegistry` chưa từng bị bất kỳ ca
 * nào chạm THẬT trước spec này — và ba trong bốn thao tác chạy **khuôn bốn bước** (ghi SQL →
 * `WorkMeta::rebuild_from_store` → `write_atomic` → `reindex`), tức chúng chạm `meta.json` và
 * `library-index.db` trên đĩa thật. Một lỗi thứ tự ở đó không lộ ra ở `happy-dom`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 §GIỚI HẠN — ghi ra thay vì để người sau tưởng đã phủ
 * ═════════════════════════════════════════════════════════════════════════════════
 * 1. 🔴 **Vế "làm được BẰNG BÀN PHÍM" KHÔNG được spec này nghiệm thu.** Mọi lượt bấm ở đây đi
 *    qua `realClick()` (chuột thật). Phép đo 2026-08-29 ghi ở
 *    `story-5-7-open-chapter.e2e.mjs` §Giới hạn và ở `deferred-work.md` cho thấy
 *    `element.focus()` bằng JS cộng `browser.keys()` **không tới được phần tử** — WebDriver
 *    gửi phím tới phần tử mà **nó** coi là đang có tiêu điểm. *"Không có mã để hỏng"* không
 *    phải một phép đo (`AGENTS.md`: *"Không đánh dấu đạt bằng suy luận"*). Món nợ có chủ Ice.
 * 2. 🔴 **Lượt TÁCH đi qua cầu IPC trần, không qua hợp âm `Mod+Shift+Slash`** — cùng lý do
 *    mục 1, và cùng khuôn `saveChapterPositionViaIpc` của spec Story 5.7. Thứ spec này đo ở
 *    nhánh tách là **hợp đồng IPC + hệ quả trên đĩa**, không phải đường bàn phím.
 *    `editorPanelState.ts::splitChapterHere` (vị từ caret rỗng) có lưới ở
 *    `tests/frontend/libraryChapters.test.ts`.
 * 3. Ô nhập tên đặt giá trị bằng `value = …` cộng một `InputEvent` phát tay — `v-model` của
 *    Vue nghe sự kiện `input`, nên đường **sản phẩm** (`chapterRenameDraft` → `renameChapter`)
 *    chạy trọn; thứ KHÔNG được đo là lượt gõ phím thật vào ô đó, cùng lý do mục 1.
 * 4. Thư mục gốc Library là MỘT thư mục TẠM DÙNG CHUNG cho cả lượt chạy (`wdio.conf.mjs`
 *    §onPrepare). Spec này định vị Tác phẩm của chính nó bằng TÊN, không bằng chỉ số `0`.
 */

import { realClick } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_NAME = `e2e-reorg-${RUN_TAG}`
// zh — bộ tách nhìn `。！？；`. BỐN câu: tách ở câu thứ ba cho hai nửa 2 + 2, tức hai con số
// BẰNG NHAU sẽ không phân biệt được nửa nào là nửa nào. Dùng 1 + 3 (tách ở câu thứ hai) để
// mọi phép khẳng định phía sau đỏ được.
const SOURCE_TEXT = 'Cau mot。Cau hai。Cau ba。Cau bon。'

const LIST_WORKS_BTN = '[data-lifecycle-action="list_works"]'
const WORK_NEXT_BTN = '[data-library-work-next]'
const OPEN_WORK_BTN = '[data-library-open-work]'
const CHAPTER_ROWS = '.chapters-list [data-library-chapter-row]'
const CHAPTER_NEXT_BTN = '[data-library-chapter-next]'
const RENAME_INPUT = '[data-library-chapter-rename-input]'
const RENAME_BTN = '[data-library-chapter-rename]'
const MOVE_DOWN_BTN = '[data-library-chapter-move-down]'
const MERGE_UP_BTN = '[data-library-chapter-merge-up]'

/** Tạo một Tác phẩm qua cầu IPC thật, cùng khuôn mọi spec Epic 5 khác. */
async function createWork(name, text) {
  const result = await browser.execute(
    async (workName, sourceText) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) return { ok: false, detail: 'không có cầu IPC' }
      try {
        const created = await internals.invoke('create_work_from_text', {
          name: workName,
          sourceLang: 'zh',
          genre: 'general',
          text: sourceText,
        })
        return { ok: true, workId: created.meta.work_id, detail: '' }
      } catch (err) {
        return { ok: false, detail: String(err && err.code ? err.code : err) }
      }
    },
    name,
    text,
  )
  if (!result.ok) {
    throw new Error(
      `Fixture không tạo được Tác phẩm "${name}": ${result.detail}\n\n` +
        'Đây là lỗi HẠ TẦNG của bàn đo, không một hồi quy sản phẩm.',
    )
  }
  return result.workId
}

/** Đọc segment của Chương ĐANG MỞ bằng ĐÚNG lệnh IPC của sản phẩm. Câu đã VỀ HƯU không nằm
 * trong kết quả (`retired_at IS NULL`), nên `segments.length` là một phép đo TRỰC TIẾP của
 * mệnh đề *"không segment nào bị về hưu"* của AC7. */
async function readSegmentsFromDisk() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('read_open_chapter_segments', {})
  })
}

/** Tách Chương đang mở tại một câu — xem §Giới hạn mục 2 cho vì sao đi thẳng IPC. */
async function splitAtSegmentViaIpc(segmentId) {
  return browser.execute(async (sid) => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    // ⚠️ Tên THAM SỐ lệnh đi camelCase — `invoke()` chỉ đổi tên ở cấp đỉnh.
    return internals.invoke('split_chapter_at_segment', { segmentId: sid })
  }, segmentId)
}

/** Chụp danh sách Chương đang hiện trên màn hình, trong đúng một lệnh WebDriver. */
async function chapterProbe() {
  return browser.execute((sel) => {
    const rows = Array.from(document.querySelectorAll(sel))
    return rows.map((row) => ({
      ord: (row.querySelector('.chapter-ord')?.textContent || '').trim(),
      title: (row.querySelector('.chapter-title')?.textContent || '').trim(),
      count: (row.querySelector('.chapter-segment-count')?.textContent || '').trim(),
      current: row.getAttribute('aria-current') === 'true',
    }))
  }, CHAPTER_ROWS)
}

async function workGridProbe() {
  return browser.execute(() => {
    const cells = Array.from(document.querySelectorAll('.works-grid .work-cell'))
    return {
      names: cells.map((cell) => (cell.querySelector('.work-name')?.textContent || '').trim()),
      currentIndex: cells.findIndex((cell) => cell.getAttribute('aria-current') === 'true'),
    }
  })
}

async function moveWorkCursorTo(name) {
  await browser.waitUntil(async () => (await workGridProbe()).names.includes(name), {
    timeout: 30_000,
    timeoutMsg: `Tác phẩm "${name}" không xuất hiện trong lưới sau 30 giây`,
  })
  const nextBtn = await $(WORK_NEXT_BTN)
  await browser.waitUntil(
    async () => {
      const probe = await workGridProbe()
      if (probe.names[probe.currentIndex] === name) return true
      await realClick(nextBtn)
      return false
    },
    { timeout: 20_000, timeoutMsg: `con trỏ lưới không tới được ô "${name}" sau 20 giây` },
  )
}

/** Nạp lại danh sách Chương qua đường SẢN PHẨM — bấm "Mở Tác phẩm" (`library.open_work`),
 * thứ đọc lại `.atproj` từ đĩa rồi `loadChapters()`. Không một lối tắt gọi thẳng state. */
async function reloadChaptersViaOpenWork() {
  await realClick(await $(OPEN_WORK_BTN))
}

describe('Story 5.8 — tổ chức lại Chương: tách, đổi tên, dời, gộp', () => {
  it('tách một Chương làm hai, đổi tên, dời xuống, rồi gộp lại — segment giữ nguyên suốt bốn lượt', async () => {
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
      timeout: 30_000,
      interval: 250,
    })

    await createWork(WORK_NAME, SOURCE_TEXT)

    // ── Ảnh chụp GỐC: bốn câu, bốn `segment.id`. Đây là mốc so cho AC5/AC6/AC7. ──────
    const before = await readSegmentsFromDisk()
    expect(before.segments).toHaveLength(4)
    const idsGoc = before.segments.map((s) => s.id)
    const textsGoc = before.segments.map((s) => s.source_text)

    // ── Đưa Tác phẩm này thành Tác phẩm đang chọn trong lưới. ───────────────────────
    await realClick(await $(LIST_WORKS_BTN))
    await moveWorkCursorTo(WORK_NAME)
    await reloadChaptersViaOpenWork()

    await browser.waitUntil(async () => (await chapterProbe()).length === 1, {
      timeout: 20_000,
      timeoutMsg: 'danh sách Chương không hiện đúng một hàng trước lượt tách',
    })

    // ═══════════════════════════════════════════════════════════════════════════════
    // ① TÁCH tại câu THỨ HAI ⇒ hai Chương 1 + 3 (§Giới hạn mục 2: qua IPC trần).
    // ═══════════════════════════════════════════════════════════════════════════════
    await splitAtSegmentViaIpc(idsGoc[1])
    await reloadChaptersViaOpenWork()

    await browser.waitUntil(async () => (await chapterProbe()).length === 2, {
      timeout: 20_000,
      timeoutMsg: 'lượt tách không sinh ra Chương thứ hai trên màn hình sau 20 giây',
    })
    const sauTach = await chapterProbe()
    // `ord` phải LIÊN TỤC từ 1 — tiền đề mà `open_adjacent_chapter`/Story 2.10 đứng trên.
    expect(sauTach.map((r) => r.ord)).toEqual(['1', '2'])
    // 1 + 3, không 2 + 2: xem chú thích của `SOURCE_TEXT` cho vì sao hai số phải KHÁC nhau.
    expect(sauTach.map((r) => r.count)).toEqual(['1', '3'])

    // ═══════════════════════════════════════════════════════════════════════════════
    // ② ĐỔI TÊN Chương đang chọn (hàng đầu) — đường SẢN PHẨM, `library.chapter_rename`.
    // ═══════════════════════════════════════════════════════════════════════════════
    const TEN_MOI = 'Hoi Mot'
    await browser.execute(
      (sel, ten) => {
        const input = document.querySelector(sel)
        if (input === null) throw new Error('không tìm thấy ô nhập tên Chương')
        input.value = ten
        // `v-model` nghe `input` — xem §Giới hạn mục 3.
        input.dispatchEvent(new Event('input', { bubbles: true }))
      },
      RENAME_INPUT,
      TEN_MOI,
    )
    await realClick(await $(RENAME_BTN))

    await browser.waitUntil(async () => (await chapterProbe())[0]?.title === TEN_MOI, {
      timeout: 20_000,
      timeoutMsg: `hàng Chương đầu không mang tên "${TEN_MOI}" sau lượt đổi tên`,
    })
    // Đổi tên KHÔNG được đụng số câu của bất kỳ hàng nào.
    expect((await chapterProbe()).map((r) => r.count)).toEqual(['1', '3'])

    // ═══════════════════════════════════════════════════════════════════════════════
    // ③ DỜI XUỐNG — chỉ cột `ord` đổi, số câu đi theo đúng Chương của nó.
    // ═══════════════════════════════════════════════════════════════════════════════
    await realClick(await $(MOVE_DOWN_BTN))
    await browser.waitUntil(async () => (await chapterProbe())[1]?.title === TEN_MOI, {
      timeout: 20_000,
      timeoutMsg: 'lượt dời xuống không đưa Chương đã đổi tên xuống hàng thứ hai',
    })
    const sauDoi = await chapterProbe()
    expect(sauDoi.map((r) => r.ord)).toEqual(['1', '2'])
    // Ba câu nay ở hàng ĐẦU, một câu ở hàng hai — đúng phép hoán vị, không một lượt đánh số lại.
    expect(sauDoi.map((r) => r.count)).toEqual(['3', '1'])

    // ═══════════════════════════════════════════════════════════════════════════════
    // ④ GỘP hàng thứ hai vào hàng đầu ⇒ một Chương, BỐN câu, KHÔNG câu nào về hưu.
    // ═══════════════════════════════════════════════════════════════════════════════
    await realClick(await $(CHAPTER_NEXT_BTN)) // Con trỏ về hàng thứ hai.
    await browser.waitUntil(async () => (await chapterProbe())[1]?.current === true, {
      timeout: 20_000,
      timeoutMsg: 'con trỏ danh sách Chương không tới được hàng thứ hai',
    })
    await realClick(await $(MERGE_UP_BTN))

    await browser.waitUntil(async () => (await chapterProbe()).length === 1, {
      timeout: 20_000,
      timeoutMsg: 'lượt gộp không đưa danh sách về đúng một hàng sau 20 giây',
    })
    expect((await chapterProbe())[0]?.count).toBe('4')

    // ═══════════════════════════════════════════════════════════════════════════════
    // 🔴 MỆNH ĐỀ CHÍNH — AD-32 đọc trên ĐĨA THẬT sau BỐN lượt tổ chức nối tiếp.
    // ═══════════════════════════════════════════════════════════════════════════════
    // `read_open_chapter_segments` lọc `retired_at IS NULL`, nên bốn hàng trả về là bằng
    // chứng trực tiếp của AC7 (*"không segment nào bị về hưu"*): một lượt gộp/tách cài theo
    // khuôn AD-5 (về hưu + tạo mới) sẽ cho **id MỚI** và số câu sống khác đi.
    const sauCung = await readSegmentsFromDisk()
    expect(sauCung.segments).toHaveLength(4)

    // 🔴 **THỨ TỰ MONG ĐỢI LÀ `[2, 3, 4, 1]`, KHÔNG PHẢI THỨ TỰ GỐC — và đây là hành vi
    // ĐÚNG, không một chỗ nới lỏng phép kiểm.** Câu thứ nhất nằm ở Chương đã bị người dùng
    // DỜI XUỐNG rồi GỘP LÊN, nên câu của nó phải đứng SAU ba câu kia: `merge` tịnh tiến
    // `ord` của Chương bị gộp thêm `MAX(ord)` của Chương nhận (ở đây `3`), tức câu 1 nhận
    // `ord = 4`. Một bàn đo khẳng định `[1, 2, 3, 4]` ở đây sẽ đỏ trên một sản phẩm ĐÚNG —
    // và cách "sửa" nó bằng cách so như một TẬP sẽ làm mất đúng mệnh đề đáng đo nhất.
    expect(sauCung.segments.map((s) => s.id)).toEqual([idsGoc[1], idsGoc[2], idsGoc[3], idsGoc[0]])
    expect(sauCung.segments.map((s) => s.source_text)).toEqual([
      textsGoc[1],
      textsGoc[2],
      textsGoc[3],
      textsGoc[0],
    ])
    // Đối chứng ÂM cho vế AD-32 quan trọng nhất: bốn `segment.id` là ĐÚNG bốn id gốc —
    // không một id MỚI nào (tức không một lượt "về hưu + tạo mới" theo khuôn AD-5).
    expect([...sauCung.segments.map((s) => s.id)].sort((a, b) => a - b)).toEqual(
      [...idsGoc].sort((a, b) => a - b),
    )
    // `ord` của Chương gộp phải liên tục 1..4 — phép tịnh tiến hằng số, không một lỗ.
    expect(sauCung.segments.map((s) => s.ord)).toEqual([1, 2, 3, 4])
  })
})
