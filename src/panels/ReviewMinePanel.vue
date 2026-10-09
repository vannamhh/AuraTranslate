<script setup lang="ts">
import { t } from '../i18n'
import type { DockviewPanelProps } from '../layout/panelProps'
import { reviewModeSegments } from '../reviewModeState'
import PanelFrame from './PanelFrame.vue'

defineProps<DockviewPanelProps>()
</script>

<template>
  <PanelFrame owner="panel.review_mine" status-key="panel.review_mine.status" :show-status="false">
    <ol class="review-list" data-review-mine>
      <li v-for="s in reviewModeSegments" :key="s.id" class="review-item" data-review-mine-item>
        <span v-if="s.target_text === ''" class="review-empty">{{ t('review.untranslated') }}</span>
        <!-- aura-allow-text: DỮ LIỆU (bản dịch của người dùng). -->
        <template v-else>{{ s.target_text }}</template>
      </li>
    </ol>
  </PanelFrame>
</template>

<style scoped>
.review-list {
  flex: 1;
  min-height: 0;
  margin: 0;
  padding: 0;
  list-style: none;
  overflow: auto;
}

.review-item {
  padding: calc(var(--space-unit) * 1) 0;
  white-space: pre-wrap;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.review-empty {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}
</style>
