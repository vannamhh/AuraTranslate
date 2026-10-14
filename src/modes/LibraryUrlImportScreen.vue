<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from '../i18n'
import { dispatch } from '../commands'
import { focusReturnTargetOnOpen } from '../commands/focus'
import { recordImportOpener, takeRecordedImportOpener } from '../importOpener'
import { importPreviewIsOpen } from '../importPreviewState'
import {
  busy,
  destinationExistingWorkAvailable,
  destinationLibraryGenuinelyEmpty,
  destinationMode,
  destinationPending,
  destinationWorkId,
  destinationWorks,
  destinationWorksError,
  effectiveName,
  lastError,
  name,
  pastedUrlCount,
  pastedUrlDomainCount,
  pastedUrls,
  pickedDestinationWork,
  setDestinationMode,
  setDestinationWorkId,
  sourceLang,
} from './libraryImport'
import { libraryUrlImportScreenIsOpen } from './libraryImportOverlayState'

const textarea = useTemplateRef<HTMLTextAreaElement>('textarea')
const gutter = useTemplateRef<HTMLElement>('gutter')

let returnFocusTo: HTMLElement | null = null

const lineNumbers = computed<number[]>(() => {
  const count = Math.max(1, pastedUrls.value.split('\n').length)
  return Array.from({ length: count }, (_, i) => i + 1)
})

const covered = computed<boolean>(() => importPreviewIsOpen.value)

const submitLabel = computed<string>(() =>
  destinationMode.value === 'existing'
    ? t('mode.library.url_screen_submit_append', { count: String(pastedUrlCount.value) })
    : t('mode.library.url_screen_submit', { count: String(pastedUrlCount.value) }),
)

watch(libraryUrlImportScreenIsOpen, (open) => {
  if (open) {
    returnFocusTo = takeRecordedImportOpener() ?? focusReturnTargetOnOpen('[data-library-url-screen-open]')
    void nextTick(() => textarea.value?.focus())
    return
  }
  const back = returnFocusTo
  returnFocusTo = null
  void nextTick(() => {
    if (back !== null && back.isConnected) {
      back.focus()
      return
    }
    const opener = document.querySelector<HTMLElement>('[data-library-url-screen-open]')
    if (opener !== null) opener.focus()
  })
})

function onTextareaScroll(): void {
  if (gutter.value !== null && textarea.value !== null) gutter.value.scrollTop = textarea.value.scrollTop
}

function onEscape(): void {
  if (busy.value) return
  dispatch('library.import_urls_close')
}
</script>

<template>
  <section
    v-if="libraryUrlImportScreenIsOpen"
    v-show="!covered"
    class="us"
    aria-labelledby="us-title"
    data-library-url-import-screen
    @keydown.esc="onEscape"
  >
    <header class="us-head">
      <h2 id="us-title" class="us-title">{{ t('mode.library.url_screen_title') }}</h2>
      <p class="us-note">{{ t('mode.library.url_screen_subtitle') }}</p>
    </header>

    <div class="us-two">
      <div class="us-list">
        <label class="us-label" for="us-urls">{{ t('mode.library.field_urls') }}</label>
        <div class="us-paste">
          <ol ref="gutter" class="us-gutter" aria-hidden="true">
            <!-- aura-allow-text: DỮ LIỆU (số dòng của ô dán, không phải câu văn). -->
            <li v-for="n in lineNumbers" :key="n">{{ n }}</li>
          </ol>
          <textarea
            id="us-urls"
            ref="textarea"
            v-model="pastedUrls"
            class="us-textarea"
            wrap="off"
            rows="12"
            autocomplete="off"
            spellcheck="false"
            placeholder="https://…"
            @scroll="onTextareaScroll"
          ></textarea>
        </div>
        <p class="us-count" role="status">
          {{
            t('mode.library.url_screen_count', {
              links: String(pastedUrlCount),
              chapters: String(pastedUrlCount),
              domains: String(pastedUrlDomainCount),
            })
          }}
        </p>
        <p class="us-promise">{{ t('mode.library.url_screen_promise', { count: String(pastedUrlCount) }) }}</p>
      </div>

      <form class="us-side" @submit.prevent>
        <fieldset class="us-group" data-import-destination>
          <legend>{{ t('mode.library.field_destination') }}</legend>
          <label class="us-radio">
            <input
              type="radio"
              name="us-destination-mode"
              :checked="destinationMode === 'new'"
              @change="setDestinationMode('new')"
            />
            <span class="us-radio-copy">
              <span class="us-radio-title">{{ t('mode.library.destination_new_work') }}</span>
              <span class="us-radio-desc">{{ t('mode.library.destination_new_work_desc') }}</span>
            </span>
          </label>
          <label class="us-radio">
            <input
              type="radio"
              name="us-destination-mode"
              :checked="destinationMode === 'existing'"
              :disabled="!destinationExistingWorkAvailable"
              @change="setDestinationMode('existing')"
            />
            <span class="us-radio-copy">
              <span class="us-radio-title">{{ t('mode.library.destination_existing_work') }}</span>
              <span class="us-radio-desc">{{ t('mode.library.destination_existing_work_desc') }}</span>
            </span>
          </label>
          <p v-if="destinationLibraryGenuinelyEmpty" class="us-hint" role="status">
            {{ t('mode.library.destination_empty_library') }}
          </p>
          <p v-if="destinationWorksError !== null" class="us-hint us-error" role="alert">
            <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
            {{ tError(destinationWorksError) }}
          </p>
          <label v-if="destinationMode === 'existing'" class="us-field">
            <span>{{ t('mode.library.destination_pick_label') }}</span>
            <select
              :value="destinationWorkId ?? ''"
              @change="setDestinationWorkId(($event.target as HTMLSelectElement).value)"
            >
              <option value="" disabled>{{ t('mode.library.destination_pick_placeholder') }}</option>
              <!-- aura-allow-text: DỮ LIỆU (tên Tác phẩm người dùng tự đặt). -->
              <option v-for="w in destinationWorks" :key="w.work_id" :value="w.work_id">{{ w.name }}</option>
            </select>
          </label>
        </fieldset>

        <label class="us-field">
          <span>{{ t('mode.library.field_name') }}</span>
          <input v-if="pickedDestinationWork === null" v-model="name" type="text" autocomplete="off" />
          <input v-else :value="pickedDestinationWork.name" type="text" autocomplete="off" disabled />
        </label>

        <label class="us-field">
          <span>{{ t('mode.library.field_source_lang') }}</span>
          <select v-if="pickedDestinationWork === null" v-model="sourceLang">
            <option value="zh">{{ t('mode.library.lang_zh') }}</option>
            <option value="en">{{ t('mode.library.lang_en') }}</option>
          </select>
          <input v-else :value="pickedDestinationWork.source_lang" type="text" autocomplete="off" disabled />
        </label>
      </form>
    </div>

    <p v-if="lastError !== null" class="us-error" role="alert">
      <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
      {{ tError(lastError) }}
    </p>

    <div class="us-actions">
      <button type="button" class="us-btn" :disabled="busy" @click="dispatch('library.import_urls_close')">
        {{ t('command.library.import_urls_close') }}
      </button>
      <button
        type="button"
        class="us-btn us-btn-primary"
        data-import-preview-open
        :disabled="busy || pastedUrlCount === 0 || destinationPending || effectiveName.trim() === ''"
        @pointerdown="recordImportOpener"
        @mousedown="recordImportOpener"
        @focus="recordImportOpener"
        @click="dispatch('library.import_urls')"
      >
        <!-- aura-allow-text: KẾT QUẢ của `t()` ở cả hai nhánh. -->
        {{ submitLabel }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.us {
  width: 100%;
}

.us-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.us-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.us-note,
.us-hint,
.us-count,
.us-promise {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.us-two {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 300px;
  gap: calc(var(--space-unit) * 5);
  align-items: start;
}

.us-list {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
  min-width: 0;
}

.us-label,
.us-field,
.us-group legend {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.us-paste {
  display: flex;
  height: 260px;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  background: var(--color-surface);
}

.us-gutter {
  margin: 0;
  padding: 8px 8px;
  list-style: none;
  overflow: hidden;
  box-sizing: border-box;
  height: 100%;
  text-align: right;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface-variant);
  border-right: 1px solid var(--color-outline);
  background: var(--color-surface-sunken);
}

.us-textarea {
  flex: 1;
  min-width: 0;
  box-sizing: border-box;
  height: 100%;
  padding: 8px 8px;
  border: none;
  resize: none;
  overflow: auto;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
  background: var(--color-surface);
}

.us-count {
  color: var(--color-on-surface);
}

.us-side {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 3);
}

.us-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  border: none;
  margin: 0;
  padding: 0;
}

.us-group legend {
  padding: 0;
}

.us-radio {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  cursor: pointer;
}

.us-radio-copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.us-radio-title {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.us-radio-desc {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.us-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.us-field input,
.us-field select {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 8px 8px;
}

.us-error {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-error);
}

.us-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: calc(var(--space-unit) * 2);
  margin-top: var(--space-panel-block);
}

.us-btn {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: none;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 8px 12px;
  cursor: pointer;
}

.us-btn:disabled {
  color: var(--color-on-surface-variant);
  cursor: not-allowed;
}

.us-btn-primary {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: var(--color-on-primary);
}

.us-btn-primary:disabled {
  background: var(--color-surface-sunken);
  color: var(--color-on-surface-variant);
}
</style>
