<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import { chapterMergeOverlayIsOpen, chapterMergeRows } from './chapterMergeState'
import {
  libraryChapterReorgBusy,
  libraryChapterReorgError,
  libraryChapterReorgNotice,
} from './modes/libraryChapters'
import { chapterTitleLabel, lifecycleStatusLabel } from './modes/libraryChapterLabels'

let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

const first = computed(() => chapterMergeRows.value.at(0) ?? null)
const totalSentences = computed(() => chapterMergeRows.value.reduce((sum, row) => sum + row.in_translation_count, 0))

watch(chapterMergeOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-library-chapter-merge-open]')
    void nextTick(() => panel.value?.focus())
    return
  }
  const back = returnFocusTo
  returnFocusTo = null
  if (back !== null && back.isConnected && !(back instanceof HTMLButtonElement && back.disabled)) {
    back.focus()
    return
  }
  const list = document.querySelector<HTMLElement>('[data-library-chapters-list]')
  if (list !== null) {
    list.focus()
    return
  }
  console.warn('[chapter-merge] focus-return target is gone; focus falls back to body.')
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
    v-if="chapterMergeOverlayIsOpen"
    class="cm-scrim"
    data-library-chapter-merge-overlay
    @keydown.esc="dispatch('library.chapter_merge.close')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="cm-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="cm-title">
      <header class="cm-head">
        <h2 id="cm-title" class="cm-title">
          {{ t('library.chapter_merge.title', { count: String(chapterMergeRows.length) }) }}
        </h2>
        <button type="button" class="cm-close" data-library-chapter-merge-close @click="dispatch('library.chapter_merge.close')">
          {{ t('command.library.chapter_merge.close') }}
        </button>
      </header>

      <p class="cm-note">{{ t('library.chapter_merge.intro') }}</p>

      <section class="cm-card">
        <h3 class="cm-card-title">{{ t('library.chapter_merge.order_heading') }}</h3>
        <ul class="cm-list" data-library-chapter-merge-list>
          <li v-for="row in chapterMergeRows" :key="row.chapter_id" class="cm-row">
            <!-- aura-allow-text: thứ tự Chương là số đếm (dữ liệu). -->
            <span class="cm-ord">{{ row.ord }}</span>
            <!-- aura-allow-text: tên Chương là dữ liệu người dùng; nhãn dự phòng qua t(). -->
            <span class="cm-name">{{ chapterTitleLabel(row) }}</span>
            <!-- aura-allow-text: ghép từ t() với số đếm và nhãn trạng thái đã dịch. -->
            <span class="cm-meta">{{
              t('library.chapter_merge.row_meta', {
                count: String(row.in_translation_count),
                status: lifecycleStatusLabel(row.status),
              })
            }}</span>
          </li>
        </ul>
        <p v-if="first !== null" class="cm-result" data-library-chapter-merge-result>
          <!-- aura-allow-text: ghép từ t() với dữ liệu. -->
          {{
            t('library.chapter_merge.result', {
              ord: String(first.ord),
              name: chapterTitleLabel(first),
              count: String(totalSentences),
            })
          }}
        </p>
        <p class="cm-note">{{ t('library.chapter_merge.name_note') }}</p>
      </section>

      <section class="cm-card">
        <h3 class="cm-card-title">{{ t('library.chapter_merge.work_heading') }}</h3>
        <p class="cm-note">{{ t('library.chapter_merge.work_kept') }}</p>
        <p class="cm-note">{{ t('library.chapter_merge.no_undo') }}</p>
      </section>

      <!-- aura-allow-text: mọi nhánh đều qua t()/tError(). -->
      <p class="cm-error" role="alert">
        {{
          libraryChapterReorgError
            ? tError(libraryChapterReorgError)
            : libraryChapterReorgNotice === 'flush-failed'
              ? t('mode.library.chapter_reorg_flush_failed')
              : libraryChapterReorgNotice === 'still-dirty'
                ? t('mode.library.chapter_reorg_still_dirty')
                : ''
        }}
      </p>

      <div class="cm-actions">
        <button
          type="button"
          class="cm-act cm-act-primary"
          data-library-chapter-merge-confirm
          :disabled="libraryChapterReorgBusy"
          @click="dispatch('library.chapter_merge.confirm')"
        >
          {{ t('command.library.chapter_merge.confirm') }}
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.cm-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: above the shell like the other overlays. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.cm-panel {
  width: 100%;
  max-width: 720px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.cm-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.cm-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.cm-close {
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

.cm-card {
  margin: 0 0 var(--space-panel-block) 0;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
}

.cm-card-title {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.cm-list {
  list-style: none;
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  padding: 0;
}

.cm-row {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 3);
  padding: calc(var(--space-unit) * 1) 0;
  border-bottom: 1px solid var(--color-outline);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.cm-ord {
  min-width: calc(var(--space-unit) * 8);
  color: var(--color-on-surface-variant);
}

.cm-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cm-meta {
  white-space: nowrap;
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.cm-result {
  margin: calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.cm-note {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.cm-error {
  min-height: 1em;
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-error);
}

.cm-actions {
  display: flex;
  justify-content: flex-end;
}

.cm-act {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.cm-act-primary {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: var(--color-on-primary);
}

.cm-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}
</style>
