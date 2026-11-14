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
const noResults = computed(() => paletteItems.value.length === 0 && paletteWorksState.value === 'ready')

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
  const stops = Array.from(root.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled])'))
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
    @keydown.esc="dispatch('palette.close')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="pal-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="palette-title">
      <header class="pal-head">
        <h2 id="palette-title" class="pal-title">{{ t('palette.title') }}</h2>
        <button type="button" class="pal-close ui-btn ui-btn--quiet" @click="dispatch('palette.close')">
          {{ t('command.palette.close') }}
        </button>
      </header>
      <input
        ref="input"
        class="pal-input ui-field ui-field--full"
        type="text"
        role="combobox"
        aria-controls="palette-list"
        aria-expanded="true"
        aria-autocomplete="list"
        :aria-activedescendant="activeId"
        :aria-label="t('palette.input_label')"
        :placeholder="t('palette.input_placeholder')"
        :value="paletteQuery"
        autocomplete="off"
        @input="onInput($event)"
        @keydown="onInputKeydown($event)"
      />
      <p v-if="paletteWorksState === 'loading'" class="pal-status" role="status">{{ t('palette.works_loading') }}</p>
      <p v-else-if="paletteWorksState === 'error'" class="pal-status" role="alert">
        {{ t('palette.works_error') }}
        <!-- aura-allow-text: result of tError() on the Library load failure. -->
        <span>{{ paletteWorksErrorText }}</span>
      </p>
      <p v-else-if="paletteWorksAreFiltered" class="pal-status" role="status">
        {{ t('palette.works_filtered', { matched_count: String(libraryWorksMatched), total_count: String(libraryWorksTotal) }) }}
      </p>
      <div id="palette-list" class="pal-list" role="listbox" :aria-label="t('palette.title')">
        <div v-if="paletteWorkItems.length > 0" class="pal-group" role="group" aria-labelledby="palette-group-works">
          <h3 id="palette-group-works" class="pal-group-title">{{ t('palette.group_works') }}</h3>
          <div
            v-for="(item, i) in paletteWorkItems"
            :id="optionId(i)"
            :key="item.key"
            class="pal-option"
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
            class="pal-option"
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
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

:root[data-platform='mac'] .pal-scrim {
  padding-top: var(--space-titlebar-height);
}

.pal-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-panel-block);
  width: 100%;
  max-width: 637px;
  max-height: 100%;
  padding: 13px; /* aura-allow-spacing: 1px border sits inside the 14px inset */
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
  color: var(--color-on-surface);
}

.pal-panel:focus-visible {
  outline: none;
  border-color: var(--color-primary);
}

.pal-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
}

.pal-title {
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-md);
}

.pal-status {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pal-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-panel-block);
  min-height: 0;
  overflow-y: auto;
}

.pal-group-title {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pal-option {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

.pal-option-current {
  background: var(--color-surface-accent);
}

.pal-hint {
  flex: none;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}
</style>
