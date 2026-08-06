<script setup lang="ts">
// Panel `Đề xuất AI`. Story 1.14 · AC1 · AC8 — **khung**, ⛔ không phải nội dung.
//
// ⛔ Bản dịch AI thật, chọn nhà cung cấp, và ba điểm ra mạng của AD-15 là **Epic 4**.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 "CHƯA CẤU HÌNH" ⛔ KHÔNG PHẢI MỘT TRẠNG THÁI LỖI — UX-DR27 · FR77
// ─────────────────────────────────────────────────────────────────────────────────
// Panel này **MỜI CẤU HÌNH**. ⛔ Không cảnh báo, ⛔ không màu `error`, ⛔ không dấu chấm
// than. Một người dùng chưa từng dán khoá API vào đâu thì ⛔ không làm sai gì cả — vẽ
// một cảnh báo ở đây là dạy họ rằng ứng dụng đang hỏng.
//
// ⚠️ Câu trạng thái sống ở `vi.json` (`panel.ai_translation.status`) và Kiểm D của
// `check-i18n.mjs` chấm phần máy chấm được của UX-DR47 (⛔ không "chúng tôi", ⛔ không
// "bạn"). Phần còn lại — giọng MỜI thay vì giọng CẢNH BÁO — là chỗ con người phải đọc.
import PanelFrame from './PanelFrame.vue'
import type { DockviewPanelProps } from '../layout/panelProps'

defineProps<DockviewPanelProps>()
</script>

<template>
  <PanelFrame owner="panel.ai_translation" status-key="panel.ai_translation.status" />
</template>
