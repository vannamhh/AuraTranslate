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
 *
 * 🔵 **SỬA (2026-08-28, Story 5.6)** — ba móc `.works-block .works-list .works-row` đổi
 * thành `.works-block .works-grid .work-cell`: Story 5.6 đổi danh sách phẳng của Story 5.4
 * thành LƯỚI. `story-5-6-luoi-tac-pham-loc-va-sap-xep.md::Code Map` chỉ nêu đích danh
 * `story-5-5-progress.e2e.mjs` là spec phải cập nhật cùng lượt — spec NÀY cũng neo vào cùng
 * hai lớp CSS đó và bị bỏ sót khỏi danh sách đó; sửa ở đây để §Verification "không spec nào
 * khác chuyển từ xanh sang đỏ" không bị chính lượt đổi bố cục này phá vỡ.
 */

import { realClick } from '../support/pointer.mjs'

// ⚠️ TÊN NGẮN có chủ ý: ca đầu gõ tên này qua `setValue` trên form THẬT, và mỗi ký tự là một
// lệnh WebDriver đi qua `ensureActiveWindowFocus` — một tên 30 ký tự đủ làm ca đó vượt trần
// 120 giây của mocha (đo 2026-08-28). Đủ dài để không đụng tên Tác phẩm của spec khác trong
// cùng thư mục gốc dùng chung.
const WORK_NAME_A = `l4a${Date.now() % 1_000_000}`
const WORK_NAME_B = `e2e-lifecycle-b-${Date.now()}`

// 🔵 SỬA (2026-08-28) — định vị bằng MÓC ĐỊNH DANH, không `:nth-of-type`. Bản trước đếm vị
// trí nút, nên một lượt đổi thứ tự sáu nút trong `LibraryMode.vue` làm spec bấm NHẦM nút và
// vẫn xanh — một lượt đỏ sai nguyên nhân là thứ đắt nhất trong bộ này (`e2e/AGENTS.md`).
const LIST_WORKS_BTN = '[data-lifecycle-action="list_works"]'
const FILTER_NOT_STARTED_BTN = '[data-lifecycle-filter="not_started"]'
const FILTER_DONE_BTN = '[data-lifecycle-filter="done"]'
const CLEAR_FILTER_BTN = '[data-lifecycle-action="clear_filter"]'
const WORKS_ROWS = '.works-block .works-grid .work-cell'

const SET_OVERRIDE_PAUSED_BTN = '[data-lifecycle-action="set_override_paused"]'
const CLEAR_OVERRIDE_BTN = '[data-lifecycle-action="clear_override"]'

/**
 * Tạo một Tác phẩm qua **ĐÚNG FORM người dùng bấm** — không qua cầu IPC trần.
 *
 * 🔴 Ca đầu dùng đường này chứ không dùng [`createWork`], và đó là một quyết định: nó là
 * đường DUY NHẤT canh được bản vá 2026-08-28 ở `LibraryMode.vue` (`watch(createdWork, …)` tải
 * lại khối "Tác phẩm đang mở" sau khi tạo). Qua cầu IPC trần thì `createdWork` không đổi, nên
 * `watch` không bắn, và một lượt gỡ bản vá đó vẫn xanh — tức chỗ nối không có ai canh.
 * Đối chứng để chạy lại: gỡ khối `watch` kia ⇒ ca đầu phải ĐỎ ở `waitEnabled`.
 */
async function createWorkThroughForm(name) {
  const form = await $('.import-form')
  await (await form.$('input[type="text"]')).setValue(name)
  // Một ký tự là đủ: story này đo TRẠNG THÁI, không đo bộ tách câu. Xem ghi chú ở `WORK_NAME_A`.
  await (await form.$('textarea')).setValue('x')
  await realClick(await form.$('button'))
}

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

/**
 * Chụp TOÀN BỘ thứ spec này cần, trong **đúng một** lệnh WebDriver.
 *
 * 🔴 Đây là quyết định về TỐC ĐỘ, và nó có số đo. Mỗi lệnh WebDriver ở bộ này đi qua
 * `ensureActiveWindowFocus` của `@wdio/tauri-service` (một `executeAsync` riêng), nên một
 * `await (await $(sel)).getText()` tốn HAI lệnh, và một `waitUntil` polling nó tốn hai lệnh
 * MỖI LƯỢT. Đo 2026-08-28: ca đầu dựng bằng `$()`/`getText()`/`isEnabled()` chạy ~2 phút và
 * **nằm sát trần 120 giây của mocha** — xanh ở `--mochaOpts.timeout 400000`, đỏ ở mặc định.
 * Một ca đỏ vì sát vạch là một ca không nói được gì về sản phẩm.
 *
 * ⚠️ Vẫn là DOM THẬT của WKWebView thật; chỉ chỗ ĐỌC gom lại, mọi cú bấm vẫn qua `realClick`.
 */
async function screenProbe() {
  return browser.execute(() => {
    const text = (selector) => {
      const node = document.querySelector(selector)
      return node === null ? '' : (node.textContent || '').trim()
    }
    const button = (selector) => {
      const node = document.querySelector(selector)
      return { present: node !== null, enabled: node !== null && node.disabled !== true }
    }
    return {
      openStatus: text('.open-work-block p.status'),
      setOverride: button('[data-lifecycle-action="set_override_paused"]'),
      names: Array.from(document.querySelectorAll('.works-block .works-grid .work-cell .work-name')).map(
        (node) => (node.textContent || '').trim(),
      ),
    }
  })
}

/**
 * Tên của mọi hàng đang hiện trong danh sách "Tác phẩm" — đọc trong MỘT lệnh.
 *
 * 🔴 Quét bằng `browser.execute`, KHÔNG bằng `$$()` rồi lặp. Hai lý do, cả hai đo được
 * 2026-08-28:
 *  ① **Tốc độ.** Mỗi lệnh WebDriver ở bộ này đi qua `ensureActiveWindowFocus` của
 *     `@wdio/tauri-service`, nên một vòng lặp `$$` + `.$('.work-name')` tốn 2 lệnh MỖI HÀNG,
 *     mỗi lượt poll. Thư mục gốc Library dùng CHUNG cho cả lượt chạy (`wdio.conf.mjs`
 *     §onPrepare) nên danh sách chở Tác phẩm của mọi spec khác — 22 hàng ở lượt chạy đầy đủ
 *     đầu tiên. Bản `$$` mất **5 phút 38** cho spec này; bản này mất một lệnh mỗi lượt poll.
 *  ② `await $$()` trên WebdriverIO 9.30.1 trả một đối tượng LẶP ĐƯỢC nhưng `.map()` của nó
 *     KHÔNG trả mảng, nên `Promise.all()` ném `object is not iterable` — đã dính một lần.
 *
 * ⚠️ Vẫn là DOM THẬT của WKWebView thật; chỉ chỗ ĐỌC đổi, cú bấm vẫn qua `realClick`.
 */
async function workRowNames() {
  return (await screenProbe()).names
}

/**
 * Trạng thái hiển thị của hàng mang đúng `name`: có mặt không, có dấu ghi đè không, dấu nói gì.
 *
 * Tìm THEO TÊN, không theo vị trí và không theo số lượng — thư mục gốc dùng chung nên danh
 * sách chở cả Tác phẩm của spec khác.
 */
async function workRowState(name) {
  return browser.execute((wanted) => {
    const rows = Array.from(document.querySelectorAll('.works-block .works-grid .work-cell'))
    const row = rows.find((candidate) => {
      const nameNode = candidate.querySelector('.work-name')
      return nameNode !== null && (nameNode.textContent || '').trim() === wanted
    })
    if (row === undefined) return { found: false, hasMarker: false, markerText: '' }
    const marker = row.querySelector('.override-marker')
    return {
      found: true,
      hasMarker: marker !== null,
      markerText: marker === null ? '' : (marker.textContent || '').trim(),
    }
  }, name)
}

/**
 * Chờ tới khi tên các hàng thoả `check`.
 *
 * 🔴 KHÔNG chờ *"dòng trạng thái khác rỗng"*. Đo 2026-08-28: trước lượt tải đầu, dòng đó ĐÃ
 * mang chữ *"Bấm Tải danh sách để xem các Tác phẩm."*, nên điều kiện ấy đúng NGAY LẬP TỨC sau
 * cú bấm — spec đọc một danh sách chưa render rồi kết luận "không tìm thấy Tác phẩm". Một điều
 * kiện chờ đã đúng sẵn không phải một điều kiện chờ.
 */
async function waitForWorkRows(check, timeoutMsg) {
  await browser.waitUntil(async () => check(await workRowNames()), { timeout: 30_000, timeoutMsg })
}

/** Bấm "Tải danh sách" rồi chờ tới khi `name` có mặt trong danh sách. */
async function loadWorksUntilPresent(name) {
  await realClick(await $(LIST_WORKS_BTN))
  await waitForWorkRows(
    (names) => names.includes(name),
    `hàng của Tác phẩm "${name}" không xuất hiện trong danh sách sau 30 giây`,
  )
}

/** Chờ tới khi một nút hết `disabled` — bấm một nút đang tắt là một cú bấm im lặng không làm gì. */
async function waitOverrideButtonEnabled(timeoutMsg) {
  await browser.waitUntil(async () => (await screenProbe()).setOverride.enabled, { timeout: 30_000, timeoutMsg })
}

describe('Story 5.4 · FR5/FR6 — bốn trạng thái vòng đời trong cửa sổ thật', () => {
  it('ghi đè thủ công hiện dấu phân biệt — cả ở khối "Tác phẩm đang mở" lẫn ở hàng trong danh sách', async () => {
    await createWorkThroughForm(WORK_NAME_A)

    // 🔴 Chờ nút BẬT, không chờ "dòng trạng thái khác rỗng". Ở lượt khởi động chưa Tác phẩm nào
    // mở nên khối này ở trạng thái "chưa tải" — dòng chữ ĐÃ khác rỗng ngay từ đầu, và ba nút
    // vòng đời `disabled`. Chờ đúng thứ quyết định được: nút đã bấm được hay chưa.
    await waitOverrideButtonEnabled(
      'nút "Tạm ngưng Tác phẩm này" vẫn tắt sau 30 giây — khối "Tác phẩm đang mở" chưa tải lại '
        + 'sau khi tạo Tác phẩm (bản vá `watch(createdWork, …)` của `LibraryMode.vue`)',
    )
    const beforeOverride = (await screenProbe()).openStatus
    expect(beforeOverride).not.toContain('ghi đè thủ công')

    await realClick(await $(SET_OVERRIDE_PAUSED_BTN))
    await browser.waitUntil(async () => (await screenProbe()).openStatus.includes('ghi đè thủ công'), {
      timeout: 30_000,
      timeoutMsg: 'dấu ghi đè thủ công không xuất hiện sau 30 giây',
    })
    const afterOverride = (await screenProbe()).openStatus
    expect(afterOverride).toContain('Tạm ngưng')
    expect(afterOverride).toContain('ghi đè thủ công')

    // Hàng tương ứng trong danh sách "Tác phẩm" cũng phải mang dấu — không chỉ khối riêng.
    // Vế này là thứ chứng minh lượt reindex SAU một lượt ghi thật sự chạy: dấu chỉ tới được
    // đây qua `meta.json` → `Indexer::rebuild` → `library_work` → `library_list_works`.
    await loadWorksUntilPresent(WORK_NAME_A)
    const row = await workRowState(WORK_NAME_A)
    expect(row.found).toBe(true)
    expect(row.hasMarker).toBe(true)
    expect(row.markerText).toContain('ghi đè thủ công')

    // Bỏ ghi đè — dấu phải biến mất.
    await realClick(await $(CLEAR_OVERRIDE_BTN))
    await browser.waitUntil(async () => !(await screenProbe()).openStatus.includes('ghi đè thủ công'), {
      timeout: 30_000,
      timeoutMsg: 'dấu ghi đè thủ công không biến mất sau khi bỏ ghi đè',
    })
  })

  it('một bộ lọc trạng thái lọc RIÊNG RẼ, không kéo theo bộ lọc khác', async () => {
    // Tác phẩm B mới, không ghi đè — vẫn giữ nguyên `not_started` (default sau khi tạo).
    await createWork(WORK_NAME_B)
    await loadWorksUntilPresent(WORK_NAME_B)

    // Lọc theo "Chưa bắt đầu" — Tác phẩm B (chưa từng đổi trạng thái) phải CÒN.
    // Bấm một nút lọc tự kéo theo một lượt `loadWorks()`, không cần bấm "Tải danh sách".
    await realClick(await $(FILTER_NOT_STARTED_BTN))
    await waitForWorkRows(
      (names) => names.includes(WORK_NAME_B),
      'lọc theo "Chưa bắt đầu" làm mất một Tác phẩm ĐANG ở not_started',
    )

    // Bỏ bộ lọc "Chưa bắt đầu", bật RIÊNG "Đã xong" — Tác phẩm B (chưa xong) KHÔNG được khớp.
    // Đây là vế "riêng rẽ": tắt một giá trị rồi bật một giá trị khác phải đổi HẲN tập kết quả.
    await realClick(await $(FILTER_NOT_STARTED_BTN))
    await realClick(await $(FILTER_DONE_BTN))
    await waitForWorkRows(
      (names) => !names.includes(WORK_NAME_B),
      'lọc theo "Đã xong" vẫn hiện một Tác phẩm chưa xong sau 30 giây',
    )

    // Và bỏ lọc thì nó quay lại — chứng minh nó biến mất vì BỘ LỌC, không vì rơi khỏi chỉ mục.
    await realClick(await $(CLEAR_FILTER_BTN))
    await waitForWorkRows(
      (names) => names.includes(WORK_NAME_B),
      'bỏ mọi bộ lọc mà Tác phẩm B không quay lại — nó đã rơi khỏi chỉ mục, không phải bị lọc',
    )
  })
})
