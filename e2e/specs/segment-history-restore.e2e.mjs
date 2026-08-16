/**
 * Story 2.6 — **vòng trọn của lịch sử phiên bản, trong WKWebView THẬT.**
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO SPEC NÀY LÀ ĐƯỜNG DUY NHẤT ĐO ĐƯỢC MỆNH ĐỀ CỦA NÓ
 * ═════════════════════════════════════════════════════════════════════════════════
 * Story 2.6 dựng một **struct mới hoàn toàn** đi qua dây (`SegmentVersionRow`), tức đúng lớp
 * rủi ro mà Epic 2 đã trả giá một lần: bản đầu Story 2.5 thêm `status` vào kiểu TypeScript
 * nhưng **quên** thêm vào struct Rust và vào câu `SELECT`. Kết quả: `undefined` phía webview,
 * `isConfirmed` **luôn `false` trên sản phẩm thật** — và **74/74** test frontend vẫn xanh, vì
 * fixture vitest dựng struct **bằng tay** và có sẵn cột. **Chỉ e2e bắt được.**
 *
 * ⇒ Spec này đi trọn chuỗi **IPC → `project.db` → IPC** trên webview thật:
 * ký → sửa → ký lại → đọc lịch sử → khôi phục → đọc lại từ đĩa.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO NÓ ĐI BẰNG `invoke` CHỨ KHÔNG BẰNG CHUỘT VÀ PHÍM
 * ─────────────────────────────────────────────────────────────────────────────────
 * Mệnh đề spec này đo là **hợp đồng dây**, không phải một tương tác. Lái nó bằng chuột kéo
 * theo ba nguồn chập chờn đã đo của bộ e2e *(`browser.keys` đánh rơi `Meta` ở `Enter`;
 * fixture không reset state panel giữa các spec; `devServerIsUp()` tin một Vite hấp hối)* —
 * và một ca đỏ vì một trong ba thứ đó **không phân biệt được** với *"Story 2.6 hỏng"*.
 *
 * ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tưởng đã được xét:** spec này **không** đo
 * lớp phủ — không vế thị giác, không bẫy `Tab`, không `Escape`, không hợp âm `⌘H`. Vế hành vi
 * của lớp phủ có chủ ở `tests/frontend/segmentHistory.test.ts`; vế **hình học** thì thuộc bàn
 * đo, và `happy-dom` **không phải** WebKit.
 *
 * ⚠️ Mọi lượt bấm, nếu có, đi qua `realClick()`. ESLint **cấm** `.click()` trong `e2e/**`.
 */

import { openWorkspaceWithWork } from '../support/workspace.mjs'

async function readSegmentsFromDisk() {
  return browser.execute(async () => {
    const internals = window.__TAURI_INTERNALS__
    if (internals === undefined) throw new Error('không có cầu IPC trong webview')
    return internals.invoke('read_open_chapter_segments', {})
  })
}

/**
 * Ký một câu, đi qua **đúng ba lệnh của sản phẩm và đúng thứ tự đó**.
 *
 * 🔴 `unconfirm_edited_segments` **KHÔNG** phải một lệnh trên dây — nó là một hàm thuần mà vỏ
 * `wire::save_segment_targets` gọi **trước** lượt ghi. Nên ở đây một lượt `save_segment_targets`
 * là đủ: nó tự hạ `'confirmed'` → `'draft'` khi văn bản thật sự đổi (AD-31 hàng 3).
 * ⚠️ Thứ tự hạ-trước-ghi-sau là một quyết định về **mất mát dữ liệu**, không một chi tiết:
 * ghi-rồi-hạ mà sập ở giữa để lại văn bản đã đổi trên một segment vẫn `'confirmed'` ⇒ không
 * lần xác nhận nào nữa xảy ra ⇒ cặp TM mới không bao giờ được ghi, im lặng vĩnh viễn.
 */
async function signWith(chapterId, id, text) {
  return browser.execute(
    async (c, i, t) => {
      const internals = window.__TAURI_INTERNALS__
      await internals.invoke('save_segment_targets', {
        chapterId: c,
        edits: [{ id: i, target_text: t }],
      })
      return internals.invoke('confirm_segment', { segmentId: i })
    },
    chapterId,
    id,
    text,
  )
}

async function readHistory(id) {
  return browser.execute(async (i) => {
    const internals = window.__TAURI_INTERNALS__
    return internals.invoke('read_segment_history', { segmentId: i })
  }, id)
}

async function restore(id, versionId, force) {
  return browser.execute(
    async (i, v, f) => {
      const internals = window.__TAURI_INTERNALS__
      return internals.invoke('restore_segment_version', {
        segmentId: i,
        versionId: v,
        force: f,
      })
    },
    id,
    versionId,
    force,
  )
}

describe('Story 2.6 — lịch sử phiên bản và khôi phục, trong WKWebView thật', () => {
  it('ký → sửa → ký lại → đọc lịch sử → khôi phục → đọc lại từ đĩa', async () => {
    await openWorkspaceWithWork('Story 2.6 — lịch sử phiên bản')

    const before = await readSegmentsFromDisk()
    const targetId = before.segments[0].id
    const chapterId = before.chapter_id

    // ── ① Câu chưa từng ký ⇒ lịch sử RỖNG, không lỗi. AC3 vế dữ liệu ────────────────
    const empty = await readHistory(targetId)
    await expect(Array.isArray(empty)).toBe(true)
    await expect(empty.length).toBe(0)

    // ── ② Ba lượt ký trên ba văn bản khác nhau ──────────────────────────────────────
    await signWith(chapterId, targetId, 'Bản một.')
    await signWith(chapterId, targetId, 'Bản hai.')
    await signWith(chapterId, targetId, 'Bản ba.')

    // ── ③ 🔴 MỆNH ĐỀ TRUNG TÂM: bốn trường CÓ THẬT trên dây, không `undefined` ──────
    //     Đây là lưới cho đúng lớp lỗi mà Story 2.5 đã trả giá.
    const history = await readHistory(targetId)
    await expect(history.length).toBe(3)

    for (const row of history) {
      await expect(typeof row.id).toBe('number')
      await expect(typeof row.segment_id).toBe('number')
      await expect(typeof row.target_text).toBe('string')
      await expect(typeof row.created_at).toBe('string')
      // AC5 — ISO-8601 UTC có mili giây, đúng 24 ký tự.
      await expect(row.created_at.endsWith('Z')).toBe(true)
      await expect(row.created_at.length).toBe(24)
      await expect(row.segment_id).toBe(targetId)
    }

    // AC1 — MỚI NHẤT TRƯỚC. Một danh sách sắp ngược vẫn có ba hàng.
    await expect(history.map((r) => r.target_text)).toEqual(['Bản ba.', 'Bản hai.', 'Bản một.'])

    // ── ④ Khôi phục về bản CŨ NHẤT ─────────────────────────────────────────────────
    const oldest = history[history.length - 1]
    await expect(oldest.target_text).toBe('Bản một.')

    const outcome = await restore(targetId, oldest.id, false)
    // Văn bản hiện tại (`Bản ba.`) ĐÃ được ký nên nó có bản sao ⇒ không có gì để mất
    // ⇒ không hỏi lại.
    await expect(outcome.needs_confirmation).toBe(false)
    await expect(outcome.restored).toBe(true)
    await expect(outcome.status).toBe('draft')

    // ── ⑤ ĐỌC LẠI TỪ ĐĨA qua chính lệnh nạp của sản phẩm — AC2 ─────────────────────
    const after = await readSegmentsFromDisk()
    const seg = after.segments.find((s) => s.id === targetId)
    await expect(seg.target_text).toBe('Bản một.')
    await expect(seg.status).toBe('draft')

    // ── ⑥ 🔴 LỊCH SỬ KHÔNG DÀI THÊM — chữ ký #1(a), Ice ký 2026-08-16 ──────────────
    //     Mockup nói khôi phục "đẩy nó lên thành phiên bản thứ sáu"; bảng Rule của AD-31 có
    //     đúng SÁU hàng và không hàng nào là "khôi phục". AC2 đứng về phía AD-31.
    const afterRestore = await readHistory(targetId)
    await expect(afterRestore.length).toBe(3)

    // ── ⑦ Và nó dài thêm ở LƯỢT XÁC NHẬN KẾ TIẾP, do hàng 2 của AD-31 ──────────────
    //     Lời hứa "lịch sử chỉ dài thêm, không bao giờ ngắn đi" được giữ — chỉ muộn một nhịp.
    await browser.execute(async (i) => {
      const internals = window.__TAURI_INTERNALS__
      return internals.invoke('confirm_segment', { segmentId: i })
    }, targetId)

    const grown = await readHistory(targetId)
    await expect(grown.length).toBe(4)
    await expect(grown[0].target_text).toBe('Bản một.')
  })

  it('chốt chống mất bản nháp giữ lượt ghi lại, và KHÔNG một byte nào xuống đĩa', async () => {
    await openWorkspaceWithWork('Story 2.6 — chốt chống mất bản nháp')

    const before = await readSegmentsFromDisk()
    const targetId = before.segments[0].id
    const chapterId = before.chapter_id

    // Một lượt ký ⇒ một phiên bản có bản sao.
    await signWith(chapterId, targetId, 'Bản đã ký.')

    // Rồi gõ một bản nháp và KHÔNG ký — bản này không có bản sao nào ở đâu cả.
    const draft = 'Bản nháp chưa ai ký.'
    await browser.execute(
      async (c, i, t) => {
        const internals = window.__TAURI_INTERNALS__
        return internals.invoke('save_segment_targets', {
          chapterId: c,
          edits: [{ id: i, target_text: t }],
        })
      },
      chapterId,
      targetId,
      draft,
    )

    const history = await readHistory(targetId)
    await expect(history.length).toBe(1)

    // ── ① Lượt gọi đầu: GIỮ LẠI, và mang chính bản nháp ra ─────────────────────────
    const held = await restore(targetId, history[0].id, false)
    await expect(held.needs_confirmation).toBe(true)
    await expect(held.restored).toBe(false)
    await expect(held.unsigned_draft).toBe(draft)

    // ── ② 🔴 ĐĨA CÒN NGUYÊN ────────────────────────────────────────────────────────
    const untouched = await readSegmentsFromDisk()
    await expect(untouched.segments.find((s) => s.id === targetId).target_text).toBe(draft)

    // ── ③ Lượt gọi thứ hai với `force`: ghi thật ───────────────────────────────────
    const forced = await restore(targetId, history[0].id, true)
    await expect(forced.restored).toBe(true)

    const after = await readSegmentsFromDisk()
    await expect(after.segments.find((s) => s.id === targetId).target_text).toBe('Bản đã ký.')
  })
})
