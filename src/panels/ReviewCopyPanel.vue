<script setup lang="ts">
import { t } from '../i18n'
import type { DockviewPanelProps } from '../layout/panelProps'
import { reviewModeRows } from '../reviewModeState'
import PanelFrame from './PanelFrame.vue'

defineProps<DockviewPanelProps>()
</script>

<template>
  <PanelFrame owner="panel.review_copy" status-key="panel.review_copy.status" :show-status="false">
    <ol class="review-list" data-review-copy>
      <li v-for="r in reviewModeRows" :key="r.id" class="review-item" data-review-copy-item>
        <span v-if="r.kind === 'alt'" class="review-kind">{{ t('review.row_alt') }}</span>
        <span v-else-if="r.kind === 'caption'" class="review-kind">{{ t('review.row_caption') }}</span>
        <span v-if="r.target_text === ''" class="review-empty">{{ t('review.row_empty') }}</span>
        <!-- aura-allow-text: DỮ LIỆU (bản của Reviewer). -->
        <template v-else>{{ r.target_text }}</template>
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

.review-kind,
.review-empty {
  margin-right: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}
</style>
