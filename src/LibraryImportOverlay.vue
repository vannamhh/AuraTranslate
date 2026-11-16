<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { recordImportOpener } from './importOpener'
import { useSelectionSurface } from './panels/selectionContract'
import {
  aimImportFile,
  busy,
  destinationMode,
  destinationLibraryGenuinelyEmpty,
  destinationWorkId,
  destinationWorks,
  destinationWorksError,
  dragFileCount,
  genre,
  importActiveTab,
  importAutoName,
  importFileName,
  importFiles,
  importFilesLockKey,
  importSubmitReady,
  isDragOver,
  lastError,
  name,
  noticeKey,
  pastedText,
  pastedUrlCount,
  pastedUrlDomainCount,
  pastedUrls,
  pickedDestinationWork,
  setDestinationMode,
  setDestinationWorkId,
  sourceLang,
  destinationExistingWorkAvailable,
} from './modes/libraryImport'
import type { ImportTab } from './modes/libraryImport'
import type { ClassifiedImportFile } from './config/library'
import { bilingualImportPreviewIsOpen } from './bilingualImportPreviewState'
import SelectMenu from './SelectMenu.vue'
import SourceLangChips from './modes/SourceLangChips.vue'
import { genreChoices, genreLabel, sourceLangLabel } from './modes/libraryWorkCard'
import { importPreviewIsOpen } from './importPreviewState'
import { libraryImportOverlayIsOpen } from './modes/libraryImportOverlayState'

const FIRST_FIELD =
  'input:not([type="radio"]):not([type="checkbox"]):not([type="file"]):not([disabled]), textarea:not([disabled])'

let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
const destinationOptions = computed(() =>
  destinationWorks.value.map((work) => ({ value: work.work_id, label: work.name })),
)
const genreOptions = computed(() => genreChoices())

const nameIsAuto = computed(() => name.value.trim() === '' && importAutoName.value !== '')
const nameFieldValue = computed(() => (name.value.trim() === '' ? importAutoName.value : name.value))
const nameHintKey = computed<string | null>(() => {
  if (!nameIsAuto.value) return null
  return importActiveTab.value === 'paste' ? 'mode.library.name_auto_hint_paste' : 'mode.library.name_auto_hint_files'
})

function onNameInput(event: Event): void {
  const field = event.target as HTMLInputElement
  name.value = field.value
  if (name.value.trim() === '') void nextTick(() => (field.value = nameFieldValue.value))
}

const TABS: readonly ImportTab[] = ['paste', 'files', 'urls']

const urlLineNumbers = computed<number[]>(() => {
  const count = Math.max(1, pastedUrls.value.split('\n').length)
  return Array.from({ length: count }, (_, i) => i + 1)
})
const urlGutter = useTemplateRef<HTMLElement>('urlGutter')
const urlTextarea = useTemplateRef<HTMLTextAreaElement>('urlTextarea')

function onUrlScroll(): void {
  if (urlGutter.value !== null && urlTextarea.value !== null) urlGutter.value.scrollTop = urlTextarea.value.scrollTop
}

const submitLabel = computed<string>(() =>
  importActiveTab.value === 'urls'
    ? t(
        destinationMode.value === 'existing' ? 'mode.library.url_screen_submit_append' : 'mode.library.url_screen_submit',
        { count: String(pastedUrlCount.value) },
      )
    : t('mode.library.import_preview_action'),
)

const submitChord = computed<string>(() =>
  document.documentElement.dataset.platform === 'mac'
    ? t('mode.library.import_submit_chord_mac')
    : t('mode.library.import_submit_chord_other'),
)

function fileKindKey(file: ClassifiedImportFile): string {
  if (file.problem === 'unsupported') return 'mode.library.file_kind_unsupported'
  if (file.problem === 'kind_mismatch') return 'mode.library.file_kind_bilingual_mismatch'
  if (file.problem === 'one_bilingual_only') return 'mode.library.file_kind_bilingual_extra'
  if (file.kind === 'docx') return 'mode.library.file_kind_docx'
  return file.kind === 'bilingual' ? 'mode.library.file_kind_bilingual' : 'mode.library.file_kind_text'
}

function aimFileRow(event: Event): void {
  const row = (event.currentTarget as HTMLElement).closest<HTMLElement>('[data-import-file-path]')
  if (row?.dataset.importFilePath !== undefined) aimImportFile(row.dataset.importFilePath)
}

function onSubmitChord(event: KeyboardEvent): void {
  if (!(event.metaKey || event.ctrlKey)) return
  event.preventDefault()
  if (importSubmitReady.value) dispatch('library.import.submit')
}

function onTabKeydown(event: KeyboardEvent): void {
  const step = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0
  if (step === 0) return
  event.preventDefault()
  const next = TABS[(TABS.indexOf(importActiveTab.value) + step + TABS.length) % TABS.length]
  if (next === 'paste') dispatch('library.import.tab_paste')
  else if (next === 'files') dispatch('library.import.tab_files')
  else dispatch('library.import.tab_urls')
  void nextTick(() => panel.value?.querySelector<HTMLElement>(`[data-import-tab="${next}"]`)?.focus())
}

useSelectionSurface(panel, 'display')

// A removed row unmounts the focused Bỏ button and focus falls to body, outside the keydown scope.
watch(importFiles, () => {
  void nextTick(() => {
    const root = panel.value
    if (root !== null && libraryImportOverlayIsOpen.value && !root.contains(document.activeElement)) root.focus()
  })
})

watch(libraryImportOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-library-import-open]')
    if (returnFocusTo === null || returnFocusTo === document.body || returnFocusTo.matches('section.mode')) {
      returnFocusTo = document.querySelector<HTMLElement>('[data-library-import-open]') ?? returnFocusTo
    }
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
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), summary:not([aria-disabled="true"]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  ).filter(
    (el) => el.getAttribute('tabindex') !== '-1' && !insideClosedMenu(el) && !isUncheckedRadioOfCheckedGroup(root, el),
  )
}

// focus() on anything inside a closed <details> is a no-op in WebKit, so the trap would stall there.
function insideClosedMenu(el: HTMLElement): boolean {
  const details = el.closest('details')
  return details !== null && !details.open && el.tagName !== 'SUMMARY'
}

function isUncheckedRadioOfCheckedGroup(root: HTMLElement, el: HTMLElement): boolean {
  if (!(el instanceof HTMLInputElement) || el.type !== 'radio' || el.checked || el.name === '') return false
  return Array.from(root.querySelectorAll<HTMLInputElement>('input[type="radio"]')).some(
    (other) => other.name === el.name && other.checked,
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
    v-show="!(importPreviewIsOpen || bilingualImportPreviewIsOpen)"
    class="li-scrim"
    @mousedown.self.prevent
    @keydown.esc="dispatch('library.import.close')"
    @keydown.enter="onSubmitChord($event)"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="li-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="li-title">
      <header class="li-head">
        <h2 id="li-title" class="li-title">{{ t('mode.library.import_title') }}</h2>
        <button type="button" class="li-close ui-btn ui-btn--quiet" :disabled="busy" @click="dispatch('library.import.close')">
          {{ t('command.library.import.close') }}
        </button>
      </header>

      <form class="import-form" @submit.prevent>
        <fieldset class="field field-group" data-import-destination>
          <legend>{{ t('mode.library.field_destination') }}</legend>

          <label class="radio-field ui-check">
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

          <label class="radio-field ui-check">
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

          <div v-if="destinationMode === 'existing'" class="field" data-import-destination-picker>
            <span>{{ t('mode.library.destination_pick_label') }}</span>
            <SelectMenu
              variant="field"
              :model-value="destinationWorkId ?? ''"
              :options="destinationOptions"
              :label="t('mode.library.destination_pick_label')"
              :placeholder="t('mode.library.destination_pick_placeholder')"
              @change="setDestinationWorkId($event)"
            />
          </div>
        </fieldset>

        <label class="field">
          <span>{{ t('mode.library.field_name') }}</span>
          <input
            v-if="pickedDestinationWork === null"
            :value="nameFieldValue"
            class="ui-field ui-field--lg ui-field--full"
            :data-name-auto="nameIsAuto"
            :placeholder="importActiveTab === 'urls' ? t('mode.library.name_url_placeholder') : undefined"
            data-import-name
            type="text"
            autocomplete="off"
            @input="onNameInput"
          />
          <input v-else :value="pickedDestinationWork.name" class="ui-field ui-field--lg ui-field--full" type="text" autocomplete="off" disabled />
          <span v-if="nameHintKey !== null && pickedDestinationWork === null" class="hint" data-import-name-hint>{{ t(nameHintKey) }}</span>
        </label>

        <div class="field">
          <span>{{ t('mode.library.field_source_lang') }}</span>
          <SourceLangChips
            v-if="pickedDestinationWork === null"
            v-model="sourceLang"
            name="library-import-source-lang"
            data-import-source-lang
          />
          <input
            v-else
            class="ui-field ui-field--lg ui-field--full"
            :value="sourceLangLabel(pickedDestinationWork.source_lang) ?? ''"
            type="text"
            autocomplete="off"
            disabled
          />
        </div>

        <div class="field">
          <span>{{ t('mode.library.field_genre') }}</span>
          <SelectMenu
            v-if="pickedDestinationWork === null"
            v-model="genre"
            variant="field"
            :columns="3"
            :options="genreOptions"
            :label="t('mode.library.field_genre')"
            data-import-genre
          />
          <input
            v-else
            class="ui-field ui-field--lg ui-field--full"
            :value="genreLabel(pickedDestinationWork.genre) ?? ''"
            type="text"
            autocomplete="off"
            disabled
          />
        </div>

        <div class="li-tabs" role="tablist" :aria-label="t('mode.library.tabs_label')">
          <button
            type="button"
            role="tab"
            class="ui-tab"
            data-import-tab="paste"
            :aria-selected="importActiveTab === 'paste'"
            :aria-controls="importActiveTab === 'paste' ? 'li-tabpanel-paste' : undefined"
            :tabindex="importActiveTab === 'paste' ? 0 : -1"
            @click="dispatch('library.import.tab_paste')"
            @keydown="onTabKeydown"
          >
            {{ t('command.library.import.tab_paste') }}
          </button>
          <button
            type="button"
            role="tab"
            class="ui-tab"
            data-import-tab="files"
            :aria-selected="importActiveTab === 'files'"
            :aria-controls="importActiveTab === 'files' ? 'li-tabpanel-files' : undefined"
            :tabindex="importActiveTab === 'files' ? 0 : -1"
            @click="dispatch('library.import.tab_files')"
            @keydown="onTabKeydown"
          >
            {{ t('command.library.import.tab_files') }}
          </button>
          <button
            type="button"
            role="tab"
            class="ui-tab"
            data-import-tab="urls"
            :aria-selected="importActiveTab === 'urls'"
            :aria-controls="importActiveTab === 'urls' ? 'li-tabpanel-urls' : undefined"
            :tabindex="importActiveTab === 'urls' ? 0 : -1"
            @click="dispatch('library.import.tab_urls')"
            @keydown="onTabKeydown"
          >
            {{ t('command.library.import.tab_urls') }}
          </button>
        </div>

        <div v-if="importActiveTab === 'paste'" id="li-tabpanel-paste" class="li-tabpanel" role="tabpanel" data-import-tabpanel="paste">
          <label class="field">
            <span>{{ t('mode.library.field_paste') }}</span>
            <textarea v-model="pastedText" class="ui-field ui-field--full" rows="10"></textarea>
          </label>
        </div>

        <div v-else-if="importActiveTab === 'files'" id="li-tabpanel-files" class="li-tabpanel" role="tabpanel" data-import-tabpanel="files">
          <ul v-if="importFiles.length > 0" class="li-files" data-import-files>
            <li
              v-for="file in importFiles"
              :key="file.path"
              class="li-file"
              :class="{ 'li-file--bad': file.problem !== null }"
              :data-import-file-path="file.path"
              :data-import-file-kind="file.kind"
              :data-import-file-problem="file.problem ?? ''"
            >
              <!-- aura-allow-text: tên tệp là dữ liệu người dùng chọn, không phải chuỗi giao diện. -->
              <span class="li-file-name" :title="file.path">{{ importFileName(file.path) }}</span>
              <!-- aura-allow-text: qua t(). -->
              <span class="li-file-kind">{{ t(fileKindKey(file)) }}</span>
              <button
                type="button"
                class="li-file-remove ui-btn ui-btn--quiet"
                data-import-file-remove
                :disabled="busy"
                @mousedown="aimFileRow"
                @focus="aimFileRow"
                @click="dispatch('library.import.file_remove')"
              >
                {{ t('command.library.import.file_remove') }}
              </button>
            </li>
          </ul>
          <div class="li-droprow">
            <button type="button" class="ui-btn ui-btn--lg" data-import-pick-files :disabled="busy" @click="dispatch('library.import.pick_files')">
              {{ t('mode.library.pick_file') }}
            </button>
            <span class="li-droptext">{{ t('mode.library.dropzone') }}</span>
          </div>
          <!-- aura-allow-text: qua t(). -->
          <p v-if="importFilesLockKey !== null" class="li-lock" role="alert" data-import-files-lock>{{ t(importFilesLockKey) }}</p>
        </div>

        <div v-else id="li-tabpanel-urls" class="li-tabpanel" role="tabpanel" data-import-tabpanel="urls">
          <p class="li-note">{{ t('mode.library.url_screen_subtitle') }}</p>
          <label class="field">
            <span>{{ t('mode.library.field_urls') }}</span>
            <div class="li-urls">
              <ol ref="urlGutter" class="li-url-gutter" aria-hidden="true">
                <!-- aura-allow-text: DỮ LIỆU (số dòng của ô dán, không phải câu văn). -->
                <li v-for="n in urlLineNumbers" :key="n">{{ n }}</li>
              </ol>
              <textarea
                ref="urlTextarea"
                v-model="pastedUrls"
                class="li-url-textarea ui-field ui-field--full"
                wrap="off"
                rows="10"
                autocomplete="off"
                spellcheck="false"
                placeholder="https://…"
                @scroll="onUrlScroll"
              ></textarea>
            </div>
          </label>
          <p class="li-count" role="status" data-import-url-count>
            {{
              t('mode.library.url_screen_count', {
                links: String(pastedUrlCount),
                chapters: String(pastedUrlCount),
                domains: String(pastedUrlDomainCount),
              })
            }}
          </p>
          <p class="li-note">{{ t('mode.library.url_screen_promise', { count: String(pastedUrlCount) }) }}</p>
        </div>
      </form>

      <!-- aura-allow-text: qua t(). -->
      <p class="notice" role="status">{{ noticeKey ? t(noticeKey) : '' }}</p>
      <!-- aura-allow-text: qua tError(). -->
      <p class="error" role="status">{{ lastError ? tError(lastError) : '' }}</p>

      <footer class="li-foot">
        <span class="li-dest">
          <!-- aura-allow-text: tên Tác phẩm đích là dữ liệu; câu còn lại qua t(). -->
          {{
            pickedDestinationWork === null
              ? t('mode.library.footer_dest_new')
              : t('mode.library.footer_dest_existing', { name: pickedDestinationWork.name })
          }}
        </span>
        <div class="li-foot-actions">
          <button type="button" class="ui-btn ui-btn--lg" data-import-cancel :disabled="busy" @click="dispatch('library.import.close')">
            {{ t('command.library.import.cancel') }}
          </button>
          <button
            type="button"
            class="btn li-primary ui-btn ui-btn--lg ui-btn--primary"
            data-import-preview-open
            :disabled="!importSubmitReady"
            @pointerdown="recordImportOpener"
            @mousedown="recordImportOpener"
            @focus="recordImportOpener"
            @click="dispatch('library.import.submit')"
          >
            <!-- aura-allow-text: qua t(). -->
            {{ submitLabel }}
            <!-- aura-allow-text: qua t() trong `submitChord`. -->
            <span class="li-chord">{{ submitChord }}</span>
          </button>
        </div>
      </footer>

      <div v-if="isDragOver" class="li-dropveil" aria-hidden="true" data-import-drop-frame>
        <!-- aura-allow-text: qua t(). -->
        {{ dragFileCount > 0 ? t('mode.library.drop_frame', { count: String(dragFileCount) }) : t('mode.library.drop_frame_unknown') }}
      </div>
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

:root[data-platform='mac'] .li-scrim {
  padding-top: var(--space-titlebar-height);
}

.li-panel {
  position: relative;
  width: 100%;
  max-width: 721px;
  max-height: 100%;
  overflow: auto;
  padding: 13px; /* aura-allow-spacing: 1px border sits inside the 14px inset */
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.li-panel:focus-visible {
  outline: none;
  border-color: var(--color-primary);
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

.import-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 7px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pick-row {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
}

.pick-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.field-group {
  gap: 7px;
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
  gap: 7px;
  cursor: pointer;
}

.radio-copy {
  display: flex;
  flex-direction: column;
  gap: 7px;
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
}

.li-tabs {
  display: flex;
  gap: calc(var(--space-unit) * 3);
  border-bottom: 1px solid var(--color-outline);
}

.li-tabs .ui-tab {
  padding: calc(var(--space-unit) * 1) 0;
  margin-bottom: -1px; /* aura-allow-spacing: overlaps the 1px rule under the tab row */
}

.li-tabpanel {
  display: flex;
  flex-direction: column;
  gap: 7px;
}

.li-files {
  margin: 0;
  padding: 0;
  list-style: none;
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-default);
}

.li-file {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 7px 14px;
  border-bottom: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.li-file:last-child {
  border-bottom: none;
}

.li-file--bad {
  background: var(--color-surface-sunken);
}

.li-file-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface);
}

.li-file-kind {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.li-file--bad .li-file-kind {
  color: var(--color-error);
  font-weight: var(--weight-ui-md-strong);
}

.li-file-remove {
  margin-left: auto;
}

.li-droprow {
  display: flex;
  align-items: center;
  gap: 14px;
}

.li-droptext,
.li-note,
.li-count {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.li-count {
  color: var(--color-on-surface);
}

.li-lock {
  margin: 0;
  padding: 7px 14px;
  border-left: 2px solid var(--color-error);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.li-urls {
  display: flex;
  height: 245px;
}

.li-url-gutter {
  margin: 0;
  padding: 7px; /* aura-allow-spacing: 1px border + 6px padding of textarea.ui-field so numbers meet the text lines */
  list-style: none;
  overflow: hidden;
  box-sizing: border-box;
  height: 100%;
  text-align: right;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
  border-right: 1px solid var(--color-outline);
}

.li-url-textarea {
  flex: 1;
  min-width: 0;
  resize: none;
  overflow: auto;
}

.li-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  margin-top: 14px;
}

.li-foot-actions {
  display: flex;
  align-items: center;
  gap: 14px;
}

.li-dest {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.li-chord {
  margin-left: 7px;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  font-weight: normal;
}

.li-dropveil {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px dashed var(--color-primary);
  background: var(--color-surface);
  color: var(--color-primary);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  pointer-events: none;
}

.notice {
  margin: 7px 0 0;
  min-height: 21px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.error {
  margin: 14px 0 0;
  min-height: 21px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}
</style>
