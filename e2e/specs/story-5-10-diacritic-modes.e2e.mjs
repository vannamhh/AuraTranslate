/**
 * Bàn đo Story 5.10 — "Hai chế độ dấu" (FR9) trong **WKWebView thật**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY TỒN TẠI
 * ═════════════════════════════════════════════════════════════════════════════════
 * `library_index_contract.rs` chứng minh `Indexer::search(mode)`/`SearchMode`/`MatchKind` là
 * hàm THUẦN trên dữ liệu dựng tay — nó không chứng minh một cú CLICK "Khoan dung dấu" thật vào
 * webview có thật sự phát `library_search` với `mode: "lenient"` trên dây hay không, và không
 * chứng minh lượt TỰ NỚI (không người dùng bấm gì) chạy TRỌN qua cầu IPC thật khi chính xác
 * trả 0 hàng. `tests/frontend/librarySearch.test.ts` chạy trên `happy-dom` với `invoke` GIẢ.
 * Hai mệnh đề dưới đây là đúng lớp "hành vi trong engine thật không có chủ ở ba đường kia".
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 §GIỚI HẠN — ghi ra thay vì để người sau tưởng đã phủ, cùng khuôn spec Story 5.9
 * ═════════════════════════════════════════════════════════════════════════════════
 * 1. **Nửa BẢN DỊCH không dựng được trong bàn đo này.** `create_work_from_text` (đường sản
 *    phẩm DUY NHẤT mà spec e2e có để tạo dữ liệu) chỉ tạo `source_text` — không bước dịch tự
 *    động nào để `target_text` khác rỗng. Chỉ mục khoan dung `library_target_fts_nd` chỉ lập
 *    trên `target_text` (§Design Notes của story: nửa nguyên văn KHÔNG có bản khoan dung), nên
 *    **hai mệnh đề dưới đây cố ý KHÔNG đo trực tiếp việc `_nd` tìm RA một câu có dấu hai lớp
 *    (`ễ`/`ệ`) khi gõ không dấu** — mệnh đề đó đã có lưới ĐẦY ĐỦ ở `library_index_contract.rs`
 *    (`a_two_diacritic_query_widens_and_the_snippet_keeps_the_original_accented_text`, dữ liệu
 *    dựng tay qua `project.db` THẬT). Hai ca ở đây đo phần CHỈ webview thật mới lộ ra:
 *      ① một truy vấn không khớp gì chạy TRỌN đường tự nới qua IPC thật (trạng thái đổi đúng
 *         câu "đã thử cả hai chế độ" trên DOM thật, không phải trên một report giả);
 *      ② bật khoan dung KHÔNG làm một hit ở nửa NGUYÊN VĂN (thứ DỰNG ĐƯỢC ở đây) biến mất —
 *         đúng §Always "chuyển chế độ không được làm mất kết quả", đo qua đúng nút bấm thật.
 * 2. Ô nhập truy vấn đặt giá trị bằng `value = …` cộng một `InputEvent` phát tay (cùng lý do
 *    §Giới hạn mục 1 của `story-5-9-library-search.e2e.mjs`).
 * 3. Thư mục gốc Library là MỘT thư mục TẠM DÙNG CHUNG cho cả lượt chạy (`wdio.conf.mjs`
 *    §onPrepare) — spec này định vị bằng CHUỖI TRUY VẤN riêng (đóng dấu thời gian), không bằng
 *    chỉ số hàng.
 * 4. `story-5-6-library-grid.e2e.mjs` đỏ từ baseline (`6b2cb24`, trước Epic 5 hiện tại) — đỏ đó
 *    không đọc thành hồi quy của story này.
 */

import { realClick } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_NAME = `e2e-diacritic-modes-${RUN_TAG}`

// ── Ca ① — truy vấn KHÔNG khớp gì, trên một chỉ mục CÓ dữ liệu ⇒ tự nới. ────────────────
// Chuỗi vô nghĩa, đủ dài để KHÔNG lẫn vào bất kỳ token nào của SOURCE_TEXT — cả nhánh chính
// xác lẫn nhánh `_nd` (chạy trên `target_text`, ở đây LUÔN RỖNG — §Giới hạn mục 1) đều phải
// trả 0 hàng, đúng ca "Nới vẫn 0" của §I/O Matrix.
const NONSENSE_QUERY = `zzqqkhongkhopgivowidenstill${RUN_TAG}`

// ── Ca ② — một chuỗi CÓ THẬT ở nửa NGUYÊN VĂN, ≥ 3 ký tự (sàn cứng của `trigram`). ──────
const SOURCE_MARKER = `zzqqsourcehalfmarker${RUN_TAG}`
const SOURCE_TEXT = `Cau khong lien quan gi ca。Cau chua tu khoa ${SOURCE_MARKER} o day。`

const SEARCH_INPUT = '[data-library-search-input]'
const SEARCH_BUTTON = '[data-library-search-button]'
const SEARCH_STATUS = '[data-library-search-status]'
const SEARCH_HITS = '[data-library-search-hit]'
const MODE_LENIENT_BTN = '[data-library-search-mode="lenient"]'
const MODE_EXACT_BTN = '[data-library-search-mode="exact"]'

/** Tạo một Tác phẩm qua cầu IPC thật, cùng khuôn `story-5-9-library-search.e2e.mjs`. */
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

/** Gõ một chuỗi vào ô tìm kiếm — cùng khuôn `story-5-9-library-search.e2e.mjs`. */
async function typeSearchQuery(text) {
  await browser.execute(
    (sel, value) => {
      const input = document.querySelector(sel)
      if (input === null) throw new Error('không tìm thấy ô nhập tìm kiếm')
      input.value = value
      input.dispatchEvent(new Event('input', { bubbles: true }))
    },
    SEARCH_INPUT,
    text,
  )
}

async function searchHitCount() {
  return browser.execute((sel) => document.querySelectorAll(sel).length, SEARCH_HITS)
}

async function searchStatusText() {
  return browser.execute((sel) => (document.querySelector(sel)?.textContent || '').trim(), SEARCH_STATUS)
}

/** Văn bản của MỌI hàng kết quả đang hiện, nối lại — cùng khuôn `story-5-9-…e2e.mjs`. */
async function hitTexts() {
  return (
    await browser.execute(
      (sel) => Array.from(document.querySelectorAll(sel)).map((el) => el.textContent ?? ''),
      SEARCH_HITS,
    )
  ).join(' ')
}

describe('Story 5.10 — hai chế độ dấu (FR9)', () => {
  before(async () => {
    // ── Chờ cầu IPC — cùng khuôn mọi spec Epic 5 khác. ──────────────────────────────
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
      timeout: 30_000,
      interval: 250,
    })
    await createWork(WORK_NAME, SOURCE_TEXT)

    // Quay về Library — cùng khuôn `story-5-9-library-search.e2e.mjs` (`Mod+1`).
    await browser.keys(['Meta', '1'])
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('.works-block') !== null),
      { timeout: 30_000, timeoutMsg: 'quay về Library rồi mà khối "Tác phẩm" không có mặt sau 30 giây' },
    )
  })

  // ═══════════════════════════════════════════════════════════════════════════════
  // Ca ① — một truy vấn không khớp gì trên một chỉ mục CÓ dữ liệu ⇒ lượt TỰ NỚI chạy TRỌN
  // qua IPC thật, và trạng thái nói ra đúng câu "đã thử cả hai chế độ" (§I/O Matrix "Nới vẫn 0").
  // ═══════════════════════════════════════════════════════════════════════════════
  it('truy vấn không khớp gì ⇒ tự nới chạy qua IPC thật, trạng thái nói "đã thử cả hai chế độ"', async () => {
    await typeSearchQuery(NONSENSE_QUERY)
    const searchBtn = await $(SEARCH_BUTTON)

    // ⚠️ Chờ trên NỘI DUNG trạng thái, không trên số hàng: 0 hit là ĐÍCH của ca này, nên
    // `searchHitCount() === 0` xanh NGAY LẬP TỨC (kể cả trước khi lượt tìm chạy) — hàng rào
    // đúng là chờ câu trạng thái chứa "khoan dung" (chỉ `search_no_match_widened` mang cụm
    // đó — xem `vi.json`; `search_not_typed`/`search_no_match` thường thì không).
    await browser.waitUntil(
      async () => {
        const status = await searchStatusText()
        if (status.includes('khoan dung')) return true
        await realClick(searchBtn)
        return false
      },
      {
        timeout: 20_000,
        interval: 500,
        timeoutMsg: `trạng thái không đổi thành "đã thử cả hai chế độ" sau 20 giây cho truy vấn ${NONSENSE_QUERY}`,
      },
    )

    // AC — 0 hit thật (không phải "chưa tìm"), và trạng thái phân biệt được với "không khớp"
    // thường (không tự nới) lẫn với "chưa gõ gì".
    expect(await searchHitCount()).toBe(0)
    const statusText = await searchStatusText()
    expect(statusText).toContain('khoan dung')

    // Chế độ NGƯỜI DÙNG vẫn là "Chính xác" — lượt tự nới KHÔNG đổi lựa chọn của người dùng,
    // chỉ đổi CHẾ ĐỘ ĐÃ CHẠY (AD-27 · AC4: mặc định không bao giờ là khoan dung).
    const exactPressed = await browser.execute(
      (sel) => document.querySelector(sel)?.getAttribute('aria-pressed'),
      MODE_EXACT_BTN,
    )
    expect(exactPressed).toBe('true')
  })

  // ═══════════════════════════════════════════════════════════════════════════════
  // Ca ② — bật khoan dung rồi tìm một chuỗi ở nửa NGUYÊN VĂN ⇒ hit VẪN còn (§Always: "chuyển
  // chế độ không được làm mất kết quả"). §Giới hạn mục 1: nửa bản dịch không dựng được ở đây.
  // ═══════════════════════════════════════════════════════════════════════════════
  it('bật khoan dung dấu rồi tìm một chuỗi ở nửa nguyên văn ⇒ hit vẫn còn', async () => {
    await typeSearchQuery(SOURCE_MARKER)
    const searchBtn = await $(SEARCH_BUTTON)

    // ── Trước: tìm ở chế độ CHÍNH XÁC (mặc định), hit phải có mặt. ──────────────────────
    await browser.waitUntil(
      async () => {
        if ((await hitTexts()).includes(SOURCE_MARKER)) return true
        await realClick(searchBtn)
        return false
      },
      {
        timeout: 20_000,
        interval: 500,
        timeoutMsg: `truy vấn "${SOURCE_MARKER}" không cho ra hàng kết quả nào ở chế độ chính xác sau 20 giây`,
      },
    )
    expect(await searchHitCount()).toBe(1)

    // ── Bật "Khoan dung dấu" — §Always: "cả hai nút chế độ chạy lại lượt tìm nếu đã có
    //    truy vấn". Không tự gõ lại, không bấm "Tìm" lần hai. ───────────────────────────
    const lenientBtn = await $(MODE_LENIENT_BTN)
    await realClick(lenientBtn)

    await browser.waitUntil(
      async () => browser.execute((sel) => document.querySelector(sel)?.getAttribute('aria-pressed'), MODE_LENIENT_BTN),
      {
        timeout: 10_000,
        timeoutMsg: 'nút "Khoan dung dấu" không chuyển sang aria-pressed="true" sau 10 giây',
      },
    )
    const lenientPressed = await browser.execute(
      (sel) => document.querySelector(sel)?.getAttribute('aria-pressed'),
      MODE_LENIENT_BTN,
    )
    expect(lenientPressed).toBe('true')

    // ── Sau: hit ở nửa NGUYÊN VĂN vẫn phải còn — chuyển chế độ KHÔNG được làm nó biến mất.
    await browser.waitUntil(async () => (await hitTexts()).includes(SOURCE_MARKER), {
      timeout: 20_000,
      interval: 500,
      timeoutMsg: `hit ở nửa nguyên văn "${SOURCE_MARKER}" biến mất sau khi bật khoan dung dấu`,
    })
    expect(await searchHitCount()).toBe(1)
  })
})
