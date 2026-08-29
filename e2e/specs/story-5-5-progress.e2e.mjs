/**
 * Bàn đo Story 5.5 — "Tiến độ Tác phẩm" (FR7) trong **WKWebView thật**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * VÌ SAO SPEC NÀY TỒN TẠI
 * ═════════════════════════════════════════════════════════════════════════════════
 * `project_contract.rs`/`lifecycle_contract.rs` gọi hàm THUẦN; `tests/frontend/libraryWorks.test.ts`
 * mount `LibraryMode.vue` trên `happy-dom` với `invoke` GIẢ. Cả hai đường đó chứng minh được
 * PHÉP TÍNH (Rust) và PHÉP RENDER (template) đúng — nhưng không đường nào chứng minh CHỖ NỐI
 * đầu-tới-cuối thật.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * AC5 ĐỌC NGUYÊN VĂN "TRONG WORKSPACE", VÀ ĐÓ LÀ MỆNH ĐỀ SPEC NÀY ĐO — KHÔNG PHẢI MỘT MỆNH
 * ĐỀ GẦN GIỐNG
 * ═════════════════════════════════════════════════════════════════════════════════
 * AC5 của story: *"Given một Chương của Tác phẩm đang mở đổi sang `done`, when người dùng
 * quay về Library, then tiến độ đã tăng — không còn số của lượt hiện trước."* Cơ chế mà câu
 * đó gọi tên là `LibraryMode.vue::onActivated` (chạy MỖI LẦN quay về Library, gọi lại
 * `loadWorks()`) — KHÁC hẳn `applyOpenWorkLifecycleResult()` (chạy NGAY sau một lượt ghi vòng
 * đời, cũng gọi `loadWorks()`, nhưng không cần rời Library một bước nào).
 *
 * 🔵 **SỬA (2026-08-28, vòng rà thứ hai)** — bản đầu của spec này bấm nút "Đặt Chương này là
 * Đã xong" (`[data-lifecycle-action="set_chapter_done"]`) NGAY TRONG khối Library, không hề
 * chuyển chế độ. Thứ ca đó chứng minh được là *"ghi xong thì tính lại ngay"*
 * (`applyOpenWorkLifecycleResult`), KHÔNG phải *"quay về Library thì tải lại"*
 * (`onActivated`) — hai cơ chế khác nhau, và bản đầu đo nhầm cơ chế.
 *
 * ⇒ Ca dưới đây đổi trạng thái Chương BẰNG MỘT LỜI GỌI IPC TRẦN (`set_chapter_status`) trong
 * lúc app đang ở `workspace`, KHÔNG qua `dispatch('lifecycle.set_chapter_done')` của Library —
 * lệnh đó mới là thứ kéo theo `applyOpenWorkLifecycleResult`. Gọi trần xong, chuyển VỀ
 * `library` bằng đúng hợp âm sản phẩm (`Mod+1`) rồi mới đọc tiến độ: nếu `onActivated` không
 * tự tải lại, `works` (state module-level, sống sót qua việc đổi chế độ vì `LibraryMode.vue`
 * nằm trong `<KeepAlive>`) vẫn giữ ảnh chụp CŨ (`0 / 1`) — con số sẽ KHÔNG đổi, và ca này ĐỎ.
 *
 * ⚠️ **NỬA CÒN HỞ, GHI RA THAY VÌ ĐỂ NGƯỜI SAU TƯỞNG ĐÃ PHỦ** — Workspace hôm nay KHÔNG có
 * một nút/bề mặt sản phẩm nào đổi trạng thái Chương (nút đó chỉ tồn tại ở khối "Tác phẩm đang
 * mở" của chính Library). Lời gọi IPC trần ở ca này là một GIÁ ĐỠ đứng thay cho bề mặt đó,
 * không phải bằng chứng bề mặt đó đã tồn tại. **Chủ: Story 5.7** (danh sách Chương và mở
 * Chương vào Workspace — nơi tự nhiên nhất để đặt một điều khiển đổi trạng thái Chương ngay
 * tại Workspace).
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * §Giới hạn — ghi ra thay vì để người sau tưởng đã phủ
 * ═════════════════════════════════════════════════════════════════════════════════
 * 1. Chỉ MỘT Chương trong Tác phẩm của spec này (hôm nay chưa đường sản phẩm nào tạo Chương
 *    thứ hai — Epic 6, FR14). Đo trên Chương THỨ HAI trở lên (đếm > 1) ở lại
 *    `project_contract.rs::rebuild_counts_done_chapters_from_the_resolved_set_ignoring_a_corrupt_row`
 *    (chèn Chương qua SQL trực tiếp).
 * 2. KHÔNG kiểm hình học của thanh tiến độ (bề rộng THẬT tính bằng pixel trên màn hình) —
 *    đó là bàn đo chạy tay, không phải WebdriverIO (`tests/AGENTS.md::Bốn đường nghiệm thu`).
 *    Spec này chỉ đọc `aria-valuenow`/`aria-valuemax` (thuộc tính DOM, không hình học).
 * 3. Spec này ghi vào thư mục gốc Library TẠM do `wdio.conf.mjs` cấp qua
 *    `AURATRANSLATE_E2E_LIBRARY_ROOT` — cùng khuôn `story-5-4-lifecycle.e2e.mjs`.
 *
 * 🔴 Đối chứng bắt buộc (đã chạy thật, xem báo cáo triển khai): gỡ khối `onActivated` gọi
 * `loadWorks()` ở `LibraryMode.vue` ⇒ ca dưới đây phải ĐỎ ở bước chờ `aria-valuenow="1"` sau
 * khi quay về Library — không đỏ ở bước nào khác. Khôi phục lại thì xanh.
 *
 * 🔵 **SỬA (2026-08-28, Story 5.6)** — `screenProbe()` đổi bộ chọn từ
 * `.works-block .works-list .works-row` sang `.works-block .works-grid .work-cell`: Story
 * 5.6 đổi danh sách phẳng của Story 5.4 thành LƯỚI (AC2/AC3/AC4/AC6/AC7). Lượt đỏ do đổi bố
 * cục này KHÔNG phải một hồi quy tiến độ — §Design Notes "Bàn đo sẽ đỏ vì bố cục, không vì
 * hồi quy" của `5-6-luoi-tac-pham-loc-va-sap-xep.md` đã nêu trước đúng lượt đỏ này.
 */

import { realClick } from '../support/pointer.mjs'

// ⚠️ TÊN NGẮN có chủ ý — cùng lý lẽ đo được ở `story-5-4-lifecycle.e2e.mjs:33-36`: gõ qua
// `setValue` trên form THẬT, mỗi ký tự là một lệnh WebDriver đi qua `ensureActiveWindowFocus`.
const WORK_NAME = `l5p${Date.now() % 1_000_000}`

// Móc định danh, không `:nth-of-type` — cùng lý lẽ Story 5.4 (một lượt đổi thứ tự nút không
// được phép làm spec bấm NHẦM nút mà vẫn xanh).
const LIST_WORKS_BTN = '[data-lifecycle-action="list_works"]'

/** Tạo một Tác phẩm qua **ĐÚNG FORM người dùng bấm** — không qua cầu IPC trần. */
async function createWorkThroughForm(name) {
  const form = await $('.import-form')
  await (await form.$('input[type="text"]')).setValue(name)
  // Một ký tự là đủ: story này đo TIẾN ĐỘ, không đo bộ tách câu.
  await (await form.$('textarea')).setValue('x')
  await realClick(await form.$('button'))
}

/**
 * Chụp TOÀN BỘ thứ spec này cần, trong **đúng một** lệnh WebDriver — cùng lý lẽ đo được ở
 * `story-5-4-lifecycle.e2e.mjs::screenProbe` (mỗi lệnh WebDriver ở bộ này đi qua
 * `ensureActiveWindowFocus`, và một chuỗi `$()`/`getText()`/`getAttribute()` rời rạc cho một
 * hàng cụ thể tốn nhiều lệnh mỗi lượt poll — thư mục gốc Library dùng CHUNG cho cả lượt chạy
 * nên danh sách chở Tác phẩm của mọi spec khác).
 */
async function screenProbe(name) {
  return browser.execute((wanted) => {
    const rows = Array.from(document.querySelectorAll('.works-block .works-grid .work-cell'))
    const row = rows.find((candidate) => {
      const nameNode = candidate.querySelector('.work-name')
      return nameNode !== null && (nameNode.textContent || '').trim() === wanted
    })
    if (row === undefined) {
      return { rowFound: false, progressText: '', barPresent: false, ariaValueNow: null, ariaValueMax: null }
    }
    const bar = row.querySelector('.work-progress-track')
    return {
      rowFound: true,
      progressText: (row.querySelector('.work-progress')?.textContent || '').trim(),
      barPresent: bar !== null,
      ariaValueNow: bar === null ? null : bar.getAttribute('aria-valuenow'),
      ariaValueMax: bar === null ? null : bar.getAttribute('aria-valuemax'),
    }
  }, name)
}

/** Bấm "Tải danh sách" rồi chờ tới khi hàng của `name` xuất hiện, trả về ảnh chụp cuối. */
async function loadWorksUntilPresent(name) {
  await realClick(await $(LIST_WORKS_BTN))
  let probe = null
  await browser.waitUntil(
    async () => {
      probe = await screenProbe(name)
      return probe.rowFound
    },
    { timeout: 30_000, timeoutMsg: `hàng của Tác phẩm "${name}" không xuất hiện trong danh sách sau 30 giây` },
  )
  return probe
}

/** `Mod+2` — `mode.workspace`. Chờ lưới đối chiếu nạp ít nhất một hàng, cùng khuôn `support/workspace.mjs`. */
async function switchToWorkspace() {
  await browser.keys(['Meta', '2'])
  let seenRows = null
  await browser.waitUntil(
    async () => {
      seenRows = await browser.execute(() => document.querySelectorAll('[data-col="src"]').length)
      return seenRows > 0
    },
    {
      timeout: 30_000,
      timeoutMsg:
        `vào workspace rồi mà lưới không nạp một hàng nào sau 30 giây (lần đọc cuối: ${seenRows} ô)`,
    },
  )
}

/** `Mod+1` — `mode.library`. Chờ khối "Tác phẩm" thật sự có mặt trở lại trên DOM. */
async function switchToLibrary() {
  await browser.keys(['Meta', '1'])
  await browser.waitUntil(async () => browser.execute(() => document.querySelector('.works-block') !== null), {
    timeout: 30_000,
    timeoutMsg: 'quay về library rồi mà khối "Tác phẩm" không có mặt sau 30 giây',
  })
}

/** Đọc `chapter_id` của Chương đang mở, qua cầu IPC trần — không qua adapter TS nào. */
async function readOpenChapterIdViaIpc() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) return null
    try {
      const chapter = await internals.invoke('read_open_chapter')
      return chapter.chapter_id
    } catch {
      return null
    }
  })
}

/**
 * Đổi trạng thái một Chương qua cầu IPC trần (`set_chapter_status`) — KHÔNG qua
 * `dispatch('lifecycle.set_chapter_done')`. Xem khối §AC5 ở đầu tệp: đây là GIÁ ĐỠ đứng thay
 * cho một điều khiển Workspace chưa tồn tại (chủ Story 5.7), cố ý ĐỘC LẬP với
 * `applyOpenWorkLifecycleResult()` để phép chờ ở `switchToLibrary()` sau đó chỉ có thể xanh
 * nhờ `onActivated` — không nhờ đường tải lại tức thời của Library.
 */
async function setChapterStatusViaIpc(chapterId, status) {
  return browser.execute(
    async (id, st) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) return { ok: false, detail: 'khong co cau IPC' }
      try {
        await internals.invoke('set_chapter_status', { chapterId: id, status: st })
        return { ok: true, detail: '' }
      } catch (err) {
        return { ok: false, detail: String(err && err.code ? err.code : err) }
      }
    },
    chapterId,
    status,
  )
}

describe('Story 5.5 · FR7 — tiến độ Tác phẩm trong cửa sổ thật', () => {
  it('Chương đổi trạng thái TRONG Workspace, quay về Library — tiến độ đã cập nhật (0/1 → 1/1)', async () => {
    await createWorkThroughForm(WORK_NAME)

    // Trước khi đổi trạng thái Chương: đúng `0 / 1`, thanh có mặt, `aria-valuenow="0"`,
    // `aria-valuemax="1"` — §I/O Matrix "Tác phẩm một Chương chưa bắt đầu".
    const before = await loadWorksUntilPresent(WORK_NAME)
    expect(before.progressText).toContain('0')
    expect(before.progressText).toContain('1')
    expect(before.barPresent).toBe(true)
    expect(before.ariaValueNow).toBe('0')
    expect(before.ariaValueMax).toBe('1')

    // Rời Library — đúng động tác AC5 mô tả ("Chương của Tác phẩm ĐANG MỞ đổi sang done"
    // xảy ra trong khi người dùng đang làm việc, tức đang ở Workspace).
    await switchToWorkspace()

    const chapterId = await readOpenChapterIdViaIpc()
    expect(chapterId).not.toBeNull()

    const changed = await setChapterStatusViaIpc(chapterId, 'done')
    expect(changed.ok).toBe(true)

    // Quay về Library — đúng động tác thứ hai AC5 mô tả. `onActivated` của `LibraryMode.vue`
    // phải tự gọi lại `loadWorks()` ở ĐÂY, không cần bấm "Tải danh sách" tường minh.
    await switchToLibrary()

    let after = null
    await browser.waitUntil(
      async () => {
        after = await screenProbe(WORK_NAME)
        return after.rowFound && after.ariaValueNow === '1'
      },
      {
        timeout: 30_000,
        timeoutMsg:
          'tiến độ không tăng lên 1/1 sau khi quay về Library — `onActivated` không tải lại, '
          + 'hoặc chỗ nối đếm tiến độ chưa được canh',
      },
    )
    expect(after.progressText).toContain('1')
    expect(after.progressText).not.toContain('0')
    expect(after.ariaValueNow).toBe('1')
    expect(after.ariaValueMax).toBe('1')
  })
})
