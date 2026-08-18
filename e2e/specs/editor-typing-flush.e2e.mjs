/**
 * Bàn đo Story 2.3 — **vùng gõ lên đúng một câu trong WKWebView THẬT.**
 *
 * ⚠️ Tên tệp và tên `describe` cố ý **không** hứa *"chữ chạm đĩa"*: vế đó chưa nghiệm thu được ở
 * đây, và khối §DỪNG Ở ĐÂY trong ca thứ nhất ghi đủ chuỗi phép đo cùng chủ của món nợ.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY RA ĐỜI, VÀ NÓ ĐO ĐÚNG CÁI GÌ
 * ═════════════════════════════════════════════════════════════════════════════════
 * Mọi bằng chứng khác của story này chạy ở một chỗ **không phải** WKWebView:
 *   - `tests/frontend/**` chạy trên `happy-dom` — một bản mô phỏng DOM trong Node;
 *   - bàn đo `2-3-ban-do-vung-go.html` chạy trên **WebKit của Playwright** — khác phiên bản
 *     và khác lớp nhúng so với **WKWebView của Tauri**;
 *   - `src-tauri/tests/segment_contract.rs` không có webview nào.
 *
 * Đó đúng là món nợ mà `deferred-work.md:2127-2134` ghi cho Story 2.2 và story này **kế thừa**.
 * Spec này đóng phần đóng được của nó: hành vi `contenteditable` trên engine mà sản phẩm thật
 * sự chạy.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * 🔴 VẾ *"ĐÓNG APP → MỞ LẠI → CHỮ CÒN ĐÓ"* KHÔNG CHẠM TỚI ĐƯỢC, và lý do là một phép đo
 * ─────────────────────────────────────────────────────────────────────────────────
 * Task 7.2 của story viết vế đó bằng chữ. Nó **không cài được hôm nay**, và không vì spec này
 * thiếu sức: **không tồn tại đường mở lại một `.atproj`** trong sản phẩm. `OpenWorkState` khởi
 * tạo `None` mỗi lượt chạy, và cách duy nhất để một Tác phẩm được mở là **tạo mới** nó
 * (`create_work_from_*`). Màn hình mở lại một Tác phẩm thuộc **Epic 5**.
 *
 * ⇒ Vế *"chữ còn đó sau khi nạp lại"* nghiệm thu ở
 * `segment_contract.rs::typed_text_round_trips_through_the_flush_and_the_load_command` — nó ghi
 * rồi đọc lại qua **đúng hai lệnh IPC của sản phẩm**. Spec này đo vế còn lại: **gõ trong
 * WKWebView có đi tới `project.db` hay không**, đọc lại bằng chính lệnh đọc của sản phẩm.
 * Một mệnh đề, một đường (AC25).
 *
 * ⚠️ Mọi lượt bấm đi qua `realClick()`. ESLint **cấm** `.click()` trong `e2e/**` từ Story 1.22;
 * lý do và số đo ở `e2e/support/pointer.mjs`.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * 🔴 GIỚI HẠN CỦA BỘ ĐO — `browser.keys()` KHÔNG GÕ ĐƯỢC CHỮ, và đó KHÔNG phải lỗi sản phẩm
 * ─────────────────────────────────────────────────────────────────────────────────
 * Đo 2026-08-12 trong chính cửa sổ này, trên một `<span contenteditable>` **đã có caret thật**:
 *
 * | Đường nhập | Sự kiện quan sát được | `textContent` |
 * |---|---|---|
 * | `browser.keys(['Z'])` | **chỉ** `keydown` *(cả trên `<span>` lẫn `document`)* | **không đổi** |
 * | `document.execCommand('insertText', false, 'K')` | `beforeinput` *(`insertText`, `data: "K"`)* → `input` | `"abc"` → `"abcK"` |
 *
 * ⇒ `contenteditable` của WKWebView **sống bình thường** — đường soạn thảo chạy đủ, `beforeinput`
 * bắn đúng `inputType`, chữ hạ cánh. Thứ **không** đi qua là `browser.keys()`: nó synthesize
 * `keydown`/`keyup` chứ không đi vào đường nhập văn bản gốc của engine.
 *
 * ⇒ Spec này gõ bằng `execCommand('insertText', …)` — **cùng `beforeinput` mà một phím thật sinh
 * ra**, nên nó đi qua đúng `onBeforeInput`/`onEditInput` của sản phẩm. Vế *"một phím vật lý thật
 * sinh ra `beforeinput`"* nằm **ngoài** tầm bộ đo này; nó là một hàng nợ có chủ.
 * ⚠️ `browser.keys()` **vẫn dùng được** cho vế PHÍM TẮT (`Enter` bị chặn) — chỗ đó `keydown` là
 * đúng thứ cần đo.
 */

import { realClick } from '../support/pointer.mjs'
import { waitForGridRows } from '../support/gridWait.mjs'
import { markFlushBaseline, waitForFlushAfter } from '../support/flushWait.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

/**
 * Đọc lại segment bằng ĐÚNG lệnh IPC của sản phẩm — không một đường đọc riêng cho bàn đo.
 *
 * ⚠️ `window.__TAURI_INTERNALS__.invoke`, **không** `await import('@tauri-apps/api/core')`.
 * Đo được ở lượt dựng spec này: một `import()` với **định danh bare** trong `browser.execute`
 * ném `Module name, '@tauri-apps/api/core' does not resolve to a valid URL` — webview không có
 * bộ phân giải của bundler, và mã bơm vào qua WebDriver không đi qua Vite. Fixture
 * `e2e/support/workspace.mjs` đã đi đúng đường đó từ đầu.
 */
/*
 * 🔵 **`FLUSH_WAIT_MS` ĐÃ GỠ 2026-08-18 — Story 2.12 · AC4, và nó GỠ chứ không NỚI.**
 *
 * Hằng cũ là `3_500`, so với `EDITOR_IDLE_MS = 2000` cho biên **1.500 ms** — biên mà một máy
 * đang biên dịch Rust ăn hết *(phân xử Story 2.7: máy bận 7/8 ba lần liên tiếp, máy rảnh 8/8
 * trên cả hai cây)*. Hai chỗ dùng nó nay chờ **mốc lưu đổi** qua `support/flushWait.mjs`.
 *
 * 🔴 Ghi ra vì đường vá SAI ở đây rẻ và trông giống hệt đường đúng: nâng `3_500` lên `8_000`
 * cũng làm bộ hết chập chờn — và làm NFR18 hết được canh. B2 cấm đích danh việc đó.
 */

async function readSegmentsFromDisk() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('read_open_chapter_segments', {})
  })
}

describe('Story 2.3 — vùng gõ MỘT câu, và lượt flush chạm đĩa trong WKWebView thật', () => {
  it('bấm chuột thật ⇒ vùng gõ lên đúng câu, gõ được, và chữ đi vào `project.db`', async () => {
    // Fixture tạo một Tác phẩm bằng IPC rồi vào `workspace`. Văn bản nguồn có hai câu, nên
    // bộ tách của Story 2.1 cho hai hàng `segment`.
    await openWorkspaceWithWork('Story 2.3 — vùng gõ')

    // 🔵 B11 (Story 2.5b): `[data-segment-id]` **KHÔNG còn duy nhất** — một câu có hai neo
    // *(ô nguyên văn + ô bản dịch)*. Mọi phép đếm và mọi `$()` phải nói rõ **CỘT nào**, nếu
    // không phép đếm đọc ra `2 ×` số câu và `$()` trả về **ô nguyên văn**.
    const sentences = await $$('[data-col="tgt"]')
    await expect(sentences.length).toBeGreaterThan(0)
    const before = await readSegmentsFromDisk()

    // 🔵 **STORY 2.12 · AC4** — chụp mốc lưu TRƯỚC khi gõ. Xem `support/flushWait.mjs`:
    // các ca của tệp này dùng CHUNG một phiên app, nên ca thứ hai bắt đầu với một mốc đã
    // khác `null` — một phép chờ *"tới khi có mốc lưu"* sẽ trả về ngay và không đo gì cả.
    const flushBaseline = await markFlushBaseline()
    const targetId = before.segments[0].id

    // ── ① 🔵 MỆNH ĐỀ ĐỔI (Story 2.5b · Quyết định #3b): MỌI ô bản dịch gõ được ─────────
    //
    // Bản trước khẳng định *"`.doc` không bao giờ là editing host"* và *"nhiều nhất một câu
    // gõ được"*. Cả hai thuộc đường (c) của Story 2.3, và **tiền đề của chúng không còn tồn
    // tại**: không còn `.doc`, và mỗi ô nay là một editing host **riêng**.
    //
    // 🔴 Cái mà mệnh đề cũ **mua** thì không mất, nó đổi cơ chế: trình duyệt không được sửa
    // cây `data-segment-id`. Trước: chỉ một span gõ được. Nay: một `Range` soạn thảo **không
    // bắc cầu hai editing host được**. Ca ⑤ ở dưới vẫn khoá đúng mệnh đề đó bằng một phép đếm.
    const editableBefore = await browser.execute(() => {
      const all = [...document.querySelectorAll('[contenteditable="true"]')]
      return {
        count: all.length,
        moiCotDich: all.every((el) => el.getAttribute('data-col') === 'tgt'),
        khongCoDoc: document.querySelector('.doc') === null,
      }
    })
    await expect(editableBefore.count).toBe(sentences.length)
    await expect(editableBefore.moiCotDich).toBe(true)
    await expect(editableBefore.khongCoDoc).toBe(true)

    // ── ② Bấm CHUỘT THẬT vào câu đầu ⇒ đúng MỘT vùng gõ, và nó là câu đó ─────────────
    const first = await $(`[data-col="tgt"][data-segment-id="${targetId}"]`)
    await realClick(first)
    await browser.pause(200)

    // 🔵 Mệnh đề đổi từ *"đúng MỘT vùng gõ, và nó là câu đó"* sang *"TIÊU ĐIỂM lên đúng ô đó,
    // và có caret trong nó"* — thứ thật sự quyết việc người dùng gõ được hay không.
    //
    // 🔴 Đây là chỗ bàn đo Task 1.2 đo được một khuyết tật thật: `contenteditable` **trần**
    // trong WKWebView **không nhận tiêu điểm** khi bấm chuột (`activeElement = SECTION.mode`,
    // `selection = None`). Đường chuột của `GridPanel.vue` là thứ đóng nó — nên ca này canh
    // đúng đường đó, không canh một thuộc tính tĩnh.
    const zone = await browser.execute(() => {
      const active = document.activeElement
      const sel = window.getSelection()
      return {
        id: active?.getAttribute?.('data-segment-id') ?? null,
        col: active?.getAttribute?.('data-col') ?? null,
        // 🔴 Vế quyết định AC21: `isTypingZone` của `keys.ts` đọc CHÍNH thuộc tính này.
        isContentEditable: active?.isContentEditable ?? null,
        selectionType: sel ? sel.type : null,
        neoTrongO: !!(sel && sel.anchorNode && active && active.contains(sel.anchorNode)),
      }
    })
    await expect(String(zone.id)).toBe(String(targetId))
    await expect(zone.col).toBe('tgt')
    await expect(zone.isContentEditable).toBe(true)
    await expect(zone.selectionType).toBe('Caret')
    await expect(zone.neoTrongO).toBe(true)

    // ── ③ Gõ, qua ĐÚNG `beforeinput` mà một phím vật lý sinh ra ──────────────────────
    const typed = 'Bản dịch gõ trong WKWebView thật.'
    const inserted = await browser.execute((text) => {
      const ok = document.execCommand('insertText', false, text)
      // 🔵 B11 — đọc ô đang có TIÊU ĐIỂM, không ô đầu tiên khớp selector.
      return { ok, text: document.activeElement?.textContent ?? null }
    }, typed)
    // 🔴 **CA NÀY ĐANG ĐỎ, CÓ CHỦ — đừng "sửa" nó bằng cách nới mệnh đề.**
    //
    // Đo 2026-08-13 (code review): `execCommand('insertText')` trả **`false`** khi câu đích là
    // một câu **CHƯA DỊCH** — `<span>` rỗng, rộng **0 px**, không text node để neo caret. Fixture
    // của spec này tạo Tác phẩm bằng `create_work_from_text`, và `target_text` khởi tạo là chuỗi
    // rỗng ⇒ nó rơi đúng vào ca đó.
    //
    // ⚠️ Đây là ca **thường nhất** của tính năng, không một ca biên: mọi Chương mới mở ra đều
    // toàn câu rỗng. Nên nó **không** được `skip`, không được đổi thành `toBe(false)`, và không
    // được né bằng một fixture đã có sẵn chữ — cả ba đều biến một khuyết tật sản phẩm thành một
    // dòng xanh. Bộ e2e đỏ ở đây là bộ e2e đang nói thật.
    //
    // Ba hướng đã thử và ba hướng chưa thử: `deferred-work.md` §*CÒN LẠI SAU ĐÍNH CHÍNH*.
    await expect(inserted.ok).toBe(true)
    await expect(inserted.text).toContain(typed)

    // Số câu KHÔNG đổi — gõ không phải một lượt tách (AD-4).
    const ids = await browser.execute(() =>
      // 🔵 B11 — đếm theo CỘT, không theo id trần: `[data-segment-id]` nay khớp `2 ×` số câu.
      [...document.querySelectorAll('[data-col="tgt"]')].map((el) =>
        Number(el.getAttribute('data-segment-id')),
      ),
    )
    await expect(ids.length).toBe(before.segments.length)

    // ── ④ Chờ nhịp flush của AD-35, rồi đọc lại từ ĐĨA qua chính lệnh đọc của sản phẩm ──
    //
    // 🔵 **STORY 2.12 · AC4** — chờ một TRẠNG THÁI, không một khoảng thời gian. Bản cũ là
    // `pause(FLUSH_WAIT_MS)`, và biên 1.500 ms của nó so với `EDITOR_IDLE_MS` bị một máy
    // đang biên dịch Rust ăn hết *(phân xử 2.7: máy bận 7/8 ba lần, máy rảnh 8/8)*.
    await waitForFlushAfter(flushBaseline, { what: 'lượt flush sau khi gõ' })
    const after = await readSegmentsFromDisk()

    const saved = after.segments.find((s) => s.id === targetId)
    await expect(saved).toBeDefined()
    await expect(saved.target_text).toContain(typed)

    // Ranh giới câu KHÔNG đổi — AD-4 đóng băng nó vĩnh viễn.
    await expect(after.segments.length).toBe(before.segments.length)
    await expect(after.segments.map((s) => s.source_text)).toEqual(
      before.segments.map((s) => s.source_text),
    )
    await expect(after.segments.map((s) => s.ord)).toEqual(before.segments.map((s) => s.ord))

    // ── ⑤ Thanh trạng thái nói *"Đã lưu N giây trước"* — AC7 ─────────────────────────
    // ⚠️ `footer.status`, KHÔNG `.status` trần: đo được trong webview thật rằng lớp `status` có
    // **ba** chỗ dùng (vỏ `PanelFrame` mang câu trạng thái của panel cũng dùng nó).
    const status = await browser.execute(
      () => document.querySelector('footer.status')?.textContent?.trim() ?? '',
    )
    await expect(status).toMatch(/^Đã lưu \d+ giây trước$/)
  })

  /*
   * 🔵 **CA NÀY ĐỔI MỆNH ĐỀ 2026-08-16 (Story 2.5d, FR134/AD-46) — viết lại, không xoá.**
   *
   * Tên cũ: *"`Enter` KHÔNG tách câu — cấu trúc đoạn là dữ liệu đã lưu (AD-37)"*, và nó khẳng
   * định `Enter` **bị chặn**. Vế *"không tách câu"* **vẫn đúng và ở lại**; vế *"bị chặn"* đã
   * hết đúng — trong ô bản dịch `Enter` nay **xuống dòng** (AC1).
   *
   * 🔴 **VÀ ĐÂY LÀ ĐƯỜNG DUY NHẤT BẮT ĐƯỢC LỚP LỖI TRUNG TÂM CỦA STORY.** Bàn đo dừng ở DOM;
   * `vitest` chạy trên `happy-dom` với fixture chép tay; `cargo test` không có webview nào.
   * Chỉ ca này đi trọn **phím → DOM → tập chờ → flush → `project.db` → lệnh đọc**, trên engine
   * mà sản phẩm thật sự chạy. Lớp lỗi nó canh: *"DOM có hai dòng, đĩa có một chuỗi liền"* —
   * `textContent` của `<div>a</div><div>b</div>` là `"ab"`, và **không cổng nào đỏ vì chuyện đó**.
   */
  it('`Enter` xuống dòng trong ô, `\n` đi tới `project.db`, và KHÔNG câu nào bị tách', async () => {
    await openWorkspaceWithWork('Story 2.5d — Enter xuống dòng')

    /*
     * 🔴 NẠP LẠI WEBVIEW — cùng lý do và cùng phép đo với `grid-empty-cell.e2e.mjs`.
     *
     * ⚠️ **Lượt vá này KHÔNG thừa, và nó có một phép đo riêng ở chính ca này:** bản đầu không
     * nạp lại và đỏ với `target_text` = `"Bản dịch gõ trong WKWebView thật.Dong mot.\nDong hai."`
     * — tức ô đang mang chữ mà **ca thứ nhất của chính tệp này** vừa gõ. Cả hai ca dùng chung
     * một `$APPDATA` tạm (`wdio.conf.mjs::onPrepare`), nên `app_config` sống sót qua từng phiên
     * app: app khởi động **thẳng vào** `workspace` với Tác phẩm cũ, lưới mount và nạp segment
     * của Tác phẩm đó, rồi `create_work_from_text` của fixture đi đường **IPC** — đường **không**
     * gọi `resetEditorPanel()`.
     *
     * 🔴 Ghi rõ vì nó dễ đọc nhầm thành một khuyết tật sản phẩm: **phép đo của ca này ĐẠT ngay
     * ở lượt đỏ đó** — chuỗi trên đĩa **có** `\n` đúng chỗ. Thứ sai là **điều kiện đầu vào**,
     * không phải cơ chế. Món nợ *"fixture e2e không reset state panel"* có chủ: Story 1.22.
     */
    // 🔵 **STORY 2.12 · AC2 — lượt `reload()` ở đây ĐÃ GỠ 2026-08-18.**
    // Fixture `openWorkspaceWithWork` nay tự dọn state panel bằng `resetPanelState()` (cầu
    // `import()` gọi thẳng năm hàm `reset*`, quyết định #5(b) Ice ký). Vá tại chỗ hết việc.
    // 🔵 **STORY 2.12 · AC3** — chờ TRẠNG THÁI ĐÍCH, không chờ *"phần tử tồn tại"*.
    await waitForGridRows(1, { col: 'tgt', what: 'lưới sau lượt dựng Tác phẩm' })

    const before = await readSegmentsFromDisk()

    // 🔵 **STORY 2.12 · AC4** — chụp mốc lưu TRƯỚC khi gõ. Xem `support/flushWait.mjs`:
    // các ca của tệp này dùng CHUNG một phiên app, nên ca thứ hai bắt đầu với một mốc đã
    // khác `null` — một phép chờ *"tới khi có mốc lưu"* sẽ trả về ngay và không đo gì cả.
    const flushBaseline = await markFlushBaseline()
    const targetId = before.segments[0].id
    await realClick(await $(`[data-col="tgt"][data-segment-id="${targetId}"]`))
    await browser.pause(200)

    // ── ① Gõ, xuống dòng, gõ tiếp ────────────────────────────────────────────────
    //
    // ⚠️ `execCommand`, **không** `browser.keys`: giới hạn của bộ đo đã ghi ở đầu tệp —
    // `browser.keys()` chỉ bắn `keydown`/`keyup` và không đi vào đường nhập văn bản gốc của
    // WKWebView. Cả hai đi qua **cùng** đường soạn thảo của engine, và nhánh ① của
    // `onBeforeInput` chặn theo `inputType` chứ không theo phím.
    await browser.execute(() => {
      document.execCommand('insertText', false, 'Dong mot.')
      document.execCommand('insertParagraph')
      document.execCommand('insertText', false, 'Dong hai.')
    })
    await browser.pause(300)

    // ── ② DOM: hai dòng THẬT, một text node phẳng, KHÔNG markup ───────────────────
    const dom = await browser.execute(() => {
      const cell = document.activeElement
      const r = document.createRange()
      r.selectNodeContents(cell)
      const tops = [...new Set([...r.getClientRects()].map((b) => +b.top.toFixed(2)))]
      return {
        soDong: tops.length,
        soPhanTuCon: cell.querySelectorAll('*').length,
        textContent: cell.textContent,
      }
    })
    // 🔴 `soPhanTuCon === 0` là mệnh đề chống **tiêm markup**: nếu engine dựng `<div>` hay
    // `<br>` thì `textContent` ngay dưới sẽ nuốt mất ranh giới, và đĩa nhận một chuỗi liền.
    await expect(dom.soPhanTuCon).toBe(0)
    await expect(dom.textContent).toBe('Dong mot.\nDong hai.')
    await expect(dom.soDong).toBe(2)

    // ── ③ KHÔNG câu nào bị tách — vế GIỮ NGUYÊN từ bản cũ, và là vế đắt nhất ──────
    const ids = await browser.execute(() =>
      [...document.querySelectorAll('[data-col="tgt"]')].map((el) =>
        Number(el.getAttribute('data-segment-id')),
      ),
    )
    await expect(ids.length).toBe(before.segments.length)
    await expect(new Set(ids).size).toBe(ids.length)

    // ── ④ FLUSH, rồi đọc lại bằng ĐÚNG lệnh IPC của sản phẩm ─────────────────────
    //
    // 🔵 **STORY 2.12 · AC4** — và ĐÂY là ca mà bẫy *"đã có sẵn"* cắn thật: ca trên đã
    // flush trong cùng phiên app, nên mốc lưu khác `null` ngay khi ca này bắt đầu.
    await waitForFlushAfter(flushBaseline, { what: 'lượt flush sau lượt xuống dòng' })
    const after = await readSegmentsFromDisk()
    const saved = after.segments.find((s) => s.id === targetId)

    // 🔴 MỆNH ĐỀ TRUNG TÂM. Một chặng làm phẳng sẽ cho `"Dong mot. Dong hai."` — một chuỗi
    // **hợp lệ trông như thật**, đi trọn xuống đĩa mà không một lỗi nào được ném.
    await expect(saved.target_text).toBe('Dong mot.\nDong hai.')
    await expect(saved.target_text.includes('\n')).toBe(true)

    // ── ⑤ Cờ kết đoạn của BẢN DỊCH đi qua dây, và một `\n` KHÔNG bật nó ──────────
    //
    // 🔴 Hai khái niệm khác nhau: `\n` là xuống dòng **trong** một câu; cờ là ranh giới đoạn
    // **sau** câu (AC4). Nếu một đường mã nào suy cờ từ nội dung, ca này đỏ.
    await expect(typeof saved.is_target_paragraph_end).toBe('boolean')
    await expect(saved.is_target_paragraph_end).toBe(before.segments[0].is_target_paragraph_end)
  })
})
