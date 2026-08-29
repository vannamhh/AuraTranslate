/**
 * State + thao tác của khối "Tác phẩm" (danh sách + bộ lọc bốn trạng thái) VÀ khối "Trạng
 * thái Tác phẩm đang mở" ở Library — Story 5.4, FR5/FR6.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ VÌ SAO MỘT MODULE THUẦN RIÊNG, KHÔNG VIẾT THẲNG TRONG `LibraryMode.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `libraryRescan.ts`: AD-34 §1 đòi mọi `@click` là đúng một `dispatch('<id>')`,
 * và chín thao tác (`library.list_works` · bốn `library.filter_*` · `library.filter_clear` ·
 * ba `lifecycle.*`) đăng ký ở `src/commands/index.ts` như các `CommandDeps` TIÊM VÀO —
 * `src/main.ts` nối chúng vào `installCommands({...})`. Module này là phía CUNG CẤP.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 `worksHaveLoaded`/`openWorkLifecycleHasLoaded` — LÝ DO CHÚNG TỒN TẠI (`AGENTS.md::Known pitfalls`)
 * ─────────────────────────────────────────────────────────────────────────────
 * "Library trống thật" chỉ được phép nói SAU khi đã tải danh sách ít nhất một lần trong
 * phiên này. Trước lượt tải đầu, danh sách cũng rỗng — nhưng đó là "chưa biết", không phải
 * "không có". `LibraryMode.vue` phải hỏi hai vị từ này TRƯỚC khi kết luận.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 HAI CON SỐ, MỘT LƯỢT ĐỌC — `worksTotal`/`worksMatched` KHÔNG BAO GIỜ SUY TỪ `.length`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cả hai đến THẲNG từ `WorkListReport` (`total`/`matched`, đã tính ở Rust trong CÙNG một
 * lượt `Store::read`) — Story 3.9 từng bịa `totalCount` bằng chính `filteredCount`, và đó là
 * đúng lớp lỗi mà hai trường tường minh này tồn tại để chặn.
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { listLibraryWorks } from '../config/library'
import type { WorkRow, WorkSortKey } from '../config/library'
import { readWorkLifecycle, setChapterStatus, setWorkStatusOverride } from '../config/lifecycle'
import { editorChapterId } from '../panels/editorPanelState'
import type { IpcError } from '../i18n'

/** Bốn giá trị trên dây — khớp `LifecycleStatus::ALL` phía Rust (`core/lifecycle/mod.rs`). */
export const LIFECYCLE_STATUS_VALUES: readonly string[] = ['not_started', 'in_progress', 'paused', 'done']

// ─────────────────────────────────────────────────────────────────────────────
// State module-level — singleton của cả tiến trình, cùng khuôn libraryRescan.ts. Mỗi khai
// báo NẰM TRÊN MỘT DÒNG (`check:panel-refs` Kiểm 5 — cú pháp ngoài tập con cho ĐỎ).
// ─────────────────────────────────────────────────────────────────────────────
const works = ref<WorkRow[]>([])
const worksTotal = ref(0)
const worksMatched = ref(0)
const statusFilter = ref<Set<string>>(new Set())
// 🔵 THÊM (2026-08-28, Story 5.6) — `null` ⇒ không lọc, cùng ngữ nghĩa `genre: None` phía
// Rust. Khác `statusFilter` (một `Set`, danh mục ĐÓNG bốn giá trị): lĩnh vực/ngôn ngữ nguồn
// là tập MỞ, nên ĐÚNG MỘT giá trị đang chọn (hoặc không giá trị nào) — một `<select>`, không
// bốn nút.
const genreFilter = ref<string | null>(null)
const sourceLangFilter = ref<string | null>(null)
// Khoá sắp — danh mục ĐÓNG hai giá trị, khớp `WorkSortKey` phía Rust. Mặc định `updated_desc`
// (§I/O Matrix "Sắp mặc định", FR10 nêu ngày sửa trước).
const sortKey = ref<WorkSortKey>('updated_desc')
// Hai tập lựa chọn CÓ THẬT cho hai `<select>` lĩnh vực/ngôn ngữ — luôn đến từ
// `WorkListReport.genres`/`source_langs` (Rust, `DISTINCT` trên bảng CHƯA LỌC), KHÔNG BAO
// GIỜ suy từ `works.value` (AD-1, §Always: suy vậy làm lựa chọn TEO DẦN theo mỗi lượt lọc).
const genres = ref<string[]>([])
const sourceLangs = ref<string[]>([])
// Con trỏ ô đang chọn trong lưới — chép khuôn `orphanCursor` của `libraryRescan.ts` (AC7).
const workCursor = ref(0)
const worksHaveLoaded = ref(false)
const worksBusy = ref(false)
// Một lượt `loadWorks()` bị chặn vì đang bận, đang chờ chạy lại. Không phải `ref` -- không
// khối template nào đọc nó, và một `ref` thừa là một thứ `check:panel-refs` phải đếm.
let worksReloadPending = false
const worksError = ref<IpcError | null>(null)

const openWorkStatus = ref<string | null>(null)
const openWorkIsOverride = ref(false)
const openWorkLifecycleHasLoaded = ref(false)
const openWorkBusy = ref(false)
const openWorkError = ref<IpcError | null>(null)

/** Chặn một lượt CŨ ghi đè lên state của một lượt MỚI hơn — cùng khuôn `libraryRescan.ts`.
 * Hai bộ đếm RIÊNG: khối "Tác phẩm" và khối "Tác phẩm đang mở" là hai luồng IPC độc lập. */
let worksSequence = 0
let lifecycleSequence = 0

export const libraryWorks: DeepReadonly<Ref<WorkRow[]>> = readonly(works)
export const libraryWorksTotal: DeepReadonly<Ref<number>> = readonly(worksTotal)
export const libraryWorksMatched: DeepReadonly<Ref<number>> = readonly(worksMatched)
export const libraryStatusFilter: DeepReadonly<Ref<Set<string>>> = readonly(statusFilter)
export const libraryGenreFilter: DeepReadonly<Ref<string | null>> = readonly(genreFilter)
export const librarySourceLangFilter: DeepReadonly<Ref<string | null>> = readonly(sourceLangFilter)
export const librarySortKey: DeepReadonly<Ref<WorkSortKey>> = readonly(sortKey)
export const libraryGenres: DeepReadonly<Ref<string[]>> = readonly(genres)
export const librarySourceLangs: DeepReadonly<Ref<string[]>> = readonly(sourceLangs)
export const libraryWorkCursor: DeepReadonly<Ref<number>> = readonly(workCursor)
export const libraryWorksHaveLoaded: DeepReadonly<Ref<boolean>> = readonly(worksHaveLoaded)
export const libraryWorksBusy: DeepReadonly<Ref<boolean>> = readonly(worksBusy)
export const libraryWorksError: DeepReadonly<Ref<IpcError | null>> = readonly(worksError)

/**
 * Tác phẩm ĐANG CHỌN trong lưới, hoặc `null` nếu con trỏ ngoài phạm vi (danh sách rỗng, hoặc
 * chưa tải lần nào). `.at(cursor.value)`, KHÔNG `[cursor.value]` — cùng lý do
 * `currentLibraryOrphan` (`libraryRescan.ts`): `noUncheckedIndexedAccess` không bật, `.at()`
 * khai đúng `T | undefined`.
 */
export const currentLibraryWork = computed<WorkRow | null>(() => works.value.at(workCursor.value) ?? null)

export const openWorkLifecycleStatus: DeepReadonly<Ref<string | null>> = readonly(openWorkStatus)
export const openWorkLifecycleIsOverride: DeepReadonly<Ref<boolean>> = readonly(openWorkIsOverride)
export const openWorkLifecycleLoaded: DeepReadonly<Ref<boolean>> = readonly(openWorkLifecycleHasLoaded)
export const openWorkLifecycleBusy: DeepReadonly<Ref<boolean>> = readonly(openWorkBusy)
export const openWorkLifecycleError: DeepReadonly<Ref<IpcError | null>> = readonly(openWorkError)

/**
 * `true` ⇔ CẢ BA bộ lọc đều RỖNG (không lọc gì) — cùng ngữ nghĩa "không lọc" phía Rust cho cả
 * trạng thái, lĩnh vực, ngôn ngữ nguồn.
 *
 * 🔵 **MỞ RỘNG (2026-08-28, Story 5.6)** — trước đó chỉ kiểm `statusFilter`; nút "Bỏ lọc"
 * (`data-lifecycle-action="clear_filter"`) giờ bỏ CẢ BA bộ lọc cùng lúc, nên điều kiện bật/tắt
 * của nó phải kiểm CẢ BA — nút "sáng" (không `disabled`) khi bất kỳ bộ lọc nào đang bật.
 */
export const libraryFilterIsEmpty = computed<boolean>(
  () => statusFilter.value.size === 0 && genreFilter.value === null && sourceLangFilter.value === null,
)

function clampWorkCursor(): void {
  const maxIndex = works.value.length - 1
  if (workCursor.value > maxIndex) workCursor.value = Math.max(0, maxIndex)
  if (workCursor.value < 0) workCursor.value = 0
}

/**
 * Đọc lại danh sách Tác phẩm theo bộ lọc/khoá sắp HIỆN THỜI — lệnh `library.list_works`.
 *
 * Bộ lọc trạng thái RỖNG ⇒ gửi `undefined` (không lọc), đúng khuôn `filter = None` phía Rust
 * — một `Set` rỗng và "không lọc" là CÙNG một ý định, không hai trạng thái khác nhau.
 * `genreFilter`/`sourceLangFilter` là `string | null` sẵn nên gửi thẳng (`null` ⇒ `undefined`
 * ở adapter, xem `config/library.ts`).
 */
export async function loadWorks(): Promise<void> {
  // 🔵 SỬA (2026-08-28, lượt rà) — bản trước `return` THẲNG khi đang bận, tức NUỐT lượt tải
  // lại. Bốn nút lọc gọi `loadWorks()` sau mỗi lượt bật/tắt, nên bấm nút thứ hai trong lúc
  // lượt đầu còn bay làm danh sách đứng lại ở BỘ LỌC CŨ trong khi các nút đã hiện bộ lọc MỚI
  // -- màn hình khẳng định một kết quả không thuộc về câu hỏi đang hiển thị. Nay lượt bị chặn
  // được GHI NHỚ và chạy lại ngay khi lượt đang bay kết thúc.
  if (worksBusy.value) {
    worksReloadPending = true
    return
  }

  worksBusy.value = true
  worksError.value = null
  const mySequence = ++worksSequence

  const filter = statusFilter.value.size === 0 ? undefined : Array.from(statusFilter.value)
  const result = await listLibraryWorks(
    filter,
    genreFilter.value ?? undefined,
    sourceLangFilter.value ?? undefined,
    sortKey.value,
  )
  if (mySequence !== worksSequence) return // Một lượt MỚI hơn đã bắt đầu -- bỏ, không ghi đè.

  worksBusy.value = false
  if (result.error !== null) {
    worksError.value = result.error
    await runPendingWorksReload()
    return
  }
  if (result.report === null) {
    // Không có cầu IPC -- im lặng, cùng nhánh mọi adapter khác.
    await runPendingWorksReload()
    return
  }

  works.value = result.report.works
  worksTotal.value = result.report.total
  worksMatched.value = result.report.matched
  // Hai tập lựa chọn -- LUÔN chép nguyên vẹn từ Rust, không bao giờ suy từ `works.value`.
  genres.value = result.report.genres
  sourceLangs.value = result.report.source_langs
  worksHaveLoaded.value = true
  // §I/O Matrix "Con trỏ ra ngoài sau lọc" -- một lượt lọc làm danh sách ngắn đi phải kẹp con
  // trỏ về ô cuối cùng còn lại NGAY LẬP TỨC, trước khi bất kỳ ai đọc `currentLibraryWork`.
  clampWorkCursor()
  await runPendingWorksReload()
}

/** Đặt bộ lọc lĩnh vực (hoặc bỏ, khi `value === null`) rồi tải lại — chọn từ
 * `<select data-library-genre-filter>`, dựng `<option>` từ [`libraryGenres`]. */
export function setGenreFilter(value: string | null): void {
  if (genreFilter.value === value) return
  genreFilter.value = value
  void loadWorks()
}

/** Đặt bộ lọc ngôn ngữ nguồn (hoặc bỏ) rồi tải lại — cùng khuôn [`setGenreFilter`]. */
export function setSourceLangFilter(value: string | null): void {
  if (sourceLangFilter.value === value) return
  sourceLangFilter.value = value
  void loadWorks()
}

/** Đổi khoá sắp rồi tải lại (AC4). */
export function setSortKey(value: WorkSortKey): void {
  if (sortKey.value === value) return
  sortKey.value = value
  void loadWorks()
}

/** Chuyển con trỏ lưới xuống ô kế tiếp — không vòng (AC7). No-op trên danh sách rỗng. */
export function nextWork(): void {
  if (workCursor.value < works.value.length - 1) workCursor.value += 1
}

/** Chuyển con trỏ lưới lên ô trước — không vòng. No-op trên danh sách rỗng. */
export function prevWork(): void {
  if (workCursor.value > 0) workCursor.value -= 1
}

/// Chạy lượt tải lại đã bị chặn, nếu có. Gọi ở MỌI đường ra của `loadWorks` sau khi
/// `worksBusy` đã tắt -- một đường ra quên gọi nó là đúng lỗi mà bản vá này đang đóng.
async function runPendingWorksReload(): Promise<void> {
  if (!worksReloadPending) return
  worksReloadPending = false
  await loadWorks()
}

/** Bật/tắt một giá trị lọc, RIÊNG RẼ với ba giá trị còn lại, rồi tải lại. */
export function toggleStatusFilter(status: string): void {
  const next = new Set(statusFilter.value)
  if (next.has(status)) {
    next.delete(status)
  } else {
    next.add(status)
  }
  statusFilter.value = next
  void loadWorks()
}

/**
 * Bỏ MỌI bộ lọc đang bật (trạng thái, lĩnh vực, ngôn ngữ nguồn) rồi tải lại. No-op nếu cả ba
 * đã không lọc gì.
 *
 * 🔵 **MỞ RỘNG (2026-08-28, Story 5.6)** — trước đó chỉ bỏ `statusFilter`. Tên hàm GIỮ NGUYÊN
 * (`clearStatusFilter`, đúng id lệnh `library.filter_clear` đã đăng ký ở `commands/index.ts`)
 * dù nay bỏ CẢ BA — đổi tên sẽ đòi sửa lại lệnh/nhãn i18n cho một hành vi vẫn phục vụ đúng
 * một nút "Bỏ lọc" duy nhất trên màn hình.
 */
export function clearStatusFilter(): void {
  if (libraryFilterIsEmpty.value) return
  statusFilter.value = new Set()
  genreFilter.value = null
  sourceLangFilter.value = null
  void loadWorks()
}

/** Đọc trạng thái vòng đời của Tác phẩm ĐANG MỞ — lệnh `read_work_lifecycle`. Chưa Tác phẩm
 * nào mở ⇒ lỗi `work.none_open` đi vào `openWorkLifecycleError`, không phải một ngoại lệ. */
export async function loadOpenWorkLifecycle(): Promise<void> {
  if (openWorkBusy.value) return

  openWorkBusy.value = true
  openWorkError.value = null
  const mySequence = ++lifecycleSequence

  const result = await readWorkLifecycle()
  if (mySequence !== lifecycleSequence) return

  openWorkBusy.value = false
  if (result.error !== null) {
    openWorkError.value = result.error
    return
  }
  if (result.lifecycle === null) return

  openWorkStatus.value = result.lifecycle.status
  openWorkIsOverride.value = result.lifecycle.status_is_override
  openWorkLifecycleHasLoaded.value = true
}

function applyOpenWorkLifecycleResult(
  lifecycle: { status: string | null; status_is_override: boolean } | null,
): void {
  if (lifecycle === null) return
  openWorkStatus.value = lifecycle.status
  openWorkIsOverride.value = lifecycle.status_is_override
  openWorkLifecycleHasLoaded.value = true
  // Chỉ mục Library đã đổi (Rust gọi lại `reindex_library` sau lượt ghi này) -- tải lại danh
  // sách để hàng của Tác phẩm đang mở phản ánh giá trị mới, không đợi một lượt Quét lại
  // riêng.
  void loadWorks()
}

/** Ghi đè trạng thái Tác phẩm đang mở thành *Tạm ngưng* — lệnh
 * `lifecycle.set_work_override_paused`. Bề mặt tối thiểu của story chỉ cấp ĐÚNG một giá trị
 * ghi đè qua nút bấm (§Never: "không dựng lưới Tác phẩm của Story 5.6"). */
export async function setOpenWorkOverride(): Promise<void> {
  if (openWorkBusy.value) return

  openWorkBusy.value = true
  openWorkError.value = null
  const mySequence = ++lifecycleSequence

  const result = await setWorkStatusOverride('paused')
  if (mySequence !== lifecycleSequence) return

  openWorkBusy.value = false
  if (result.error !== null) {
    openWorkError.value = result.error
    return
  }
  applyOpenWorkLifecycleResult(result.lifecycle)
}

/** Bỏ ghi đè, quay về giá trị SUY RA hiện thời — lệnh `lifecycle.clear_work_override`. */
export async function clearOpenWorkOverride(): Promise<void> {
  if (openWorkBusy.value) return

  openWorkBusy.value = true
  openWorkError.value = null
  const mySequence = ++lifecycleSequence

  const result = await setWorkStatusOverride(null)
  if (mySequence !== lifecycleSequence) return

  openWorkBusy.value = false
  if (result.error !== null) {
    openWorkError.value = result.error
    return
  }
  applyOpenWorkLifecycleResult(result.lifecycle)
}

/** Đặt Chương ĐANG MỞ (Workspace) thành *Đã xong* — lệnh `lifecycle.set_chapter_done`.
 * Không Chương nào đang mở ⇒ no-op, không gọi IPC. */
export async function setOpenChapterStatus(): Promise<void> {
  if (openWorkBusy.value) return
  const chapterId = editorChapterId.value
  if (chapterId === null) return

  openWorkBusy.value = true
  openWorkError.value = null
  const mySequence = ++lifecycleSequence

  const result = await setChapterStatus(chapterId, 'done')
  if (mySequence !== lifecycleSequence) return

  openWorkBusy.value = false
  if (result.error !== null) {
    openWorkError.value = result.error
    return
  }
  applyOpenWorkLifecycleResult(result.lifecycle)
}

/**
 * 🔴 Vứt toàn bộ state — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`. Dùng bởi bàn đo/test; sản phẩm không có chỗ gọi (khối này sống suốt phiên).
 */
export function resetLibraryWorks(): void {
  worksSequence += 1
  lifecycleSequence += 1
  works.value = []
  worksTotal.value = 0
  worksMatched.value = 0
  statusFilter.value = new Set()
  genreFilter.value = null
  sourceLangFilter.value = null
  sortKey.value = 'updated_desc'
  genres.value = []
  sourceLangs.value = []
  workCursor.value = 0
  worksHaveLoaded.value = false
  worksBusy.value = false
  // Ô nhớ cấp module như mọi ô khác ở trên -- một lượt reset bỏ sót nó sẽ để một lượt tải lại
  // của phiên TRƯỚC bắn vào phiên SAU. `check:panel-refs` canh đúng mệnh đề này.
  worksReloadPending = false
  worksError.value = null
  openWorkStatus.value = null
  openWorkIsOverride.value = false
  openWorkLifecycleHasLoaded.value = false
  openWorkBusy.value = false
  openWorkError.value = null
}
