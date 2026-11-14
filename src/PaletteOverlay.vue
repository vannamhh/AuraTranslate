<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t } from './i18n'
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
  setPaletteQuery,
} from './paletteState'
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

const activeId = computed(() => (paletteItems.value.length === 0 ? undefined : optionId(paletteCursor.value)))
const noResults = computed(
  () => paletteItems.value.length === 0 && (!paletteIsSearching.value || paletteWorksState.value === 'ready'),
)
const worksNotice = computed(() => {
  if (!paletteIsSearching.value) return null
  if (paletteWorksState.value === 'ready') return paletteWorksAreFiltered.value ? 'filtered' : null
  return paletteWorksState.value
})
const bodyShown = computed(() => paletteItems.value.length > 0 || worksNotice.value !== null || noResults.value)

function optionId(index: number): string {
  return `palette-option-${String(index)}`
}

function onInputKeydown(event: KeyboardEvent): void {
  if (event.isComposing) return
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
  if (event.target instanceof HTMLInputElement) setPaletteQuery(event.target.value)
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
        <input
          ref="input"
          class="ui-field ui-field--lg ui-field--full"
          type="text"
          role="combobox"
          aria-controls="palette-list"
          :aria-expanded="paletteItems.length > 0"
          aria-autocomplete="list"
          :aria-activedescendant="activeId"
          :aria-label="t('palette.input_label')"
          :placeholder="t('palette.input_placeholder')"
          :value="paletteQuery"
          autocomplete="off"
          @input="onInput($event)"
          @keydown="onInputKeydown($event)"
        />
      </div>
      <div class="pal-body" :class="{ 'pal-body--shown': bodyShown }">
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
}

.pal-panel {
  display: flex;
  flex-direction: column;
  width: min(637px, 100%);
  max-height: 60vh;
  overflow: hidden;
  border: 1px solid var(--color-control-rule);
  border-radius: var(--radius-default);
  background: var(--color-surface);
  color: var(--color-on-surface);
  box-shadow: var(--shadow-popover);
}

.pal-panel:focus-visible {
  outline: none;
  border-color: var(--color-primary);
}

.pal-search {
  flex: none;
  padding: calc(var(--space-unit) * 1);
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
