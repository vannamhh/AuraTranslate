<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import type { IpcError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import { historyTimeLabel } from './panels/segmentHistoryTime'
import type { TmManagePairOriginFilter, TmManageTier, TmManageTierFilter, TmPairOrigin } from './config/tm'
import { glossaryExchangeBusy } from './glossaryExchangeGate'
import { tmExchangeErrorText } from './tmExchangeError'
import {
  cancelTmManageDeleteConfirm,
  setTmManagePairOriginFilter,
  setTmManageExchangeTier,
  setTmManageSearch,
  setTmManageTierFilter,
  tmManageActionError,
  tmManageActionNotice,
  tmManageBulkDeleted,
  tmManageBulkPending,
  tmManageCurrentCopyCount,
  tmManageCurrentRow,
  tmManageCursor,
  tmManageDeletePending,
  tmManageEditTarget,
  tmManageEditing,
  tmManageEmptyReasonFor,
  tmManageExchangeTier,
  tmManageExportBusy,
  tmManageExportError,
  tmManageExportIpcUnavailable,
  tmManageExportLeftOutCount,
  tmManageExportedPath,
  tmManageFlatRows,
  tmManageHealth,
  tmManageHealthTotal,
  tmManageImportDone,
  tmManageLoadError,
  tmManagePairOriginFilter,
  tmManageOthersCount,
  tmManageOverlayIsOpen,
  tmManageRowKey,
  tmManageSaving,
  tmManageSavingAction,
  tmManageSearchQuery,
  tmManageShownGroups,
  tmManageStatus,
  tmManageTierFilter,
  tmManageTmEmpty,
  tmManageTotalGroups,
  tmManageWorkOpen,
} from './tmManageState'

let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

const list = useTemplateRef<HTMLElement>('list')
const editInput = useTemplateRef<HTMLInputElement>('editInput')

function focusInitialTarget(): void {
  if (tmManageStatus.value === 'loaded' && tmManageFlatRows.value.length > 0 && list.value !== null) {
    list.value.focus()
    return
  }
  panel.value?.focus()
}

watch(tmManageOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-tm-manage-open]')
    void nextTick(focusInitialTarget)
    return
  }
  const back = returnFocusTo
  returnFocusTo = null
  if (back !== null && back.isConnected) back.focus()
})

function focusIsStray(): boolean {
  const active = document.activeElement
  return (
    active === null ||
    active === document.body ||
    active === list.value ||
    (active instanceof HTMLButtonElement && active.disabled)
  )
}

watch(tmManageFlatRows, () => {
  if (!tmManageOverlayIsOpen.value) return
  const rows = tmManageFlatRows.value
  if (rows.length > 0 && document.activeElement === panel.value && tmManageStatus.value === 'loaded') {
    void nextTick(() => list.value?.focus())
    return
  }
  if (rows.length === 0) {
    void nextTick(() => {
      if (tmManageOverlayIsOpen.value && tmManageFlatRows.value.length === 0 && focusIsStray()) panel.value?.focus()
    })
  }
})

watch([tmManageCursor, tmManageFlatRows], () => {
  if (!tmManageOverlayIsOpen.value) return
  const current = tmManageCurrentRow.value
  if (current === null) return
  void nextTick(() => {
    document.getElementById(optionId(current.tier, current.unit_id))?.scrollIntoView({ block: 'nearest' })
  })
})

watch(tmManageEditing, (editing) => {
  if (!tmManageOverlayIsOpen.value) return
  void nextTick(() => {
    if (editing) editInput.value?.focus()
    else focusInitialTarget()
  })
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

function optionId(tier: string, unitId: number): string {
  return `tm-option-${tier}-${unitId}`
}

const TIER_OPTIONS: ReadonlyArray<{
  value: TmManageTierFilter
  labelKey: string
}> = [
  { value: 'both', labelKey: 'tm.manage.tier_both' },
  { value: 'work', labelKey: 'tm.fuzzy.tier_work' },
  { value: 'global', labelKey: 'tm.fuzzy.tier_global' },
]

const PAIR_ORIGIN_OPTIONS: ReadonlyArray<{
  value: TmManagePairOriginFilter
  labelKey: string
}> = [
  { value: 'all', labelKey: 'tm.manage.origin_filter_all' },
  { value: 'others', labelKey: 'tm.manage.origin_filter_others' },
  { value: 'self', labelKey: 'tm.manage.origin_self' },
  { value: 'other', labelKey: 'tm.manage.origin_other' },
  { value: 'bilingual_import', labelKey: 'tm.manage.origin_bilingual_import' },
]

function pairOriginLabel(pairOrigin: TmPairOrigin): string {
  return t(`tm.manage.origin_${pairOrigin}`)
}

function tierLabel(tier: string): string {
  return t(tier === 'global' ? 'tm.fuzzy.tier_global' : 'tm.fuzzy.tier_work')
}

function healthPercent(count: number): number {
  return tmManageHealthTotal.value === 0 ? 0 : Math.round((count * 100) / tmManageHealthTotal.value)
}

const nowMs = computed(() => {
  void tmManageFlatRows.value
  return Date.now()
})

function dateLabel(createdAt: string): string {
  const { key, params } = historyTimeLabel(createdAt, nowMs.value)
  return t(key, params)
}

const emptyReason = computed(() =>
  tmManageEmptyReasonFor(tmManageStatus.value, tmManageTmEmpty.value, tmManageFlatRows.value.length),
)

const bulkConfirmCount = computed(() => ({
  count: String(tmManageOthersCount.value),
}))

const BULK_BUTTON_KEYS = {
  both: 'tm.manage.delete_others_both',
  work: 'tm.manage.delete_others_work',
  global: 'tm.manage.delete_others_global',
} as const

const BULK_CONFIRM_BUTTON_KEYS = {
  both: 'tm.manage.bulk_confirm_button_both',
  work: 'tm.manage.bulk_confirm_button_work',
  global: 'tm.manage.bulk_confirm_button_global',
} as const

const BULK_CONFIRM_HINT_KEYS = {
  both: 'tm.manage.bulk_confirm_hint_both',
  work: 'tm.manage.bulk_confirm_hint_work',
  global: 'tm.manage.bulk_confirm_hint_global',
} as const

const bulkButtonLabel = computed(() =>
  tmManageBulkPending.value
    ? t(BULK_CONFIRM_BUTTON_KEYS[tmManageTierFilter.value], bulkConfirmCount.value)
    : t(BULK_BUTTON_KEYS[tmManageTierFilter.value]),
)

const deleteHintText = computed(() => {
  const hint =
    tmManageCurrentCopyCount.value > 1
      ? t('tm.manage.delete_confirm_hint', { count: String(tmManageCurrentCopyCount.value) })
      : t('tm.manage.delete_confirm_hint_one')
  const hasGlobal = tmManageCurrentRow.value?.copies.some((c) => c.tier === 'global') ?? false
  const withGlobal = hasGlobal ? `${hint} ${t('tm.manage.delete_global_note')}` : hint
  const hidden = hiddenCopiesNote.value
  return hidden === '' ? withGlobal : `${withGlobal} ${hidden}`
})

const hiddenCopiesCount = computed(() => tmManageCurrentRow.value?.hidden_copies ?? 0)

const hiddenCopiesNote = computed(() =>
  hiddenCopiesCount.value > 0 ? t('tm.manage.hidden_copies_note', { count: String(hiddenCopiesCount.value) }) : '',
)

const editHasGlobal = computed(() => tmManageCurrentRow.value?.copies.some((c) => c.tier === 'global') ?? false)

const bulkDoneText = computed(() => {
  const done = tmManageBulkDeleted.value
  if (done === null) return ''
  const params = { work: String(done.work), global: String(done.global) }
  if (tmManageTierFilter.value === 'global' || !tmManageWorkOpen.value) return t('tm.manage.bulk_done_global', params)
  if (tmManageTierFilter.value === 'work') return t('tm.manage.bulk_done_work', params)
  return t('tm.manage.bulk_done', params)
})

function copyTiers(copies: ReadonlyArray<{ tier: string }>): string[] {
  return Array.from(new Set(copies.map((c) => c.tier)))
}

function actionErrorText(err: IpcError): string {
  switch (err.code) {
    case 'tm.pair_not_found':
      return t('tm.fuzzy.pair_gone')
    case 'tm.global_pair_exists':
      return t('tm.manage.push_exists')
    case 'tm.target_empty':
      return t('tm.manage.target_empty')
    default:
      return tError(err)
  }
}

function onExchangeTierChange(value: TmManageTier, event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement && target.checked) setTmManageExchangeTier(value)
}

function onSearchInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) setTmManageSearch(target.value)
}

function onTierChange(event: Event): void {
  const target = event.target
  if (target instanceof HTMLSelectElement) setTmManageTierFilter(target.value as TmManageTierFilter)
}

function onPairOriginChange(event: Event): void {
  const target = event.target
  if (target instanceof HTMLSelectElement) setTmManagePairOriginFilter(target.value as TmManagePairOriginFilter)
}

function onEscape(): void {
  if (tmManageEditing.value) {
    dispatch('tm.manage.cancel')
    return
  }
  if (tmManageDeletePending.value || tmManageBulkPending.value) {
    cancelTmManageDeleteConfirm()
    return
  }
  dispatch('tm.manage.close')
}

function onKeydown(event: KeyboardEvent): void {
  if (event.ctrlKey || event.metaKey || event.altKey) return

  const target = event.target
  const isFormField =
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    target instanceof HTMLButtonElement
  if (isFormField) return

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      dispatch('tm.manage.next')
      return
    case 'ArrowUp':
      event.preventDefault()
      dispatch('tm.manage.prev')
      return
    case 'Enter':
      event.preventDefault()
      dispatch('tm.manage.edit')
      return
    case 'Backspace':
    case 'Delete':
      event.preventDefault()
      if (event.repeat) return
      dispatch('tm.manage.delete')
      return
    default:
      return
  }
}
</script>

<template>
  <div
    v-if="tmManageOverlayIsOpen"
    class="tm-scrim"
    @keydown.esc="onEscape"
    @keydown.tab="trapTab($event)"
    @keydown="onKeydown"
  >
    <section
      ref="panel"
      class="tm-panel"
      tabindex="-1"
      role="dialog"
      aria-modal="true"
      aria-labelledby="tm-manage-title"
    >
      <header class="tm-head">
        <h2 id="tm-manage-title" class="tm-title">
          {{ t('tm.manage.title') }}
        </h2>
        <button type="button" class="tm-close" @click="dispatch('tm.manage.close')">
          {{ t('command.tm.manage.close') }}
        </button>
      </header>

      <p v-if="tmManageStatus === 'loaded' && !tmManageWorkOpen" class="tm-note">
        {{ t('tm.manage.work_note') }}
      </p>

      <ul v-if="tmManageStatus === 'loaded'" class="tm-health" :aria-label="t('tm.manage.health_label')">
        <li v-for="h in tmManageHealth" :key="h.translation_origin" class="tm-badge">
          {{
            t('tm.manage.health_item', {
              label: pairOriginLabel(h.translation_origin),
              count: String(h.count),
              percent: String(healthPercent(h.count)),
            })
          }}
        </li>
      </ul>

      <div class="tm-toolbar">
        <label class="tm-field">
          <span class="tm-field-label">{{ t('tm.manage.search_label') }}</span>
          <input
            type="text"
            class="tm-input"
            autocomplete="off"
            :value="tmManageSearchQuery"
            :disabled="tmManageEditing || tmManageSaving"
            @input="onSearchInput"
          />
        </label>

        <label class="tm-field">
          <span class="tm-field-label">{{ t('tm.manage.tier_filter_label') }}</span>
          <select
            class="tm-input"
            :value="tmManageTierFilter"
            :disabled="tmManageEditing || tmManageSaving"
            @change="onTierChange"
          >
            <option
              v-for="opt in TIER_OPTIONS"
              :key="opt.value"
              :value="opt.value"
              :disabled="opt.value === 'work' && tmManageStatus === 'loaded' && !tmManageWorkOpen"
            >
              {{ t(opt.labelKey) }}
            </option>
          </select>
        </label>

        <label class="tm-field">
          <span class="tm-field-label">{{ t('tm.manage.origin_filter_label') }}</span>
          <select
            class="tm-input"
            :value="tmManagePairOriginFilter"
            :disabled="tmManageEditing || tmManageSaving"
            @change="onPairOriginChange"
          >
            <option v-for="opt in PAIR_ORIGIN_OPTIONS" :key="opt.value" :value="opt.value">
              {{ t(opt.labelKey) }}
            </option>
          </select>
        </label>
      </div>

      <p v-if="emptyReason === 'not_loaded'" class="tm-status" role="status">
        {{ t('tm.manage.loading') }}
      </p>
      <p v-else-if="emptyReason === 'ipc_unavailable'" class="tm-empty">
        {{ t('tm.manage.empty_ipc_unavailable') }}
      </p>
      <p v-else-if="tmManageStatus === 'error' && tmManageLoadError !== null" class="tm-empty tm-error" role="alert">
        {{ tError(tmManageLoadError) }}
      </p>
      <p v-else-if="emptyReason === 'tm_empty'" class="tm-empty">
        {{ t('tm.manage.empty_tm') }}
      </p>

      <template v-else-if="tmManageStatus === 'loaded'">
        <p v-if="emptyReason === 'filter_no_match'" class="tm-empty">
          {{ t('tm.manage.empty_filter_no_match') }}
        </p>
        <template v-else>
          <p v-if="tmManageTotalGroups > tmManageShownGroups" class="tm-status" role="status">
            {{
              t('tm.manage.capped', {
                shown: String(tmManageShownGroups),
                total: String(tmManageTotalGroups),
              })
            }}
          </p>
          <ul
            ref="list"
            class="tm-list"
            role="listbox"
            tabindex="-1"
            :aria-label="t('tm.manage.list_label')"
            :aria-activedescendant="
              tmManageCurrentRow === null ? undefined : optionId(tmManageCurrentRow.tier, tmManageCurrentRow.unit_id)
            "
          >
            <template v-for="(flat, i) in tmManageFlatRows" :key="tmManageRowKey(flat.row)">
              <li v-if="flat.groupHeader" class="tm-group" role="presentation">
                <!-- aura-allow-text: stored source text of the pair, user content. -->
                <span :id="`tm-source-${flat.groupKey}`" class="tm-source">{{ flat.source_text }}</span>
                <span class="tm-badge">{{
                  t('tm.manage.group_header', {
                    count: String(flat.groupDistinctTargets),
                  })
                }}</span>
              </li>
              <li
                :id="optionId(flat.row.tier, flat.row.unit_id)"
                class="tm-row"
                role="option"
                tabindex="-1"
                :aria-labelledby="
                  flat.showSource
                    ? undefined
                    : `tm-source-${flat.groupKey} ${optionId(flat.row.tier, flat.row.unit_id)}`
                "
                :aria-selected="i === tmManageCursor"
                :class="{
                  'tm-row-current': i === tmManageCursor,
                  'tm-row-delete-pending': i === tmManageCursor && tmManageDeletePending,
                }"
              >
                <!-- aura-allow-text: stored source text of the pair, user content. -->
                <span v-if="flat.showSource" class="tm-source">{{ flat.source_text }}</span>
                <!-- aura-allow-text: stored target text of the pair, user content. -->
                <span class="tm-target">{{ flat.row.target_text }}</span>
                <!-- aura-allow-text: result of t() computed in the script. -->
                <span class="tm-badge">{{ pairOriginLabel(flat.row.translation_origin) }}</span>
                <!-- aura-allow-text: result of t() computed in the script. -->
                <span v-for="tier in copyTiers(flat.row.copies)" :key="tier" class="tm-badge">{{
                  tierLabel(tier)
                }}</span>
                <span v-if="flat.row.copies.length > 1" class="tm-badge">{{
                  t('tm.manage.copies', {
                    count: String(flat.row.copies.length),
                  })
                }}</span>
                <!-- aura-allow-text: result of t() computed in the script. -->
                <span class="tm-date">{{ dateLabel(flat.row.created_at) }}</span>
                <span v-if="i === tmManageCursor && tmManageDeletePending" class="tm-badge tm-badge-delete-pending">
                  {{ t('tm.manage.delete_confirm_badge') }}
                </span>
              </li>
            </template>
          </ul>
        </template>
      </template>

      <form
        v-if="tmManageEditing && tmManageCurrentRow !== null"
        class="tm-edit-form"
        @submit.prevent="dispatch('tm.manage.save')"
        @keydown.esc.prevent.stop="dispatch('tm.manage.cancel')"
      >
        <label class="tm-field">
          <span class="tm-field-label">{{ t('tm.manage.target_label') }}</span>
          <input
            ref="editInput"
            v-model="tmManageEditTarget"
            type="text"
            class="tm-input"
            autocomplete="off"
            :disabled="tmManageSaving"
          />
        </label>
        <p class="tm-status">{{ t('tm.manage.edit_hint') }}</p>
        <p v-if="editHasGlobal" class="tm-status">{{ t('tm.manage.edit_global_note') }}</p>
        <p v-if="hiddenCopiesCount > 0" class="tm-status">
          <!-- aura-allow-text: result of t() with the count interpolated. -->
          {{ t('tm.manage.hidden_copies_note', { count: String(hiddenCopiesCount) }) }}
        </p>
        <div class="tm-edit-actions">
          <button type="submit" class="tm-act tm-act-primary" :disabled="tmManageSaving">
            {{ t('tm.manage.save') }}
          </button>
          <button type="button" class="tm-act" :disabled="tmManageSaving" @click="dispatch('tm.manage.cancel')">
            {{ t('tm.manage.cancel') }}
          </button>
        </div>
      </form>

      <!-- aura-allow-text: result of t()/tError() computed in the script. -->
      <p v-if="tmManageActionError !== null" class="tm-status tm-error" role="alert">
        {{ actionErrorText(tmManageActionError) }}
      </p>
      <p v-else-if="tmManageActionNotice === 'push_not_applicable'" class="tm-status" role="status">
        {{ t('tm.manage.push_not_applicable') }}
      </p>
      <p v-else-if="tmManageActionNotice === 'work_not_open'" class="tm-status" role="status">
        {{ t('tm.manage.work_not_open') }}
      </p>
      <p v-else-if="tmManageActionNotice === 'pushed'" class="tm-status" role="status">
        {{ t('tm.manage.pushed') }}
      </p>
      <p v-else-if="tmManageActionNotice === 'pushed_unlisted'" class="tm-status" role="status">
        {{ t('tm.manage.pushed_unlisted') }}
      </p>
      <p v-else-if="tmManageSaving" class="tm-status" role="status">
        {{
          t(
            tmManageSavingAction === 'delete'
              ? 'tm.manage.deleting'
              : tmManageSavingAction === 'push'
                ? 'tm.manage.pushing'
                : tmManageSavingAction === 'bulk'
                  ? 'tm.manage.bulk_deleting'
                  : 'tm.manage.saving',
          )
        }}
      </p>
      <!-- aura-allow-text: result of t() computed in the script. -->
      <p v-else-if="tmManageDeletePending" class="tm-status tm-error" role="status">
        {{ deleteHintText }}
      </p>
      <p v-else-if="tmManageBulkPending" class="tm-status tm-error" role="status">
        {{ t(BULK_CONFIRM_HINT_KEYS[tmManageTierFilter], bulkConfirmCount) }}
      </p>
      <!-- aura-allow-text: result of t() computed in the script. -->
      <p v-else-if="tmManageBulkDeleted !== null" class="tm-status" role="status">
        {{ bulkDoneText }}
      </p>

      <div v-if="!tmManageEditing" class="tm-actions">
        <button
          type="button"
          class="tm-act"
          :disabled="tmManageCurrentRow === null"
          @click="dispatch('tm.manage.edit')"
        >
          {{ t('tm.manage.edit') }}
        </button>
        <button
          type="button"
          class="tm-act"
          :class="{ 'tm-act-danger': tmManageDeletePending }"
          :disabled="tmManageCurrentRow === null"
          @click="dispatch('tm.manage.delete')"
        >
          {{ t(tmManageDeletePending ? 'tm.manage.delete_confirm_button' : 'tm.manage.delete') }}
        </button>
        <button
          type="button"
          class="tm-act"
          :disabled="
            tmManageCurrentRow === null ||
            tmManageCurrentRow.copies.some((c) => c.tier === 'global') ||
            !tmManageWorkOpen
          "
          @click="dispatch('tm.manage.push')"
        >
          {{ t('tm.manage.push') }}
        </button>
        <!-- aura-allow-text: result of t() computed in the script. -->
        <button
          type="button"
          class="tm-act"
          :class="{ 'tm-act-danger': tmManageBulkPending }"
          :disabled="tmManageOthersCount === 0"
          @click="dispatch('tm.manage.delete_others')"
        >
          {{ bulkButtonLabel }}
        </button>
        <button type="button" class="tm-act" @click="dispatch('tm.manage.prev')">
          {{ t('tm.manage.prev') }}
        </button>
        <button type="button" class="tm-act" @click="dispatch('tm.manage.next')">
          {{ t('tm.manage.next') }}
        </button>
      </div>
      <div v-if="!tmManageEditing" class="tm-exchange">
        <fieldset class="tm-exchange-tier" role="radiogroup" :aria-label="t('tm.exchange.tier_label')">
          <legend class="tm-field-label">{{ t('tm.exchange.tier_label') }}</legend>
          <label class="tm-radio-label">
            <input
              type="radio"
              name="tm-exchange-tier"
              :disabled="!tmManageWorkOpen"
              :checked="tmManageExchangeTier === 'work'"
              @change="onExchangeTierChange('work', $event)"
            />
            {{ t('tm.fuzzy.tier_work') }}
          </label>
          <label class="tm-radio-label">
            <input
              type="radio"
              name="tm-exchange-tier"
              :checked="tmManageExchangeTier === 'global'"
              @change="onExchangeTierChange('global', $event)"
            />
            {{ t('tm.fuzzy.tier_global') }}
          </label>
        </fieldset>
        <p v-if="!tmManageWorkOpen" class="tm-status" role="status">{{ t('tm.exchange.work_unavailable') }}</p>
        <p v-else-if="tmManageExchangeTier === 'global'" class="tm-status" role="status">
          {{ t('tm.exchange.global_note') }}
        </p>

        <div class="tm-exchange-actions">
          <button
            type="button"
            class="tm-act"
            :disabled="glossaryExchangeBusy"
            @click="dispatch('tm.manage.export_tmx')"
          >
            {{ t('tm.exchange.export_tmx') }}
          </button>
          <button
            type="button"
            class="tm-act"
            data-tm-import-open
            :disabled="glossaryExchangeBusy"
            @click="dispatch('tm.manage.import_tmx')"
          >
            {{ t('tm.exchange.import_tmx') }}
          </button>
        </div>

        <p v-if="glossaryExchangeBusy && !tmManageExportBusy" class="tm-status" role="status">
          {{ t('tm.exchange.busy_other') }}
        </p>
        <p v-else-if="tmManageExportError !== null" class="tm-status tm-error" role="alert">
          <!-- aura-allow-text: result of tmExchangeErrorText() computed in the script. -->
          {{ tmExchangeErrorText(tmManageExportError) }}
        </p>
        <p v-else-if="tmManageExportIpcUnavailable" class="tm-status" role="status">
          {{ t('tm.exchange.export_ipc_unavailable') }}
        </p>
        <p v-else-if="tmManageExportBusy" class="tm-status" role="status">{{ t('tm.exchange.exporting') }}</p>
        <p v-else-if="tmManageExportedPath !== null" class="tm-status" role="status">
          <!-- aura-allow-text: result of t() with the path interpolated. -->
          {{ t('tm.exchange.export_done', { path: tmManageExportedPath }) }}
          <template v-if="tmManageExportLeftOutCount > 0">
            {{ t('tm.exchange.export_left_out', { count: String(tmManageExportLeftOutCount) }) }}
          </template>
        </p>
        <p v-else-if="tmManageImportDone !== null" class="tm-status" role="status">
          <!-- aura-allow-text: result of t() with the counts interpolated. -->
          {{
            t('tm.exchange.import_done', {
              inserted: String(tmManageImportDone.inserted),
              already: String(tmManageImportDone.already_count),
            })
          }}
          <template v-if="tmManageImportDone.future_dated_count > 0">
            {{ t('tm.exchange.import_future_dated', { count: String(tmManageImportDone.future_dated_count) }) }}
          </template>
        </p>
      </div>
    </section>
  </div>
</template>

<style scoped>
.tm-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: mechanical overlay stacking, same as the other overlays. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.tm-panel {
  width: 100%;
  max-width: 880px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.tm-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.tm-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.tm-close {
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

.tm-note,
.tm-status {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.tm-health {
  list-style: none;
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
}

.tm-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 4);
  margin-bottom: var(--space-panel-block);
}

.tm-field {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  flex: 1;
  min-width: 8rem;
}

.tm-field-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.tm-input {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
}

.tm-empty {
  margin: 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.tm-error {
  color: var(--color-error);
}

.tm-list {
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  max-height: 50vh;
  overflow: auto;
  border: 1px solid var(--color-outline);
}

.tm-group {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 3);
  padding: calc(var(--space-unit) * 2) var(--space-panel-inline) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.tm-row {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 3);
  padding: calc(var(--space-unit) * 2) var(--space-panel-inline);
  border-bottom: 1px solid var(--color-outline);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.tm-row:last-child {
  border-bottom: none;
}

.tm-row-current {
  background: var(--color-surface-accent);
}

.tm-row-delete-pending {
  border-color: var(--color-error);
}

.tm-badge-delete-pending {
  color: var(--color-error);
  border-color: var(--color-error);
}

.tm-source {
  font-weight: var(--weight-ui-md-strong);
}

.tm-target {
  flex: 1;
  min-width: 8rem;
  color: var(--color-on-surface);
}

.tm-badge {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  border: 1px solid var(--color-outline);
  padding: 0 calc(var(--space-unit) * 1);
}

.tm-date {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.tm-edit-form {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 4);
  align-items: flex-end;
  margin-bottom: var(--space-panel-block);
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
}

.tm-edit-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}

.tm-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-panel-inline);
}

.tm-act {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: none;
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  cursor: pointer;
}

.tm-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.tm-act-primary {
  border-color: var(--color-primary);
}

.tm-act-danger {
  color: var(--color-error);
  border-color: var(--color-error);
}

.tm-exchange {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
  margin-top: var(--space-panel-block);
  padding-top: var(--space-panel-block);
  border-top: 1px solid var(--color-outline);
}

.tm-exchange-tier {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 3);
  margin: 0;
  padding: 0;
  border: none;
}

.tm-radio-label {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
  cursor: pointer;
}

.tm-exchange-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}
</style>
