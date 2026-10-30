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
      <div class="gs-row">
        <label class="gs-field">
          <span class="gs-field-label">{{ t('glossary.settings.threshold_label') }}</span>
          <input
            type="number"
            class="gs-input ui-field ui-field--lg ui-field--short"
            min="1"
            step="1"
            autocomplete="off"
            :value="glossarySettingsThresholdInput"
            @input="onInput"
          />
        </label>
        <button
          type="submit"
          class="gs-save ui-btn ui-btn--lg"
          :disabled="parsedGlossaryScanThreshold(glossarySettingsThresholdInput) === null"
        >
          {{ t('command.glossary.settings.save') }}
        </button>
      </div>

      <p v-if="parsedGlossaryScanThreshold(glossarySettingsThresholdInput) === null" class="gs-alert">
        {{ t('glossary.settings.threshold_invalid') }}
      </p>
      <p v-else-if="glossarySettingsSaveError !== null" class="gs-alert" role="alert">
        {{ tError(glossarySettingsSaveError) }}
      </p>
      <p v-else-if="glossarySettingsSaved" class="gs-saved" role="status">
        {{ t('glossary.settings.saved') }}
      </p>
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

.gs-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.gs-field {
  display: flex;
  align-items: baseline;
  gap: var(--space-unit);
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

.gs-alert,
.gs-saved {
  margin: 0 0 var(--space-panel-block) 0;
  padding-left: 12px; /* aura-allow-spacing: 2px left border sits inside the 14px inset */
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
</style>
