<script setup lang="ts">
// Lớp phủ **Xem trước lượt nhập — bảng mã** (Story 6.3, FR126, AD-39 bước 1).
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 KHUÔN `GlossaryImportOverlay.vue` — scrim + `role="dialog"` + bẫy Tab tự viết +
// focus-return qua `data-import-preview-open` (đặt trên hai nút nộp form của
// `LibraryMode.vue`: dán văn bản VÀ tệp — cả hai mở lớp phủ NÀY).
//
// 🔴 Ba tầng theo thứ tự NHÂN QUẢ, KHÔNG nút "Tiếp theo" (§Always spec 6.3): bảng mã →
// ranh giới nội dung → luật làm sạch, tất cả trên MỘT màn hình. Tầng 2 (Story 6.9) LUÔN
// hiện, LUÔN rỗng, LUÔN nói ra vì sao. 🔵 SỬA 2026-09-05 (Story 6.5) — tầng 3 nay CÓ THÂN:
// văn bản đã đánh dấu gạch ngang + danh sách luật (xem/sửa/tắt/thêm/xoá). Gạch ngang render
// Ở ĐÂY, không ở tầng 2 — tầng 2 (bóc nội dung khối) vẫn rỗng, và §Design Notes spec 6.5
// giải thích đây là một lượt dời có chủ khi 6.9 dựng xong, không phải một lệch UX.
//
// 🔴 Dải năm ứng viên đi qua `<input type="radio">` + `@change`, KHÔNG `dispatch` — cùng lý
// do đúng hệt `GlossaryImportOverlay.vue`: `dispatch` không nhận tham số, và "hàng thứ N
// được chọn" không diễn đạt được qua nó. `check:commands` Kiểm A chỉ canh `@click`, nên
// `@change` nằm ngoài phạm vi của nó — có tiền lệ, không phải một lỗ hổng.
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
// Không `v-html` ở bất kỳ đâu (AD-16) — mọi bản dựng thật của dải là DỮ LIỆU văn bản thô từ
// Rust, không phải markup.
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import {
  addImportPreviewCleanupRule,
  cancelImportPreviewCleanupDeletePending,
  deleteImportPreviewCleanupRule,
  editImportPreviewCleanupRule,
  importPreview,
  importPreviewBlockActionError,
  importPreviewBlockFocusedIndex,
  importPreviewBlockRangeMissingStartNotice,
  importPreviewBlockRangeStart,
  importPreviewChapterPatternError,
  importPreviewChapterPatternKind,
  importPreviewChapterPatternSending,
  importPreviewChapterPatternText,
  importPreviewCleanupActionError,
  importPreviewCleanupAdding,
  importPreviewCleanupDeletePendingKey,
  importPreviewCleanupDeleting,
  importPreviewCleanupSavingEdit,
  importPreviewCleanupToggling,
  importPreviewConfirmError,
  importPreviewConfirming,
  importPreviewDomainLogDomainCount,
  importPreviewEmptyReasonForTier,
  importPreviewIsOpen,
  importPreviewJumpToCleanupRulesSignal,
  importPreviewLastSubmittedFrom,
  importPreviewLoadError,
  importPreviewSelectedBlocks,
  importPreviewSelectedCandidate,
  importPreviewSelectedChapters,
  importPreviewSelectedCleanup,
  importPreviewSelectedEncoding,
  importPreviewSelectedNormalized,
  importPreviewStatus,
  importPreviewStripIsOpen,
  importPreviewUrlImportBusy,
  importPreviewUrlImportError,
  importPreviewUrlItems,
  removeImportPreviewUrlItem,
  reloadImportPreviewUrlItem,
  selectImportPreviewCandidate,
  setImportPreviewChapterPattern,
  toggleImportPreviewCleanupRule,
} from './importPreviewState'
import type {
  BlockWire,
  ChapterPatternKindWire,
  ChapterSplitPreviewEntryWire,
  CleanupRuleKindWire,
  CleanupRuleReportWire,
  CleanupRuleTierWire,
  CleanupSpanWire,
  ImportConfidence,
} from './config/project'

/**
 * 🔴 SỬA (vòng rà đối kháng 2, mục 22) — hai hàm này thay cho việc GHÉP CHUỖI khoá i18n
 * bằng nội suy (`` `mode.library.preview.confidence_${…}` ``): một khoá dựng bằng nội suy vô
 * hình với BẤT KỲ cổng nào quét mã tìm literal `t('…')`/`` t(`…`) `` cố định (kể cả
 * `check:i18n`) — xoá nhầm một khoá `confidence_high` ở `vi.json` không lộ ra ở đây, chỉ lộ
 * lúc CHẠY THẬT. `switch` cạn (không nhánh `default`) buộc trình biên dịch TypeScript báo lỗi
 * nếu [`ImportConfidence`]/kiểu trả của [`importPreviewEmptyReasonForTier`] thêm một biến thể
 * mới mà quên cập nhật ở đây — cùng an toàn của một BẢNG dữ liệu, không phải một chuỗi ghép.
 */
function confidenceMessageKey(confidence: ImportConfidence): string {
  switch (confidence) {
    case 'self_declared':
      return 'mode.library.preview.confidence_self_declared'
    case 'high':
      return 'mode.library.preview.confidence_high'
    case 'low':
      return 'mode.library.preview.confidence_low'
  }
}

// 🔵 SỬA 2026-09-05 (Story 6.5) — không còn `switch` (chỉ MỘT biến thể từ khi tầng 3 có
// thân, `@typescript-eslint/no-unnecessary-condition` đúng khi báo một nhánh `switch`/so
// sánh là thừa trên một kiểu chỉ CÒN một giá trị). Tham số giữ nguyên KIỂU (không đổi
// thành `void`) để chữ ký không đổi hình dạng nếu một lý do RỖNG thứ hai xuất hiện cho
// tầng 2 sau này — hàm gọi vẫn phải truyền ĐÚNG kiểu, chỉ thân hàm không còn rẽ nhánh.
function tierEmptyMessageKey(reason: 'story_6_9'): string {
  void reason
  return 'mode.library.preview.tier_empty_story_6_9'
}

/**
 * Tầng chuẩn hoá (Story 6.4, FR124/FR125) KHÔNG có bản dựng để hiện vì đúng MỘT trong hai lý
 * do — chưa CHỌN ứng viên nào (dải rỗng thật, nhánh tự khai `AlreadyText`/rỗng) hoặc ứng
 * viên ĐANG chọn "không ra chữ" (`candidate.normalized === null`, đồng bộ `preview === null`).
 * Cùng khuôn `tierEmptyMessageKey` — `switch` cạn, không ghép chuỗi khoá.
 */
function normalizedTierEmptyReason(): 'no_candidate' | 'undecodable' {
  return importPreviewSelectedCandidate.value === null ? 'no_candidate' : 'undecodable'
}

function normalizedTierEmptyMessageKey(reason: 'no_candidate' | 'undecodable'): string {
  switch (reason) {
    case 'no_candidate':
      return 'mode.library.preview.tier_normalized_empty_no_candidate'
    case 'undecodable':
      return 'mode.library.preview.tier_normalized_empty_undecodable'
  }
}

/**
 * Tầng làm sạch (Story 6.5) KHÔNG có bản dựng để hiện vì đúng MỘT trong hai lý do — CÙNG hai
 * lý do và CÙNG điều kiện với tầng chuẩn hoá ([`normalizedTierEmptyReason`]):
 * `candidate.cleanup`/`self_declared_cleanup` đồng bộ `null` với `candidate.normalized`/
 * `self_declared_normalized` (cả hai tính từ CÙNG lượt chạy chuỗi thật cho ứng viên đó).
 */
function cleanupTierEmptyMessageKey(reason: 'no_candidate' | 'undecodable'): string {
  switch (reason) {
    case 'no_candidate':
      return 'mode.library.preview.tier_cleanup_empty_no_candidate'
    case 'undecodable':
      return 'mode.library.preview.tier_cleanup_empty_undecodable'
  }
}

/** Một mảnh của văn bản tầng 3, cộng việc mảnh đó có bị MỘT (hoặc nhiều) luật ĐANG BẬT phủ
 * hay không — Story 6.5. Điểm mã, cùng quy ước `SegmentTermSpan`/`GridPanel.vue`. */
type CleanupTextPiece = { text: string; struck: boolean }

/**
 * Cắt `text` thành các mảnh LIỀN NHAU theo biên của `spans` (điểm mã, nửa-mở `[start,
 * end)`) — mỗi mảnh HOẶC hoàn toàn bị phủ HOÀN TOÀN không bị phủ, không bao giờ NỬA mảnh.
 * `spans` đã được Rust lọc còn ĐÚNG luật ĐANG BẬT (`commands::project::build_cleanup_preview_wire`),
 * nên "bị phủ" ở đây LUÔN đồng nghĩa "sẽ bị xoá khi xác nhận".
 *
 * Hai luật khớp CHỒNG NHAU (AD-18 hợp nhất, không khử trùng lặp trong `spans`) vẫn chỉ tạo
 * ĐÚNG một dải "bị phủ" ở đây — `some()` không đếm số luật, chỉ hỏi CÓ hay KHÔNG.
 */
function cleanupPiecesOf(text: string, spans: readonly CleanupSpanWire[]): CleanupTextPiece[] {
  const codepoints = [...text]
  if (spans.length === 0 || codepoints.length === 0) return [{ text, struck: false }]

  const boundarySet = new Set<number>([0, codepoints.length])
  for (const span of spans) {
    boundarySet.add(Math.max(0, Math.min(span.start, codepoints.length)))
    boundarySet.add(Math.max(0, Math.min(span.end, codepoints.length)))
  }
  const boundaries = [...boundarySet].sort((a, b) => a - b)

  const pieces: CleanupTextPiece[] = []
  for (let i = 0; i < boundaries.length - 1; i += 1) {
    const start = boundaries[i]
    const end = boundaries[i + 1]
    if (start === end) continue
    const struck = spans.some((span) => span.start <= start && end <= span.end)
    pieces.push({ text: codepoints.slice(start, end).join(''), struck })
  }
  return pieces
}

/** Mảnh văn bản của khối làm sạch ĐANG HIỆN — `[]` khi tầng này rỗng (xem
 * [`cleanupTierEmptyMessageKey`]). */
const cleanupPieces = computed<CleanupTextPiece[]>(() => {
  const cleanup = importPreviewSelectedCleanup.value
  if (cleanup === null || cleanup.text === '') return []
  return cleanupPiecesOf(cleanup.text, cleanup.spans)
})

function cleanupTierLabelKey(tier: CleanupRuleTierWire): string {
  return tier === 'global' ? 'mode.library.preview.cleanup_tier_global' : 'mode.library.preview.cleanup_tier_work'
}

/** Handler `@change` của tick bật/tắt một luật — `<input type="checkbox">` nằm ngoài phạm vi
 * Kiểm A của `check:commands` (chỉ canh `@click`), cùng tiền lệ dải bảng mã. */
function onToggleCleanupRule(rule: CleanupRuleReportWire, event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  void toggleImportPreviewCleanupRule(rule.tier, rule.id, target.checked)
}

/** Khoá `${tier}:${id}` của DÒNG này — cùng phép dựng của `importPreviewState.ts::cleanupRuleKey`
 * (không export hàm đó; so bằng chuỗi ở đây là đủ, template chỉ cần một vị từ true/false). */
function ruleKey(rule: CleanupRuleReportWire): string {
  return `${rule.tier}:${rule.id}`
}

/** Dòng này đang ở NHỊP HAI (chờ bấm lại để xoá thật) hay chưa — khuôn
 * `GlossaryManageOverlay.vue::manageDeletePending`. */
function isDeletePending(rule: CleanupRuleReportWire): boolean {
  return importPreviewCleanupDeletePendingKey.value === ruleKey(rule)
}

/** Khoá i18n của nút xoá — `switch` cạn (thật ra hai nhánh, đủ để cạn), khuôn
 * `cleanupTierLabelKey`: KHÔNG ghép chuỗi bằng nội suy, để `check:i18n` thấy được literal. */
function cleanupDeleteLabelKey(pending: boolean): string {
  return pending
    ? 'mode.library.preview.cleanup_delete_rule_confirm'
    : 'mode.library.preview.cleanup_delete_rule'
}

function cleanupDeleteAriaLabelKey(pending: boolean): string {
  return pending
    ? 'mode.library.preview.cleanup_delete_rule_confirm_aria_label'
    : 'mode.library.preview.cleanup_delete_rule_aria_label'
}

function onDeleteCleanupRule(rule: CleanupRuleReportWire): void {
  void deleteImportPreviewCleanupRule(rule.tier, rule.id)
}

/** Luật đang được SỬA — `null` khi không luật nào đang mở form sửa. Cùng khuôn "một chế độ
 * sửa nội tuyến" của `GlossaryImportOverlay.vue`. */
const editingRule = ref<{ tier: CleanupRuleTierWire; id: number } | null>(null)
const editPattern = ref('')
const editKind = ref<CleanupRuleKindWire>('literal')

function isEditingRule(rule: CleanupRuleReportWire): boolean {
  return editingRule.value !== null && editingRule.value.tier === rule.tier && editingRule.value.id === rule.id
}

function onStartEditCleanupRule(rule: CleanupRuleReportWire): void {
  editingRule.value = { tier: rule.tier, id: rule.id }
  editPattern.value = rule.pattern
  editKind.value = rule.kind
  // Đổi ý sang SỬA khi một hàng KHÁC đang ở nhịp "chờ xác nhận xoá" — tan trạng thái đó, đúng
  // khuôn `discardOpenEdit` của Glossary (một hành động mới làm nhịp chờ cũ hết hiệu lực).
  cancelImportPreviewCleanupDeletePending()
}

function onCancelEditCleanupRule(): void {
  editingRule.value = null
  editPattern.value = ''
}

async function onSaveEditCleanupRule(): Promise<void> {
  const target = editingRule.value
  if (target === null || editPattern.value.trim() === '') return
  await editImportPreviewCleanupRule(target.tier, target.id, editPattern.value, editKind.value)
  editingRule.value = null
  editPattern.value = ''
}

/**
 * Ô "＋ thêm luật mới" — Story 6.5, khuôn mockup `web-import.html:355`.
 *
 * 🔴 **KHÔNG ô chọn tầng — CHỈ tạo luật tầng Toàn cục (vòng rà 2026-09-06, phán quyết Ice).**
 * `commands::project::store_for_tier` (qua `commands::cleanup`) phân giải tầng Tác phẩm từ
 * `OpenWorkState` — Tác phẩm ĐANG MỞ. Lượt nhập Library (nơi lớp phủ này sống) đang TẠO một
 * Tác phẩm CHƯA TỒN TẠI, nên một ô chọn "Tác phẩm" ở màn này hoặc trượt với
 * `cleanup.work_tier_unavailable`, hoặc — tệ hơn — âm thầm đính luật vào `project.db` của một
 * Tác phẩm KHÁC đang mở (đúng lớp "luật ẩn xoá nhầm" mà FR124 tồn tại để chặn). Nửa ĐỌC/hợp
 * nhất của tầng Tác phẩm giữ NGUYÊN (`ScopeResolver::apply_merge` không đổi) — chỉ bề mặt
 * SOẠN ở màn này bị thu hẹp. Ghi nợ có chủ ở `deferred-work.md` cho việc soạn luật tầng Tác
 * phẩm (cần một bề mặt riêng, sau khi Tác phẩm đã tồn tại — ví dụ màn Cài đặt của chính
 * Tác phẩm đó, không phải màn xem trước NHẬP).
 */
const newRulePattern = ref('')
const newRuleKind = ref<CleanupRuleKindWire>('literal')

async function onAddCleanupRule(): Promise<void> {
  if (newRulePattern.value.trim() === '') return
  await addImportPreviewCleanupRule('global', newRulePattern.value, newRuleKind.value)
  newRulePattern.value = ''
}

/**
 * Tầng tách Chương (tầng 4, Story 6.6) — CÙNG điều kiện rỗng với tầng làm sạch
 * ([`normalizedTierEmptyReason`]): `candidate.chapters`/`self_declared_chapters` đồng bộ
 * `null` với `candidate.cleanup`/`self_declared_cleanup` (cả hai tính từ CÙNG lượt chạy
 * chuỗi thật cho ứng viên đó).
 */
function chaptersTierEmptyMessageKey(reason: 'no_candidate' | 'undecodable'): string {
  switch (reason) {
    case 'no_candidate':
      return 'mode.library.preview.tier_chapters_empty_no_candidate'
    case 'undecodable':
      return 'mode.library.preview.tier_chapters_empty_undecodable'
  }
}

/**
 * Ô nhập mẫu phân tách Chương — bản NHÁP cục bộ, đồng bộ MỘT CHIỀU từ state module (reset
 * khi mở lượt xem trước mới/huỷ/xác nhận — §Ask First spec 6.6: KHÔNG nhớ mẫu giữa hai lượt
 * nhập). Hành động chỉ gửi lên Rust ở `@change` (mất tiêu điểm/Enter cho ô chữ, chọn lại cho
 * ô kind) — KHÔNG mỗi phím gõ, đúng "MỘT vòng IPC" của §I/O Matrix spec 6.6.
 */
const chapterPatternDraft = ref(importPreviewChapterPatternText.value)
const chapterPatternKindDraft = ref<ChapterPatternKindWire>(importPreviewChapterPatternKind.value)

watch(importPreviewChapterPatternText, (value) => {
  chapterPatternDraft.value = value
})
watch(importPreviewChapterPatternKind, (value) => {
  chapterPatternKindDraft.value = value
})

function onChapterPatternChange(): void {
  void setImportPreviewChapterPattern(chapterPatternDraft.value, chapterPatternKindDraft.value)
}

/**
 * Sắp theo độ dài — ĐƯỜNG DUY NHẤT để chỗ bắt nhầm tự lộ ra (AC5 spec 6.6, §Design Notes "Vì
 * sao KHÔNG có cờ đáng ngờ"). State HIỂN THỊ THUẦN, không một lượt IPC nào — Rust đã cấp sẵn
 * độ dài của MỌI Chương. Toggle qua `@change` của một checkbox (không `@click`, AD-34).
 */
const chapterSortByLength = ref(false)

function onToggleChapterSortByLength(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  chapterSortByLength.value = target.checked
}

/** MỌI Chương, sắp NGẮN NHẤT trước — 14 mảnh 38-51 chữ lên đầu danh sách giữa những mảnh
 * 4.000 chữ (đúng ví dụ §Design Notes spec 6.6). */
const chapterEntriesSortedByLength = computed<ChapterSplitPreviewEntryWire[]>(() => {
  const chapters = importPreviewSelectedChapters.value
  if (chapters === null) return []
  return [...chapters.chapters].sort((a, b) => a.length - b.length)
})

/** Khung nhìn MẶC ĐỊNH (chưa sắp xếp) — ba Chương đầu, `⋯`, ba Chương cuối, đúng Task list
 * spec 6.6. N ≤ 6 thì hiện TRỌN, không cắt (không có gì để mà `⋯`). */
const chapterEntriesDefaultWindow = computed<{
  first: ChapterSplitPreviewEntryWire[]
  last: ChapterSplitPreviewEntryWire[]
  showEllipsis: boolean
}>(() => {
  const chapters = importPreviewSelectedChapters.value
  if (chapters === null) return { first: [], last: [], showEllipsis: false }
  const list = chapters.chapters
  if (list.length <= 6) return { first: list, last: [], showEllipsis: false }
  return { first: list.slice(0, 3), last: list.slice(-3), showEllipsis: true }
})

/** 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Khuôn và lý lẽ chép từ `GlossaryImportOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
// Lớp phủ chứa bản dựng thật của nội dung đang nhập (không phải nguồn từ điển).
useSelectionSurface(panel, 'display')

watch(importPreviewIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-import-preview-open]')
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-import-preview-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[import-preview] focus-return target is gone; focus falls back to body.')
})

function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
}

/** Bẫy tiêu điểm — điều kiện để `aria-modal="true"` không phải một lời khai sai. */
function trapTab(event: KeyboardEvent): void {
  const root = panel.value
  if (root === null) return

  event.preventDefault()
  const stops = focusableWithin(root)
  if (stops.length === 0) {
    root.focus()
    return
  }

  const active = document.activeElement
  const index = active instanceof HTMLElement ? stops.indexOf(active) : -1
  const step = event.shiftKey ? -1 : 1
  const next = index === -1 ? (event.shiftKey ? stops.length - 1 : 0) : index + step
  stops[(next + stops.length) % stops.length].focus()
}

function onCandidateChange(encoding: string, event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement) || !target.checked) return
  selectImportPreviewCandidate(encoding)
}

/**
 * **THÊM (Story 6.7)** — tải lại/bỏ MỘT mục của danh sách URL. Tham số `index` không diễn
 * đạt được qua `dispatch('<id>')` (không nhận tham số) — thao tác mang tham số đi qua
 * `@submit`, cùng khuôn `onDeleteCleanupRule`/`onStartEditCleanupRule` ngay trên.
 */
function onReloadUrlItem(index: number): void {
  void reloadImportPreviewUrlItem(index)
}
function onRemoveUrlItem(index: number): void {
  void removeImportPreviewUrlItem(index)
}

/**
 * 🔴 Vòng rà đối kháng 2, mục 4 — CHẶN THỊ GIÁC, lớp phòng thủ THỨ HAI. Lớp CHÍNH sống ở
 * `importPreviewState.ts::cancelImportPreview` (no-op khi `importPreviewConfirming`) — hàm
 * đó đóng cửa sổ đua triệt để, bất kể tầng `.vue` có gọi tới hay không. Chặn ở đây chỉ để
 * `Esc` không phát một `dispatch` vô ích trong lúc phím tắt khác (Tab) vẫn cần chạy —
 * `@keydown` nằm ngoài Kiểm A của `check:commands` (chỉ canh `@click`), nên một hàm thay vì
 * `dispatch(...)` trần ở đây không phải một lỗ hổng.
 */
function onEscapeCancel(): void {
  if (importPreviewConfirming.value) return
  dispatch('import.preview.cancel')
}

/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * Story 6.9 — sửa ranh giới bóc bằng bàn phím (FR123)
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * Hai số ở đầu tầng — đếm CỤC BỘ trên `importPreviewSelectedBlocks.value.blocks` (dữ liệu đã
 * áp override, Rust đã tính `kept` HIỆU LỰC — không tính lại gì ở đây, chỉ đếm).
 */
const tier2Counts = computed<{ kept: number; excluded: number } | null>(() => {
  const blocks = importPreviewSelectedBlocks.value?.blocks
  if (blocks === undefined) return null
  let kept = 0
  for (const block of blocks) {
    if (block.kept) kept += 1
  }
  return { kept, excluded: blocks.length - kept }
})

/** Ba vạch lề hiển thị — `switch` cạn, cùng khuôn [`confidenceMessageKey`]/[`tierEmptyMessageKey`]
 * (không ghép chuỗi khoá bằng nội suy). */
function blockStateMessageKey(block: BlockWire): string {
  if (!block.kept) return 'mode.library.preview.tier2_state_dropped'
  return block.confirmed
    ? 'mode.library.preview.tier2_state_kept_confirmed'
    : 'mode.library.preview.tier2_state_kept_guessed'
}

/** Class CSS của một khối — vạch lề (border-left) đổi màu theo TRẠNG THÁI, tiêu điểm bàn
 * phím đổi RIÊNG (khối `primary`, §Always spec 6.9: không qua `box-shadow`, Kiểm F cấm tuyệt
 * đối). Hai điều kiện ĐỘC LẬP — một khối có thể VỪA `dropped` VỪA đang giữ tiêu điểm. */
function blockStateClass(block: BlockWire, index: number): Record<string, boolean> {
  return {
    'ip-block-dropped': !block.kept,
    'ip-block-kept-guessed': block.kept && !block.confirmed,
    'ip-block-kept-confirmed': block.kept && block.confirmed,
    'ip-block-focused': index === importPreviewBlockFocusedIndex.value,
  }
}

/** `id` DOM ổn định của một khối — dùng cho `aria-activedescendant`, cùng khuôn
 * `GlossaryManageOverlay.vue::manageOptionId`. */
function blockDomId(index: number): string {
  return `ip-block-${index}`
}

/**
 * Handler DOM CỤC BỘ trên `.ip-scrim` — KHÔNG một hợp âm toàn cục (xem doc-comment
 * `commands/index.ts` tại chỗ đăng ký sáu command tương ứng). Cùng khuôn
 * `GlossaryManageOverlay.vue::onKeydown`: lọc `ctrl/meta/alt` VÀ target là trường nhập
 * (kể cả `<button>`/`contenteditable`) TRƯỚC MỌI NHÁNH — nếu không, `Space` trên nút
 * "Xác nhận"/"Huỷ"/dải bảng mã sẽ bị `preventDefault()` cướp mất, đúng khuyết tật mà spec 6.9
 * đo được ở `keys.ts`.
 *
 * 🔴 **SỬA 2026-09-07 (vòng rà bước 4, mục 8) — thêm `contenteditable` vào bộ lọc trường
 * nhập.** Bốn kiểu `HTMLInputElement`/`HTMLTextAreaElement`/`HTMLSelectElement`/
 * `HTMLButtonElement` không phủ một vùng `contenteditable` (không tồn tại trong màn HIỆN
 * TẠI, nhưng đây là bộ lọc CHUNG cho cả `.ip-scrim` — một trường soạn thảo thêm sau sẽ lặng
 * lẽ lọt qua nếu bộ lọc không tự khai đủ điều kiện của chính nó).
 *
 * 🔴 **SỬA 2026-09-07 (vòng rà bước 4, mục 8) — return sớm khi `importPreviewSelectedBlocks
 * === null`.** Đường KHÔNG phải URL (dán văn bản/tệp) không có khái niệm "khối" — trước bản
 * vá này, `Space` ở đó vẫn bị `preventDefault()` (dispatch rơi vào no-op ở tầng state, nhưng
 * phím GIỮ NGUYÊN bị cướp mất khỏi mọi chỗ khác trên trang, ví dụ một `<textarea>` KHÔNG bị
 * bộ lọc `isFormField` bắt vì nó nằm NGOÀI cây `.ip-scrim` nhưng vẫn nhận `keydown` bị bọc
 * — điều này không xảy ra ở đây vì `.ip-scrim` chỉ bắt sự kiện bên trong nó, nhưng giữ return
 * sớm là đúng ĐẦU TIÊN vì lý do ngữ nghĩa: các phím này không có gì để làm khi tầng 2 không
 * tồn tại).
 */
function onTier2Keydown(event: KeyboardEvent): void {
  if (event.ctrlKey || event.metaKey || event.altKey) return

  const target = event.target
  const isFormField =
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    target instanceof HTMLButtonElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  if (isFormField) return

  if (importPreviewSelectedBlocks.value === null) return

  switch (event.key) {
    case 'j':
    case 'J':
      event.preventDefault()
      dispatch('import.preview.block_next')
      return
    case 'k':
    case 'K':
      event.preventDefault()
      dispatch('import.preview.block_prev')
      return
    case ' ':
    case 'Spacebar': // Safari cũ
      // 🔴 SỬA 2026-09-07 (vòng rà bước 4, mục 7) — chặn AUTO-REPEAT. Giữ `Space` xuống phát
      // một chuỗi `keydown` lặp (`event.repeat === true` từ lần thứ hai) — không chặn thì
      // MỘT lượt giữ phím gửi NHIỀU lệnh `tier2_block_set_kept` chồng nhau (mỗi lệnh MỘT lượt
      // ghi thật, §Always spec 6.9), đảo trạng thái qua lại vô ích và tốn N lượt IPC cho một
      // cử chỉ người dùng chỉ định làm MỘT LẦN — cùng khuôn `GlossaryManageOverlay.vue:336`
      // (gác `event.repeat` trên một hành động phá huỷ/đảo trạng thái).
      if (event.repeat) return
      event.preventDefault()
      dispatch('import.preview.block_toggle_kept')
      return
    case '[':
      event.preventDefault()
      dispatch('import.preview.block_mark_range_start')
      return
    case ']':
      event.preventDefault()
      dispatch('import.preview.block_confirm_range')
      return
    case 'r':
    case 'R':
      event.preventDefault()
      dispatch('import.preview.jump_to_cleanup_rules')
      return
    default:
      return
  }
}

/**
 * `<ol role="listbox">` — template ref RIÊNG (khuôn `GlossaryManageOverlay.vue::list`,
 * §doc-comment tại đó). `tabindex="-1"` (template) + `.focus()` ở đây khi tiêu điểm khối đổi
 * (`J`/`K`) là ĐIỀU KIỆN để `aria-activedescendant` trên chính `<ol>` có nghĩa với trình đọc
 * màn hình — ARIA 1.2 chỉ tôn trọng thuộc tính đó trên phần tử ĐANG giữ tiêu điểm DOM thật,
 * và trước bản vá này không phần tử nào trong tầng 2 từng nhận `focus()` cả (mục 10, vòng rà
 * bước 4).
 */
const blocksList = useTemplateRef<HTMLElement>('blocksList')

/** `J`/`K` — cuộn khối mới vào tầm nhìn (mục 9) VÀ chuyển tiêu điểm DOM sang `<ol>` (mục 10).
 * Không `{ immediate: true }` — chỉ cuộn/focus khi tiêu điểm THẬT SỰ đổi qua một thao tác
 * điều hướng, không phải lúc mở màn (tiêu điểm ban đầu vẫn ở `panel`, giữ khuôn hiện có). */
watch(importPreviewBlockFocusedIndex, (index) => {
  void nextTick(() => {
    document.getElementById(blockDomId(index))?.scrollIntoView({ block: 'nearest' })
    blocksList.value?.focus()
  })
})

/** `R` — cuộn + đặt tiêu điểm sang tầng 3 (§Spec Change Log spec 6.9). */
const cleanupTierSection = useTemplateRef<HTMLElement>('cleanupTierSection')
watch(importPreviewJumpToCleanupRulesSignal, () => {
  void nextTick(() => {
    const el = cleanupTierSection.value
    if (el === null) return
    el.scrollIntoView({ block: 'nearest' })
    el.focus()
  })
})


</script>

<template>
  <div
    v-if="importPreviewIsOpen"
    class="ip-scrim"
    @keydown.esc="onEscapeCancel"
    @keydown.tab="trapTab($event)"
    @keydown="onTier2Keydown"
  >
    <section ref="panel" class="ip-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="ip-head">
        <h2 class="ip-title">{{ t('mode.library.preview.title') }}</h2>
        <button
          type="button"
          class="ip-close"
          :disabled="importPreviewConfirming"
          @click="dispatch('import.preview.cancel')"
        >
          {{ t('command.import.preview.cancel') }}
        </button>
      </header>

      <p v-if="importPreviewStatus === 'unknown'" class="ip-status" role="status">
        {{ t('mode.library.preview.loading') }}
      </p>
      <p v-else-if="importPreviewStatus === 'ipc_unavailable'" class="ip-empty">
        {{ t('mode.library.preview.empty_ipc_unavailable') }}
      </p>
      <p v-else-if="importPreviewStatus === 'error' && importPreviewLoadError !== null" class="ip-empty ip-error" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(importPreviewLoadError) }}
      </p>

      <template v-else-if="importPreviewStatus === 'loaded'">
        <!--
          ═══════════ Danh sách mục-theo-link (Story 6.7, FR122) ═══════════
          LUÔN hiện khi lượt đang mở đến từ nhánh URL — kể cả khi TOÀN BỘ mục đã OK (người
          dùng vẫn cần thấy/soát lại danh sách trước khi xác nhận). Vị trí GIỮ NGUYÊN
          (§Always spec 6.7): một mục hỏng không bị đẩy xuống cuối, `v-for` lặp thẳng trên
          mảng đã nhận từ Rust theo ĐÚNG thứ tự đó.
        -->
        <section v-if="importPreviewLastSubmittedFrom === 'urls'" class="ip-tier ip-url-list" aria-labelledby="ip-url-list-title">
          <h3 id="ip-url-list-title" class="ip-tier-title">
            {{ t('mode.library.preview.url_list_title', { count: String(importPreviewUrlItems.length) }) }}
          </h3>
          <p v-if="importPreviewUrlImportError !== null" class="ip-status ip-error" role="alert">
            <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
            {{ tError(importPreviewUrlImportError) }}
          </p>
          <ul class="ip-url-items">
            <li v-for="(item, i) in importPreviewUrlItems" :key="i" class="ip-url-item" :class="{ 'ip-url-item-broken': !item.ok }">
              <!-- aura-allow-text: DỮ LIỆU (vị trí 1-based trong danh sách, KHÔNG markup — AD-16). -->
              <span class="ip-url-position">{{ i + 1 }}</span>
              <!-- aura-allow-text: DỮ LIỆU (URL người dùng tự dán). -->
              <span class="ip-url-address">{{ item.url }}</span>
              <span v-if="item.ok" class="ip-url-ok">{{ t('mode.library.preview.url_item_ok') }}</span>
              <span v-else-if="item.error !== null" class="ip-url-reason" role="alert">
                <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
                {{ tError(item.error) }}
              </span>
              <form class="ip-url-action-form" @submit.prevent="onReloadUrlItem(i)">
                <button
                  v-if="!item.ok"
                  type="submit"
                  class="ip-url-reload"
                  :disabled="importPreviewConfirming || importPreviewUrlImportBusy"
                >
                  {{ t('mode.library.preview.url_item_reload') }}
                </button>
              </form>
              <form class="ip-url-action-form" @submit.prevent="onRemoveUrlItem(i)">
                <button
                  type="submit"
                  class="ip-url-remove"
                  :disabled="importPreviewConfirming || importPreviewUrlImportBusy"
                  :aria-label="t('mode.library.preview.url_item_remove_aria_label', { url: item.url })"
                >
                  {{ t('mode.library.preview.url_item_remove') }}
                </button>
              </form>
            </li>
          </ul>
        </section>

      <template v-if="importPreview !== null">
        <!-- ═══════════════════ Tầng 1 — bảng mã (CÓ THÂN) ═══════════════════ -->
        <section class="ip-tier ip-tier-1" aria-labelledby="ip-tier-1-title">
          <h3 id="ip-tier-1-title" class="ip-tier-title">{{ t('mode.library.preview.tier1_title') }}</h3>

          <p class="ip-confidence-chip" :data-confidence="importPreview.confidence">
            <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
            {{ t(confidenceMessageKey(importPreview.confidence)) }}
            <span v-if="importPreviewSelectedCandidate !== null" class="ip-confidence-label">
              <!-- aura-allow-text: DỮ LIỆU (nhãn FR126 của ứng viên đang chọn). -->
              — {{ importPreviewSelectedCandidate.label }}
            </span>
          </p>

          <p
            v-if="!importPreviewStripIsOpen && importPreview.candidates.length > 0"
            class="ip-strip-hint"
          >
            {{ t('mode.library.preview.strip_open_hint') }}
            <button type="button" class="ip-strip-open" @click="dispatch('import.preview.open_picker')">
              {{ t('command.import.preview.open_picker') }}
            </button>
          </p>

          <template v-if="importPreviewStripIsOpen">
            <p class="ip-strip-hint">{{ t('mode.library.preview.strip_hint') }}</p>
            <div class="ip-strip" role="radiogroup" :aria-label="t('mode.library.preview.strip_hint')">
              <label
                v-for="candidate in importPreview.candidates"
                :key="candidate.encoding"
                class="ip-candidate"
                :class="{ 'ip-candidate-selected': importPreviewSelectedEncoding === candidate.encoding }"
              >
                <input
                  type="radio"
                  name="ip-candidate"
                  class="ip-candidate-radio"
                  :checked="importPreviewSelectedEncoding === candidate.encoding"
                  :disabled="importPreviewConfirming"
                  @change="onCandidateChange(candidate.encoding, $event)"
                />
                <!-- aura-allow-text: DỮ LIỆU (nhãn FR126, một trong năm chuỗi cố định). -->
                <span class="ip-candidate-label">{{ candidate.label }}</span>
                <span v-if="candidate.preview !== null" class="ip-candidate-preview">
                  <!-- aura-allow-text: DỮ LIỆU (bản dựng thật từ Rust, KHÔNG markup — AD-16). -->
                  {{ candidate.preview }}
                </span>
                <span v-else class="ip-candidate-preview ip-candidate-undecodable">
                  {{ t('mode.library.preview.candidate_undecodable') }}
                </span>
              </label>
            </div>
          </template>
        </section>

        <!-- ═══ Tầng chuẩn hoá — xuống dòng & khoảng trắng (CÓ THÂN, Story 6.4) ═══ -->
        <section class="ip-tier" aria-labelledby="ip-tier-normalized-title">
          <h3 id="ip-tier-normalized-title" class="ip-tier-title">
            {{ t('mode.library.preview.tier_normalized_title') }}
          </h3>

          <template v-if="importPreviewSelectedNormalized !== null">
            <p class="ip-normalized-counts">
              <!-- aura-allow-text: KẾT QUẢ của `t()`, tham số là DỮ LIỆU (số đếm từ Rust). -->
              {{
                t('mode.library.preview.tier_normalized_joined_lines', {
                  count: String(importPreviewSelectedNormalized.joined_lines),
                })
              }}
              ·
              {{
                t('mode.library.preview.tier_normalized_blank_lines_removed', {
                  count: String(importPreviewSelectedNormalized.blank_lines_removed),
                })
              }}
            </p>
            <p v-if="importPreviewSelectedNormalized.window_truncated" class="ip-normalized-window-note">
              {{ t('mode.library.preview.tier_normalized_window_truncated') }}
            </p>
            <p v-if="importPreviewSelectedNormalized.text !== ''" class="ip-normalized-text">
              <!-- aura-allow-text: DỮ LIỆU (bản dựng đã chuẩn hoá thật từ Rust, KHÔNG markup — AD-16). -->
              {{ importPreviewSelectedNormalized.text }}
            </p>
            <!-- Vá vòng rà 1, mục 2 — `text === ''` (cửa sổ không đủ MỘT dòng trọn vẹn, hoặc
                 nguồn chỉ toàn khoảng trắng) phải nói RA, không để lại một đoạn trắng câm. -->
            <p v-else class="ip-normalized-window-note">
              {{ t('mode.library.preview.tier_normalized_text_empty') }}
            </p>
          </template>
          <p v-else class="ip-tier-empty-reason">
            {{ t(normalizedTierEmptyMessageKey(normalizedTierEmptyReason())) }}
          </p>
        </section>

        <!--
          ═══════════ Tầng 2 — ranh giới nội dung (Story 6.9, FR123) ═══════════
          🔴 THAY TRỌN (2026-09-07, Story 6.9) — dãy khối cả trang thay cho một đoạn văn liền.
          Khối bị thuật toán loại VẪN hiện (đánh dấu "Đã loại") — `Space`/`[`/`]` cần chúng
          làm đối tượng để sửa bóc THIẾU, không riêng bóc THỪA (§Design Notes spec 6.9).
          ⚠️ Giới hạn thật KHÔNG đổi (`tier2_url_first_note`): dãy khối là của Chương ĐẦU TIÊN
          trong danh sách URL, không phải "mục đang chọn". Đường tệp/dán tay vẫn RỖNG — lý do
          nay là "nguồn này không bóc gì" (khoá `tier_empty_story_6_9` viết lại).
        -->
        <section
          class="ip-tier"
          :class="{ 'ip-tier-empty': importPreviewLastSubmittedFrom !== 'urls' }"
          aria-labelledby="ip-tier-2-title"
        >
          <h3 id="ip-tier-2-title" class="ip-tier-title">{{ t('mode.library.preview.tier2_title') }}</h3>
          <!-- aura-allow-text: KẾT QUẢ của `t()`, tham số là DỮ LIỆU (số đếm từ Rust). -->
          <p v-if="tier2Counts !== null" class="ip-tier2-counts">
            {{ t('mode.library.preview.tier2_counts', { kept: String(tier2Counts.kept), excluded: String(tier2Counts.excluded) }) }}
          </p>
          <template v-if="importPreviewLastSubmittedFrom === 'urls'">
            <p class="ip-normalized-window-note">{{ t('mode.library.preview.tier2_url_first_note') }}</p>
            <p
              v-if="importPreviewBlockRangeMissingStartNotice"
              class="ip-tier2-range-notice"
              role="status"
            >
              {{ t('mode.library.preview.tier2_range_missing_start') }}
            </p>
            <p v-if="importPreviewBlockActionError !== null" class="ip-tier2-range-notice" role="alert">
              <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
              {{ tError(importPreviewBlockActionError) }}
            </p>
            <ol
              v-if="importPreviewSelectedBlocks !== null && importPreviewSelectedBlocks.blocks.length > 0"
              ref="blocksList"
              class="ip-blocks"
              role="listbox"
              tabindex="-1"
              :aria-label="t('mode.library.preview.tier2_title')"
              :aria-activedescendant="blockDomId(importPreviewBlockFocusedIndex)"
            >
              <template v-for="(block, index) in importPreviewSelectedBlocks.blocks" :key="index">
                <!--
                  Mốc `[` — vạch "Đầu vùng giữ" đứng NGAY TRƯỚC khối tại `importPreviewBlockRangeStart`
                  (mục 9, vòng rà bước 4; bản dựng khoá `web-import.html:284`). CHỈ trên đường
                  URL (`v-if` cha đã chốt `importPreviewLastSubmittedFrom === 'urls'`) — mốc
                  không có nghĩa gì ở đường khác vì tầng 2 rỗng ở đó.
                -->
                <li v-if="importPreviewBlockRangeStart === index" class="ip-block-range-marker" role="presentation">
                  {{ t('mode.library.preview.tier2_range_start_marker') }}
                </li>
                <li
                  :id="blockDomId(index)"
                  class="ip-block"
                  :class="blockStateClass(block, index)"
                  role="option"
                  :aria-selected="index === importPreviewBlockFocusedIndex"
                >
                  <div class="ip-block-gutter" aria-hidden="true"></div>
                  <div class="ip-block-body">
                    <p class="ip-block-tag">{{ t(blockStateMessageKey(block)) }}</p>
                    <p v-if="block.body.kind === 'paragraph' || block.body.kind === 'caption'" class="ip-block-text">
                      <!-- aura-allow-text: DỮ LIỆU (thân khối đã bóc thật từ Rust, KHÔNG markup — AD-16). -->
                      {{ block.body.text }}
                    </p>
                    <p v-else class="ip-block-text">
                      <!-- aura-allow-text: DỮ LIỆU (alt/src ảnh, KHÔNG markup — AD-16). Story
                           6.9: mô hình chở nhánh Image, 0 chỗ gọi sản phẩm cho tới 6.11/6.13 —
                           hiện `alt`/`src` thô làm chỗ giữ chỗ. -->
                      {{ block.body.alt ?? block.body.src ?? '' }}
                    </p>
                  </div>
                </li>
              </template>
            </ol>
            <p v-else-if="importPreviewSelectedBlocks !== null" class="ip-tier-empty-reason">
              {{ t('mode.library.preview.tier2_empty_blocks') }}
            </p>
            <p v-else class="ip-tier-empty-reason">
              {{ t(cleanupTierEmptyMessageKey(normalizedTierEmptyReason())) }}
            </p>
          </template>
          <p v-else class="ip-tier-empty-reason">
            {{ t(tierEmptyMessageKey(importPreviewEmptyReasonForTier(2))) }}
          </p>
        </section>

        <!-- ═══════════════ Tầng 3 — luật làm sạch (CÓ THÂN, Story 6.5) ═══════════════════ -->
        <section ref="cleanupTierSection" class="ip-tier" tabindex="-1" aria-labelledby="ip-tier-3-title">
          <h3 id="ip-tier-3-title" class="ip-tier-title">{{ t('mode.library.preview.tier3_title') }}</h3>

          <template v-if="importPreviewSelectedCleanup !== null">
            <p v-if="importPreviewSelectedCleanup.window_truncated" class="ip-cleanup-window-note">
              {{ t('mode.library.preview.tier_cleanup_window_truncated') }}
            </p>
            <!-- AD-16: mảnh văn bản là DỮ LIỆU (điểm mã cắt từ Rust), không markup — viết
                 LIỀN, không khoảng trắng giữa các <span>, cùng khuôn `GridPanel.vue`. -->
            <p v-if="cleanupPieces.length > 0" class="ip-cleanup-text"
              ><!-- aura-allow-text: DỮ LIỆU (văn bản Chương thật, không phải chuỗi giao diện). --><span
                v-for="(piece, i) in cleanupPieces"
                :key="i"
                :class="{ 'ip-cleanup-struck': piece.struck }"
                >{{ piece.text }}</span
              ></p
            >
            <p v-else class="ip-cleanup-window-note">
              {{ t('mode.library.preview.tier_cleanup_text_empty') }}
            </p>

            <div v-if="importPreviewSelectedCleanup.rules.length > 0" class="ip-cleanup-rules">
              <div
                v-for="rule in importPreviewSelectedCleanup.rules"
                :key="`${rule.tier}-${rule.id}`"
                class="ip-cleanup-rule"
                :class="{ 'ip-cleanup-rule-off': !rule.enabled }"
              >
                <!--
                  🔴 KHÔNG một `@click` nào ở block này — `check:commands` Kiểm A đòi MỌI
                  `@click` là ĐÚNG MỘT `dispatch('<id>')`, và bốn thao tác dưới đây (sửa/lưu/
                  huỷ/xoá) đều mang THAM SỐ (`rule.tier`/`rule.id`) mà `dispatch()` không
                  nhận được (§Design Notes spec 6.3, cùng lý do dải bảng mã dùng `@change`
                  thay vì `dispatch`). Mỗi nút bọc trong một `<form>` riêng và kích hoạt qua
                  `@submit`/`@reset` — CẢ HAI đều NGOÀI phạm vi Kiểm A (script cổng liệt rõ:
                  "Chỉ @click. @keydown, @input, @change, @submit KHÔNG thuộc luật Kiểm A"),
                  và giữ nguyên khả năng bấm Enter/Space bằng bàn phím trên một `<button
                  type="submit">`/`type="reset"` bên trong `<form>` (khác `@mousedown`, thứ
                  sẽ mất lối vào bằng bàn phím).
                -->
                <template v-if="isEditingRule(rule)">
                  <form
                    class="ip-cleanup-edit-form"
                    @submit.prevent="onSaveEditCleanupRule"
                    @reset.prevent="onCancelEditCleanupRule"
                  >
                    <!-- 🔴 `<fieldset :disabled>` — MỘT thuộc tính HTML gốc khoá NGUYÊN khối
                      điều khiển con (khuôn `GlossaryQuickAdd.vue`, cùng lý do tại đó: Enter
                      không được đi qua một control còn BẬT trong lúc lượt LƯU đang bay). Cờ
                      RIÊNG `importPreviewCleanupSavingEdit` — không còn mượn
                      `importPreviewConfirming` (vòng rà 2026-09-06). -->
                    <fieldset
                      class="ip-cleanup-fieldset-inline"
                      :disabled="importPreviewConfirming || importPreviewCleanupSavingEdit"
                    >
                      <input v-model="editPattern" type="text" class="ip-cleanup-edit-pattern" />
                      <select v-model="editKind" class="ip-cleanup-edit-kind">
                        <option value="literal">{{ t('mode.library.preview.cleanup_kind_literal') }}</option>
                        <option value="regex">{{ t('mode.library.preview.cleanup_kind_regex') }}</option>
                      </select>
                      <button type="submit" class="ip-cleanup-save" :disabled="editPattern.trim() === ''">
                        {{ t('mode.library.preview.cleanup_save_rule') }}
                      </button>
                      <button type="reset" class="ip-cleanup-cancel">
                        {{ t('mode.library.preview.cleanup_cancel_edit') }}
                      </button>
                    </fieldset>
                  </form>
                </template>
                <template v-else>
                  <!-- 🔴 `aria-label` nêu MẪU luật — vòng rà 2026-09-06: một `<input
                    type="checkbox">` trần đọc ra trợ năng là "checkbox", không nói bật/tắt
                    LUẬT NÀO khi trang có nhiều hàng. `id`/`for` không dùng được ở đây (mẫu
                    luật không phải một định danh HTML hợp lệ), nên gắn thẳng `aria-label` lên
                    chính ô tick — khuôn tối thiểu, không cần một `<label>` bọc ngoài. -->
                  <input
                    type="checkbox"
                    class="ip-cleanup-tick"
                    :checked="rule.enabled"
                    :disabled="importPreviewConfirming || importPreviewCleanupToggling"
                    :aria-label="t('mode.library.preview.cleanup_toggle_aria_label', { pattern: rule.pattern })"
                    @change="onToggleCleanupRule(rule, $event)"
                  />
                  <div class="ip-cleanup-rule-body">
                    <!-- aura-allow-text: DỮ LIỆU (mẫu luật do người dùng tự soạn). -->
                    <p class="ip-cleanup-pattern">{{ rule.pattern }}</p>
                    <p class="ip-cleanup-meta">
                      {{
                        t('mode.library.preview.cleanup_rule_meta', {
                          chapter_count: String(rule.count_in_chapter),
                          import_count: String(rule.count_in_import),
                        })
                      }}
                    </p>
                  </div>
                  <span class="ip-cleanup-tier">{{ t(cleanupTierLabelKey(rule.tier)) }}</span>
                  <form class="ip-cleanup-action-form" @submit.prevent="onStartEditCleanupRule(rule)">
                    <button
                      type="submit"
                      class="ip-cleanup-edit"
                      :disabled="importPreviewConfirming || importPreviewCleanupDeleting"
                    >
                      {{ t('mode.library.preview.cleanup_edit_rule') }}
                    </button>
                  </form>
                  <!-- 🔴 HAI NHỊP — vòng rà 2026-09-06 (khuôn `GlossaryManageOverlay.vue`,
                    `manageDeletePending`): xoá một luật KHÔNG HOÀN TÁC ĐƯỢC, nên nút này đổi
                    nhãn/`aria-label` sang "bấm lại để xoá thật" ở nhịp hai, cùng CHỖ GỌI
                    (`onDeleteCleanupRule`/`deleteImportPreviewCleanupRule` tự phân biệt hai
                    nhịp bằng khoá `(tier, id)`, không phải hai command khác nhau). -->
                  <form class="ip-cleanup-action-form" @submit.prevent="onDeleteCleanupRule(rule)">
                    <button
                      type="submit"
                      class="ip-cleanup-delete"
                      :class="{ 'ip-cleanup-delete-pending': isDeletePending(rule) }"
                      :disabled="importPreviewConfirming || importPreviewCleanupDeleting"
                      :aria-label="t(cleanupDeleteAriaLabelKey(isDeletePending(rule)), { pattern: rule.pattern })"
                    >
                      {{ t(cleanupDeleteLabelKey(isDeletePending(rule))) }}
                    </button>
                  </form>
                </template>
              </div>
            </div>

            <form class="ip-cleanup-add" @submit.prevent="onAddCleanupRule">
              <!-- KHÔNG ô chọn tầng — bề mặt này chỉ tạo luật tầng Toàn cục (xem doc-comment
                `onAddCleanupRule`). -->
              <fieldset
                class="ip-cleanup-fieldset-inline"
                :disabled="importPreviewConfirming || importPreviewCleanupAdding"
              >
                <input
                  v-model="newRulePattern"
                  type="text"
                  class="ip-cleanup-add-pattern"
                  :placeholder="t('mode.library.preview.cleanup_add_pattern_placeholder')"
                />
                <select v-model="newRuleKind" class="ip-cleanup-add-kind">
                  <option value="literal">{{ t('mode.library.preview.cleanup_kind_literal') }}</option>
                  <option value="regex">{{ t('mode.library.preview.cleanup_kind_regex') }}</option>
                </select>
                <button type="submit" class="ip-cleanup-add-submit" :disabled="newRulePattern.trim() === ''">
                  {{ t('mode.library.preview.cleanup_add_rule') }}
                </button>
              </fieldset>
            </form>

            <p v-if="importPreviewCleanupActionError !== null" class="ip-cleanup-error" role="alert">
              <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
              {{ tError(importPreviewCleanupActionError) }}
            </p>
          </template>
          <p v-else class="ip-tier-empty-reason">
            {{ t(cleanupTierEmptyMessageKey(normalizedTierEmptyReason())) }}
          </p>
        </section>

        <!-- ═══════════════ Tầng 4 — tách Chương (CÓ THÂN, Story 6.6) ═══════════════════ -->
        <section class="ip-tier" aria-labelledby="ip-tier-4-title">
          <h3 id="ip-tier-4-title" class="ip-tier-title">{{ t('mode.library.preview.tier4_title') }}</h3>

          <template v-if="importPreviewSelectedChapters !== null">
            <form class="ip-chapters-pattern-row" @submit.prevent>
              <!-- 🔴 KHÔNG `@click` — `@change` mang tham số (giá trị ô/kind), cùng lý do
                   dải bảng mã và luật làm sạch dùng `@change` thay vì `dispatch()` trần
                   (§Design Notes spec 6.3). -->
              <label class="ip-chapters-pattern-label">
                {{ t('mode.library.preview.chapters_pattern_label') }}
                <input
                  v-model="chapterPatternDraft"
                  type="text"
                  class="ip-chapters-pattern-input"
                  :placeholder="t('mode.library.preview.chapters_pattern_placeholder')"
                  :disabled="importPreviewConfirming || importPreviewChapterPatternSending"
                  @change="onChapterPatternChange"
                />
              </label>
              <select
                v-model="chapterPatternKindDraft"
                class="ip-chapters-pattern-kind"
                :disabled="importPreviewConfirming || importPreviewChapterPatternSending"
                @change="onChapterPatternChange"
              >
                <option value="literal">{{ t('mode.library.preview.cleanup_kind_literal') }}</option>
                <option value="regex">{{ t('mode.library.preview.cleanup_kind_regex') }}</option>
              </select>
            </form>

            <p v-if="importPreviewChapterPatternError !== null" class="ip-chapters-error" role="alert">
              <!-- aura-allow-text: KẾT QUẢ của `tError()`. §I/O Matrix spec 6.6: mẫu hỏng GIỮ
                   NGUYÊN danh sách bên dưới, chỉ thêm dòng báo lỗi này. -->
              {{ tError(importPreviewChapterPatternError) }}
            </p>

            <p class="ip-chapters-count">
              <!-- aura-allow-text: KẾT QUẢ của `t()`, tham số là DỮ LIỆU (số đếm từ Rust). -->
              {{ t('mode.library.preview.chapters_count', { count: String(importPreviewSelectedChapters.chapter_count) }) }}
            </p>

            <label class="ip-chapters-sort-toggle">
              <input
                type="checkbox"
                :checked="chapterSortByLength"
                @change="onToggleChapterSortByLength($event)"
              />
              {{ t('mode.library.preview.chapters_sort_by_length') }}
            </label>

            <ul class="ip-chapters-list">
              <template v-if="chapterSortByLength">
                <li v-for="entry in chapterEntriesSortedByLength" :key="entry.ord" class="ip-chapters-entry">
                  <!-- aura-allow-text: DỮ LIỆU (số thứ tự Chương từ Rust, KHÔNG markup — AD-16). -->
                  <span class="ip-chapters-ord">{{ entry.ord }}</span>
                  <span v-if="entry.title !== null" class="ip-chapters-title">
                    <!-- aura-allow-text: DỮ LIỆU (dòng khớp mẫu, KHÔNG markup — AD-16). -->
                    {{ entry.title }}
                  </span>
                  <span v-else class="ip-chapters-title ip-chapters-title-none">
                    {{ t('mode.library.preview.chapters_no_title') }}
                  </span>
                  <span class="ip-chapters-length">
                    {{ t('mode.library.preview.chapters_length', { count: String(entry.length) }) }}
                  </span>
                </li>
              </template>
              <template v-else>
                <li v-for="entry in chapterEntriesDefaultWindow.first" :key="entry.ord" class="ip-chapters-entry">
                  <!-- aura-allow-text: DỮ LIỆU (số thứ tự Chương từ Rust, KHÔNG markup — AD-16). -->
                  <span class="ip-chapters-ord">{{ entry.ord }}</span>
                  <span v-if="entry.title !== null" class="ip-chapters-title">
                    <!-- aura-allow-text: DỮ LIỆU (dòng khớp mẫu, KHÔNG markup — AD-16). -->
                    {{ entry.title }}
                  </span>
                  <span v-else class="ip-chapters-title ip-chapters-title-none">
                    {{ t('mode.library.preview.chapters_no_title') }}
                  </span>
                  <span class="ip-chapters-length">
                    {{ t('mode.library.preview.chapters_length', { count: String(entry.length) }) }}
                  </span>
                </li>
                <li v-if="chapterEntriesDefaultWindow.showEllipsis" class="ip-chapters-ellipsis" aria-hidden="true">⋯</li>
                <li v-for="entry in chapterEntriesDefaultWindow.last" :key="entry.ord" class="ip-chapters-entry">
                  <!-- aura-allow-text: DỮ LIỆU (số thứ tự Chương từ Rust, KHÔNG markup — AD-16). -->
                  <span class="ip-chapters-ord">{{ entry.ord }}</span>
                  <span v-if="entry.title !== null" class="ip-chapters-title">
                    <!-- aura-allow-text: DỮ LIỆU (dòng khớp mẫu, KHÔNG markup — AD-16). -->
                    {{ entry.title }}
                  </span>
                  <span v-else class="ip-chapters-title ip-chapters-title-none">
                    {{ t('mode.library.preview.chapters_no_title') }}
                  </span>
                  <span class="ip-chapters-length">
                    {{ t('mode.library.preview.chapters_length', { count: String(entry.length) }) }}
                  </span>
                </li>
              </template>
            </ul>
          </template>
          <p v-else class="ip-tier-empty-reason">
            {{ t(chaptersTierEmptyMessageKey(normalizedTierEmptyReason())) }}
          </p>
        </section>
      </template>
      <!--
        `importPreview === null` — chỉ chạm được qua nhánh URL còn mục hỏng/danh sách rỗng
        (§Always spec 6.7: `sync_pending_from_url_items` phía Rust dọn `PendingImportSourceState`
        đúng lúc này). Bốn tầng KHÔNG hiện — không có gì hợp lệ để mà xem trước — nhưng danh
        sách mục-theo-link ở TRÊN vẫn hiện, và người dùng SỬA được nó (bỏ/tải lại) mà không
        cần đóng lớp phủ.
      -->
      <p v-else class="ip-tier-empty-reason ip-url-locked-reason" role="status">
        {{ t('mode.library.preview.url_list_locked') }}
      </p>

      <p v-if="importPreviewConfirmError !== null" class="ip-status ip-error" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(importPreviewConfirmError) }}
      </p>
      <p v-else-if="importPreviewConfirming" class="ip-status" role="status">
        {{ t('mode.library.preview.confirming') }}
      </p>

      <!--
        🔴 Story 6.8 (NFR19) — dòng tóm tắt nhật ký domain. ĐẶT Ở ĐÂY, NGOÀI khối bốn tầng
        (khối đó đóng ở `</template>` phía trên, xem doc-comment §Code Map spec 6.8): một
        mục hỏng làm bốn tầng biến mất (`importPreview === null`), nhưng đây chính là lúc
        mạng đã bị gọi NHIỀU NHẤT — dòng này phải sống sót qua chính ca đó. `v-if > 0`: một
        lượt dán chưa bấm tải chưa hề gọi mạng, chân màn không được có dòng domain nào
        (I/O Matrix spec 6.8, hàng 1).
      -->
      <p v-if="importPreviewDomainLogDomainCount > 0" class="ip-domain-log-summary" role="status">
        {{ t('mode.library.preview.domain_log_summary', { count: String(importPreviewDomainLogDomainCount) }) }}
        <button type="button" class="ip-domain-log-view" @click="dispatch('settings.privacy.open')">
          {{ t('mode.library.preview.domain_log_view') }}
        </button>
      </p>

      <p class="ip-hint">{{ t('mode.library.preview.hint_no_write_before_confirm') }}</p>

      <div class="ip-actions">
        <button
          type="button"
          class="ip-act ip-act-primary"
          :disabled="importPreviewConfirming || importPreview === null || importPreviewUrlImportBusy"
          @click="dispatch('import.preview.confirm')"
        >
          {{ t('command.import.preview.confirm') }}
        </button>
        <button
          type="button"
          class="ip-act"
          :disabled="importPreviewConfirming"
          @click="dispatch('import.preview.cancel')"
        >
          {{ t('command.import.preview.cancel') }}
        </button>
      </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.ip-scrim {
  position: fixed;
  inset: 0;
  z-index: 11; /* aura-allow-z-index: cùng tầng GlossaryImportOverlay — hai lớp phủ không mở đồng thời. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.ip-panel {
  width: 100%;
  max-width: 760px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.ip-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.ip-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.ip-close {
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.ip-close:disabled {
  cursor: default;
}

.ip-status {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-empty {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.ip-error {
  color: var(--color-error);
}

.ip-tier {
  margin: 0 0 var(--space-panel-block) 0;
  padding-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

.ip-tier-title {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong, var(--weight-ui-md));
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.ip-confidence-chip {
  display: inline-flex;
  gap: calc(var(--space-unit) * 1);
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
}

.ip-strip-hint {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-strip-open {
  margin-left: calc(var(--space-unit) * 1);
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-strip {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: calc(var(--space-unit) * 2);
}

.ip-candidate {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  padding: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  cursor: pointer;
}

.ip-candidate-selected {
  border-color: var(--color-primary);
}

.ip-candidate-radio {
  align-self: flex-start;
}

.ip-candidate-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong, var(--weight-ui-md));
  color: var(--color-on-surface);
}

/* 🔴 §Always spec 6.3 — mẫu chữ trong dải đặt ở CỠ `read`, không cỡ giao diện. */
.ip-candidate-preview {
  font-family: var(--face-read-md);
  font-size: var(--font-read-md);
  line-height: var(--leading-read-md);
  color: var(--color-on-surface);
  word-break: break-word;
}

.ip-candidate-undecodable {
  color: var(--color-on-surface-variant);
}

/* Vá vòng rà 1, mục 7 — hai luật trùng nguyên văn (`.ip-normalized-counts` ·
   `.ip-normalized-window-note`) gộp làm một; `.ip-normalized-window-note` còn dùng lại cho
   nhánh "text rỗng" (cùng cỡ chữ, cùng vai trò: một dòng chú thích phụ dưới hai số đếm). */
.ip-normalized-counts,
.ip-normalized-window-note {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

/* 🔴 §Always spec 6.4 — cùng luật tầng bảng mã: mẫu chữ đã chuẩn hoá ở CỠ `read`. */
.ip-normalized-text {
  margin: 0;
  font-family: var(--face-read-md);
  font-size: var(--font-read-md);
  line-height: var(--leading-read-md);
  color: var(--color-on-surface);
  word-break: break-word;
  white-space: pre-wrap;
}

/* ── Story 6.9 — tầng 2, dãy khối cả trang (FR123) ──────────────────────────────── */

.ip-tier2-counts {
  margin: calc(var(--space-unit) * -1) 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-tier2-range-notice {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.ip-blocks {
  margin: 0;
  padding: 0;
  list-style: none;
  border: 1px solid var(--color-outline);
}

/* Mốc `[` — vạch "Đầu vùng giữ" (mục 9, vòng rà bước 4). Cùng chữ `.ip-tier2-range-notice`
   (một cảnh báo/hướng dẫn tầng 2), khác MÀU — đây KHÔNG phải lỗi, dùng `primary` (cùng tông
   tiêu điểm bàn phím `.ip-block-focused`) thay `error`. */
.ip-block-range-marker {
  padding: calc(var(--space-unit) * 0.5) calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-primary);
  border-bottom: 1px solid var(--color-outline);
}

/* 🔴 §Always spec 6.9 — tiêu điểm bàn phím dùng `primary` qua `border-left`, KHÔNG
   `box-shadow` (Kiểm F cấm tuyệt đối, không miễn trừ). Vạch TRẠNG THÁI (giữ/loại) sống RIÊNG
   ở `.ip-block-gutter` — hai tín hiệu độc lập, một khối có thể vừa `dropped` vừa đang giữ
   tiêu điểm. */
.ip-block {
  display: flex;
  border-bottom: 1px solid var(--color-outline);
  border-left: 2px solid transparent;
}

.ip-block:last-child {
  border-bottom: none;
}

.ip-block-focused {
  border-left-color: var(--color-primary);
}

/* Vạch TRẠNG THÁI — cùng ngữ pháp vạch lề segment của `GridPanel.vue`
   (`confirmed`/`tm-rule`/`ornament`, EXPERIENCE.md:89-92). Mặc định "giữ · máy đoán". */
.ip-block-gutter {
  flex: none;
  /* ⚠️ KHÔNG `var(--space-gutter-width)` — spine đo được (`panels/GridPanel.vue:1795`) rằng
     khoá `spacing.gutter-width` của `tokens.json` KHÔNG được `check:tokens` đối chiếu với
     `var()` thật (một tham chiếu sai tên là CSS CHẾT ÂM THẦM, 0 cổng canh) — dùng thẳng
     `--space-unit` đã xác nhận CÓ tồn tại thay vì tin một tên chưa ai kiểm. */
  width: calc(var(--space-unit) * 2);
  background-color: var(--color-tm-rule);
}

.ip-block-kept-confirmed .ip-block-gutter {
  background-color: var(--color-confirmed);
}

.ip-block-dropped .ip-block-gutter {
  background-color: var(--color-ornament);
}

.ip-block-body {
  flex: 1 1 auto;
  min-width: 0;
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
}

/* §Always spec 6.9 — "phân biệt bằng ĐỘ LÙI, không màu nhấn thứ hai" (EXPERIENCE.md:95):
   khối loại chìm xuống `surface-sunken`, chữ rút về `on-surface-variant`. */
.ip-block-dropped {
  background: var(--color-surface-sunken);
}

.ip-block-tag {
  margin: 0 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
}

.ip-block-text {
  margin: 0;
  font-family: var(--face-read-md);
  font-size: var(--font-read-md);
  line-height: var(--leading-read-md);
  color: var(--color-on-surface);
  word-break: break-word;
  white-space: pre-wrap;
}

/* §Always spec 6.9 — khối đã loại đổi CẢ thang chữ (đọc → giao diện), không chỉ màu. Token
   `ui-md-wrap` — token họ `ui` DUY NHẤT khai `wraps: true`, đúng vai cho thân khối nhiều
   dòng (KHÔNG `ui-label`, 11px/`wraps: false`, sai vai — xem doc-comment Task list spec 6.9). */
.ip-block-dropped .ip-block-text {
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

/* ── Story 6.5 — tầng 3, luật làm sạch ──────────────────────────────────────────── */

.ip-cleanup-window-note {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

/* 🔴 §Always spec 6.5 — cùng luật tầng bảng mã/chuẩn hoá: mẫu chữ ở CỠ `read`. */
.ip-cleanup-text {
  margin: 0 0 calc(var(--space-unit) * 3) 0;
  font-family: var(--face-read-md);
  font-size: var(--font-read-md);
  line-height: var(--leading-read-md);
  color: var(--color-on-surface);
  word-break: break-word;
  white-space: pre-wrap;
}

/* 🔴 `ornament` là màu của NÉT, KHÔNG BAO GIỜ là màu của chữ (`DESIGN.md:213`,
   `check:tokens` `neverTextTokens`, đo 2,44/2,64 — trượt AA). Gạch ngang dùng
   `text-decoration-color`, KHÔNG `color` — đúng mockup `web-import.html:103`. */
.ip-cleanup-struck {
  color: var(--color-on-surface-variant);
  text-decoration-line: line-through;
  text-decoration-color: var(--color-ornament);
  text-decoration-thickness: 1px;
}

.ip-cleanup-rules {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
  margin: 0 0 calc(var(--space-unit) * 3) 0;
}

.ip-cleanup-rule {
  display: flex;
  align-items: flex-start;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
}

/* `<form>` bọc riêng mỗi thao tác (xem chú thích tại template) — `display: contents` để
   thẻ `<form>` không tự thành một hộp bố cục, giữ nút BÊN TRONG cư xử như một con trực
   tiếp của flex `.ip-cleanup-rule`. */
.ip-cleanup-action-form,
.ip-cleanup-edit-form {
  display: contents;
}

.ip-cleanup-rule-off {
  background: var(--color-surface-sunken);
}

.ip-cleanup-tick {
  margin-top: calc(var(--space-unit) * 0.5);
  flex: none;
}

.ip-cleanup-rule-body {
  flex: 1;
  min-width: 0;
}

/* 🔴 Mẫu luật là dữ liệu người dùng tự soạn (regex/chuỗi con) — cỡ `mono` để phân biệt CHỮ
   THẬT với CÚ PHÁP MẪU, đúng vai trò cột "rx" của mockup. */
.ip-cleanup-pattern {
  margin: 0 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
  word-break: break-all;
}

.ip-cleanup-rule-off .ip-cleanup-pattern {
  color: var(--color-on-surface-variant);
}

.ip-cleanup-meta {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-cleanup-tier {
  flex: none;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  line-height: var(--leading-ui-label);
  font-weight: var(--weight-ui-md-strong, var(--weight-ui-md));
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
  border: 1px solid var(--color-outline);
  padding: 0 calc(var(--space-unit) * 1);
}

.ip-cleanup-edit,
.ip-cleanup-delete,
.ip-cleanup-save,
.ip-cleanup-cancel {
  flex: none;
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-cleanup-edit:disabled,
.ip-cleanup-delete:disabled,
.ip-cleanup-save:disabled,
.ip-cleanup-cancel:disabled {
  cursor: default;
}

/* Nhịp HAI của xoá (khuôn `GlossaryManageOverlay.vue::gm-act-danger`) — cùng token báo lỗi
   cho một thao tác không hoàn tác được, đang chờ bấm lại để xác nhận. */
.ip-cleanup-delete-pending {
  color: var(--color-error);
  border-color: var(--color-error);
}

/* `<fieldset>` bọc nhóm điều khiển của mỗi form CRUD (khuôn `GlossaryQuickAdd.vue` —
   `:disabled` trên CHÍNH `<fieldset>` khoá nguyên khối bằng một thuộc tính HTML gốc, không
   lặp `:disabled` trên từng ô). Reset toàn bộ khung mặc định của trình duyệt VÀ giữ
   `display: contents` để nó không tự thành một hộp bố cục — các control bên trong vẫn cư xử
   như con trực tiếp của flex `.ip-cleanup-rule`/`.ip-cleanup-add`, đúng lý do `.ip-cleanup-action-form`/
   `.ip-cleanup-edit-form` đã dùng `display: contents` ở trên. */
.ip-cleanup-fieldset-inline {
  display: contents;
  margin: 0;
  padding: 0;
  border: 0;
  min-width: 0;
}

.ip-cleanup-edit-pattern {
  flex: 1;
  min-width: 0;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1);
}

.ip-cleanup-edit-kind,
.ip-cleanup-add-kind {
  flex: none;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
}

.ip-cleanup-add {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  margin: 0 0 calc(var(--space-unit) * 2) 0;
}

.ip-cleanup-add-pattern {
  flex: 1;
  min-width: 12ch;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1);
}

.ip-cleanup-add-submit {
  flex: none;
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
}

.ip-cleanup-add-submit:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.ip-cleanup-error {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

/* ── Story 6.6 — tầng 4, tách Chương ────────────────────────────────────────────── */

.ip-chapters-pattern-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  margin: 0 0 calc(var(--space-unit) * 2) 0;
}

.ip-chapters-pattern-label {
  display: flex;
  flex: 1;
  min-width: 12ch;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-chapters-pattern-input {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1);
}

.ip-chapters-pattern-kind {
  flex: none;
  align-self: flex-end;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
}

.ip-chapters-error {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.ip-chapters-count {
  margin: 0 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-chapters-sort-toggle {
  display: inline-flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
  cursor: pointer;
}

.ip-chapters-list {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin: 0;
  padding: 0;
  list-style: none;
}

.ip-chapters-entry {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
}

.ip-chapters-ord {
  flex: none;
  min-width: 3ch;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface-variant);
  text-align: right;
}

.ip-chapters-title {
  flex: 1;
  min-width: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ip-chapters-title-none {
  color: var(--color-on-surface-variant);
}

.ip-chapters-length {
  flex: none;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  color: var(--color-on-surface-variant);
}

.ip-chapters-ellipsis {
  text-align: center;
  color: var(--color-on-surface-variant);
}

.ip-tier-empty-reason {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

/* Story 6.7 — danh sách mục-theo-link. Cùng khuôn `.ip-chapters-*` ngay trên. */
.ip-url-items {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin: 0;
  padding: 0;
  list-style: none;
}

.ip-url-item {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
}

.ip-url-item-broken {
  border-color: var(--color-error);
}

.ip-url-position {
  flex: none;
  min-width: 3ch;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface-variant);
  text-align: right;
}

.ip-url-address {
  flex: 1;
  min-width: 20ch;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ip-url-ok {
  flex: none;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  color: var(--color-on-surface-variant);
}

.ip-url-reason {
  flex: none;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-error);
}

.ip-url-action-form {
  display: contents;
}

.ip-url-reload,
.ip-url-remove {
  flex: none;
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-url-reload:disabled,
.ip-url-remove:disabled {
  cursor: default;
}

.ip-url-locked-reason {
  color: var(--color-error);
}

.ip-hint {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-domain-log-summary {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

/* Khuôn mockup `web-import.html:393` — "xem" gạch chân, KHÔNG một màu nhấn thứ hai
   (`check:tokens` Kiểm D cấm màu phân loại/`opacity` trung gian, không miễn trừ). */
.ip-domain-log-view {
  padding: 0;
  margin-left: calc(var(--space-unit) * 1);
  background: none;
  border: none;
  cursor: pointer;
  font-family: inherit;
  font-size: inherit;
  color: inherit;
  text-decoration: underline;
}

.ip-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}

.ip-act {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.ip-act-primary {
  background: var(--color-primary);
  color: var(--color-on-primary);
  border-color: var(--color-primary);
}

.ip-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}
</style>
