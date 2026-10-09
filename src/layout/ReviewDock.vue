<script setup lang="ts">
import { onBeforeUnmount } from 'vue'
import { DockviewVue } from 'dockview-vue'
import type { DockviewReadyEvent, VueComponent } from 'dockview-vue'
import { enterFocus, focusRegistry } from '../commands'
import PanelTab from '../panels/PanelTab.vue'
import ReviewCopyPanel from '../panels/ReviewCopyPanel.vue'
import ReviewMinePanel from '../panels/ReviewMinePanel.vue'
import { isUserActivation } from './dockController'

const MINE = 'panel.review_mine'
const COPY = 'panel.review_copy'
const FOCUS_RETRY_FRAMES = 10

const auraTheme = {
  name: 'aura',
  className: 'dockview-theme-aura',
  gap: 0,
  tabGroupIndicator: 'none',
} as const

const components = {
  [MINE]: ReviewMinePanel,
  [COPY]: ReviewCopyPanel,
} as unknown as Record<string, VueComponent>
const tabComponents = { aura: PanelTab } as unknown as Record<string, VueComponent>

const disposables: { dispose: () => void }[] = []
let focusFrame: number | null = null

function focusMineWhenMounted(framesLeft: number): void {
  focusFrame = requestAnimationFrame(() => {
    focusFrame = null
    if (focusRegistry.has(MINE)) {
      enterFocus(MINE)
      return
    }
    if (framesLeft > 0) focusMineWhenMounted(framesLeft - 1)
    else console.error('[review] panel.review_mine chua khai diem vao focus -- focus khong doi duoc.')
  })
}

function onReady(event: DockviewReadyEvent): void {
  const api = event.api
  api.addPanel({
    id: MINE,
    component: MINE,
    tabComponent: 'aura',
    params: { titleKey: 'panel.review_mine.title' },
  } as Parameters<typeof api.addPanel>[0])
  api.addPanel({
    id: COPY,
    component: COPY,
    tabComponent: 'aura',
    params: { titleKey: 'panel.review_copy.title' },
    position: { referencePanel: MINE, direction: 'right' },
  } as Parameters<typeof api.addPanel>[0])
  disposables.push(
    api.onDidActivePanelChange((e) => {
      if (!isUserActivation(e)) return
      const id = e.panel?.id
      if (id !== undefined) void enterFocus(id)
    }),
  )
  focusMineWhenMounted(FOCUS_RETRY_FRAMES)
}

onBeforeUnmount(() => {
  if (focusFrame !== null) cancelAnimationFrame(focusFrame)
  for (const d of disposables) d.dispose()
  disposables.length = 0
})
</script>

<template>
  <div class="dock-host" data-review-dock>
    <DockviewVue
      class="dock dockview-theme-aura"
      :theme="auraTheme"
      :components="components"
      :tab-components="tabComponents"
      single-tab-mode="fullwidth"
      @ready="onReady"
    />
  </div>
</template>

<style scoped>
.dock-host {
  display: flex;
  flex: 1;
  min-height: 0;
  min-width: 0;
}

.dock-host > :deep(.dock) {
  flex: 1;
  min-width: 0;
  min-height: 0;
}
</style>
