<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import {
  conventionAvoidDraft,
  conventionManageBusy,
  conventionManageError,
  conventionManageHasLoaded,
  conventionManageIsOpen,
  conventionManageRules,
  conventionReplacementDraft,
} from './conventionManageState'

const panel = useTemplateRef<HTMLElement>('panel')
const avoidInput = useTemplateRef<HTMLInputElement>('avoidInput')

let returnFocusTo: HTMLElement | null = null

watch(conventionManageIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-convention-manage-open]')
    void nextTick(() => {
      ;(avoidInput.value ?? panel.value)?.focus()
    })
    return
  }
  const back = returnFocusTo
  returnFocusTo = null
  if (back !== null && back.isConnected) {
    back.focus()
    return
  }
  document.querySelector<HTMLElement>('[data-convention-manage-open]')?.focus()
})

const rows = computed(() =>
  conventionManageRules.value.map((rule) => ({
    id: rule.id,
    avoid: rule.avoid_phrase,
    replacement: rule.replacement_phrase,
  })),
)

const errorText = computed<string | null>(() =>
  conventionManageError.value === null ? null : tError(conventionManageError.value),
)

function onEscape(): void {
  dispatch('convention.manage.close')
}

function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled])'))
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
    v-if="conventionManageIsOpen"
    class="cvm-scrim"
    @keydown.esc="onEscape()"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="cvm-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="cvm-title">
      <header class="cvm-head">
        <h2 id="cvm-title" class="cvm-title">{{ t('convention.manage.title') }}</h2>
        <button type="button" class="cvm-close ui-btn ui-btn--quiet" data-convention-manage-close @click="dispatch('convention.manage.close')">
          {{ t('convention.manage.close') }}
        </button>
      </header>

      <p class="cvm-tier">
        <!-- aura-allow-text: result of t() with a count from Rust. -->
        {{ t('convention.manage.tier_global', { rule_count: String(rows.length) }) }}
      </p>

      <div class="cvm-add" role="group" :aria-label="t('convention.manage.add')">
        <label class="cvm-field">
          <span class="cvm-label">{{ t('convention.manage.add_avoid') }}</span>
          <input
            ref="avoidInput"
            v-model="conventionAvoidDraft"
            type="text"
            class="cvm-input ui-field"
            autocomplete="off"
            data-convention-add-avoid
            @keydown.enter.prevent="dispatch('convention.manage.add')"
          />
        </label>
        <label class="cvm-field">
          <span class="cvm-label">{{ t('convention.manage.add_replacement') }}</span>
          <input
            v-model="conventionReplacementDraft"
            type="text"
            class="cvm-input ui-field"
            autocomplete="off"
            data-convention-add-replacement
            @keydown.enter.prevent="dispatch('convention.manage.add')"
          />
        </label>
        <button
          type="button"
          class="cvm-add-btn ui-btn"
          data-convention-add
          :disabled="conventionManageBusy"
          @click="dispatch('convention.manage.add')"
        >
          {{ t('convention.manage.add') }}
        </button>
      </div>

      <p v-if="errorText !== null" class="cvm-note cvm-error" role="alert">
        <!-- aura-allow-text: result of tError(). -->
        {{ errorText }}
      </p>
      <p v-if="!conventionManageHasLoaded && conventionManageError === null" class="cvm-note" role="status">
        {{ t('convention.manage.loading') }}
      </p>
      <p v-else-if="conventionManageHasLoaded && rows.length === 0" class="cvm-note" role="status">
        {{ t('convention.manage.empty_rules') }}
      </p>
      <table v-else-if="rows.length > 0" class="cvm-table">
        <thead>
          <tr>
            <th scope="col">{{ t('convention.manage.col_avoid') }}</th>
            <th scope="col">{{ t('convention.manage.col_replacement') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in rows" :key="row.id" data-convention-rule-row>
            <!-- aura-allow-text: text the user typed, shown as text only. -->
            <td class="cvm-avoid">{{ row.avoid }}</td>
            <!-- aura-allow-text: text the user typed, shown as text only. -->
            <td v-if="row.replacement !== null">{{ row.replacement }}</td>
            <td v-else class="cvm-none">{{ t('convention.manage.replacement_none') }}</td>
          </tr>
        </tbody>
      </table>
    </section>
  </div>
</template>

<style scoped>
.cvm-scrim {
  position: fixed;
  inset: 0;
  /* aura-allow-z-index: xếp lớp CƠ HỌC — dockview dựng ngữ cảnh xếp lớp riêng cho mỗi nhóm panel, nên thứ tự tài liệu một mình không đủ để lớp phủ nằm TRÊN lưới. */
  z-index: 10;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

:root[data-platform='mac'] .cvm-scrim {
  padding-top: var(--space-titlebar-height);
}

.cvm-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-panel-block);
  width: 100%;
  max-width: 1099px;
  max-height: 100%;
  overflow: auto;
  padding: 13px; /* aura-allow-spacing: 1px border sits inside the 14px inset */
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
  color: var(--color-on-surface);
}

.cvm-panel:focus-visible {
  outline: none;
  border-color: var(--color-primary);
}

.cvm-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
}

.cvm-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
}

.cvm-tier {
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-md);
}

.cvm-add {
  display: flex;
  align-items: flex-end;
  flex-wrap: wrap;
  gap: var(--space-panel-inline);
}

.cvm-field {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  flex: 1;
  min-width: 133px;
}

.cvm-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.cvm-note {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.cvm-error {
  color: var(--color-error);
}

.cvm-table {
  width: 100%;
  border-collapse: collapse;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

.cvm-table th,
.cvm-table td {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border-bottom: 1px solid var(--color-outline);
  text-align: left;
  overflow-wrap: anywhere;
}

.cvm-table th {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface-variant);
}

.cvm-none {
  font-style: var(--style-ui-sm-italic);
  color: var(--color-on-surface-variant);
}
</style>
