<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import {
  conventionImportBlocked,
  conventionImportConfirmError,
  conventionImportConfirming,
  conventionImportCursor,
  conventionImportDecisionFor,
  conventionImportIsOpen,
  conventionImportLoadError,
  conventionImportPreview,
  conventionImportSummary,
  setConventionImportCursor,
  stepConventionImportCursor,
} from './conventionImportState'

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

let returnFocusTo: HTMLElement | null = null

watch(conventionImportIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-convention-import-open]')
    void nextTick(() => panel.value?.focus())
    return
  }
  const back = returnFocusTo
  returnFocusTo = null
  if (back !== null && back.isConnected) {
    back.focus()
    return
  }
  document.querySelector<HTMLElement>('[data-convention-import-open]')?.focus()
})

function focusDecisionOfCursor(): void {
  void nextTick(() => {
    const row = panel.value?.querySelector<HTMLElement>('[data-convention-import-row][aria-selected="true"]')
    const keep = row?.querySelector<HTMLElement>('[aria-pressed="true"]') ?? row?.querySelector<HTMLElement>('button')
    keep?.focus()
  })
}

function aimRow(index: number): void {
  setConventionImportCursor(index)
}

function onListKeydown(event: KeyboardEvent): void {
  if (event.ctrlKey || event.metaKey || event.altKey) return
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    stepConventionImportCursor(1)
    focusDecisionOfCursor()
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    stepConventionImportCursor(-1)
    focusDecisionOfCursor()
  }
}

function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>('button:not([disabled]):not([tabindex="-1"])'))
}

function trapTab(event: KeyboardEvent): void {
  const root = panel.value
  if (root === null) return
  event.preventDefault()
  const stops = focusableWithin(root)
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
    v-if="conventionImportIsOpen"
    class="cvi-scrim"
    @keydown.esc="dispatch('convention.import.cancel')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="cvi-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="cvi-title">
      <header class="cvi-head">
        <h2 id="cvi-title" class="cvi-title">{{ t('convention.manage.import') }}</h2>
        <button
          type="button"
          class="ui-btn ui-btn--quiet"
          :disabled="conventionImportConfirming"
          @click="dispatch('convention.import.cancel')"
        >
          {{ t('convention.manage.import_cancel') }}
        </button>
      </header>

      <p v-if="conventionImportLoadError !== null" class="cvi-note cvi-error" role="alert">
        <!-- aura-allow-text: result of tError(). -->
        {{ tError(conventionImportLoadError) }}
      </p>

      <template v-else-if="conventionImportPreview !== null">
        <!-- aura-allow-text: the file name the user picked, shown as text only. -->
        <p class="cvi-file">{{ conventionImportPreview.file_name }}</p>
        <p class="cvi-note" role="status">
          <!-- aura-allow-text: result of t() with counts from Rust. -->
          {{
            t('convention.manage.import_summary', {
              new_count: String(conventionImportPreview.new_count),
              same_count: String(conventionImportPreview.identical_count),
              conflict_count: String(conventionImportPreview.conflicts.length),
            })
          }}
        </p>

        <ul v-if="conventionImportPreview.conflicts.length > 0" class="cvi-list" @keydown="onListKeydown($event)">
          <li
            v-for="(row, i) in conventionImportPreview.conflicts"
            :key="row.rule_key"
            class="cvi-row"
            :class="{ 'cvi-row-current': i === conventionImportCursor }"
            data-convention-import-row
            :aria-selected="i === conventionImportCursor"
            @mousedown="aimRow(i)"
            @focusin="aimRow(i)"
          >
            <span class="cvi-rule">
              <!-- aura-allow-text: text from the list, shown as text only. -->
              {{ row.existing_avoid_phrase }}
              <span aria-hidden="true">→</span>
              <!-- aura-allow-text: text from the list, shown as text only. -->
              <template v-if="row.existing_replacement_phrase !== null">{{ row.existing_replacement_phrase }}</template>
              <span v-else class="cvi-none">{{ t('convention.manage.replacement_none') }}</span>
            </span>
            <span class="cvi-rule cvi-rule-file">
              <!-- aura-allow-text: text from the file, shown as text only. -->
              {{ row.file_avoid_phrase }}
              <span aria-hidden="true">→</span>
              <!-- aura-allow-text: text from the file, shown as text only. -->
              <template v-if="row.file_replacement_phrase !== null">{{ row.file_replacement_phrase }}</template>
              <span v-else class="cvi-none">{{ t('convention.manage.replacement_none') }}</span>
            </span>
            <span class="cvi-choice">
              <button
                type="button"
                class="ui-btn"
                data-convention-import-keep
                :aria-pressed="conventionImportDecisionFor(row.rule_key) === 'keep_mine'"
                :tabindex="i === conventionImportCursor ? 0 : -1"
                :disabled="conventionImportConfirming"
                @click="dispatch('convention.import.keep_mine')"
              >
                {{ t('convention.manage.import_keep_mine') }}
              </button>
              <button
                type="button"
                class="ui-btn"
                data-convention-import-take
                :aria-pressed="conventionImportDecisionFor(row.rule_key) === 'take_file'"
                :tabindex="i === conventionImportCursor ? 0 : -1"
                :disabled="conventionImportConfirming"
                @click="dispatch('convention.import.take_file')"
              >
                {{ t('convention.manage.import_take_file') }}
              </button>
            </span>
          </li>
        </ul>

        <p v-if="conventionImportConfirmError !== null" class="cvi-note cvi-error" role="alert">
          <!-- aura-allow-text: result of tError(). -->
          {{ tError(conventionImportConfirmError) }}
        </p>

        <div class="cvi-actions">
          <button
            type="button"
            class="ui-btn ui-btn--primary"
            data-convention-import-apply
            :disabled="conventionImportConfirming || conventionImportBlocked"
            @click="dispatch('convention.import.confirm')"
          >
            <!-- aura-allow-text: result of t() with counts. -->
            {{ t('convention.manage.import_apply', conventionImportSummary) }}
          </button>
          <button
            type="button"
            class="ui-btn"
            :disabled="conventionImportConfirming"
            @click="dispatch('convention.import.cancel')"
          >
            {{ t('convention.manage.import_cancel') }}
          </button>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.cvi-scrim {
  position: fixed;
  inset: 0;
  z-index: 11; /* aura-allow-z-index: above ConventionManageOverlay (10), opened from inside it. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

:root[data-platform='mac'] .cvi-scrim {
  padding-top: var(--space-titlebar-height);
}

.cvi-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-panel-block);
  width: 100%;
  max-width: 721px;
  max-height: 100%;
  overflow: auto;
  padding: 13px; /* aura-allow-spacing: 1px border sits inside the 14px inset */
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.cvi-panel:focus-visible {
  outline: none;
  border-color: var(--color-primary);
}

.cvi-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
}

.cvi-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
}

.cvi-file,
.cvi-note {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.cvi-error {
  color: var(--color-error);
}

.cvi-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
}

.cvi-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
  padding: 6px; /* aura-allow-spacing: 1px border sits inside the 7px inset */
  border: 1px solid var(--color-outline);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.cvi-row-current {
  border-color: var(--color-primary);
}

.cvi-rule-file,
.cvi-none {
  color: var(--color-on-surface-variant);
}

.cvi-choice {
  display: flex;
  gap: calc(var(--space-unit) * 1);
  margin-left: auto;
}

.cvi-actions {
  display: flex;
  gap: calc(var(--space-unit) * 1);
}
</style>
