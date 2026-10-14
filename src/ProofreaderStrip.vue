<script setup lang="ts">
import { computed } from 'vue'
import { dispatch } from './commands'
import { t, tError } from './i18n'
import { aiUsageLine, aiUsageSummaryLine } from './aiUsageLine'
import { eligibleInlineStrips, proofreaderStripPlace } from './inlineStripEligibility'
import type { ProofreaderStripPlace } from './inlineStripEligibility'
import { topmostStrip } from './panels/inlineStripPriority'
import type { ProofreadFindingKind } from './config/proofread'
import {
  proofreadActionError,
  proofreadAmbiguous,
  proofreadBusy,
  proofreadCurrent,
  proofreadError,
  proofreadFiltered,
  proofreadFindingCount,
  proofreadNotScanned,
  proofreadProgress,
  proofreadRevertible,
  proofreadRunSegmentIds,
  proofreadStateValue,
  proofreadUnlocated,
  proofreadUsageSummary,
} from './proofreadState'

const props = defineProps<{ place?: ProofreaderStripPlace }>()

const isVisible = computed(() => {
  const top = topmostStrip(eligibleInlineStrips.value)
  if (top !== 'proofreader' && top !== 'proofreader_summary') return false
  return props.place === undefined || proofreaderStripPlace.value === props.place
})

const isJob = computed(() => topmostStrip(eligibleInlineStrips.value) === 'proofreader')

const KIND_LABELS: Readonly<Record<ProofreadFindingKind, () => string>> = {
  spelling: () => t('proofread.kind.spelling'),
  grammar: () => t('proofread.kind.grammar'),
  mistranslation: () => t('proofread.kind.mistranslation'),
  too_loose: () => t('proofread.kind.too_loose'),
  obscure: () => t('proofread.kind.obscure'),
}

const card = computed(() => {
  const current = isJob.value ? proofreadCurrent.value : null
  if (current === null) return null
  const { finding } = current
  return {
    kind: KIND_LABELS[finding.kind](),
    phrase: current.scannedText.slice(finding.start, finding.end),
    explanation: finding.explanation,
    suggestion:
      finding.suggestion === ''
        ? t('proofread.strip.suggestion_delete')
        : t('proofread.strip.suggestion', { suggestion_text: finding.suggestion }),
    position: t('proofread.strip.position', {
      finding_index: String(current.index + 1),
      finding_count: String(current.count),
    }),
    many: current.count > 1,
  }
})

const actionErrorText = computed<string | null>(() => {
  const err = isJob.value ? proofreadActionError.value : null
  return err === null ? null : tError(err)
})

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
  if (card.value !== null) return null
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

const ambiguousText = computed<string | null>(() =>
  proofreadStateValue.value !== 'scanning' && proofreadAmbiguous.value > 0
    ? t('proofread.strip.ambiguous', { ambiguous_count: String(proofreadAmbiguous.value) })
    : null,
)

const notScannedText = computed<string | null>(() =>
  proofreadStateValue.value !== 'scanning' && proofreadNotScanned.value > 0
    ? t('proofread.strip.not_scanned', { not_scanned_count: String(proofreadNotScanned.value) })
    : null,
)

const filteredText = computed<string | null>(() =>
  proofreadStateValue.value !== 'scanning' && proofreadFiltered.value > 0
    ? t('proofread.strip.filtered', { filtered_count: String(proofreadFiltered.value) })
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

    <div v-if="card !== null" class="pfs-card">
      <p class="pfs-status pfs-card-head">
        <!-- aura-allow-text: result of t() computed in the script. -->
        <span class="pfs-kind">{{ card.kind }}</span>
        <!-- aura-allow-text: model output, shown as text only. -->
        <span class="pfs-phrase">{{ card.phrase }}</span>
        <!-- aura-allow-text: result of t() computed in the script. -->
        <span class="pfs-position">{{ card.position }}</span>
      </p>
      <!-- aura-allow-text: model output, shown as text only. -->
      <p class="pfs-status">{{ card.explanation }}</p>
      <!-- aura-allow-text: result of t() computed in the script. -->
      <p class="pfs-status pfs-suggestion">{{ card.suggestion }}</p>
      <div class="pfs-actions">
        <button type="button" class="pfs-btn" :disabled="proofreadBusy" @click="dispatch('ai.proofread.accept')">
          {{ t('command.ai.proofread.accept') }}
        </button>
        <button type="button" class="pfs-btn" :disabled="proofreadBusy" @click="dispatch('ai.proofread.dismiss')">
          {{ t('command.ai.proofread.dismiss') }}
        </button>
        <button type="button" class="pfs-btn" :disabled="proofreadBusy" @click="dispatch('ai.proofread.ignore')">
          {{ t('command.ai.proofread.ignore') }}
        </button>
        <template v-if="card.many">
          <button type="button" class="pfs-btn" @click="dispatch('ai.proofread.prev')">
            {{ t('command.ai.proofread.prev') }}
          </button>
          <button type="button" class="pfs-btn" @click="dispatch('ai.proofread.next')">
            {{ t('command.ai.proofread.next') }}
          </button>
        </template>
      </div>
    </div>
    <div v-if="isJob && proofreadRevertible" class="pfs-actions">
      <span class="pfs-status">{{ t('proofread.strip.reverted_hint') }}</span>
      <button type="button" class="pfs-btn" :disabled="proofreadBusy" @click="dispatch('ai.proofread.revert')">
        {{ t('command.ai.proofread.revert') }}
      </button>
    </div>
    <p v-if="actionErrorText !== null" class="pfs-status pfs-error" role="alert">
      <!-- aura-allow-text: result of tError(). -->
      {{ actionErrorText }}
    </p>
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
    <p v-if="ambiguousText !== null" class="pfs-status" role="status">
      <!-- aura-allow-text: result of t() computed in the script. -->
      {{ ambiguousText }}
    </p>
    <p v-if="filteredText !== null" class="pfs-status" role="status">
      <!-- aura-allow-text: result of t() computed in the script. -->
      {{ filteredText }}
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

.pfs-card {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
}

.pfs-card-head {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 3);
}

.pfs-kind {
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-error);
}

.pfs-phrase {
  color: var(--color-on-surface);
}

.pfs-position {
  margin-left: auto;
}

.pfs-suggestion {
  color: var(--color-on-surface);
}

.pfs-actions {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
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
