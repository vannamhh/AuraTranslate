<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import {
  conventionReplaceAlert,
  conventionReplaceArmed,
  conventionReplaceCursor,
  conventionReplaceDraftList,
  conventionReplaceDraftSentences,
  conventionReplaceFailure,
  conventionReplaceFilter,
  conventionReplaceHasScanned,
  conventionReplaceHidden,
  conventionReplaceIsOpen,
  conventionReplaceKeptNote,
  conventionReplaceOptions,
  conventionReplaceProgress,
  conventionReplaceScanSlow,
  conventionReplaceScanning,
  conventionReplaceTotals,
  conventionReplaceVisible,
  conventionReplaceWriting,
  disarmConventionReplace,
  isConventionReplaceSelected,
  moveConventionReplaceCursor,
  setConventionReplaceFilter,
  toggleConventionReplaceAtCursor,
  toggleConventionReplaceSpot,
} from './conventionReplaceState'
import type { ReplaceSpot } from './conventionReplaceState'

const panel = useTemplateRef<HTMLElement>('panel')
const list = useTemplateRef<HTMLElement>('list')

let returnFocusTo: HTMLElement | null = null

function restoreFocus(): void {
  const strip = document.querySelector<HTMLElement>('[data-convention-replace-chapter]')
  if (strip !== null) {
    strip.focus()
    return
  }
  if (returnFocusTo !== null && returnFocusTo.isConnected) {
    returnFocusTo.focus()
    return
  }
  const caret = document.querySelector<HTMLElement>('[data-col="tgt"]')
  caret?.focus()
}

watch(conventionReplaceIsOpen, (isOpen) => {
  if (isOpen) {
    returnFocusTo = focusReturnTargetOnOpen('[data-convention-replace-open]')
    void nextTick(() => panel.value?.focus())
    return
  }
  void nextTick(restoreFocus)
  returnFocusTo = null
})

watch(conventionReplaceScanning, (busy) => {
  if (busy || !conventionReplaceIsOpen.value) return
  void nextTick(() => (list.value ?? panel.value)?.focus())
})

const totals = computed(() => conventionReplaceTotals.value)
const visible = computed(() => conventionReplaceVisible.value)
const options = computed(() => conventionReplaceOptions.value)
const hasRows = computed(() => totals.value.matches > 0)
const stuck = computed(
  () => conventionReplaceFailure.value !== null || conventionReplaceAlert.value !== null,
)
const busy = computed(() => conventionReplaceScanning.value || conventionReplaceWriting.value)
const draftCount = computed(() => conventionReplaceDraftSentences.value.length)
const failureText = computed<string | null>(() =>
  conventionReplaceFailure.value === null ? null : tError(conventionReplaceFailure.value),
)
const alertText = computed<string | null>(() =>
  conventionReplaceAlert.value === null ? null : tError(conventionReplaceAlert.value),
)

const filterValue = computed(() => conventionReplaceFilter.value ?? '')

const scanPercent = computed(() => {
  const p = conventionReplaceProgress.value
  return p === null || p.total === 0 ? 0 : Math.min(100, Math.round((p.done / p.total) * 100))
})

const applyLabel = computed(() => {
  const count = String(totals.value.selected)
  return conventionReplaceArmed.value
    ? t('convention.replace.apply_again', { selected_count: count })
    : t('convention.replace.apply', { selected_count: count })
})

const armedHint = computed(() =>
  t('convention.replace.apply_again_hint', {
    draft_count: String(draftCount.value),
    segment_list: conventionReplaceDraftList(),
  }),
)

const hiddenParts = computed(() => {
  const h = conventionReplaceHidden.value
  const parts: string[] = []
  if (h.ignored > 0) parts.push(t('convention.replace.hidden_ignored', { ignored_count: String(h.ignored) }))
  if (h.glossarySuppressed > 0) {
    parts.push(t('convention.replace.hidden_glossary', { glossary_count: String(h.glossarySuppressed) }))
  }
  if (h.overlapped > 0) parts.push(t('convention.replace.hidden_overlapped', { overlapped_count: String(h.overlapped) }))
  if (h.notScanned > 0) {
    parts.push(t('convention.replace.hidden_not_scanned', { not_scanned_count: String(h.notScanned) }))
  }
  return parts
})

const emptyText = computed(() => {
  const only = conventionReplaceFilter.value
  const avoid = only === null ? undefined : options.value.find((o) => o.ruleKey === only)?.avoid
  return avoid === undefined
    ? t('convention.replace.empty')
    : t('convention.replace.empty_rule', { avoid_phrase: avoid })
})

function onEscape(): void {
  if (disarmConventionReplace()) return
  dispatch('convention.replace_chapter_close')
}

function optionId(spot: ReplaceSpot): string {
  return `crv-opt-${spot.key}`
}

const activeId = computed(() => {
  const spots = visible.value.flatMap((g) => g.spots)
  if (spots.length === 0) return undefined
  return optionId(spots[Math.min(conventionReplaceCursor.value, spots.length - 1)])
})

type Part = { kind: 'plain' | 'del' | 'ins' | 'keep'; text: string }

function partsOf(spot: ReplaceSpot): Part[] {
  const parts: Part[] = [{ kind: 'plain', text: spot.before }]
  if (isConventionReplaceSelected(spot.key)) {
    parts.push({ kind: 'del', text: spot.phrase }, { kind: 'ins', text: spot.replacement })
  } else {
    parts.push({ kind: 'keep', text: spot.phrase })
  }
  parts.push({ kind: 'plain', text: spot.after })
  return parts
}

function cursorKeyOf(spot: ReplaceSpot): boolean {
  return activeId.value === optionId(spot)
}

function ariaLabelOf(spot: ReplaceSpot, ord: number): string {
  return t('convention.replace.match_aria_label', {
    avoid_phrase: spot.phrase,
    replacement_phrase: spot.replacement,
    segment_number: String(ord),
  })
}

function filterLabel(ruleKey: string | null): string {
  if (ruleKey === null) return t('convention.replace.filter_all', { match_count: String(totals.value.matches) })
  const option = options.value.find((o) => o.ruleKey === ruleKey)
  return t('convention.replace.filter_rule', { avoid_phrase: option?.avoid ?? ruleKey, match_count: String(option?.count ?? 0) })
}

function onFilterChange(ruleKey: string | null): void {
  setConventionReplaceFilter(ruleKey)
}

function scrollActiveIntoView(): void {
  void nextTick(() => {
    const id = activeId.value
    if (id === undefined) return
    document.getElementById(id)?.scrollIntoView({ block: 'nearest' })
  })
}

function onListKeydown(event: KeyboardEvent): void {
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    moveConventionReplaceCursor(event.key === 'ArrowDown' ? 1 : -1)
    scrollActiveIntoView()
    return
  }
  if (event.key === ' ') {
    event.preventDefault()
    toggleConventionReplaceAtCursor()
  }
}

function onRowMouseDown(spot: ReplaceSpot): void {
  if (conventionReplaceWriting.value) return
  toggleConventionReplaceSpot(spot.key)
  list.value?.focus()
}

function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled]), [tabindex="0"]'),
  ).filter((el) => !(el instanceof HTMLInputElement && el.type === 'radio' && !el.checked))
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
    v-if="conventionReplaceIsOpen"
    class="crv-scrim"
    @keydown.esc.stop.prevent="onEscape()"
    @keydown.tab="trapTab($event)"
  >
    <section
      ref="panel"
      class="crv-panel"
      tabindex="-1"
      role="dialog"
      aria-modal="true"
      aria-labelledby="crv-title"
      data-convention-replace
    >
      <header class="crv-head">
        <div class="crv-heading">
          <h2 id="crv-title" class="crv-title">
            {{ t('convention.replace.title', { chapter_title: t('convention.replace.this_chapter') }) }}
          </h2>
          <p v-if="conventionReplaceHasScanned" class="crv-sub" role="status" data-convention-replace-summary>
            <!-- aura-allow-text: result of t() with counts from the scan. -->
            {{
              t('convention.replace.summary', {
                match_count: String(totals.matches),
                sentence_count: String(totals.sentences),
                selected_count: String(totals.selected),
              })
            }}
          </p>
        </div>
        <div class="crv-bulk">
          <button
            type="button"
            class="ui-btn"
            data-convention-replace-select-all
            :disabled="busy || stuck || !hasRows"
            @click="dispatch('convention.replace_chapter_select_all')"
          >
            {{ t('convention.replace.select_all') }}
          </button>
          <button
            type="button"
            class="ui-btn"
            data-convention-replace-select-none
            :disabled="busy || stuck || !hasRows"
            @click="dispatch('convention.replace_chapter_select_none')"
          >
            {{ t('convention.replace.select_none') }}
          </button>
        </div>
      </header>

      <div v-if="options.length > 0" class="crv-bar">
        <span class="crv-bar-label">{{ t('convention.replace.filter_label') }}</span>
        <fieldset class="crv-seg ui-seg" role="radiogroup" :aria-label="t('convention.replace.filter_label')" data-convention-replace-filter>
          <label class="ui-seg__item" :class="{ 'crv-seg-on': filterValue === '' }">
            <input
              type="radio"
              name="crv-filter"
              class="crv-sr-only"
              :checked="filterValue === ''"
              :disabled="busy || stuck"
              @change="onFilterChange(null)"
            />
            <!-- aura-allow-text: result of t() with a count from the scan. -->
            <span class="ui-row">{{ filterLabel(null) }}</span>
          </label>
          <label
            v-for="option in options"
            :key="option.ruleKey"
            class="ui-seg__item"
            :class="{ 'crv-seg-on': filterValue === option.ruleKey }"
          >
            <input
              type="radio"
              name="crv-filter"
              class="crv-sr-only"
              :checked="filterValue === option.ruleKey"
              :disabled="busy || stuck"
              @change="onFilterChange(option.ruleKey)"
            />
            <!-- aura-allow-text: result of t() with a user rule and a count. -->
            <span class="ui-row">{{ filterLabel(option.ruleKey) }}</span>
          </label>
        </fieldset>
      </div>

      <div v-if="conventionReplaceScanning && conventionReplaceScanSlow" class="crv-scan" role="status" data-convention-replace-scanning>
        <div class="crv-scan-bar" aria-hidden="true"><i :style="{ width: scanPercent + '%' }"></i></div>
        <!-- aura-allow-text: result of t() with counts streamed from Rust. -->
        <p class="crv-note">
          {{
            t('convention.replace.scanning', {
              done_count: String(conventionReplaceProgress?.done ?? 0),
              total_count: String(conventionReplaceProgress?.total ?? 0),
            })
          }}
        </p>
      </div>

      <p v-if="failureText !== null" class="crv-frame" role="alert" data-convention-replace-error>
        <!-- aura-allow-text: result of tError(). -->
        {{ failureText }}
      </p>
      <p v-if="alertText !== null" class="crv-frame" role="alert" data-convention-replace-refused>
        <!-- aura-allow-text: result of tError(). -->
        {{ alertText }}
      </p>
      <p
        v-if="conventionReplaceKeptNote !== null && !stuck && !conventionReplaceScanning"
        class="crv-note"
        role="status"
        data-convention-replace-kept
      >
        {{ t('convention.replace.rescan_kept', { kept_count: String(conventionReplaceKeptNote) }) }}
      </p>

      <p
        v-if="conventionReplaceHasScanned && !conventionReplaceScanning && !stuck && visible.length === 0"
        class="crv-note"
        role="status"
        data-convention-replace-empty
      >
        <!-- aura-allow-text: result of t(). -->
        {{ emptyText }}
      </p>

      <ul
        v-if="visible.length > 0 && !conventionReplaceScanning"
        ref="list"
        class="crv-list"
        tabindex="0"
        role="listbox"
        aria-multiselectable="true"
        :aria-label="t('convention.replace.list_aria_label')"
        :aria-activedescendant="activeId"
        data-convention-replace-list
        @keydown="onListKeydown($event)"
      >
        <li v-for="group in visible" :key="group.segmentId" class="crv-group" role="presentation" data-convention-replace-group>
          <div class="crv-group-head" role="presentation">
            <b>{{ t('convention.replace.sentence', { segment_number: String(group.ord) }) }}</b>
            <span class="crv-label" :class="group.confirmed ? 'crv-label--confirmed' : 'crv-label--draft'">
              {{ t(group.confirmed ? 'convention.replace.confirmed_label' : 'convention.replace.draft_label') }}
            </span>
          </div>
          <ul class="crv-spots" role="group">
            <li
              v-for="spot in group.spots"
              :id="optionId(spot)"
              :key="spot.key"
              class="crv-spot"
              :class="{ 'crv-spot--off': !isConventionReplaceSelected(spot.key), 'crv-spot--cursor': cursorKeyOf(spot) }"
              role="option"
              :aria-selected="isConventionReplaceSelected(spot.key)"
              :aria-label="ariaLabelOf(spot, group.ord)"
              data-convention-replace-spot
              :data-segment-id="group.segmentId"
              :data-spot-start="spot.start"
              :data-selected="isConventionReplaceSelected(spot.key)"
              @mousedown.prevent="onRowMouseDown(spot)"
            >
              <span class="crv-ck" aria-hidden="true"><i :class="{ 'crv-ck--on': isConventionReplaceSelected(spot.key) }"></i></span>
              <span class="crv-text"
                ><template v-for="(part, i) in partsOf(spot)" :key="i"
                  ><del v-if="part.kind === 'del'" class="crv-del"><!-- aura-allow-text: the user's phrase, shown as text only. -->{{ part.text }}</del
                  ><ins v-else-if="part.kind === 'ins'" class="crv-ins"><!-- aura-allow-text: the rule's replacement, shown as text only. -->{{ part.text }}</ins
                  ><span v-else-if="part.kind === 'keep'" class="crv-keep"><!-- aura-allow-text: the user's phrase, shown as text only. -->{{ part.text }}</span
                  ><template v-else><!-- aura-allow-text: the user's sentence, shown as text only. -->{{ part.text }}</template></template
              ></span>
            </li>
          </ul>
        </li>
      </ul>

      <p v-if="conventionReplaceHasScanned && !conventionReplaceScanning" class="crv-foot" data-convention-replace-hidden>
        <template v-if="hiddenParts.length === 0">{{ t('convention.replace.hidden_none') }}</template>
        <template v-else>
          <b>{{ t('convention.replace.hidden_title') }}</b>
          <!-- aura-allow-text: result of t() with counts from Rust. -->
          {{ ' ' + hiddenParts.join(' · ') }}
        </template>
      </p>

      <p v-if="conventionReplaceArmed" class="crv-confirm" role="status" data-convention-replace-armed>
        <!-- aura-allow-text: result of t() with the draft sentence numbers. -->
        {{ armedHint }}
      </p>

      <footer class="crv-actions">
        <template v-if="stuck">
          <button
            type="button"
            class="crv-btn ui-btn ui-btn--primary"
            data-convention-replace-rescan
            :disabled="busy"
            @click="dispatch('convention.replace_chapter_rescan')"
          >
            {{ t('convention.replace.rescan') }}
          </button>
          <button type="button" class="crv-btn ui-btn" data-convention-replace-close @click="dispatch('convention.replace_chapter_close')">
            {{ t('convention.replace.close') }}
          </button>
          <span class="crv-hint">{{ t('convention.replace.hint') }}</span>
        </template>
        <template v-else>
          <button
            type="button"
            class="crv-btn ui-btn ui-btn--primary"
            data-convention-replace-confirm
            :disabled="busy || totals.selected === 0"
            @click="dispatch('convention.replace_chapter_confirm')"
          >
            <!-- aura-allow-text: result of t() with the selected count. -->
            {{ totals.selected === 0 ? t('convention.replace.apply_none') : applyLabel }}
          </button>
          <button type="button" class="crv-btn ui-btn" data-convention-replace-close @click="dispatch('convention.replace_chapter_close')">
            {{ t('convention.replace.cancel') }}
          </button>
          <span class="crv-hint">{{ t(conventionReplaceArmed ? 'convention.replace.hint_armed' : 'convention.replace.hint') }}</span>
        </template>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.crv-scrim {
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

:root[data-platform='mac'] .crv-scrim {
  padding-top: var(--space-titlebar-height);
}

.crv-panel {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 917px;
  max-height: 100%;
  overflow: auto;
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
  color: var(--color-on-surface);
}

.crv-panel:focus-visible {
  outline: none;
  border-color: var(--color-primary);
}

.crv-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  padding: 13px; /* aura-allow-spacing: the panel's 1px border sits inside the 14px inset */
  border-bottom: 1px solid var(--color-outline);
}

.crv-title {
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-md);
}

.crv-sub,
.crv-note,
.crv-hint,
.crv-bar-label {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.crv-bulk {
  display: flex;
  gap: calc(var(--space-unit) * 1);
}

.crv-bar {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 1) 13px; /* aura-allow-spacing: aligns with the head's 14px inset inside the 1px border */
  border-bottom: 1px solid var(--color-outline);
}

.crv-seg {
  margin: 0;
  padding: 0;
  min-width: 0;
}

.crv-sr-only {
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

.crv-scan {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  padding: calc(var(--space-unit) * 2) 13px; /* aura-allow-spacing: aligns with the head's 14px inset inside the 1px border */
}

.crv-scan-bar {
  height: 2px;
  background: var(--color-outline-faint);
}

.crv-scan-bar > i {
  display: block;
  height: 100%;
  background: var(--color-on-surface);
}

.crv-note {
  padding: calc(var(--space-unit) * 2) 13px; /* aura-allow-spacing: aligns with the head's 14px inset inside the 1px border */
}

.crv-frame {
  margin: 0;
  padding: calc(var(--space-unit) * 2) 13px; /* aura-allow-spacing: aligns with the head's 14px inset inside the 1px border */
  border-left: 2px solid var(--color-error);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

.crv-list {
  margin: 0;
  padding: 0;
  list-style: none;
  max-height: 60vh;
  overflow: auto;
}

.crv-group {
  border-bottom: 1px solid var(--color-outline-faint);
}

.crv-group-head {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 1) 13px 0; /* aura-allow-spacing: aligns with the head's 14px inset inside the 1px border */
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

.crv-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
}

.crv-label--draft {
  color: var(--color-error);
  font-weight: var(--weight-ui-md-strong);
}

.crv-label--confirmed {
  color: var(--color-on-surface-variant);
}

.crv-spots {
  margin: 0;
  padding: 0;
  list-style: none;
}

.crv-spot {
  display: flex;
  align-items: flex-start;
  gap: calc(var(--space-unit) * 1);
  padding: 0 13px 0 11px; /* aura-allow-spacing: the 2px cursor rule and the panel's 1px border sit inside the 14px inset */
  border-left: 2px solid transparent;
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

.crv-spot--cursor {
  border-left-color: var(--color-primary);
}

.crv-spot--off .crv-text {
  color: var(--color-on-surface-variant);
}

.crv-ck {
  display: grid;
  place-items: center;
  flex: none;
  width: 28px;
  height: 28px;
}

.crv-ck > i {
  display: grid;
  place-items: center;
  width: 14px;
  height: 14px;
  border: 1px solid var(--color-control-rule);
  border-radius: var(--radius-sm);
  background: var(--color-surface);
}

.crv-ck > i.crv-ck--on {
  background: var(--color-on-surface);
  border-color: var(--color-on-surface);
}

.crv-ck > i.crv-ck--on::after {
  content: '';
  width: 8px; /* aura-allow-spacing: the tick is a drawn 8x4 shape inside the 14px box */
  height: 4px; /* aura-allow-spacing: tick shape, see width */
  margin-top: -2px; /* aura-allow-spacing: optical centring of the tick inside the 14px box */
  border-left: 2px solid var(--color-surface);
  border-bottom: 2px solid var(--color-surface);
  transform: rotate(-45deg);
}

.crv-text {
  padding-block: 3.5px; /* aura-allow-spacing: centres a 21px text line in the 28px row */
  overflow-wrap: anywhere;
}

.crv-del {
  background: var(--color-diff-del-bg);
  color: var(--color-diff-del-ink);
  text-decoration: line-through;
}

.crv-ins {
  background: var(--color-diff-add-bg);
  color: var(--color-diff-add-ink);
  text-decoration: underline;
}

.crv-keep {
  text-decoration: underline dotted 2px;
  text-underline-offset: 4px;
}

.crv-foot {
  margin: 0;
  padding: calc(var(--space-unit) * 2) 13px; /* aura-allow-spacing: aligns with the head's 14px inset inside the 1px border */
  border-top: 1px solid var(--color-outline);
  background: var(--color-background);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.crv-confirm {
  margin: 0;
  padding: calc(var(--space-unit) * 2) 13px; /* aura-allow-spacing: aligns with the head's 14px inset inside the 1px border */
  border-top: 1px solid var(--color-outline);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

.crv-actions {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 2) 13px; /* aura-allow-spacing: aligns with the head's 14px inset inside the 1px border */
  border-top: 1px solid var(--color-outline);
  background: var(--color-background);
}

.crv-btn {
  min-width: 8.5em;
}
</style>
