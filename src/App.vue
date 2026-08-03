<script setup lang="ts">
// Story 1.2 dựng một cửa sổ trống có chủ ý. Bốn panel và dockview thuộc Story 1.14;
// token màu/chữ thuộc Story 1.4; chuỗi giao diện thuộc Story 1.5.
//
// ⛔ Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — áp từ dòng code đầu tiên.
// Mọi văn bản hiển thị sống ở `src/i18n/vi.json`.
//
// Self-check phạm vi asset protocol (Kiểm 3 của Task 8) chạy khi bật cờ
// `VITE_SCOPE_SELFTEST=1`; xem `src/selftest/scopeCheck.ts`.
import { onMounted, ref } from 'vue'
import type { ScopeCheckReport } from './selftest/scopeCheck'

const report = ref<ScopeCheckReport | null>(null)
const selftestEnabled = import.meta.env.VITE_SCOPE_SELFTEST === '1'

onMounted(async () => {
  if (!selftestEnabled) return
  const { runScopeCheck } = await import('./selftest/scopeCheck')
  report.value = await runScopeCheck()
})
</script>

<template>
  <main class="shell">
    <pre v-if="report" class="selftest" :data-verdict="report.verdict">{{ report.text }}</pre>
  </main>
</template>

<style scoped>
.shell {
  min-height: 100vh;
  margin: 0;
}

.selftest {
  padding: 1rem;
  font-family: ui-monospace, monospace;
  font-size: 0.8125rem;
  white-space: pre-wrap;
}
</style>
