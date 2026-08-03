/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<Record<string, never>, Record<string, never>, unknown>
  export default component
}

interface ImportMetaEnv {
  /** Bật self-check phạm vi asset protocol (Kiểm 3 của Story 1.2, Task 8). */
  readonly VITE_SCOPE_SELFTEST?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
