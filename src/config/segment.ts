/**
 * Adapter IPC phía webview của tầng segment — lệnh **tách tường minh** (Story 2.1, AC8 · AC14)
 * và lệnh **nạp segment của Chương đang mở** (Story 2.2, AC13).
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

/**
 * Một hàng `segment` như nó đi trên dây — **`snake_case`**, đúng `commands::segment::ChapterSegment`.
 *
 * ⚠️ `target_text` **chuỗi rỗng** nghĩa là *"chưa dịch"*, không phải một giá trị vắng mặt.
 * Kiểu ở đây là `string`, không `string | null`, và đó là mệnh đề của bước di trú 6
 * (`schema.rs::SEGMENT_TARGET_TEXT_DDL`) chứ không phải một lượt thu hẹp kiểu cho tiện.
 */
export type ChapterSegment = {
  id: number
  ord: number
  source_text: string
  /** Bản dịch. **Chuỗi rỗng = chưa dịch** ⇒ nhánh *không vạch* của năm giá trị vạch lề. */
  target_text: string
  /** Cờ kết đoạn, **đã lưu** lúc nhập. AD-37 cấm suy ra lúc render. */
  is_paragraph_end: boolean
  /** `null` cho mọi segment hôm nay — chưa đường nào cho segment về hưu (Story 2.8). */
  retired_at: string | null
  /**
   * Máy trạng thái AD-31 — **`'draft'` | `'confirmed'`**, và đúng hai (Story 2.5).
   *
   * ⚠️ Kiểu ở đây là `string` **rộng hơn** hai giá trị đó, và đó là có chủ ý: dữ liệu này
   * đi **qua dây**, nên nó là một **lời khai** về thứ Rust vừa gửi, không phải một bảo đảm
   * của trình biên dịch. Một union hai nhánh ở đây sẽ nói dối vào đúng ngày lược đồ mọc
   * thêm một giá trị thứ ba mà webview chưa được dựng lại. Phép thu hẹp làm ở
   * [`isSegmentConfirmed`], nơi nó chạy **lúc chạy**.
   *
   * 🔴 Cưỡng chế giá trị hợp lệ là việc của **tầng Rust** (`commands/segment.rs`), không
   * của chỗ này — AD-1.
   */
  status: string
  /**
   * Câu này đã bị **cắt bỏ khỏi bản dịch** chưa (FR133, Story 2.5c) — bước di trú 8.
   *
   * 🔴 Một **trục độc lập** với [`status`], không phải một giá trị thứ ba của nó (AC2):
   * *"cắt bỏ"* trả lời **thuộc hay không thuộc bản dịch**, `status` trả lời **mức độ hoàn
   * thành**. Một câu đã cắt bỏ **giữ nguyên** cả `status` lẫn `target_text` của nó — nên
   * bỏ cờ là một lượt đổi **một** trường, không một lượt khôi phục.
   *
   * ⚠️ `boolean` chứ không `string`, khác hẳn `status` ngay trên: ở đây danh mục giá trị
   * **đóng ở mức hai** theo đúng kiểu của cột (`INTEGER NOT NULL DEFAULT 0`), và Rust đã
   * làm phép đổi `0/1 → bool` trước khi gửi. Không có giá trị thứ ba để một union nói dối.
   */
  is_omitted: boolean
  /**
   * Cờ **kết đoạn của BẢN DỊCH** (FR134/AD-46, Story 2.5d) — bước di trú 9.
   *
   * 🔴 **Một cờ THỨ HAI, không một cách đọc khác của [`is_paragraph_end`]:** nhịp của tiếng
   * Việt không buộc phải là nhịp của bản gốc. Lúc một Chương được nhập, hai cờ **bằng
   * nhau** (AC2 — *"bản dịch soi gương bản gốc cho tới khi người dùng đổi"*); từ đó chúng
   * sống độc lập.
   *
   * 🔴 **AC4 — đường mã nào cần cấu trúc đoạn của bản dịch thì ĐỌC TRƯỜNG NÀY.** Không suy
   * từ `is_paragraph_end`, không suy từ nội dung nguồn, và **không** suy từ vị trí các `\n`
   * trong `target_text`. Ba phép suy đó đều chạy được và đều **rẽ khỏi đĩa** đúng vào ngày
   * người dùng đổi cờ đầu tiên — một nguồn sự thật thứ hai, im lặng.
   *
   * ⚠️ Đừng nhầm nó với `\n` **trong** `target_text`: `\n` là **xuống dòng bên trong một
   * câu** (AC1, người dùng gõ ra); cờ này là **ranh giới đoạn sau câu** (dữ liệu đã lưu).
   * Hai khái niệm khác nhau, và AD-46 giữ chúng tách rời có chủ ý.
   */
  is_target_paragraph_end: boolean
}

/**
 * Segment này đã được người dùng xác nhận chưa — phép thu hẹp **lúc chạy** cho một trường
 * đi qua dây. Story 2.5, AC10 · AC12.
 *
 * 🔴 Vì sao một hàm chứ không một phép so viết thẳng ở chỗ đọc: `status` là `string`, nên
 * `segment.status === 'confirmed'` rải ra nhiều chỗ là nhiều chỗ phải gõ đúng một chuỗi. Một
 * chỗ sai chính tả cho `false` **im lặng** — vạch không đổi màu, không lỗi nào được ném, và
 * triệu chứng là *"xác nhận không ăn"* mà không ai lần được về dòng nào.
 *
 * ⚠️ Đây **không** phải một quy tắc nghiệp vụ (AD-1 cấm chúng ở TypeScript): nó không quyết
 * định trạng thái nào hợp lệ hay chuyển tiếp nào được phép — nó **đọc** một giá trị Rust đã
 * quyết, đúng vai *"Vue chỉ render vạch"*.
 */
export function isSegmentConfirmed(segment: ChapterSegment): boolean {
  return segment.status === 'confirmed'
}

/** Trọn bộ segment của Chương đang mở, kèm `chapter_id` của chính nó. */
export type ChapterSegments = {
  chapter_id: number
  segments: ChapterSegment[]
}

/** Ba trạng thái, cùng khuôn `SplitChapterResult`. */
export type ReadChapterSegmentsResult = {
  loaded: ChapterSegments | null
  error: IpcError | null
}

/** Một mục của lô ghi bản dịch — **`snake_case`**, đúng `commands::segment::SegmentTargetEdit`. */
export type SegmentTargetEdit = {
  /**
   * 🔴 Khoá theo `segment.id`, **KHÔNG** theo `ord`. Story 2.8 sắp lại `ord` mà giữ nguyên
   * `id` (AD-3), nên một lô khoá theo `ord` sẽ ghi bản dịch vào câu khác sau lượt sắp lại.
   */
  id: number
  target_text: string
}

/** Kết quả một lượt flush — **`snake_case`**, đúng `commands::segment::SaveOutcome`. */
export type SaveOutcome = {
  chapter_id: number
  /** Số hàng thật sự được `UPDATE`. **0 là hợp lệ** — một lô rỗng. */
  saved: number
}

/** Ba trạng thái, cùng khuôn `SplitChapterResult`. */
export type SaveSegmentTargetsResult = {
  outcome: SaveOutcome | null
  error: IpcError | null
}

/**
 * Kết quả một lượt xác nhận — **`snake_case`**, đúng `commands::segment::ConfirmOutcome`.
 * Story 2.5.
 */
export type ConfirmOutcome = {
  segment_id: number
  /** Trạng thái **sau** lượt gọi. Luôn `'confirmed'` khi không có lỗi. */
  status: string
  /**
   * Lượt gọi này có **chuyển tiếp** hay không. `false` ⇒ segment đã ở đích từ trước (AC13),
   * và **không** `SegmentVersion` nào được sinh.
   *
   * ⚠️ Không suy ra được từ `status` — hai lượt cùng cho `'confirmed'` khác nhau ở đúng
   * trường này, và Story 2.7 (xuất xứ) cùng Epic 7 (cặp TM) móc vào **chuyển tiếp**.
   */
  version_created: boolean
}

/** Ba trạng thái, cùng khuôn `SplitChapterResult`. */
export type ConfirmSegmentResult = {
  outcome: ConfirmOutcome | null
  error: IpcError | null
}

/**
 * Kết quả một lượt **cắt bỏ / bỏ cờ** — **`snake_case`**, đúng `commands::segment::OmitOutcome`.
 * Story 2.5c · FR133.
 *
 * ⚠️ Không có trường kiểu `version_created` ở đây, và đó là một mệnh đề chứ không một lượt bỏ
 * sót: cắt bỏ **không phải** một chuyển tiếp AD-31, nên không có sự kiện nào để phân biệt
 * *"vừa cắt"* với *"đã cắt từ trước"*. Trạng thái mới nói đủ.
 */
export type OmitOutcome = {
  segment_id: number
  /** Trạng thái **sau** lượt gọi — không phải trạng thái trước. */
  is_omitted: boolean
}

/** Ba trạng thái, cùng khuôn `ConfirmSegmentResult`. */
export type SetSegmentOmittedResult = {
  outcome: OmitOutcome | null
  error: IpcError | null
}

/**
 * Kết quả một lượt đặt **cờ kết đoạn của bản dịch** — **`snake_case`**, đúng
 * `commands::segment::ParagraphEndOutcome`. Story 2.5d · FR134 · AD-46.
 *
 * ⚠️ Cùng hình dạng và cùng lý do với [`OmitOutcome`]: trạng thái **sau** lượt gọi, không
 * một cờ *"vừa đổi"* — đổi cờ đoạn không phải một chuyển tiếp AD-31.
 */
export type ParagraphEndOutcome = {
  segment_id: number
  /** Trạng thái **sau** lượt gọi. */
  is_target_paragraph_end: boolean
}

/** Ba trạng thái, cùng khuôn `SetSegmentOmittedResult`. */
export type SetSegmentParagraphEndResult = {
  outcome: ParagraphEndOutcome | null
  error: IpcError | null
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/segment.rs` (module `wire`). */
const CMD_SPLIT_CHAPTER = 'split_chapter_into_segments'
/** Tên command trên dây — **không tham số nào**, xem doc-comment của hàm thuần phía Rust. */
const CMD_READ_SEGMENTS = 'read_open_chapter_segments'
/** Tên command trên dây — đường **flush** của AD-35, Story 2.3. */
const CMD_SAVE_TARGETS = 'save_segment_targets'
/** Tên command trên dây — máy trạng thái AD-31, Story 2.5. */
const CMD_CONFIRM_SEGMENT = 'confirm_segment'
/**
 * Tên command trên dây — cờ **cắt bỏ** của FR133, Story 2.5c.
 *
 * ⚠️ **MỘT** tên trên dây cho **HAI** lệnh của `CommandRegistry` (`editor.omit_segment` ·
 * `editor.restore_segment`) — chúng khác nhau ở đúng tham số `omitted`. Lý do đầy đủ ở
 * doc-comment của `wire::set_segment_omitted` phía Rust.
 */
const CMD_SET_SEGMENT_OMITTED = 'set_segment_omitted'
/**
 * Tên command trên dây — cờ **kết đoạn của bản dịch** (FR134/AD-46), Story 2.5d.
 *
 * ⚠️ Cùng khuôn `CMD_SET_SEGMENT_OMITTED` ngay trên: **MỘT** tên trên dây cho **HAI** lệnh
 * của `CommandRegistry` (`editor.end_target_paragraph` · `editor.join_target_paragraph`) —
 * chúng khác nhau ở đúng tham số `endsParagraph`.
 * 🔴 Hai lệnh chứ không một lệnh bập bênh: Quyết định #3 của Story 2.5c đã bác hình dạng
 * bập bênh, và lý do vẫn đứng nguyên ở đây — *"nhãn của một phím bập bênh không nói được nó
 * sắp làm gì"*.
 */
const CMD_SET_SEGMENT_PARAGRAPH_END = 'set_segment_paragraph_end'

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

/**
 * Nạp trọn bộ segment của Chương **đang mở** — Story 2.2, AC13.
 *
 * Không ném, cùng lý do `splitChapterIntoSegments` không ném.
 *
 * ⚠️ **Không tham số nào đi trên dây.** Lệnh tự phân giải Chương đang mở phía Rust; một
 * `chapter_id` ở đây sẽ bắt webview gọi `read_open_chapter` trước chỉ để lấy một số nguyên,
 * và lượt gọi đó kéo theo **nguyên khối** `source_text` của cả Chương. Biến thể nhận
 * `chapter_id` thuộc **Story 2.11** (chuyển Chương trong Workspace).
 *
 * ⚠️ Danh sách **rỗng** là một kết quả HỢP LỆ, không phải một lỗi: 25 Chương của Epic 1 chưa
 * ai bấm lệnh tách. Chỗ gọi phân biệt *"rỗng"* với *"lỗi"* bằng hai trường của kết quả này,
 * không bằng độ dài mảng.
 */
export async function readOpenChapterSegments(): Promise<ReadChapterSegmentsResult> {
  try {
    const loaded = await invoke<ChapterSegments>(CMD_READ_SEGMENTS)
    return { loaded, error: null }
  } catch (err) {
    if (isIpcError(err)) return { loaded: null, error: err }

    // 🔴 Cùng ba nhánh và cùng lý do với `splitChapterIntoSegments` — xem chú thích ở đó.
    if (hasIpcBridge()) {
      console.error(
        `[segment] \`${CMD_READ_SEGMENTS}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { loaded: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(`[segment] không gọi được \`${CMD_READ_SEGMENTS}\` — chạy ngoài Tauri? ${String(err)}`)
    return { loaded: null, error: null }
  }
}

/**
 * Ghi bản dịch cho **một LÔ** segment của một Chương — đường **flush** của AD-35, Story 2.3.
 *
 * Không ném, cùng lý do `readOpenChapterSegments` không ném: chỗ gọi hiển thị lỗi bằng
 * `tError()`, không bằng `try/catch` ở tầng UI.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MỘT LƯỢT GỌI CHO CẢ LÔ — KHÔNG một lượt `invoke` mỗi câu (AC13)
 * ─────────────────────────────────────────────────────────────────────────────
 * Một nhịp flush 2 giây có thể mang **nhiều** segment đã đổi *(gõ xuyên qua ba câu trong 5
 * giây là chuyện thường)*. N lượt `invoke` cho N **giao dịch** trên writer **duy nhất, nối
 * tiếp** của AD-11, và `Store::write` phía Rust **chặn** — tức N lượt xếp hàng. Story 2.2 đo
 * trên đúng đường đó: riêng chi phí **parse** đáng 57–64 ms cho 9.850 hàng.
 *
 * ⚠️ Rust **từ chối TRỌN lô** nếu một `id` nào không thuộc `chapterId`
 * (`segment.unknown_ids`) — không ghi một phần. Chỗ gọi vì thế được phép coi kết quả là
 * *"tất cả hoặc không gì cả"*, và `editorFlush.onFlushed()` chỉ được gọi khi `outcome`
 * khác `null`.
 *
 * ⚠️ `invoke()` gửi tham số ở dạng **camelCase**: `chapter_id` phía Rust đi trên dây dưới tên
 * `chapterId`. Đây là chỗ duy nhất trong kho gõ cái tên đó cho lệnh này.
 */
export async function saveSegmentTargets(
  chapterId: number,
  edits: readonly SegmentTargetEdit[],
): Promise<SaveSegmentTargetsResult> {
  try {
    const outcome = await invoke<SaveOutcome>(CMD_SAVE_TARGETS, { chapterId, edits })
    return { outcome, error: null }
  } catch (err) {
    if (isIpcError(err)) return { outcome: null, error: err }

    // 🔴 Cùng ba nhánh và cùng lý do với `splitChapterIntoSegments` — xem chú thích ở đó.
    if (hasIpcBridge()) {
      console.error(
        `[segment] \`${CMD_SAVE_TARGETS}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { outcome: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(`[segment] không gọi được \`${CMD_SAVE_TARGETS}\` — chạy ngoài Tauri? ${String(err)}`)
    return { outcome: null, error: null }
  }
}

/**
 * **Xác nhận một segment** — máy trạng thái AD-31, Story 2.5 · FR24.
 *
 * Không ném, cùng lý do `saveSegmentTargets` không ném: chỗ gọi hiển thị lỗi bằng
 * `tError()`, không bằng `try/catch` ở tầng UI.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 CHỖ GỌI PHẢI FLUSH **XONG** TRƯỚC KHI GỌI HÀM NÀY — AD-35 vế (c)
 * ─────────────────────────────────────────────────────────────────────────────
 * Lệnh này chỉ đọc thứ **đã ở trên đĩa**; nó không biết gì về văn bản đang gõ trong webview.
 * Gọi nó trước khi lượt flush xong sẽ ký một văn bản **cũ hơn** thứ người dùng đang nhìn —
 * và `SegmentVersion` sinh ra mang đúng văn bản cũ đó, tức FR101 khôi phục về một thứ chưa
 * bao giờ tồn tại trên màn hình.
 *
 * ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện:** mệnh đề trên **không**
 * cưỡng chế được ở tầng Rust — xem doc-comment của `wire::confirm_segment`. Lưới duy nhất là
 * `tests/frontend/editorConfirmSegment.test.ts`, và nó canh **chỗ gọi trong
 * `src/commands/index.ts`**, không canh mọi chỗ gọi tương lai.
 * 🔵 *(Code review 2026-08-14: sửa đường dẫn — bản cũ trỏ vào
 * `tests/frontend/commands/confirmSegment.test.ts`, một tệp KHÔNG tồn tại, và nó bịa ra thư
 * mục con `commands/` mà cây test cố ý không dùng.)*
 *
 * ⚠️ `invoke()` gửi tham số ở dạng **camelCase**: `segment_id` phía Rust đi trên dây dưới tên
 * `segmentId`. Đây là chỗ duy nhất trong kho gõ cái tên đó.
 *
 * ⚠️ Bốn lối từ chối đều **phân biệt được** bằng `message_key` (AC14) — `err.segment.*`:
 * `not_found` · `retired` · `nothing_to_confirm`, cộng `err.project.no_work_open`. Chỗ gọi
 * **không** được đoán lại lý do từ chuỗi.
 */
export async function confirmSegment(segmentId: number): Promise<ConfirmSegmentResult> {
  try {
    const outcome = await invoke<ConfirmOutcome>(CMD_CONFIRM_SEGMENT, { segmentId })
    return { outcome, error: null }
  } catch (err) {
    if (isIpcError(err)) return { outcome: null, error: err }

    // 🔴 Cùng ba nhánh và cùng lý do với `splitChapterIntoSegments` — xem chú thích ở đó.
    if (hasIpcBridge()) {
      console.error(
        `[segment] \`${CMD_CONFIRM_SEGMENT}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { outcome: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(
      `[segment] không gọi được \`${CMD_CONFIRM_SEGMENT}\` — chạy ngoài Tauri? ${String(err)}`,
    )
    return { outcome: null, error: null }
  }
}

/**
 * **Cắt bỏ một câu khỏi bản dịch, hoặc bỏ cờ đó** — FR133, Story 2.5c · AC1 · AC4.
 *
 * Không ném, cùng lý do `confirmSegment` không ném: chỗ gọi hiển thị lỗi bằng `tError()`,
 * không bằng `try/catch` ở tầng UI.
 *
 * ⚠️ `invoke()` gửi tham số ở dạng **camelCase**: `segment_id` phía Rust đi trên dây dưới tên
 * `segmentId`. `omitted` là một từ đơn nên hai dạng trùng nhau — đừng đọc sự trùng đó thành
 * *"dây không đổi hoa thường"*.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHÔNG PHẢI FLUSH TRƯỚC, khác hẳn `confirmSegment` ngay trên
 * ─────────────────────────────────────────────────────────────────────────────
 * `confirmSegment` **ký** văn bản đang có trên đĩa, nên gọi nó trước lúc flush xong sẽ ký
 * một bản cũ. Lệnh này không đọc `target_text` một chữ nào — nó đổi **đúng một cột boolean**
 * và **không chạm** `target_text` lẫn `updated_at`. ⇒ Một lượt gõ đang chờ flush vẫn bay
 * bình thường sau đó và ghi đè đúng chỗ của nó.
 *
 * ⚠️ Ba lối từ chối đều **phân biệt được** bằng `message_key`: `err.segment.not_found` ·
 * `err.segment.retired` · `err.project.no_work_open`. Chỗ gọi **không** được đoán lại lý do
 * từ chuỗi.
 */
export async function setSegmentOmitted(
  segmentId: number,
  omitted: boolean,
): Promise<SetSegmentOmittedResult> {
  try {
    const outcome = await invoke<OmitOutcome>(CMD_SET_SEGMENT_OMITTED, { segmentId, omitted })
    return { outcome, error: null }
  } catch (err) {
    if (isIpcError(err)) return { outcome: null, error: err }

    // 🔴 Cùng ba nhánh và cùng lý do với `splitChapterIntoSegments` — xem chú thích ở đó.
    if (hasIpcBridge()) {
      console.error(
        `[segment] \`${CMD_SET_SEGMENT_OMITTED}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { outcome: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(
      `[segment] không gọi được \`${CMD_SET_SEGMENT_OMITTED}\` — chạy ngoài Tauri? ${String(err)}`,
    )
    return { outcome: null, error: null }
  }
}

/**
 * Đặt **cờ kết đoạn của bản dịch** cho một câu — Story 2.5d · FR134 · AD-46.
 *
 * 🔴 **KHÔNG BAO GIỜ NÉM.** Một `invoke`, một `try/catch`, trả hình dạng **ba trạng thái** —
 * cùng luật với mọi adapter ở `src/config/*.ts`.
 *
 * ⚠️ `invoke()` gửi tham số dạng **camelCase** (`segmentId`, `endsParagraph`) dù hàm Rust
 * nhận `snake_case`; chiều **trả về** thì giữ `snake_case`. Hai chiều khác nhau, và đây là
 * chỗ dễ sai nhất trên dây.
 *
 * ⚠️ Ba lối từ chối đều **phân biệt được** bằng `message_key`: `err.segment.not_found` ·
 * `err.segment.retired` · `err.project.no_work_open`. Chỗ gọi **không** được đoán lại lý do
 * từ chuỗi.
 */
export async function setSegmentParagraphEnd(
  segmentId: number,
  endsParagraph: boolean,
): Promise<SetSegmentParagraphEndResult> {
  try {
    const outcome = await invoke<ParagraphEndOutcome>(CMD_SET_SEGMENT_PARAGRAPH_END, {
      segmentId,
      endsParagraph,
    })
    return { outcome, error: null }
  } catch (err) {
    if (isIpcError(err)) return { outcome: null, error: err }

    // 🔴 Cùng ba nhánh và cùng lý do với `setSegmentOmitted` — xem chú thích ở đó.
    if (hasIpcBridge()) {
      console.error(
        `[segment] \`${CMD_SET_SEGMENT_PARAGRAPH_END}\` trượt bằng một lỗi không phải IpcError: ${String(err)}`,
      )
      return { outcome: null, error: UNKNOWN_IPC_ERROR }
    }

    console.info(
      `[segment] không gọi được \`${CMD_SET_SEGMENT_PARAGRAPH_END}\` — chạy ngoài Tauri? ${String(err)}`,
    )
    return { outcome: null, error: null }
  }
}
