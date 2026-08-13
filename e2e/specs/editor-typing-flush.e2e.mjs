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
/** Khớp `EDITOR_IDLE_MS` của `src/panels/editorFlush.ts` (2 000 ms) + một khoảng dư. */
const FLUSH_WAIT_MS = 3_500

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

    const sentences = await $$('[data-segment-id]')
    await expect(sentences.length).toBeGreaterThan(0)
    const before = await readSegmentsFromDisk()
    const targetId = before.segments[0].id

    // ── ① `.doc` KHÔNG gõ được, và trước lượt bấm nào thì không câu nào gõ được ──────
    const docEditable = await browser.execute(
      () => document.querySelector('.doc')?.getAttribute('contenteditable') ?? 'absent',
    )
    await expect(docEditable).toBe('absent')
    // ⚠️ **Không** khẳng định *"0 câu gõ được trước lượt bấm"*. Đo được 2026-08-13: một lượt
    // `selectionchange` lúc vào Workspace có thể đã đặt caret vào một câu, và khi đó vùng gõ lên
    // trước cú bấm — **đúng hành vi**, không một khuyết tật. Mệnh đề thật của bước này là
    // *"`.doc` không bao giờ là editing host"*, và nó ở dòng trên.
    const editableBefore = await browser.execute(
      () => document.querySelectorAll('[contenteditable="true"]').length,
    )
    await expect(editableBefore).toBeLessThanOrEqual(1)

    // ── ② Bấm CHUỘT THẬT vào câu đầu ⇒ đúng MỘT vùng gõ, và nó là câu đó ─────────────
    const first = await $(`[data-segment-id="${targetId}"]`)
    await realClick(first)
    await browser.pause(200)

    const zone = await browser.execute(() => {
      const editable = [...document.querySelectorAll('[contenteditable="true"]')]
      return {
        count: editable.length,
        id: editable[0]?.getAttribute('data-segment-id') ?? null,
        tag: editable[0]?.tagName ?? null,
        // 🔴 Vế quyết định AC21: `isTypingZone` của `keys.ts` đọc CHÍNH thuộc tính này, và một
        // `<span>` KHÔNG được nhánh `tagName` của nó cứu.
        isContentEditable: editable[0]?.isContentEditable ?? null,
      }
    })
    await expect(zone.count).toBe(1)
    await expect(String(zone.id)).toBe(String(targetId))
    await expect(zone.tag).toBe('SPAN')
    await expect(zone.isContentEditable).toBe(true)

    // ── ③ Gõ, qua ĐÚNG `beforeinput` mà một phím vật lý sinh ra ──────────────────────
    const typed = 'Bản dịch gõ trong WKWebView thật.'
    const inserted = await browser.execute((text) => {
      const ok = document.execCommand('insertText', false, text)
      return { ok, text: document.querySelector('[contenteditable="true"]')?.textContent ?? null }
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
      [...document.querySelectorAll('[data-segment-id]')].map((el) =>
        Number(el.getAttribute('data-segment-id')),
      ),
    )
    await expect(ids.length).toBe(before.segments.length)

    // ── ④ Chờ nhịp flush của AD-35, rồi đọc lại từ ĐĨA qua chính lệnh đọc của sản phẩm ──
    await browser.pause(FLUSH_WAIT_MS)
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

  it('`Enter` KHÔNG tách câu — cấu trúc đoạn là dữ liệu đã lưu (AD-37)', async () => {
    await openWorkspaceWithWork('Story 2.3 — Enter bị chặn')

    const before = await readSegmentsFromDisk()
    const targetId = before.segments[0].id
    await realClick(await $(`[data-segment-id="${targetId}"]`))
    await browser.pause(200)

    // Vế PHÍM: `browser.keys` bắn `keydown` thật, và đó đúng là thứ `onEditKeydown` canh.
    await browser.keys(['Enter'])
    await browser.keys(['Enter'])
    // Vế SỰ KIỆN SOẠN THẢO: `insertParagraph` là inputType mà một `Enter` thật sinh ra — nhánh
    // ① của `onBeforeInput` phải chặn nó, kể cả khi lượt phím đã bị chặn ở tầng trên.
    await browser.execute(() => {
      document.execCommand('insertParagraph')
    })
    await browser.pause(300)

    const ids = await browser.execute(() =>
      [...document.querySelectorAll('[data-segment-id]')].map((el) =>
        Number(el.getAttribute('data-segment-id')),
      ),
    )
    await expect(ids.length).toBe(before.segments.length)
    await expect(new Set(ids).size).toBe(ids.length)

    // Và không `<br>` nào bị tiêm vào TRONG câu đang gõ.
    // ⚠️ `?? 0`, không `?? -1`: vế *"vùng gõ tồn tại"* đã được ca thứ nhất khẳng định bằng bốn
    // assert. Ở đây mệnh đề là *"không có `<br>` nào"*, và một vùng gõ vắng mặt cũng thoả nó —
    // đọc `-1` thành một ca đỏ là trộn hai mệnh đề vào một phép kiểm.
    const brInside = await browser.execute(
      () => document.querySelector('[contenteditable="true"]')?.querySelectorAll('br').length ?? 0,
    )
    await expect(brInside).toBe(0)
  })
})
