<script setup lang="ts">
import { computed, watch } from 'vue'
import { t, tError } from './i18n'
import {
  ensureProofreadDecisionsLoaded,
  proofreadDecisionError,
  proofreadDecisionRatio,
} from './proofreadDecisionState'

const props = defineProps<{ active: boolean }>()

watch(
  () => [props.active, proofreadDecisionRatio.value === null] as const,
  ([active, unread]) => {
    if (active && unread) ensureProofreadDecisionsLoaded()
  },
  { immediate: true },
)

const none = computed(() => proofreadDecisionRatio.value?.kind === 'none')

const parts = computed(() => {
  const ratio = proofreadDecisionRatio.value
  if (ratio === null || ratio.kind !== 'counts') return []
  return [
    {
      id: 'accepted',
      width: ratio.acceptedPercent,
      text: t('proofread.ratio.accepted', { decision_count: String(ratio.accepted), percent: String(ratio.acceptedPercent) }),
    },
    {
      id: 'not-error',
      width: ratio.notErrorPercent,
      text: t('proofread.ratio.not_error', { decision_count: String(ratio.notError), percent: String(ratio.notErrorPercent) }),
    },
    {
      id: 'skipped',
      width: ratio.skippedPercent,
      text: t('proofread.ratio.skipped', { decision_count: String(ratio.skipped), percent: String(ratio.skippedPercent) }),
    },
  ]
})

const errorText = computed<string | null>(() =>
  proofreadDecisionError.value === null ? null : tError(proofreadDecisionError.value),
)
</script>

<template>
  <div v-if="errorText !== null" class="pfr pfr-error" role="alert" data-proofread-ratio="error">
    <!-- aura-allow-text: result of tError(). -->
    {{ errorText }}
  </div>
  <div v-else-if="none" class="pfr" role="status" data-proofread-ratio="none">
    {{ t('proofread.ratio.none') }}
  </div>
  <div v-else-if="parts.length > 0" class="pfr" role="status" data-proofread-ratio="counts" :aria-label="t('proofread.ratio.label')">
    <div class="pfr-track" aria-hidden="true">
      <i v-for="part in parts" :key="part.id" :class="`pfr-${part.id}`" :style="{ width: `${String(part.width)}%` }"></i>
    </div>
    <div class="pfr-legend">
      <!-- aura-allow-text: result of t() computed in the script. -->
      <span v-for="part in parts" :key="part.id">{{ part.text }}</span>
    </div>
  </div>
</template>

<style scoped>
.pfr {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pfr-error {
  color: var(--color-error);
}

.pfr-track {
  display: flex;
  height: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
}

.pfr-track > i {
  display: block;
  height: 100%;
}

.pfr-accepted {
  background: var(--color-confirmed);
}

.pfr-not-error {
  background: var(--color-error);
}

.pfr-skipped {
  background: var(--color-ornament);
}

.pfr-legend {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 3);
}
</style>
