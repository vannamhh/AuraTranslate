/**
 * Nhịp flush của Editor — **ba mệnh đề ĐỊNH LƯỢNG của AC11**. Story 2.3 · Task 1.3.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHÔNG DỰNG THÊM MỘT CỔNG CHO CÙNG BA MỆNH ĐỀ NÀY — AC25
 * ─────────────────────────────────────────────────────────────────────────────
 * Bốn đường nghiệm thu (vitest · cổng tĩnh · bàn đo · e2e) có bốn vai **không chồng nhau**,
 * và một mệnh đề được nghiệm thu ở **đúng một** đường. Ba mệnh đề dưới đây là mệnh đề về
 * **hành vi của một module thuần** ⇒ chúng thuộc vitest, và `scripts/check-layout.mjs` Kiểm B
 * **ở nguyên chỗ** canh nhịp ghi **bố cục** — một tính năng khác, một cặp hằng khác.
 *
 * ⚠️ Không `vi.useFakeTimers()` ở đây, và đó là có chủ ý: `createEditorFlush` **không đọc
 * `Date.now()`** — mọi thời điểm đi vào qua tham số. Một hàm như vậy kiểm được **tất định và
 * tức thời**; bọc nó bằng đồng hồ giả là đổi một bảo đảm lấy một thói quen.
 */
import { describe, expect, it } from 'vitest'
import {
  createEditorFlush,
  EDITOR_HARD_CAP_MS,
  EDITOR_IDLE_MS,
} from '../../src/panels/editorFlush'

/**
 * Chạy một dòng sự kiện gõ qua nhịp flush và trả về **thời điểm của từng lượt flush**.
 *
 * ⚠️ Đây là bản mô phỏng vòng lặp mà `EditorPanel.vue` chạy (`setTimeout` tới `deadline()`,
 * gửi lô, `onFlushed`), viết ở dạng **không đồng hồ và không timer** — cùng khuôn
 * `simulateWrites` của `writeSchedule.ts`. Nó sống trong tệp test chứ không trong sản phẩm
 * vì sản phẩm không có chỗ gọi nào cho nó.
 */
function flushesAt(eventsAt: readonly number[], idleMs: number, hardCapMs: number): number[] {
  const flush = createEditorFlush(idleMs, hardCapMs)
  const out: number[] = []
  let segmentId = 0

  for (const at of eventsAt) {
    // Mọi mốc đã tới hạn TRƯỚC sự kiện này thì timer đã nổ rồi — nó không chờ phím kế tiếp.
    const due = flush.deadline()
    if (due !== null && due <= at) {
      out.push(due)
      flush.onFlushed(due, flush.pending())
    }
    // Mỗi phím gõ là một lượt sửa của **cùng một câu** — ca thật nhất của một dòng gõ liền.
    segmentId = 1
    flush.markChanged(segmentId, `văn bản tại ${at}`, at)
  }

  const last = flush.deadline()
  if (last !== null) out.push(last)
  return out
}

describe('nhịp flush của Editor — hợp đồng AD-35', () => {
  it('mang đúng cặp hằng của Editor, KHÔNG mượn cặp của bố cục', () => {
    // Bố cục giữ 500/5000 (Story 1.14); Editor mang 2000/5000. Hai tính năng, hai cặp số.
    expect(EDITOR_IDLE_MS).toBe(2000)
    expect(EDITOR_HARD_CAP_MS).toBe(5000)
  })

  // ── Mệnh đề 1 + 2 — TRẦN CỨNG THẬT SỰ NỔ, và nó không bị phím gõ đẩy lùi ──────────
  it('một dòng phím LIÊN TỤC 30 giây cho ≥ 6 lượt flush ⇒ trần 5 s thật sự nổ', () => {
    // Sự kiện mỗi 100 ms trong 30 giây — người dùng gõ không nghỉ.
    const events = Array.from({ length: 300 }, (_, i) => (i + 1) * 100)
    const writes = flushesAt(events, EDITOR_IDLE_MS, EDITOR_HARD_CAP_MS)

    expect(writes.length).toBeGreaterThanOrEqual(6)
  })

  it('giữa hai lượt flush liên tiếp trong dòng đó, khoảng cách ≤ 5.000 ms', () => {
    const events = Array.from({ length: 300 }, (_, i) => (i + 1) * 100)
    const writes = flushesAt(events, EDITOR_IDLE_MS, EDITOR_HARD_CAP_MS)

    const gaps = writes.slice(1).map((at, i) => at - writes[i])
    // 🔴 Đây là mệnh đề mà NFR18 đứng trên: *"mất tối đa 5 giây công việc"*. Một khoảng
    // cách > 5.000 ms là một cửa sổ mà một lần sập máy ăn nhiều hơn ngưỡng đã hứa.
    for (const gap of gaps) expect(gap).toBeLessThanOrEqual(EDITOR_HARD_CAP_MS)
    expect(gaps.length).toBeGreaterThan(0)
  })

  // ── Mệnh đề 3 — VẾ IDLE THẬT SỰ CHẠY ────────────────────────────────────────────
  it('một dòng phím THƯA (một sự kiện rồi im) cho ĐÚNG MỘT lượt flush ở 2.000 ms', () => {
    const writes = flushesAt([0], EDITOR_IDLE_MS, EDITOR_HARD_CAP_MS)

    expect(writes).toEqual([EDITOR_IDLE_MS])
  })

  // ── Hệ quả của `onFlushed` — vế mà một lượt xoá trắng tập chờ sẽ nuốt ────────────
  it('gõ tiếp TRONG LÚC lô đang bay thì ký tự mới KHÔNG bị nuốt', () => {
    const flush = createEditorFlush()

    flush.markChanged(1, 'bản A', 0)
    const inFlight = flush.pending() // ảnh chụp đưa xuống Rust
    flush.markChanged(1, 'bản A dài hơn', 100) // người dùng gõ tiếp trong lúc lô bay
    flush.onFlushed(200, inFlight) // Rust trả lời cho ảnh chụp CŨ

    expect(flush.isDirty()).toBe(true)
    expect(flush.pending().get(1)).toBe('bản A dài hơn')
  })

  it('lô ghi xong trọn vẹn thì tập chờ SẠCH và lịch về null', () => {
    const flush = createEditorFlush()

    flush.markChanged(1, 'xong', 0)
    flush.onFlushed(200, flush.pending())

    expect(flush.isDirty()).toBe(false)
    expect(flush.deadline()).toBeNull()
  })

  it('một lượt ghi thành công KHÔNG được reset trần cứng khi tập chờ còn sót', () => {
    // 🔴 Ca này canh đúng dòng `if (dirty.size === 0) schedule.onWrite(now)`. Gọi `onWrite`
    // vô điều kiện sẽ nhả `firstChangeAt`, tức mở lại một cửa sổ 5 giây mới bằng một lượt
    // ghi thành công — chính thứ AC2 đóng.
    const flush = createEditorFlush()

    flush.markChanged(1, 'câu một', 0)
    const inFlight = flush.pending()
    flush.markChanged(2, 'câu hai', 100)
    flush.onFlushed(200, inFlight)

    // ⚠️ Mốc 4.900 được chọn vì nó là mốc **phân biệt được hai giả thuyết**, không phải một
    // con số cho đẹp. Ở mốc gần 0 thì vế idle luôn thắng và `deadline()` bằng nhau dưới cả
    // hai giả thuyết — một assert ở đó không kiểm gì:
    //   trần còn neo ở 0   ⇒ min(4900 + 2000, 0 + 5000)    = 5000  ← đúng
    //   trần đã bị reset   ⇒ min(4900 + 2000, 4900 + 5000) = 6900
    flush.markChanged(2, 'câu hai dài hơn', 4900)

    expect(flush.deadline()).toBe(EDITOR_HARD_CAP_MS)
  })

  it('một lô chỉ mang segment ĐÃ ĐỔI, không cả Chương (AC13)', () => {
    const flush = createEditorFlush()

    flush.markChanged(7, 'câu bảy', 0)
    flush.markChanged(9, 'câu chín', 50)

    expect([...flush.pending().keys()].sort((a, b) => a - b)).toEqual([7, 9])
  })
})
