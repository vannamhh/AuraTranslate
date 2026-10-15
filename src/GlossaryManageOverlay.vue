<script setup lang="ts">
// Lớp phủ **Quản lý Glossary** — Story 3.9, FR49, lớp phủ THỨ SÁU.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT LỚP PHỦ, KHUÔN `GlossaryQueueOverlay.vue` — scrim + `role="dialog"` + bẫy Tab tự
// viết + focus-return qua `data-glossary-manage-open`
// ─────────────────────────────────────────────────────────────────────────────
//
// Sửa/Lưu/Huỷ/Xoá/Đẩy/Chuyển/Đóng đi qua `dispatch('<id>')` (AD-34) — mọi `@click` trong
// tệp này là ĐÚNG MỘT lời gọi `dispatch(...)`, không hàm khác (`check:commands` Kiểm A). Ô
// tìm và ba bộ lọc KHÔNG qua registry — chúng là chỉnh sửa dữ liệu form, cùng tiền lệ
// `GlossarySettingsOverlay.vue::onInput` (ngưỡng quét).
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 KHÁC `GlossaryQueueOverlay.vue`: LỚP PHỦ NÀY MANG Ô GÕ CHỮ THẬT (tìm + form Sửa)
// ─────────────────────────────────────────────────────────────────────────────
// `onKeydown` dưới đây filter modifier TRƯỚC (§Always: quên nó thì `⌘Backspace` xoá một mục
// ngoài ý định — bài học vá P1 của Story 3.8), VÀ filter tiếp một lần nữa theo TARGET: nếu
// phím đang gõ rơi vào `<input>`/`<textarea>`/`<select>` (ô tìm, hoặc ba ô của form Sửa),
// handler thoát sớm để gõ chữ bình thường (kể cả `Backspace` xoá một ký tự) không kích hoạt
// bất kỳ lệnh nào. Chỉ khi focus đang ở NGOÀI mọi ô form (mặc định — focus rơi vào `panel`
// lúc mở, giống `GlossaryQueueOverlay.vue`) thì mũi tên/`Backspace`/`Delete`/`Enter` mới có
// tác dụng.
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import type { GlossaryCategory, GlossaryEntry } from './config/glossary'
import type { GlossaryTierWire } from './config/glossary'
import { glossaryExchangeBusy } from './glossaryExchangeGate'
import {
  cancelGlossaryManageDeleteConfirm,
  manageActionError,
  manageActionNotice,
  manageCategoryFilter,
  manageConfirmedFilter,
  manageCursor,
  manageCurrentRow,
  manageDeletePending,
  manageEditCategory,
  manageEditNote,
  manageEditTranslation,
  manageEditing,
  manageEmptyReasonFor,
  manageExchangeTier,
  manageExportBusy,
  manageExportError,
  manageExportIpcUnavailable,
  manageExportedPath,
  manageFilteredRows,
  manageTierCounts,
  manageTierFilter,
  manageTotalRows,
  managePendingCount,
  manageLoadError,
  manageOriginFilter,
  manageOverlayIsOpen,
  manageSaving,
  manageSavingAction,
  manageSearchQuery,
  manageSortByFrequency,
  manageStatus,
  manageWorkTierAvailable,
  setGlossaryManageCategoryFilter,
  setGlossaryManageConfirmedFilter,
  setGlossaryManageCursor,
  setGlossaryManageExchangeTier,
  setGlossaryManageOriginFilter,
  setGlossaryManageSearch,
  setGlossaryManageSortByFrequency,
  setGlossaryManageTierFilter,
} from './glossaryManageState'
import type { GlossaryManageOriginFilter, GlossaryManageTierFilter } from './glossaryManageState'

/** 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Khuôn và lý lẽ chép từ `GlossaryQueueOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
// Lớp phủ chứa chữ thật (thuật ngữ, ghi chú) nhưng không phải nguồn từ điển. Đăng ký
// `display` để một vùng chọn trong modal không phát Auto-Lookup rồi thay nội dung phía sau.
useSelectionSurface(panel, 'display')

/**
 * 🔵 THÊM 2026-08-26 (cụm F ⑦, vòng rà 1 — Ice chốt) — `<ul class="gm-list" role="listbox">`,
 * template ref RIÊNG khỏi `panel`. Bản đầu của mục ⑦ đặt `aria-activedescendant` lên `<ul>`
 * nhưng KHÔNG BAO GIỜ focus nó — tiêu điểm luôn ở `panel` (`role="dialog"`), và WAI-ARIA chỉ
 * cho trình đọc màn hình tôn trọng `aria-activedescendant` trên phần tử ĐANG giữ tiêu điểm.
 * Ice chốt: cho `<ul>` một `tabindex="-1"` (template) và focus CHÍNH NÓ thay `panel` khi
 * danh sách có hàng — KHÔNG chuyển thuộc tính sang `panel`, vì `panel` mang `role="dialog"`
 * và ARIA 1.2 không liệt `dialog` trong các vai nhận `aria-activedescendant`.
 */
const list = useTemplateRef<HTMLElement>('list')

/**
 * Tiêu điểm ban đầu của lớp phủ: `<ul>` khi danh sách CÓ hàng (để `aria-activedescendant`
 * có nghĩa ngay từ đầu), `panel` khi không (đang tải / rỗng / lỗi — `<ul>` không render ở
 * các trạng thái đó, xem template `v-if="manageFilteredRows.length === 0"`).
 */
function focusInitialTarget(): void {
  if (manageStatus.value === 'loaded' && manageFilteredRows.value.length > 0 && list.value !== null) {
    list.value.focus()
    return
  }
  panel.value?.focus()
}

watch(manageOverlayIsOpen, (open) => {
  if (open) {
    // 🔴 KHÔNG lưu `document.activeElement` trần — xem `focusReturnTargetOnOpen`.
    returnFocusTo = focusReturnTargetOnOpen('[data-glossary-manage-open]')
    void nextTick(focusInitialTarget)
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  // Một node đã rời DOM vẫn nhận `focus()` mà KHÔNG ném và KHÔNG có tác dụng — tiêu điểm
  // rơi về `body` (UX-DR17), không dấu hiệu nào báo.
  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-glossary-manage-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  // Chẩn đoán tiếng Anh — Kiểm A của `check:i18n` cấm chuỗi tiếng Việt ở vị trí mã.
  console.warn('[glossary-manage] focus-return target is gone; focus falls back to body.')
})

/**
 * 🔵 THÊM 2026-08-26 (cụm F ⑦, vòng rà 1) — `openGlossaryManage()` đặt `manageOverlayIsOpen`
 * THÀNH `true` NGAY, rồi mới `await` lượt tải danh sách (`glossaryManageState.ts`) — nên tại
 * thời điểm watcher trên chạy, danh sách gần như luôn CHƯA có hàng (`manageStatus` vẫn
 * `'unknown'`), và `focusInitialTarget()` đúng đắn rơi về `panel`. Watcher RIÊNG này bắt lượt
 * danh sách chuyển sang "có hàng" SAU ĐÓ (dù lúc mở lần đầu, hay một bộ lọc vừa cho ra kết
 * quả) và chuyển tiêu điểm từ `panel` sang `<ul>` — CHỈ khi tiêu điểm còn đang ở ĐÚNG `panel`
 * (tức người dùng CHƯA chủ động chuyển nó đi đâu, ví dụ gõ vào ô tìm) — không bao giờ cướp
 * tiêu điểm khỏi một ô người dùng đang thao tác.
 */
watch(manageFilteredRows, () => {
  if (!manageOverlayIsOpen.value) return
  if (manageStatus.value !== 'loaded' || manageFilteredRows.value.length === 0) return
  if (document.activeElement !== panel.value) return
  void nextTick(() => list.value?.focus())
})

/**
 * 🔴 P1 (vòng rà 2, 2026-08-26) — HỒI QUY do chính watcher trên gây ra, ba nguồn độc lập
 * cùng bắt. Watcher trên chỉ có chiều 0 → N (không hàng → có hàng). Chiều NGƯỢC (N → 0)
 * thiếu: tiêu điểm đang ở `<ul>` → người dùng xoá hàng CUỐI CÙNG (nhịp `Backspace`/`Delete`
 * hai lượt) → danh sách nạp lại RỖNG → `v-if="manageFilteredRows.length === 0"` GỠ HẲN
 * `<ul>` khỏi DOM trong khi nó đang giữ tiêu điểm → trình duyệt đẩy tiêu điểm về
 * `document.body`.
 *
 * Hậu quả nặng hơn một lỗi tiêu điểm đơn thuần: `trapTab`/`onEscape`/`onKeydown` đều gắn
 * trên `.gm-scrim` và nghe qua BUBBLING từ `document.activeElement`. Tiêu điểm ở `body`
 * nằm NGOÀI `.gm-scrim` ⇒ Tab, Escape, mũi tên, và cả nhịp xoá hai lượt NGỪNG PHẢN HỒI cho
 * tới khi người dùng bấm chuột vào modal. TRƯỚC vòng rà 1 lỗi này không tồn tại: tiêu điểm
 * luôn ở `panel` (nằm TRONG `.gm-scrim`), nên phím vẫn bubbling — bản vá ⑦ (chuyển tiêu
 * điểm sang `<ul>`) mở ra nó.
 *
 * Vệ ĐỐI XỨNG với watcher trên: khi danh sách VỀ rỗng VÀ tiêu điểm đang ở ĐÚNG `<ul>` (tức
 * nó chưa bị người dùng chuyển đi đâu khác — cùng kỷ luật "không cướp tiêu điểm" của watcher
 * kia), dời tiêu điểm về `panel` — nơi nó SỐNG TRONG `.gm-scrim`, giữ Tab/Escape/mũi tên
 * bubbling được.
 */
watch(manageFilteredRows, () => {
  if (!manageOverlayIsOpen.value) return
  if (manageFilteredRows.value.length > 0) return
  if (document.activeElement !== list.value) return
  void nextTick(() => panel.value?.focus())
})

function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
}

/**
 * Bẫy tiêu điểm — điều kiện để `aria-modal="true"` không phải một lời khai sai.
 *
 * 🔵 XÉT LẠI 2026-08-26 (cụm F ⑦, vòng rà 1) — tiêu điểm nay có thể xuất phát từ `<ul
 * class="gm-list">`, không chỉ từ `panel`. KHÔNG cần sửa: `focusableWithin(root)` đã loại
 * MỌI phần tử `tabindex="-1"` khỏi `stops` (`:not([tabindex="-1"])`), và `<ul>`/`<li>` đều
 * mang thuộc tính đó — nên `<ul>` không bao giờ lọt vào `stops`, đúng như `panel` chưa từng
 * lọt vào trước đây. `active instanceof HTMLElement && stops.indexOf(active) === -1` đã sẵn
 * một nhánh cho "tiêu điểm đang ở một phần tử NGOÀI `stops`" (rơi về đầu/cuối danh sách tuỳ
 * `shiftKey`) — nhánh đó áp ĐÚNG như nhau cho cả hai điểm xuất phát `panel`/`<ul>`, không
 * phân biệt. Ca `..._focus_moves_off_the_list_into_the_toolbar_on_tab` canh đúng mệnh đề này.
 */
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

/**
 * `id` DOM ổn định cho một hàng của `.gm-list` — 🔵 THÊM 2026-08-26 (cụm F ⑦). Dùng CHUNG
 * giữa `<li :id>` và `<ul :aria-activedescendant>` bên dưới, để hai bên không thể lệch
 * nhau bằng cách chép công thức ghép chuỗi ở hai chỗ. Cùng định danh mà `:key` của `<li>`
 * đã dùng (`${row.tier}:${row.id}`) — chỉ đổi `:` thành `-` vì `id` không cấm ký tự đó
 * nhưng một `id` mang `:` gây rắc rối với `querySelector` (`:` mở một pseudo-class).
 */
function manageOptionId(row: Pick<GlossaryEntry, 'tier' | 'id'>): string {
  return `gm-option-${row.tier}-${row.id}`
}

/** Bốn phân loại — khớp `Category::as_str()` phía Rust, khuôn `CATEGORY_OPTIONS` của
 * `GlossaryQuickAdd.vue`/`GlossaryQueueOverlay.vue` (không mang hậu tố phím số — màn hình
 * này không dùng phím số cho phân loại). */
const CATEGORY_OPTIONS: ReadonlyArray<{ value: GlossaryCategory; labelKey: string }> = [
  { value: 'person', labelKey: 'glossary.manage.category_person' },
  { value: 'place', labelKey: 'glossary.manage.category_place' },
  { value: 'domain_term', labelKey: 'glossary.manage.category_domain_term' },
  { value: 'other', labelKey: 'glossary.manage.category_other' },
]

const ORIGIN_OPTIONS: ReadonlyArray<{ value: GlossaryManageOriginFilter; labelKey: string }> = [
  { value: 'all', labelKey: 'glossary.manage.origin_filter_all' },
  { value: 'manual', labelKey: 'glossary.manage.origin_manual' },
  { value: 'import_scan', labelKey: 'glossary.manage.origin_import_scan' },
  { value: 'review_harvest', labelKey: 'glossary.manage.origin_review_harvest' },
]

const TIER_OPTIONS: ReadonlyArray<{ value: GlossaryManageTierFilter; labelKey: string }> = [
  { value: 'global', labelKey: 'glossary.manage.tier_global' },
  { value: 'work', labelKey: 'glossary.manage.tier_work' },
  { value: 'both', labelKey: 'glossary.manage.tier_both' },
]

const ORIGIN_KEYS: Readonly<Partial<Record<string, string>>> = {
  manual: 'glossary.manage.origin_manual',
  import_scan: 'glossary.manage.origin_import_scan',
  review_harvest: 'glossary.manage.origin_review_harvest',
  file_import: 'glossary.manage.origin_file_import',
}

const hasShadowedRows = computed(() => manageFilteredRows.value.some((row) => row.is_shadowed))

function originLabel(termOrigin: string): string {
  const key = ORIGIN_KEYS[termOrigin]
  return key === undefined ? termOrigin : t(key)
}

function addedDate(createdAt: string): string {
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(createdAt)
  return m === null ? '—' : `${m[3]}-${m[2]}-${m[1]}`
}

function tierCountVisible(value: GlossaryManageTierFilter): boolean {
  return manageStatus.value === 'loaded' && (value !== 'work' || manageWorkTierAvailable.value)
}

function marginBarClass(row: GlossaryEntry): string {
  if (row.translation === null) return 'gm-row-bar-pending'
  return row.tier === 'work' ? 'gm-row-bar-work' : ''
}

function aimRow(index: number): void {
  setGlossaryManageCursor(index)
}

watch(manageEditing, (editing) => {
  void nextTick(() => {
    if (editing) {
      panel.value?.querySelector<HTMLElement>('[data-gm-edit-translation]')?.focus()
      return
    }
    if (manageOverlayIsOpen.value && !panel.value?.contains(document.activeElement)) list.value?.focus()
  })
})

function onRowEscape(event: KeyboardEvent, index: number): void {
  if (!manageEditing.value || index !== manageCursor.value) return
  event.preventDefault()
  event.stopPropagation()
  dispatch('glossary.manage.cancel')
}

function onTierChange(value: GlossaryManageTierFilter): void {
  setGlossaryManageTierFilter(value)
}

function onPendingToggle(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) setGlossaryManageConfirmedFilter(target.checked ? 'pending' : 'all')
}

function onSearchInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) setGlossaryManageSearch(target.value)
}

function onCategoryChange(value: GlossaryCategory | 'all'): void {
  setGlossaryManageCategoryFilter(value)
}

function onOriginFilterChange(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLSelectElement)) return
  setGlossaryManageOriginFilter(target.value as GlossaryManageOriginFilter)
}

function onSortByFrequencyChange(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  setGlossaryManageSortByFrequency(target.checked)
}

/** Story 3.10b — radio (KHÔNG `<select>`, hai lựa chọn cố định) chọn tầng Xuất/Nhập. */
function onExchangeTierChange(value: GlossaryTierWire, event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement) || !target.checked) return
  setGlossaryManageExchangeTier(value)
}

/**
 * 🔴 Cụm D vá — `Escape` khi đang ở nhịp MỘT (chờ xác nhận xoá) HUỶ nhịp đó, KHÔNG đóng cả
 * lớp phủ (§Always: "Escape huỷ nhịp một"). Chỉ khi KHÔNG có gì đang chờ xác nhận thì
 * `Escape` mới rơi về hành vi cũ — đóng lớp phủ, đúng một lời gọi `dispatch(...)`.
 */
function onEscape(): void {
  if (manageDeletePending.value) {
    cancelGlossaryManageDeleteConfirm()
    return
  }
  dispatch('glossary.manage.close')
}

/**
 * 🔴 Cùng khuôn `GlossaryQueueOverlay.vue::onKeydown` — filter `ctrlKey`/`metaKey`/`altKey`
 * TRƯỚC MỌI NHÁNH (§Always: quên nó thì `⌘Backspace` xoá một mục ngoài ý định).
 *
 * 🔵 THÊM (khác Queue) — filter TIẾP theo TARGET: lớp phủ này mang ô gõ chữ thật (ô tìm,
 * form Sửa). Không có bước này, gõ `Backspace` để xoá một ký tự trong ô Ghi chú sẽ kích
 * hoạt `glossary.manage.delete` — mất đúng mục đang sửa.
 */
function onKeydown(event: KeyboardEvent): void {
  if (event.ctrlKey || event.metaKey || event.altKey) return

  const target = event.target
  // 🔴 `HTMLButtonElement` nằm trong danh sách này từ bản vá 2026-08-24 (vòng rà ba lớp).
  // Listener gắn trên `.gm-scrim`, tức TỔ TIÊN của mọi nút trong lớp phủ, nên một nút đang
  // giữ tiêu điểm vẫn rơi vào `case 'Enter'` bên dưới: bấm Enter trên nút "Xoá" hay "Đẩy lên
  // Toàn cục" gọi `preventDefault()` rồi `dispatch('glossary.manage.edit')` — nó không phải
  // một phím chết, nó làm VIỆC KHÁC. `Space` thì thoát vì rơi vào `default`, nên lỗi chỉ lộ
  // ở nửa số cách kích hoạt một nút, và AC7 ("mọi thao tác làm được bằng bàn phím") xanh giả.
  const isFormField =
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    target instanceof HTMLButtonElement
  if (isFormField) return

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      dispatch('glossary.manage.next')
      return
    case 'ArrowUp':
      event.preventDefault()
      dispatch('glossary.manage.prev')
      return
    case 'Enter':
      event.preventDefault()
      dispatch('glossary.manage.edit')
      return
    case 'Backspace':
    case 'Delete':
      // 🔴 THÊM (vòng rà thứ hai, #1 — NẶNG NHẤT) — `event.repeat` CHỈ lọc ở NHÁNH NÀY, không
      // ở đầu hàm: `ArrowUp`/`ArrowDown` phải giữ được tự lặp (giữ phím để chạy nhanh qua
      // danh sách là thao tác bình thường). Xoá thì khác — `deleteGlossaryManageEntry` nhịp
      // MỘT trả về ĐỒNG BỘ và không đặt cờ bận nào, nên một cú nhấn-GIỮ (hệ điều hành tự lặp
      // `keydown`) chạy nhịp một rồi nhịp hai trong đúng MỘT thao tác người dùng cảm nhận là
      // một lần bấm — đúng kịch bản mà cả tính năng hai-nhịp tồn tại để chặn.
      if (event.repeat) return
      event.preventDefault()
      dispatch('glossary.manage.delete')
      return
    default:
      return
  }
}
</script>

<template>
  <div
    v-if="manageOverlayIsOpen"
    class="gm-scrim"
    @keydown.esc="onEscape"
    @keydown.tab="trapTab($event)"
    @keydown="onKeydown"
  >
    <section ref="panel" class="gm-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="gm-head">
        <h2 class="gm-title">{{ t('glossary.manage.title') }}</h2>
        <button type="button" class="gm-close" @click="dispatch('glossary.manage.close')">
          {{ t('command.glossary.manage.close') }}
        </button>
      </header>

      <!--
        🔵 `manageStatus === 'loaded'` là VẾ BẮT BUỘC (vá 2026-08-24). Thiếu nó, ghi chú nháy
        lên ngay lúc lớp phủ vừa mở — `workTierAvailable` khởi tạo `false` và lượt dò
        `lookupGlossaryTerm('')` còn đang bay — nên một người ĐANG mở Tác phẩm vẫn đọc thấy
        câu "chưa mở Tác phẩm nào" trong khoảng round-trip đầu. Một câu khẳng định sai trong
        200 ms vẫn là một câu khẳng định sai.
      -->
      <p v-if="manageStatus === 'loaded' && manageWorkTierAvailable === false" class="gm-note">
        {{ t('glossary.manage.work_tier_unavailable_note') }}
      </p>

      <div class="gm-toolbar">
        <fieldset class="gm-seg" role="radiogroup" :aria-label="t('glossary.manage.tier_label')">
          <label
            v-for="opt in TIER_OPTIONS"
            :key="opt.value"
            class="gm-seg-opt"
            :class="{ 'gm-seg-on': manageTierFilter === opt.value }"
          >
            <input
              type="radio"
              name="gm-tier"
              class="gm-sr-only"
              :checked="manageTierFilter === opt.value"
              :disabled="manageEditing || (opt.value === 'work' && !manageWorkTierAvailable)"
              @change="onTierChange(opt.value)"
            />
            {{ t(opt.labelKey) }}
            <!-- aura-allow-text: DỮ LIỆU (số hàng đếm được ở nấc này). -->
            <b v-if="tierCountVisible(opt.value)" class="gm-seg-count">{{ manageTierCounts[opt.value] }}</b>
          </label>
        </fieldset>

        <!-- `:disabled` here is UX only; the real guard is `isEditingBlocked` in
             glossaryManageState.ts. -->
        <label class="gm-field gm-field-search">
          <span class="gm-sr-only">{{ t('glossary.manage.search_label') }}</span>
          <input
            type="text"
            class="gm-input"
            autocomplete="off"
            :placeholder="t('glossary.manage.search_label')"
            :value="manageSearchQuery"
            :disabled="manageEditing"
            @input="onSearchInput"
          />
        </label>

        <fieldset class="gm-pills" role="radiogroup" :aria-label="t('glossary.manage.category_filter_label')">
          <label class="gm-pill" :class="{ 'gm-pill-on': manageCategoryFilter === 'all' }">
            <input
              type="radio"
              name="gm-category"
              class="gm-sr-only"
              :checked="manageCategoryFilter === 'all'"
              :disabled="manageEditing"
              @change="onCategoryChange('all')"
            />
            {{ t('glossary.manage.category_all') }}
          </label>
          <label
            v-for="opt in CATEGORY_OPTIONS"
            :key="opt.value"
            class="gm-pill"
            :class="{ 'gm-pill-on': manageCategoryFilter === opt.value }"
          >
            <input
              type="radio"
              name="gm-category"
              class="gm-sr-only"
              :checked="manageCategoryFilter === opt.value"
              :disabled="manageEditing"
              @change="onCategoryChange(opt.value)"
            />
            {{ t(opt.labelKey) }}
          </label>
        </fieldset>

        <label
          v-if="manageStatus === 'loaded'"
          class="gm-pill gm-pill-pending"
          :class="{ 'gm-pill-on': manageConfirmedFilter === 'pending' }"
          :title="t('glossary.manage.pending_pill_title')"
        >
          <input
            type="checkbox"
            class="gm-sr-only"
            :checked="manageConfirmedFilter === 'pending'"
            :disabled="manageEditing"
            @change="onPendingToggle"
          />
          <!-- aura-allow-text: KẾT QUẢ của `t()` (đã nội suy số đếm). -->
          {{ t('glossary.manage.pending_pill', { count: String(managePendingCount) }) }}
        </label>

        <label class="gm-field gm-field-origin">
          <span class="gm-sr-only">{{ t('glossary.manage.origin_filter_label') }}</span>
          <select class="gm-input" :value="manageOriginFilter" :disabled="manageEditing" @change="onOriginFilterChange">
            <option v-for="opt in ORIGIN_OPTIONS" :key="opt.value" :value="opt.value">{{ t(opt.labelKey) }}</option>
          </select>
        </label>

        <!-- Not disabled while editing: sorting only reorders, it doesn't change the row set. -->
        <label class="gm-field gm-field-checkbox gm-sort">
          <input
            type="checkbox"
            :aria-label="t('glossary.manage.sort_by_frequency_label')"
            :checked="manageSortByFrequency"
            @change="onSortByFrequencyChange"
          />
          <span class="gm-field-label">
            {{ t('glossary.manage.sort_prefix') }}
            <b>{{ t(manageSortByFrequency ? 'glossary.manage.sort_current_frequency' : 'glossary.manage.sort_current_default') }}</b>
          </span>
        </label>
      </div>

      <!-- BỐN ca rỗng KHÁC NHAU, cộng nhánh "đang tải" — §Always: "rỗng phải nói vì sao nó rỗng". -->
      <p v-if="manageStatus === 'unknown'" class="gm-status" role="status">{{ t('glossary.manage.loading') }}</p>
      <p
        v-else-if="manageEmptyReasonFor(manageStatus, 0, 0) === 'ipc_unavailable'"
        class="gm-empty"
      >
        {{ t('glossary.manage.empty_ipc_unavailable') }}
      </p>
      <p v-else-if="manageStatus === 'error' && manageLoadError !== null" class="gm-empty gm-error" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(manageLoadError) }}
      </p>
      <p
        v-else-if="manageEmptyReasonFor(manageStatus, manageTotalRows, manageFilteredRows.length) === 'glossary_empty'"
        class="gm-empty"
      >
        {{ t('glossary.manage.empty_glossary') }}
      </p>

      <template v-else-if="manageStatus === 'loaded'">
        <p v-if="manageFilteredRows.length === 0" class="gm-empty">
          {{ t('glossary.manage.empty_filter_no_match') }}
        </p>

        <!--
          `role="listbox"`/`option` + `aria-selected` (vá 2026-08-24) + `id`/
          `aria-activedescendant`/tiêu điểm (🔵 SỬA 2026-08-26, cụm F ⑦, vòng rà 1).
          Bản 2026-08-24 gắn `aria-selected` trên MỖI hàng, nhưng tiêu điểm DOM sống ở
          `panel` (`role="dialog"`) chứ không đi theo `manageCursor` — `aria-selected` một
          mình không nói được "con trỏ đang ở đâu" cho trình đọc màn hình (nó chỉ nói "hàng
          nào ĐƯỢC CHỌN", một câu khác).

          Bản đầu của mục ⑦ (cùng ngày) thêm `aria-activedescendant` lên `<ul>` này nhưng
          KHÔNG cho `<ul>` `tabindex` và KHÔNG bao giờ focus nó — chú thích lúc đó khai `<ul>`
          là "container giữ tiêu điểm", trong khi tiêu điểm THẬT vẫn ở `panel`. WAI-ARIA chỉ
          tôn trọng `aria-activedescendant` trên phần tử ĐANG giữ tiêu điểm, nên bản đó không
          giao được điều nó khai — một chú thích "đã đóng lỗ hổng" trong khi lỗ hổng vẫn còn,
          đúng lớp lỗi `khoi-phuc-trung-thanh-khong-phai-dung`.

          Bản NÀY: `<ul>` mang `tabindex="-1"` (TypeScript, `list.value?.focus()`) và tiêu
          điểm chuyển TỚI CHÍNH `<ul>` khi danh sách có hàng (`focusInitialTarget`/watcher
          trên `manageFilteredRows`, xem khối kịch bản phía trên) — KHÔNG chuyển `aria-
          activedescendant` sang `panel` (`role="dialog"` không được ARIA 1.2 liệt trong các
          vai nhận thuộc tính này). `tabindex="-1"` trên `<li>` giữ HÀNG ngoài vòng Tab —
          điều hướng hàng là việc của mũi tên (bắt ở `.gm-scrim`), `Tab` dành cho toolbar/nút
          thao tác; `<ul>` CŨNG mang `tabindex="-1"` vì nó nhận tiêu điểm bằng JS, không bằng
          Tab (cùng khuôn `panel` — cả hai đều KHÔNG nằm trong `stops` của `trapTab`).
        -->
        <table
          v-else
          ref="list"
          class="gm-list"
          role="grid"
          tabindex="-1"
          :aria-label="t('glossary.manage.list_label')"
          :aria-activedescendant="manageCurrentRow === null ? undefined : manageOptionId(manageCurrentRow)"
        >
          <thead>
            <tr>
              <th scope="col">{{ t('glossary.manage.col_term') }}</th>
              <th scope="col">{{ t('glossary.manage.col_translation') }}</th>
              <th scope="col">{{ t('glossary.manage.col_category') }}</th>
              <th scope="col">{{ t('glossary.manage.col_origin') }}</th>
              <th scope="col">{{ t('glossary.manage.col_count') }}</th>
              <th scope="col">{{ t('glossary.manage.col_note') }}</th>
              <th scope="col">{{ t('glossary.manage.col_added') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(row, i) in manageFilteredRows"
              :id="manageOptionId(row)"
              :key="`${row.tier}:${row.id}`"
              class="gm-row"
              tabindex="-1"
              :aria-selected="i === manageCursor"
              :class="[
                marginBarClass(row),
                {
                  'gm-row-current': i === manageCursor,
                  'gm-row-shadowed': row.is_shadowed,
                  'gm-row-delete-pending': i === manageCursor && manageDeletePending,
                },
              ]"
              @mousedown="aimRow(i)"
              @dblclick="dispatch('glossary.manage.edit')"
              @keydown.esc="onRowEscape($event, i)"
            >
              <td class="gm-cell-term"><div class="gm-term-flex">
                <!-- aura-allow-text: DỮ LIỆU (`source_term` của chính hàng). -->
                <span class="gm-term">{{ row.source_term }}</span>
                <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
                <span class="gm-badge">{{ row.tier === 'global' ? t('glossary.quick_add.tier_global') : t('glossary.quick_add.tier_work') }}</span>
                <span v-if="row.is_shadowed" class="gm-badge gm-badge-shadowed">{{ t('glossary.manage.shadowed_badge') }}</span>
                <span v-if="i === manageCursor && manageDeletePending" class="gm-badge gm-badge-delete-pending">
                  {{ t('glossary.manage.delete_confirm_badge') }}
                </span>
                <span class="gm-sr-only">{{ t(row.translation === null ? 'glossary.manage.status_pending' : 'glossary.manage.status_confirmed') }}</span>
              </div></td>
              <td>
                <input
                  v-if="i === manageCursor && manageEditing"
                  v-model="manageEditTranslation"
                  form="gm-edit-form"
                  type="text"
                  class="gm-input gm-cell-input"
                  autocomplete="off"
                  data-gm-edit-translation
                  :aria-label="t('glossary.manage.translation_label')"
                  :disabled="manageSaving"
                />
                <!-- aura-allow-text: DỮ LIỆU (`translation` của chính hàng). -->
                <span v-else-if="row.translation !== null" class="gm-translation">{{ row.translation }}</span>
                <span v-else class="gm-badge gm-badge-pending">{{ t('glossary.manage.pending_badge') }}</span>
              </td>
              <td>
                <select
                  v-if="i === manageCursor && manageEditing"
                  v-model="manageEditCategory"
                  form="gm-edit-form"
                  class="gm-input gm-cell-input"
                  :aria-label="t('glossary.quick_add.category_label')"
                  :disabled="manageSaving"
                >
                  <option v-for="opt in CATEGORY_OPTIONS" :key="opt.value" :value="opt.value">{{ t(opt.labelKey) }}</option>
                </select>
                <span v-else class="gm-category">{{ t(`glossary.manage.category_${row.category}`) }}</span>
              </td>
              <!-- aura-allow-text: KẾT QUẢ của `t()` hoặc DỮ LIỆU (xuất xứ lạ hiện nguyên giá trị). -->
              <td class="gm-origin">{{ originLabel(row.term_origin) }}</td>
              <td>
                <!-- aura-allow-text: DỮ LIỆU (`occurrence_count` của chính hàng) hoặc dấu "—" cố định. -->
                <span class="gm-count" :title="t('glossary.manage.occurrence_count_title')">{{
                  row.occurrence_count === null ? '—' : row.occurrence_count
                }}</span>
              </td>
              <td class="gm-cell-note">
                <input
                  v-if="i === manageCursor && manageEditing"
                  v-model="manageEditNote"
                  form="gm-edit-form"
                  type="text"
                  class="gm-input gm-cell-input"
                  autocomplete="off"
                  :aria-label="t('glossary.manage.note_label')"
                  :disabled="manageSaving"
                />
                <!-- aura-allow-text: DỮ LIỆU (`note` của chính hàng). -->
                <span v-else class="gm-note-cell" :title="row.note">{{ row.note }}</span>
              </td>
              <!-- aura-allow-text: KẾT QUẢ của `addedDate()` (ngày cắt từ `created_at`, hoặc dấu "—"). -->
              <td class="gm-added">{{ addedDate(row.created_at) }}</td>
            </tr>
          </tbody>
        </table>

        <div v-if="managePendingCount > 0" class="gm-box gm-box-pending">
          <b>{{ t('glossary.manage.pending_box_title', { count: String(managePendingCount) }) }}</b>
          {{ t('glossary.manage.pending_box_body') }}
        </div>
        <div v-if="hasShadowedRows" class="gm-box gm-box-shadow">
          <b>{{ t('glossary.manage.shadow_box_title') }}</b>
          {{ t('glossary.manage.shadow_box_body') }}
        </div>
      </template>

      <!-- Inline edit: the three inputs live in the cursor row and join this form by `form="gm-edit-form"`. -->
      <form
        v-if="manageEditing && manageCurrentRow !== null"
        id="gm-edit-form"
        class="gm-edit-form"
        @submit.prevent="dispatch('glossary.manage.save')"
        @keydown.esc.prevent.stop="dispatch('glossary.manage.cancel')"
      >
        <div class="gm-edit-actions">
          <button type="submit" class="gm-act gm-act-primary" :disabled="manageSaving">
            {{ t('glossary.manage.save') }}
          </button>
          <button type="button" class="gm-act" :disabled="manageSaving" @click="dispatch('glossary.manage.cancel')">
            {{ t('glossary.manage.cancel') }}
          </button>
        </div>
      </form>

      <p v-if="manageActionError !== null" class="gm-status gm-error" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(manageActionError) }}
      </p>
      <p v-else-if="manageActionNotice === 'promote_not_applicable'" class="gm-status" role="status">
        {{ t('glossary.manage.promote_not_applicable') }}
      </p>
      <!--
        Câu trạng thái theo ĐÚNG thao tác đang bay. Trước bản vá 2026-08-24 cả ba lượt ghi
        dùng chung `glossary.manage.saving`, nên xoá một mục thì màn hình nói "Đang lưu…".
      -->
      <p v-else-if="manageSaving" class="gm-status" role="status">
        {{
          t(
            manageSavingAction === 'delete'
              ? 'glossary.manage.deleting'
              : manageSavingAction === 'promote'
                ? 'glossary.manage.promoting'
                : 'glossary.manage.saving',
          )
        }}
      </p>
      <!--
        Cụm D vá — nhịp MỘT của lượt xoá: chữ RIÊNG (không chỉ đổi nút), cùng độ ưu tiên với
        `manageSaving` (không đang lưu khi đang chờ xác nhận, hai nhánh không chồng nhau).
      -->
      <p v-else-if="manageDeletePending" class="gm-status gm-error" role="status">
        {{ t('glossary.manage.delete_confirm_hint') }}
      </p>

      <div v-if="!manageEditing" class="gm-actions">
        <button type="button" class="gm-act" :disabled="manageCurrentRow === null" @click="dispatch('glossary.manage.edit')">
          {{ t('glossary.manage.edit') }}
        </button>
        <button
          type="button"
          class="gm-act"
          :class="{ 'gm-act-danger': manageDeletePending }"
          :disabled="manageCurrentRow === null"
          @click="dispatch('glossary.manage.delete')"
        >
          {{ t(manageDeletePending ? 'glossary.manage.delete_confirm_button' : 'glossary.manage.delete') }}
        </button>
        <button type="button" class="gm-act" :disabled="manageCurrentRow === null" @click="dispatch('glossary.manage.promote')">
          {{ t('glossary.manage.promote') }}
        </button>
        <button type="button" class="gm-act" @click="dispatch('glossary.manage.prev')">
          {{ t('glossary.manage.prev') }}
        </button>
        <button type="button" class="gm-act" @click="dispatch('glossary.manage.next')">
          {{ t('glossary.manage.next') }}
        </button>
      </div>

      <!--
        Story 3.10b (AD-48) — hộp thoại chọn tệp gọi TỪ RUST. Tầng đang chọn ở đây quyết
        định CẢ Xuất lẫn Nhập (`dispatch` không nhận tham số — handler tiêm đọc
        `manageExchangeTier` ngay trước khi mở hộp thoại, xem `glossaryManageState.ts`).
      -->
      <div class="gm-exchange">
        <fieldset class="gm-exchange-tier" role="radiogroup" :aria-label="t('glossary.manage.exchange_tier_label')">
          <legend class="gm-field-label">{{ t('glossary.manage.exchange_tier_label') }}</legend>
          <label class="gm-radio-label">
            <input
              type="radio"
              name="gm-exchange-tier"
              :checked="manageExchangeTier === 'global'"
              @change="onExchangeTierChange('global', $event)"
            />
            {{ t('glossary.quick_add.tier_global') }}
          </label>
          <label class="gm-radio-label">
            <input
              type="radio"
              name="gm-exchange-tier"
              :disabled="!manageWorkTierAvailable"
              :checked="manageExchangeTier === 'work'"
              @change="onExchangeTierChange('work', $event)"
            />
            {{ t('glossary.quick_add.tier_work') }}
          </label>
        </fieldset>

        <div class="gm-exchange-actions">
          <!--
            Cụm D vá — Xuất và Nhập LOẠI TRỪ lẫn nhau: hai hộp thoại hệ điều hành cùng bay là
            một trạng thái không ai định nghĩa hành vi (§Tasks). `glossaryExchangeBusy` là cờ
            DÙNG CHUNG (xem `glossaryExchangeGate.ts`) — đúng khi MỘT TRONG HAI đang mở hộp
            thoại, không chỉ cờ RIÊNG của chính nút này.
          -->
          <button
            type="button"
            class="gm-act"
            :disabled="glossaryExchangeBusy"
            @click="dispatch('glossary.manage.export_csv')"
          >
            {{ t('glossary.manage.export_csv') }}
          </button>
          <button
            type="button"
            class="gm-act"
            data-glossary-import-open
            :disabled="glossaryExchangeBusy"
            @click="dispatch('glossary.manage.import_csv')"
          >
            {{ t('glossary.manage.import_csv') }}
          </button>
        </div>

        <!--
          🔴 THÊM (vòng rà thứ hai, #10) — `:disabled="glossaryExchangeBusy"` làm MỘT trong
          hai nút xám đi mà không một câu nào khi chính THAO TÁC KIA (không phải thao tác của
          nút này) đang mở hộp thoại — trái nguyên tắc "rỗng/khoá phải nói vì sao" của kho.
          Đứng ĐẦU chuỗi `v-if`: đây là tin MỚI NHẤT (lý do nút vừa xám đi ngay bây giờ), ưu
          tiên hơn một kết quả CŨ còn sót của chính lượt Xuất trước đó.
        -->
        <p v-if="glossaryExchangeBusy && !manageExportBusy" class="gm-status" role="status">
          {{ t('glossary.manage.exchange_busy_other') }}
        </p>
        <p v-else-if="manageExportError !== null" class="gm-status gm-error" role="alert">
          <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
          {{ tError(manageExportError) }}
        </p>
        <!--
          🔴 P2 (vòng rà ba lớp 2026-08-25) — nhánh RIÊNG cho "không có cầu IPC", tách khỏi
          huỷ hộp thoại (huỷ không đổi state nào, nên không rơi vào nhánh nào ở đây cả).
        -->
        <p v-else-if="manageExportIpcUnavailable" class="gm-status" role="status">
          {{ t('glossary.manage.export_ipc_unavailable') }}
        </p>
        <p v-else-if="manageExportBusy" class="gm-status" role="status">{{ t('glossary.manage.exporting') }}</p>
        <p v-else-if="manageExportedPath !== null" class="gm-status" role="status">
          <!-- aura-allow-text: KẾT QUẢ của `t()` (đã nội suy đường dẫn qua tham số). -->
          {{ t('glossary.manage.export_done', { path: manageExportedPath }) }}
        </p>
      </div>

      <div class="gm-foot">
        <button type="button" class="gm-act gm-act-primary" @click="dispatch('glossary.manage.add_term')">
          {{ t('glossary.manage.add_term') }}
        </button>
        <span class="gm-foot-hint">{{ t('glossary.manage.foot_hint') }}</span>
      </div>
    </section>
  </div>
</template>

<style scoped>
.gm-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: xếp lớp CƠ HỌC — cùng lý do các lớp phủ khác. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.gm-panel {
  width: 100%;
  max-width: 1100px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.gm-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.gm-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.gm-close {
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

.gm-note {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gm-toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 4);
  margin-bottom: var(--space-panel-block);
}

.gm-field {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  flex: 1;
  min-width: 8rem;
}

/* Checkbox and label on one row, unlike `.gm-field`'s column layout. */
.gm-field-checkbox {
  flex-direction: row;
  align-items: center;
  flex: 0 0 auto;
}

.gm-field-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gm-input {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
}

.gm-status {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gm-empty {
  margin: 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.gm-error {
  color: var(--color-error);
}

.gm-list {
  width: 100%;
  margin: 0 0 var(--space-panel-block) 0;
  border-collapse: collapse;
  border: 1px solid var(--color-outline);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.gm-list th {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  text-align: left;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface-variant);
  border-bottom: 1px solid var(--color-outline);
}

.gm-row > td {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  vertical-align: top;
  border-bottom: 1px solid var(--color-outline);
}

/* Margin bar of the row status; the shapes mirror the editor margin (2px solid confirmed, 2px dashed draft). */
.gm-row > td:first-child {
  border-left: 2px solid transparent;
}

.gm-row-bar-work > td:first-child {
  border-left: 2px solid var(--color-confirmed);
}

.gm-row-bar-pending > td:first-child {
  border-left: 2px dashed var(--color-draft);
}

.gm-row-current {
  background: var(--color-surface-accent);
}

.gm-row-shadowed > td:first-child {
  padding-left: calc(var(--space-unit) * 8);
}

.gm-term-flex {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
}

.gm-cell-input {
  width: 100%;
}

.gm-category,
.gm-origin,
.gm-added {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gm-added {
  white-space: nowrap;
}

.gm-seg {
  display: flex;
  margin: 0;
  padding: 0;
  border: 1px solid var(--color-outline);
}

.gm-seg-opt,
.gm-pill {
  position: relative;
  display: inline-flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 1);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  cursor: pointer;
}

.gm-seg-opt + .gm-seg-opt {
  border-left: 1px solid var(--color-outline);
}

.gm-seg-on,
.gm-pill-on {
  background: var(--color-surface-accent);
  color: var(--color-primary);
  font-weight: var(--weight-ui-md-strong);
}

.gm-pills {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 1);
  margin: 0;
  padding: 0;
  border: none;
}

.gm-pill {
  border: 1px solid var(--color-outline);
}

.gm-pill-pending.gm-pill-on {
  border-color: var(--color-draft);
}

.gm-seg-opt:has(input:focus-visible),
.gm-pill:has(input:focus-visible) {
  outline: 2px solid var(--color-primary);
}

.gm-field-search {
  flex: 1;
  max-width: 18rem;
}

.gm-field-origin {
  flex: 0 0 auto;
  min-width: 0;
}

.gm-sort {
  margin-left: auto;
}

.gm-box {
  margin: 0 0 var(--space-panel-block) 0;
  padding: calc(var(--space-unit) * 3) var(--space-panel-inline);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  background: var(--color-background);
}

.gm-box-pending {
  border-left: 2px dashed var(--color-draft);
}

.gm-box-shadow {
  border-left: 2px solid var(--color-confirmed);
}

.gm-foot {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 3);
  margin-top: var(--space-panel-block);
}

.gm-foot-hint {
  margin-left: auto;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gm-sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  margin: -1px; /* aura-allow-spacing: kích thước hình học của kỹ thuật ẩn sr-only, không phải khoảng cách lưới */
  padding: 0;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

/*
 * Lùi hàng BỊ CHE bằng MÀU CHỮ — cấm `opacity`, không miễn trừ (§Always của spec).
 * Một hàng bị che vẫn có thật (AD-18: mục Global vẫn tồn tại, chỉ không thắng), nên nó
 * không "đã xử lý" như hàng của Story 3.8 — chỉ CHÍNH `.gm-badge-shadowed` mang màu lùi,
 * không cả hàng, để `source_term`/`tier` vẫn đọc rõ ràng ở độ tương phản đầy đủ.
 */
.gm-row-shadowed .gm-badge-shadowed {
  color: var(--color-on-surface-variant);
}

/*
 * Cụm D vá — trạng thái "chờ xác nhận xoá" phân biệt bằng chữ riêng ở
 * `.gm-badge-delete-pending` (span text + viền BỐN CẠNH của chính badge, xem khối dưới),
 * CỘNG nhãn nút đổi chữ ở template — không một mình `opacity` (§Always của spec).
 *
 * 🔵 SỬA (vòng rà thứ hai, #15) — câu trên bớt chữ "VIỀN...trên hàng": `.gm-row` (`:677-688`)
 * chỉ khai `border-bottom`, không bốn cạnh, nên quy tắc dưới đây chỉ đổi màu ĐÚNG MỘT nét
 * ĐÁY của hàng — một tín hiệu phụ, tinh tế, không phải "một cái viền quanh hàng". Tín hiệu
 * CHÍNH của nhịp chờ là chữ (badge + nhãn nút), đúng như câu trên (đã sửa) khai; nét đáy này
 * chỉ là một gợi ý thị giác thêm, không đứng một mình.
 */
.gm-row-delete-pending > td {
  border-bottom-color: var(--color-error);
}

.gm-badge-delete-pending {
  color: var(--color-error);
  border-color: var(--color-error);
}

.gm-term {
  font-weight: var(--weight-ui-md-strong);
}

.gm-badge {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  border: 1px solid var(--color-outline);
  padding: 0 calc(var(--space-unit) * 1);
}

.gm-badge-shadowed {
  /*
   * 🔵 SỬA 2026-08-24 (vòng rà ba lớp) — bỏ `color: var(--color-error)`. Nó là MÃ CHẾT: huy
   * hiệu này chỉ render bên trong `.gm-row-shadowed`, và `.gm-row-shadowed .gm-badge-shadowed`
   * ở trên có độ đặc hiệu (0,2,0) cao hơn (0,1,0) nên luôn thắng bất kể thứ tự nguồn. Kết quả
   * thật trên màn hình là viền đỏ + chữ xám, không phải chữ đỏ. Chữ xám ĐÚNG chủ ý đã ghi ở
   * khối trên; dòng bị bỏ mới là dòng lạc.
   */
  border-color: var(--color-error);
}

.gm-badge-pending {
  color: var(--color-on-surface-variant);
}

.gm-translation {
  color: var(--color-on-surface);
}

.gm-cell-note {
  max-width: 14rem;
}

.gm-note-cell {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--family-read);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gm-count {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface-variant);
}

.gm-edit-form {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 4);
  align-items: flex-end;
  margin-bottom: var(--space-panel-block);
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
}

.gm-edit-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}

.gm-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-panel-inline);
}

.gm-act {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: none;
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  cursor: pointer;
}

.gm-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.gm-act-primary {
  color: var(--color-on-surface);
  border-color: var(--color-primary);
}

/* Cụm D vá — nút Xoá ở nhịp HAI (chờ xác nhận): viền + chữ token màu lỗi, cộng nhãn nút đã
 * đổi chữ ở template. Không `opacity`, không màu viết thẳng (§Always của spec). */
.gm-act-danger {
  color: var(--color-error);
  border-color: var(--color-error);
}

/* Story 3.10b (AD-48) — hộp thoại chọn tệp. */
.gm-exchange {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
  margin-top: var(--space-panel-block);
  padding-top: var(--space-panel-block);
  border-top: 1px solid var(--color-outline);
}

.gm-exchange-tier {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 3);
  margin: 0;
  padding: 0;
  border: none;
}

.gm-radio-label {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
  cursor: pointer;
}

.gm-exchange-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}
</style>
