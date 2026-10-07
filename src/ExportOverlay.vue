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
  exportLoadError,
  exportLoadStatus,
  exportOverlayIsOpen,
  exportScopeKind,
  exportSelectedChapterIds,
  exportSingleChapterId,
  selectExportSingleChapter,
  setExportScopeKind,
  toggleExportChapter,
} from './exportState'
import type { ExportScopeKind } from './exportState'
import type { ChapterRow } from './config/chapter'

let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

const scopeChoices: readonly { kind: ExportScopeKind; labelKey: string }[] = [
  { kind: 'chapter', labelKey: 'export.scope.chapter' },
  { kind: 'chapters', labelKey: 'export.scope.chapters' },
  { kind: 'work', labelKey: 'export.scope.work' },
]

// Word mock: hard-coded paper colours on purpose, so the block shows what the reviewer sees in
// Word and not the app's paper tokens. Every pair is WCAG AA (text >= 4.5:1, rules >= 3:1).
/* aura-allow-literal: Word page white; ink on it is 17:1 */
const WORD_PAGE = '#ffffff'
/* aura-allow-literal: Word body ink on white, 17:1 */
const WORD_INK = '#1a1a1a'
/* aura-allow-literal: Word rules and page edge, 4.5:1 against white */
const WORD_RULE = '#767676'
/* aura-allow-literal: Word table header fill; ink on it is 13:1 */
const WORD_HEADER_FILL = '#e7e6e6'

const wordPageStyle = { backgroundColor: WORD_PAGE, border: `1px solid ${WORD_RULE}` }
const wordTableStyle = { color: WORD_INK }
const wordCellStyle = { border: `1px solid ${WORD_RULE}` }
const wordHeadStyle = { border: `1px solid ${WORD_RULE}`, backgroundColor: WORD_HEADER_FILL }

const listVisible = computed(() => exportScopeKind.value !== 'work')

function chapterLabel(row: ChapterRow): string {
  return row.title === null ? t('mode.library.chapter_untitled', { ord: String(row.ord) }) : row.title
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
            <p v-if="exportCounts.unconfirmed_count > 0" class="ex-warning" role="alert" data-export-unconfirmed>
              <!-- aura-allow-text: result of t() with the count interpolated. -->
              {{ t('export.unconfirmed_warning', { count: String(exportCounts.unconfirmed_count) }) }}
            </p>
          </template>
        </div>

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

        <section class="ex-preview" aria-labelledby="ex-preview-title">
          <h3 id="ex-preview-title" class="ex-legend">{{ t('export.preview.heading') }}</h3>
          <p class="ex-note">{{ t('export.preview.note') }}</p>
          <div class="ex-word" data-export-word-preview :style="wordPageStyle">
            <table class="ex-word-table" :style="wordTableStyle">
              <thead>
                <tr>
                  <th :style="wordHeadStyle">{{ t('export.preview.source_header') }}</th>
                  <th :style="wordHeadStyle">{{ t('export.preview.target_header') }}</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td :style="wordCellStyle">{{ t('export.preview.source_sample') }}</td>
                  <td :style="wordCellStyle">{{ t('export.preview.target_sample') }}</td>
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
}

.ex-word-table {
  width: 100%;
  border-collapse: collapse;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-sm);
}

.ex-word-table th,
.ex-word-table td {
  padding: calc(var(--space-unit) * 2);
  text-align: left;
  vertical-align: top;
}
</style>

