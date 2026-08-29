/**
 * State + thao tác của khối "Tìm kiếm" ở Library — Story 5.9, FR8: tìm một câu từng dịch ở
 * bất kỳ đâu trong cả thư viện, đồng thời trên nguyên văn và bản dịch.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ VÌ SAO MỘT MODULE THUẦN RIÊNG, KHÔNG VIẾT THẲNG TRONG `LibraryMode.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `libraryWorks.ts`/`libraryChapters.ts`: AD-34 §1 đòi mọi `@click` là đúng một
 * `dispatch('<id>')`, và hai thao tác mới (`library.search` · `library.open_search_hit`) đăng
 * ký ở `src/commands/index.ts` như các `CommandDeps` TIÊM VÀO — `src/main.ts` nối chúng vào
 * `installCommands`. Module này là phía CUNG CẤP.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 NĂM CA RỖNG PHÂN BIỆT ĐƯỢC — §Always của story, không gộp
 * ─────────────────────────────────────────────────────────────────────────────
 * ① chưa gõ gì (`searchHasLoaded === false` sau khi ô tìm rỗng) · ② đang tìm (`searchBusy`) ·
 * ③ chỉ mục chưa có dòng nào (`searchIndexedSegments === 0`) · ④ truy vấn dưới 3 ký tự
 * (`searchShortQuery === true` VÀ `hits` rỗng — một truy vấn ngắn VẪN có thể khớp ở nửa bản
 * dịch, xem [`runLibrarySearch`]) · ⑤ chỉ mục có N dòng mà không khớp. `LibraryMode.vue` đọc
 * đúng thứ tự ưu tiên đó, không suy luận lại.
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
import type { SearchHit } from '../config/library'
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
const shortQuery = ref(false)
/** Xem [`librarySearchTruncated`]. */
const truncated = ref(false)
const hasLoaded = ref(false)
const busy = ref(false)
const error = ref<IpcError | null>(null)
// Con trỏ ô đang chọn trong danh sách kết quả — chép khuôn `workCursor` của `libraryWorks.ts`.
const cursor = ref(0)
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
export const librarySearchShortQuery: DeepReadonly<Ref<boolean>> = readonly(shortQuery)
/** 🔴 `true` ⇔ danh sách đã bị trần CẮT — `librarySearchTotal` khi đó là *"số hàng đang
 * hiện"*, không phải *"số hàng khớp"*. Giao diện phải nói ra: một trần cắt trong im lặng biến
 * một con số thành một lời khai sai. */
export const librarySearchTruncated: DeepReadonly<Ref<boolean>> = readonly(truncated)
export const librarySearchHasLoaded: DeepReadonly<Ref<boolean>> = readonly(hasLoaded)
export const librarySearchBusy: DeepReadonly<Ref<boolean>> = readonly(busy)
export const librarySearchError: DeepReadonly<Ref<IpcError | null>> = readonly(error)
export const librarySearchCursor: DeepReadonly<Ref<number>> = readonly(cursor)

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

/** Danh mục ĐÓNG, năm giá trị — khớp ĐÚNG năm ca rỗng/có-kết-quả của §I/O Matrix, theo thứ tự
 * ưu tiên đã ghi ở khối doc-comment đầu tệp. */
export type LibrarySearchStatus = 'not_typed' | 'searching' | 'index_empty' | 'short_query' | 'no_match' | 'result'

/**
 * **Hàm THUẦN** — suy trạng thái hiển thị từ bốn dữ kiện thô, tách khỏi mọi `ref` để vitest
 * kiểm được TẤT ĐỊNH mà không cần mount `LibraryMode.vue` (cùng lý do `chapterWindow` của
 * `libraryChapters.ts` là một hàm thuần: `tests/AGENTS.md`, "hành vi của module thuần").
 *
 * ⚠️ Thứ tự các nhánh LÀ hợp đồng — `short_query` chỉ được đọc SAU khi `hits.length === 0`,
 * vì một truy vấn ngắn VẪN có thể khớp thật ở nửa bản dịch (unicode61 không có sàn 3 ký tự).
 */
export function librarySearchStatus(
  hasLoaded: boolean,
  busy: boolean,
  indexedSegmentsValue: number,
  hitCount: number,
  shortQueryValue: boolean,
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
  if (hitCount === 0 && shortQueryValue) return 'short_query'
  if (hitCount === 0) return 'no_match'
  return 'result'
}

/** `computed` đọc live -- `LibraryMode.vue` dùng thẳng cái này, không tự lặp lại logic. */
export const librarySearchStatusKey = computed<LibrarySearchStatus>(() =>
  librarySearchStatus(hasLoaded.value, busy.value, indexedSegments.value, hits.value.length, shortQuery.value),
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
    shortQuery.value = false
    truncated.value = false
    hasLoaded.value = false
    busy.value = false
    error.value = null
    cursor.value = 0
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

  const result = await searchLibrary(trimmed)
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
  shortQuery.value = result.report.short_query
  truncated.value = result.report.truncated
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
  shortQuery.value = false
  truncated.value = false
  hasLoaded.value = false
  busy.value = false
  reloadPending = false
  error.value = null
  cursor.value = 0
}
