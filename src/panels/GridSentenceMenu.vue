<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'
import { dispatch } from '../commands'
import { shortcutRows } from '../config/shortcutsState'
import { quickAddIsOpen } from '../glossaryQuickAddState'
import { t } from '../i18n'
import { editorSegments, editorSourceCut } from './editorPanelState'
import { closeSentenceMenu, placeMenu, sentenceMenu } from './sentenceMenuState'

const root = useTemplateRef<HTMLElement>('root')
const position = ref({ x: 0, y: 0 })
// Read once at open: a real click flushes Vue between the item's listener and the document's, so a
// live binding would disable the item before `onDocumentClick` reads it and the menu would stay open.
const stripWasOpen = ref(false)

const index = computed(() => {
  const open = sentenceMenu.value
  return open === null ? -1 : editorSegments.value.findIndex((s) => s.id === open.segmentId)
})
const segment = computed(() => (index.value < 0 ? null : (editorSegments.value[index.value] ?? null)))
const isFirst = computed(() => index.value === 0)

const cutElsewhereNumber = computed<number | null>(() => {
  const cut = editorSourceCut.value
  const open = sentenceMenu.value
  if (cut === null || open === null || cut.segmentId === open.segmentId) return null
  return editorSegments.value.findIndex((s) => s.id === cut.segmentId) + 1
})

const chordById = computed(() => {
  const byId = new Map<string, string>()
  for (const row of shortcutRows.value) byId.set(row.id, row.display.join(' / '))
  return byId
})

function chordOf(id: string): string {
  return chordById.value.get(id) ?? ''
}

function holdFocus(event: MouseEvent): void {
  event.preventDefault()
}

watch(sentenceMenu, async (open) => {
  if (open === null) return
  stripWasOpen.value = quickAddIsOpen.value
  position.value = { x: open.x, y: open.y }
  await nextTick()
  const rect = root.value?.getBoundingClientRect()
  if (rect === undefined) return
  position.value = placeMenu(
    { x: open.x, y: open.y },
    { width: rect.width, height: rect.height },
    { width: window.innerWidth, height: window.innerHeight },
  )
})

function inside(event: Event): boolean {
  return event.target instanceof Node && root.value?.contains(event.target) === true
}

function onDocumentMouseDown(event: MouseEvent): void {
  if (sentenceMenu.value === null || inside(event)) return
  closeSentenceMenu('dismissed')
}

const MODIFIER_KEYS = new Set(['Shift', 'Control', 'Alt', 'Meta'])

function onDocumentKeydown(event: KeyboardEvent): void {
  if (event.defaultPrevented || sentenceMenu.value === null) return
  if (event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    closeSentenceMenu('dismissed')
    return
  }
  if (MODIFIER_KEYS.has(event.key)) return
  closeSentenceMenu('dismissed')
}

function onDocumentClick(event: MouseEvent): void {
  if (sentenceMenu.value === null || !inside(event)) return
  const button = event.target instanceof Element ? event.target.closest('button') : null
  if (button === null || button.disabled) return
  if (button.hasAttribute('data-opens-glossary-strip')) closeSentenceMenu('glossary')
  else closeSentenceMenu(button.hasAttribute('data-opens-overlay') ? 'overlay' : 'chosen')
}

function onDocumentContextMenu(event: MouseEvent): void {
  if (sentenceMenu.value !== null && inside(event)) event.preventDefault()
}

onMounted(() => {
  document.addEventListener('mousedown', onDocumentMouseDown, true)
  document.addEventListener('keydown', onDocumentKeydown, true)
  document.addEventListener('click', onDocumentClick)
  document.addEventListener('contextmenu', onDocumentContextMenu)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocumentMouseDown, true)
  document.removeEventListener('keydown', onDocumentKeydown, true)
  document.removeEventListener('click', onDocumentClick)
  document.removeEventListener('contextmenu', onDocumentContextMenu)
})
</script>

<template>
  <Teleport to="body">
    <ul
      v-if="segment !== null"
      ref="root"
      class="sentence-menu ui-menu"
      role="menu"
      :aria-label="t('panel.grid.menu.aria')"
      :style="{ left: `${position.x}px`, top: `${position.y}px` }"
      data-sentence-menu
      @mousedown="holdFocus"
    >
      <li class="sentence-menu-title" role="presentation">{{ t('panel.grid.menu.title', { n: String(index + 1) }) }}</li>
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="ui-menu-item" :title="t('command.editor.confirm_segment')" @click="dispatch('editor.confirm_segment')">
          <span>{{ t('panel.grid.menu.confirm') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('editor.confirm_segment') }}</span>
        </button>
      </li>
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="ui-menu-item" :disabled="isFirst" :title="t('command.editor.merge_segments')" @click="dispatch('editor.merge_segments')">
          <span>{{ t('panel.grid.menu.merge') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('editor.merge_segments') }}</span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="ui-menu-item"
          :disabled="cutElsewhereNumber !== null"
          :title="cutElsewhereNumber === null ? t('command.editor.split_segment') : t('panel.grid.menu.split_cut_elsewhere', { n: String(cutElsewhereNumber) })"
          @click="dispatch('editor.split_segment')"
        >
          <span>{{ t('panel.grid.menu.split') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('editor.split_segment') }}</span>
        </button>
      </li>
      <li v-if="editorSourceCut !== null" role="none">
        <button type="button" role="menuitem" tabindex="-1" class="ui-menu-item" :title="t('command.editor.clear_source_cuts')" @click="dispatch('editor.clear_source_cuts')">
          <span>{{ t('panel.grid.menu.clear_cuts') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('editor.clear_source_cuts') }}</span>
        </button>
      </li>
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="ui-menu-item" :title="t('command.editor.next_untranslated')" @click="dispatch('editor.next_untranslated')">
          <span>{{ t('panel.grid.menu.next_untranslated') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('editor.next_untranslated') }}</span>
        </button>
      </li>
      <li class="ui-menu-sep" role="separator"></li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="ui-menu-item"
          data-opens-glossary-strip
          :disabled="stripWasOpen"
          :title="stripWasOpen ? t('panel.grid.menu.add_term_open') : t('command.glossary.add_term')"
          @click="dispatch('glossary.add_term')"
        >
          <span>{{ t('panel.grid.menu.add_term') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('glossary.add_term') }}</span>
        </button>
      </li>
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="ui-menu-item" :title="t('command.tm.concordance')" @click="dispatch('tm.concordance')">
          <span>{{ t('panel.grid.menu.concordance') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('tm.concordance') }}</span>
        </button>
      </li>
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="ui-menu-item" data-opens-overlay :title="t('command.history.open')" @click="dispatch('history.open')">
          <span>{{ t('panel.grid.menu.history') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('history.open') }}</span>
        </button>
      </li>
      <li class="ui-menu-sep" role="separator"></li>
      <li v-if="!segment.is_omitted" role="none">
        <button type="button" role="menuitem" tabindex="-1" class="ui-menu-item" :title="t('command.editor.omit_segment')" @click="dispatch('editor.omit_segment')">
          <span>{{ t('panel.grid.menu.omit') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('editor.omit_segment') }}</span>
        </button>
      </li>
      <li v-else role="none">
        <button type="button" role="menuitem" tabindex="-1" class="ui-menu-item" :title="t('command.editor.restore_segment')" @click="dispatch('editor.restore_segment')">
          <span>{{ t('panel.grid.menu.restore') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('editor.restore_segment') }}</span>
        </button>
      </li>
      <li v-if="!segment.is_target_paragraph_end" role="none">
        <button type="button" role="menuitem" tabindex="-1" class="ui-menu-item" :title="t('command.editor.end_target_paragraph')" @click="dispatch('editor.end_target_paragraph')">
          <span>{{ t('panel.grid.menu.end_paragraph') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('editor.end_target_paragraph') }}</span>
        </button>
      </li>
      <li v-else role="none">
        <button type="button" role="menuitem" tabindex="-1" class="ui-menu-item" :title="t('command.editor.join_target_paragraph')" @click="dispatch('editor.join_target_paragraph')">
          <span>{{ t('panel.grid.menu.join_paragraph') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('editor.join_target_paragraph') }}</span>
        </button>
      </li>
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="ui-menu-item" :title="t('command.editor.split_chapter')" @click="dispatch('editor.split_chapter')">
          <span>{{ t('panel.grid.menu.split_chapter') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="sentence-menu-chord">{{ chordOf('editor.split_chapter') }}</span>
        </button>
      </li>
    </ul>
  </Teleport>
</template>

<style scoped>
.sentence-menu {
  /* aura-allow-z-index: the menu floats above the dockview tree, which isolates its own stacking context. */
  z-index: 30;
  position: fixed;
  width: calc(var(--space-unit) * 42);
  box-shadow: var(--shadow-popover);
}

.sentence-menu-title {
  padding: 0 calc(var(--space-unit) * 2);
  color: var(--color-on-surface-variant);
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  letter-spacing: var(--tracking-ui-label);
}

.sentence-menu-chord {
  margin-left: auto;
  color: var(--color-on-surface-variant);
}
</style>
