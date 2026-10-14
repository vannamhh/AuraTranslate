<script setup lang="ts">
import { computed } from 'vue'
import { dispatch } from '../commands'
import type { SearchHit } from '../config/library'
import { t, tError } from '../i18n'
import { shortcutRows } from '../config/shortcutsState'
import {
  SEARCH_GROUP_COLLAPSED_HITS,
  librarySearchChaptersFilteredOut,
  librarySearchChaptersTotal,
  librarySearchCoverageGapState,
  librarySearchCursor,
  librarySearchEffectiveMode,
  librarySearchElapsedMs,
  librarySearchError,
  librarySearchFilterActive,
  librarySearchGroups,
  librarySearchHits,
  librarySearchReportScope,
  librarySearchResultCount,
  librarySearchStatusKey,
  librarySearchTruncated,
  librarySearchVisibleHits,
  librarySearchWorkId,
  librarySearchWorksTotal,
  selectLibrarySearchGroup,
  setLibrarySearchCursor,
  splitSnippet,
} from './librarySearch'
import { librarySortKey, librarySourceLangFilter, libraryStatusFilter } from './libraryWorks'

const searchChord = computed(
  () => shortcutRows.value.find((row) => row.id === 'library.search')?.display.join(' / ') ?? '',
)

const countText = computed(() => {
  const { count, lowerBound } = librarySearchResultCount.value
  return t('mode.library.search_counter', {
    count: lowerBound ? `${count}+` : String(count),
    ms: String(librarySearchElapsedMs.value ?? 0),
  })
})

const groupOffsets = computed(() => {
  let offset = 0
  return librarySearchGroups.value.map((group) => {
    const start = offset
    offset += group.visible.length
    return start
  })
})

const showsResults = computed(() => librarySearchHits.value.length > 0)
const showsEmpty = computed(
  () =>
    !showsResults.value &&
    (librarySearchStatusKey.value === 'no_match' ||
      librarySearchStatusKey.value === 'no_match_widened' ||
      librarySearchStatusKey.value === 'short_query'),
)
const showsModeNote = computed(
  () =>
    showsResults.value ||
    librarySearchStatusKey.value === 'no_match' ||
    librarySearchStatusKey.value === 'no_match_widened',
)
const modeIsRelaxed = computed(
  () => librarySearchEffectiveMode.value === 'lenient' || librarySearchStatusKey.value === 'result_widened',
)
const truncatedNote = computed(() => {
  const { count } = librarySearchResultCount.value
  const shown = librarySearchHits.value.length
  if (!librarySearchTruncated.value || count <= shown) return ''
  return t('mode.library.search_truncated_note', { shown: String(shown), count: String(count) })
})

function statusText(): string {
  switch (librarySearchStatusKey.value) {
    case 'searching':
      return t('mode.library.search_searching')
    case 'not_typed':
      return t('mode.library.search_not_typed')
    case 'index_empty':
      return t('mode.library.search_index_empty')
    case 'short_query':
      return t('mode.library.search_short_query')
    case 'no_match_widened':
      return t('mode.library.search_no_match_widened')
    case 'no_match':
      return t('mode.library.search_no_match')
    default:
      return countText.value
  }
}

function modeNoteText(): string {
  if (librarySearchStatusKey.value === 'result_widened' || librarySearchStatusKey.value === 'no_match_widened') {
    return t('mode.library.search_note_widened')
  }
  return librarySearchEffectiveMode.value === 'lenient'
    ? t('mode.library.search_note_lenient')
    : t('mode.library.search_note_exact')
}

function emptyScopeText(): string {
  switch (librarySearchReportScope.value) {
    case 'source':
      return t('mode.library.search_empty_scope_source')
    case 'target':
      return t('mode.library.search_empty_scope_target')
    default:
      return t('mode.library.search_empty_scope_both')
  }
}

function workStatusLabel(status: string | null): string {
  switch (status) {
    case 'not_started':
      return t('lifecycle.not_started')
    case 'in_progress':
      return t('lifecycle.in_progress')
    case 'paused':
      return t('lifecycle.paused')
    case 'done':
      return t('lifecycle.done')
    default:
      return t('mode.library.works_status_unknown')
  }
}

function directionLabel(lang: string): string {
  const name = lang === 'zh' ? t('mode.library.search_lang_zh') : lang === 'en' ? t('mode.library.search_lang_en') : lang
  return t('mode.library.search_chip_direction', { lang: name })
}

function sortLabel(): string {
  return librarySortKey.value === 'name_asc' ? t('mode.library.sort_name_asc') : t('mode.library.sort_updated_desc')
}

function chapterNumber(hit: SearchHit): string {
  return t('mode.library.chapter_untitled', { ord: String(hit.chapter_ord) })
}

function hitMeta(hit: SearchHit): string {
  if (hit.segment_ord === null) return t('mode.library.search_chapter_level')
  const segment = t('mode.library.search_hit_segment', { ord: String(hit.segment_ord) })
  if (hit.field === 'source') return `${segment} · ${t('mode.library.search_hit_side_source')}`
  const state =
    hit.segment_status === 'confirmed' ? t('mode.library.search_hit_confirmed') : t('mode.library.search_hit_unconfirmed')
  return `${segment} · ${state}`
}

function groupCountText(count: number, lowerBound: boolean): string {
  return t('mode.library.search_group_count', { count: lowerBound ? `${count}+` : String(count) })
}

function remainingText(remaining: number): string {
  return t('mode.library.search_group_more', { count: String(remaining) })
}

function seeAllText(count: number, lowerBound: boolean): string {
  return t('mode.library.search_group_see_all', { count: lowerBound ? `${count}+` : String(count) })
}
</script>

<template>
  <div class="search-results-block" data-library-search-results-block>
    <!-- aura-allow-text: mọi nhánh đều qua t() trong statusText(). -->
    <p class="status" role="status" data-library-search-status>{{ statusText() }}</p>
    <!-- aura-allow-text: như trên, qua tError(). -->
    <p class="error" role="status">{{ librarySearchError ? tError(librarySearchError) : '' }}</p>

    <div v-if="showsResults" class="search-summary">
      <span class="search-extent" data-library-search-extent>{{
        t('mode.library.search_extent', {
          chapters: String(librarySearchChaptersTotal),
          works: String(librarySearchWorksTotal),
        })
      }}</span>
    </div>

    <div class="search-chips" data-library-search-chips>
      <span v-if="libraryStatusFilter.size === 0" class="search-chip">{{ t('mode.library.search_chip_all_statuses') }}</span>
      <template v-else>
        <!-- aura-allow-text: nhãn trạng thái qua t() trong workStatusLabel(). -->
        <span v-for="status in Array.from(libraryStatusFilter)" :key="status" class="search-chip search-chip--on">{{
          workStatusLabel(status)
        }}</span>
      </template>
      <!-- aura-allow-text: nhãn hướng dịch qua t() trong directionLabel(). -->
      <span v-if="librarySourceLangFilter !== null" class="search-chip search-chip--on">{{
        directionLabel(librarySourceLangFilter)
      }}</span>
      <!-- aura-allow-text: nhãn sắp xếp qua t() trong sortLabel(). -->
      <span class="search-chip">{{ sortLabel() }}</span>
      <template v-if="librarySearchWorkId !== null">
        <span class="search-chip search-chip--on">{{ t('mode.library.search_chip_work_only') }}</span>
        <button type="button" class="btn" data-library-search-work-clear @click="dispatch('library.search')">
          {{ t('mode.library.search_chip_work_clear') }}
        </button>
      </template>
    </div>

    <!-- aura-allow-text: mọi nhánh của modeNoteText() đều qua t(). -->
    <p
      v-if="showsModeNote"
      class="mode-note"
      :class="{ 'mode-note--relax': modeIsRelaxed }"
      data-library-search-mode-note
    >
      {{ modeNoteText() }}
    </p>

    <p v-if="librarySearchCoverageGapState !== null" class="status" role="status" data-library-search-coverage-gap>
      {{
        t('mode.library.search_coverage_gap', {
          missing: String(librarySearchCoverageGapState.missing),
          total: String(librarySearchCoverageGapState.total),
        })
      }}
    </p>

    <!-- aura-allow-text: truncatedNote dựng bằng t(). -->
    <p v-if="truncatedNote !== ''" class="status" data-library-search-truncated>{{ truncatedNote }}</p>

    <div v-if="librarySearchVisibleHits.length > 0" class="grid-nav">
      <button
        type="button"
        class="btn"
        data-library-search-prev
        :aria-label="t('mode.library.search_prev')"
        @click="dispatch('library.search_prev')"
      >
        ‹
      </button>
      <span class="grid-nav-position">{{
        t('mode.library.search_position', {
          current: String(librarySearchCursor + 1),
          total: String(librarySearchVisibleHits.length),
        })
      }}</span>
      <button
        type="button"
        class="btn"
        data-library-search-next
        :aria-label="t('mode.library.search_next')"
        @click="dispatch('library.search_next')"
      >
        ›
      </button>
    </div>

    <ul v-if="showsResults" class="search-groups" data-library-search-results>
      <li
        v-for="(group, groupIndex) in librarySearchGroups"
        :key="group.workId"
        class="search-group"
        data-library-search-group
        :data-work-id="group.workId"
      >
        <div class="wk">
          <!-- aura-allow-text: tên Tác phẩm là DỮ LIỆU người dùng. -->
          <span class="wk-title" data-library-search-group-name>{{ group.workName }}</span>
          <!-- aura-allow-text: nhãn qua t() trong directionLabel()/workStatusLabel(). -->
          <span class="wk-meta">{{ directionLabel(group.sourceLang) }} · {{ workStatusLabel(group.workStatus) }}</span>
          <!-- aura-allow-text: groupCountText() dựng bằng t(). -->
          <span class="wk-count" data-library-search-group-count>{{
            groupCountText(group.count, group.countIsLowerBound)
          }}</span>
        </div>
        <ul class="hit-rows">
          <li
            v-for="(hit, hitIndex) in group.visible"
            :key="`${hit.work_id}-${hit.chapter_id}-${hit.segment_id ?? 'chapter'}-${hit.field}`"
            class="hitrow"
            :class="[
              hit.field === 'target' ? 'hitrow--target' : 'hitrow--source',
              { 'hitrow--current': groupOffsets[groupIndex] + hitIndex === librarySearchCursor },
            ]"
            :aria-current="groupOffsets[groupIndex] + hitIndex === librarySearchCursor ? 'true' : undefined"
            :data-field="hit.field"
            data-library-search-hit
          >
            <div class="hit-locus">
              <!-- aura-allow-text: chapterNumber() dựng bằng t(). -->
              <strong data-library-search-hit-chapter>{{ chapterNumber(hit) }}</strong>
              <!-- aura-allow-text: tiêu đề Chương là DỮ LIỆU người dùng. -->
              <span v-if="hit.chapter_title !== null" class="hit-chapter-title">{{ hit.chapter_title }}</span>
              <!-- aura-allow-text: hitMeta() dựng bằng t(). -->
              <span>{{ hitMeta(hit) }}</span>
              <span v-if="hit.match_kind === 'lenient'" class="hit-tag" data-library-search-hit-lenient>{{
                t('mode.library.search_hit_lenient')
              }}</span>
            </div>
            <!-- aura-allow-text: đoạn trích là DỮ LIỆU người dùng, văn bản THUẦN (AD-16) -- không v-html. -->
            <p class="hit-snippet">
              <!-- aura-allow-text: phần đoạn trích là DỮ LIỆU người dùng, văn bản THUẦN (AD-16). -->
              <template v-for="(part, partIndex) in splitSnippet(hit.snippet)" :key="partIndex">
                <!-- aura-allow-text: phần khớp là DỮ LIỆU người dùng (AD-16). -->
                <mark
                  v-if="part.match"
                  :class="{ 'mark--loose': hit.match_kind === 'lenient' }"
                  data-library-search-mark
                  :data-match-kind="hit.match_kind"
                  >{{ part.text }}</mark
                >
                <!-- aura-allow-text: phần còn lại của đoạn trích là DỮ LIỆU người dùng (AD-16). -->
                <span v-else>{{ part.text }}</span>
              </template>
            </p>
            <div class="hit-actions">
              <button
                type="button"
                class="btn"
                data-library-search-open
                @pointerdown="setLibrarySearchCursor(groupOffsets[groupIndex] + hitIndex)"
                @focus="setLibrarySearchCursor(groupOffsets[groupIndex] + hitIndex)"
                @click="dispatch('library.open_search_hit')"
              >
                {{ t('mode.library.search_open_button') }}
              </button>
              <button
                type="button"
                class="btn"
                data-library-search-open-reading
                @pointerdown="setLibrarySearchCursor(groupOffsets[groupIndex] + hitIndex)"
                @focus="setLibrarySearchCursor(groupOffsets[groupIndex] + hitIndex)"
                @click="dispatch('library.open_search_hit_reading')"
              >
                {{ t('mode.library.search_open_reading_button') }}
              </button>
            </div>
          </li>
        </ul>
        <div class="group-foot">
          <button
            v-if="!group.expanded && group.hits.length > SEARCH_GROUP_COLLAPSED_HITS"
            type="button"
            class="btn"
            data-library-search-group-expand
            @pointerdown="selectLibrarySearchGroup(group.workId)"
            @focus="selectLibrarySearchGroup(group.workId)"
            @click="dispatch('library.search_group_toggle')"
          >
            <!-- aura-allow-text: remainingText() dựng bằng t(). -->
            {{ remainingText(group.remaining) }}
          </button>
          <button
            v-if="group.expanded"
            type="button"
            class="btn"
            data-library-search-group-collapse
            @pointerdown="selectLibrarySearchGroup(group.workId)"
            @focus="selectLibrarySearchGroup(group.workId)"
            @click="dispatch('library.search_group_toggle')"
          >
            {{ t('mode.library.search_group_collapse') }}
          </button>
          <button
            v-if="
              librarySearchWorkId === null &&
              group.hasUnreturned &&
              (group.expanded || group.hits.length <= SEARCH_GROUP_COLLAPSED_HITS)
            "
            type="button"
            class="btn"
            data-library-search-group-see-all
            @pointerdown="selectLibrarySearchGroup(group.workId)"
            @focus="selectLibrarySearchGroup(group.workId)"
            @click="dispatch('library.search_group_see_all')"
          >
            <!-- aura-allow-text: seeAllText() dựng bằng t(). -->
            {{ seeAllText(group.count, group.countIsLowerBound) }}
          </button>
        </div>
      </li>
    </ul>

    <div v-if="showsEmpty" class="empty" data-library-search-empty>
      <div class="rule" aria-hidden="true"></div>
      <!-- aura-allow-text: emptyScopeText() dựng bằng t(). -->
      <p v-if="librarySearchStatusKey !== 'short_query'" class="small" data-library-search-empty-scope>
        {{ emptyScopeText() }}
      </p>
      <p v-if="librarySearchChaptersFilteredOut > 0" class="small" data-library-search-empty-filtered>
        {{
          t('mode.library.search_empty_filtered', {
            filtered: String(librarySearchChaptersFilteredOut),
            total: String(librarySearchChaptersTotal),
          })
        }}
      </p>
      <div class="sugg">
        <p v-if="librarySearchFilterActive" class="small">
          <button type="button" class="btn" data-library-search-clear-filters @click="dispatch('library.filter_clear')">
            {{ t('mode.library.clear_filter') }}
          </button>
          {{ t('mode.library.search_empty_step_clear_filter', { total: String(librarySearchChaptersTotal) }) }}
        </p>
        <p class="small">{{ t('mode.library.search_empty_step_concordance') }}</p>
        <p class="small">
          <button type="button" class="btn" data-library-search-rescan @click="dispatch('library.rescan')">
            {{ t('mode.library.rescan') }}
          </button>
          {{ t('mode.library.search_empty_step_rescan') }}
        </p>
      </div>
    </div>

    <p v-if="showsResults && searchChord !== ''" class="key-hint" data-library-search-key-hint>
      {{ t('mode.library.search_key_hint', { chord: searchChord }) }}
    </p>
  </div>
</template>

<style scoped>
.search-results-block {
  margin-bottom: var(--space-panel-block);
}

.status {
  margin: 12px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.error {
  margin: 4px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-error);
}

.search-summary,
.search-chips,
.grid-nav,
.hit-actions,
.group-foot {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.search-summary,
.search-chips,
.grid-nav {
  margin: 12px 0 0;
}

.search-extent,
.grid-nav-position,
.key-hint {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.key-hint {
  margin: 12px 0 0;
}

.search-chip {
  padding: 4px 12px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-full);
}

.search-chip--on {
  color: var(--color-primary);
  background: var(--color-surface-accent);
  border-color: var(--color-primary);
  font-weight: var(--weight-ui-md-strong);
}

.mode-note {
  margin: 12px 0 0;
  padding: 8px 12px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  background: var(--color-surface-accent);
  border-radius: 4px;
}

.mode-note--relax {
  color: var(--color-tm-text);
  background: var(--color-surface-tm);
}

.btn {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-primary);
  background: var(--color-surface-sunken);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 4px 12px;
  cursor: pointer;
}

.search-groups,
.hit-rows {
  margin: 12px 0 0;
  padding: 0;
  list-style: none;
}

.search-group {
  border-top: 1px solid var(--color-outline-faint);
  padding-bottom: 8px;
}

.wk {
  display: flex;
  align-items: baseline;
  gap: 12px;
  padding: 12px 0 4px;
}

.wk-title {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.wk-meta,
.wk-count {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.wk-count {
  margin-left: auto;
}

.hit-rows {
  margin: 0;
}

.hitrow {
  position: relative;
  display: grid;
  grid-template-columns: 160px 1fr;
  gap: 4px 16px;
  padding: 8px 0 8px 16px;
  border-bottom: 1px solid var(--color-outline-faint);
}

.hitrow::before {
  content: '';
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 2px;
  border-radius: 2px;
  background: var(--color-outline-faint);
}

.hitrow--source::before {
  background: var(--color-on-surface-variant);
}

.hitrow--target::before {
  background: var(--color-primary);
}

.hitrow--current {
  background: var(--color-surface-sunken);
}

.hit-locus {
  display: flex;
  flex-direction: column;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.hit-locus strong {
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.hit-tag {
  color: var(--color-tm-text);
}

.hit-snippet {
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  word-break: break-word;
  min-width: 0;
}

mark {
  padding: 0 4px;
  border-radius: 2px;
  color: var(--color-primary);
  background: var(--color-surface-accent);
  font-weight: var(--weight-ui-md-strong);
}

.mark--loose {
  color: var(--color-tm-text);
  background: var(--color-surface-tm);
}

.hit-actions {
  grid-column: 2;
}

.group-foot {
  margin-top: 8px;
}

.empty {
  padding: 24px 0;
}

.rule {
  width: 34px;
  height: 2px;
  background: var(--color-ornament);
  border-radius: 1px;
  margin-bottom: 8px;
}

.small {
  margin: 0 0 8px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.sugg {
  margin-top: 16px;
}
</style>
