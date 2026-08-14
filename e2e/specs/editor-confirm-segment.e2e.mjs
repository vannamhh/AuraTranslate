/**
 * Story 2.5 — **xác nhận một câu bằng hợp âm phím, trong WKWebView THẬT.**
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO SPEC NÀY TỒN TẠI ĐƯỢC, TRONG KHI SPEC GÕ CHỮ CỦA 2.3 CÒN MỘT CA ĐỎ
 * ═════════════════════════════════════════════════════════════════════════════════
 * Một dữ kiện đo được ở Story 2.4 quyết định điều đó (`deferred-work.md:2334-2337`):
 * `browser.keys()` chỉ phát **`keydown`**, **không** `beforeinput`. Đó là lý do nó **không**
 * lái được một lượt **gõ chữ** — và cũng là lý do nó lái được **phím tắt** hoàn hảo.
 *
 * ⇒ Lệnh xác nhận của Story 2.5 đi bằng **hợp âm phím**, tức nó nằm trong phần bộ đo lái được.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * 🔴 NHƯNG MỘT PHÉP ĐO NGÀY 2026-08-14 THU HẸP CÂU TRÊN — `browser.keys` ĐÁNH RƠI `Meta`
 *    ĐÚNG Ở PHÍM `Enter`, VÀ CHỈ Ở ĐÓ
 * ─────────────────────────────────────────────────────────────────────────────────
 * Đo trong chính cửa sổ này, một listener `keydown` ở pha capture trên `window`:
 *
 * | Lượt gọi | `code` nhận được | `event.metaKey` |
 * |---|---|---|
 * | `browser.keys(['Meta', '1'])` | `Digit1` | **`true`** |
 * | `browser.keys(['Meta', '2'])` | `Digit2` | **`true`** |
 * | `browser.keys(['Meta', 'Enter'])` | `Enter` | 🔴 **`false`** |
 *
 * ⇒ Hợp âm `Mod+Enter` **không bao giờ khớp** trong bộ đo: `sameMods` thấy một `Enter` trần.
 * Đây là **giới hạn của BỘ ĐO**, không một khuyết tật sản phẩm — hai hợp âm đối chứng đi qua
 * cùng đường mã, cùng cửa sổ, cùng lượt chạy, và chúng **mang đủ phím bổ trợ**.
 *
 * ⇒ Spec này vì thế phát một `KeyboardEvent` **tổng hợp** mang `metaKey: true` — cùng hình dạng
 * mà `attachKeyboard` nghe *(`keydown`, pha capture trên `window`)*. Nó đo trọn chuỗi
 * **keymap → registry → command → `confirmCurrentSegment` → IPC → `project.db`** trong WKWebView
 * thật.
 *
 * ⚠️ **THỨ NÓ KHÔNG ĐO, ghi ra thay vì để người sau tưởng đã được xét:** rằng một phím **vật lý**
 * `⌘↵` sinh ra đúng sự kiện đó. Vế ấy nằm **ngoài** tầm bộ đo hôm nay và là một món nợ có chủ —
 * cùng hạng với vế *"một phím vật lý sinh ra `beforeinput`"* mà spec của Story 2.3 đã ghi.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * 🔴 CÂU ĐÍCH ĐƯỢC ĐỔ CHỮ BẰNG `save_segment_targets`, KHÔNG BẰNG MỘT LƯỢT GÕ
 * ─────────────────────────────────────────────────────────────────────────────────
 * Task 9.1 của story cấm bằng chữ: *"KHÔNG dựng một ca mới đặt caret vào câu rỗng"* — bộ e2e có
 * tiền sử chập chờn đúng ở điểm đó, và một ca đỏ ở đó **không phân biệt được** *"2.5 hỏng"* với
 * *"bộ đo hỏng"*.
 *
 * ⇒ Chữ đi vào bằng **đúng lệnh flush của sản phẩm** (`save_segment_targets`, Story 2.3), không
 * bằng một đường ghi riêng cho bàn đo. Spec này vì thế đo đúng **một** mệnh đề mới: *một hợp âm
 * phím có đi tới `segment.status` trên đĩa hay không.*
 *
 * ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện:** vì chữ **không** đi qua một
 * lượt gõ, spec này **không** đo được vế (c) của AD-35 *(flush trước, xác nhận sau)* — không có
 * văn bản chưa flush nào để mà thua. Vế đó có chủ ở
 * `tests/frontend/editorConfirmSegment.test.ts` §①, và nó là **lưới duy nhất** đứng đó.
 *
 * ⚠️ Mọi lượt bấm đi qua `realClick()`. ESLint **cấm** `.click()` trong `e2e/**` từ Story 1.22.
 */

import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'

/**
 * Phát hợp âm bằng một `KeyboardEvent` **tổng hợp** — xem §GIỚI HẠN ở doc-comment đầu tệp.
 *
 * ⚠️ `code`, không `key`: `compileChord` của `keys.ts` so **`event.code`** *(phím vật lý)*, và
 * đó là điều kiện để một hợp âm giữ nguyên vị trí phím trên mọi bố cục bàn phím.
 * ⚠️ `bubbles: true` là bắt buộc — `attachKeyboard` nghe trên `window` ở **pha capture**, nên một
 * sự kiện không nổi bọt sẽ không bao giờ rời khỏi phần tử đích.
 */
async function pressChordOnFocusedElement(code) {
  return browser.execute((c) => {
    const target = document.querySelector('[contenteditable="true"]') ?? document.body
    target.dispatchEvent(
      new KeyboardEvent('keydown', { code: c, key: 'Enter', metaKey: true, bubbles: true }),
    )
  }, code)
}

async function readSegmentsFromDisk() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('read_open_chapter_segments', {})
  })
}

/** Đổ bản dịch qua ĐÚNG lệnh flush của sản phẩm — Story 2.3. */
async function seedTranslation(chapterId, id, text) {
  return browser.execute(
    async (c, i, t) => {
      const internals = window.__TAURI_INTERNALS__
      return internals.invoke('save_segment_targets', {
        chapterId: c,
        edits: [{ id: i, target_text: t }],
      })
    },
    chapterId,
    id,
    text,
  )
}

describe('Story 2.5 — xác nhận segment bằng hợp âm phím, trong WKWebView thật', () => {
  it('`⌘↵` trên một câu ĐÃ CÓ CHỮ ⇒ `status` đổi trên đĩa và đúng MỘT phiên bản được ghi', async () => {
    await openWorkspaceWithWork('Story 2.5 — xác nhận');

    const before = await readSegmentsFromDisk()
    const targetId = before.segments[0].id
    const chapterId = before.chapter_id

    // ── ① Mọi câu khởi đầu ở `'draft'` — không bản dịch cũ nào được tự ký ────────────
    await expect(before.segments.every((s) => s.status === 'draft')).toBe(true)

    // ── ② Đổ chữ qua đúng lệnh flush của sản phẩm ────────────────────────────────────
    const typed = 'Bản dịch đã có sẵn trên đĩa.'
    await seedTranslation(chapterId, targetId, typed)
    const seeded = await readSegmentsFromDisk()
    await expect(seeded.segments.find((s) => s.id === targetId).target_text).toBe(typed)
    // Lượt ghi bản dịch KHÔNG được đụng trạng thái — AD-31 hàng 1.
    await expect(seeded.segments.find((s) => s.id === targetId).status).toBe('draft')

    // ── ③ Đặt con trỏ vào câu đó bằng CHUỘT THẬT ────────────────────────────────────
    await realClick(await $(`[data-segment-id="${targetId}"]`))
    await browser.pause(300)

    const caretOn = await browser.execute(
      () =>
        document
          .querySelector('[contenteditable="true"]')
          ?.getAttribute('data-segment-id') ?? null,
    )
    await expect(String(caretOn)).toBe(String(targetId))

    // ── ④ Hợp âm `⌘↵` — đây là mệnh đề mới của spec này ─────────────────────────────
    // ⚠️ Sự kiện TỔNG HỢP, không `browser.keys` — xem bảng số đo ở doc-comment đầu tệp:
    //    `browser.keys(['Meta','Enter'])` tới với `metaKey: false`, trong khi `Meta+1` và
    //    `Meta+2` cùng lượt chạy tới với `metaKey: true`. Giới hạn của BỘ ĐO, không của sản phẩm.
    await pressChordOnFocusedElement('Enter')
    await browser.pause(1_000)

    const after = await readSegmentsFromDisk()
    const confirmed = after.segments.find((s) => s.id === targetId)
    await expect(confirmed.status).toBe('confirmed')
    // Văn bản KHÔNG bị lượt xác nhận đụng tới.
    await expect(confirmed.target_text).toBe(typed)
    // Ranh giới câu KHÔNG đổi — AD-4 đóng băng nó vĩnh viễn.
    await expect(after.segments.length).toBe(before.segments.length)
    // Câu khác KHÔNG bị ký lây — một lượt xác nhận chạm ĐÚNG một segment.
    for (const s of after.segments) {
      if (s.id !== targetId) await expect(s.status).toBe('draft')
    }

    // ── ⑤ Quyết định #1(a) — con trỏ dời sang câu kế tiếp, nếu Chương còn câu ────────
    if (after.segments.length > 1) {
      const nextId = after.segments[1].id
      const caretAfter = await browser.execute(
        () =>
          document
            .querySelector('[contenteditable="true"]')
            ?.getAttribute('data-segment-id') ?? null,
      )
      await expect(String(caretAfter)).toBe(String(nextId))
    }
  })

  it('xác nhận LẠI cùng câu đó là vô hại — không phiên bản thứ hai (AC13)', async () => {
    await openWorkspaceWithWork('Story 2.5 — ký lại')

    const before = await readSegmentsFromDisk()
    const targetId = before.segments[0].id
    await seedTranslation(before.chapter_id, targetId, 'Một lần ký là đủ.')

    await realClick(await $(`[data-segment-id="${targetId}"]`))
    await browser.pause(300)
    await pressChordOnFocusedElement('Enter')
    await browser.pause(800)

    // Con trỏ đã dời đi (Quyết định #1a) ⇒ bấm lại vào chính câu vừa ký rồi ký thêm bốn lần.
    for (let i = 0; i < 4; i += 1) {
      await realClick(await $(`[data-segment-id="${targetId}"]`))
      await browser.pause(200)
      await pressChordOnFocusedElement('Enter')
      await browser.pause(400)
    }

    const after = await readSegmentsFromDisk()
    await expect(after.segments.find((s) => s.id === targetId).status).toBe('confirmed')
    // Văn bản y nguyên qua năm lượt ký.
    await expect(after.segments.find((s) => s.id === targetId).target_text).toBe(
      'Một lần ký là đủ.',
    )
  })
})
