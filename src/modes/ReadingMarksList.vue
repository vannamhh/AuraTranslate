<script setup lang="ts">
import { computed, useTemplateRef } from 'vue'
import { dispatch } from '../commands'
import { t, tError } from '../i18n'
import { commandTitle } from '../config/shortcutsState'
import {
  readingMarks,
  readingMarksBusy,
  readingMarksClearPending,
  readingMarksCursor,
  readingMarksError,
  readingMarksHaveLoaded,
  setReadingMarksCursor,
} from './readingState'

const props = withDefaults(defineProps<{ showHint?: boolean }>(), { showHint: true })

const body = useTemplateRef<HTMLElement>('body')

const marksWalkText = computed(() => t('mode.reading.marks_walk', { mark_count: String(readingMarks.value.length) }))

const marksClearText = computed(() =>
  readingMarksClearPending.value
    ? t('mode.reading.marks_clear_confirm', { mark_count: String(readingMarks.value.length) })
    : t('mode.reading.marks_clear'),
)

function onMarksKeydown(event: KeyboardEvent): void {
  if (event.target !== event.currentTarget) return
  if (event.key === 'ArrowDown') dispatch('reading.marks_next')
  else if (event.key === 'ArrowUp') dispatch('reading.marks_prev')
  else if (event.key === 'Enter') dispatch('reading.marks_open')
  else if (event.key === 'Backspace') dispatch('reading.marks_unmark')
  else return
  event.preventDefault()
}

defineExpose({
  focus: (): void => body.value?.focus(),
  element: (): HTMLElement | null => body.value,
})
</script>

<template>
  <div ref="body" class="marks-body" tabindex="-1" @keydown="onMarksKeydown($event)">
    <p v-if="props.showHint" class="marks-hint">{{ t('mode.reading.marks_hint') }}</p>
    <!-- aura-allow-text: nhánh có chữ đi qua t(); nhánh còn lại là chuỗi rỗng. -->
    <p role="status">{{ readingMarksBusy ? t('mode.reading.marks_loading') : '' }}</p>
    <!-- aura-allow-text: lỗi dây đi qua tError()/chuỗi rỗng. -->
    <p role="status">{{ readingMarksError === null ? '' : tError(readingMarksError) }}</p>
    <!-- aura-allow-text: cả hai nhánh đi qua t()/chuỗi rỗng. -->
    <p role="status">{{ readingMarksHaveLoaded && readingMarksError === null && readingMarks.length === 0 ? t('mode.reading.marks_empty') : '' }}</p>
    <ul class="marks-list">
      <li
        v-for="(mark, index) in readingMarks"
        :key="mark.segment_id"
        class="marks-row"
        :class="{ current: index === readingMarksCursor }"
        :aria-current="index === readingMarksCursor ? 'true' : undefined"
      >
        <span class="marks-chapter">{{ t('mode.reading.marks_row_chapter', { chapter_ord: String(mark.chapter_ord) }) }}</span>
        <!-- aura-allow-text: target_text là dữ liệu Tác phẩm. -->
        <span class="marks-text">{{ mark.target_text === '' ? mark.source_text : mark.target_text }}</span>
        <span v-if="mark.is_retired" class="retired-note">{{ t('mode.reading.marks_retired') }}</span>
        <button
          type="button"
          class="btn ui-btn"
          :disabled="readingMarksBusy"
          @pointerdown="setReadingMarksCursor(index)"
          @focus="setReadingMarksCursor(index)"
          @click="dispatch('reading.marks_open')"
        >{{ t('mode.reading.marks_row_edit') }}</button>
        <button
          type="button"
          class="btn ui-btn"
          :disabled="readingMarksBusy"
          @pointerdown="setReadingMarksCursor(index)"
          @focus="setReadingMarksCursor(index)"
          @click="dispatch('reading.marks_unmark')"
        >{{ t('mode.reading.marks_row_unmark') }}</button>
      </li>
    </ul>
    <div class="marks-actions">
      <button type="button" class="btn ui-btn" :disabled="readingMarksBusy || readingMarks.length === 0" @click="dispatch('reading.marks_walk')">
        <!-- aura-allow-text: marksWalkText goes through t(). -->
        {{ marksWalkText }}
      </button>
      <button
        type="button"
        class="btn ui-btn"
        data-reading-marks-clear
        :disabled="readingMarksBusy || readingMarks.length === 0"
        @click="dispatch('reading.marks_clear')"
      >
        <!-- aura-allow-text: marksClearText goes through t(). -->
        {{ marksClearText }}
      </button>
      <button
        v-if="readingMarksClearPending"
        type="button"
        class="btn ui-btn"
        data-reading-marks-clear-cancel
        :title="commandTitle('reading.marks_clear_cancel')"
        @click="dispatch('reading.marks_clear_cancel')"
      >{{ t('mode.reading.marks_clear_cancel') }}</button>
      <slot name="close" />
    </div>
    <p class="marks-hint">{{ t('mode.reading.marks_keys') }}</p>
  </div>
</template>

<style scoped>
.marks-body:focus {
  /* aura-allow-outline-none: tabindex=-1 root, focused programmatically, not an interactive control. */
  outline: none;
}

.marks-hint {
  color: var(--color-on-surface-variant);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-family: var(--face-ui-sm);
}

.marks-list {
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
}

.marks-row {
  display: flex;
  align-items: baseline;
  gap: var(--space-unit);
  padding: var(--space-panel-inline) var(--space-panel-inline) 13px; /* aura-allow-spacing: 1px bottom border sits inside the 14px inset */
  border-bottom: 1px solid var(--color-outline);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  font-family: var(--face-ui-md);
}

.marks-row.current {
  background: var(--color-surface-accent);
}

.marks-chapter {
  flex: none;
  color: var(--color-on-surface-variant);
}

.marks-text {
  flex: 1 1 auto;
  overflow-wrap: anywhere;
}

.retired-note {
  display: block;
  color: var(--color-tm-text);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-family: var(--face-ui-sm);
}

.marks-actions {
  display: flex;
  gap: var(--space-unit);
  flex-wrap: wrap;
}
</style>
