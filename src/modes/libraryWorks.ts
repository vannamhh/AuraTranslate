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
import type { WorkRow } from '../config/library'
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
export const libraryWorksHaveLoaded: DeepReadonly<Ref<boolean>> = readonly(worksHaveLoaded)
export const libraryWorksBusy: DeepReadonly<Ref<boolean>> = readonly(worksBusy)
export const libraryWorksError: DeepReadonly<Ref<IpcError | null>> = readonly(worksError)

export const openWorkLifecycleStatus: DeepReadonly<Ref<string | null>> = readonly(openWorkStatus)
export const openWorkLifecycleIsOverride: DeepReadonly<Ref<boolean>> = readonly(openWorkIsOverride)
export const openWorkLifecycleLoaded: DeepReadonly<Ref<boolean>> = readonly(openWorkLifecycleHasLoaded)
export const openWorkLifecycleBusy: DeepReadonly<Ref<boolean>> = readonly(openWorkBusy)
export const openWorkLifecycleError: DeepReadonly<Ref<IpcError | null>> = readonly(openWorkError)

/** `true` ⇔ bộ lọc RỖNG (không lọc gì) — cùng ngữ nghĩa `filter = None` phía Rust. */
export const libraryFilterIsEmpty = computed<boolean>(() => statusFilter.value.size === 0)

/**
 * Đọc lại danh sách Tác phẩm theo bộ lọc HIỆN THỜI — lệnh `library.list_works`.
 *
 * Bộ lọc RỖNG ⇒ gửi `undefined` (không lọc), đúng khuôn `filter = None` phía Rust — một
 * `Set` rỗng và "không lọc" là CÙNG một ý định, không hai trạng thái khác nhau.
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
  const result = await listLibraryWorks(filter)
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
  worksHaveLoaded.value = true
  await runPendingWorksReload()
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

/** Bỏ mọi bộ lọc đang bật rồi tải lại. No-op nếu đã không lọc gì. */
export function clearStatusFilter(): void {
  if (statusFilter.value.size === 0) return
  statusFilter.value = new Set()
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
