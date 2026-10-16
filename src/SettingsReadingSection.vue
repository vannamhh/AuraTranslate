<script setup lang="ts">
// Reading-mode section of the Settings frame: the four options Reading mode already has, on the
// same state and the same write schedule as `modes/readingState.ts`. `AppConfig` is `GlobalOnly`.
import { t } from './i18n'
import { dispatch } from './commands'
import {
  READING_FONT_SIZE_MAX,
  READING_FONT_SIZE_MIN,
  READING_LINE_HEIGHT_FLOOR,
  currentReadingLevel,
  effectiveFontSize,
  effectiveLineHeight,
  readingBilingual,
  setFontSize,
  setLineHeight,
} from './modes/readingState'

function bilingualStateKey(): string {
  return readingBilingual.value ? 'settings.reading.bilingual_on' : 'settings.reading.bilingual_off'
}

function onFontSizeInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement && target.value !== '') setFontSize(Number(target.value))
}

function onLineHeightInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement && target.value !== '') setLineHeight(Number(target.value))
}
</script>

<template>
  <div class="sr-body" data-settings-reading>
    <div class="sr-row" role="group" :aria-label="t('mode.reading.level_group')">
      <span class="sr-label">{{ t('mode.reading.level_group') }}</span>
      <button
        type="button"
        class="sr-btn"
        data-settings-reading-level="lg"
        :aria-pressed="currentReadingLevel === 'lg'"
        @click="dispatch('reading.level_airy')"
      >
        {{ t('mode.reading.level_airy') }}
      </button>
      <button
        type="button"
        class="sr-btn"
        data-settings-reading-level="md"
        :aria-pressed="currentReadingLevel === 'md'"
        @click="dispatch('reading.level_balanced')"
      >
        {{ t('mode.reading.level_balanced') }}
      </button>
      <button
        type="button"
        class="sr-btn"
        data-settings-reading-level="sm"
        :aria-pressed="currentReadingLevel === 'sm'"
        @click="dispatch('reading.level_dense')"
      >
        {{ t('mode.reading.level_dense') }}
      </button>
    </div>

    <div class="sr-row">
      <span class="sr-label">{{ t('mode.reading.bilingual') }}</span>
      <button
        type="button"
        class="sr-btn"
        data-settings-reading-bilingual
        :aria-pressed="readingBilingual"
        @click="dispatch('reading.toggle_bilingual')"
      >
        {{ t(bilingualStateKey()) }}
      </button>
    </div>

    <label class="sr-row">
      <span class="sr-label">{{ t('mode.reading.tuner_font_size') }}</span>
      <input
        type="range"
        class="sr-range"
        data-settings-reading-font-size
        :min="READING_FONT_SIZE_MIN"
        :max="READING_FONT_SIZE_MAX"
        step="0.5"
        :value="effectiveFontSize"
        @input="onFontSizeInput($event)"
      />
      <!-- aura-allow-text: giá trị số đang áp — DỮ LIỆU. -->
      <span class="sr-value">{{ effectiveFontSize }}px</span>
    </label>

    <label class="sr-row">
      <span class="sr-label">{{ t('mode.reading.tuner_line_height') }}</span>
      <input
        type="range"
        class="sr-range"
        data-settings-reading-line-height
        :min="READING_LINE_HEIGHT_FLOOR"
        max="2.4"
        step="0.01"
        :value="effectiveLineHeight"
        @input="onLineHeightInput($event)"
      />
      <!-- aura-allow-text: giá trị số đang áp — DỮ LIỆU. -->
      <span class="sr-value">{{ effectiveLineHeight }}</span>
    </label>

    <p class="sr-note">{{ t('settings.reading.note') }}</p>
  </div>
</template>

<style scoped>
.sr-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-panel-inline);
  padding: var(--space-panel-block) 0;
  border-bottom: 1px solid var(--color-outline);
}

.sr-label {
  min-width: 10em;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.sr-btn {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: none;
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.sr-btn[aria-pressed='true'] {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.sr-range {
  flex: 0 1 280px;
}

.sr-value {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface-variant);
}

.sr-note {
  margin: var(--space-panel-block) 0 0 0;
  padding-left: 12px;
  border-left: 2px solid var(--color-tm-rule);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-tm-text);
}
</style>
