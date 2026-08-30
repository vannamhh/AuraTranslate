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
import { computed, readonly, ref, watch } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { listChapters, mergeChapterIntoPrevious, moveChapter, renameChapter } from '../config/chapter'
import type { ChapterDirection, ChapterRow } from '../config/chapter'
import { openWork } from '../config/library'
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
import { currentLibraryWork } from './libraryWorks'
// Story 5.11 — Chế độ đọc mang CÙNG lớp cache module-level (`readingState.ts`), nên nó phải
// đi cùng lượt vứt state Tác phẩm/Chương này — cùng lý lẽ `resetEditorPanel` đã ghi.
import { resetReading, resetReadingToc } from './readingState'
import type { IpcError } from '../i18n'

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
/** Kết cục một lượt tổ chức bị CHẶN vì tập chờ Editor chưa sạch — cùng khuôn `OpenWorkNotice`
 * ngay trên, và cùng hai giá trị: §Always của story chỉ đòi phân biệt được hai ca đó. */
export type ChapterReorgNotice = 'flush-failed' | 'still-dirty'
const chapterReorgNotice = ref<ChapterReorgNotice | null>(null)
const chapterReorgError = ref<IpcError | null>(null)

export const libraryChapters: DeepReadonly<Ref<ChapterRow[]>> = readonly(chapters)
export const libraryChaptersHaveLoaded: DeepReadonly<Ref<boolean>> = readonly(chaptersHaveLoaded)
export const libraryChaptersBusy: DeepReadonly<Ref<boolean>> = readonly(chaptersBusy)
export const libraryChaptersError: DeepReadonly<Ref<IpcError | null>> = readonly(chaptersError)
export const libraryChapterCursor: DeepReadonly<Ref<number>> = readonly(chapterCursor)
export const libraryOpenWorkBusy: DeepReadonly<Ref<boolean>> = readonly(openWorkBusy)
export const libraryOpenWorkNotice: DeepReadonly<Ref<OpenWorkNotice | null>> = readonly(openWorkNotice)
export const libraryOpenWorkError: DeepReadonly<Ref<IpcError | null>> = readonly(openWorkError)
export const libraryChapterReorgBusy: DeepReadonly<Ref<boolean>> = readonly(chapterReorgBusy)
export const libraryChapterReorgNotice: DeepReadonly<Ref<ChapterReorgNotice | null>> = readonly(chapterReorgNotice)
export const libraryChapterReorgError: DeepReadonly<Ref<IpcError | null>> = readonly(chapterReorgError)

/**
 * Chương ĐANG CHỌN trong danh sách, hoặc `null` nếu con trỏ ngoài phạm vi (danh sách rỗng,
 * hoặc chưa tải lần nào). `.at(cursor.value)`, KHÔNG `[cursor.value]` — cùng lý do
 * `currentLibraryWork` (`libraryWorks.ts`): `noUncheckedIndexedAccess` không bật, `.at()`
 * khai đúng `T | undefined`.
 */
export const currentLibraryChapter = computed<ChapterRow | null>(
  () => chapters.value.at(chapterCursor.value) ?? null,
)

/**
 * 🔴 **Ô nhập tên đi THEO Chương đang chọn — và ô nhớ này phải được đồng bộ, không để trôi.**
 *
 * ⚠️ **Lỗ bắt được ở lượt rà 2026-08-29:** `chapterRenameDraft` là một ô nhớ cấp module, chỉ
 * được dọn bởi `resetLibraryChapters()`. Không có khối này, dời con trỏ sang một Chương KHÁC
 * rồi bấm "Đổi tên" **mà không chạm ô nhập** sẽ áp chữ còn sót của Chương TRƯỚC lên Chương
 * mới — một lượt ghi dữ liệu người dùng, im lặng, không lỗi nào. Cùng hạng *"một kết quả sai
 * trông như bình thường"* mà `AGENTS.md::Known pitfalls` đặt lên đầu.
 *
 * 🔴 **Theo dõi `chapter_id`, KHÔNG theo dõi chính đối tượng hàng.** `loadChapters()` thay cả
 * mảng ở mỗi lượt nạp (sau mỗi thao tác tổ chức), nên một `watch` trên `currentLibraryChapter`
 * sẽ bắn cả khi Chương đang chọn KHÔNG đổi — và nó sẽ giẫm lên chữ người dùng đang gõ dở. Danh
 * tính của "Chương đang chọn" là `chapter_id`, và đó là thứ đáng nghe.
 *
 * Mồi bằng tên hiện thời (không phải chuỗi rỗng): lượt đổi tên trở thành một lượt SỬA thấy
 * được chữ cũ, thay vì một lượt ghi đè mù.
 */
watch(
  () => currentLibraryChapter.value?.chapter_id ?? null,
  () => {
    chapterRenameDraft.value = currentLibraryChapter.value?.title ?? ''
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
export async function openWorkById(workId: string): Promise<void> {
  if (openWorkBusy.value) return

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
    return
  }

  // Vị trí đang chờ ghi của Tác phẩm CŨ cũng phải rời đi trước khi `OpenWorkState` đổi chỗ —
  // KHÔNG chặn (mất một lượt chỉ mất một lời nhắc, không mất bản dịch).
  await flushChapterPositionNow()

  const { opened, error } = await openWork(workId)

  if (error !== null) {
    openWorkError.value = error
    openWorkBusy.value = false
    return
  }
  if (opened === null) {
    // Không có cầu IPC -- im lặng, cùng nhánh mọi adapter khác.
    openWorkBusy.value = false
    return
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
  chapters.value = []
  chaptersHaveLoaded.value = false
  chapterCursor.value = 0

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

  // Danh sách Chương của Tác phẩm mới — bề mặt riêng của story này. (`chapterCursor` đã về 0
  // ở khối vứt phía trên, cùng lượt với `chapters`/`chaptersHaveLoaded`.)
  await loadChapters()

  // Nhả cờ bận SAU CÙNG — xem khối 🔴 "CỬA SỔ MỞ NHẦM CHƯƠNG" phía trên.
  openWorkBusy.value = false
}

/** Mở Tác phẩm ĐANG CHỌN trong lưới — nút "Mở Tác phẩm" của mỗi ô, cùng khuôn
 * `forgetCurrentLibraryOrphan` (`libraryRescan.ts`): đọc con trỏ hiện thời, không nhận tham
 * số qua `dispatch()`. No-op khi không có ô nào đang chọn. */
export async function openCurrentLibraryWork(): Promise<void> {
  const work = currentLibraryWork.value
  if (work === null) return
  await openWorkById(work.work_id)
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
  const row = currentLibraryChapter.value
  if (row === null) return
  if (!(await beginChapterReorg())) return

  const { error } = await renameChapter(row.chapter_id, chapterRenameDraft.value)
  if (error !== null) {
    chapterReorgError.value = error
    chapterReorgBusy.value = false
    return
  }

  await loadChapters()
  chapterReorgBusy.value = false
}

/** Nhịp chung của `moveCurrentChapterUp`/`moveCurrentChapterDown` — chỉ khác nhau ở hướng. */
async function moveCurrentChapter(direction: ChapterDirection): Promise<void> {
  const row = currentLibraryChapter.value
  if (row === null) return
  if (!(await beginChapterReorg())) return

  const { error } = await moveChapter(row.chapter_id, direction)
  if (error !== null) {
    chapterReorgError.value = error
    chapterReorgBusy.value = false
    return
  }

  await loadChapters()
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

/** Gộp Chương ĐANG CHỌN vào Chương liền trước nó — lệnh `library.chapter_merge_up`.
 *
 * 🔴 Chương liền trước (A) và Chương đang chọn (B) đều có thể là "Chương đang mở" của Editor
 * (`editorChapterId`) — B biến mất, A đổi nội dung. Cả hai id lấy được TỪ TRƯỚC lượt gọi IPC:
 * B là chính hàng đang chọn; A là hàng liền trước nó trong `chapters.value`, danh sách đã
 * tải sẵn (đúng thứ tự `(ord, id)` mà Rust trả về, `libraryChapters.ts::loadChapters`) —
 * không cần đoán, không cần một lượt IPC phụ chỉ để hỏi "ai là Chương trước". */
export async function mergeCurrentChapterUp(): Promise<void> {
  const row = currentLibraryChapter.value
  if (row === null) return
  if (!(await beginChapterReorg())) return
  // 🔴 ĐỌC SAU lượt flush, và KẸP BIÊN DƯỚI — hai lỗi của bản đầu, cùng bắt được ở lượt rà
  // 2026-08-29:
  //   ① `chapters.value.at(chapterCursor.value - 1)` với con trỏ ở **0** cho `.at(-1)`, tức
  //      phần tử CUỐI mảng, không phải `undefined`. `Array.prototype.at` nhận chỉ số ÂM là
  //      đếm ngược — một hành vi đúng của ngôn ngữ, đọc ra thành một bug ở đây.
  //   ② đọc TRƯỚC `beginChapterReorg()` là đọc trước một lượt `await` có thể kéo tới trần
  //      cứng 5 giây của AD-35, tức một ảnh chụp có thể đã cũ khi lượt IPC bay đi.
  // Hậu quả của ① hôm nay hẹp (một lượt gộp ở con trỏ 0 luôn bị Rust từ chối bằng
  // `chapter.at_first`, nên hàm trả về trước khi dùng tới `previousRow`) — nhưng nó là một
  // giá trị SAI nằm sẵn chờ đường gọi thứ hai, và cửa nó mở ra là *"Editor không nạp lại sau
  // một lượt gộp"*.
  const previousRow = chapterCursor.value > 0 ? (chapters.value.at(chapterCursor.value - 1) ?? null) : null

  const { error } = await mergeChapterIntoPrevious(row.chapter_id)
  if (error !== null) {
    chapterReorgError.value = error
    chapterReorgBusy.value = false
    return
  }

  // Chương đang mở của Editor bị ẢNH HƯỞNG ⇔ nó là A hoặc B — vứt state Chương của Editor rồi
  // nạp lại NGAY, cùng khuôn `openWorkById`/`openChapterById` (`resetEditorPanel()` +
  // `resetSourcePanel()` rồi `ensureChapterLoaded()` + `ensureSegmentsLoaded()`).
  const openId = editorChapterId.value
  if (openId !== null && (openId === row.chapter_id || openId === previousRow?.chapter_id)) {
    resetEditorPanel()
    resetSourcePanel()
    // 🔵 THÊM Story 5.11 — `OpenWork::chapter_id` là MỘT con trỏ duy nhất phía Rust, dùng
    // chung giữa Editor và Chế độ đọc; Chương đang đọc (nếu có) cũng vừa bị gộp/đổi số.
    resetReading()
    resetReadingToc()
    await ensureChapterLoaded()
    await ensureSegmentsLoaded()
  }

  await loadChapters()
  chapterReorgBusy.value = false
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
  chapterReorgBusy.value = false
  chapterReorgNotice.value = null
  chapterReorgError.value = null
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
