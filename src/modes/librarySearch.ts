/**
 * State + thao tác của khối "Tìm kiếm" ở Library — Story 5.9, FR8: tìm một câu từng dịch ở
 * bất kỳ đâu trong cả thư viện, đồng thời trên nguyên văn và bản dịch. Story 5.10 (FR9) thêm
 * hai chế độ dấu: chính xác (mặc định) và khoan dung, cộng lượt TỰ NỚI khi chính xác trả 0
 * hàng trên một chỉ mục KHÔNG rỗng.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ VÌ SAO MỘT MODULE THUẦN RIÊNG, KHÔNG VIẾT THẲNG TRONG `LibraryMode.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `libraryWorks.ts`/`libraryChapters.ts`: AD-34 §1 đòi mọi `@click` là đúng một
 * `dispatch('<id>')`, và các thao tác mới (`library.search` · `library.open_search_hit` ·
 * `library.search_mode_exact` · `library.search_mode_lenient`) đăng ký ở
 * `src/commands/index.ts` như các `CommandDeps` TIÊM VÀO — `src/main.ts` nối chúng vào
 * `installCommands`. Module này là phía CUNG CẤP.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 TÁM TRẠNG THÁI PHÂN BIỆT ĐƯỢC — §Always của story, không gộp
 * ─────────────────────────────────────────────────────────────────────────────
 * ① chưa gõ gì · ② đang tìm · ③ chỉ mục chưa có dòng nào · ④ truy vấn dưới 3 ký tự (VÀ `hits`
 * rỗng — một truy vấn ngắn VẪN có thể khớp ở nửa bản dịch, xem [`runLibrarySearch`]) · ⑤ không
 * khớp · ⑥ không khớp SAU KHI ĐÃ NỚI · ⑦ có kết quả · ⑧ có kết quả SAU KHI ĐÃ NỚI.
 * `LibraryMode.vue` đọc đúng thứ tự ưu tiên đó, không suy luận lại.
 *
 * 🔵 **Vì sao "tám", không "bảy" như §Tasks của story đọc lướt qua.** §Tasks 6 nói "bảy giá
 * trị" — con số đó phái sinh từ một chú thích CŨ (`Story 5.9`) tự xưng "năm giá trị" trong khi
 * `LibrarySearchStatus` thật đã có SÁU (`not_typed`/`searching`/`index_empty`/`short_query`/
 * `no_match`/`result`, đếm trực tiếp trên khai báo). Story này CỘNG THÊM đúng hai giá trị
 * (`no_match_widened`/`result_widened`, tách khỏi `no_match`/`result` khi một lượt tự nới xảy
 * ra) ⇒ 6 + 2 = **8**, không phải 7. Bảy ca của AC3 là bảy KỊCH BẢN được nêu tên để DEMO —
 * chúng không loại `result` (ca "có kết quả, KHÔNG nới" đã có từ Story 5.9 và không đổi ở đây)
 * khỏi danh mục. Sửa tại chỗ theo luật "một tiền đề sai không làm kết luận sai" — kết luận
 * đúng ("tám giá trị phân biệt được") vẫn đứng, chỉ số đến từ §Tasks bị lệch.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 `openCurrentLibrarySearchHit` — KHÔNG BAO GIỜ mở Chương TRƯỚC KHI mở Tác phẩm đã XONG
 * ─────────────────────────────────────────────────────────────────────────────
 * `chapter_id` là số `AUTOINCREMENT` CỤC BỘ theo từng `project.db` — một `chapter_id` của Tác
 * phẩm B áp lên Tác phẩm A mở đúng một Chương SAI mà không lỗi nào ném
 * (`libraryChapters.ts:234-252` giải thích trọn cửa sổ đó). Hàm dưới đây `await`
 * `openWorkById` XONG, kiểm nó THẬT SỰ thành công (không lỗi, không bị CHẶN vì flush) TRƯỚC
 * khi phát `openChapterById` — không suy luận từ việc `openWorkById` đã `await` xong là nó đã
 * THÀNH CÔNG (hai điều khác nhau: một lượt bị CHẶN vẫn `await` xong, chỉ là không đổi Tác
 * phẩm).
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { searchLibrary } from '../config/library'
import type { SearchHit, SearchMode } from '../config/library'
import { openChapterById } from '../panels/editorPanelState'
import { libraryOpenWorkError, libraryOpenWorkNotice, openWorkById } from './libraryChapters'
import { setMode } from './modeState'
import type { IpcError } from '../i18n'

// ─────────────────────────────────────────────────────────────────────────────
// State module-level — singleton của cả tiến trình, cùng khuôn `libraryWorks.ts`. Mỗi khai
// báo NẰM TRÊN MỘT DÒNG (`check:panel-refs` Kiểm 5 — cú pháp ngoài tập con cho ĐỎ).
// ─────────────────────────────────────────────────────────────────────────────
const query = ref('')
const hits = ref<SearchHit[]>([])
const total = ref(0)
const indexedSegments = ref(0)
// **THÊM (retro Epic 5, AI-3 — 2026-09-03).** Độ phủ cấp Tác phẩm của lượt tìm gần nhất — chép
// nguyên `SearchReport.works_total`/`works_with_text`. Bề mặt ĐỘC LẬP với `hits`/`total`: xem
// [`librarySearchCoverageGap`].
const worksTotal = ref(0)
const worksWithText = ref(0)
const shortQuery = ref(false)
/** Xem [`librarySearchTruncated`]. */
const truncated = ref(false)
const hasLoaded = ref(false)
const busy = ref(false)
const error = ref<IpcError | null>(null)
// Con trỏ ô đang chọn trong danh sách kết quả — chép khuôn `workCursor` của `libraryWorks.ts`.
const cursor = ref(0)
// **THÊM Story 5.10.** Chế độ NGƯỜI DÙNG đang chọn (mặc định `'exact'`, AD-27 · AC4) — cờ này
// dẫn tham số `mode` gửi xuống `searchLibrary`, KHÔNG suy ra từ `effectiveMode` của lượt tìm
// gần nhất.
const mode = ref<SearchMode>('exact')
// Chế độ THỰC SỰ đã chạy ở lượt tìm gần nhất — chép nguyên `SearchReport.effective_mode`.
const effectiveMode = ref<SearchMode>('exact')
// `true` ⇔ lượt tìm gần nhất là một lượt TỰ NỚI (`mode === 'exact'` nhưng chính xác trả 0
// hàng trên một chỉ mục KHÔNG rỗng) — chép nguyên `SearchReport.widened`.
const widened = ref(false)
// Một lượt `runLibrarySearch()` bị chặn vì đang bận, ghi nhớ để chạy lại — cùng khuôn
// `worksReloadPending`. Không phải `ref`: không khối template nào đọc nó.
let reloadPending = false
let sequence = 0

/** Ghi được cho `v-model` của ô tìm — cùng khuôn `libraryImport.ts::pastedText` (export
 * TRẦN, không bọc `readonly()`). */
export const librarySearchQuery = query
export const librarySearchHits: DeepReadonly<Ref<SearchHit[]>> = readonly(hits)
export const librarySearchTotal: DeepReadonly<Ref<number>> = readonly(total)
export const librarySearchIndexedSegments: DeepReadonly<Ref<number>> = readonly(indexedSegments)
/** Xem [`librarySearchCoverageGap`]. */
export const librarySearchWorksTotal: DeepReadonly<Ref<number>> = readonly(worksTotal)
/** Xem [`librarySearchCoverageGap`]. */
export const librarySearchWorksWithText: DeepReadonly<Ref<number>> = readonly(worksWithText)
export const librarySearchShortQuery: DeepReadonly<Ref<boolean>> = readonly(shortQuery)
/** 🔴 `true` ⇔ danh sách đã bị trần CẮT — `librarySearchTotal` khi đó là *"số hàng đang
 * hiện"*, không phải *"số hàng khớp"*. Giao diện phải nói ra: một trần cắt trong im lặng biến
 * một con số thành một lời khai sai. */
export const librarySearchTruncated: DeepReadonly<Ref<boolean>> = readonly(truncated)
export const librarySearchHasLoaded: DeepReadonly<Ref<boolean>> = readonly(hasLoaded)
export const librarySearchBusy: DeepReadonly<Ref<boolean>> = readonly(busy)
export const librarySearchError: DeepReadonly<Ref<IpcError | null>> = readonly(error)
export const librarySearchCursor: DeepReadonly<Ref<number>> = readonly(cursor)
/** Chế độ NGƯỜI DÙNG đang chọn — hai nút chế độ đọc cờ này để tô `aria-pressed`. */
export const librarySearchMode: DeepReadonly<Ref<SearchMode>> = readonly(mode)
/** Chế độ THỰC SỰ đã chạy ở lượt tìm gần nhất (AC4: "đọc được chế độ đang có hiệu lực trên
 * màn hình mà không phải suy ra từ số kết quả"). */
export const librarySearchEffectiveMode: DeepReadonly<Ref<SearchMode>> = readonly(effectiveMode)
/** `true` ⇔ lượt tìm gần nhất đã TỰ NỚI. */
export const librarySearchWidened: DeepReadonly<Ref<boolean>> = readonly(widened)

/**
 * Kết quả ĐANG CHỌN trong danh sách, hoặc `null` nếu con trỏ ngoài phạm vi — cùng lý do
 * `currentLibraryWork` (`libraryWorks.ts`): `.at()`, không `[]` (`noUncheckedIndexedAccess`
 * không bật).
 */
export const currentLibrarySearchHit = computed<SearchHit | null>(() => hits.value.at(cursor.value) ?? null)

function clampCursor(): void {
  const maxIndex = hits.value.length - 1
  if (cursor.value > maxIndex) cursor.value = Math.max(0, maxIndex)
  if (cursor.value < 0) cursor.value = 0
}

/** Danh mục ĐÓNG, TÁM giá trị — khớp ĐÚNG tám trạng thái phân biệt được của §I/O Matrix
 * (Story 5.9 + Story 5.10), theo thứ tự ưu tiên đã ghi ở khối doc-comment đầu tệp. Xem khối
 * 🔵 đầu tệp cho lý do "tám", không "bảy". */
export type LibrarySearchStatus =
  | 'not_typed'
  | 'searching'
  | 'index_empty'
  | 'short_query'
  | 'no_match'
  | 'no_match_widened'
  | 'result'
  | 'result_widened'

/**
 * **Hàm THUẦN** — suy trạng thái hiển thị từ năm dữ kiện thô, tách khỏi mọi `ref` để vitest
 * kiểm được TẤT ĐỊNH mà không cần mount `LibraryMode.vue` (cùng lý do `chapterWindow` của
 * `libraryChapters.ts` là một hàm thuần: `tests/AGENTS.md`, "hành vi của module thuần").
 *
 * ⚠️ Thứ tự các nhánh LÀ hợp đồng:
 * - `short_query` chỉ được đọc SAU khi `hits.length === 0`, vì một truy vấn ngắn VẪN có thể
 *   khớp thật ở nửa bản dịch (unicode61 không có sàn 3 ký tự).
 * - **THÊM Story 5.10** — `widenedValue` được đọc TRƯỚC `shortQueryValue` khi `hits.length ===
 *   0`: một lượt ĐÃ tự nới (đã thử cả hai chế độ) là thông tin ĐẦY ĐỦ HƠN "truy vấn dưới 3 ký
 *   tự" — nói "đã thử cả hai chế độ, vẫn không ra" đúng sự thật hơn "quá ngắn", nhất là khi
 *   nhánh nguyên văn (nơi sàn 3 ký tự áp dụng) không liên quan gì tới thứ vừa được nới (nửa
 *   bản dịch).
 */
export function librarySearchStatus(
  hasLoaded: boolean,
  busy: boolean,
  indexedSegmentsValue: number,
  hitCount: number,
  shortQueryValue: boolean,
  widenedValue: boolean,
): LibrarySearchStatus {
  // 🔴 **`busy` thắng TRƯỚC, không chỉ khi chưa nạp lần nào.** Bản đầu viết
  // `if (!hasLoaded) return busy ? 'searching' : 'not_typed'` rồi mới xét các nhánh dưới — nên
  // một lượt tìm THỨ HAI (lúc này `hasLoaded` đã `true`) trả về `'result'` cùng danh sách hit
  // của lượt TRƯỚC trong suốt thời gian nó bay. Màn hình khẳng định dứt khoát một kết quả cho
  // một truy vấn KHÁC với truy vấn đang nằm trong ô tìm — cùng lớp lỗi *"một câu trả lời đúng
  // về hình dạng nhưng sai về sự thật"*, và chính doc-comment của module này khai ca ② là
  // `searchBusy` không kèm điều kiện nào.
  if (busy) return 'searching'
  if (!hasLoaded) return 'not_typed'
  if (indexedSegmentsValue === 0) return 'index_empty'
  if (hitCount === 0 && widenedValue) return 'no_match_widened'
  if (hitCount === 0 && shortQueryValue) return 'short_query'
  if (hitCount === 0) return 'no_match'
  if (widenedValue) return 'result_widened'
  return 'result'
}

/** `computed` đọc live -- `LibraryMode.vue` dùng thẳng cái này, không tự lặp lại logic. */
export const librarySearchStatusKey = computed<LibrarySearchStatus>(() =>
  librarySearchStatus(
    hasLoaded.value,
    busy.value,
    indexedSegments.value,
    hits.value.length,
    shortQuery.value,
    widened.value,
  ),
)

/**
 * **THÊM (retro Epic 5, AI-3 — 2026-09-03).** Hàm THUẦN — độ phủ cấp Tác phẩm của lượt tìm gần
 * nhất. Trả `{ missing, total }` khi chỉ mục THỦNG (`worksWithText < worksTotal`), `null` khi
 * không có gì để nói (`worksTotal === 0` — chỉ mục rỗng hẳn, đã có bề mặt `index_empty` riêng;
 * hoặc chỉ mục ĐẦY ĐỦ, `worksWithText === worksTotal`).
 *
 * ⚠️ Bề mặt này ĐỘC LẬP với `librarySearchStatusKey`: nó phải hiện được CẢ KHI có kết quả — một
 * lượt trả "3 kết quả" trong lúc 35/47 Tác phẩm vô hình khỏi chỉ mục là đúng thứ lỗi mà spec
 * retro AI-2/AI-3 tồn tại để sửa. `LibraryMode.vue` dùng hàm này qua một `v-if` RIÊNG, không
 * gộp vào bảng ternary tám nhánh của `role="status"` hiện có.
 */
export function librarySearchCoverageGap(
  worksTotalValue: number,
  worksWithTextValue: number,
): { missing: number; total: number } | null {
  if (worksTotalValue === 0) return null
  if (worksWithTextValue >= worksTotalValue) return null
  return { missing: worksTotalValue - worksWithTextValue, total: worksTotalValue }
}

/** `computed` đọc live -- cùng khuôn `librarySearchStatusKey`. */
export const librarySearchCoverageGapState = computed<{ missing: number; total: number } | null>(() =>
  librarySearchCoverageGap(worksTotal.value, worksWithText.value),
)

/**
 * Chạy một lượt tìm kiếm theo [`librarySearchQuery`] hiện thời — lệnh `library.search`.
 *
 * 🔴 **Truy vấn RỖNG (sau `trim()`) ⇒ 0 lượt IPC** (§I/O Matrix "Truy vấn rỗng / chỉ khoảng
 * trắng") — mọi state trở về ca ① "chưa gõ gì", KHÔNG gọi `searchLibrary`.
 *
 * `short_query = true` KHÔNG tự nó là một ca rỗng: nửa bản dịch (`unicode61`) vẫn chạy dưới 3
 * ký tự và có thể khớp thật (§Design Notes bảng đo). `LibraryMode.vue` chỉ đọc ca ④ khi
 * `short_query && hits.length === 0`.
 */
export async function runLibrarySearch(): Promise<void> {
  const trimmed = query.value.trim()
  if (trimmed.length === 0) {
    hits.value = []
    total.value = 0
    indexedSegments.value = 0
    // **THÊM (retro Epic 5, AI-3)** — cùng lý lẽ `indexedSegments`: ô tìm rỗng nghĩa là "chưa
    // có lượt tìm nào để mô tả", nên `librarySearchCoverageGap` phải trả `null` (worksTotal = 0).
    worksTotal.value = 0
    worksWithText.value = 0
    shortQuery.value = false
    truncated.value = false
    hasLoaded.value = false
    busy.value = false
    error.value = null
    cursor.value = 0
    // 🔵 THÊM Story 5.10 — hai trường mô tả LƯỢT TÌM gần nhất, không phải lựa chọn của người
    // dùng (`mode` giữ nguyên): ô tìm rỗng nghĩa là "chưa có lượt tìm nào để mô tả".
    effectiveMode.value = mode.value
    widened.value = false
    reloadPending = false
    return
  }

  if (busy.value) {
    reloadPending = true
    return
  }

  busy.value = true
  error.value = null
  const mySequence = ++sequence

  const result = await searchLibrary(trimmed, undefined, mode.value)
  // ⚠️ **Đo 2026-08-29 — hàng rào này KHÔNG với tới được từ đường gõ nhanh, và câu chú thích
  // đầu tiên ở đây ("một lượt MỚI hơn đã bắt đầu") nói sai lý do.** Cửa `busy` ngay trên
  // (`:142-145`) chặn lượt hai TRƯỚC khi nó kịp bump `sequence`, nên hai lượt không bao giờ
  // cùng bay; thứ giữ kết quả mới khỏi bị kết quả cũ ghi đè là cặp `busy` + `reloadPending`
  // (ca `chống đua giữa hai lượt gõ` của `tests/frontend/librarySearch.test.ts` đo đúng cặp
  // đó). Đường DUY NHẤT làm vế này đúng hôm nay là một lượt [`resetLibrarySearch`] chạy giữa
  // chừng — nó bump `sequence` ở `:222`. Giữ hàng rào vì lượt vứt state đó phải thắng một lượt
  // IPC đang bay; đừng đọc nó như một cơ chế chống đua giữa hai lượt gõ.
  if (mySequence !== sequence) return

  busy.value = false
  if (result.error !== null) {
    error.value = result.error
    await runPendingSearchReload()
    return
  }
  if (result.report === null) {
    // Không có cầu IPC -- im lặng, cùng nhánh mọi adapter khác.
    await runPendingSearchReload()
    return
  }

  hits.value = result.report.hits
  total.value = result.report.total
  indexedSegments.value = result.report.indexed_segments
  worksTotal.value = result.report.works_total
  worksWithText.value = result.report.works_with_text
  shortQuery.value = result.report.short_query
  truncated.value = result.report.truncated
  effectiveMode.value = result.report.effective_mode
  widened.value = result.report.widened
  hasLoaded.value = true
  // §I/O Matrix "Con trỏ ra ngoài sau lọc" (cùng lý lẽ `libraryWorks.ts`) -- kẹp NGAY, trước
  // khi ai đọc `currentLibrarySearchHit`.
  clampCursor()
  await runPendingSearchReload()
}

async function runPendingSearchReload(): Promise<void> {
  if (!reloadPending) return
  reloadPending = false
  await runLibrarySearch()
}

/**
 * Đặt cờ chế độ rồi chạy LẠI lượt tìm — **chỉ khi** ô tìm không rỗng (§Always: "cả hai nút chế
 * độ chạy lại lượt tìm nếu đã có truy vấn"; §I/O Matrix "Đổi chế độ khi ô tìm rỗng ⇒ 0 lượt
 * IPC; chỉ cờ chế độ đổi"). Dùng CHUNG bởi hai hàm export ngay dưới — chúng chỉ khác giá trị
 * gán cho `mode`.
 *
 * 🔴 Một danh sách kết quả CŨ nằm dưới một nhãn chế độ MỚI là đúng lỗi mà `librarySearchStatus`
 * đã phải sửa một lần ở vòng rà 5.9 (`busy` thắng trước) — `runLibrarySearch()` tự đặt `busy`
 * NGAY khi bắt đầu, nên không có cửa sổ nào để người dùng thấy hit cũ dưới nhãn mới.
 */
function setLibrarySearchMode(next: SearchMode): void {
  mode.value = next
  if (query.value.trim().length === 0) return
  void runLibrarySearch()
}

/** Handler của `library.search_mode_exact`. */
export function setLibrarySearchModeExact(): void {
  setLibrarySearchMode('exact')
}

/** Handler của `library.search_mode_lenient`. */
export function setLibrarySearchModeLenient(): void {
  setLibrarySearchMode('lenient')
}

/** Chuyển con trỏ danh sách kết quả xuống ô kế tiếp — không vòng. No-op trên danh sách rỗng. */
export function nextLibrarySearchHit(): void {
  if (cursor.value < hits.value.length - 1) cursor.value += 1
}

/** Chuyển con trỏ danh sách kết quả lên ô trước — không vòng. */
export function prevLibrarySearchHit(): void {
  if (cursor.value > 0) cursor.value -= 1
}

/**
 * Mở kết quả ĐANG CHỌN vào Workspace — lệnh `library.open_search_hit`. No-op trên danh sách
 * rỗng. Xem khối 🔴 ở doc-comment đầu tệp cho lý do thứ tự CHỜ TỪNG BƯỚC.
 *
 * Chép khuôn `libraryChapters.ts::openCurrentChapter`: chỉ đổi chế độ khi Chương THẬT SỰ mở
 * được — một lượt CHẶN (flush trượt) hay lỗi phải giữ người dùng ở Library để họ thấy được lý
 * do, không đẩy họ sang một Workspace trống/sai.
 */
export async function openCurrentLibrarySearchHit(): Promise<void> {
  const hit = currentLibrarySearchHit.value
  if (hit === null) return

  await openWorkById(hit.work_id)
  // 🔴 `openWorkById` đã CHỞ flush + đổi `OpenWorkState` + vứt/nạp lại state Tác phẩm —
  // nhưng `await` xong KHÔNG nghĩa là nó THÀNH CÔNG: một lượt bị CHẶN (flush trượt) hay lỗi
  // IPC vẫn `await` xong bình thường, chỉ là Tác phẩm KHÔNG đổi. Phát `openChapterById` trong
  // ca đó sẽ gọi `open_chapter` trên Tác phẩm ĐANG MỞ (có thể là Tác phẩm CŨ, hoặc không Tác
  // phẩm nào) với một `chapter_id` thuộc về Tác phẩm KHÁC — đúng "CỬA SỔ MỞ NHẦM CHƯƠNG".
  if (libraryOpenWorkError.value !== null || libraryOpenWorkNotice.value !== null) return

  const moved = await openChapterById(hit.chapter_id, hit.segment_id ?? undefined)
  if (moved) setMode('workspace')
}

/**
 * 🔴 Vứt toàn bộ state — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`. Dùng bởi bàn đo/test; sản phẩm không có chỗ gọi (khối này sống suốt phiên).
 */
export function resetLibrarySearch(): void {
  sequence += 1
  query.value = ''
  hits.value = []
  total.value = 0
  indexedSegments.value = 0
  worksTotal.value = 0
  worksWithText.value = 0
  shortQuery.value = false
  truncated.value = false
  hasLoaded.value = false
  busy.value = false
  reloadPending = false
  error.value = null
  cursor.value = 0
  mode.value = 'exact'
  effectiveMode.value = 'exact'
  widened.value = false
}
