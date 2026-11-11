<script setup lang="ts">
import { watch } from 'vue'
import { dispatch } from './commands'
import { t, tError } from './i18n'
import {
  QUICK_AI_FIELDS, quickAiConfig, quickAiConfigDrafts, quickAiConfigLocked,
  quickAiConfigLoadError, quickAiConfigSaveError, quickAiConfigSaving,
  quickAiConfigKeyDraft, quickAiConfigKeyBusy, quickAiConfigKeyError,
  quickAiConfigTestStatus, quickAiConfigTestError, quickAiConfigTestModelCount,
  setQuickAiConfigDraft, setQuickAiConfigKeyDraft, discardQuickAiConfigKeyDraft, commitQuickAiConfigDrafts,
} from './quickAiConfigState'
import type { QuickAiConfigField } from './config/quickAiConfig'

function fieldValue(field: QuickAiConfigField): string {
  return quickAiConfigLocked.value ? quickAiConfig.value?.fields.find(entry => entry.field === field)?.value ?? '' : quickAiConfigDrafts.value[field]
}
function onInput(field: QuickAiConfigField, event: Event): void {
  if (event.target instanceof HTMLInputElement) setQuickAiConfigDraft(field, event.target.value)
}
function onKeyInput(event: Event): void {
  if (event.target instanceof HTMLInputElement) setQuickAiConfigKeyDraft(event.target.value)
}
function fieldLabel(field: QuickAiConfigField): string {
  switch (field) {
    case 'quick_provider': return 'settings.ai_config.field_provider'
    case 'quick_endpoint': return 'settings.ai_config.field_endpoint'
    case 'quick_model': return 'settings.ai_config.field_model'
  }
}
function keyStatus(): string {
  const configured = quickAiConfig.value?.key_configured
  return configured === true ? 'settings.ai_config.key_status_configured' : configured === false ? 'settings.ai_config.key_status_not_configured' : 'settings.ai_config.key_status_unknown'
}
function configStatus(): string {
  switch (quickAiConfig.value?.source) {
    case 'main_global': return 'settings.quick_ai.fallback'
    case 'incomplete': return 'settings.quick_ai.incomplete'
    default: return 'settings.quick_ai.dedicated'
  }
}
watch(quickAiConfigLocked, (locked) => {
  if (locked) discardQuickAiConfigKeyDraft()
})
</script>

<template>
  <section class="quick-ai" data-quick-ai-section :aria-label="t('settings.quick_ai.title')">
    <h4>{{ t('settings.quick_ai.title') }}</h4>
    <p>{{ t('settings.quick_ai.shared') }}</p>
    <p v-if="quickAiConfigLocked" data-quick-ai-locked>{{ t('settings.quick_ai.locked') }}</p>
    <p v-if="quickAiConfigLoadError !== null" role="alert">{{ tError(quickAiConfigLoadError) }}</p>
    <fieldset :disabled="quickAiConfigLocked || quickAiConfigSaving" class="quick-ai-fields">
      <label v-for="field in QUICK_AI_FIELDS" :key="field">
        <span>{{ t(fieldLabel(field)) }}</span>
        <input class="ui-field" type="text" :data-quick-ai-field="field" :value="fieldValue(field)" @input="onInput(field, $event)" @blur="commitQuickAiConfigDrafts()" />
      </label>
    </fieldset>
    <p v-if="quickAiConfigSaveError !== null" role="alert">{{ tError(quickAiConfigSaveError) }}</p>
    <label class="quick-ai-key">
      <span>{{ t('settings.ai_config.key_label') }}</span>
      <input class="ui-field" type="password" autocomplete="new-password" :value="quickAiConfigKeyDraft" :disabled="quickAiConfigLocked || quickAiConfigKeyBusy" data-quick-ai-key @input="onKeyInput" />
    </label>
    <p data-quick-ai-key-status>{{ t(keyStatus()) }}</p>
    <p v-if="quickAiConfigKeyError !== null" role="alert">{{ tError(quickAiConfigKeyError) }}</p>
    <div class="quick-ai-actions">
      <button class="ui-btn ui-btn--lg" type="button" :disabled="quickAiConfigLocked || quickAiConfigKeyBusy || quickAiConfigKeyDraft === ''" @click="dispatch('settings.quick_ai.key_save')">{{ t('settings.ai_config.key_save') }}</button>
      <button class="ui-btn ui-btn--danger ui-btn--lg" type="button" :disabled="quickAiConfigLocked || quickAiConfigKeyBusy || quickAiConfig?.key_configured === false" @click="dispatch('settings.quick_ai.key_delete')">{{ t('settings.ai_config.key_delete') }}</button>
      <button class="ui-btn ui-btn--lg" type="button" :disabled="quickAiConfigLocked || quickAiConfigTestStatus === 'running'" @click="dispatch('settings.quick_ai.test')">{{ t('settings.ai_config.test_connection') }}</button>
      <button v-if="quickAiConfigTestStatus === 'running'" class="ui-btn ui-btn--quiet ui-btn--lg" type="button" @click="dispatch('settings.quick_ai.test_cancel')">{{ t('settings.ai_config.test_cancel') }}</button>
    </div>
    <p v-if="quickAiConfigTestStatus === 'running'" role="status" data-quick-ai-test="running">{{ t('settings.ai_config.test_running') }}</p>
    <p v-else-if="quickAiConfigTestStatus === 'connected'" role="status" data-quick-ai-test="connected">{{ t('settings.ai_config.test_connected', { count: String(quickAiConfigTestModelCount) }) }}</p>
    <p v-else-if="quickAiConfigTestStatus === 'not_configured'" role="status" data-quick-ai-test="not_configured">{{ t('settings.ai_config.test_not_configured') }}</p>
    <p v-else-if="quickAiConfigTestStatus === 'cancelled'" role="status" data-quick-ai-test="cancelled">{{ t('settings.ai_config.test_cancelled') }}</p>
    <p v-else-if="quickAiConfigTestError !== null" role="alert" data-quick-ai-test="error">{{ tError(quickAiConfigTestError) }}</p>
    <p v-if="quickAiConfig !== null" data-quick-ai-config-status>{{ t(configStatus()) }}</p>
  </section>
</template>

<style scoped>
.quick-ai { border-top: 1px solid var(--color-outline); margin-top: calc(var(--space-unit) * 4); padding-top: calc(var(--space-unit) * 4); }
.quick-ai-fields { border: 0; padding: 0; margin: 0; display: grid; gap: calc(var(--space-unit) * 3); }
.quick-ai-fields label, .quick-ai-key { display: grid; gap: var(--space-unit); }
.quick-ai-actions { display: flex; flex-wrap: wrap; gap: calc(var(--space-unit) * 2); }
</style>
