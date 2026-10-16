<script setup lang="ts">
// Update section of the Settings frame (frame only): states what the app does and does not do.
// There is no switch and no button here; the version line and the check belong to a later story.
import { t } from './i18n'
</script>

<template>
  <div class="su-body" data-settings-update>
    <p class="su-lead">{{ t('settings.update.lead') }}</p>
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
