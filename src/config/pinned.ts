/**
 * Adapter IPC phía webview cho **mục đã ghim** — Story 1.20, AC2 · AC3 · AC12.
 *
 * Cùng khuôn `./chapter.ts`: một lời gọi `invoke`, một `try/catch`, không quy tắc nghiệp
 * vụ nào ở đây — quy tắc sống ở Rust (`commands/pinned.rs`).
 *
 * ⚠️ **`invoke()` gửi tham số ở dạng camelCase.** Ba command của story này CÓ tham số, nên
 * đây là chỗ lời cảnh báo ở `./chapter.ts:7-9` thành thật: Tauri v2 tự chuyển `sourceCode`
 * (JS) thành `source_code` (Rust). Gửi `source_code` thẳng làm Rust nhận `None` cho một
 * tham số bắt buộc và lượt gọi trượt bằng một lỗi **không** phải `IpcError` — xem
 * `./project.ts` cho cùng quy ước ở `create_work_from_text`.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/**
 * Hình dạng `PinnedEntry` phía Rust — **`snake_case`, đúng như trên dây**.
 *
 * ⚠️ `commands/pinned.rs::PinnedEntry` cố ý KHÔNG đặt `#[serde(rename_all = "camelCase")]`
 * — cùng luật với `OpenChapter`/`BootstrapConfig`. Đổi một tên ở đây mà không đổi ở kia
 * cho ra `undefined` mà TypeScript không hề biết.
 */
export type PinnedEntry = {
  id: number
  source_code: string
  /** `dict_entry.id` **trong tệp `.db` của nguồn đó** — không duy nhất xuyên tệp. */
  entry_id: number
  /** Đầu mục THẬT trong từ điển, không phải truy vấn người dùng bôi đen. */
  headword: string
  /** `null` khi lượt tra không lấy về nghĩa nào — hình dạng THẬT, không một ca biên. */
  gloss: string | null
  pinned_at: string
}

/**
 * Ba trạng thái, cùng khuôn `ReadOpenChapterResult` — và ở story này phân biệt chúng là
 * **bắt buộc**, không một tiện nghi: Bẫy 4 nói bốn trạng thái của tab ghim dễ sập vào
 * nhau, và `{ entries: null, error: null }` *(chạy ngoài Tauri)* phải KHÁC hẳn
 * `{ entries: [], error: null }` *(đã mở Tác phẩm, chưa ghim gì)*.
 */
export type PinnedResult = {
  entries: readonly PinnedEntry[] | null
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

/** Tên command trên dây. Khớp `src-tauri/src/commands/pinned.rs` (module `wire`). */
const CMD_LIST = 'list_pinned_entries'
const CMD_PIN = 'pin_entry'
const CMD_UNPIN = 'unpin_entry'

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
 * Một bản CHUNG cho cả ba lượt gọi — chúng trả về **cùng một** hình dạng (bộ ghim mới,
 * đã sắp xếp), nên ba khối `try/catch` chép tay là ba chỗ để lệch nhau.
 *
 * 🔴 Rust trả về **cả bộ** ở cả ba đường, kể cả hai đường ghi. Đó không phải một sự lười:
 * một lượt `invoke` thứ hai ngay sau lượt ghi là một vòng IPC thừa cho cùng một sự thật,
 * và nó mở một khe mà một lượt ghi khác chen vào giữa.
 */
async function callPinned(cmd: string, args?: Record<string, unknown>): Promise<PinnedResult> {
  try {
    const entries = await invoke<PinnedEntry[]>(cmd, args)
    return { entries, error: null }
  } catch (err) {
    if (isIpcError(err)) return { entries: null, error: err }

    // 🔴 CÓ cầu IPC mà vẫn trượt bằng một thứ không phải `IpcError` ⇒ lỗi THẬT (command
    // chưa đăng ký, một panic phía Rust) — cùng bài học `./chapter.ts`.
    if (hasIpcBridge()) {
      console.error(`[pinned] \`${cmd}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { entries: null, error: UNKNOWN_IPC_ERROR }
    }

    // Không có cầu IPC — `npm run dev` trong một trình duyệt thường. Không phải một lỗi
    // để hiện lên, và **không** phải một danh sách rỗng để hiện lên.
    console.info(`[pinned] không gọi được \`${cmd}\` — chạy ngoài Tauri? ${String(err)}`)
    return { entries: null, error: null }
  }
}

/** Đọc toàn bộ mục ghim của Tác phẩm đang mở. Không ném. */
export async function listPinnedEntries(): Promise<PinnedResult> {
  return callPinned(CMD_LIST)
}

/**
 * Ghim một mục từ. Không ném. Trả về bộ ghim **mới**.
 *
 * ⚠️ Khoá tham số viết **camelCase** — xem doc-comment đầu tệp.
 */
export async function pinEntry(
  sourceCode: string,
  entryId: number,
  headword: string,
  gloss: string | null,
): Promise<PinnedResult> {
  return callPinned(CMD_PIN, { sourceCode, entryId, headword, gloss })
}

/** Bỏ ghim một mục từ. Không ném. Trả về bộ ghim **mới**. */
export async function unpinEntry(sourceCode: string, entryId: number): Promise<PinnedResult> {
  return callPinned(CMD_UNPIN, { sourceCode, entryId })
}
