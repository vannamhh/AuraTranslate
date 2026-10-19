/**
 * State + thao tác của khối "Chương" ở Library — Story 5.7, FR12: liệt kê Chương của một
 * Tác phẩm, mở lại chính Tác phẩm đó (`open_work`), và mở một Chương đích danh vào Workspace.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ VÌ SAO MỘT MODULE THUẦN RIÊNG, KHÔNG VIẾT THẲNG TRONG `LibraryMode.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `libraryWorks.ts`/`libraryRescan.ts`: AD-34 §1 đòi mọi `@click` là đúng một
 * `dispatch('<id>')`, và năm thao tác mới (`library.open_work` · `library.chapter_next` ·
 * `library.chapter_prev` · `library.open_chapter` · `library.list_chapters`) đăng ký ở
 * `src/commands/index.ts` như các `CommandDeps` TIÊM VÀO. Module này là phía CUNG CẤP.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 `chaptersHaveLoaded` — LÝ DO NÓ TỒN TẠI (`AGENTS.md::Known pitfalls`)
 * ─────────────────────────────────────────────────────────────────────────────
 * "Tác phẩm này chưa có Chương nào" chỉ được phép nói SAU khi đã tải danh sách ít nhất một
 * lần cho Tác phẩm ĐANG MỞ. Trước lượt tải đầu, danh sách cũng rỗng — nhưng đó là "chưa
 * biết", không phải "không có". `LibraryMode.vue` phải hỏi vị từ này TRƯỚC khi kết luận —
 * cùng khuôn `worksHaveLoaded` của `libraryWorks.ts`, và cùng lớp lỗi mà `AGENTS.md` đếm
 * BA lần (Story 1.16 · 2.10 · 3.9).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 `openWorkById` — KHUÔN CHÉP TỪ `libraryImport.ts::beginSubmit`/`finishSubmit`
 * ─────────────────────────────────────────────────────────────────────────────
 * Mở một `.atproj` khác cũng là "đổi Tác phẩm" đúng nghĩa `beginSubmit` đã định nghĩa: phải
 * FLUSH TRƯỚC (đọc CẢ BA giá trị — `'clean'`/`'failed'`/`'still-dirty'` — §Always của story),
 * CHẶN kèm câu báo khi trượt, rồi mới gọi IPC đổi `OpenWorkState`. Sau khi đổi xong, vứt state
 * tầng TÁC PHẨM (`resetSourcePanel`/`resetLookupPanel`/`resetEditorPanel`/
 * `resetSegmentHistory` — `finishSubmit` là tiền lệ đầy đủ, xem doc-comment của nó) rồi nạp
 * lại NGAY, không để màn hình đứng ở Tác phẩm cũ vì ba chế độ sống trong `<KeepAlive>`.
 */
import { computed, nextTick, readonly, ref, watch } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { dispatch, enterFocus } from '../commands'
import {
  listChapters,
  mergeChapters,
  moveChapter,
  renameChapter,
  updateChapterOrigin,
} from '../config/chapter'
import type { ChapterDirection, ChapterOriginEdit, ChapterRow } from '../config/chapter'
import type { ChapterOriginEditFields } from '../config/project'
import { openWork } from '../config/library'
import type { WorkRow } from '../config/library'
import { setChaptersStatus } from '../config/lifecycle'
import {
  editorChapterId,
  ensureSegmentsLoaded,
  flushEditorBeforeDiscreteWrite,
  flushChapterPositionNow,
  openChapterById,
  resetEditorPanel,
} from '../panels/editorPanelState'
import { ensureChapterLoaded, resetSourcePanel } from '../panels/sourcePanelState'
import { resetLookupPanel } from '../panels/lookupPanelState'
import { resetSegmentHistory } from '../panels/segmentHistoryState'
import { setMode } from './modeState'
import {
  applyOpenWorkLifecycleResult,
  currentLibraryWork,
  loadOpenWorkLifecycle,
  loadWorks,
  selectLibraryWorkById,
} from './libraryWorks'
import {
  applyChapterClick,
  buildChapterView,
  chapterActionRules,
  clearChapterSelection,
  closeChapterStatusMenu,
  countChaptersByGroup,
  flatIndexOfChapter,
  libraryChapterAnchorId,
  libraryChapterFilter,
  libraryChapterNotStartedExpanded,
  libraryChapterSearch,
  libraryChapterSelectedIds,
  libraryChapterSort,
  pruneChapterSelection,
  resetLibraryChapterSelection,
  selectedRowsInOrder,
  setChapterSelection,
  visibleChapterIds,
} from './libraryChapterSelection'
import type { ChapterClickModifiers } from './libraryChapterSelection'
// Story 5.11 — Chế độ đọc mang CÙNG lớp cache module-level (`readingState.ts`), nên nó phải
// đi cùng lượt vứt state Tác phẩm/Chương này — cùng lý lẽ `resetEditorPanel` đã ghi.
import { resetReading, resetReadingToc } from './readingState'
// 🔴 Story 4.7 loop 2, finding P4 — cùng lý lẽ Story 5.11 vừa ghi ở trên, một bậc khác: bản ghi
// prompt đã lắp cũng mang danh tính THEO TÁC PHẨM (`segment_id`/`chapter_id` là khoá hàng của
// CHÍNH `project.db` sắp rời, `commands/aiprompt.rs::clear_last_assembled_prompt_on_work_close`'s
// doc-comment). Rust đã dọn phía nó (`replace_open_work`, finding V1) khi `openWork()` NGAY
// DƯỚI thành công — nhưng phía webview, `aiPromptInspectorState.ts::record` không tự biết
// điều đó cho tới lượt Đọc kế tiếp, và dòng tóm tắt của `AiTranslationPanel.vue` là LUÔN HIỆN,
// không đợi ai mở lớp phủ Xem prompt để kích một lượt Đọc mới. Không vứt ở đây thì dòng tóm
// tắt tiếp tục mô tả Tác phẩm VỪA RỜI đi cho tới khi người dùng tình cờ mở lớp phủ.
import { resetAiPromptInspector } from '../aiPromptInspectorState'
// 🔴 THÊM Story 4.8 — cùng lý lẽ dòng trên: kết quả dịch mang danh tính THEO Tác phẩm
// (`aiTranslateState.ts::runSegmentId` chỉ có nghĩa trong `project.db` đang mở). Không vứt ở
// đây thì `.ai-surface` tiếp tục hiện bản dịch của Tác phẩm VỪA RỜI dưới nhãn Tác phẩm mới.
import { resetAiTranslate } from '../aiTranslateState'
// 🔴 THÊM Story 4.9, Phase 3 — cùng lý lẽ dòng trên, kết quả của một LÔ cũng mang danh tính
// THEO Tác phẩm (mỗi `segmentId` của một hàng chỉ có nghĩa trong `project.db` đang mở). Xem
// doc-comment đầu `aiTranslateBatchState.ts`.
import { resetAiTranslateBatch } from '../aiTranslateBatchState'
import { resetProofread } from '../proofreadState'
// 🔴 THÊM Story 4.9 — vùng chọn nhiều-segment mang hai `segment.id` (`anchor`/`focus`), cùng
// lý lẽ dòng trên. Xem doc-comment đầu `panels/segmentSelectionState.ts`.
import { resetSegmentSelection } from '../panels/segmentSelectionState'
import { resetReviewMode } from '../reviewModeState'
import { resetGlossaryQueue } from '../glossaryQueueState'
import { resetAlignment } from '../alignmentState'
import { resetReviewerImport } from '../reviewerImportState'
import type { IpcError } from '../i18n'
import { setOpenWorkId } from '../openWorkIdentity'

// ─────────────────────────────────────────────────────────────────────────────
// State module-level — singleton của cả tiến trình, cùng khuôn `libraryWorks.ts`. Mỗi khai
// báo NẰM TRÊN MỘT DÒNG (`check:panel-refs` Kiểm 5 — cú pháp ngoài tập con cho ĐỎ).
// ─────────────────────────────────────────────────────────────────────────────
const chapters = ref<ChapterRow[]>([])
const chaptersHaveLoaded = ref(false)
const chaptersBusy = ref(false)
const chaptersError = ref<IpcError | null>(null)
// Con trỏ ô đang chọn trong danh sách — chép khuôn `workCursor` của `libraryWorks.ts`.
const chapterCursor = ref(0)
// Một lượt `loadChapters()` bị chặn vì đang bận, ghi nhớ để chạy lại — cùng khuôn
// `worksReloadPending`. Không phải `ref`: không khối template nào đọc nó.
let chaptersReloadPending = false
let chaptersSequence = 0

/** Kết cục một lượt `openWorkById` bị CHẶN vì tập chờ Editor chưa sạch — §Always của story:
 * *"'failed'/'still-dirty' ⇒ CHẶN kèm câu báo"*. Hai giá trị, phân biệt được, cùng khuôn
 * `NavNotice` của `editorPanelState.ts`. */
export type OpenWorkNotice = 'flush-failed' | 'still-dirty'
const openWorkBusy = ref(false)
const openWorkNotice = ref<OpenWorkNotice | null>(null)
const openWorkError = ref<IpcError | null>(null)

// ─────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 5.8 — TỔ CHỨC LẠI CHƯƠNG (đổi tên · dời lên/xuống · gộp vào Chương liền trước)
// ─────────────────────────────────────────────────────────────────────────────
/** Ô nhập tên Chương mới — ghi được cho `v-model`, cùng khuôn `libraryImport.ts::pastedText`
 * (export TRẦN, không bọc `readonly()`: `LibraryMode.vue` phải ghi được qua `v-model`). */
export const chapterRenameDraft = ref('')
const chapterReorgBusy = ref(false)
const chapterRenameEditing = ref(false)
const workPageWork = ref<WorkRow | null>(null)
/** Kết cục một lượt tổ chức bị CHẶN vì tập chờ Editor chưa sạch — cùng khuôn `OpenWorkNotice`
 * ngay trên, và cùng hai giá trị: §Always của story chỉ đòi phân biệt được hai ca đó. */
export type ChapterReorgNotice = 'flush-failed' | 'still-dirty'
const chapterReorgNotice = ref<ChapterReorgNotice | null>(null)
const chapterReorgError = ref<IpcError | null>(null)

// ─────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 6.15 — XUẤT XỨ TÀI LIỆU Ở TẦNG CHƯƠNG (FR128/AD-43)
// ─────────────────────────────────────────────────────────────────────────────
/** Một lượt sửa xuất xứ đang bay — cờ RIÊNG, không mượn `chapterReorgBusy`: sửa xuất xứ
 * không đụng `segment`/`ord` nên không cần lượt flush Editor mà `beginChapterReorg` gác. */
const chapterOriginBusy = ref(false)
const chapterOriginError = ref<IpcError | null>(null)
/** Ord đích của lượt áp xuất xứ THEO ĐOẠN — `null` = tắt (chỉ Chương đang chọn, hành vi cũ).
 * Đọc bởi [`saveCurrentChapterOrigin`]; `LibraryMode.vue` ghi qua [`setChapterOriginApplyThroughOrd`]. */
const chapterOriginApplyThroughOrd = ref<number | null>(null)

export const libraryChapters: DeepReadonly<Ref<ChapterRow[]>> = readonly(chapters)
export const libraryChaptersHaveLoaded: DeepReadonly<Ref<boolean>> = readonly(chaptersHaveLoaded)
export const libraryChaptersBusy: DeepReadonly<Ref<boolean>> = readonly(chaptersBusy)
export const libraryChaptersError: DeepReadonly<Ref<IpcError | null>> = readonly(chaptersError)
export const libraryChapterCursor: DeepReadonly<Ref<number>> = readonly(chapterCursor)
export const libraryOpenWorkBusy: DeepReadonly<Ref<boolean>> = readonly(openWorkBusy)
export const libraryOpenWorkNotice: DeepReadonly<Ref<OpenWorkNotice | null>> = readonly(openWorkNotice)
export const libraryOpenWorkError: DeepReadonly<Ref<IpcError | null>> = readonly(openWorkError)
export const libraryChapterRenameEditing: DeepReadonly<Ref<boolean>> = readonly(chapterRenameEditing)
export const libraryWorkPageWork: DeepReadonly<Ref<WorkRow | null>> = readonly(workPageWork)
export const libraryWorkPageIsOpen = computed<boolean>(() => workPageWork.value !== null)
export const libraryChapterReorgBusy: DeepReadonly<Ref<boolean>> = readonly(chapterReorgBusy)
export const libraryChapterReorgNotice: DeepReadonly<Ref<ChapterReorgNotice | null>> = readonly(chapterReorgNotice)
export const libraryChapterReorgError: DeepReadonly<Ref<IpcError | null>> = readonly(chapterReorgError)
export const libraryChapterOriginBusy: DeepReadonly<Ref<boolean>> = readonly(chapterOriginBusy)
export const libraryChapterOriginError: DeepReadonly<Ref<IpcError | null>> = readonly(chapterOriginError)
export const libraryChapterOriginApplyThroughOrd: DeepReadonly<Ref<number | null>> =
  readonly(chapterOriginApplyThroughOrd)

/**
 * Chương ĐANG CHỌN trong danh sách, hoặc `null` nếu con trỏ ngoài phạm vi (danh sách rỗng,
 * hoặc chưa tải lần nào). `.at(cursor.value)`, KHÔNG `[cursor.value]` — cùng lý do
 * `currentLibraryWork` (`libraryWorks.ts`): `noUncheckedIndexedAccess` không bật, `.at()`
 * khai đúng `T | undefined`.
 */
export const currentLibraryChapter = computed<ChapterRow | null>(
  () => chapters.value.at(chapterCursor.value) ?? null,
)

/** The one selected Chapter, or `null` for zero or several. Every single-Chapter write targets it. */
export const soleSelectedLibraryChapter = computed<ChapterRow | null>(() => {
  if (libraryChapterSelectedIds.value.size !== 1) return null
  return chapters.value.find((row) => libraryChapterSelectedIds.value.has(row.chapter_id)) ?? null
})

export const libraryChapterView = computed(() =>
  buildChapterView(
    chapters.value,
    libraryChapterFilter.value,
    libraryChapterSearch.value,
    libraryChapterSort.value,
    libraryChapterNotStartedExpanded.value,
  ),
)
export const libraryChapterGroupCounts = computed(() => countChaptersByGroup(chapters.value))
export const libraryChapterSelectedRows = computed<ChapterRow[]>(() =>
  selectedRowsInOrder(chapters.value, libraryChapterSelectedIds.value),
)
export const libraryChapterRules = computed(() => chapterActionRules(chapters.value, libraryChapterSelectedIds.value))

/** The action bar must never act on rows the filter, search or fold hides. */
watch(
  () => visibleChapterIds(libraryChapterView.value.items),
  (visible) => {
    const shown = new Set(visible)
    pruneChapterSelection(chapters.value.filter((row) => shown.has(row.chapter_id)))
  },
  { flush: 'sync' },
)

/**
 * The rename draft follows the sole selected Chapter by `chapter_id`, not by row object:
 * `loadChapters()` replaces the array, and a watch on the object would overwrite text being typed.
 */
watch(
  () => soleSelectedLibraryChapter.value?.chapter_id ?? null,
  () => {
    chapterRenameDraft.value = soleSelectedLibraryChapter.value?.title ?? ''
    chapterRenameEditing.value = false
    chapterOriginApplyThroughOrd.value = null
  },
)

function clampChapterCursor(): void {
  const maxIndex = chapters.value.length - 1
  if (chapterCursor.value > maxIndex) chapterCursor.value = Math.max(0, maxIndex)
  if (chapterCursor.value < 0) chapterCursor.value = 0
}

/**
 * Đọc lại danh sách Chương của Tác phẩm ĐANG MỞ — lệnh `library.list_chapters`. Cùng cơ chế
 * chống nuốt lượt tải với [`loadWorks`] của `libraryWorks.ts`: một lượt gọi trong khi lượt
 * trước còn bay được GHI NHỚ và chạy lại ngay khi lượt đang bay kết thúc, không bị bỏ.
 */
export async function loadChapters(): Promise<void> {
  if (chaptersBusy.value) {
    chaptersReloadPending = true
    return
  }

  chaptersBusy.value = true
  chaptersError.value = null
  const mySequence = ++chaptersSequence

  const result = await listChapters()
  if (mySequence !== chaptersSequence) return // Một lượt MỚI hơn đã bắt đầu -- bỏ.

  chaptersBusy.value = false
  if (result.error !== null) {
    chaptersError.value = result.error
    await runPendingChaptersReload()
    return
  }
  if (result.chapters === null) {
    // Không có cầu IPC -- im lặng, cùng nhánh mọi adapter khác.
    await runPendingChaptersReload()
    return
  }

  chapters.value = result.chapters
  chaptersHaveLoaded.value = true
  pruneChapterSelection(result.chapters)
  // §I/O Matrix "Con trỏ Chương ra ngoài sau lượt tải" -- kẹp NGAY, trước khi ai đọc
  // `currentLibraryChapter`.
  clampChapterCursor()
  await runPendingChaptersReload()
}

async function runPendingChaptersReload(): Promise<void> {
  if (!chaptersReloadPending) return
  chaptersReloadPending = false
  await loadChapters()
}

/** Chuyển con trỏ xuống ô kế tiếp — không vòng. No-op trên danh sách rỗng. */
export function nextChapter(): void {
  if (chapterCursor.value < chapters.value.length - 1) chapterCursor.value += 1
}

/** Chuyển con trỏ lên ô trước — không vòng. No-op trên danh sách rỗng. */
export function prevChapter(): void {
  if (chapterCursor.value > 0) chapterCursor.value -= 1
}

/**
 * Mở lại một `.atproj` đã có trên đĩa — lệnh `library.open_work`. Chép khuôn
 * `libraryImport.ts::beginSubmit`/`finishSubmit` — xem khối doc-comment đầu tệp.
 */
export async function openWorkById(workId: string): Promise<boolean> {
  if (openWorkBusy.value) return false

  openWorkBusy.value = true
  openWorkNotice.value = null
  openWorkError.value = null

  // ── FLUSH TRƯỚC, và ĐỌC CẢ BA GIÁ TRỊ — §Always của story ─────────────────────
  const flushed = await flushEditorBeforeDiscreteWrite()
  if (flushed !== 'clean') {
    console.error(
      `[library] mo Tac pham bi CHAN: luot flush tra '${flushed}' — ban dich chua xuong dia`,
    )
    openWorkNotice.value = flushed === 'failed' ? 'flush-failed' : 'still-dirty'
    openWorkBusy.value = false
    return false
  }

  // Vị trí đang chờ ghi của Tác phẩm CŨ cũng phải rời đi trước khi `OpenWorkState` đổi chỗ —
  // KHÔNG chặn (mất một lượt chỉ mất một lời nhắc, không mất bản dịch).
  await flushChapterPositionNow()

  const { opened, error } = await openWork(workId)

  if (error !== null) {
    openWorkError.value = error
    openWorkBusy.value = false
    return false
  }
  if (opened === null) {
    // Không có cầu IPC -- im lặng, cùng nhánh mọi adapter khác.
    openWorkBusy.value = false
    return false
  }

  // ═════════════════════════════════════════════════════════════════════════════════
  // 🔴 CỬA SỔ MỞ **NHẦM CHƯƠNG**, VÀ VÌ SAO `openWorkBusy` PHẢI GIỮ TỚI CUỐI HÀM
  // ═════════════════════════════════════════════════════════════════════════════════
  // 🔵 **SỬA 2026-08-29 (lượt review).** Bản trước nhả `openWorkBusy.value = false` NGAY sau
  // lượt IPC `open_work`, tức **trước** bốn lượt vứt state và trước `loadChapters()`. Trong
  // cửa sổ đó, `OpenWorkState` phía Rust **đã** trỏ sang Tác phẩm MỚI trong khi
  // `.chapters-list` trên màn hình vẫn là danh sách của Tác phẩm **CŨ** — và cả hai nút
  // ("Mở Tác phẩm", "Mở Chương") đã sáng trở lại.
  //
  // 🔴 Hậu quả **không** phải một lỗi báo ra: `chapter.id` là `AUTOINCREMENT` **cục bộ trong
  // từng `project.db`** (đo được ở bàn đo e2e của chính story này: hai Tác phẩm khác nhau
  // **đều** có `chapter_id = 1`). ⇒ Người dùng bấm "Mở Chương" trên một hàng của Tác phẩm CŨ,
  // id đó **tồn tại thật** trong kho MỚI, `open_chapter` trả `Ok`, và Workspace mở một Chương
  // **hoàn toàn khác** thứ người dùng vừa bấm — không một lỗi nào, không một dấu hiệu nào.
  // Đúng hạng *"một kết quả sai trông như bình thường"* mà `project-context.md`
  // §Critical Don't-Miss đặt lên hàng đầu.
  //
  // ⇒ Hai vế, và cần **cả hai**: ① `openWorkBusy` giữ tới khi mọi lượt nạp xong (nút tắt suốt
  // cửa sổ ấy); ② danh sách Chương CŨ bị vứt **ngay** ở đây, nên kể cả khi một đường khác mở
  // được nút, `currentLibraryChapter` là `null` chứ không phải một hàng của Tác phẩm cũ.
  setOpenWorkId(opened.meta.work_id)
  chapters.value = []
  chaptersHaveLoaded.value = false
  chapterCursor.value = 0
  resetLibraryChapterSelection()
  chapterRenameEditing.value = false
  workPageWork.value = null

  // 🔴 Tác phẩm MỚI ⇒ vứt state tầng TÁC PHẨM của Tác phẩm CŨ — nguyên văn bốn lượt vứt của
  // `libraryImport.ts::finishSubmit` (Story 1.17), không một lời gọi thứ hai rải ra.
  resetSourcePanel()
  resetLookupPanel()
  resetEditorPanel()
  resetSegmentHistory()
  // 🔵 THÊM Story 5.11 — Chế độ đọc mang cùng lớp cache module-level; Tác phẩm cũ bị thay
  // thì Chương đang đọc của nó cũng phải bị vứt, cùng lượt với bốn dòng trên.
  resetReading()
  resetReadingToc()
  // 🔴 THÊM Story 4.7 loop 2, finding P4 — bản ghi prompt đã lắp mang danh tính THEO TÁC PHẨM
  // (segment_id/chapter_id), cùng lý lẽ bốn/sáu lượt vứt trên; xem doc-comment tại chỗ `import`.
  resetAiPromptInspector()
  // 🔴 THÊM Story 4.8 — cùng lý lẽ dòng trên, kết quả dịch cũng mang danh tính THEO TÁC PHẨM;
  // xem doc-comment tại chỗ `import`.
  resetAiTranslate()
  // 🔴 THÊM Story 4.9, Phase 3 — cùng lý lẽ dòng trên, kết quả của một LÔ cũng mang danh tính
  // THEO TÁC PHẨM; xem doc-comment tại chỗ `import`.
  resetAiTranslateBatch()
  resetProofread()
  // 🔴 THÊM Story 4.9 — vùng chọn nhiều-segment mang `anchor`/`focus` là hai `segment.id`,
  // cùng lý lẽ hai dòng trên: `AUTOINCREMENT` của `project.db` đếm RIÊNG mỗi Tác phẩm, nên
  // hai id cũ tồn tại THẬT ở Tác phẩm mới và trỏ vào câu khác hẳn — xem doc-comment đầu
  // `panels/segmentSelectionState.ts` cho lý do lượt đổi CHƯƠNG trong cùng Tác phẩm KHÔNG
  // cần dòng này (tự lành qua `indexOf`).
  resetSegmentSelection()
  resetReviewMode()
  resetGlossaryQueue()
  resetAlignment()
  resetReviewerImport()

  // Vứt là CHƯA ĐỦ — nạp lại NGAY, cùng lý do `finishSubmit`: ba chế độ sống trong
  // `<KeepAlive>`, không có `mounted` lần thứ hai.
  //
  // 🔵 **`await`, KHÔNG `void` (2026-08-29, sau lượt e2e đỏ)** — và đây là chỗ CỐ Ý lệch với
  // tiền lệ `libraryImport.ts::finishSubmit` (`void`, dòng ~232). Lý do là một khác biệt về
  // thứ tự người dùng: sau `finishSubmit` không thao tác nào **phụ thuộc** hai lượt nạp đó,
  // còn ở đây thao tác kế tiếp của người dùng — bấm "Mở Chương" ngay trên danh sách vừa hiện
  // — chạy trên cùng state ấy. Một `void` để lại một cửa sổ mà hai lượt nạp còn đang bay, tức
  // một cuộc đua **phụ thuộc tốc độ máy**: đúng lớp lỗi mà một bàn đo sẽ báo là "chập chờn"
  // thay vì "hỏng". Chờ xong rồi mới trả về làm lượt mở kế tiếp TẤT ĐỊNH.
  await ensureChapterLoaded()
  await ensureSegmentsLoaded()
  // 🔴 TIÊU ĐIỂM phải Ở LẠI trong lưới sau lượt nạp này, cùng khuôn đuôi
  // `await nextTick(); enterFocus('panel.grid')` của
  // `editorPanelState.ts::switchChapter`/`openChapterById` — Vue chưa vá DOM ngay khi lượt
  // nạp segment về, và một `enterFocus` sớm hơn là no-op.
  await nextTick()
  enterFocus('panel.grid')

  // Danh sách Chương của Tác phẩm mới — bề mặt riêng của story này. (`chapterCursor` đã về 0
  // ở khối vứt phía trên, cùng lượt với `chapters`/`chaptersHaveLoaded`.)
  await loadChapters()

  // Nhả cờ bận SAU CÙNG — xem khối 🔴 "CỬA SỔ MỞ NHẦM CHƯƠNG" phía trên.
  openWorkBusy.value = false
  return true
}

/** Mở Tác phẩm ĐANG CHỌN trong lưới — nút "Mở Tác phẩm" của mỗi ô, cùng khuôn
 * `forgetCurrentLibraryOrphan` (`libraryRescan.ts`): đọc con trỏ hiện thời, không nhận tham
 * số qua `dispatch()`. No-op khi không có ô nào đang chọn. Thành công ⇒ trang Tác phẩm thay lưới. */
export async function openCurrentLibraryWork(): Promise<void> {
  const work = currentLibraryWork.value
  if (work === null) return
  const opened = await openWorkById(work.work_id)
  if (!opened) return
  workPageWork.value = { ...work }
  await loadOpenWorkLifecycle()
}

/** Opens the selected Work, then the reviewer-import file picker; a failed or blocked open stops before the picker. */
export async function openCurrentLibraryWorkThenReviewerImport(): Promise<void> {
  await openCurrentLibraryWork()
  if (workPageWork.value === null) return
  await nextTick()
  document.querySelector<HTMLElement>('[data-library-chapters-reviewer-import]')?.focus()
  dispatch('export.reviewer_import.open')
}

/** Back to the Works grid; the grid reloads so the card shows the new status. */
export function closeLibraryWorkPage(): void {
  const closing = workPageWork.value
  if (closing === null) return
  workPageWork.value = null
  resetLibraryChapterSelection()
  chapterRenameEditing.value = false
  selectLibraryWorkById(closing.work_id)
  void loadWorks().then(() => selectLibraryWorkById(closing.work_id))
}

/** Mở Chương ĐANG CHỌN vào Workspace — lệnh `library.open_chapter`. No-op trên danh sách
 * rỗng. Chỉ đổi chế độ khi Chương THẬT SỰ mở được — một lượt CHẶN (flush trượt) hay lỗi phải
 * giữ người dùng ở Library để họ thấy được lý do, không đẩy họ sang một Workspace trống. */
export async function openCurrentChapter(): Promise<void> {
  const row = currentLibraryChapter.value
  if (row === null) return
  const moved = await openChapterById(row.chapter_id)
  if (moved) setMode('workspace')
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.8 — BỐN THAO TÁC TỔ CHỨC (đổi tên · dời lên · dời xuống · gộp vào Chương trước)
// ═════════════════════════════════════════════════════════════════════════════════
// Cùng khuôn CHUNG cho cả bốn: chống hai lượt cùng bay bằng `chapterReorgBusy`, flush TRƯỚC
// (đọc CẢ BA giá trị — §Always của story), CHẶN kèm câu báo khi khác 'clean', gọi IPC, rồi
// `await loadChapters()`; nhả cờ bận SAU CÙNG.

/** Nhịp chung của bốn thao tác: khoá + flush + đọc kết quả flush. `null` ⇒ đi tiếp được;
 * khác `null` ⇒ CHẶN, chỗ gọi phải trả về ngay không phát IPC nào. */
async function beginChapterReorg(): Promise<boolean> {
  if (chapterReorgBusy.value) return false
  chapterReorgBusy.value = true
  chapterReorgError.value = null
  chapterReorgNotice.value = null

  const flushed = await flushEditorBeforeDiscreteWrite()
  if (flushed !== 'clean') {
    console.error(
      `[library] to chuc lai Chuong bi CHAN: luot flush tra '${flushed}' — ban dich chua xuong dia`,
    )
    chapterReorgNotice.value = flushed === 'failed' ? 'flush-failed' : 'still-dirty'
    chapterReorgBusy.value = false
    return false
  }
  return true
}

/** Đổi tên Chương ĐANG CHỌN trong danh sách — lệnh `library.chapter_rename`. No-op trên danh
 * sách rỗng. */
export async function renameCurrentChapter(): Promise<void> {
  const row = soleSelectedLibraryChapter.value
  if (row === null) return
  if (!(await beginChapterReorg())) return

  const { error } = await renameChapter(row.chapter_id, chapterRenameDraft.value)
  if (error !== null) {
    chapterReorgError.value = error
    chapterReorgBusy.value = false
    return
  }

  chapterRenameEditing.value = false
  await loadChapters()
  chapterReorgBusy.value = false
}

/** Opens the inline rename field for the sole selected Chapter, seeded with its current title. */
export function beginChapterRename(): void {
  const row = soleSelectedLibraryChapter.value
  if (row === null) return
  chapterRenameDraft.value = row.title ?? ''
  chapterRenameEditing.value = true
}

export function cancelChapterRename(): void {
  chapterRenameEditing.value = false
}

/** Nhịp chung của `moveCurrentChapterUp`/`moveCurrentChapterDown` — chỉ khác nhau ở hướng. */
async function moveCurrentChapter(direction: ChapterDirection): Promise<void> {
  const row = soleSelectedLibraryChapter.value
  if (row === null) return
  if (!(await beginChapterReorg())) return

  const { error } = await moveChapter(row.chapter_id, direction)
  if (error !== null) {
    chapterReorgError.value = error
    chapterReorgBusy.value = false
    return
  }

  await loadChapters()
  syncCursorToChapter(row.chapter_id)
  chapterReorgBusy.value = false
}

/** Dời Chương ĐANG CHỌN lên — lệnh `library.chapter_move_up`. */
export async function moveCurrentChapterUp(): Promise<void> {
  await moveCurrentChapter('prev')
}

/** Dời Chương ĐANG CHỌN xuống — lệnh `library.chapter_move_down`. */
export async function moveCurrentChapterDown(): Promise<void> {
  await moveCurrentChapter('next')
}

/**
 * **Sửa bốn ô xuất xứ của Chương ĐANG CHỌN** — Story 6.15, FR128/AD-43. No-op trên danh sách
 * rỗng. Không đi qua `beginChapterReorg()` (không flush Editor) — xem doc-comment
 * `chapterOriginBusy`.
 *
 * Gọi thẳng từ `@commit` của `ChapterOrigin.vue`, KHÔNG qua `dispatch('<id>')`: đây là một sự
 * kiện MANG THAM SỐ (bốn trường vừa gõ), và `@change`/`@commit` (custom event của component
 * con) nằm NGOÀI phạm vi Kiểm A của `check:commands` (chỉ canh `@click`).
 *
 * `edit`: `null` ⇔ ô đó CHƯA CHẠM — ở đây (khác `ImportPreviewOverlay.vue`, nơi `null` giữ
 * nguyên giá trị MÁY phía Rust) không có "giá trị máy" nào để mà giữ, nên ô chưa chạm được
 * điền lại bằng giá trị ĐÃ TRÊN ĐĨA của CHÍNH Chương này (`row.origin_*`) trước khi gửi —
 * `update_chapter_origin` nhận đủ bốn trường, không phân biệt "chưa chạm"/"giá trị cũ".
 */
export async function saveCurrentChapterOrigin(edit: ChapterOriginEditFields): Promise<void> {
  const row = soleSelectedLibraryChapter.value
  if (row === null) return
  chapterOriginBusy.value = true
  chapterOriginError.value = null

  const resolved: ChapterOriginEdit = {
    author: edit.author ?? (row.origin_author ?? ''),
    siteName: edit.siteName ?? (row.origin_site_name ?? ''),
    url: edit.url ?? (row.origin_url ?? ''),
    publishedAt: edit.publishedAt ?? (row.origin_published_at ?? ''),
  }

  const { error } = await updateChapterOrigin(row.chapter_id, resolved, chapterOriginApplyThroughOrd.value)
  if (error !== null) {
    chapterOriginError.value = error
    chapterOriginBusy.value = false
    return
  }

  chapterOriginApplyThroughOrd.value = null
  await loadChapters()
  chapterOriginBusy.value = false
}

/** Ghi ord đích của lượt áp xuất xứ THEO ĐOẠN — `@change` của ô số bên cạnh khối xuất xứ,
 * cùng lý do `saveCurrentChapterOrigin` không qua `dispatch('<id>')`. Chuỗi rỗng/không phải
 * số ⇒ tắt (đoạn về lại một Chương). Ord phải nằm trong `[Chương đang chọn, Chương cuối đã
 * tải]` — ngoài dải đó cũng tắt, để một lượt gõ nhầm không lặng lẽ nới rộng đoạn ghi đè. */
export function setChapterOriginApplyThroughOrd(raw: string): void {
  const trimmed = raw.trim()
  if (trimmed === '') {
    chapterOriginApplyThroughOrd.value = null
    return
  }
  const parsed = Number(trimmed)
  const row = soleSelectedLibraryChapter.value
  const lastOrd = chapters.value.at(-1)?.ord ?? null
  if (!Number.isInteger(parsed) || row === null || lastOrd === null || parsed < row.ord || parsed > lastOrd) {
    chapterOriginApplyThroughOrd.value = null
    return
  }
  chapterOriginApplyThroughOrd.value = parsed
}

// ═════════════════════════════════════════════════════════════════════════════════
// Work page: selection, keyboard cursor, bulk status, multi-Chapter merge
// ═════════════════════════════════════════════════════════════════════════════════

function syncCursorToChapter(chapterId: number): void {
  const index = chapters.value.findIndex((row) => row.chapter_id === chapterId)
  if (index !== -1) chapterCursor.value = index
}

function syncCursorToSoleSelection(): void {
  const only = soleSelectedLibraryChapter.value
  if (only !== null) syncCursorToChapter(only.chapter_id)
}

/** Pointer or key selection of a Chapter row. A sole selection always moves the cursor with it. */
export function selectLibraryChapter(chapterId: number, mods: ChapterClickModifiers): void {
  const result = applyChapterClick(
    libraryChapterSelectedIds.value,
    libraryChapterAnchorId.value,
    visibleChapterIds(libraryChapterView.value.items),
    chapterId,
    mods,
  )
  setChapterSelection(result.selected, result.anchor)
  closeChapterStatusMenu()
  if (result.selected.size === 1) syncCursorToSoleSelection()
  else syncCursorToChapter(chapterId)
}

/** Up/Down on the list: moves the cursor over the visible rows; Shift extends the range. */
export function moveLibraryChapterSelection(step: 1 | -1, extend: boolean): void {
  const visible = visibleChapterIds(libraryChapterView.value.items)
  if (visible.length === 0) return
  const current = chapters.value.at(chapterCursor.value)?.chapter_id
  const at = current === undefined ? -1 : visible.indexOf(current)
  const next = at === -1 ? (step === 1 ? 0 : visible.length - 1) : Math.min(visible.length - 1, Math.max(0, at + step))
  const targetId = visible[next]
  selectLibraryChapter(targetId, { toggle: false, range: extend })
}

/** Space on the list: toggles the cursor row in the selection. */
export function toggleLibraryChapterAtCursor(): void {
  const row = chapters.value.at(chapterCursor.value)
  if (row === undefined) return
  selectLibraryChapter(row.chapter_id, { toggle: true, range: false })
}

export function clearLibraryChapterSelection(): void {
  clearChapterSelection()
}

/** Flat-list index of the cursor row inside the filtered view, or -1 when it is filtered out. */
export function libraryChapterCursorFlatIndex(): number {
  const row = chapters.value.at(chapterCursor.value)
  return row === undefined ? -1 : flatIndexOfChapter(libraryChapterView.value.items, row.chapter_id)
}

/** Bulk status change for the selected Chapters (one Rust transaction). */
export async function setSelectedChaptersStatus(status: string): Promise<void> {
  const ids = libraryChapterSelectedRows.value.map((row) => row.chapter_id)
  if (ids.length === 0 || chapterReorgBusy.value) return
  chapterReorgBusy.value = true
  chapterReorgError.value = null
  chapterReorgNotice.value = null
  closeChapterStatusMenu()

  const { lifecycle, error } = await setChaptersStatus(ids, status)
  if (error !== null) {
    chapterReorgError.value = error
    chapterReorgBusy.value = false
    return
  }
  applyOpenWorkLifecycleResult(lifecycle)
  await loadChapters()
  chapterReorgBusy.value = false
}

export type MergeChaptersOutcome = 'merged' | 'blocked' | 'refused'

/**
 * Merges the selected, contiguous Chapters into the first one. Flushes the Editor first; reloads
 * the Editor when the open Chapter was one of the merged ones.
 */
export async function mergeSelectedChapters(): Promise<MergeChaptersOutcome> {
  const picked = libraryChapterSelectedRows.value
  if (picked.length < 2) return 'refused'
  if (!(await beginChapterReorg())) return 'blocked'

  const ids = picked.map((row) => row.chapter_id)
  const { chapters: rebuilt, error } = await mergeChapters(ids)
  if (error !== null) {
    chapterReorgError.value = error
    chapterReorgBusy.value = false
    return 'refused'
  }
  if (rebuilt === null) {
    chapterReorgBusy.value = false
    return 'refused'
  }

  const survivorId = picked[0].chapter_id
  const openId = editorChapterId.value
  chapters.value = rebuilt
  chaptersHaveLoaded.value = true
  clampChapterCursor()
  setChapterSelection(new Set([survivorId]), survivorId)
  syncCursorToChapter(survivorId)

  if (openId !== null && ids.includes(openId)) {
    resetEditorPanel()
    resetSourcePanel()
    resetReading()
    resetReadingToc()
    await ensureChapterLoaded()
    await ensureSegmentsLoaded()
    await nextTick()
    enterFocus('panel.grid')
  }
  chapterReorgBusy.value = false
  return 'merged'
}

/**
 * 🔴 Vứt toàn bộ state — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`. Dùng bởi bàn đo/test; sản phẩm không có chỗ gọi (khối này sống suốt phiên).
 */
export function resetLibraryChapters(): void {
  chaptersSequence += 1
  chapters.value = []
  chaptersHaveLoaded.value = false
  chaptersBusy.value = false
  chaptersReloadPending = false
  chaptersError.value = null
  chapterCursor.value = 0
  openWorkBusy.value = false
  openWorkNotice.value = null
  openWorkError.value = null
  chapterRenameDraft.value = ''
  chapterRenameEditing.value = false
  workPageWork.value = null
  resetLibraryChapterSelection()
  chapterReorgBusy.value = false
  chapterReorgNotice.value = null
  chapterReorgError.value = null
  chapterOriginBusy.value = false
  chapterOriginError.value = null
  chapterOriginApplyThroughOrd.value = null
}

/** Hình dạng trả về của [`chapterWindow`] — chỉ số MẢNG (nửa mở `[start, end)`), cộng hai
 * đệm tính sẵn bằng PIXEL cho hai `<li>` trên/dưới của danh sách CÓ CỬA SỔ (AC2). */
export type ChapterWindowSlice = {
  start: number
  end: number
  padTop: number
  padBottom: number
}

/**
 * **Hàm THUẦN** — cửa sổ hiển thị của một danh sách CUỘN ẢO, Story 5.7 AC2. Không đọc DOM,
 * không đọc đồng hồ — tách khỏi `.vue` để vitest kiểm được TẤT ĐỊNH trên `happy-dom` (không
 * có hình học thật, xem `tests/AGENTS.md`).
 *
 * `overscan` là số hàng đệm THÊM ở mỗi đầu, để một cú cuộn nhanh không thấy khoảng trắng
 * trước khi Vue kịp vá DOM. Chiều cao hàng **cố định** (từ token) — chiều cao đổi theo nội
 * dung sẽ đòi đo hình học thật, thuộc bàn đo/e2e chứ không thuộc vitest (§Design Notes).
 */
export function chapterWindow(
  scrollTop: number,
  viewportHeight: number,
  rowHeight: number,
  total: number,
  overscan: number,
): ChapterWindowSlice {
  if (total <= 0 || rowHeight <= 0) return { start: 0, end: 0, padTop: 0, padBottom: 0 }

  const firstVisible = Math.floor(Math.max(0, scrollTop) / rowHeight)
  const visibleCount = Math.ceil(Math.max(0, viewportHeight) / rowHeight) + 1

  const start = Math.max(0, firstVisible - overscan)
  const end = Math.min(total, firstVisible + visibleCount + overscan)

  return {
    start,
    end,
    padTop: start * rowHeight,
    padBottom: (total - end) * rowHeight,
  }
}
