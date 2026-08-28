/**
 * Bàn đo Story 5.4 — "Bốn trạng thái vòng đời" (FR5/FR6) trong **WKWebView thật**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY TỒN TẠI
 * ═════════════════════════════════════════════════════════════════════════════════
 * `lifecycle_contract.rs` gọi hàm THUẦN; `tests/frontend/libraryWorks.test.ts` chạy trên
 * `happy-dom` với `invoke` GIẢ. Năm vỏ IPC mới (`library_list_works` · `read_work_lifecycle` ·
 * `set_chapter_status` · `set_work_status_override`, cộng chín id lệnh mới của
 * `CommandRegistry`) cho tới trước spec này chưa bị bất kỳ ca nào chạm THẬT: đăng ký thiếu ở
 * `generate_handler!`, tên tham số lệch `camelCase`, hay một `dispatch('lifecycle.*')` chưa
 * nối port đều đi qua **toàn bộ** cổng hiện có mà không ai đỏ. Spec này đi trọn đường: nút
 * thật → `dispatch` thật → registry thật → `invoke` thật → `commands::lifecycle`/
 * `commands::library` thật → DOM thật, cho ĐÚNG hai kịch bản tối thiểu của §Verification:
 * *"ghi đè thủ công hiện dấu phân biệt"* và *"một bộ lọc lọc riêng rẽ"*.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * §Giới hạn — ghi ra thay vì để người sau tưởng đã phủ
 * ═════════════════════════════════════════════════════════════════════════════════
 * 1. Chỉ MỘT Tác phẩm mở được tại một thời điểm (`OpenWorkState`) — spec này tạo Tác phẩm
 *    THỨ HAI để đổi trạng thái Chương của nó, làm Tác phẩm ĐẦU không còn "đang mở". Cả hai
 *    kịch bản dưới đây chỉ cần TRẠNG THÁI đã ghi (đọc qua danh sách Library), không cần giữ
 *    Tác phẩm đầu ở trạng thái "đang mở" xuyên suốt.
 * 2. Không đo trạng thái *hỗn hợp* (`in_progress` từ nhiều Chương) — cần Chương thứ hai
 *    trong CÙNG một Tác phẩm, thứ chưa có đường sản phẩm nào tạo (Epic 6). Ca đó ở lại
 *    `lifecycle_contract.rs` (chèn Chương qua SQL trực tiếp).
 * 3. Spec này ghi vào thư mục gốc Library TẠM do `wdio.conf.mjs` cấp qua
 *    `AURATRANSLATE_E2E_LIBRARY_ROOT` — cùng khuôn `story-5-3-rescan.e2e.mjs`.
 */

import { realClick } from '../support/pointer.mjs'

const WORK_NAME_A = `e2e-lifecycle-a-${Date.now()}`
const WORK_NAME_B = `e2e-lifecycle-b-${Date.now()}`

const LIST_WORKS_BTN = '.works-block .filter-actions .btn:nth-of-type(6)'
const FILTER_NOT_STARTED_BTN = '.works-block .filter-actions .btn:nth-of-type(1)'
const FILTER_DONE_BTN = '.works-block .filter-actions .btn:nth-of-type(4)'
const CLEAR_FILTER_BTN = '.works-block .filter-actions .btn:nth-of-type(5)'
const WORKS_STATUS_LINE = '.works-block > p.status'
const WORKS_ROWS = '.works-block .works-list .works-row'

const OPEN_WORK_STATUS_LINE = '.open-work-block p.status'
const SET_OVERRIDE_PAUSED_BTN = '.open-work-block .open-work-actions .btn:nth-of-type(1)'
const CLEAR_OVERRIDE_BTN = '.open-work-block .open-work-actions .btn:nth-of-type(2)'

/** Tạo một Tác phẩm qua cầu IPC thật, cùng khuôn `story-5-3-rescan.e2e.mjs`. */
async function createWork(name) {
  const created = await browser.execute(async (workName) => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) return { ok: false, detail: 'không có cầu IPC' }
    try {
      await internals.invoke('create_work_from_text', {
        name: workName,
        sourceLang: 'zh',
        genre: 'general',
        text: 'Một câu nguồn để bộ nhập có việc mà làm.',
      })
      return { ok: true, detail: '' }
    } catch (err) {
      return { ok: false, detail: String(err && err.code ? err.code : err) }
    }
  }, name)
  expect(created.ok).toBe(true)
}

/** Bấm "Tải danh sách" rồi chờ danh sách phản ánh xong (dòng trạng thái đổi khỏi rỗng). */
async function loadWorksAndWait() {
  await realClick(await $(LIST_WORKS_BTN))
  await browser.waitUntil(
    async () => {
      const text = await (await $(WORKS_STATUS_LINE)).getText()
      return text.trim() !== ''
    },
    { timeout: 20_000, timeoutMsg: 'dòng trạng thái "Tác phẩm" không cập nhật sau 20 giây' },
  )
}

describe('Story 5.4 · FR5/FR6 — bốn trạng thái vòng đời trong cửa sổ thật', () => {
  it('ghi đè thủ công hiện dấu phân biệt — cả ở khối "Tác phẩm đang mở" lẫn ở hàng trong danh sách', async () => {
    await createWork(WORK_NAME_A)

    // Trước khi ghi đè: dòng trạng thái Tác phẩm đang mở KHÔNG mang dấu ghi đè.
    await browser.waitUntil(
      async () => (await (await $(OPEN_WORK_STATUS_LINE)).getText()).trim() !== '',
      { timeout: 20_000, timeoutMsg: 'dòng trạng thái Tác phẩm đang mở không tải được sau 20 giây' },
    )
    const beforeOverride = await (await $(OPEN_WORK_STATUS_LINE)).getText()
    expect(beforeOverride).not.toContain('ghi đè thủ công')

    await realClick(await $(SET_OVERRIDE_PAUSED_BTN))
    await browser.waitUntil(
      async () => (await (await $(OPEN_WORK_STATUS_LINE)).getText()).includes('ghi đè thủ công'),
      { timeout: 20_000, timeoutMsg: 'dấu ghi đè thủ công không xuất hiện sau 20 giây' },
    )
    const afterOverride = await (await $(OPEN_WORK_STATUS_LINE)).getText()
    expect(afterOverride).toContain('Tạm ngưng')
    expect(afterOverride).toContain('ghi đè thủ công')

    // Hàng tương ứng trong danh sách "Tác phẩm" cũng phải mang dấu — không chỉ khối riêng.
    await loadWorksAndWait()
    const rows = await $$(WORKS_ROWS)
    let found = false
    for (const row of rows) {
      const name = await (await row.$('.work-name')).getText()
      if (name === WORK_NAME_A) {
        found = true
        const marker = await row.$('.override-marker')
        expect(await marker.isExisting()).toBe(true)
        expect(await marker.getText()).toContain('ghi đè thủ công')
      }
    }
    expect(found).toBe(true)

    // Bỏ ghi đè — dấu phải biến mất.
    await realClick(await $(CLEAR_OVERRIDE_BTN))
    await browser.waitUntil(
      async () => !(await (await $(OPEN_WORK_STATUS_LINE)).getText()).includes('ghi đè thủ công'),
      { timeout: 20_000, timeoutMsg: 'dấu ghi đè thủ công không biến mất sau khi bỏ ghi đè' },
    )
  })

  it('một bộ lọc trạng thái lọc RIÊNG RẼ, không kéo theo bộ lọc khác', async () => {
    // Tác phẩm B mới, không ghi đè — vẫn giữ nguyên `not_started` (default sau khi tạo).
    await createWork(WORK_NAME_B)
    await loadWorksAndWait()

    // Đảm bảo không bộ lọc nào đang bật trước khi bắt đầu ca này.
    await realClick(await $(CLEAR_FILTER_BTN))
    await browser.waitUntil(
      async () => (await (await $(WORKS_STATUS_LINE)).getText()).trim() !== '',
      { timeout: 20_000 },
    )

    const beforeFilter = await (await $(WORKS_STATUS_LINE)).getText()

    // Lọc theo "Chưa bắt đầu" — Tác phẩm B (chưa từng đổi trạng thái) phải khớp.
    await realClick(await $(FILTER_NOT_STARTED_BTN))
    await browser.waitUntil(
      async () => (await (await $(WORKS_STATUS_LINE)).getText()) !== beforeFilter,
      { timeout: 20_000, timeoutMsg: 'dòng trạng thái không đổi sau khi bật bộ lọc' },
    )
    const rowsNotStarted = await $$(WORKS_ROWS)
    const namesNotStarted = await Promise.all(
      rowsNotStarted.map(async (row) => (await row.$('.work-name')).getText()),
    )
    expect(namesNotStarted).toContain(WORK_NAME_B)

    // Bỏ bộ lọc "Chưa bắt đầu", bật RIÊNG "Đã xong" — Tác phẩm B (chưa xong) KHÔNG được khớp.
    await realClick(await $(FILTER_NOT_STARTED_BTN))
    await realClick(await $(FILTER_DONE_BTN))
    await browser.waitUntil(
      async () => {
        const rows = await $$(WORKS_ROWS)
        const names = await Promise.all(rows.map(async (row) => (await row.$('.work-name')).getText()))
        return !names.includes(WORK_NAME_B)
      },
      { timeout: 20_000, timeoutMsg: 'lọc theo "Đã xong" vẫn hiện một Tác phẩm chưa xong sau 20 giây' },
    )

    // Dọn: bỏ mọi bộ lọc để không ảnh hưởng ca chạy sau trong CÙNG một phiên webview.
    await realClick(await $(CLEAR_FILTER_BTN))
  })
})
