/**
 * Hình dạng prop mà **dockview-vue** mount mọi component nội dung/tab bằng.
 * Story 1.14 · Task 2.
 *
 * ⚠️ Đo thật, không suy đoán: `VueRenderer.init()` dựng
 * `props = { params, api, containerApi, tabLocation }` rồi gọi `this.mount({ params: props })`
 * — tức component nhận **đúng một prop tên `params`**, và bên trong nó mới là bốn mảnh kia
 * (`node_modules/dockview-vue/dist/dockview-vue.es.js:116-125`).
 *
 * ⇒ Bốn panel khai kiểu này để hai chuyện xảy ra:
 *   1. `vue-tsc` biết `params` là một prop hợp lệ, nên nó không rơi xuống thành thuộc
 *      tính DOM `params="[object Object]"` trên phần tử gốc của panel;
 *   2. ngày dockview đổi hình dạng đó, **một** tệp đỏ chứ không phải bốn.
 *
 * Tệp này KHÔNG `import` gì cả — nó chỉ chở kiểu. `src/layout/workspaceLayout.ts`
 * (tệp mà `scripts/check-layout.mjs` nạp bằng Node thuần) không đọc tới nó, nên luật
 * "erasable-only" ở đó không bị kéo theo.
 */

/** Panel `params` do `AddPanelOptions.params` truyền xuống — story này chỉ dùng `titleKey`. */
export type PanelParams = {
  /** Khoá `vi.json` của tiêu đề panel. `PanelTab.vue` là chỗ nó đi qua `t()`. */
  titleKey?: string
}

/** Prop của một component **nội dung** panel. */
export type DockviewPanelProps = {
  params: {
    params: PanelParams
    [key: string]: unknown
  }
}
