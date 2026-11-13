<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'
import { dispatch } from '../commands'
import { holdTargetCaret } from '../commands/focus'
import AppIcon from '../AppIcon.vue'
import { t } from '../i18n'
import {
  aimDictSourceFrom,
  closeSourcesMenu,
  dictSources,
  enabledSourceCount,
  placeDictSource,
  sourceIsDisabled,
  sourcesMenuIsOpen,
} from './dictSourcesState'
import { groupedLookup, sourceEntryCount } from './lookupPanelState'

const root = useTemplateRef<HTMLElement>('root')
const trigger = useTemplateRef<HTMLButtonElement>('trigger')

// A long source list must not outgrow the panel: past its bottom edge it sits under the next dock group.
const listMaxHeight = ref<string | undefined>(undefined)
const listMaxWidth = ref<string | undefined>(undefined)

const EDGE_SCROLL_ZONE_PX = 24
const EDGE_SCROLL_STEP_PX = 12

const dragOrder = ref<string[] | null>(null)
const draggedCode = ref<string | null>(null)
let drag: { code: string; pointerId: number; clientY: number } | null = null
let scrollFrame = 0

watch(
  sourcesMenuIsOpen,
  (open) => {
    if (open) holdTargetCaret()
  },
  { flush: 'sync' },
)

watch(sourcesMenuIsOpen, async (open) => {
  if (!open) return
  await nextTick()
  const list = root.value?.querySelector<HTMLElement>('.sources-list')
  const body = root.value?.closest<HTMLElement>('.lookup-body')
  if (!list || !body) return
  const room = body.getBoundingClientRect().bottom - list.getBoundingClientRect().top
  listMaxHeight.value = room > 0 ? `${Math.floor(room)}px` : undefined
  const span = list.getBoundingClientRect().right - body.getBoundingClientRect().left
  listMaxWidth.value = span > 0 ? `${Math.floor(span)}px` : undefined
})

watch(sourcesMenuIsOpen, (open) => {
  if (!open) cancelDrag()
})

const rows = computed(() => {
  const order = dragOrder.value
  const sources = dictSources.value
  const shown = order === null ? sources : order.flatMap((code) => sources.find((src) => src.code === code) ?? [])
  return shown.map((src) => {
    const off = sourceIsDisabled(src.code)
    return { code: src.code, name: src.display_name, off, count: sourceEntryCount(groupedLookup.value, src, off) }
  })
})

function endDrag(list: HTMLElement | null): void {
  const ended = drag
  drag = null
  draggedCode.value = null
  cancelAnimationFrame(scrollFrame)
  scrollFrame = 0
  if (ended !== null && list?.hasPointerCapture(ended.pointerId)) list.releasePointerCapture(ended.pointerId)
}

function cancelDrag(): void {
  endDrag(root.value?.querySelector<HTMLElement>('.sources-list') ?? null)
  dragOrder.value = null
}

function onGripPointerDown(event: PointerEvent, code: string) {
  if (event.button !== 0 || drag !== null) return
  const list = root.value?.querySelector<HTMLElement>('.sources-list')
  if (!list) return
  event.preventDefault()
  drag = { code, pointerId: event.pointerId, clientY: event.clientY }
  draggedCode.value = code
  list.setPointerCapture(event.pointerId)
  dragOrder.value = dictSources.value.map((src) => src.code)
}

function edgeDirection(list: HTMLElement, clientY: number): -1 | 0 | 1 {
  const bounds = list.getBoundingClientRect()
  if (clientY < bounds.top + EDGE_SCROLL_ZONE_PX) return -1
  if (clientY > bounds.bottom - EDGE_SCROLL_ZONE_PX) return 1
  return 0
}

function retarget(list: HTMLElement, active: { code: string; clientY: number }) {
  const order = dragOrder.value
  if (order === null) return
  let target = 0
  for (const li of Array.from(list.querySelectorAll<HTMLElement>('.sources-row'))) {
    if (li.dataset.sourceRow === active.code) continue
    const rect = li.getBoundingClientRect()
    if (rect.top + rect.height / 2 < active.clientY) target++
  }
  const next = order.filter((code) => code !== active.code)
  next.splice(target, 0, active.code)
  if (next.some((code, i) => code !== order[i])) dragOrder.value = next
}

function scrollTick(list: HTMLElement) {
  scrollFrame = 0
  const active = drag
  if (active === null) return
  const direction = edgeDirection(list, active.clientY)
  if (direction === 0) return
  list.scrollTop += direction * EDGE_SCROLL_STEP_PX
  retarget(list, active)
  scrollFrame = requestAnimationFrame(() => scrollTick(list))
}

function onListPointerMove(event: PointerEvent) {
  const active = drag
  const list = event.currentTarget
  if (active === null || dragOrder.value === null || event.pointerId !== active.pointerId || !(list instanceof HTMLElement)) return
  active.clientY = event.clientY
  retarget(list, active)
  if (scrollFrame === 0 && edgeDirection(list, event.clientY) !== 0) scrollFrame = requestAnimationFrame(() => scrollTick(list))
}

function onListPointerUp(event: PointerEvent) {
  const active = drag
  const order = dragOrder.value
  if (active === null || event.pointerId !== active.pointerId) return
  endDrag(event.currentTarget instanceof HTMLElement ? event.currentTarget : null)
  dragOrder.value = null
  if (order !== null) placeDictSource(active.code, order.indexOf(active.code))
}

function onListPointerCancel(event: PointerEvent) {
  if (drag !== null && event.pointerId === drag.pointerId) cancelDrag()
}

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
    cancelDrag()
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
  cancelDrag()
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
      :class="{ 'is-dragging': draggedCode !== null }"
      role="menu"
      :aria-label="t('panel.lookup.sources_menu')"
      :style="{ maxHeight: listMaxHeight, maxWidth: listMaxWidth }"
      @mousedown="aimDictSourceFrom($event)"
      @pointermove="onListPointerMove($event)"
      @pointerup="onListPointerUp($event)"
      @pointercancel="onListPointerCancel($event)"
      @lostpointercapture="onListPointerCancel($event)"
    >
      <li v-for="row in rows" :key="row.code" role="none" class="sources-row" :class="{ dragged: row.code === draggedCode }" :data-source-row="row.code">
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
          <span class="sources-name" :title="row.name">{{ row.name }}</span>
          <span v-if="row.off" class="sources-count">{{ t('panel.lookup.source_off') }}</span>
          <!-- aura-allow-text: số mục của chữ đang tra — DỮ LIỆU dẫn xuất. -->
          <span v-else-if="row.count !== null" class="sources-count">{{ row.count }}</span>
        </button>
        <span
          class="sources-grip"
          aria-hidden="true"
          tabindex="-1"
          :title="t('panel.lookup.source_drag')"
          @pointerdown="onGripPointerDown($event, row.code)"
        >
          <AppIcon name="grip" />
        </span>
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
  align-items: center;
}

.sources-row.dragged {
  background: var(--color-outline);
}

.sources-list.is-dragging .sources-grip {
  cursor: grabbing;
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
  overflow: hidden;
  text-overflow: ellipsis;
}

.sources-grip {
  display: flex;
  flex: none;
  align-items: center;
  justify-content: center;
  align-self: stretch;
  width: calc(var(--space-unit) * 7);
  color: var(--color-on-surface-variant);
  cursor: grab;
  touch-action: none;
}

.sources-count {
  flex: none;
  color: var(--color-on-surface-variant);
}
</style>
