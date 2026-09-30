<script setup lang="ts">
// Glossary scan-threshold section of the Settings frame (FR47). The threshold is `AppConfig`
// ⇒ `GlobalOnly` (`core/scope/kinds.rs`), so there is deliberately no scope switch here.
import { onMounted } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import {
  glossarySettingsSaveError,
  glossarySettingsSaved,
  glossarySettingsSaving,
  glossarySettingsThresholdInput,
  loadGlossarySettingsForm,
  markGlossarySettingsEdited,
  parsedGlossaryScanThreshold,
} from './glossarySettingsState'

onMounted(loadGlossarySettingsForm)

function onInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) {
    glossarySettingsThresholdInput.value = target.value
    markGlossarySettingsEdited()
  }
}
</script>

<template>
  <!-- The submit button is the form's only route to `dispatch`; a second `@click` on it would
       be a second dispatch path that Check A cannot see. -->
  <form class="gs-form" @submit.prevent="dispatch('glossary.settings.save')">
    <fieldset class="gs-fieldset" :disabled="glossarySettingsSaving">
      <label class="gs-field">
        <span class="gs-field-label">{{ t('glossary.settings.threshold_label') }}</span>
        <input
          type="number"
          class="gs-input"
          min="1"
          step="1"
          autocomplete="off"
          :value="glossarySettingsThresholdInput"
          @input="onInput"
        />
      </label>

      <p v-if="parsedGlossaryScanThreshold(glossarySettingsThresholdInput) === null" class="gs-alert">
        {{ t('glossary.settings.threshold_invalid') }}
      </p>
      <p v-else-if="glossarySettingsSaveError !== null" class="gs-alert" role="alert">
        {{ tError(glossarySettingsSaveError) }}
      </p>
      <p v-else-if="glossarySettingsSaved" class="gs-saved" role="status">
        {{ t('glossary.settings.saved') }}
      </p>

      <div class="gs-actions">
        <button
          type="submit"
          class="gs-save"
          :disabled="parsedGlossaryScanThreshold(glossarySettingsThresholdInput) === null"
        >
          {{ t('command.glossary.settings.save') }}
        </button>
      </div>
    </fieldset>
  </form>
</template>

<style scoped>
.gs-form {
  display: block;
}

.gs-fieldset {
  margin: 0;
  padding: 0;
  border: none;
}

.gs-field {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin-bottom: var(--space-panel-block);
}

.gs-field-label {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.gs-input {
  width: 8em;
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
}

.gs-alert,
.gs-saved {
  margin: 0 0 var(--space-panel-block) 0;
  padding-left: calc(var(--space-unit) * 3);
  border-left: 2px solid var(--color-error);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-error);
}

.gs-saved {
  border-left-color: var(--color-outline);
  color: var(--color-on-surface-variant);
}

.gs-actions {
  display: flex;
  gap: var(--space-panel-inline);
}

.gs-save {
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
