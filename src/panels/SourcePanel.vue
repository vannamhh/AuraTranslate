<script setup lang="ts">
// Panel `Nguyên văn`. Story 1.14 · AC1 · AC8 — **khung**, ⛔ không phải nội dung.
//
// ⛔ Nội dung thật (nguyên văn tiếng Trung, tab Hán Việt, ba nhánh truy vấn) là
// **Story 1.16**. Ở đây chỉ có: một điểm vào focus đã khai, một tiêu đề trên tab, và một
// câu trạng thái nói rõ vì sao panel đang trống (UX-DR27).
//
// ⚠️ `owner` và `status-key` viết **literal**, ⛔ không qua biến: Kiểm E của
// `npm run check:commands` đọc tĩnh hai thuộc tính này để đối chiếu HAI CHIỀU với
// `FOCUS_OWNERS` và với `vi.json`. Một biểu thức ở đây bị đếm rồi bỏ qua — tức mất lưới.
import PanelFrame from './PanelFrame.vue'
import type { DockviewPanelProps } from '../layout/panelProps'

// dockview mount mọi component nội dung với đúng một prop tên `params`. Panel này ⛔ không
// đọc gì trong đó — khai ra để nó ⛔ không rơi xuống thành thuộc tính DOM `params="[object
// Object]"` trên phần tử gốc.
defineProps<DockviewPanelProps>()
</script>

<template>
  <PanelFrame owner="panel.source" status-key="panel.source.status" />
</template>
