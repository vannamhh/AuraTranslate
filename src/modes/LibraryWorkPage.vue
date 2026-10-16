<script setup lang="ts">
// Work page: header, status cell, filter strip, grouped Chapter list, action bar. Replaces the
// Works grid after "Mở Tác phẩm". Counts and statuses come from Rust; this file only renders.
// Every @click is one dispatch('<id>'); row selection uses @mousedown, which Kiểm A does not read.
import { computed, nextTick, onMounted, ref, useTemplateRef, watch } from 'vue'
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
const visibleItems = computed(() => items.value.slice(slice.value.start, slice.value.end))

const work = computed(() => libraryWorkPageWork.value)
const total = computed(() => libraryChapters.value.length)
const counts = computed(() => libraryChapterGroupCounts.value)
const selectedCount = computed(() => libraryChapterSelectedIds.value.size)
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

function isSelected(row: ChapterRow): boolean {
  return libraryChapterSelectedIds.value.has(row.chapter_id)
}

function onScroll(event: Event): void {
  scrollTop.value = (event.target as HTMLElement).scrollTop
}

function onRowMouseDown(row: ChapterRow, event: MouseEvent): void {
  if (event.button !== 0) return
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
    dispatch('library.chapters.clear_selection')
  }
}

// WKWebView does not focus a <button> on mouse click; without this an overlay saves the
// previously focused node as its opener and UX-DR17 returns focus there instead of to the button.
function focusOnPointerDown(event: MouseEvent): void {
  const target = event.currentTarget
  if (target instanceof HTMLElement) target.focus()
}

function onSortChange(event: Event): void {
  setChapterSort((event.target as HTMLSelectElement).value as ChapterSortKey)
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
  if (index < start + 1) syncScroll(index * ITEM_HEIGHT_PX)
  else if (index >= end - 1) syncScroll((index + 1) * ITEM_HEIGHT_PX - VIEWPORT_HEIGHT_PX)
})

watch([libraryChapterFilter, libraryChapterSearch, libraryChapterSort], () => syncScroll(0))

watch(libraryChapterRenameEditing, (editing) => {
  if (!editing) return
  void nextTick(() => renameInput.value?.focus())
})

onMounted(() => {
  root.value?.focus()
})
</script>

<template>
  <section ref="root" class="wp" tabindex="-1" data-library-work-page>
    <div class="wp-back">
      <button type="button" class="btn" data-library-work-page-close @click="dispatch('library.work_page.close')">
        {{ t('command.library.work_page.close') }}
      </button>
    </div>

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

        <div class="wp-progress-row">
          <div class="wp-progress" data-library-work-progress>
            <div class="wp-track" role="img" :aria-label="t('library.work_page.progress_aria')">
              <i
                v-if="total > 0"
                class="wp-fill wp-fill--done"
                :style="{ width: (counts.done / total) * 100 + '%' }"
              ></i>
              <i
                v-if="total > 0"
                class="wp-fill wp-fill--doing"
                :style="{ width: (counts.in_progress / total) * 100 + '%' }"
              ></i>
              <i
                v-if="total > 0"
                class="wp-fill wp-fill--pause"
                :style="{ width: (counts.paused / total) * 100 + '%' }"
              ></i>
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
                class="btn"
                data-lifecycle-action="set_override_paused"
                :disabled="openWorkLifecycleBusy || !openWorkLifecycleLoaded"
                @click="dispatch('lifecycle.set_work_override_paused')"
              >
                {{ t('mode.library.set_override_paused') }}
              </button>
              <button
                type="button"
                class="btn"
                data-lifecycle-action="clear_override"
                :disabled="openWorkLifecycleBusy || !openWorkLifecycleLoaded || !openWorkLifecycleIsOverride"
                @click="dispatch('lifecycle.clear_work_override')"
              >
                {{ t('mode.library.clear_override') }}
              </button>
              <button
                type="button"
                class="btn"
                data-lifecycle-action="set_chapter_done"
                :disabled="openWorkLifecycleBusy || !openWorkLifecycleLoaded"
                @click="dispatch('lifecycle.set_chapter_done')"
              >
                {{ t('mode.library.set_chapter_done') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="wp-bar">
      <input
        v-model="libraryChapterSearch"
        type="text"
        class="wp-search"
        autocomplete="off"
        :placeholder="t('library.work_page.search_placeholder')"
        :aria-label="t('library.work_page.search_placeholder')"
        data-library-chapter-search
      />
      <button
        type="button"
        class="wp-pill"
        data-library-chapter-filter="all"
        :aria-pressed="libraryChapterFilter === 'all'"
        @click="dispatch('library.chapter_filter.all')"
      >
        <!-- aura-allow-text: ghép từ t() với số đếm. -->
        {{ t('library.work_page.filter_all', { count: String(total) }) }}
      </button>
      <button
        type="button"
        class="wp-pill"
        data-library-chapter-filter="in_progress"
        :aria-pressed="libraryChapterFilter === 'in_progress'"
        @click="dispatch('library.chapter_filter.in_progress')"
      >
        <!-- aura-allow-text: ghép từ t() với số đếm. -->
        {{ t('library.work_page.legend', { label: groupLabel('in_progress'), count: String(counts.in_progress) }) }}
      </button>
      <button
        type="button"
        class="wp-pill"
        data-library-chapter-filter="paused"
        :aria-pressed="libraryChapterFilter === 'paused'"
        @click="dispatch('library.chapter_filter.paused')"
      >
        <!-- aura-allow-text: ghép từ t() với số đếm. -->
        {{ t('library.work_page.legend', { label: groupLabel('paused'), count: String(counts.paused) }) }}
      </button>
      <button
        type="button"
        class="wp-pill"
        data-library-chapter-filter="done"
        :aria-pressed="libraryChapterFilter === 'done'"
        @click="dispatch('library.chapter_filter.done')"
      >
        <!-- aura-allow-text: ghép từ t() với số đếm. -->
        {{ t('library.work_page.legend', { label: groupLabel('done'), count: String(counts.done) }) }}
      </button>
      <button
        type="button"
        class="wp-pill"
        data-library-chapter-filter="not_started"
        :aria-pressed="libraryChapterFilter === 'not_started'"
        @click="dispatch('library.chapter_filter.not_started')"
      >
        <!-- aura-allow-text: ghép từ t() với số đếm. -->
        {{ t('library.work_page.legend', { label: groupLabel('not_started'), count: String(counts.not_started) }) }}
      </button>
      <label class="wp-sort">
        <span>{{ t('library.work_page.sort_label') }}</span>
        <select :value="libraryChapterSort" data-library-chapter-sort @change="onSortChange($event)">
          <option value="order">{{ t('library.work_page.sort_order') }}</option>
          <option value="recent">{{ t('library.work_page.sort_recent') }}</option>
        </select>
      </label>
    </div>

    <div class="wp-selbar" data-library-chapter-selbar>
      <!-- aura-allow-text: mọi nhánh đều qua t(). -->
      <span class="wp-sel-count" data-library-chapter-selected-count>{{
        selectedCount === 0
          ? t('library.work_page.selection_none')
          : t('library.work_page.selection_count', { count: String(selectedCount) })
      }}</span>
      <button
        type="button"
        class="btn"
        data-library-open-chapter
        :disabled="!libraryChapterRules.open || writesBlocked"
        @click="dispatch('library.open_chapter')"
      >
        {{ t('mode.library.open_chapter_button') }}
      </button>
      <button
        type="button"
        class="btn btn-primary"
        data-library-chapter-merge-open
        :disabled="!libraryChapterRules.merge || writesBlocked"
        @mousedown="focusOnPointerDown($event)"
        @click="dispatch('library.chapter_merge.open')"
      >
        {{ t('command.library.chapter_merge.open') }}
      </button>
      <button
        type="button"
        class="btn"
        data-library-chapter-split-open
        :disabled="!libraryChapterRules.split || writesBlocked"
        @mousedown="focusOnPointerDown($event)"
        @click="dispatch('library.chapter_split.open')"
      >
        {{ t('command.library.chapter_split.open') }}
      </button>
      <button
        type="button"
        class="btn"
        data-library-chapter-rename-begin
        :disabled="!libraryChapterRules.rename || writesBlocked"
        @click="dispatch('library.chapter_rename.begin')"
      >
        {{ t('command.library.chapter_rename.begin') }}
      </button>
      <button
        type="button"
        class="btn"
        data-library-chapter-move-up
        :disabled="!libraryChapterRules.moveUp || writesBlocked"
        @click="dispatch('library.chapter_move_up')"
      >
        {{ t('mode.library.chapter_move_up') }}
      </button>
      <button
        type="button"
        class="btn"
        data-library-chapter-move-down
        :disabled="!libraryChapterRules.moveDown || writesBlocked"
        @click="dispatch('library.chapter_move_down')"
      >
        {{ t('mode.library.chapter_move_down') }}
      </button>
      <button
        type="button"
        class="btn"
        data-library-chapters-status-menu
        :aria-expanded="libraryChapterStatusMenuOpen"
        :disabled="!libraryChapterRules.status || writesBlocked"
        @click="dispatch('library.chapters.status_menu')"
      >
        {{ t('command.library.chapters.status_menu') }}
      </button>
      <button
        type="button"
        class="btn"
        data-export-open
        :disabled="!libraryChapterRules.export"
        @mousedown="focusOnPointerDown($event)"
        @click="dispatch('export.open')"
      >
        {{ t('library.work_page.export') }}
      </button>
      <button
        type="button"
        class="btn"
        data-library-chapters-reviewer-import
        data-reviewer-import-open
        @mousedown="focusOnPointerDown($event)"
        @click="dispatch('export.reviewer_import.open')"
      >
        {{ t('command.export.reviewer_import.open') }}
      </button>
      <span class="wp-sel-hint">{{ t('library.work_page.selection_hint') }}</span>
    </div>

    <div v-if="libraryChapterStatusMenuOpen" class="wp-status-menu" data-library-chapters-status-menu-list>
      <button
        type="button"
        class="btn"
        data-library-chapters-status-set="in_progress"
        :disabled="writesBlocked"
        @click="dispatch('library.chapters.status_set.in_progress')"
      >
        {{ t('lifecycle.in_progress') }}
      </button>
      <button
        type="button"
        class="btn"
        data-library-chapters-status-set="paused"
        :disabled="writesBlocked"
        @click="dispatch('library.chapters.status_set.paused')"
      >
        {{ t('lifecycle.paused') }}
      </button>
      <button
        type="button"
        class="btn"
        data-library-chapters-status-set="done"
        :disabled="writesBlocked"
        @click="dispatch('library.chapters.status_set.done')"
      >
        {{ t('lifecycle.done') }}
      </button>
      <button
        type="button"
        class="btn"
        data-library-chapters-status-set="not_started"
        :disabled="writesBlocked"
        @click="dispatch('library.chapters.status_set.not_started')"
      >
        {{ t('lifecycle.not_started') }}
      </button>
    </div>

    <div v-if="libraryChapterRenameEditing" class="wp-rename" data-library-chapter-rename-bar>
      <label class="wp-rename-field">
        <span>{{ t('mode.library.chapter_rename_label') }}</span>
        <input
          ref="renameInput"
          v-model="chapterRenameDraft"
          type="text"
          autocomplete="off"
          data-library-chapter-rename-input
          @keydown.enter="dispatch('library.chapter_rename')"
          @keydown.esc.stop="dispatch('library.chapter_rename.cancel')"
        />
      </label>
      <button
        type="button"
        class="btn btn-primary"
        data-library-chapter-rename
        :disabled="writesBlocked || soleSelectedLibraryChapter === null"
        @click="dispatch('library.chapter_rename')"
      >
        {{ t('mode.library.chapter_rename_button') }}
      </button>
      <button type="button" class="btn" data-library-chapter-rename-cancel @click="dispatch('library.chapter_rename.cancel')">
        {{ t('command.library.chapter_rename.cancel') }}
      </button>
    </div>

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
        <li class="chapters-pad" :style="{ height: slice.padTop + 'px' }" aria-hidden="true"></li>
        <template v-for="item in visibleItems" :key="itemKey(item)">
          <li v-if="item.kind === 'group'" class="wp-item wp-group" :data-library-chapter-group="item.key" role="presentation">
            <!-- aura-allow-text: nhãn nhóm qua t(). -->
            <span class="wp-group-label">{{ groupLabel(item.key) }}</span>
            <!-- aura-allow-text: ghép từ t() với số đếm. -->
            <span class="wp-group-count">{{ t('library.work_page.group_count', { count: String(item.total) }) }}</span>
            <button
              v-if="item.key === 'not_started' && foldToggleVisible"
              type="button"
              class="wp-group-toggle"
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
          </li>
          <li v-else class="wp-item wp-more" data-library-chapter-more>
            <!-- aura-allow-text: ghép từ t() với số đếm. -->
            {{ t('library.work_page.more_hidden', { count: String(item.hidden) }) }}
          </li>
        </template>
        <li class="chapters-pad" :style="{ height: slice.padBottom + 'px' }" aria-hidden="true"></li>
      </ul>
    </div>

    <ChapterOrigin
      v-if="soleSelectedLibraryChapter !== null"
      :author="soleSelectedLibraryChapter.origin_author"
      :site-name="soleSelectedLibraryChapter.origin_site_name"
      :url="soleSelectedLibraryChapter.origin_url"
      :published-at="soleSelectedLibraryChapter.origin_published_at"
      :disabled="libraryChapterOriginBusy"
      class="chapter-origin-block"
      @commit="(edit) => saveCurrentChapterOrigin(edit)"
    />
    <label v-if="soleSelectedLibraryChapter !== null" class="field chapter-origin-apply-through">
      <span>{{ t('mode.library.chapter_origin_apply_through_label') }}</span>
      <input
        type="number"
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
    <p v-if="libraryChapterOriginError !== null" class="error" role="alert">
      <!-- aura-allow-text: KẾT QUẢ của tError(). -->
      {{ tError(libraryChapterOriginError) }}
    </p>
  </section>
</template>

<style scoped>
.wp {
  display: flex;
  flex-direction: column;
  gap: var(--space-panel-block);
  min-height: 100%;
}

.wp:focus {
  /* aura-allow-outline-none: tabindex=-1 root, focused programmatically, not an interactive control. */
  outline: none;
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

.btn-primary {
  color: var(--color-on-primary);
  background: var(--color-primary);
  border-color: var(--color-primary);
}

.btn-primary:disabled {
  color: var(--color-on-surface-variant);
  background: var(--color-surface-sunken);
  border-color: var(--color-outline);
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

.field input {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 8px 8px;
}

.wp-head {
  display: grid;
  grid-template-columns: 78px 1fr;
  gap: calc(var(--space-unit) * 4);
  padding-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

.wp-cover {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 106px;
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

.wp-progress-row {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: calc(var(--space-unit) * 4);
}

.wp-progress {
  flex: 1;
  min-width: 220px;
  max-width: 330px;
}

.wp-track {
  display: flex;
  height: 4px;
  overflow: hidden;
  border-radius: 2px;
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
  gap: calc(var(--space-unit) * 3);
  margin-top: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-status {
  min-width: 238px;
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  background: var(--color-surface);
}

.wp-status--override {
  border-color: var(--color-tm-rule);
  background: var(--color-surface-tm);
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
  gap: calc(var(--space-unit) * 3);
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  background: var(--color-surface-sunken);
  border: 1px solid var(--color-outline);
}

.wp-search {
  min-width: 200px;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 8px 12px;
}

.wp-pill {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 20px;
  padding: 4px 12px;
  cursor: pointer;
}

.wp-pill[aria-pressed='true'] {
  border-color: var(--color-primary);
  border-width: 2px;
  color: var(--color-primary);
  background: var(--color-surface-accent);
  font-weight: var(--weight-ui-md-strong);
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

.wp-selbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 3) calc(var(--space-unit) * 3);
  background: var(--color-surface-accent);
  border-top: 1px solid var(--color-outline);
  border-bottom: 1px solid var(--color-outline);
}

.wp-sel-count {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-primary);
}

.wp-sel-hint {
  margin-left: auto;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-status-menu,
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

.wp-rename-field input {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 8px 8px;
}

.chapters-block {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

/* `max-height` must match VIEWPORT_HEIGHT_PX: `@scroll` only fires when the element scrolls. */
.chapters-list {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 440px;
  overflow-y: auto;
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
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
  padding: 0 calc(var(--space-unit) * 3);
  border-bottom: 1px solid var(--color-outline-faint);
}

.wp-group {
  gap: calc(var(--space-unit) * 2);
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
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wp-more {
  justify-content: center;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.chapter-row {
  position: relative;
  cursor: default;
  user-select: none;
}

.chapter-row--in_progress {
  border-left: 2px solid var(--color-primary);
}

.chapter-row--paused {
  border-left: 2px solid var(--color-tm-rule);
}

.chapter-row--done {
  border-left: 2px solid var(--color-confirmed);
}

.chapter-row--selected {
  background: var(--color-surface-accent);
}

.chapter-row--current {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.chapter-ord {
  min-width: 3em;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
  text-align: right;
}

.chapter-name {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
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
  height: 3px;
  overflow: hidden;
  border-radius: 2px;
  background: var(--color-surface-sunken);
}

.chapter-mini i {
  display: block;
  height: 100%;
  background: var(--color-confirmed);
}

.chapter-seg,
.chapter-when {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
  white-space: nowrap;
}

.chapter-seg {
  min-width: 120px;
}

.chapter-unconfirmed {
  color: var(--color-tm-text);
}

.chapter-unlinked {
  color: var(--color-tm-text);
}

.chapter-chip {
  min-width: 80px;
  padding: 0 calc(var(--space-unit) * 2);
  border-radius: 2px;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  background: var(--color-surface-sunken);
  color: var(--color-on-surface-variant);
  text-align: center;
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

.chapter-origin-block {
  margin-top: 0;
}
</style>
