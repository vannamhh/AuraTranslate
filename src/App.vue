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
// Cùng khuôn, cùng lý do an toàn bundle — nhưng vì AC2 của Story 1.5: hai chuỗi chẩn
// đoán dưới đây từng nằm nguyên văn trong template literal ở tệp này, và một chuỗi
// tiếng Việt trong `.vue` là vi phạm NFR16 mà `npm run check:i18n` bắt được.
// ⛔ Chúng KHÔNG thuộc `vi.json` — đọc doc-comment của `fallbackReport.ts`.
import { emitFailureLine, fallbackReportText } from './selftest/fallbackReport'
// ── Story 1.6 — vỏ một cửa sổ, ba chế độ ngang hàng (AD-24, AC3) ────────────────────
//
// ⚠️ `dispatch` là cửa DUY NHẤT mà một handler chuột được đi qua (AC1). Ba `@click` dưới
// đây là **đúng một** lời gọi `dispatch('<id>')`, và `scripts/check-commands.mjs` Kiểm A
// cưỡng chế điều đó bằng cú pháp trên mọi `.vue` của cây nguồn.
import { dispatch } from './commands'
import { currentMode } from './modes/modeState'
import { t } from './i18n'
import LibraryMode from './modes/LibraryMode.vue'
import WorkspaceMode from './modes/WorkspaceMode.vue'
import ReadingMode from './modes/ReadingMode.vue'

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
    const text = fallbackReportText(err)
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
      console.log(emitFailureLine(emitErr))
    }
  }
})
</script>

<template>
  <main class="shell">
    <header class="titlebar">
      <nav class="modes">
        <button
          type="button"
          class="mode-tab"
          :class="{ on: currentMode === 'library' }"
          @click="dispatch('mode.library')"
        >
          {{ t('command.mode.library') }}
        </button>
        <button
          type="button"
          class="mode-tab"
          :class="{ on: currentMode === 'workspace' }"
          @click="dispatch('mode.workspace')"
        >
          {{ t('command.mode.workspace') }}
        </button>
        <button
          type="button"
          class="mode-tab"
          :class="{ on: currentMode === 'reading' }"
          @click="dispatch('mode.reading')"
        >
          {{ t('command.mode.reading') }}
        </button>
      </nav>
    </header>

    <!--
      🔴 `<KeepAlive>` chứ KHÔNG `v-if` trần — §Quyết định thiết kế #6.

      UX-DR34 và FR12 hứa *"chuyển chế độ luôn giữ ngữ cảnh — rời Workspace sang Chế độ
      đọc rồi quay lại thì vẫn đúng Chương, đúng câu, đúng vị trí cuộn"*. Hôm nay chưa có
      ngữ cảnh nào để mất, nên hai cách cho kết quả quan sát được y hệt và mọi cổng đều
      xanh với cả hai. Khác biệt hiện ra ở Epic 2, khi Editor mang văn bản đang gõ và vị
      trí cuộn: lúc đó `v-if` huỷ component và mất chúng, và người sửa sẽ phải mổ lại vỏ
      chế độ mà không có gì nối được lỗi đó về story này. Chọn đúng ngay bây giờ tốn một
      cặp thẻ.
    -->
    <div class="modeport">
      <KeepAlive>
        <LibraryMode v-if="currentMode === 'library'" />
        <WorkspaceMode v-else-if="currentMode === 'workspace'" />
        <ReadingMode v-else />
      </KeepAlive>
    </div>

    <pre v-if="report" class="selftest" :data-verdict="report.verdict">{{ report.text }}</pre>
  </main>
</template>

<style scoped>
.shell {
  /* Story 1.6: vỏ MỘT cửa sổ hệ điều hành (AD-24). `height` chứ không `min-height` —
     ba chế độ chia nhau đúng chiều cao còn lại, không đẩy ra thanh cuộn. */
  display: flex;
  flex-direction: column;
  height: 100vh;
  margin: 0;
}

.titlebar {
  display: flex;
  align-items: center;
  flex: none;
  height: var(--space-titlebar-height);
  padding: 0 var(--space-panel-inline);
  border-bottom: 1px solid var(--color-outline);
}

.modes {
  display: flex;
  gap: calc(var(--space-unit) * 4);
}

/*
 * Ba tab chế độ. ⚠️ Chúng là `<button>` chứ không phải `<span>` như mockup vẽ: một tab
 * chế độ là thao tác, nên nó phải vào được thứ tự Tab và nhận được `Enter`/`Space` —
 * NFR17 nói *"mọi thao tác gọi được bằng bàn phím"*, và một `<span @click>` không gọi
 * được bằng bàn phím ở bất kỳ trình duyệt nào.
 *
 * ⛔ KHÔNG `outline: none` ở đây. `outline: none` chỉ áp cho gốc `tabindex="-1"` của
 * chế độ và panel (§Trap 4); nút bấm giữ nguyên focus ring của trình duyệt, vì đó chính
 * là nửa còn lại của NFR17.
 */
.mode-tab {
  padding: 0;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  color: var(--color-on-surface-variant);
  /* `<button>` KHÔNG kế thừa chữ của `body` — thiếu ba dòng này là ba tab chạy bằng font
     mặc định của hệ điều hành, đúng thứ Kiểm B2 của `check-tokens.mjs` tồn tại để chặn. */
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

/*
 * Tab đang hiện — theo `mockups/key-screen-workspace.html:23`. Chữ đậm màu `on-surface`
 * cộng một gạch chân 2px `primary`.
 *
 * ⚠️ Cùng lý do mượn trọng lượng như `PanelFrame.vue`: bộ token không có biến trọng
 * lượng cho nhãn giao diện đậm, và viết thẳng `600` thì Kiểm B2 đỏ. Đã mở mục
 * `deferred-work.md` giao Story 1.14 quyết token thật.
 */
.mode-tab.on {
  color: var(--color-on-surface);
  font-weight: var(--weight-read-title);
  border-bottom-color: var(--color-primary);
}

.modeport {
  flex: 1;
  min-height: 0;
}

.selftest {
  /* ⚠️ `.shell` là một flex container dọc cao đúng `100vh`, và `.modeport` là `flex: 1`
     (basis `0%`). Không có ba dòng dưới đây thì `.selftest` chạy ở `flex: 0 1 auto` với
     `min-height: auto`, tức **không co xuống dưới chiều cao nội dung** — một báo cáo scope
     nhiều dòng bóp `.modeport` về gần 0 và tràn khỏi `100vh`. Không cổng nào bắt được:
     `check:scope` đọc dòng `VERDICT:` từ stdout, không đọc bố cục. Hỏng đúng trong bản
     debug tồn tại để hiển thị chính báo cáo này (Story 1.2/1.3). */
  flex: 0 1 auto;
  min-height: 0;
  overflow: auto;
  padding: var(--space-panel-inline);
  /* Story 1.4: mọi cỡ chữ đến từ token. `--face-ui-mono` trỏ về `--family-mono` qua
     chính token, nên một lần đổi họ chữ của `ui-mono` đi theo mà không phải sửa ở đây. */
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  white-space: pre-wrap;
}
</style>
