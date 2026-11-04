<script setup lang="ts">
import { onBeforeUnmount, onMounted, useTemplateRef } from 'vue'
import { dispatch } from '../commands'
import { t } from '../i18n'
import ReadingMarksList from '../modes/ReadingMarksList.vue'
import PanelFrame from './PanelFrame.vue'
import { useSelectionSurface } from './selectionContract'

const surface = useTemplateRef<HTMLElement>('surface')
const list = useTemplateRef<InstanceType<typeof ReadingMarksList>>('list')
let frame: HTMLElement | null = null
const handOverFocus = (event: FocusEvent): void => {
  if (event.target === frame) list.value?.focus()
}

// The focus entry point is the frame root; the list keys only act on the list's own root.
onMounted(() => {
  frame = surface.value?.closest<HTMLElement>('section.panel') ?? null
  frame?.addEventListener('focus', handOverFocus)
})
onBeforeUnmount(() => {
  frame?.removeEventListener('focus', handOverFocus)
})
useSelectionSurface(surface, 'display')
</script>

<template>
  <PanelFrame owner="panel.reading_marks" status-key="panel.reading_marks.title" :show-status="false">
    <div ref="surface" class="marks-surface">
      <ReadingMarksList ref="list" :show-hint="false">
        <template #close>
          <button type="button" class="btn ui-btn" @click="dispatch('reading.marks_panel_close')">{{ t('mode.reading.marks_close') }}</button>
        </template>
      </ReadingMarksList>
    </div>
  </PanelFrame>
</template>

<style scoped>
.marks-surface {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
</style>
