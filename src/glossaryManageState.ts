/**
 * State của lớp phủ **Quản lý Glossary** — Story 3.9, FR49.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY (`src/`, không `src/panels/`)
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `glossaryQueueState.ts`/`glossarySettingsState.ts`: lớp phủ không phải một
 * panel của một chế độ cụ thể — nó nói về CẢ Glossary (hai tầng, không riêng Tác phẩm đang
 * mở), đúng khuôn `GlossaryQueueOverlay.vue`.
 *
 * ⚠️ Tệp này dùng `ref`/`computed` của Vue **và** gọi `@tauri-apps/api` xuyên qua
 * `config/glossary.ts` — KHÔNG được `import` vào `src/commands/index.ts` (Kiểm C/D/E của
 * `npm run check:commands` nạp tệp đó bằng Node thuần). Chín handler được TIÊM VÀO qua
 * `CommandDeps`, nối thật ở `src/main.ts`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 LỌC/TÌM/SẮP CHẠY Ở FRONTEND, TRÊN DANH SÁCH ĐÃ NẠP — 0 round-trip IPC mỗi lần gõ
 * ─────────────────────────────────────────────────────────────────────────────
 * `listGlossaryEntries()` nạp TRỌN cả hai tầng một lần lúc mở lớp phủ; ba bộ lọc (phân
 * loại · xuất xứ · trạng thái chốt) và ô tìm chạy trong bộ nhớ trên `rows` đã nạp (§Design
 * Notes của spec). Sau một lượt XOÁ hoặc ĐẨY TẦNG thành công, `rows` được NẠP LẠI TRỌN VẸN
 * (không patch một phần tử tại chỗ) — lý do là `is_shadowed` của các hàng KHÁC có thể đổi
 * theo (xoá mục Work đang che một mục Global làm mục Global đó hết bị che), và chỉ Rust mới
 * tính lại đúng cờ đó (AD-1, AD-18) — patch tại chỗ ở đây sẽ để `is_shadowed` của một hàng
 * KHÔNG liên quan tới thao tác vừa làm đứng sai. SỬA (edit) không đổi `is_shadowed` của bất
 * kỳ hàng nào khác, nên nó patch tại chỗ, không nạp lại.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 "CHƯA MỞ TÁC PHẨM" KHÔNG PHẢI MỘT CA RỖNG — nó là một GHI CHÚ đi kèm danh sách
 * ─────────────────────────────────────────────────────────────────────────────
 * Bốn ca rỗng của §Always (chưa nạp · cầu IPC vắng · Glossary trống thật · bộ lọc không
 * khớp) KHÔNG bao gồm "chưa mở Tác phẩm" — `listGlossaryEntries()` (chưa mở Tác phẩm) trả
 * về đúng mục tầng Global, một danh sách CÓ THỂ khác rỗng. `workTierAvailable` là một cờ
 * RIÊNG (đúng khuôn `quickAddWorkTierAvailable`/`queueEmptyReasonFor`'s `no_work`), đọc qua
 * MỘT lượt `lookupGlossaryTerm('')` bổ sung (adapter đã có từ Story 3.3, không một vỏ IPC
 * mới) — tiền lệ `glossaryQueueState.ts::openGlossaryQueue`.
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import {
  deleteGlossaryTerm,
  exportGlossaryTier,
  listGlossaryEntries,
  lookupGlossaryTerm,
  promoteGlossaryTermToGlobal,
  updateGlossaryTerm,
} from './config/glossary'
import type { GlossaryCategory, GlossaryEntry, GlossaryTierWire } from './config/glossary'
import type { IpcError } from './i18n'
import { editorChapterId, editorSegments } from './panels/editorPanelState'
import { refreshGlossaryMarks } from './panels/glossaryMarksState'
import { sourceChapter } from './panels/sourcePanelState'

/**
 * 🔴 Vị từ *"…HasLoaded"* mở rộng — Known pitfall trung tâm của dự án. BỐN trạng thái:
 * - `'unknown'` — chưa nạp lần nào (lớp phủ vừa mở, lượt tra đang bay);
 * - `'ipc_unavailable'` — cầu IPC vắng (chạy ngoài Tauri) — KHÔNG một lỗi;
 * - `'error'` — lượt tải trượt THẬT (một `IpcError` đọc được qua [`manageLoadError`]);
 * - `'loaded'` — đã tải xong (`manageRows` có thể rỗng == Glossary trống thật, hoặc bộ lọc
 *   không khớp gì — hai ca đó phân biệt qua [`manageEmptyReasonFor`]).
 */
export type GlossaryManageStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'loaded'

/** Ba trạng thái chốt để lọc — `'all'` là mặc định (không lọc theo trục này). */
export type GlossaryManageConfirmedFilter = 'all' | 'confirmed' | 'pending'

/** Ba xuất xứ (khớp `TermOrigin::as_str()` phía Rust) cộng `'all'`. */
export type GlossaryManageOriginFilter = 'all' | 'manual' | 'import_scan' | 'review_harvest'

// ─────────────────────────────────────────────────────────────────────────────
// State module-level — singleton của cả tiến trình, cùng khuôn glossaryQueueState.ts. Mỗi
// khai báo NẰM TRÊN MỘT DÒNG (`check:panel-refs` Kiểm 5 — cú pháp ngoài tập con cho ĐỎ).
// ─────────────────────────────────────────────────────────────────────────────
const overlayOpen = ref(false)
const status = ref<GlossaryManageStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const rows = ref<GlossaryEntry[]>([])
const searchQuery = ref('')
const categoryFilterState = ref<GlossaryCategory | 'all'>('all')
const originFilterState = ref<GlossaryManageOriginFilter>('all')
const confirmedFilterState = ref<GlossaryManageConfirmedFilter>('all')
const cursor = ref(0)
const editing = ref(false)
const editTranslationInput = ref('')
const editNoteInput = ref('')
const editCategoryInput = ref<GlossaryCategory>('other')
const saving = ref(false)
/**
 * Thao tác nào đang bay — `saving` một mình KHÔNG đủ.
 *
 * 🔵 **THÊM 2026-08-24 (vòng rà ba lớp).** Cả ba lượt ghi (`save` · `delete` · `promote`)
 * cùng bật `saving`, mà lớp phủ chỉ có MỘT câu trạng thái `glossary.manage.saving`
 * (*"Đang lưu…"*) — nên xoá một mục thì màn hình nói đang lưu, và đẩy tầng cũng thế. Một câu
 * nói sai việc đang làm còn tệ hơn không câu nào: nó là thứ người dùng tin.
 */
const savingAction = ref<'save' | 'delete' | 'promote'>('save')
const actionError = ref<IpcError | null>(null)
const actionNotice = ref<'promote_not_applicable' | null>(null)
const workTierAvailable = ref(false)
/**
 * 🔵 **THÊM 2026-08-25 (Story 3.10b, AD-48).** Tầng đang chọn cho lượt Xuất/Nhập CSV —
 * dùng CHUNG cho cả hai nút, vì Xuất và mở-hộp-thoại-Nhập đều cần biết tầng TRƯỚC khi
 * `dispatch('<id>')` chạy (`dispatch` không nhận tham số, `check:commands` Kiểm A). Đây
 * là chỗ DUY NHẤT nắm giữ lựa chọn đó — handler tiêm ở `main.ts` đọc `ref` này, không
 * nhận nó qua đối số.
 */
const exchangeTierState = ref<GlossaryTierWire>('global')
/** Thao tác Xuất đang bay — cùng khuôn `saving`, nhưng RIÊNG vì Xuất/Nhập không đổi hàng
 * nào trong `rows` (không cần khoá form Sửa). */
const exportBusy = ref(false)
const exportError = ref<IpcError | null>(null)
/** Đường dẫn vừa ghi — hiện câu "đã ghi vào …" (`role="status"`). `null` == chưa xuất lần
 * nào kể từ lúc mở lớp phủ, HOẶC lượt xuất gần nhất bị HUỶ hoặc TRƯỢT (không lỗi, không
 * câu) — xoá TRƯỚC MỖI lượt xuất mới, không chỉ khi lượt mới thành công (P3, vòng rà ba
 * lớp 2026-08-25: bản trước giữ nguyên đường dẫn CŨ qua một lượt huỷ, nên huỷ sau một lượt
 * xuất thành công đọc như thể lượt huỷ đó CŨNG đã ghi). */
const exportedPath = ref<string | null>(null)
/** 🔵 THÊM (P2, cùng vòng rà) — không có cầu IPC (chạy ngoài Tauri). Tách khỏi "huỷ hộp
 * thoại" (không status nào, im lặng CÓ CHỦ) — đây PHẢI nói ra, cùng lý do
 * `glossaryImportState.ts::GlossaryImportStatus`. */
const exportIpcUnavailable = ref(false)

/** Số thứ tự lượt mở/thao tác — chặn một lượt CŨ ghi đè lên state của một lượt MỚI hơn (đua
 * round-trip IPC), cùng khuôn `sequence` của các state Glossary khác. */
let sequence = 0

export const manageOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const manageStatus: DeepReadonly<Ref<GlossaryManageStatus>> = readonly(status)
export const manageLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const manageSearchQuery: DeepReadonly<Ref<string>> = readonly(searchQuery)
export const manageCategoryFilter: DeepReadonly<Ref<GlossaryCategory | 'all'>> = readonly(categoryFilterState)
export const manageOriginFilter: DeepReadonly<Ref<GlossaryManageOriginFilter>> = readonly(originFilterState)
export const manageConfirmedFilter: DeepReadonly<Ref<GlossaryManageConfirmedFilter>> = readonly(confirmedFilterState)
export const manageCursor: DeepReadonly<Ref<number>> = readonly(cursor)
export const manageEditing: DeepReadonly<Ref<boolean>> = readonly(editing)
export const manageEditTranslation: Ref<string> = editTranslationInput
export const manageEditNote: Ref<string> = editNoteInput
export const manageEditCategory: Ref<GlossaryCategory> = editCategoryInput
export const manageSaving: DeepReadonly<Ref<boolean>> = readonly(saving)
export const manageSavingAction: DeepReadonly<Ref<'save' | 'delete' | 'promote'>> = readonly(savingAction)
export const manageActionError: DeepReadonly<Ref<IpcError | null>> = readonly(actionError)
export const manageActionNotice: DeepReadonly<Ref<'promote_not_applicable' | null>> = readonly(actionNotice)
export const manageWorkTierAvailable: DeepReadonly<Ref<boolean>> = readonly(workTierAvailable)
export const manageExchangeTier: DeepReadonly<Ref<GlossaryTierWire>> = readonly(exchangeTierState)
export const manageExportBusy: DeepReadonly<Ref<boolean>> = readonly(exportBusy)
export const manageExportError: DeepReadonly<Ref<IpcError | null>> = readonly(exportError)
export const manageExportedPath: DeepReadonly<Ref<string | null>> = readonly(exportedPath)
export const manageExportIpcUnavailable: DeepReadonly<Ref<boolean>> = readonly(exportIpcUnavailable)

/**
 * Số hàng CHƯA LỌC đang giữ trong bộ nhớ.
 *
 * 🔴 **Phải là một export riêng, không suy được từ [`manageFilteredRows`].** `rows` là ô nhớ
 * riêng tư, nên trước bản vá 2026-08-24 template tự bịa tham số `totalCount` bằng
 * `manageFilteredRows.length === 0 ? 0 : 1` — một biểu thức lấy chính `filteredCount` làm
 * nguồn. Hệ quả đo được: bộ lọc loại hết hàng trên một Glossary CÓ dữ liệu thì `totalCount`
 * giả cũng thành 0, [`manageEmptyReasonFor`] trả `'glossary_empty'`, và màn hình nói
 * *"Glossary đang trống"* trong khi nhánh `'filter_no_match'` thành mã chết. Đúng lớp lỗi
 * *"rỗng im lặng"* mà `AGENTS.md` gọi là lỗi trung tâm của kho — chỉ khác là nó không im,
 * nó nói SAI. Ba lượt rà độc lập cùng chỉ vào chỗ này.
 */
export const manageTotalRows = computed<number>(() => rows.value.length)

/**
 * Danh sách đã qua BA bộ lọc + ô tìm — tính lại mỗi lần một trong bốn đầu vào đổi,
 * KHÔNG một round-trip IPC nào (§Design Notes của spec).
 */
export const manageFilteredRows = computed<GlossaryEntry[]>(() => {
  const q = searchQuery.value.trim().toLowerCase()
  return rows.value.filter((entry) => {
    if (categoryFilterState.value !== 'all' && entry.category !== categoryFilterState.value) return false
    if (originFilterState.value !== 'all' && entry.term_origin !== originFilterState.value) return false
    if (confirmedFilterState.value === 'confirmed' && entry.translation === null) return false
    if (confirmedFilterState.value === 'pending' && entry.translation !== null) return false
    if (q === '') return true
    return entry.source_term.toLowerCase().includes(q) || (entry.translation ?? '').toLowerCase().includes(q)
  })
})

/**
 * Hàng ĐANG CHỌN trong danh sách ĐÃ LỌC, hoặc `null` nếu con trỏ ngoài phạm vi.
 *
 * 🔴 `.at(cursor.value)`, KHÔNG `[cursor.value]` — cùng lý do `queueCurrentRow`
 * (`noUncheckedIndexedAccess` không bật; `.at()` khai đúng `T | undefined`).
 */
export const manageCurrentRow = computed<GlossaryEntry | null>(
  () => manageFilteredRows.value.at(cursor.value) ?? null,
)

/**
 * BỐN ca rỗng, phân biệt được trên màn hình (§Always của spec — "rỗng phải nói vì sao nó
 * rỗng"). `totalCount`/`filteredCount` phân biệt ca thứ ba (Glossary trống thật) với ca thứ
 * tư (bộ lọc không khớp gì): cùng hình dạng `manageFilteredRows.length === 0`, hai câu chuyện
 * khác nhau.
 */
export function manageEmptyReasonFor(
  s: GlossaryManageStatus,
  totalCount: number,
  filteredCount: number,
): 'not_loaded' | 'ipc_unavailable' | 'glossary_empty' | 'filter_no_match' | null {
  if (s === 'unknown') return 'not_loaded'
  if (s === 'ipc_unavailable') return 'ipc_unavailable'
  if (s !== 'loaded') return null // 'error' hiện qua nhánh riêng (tError), không qua đây.
  if (totalCount === 0) return 'glossary_empty'
  if (filteredCount === 0) return 'filter_no_match'
  return null
}

/** Đóng form Sửa mà KHÔNG lưu — dùng nội bộ mỗi khi lọc/tìm/điều hướng đổi hàng đang chọn. */
function discardOpenEdit(): void {
  editing.value = false
}

async function loadGlossaryManageRows(mySequence: number): Promise<boolean> {
  const result = await listGlossaryEntries()
  if (mySequence !== sequence) return false

  if (result.entries === null) {
    status.value = result.error === null ? 'ipc_unavailable' : 'error'
    loadError.value = result.error
    return false
  }

  rows.value = result.entries
  status.value = 'loaded'
  loadError.value = null
  return true
}

/**
 * Mở lớp phủ — điểm vào DUY NHẤT, gọi từ handler của lệnh `glossary.manage.open`.
 *
 * 🔴 CHỐT TÁI MỞ — cùng lý do `openGlossaryQueue`: hợp âm mặc định có thể bắn lần hai trong
 * khi lớp phủ đã mở; mở lại là VÔ HẠI (bỏ qua).
 */
export async function openGlossaryManage(): Promise<void> {
  if (overlayOpen.value) return

  sequence += 1
  const mySequence = sequence

  overlayOpen.value = true
  status.value = 'unknown'
  loadError.value = null
  rows.value = []
  searchQuery.value = ''
  categoryFilterState.value = 'all'
  originFilterState.value = 'all'
  confirmedFilterState.value = 'all'
  cursor.value = 0
  editing.value = false
  editTranslationInput.value = ''
  editNoteInput.value = ''
  editCategoryInput.value = 'other'
  saving.value = false
  actionError.value = null
  actionNotice.value = null
  workTierAvailable.value = false
  exchangeTierState.value = 'global'
  exportBusy.value = false
  exportError.value = null
  exportedPath.value = null
  exportIpcUnavailable.value = false

  const [, probe] = await Promise.all([loadGlossaryManageRows(mySequence), lookupGlossaryTerm('')])
  if (mySequence !== sequence) return
  if (probe.found !== 'unknown') workTierAvailable.value = probe.workTierAvailable
}

/**
 * Đóng lớp phủ — `Esc` hoặc nút Đóng. **KHÔNG dọn state** (cùng chủ ý `closeGlossaryQueue`):
 * mở lại luôn tải lại từ đầu.
 *
 * 🔴 Chặn trong lúc một lượt ghi đang bay — cùng khuôn `closeGlossaryQueue`.
 */
export function closeGlossaryManage(): void {
  if (saving.value) return
  overlayOpen.value = false
}

/** Đổi ô tìm — lọc chạy trong bộ nhớ, 0 round-trip IPC. Reset con trỏ về đầu danh sách đã
 * lọc mới và đóng form Sửa đang mở (hàng đang sửa có thể không còn khớp bộ lọc mới). */
export function setGlossaryManageSearch(query: string): void {
  searchQuery.value = query
  cursor.value = 0
  discardOpenEdit()
}

/** Đổi bộ lọc phân loại — cùng chủ ý `setGlossaryManageSearch`. */
export function setGlossaryManageCategoryFilter(value: GlossaryCategory | 'all'): void {
  categoryFilterState.value = value
  cursor.value = 0
  discardOpenEdit()
}

/** Đổi bộ lọc xuất xứ — cùng chủ ý `setGlossaryManageSearch`. */
export function setGlossaryManageOriginFilter(value: GlossaryManageOriginFilter): void {
  originFilterState.value = value
  cursor.value = 0
  discardOpenEdit()
}

/** Đổi bộ lọc trạng thái chốt — cùng chủ ý `setGlossaryManageSearch`. */
export function setGlossaryManageConfirmedFilter(value: GlossaryManageConfirmedFilter): void {
  confirmedFilterState.value = value
  cursor.value = 0
  discardOpenEdit()
}

/** Chuyển con trỏ xuống hàng kế tiếp trong danh sách ĐÃ LỌC — không vòng. */
export function nextGlossaryManageRow(): void {
  if (cursor.value < manageFilteredRows.value.length - 1) {
    cursor.value += 1
    discardOpenEdit()
  }
}

/** Chuyển con trỏ lên hàng trước trong danh sách ĐÃ LỌC — không vòng. */
export function prevGlossaryManageRow(): void {
  if (cursor.value > 0) {
    cursor.value -= 1
    discardOpenEdit()
  }
}

/** Mở form Sửa cho hàng ĐANG CHỌN — nạp giá trị hiện tại vào ba ô nhập. */
export function beginGlossaryManageEdit(): void {
  if (saving.value) return
  const row = manageCurrentRow.value
  if (row === null) return
  editing.value = true
  editTranslationInput.value = row.translation ?? ''
  editNoteInput.value = row.note
  editCategoryInput.value = row.category
  actionError.value = null
  actionNotice.value = null
}

/** Đóng form Sửa mà KHÔNG lưu — lệnh `glossary.manage.cancel`. */
export function cancelGlossaryManageEdit(): void {
  if (saving.value) return
  discardOpenEdit()
}

/**
 * Lưu form Sửa — `update_manual_term(tier, id, …)` qua adapter đã có. Thành công ⇒ hàng cập
 * nhật TẠI CHỖ (không nạp lại toàn bộ — sửa không đổi `is_shadowed` của bất kỳ hàng nào),
 * đóng form. Trượt ⇒ hiện lỗi, hàng giữ nguyên giá trị cũ.
 *
 * 🔴 **KHÔNG gọi `refreshGlossaryMarks` ở đây** — §Always của spec chỉ định "Xoá hoặc đẩy
 * tầng thành công ⇒ gọi `refreshGlossaryMarks`", SỬA không nằm trong danh sách đó.
 */
export async function saveGlossaryManageEdit(): Promise<void> {
  if (saving.value) return
  const row = manageCurrentRow.value
  if (row === null || !editing.value) return

  savingAction.value = 'save'
  saving.value = true
  actionError.value = null
  const mySequence = sequence

  const translationValue = editTranslationInput.value.trim() === '' ? null : editTranslationInput.value
  const result = await updateGlossaryTerm(row.tier, row.id, translationValue, editNoteInput.value, editCategoryInput.value)
  if (mySequence !== sequence) return // lớp phủ đã đóng rồi mở lại — bỏ, không ghi đè.

  saving.value = false
  if (result.error !== null) {
    actionError.value = result.error
    return
  }

  const target = rows.value.find((entry) => entry.tier === row.tier && entry.id === row.id)
  if (target !== undefined) {
    target.translation = translationValue
    target.note = editNoteInput.value
    target.category = editCategoryInput.value
  }
  editing.value = false
}

/** Làm mới dấu thuật ngữ của Chương đang mở — cùng khuôn `refreshGlossaryMarksAfterSave`
 * của `glossaryQuickAddState.ts`. `void` có chủ ý: mục Glossary đã đổi XUỐNG ĐĨA THẬT dù
 * lượt làm mới này có trượt. */
function refreshGlossaryMarksAfterMutation(): void {
  const chapterId = editorChapterId.value
  const chapter = sourceChapter.value
  if (chapterId === null || chapter === null || chapterId !== chapter.chapter_id) return
  void refreshGlossaryMarks(chapterId, editorSegments.value, chapter.source_lang)
}

/** Nạp lại TRỌN danh sách sau một lượt Xoá/Đẩy tầng thành công — xem khối 🔴 đầu tệp cho lý
 * do không patch tại chỗ. Kẹp con trỏ về trong phạm vi danh sách đã lọc mới. */
async function reloadGlossaryManageRowsAfterMutation(mySequence: number): Promise<void> {
  const ok = await loadGlossaryManageRows(mySequence)
  if (!ok || mySequence !== sequence) return
  const maxIndex = manageFilteredRows.value.length - 1
  if (cursor.value > maxIndex) cursor.value = Math.max(0, maxIndex)
}

/**
 * Xoá hàng ĐANG CHỌN — kể cả một mục ĐÃ CHỐT (hợp lệ). Lệnh `glossary.manage.delete`.
 *
 * Thành công ⇒ mục biến khỏi danh sách (nạp lại trọn), `refreshGlossaryMarks` chạy.
 * Trượt ⇒ hàng ở lại, KHÔNG gọi refresh (§I/O Matrix).
 */
export async function deleteGlossaryManageEntry(): Promise<void> {
  if (saving.value) return
  const row = manageCurrentRow.value
  if (row === null) return

  savingAction.value = 'delete'
  saving.value = true
  actionError.value = null
  actionNotice.value = null
  const mySequence = sequence

  const result = await deleteGlossaryTerm(row.tier, row.id)
  if (mySequence !== sequence) return

  saving.value = false
  if (result.error !== null) {
    actionError.value = result.error
    return
  }

  editing.value = false
  await reloadGlossaryManageRowsAfterMutation(mySequence)
  refreshGlossaryMarksAfterMutation()
}

/**
 * Đẩy hàng ĐANG CHỌN lên tầng Toàn cục — lệnh `glossary.manage.promote`.
 *
 * 🔴 **Hàng tầng Global ⇒ "không áp dụng", NÓI RA** (§I/O Matrix) — không gọi IPC, đặt
 * [`manageActionNotice`] để lớp phủ hiện một câu giải thích thay vì im lặng không làm gì.
 *
 * Thành công ⇒ hàng đổi tầng (nạp lại trọn — `id` mới sinh ở `global.db`, patch tại chỗ
 * không biết `id` đó), `refreshGlossaryMarks` chạy. Trượt (kể cả `GlobalTermExists`) ⇒ cả
 * hai mục giữ nguyên, KHÔNG gọi refresh.
 */
export async function promoteGlossaryManageEntry(): Promise<void> {
  if (saving.value) return
  const row = manageCurrentRow.value
  if (row === null) return

  if (row.tier === 'global') {
    actionNotice.value = 'promote_not_applicable'
    return
  }

  savingAction.value = 'promote'
  saving.value = true
  actionError.value = null
  actionNotice.value = null
  const mySequence = sequence

  const result = await promoteGlossaryTermToGlobal(row.id)
  if (mySequence !== sequence) return

  saving.value = false
  if (result.error !== null) {
    actionError.value = result.error
    return
  }

  editing.value = false
  await reloadGlossaryManageRowsAfterMutation(mySequence)
  refreshGlossaryMarksAfterMutation()
}

/** Đổi tầng đang chọn cho Xuất/Nhập — lệnh `glossary.manage.export_csv`/`…import_csv` đọc
 * `ref` này ngay trước khi dispatch. */
export function setGlossaryManageExchangeTier(value: GlossaryTierWire): void {
  exchangeTierState.value = value
}

/**
 * Xuất tầng đang chọn — mở hộp thoại LƯU trong Rust rồi ghi. Lệnh
 * `glossary.manage.export_csv`.
 *
 * 🔴 **Huỷ hộp thoại là im lặng, KHÔNG một câu nào** (§Always) — `exportedPath`/`exportError`
 * đều giữ nguyên giá trị TRƯỚC lượt gọi này khi `path === null, error === null`.
 */
export async function exportGlossaryManageTier(): Promise<void> {
  if (exportBusy.value) return

  exportBusy.value = true
  exportError.value = null
  exportIpcUnavailable.value = false
  // 🔴 P3 (vòng rà ba lớp 2026-08-25) — xoá đường dẫn CŨ TRƯỚC KHI hộp thoại mở, không chỉ
  // khi lượt MỚI thành công. Bản trước giữ nguyên `exportedPath` qua một lượt huỷ/trượt kế
  // tiếp: xuất thành công vào X, mở lại rồi HUỶ ⇒ overlay vẫn đọc "Đã ghi vào X" — đọc như
  // thể lượt vừa huỷ cũng đã ghi.
  exportedPath.value = null
  const mySequence = sequence

  const result = await exportGlossaryTier(exchangeTierState.value)
  if (mySequence !== sequence) return

  exportBusy.value = false
  if (result.outcome === 'cancelled') return // Huy hop thoai -- im lang, khong doi gi (§Always).
  if (result.outcome === 'ipc_unavailable') {
    exportIpcUnavailable.value = true
    return
  }
  if (result.outcome === 'error') {
    exportError.value = result.error
    return
  }

  exportedPath.value = result.path
}

/**
 * 🔴 Vứt toàn bộ state của lớp phủ — `check:panel-refs` đòi mọi ô nhớ cấp module có một
 * đường `reset*()`. **0 chỗ gọi sản phẩm hôm nay**, đúng tiền lệ `resetGlossaryQueue`: lớp
 * phủ là một MODAL chắn hết tương tác phía sau nó, và mỗi lượt mở lại
 * (`openGlossaryManage`) đã tự nạp lại TOÀN BỘ state từ đầu.
 */
export function resetGlossaryManage(): void {
  sequence += 1
  overlayOpen.value = false
  status.value = 'unknown'
  loadError.value = null
  rows.value = []
  searchQuery.value = ''
  categoryFilterState.value = 'all'
  originFilterState.value = 'all'
  confirmedFilterState.value = 'all'
  cursor.value = 0
  editing.value = false
  editTranslationInput.value = ''
  editNoteInput.value = ''
  editCategoryInput.value = 'other'
  saving.value = false
  savingAction.value = 'save'
  actionError.value = null
  actionNotice.value = null
  workTierAvailable.value = false
  exchangeTierState.value = 'global'
  exportBusy.value = false
  exportError.value = null
  exportedPath.value = null
  exportIpcUnavailable.value = false
}
