<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import {
  chapterSplitBusy,
  chapterSplitCanConfirm,
  chapterSplitChapter,
  chapterSplitFailure,
  chapterSplitLoadStatus,
  chapterSplitOverlayIsOpen,
  chapterSplitPickedId,
  chapterSplitSentences,
  pickChapterSplitSentence,
} from './chapterSplitState'
import { chapterTitleLabel } from './modes/libraryChapterLabels'

let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

watch(chapterSplitOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-library-chapter-split-open]')
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
  console.warn('[chapter-split] focus-return target is gone; focus falls back to body.')
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
    v-if="chapterSplitOverlayIsOpen"
    class="cs-scrim"
    data-library-chapter-split-overlay
    @keydown.esc="dispatch('library.chapter_split.close')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="cs-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="cs-title">
      <header class="cs-head">
        <h2 id="cs-title" class="cs-title">
          <!-- aura-allow-text: ghép từ t() với dữ liệu Chương. -->
          {{
            chapterSplitChapter === null
              ? ''
              : t('library.chapter_split.title', { name: chapterTitleLabel(chapterSplitChapter) })
          }}
        </h2>
        <button type="button" class="cs-close" data-library-chapter-split-close @click="dispatch('library.chapter_split.close')">
          {{ t('command.library.chapter_split.close') }}
        </button>
      </header>

      <p v-if="chapterSplitLoadStatus === 'loading'" class="cs-note" role="status">{{ t('library.chapter_split.loading') }}</p>
      <p v-else-if="chapterSplitLoadStatus === 'failed'" class="cs-error" role="alert">{{ t('library.chapter_split.load_failed') }}</p>
      <template v-else>
        <p class="cs-note">{{ t('library.chapter_split.intro') }}</p>
        <p class="cs-note">{{ t('library.chapter_split.first_note') }}</p>
        <ul class="cs-list" data-library-chapter-split-list>
          <li v-for="(sentence, index) in chapterSplitSentences" :key="sentence.id" class="cs-row">
            <label class="cs-label">
              <input
                type="radio"
                name="chapter-split-at"
                :value="sentence.id"
                :checked="chapterSplitPickedId === sentence.id"
                :disabled="index === 0 || chapterSplitBusy"
                :data-library-chapter-split-sentence="sentence.id"
                @change="pickChapterSplitSentence(sentence.id)"
              />
              <!-- aura-allow-text: số thứ tự câu là số đếm (dữ liệu). -->
              <span class="cs-ord">{{ index + 1 }}</span>
              <!-- aura-allow-text: câu nguồn là dữ liệu người dùng. -->
              <span class="cs-text">{{ sentence.source_text }}</span>
            </label>
          </li>
        </ul>
      </template>

      <!-- aura-allow-text: mọi nhánh đều qua t()/tError(). -->
      <p class="cs-error" role="alert">
        {{
          chapterSplitFailure === null
            ? ''
            : chapterSplitFailure.error !== null
              ? tError(chapterSplitFailure.error)
              : chapterSplitFailure.notice === 'flush-failed'
                ? t('mode.library.chapter_reorg_flush_failed')
                : t('library.chapter_split.refused')
        }}
      </p>

      <div class="cs-actions">
        <button
          type="button"
          class="cs-act cs-act-primary"
          data-library-chapter-split-confirm
          :disabled="chapterSplitLoadStatus !== 'loaded' || !chapterSplitCanConfirm()"
          @click="dispatch('library.chapter_split.confirm')"
        >
          {{ t('command.library.chapter_split.confirm') }}
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.cs-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: above the shell like the other overlays. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.cs-panel {
  width: 100%;
  max-width: 720px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.cs-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.cs-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.cs-close {
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

.cs-note {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.cs-list {
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
  max-height: 50vh;
  overflow-y: auto;
  border: 1px solid var(--color-outline);
}

.cs-row {
  border-bottom: 1px solid var(--color-outline);
}

.cs-label {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 3);
  padding: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.cs-ord {
  min-width: calc(var(--space-unit) * 8);
  color: var(--color-on-surface-variant);
}

.cs-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cs-error {
  min-height: 1em;
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-error);
}

.cs-actions {
  display: flex;
  justify-content: flex-end;
}

.cs-act {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.cs-act-primary {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: var(--color-on-primary);
}

.cs-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}
</style>
