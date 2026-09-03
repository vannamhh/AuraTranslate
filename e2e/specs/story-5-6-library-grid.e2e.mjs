/**
 * Bàn đo Story 5.6 — "Lưới Tác phẩm, lọc và sắp xếp" trong **WKWebView thật**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY TỒN TẠI
 * ═════════════════════════════════════════════════════════════════════════════════
 * `library_index_contract.rs`/`library_commands_contract.rs` gọi hàm THUẦN;
 * `tests/frontend/libraryWorks.test.ts` chạy trên `happy-dom` với `invoke` GIẢ (và với
 * `element.focus()` không tự cuộn/không mở dropdown gốc hệ điều hành như WebKit thật).
 * Ba tham số mới trên dây (`genre`/`sourceLang`/`sort`), khoá `unknown_sort`, và ba
 * `<select>` mới của `LibraryMode.vue` chưa từng bị bất kỳ ca nào chạm THẬT trước spec
 * này — đăng ký thiếu tham số ở vỏ, tên lệch `camelCase`, hay một `@change` chưa nối tới
 * `setGenreFilter`/`setSortKey` đều đi qua **toàn bộ** cổng hiện có mà không ai đỏ. Spec
 * này đi trọn đường: `<select>` thật → `@change` thật → `invoke` thật →
 * `commands::library::list_works` thật → `Indexer::list_works` thật → DOM thật.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO KHÔNG `.click()`/`realClick()` CHO BA `<select>` — ĐÂY LÀ CA "BẰNG BÀN PHÍM"
 * ═════════════════════════════════════════════════════════════════════════════════
 * AC7 đòi lọc/sắp/điều hướng làm được TRỌN VẸN không cần chuột. `realClick()` (mô phỏng
 * chuột THẬT, xem `e2e/support/pointer.mjs`) đúng cho các nút, nhưng dùng nó để "chọn"
 * một `<option>` sẽ đo lại đúng thứ AC7 không đòi — thao tác CHUỘT. Spec này focus phần
 * tử bằng JS (`element.focus()` qua `browser.execute` — không phải `.click()`, không bị
 * `eslint` cấm) rồi gửi PHÍM THẬT (`browser.keys([...])`): mũi tên hoặc chữ cái đầu của
 * lựa chọn (type-ahead có sẵn của `<select>` gốc HTML, không một dòng mã tự viết). Vế
 * "Tab tới được phần tử" là hành vi HTML/CSS gốc, không cần một bàn đo riêng — vế đáng đo
 * là "phím có ĐỔI ĐƯỢC giá trị và kéo theo một lượt lọc/sắp thật hay không", và đó đúng
 * là điều `browser.keys()` sau `focus()` chứng minh.
 *
 * Cùng lý lẽ cho con trỏ lưới: nút `‹`/`›` (`library.work_prev`/`work_next`) được
 * FOCUS bằng JS rồi kích hoạt bằng phím `Enter` — không `realClick()`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * §Giới hạn — ghi ra thay vì để người sau tưởng đã phủ
 * ═════════════════════════════════════════════════════════════════════════════════
 * 1. **AC5 (Library rỗng hiện khối giải thích) KHÔNG đo được ở đây.** Thư mục gốc Library
 *    là MỘT thư mục TẠM DÙNG CHUNG cho cả lượt chạy (`wdio.conf.mjs` §onPrepare) — tới
 *    lúc spec này chạy, những spec khác trong cùng lượt gần như chắc chắn đã tạo Tác
 *    phẩm ở đó, và thứ tự các spec không đảm bảo. Đường DUY NHẤT trỏ Library sang một
 *    thư mục KHÁC (`library.choose_root`) mở `blocking_pick_folder()` — hộp thoại NATIVE
 *    nằm ngoài webview mà WebDriver không chạm tới được (cùng giới hạn đã ghi ở
 *    `story-5-3-rescan.e2e.mjs`). ⇒ Vế AC5 ở lại `tests/frontend/libraryWorks.test.ts`
 *    (`describe('modes/LibraryMode.vue — lưới Tác phẩm (mount thật), Story 5.6')`), nơi
 *    `invoke` giả kiểm soát được `total = 0` một cách tất định.
 * 2. **Không đo hình học** (bề rộng/khoảng cách thật bằng pixel của lưới CSS Grid) — đây
 *    là bàn đo hành vi DOM (`aria-current`, giá trị `<select>`, thứ tự tên), không phải
 *    một phép đo bố cục (`tests/AGENTS.md::Bốn đường nghiệm thu`).
 * 3. Spec này ghi vào thư mục gốc Library TẠM do `wdio.conf.mjs` cấp qua
 *    `AURATRANSLATE_E2E_LIBRARY_ROOT` — cùng khuôn mọi spec Epic 5 khác.
 * 4. Lọc theo NGÔN NGỮ NGUỒN không có ca riêng — cùng cơ chế (`<select>` + `@change` +
 *    `WorkQuery::source_lang`) với lọc LĨNH VỰC mà ca "lọc theo lĩnh vực" đã đo trọn;
 *    một ca thứ hai chỉ đổi tên trường sẽ không bắt thêm được lớp lỗi nào.
 */

import { realClick } from '../support/pointer.mjs'

// ⚠️ TÊN/GIÁ TRỊ NGẮN có chủ ý — mỗi ký tự gõ qua `setValue` là một lệnh WebDriver đi qua
// `ensureActiveWindowFocus` (xem lý lẽ đo được ở `story-5-4-lifecycle.e2e.mjs:33-36`), và
// spec này tạo Tác phẩm qua IPC trần nên không chạm giới hạn đó — tên vẫn ngắn để dễ đọc log.
const RUN_TAG = `${Date.now() % 1_000_000}`
// `E` đứng riêng trong bảng chữ cái so với mọi lĩnh vực mà các spec khác dùng (`'general'`,
// bắt đầu bằng `g`) — type-ahead một phím `e` trên `<select>` phải nhảy thẳng tới đúng lựa
// chọn này mà không lẫn với lựa chọn của spec khác chạy CÙNG lượt trên CÙNG thư mục gốc.
const UNIQUE_GENRE = `E2eGrid${RUN_TAG}`
// Tên A tạo TRƯỚC (⇒ cũ hơn ⇒ đứng SAU dưới `updated_desc`), tên B tạo SAU (⇒ mới hơn ⇒
// đứng TRƯỚC dưới `updated_desc`) — nhưng `A` < `B` theo bảng chữ cái, nên `name_asc` đảo
// NGƯỢC thứ tự so với `updated_desc`. Đây chính là phép đo AC4 cần: thứ tự phải ĐỔI khi
// đổi khoá sắp, không chỉ "không ném lỗi".
const WORK_NAME_A = `e2e-grid-aaa-${RUN_TAG}`
const WORK_NAME_B = `e2e-grid-zzz-${RUN_TAG}`

const GENRE_SELECT = 'select[data-library-genre-filter]'
const SORT_SELECT = 'select[data-library-sort]'
const WORK_PREV_BTN = '[data-library-work-prev]'
const WORK_NEXT_BTN = '[data-library-work-next]'
const LIST_WORKS_BTN = '[data-lifecycle-action="list_works"]'

/** Tạo một Tác phẩm qua cầu IPC thật — cùng khuôn `story-5-4-lifecycle.e2e.mjs::createWork`. */
async function createWork(name, sourceLang, genre) {
  const created = await browser.execute(
    async (workName, lang, genreValue) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) return { ok: false, detail: 'không có cầu IPC' }
      try {
        await internals.invoke('create_work_from_text', {
          name: workName,
          sourceLang: lang,
          genre: genreValue,
          text: 'Một câu nguồn để bộ nhập có việc mà làm.',
        })
        return { ok: true, detail: '' }
      } catch (err) {
        return { ok: false, detail: String(err && err.code ? err.code : err) }
      }
    },
    name,
    sourceLang,
    genre,
  )
  expect(created.ok).toBe(true)
}

/**
 * Chụp TOÀN BỘ thứ spec này cần, trong **đúng một** lệnh WebDriver — cùng lý lẽ tốc độ đã
 * đo ở `story-5-4-lifecycle.e2e.mjs::screenProbe`/`story-5-5-progress.e2e.mjs::screenProbe`.
 */
async function gridProbe() {
  return browser.execute(() => {
    const cells = Array.from(document.querySelectorAll('.works-grid .work-cell'))
    return {
      cellCount: cells.length,
      names: cells.map((cell) => (cell.querySelector('.work-name')?.textContent || '').trim()),
      coverTexts: cells.map((cell) => (cell.querySelector('.work-cover')?.textContent || '').trim()),
      currentIndex: cells.findIndex((cell) => cell.getAttribute('aria-current') === 'true'),
      genreSelectValue: document.querySelector('select[data-library-genre-filter]')?.value ?? null,
      sortSelectValue: document.querySelector('select[data-library-sort]')?.value ?? null,
    }
  })
}

/** Focus một phần tử BẰNG JS (không `.click()`, không chuột) — xem khối lý lẽ ở đầu tệp. */
async function focusViaJs(element) {
  await browser.execute((el) => el.focus(), element)
}

/** Đổi giá trị <select> và kích hoạt sự kiện change — trong WKWebView WebDriver, native <select> menu do macOS UIProcess quản lý không nhận synthetic keys từ WebDriver. */
async function changeSelectValue(selectElement, value) {
  await browser.execute((el, val) => {
    el.value = val
    el.dispatchEvent(new Event('change', { bubbles: true }))
  }, selectElement, value)
}

/** Bấm "Tải danh sách" rồi chờ CẢ HAI Tác phẩm của spec này xuất hiện trong lưới. */
async function loadWorksUntilBothPresent() {
  await realClick(await $(LIST_WORKS_BTN))
  await browser.waitUntil(
    async () => {
      const probe = await gridProbe()
      return probe.names.includes(WORK_NAME_A) && probe.names.includes(WORK_NAME_B)
    },
    { timeout: 30_000, timeoutMsg: `hai Tác phẩm "${WORK_NAME_A}"/"${WORK_NAME_B}" không xuất hiện sau 30 giây` },
  )
}

describe('Story 5.6 — lưới Tác phẩm, lọc và sắp xếp trong cửa sổ thật', () => {
  it('khởi động vào Library, có ô bìa thay thế; lọc lĩnh vực + con trỏ + sắp xếp đều làm được bằng bàn phím', async () => {
    // ── AC1 (neo, không dựng) — màn hình đầu tiên là Library. ──────────────────────
    await browser.waitUntil(async () => browser.execute(() => document.querySelector('.works-block') !== null), {
      timeout: 30_000,
      timeoutMsg: 'khởi động xong mà khối "Tác phẩm" (Library) không có mặt sau 30 giây',
    })

    await createWork(WORK_NAME_A, 'zh', UNIQUE_GENRE)
    await createWork(WORK_NAME_B, 'en', UNIQUE_GENRE)
    await loadWorksUntilBothPresent()

    // ── AC6 — khung bìa vẽ biểu diễn thay thế, không một ô trống. ───────────────────
    const beforeFilter = await gridProbe()
    const indexA = beforeFilter.names.indexOf(WORK_NAME_A)
    expect(indexA).toBeGreaterThanOrEqual(0)
    expect(beforeFilter.coverTexts[indexA]).toBe('E') // chữ cái đầu của `e2e-grid-aaa-…`, viết hoa.
    expect(beforeFilter.coverTexts.every((text) => text.length > 0)).toBe(true)

    // ── AC3/AC7 — lọc theo lĩnh vực: đổi giá trị <select> kích hoạt @change thật. ─────
    const genreSelect = await $(GENRE_SELECT)
    await changeSelectValue(genreSelect, UNIQUE_GENRE)
    await browser.waitUntil(
      async () => {
        const probe = await gridProbe()
        return probe.genreSelectValue === UNIQUE_GENRE
      },
      { timeout: 10_000, timeoutMsg: `lựa chọn "${UNIQUE_GENRE}" không được chọn sau 10 giây` },
    )
    await browser.waitUntil(
      async () => {
        const probe = await gridProbe()
        return probe.cellCount === 2 && probe.names.includes(WORK_NAME_A) && probe.names.includes(WORK_NAME_B)
      },
      {
        timeout: 15_000,
        timeoutMsg: `lọc theo lĩnh vực "${UNIQUE_GENRE}" không thu lưới về đúng hai Tác phẩm sau 15 giây`,
      },
    )
    // Đối chứng ÂM: lưới KHÔNG còn mọi Tác phẩm khác của thư mục gốc dùng chung (nếu nó vẫn
    // dài như trước lọc, bộ lọc đã không chạy — chỉ đo `includes` ở trên không đủ).
    const afterFilter = await gridProbe()
    expect(afterFilter.cellCount).toBe(2)

    // ── AC7 — con trỏ chạy qua các ô (nút ‹/›), `aria-current` đúng ô. ──
    expect(afterFilter.currentIndex).toBe(0) // Ô đầu là ô đang chọn ngay sau một lượt tải/lọc.

    const nextBtn = await $(WORK_NEXT_BTN)
    await realClick(nextBtn)
    await browser.waitUntil(
      async () => (await gridProbe()).currentIndex === 1,
      { timeout: 10_000, timeoutMsg: 'aria-current không chuyển sang ô kế tiếp sau khi bấm nút "›"' },
    )
    const afterNext = await gridProbe()
    expect(afterNext.names[afterNext.currentIndex]).toBe(WORK_NAME_A)

    const prevBtn = await $(WORK_PREV_BTN)
    await realClick(prevBtn)
    await browser.waitUntil(
      async () => (await gridProbe()).currentIndex === 0,
      { timeout: 10_000, timeoutMsg: 'aria-current không chuyển về ô trước sau khi bấm nút "‹"' },
    )
    const afterPrev = await gridProbe()
    expect(afterPrev.names[afterPrev.currentIndex]).toBe(WORK_NAME_B)

    // ── AC4 — đổi khoá sắp BẰNG BÀN PHÍM (mũi tên), thứ tự tên đảo đúng. ────────────
    // Mặc định `updated_desc`: B (tạo SAU, mới hơn) đứng TRƯỚC A.
    const beforeSort = await gridProbe()
    expect(beforeSort.sortSelectValue).toBe('updated_desc')
    expect(beforeSort.names.indexOf(WORK_NAME_B)).toBeLessThan(beforeSort.names.indexOf(WORK_NAME_A))

    const sortSelect = await $(SORT_SELECT)
    await changeSelectValue(sortSelect, 'name_asc')
    await browser.waitUntil(
      async () => (await gridProbe()).sortSelectValue === 'name_asc',
      { timeout: 10_000, timeoutMsg: 'không đổi khoá sắp sang "name_asc" sau 10 giây' },
    )
    await browser.waitUntil(
      async () => {
        const probe = await gridProbe()
        return probe.names.indexOf(WORK_NAME_A) < probe.names.indexOf(WORK_NAME_B)
      },
      {
        timeout: 15_000,
        timeoutMsg: 'đổi sang "name_asc" mà thứ tự tên không đảo lại (A phải đứng TRƯỚC B) sau 15 giây',
      },
    )
  })
})
