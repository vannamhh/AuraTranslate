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
// và UX-DR15 cấm tường minh việc đóng chúng ở đây.
//
// 🔵 2026-09-23 (Phase 3b) — vế trên vẫn đúng theo NGHĨA đó: chế độ này không TỰ đo
// ngưỡng, không tự quyết sacrifice. Nó có thêm một việc thứ tư, thuần hiển thị: nghe
// sự kiện `tier-change` từ `WorkspaceDock` để hiện một thông báo KHÔNG CHẶN khi tầng đo
// được là `unsupported` — lưới vẫn gắn DOM và dùng được nguyên vẹn bên dưới thông báo.
//
// ─────────────────────────────────────────────────────────────────────────────────
// ⚠️ CHUỖI CHẨN ĐOÁN TRONG TỆP NÀY VIẾT **KHÔNG DẤU** — và đó KHÔNG phải cẩu thả
// ─────────────────────────────────────────────────────────────────────────────────
// Kiểm A của `scripts/check-i18n.mjs` quét mọi `.vue` dưới `src/**` và đỏ với một chuỗi
// tiếng Việt CÓ DẤU ở vị trí mã (AC2 của Story 1.5). Nó không phân biệt được *"chuỗi
// hiển thị"* với *"chẩn đoán ra console"* — nó đo DẤU, và `deferred-work.md §*Deferred from: code review of 1-2-scaffold-du-an-va-khoa-pham-vi-filesystem-pham-vi-mang (2026-08-03)*` đã ghi
// đúng giới hạn đó.
//
// Đường thoát dễ là dời `console.warn` sang một tệp `.ts` — Kiểm A không quét `.ts`.
// `deferred-work.md §*Deferred from: code review of 1-2-scaffold-du-an-va-khoa-pham-vi-filesystem-pham-vi-mang (2026-08-03)*` gọi tên đúng đường đó và cấm nó bằng chữ: *"dời một chuỗi từ
// `.vue` sang `.ts` là cách hợp lệ về mặt cổng để cho xanh — đừng dùng."*
//
// ⇒ Dùng tiền lệ đã có: `src-tauri/src/commands/config.rs:36` cũng viết không dấu, cùng
// lý do. Người đọc dòng này là người đang mở DevTools, không phải người dùng cuối.
import { onActivated, onBeforeUnmount, onMounted, shallowRef, useTemplateRef } from 'vue'
import { declareFocus, enterFocus, releaseFocus } from '../commands'
import { bootstrapLayout, KEY_LAYOUT, putConfig, SCOPE_APP_CONFIG } from '../config/bootstrap'
import { t } from '../i18n'
import WorkspaceDock from '../layout/WorkspaceDock.vue'
import type { LayoutTier } from '../layout/workspaceLayout'

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
 * `panel.grid` mỗi lần quay lại Workspace là ghi đè chỗ người dùng đang đứng. `dockview`
 * đã tự khôi phục `activePanel`, và `onDidActivePanelChange` ở `WorkspaceDock` dời focus
 * DOM theo nó khi có lượt đổi thật. Điều AC4 của Story 1.6 đòi là focus **không rơi về
 * `body`** — gốc chế độ mang `tabindex="-1"` nên nó nhận được focus thật.
 *
 * Đường vào panel là `focus.next_panel`, và từ Story 1.14 nó CÓ phím (`Mod+Alt+→`) — tức
 * `deferred-work.md §*Deferred from: 1-4-bo-token-mau-va-chu-hai-theme-co-kiem-tuong-phan-tu-dong (2026-08-03)*` và `:161` đóng ở đây.
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

/**
 * Tầng đang áp — bản sao CỤC BỘ, nhận qua sự kiện `tier-change` (Story 4.12 Phase 3b).
 * `WorkspaceDock` giữ bản chính (`currentTier`, đóng kín trong closure của nó); chế độ này
 * chỉ cần biết có đang ở tầng `unsupported` hay không, để quyết định hiện thông báo.
 */
const currentTier = shallowRef<LayoutTier | null>(null)

function onTierChange(tier: LayoutTier): void {
  currentTier.value = tier
}
</script>

<template>
  <section ref="root" class="mode" tabindex="-1">
    <!--
      Story 4.12, Phase 3b — thông báo tầng `unsupported` (§Boundaries: "grid never yields at
      any size"). Nằm TRƯỚC `WorkspaceDock` trong cùng cột flex, NẰM XUÔI DÒNG — không overlay,
      không modal, không chặn nhập liệu: lưới bên dưới vẫn gắn DOM và dùng được nguyên vẹn.
      `data-workspace-narrow-notice` là móc cho test, không phải nguồn dữ liệu của chính nó.
    -->
    <p v-if="currentTier === 'unsupported'" class="narrow-notice" role="status" data-workspace-narrow-notice>
      <!-- aura-allow-text: KẾT QUẢ của `t()` — chuỗi đã đi qua `vi.json`. -->
      {{ t('mode.workspace.narrow_notice') }}
    </p>
    <WorkspaceDock :saved-layout="bootstrapLayout" @persist="onPersist" @tier-change="onTierChange" />
  </section>
</template>

<style scoped>
.mode {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

/*
 * Dải xuôi dòng, không phải lớp phủ — chiếm chỗ thật trong cột flex nên `WorkspaceDock`
 * (`flex: 1`) tự co lại nhường chỗ, không đè lên lưới. Không z-index, không opacity trung
 * gian: chỉ nền và chữ từ token (`check:tokens`).
 */
.narrow-notice {
  flex: none;
  margin: 0;
  padding: var(--space-panel-block) var(--space-panel-inline);
  border-bottom: 1px solid var(--color-outline);
  background: var(--color-surface-accent);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
}

/* Xem lý do đầy đủ ở `LibraryMode.vue` — chỉ gốc `tabindex="-1"`, không `*:focus`. */
.mode:focus {
  outline: none;
}
</style>
