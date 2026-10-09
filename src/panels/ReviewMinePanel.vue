<script setup lang="ts">
import { onBeforeUnmount, onMounted, useTemplateRef } from 'vue'
import { t } from '../i18n'
import type { DockviewPanelProps } from '../layout/panelProps'
import { reviewModeCurrentPairKey, reviewModeMineItems } from '../reviewModeState'
import { onReviewScroll, registerReviewScroller } from '../reviewScrollSync'
import PanelFrame from './PanelFrame.vue'
import ReviewSpans from './ReviewSpans.vue'

defineProps<DockviewPanelProps>()

const list = useTemplateRef<HTMLElement>('list')

onMounted(() => registerReviewScroller('mine', list.value))
onBeforeUnmount(() => registerReviewScroller('mine', null))
</script>

<template>
  <PanelFrame owner="panel.review_mine" status-key="panel.review_mine.status" :show-status="false">
    <ol ref="list" class="review-list" data-review-mine @scroll="onReviewScroll('mine')">
      <template v-for="item in reviewModeMineItems" :key="item.kind === 'pair' ? item.pair.key : `u${item.segment.id}`">
        <li
          v-if="item.kind === 'pair'"
          class="review-item"
          :class="{ current: item.pair.key === reviewModeCurrentPairKey }"
          :data-review-pair="item.pair.key"
          :data-review-current="item.pair.key === reviewModeCurrentPairKey ? '' : undefined"
          data-review-mine-item
        >
          <span v-if="!item.pair.hasSegment" class="review-empty">{{ t('review.no_counterpart') }}</span>
          <span v-else-if="item.pair.mine.length === 0" class="review-empty">{{ t('review.untranslated') }}</span>
          <ReviewSpans v-else :spans="item.pair.mine" />
        </li>
        <li v-else class="review-item" data-review-mine-item data-review-unmatched>
          <span class="review-kind">{{ t('review.unmatched') }}</span>
          <span v-if="item.segment.target_text === ''" class="review-empty">{{ t('review.untranslated') }}</span>
          <!-- aura-allow-text: DỮ LIỆU (bản dịch của người dùng). -->
          <template v-else>{{ item.segment.target_text }}</template>
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
