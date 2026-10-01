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
 * 🔴 `libraryScanHasLoaded` — LÝ DO NÓ TỒN TẠI (`AGENTS.md::Known pitfalls`), VÀ VÌ SAO
 * MỘT CỜ THỨ HAI (`rescanResultHasLoaded`) ĐỨNG RIÊNG
 * ─────────────────────────────────────────────────────────────────────────────
 * "Không có mục mồ côi nào" chỉ được phép nói SAU khi đã đọc trạng thái THẬT ít nhất một lần
 * trong phiên này. Trước lượt đọc đầu, danh sách mồ côi cũng rỗng — nhưng đó là "chưa biết",
 * không phải "không có". `LibraryMode.vue` phải hỏi vị từ này TRƯỚC khi kết luận.
 *
 * Cờ này không còn CHỈ bật bởi một lượt Quét lại thư mục (`rescanLibraryFolder`/
 * `chooseLibraryRootFolder`): [`loadLibraryOrphans`] (một `SELECT` thuần, không
 * `Indexer::rebuild`) cũng bật nó, vì nó cũng là một lượt đọc trạng thái mồ côi THẬT từ
 * `library-index.db`.
 *
 * ⚠️ Chính vì thế nó KHÔNG được dùng để canh ba-con-số Quét lại (`indexedCount`/
 * `conflictCount`/`skippedCount`/`textSkippedCount`) hay `rootMissing`: những trường đó chỉ
 * [`applyReport`] (một lượt Quét lại/Đổi thư mục gốc THẬT) ghi; [`loadLibraryOrphans`] không
 * chạm tới chúng. Gộp chung một cờ nghĩa là bật `libraryScanHasLoaded` qua đường mồ côi sẽ
 * làm dòng "Đã lập chỉ mục 0 · …" hiện ra như một kết quả THẬT dù chưa lượt Quét lại nào từng
 * chạy — đúng lớp lỗi "0 hàng, không lỗi, im lặng thành một con số" mà `…HasLoaded` sinh ra để
 * chặn, chỉ đổi chỗ. `rescanResultHasLoaded` vì thế đứng riêng, [`applyReport`] MỚI bật nó.
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { chooseLibraryRoot, forgetLibraryOrphan, listOrphans, rescanLibrary } from '../config/library'
import type { ConflictEntry, OrphanEntry, TextSkippedEntry } from '../config/library'
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
// 🔵 THÊM (2026-08-27, phán quyết Ice #3) — dữ liệu CÓ CẤU TRÚC của xung đột, không chỉ đếm.
// `conflictCount` (ngay dưới) vẫn giữ NGUYÊN vai cũ (dòng ba-con-số); `conflicts` phục vụ
// node cảnh báo RIÊNG mà AC4 đòi ("phát hiện VÀ cảnh báo" — hai vế).
const conflicts = ref<ConflictEntry[]>([])
const conflictCount = ref(0)
const skippedCount = ref(0)
// **THÊM (retro Epic 5, AI-2/AI-3 — 2026-09-03)** — thay vì bị vứt, số Tác phẩm bị bỏ qua PHẦN
// VĂN BẢN ở lượt quét gần nhất. Đếm từ `.length`, cùng khuôn `conflictCount`/`skippedCount`
// ngay trên — một nguồn sự thật DUY NHẤT.
const textSkippedCount = ref(0)
const rescanBusy = ref(false)
const libraryScanHasLoaded = ref(false)
const rescanResultHasLoaded = ref(false)
const lastError = ref<IpcError | null>(null)

/** Bumped when a write starts and by `resetLibraryRescan()`. `rescanBusy` keeps the writes
 * mutually exclusive, so only a reset can invalidate an in-flight write. */
let sequence = 0

/** Bumped where a write applies its result and in `resetLibraryRescan()`: an orphan read that
 * sees it change between start and return drops its result. */
let writeEpoch = 0

export const currentLibraryRoot: DeepReadonly<Ref<string | null>> = readonly(libraryRoot)
export const libraryRootMissing: DeepReadonly<Ref<boolean>> = readonly(rootMissing)
export const libraryOrphans: DeepReadonly<Ref<OrphanEntry[]>> = readonly(orphans)
export const libraryOrphanCursor: DeepReadonly<Ref<number>> = readonly(orphanCursor)
export const libraryIndexedCount: DeepReadonly<Ref<number>> = readonly(indexedCount)
export const libraryConflicts: DeepReadonly<Ref<ConflictEntry[]>> = readonly(conflicts)
export const libraryConflictCount: DeepReadonly<Ref<number>> = readonly(conflictCount)
export const librarySkippedCount: DeepReadonly<Ref<number>> = readonly(skippedCount)
export const libraryTextSkippedCount: DeepReadonly<Ref<number>> = readonly(textSkippedCount)
export const libraryRescanBusy: DeepReadonly<Ref<boolean>> = readonly(rescanBusy)
export const libraryScanHasLoadedState: DeepReadonly<Ref<boolean>> = readonly(libraryScanHasLoaded)
/** Bật DUY NHẤT bởi [`applyReport`] (một lượt Quét lại/Đổi thư mục gốc THẬT) — xem khối lý
 * do đầu tệp cho vì sao đây KHÔNG phải cùng cờ với `libraryScanHasLoadedState`. */
export const libraryRescanResultHasLoadedState: DeepReadonly<Ref<boolean>> = readonly(rescanResultHasLoaded)
export const libraryRescanError: DeepReadonly<Ref<IpcError | null>> = readonly(lastError)

/**
 * Mục mồ côi ĐANG CHỌN, hoặc `null` nếu con trỏ ngoài phạm vi (danh sách rỗng, hoặc chưa
 * quét lần nào).
 *
 * 🔴 `.at(cursor.value)`, KHÔNG `[cursor.value]` — cùng lý do `manageCurrentRow`/
 * `queueCurrentRow` (`noUncheckedIndexedAccess` không bật; `.at()` khai đúng `T | undefined`).
 */
export const currentLibraryOrphan = computed<OrphanEntry | null>(() => orphans.value.at(orphanCursor.value) ?? null)

/**
 * **THÊM (2026-08-27, phán quyết Ice #3)** — chỗ trùng `work_id` ĐẦU TIÊN, hoặc `null` nếu
 * lượt quét gần nhất không phát hiện chỗ nào. Cùng khuôn `currentLibraryOrphan` ngay trên
 * (`.at(0)`, không `[0]` — `noUncheckedIndexedAccess` không bật, `.at()` khai đúng
 * `T | undefined`): `LibraryMode.vue` chỉ cần nêu đích danh chỗ trùng đầu tiên kèm cả hai
 * đường dẫn, và "và N chỗ nữa" khi nhiều hơn một — không dựng danh sách/lưới (Story 5.6 sở
 * hữu phần đó).
 */
export const firstLibraryConflict = computed<ConflictEntry | null>(() => conflicts.value.at(0) ?? null)

function clampCursor(): void {
  const maxIndex = orphans.value.length - 1
  if (orphanCursor.value > maxIndex) orphanCursor.value = Math.max(0, maxIndex)
  if (orphanCursor.value < 0) orphanCursor.value = 0
}

function applyReport(report: {
  root: string
  root_missing: boolean
  indexed: number
  conflicts: ConflictEntry[]
  skipped: number
  orphans: OrphanEntry[]
  text_skipped: TextSkippedEntry[]
}): void {
  libraryRoot.value = report.root
  rootMissing.value = report.root_missing
  indexedCount.value = report.indexed
  // Phán quyết Ice #3 -- giữ NGUYÊN dữ liệu xung đột (node cảnh báo), và suy con số cũ từ
  // `.length` (dòng ba-con-số) thay vì nhận nó rời rạc từ Rust -- một nguồn sự thật DUY NHẤT,
  // không hai trường có thể trôi khỏi nhau.
  conflicts.value = report.conflicts
  conflictCount.value = report.conflicts.length
  skippedCount.value = report.skipped
  orphans.value = report.orphans
  // **THÊM (retro Epic 5, AI-2/AI-3)** -- cùng lý lẽ `conflictCount`: suy từ `.length`, không
  // nhận một con số rời từ Rust.
  textSkippedCount.value = report.text_skipped.length
  libraryScanHasLoaded.value = true
  rescanResultHasLoaded.value = true
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

  writeEpoch += 1
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

  writeEpoch += 1
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

  writeEpoch += 1
  orphans.value = result.orphans
  clampCursor()
}

/**
 * Đọc danh sách mồ côi HIỆN CÓ trong chỉ mục — lệnh
 * `library_list_orphans`, KHÔNG quét lại thư mục gốc (`Indexer::rebuild` không chạy — chữ ký
 * của lệnh phía Rust không nhận `root: &Path` nên về cấu trúc không gọi được hàm đó).
 * `LibraryMode.vue::onActivated` gọi hàm này (không phải `rescanLibraryFolder`) để khối
 * "Mồ côi" hiện đúng trạng thái NGAY khi vào Library, không cần người dùng tự bấm Quét lại.
 *
 * Does not touch the writers' `sequence`, so an in-flight write still applies its report and
 * frees the button. Its own result is dropped when a write applied meanwhile (`writeEpoch`) or
 * `resetLibraryRescan()` ran. It does not check `rescanBusy`: it is a light read.
 */
export async function loadLibraryOrphans(): Promise<void> {
  const myEpoch = writeEpoch

  const result = await listOrphans()
  if (myEpoch !== writeEpoch) return

  if (result.error !== null) {
    lastError.value = result.error
    return
  }
  if (result.orphans === null) return // Không có cầu IPC -- im lặng, cùng nhánh mọi adapter khác.

  orphans.value = result.orphans
  libraryScanHasLoaded.value = true
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
  writeEpoch += 1
  libraryRoot.value = null
  rootMissing.value = false
  orphans.value = []
  orphanCursor.value = 0
  indexedCount.value = 0
  conflicts.value = []
  conflictCount.value = 0
  skippedCount.value = 0
  textSkippedCount.value = 0
  rescanBusy.value = false
  libraryScanHasLoaded.value = false
  rescanResultHasLoaded.value = false
  lastError.value = null
}
