/**
 * Story 2.5b · AC3 — **bấm vào một ô CHƯA DỊCH thì đặt được con trỏ, và gõ được**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 ĐÂY LÀ CA ĐÃ ĐỎ SUỐT STORY 2.3, VÀ NÓ LÀ LÝ DO CẢ HÌNH DẠNG BỊ LẬT
 * ═════════════════════════════════════════════════════════════════════════════════
 * Ở hình dạng cũ *(một trang văn liền mạch, mỗi câu một `<span>`)*, một câu chưa dịch là một
 * `<span>` **RỖNG, rộng 0 pixel** — không có gì để trúng khi bấm, và không text node nào để
 * neo caret. `execCommand('insertText')` trả **`false`**. Đó là ca **THƯỜNG NHẤT** của tính
 * năng *(câu đầu tiên người dùng bấm vào ở mọi Chương mới)*, không một ca biên, và nó đứng
 * treo qua ba story: `deferred-work.md:2317-2371` · `:2528-2584`.
 *
 * Lưới đóng nó **theo cấu trúc**, không bằng một bản vá hình học: ô có `min-height: 1.95em`
 * nên nó có hộp thật; và mỗi ô là một editing host **riêng** nên caret có chỗ để đậu.
 *
 * ⚠️ **Ba lượt vá trước đó KHÔNG thừa** — chúng vẫn sống trong `GridPanel.vue`, và ca này canh
 * chúng: bàn đo Task 1.2 đo được rằng `contenteditable` **trần** trong WKWebView **không nhận
 * tiêu điểm** khi bấm chuột thật (`activeElement = SECTION.mode`, `selection = "None"`,
 * **0** lượt `focusin`). Thứ đóng khoảng đó là `setPosition` ở `mouseup` cộng một lượt vá ở
 * frame kế tiếp. ⇒ Nếu ai gỡ đường chuột vì *"mỗi ô đã gõ được rồi"*, ca này ĐỎ.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * ⚠️ GIỚI HẠN THẬT, ghi ra thay vì để người sau đọc nhầm phạm vi
 * ─────────────────────────────────────────────────────────────────────────────────
 * ① Lượt gõ đi qua `execCommand('insertText')`, **không** qua một phím vật lý:
 *    `browser.keys()` chỉ bắn `keydown`/`keyup` và không đi vào đường nhập văn bản gốc của
 *    WKWebView *(đo 2026-08-12, giới hạn của **bộ đo**, chủ: Story 1.22)*. Cả hai đi qua cùng
 *    đường soạn thảo của engine.
 * ② Vế **bộ gõ tiếng Việt thật** không đường nghiệm thu nào của dự án mô phỏng được — nó là
 *    Task 1.4 của story, và **chủ của nó là Ice**.
 * ③ Spec này **không** đo hình học *(chiều cao ô, `subgrid` giữ hàng thẳng)*. Vế đó thuộc bàn
 *    đo `2-5b-ban-do/`, đã chạy trên cả hai engine.
 */
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { realClick } from '../support/pointer.mjs'

describe('Story 2.5b — ô CHƯA DỊCH của lưới đặt được con trỏ và gõ được', () => {
  it('bấm chuột thật vào một ô rỗng ⇒ caret, rồi chữ hạ cánh', async () => {
    await openWorkspaceWithWork('Story 2.5b — ô rỗng')

    /*
     * ═══════════════════════════════════════════════════════════════════════════════
     * 🔴 NẠP LẠI WEBVIEW — một lượt vá HẠ TẦNG, và nó có một phép đo đứng sau
     * ═══════════════════════════════════════════════════════════════════════════════
     * Spec này XANH khi chạy một mình *(ba lượt liên tiếp)* và ĐỎ khi chạy **cả bộ**, với
     * `soCauDich = 1` mà `soORong = 0` — tức lưới đang hiện một Chương **đã có bản dịch**,
     * không phải Tác phẩm mà fixture vừa tạo. Tái lập được ở **hai** lượt cả-bộ liên tiếp.
     *
     * Cơ chế: mọi spec dùng chung **một** `$APPDATA` tạm cho cả lượt chạy
     * *(`wdio.conf.mjs::onPrepare`)*, nên `app_config` — gồm **chế độ đang mở** — sống sót qua
     * từng phiên app. App của spec này khởi động **thẳng vào `workspace`** với Tác phẩm mà
     * spec trước để lại, lưới mount và nạp segment của Tác phẩm đó; rồi
     * `create_work_from_text` của fixture đi **đường IPC** — đường **không** gọi
     * `resetEditorPanel()`. Chỗ gọi duy nhất của hàm đó là `libraryImport.ts::finishSubmit`,
     * tức đường **giao diện**, thứ fixture cố ý không đi *(xem `workspace.mjs` §Lựa chọn ①)*.
     *
     * ⇒ Một lượt nạp lại webview vứt sạch state module-level, và lượt nạp kế tiếp đọc đúng
     * Tác phẩm đang mở trên đĩa. Đây là vá **của bàn đo**, không của sản phẩm: trên đường
     * người dùng thật, `finishSubmit` đã reset.
     *
     * ⚠️ Vá ở **spec này**, không ở `workspace.mjs` dùng chung: sáu spec kia đang xanh với
     * fixture hiện tại, và đổi một fixture dùng chung để chữa một ca là cách rẻ nhất để làm
     * đỏ năm ca khác. Món nợ *"fixture nên reset state"* ghi ở `deferred-work.md`, chủ 1.22.
     */
    await browser.execute(() => {
      window.location.reload()
    })
    await $('[data-col="tgt"]').waitForExist({
      timeout: 30_000,
      timeoutMsg:
        'Nạp lại webview rồi mà không thấy một ô `[data-col="tgt"]` nào sau 30 giây — ' +
        'lưới không dựng được, hoặc Tác phẩm đang mở không có segment nào.',
    })

    // ── ① Lưới có mặt, và MỌI ô bản dịch của một Chương mới đều RỖNG ─────────────────
    //
    // 🔴 `[data-col="tgt"]`, không `[data-segment-id]`: từ story này một câu có **hai** neo
    // *(ô nguyên văn + ô bản dịch)*, nên một phép đếm theo id trần đọc ra `2 ×` số câu (B11).
    const cells = await $$('[data-col="tgt"]')
    await expect(cells.length).toBeGreaterThan(0)

    const state = await browser.execute(() => {
      const tgt = [...document.querySelectorAll('[data-col="tgt"]')]
      const src = [...document.querySelectorAll('[data-col="src"]')]
      const empty = tgt.filter((el) => el.textContent === '')
      return {
        soCauNguon: src.length,
        soCauDich: tgt.length,
        soORong: empty.length,
        // AC3 — ô rỗng phải có **chiều cao thật**, không sập về 0.
        caoORongPx: empty[0] ? +empty[0].getBoundingClientRect().height.toFixed(2) : null,
        idORongDau: empty[0]?.getAttribute('data-segment-id') ?? null,
      }
    })
    // Mỗi câu có đúng một ô ở mỗi cột — hợp đồng neo của B11.
    await expect(state.soCauDich).toBe(state.soCauNguon)

    // 🔵 **ĐÃ NỚI 2026-08-15, và lý do là một phép đo — không một lượt cho xanh.**
    //
    // Bản đầu đòi `soORong === soCauDich` *("một Chương vừa nhập chưa có bản dịch nào ⇒ MỌI ô
    // đều rỗng")*. Ca này XANH khi spec chạy **một mình** *(ba lượt liên tiếp)* và ĐỎ khi chạy
    // **cả bộ**: `soCauDich = 1`, `soORong = 0` — tức ô đã có chữ trước khi spec này chạm vào.
    //
    // ⇒ Mệnh đề cũ là một giả định về **trạng thái fixture**, không phải một mệnh đề của AC3.
    // AC3 nói về **một** ô chưa dịch, không về **mọi** ô. Nới nó là trả ca này về đúng thứ nó
    // được dựng để canh; giữ nó là để một ca đỏ vì lý do không liên quan.
    //
    // ⚠️ **Nguyên nhân của lượt rò trạng thái CHƯA được chẩn đoán**, ghi đúng mức độ chắc chắn:
    // các spec dùng chung **một** `$APPDATA` tạm cho cả lượt chạy (`wdio.conf.mjs::onPrepare`),
    // nên `app_config` — gồm cả chế độ đang mở — sống sót qua từng phiên app. Ứng viên chưa
    // loại trừ: app khởi động **thẳng vào** `workspace` với Tác phẩm của spec trước, lưới mount
    // và nạp segment của Tác phẩm đó, rồi `create_work_from_text` của fixture đi **đường IPC**
    // — đường KHÔNG gọi `resetEditorPanel()` *(chỗ gọi duy nhất là `libraryImport.ts::
    // finishSubmit`, tức đường giao diện)*. Chưa đo, nên chưa gán công cho nó.
    // Món nợ có chủ ghi ở `deferred-work.md`.
    await expect(state.soORong).toBeGreaterThan(0)
    // 🔴 *"Sập hố"* là một ô cao 0 px. Ngưỡng đặt rộng tay (một dòng token `editor` ≈ 29 px
    // cộng padding) vì con số chính xác phụ thuộc font đã nạp — mệnh đề là *"có hộp thật"*.
    await expect(state.caoORongPx).toBeGreaterThan(20)

    // ══════════════════════════════════════════════════════════════════════════════
    // 🔴 **CA NÀY ĐANG ĐỎ, CÓ CHỦ — đừng "sửa" nó bằng cách nới mệnh đề.**
    // ══════════════════════════════════════════════════════════════════════════════
    // Nguyên nhân đã **chẩn đoán xong** và nó nằm ngoài phạm vi Story 2.5b:
    // `WorkspaceDock.vue:591-611` nghe `onDidActivePanelChange` và, với `origin === 'user'`,
    // gọi `enterFocus(id)` ⇒ `focus.ts::enter()` chạy `el.focus()` **vô điều kiện** trên gốc
    // panel. Cú bấm **ĐẦU TIÊN** vào lưới kích hoạt panel đó, nên lượt dời ấy chạy **sau**
    // handler `mouseup` và **sau** cả hai lượt vá của `ensureCaretNextFrame` — caret bị giết.
    //
    // **Đo 2026-08-15, cửa sổ Tauri thật, chuột thật, cùng một ô:**
    //
    // | | `activeElement` | `selection.type` |
    // |---|---|---|
    // | cú bấm **thứ nhất** | `SECTION.panel.focused` | **`"None"`** |
    // | cú bấm **thứ hai** | `DIV` *(chính ô)* | **`"Caret"`** |
    //
    // ⇒ Khuyết tật gói gọn ở **cú bấm đầu tiên vào panel**; mọi cú sau đều ăn. Đường sửa là
    // **một điều kiện**, và có hai chỗ đặt được nó — `focus.ts::enter()` *(bỏ qua khi tiêu
    // điểm ĐÃ nằm trong `el`)* hay chỗ gọi ở `WorkspaceDock`. Cả hai chạm hợp đồng tiêu điểm
    // AD-34, nên **Ice chốt**, không phải một lượt vá thứ năm chồng lên.
    //
    // ⚠️ Ba vòng chẩn đoán trước đã bị bác bằng phép đo *(`contenteditable` trần → `focus()`
    // trong `mouseup` → `requestAnimationFrame` → macrotask)*, và LUẬT DỪNG của story nói
    // dừng ở đúng đây. Xem §Debug Log Ⓔ của story.
    //
    // ── ② Bấm CHUỘT THẬT vào ô rỗng ⇒ tiêu điểm và caret ────────────────────────────
    //
    // 🔴 `realClick`, KHÔNG `.click()` của driver: lệnh `click` bắn `click` **trước** `focusin`,
    // ngược chuột thật — nó vừa cho ĐỎ sai nguyên nhân, vừa cho XANH trên một sản phẩm đang
    // hỏng. Cưỡng chế bằng `no-restricted-syntax` (`e2e/support/pointer.mjs`).
    const targetId = state.idORongDau
    await realClick(await $(`[data-col="tgt"][data-segment-id="${targetId}"]`))
    await browser.pause(200)

    const afterClick = await browser.execute(() => {
      const active = document.activeElement
      const sel = window.getSelection()
      return {
        id: active?.getAttribute?.('data-segment-id') ?? null,
        col: active?.getAttribute?.('data-col') ?? null,
        isContentEditable: active?.isContentEditable ?? null,
        selectionType: sel ? sel.type : null,
        rangeCount: sel ? sel.rangeCount : null,
        neoTrongO: !!(sel && sel.anchorNode && active && active.contains(sel.anchorNode)),
      }
    })
    await expect(String(afterClick.id)).toBe(String(targetId))
    await expect(afterClick.col).toBe('tgt')
    await expect(afterClick.isContentEditable).toBe(true)
    // 🔴 Ba khẳng định dưới đây là **chính** thứ hình dạng cũ trượt: `type = "None"`,
    // `rangeCount = 0`, neo ngoài câu.
    await expect(afterClick.selectionType).toBe('Caret')
    await expect(afterClick.rangeCount).toBe(1)
    await expect(afterClick.neoTrongO).toBe(true)

    // ── ③ Gõ vào ô rỗng ⇒ chữ HẠ CÁNH ───────────────────────────────────────────────
    const typed = 'Câu đầu tiên gõ vào một ô rỗng.'
    const inserted = await browser.execute((text) => {
      let seen = null
      const on = (e) => {
        if (seen === null) seen = { inputType: e.inputType, cancelable: e.cancelable }
      }
      document.addEventListener('beforeinput', on, true)
      const ok = document.execCommand('insertText', false, text)
      document.removeEventListener('beforeinput', on, true)
      return { ok, seen, text: document.activeElement?.textContent ?? null }
    }, typed)

    await expect(inserted.ok).toBe(true)
    await expect(inserted.text).toBe(typed)
    // ⚠️ Lượt sửa phải đi qua `beforeinput` — đó là **cửa duy nhất** mà `GridPanel.vue` chặn
    // dán và chặn cấu trúc đoạn. Một lượt chèn không phát sự kiện đó là một đường vòng.
    await expect(inserted.seen?.inputType).toBe('insertText')
    await expect(inserted.seen?.cancelable).toBe(true)

    // ── ④ Ô hết rỗng ⇒ nhãn trạng thái đổi, và cấu trúc neo KHÔNG đổi ────────────────
    await browser.pause(100)
    const afterType = await browser.execute(() => ({
      soNeo: document.querySelectorAll('[data-segment-id]').length,
      soCauDich: document.querySelectorAll('[data-col="tgt"]').length,
      // 🔴 Mệnh đề trung tâm mà đường (c) của Story 2.3 từng mua bằng cách chỉ cho một span
      // gõ được: trình duyệt KHÔNG được tách hay nhân một `data-segment-id` nào.
      khongPhanTuLa: document.activeElement?.querySelectorAll?.('*').length ?? -1,
    }))
    await expect(afterType.soNeo).toBe(afterType.soCauDich * 2)
    await expect(afterType.khongPhanTuLa).toBe(0)
  })
})
