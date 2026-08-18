/**
 * Adapter IPC phía webview cho việc đọc Chương **đang mở** — Story 1.16, AC8.
 *
 * Cùng khuôn `./project.ts`: một lời gọi `invoke`, một `try/catch`, không quy tắc
 * nghiệp vụ nào ở đây — quy tắc sống ở Rust (`commands/chapter.rs`).
 *
 * ⚠️ `invoke()` mặc định gửi tham số ở dạng **camelCase**; hàm này không nhận tham số
 * nào nên điều đó không áp dụng ở đây, nhưng vẫn ghi ra để chỗ gọi sau (nếu Epic 2 thêm
 * tham số chọn Chương) không quên.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/**
 * Hình dạng `OpenChapter` phía Rust — **`snake_case`, đúng như trên dây**.
 *
 * ⚠️ `commands/chapter.rs::OpenChapter` cố ý KHÔNG đặt `#[serde(rename_all = "camelCase")]`
 * — cùng luật với `WorkMeta`/`CreatedWork` ở `./project.ts`.
 */
export type OpenChapter = {
  chapter_id: number
  source_text: string
  /** `"zh"` hoặc `"en"` — trường bất biến của Tác phẩm (AD-18), không đoán từ nội dung. */
  source_lang: string
}

/** Ba trạng thái, cùng khuôn `CreateWorkResult`/`BootstrapResult` — xem doc-comment ở đó. */
export type ReadOpenChapterResult = {
  chapter: OpenChapter | null
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

/** Tên command trên dây. Khớp `src-tauri/src/commands/chapter.rs` (module `wire`). */
const CMD_READ_OPEN_CHAPTER = 'read_open_chapter'

/**
 * 🔵 **THÊM 2026-08-18 (Story 2.11 · FR26).** Khớp `commands/chapter.rs::wire`.
 */
const CMD_OPEN_ADJACENT_CHAPTER = 'open_adjacent_chapter'

/**
 * Hướng của một lượt chuyển Chương — **hai giá trị, đúng như trên dây**.
 *
 * ⚠️ Khớp `commands/chapter.rs::ChapterDirection`, nơi mỗi biến thể mang một
 * `#[serde(rename = …)]` tường minh. Đổi một chuỗi ở đây mà quên nửa Rust là một lượt IPC
 * trượt lúc chạy mà `vue-tsc` **không** thấy — hai nửa của một hợp đồng dây.
 */
export type ChapterDirection = 'next' | 'prev'

/**
 * Kết cục một lượt chuyển Chương — **ba giá trị, phân biệt được**.
 *
 * 🔴 `'at-first'`/`'at-last'` **không** phải lỗi và **không** phải `null`: chúng là hai câu
 * khác nhau trên màn hình (AC4 đòi *"báo rõ đã ở biên"*), và webview **không** được suy
 * chúng ra từ hướng nó vừa gửi — suy ra là dựng một nguồn sự thật thứ hai.
 */
export type ChapterSwitchOutcome = 'moved' | 'at-first' | 'at-last'

/**
 * Hình dạng `ChapterSwitch` phía Rust — `snake_case` trên dây, không `camelCase`.
 */
export type ChapterSwitch = {
  outcome: ChapterSwitchOutcome
  /** `OpenChapter` mới — **`null` khi và chỉ khi** `outcome !== 'moved'`. */
  chapter: OpenChapter | null
}

/** Ba trạng thái, cùng khuôn [`ReadOpenChapterResult`]. */
export type OpenAdjacentChapterResult = {
  switched: ChapterSwitch | null
  error: IpcError | null
}

/**
 * Có cầu IPC của Tauri trong window này không — cùng khuôn `./project.ts::hasIpcBridge`.
 */
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
 * Đọc Chương đang mở. Không ném — cùng lý do `callCreateWork` không ném
 * (`./project.ts`): chỗ gọi hiển thị lỗi bằng `tError()`, không bằng `try/catch` ở
 * tầng UI.
 */
export async function readOpenChapter(): Promise<ReadOpenChapterResult> {
  try {
    const chapter = await invoke<OpenChapter>(CMD_READ_OPEN_CHAPTER)
    return { chapter, error: null }
  } catch (err) {
    if (isIpcError(err)) return { chapter: null, error: err }

    // 🔴 CÓ cầu IPC mà vẫn trượt bằng một thứ không phải `IpcError` ⇒ lỗi THẬT (command
    // chưa đăng ký, một panic phía Rust), KHÔNG phải "chạy ngoài Tauri" — cùng bài học
    // đã ghi ở `./project.ts::callCreateWork`.
    if (hasIpcBridge()) {
      console.error(
        `[chapter] \`${CMD_READ_OPEN_CHAPTER}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { chapter: null, error: UNKNOWN_IPC_ERROR }
    }

    // Không có cầu IPC — `npm run dev` trong một trình duyệt thường. Không phải một
    // lỗi để hiện lên.
    console.info(`[chapter] không gọi được \`${CMD_READ_OPEN_CHAPTER}\` — chạy ngoài Tauri? ${String(err)}`)
    return { chapter: null, error: null }
  }
}

/**
 * Mở Chương **kề** theo một hướng. Story 2.11 · FR26 · AC1 · AC2 · AC4.
 *
 * Không ném — cùng lý do và cùng khuôn [`readOpenChapter`].
 *
 * ⚠️ `invoke()` gửi tham số dạng **camelCase**; `direction` là một từ đơn nên hai chiều
 * trùng nhau ở đây. Ghi ra vì đây là chỗ dễ sai nhất trên dây, không vì nó đang sai.
 *
 * 🔴 **KHÔNG một quy tắc nghiệp vụ nào ở đây.** *"Chương kề là chương nào"* sống trọn ở
 * `commands/chapter.rs::open_adjacent_chapter` *(so sánh bộ đôi `(ord, id)`)* — chữ ký #3
 * đường (a), Ice ký 2026-08-18. Hàm này chỉ chở một chuỗi hướng đi và một kết quả về.
 */
export async function openAdjacentChapter(
  direction: ChapterDirection,
): Promise<OpenAdjacentChapterResult> {
  try {
    const switched = await invoke<ChapterSwitch>(CMD_OPEN_ADJACENT_CHAPTER, { direction })
    return { switched, error: null }
  } catch (err) {
    if (isIpcError(err)) return { switched: null, error: err }

    if (hasIpcBridge()) {
      console.error(
        `[chapter] \`${CMD_OPEN_ADJACENT_CHAPTER}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { switched: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(
      `[chapter] không gọi được \`${CMD_OPEN_ADJACENT_CHAPTER}\` — chạy ngoài Tauri? ${String(err)}`,
    )
    return { switched: null, error: null }
  }
}
