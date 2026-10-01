/**
 * Story 11.5 — Disposition 3, L2610 + L3510 (Ice quyết định #64): nhãn trạng thái của một hàng
 * lưới phải hiện `message_key` THẬT của lỗi vừa xảy ra — không một chuỗi cố định
 * (`panel.grid.state_refused` cũ, đã gỡ). Chỉ đo được ở webview thật vì `happy-dom` không tái
 * lập được: các ca vitest (`gridPanelRowErrorPriority.test.ts`) giả cả ba nguồn lỗi ở biên
 * `config/segment.ts`, chưa từng đi qua một lượt `invoke()` thật và một Rust thật đang từ chối.
 *
 * Hai ca dưới đây — MỘT lỗi XÁC NHẬN, MỘT lỗi KHÔI PHỤC — là hai trong ba nguồn mà
 * `GridPanel.vue::rowErrorLabelById` hợp nhất (xác nhận · khôi phục · flush). Lỗi FLUSH
 * (`err.store.write_failed`) đòi một lượt ghi WAL trượt thật — không có đòn bẩy nào phun lỗi
 * đó từ webview thật mà không sửa mã sản phẩm (ổ đĩa chỉ-đọc giả lập nằm ngoài bộ đo này);
 * `tests/frontend/gridPanelRowErrorPriority.test.ts` giữ vế đó.
 *
 * 🔴 Ca KHÔI PHỤC gọi THẲNG `segmentHistoryState.ts::restoreVersion` qua `import()` với một
 * `versionId` KHÔNG THUỘC segment đang mở (cùng khuôn/cùng lý do đã ghi ở
 * `library-mode-switch-focus.e2e.mjs`): không nút bấm nào của lớp phủ lịch sử tạo được một
 * `version_id` sai lệch — mọi hàng nó vẽ ra đều là hàng THẬT của segment đang mở. Gọi thẳng
 * hàm sản phẩm (không viết lại logic của nó) là cách duy nhất đưa một `version_id` sai tới
 * ĐÚNG lệnh Rust mà lớp phủ dùng, và đó chính là mệnh đề `err.segment.not_found` cần đo:
 * Rust từ chối thật, `historyRestoreError` (webview) nhận lỗi thật, `GridPanel.vue` vẽ nó ra.
 */
import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { waitForGridRows } from '../support/gridWait.mjs'

const I18N_MODULE = '/src/i18n/index.ts'
const HISTORY_MODULE = '/src/panels/segmentHistoryState.ts'

async function readSegmentsFromDisk() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('read_open_chapter_segments', {})
  })
}

/** Dịch một `message_key`/`params` qua ĐÚNG `t()` thật của sản phẩm — không chép tay chuỗi
 * `vi.json`, để một lượt đổi câu không làm ca này đỏ SAI lý do. */
async function translate(key, params) {
  return browser.execute(
    async (modulePath, k, p) => {
      const mod = await import(/* @vite-ignore */ modulePath)
      return mod.t(k, p)
    },
    I18N_MODULE,
    key,
    params,
  )
}

/** Nhãn `.col-state .cell-state` ở đúng chỉ số hàng — cùng thứ tự `editorSegments` mà
 * `[data-col="src"]`/`[data-col="tgt"]` đã dùng (không một mối nối `data-segment-id` riêng
 * trên chính ô nhãn — xem `GridPanel.vue`). */
async function stateLabelAt(index) {
  return browser.execute(
    (i) => document.querySelectorAll('.col-state .cell-state')[i]?.textContent?.trim() ?? null,
    index,
  )
}

describe('Story 11.5 · L2610 + L3510 — nhãn hàng hiện đúng message_key thật, không một chuỗi cố định', () => {
  it('lỗi XÁC NHẬN thật (`err.segment.nothing_to_confirm`) ⇒ nhãn hàng hiện đúng câu vi.json', async () => {
    const tag = `${Date.now() % 1_000_000}`
    await openWorkspaceWithWork(`e2e-row-error-confirm-${tag}`, 'Cau chua dich。')
    await waitForGridRows(1)

    // Đặt caret vào ô bản dịch RỖNG — Rust từ chối `confirm_segment` bằng
    // `segment.nothing_to_confirm` khi văn bản trống, đúng luật thường của sản phẩm.
    const before = await readSegmentsFromDisk()
    const segmentId = before.segments[0].id
    await realClick(await $(`[data-col="tgt"][data-segment-id="${segmentId}"]`))
    await browser.pause(150)

    // 🔴 GỌI THẲNG `confirmCurrentSegment()` — ĐO ĐƯỢC, KHÔNG PHẢI LỐI TẮT.
    // `browser.keys(['Meta', 'Enter'])` đã thử trước: caret đặt đúng (`editorCaretSegmentId`
    // đọc lại bằng `import()` xác nhận đúng `segmentId`), nhưng KHÔNG một `keydown` nào tới được
    // `CommandRegistry` — `editorConfirmError` ở lại `null` sau 500 ms. Cùng lớp giới hạn bộ đo
    // đã đo ở `segment-merge-split.e2e.mjs` cho `⌘/` (`code: "/"` của `browser.keys()` lệch
    // `code: "Slash"` của bàn phím WebKit thật) — ở đây một `KeyboardEvent` tổng hợp mang
    // `code: 'Enter'` qua `document.dispatchEvent` cũng không tái lập được (phiên WebDriver
    // ném "JavaScript execution returned a result of an unsupported type" ngay sau lượt gọi).
    // Mệnh đề ca này đo là HIỆU ỨNG của một lượt từ chối THẬT trên nhãn hàng, không phải "phím
    // có tới được registry hay không" — gọi thẳng hàm sản phẩm thật (`confirmCurrentSegment`,
    // cùng hàm mà `editor.confirm_segment` gọi) đi trọn phần còn lại của đường thật: adapter →
    // `invoke` → Rust từ chối → `editorConfirmError` → `rowErrorLabelById` → nhãn hàng.
    await browser.execute(async () => {
      const mod = await import(/* @vite-ignore */ '/src/panels/editorPanelState.ts')
      await mod.confirmCurrentSegment()
    })

    const expected = await translate('err.segment.nothing_to_confirm', { segment_id: String(segmentId) })
    await browser.waitUntil(async () => (await stateLabelAt(0)) === expected, {
      timeout: 15_000,
      timeoutMsg:
        `Nhãn hàng 0 không hiện đúng "${expected}" sau 15 giây.\n` +
        'Ứng viên: `confirmCurrentSegment()` không từ chối như mong đợi · `editorConfirmError` không ' +
        'được set · `rowErrorLabelById` không đọc đúng `errorSegmentId`.',
    })

    expect(await stateLabelAt(0)).toBe(expected)
  })

  it('lỗi KHÔI PHỤC thật (`err.segment.not_found`) ⇒ nhãn hàng hiện đúng câu vi.json', async () => {
    const tag = `${Date.now() % 1_000_000}`
    await openWorkspaceWithWork(`e2e-row-error-restore-${tag}`, 'Cau se dich。')
    await waitForGridRows(1)

    const before = await readSegmentsFromDisk()
    const segmentId = before.segments[0].id
    const chapterId = before.chapter_id

    // Ký MỘT lần cho thật — segment có bản dịch, có trạng thái `confirmed`, và lịch sử có
    // đúng một phiên bản. Đi qua IPC trần cho lượt CHUẨN BỊ này (cùng khuôn `signWith` của
    // `segment-history-restore.e2e.mjs`) — mệnh đề ca này đo là lượt KHÔI PHỤC, không lượt ký.
    await browser.execute(
      async (cid, sid) => {
        const internals = window.__TAURI_INTERNALS__
        await internals.invoke('save_segment_targets', {
          chapterId: cid,
          edits: [{ id: sid, target_text: 'Ban dich that.' }],
        })
        return internals.invoke('confirm_segment', { segmentId: sid })
      },
      chapterId,
      segmentId,
    )

    // Đặt caret vào ĐÚNG câu này rồi mở lịch sử — `history.open` (`Mod+H`) đọc
    // `editorCaretSegmentId`, đúng đường sản phẩm.
    await realClick(await $(`[data-col="tgt"][data-segment-id="${segmentId}"]`))
    await browser.pause(150)
    await browser.keys(['Meta', 'h'])

    // Khôi phục với một `version_id` KHÔNG THUỘC segment này — gọi THẲNG hàm sản phẩm thật
    // (§khối lý do đầu tệp). `historySegmentId` đã trỏ đúng `segmentId` nhờ lượt `Mod+H` trên.
    await browser.execute(async (modulePath) => {
      const mod = await import(/* @vite-ignore */ modulePath)
      await mod.restoreVersion(999_999_999, false)
    }, HISTORY_MODULE)

    const expected = await translate('err.segment.not_found', { segment_id: String(segmentId) })
    await browser.waitUntil(async () => (await stateLabelAt(0)) === expected, {
      timeout: 15_000,
      timeoutMsg:
        `Nhãn hàng 0 không hiện đúng "${expected}" sau 15 giây.\n` +
        'Ứng viên: `restoreVersion` không chạm đúng segment (`historySegmentId` sai) · ' +
        '`historyRestoreError` không được set · `rowErrorLabelById` không đọc đúng ' +
        '`restoreErrorSegmentId`.',
    })

    expect(await stateLabelAt(0)).toBe(expected)
  })
})
