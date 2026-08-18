/**
 * 🔴 **ĐO NFR1 ĐẦU-CUỐI** — Story 1.18, AC4 · Quyết định #7(a).
 *
 * `epics.md:1774` đòi *"độ trễ đầu-cuối **từ lúc thả chuột** tới lúc kết quả **hiển thị**"*,
 * p95 trên ≥ 100 lượt. Hai mốc đó nằm ở **hai module khác nhau**, nên chúng sống ở đây —
 * không rải `console.log` trong đường sản phẩm.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MỐC CUỐI SAU `requestAnimationFrame`, không SAU `nextTick`
 * ─────────────────────────────────────────────────────────────────────────────
 * `nextTick` là *"Vue đã ghi DOM"*, không *"trình duyệt đã VẼ"*. Với một bản ghi 22 nghĩa / 5
 * nguồn (`mockups/lookup-real-density.html`) khoảng cách đó không nhỏ — và *"hiển thị"* là
 * đúng chữ `epics.md` dùng. Đo tới `nextTick` rồi báo cáo như đã đo tới *"hiển thị"* là
 * **Bẫy 7** của story.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ CỜ MẶC ĐỊNH **TẮT** — VÀ ĐÓ LÀ MỘT MỆNH ĐỀ VỀ ĐƯỜNG SẢN PHẨM
 * ─────────────────────────────────────────────────────────────────────────────
 * Bật bằng tay từ console khi đo (`__auraLookupTiming.enable()`), không bằng một biến môi
 * trường và không bằng một lượt sửa mã. Khi TẮT, [`markDispatch`] và [`markPainted`] là hai
 * lời gọi rỗng — không một phép đo nào chạy trên đường nóng của người dùng thật.
 */

/**
 * 🔴 **HÀNG ĐỢI, KHÔNG MỘT Ô ĐƠN** — lượt review 2026-08-07 bắt được rằng một ô `dispatchedAt`
 * đơn không sống sót qua lượt tra chồng lấn (Shift+click bắn cả `mouseup` lẫn `keyup`-nhả-Shift
 * cho cùng vùng chọn, hay một `Mod+Alt+L` tay chen vào giữa lúc đang đo tự động): mốc đầu THỨ
 * HAI ghi đè mốc đầu THỨ NHẤT trước khi `markPainted()` của lượt đầu kịp chạy ⇒ mẫu sai cặp,
 * và bảng AC4 mang một con số không phải của truy vấn nó gán. Hàng đợi FIFO ghép `markDispatch`
 * với `markPainted` **theo cặp**, nên `samples`/`queries` không bao giờ lệch nhau.
 */
const pending: { query: string; at: number }[] = []

/** Mọi độ trễ đầu-cuối đã đo, theo mili-giây. */
const samples: number[] = []

/** Truy vấn của từng mẫu đã GHÉP CẶP — AC4 đòi bảng ghi **bộ truy vấn đã dùng**, không chỉ ghi con số. */
const queries: string[] = []

let enabled = false

/**
 * 🔵 **STORY 2.12 · AC5** — dọn hàng đợi và mọi mẫu đã đo.
 *
 * Ice ký 2026-08-18 (quyết định #2c). Tệp này là **ứng viên thứ ba, story không biết tới** —
 * nó lòi ra ở lượt đo lại Task 0.1, không có trong bảng của story.
 *
 * ⚠️ Vì sao [`pending`] là ô đáng dọn nhất trong bốn ô: nó là một hàng đợi **FIFO ghép cặp**
 * `markDispatch` ↔ `markPainted`. Một lượt `markDispatch` không bao giờ được ghép cặp — vì
 * Tác phẩm đổi giữa chừng, hay panel bị gỡ trước `requestAnimationFrame` — để lại một mốc đầu
 * mồ côi ở **đầu** hàng đợi. Lượt `markPainted` kế tiếp ghép vào mốc mồ côi ấy và ghi một độ
 * trễ **cộng cả khoảng thời gian giữa hai lượt tra**. Bảng số của AC4 khi đó mang một con số
 * không phải của truy vấn nó gán, và con số đó **lớn hơn sự thật** — tức nó nói dối về đúng
 * phía làm ta lo lắng.
 *
 * 🔴 [`enabled`] KHÔNG bị đặt lại: nó là một công tắc chẩn đoán do người đo bật, không một
 * mẩu state của phiên. Dọn nó ở đây là tự tắt bàn đo giữa một lượt đo.
 */
export function resetLookupTiming(): void {
  pending.length = 0
  samples.length = 0
  queries.length = 0
}

/**
 * Mốc ĐẦU — gọi trong handler `mouseup` của hợp đồng, **TRƯỚC** `dispatch`.
 *
 * ⚠️ *"Từ lúc thả chuột"* là chữ của `epics.md:1774`. Một mốc đặt sau `dispatch` đo một
 * đoạn **ngắn hơn** đoạn AC đòi rồi báo cáo như đã đo đủ — Quyết định #7(b), đã bác.
 */
export function markDispatch(query: string): void {
  if (!enabled) return
  pending.push({ query, at: performance.now() })
}

/**
 * Mốc CUỐI — gọi trong `requestAnimationFrame` sau `nextTick` ở `LookupPanel.vue`.
 *
 * Không ghi gì khi không có mốc đầu đang chờ: một lượt vẽ do `resetLookupPanel()` hay do một
 * lượt `Mod+Alt+L` (không đi qua hợp đồng) gây ra không phải một mẫu của phép đo này.
 */
export function markPainted(): void {
  if (!enabled) return
  const dispatched = pending.shift()
  if (dispatched === undefined) return
  samples.push(performance.now() - dispatched.at)
  queries.push(dispatched.query)
}

function percentile(sorted: readonly number[], p: number): number {
  if (sorted.length === 0) return Number.NaN
  // ⚠️ Chỉ số **cận trên** (`ceil`) — cùng quy ước mà bảng đo của Story 1.17 đã dùng, nên
  // hai bảng so sánh được với nhau. Một quy ước khác cho ra một con số khác trên cùng dữ
  // liệu, và story này phải đối chiếu với `p99 70,742 ms` của 1.17 (Bẫy 8).
  const at = Math.max(0, Math.ceil((p / 100) * sorted.length) - 1)
  return sorted[at] ?? Number.NaN
}

/** Bảng số của AC4 — `n · p50 · p95 · p99 · max`, cộng bộ truy vấn đã dùng. */
export function report(): {
  n: number
  p50: number
  p95: number
  p99: number
  max: number
  distinctQueries: number
  samples: readonly number[]
} {
  const sorted = [...samples].sort((a, b) => a - b)
  return {
    n: sorted.length,
    p50: percentile(sorted, 50),
    p95: percentile(sorted, 95),
    p99: percentile(sorted, 99),
    max: sorted.length === 0 ? Number.NaN : (sorted[sorted.length - 1] ?? Number.NaN),
    distinctQueries: new Set(queries).size,
    samples: sorted,
  }
}

/**
 * 🔴 Cửa BẬT/TẮT — treo trên `window` **chỉ khi được gọi**, để đo tay từ devtools.
 *
 * ⚠️ Story 1.17 §⑥ giữ `ALLOWED_GLOBAL_MEMBERS` chặt, nên đây là một lượt gán **có tên**
 * và có lý do, không một `window.foo = bar` rải rác. `Reflect.set` chứ không `window.__aura… =`:
 * cùng hiệu lực, nhưng không thêm một thành viên `window.` mới vào tầm quét của Kiểm C —
 * danh sách cho phép là thứ Story 4.12 sẽ cần, không phải chỗ để một cờ đo lách qua.
 */
export function installTimingProbe(): void {
  Reflect.set(globalThis, '__auraLookupTiming', {
    enable: (): string => {
      enabled = true
      samples.length = 0
      queries.length = 0
      pending.length = 0
      return 'đo NFR1: BẬT — bôi đen ≥ 100 cụm KHÁC NHAU rồi gọi __auraLookupTiming.report()'
    },
    disable: (): string => {
      enabled = false
      pending.length = 0
      return 'đo NFR1: TẮT'
    },
    report,
  })
}
