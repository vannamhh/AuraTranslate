<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'
import { dispatch } from '../commands'
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
  document.addEventListener('mousedown', onDocumentMouseDown)
  root.value?.addEventListener('focusout', onFocusOut)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocumentMouseDown)
  root.value?.removeEventListener('focusout', onFocusOut)
})
</script>

<template>
  <div ref="root" class="sources-menu" @keydown="onKeydown($event)">
    <button
      ref="trigger"
      type="button"
      class="sources-trigger"
      aria-haspopup="menu"
      :aria-expanded="sourcesMenuIsOpen"
      data-sources-trigger
      @mousedown="focusOnPointerDown($event)"
      @click="dispatch('lookup.toggle_sources_menu')"
    >{{ t('panel.lookup.sources_button', { n: String(enabledSourceCount), m: String(dictSources.length) }) }}</button>

    <ul
      v-if="sourcesMenuIsOpen"
      class="sources-list"
      role="menu"
      :aria-label="t('panel.lookup.sources_menu')"
      :style="{ maxHeight: listMaxHeight }"
      @mousedown="aimDictSourceFrom($event)"
    >
      <li v-for="row in rows" :key="row.code" role="none">
        <button
          type="button"
          role="menuitemcheckbox"
          tabindex="-1"
          class="sources-item"
          :class="{ off: row.off }"
          :aria-checked="!row.off"
          :data-source-code="row.code"
          @click="dispatch('lookup.toggle_source')"
        >
          <span class="sources-switch" aria-hidden="true"></span>
          <!-- aura-allow-text: tên nguồn — DỮ LIỆU (`display_name` của chính tệp, FR31). -->
          <span class="sources-name">{{ row.name }}</span>
          <span v-if="row.off" class="sources-count">{{ t('panel.lookup.source_off') }}</span>
          <!-- aura-allow-text: số mục của chữ đang tra — DỮ LIỆU dẫn xuất. -->
          <span v-else-if="row.count !== null" class="sources-count">{{ row.count }}</span>
        </button>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.sources-menu {
  position: relative;
}

.sources-trigger {
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  white-space: nowrap;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-primary);
}

.sources-list {
  /* aura-allow-z-index: the list opens over the lookup head and body, which sit in their own stacking contexts. */
  z-index: 20;
  position: absolute;
  top: 100%;
  right: 0;
  min-width: calc(var(--space-unit) * 56);
  margin: var(--space-unit) 0 0 0;
  padding: var(--space-unit) 0;
  list-style: none;
  overflow-y: auto;
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
}

.sources-item {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 3);
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

.sources-item:hover,
.sources-item:focus-visible {
  background: var(--color-surface-sunken);
}

.sources-item.off .sources-name {
  color: var(--color-on-surface-variant);
  text-decoration: line-through;
}

/* The switch reads on/off by fill and by position, not by opacity. */
.sources-switch {
  flex: none;
  width: calc(var(--space-unit) * 4);
  height: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-primary);
  background: var(--color-primary);
}

.sources-item.off .sources-switch {
  border-color: var(--color-outline);
  background: none;
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
