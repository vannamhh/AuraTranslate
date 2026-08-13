<script setup lang="ts">
// Panel `Đề xuất AI`. Story 1.14 · AC1 · AC8 — **khung**, không phải nội dung.
//
// Bản dịch AI thật, chọn nhà cung cấp, và ba điểm ra mạng của AD-15 là **Epic 4**.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 "CHƯA CẤU HÌNH" KHÔNG PHẢI MỘT TRẠNG THÁI LỖI — UX-DR27 · FR77
// ─────────────────────────────────────────────────────────────────────────────────
// Panel này **MỜI CẤU HÌNH**. Không cảnh báo, không màu `error`, không dấu chấm
// than. Một người dùng chưa từng dán khoá API vào đâu thì không làm sai gì cả — vẽ
// một cảnh báo ở đây là dạy họ rằng ứng dụng đang hỏng.
//
// ⚠️ Câu trạng thái sống ở `vi.json` (`panel.ai_translation.status`) và Kiểm D của
// `check-i18n.mjs` chấm phần máy chấm được của UX-DR47 (không "chúng tôi", không
// "bạn"). Phần còn lại — giọng MỜI thay vì giọng CẢNH BÁO — là chỗ con người phải đọc.
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 1.18 · AC2 — ĐĂNG KÝ HỢP ĐỒNG VÙNG CHỌN, KHÔNG NỘI DUNG
// ─────────────────────────────────────────────────────────────────────────────────
// Panel này hôm nay **không có chữ**, và đó chính là lý do lượt đăng ký phải nằm ở đây NGAY
// BÂY GIỜ: `epics.md:1762` đòi AI Translation *"nhận được cùng hành vi khi nó có nội dung
// ở các epic sau, **không cần cài lại**"*. Một lượt đăng ký thiếu ở đây không để lại **bất
// kỳ triệu chứng nào** cho tới Epic 4 — tức hai epic sau, và tới lúc đó không ai nhớ AC này
// tồn tại. Cổng đếm của `check-commands.mjs` (Kiểm F) là thứ giữ mệnh đề đó bằng MÁY.
//
// Đừng "dọn" `<div ref="surface">` vì nó trông trống: nó LÀ bề mặt mà Epic 4 sẽ đổ nội
// dung vào, và là phần tử mà hợp đồng đo `contains(anchorNode)` trên.
//
// 🔵 **2026-08-13 — mệnh đề "cùng hành vi" ở trên đã ĐƯỢC THU HẸP** (Sprint Change Proposal,
// Ice ký; FR21). Panel này sẽ mang **bản dịch AI tiếng Việt**, còn từ điển nhúng là
// zh→vi / en→vi ⇒ nó KHÔNG phải nguồn tra cứu: vai nay là `'display'`.
// Phần còn đúng của AC2 — và là phần đắt nhất — vẫn nguyên: hợp đồng KHÔNG phải sửa một
// dòng nào khi Epic 4 đổ nội dung vào. Chỉ **vai** khai lúc đăng ký quyết định hành vi.
import { useTemplateRef } from 'vue'
import PanelFrame from './PanelFrame.vue'
import { useSelectionSurface } from './selectionContract'
import type { DockviewPanelProps } from '../layout/panelProps'

defineProps<DockviewPanelProps>()

const surface = useTemplateRef<HTMLElement>('surface')
// 🔴 ĐỪNG gỡ lời gọi này khi thấy vai là `'display'`. FR48 (Story 3.3) và FR60 (Story 7.7)
// đọc vùng chọn ở đây bằng lệnh của RIÊNG chúng; `'display'` tắt đúng MỘT đường —
// `currentSelectionText()`, tức đường tra TỪ ĐIỂN — chứ không tắt việc bề mặt được đăng ký.
// Ghim bằng máy: `check-commands.mjs` Kiểm F ③.
useSelectionSurface(surface, 'display')
</script>

<template>
  <PanelFrame owner="panel.ai_translation" status-key="panel.ai_translation.status">
    <div ref="surface" class="ai-surface"></div>
  </PanelFrame>
</template>

<style scoped>
/* KHÔNG nội dung — khung bề mặt cho hợp đồng vùng chọn (AC2). Epic 4 đổ chữ vào đây. */
.ai-surface {
  height: 100%;
  min-height: 0;
}
</style>
