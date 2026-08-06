<script setup lang="ts">
// Panel `Bản dịch`. Story 1.14 · AC1 · AC8 — **khung**, ⛔ không phải nội dung.
//
// ⛔ Editor thật (vùng gõ, `SegmentVersion`, hợp đồng ghi AD-35, Sync Scrolling FR20) là
// **Epic 2**. Ở đây thân panel để trống có chủ ý.
//
// ⚠️ `panel.editor` thuộc cặp `Nguyên văn | Bản dịch` mà UX-DR15 nói ⛔ **không bao giờ
// nhường** — nó nằm trong `NEVER_SACRIFICED` (`src/layout/workspaceLayout.ts`) và một cổng
// cưỡng chế rằng nó ⛔ không bao giờ là đầu ra của `nextToSacrifice()` (AC7).
import PanelFrame from './PanelFrame.vue'
import type { DockviewPanelProps } from '../layout/panelProps'

defineProps<DockviewPanelProps>()
</script>

<template>
  <PanelFrame owner="panel.editor" status-key="panel.editor.status" />
</template>
