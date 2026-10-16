<script setup lang="ts">
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import type { AlignmentRow, AlignmentSegment } from './config/alignment'
import {
  alignmentActionError,
  alignmentActionUnavailable,
  alignmentBusy,
  alignmentCursor,
  alignmentData,
  alignmentEntries,
  alignmentFlow,
  endAlignmentChain,
  alignmentLoadError,
  alignmentMarkedRows,
  alignmentMarkedSegments,
  alignmentOverlayIsOpen,
  alignmentStatus,
  setAlignmentCursor,
} from './alignmentState'

let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

const cursorModel = computed<number>({
  get: () => alignmentCursor.value,
  set: (index) => setAlignmentCursor(index),
})

const segmentById = computed(() => new Map<number, AlignmentSegment>((alignmentData.value?.segments ?? []).map((s) => [s.id, s])))
const rowById = computed(() => new Map<number, AlignmentRow>((alignmentData.value?.rows ?? []).map((r) => [r.id, r])))
const groupById = computed(() => new Map((alignmentData.value?.groups ?? []).map((g) => [g.id, g])))

const pendingCount = computed(
  () => (alignmentData.value?.unmatched_segment_ids.length ?? 0) + (alignmentData.value?.unmatched_row_ids.length ?? 0),
)
const remainingAfterCursor = computed(() => Math.max(0, pendingCount.value - Math.min(alignmentCursor.value + 1, pendingCount.value)))
const markedCount = computed(() => alignmentMarkedSegments.value.length + alignmentMarkedRows.value.length)
const hasNextChapter = computed(() => alignmentFlow.value !== null && alignmentFlow.value.chapterIndex < alignmentFlow.value.chapterCount)

function onEscape(): void {
  endAlignmentChain()
  dispatch('export.alignment.close')
}


function rowLabelKey(rowId: number): string {
  const row = rowById.value.get(rowId)
  if (row?.kind === 'alt') return 'alignment.row_alt'
  if (row?.kind === 'caption') return 'alignment.row_caption'
  return 'alignment.side_row'
}

function groupLabelKey(groupId: number): string {
  const g = groupById.value.get(groupId)
  if (g === undefined) return 'alignment.group_machine'
  if (g.row_ids.length === 0) return 'alignment.group_segment_only'
  if (g.segment_ids.length === 0) return 'alignment.group_row_only'
  return g.decided_by === 'user' ? 'alignment.group_user' : 'alignment.group_machine'
}

watch(alignmentOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-alignment-open]')
    void nextTick(() => panel.value?.focus())
    return
  }
  const back = returnFocusTo
  returnFocusTo = null
  if (back !== null && back.isConnected) {
    back.focus()
    return
  }
  const opener = document.querySelector<HTMLElement>('[data-alignment-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[alignment] focus-return target is gone; focus falls back to body.')
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

/** No text field in this modal, so one root handler is safe. Escape and Tab have their own handlers. */
function onKeydown(event: KeyboardEvent): void {
  if (event.ctrlKey || event.metaKey || event.altKey) return

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      dispatch('export.alignment.next')
      return
    case 'ArrowUp':
      event.preventDefault()
      dispatch('export.alignment.prev')
      return
    case ' ':
      if (event.target instanceof HTMLButtonElement) return
      event.preventDefault()
      dispatch('export.alignment.toggle')
      return
    case 'Enter':
      if (event.target instanceof HTMLButtonElement) return
      event.preventDefault()
      dispatch('export.alignment.join')
      return
    case 's':
    case 'S':
      event.preventDefault()
      dispatch('export.alignment.skip')
      return
    case 'u':
    case 'U':
      event.preventDefault()
      dispatch('export.alignment.unjoin')
      return
    default:
      return
  }
}
</script>

<template>
  <div
    v-if="alignmentOverlayIsOpen"
    class="al-scrim"
    @keydown.esc="onEscape"
    @keydown.tab="trapTab($event)"
    @keydown="onKeydown"
  >
    <section ref="panel" class="al-panel" tabindex="-1" role="dialog" aria-modal="true" aria-labelledby="al-title">
      <header class="al-head">
        <h2 id="al-title" class="al-title">{{ t('alignment.title') }}</h2>
        <button type="button" class="al-close" @click="dispatch('export.alignment.close')">
          <!-- aura-allow-text: result of t() chosen in the template. -->
          {{ hasNextChapter ? t('alignment.next_chapter') : t('command.export.alignment.close') }}
        </button>
      </header>

      <p v-if="alignmentStatus === 'unknown'" class="al-status" role="status">{{ t('alignment.loading') }}</p>
      <p v-else-if="alignmentStatus === 'no_chapter'" class="al-empty">{{ t('alignment.no_chapter') }}</p>
      <p v-else-if="alignmentStatus === 'ipc_unavailable'" class="al-empty">{{ t('alignment.empty_ipc_unavailable') }}</p>
      <p v-else-if="alignmentStatus === 'error' && alignmentLoadError !== null" class="al-empty al-error" role="alert">
        <!-- aura-allow-text: result of tError() computed in the script. -->
        {{ tError(alignmentLoadError) }}
      </p>

      <template v-else-if="alignmentStatus === 'loaded' && alignmentData !== null">
        <!-- aura-allow-text: result of t() with the file name interpolated. -->
        <p class="al-note">{{ t('alignment.file_label', { file_name: alignmentData.file_name }) }}</p>
        <p v-if="alignmentFlow !== null" class="al-note">
          <!-- aura-allow-text: result of t() with the import numbers interpolated. -->
          {{
            t('alignment.flow_matched', {
              matched: String(alignmentFlow.matchedRows),
              total: String(alignmentFlow.totalRows),
              unresolved: String(alignmentFlow.unresolved),
            })
          }}
          <template v-if="alignmentFlow.chapterCount > 1">
            <!-- aura-allow-text: result of t() with the Chapter position interpolated. -->
            {{ t('alignment.flow_chapter', { index: String(alignmentFlow.chapterIndex), count: String(alignmentFlow.chapterCount) }) }}
          </template>
        </p>
        <p class="al-status" role="status">
          <!-- aura-allow-text: result of t() with the count interpolated. -->
          {{ alignmentData.is_resolved ? t('alignment.resolved') : t('alignment.unresolved', { count: String(pendingCount) }) }}
        </p>

        <div class="al-cols" aria-hidden="true">
          <span>{{ t('alignment.col_mine') }}</span>
          <span></span>
          <span>{{ t('alignment.col_reviewer') }}</span>
        </div>
        <p v-if="pendingCount === 0" class="al-note">{{ t('alignment.pending_empty') }}</p>
        <ul class="al-list">
          <template v-for="(entry, i) in alignmentEntries" :key="entry.kind + ':' + entry.id">
            <li class="al-row" :class="{ 'al-row-current': i === alignmentCursor, 'al-row-grouped': entry.kind === 'group' }">
              <label class="al-row-label">
                <input v-model="cursorModel" type="radio" name="al-row-cursor" class="al-sr-only" :value="i" />

                <template v-if="entry.kind === 'segment' && segmentById.get(entry.id) !== undefined">
                  <span class="al-cell">
                    <!-- aura-allow-text: data (segment text). -->
                    <span class="al-text">{{ segmentById.get(entry.id)?.target_text }}</span>
                    <span v-if="alignmentMarkedSegments.includes(entry.id)" class="al-mark" aria-hidden="true">✓</span>
                    <span v-if="alignmentMarkedSegments.includes(entry.id)" class="al-sr-only">{{ t('alignment.marked') }}</span>
                  </span>
                  <span class="al-link al-link-pending" aria-hidden="true">?</span>
                  <span class="al-cell al-cell-empty">{{ t('alignment.no_counterpart') }}</span>
                </template>

                <template v-else-if="entry.kind === 'row' && rowById.get(entry.id) !== undefined">
                  <span class="al-cell al-cell-empty">{{ t('alignment.no_counterpart') }}</span>
                  <span class="al-link al-link-pending" aria-hidden="true">?</span>
                  <span class="al-cell">
                    <span class="al-side">{{ t(rowLabelKey(entry.id)) }}</span>
                    <!-- aura-allow-text: data (reviewer row text). -->
                    <span class="al-text">{{ rowById.get(entry.id)?.target_text }}</span>
                    <span v-if="alignmentMarkedRows.includes(entry.id)" class="al-mark" aria-hidden="true">✓</span>
                    <span v-if="alignmentMarkedRows.includes(entry.id)" class="al-sr-only">{{ t('alignment.marked') }}</span>
                  </span>
                </template>

                <template v-else-if="entry.kind === 'group'">
                  <span class="al-cell">
                    <template v-for="segmentId in groupById.get(entry.id)?.segment_ids ?? []" :key="'s' + segmentId">
                      <!-- aura-allow-text: data (segment text). -->
                      <span class="al-text">{{ segmentById.get(segmentId)?.target_text }}</span>
                    </template>
                    <span v-if="(groupById.get(entry.id)?.segment_ids.length ?? 0) === 0" class="al-cell-empty">
                      {{ t('alignment.no_counterpart') }}
                    </span>
                  </span>
                  <span class="al-link" :title="t(groupLabelKey(entry.id))">→</span>
                  <span class="al-cell">
                    <template v-for="rowId in groupById.get(entry.id)?.row_ids ?? []" :key="'r' + rowId">
                      <!-- aura-allow-text: data (reviewer row text). -->
                      <span class="al-text">{{ rowById.get(rowId)?.target_text }}</span>
                    </template>
                    <span v-if="(groupById.get(entry.id)?.row_ids.length ?? 0) === 0" class="al-cell-empty">
                      {{ t('alignment.no_counterpart') }}
                    </span>
                  </span>
                </template>
              </label>
            </li>
          </template>
        </ul>

        <p v-if="alignmentActionError !== null" class="al-status al-error" role="alert">
          <!-- aura-allow-text: result of tError() computed in the script. -->
          {{ tError(alignmentActionError) }}
        </p>
        <p v-else-if="alignmentActionUnavailable" class="al-status" role="status">{{ t('alignment.action_unavailable') }}</p>
        <p v-else-if="alignmentBusy" class="al-status" role="status">{{ t('alignment.saving') }}</p>
        <div class="al-actions">
          <button type="button" class="al-act al-act-primary" @click="dispatch('export.alignment.join')">
            <!-- aura-allow-text: result of t() with the selection count interpolated. -->
            {{ markedCount > 0 ? t('alignment.join_n', { count: String(markedCount) }) : t('alignment.join_none') }}
          </button>
          <button type="button" class="al-act" @click="dispatch('export.alignment.skip')">
            {{ t('alignment.skip_this') }}
          </button>
          <button v-if="remainingAfterCursor > 0" type="button" class="al-act" @click="dispatch('export.alignment.next')">
            <!-- aura-allow-text: result of t() with the count interpolated. -->
            {{ t('alignment.see_remaining', { count: String(remainingAfterCursor) }) }}
          </button>
          <button type="button" class="al-act" @click="dispatch('export.alignment.toggle')">
            {{ t('command.export.alignment.toggle') }}
          </button>
          <button type="button" class="al-act" @click="dispatch('export.alignment.unjoin')">
            {{ t('command.export.alignment.unjoin') }}
          </button>
          <span v-if="pendingCount > 0" class="al-hint">
            <!-- aura-allow-text: result of t() with the position interpolated. -->
            {{ t('alignment.position_hint', { index: String(Math.min(alignmentCursor + 1, Math.max(pendingCount, 1))), count: String(pendingCount) }) }}
          </span>
        </div>
        <p class="al-hint">{{ t('alignment.hint_keys') }}</p>
      </template>
    </section>
  </div>
</template>

<style scoped>
.al-scrim {
  position: fixed;
  inset: 0;
  z-index: 11; /* aura-allow-z-index: same layer as the other full-screen modal scrims. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.al-panel {
  width: 100%;
  max-width: 960px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.al-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.al-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.al-close {
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

.al-status,
.al-note,
.al-hint {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.al-empty {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.al-error {
  color: var(--color-error);
}

.al-list {
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  max-height: 55vh;
  overflow: auto;
  border: 1px solid var(--color-outline);
}

.al-row {
  padding: calc(var(--space-unit) * 2) var(--space-panel-inline);
  border-bottom: 1px solid var(--color-outline);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  cursor: pointer;
}

.al-cols,
.al-row-label {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 3rem minmax(0, 1fr);
  align-items: baseline;
  gap: calc(var(--space-unit) * 3);
}

.al-cols {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.al-cell {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
}

.al-cell-empty {
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.al-link {
  text-align: center;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface-variant);
}

.al-link-pending {
  font-weight: var(--weight-ui-md);
  color: var(--color-on-surface);
}

.al-row-grouped {
  color: var(--color-on-surface-variant);
}

.al-row-current {
  background: var(--color-surface-accent);
}

.al-side {
  min-width: 8rem;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.al-text {
  font-family: var(--family-read);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
}

.al-mark {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
}

.al-sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  margin: -1px; /* aura-allow-spacing: geometry of the sr-only technique, not a grid spacing */
  padding: 0;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.al-actions {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
}

.al-act {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.al-act-primary {
  background: var(--color-primary);
  color: var(--color-on-primary);
  border-color: var(--color-primary);
}
</style>
