<script setup lang="ts">
// Thanh tiêu đề panel = **tab của dockview**. Story 1.14 · §Quyết định #4A · UX-DR17 ·
// AC1 · AC8 · AC10 · `mockups/key-screen-workspace.html:31-34`.
//
// ─────────────────────────────────────────────────────────────────────────────────
// VÌ SAO MỘT `tabComponents` RIÊNG THAY VÌ TAB MẶC ĐỊNH CỦA DOCKVIEW
// ─────────────────────────────────────────────────────────────────────────────────
// Tab mặc định render `title` — một CHUỖI ĐÃ DỊCH — và kèm một nút đóng. Cả hai đều sai
// ở đây: chuỗi đã dịch nghĩa là nó phải đi qua `AddPanelOptions.title`, tức một chỗ ngoài
// `.vue` mang văn bản hiển thị; và "đóng panel" không phải thao tác của story này —
// ẩn/hiện đi qua bốn command `layout.toggle_*` (AC3).
//
// ⇒ Tab của ta nhận một **khoá** `vi.json` qua `params.titleKey` và tự gọi `t()`. Văn bản
// hiển thị ở lại đúng một chỗ (NFR16, AD-21).
//
// ⚠️ Hình dạng prop là của dockview-vue, không phải lựa chọn: `VueRenderer.init()`
// mount component với ĐÚNG MỘT prop tên `params`, và bên trong nó mới là
// `{ params, api, containerApi, tabLocation }`. Đọc từ
// `node_modules/dockview-vue/dist/dockview-vue.es.js:116-125`, không từ trí nhớ.
import { onBeforeUnmount, ref } from 'vue'
import { t } from '../i18n'

/** Chỉ hai mảnh của `DockviewPanelApi` mà tab này đọc — không kéo cả kiểu vào vỏ. */
type TabPanelApi = {
  readonly isActive: boolean
  readonly isGroupActive: boolean
  readonly onDidActiveChange: (listener: () => void) => { dispose: () => void }
  readonly onDidActiveGroupChange: (listener: () => void) => { dispose: () => void }
}

const props = defineProps<{
  params: {
    params: { titleKey?: string }
    api: TabPanelApi
  }
}>()

const api = props.params.api

/**
 * 🔴 HAI cờ, không một.
 *
 * dockview phân biệt *"tab nào đang hiện trong group này"* (`isActive`) với *"group này có
 * phải group đang hoạt động không"* (`isGroupActive`). Một panel gộp tab với panel khác
 * mà group của nó không hoạt động thì tab của nó vẫn `isActive` — nhấn mạnh nó bằng
 * `primary` là nói với người dùng rằng đó là chỗ họ đang đứng, trong khi không phải.
 *
 * ⚠️ Đây là mệnh đề về **tab đang chọn**, KHÔNG phải về focus DOM. Vạch tiêu điểm 2px
 * là chuyện của `PanelFrame` và nó đọc `document.activeElement` thật — xem doc-comment
 * `focused` ở đó. Hai thứ tách nhau, và cố ý.
 */
const current = ref(api.isActive && api.isGroupActive)
const sync = (): void => {
  current.value = api.isActive && api.isGroupActive
}

// `Event<T>` của dockview là một hàm nhận listener và trả `IDisposable`. Không gỡ thì
// mỗi lượt ẩn/hiện panel (AC3 — `removePanel` rồi `addPanel`) để lại một listener sống
// trỏ vào một component đã tháo.
const subs = [api.onDidActiveChange(sync), api.onDidActiveGroupChange(sync)]
onBeforeUnmount(() => {
  for (const s of subs) s.dispose()
})
</script>

<template>
  <!--
    ⚠️ `<span>` chứ không `<button>`: phần tử này nằm BÊN TRONG `.dv-tab` của dockview,
    và chính `.dv-tab` mới là thứ nhận chuột, kéo–thả và thứ tự Tab. Lồng một `<button>`
    vào trong đó là dựng hai đích bấm chồng nhau cho cùng một thao tác.

    Và vì vậy KHÔNG có `@click` ở đây — thao tác chọn tab thuộc về dockview
    (`EXPERIENCE.md:21`: dock, undock, gộp tab, đổi kích thước là năng lực SẴN CÓ của nó,
    không tự viết lại). Kiểm A của `check-commands.mjs` canh mọi `@click` phải là một
    `dispatch()`; không thêm một cái ở đây chỉ để "cho đúng luật".

    🔴 Dời focus DOM khi đổi panel (AD-34 §2) đi qua `onDidActivePanelChange` ở
    `WorkspaceDock.vue` — MỘT chỗ nghe cho cả bốn tab, thay vì bốn handler rời.
  -->
  <span class="tab" :class="{ current }">{{ t(props.params.params.titleKey ?? '') }}</span>
</template>

<style scoped>
/*
 * Thanh 34px của UX-DR17. `--space-head-height` là cùng token mà
 * `--dv-tabs-and-actions-container-height` đọc (`src/layout/dockview-theme.css`), nên
 * chiều cao tab và chiều cao thanh không thể trôi khỏi nhau.
 */
.tab {
  display: flex;
  align-items: center;
  height: var(--space-head-height);
  padding: 0 var(--space-panel-inline);
  white-space: nowrap;
  color: var(--color-on-surface-variant);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  font-weight: var(--weight-ui-md);
}

/*
 * Tab đang hiện của group đang hoạt động — `primary` + nét đậm.
 *
 * 🔴 `var(--weight-ui-md-strong)` là token thứ 15, thêm ở Story 1.14 · AC10 để đóng
 * `deferred-work.md:138`. Trước nó, ba chỗ trong dự án phải MƯỢN `--weight-read-title`
 * (một token của tiêu đề Chương, 23px họ `read`) chỉ để lấy con số 600 — và rủi ro đã
 * ghi: *"`--weight-read-title` đổi giá trị thì các chỗ này đổi theo mà không ai biết."*
 * Đừng viết thẳng `600`: Kiểm B2 của `check-tokens.mjs` đỏ, và khai một biến CSS cục
 * bộ để lách nó là đúng thứ AD-34 tồn tại để chặn.
 */
.tab.current {
  color: var(--color-primary);
  font-weight: var(--weight-ui-md-strong);
}
</style>
