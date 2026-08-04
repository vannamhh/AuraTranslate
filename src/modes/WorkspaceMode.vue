<script setup lang="ts">
// Chế độ 2/3 — môi trường dịch. Story 1.6 · AC3 · AC4 · AC5.
//
// ⛔ HAI `PanelFrame`, không phải bốn (§Câu hỏi cho Ice #2). Đúng cặp `Nguyên văn |
// Bản dịch` mà UX-DR15 nói *"không bao giờ nhường"*. Một cái không đủ để nhìn thấy tương
// phản có/không tiêu điểm — tức AC5 sẽ nghiệm thu bằng suy luận; bốn cái là dựng trước
// Story 1.14.
//
// ⛔ Không `dockview`, không lưới 2×2, không preset bố cục, không ngưỡng màn hình hẹp —
// Story 1.14 và 4.12.
import { onActivated, onBeforeUnmount, onMounted, useTemplateRef } from 'vue'
import { declareFocus, enterFocus, releaseFocus } from '../commands'
import { t } from '../i18n'
import PanelFrame from '../panels/PanelFrame.vue'

const root = useTemplateRef<HTMLElement>('root')

onMounted(() => {
  declareFocus('mode.workspace', () => root.value)
})
onBeforeUnmount(() => {
  releaseFocus('mode.workspace')
})
/**
 * ⚠️ Chế độ này vào focus ở GỐC chế độ, không nhảy thẳng vào một panel.
 *
 * Chọn panel nào là quyết định của Story 1.14, khi đã biết vòng gồm những gì và theo thứ
 * tự nào. Điều AC4 đòi hôm nay là focus **không rơi về `body`**, và gốc chế độ mang
 * `tabindex="-1"` nên nó nhận được focus thật. Đường vào panel là `focus.next_panel`.
 */
onActivated(() => {
  void enterFocus('mode.workspace')
})
</script>

<template>
  <section ref="root" class="mode" tabindex="-1">
    <p class="status">{{ t('mode.workspace.status') }}</p>
    <div class="panels">
      <PanelFrame owner="panel.source" title-key="panel.source.title" />
      <PanelFrame owner="panel.editor" title-key="panel.editor.title" />
    </div>
  </section>
</template>

<style scoped>
.mode {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  padding: var(--space-panel-block) var(--space-panel-inline);
}

/* Xem lý do đầy đủ ở `LibraryMode.vue` — chỉ gốc `tabindex="-1"`, ⛔ không `*:focus`. */
.mode:focus {
  outline: none;
}

.status {
  margin: 0 0 var(--space-panel-block);
  color: var(--color-on-surface-variant);
}

/*
 * `--panel-gap` là 0px ở theme sáng (phân tách bằng NÉT) và 2px ở theme tối (phân tách
 * bằng KHE, để `background` lộ ra). Một component không cần biết mình đang ở theme nào —
 * `applyTheme()` đã quyết. ⛔ Đừng thay bằng một khoảng cách viết thẳng: làm vậy là thống
 * nhất hai cơ chế về một cách làm, đúng thứ AC6 của Story 1.4 cấm.
 */
.panels {
  display: flex;
  flex: 1;
  min-height: 0;
  gap: var(--panel-gap);
  background-color: var(--color-background);
}

.panels > * {
  flex: 1;
  min-width: 0;
}
</style>
