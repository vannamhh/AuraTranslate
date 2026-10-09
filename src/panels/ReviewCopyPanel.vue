<script setup lang="ts">
import { onBeforeUnmount, onMounted, useTemplateRef } from 'vue'
import { t } from '../i18n'
import type { DockviewPanelProps } from '../layout/panelProps'
import { reviewModeCurrentPairKey, reviewModeCopyItems } from '../reviewModeState'
import { onReviewScroll, registerReviewScroller } from '../reviewScrollSync'
import PanelFrame from './PanelFrame.vue'
import ReviewSpans from './ReviewSpans.vue'

defineProps<DockviewPanelProps>()

const list = useTemplateRef<HTMLElement>('list')

onMounted(() => registerReviewScroller('copy', list.value))
onBeforeUnmount(() => registerReviewScroller('copy', null))
</script>

<template>
  <PanelFrame owner="panel.review_copy" status-key="panel.review_copy.status" :show-status="false">
    <ol ref="list" class="review-list" data-review-copy @scroll="onReviewScroll('copy')">
      <template v-for="item in reviewModeCopyItems" :key="item.kind === 'pair' ? item.pair.key : `u${item.row.id}`">
        <li
          v-if="item.kind === 'pair'"
          class="review-item"
          :class="{ current: item.pair.key === reviewModeCurrentPairKey }"
          :data-review-pair="item.pair.key"
          :data-review-current="item.pair.key === reviewModeCurrentPairKey ? '' : undefined"
          data-review-copy-item
        >
          <span v-if="item.pair.rowKind === 'alt'" class="review-kind">{{ t('review.row_alt') }}</span>
          <span v-else-if="item.pair.rowKind === 'caption'" class="review-kind">{{ t('review.row_caption') }}</span>
          <span v-if="!item.pair.hasRow" class="review-empty">{{ t('review.no_counterpart') }}</span>
          <span v-else-if="item.pair.copy.length === 0" class="review-empty">{{ t('review.row_empty') }}</span>
          <ReviewSpans v-else :spans="item.pair.copy" />
        </li>
        <li v-else class="review-item" data-review-copy-item data-review-unmatched>
          <span class="review-kind">{{ t('review.unmatched') }}</span>
          <span v-if="item.row.kind === 'alt'" class="review-kind">{{ t('review.row_alt') }}</span>
          <span v-else-if="item.row.kind === 'caption'" class="review-kind">{{ t('review.row_caption') }}</span>
          <span v-if="item.row.target_text === ''" class="review-empty">{{ t('review.row_empty') }}</span>
          <!-- aura-allow-text: DỮ LIỆU (bản của Reviewer). -->
          <template v-else>{{ item.row.target_text }}</template>
        </li>
      </template>
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

.review-item.current {
  border-left: 2px solid var(--color-on-surface);
  padding-left: calc(var(--space-unit) * 1);
}

.review-kind,
.review-empty {
  margin-right: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}
</style>
