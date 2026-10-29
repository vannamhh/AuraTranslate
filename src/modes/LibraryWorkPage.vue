<script setup lang="ts">
// Every @click is one dispatch('<id>'); row selection uses @mousedown, which Kiểm A does not read.
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'
import { dispatch } from '../commands'
import ChapterOrigin from '../ChapterOrigin.vue'
import { t, tError } from '../i18n'
import type { ChapterRow } from '../config/chapter'
import {
  chapterRenameDraft,
  chapterWindow,
  libraryChapterCursor,
  libraryChapterCursorFlatIndex,
  libraryChapterGroupCounts,
  libraryChapterOriginApplyThroughOrd,
  libraryChapterOriginBusy,
  libraryChapterOriginError,
  libraryChapterRenameEditing,
  libraryChapterReorgBusy,
  libraryChapterReorgError,
  libraryChapterReorgNotice,
  libraryChapterRules,
  libraryChapterSelectedRows,
  libraryChapterView,
  libraryChapters,
  libraryChaptersBusy,
  libraryChaptersError,
  libraryChaptersHaveLoaded,
  libraryOpenWorkBusy,
  libraryWorkPageWork,
  moveLibraryChapterSelection,
  saveCurrentChapterOrigin,
  selectLibraryChapter,
  setChapterOriginApplyThroughOrd,
  soleSelectedLibraryChapter,
  toggleLibraryChapterAtCursor,
} from './libraryChapters'
import {
  NOT_STARTED_PREVIEW,
  libraryChapterFilter,
  libraryChapterNotStartedExpanded,
  libraryChapterSearch,
  libraryChapterSelectedIds,
  libraryChapterSort,
  libraryChapterStatusMenuOpen,
  setChapterSort,
} from './libraryChapterSelection'
import type { ChapterGroupKey, ChapterSortKey, ChapterViewItem } from './libraryChapterSelection'
import { chapterTitleLabel, lifecycleStatusLabel } from './libraryChapterLabels'
import {
  openWorkLifecycleBusy,
  openWorkLifecycleDerivedStatus,
  openWorkLifecycleError,
  openWorkLifecycleIsOverride,
  openWorkLifecycleLoaded,
  openWorkLifecycleStatus,
} from './libraryWorks'
import { updatedLabel, workCardMeta } from './libraryWorkCard'
import SelectMenu from '../SelectMenu.vue'

const root = useTemplateRef<HTMLElement>('root')
const listRef = useTemplateRef<HTMLUListElement>('listRef')
const renameInput = useTemplateRef<HTMLInputElement>('renameInput')

// One fixed height for every flat item (group header, Chapter row, "more" note), so the pure
// `chapterWindow` can virtualise the list. Must match `.wp-item` in <style>.
const ITEM_HEIGHT_PX = 44
const VIEWPORT_HEIGHT_PX = 440
const OVERSCAN = 4

const scrollTop = ref(0)

const items = computed(() => libraryChapterView.value.items)
const slice = computed(() =>
  chapterWindow(scrollTop.value, VIEWPORT_HEIGHT_PX, ITEM_HEIGHT_PX, items.value.length, OVERSCAN),
)
const windowItems = computed(() => items.value.slice(slice.value.start, slice.value.end))

// The virtual window drops the header of the group the top row belongs to; keep it as the first
// element so `position: sticky` has something to pin. Its own flow slot is taken from the top pad.
const stickyHead = computed(() => {
  const start = slice.value.start
  if (start === 0 || items.value[start]?.kind === 'group') return null
  for (let i = start - 1; i >= 0; i--) {
    const item = items.value[i]
    if (item.kind === 'group') return item
  }
  return null
})
const visibleItems = computed(() =>
  stickyHead.value === null ? windowItems.value : [stickyHead.value, ...windowItems.value],
)
const padTopPx = computed(() =>
  stickyHead.value === null ? slice.value.padTop : Math.max(0, slice.value.padTop - ITEM_HEIGHT_PX),
)

const NARROW_PX = 1100
const narrow = ref(false)
const panelDismissed = ref(false)
const hoveredChapterId = ref<number | null>(null)
let sizeObserver: ResizeObserver | null = null

const work = computed(() => libraryWorkPageWork.value)
const total = computed(() => libraryChapters.value.length)
const counts = computed(() => libraryChapterGroupCounts.value)
const selectedCount = computed(() => libraryChapterSelectedIds.value.size)
const panelVisible = computed(() => selectedCount.value > 0 && !(narrow.value && panelDismissed.value))
const sole = computed(() => soleSelectedLibraryChapter.value)
const batchSummary = computed(() => {
  const rows = libraryChapterSelectedRows.value
  const first = rows[0]
  const last = rows[rows.length - 1]
  if (rows.length < 2) return null
  return {
    from: first.ord,
    to: last.ord,
    sentences: rows.reduce((sum, row) => sum + row.in_translation_count, 0),
  }
})
const originApplyEffect = computed(() => {
  const row = sole.value
  const through = libraryChapterOriginApplyThroughOrd.value
  if (row === null || through === null) return null
  const count = libraryChapters.value.filter((chapter) => chapter.ord >= row.ord && chapter.ord <= through).length
  return { from: row.ord, to: through, count }
})
const writesBlocked = computed(
  () => libraryChapterReorgBusy.value || libraryChaptersBusy.value || libraryOpenWorkBusy.value,
)
const foldToggleVisible = computed(
  () =>
    libraryChapterFilter.value === 'all' &&
    libraryChapterSearch.value.trim() === '' &&
    counts.value.not_started > NOT_STARTED_PREVIEW,
)

const meta = computed(() => (work.value === null ? null : workCardMeta(work.value, new Date())))
const subtitle = computed(() => {
  const parts: string[] = []
  if (meta.value?.langLabel != null) parts.push(meta.value.langLabel)
  if (meta.value?.line != null) parts.push(meta.value.line)
  parts.push(t('library.work_page.sub_chapters', { count: String(total.value) }))
  return parts.join(' · ')
})

function groupLabel(key: ChapterGroupKey): string {
  return key === 'unknown' ? t('library.work_page.group_unknown') : lifecycleStatusLabel(key)
}

function progressPercent(row: ChapterRow): number {
  if (row.in_translation_count === 0) return 0
  return Math.min(100, Math.round((row.translated_count / row.in_translation_count) * 100))
}

function showsProgress(row: ChapterRow): boolean {
  return (row.status === 'in_progress' || row.status === 'paused') && row.in_translation_count > 0
}

function segmentText(row: ChapterRow): string {
  if (showsProgress(row)) {
    return t('library.work_page.row_progress', {
      done: String(row.translated_count),
      total: String(row.in_translation_count),
    })
  }
  return t('library.work_page.row_sentences', { count: String(row.in_translation_count) })
}

function whenText(row: ChapterRow): string {
  if (row.status === 'not_started') return t('library.work_page.row_no_time')
  return updatedLabel(row.updated_at, new Date()) ?? t('library.work_page.row_no_time')
}

function itemKey(item: ChapterViewItem): string {
  if (item.kind === 'group') return `g-${item.key}`
  if (item.kind === 'chapter') return `c-${item.row.chapter_id}`
  return 'more'
}

const KNOWN_STATUSES: readonly string[] = ['in_progress', 'paused', 'done', 'not_started']

function hasKnownStatus(row: ChapterRow): boolean {
  return KNOWN_STATUSES.includes(row.status)
}

function rowButtonTier(row: ChapterRow, kind: 'read' | 'translate'): string {
  if (isSelected(row) && (row.status === 'done') === (kind === 'read')) return 'ui-btn--primary'
  return hoveredChapterId.value === row.chapter_id || isSelected(row) ? '' : 'ui-btn--quiet'
}

function isSelected(row: ChapterRow): boolean {
  return libraryChapterSelectedIds.value.has(row.chapter_id)
}

function onScroll(event: Event): void {
  scrollTop.value = (event.target as HTMLElement).scrollTop
}

function onRowMouseDown(row: ChapterRow, event: MouseEvent): void {
  if (event.button !== 0) return
  panelDismissed.value = false
  if (event.shiftKey) event.preventDefault()
  selectLibraryChapter(row.chapter_id, { toggle: event.metaKey || event.ctrlKey, range: event.shiftKey })
  listRef.value?.focus()
}

function onListKeydown(event: KeyboardEvent): void {
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    moveLibraryChapterSelection(event.key === 'ArrowDown' ? 1 : -1, event.shiftKey)
    return
  }
  if (event.key === ' ') {
    event.preventDefault()
    toggleLibraryChapterAtCursor()
    return
  }
  if (event.key === 'Enter') {
    if (!libraryChapterRules.value.open) return
    event.preventDefault()
    dispatch('library.open_chapter')
    return
  }
  if (event.key === 'F2') {
    if (!libraryChapterRules.value.rename) return
    event.preventDefault()
    dispatch('library.chapter_rename.begin')
    return
  }
  if (event.key === 'Escape' && selectedCount.value > 0) {
    event.preventDefault()
    event.stopPropagation()
    if (narrow.value && !panelDismissed.value) {
      panelDismissed.value = true
      return
    }
    dispatch('library.chapters.clear_selection')
  }
}

// WKWebView does not focus a <button> on mouse click; without this an overlay saves the
// previously focused node as its opener and UX-DR17 returns focus there instead of to the button.
function focusOnPointerDown(event: MouseEvent): void {
  const target = event.currentTarget
  if (target instanceof HTMLElement) target.focus()
}

const chapterSortOptions = computed(() => [
  { value: 'order', label: t('library.work_page.sort_order') },
  { value: 'recent', label: t('library.work_page.sort_recent') },
])

function onSortChange(value: string): void {
  setChapterSort(value as ChapterSortKey)
}

function syncScroll(next: number): void {
  scrollTop.value = Math.max(0, next)
  const el = listRef.value
  if (el !== null && el.scrollTop !== scrollTop.value) el.scrollTop = scrollTop.value
}

watch(libraryChapterCursor, () => {
  const index = libraryChapterCursorFlatIndex()
  if (index === -1) return
  const { start, end } = slice.value
  if (index < start + 1) syncScroll((index - 1) * ITEM_HEIGHT_PX)
  else if (index >= end - 1) syncScroll((index + 1) * ITEM_HEIGHT_PX - VIEWPORT_HEIGHT_PX)
})

watch([libraryChapterFilter, libraryChapterSearch, libraryChapterSort], () => syncScroll(0))

watch(libraryChapterSelectedIds, () => {
  panelDismissed.value = false
})

watch(libraryChapterRenameEditing, (editing) => {
  if (!editing) return
  void nextTick(() => renameInput.value?.focus())
})

onMounted(() => {
  root.value?.focus()
  const el = root.value
  if (el === null || typeof ResizeObserver === 'undefined') return
  narrow.value = el.clientWidth > 0 && el.clientWidth < NARROW_PX
  sizeObserver = new ResizeObserver(() => {
    narrow.value = el.clientWidth > 0 && el.clientWidth < NARROW_PX
  })
  sizeObserver.observe(el)
})

onBeforeUnmount(() => {
  sizeObserver?.disconnect()
  sizeObserver = null
})
</script>

<template>
  <section ref="root" class="wp" tabindex="-1" data-library-work-page>
    <nav class="wp-crumb" :aria-label="t('command.library.work_page.close')">
      <button
        type="button"
        class="ui-btn ui-btn--quiet ui-btn--sm"
        data-library-work-page-close
        :aria-label="t('command.library.work_page.close')"
        @click="dispatch('library.work_page.close')"
      >
        {{ t('library.work_page.crumb_root') }}
      </button>
      <template v-if="work !== null">
        <span class="wp-crumb-sep" aria-hidden="true">/</span>
        <!-- aura-allow-text: tên Tác phẩm là dữ liệu người dùng. -->
        <span class="wp-crumb-name">{{ work.name }}</span>
      </template>
    </nav>

    <div v-if="work !== null" class="wp-head" data-library-work-header>
      <div class="wp-cover" role="img" :aria-label="t('mode.library.cover_placeholder_label', { name: work.name })">
        <!-- aura-allow-text: biểu diễn thay thế là chữ cái đầu của TÊN, dữ liệu người dùng. -->
        <span>{{ work.name.trim().charAt(0).toUpperCase() || '?' }}</span>
      </div>
      <div class="wp-meta">
        <!-- aura-allow-text: tên Tác phẩm là dữ liệu người dùng. -->
        <h2 class="wp-title" data-library-work-title>{{ work.name }}</h2>
        <!-- aura-allow-text: ghép từ t() và dữ liệu. -->
        <p class="wp-sub" data-library-work-sub>{{ subtitle }}</p>

        <div class="wp-progress" data-library-work-progress>
          <div class="wp-track" role="img" :aria-label="t('library.work_page.progress_aria')">
            <i v-if="total > 0" class="wp-fill wp-fill--done" :style="{ width: (counts.done / total) * 100 + '%' }"></i>
            <i
              v-if="total > 0"
              class="wp-fill wp-fill--doing"
              :style="{ width: (counts.in_progress / total) * 100 + '%' }"
            ></i>
            <i v-if="total > 0" class="wp-fill wp-fill--pause" :style="{ width: (counts.paused / total) * 100 + '%' }"></i>
          </div>
          <div class="wp-legend">
            <!-- aura-allow-text: ghép từ t() với số đếm. -->
            <span class="wp-legend-item wp-legend-item--done">{{
              t('library.work_page.legend', { label: t('lifecycle.done'), count: String(counts.done) })
            }}</span>
            <!-- aura-allow-text: ghép từ t() với số đếm. -->
            <span class="wp-legend-item wp-legend-item--doing">{{
              t('library.work_page.legend', { label: t('lifecycle.in_progress'), count: String(counts.in_progress) })
            }}</span>
            <!-- aura-allow-text: ghép từ t() với số đếm. -->
            <span class="wp-legend-item wp-legend-item--pause">{{
              t('library.work_page.legend', { label: t('lifecycle.paused'), count: String(counts.paused) })
            }}</span>
            <!-- aura-allow-text: ghép từ t() với số đếm. -->
            <span class="wp-legend-item">{{
              t('library.work_page.legend', { label: t('lifecycle.not_started'), count: String(counts.not_started) })
            }}</span>
          </div>
        </div>
      </div>

      <div class="wp-status" :class="{ 'wp-status--override': openWorkLifecycleIsOverride }" data-library-work-status-cell>
        <p class="wp-status-label">{{ t('library.work_page.status_label') }}</p>
        <!-- aura-allow-text: mọi nhánh đều qua t()/lifecycleStatusLabel(). -->
        <p v-if="!openWorkLifecycleLoaded" class="wp-status-value" role="status">
          {{ openWorkLifecycleError ? t('mode.library.open_work_lifecycle_unreadable') : t('mode.library.open_work_not_loaded') }}
        </p>
        <template v-else>
          <p class="wp-status-value" role="status">
            <!-- aura-allow-text: nhãn trạng thái qua t(). -->
            <span class="wp-badge" data-library-work-status>{{
              openWorkLifecycleStatus === null
                ? t('mode.library.works_status_unknown')
                : lifecycleStatusLabel(openWorkLifecycleStatus)
            }}</span>
            <!-- aura-allow-text: cả hai nhánh đều qua t(). -->
            <span class="wp-status-kind" data-library-work-status-kind>{{
              openWorkLifecycleIsOverride ? t('library.work_page.status_override') : t('library.work_page.status_derived')
            }}</span>
          </p>
          <!-- aura-allow-text: ghép từ t() với nhãn trạng thái suy ra. -->
          <p class="wp-status-why" data-library-work-status-why>
            {{
              openWorkLifecycleIsOverride
                ? t('library.work_page.status_override_why', {
                    derived:
                      openWorkLifecycleDerivedStatus === null
                        ? t('mode.library.works_status_unknown')
                        : lifecycleStatusLabel(openWorkLifecycleDerivedStatus),
                  })
                : t('library.work_page.status_derived_why')
            }}
          </p>
        </template>
        <!-- aura-allow-text: kết quả của tError(). -->
        <p class="error" role="status">{{ openWorkLifecycleError ? tError(openWorkLifecycleError) : '' }}</p>
        <div class="wp-status-actions">
          <button
            type="button"
            class="ui-btn ui-btn--sm"
            data-lifecycle-action="set_override_paused"
            :disabled="openWorkLifecycleBusy || !openWorkLifecycleLoaded"
            @click="dispatch('lifecycle.set_work_override_paused')"
          >
            {{ t('mode.library.set_override_paused') }}
          </button>
          <button
            v-if="openWorkLifecycleIsOverride"
            type="button"
            class="ui-btn ui-btn--sm"
            data-lifecycle-action="clear_override"
            :disabled="openWorkLifecycleBusy || !openWorkLifecycleLoaded"
            @click="dispatch('lifecycle.clear_work_override')"
          >
            {{ t('mode.library.clear_override') }}
          </button>
        </div>
      </div>
    </div>

    <div class="wp-bar">
      <input
        v-model="libraryChapterSearch"
        type="text"
        class="wp-search ui-field ui-field--sm ui-field--on-sunken"
        autocomplete="off"
        :placeholder="t('library.work_page.search_placeholder')"
        :aria-label="t('library.work_page.search_placeholder')"
        data-library-chapter-search
      />
      <div class="ui-seg ui-seg--sm" role="group" :aria-label="t('library.work_page.filter_aria')">
        <button
          type="button"
          class="ui-seg__item"
          data-library-chapter-filter="all"
          :aria-pressed="libraryChapterFilter === 'all'"
          @click="dispatch('library.chapter_filter.all')"
        >
          <!-- aura-allow-text: nhãn nhóm qua t()/lifecycleStatusLabel(). -->
          {{ t('library.work_page.filter_all') }}
          <!-- aura-allow-text: số Chương của nhóm là dữ liệu đếm. -->
          <em class="wp-seg-count">{{ total }}</em>
        </button>
        <button
          type="button"
          class="ui-seg__item"
          data-library-chapter-filter="in_progress"
          :aria-pressed="libraryChapterFilter === 'in_progress'"
          @click="dispatch('library.chapter_filter.in_progress')"
        >
          <!-- aura-allow-text: nhãn nhóm qua t()/lifecycleStatusLabel(). -->
          {{ groupLabel('in_progress') }}
          <!-- aura-allow-text: số Chương của nhóm là dữ liệu đếm. -->
          <em class="wp-seg-count">{{ counts.in_progress }}</em>
        </button>
        <button
          type="button"
          class="ui-seg__item"
          data-library-chapter-filter="paused"
          :aria-pressed="libraryChapterFilter === 'paused'"
          @click="dispatch('library.chapter_filter.paused')"
        >
          <!-- aura-allow-text: nhãn nhóm qua t()/lifecycleStatusLabel(). -->
          {{ groupLabel('paused') }}
          <!-- aura-allow-text: số Chương của nhóm là dữ liệu đếm. -->
          <em class="wp-seg-count">{{ counts.paused }}</em>
        </button>
        <button
          type="button"
          class="ui-seg__item"
          data-library-chapter-filter="done"
          :aria-pressed="libraryChapterFilter === 'done'"
          @click="dispatch('library.chapter_filter.done')"
        >
          <!-- aura-allow-text: nhãn nhóm qua t()/lifecycleStatusLabel(). -->
          {{ groupLabel('done') }}
          <!-- aura-allow-text: số Chương của nhóm là dữ liệu đếm. -->
          <em class="wp-seg-count">{{ counts.done }}</em>
        </button>
        <button
          type="button"
          class="ui-seg__item"
          data-library-chapter-filter="not_started"
          :aria-pressed="libraryChapterFilter === 'not_started'"
          @click="dispatch('library.chapter_filter.not_started')"
        >
          <!-- aura-allow-text: nhãn nhóm qua t()/lifecycleStatusLabel(). -->
          {{ groupLabel('not_started') }}
          <!-- aura-allow-text: số Chương của nhóm là dữ liệu đếm. -->
          <em class="wp-seg-count">{{ counts.not_started }}</em>
        </button>
      </div>
      <div class="wp-sort">
        <span>{{ t('library.work_page.sort_label') }}</span>
        <SelectMenu
          variant="chip"
          :model-value="libraryChapterSort"
          :options="chapterSortOptions"
          :label="t('library.work_page.sort_label')"
          data-library-chapter-sort
          @change="onSortChange"
        />
      </div>
    </div>

    <div v-if="libraryChapterRenameEditing" class="wp-rename" data-library-chapter-rename-bar>
      <label class="wp-rename-field">
        <span>{{ t('mode.library.chapter_rename_label') }}</span>
        <input
          ref="renameInput"
          v-model="chapterRenameDraft"
          type="text"
          class="ui-field ui-field--medium"
          autocomplete="off"
          data-library-chapter-rename-input
          @keydown.enter="dispatch('library.chapter_rename')"
          @keydown.esc.stop="dispatch('library.chapter_rename.cancel')"
        />
      </label>
      <button
        type="button"
        class="btn btn-primary ui-btn ui-btn--primary"
        data-library-chapter-rename
        :disabled="writesBlocked || soleSelectedLibraryChapter === null"
        @click="dispatch('library.chapter_rename')"
      >
        {{ t('mode.library.chapter_rename_button') }}
      </button>
      <button type="button" class="btn ui-btn" data-library-chapter-rename-cancel @click="dispatch('library.chapter_rename.cancel')">
        {{ t('command.library.chapter_rename.cancel') }}
      </button>
    </div>

    <!-- aura-allow-text: kết quả của tError(). -->
    <p class="error" role="status">{{ libraryChaptersError ? tError(libraryChaptersError) : '' }}</p>
    <!-- aura-allow-text: mọi nhánh đều qua t()/tError(). -->
    <p class="error" role="status" data-library-chapter-reorg-error>
      {{
        libraryChapterReorgError
          ? tError(libraryChapterReorgError)
          : libraryChapterReorgNotice === 'flush-failed'
            ? t('mode.library.chapter_reorg_flush_failed')
            : libraryChapterReorgNotice === 'still-dirty'
              ? t('mode.library.chapter_reorg_still_dirty')
              : ''
      }}
    </p>

    <div class="wp-body" :class="{ 'wp-body--with-panel': panelVisible }">
      <div class="chapters-block">
        <!--
          role="status" is always present: an empty list must say why it is empty (not loaded yet,
          loaded and empty, loaded but filtered out).
          aura-allow-text: mọi nhánh đều qua t().
        -->
        <p class="status" role="status" data-library-chapters-status>
          {{
            !libraryChaptersHaveLoaded
              ? t('mode.library.chapters_not_loaded')
              : total === 0
                ? t('mode.library.chapters_empty')
                : libraryChapterView.matched === 0
                  ? t('library.work_page.no_match')
                  : libraryChapterView.matched === total
                    ? t('mode.library.chapters_result', { total: String(total) })
                    : t('library.work_page.result_filtered', {
                        matched: String(libraryChapterView.matched),
                        total: String(total),
                      })
          }}
        </p>

        <ul
          v-if="items.length > 0"
          ref="listRef"
          class="chapters-list"
          tabindex="0"
          role="listbox"
          aria-multiselectable="true"
          data-library-chapters-list
          @scroll="onScroll($event)"
          @keydown="onListKeydown($event)"
        >
          <li class="chapters-pad" :style="{ height: padTopPx + 'px' }" aria-hidden="true"></li>
          <template v-for="item in visibleItems" :key="itemKey(item)">
            <li v-if="item.kind === 'group'" class="wp-item wp-group" :data-library-chapter-group="item.key" role="presentation">
              <!-- aura-allow-text: nhãn nhóm qua t(). -->
              <span class="wp-group-label">{{ groupLabel(item.key) }}</span>
              <!-- aura-allow-text: ghép từ t() với số đếm. -->
              <span class="wp-group-count">{{ t('library.work_page.group_count', { count: String(item.total) }) }}</span>
              <button
                v-if="item.key === 'not_started' && foldToggleVisible"
                type="button"
                class="wp-group-toggle ui-btn ui-btn--quiet ui-btn--sm"
                data-library-chapter-group-toggle
                :aria-expanded="libraryChapterNotStartedExpanded"
                @click="dispatch('library.chapter_group.toggle_not_started')"
              >
                {{ t('command.library.chapter_group.toggle_not_started') }}
              </button>
            </li>
            <li
              v-else-if="item.kind === 'chapter'"
              class="wp-item chapter-row"
              :class="[
                'chapter-row--' + item.row.status,
                { 'chapter-row--selected': isSelected(item.row), 'chapter-row--current': item.row.chapter_id === libraryChapters.at(libraryChapterCursor)?.chapter_id && selectedCount > 0 },
              ]"
              role="option"
              :aria-selected="isSelected(item.row)"
              data-library-chapter-row
              :data-chapter-id="item.row.chapter_id"
              :data-chapter-status="item.row.status"
              @mousedown="onRowMouseDown(item.row, $event)"
              @mouseenter="hoveredChapterId = item.row.chapter_id"
              @mouseleave="hoveredChapterId = null"
              @dblclick="dispatch('library.open_chapter')"
            >
              <!-- aura-allow-text: thứ tự Chương là SỐ ĐẾM (dữ liệu), không một câu UI. -->
              <span class="chapter-ord">{{ item.row.ord }}</span>
              <span class="chapter-name">
                <!-- aura-allow-text: tiêu đề Chương là DỮ LIỆU người dùng; nhãn dự phòng qua t(). -->
                <span class="chapter-title">{{ chapterTitleLabel(item.row) }}</span>
                <span v-if="showsProgress(item.row)" class="chapter-mini" aria-hidden="true">
                  <i :style="{ width: progressPercent(item.row) + '%' }"></i>
                </span>
              </span>
              <span class="chapter-seg">
                <!-- aura-allow-text: ghép từ t() với số đếm. -->
                <span class="chapter-segment-count">{{ segmentText(item.row) }}</span>
                <!-- aura-allow-text: ghép từ t() với số đếm do Rust trả về. -->
                <span
                  v-if="item.row.status === 'done' && item.row.unconfirmed_count > 0"
                  class="chapter-unconfirmed"
                  data-library-chapter-unconfirmed
                  >{{ ' · ' + t('library.work_page.row_unconfirmed', { count: String(item.row.unconfirmed_count) }) }}</span
                >
                <!-- aura-allow-text: ghép từ t() với số đếm do Rust trả về. -->
                <span
                  v-if="item.row.unlinked_count > 0"
                  class="chapter-unlinked"
                  data-library-chapter-unlinked
                  >{{ ' · ' + t('library.work_page.row_unlinked', { count: String(item.row.unlinked_count) }) }}</span
                >
              </span>
              <!-- aura-allow-text: nhãn trạng thái qua t(). -->
              <span class="chapter-status chapter-chip" :class="'chapter-chip--' + item.row.status">{{
                lifecycleStatusLabel(item.row.status)
              }}</span>
              <!-- aura-allow-text: nhãn thời gian qua t() hoặc dấu gạch. -->
              <span class="chapter-when">{{ whenText(item.row) }}</span>
              <span class="chapter-act" @dblclick.stop>
                <button
                  type="button"
                  class="ui-btn ui-btn--sm"
                  :class="rowButtonTier(item.row, 'translate')"
                  data-library-row-translate
                  tabindex="-1"
                  :aria-label="t('library.work_page.row_translate_aria', { ord: String(item.row.ord) })"
                  :disabled="writesBlocked"
                  @click="dispatch('library.open_chapter')"
                >
                  {{ t('library.work_page.row_translate') }}
                </button>
                <button
                  v-if="item.row.status === 'done'"
                  type="button"
                  class="ui-btn ui-btn--sm"
                  :class="rowButtonTier(item.row, 'read')"
                  data-library-row-read
                  tabindex="-1"
                  :aria-label="t('library.work_page.row_read_aria', { ord: String(item.row.ord) })"
                  :disabled="writesBlocked"
                  @click="dispatch('library.open_chapter_reading')"
                >
                  {{ t('library.work_page.row_read') }}
                </button>
                <span v-else class="chapter-act-slot" aria-hidden="true"></span>
              </span>
            </li>
            <li v-else class="wp-item wp-more" data-library-chapter-more>
              <!-- aura-allow-text: ghép từ t() với số đếm. -->
              {{ t('library.work_page.more_hidden', { count: String(item.hidden) }) }}
            </li>
          </template>
          <li class="chapters-pad" :style="{ height: slice.padBottom + 'px' }" aria-hidden="true"></li>
        </ul>

        <div class="wp-foot">
          <!-- aura-allow-text: mọi nhánh đều qua t(). -->
          <span class="wp-sel-count" data-library-chapter-selected-count>{{
            selectedCount === 0
              ? t('library.work_page.selection_none')
              : t('library.work_page.selection_count', { count: String(selectedCount) })
          }}</span>
          <span class="wp-sel-hint">{{ t('library.work_page.selection_hint') }}</span>
        </div>
      </div>

      <aside v-if="panelVisible" class="wp-panel" :aria-label="t('library.work_page.panel_arrange')">
        <div v-if="sole !== null" class="wp-card" data-library-chapter-panel>
          <div>
            <!-- aura-allow-text: ghép từ t() với dữ liệu Chương. -->
            <h3 class="wp-card-title">{{ t('library.work_page.panel_title', { ord: String(sole.ord), title: chapterTitleLabel(sole) }) }}</h3>
            <p class="wp-card-meta">
              <!-- aura-allow-text: nhãn trạng thái qua t(). -->
              <span class="chapter-chip" :class="'chapter-chip--' + sole.status">{{ lifecycleStatusLabel(sole.status) }}</span>
              <!-- aura-allow-text: ghép từ t() với số đếm. -->
              <span>{{ segmentText(sole) + ' · ' + whenText(sole) }}</span>
            </p>
          </div>

          <div class="wp-go">
            <button
              type="button"
              class="ui-btn ui-btn--lg"
              :class="{ 'ui-btn--primary': sole.status !== 'done' }"
              data-library-open-chapter
              :disabled="!libraryChapterRules.open || writesBlocked"
              @click="dispatch('library.open_chapter')"
            >
              {{ t('library.work_page.panel_translate') }}
            </button>
            <button
              v-if="hasKnownStatus(sole)"
              type="button"
              class="ui-btn ui-btn--lg"
              :class="{ 'ui-btn--primary': sole.status === 'done' }"
              data-library-panel-read
              :disabled="!libraryChapterRules.read || writesBlocked"
              @click="dispatch('library.open_chapter_reading')"
            >
              {{ t('library.work_page.panel_read') }}
            </button>
          </div>
          <p v-if="hasKnownStatus(sole) && sole.status !== 'done'" class="wp-note" data-library-panel-read-locked>
            {{ t('library.work_page.panel_read_locked') }}
          </p>

          <div class="wp-sect">
            <p class="wp-sect-title">{{ t('library.work_page.panel_arrange') }}</p>
            <div class="wp-grid2">
              <button
                type="button"
                class="ui-btn ui-btn--sm"
                data-library-chapter-rename-begin
                :disabled="!libraryChapterRules.rename || writesBlocked"
                @click="dispatch('library.chapter_rename.begin')"
              >
                {{ t('command.library.chapter_rename.begin') }}
              </button>
              <button
                type="button"
                class="ui-btn ui-btn--sm"
                data-library-chapter-split-open
                :disabled="!libraryChapterRules.split || writesBlocked"
                @mousedown="focusOnPointerDown($event)"
                @click="dispatch('library.chapter_split.open')"
              >
                {{ t('command.library.chapter_split.open') }}
              </button>
              <button
                type="button"
                class="ui-btn ui-btn--sm"
                data-library-chapter-move-up
                :disabled="!libraryChapterRules.moveUp || writesBlocked"
                @click="dispatch('library.chapter_move_up')"
              >
                {{ t('mode.library.chapter_move_up') }}
              </button>
              <button
                type="button"
                class="ui-btn ui-btn--sm"
                data-library-chapter-move-down
                :disabled="!libraryChapterRules.moveDown || writesBlocked"
                @click="dispatch('library.chapter_move_down')"
              >
                {{ t('mode.library.chapter_move_down') }}
              </button>
              <button
                type="button"
                class="ui-btn ui-btn--sm"
                data-library-chapters-status-menu
                :aria-expanded="libraryChapterStatusMenuOpen"
                :disabled="!libraryChapterRules.status || writesBlocked"
                @click="dispatch('library.chapters.status_menu')"
              >
                {{ t('command.library.chapters.status_menu') }}
              </button>
              <button
                type="button"
                class="ui-btn ui-btn--sm"
                data-export-open
                :disabled="!libraryChapterRules.export"
                @mousedown="focusOnPointerDown($event)"
                @click="dispatch('export.open')"
              >
                {{ t('library.work_page.export') }}
              </button>
              <button
                type="button"
                class="ui-btn ui-btn--sm"
                data-library-chapters-reviewer-import
                data-reviewer-import-open
                @mousedown="focusOnPointerDown($event)"
                @click="dispatch('export.reviewer_import.open')"
              >
                {{ t('command.export.reviewer_import.open') }}
              </button>
            </div>
          </div>

          <div v-if="libraryChapterStatusMenuOpen" class="wp-status-menu" data-library-chapters-status-menu-list>
            <button
              type="button"
              class="ui-btn ui-btn--sm"
              data-library-chapters-status-set="in_progress"
              :disabled="writesBlocked"
              @click="dispatch('library.chapters.status_set.in_progress')"
            >
              {{ t('lifecycle.in_progress') }}
            </button>
            <button
              type="button"
              class="ui-btn ui-btn--sm"
              data-library-chapters-status-set="paused"
              :disabled="writesBlocked"
              @click="dispatch('library.chapters.status_set.paused')"
            >
              {{ t('lifecycle.paused') }}
            </button>
            <button
              type="button"
              class="ui-btn ui-btn--sm"
              data-library-chapters-status-set="done"
              :disabled="writesBlocked"
              @click="dispatch('library.chapters.status_set.done')"
            >
              {{ t('lifecycle.done') }}
            </button>
            <button
              type="button"
              class="ui-btn ui-btn--sm"
              data-library-chapters-status-set="not_started"
              :disabled="writesBlocked"
              @click="dispatch('library.chapters.status_set.not_started')"
            >
              {{ t('lifecycle.not_started') }}
            </button>
          </div>

          <div class="wp-sect">
            <ChapterOrigin
              :author="sole.origin_author"
              :site-name="sole.origin_site_name"
              :url="sole.origin_url"
              :published-at="sole.origin_published_at"
              :disabled="libraryChapterOriginBusy"
              class="chapter-origin-block"
              @commit="(edit) => saveCurrentChapterOrigin(edit)"
            />
            <label class="field chapter-origin-apply-through">
              <span>{{ t('mode.library.chapter_origin_apply_through_label') }}</span>
              <input
                type="number"
                class="ui-field ui-field--short"
                :value="libraryChapterOriginApplyThroughOrd ?? ''"
                :disabled="libraryChapterOriginBusy"
                data-library-chapter-origin-apply-through
                @change="
                  (e) => {
                    const input = e.target as HTMLInputElement
                    setChapterOriginApplyThroughOrd(input.value)
                    input.value = String(libraryChapterOriginApplyThroughOrd ?? '')
                  }
                "
              />
            </label>
            <!-- aura-allow-text: ghép từ t() với số đếm. -->
            <p v-if="originApplyEffect !== null" class="wp-note" data-library-chapter-origin-apply-effect>
              {{
                t('library.work_page.origin_apply_effect', {
                  from: String(originApplyEffect.from),
                  to: String(originApplyEffect.to),
                  count: String(originApplyEffect.count),
                })
              }}
            </p>
            <p v-if="libraryChapterOriginError !== null" class="error" role="alert">
              <!-- aura-allow-text: KẾT QUẢ của tError(). -->
              {{ tError(libraryChapterOriginError) }}
            </p>
          </div>
        </div>

        <div v-else-if="batchSummary !== null" class="wp-card" data-library-chapter-panel data-library-chapter-panel-batch>
          <div>
            <!-- aura-allow-text: ghép từ t() với số đếm. -->
            <h3 class="wp-card-title">{{ t('library.work_page.selection_count', { count: String(selectedCount) }) }}</h3>
            <!-- aura-allow-text: ghép từ t() với số đếm. -->
            <p class="wp-card-meta">
              {{
                t('library.work_page.batch_range', {
                  from: String(batchSummary.from),
                  to: String(batchSummary.to),
                  count: String(batchSummary.sentences),
                })
              }}
            </p>
          </div>
          <button
            type="button"
            class="ui-btn ui-btn--lg ui-btn--primary"
            data-library-chapter-merge-open
            :disabled="!libraryChapterRules.merge || writesBlocked"
            @mousedown="focusOnPointerDown($event)"
            @click="dispatch('library.chapter_merge.open')"
          >
            {{ t('command.library.chapter_merge.open') }}
          </button>
          <div class="wp-grid2">
            <button
              type="button"
              class="ui-btn ui-btn--sm"
              data-library-chapters-status-menu
              :aria-expanded="libraryChapterStatusMenuOpen"
              :disabled="!libraryChapterRules.status || writesBlocked"
              @click="dispatch('library.chapters.status_menu')"
            >
              {{ t('command.library.chapters.status_menu') }}
            </button>
            <button
              type="button"
              class="ui-btn ui-btn--sm"
              data-export-open
              :disabled="!libraryChapterRules.export"
              @mousedown="focusOnPointerDown($event)"
              @click="dispatch('export.open')"
            >
              {{ t('library.work_page.export') }}
            </button>
          </div>
          <div v-if="libraryChapterStatusMenuOpen" class="wp-status-menu" data-library-chapters-status-menu-list>
            <button
              type="button"
              class="ui-btn ui-btn--sm"
              data-library-chapters-status-set="in_progress"
              :disabled="writesBlocked"
              @click="dispatch('library.chapters.status_set.in_progress')"
            >
              {{ t('lifecycle.in_progress') }}
            </button>
            <button
              type="button"
              class="ui-btn ui-btn--sm"
              data-library-chapters-status-set="paused"
              :disabled="writesBlocked"
              @click="dispatch('library.chapters.status_set.paused')"
            >
              {{ t('lifecycle.paused') }}
            </button>
            <button
              type="button"
              class="ui-btn ui-btn--sm"
              data-library-chapters-status-set="done"
              :disabled="writesBlocked"
              @click="dispatch('library.chapters.status_set.done')"
            >
              {{ t('lifecycle.done') }}
            </button>
            <button
              type="button"
              class="ui-btn ui-btn--sm"
              data-library-chapters-status-set="not_started"
              :disabled="writesBlocked"
              @click="dispatch('library.chapters.status_set.not_started')"
            >
              {{ t('lifecycle.not_started') }}
            </button>
          </div>
        </div>
      </aside>
    </div>
  </section>
</template>

<style scoped>
.wp {
  container: wp / inline-size;
  display: flex;
  flex-direction: column;
  gap: var(--space-panel-block);
  min-height: 100%;
}

.wp:focus {
  /* aura-allow-outline-none: tabindex=-1 root, focused programmatically, not an interactive control. */
  outline: none;
}

.status {
  margin: 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.error {
  margin: 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
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

.wp-crumb {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-crumb-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.wp-head {
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr) 300px;
  gap: calc(var(--space-unit) * 5);
  align-items: start;
  padding-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

.wp-cover {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 96px;
  border-radius: 4px;
  background: var(--color-surface-accent);
  color: var(--color-primary);
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-ui-md-strong);
}

.wp-meta {
  min-width: 0;
}

.wp-title {
  margin: 0 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.wp-sub {
  margin: 0 0 calc(var(--space-unit) * 3) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-progress {
  max-width: 560px;
}

.wp-track {
  display: flex;
  height: 6px;
  overflow: hidden;
  border-radius: 3px;
  background: var(--color-surface-sunken);
}

.wp-fill {
  display: block;
  height: 100%;
}

.wp-fill--done {
  background: var(--color-confirmed);
}

.wp-fill--doing {
  background: var(--color-primary);
}

.wp-fill--pause {
  background: var(--color-tm-rule);
}

.wp-legend {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 4);
  margin-top: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-status {
  padding-left: calc(var(--space-unit) * 4);
  border-left: 1px solid var(--color-outline);
}

.wp-status-label {
  margin: 0 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.wp-status-value {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.wp-badge {
  padding: 0 calc(var(--space-unit) * 2);
  border-radius: 2px;
  background: var(--color-surface-accent);
  color: var(--color-primary);
  font-weight: var(--weight-ui-md-strong);
}

.wp-status--override .wp-badge {
  background: var(--color-surface-tm);
  color: var(--color-tm-text);
}

.wp-status-kind {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-status-why {
  margin: calc(var(--space-unit) * 1) 0 0 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-status-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: calc(var(--space-unit) * 2);
}

.wp-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 2) 0;
  border-bottom: 1px solid var(--color-outline);
}

.wp-search {
  min-width: 200px;
}

.wp-seg-count {
  margin-left: calc(var(--space-unit) * 1);
  font-style: normal;
  font-size: var(--font-ui-sm);
}

.wp-sort {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  margin-left: auto;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-foot {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 3);
}

.wp-sel-count {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-sel-hint {
  margin-left: auto;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-status-menu {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
}

.wp-rename {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: 8px;
}

.wp-rename-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-body {
  position: relative;
  display: grid;
  grid-template-columns: minmax(0, 1fr);
  min-height: 520px;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  background: var(--color-surface);
}

.wp-body--with-panel {
  grid-template-columns: minmax(0, 1fr) 320px;
}

.chapters-block {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  padding: calc(var(--space-unit) * 2) 0 calc(var(--space-unit) * 2) 0;
}

.chapters-block > .status,
.chapters-block > .wp-foot {
  padding: 0 calc(var(--space-unit) * 4);
}

/* `max-height` must match VIEWPORT_HEIGHT_PX: `@scroll` only fires when the element scrolls. */
.chapters-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 440px;
  overflow-y: auto;
  background: var(--color-surface);
  border-top: 1px solid var(--color-outline-faint);
}

.chapters-list:focus-visible {
  outline: none;
  border-color: var(--color-primary);
}

.chapters-pad {
  margin: 0;
  padding: 0;
  list-style: none;
}

.wp-item {
  box-sizing: border-box;
  height: 44px;
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 3);
  padding: 0 calc(var(--space-unit) * 4);
  border-bottom: 1px solid var(--color-outline-faint);
}

/* A positioned sticky header paints above the non-positioned rows without a z-index. */
.wp-group {
  position: sticky;
  top: 0;
  gap: calc(var(--space-unit) * 2);
  background: var(--color-background);
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.wp-group-label {
  font-weight: var(--weight-ui-md-strong);
}

.wp-group-count {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  text-transform: none;
}

.wp-group-toggle {
  margin-left: auto;
}

.wp-more {
  justify-content: center;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.chapter-row {
  display: grid;
  grid-template-columns: 44px minmax(0, 1fr) 170px 96px 84px 108px;
  gap: 0;
  cursor: default;
  user-select: none;
  border-left: 3px solid transparent;
  padding-left: calc(var(--space-unit) * 3);
}

.chapter-row--in_progress {
  border-left-color: var(--color-primary);
}

.chapter-row--paused {
  border-left-color: var(--color-tm-rule);
}

.chapter-row--done {
  border-left-color: var(--color-confirmed);
}

.chapter-row--selected {
  background: var(--color-surface-accent);
  border-left-color: var(--color-caret-rule);
}

.chapter-row--current {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.chapter-ord {
  padding-right: calc(var(--space-unit) * 3);
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
  text-align: right;
}

.chapter-name {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 3);
}

.chapter-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--face-read-md);
  font-size: var(--font-read-md);
  color: var(--color-on-surface);
}

.chapter-mini {
  display: block;
  flex: none;
  width: 48px;
  height: 3px;
  overflow: hidden;
  border-radius: 2px;
  background: var(--color-surface-sunken);
}

.chapter-mini i {
  display: block;
  height: 100%;
  background: var(--color-primary);
}

.chapter-seg,
.chapter-when {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
  white-space: nowrap;
}

.chapter-unconfirmed {
  color: var(--color-tm-text);
}

.chapter-unlinked {
  color: var(--color-tm-text);
}

.chapter-chip {
  padding: 0 calc(var(--space-unit) * 2);
  border-radius: 2px;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  background: var(--color-surface-sunken);
  color: var(--color-on-surface-variant);
  white-space: nowrap;
}

.chapter-status {
  justify-self: start;
}

.chapter-chip--in_progress {
  background: var(--color-surface-accent);
  color: var(--color-primary);
}

.chapter-chip--paused {
  background: var(--color-surface-tm);
  color: var(--color-tm-text);
}

.chapter-chip--done {
  background: var(--color-surface-sunken);
  color: var(--color-confirmed);
}

.chapter-act {
  display: grid;
  grid-template-columns: 54px 48px;
  gap: 4px;
  justify-content: end;
}

.chapter-act .ui-btn {
  justify-content: center;
  padding: 0;
}

.wp-panel {
  border-left: 1px solid var(--color-outline);
  background: var(--color-surface);
  min-width: 0;
}

.wp-card {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 4);
  padding: calc(var(--space-unit) * 4) calc(var(--space-unit) * 5);
}

.wp-card-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-md);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.wp-card-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  margin: calc(var(--space-unit) * 1) 0 0 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-go {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}

.wp-go .ui-btn {
  flex: 1;
  justify-content: center;
}

.wp-note {
  margin: 0;
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border-radius: 3px;
  background: var(--color-surface-sunken);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-sect {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
  padding-top: calc(var(--space-unit) * 3);
  border-top: 1px solid var(--color-outline);
}

.wp-sect-title {
  margin: 0;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.wp-grid2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: calc(var(--space-unit) * 2);
}

.wp-grid2 .ui-btn {
  justify-content: flex-start;
}

.chapter-origin-block {
  margin-top: 0;
}

@container wp (max-width: 1099px) {
  .wp-body {
    grid-template-columns: minmax(0, 1fr);
  }

  .wp-panel {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: 320px;
    overflow-y: auto;
  }

  .chapter-row {
    grid-template-columns: 44px minmax(0, 1fr) 170px 96px 108px;
  }

  .chapter-when {
    display: none;
  }
}

@container wp (max-width: 899px) {
  .chapter-row {
    grid-template-columns: 44px minmax(0, 1fr) 96px 108px;
  }

  .chapter-seg {
    display: none;
  }
}
</style>
