/**
 * State của lớp phủ **Xem trước lượt nhập song ngữ** (Story 6.16, FR115, AD-39 · AD-37/46 ·
 * AD-47 ③).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MODULE RIÊNG, KHÔNG BỌC VÀO `importPreviewState.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `importPreviewState.ts` (1685 dòng) dựng cho BA nhánh (dán/tệp/URL) chia sẻ bốn tầng
 * (bảng mã → ranh giới nội dung → luật làm sạch → tách Chương) — `openWith`/`confirmImportPreview`
 * ở đó khoá cứng chữ ký `previewImportEncodingFromText`/`confirmImportWithEncoding` (5 tham
 * số) và dọn ~25 ô tầng 2/3/4 không áp cho đường song ngữ. Đường này KHÔNG có bốn tầng đó —
 * nó có đúng MỘT khối (vai cột + tiêu đề + dải bảng mã + hàng lệch cặp). Thêm một nhánh thứ
 * tư vào state cũ nghĩa là hoặc nới chữ ký `openWith` cho MỌI nhánh (rủi ro trôi ba nhánh cũ),
 * hoặc rẽ nhánh khắp 1685 dòng theo `lastSubmittedFrom === 'bilingual'` — cả hai đắt hơn một
 * module nhỏ, tự khép kín, cùng khuôn tổng thể (`sequence`, `readonly()`, một `reset*()` nuốt
 * TOÀN BỘ state — `check:panel-refs` Kiểm A) nhưng không chạm dây của ba nhánh cũ.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHÔNG BYTE NÀO XUỐNG ĐĨA TRƯỚC KHI XÁC NHẬN — cùng bất biến AC1 của ba nhánh cũ
 * ─────────────────────────────────────────────────────────────────────────────
 * `openBilingualImportPreview` chỉ gọi `previewBilingualImportFromFile` (đọc + table-parse,
 * KHÔNG ghi). `confirmBilingualImportPreview` là lời gọi DUY NHẤT tới `confirmBilingualImport`
 * (chỗ ghi thật, `create_work` phía Rust).
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import {
  confirmBilingualImport,
  previewBilingualImportFromFile,
  rebuildBilingualImportPreview,
} from './config/project'
import type {
  BilingualEncodingCandidateWire,
  BilingualImportEncodingPreview,
  ChapterPatternInput,
  ChapterPatternKindWire,
  CreatedWork,
} from './config/project'
import type { IpcError } from './i18n'

/** Bốn trạng thái, cùng khuôn `ImportPreviewStatus`. */
export type BilingualImportPreviewStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'loaded'

const overlayOpen = ref(false)
const opening = ref(false)
const status = ref<BilingualImportPreviewStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const preview = ref<BilingualImportEncodingPreview | null>(null)
const selectedEncoding = ref<string | null>(null)
const confirming = ref(false)
const confirmError = ref<IpcError | null>(null)

/** Ba tham số nộp gần nhất — lượt xác nhận/dựng lại cần chúng. Không giữ `path`: mọi lượt dựng
 * lại đi qua `rebuildBilingualImportPreview` trên byte Rust đã cất lúc mở, không đọc lại tệp. */
const pendingName = ref('')
const pendingSourceLang = ref('')
const pendingGenre = ref('')

/** Vai cột (0-based) + cờ tiêu đề — tham số MỖI LƯỢT gọi Rust, cùng khuôn `chapterPattern`
 * (không lưu ở Rust giữa hai lượt — xem doc-comment `PipelineInput::bilingual_source_column`
 * phía Rust). Mặc định 0/1/false, đúng cột đầu/cột hai của một tệp hai cột điển hình. */
const sourceColumn = ref(0)
const targetColumn = ref(1)
const hasHeader = ref(false)

/** Mẫu phân tách Chương — cùng cơ chế FR14 mà đường văn xuôi dùng, áp lên CỘT NGUỒN của từng
 * hàng (Rust: `split_bilingual_chapters`). Rỗng ⇒ không mẫu, một Chương duy nhất. */
const chapterPatternText = ref('')
const chapterPatternKind = ref<ChapterPatternKindWire>('literal')

let sequence = 0

export const bilingualImportPreviewIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const bilingualImportPreviewOpening: DeepReadonly<Ref<boolean>> = readonly(opening)
export const bilingualImportPreviewStatus: DeepReadonly<Ref<BilingualImportPreviewStatus>> = readonly(status)
export const bilingualImportPreviewLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const bilingualImportPreview: DeepReadonly<Ref<BilingualImportEncodingPreview | null>> = readonly(preview)
export const bilingualImportPreviewSelectedEncoding: DeepReadonly<Ref<string | null>> = readonly(selectedEncoding)
export const bilingualImportPreviewConfirming: DeepReadonly<Ref<boolean>> = readonly(confirming)
export const bilingualImportPreviewConfirmError: DeepReadonly<Ref<IpcError | null>> = readonly(confirmError)
export const bilingualImportPreviewSourceColumn: DeepReadonly<Ref<number>> = readonly(sourceColumn)
export const bilingualImportPreviewTargetColumn: DeepReadonly<Ref<number>> = readonly(targetColumn)
export const bilingualImportPreviewHasHeader: DeepReadonly<Ref<boolean>> = readonly(hasHeader)
export const bilingualImportPreviewChapterPatternText: DeepReadonly<Ref<string>> = readonly(chapterPatternText)
export const bilingualImportPreviewChapterPatternKind: DeepReadonly<Ref<ChapterPatternKindWire>> =
  readonly(chapterPatternKind)

/** Ứng viên bảng mã ĐANG CHỌN — `null` khi chưa có `preview` hoặc không ứng viên nào khớp
 * `selectedEncoding` (không nên xảy ra trên đường sản phẩm, phòng thủ kiểu). */
export const bilingualImportPreviewSelectedCandidate = computed<BilingualEncodingCandidateWire | null>(() => {
  if (preview.value === null || selectedEncoding.value === null) return null
  return preview.value.candidates.find((c) => c.encoding === selectedEncoding.value) ?? null
})

/** Vị từ GHI — điều kiện DUY NHẤT `confirmBilingualImportPreview` được phép chạy tiếp. Xác
 * nhận bị KHOÁ khi còn bất kỳ hàng lệch cặp nào của ứng viên đang chọn (§Boundaries: "confirm
 * is locked" — Rust-side refusal đứng SAU vị từ này, không THAY nó: xem `create_work`). */
export const bilingualImportPreviewCanConfirm = computed<boolean>(() => {
  const candidate = bilingualImportPreviewSelectedCandidate.value
  return (
    status.value === 'loaded' &&
    !confirming.value &&
    !opening.value &&
    candidate !== null &&
    candidate.mismatches.length === 0
  )
})

function chapterPatternWire(): ChapterPatternInput | null {
  const pattern = chapterPatternText.value
  if (pattern === '') return null
  return { pattern, kind: chapterPatternKind.value }
}

async function refresh(): Promise<void> {
  sequence += 1
  const mySequence = sequence
  // 🔴 SỬA (vòng rà đối kháng) — giữ ứng viên NGƯỜI DÙNG đang chọn qua một lượt dựng lại
  // (đổi cột/tiêu đề/mẫu), cùng khuôn `reloadImportPreviewAfterRuleChange` ở `importPreviewState.ts`.
  // Không chụp trước biến này thì dòng `selectedEncoding.value = result.preview.selected_encoding`
  // luôn GHI ĐÈ lượt chọn tay bằng bảng mã Rust tự dò — xác nhận sẽ giải mã SAI bảng mã.
  const keepEncoding = selectedEncoding.value
  const result = await rebuildBilingualImportPreview(
    pendingSourceLang.value,
    chapterPatternWire(),
    sourceColumn.value,
    targetColumn.value,
    hasHeader.value,
  )
  if (mySequence !== sequence) return // một lượt mở/huỷ/sửa MỚI đã vượt mặt lượt này

  if (result.error !== null) {
    status.value = 'error'
    loadError.value = result.error
    preview.value = null
    selectedEncoding.value = null
    return
  }
  if (result.preview === null) {
    status.value = 'ipc_unavailable'
    loadError.value = null
    preview.value = null
    selectedEncoding.value = null
    return
  }
  preview.value = result.preview
  selectedEncoding.value = result.preview.candidates.some((c) => c.encoding === keepEncoding)
    ? keepEncoding
    : result.preview.selected_encoding
  status.value = 'loaded'
  loadError.value = null
}

/** Mở lớp phủ — nhánh DUY NHẤT của đường song ngữ (§Boundaries: chỉ `.csv`/`.tsv`, người
 * dùng đã chọn CHẾ ĐỘ song ngữ tường minh trước khi tới đây, xem `libraryImport.ts::submitBilingualFilePath`). */
export async function openBilingualImportPreview(
  name: string,
  sourceLang: string,
  genre: string,
  path: string,
): Promise<void> {
  if (opening.value) return
  opening.value = true
  sequence += 1
  const mySequence = sequence

  pendingName.value = name
  pendingSourceLang.value = sourceLang
  pendingGenre.value = genre
  sourceColumn.value = 0
  targetColumn.value = 1
  hasHeader.value = false
  chapterPatternText.value = ''
  chapterPatternKind.value = 'literal'
  confirming.value = false
  confirmError.value = null

  const result = await previewBilingualImportFromFile(path, sourceLang, null, 0, 1, false)
  if (mySequence !== sequence) return
  opening.value = false
  overlayOpen.value = true

  if (result.error !== null) {
    status.value = 'error'
    loadError.value = result.error
    preview.value = null
    selectedEncoding.value = null
    return
  }
  if (result.preview === null) {
    status.value = 'ipc_unavailable'
    loadError.value = null
    preview.value = null
    selectedEncoding.value = null
    return
  }
  preview.value = result.preview
  selectedEncoding.value = result.preview.selected_encoding
  status.value = 'loaded'
  loadError.value = null
}

/** Đổi cột NGUỒN — `@change` của một `<select>`, chạy lại preview TRÊN BYTE ĐÃ CẤT (0 lượt
 * đọc đĩa thêm — §I/O Matrix "Swap columns": "counts rebuild").
 *
 * 🔴 SỬA (vòng rà đối kháng) — nếu cột vừa chọn đang là cột ĐÍCH, ĐẢO hai vai thay vì để
 * hai vai trùng cột: trùng cột nghĩa là mỗi hàng tự ghép với chính nó, hiện 0 hàng lệch cặp
 * và nhập nguyên văn nguồn làm bản dịch. */
export async function setBilingualSourceColumn(column: number): Promise<void> {
  if (sourceColumn.value === column) return
  if (targetColumn.value === column) {
    targetColumn.value = sourceColumn.value
  }
  sourceColumn.value = column
  await refresh()
}

/** Xem [`setBilingualSourceColumn`]. */
export async function setBilingualTargetColumn(column: number): Promise<void> {
  if (targetColumn.value === column) return
  if (sourceColumn.value === column) {
    sourceColumn.value = targetColumn.value
  }
  targetColumn.value = column
  await refresh()
}

/** Đảo vai hai cột — Handler của `import.preview.bilingual_swap_columns`. */
export async function swapBilingualColumns(): Promise<void> {
  const nextSource = targetColumn.value
  const nextTarget = sourceColumn.value
  sourceColumn.value = nextSource
  targetColumn.value = nextTarget
  await refresh()
}

/** Bật/tắt "hàng 1 là tiêu đề" — `@change` của checkbox (§I/O Matrix "Header checkbox on":
 * "rebuilds the preview in memory"). */
export async function setBilingualHasHeader(value: boolean): Promise<void> {
  if (hasHeader.value === value) return
  hasHeader.value = value
  await refresh()
}

/** Đổi ứng viên bảng mã đang chọn — 0 lời gọi Rust (cả năm ứng viên đã chạy TRỌN chuỗi bảy
 * bước lúc mở/refresh, cùng khuôn `importPreviewState.ts`). */
export function selectBilingualEncoding(wireId: string): void {
  if (preview.value === null) return
  if (!preview.value.candidates.some((c) => c.encoding === wireId)) return
  selectedEncoding.value = wireId
}

/** Gửi mẫu phân tách Chương mới rồi chạy lại preview — `@change` của ô nhập mẫu (không dispatch
 * — cùng tiền lệ `chapterPatternText`/`@change` của `ImportPreviewOverlay.vue`). */
export async function setBilingualChapterPattern(pattern: string, kind: ChapterPatternKindWire): Promise<void> {
  chapterPatternText.value = pattern
  chapterPatternKind.value = kind
  await refresh()
}

/** Xác nhận — Handler của `import.preview.bilingual_confirm`. Trả `{created, error}` cùng
 * khuôn `confirmImportPreview`; `main.ts` tiêu thụ kết quả rồi gọi
 * `libraryImport.ts::finishImportSubmission` (dùng CHUNG với ba nhánh cũ — reset panel/nạp
 * lại Chương không khác gì theo nguồn nhập). */
export async function confirmBilingualImportPreview(): Promise<{
  created: CreatedWork | null
  error: IpcError | null
}> {
  if (!bilingualImportPreviewCanConfirm.value || selectedEncoding.value === null) {
    return { created: null, error: null }
  }
  confirming.value = true
  confirmError.value = null
  const mySequence = sequence

  const result = await confirmBilingualImport(
    pendingName.value,
    pendingSourceLang.value,
    pendingGenre.value,
    selectedEncoding.value,
    chapterPatternWire(),
    sourceColumn.value,
    targetColumn.value,
    hasHeader.value,
  )
  if (mySequence !== sequence) return { created: null, error: null }

  confirming.value = false
  if (result.error !== null) {
    confirmError.value = result.error
    return { created: null, error: result.error }
  }

  resetBilingualImportPreview()
  return { created: result.created, error: null }
}

/** Huỷ — Handler của `import.preview.bilingual_cancel`. 0 lượt gọi Rust (cùng khuôn
 * `cancelImportPreview`: "huỷ" là một quyết định TẦNG GIAO DIỆN — không một vỏ Rust riêng). */
export function cancelBilingualImportPreview(): void {
  if (confirming.value) return
  resetBilingualImportPreview()
}

/** Dọn TOÀN BỘ state cấp module — `check:panel-refs` Kiểm A. */
export function resetBilingualImportPreview(): void {
  sequence += 1
  overlayOpen.value = false
  opening.value = false
  status.value = 'unknown'
  loadError.value = null
  preview.value = null
  selectedEncoding.value = null
  confirming.value = false
  confirmError.value = null
  pendingName.value = ''
  pendingSourceLang.value = ''
  pendingGenre.value = ''
  sourceColumn.value = 0
  targetColumn.value = 1
  hasHeader.value = false
  chapterPatternText.value = ''
  chapterPatternKind.value = 'literal'
}
