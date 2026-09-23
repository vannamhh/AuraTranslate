<script setup lang="ts">
// Ngăn kéo Tra cứu — Story 4.12, Phase 3a, Task 6, Decision 2.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT NGĂN KÉO, KHÔNG MỘT HỘP THOẠI GIỮA MÀN HÌNH (Decision 2, Ice ký)
// ─────────────────────────────────────────────────────────────────────────────────
// `src/` đã có mười hai lớp phủ và cả mười hai là hộp thoại CĂN GIỮA — UX-DR16 cấm hộp thoại
// cho thao tác THƯỜNG XUYÊN, và tra cứu là thao tác thường xuyên nhất của ứng dụng. Ngăn kéo
// này trượt vào từ CẠNH PHẢI, đè lên lưới chứ không thay chỗ nó — cùng khuôn scrim/focus-trap
// của mười hai lớp phủ kia (chép từ `AttributionOverlay.vue`, lớp phủ ĐƠN GIẢN NHẤT trong đó),
// chỉ đổi hình dạng của `.panel`: neo một cạnh, cao trọn, rộng có trần — không đè kín màn hình.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 NỘI DUNG LÀ `LookupPanel.vue` THẬT, KHÔNG MỘT BẢN TÓM TẮT
// ─────────────────────────────────────────────────────────────────────────────────
// AC của spec: "ngăn kéo mở ra với Tra cứu HOÀN TOÀN dùng được — ở không kích thước cửa sổ
// nào Tra cứu không tới được". Component này VỐN đã mang trọn state module-level của Tra cứu
// (`lookupPanelState.ts`, `lookupHistoryState.ts`, …), nên mount lại nó ở đây không nhân đôi
// dữ liệu — nó chỉ là MỘT CHỖ THỨ HAI để đúng một state đó render ra.
//
// `PanelFrame.vue` (lồng bên trong `LookupPanel.vue`) tự gọi `declareFocus('panel.lookup', …)`
// lúc mount và `releaseFocus('panel.lookup', resolve)` lúc tháo — TÁI DÙNG đúng cơ chế AD-34
// §2 đã có, nên `enter('panel.lookup')` (từ `focus.next_panel`, hay bất kỳ chỗ gọi tên panel
// này) vẫn trỏ vào một phần tử THẬT đang trong DOM, dù Tra cứu đang sống trong lưới hay trong
// ngăn kéo — ĐÓ CHÍNH LÀ điều "giữ `declareFocus('panel.lookup')` trung thực" mà phase file đòi.
//
// 🔵 SỬA 2026-09-23 — ca "tầng nhảy thẳng từ `narrow`/`unsupported` về `short`/`full` trong
// MỘT sự kiện trong khi ngăn kéo đang mở" nay ĐÃ ĐÓNG TẬN GỐC, không còn là giới hạn ghi nợ.
// `WorkspaceDock.vue::applyTier` gọi `lookupDrawerState.ts::syncLayoutTier` ĐỒNG BỘ trước khi
// `restoreAllAutoHidden` có cơ hội `addPanel` lại `panel.lookup`; hàm đó đóng ngăn kéo VÀ nhả
// `panel.lookup` ngay tại chỗ trước khi dock kịp khai lại. Lượt `releaseFocus` MUỘN mà
// `PanelFrame.vue::onBeforeUnmount` của bản sao này gọi sau đó (khi Vue tháo nó bất đồng bộ)
// so `resolve` bằng THAM CHIẾU với đăng ký hiện tại và thoát êm khi không còn khớp — không xoá
// nhầm đăng ký mới của bản sao ở lưới. Xem doc-comment đầu `lookupDrawerState.ts` cho lý lẽ đầy
// đủ và vì sao `owner` vẫn là đúng MỘT chuỗi `'panel.lookup'` ở cả hai nơi nó sống.
import { nextTick, useTemplateRef, watch } from 'vue'
import { t } from '../i18n'
import { dispatch } from '../commands'
import { focusReturnTargetOnOpen } from '../commands/focus'
import { closeLookupDrawer, lookupDrawerIsOpen } from './lookupDrawerState'
import LookupPanel from '../panels/LookupPanel.vue'
import type { DockviewPanelProps } from './panelProps'

/**
 * `LookupPanel.vue` chỉ khai `defineProps<DockviewPanelProps>()` để khớp hình dạng mà
 * `dockview-vue` mount bằng — bản thân component KHÔNG đọc `params` (xem `panelProps.ts`).
 * Một object RỖNG hợp lệ về kiểu (`PanelParams.titleKey` là tuỳ chọn) và KHÔNG được dựng lại
 * mỗi lần render — hằng số module-level, không một biểu thức object literal trong template.
 */
const LOOKUP_PANEL_PARAMS: DockviewPanelProps['params'] = { params: {} }

const panel = useTemplateRef<HTMLElement>('panel')

/** 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Khuôn và lý lẽ chép nguyên từ `AttributionOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

watch(lookupDrawerIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-lookup-drawer-open]')
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-lookup-drawer-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  // Chẩn đoán viết bằng tiếng Anh — Kiểm A của `check:i18n` cấm chuỗi tiếng Việt ở vị trí mã.
  console.warn('[lookup-drawer] focus-return target is gone; focus falls back to body.')
})

/** Bẫy tiêu điểm — cùng khuôn `AttributionOverlay.vue::trapTab`, điều kiện để `aria-modal`
 *  không phải một lời khai sai. */
function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
}

function trapTab(event: KeyboardEvent): void {
  const root = panel.value
  if (root === null) return

  event.preventDefault()
  const stops = focusableWithin(root)
  if (stops.length === 0) {
    root.focus()
    return
  }

  const active = document.activeElement
  const index = active instanceof HTMLElement ? stops.indexOf(active) : -1
  const step = event.shiftKey ? -1 : 1
  const next = index === -1 ? (event.shiftKey ? stops.length - 1 : 0) : index + step
  stops[(next + stops.length) % stops.length].focus()
}
</script>

<template>
  <!--
    ⚠️ `@keydown.esc` là DOM thường, KHÔNG một command — cùng lý lẽ `ShortcutsOverlay.vue`:
    `Escape` ở đây là một lượt huỷ TRONG NGỮ CẢNH, chỉ có nghĩa khi ngăn kéo đang mở.
    Nút đóng thì ĐI QUA command — Kiểm A của `check:commands` đòi mọi `@click` là đúng một
    `dispatch('<id>')`.
  -->
  <div
    v-if="lookupDrawerIsOpen"
    class="ld-scrim"
    @keydown.esc="closeLookupDrawer()"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="ld-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="ld-head">
        <h2 class="ld-title">{{ t('panel.lookup.title') }}</h2>
        <button type="button" class="ld-close" @click="dispatch('layout.lookup_drawer_close')">
          {{ t('command.layout.lookup_drawer_close') }}
        </button>
      </header>
      <div class="ld-body">
        <LookupPanel :params="LOOKUP_PANEL_PARAMS" />
      </div>
    </section>
  </div>
</template>

<style scoped>
/*
 * Cùng khuôn `.attr-scrim` — `position: fixed; inset: 0`, nền qua token (không `rgba()` viết
 * thẳng, Kiểm B của `check:tokens`). `justify-content: flex-end` là toàn bộ khác biệt hình
 * học với một lớp phủ căn giữa: panel neo cạnh PHẢI thay vì giữa màn hình (Decision 2).
 */
.ld-scrim {
  position: fixed;
  inset: 0;
  /* aura-allow-z-index: xếp lớp cơ học, cùng lý do và cùng con số `.attr-scrim` — dockview dựng ngữ cảnh xếp lớp riêng cho mỗi nhóm panel, và `.modeport` (App.vue) đã cô lập nó bằng `isolation: isolate`. */
  z-index: 10;
  display: flex;
  justify-content: flex-end;
  align-items: stretch;
  background: var(--color-background);
}

/* `min(420px, 100%)`: đủ rộng cho một kết quả tra cứu thật, không tràn khỏi một cửa sổ đang ở
   tầng `unsupported` (< 860px). Không `max-width` cứng hơn — 420px đã là trần. */
.ld-panel {
  width: min(420px, 100%);
  height: 100%;
  overflow: auto;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.ld-head {
  flex: none;
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  padding: var(--space-panel-block) var(--space-panel-inline) 0;
}

.ld-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.ld-close {
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

/* Thân co giãn, `LookupPanel.vue` tự lo cuộn/bố cục bên trong nó (`PanelFrame.vue::.panel`
   đã khai `height: 100%`), đúng khuôn nó vốn chạy trong một ô dockview. */
.ld-body {
  flex: 1;
  min-height: 0;
}
</style>
