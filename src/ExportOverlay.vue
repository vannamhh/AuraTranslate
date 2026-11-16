<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import {
  exportChapters,
  exportChoosingDestination,
  exportCounts,
  exportCountsError,
  exportAttribution,
  exportCountsStatus,
  exportAppendixAvailable,
  exportFirstScopeChapter,
  exportGlossaryAppendix,
  exportCurlyQuotes,
  exportCurlyQuotesAvailable,
  exportMarkAvailable,
  exportMarkUnconfirmed,
  exportNotReimportable,
  exportNumberSegments,
  exportPreviewData,
  exportPreviewError,
  exportPreviewLoading,
  exportDestinationError,
  exportDestinationUnavailable,
  exportFormat,
  exportFormatIsForPublishing,
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
  exportTranslatorName,
  exportTranslatorNameError,
  selectExportSingleChapter,
  setExportAttribution,
  setExportFormat,
  setExportGlossaryAppendix,
  setExportCurlyQuotes,
  setExportMarkUnconfirmed,
  setExportNumberSegments,
  setExportImageMode,
  setExportScopeKind,
  toggleExportChapter,
} from './exportState'
import type { ExportFormat, ExportScopeKind } from './exportState'
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
  () => exportCountsStatus.value === 'loaded' && exportRunStatus.value !== 'running' && !exportChoosingDestination.value,
)

function chapterLabel(row: ChapterRow): string {
  return row.title === null ? t('mode.library.chapter_untitled', { ord: String(row.ord) }) : row.title
}

function chapterStatusLabel(status: string): string {
  switch (status) {
    case 'not_started':
      return t('export.chapter.status_not_started')
    case 'in_progress':
      return t('export.chapter.status_in_progress')
    case 'paused':
      return t('export.chapter.status_paused')
    case 'done':
      return t('export.chapter.status_done')
    default:
      return t('export.chapter.status_unknown')
  }
}

function chapterMeta(row: ChapterRow): string {
  return t('export.chapter.row_meta', { count: String(row.segment_count), status: chapterStatusLabel(row.status) })
}

const firstChapter = computed(() => exportFirstScopeChapter())

const emptyAttributionLabels = computed(() => {
  const row = firstChapter.value
  if (row === null) return []
  const fields = [
    { label: t('export.attribution.field_author'), value: row.origin_author },
    { label: t('export.attribution.field_site'), value: row.origin_site_name },
    { label: t('export.attribution.field_url'), value: row.origin_url },
    { label: t('export.attribution.field_published'), value: row.origin_published_at },
  ]
  return fields.filter((field) => field.value === null || field.value.trim() === '').map((field) => field.label)
})

const appendixEnabled = computed(() => exportAppendixAvailable())
const markEnabled = computed(() => exportMarkAvailable())
const curlyEnabled = computed(() => exportCurlyQuotesAvailable())
const notReimportable = computed(() => exportNotReimportable())

const fileExtension = computed(() => {
  switch (exportFormat.value) {
    case 'markdown':
      return '.md'
    case 'plain_text':
      return '.txt'
    default:
      return '.docx'
  }
})

const runLabel = computed(() =>
  t('export.run.button', { count: String(exportCounts.value?.chapter_count ?? 0), ext: fileExtension.value }),
)

const previewChapterLabel = computed(() => {
  const p = exportPreviewData.value
  if (p === null) return ''
  return p.chapter_title ?? t('mode.library.chapter_untitled', { ord: String(p.chapter_ord) })
})

const previewTitle = computed(() => {
  const label = previewChapterLabel.value
  if (label === '') return t('export.preview.heading')
  const wide = exportScopeKind.value !== 'chapter' && (exportCounts.value?.chapter_count ?? 2) > 1
  return t(wide ? 'export.preview.title_first' : 'export.preview.title', { chapter: label })
})

const formatCards: readonly { value: ExportFormat; nameKey: string; ext: string; descKey: string }[] = [
  { value: 'docx_two_column', nameKey: 'export.format.name.docx_two_column', ext: '.docx', descKey: 'export.format.desc.docx_two_column' },
  { value: 'docx_one_block', nameKey: 'export.format.name.docx_one_block', ext: '.docx', descKey: 'export.format.desc.docx_one_block' },
  { value: 'markdown', nameKey: 'export.format.name.markdown', ext: '.md', descKey: 'export.format.desc.markdown' },
  { value: 'plain_text', nameKey: 'export.format.name.plain_text', ext: '.txt', descKey: 'export.format.desc.plain_text' },
]

function cardNotReimportable(value: ExportFormat): boolean {
  return value === 'docx_two_column' && notReimportable.value
}

function cardTagKey(value: ExportFormat): string {
  if (value !== 'docx_two_column') return 'export.format.tag_publish'
  return notReimportable.value ? 'export.format.tag_not_reimportable' : 'export.format.tag_reimportable'
}

const appendixLines = computed(() => exportPreviewData.value?.appendix ?? [])

function runShortcut(event: KeyboardEvent): void {
  if (!(event.metaKey || event.ctrlKey)) return
  event.preventDefault()
  if (canRun.value) dispatch('export.run')
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
</script>

<template>
  <div v-if="exportOverlayIsOpen" class="ex-scrim" @mousedown.self.prevent @keydown.esc="dispatch('export.close')" @keydown.enter="runShortcut($event)" @keydown.tab="trapTab($event)">
    <section ref="panel" class="ex-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="ex-title">
      <header class="ex-head">
        <h2 id="ex-title" class="ex-title">{{ t('export.title') }}</h2>
        <button type="button" class="ex-close ui-btn ui-btn--quiet" @click="dispatch('export.close')">
          {{ t('command.export.close') }}
        </button>
      </header>

      <p v-if="exportLoadStatus === 'unknown'" class="ex-status ex-lone" role="status">{{ t('export.loading') }}</p>
      <p v-else-if="exportLoadStatus === 'ipc_unavailable'" class="ex-empty ex-lone">{{ t('export.empty_ipc_unavailable') }}</p>
      <p v-else-if="exportLoadStatus === 'error' && exportLoadError !== null" class="ex-empty ex-error ex-lone" role="alert">
        <!-- aura-allow-text: result of tError() on the IPC error. -->
        {{ tError(exportLoadError) }}
      </p>
      <template v-else>
        <div class="ex-body">
          <div class="ex-config" data-export-config>
            <fieldset class="ex-group">
              <legend class="ex-legend">{{ t('export.scope.legend') }}</legend>
              <div class="ui-seg ex-seg">
                <label v-for="choice in scopeChoices" :key="choice.kind" class="ui-seg__item ex-seg-item">
                  <input
                    type="radio"
                    class="ex-sr-only"
                    name="export-scope"
                    :checked="exportScopeKind === choice.kind"
                    @change="setExportScopeKind(choice.kind)"
                  />
                  <span>{{ t(choice.labelKey) }}</span>
                </label>
              </div>
              <template v-if="listVisible">
                <p v-if="exportChapters.length === 0" class="ex-empty">{{ t('export.scope.no_chapters') }}</p>
                <div v-else class="ex-chapters" data-export-chapter-list>
                  <template v-if="exportScopeKind === 'chapter'">
                    <label v-for="row in exportChapters" :key="row.chapter_id" class="ex-choice ui-check">
                      <input
                        type="radio"
                        name="export-chapter"
                        :checked="exportSingleChapterId === row.chapter_id"
                        @change="selectExportSingleChapter(row.chapter_id)"
                      />
                      <!-- aura-allow-text: data (Chapter title or the numbered label). -->
                      <span class="ex-grow">{{ chapterLabel(row) }}</span>
                      <!-- aura-allow-text: result of t() with count and status interpolated. -->
                      <span class="ex-meta" data-export-chapter-meta>{{ chapterMeta(row) }}</span>
                    </label>
                  </template>
                  <template v-else>
                    <label v-for="row in exportChapters" :key="row.chapter_id" class="ex-choice ui-check">
                      <input
                        type="checkbox"
                        :checked="exportSelectedChapterIds.includes(row.chapter_id)"
                        @change="toggleExportChapter(row.chapter_id)"
                      />
                      <!-- aura-allow-text: data (Chapter title or the numbered label). -->
                      <span class="ex-grow">{{ chapterLabel(row) }}</span>
                      <!-- aura-allow-text: result of t() with count and status interpolated. -->
                      <span class="ex-meta" data-export-chapter-meta>{{ chapterMeta(row) }}</span>
                    </label>
                  </template>
                </div>
              </template>
            </fieldset>

            <fieldset class="ex-group ex-formats">
              <legend class="ex-legend">{{ t('export.format.legend') }}</legend>
              <div v-for="card in formatCards" :key="card.value" class="ex-card" :data-export-format-card="card.value">
                <label class="ex-card-head ui-check">
                  <input
                    type="radio"
                    name="export-format"
                    :checked="exportFormat === card.value"
                    @change="setExportFormat(card.value)"
                  />
                  <span class="ex-grow">
                    {{ t(card.nameKey) }}
                    <!-- aura-allow-text: file extension literal. -->
                    <span class="ex-meta">{{ card.ext }}</span>
                  </span>
                  <span class="ex-tag" :class="{ 'ex-tag-bad': cardNotReimportable(card.value) }" :data-export-format-tag="card.value">
                    {{ t(cardTagKey(card.value)) }}
                  </span>
                </label>
                <template v-if="exportFormat === card.value">
                  <p
                    v-if="card.value === 'docx_two_column' && notReimportable"
                    class="ex-note"
                    role="status"
                    data-export-not-reimportable
                  >
                    {{ t('export.content.not_reimportable') }}
                  </p>
                  <p v-else class="ex-note" :data-export-reimportable="card.value === 'docx_two_column' ? '' : undefined" data-export-card-desc>
                    {{ t(card.descKey) }}
                  </p>
                </template>
              </div>
            </fieldset>

            <fieldset class="ex-group" data-export-content>
              <legend class="ex-legend">{{ t('export.content.legend') }}</legend>
              <div class="ex-opt">
                <label class="ex-choice ui-check">
                  <input
                    type="checkbox"
                    data-export-number-toggle
                    :checked="exportNumberSegments"
                    @change="setExportNumberSegments(($event.target as HTMLInputElement).checked)"
                  />
                  <span>{{ t('export.content.number_segments') }}</span>
                </label>
              </div>
              <div class="ex-opt">
                <label class="ex-choice ui-check">
                  <input
                    type="checkbox"
                    data-export-curly-quotes-toggle
                    :checked="curlyEnabled && exportCurlyQuotes"
                    :disabled="!curlyEnabled"
                    @change="setExportCurlyQuotes(($event.target as HTMLInputElement).checked)"
                  />
                  <span>{{ t('export.content.curly_quotes') }}</span>
                </label>
                <span v-if="!curlyEnabled" class="ex-why" data-export-curly-quotes-disabled>{{ t('export.content.curly_quotes_publish_only') }}</span>
              </div>
              <div class="ex-opt">
                <label class="ex-choice ui-check">
                  <input
                    type="checkbox"
                    data-export-appendix-toggle
                    :checked="appendixEnabled && exportGlossaryAppendix"
                    :disabled="!appendixEnabled"
                    @change="setExportGlossaryAppendix(($event.target as HTMLInputElement).checked)"
                  />
                  <span>{{ t('export.content.glossary_appendix') }}</span>
                </label>
                <span v-if="!appendixEnabled" class="ex-why" data-export-appendix-empty>{{ t('export.content.glossary_appendix_empty') }}</span>
                <span v-else-if="exportPreviewData !== null" class="ex-meta" data-export-appendix-count>
                  <!-- aura-allow-text: result of t() with the count interpolated. -->
                  {{ t('export.content.glossary_appendix_count', { count: String(exportPreviewData.appendix_term_count) }) }}
                </span>
              </div>
              <div class="ex-opt">
                <label class="ex-choice ui-check">
                  <input
                    type="checkbox"
                    data-export-mark-toggle
                    :checked="markEnabled && exportMarkUnconfirmed"
                    :disabled="!markEnabled"
                    @change="setExportMarkUnconfirmed(($event.target as HTMLInputElement).checked)"
                  />
                  <span>{{ t('export.content.mark_unconfirmed') }}</span>
                </label>
                <span v-if="!markEnabled" class="ex-why" data-export-mark-disabled>{{ t('export.content.mark_two_column_only') }}</span>
                <span v-else-if="exportCounts !== null" class="ex-meta" data-export-mark-count>
                  <!-- aura-allow-text: result of t() with the count interpolated. -->
                  {{ t('export.content.mark_count', { count: String(exportCounts.unconfirmed_count) }) }}
                </span>
              </div>
            </fieldset>

            <fieldset v-if="exportCounts !== null" class="ex-group" data-export-images>
              <legend class="ex-legend">
                {{ t('export.images.legend') }}
                <!-- aura-allow-text: result of t() with the count interpolated. -->
                <span class="ex-meta" data-export-image-count>{{ t('export.images.count', { count: String(exportCounts.image_count) }) }}</span>
              </legend>
              <div class="ex-opt">
                <div class="ui-seg ex-seg">
                  <label class="ui-seg__item ex-seg-item">
                    <input
                      type="radio"
                      class="ex-sr-only"
                      name="export-image-mode"
                      value="file"
                      :checked="exportImageMode === 'file'"
                      @change="setExportImageMode('file')"
                    />
                    <span>{{ t('export.images.file') }}</span>
                  </label>
                  <label class="ui-seg__item ex-seg-item">
                    <input
                      type="radio"
                      class="ex-sr-only"
                      name="export-image-mode"
                      value="link"
                      :checked="exportImageMode === 'link'"
                      :disabled="!exportLinkUsable()"
                      @change="setExportImageMode('link')"
                    />
                    <span>{{ t('export.images.link') }}</span>
                  </label>
                </div>
                <span v-if="!exportLinkUsable()" class="ex-why" data-export-link-disabled>{{ t('export.images.link_disabled') }}</span>
              </div>
              <template v-if="exportLinkUsable() && exportImageMode === 'link' && exportCounts.missing_link_images.length > 0">
                <!-- aura-allow-text: result of t() with the count interpolated. -->
                <p class="ex-warning" data-export-missing-heading>
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

            <div class="ex-group" data-export-attribution>
              <div class="ex-opt">
                <label class="ex-choice ui-check">
                  <input
                    type="checkbox"
                    data-export-attribution-toggle
                    :checked="exportAttribution"
                    @change="setExportAttribution(($event.target as HTMLInputElement).checked)"
                  />
                  <span>{{ t('export.attribution.toggle') }}</span>
                </label>
                <span v-if="!exportAttribution" class="ex-meta" data-export-attribution-default>{{ t('export.attribution.default_off') }}</span>
                <span v-else-if="emptyAttributionLabels.length > 0" class="ex-why" data-export-attribution-missing>
                  <!-- aura-allow-text: result of t() with the empty field labels interpolated. -->
                  {{ t('export.attribution.missing_fields', { fields: emptyAttributionLabels.join(', ') }) }}
                </span>
              </div>
              <template v-if="exportAttribution">
                <p v-if="exportTranslatorNameError !== null" class="ex-status ex-error" role="alert" data-export-translator-error>
                  <!-- aura-allow-text: result of tError() on the IPC error. -->
                  {{ tError(exportTranslatorNameError) }}
                </p>
                <p v-else-if="exportTranslatorName !== null" class="ex-note" data-export-translator>
                  {{ t('export.attribution.translator') }}
                  <!-- aura-allow-text: data (translator name set in Settings). -->
                  <strong>{{ exportTranslatorName }}</strong>
                  <span class="ex-badge">{{ t('export.attribution.global_badge') }}</span>
                </p>
                <p v-else class="ex-note" data-export-translator-missing>
                  {{ t('export.attribution.translator_missing') }}
                </p>
              </template>
            </div>
          </div>

          <section class="ex-preview" aria-labelledby="ex-preview-title">
            <header class="ex-preview-head">
              <h3 id="ex-preview-title" class="ex-preview-title" data-export-preview-chapter>
                <!-- aura-allow-text: result of t() with the Chapter label interpolated. -->
                {{ previewTitle }}
              </h3>
              <!-- aura-allow-text: data (file name Rust would write, before de-duplication). -->
              <span v-if="exportPreviewData !== null" class="ex-path" data-export-file-name>{{ exportPreviewData.file_name }}</span>
            </header>
            <p v-if="exportPreviewError !== null" class="ex-status ex-error" role="alert" data-export-preview-error>
              <!-- aura-allow-text: result of tError() on the IPC error. -->
              {{ tError(exportPreviewError) }}
            </p>
            <p v-else-if="exportPreviewData === null && exportPreviewLoading" class="ex-status" role="status">
              {{ t('export.preview.loading') }}
            </p>
            <p v-else-if="exportPreviewData === null" class="ex-note" data-export-preview-empty>{{ t('export.preview.empty') }}</p>
            <template v-else>
              <!-- aura-allow-text: data (lines Rust built with the export attribution formatter). -->
              <p v-for="(line, index) in exportPreviewData.attribution" :key="`a-${index}`" class="ex-note" data-export-preview-attribution>{{ line }}</p>
              <!-- aura-allow-text: data (appendix lines Rust built from the confirmed Glossary entries). -->
              <p v-for="(line, index) in appendixLines" :key="`g-${index}`" class="ex-note" data-export-preview-appendix>{{ line }}</p>
              <p v-if="exportPreviewData.appendix_term_count > appendixLines.length - 1 && appendixLines.length > 0" class="ex-note">
                {{ t('export.preview.appendix_more', { count: String(exportPreviewData.appendix_term_count - (appendixLines.length - 1)) }) }}
              </p>
              <div v-if="exportFormat === 'docx_two_column'" class="ex-word" data-export-word-preview>
                <table class="ex-word-table">
                  <tbody>
                    <tr v-for="(row, index) in exportPreviewData.rows" :key="index" data-export-preview-row>
                      <!-- aura-allow-text: data (source and target text of the first Chapter). -->
                      <td>{{ row.source }}</td>
                      <!-- aura-allow-text: data (target text of the first Chapter). -->
                      <td class="ex-pre">{{ row.target }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <div v-else-if="exportFormat === 'docx_one_block'" class="ex-word" data-export-word-preview>
                <table class="ex-word-table">
                  <tbody>
                    <tr v-for="(row, index) in exportPreviewData.rows" :key="index" data-export-preview-row>
                      <!-- aura-allow-text: data (source and target paragraphs of the first Chapter). -->
                      <td class="ex-pre">{{ row.source }}</td>
                      <!-- aura-allow-text: data (target paragraphs of the first Chapter). -->
                      <td class="ex-pre">{{ row.target }}</td>
                    </tr>
                  </tbody>
                </table>
              </div>
              <pre v-else class="ex-mono" data-export-text-preview><!-- aura-allow-text: data (paragraphs of the first Chapter). -->{{ exportPreviewData.rows.map((row) => row.target).join('\n\n') }}</pre>
            </template>
          </section>
        </div>

        <footer class="ex-footer">
          <div class="ex-foot-info">
            <p v-if="exportCountsStatus === 'unknown'" class="ex-summary ex-status" role="status" data-export-counts>{{ t('export.counts.loading') }}</p>
            <p v-else-if="exportCountsStatus === 'none_selected'" class="ex-summary" role="status" data-export-counts>
              {{ t('export.counts.none_selected') }}
            </p>
            <p v-else-if="exportCountsStatus === 'ipc_unavailable'" class="ex-summary" role="status" data-export-counts>
              {{ t('export.counts.ipc_unavailable') }}
            </p>
            <p v-else-if="exportCountsStatus === 'error' && exportCountsError !== null" class="ex-summary ex-error" role="alert" data-export-counts>
              <!-- aura-allow-text: result of tError() on the IPC error. -->
              {{ tError(exportCountsError) }}
            </p>
            <p v-else-if="exportCounts !== null" class="ex-summary" role="status" data-export-counts>
              <!-- aura-allow-text: result of t() with the counts interpolated. -->
              <span>{{ t('export.summary.base', { chapters: String(exportCounts.chapter_count), segments: String(exportCounts.segment_count) }) }}</span>
              <span v-if="exportFormat === 'docx_two_column' && exportCounts.unconfirmed_count > 0" class="ex-piece" data-export-unconfirmed>
                <!-- aura-allow-text: result of t() with the count interpolated. -->
                {{ t('export.summary.unconfirmed', { count: String(exportCounts.unconfirmed_count) }) }}
              </span>
              <template v-else-if="exportFormatIsForPublishing(exportFormat)">
                <span v-if="exportCounts.unconfirmed_translated_count > 0" class="ex-piece" data-export-block-unconfirmed>
                  <!-- aura-allow-text: result of t() with the count interpolated. -->
                  {{ t('export.summary.unconfirmed_publish', { count: String(exportCounts.unconfirmed_translated_count) }) }}
                </span>
                <span v-if="exportCounts.untranslated_count > 0" class="ex-piece" data-export-block-untranslated>
                  <!-- aura-allow-text: result of t() with the count interpolated. -->
                  {{ t('export.summary.untranslated', { count: String(exportCounts.untranslated_count) }) }}
                </span>
              </template>
              <span v-if="exportFormat === 'docx_two_column' && notReimportable" class="ex-piece ex-piece-bad" data-export-summary-not-reimportable>{{ t('export.summary.not_reimportable') }}</span>
              <span
                v-if="exportLinkUsable() && exportImageMode === 'link' && exportCounts.missing_link_images.length > 0"
                class="ex-piece"
                data-export-summary-missing-links
              >
                <!-- aura-allow-text: result of t() with the count interpolated. -->
                {{ t('export.summary.missing_links', { count: String(exportCounts.missing_link_images.length) }) }}
              </span>
            </p>

            <p v-if="exportDestinationError !== null" class="ex-status ex-error" role="alert">
              <!-- aura-allow-text: result of tError() on the IPC error. -->
              {{ tError(exportDestinationError) }}
            </p>
            <p v-else-if="exportDestinationUnavailable" class="ex-status" role="status">{{ t('export.destination.unavailable') }}</p>
            <p v-else-if="exportRunStatus === 'running'" class="ex-status" role="status">{{ t('export.run.running') }}</p>
            <p v-else-if="exportRunStatus === 'ipc_unavailable'" class="ex-status" role="status">
              {{ t('export.run.ipc_unavailable') }}
            </p>
            <p v-else-if="exportRunStatus === 'error' && exportRunError !== null" class="ex-status ex-error" role="alert">
              <!-- aura-allow-text: result of tError() on the IPC error. -->
              {{ tError(exportRunError) }}
            </p>
            <template v-else-if="exportRunStatus === 'done' && exportRunResult !== null">
              <p class="ex-status" role="status">
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
              <p class="ex-status" role="status" data-export-image-result>
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
            </template>
          </div>
          <button type="button" class="ex-act ui-btn ui-btn--lg ui-btn--pair" data-export-cancel @click="dispatch('export.close')">
            {{ t('export.run.cancel') }}
          </button>
          <button type="button" class="ex-act ui-btn ui-btn--lg ui-btn--primary ui-btn--pair" data-export-run :disabled="!canRun" @click="dispatch('export.run')">
            <!-- aura-allow-text: result of t() with chapter count and extension interpolated. -->
            <span data-export-run-label>{{ runLabel }}</span>
            <span class="ex-kbd" data-export-run-shortcut>{{ t('export.run.shortcut') }}</span>
          </button>
        </footer>
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
  align-items: stretch;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

:root[data-platform='mac'] .ex-scrim {
  padding-top: var(--space-titlebar-height);
}

.ex-panel {
  display: flex;
  flex-direction: column;
  width: 100%;
  min-height: 0;
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.ex-panel:focus-visible {
  outline: none;
  border-color: var(--color-primary);
}

.ex-head {
  flex: none;
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  padding: var(--space-panel-block) var(--space-panel-inline);
  border-bottom: 1px solid var(--color-outline);
}

.ex-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.ex-lone {
  margin: var(--space-panel-block) var(--space-panel-inline);
}

.ex-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 343px minmax(0, 1fr);
}

.ex-config {
  min-width: 0;
  overflow-y: auto;
  padding: var(--space-panel-block) var(--space-panel-inline);
  border-right: 1px solid var(--color-outline);
  background: var(--color-background);
}

.ex-preview {
  min-width: 0;
  overflow-y: auto;
  padding: var(--space-panel-block) var(--space-panel-inline);
  background: var(--color-surface-sunken);
}

.ex-preview-head {
  display: flex;
  align-items: baseline;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.ex-preview-title {
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.ex-group {
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
  border: none;
  min-width: 0;
}

.ex-legend {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: calc(var(--space-unit) * 1);
  width: 100%;
  padding: 0;
  margin: 0 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-weight: var(--weight-ui-sm);
  color: var(--color-on-surface-variant);
}

.ex-seg {
  display: flex;
}

.ex-seg-item {
  position: relative;
  flex: 1;
  padding: 0 calc(var(--space-unit) * 1);
  white-space: nowrap;
}

.ex-sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  margin: -1px; /* aura-allow-spacing: geometry of the sr-only hiding technique, not a grid gap */
  padding: 0;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.ex-chapters {
  max-height: calc(var(--space-unit) * 20);
  margin-top: calc(var(--space-unit) * 1);
  overflow-y: auto;
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.ex-chapters .ex-choice {
  display: flex;
  padding-inline: calc(var(--space-unit) * 1);
}

.ex-choice {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 1);
}

.ex-grow {
  flex: 1;
  min-width: 0;
}

.ex-opt {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: calc(var(--space-unit) * 1);
  padding-inline: 8px; /* aura-allow-spacing: 1px card border + 7px card inset, so rows line up with the card contents */
}

.ex-opt:has(> .ui-seg) {
  padding-left: 0;
}

.ex-group > .ex-note,
.ex-group > .ex-status {
  padding-inline: 8px; /* aura-allow-spacing: same inset as .ex-opt */
}

.ex-why,
.ex-meta {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ex-why {
  flex: none;
  max-width: 50%;
  text-align: right;
}

.ex-opt > .ui-seg {
  flex: none;
}

.ex-opt > .ui-seg + .ex-why {
  flex: 1 1 0;
  max-width: none;
}

.ex-opt > .ex-meta {
  flex: none;
}

.ex-card {
  margin-bottom: calc(var(--space-unit) * 1);
  padding: 0 calc(var(--space-unit) * 1);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.ex-card:has(input:checked) {
  border-color: var(--color-on-surface);
}

.ex-card-head {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 1);
}

.ex-tag {
  flex: none;
  padding: 0 calc(var(--space-unit) * 1);
  background: var(--color-surface-sunken);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  white-space: nowrap;
}

.ex-tag-bad {
  color: var(--color-error);
  font-weight: var(--weight-ui-md-strong);
}

.ex-badge {
  margin-left: calc(var(--space-unit) * 1);
  padding: 0 6px; /* aura-allow-spacing: right/left border sits inside the 7px inset */
  border: 1px solid var(--color-outline);
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.ex-status,
.ex-empty,
.ex-note {
  margin: 0 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ex-error {
  color: var(--color-error);
}

.ex-count-list {
  list-style: none;
  padding: 0;
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
}

.ex-warning {
  margin: calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
  border-left: 2px solid var(--color-tm-rule);
  padding-left: 5px; /* aura-allow-spacing: 2px left border sits inside the 7px inset */
}

.ex-footer {
  flex: none;
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  padding: calc(var(--space-unit) * 1) var(--space-panel-inline);
  border-top: 1px solid var(--color-outline);
  background: var(--color-surface-sunken);
}

.ex-kbd {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-sm);
}

.ex-foot-info {
  flex: 1;
  min-width: 0;
}

.ex-foot-info .ex-status {
  margin: 0;
}

.ex-summary {
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.ex-piece-bad {
  color: var(--color-error);
}

.ex-path {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
  overflow-wrap: anywhere;
}

.ex-pre {
  white-space: pre-wrap;
}

.ex-mono {
  margin: 0;
  padding: 13px; /* aura-allow-spacing: 1px border sits inside the 14px inset */
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
  border: 1px solid var(--color-outline);
}

.ex-word {
  max-width: calc(var(--space-unit) * 117);
  margin: 0 auto;
  padding: 13px; /* aura-allow-spacing: 1px border sits inside the 14px inset */
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
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 1) 6px; /* aura-allow-spacing: collapsed 1px row border sits inside the 7px inset */
  text-align: left;
  vertical-align: top;
}
</style>
