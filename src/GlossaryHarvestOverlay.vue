<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import { isInconsistentHarvest } from './glossaryQueueState'
import {
  harvestBusy,
  harvestCursor,
  harvestLoadError,
  harvestOverlayIsOpen,
  harvestRows,
  harvestSelectedCount,
  harvestStatus,
  setGlossaryHarvestCursor,
  setGlossaryHarvestSelected,
} from './glossaryHarvestState'

let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

const allDecided = computed(() => harvestRows.value.length > 0 && harvestRows.value.every((r) => r.outcome !== null))

watch(harvestBusy, (busy) => {
  if (busy) return
  void nextTick(() => {
    const root = panel.value
    if (root !== null && harvestOverlayIsOpen.value && !root.contains(document.activeElement)) root.focus()
  })
})

watch(harvestOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-glossary-harvest-open]')
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-glossary-harvest-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[glossary-harvest] focus-return target is gone; focus falls back to body.')
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

function onToggle(index: number, event: Event): void {
  if (event.target instanceof HTMLInputElement) setGlossaryHarvestSelected(index, event.target.checked)
}

function aimRow(index: number): void {
  setGlossaryHarvestCursor(index)
}

// A modifier chord must never act as a bare N/B: accepting is a deliberate press (AD-20).
function onKeydown(event: KeyboardEvent): void {
  if (event.ctrlKey || event.metaKey || event.altKey) return

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      dispatch('glossary.harvest.next')
      return
    case 'ArrowUp':
      event.preventDefault()
      dispatch('glossary.harvest.prev')
      return
    case 'n':
    case 'N':
      event.preventDefault()
      dispatch('glossary.harvest.accept')
      return
    case 'b':
    case 'B':
      event.preventDefault()
      dispatch('glossary.harvest.reject')
      return
    default:
      return
  }
}
</script>

<template>
  <div
    v-if="harvestOverlayIsOpen"
    class="gh-scrim"
    @keydown.esc="dispatch('glossary.harvest.close')"
    @keydown.tab="trapTab($event)"
    @keydown="onKeydown"
  >
    <section ref="panel" class="gh-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="gh-head">
        <h2 class="gh-title">{{ t('glossary.harvest.title') }}</h2>
        <button type="button" class="gh-close" @click="dispatch('glossary.harvest.close')">
          {{ t('command.glossary.queue.close') }}
        </button>
      </header>
      <p class="gh-subtitle">{{ t('glossary.harvest.subtitle') }}</p>

      <p v-if="harvestStatus === 'unknown'" class="gh-status" role="status">{{ t('glossary.harvest.loading') }}</p>
      <p v-else-if="harvestStatus === 'ipc_unavailable'" class="gh-empty">
        {{ t('glossary.harvest.empty_ipc_unavailable') }}
      </p>
      <p v-else-if="harvestStatus === 'error' && harvestLoadError !== null" class="gh-empty gh-error" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(harvestLoadError) }}
      </p>
      <p v-else-if="harvestStatus === 'no_work'" class="gh-empty">{{ t('glossary.harvest.empty_no_work') }}</p>
      <p v-else-if="harvestRows.length === 0" class="gh-empty">{{ t('glossary.harvest.empty') }}</p>

      <template v-else>
        <ul class="gh-list">
          <li
            v-for="(row, i) in harvestRows"
            :key="row.candidate.id"
            class="gh-row"
            :class="{
              'gh-row-current': i === harvestCursor,
              'gh-row-done': row.outcome !== null,
              'gh-row-accepted': row.outcome === 'accepted',
              'gh-row-rejected': row.outcome === 'rejected',
              'gh-row-inconsistent': isInconsistentHarvest(row.candidate),
            }"
            @mousedown="aimRow(i)"
            @focusin="aimRow(i)"
          >
            <input
              type="checkbox"
              class="gh-check"
              :checked="row.selected"
              :disabled="row.outcome !== null || harvestBusy"
              :aria-label="t('glossary.harvest.select_row', { term: row.candidate.replaced_translation ?? row.candidate.source_term })"
              @change="onToggle(i, $event)"
            />
            <!-- aura-allow-text: DỮ LIỆU (X → Y của chính hàng). -->
            <span class="gh-term">{{ row.candidate.replaced_translation ?? row.candidate.source_term }}</span>
            <span class="gh-arrow" aria-hidden="true">→</span>
            <!-- aura-allow-text: DỮ LIỆU (bản dịch đề xuất của chính hàng). -->
            <span class="gh-term gh-term-to">{{ row.candidate.proposed_translation ?? '' }}</span>
            <!-- aura-allow-text: KẾT QUẢ của `t()` cộng hai con số đếm. -->
            <span class="gh-count">{{
              t(isInconsistentHarvest(row.candidate) ? 'glossary.harvest.change_inconsistent' : 'glossary.harvest.change', {
                changed: String(row.candidate.changed_count ?? 0),
                seen: String(row.candidate.seen_count ?? 0),
              })
            }}</span>
            <span v-if="row.outcome === null" class="gh-keys">
              <button
                type="button"
                class="gh-mini gh-mini-primary"
                :disabled="harvestBusy"
                @click="dispatch('glossary.harvest.accept')"
              >
                {{ t('glossary.harvest.accept_row') }} <!-- aura-allow-text: tên phím. --><kbd>N</kbd>
              </button>
              <button type="button" class="gh-mini" :disabled="harvestBusy" @click="dispatch('glossary.harvest.reject')">
                {{ t('glossary.harvest.reject_row') }} <!-- aura-allow-text: tên phím. --><kbd>B</kbd>
              </button>
            </span>
            <!-- aura-allow-text: dấu ✓/✕ trang trí, trạng thái bằng chữ đã có ở span sr-only. -->
            <span v-else class="gh-mark" aria-hidden="true">{{ row.outcome === 'accepted' ? '✓' : '✕' }}</span>
            <!-- aura-allow-text: KẾT QUẢ của `t()`; trạng thái luôn có chữ. -->
            <span v-if="row.outcome !== null" class="gh-sr-only">{{
              row.outcome === 'accepted' ? t('glossary.harvest.row_status_accepted') : t('glossary.harvest.row_status_rejected')
            }}</span>
            <p v-if="row.error !== null" class="gh-row-error" role="alert">
              <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
              {{ tError(row.error) }}
            </p>
          </li>
        </ul>

        <p v-if="allDecided" class="gh-status" role="status">{{ t('glossary.harvest.all_decided') }}</p>
        <p v-else-if="harvestBusy" class="gh-status" role="status">{{ t('glossary.harvest.saving') }}</p>

        <div class="gh-actions">
          <button
            type="button"
            class="gh-act gh-act-primary"
            :disabled="harvestBusy || harvestSelectedCount === 0"
            @click="dispatch('glossary.harvest.accept_selected')"
          >
            <!-- aura-allow-text: KẾT QUẢ của `t()` cộng một con số đếm. -->
            {{ t('glossary.harvest.accept_selected', { count: String(harvestSelectedCount) }) }}
          </button>
          <button type="button" class="gh-act" @click="dispatch('glossary.harvest.close')">
            {{ t('glossary.harvest.leave') }}
          </button>
        </div>
        <p class="gh-hint">{{ t('glossary.harvest.hint') }}</p>
      </template>
    </section>
  </div>
</template>

<style scoped>
.gh-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: xếp lớp CƠ HỌC — cùng lý do các lớp phủ khác. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.gh-panel {
  width: 100%;
  max-width: 720px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.gh-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.gh-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.gh-close {
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

.gh-subtitle,
.gh-empty,
.gh-status,
.gh-hint {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.gh-error,
.gh-row-error {
  color: var(--color-error);
}

.gh-list {
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
  border: 1px solid var(--color-outline);
  max-height: 60vh;
  overflow: auto;
}

.gh-row {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 3);
  padding: calc(var(--space-unit) * 2) var(--space-panel-inline);
  border-bottom: 1px solid var(--color-outline);
  border-left: 2px solid transparent;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.gh-row:last-child {
  border-bottom: none;
}

.gh-row-current {
  background: var(--color-surface-accent);
  border-left: 3px solid var(--color-primary);
}

.gh-row-accepted {
  border-left: 2px solid var(--color-confirmed);
}

.gh-row-rejected {
  border-left: 2px solid var(--color-ornament);
}

.gh-row-done {
  color: var(--color-on-surface-variant);
}

.gh-term {
  font-family: var(--family-read);
}

.gh-term-to {
  color: var(--color-primary);
}

.gh-row-done .gh-term-to {
  color: var(--color-on-surface-variant);
}

.gh-count {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gh-row-inconsistent .gh-count {
  color: var(--color-tm-text);
}

.gh-keys {
  display: inline-flex;
  gap: calc(var(--space-unit) * 2);
  margin-left: auto;
}

.gh-mini {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  background: none;
  border: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
}

.gh-mini-primary {
  border-color: var(--color-primary);
}

.gh-mini kbd {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface-variant);
}

.gh-mark {
  margin-left: auto;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
}

.gh-row-error {
  flex-basis: 100%;
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
}

.gh-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.gh-act {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: none;
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  cursor: pointer;
}

.gh-act-primary {
  border-color: var(--color-primary);
}

.gh-sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  margin: -1px; /* aura-allow-spacing: kích thước hình học của kỹ thuật ẩn sr-only, không phải khoảng cách lưới */
  padding: 0;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
</style>
