<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import {
  exportChapters,
  exportChoosingFolder,
  exportCounts,
  exportCountsError,
  exportCountsStatus,
  exportFolder,
  exportFolderError,
  exportFolderUnavailable,
  exportFormat,
  exportImageMode,
  exportLinkUsable,
  exportLoadError,
  exportLoadStatus,
  exportOverlayIsOpen,
  exportRunError,
  exportRunResult,
  exportRunStatus,
  exportScopeKind,
  exportSelectedChapterIds,
  exportSingleChapterId,
  selectExportSingleChapter,
  setExportFormat,
  setExportImageMode,
  setExportScopeKind,
  toggleExportChapter,
} from './exportState'
import type { ExportScopeKind } from './exportState'
import type { ChapterRow } from './config/chapter'
import type { MissingLinkImage } from './config/export'

let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

const scopeChoices: readonly { kind: ExportScopeKind; labelKey: string }[] = [
  { kind: 'chapter', labelKey: 'export.scope.chapter' },
  { kind: 'chapters', labelKey: 'export.scope.chapters' },
  { kind: 'work', labelKey: 'export.scope.work' },
]

const listVisible = computed(() => exportScopeKind.value !== 'work')

const canRun = computed(
  () => exportFolder.value !== null && exportCountsStatus.value === 'loaded' && exportRunStatus.value !== 'running',
)

function chapterLabel(row: ChapterRow): string {
  return row.title === null ? t('mode.library.chapter_untitled', { ord: String(row.ord) }) : row.title
}

function missingLabel(item: MissingLinkImage): string {
  const chapter = item.chapter_title ?? t('mode.library.chapter_untitled', { ord: String(item.chapter_ord) })
  const index = String(item.image_index)
  return item.alt_text === null || item.alt_text === ''
    ? t('export.images.missing_item', { chapter, index })
    : t('export.images.missing_item_alt', { chapter, index, alt: item.alt_text })
}

watch(exportOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-export-open]')
    void nextTick(() => panel.value?.focus())
    return
  }
  const back = returnFocusTo
  returnFocusTo = null
  if (back !== null && back.isConnected) {
    back.focus()
    return
  }
  const opener = document.querySelector<HTMLElement>('[data-export-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[export] focus-return target is gone; focus falls back to body.')
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
  <div v-if="exportOverlayIsOpen" class="ex-scrim" @keydown.esc="dispatch('export.close')" @keydown.tab="trapTab($event)">
    <section ref="panel" class="ex-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="ex-title">
      <header class="ex-head">
        <h2 id="ex-title" class="ex-title">{{ t('export.title') }}</h2>
        <button type="button" class="ex-close" @click="dispatch('export.close')">
          {{ t('command.export.close') }}
        </button>
      </header>

      <p v-if="exportLoadStatus === 'unknown'" class="ex-status" role="status">{{ t('export.loading') }}</p>
      <p v-else-if="exportLoadStatus === 'ipc_unavailable'" class="ex-empty">{{ t('export.empty_ipc_unavailable') }}</p>
      <p v-else-if="exportLoadStatus === 'error' && exportLoadError !== null" class="ex-empty ex-error" role="alert">
        <!-- aura-allow-text: result of tError() on the IPC error. -->
        {{ tError(exportLoadError) }}
      </p>
      <template v-else>
        <fieldset class="ex-scope">
          <legend class="ex-legend">{{ t('export.scope.legend') }}</legend>
          <label v-for="choice in scopeChoices" :key="choice.kind" class="ex-choice">
            <input
              type="radio"
              name="export-scope"
              :checked="exportScopeKind === choice.kind"
              @change="setExportScopeKind(choice.kind)"
            />
            <span>{{ t(choice.labelKey) }}</span>
          </label>
        </fieldset>

        <template v-if="listVisible">
          <p v-if="exportChapters.length === 0" class="ex-empty">{{ t('export.scope.no_chapters') }}</p>
          <fieldset v-else class="ex-chapters">
            <legend class="ex-legend">{{ t('export.scope.chapter_list_label') }}</legend>
            <template v-if="exportScopeKind === 'chapter'">
              <label v-for="row in exportChapters" :key="row.chapter_id" class="ex-choice">
                <input
                  type="radio"
                  name="export-chapter"
                  :checked="exportSingleChapterId === row.chapter_id"
                  @change="selectExportSingleChapter(row.chapter_id)"
                />
                <!-- aura-allow-text: data (Chapter title or the numbered label). -->
                <span>{{ chapterLabel(row) }}</span>
              </label>
            </template>
            <template v-else>
              <label v-for="row in exportChapters" :key="row.chapter_id" class="ex-choice">
                <input
                  type="checkbox"
                  :checked="exportSelectedChapterIds.includes(row.chapter_id)"
                  @change="toggleExportChapter(row.chapter_id)"
                />
                <!-- aura-allow-text: data (Chapter title or the numbered label). -->
                <span>{{ chapterLabel(row) }}</span>
              </label>
            </template>
          </fieldset>
        </template>

        <div class="ex-counts" data-export-counts>
          <p v-if="exportCountsStatus === 'unknown'" class="ex-status" role="status">{{ t('export.counts.loading') }}</p>
          <p v-else-if="exportCountsStatus === 'none_selected'" class="ex-empty" role="status">
            {{ t('export.counts.none_selected') }}
          </p>
          <p v-else-if="exportCountsStatus === 'ipc_unavailable'" class="ex-empty" role="status">
            {{ t('export.counts.ipc_unavailable') }}
          </p>
          <p v-else-if="exportCountsStatus === 'error' && exportCountsError !== null" class="ex-empty ex-error" role="alert">
            <!-- aura-allow-text: result of tError() on the IPC error. -->
            {{ tError(exportCountsError) }}
          </p>
          <template v-else-if="exportCounts !== null">
            <ul class="ex-count-list">
              <!-- aura-allow-text: result of t() with the count interpolated. -->
              <li>{{ t('export.counts.chapters', { count: String(exportCounts.chapter_count) }) }}</li>
              <!-- aura-allow-text: result of t() with the count interpolated. -->
              <li>{{ t('export.counts.segments', { count: String(exportCounts.segment_count) }) }}</li>
            </ul>
            <p
              v-if="exportFormat === 'docx_two_column' && exportCounts.unconfirmed_count > 0"
              class="ex-warning" role="alert" data-export-unconfirmed>
              <!-- aura-allow-text: result of t() with the count interpolated. -->
              {{ t('export.unconfirmed_warning', { count: String(exportCounts.unconfirmed_count) }) }}
            </p>
            <template v-if="exportFormat === 'docx_one_block'">
              <p v-if="exportCounts.unconfirmed_translated_count > 0" class="ex-warning" role="alert" data-export-block-unconfirmed>
                <!-- aura-allow-text: result of t() with the count interpolated. -->
                {{ t('export.block.unconfirmed_warning', { count: String(exportCounts.unconfirmed_translated_count) }) }}
              </p>
              <p v-if="exportCounts.untranslated_count > 0" class="ex-warning" role="alert" data-export-block-untranslated>
                <!-- aura-allow-text: result of t() with the count interpolated. -->
                {{ t('export.block.untranslated_warning', { count: String(exportCounts.untranslated_count) }) }}
              </p>
            </template>
          </template>
        </div>

        <fieldset class="ex-format">
          <legend class="ex-legend">{{ t('export.format.legend') }}</legend>
          <label class="ex-choice">
            <input
              type="radio"
              name="export-format"
              :checked="exportFormat === 'docx_two_column'"
              @change="setExportFormat('docx_two_column')"
            />
            <span>{{ t('export.format.docx_two_column') }}</span>
          </label>
          <p v-if="exportFormat === 'docx_two_column'" class="ex-note" data-export-reimportable>
            {{ t('export.format.reimportable') }}
          </p>
          <label class="ex-choice">
            <input
              type="radio"
              name="export-format"
              :checked="exportFormat === 'docx_one_block'"
              @change="setExportFormat('docx_one_block')"
            />
            <span>{{ t('export.format.docx_one_block') }}</span>
          </label>
          <p v-if="exportFormat === 'docx_one_block'" class="ex-note" data-export-not-reimportable>
            {{ t('export.format.not_reimportable') }}
          </p>
        </fieldset>

        <fieldset v-if="exportCounts !== null" class="ex-format" data-export-images>
          <legend class="ex-legend">{{ t('export.images.legend') }}</legend>
          <!-- aura-allow-text: result of t() with the count interpolated. -->
          <p class="ex-note" data-export-image-count>{{ t('export.images.count', { count: String(exportCounts.image_count) }) }}</p>
          <label class="ex-choice">
            <input
              type="radio"
              name="export-image-mode"
              value="file"
              :checked="exportImageMode === 'file'"
              @change="setExportImageMode('file')"
            />
            <span>{{ t('export.images.file') }}</span>
          </label>
          <label class="ex-choice">
            <input
              type="radio"
              name="export-image-mode"
              value="link"
              :checked="exportImageMode === 'link'"
              :disabled="!exportLinkUsable()"
              @change="setExportImageMode('link')"
            />
            <span>{{ t('export.images.link') }}</span>
          </label>
          <p v-if="!exportLinkUsable()" class="ex-note" data-export-link-disabled>{{ t('export.images.link_disabled') }}</p>
          <template v-else-if="exportImageMode === 'link' && exportCounts.missing_link_images.length > 0">
            <!-- aura-allow-text: result of t() with the count interpolated. -->
            <p class="ex-warning" role="alert" data-export-missing-heading>
              {{ t('export.images.missing_heading', { count: String(exportCounts.missing_link_images.length) }) }}
            </p>
            <ul class="ex-count-list" data-export-missing-links>
              <!-- aura-allow-text: result of t() with chapter label and alt text interpolated. -->
              <li v-for="item in exportCounts.missing_link_images" :key="`${item.chapter_id}-${item.image_index}`">
                {{ missingLabel(item) }}
              </li>
            </ul>
          </template>
        </fieldset>

        <div class="ex-folder">
          <span class="ex-legend">{{ t('export.folder.label') }}</span>
          <!-- aura-allow-text: data (folder path chosen by the user). -->
          <span v-if="exportFolder !== null" class="ex-path" data-export-folder>{{ exportFolder }}</span>
          <span v-else class="ex-note">{{ t('export.folder.none') }}</span>
          <button type="button" class="ex-act" :disabled="exportChoosingFolder" @click="dispatch('export.choose_folder')">
            {{ t('command.export.choose_folder') }}
          </button>
        </div>
        <p v-if="exportFolderError !== null" class="ex-status ex-error" role="alert">
          <!-- aura-allow-text: result of tError() on the IPC error. -->
          {{ tError(exportFolderError) }}
        </p>
        <p v-else-if="exportFolderUnavailable" class="ex-status" role="status">{{ t('export.folder.unavailable') }}</p>

        <div class="ex-folder">
          <button type="button" class="ex-act" data-export-run :disabled="!canRun" @click="dispatch('export.run')">
            {{ t('command.export.run') }}
          </button>
          <span v-if="exportFolder === null" class="ex-note">{{ t('export.run.need_folder') }}</span>
        </div>
        <p v-if="exportRunStatus === 'running'" class="ex-status" role="status">{{ t('export.run.running') }}</p>
        <p v-else-if="exportRunStatus === 'ipc_unavailable'" class="ex-status" role="status">
          {{ t('export.run.ipc_unavailable') }}
        </p>
        <p v-else-if="exportRunStatus === 'error' && exportRunError !== null" class="ex-status ex-error" role="alert">
          <!-- aura-allow-text: result of tError() on the IPC error. -->
          {{ tError(exportRunError) }}
        </p>
        <p v-else-if="exportRunStatus === 'done' && exportRunResult !== null" class="ex-status" role="status">
          <!-- aura-allow-text: result of t() with the counts interpolated. -->
          {{
            t('export.run.done', {
              count: String(exportRunResult.segment_count),
              chapters: String(exportRunResult.chapter_count),
            })
          }}
          <!-- aura-allow-text: data (path of the file Rust wrote). -->
          <span class="ex-path" data-export-result>{{ exportRunResult.path }}</span>
        </p>
        <p v-if="exportRunStatus === 'done' && exportRunResult !== null" class="ex-status" role="status" data-export-image-result>
          <!-- aura-allow-text: result of t() with the counts interpolated. -->
          {{ t('export.run.images_written', { count: String(exportRunResult.image_count) }) }}
          <template v-if="exportRunResult.images_skipped_missing_link > 0">
            <!-- aura-allow-text: result of t() with the count interpolated. -->
            {{ t('export.run.images_skipped', { count: String(exportRunResult.images_skipped_missing_link) }) }}
          </template>
          <template v-if="exportRunResult.images_dir !== null">
            {{ t('export.run.images_dir') }}
            <!-- aura-allow-text: data (folder Rust wrote the image files into). -->
            <span class="ex-path" data-export-images-dir>{{ exportRunResult.images_dir }}</span>
          </template>
        </p>

        <section class="ex-preview" aria-labelledby="ex-preview-title">
          <h3 id="ex-preview-title" class="ex-legend">{{ t('export.preview.heading') }}</h3>
          <p class="ex-note">{{ t('export.preview.note') }}</p>
          <div class="ex-word" data-export-word-preview>
            <table class="ex-word-table">
              <tbody>
                <tr>
                  <td>{{ t('export.preview.source_sample') }}</td>
                  <td>{{ t('export.preview.target_sample') }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>
      </template>
    </section>
  </div>
</template>

<style scoped>
.ex-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: above the shell like the other overlays. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.ex-panel {
  width: 100%;
  max-width: 720px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.ex-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.ex-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.ex-close {
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

.ex-scope,
.ex-format,
.ex-chapters {
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
  border: none;
}

.ex-legend {
  padding: 0;
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-weight: var(--weight-ui-sm);
  color: var(--color-on-surface-variant);
}

.ex-choice {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.ex-status,
.ex-empty,
.ex-note {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ex-error {
  color: var(--color-error);
}

.ex-counts {
  margin: 0 0 var(--space-panel-block) 0;
}

.ex-count-list {
  list-style: none;
  padding: 0;
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.ex-warning {
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-error);
}

.ex-folder {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
  margin: 0 0 var(--space-panel-block) 0;
}

.ex-path {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
  overflow-wrap: anywhere;
}

.ex-act {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.ex-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.ex-word {
  padding: calc(var(--space-unit) * 4);
  background-color: var(--color-word-page);
  border: 1px solid var(--color-word-rule);
}

.ex-word-table {
  width: 100%;
  border-collapse: collapse;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-sm);
  color: var(--color-word-ink);
}

.ex-word-table td {
  border: 1px solid var(--color-word-rule);
  padding: calc(var(--space-unit) * 2);
  text-align: left;
  vertical-align: top;
}
</style>

