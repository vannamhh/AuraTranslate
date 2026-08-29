/**
 * Bàn đo Story 5.7 — "Danh sách Chương và mở Chương vào Workspace" (FR12) trong **WKWebView
 * thật**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY TỒN TẠI
 * ═════════════════════════════════════════════════════════════════════════════════
 * `project_contract.rs`/`library_index_contract.rs` gọi hàm THUẦN; `tests/frontend/
 * libraryChapters.test.ts` chạy trên `happy-dom` với `invoke` GIẢ. Bốn vỏ IPC mới
 * (`open_work` · `list_chapters` · `open_chapter` · `save_chapter_position`) và năm id lệnh
 * mới của `CommandRegistry` chưa từng bị bất kỳ ca nào chạm THẬT trước spec này — đường
 * MỞ LẠI một `.atproj` **đã có trên đĩa** (món nợ kiến trúc trung tâm của Epic 5, đóng ở
 * story này) đặc biệt cần một phép đo thật: nó là chỗ Rust dựng lại `Store`/`ScopeResolver`
 * từ đầu, và một lỗi thứ tự ở đó (mở trước khi chỉ mục sẵn sàng, hay `replace_open_work`
 * chạy trước khi `Store` cũ đóng xong) không lộ ra ở bất kỳ ca `happy-dom`/`cargo test` nào.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 GIỚI HẠN CỦA BỘ ĐO — ĐO ĐƯỢC 2026-08-29, GHI RA THAY VÌ VÁ BẰNG MỘT MỆNH ĐỀ YẾU HƠN
 * ═════════════════════════════════════════════════════════════════════════════════
 * Chuỗi phép đo tại chỗ (bốn lượt chạy, cùng kết quả):
 *
 * | Bước | Đo được |
 * |---|---|
 * | `realClick()` vào một nút thật TRƯỚC khi làm gì khác | `document.hasFocus()` **vẫn `false`** ngay sau đó |
 * | `caretPlacement.value = id` → watcher gọi `target.focus()` trên một `<span contenteditable>` | KHÔNG dính — `document.activeElement` là gốc `<section class="mode">`, không phải `<span>` |
 * | `focusViaJs(button)` + `browser.keys(['Enter'])` trên một `<button>` | 🔵 **ĐO LẠI 2026-08-29: KHÔNG dính — mệnh đề cũ SAI, sửa tại chỗ** (xem khối ngay dưới) |
 * | `realClick(button)` trên **cùng** nút đó, cùng lượt chạy | **dính** — chế độ đổi sang Workspace, lưới nạp `2` hàng |
 *
 * 🔴 **HAI DÒNG CUỐI BẢNG LÀ MỘT PHÉP ĐO ĐỐI CHỨNG, VÀ NÓ LẬT MỘT MỆNH ĐỀ BẢN ĐẦU ĐÃ KHAI.**
 * Bản đầu của tệp này viết *"dính bình thường — cùng cơ chế `story-5-6-library-grid.e2e.mjs`
 * đang xanh"*, và **cả hai vế đều sai**: (a) đo 2026-08-29 trong cùng một lượt chạy, lượt bấm
 * bằng phím cho `window.__logs` **RỖNG** *(không một dòng nào từ `openChapterById` — tức
 * handler `@click` chưa từng chạy)* trong khi `realClick()` ngay sau đó trên **cùng phần tử**
 * đổi được chế độ và nạp lưới; (b) `story-5-6-library-grid.e2e.mjs` **KHÔNG** xanh — nó đỏ ở
 * `6b2cb24` (baseline, trước story này) với đúng lỗi type-ahead, đo bằng cách `git stash` toàn
 * bộ lượt sửa rồi chạy riêng spec đó.
 *
 * ⇒ `element.focus()` bằng JS **không** làm phần tử đó thành đích của `browser.keys()`:
 * WebDriver gửi phím tới phần tử mà **nó** coi là đang có tiêu điểm, và một lượt focus không
 * đi qua driver không cập nhật trạng thái ấy. Đây là **khuyết tật của bàn đo**, không của sản
 * phẩm — bằng chứng là lượt đối chứng bằng chuột trên cùng nút, cùng phiên, chạy đúng.
 *
 * ⇒ Khuyết tật khoanh vùng đúng vào **tiêu điểm DOM trên một `<span contenteditable>`** khi
 * phiên WebDriver chưa từng có tiêu điểm hệ điều hành thật — cùng lớp phát hiện đã ghi ở
 * `deferred-work.md:2557-2564` cho Story 2.3 (*"còn hỏng với tay người hay không CHƯA TRẢ LỜI
 * ĐƯỢC bằng bộ e2e"*). Nút bấm (không phải `contenteditable`) không mắc khuyết tật này.
 *
 * ⇒ Spec này đo đúng phần **mắc lại được**: cơ chế `chapter_position` (ghi/đọc/khôi phục)
 * qua ĐÚNG lệnh IPC của sản phẩm (`save_chapter_position`/`read_open_chapter_segments`), và
 * đường MỞ LẠI (`library.open_work`/`library.open_chapter`) qua nút bấm THẬT — không đoán
 * bằng `document.activeElement` trên một `contenteditable`. Vế "một `<span>` cụ thể có nhận
 * tiêu điểm DOM hay không" ở lại đúng món nợ đã có chủ (`deferred-work.md:2557`, Story 2.3),
 * không phải một mục mới của story này.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO "MỞ LẠI" Ở ĐÂY LÀ MỘT PHÉP ĐO THẬT DÙ KHÔNG RESTART TIẾN TRÌNH
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔵 **VIẾT LẠI 2026-08-29 — bản trước dựa trên một tiền đề KHÔNG ĐO ĐƯỢC.** Nó lập luận rằng
 * gọi `library.open_work` trên **CHÍNH Tác phẩm đang mở** đã là đường thật, vì hàm thuần vẫn
 * chạy trọn. Vế ấy đúng **về mã**, nhưng nó làm cả ca **không phân biệt được**: mọi phép
 * khẳng định phía sau (danh sách Chương, `chapter_id`, vị trí caret) cho ra **kết quả y hệt**
 * khi `open_work` chạy và khi nó **không chạy một dòng nào** — vì Tác phẩm đó vốn đã mở sẵn.
 * Một ca không phân biệt được hai thế giới thì không đo thế giới nào.
 *
 * ⇒ Ca này tạo một Tác phẩm **THỨ HAI** trước khi mở lại Tác phẩm thứ nhất. `create_work` gọi
 * `replace_open_work`, tức `Store` của A **đóng qua `Drop`** và `OpenWorkState` trỏ sang B —
 * A nay chỉ còn là một `.atproj` **nằm trên đĩa**. Mở lại A từ đó là đúng món nợ kiến trúc
 * trung tâm của Epic 5 (`atproj_path` từ `library-index.db` → `WorkMeta::read` → `Store::open`
 * MỘT kết nối mới → `ScopeResolver::with_work` → `replace_open_work`), và `chapter_id` của A
 * khác `chapter_id` của B nên mọi phép khẳng định phía sau **đỏ được**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * §Giới hạn — ghi ra thay vì để người sau tưởng đã phủ
 * ═════════════════════════════════════════════════════════════════════════════════
 * 1. Thư mục gốc Library là MỘT thư mục TẠM DÙNG CHUNG cho cả lượt chạy (`wdio.conf.mjs`
 *    §onPrepare) — cùng giới hạn đã ghi ở `story-5-6-library-grid.e2e.mjs`. Spec này định vị
 *    Tác phẩm của chính nó bằng TÊN (không bằng chỉ số `0`) và di chuyển con trỏ lưới tới
 *    đúng ô đó trước khi bấm "Mở Tác phẩm".
 * 2. Không đo `meta.json` mới hơn / thư mục biến mất — hai ca đó là hợp đồng dữ liệu, đã có
 *    lưới đủ ở `project_contract.rs`.
 * 3. Ghi vị trí caret ở đây đi qua `save_chapter_position` (IPC trần) thay vì gõ/click vào
 *    một câu — xem khối §Giới hạn ở trên. `panels/positionFlush.ts`/`setEditorCaret` (đường
 *    mà một người dùng thật đi qua) không có phép đo THẬT ở bộ e2e vì đúng lý do đó; chúng có
 *    lưới ở `src-tauri/tests/segment_contract.rs` (Rust) và ở hành vi module thuần.
 * 4. 🔴 **Vế "làm được BẰNG BÀN PHÍM" của AC7 KHÔNG được spec này nghiệm thu**, và đó là một
 *    khoảng hở ghi ra chứ không một vế được làm tròn lên. Mọi lượt bấm ở đây đi qua
 *    `realClick()` (chuột thật) vì phép đo ở bảng trên cho thấy `focus()` bằng JS cộng
 *    `browser.keys()` **không tới được phần tử**. Thứ AC7 đòi — `Tab` tới nút rồi `Enter` —
 *    là hành vi **gốc của HTML/WebKit** trên một `<button>` thật mang `@click`, không một
 *    dòng mã sản phẩm nào; nhưng "không có mã để hỏng" **không phải** một phép đo. ⇒ Món nợ
 *    có chủ ở `deferred-work.md` (đường kích hoạt bằng bàn phím trong bộ e2e), không phải một
 *    vế được coi là đạt ở đây.
 */

import { realClick } from '../support/pointer.mjs'

const RUN_TAG = `${Date.now() % 1_000_000}`
const WORK_NAME = `e2e-open-chapter-${RUN_TAG}`
// zh — bộ tách nhìn `。！？；`. Hai câu là đủ: một để LÀM VIỆC (lưu vị trí), một để phân biệt
// với "segment đầu" (AC5's giá trị hồi phòng) — hai giá trị khác nhau chứng minh vị trí THẬT
// SỰ đã được đọc lại, không phải một trùng hợp.
const SOURCE_TEXT = 'Cau thu nhat。Cau thu hai。'
// Tác phẩm B chỉ tồn tại để ĐẨY A ra khỏi `OpenWorkState` — xem khối đối chứng âm trong ca.
// MỘT câu, cố ý: `segments.length` khác A (hai câu) là một dấu hiệu THỨ HAI phân biệt hai kho,
// độc lập với `chapter_id`.
const WORK_NAME_B = `e2e-open-chapter-b-${RUN_TAG}`
const SOURCE_TEXT_B = 'Chi mot cau。'

const WORK_NEXT_BTN = '[data-library-work-next]'
const OPEN_WORK_BTN = '[data-library-open-work]'
const CHAPTER_ROWS = '.chapters-list [data-library-chapter-row]'
const OPEN_CHAPTER_BTN = '[data-library-open-chapter]'
const LIST_WORKS_BTN = '[data-lifecycle-action="list_works"]'

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

/** Đọc lại segment của Chương đang mở bằng ĐÚNG lệnh IPC của sản phẩm. */
async function readSegmentsFromDisk() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('read_open_chapter_segments', {})
  })
}

/** Ghi vị trí caret của một Chương qua ĐÚNG lệnh IPC của sản phẩm — `save_chapter_position`.
 * Xem khối §Giới hạn đầu tệp cho vì sao đi thẳng IPC thay vì gõ/click vào một câu. */
async function saveChapterPositionViaIpc(chapterId, segmentId) {
  return browser.execute(
    async (cid, sid) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) throw new Error('không có cầu IPC trong webview')
      // ⚠️ Tên THAM SỐ lệnh đi camelCase — `invoke()` chỉ đổi tên ở cấp đỉnh.
      return internals.invoke('save_chapter_position', { chapterId: cid, segmentId: sid })
    },
    chapterId,
    segmentId,
  )
}

/** Chụp trạng thái lưới Tác phẩm cần cho spec này, trong đúng một lệnh WebDriver. */
async function workGridProbe() {
  return browser.execute(() => {
    const cells = Array.from(document.querySelectorAll('.works-grid .work-cell'))
    return {
      names: cells.map((cell) => (cell.querySelector('.work-name')?.textContent || '').trim()),
      currentIndex: cells.findIndex((cell) => cell.getAttribute('aria-current') === 'true'),
    }
  })
}

/** Di chuyển con trỏ lưới Tác phẩm (bằng phím `Enter` trên nút `›`) tới đúng ô mang `name`. */
async function moveWorkCursorTo(name) {
  await browser.waitUntil(
    async () => (await workGridProbe()).names.includes(name),
    { timeout: 30_000, timeoutMsg: `Tác phẩm "${name}" không xuất hiện trong lưới sau 30 giây` },
  )
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

describe('Story 5.7 — mở lại một .atproj đã có trên đĩa và khôi phục vị trí làm việc', () => {
  it('tạo Tác phẩm, lưu vị trí câu thứ hai, mở lại qua library.open_work, và vị trí khôi phục đúng câu đó', async () => {
    // ── Chờ cầu IPC — cùng khuôn `support/workspace.mjs`. ───────────────────────────
    await browser.waitUntil(
      async () => browser.execute(() => window.__TAURI_INTERNALS__ !== undefined),
      { timeout: 30_000, interval: 250 },
    )

    await createWork(WORK_NAME, SOURCE_TEXT)

    // ── Vẫn ở Library — đọc segment rồi LƯU vị trí câu THỨ HAI qua IPC trần. ─────────
    // AC5 (Chương chưa từng mở) — trước lượt ghi, vị trí phải là segment ĐẦU.
    const beforeAny = await readSegmentsFromDisk()
    expect(beforeAny.segments.length).toBeGreaterThanOrEqual(2)
    const firstSegment = beforeAny.segments[0]
    const secondSegment = beforeAny.segments[1]
    expect(beforeAny.caret_segment_id).toBe(firstSegment.id)

    await saveChapterPositionViaIpc(beforeAny.chapter_id, secondSegment.id)

    // AC4 — đọc lại NGAY (cùng phiên, chưa mở lại) phải thấy đúng vị trí vừa ghi.
    const afterSave = await readSegmentsFromDisk()
    expect(afterSave.caret_segment_id).toBe(secondSegment.id)

    // ═══════════════════════════════════════════════════════════════════════════════
    // 🔴 ĐẨY TÁC PHẨM A RA KHỎI `OpenWorkState` — ĐỐI CHỨNG ÂM, VÀ NÓ LÀ THỨ LÀM CẢ
    //    SPEC NÀY CÓ NGHĨA
    // ═══════════════════════════════════════════════════════════════════════════════
    // 🔵 **THÊM 2026-08-29, sau khi đo được rằng bản đầu KHÔNG đo gì ở bước then chốt.**
    // Bản đầu gọi `library.open_work` trên **chính Tác phẩm đang mở** rồi khẳng định *"danh
    // sách Chương hiện đúng một hàng"*. Phép khẳng định đó **đúng dù `open_work` có chạy hay
    // không**: `LibraryMode.vue` tự gọi `loadChapters()` ở `onActivated`, và Tác phẩm A vẫn
    // đang mở từ lượt `create_work` — nên một lượt `open_work` **trượt hoàn toàn** vẫn cho ô
    // xanh. Đo được ngay lượt đầu: lượt bấm bằng phím không tới được nút, `window.__logs`
    // rỗng, mà hàng Chương vẫn hiện đủ.
    //
    // ⇒ Tạo một Tác phẩm THỨ HAI. `create_work` gọi `replace_open_work`, tức `Store` của A
    // **đóng qua `Drop`** và `OpenWorkState` trỏ sang B. Từ điểm này, mở lại A là một lượt
    // đọc `.atproj` **thật sự từ đĩa** — đúng món nợ kiến trúc mà story này đóng — và mọi
    // phép khẳng định phía sau **phân biệt được** A với B bằng `chapter_id`.
    await createWork(WORK_NAME_B, SOURCE_TEXT_B)

    // 🔴 **PHÂN BIỆT BẰNG `segments`, KHÔNG BẰNG `chapter_id` — và đây là một phép đo, không
    // một gu.** Lượt viết đầu của khối này khẳng định `whileBIsOpen.chapter_id !==
    // beforeAny.chapter_id` và **ĐỎ NGAY**: `chapter.id` là `INTEGER PRIMARY KEY
    // AUTOINCREMENT` **cục bộ trong từng `project.db`** (doc-comment của
    // `commands/chapter.rs::OpenChapter::chapter_id` nói đúng câu đó: *"chỉ có nghĩa trong
    // `project.db` của chính Tác phẩm đang mở"*), nên A và B **đều** có `chapter_id = 1`.
    // ⇒ Dấu hiệu phân biệt hai kho phải là NỘI DUNG: A hai câu, B một câu.
    const whileBIsOpen = await readSegmentsFromDisk()
    expect(whileBIsOpen.segments).toHaveLength(1)
    expect(whileBIsOpen.segments[0].source_text).not.toBe(firstSegment.source_text)

    // ── Làm mới lưới Tác phẩm rồi đưa con trỏ tới Ô CỦA A. ──────────────────────────
    // `create_work` qua IPC trần KHÔNG đi qua `libraryImport.ts`, nên `watch(createdWork)`
    // không bắn và lưới chưa biết về B. Bấm "Tải danh sách" là đường làm mới TƯỜNG MINH của
    // sản phẩm — không một lượt đổi chế độ chỉ để kích `onActivated`.
    await realClick(await $(LIST_WORKS_BTN))
    await moveWorkCursorTo(WORK_NAME)

    // ── "Mở Tác phẩm" — `library.open_work`, đường MỞ LẠI thật. ─────────────────────
    const openWorkBtn = await $(OPEN_WORK_BTN)
    await realClick(openWorkBtn)

    // 🔴 ĐỐI CHỨNG DƯƠNG của lượt mở lại: Chương đang mở phải QUAY VỀ của A, không còn của
    // B. Không một `loadChapters()` tự động nào dựng được mệnh đề này — nó chỉ đúng khi
    // `open_work` đã thật sự chạy trọn (phân giải `atproj_path` → `WorkMeta::read` →
    // `Store::open` mới → `replace_open_work`).
    await browser.waitUntil(
      async () => {
        const now = await readSegmentsFromDisk()
        return now.segments.length === beforeAny.segments.length
      },
      {
        timeout: 20_000,
        interval: 250,
        timeoutMsg:
          '`library.open_work` không đưa Tác phẩm A trở lại `OpenWorkState` sau 20 giây — ' +
          'Chương đang mở vẫn mang MỘT câu, tức vẫn là của Tác phẩm B',
      },
    )

    // AC2 — danh sách Chương của Tác phẩm vừa mở lại hiện ĐÚNG một hàng.
    await browser.waitUntil(
      async () => (await browser.execute((sel) => document.querySelectorAll(sel).length, CHAPTER_ROWS)) === 1,
      { timeout: 20_000, timeoutMsg: 'danh sách Chương không hiện đúng một hàng sau `library.open_work`' },
    )

    // ── "Mở Chương" BẰNG BÀN PHÍM — `library.open_chapter`. ─────────────────────────
    //
    // 🔴 **ĐÚNG MỘT lượt bấm, KHÔNG một vòng bấm lại** — và đây là chỗ bàn đo này đã suýt
    // nói dối, ghi ra thay vì để người sau đi lại.
    //
    // 🔵 **SỬA 2026-08-29, sau lượt e2e ĐỎ đầu tiên.** Bản đầu bọc lượt bấm trong một
    // `waitUntil` **bấm lại mỗi 500 ms trong 20 giây**, kèm một chú thích khai rằng lượt chặn
    // là *"đúng hành vi sản phẩm đã định"*. Cả hai đều sai, và cái sai thứ hai đẻ ra cái thứ
    // nhất: `openChapterById` chặn vì `editorHasLoaded()` — một vị từ đọc `chapterId.value`
    // mà **chỉ** `GridPanel.vue::onMounted` đặt. Ở màn hình Library lúc khởi động lạnh,
    // Workspace **chưa mount lần nào**, nên cửa ấy **không bao giờ mở**: một vòng bấm lại 20
    // giây cũng không cứu được, nó chỉ đổi một lượt đỏ TỨC THÌ thành một lượt đỏ CHẬM, và
    // trên một máy khác nó sẽ đổi thành một lượt XANH CHẬP CHỜN.
    //
    // ⇒ Cửa ấy đã được GỠ khỏi mã sản phẩm (`editorPanelState.ts::openChapterById`), và bàn
    // đo quay về đúng vai của nó: **một** thao tác người dùng, **một** phép khẳng định. Một
    // vòng bấm lại ở đây là đúng thứ `e2e/AGENTS.md` cấm bằng chữ — *"đừng vá bằng
    // `continue-on-error` hay một vòng chạy lại, cả hai biến job thành thứ không bao giờ đỏ"*.
    const openChapterBtn = await $(OPEN_CHAPTER_BTN)
    await realClick(openChapterBtn)

    await browser.waitUntil(
      async () => browser.execute(() => document.querySelectorAll('[data-col="src"]').length > 0),
      {
        timeout: 20_000,
        interval: 250,
        timeoutMsg:
          'mở Chương từ Library không đổi sang Workspace/nạp lưới sau 20 giây — MỘT lượt bấm phải đủ',
      },
    )

    // ── AC4/AC6 — MỆNH ĐỀ CHÍNH: sau một lượt MỞ LẠI THẬT (đóng `Store` cũ qua `Drop`,
    //    mở một `Store` MỚI, phân giải lại `chapter_id`), vị trí đã lưu đọc lại ĐÚNG câu
    //    thứ hai — không rơi về câu đầu (giá trị hồi phòng của AC5). ───────────────────
    const reopened = await readSegmentsFromDisk()
    // Nội dung là dấu hiệu kho, `chapter_id` thì KHÔNG (xem khối 🔴 ở trên).
    expect(reopened.segments.map((s) => s.source_text)).toEqual(
      beforeAny.segments.map((s) => s.source_text),
    )
    expect(reopened.caret_segment_id).toBe(secondSegment.id)
    expect(reopened.caret_segment_id).not.toBe(firstSegment.id)

  })
})
