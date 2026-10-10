<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import {
  reviewerImportConfirmError,
  reviewerImportConfirmUnavailable,
  reviewerImportConfirming,
  reviewerImportLoadError,
  reviewerImportOverlayIsOpen,
  reviewerImportPreview,
  reviewerImportStatus,
  reviewerImportSummary,
} from './reviewerImportState'

let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

watch(reviewerImportOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-reviewer-import-open]')
    void nextTick(() => panel.value?.focus())
    return
  }
  const back = returnFocusTo
  returnFocusTo = null
  if (back !== null && back.isConnected) {
    back.focus()
    return
  }
  const opener = document.querySelector<HTMLElement>('[data-reviewer-import-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[reviewer-import] focus-return target is gone; focus falls back to body.')
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
    v-if="reviewerImportOverlayIsOpen"
    class="ri-scrim"
    @keydown.esc="dispatch('export.reviewer_import.cancel')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="ri-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="ri-title">
      <header class="ri-head">
        <h2 id="ri-title" class="ri-title">{{ t('reviewer.import.title') }}</h2>
        <button
          type="button"
          class="ri-close"
          :disabled="reviewerImportConfirming"
          @click="dispatch('export.reviewer_import.cancel')"
        >
          {{ t('command.export.reviewer_import.cancel') }}
        </button>
      </header>

      <p v-if="reviewerImportStatus === 'unknown'" class="ri-status" role="status">{{ t('reviewer.import.loading') }}</p>
      <p v-else-if="reviewerImportStatus === 'ipc_unavailable'" class="ri-empty">
        {{ t('reviewer.import.empty_ipc_unavailable') }}
      </p>
      <p
        v-else-if="reviewerImportStatus === 'error' && reviewerImportLoadError !== null"
        class="ri-empty ri-error"
        role="alert"
      >
        <!-- aura-allow-text: result of tError() computed in the script. -->
        {{ tError(reviewerImportLoadError) }}
      </p>
      <template v-else-if="reviewerImportStatus === 'done' && reviewerImportSummary !== null">
        <p class="ri-status" role="status">
          <!-- aura-allow-text: result of t() with the counts interpolated. -->
          {{
            t('reviewer.import.done', {
              chapters: String(reviewerImportSummary.chapter_count),
              rows: String(reviewerImportSummary.row_count),
              replaced: String(reviewerImportSummary.replaced_count),
            })
          }}
        </p>
        <p v-if="reviewerImportSummary.harvest_error !== null" class="ri-status ri-error" role="alert">
          <!-- aura-allow-text: result of tError() computed in the script. -->
          {{ tError(reviewerImportSummary.harvest_error) }}
        </p>
        <p v-else-if="reviewerImportSummary.harvest_candidate_count !== null" class="ri-status" role="status">
          <!-- aura-allow-text: result of t() with the count interpolated. -->
          {{ t('reviewer.import.harvest_done', { count: String(reviewerImportSummary.harvest_candidate_count) }) }}
        </p>
        <div class="ri-actions">
          <button type="button" class="ri-act ri-act-primary" @click="dispatch('export.alignment.open')">
            {{ t('command.export.alignment.open') }}
          </button>
        </div>
      </template>
      <template v-else-if="reviewerImportStatus === 'loaded' && reviewerImportPreview !== null">
        <dl class="ri-summary">
          <div class="ri-summary-row">
            <dt>{{ t('reviewer.import.file_name_label') }}</dt>
            <!-- aura-allow-text: data (file name chosen by the user). -->
            <dd>{{ reviewerImportPreview.file_name }}</dd>
          </div>
          <div class="ri-summary-row">
            <dt>{{ t('reviewer.import.file_kind_label') }}</dt>
            <dd>
              {{ t(reviewerImportPreview.file_kind === 'docx' ? 'reviewer.import.file_kind_docx' : 'reviewer.import.file_kind_md') }}
            </dd>
          </div>
        </dl>

        <h3 class="ri-sub">{{ t('reviewer.import.chapters_heading') }}</h3>
        <ul class="ri-list">
          <li v-for="chapter in reviewerImportPreview.chapters" :key="chapter.chapter_id">
            <!-- aura-allow-text: result of t() with the chapter data interpolated. -->
            {{
              chapter.title === null
                ? t('reviewer.import.chapter_row', { ord: String(chapter.chapter_ord), count: String(chapter.row_count) })
                : t('reviewer.import.chapter_row_titled', {
                    ord: String(chapter.chapter_ord),
                    title: chapter.title,
                    count: String(chapter.row_count),
                  })
            }}
            <span v-if="chapter.replaces !== null" class="ri-note">
              <!-- aura-allow-text: result of t() with the previous file name interpolated. -->
              {{
                t(chapter.replaces.stale ? 'reviewer.import.replaces_stale' : 'reviewer.import.replaces', {
                  file_name: chapter.replaces.file_name,
                })
              }}
              <template v-if="chapter.replaces.user_group_count > 0">
                <!-- aura-allow-text: result of t() with the count interpolated. -->
                {{ t('reviewer.import.replaces_user_groups', { count: String(chapter.replaces.user_group_count) }) }}
              </template>
              <template v-if="chapter.replaces.accepted_group_count > 0">
                <!-- aura-allow-text: result of t() with the count interpolated. -->
                {{ t('reviewer.import.replaces_accepted_groups', { count: String(chapter.replaces.accepted_group_count) }) }}
              </template>
            </span>
          </li>
        </ul>

        <template v-if="reviewerImportPreview.skipped.length > 0">
          <h3 class="ri-sub">{{ t('reviewer.import.skipped_heading') }}</h3>
          <ul class="ri-list">
            <li v-for="(skipped, index) in reviewerImportPreview.skipped" :key="index">
              <!-- aura-allow-text: result of t() with the section heading interpolated. -->
              {{ t('reviewer.import.skipped_row', { heading: skipped.heading, count: String(skipped.row_count) }) }}
            </li>
          </ul>
        </template>
        <p v-if="reviewerImportPreview.image_rows_ignored > 0" class="ri-note">
          <!-- aura-allow-text: result of t() with the count interpolated. -->
          {{ t('reviewer.import.images_ignored', { count: String(reviewerImportPreview.image_rows_ignored) }) }}
        </p>

        <p v-if="reviewerImportConfirmError !== null" class="ri-status ri-error" role="alert">
          <!-- aura-allow-text: result of tError() computed in the script. -->
          {{ tError(reviewerImportConfirmError) }}
        </p>
        <p v-else-if="reviewerImportConfirmUnavailable" class="ri-status" role="status">
          {{ t('reviewer.import.confirm_ipc_unavailable') }}
        </p>
        <p v-else-if="reviewerImportConfirming" class="ri-status" role="status">{{ t('reviewer.import.confirming') }}</p>
        <p class="ri-hint">{{ t('reviewer.import.hint_no_write_before_confirm') }}</p>

        <div class="ri-actions">
          <button
            type="button"
            class="ri-act ri-act-primary"
            :disabled="reviewerImportConfirming"
            @click="dispatch('export.reviewer_import.confirm')"
          >
            <!-- aura-allow-text: result of t() with the count interpolated. -->
            {{ t('reviewer.import.confirm_button', { count: String(reviewerImportPreview.chapters.length) }) }}
          </button>
          <button
            type="button"
            class="ri-act"
            :disabled="reviewerImportConfirming"
            @click="dispatch('export.reviewer_import.cancel')"
          >
            {{ t('command.export.reviewer_import.cancel') }}
          </button>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.ri-scrim {
  position: fixed;
  inset: 0;
  z-index: 11; /* aura-allow-z-index: same layer as TmImportOverlay; both are full-screen scrims. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.ri-panel {
  width: 100%;
  max-width: 720px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.ri-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.ri-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.ri-close {
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

.ri-status {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ri-empty {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.ri-error {
  color: var(--color-error);
}

.ri-summary {
  margin: 0 0 var(--space-panel-block) 0;
}

.ri-summary-row {
  display: flex;
  gap: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
}

.ri-summary-row dt {
  color: var(--color-on-surface-variant);
  min-width: 8rem;
}

.ri-summary-row dd {
  margin: 0;
  color: var(--color-on-surface);
}

.ri-note {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ri-sub {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.ri-list {
  list-style: none;
  padding: 0;
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.ri-hint {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ri-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}

.ri-act {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.ri-act-primary {
  background: var(--color-primary);
  color: var(--color-on-primary);
  border-color: var(--color-primary);
}

.ri-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}
</style>
