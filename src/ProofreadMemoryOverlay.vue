<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import {
  proofreadMemoryBusy,
  proofreadMemoryCleared,
  proofreadMemoryClearPending,
  proofreadMemoryError,
  proofreadMemoryHasLoaded,
  proofreadMemoryIsOpen,
  proofreadMemoryRows,
  proofreadMemorySelectedId,
  selectProofreadMemoryRow,
} from './proofreadMemoryState'
import ProofreadDecisionRatio from './ProofreadDecisionRatio.vue'
import { reloadProofreadDecisions } from './proofreadDecisionState'
import { shortDayMonth } from './panels/shortDate'
import type { ProofreadFindingKind } from './config/proofread'

const panel = useTemplateRef<HTMLElement>('panel')

let returnFocusTo: HTMLElement | null = null

watch(proofreadMemoryIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-proofread-memory-open]')
    void reloadProofreadDecisions()
    void nextTick(() => {
      const first = panel.value?.querySelector<HTMLElement>('button:not([disabled])')
      ;(first ?? panel.value)?.focus()
    })
    return
  }

  const back = returnFocusTo
  returnFocusTo = null
  if (back !== null && back.isConnected) {
    back.focus()
    return
  }
  document.querySelector<HTMLElement>('[data-proofread-memory-open]')?.focus()
})

watch(proofreadMemoryBusy, (busy) => {
  if (busy || !proofreadMemoryIsOpen.value) return
  void nextTick(() => {
    const root = panel.value
    if (root === null || root.contains(document.activeElement)) return
    const first = root.querySelector<HTMLElement>('button:not([disabled])')
    ;(first ?? root).focus()
  })
})

const KIND_LABELS: Readonly<Record<ProofreadFindingKind, () => string>> = {
  spelling: () => t('proofread.kind.spelling'),
  grammar: () => t('proofread.kind.grammar'),
  mistranslation: () => t('proofread.kind.mistranslation'),
  too_loose: () => t('proofread.kind.too_loose'),
  obscure: () => t('proofread.kind.obscure'),
}

function metaOf(row: { created_at: string; blocked_count: number | null }): string {
  const parts: string[] = []
  const date = shortDayMonth(row.created_at)
  if (date !== null) parts.push(t('proofread.memory.meta_date', { date }))
  if (row.blocked_count !== null) parts.push(t('proofread.memory.meta_blocked', { blocked_count: String(row.blocked_count) }))
  return parts.join(' · ')
}

const lines = computed(() =>
  proofreadMemoryRows.value.map((row) => ({
    id: row.id,
    meta: metaOf(row),
    kind: KIND_LABELS[row.kind](),
    text:
      row.context_before === '' && row.context_after === ''
        ? row.phrase
        : t('proofread.memory.context', {
            context_before: row.context_before,
            phrase: row.phrase,
            context_after: row.context_after,
          }),
  })),
)

const count = computed(() => String(proofreadMemoryRows.value.length))

const clearText = computed(() =>
  proofreadMemoryClearPending.value
    ? t('proofread.memory.clear_confirm', { memory_count: count.value })
    : t('command.ai.proofread.memory.clear'),
)

const notice = computed<string | null>(() =>
  proofreadMemoryCleared.value === null
    ? null
    : t('proofread.memory.cleared', { memory_count: String(proofreadMemoryCleared.value) }),
)

const errorText = computed<string | null>(() =>
  proofreadMemoryError.value === null ? null : tError(proofreadMemoryError.value),
)

function onEscape(): void {
  dispatch('ai.proofread.memory.close')
}

function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>('button:not([disabled])'))
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
    v-if="proofreadMemoryIsOpen"
    class="pfmem-scrim"
    @keydown.esc="onEscape()"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="pfmem-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="pfmem-title">
      <header class="pfmem-head">
        <h2 id="pfmem-title" class="pfmem-title">{{ t('proofread.memory.title') }}</h2>
        <button type="button" class="pfmem-close" @click="dispatch('ai.proofread.memory.close')">
          {{ t('command.ai.proofread.memory.close') }}
        </button>
      </header>

      <ProofreadDecisionRatio :active="proofreadMemoryIsOpen" />

      <p v-if="!proofreadMemoryHasLoaded && proofreadMemoryBusy" class="pfmem-note" role="status">
        {{ t('proofread.memory.loading') }}
      </p>
      <p v-else-if="proofreadMemoryHasLoaded && lines.length === 0" class="pfmem-note" role="status">
        {{ t('proofread.memory.empty') }}
      </p>
      <ul v-else class="pfmem-list" :aria-label="t('proofread.memory.list_label')">
        <li v-for="line in lines" :key="line.id">
          <button
            type="button"
            class="pfmem-row"
            data-proofread-memory-row
            :aria-pressed="proofreadMemorySelectedId === line.id"
            @pointerdown="selectProofreadMemoryRow(line.id)"
            @focus="selectProofreadMemoryRow(line.id)"
          >
            <!-- aura-allow-text: result of t() computed in the script. -->
            <span class="pfmem-kind">{{ line.kind }}</span>
            <!-- aura-allow-text: text the user marked, shown as text only. -->
            <span class="pfmem-text">{{ line.text }}</span>
            <!-- aura-allow-text: result of t() computed in the script. -->
            <span v-if="line.meta !== ''" class="pfmem-meta">{{ line.meta }}</span>
          </button>
        </li>
      </ul>

      <p v-if="proofreadMemoryClearPending" class="pfmem-note" role="status">
        {{ t('proofread.memory.clear_hint', { memory_count: count }) }}
      </p>
      <p v-if="notice !== null" class="pfmem-note" role="status">
        <!-- aura-allow-text: result of t() computed in the script. -->
        {{ notice }}
      </p>
      <p v-if="errorText !== null" class="pfmem-note pfmem-error" role="alert">
        <!-- aura-allow-text: result of tError(). -->
        {{ errorText }}
      </p>

      <div class="pfmem-actions">
        <button
          type="button"
          class="pfmem-act"
          data-proofread-memory-forget
          :disabled="proofreadMemoryBusy || proofreadMemorySelectedId === null"
          @click="dispatch('ai.proofread.memory.forget')"
        >
          {{ t('command.ai.proofread.memory.forget') }}
        </button>
        <button
          type="button"
          class="pfmem-act"
          data-proofread-memory-clear
          :disabled="proofreadMemoryBusy || lines.length === 0"
          @click="dispatch('ai.proofread.memory.clear')"
        >
          <!-- aura-allow-text: result of t() computed in the script. -->
          {{ clearText }}
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.pfmem-scrim {
  position: fixed;
  inset: 0;
  /* aura-allow-z-index: xếp lớp CƠ HỌC — dockview dựng ngữ cảnh xếp lớp riêng cho mỗi nhóm panel, nên thứ tự tài liệu một mình không đủ để lớp phủ nằm TRÊN lưới. */
  z-index: 10;
  display: flex;
  justify-content: flex-end;
  align-items: flex-start;
  padding: var(--space-panel-inline);
}

.pfmem-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-panel-block);
  min-width: 360px;
  max-width: 560px;
  max-height: 80vh;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
  color: var(--color-on-surface);
}

.pfmem-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
}

.pfmem-title {
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-md);
}

.pfmem-close {
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

.pfmem-note {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pfmem-error {
  color: var(--color-error);
}

.pfmem-list {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin: 0;
  padding: 0;
  list-style: none;
  overflow-y: auto;
}

.pfmem-row {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 3);
  width: 100%;
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  color: var(--color-on-surface);
  cursor: pointer;
  text-align: left;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

.pfmem-row[aria-pressed='true'] {
  border-left: 3px solid var(--color-error);
}

.pfmem-kind {
  flex: none;
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-error);
}

.pfmem-text {
  overflow-wrap: anywhere;
}

.pfmem-meta {
  flex: none;
  margin-left: auto;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pfmem-actions {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
}

.pfmem-act {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  color: var(--color-on-surface);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

.pfmem-act:disabled {
  color: var(--color-on-surface-variant);
  cursor: default;
}
</style>
