/**
 * Nhịp ghi vị trí làm việc của Chương — **tầng THUẦN**. Story 5.7 (AC4/AC6), `chapter_position`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 CẶP HẰNG THỨ BA — VÀ NÓ **KHÔNG** MANG BẢO ĐẢM AD-35
 * ─────────────────────────────────────────────────────────────────────────────
 * `src/AGENTS.md` ghi *"Ba cặp hằng nhịp ghi, chỉ MỘT mang bảo đảm AD-35"* — bố cục
 * (`layout/writeSchedule.ts`) và Editor (`panels/editorFlush.ts`, CÓ mang) đã có mặt; đây
 * là cặp thứ ba. Mất một lượt ghi vị trí là mất **một lời nhắc** — mở lại Chương thì caret
 * rơi về câu đầu thay vì đúng câu đang dở — KHÔNG mất công việc: không một ký tự bản dịch
 * nào đi theo lượt ghi này. Cùng hạng với nhịp ghi bố cục, khác hạng với `EDITOR_IDLE_MS`.
 * Dùng chung HÌNH DẠNG (`createWriteSchedule`), không dùng chung bảo đảm — đừng gộp ba cặp.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO PHẢI CÓ MỘT NHỊP GHI, CHỨ KHÔNG GHI THẲNG Ở MỖI LƯỢT RỜI SEGMENT
 * ─────────────────────────────────────────────────────────────────────────────
 * `setEditorCaret` (`panels/editorPanelState.ts`) — định nghĩa DUY NHẤT của *"rời segment"*
 * — chạy theo **mỗi phím mũi tên** của `segmentNavigation`. Ghi thẳng một lượt IPC ở đó biến
 * mỗi lượt rê caret thành một job nối tiếp qua `store::Writer` — đúng bẫy `onDidLayoutChange`
 * mà `writeSchedule.ts` tồn tại để chặn (xem doc-comment của tệp đó).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ KHÔNG ĐỌC `Date.now()` BÊN TRONG — mọi thời điểm ĐI VÀO qua tham số
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng luật `writeSchedule.ts`/`editorFlush.ts` đã ghi: một hàm nhận thời điểm qua tham số
 * kiểm được TẤT ĐỊNH và TỨC THỜI trên vitest, không cần `sleep` thật hay đồng hồ giả.
 */
import { createWriteSchedule } from '../layout/writeSchedule'

/**
 * Yên bao lâu thì ghi vị trí — **500 ms**, cùng con số với nhịp ghi bố cục (`IDLE_MS`) —
 * không phải trùng hợp: cả hai đều thuộc nhóm KHÔNG mang bảo đảm AD-35, và cả hai đều chỉ
 * cần "gần đúng thời điểm gần nhất", không cần đóng băng một nhịp gõ như Editor.
 */
export const POSITION_IDLE_MS = 500

/**
 * Trần cứng — **5000 ms**. Không mang ý nghĩa NFR18 (đó là lời hứa cho CÔNG VIỆC của người
 * dùng); ở đây nó chỉ giữ cho vị trí đã lưu không lệch quá xa vị trí thật khi người dùng rê
 * caret liên tục trong một thời gian dài mà chưa dừng.
 */
export const POSITION_HARD_CAP_MS = 5000

/** Vị trí đang chờ ghi: đúng MỘT cặp `(chapterId, segmentId)`, hoặc `null` khi đã sạch. */
export type PendingPosition = { chapterId: number; segmentId: number }

export type PositionFlush = {
  /**
   * Caret vừa dời sang `segmentId` của `chapterId`, tại `now`.
   * @returns thời điểm TUYỆT ĐỐI phải ghi. Chỗ gọi đặt lại timer về đúng mốc này.
   */
  markMoved(chapterId: number, segmentId: number, now: number): number
  /** Vị trí đang chờ ghi, hoặc `null` khi sạch. */
  pending(): PendingPosition | null
  /**
   * Vừa ghi xong `flushed` tại `now`.
   *
   * 🔴 Chỉ xoá tập chờ khi nó **vẫn còn khớp** `flushed` — cùng lý lẽ
   * `editorFlush.ts::onFlushed`: caret có thể đã rê tiếp trong lúc lượt ghi đang bay, và xoá
   * trắng vô điều kiện ở đây là nuốt đúng lượt rê đó, không một dấu hiệu nào báo.
   */
  onFlushed(now: number, flushed: PendingPosition): void
  /** Có vị trí chưa ghi không? Điều kiện của lượt flush lúc đổi Chương / đổi Tác phẩm / thoát. */
  isDirty(): boolean
  /** Mốc phải ghi đang chờ, hoặc `null` khi sạch. */
  deadline(): number | null
  /**
   * Vứt vị trí đang chờ **không** ghi — chỉ dùng khi Tác phẩm/Chương đang mở bị THAY và vị
   * trí cũ đã hết ý nghĩa (một Chương/Tác phẩm khác không đọc `chapter_position` của Chương
   * vừa rời). Cùng khuôn `EditorFlush::reset`.
   */
  reset(): void
}

/**
 * Dựng một nhịp flush cho vị trí làm việc của Chương.
 *
 * ⚠️ Hai tham số có mặt để **test lái được bằng số nhỏ**, không phải để chỗ gọi trong sản
 * phẩm đổi nhịp — cùng khuôn `createEditorFlush`. Chỗ gọi thật để mặc định.
 */
export function createPositionFlush(
  idleMs: number = POSITION_IDLE_MS,
  hardCapMs: number = POSITION_HARD_CAP_MS,
): PositionFlush {
  const schedule = createWriteSchedule(idleMs, hardCapMs)
  let pendingValue: PendingPosition | null = null

  return {
    markMoved(chapterId, segmentId, now) {
      pendingValue = { chapterId, segmentId }
      return schedule.onChange(now)
    },

    pending: () => pendingValue,

    onFlushed(now, flushed) {
      if (
        pendingValue !== null &&
        pendingValue.chapterId === flushed.chapterId &&
        pendingValue.segmentId === flushed.segmentId
      ) {
        pendingValue = null
      }
      // Cùng lý lẽ `editorFlush.ts`: chỉ dọn lịch khi tập chờ đã SẠCH. Còn khác ⇒ caret đã
      // rê tiếp trong lúc lô bay, và chu kỳ hiện tại phải chạy tiếp.
      if (pendingValue === null) schedule.onWrite(now)
    },

    isDirty: () => pendingValue !== null,
    deadline: () => (pendingValue === null ? null : schedule.deadline()),
    reset() {
      pendingValue = null
      schedule.onWrite(0)
    },
  }
}
