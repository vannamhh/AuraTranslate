<script setup lang="ts">
import { computed } from 'vue'
import { dispatch } from './commands'
import { t, tError } from './i18n'
import { aiUsageLine, aiUsageSummaryLine } from './aiUsageLine'
import { eligibleInlineStrips } from './inlineStripEligibility'
import { topmostStrip } from './panels/inlineStripPriority'
import {
  proofreadError,
  proofreadFindingCount,
  proofreadNotScanned,
  proofreadProgress,
  proofreadRunSegmentIds,
  proofreadStateValue,
  proofreadUnlocated,
  proofreadUsageSummary,
} from './proofreadState'

const isVisible = computed(() => topmostStrip(eligibleInlineStrips.value) === 'proofreader')

const isSingle = computed(() => proofreadRunSegmentIds.value.length === 1)

const summary = computed<string | null>(() => {
  switch (proofreadStateValue.value) {
    case 'scanning': {
      if (isSingle.value) return t('proofread.strip.scanning')
      const { done, total } = proofreadProgress.value
      return t('proofread.strip.scanning_progress', { done_count: String(done), total_count: String(total) })
    }
    case 'cancelled':
      return t('proofread.strip.cancelled')
    case 'not_configured':
      return t('panel.ai_translation.status')
    default:
      return null
  }
})

const findingsText = computed<string | null>(() => {
  const state = proofreadStateValue.value
  if (state !== 'done' && state !== 'cancelled' && state !== 'error') return null
  if (state === 'done' && proofreadProgress.value.total === 0) return t('proofread.strip.nothing_to_scan')
  const count = proofreadFindingCount.value
  if (count === 0) {
    if (state !== 'done') return null
    return isSingle.value
      ? t('proofread.strip.clean')
      : t('proofread.strip.clean_many', { sentence_count: String(proofreadProgress.value.done) })
  }
  return isSingle.value
    ? t('proofread.strip.found', { finding_count: String(count) })
    : t('proofread.strip.found_many', {
        finding_count: String(count),
        sentence_count: String(proofreadProgress.value.done),
      })
})

const unlocatedText = computed<string | null>(() =>
  proofreadStateValue.value !== 'scanning' && proofreadUnlocated.value > 0
    ? t('proofread.strip.unlocated', { unlocated_count: String(proofreadUnlocated.value) })
    : null,
)

const notScannedText = computed<string | null>(() =>
  proofreadStateValue.value !== 'scanning' && proofreadNotScanned.value > 0
    ? t('proofread.strip.not_scanned', { not_scanned_count: String(proofreadNotScanned.value) })
    : null,
)

const usageText = computed<string | null>(() => {
  const state = proofreadStateValue.value
  if (state !== 'done' && state !== 'cancelled' && state !== 'error') return null
  const summaryValue = proofreadUsageSummary.value
  if (summaryValue.sentenceCount === 0) return null
  const line =
    summaryValue.reportedCount === 0
      ? aiUsageLine(null)
      : aiUsageSummaryLine(summaryValue, {
          total: 'proofread.usage_total',
          totalWithCost: 'proofread.usage_total_with_cost',
          partial: 'proofread.usage_partial',
          partialWithCost: 'proofread.usage_partial_with_cost',
        })
  return line === null ? null : t(line.key, line.params)
})
</script>

<template>
  <section v-if="isVisible" class="proofreader-strip" :aria-label="t('proofread.strip.title')">
    <header class="pfs-head">
      <span class="pfs-title">{{ t('proofread.strip.title') }}</span>
      <button
        v-if="proofreadStateValue === 'scanning'"
        type="button"
        class="pfs-btn"
        @click="dispatch('ai.proofread.cancel')"
      >
        {{ t('proofread.strip.cancel') }}
      </button>
    </header>

    <p v-if="proofreadStateValue === 'error' && proofreadError !== null" class="pfs-status pfs-error" role="alert">
      <!-- aura-allow-text: result of tError(). -->
      {{ tError(proofreadError) }}
    </p>
    <p v-else-if="summary !== null" class="pfs-status" role="status">
      <!-- aura-allow-text: result of t() computed in the script. -->
      {{ summary }}
    </p>
    <p v-if="findingsText !== null" class="pfs-status" role="status">
      <!-- aura-allow-text: result of t() computed in the script. -->
      {{ findingsText }}
    </p>
    <p v-if="notScannedText !== null" class="pfs-status" role="status">
      <!-- aura-allow-text: result of t() computed in the script. -->
      {{ notScannedText }}
    </p>
    <p v-if="unlocatedText !== null" class="pfs-status" role="status">
      <!-- aura-allow-text: result of t() computed in the script. -->
      {{ unlocatedText }}
    </p>
    <p v-if="usageText !== null" class="pfs-status" role="status">
      <!-- aura-allow-text: result of t() computed in the script. -->
      {{ usageText }}
    </p>
  </section>
</template>

<style scoped>
.proofreader-strip {
  display: flex;
  flex-direction: column;
  flex: none;
  gap: calc(var(--space-unit) * 1);
  padding: var(--space-panel-inline);
  border-top: 1px solid var(--color-outline);
  border-left: 3px solid var(--color-error);
  background: var(--color-surface);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.pfs-head {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 4);
}

.pfs-title {
  font-weight: var(--weight-ui-md-strong);
}

.pfs-status {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pfs-error {
  color: var(--color-error);
}

.pfs-btn {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  cursor: pointer;
}
</style>
