<script setup lang="ts">
import type { ReviewDiffSpan } from '../config/alignment'

defineProps<{ spans: ReviewDiffSpan[] }>()
</script>

<template>
  <template v-for="(span, k) in spans" :key="k">
    <del v-if="span.kind === 'delete'" class="rv-del" data-review-del><!-- aura-allow-text: DỮ LIỆU (bản dịch của người dùng). -->{{ span.text }}</del>
    <ins v-else-if="span.kind === 'insert'" class="rv-ins" data-review-ins><!-- aura-allow-text: DỮ LIỆU (bản của Reviewer). -->{{ span.text }}</ins>
    <template v-else><!-- aura-allow-text: DỮ LIỆU (bản dịch). -->{{ span.text }}</template>
  </template>
</template>

<style scoped>
.rv-ins {
  background: var(--color-diff-add-bg);
  color: var(--color-diff-add-ink);
  text-decoration: underline;
}

.rv-del {
  background: var(--color-diff-del-bg);
  color: var(--color-diff-del-ink);
  text-decoration: line-through;
}
</style>
