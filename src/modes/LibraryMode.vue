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
  aimLibraryWork,
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
import LibraryUrlImportScreen from './LibraryUrlImportScreen.vue'
import ImportPreviewOverlay from '../ImportPreviewOverlay.vue'
import BilingualImportPreviewOverlay from '../BilingualImportPreviewOverlay.vue'
import { importPreviewIsOpen } from '../importPreviewState'
import { bilingualImportPreviewIsOpen } from '../bilingualImportPreviewState'
import { libraryUrlImportScreenIsOpen } from './libraryImportOverlayState'
// ── Story 5.9 — "Tìm kiếm full-text xuyên Library" (FR8) ─────────────────────────────
// 🔵 SỬA (retro Epic 5, AI-3 — 2026-09-03) — thêm `librarySearchCoverageGapState`, bề mặt ĐỘC
// LẬP với `librarySearchStatusKey` (§Always: "phải hiện cả khi có kết quả").
import LibrarySearchResults from './LibrarySearchResults.vue'
import {
  librarySearchBusy,
  librarySearchError,
  librarySearchFocusPending,
  librarySearchHasLoaded,
  librarySearchQuery,
  librarySearchScope,
  rerunLibrarySearch,
  setLibrarySearchScope,
  takeLibrarySearchFocus,
} from './librarySearch'
import type { SearchScope } from '../config/library'
import { t, tError } from '../i18n'
import { shortcutRows } from '../config/shortcutsState'
import { genreLabel, sourceLangLabel, workCardMeta, workCoverTone } from './libraryWorkCard'
import DropdownMenu from '../DropdownMenu.vue'
import SelectMenu from '../SelectMenu.vue'

const root = useTemplateRef<HTMLElement>('root')
const searchInput = useTemplateRef<HTMLInputElement>('searchInput')

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
  void nextTick(focusSearchOrCard)
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

function lifecycleLabel(status: string): string | null {
  if (status === 'not_started') return t('lifecycle.not_started')
  if (status === 'in_progress') return t('lifecycle.in_progress')
  if (status === 'paused') return t('lifecycle.paused')
  if (status === 'done') return t('lifecycle.done')
  return null
}

function statusText(work: WorkRow): string | null {
  if (work.status === null) return null
  return lifecycleLabel(work.status) ?? t('mode.library.works_status_invalid', { status: work.status })
}

function progressText(work: WorkRow): string {
  if (work.chapter_done_count === null) {
    return t('mode.library.works_chapter_count', { total: String(work.chapter_count) })
  }
  return t('mode.library.works_progress_short', {
    done: String(work.chapter_done_count),
    total: String(work.chapter_count),
  })
}

const statusChipLabel = computed(() => {
  const picked = Array.from(libraryStatusFilter.value)
  if (picked.length === 0) return t('mode.library.filter_status_label')
  if (picked.length === 1) return lifecycleLabel(picked[0]) ?? t('mode.library.filter_status_label')
  return t('mode.library.filter_status_many', { count: String(picked.length) })
})

const sortChipLabel = computed(() =>
  librarySortKey.value === 'name_asc' ? t('mode.library.sort_name_asc') : t('mode.library.sort_updated_desc'),
)

const genreOptions = computed(() => [
  { value: '', label: t('mode.library.filter_all_genres'), disabled: libraryWorksBusy.value },
  ...libraryGenres.value.map((genre) => ({ value: genre, label: genreLabel(genre) ?? genre, disabled: libraryWorksBusy.value })),
])
const sourceLangOptions = computed(() => [
  { value: '', label: t('mode.library.filter_all_source_langs'), disabled: libraryWorksBusy.value },
  ...librarySourceLangs.value.map((lang) => ({
    value: lang,
    label: sourceLangLabel(lang) ?? lang,
    disabled: libraryWorksBusy.value,
  })),
])
const sortOptions = computed(() => [
  { value: 'updated_desc', label: t('mode.library.sort_updated_desc'), disabled: libraryWorksBusy.value },
  { value: 'name_asc', label: t('mode.library.sort_name_asc'), disabled: libraryWorksBusy.value },
])

function onGenreChange(value: string | null): void {
  setGenreFilter(value)
}
function onSourceLangChange(value: string | null): void {
  setSourceLangFilter(value)
}
function onSortChange(value: WorkSortKey): void {
  setSortKey(value)
}

function aimWork(index: number, event: Event): void {
  aimLibraryWork(index)
  // WebKit does not focus a tabindex ancestor when the press lands on a non-focusable child.
  if (event.type !== 'mousedown' || !(event.currentTarget instanceof HTMLElement)) return
  if (event.target instanceof Element && event.target.closest('button, summary, a, input') !== null) return
  event.currentTarget.focus({ preventScroll: true })
}

// A v-for template ref array follows mount order, not list order, once a reload reorders the cards.
function cardAt(index: number): HTMLElement | null {
  return root.value?.querySelectorAll<HTMLElement>('[data-library-work-cell]')[index] ?? null
}

function currentCard(): HTMLElement | null {
  return cardAt(libraryWorkCursor.value)
}

function focusCurrentCard(): void {
  currentCard()?.focus({ preventScroll: true })
}

function focusRequestedSearch(): boolean {
  if (!takeLibrarySearchFocus() || searchInput.value === null) return false
  searchInput.value.focus()
  searchInput.value.select()
  return true
}

// The activation tick races the request's tick and must not take the caret back from the field.
function focusSearchOrCard(): void {
  if (focusRequestedSearch() || document.activeElement === searchInput.value) return
  focusCurrentCard()
}

watch(librarySearchFocusPending, (pending) => {
  if (pending) void nextTick(focusRequestedSearch)
})

function gridColumnCount(): number {
  const grid = root.value?.querySelector<HTMLElement>('[data-library-grid]')
  if (grid === null || grid === undefined) return 1
  const tracks = getComputedStyle(grid).gridTemplateColumns.split(' ').filter((track) => track !== '')
  return Math.max(1, tracks.length)
}

function onGridKeydown(event: KeyboardEvent): void {
  const target = event.target
  if (!(target instanceof HTMLElement) || !target.matches('[data-library-work-cell]')) return
  if (event.shiftKey || event.altKey || event.ctrlKey || event.metaKey) return
  if (event.key === 'Enter') {
    event.preventDefault()
    dispatch('library.open_work')
    return
  }
  const rowStep = event.key === 'ArrowDown' || event.key === 'ArrowUp' ? gridColumnCount() : 1
  if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
    event.preventDefault()
    for (let step = 0; step < rowStep; step += 1) dispatch('library.work_next')
  } else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
    event.preventDefault()
    for (let step = 0; step < rowStep; step += 1) dispatch('library.work_prev')
  } else {
    return
  }
  void nextTick(focusCurrentCard)
}

watch(
  libraryWorkCursor,
  () => {
    void nextTick(() => {
      const cell = currentCard()
      // `block: 'nearest'`/`inline: 'nearest'` — tức thì theo mặc định (không `behavior:
      // 'smooth'`), cùng chủ ý "không hiệu ứng" mà `GridPanel.vue` §AC8 đã ghi cho cuộn theo
      // con trỏ.
      cell?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
    })
  },
  { flush: 'post' },
)

// Closing the Work page returns focus to the selected card (UX-DR17), else to the mode root.
watch(libraryWorkPageIsOpen, (open, wasOpen) => {
  if (open || !wasOpen) return
  void nextTick(() => {
    const card = currentCard()
    if (card !== null) card.focus()
    else root.value?.focus()
  })
})
const importScreenIsOpen = computed<boolean>(
  () => libraryUrlImportScreenIsOpen.value || importPreviewIsOpen.value || bilingualImportPreviewIsOpen.value,
)
</script>

<template>
  <!-- `tabindex="-1"` để phần tử nhận được focus lập trình. Nó KHÔNG vào thứ tự Tab. -->
  <section ref="root" class="mode" tabindex="-1">
    <LibraryUrlImportScreen />
    <ImportPreviewOverlay />
    <BilingualImportPreviewOverlay />
    <LibraryWorkPage v-if="!importScreenIsOpen && libraryWorkPageIsOpen" />
    <template v-else-if="!importScreenIsOpen">
    <div class="top-bar">
      <div class="search-field">
        <!-- aura-allow-text: decorative glyph, hidden from assistive tech. -->
        <span class="search-icon" aria-hidden="true">⌕</span>
        <input
          ref="searchInput"
          v-model="librarySearchQuery"
          type="text"
          class="search-input ui-field ui-field--full"
          autocomplete="off"
          :placeholder="t('mode.library.search_placeholder')"
          :aria-label="t('mode.library.search_placeholder')"
          data-library-search-input
          @keyup.enter="dispatch('library.search')"
        />
      </div>
      <div
        v-if="librarySearchQuery.trim() !== ''"
        class="segmented ui-seg"
        role="radiogroup"
        :aria-label="t('mode.library.search_scope_heading')"
      >
        <label class="segment ui-seg__item">
          <input
            type="radio"
            class="lib-sr-only"
            name="library-search-scope"
            data-library-search-scope="both"
            :checked="librarySearchScope === 'both'"
            @change="onSearchScopeChange('both')"
          />
          {{ t('mode.library.search_scope_both') }}
        </label>
        <label class="segment ui-seg__item">
          <input
            type="radio"
            class="lib-sr-only"
            name="library-search-scope"
            data-library-search-scope="source"
            :checked="librarySearchScope === 'source'"
            @change="onSearchScopeChange('source')"
          />
          {{ t('mode.library.search_scope_source') }}
        </label>
        <label class="segment ui-seg__item">
          <input
            type="radio"
            class="lib-sr-only"
            name="library-search-scope"
            data-library-search-scope="target"
            :checked="librarySearchScope === 'target'"
            @change="onSearchScopeChange('target')"
          />
          {{ t('mode.library.search_scope_target') }}
        </label>
      </div>

      <div class="chips">
        <DropdownMenu
          variant="field"
          :label="statusChipLabel"
          :active="libraryStatusFilter.size > 0"
          data-library-chip="status"
        >
          <button
            type="button"
            class="menu-item ui-menu-item"
            data-lifecycle-filter="not_started"
            :aria-pressed="libraryStatusFilter.has('not_started')"
            :disabled="libraryWorksBusy"
            @click="dispatch('library.filter_not_started')"
          >
            {{ t('lifecycle.not_started') }}
          </button>
          <button
            type="button"
            class="menu-item ui-menu-item"
            data-lifecycle-filter="in_progress"
            :aria-pressed="libraryStatusFilter.has('in_progress')"
            :disabled="libraryWorksBusy"
            @click="dispatch('library.filter_in_progress')"
          >
            {{ t('lifecycle.in_progress') }}
          </button>
          <button
            type="button"
            class="menu-item ui-menu-item"
            data-lifecycle-filter="paused"
            :aria-pressed="libraryStatusFilter.has('paused')"
            :disabled="libraryWorksBusy"
            @click="dispatch('library.filter_paused')"
          >
            {{ t('lifecycle.paused') }}
          </button>
          <button
            type="button"
            class="menu-item ui-menu-item"
            data-lifecycle-filter="done"
            :aria-pressed="libraryStatusFilter.has('done')"
            :disabled="libraryWorksBusy"
            @click="dispatch('library.filter_done')"
          >
            {{ t('lifecycle.done') }}
          </button>
          <div class="ui-menu-sep" aria-hidden="true"></div>
          <button
            type="button"
            class="menu-item ui-menu-item"
            data-lifecycle-action="clear_filter"
            data-menu-close
            :disabled="libraryWorksBusy || libraryFilterIsEmpty"
            @click="dispatch('library.filter_clear')"
          >
            {{ t('mode.library.clear_filter') }}
          </button>
        </DropdownMenu>

        <SelectMenu
          variant="field"
          name="library-genre"
          option-attr="data-library-chip-option"
          :model-value="libraryGenreFilter ?? ''"
          :options="genreOptions"
          :label="t('mode.library.filter_genre_label')"
          :trigger-text="
            libraryGenreFilter === null
              ? t('mode.library.filter_genre_label')
              : (genreLabel(libraryGenreFilter) ?? libraryGenreFilter)
          "
          :active="libraryGenreFilter !== null"
          data-library-chip="genre"
          @change="onGenreChange($event === '' ? null : $event)"
        />

        <SelectMenu
          variant="field"
          name="library-source-lang"
          option-attr="data-library-chip-option"
          :model-value="librarySourceLangFilter ?? ''"
          :options="sourceLangOptions"
          :label="t('mode.library.filter_source_lang_label')"
          :trigger-text="
            librarySourceLangFilter === null
              ? t('mode.library.filter_source_lang_label')
              : (sourceLangLabel(librarySourceLangFilter) ?? librarySourceLangFilter)
          "
          :active="librarySourceLangFilter !== null"
          data-library-chip="source_lang"
          @change="onSourceLangChange($event === '' ? null : $event)"
        />

        <SelectMenu
          variant="field"
          name="library-sort"
          option-attr="data-library-chip-option"
          :model-value="librarySortKey"
          :options="sortOptions"
          :label="t('library.work_page.sort_label')"
          :trigger-text="sortChipLabel"
          data-library-chip="sort"
          @change="onSortChange($event as WorkSortKey)"
        />

        <!-- aura-allow-text: KẾT QUẢ của `t()` với hai số đếm. -->
        <span v-if="!libraryFilterIsEmpty && libraryWorksHaveLoaded" class="chips-count" data-library-filter-count>{{
          t('mode.library.filter_count', { matched: String(libraryWorksMatched), total: String(libraryWorksTotal) })
        }}</span>
      </div>
    </div>

    <LibrarySearchResults v-if="searchActive" />
    <template v-else>
    <div class="works-block">
      <!-- aura-allow-text: ba nhánh đều qua t()/chuỗi rỗng -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi. -->
      <p class="status" role="status">
        {{
          !libraryWorksHaveLoaded
            ? t('mode.library.works_loading')
            : libraryShowsFirstRunInvitation
              ? ''
              : libraryWorksTotal === 0
                ? t('mode.library.works_empty')
                : libraryWorksMatched === 0
                  ? t('mode.library.works_no_match', { total: String(libraryWorksTotal) })
                  : ''
        }}
      </p>
      <!-- aura-allow-text: như trên, qua tError(). -->
      <p class="error" role="status">{{ libraryWorksError ? tError(libraryWorksError) : '' }}</p>

      <div v-if="libraryShowsFirstRunInvitation" class="empty" data-library-empty>
        <div class="rule" aria-hidden="true"></div>
        <p class="big">{{ t('mode.library.status') }}</p>
        <p class="small">{{ t('mode.library.empty_body') }}</p>
        <div class="empty-actions">
          <button type="button" class="btn btn-primary ui-btn ui-btn--primary" data-library-empty-invite="paste" @click="dispatch('library.import.open')">
            {{ t('mode.library.empty_invite_paste') }}
          </button>
          <button type="button" class="btn ui-btn" data-library-empty-invite="file" @click="dispatch('library.import.open')">
            {{ t('mode.library.empty_invite_file') }}
          </button>
        </div>
        <!-- aura-allow-text: qua t(); phím tắt đã định dạng theo nền tảng. -->
        <p v-if="importChord !== ''" class="small">{{ t('mode.library.empty_shortcut_hint', { chord: importChord }) }}</p>
      </div>

      <ul v-if="libraryWorks.length > 0" class="works-grid" data-library-grid @keydown="onGridKeydown($event)">
        <li
          v-for="(work, workIndex) in libraryWorks"
          :key="work.work_id"
          class="work-cell"
          :class="{ 'work-cell--current': workIndex === libraryWorkCursor }"
          :tabindex="workIndex === libraryWorkCursor ? 0 : -1"
          :aria-current="workIndex === libraryWorkCursor ? 'true' : undefined"
          data-library-work-cell
          @mousedown="aimWork(workIndex, $event)"
          @focusin="aimWork(workIndex, $event)"
          @dblclick="dispatch('library.open_work')"
        >
          <div
            class="work-cover"
            :class="'work-cover--tone-' + workCoverTone(work.work_id)"
            data-library-work-cover
            :data-cover-tone="workCoverTone(work.work_id)"
          >
            <!-- aura-allow-text: tên Tác phẩm là DỮ LIỆU người dùng gõ, không một câu UI. -->
            <span class="work-name" :title="work.name" data-library-work-name>{{ work.name }}</span>
            <div class="work-go-slot">
              <button
                type="button"
                class="work-go ui-btn"
                :class="workIndex === libraryWorkCursor ? 'ui-btn--cover-on' : 'ui-btn--cover'"
                data-library-continue-translation
                :disabled="libraryOpenWorkBusy"
                @dblclick.stop
                @click="dispatch('library.continue_translation')"
              >
                <!-- aura-allow-text: cả hai nhánh đi qua t(). -->
                {{ work.status === 'not_started' ? t('mode.library.start_translation') : t('mode.library.continue_translation') }}
              </button>
            </div>
            <div class="work-more" @dblclick.stop>
              <DropdownMenu
                variant="more"
                trigger-class="ui-btn--cover-plain"
                label="⋯"
                :aria-label="t('mode.library.work_more_menu')"
                data-library-work-more
              >
                <button
                  type="button"
                  class="menu-item ui-menu-item"
                  data-library-open-work-folder
                  data-menu-close
                  @click="dispatch('library.open_work_folder')"
                >
                  {{ t('mode.library.open_work_folder_button') }}
                </button>
                <button
                  type="button"
                  class="menu-item ui-menu-item"
                  data-library-work-reviewer-import
                  data-reviewer-import-open
                  data-menu-close
                  :disabled="libraryOpenWorkBusy"
                  @click="dispatch('library.open_work_reviewer_import')"
                >
                  {{ t('command.export.reviewer_import.open') }}
                </button>
              </DropdownMenu>
            </div>
          </div>
          <div class="work-body">
            <!-- aura-allow-text: lĩnh vực là dữ liệu, nhãn thời gian qua t(). -->
            <span class="work-meta" data-library-work-meta>{{ workCards[workIndex].line ?? '' }}</span>
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
            <div class="work-stat">
              <!-- aura-allow-text: KẾT QUẢ của `t()` với số đếm. -->
              <span class="work-progress" data-library-work-progress>{{ progressText(work) }}</span>
              <span class="work-stat-end">
                <!-- aura-allow-text: KẾT QUẢ của `t()`; null (chưa biết) thì không có huy hiệu. -->
                <span
                  v-if="statusText(work) !== null"
                  class="work-status"
                  :class="'work-status--' + (work.status ?? 'unknown')"
                  data-library-work-status
                  >{{ statusText(work) }}</span
                >
                <span v-if="work.status_is_override" class="override-marker">{{ t('mode.library.override_marker') }}</span>
              </span>
            </div>
          </div>
        </li>
      </ul>
    </div>

    </template>

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
          class="btn ui-btn"
          :disabled="libraryRescanBusy"
          @click="dispatch('library.choose_root')"
        >
          {{ t('mode.library.choose_root') }}
        </button>
        <button
          type="button"
          class="btn ui-btn"
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
            class="btn ui-icon-btn"
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
            class="btn ui-icon-btn"
            :disabled="libraryOrphanCursor >= libraryOrphans.length - 1"
            :aria-label="t('mode.library.orphan_next')"
            @click="dispatch('library.orphan_next')"
          >
            ›
          </button>
          <button
            type="button"
            class="btn ui-btn"
            :disabled="libraryRescanBusy"
            @click="dispatch('library.forget_orphan')"
          >
            {{ t('mode.library.forget_orphan') }}
          </button>
          <button
            type="button"
            class="btn ui-btn"
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
        class="btn ui-btn"
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
  padding: var(--space-panel-block) 21px;
  overflow-y: auto;
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
  margin: 14px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.notice {
  margin: 7px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.rule {
  width: 35px;
  height: 2px;
  background: var(--color-ornament);
  border-radius: 1px;
  margin-bottom: var(--space-unit);
}

.big {
  margin: 0 0 7px;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.small {
  margin: 0 0 14px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.btn {
  align-self: flex-start;
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
  margin: 14px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

/* Story 5.3 — "Quét lại thư mục". Cùng khuôn token/cỡ chữ với `.empty`/`.field` ở trên. */
.root-block {
  margin-bottom: var(--space-panel-block);
  padding-bottom: 13px; /* aura-allow-spacing: 1px bottom border sits inside the 14px inset */
  border-bottom: 1px solid var(--color-outline);
}

.section-heading {
  margin: 0 0 7px;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.root-value {
  margin: 0 0 14px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  word-break: break-all;
}

.root-missing {
  margin: 7px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.root-actions {
  display: flex;
  gap: 7px;
}

/**
 * 🔵 THÊM (2026-08-27, phán quyết Ice #3) — cùng khuôn `.root-missing` ngay trên: một cảnh
 * báo phát sinh TỪ một lượt quét (không phải một lỗi thao tác). Sống bên trong `.root-block`
 * (thừa `max-width` của khối cha) — chỉ khác `.root-missing` ở chỗ nó có thể mang HAI đường
 * dẫn dài trong cùng một câu, nên `word-break` được đặt tường minh tại đây thay vì dựa vào
 * kế thừa. Toàn bộ màu/cỡ chữ qua token — `check:tokens` cấm viết thẳng.
 */
.conflict-warning {
  margin: 7px 0 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
  word-break: break-all;
}

.orphan-row {
  margin-top: 14px;
  padding: 13px; /* aura-allow-spacing: 1px border sits inside the 14px inset */
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  background: var(--color-surface-sunken);
}

.orphan-name {
  margin: 0 0 7px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.orphan-path {
  margin: 0 0 7px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  word-break: break-all;
}

.orphan-actions {
  display: flex;
  align-items: baseline;
  height: 28px;
  gap: 7px;
}

.orphan-position {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.works-block .status:empty,
.works-block .error:empty {
  margin: 0;
  min-height: 0;
}

.works-block {
  margin-bottom: var(--space-panel-block);
  padding-bottom: 13px; /* aura-allow-spacing: 1px bottom border sits inside the 14px inset */
  border-bottom: 1px solid var(--color-outline);
}

.works-grid {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(238px, 1fr));
  gap: 21px;
}

.work-cell {
  display: flex;
  flex-direction: column;
  position: relative;
  border: 1px solid var(--color-outline-faint);
  border-radius: 4px;
  background: var(--color-surface);
}

.work-cell:not(.work-cell--current):focus-visible {
  outline: none;
  border-color: var(--color-primary);
}

.work-cell:hover {
  border-color: var(--color-outline);
}

/* The selected card is told apart by a 2px primary ring, not by colour alone. */
.work-cell--current {
  border-color: var(--color-primary);
  outline: 1px solid var(--color-primary);
}

.work-cover {
  position: relative;
  display: flex;
  flex-wrap: wrap;
  align-content: space-between;
  align-items: flex-end;
  justify-content: space-between;
  height: 133px;
  padding: 13px 13px 14px; /* aura-allow-spacing: 1px card border sits inside the 14px inset */
  border-radius: 3px 3px 0 0;
  background: var(--color-cover-1-from);
}

.work-cover--tone-1 {
  background: var(--color-cover-1-from);
}
.work-cover--tone-2 {
  background: var(--color-cover-2-from);
}
.work-cover--tone-3 {
  background: var(--color-cover-3-from);
}
.work-cover--tone-4 {
  background: var(--color-cover-4-from);
}
.work-cover--tone-5 {
  background: var(--color-cover-5-from);
}
.work-cover--tone-6 {
  background: var(--color-cover-6-from);
}

.work-go-slot,
.work-more {
  display: none;
}

.work-cell--current .work-go-slot,
.work-cell--current .work-more,
.work-cell:hover .work-go-slot,
.work-cell:hover .work-more,
.work-cell:focus-within .work-go-slot,
.work-cell:focus-within .work-more {
  display: block;
}

.work-more {
  margin-left: auto;
}

.work-body {
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding: 13px; /* aura-allow-spacing: 1px card border sits inside the 14px inset */
}

.work-name {
  flex: 0 0 100%;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 3;
  overflow: hidden;
  font-family: var(--face-read-sm);
  font-size: var(--font-read-sm);
  line-height: var(--leading-read-sm);
  color: var(--color-on-cover);
  overflow-wrap: anywhere;
}

.work-meta {
  margin-bottom: 7px;
  min-height: 21px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.work-progress-track {
  margin-bottom: -3px; /* aura-allow-spacing: the 3px bar fills a 7px band, so the row below stays on the grid */
  height: 3px;
  border-radius: 2px;
  background: var(--color-surface-sunken);
  overflow: hidden;
}

.work-progress-fill {
  height: 100%;
  background: var(--color-confirmed);
}

.work-stat {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 7px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.work-stat-end {
  display: flex;
  align-items: baseline;
  gap: 7px;
}

.work-status {
  padding: 0 7px;
  border-radius: 2px;
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface-variant);
  background: var(--color-surface-sunken);
}

.work-status--in_progress {
  color: var(--color-primary);
  background: var(--color-surface-accent);
}

.work-status--done {
  color: var(--color-confirmed);
  background: var(--color-surface-sunken);
}

.work-status--paused {
  color: var(--color-tm-text);
  background: var(--color-surface-tm);
}

.work-status--not_started {
  font-weight: normal;
}

.override-marker {
  color: var(--color-primary);
}

.menu-item[aria-pressed='true'] {
  background: var(--color-surface-accent);
}

.top-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 14px;
  padding-bottom: 14px;
  margin-bottom: 21px;
  border-bottom: 1px solid var(--color-outline);
}

.chips {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 7px;
  margin-left: auto;
}

.chips-count {
  padding: 0 7px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.search-field {
  position: relative;
  display: flex;
  align-items: baseline;
  flex: 1;
  max-width: 420px;
  min-width: 203px;
}

.search-icon {
  position: absolute;
  left: 7px;
  pointer-events: none;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface-variant);
}

.search-input {
  flex: 1;
  min-width: 0;
  padding-left: 28px;
}

.segment {
  position: relative;
}

.lib-sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  margin: -1px; /* aura-allow-spacing: sr-only geometry, not grid spacing */
  padding: 0;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.empty-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 7px;
  margin-bottom: 14px;
}
</style>
