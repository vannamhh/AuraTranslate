<script setup lang="ts">
// Data section of the Settings frame: the Library folder, a rescan and the index rebuild, all
// on commands and state Library mode already has.
import { nextTick, onMounted, ref, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import {
  currentLibraryRoot,
  libraryRebuiltAtLabel,
  libraryRescanBusy,
  libraryRescanError,
  loadLibraryRebuiltAt,
} from './modes/libraryRescan'
import {
  commitQuickDataDays,
  formatQuickDataBytes,
  formatQuickDataCount,
  loadQuickDataRow,
  quickDataBusy,
  quickDataCanPurgeAll,
  quickDataCanPurgeOlder,
  quickDataCompactError,
  quickDataConfirmingAll,
  quickDataDaysInput,
  quickDataDaysSaveError,
  quickDataDaysValid,
  quickDataOlderCount,
  quickDataOlderError,
  quickDataPurgeError,
  quickDataResult,
  quickDataStats,
  quickDataStatsError,
  quickDataStatsLoaded,
  quickDataResultText,
  setQuickDataDaysInput,
} from './settingsQuickDataState'

onMounted(() => {
  void loadLibraryRebuiltAt()
  void loadQuickDataRow()
})

const purgeAllButton = ref<HTMLButtonElement | null>(null)
const confirmButton = ref<HTMLButtonElement | null>(null)

function focusIfLost(target: HTMLElement | null): void {
  const active = document.activeElement
  if (target !== null && !target.hasAttribute('disabled') && (active === null || active === document.body)) target.focus()
}

watch(quickDataConfirmingAll, async (confirming) => {
  await nextTick()
  if (confirming) confirmButton.value?.focus()
  else if (!quickDataBusy.value) purgeAllButton.value?.focus()
})

watch(quickDataBusy, async (isBusy) => {
  if (isBusy) return
  await nextTick()
  focusIfLost(purgeAllButton.value)
})

function onDaysInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) setQuickDataDaysInput(target.value)
}

function onDaysCommit(): void {
  void commitQuickDataDays()
}
</script>

<template>
  <div class="sd-body" data-settings-data>
    <div class="sd-row">
      <span class="sd-label">{{ t('settings.data.root_label') }}</span>
      <!-- aura-allow-text: đường dẫn thư mục — DỮ LIỆU, hoặc câu chưa quét lần nào. -->
      <span class="sd-root" data-settings-data-root>{{
        currentLibraryRoot ?? t('mode.library.root_not_scanned_yet')
      }}</span>
    </div>
    <div class="sd-actions">
      <button type="button" class="sd-btn ui-btn ui-btn--lg" data-settings-data-choose :disabled="libraryRescanBusy" @click="dispatch('library.choose_root')">
        {{ t('mode.library.choose_root') }}
      </button>
      <button type="button" class="sd-btn ui-btn ui-btn--lg" data-settings-data-rescan :disabled="libraryRescanBusy" @click="dispatch('library.rescan')">
        {{ t('mode.library.rescan') }}
      </button>
      <button type="button" class="sd-btn ui-btn ui-btn--lg" data-settings-data-report @click="dispatch('settings.data.open_report')">
        {{ t('command.settings.data.open_report') }}
      </button>
    </div>
    <p class="sd-note">{{ t('settings.data.report_note') }}</p>

    <div class="sd-row">
      <span class="sd-label">{{ t('settings.data.index_label') }}</span>
      <button type="button" class="sd-btn ui-btn ui-btn--lg" data-settings-data-rebuild :disabled="libraryRescanBusy" @click="dispatch('library.rebuild_index')">
        {{ t('mode.library.rebuild_index') }}
      </button>
      <span v-if="libraryRebuiltAtLabel !== null" class="sd-when" data-settings-data-rebuilt-at>
        {{ t('mode.library.rebuilt_at', { when: libraryRebuiltAtLabel }) }}
      </span>
    </div>
    <p class="sd-note">{{ t('mode.library.rebuild_index_note') }}</p>

    <div class="sd-row" data-settings-quick-data>
      <span class="sd-label">{{ t('settings.data.quick_label') }}</span>
      <span v-if="quickDataStatsError !== null" class="sd-alert" role="alert" data-settings-quick-data-error>
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(quickDataStatsError) }}
      </span>
      <span v-else-if="!quickDataStatsLoaded" class="sd-when">{{ t('settings.data.quick_loading') }}</span>
      <span v-else-if="quickDataStats === null || quickDataStats.count === 0" class="sd-when" data-settings-quick-data-empty>
        {{ t('settings.data.quick_empty') }}
      </span>
      <span v-else class="sd-when" data-settings-quick-data-summary>
        {{
          t('settings.data.quick_summary', {
            count: formatQuickDataCount(quickDataStats.count),
            size: formatQuickDataBytes(quickDataStats.bytes),
          })
        }}
      </span>
    </div>
    <ul v-if="quickDataStats !== null && quickDataStats.count > 0" class="sd-models" data-settings-quick-data-models>
      <li v-for="row in quickDataStats.by_model" :key="row.model" class="sd-note">
        <!-- aura-allow-text: tên mô hình — DỮ LIỆU — kèm số câu đã định dạng. -->
        {{ t('settings.data.quick_model_line', { model: row.model, count: formatQuickDataCount(row.count) }) }}
      </li>
    </ul>

    <div class="sd-row">
      <label class="sd-older">
        <span class="sd-when">{{ t('settings.data.quick_older_before') }}</span>
        <input
          type="number"
          class="ui-field ui-field--lg ui-field--short"
          min="1"
          step="1"
          autocomplete="off"
          data-settings-quick-data-days
          :value="quickDataDaysInput"
          :disabled="quickDataBusy"
          :aria-invalid="!quickDataDaysValid"
          @input="onDaysInput"
          @change="onDaysCommit"
        />
        <span class="sd-when">{{ t('settings.data.quick_older_after') }}</span>
      </label>
      <span v-if="!quickDataDaysValid" class="sd-alert" role="alert" data-settings-quick-data-days-invalid>
        {{ t('settings.data.quick_days_invalid') }}
      </span>
      <span v-else-if="quickDataOlderError !== null" class="sd-alert" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(quickDataOlderError) }}
      </span>
      <span v-else-if="quickDataOlderCount !== null" class="sd-when" data-settings-quick-data-will-delete>
        {{ t('settings.data.quick_will_delete', { count: formatQuickDataCount(quickDataOlderCount) }) }}
      </span>
      <button
        type="button"
        class="sd-btn ui-btn ui-btn--lg"
        data-settings-quick-data-purge-older
        :disabled="!quickDataCanPurgeOlder"
        @click="dispatch('settings.quick_data.purge_older')"
      >
        {{ t('settings.data.quick_purge_older') }}
      </button>
    </div>
    <p v-if="quickDataDaysSaveError !== null" class="sd-alert" role="alert" data-settings-quick-data-days-save-error>
      <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
      {{ tError(quickDataDaysSaveError) }}
    </p>
    <div class="sd-actions">
      <template v-if="quickDataConfirmingAll && quickDataStats !== null">
        <button
          type="button"
          class="sd-btn ui-btn ui-btn--lg ui-btn--danger"
          ref="confirmButton"
          data-settings-quick-data-purge-all-confirm
          :disabled="!quickDataCanPurgeAll"
          @click="dispatch('settings.quick_data.purge_all_confirm')"
        >
          {{ t('settings.data.quick_purge_all_confirm', { count: formatQuickDataCount(quickDataStats.count) }) }}
        </button>
        <button
          type="button"
          class="sd-btn ui-btn ui-btn--lg"
          data-settings-quick-data-purge-all-cancel
          @click="dispatch('settings.quick_data.purge_all_cancel')"
        >
          {{ t('settings.data.quick_purge_all_cancel') }}
        </button>
      </template>
      <button
        v-else
        type="button"
        class="sd-btn ui-btn ui-btn--lg ui-btn--danger"
        ref="purgeAllButton"
        data-settings-quick-data-purge-all
        :disabled="!quickDataCanPurgeAll"
        @click="dispatch('settings.quick_data.purge_all')"
      >
        {{ t('settings.data.quick_purge_all') }}
      </button>
    </div>
    <p v-if="quickDataBusy" class="sd-note" role="status" data-settings-quick-data-busy>
      {{ t('settings.data.quick_purging') }}
    </p>
    <template v-else>
      <p v-if="quickDataResult !== null" class="sd-note" role="status" data-settings-quick-data-result>
        <!-- aura-allow-text: KẾT QUẢ của `quickDataResultText()`. -->
        {{ quickDataResultText(quickDataResult) }}
      </p>
      <p v-if="quickDataCompactError !== null" class="sd-alert" role="alert" data-settings-quick-data-compact-error>
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(quickDataCompactError) }}
      </p>
      <p v-if="quickDataPurgeError !== null" class="sd-alert" role="alert" data-settings-quick-data-purge-error>
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(quickDataPurgeError) }}
      </p>
    </template>

    <p v-if="libraryRescanError !== null" class="sd-alert" role="alert" data-settings-data-error>
      <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
      {{ tError(libraryRescanError) }}
    </p>
  </div>
</template>

<style scoped>
.sd-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: var(--space-panel-inline);
  padding: var(--space-panel-block) 0;
}

.sd-label {
  min-width: 133px;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.sd-root {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface);
  overflow-wrap: anywhere;
}

.sd-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-unit);
  margin-bottom: calc(var(--space-unit) * 1);
}

.sd-when,
.sd-note {
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.sd-older {
  display: inline-flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-unit);
}

.sd-models {
  margin: 0;
  padding: 0 0 0 var(--space-panel-inline);
  list-style: none;
}

.sd-models .sd-note {
  margin: 0;
}

.sd-note {
  margin: 0 0 var(--space-panel-block) 0;
}

.sd-alert {
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  color: var(--color-error);
}
</style>
