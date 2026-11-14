<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, onUpdated, ref, useTemplateRef, watch } from 'vue'
import LookupRecord from './LookupRecord.vue'
import { t } from '../i18n'
import { tokens } from '../tokens'
import { enabledSourceCount, everySourceOffForRoute } from './dictSourcesState'
import { notFoundOf, queryTooShortOf } from './lookupPanelState'
import {
  closeLookupCharPopup,
  lookupCharPopupAnchor as anchor,
  lookupCharPopupChar as char,
  lookupCharPopupError as error,
  lookupCharPopupPending as pending,
  lookupCharPopupResponse as response,
} from './lookupCharPopupState'
import { headReadingOf, lookupBodyOf, sensesOfGroup } from './lookupView'

const props = defineProps<{
  /** The scrolling `.lookup-scroll`; the popup is positioned inside its content box. */
  host: HTMLElement | null
}>()

const popup = useTemplateRef<HTMLElement>('popup')
const GAP_PX = Number.parseFloat(tokens.spacing.unit)

type Placement = { top: number; left: number; maxHeight: number }
const placement = ref<Placement | null>(null)

const grouped = computed(() => response.value?.grouped ?? null)
const groups = computed(() => grouped.value?.groups ?? [])
const headReading = computed(() => headReadingOf(groups.value))

const body = computed(() => {
  const g = grouped.value
  const resolved = !pending.value && error.value === null && response.value !== null
  const route = g?.route ?? null
  return lookupBodyOf({
    error: error.value !== null,
    never: char.value === null,
    resolved,
    layersLoaded: g?.layers_loaded ?? true,
    tooShort: resolved && g !== null && queryTooShortOf(g),
    everySourceOff: route !== null && everySourceOffForRoute(route),
    notFound: resolved && g !== null && notFoundOf(g),
    displayable: error.value === null && response.value !== null,
  })
})

const placementStyle = computed(() => {
  const p = placement.value
  const visibility: 'hidden' | undefined = pending.value ? 'hidden' : undefined
  if (p === null) return { visibility }
  return { top: `${p.top}px`, left: `${p.left}px`, maxHeight: `${p.maxHeight}px`, visibility }
})

/** Below the button when it fits, above when the space above is larger; never past the host's visible box. */
function place(): void {
  const el = popup.value
  const host = props.host
  const button = anchor.value
  if (el === null || host === null || button === null) return
  const hostRect = host.getBoundingClientRect()
  const rect = button.getBoundingClientRect()
  const anchorTop = rect.top - hostRect.top - host.clientTop + host.scrollTop
  const anchorBottom = anchorTop + rect.height
  const anchorLeft = rect.left - hostRect.left - host.clientLeft + host.scrollLeft
  const natural = el.scrollHeight + (el.offsetHeight - el.clientHeight)
  const viewTop = host.scrollTop
  const viewBottom = viewTop + host.clientHeight
  const below = viewBottom - anchorBottom - GAP_PX
  const above = anchorTop - GAP_PX - viewTop
  const flip = natural > below && above > below
  const maxHeight = Math.max(flip ? above : below, 0)
  const top = flip ? anchorTop - GAP_PX - Math.min(natural, maxHeight) : anchorBottom + GAP_PX
  const maxLeft = host.scrollLeft + host.clientWidth - el.offsetWidth
  const left = Math.max(host.scrollLeft, Math.min(anchorLeft, maxLeft))
  const prev = placement.value
  if (
    prev !== null &&
    Math.abs(prev.top - top) < 0.5 &&
    Math.abs(prev.left - left) < 0.5 &&
    Math.abs(prev.maxHeight - maxHeight) < 0.5
  ) {
    return
  }
  placement.value = { top, left, maxHeight }
}

watch([anchor, pending, response], () => {
  void nextTick(place)
})

let openedFrom: HTMLElement | null = null

function focusIsUnplaced(button: HTMLElement): boolean {
  const active = document.activeElement
  if (active === null || active === document.body || active === button) return true
  return popup.value?.contains(active) === true || active.contains(button)
}

function restoreFocus(): void {
  const button = openedFrom
  if (button === null || !button.isConnected || !focusIsUnplaced(button)) return
  button.focus({ preventScroll: true })
}

function onDocumentMouseDown(event: MouseEvent): void {
  const target = event.target
  if (!(target instanceof Node)) return
  if (popup.value?.contains(target) === true) return
  if (anchor.value?.contains(target) === true) return
  const inPanel = target instanceof Element && target.closest('section.panel')?.contains(popup.value) === true
  const interactive =
    target instanceof Element && target.closest('button, a, input, select, textarea, [contenteditable]') !== null
  closeLookupCharPopup()
  // The browser moves focus after this handler; restoring once it has settled.
  if (inPanel && !interactive) setTimeout(restoreFocus, 0)
}

function onDocumentKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape' || event.isComposing) return
  const active = document.activeElement
  const scope = popup.value?.closest('section.panel') ?? props.host
  if (active !== null && active !== document.body && scope?.contains(active) !== true) return
  restoreFocus()
  closeLookupCharPopup()
  restoreFocus()
}

function onWindowResize(): void {
  closeLookupCharPopup()
}

onMounted(() => {
  openedFrom = anchor.value
  document.addEventListener('mousedown', onDocumentMouseDown, true)
  document.addEventListener('keydown', onDocumentKeydown)
  window.addEventListener('resize', onWindowResize)
  place()
})
onUpdated(place)
onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocumentMouseDown, true)
  document.removeEventListener('keydown', onDocumentKeydown)
  window.removeEventListener('resize', onWindowResize)
  const active = document.activeElement
  if (active !== null && popup.value?.contains(active) === true) restoreFocus()
  closeLookupCharPopup()
})
</script>

<template>
  <div
    ref="popup"
    class="lookup-char-popup"
    role="dialog"
    :aria-label="t('panel.lookup.char_dialog')"
    data-lookup-char-popup
    :style="placementStyle"
  >
    <div class="lcp-head">
      <!-- aura-allow-text: the looked-up character is DATA from the query. -->
      <p class="lcp-headword" data-lookup-char-popup-headword>{{ char }}</p>
      <!-- aura-allow-text: shared reading of every entry that has one — dictionary DATA. -->
      <span v-if="headReading !== null" class="lcp-reading">{{ headReading }}</span>
    </div>

    <p v-if="body === 'error'" class="lcp-note" data-lookup-char-popup-error>{{ t('panel.lookup.lookup_failed') }}</p>
    <p v-else-if="body === 'no_layers'" class="lcp-note">{{ t('panel.lookup.no_layers') }}</p>
    <p v-else-if="body === 'too_short'" class="lcp-note">{{ t('panel.lookup.query_too_short') }}</p>
    <p v-else-if="body === 'all_sources_off'" class="lcp-note">{{ t('panel.lookup.all_sources_off') }}</p>
    <template v-else-if="body === 'not_found'">
      <!-- aura-allow-text: not-found title — the character is user DATA passed to t(). -->
      <p class="lcp-title" data-lookup-char-popup-not-found>{{ t('panel.lookup.not_found', { query: char ?? '' }) }}</p>
      <p class="lcp-note">{{ t('panel.lookup.not_found_checked', { n: String(enabledSourceCount) }) }}</p>
    </template>
    <template v-else-if="body === 'results'">
      <LookupRecord
        v-for="group in groups"
        :key="`${group.layer}:${group.source.code}`"
        :group="group"
        :senses="sensesOfGroup(group, response?.senses_by_layer ?? {})"
        :senses-failed="(response?.senses_failed ?? []).includes(group.layer)"
      />
    </template>
  </div>
</template>

<style scoped>
.lookup-char-popup {
  position: absolute;
  /* aura-allow-z-index: the popup floats over the record blocks and the sticky content of the scroll box. */
  z-index: 2;
  inline-size: min(calc(var(--space-unit) * 57), 100%);
  overflow: auto;
  padding: var(--space-panel-block) var(--space-panel-inline);
  background: var(--color-surface);
  border: 1px solid var(--color-control-rule);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-popover);
  text-align: start;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface);
}

.lcp-head {
  display: flex;
  align-items: baseline;
  gap: var(--space-panel-inline);
  margin: 0 0 var(--space-panel-block) 0;
}

.lcp-headword {
  margin: 0;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--face-lookup-headword);
  font-size: var(--font-lookup-headword);
  line-height: var(--leading-lookup-headword);
  color: var(--color-on-surface);
}

.lcp-reading {
  flex: none;
  font-family: var(--face-lookup-gloss);
  font-size: var(--font-lookup-gloss);
  line-height: var(--leading-lookup-gloss);
  color: var(--color-on-surface-variant);
}

.lcp-note {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.lcp-title {
  margin: 0 0 var(--space-unit) 0;
  font-family: var(--face-ui-md-strong);
  font-size: var(--font-ui-md-strong);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-md-strong);
  color: var(--color-on-surface);
}
</style>
