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
  <form class="ts-form" @submit.prevent="void saveExportSettings()">
    <fieldset class="ts-fieldset" :disabled="exportSettingsSaving">
      <label class="ts-field">
        <span class="ts-field-label">{{ t('export.settings.translator_label') }}</span>
        <input
          type="text"
          class="ts-input"
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
        <button type="submit" class="ts-save">{{ t('export.settings.save') }}</button>
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

.ts-input {
  width: 24em;
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
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

.ts-save {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: none;
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}
</style>
