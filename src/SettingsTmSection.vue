<script setup lang="ts">
// Fuzzy TM threshold section of the Settings frame. `AppConfig` is `GlobalOnly`, so there is
// deliberately no scope switch here.
import { onMounted } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import {
  TM_FUZZY_THRESHOLD_MAX,
  TM_FUZZY_THRESHOLD_MIN,
  loadTmSettingsForm,
  markTmSettingsEdited,
  parsedTmFuzzyThreshold,
  tmSettingsSaveError,
  tmSettingsSaved,
  tmSettingsSaving,
  tmSettingsThresholdInput,
} from './tmSettingsState'

onMounted(loadTmSettingsForm)

function onInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) {
    tmSettingsThresholdInput.value = target.value
    markTmSettingsEdited()
  }
}
</script>

<template>
  <form class="ts-form" @submit.prevent="dispatch('tm.settings.save')">
    <fieldset class="ts-fieldset" :disabled="tmSettingsSaving">
      <label class="ts-field">
        <span class="ts-field-label">{{ t('tm.settings.threshold_label') }}</span>
        <input
          type="number"
          class="ts-input"
          :min="TM_FUZZY_THRESHOLD_MIN"
          :max="TM_FUZZY_THRESHOLD_MAX"
          step="1"
          autocomplete="off"
          :value="tmSettingsThresholdInput"
          @input="onInput"
        />
      </label>

      <p v-if="parsedTmFuzzyThreshold(tmSettingsThresholdInput) === null" class="ts-alert">
        {{ t('tm.settings.threshold_invalid', { min: String(TM_FUZZY_THRESHOLD_MIN), max: String(TM_FUZZY_THRESHOLD_MAX) }) }}
      </p>
      <p v-else-if="tmSettingsSaveError !== null" class="ts-alert" role="alert">
        {{ tError(tmSettingsSaveError) }}
      </p>
      <p v-else-if="tmSettingsSaved" class="ts-saved" role="status">
        {{ t('tm.settings.saved') }}
      </p>

      <div class="ts-actions">
        <button type="submit" class="ts-save" :disabled="parsedTmFuzzyThreshold(tmSettingsThresholdInput) === null">
          {{ t('command.tm.settings.save') }}
        </button>
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
  width: 8em;
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
