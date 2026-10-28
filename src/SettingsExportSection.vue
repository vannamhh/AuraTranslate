<script setup lang="ts">
import { onMounted } from 'vue'
import { t, tError } from './i18n'
import {
  exportSettingsNameInput,
  exportSettingsSaveError,
  exportSettingsSaved,
  exportSettingsSaving,
  loadExportSettingsForm,
  markExportSettingsEdited,
  saveExportSettings,
} from './exportSettingsState'

onMounted(() => {
  void loadExportSettingsForm()
})

function onInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) {
    exportSettingsNameInput.value = target.value
    markExportSettingsEdited()
  }
}
</script>

<template>
  <form class="ts-form" @submit.prevent="saveExportSettings">
    <fieldset class="ts-fieldset" :disabled="exportSettingsSaving">
      <label class="ts-field">
        <span class="ts-field-label">{{ t('export.settings.translator_label') }}</span>
        <input
          type="text"
          class="ts-input ui-field ui-field--lg ui-field--medium"
          autocomplete="off"
          data-export-translator-input
          :value="exportSettingsNameInput"
          @input="onInput"
        />
      </label>

      <p v-if="exportSettingsSaveError !== null" class="ts-alert" role="alert">
        <!-- aura-allow-text: result of tError() on the IPC error. -->
        {{ tError(exportSettingsSaveError) }}
      </p>
      <p v-else-if="exportSettingsSaved" class="ts-saved" role="status">
        {{ t('export.settings.saved') }}
      </p>

      <div class="ts-actions">
        <button type="submit" class="ts-save ui-btn ui-btn--lg">{{ t('export.settings.save') }}</button>
      </div>
    </fieldset>
  </form>
</template>

<style scoped>
.ts-form {
  display: block;
}

.ts-fieldset {
  margin: 0;
  padding: 0;
  border: none;
}

.ts-field {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin-bottom: var(--space-panel-block);
}

.ts-field-label {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.ts-alert,
.ts-saved {
  margin: 0 0 var(--space-panel-block) 0;
  padding-left: calc(var(--space-unit) * 3);
  border-left: 2px solid var(--color-error);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-error);
}

.ts-saved {
  border-left-color: var(--color-outline);
  color: var(--color-on-surface-variant);
}

.ts-actions {
  display: flex;
  gap: var(--space-panel-inline);
}
</style>
