<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef } from 'vue'
import { dispatch } from './commands'
import { t, tError } from './i18n'
import AppIcon from './AppIcon.vue'
import { commandTitle, shortcutRows } from './config/shortcutsState'
import { closeToolsMenu, toolsMenuIsOpen } from './titlebarToolsState'
import { panelHeadersSaveError, panelHeadersShown } from './layout/panelHeadersState'
import { attributionIsOpen } from './panels/dictSourcesState'
import { visibleDockPanels } from './layout/panelVisibilityState'
import { currentMode } from './modes/modeState'
import { reviewModeIsOpen } from './reviewModeState'
import type { PanelId } from './layout/workspaceLayout'

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

function panelSwitchDisabled(id: PanelId): boolean {
  return reviewModeIsOpen.value || (visibleDockPanels.value.has(id) && visibleDockPanels.value.size <= 1)
}

function menuItems(): HTMLElement[] {
  return Array.from(
    root.value?.querySelectorAll<HTMLButtonElement>('[role="menuitem"], [role="menuitemcheckbox"]') ?? [],
  ).filter((item) => !item.disabled)
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
  if (target instanceof Element && target.closest('[role="menuitem"]:not(:disabled), [role="menuitemcheckbox"]:not(:disabled)') !== null) closeToolsMenu()
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
      class="tools-trigger ui-btn ui-btn--quiet"
      aria-haspopup="menu"
      :aria-expanded="toolsMenuIsOpen"
      :disabled="attributionIsOpen"
      data-tools-trigger
      data-glossary-manage-open
      data-tm-manage-open
      data-settings-open
      data-proofread-memory-open
      data-convention-manage-open
      data-reviewer-import-open
      data-alignment-open
      data-glossary-harvest-open
      @mousedown="focusOnPointerDown($event)"
      @click="dispatch('titlebar.tools.toggle')"
    >
      <span class="ui-row">
        {{ t('titlebar.tools.label') }}
        <AppIcon name="chevron" />
      </span>
    </button>

    <ul v-if="toolsMenuIsOpen" class="tools-list ui-menu" role="menu" :aria-label="t('titlebar.tools.menu')">
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="tools-item ui-menu-item" data-tools-item="glossary.manage.open" @click="dispatch('glossary.manage.open')">
          <span class="ui-row">
            <span>{{ t('command.glossary.manage.open') }}</span>
            <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
            <span class="tools-chord">{{ chordOf('glossary.manage.open') }}</span>
          </span>
        </button>
      </li>
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="tools-item ui-menu-item" data-tools-item="glossary.settings.open" @click="dispatch('glossary.settings.open')">
          <span class="ui-row">
            <span>{{ t('command.glossary.settings.open') }}</span>
            <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
            <span class="tools-chord">{{ chordOf('glossary.settings.open') }}</span>
          </span>
        </button>
      </li>
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="tools-item ui-menu-item" data-tools-item="tm.manage.open" @click="dispatch('tm.manage.open')">
          <span class="ui-row">
            <span>{{ t('command.tm.manage.open') }}</span>
            <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
            <span class="tools-chord">{{ chordOf('tm.manage.open') }}</span>
          </span>
        </button>
      </li>
      <li role="none">
        <button type="button" role="menuitem" tabindex="-1" class="tools-item ui-menu-item" data-tools-item="settings.open" @click="dispatch('settings.open')">
          <span class="ui-row">
            <span>{{ t('command.settings.open') }}</span>
            <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
            <span class="tools-chord">{{ chordOf('settings.open') }}</span>
          </span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="tools-item ui-menu-item"
          data-tools-item="ai.proofread.memory.open" @click="dispatch('ai.proofread.memory.open')"
        >
          <span class="ui-row">
            <span>{{ t('command.ai.proofread.memory.open') }}</span>
            <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
            <span class="tools-chord">{{ chordOf('ai.proofread.memory.open') }}</span>
          </span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="tools-item ui-menu-item"
          data-tools-item="convention.manage.open" @click="dispatch('convention.manage.open')"
        >
          <span class="ui-row">
            <span>{{ t('command.convention.manage.open') }}</span>
            <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
            <span class="tools-chord">{{ chordOf('convention.manage.open') }}</span>
          </span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="tools-item ui-menu-item"
          data-tools-item="export.reviewer_import.open" @click="dispatch('export.reviewer_import.open')"
        >
          <span class="ui-row">
            <span>{{ t('command.export.reviewer_import.open') }}</span>
            <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
            <span class="tools-chord">{{ chordOf('export.reviewer_import.open') }}</span>
          </span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="tools-item ui-menu-item"
          data-tools-item="export.alignment.open" @click="dispatch('export.alignment.open')"
        >
          <span class="ui-row">
            <span>{{ t('command.export.alignment.open') }}</span>
            <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
            <span class="tools-chord">{{ chordOf('export.alignment.open') }}</span>
          </span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitem"
          tabindex="-1"
          class="tools-item ui-menu-item"
          data-tools-item="glossary.harvest.open" @click="dispatch('glossary.harvest.open')"
        >
          <span class="ui-row">
            <span>{{ t('command.glossary.harvest.open') }}</span>
            <!-- aura-allow-text: chord already formatted for the platform by formatChord. -->
            <span class="tools-chord">{{ chordOf('glossary.harvest.open') }}</span>
          </span>
        </button>
      </li>
      <li role="none">
        <button
          type="button"
          role="menuitemcheckbox"
          tabindex="-1"
          class="tools-item ui-menu-item"
          :aria-checked="panelHeadersShown"
          data-tools-item="layout.toggle_panel_headers"
          @click="dispatch('layout.toggle_panel_headers')"
        >
          <span class="ui-row">
            <span>{{ t('command.layout.toggle_panel_headers') }}</span>
            <span class="tools-switch ui-switch__track" :class="{ on: panelHeadersShown }" aria-hidden="true"></span>
          </span>
        </button>
      </li>
      <li v-if="panelHeadersSaveError !== null" role="none">
        <p class="tools-alert" role="alert">{{ tError(panelHeadersSaveError) }}</p>
      </li>
      <li v-if="currentMode === 'workspace'" role="none">
        <button
          type="button"
          role="menuitemcheckbox"
          tabindex="-1"
          class="tools-item ui-menu-item"
          :aria-checked="visibleDockPanels.has('panel.grid')"
          :disabled="panelSwitchDisabled('panel.grid')"
          :title="commandTitle('layout.toggle_grid')"
          data-tools-item="layout.toggle_grid"
          @click="dispatch('layout.toggle_grid')"
        >
          <span class="ui-row">
            <span>{{ t('panel.grid.title') }}</span>
            <span class="ui-switch__track" aria-hidden="true"></span>
          </span>
        </button>
      </li>
      <li v-if="currentMode === 'workspace'" role="none">
        <button
          type="button"
          role="menuitemcheckbox"
          tabindex="-1"
          class="tools-item ui-menu-item"
          :aria-checked="visibleDockPanels.has('panel.lookup')"
          :disabled="panelSwitchDisabled('panel.lookup')"
          :title="commandTitle('layout.toggle_lookup')"
          data-tools-item="layout.toggle_lookup"
          @click="dispatch('layout.toggle_lookup')"
        >
          <span class="ui-row">
            <span>{{ t('panel.lookup.title') }}</span>
            <span class="ui-switch__track" aria-hidden="true"></span>
          </span>
        </button>
      </li>
      <li v-if="currentMode === 'workspace'" role="none">
        <button
          type="button"
          role="menuitemcheckbox"
          tabindex="-1"
          class="tools-item ui-menu-item"
          :aria-checked="visibleDockPanels.has('panel.ai_translation')"
          :disabled="panelSwitchDisabled('panel.ai_translation')"
          :title="commandTitle('layout.toggle_ai_translation')"
          data-tools-item="layout.toggle_ai_translation"
          @click="dispatch('layout.toggle_ai_translation')"
        >
          <span class="ui-row">
            <span>{{ t('panel.ai_translation.title') }}</span>
            <span class="ui-switch__track" aria-hidden="true"></span>
          </span>
        </button>
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
  min-width: calc(var(--space-unit) * 32);
}

.tools-item > .ui-row {
  flex: 1;
  justify-content: space-between;
}

.tools-alert {
  margin: 0;
  padding: var(--space-unit) calc(var(--space-unit) * 2);
  color: var(--color-error);
}

.tools-chord {
  color: var(--color-on-surface-variant);
}
</style>
