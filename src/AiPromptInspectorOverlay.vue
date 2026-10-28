<script setup lang="ts">
// Prompt inspector. It only READS the record Rust holds; `aiPromptAssemble` must never be imported here.
// Three-valued states stay distinct: no record vs empty prompt, Glossary not_asked vs asked(0),
// TM not_asked vs searched(0), unsent vs sent-without-token-count.
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import { editorActiveSegmentId } from './panels/editorPanelState'
import SelectMenu from './SelectMenu.vue'
import { promptSets, selectedPromptSetName, setSelectedPromptSetName, promptSetChoiceError } from './promptSetState'
import { confirmStripEntryId, confirmStripTier } from './glossaryConfirmStripState'
import {
  aiPromptCallNote,
  aiPromptCharsByKind,
  aiPromptHighlightParts,
  aiPromptAssembleBusy,
  aiPromptAssembleError,
  aiPromptCopyState,
  aiPromptDynamicRatio,
  aiPromptGlossaryDelta,
  aiPromptInspectorIsOpen,
  aiPromptInspectorTab,
  aiPromptReadError,
  aiPromptRecord,
  aiPromptRecordIsStale,
  aiPromptResultFor,
  aiPromptSystemText,
  aiPromptTierBlocks,
  aiPromptTokenLine,
  formatPercentParam,
  formatSentAtClock,
  glossaryInjectionSummary,
} from './aiPromptInspectorState'
import type { AiPromptTierBlockKind } from './aiPromptInspectorState'
import {
  aiTranslateAccumulatedText,
  aiTranslateError,
  aiTranslateRunSegmentId,
  aiTranslateStartStamp,
  aiTranslateStateValue,
  aiTranslateUsage,
} from './aiTranslateState'
import {
  aiTranslateBatchRows,
  aiTranslateBatchStartStamp,
  aiTranslateBatchStateValue,
  aiTranslateBatchTextForSegment,
} from './aiTranslateBatchState'
import { aiUsageLine, formatUsdParam } from './aiUsageLine'

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

let returnFocusTo: HTMLElement | null = null

watch(aiPromptInspectorIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-ai-prompt-inspector-open]')
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-ai-prompt-inspector-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[ai-prompt-inspector] focus-return target is gone; focus falls back to body.')
})

function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), summary:not([aria-disabled="true"]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
}

function trapTab(event: KeyboardEvent): void {
  const root = panel.value
  if (root === null) return

  event.preventDefault()
  const stops = focusableWithin(root)
  if (stops.length === 0) {
    root.focus()
    return
  }

  const active = document.activeElement
  const index = active instanceof HTMLElement ? stops.indexOf(active) : -1
  const step = event.shiftKey ? -1 : 1
  const next = index === -1 ? (event.shiftKey ? stops.length - 1 : 0) : index + step
  stops[(next + stops.length) % stops.length].focus()
}

function onEscape(): void {
  dispatch('ai.prompt_inspector.close')
}

/** `glossary.confirm.focus` reaches only the entry the confirm strip asks now, so the button shows for that entry alone. */
function canConfirmNow(term: { id: number; tier: 'global' | 'work' }): boolean {
  return !isStale.value && confirmStripEntryId.value === term.id && confirmStripTier.value === term.tier
}

function tierLabel(tier: 'global' | 'work'): string {
  return t(tier === 'work' ? 'ai.prompt_inspector.tier_work' : 'ai.prompt_inspector.tier_global')
}

const emptyKey = computed<string | null>(() => {
  if (aiPromptAssembleError.value !== null || aiPromptReadError.value !== null) return null
  return aiPromptAssembleBusy.value || editorActiveSegmentId.value !== null
    ? 'ai.prompt_inspector.assembling'
    : 'ai.prompt_inspector.no_segment_hint'
})

const isStale = computed<boolean>(() => aiPromptRecordIsStale(aiPromptRecord.value, editorActiveSegmentId.value))

const isOtherSet = computed<boolean>(
  () =>
    aiPromptRecord.value !== null &&
    selectedPromptSetName.value !== null &&
    aiPromptRecord.value.prompt_set_name !== selectedPromptSetName.value,
)

const promptSetOptions = computed(() => [
  { value: '', label: t('ai.prompt_inspector.set_placeholder') },
  ...promptSets.value.map((set) => ({ value: set.name, label: set.name })),
])

function onPromptSetChange(value: string): void {
  setSelectedPromptSetName(value === '' ? null : value)
  if (value !== '' && aiPromptRecord.value === null && editorActiveSegmentId.value !== null) {
    dispatch('ai.prompt.assemble')
  }
}

const glossarySummary = computed(() => glossaryInjectionSummary(aiPromptRecord.value))

const glossarySummaryText = computed<string>(() => {
  const summary = glossarySummary.value
  if (summary.kind === 'no_record') return ''
  const stale = isStale.value ? ` ${t('ai.prompt_inspector.summary_other_segment')}` : ''
  if (summary.kind === 'not_asked') {
    return (
      (summary.tmCount === null
        ? t('ai.prompt.summary_not_asked')
        : t('ai.prompt.summary_not_asked_tm', { tm_count: String(summary.tmCount) })) + stale
    )
  }
  const delta = aiPromptGlossaryDelta.value
  if (delta !== null) {
    const direction = delta > 0 ? 'up' : 'down'
    const params = { count: String(summary.count), delta: String(Math.abs(delta)) }
    return (
      (summary.tmCount === null
        ? t(`ai.prompt.summary_asked_${direction}`, params)
        : t(`ai.prompt.summary_asked_tm_${direction}`, { ...params, tm_count: String(summary.tmCount) })) + stale
    )
  }
  return (
    (summary.tmCount === null
      ? t('ai.prompt.summary_asked', { count: String(summary.count) })
      : t('ai.prompt.summary_asked_tm', { count: String(summary.count), tm_count: String(summary.tmCount) })) + stale
  )
})

const glossaryAsked = computed(() => {
  const rec = aiPromptRecord.value
  if (rec === null) return null
  const status = rec.ledger.glossary
  return status.kind === 'asked' ? status : null
})

const tmSearched = computed(() => {
  const rec = aiPromptRecord.value
  if (rec === null) return null
  const status = rec.ledger.tm
  return status.kind === 'searched' ? status : null
})

const blocks = computed(() => aiPromptTierBlocks(aiPromptRecord.value))
const callNote = computed(() => aiPromptCallNote(aiPromptRecord.value))
const highlightBlockNumber = computed<number | null>(() =>
  callNote.value.kind === 'block' ? callNote.value.blockNumber : null,
)
const systemText = computed(() => aiPromptSystemText(aiPromptRecord.value))
const systemChars = computed(() => Array.from(systemText.value ?? '').length)
const charsByKind = computed(() => aiPromptCharsByKind(blocks.value))
const totalChars = computed(() => {
  const c = charsByKind.value
  return c.authored + c.glossary + c.tm + c.source_segment
})
const dynamicRatio = computed(() => aiPromptDynamicRatio(blocks.value))
const tokenLine = computed(() => aiPromptTokenLine(aiPromptRecord.value))

const BAR_KINDS: readonly AiPromptTierBlockKind[] = ['authored', 'glossary', 'tm', 'source_segment']

function blockLabel(kind: AiPromptTierBlockKind): string {
  return t(`ai.prompt_inspector.block_${kind}`)
}

function barWidth(kind: AiPromptTierBlockKind): string {
  if (totalChars.value === 0) return '0%'
  return `${(charsByKind.value[kind] / totalChars.value) * 100}%`
}

const sentMeta = computed(() => {
  const rec = aiPromptRecord.value
  if (rec === null) return ''
  if (rec.sent_at === null) return t('ai.prompt_inspector.meta_unsent')
  return t('ai.prompt_inspector.meta_sent', {
    time: formatSentAtClock(rec.sent_at),
    model: rec.sent_model ?? '',
  })
})

const copyLabelKey = computed(() => {
  if (aiPromptCopyState.value === 'copied') return 'ai.prompt_inspector.copy_done'
  if (aiPromptCopyState.value === 'failed') return 'ai.prompt_inspector.copy_failed'
  return 'command.ai.prompt_inspector.copy'
})

const canResend = computed<boolean>(
  () =>
    aiPromptRecord.value !== null &&
    aiTranslateStateValue.value !== 'generating' &&
    aiTranslateBatchStateValue.value !== 'generating',
)

const result = computed(() => {
  const rec = aiPromptRecord.value
  if (rec === null) return null
  return aiPromptResultFor({
    recordSegmentId: rec.segment_id,
    singleSegmentId: aiTranslateRunSegmentId.value,
    singleState: aiTranslateStateValue.value,
    singleText: aiTranslateAccumulatedText.value,
    singleStamp: aiTranslateStartStamp.value,
    batchText: aiTranslateBatchTextForSegment(aiTranslateBatchRows.value, rec.segment_id),
    batchStamp: aiTranslateBatchStartStamp.value,
  })
})

const resultCancelled = computed<boolean>(() => {
  const r = result.value
  return r !== null && r.kind === 'text' && r.from === 'single' && aiTranslateStateValue.value === 'cancelled'
})

const resultUsage = computed(() => {
  const r = result.value
  if (r === null || r.kind !== 'text' || r.from !== 'single' || aiTranslateStateValue.value !== 'done') return null
  return aiUsageLine(aiTranslateUsage.value)
})
</script>

<template>
  <div
    v-if="aiPromptInspectorIsOpen"
    class="aip-scrim"
    @keydown.esc="onEscape"
    @keydown.tab="trapTab($event)"
  >
    <section
      ref="panel"
      class="aip-panel"
      tabindex="-1"
      role="dialog"
      aria-modal="true"
      aria-labelledby="aip-title"
    >
      <header class="aip-head">
        <h2 id="aip-title" class="aip-title">{{ t('ai.prompt_inspector.title') }}</h2>
        <button type="button" class="aip-close ui-btn ui-btn--quiet" @click="dispatch('ai.prompt_inspector.close')">
          {{ t('command.ai.prompt_inspector.close') }}
        </button>
      </header>

      <div class="aip-setbar" data-aip-set-bar>
        <p v-if="promptSets.length === 0" class="aip-note" data-aip-set-empty>{{ t('panel.ai_translation.prompt_set_empty') }}</p>
        <div v-else class="aip-set-field">
          <span class="aip-blabel">{{ t('ai.prompt_inspector.set_label') }}</span>
          <SelectMenu
            class="aip-set-select"
            variant="field"
            :model-value="selectedPromptSetName ?? ''"
            :options="promptSetOptions"
            :columns="3"
            :label="t('ai.prompt_inspector.set_label')"
            @change="onPromptSetChange"
          />
        </div>
        <button type="button" class="aip-close ui-btn ui-btn--quiet" data-prompt-library-open @click="dispatch('prompt.library.open')">
          {{ t('command.prompt.library.open') }}
        </button>
      </div>
      <p v-if="promptSetChoiceError !== null" class="aip-read-error" role="alert" data-aip-set-choice-error>
        <!-- aura-allow-text: KẾT QUẢ của tError(). -->
        {{ tError(promptSetChoiceError) }}
      </p>
      <!-- aura-allow-text: KẾT QUẢ của t() (ba-giá-trị: not_asked / asked kèm số đếm, có dấu của câu khác). -->
      <p
        v-if="glossarySummary.kind !== 'no_record'"
        class="aip-note"
        :data-ai-prompt-summary-kind="glossarySummary.kind"
        data-aip-summary
      >
        {{ glossarySummaryText }}
      </p>

      <p v-if="aiPromptReadError !== null" class="aip-read-error" role="alert" data-aip-read-error="true">
        <!-- aura-allow-text: KẾT QUẢ của tError(). -->
        {{ tError(aiPromptReadError) }}
      </p>

      <p v-if="aiPromptAssembleError !== null" class="aip-read-error" role="alert" data-aip-assemble-error>
        <!-- aura-allow-text: KẾT QUẢ của tError(). -->
        {{ tError(aiPromptAssembleError) }}
      </p>

      <p v-if="aiPromptRecord === null" class="aip-empty" role="status" data-aip-record-state="none">
        <!-- aura-allow-text: KẾT QUẢ của t() (hai nhãn: chưa lắp / chưa có câu). -->
        {{ emptyKey === null ? '' : t(emptyKey) }}
      </p>

      <div v-else class="aip-body" data-aip-record-state="present">
        <div class="aip-tabs" role="tablist" :aria-label="t('ai.prompt_inspector.tabs_label')">
          <button
            type="button"
            role="tab"
            class="aip-tab ui-tab"
            :aria-selected="aiPromptInspectorTab === 'result'"
            data-aip-tab="result"
            @click="dispatch('ai.prompt_inspector.tab_result')"
          >
            {{ t('command.ai.prompt_inspector.tab_result') }}
          </button>
          <button
            type="button"
            role="tab"
            class="aip-tab ui-tab"
            :aria-selected="aiPromptInspectorTab === 'prompt'"
            data-aip-tab="prompt"
            @click="dispatch('ai.prompt_inspector.tab_prompt')"
          >
            {{ t('command.ai.prompt_inspector.tab_prompt') }}
          </button>
          <p class="aip-meta" data-aip-sent-meta>
            <!-- aura-allow-text: KẾT QUẢ của t() (giờ gửi, tên mô hình). -->
            {{ sentMeta }}
          </p>
        </div>

        <p class="aip-identity">
          <!-- aura-allow-text: KẾT QUẢ của t() (nội suy dữ liệu bản ghi — số câu/Chương/tên bộ/tầng). -->
          {{
            t('ai.prompt_inspector.identity', {
              segment_id: String(aiPromptRecord.segment_id),
              chapter_id: String(aiPromptRecord.chapter_id),
              name: aiPromptRecord.prompt_set_name,
              tier: tierLabel(aiPromptRecord.prompt_set_tier),
            })
          }}
        </p>

        <p v-if="isStale" class="aip-stale" role="status" data-aip-stale="true">
          <!-- aura-allow-text: KẾT QUẢ của t() (nội suy số câu). -->
          {{
            t('ai.prompt_inspector.stale_notice', {
              record_segment_id: String(aiPromptRecord.segment_id),
              focused_segment_id: String(editorActiveSegmentId ?? ''),
            })
          }}
        </p>
        <p v-if="!isStale && isOtherSet" class="aip-other-set" role="status" data-aip-other-set>
          <!-- aura-allow-text: KẾT QUẢ của t() (tên hai bộ). -->
          {{
            t('ai.prompt_inspector.set_mismatch_notice', {
              record_name: aiPromptRecord.prompt_set_name,
              selected_name: selectedPromptSetName ?? '',
            })
          }}
        </p>
        <button
          v-if="isStale || isOtherSet"
          type="button"
          class="aip-close ui-btn ui-btn--quiet"
          data-aip-assemble-selected
          :disabled="aiPromptAssembleBusy"
          @click="dispatch('ai.prompt.assemble')"
        >
          {{ t('ai.prompt_inspector.assemble_selected') }}
        </button>

        <div v-if="aiPromptInspectorTab === 'result'" class="aip-result" data-aip-tab-panel="result">
          <h3 class="aip-sh">
            <!-- aura-allow-text: KẾT QUẢ của t() (số câu). -->
            {{ t('ai.prompt_inspector.result_heading', { segment_id: String(aiPromptRecord.segment_id) }) }}
          </h3>
          <p v-if="result === null || result.kind === 'empty'" class="aip-note" data-aip-result-state="empty">
            {{ t('ai.prompt_inspector.result_empty', { segment_id: String(aiPromptRecord.segment_id) }) }}
          </p>
          <p v-else-if="result.kind === 'not_configured'" class="aip-note" data-aip-result-state="not_configured">
            {{ t('panel.ai_translation.status') }}
          </p>
          <p v-else-if="result.kind === 'error'" class="aip-warn" role="alert" data-aip-result-state="error">
            <!-- aura-allow-text: KẾT QUẢ của tError(). -->
            {{ aiTranslateError === null ? '' : tError(aiTranslateError) }}
          </p>
          <template v-else>
            <p v-if="result.kind === 'generating'" class="aip-note" data-aip-result-state="generating">
              {{ t('panel.ai_translation.state_generating') }}
            </p>
            <p v-else-if="resultCancelled" class="aip-note" data-aip-result-state="cancelled">
              {{ t('panel.ai_translation.state_cancelled') }}
            </p>
            <p v-else class="aip-hidden" data-aip-result-state="text" />
            <!-- aura-allow-text: DỮ LIỆU (bản dịch AI đang chảy về, không phải chữ giao diện). -->
            <pre class="aip-result-text" data-aip-result-text>{{ result.text }}</pre>
            <p v-if="resultUsage !== null" class="aip-note" data-aip-result-usage>
              <!-- aura-allow-text: KẾT QUẢ của t() (số token, chi phí). -->
              {{ t(resultUsage.key, resultUsage.params) }}
            </p>
          </template>
          <div class="aip-acts">
            <button
              type="button"
              class="aip-btn ui-btn"
              :disabled="!canResend"
              data-aip-resend
              @click="dispatch('ai.prompt_inspector.resend')"
            >
              {{ t('command.ai.prompt_inspector.resend') }}
            </button>
          </div>
        </div>

        <div v-else class="aip-split" data-aip-tab-panel="prompt">
          <div class="aip-doc">
            <section v-if="systemText !== null" class="aip-blk aip-blk-system" data-aip-system="present">
              <div class="aip-bh">
                <span class="aip-blabel">{{ t('ai.prompt_inspector.block_system') }}</span>
                <span class="aip-bchars" data-aip-system-chars>
                  <!-- aura-allow-text: KẾT QUẢ của t() (số ký tự). -->
                  {{ t('ai.prompt_inspector.block_chars', { count: String(systemChars) }) }}
                </span>
              </div>
              <div class="aip-txt">
                <!-- aura-allow-text: DỮ LIỆU (văn bản system prompt đã gửi, không phải chữ giao diện). -->
                <span class="aip-piece aip-piece-system">{{ systemText }}</span>
              </div>
            </section>
            <p v-else class="aip-note" data-aip-system="none">
              {{ t('ai.prompt_inspector.system_none') }}
            </p>
            <p v-if="callNote.kind === 'context_only'" class="aip-note" data-aip-call-note="context_only">
              {{ t('ai.prompt_inspector.call_context_only', { call_index: String(callNote.callIndex) }) }}
            </p>
            <p v-else-if="callNote.kind === 'block'" class="aip-note" data-aip-call-note="block">
              {{ t('ai.prompt_inspector.call_block', { call_index: String(callNote.callIndex), block_number: String(callNote.blockNumber) }) }}
            </p>
            <p v-if="aiPromptRecord.prompt === ''" class="aip-note" data-aip-body-state="empty">
              {{ t('ai.prompt_inspector.body_empty_note') }}
            </p>
            <div v-else data-aip-body-state="present">
              <section
                v-for="(block, i) in blocks"
                :key="`blk-${i}`"
                class="aip-blk"
                :class="`aip-blk-${block.kind}`"
                :data-aip-block-kind="block.kind"
              >
                <div class="aip-bh">
                  <!-- aura-allow-text: KẾT QUẢ của t() qua hàm cục bộ `blockLabel()`. -->
                  <span class="aip-blabel">{{ blockLabel(block.kind) }}</span>
                  <!-- aura-allow-text: KẾT QUẢ của t() qua hàm cục bộ `tierLabel()`. -->
                  <span v-if="block.kind === 'authored'" class="aip-scope">{{ tierLabel(aiPromptRecord.prompt_set_tier) }}</span>
                  <span class="aip-bchars" data-aip-block-chars>
                    <!-- aura-allow-text: KẾT QUẢ của t() (số ký tự). -->
                    {{ t('ai.prompt_inspector.block_chars', { count: String(block.chars) }) }}
                  </span>
                </div>
                <div class="aip-txt">
                  <!-- aura-allow-text: DỮ LIỆU (văn bản của khối prompt đã lắp, không phải chữ giao diện). -->
                  <span
                    v-for="(part, pi) in aiPromptHighlightParts(block.text, block.kind === 'source_segment' ? highlightBlockNumber : null)"
                    :key="`part-${pi}`"
                    :class="['aip-piece', `aip-piece-${block.kind}`, { 'aip-piece-mine': part.highlighted }]"
                    :data-aip-piece-kind="block.kind"
                    :data-aip-piece-mine="part.highlighted ? '' : undefined"
                    >{{ part.text }}</span
                  >
                </div>
              </section>
            </div>

            <p v-if="aiPromptRecord.ledger.source_segment_missing" class="aip-warn" role="alert">
              {{ t('ai.prompt_inspector.source_missing_warning') }}
            </p>

            <div v-if="aiPromptRecord.ledger.unknown_markers.length > 0" class="aip-markers">
              <h4 class="aip-sh2">{{ t('ai.prompt_inspector.markers_heading') }}</h4>
              <ul class="aip-marker-list">
                <li v-for="m in aiPromptRecord.ledger.unknown_markers" :key="m">
                  <!-- aura-allow-text: DỮ LIỆU (toàn văn ký hiệu lạ, không phải chữ giao diện). -->
                  <code>{{ m }}</code>
                </li>
              </ul>
            </div>

            <p v-if="glossaryAsked === null" class="aip-note" data-aip-glossary-kind="not_asked">
              {{ t('ai.prompt_inspector.glossary_not_asked') }}
            </p>
            <p v-else class="aip-summary" data-aip-glossary-kind="asked">
              {{ t('ai.prompt.summary_asked', { count: String(glossaryAsked.injected.length) }) }}
            </p>
            <p v-if="glossaryAsked !== null && glossaryAsked.injected.length === 0" class="aip-note">
              {{ t('ai.prompt_inspector.glossary_injected_empty') }}
            </p>
            <ul v-else-if="glossaryAsked !== null" class="aip-term-list aip-term-list-injected">
              <li v-for="(term, i) in glossaryAsked.injected" :key="`inj-${i}`" class="aip-term-row">
                <!-- aura-allow-text: DỮ LIỆU (thuật ngữ Glossary do người dùng đặt). -->
                <span class="aip-term-source">{{ term.source_term }}</span>
                <span class="aip-term-arrow" aria-hidden="true">→</span>
                <!-- aura-allow-text: DỮ LIỆU (bản dịch đã chốt do người dùng đặt). -->
                <span class="aip-term-translation">{{ term.translation }}</span>
                <!-- aura-allow-text: KẾT QUẢ của t() qua hàm cục bộ `tierLabel()`, không một chuỗi viết thẳng. -->
                <span class="aip-term-tier">{{ tierLabel(term.tier) }}</span>
              </li>
            </ul>
            <p v-if="tmSearched === null" class="aip-note" data-aip-tm-kind="not_asked">
              {{ t('ai.prompt_inspector.tm_not_asked') }}
            </p>
            <p v-else class="aip-note" data-aip-tm-kind="searched">
              {{ t('ai.prompt_inspector.tm_searched', { count: String(tmSearched.similar_segments.length) }) }}
            </p>
            <p v-if="tmSearched !== null && tmSearched.hidden_count > 0" class="aip-note" data-aip-tm-hidden>
              {{ t('tm.policy.hidden_note', { count: String(tmSearched.hidden_count) }) }}
            </p>
            <ul v-if="tmSearched !== null && tmSearched.similar_segments.length > 0" class="aip-term-list aip-term-list-tm">
              <li v-for="(pair, i) in tmSearched.similar_segments" :key="`tm-${i}`" class="aip-term-row">
                <!-- aura-allow-text: DỮ LIỆU (câu nguồn trong TM của người dùng). -->
                <span class="aip-term-source">{{ pair.source_text }}</span>
                <span class="aip-term-arrow" aria-hidden="true">→</span>
                <!-- aura-allow-text: DỮ LIỆU (bản dịch trong TM). -->
                <span class="aip-term-translation">{{ pair.target_text }}</span>
                <!-- aura-allow-text: KẾT QUẢ của t() qua hàm cục bộ `tierLabel()`, không một chuỗi viết thẳng. -->
                <span class="aip-term-tier">{{ tierLabel(pair.tier) }}</span>
                <span class="aip-term-tier">{{ t('ai.prompt_inspector.tm_percent', { percent: String(pair.percent) }) }}</span>
                <span v-if="pair.reference" class="aip-ewhy" data-aip-tm-reference>
                  {{ t('ai.prompt_inspector.tm_reference_tag') }}
                </span>
              </li>
            </ul>

            <section class="aip-excl" data-aip-excluded>
              <h3 class="aip-eh">{{ t('ai.prompt_inspector.excluded_heading') }}</h3>
              <p class="aip-note">{{ t('ai.prompt_inspector.excluded_summary') }}</p>

              <p
                v-if="
                  (glossaryAsked === null ||
                    (glossaryAsked.pending_matched.length === 0 &&
                      glossaryAsked.suppressed_by_pending_overlap.length === 0)) &&
                  (tmSearched === null || tmSearched.omitted_count === 0)
                "
                class="aip-note"
                data-aip-excluded-empty
              >
                {{ t('ai.prompt_inspector.excluded_empty') }}
              </p>

              <ul class="aip-term-list">
                <template v-if="glossaryAsked !== null">
                  <li
                    v-for="(term, i) in glossaryAsked.pending_matched"
                    :key="`pend-${i}`"
                    class="aip-term-row"
                    data-aip-excluded-kind="pending"
                  >
                    <!-- aura-allow-text: DỮ LIỆU (thuật ngữ Glossary do người dùng đặt). -->
                    <span class="aip-term-source">{{ term.source_term }}</span>
                    <!-- aura-allow-text: KẾT QUẢ của t() qua hàm cục bộ `tierLabel()`, không một chuỗi viết thẳng. -->
                    <span class="aip-term-tier">{{ tierLabel(term.tier) }}</span>
                    <span class="aip-ewhy">{{ t('ai.prompt_inspector.excluded_pending_why') }}</span>
                    <button
                      v-if="canConfirmNow(term)"
                      type="button"
                      class="aip-efix ui-btn ui-btn--sm ui-btn--quiet"
                      data-aip-fix="confirm_now"
                      @click="dispatch('ai.prompt_inspector.confirm_now')"
                    >
                      {{ t('command.ai.prompt_inspector.confirm_now') }}
                    </button>
                    <button
                      v-else
                      type="button"
                      class="aip-efix ui-btn ui-btn--sm ui-btn--quiet"
                      data-aip-fix="glossary_entry"
                      @click="dispatch('ai.prompt_inspector.open_glossary_entry')"
                    >
                      {{ t('command.ai.prompt_inspector.open_glossary_entry') }}
                    </button>
                  </li>
                  <li
                    v-for="(term, i) in glossaryAsked.suppressed_by_pending_overlap"
                    :key="`sup-${i}`"
                    class="aip-term-row"
                    data-aip-excluded-kind="suppressed"
                  >
                    <!-- aura-allow-text: DỮ LIỆU (thuật ngữ Glossary do người dùng đặt). -->
                    <span class="aip-term-source">{{ term.source_term }}</span>
                    <!-- aura-allow-text: DỮ LIỆU (bản dịch đã chốt do người dùng đặt). -->
                    <span class="aip-term-translation">{{ term.translation }}</span>
                    <!-- aura-allow-text: KẾT QUẢ của t() qua hàm cục bộ `tierLabel()`, không một chuỗi viết thẳng. -->
                    <span class="aip-term-tier">{{ tierLabel(term.tier) }}</span>
                    <span class="aip-ewhy">
                      {{ t('ai.prompt_inspector.glossary_suppressed_reason_pending_overlap') }}
                    </span>
                    <button type="button" class="aip-efix ui-btn ui-btn--sm ui-btn--quiet" @click="dispatch('ai.prompt_inspector.open_glossary_entry')">
                      {{ t('command.ai.prompt_inspector.open_glossary_entry') }}
                    </button>
                  </li>
                </template>
                <li
                  v-if="tmSearched !== null && tmSearched.omitted_count > 0"
                  class="aip-term-row"
                  data-aip-excluded-kind="tm_omitted"
                >
                  <span class="aip-term-source">
                    <!-- aura-allow-text: KẾT QUẢ của t() (số câu TM). -->
                    {{ t('ai.prompt_inspector.excluded_tm_label', { count: String(tmSearched.omitted_count) }) }}
                  </span>
                  <span class="aip-ewhy">{{ t('ai.prompt_inspector.excluded_tm_why') }}</span>
                  <button type="button" class="aip-efix ui-btn ui-btn--sm ui-btn--quiet" @click="dispatch('ai.prompt_inspector.tm_concordance')">
                    <!-- aura-allow-text: KẾT QUẢ của t() (số câu TM). -->
                    {{ t('ai.prompt_inspector.tm_see_all', { count: String(tmSearched.omitted_count) }) }}
                  </button>
                </li>
              </ul>
            </section>
          </div>

          <aside class="aip-side">
            <h3 class="aip-sh">{{ t('ai.prompt_inspector.budget_heading') }}</h3>
            <div class="aip-bar" aria-hidden="true">
              <i
                v-for="kind in BAR_KINDS"
                :key="`bar-${kind}`"
                :class="`aip-bar-${kind}`"
                :style="{ width: barWidth(kind) }"
              />
            </div>
            <div v-for="kind in BAR_KINDS" :key="`row-${kind}`" class="aip-brow">
              <span class="aip-sw" :class="`aip-bar-${kind}`" />
              <!-- aura-allow-text: KẾT QUẢ của t() qua hàm cục bộ `blockLabel()`. -->
              <span>{{ blockLabel(kind) }}</span>
              <!-- aura-allow-text: DỮ LIỆU (số ký tự của tầng). -->
              <span class="aip-bn">{{ charsByKind[kind] }}</span>
            </div>

            <div class="aip-total" data-aip-token-line>
              <p v-if="tokenLine.kind === 'real'" class="aip-trow" data-aip-token-state="real">
                <!-- aura-allow-text: KẾT QUẢ của t() (số token thật). -->
                {{ t('ai.prompt_inspector.tokens_real', { count: String(tokenLine.promptTokens) }) }}
              </p>
              <p v-else-if="tokenLine.kind === 'unavailable'" class="aip-trow" data-aip-token-state="unavailable">
                {{ t('ai.prompt_inspector.tokens_unavailable') }}
              </p>
              <p v-else class="aip-trow" data-aip-token-state="unsent">
                {{ t('ai.prompt_inspector.tokens_unsent') }}
              </p>
              <p v-if="dynamicRatio !== null" class="aip-trow" data-aip-dynamic-ratio>
                <!-- aura-allow-text: KẾT QUẢ của t() (tỉ lệ chèn động). -->
                {{ t('ai.prompt_inspector.dynamic_ratio', { percent: formatPercentParam(dynamicRatio) }) }}
              </p>
              <p v-if="tokenLine.kind === 'real' && tokenLine.costUsd !== null" class="aip-trow" data-aip-cost>
                <!-- aura-allow-text: KẾT QUẢ của t() (chi phí ước tính). -->
                {{ t('ai.prompt_inspector.cost_estimate', { cost_usd: formatUsdParam(tokenLine.costUsd) }) }}
              </p>
            </div>

            <div class="aip-acts">
              <button
                type="button"
                class="aip-btn aip-btn-primary ui-btn ui-btn--primary"
                data-aip-copy
                :data-aip-copy-state="aiPromptCopyState"
                @click="dispatch('ai.prompt_inspector.copy')"
              >
                {{ t(copyLabelKey) }}
              </button>
              <button
                type="button"
                class="aip-btn ui-btn"
                :disabled="!canResend"
                data-aip-resend
                @click="dispatch('ai.prompt_inspector.resend')"
              >
                {{ t('command.ai.prompt_inspector.resend') }}
              </button>
              <button type="button" class="aip-btn ui-btn" @click="dispatch('ai.prompt_inspector.open_work_prompts')">
                {{ t('command.ai.prompt_inspector.open_work_prompts') }}
              </button>
            </div>
            <p class="aip-note aip-exact">{{ t('ai.prompt_inspector.exact_note') }}</p>
          </aside>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.aip-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: xếp lớp CƠ HỌC — cùng lý do các lớp phủ khác. */
  display: flex;
  justify-content: center;
  align-items: stretch;
  background: var(--color-background);
}

:root[data-platform='mac'] .aip-scrim {
  padding-top: var(--space-titlebar-height);
}

.aip-panel {
  width: 100%;
  overflow: auto;
  background: var(--color-surface);
  display: flex;
  flex-direction: column;
  padding: var(--space-panel-inline);
}

.aip-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.aip-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.aip-setbar {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 4);
  margin-bottom: calc(var(--space-unit) * 2);
}

.aip-set-field {
  flex: 0 1 50%;
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
}

.aip-set-select {
  flex: 1;
  min-width: 0;
}

.aip-empty {
  margin: 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.aip-tabs {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 5);
  padding-bottom: calc(var(--space-unit) * 1);
  margin-bottom: calc(var(--space-unit) * 3);
  border-bottom: 1px solid var(--color-outline);
}

.aip-meta {
  margin: 0 0 0 auto;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.aip-identity {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.aip-stale,
.aip-other-set,
.aip-read-error {
  margin: 0 0 calc(var(--space-unit) * 3) 0;
  padding: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-error);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.aip-split {
  display: grid;
  grid-template-columns: 1fr 310px;
  gap: calc(var(--space-unit) * 5);
}

.aip-doc {
  min-width: 0;
}

.aip-side {
  padding-left: calc(var(--space-unit) * 4);
  border-left: 1px solid var(--color-outline);
}

.aip-sh {
  font-size: var(--font-ui-label);
  font-family: var(--face-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
  margin: 0 0 calc(var(--space-unit) * 2) 0;
}

.aip-sh2 {
  font-size: var(--font-ui-label);
  font-family: var(--face-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
  margin: calc(var(--space-unit) * 3) 0 calc(var(--space-unit) * 1) 0;
}

.aip-note,
.aip-summary {
  margin: calc(var(--space-unit) * 1) 0 0 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.aip-summary {
  color: var(--color-on-surface);
  font-weight: var(--weight-read-title);
}

.aip-warn {
  margin: calc(var(--space-unit) * 2) 0 0 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.aip-hidden {
  display: none;
}

.aip-blk {
  padding-left: calc(var(--space-unit) * 3);
  border-left: 2px solid var(--color-outline-faint);
  margin-bottom: calc(var(--space-unit) * 4);
}

.aip-blk-glossary {
  border-left-color: var(--color-primary);
}

.aip-blk-tm {
  border-left-color: var(--color-tm-rule);
}

.aip-blk-system {
  border-left-color: var(--color-outline);
}

.aip-blk-source_segment {
  border-left-color: var(--color-on-surface);
}

.aip-bh {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
  margin-bottom: calc(var(--space-unit) * 1);
}

.aip-blabel {
  font-size: var(--font-ui-label);
  font-family: var(--face-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
}

.aip-blk-glossary .aip-blabel {
  color: var(--color-primary);
}

.aip-blk-tm .aip-blabel {
  color: var(--color-tm-text);
}

.aip-blk-source_segment .aip-blabel {
  color: var(--color-on-surface);
}

.aip-scope {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  border: 1px solid var(--color-outline);
  padding: 0 calc(var(--space-unit) * 1);
}

.aip-bchars {
  margin-left: auto;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface-variant);
}

.aip-txt,
.aip-result-text {
  margin: 0;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.aip-piece-glossary {
  background: var(--color-surface-accent);
}

.aip-piece-source_segment {
  color: var(--color-on-surface);
}

.aip-piece-tm {
  background: var(--color-surface-tm);
}

.aip-piece-mine {
  background: var(--color-surface-accent);
}

.aip-markers {
  margin-top: calc(var(--space-unit) * 2);
}

.aip-marker-list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
}

.aip-marker-list code {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-error);
}

.aip-excl {
  margin-top: calc(var(--space-unit) * 5);
  padding-top: calc(var(--space-unit) * 4);
  border-top: 1px solid var(--color-outline);
}

.aip-eh {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.aip-term-list {
  margin: calc(var(--space-unit) * 2) 0 0 0;
  padding: 0;
  list-style: none;
}

.aip-term-row {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 2) 0;
  border-bottom: 1px solid var(--color-outline-faint);
  font-family: var(--family-read);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.aip-term-row:last-child {
  border-bottom: none;
}

.aip-term-arrow {
  color: var(--color-on-surface-variant);
}

.aip-term-list-injected .aip-term-row {
  background: var(--color-surface-accent);
}

.aip-term-tier {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-primary);
  border: 1px solid var(--color-outline);
  padding: 0 calc(var(--space-unit) * 1);
}

.aip-ewhy {
  flex: 1;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.aip-bar {
  display: flex;
  height: calc(var(--space-unit) * 1);
  overflow: hidden;
  background: var(--color-surface-sunken);
  margin-bottom: calc(var(--space-unit) * 2);
}

.aip-bar i {
  display: block;
  height: 100%;
}

.aip-bar-authored {
  background: var(--color-ornament);
}

.aip-bar-glossary {
  background: var(--color-primary);
}

.aip-bar-tm {
  background: var(--color-tm-rule);
}

.aip-bar-source_segment {
  background: var(--color-on-surface);
}

.aip-brow {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
}

.aip-sw {
  flex: none;
  width: calc(var(--space-unit) * 2);
  height: calc(var(--space-unit) * 2);
}

.aip-bn {
  margin-left: auto;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface-variant);
}

.aip-total {
  margin-top: calc(var(--space-unit) * 2);
  padding-top: calc(var(--space-unit) * 2);
  border-top: 1px solid var(--color-outline);
}

.aip-trow {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.aip-acts {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
  margin-top: calc(var(--space-unit) * 4);
}

.aip-exact {
  margin-top: calc(var(--space-unit) * 3);
  padding-top: calc(var(--space-unit) * 2);
  border-top: 1px solid var(--color-outline);
}
</style>
