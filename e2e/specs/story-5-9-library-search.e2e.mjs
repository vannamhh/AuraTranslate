/**
 * Bàn đo Story 5.9 — "Tìm kiếm full-text xuyên Library" (FR8) trong **WKWebView thật**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY TỒN TẠI
 * ═════════════════════════════════════════════════════════════════════════════════
 * `library_index_contract.rs` gọi `Indexer::search`/`Indexer::rebuild` là hàm THUẦN — nó
 * chứng minh hợp đồng dữ liệu và hai chỉ mục FTS5, không chứng minh một cú CLICK thật vào một
 * hàng kết quả có mở đúng Workspace/đúng câu hay không. `tests/frontend/librarySearch.test.ts`
 * chạy trên `happy-dom` với `invoke` GIẢ. AC trung tâm của story này — *"một kết quả của một
 * Tác phẩm KHÁC Tác phẩm đang mở phải mở đúng Chương của đúng Tác phẩm đó"* — là đúng lớp lỗi
 * mà `libraryChapters.ts:234-252` cảnh báo (`chapter_id` là số AUTOINCREMENT CỤC BỘ theo từng
 * `project.db`, nên hai Tác phẩm khác nhau có thể cùng mang `chapter_id = 1`) và không một
 * đường nào trong ba đường kia đo được nó bằng một `project.db` THẬT thứ hai.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 §GIỚI HẠN — ghi ra thay vì để người sau tưởng đã phủ, cùng khuôn hai spec Epic 5 trước
 * ═════════════════════════════════════════════════════════════════════════════════
 * 1. Ô nhập truy vấn đặt giá trị bằng `value = …` cộng một `InputEvent` phát tay (§Giới hạn
 *    mục 3 của `story-5-8-reorganise-chapters.e2e.mjs`, cùng lý do: `v-model` nghe `input`,
 *    nên đường SẢN PHẨM (`librarySearchQuery` → `library_search`) chạy trọn; lượt gõ phím
 *    THẬT không phải mệnh đề spec này đo).
 * 2. Kết quả tìm được ở NỬA NGUYÊN VĂN (`source_text`, qua `trigram`) — không phải nửa bản
 *    dịch: `create_work_from_text` chỉ tạo `source_text`, không có bước dịch tự động nào để
 *    dựng `target_text` khác rỗng trong một bàn đo không lái Editor thật. Vế "hit ở nửa bản
 *    dịch mở đúng câu" đã có lưới ĐẦY ĐỦ ở `library_index_contract.rs` (hàm thuần, dữ liệu
 *    dựng tay) — spec này đo phần CHỈ webview thật mới lộ ra: cú click mở đúng Tác phẩm/Chương.
 * 3. Thư mục gốc Library là MỘT thư mục TẠM DÙNG CHUNG cho cả lượt chạy (`wdio.conf.mjs`
 *    §onPrepare) — spec này định vị bằng CHUỖI TRUY VẤN riêng (đóng dấu thời gian), không bằng
 *    chỉ số hàng.
 * 4. `story-5-6-library-grid.e2e.mjs` đỏ từ baseline (`6b2cb24`, trước Epic 5 hiện tại) — đỏ đó
 *    không đọc thành hồi quy của story này (`5-9-tim-kiem-full-text-xuyen-library.md` §Verification).
 */

import { realClick } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_NAME_A = `e2e-search-a-${RUN_TAG}`
const WORK_NAME_B = `e2e-search-b-${RUN_TAG}`
// Chuỗi ĐÁNH DẤU duy nhất trong toàn bộ thư viện — Latin, ≥ 3 ký tự (sàn cứng của `trigram`,
// đo 2026-08-29). Câu ĐẦU không mang dấu, câu THỨ HAI mang dấu — hai `segment.id` khác nhau
// để phép khẳng định "con trỏ ở ĐÚNG câu khớp" phân biệt được với "mở đúng Chương nhưng caret
// rơi vào câu đầu" (giá trị hồi phòng nếu logic đặt caret bị bỏ qua).
const MARKER = `zzqqsearchmarker${RUN_TAG}`
const SOURCE_TEXT_A = `Cau khong lien quan gi ca。Cau chua tu khoa ${MARKER} o day。`
// 🔵 THÊM (lượt rà 2026-08-29) — CẶP PHÂN BIỆT DẤU, đo qua GIAO DIỆN THẬT. AC *"truy vấn `má`
// chỉ ra kết quả chứa `má`, không ra `ma`/`mà`/…"* trước đó chỉ có lưới ở tầng Rust
// (`library_index_contract.rs`), tức nó đúng với ĐỘNG CƠ mà chưa đúng với ĐƯỜNG NGƯỜI DÙNG GÕ.
// Hai chuỗi dưới đây khác nhau ĐÚNG một dấu sắc và đều ≥ 3 ký tự (sàn cứng của `trigram`).
// ⚠️ Đặt ở NỬA NGUYÊN VĂN, không nửa bản dịch: `create_work_from_text` không dựng `target_text`
// (§GIỚI HẠN mục 2). Đo 2026-08-29, SQLite 3.43.2: `trigram` mặc định PHÂN BIỆT dấu —
// `"khoáng"` khớp `khoáng sản` và KHÔNG khớp `khoang trong`, nên phép đối chứng này có nghĩa.
const DIA_HIT = `khoáng${RUN_TAG}`
const DIA_MISS = `khoang${RUN_TAG}`
const SOURCE_TEXT_B = `Tac pham khac, khong lien quan。Cau mang ${DIA_MISS} khong dau。Cau mang ${DIA_HIT} co dau。`

const SEARCH_INPUT = '[data-library-search-input]'
const SEARCH_BUTTON = '[data-library-search-button]'
const SEARCH_STATUS = '[data-library-search-status]'
const SEARCH_HITS = '[data-library-search-hit]'
const SEARCH_OPEN_BTN = '[data-library-search-open]'

/** Tạo một Tác phẩm qua cầu IPC thật, cùng khuôn mọi spec Epic 5 khác. Trả về `work_id`. */
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

/** Đọc segment của Chương ĐANG MỞ bằng ĐÚNG lệnh IPC của sản phẩm — cùng khuôn hai spec
 * Epic 5 trước. */
async function readSegmentsFromDisk() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('read_open_chapter_segments', {})
  })
}

/** Gõ một chuỗi vào ô tìm kiếm — §Giới hạn mục 1 cho vì sao đặt `.value` thay vì gõ phím thật. */
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

describe('Story 5.9 — tìm kiếm full-text xuyên Library, mở đúng Tác phẩm/Chương/câu', () => {
  it('tìm một câu ở Tác phẩm KHÁC Tác phẩm đang mở, bấm kết quả, và Workspace mở đúng câu khớp', async () => {
    // ── Chờ cầu IPC — cùng khuôn mọi spec Epic 5 khác. ──────────────────────────────
    await browser.waitUntil(async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined), {
      timeout: 30_000,
      interval: 250,
    })

    // ── Tạo Tác phẩm A (mang MARKER), rồi chụp segment của nó TRƯỚC khi B đẩy nó ra
    //    khỏi `OpenWorkState` — đúng khuôn "chụp mốc so" của `story-5-7-open-chapter.e2e.mjs`.
    await createWork(WORK_NAME_A, SOURCE_TEXT_A)
    const workASegments = await readSegmentsFromDisk()
    expect(workASegments.segments.length).toBeGreaterThanOrEqual(2)
    const markerSegment = workASegments.segments.find((s) => s.source_text.includes(MARKER))
    if (markerSegment === undefined) {
      throw new Error(`fixture khong dung dung cau mang MARKER: ${JSON.stringify(workASegments.segments)}`)
    }

    // ── Tạo Tác phẩm B — CHỈ để đẩy A ra khỏi `OpenWorkState` (`replace_open_work`), đúng
    //    khuôn đối chứng của `story-5-7-open-chapter.e2e.mjs`. Cả A lẫn B đều có
    //    `chapter_id = 1` cục bộ (AUTOINCREMENT riêng theo từng `project.db`) — đây CHÍNH LÀ
    //    cửa sổ mà một lượt mở kết quả tìm kiếm hỏng sẽ rơi vào.
    await createWork(WORK_NAME_B, SOURCE_TEXT_B)
    const whileBIsOpen = await readSegmentsFromDisk()
    expect(whileBIsOpen.segments.map((s) => s.source_text)).not.toContain(
      workASegments.segments[0].source_text,
    )

    // ── Gõ MARKER vào ô tìm kiếm rồi bấm "Tìm". `Indexer::rebuild` chạy lại sau MỖI lượt
    //    `create_work_from_text` (đường sản phẩm có sẵn từ Story 5.2/5.3), nên MARKER phải
    //    tra được ngay — nhưng `waitUntil` dưới đây (bấm lại nếu chưa ra hàng nào) là hàng
    //    rào cho một lượt chỉ mục CHƯA commit kịp trên máy chậm, không phải một vòng che lỗi.
    await typeSearchQuery(MARKER)
    const searchBtn = await $(SEARCH_BUTTON)
    await browser.waitUntil(
      async () => {
        if ((await searchHitCount()) >= 1) return true
        await realClick(searchBtn)
        return false
      },
      {
        timeout: 20_000,
        interval: 500,
        timeoutMsg: `truy vấn "${MARKER}" không cho ra hàng kết quả nào sau 20 giây`,
      },
    )

    // AC — MỘT hit đúng nghĩa đen: MARKER duy nhất trong cả thư viện.
    expect(await searchHitCount()).toBe(1)
    const statusText = await searchStatusText()
    expect(statusText.length).toBeGreaterThan(0)

    // ── Bấm "Mở kết quả" — hàng DUY NHẤT, nên nó luôn là hàng ĐANG CHỌN (`cursor` mặc định
    //    0), nút "Mở kết quả" luôn hiện. ──────────────────────────────────────────────────
    const openBtn = await $(SEARCH_OPEN_BTN)
    await realClick(openBtn)

    // ── AC chính: Workspace mở, và nó là ĐÚNG Chương của ĐÚNG Tác phẩm A — không phải B
    //    (dù `chapter_id` trùng số cục bộ), và con trỏ ở ĐÚNG câu mang MARKER. ─────────────
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelectorAll('[data-col="src"]').length > 0),
      {
        timeout: 20_000,
        interval: 250,
        timeoutMsg: 'bấm kết quả tìm kiếm không đổi sang Workspace/nạp lưới sau 20 giây',
      },
    )

    const reopened = await readSegmentsFromDisk()
    // Nội dung là dấu hiệu KHO — `chapter_id` không phải (hai project.db độc lập, xem khối
    // đối chứng ở trên).
    expect(reopened.segments.map((s) => s.source_text)).toEqual(
      workASegments.segments.map((s) => s.source_text),
    )

    // ═══════════════════════════════════════════════════════════════════════════════
    // 🔴 CON TRỎ ĐI QUA `doiConTroToi` (TRẠNG THÁI TRÌNH BÀY), KHÔNG GHI ĐĨA NGAY —
    //    ĐÚNG §Design Notes "Vì sao con trỏ đặt qua `doiConTroToi`, không qua
    //    `save_chapter_position`" của chính story này.
    // ═══════════════════════════════════════════════════════════════════════════════
    // `chapter_position` trên đĩa chỉ đổi qua nhịp `positionFlush` (idle 500 ms · trần
    // 5000 ms, KHÔNG mang bảo đảm AD-35) — SAU khi caret đã dời, không CÙNG LÚC. Đọc
    // `read_open_chapter_segments` ngay một lượt sẽ thấy giá trị CŨ (hồi phòng "segment
    // đầu"), và đó không phải một hồi quy — đo được ở LƯỢT ĐẦU của chính spec này (`received:
    // 1`, tức segment đầu, trước khi khối chờ dưới đây được thêm vào). ⇒ chờ nhịp ghi đó tự
    // chạy (poll, KHÔNG gọi thẳng một hàm nội bộ) rồi mới đọc đĩa lần NỮA.
    await browser.waitUntil(
      async () => {
        const now = await readSegmentsFromDisk()
        return now.caret_segment_id === markerSegment.id
      },
      {
        timeout: 8_000,
        interval: 300,
        timeoutMsg:
          `chapter_position trên đĩa không đổi thành segment mang MARKER (id=${markerSegment.id}) ` +
          'sau 8 giây — nhịp `positionFlush` (idle 500 ms) đáng lẽ đã chạy xong',
      },
    )

    // 🔴 Mệnh đề trung tâm của story: con trỏ ở ĐÚNG câu khớp, không phải câu đầu (giá trị
    // hồi phòng nếu `openChapterById(chapterId, segmentId)` bị gọi thiếu `segmentId`).
    const settled = await readSegmentsFromDisk()
    expect(settled.caret_segment_id).toBe(markerSegment.id)
    expect(settled.caret_segment_id).not.toBe(workASegments.segments[0].id)
  })

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 PHÂN BIỆT DẤU, ĐO QUA ĐÚNG Ô NHẬP NGƯỜI DÙNG GÕ
  // ═══════════════════════════════════════════════════════════════════════════════
  // Ca ở `library_index_contract.rs` đã canh mệnh đề này ở tầng động cơ. Ca dưới đây canh nó ở
  // tầng mà AC thật sự phát biểu — người dùng gõ vào ô tìm và đọc danh sách — nên một lượt
  // trượt trong CHUỖI ĐI QUA DÂY (bọc cụm, tham số camelCase, guard hình dạng) cũng bị bắt,
  // chứ không chỉ một lượt trượt trong SQL.
  it('truy vấn CÓ DẤU chỉ ra câu có dấu, không kéo theo biến thể không dấu', async () => {
    // Quay về Library — cùng khuôn `story-5-5-progress.e2e.mjs::switchToLibrary` (`Mod+1`),
    // không một selector tự đặt.
    await browser.keys(['Meta', '1'])
    await browser.waitUntil(
      async () => browser.execute(() => document.querySelector('.works-block') !== null),
      { timeout: 30_000, timeoutMsg: 'quay về Library rồi mà khối "Tác phẩm" không có mặt sau 30 giây' },
    )

    await typeSearchQuery(DIA_HIT)
    const searchBtn = await $(SEARCH_BUTTON)

    /** Văn bản của MỌI hàng kết quả đang hiện, nối lại. */
    const hitTexts = async () =>
      (
        await browser.execute(
          (sel) => Array.from(document.querySelectorAll(sel)).map((el) => el.textContent ?? ''),
          SEARCH_HITS,
        )
      ).join(' ')

    // ⚠️ Chờ trên NỘI DUNG, không trên SỐ HÀNG. Danh sách còn mang kết quả của ca trước, nên
    // một phép chờ `count >= 1` xanh NGAY LẬP TỨC mà lượt tìm mới chưa hề chạy — đo được ở
    // lượt đầu của chính ca này (nó khẳng định trên chuỗi `zzqqsearchmarker…` của ca trước).
    await browser.waitUntil(
      async () => {
        if ((await hitTexts()).includes(DIA_HIT)) return true
        await realClick(searchBtn)
        return false
      },
      {
        timeout: 20_000,
        interval: 500,
        timeoutMsg: `truy vấn có dấu "${DIA_HIT}" không cho ra hàng kết quả nào sau 20 giây`,
      },
    )

    // ĐÚNG MỘT hàng: câu mang `khoáng…`. Nếu chỉ mục khoan dung dấu thì câu mang `khoang…`
    // cũng vào và con số này thành 2 — tức AC phân biệt dấu đã chết trên đường người dùng đi.
    expect(await searchHitCount()).toBe(1)
    expect(await hitTexts()).not.toContain(DIA_MISS)
  })
})
