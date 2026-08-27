/**
 * State + thao tác của khối "Quét lại thư mục" ở Library — Story 5.3, FR99.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ VÌ SAO MỘT MODULE THUẦN RIÊNG, KHÔNG VIẾT THẲNG TRONG `LibraryMode.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `libraryImport.ts`: AD-34 §1 đòi mọi `@click` là đúng một `dispatch('<id>')`,
 * và năm thao tác (`library.rescan` · `…choose_root` · `…forget_orphan` · `…orphan_next` ·
 * `…orphan_prev`) đăng ký ở `src/commands/index.ts` như các `CommandDeps` TIÊM VÀO —
 * `src/main.ts` nối chúng vào `installCommands({...})`. Module này là phía CUNG CẤP.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 `libraryScanHasLoaded` — LÝ DO NÓ TỒN TẠI (`AGENTS.md::Known pitfalls`)
 * ─────────────────────────────────────────────────────────────────────────────
 * "Không có mục mồ côi nào" chỉ được phép nói SAU khi đã quét ít nhất một lần trong phiên
 * này. Trước lượt quét đầu, danh sách mồ côi cũng rỗng — nhưng đó là "chưa biết", không phải
 * "không có". `LibraryMode.vue` phải hỏi vị từ này TRƯỚC khi kết luận.
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { chooseLibraryRoot, forgetLibraryOrphan, rescanLibrary } from '../config/library'
import type { OrphanEntry } from '../config/library'
import type { IpcError } from '../i18n'

// ─────────────────────────────────────────────────────────────────────────────
// State module-level — singleton của cả tiến trình, cùng khuôn glossaryManageState.ts. Mỗi
// khai báo NẰM TRÊN MỘT DÒNG (`check:panel-refs` Kiểm 5 — cú pháp ngoài tập con cho ĐỎ).
// ─────────────────────────────────────────────────────────────────────────────
const libraryRoot = ref<string | null>(null)
// 🔵 THÊM (2026-08-27, vòng rà bốn lớp P1) — phân biệt "gốc không còn ở đó" với "gốc rỗng
// thật". Xem doc-comment của `RescanReport::root_missing` (Rust) và `applyReport` ngay dưới.
const rootMissing = ref(false)
const orphans = ref<OrphanEntry[]>([])
const orphanCursor = ref(0)
const indexedCount = ref(0)
const conflictCount = ref(0)
const skippedCount = ref(0)
const rescanBusy = ref(false)
const libraryScanHasLoaded = ref(false)
const lastError = ref<IpcError | null>(null)

/** Số thứ tự lượt gọi — chặn một lượt CŨ ghi đè lên state của một lượt MỚI hơn (round-trip
 * IPC đua nhau), cùng khuôn `sequence` của các state Glossary khác. */
let sequence = 0

export const currentLibraryRoot: DeepReadonly<Ref<string | null>> = readonly(libraryRoot)
export const libraryRootMissing: DeepReadonly<Ref<boolean>> = readonly(rootMissing)
export const libraryOrphans: DeepReadonly<Ref<OrphanEntry[]>> = readonly(orphans)
export const libraryOrphanCursor: DeepReadonly<Ref<number>> = readonly(orphanCursor)
export const libraryIndexedCount: DeepReadonly<Ref<number>> = readonly(indexedCount)
export const libraryConflictCount: DeepReadonly<Ref<number>> = readonly(conflictCount)
export const librarySkippedCount: DeepReadonly<Ref<number>> = readonly(skippedCount)
export const libraryRescanBusy: DeepReadonly<Ref<boolean>> = readonly(rescanBusy)
export const libraryScanHasLoadedState: DeepReadonly<Ref<boolean>> = readonly(libraryScanHasLoaded)
export const libraryRescanError: DeepReadonly<Ref<IpcError | null>> = readonly(lastError)

/**
 * Mục mồ côi ĐANG CHỌN, hoặc `null` nếu con trỏ ngoài phạm vi (danh sách rỗng, hoặc chưa
 * quét lần nào).
 *
 * 🔴 `.at(cursor.value)`, KHÔNG `[cursor.value]` — cùng lý do `manageCurrentRow`/
 * `queueCurrentRow` (`noUncheckedIndexedAccess` không bật; `.at()` khai đúng `T | undefined`).
 */
export const currentLibraryOrphan = computed<OrphanEntry | null>(() => orphans.value.at(orphanCursor.value) ?? null)

function clampCursor(): void {
  const maxIndex = orphans.value.length - 1
  if (orphanCursor.value > maxIndex) orphanCursor.value = Math.max(0, maxIndex)
  if (orphanCursor.value < 0) orphanCursor.value = 0
}

function applyReport(report: {
  root: string
  root_missing: boolean
  indexed: number
  conflicts: number
  skipped: number
  orphans: OrphanEntry[]
}): void {
  libraryRoot.value = report.root
  rootMissing.value = report.root_missing
  indexedCount.value = report.indexed
  conflictCount.value = report.conflicts
  skippedCount.value = report.skipped
  orphans.value = report.orphans
  libraryScanHasLoaded.value = true
  clampCursor()
}

/** Quét lại thư mục gốc ĐANG cấu hình — lệnh `library.rescan` (AC1, có phím mặc định). */
export async function rescanLibraryFolder(): Promise<void> {
  if (rescanBusy.value) return

  rescanBusy.value = true
  lastError.value = null
  const mySequence = ++sequence

  const result = await rescanLibrary()
  if (mySequence !== sequence) return // Một lượt MỚI hơn đã bắt đầu -- bỏ, không ghi đè.

  rescanBusy.value = false
  if (result.error !== null) {
    lastError.value = result.error
    return
  }
  if (result.report === null) return // Không có cầu IPC -- im lặng, cùng nhánh mọi adapter khác.

  applyReport(result.report)
}

/**
 * Mở hộp thoại chọn thư mục, đổi thư mục gốc rồi quét lại ngay trên đó — lệnh
 * `library.choose_root` (AD-48).
 *
 * 🔴 **Huỷ hộp thoại là IM LẶNG, không một câu nào** (§I/O Matrix "Huỷ hộp thoại") —
 * `result.report === null, result.error === null` giữ NGUYÊN mọi state hiện có.
 */
export async function chooseLibraryRootFolder(): Promise<void> {
  if (rescanBusy.value) return

  rescanBusy.value = true
  lastError.value = null
  const mySequence = ++sequence

  const result = await chooseLibraryRoot()
  if (mySequence !== sequence) return

  rescanBusy.value = false
  if (result.error !== null) {
    lastError.value = result.error
    return
  }
  if (result.report === null) return // Huỷ hộp thoại HOẶC không cầu IPC -- không đổi gì.

  applyReport(result.report)
}

/** Gỡ mục mồ côi ĐANG CHỌN khỏi chỉ mục — lệnh `library.forget_orphan`. */
export async function forgetCurrentLibraryOrphan(): Promise<void> {
  if (rescanBusy.value) return
  const target = currentLibraryOrphan.value
  if (target === null) return

  rescanBusy.value = true
  lastError.value = null
  const mySequence = ++sequence

  // P9 (vòng rà THỨ HAI, 2026-08-27) -- gửi kèm `name` đang hiển thị, để Rust dựng được
  // một câu từ chối nói TÊN thay vì chỉ UUID trần.
  const result = await forgetLibraryOrphan(target.work_id, target.name)
  if (mySequence !== sequence) return

  rescanBusy.value = false
  if (result.error !== null) {
    lastError.value = result.error
    return
  }
  if (result.orphans === null) return // Không có cầu IPC.

  orphans.value = result.orphans
  clampCursor()
}

/** Chuyển con trỏ xuống mục mồ côi kế tiếp — không vòng. */
export function nextLibraryOrphan(): void {
  if (orphanCursor.value < orphans.value.length - 1) orphanCursor.value += 1
}

/** Chuyển con trỏ lên mục mồ côi trước — không vòng. */
export function prevLibraryOrphan(): void {
  if (orphanCursor.value > 0) orphanCursor.value -= 1
}

/**
 * 🔴 Vứt toàn bộ state — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`. Dùng bởi bàn đo/test; sản phẩm không có chỗ gọi (khối này sống suốt phiên,
 * cùng khuôn `libraryImport.ts` không có `reset*` gọi từ sản phẩm — Library không bị tháo).
 */
export function resetLibraryRescan(): void {
  sequence += 1
  libraryRoot.value = null
  rootMissing.value = false
  orphans.value = []
  orphanCursor.value = 0
  indexedCount.value = 0
  conflictCount.value = 0
  skippedCount.value = 0
  rescanBusy.value = false
  libraryScanHasLoaded.value = false
  lastError.value = null
}
