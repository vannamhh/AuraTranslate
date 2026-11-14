<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import LookupRecord from './panels/LookupRecord.vue'
import { toneMarkedReading } from './panels/pinyin'
import { sensesOfGroup } from './panels/lookupView'
import { enabledSourceCount } from './panels/dictSourcesState'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import {
  aimPaletteItem,
  paletteCommandItems,
  paletteCursor,
  paletteIsOpen,
  paletteIsSearching,
  paletteItems,
  paletteQuery,
  paletteWorkItems,
  paletteWorksAreFiltered,
  paletteWorksErrorText,
  paletteWorksState,
  exitPaletteLookupMode,
  setPaletteQuery,
} from './paletteState'
import {
  paletteLookupActive,
  paletteLookupCursor,
  paletteLookupError,
  paletteLookupGroups,
  paletteLookupPreviewBody,
  paletteLookupPreviewError,
  paletteLookupPreviewHeadword,
  paletteLookupPreviewPending,
  paletteLookupPreviewResponse,
  paletteLookupQueryTruncated,
  paletteLookupRows,
  paletteLookupSkipped,
  paletteLookupStatus,
} from './paletteLookupState'
import { libraryWorksMatched, libraryWorksTotal } from './modes/libraryWorks'

const panel = useTemplateRef<HTMLElement>('panel')
const input = useTemplateRef<HTMLInputElement>('input')

let returnFocusTo: HTMLElement | null = null

watch(paletteIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-palette-open]')
    void nextTick(() => input.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null
  if (back !== null && back.isConnected) {
    back.focus()
    return
  }
  document.querySelector<HTMLElement>('[data-palette-open]')?.focus()
})

watch(paletteLookupActive, () => {
  void nextTick(() => input.value?.focus())
})

watch([paletteLookupCursor, paletteLookupRows], () => {
  if (!paletteLookupActive.value) return
  void nextTick(() => {
    const row = panel.value?.querySelector(`#${optionId(paletteLookupCursor.value)}`)
    row?.scrollIntoView({ block: 'nearest' })
  })
})

const lookupErrorText = computed(() => (paletteLookupError.value === null ? null : tError(paletteLookupError.value)))
const lookupPreviewErrorText = computed(() =>
  paletteLookupPreviewError.value === null ? null : tError(paletteLookupPreviewError.value),
)
const lookupPreviewGroups = computed(() => paletteLookupPreviewResponse.value?.grouped.groups ?? [])
/** First row index of each visible group in the flat cursor order. */
const lookupGroupStarts = computed(() => {
  let start = 0
  return paletteLookupGroups.value.map((group) => {
    const at = start
    start += group.rows.length
    return at
  })
})

const activeId = computed(() => {
  if (paletteLookupActive.value) return paletteLookupRows.value.length === 0 ? undefined : optionId(paletteLookupCursor.value)
  return paletteItems.value.length === 0 ? undefined : optionId(paletteCursor.value)
})
const noResults = computed(
  () =>
    !paletteLookupActive.value &&
    paletteItems.value.length === 0 &&
    (!paletteIsSearching.value || paletteWorksState.value === 'ready'),
)
const worksNotice = computed(() => {
  if (!paletteIsSearching.value) return null
  if (paletteWorksState.value === 'ready') return paletteWorksAreFiltered.value ? 'filtered' : null
  return paletteWorksState.value
})
const bodyShown = computed(
  () => paletteLookupActive.value || paletteItems.value.length > 0 || worksNotice.value !== null || noResults.value,
)

function optionId(index: number): string {
  return `palette-option-${String(index)}`
}

function onInputKeydown(event: KeyboardEvent): void {
  if (event.isComposing) return
  if (event.key === 'Backspace' && paletteLookupActive.value && paletteQuery.value === '') {
    event.preventDefault()
    exitPaletteLookupMode()
    return
  }
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    dispatch('palette.select_next')
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    dispatch('palette.select_prev')
  } else if (event.key === 'Enter') {
    event.preventDefault()
    dispatch('palette.run_selected')
  }
}

function onInput(event: Event): void {
  const el = event.target
  if (!(el instanceof HTMLInputElement)) return
  setPaletteQuery(el.value, event instanceof InputEvent && event.isComposing)
  // The `/` that switched the mode is not part of the text, but the query may not have changed to re-render it.
  if (el.value !== paletteQuery.value) el.value = paletteQuery.value
}

function onCompositionEnd(event: CompositionEvent): void {
  if (event.target instanceof HTMLInputElement) setPaletteQuery(event.target.value)
}

function readingOf(raw: string | null): string | null {
  return raw === null ? null : toneMarkedReading(raw)
}

function trapTab(event: KeyboardEvent): void {
  const root = panel.value
  if (root === null) return

  event.preventDefault()
  const stops = Array.from(root.querySelectorAll<HTMLElement>('input:not([disabled])'))
  if (stops.length === 0) {
    root.focus()
    return
  }
  const active = document.activeElement
  const index = active instanceof HTMLElement ? stops.indexOf(active) : -1
  const step = event.shiftKey ? -1 : 1
  const next = index === -1 ? (event.shiftKey ? stops.length - 1 : 0) : index + step
  stops[(next + stops.length) % stops.length].focus()
}
</script>

<template>
  <div
    v-if="paletteIsOpen"
    class="pal-scrim"
    @mousedown.self.prevent
    @click.self="dispatch('palette.close')"
    @keydown.esc="dispatch('palette.close')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="pal-panel" tabindex="-1" role="dialog" aria-modal="true" :aria-label="t('palette.title')">
      <div class="pal-search">
        <span v-if="paletteLookupActive" class="pal-mode" data-palette-mode="lookup">{{ t('palette.lookup.mode_label') }}</span>
        <input
          ref="input"
          class="ui-field ui-field--lg ui-field--full"
          type="text"
          role="combobox"
          aria-controls="palette-list"
          :aria-expanded="paletteLookupActive ? paletteLookupRows.length > 0 : paletteItems.length > 0"
          aria-autocomplete="list"
          :aria-activedescendant="activeId"
          :aria-label="t(paletteLookupActive ? 'palette.lookup.input_label' : 'palette.input_label')"
          :placeholder="t(paletteLookupActive ? 'palette.lookup.input_placeholder' : 'palette.input_placeholder')"
          :value="paletteQuery"
          autocomplete="off"
          @input="onInput($event)"
          @compositionend="onCompositionEnd($event)"
          @keydown="onInputKeydown($event)"
        />
        <button
          v-if="!paletteLookupActive"
          type="button"
          class="ui-btn ui-btn--quiet"
          data-palette-lookup-button
          @mousedown.prevent
          @click="dispatch('palette.lookup_mode')"
        >
          {{ t('palette.lookup.mode_button') }}
        </button>
      </div>
      <div v-if="paletteLookupActive" class="pal-body pal-body--shown pal-body--lookup" data-palette-lookup-body>
        <p v-if="paletteLookupStatus === 'idle'" class="pal-status" role="status">{{ t('palette.lookup.hint') }}</p>
        <p v-else-if="paletteLookupStatus === 'loading'" class="pal-status" role="status">{{ t('palette.lookup.loading') }}</p>
        <p v-else-if="paletteLookupStatus === 'error'" class="pal-status pal-status--error" role="alert" data-palette-lookup-error>
          {{ t('palette.lookup.error') }}
          <!-- aura-allow-text: result of tError() on the quick-lookup failure. -->
          <span>{{ lookupErrorText }}</span>
        </p>
        <p v-else-if="paletteLookupStatus === 'unavailable'" class="pal-status pal-status--error" role="alert">
          {{ t('palette.lookup.unavailable') }}
        </p>
        <p v-else-if="paletteLookupStatus === 'no_layers'" class="pal-status" role="status">{{ t('palette.lookup.no_layers') }}</p>
        <p v-else-if="paletteLookupStatus === 'no_results'" class="pal-status" role="status" data-palette-lookup-empty>
          {{ t('palette.lookup.no_results') }}
        </p>
        <p v-if="paletteLookupQueryTruncated" class="pal-status" role="status">{{ t('palette.lookup.truncated') }}</p>
        <p v-if="paletteLookupSkipped.length > 0" class="pal-status" role="status">{{ t('palette.lookup.skipped') }}</p>
        <div id="palette-list" class="pal-list pal-list--lookup" role="listbox" :aria-label="t('palette.lookup.mode_label')">
          <div
            v-for="(group, g) in paletteLookupGroups"
            :key="group.id"
            class="pal-group"
            role="group"
            :aria-labelledby="`palette-lookup-group-${group.id}`"
            :data-palette-lookup-group="group.id"
          >
            <h3 :id="`palette-lookup-group-${group.id}`" class="pal-group-title">{{ t(`palette.lookup.group_${group.id}`) }}</h3>
            <p v-if="group.status === 'outdated_dictionary'" class="pal-status" role="status">{{ t('palette.lookup.group_outdated') }}</p>
            <p v-else-if="group.status === 'error'" class="pal-status pal-status--error" role="alert">
              {{ t('palette.lookup.group_error') }}
              <!-- aura-allow-text: result of tError() on the group failure. -->
              <span>{{ group.error === null ? '' : tError(group.error) }}</span>
            </p>
            <p v-if="group.incomplete" class="pal-status" role="status">{{ t('palette.lookup.group_incomplete') }}</p>
            <div
              v-for="(row, i) in group.rows"
              :id="optionId(lookupGroupStarts[g] + i)"
              :key="`${group.id}:${String(i)}`"
              class="ui-menu-item pal-option"
              :class="{ 'pal-option-current': lookupGroupStarts[g] + i === paletteLookupCursor }"
              role="option"
              :aria-selected="lookupGroupStarts[g] + i === paletteLookupCursor"
              data-palette-kind="lookup"
              @mousedown.prevent="aimPaletteItem(lookupGroupStarts[g] + i)"
              @click="dispatch('palette.run_selected')"
            >
              <!-- aura-allow-text: dictionary headword — DATA. -->
              <span class="pal-headword">{{ row.headword }}</span>
              <!-- aura-allow-text: reading — dictionary DATA, tone-marked by toneMarkedReading. -->
              <span v-if="row.reading !== null" class="pal-hint">{{ readingOf(row.reading) }}</span>
              <!-- aura-allow-text: first gloss — dictionary DATA. -->
              <span class="pal-label pal-gloss">{{ row.gloss ?? '' }}</span>
            </div>
            <p v-if="group.has_more" class="pal-status" role="status">{{ t('palette.lookup.group_more') }}</p>
          </div>
        </div>
        <section
          v-if="paletteLookupPreviewHeadword !== null"
          class="pal-preview"
          :aria-label="t('palette.lookup.preview_label')"
          data-palette-preview
        >
          <p v-if="paletteLookupPreviewBody === 'waiting' && paletteLookupPreviewPending" class="pal-status" role="status">
            {{ t('palette.lookup.preview_loading') }}
          </p>
          <p v-else-if="paletteLookupPreviewBody === 'waiting'" class="pal-status pal-status--error" role="alert">
            {{ t('palette.lookup.unavailable') }}
          </p>
          <p v-else-if="paletteLookupPreviewBody === 'error'" class="pal-status pal-status--error" role="alert">
            {{ t('panel.lookup.lookup_failed') }}
            <!-- aura-allow-text: result of tError() on the preview failure. -->
            <span>{{ lookupPreviewErrorText }}</span>
          </p>
          <p v-else-if="paletteLookupPreviewBody === 'no_layers'" class="pal-status">{{ t('panel.lookup.no_layers') }}</p>
          <p v-else-if="paletteLookupPreviewBody === 'too_short'" class="pal-status">{{ t('panel.lookup.query_too_short') }}</p>
          <p v-else-if="paletteLookupPreviewBody === 'all_sources_off'" class="pal-status">{{ t('panel.lookup.all_sources_off') }}</p>
          <template v-else-if="paletteLookupPreviewBody === 'not_found'">
            <!-- aura-allow-text: not-found title — the headword is DATA passed to t(). -->
            <p class="pal-status">{{ t('panel.lookup.not_found', { query: paletteLookupPreviewHeadword }) }}</p>
            <p class="pal-status">{{ t('panel.lookup.not_found_checked', { n: String(enabledSourceCount) }) }}</p>
          </template>
          <template v-else-if="paletteLookupPreviewBody === 'results'">
            <LookupRecord
              v-for="group in lookupPreviewGroups"
              :key="`${group.layer}:${group.source.code}`"
              :group="group"
              :senses="sensesOfGroup(group, paletteLookupPreviewResponse?.senses_by_layer ?? {})"
              :senses-failed="(paletteLookupPreviewResponse?.senses_failed ?? []).includes(group.layer)"
              no-pin
            />
          </template>
        </section>
      </div>
      <div v-else class="pal-body" :class="{ 'pal-body--shown': bodyShown }">
        <p v-if="worksNotice === 'loading'" class="pal-status" role="status">{{ t('palette.works_loading') }}</p>
        <p v-else-if="worksNotice === 'error'" class="pal-status pal-status--error" role="alert">
          {{ t('palette.works_error') }}
          <!-- aura-allow-text: result of tError() on the Library load failure. -->
          <span>{{ paletteWorksErrorText }}</span>
        </p>
        <p v-else-if="worksNotice === 'filtered'" class="pal-status" role="status">
          {{ t('palette.works_filtered', { matched_count: String(libraryWorksMatched), total_count: String(libraryWorksTotal) }) }}
        </p>
        <div id="palette-list" class="pal-list" role="listbox" :aria-label="t('palette.title')">
          <div v-if="paletteWorkItems.length > 0" class="pal-group" role="group" aria-labelledby="palette-group-works">
            <h3 id="palette-group-works" class="pal-group-title">{{ t('palette.group_works') }}</h3>
            <div
              v-for="(item, i) in paletteWorkItems"
              :id="optionId(i)"
              :key="item.key"
              class="ui-menu-item pal-option"
              :class="{ 'pal-option-current': i === paletteCursor }"
              role="option"
              :aria-selected="i === paletteCursor"
              data-palette-kind="work"
              @mousedown.prevent="aimPaletteItem(i)"
              @click="dispatch('palette.run_selected')"
            >
              <!-- aura-allow-text: Work name typed by the user. -->
              <span class="pal-label">{{ item.label }}</span>
              <span v-if="item.kind === 'work' && item.doneCount !== null" class="pal-hint">{{
                t('palette.work_progress', { done_count: String(item.doneCount), chapter_count: String(item.chapterCount) })
              }}</span>
            </div>
          </div>
          <div v-if="paletteCommandItems.length > 0" class="pal-group" role="group" aria-labelledby="palette-group-commands">
            <h3 id="palette-group-commands" class="pal-group-title">{{ t('palette.group_commands') }}</h3>
            <div
              v-for="(item, i) in paletteCommandItems"
              :id="optionId(paletteWorkItems.length + i)"
              :key="item.key"
              class="ui-menu-item pal-option"
              :class="{ 'pal-option-current': paletteWorkItems.length + i === paletteCursor }"
              role="option"
              :aria-selected="paletteWorkItems.length + i === paletteCursor"
              data-palette-kind="command"
              @mousedown.prevent="aimPaletteItem(paletteWorkItems.length + i)"
              @click="dispatch('palette.run_selected')"
            >
              <!-- aura-allow-text: label already resolved through t(). -->
              <span class="pal-label">{{ item.label }}</span>
              <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
              <span v-if="item.kind === 'command'" class="pal-hint">{{ item.chords.join(' · ') }}</span>
            </div>
          </div>
        </div>
        <p v-if="noResults" class="pal-status" role="status">{{ t('palette.no_results') }}</p>
      </div>
    </section>
  </div>
</template>

<style scoped>
.pal-scrim {
  position: fixed;
  inset: 0;
  /* aura-allow-z-index: dockview gives each panel group its own stacking context, so DOM order alone cannot put this above the grid. */
  z-index: 10;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: calc(var(--space-unit) * 16) var(--space-panel-inline) var(--space-panel-inline);
  backdrop-filter: brightness(0.82);
}

.pal-panel {
  display: flex;
  flex-direction: column;
  width: min(637px, 100%);
  max-height: 60vh;
  overflow: hidden;
  border-radius: var(--radius-default);
  background: var(--color-surface);
  color: var(--color-on-surface);
  box-shadow: var(--shadow-popover);
}

.pal-panel:focus-visible {
  outline: 1px solid var(--color-primary);
}

.pal-search {
  flex: none;
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  padding: calc(var(--space-unit) * 1);
}

.pal-search .ui-field {
  flex: 1 1 0;
  min-width: 0;
}

.pal-mode {
  flex: none;
  padding: 0 calc(var(--space-unit) * 1);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-sm);
  color: var(--color-primary);
}

.pal-headword {
  flex: none;
  font-weight: var(--weight-ui-md-strong);
}

.pal-gloss {
  flex: 1 1 auto;
  text-align: end;
  color: var(--color-on-surface-variant);
}

.pal-body--lookup {
  overflow: hidden;
}

.pal-list--lookup {
  flex: 1 1 auto;
}

.pal-preview {
  flex: 0 1 auto;
  max-height: 24vh;
  overflow-y: auto;
  margin-top: calc(var(--space-unit) * 1);
  padding: 0 calc(var(--space-unit) * 2) calc(var(--space-unit) * 1);
  border-top: 1px solid var(--color-outline-faint);
}

.pal-body {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.pal-body--shown {
  padding-bottom: calc(var(--space-unit) * 1);
  border-top: 1px solid var(--color-outline-faint);
}

.pal-list {
  min-height: 0;
  overflow-y: auto;
}

.pal-group {
  padding-top: calc(var(--space-unit) * 1);
}

.pal-group-title {
  display: flex;
  align-items: center;
  height: 28px;
  margin: 0;
  padding: 0 calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pal-option {
  justify-content: space-between;
}

.pal-option-current {
  background: var(--color-outline);
}

.pal-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pal-hint {
  flex: none;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pal-status {
  margin: 0;
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pal-status--error {
  color: var(--color-error);
}
</style>
