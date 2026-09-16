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
  BilingualMismatchWire,
  BilingualRegroupingInput,
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

/**
 * **THÊM Story 6.17 (FR116)** — quy nhóm câu đích của mọi hàng lệch cặp người dùng ĐÃ làm,
 * theo `row_number`. Rust re-validate TOÀN BỘ tập này ở mỗi `refresh()` (rebuild) — một mục
 * không còn khớp hàng nào (đã cặp được, hoặc ảnh chụp cũ) đơn giản không đổi gì (§I/O Matrix
 * "Unaffected regrouping"/"Stale regrouping"). Không lộ ra `readonly()` trực tiếp — mọi tầng
 * ngoài đọc qua `bilingualImportPreviewActiveCuts`/`bilingualImportPreviewMismatches`.
 */
const regroupings = ref<Map<number, BilingualRegroupingInput>>(new Map())
/** Hàng lệch cặp đang lấy tiêu điểm — `null` = hàng ĐẦU của danh sách hiện hành. */
const activeMismatchRow = ref<number | null>(null)
/** Vị trí caret (chỉ số KÝ TỰ UNICODE vào `target_line` của hàng đang lấy tiêu điểm). */
const caretPosition = ref(0)

/**
 * **THÊM Story 6.16b (FR132)** — bộ lọc "cần xem" của tầng tách Chương, TÁI DÙNG con số Rust
 * đã cộng sẵn (`ChapterSplitPreviewWire.needs_review_count`/`.clean_count`/`.any_signal_participated`)
 * qua `BilingualEncodingCandidateWire.chapters` — state HIỂN THỊ THUẦN, 0 lời gọi IPC, cùng
 * khuôn `importPreviewState.ts::chapterFilterActive`.
 */
const chapterFilterActive = ref(false)
/** Con trỏ Chương ĐANG CHỌN trong danh sách tầng tách Chương (0-based) — cùng khuôn
 * `importPreviewState.ts::chapterCursor`. Không có cặp phím `⌥←`/`⌥→` di con trỏ này trên
 * đường song ngữ (ngoài phạm vi story) — trường này chỉ đổi khi bộ lọc BẬT dời nó tới Chương
 * "cần xem" đầu tiên, xem `toggleBilingualImportPreviewChapterFilter`. */
const chapterCursor = ref(0)

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
export const bilingualImportPreviewChapterFilterActive: DeepReadonly<Ref<boolean>> = readonly(chapterFilterActive)
export const bilingualImportPreviewChapterCursor: DeepReadonly<Ref<number>> = readonly(chapterCursor)

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

// ─────────────────────────────────────────────────────────────────────────────
// Story 6.17 (FR116) — quy nhóm câu đích trong từng hàng lệch cặp
// ─────────────────────────────────────────────────────────────────────────────

/** Danh sách hàng lệch cặp của ứng viên ĐANG CHỌN — rỗng khi chưa có preview. */
export const bilingualImportPreviewMismatches = computed<readonly BilingualMismatchWire[]>(() => {
  return bilingualImportPreviewSelectedCandidate.value?.mismatches ?? []
})

/** Hàng ĐANG lấy tiêu điểm — hàng ĐẦU của danh sách khi `activeMismatchRow` không còn khớp
 * hàng nào (đã cặp được ở lượt trước, hoặc chưa từng chọn). `null` khi danh sách rỗng. */
export const bilingualImportPreviewActiveMismatch = computed<BilingualMismatchWire | null>(() => {
  const list = bilingualImportPreviewMismatches.value
  if (list.length === 0) return null
  return list.find((m) => m.row_number === activeMismatchRow.value) ?? list[0]
})

/** Vị trí 0-based của hàng đang lấy tiêu điểm trong danh sách — `-1` khi danh sách rỗng. */
export const bilingualImportPreviewActiveMismatchIndex = computed<number>(() => {
  const active = bilingualImportPreviewActiveMismatch.value
  if (active === null) return -1
  return bilingualImportPreviewMismatches.value.findIndex((m) => m.row_number === active.row_number)
})

export const bilingualImportPreviewCaretPosition: DeepReadonly<Ref<number>> = readonly(caretPosition)

function cutsForRow(mismatch: BilingualMismatchWire): number[] {
  const existing = regroupings.value.get(mismatch.row_number)
  return existing?.kind === 'cuts' ? existing.cuts : mismatch.initial_cuts
}

/** Tập cắt HIỆN HÀNH của hàng đang lấy tiêu điểm — khởi đầu là [`BilingualMismatchWire.initial_cuts`]
 * (đúng cách máy đã tách), đổi khi người dùng bật/tắt một chỗ cắt. */
export const bilingualImportPreviewActiveCuts = computed<number[]>(() => {
  const active = bilingualImportPreviewActiveMismatch.value
  return active === null ? [] : cutsForRow(active)
})

/** "Bỏ qua hàng này" chỉ hiện khi một trong hai phía có 0 câu (§Never: "No skip on a row
 * whose two sides both have at least one sentence"). */
export const bilingualImportPreviewCanSkipActiveRow = computed<boolean>(() => {
  const active = bilingualImportPreviewActiveMismatch.value
  if (active === null) return false
  return active.source_sentences.length === 0 || active.target_line === ''
})

/** Tổng số câu đích sẽ bị BỎ nếu xác nhận NGAY BÂY GIỜ — §I/O Matrix "Skip blank source":
 * "Translation dropped, count shown in preview". Đọc THẲNG từ
 * `BilingualEncodingCandidateWire.skipped_target_sentence_count` — Rust tính lại con số này
 * MỖI LƯỢT chạy trọn chuỗi (cạnh `pair_count`), nên nó SỐNG SÓT qua chính hàng đã biến mất
 * khỏi `mismatches` ngay khi được giải quyết bằng Skip.
 *
 * 🔴 SỬA (vòng rà đối kháng) — bản trước CỘNG DỒN `target_sentence_count` của các hàng còn
 * trong `bilingualImportPreviewMismatches` mà quy nhóm hiện hành là Skip: một hàng Skip hợp
 * lệ biến mất khỏi danh sách đó NGAY sau lượt rebuild kế tiếp, nên số hiện ra rồi rơi về 0
 * đúng lúc lượt xác nhận đang chờ nó — trái §I/O Matrix. Trường trên KHÔNG suy từ danh sách
 * mismatch, nên nó không có lỗ hổng đó. */
export const bilingualImportPreviewSkippedTargetSentenceCount = computed<number>(() => {
  return bilingualImportPreviewSelectedCandidate.value?.skipped_target_sentence_count ?? 0
})

/** Mọi điểm caret có thể dừng trên `target_line` của một hàng — hai đầu cộng
 * `candidate_positions` (Rust cấp, AD-1: không luật kinh doanh nào tính lại ở đây). */
function caretStops(mismatch: BilingualMismatchWire): number[] {
  const length = [...mismatch.target_line].length
  return [0, ...mismatch.candidate_positions, length]
}

function setRegroupingCuts(mismatch: BilingualMismatchWire, cuts: number[]): void {
  regroupings.value.set(mismatch.row_number, {
    row_number: mismatch.row_number,
    source_sentences: mismatch.source_sentences,
    target_line: mismatch.target_line,
    kind: 'cuts',
    cuts,
  })
}

function setRegroupingSkip(mismatch: BilingualMismatchWire): void {
  regroupings.value.set(mismatch.row_number, {
    row_number: mismatch.row_number,
    source_sentences: mismatch.source_sentences,
    target_line: mismatch.target_line,
    kind: 'skip',
    cuts: [],
  })
}

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
  // Chụp TRƯỚC lượt gọi Rust — hàng đang lấy tiêu điểm và `target_line` của nó NGAY LÚC NÀY,
  // để so sánh sau khi preview mới về (xem nhánh đặt lại caret cuối hàm).
  const focusedBefore = bilingualImportPreviewActiveMismatch.value
  const focusedRowNumber = focusedBefore?.row_number ?? null
  const focusedTargetLineBefore = focusedBefore?.target_line ?? null
  const result = await rebuildBilingualImportPreview(
    pendingSourceLang.value,
    chapterPatternWire(),
    sourceColumn.value,
    targetColumn.value,
    hasHeader.value,
    Array.from(regroupings.value.values()),
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

  // Hàng đang lấy tiêu điểm vừa cặp được (biến mất khỏi danh sách) ⇒ caret của nó không còn
  // nghĩa cho hàng KẾ mà `bilingualImportPreviewActiveMismatch` tự rơi về — đặt lại 0.
  const stillFocused = bilingualImportPreviewMismatches.value.find((m) => m.row_number === focusedRowNumber)
  if (stillFocused === undefined) {
    caretPosition.value = 0
    return
  }
  // 🔴 SỬA (vòng rà đối kháng) — hàng VẪN còn (cùng `row_number`) nhưng `target_line` đã đổi
  // (đảo cột, bật/tắt tiêu đề) ⇒ caret cũ trỏ vào một chỉ số KÝ TỰ của một chuỗi KHÁC, không
  // còn nghĩa; `candidate_positions` cũng đã đổi theo, nên bước caret kế tiếp sẽ nhảy lung
  // tung (thường về cuối dòng mới). Đặt lại 0 — cùng lý do hàng biến mất khỏi danh sách.
  if (stillFocused.target_line !== focusedTargetLineBefore) {
    caretPosition.value = 0
  }
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
  regroupings.value = new Map()
  activeMismatchRow.value = null
  caretPosition.value = 0
  chapterFilterActive.value = false
  chapterCursor.value = 0

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

/**
 * **THÊM Story 6.16b (FR132)** — bật/tắt bộ lọc "cần xem" của tầng tách Chương. Handler của
 * `import.preview.bilingual_chapter_filter_toggle` (`⌥W`, cùng phím monolingual đã dùng).
 *
 * TẮT một bộ lọc đang bật LUÔN được phép. BẬT bị chặn khi `needs_review_count === 0` HOẶC
 * `!any_signal_participated` — cùng lý lẽ `importPreviewState.ts::toggleImportPreviewChapterFilter`:
 * `needs_review_count` cộng cả vế link hỏng (`0` trên đường song ngữ, §Always spec 6.16b) NÊN ở
 * đây vế đó không áp, nhưng điều kiện vẫn đọc CẢ hai cờ để không khoá cứng vào giả định "link
 * hỏng luôn 0" của HÔM NAY.
 */
export function toggleBilingualImportPreviewChapterFilter(): void {
  if (!overlayOpen.value) return
  if (chapterFilterActive.value) {
    chapterFilterActive.value = false
    return
  }
  const chapters = bilingualImportPreviewSelectedCandidate.value?.chapters ?? null
  if (chapters === null || chapters.needs_review_count === 0 || !chapters.any_signal_participated) return
  chapterFilterActive.value = true
  // Chương đang chọn (con trỏ) vừa bị lọc khỏi DOM (nó SẠCH) — dời con trỏ tới Chương CẦN XEM
  // đầu tiên, cùng khuôn `importPreviewState.ts::toggleImportPreviewChapterFilter`.
  const currentIsClean =
    chapterCursor.value >= 0 &&
    chapterCursor.value < chapters.chapters.length &&
    !chapters.chapters[chapterCursor.value].needs_review
  if (currentIsClean) {
    const firstNeedsReview = chapters.chapters.findIndex((c) => c.needs_review)
    if (firstNeedsReview !== -1) chapterCursor.value = firstNeedsReview
  }
}

/** Gửi mẫu phân tách Chương mới rồi chạy lại preview — `@change` của ô nhập mẫu (không dispatch
 * — cùng tiền lệ `chapterPatternText`/`@change` của `ImportPreviewOverlay.vue`). */
export async function setBilingualChapterPattern(pattern: string, kind: ChapterPatternKindWire): Promise<void> {
  chapterPatternText.value = pattern
  chapterPatternKind.value = kind
  await refresh()
}

/** Chuyển tiêu điểm sang hàng lệch cặp KẾ TIẾP — Handler của
 * `import.preview.bilingual_next_mismatch` (`↓`). Vòng tròn: hàng cuối → hàng đầu. */
export function moveToNextBilingualMismatch(): void {
  const list = bilingualImportPreviewMismatches.value
  if (list.length === 0) return
  const idx = bilingualImportPreviewActiveMismatchIndex.value
  const from = idx === -1 ? 0 : idx
  activeMismatchRow.value = list[(from + 1) % list.length].row_number
  caretPosition.value = 0
}

/** Xem [`moveToNextBilingualMismatch`] — Handler của `import.preview.bilingual_previous_mismatch`
 * (`↑`). */
export function moveToPreviousBilingualMismatch(): void {
  const list = bilingualImportPreviewMismatches.value
  if (list.length === 0) return
  const idx = bilingualImportPreviewActiveMismatchIndex.value
  const from = idx === -1 ? 0 : idx
  activeMismatchRow.value = list[(from - 1 + list.length) % list.length].row_number
  caretPosition.value = 0
}

/** Di caret một điểm ứng viên — Handler của `import.preview.bilingual_caret_left` (`←`). */
export function moveBilingualCaretLeft(): void {
  const active = bilingualImportPreviewActiveMismatch.value
  if (active === null) return
  const stops = caretStops(active)
  const idx = stops.indexOf(caretPosition.value)
  const from = idx === -1 ? stops.length - 1 : idx
  caretPosition.value = stops[Math.max(0, from - 1)]
}

/** Xem [`moveBilingualCaretLeft`] — Handler của `import.preview.bilingual_caret_right` (`→`). */
export function moveBilingualCaretRight(): void {
  const active = bilingualImportPreviewActiveMismatch.value
  if (active === null) return
  const stops = caretStops(active)
  const idx = stops.indexOf(caretPosition.value)
  const from = idx === -1 ? 0 : idx
  caretPosition.value = stops[Math.min(stops.length - 1, from + 1)]
}

/** Bật/tắt chỗ cắt tại caret — Handler của `import.preview.bilingual_toggle_cut`. 0 lượt gọi
 * Rust nếu caret không đứng trên một điểm ứng viên hợp lệ. */
export async function toggleBilingualCutAtCaret(): Promise<void> {
  const active = bilingualImportPreviewActiveMismatch.value
  if (active === null) return
  const pos = caretPosition.value
  if (!active.candidate_positions.includes(pos)) return
  const current = cutsForRow(active)
  const next = current.includes(pos) ? current.filter((c) => c !== pos) : [...current, pos].sort((a, b) => a - b)
  setRegroupingCuts(active, next)
  await refresh()
}

/** Áp đề xuất máy cho MỌI hàng lệch cặp hiện đang liệt kê, một lượt duy nhất — Handler của
 * `import.preview.bilingual_accept_all_proposals` (§Always: "one command applies every
 * proposal at once"). Hàng chỉ giải quyết được bằng Skip (một phía 0 câu) không nhận đề xuất
 * — nó không có `Cuts` nào để mà đề xuất. */
export async function acceptAllBilingualProposals(): Promise<void> {
  const list = bilingualImportPreviewMismatches.value
  let applied = false
  for (const mismatch of list) {
    if (mismatch.source_sentences.length === 0 || mismatch.target_line === '') continue
    setRegroupingCuts(mismatch, mismatch.proposed_cuts)
    applied = true
  }
  if (applied) await refresh()
}

/** "Bỏ qua hàng này" cho hàng đang lấy tiêu điểm — Handler của
 * `import.preview.bilingual_skip_row`. 0 lượt gọi Rust khi hàng không đủ điều kiện bỏ qua
 * (§Never: "No skip on a row whose two sides both have at least one sentence") — Rust vẫn
 * canh lại mệnh đề này ở phía nó, đây chỉ tránh một lượt IPC vô ích. */
export async function skipActiveBilingualRow(): Promise<void> {
  const active = bilingualImportPreviewActiveMismatch.value
  if (active === null || !bilingualImportPreviewCanSkipActiveRow.value) return
  setRegroupingSkip(active)
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
    Array.from(regroupings.value.values()),
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
  regroupings.value = new Map()
  activeMismatchRow.value = null
  caretPosition.value = 0
  chapterFilterActive.value = false
  chapterCursor.value = 0
}
