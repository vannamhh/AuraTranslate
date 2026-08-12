/**
 * Adapter IPC phía webview cho lệnh **tách segment tường minh** — Story 2.1, AC8 · AC14.
 *
 * Cùng khuôn `./chapter.ts`: một lời gọi `invoke`, một `try/catch`, không quy tắc nghiệp vụ
 * nào ở đây — quy tắc sống ở Rust (`commands/segment.rs`).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHÔNG CÓ HÀM "TÁCH LẠI TOÀN BỘ THƯ VIỆN" Ở ĐÂY, VÀ ĐÓ LÀ AC8 VẾ HAI
 * ─────────────────────────────────────────────────────────────────────────────
 * AC8 nguyên văn: *"không có đường nào tự động tách lại toàn bộ Thư viện"*. Tệp này giao vế
 * đó bằng cách **không tồn tại** một hàm như vậy — một Chương một lượt, và mỗi lượt do người
 * dùng bấm. `tests/segment_boundary.rs` khẳng định điều đó trên cả cây nguồn.
 *
 * Vế một của AC8 (nút **tái tách** một Chương đã có segment, kèm cảnh báo về dữ liệu sẽ về
 * hưu) thuộc **Story 2.8** — nó cần ngữ nghĩa về hưu của AD-5, mà hôm nay chưa có
 * `SegmentVersion` nào để giữ lại. Lệnh dưới đây **từ chối** một Chương đã có segment.
 *
 * ⚠️ `invoke()` gửi tham số ở dạng **camelCase** — `chapter_id` phía Rust đi trên dây dưới
 * tên `chapterId`. Đây là chỗ duy nhất trong kho gõ cái tên đó.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/**
 * Hình dạng `SplitOutcome` phía Rust — **`snake_case`, đúng như trên dây**.
 *
 * ⚠️ `commands/segment.rs::SplitOutcome` cố ý KHÔNG đặt `#[serde(rename_all = "camelCase")]`
 * — cùng luật với `OpenChapter`/`WorkMeta`/`CreatedWork`.
 */
export type SplitOutcome = {
  chapter_id: number
  /** Số câu vừa ghi xuống. **0 là giá trị hợp lệ** — một Chương chỉ có khoảng trắng. */
  segment_count: number
}

/** Ba trạng thái, cùng khuôn `ReadOpenChapterResult` — xem doc-comment ở `./chapter.ts`. */
export type SplitChapterResult = {
  outcome: SplitOutcome | null
  error: IpcError | null
}

function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    // ⚠️ Hình dạng `IpcError` là một LỜI KHAI về dữ liệu đã qua dây IPC, không một bảo đảm của trình
    //    biên dịch. Rust có thể trả `null` cho `params` sau một lượt đổi lược đồ, và guard này là chỗ
    //    duy nhất biết điều đó.
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
    v.params !== null
  )
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/segment.rs` (module `wire`). */
const CMD_SPLIT_CHAPTER = 'split_chapter_into_segments'

/** Có cầu IPC của Tauri trong window này không — cùng khuôn `./chapter.ts::hasIpcBridge`. */
function hasIpcBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

/** Lỗi hồi phòng khi Rust trượt bằng một thứ không phải `IpcError`. */
const UNKNOWN_IPC_ERROR: IpcError = {
  code: 'ipc.unknown',
  message_key: 'err.unknown',
  params: {},
  retryable: false,
}

/**
 * Tách **một** Chương đã có trên đĩa thành các hàng `segment`.
 *
 * Không ném — cùng lý do `readOpenChapter` không ném: chỗ gọi hiển thị lỗi bằng `tError()`,
 * không bằng `try/catch` ở tầng UI.
 *
 * Rust từ chối khi Chương đã có segment (`segment.already_split`) hoặc `chapterId` không tồn
 * tại (`segment.chapter_not_found`). **Không** ghi đè im lặng — AD-4 đóng băng ranh giới
 * vĩnh viễn, nên một lượt ghi đè là một lượt cho về hưu mà không ai ký.
 */
export async function splitChapterIntoSegments(chapterId: number): Promise<SplitChapterResult> {
  try {
    const outcome = await invoke<SplitOutcome>(CMD_SPLIT_CHAPTER, { chapterId })
    return { outcome, error: null }
  } catch (err) {
    if (isIpcError(err)) return { outcome: null, error: err }

    // 🔴 CÓ cầu IPC mà vẫn trượt bằng một thứ không phải `IpcError` ⇒ lỗi THẬT (command
    // chưa đăng ký, một panic phía Rust), KHÔNG phải "chạy ngoài Tauri".
    if (hasIpcBridge()) {
      console.error(
        `[segment] \`${CMD_SPLIT_CHAPTER}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { outcome: null, error: UNKNOWN_IPC_ERROR }
    }

    // Không có cầu IPC — `npm run dev` trong một trình duyệt thường. Không phải một lỗi
    // để hiện lên.
    console.info(`[segment] không gọi được \`${CMD_SPLIT_CHAPTER}\` — chạy ngoài Tauri? ${String(err)}`)
    return { outcome: null, error: null }
  }
}
