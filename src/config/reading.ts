/**
 * Adapter IPC phía webview cho Chế độ đọc — Story 5.11, FR11.
 *
 * Cùng khuôn `./chapter.ts`: một `invoke`, một `try/catch`, hình dạng BA TRẠNG THÁI
 * `{ chapter: ReadingChapter | null, error: IpcError | null }` — KHÔNG BAO GIỜ ném. Tầng
 * UI (`src/modes/readingState.ts`) hiển thị lỗi bằng `tError()`, không bằng `try/catch`.
 *
 * ⚠️ `invoke()` gửi tham số dạng camelCase; hàm này không nhận tham số nào nên điều đó
 * không áp dụng ở đây — cùng ghi chú mà `./chapter.ts::readOpenChapter` để lại.
 */
import { invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'

/**
 * Một câu, cho Chế độ đọc — khớp `commands::segment::ReadingSegment` phía Rust,
 * `snake_case` trên dây.
 *
 * ⚠️ **KHÔNG** `is_omitted`: câu đã cắt bỏ không bao giờ đi qua dây tới đây — chốt lọc
 * sống ở Rust (`core::segment::omit`/`core::segment::reading`, AD-1). Một component đọc
 * kiểu này **không có** đường nào để tự lọc lần hai, vì không có gì để lọc.
 */
export type ReadingSegment = {
  id: number
  source_text: string
  target_text: string
}

/**
 * Một đoạn — khớp `commands::segment::ReadingParagraph`. Không đoạn nào rỗng: nếu mọi câu
 * của một đoạn đều đã bị cắt bỏ, đoạn đó không xuất hiện trong `paragraphs` (Rust quyết,
 * xem doc-comment của `paragraphs_in_translation`).
 */
export type ReadingParagraph = {
  segments: ReadingSegment[]
}

/**
 * Chương đang mở, đã gom đoạn — khớp `commands::segment::ReadingChapter`, `snake_case`
 * trên dây.
 */
export type ReadingChapter = {
  chapter_id: number
  chapter_ord: number
  /** `null` ⇒ Chương chưa đặt tên — cùng ngữ nghĩa `ChapterRow::title` (Story 5.7). */
  chapter_title: string | null
  paragraphs: ReadingParagraph[]
}

/** Ba trạng thái cho [`readReadingChapter`], cùng khuôn `ReadOpenChapterResult`. */
export type ReadReadingChapterResult = {
  chapter: ReadingChapter | null
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
    // ⚠️ Cùng canh gác mà mọi adapter khác của kho này giữ — `IpcError` là một lời khai về
    //    dữ liệu đã qua dây, không một bảo đảm của trình biên dịch.
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
    v.params !== null
  )
}

function isReadingSegment(value: unknown): value is ReadingSegment {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ReadingSegment>
  return typeof v.id === 'number' && typeof v.source_text === 'string' && typeof v.target_text === 'string'
}

/** 🔴 Kiểm MỌI phần tử, không chỉ phần tử đầu — cùng lý lẽ `config/library.ts::isSearchHitArray`. */
function isReadingSegmentArray(value: unknown): value is ReadingSegment[] {
  return Array.isArray(value) && value.every(isReadingSegment)
}

function isReadingParagraph(value: unknown): value is ReadingParagraph {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ReadingParagraph>
  return isReadingSegmentArray(v.segments)
}

function isReadingParagraphArray(value: unknown): value is ReadingParagraph[] {
  return Array.isArray(value) && value.every(isReadingParagraph)
}

/** Vị từ kiểm kiểu LÚC CHẠY cho toàn bộ `ReadingChapter` — kiểm MỌI trường, MỌI phần tử. */
function isReadingChapter(value: unknown): value is ReadingChapter {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ReadingChapter>
  return (
    typeof v.chapter_id === 'number' &&
    typeof v.chapter_ord === 'number' &&
    (typeof v.chapter_title === 'string' || v.chapter_title === null) &&
    isReadingParagraphArray(v.paragraphs)
  )
}

/** Có cầu IPC thật hay chỉ là phiên `npm run dev` trong trình duyệt thường. */
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

/** Tên command trên dây. Khớp `src-tauri/src/commands/segment.rs` (module `wire`). */
const CMD_READ_READING_CHAPTER = 'read_reading_chapter'

/**
 * Đọc Chương đang mở, đã gom đoạn cho Chế độ đọc. Không ném — cùng lý do và cùng khuôn
 * [`./chapter.ts::readOpenChapter`].
 */
export async function readReadingChapter(): Promise<ReadReadingChapterResult> {
  try {
    const raw = await invoke<unknown>(CMD_READ_READING_CHAPTER)
    if (!isReadingChapter(raw)) {
      console.error(`[reading] \`${CMD_READ_READING_CHAPTER}\` trả một hình dạng không phải ReadingChapter: ${String(raw)}`)
      return { chapter: null, error: UNKNOWN_IPC_ERROR }
    }
    return { chapter: raw, error: null }
  } catch (err) {
    if (isIpcError(err)) return { chapter: null, error: err }

    // 🔴 CÓ cầu IPC mà vẫn trượt bằng một thứ không phải `IpcError` ⇒ lỗi THẬT, KHÔNG
    // phải "chạy ngoài Tauri" — cùng bài học đã ghi ở `./chapter.ts::readOpenChapter`.
    if (hasIpcBridge()) {
      console.error(`[reading] \`${CMD_READ_READING_CHAPTER}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { chapter: null, error: UNKNOWN_IPC_ERROR }
    }

    // Không có cầu IPC — `npm run dev` trong một trình duyệt thường. Không phải một lỗi
    // để hiện lên; trạng thái thứ BA của I/O Matrix.
    console.info(`[reading] không gọi được \`${CMD_READ_READING_CHAPTER}\` — chạy ngoài Tauri? ${String(err)}`)
    return { chapter: null, error: null }
  }
}
