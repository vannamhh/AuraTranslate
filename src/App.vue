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
// ⚠️ Import TĨNH có chủ ý, và chỉ hằng này thôi. `./selftest/eventName` là một module
// rỗng ngoài một chuỗi, nên nó vào bundle chính mà không tốn gì — còn `./selftest/
// scopeCheck` thì KHÔNG được import tĩnh: làm vậy là kéo cả mã self-check vào bản
// release, phá đúng bất biến mà `#[cfg(debug_assertions)]` phía Rust đang giữ.
import { SELFTEST_EVENT } from './selftest/eventName'

const report = ref<ScopeCheckReport | null>(null)
const selftestEnabled = import.meta.env.VITE_SCOPE_SELFTEST === '1'

onMounted(async () => {
  if (!selftestEnabled) return
  try {
    const { runScopeCheck } = await import('./selftest/scopeCheck')
    report.value = await runScopeCheck()
  } catch (err) {
    // Một rejection không bắt ở đây là lượt chạy treo: Rust chờ một event không bao
    // giờ tới. Phát FAIL tường minh thay vì im lặng — script bọc mới đọc được VERDICT.
    //
    // ⚠️ Và chính khối này cũng ném được, ở hai đường mà bản trước không bọc:
    //   1. nếu thứ gãy ban đầu LÀ import động (bundle lỗi, CSP chặn chunk) thì
    //      `import('@tauri-apps/api/event')` dưới đây cũng reject;
    //   2. nếu thứ reject là chính `emit` (IPC đã tụt xuống `postMessage`) thì ta gọi
    //      lại đúng cái vừa hỏng.
    // Cả hai đều cho một unhandled rejection trong `onMounted` ⇒ không event nào được
    // phát ⇒ script bọc treo tới timeout rồi báo "webview không mở được", trong khi
    // webview mở bình thường. Nên: bọc lần hai, và luôn còn `console.log` làm đường lui.
    const text = `AuraTranslate — asset protocol scope self-check (Story 1.2 AC3 · Story 1.3 AC8)\nmode:     undetermined  (self-check gãy trước khi chạy)\n\n[FAIL] self-check gãy trước khi chạy: ${String(err)}\n\nVERDICT: FAIL`
    console.log(text)
    try {
      const { emit } = await import('@tauri-apps/api/event')
      // `mode` là trường bắt buộc của `ScopeCheckReport` — thiếu nó thì phía Rust đọc
      // được `verdict` nhưng bảng chẩn đoán mất một cột.
      await emit(SELFTEST_EVENT, {
        verdict: 'FAIL',
        mode: 'undetermined',
        results: [],
        text,
      })
    } catch (emitErr) {
      // Không còn đường nào phát về Rust. Dòng `VERDICT: FAIL` ở trên đã ra stdout, và
      // đó chính là thứ `scripts/check-scope*.mjs` đọc — nên lượt chạy vẫn kết luận
      // được, chỉ là qua log thay vì qua mã thoát của ứng dụng.
      console.log(`[FAIL] không phát được event self-check về Rust: ${String(emitErr)}`)
    }
  }
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
