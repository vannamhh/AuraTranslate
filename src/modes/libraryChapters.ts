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
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { listChapters } from '../config/chapter'
import type { ChapterRow } from '../config/chapter'
import { openWork } from '../config/library'
import {
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

export const libraryChapters: DeepReadonly<Ref<ChapterRow[]>> = readonly(chapters)
export const libraryChaptersHaveLoaded: DeepReadonly<Ref<boolean>> = readonly(chaptersHaveLoaded)
export const libraryChaptersBusy: DeepReadonly<Ref<boolean>> = readonly(chaptersBusy)
export const libraryChaptersError: DeepReadonly<Ref<IpcError | null>> = readonly(chaptersError)
export const libraryChapterCursor: DeepReadonly<Ref<number>> = readonly(chapterCursor)
export const libraryOpenWorkBusy: DeepReadonly<Ref<boolean>> = readonly(openWorkBusy)
export const libraryOpenWorkNotice: DeepReadonly<Ref<OpenWorkNotice | null>> = readonly(openWorkNotice)
export const libraryOpenWorkError: DeepReadonly<Ref<IpcError | null>> = readonly(openWorkError)

/**
 * Chương ĐANG CHỌN trong danh sách, hoặc `null` nếu con trỏ ngoài phạm vi (danh sách rỗng,
 * hoặc chưa tải lần nào). `.at(cursor.value)`, KHÔNG `[cursor.value]` — cùng lý do
 * `currentLibraryWork` (`libraryWorks.ts`): `noUncheckedIndexedAccess` không bật, `.at()`
 * khai đúng `T | undefined`.
 */
export const currentLibraryChapter = computed<ChapterRow | null>(
  () => chapters.value.at(chapterCursor.value) ?? null,
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
