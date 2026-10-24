<script setup lang="ts">
// Dải "Chờ chốt lần đầu gặp" — Story 3.6, FR114.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 DẢI, KHÔNG PANEL — chèn ở CHÂN WORKSPACE, ngay dưới `<GlossaryQuickAdd />` và ngay
// trên `<StatusBar />` (App.vue) — cùng slot, cùng khuôn `GlossaryQuickAdd.vue:1-13`.
// ─────────────────────────────────────────────────────────────────────────────
// Nó KHÔNG đăng ký `FocusOwner` (`commands/index.ts::FOCUS_OWNERS` — dải không thêm vào
// đó, nó không phải panel — tiền lệ `GlossaryQuickAdd.vue:12-13`).
//
// `v-if` TỰ QUẢN qua `topmostStrip(...) === 'glossary_confirm'` — sổ ưu tiên
// (`panels/inlineStripPriority.ts`) quyết định dải nào THẮNG khi cả `GlossaryQuickAdd`
// (thao tác người dùng VỪA yêu cầu) LẪN dải này cùng đủ điều kiện.
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
import { computed, nextTick, toRaw, watch } from 'vue'
import { useTemplateRef } from 'vue'
import { dispatch } from './commands'
import { t, tError } from './i18n'
import {
  aimGlossaryConfirmOption,
  confirmStripAimedOption,
  confirmStripCategory,
  confirmStripEmptyInputError,
  confirmStripFocusRequest,
  confirmStripIsOpen,
  confirmStripOccurrenceCount,
  confirmStripOptions,
  confirmStripSaveError,
  confirmStripSaving,
  confirmStripSourceTerm,
  confirmStripSuggestionStatus,
  confirmStripTermOrigin,
  confirmStripTier,
  confirmStripTmError,
  confirmStripTmHiddenCount,
  confirmStripUndo,
  confirmStripUndoError,
  confirmStripUndoing,
  confirmStripTranslationInput,
  syncGlossaryConfirmStripTarget,
} from './glossaryConfirmStripState'
import type { ConfirmStripOption } from './glossaryConfirmStripState'
import { shortcutRows } from './config/shortcutsState'
import { quickAddIsOpen } from './glossaryQuickAddState'
import { editorCaretSegmentId, editorSegments } from './panels/editorPanelState'
import { glossaryMarks, glossaryMarksHaveLoaded } from './panels/glossaryMarksState'
import { topmostStrip } from './panels/inlineStripPriority'
import type { InlineStripKind } from './panels/inlineStripPriority'

/**
 * `watch` THẬT của `editorCaretSegmentId` sống Ở ĐÂY (một LEAF), không trong
 * `glossaryConfirmStripState.ts` — §Design Notes của spec giải thích lý do: một `import`
 * ngược từ tệp state đó lên `editorPanelState.ts` dựng một VÒNG. Component này được PHÉP
 * import cả hai phía.
 *
 * `glossaryMarksHaveLoaded()` là một HÀM (không một `ref`) nên nó không tự kích `watch` —
 * theo dõi luôn `glossaryMarks` (đổi giá trị mỗi lượt nạp/nạp lại/reset) làm nhiệm vụ đó.
 */
watch(
  [editorCaretSegmentId, editorSegments, glossaryMarks],
  ([segmentId, segments, marks]) => {
    syncGlossaryConfirmStripTarget(segmentId, toRaw(segments), glossaryMarksHaveLoaded(), marks)
  },
  { immediate: true },
)

const eligible = computed<InlineStripKind[]>(() => {
  const list: InlineStripKind[] = []
  if (quickAddIsOpen.value) list.push('glossary_quick_add')
  if (confirmStripIsOpen.value) list.push('glossary_confirm')
  return list
})
const isVisible = computed(() => topmostStrip(eligible.value) === 'glossary_confirm')

/**
 * 🔵 THÊM 2026-08-22 (rà ba lớp) — `aria-describedby` cho ô nhập, trỏ tới đoạn `role="alert"`/
 * `role="status"` ĐANG RENDER (`id="gcs-status-msg"` — đúng MỘT trong ba đoạn luôn render tại
 * một thời điểm, xem `<template>`). Không có liên kết này, người dùng đọc màn hình đứng
 * trong ô nhập không biết có một câu lỗi/trạng thái đang hiện ngay bên dưới.
 */
/**
 * 🔵 THÊM 2026-08-24 (Story 3.7, FR113) — dòng *"chưa cài dữ liệu từ điển"* nay CŨNG chiếm
 * `id="gcs-status-msg"`, cùng chuỗi `v-if`/`v-else-if` với ba đoạn cũ (đúng MỘT đoạn render
 * tại một thời điểm — xem `<template>`). Ưu tiên THẤP NHẤT: lỗi/đang lưu là phản hồi cho
 * THAO TÁC người dùng vừa làm, còn dòng này chỉ mô tả TRẠNG THÁI của dữ liệu nền.
 */
const statusMessageId = computed<string | undefined>(() =>
  confirmStripSaveError.value !== null ||
  confirmStripEmptyInputError.value ||
  confirmStripSaving.value ||
  confirmStripSuggestionStatus.value === 'dict_unavailable'
    ? 'gcs-status-msg'
    : undefined,
)

const translationInputEl = useTemplateRef<HTMLInputElement>('translationInputEl')
const root = useTemplateRef<HTMLElement>('root')

const metaText = computed(() =>
  confirmStripOccurrenceCount.value === null
    ? t('glossary.confirm.meta_pending')
    : t('glossary.confirm.meta_occurrences', { count: String(confirmStripOccurrenceCount.value) }),
)

const ORIGIN_KEYS: Readonly<Partial<Record<string, string>>> = {
  manual: 'glossary.manage.origin_manual',
  import_scan: 'glossary.manage.origin_import_scan',
  review_harvest: 'glossary.manage.origin_review_harvest',
  file_import: 'glossary.manage.origin_file_import',
}

const originLine = computed(() => {
  const parts: string[] = []
  const originKey = ORIGIN_KEYS[confirmStripTermOrigin.value ?? '']
  if (originKey !== undefined) parts.push(t('glossary.confirm.origin_clause', { term_origin: t(originKey) }))
  if (confirmStripCategory.value !== null) {
    parts.push(
      t('glossary.confirm.category_clause', { category: t(`glossary.manage.category_${confirmStripCategory.value}`) }),
    )
  }
  return parts.join(' · ')
})

const undoChord = computed(() => {
  const row = shortcutRows.value.find((r) => r.id === 'glossary.confirm.undo')
  const chord = row?.display.join(' / ') ?? ''
  return chord === '' ? null : chord
})

const undoLabel = computed(() =>
  confirmStripUndoing.value ? t('glossary.confirm.undoing') : t('glossary.confirm.undo'),
)

function optionSource(option: ConfirmStripOption): string {
  if (option.isHanViet) {
    return option.tmCount === null
      ? t('glossary.confirm.source_han_viet')
      : t('glossary.confirm.source_han_viet_tm', { count: String(option.tmCount) })
  }
  return t('glossary.confirm.source_tm', { count: String(option.tmCount ?? 0) })
}

const DIGIT_CODE = /^(?:Digit|Numpad)([1-9])$/

/** Digits confirm a proposal only when focus is not in the free-text input. */
function onKeydown(event: KeyboardEvent): void {
  if (event.target === translationInputEl.value) return
  if (event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return
  const digit = DIGIT_CODE.exec(event.code)?.[1]
  if (digit !== undefined) {
    const index = Number(digit) - 1
    if (index >= confirmStripOptions.value.length) return
    event.preventDefault()
    aimGlossaryConfirmOption(index)
    dispatch('glossary.confirm.save')
  } else if (event.code === 'Enter' && event.target === event.currentTarget) {
    event.preventDefault()
    dispatch('glossary.confirm.save')
  }
}

/** Focus is requested only explicitly (`glossary.confirm.focus`): the strip never steals it on
 * appearing. With proposals on offer the strip itself takes it so digits work. */
watch(confirmStripFocusRequest, () => {
  void nextTick(() => {
    if (confirmStripOptions.value.length > 0) root.value?.focus()
    else translationInputEl.value?.focus()
  })
})
</script>

<template>
  <div
    v-if="confirmStripUndo !== null || confirmStripUndoError !== null"
    class="gcs-done"
    role="status"
    data-glossary-confirm-done
  >
    <template v-if="confirmStripUndo !== null">
      <!-- aura-allow-text: result of t() over user data. -->
      <span class="gcs-done-text">
        {{ t('glossary.confirm.done', { source_term: confirmStripUndo.sourceTerm, translation: confirmStripUndo.translation }) }}
      </span>
      <button
        type="button"
        class="gcs-act gcs-undo"
        :disabled="confirmStripUndoing"
        @click="dispatch('glossary.confirm.undo')"
      >
        <!-- aura-allow-text: result of t() computed in the script. -->
        {{ undoLabel }}
      </button>
      <!-- aura-allow-text: the user's bound chord. -->
      <kbd v-if="undoChord !== null" class="gcs-key">{{ undoChord }}</kbd>
    </template>
    <span v-if="confirmStripUndoError !== null" class="gcs-error" role="alert">
      {{ tError(confirmStripUndoError) }}
    </span>
  </div>
  <form
    v-if="isVisible"
    ref="root"
    class="glossary-confirm-strip"
    tabindex="-1"
    :aria-label="t('glossary.confirm.title', { source_term: confirmStripSourceTerm ?? '' })"
    @submit.prevent="dispatch('glossary.confirm.save')"
    @keydown.esc.prevent="dispatch('glossary.confirm.defer')"
    @keydown="onKeydown"
  >
    <fieldset class="gcs-fieldset" :disabled="confirmStripSaving">
      <div class="gcs-row gcs-title-row">
        <!-- aura-allow-text: the Glossary source term, user data. -->
        <span class="gcs-title">{{ confirmStripSourceTerm }}</span>
        <!-- aura-allow-text: result of t(). -->
        <span class="gcs-tier">
          {{ confirmStripTier === 'work' ? t('glossary.quick_add.tier_work') : t('glossary.quick_add.tier_global') }}
        </span>
        <!-- aura-allow-text: result of t() computed in the script. -->
        <span class="gcs-occurrence">{{ metaText }}</span>
        <!-- aura-allow-text: result of t() computed in the script. -->
        <span v-if="originLine !== ''" class="gcs-origin">{{ originLine }}</span>
      </div>

      <ol v-if="confirmStripOptions.length > 0" class="gcs-options">
        <li v-for="(option, i) in confirmStripOptions" :key="option.text">
          <button
            type="button"
            class="gcs-option"
            :class="{ 'gcs-option-aimed': i === confirmStripAimedOption }"
            :aria-keyshortcuts="String(i + 1)"
            @focus="aimGlossaryConfirmOption(i)"
            @mousedown="aimGlossaryConfirmOption(i)"
            @click="dispatch('glossary.confirm.save')"
          >
            <!-- aura-allow-text: the option's ordinal, not a sentence. -->
            <kbd class="gcs-key">{{ i + 1 }}</kbd>
            <!-- aura-allow-text: a proposed translation, user or dictionary data. -->
            <span class="gcs-option-text">{{ option.text }}</span>
            <!-- aura-allow-text: result of t() computed in the script. -->
            <span class="gcs-option-source">{{ optionSource(option) }}</span>
          </button>
        </li>
      </ol>

      <p v-if="confirmStripTmHiddenCount > 0" class="gcs-origin" role="status" data-tm-hidden-note>
        {{ t('tm.policy.hidden_note', { count: String(confirmStripTmHiddenCount) }) }}
      </p>

      <label class="gcs-field">
        <span class="gcs-field-label">{{ t('glossary.confirm.translation_label') }}</span>
        <input
          ref="translationInputEl"
          v-model="confirmStripTranslationInput"
          type="text"
          class="gcs-input"
          autocomplete="off"
          :placeholder="t('glossary.confirm.type_other_placeholder')"
          :aria-describedby="statusMessageId"
          @focus="aimGlossaryConfirmOption(null)"
          @input="aimGlossaryConfirmOption(null)"
        />
      </label>

      <p v-if="confirmStripTmError !== null" class="gcs-status gcs-error" role="alert">
        {{ tError(confirmStripTmError) }}
      </p>
      <p v-if="confirmStripSaveError !== null" id="gcs-status-msg" class="gcs-status gcs-error" role="alert">
        {{ tError(confirmStripSaveError) }}
      </p>
      <p v-else-if="confirmStripEmptyInputError" id="gcs-status-msg" class="gcs-status gcs-error" role="alert">
        {{ t('glossary.confirm.empty_error') }}
      </p>
      <p v-else-if="confirmStripSaving" id="gcs-status-msg" class="gcs-status" role="status">
        {{ t('glossary.confirm.saving') }}
      </p>
      <p
        v-else-if="confirmStripSuggestionStatus === 'dict_unavailable'"
        id="gcs-status-msg"
        class="gcs-status"
        role="status"
      >
        {{ t('glossary.confirm.suggestion_unavailable') }}
      </p>

      <div class="gcs-row gcs-actions">
        <button type="submit" class="gcs-act gcs-act-primary">{{ t('glossary.confirm.save') }}</button>
        <button type="button" class="gcs-act" @click="dispatch('glossary.confirm.defer')">
          {{ t('glossary.confirm.defer') }}
        </button>
        <span class="gcs-status">{{ t('glossary.confirm.defer_hint') }}</span>
      </div>

      <p class="gcs-status gcs-why">{{ t('glossary.confirm.rag_note') }}</p>
    </fieldset>
  </form>
</template>

<style scoped>
.gcs-done {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: calc(var(--space-unit) * 3);
  padding: var(--space-panel-inline);
  border-top: 1px solid var(--color-outline);
  background: var(--color-surface);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.gcs-done-text {
  font-weight: var(--weight-ui-md-strong);
}

/*
 * AC — dải ĐẨY nội dung workspace lên, không phủ lên nó: KHÔNG `position: fixed`, KHÔNG
 * `z-index`, KHÔNG `box-shadow` — cùng khuôn `GlossaryQuickAdd.vue`. Màu và cỡ chữ CHỈ qua
 * token (`check:tokens` Kiểm B/B2/F).
 */
.glossary-confirm-strip {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
  padding: var(--space-panel-inline);
  border-top: 1px solid var(--color-outline);
  background: var(--color-surface);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.gcs-fieldset {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
  border: none;
  margin: 0;
  padding: 0;
}

.gcs-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: calc(var(--space-unit) * 4);
}

.gcs-title {
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.gcs-tier {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gcs-occurrence {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gcs-field {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  flex: 1;
  min-width: 8rem;
}

.gcs-field-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gcs-origin {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gcs-options {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin: 0;
  padding: 0;
  list-style: none;
}

.gcs-option {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 3);
  width: 100%;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid transparent;
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  text-align: left;
  cursor: pointer;
}

.gcs-option-aimed {
  border-color: var(--color-primary);
}

.gcs-key {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-primary);
}

.gcs-option-source {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gcs-why {
  margin: 0;
}

.gcs-input {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
}

.gcs-status {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gcs-error {
  color: var(--color-error);
}

.gcs-actions {
  gap: calc(var(--space-unit) * 2);
}

.gcs-act {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  cursor: pointer;
}

.gcs-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.gcs-act-primary {
  color: var(--color-on-surface);
  border-color: var(--color-primary);
}
</style>
