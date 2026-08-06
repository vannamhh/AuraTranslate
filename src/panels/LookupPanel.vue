<script setup lang="ts">
// Panel `Tra cứu`. Story 1.14 · AC1 · AC8 — **khung**, ⛔ không phải nội dung.
//
// ⛔ Bản ghi có cấu trúc (đầu mục, nghĩa, ví dụ, nhãn nguồn từ điển) là **Story 1.17**;
// đường tra cứu phía Rust đã có từ Story 1.11–1.13 nhưng story này ⛔ không chạm tới nó.
//
// ⚠️ UX-DR15 xếp `Tra cứu` vào nhóm ĐƯỢC nhường chỗ — nhưng *"rút về thanh trạng thái,
// ⛔ không bao giờ mất hẳn"*. Vế "thanh trạng thái" là **Story 4.12**; story này chỉ giao
// cơ chế ẩn (`SACRIFICE_ORDER` ở `src/layout/workspaceLayout.ts`).
import PanelFrame from './PanelFrame.vue'
import type { DockviewPanelProps } from '../layout/panelProps'

defineProps<DockviewPanelProps>()
</script>

<template>
  <PanelFrame owner="panel.lookup" status-key="panel.lookup.status" />
</template>
