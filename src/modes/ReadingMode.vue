<script setup lang="ts">
// Chế độ 3/3 — đọc lại bản dịch đã hoàn thành, không có công cụ biên tập. Story 1.6 ·
// AC3 · AC4. Story 5.11 đổ chữ THẬT vào đây: typography ba mức, song ngữ, mục lục. Story
// 5.12 đổi bề mặt từ MỘT Chương thành MỘT LƯỢT ĐỌC (`ReadingRun`, FR120): chỉ Chương
// `Done` mới ra trang, và cuối trang luôn có một mốc biên nói vì sao dãy dừng ở đó.
//
// 🔵 SỬA (2026-08-30, Story 5.11) — khối chú thích cũ tự khai "KHUNG RỖNG có chủ ý" và
// cảnh báo "body chạy ở ui-md, giãn dòng 1.5 — dưới sàn 1.66, không phép kiểm nào canh
// được chỗ đó" đã HẾT ĐÚNG: bề mặt này nay khai token `read-*` của chính nó
// (`readingStyle` của `readingState.ts`), đặt trên CHÍNH phần tử mang cỡ chữ (`.column`)
// — không còn kế thừa `ui-md` của `body`.
//
// 🔴 Không công cụ biên tập nào ở đây (AD-1, §Always của story): không vạch lề segment,
// không nút xác nhận, không panel, không `contenteditable`, không lưới hai cột. Chỉ
// bản dịch — song ngữ (nếu bật) hiện nguyên văn ở LỀ, không chen vào dòng đọc.
//
// 🔴 Cắt bỏ, cấu trúc đoạn, VÀ chọn Chương nào được đọc đều đến từ Rust
// (`read_reading_run`) — không một `v-if="!s.is_omitted"` nào ở đây (không CÓ `is_omitted`
// để mà lọc), và không một đường nào ở tệp này hỏi `chapter.status` để quyết định hiển thị
// hay không (§Always của Story 5.12): trang chỉ render thứ Rust ĐÃ gửi.
import { computed, nextTick, onActivated, onBeforeUnmount, onDeactivated, onMounted, ref, useTemplateRef, watch } from 'vue'
import type { DeepReadonly } from 'vue'
import { declareFocus, dispatch, enterFocus, releaseFocus } from '../commands'
import { hasMessageKey, t, tError } from '../i18n'
import { tokens } from '../tokens'
import { currentTheme } from '../tokens/themeState'
import {
  currentReadingLevel,
  clearReadingAim,
  ensureReadingLoaded,
  readingBilingual,
  readingChaptersDone,
  readingCurrentChapter,
  readingFontCanGrow,
  readingFontCanShrink,
  readingLineCanLoosen,
  readingLineCanTighten,
  readingAnchorSegmentId,
  readingRun,
  readingLoadError,
  readingMarkerError,
  readingMarks,
  readingMarksBusy,
  readingMarksCursor,
  readingMarksError,
  readingMarksHaveLoaded,
  readingMarksClearPending,
  readingMarksOpen,
  readingMarksTotal,
  readingStatusKind,
  readingStyle,
  readingTocBusy,
  readingTocChapters,
  readingTocCursor,
  readingTocError,
  readingTocHaveLoaded,
  readingTocOpen,
  resetReading,
  setReadingAnchor,
  setReadingAim,
  setReadingMarksCursor,
} from './readingState'
import { readingOriginLabel } from './readingOrigin'
import { commandTitle, shortcutRows } from '../config/shortcutsState'
import type { ReadingChapter, ReadingFrontierChapter, ReadingImage, ReadingParagraph, ReadingSegment } from '../config/reading'
import AppIcon from '../AppIcon.vue'
import ChapterImage from '../ChapterImage.vue'
import ReadingLookupPopup from './ReadingLookupPopup.vue'
import { readingLookupAnchor, resetReadingLookup } from './readingLookupState'
import { useSelectionSurface } from '../panels/selectionContract'

const root = useTemplateRef<HTMLElement>('root')
const tocPanel = useTemplateRef<HTMLElement>('tocPanel')
const marksPanel = useTemplateRef<HTMLElement>('marksPanel')

useSelectionSurface(root, 'source', undefined, false, '[data-reading-source-note]', false)

watch(readingBilingual, (on) => {
  if (!on) resetReadingLookup()
})
// The popup keeps content coordinates, so any reflow of the text under it closes it.
watch([readingStyle, readingRun], () => {
  resetReadingLookup()
})
let readingModeActive = false

/**
 * Tiêu điểm đi theo lớp phủ mục lục — 🔵 THÊM ở lượt rà 2026-08-30 (NFR17).
 *
 * ⚠️ `await nextTick()` là bắt buộc ở CẢ HAI chiều: lúc mở, `tocPanel` chưa có trong DOM tại
 * thời điểm `readingTocOpen` đổi (`v-if`); lúc đóng, gốc chế độ mới lại là thứ nhận được
 * tiêu điểm sau khi lớp phủ bị gỡ.
 *
 * 🔴 Không ném, kể cả khi phần tử vắng mặt — hàm này chạy sau một hợp âm bàn phím
 * (`src/AGENTS.md`: *"hàm chạy từ một hợp âm KHÔNG BAO GIỜ ném — nó ghi chẩn đoán rồi trả"*).
 */
watch(readingTocOpen, async (open) => {
  await nextTick()
  if (open) {
    const panel = tocPanel.value
    if (panel === null) {
      console.warn('[reading] muc luc mo ma `tocPanel` khong co trong DOM — bo qua luot dat tieu diem')
      return
    }
    panel.focus()
    return
  }
  if (readingModeActive) void enterFocus('mode.reading')
})

watch(
  readingMarksClearPending,
  (pending) => {
    if (!pending && readingMarksOpen.value && document.activeElement === document.body) marksPanel.value?.focus()
  },
  { flush: 'post' },
)

watch(readingMarksOpen, async (open) => {
  await nextTick()
  if (open) {
    const panel = marksPanel.value
    if (panel === null) {
      console.warn('[reading] danh sach marker mo nhung panel vang mat')
      return
    }
    panel.focus()
    return
  }
  if (readingModeActive) void enterFocus('mode.reading')
})

/** Neo đọc là `segment.id`; pixel chỉ được ĐO lúc dựng DOM, không bao giờ được lưu. */
async function restoreReadingAnchor(): Promise<void> {
  const loaded = readingRun.value
  const anchor = readingAnchorSegmentId.value
  if (loaded === null) return
  await nextTick()
  if (!readingModeActive) return
  const host = root.value
  if (host === null) return
  if (anchor === null) {
    host.scrollTop = 0
    return
  }
  const exact = host.querySelector<HTMLElement>(`[data-reading-segment="${String(anchor)}"]`)
  if (exact !== null) {
    exact.scrollIntoView({ block: 'center', inline: 'nearest' })
    return
  }
  const first = host.querySelector<HTMLElement>('[data-reading-segment]')
  console.warn(`[reading] neo segment ${String(anchor)} khong con tren trang — roi ve cau song dau tien`)
  const fallbackId = Number(first?.dataset.readingSegment)
  if (Number.isSafeInteger(fallbackId)) setReadingAnchor(fallbackId)
  first?.scrollIntoView({ block: 'center', inline: 'nearest' })
}

watch(readingRun, (loaded, previous) => {
  // Chỉ lượt NẠP đầu từ `null` mới cần khôi phục. Aim không cuộn, và lượt `M` thay cây
  // `ReadingRun` để bật `is_marked` cũng không được kéo câu về giữa màn hình.
  if (loaded !== null && previous === null) void restoreReadingAnchor()
})

let bilingualAnchor: { node: HTMLElement; top: number } | null = null

/** Source notes change paragraph heights, so `B` keeps the first visible sentence where it was on screen. */
watch(readingBilingual, () => {
  const host = root.value
  if (host === null) return
  const boundary = host.getBoundingClientRect().top
  const node = [...host.querySelectorAll<HTMLElement>('[data-reading-segment]')].find((el) => el.getBoundingClientRect().bottom > boundary)
  bilingualAnchor = node === undefined ? null : { node, top: node.getBoundingClientRect().top }
}, { flush: 'pre' })

watch(readingBilingual, () => {
  const host = root.value
  if (host !== null && bilingualAnchor !== null) host.scrollTop += bilingualAnchor.node.getBoundingClientRect().top - bilingualAnchor.top
  bilingualAnchor = null
}, { flush: 'post' })

/**
 * Chụp câu gần mép trên vùng đọc nhất NGAY TRƯỚC khi `<KeepAlive>` deactive component.
 * Đây là phép đo hình học nhất thời để chọn một ID; `DOMRect.top` không đi vào state hay
 * xuống đĩa. Không dùng `aimedSegmentId`: cuộn trackpad không trao quyền cho `M` đánh dấu.
 */
function rememberReadingViewportAnchor(): void {
  const host = root.value
  if (host === null) return
  const boundary = host.getBoundingClientRect().top
  let nearest: { id: number; distance: number } | null = null
  let lastLiveId: number | null = null
  for (const node of host.querySelectorAll<HTMLElement>('[data-reading-segment]')) {
    const rawId = node.dataset.readingSegment
    const id = rawId === undefined ? Number.NaN : Number(rawId)
    if (!Number.isSafeInteger(id)) continue
    lastLiveId = id
    const rect = node.getBoundingClientRect()
    if (rect.bottom <= boundary) continue
    const distance = Math.abs(rect.top - boundary)
    if (nearest === null || distance < nearest.distance) nearest = { id, distance }
  }
  // Ở cuối trang, frontier có thể nằm trong viewport trong khi MỌI câu đã trôi lên trên
  // mép host. Khi ấy giữ neo cũ là sai vị trí; câu sống cuối là neo gần nhất còn có nghĩa.
  const captured = nearest?.id ?? lastLiveId
  if (captured !== null) setReadingAnchor(captured)
}

onMounted(() => {
  declareFocus('mode.reading', () => root.value)
})
onBeforeUnmount(() => {
  releaseFocus('mode.reading')
})
onActivated(() => {
  readingModeActive = true
  void enterFocus('mode.reading')
  void ensureReadingLoaded().then(restoreReadingAnchor)
})
onDeactivated(() => {
  rememberReadingViewportAnchor()
  readingModeActive = false
  resetReadingLookup()
  clearReadingAim()
  focusedSegmentId.value = null
  markChipPress.value = null
  // `<KeepAlive>` giữ subtree của BA chế độ sống, nhưng `readingRun` (Chương + đoạn + câu của
  // TOÀN BỘ lượt đọc đang mở) sống
  // ở module-level `readingState.ts`, không theo vòng đời component — không dọn ở đây thì
  // mảng đó nằm lại trong bộ nhớ suốt phiên kể cả khi người dùng đã rời hẳn Chế độ đọc.
  // `preserveAnchor: true` — `rememberReadingViewportAnchor()` ngay trên vừa ghi neo vị trí
  // cuộn, và `resetReading()` mặc định xoá nó; `restoreReadingAnchor()` ở `onActivated` cần
  // giữ nguyên neo đó để cuộn về đúng chỗ khi quay lại. `ensureReadingLoaded()` idempotent
  // qua cờ `requested` (`readingState.ts`) — `resetReading()` hạ cờ đó, nên lượt `onActivated`
  // kế tiếp nạp lại THẬT, không phải một no-op đọc từ bộ nhớ đã vứt.
  resetReading({ preserveAnchor: true })
})

const focusedSegmentId = ref<number | null>(null)
const markChipSideId = ref<number | null>(null)
const MARK_CHIP_GAP_PX = Number.parseFloat(tokens.spacing.unit)

/** Column-relative, so a scroll between the press and the focus does not move the anchor. */
type MarkChipPress = { segmentId: number; x: number; lineTop: number; lineBottom: number }
const markChipPress = ref<MarkChipPress | null>(null)
const markChipAtPress = ref<{ segmentId: number; top: number; left: number } | null>(null)

watch([focusedSegmentId, markChipPress], ([segmentId, press]) => {
  markChipAtPress.value = segmentId !== null && press?.segmentId === segmentId ? placeMarkChipAtPress(press) : null
  markChipSideId.value =
    segmentId !== null && markChipAtPress.value === null && markChipFitsRightMargin(segmentId) ? segmentId : null
}, { flush: 'post' })

function onSegmentTextMouseDown(event: MouseEvent, segmentId: number): void {
  if (event.button !== 0) return
  const text = event.currentTarget as HTMLElement
  const column = text.closest<HTMLElement>('.column')
  if (column === null) return
  const columnBox = column.getBoundingClientRect()
  const line = [...text.getClientRects()].find((rect) => event.clientY >= rect.top && event.clientY <= rect.bottom)
  markChipPress.value = {
    segmentId,
    x: event.clientX - columnBox.left,
    lineTop: (line?.top ?? event.clientY) - columnBox.top,
    lineBottom: (line?.bottom ?? event.clientY) - columnBox.top,
  }
}

/** Below the pressed line, above it when the scroll host would clip the chip; kept inside the column. */
function placeMarkChipAtPress(press: MarkChipPress): { segmentId: number; top: number; left: number } | null {
  const segment = root.value?.querySelector<HTMLElement>(`[data-reading-segment="${String(press.segmentId)}"]`)
  const column = segment?.closest<HTMLElement>('.column')
  const chip = segment?.querySelector<HTMLElement>('.mark-actions')
  const host = root.value
  if (!(column instanceof HTMLElement) || !(chip instanceof HTMLElement) || host === null) return null
  const columnTop = column.getBoundingClientRect().top
  const hostBox = host.getBoundingClientRect()
  const below = press.lineBottom + MARK_CHIP_GAP_PX
  const above = press.lineTop - MARK_CHIP_GAP_PX - chip.offsetHeight
  const clippedBelow = columnTop + below + chip.offsetHeight > hostBox.bottom
  const top = clippedBelow && columnTop + above >= hostBox.top ? above : below
  const left = Math.max(0, Math.min(press.x, column.clientWidth - chip.offsetWidth))
  return { segmentId: press.segmentId, top, left }
}

/** `false` keeps the chip right after the last glyph: the page has no right margin wide enough for it. */
function markChipFitsRightMargin(segmentId: number): boolean {
  const segment = root.value?.querySelector<HTMLElement>(`[data-reading-segment="${String(segmentId)}"]`)
  const column = segment?.closest<HTMLElement>('.column')
  const page = column?.parentElement
  const chip = segment?.querySelector<HTMLElement>('.mark-actions')
  if (!(column instanceof HTMLElement) || !(page instanceof HTMLElement) || !(chip instanceof HTMLElement)) return false
  const gap = parseFloat(getComputedStyle(chip).marginInlineStart)
  return page.getBoundingClientRect().right - column.getBoundingClientRect().right >= chip.offsetWidth + 2 * gap
}

/** A focused sentence's chip is the visible target of `M` and Enter, so the pointer must not move the aim off it. */
function onSegmentMouseEnter(segmentId: number): void {
  if (focusedSegmentId.value === null) setReadingAim(segmentId)
}

function onSegmentFocusIn(segmentId: number): void {
  setReadingAim(segmentId)
  focusedSegmentId.value = segmentId
}

function onSegmentFocusOut(event: FocusEvent, segmentId: number): void {
  const wrapper = event.currentTarget as HTMLElement
  const next = event.relatedTarget
  if (next instanceof Node && wrapper.contains(next)) return
  clearReadingAim(segmentId)
  if (focusedSegmentId.value === segmentId) focusedSegmentId.value = null
  if (markChipPress.value?.segmentId === segmentId) markChipPress.value = null
}

function onSegmentMouseLeave(event: MouseEvent, segmentId: number): void {
  const wrapper = event.currentTarget as HTMLElement
  if (!wrapper.matches(':focus-within')) clearReadingAim(segmentId)
}

/** Giữ Tab trong overlay `aria-modal`; không có dependency/focus utility thứ hai. */
function trapOverlayTab(event: KeyboardEvent, panel: HTMLElement | null): void {
  if (panel === null) return
  const controls = [...panel.querySelectorAll<HTMLElement>('button:not([disabled]), [href], [tabindex]:not([tabindex="-1"])')]
    .filter((node) => !node.hasAttribute('hidden'))
  const first = controls.at(0)
  const last = controls.at(-1)
  if (first === undefined || last === undefined) return
  if (event.shiftKey && (document.activeElement === first || !panel.contains(document.activeElement))) {
    event.preventDefault()
    last.focus()
    return
  }
  if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first.focus()
  }
}

/**
 * `Enter` chỉ thuộc wrapper, không ăn lượt Enter của nút marker bên trong. Viết điều kiện
 * tường minh thay cho modifier `.self` vì `check:layout` cố ý từ chối mọi member global
 * chưa khai; đồng thời chỉ `preventDefault()` SAU khi biết đây đúng là cử chỉ mở câu.
 */
function onReadingSegmentEnter(event: KeyboardEvent): void {
  if (event.target !== event.currentTarget) return
  event.preventDefault()
  void dispatch('reading.open_aimed')
}

/**
 * Câu trạng thái — TÁM nhánh của [`readingStatusKind`]. Một `switch` thay vì một chuỗi
 * ternary dài: mỗi nhánh gọi ĐÚNG MỘT `t()`/`tError()`, không một biểu thức trộn — cùng
 * khuôn mà `check:i18n` Kiểm A2 đòi (xem `<!-- aura-allow-text -->` ngay dưới template,
 * vì bản thân biến này không "bắt đầu bằng `t(`" ở góc nhìn tĩnh của cổng).
 *
 * 🔵 SỬA TẠI CHỖ (2026-08-30, Story 5.12) — bốn nhánh cuối đổi hẳn theo hình dạng
 * `ReadingRun`: không còn `'empty-chapter'`/`'empty-unknown'` của một Chương đơn.
 */
const statusMessage = computed(() => {
  switch (readingStatusKind.value) {
    case 'not-loaded':
    case 'content':
      return ''
    case 'pending':
      return t('mode.reading.status_loading')
    case 'no-work':
      return t('mode.reading.status_no_work')
    case 'error':
      return readingLoadError.value === null ? '' : tError(readingLoadError.value)
    case 'frontier-only':
      return t('mode.reading.status_frontier_only')
    case 'all-omitted':
      return t('mode.reading.status_all_omitted')
    case 'empty-chapters':
      return t('mode.reading.status_empty_chapters')
  }
})

/**
 * Nhãn "Chương {ord}" cho một hàng chưa đặt tên.
 *
 * 🔵 SỬA (rà 2026-08-30) — khoá RIÊNG của miền `mode.reading.*`, không mượn
 * `mode.library.chapter_untitled` nữa. Mượn khoá của miền khác làm câu chữ của màn hình này
 * đổi theo một quyết định biên tập của Library, và văn phạm khoá chấm có tiền tố miền tồn tại
 * đúng để chặn chuyện đó (`check:i18n` Kiểm B).
 */
function untitled(ord: number): string {
  return t('mode.reading.chapter_untitled', { ord: String(ord) })
}

/** Nhãn hiển thị của một Chương — tên thật nếu có, ngược lại nhãn "Chương {ord}". */
function chapterLabel(chapter: { chapter_ord: number; chapter_title: string | null }): string {
  return chapter.chapter_title ?? untitled(chapter.chapter_ord)
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.12 — MỐC BIÊN (`.frontier`)
// ═════════════════════════════════════════════════════════════════════════════════
// Bốn `computed` dưới đây dựng câu chữ của khối `.frontier` — mỗi cái đúng MỘT `t()`,
// cùng kỷ luật `statusMessage`. Tham số mang DỮ LIỆU (tên Chương, nhãn trạng thái đã tra),
// không mang CÂU — định dạng câu là việc của khoá `vi.json`, không việc của đây.

/**
 * Nhãn trạng thái cho mốc biên — `chapter.status` NGOÀI bốn giá trị `LifecycleStatus`
 * (§I/O Matrix "Trạng thái lạ") hiện NGUYÊN VĂN, không tra nhãn.
 *
 * 🔵 SỬA (rà 2026-08-30, Bản vá 7) — bản trước chép tay bốn giá trị vào một `Set` RIÊNG,
 * bản chép THỨ BA của cùng bốn giá trị mà `LifecycleStatus`/`vi.json` đã khai — không cổng
 * nào nối nó với hai đầu kia. Nay hỏi THẲNG `vi.json` qua [`hasMessageKey`], đóng đúng mắt
 * xích còn thiếu (xem doc-comment của hàm đó) thay vì một bản chép thứ tư.
 */
function frontierStatusLabel(status: string): string {
  const key = `lifecycle.${status}`
  return hasMessageKey(key) ? t(key) : status
}

/**
 * Câu nói dãy vừa đọc từ Chương nào tới Chương nào — hoặc dãy rỗng (AC5).
 *
 * 🔵 SỬA (rà 2026-08-30, Bản vá 6) — dãy ĐÚNG MỘT Chương (ca THƯỜNG NHẤT hôm nay: chưa
 * đường sản phẩm nào tạo Chương thứ hai ngoài `split_chapter_at_segment`) đi qua một khoá
 * RIÊNG (`frontier_range_single`) thay vì `frontier_range` — bản trước cho "từ Chương 1 đến
 * Chương 1" khi `first === last`, một câu đúng nghĩa đen nhưng đọc như lỗi lặp từ.
 */
const frontierRangeText = computed(() => {
  const run = readingRun.value
  if (run === null) return ''
  const first = run.chapters.at(0)
  const last = run.chapters.at(-1)
  if (first === undefined || last === undefined) return t('mode.reading.frontier_range_empty')
  if (run.chapters.length === 1) return t('mode.reading.frontier_range_single', { ten: chapterLabel(first) })
  return t('mode.reading.frontier_range', { tu: chapterLabel(first), den: chapterLabel(last) })
})

/** Câu nói Chương nào chặn kèm nhãn trạng thái — chỉ có nghĩa khi `kind === 'next-not-done'`. */
const frontierBlockedText = computed(() => {
  const frontier = readingRun.value?.frontier
  if (frontier === undefined || frontier.kind !== 'next-not-done') return ''
  const blocked: DeepReadonly<ReadingFrontierChapter> | null = frontier.chapter
  if (blocked === null) return ''
  return t('mode.reading.frontier_blocked', {
    title: chapterLabel(blocked),
    status_label: frontierStatusLabel(blocked.status),
  })
})

/** Câu nói hết Tác phẩm — chỉ có nghĩa khi `kind === 'end-of-work'`. */
const frontierEndOfWorkText = computed(() => {
  const frontier = readingRun.value?.frontier
  return frontier !== undefined && frontier.kind === 'end-of-work' ? t('mode.reading.frontier_end_of_work') : ''
})

/** Chữ trên nút *Dịch tiếp Chương N* — chỉ có nghĩa khi `kind === 'next-not-done'`. */
const frontierButtonText = computed(() => {
  const chapter = readingRun.value?.frontier.chapter
  return chapter === null || chapter === undefined ? '' : t('mode.reading.frontier_button', { ord: String(chapter.chapter_ord) })
})

/** Vì sao MỘT Chương của dãy không hiện câu nào — Chương rỗng, hoặc mọi câu đã cắt bỏ. */
function chapterEmptyNote(chapter: DeepReadonly<ReadingChapter>): string {
  return chapter.segment_count > 0 ? t('mode.reading.chapter_all_omitted') : t('mode.reading.chapter_empty')
}

/**
 * Chương của dãy đọc, hay một mảng RỖNG khi chưa nạp — tránh phối `v-if`/`v-for` trên
 * CÙNG một `<template>` (Vue 3 cảnh báo cách đó, và giá trị `null` sẽ ném ngay khi
 * `v-for` cố đọc `.chapters` của nó).
 */
const runChapters = computed<DeepReadonly<ReadingChapter>[]>(() => [...(readingRun.value?.chapters ?? [])])

const chordById = computed(() => {
  const byId = new Map<string, string>()
  for (const row of shortcutRows.value) byId.set(row.id, row.display.join(' / '))
  return byId
})

function chordOf(id: string): string {
  return chordById.value.get(id) ?? ''
}

const themeText = computed(() => (currentTheme.value === 'dark' ? t('mode.reading.theme_dark') : t('mode.reading.theme_light')))

const levelChords = computed(() =>
  [chordOf('reading.level_airy'), chordOf('reading.level_balanced'), chordOf('reading.level_dense')].filter((c) => c !== '').join(' '),
)

/** Null while the done count or the current Chapter is unknown, so the strip is not drawn. */
const progress = computed(() => {
  const done = readingChaptersDone.value
  const current = readingCurrentChapter.value
  if (done === null || current === null) return null
  const percent = done === 0 ? 0 : Math.round((Math.min(current.chapter_ord, done) / done) * 100)
  return { done, ord: current.chapter_ord, title: current.chapter_title, percent }
})

function originText(chapter: DeepReadonly<ReadingChapter>): string {
  return t(`mode.reading.origin_${readingOriginLabel(chapter.translation_origin_counts)}`)
}

const marksWalkText = computed(() => t('mode.reading.marks_walk', { mark_count: String(readingMarks.value.length) }))

const marksClearText = computed(() =>
  readingMarksClearPending.value
    ? t('mode.reading.marks_clear_confirm', { mark_count: String(readingMarks.value.length) })
    : t('mode.reading.marks_clear'),
)

const frontierMarksText = computed(() => t('mode.reading.frontier_marks', { mark_count: String(readingMarksTotal.value ?? 0) }))

function onMarksEscape(): void {
  if (readingMarksClearPending.value) dispatch('reading.marks_clear_cancel')
  else dispatch('reading.marks_close')
}

function onMarksKeydown(event: KeyboardEvent): void {
  if (event.target !== event.currentTarget) return
  if (event.key === 'ArrowDown') dispatch('reading.marks_next')
  else if (event.key === 'ArrowUp') dispatch('reading.marks_prev')
  else if (event.key === 'Enter') dispatch('reading.marks_open')
  else if (event.key === 'Backspace') dispatch('reading.marks_unmark')
  else return
  event.preventDefault()
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔵 THÊM Story 6.14 (FR42 · FR43) — ẢNH ĐÚNG VỊ TRÍ
// ═════════════════════════════════════════════════════════════════════════════════
/** Đường dẫn tuyệt đối tới `assets/` — MỘT lần cho cả lượt đọc (`ReadingRun.assets_dir`). */
const readingAssetsDir = computed<string>(() => readingRun.value?.assets_dir ?? '')

/** Ảnh đứng TRƯỚC đoạn ĐẦU TIÊN của Chương — `after_segment_id === null` (neo `0`, hoặc dời hẳn về đầu Chương). */
function leadingImagesOf(chapter: DeepReadonly<ReadingChapter>): DeepReadonly<ReadingImage>[] {
  return chapter.images.filter((img) => img.after_segment_id === null)
}

/**
 * Một MẢNH của một đoạn — hoặc một dãy câu liên tiếp để render trong một `<p>`, hoặc một ảnh
 * đứng GIỮA hai câu của CÙNG một đoạn. §I/O Matrix spec 6.14: "ảnh giữa hai câu ⇒ hiện SAU
 * câu k ở CẢ HAI bề mặt" — kể cả khi câu k và câu k+1 thuộc cùng một đoạn Chế độ đọc. Một
 * `<figure>` (block) không sống hợp lệ bên trong một `<p>` (inline flow), nên đoạn phải được
 * CHẺ quanh ảnh thành nhiều `<p>` — `paragraphRuns` là nơi DUY NHẤT làm phép chẻ này.
 */
type ParagraphRun =
  | { kind: 'text'; segments: DeepReadonly<ReadingSegment>[] }
  | { kind: 'image'; image: DeepReadonly<ReadingImage> }

/**
 * Chẻ MỘT đoạn quanh các ảnh neo vào câu bên trong nó — xem [`ParagraphRun`]. Đoạn của Rust
 * (`ReadingParagraph`) là MỘT khái niệm ngữ nghĩa (một đoạn văn bản dịch); nó có thể render
 * thành NHIỀU thẻ `<p>` liền kề khi một ảnh chen giữa — điều đó không đổi ranh giới đoạn Rust
 * đã quyết, chỉ đổi số thẻ HTML cần để vẽ nó.
 */
function paragraphRuns(
  chapter: DeepReadonly<ReadingChapter>,
  paragraph: DeepReadonly<ReadingParagraph>,
): ParagraphRun[] {
  const imagesAfter = new Map<number, DeepReadonly<ReadingImage>[]>()
  for (const img of chapter.images) {
    if (img.after_segment_id === null) continue
    const list = imagesAfter.get(img.after_segment_id) ?? []
    list.push(img)
    imagesAfter.set(img.after_segment_id, list)
  }

  const runs: ParagraphRun[] = []
  let current: DeepReadonly<ReadingSegment>[] = []
  for (const segment of paragraph.segments) {
    current.push(segment)
    for (const img of imagesAfter.get(segment.id) ?? []) {
      // 🔴 `current.length > 0` — ảnh THỨ HAI của cùng một neo đóng một mảnh chữ đã RỖNG,
      // và một mảnh rỗng render thành `<p class="paragraph">` không có câu nào: một khoảng
      // trắng dọc giữa hai `<figure>`. Bắt ở vòng rà 2026-09-10.
      if (current.length > 0) runs.push({ kind: 'text', segments: current })
      runs.push({ kind: 'image', image: img })
      current = []
    }
  }
  if (current.length > 0) runs.push({ kind: 'text', segments: current })
  return runs
}
</script>

<template>
  <section ref="root" class="mode" tabindex="-1">
    <div class="toolbar">
      <button
        type="button"
        class="ui-switch"
        role="switch"
        data-reading-toggle-bilingual
        :aria-checked="readingBilingual"
        @click="dispatch('reading.toggle_bilingual')"
      >
        {{ t('mode.reading.bilingual') }}
        <span class="ui-switch__track" aria-hidden="true"></span>
        <!-- aura-allow-text: chord display from the shortcut registry. -->
        <span v-if="chordOf('reading.toggle_bilingual') !== ''" class="ui-kbd">{{ chordOf('reading.toggle_bilingual') }}</span>
      </button>
      <span class="level-group" role="group" :aria-label="t('mode.reading.level_group')">
        <span class="level-label">{{ t('mode.reading.level_group') }}</span>
        <span class="ui-seg level-seg">
          <button
            type="button"
            class="ui-seg__item"
            data-reading-level="lg"
            :aria-pressed="currentReadingLevel === 'lg'"
            @click="dispatch('reading.level_airy')"
          >
            {{ t('mode.reading.level_airy') }}
          </button>
          <button
            type="button"
            class="ui-seg__item"
            data-reading-level="md"
            :aria-pressed="currentReadingLevel === 'md'"
            @click="dispatch('reading.level_balanced')"
          >
            {{ t('mode.reading.level_balanced') }}
          </button>
          <button
            type="button"
            class="ui-seg__item"
            data-reading-level="sm"
            :aria-pressed="currentReadingLevel === 'sm'"
            @click="dispatch('reading.level_dense')"
          >
            {{ t('mode.reading.level_dense') }}
          </button>
        </span>
        <!-- aura-allow-text: chord displays from the shortcut registry. -->
        <span v-if="levelChords !== ''" class="ui-kbd">{{ levelChords }}</span>
      </span>
      <span class="type-group" role="group" :aria-label="t('mode.reading.tuner_font_size')">
        <button
          type="button"
          class="ui-icon-btn"
          data-reading-font-smaller
          :disabled="!readingFontCanShrink"
          :aria-label="t('command.reading.font_smaller')"
          :title="commandTitle('reading.font_smaller')"
          @click="dispatch('reading.font_smaller')"
        >
          <AppIcon name="font-smaller" />
        </button>
        <button
          type="button"
          class="ui-icon-btn"
          data-reading-font-default
          :aria-label="t('command.reading.font_default')"
          :title="commandTitle('reading.font_default')"
          @click="dispatch('reading.font_default')"
        >
          <AppIcon name="font-default" />
        </button>
        <button
          type="button"
          class="ui-icon-btn"
          data-reading-font-larger
          :disabled="!readingFontCanGrow"
          :aria-label="t('command.reading.font_larger')"
          :title="commandTitle('reading.font_larger')"
          @click="dispatch('reading.font_larger')"
        >
          <AppIcon name="font-larger" />
        </button>
      </span>
      <span class="type-group" role="group" :aria-label="t('mode.reading.tuner_line_height')">
        <button
          type="button"
          class="ui-icon-btn"
          data-reading-line-tighter
          :disabled="!readingLineCanTighten"
          :aria-label="t('command.reading.line_tighter')"
          :title="commandTitle('reading.line_tighter')"
          @click="dispatch('reading.line_tighter')"
        >
          <AppIcon name="line-tighter" />
        </button>
        <button
          type="button"
          class="ui-icon-btn"
          data-reading-line-default
          :aria-label="t('command.reading.line_default')"
          :title="commandTitle('reading.line_default')"
          @click="dispatch('reading.line_default')"
        >
          <AppIcon name="line-default" />
        </button>
        <button
          type="button"
          class="ui-icon-btn"
          data-reading-line-looser
          :disabled="!readingLineCanLoosen"
          :aria-label="t('command.reading.line_looser')"
          :title="commandTitle('reading.line_looser')"
          @click="dispatch('reading.line_looser')"
        >
          <AppIcon name="line-looser" />
        </button>
      </span>
      <button
        type="button"
        class="ui-btn ui-btn--quiet"
        data-reading-toggle-theme
        @click="dispatch('reading.toggle_theme')"
      >
        <!-- aura-allow-text: themeText is a computed that goes through t(). -->
        {{ themeText }}
        <!-- aura-allow-text: chord display from the shortcut registry. -->
        <span v-if="chordOf('reading.toggle_theme') !== ''" class="ui-kbd">{{ chordOf('reading.toggle_theme') }}</span>
      </button>
      <button type="button" class="ui-btn ui-btn--quiet" data-reading-toc :aria-expanded="readingTocOpen" @click="dispatch('reading.toc')">
        {{ t('mode.reading.toc') }}
        <!-- aura-allow-text: chord display from the shortcut registry. -->
        <span v-if="chordOf('reading.toc') !== ''" class="ui-kbd">{{ chordOf('reading.toc') }}</span>
      </button>
      <button
        type="button"
        class="ui-btn ui-btn--quiet marks-button"
        data-reading-marks
        :aria-expanded="readingMarksOpen"
        @click="dispatch('reading.marks')"
      >
        <template v-if="readingMarksTotal === null">{{ t('mode.reading.marks_button') }}</template>
        <template v-else>
          {{ t('mode.reading.marks_button_glyph') }}
          <!-- aura-allow-text: the total is a number. -->
          <b>{{ readingMarksTotal }}</b>
          {{ t('mode.reading.marks_button_unit') }}
        </template>
        <!-- aura-allow-text: chord display from the shortcut registry. -->
        <span v-if="chordOf('reading.mark_aimed') !== ''" class="ui-kbd">{{ chordOf('reading.mark_aimed') }}</span>
      </button>
    </div>

    <div v-if="progress !== null" class="progress-line" data-reading-progress>
      <span v-if="progress.title !== null" class="progress-chapter">
        {{ t('mode.reading.position_prefix', { ord: String(progress.ord) }) }}
        <!-- aura-allow-text: the Chapter title is Work data. -->
        <b>{{ progress.title }}</b>
      </span>
      <!-- aura-allow-text: untitled() goes through t(). -->
      <b v-else class="progress-chapter">{{ untitled(progress.ord) }}</b>
      <span class="progress-track" data-reading-progress-bar aria-hidden="true">
        <span class="progress-fill" data-reading-progress-fill :style="{ width: `${String(progress.percent)}%` }"></span>
      </span>
      <span class="progress-count">
        {{ t('mode.reading.progress', { done_count: String(progress.done) }) }}
        <!-- aura-allow-text: the order is a number. -->
        <b>{{ progress.ord }}</b>
      </span>
    </div>

    <!-- aura-allow-text: lỗi marker đi qua tError; đặt cạnh toolbar để một lỗi ghi ở đầu
         Tác phẩm dài không nằm im ngoài viewport tận sau mốc biên. -->
    <p class="marker-error" role="status">{{ readingMarkerError === null ? '' : tError(readingMarkerError) }}</p>

    <!-- role="status" LUÔN có mặt, không v-if trên chính node -- TÁM nhánh phân biệt
         được qua `data-reading-status`, không chỉ qua chữ hiển thị.
         🔵 SỬA TẠI CHỖ (2026-08-30, Story 5.12) — bốn nhánh cuối đổi tên theo `ReadingRun`,
         xem `readingStatusKind` ở `readingState.ts`. -->
    <!-- aura-allow-text: `statusMessage` là một computed đã tự đi qua t()/tError() bên
         trong phần kịch bản phía trên -- xem doc-comment của nó ngay trên. -->
    <p class="status" role="status" :data-reading-status="readingStatusKind">{{ statusMessage }}</p>

    <!--
      🔴 THÊM (Story 5.12) — vòng lặp CHƯƠNG bọc ngoài khối `.page`: mỗi Chương của
      `readingRun.chapters` (đã lọc `Done` ở Rust) sinh MỘT `.page` (lề song ngữ + cột dịch)
      của riêng nó. Dãy rỗng (§I/O Matrix "Chạm biên ngay") ⇒ không `.page` nào render, chỉ
      `.frontier` bên dưới còn lại trên trang.
    -->
    <!--
      🔵 SỬA (rà 2026-08-30, Bản vá 4) — `<section>`, không `<div>`, và tiêu đề Chương hạ
      xuống `<h2>` (xem template bên trong). Bản trước mỗi Chương một `<h1>` ngang hàng, nên
      một dãy bốn Chương cho ra BỐN `<h1>` rồi một `<h2>` (mốc biên) treo dưới cái CUỐI cùng
      — cây tiêu đề nói sai cấu trúc trang cho trình đọc màn hình. `<section>` là nội dung
      PHÂN ĐOẠN có tên (qua `<h2>` bên trong nó, theo thuật toán outline HTML), và mốc biên
      giữ `<h2>` NGANG HÀNG — đúng thứ tự đọc tuyến tính "Chương 1, Chương 2, …, Mốc biên".
    -->
    <template v-for="chapter in runChapters" :key="chapter.chapter_id">
      <section class="page" :class="{ bilingual: readingBilingual }">
        <!--
          🔴 `font-size`/`line-height` KHÔNG đi thẳng vào `:style` — `check:tokens` Kiểm B2 đòi
          hai thuộc tính đó luôn là `var(--font-…)`/`var(--leading-…)` LITERAL trong khối kiểu
          scoped bên dưới. Giá trị HIỆU LỰC (token thuần, hoặc override từ thanh trượt) đi qua
          HAI biến CSS riêng (`--reading-font-size`/`--reading-line-height`, không nằm trong
          danh sách thuộc tính Kiểm B2 canh) rồi khối kiểu đó đọc lại bằng `var(...)`.
          The measure goes through `--reading-measure` so Kiểm K sees the rounding and `ch` resolves on `.column`.
        -->
        <div
          class="column"
          :style="{
            '--reading-font-size': readingStyle.fontSize,
            '--reading-line-height': readingStyle.lineHeight,
            '--reading-measure': readingStyle.measure,
          }"
        >
          <div class="chapter-head">
            <!-- aura-allow-text: untitled() goes through t(). -->
            <span v-if="chapter.chapter_title !== null" class="chapter-label">{{ untitled(chapter.chapter_ord) }}</span>
            <!-- aura-allow-text: originText() goes through t(). -->
            <span class="chapter-origin" data-reading-chapter-origin>{{ originText(chapter) }}</span>
          </div>
          <!-- aura-allow-text: `chapterLabel()` trả tên Chương (dữ liệu) hoặc gọi `untitled()`
               (qua t()) cho hàng chưa đặt tên -- không một biểu thức trộn nào lộ ra template. -->
          <h2 class="chapter-title">{{ chapterLabel(chapter) }}</h2>
          <!--
            🔵 Story 6.14 (FR42) — ảnh neo `0` (hoặc dời hẳn về đầu Chương), TRƯỚC đoạn ĐẦU
            TIÊN. `<figcaption>` chỉ dựng khi `caption_text` có nội dung THẬT (§Always: "ảnh
            không caption ⇒ không chừa chỗ trống") — chuỗi rỗng (chưa dịch) cùng luật với `null`.
          -->
          <figure v-for="image in leadingImagesOf(chapter)" :key="'lead-' + image.asset_id" class="reading-figure">
            <ChapterImage
              :file-name="image.file_name"
              :assets-dir="readingAssetsDir"
              :source-url="image.source_url"
              :alt-text="image.alt_text ?? ''"
            />
            <!-- aura-allow-text: `caption_text` là bản dịch của Tác phẩm — DỮ LIỆU, không câu giao diện. -->
            <figcaption v-if="image.caption_text !== null && image.caption_text !== ''" class="reading-figure-caption">{{ image.caption_text }}</figcaption>
          </figure>
          <template v-if="chapter.paragraphs.length > 0">
            <div v-for="(paragraph, index) in chapter.paragraphs" :key="index" class="reading-paragraph">
              <!-- aura-allow-text: nguyên văn là DỮ LIỆU của Tác phẩm, không câu giao diện. -->
              <p v-if="readingBilingual" class="source-note" data-reading-source-note @dblclick="dispatch('reading.lookup_open')">{{ paragraph.segments.map((s) => s.source_text).join(' ') }}</p>
              <!--
                🔵 Story 6.14 — MỘT đoạn Rust có thể vẽ thành NHIỀU `<p>` liền kề khi một ảnh
                chen giữa hai câu của nó (§I/O Matrix "ảnh giữa hai câu ⇒ hiện sau câu k ở CẢ
                HAI bề mặt"; xem doc-comment `paragraphRuns`). Ranh giới ĐOẠN của Rust không
                đổi — chỉ số THẺ HTML cần để vẽ nó đổi.
              -->
              <template v-for="(run, runIndex) in paragraphRuns(chapter, paragraph)" :key="runIndex">
                <p v-if="run.kind === 'text'" class="paragraph">
                  <!--
                    aura-allow-text: bản dịch là DỮ LIỆU của Tác phẩm, không câu giao diện.
                    🔴 **DẤU CÁCH GIỮA HAI CÂU LÀ MỘT NỘI DUNG, KHÔNG MỘT KHOẢNG TRẮNG THỪA** — bắt ở
                    lượt rà 2026-08-30. Bản đầu viết hai `<span>` liền nhau không gì ngăn, nên một đoạn
                    hai câu render thành *"…giữa bóng tối.Gió thổi tới…"*. Bản dựng UX
                    (`mockups/reading-mode.html`) không dính vì HTML nguồn của nó XUỐNG DÒNG giữa hai
                    `<span>` và khoảng trắng ấy co lại thành một dấu cách; một `v-for` trên một dòng thì
                    không có gì để co. Bàn đo e2e dùng `toContain` nên mù với chỗ này.
                    ⚠️ Dấu cách đặt Ở ĐẦU mọi câu TRỪ câu đầu — đặt ở CUỐI sẽ để lại một khoảng trắng
                    lơ lửng sau câu chót của mỗi đoạn. 🔵 Story 6.14 — "câu đầu" nay là câu đầu của
                    RUN (mảnh `<p>` sau khi chẻ quanh ảnh), không của cả đoạn Rust.
                    🔴 THÊM (Story 5.12, AC6) — `:class="{ unconfirmed: !segment.is_confirmed }"`: câu
                    CHƯA xác nhận mang gạch chấm nhẹ, dấu hiệu đến từ `is_confirmed` trên dây.
                  -->
                  <span
                    v-for="(segment, i) in run.segments"
                    :key="segment.id"
                    class="reading-segment"
                    :class="{
                      unconfirmed: !segment.is_confirmed,
                      marked: segment.is_marked,
                      'mark-side': markChipSideId === segment.id,
                      'mark-at-press': markChipAtPress?.segmentId === segment.id,
                    }"
                    :data-reading-segment="segment.id"
                    tabindex="0"
                    @mouseenter="onSegmentMouseEnter(segment.id)"
                    @mouseleave="onSegmentMouseLeave($event, segment.id)"
                    @focusin="onSegmentFocusIn(segment.id)"
                    @focusout="onSegmentFocusOut($event, segment.id)"
                    @keydown.enter="onReadingSegmentEnter"
                    ><!-- aura-allow-text: bản dịch là DỮ LIỆU Tác phẩm; biểu thức đầu chỉ ngăn câu. -->
                    <span class="segment-text" @mousedown="onSegmentTextMouseDown($event, segment.id)">{{ i === 0 ? '' : ' ' }}{{ segment.target_text }}</span>
                    <span
                      class="mark-actions"
                      :style="markChipAtPress?.segmentId === segment.id
                        ? { insetBlockStart: `${String(markChipAtPress.top)}px`, insetInlineStart: `${String(markChipAtPress.left)}px` }
                        : undefined"
                    >
                      <button
                        type="button"
                        tabindex="-1"
                        class="mark-affordance ui-btn ui-btn--quiet"
                        :aria-disabled="segment.is_marked ? 'true' : undefined"
                        @click="dispatch('reading.mark_aimed')"
                      ><!-- aura-allow-text: cả hai nhánh đều đi qua t(). -->{{ segment.is_marked ? t('mode.reading.marked') : t('mode.reading.mark_action') }}</button>
                      <button
                        type="button"
                        tabindex="-1"
                        class="mark-affordance ui-btn ui-btn--quiet"
                        @click="dispatch('reading.open_aimed')"
                      >{{ t('mode.reading.edit_action') }}</button>
                    </span></span
                  >
                </p>
                <!-- 🔵 Story 6.14 (FR42) — ảnh GIỮA hai câu của cùng một đoạn. -->
                <figure v-else class="reading-figure">
                  <ChapterImage
                    :file-name="run.image.file_name"
                    :assets-dir="readingAssetsDir"
                    :source-url="run.image.source_url"
                    :alt-text="run.image.alt_text ?? ''"
                  />
                  <!-- aura-allow-text: `caption_text` là bản dịch của Tác phẩm — DỮ LIỆU. -->
                  <figcaption v-if="run.image.caption_text !== null && run.image.caption_text !== ''" class="reading-figure-caption">{{ run.image.caption_text }}</figcaption>
                </figure>
              </template>
            </div>
          </template>
          <!-- aura-allow-text: `chapterEmptyNote()` tự đi qua t() bên trong. -->
          <p v-else class="chapter-note">{{ chapterEmptyNote(chapter) }}</p>
        </div>
      </section>
    </template>

    <!--
      🔴 THÊM (Story 5.12) — MỐC BIÊN, luôn có mặt khi một lượt đọc đã nạp (kể cả dãy rỗng):
      nó là lý do trang dừng ở đó, không phải một phần thưởng chỉ hiện khi có nội dung.
    -->
    <div v-if="readingRun !== null" class="frontier" :style="{ '--reading-font-size': readingStyle.fontSize, '--reading-measure': readingStyle.measure }">
      <h2 class="frontier-heading">{{ t('mode.reading.frontier_heading') }}</h2>
      <!-- aura-allow-text: `frontierRangeText` là một computed đã tự đi qua t() bên trong
           phần kịch bản phía trên -- xem doc-comment của nó ngay trên. -->
      <p class="frontier-range">{{ frontierRangeText }}</p>
      <!-- aura-allow-text: `frontierBlockedText` tự đi qua t() bên trong. -->
      <p v-if="readingRun.frontier.kind === 'next-not-done'" class="frontier-note" data-reading-frontier-kind="next-not-done">{{ frontierBlockedText }}</p>
      <!-- aura-allow-text: `frontierEndOfWorkText` tự đi qua t() bên trong. -->
      <p v-else class="frontier-note" data-reading-frontier-kind="end-of-work">{{ frontierEndOfWorkText }}</p>
      <div class="frontier-actions">
        <button
          v-if="readingRun.frontier.kind === 'next-not-done'"
          type="button"
          class="ui-btn ui-btn--primary"
          data-reading-frontier-continue
          @click="dispatch('reading.continue_in_workspace')"
        >
          <!-- aura-allow-text: `frontierButtonText` tự đi qua t() bên trong. -->
          {{ frontierButtonText }}
        </button>
        <button
          v-if="readingMarksTotal !== null && readingMarksTotal > 0"
          type="button"
          class="ui-btn"
          :class="{ 'ui-btn--primary': readingRun.frontier.kind !== 'next-not-done' }"
          data-reading-frontier-marks
          @click="dispatch('reading.marks')"
        >
          <!-- aura-allow-text: frontierMarksText goes through t(). -->
          {{ frontierMarksText }}
        </button>
        <button
          type="button"
          class="ui-btn"
          :class="{ 'ui-btn--primary': readingRun.frontier.kind !== 'next-not-done' && !(readingMarksTotal !== null && readingMarksTotal > 0) }"
          data-reading-frontier-work-start
          @click="dispatch('reading.work_start')"
        >
          {{ t('mode.reading.frontier_work_start') }}
        </button>
      </div>
    </div>

    <div
      v-if="readingMarksOpen"
      class="marks-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="reading-marks-heading"
      @click.self="dispatch('reading.marks_close')"
      @keydown.esc="onMarksEscape()"
      @keydown.tab="trapOverlayTab($event, marksPanel)"
    >
      <div ref="marksPanel" class="marks-panel" tabindex="-1" @keydown="onMarksKeydown($event)">
        <p id="reading-marks-heading" class="toc-heading">{{ t('mode.reading.marks_heading') }}</p>
        <p class="marks-hint">{{ t('mode.reading.marks_hint') }}</p>
        <!-- aura-allow-text: nhánh có chữ đi qua t(); nhánh còn lại là chuỗi rỗng. -->
        <p role="status">{{ readingMarksBusy ? t('mode.reading.marks_loading') : '' }}</p>
        <!-- aura-allow-text: lỗi dây đi qua tError()/chuỗi rỗng. -->
        <p role="status">{{ readingMarksError === null ? '' : tError(readingMarksError) }}</p>
        <!-- aura-allow-text: cả hai nhánh đi qua t()/chuỗi rỗng. -->
        <p role="status">{{ readingMarksHaveLoaded && readingMarksError === null && readingMarks.length === 0 ? t('mode.reading.marks_empty') : '' }}</p>
        <ul class="marks-list">
          <li
            v-for="(mark, index) in readingMarks"
            :key="mark.segment_id"
            class="marks-row"
            :class="{ current: index === readingMarksCursor }"
            :aria-current="index === readingMarksCursor ? 'true' : undefined"
          >
            <span class="marks-chapter">{{ t('mode.reading.marks_row_chapter', { chapter_ord: String(mark.chapter_ord) }) }}</span>
            <!-- aura-allow-text: target_text là dữ liệu Tác phẩm. -->
            <span class="marks-text">{{ mark.target_text === '' ? mark.source_text : mark.target_text }}</span>
            <span v-if="mark.is_retired" class="retired-note">{{ t('mode.reading.marks_retired') }}</span>
            <button
              type="button"
              class="btn ui-btn"
              :disabled="readingMarksBusy"
              @pointerdown="setReadingMarksCursor(index)"
              @focus="setReadingMarksCursor(index)"
              @click="dispatch('reading.marks_open')"
            >{{ t('mode.reading.marks_row_edit') }}</button>
            <button
              type="button"
              class="btn ui-btn"
              :disabled="readingMarksBusy"
              @pointerdown="setReadingMarksCursor(index)"
              @focus="setReadingMarksCursor(index)"
              @click="dispatch('reading.marks_unmark')"
            >{{ t('mode.reading.marks_row_unmark') }}</button>
          </li>
        </ul>
        <div class="toc-actions">
          <button type="button" class="btn ui-btn" :disabled="readingMarksBusy || readingMarks.length === 0" @click="dispatch('reading.marks_walk')">
            <!-- aura-allow-text: marksWalkText goes through t(). -->
            {{ marksWalkText }}
          </button>
          <button
            type="button"
            class="btn ui-btn"
            data-reading-marks-clear
            :disabled="readingMarksBusy || readingMarks.length === 0"
            @click="dispatch('reading.marks_clear')"
          >
            <!-- aura-allow-text: marksClearText goes through t(). -->
            {{ marksClearText }}
          </button>
          <button
            v-if="readingMarksClearPending"
            type="button"
            class="btn ui-btn"
            data-reading-marks-clear-cancel
            :title="commandTitle('reading.marks_clear_cancel')"
            @click="dispatch('reading.marks_clear_cancel')"
          >{{ t('mode.reading.marks_clear_cancel') }}</button>
          <button type="button" class="btn ui-btn" @click="dispatch('reading.marks_close')">{{ t('mode.reading.marks_close') }}</button>
        </div>
        <p class="marks-hint">{{ t('mode.reading.marks_keys') }}</p>
      </div>
    </div>

    <!--
      🔵 SỬA (rà 2026-08-30) — BA lỗ bàn phím của lớp phủ này, cả ba đều ngược NFR17:
      ① lớp phủ mở ra mà KHÔNG gì nhận tiêu điểm ⇒ người dùng bấm `⌘L` rồi phải Tab mò từ
         đầu tài liệu. Nay `tocPanel` được `focus()` ngay sau lượt mở (watcher dưới đây).
      ② `Escape` không đóng được ⇒ lối ra duy nhất là Tab tới nút "Đóng". Nay `@keydown.esc`
         bắt TẠI CHỖ trên lớp phủ. ⚠️ **Không** đăng ký một hợp âm `Escape` TRẦN toàn ứng
         dụng: `registry` không có phạm vi theo chế độ, nên một `Escape` toàn cục sẽ bắn
         trong mọi lớp phủ khác — Kiểm A của `check:commands` chỉ canh `@click`, nên
         `@keydown` tại chỗ là đúng cửa.
      ③ đóng xong tiêu điểm rơi tự do ⇒ trả nó về gốc Chế độ đọc.
    -->
    <div
      v-if="readingTocOpen"
      class="toc-overlay"
      role="dialog"
      aria-modal="true"
      @click.self="dispatch('reading.toc_close')"
      @keydown.esc="dispatch('reading.toc_close')"
      @keydown.tab="trapOverlayTab($event, tocPanel)"
    >
      <div ref="tocPanel" class="toc-panel" tabindex="-1">
        <p class="toc-heading">{{ t('mode.reading.toc_heading') }}</p>
        <!-- role="status" LUÔN có mặt (không v-if) -- cùng khuôn LibraryMode. -->
        <!-- aura-allow-text: cả hai nhánh đi qua tError()/chuỗi rỗng. -->
        <p class="toc-error" role="status">{{ readingTocError !== null ? tError(readingTocError) : '' }}</p>
        <!-- aura-allow-text: cả hai nhánh đi qua t()/chuỗi rỗng. -->
        <p class="toc-empty" role="status">
          {{ readingTocHaveLoaded && readingTocError === null && readingTocChapters.length === 0 ? t('mode.reading.toc_empty') : '' }}
        </p>
        <ul class="toc-list">
          <!-- 🔵 THÊM (rà 2026-08-30) `aria-current` — con trỏ danh sách chỉ được báo bằng
               MÀU NỀN thì một người dùng trình đọc màn hình đi `toc_next`/`toc_prev` không có
               đường nào biết đang chọn Chương nào trước khi bấm "Mở". Cùng khuôn `aria-current`
               mà `LibraryMode.vue` đã dùng cho hàng kết quả tìm kiếm. -->
          <li
            v-for="(row, index) in readingTocChapters"
            :key="row.chapter_id"
            :class="{ current: index === readingTocCursor }"
            :aria-current="index === readingTocCursor ? 'true' : undefined"
          >
            <!-- aura-allow-text: nhánh trái là tên Chương (dữ liệu); nhánh phải qua t(). -->
            {{ row.title ?? untitled(row.ord) }}
          </li>
        </ul>
        <div class="toc-actions">
          <button type="button" class="btn ui-btn" :disabled="readingTocBusy" @click="dispatch('reading.toc_prev')">
            {{ t('mode.reading.toc_prev') }}
          </button>
          <button type="button" class="btn ui-btn" :disabled="readingTocBusy" @click="dispatch('reading.toc_next')">
            {{ t('mode.reading.toc_next') }}
          </button>
          <button type="button" class="btn ui-btn" :disabled="readingTocBusy" @click="dispatch('reading.toc_open')">
            {{ t('mode.reading.toc_open_button') }}
          </button>
          <button type="button" class="btn ui-btn" @click="dispatch('reading.toc_close')">
            {{ t('mode.reading.toc_close_button') }}
          </button>
        </div>
      </div>
    </div>
    <ReadingLookupPopup v-if="readingLookupAnchor !== null && readingBilingual" :host="root" />
  </section>
</template>

<style scoped>
.mode {
  position: relative;
  height: 100%;
  padding: var(--space-panel-block) var(--space-screen-inline);
  padding-right: 11px; /* aura-allow-spacing: the reserved 10px scrollbar gutter completes the 21px inset */
  overflow: auto;
  scrollbar-gutter: stable;
}

/* Xem lý do đầy đủ ở `LibraryMode.vue` — `outline: none` chỉ áp cho gốc `tabindex="-1"`,
   không bao giờ áp cho `*:focus` (§Trap 4). */
.mode:focus {
  outline: none;
}

.toolbar,
.progress-line {
  margin-left: calc(-1 * var(--space-screen-inline));
  margin-right: -11px; /* aura-allow-spacing: cancels the 11px right inset of .mode */
  padding-left: var(--space-screen-inline);
  padding-right: 11px; /* aura-allow-spacing: with the 10px scrollbar gutter the right edge is 21px from the window */
  border-bottom: 1px solid var(--color-outline);
  background: var(--color-surface);
  color: var(--color-on-surface-variant);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-family: var(--face-ui-sm);
}

.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-unit) calc(var(--space-unit) * 3);
  min-height: 42px;
  margin-top: calc(-1 * var(--space-panel-block));
  padding-top: 6px; /* aura-allow-spacing: with the 7px below and the 1px border a single row is exactly 42px */
  padding-bottom: 7px;
}

.toolbar b {
  color: inherit;
  font-weight: var(--weight-ui-md-strong);
}

/* The hover box reaches 7px past the text, so buttons and plain items keep one 21px text-to-text rhythm. */
.toolbar .ui-btn {
  margin: 0 calc(var(--space-unit) * -1);
  padding: 0 var(--space-unit);
}

.toolbar .marks-button {
  margin-left: auto;
  color: var(--color-tm-text);
}

.type-group {
  display: inline-flex;
  margin: 0 calc(var(--space-unit) * -1);
}

.status {
  margin: 0 0 var(--space-panel-block) 0;
  color: var(--color-on-surface-variant);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  font-family: var(--face-ui-md-wrap);
}

/* `.page` carries the source-note font so `ch` in its grid track measures the same width as the note. */
.page {
  --source-note-inline: round(up, 30ch, 7px);
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, auto) minmax(0, 1fr);
  font-size: var(--font-read-lg);
  line-height: var(--leading-read-lg);
  font-family: var(--face-read-lg);
}

.page.bilingual {
  grid-template-columns: minmax(calc(var(--source-note-inline) + var(--space-panel-inline)), 1fr) minmax(0, auto) minmax(0, 1fr);
}

.reading-paragraph {
  display: flow-root;
}

/* Hangs into the left track beside its own paragraph; the flow-root paragraph grows to hold a long
   note, so it pushes the next paragraph down instead of overlapping the next note. */
.source-note {
  float: inline-start;
  inline-size: var(--source-note-inline);
  margin: 0 0 var(--reading-line-height) calc(-1 * (var(--source-note-inline) + var(--space-panel-inline))); /* aura-allow-spacing: one reading line box below; the negative inline margin is the note width (ch) plus the gap, not a grid step */
  text-align: end;
  font-size: var(--font-read-lg);
  line-height: var(--leading-read-lg);
  font-family: var(--face-read-lg);
  color: var(--color-on-surface-variant);
}

.column {
  grid-column: 2;
  /* `max-width` only guards narrow windows; the measure is `width`, since a `max-width` measure
     lets the `auto` grid track shrink the column to its content. */
  max-width: 100%;
  width: round(up, var(--reading-measure), 7px);
  /* All three levels share `family: read`; only size, leading and measure change, through the
     three CSS variables set in `:style`. */
  font-family: var(--family-read);
  font-size: var(--reading-font-size);
  line-height: var(--reading-line-height);
  position: relative;
}

.chapter-title {
  margin: 0 0 var(--reading-line-height) 0; /* aura-allow-spacing: one reading line box, a multiple of 7 by readingStyle */
  font-size: var(--font-read-title);
  line-height: var(--leading-read-title);
  font-weight: var(--weight-read-title);
  font-family: var(--face-read-title);
}

.paragraph {
  margin: 0 0 1lh 0; /* aura-allow-spacing: one reading line box, a multiple of 7 by readingStyle */
}

/*
 * 🔵 Story 6.14 (FR42 · FR43) — ảnh + chú thích. `<figure>` không viền/nền — nó là một khối
 * NỘI DUNG bình thường trong cột đọc, cùng `margin` dọc với `.paragraph` để nhịp đọc không đổi
 * khi một ảnh chen vào giữa hai đoạn/hai câu.
 */
.reading-figure {
  margin: 0 0 1lh 0; /* aura-allow-spacing: one reading line box, a multiple of 7 by readingStyle */
}

/*
 * Chú thích — `on-surface-variant` + `ui-sm` (chữ PHỤ, không phải văn xuôi của Tác phẩm — cùng
 * vai với khung giữ chỗ ảnh thiếu ở `ChapterImage.vue`), căn giữa dưới ảnh.
 */
.reading-figure-caption {
  margin: 7px 0 0 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  text-align: center;
}

/* Câu CHƯA xác nhận (Story 5.12, AC6) — gạch chấm nhẹ dưới chân, không đổi màu chữ và
   không `opacity` (Kiểm D cấm opacity lùi chữ). `ornament` là token vai `stroke` ĐÃ CÓ,
   đúng giá trị `--orn` mà mockup dùng cho `.s.unconf` -- 0 cặp tương phản mới. */
.unconfirmed {
  border-bottom: 1px dotted var(--color-ornament);
}

.reading-segment {
  display: inline;
  border-radius: var(--radius-default);
}

.reading-segment:focus,
.reading-segment:hover {
  background: var(--color-surface-accent);
}

.reading-segment.marked {
  background: var(--color-surface-tm);
}

.reading-segment.marked::after {
  content: '⚑';
  position: relative;
  top: -0.4em;
  color: var(--color-tm-text);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-family: var(--face-ui-sm);
}

/* Without a press anchor the block inset stays auto so the chip sits on the sentence's last line;
   a percentage block inset resolves against the first line and lands over the sentence's own text. */
.mark-actions {
  display: none;
  position: absolute;
  /* aura-allow-z-index: thứ tự sơn trong cùng dòng để nút tuyệt đối không nằm dưới chữ câu kế, không elevation */
  z-index: 1;
  white-space: nowrap;
  gap: var(--space-unit);
  margin-inline-start: var(--space-panel-inline);
  background: var(--color-surface);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-popover);
}

.reading-segment.mark-side .mark-actions {
  inset-inline-start: 100%;
}

.reading-segment.mark-at-press .mark-actions {
  margin-inline-start: 0;
}

.reading-segment:focus-within .mark-actions {
  display: inline-flex;
}

.marker-error,
.retired-note {
  color: var(--color-tm-text);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-family: var(--face-ui-sm);
}

.retired-note {
  display: block;
}

.chapter-note {
  margin: 0 0 var(--space-panel-block) 0;
  color: var(--color-on-surface-variant);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  font-family: var(--face-ui-md);
}

/* Mốc biên (Story 5.12) — chỉ token đã có: vạch ngăn dùng `--color-outline`, chữ phụ dùng
   `--color-on-surface-variant`. Không `box-shadow`, không gradient, không `opacity` lùi chữ. */
.frontier {
  width: round(up, var(--reading-measure), 7px);
  max-width: 100%;
  margin: calc(var(--space-unit) * 4) auto 0;
  font-size: var(--reading-font-size);
  font-family: var(--family-read);
  text-align: center;
  padding-top: 13px; /* aura-allow-spacing: 1px top border sits inside the 14px inset */
  border-top: 1px solid var(--color-outline);
}

.frontier-heading {
  margin: 0 0 var(--space-unit) 0;
  font-size: var(--font-read-sm);
  line-height: var(--leading-read-sm);
  font-weight: normal;
  font-family: var(--face-read-sm);
}

.frontier-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: var(--space-unit);
}

/* 🔵 SỬA (rà 2026-08-30, Bản vá 9) — `.frontier-blocked` đổi tên `.frontier-note`: lớp đó
   dùng cho CẢ HAI nhánh (`next-not-done` VÀ `end-of-work`), và "blocked" nói sai một nửa số
   ca -- nhánh `end-of-work` không hề bị chặn, nó báo tin đã đọc hết. */
.frontier-range,
.frontier-note {
  margin: 0 0 var(--space-panel-inline) 0;
  color: var(--color-on-surface-variant);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  font-family: var(--face-ui-md);
}

.toc-overlay,
.marks-overlay {
  position: fixed;
  inset: var(--space-titlebar-height) 0 var(--space-status-height);
  display: flex;
  justify-content: flex-end;
  align-items: stretch;
}

.toc-panel,
.marks-panel {
  width: min(420px, 100%);
  overflow: auto;
  padding: var(--space-panel-block) var(--space-screen-inline) var(--space-panel-block) var(--space-panel-inline);
  background: var(--color-surface);
  box-shadow: var(--shadow-popover);
}

.toc-heading {
  margin: 0 0 var(--space-panel-block) 0;
  font-size: var(--font-ui-md-strong);
  line-height: var(--leading-ui-md-strong);
  font-weight: var(--weight-ui-md-strong);
  font-family: var(--face-ui-md-strong);
}

.toc-list {
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
}

.toc-list li {
  padding: var(--space-panel-inline);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  font-family: var(--face-ui-md);
}

.toc-list li.current {
  background: var(--color-surface-accent);
}

.marks-panel:focus {
  /* aura-allow-outline-none: tabindex=-1 root, focused programmatically, not an interactive control. */
  outline: none;
}

.marks-hint {
  color: var(--color-on-surface-variant);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-family: var(--face-ui-sm);
}

.marks-list {
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
}

.marks-row {
  display: flex;
  align-items: baseline;
  gap: var(--space-unit);
  padding: var(--space-panel-inline) var(--space-panel-inline) 13px; /* aura-allow-spacing: 1px bottom border sits inside the 14px inset */
  border-bottom: 1px solid var(--color-outline);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  font-family: var(--face-ui-md);
}

.marks-row.current {
  background: var(--color-surface-accent);
}

.marks-chapter {
  flex: none;
  color: var(--color-on-surface-variant);
}

.marks-text {
  flex: 1 1 auto;
  overflow-wrap: anywhere;
}

.level-group {
  display: inline-flex;
  align-items: center;
  gap: var(--space-unit);
}

.level-seg .ui-seg__item[aria-pressed='true'] {
  background: var(--color-surface-accent);
  color: var(--color-primary);
}

.progress-line {
  display: flex;
  align-items: center;
  gap: var(--space-panel-inline);
  height: 28px;
  margin-bottom: 28px;
  background: var(--color-background);
  border-bottom-color: var(--color-outline-faint);
}

.progress-chapter {
  flex: 0 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.progress-count {
  flex: none;
  white-space: nowrap;
}

.progress-line b {
  color: var(--color-on-surface);
  font-weight: var(--weight-ui-md-strong);
}

.progress-track {
  flex: 1 1 auto;
  height: 2px; /* aura-allow-spacing: a drawn 2px rule */
  min-width: 70px;
  border-radius: var(--radius-full);
  background: var(--color-surface-sunken);
  overflow: hidden;
}

.progress-fill {
  display: block;
  height: 100%;
  background: var(--color-primary);
}

.chapter-head {
  display: flex;
  align-items: center;
  gap: var(--space-panel-inline);
  height: calc(var(--space-unit) * 3); /* the pill border overhangs by 1px instead of growing the row off the 7px grid */
  margin: 0 0 var(--space-unit) 0;
}

.chapter-label {
  color: var(--color-on-surface-variant);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-family: var(--face-ui-sm);
  text-transform: uppercase;
}

.chapter-origin {
  display: inline-flex;
  align-items: center;
  padding-left: var(--space-unit);
  padding-right: var(--space-unit);
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-full);
  color: var(--color-tm-text);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-family: var(--face-ui-sm);
}

.toc-actions {
  display: flex;
  gap: var(--space-unit);
  flex-wrap: wrap;
}
</style>
