/**
 * State của Chế độ đọc — Story 5.11, FR11 · Story 5.12, FR120. Ba nhóm, một tệp: nội dung
 * lượt đọc, mục lục, và ba tuỳ chọn typography.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY, KHÔNG TRONG `ReadingMode.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `editorPanelState.ts`/`sourcePanelState.ts`: ba chế độ sống trong
 * `<KeepAlive>` (`App.vue`), nhưng ngay cả không có `<KeepAlive>` thì đây vẫn là chỗ
 * ĐÚNG — `check:commands` Kiểm C/D/E nạp `src/commands/index.ts` bằng Node thuần, và tệp
 * này dùng `ref` của Vue nên KHÔNG được `import` vào đó. Handler tiêm qua
 * `installCommands()` từ `src/main.ts`, đúng cửa mọi state module khác đã đi qua.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 SỬA 2026-08-30 (Story 5.12) — mệnh đề cũ ở đây hết đúng, xoá luôn nguyên nhân
 * ─────────────────────────────────────────────────────────────────────────────
 * Bản trước (Story 5.11) khai *"`ReadingChapter.paragraphs` rỗng trong CẢ HAI trường hợp
 * — Chương rỗng và mọi câu đã cắt bỏ — nên `ensureReadingLoaded()` phải gọi THÊM
 * `listChapters()` để phân biệt, cộng một nhánh thứ tám `'empty-unknown'` cho ca lượt hỏi
 * phụ đó TRƯỢT"*. Mệnh đề đó hết đúng: `ReadingChapter` (nay là một phần tử của
 * `ReadingRun::chapters`) mang sẵn `segment_count` — đếm trong CHÍNH lượt đọc, không một
 * lượt IPC phụ. Nhánh `'empty-unknown'` không còn lý do tồn tại vì NGUYÊN NHÂN của nó (lượt
 * hỏi phụ có thể trượt) đã biến mất — gỡ cả `chapterHasSegments`, cả lượt `listChapters()`
 * phụ trong `ensureReadingLoaded`, cả nhánh này.
 */
import { computed, readonly, ref, shallowRef } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import { enterFocus } from '../commands'
import { listChapters } from '../config/chapter'
import type { ChapterRow } from '../config/chapter'
import { readReadingRun } from '../config/reading'
import type { ReadingRun } from '../config/reading'
import { openChapterById } from '../panels/editorPanelState'
import { setMode } from './modeState'
import type { IpcError } from '../i18n'
import { tokens } from '../tokens'
import type { TypographyToken } from '../tokens'

// ═════════════════════════════════════════════════════════════════════════════════
// ① NỘI DUNG — lượt đọc hiện tại (Story 5.12: `ReadingRun`, không còn một Chương đơn)
// ═════════════════════════════════════════════════════════════════════════════════

const run = shallowRef<ReadingRun | null>(null)
/** `true` sau khi lượt nạp ĐẦU TIÊN đã trả lời — trước đó là "chưa biết", không "rỗng". */
const hasLoaded = ref(false)
const pending = ref(false)
const loadError = shallowRef<IpcError | null>(null)
let sequence = 0
let requested = false

export const readingRun: DeepReadonly<Ref<ReadingRun | null>> = readonly(run)
export const readingHasLoaded: DeepReadonly<Ref<boolean>> = readonly(hasLoaded)
export const readingPending: DeepReadonly<Ref<boolean>> = readonly(pending)
export const readingLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)

/**
 * Tám nhánh phân biệt được của `role="status"` — **SỬA TẠI CHỖ Story 5.12**: bốn nhánh
 * đầu không đổi tên; bốn nhánh sau đổi hẳn theo hình dạng `ReadingRun` (không còn
 * `'empty-chapter'`/`'empty-unknown'` của một Chương đơn).
 *
 * - `'content'` — CẢ dãy có ít nhất một đoạn ở BẤT KỲ Chương nào trong nó.
 * - `'frontier-only'` — dãy RỖNG (`chapters.length === 0`, §I/O Matrix "Chạm biên ngay":
 *   Chương đang mở chưa `done`) — trang chỉ có mốc biên.
 * - `'all-omitted'` — dãy có Chương, KHÔNG đoạn nào, nhưng tổng `segment_count` toàn dãy
 *   dương — mọi câu của mọi Chương trong dãy đều đã cắt bỏ.
 * - `'empty-chapters'` — dãy có Chương, KHÔNG đoạn nào, tổng `segment_count` toàn dãy
 *   bằng 0 — mọi Chương trong dãy đều thật sự rỗng.
 */
export type ReadingStatusKind =
  | 'not-loaded'
  | 'pending'
  | 'no-work'
  | 'error'
  | 'content'
  | 'frontier-only'
  | 'all-omitted'
  | 'empty-chapters'

export const readingStatusKind: ComputedRef<ReadingStatusKind> = computed(() => {
  if (pending.value) return 'pending'
  if (!hasLoaded.value) return 'not-loaded'
  if (loadError.value !== null) return loadError.value.code === 'work.none_open' ? 'no-work' : 'error'
  const loaded = run.value
  // Phòng hờ: không lỗi mà cũng không lượt đọc là một hình dạng không xảy ra trên sản phẩm
  // thật (adapter chỉ trả `null` VỚI một lỗi, hoặc khi không có cầu IPC — ca đó `hasLoaded`
  // vẫn lên `true` để tránh khoá màn hình, và ta không có gì để nói ngoài "lỗi").
  if (loaded === null) return 'error'
  if (loaded.chapters.length === 0) return 'frontier-only'
  if (loaded.chapters.some((c) => c.paragraphs.length > 0)) return 'content'
  const totalSegments = loaded.chapters.reduce((sum, c) => sum + c.segment_count, 0)
  return totalSegments > 0 ? 'all-omitted' : 'empty-chapters'
})

/**
 * Nạp lượt đọc bắt đầu tại Chương đang mở — **idempotent**: gọi lại ở mỗi lượt
 * `onActivated` chỉ chạy IPC ở lượt ĐẦU TIÊN, đúng khuôn
 * `editorPanelState.ts::ensureSegmentsLoaded`.
 */
export async function ensureReadingLoaded(): Promise<void> {
  if (requested) return
  requested = true

  // Đặt TRƯỚC `await` — một lượt `resetReading()` xen giữa phải vượt mặt được lượt này.
  const mine = ++sequence
  pending.value = true

  const { run: loaded, error } = await readReadingRun()
  if (mine !== sequence) return

  pending.value = false
  run.value = loaded
  loadError.value = error
  hasLoaded.value = true

  // Một lượt TRƯỢT không được khoá vĩnh viễn đường nạp — cùng lý do `ensureSegmentsLoaded`.
  if (loaded === null) requested = false
}

/** Nạp lại CƯỠNG BỨC — dùng sau một lượt mở Chương khác qua mục lục. */
export function reloadReading(): Promise<void> {
  requested = false
  return ensureReadingLoaded()
}

/**
 * Vứt state NỘI DUNG — gọi ở mọi chỗ đang gọi `resetEditorPanel()`/`resetSourcePanel()`
 * khi đổi Tác phẩm/Chương (Task 13 của story 5.11, và [`openFrontierInWorkspace`] của
 * story này). KHÔNG dọn nhóm ③ (typography) — đó là tuỳ chọn ứng dụng, xem
 * `resetReadingPreferences()`.
 */
export function resetReading(): void {
  sequence += 1
  requested = false
  run.value = null
  hasLoaded.value = false
  pending.value = false
  loadError.value = null
}

/**
 * **THÊM Story 5.12.** Đi tiếp từ mốc biên — bấm nút *Dịch tiếp Chương N* trên
 * `.frontier`, handler của lệnh `reading.continue_in_workspace`.
 *
 * `frontier.chapter === null` (mốc `end-of-work`, không Chương nào chặn) ⇒ ghi chẩn đoán
 * và trả — **không ném**: hàm này chạy từ một `dispatch()`, cùng luật "hàm chạy từ một
 * hợp âm/lệnh KHÔNG BAO GIỜ ném" (`src/AGENTS.md`). Nút trên `.frontier` vốn chỉ hiện khi
 * `kind === 'next-not-done'` nên nhánh này không nên tới được từ chuột — nó phòng thủ cho
 * đường bàn phím/lệnh gọi thẳng.
 *
 * `openChapterById` trả `false` (flush chặn, lỗi IPC…) ⇒ **không đổi chế độ**: người dùng
 * ở lại Chế độ đọc để thấy chẩn đoán mà `openChapterById` đã ghi, đúng khuôn
 * `openCurrentChapter` (`libraryChapters.ts`).
 *
 * `true` ⇒ vứt TOÀN BỘ state đọc (`resetReading` + `resetReadingToc`) rồi chuyển
 * Workspace — lượt vào Chế độ đọc kế tiếp phải nạp lại từ con trỏ Chương MỚI, không giữ
 * lại dãy của Chương cũ.
 */
export async function openFrontierInWorkspace(): Promise<void> {
  const frontierChapter = run.value?.frontier.chapter ?? null
  if (frontierChapter === null) {
    console.error('[reading] moc bien khong co Chuong nao de mo sang Workspace (kind = end-of-work?)')
    return
  }

  const moved = await openChapterById(frontierChapter.chapter_id)
  if (!moved) return

  resetReading()
  resetReadingToc()
  setMode('workspace')
}

// ═════════════════════════════════════════════════════════════════════════════════
// ② MỤC LỤC
// ═════════════════════════════════════════════════════════════════════════════════

const tocOpen = ref(false)
const tocChapters = shallowRef<ChapterRow[]>([])
const tocHaveLoaded = ref(false)
const tocBusy = ref(false)
const tocError = shallowRef<IpcError | null>(null)
const tocCursor = ref(0)
let tocSequence = 0

export const readingTocOpen: DeepReadonly<Ref<boolean>> = readonly(tocOpen)
export const readingTocChapters: DeepReadonly<Ref<ChapterRow[]>> = readonly(tocChapters)
export const readingTocHaveLoaded: DeepReadonly<Ref<boolean>> = readonly(tocHaveLoaded)
export const readingTocBusy: DeepReadonly<Ref<boolean>> = readonly(tocBusy)
export const readingTocError: DeepReadonly<Ref<IpcError | null>> = readonly(tocError)
export const readingTocCursor: DeepReadonly<Ref<number>> = readonly(tocCursor)

/**
 * Mở lớp phủ mục lục và tải danh sách Chương — Story 5.11, `Mod+L`.
 *
 * ⚠️ **Vẫn phát lượt IPC kể cả khi chưa mở Tác phẩm nào** — `listChapters()` trả về
 * `err.work.none_open`, và đó chính là câu *"vì sao rỗng"* mà §I/O Matrix đòi (không một
 * `if` chặn trước làm màn hình phải tự bịa lý do).
 */
export async function openTableOfContents(): Promise<void> {
  tocOpen.value = true

  const mine = ++tocSequence
  tocBusy.value = true

  const { chapters, error } = await listChapters()
  if (mine !== tocSequence) return

  tocBusy.value = false
  tocChapters.value = chapters ?? []
  tocError.value = error
  tocHaveLoaded.value = true
  tocCursor.value = 0
}

export function closeTableOfContents(): void {
  tocOpen.value = false
}

export function nextTocChapter(): void {
  if (tocChapters.value.length === 0) return
  tocCursor.value = Math.min(tocCursor.value + 1, tocChapters.value.length - 1)
}

export function prevTocChapter(): void {
  if (tocChapters.value.length === 0) return
  tocCursor.value = Math.max(tocCursor.value - 1, 0)
}

/**
 * Mở Chương ĐANG CHỌN trong mục lục vào Chế độ đọc.
 *
 * 🔴 `openChapterById()` (`editorPanelState.ts`) kết thúc bằng `enterFocus('panel.grid')` —
 * panel đó KHÔNG có trong DOM của Chế độ đọc, nên lượt gọi đó tự ghi một `console.error`
 * vô hại rồi trả `false`. Hàm này CHỦ Ý gọi `enterFocus('mode.reading')` NGAY SAU, ghi đè
 * đúng như doc-comment của `openChapterById` đã dặn cho ca này.
 */
export async function openCurrentTocChapter(): Promise<void> {
  // ⚠️ `.at(...)`, không `[...]` — kiểu TypeScript của chỉ số mảng trần là `ChapterRow`
  // (không `noUncheckedIndexedAccess`), nên phép kiểm `undefined` dưới đây bị ESLint chấm
  // là "không thể xảy ra" nếu viết bằng `[]`. `.at()` khai đúng `T | undefined`.
  const row = tocChapters.value.at(tocCursor.value)
  if (row === undefined) return

  const moved = await openChapterById(row.chapter_id)
  if (!moved) return

  await reloadReading()
  tocOpen.value = false
  enterFocus('mode.reading')
}

/**
 * Vứt state MỤC LỤC — cùng lý lẽ `resetReading()`, phạm vi riêng.
 *
 * 🔴 **Gọi Ở MỌI CHỖ gọi `resetReading()`** (bắt ở lượt rà 2026-08-30 — bản đầu khai hàm này
 * mà **không** một chỗ gọi nào trong sản phẩm). Danh sách Chương là dữ liệu của MỘT Tác phẩm;
 * để nó sống qua một lượt đổi Tác phẩm thì mục lục mở ra vẫn liệt Chương của Tác phẩm CŨ, và
 * cú bấm "Mở" phát một `chapter_id` thuộc kho khác — một `chapter.id` là số nguyên CỤC BỘ
 * trong `project.db` (AD-3), nên nó gần như chắc chắn TRÙNG một Chương có thật của Tác phẩm
 * mới. Không lỗi nào được ném, và người dùng mở nhầm Chương.
 */
export function resetReadingToc(): void {
  tocSequence += 1
  tocOpen.value = false
  tocChapters.value = []
  tocHaveLoaded.value = false
  tocBusy.value = false
  tocError.value = null
  tocCursor.value = 0
}

// ═════════════════════════════════════════════════════════════════════════════════
// ③ TYPOGRAPHY — tuỳ chọn ỨNG DỤNG, không theo Tác phẩm
// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 KHÔNG đưa vào `resetReading()` — xem `resetReadingPreferences()` ở cuối nhóm này.
// Không lưu xuống đĩa ở story này (§Never — món nợ có chủ, `deferred-work.md`).

export type ReadingLevel = 'lg' | 'md' | 'sm'

/** Sàn cứng của giãn dòng — DESIGN.md §Giãn dòng 1.66 là sàn cứng, và `check:tokens` Kiểm
 * E KHÔNG canh được giá trị lúc chạy (chỉ đọc `tokens.json`) — đây là lưới DUY NHẤT. */
export const READING_LINE_HEIGHT_FLOOR = 1.66

/**
 * Dải cỡ chữ của thanh trượt tinh chỉnh. ⚠️ **Hai hằng, không hai chuỗi viết thẳng trong
 * template**: `min`/`max` của `<input type="range">` và phép ghìm ở [`setFontSize`] phải là
 * CÙNG một cặp số — hai bản chép sẽ lệch nhau ở lượt sửa thứ hai, và lúc đó thanh trượt kéo
 * tới một giá trị mà hàm lặng lẽ cắt lại.
 */
export const READING_FONT_SIZE_MIN = 14
export const READING_FONT_SIZE_MAX = 28

const TYPOGRAPHY_KEYS: Record<ReadingLevel, TypographyToken> = {
  lg: 'read-lg',
  md: 'read-md',
  sm: 'read-sm',
}
const MEASURE_KEYS: Record<ReadingLevel, keyof typeof tokens.spacing> = {
  lg: 'read-measure-lg',
  md: 'read-measure-md',
  sm: 'read-measure-sm',
}

const readingLevel = ref<ReadingLevel>('md')
const bilingual = ref(false)
const tunerOpen = ref(false)
/** `null` ⇒ dùng nguyên giá trị của `readingLevel`, chưa ai tinh chỉnh. */
const fontSizeOverride = ref<number | null>(null)
/** `null` ⇒ dùng nguyên giá trị của `readingLevel`. Luôn ≥ [`READING_LINE_HEIGHT_FLOOR`]
 * khi khác `null` — cưỡng chế trong [`setLineHeight`], không ở đây. */
const lineHeightOverride = ref<number | null>(null)

export const currentReadingLevel: DeepReadonly<Ref<ReadingLevel>> = readonly(readingLevel)
export const readingBilingual: DeepReadonly<Ref<boolean>> = readonly(bilingual)
export const readingTunerOpen: DeepReadonly<Ref<boolean>> = readonly(tunerOpen)

/**
 * Đặt mức chữ — bấm một trong ba preset. Xoá MỌI tinh chỉnh đang có: một preset phải mang
 * đúng con số đã ký (AC), không cộng dồn một lượt kéo thanh trượt trước đó.
 */
export function setReadingLevel(level: ReadingLevel): void {
  readingLevel.value = level
  fontSizeOverride.value = null
  lineHeightOverride.value = null
}

export function toggleBilingual(): void {
  bilingual.value = !bilingual.value
}

export function toggleTuner(): void {
  tunerOpen.value = !tunerOpen.value
}

/**
 * 🔴 GHÌM trong dải `[READING_FONT_SIZE_MIN, READING_FONT_SIZE_MAX]` — cùng kỷ luật, cùng lý
 * do với [`setLineHeight`] ngay dưới, và bắt ở lượt rà 2026-08-30. Thuộc tính `min`/`max` của
 * thanh trượt chỉ canh ĐƯỜNG CHUỘT; một lời gọi hàm trực tiếp (bàn đo, một `dispatch` sau
 * này) đi vòng qua nó. Hai cửa cho một dải là một cửa không có khoá.
 */
export function setFontSize(px: number): void {
  fontSizeOverride.value = Math.min(READING_FONT_SIZE_MAX, Math.max(READING_FONT_SIZE_MIN, px))
}

/** 🔴 GHÌM ở [`READING_LINE_HEIGHT_FLOOR`] — kể cả khi giá trị đến từ thanh trượt, gõ số,
 * hay một lời gọi hàm trực tiếp. Đây là sàn LÚC CHẠY, cổng tĩnh không canh được nó. */
export function setLineHeight(value: number): void {
  lineHeightOverride.value = Math.max(READING_LINE_HEIGHT_FLOOR, value)
}

/** Cỡ chữ HIỆU LỰC, dạng số (px) — nguồn cho `:value` của thanh trượt cỡ chữ. */
export const effectiveFontSize: ComputedRef<number> = computed(() => {
  if (fontSizeOverride.value !== null) return fontSizeOverride.value
  return Number.parseFloat(String(tokens.typography[TYPOGRAPHY_KEYS[readingLevel.value]].fontSize))
})

/**
 * Giãn dòng HIỆU LỰC, dạng số — nguồn cho `:value` của thanh trượt giãn dòng.
 *
 * ⚠️ **KHÔNG ghìm sàn lần thứ hai ở đây.** [`setLineHeight`] là chốt DUY NHẤT — hai chốt cho
 * cùng một sàn là hai chỗ phải sửa vào ngày sàn đổi, và một đối chứng ĐỎ (gỡ chốt khỏi
 * `setLineHeight`) sẽ không đỏ nếu còn một bản sao ở đây. Nhánh KHÔNG override (`null`) vẫn an
 * toàn: ba token `read-lg/md/sm` đều ≥ sàn theo thiết kế, và `check:tokens` Kiểm E cưỡng chế
 * điều đó tĩnh trên `tokens.json`.
 */
export const effectiveLineHeight: ComputedRef<number> = computed(
  () =>
    lineHeightOverride.value ??
    Number.parseFloat(String(tokens.typography[TYPOGRAPHY_KEYS[readingLevel.value]].lineHeight)),
)

/**
 * Ba biến CSS mà trang đọc áp — Task 12 của story. `measure` LUÔN LÀ token `ch` của
 * `readingLevel`, không đổi theo `fontSizeOverride`: `1ch` tính theo cỡ chữ của CHÍNH phần
 * tử mang nó (§Design Notes "Vì sao `ch` phải nằm trên CHÍNH phần tử mang cỡ chữ"), nên số
 * ký tự mỗi dòng tự giữ nguyên khi cỡ chữ đổi — không một phép quy đổi px nào ở đây.
 *
 * 🔵 **SỬA 2026-08-30 — thước đọc phải là `width`, KHÔNG `max-width`; tên `maxWidth` cũ nói sai
 * thứ nó là.** `.column` mang `flex: 0 0 auto`, nên với `max-width` nó co về bề rộng **NỘI
 * DUNG** và thước chỉ còn là một cái trần không bao giờ chạm tới.
 *
 * 🔴 **Đo bằng một phép GỠ THẬT trong WKWebView (2026-08-30, mức Cân = 68ch)** — hoàn nguyên
 * đúng một dòng (`width:` → `maxWidth:`) rồi chạy lại chính bàn đo e2e:
 *
 * | bind | bề rộng / `1ch` ở 17,5px | ở 22px |
 * |---|---|---|
 * | `max-width` (bản đầu) | **57,27 ch** | **57,09 ch** |
 * | `width` (hôm nay) | **68,00 ch** | **68,00 ch** |
 *
 * ⇒ Bản đầu đọc ở **57 ký tự mỗi dòng**, không phải 68 mà đặc tả và `tokens.json` khai — một
 * mệnh đề sai mà `check:tokens` không thấy (nó soi `tokens.json`, không soi bố cục) và
 * `happy-dom` cũng không thấy (nó không bố cục). `mockups/reading-mode.html:60` khai
 * `.col{width:68ch}` — **`width`**, đúng thứ phép đo vừa xác nhận.
 */
export const readingStyle: ComputedRef<{ fontSize: string; lineHeight: string; measure: string }> = computed(() => ({
  fontSize: `${effectiveFontSize.value}px`,
  lineHeight: String(effectiveLineHeight.value),
  measure: String(tokens.spacing[MEASURE_KEYS[readingLevel.value]]),
}))

/**
 * Vứt state TYPOGRAPHY — `check:panel-refs` đòi mọi ô nhớ cấp module đi qua một hàm
 * `reset*()`. Sản phẩm KHÔNG có chỗ gọi (đây là tuỳ chọn ứng dụng, không theo Tác phẩm —
 * cùng lý lẽ `resetThemeState()`), hàm tồn tại cho bàn đo/test.
 */
export function resetReadingPreferences(): void {
  readingLevel.value = 'md'
  bilingual.value = false
  tunerOpen.value = false
  fontSizeOverride.value = null
  lineHeightOverride.value = null
}
