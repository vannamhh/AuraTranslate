/**
 * Adapter IPC phía webview cho Chế độ đọc — Story 5.11, FR11 · Story 5.12, FR120.
 *
 * Cùng khuôn `./chapter.ts`: một `invoke`, một `try/catch`, hình dạng BA TRẠNG THÁI
 * `{ run: ReadingRun | null, error: IpcError | null }` — KHÔNG BAO GIỜ ném. Tầng UI
 * (`src/modes/readingState.ts`) hiển thị lỗi bằng `tError()`, không bằng `try/catch`.
 *
 * 🔵 **SỬA 2026-08-30 (Story 5.12)** — bề mặt trước đọc MỘT Chương đang mở bất kể trạng thái
 * (`readReadingChapter` → `ReadingChapter`). Nay nó đọc MỘT LƯỢT ĐỌC (`readReadingRun` →
 * `ReadingRun`): một dãy Chương `Done` liên tiếp bắt đầu tại Chương đang mở, kèm một MỐC
 * BIÊN nói vì sao dãy dừng ở đó (FR120). Tên trên dây đổi theo — `read_reading_run`.
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
 *
 * `is_confirmed` — **THÊM Story 5.12**: `true` khi và chỉ khi `segment.status ==
 * SEGMENT_STATUS_CONFIRMED` ở tầng Rust. Trang đọc gạch chấm nhẹ câu `false` (AC6) — dấu
 * hiệu đến từ dây, không phải một phép đoán ở webview.
 */
export type ReadingSegment = {
  id: number
  source_text: string
  target_text: string
  is_confirmed: boolean
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
 * Một Chương trong dãy đọc, đã gom đoạn — khớp `commands::segment::ReadingChapter`,
 * `snake_case` trên dây.
 *
 * `segment_count` — **THÊM Story 5.12**: số hàng `segment` còn sống của Chương này, đếm ở
 * Rust trong CÙNG lượt đọc. Đây là dữ kiện GỠ nhánh `'empty-unknown'` cũ (xem
 * `readingState.ts`): `paragraphs.length === 0` cùng `segment_count === 0` ⇒ Chương thật sự
 * rỗng; `paragraphs.length === 0` cùng `segment_count > 0` ⇒ mọi câu đã cắt bỏ.
 */
export type ReadingChapter = {
  chapter_id: number
  chapter_ord: number
  /** `null` ⇒ Chương chưa đặt tên — cùng ngữ nghĩa `ChapterRow::title` (Story 5.7). */
  chapter_title: string | null
  paragraphs: ReadingParagraph[]
  segment_count: number
}

/**
 * Vì sao dãy đọc dừng — khớp `commands::segment::ReadingFrontierKind`. Hai chuỗi trên dây
 * ĐÓNG BĂNG, cùng khuôn `ChapterSwitchOutcome`/`ChapterSwitch` phía Rust.
 */
export type ReadingFrontierKind = 'next-not-done' | 'end-of-work'

/**
 * Chương đứng CHẶN dãy đọc — khớp `commands::segment::ReadingFrontierChapter`.
 *
 * ⚠️ `status` là CHUỖI THÔ trên `chapter.status` — có thể mang một giá trị NGOÀI bốn giá trị
 * của `lifecycle.*` (§I/O Matrix "Trạng thái lạ"). Tầng UI tra nhãn qua `t('lifecycle.' +
 * status)`; không khớp thì hiện nguyên văn chuỗi thô — không đoán, không rơi vào một nhãn
 * nào đó.
 */
export type ReadingFrontierChapter = {
  chapter_id: number
  chapter_ord: number
  chapter_title: string | null
  status: string
}

/**
 * Mốc biên giới của một lượt đọc — khớp `commands::segment::ReadingFrontier`.
 *
 * 🔴 `chapter` là `Some` (khác `null`) **khi và chỉ khi** `kind === 'next-not-done'`.
 */
export type ReadingFrontier = {
  kind: ReadingFrontierKind
  chapter: ReadingFrontierChapter | null
}

/** Một lượt đọc trọn vẹn — khớp `commands::segment::ReadingRun`. */
export type ReadingRun = {
  chapters: ReadingChapter[]
  frontier: ReadingFrontier
}

/** Ba trạng thái cho [`readReadingRun`], cùng khuôn `ReadOpenChapterResult`. */
export type ReadReadingRunResult = {
  run: ReadingRun | null
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
  return (
    typeof v.id === 'number' &&
    typeof v.source_text === 'string' &&
    typeof v.target_text === 'string' &&
    typeof v.is_confirmed === 'boolean'
  )
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

/** Vị từ kiểm kiểu LÚC CHẠY cho `ReadingChapter` — kiểm MỌI trường, MỌI phần tử. */
function isReadingChapter(value: unknown): value is ReadingChapter {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ReadingChapter>
  return (
    typeof v.chapter_id === 'number' &&
    typeof v.chapter_ord === 'number' &&
    (typeof v.chapter_title === 'string' || v.chapter_title === null) &&
    isReadingParagraphArray(v.paragraphs) &&
    typeof v.segment_count === 'number'
  )
}

function isReadingChapterArray(value: unknown): value is ReadingChapter[] {
  return Array.isArray(value) && value.every(isReadingChapter)
}

/** Chốt ĐÚNG hai chuỗi trên dây — một biến thể thứ ba là một hình dạng chưa ai ký. */
function isReadingFrontierKind(value: unknown): value is ReadingFrontierKind {
  return value === 'next-not-done' || value === 'end-of-work'
}

function isReadingFrontierChapter(value: unknown): value is ReadingFrontierChapter {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ReadingFrontierChapter>
  return (
    typeof v.chapter_id === 'number' &&
    typeof v.chapter_ord === 'number' &&
    (typeof v.chapter_title === 'string' || v.chapter_title === null) &&
    typeof v.status === 'string'
  )
}

/**
 * Kiểm CẢ bất biến *"`chapter` non-null khi và chỉ khi `kind === 'next-not-done'`"* — không
 * chỉ hình dạng từng trường. Một `run` vi phạm bất biến này bị TỪ CHỐI toàn bộ, không được
 * vá tại chỗ: một cài đặt "vá cho khớp" sẽ giấu đúng lỗi mà hợp đồng này tồn tại để bắt.
 */
function isReadingFrontier(value: unknown): value is ReadingFrontier {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ReadingFrontier>
  if (!isReadingFrontierKind(v.kind)) return false
  if (v.kind === 'next-not-done') return isReadingFrontierChapter(v.chapter)
  return v.chapter === null
}

/** Vị từ kiểm kiểu LÚC CHẠY cho toàn bộ `ReadingRun` — kiểm MỌI trường, MỌI phần tử. */
function isReadingRun(value: unknown): value is ReadingRun {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<ReadingRun>
  return isReadingChapterArray(v.chapters) && isReadingFrontier(v.frontier)
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
const CMD_READ_READING_RUN = 'read_reading_run'

/**
 * Đọc lượt đọc bắt đầu tại Chương đang mở. Không ném — cùng lý do và cùng khuôn
 * [`./chapter.ts::readOpenChapter`].
 */
export async function readReadingRun(): Promise<ReadReadingRunResult> {
  try {
    const raw = await invoke<unknown>(CMD_READ_READING_RUN)
    if (!isReadingRun(raw)) {
      console.error(`[reading] \`${CMD_READ_READING_RUN}\` trả một hình dạng không phải ReadingRun: ${String(raw)}`)
      return { run: null, error: UNKNOWN_IPC_ERROR }
    }
    return { run: raw, error: null }
  } catch (err) {
    if (isIpcError(err)) return { run: null, error: err }

    // 🔴 CÓ cầu IPC mà vẫn trượt bằng một thứ không phải `IpcError` ⇒ lỗi THẬT, KHÔNG
    // phải "chạy ngoài Tauri" — cùng bài học đã ghi ở `./chapter.ts::readOpenChapter`.
    if (hasIpcBridge()) {
      console.error(`[reading] \`${CMD_READ_READING_RUN}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`)
      return { run: null, error: UNKNOWN_IPC_ERROR }
    }

    // Không có cầu IPC — `npm run dev` trong một trình duyệt thường. Không phải một lỗi
    // để hiện lên; trạng thái thứ BA của I/O Matrix.
    console.info(`[reading] không gọi được \`${CMD_READ_READING_RUN}\` — chạy ngoài Tauri? ${String(err)}`)
    return { run: null, error: null }
  }
}
