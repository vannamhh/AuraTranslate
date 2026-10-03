<script setup lang="ts">
// Fuzzy TM strip in the shared inline slot above the status bar. It shows only when
// `topmostStrip` says `tm_fuzzy` wins, so a Glossary strip always takes the slot first.
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { dispatch } from './commands'
import { t, tError } from './i18n'
import { eligibleInlineStrips } from './inlineStripEligibility'
import { editorCaretSegmentId, editorSegments } from './panels/editorPanelState'
import { historyTimeLabel } from './panels/segmentHistoryTime'
import { topmostStrip } from './panels/inlineStripPriority'
import {
  aimTmFuzzyRow,
  syncTmFuzzyStrip,
  tmFuzzyAcceptError,
  tmFuzzyAccepting,
  tmFuzzyAimedIndex,
  tmFuzzyExactShown,
  tmFuzzyFocusRequest,
  tmFuzzyMatchesShown,
  tmFuzzyPendingAccept,
  tmFuzzyRowCount,
  tmFuzzyScanError,
  tmFuzzySegmentId,
} from './tmFuzzyStripState'

watch(editorCaretSegmentId, (segmentId) => { syncTmFuzzyStrip(segmentId) }, { immediate: true })

const isVisible = computed(() => topmostStrip(eligibleInlineStrips.value) === 'tm_fuzzy')

const root = useTemplateRef<HTMLElement>('root')
watch(tmFuzzyFocusRequest, () => {
  void nextTick(() => {
    root.value?.focus()
  })
})

const DIGIT_ROWS: Readonly<Record<string, number>> = {
  '1': 0, '2': 1, '3': 2, '4': 3, '5': 4, '6': 5, '7': 6, '8': 7, '9': 8,
}

const isExactList = computed(() => tmFuzzyExactShown.value.length > 0)

const currentTarget = computed(() => {
  const id = tmFuzzySegmentId.value
  return editorSegments.value.find((s) => s.id === id)?.target_text ?? null
})

const stripTitle = computed(() => (isExactList.value ? t('tm.exact.title') : t('tm.fuzzy.title')))
const stripHint = computed(() => (isExactList.value ? t('tm.exact.hint') : t('tm.fuzzy.hint')))
const overwriteQuestion = computed(() =>
  tmFuzzyPendingAccept.value?.kind === 'exact' ? t('tm.exact.overwrite_question') : t('tm.fuzzy.overwrite_question'),
)

const exactRows = computed(() => {
  const nowMs = Date.now()
  return tmFuzzyExactShown.value.map((row) => {
    const { key, params } = historyTimeLabel(row.created_at, nowMs)
    return { row, date: t(key, params), inUse: row.target_text === currentTarget.value }
  })
})

/** Keys act only when the strip itself holds focus, so its buttons keep their own Enter. */
function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.preventDefault()
    dispatch('tm.fuzzy.hide')
    return
  }
  if (event.target !== event.currentTarget) return
  if (event.ctrlKey || event.metaKey || event.altKey) return
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    dispatch('tm.fuzzy.next')
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    dispatch('tm.fuzzy.prev')
  } else if (event.key === 'Enter') {
    event.preventDefault()
    if (tmFuzzyPendingAccept.value === null) dispatch('tm.fuzzy.accept')
    else dispatch('tm.fuzzy.confirm_overwrite')
  } else if (event.key in DIGIT_ROWS && tmFuzzyPendingAccept.value === null) {
    const row = DIGIT_ROWS[event.key] ?? 0
    event.preventDefault()
    if (row >= tmFuzzyRowCount()) return
    aimTmFuzzyRow(row)
    dispatch('tm.fuzzy.accept')
  }
}

const acceptErrorText = computed(() => {
  const err = tmFuzzyAcceptError.value
  if (err === null) return null
  return err.code === 'tm.pair_not_found' ? t('tm.fuzzy.pair_gone') : tError(err)
})
</script>

<template>
  <section
    v-if="isVisible"
    ref="root"
    class="tm-fuzzy-strip"
    tabindex="-1"
    :aria-label="t('tm.fuzzy.title')"
    @keydown="onKeydown"
  >
    <header class="tmf-head">
      <!-- aura-allow-text: result of t() computed in the script. -->
      <span class="tmf-title">{{ stripTitle }}</span>
      <!-- aura-allow-text: result of t() computed in the script. -->
      <span class="tmf-hint">{{ stripHint }}</span>
    </header>

    <p v-if="tmFuzzyScanError !== null" class="tmf-status tmf-error" role="alert">
      {{ tError(tmFuzzyScanError) }}
    </p>
    <p v-if="acceptErrorText !== null" class="tmf-status tmf-error" role="alert">
      <!-- aura-allow-text: result of t()/tError() computed in the script. -->
      {{ acceptErrorText }}
    </p>

    <div v-if="tmFuzzyPendingAccept !== null" class="tmf-overwrite" role="alert">
      <!-- aura-allow-text: result of t() computed in the script. -->
      <p class="tmf-status">{{ overwriteQuestion }}</p>
      <!-- aura-allow-text: the user's own draft text, shown so the overwrite is an informed choice. -->
      <p class="tmf-draft">{{ tmFuzzyPendingAccept.draft }}</p>
      <div class="tmf-actions">
        <button type="button" class="tmf-btn tmf-btn-primary" @click="dispatch('tm.fuzzy.confirm_overwrite')">
          {{ t('tm.fuzzy.overwrite_confirm') }}
        </button>
        <button type="button" class="tmf-btn" @click="dispatch('tm.fuzzy.hide')">
          {{ t('tm.fuzzy.overwrite_keep') }}
        </button>
      </div>
    </div>

    <ol v-if="isExactList" class="tmf-list tmf-exact-list">
      <li
        v-for="(item, i) in exactRows"
        :key="`${item.row.tier}:${item.row.unit_id}`"
        class="tmf-row tmf-exact-row"
        :class="{ 'tmf-row-aimed': i === tmFuzzyAimedIndex, 'tmf-row-in-use': item.inUse }"
        :aria-current="i === tmFuzzyAimedIndex ? 'true' : undefined"
        @mouseenter="aimTmFuzzyRow(i)"
      >
        <span class="tmf-pct tmf-date">
          <!-- aura-allow-text: result of t(). -->
          {{ item.date }}
        </span>
        <div class="tmf-body">
          <!-- aura-allow-text: TM target text, user data. -->
          <p class="tmf-target">{{ item.row.target_text }}</p>
          <p v-if="item.inUse" class="tmf-in-use">{{ t('tm.exact.in_use') }}</p>
        </div>
        <div class="tmf-meta">
          <span class="tmf-side">
            <!-- aura-allow-text: result of t(). -->
            {{ item.row.side === 'mine' ? t('tm.fuzzy.side_mine') : t('tm.fuzzy.side_others') }}
          </span>
          <span class="tmf-tier">
            <!-- aura-allow-text: result of t(). -->
            {{ item.row.tier === 'work' ? t('tm.fuzzy.tier_work') : t('tm.fuzzy.tier_global') }}
          </span>
          <button
            type="button"
            class="tmf-btn"
            :disabled="tmFuzzyAccepting || tmFuzzyPendingAccept !== null"
            @focus="aimTmFuzzyRow(i)"
            @mousedown="aimTmFuzzyRow(i)"
            @click="dispatch('tm.fuzzy.accept')"
          >
            {{ t('tm.fuzzy.accept', { n: String(i + 1) }) }}
          </button>
        </div>
      </li>
    </ol>

    <ol v-else class="tmf-list">
      <li
        v-for="(match, i) in tmFuzzyMatchesShown"
        :key="`${match.tier}:${match.unit_id}`"
        class="tmf-row"
        :class="{ 'tmf-row-aimed': i === tmFuzzyAimedIndex }"
        :aria-current="i === tmFuzzyAimedIndex ? 'true' : undefined"
        @mouseenter="aimTmFuzzyRow(i)"
      >
        <span class="tmf-pct">{{ t('tm.fuzzy.percent', { percent: String(match.percent) }) }}</span>
        <div class="tmf-body">
          <p class="tmf-source">
            <template v-for="(span, k) in match.diff" :key="k">
              <del v-if="span.kind === 'delete'" class="tmf-del"><!-- aura-allow-text: TM source text, user data. -->{{ span.text }}</del>
              <ins v-else-if="span.kind === 'insert'" class="tmf-ins"><!-- aura-allow-text: TM source text, user data. -->{{ span.text }}</ins>
              <span v-else><!-- aura-allow-text: TM source text, user data. -->{{ span.text }}</span>
            </template>
          </p>
          <!-- aura-allow-text: TM target text, user data. -->
          <p class="tmf-target">{{ match.target_text }}</p>
        </div>
        <div class="tmf-meta">
          <span class="tmf-side">
            <!-- aura-allow-text: result of t(). -->
            {{ match.side === 'mine' ? t('tm.fuzzy.side_mine') : t('tm.fuzzy.side_others') }}
          </span>
          <span class="tmf-tier">
            <!-- aura-allow-text: result of t(). -->
            {{ match.tier === 'work' ? t('tm.fuzzy.tier_work') : t('tm.fuzzy.tier_global') }}
          </span>
          <button
            type="button"
            class="tmf-btn"
            :disabled="tmFuzzyAccepting || tmFuzzyPendingAccept !== null"
            @focus="aimTmFuzzyRow(i)"
            @mousedown="aimTmFuzzyRow(i)"
            @click="dispatch('tm.fuzzy.accept')"
          >
            {{ t('tm.fuzzy.accept', { n: String(i + 1) }) }}
          </button>
        </div>
      </li>
    </ol>
  </section>
</template>

<style scoped>
.tm-fuzzy-strip {
  display: flex;
  flex-direction: column;
  flex: none;
  gap: calc(var(--space-unit) * 2);
  padding: var(--space-panel-inline);
  border-top: 1px solid var(--color-outline);
  border-left: 3px solid var(--color-tm-rule);
  background: var(--color-surface-tm);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.tmf-head {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: calc(var(--space-unit) * 4);
}

.tmf-title {
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-tm-text);
}

.tmf-hint,
.tmf-side,
.tmf-tier,
.tmf-status {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.tmf-status {
  margin: 0;
}

.tmf-error {
  color: var(--color-error);
}

.tmf-list {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin: 0;
  padding: 0;
  list-style: none;
}

.tmf-row {
  display: grid;
  grid-template-columns: 4em 1fr auto;
  gap: calc(var(--space-unit) * 3);
  align-items: start;
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid transparent;
}

.tmf-row-aimed {
  border-color: var(--color-tm-rule);
}

.tmf-pct {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-tm-text);
  text-align: right;
}

.tmf-body p {
  margin: 0;
}

.tmf-target {
  color: var(--color-on-surface-variant);
}

.tmf-date {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  text-align: left;
}

.tmf-exact-row {
  grid-template-columns: 8em 1fr auto;
}

.tmf-in-use {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-tm-text);
}

.tmf-ins {
  background: var(--color-diff-add-bg);
  color: var(--color-diff-add-ink);
  text-decoration: none;
}

.tmf-del {
  background: var(--color-diff-del-bg);
  color: var(--color-diff-del-ink);
}

.tmf-meta {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: calc(var(--space-unit) * 1);
}

.tmf-draft {
  margin: 0;
  color: var(--color-on-surface-variant);
}

.tmf-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}

.tmf-btn {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  cursor: pointer;
}

.tmf-btn:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.tmf-btn-primary {
  border-color: var(--color-primary);
}
</style>
