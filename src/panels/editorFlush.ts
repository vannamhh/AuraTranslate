/**
 * Nhịp flush của Editor + tập segment đã đổi — **tầng THUẦN**. Story 2.3 · AC1 · AC2 · AC11 · AD-35.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 TỆP NÀY **TÁI DÙNG** `createWriteSchedule`, KHÔNG CHÉP NÓ — Ice chốt Quyết định #2
 * ─────────────────────────────────────────────────────────────────────────────
 * Ice ký đường **(a)** ngày 2026-08-12: `import` thẳng `../layout/writeSchedule`, hằng riêng
 * khai ở đây. Lý lẽ đo được của lượt ký, ghi lại để không ai "dọn" nó về sau:
 *
 * - Đường (c) — *nâng `createWriteSchedule` lên một chỗ trung lập* — là một lượt **di chuyển
 *   tệp**, và `scripts/check-layout.mjs:295-296` `import()` tệp đó bằng **đường dẫn viết
 *   thẳng** `join(SRC_ROOT, 'layout', 'writeSchedule.ts')` rồi `abort()` nếu không nạp được.
 *   Đổi chỗ tệp làm **Kiểm B đỏ ngay**, và chạm `FILE_FLOOR` của bốn cổng — để đổi lấy đúng
 *   một thứ: một cái tên đọc thuận hơn.
 * - Đường (b) — *chép hàm sang đây* — là hai bản cài đặt của cùng một `Math.min`. Chúng sẽ
 *   rẽ nhau ở lượt sửa thứ hai, và lúc đó không ai biết bản nào đúng.
 *
 * ⚠️ Hệ quả phải giữ: **KHÔNG sửa hành vi của `createWriteSchedule` ở story này.**
 * `check-layout.mjs` Kiểm B đang đứng trên nó. Cần một hành vi khác ⇒ đó là một tham số mới
 * có giá trị mặc định giữ nguyên hành vi cũ, cộng một test cho **cả hai** giá trị.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ KHÔNG ĐỌC `Date.now()` BÊN TRONG — mọi thời điểm ĐI VÀO qua tham số
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng luật `writeSchedule.ts:18-20` đã ghi, và luật đó **không hết hiệu lực** chỉ vì kho nay
 * có `vi.useFakeTimers()`: một hàm nhận thời điểm qua tham số kiểm được **tất định và tức
 * thời**; một hàm bị đồng hồ giả bao quanh kiểm được đúng chừng nào không ai quên bật đồng hồ
 * giả. Đường rẻ hơn không phải đường đúng hơn.
 */
import { createWriteSchedule } from '../layout/writeSchedule'

/**
 * Yên bao lâu thì flush — **2 giây**, AC1 nguyên văn.
 *
 * ⚠️ **TẠM — chủ là Story 2.4** (`ARCHITECTURE-SPINE.md:883`, hàng Deferred *"ngưỡng WAL +
 * nhịp flush cụ thể"*). Story này dựng **cơ chế**; 2.4 hiệu chỉnh **con số**, và nó phải
 * hiệu chỉnh cùng lúc với `Tuning::wal_threshold_bytes` vì hai số đó **đánh đổi lẫn nhau**.
 *
 * 🔴 Con số này đã có một chỗ khác đứng trên nó: `Tuning::idle_before_passive = 5 s` được đặt
 * *"cố ý dài hơn nhịp flush 2 s của AD-35"* (`core/store/mod.rs:207-208`). Đổi 2000 ở đây mà
 * không đọc dòng đó là làm luồng checkpoint đánh nhau với đường gõ.
 */
export const EDITOR_IDLE_MS = 2000

/**
 * Trần cứng — **5 giây**, AC2 nguyên văn, và nó **KHÔNG được reset bởi phím gõ**.
 *
 * ⚠️ **TẠM — chủ là Story 2.4**, cùng hàng Deferred với [`EDITOR_IDLE_MS`].
 *
 * Đây là con số mà NFR18 (*"mất tối đa 5 giây công việc"*) đứng trên. Vế *"không reset"* do
 * `createWriteSchedule` giữ: `firstChangeAt` không được gán lại cho tới khi `onWrite` chạy.
 *
 * ⚠️ **KHÔNG** đè lên `IDLE_MS`/`HARD_CAP_MS` của `../layout/writeSchedule`. Bố cục giữ
 * `500/5000` và con số đó thuộc Story 1.14; hai tính năng, hai cặp số, một hàm.
 */
export const EDITOR_HARD_CAP_MS = 5000

/**
 * 🔴 **SÀN giữa hai lượt thử lại sau một lượt ghi TRƯỢT** — code review 2026-08-13, Ice ký.
 *
 * ⚠️ **TẠM — chủ là Story 2.4**, cùng hàng Deferred với [`EDITOR_IDLE_MS`] và
 * [`EDITOR_HARD_CAP_MS`] (`ARCHITECTURE-SPINE.md:883`).
 *
 * Vì sao con số này phải tồn tại, và nó KHÔNG phải một lượt "cho chắc": `onFlushed()` chỉ gọi
 * `schedule.onWrite()` khi tập chờ đã **sạch**, nên một lượt ghi trượt để `deadline()` đứng
 * nguyên ở một mốc **đã quá hạn**. Chỗ gọi tính độ trễ bằng `due - Date.now()` ⇒ ra một số
 * **âm** ⇒ kẹp về `0` ⇒ `setTimeout(…, 0)` ⇒ gọi lại ngay ⇒ trượt lại. Vòng lặp chặt, không
 * nhịp, và nó dồn thẳng vào writer **duy nhất, nối tiếp** của AD-11.
 *
 * Đường tái lập chắc chắn, không cần một lỗi kho nào: `npm run dev` trong một trình duyệt
 * thường không có cầu IPC ⇒ `outcome` là `null` ở **mọi** lượt ⇒ một ký tự là đủ.
 *
 * 🔴 Sàn này **KHÔNG** đụng tới trần cứng của AC2: `firstChangeAt` vẫn không được gán lại cho
 * tới khi `onWrite` chạy, nên một chuỗi lượt trượt không hề nới cửa sổ 5 giây — nó chỉ thôi
 * quay vòng. Đặt bằng [`EDITOR_IDLE_MS`] có chủ ý: một lượt thử lại không được dày hơn một
 * lượt gõ bình thường.
 */
export const EDITOR_RETRY_FLOOR_MS = 2000

/** Một segment đã đổi và chưa ghi xuống: `id` → văn bản đích **hiện tại**. */
export type PendingEdits = ReadonlyMap<number, string>

export type EditorFlush = {
  /**
   * Người dùng vừa sửa `segmentId` thành `targetText` tại `now`.
   * @returns thời điểm TUYỆT ĐỐI phải flush. Chỗ gọi đặt lại timer về đúng mốc này.
   */
  markChanged(segmentId: number, targetText: string, now: number): number
  /** Tập đã đổi chưa ghi. Ảnh chụp để đưa xuống Rust — **một lô, một lượt IPC** (AC13). */
  pending(): PendingEdits
  /**
   * Vừa ghi xong `flushed` tại `now`.
   *
   * 🔴 **Chỉ xoá khỏi tập chờ những mục mà văn bản KHÔNG đổi kể từ lúc chụp ảnh.** Một lượt
   * `Store::write` chặn qua biên IPC là bất đồng bộ với đường gõ; người dùng gõ tiếp **trong
   * lúc** lô đang bay là chuyện thường. Xoá trắng tập chờ ở đây là nuốt đúng những ký tự đó,
   * và không một dấu hiệu nào báo — đúng lớp khuyết tật NFR18 mà cả Epic 2 chống.
   */
  onFlushed(now: number, flushed: PendingEdits): void
  /** Có gì chưa ghi không? Điều kiện của lượt flush lúc rời segment / đóng Tác phẩm / thoát. */
  isDirty(): boolean
  /** Mốc phải ghi đang chờ, hoặc `null` khi sạch. */
  deadline(): number | null
  /**
   * Vứt tập chờ **không** ghi — chỉ dùng khi Tác phẩm đang mở bị THAY và tập chờ đã được
   * flush xong trước đó. Cùng khuôn `resetEditorPanel()`.
   */
  reset(): void
}

/**
 * Dựng một nhịp flush cho Editor.
 *
 * ⚠️ Hai tham số có mặt để **test lái được bằng số nhỏ**, không phải để chỗ gọi trong sản
 * phẩm đổi nhịp: cùng luật `Tuning` thu nhỏ mà §Testing standards đã đặt cho tầng Rust
 * (*"tham số hoá, đừng `sleep`"*). Chỗ gọi thật để mặc định.
 */
export function createEditorFlush(
  idleMs: number = EDITOR_IDLE_MS,
  hardCapMs: number = EDITOR_HARD_CAP_MS,
): EditorFlush {
  const schedule = createWriteSchedule(idleMs, hardCapMs)
  /** `id` → văn bản đích hiện tại. Chỉ mang segment **đã đổi**, không cả Chương (AC13). */
  const dirty = new Map<number, string>()

  return {
    markChanged(segmentId, targetText, now) {
      dirty.set(segmentId, targetText)
      return schedule.onChange(now)
    },

    pending() {
      // Ảnh chụp: chỗ gọi giữ nó qua một lượt `await`, và `dirty` đổi dưới chân trong lúc đó.
      return new Map(dirty)
    },

    onFlushed(now, flushed) {
      for (const [id, text] of flushed) {
        if (dirty.get(id) === text) dirty.delete(id)
      }
      // 🔴 Chỉ dọn lịch khi tập chờ đã SẠCH. Còn sót ⇒ người dùng đã gõ tiếp trong lúc lô
      // bay, và chu kỳ hiện tại phải chạy tiếp: gọi `onWrite` ở đây sẽ nhả `firstChangeAt`,
      // tức **reset trần cứng bằng một lượt ghi thành công** và mở lại đúng cửa sổ 5 giây
      // mà AC2 đóng.
      if (dirty.size === 0) schedule.onWrite(now)
    },

    isDirty: () => dirty.size > 0,
    deadline: () => (dirty.size === 0 ? null : schedule.deadline()),
    reset() {
      dirty.clear()
      schedule.onWrite(0)
    },
  }
}
