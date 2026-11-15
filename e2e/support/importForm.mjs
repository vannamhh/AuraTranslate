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

const IMPORT_OPEN_BTN = '[data-library-import-open]'
const IMPORT_PANEL = '.li-panel'
const OVERLAY_SCRIM = '.ip-screen'
const OVERLAY_CONFIRM_BTN = '.ip-screen .ip-act-primary'
const OVERLAY_ERROR = '.ip-screen .ip-error'

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

/** Chờ màn xem trước ĐÓNG (`.ip-screen` biến mất) — đọc lỗi hiện có nếu nó còn mở lúc timeout. */
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
 * Tạo một Tác phẩm qua ĐÚNG ĐƯỜNG người dùng bấm: bấm `Nhập tài liệu` mở lớp phủ nhập → tab Dán → điền form → bấm nút chính → chờ màn
 * xem trước cho phép xác nhận → bấm xác nhận → chờ màn đóng. Xem doc-comment đầu tệp cho lý
 * do đường này (không phải cầu IPC trần).
 *
 * @param {string} name tên Tác phẩm
 * @returns {Promise<void>}
 */
export async function createWorkThroughForm(name) {
  await realClick(await $(IMPORT_OPEN_BTN))
  const form = await $(`${IMPORT_PANEL} .import-form`)
  await form.waitForDisplayed({ timeout: 10_000 })
  await selectImportTab('paste')
  await (await form.$('input[type="text"]')).setValue(name)
  // Một ký tự là đủ — cả hai spec dùng hàm này đo TRẠNG THÁI/TIẾN ĐỘ, không đo bộ tách câu.
  await (await form.$('textarea')).setValue('x')

  await realClick(await $(`${IMPORT_PANEL} [data-import-preview-open]`))

  await waitConfirmEnabled()
  await realClick(await $(OVERLAY_CONFIRM_BTN))
  await waitOverlayClosed()
  await $(IMPORT_PANEL).waitForExist({ reverse: true, timeout: 10_000 })
}

/**
 * Selects a source tab of the open import overlay with a real pointer press and waits for its
 * body. `tab` is `paste`, `files` or `urls`. Returns the overlay panel.
 *
 * @param {'paste' | 'files' | 'urls'} tab
 */
export async function selectImportTab(tab) {
  const button = await $(`${IMPORT_PANEL} [data-import-tab="${tab}"]`)
  await button.waitForDisplayed({ timeout: 10_000 })
  await realClick(button)
  await $(`${IMPORT_PANEL} [data-import-tabpanel="${tab}"]`).waitForDisplayed({ timeout: 10_000 })
  return $(IMPORT_PANEL)
}

/** Opens the import overlay from the Library button on the URL tab; returns the overlay panel. */
export async function openUrlImportTab() {
  await realClick(await $(IMPORT_OPEN_BTN))
  await $(IMPORT_PANEL).waitForDisplayed({ timeout: 10_000 })
  return selectImportTab('urls')
}

/**
 * Feeds the same `aura://file-dropped` event Rust emits on an OS drop, so the webview listener
 * and the classification behind it run exactly as for a real drop.
 *
 * @param {string[]} paths absolute file paths
 */
export async function dropFiles(paths) {
  await browser.execute(async (list) => {
    await window.__TAURI_INTERNALS__.invoke('plugin:event|emit', { event: 'aura://file-dropped', payload: list })
  }, paths)
}
