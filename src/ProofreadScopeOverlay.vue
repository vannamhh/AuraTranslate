<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from 'vue'
import { t } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { proofreadScopeCounts } from './proofreadScope'
import { proofreadScopeIsOpen } from './proofreadState'

const panel = useTemplateRef<HTMLElement>('panel')

let returnFocusTo: HTMLElement | null = null

watch(proofreadScopeIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-proofread-scope-open]')
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
  document.querySelector<HTMLElement>('[data-proofread-scope-open]')?.focus()
})

function onEscape(): void {
  dispatch('ai.proofread.scope.close')
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
    v-if="proofreadScopeIsOpen"
    class="pfscope-scrim"
    @keydown.esc="onEscape()"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="pfscope-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="pfscope-title">
      <header class="pfscope-head">
        <h2 id="pfscope-title" class="pfscope-title">{{ t('proofread.scope.title') }}</h2>
        <button type="button" class="pfscope-close" @click="dispatch('ai.proofread.scope.close')">
          {{ t('command.ai.proofread.scope.close') }}
        </button>
      </header>
      <div class="pfscope-cells">
        <button
          type="button"
          class="pfscope-cell"
          data-proofread-scope="segment"
          :disabled="proofreadScopeCounts.segment === 0"
          @click="dispatch('ai.proofread.run')"
        >
          <span class="pfscope-label">{{ t('proofread.scope.segment') }}</span>
          <span class="pfscope-count">{{
            t('proofread.scope.count', { sentence_count: String(proofreadScopeCounts.segment) })
          }}</span>
        </button>
        <button
          type="button"
          class="pfscope-cell"
          data-proofread-scope="selection"
          :disabled="proofreadScopeCounts.selection === 0"
          @click="dispatch('ai.proofread.run_selection')"
        >
          <span class="pfscope-label">{{ t('proofread.scope.selection') }}</span>
          <span class="pfscope-count">{{
            t('proofread.scope.count', { sentence_count: String(proofreadScopeCounts.selection) })
          }}</span>
        </button>
        <button
          type="button"
          class="pfscope-cell"
          data-proofread-scope="chapter"
          :disabled="proofreadScopeCounts.chapter === 0"
          @click="dispatch('ai.proofread.run_chapter')"
        >
          <span class="pfscope-label">{{ t('proofread.scope.chapter') }}</span>
          <span class="pfscope-count">{{
            t('proofread.scope.count', { sentence_count: String(proofreadScopeCounts.chapter) })
          }}</span>
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.pfscope-scrim {
  position: fixed;
  inset: 0;
  /* aura-allow-z-index: xếp lớp CƠ HỌC — dockview dựng ngữ cảnh xếp lớp riêng cho mỗi nhóm panel, nên thứ tự tài liệu một mình không đủ để lớp phủ nằm TRÊN lưới. */
  z-index: 10;
  display: flex;
  justify-content: flex-end;
  align-items: flex-start;
  padding: var(--space-panel-inline);
}

.pfscope-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-panel-block);
  min-width: 320px;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
  color: var(--color-on-surface);
}

.pfscope-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
}

.pfscope-title {
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-md);
}

.pfscope-close {
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

.pfscope-cells {
  display: flex;
  gap: var(--space-panel-inline);
}

.pfscope-cell {
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: flex-start;
  gap: calc(var(--space-unit) * 1);
  padding: calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  color: var(--color-on-surface);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

.pfscope-cell:disabled {
  color: var(--color-on-surface-variant);
  cursor: default;
}

.pfscope-count {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}
</style>
