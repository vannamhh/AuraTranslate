<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from 'vue'
import { dispatch } from './commands'
import { t, tError } from './i18n'
import AppIcon from './AppIcon.vue'
import { shortcutRows } from './config/shortcutsState'
import { closeToolsMenu, toolsMenuIsOpen } from './titlebarToolsState'
import { panelHeadersSaveError, panelHeadersShown } from './layout/panelHeadersState'
import { attributionIsOpen } from './panels/dictSourcesState'

const root = useTemplateRef<HTMLElement>('root')
const trigger = useTemplateRef<HTMLButtonElement>('trigger')

const chordById = computed(() => {
  const byId = new Map<string, string>()
  for (const row of shortcutRows.value) byId.set(row.id, row.display.join(' / '))
  return byId
})

function chordOf(id: string): string {
  return chordById.value.get(id) ?? ''
}

function menuItems(): HTMLElement[] {
  return Array.from(root.value?.querySelectorAll<HTMLElement>('[role="menuitem"], [role="menuitemcheckbox"]') ?? [])
}

// WKWebView does not focus a <button> on mouse click; without this the overlay saves the previously
// focused node as its opener and UX-DR17 returns focus there instead of to `Công cụ ▾`.
function focusOnPointerDown(event: MouseEvent) {
  const target = event.currentTarget
  if (target instanceof HTMLElement) target.focus()
}

function onKeydown(event: KeyboardEvent) {
  if (!toolsMenuIsOpen.value) return
  if (event.key === 'Escape') {
    event.preventDefault()
    closeToolsMenu()
    trigger.value?.focus()
    return
  }
  if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return
  const items = menuItems()
  if (items.length === 0) return
  event.preventDefault()
  const at = items.findIndex((item) => item === document.activeElement)
  const step = event.key === 'ArrowDown' ? 1 : -1
  const next = at === -1 ? (step === 1 ? 0 : items.length - 1) : (at + step + items.length) % items.length
  items[next]?.focus()
}

function onDocumentMouseDown(event: MouseEvent) {
  if (!toolsMenuIsOpen.value) return
  const target = event.target
  if (target instanceof Node && root.value?.contains(target)) return
  closeToolsMenu()
}

function onRootClick(event: MouseEvent) {
  const target = event.target
  if (target instanceof Element && target.closest('[role="menuitem"], [role="menuitemcheckbox"]') !== null) closeToolsMenu()
}

function onFocusOut(event: FocusEvent) {
  const next = event.relatedTarget
  if (next instanceof Node && root.value !== null && !root.value.contains(next)) closeToolsMenu()
}

onMounted(() => {
  document.addEventListener('mousedown', onDocumentMouseDown, true)
  root.value?.addEventListener('click', onRootClick)
  root.value?.addEventListener('focusout', onFocusOut)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocumentMouseDown, true)
  root.value?.removeEventListener('click', onRootClick)
  root.value?.removeEventListener('focusout', onFocusOut)
})
</script>

<template>
  <div ref="root" class="tools" @keydown="onKeydown($event)">
    <button
      ref="trigger"
      type="button"
      class="tools-trigger ui-btn ui-btn--sm ui-btn--quiet"
      aria-haspopup="menu"
      :aria-expanded="toolsMenuIsOpen"
      :disabled="attributionIsOpen"
      data-tools-trigger
      data-glossary-manage-open
      data-tm-manage-open
      data-settings-open
      data-proofread-scope-open
      data-proofread-memory-open
      data-reviewer-import-open
      data-alignment-open
      data-glossary-harvest-open
      @mousedown="focusOnPointerDown($event)"
      @click="dispatch('titlebar.tools.toggle')"
    >
      {{ t('titlebar.tools.label') }}
      <AppIcon name="chevron" />
    </button>

    <ul v-if="toolsMenuIsOpen" class="tools-list" role="menu" :aria-label="t('titlebar.tools.menu')">
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="tools-item" data-tools-item="glossary.manage.open" @click="dispatch('glossary.manage.open')">
          <span>{{ t('command.glossary.manage.open') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="tools-chord">{{ chordOf('glossary.manage.open') }}</span>
        </button>
      </li>
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="tools-item" data-tools-item="tm.manage.open" @click="dispatch('tm.manage.open')">
          <span>{{ t('command.tm.manage.open') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="tools-chord">{{ chordOf('tm.manage.open') }}</span>
        </button>
      </li>
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="tools-item" data-tools-item="settings.open" @click="dispatch('settings.open')">
          <span>{{ t('command.settings.open') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="tools-chord">{{ chordOf('settings.open') }}</span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="tools-item"
          data-tools-item="ai.proofread.scope.open" @click="dispatch('ai.proofread.scope.open')"
        >
          <span>{{ t('command.ai.proofread.scope.open') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="tools-chord">{{ chordOf('ai.proofread.scope.open') }}</span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="tools-item"
          data-tools-item="ai.proofread.memory.open" @click="dispatch('ai.proofread.memory.open')"
        >
          <span>{{ t('command.ai.proofread.memory.open') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="tools-chord">{{ chordOf('ai.proofread.memory.open') }}</span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="tools-item"
          data-tools-item="export.reviewer_import.open" @click="dispatch('export.reviewer_import.open')"
        >
          <span>{{ t('command.export.reviewer_import.open') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="tools-chord">{{ chordOf('export.reviewer_import.open') }}</span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="tools-item"
          data-tools-item="export.alignment.open" @click="dispatch('export.alignment.open')"
        >
          <span>{{ t('command.export.alignment.open') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="tools-chord">{{ chordOf('export.alignment.open') }}</span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="tools-item"
          data-tools-item="glossary.harvest.open" @click="dispatch('glossary.harvest.open')"
        >
          <span>{{ t('command.glossary.harvest.open') }}</span>
          <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
          <span class="tools-chord">{{ chordOf('glossary.harvest.open') }}</span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitemcheckbox"
          tabindex="-1"
          class="tools-item"
          :aria-checked="panelHeadersShown"
          data-tools-item="layout.toggle_panel_headers"
          @click="dispatch('layout.toggle_panel_headers')"
        >
          <span>{{ t('command.layout.toggle_panel_headers') }}</span>
          <span class="tools-switch" :class="{ on: panelHeadersShown }" aria-hidden="true"></span>
        </button>
      </li>
      <li v-if="panelHeadersSaveError !== null" role="none">
        <p class="tools-alert" role="alert">{{ tError(panelHeadersSaveError) }}</p>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.tools {
  position: relative;
}

.tools-list {
  /* aura-allow-z-index: the list opens over the dockview tree, which isolates its own stacking context under .modeport. */
  z-index: 20;
  position: absolute;
  top: 100%;
  right: 0;
  min-width: calc(var(--space-unit) * 56);
  margin: 0;
  padding: var(--space-unit) 0;
  list-style: none;
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
}

.tools-item {
  display: flex;
  justify-content: space-between;
  gap: calc(var(--space-unit) * 4);
  width: 100%;
  padding: var(--space-unit) calc(var(--space-unit) * 3);
  background: none;
  border: none;
  cursor: pointer;
  text-align: left;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.tools-item:hover,
.tools-item:focus-visible {
  background: var(--color-surface-sunken);
}

/* On and off differ by fill, not by colour alone. */
.tools-switch {
  flex: none;
  align-self: center;
  width: calc(var(--space-unit) * 4);
  height: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-control-rule);
  border-radius: var(--radius-sm);
}

.tools-switch.on {
  border-color: var(--color-on-surface);
  background: var(--color-on-surface);
}

.tools-alert {
  margin: 0;
  padding: var(--space-unit) calc(var(--space-unit) * 3);
  color: var(--color-error);
}

.tools-chord {
  color: var(--color-on-surface-variant);
}
</style>
