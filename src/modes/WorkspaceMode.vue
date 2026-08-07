<script setup lang="ts">
// Chế độ 2/3 — môi trường dịch. Story 1.6 · AC3 · AC4 · AC5 · Story 1.14 · AC1 · AC4.
//
// HAI `PanelFrame` trần đã đi — Story 1.14 thay chúng bằng `WorkspaceDock`, tức bốn
// panel thật sống trong `dockview`. Chế độ này giờ chỉ còn ba việc:
//   1. giữ điểm vào focus `mode.workspace` (AD-34 §2, không đổi một dòng nào);
//   2. nối bố cục đã lưu từ `bootstrap_config` xuống vỏ dock (AC4);
//   3. đẩy lượt ghi ngược lên `putConfig`.
//
// Không ngưỡng màn hình hẹp, không `matchMedia`, không ngăn kéo — **Story 4.12**,
// và `epics.md:1617` cấm tường minh việc đóng chúng ở đây.
//
// ─────────────────────────────────────────────────────────────────────────────────
// ⚠️ CHUỖI CHẨN ĐOÁN TRONG TỆP NÀY VIẾT **KHÔNG DẤU** — và đó KHÔNG phải cẩu thả
// ─────────────────────────────────────────────────────────────────────────────────
// Kiểm A của `scripts/check-i18n.mjs` quét mọi `.vue` dưới `src/**` và đỏ với một chuỗi
// tiếng Việt CÓ DẤU ở vị trí mã (AC2 của Story 1.5). Nó không phân biệt được *"chuỗi
// hiển thị"* với *"chẩn đoán ra console"* — nó đo DẤU, và `deferred-work.md:36` đã ghi
// đúng giới hạn đó.
//
// Đường thoát dễ là dời `console.warn` sang một tệp `.ts` — Kiểm A không quét `.ts`.
// `deferred-work.md:35` gọi tên đúng đường đó và cấm nó bằng chữ: *"dời một chuỗi từ
// `.vue` sang `.ts` là cách hợp lệ về mặt cổng để cho xanh — đừng dùng."*
//
// ⇒ Dùng tiền lệ đã có: `src-tauri/src/commands/config.rs:36` cũng viết không dấu, cùng
// lý do. Người đọc dòng này là người đang mở DevTools, không phải người dùng cuối.
import { onActivated, onBeforeUnmount, onMounted, useTemplateRef } from 'vue'
import { declareFocus, enterFocus, releaseFocus } from '../commands'
import { bootstrapLayout, KEY_LAYOUT, putConfig, SCOPE_APP_CONFIG } from '../config/bootstrap'
import WorkspaceDock from '../layout/WorkspaceDock.vue'

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
 * 🔴 Story 1.14 giữ nguyên quyết định đó, và nay nó có một lý do THỨ HAI: nhảy thẳng vào
 * `panel.source` mỗi lần quay lại Workspace là ghi đè chỗ người dùng đang đứng. `dockview`
 * đã tự khôi phục `activePanel`, và `onDidActivePanelChange` ở `WorkspaceDock` dời focus
 * DOM theo nó khi có lượt đổi thật. Điều AC4 của Story 1.6 đòi là focus **không rơi về
 * `body`** — gốc chế độ mang `tabindex="-1"` nên nó nhận được focus thật.
 *
 * Đường vào panel là `focus.next_panel`, và từ Story 1.14 nó CÓ phím (`Mod+Alt+→`) — tức
 * `deferred-work.md:134` và `:161` đóng ở đây.
 */
onActivated(() => {
  void enterFocus('mode.workspace')
})

/**
 * 🔴 BỐ CỤC ĐANG HIỂN THỊ → `ScopeKind::AppConfig`, KHÔNG `localStorage` (§Quyết định #5A).
 *
 * `kinds.rs:206-213` đã phân xử một nửa: *"**bố cục đang hiển thị** là của frontend;
 * **preset đã đặt tên và lưu lại** là dữ liệu Rust"* — và cảnh báo thẳng rằng *"cách đọc
 * kia dẫn thẳng tới `localStorage`"*. Nửa còn lại chốt ở đây: *của frontend* nói về ai
 * QUYẾT bố cục, không nói về việc nó được cất ở đâu. Nó đi cùng cửa với `theme` và
 * `mode` — cùng là *"trạng thái cuối cùng của ứng dụng"* — nên nó vào `app_config`, qua
 * `store::Writer` nối tiếp (AD-11).
 *
 * KHÔNG nhét nó vào `layout_presets` dưới một khoá `__current`: làm vậy là bẻ nghĩa
 * của *"preset đã ĐẶT TÊN"*, và màn hình của Story 1.21 sẽ hiện `__current` ra như một
 * preset người dùng tự tạo.
 *
 * ⚠️ `putConfig` không bao giờ ném; một lượt lưu trượt chỉ ghi chẩn đoán. Cùng lý lẽ với
 * `watch(currentMode)` ở `main.ts`: sắp lại panel là một thao tác phải MƯỢT (AD-34), và
 * một hộp thoại lỗi ở đó là quy tắc nghiệp vụ giả đặt sai chỗ.
 */
function onPersist(json: string): void {
  void putConfig(SCOPE_APP_CONFIG, KEY_LAYOUT, json).then((err) => {
    // ⚠️ Chẩn đoán viết KHÔNG DẤU — xem khối `⚠️ CHUỖI CHẨN ĐOÁN` ở đầu tệp.
    if (err !== null) console.warn(`[layout] khong luu duoc bo cuc (\`${err.code}\`).`)
  })
}
</script>

<template>
  <section ref="root" class="mode" tabindex="-1">
    <WorkspaceDock :saved-layout="bootstrapLayout" @persist="onPersist" />
  </section>
</template>

<style scoped>
.mode {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

/* Xem lý do đầy đủ ở `LibraryMode.vue` — chỉ gốc `tabindex="-1"`, không `*:focus`. */
.mode:focus {
  outline: none;
}
</style>
