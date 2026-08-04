<script setup lang="ts">
// Chế độ 3/3 — đọc lại bản dịch đã hoàn thành, không có công cụ biên tập. Story 1.6 ·
// AC3 · AC4.
//
// ⛔ KHUNG RỖNG có chủ ý. Typography đọc dài, công tắc song ngữ, ba mức chữ, đánh dấu
// "cần sửa" — toàn bộ thuộc Epic 5 (UX-DR46). Một câu trạng thái, đúng một câu.
//
// ⚠️ Ngày Epic 5 đổ chữ THẬT vào đây, bề mặt này phải khai token `read-*` của chính nó.
// Mặc định của `body` là `ui-md` ở giãn dòng **1.5** — dưới sàn 1.66 của AC5 Story 1.4 —
// và không phép kiểm nào canh được chỗ đó (`deferred-work.md`, mục `body` chạy ở 1.5).
import { onActivated, onBeforeUnmount, onMounted, useTemplateRef } from 'vue'
import { declareFocus, enterFocus, releaseFocus } from '../commands'
import { t } from '../i18n'

const root = useTemplateRef<HTMLElement>('root')

onMounted(() => {
  declareFocus('mode.reading', () => root.value)
})
onBeforeUnmount(() => {
  releaseFocus('mode.reading')
})
onActivated(() => {
  void enterFocus('mode.reading')
})
</script>

<template>
  <section ref="root" class="mode" tabindex="-1">
    <p class="status">{{ t('mode.reading.status') }}</p>
  </section>
</template>

<style scoped>
.mode {
  height: 100%;
  padding: var(--space-panel-block) var(--space-panel-inline);
}

/* Xem lý do đầy đủ ở `LibraryMode.vue` — `outline: none` chỉ áp cho gốc `tabindex="-1"`,
   ⛔ không bao giờ áp cho `*:focus` (§Trap 4). */
.mode:focus {
  outline: none;
}

.status {
  margin: 0;
  color: var(--color-on-surface-variant);
}
</style>
