<script setup lang="ts">
// Chế độ 1/3 — kho Tác phẩm và **điểm vào ứng dụng** (PRD §5.2). Story 1.6 · AC3 · AC4.
//
// ⛔ Đây là một KHUNG RỖNG có chủ ý. Lưới Tác phẩm, bộ lọc, sắp xếp và bốn trạng thái
// vòng đời thuộc Epic 5; bốn trạng thái rỗng của UX-DR31 cần nội dung thật mới viết đúng
// được. Một câu trạng thái, đúng một câu.
//
// ⛔ Không chuỗi tiếng Việt nào trong tệp này (NFR16, AD-21) — mọi nhãn đi qua `t()`.
import { onActivated, onBeforeUnmount, onMounted, useTemplateRef } from 'vue'
import { declareFocus, enterFocus, releaseFocus } from '../commands'
import { t } from '../i18n'

const root = useTemplateRef<HTMLElement>('root')

onMounted(() => {
  declareFocus('mode.library', () => root.value)
})
onBeforeUnmount(() => {
  releaseFocus('mode.library')
})
// `onActivated` chứ không phải `onMounted`: ba chế độ sống trong `<KeepAlive>` (§Quyết
// định thiết kế #6), nên lần hiện thứ hai trở đi KHÔNG có `mounted`. Ở lượt mount đầu
// tiên Vue gọi `mounted` rồi `activated`, nên điểm vào đã khai xong trước khi dùng.
onActivated(() => {
  void enterFocus('mode.library')
})
</script>

<template>
  <!-- `tabindex="-1"` để phần tử nhận được focus lập trình. Nó KHÔNG vào thứ tự Tab. -->
  <section ref="root" class="mode" tabindex="-1">
    <p class="status">{{ t('mode.library.status') }}</p>
  </section>
</template>

<style scoped>
.mode {
  height: 100%;
  padding: var(--space-panel-block) var(--space-panel-inline);
}

/*
 * ⚠️ `outline: none` CHỈ ở đây, và chỉ vì phần tử này mang `tabindex="-1"`: nó là một
 * vùng chứa nhận focus lập trình, không phải một điều khiển tương tác. Chỉ báo tiêu điểm
 * thật của sản phẩm là vạch dọc ở `PanelFrame` (AC5).
 *
 * ⛔ Đừng nhân luật này ra thành `*:focus { outline: none }`. Đó là cách nhanh nhất phá
 * NFR17 (*"trạng thái focus luôn nhìn thấy rõ"*) mà vẫn qua được MỌI cổng hiện có —
 * `check-tokens.mjs` canh màu, cỡ chữ, tương phản, opacity và elevation, KHÔNG canh focus
 * ring. §Trap 4 của story.
 */
.mode:focus {
  outline: none;
}

.status {
  margin: 0;
  color: var(--color-on-surface-variant);
}
</style>
