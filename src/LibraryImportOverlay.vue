<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { recordImportOpener } from './importOpener'
import { useSelectionSurface } from './panels/selectionContract'
import {
  bilingualFilePath,
  busy,
  destinationMode,
  destinationLibraryGenuinelyEmpty,
  destinationPending,
  destinationWorkId,
  destinationWorks,
  destinationWorksError,
  effectiveFilePaths,
  effectiveName,
  filePath,
  genre,
  isDragOver,
  lastError,
  name,
  noticeKey,
  onFilePathInput,
  pastedText,
  pickedDestinationWork,
  setDestinationMode,
  setDestinationWorkId,
  sourceLang,
  destinationExistingWorkAvailable,
} from './modes/libraryImport'
import { bilingualImportPreviewIsOpen } from './bilingualImportPreviewState'
import { importPreviewIsOpen } from './importPreviewState'
import { libraryImportOverlayIsOpen, libraryUrlImportScreenIsOpen } from './modes/libraryImportOverlayState'

const FIRST_FIELD =
  'input:not([type="radio"]):not([type="checkbox"]):not([type="file"]):not([disabled]), textarea:not([disabled])'

let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

watch(libraryImportOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-library-import-open]')
    void nextTick(() => {
      const first = panel.value?.querySelector<HTMLElement>(FIRST_FIELD)
      if (first !== null && first !== undefined) first.focus()
      else panel.value?.focus()
    })
    return
  }
  const back = returnFocusTo
  returnFocusTo = null
  if (back !== null && back.isConnected) {
    back.focus()
    return
  }
  const opener = document.querySelector<HTMLElement>('[data-library-import-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[library] import focus-return target is gone; focus falls back to body.')
})

function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
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
</script>

<template>
  <div
    v-if="libraryImportOverlayIsOpen"
    v-show="!(libraryUrlImportScreenIsOpen || importPreviewIsOpen || bilingualImportPreviewIsOpen)"
    class="li-scrim"
    @keydown.esc="dispatch('library.import.close')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="li-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="li-title">
      <header class="li-head">
        <h2 id="li-title" class="li-title">{{ t('mode.library.import_title') }}</h2>
        <button type="button" class="li-close" @click="dispatch('library.import.close')">
          {{ t('command.library.import.close') }}
        </button>
      </header>

      <form class="import-form" @submit.prevent>
        <fieldset class="field field-group" data-import-destination>
          <legend>{{ t('mode.library.field_destination') }}</legend>

          <label class="radio-field">
            <input
              type="radio"
              name="import-destination-mode"
              value="new"
              :checked="destinationMode === 'new'"
              @change="setDestinationMode('new')"
            />
            <span class="radio-copy">
              <span class="radio-title">{{ t('mode.library.destination_new_work') }}</span>
              <span class="radio-desc">{{ t('mode.library.destination_new_work_desc') }}</span>
            </span>
          </label>

          <label class="radio-field">
            <input
              type="radio"
              name="import-destination-mode"
              value="existing"
              :checked="destinationMode === 'existing'"
              :disabled="!destinationExistingWorkAvailable"
              @change="setDestinationMode('existing')"
            />
            <span class="radio-copy">
              <span class="radio-title">{{ t('mode.library.destination_existing_work') }}</span>
              <span class="radio-desc">{{ t('mode.library.destination_existing_work_desc') }}</span>
            </span>
          </label>

          <p v-if="destinationLibraryGenuinelyEmpty" class="hint" role="status">
            {{ t('mode.library.destination_empty_library') }}
          </p>
          <p v-if="destinationWorksError !== null" class="hint hint-error" role="alert">
            <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
            {{ tError(destinationWorksError) }}
          </p>

          <label v-if="destinationMode === 'existing'" class="field" data-import-destination-picker>
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

        <label class="field">
          <span>{{ t('mode.library.field_name') }}</span>
          <input v-if="pickedDestinationWork === null" v-model="name" type="text" autocomplete="off" />
          <input v-else :value="pickedDestinationWork.name" type="text" autocomplete="off" disabled />
        </label>

        <label class="field">
          <span>{{ t('mode.library.field_source_lang') }}</span>
          <select v-if="pickedDestinationWork === null" v-model="sourceLang">
            <option value="zh">{{ t('mode.library.lang_zh') }}</option>
            <option value="en">{{ t('mode.library.lang_en') }}</option>
          </select>
          <input v-else :value="pickedDestinationWork.source_lang" type="text" autocomplete="off" disabled />
        </label>

        <label class="field">
          <span>{{ t('mode.library.field_genre') }}</span>
          <input v-if="pickedDestinationWork === null" v-model="genre" type="text" autocomplete="off" />
          <input v-else :value="pickedDestinationWork.genre" type="text" autocomplete="off" disabled />
        </label>

        <label class="field">
          <span>{{ t('mode.library.field_paste') }}</span>
          <textarea v-model="pastedText" rows="6"></textarea>
        </label>
        <button
          type="button"
          class="btn"
          data-import-preview-open
          :disabled="busy || pastedText.trim() === '' || destinationPending || effectiveName.trim() === ''"
          @pointerdown="recordImportOpener"
          @mousedown="recordImportOpener"
          @focus="recordImportOpener"
          @click="dispatch('library.import_text')"
        >
          <!-- aura-allow-text: qua t() cả hai nhánh, mục B4 — Kiểm A2 không đọc tĩnh được toán
               tử ba ngôi. -->
          {{ destinationMode === 'existing' ? t('mode.library.submit_text_append') : t('mode.library.submit_text') }}
        </button>

        <!-- Indicator only: a drop fills the path field below, so it takes no tabindex. -->
        <div class="dropzone" :class="{ over: isDragOver }">
          {{ t('mode.library.dropzone') }}
        </div>

        <label class="field">
          <span>{{ t('mode.library.field_path') }}</span>
          <input
            :value="filePath"
            type="text"
            autocomplete="off"
            @input="onFilePathInput(($event.target as HTMLInputElement).value)"
          />
        </label>
        <button
          type="button"
          class="btn"
          data-import-preview-open
          :disabled="busy || effectiveFilePaths.length === 0 || destinationPending || effectiveName.trim() === ''"
          @pointerdown="recordImportOpener"
          @mousedown="recordImportOpener"
          @focus="recordImportOpener"
          @click="dispatch('library.import_file')"
        >
          <!-- aura-allow-text: qua t() cả hai nhánh, mục B4 — Kiểm A2 không đọc tĩnh được toán
               tử ba ngôi. -->
          {{ destinationMode === 'existing' ? t('mode.library.submit_file_append') : t('mode.library.submit_file') }}
        </button>

        <label class="field">
          <span>{{ t('mode.library.field_bilingual_path') }}</span>
          <input v-model="bilingualFilePath" type="text" autocomplete="off" />
        </label>
        <button
          type="button"
          class="btn"
          data-bilingual-import-preview-open
          :disabled="busy || bilingualFilePath.trim() === '' || name.trim() === ''"
          @pointerdown="recordImportOpener"
          @mousedown="recordImportOpener"
          @focus="recordImportOpener"
          @click="dispatch('library.import_bilingual')"
        >
          {{ t('mode.library.submit_bilingual') }}
        </button>

        <button
          type="button"
          class="btn"
          data-library-url-screen-open
          :disabled="busy"
          @pointerdown="recordImportOpener"
          @mousedown="recordImportOpener"
          @focus="recordImportOpener"
          @click="dispatch('library.import_urls_open')"
        >
          {{ t('mode.library.url_screen_open') }}
        </button>
      </form>

      <!-- aura-allow-text: qua t(). -->
      <p class="notice" role="status">{{ noticeKey ? t(noticeKey) : '' }}</p>
      <!-- aura-allow-text: qua tError(). -->
      <p class="error" role="status">{{ lastError ? tError(lastError) : '' }}</p>
    </section>
  </div>
</template>

<style scoped>
.li-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: above the shell like the other overlays. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.li-panel {
  width: 100%;
  max-width: 720px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.li-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.li-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.li-close {
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.import-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.field input,
.field select,
.field textarea {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 8px 8px;
}

.field-group {
  gap: 8px;
  border: none;
  margin: 0;
  padding: 0;
}

.field-group legend {
  padding: 0;
  margin: 0;
}

.radio-field {
  display: flex;
  flex-direction: row;
  align-items: flex-start;
  gap: 8px;
  cursor: pointer;
}

.radio-copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.radio-title {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.radio-desc {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.hint {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
  margin: 0;
}

.hint-error {
  color: var(--color-error);
}

.btn {
  align-self: flex-start;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-primary);
  background: var(--color-surface-sunken);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 8px 12px;
  cursor: pointer;
}

.btn:disabled {
  color: var(--color-on-surface-variant);
  cursor: not-allowed;
}

.dropzone {
  border: 1px dashed var(--color-outline);
  border-radius: 4px;
  padding: 16px;
  text-align: center;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  background: var(--color-surface-sunken);
}

.dropzone.over {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.notice {
  margin: 4px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.error {
  margin: 12px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}
</style>
