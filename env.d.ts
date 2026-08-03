/// <reference types="vite/client" />

// Shim dự phòng cho công cụ chưa bật Vue plugin. `vue-tsc` luôn ưu tiên tệp `.vue`
// thật nên shim này không che mất kiểu của component.
// ⚠️ Đừng siết thành `Record<string, never>`: mọi công cụ dùng `tsc` trần sẽ áp shim
// và báo lỗi ở mọi `<LookupPanel :segment="x" />` từ Story 1.14 trở đi.
declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

interface ImportMetaEnv {
  /** Bật self-check phạm vi asset protocol (Kiểm 3 của Story 1.2, Task 8). */
  readonly VITE_SCOPE_SELFTEST?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
