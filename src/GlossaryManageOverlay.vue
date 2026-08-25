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
import { nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import type { GlossaryCategory } from './config/glossary'
import type { GlossaryTierWire } from './config/glossary'
import { importOpening } from './glossaryImportState'
import {
  manageActionError,
  manageActionNotice,
  manageCategoryFilter,
  manageConfirmedFilter,
  manageCursor,
  manageCurrentRow,
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
  manageTotalRows,
  manageLoadError,
  manageOriginFilter,
  manageOverlayIsOpen,
  manageSaving,
  manageSavingAction,
  manageSearchQuery,
  manageStatus,
  manageWorkTierAvailable,
  setGlossaryManageCategoryFilter,
  setGlossaryManageConfirmedFilter,
  setGlossaryManageExchangeTier,
  setGlossaryManageOriginFilter,
  setGlossaryManageSearch,
} from './glossaryManageState'
import type { GlossaryManageConfirmedFilter, GlossaryManageOriginFilter } from './glossaryManageState'

/** 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Khuôn và lý lẽ chép từ `GlossaryQueueOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
// Lớp phủ chứa chữ thật (thuật ngữ, ghi chú) nhưng không phải nguồn từ điển. Đăng ký
// `display` để một vùng chọn trong modal không phát Auto-Lookup rồi thay nội dung phía sau.
useSelectionSurface(panel, 'display')

watch(manageOverlayIsOpen, (open) => {
  if (open) {
    // 🔴 KHÔNG lưu `document.activeElement` trần — xem `focusReturnTargetOnOpen`.
    returnFocusTo = focusReturnTargetOnOpen('[data-glossary-manage-open]')
    void nextTick(() => panel.value?.focus())
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

function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
}

/** Bẫy tiêu điểm — điều kiện để `aria-modal="true"` không phải một lời khai sai. */
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

const CONFIRMED_OPTIONS: ReadonlyArray<{ value: GlossaryManageConfirmedFilter; labelKey: string }> = [
  { value: 'all', labelKey: 'glossary.manage.confirmed_filter_all' },
  { value: 'confirmed', labelKey: 'glossary.manage.confirmed_filter_confirmed' },
  { value: 'pending', labelKey: 'glossary.manage.confirmed_filter_pending' },
]

function onSearchInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) setGlossaryManageSearch(target.value)
}

function onCategoryFilterChange(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLSelectElement)) return
  setGlossaryManageCategoryFilter(target.value as GlossaryCategory | 'all')
}

function onOriginFilterChange(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLSelectElement)) return
  setGlossaryManageOriginFilter(target.value as GlossaryManageOriginFilter)
}

function onConfirmedFilterChange(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLSelectElement)) return
  setGlossaryManageConfirmedFilter(target.value as GlossaryManageConfirmedFilter)
}

/** Story 3.10b — radio (KHÔNG `<select>`, hai lựa chọn cố định) chọn tầng Xuất/Nhập. */
function onExchangeTierChange(value: GlossaryTierWire, event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement) || !target.checked) return
  setGlossaryManageExchangeTier(value)
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
    @keydown.esc="dispatch('glossary.manage.close')"
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
        <label class="gm-field">
          <span class="gm-field-label">{{ t('glossary.manage.search_label') }}</span>
          <input type="text" class="gm-input" autocomplete="off" :value="manageSearchQuery" @input="onSearchInput" />
        </label>

        <label class="gm-field">
          <span class="gm-field-label">{{ t('glossary.manage.category_filter_label') }}</span>
          <select class="gm-input" :value="manageCategoryFilter" @change="onCategoryFilterChange">
            <option value="all">{{ t('glossary.manage.category_filter_all') }}</option>
            <option v-for="opt in CATEGORY_OPTIONS" :key="opt.value" :value="opt.value">
              {{ t(opt.labelKey) }}
            </option>
          </select>
        </label>

        <label class="gm-field">
          <span class="gm-field-label">{{ t('glossary.manage.origin_filter_label') }}</span>
          <select class="gm-input" :value="manageOriginFilter" @change="onOriginFilterChange">
            <option v-for="opt in ORIGIN_OPTIONS" :key="opt.value" :value="opt.value">{{ t(opt.labelKey) }}</option>
          </select>
        </label>

        <label class="gm-field">
          <span class="gm-field-label">{{ t('glossary.manage.confirmed_filter_label') }}</span>
          <select class="gm-input" :value="manageConfirmedFilter" @change="onConfirmedFilterChange">
            <option v-for="opt in CONFIRMED_OPTIONS" :key="opt.value" :value="opt.value">
              {{ t(opt.labelKey) }}
            </option>
          </select>
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
          🔵 `role="listbox"`/`option` + `aria-selected` (vá 2026-08-24). Trước đó hàng đang
          chọn CHỈ được đánh dấu bằng lớp nền `.gm-row-current`, tức một tín hiệu chỉ tồn tại
          bằng MÀU — cùng lớp thiếu sót mà bản vá P7 của Story 3.8 đã đóng cho dấu ✓/✕. Người
          dùng trình đọc màn hình điều hướng bằng mũi tên (AC7) không có cách nào biết con trỏ
          đang ở đâu. `tabindex="-1"` giữ hàng ngoài vòng Tab: điều hướng hàng là việc của
          mũi tên, `Tab` dành cho các nút thao tác.
        -->
        <ul v-else class="gm-list" role="listbox" :aria-label="t('glossary.manage.list_label')">
          <li
            v-for="(row, i) in manageFilteredRows"
            :key="`${row.tier}:${row.id}`"
            class="gm-row"
            role="option"
            tabindex="-1"
            :aria-selected="i === manageCursor"
            :class="{ 'gm-row-current': i === manageCursor, 'gm-row-shadowed': row.is_shadowed }"
          >
            <!-- aura-allow-text: DỮ LIỆU (`source_term` của chính hàng). -->
            <span class="gm-term">{{ row.source_term }}</span>
            <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
            <span class="gm-badge">{{ row.tier === 'global' ? t('glossary.quick_add.tier_global') : t('glossary.quick_add.tier_work') }}</span>
            <span v-if="row.is_shadowed" class="gm-badge gm-badge-shadowed">{{ t('glossary.manage.shadowed_badge') }}</span>
            <!-- aura-allow-text: DỮ LIỆU (`translation` của chính hàng) hoặc KẾT QUẢ của `t()`. -->
            <span v-if="row.translation !== null" class="gm-translation">{{ row.translation }}</span>
            <span v-else class="gm-badge gm-badge-pending">{{ t('glossary.manage.pending_badge') }}</span>
            <!-- aura-allow-text: DỮ LIỆU (`note` của chính hàng). -->
            <span class="gm-note-cell" :title="row.note">{{ row.note }}</span>
          </li>
        </ul>
      </template>

      <!-- Form Sửa — chỉ hiện khi có một hàng đang chọn VÀ `manageEditing`. -->
      <form
        v-if="manageEditing && manageCurrentRow !== null"
        class="gm-edit-form"
        @submit.prevent="dispatch('glossary.manage.save')"
        @keydown.esc.prevent.stop="dispatch('glossary.manage.cancel')"
      >
        <label class="gm-field">
          <span class="gm-field-label">{{ t('glossary.manage.translation_label') }}</span>
          <input v-model="manageEditTranslation" type="text" class="gm-input" autocomplete="off" :disabled="manageSaving" />
        </label>
        <label class="gm-field">
          <span class="gm-field-label">{{ t('glossary.manage.note_label') }}</span>
          <input v-model="manageEditNote" type="text" class="gm-input" autocomplete="off" :disabled="manageSaving" />
        </label>
        <label class="gm-field">
          <span class="gm-field-label">{{ t('glossary.quick_add.category_label') }}</span>
          <select v-model="manageEditCategory" class="gm-input" :disabled="manageSaving">
            <option v-for="opt in CATEGORY_OPTIONS" :key="opt.value" :value="opt.value">{{ t(opt.labelKey) }}</option>
          </select>
        </label>
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

      <div v-if="!manageEditing" class="gm-actions">
        <button type="button" class="gm-act" :disabled="manageCurrentRow === null" @click="dispatch('glossary.manage.edit')">
          {{ t('glossary.manage.edit') }}
        </button>
        <button type="button" class="gm-act" :disabled="manageCurrentRow === null" @click="dispatch('glossary.manage.delete')">
          {{ t('glossary.manage.delete') }}
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
          <button
            type="button"
            class="gm-act"
            :disabled="manageExportBusy"
            @click="dispatch('glossary.manage.export_csv')"
          >
            {{ t('glossary.manage.export_csv') }}
          </button>
          <button
            type="button"
            class="gm-act"
            data-glossary-import-open
            :disabled="importOpening"
            @click="dispatch('glossary.manage.import_csv')"
          >
            {{ t('glossary.manage.import_csv') }}
          </button>
        </div>

        <p v-if="manageExportError !== null" class="gm-status gm-error" role="alert">
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
  max-width: 880px;
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
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  max-height: 50vh;
  overflow: auto;
  border: 1px solid var(--color-outline);
}

.gm-row {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 3);
  padding: calc(var(--space-unit) * 2) var(--space-panel-inline);
  border-bottom: 1px solid var(--color-outline);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.gm-row:last-child {
  border-bottom: none;
}

.gm-row-current {
  background: var(--color-surface-accent);
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

.gm-note-cell {
  flex: 1;
  min-width: 8rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--face-read);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
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
