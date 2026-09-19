/**
 * Bàn đo dùng CHUNG — tạo một Tác phẩm qua ĐÚNG ĐƯỜNG người dùng bấm, xuyên màn xem trước
 * bảng mã (Story 6.3). Dùng bởi `story-5-4-lifecycle.e2e.mjs` và `story-5-5-progress.e2e.mjs`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO HÀM NÀY TỒN TẠI — G2 (`deferred-work.md §*Deferred from: 6-13-alt-text-va-caption-la-hai-segment-mang-truong-vai (2026-09-09)*`)
 * ═════════════════════════════════════════════════════════════════════════════════
 * Trước Story 6.3 (`d20fe67`), bấm nút nộp dán-văn-bản GHI Tác phẩm ngay. Từ Story 6.3,
 * `submitPastedText` (`src/modes/libraryImport.ts:332-358`) chỉ MỞ `ImportPreviewOverlay.vue`;
 * Tác phẩm chỉ được ghi khi người dùng bấm "Xác nhận" trong đó
 * (`.ip-act-primary` → `dispatch('import.preview.confirm')` → `confirmImportPreview()` →
 * `finishImportSubmission()`, `src/main.ts:465-472`). Hai spec trên mỗi tệp từng giữ MỘT bản
 * sao của một hàm bấm `form.$('button')` KHÔNG có tên — sau Story 6.3 đó là nút MỞ màn xem
 * trước, không phải nút ghi — nên cả ba ca đỏ với "hàng ... không xuất hiện sau 30 giây". Hàm
 * này đi trọn đường thật: mở → chờ nút xác nhận BẬT → bấm xác nhận → chờ màn đóng — MỘT chỗ
 * để sửa khi màn xem trước đổi lần sau.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO KHÔNG QUA CẦU IPC TRẦN (`create_work_from_text`)
 * ═════════════════════════════════════════════════════════════════════════════════
 * `e2e/support/workspace.mjs:19-26` đã ghi lập trường này cho fixture của nó: đi qua cầu IPC
 * trần đo một thứ KHÁC hẳn — đường ghi của Rust, không đường nhập của người dùng. Hai spec
 * dùng hàm NÀY đo đúng ĐƯỜNG NHẬP: form → màn xem trước → xác nhận — đúng bước Story 6.3
 * chèn thêm giữa cú bấm và lượt ghi. Qua IPC trần là bỏ qua chính bước đó, cho một bộ xanh mà
 * không spec nào chạm `ImportPreviewOverlay.vue` — trái với vai trò DUY NHẤT của thư mục này
 * (`e2e/AGENTS.md`: hành vi trong WKWebView THẬT).
 */

import { realClick } from './pointer.mjs'

const OVERLAY_SCRIM = '.ip-scrim'
const OVERLAY_CONFIRM_BTN = '.ip-scrim .ip-act-primary'
const OVERLAY_ERROR = '.ip-scrim .ip-error'

/**
 * Tìm nút nộp dán-văn-bản bằng CẤU TRÚC DOM, không bằng việc tin `form.$('[data-import-preview-open]')`
 * luôn trúng đúng nút. Ba nút — dán văn bản, tệp, URL — CÙNG mang thuộc tính
 * `[data-import-preview-open]` (`src/modes/LibraryMode.vue:1250,1279,1325`), không thuộc tính
 * nào phân biệt được chúng với nhau; "lấy nút khớp ĐẦU TIÊN" chỉ đúng vì thứ tự HIỆN TẠI
 * trong form, và một lượt thêm/đổi thứ tự nhánh nhập sẽ làm giả định đó sai một cách im lặng.
 *
 * Neo thay vào đó: textarea dán văn bản (`<textarea>` ĐẦU TIÊN trong form — `field_paste`,
 * `LibraryMode.vue:1245`) rồi đi tới nút `[data-import-preview-open]` đứng NGAY SAU label bọc
 * nó (`:1247-1255`). Nếu nút neo được không tồn tại, hoặc không phải là
 * `[data-import-preview-open]` ĐẦU TIÊN trong form, bố cục đã trôi khỏi giả định này — ném
 * lỗi nêu tên chỗ trôi, không bấm liều.
 *
 * ⚠️ Đi qua `browser.execute` với CHỈ chuỗi CSS (trả về CHỈ SỐ trong danh sách nút), không
 * qua XPath tương đối `form.$('.//…')`: giao thức WebDriver classic (bộ lái WebKit của thư
 * mục này) chạy XPath tương đối bằng `document.evaluate(value, contextNode)` — đúng cho
 * `.` — nhưng đo được 2026-09-14 nó KHÔNG neo đúng contextNode qua endpoint "find element
 * from element" của bộ lái này, nên một biểu thức tưởng ĐÃ neo vào `form` vẫn lệch. Duyệt
 * DOM trong `execute()` (chạy thẳng trong webview, không qua endpoint đó) không dính lỗi này.
 */
async function findPasteTextSubmit(form) {
  const result = await browser.execute((formSel) => {
    const formEl = document.querySelector(formSel)
    if (formEl === null) return { ok: false, reason: 'form_missing' }
    const buttons = Array.from(formEl.querySelectorAll('[data-import-preview-open]'))
    const pasteTextarea = formEl.querySelector('textarea')
    if (pasteTextarea === null) return { ok: false, reason: 'no_textarea' }
    const label = pasteTextarea.closest('label')
    if (label === null) return { ok: false, reason: 'no_label' }
    let anchored = null
    for (let node = label.nextElementSibling; node !== null; node = node.nextElementSibling) {
      if (node.matches('[data-import-preview-open]')) {
        anchored = node
        break
      }
    }
    if (anchored === null) return { ok: false, reason: 'no_anchored_button' }
    const anchoredIndex = buttons.indexOf(anchored)
    if (anchoredIndex !== 0) return { ok: false, reason: 'not_first', anchoredIndex, buttonCount: buttons.length }
    return { ok: true, index: anchoredIndex }
  }, '.import-form')

  if (!result.ok) {
    throw new Error(
      `createWorkThroughForm: lệch bố cục ở nút nộp dán-văn-bản (mã: ${result.reason}, ` +
        `${JSON.stringify(result)}) — mong nó là nút \`[data-import-preview-open]\` ĐẦU TIÊN ` +
        'trong form, đứng ngay sau label bọc textarea dán văn bản. Không bấm gì.',
    )
  }

  const buttons = await form.$$('[data-import-preview-open]')
  return buttons[result.index]
}

/**
 * Chờ nút xác nhận của màn xem trước BẬT. Đọc trạng thái LẦN CUỐI trong biến ngoài vòng
 * `waitUntil` rồi chỉ DỰNG thông báo lỗi ở nhánh `catch` — cùng luật `support/workspace.mjs`
 * ("số trong câu báo lỗi đọc SAU vòng chờ, không nội suy vào tham số lúc tạo object").
 */
async function waitConfirmEnabled() {
  let last = null
  try {
    await browser.waitUntil(async () => {
      last = await browser.execute(
        (scrimSel, btnSel, errSel) => {
          const scrim = document.querySelector(scrimSel)
          if (scrim === null) return { scrimPresent: false, confirmEnabled: false, errorText: null }
          const btn = document.querySelector(btnSel)
          const err = document.querySelector(errSel)
          return {
            scrimPresent: true,
            confirmEnabled: btn !== null && btn.disabled !== true,
            errorText: err === null ? null : (err.textContent || '').trim(),
          }
        },
        OVERLAY_SCRIM,
        OVERLAY_CONFIRM_BTN,
        OVERLAY_ERROR,
      )
      return last.scrimPresent && last.confirmEnabled
    }, { timeout: 30_000 })
  } catch (err) {
    // ⚠️ `catch (err)`, không `catch` trơ — `browser.waitUntil` cũng ném khi CHÍNH điều kiện
    // (hàm truyền vào) ném lỗi giữa chừng ("waitUntil condition failed with the following
    // reason: …", `webdriverio`), không chỉ khi hết 30 giây. Bắt trơ sẽ nói "sau 30 giây" cả
    // khi lỗi thật đến sớm hơn nhiều và vì lý do khác hẳn — `err.message` phải đứng CẠNH lần
    // đọc cuối, không bị nuốt.
    throw new Error(
      'createWorkThroughForm: nút xác nhận màn xem trước vẫn TẮT (hoặc màn chưa mở) sau 30 ' +
        `giây — lần đọc cuối: ${JSON.stringify(last)} — lỗi chờ: ${err instanceof Error ? err.message : String(err)}`,
    )
  }
}

/** Chờ màn xem trước ĐÓNG (`.ip-scrim` biến mất) — đọc lỗi hiện có nếu nó còn mở lúc timeout. */
async function waitOverlayClosed() {
  let last = null
  try {
    await browser.waitUntil(async () => {
      last = await browser.execute(
        (scrimSel, errSel) => {
          const scrim = document.querySelector(scrimSel)
          if (scrim === null) return { scrimPresent: false, errorText: null }
          const err = document.querySelector(errSel)
          return { scrimPresent: true, errorText: err === null ? null : (err.textContent || '').trim() }
        },
        OVERLAY_SCRIM,
        OVERLAY_ERROR,
      )
      return !last.scrimPresent
    }, { timeout: 30_000 })
  } catch (err) {
    // Cùng lý do `catch (err)` ở `waitConfirmEnabled` ngay trên.
    throw new Error(
      'createWorkThroughForm: màn xem trước không đóng sau 30 giây sau khi bấm xác nhận — lần ' +
        `đọc cuối: ${JSON.stringify(last)} — lỗi chờ: ${err instanceof Error ? err.message : String(err)}`,
    )
  }
}

/**
 * Tạo một Tác phẩm qua ĐÚNG ĐƯỜNG người dùng bấm: điền form → bấm nộp dán-văn-bản → chờ màn
 * xem trước cho phép xác nhận → bấm xác nhận → chờ màn đóng. Xem doc-comment đầu tệp cho lý
 * do đường này (không phải cầu IPC trần) và cách neo nút (không phải vị trí nút).
 *
 * @param {string} name tên Tác phẩm
 * @returns {Promise<void>}
 */
export async function createWorkThroughForm(name) {
  const form = await $('.import-form')
  await (await form.$('input[type="text"]')).setValue(name)
  // Một ký tự là đủ — cả hai spec dùng hàm này đo TRẠNG THÁI/TIẾN ĐỘ, không đo bộ tách câu.
  await (await form.$('textarea')).setValue('x')

  const submit = await findPasteTextSubmit(form)
  await realClick(submit)

  await waitConfirmEnabled()
  await realClick(await $(OVERLAY_CONFIRM_BTN))
  await waitOverlayClosed()
}
