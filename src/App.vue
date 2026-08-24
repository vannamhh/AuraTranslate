<script setup lang="ts">
// Story 1.2 dựng một cửa sổ trống có chủ ý. Bốn panel và dockview thuộc Story 1.14;
// token màu/chữ thuộc Story 1.4; chuỗi giao diện thuộc Story 1.5.
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — áp từ dòng code đầu tiên.
// Mọi văn bản hiển thị sống ở `src/i18n/vi.json`.
//
// Self-check phạm vi asset protocol (Kiểm 3 của Task 8) chạy khi bật cờ
// `VITE_SCOPE_SELFTEST=1`; xem `src/selftest/scopeCheck.ts`.
import { nextTick, onMounted, ref } from 'vue'
import type { ScopeCheckReport } from './selftest/scopeCheck'
// ⚠️ Import TĨNH có chủ ý, và chỉ hằng này thôi. `./selftest/eventName` là một module
// rỗng ngoài một chuỗi, nên nó vào bundle chính mà không tốn gì — còn `./selftest/
// scopeCheck` thì KHÔNG được import tĩnh: làm vậy là kéo cả mã self-check vào bản
// release, phá đúng bất biến mà `#[cfg(debug_assertions)]` phía Rust đang giữ.
import { SELFTEST_EVENT } from './selftest/eventName'
// Cùng khuôn, cùng lý do an toàn bundle — nhưng vì AC2 của Story 1.5: hai chuỗi chẩn
// đoán dưới đây từng nằm nguyên văn trong template literal ở tệp này, và một chuỗi
// tiếng Việt trong `.vue` là vi phạm NFR16 mà `npm run check:i18n` bắt được.
// Chúng KHÔNG thuộc `vi.json` — đọc doc-comment của `fallbackReport.ts`.
import { emitFailureLine, fallbackReportText } from './selftest/fallbackReport'
// ── Story 1.6 — vỏ một cửa sổ, ba chế độ ngang hàng (AD-24, AC3) ────────────────────
//
// ⚠️ `dispatch` là cửa DUY NHẤT mà một handler chuột được đi qua (AC1). Ba `@click` dưới
// đây là **đúng một** lời gọi `dispatch('<id>')`, và `scripts/check-commands.mjs` Kiểm A
// cưỡng chế điều đó bằng cú pháp trên mọi `.vue` của cây nguồn.
import { dispatch } from './commands'
import { currentMode } from './modes/modeState'
import { t, tError } from './i18n'
// ── Story 1.8 — bề mặt hiển thị lỗi kho, đóng `deferred-work.md:177` ────────────────
//
// 🔴 `configError` chỉ khác `null` khi **Rust đã trả lời** bằng một lỗi thật. Một phiên
// `npm run dev` không có cầu IPC cho `null` — dựng một lỗi giả ở đó làm mọi lần chạy dev
// mọc một dải *"Không mở được kho dữ liệu"*, tức dạy đúng người đọc bỏ qua đúng dải này.
// Xem `src/config/bootstrap.ts` §BootstrapResult.
import { configError } from './config/bootstrap'
// ── Story 1.19 — bề mặt Attribution (§Quyết định #4a, Ice chốt 2026-08-08) ──────────
//
// 🔴 Một **LỚP PHỦ dựng ở đây**, không một chế độ thứ tư: AD-24 khai đúng ba chế độ ngang
// hàng và `MODE_IDS` là một hằng ba phần tử; `Mod+4` thuộc Story 8.11. Nó nói về **cả ứng
// dụng** chứ không về một panel, nên nó sống cùng tầng với dải báo lỗi cấu hình.
import AttributionOverlay from './AttributionOverlay.vue'
// Story 2.3 — thanh trạng thái, vỏ ứng dụng (Quyết định #5).
import StatusBar from './StatusBar.vue'
// Story 3.3 — dải "Thêm thuật ngữ" (FR48), MỘT thể hiện ở chân workspace, ngay TRÊN
// `<StatusBar />`. Xem doc-comment đầu `GlossaryQuickAdd.vue` cho lý do chỗ này.
import GlossaryQuickAdd from './GlossaryQuickAdd.vue'
// Story 3.6 — dải "Chờ chốt lần đầu gặp" (FR114), cùng slot, mọc DƯỚI `<GlossaryQuickAdd
// />` — thứ tự DOM là thứ tự thị giác (§Tasks của spec), dù `topmostStrip` đã đảm bảo
// không bao giờ cả hai cùng hiện.
import GlossaryConfirmStrip from './GlossaryConfirmStrip.vue'
import ShortcutsOverlay from './ShortcutsOverlay.vue'
import SegmentHistoryOverlay from './SegmentHistoryOverlay.vue'
// Story 3.5 — lớp phủ "Cài đặt ngưỡng quét Glossary" (FR47), lớp phủ THỨ TƯ. Cùng tầng,
// cùng lý do ba lớp phủ kia.
import GlossarySettingsOverlay from './GlossarySettingsOverlay.vue'
// Story 3.8 — lớp phủ "Duyệt hàng loạt một phím" (FR53/FR55), lớp phủ THỨ NĂM. Cùng tầng,
// cùng lý do bốn lớp phủ kia.
import GlossaryQueueOverlay from './GlossaryQueueOverlay.vue'
import GlossaryManageOverlay from './GlossaryManageOverlay.vue'
import LibraryMode from './modes/LibraryMode.vue'
import WorkspaceMode from './modes/WorkspaceMode.vue'
import ReadingMode from './modes/ReadingMode.vue'

const report = ref<ScopeCheckReport | null>(null)
const selftestEnabled = import.meta.env.VITE_SCOPE_SELFTEST === '1'

// ── Lượt review 2026-08-04 — công bố lỗi kho cho trình đọc màn hình ─────────────────
//
// 🔴 `configError` đã có giá trị TRƯỚC lượt vẽ đầu tiên: `boot()` (`src/main.ts`) `await`
// xong cấu hình rồi mới `mount()`, nên nếu dải `role="status"` bên dưới đọc thẳng
// `configError` thì nội dung của nó đã có mặt ngay từ khi node đó lần đầu vào DOM. Theo
// ngữ nghĩa live-region của ARIA, một vùng chỉ được đảm bảo công bố khi nội dung của nó
// ĐỔI sau khi vùng đã tồn tại — nội dung có sẵn từ lượt vẽ đầu không được đảm bảo đọc.
// Đó đúng là ca *"kho chết lúc khởi động"* mà dải này dựng ra để phục vụ.
//
// Chốt: `announcedConfigError` khởi tạo RỖNG, nên node `role="status"` vào DOM trước với
// nội dung trống, rồi được điền ở một lượt render SAU (`nextTick`) — một lượt ĐỔI nội
// dung thật mà trình đọc màn hình quan sát được, không phải nội dung có sẵn từ đầu.
const announcedConfigError = ref('')

onMounted(async () => {
  await nextTick()
  if (configError.value) announcedConfigError.value = tError(configError.value)
})

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

/**
 * Ép tiêu điểm lên chính nút vừa bị bấm — điều kiện để UX-DR17 đúng trên WKWebView.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MỘT PHÉP ĐO, KHÔNG MỘT LƯỢT PHÒNG XA — bắt được 2026-08-11, Story 1.22 C2
 * ─────────────────────────────────────────────────────────────────────────────
 * WKWebView **không** đặt tiêu điểm cho `<button>` khi bấm chuột. Đường lui của UX-DR17
 * ở `ShortcutsOverlay.vue` lưu `document.activeElement` **lúc mở** rồi trả về đúng node
 * đó; không có dòng này, node ấy là thứ đang giữ tiêu điểm từ trước — điểm vào focus của
 * chế độ — chứ không phải nút mở. Đo được trên webview thật: sau `Escape`, tiêu điểm rơi
 * vào `section.mode`, không về nút.
 *
 * ⚠️ Nhánh dự phòng `querySelector('[data-shortcuts-open]')` của lớp phủ **không cứu
 * được** ca này, và đó là chỗ dễ đọc sai nhất: nó chỉ chạy khi node đã lưu **rời DOM**,
 * mà `section.mode` thì vẫn ở nguyên đó. Một đường lui không bao giờ chạy tới không phải
 * một đường lui.
 *
 * ⚠️ Vì sao bàn đo cũ vẫn XANH suốt thời gian đó: nó bấm bằng `element.click()` của
 * driver, và lệnh ấy **có** đặt tiêu điểm — tức nó xanh vì một hành vi mà chuột thật
 * không có. Story 1.22 AC3 đổi sang Actions API và ca đỏ ngay lượt đầu.
 *
 * `@mousedown` chứ không `@focus`: mousedown chạy **trước** `@click` phát command, nên
 * lúc lớp phủ mở ra thì `activeElement` đã đúng. Cùng khuôn `@mousedown` mà
 * `config/shortcutsState.ts` đã dùng cho đúng khuyết tật engine này.
 */
function focusOnPointerDown(event: MouseEvent) {
  const target = event.currentTarget
  if (target instanceof HTMLElement) target.focus()
}
</script>

<template>
  <main class="shell">
    <!--
      🔴 DẢI BÁO LỖI KHÔNG CHẶN — Story 1.8, đóng `deferred-work.md:177`.

      Trước story này, một `$APPDATA` không ghi được chỉ ra `stderr`: `lib.rs::open_global_store`
      ghi chẩn đoán rồi đi tiếp, và người dùng nhận đúng thứ tệ nhất — im lặng. Nay nó nói ra.

      ⚠️ **KHÔNG CHẶN**, và đó là cả điểm: ứng dụng vẫn dùng được bằng cấu hình mặc định.
      Một modal ở đây biến một lỗi *"lựa chọn của bạn sẽ không được nhớ"* thành một bức
      tường — và AD-22 cùng tinh thần: hỏng thì hiện ra, đừng sập.

      Nội dung đi qua `tError(err)`, không phải một chuỗi viết thẳng: NFR16 nói mọi
      văn bản hiển thị sống ở `vi.json` và chỉ ở đó. không `err.code` không bao giờ ra màn
      hình (AD-21) — nó chỉ để rẽ nhánh.

      ⚠️ `role="status"` chứ không `role="alert"`: `alert` cắt ngang trình đọc màn hình
      giữa câu, và đây là một thông báo về trạng thái, không phải một tình huống khẩn.

      🔴 HAI NODE, KHÔNG MỘT — xem doc-comment của `announcedConfigError` ở khối script
      phía trên.
      `.sr-announcer` mang `role="status"` và LUÔN có mặt trong DOM (không `v-if`) để
      trình đọc màn hình công bố được nội dung ĐỔI sau `nextTick`; dải `.config-error`
      hiển thị cho người dùng sáng mắt và vẫn dùng `v-if` bình thường — không cần đợi gì,
      họ đọc bằng mắt ngay khi nó xuất hiện.
    -->
    <!--
      aura-allow-text: `announcedConfigError` mang KẾT QUẢ của `tError(configError)` —
      chuỗi ĐÃ dịch, đã đi qua `vi.json`. Nó phải là một `ref` chứ không phải một lời gọi
      `tError()` tại chỗ, vì `role="status"` chỉ công bố được nội dung ĐỔI sau khi node đã
      tồn tại (xem doc-comment của `announcedConfigError` ở khối script). Kiểm A2 đọc cú
      pháp chứ không đọc được điều đó — nên đây là chỗ một con người ký.
    -->
    <p class="sr-announcer" role="status">{{ announcedConfigError }}</p>
    <p v-if="configError" class="config-error">
      {{ tError(configError) }}
    </p>

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

      <!--
        🔴 Story 1.21 — ĐƯỜNG VÀO màn hình phím tắt, đặt ở `titlebar` vì đó là chỗ **duy
        nhất luôn hiện ở cả ba chế độ**.

        `data-shortcuts-open` là **đường lui của tiêu điểm** khi lớp phủ đóng (UX-DR17) —
        một hợp đồng đi bằng thuộc tính `data-`, không một tên lớp CSS: tên lớp là chuyện
        trình bày và đổi được tự do, còn đây là một mối nối. Cùng khuôn
        `data-attribution-open` của Story 1.19.
      -->
      <button
        type="button"
        class="titlebar-act"
        data-shortcuts-open
        @mousedown="focusOnPointerDown($event)"
        @click="dispatch('shortcuts.open')"
      >
        {{ t('command.shortcuts.open') }}
      </button>

      <!--
        Story 3.5 — ĐƯỜNG VÀO lớp phủ ngưỡng quét Glossary. Cùng khuôn nút phím tắt ngay
        trên: `data-glossary-settings-open` là đường lui của tiêu điểm (UX-DR17), và
        `@mousedown` đặt tiêu điểm trước khi `@click` phát command (cùng khuyết tật engine
        đã ghi ở `focusOnPointerDown`).
      -->
      <button
        type="button"
        class="titlebar-act"
        data-glossary-settings-open
        @mousedown="focusOnPointerDown($event)"
        @click="dispatch('glossary.settings.open')"
      >
        {{ t('command.glossary.settings.open') }}
      </button>

      <!--
        Story 3.8 — ĐƯỜNG VÀO lớp phủ duyệt hàng loạt. Cùng khuôn hai nút ngay trên:
        `data-glossary-queue-open` là đường lui của tiêu điểm (UX-DR17).
      -->
      <button
        type="button"
        class="titlebar-act"
        data-glossary-queue-open
        @mousedown="focusOnPointerDown($event)"
        @click="dispatch('glossary.queue.open')"
      >
        {{ t('command.glossary.queue.open') }}
      </button>

      <!--
        Story 3.9 — ĐƯỜNG VÀO lớp phủ Quản lý Glossary. Cùng khuôn ba nút ngay trên:
        `data-glossary-manage-open` là đường lui của tiêu điểm (UX-DR17).
      -->
      <button
        type="button"
        class="titlebar-act"
        data-glossary-manage-open
        @mousedown="focusOnPointerDown($event)"
        @click="dispatch('glossary.manage.open')"
      >
        {{ t('command.glossary.manage.open') }}
      </button>
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

    <!--
      🔴 STORY 2.3 · AC7 · AC9 — THANH TRẠNG THÁI, lượt đầu tiên nó tồn tại.

      Đặt ở **vỏ ứng dụng** dưới `.modeport`, không trong `WorkspaceMode.vue` (Quyết định #5,
      đường (a)): `EXPERIENCE.md:417` đã dùng nó để tính chiều cao vùng làm việc cho **cả ba**
      chế độ, và UX-DR15 hứa Panel Lookup *"rút về thanh trạng thái"* ở màn hình hẹp (Story
      4.12) — một thanh chỉ sống trong Workspace sẽ phải chuyển chỗ lần nữa ở đó.

      ⚠️ Đặt TRƯỚC `pre.selftest`: dải chẩn đoán đó chỉ tồn tại khi `VITE_SCOPE_SELFTEST=1`, và
      nó là thứ cuối cùng của vỏ trong bản debug. Thanh trạng thái là một phần **thường trực**
      của bố cục mà `EXPERIENCE.md:417` tính vào chiều cao — nó không được nằm dưới một khối
      chỉ có mặt trong một chế độ chạy.
    -->
    <!--
      Story 3.3 · FR48 — dải "Thêm thuật ngữ", tự quản `v-if` của nó qua `quickAddIsOpen`.
      NGAY TRÊN `<StatusBar />` — xem doc-comment đầu `GlossaryQuickAdd.vue`.
    -->
    <GlossaryQuickAdd />

    <!--
      Story 3.6 · FR114 — dải "Chờ chốt lần đầu gặp", tự quản `v-if` của nó qua
      `topmostStrip(...) === 'glossary_confirm'`. Mọc DƯỚI `<GlossaryQuickAdd />`, NGAY TRÊN
      `<StatusBar />` — hai dải không bao giờ hoán chỗ nhau vì `topmostStrip` chỉ cho ĐÚNG
      một dải hiện tại một thời điểm.
    -->
    <GlossaryConfirmStrip />

    <StatusBar />

    <!--
      aura-allow-text: báo cáo self-check phạm vi asset protocol — CHẨN ĐOÁN cho log CI,
      chỉ dựng khi `VITE_SCOPE_SELFTEST=1` và không vào bản phát hành. `vi.json` là tài
      nguyên HIỂN THỊ; trộn chẩn đoán vào đó là hỏng chính ranh giới Story 1.5 dựng — cùng
      lý do mà `src/selftest/**` được miễn trừ TRỌN ở `EXEMPT` của cổng này.
    -->
    <pre v-if="report" class="selftest" :data-verdict="report.verdict">{{ report.text }}</pre>

    <!-- Story 1.19 · AC7–AC11 — lớp phủ tự quản `v-if` của nó qua `attributionIsOpen`. -->
    <AttributionOverlay />

    <!-- Story 1.21 · AC1–AC13 — cùng khuôn: lớp phủ tự quản `v-if` qua `shortcutsOverlayIsOpen`. -->
    <ShortcutsOverlay />

    <!-- Story 2.6 · FR101 · AC1–AC3 — cùng khuôn: lớp phủ tự quản `v-if` qua `historyIsOpen`. -->
    <SegmentHistoryOverlay />

    <!-- Story 3.5 · FR47 — cùng khuôn: lớp phủ tự quản `v-if` qua `glossarySettingsOverlayIsOpen`. -->
    <GlossarySettingsOverlay />

    <!-- Story 3.8 · FR53/FR55 — cùng khuôn: lớp phủ tự quản `v-if` qua `queueOverlayIsOpen`. -->
    <GlossaryQueueOverlay />

    <!-- Story 3.9 · FR49 — cùng khuôn: lớp phủ tự quản `v-if` qua `manageOverlayIsOpen`. -->
    <GlossaryManageOverlay />
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

/*
 * `.sr-announcer` — vùng công bố cho trình đọc màn hình (lượt review 2026-08-04).
 *
 * ⚠️ "Visually hidden" bằng `clip`/`position: absolute`, KHÔNG `display: none`: một
 * phần tử `display: none` bị loại khỏi accessibility tree, tức trình đọc màn hình coi nó
 * chưa từng tồn tại — và một `role="status"` "chưa từng tồn tại" thì lượt điền nội dung
 * sau đó lại đúng vấn đề ban đầu (node với nội dung mới CÙNG lúc mới xuất hiện). Node
 * này phải LUÔN hiện diện trong accessibility tree, chỉ ẩn về mặt THỊ GIÁC.
 *
 * Các giá trị `1px`/`-1px` ở đây là kích thước hình học của kỹ thuật ẩn, không phải
 * màu hay cỡ chữ — `check:tokens` Kiểm B/D chỉ cưỡng chế hai loại đó.
 */
.sr-announcer {
  position: absolute;
  width: 1px;
  height: 1px;
  margin: -1px;
  padding: 0;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

/*
 * Dải báo lỗi cấu hình (Story 1.8). Chỉ token đã có — `check:tokens` Kiểm B2 đỏ với một
 * giá trị màu hay cỡ chữ viết thẳng, và ở đây không có gì cần lách.
 *
 * ⚠️ `flex: none` là bắt buộc, không phải làm đẹp: `.shell` là một flex container dọc cao
 * đúng `100vh`. Không có nó, dải này chạy ở `flex: 0 1 auto` và bị co lại khi `.modeport`
 * đòi chỗ — tức đúng thông báo mà story này dựng ra để người dùng đọc lại là thứ bị bóp
 * mất trước tiên. Cùng bài học với `.selftest` ngay dưới.
 */
.config-error {
  flex: none;
  margin: 0;
  padding: var(--space-panel-inline);
  border-bottom: 1px solid var(--color-outline);
  color: var(--color-error);
  background: var(--color-surface);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
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

/* Story 1.21 — đường vào màn hình phím tắt, đẩy về mép phải của thanh tiêu đề. */
.titlebar-act {
  margin-left: auto;
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

/*
 * Ba tab chế độ. ⚠️ Chúng là `<button>` chứ không phải `<span>` như mockup vẽ: một tab
 * chế độ là thao tác, nên nó phải vào được thứ tự Tab và nhận được `Enter`/`Space` —
 * NFR17 nói *"mọi thao tác gọi được bằng bàn phím"*, và một `<span @click>` không gọi
 * được bằng bàn phím ở bất kỳ trình duyệt nào.
 *
 * KHÔNG `outline: none` ở đây. `outline: none` chỉ áp cho gốc `tabindex="-1"` của
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
 * `var(--weight-ui-md-strong)` là token thứ 15, thêm ở Story 1.14 · AC10 để đóng
 * `deferred-work.md:138`. Trước nó, tab này MƯỢN `--weight-read-title` (một token của
 * tiêu đề Chương, 23px họ `read`) chỉ để lấy con số 600 — rủi ro đã ghi: đổi giá trị
 * `--weight-read-title` thì tab chế độ đổi theo mà không ai biết. Nay không còn mượn.
 */
.mode-tab.on {
  color: var(--color-on-surface);
  font-weight: var(--weight-ui-md-strong);
  border-bottom-color: var(--color-primary);
}

.modeport {
  flex: 1;
  min-height: 0;
  /*
   * ═════════════════════════════════════════════════════════════════════════════════
   * 🔴 NGỮ CẢNH XẾP LỚP CHO CẢ CÂY DOCKVIEW — Ice bắt bằng mắt 2026-08-10
   * ═════════════════════════════════════════════════════════════════════════════════
   *
   * Triệu chứng: mở màn hình *Nguồn dữ liệu*, **thanh kéo (sash) chia ba panel vẽ ĐÈ lên
   * lớp phủ** — hai vệt màu nền cắt ngang bảng ghi công.
   *
   * Ba phép đo, không một suy đoán:
   *   `dockview.css:2940-2942`  → `.dv-sash { position: absolute; z-index: 99 }`
   *   `AttributionOverlay.vue`  → `.attr-scrim { position: fixed; z-index: 10 }`
   *   khối này TRƯỚC lượt vá    → `flex: 1; min-height: 0`, tức **không** thuộc tính nào
   *                               tạo ngữ cảnh xếp lớp
   * ⇒ `99` và `10` tranh nhau trong **cùng một** ngữ cảnh gốc, và sash thắng. Thanh sash
   * mang `--dv-sash-color: var(--color-background)` (`dockview-theme.css:62`), nên nó vẽ
   * ra đúng hai vệt màu nền mà ảnh chụp cho thấy.
   *
   * 🔴 **VÌ SAO KHÔNG NÂNG `z-index` CỦA LỚP PHỦ.** Đó là cách vá hiển nhiên và là cách
   * sai: dockview đang dùng `99` (sash), `999` (drop target), `1000`, và `9999`
   * (`.dv-drop-target-container`). Chọn một số lớn hơn là bước vào một cuộc đua với một
   * thư viện ghim theo phiên bản — ta thắng hôm nay và thua im lặng ở lần nâng
   * `dockview-vue` kế tiếp, với triệu chứng y hệt và không cổng nào đỏ.
   *
   * `isolation: isolate` nói đúng mệnh đề cần nói: *"mọi `z-index` bên trong cây dockview
   * là chuyện NỘI BỘ của nó"*. Cả `99`, `999` lẫn `9999` thành cục bộ, thứ tự giữa chúng
   * **không đổi một chút nào** (drop target vẫn trên panel), còn `.attr-scrim` — nằm
   * NGOÀI khối này, anh em với nó trong `.shell` — chỉ cần đứng trên `.modeport` như một
   * khối duy nhất. `z-index: 10` của nó ở lại nguyên giá trị và nguyên lý do.
   *
   * ⚠️ `isolation` KHÔNG phải `z-index`, nên nó không cần `aura-allow-z-index` của Kiểm F
   * (`check-tokens.mjs:1370`) — và nó cũng không đụng bố cục: không `position`, không
   * `transform`, không `opacity`.
   *
   * ⚠️ Giới hạn đã biết: nếu dockview gắn một lớp phủ vào thẳng `document.body` (ngoài
   * khối này) thì `9999` của nó lại vượt lên. Đường duy nhất làm việc đó là
   * `addPopoutGroup`, và `scripts/check-layout.mjs` **cấm** nó (Story 1.14 · AC1) — nên
   * hôm nay không có đường nào, và ngày có thì cổng đó đỏ trước.
   */
  isolation: isolate;
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
