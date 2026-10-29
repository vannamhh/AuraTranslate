<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'
import { dispatch } from '../commands'
import AppIcon from '../AppIcon.vue'
import { t } from '../i18n'
import {
  aimDictSourceFrom,
  closeSourcesMenu,
  dictSources,
  enabledSourceCount,
  sourceIsDisabled,
  sourcesMenuIsOpen,
} from './dictSourcesState'
import { groupedLookup, sourceEntryCount } from './lookupPanelState'

const root = useTemplateRef<HTMLElement>('root')
const trigger = useTemplateRef<HTMLButtonElement>('trigger')

// A long source list must not outgrow the panel: past its bottom edge it sits under the next dock group.
const listMaxHeight = ref<string | undefined>(undefined)

watch(sourcesMenuIsOpen, async (open) => {
  if (!open) return
  await nextTick()
  const list = root.value?.querySelector<HTMLElement>('.sources-list')
  const body = root.value?.closest<HTMLElement>('.lookup-body')
  if (!list || !body) return
  const room = body.getBoundingClientRect().bottom - list.getBoundingClientRect().top
  listMaxHeight.value = room > 0 ? `${Math.floor(room)}px` : undefined
})

const rows = computed(() =>
  dictSources.value.map((src) => {
    const off = sourceIsDisabled(src.code)
    return { code: src.code, name: src.display_name, off, count: sourceEntryCount(groupedLookup.value, src, off) }
  }),
)
const lastRow = computed(() => rows.value.length - 1)

function menuItems(): HTMLElement[] {
  return Array.from(root.value?.querySelectorAll<HTMLElement>('[role="menuitemcheckbox"]') ?? [])
}

// WKWebView does not focus a <button> on mouse click; without this Esc has no focused opener to return to.
function focusOnPointerDown(event: MouseEvent) {
  const target = event.currentTarget
  if (target instanceof HTMLElement) target.focus()
}

function onKeydown(event: KeyboardEvent) {
  if (!sourcesMenuIsOpen.value) return
  if (event.key === 'Escape') {
    event.preventDefault()
    closeSourcesMenu()
    trigger.value?.focus()
    return
  }
  if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return
  if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey || event.defaultPrevented) return
  const items = menuItems()
  if (items.length === 0) return
  event.preventDefault()
  const at = items.findIndex((item) => item === document.activeElement)
  const step = event.key === 'ArrowDown' ? 1 : -1
  const next = at === -1 ? (step === 1 ? 0 : items.length - 1) : (at + step + items.length) % items.length
  items[next]?.focus()
}

function onDocumentMouseDown(event: MouseEvent) {
  if (!sourcesMenuIsOpen.value) return
  const target = event.target
  if (target instanceof Node && root.value?.contains(target)) return
  closeSourcesMenu()
}

function onFocusOut(event: FocusEvent) {
  const next = event.relatedTarget
  if (next instanceof Node && root.value !== null && !root.value.contains(next)) closeSourcesMenu()
}

onMounted(() => {
  document.addEventListener('mousedown', onDocumentMouseDown, true)
  root.value?.addEventListener('focusout', onFocusOut)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocumentMouseDown, true)
  root.value?.removeEventListener('focusout', onFocusOut)
})
</script>

<template>
  <div ref="root" class="sources-menu" @keydown="onKeydown($event)">
    <button
      ref="trigger"
      type="button"
      class="ui-btn sources-trigger"
      aria-haspopup="menu"
      :aria-expanded="sourcesMenuIsOpen"
      data-sources-trigger
      @mousedown="focusOnPointerDown($event)"
      @click="dispatch('lookup.toggle_sources_menu')"
    >
      <span class="ui-row">
        {{ t('panel.lookup.sources_button', { n: String(enabledSourceCount), m: String(dictSources.length) }) }}
        <AppIcon name="chevron" />
      </span>
    </button>

    <ul
      v-if="sourcesMenuIsOpen"
      class="sources-list ui-menu"
      role="menu"
      :aria-label="t('panel.lookup.sources_menu')"
      :style="{ maxHeight: listMaxHeight }"
      @mousedown="aimDictSourceFrom($event)"
    >
      <li v-for="(row, index) in rows" :key="row.code" role="none" class="sources-row">
        <button
          type="button"
          role="menuitemcheckbox"
          tabindex="-1"
          class="sources-item ui-menu-item"
          :class="{ off: row.off }"
          :aria-checked="!row.off"
          :data-source-code="row.code"
          @click="dispatch('lookup.toggle_source')"
        >
          <span class="sources-switch ui-switch__track" aria-hidden="true"></span>
          <!-- aura-allow-text: tên nguồn — DỮ LIỆU (`display_name` của chính tệp, FR31). -->
          <span class="sources-name">{{ row.name }}</span>
          <span v-if="row.off" class="sources-count">{{ t('panel.lookup.source_off') }}</span>
          <!-- aura-allow-text: số mục của chữ đang tra — DỮ LIỆU dẫn xuất. -->
          <span v-else-if="row.count !== null" class="sources-count">{{ row.count }}</span>
        </button>
        <button
          type="button"
          tabindex="-1"
          class="ui-icon-btn"
          :data-source-code="row.code"
          :disabled="index === 0"
          :aria-label="t('panel.lookup.source_move_up')"
          @click="dispatch('lookup.move_source_up')"
        >
          ↑
        </button>
        <button
          type="button"
          tabindex="-1"
          class="ui-icon-btn"
          :data-source-code="row.code"
          :disabled="index === lastRow"
          :aria-label="t('panel.lookup.source_move_down')"
          @click="dispatch('lookup.move_source_down')"
        >
          ↓
        </button>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.sources-menu {
  position: relative;
}

.sources-list {
  /* aura-allow-z-index: the list opens over the lookup head and body, which sit in their own stacking contexts. */
  z-index: 20;
  position: absolute;
  top: 100%;
  right: 0;
  min-width: calc(var(--space-unit) * 32);
  margin-top: var(--space-unit);
  overflow-y: auto;
}

.sources-row {
  display: flex;
  align-items: baseline;
}

.sources-row .sources-item {
  flex: 1 1 auto;
  min-width: 0;
}

.sources-item.off .sources-name {
  color: var(--color-on-surface-variant);
  text-decoration: line-through;
}

.sources-name {
  flex: 1 1 auto;
  min-width: 0;
}

.sources-count {
  flex: none;
  color: var(--color-on-surface-variant);
}
</style>
