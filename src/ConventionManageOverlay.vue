<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import SelectMenu from './SelectMenu.vue'
import ConventionIgnoreCard from './ConventionIgnoreCard.vue'
import type { ConventionRuleWire, ConventionTierFilter } from './config/convention'
import {
  CONVENTION_TIER_ORDER,
  conventionAddTier,
  conventionAvoidDraft,
  conventionEditAvoid,
  conventionEditReplacement,
  conventionManageBusy,
  conventionManageCounts,
  conventionManageCursor,
  conventionManageDeletePending,
  conventionManageEditing,
  conventionManageError,
  conventionManageExchangeBusy,
  conventionManageExchangeTier,
  conventionManageExportedPath,
  conventionManageHasLoaded,
  conventionManageIgnoreRows,
  conventionManageIsOpen,
  conventionManageRows,
  conventionManageSummary,
  conventionManageSearch,
  conventionManageTab,
  conventionManageTierFilter,
  conventionManageWorkTierAvailable,
  conventionReplacementDraft,
  conventionRuleCanDisable,
  conventionRuleCanEnable,
  conventionRuleKey,
  setConventionManageCursor,
  setConventionManageSearch,
  setConventionManageTier,
} from './conventionManageState'

const panel = useTemplateRef<HTMLElement>('panel')
const avoidInput = useTemplateRef<HTMLInputElement>('avoidInput')
const list = useTemplateRef<HTMLElement>('list')

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

watch(conventionManageEditing, (editing) => {
  void nextTick(() => {
    if (editing) {
      panel.value?.querySelector<HTMLElement>('[data-convention-edit-avoid]')?.focus()
      return
    }
    if (conventionManageIsOpen.value && !panel.value?.contains(document.activeElement)) list.value?.focus()
  })
})

const addTierOptions = computed(() => [
  { value: 'global', label: t('convention.manage.cell_tier_global') },
  { value: 'work', label: t('convention.manage.cell_tier_work'), disabled: !conventionManageWorkTierAvailable.value },
])

watch(
  () => conventionManageRows.value.length,
  () => {
    void nextTick(() => {
      if (conventionManageIsOpen.value && !panel.value?.contains(document.activeElement)) panel.value?.focus()
    })
  },
)

const TIER_LABEL_KEY: Record<ConventionTierFilter, string> = {
  global: 'convention.manage.tier_global',
  work: 'convention.manage.tier_work',
  both: 'convention.manage.tier_both',
}



const errorText = computed<string | null>(() =>
  conventionManageError.value === null ? null : tError(conventionManageError.value),
)

const currentRule = computed(() => conventionManageRows.value[conventionManageCursor.value] ?? null)

function stateKey(rule: ConventionRuleWire): string {
  if (rule.status === 'shadowed') return 'convention.manage.shadowed'
  if (rule.status === 'disabled_in_work') return 'convention.manage.state_off_work'
  return 'convention.manage.state_on'
}

function barClass(rule: ConventionRuleWire): string {
  return rule.tier === 'work' ? 'cvm-row-bar-work' : ''
}

function deleteHint(rule: ConventionRuleWire): string {
  return t(rule.tier === 'work' ? 'convention.manage.delete_hint_work' : 'convention.manage.delete_hint_global')
}

// WebKit leaves focus on `body` after a mouse press inside the grid, so the list loses the
// keydown handler that Esc and the cursor keys need. Chromium and Vitest do not show this.
function aimRow(index: number, event: MouseEvent): void {
  setConventionManageCursor(index)
  const target = event.target
  if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) return
  event.preventDefault()
  list.value?.focus()
}

function onAddTierChange(): void {
  void nextTick(() => {
    panel.value?.querySelector<HTMLElement>('[data-convention-add-tier] summary')?.focus()
  })
}

function onTierChange(value: ConventionTierFilter): void {
  setConventionManageTier(value)
}

function onSearchInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) setConventionManageSearch(target.value)
}

function onEscape(): void {
  if (conventionManageDeletePending.value || conventionManageEditing.value) {
    dispatch('convention.manage.cancel')
    return
  }
  dispatch('convention.manage.close')
}

function onKeydown(event: KeyboardEvent): void {
  if (conventionManageTab.value !== 'rules') return
  if (event.ctrlKey || event.metaKey || event.altKey) return
  const target = event.target
  if (
    target instanceof HTMLInputElement ||
    target instanceof HTMLSelectElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLButtonElement ||
    (target instanceof HTMLElement && target.tagName === 'SUMMARY')
  ) {
    return
  }
  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      dispatch('convention.manage.next')
      return
    case 'ArrowUp':
      event.preventDefault()
      dispatch('convention.manage.prev')
      return
    case 'Enter':
      event.preventDefault()
      dispatch('convention.manage.edit')
      return
    case ' ':
      event.preventDefault()
      if (conventionRuleCanEnable(currentRule.value)) dispatch('convention.manage.enable')
      else dispatch('convention.manage.disable')
      return
    case 'Backspace':
    case 'Delete':
      event.preventDefault()
      if (!event.repeat) dispatch('convention.manage.delete')
      return
    default:
  }
}

function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'button:not([disabled]):not([tabindex="-1"]), input:not([disabled]):not([data-select-option]), summary, [role="grid"]',
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
    v-if="conventionManageIsOpen"
    class="cvm-scrim"
    @keydown.esc="onEscape()"
    @keydown.tab="trapTab($event)"
    @keydown="onKeydown($event)"
  >
    <section ref="panel" class="cvm-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="cvm-title">
      <header class="cvm-head">
        <h2 id="cvm-title" class="cvm-title">{{ t('convention.manage.title') }}</h2>
        <div class="cvm-head-actions">
          <button
            type="button"
            class="ui-btn"
            data-convention-export
            :disabled="conventionManageBusy || conventionManageEditing || conventionManageExchangeBusy"
            @click="dispatch('convention.manage.export')"
          >
            {{ t('convention.manage.export') }}
          </button>
          <button
            type="button"
            class="ui-btn"
            data-convention-import-open
            :disabled="conventionManageBusy || conventionManageEditing || conventionManageExchangeBusy"
            @click="dispatch('convention.manage.import')"
          >
            {{ t('convention.manage.import') }}
          </button>
          <button type="button" class="cvm-close ui-btn ui-btn--quiet" data-convention-manage-close @click="dispatch('convention.manage.close')">
            {{ t('convention.manage.close') }}
          </button>
        </div>
      </header>

      <div class="cvm-tabs" role="tablist" :aria-label="t('convention.manage.tabs_label')">
        <button
          type="button"
          role="tab"
          class="ui-tab"
          data-convention-tab="rules"
          :aria-selected="conventionManageTab === 'rules'"
          :disabled="conventionManageEditing"
          @click="dispatch('convention.manage.tab_rules')"
        >
          {{ t('convention.manage.tab_rules') }}
        </button>
        <button
          type="button"
          role="tab"
          class="ui-tab"
          data-convention-tab="ignored"
          :aria-selected="conventionManageTab === 'ignored'"
          :disabled="conventionManageEditing"
          @click="dispatch('convention.manage.tab_ignored')"
        >
          {{ t('convention.manage.tab_ignored', { ignored_count: String(conventionManageIgnoreRows.length) }) }}
        </button>
      </div>

      <ConventionIgnoreCard v-if="conventionManageTab === 'ignored'" />
      <template v-else>
      <p v-if="conventionManageHasLoaded && !conventionManageWorkTierAvailable" class="cvm-note">
        {{ t('convention.manage.work_locked') }}
      </p>

      <div class="cvm-toolbar">
        <fieldset class="cvm-seg ui-seg" role="radiogroup" :aria-label="t('convention.manage.tier_label')">
          <label
            v-for="opt in CONVENTION_TIER_ORDER"
            :key="opt"
            class="cvm-seg-opt ui-seg__item"
            :class="{ 'cvm-seg-on': conventionManageTierFilter === opt }"
          >
            <input
              type="radio"
              name="cvm-tier"
              class="cvm-sr-only"
              :data-convention-tier="opt"
              :checked="conventionManageTierFilter === opt"
              :disabled="conventionManageEditing || (opt === 'work' && !conventionManageWorkTierAvailable)"
              @change="onTierChange(opt)"
            />
            <span class="ui-row">
              <!-- aura-allow-text: result of t() with a count from Rust. -->
              {{ t(TIER_LABEL_KEY[opt], { rule_count: String(conventionManageCounts[opt]) }) }}
            </span>
          </label>
        </fieldset>
        <label class="cvm-field cvm-field-search">
          <span class="cvm-sr-only">{{ t('convention.manage.search_placeholder') }}</span>
          <input
            type="text"
            class="cvm-input ui-field"
            autocomplete="off"
            data-convention-search
            :placeholder="t('convention.manage.search_placeholder')"
            :value="conventionManageSearch"
            :disabled="conventionManageEditing"
            @input="onSearchInput"
          />
        </label>
      </div>

      <p v-if="conventionManageHasLoaded" class="cvm-note">
        <!-- aura-allow-text: result of t() with counts from Rust. -->
        {{ t('convention.manage.counts', conventionManageSummary) }}
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
        <label class="cvm-field cvm-field-tier">
          <span class="cvm-label">{{ t('convention.manage.add_tier') }}</span>
          <div data-convention-add-tier>
            <SelectMenu
              v-model="conventionAddTier"
              variant="field"
              name="cvm-add-tier"
              option-attr="data-convention-add-tier-option"
              :options="addTierOptions"
              :label="t('convention.manage.add_tier')"
              @change="onAddTierChange"
            />
          </div>
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
      <p v-else-if="conventionManageHasLoaded && conventionManageRows.length === 0" class="cvm-note" role="status">
        <!-- aura-allow-text: result of t() with the user's query. -->
        {{
          conventionManageSearch.trim() === ''
            ? t('convention.manage.empty_rules')
            : t('convention.manage.no_match', { query: conventionManageSearch.trim() })
        }}
      </p>
      <template v-else-if="conventionManageRows.length > 0">
        <table
          ref="list"
          class="cvm-table"
          role="grid"
          tabindex="0"
          :aria-label="t('convention.manage.list_label')"
          :aria-activedescendant="currentRule === null ? undefined : `cvm-row-${conventionRuleKey(currentRule)}`"
        >
          <thead>
            <tr>
              <th scope="col">{{ t('convention.manage.col_avoid') }}</th>
              <th scope="col">{{ t('convention.manage.col_tier') }}</th>
              <th scope="col">{{ t('convention.manage.col_state') }}</th>
              <th scope="col">{{ t('convention.manage.col_actions') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(rule, i) in conventionManageRows"
              :id="`cvm-row-${conventionRuleKey(rule)}`"
              :key="conventionRuleKey(rule)"
              data-convention-rule-row
              :data-convention-rule-tier="rule.tier"
              :data-convention-rule-status="rule.status"
              class="cvm-row"
              tabindex="-1"
              :aria-selected="i === conventionManageCursor"
              :class="[
                barClass(rule),
                {
                  'cvm-row-current': i === conventionManageCursor,
                  'cvm-row-shadowed': rule.status === 'shadowed',
                  'cvm-row-delete-pending': i === conventionManageCursor && conventionManageDeletePending,
                },
              ]"
              @mousedown="aimRow(i, $event)"
              @dblclick="dispatch('convention.manage.edit')"
            >
              <td class="cvm-avoid">
                <template v-if="i === conventionManageCursor && conventionManageEditing">
                  <input
                    v-model="conventionEditAvoid"
                    type="text"
                    class="cvm-input ui-field"
                    autocomplete="off"
                    data-convention-edit-avoid
                    :aria-label="t('convention.manage.edit_avoid_label')"
                    @keydown.enter.prevent="dispatch('convention.manage.save')"
                  />
                  <input
                    v-model="conventionEditReplacement"
                    type="text"
                    class="cvm-input ui-field"
                    autocomplete="off"
                    data-convention-edit-replacement
                    :aria-label="t('convention.manage.edit_replacement_label')"
                    @keydown.enter.prevent="dispatch('convention.manage.save')"
                  />
                </template>
                <template v-else>
                  <!-- aura-allow-text: text the user typed, shown as text only. -->
                  <span>{{ rule.avoid_phrase }}</span>
                  <span class="cvm-arrow" aria-hidden="true">→</span>
                  <!-- aura-allow-text: text the user typed, shown as text only. -->
                  <span v-if="rule.replacement_phrase !== null">{{ rule.replacement_phrase }}</span>
                  <span v-else class="cvm-none">{{ t('convention.manage.replacement_none') }}</span>
                </template>
              </td>
              <td class="cvm-cell-meta">
                {{ t(rule.tier === 'work' ? 'convention.manage.cell_tier_work' : 'convention.manage.cell_tier_global') }}
              </td>
              <td class="cvm-cell-meta">
                <span class="cvm-badge" :class="{ 'cvm-badge-shadowed': rule.status === 'shadowed' }">{{ t(stateKey(rule)) }}</span>
                <span v-if="i === conventionManageCursor && conventionManageDeletePending" class="cvm-note cvm-error">
                  <!-- aura-allow-text: result of t(). -->
                  {{ deleteHint(rule) }}
                </span>
              </td>
              <td class="cvm-cell-actions">
                <template v-if="i === conventionManageCursor && conventionManageEditing">
                  <button type="button" class="cvm-act ui-btn" data-convention-save :disabled="conventionManageBusy" @click="dispatch('convention.manage.save')">
                    {{ t('convention.manage.save') }}
                  </button>
                  <button type="button" class="cvm-act ui-btn ui-btn--quiet" data-convention-cancel @click="dispatch('convention.manage.cancel')">
                    {{ t('convention.manage.cancel') }}
                  </button>
                </template>
                <template v-else>
                  <button
                    type="button"
                    class="cvm-act ui-btn ui-btn--quiet"
                    data-convention-edit
                    :tabindex="i === conventionManageCursor ? 0 : -1"
                    @click="dispatch('convention.manage.edit')"
                  >
                    {{ t('convention.manage.edit') }}
                  </button>
                  <button
                    v-if="conventionRuleCanDisable(rule)"
                    type="button"
                    class="cvm-act ui-btn ui-btn--quiet"
                    data-convention-disable
                    :tabindex="i === conventionManageCursor ? 0 : -1"
                    @click="dispatch('convention.manage.disable')"
                  >
                    {{ t('convention.manage.disable_in_work') }}
                  </button>
                  <button
                    v-if="conventionRuleCanEnable(rule)"
                    type="button"
                    class="cvm-act ui-btn ui-btn--quiet"
                    data-convention-enable
                    :tabindex="i === conventionManageCursor ? 0 : -1"
                    @click="dispatch('convention.manage.enable')"
                  >
                    {{ t('convention.manage.enable_in_work') }}
                  </button>
                  <button
                    type="button"
                    class="cvm-act ui-btn"
                    :class="{ 'ui-btn--danger': i === conventionManageCursor && conventionManageDeletePending }"
                    data-convention-delete
                    :tabindex="i === conventionManageCursor ? 0 : -1"
                    @click="dispatch('convention.manage.delete')"
                  >
                    {{ t(i === conventionManageCursor && conventionManageDeletePending ? 'convention.manage.delete_confirm' : 'convention.manage.delete') }}
                  </button>
                </template>
              </td>
            </tr>
          </tbody>
        </table>
        <p class="cvm-note">{{ t('convention.manage.shadowed_hint') }}</p>
        <p class="cvm-note">{{ t('convention.manage.hint') }}</p>
      </template>
      </template>

      <footer class="cvm-foot">
        <p class="cvm-note" data-convention-exchange-tier>
          {{ t('convention.manage.exchange_tier_label') }}:
          {{ t(conventionManageExchangeTier === 'work' ? 'convention.manage.cell_tier_work' : 'convention.manage.cell_tier_global') }}
        </p>
        <p v-if="conventionManageExportedPath !== null" class="cvm-note" role="status" data-convention-export-done>
          <!-- aura-allow-text: result of t() with the path Rust wrote. -->
          {{ t('convention.manage.export_done', { path: conventionManageExportedPath }) }}
        </p>
      </footer>
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

.cvm-head-actions {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 1);
}

.cvm-foot {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-panel-inline);
}

.cvm-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
}


.cvm-tabs {
  display: flex;
  gap: calc(var(--space-unit) * 1);
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

.cvm-toolbar {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--space-panel-inline);
}

.cvm-seg {
  margin: 0;
  padding: 0;
}

.cvm-seg-opt {
  position: relative;
}

.cvm-field-search {
  flex: 1;
  max-width: 294px;
}

.cvm-field-tier {
  flex: 0 0 auto;
  min-width: 0;
}

.cvm-sr-only {
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

.cvm-row > td:first-child {
  border-left: 2px solid transparent;
}

.cvm-row-bar-work > td:first-child {
  border-left: 2px solid var(--color-confirmed);
}

.cvm-row-current {
  background: var(--color-surface-accent);
}

.cvm-row-shadowed > td:first-child {
  padding-left: calc(var(--space-unit) * 5);
}

.cvm-row-delete-pending > td {
  border-bottom-color: var(--color-error);
}

.cvm-arrow {
  margin: 0 calc(var(--space-unit) * 1);
  color: var(--color-on-surface-variant);
}

.cvm-cell-meta {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.cvm-cell-actions {
  white-space: nowrap;
}

.cvm-badge {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  border: 1px solid var(--color-outline);
  padding: 0 6px; /* aura-allow-spacing: right/left border sits inside the 7px inset */
}

.cvm-badge-shadowed {
  border-style: dashed;
}
</style>
