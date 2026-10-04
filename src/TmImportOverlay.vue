<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from 'vue'
import { t } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import { tmExchangeErrorText } from './tmExchangeError'
import {
  tmImportConfirmError,
  tmImportConfirmUnavailable,
  tmImportConfirming,
  tmImportLoadError,
  tmImportOverlayIsOpen,
  tmImportPreview,
  tmImportStatus,
} from './tmImportState'

let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

watch(tmImportOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-tm-import-open]')
    void nextTick(() => panel.value?.focus())
    return
  }
  const back = returnFocusTo
  returnFocusTo = null
  if (back !== null && back.isConnected) {
    back.focus()
    return
  }
  const opener = document.querySelector<HTMLElement>('[data-tm-import-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[tm-import] focus-return target is gone; focus falls back to body.')
})

function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
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
    v-if="tmImportOverlayIsOpen"
    class="ti-scrim"
    @keydown.esc="dispatch('tm.import.cancel')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="ti-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="ti-title">
      <header class="ti-head">
        <h2 id="ti-title" class="ti-title">{{ t('tm.import.title') }}</h2>
        <button type="button" class="ti-close" @click="dispatch('tm.import.cancel')">
          {{ t('command.tm.import.cancel') }}
        </button>
      </header>

      <p v-if="tmImportStatus === 'unknown'" class="ti-status" role="status">{{ t('tm.import.loading') }}</p>
      <p v-else-if="tmImportStatus === 'ipc_unavailable'" class="ti-empty">
        {{ t('tm.import.empty_ipc_unavailable') }}
      </p>
      <p v-else-if="tmImportStatus === 'error' && tmImportLoadError !== null" class="ti-empty ti-error" role="alert">
        <!-- aura-allow-text: result of tmExchangeErrorText() computed in the script. -->
        {{ tmExchangeErrorText(tmImportLoadError) }}
      </p>
      <template v-else-if="tmImportStatus === 'loaded' && tmImportPreview !== null">
        <dl class="ti-summary">
          <div class="ti-summary-row">
            <dt>{{ t('tm.import.file_name_label') }}</dt>
            <!-- aura-allow-text: data (file name chosen by the user). -->
            <dd>{{ tmImportPreview.file_name }}</dd>
          </div>
          <div class="ti-summary-row">
            <dt>{{ t('tm.import.tier_label') }}</dt>
            <dd>{{ t(tmImportPreview.tier === 'global' ? 'tm.fuzzy.tier_global' : 'tm.fuzzy.tier_work') }}</dd>
          </div>
          <div class="ti-summary-row">
            <dt>{{ t('tm.import.units_label') }}</dt>
            <!-- aura-allow-text: data (a count read from the validated preview). -->
            <dd>{{ tmImportPreview.unit_count }}</dd>
          </div>
        </dl>
        <p v-if="tmImportPreview.tier === 'global'" class="ti-note">{{ t('tm.exchange.global_note') }}</p>
        <p class="ti-note">{{ t('tm.import.origin_note') }}</p>
        <ul class="ti-counts">
          <!-- aura-allow-text: result of t() with the counts interpolated. -->
          <li>{{ t('tm.import.new_count', { count: String(tmImportPreview.new_count) }) }}</li>
          <li>{{ t('tm.import.already_count', { count: String(tmImportPreview.already_count) }) }}</li>
          <li>{{ t('tm.import.skipped_count', { count: String(tmImportPreview.skipped_count) }) }}</li>
        </ul>
        <p v-if="tmImportPreview.new_count === 0" class="ti-empty">{{ t('tm.import.nothing_new') }}</p>

        <p v-if="tmImportConfirmError !== null" class="ti-status ti-error" role="alert">
          <!-- aura-allow-text: result of tmExchangeErrorText() computed in the script. -->
          {{ tmExchangeErrorText(tmImportConfirmError) }}
        </p>
        <p v-else-if="tmImportConfirmUnavailable" class="ti-status" role="status">
          {{ t('tm.import.confirm_ipc_unavailable') }}
        </p>
        <p v-else-if="tmImportConfirming" class="ti-status" role="status">{{ t('tm.import.confirming') }}</p>
        <p class="ti-hint">{{ t('tm.import.hint_no_write_before_confirm') }}</p>

        <div class="ti-actions">
          <button
            type="button"
            class="ti-act ti-act-primary"
            :disabled="tmImportConfirming"
            @click="dispatch('tm.import.confirm')"
          >
            <!-- aura-allow-text: result of t() with the count interpolated. -->
            {{ t('tm.import.confirm_button', { count: String(tmImportPreview.new_count) }) }}
          </button>
          <button type="button" class="ti-act" :disabled="tmImportConfirming" @click="dispatch('tm.import.cancel')">
            {{ t('command.tm.import.cancel') }}
          </button>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.ti-scrim {
  position: fixed;
  inset: 0;
  z-index: 11; /* aura-allow-z-index: stacks above TmManageOverlay (10); opened from inside it. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.ti-panel {
  width: 100%;
  max-width: 720px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.ti-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.ti-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.ti-close {
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.ti-status {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ti-empty {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.ti-error {
  color: var(--color-error);
}

.ti-summary {
  margin: 0 0 var(--space-panel-block) 0;
}

.ti-summary-row {
  display: flex;
  gap: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
}

.ti-summary-row dt {
  color: var(--color-on-surface-variant);
  min-width: 8rem;
}

.ti-summary-row dd {
  margin: 0;
  color: var(--color-on-surface);
}

.ti-note {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ti-counts {
  list-style: none;
  padding: 0;
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.ti-hint {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ti-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}

.ti-act {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.ti-act-primary {
  background: var(--color-primary);
  color: var(--color-on-primary);
  border-color: var(--color-primary);
}

.ti-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}
</style>
