<script setup lang="ts">
// Update section of the Settings frame: version line, the one switch, a manual check and its outcome.
// The check never runs on its own (AD-15); every request starts from the button below.
import { t, tError } from './i18n'
import {
  openUpdateReleasePage,
  runUpdateCheck,
  setUpdateCheckEnabled,
  updateAppVersion,
  updateCheckEnabled,
  updateCheckError,
  updateChecking,
  updateOpenError,
  updateOutcome,
  updateSaveError,
} from './updateCheckState'

function onToggle(): void {
  void setUpdateCheckEnabled(!updateCheckEnabled.value)
}
</script>

<template>
  <div class="su-body" data-settings-update>
    <p class="su-lead">{{ t('settings.update.lead') }}</p>
    <p class="su-row" data-update-version>{{ t(updateAppVersion === '' ? 'settings.update.version_unknown' : 'settings.update.version', { version: updateAppVersion }) }}</p>

    <div class="su-row">
      <span class="su-label">{{ t('settings.update.switch_label') }}</span>
      <form class="su-form" @submit.prevent="onToggle()">
        <button type="submit" class="su-btn" :aria-pressed="updateCheckEnabled" data-update-toggle>
          {{ t(updateCheckEnabled ? 'settings.update.switch_on' : 'settings.update.switch_off') }}
        </button>
      </form>
    </div>
    <p v-if="updateSaveError !== null" class="su-alert" role="alert">
      <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
      {{ tError(updateSaveError) }}
    </p>

    <form class="su-form su-row" @submit.prevent="runUpdateCheck()">
      <button type="submit" class="su-btn" :disabled="!updateCheckEnabled || updateChecking" data-update-check>
        {{ t(updateChecking ? 'settings.update.checking' : 'settings.update.check') }}
      </button>
    </form>

    <p v-if="updateCheckError !== null" class="su-alert" role="alert" data-update-error>
      <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
      {{ tError(updateCheckError) }}
    </p>
    <div v-if="updateOutcome !== null" class="su-row" role="status" data-update-outcome :data-update-kind="updateOutcome.kind">
      <p v-if="updateOutcome.kind === 'up_to_date'" class="su-result">
        {{ t('settings.update.up_to_date', { version: updateOutcome.current }) }}
      </p>
      <p v-else-if="updateOutcome.kind === 'no_app_release'" class="su-result">
        {{ t('settings.update.no_app_release') }}
      </p>
      <template v-else>
        <p class="su-result">{{ t('settings.update.available', { version: updateOutcome.latest }) }}</p>
        <form class="su-form" @submit.prevent="openUpdateReleasePage()">
          <button type="submit" class="su-btn" data-update-open>{{ t('settings.update.open_page') }}</button>
        </form>
        <p class="su-hint" data-update-checksum>{{ t('settings.update.checksum_hint') }}</p>
        <p v-if="updateOpenError !== null" class="su-alert" role="alert">
          <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
          {{ tError(updateOpenError) }}
        </p>
      </template>
    </div>

    <div class="su-note" data-settings-update-why>
      <strong class="su-note-title">{{ t('settings.update.why_title') }}</strong>
      {{ t('settings.update.why_body') }}
    </div>
  </div>
</template>

<style scoped>
.su-lead {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface);
}

.su-row {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: calc(var(--space-unit) * 1);
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface);
}

.su-form {
  margin: 0;
}

.su-label,
.su-hint {
  margin: 0;
  color: var(--color-on-surface-variant);
}

.su-result {
  margin: 0;
}

.su-btn {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: none;
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.su-btn:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.su-alert {
  margin: 0 0 var(--space-panel-block) 0;
  padding-left: calc(var(--space-unit) * 3);
  border-left: 2px solid var(--color-error);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-error);
}

.su-note {
  padding-left: 12px;
  border-left: 2px solid var(--color-tm-rule);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-tm-text);
}

.su-note-title {
  color: var(--color-on-surface);
}
</style>
