<script setup lang="ts">
// Chế độ 1/3 — kho Tác phẩm và **điểm vào ứng dụng** (PRD §5.2). Story 1.6 · AC3 · AC4.
// Story 1.15 thêm đường nhập tối thiểu (AC1/AC8/NFR17): dán văn bản · kéo-thả · ô nhập
// đường dẫn — đủ để có văn bản mà Story 1.16 hiển thị ở Panel Source.
//
// 🔵 SỬA (2026-08-27, Story 5.4) — câu cũ gộp CẢ "bốn trạng thái vòng đời" vào phần chưa dựng
// đã HẾT ĐÚNG: khối "Tác phẩm" (danh sách phẳng + bốn nút lọc) và khối "Trạng thái Tác phẩm
// đang mở" (ba lệnh vòng đời) nay có mặt ở đây.
// 🔵 SỬA (2026-08-28, Story 5.6) — câu "Lưới Tác phẩm (bìa, sắp xếp, lọc thể loại/ngôn ngữ)
// thuộc Story 5.6 — đây vẫn là một khung rỗng cho phần đó" đã HẾT ĐÚNG: khối `.works-block`
// nay là một LƯỚI (`.works-grid`/`.work-cell`, khung bìa + biểu diễn thay thế), ba `<select>`
// lọc lĩnh vực/ngôn ngữ/sắp xếp, và con trỏ ô (`workCursor`, AC7) đều có mặt.
//
// Không chuỗi tiếng Việt nào trong tệp này (NFR16, AD-21) — mọi nhãn đi qua `t()`.
import { computed, nextTick, onActivated, onBeforeUnmount, onMounted, useTemplateRef, watch } from 'vue'
import { declareFocus, dispatch, enterFocus, releaseFocus } from '../commands'
import type { WorkRow, WorkSortKey } from '../config/library'
import { createdWork, loadDestinationWorks, unwireDragDrop, wireDragDropOnce } from './libraryImport'
import {
  currentLibraryOrphan,
  currentLibraryRoot,
  firstLibraryConflict,
  libraryAppeared,
  libraryConflicts,
  libraryIndexRevision,
  libraryIndexedCount,
  libraryMoved,
  libraryNewlyOrphaned,
  libraryRebuiltAtLabel,
  libraryRebuiltAt,
  libraryOrphanCursor,
  libraryOrphans,
  libraryRescanBusy,
  libraryRescanError,
  libraryRescanResultHasLoadedState,
  libraryRootMissing,
  libraryScanHasLoadedState,
  libraryConflictCount,
  librarySkippedCount,
  libraryTextSkippedCount,
  loadLibraryOrphans,
  loadLibraryRebuiltAt,
} from './libraryRescan'
import {
  libraryFilterIsEmpty,
  libraryGenreFilter,
  libraryGenres,
  libraryShowsFirstRunInvitation,
  librarySortKey,
  librarySourceLangFilter,
  librarySourceLangs,
  libraryStatusFilter,
  libraryWorkCursor,
  libraryWorks,
  libraryWorksBusy,
  libraryWorksError,
  libraryWorksHaveLoaded,
  libraryWorksMatched,
  libraryWorksTotal,
  loadOpenWorkLifecycle,
  loadWorks,
  setGenreFilter,
  setSortKey,
  setSourceLangFilter,
} from './libraryWorks'
import {
  libraryOpenWorkBusy,
  libraryOpenWorkError,
  libraryOpenWorkNotice,
  closeLibraryWorkPage,
  libraryWorkPageIsOpen,
  loadChapters,
} from './libraryChapters'
import LibraryWorkPage from './LibraryWorkPage.vue'
// ── Story 5.9 — "Tìm kiếm full-text xuyên Library" (FR8) ─────────────────────────────
// 🔵 SỬA (2026-08-29, Story 5.10) — thêm `librarySearchMode` cho hai nút chế độ + đọc thêm
// `match_kind` của mỗi hit; `role="status"` mở rộng tám nhánh (đọc thẳng `librarySearchStatusKey`,
// không cần một export mới cho `widened` — tám giá trị của nó đã chở đủ thông tin).
// 🔵 SỬA (retro Epic 5, AI-3 — 2026-09-03) — thêm `librarySearchCoverageGapState`, bề mặt ĐỘC
// LẬP với `librarySearchStatusKey` (§Always: "phải hiện cả khi có kết quả").
import LibrarySearchResults from './LibrarySearchResults.vue'
import {
  librarySearchBusy,
  librarySearchError,
  librarySearchHasLoaded,
  librarySearchMode,
  librarySearchQuery,
  librarySearchScope,
  rerunLibrarySearch,
  setLibrarySearchScope,
} from './librarySearch'
import type { SearchScope } from '../config/library'
import { t, tError } from '../i18n'
import { shortcutRows } from '../config/shortcutsState'
import { workCardMeta } from './libraryWorkCard'

const root = useTemplateRef<HTMLElement>('root')

const searchActive = computed(
  () => librarySearchHasLoaded.value || librarySearchBusy.value || librarySearchError.value !== null,
)

watch([libraryStatusFilter, librarySourceLangFilter, librarySortKey], () => {
  void rerunLibrarySearch()
})

function onSearchScopeChange(scope: SearchScope): void {
  setLibrarySearchScope(scope)
}

const workCards = computed(() => libraryWorks.value.map((work) => workCardMeta(work, new Date())))

const importChord = computed(
  () => shortcutRows.value.find((row) => row.id === 'library.import.open')?.display.join(' / ') ?? '',
)

onMounted(() => {
  declareFocus('mode.library', () => root.value)
  // ⚠️ Gắn MỘT LẦN ở `onMounted`, không `onActivated`: `<KeepAlive>` giữ subtree
  // (§Quyết định thiết kế #6) nên `mounted` chỉ chạy lượt đầu — gắn ở `onActivated` sẽ
  // không lặp lại gì thêm, nhưng đặt nó ở `onMounted` khớp đúng vòng đời "một lần" của
  // `wireDragDropOnce()` (tự chốt idempotent, nhưng ý định code phải khớp cơ chế).
  wireDragDropOnce()
  // 🔴 THÊM (Story 6.7b) — tải TRƯỚC khi người dùng chạm radio "Tác phẩm đã có": I/O Matrix
  // "Empty Library" đòi điều khiển NÓI vì sao ngay từ đầu, không chỉ sau lượt bấm đầu tiên
  // (`destinationExistingWorkAvailable` đọc `destinationWorksHaveLoaded`/`destinationWorks`
  // — cả hai còn ở trạng thái "chưa biết" cho tới khi lượt này về).
  void loadDestinationWorks()
})
onBeforeUnmount(() => {
  releaseFocus('mode.library')
  // ⚠️ Bộ nghe kéo-thả là tầng CỬA SỔ, không phải của `.dropzone` — để nó sống sau khi
  // chế độ bị tháo nghĩa là một cú thả ở Workspace vẫn điền vào một form không còn trên
  // màn hình. Code review 2026-08-06.
  unwireDragDrop()
})
// `onActivated` chứ không phải `onMounted`: ba chế độ sống trong `<KeepAlive>` (§Quyết
// định thiết kế #6), nên lần hiện thứ hai trở đi KHÔNG có `mounted`. Ở lượt mount đầu
// tiên Vue gọi `mounted` rồi `activated`, nên điểm vào đã khai xong trước khi dùng.
onActivated(() => {
  void enterFocus('mode.library')
  // Story 5.4 — tải lại MỖI LẦN quay về Library: một Chương có thể vừa đổi trạng thái từ
  // Workspace (qua `lifecycle.set_chapter_done`), và danh sách/khối "Tác phẩm đang mở" phải
  // phản ánh giá trị mới nhất, không giữ ảnh chụp của lần hiện trước.
  void loadWorks()
  void loadOpenWorkLifecycle()
  // Story 5.7 — cùng lý do: danh sách Chương của Tác phẩm ĐANG MỞ phải phản ánh giá trị mới
  // nhất mỗi lần quay lại Library (một Chương có thể vừa đổi trạng thái từ Workspace).
  void loadChapters()
  // Khối "Mồ côi" phải hiện đúng trạng thái NGAY khi vào
  // Library, không đợi người dùng tự bấm Quét lại: `.atproj` mồ côi có thể đã xuất hiện trên
  // đĩa (ví dụ đồng bộ đám mây) từ một phiên trước. `loadLibraryOrphans` là một lượt ĐỌC
  // thuần, không `Indexer::rebuild` — cùng nguyên lý `loadWorks` ngay trên (danh sách phải
  // phản ánh trạng thái thật, không giữ ảnh chụp của lần hiện trước).
  void loadLibraryOrphans()
  void loadLibraryRebuiltAt()
})

watch(libraryIndexRevision, () => {
  void loadWorks()
})

// 🔵 THÊM (2026-08-28) — KHUYẾT TẬT ĐO ĐƯỢC, tìm ra ở lượt chạy e2e đầu tiên của Story 5.4.
//
// `onActivated` là lối vào DUY NHẤT của hai lượt tải trên, và ở lượt khởi động nó chạy khi
// CHƯA Tác phẩm nào mở — `read_work_lifecycle` trả `work.none_open`, `openWorkLifecycleLoaded`
// ở lại `false`. Tạo một Tác phẩm ngay tại màn hình này KHÔNG rời chế độ, nên `onActivated`
// không chạy lại: khối "Tác phẩm đang mở" tiếp tục nói "Chưa có Tác phẩm nào đang mở" trong
// khi CÓ một Tác phẩm vừa mở, và cả ba nút vòng đời kẹt ở `disabled` cho tới khi người dùng
// tình cờ chuyển chế độ rồi quay lại. Một màn hình khẳng định điều nó biết là sai — đúng lớp
// lỗi mà `AGENTS.md::Known pitfalls` đặt lên hàng đầu.
//
// ⚠️ Theo dõi `createdWork` chứ không gọi từ `libraryImport.ts`: đường nhập không được biết
// gì về vòng đời (hai module thuần, hai vai), còn `.vue` này vốn đã là nơi điều phối cả hai
// lượt tải ở `onActivated` ngay trên.
watch(createdWork, (created) => {
  if (created === null) return
  closeLibraryWorkPage()
  void loadOpenWorkLifecycle()
  void loadWorks()
  // Story 5.7 — một Tác phẩm MỚI thay `OpenWorkState`, nên danh sách Chương của nó (một
  // Chương, fresh) phải nạp lại đúng lý lẽ hai lượt tải ngay trên.
  void loadChapters()
})

// **KHÔNG có handler `dragenter`/`dragover`/`dragleave`/`drop` của DOM ở đây, và đó là
// một quyết định** — code review 2026-08-06. `drag_drop_enabled` mặc định `true` ở Tauri
// v2 (`tauri.conf.json` không override), nên bộ xử lý tầng **hệ điều hành** giành lấy
// thao tác kéo và webview **không bao giờ** nhận được các sự kiện DOM đó. Bản trước gắn
// cả bốn, và cả bốn đều là mã chết: `.dropzone.over` không bao giờ bật, nên vùng kéo-thả
// không có một tín hiệu nào cho biết nó còn sống.
//
// ⇒ Cả ba trạng thái (vào · rời · thả) đến từ **Rust** qua `on_window_event`, xem
// `src-tauri/src/lib.rs::wire_drag_drop` và `./libraryImport.ts::wireDragDropOnce`.

/**
 * Story 5.5 — phần trăm bề rộng thanh tiến độ. THUẦN trình bày (không quy tắc nghiệp vụ,
 * AD-1): "đã xong" đến nguyên vẹn từ `WorkRow.chapter_done_count` (đã tính ở Rust,
 * `WorkMeta::rebuild_from_store`), hàm này chỉ đổi một cặp số nguyên thành một tỉ lệ hiển thị.
 *
 * `chapter_count === 0` ⇒ `0`, KHÔNG chia cho 0 (`NaN` sẽ làm `width: NaN%` — CSS im lặng bỏ
 * qua giá trị đó, nhưng đó là một trạng thái không viết ra được, không phải "vẽ ở 0%" như
 * §I/O Matrix đòi). Chỉ gọi khi `chapter_done_count !== null` — chỗ gọi trong `<template>`
 * đã canh bằng `v-if`.
 */
function progressPercent(work: WorkRow): number {
  if (work.chapter_count === 0) return 0
  const done = work.chapter_done_count ?? 0
  return Math.min(100, Math.round((done / work.chapter_count) * 100))
}

/**
 * Story 5.6 — biểu diễn thay thế nhất quán cho khung bìa (AC6): chữ cái đầu của tên, viết
 * hoa. `name` rỗng/toàn khoảng trắng ⇒ `'?'` (§I/O Matrix "Tên rỗng/khoảng trắng") — KHÔNG
 * một ô trống, và KHÔNG chuỗi rỗng (một ô trống là chính điều AC6 cấm).
 */
function coverInitial(name: string): string {
  const trimmed = name.trim()
  if (trimmed.length === 0) return '?'
  return trimmed.charAt(0).toUpperCase()
}

/**
 * Ba `<select>` (lĩnh vực · ngôn ngữ · sắp xếp) dùng `@change`, NGOÀI luật Kiểm A (§Design
 * Notes "Vì sao `<select>` chứ không thêm nút") — chúng gọi THẲNG các hàm mở tường của
 * `libraryWorks.ts`, không qua `dispatch()`. Chuỗi rỗng (`<option value="">`) ⇒ bỏ lọc,
 * đúng quy ước "mọi lĩnh vực"/"mọi ngôn ngữ".
 */
function onGenreFilterChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value
  setGenreFilter(value === '' ? null : value)
}
function onSourceLangFilterChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value
  setSourceLangFilter(value === '' ? null : value)
}
function onSortKeyChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value as WorkSortKey
  setSortKey(value)
}

// AC7 — ô đang chọn phải LUÔN nhìn thấy được. `workCellRefs` gom MỌI `<li>` của lưới theo
// đúng thứ tự `v-for` (Vue điền mảng này tự động khi `ref="workCellRefs"` sống trong
// `v-for`); `flush: 'post'` để DOM đã cập nhật xong trước khi tính lại ô đích (con trỏ có
// thể đổi CÙNG lượt danh sách rút ngắn, xem `clampWorkCursor` ở `libraryWorks.ts`).
const workCellRefs = useTemplateRef<HTMLLIElement[]>('workCellRefs')
watch(
  libraryWorkCursor,
  () => {
    void nextTick(() => {
      const cell = workCellRefs.value?.[libraryWorkCursor.value]
      // `block: 'nearest'`/`inline: 'nearest'` — tức thì theo mặc định (không `behavior:
      // 'smooth'`), cùng chủ ý "không hiệu ứng" mà `GridPanel.vue` §AC8 đã ghi cho cuộn theo
      // con trỏ.
      cell?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
    })
  },
  { flush: 'post' },
)

// Closing the Work page returns focus to the card's open button (UX-DR17), else to the mode root.
watch(libraryWorkPageIsOpen, (open, wasOpen) => {
  if (open || !wasOpen) return
  void nextTick(() => {
    const opener = root.value?.querySelector<HTMLElement>('[data-library-open-work]') ?? null
    if (opener !== null) opener.focus()
    else root.value?.focus()
  })
})
</script>

<template>
  <!-- `tabindex="-1"` để phần tử nhận được focus lập trình. Nó KHÔNG vào thứ tự Tab. -->
  <section ref="root" class="mode" tabindex="-1">
    <LibraryWorkPage v-if="libraryWorkPageIsOpen" />
    <template v-else>
    <div class="top-bar">
      <div class="top-row">
        <div class="search-field">
          <!-- aura-allow-text: decorative glyph, hidden from assistive tech. -->
          <span class="search-icon" aria-hidden="true">⌕</span>
          <input
            v-model="librarySearchQuery"
            type="text"
            class="search-input"
            autocomplete="off"
            :placeholder="t('mode.library.search_placeholder')"
            :aria-label="t('mode.library.search_placeholder')"
            data-library-search-input
            @keyup.enter="dispatch('library.search')"
          />
        </div>
      <div class="segmented" role="group" :aria-label="t('mode.library.search_mode_heading')">
        <button
          type="button"
          class="segment"
          data-library-search-mode="exact"
          :aria-pressed="librarySearchMode === 'exact'"
          @click="dispatch('library.search_mode_exact')"
        >
          {{ t('mode.library.search_mode_exact') }}
        </button>
        <button
          type="button"
          class="segment"
          data-library-search-mode="lenient"
          :aria-pressed="librarySearchMode === 'lenient'"
          @click="dispatch('library.search_mode_lenient')"
        >
          {{ t('mode.library.search_mode_lenient') }}
        </button>
      </div>
      <div class="segmented" role="radiogroup" :aria-label="t('mode.library.search_scope_heading')">
        <label class="segment" :class="{ 'segment--on': librarySearchScope === 'both' }">
          <input
            type="radio"
            name="library-search-scope"
            data-library-search-scope="both"
            :checked="librarySearchScope === 'both'"
            @change="onSearchScopeChange('both')"
          />
          {{ t('mode.library.search_scope_both') }}
        </label>
        <label class="segment" :class="{ 'segment--on': librarySearchScope === 'source' }">
          <input
            type="radio"
            name="library-search-scope"
            data-library-search-scope="source"
            :checked="librarySearchScope === 'source'"
            @change="onSearchScopeChange('source')"
          />
          {{ t('mode.library.search_scope_source') }}
        </label>
        <label class="segment" :class="{ 'segment--on': librarySearchScope === 'target' }">
          <input
            type="radio"
            name="library-search-scope"
            data-library-search-scope="target"
            :checked="librarySearchScope === 'target'"
            @change="onSearchScopeChange('target')"
          />
          {{ t('mode.library.search_scope_target') }}
        </label>
      </div>
        <button type="button" class="btn btn-primary" data-library-import-open @click="dispatch('library.import.open')">
          {{ t('command.library.import.open') }}
        </button>
      </div>


      <!--
        🔵 THÊM (2026-08-28) `data-lifecycle-filter`/`data-lifecycle-action` — MÓC ĐỊNH DANH
        cho `e2e/specs/story-5-4-lifecycle.e2e.mjs`, cùng khuôn `data-shortcuts-open`
        (`ShortcutsOverlay`) và `data-col` (lưới) đã có. Trước đó spec định vị bằng
        `:nth-of-type(n)`: đổi thứ tự sáu nút làm spec bấm NHẦM nút và vẫn xanh, thay vì đỏ ở
        đúng chỗ vừa đổi. Một móc đọc được là rẻ hơn một lượt đỏ sai nguyên nhân.
      -->
      <!--
        🔴 BỐN nút RIÊNG, KHÔNG một `v-for` chọn id động — `check:commands` Kiểm A đòi mỗi
        `@click` là ĐÚNG MỘT lời gọi `dispatch('<id>')` với id LITERAL, đọc TĨNH được từ mã
        nguồn; một biểu thức ba ngôi chọn id lúc chạy không đọc tĩnh được (đã đo: Kiểm A đỏ).
      -->
      <div class="filter-actions">
        <button
          type="button"
          class="btn"
          data-lifecycle-filter="not_started"
          :aria-pressed="libraryStatusFilter.has('not_started')"
          :disabled="libraryWorksBusy"
          @click="dispatch('library.filter_not_started')"
        >
          {{ t('lifecycle.not_started') }}
        </button>
        <button
          type="button"
          class="btn"
          data-lifecycle-filter="in_progress"
          :aria-pressed="libraryStatusFilter.has('in_progress')"
          :disabled="libraryWorksBusy"
          @click="dispatch('library.filter_in_progress')"
        >
          {{ t('lifecycle.in_progress') }}
        </button>
        <button
          type="button"
          class="btn"
          data-lifecycle-filter="paused"
          :aria-pressed="libraryStatusFilter.has('paused')"
          :disabled="libraryWorksBusy"
          @click="dispatch('library.filter_paused')"
        >
          {{ t('lifecycle.paused') }}
        </button>
        <button
          type="button"
          class="btn"
          data-lifecycle-filter="done"
          :aria-pressed="libraryStatusFilter.has('done')"
          :disabled="libraryWorksBusy"
          @click="dispatch('library.filter_done')"
        >
          {{ t('lifecycle.done') }}
        </button>
        <button
          type="button"
          class="btn"
          data-lifecycle-action="clear_filter"
          :disabled="libraryWorksBusy || libraryFilterIsEmpty"
          @click="dispatch('library.filter_clear')"
        >
          {{ t('mode.library.clear_filter') }}
        </button>
        <button
          type="button"
          class="btn"
          data-lifecycle-action="list_works"
          :disabled="libraryWorksBusy"
          @click="dispatch('library.list_works')"
        >
          {{ t('mode.library.list_works') }}
        </button>
      </div>

      <!--
        Story 5.6 — ba `<select>` (lĩnh vực · ngôn ngữ · sắp xếp), tập-mở nên KHÔNG viết được
        thành nút literal (§Design Notes "Vì sao `<select>` chứ không thêm nút"). `@change`
        is outside both Kiểm A and Kiểm K (not one of Kiểm K's five event types).
        `<option>` dựng từ hai mảng DO
        RUST TRẢ VỀ (`libraryGenres`/`librarySourceLangs`, `DISTINCT` trên bảng CHƯA LỌC) —
        KHÔNG BAO GIỜ suy từ `libraryWorks` đã lọc (AD-1, §Always).
      -->
      <div class="filter-selects">
        <label class="field">
          <span>{{ t('mode.library.filter_genre_label') }}</span>
          <select
            data-library-genre-filter
            :value="libraryGenreFilter ?? ''"
            :disabled="libraryWorksBusy"
            @change="onGenreFilterChange"
          >
            <option value="">{{ t('mode.library.filter_all_genres') }}</option>
            <!-- aura-allow-text: lĩnh vực là DỮ LIỆU người dùng gõ, không một câu UI. -->
            <option v-for="genreOption in libraryGenres" :key="genreOption" :value="genreOption">
              {{ genreOption }}
            </option>
          </select>
        </label>
        <label class="field">
          <span>{{ t('mode.library.filter_source_lang_label') }}</span>
          <select
            data-library-source-lang-filter
            :value="librarySourceLangFilter ?? ''"
            :disabled="libraryWorksBusy"
            @change="onSourceLangFilterChange"
          >
            <option value="">{{ t('mode.library.filter_all_source_langs') }}</option>
            <!-- aura-allow-text: ngôn ngữ nguồn là DỮ LIỆU (mã ngôn ngữ), không một câu UI. -->
            <option v-for="langOption in librarySourceLangs" :key="langOption" :value="langOption">
              {{ langOption }}
            </option>
          </select>
        </label>
        <label class="field">
          <span>{{ t('mode.library.sort_label') }}</span>
          <select data-library-sort :value="librarySortKey" :disabled="libraryWorksBusy" @change="onSortKeyChange">
            <option value="updated_desc">{{ t('mode.library.sort_updated_desc') }}</option>
            <option value="name_asc">{{ t('mode.library.sort_name_asc') }}</option>
          </select>
        </label>
      </div>
    </div>

    <LibrarySearchResults v-if="searchActive" />
    <template v-else>
    <p class="status" role="status" data-library-search-status>{{ t('mode.library.search_not_typed') }}</p>

    <div class="works-block">
      <p class="section-heading">{{ t('mode.library.works_heading') }}</p>
      <!-- role="status" LUÔN có mặt (không v-if) -- cùng khuôn dải trạng thái của khối quét lại. -->
      <!-- aura-allow-text: ba nhánh đều qua t()/chuỗi rỗng -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi. -->
      <p class="status" role="status">
        {{
          !libraryWorksHaveLoaded
            ? t('mode.library.works_not_loaded')
            : libraryShowsFirstRunInvitation
              ? ''
              : libraryWorksTotal === 0
              ? t('mode.library.works_empty')
              : libraryWorksMatched === 0
                ? t('mode.library.works_no_match', { total: String(libraryWorksTotal) })
                : t('mode.library.works_result', { matched: String(libraryWorksMatched), total: String(libraryWorksTotal) })
        }}
      </p>
      <!-- aura-allow-text: như trên, qua tError(). -->
      <p class="error" role="status">{{ libraryWorksError ? tError(libraryWorksError) : '' }}</p>

      <div v-if="libraryShowsFirstRunInvitation" class="empty" data-library-empty>
        <div class="rule" aria-hidden="true"></div>
        <p class="big">{{ t('mode.library.status') }}</p>
        <p class="small">{{ t('mode.library.empty_body') }}</p>
        <div class="empty-actions">
          <button type="button" class="btn btn-primary" data-library-empty-invite="paste" @click="dispatch('library.import.open')">
            {{ t('mode.library.empty_invite_paste') }}
          </button>
          <button type="button" class="btn" data-library-empty-invite="file" @click="dispatch('library.import.open')">
            {{ t('mode.library.empty_invite_file') }}
          </button>
        </div>
        <!-- aura-allow-text: qua t(); phím tắt đã định dạng theo nền tảng. -->
        <p v-if="importChord !== ''" class="small">{{ t('mode.library.empty_shortcut_hint', { chord: importChord }) }}</p>
      </div>

      <!--
        Story 5.6 — lưới Tác phẩm (AC2/AC3/AC4/AC6/AC7), thay danh sách phẳng của Story 5.4.
        Con trỏ (`libraryWorkCursor`) di chuyển bằng HAI NÚT thật (`‹`/`›`, `dispatch()` id
        literal — Kiểm A) chứ không một `@keydown` tự chế trên từng ô: cùng khuôn
        `library.orphan_next`/`orphan_prev` đã có trong chính tệp này.
      -->
      <div v-if="libraryWorks.length > 0" class="grid-nav">
        <button
          type="button"
          class="btn"
          data-library-work-prev
          :aria-label="t('mode.library.work_prev')"
          @click="dispatch('library.work_prev')"
        >
          ‹
        </button>
        <span class="grid-nav-position">{{ t('mode.library.work_position', { current: String(libraryWorkCursor + 1), total: String(libraryWorks.length) }) }}</span>
        <button
          type="button"
          class="btn"
          data-library-work-next
          :aria-label="t('mode.library.work_next')"
          @click="dispatch('library.work_next')"
        >
          ›
        </button>
      </div>

      <ul v-if="libraryWorks.length > 0" class="works-grid" data-library-grid>
        <li
          v-for="(work, workIndex) in libraryWorks"
          ref="workCellRefs"
          :key="work.work_id"
          class="work-cell"
          :class="{ 'work-cell--current': workIndex === libraryWorkCursor }"
          :aria-current="workIndex === libraryWorkCursor ? 'true' : undefined"
          data-library-work-cell
        >
          <div class="work-cover" role="img" :aria-label="t('mode.library.cover_placeholder_label', { name: work.name })">
            <!-- aura-allow-text: biểu diễn thay thế là chữ cái đầu của TÊN, dữ liệu người dùng. -->
            <span class="work-initial" data-library-work-initial>{{ coverInitial(work.name) }}</span>
            <!-- aura-allow-text: nhãn ngôn ngữ nguồn qua t() hoặc chính mã ngôn ngữ (dữ liệu). -->
            <span v-if="workCards[workIndex].langLabel !== null" class="work-lang" data-library-work-lang>{{
              workCards[workIndex].langLabel
            }}</span>
          </div>
          <!-- aura-allow-text: tên Tác phẩm là DỮ LIỆU người dùng gõ, không một câu UI. -->
          <span class="work-name">{{ work.name }}</span>
          <!-- aura-allow-text: lĩnh vực là dữ liệu, nhãn thời gian qua t(). -->
          <span v-if="workCards[workIndex].line !== null" class="work-meta" data-library-work-meta>{{
            workCards[workIndex].line
          }}</span>
          <!--
            aura-allow-text: mọi nhánh đều qua t().
            🔵 SỬA (2026-08-28, lượt rà) — TÁCH nhánh cuối làm HAI. Bản trước cho `status = null`
            (`meta.json` v1, thật sự CHƯA BIẾT) và một `status` mang giá trị NGOÀI danh mục bốn
            (dữ liệu hỏng) đọc lên GIỐNG HỆT nhau, nên một hàng hỏng đội lốt một hàng chưa di
            trú và không ai lần được. Hai câu khác nhau vì hai trạng thái khác nhau.
          -->
          <span class="work-status">
            {{
              work.status === 'not_started'
                ? t('lifecycle.not_started')
                : work.status === 'in_progress'
                  ? t('lifecycle.in_progress')
                  : work.status === 'paused'
                    ? t('lifecycle.paused')
                    : work.status === 'done'
                      ? t('lifecycle.done')
                      : work.status === null
                        ? t('mode.library.works_status_unknown')
                        : t('mode.library.works_status_invalid', { status: work.status })
            }}
          </span>
          <span v-if="work.status_is_override" class="override-marker">{{ t('mode.library.override_marker') }}</span>
          <!--
            Story 5.5 — "Tiến độ Tác phẩm" (FR7). `chapter_done_count === null` ⇒ câu "chưa
            biết" và KHÔNG vẽ thanh (§I/O Matrix: "không hiện 0 /") — một `meta.json` chưa
            từng qua `WorkMeta::rebuild_from_store` không được phép trông như "0 Chương xong".
            aura-allow-text: cả hai nhánh đi qua t() -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi.
          -->
          <span class="work-progress">
            {{
              work.chapter_done_count === null
                ? t('mode.library.works_progress_unknown')
                : t('mode.library.works_progress', {
                    done: String(work.chapter_done_count),
                    total: String(work.chapter_count),
                  })
            }}
          </span>
          <div
            v-if="work.chapter_done_count !== null"
            class="work-progress-track"
            role="progressbar"
            :aria-valuemin="0"
            :aria-valuemax="work.chapter_count"
            :aria-valuenow="work.chapter_done_count"
            :aria-valuetext="
              t('mode.library.works_progress_aria', {
                done: String(work.chapter_done_count),
                total: String(work.chapter_count),
              })
            "
          >
            <div class="work-progress-fill" :style="{ width: progressPercent(work) + '%' }"></div>
          </div>
          <!--
            Story 5.7 — "Mở Tác phẩm" (FR12). Cùng khuôn `library.forget_orphan`: thao tác
            trên `currentLibraryWork` (con trỏ), không một tham số qua `dispatch()` — nên NÚT
            chỉ hiện ở đúng ô đang chọn (`v-if`), tránh việc bấm nút ở một ô KHÁC ô cursor mà
            lại mở đúng Tác phẩm cursor đang trỏ tới, một hành vi lệch trực quan.
          -->
          <button
            v-if="workIndex === libraryWorkCursor"
            type="button"
            class="btn"
            data-library-open-work
            :disabled="libraryOpenWorkBusy"
            @click="dispatch('library.open_work')"
          >
            {{ t('mode.library.open_work_button') }}
          </button>
          <button
            v-if="workIndex === libraryWorkCursor"
            type="button"
            class="btn"
            data-library-open-work-folder
            @click="dispatch('library.open_work_folder')"
          >
            {{ t('mode.library.open_work_folder_button') }}
          </button>
        </li>
      </ul>
    </div>

    </template>

    <div class="created-block">

      <!--
        Bốn node LUÔN có mặt (không `v-if`) để trình đọc màn hình công bố được nội
        dung ĐỔI — cùng lý lẽ với dải báo lỗi cấu hình của `App.vue`. `role="status"`,
        không `role="alert"`: đây là kết quả một thao tác, không phải tình huống khẩn.
      -->
      <!-- aura-allow-text: cả hai nhánh đi qua t()/chuỗi rỗng, không chuỗi viết thẳng
           nào — Kiểm A2 không đọc tĩnh được toán tử ba ngôi. -->
      <p class="status" role="status">
        {{ createdWork ? t('mode.library.created', { name: createdWork.meta.name, folder: createdWork.folder }) : '' }}
      </p>
      <!--
        `source_lang_mismatch` là một
        CẢNH BÁO không chặn (script của văn bản không khớp `source_lang` đã khai): Rust vẫn
        ghi, và dải này chỉ NÓI, không khoá gì. Đứng CẠNH `.status`, không lồng vào trong —
        cùng khuôn ba node kia, mỗi node một câu độc lập.
      -->
      <!-- aura-allow-text: qua t()/chuỗi rỗng, Kiểm A2 không đọc tĩnh được toán tử ba ngôi. -->
      <p class="notice source-lang-warning" role="status">
        {{ createdWork?.source_lang_mismatch === true ? t('mode.library.source_lang_mismatch_warning') : '' }}
      </p>
      <!--
        `images_failed` — chỉ hiện khi CÓ ảnh không tải được (§Never: không hiện một dòng
        "0 ảnh không tải được" vô nghĩa). Nút "xem" mở CÙNG màn Cài đặt > Quyền riêng tư mà
        `ImportPreviewOverlay.vue` dùng cho nhật ký domain — không một màn riêng.
      -->
      <p v-if="(createdWork?.images_failed ?? 0) > 0" class="notice" role="status">
        <!-- aura-allow-text: KẾT QUẢ của `t()`, tham số là DỮ LIỆU (số đếm từ Rust). -->
        {{ t('mode.library.images_failed_warning', { count: String(createdWork?.images_failed ?? 0) }) }}
        <button type="button" class="images-failed-domain-log-view" @click="dispatch('settings.privacy.open')">
          {{ t('mode.library.preview.domain_log_view') }}
        </button>
      </p>
    </div>

    <!--
      Story 5.3 — "Quét lại thư mục" (FR99). Màn hình TỐI THIỂU CÓ CHỦ: chỉ thư mục gốc,
      nút quét lại, danh sách mục mồ côi (con trỏ một mục, cùng khuôn GlossaryQueueOverlay),
      và ba con số kết quả — KHÔNG lưới Tác phẩm/lọc/sắp xếp/bìa/vòng đời/tiến độ (5.4-5.6
      sở hữu).
    -->
    <div class="root-block">
      <p class="section-heading">{{ t('mode.library.root_heading') }}</p>
      <!--
        🔵 SỬA (2026-08-27, vòng rà bốn lớp P2) — bản trước khẳng định một điều CHƯA BIẾT.
        `currentLibraryRoot === null` chỉ nghĩa là "chưa quét lần nào trong PHIÊN NÀY" -- một
        người dùng đã cấu hình gốc riêng từ phiên trước sẽ thấy màn hình nói sai về thư mục
        của họ. Bản trước còn chép cứng `~/Documents/AuraTranslate/` vào chuỗi hiển thị,
        đúng thứ `default_library_root` (Rust) cố ý tránh ("Không viết cứng $HOME") và không
        biểu diễn được ca `document_dir()` trượt. Câu MỚI chỉ nói đúng trạng thái CHƯA BIẾT;
        đường dẫn THẬT chỉ hiện sau khi `RescanReport.root` (do Rust phân giải) về tới đây.
      -->
      <!--
        🔵 THÊM (2026-08-27, vòng rà THỨ HAI P8) — `role="status"`: ba node hàng xóm
        (`.root-missing`/`.status`/`.error`) đều mang nó, còn node này thì không -- đường dẫn
        gốc đổi (sau một lượt quét/đổi thư mục gốc) mà trình đọc màn hình không được báo.
      -->
      <!-- aura-allow-text: `currentLibraryRoot` là ĐƯỜNG DẪN đĩa (dữ liệu), không câu UI; nhánh
           còn lại đi qua t() -- Kiểm A2 không đọc tĩnh được toán tử `??`. -->
      <p class="root-value" role="status">{{ currentLibraryRoot ?? t('mode.library.root_not_scanned_yet') }}</p>
      <!--
        🔵 THÊM (2026-08-27, vòng rà bốn lớp P1) — câu RIÊNG khi gốc đã quét KHÔNG còn tồn
        tại trên đĩa, phân biệt hẳn với "đã quét, không có Tác phẩm nào" (đó là node
        `.status` bên dưới nói ba con số kết quả). role="status" LUÔN có mặt.
      -->
      <!-- aura-allow-text: nhánh đúng đi qua t() với tham số root (dữ liệu); nhánh sai là chuỗi rỗng. -->
      <p class="root-missing" role="status">
        {{ libraryRescanResultHasLoadedState && libraryRootMissing ? t('mode.library.root_missing', { root: currentLibraryRoot ?? '' }) : '' }}
      </p>
      <div class="root-actions">
        <button
          type="button"
          class="btn"
          :disabled="libraryRescanBusy"
          @click="dispatch('library.choose_root')"
        >
          {{ t('mode.library.choose_root') }}
        </button>
        <button
          type="button"
          class="btn"
          :disabled="libraryRescanBusy"
          @click="dispatch('library.rescan')"
        >
          <!-- aura-allow-text: cả hai nhánh đi qua t() -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi. -->
          {{ libraryRescanBusy ? t('mode.library.rescan_busy') : t('mode.library.rescan') }}
        </button>
      </div>

      <!-- role="status" LUÔN có mặt (không v-if) -- cùng lý lẽ với dải báo lỗi của form nhập. -->
      <!-- aura-allow-text: cả hai nhánh đi qua t()/chuỗi rỗng -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi. -->
      <p class="status" role="status">
        {{
          libraryRescanResultHasLoadedState
            ? t('mode.library.rescan_result', {
                indexed: String(libraryIndexedCount),
                conflicts: String(libraryConflictCount),
                skipped: String(librarySkippedCount),
                text_skipped: String(libraryTextSkippedCount),
              })
            : ''
        }}
      </p>
      <div v-if="libraryRescanResultHasLoadedState" class="rescan-changes" data-library-rescan-changes>
        <p
          v-if="libraryAppeared.length === 0 && libraryMoved.length === 0 && libraryNewlyOrphaned.length === 0"
          class="rescan-changes-none"
          data-library-rescan-unchanged
        >
          {{ t('mode.library.changes_none') }}
        </p>
        <template v-else>
          <section v-if="libraryAppeared.length > 0" data-library-changes="appeared">
            <p class="section-heading">{{ t('mode.library.changes_appeared') }}</p>
            <ul>
              <li v-for="entry in libraryAppeared" :key="entry.work_id" data-library-change="appeared" :data-work-id="entry.work_id">
                <!-- aura-allow-text: tên là DỮ LIỆU đĩa. -->
                <span class="change-name">{{ entry.name }}</span>
                <!-- aura-allow-text: đường dẫn là DỮ LIỆU đĩa. -->
                <span class="change-path">{{ entry.atproj_path }}</span>
              </li>
            </ul>
          </section>
          <section v-if="libraryMoved.length > 0" data-library-changes="moved">
            <p class="section-heading">{{ t('mode.library.changes_moved') }}</p>
            <ul>
              <li v-for="entry in libraryMoved" :key="entry.work_id" data-library-change="moved" :data-work-id="entry.work_id">
                <!-- aura-allow-text: tên là DỮ LIỆU; câu nối đi qua t(). -->
                <span class="change-name">{{ entry.name }}</span>
                <span class="change-path">{{ t('mode.library.change_moved_path', { from: entry.from_path, to: entry.to_path }) }}</span>
              </li>
            </ul>
          </section>
          <section v-if="libraryNewlyOrphaned.length > 0" data-library-changes="orphaned">
            <p class="section-heading">{{ t('mode.library.changes_orphaned') }}</p>
            <ul>
              <li v-for="entry in libraryNewlyOrphaned" :key="entry.work_id" data-library-change="orphaned" :data-work-id="entry.work_id">
                <!-- aura-allow-text: tên là DỮ LIỆU đĩa. -->
                <span class="change-name">{{ entry.name }}</span>
                <!-- aura-allow-text: đường dẫn là DỮ LIỆU đĩa. -->
                <span class="change-path">{{ entry.atproj_path }}</span>
              </li>
            </ul>
          </section>
        </template>
      </div>
      <!-- aura-allow-text: như trên, qua tError(). -->
      <p class="error" role="status">{{ libraryRescanError ? tError(libraryRescanError) : '' }}</p>

      <!--
        🔵 THÊM (2026-08-27, phán quyết Ice #3) — AC4 nói "phát hiện VÀ CẢNH BÁO", hai vế.
        Trước bản vá, `conflicts` chỉ là một con số nén nằm CÙNG câu với "đã lập chỉ mục"/"bỏ
        qua" ở node `.status` ngay trên — không phải một bề mặt cảnh báo, đúng lỗ mà mục nợ
        AC4 trong deferred-work.md ghi. Node này TÁCH HẲN, chỉ hiện khi có xung đột, nêu ĐÍCH
        DANH chỗ trùng ĐẦU TIÊN kèm CẢ HAI đường dẫn, và "và N chỗ nữa" khi nhiều hơn một —
        KHÔNG dựng danh sách/lưới các chỗ trùng (Story 5.6 sở hữu phần "hiển thị danh sách";
        story này chỉ nợ 5.6 phần đó, không nợ phần "nói ra rằng có chuyện").
        `role="status"`, cùng khuôn `.root-missing` ngay trên: một cảnh báo phát sinh từ một
        lượt quét đã THÀNH CÔNG, không phải một lỗi thao tác (đó là `.error`, `role="alert"`
        thuộc về `GlossaryQueueOverlay.vue`, không phải đây).
      -->
      <p v-if="firstLibraryConflict" class="conflict-warning" role="status">
        <!-- aura-allow-text: hai đường dẫn là DỮ LIỆU đĩa; câu bao quanh đi qua t(). -->
        {{
          t('mode.library.conflict_warning', {
            kept_path: firstLibraryConflict.kept_path,
            duplicate_path: firstLibraryConflict.duplicate_path,
          })
        }}
        <!-- aura-allow-text: số đếm là DỮ LIỆU; câu bao quanh đi qua t(). -->
        <template v-if="libraryConflicts.length > 1">
          {{ t('mode.library.conflict_more', { count: String(libraryConflicts.length - 1) }) }}
        </template>
      </p>

      <p class="section-heading">{{ t('mode.library.orphans_heading') }}</p>
      <!--
        🔴 Vị từ `libraryScanHasLoadedState` TRƯỚC khi kết luận "không có mục mồ côi nào" —
        danh sách rỗng TRƯỚC lượt quét đầu tiên là "chưa biết", không phải "không có"
        (AGENTS.md::Known pitfalls).
      -->
      <!-- aura-allow-text: ba nhánh đều qua t()/chuỗi rỗng. -->
      <p class="status" role="status">
        {{
          !libraryScanHasLoadedState
            ? t('mode.library.orphans_not_loaded')
            : libraryOrphans.length === 0
              ? t('mode.library.orphans_none')
              : ''
        }}
      </p>

      <div v-if="currentLibraryOrphan" class="orphan-row">
        <!-- aura-allow-text: tên Tác phẩm là DỮ LIỆU người dùng gõ, không một câu UI. -->
        <p class="orphan-name">{{ currentLibraryOrphan.name }}</p>
        <p class="orphan-path">{{ t('mode.library.orphan_path', { path: currentLibraryOrphan.atproj_path }) }}</p>
        <div class="orphan-actions">
          <!--
            🔵 THÊM (2026-08-27, vòng rà bốn lớp P9) — `aria-label` cho cả hai nút: `‹`/`›`
            trần không mang nghĩa gì cho trình đọc màn hình (NFR17 đòi mọi thao tác dùng
            được bằng bàn phím VÀ có nhãn rõ ràng). Khoá qua `t()`, không một chuỗi viết
            thẳng — cùng luật NFR16/AD-21.
          -->
          <button
            type="button"
            class="btn"
            :disabled="libraryOrphanCursor === 0"
            :aria-label="t('mode.library.orphan_prev')"
            @click="dispatch('library.orphan_prev')"
          >
            ‹
          </button>
          <!-- aura-allow-text: vị trí con trỏ là SỐ ĐẾM (dữ liệu), không một câu UI. -->
          <span class="orphan-position">{{ libraryOrphanCursor + 1 }} / {{ libraryOrphans.length }}</span>
          <button
            type="button"
            class="btn"
            :disabled="libraryOrphanCursor >= libraryOrphans.length - 1"
            :aria-label="t('mode.library.orphan_next')"
            @click="dispatch('library.orphan_next')"
          >
            ›
          </button>
          <button
            type="button"
            class="btn"
            :disabled="libraryRescanBusy"
            @click="dispatch('library.forget_orphan')"
          >
            {{ t('mode.library.forget_orphan') }}
          </button>
          <button
            type="button"
            class="btn"
            data-library-relocate-orphan
            :disabled="libraryRescanBusy"
            @click="dispatch('library.relocate_orphan')"
          >
            {{ t('mode.library.relocate_orphan') }}
          </button>
        </div>
      </div>
    </div>

    <div class="rebuild-block">
      <button
        type="button"
        class="btn"
        data-library-rebuild-index
        :disabled="libraryRescanBusy"
        @click="dispatch('library.rebuild_index')"
      >
        {{ t('mode.library.rebuild_index') }}
      </button>
      <p class="notice">{{ t('mode.library.rebuild_index_note') }}</p>
      <!-- aura-allow-text: mốc dựng là DỮ LIỆU; câu bao quanh đi qua t(). -->
      <p v-if="libraryRebuiltAt !== null" class="notice" data-library-rebuilt-at :data-rebuilt-at="libraryRebuiltAt">
        {{ t('mode.library.rebuilt_at', { when: libraryRebuiltAtLabel ?? '' }) }}
      </p>
    </div>

    <!-- aura-allow-text: mọi nhánh đều qua t()/tError(). -->
    <p class="error" role="status" data-library-open-work-error>
      {{
        libraryOpenWorkError
          ? tError(libraryOpenWorkError)
          : libraryOpenWorkNotice === 'flush-failed'
            ? t('mode.library.open_work_flush_failed')
            : libraryOpenWorkNotice === 'still-dirty'
              ? t('mode.library.open_work_still_dirty')
              : ''
      }}
    </p>
    </template>
  </section>
</template>

<style scoped>
.mode {
  height: 100%;
  padding: var(--space-panel-block) var(--space-panel-inline);
  overflow-y: auto;
  container-type: inline-size;
}

/*
 * ⚠️ `outline: none` CHỈ ở đây, và chỉ vì phần tử này mang `tabindex="-1"`: nó là một
 * vùng chứa nhận focus lập trình, không phải một điều khiển tương tác. Chỉ báo tiêu điểm
 * thật của sản phẩm là vạch dọc ở `PanelFrame` (AC5) — và, ở trong tệp này, vòng focus
 * của `.dropzone:focus-visible` bên dưới.
 *
 * Đừng nhân luật này ra thành `*:focus { outline: none }`. Đó là cách nhanh nhất phá
 * NFR17 (*"trạng thái focus luôn nhìn thấy rõ"*) mà vẫn qua được MỌI cổng hiện có —
 * `check-tokens.mjs` canh màu, cỡ chữ, tương phản, opacity và elevation, KHÔNG canh focus
 * ring. §Trap 4 của story 1.6.
 */
.mode:focus {
  outline: none;
}

.status {
  margin: 12px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.notice {
  margin: 4px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.images-failed-domain-log-view {
  padding: 0;
  margin-left: calc(var(--space-unit) * 1);
  background: none;
  border: none;
  cursor: pointer;
  font-family: inherit;
  font-size: inherit;
  color: inherit;
  text-decoration: underline;
}


.rule {
  width: 34px;
  height: 2px;
  background: var(--color-ornament);
  border-radius: 1px;
  margin-bottom: var(--space-unit);
}

.big {
  margin: 0 0 8px;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.small {
  margin: 0 0 16px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.field input,
.field select,
.field textarea {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 8px 8px;
}

.btn {
  align-self: flex-start;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-primary);
  background: var(--color-surface-sunken);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 8px 12px;
  cursor: pointer;
}

.btn:disabled {
  color: var(--color-on-surface-variant);
  cursor: not-allowed;
}

/*
 * Không còn luật `:focus-visible` cho `.dropzone` — nó không mang `tabindex` nữa nên
 * không nhận tiêu điểm được (xem comment trong `<template>`). Chỉ báo tiêu điểm thật của
 * màn này là chỉ báo gốc của trình duyệt trên `input`/`select`/`textarea`/`button`.
 *
 * Đừng nhân luật `outline: none` ở `.mode:focus` ra thành `*:focus` — đó là cách nhanh
 * nhất phá NFR17 mà vẫn qua được MỌI cổng hiện có. §Trap 4 của story 1.6.
 */

.error {
  margin: 12px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

/* Story 5.3 — "Quét lại thư mục". Cùng khuôn token/cỡ chữ với `.empty`/`.field` ở trên. */
.root-block {
  margin-bottom: var(--space-panel-block);
  padding-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

.section-heading {
  margin: 0 0 8px;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.root-value {
  margin: 0 0 12px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  word-break: break-all;
}

.root-missing {
  margin: 4px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.root-actions {
  display: flex;
  gap: 8px;
}

/**
 * 🔵 THÊM (2026-08-27, phán quyết Ice #3) — cùng khuôn `.root-missing` ngay trên: một cảnh
 * báo phát sinh TỪ một lượt quét (không phải một lỗi thao tác). Sống bên trong `.root-block`
 * (thừa `max-width` của khối cha) — chỉ khác `.root-missing` ở chỗ nó có thể mang HAI đường
 * dẫn dài trong cùng một câu, nên `word-break` được đặt tường minh tại đây thay vì dựa vào
 * kế thừa. Toàn bộ màu/cỡ chữ qua token — `check:tokens` cấm viết thẳng.
 */
.conflict-warning {
  margin: 8px 0 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
  word-break: break-all;
}

.orphan-row {
  margin-top: 12px;
  padding: 12px;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  background: var(--color-surface-sunken);
}

.orphan-name {
  margin: 0 0 4px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.orphan-path {
  margin: 0 0 8px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  word-break: break-all;
}

.orphan-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.orphan-position {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/* Story 5.4 — "Bốn trạng thái vòng đời". Cùng khuôn token/cỡ chữ với `.root-block` ở trên. */
.works-block {
  margin-bottom: var(--space-panel-block);
  padding-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

.filter-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}

/*
 * Nút lọc đang BẬT phân biệt bằng `aria-pressed` (đọc được bằng trình đọc màn hình) VÀ
 * bằng token màu — cùng khuôn `.dropzone.over` ở trên: không dùng bóng đổ/gradient (AD-21).
 */
.filter-actions .btn[aria-pressed='true'] {
  color: var(--color-primary);
  border-color: var(--color-primary);
}

/*
 * Story 5.6 — ba `<select>` lọc/sắp, cùng khuôn `.field` (đã khai ở trên cho form nhập) —
 * không đúc thêm một kiểu ô nhập thứ hai cho cùng một vai.
 */
.filter-selects {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 12px;
}

.filter-selects .field {
  min-width: 120px;
}

/* Story 5.6 — thanh điều hướng con trỏ lưới (AC7). Hai nút `‹`/`›` cộng vị trí "k / n". */
.grid-nav {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 12px 0 0;
}

.grid-nav-position {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/* Story 5.6 — lưới Tác phẩm, thay `.works-list` (danh sách phẳng) của Story 5.4. */
.works-grid {
  list-style: none;
  margin: 12px 0 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.work-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  background: var(--color-surface-sunken);
}

/*
 * Ô ĐANG CHỌN (AC7) — phân biệt bằng BỀ RỘNG viền (1px → 2px) CỘNG màu, không chỉ màu
 * (WCAG AA: một tín hiệu chỉ-màu không đủ cho người không phân biệt được màu sắc). Cùng
 * chủ ý `.filter-actions .btn[aria-pressed='true']` ở trên — `aria-current` đã tự nói phần
 * còn lại cho trình đọc màn hình.
 */
.work-cell--current {
  border-width: 2px;
  border-color: var(--color-primary);
}

/*
 * Khung bìa — biểu diễn thay thế NHẤT QUÁN (AC6): ô vuông, chữ cái đầu của tên trên nền
 * token. KHÔNG ảnh, KHÔNG gradient/bóng đổ (AD-21) — xem §Design Notes "Vì sao KHÔNG thêm
 * cột `cover`" của story: đây là TOÀN BỘ khung bìa hôm nay, không phải một trạng thái tạm.
 */
.work-cover {
  display: flex;
  align-items: center;
  justify-content: center;
  aspect-ratio: 1 / 1;
  border-radius: 4px;
  background: var(--color-surface-accent);
  color: var(--color-primary);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
}

.work-name {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
  word-break: break-all;
}

.work-status {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/*
 * Dấu ghi đè thủ công — chữ, không ký hiệu đúc mới (§Never của story: "đúc ký hiệu mới cho
 * dấu 'ghi đè thủ công' -- dấu phân biệt viết thành CHỮ qua t()").
 */
.override-marker {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-primary);
}

/* Story 5.5 — "Tiến độ Tác phẩm" (FR7). Chữ "k / n" hoặc "chưa biết", cùng cỡ `.work-status`. */
.work-progress {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/*
 * Thanh tiến độ — chỉ TOKEN, không màu viết thẳng/gradient/bóng đổ (§Always của story).
 * 🔵 SỬA (2026-08-28, Story 5.6) — gỡ `flex-basis: 100%`: `.work-cell` nay là
 * `flex-direction: column` (thay `.works-row` cũ, `flex-wrap: wrap` theo hàng ngang), nên
 * mọi con đã tự trải hết bề rộng theo mặc định `align-items: stretch` — không cần buộc nữa.
 */
.work-progress-track {
  height: 4px;
  border-radius: var(--radius-full);
  background: var(--color-outline);
  overflow: hidden;
}

.work-progress-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: var(--radius-full);
}

@container (min-width: 800px) {
  .works-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

@container (min-width: 1000px) {
  .works-grid {
    grid-template-columns: repeat(4, 1fr);
  }
}

.top-bar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: var(--space-panel-block);
}

.top-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
}

.search-field {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 200px;
  gap: 8px;
  padding: 0 8px;
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
}

.search-icon {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface-variant);
}

.search-input {
  flex: 1;
  min-width: 0;
  padding: 8px 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: none;
  border: none;
}

.search-field:focus-within {
  border-color: var(--color-primary);
}

.segmented {
  display: flex;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  overflow: hidden;
}

.segment {
  padding: 8px 12px;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface-variant);
  background: var(--color-surface-sunken);
  border: none;
  cursor: pointer;
}

.segment + .segment {
  border-left: 1px solid var(--color-outline);
}

.segment[aria-pressed='true'],
.segment--on {
  color: var(--color-primary);
  background: var(--color-surface-accent);
  font-weight: var(--weight-ui-md-strong);
}

.btn-primary {
  color: var(--color-on-primary);
  background: var(--color-primary);
  border-color: var(--color-primary);
}

.filter-actions,
.filter-selects {
  margin-bottom: 0;
}

.empty-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}

.work-cover {
  position: relative;
}

.work-lang {
  position: absolute;
  top: 4px;
  left: 4px;
  padding: 0 4px;
  border-radius: 2px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
  background: var(--color-surface);
}

.work-meta {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}
</style>
