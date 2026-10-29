<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, onUpdated, ref, useTemplateRef, watch } from 'vue'
import AppIcon from '../AppIcon.vue'
import LookupRecord from '../panels/LookupRecord.vue'
import { dispatch } from '../commands'
import { t, tError } from '../i18n'
import { tokens } from '../tokens'
import { enabledSourceCount } from '../panels/dictSourcesState'
import {
  aimLookupCharFrom,
  currentQuery,
  layerSensesFailed,
  lookupDisplayable,
  queryWasTruncated,
  someLayerFailed,
  someLayerTruncated,
} from '../panels/lookupPanelState'
import { aimLookupEntryFrom, pinNoticeKey, pinWriteError } from '../panels/lookupHistoryState'
import {
  lookupBody,
  lookupGroups as groups,
  lookupHeadPinKey as headPinKey,
  lookupHeadPinned as headPinned,
  lookupHeadReading as headReading,
  lookupNotFoundChars as notFoundChars,
  lookupSensesFor as sensesFor,
} from '../panels/lookupView'
import { readingLookupAnchor, readingLookupAwaitingAnswer } from './readingLookupState'

const props = defineProps<{
  /** The scrolling `section.mode`; the popup is positioned inside its content box. */
  host: HTMLElement | null
}>()

const popup = useTemplateRef<HTMLElement>('popup')
const GAP_PX = Number.parseFloat(tokens.spacing.unit)

type Placement = { top: number; left: number; maxHeight: number }
const placement = ref<Placement | null>(null)

const placementStyle = computed(() => {
  const p = placement.value
  const visibility: 'hidden' | undefined = readingLookupAwaitingAnswer.value ? 'hidden' : undefined
  if (p === null) return { visibility }
  return { top: `${p.top}px`, left: `${p.left}px`, maxHeight: `${p.maxHeight}px`, visibility }
})

/** Below the selection when it fits, above when the space above is larger; never past the host's visible box. */
function place(): void {
  const el = popup.value
  const host = props.host
  const anchor = readingLookupAnchor.value
  if (el === null || host === null || anchor === null) return
  const natural = el.scrollHeight + (el.offsetHeight - el.clientHeight)
  const viewTop = host.scrollTop
  const viewBottom = viewTop + host.clientHeight
  const below = viewBottom - anchor.bottom - GAP_PX
  const above = anchor.top - GAP_PX - viewTop
  const flip = natural > below && above > below
  const maxHeight = Math.max(flip ? above : below, 0)
  const top = flip ? anchor.top - GAP_PX - Math.min(natural, maxHeight) : anchor.bottom + GAP_PX
  const maxLeft = host.scrollLeft + host.clientWidth - el.offsetWidth
  const left = Math.max(host.scrollLeft, Math.min(anchor.left, maxLeft))
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

watch(readingLookupAnchor, () => {
  void nextTick(place)
})

function onDocumentMouseDown(event: MouseEvent): void {
  const target = event.target
  if (target instanceof Node && popup.value?.contains(target) === true) return
  dispatch('reading.lookup_close')
}

function onDocumentKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape') return
  dispatch('reading.lookup_close')
}

function onWindowResize(): void {
  dispatch('reading.lookup_close')
}

onMounted(() => {
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
  if (active !== null && popup.value?.contains(active) === true) props.host?.focus({ preventScroll: true })
})
</script>

<template>
  <div
    ref="popup"
    class="reading-lookup"
    role="dialog"
    :aria-label="t('mode.reading.lookup_dialog')"
    data-reading-lookup
    :style="placementStyle"
    @mousedown="aimLookupEntryFrom($event)"
  >
    <div v-if="currentQuery !== null" class="rl-head">
      <!-- aura-allow-text: the looked-up word is user-selected DATA. -->
      <p class="rl-headword" data-reading-lookup-headword>{{ currentQuery }}</p>
      <!-- aura-allow-text: shared reading of every entry that has one — dictionary DATA. -->
      <span v-if="headReading !== null" class="rl-reading">{{ headReading }}</span>
      <button
        v-if="lookupDisplayable && groups.length > 0"
        type="button"
        class="ui-icon-btn rl-pin"
        data-reading-lookup-pin
        :title="t('panel.lookup.head_pin_hint')"
        :aria-label="t(headPinKey)"
        :aria-pressed="headPinned"
        @click="dispatch('lookup.toggle_pin')"
      >
        <AppIcon :name="headPinned ? 'pin-on' : 'pin'" />
      </button>
    </div>

    <p v-if="pinNoticeKey !== null" class="rl-note">{{ t(pinNoticeKey) }}</p>
    <p v-if="pinWriteError !== null" class="rl-note">{{ tError(pinWriteError) }}</p>

    <p v-if="lookupBody === 'error'" class="rl-note" data-reading-lookup-error>{{ t('panel.lookup.lookup_failed') }}</p>
    <template v-else>
      <p v-if="someLayerFailed" class="rl-note">{{ t('panel.lookup.some_layer_failed') }}</p>
      <p v-if="someLayerTruncated" class="rl-note">{{ t('panel.lookup.list_incomplete') }}</p>
      <p v-if="queryWasTruncated" class="rl-note">{{ t('panel.lookup.query_truncated') }}</p>

      <p v-if="lookupBody === 'no_layers'" class="rl-note">{{ t('panel.lookup.no_layers') }}</p>
      <p v-else-if="lookupBody === 'too_short'" class="rl-note">{{ t('panel.lookup.query_too_short') }}</p>
      <p v-else-if="lookupBody === 'all_sources_off'" class="rl-note">
        {{ t('panel.lookup.all_sources_off') }}
      </p>
      <template v-else-if="lookupBody === 'not_found'">
        <!-- aura-allow-text: not-found title — the query is user DATA passed to t(). -->
        <p class="rl-title" data-reading-lookup-not-found>{{ t('panel.lookup.not_found', { query: currentQuery ?? '' }) }}</p>
        <p class="rl-note">{{ t('panel.lookup.not_found_checked', { n: String(enabledSourceCount) }) }}</p>
        <div v-if="notFoundChars.length > 0" class="rl-chars" @mousedown="aimLookupCharFrom($event)">
          <span class="rl-chars-label">{{ t('panel.lookup.lookup_each') }}</span>
          <button
            v-for="char in notFoundChars"
            :key="char"
            type="button"
            class="rl-char"
            :data-lookup-char="char"
            @click="dispatch('lookup.lookup_char')"
          >
            <!-- aura-allow-text: a character of the query — user DATA. -->
            {{ char }}
          </button>
        </div>
      </template>
      <template v-else-if="lookupBody === 'results'">
        <LookupRecord
          v-for="group in groups"
          :key="`${group.layer}:${group.source.code}`"
          :group="group"
          :senses="sensesFor(group)"
          :senses-failed="layerSensesFailed(group.layer)"
        />
      </template>

      <button
        v-if="currentQuery !== null"
        type="button"
        class="rl-concordance"
        data-reading-lookup-concordance
        @click="dispatch('reading.lookup_concordance')"
      >{{ t('mode.reading.lookup_concordance') }}</button>
    </template>
  </div>
</template>

<style scoped>
.reading-lookup {
  position: absolute;
  /* aura-allow-z-index: the popup floats over the reading page and the absolutely placed mark chips (z-index 1). */
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

.rl-head {
  display: flex;
  align-items: baseline;
  gap: var(--space-panel-inline);
  margin: 0 0 var(--space-panel-block) 0;
}

.rl-headword {
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

.rl-reading {
  flex: none;
  font-family: var(--face-lookup-gloss);
  font-size: var(--font-lookup-gloss);
  line-height: var(--leading-lookup-gloss);
  color: var(--color-on-surface-variant);
}

.rl-pin {
  margin-left: auto;
}

.rl-note {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.rl-title {
  margin: 0 0 var(--space-unit) 0;
  font-family: var(--face-ui-md-strong);
  font-size: var(--font-ui-md-strong);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-md-strong);
  color: var(--color-on-surface);
}

.rl-chars {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: var(--space-panel-inline);
  margin: 0 0 var(--space-panel-block) 0;
}

.rl-chars-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.rl-char,
.rl-concordance {
  appearance: none;
  border: none;
  background: none;
  padding: 0;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
}

.rl-char {
  font-family: var(--face-lookup-gloss);
  font-size: var(--font-lookup-gloss);
  line-height: var(--leading-lookup-gloss);
  color: var(--color-on-surface);
}

.rl-concordance {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.rl-char:focus-visible,
.rl-concordance:focus-visible {
  outline: none;
  border-bottom-color: var(--color-primary);
}
</style>
