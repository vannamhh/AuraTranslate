/**
 * Story 3.3 · FR48 — dải "Thêm thuật ngữ", trên WEBVIEW THẬT.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO TỆP NÀY TỒN TẠI, VÀ VÌ SAO NÓ KHÔNG CHÉP LẠI `glossaryQuickAdd.test.ts`
 * ═════════════════════════════════════════════════════════════════════════════════
 * Story 3.3 có một bộ test `tests/frontend/glossaryQuickAdd.test.ts` chạy trên `happy-dom`,
 * và chính story ghi rõ giới hạn của nó (§Tasks & Acceptance, dòng cuối): *"vế THỊ GIÁC và
 * vế VÙNG CHỌN TRÊN ENGINE THẬT thuộc bàn đo tay, không thuộc `happy-dom`"*. `happy-dom`
 * không có hình học (không `getBoundingClientRect` thật) và `Selection`/`Range` của nó là một
 * bản mô phỏng rất hẹp — nó không đi qua đúng `window.getSelection()` mà `surfaceFor()`
 * (`src/panels/selectionContract.ts`) đọc `anchorNode.nodeType` từ đó. Ba mệnh đề dưới đây là
 * đúng BA chỗ mà khoảng trống đó có thật:
 *
 *   ① Dải ĐẨY `.modeport` lên, không PHỦ lên nó — một mệnh đề HÌNH HỌC, đo bằng
 *      `getBoundingClientRect()` trong webview thật.
 *   ② Vùng chọn đọc được ở bề mặt vai `display` — điều kiện khởi hành của story
 *      (`deferred-work.md:2687-2697`), và nó chỉ có ý nghĩa khi `window.getSelection()` là
 *      CỦA ENGINE THẬT, không một `Range` do bàn đo tự dựng bằng tay.
 *   ③ `Esc` trả tiêu điểm VÀ vùng chọn cũ — `restoreFocusAndSelection()`
 *      (`glossaryQuickAddState.ts`) gọi `Selection.removeAllRanges()`/`addRange()` thật, một
 *      API mà `happy-dom` không cài đúng ngữ nghĩa.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO DỮ LIỆU CỦA MỆNH ĐỀ ② ĐI QUA CỘT BẢN DỊCH CỦA LƯỚI, KHÔNG QUA PANEL LOOKUP
 * ═════════════════════════════════════════════════════════════════════════════════
 * Story cho phép cả hai ("chọn cái nào dựng được dữ liệu rẻ hơn"). Panel Lookup ĐẮT hơn theo
 * hai trục: (a) nó cần một kết quả tra từ điển thật hoặc phải chuyển sang tab Lịch sử để có
 * chữ tĩnh, và cả hai đường đều cần bấm một `<button>` trước — mà `App.vue` đã tự ghi nhận
 * *"WKWebView không đặt tiêu điểm cho `<button>` khi bấm chuột"* (UX-DR17), một khuyết tật
 * engine đã buộc `data-shortcuts-open` phải có `@mousedown="focusOnPointerDown"` riêng; nút
 * tab của Panel Lookup KHÔNG có lượt ép đó, nên `document.activeElement` sau khi bấm nó là
 * một ẩn số không đáng đặt cược cho mệnh đề ③ (cần MỘT tiêu điểm biết trước để so sánh sau
 * `Esc`). (b) Cột bản dịch của lưới là một `contenteditable` — nó NHẬN tiêu điểm khi bấm
 * chuột thật (`grid-empty-cell.e2e.mjs` đã đo việc này), nên nó cho **cùng lúc** một tiêu
 * điểm biết trước VÀ một vùng chọn thật trên CÙNG một phần tử, không cần đoán.
 *
 * ⚠️ **Vì sao KÉO CHUỘT (drag), không nhấp đúp (double-click):** nhấp đúp chọn một TỪ theo
 * thuật toán ngắt từ của engine — không đoán trước được từ nào bị chọn nếu ô mang nhiều hơn
 * một từ, và nó buộc phải chọn cụm bằng chính công cụ mình đang muốn xác nhận (vòng luẩn
 * quẩn). Ta gõ một chuỗi NGẮN, không dấu cách, chọn TOÀN BỘ nó, rồi so trực tiếp với chuỗi đã
 * gõ — biết chính xác cụm mong đợi, không suy ngược từ những gì vừa chọn được.
 *
 * ⚠️ **Vì sao toạ độ kéo là OFFSET quanh TÂM phần tử (`origin: element, x, y`), không toạ độ
 * tuyệt đối của viewport:** offset quanh tâm phần tử đi qua đúng cơ chế mà driver dùng để ánh
 * xạ điểm bấm sang màn hình thật (giống `realClick()` ở `pointer.mjs`, chỉ thêm độ lệch) —
 * né hẳn câu hỏi devicePixelRatio của một màn Retina mà một toạ độ viewport tự tính tay bằng
 * `getBoundingClientRect()` rồi bơm thẳng vào `origin: 'viewport'` sẽ phải trả lời.
 */
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { waitForGridRows } from '../support/gridWait.mjs'

/** Ô bản dịch duy nhất của một Tác phẩm mới — `openWorkspaceWithWork` dựng đúng MỘT segment. */
const TGT_CELL = '[data-col="tgt"]'
const STRIP = '.glossary-quick-add'

/** Hợp âm `Mod+Alt+G` của `glossary.add_term` — `Mod` phân giải thành `Meta` trên macOS. */
async function dispatchAddTerm() {
  await browser.keys(['Meta', 'Alt', 'g'])
}

/**
 * Bấm CHUỘT THẬT vào một ô CHƯA DỊCH rồi gõ — khuôn chép từ `grid-empty-cell.e2e.mjs`
 * (Story 2.5b, đã đo việc này trên webview thật).
 *
 * 🔴 Kiểm caret TRƯỚC khi gõ, và ném với một câu quy TRÁCH NHIỆM đúng lớp: nếu caret không
 * đặt được, đó là hạ tầng của bàn đo (hoặc một hồi quy của CHÍNH cơ chế đặt caret mà
 * `grid-empty-cell.e2e.mjs` canh) — KHÔNG một hồi quy của dải "Thêm thuật ngữ", vì spec này
 * chưa chạm tới phần việc của Story 3.3 ở dòng này.
 */
async function typeIntoEmptyCell(cellSelector, text) {
  await realClick(await $(cellSelector))
  await browser.pause(200)

  const caret = await browser.execute((sel) => {
    const el = document.querySelector(sel)
    return {
      isActive: document.activeElement === el,
      selectionType: window.getSelection()?.type ?? null,
    }
  }, cellSelector)

  if (!caret.isActive || caret.selectionType !== 'Caret') {
    throw new Error(
      `Bấm vào ô "${cellSelector}" KHÔNG đặt được caret ` +
        `(activeElement khớp ô: ${caret.isActive}, selection.type: ${caret.selectionType}).\n\n` +
        '🔴 Đây là lỗi HẠ TẦNG của bàn đo (đường đặt caret mà `grid-empty-cell.e2e.mjs` canh), ' +
        'KHÔNG một hồi quy của dải "Thêm thuật ngữ" — spec này chưa gõ tới phần việc của Story 3.3.',
    )
  }

  await browser.execute((t) => {
    document.execCommand('insertText', false, t)
  }, text)
  await browser.pause(100)
}



/** Chữ ký gọn của phần tử đang giữ tiêu điểm — đủ để so sánh trước/sau `Esc` (mệnh đề ③). */
async function activeElementSignature() {
  return browser.execute(() => {
    const el = document.activeElement
    if (!(el instanceof HTMLElement)) return 'null'
    return [
      el.tagName.toLowerCase(),
      el.getAttribute('data-col') ?? '',
      el.getAttribute('data-segment-id') ?? '',
    ].join('|')
  })
}

/** Vùng chọn hiện tại, dưới dạng chuỗi — đọc qua `window.getSelection()` THẬT. */
async function currentSelectionString() {
  return browser.execute(() => window.getSelection()?.toString() ?? '')
}

/** Giá trị của Ô NGUỒN trong dải — nó là `.gqa-input` ĐẦU TIÊN theo thứ tự DOM (source, rồi
 * translation, rồi note — thứ tự cố định của `GlossaryQuickAdd.vue`), và nó không mang một
 * `data-` nào để chọn trực tiếp (chỉ `ref="sourceInput"`, không lộ ra ngoài Vue). */
async function quickAddSourceInputValue() {
  return browser.execute(() => {
    const input = document.querySelectorAll(`${'.glossary-quick-add'} .gqa-input`)[0]
    return input instanceof HTMLInputElement ? input.value : null
  })
}

/** Hai hình chữ nhật có giao nhau không — phép thử AABB chuẩn, biên chạm nhau KHÔNG tính là giao. */
function rectsIntersect(a, b) {
  return a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top
}

/**
 * 🔵 **SỬA 2026-08-20 — bôi đen bằng `Range` trong trang, KHÔNG bằng driver pointer.**
 *
 * Lượt đầu của tệp này kéo chuột bằng `browser.action('pointer')` và đo được vùng chọn
 * RỖNG **hai lượt liên tiếp**, kể cả sau khi sửa hộp dòng đầu (`getClientRects()[0]`).
 * ⇒ Giả thuyết "toạ độ lệch" SAI: WebDriver pointer trong WKWebView không sinh ra một text
 * selection. Đó là hành vi của driver, không phải của sản phẩm.
 *
 * Khuôn đúng đã có sẵn ở `segment-backspace-merge.e2e.mjs:86-101`: `realClick()` THẬT trước
 * để tiêu điểm đi đúng đường sản phẩm (`focusin` trước `click` — lý do của lệnh cấm
 * `.click()` trong `e2e/AGENTS.md`), rồi đặt `Range` bằng mã trong trang cho tất định.
 *
 * ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì giấu:** cách này KHÔNG kiểm chặng "chuột kéo ⇒ trình
 * duyệt dựng Selection" — chặng đó là hành vi của WebKit, không phải mã của dự án. Thứ nó
 * VẪN kiểm, và là thứ story 3.3 khẳng định: một `Selection` THẬT của WebKit nằm trong một
 * bề mặt vai `display` thì `currentSelectionTextForGlossaryQuickAdd()` đọc được, còn
 * `currentSelectionText()` thì không.
 */
async function boiDenO(cellSelector) {
  await realClick(await $(cellSelector))
  await browser.pause(250)
  return await browser.execute((sel) => {
    const cell = document.querySelector(sel)
    if (cell === null) return { err: `khong tim thay o "${sel}"` }
    const walker = document.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
    let node = walker.nextNode()
    while (node !== null && node.data.length === 0) node = walker.nextNode()
    if (node === null) return { err: `o "${sel}" khong co text node nao sau khi go` }

    const selection = window.getSelection()
    selection.removeAllRanges()
    const r = document.createRange()
    r.selectNodeContents(node)
    selection.addRange(r)
    return { type: selection.type, rangeCount: selection.rangeCount, text: selection.toString() }
  }, cellSelector)
}

/**
 * 🔴 **Bàn đo hỏng KHÁC một mệnh đề đỏ, và phải đọc ra được sự khác nhau đó.**
 *
 * Luật cổng của kho (`_bmad-output/project-context.md` §Luật của một CỔNG): *"Lỗi hạ tầng
 * KHÔNG phải một phép kiểm đỏ... đừng bao giờ báo một kết quả không có thật."*
 *
 * ⚠️ **Lượt đầu của tệp này vi phạm đúng luật đó theo chiều ngược lại.** Nó in sẵn câu
 * *"không một hồi quy của Story 3.3"* vào thông điệp lỗi — tức bài kiểm tự tuyên án trước
 * khi chạy rằng nếu nó đỏ thì lỗi thuộc về người khác. Sai hai lần: một bài kiểm không
 * biết trước nguyên nhân của lượt đỏ tương lai, và khi tiền đề không dựng được thì mệnh đề
 * sản phẩm **chưa hề được đo** — nó không "đạt", cũng không "hỏng".
 *
 * ⇒ Câu đúng phải nói ra CHÍNH XÁC cái đã biết: bàn đo dừng ở bước dựng, và lượt đỏ này
 * không được đọc thành một phán quyết nào về sản phẩm.
 */
function benchAbort(buocDung, doDuoc, mongDoi) {
  return new Error(
    `[BÀN ĐO HỎNG] dừng ở bước dựng "${buocDung}" — mong đợi ${mongDoi}, đo được ${doDuoc}.\n\n` +
      'Mệnh đề sản phẩm bên dưới CHƯA ĐƯỢC ĐO ở lượt này. Đừng đọc lượt đỏ này thành ' +
      '"sản phẩm đạt" hay "sản phẩm hỏng" — cả hai đều là kết quả không có thật.',
  )
}

describe('Story 3.3 — dải "Thêm thuật ngữ" trên webview thật (ba mệnh đề happy-dom không canh được)', () => {
  it('mệnh đề ① — dải mở ĐẨY `.modeport` lên, không PHỦ lên nó (đo hình học, không ảnh chụp)', async () => {
    await openWorkspaceWithWork('Story 3.3 — hình học không phủ')
    await waitForGridRows(1, { col: 'tgt', what: 'lưới của glossary-quick-add (mệnh đề ①)' })

    const before = await browser.execute(() => {
      const r = document.querySelector('.modeport').getBoundingClientRect()
      return { top: r.top, left: r.left, right: r.right, bottom: r.bottom }
    })

    await dispatchAddTerm()
    const strip = await $(STRIP)
    await strip.waitForDisplayed({ timeout: 10_000 })

    const after = await browser.execute(() => {
      const m = document.querySelector('.modeport').getBoundingClientRect()
      const g = document.querySelector('.glossary-quick-add').getBoundingClientRect()
      return {
        modeport: { top: m.top, left: m.left, right: m.right, bottom: m.bottom },
        gqa: { top: g.top, left: g.left, right: g.right, bottom: g.bottom },
      }
    })

    // ── Không giao nhau — mệnh đề TRUNG TÂM của ①. AABB, biên chạm KHÔNG tính là giao. ──
    expect(rectsIntersect(after.modeport, after.gqa)).toBe(false)

    // ── Và quan hệ trên-dưới ĐÚNG CHIỀU: `.modeport` phải kết thúc TRƯỚC khi dải bắt đầu,
    //    không phải chuyện ngẫu nhiên hai hình chữ nhật không chạm nhau ở đâu đó trên màn hình.
    expect(after.modeport.bottom).toBeLessThanOrEqual(after.gqa.top)

    // ── Đáy vùng workspace dịch LÊN so với lúc dải đóng — vế "ĐẨY", không chỉ "không PHỦ".
    //    Một `.glossary-quick-add` kiểu `position: fixed` (bị cấm bởi spec) cũng có thể
    //    KHÔNG giao với `.modeport` mà không hề đẩy nó lên — hai khẳng định trên một mình
    //    không phân biệt được hai hình dạng đó, nên vế `bottom` này là bắt buộc.
    expect(after.modeport.bottom).toBeLessThan(before.bottom)
    // Đỉnh không đổi (dung sai 1px cho làm tròn subpixel) — thanh tiêu đề phía trên cố định,
    // chứng minh đây là một khối CHÈN VÀO LUỒNG từ dưới, không phải `.modeport` bị co từ trên.
    expect(Math.abs(after.modeport.top - before.top)).toBeLessThan(1)

    await browser.keys(['Escape'])
    await strip.waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('mệnh đề ② — bôi đen ở cột bản dịch của lưới (vai `display`) ⇒ ô nguồn của dải mang đúng cụm đó', async () => {
    const TYPED = 'AuraGlossaryX'
    await openWorkspaceWithWork('Story 3.3 — vùng chọn vai display')
    await waitForGridRows(1, { col: 'tgt', what: 'lưới của glossary-quick-add (mệnh đề ②)' })

    await typeIntoEmptyCell(TGT_CELL, TYPED)

    const daBoiDen = await boiDenO(TGT_CELL)
    if ('err' in daBoiDen) throw benchAbort('dung vung chon', daBoiDen.err, 'mot text node co chu')

    // ── Tiền đề: lượt KÉO CHUỘT phải thật sự chọn đúng cụm ta vừa gõ. Nếu không, lỗi nằm
    //    ở TOẠ ĐỘ KÉO của bàn đo — không phải một hồi quy của dải "Thêm thuật ngữ", vì lệnh
    //    `glossary.add_term` còn chưa được gọi tới ở dòng này.
    const preSelection = await currentSelectionString()
    if (preSelection.trim() !== TYPED) {
      throw benchAbort('kéo chuột bôi đen ô bản dịch', `"${preSelection}"`, `"${TYPED}"`)
    }

    // ── Mệnh đề ② thật sự: gọi lệnh TỪ ĐÂY, đọc vùng chọn qua ĐƯỜNG RIÊNG của FR48 ────────
    // `[data-col="tgt"]` đăng ký vai `'display'` (`GridPanel.vue:416`) — `currentSelectionText()`
    // lọc `role !== 'source'` nên trả rỗng ở đây; đúng khuyết tật mà
    // `currentSelectionTextForGlossaryQuickAdd()` tồn tại để tránh (§Boundaries của spec).
    await dispatchAddTerm()
    const strip = await $(STRIP)
    await strip.waitForDisplayed({ timeout: 10_000 })

    const sourceValue = await quickAddSourceInputValue()
    expect(sourceValue).toBe(TYPED)

    await browser.keys(['Escape'])
    await strip.waitForExist({ reverse: true, timeout: 10_000 })
  })

  it('mệnh đề ③ — Esc trả tiêu điểm về đúng ô cũ, và vùng chọn cũ vẫn còn nguyên', async () => {
    const TYPED = 'AuraFocusY'
    await openWorkspaceWithWork('Story 3.3 — Esc trả tiêu điểm')
    await waitForGridRows(1, { col: 'tgt', what: 'lưới của glossary-quick-add (mệnh đề ③)' })

    await typeIntoEmptyCell(TGT_CELL, TYPED)

    const daBoiDen = await boiDenO(TGT_CELL)
    if ('err' in daBoiDen) throw benchAbort('dung vung chon', daBoiDen.err, 'mot text node co chu')

    const beforeSelection = await currentSelectionString()
    if (beforeSelection.trim() !== TYPED) {
      throw benchAbort('kéo chuột bôi đen ô bản dịch', `"${beforeSelection}"`, `"${TYPED}"`)
    }
    const beforeFocus = await activeElementSignature()

    await dispatchAddTerm()
    const strip = await $(STRIP)
    await strip.waitForDisplayed({ timeout: 10_000 })

    // ── Xác nhận tiêu điểm THẬT SỰ dời vào dải trước khi bấm Esc — nếu không, phép thử dưới
    //    đây không kiểm được gì (không có lượt "dời đi" nào để mà "trả lại").
    await browser.waitUntil(
      async () =>
        browser.execute(() => {
          const el = document.querySelector('.glossary-quick-add')
          return el !== null && el.contains(document.activeElement)
        }),
      {
        timeout: 5_000,
        timeoutMsg: 'tiêu điểm không dời vào dải sau khi mở — mệnh đề ③ không có gì để `Esc` trả lại',
      },
    )

    await browser.keys(['Escape'])
    await strip.waitForExist({ reverse: true, timeout: 10_000 })

    const afterFocus = await activeElementSignature()
    const afterSelection = await currentSelectionString()

    // ── Hai vế của mệnh đề ③, và cả hai đều bắt buộc: một đường lui chỉ trả tiêu điểm mà
    //    KHÔNG trả vùng chọn (hoặc ngược lại) vẫn để người dùng phải bôi đen lại từ đầu.
    expect(afterFocus).toBe(beforeFocus)
    expect(afterSelection.trim()).toBe(TYPED)
  })
})
