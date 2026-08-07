<script setup lang="ts">
// Panel `Bản dịch`. Story 1.14 · AC1 · AC8 — **khung**, không phải nội dung.
//
// Editor thật (vùng gõ, `SegmentVersion`, hợp đồng ghi AD-35, Sync Scrolling FR20) là
// **Epic 2**. Ở đây thân panel để trống có chủ ý.
//
// ⚠️ `panel.editor` thuộc cặp `Nguyên văn | Bản dịch` mà UX-DR15 nói **không bao giờ
// nhường** — nó nằm trong `NEVER_SACRIFICED` (`src/layout/workspaceLayout.ts`) và một cổng
// cưỡng chế rằng nó không bao giờ là đầu ra của `nextToSacrifice()` (AC7).
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 1.18 · AC2 — ĐĂNG KÝ HỢP ĐỒNG VÙNG CHỌN, KHÔNG NỘI DUNG
// ─────────────────────────────────────────────────────────────────────────────────
// Cùng lý do và cùng khuôn `AiTranslationPanel.vue`: bề mặt không có chữ hôm nay, nên một
// lượt đăng ký thiếu không để lại triệu chứng nào cho tới **Epic 2**. `epics.md:1762` đòi
// Editor *"nhận được cùng hành vi … không cần cài lại"*, và cổng Kiểm F giữ nó bằng máy.
import { useTemplateRef } from 'vue'
import PanelFrame from './PanelFrame.vue'
import { useSelectionSurface } from './selectionContract'
import type { DockviewPanelProps } from '../layout/panelProps'

defineProps<DockviewPanelProps>()

const surface = useTemplateRef<HTMLElement>('surface')
useSelectionSurface(surface, 'source')
</script>

<template>
  <PanelFrame owner="panel.editor" status-key="panel.editor.status">
    <div ref="surface" class="editor-surface"></div>
  </PanelFrame>
</template>

<style scoped>
/* KHÔNG nội dung — khung bề mặt cho hợp đồng vùng chọn (AC2). Epic 2 đổ chữ vào đây. */
.editor-surface {
  height: 100%;
  min-height: 0;
}
</style>
