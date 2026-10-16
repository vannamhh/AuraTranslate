<script setup lang="ts">
// Data section of the Settings frame: the Library folder, a rescan and the index rebuild, all
// on commands and state Library mode already has.
import { onMounted } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import {
  currentLibraryRoot,
  libraryRebuiltAtLabel,
  libraryRescanBusy,
  libraryRescanError,
  loadLibraryRebuiltAt,
} from './modes/libraryRescan'

onMounted(() => {
  void loadLibraryRebuiltAt()
})
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
      <button type="button" class="sd-btn" data-settings-data-choose :disabled="libraryRescanBusy" @click="dispatch('library.choose_root')">
        {{ t('mode.library.choose_root') }}
      </button>
      <button type="button" class="sd-btn" data-settings-data-rescan :disabled="libraryRescanBusy" @click="dispatch('library.rescan')">
        {{ t('mode.library.rescan') }}
      </button>
    </div>
    <div class="sd-actions">
      <button type="button" class="sd-btn" data-settings-data-report @click="dispatch('settings.data.open_report')">
        {{ t('command.settings.data.open_report') }}
      </button>
    </div>
    <p class="sd-note">{{ t('settings.data.report_note') }}</p>

    <div class="sd-row">
      <span class="sd-label">{{ t('settings.data.index_label') }}</span>
      <button type="button" class="sd-btn" data-settings-data-rebuild :disabled="libraryRescanBusy" @click="dispatch('library.rebuild_index')">
        {{ t('mode.library.rebuild_index') }}
      </button>
      <span v-if="libraryRebuiltAtLabel !== null" class="sd-when" data-settings-data-rebuilt-at>
        {{ t('mode.library.rebuilt_at', { when: libraryRebuiltAtLabel }) }}
      </span>
    </div>
    <p class="sd-note">{{ t('mode.library.rebuild_index_note') }}</p>

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
  align-items: center;
  gap: var(--space-panel-inline);
  padding: var(--space-panel-block) 0;
}

.sd-label {
  min-width: 10em;
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
  gap: var(--space-panel-inline);
}

.sd-btn {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: none;
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.sd-when,
.sd-note {
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
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
