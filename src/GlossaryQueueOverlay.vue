<script setup lang="ts">
// Lớp phủ **Duyệt hàng loạt một phím** — Story 3.8, FR53/FR55, lớp phủ THỨ NĂM.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT LỚP PHỦ, KHUÔN `GlossarySettingsOverlay.vue` — scrim, `role="dialog"`, bẫy Tab tự
// viết, focus-return qua `data-glossary-queue-open` (UX-DR17)
// ─────────────────────────────────────────────────────────────────────────────
//
// Nhận/Bỏ/Chuyển/Đóng đi qua `dispatch('<id>')` (AD-34, §Always của spec) — bàn phím cục bộ
// của lớp phủ KHÔNG gọi thẳng handler. Phím `1`–`4` đổi phân loại là NGOẠI LỆ tường minh
// (§Always): cục bộ, KHÔNG qua registry, đúng tiền lệ `GlossaryQuickAdd.vue:55-60`.
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import type { GlossaryCategory } from './config/glossary'
import {
  isInconsistentHarvest,
  queueActionError,
  queueCurrentRow,
  queueCursor,
  queueEmptyReasonFor,
  queueLoadError,
  queueOverlayIsOpen,
  queuePaceSeconds,
  queueRows,
  queueSaving,
  queueSessionCounts,
  queueStatus,
  queueUnprocessedCount,
  setGlossaryQueueCategory,
  setGlossaryQueueCursor,
} from './glossaryQueueState'
import { glossaryScanThresholdInUse } from './glossarySettingsState'

/** Row selection is data (AD-34 §1): keyboard and the hidden per-row radio share this one
 *  get/set instead of two parallel selection mechanisms. */
const cursorModel = computed<number>({
  get: () => queueCursor.value,
  set: (index) => setGlossaryQueueCursor(index),
})

/** Same shape as [`cursorModel`], for the current row's category. */
const currentCategoryModel = computed<GlossaryCategory>({
  get: () => queueCurrentRow.value?.category ?? 'other',
  set: (value) => setGlossaryQueueCategory(value),
})

/** 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Khuôn và lý lẽ chép từ `GlossarySettingsOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
// Lớp phủ chứa chữ thật (thuật ngữ, ví dụ ngữ cảnh) nhưng không phải nguồn từ điển. Đăng ký
// `display` để một vùng chọn trong modal không phát Auto-Lookup rồi thay nội dung phía sau.
useSelectionSurface(panel, 'display')

/**
 * 🔴 THÊM (vòng rà thứ hai, #13) — `queueEmptyReasonFor(...)` gọi LẶP bốn lần trong template
 * bản trước (một lần mỗi nhánh `v-if`/`v-else-if`). Một `computed` DUY NHẤT ở đây tính lại
 * đúng một lần mỗi khi `queueStatus`/`queueUnprocessedCount` đổi, và bốn nhánh template chỉ
 * còn so sánh với giá trị đã có sẵn — không đổi Ý NGHĨA (cùng hàm thuần, cùng hai tham số),
 * chỉ đổi SỐ LẦN GỌI.
 */
const queueEmptyReason = computed(() => queueEmptyReasonFor(queueStatus.value, queueUnprocessedCount.value))

const subtitleText = computed<string>(() => {
  const threshold = glossaryScanThresholdInUse.value
  return threshold === null
    ? t('glossary.queue.subtitle_no_threshold')
    : t('glossary.queue.subtitle', { threshold: String(threshold) })
})

/** Share of the session bar per outcome, against the rows loaded for this opening. */
function barPercent(count: number): string {
  const total = queueSessionCounts.value.total
  return total === 0 ? '0%' : `${(count / total) * 100}%`
}

const paceText = computed<string | null>(() => {
  const pace = queuePaceSeconds.value
  if (pace === null) return null
  const remaining = queueUnprocessedCount.value
  const minutes = Math.max(1, Math.ceil((pace * remaining) / 60))
  return t('glossary.queue.pace', {
    pace: pace.toLocaleString('vi-VN', { maximumFractionDigits: 1 }),
    minutes: String(minutes),
  })
})

function categoryLabel(category: GlossaryCategory): string {
  const opt = CATEGORY_OPTIONS.find((o) => o.value === category)
  return opt === undefined ? '' : t(opt.labelKey)
}

watch(queueCursor, () => {
  void nextTick(() => {
    panel.value?.querySelector<HTMLElement>('.gq-row-current')?.scrollIntoView({ block: 'nearest' })
  })
})

watch(queueOverlayIsOpen, (open) => {
  if (open) {
    // 🔴 KHÔNG lưu `document.activeElement` trần — xem `focusReturnTargetOnOpen`.
    returnFocusTo = focusReturnTargetOnOpen('[data-glossary-queue-open]')
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

  const opener = document.querySelector<HTMLElement>('[data-glossary-queue-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  // Chẩn đoán tiếng Anh — Kiểm A của `check:i18n` cấm chuỗi tiếng Việt ở vị trí mã.
  console.warn('[glossary-queue] focus-return target is gone; focus falls back to body.')
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

/** Bốn phân loại theo phím số (FR46) — khớp `Category::as_str()` phía Rust, cùng khuôn
 * `GlossaryQuickAdd.vue::CATEGORY_OPTIONS` (tái dùng nguyên bốn khoá i18n đã có). */
const CATEGORY_OPTIONS: ReadonlyArray<{ value: GlossaryCategory; labelKey: string; digit: string }> = [
  { value: 'person', labelKey: 'glossary.quick_add.category_person', digit: '1' },
  { value: 'place', labelKey: 'glossary.quick_add.category_place', digit: '2' },
  { value: 'domain_term', labelKey: 'glossary.quick_add.category_domain_term', digit: '3' },
  { value: 'other', labelKey: 'glossary.quick_add.category_other', digit: '4' },
]

/**
 * Bàn phím cục bộ của lớp phủ — KHÔNG có ô nhập chữ nào trong modal này (khác
 * `GlossaryQuickAdd.vue`), nên một handler DUY NHẤT ở gốc modal là an toàn: không có ô gõ
 * nào để phím số/`N`/`B`/mũi tên vô tình cướp mất lượt gõ chữ của người dùng.
 *
 * `Escape`/`Tab` đã có `@keydown.esc`/`@keydown.tab` riêng (cùng khuôn
 * `GlossarySettingsOverlay.vue`) — handler này KHÔNG xử lý lại hai phím đó (rơi vào
 * `default`, không làm gì), tránh gọi `dispatch('glossary.queue.close')` hai lần cho một
 * lần bấm `Esc`.
 *
 * 🔴 **SỬA 2026-08-24 (vòng rà ba lớp) — LỌC `ctrlKey`/`metaKey`/`altKey` TRƯỚC MỌI NHÁNH.**
 * Bản trước so thẳng `event.key` — `⌘N` (macOS) khớp `case 'n'` y hệt một phím `N` trần,
 * nên một hợp âm quen tay **ghi thẳng một ứng viên vào Glossary**, trái AD-20/FR55 ("máy đề
 * xuất, NGƯỜI bấm phải là NGƯỜI, không phải một tổ hợp phím tình cờ trùng"). Cùng lỗi cho
 * `⌘1`/`⌘2`/`⌘3`: chúng khớp nhánh phím số ở trên (đổi phân loại).
 *
 * ⚠️ **ĐÂY LÀ MỘT LỖ RIÊNG, KHÔNG PHẢI CÙNG LỖ VỚI BA LỆNH CHUYỂN CHẾ ĐỘ.**
 * `src/main.ts:527-532` đã chặn `Mod+1`/`Mod+2`/`Mod+3` khỏi `CommandRegistry` suốt lúc lớp
 * phủ này mở — `queueOverlayIsOpen.value` nằm trong `KeymapGate.isBlocked` truyền cho
 * `attachKeyboard`. Cơ chế của `isBlocked` là **thoát sớm, KHÔNG `preventDefault()`**
 * (doc-comment `KeymapGate`, `src/commands/keys.ts:550-560`): listener của keymap gắn ở pha
 * `capture` trên `window` (`src/commands/keys.ts:585-593`), tức chạy **TRƯỚC** handler DOM
 * cục bộ này, và khi `isBlocked` trả `true` nó return ngay — 0 command nào của registry
 * chạy, nhưng sự kiện vẫn đi TIẾP xuống `.gq-scrim` như chưa từng bị chặn. Vì vậy `⌘N`/`⌘1`
 * **không** kích hoạt `mode.library`/`glossary.settings.open`/bất kỳ command nào khác (đã
 * chặn ở `main.ts`), nhưng **vẫn** tới được hàm `onKeydown` này — một listener DOM RIÊNG
 * trên `.gq-scrim`, hoàn toàn SONG SONG với `CommandRegistry`, không đi qua `isBlocked`.
 * Bản trước của hàm này không lọc phím bổ trợ nên `⌘N` vẫn khớp `case 'n'` ở đây và vẫn Nhận
 * — đây mới là lỗ mà bản vá này bịt: đường DOM cục bộ, không phải đường registry.
 *
 * `Shift` CỐ Ý không nằm trong danh sách lọc: nó không đổi `event.key` của digit (bố cục
 * US: `Shift+1` ra `'!'`, không khớp bất kỳ nhánh nào ở đây một cách tự nhiên) và không có
 * xung đột nào với `trapTab` (một `@keydown.tab` RIÊNG, không đi qua hàm này) để mà phải
 * giữ nó y nguyên.
 */
function onKeydown(event: KeyboardEvent): void {
  if (event.ctrlKey || event.metaKey || event.altKey) return

  const digit = CATEGORY_OPTIONS.find((opt) => opt.digit === event.key)
  if (digit !== undefined) {
    event.preventDefault()
    setGlossaryQueueCategory(digit.value)
    return
  }

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      dispatch('glossary.queue.next')
      return
    case 'ArrowUp':
      event.preventDefault()
      dispatch('glossary.queue.prev')
      return
    case 'n':
    case 'N':
      event.preventDefault()
      dispatch('glossary.queue.accept')
      return
    case 'b':
    case 'B':
      event.preventDefault()
      dispatch('glossary.queue.reject')
      return
    case ' ':
      if (event.target instanceof HTMLButtonElement) return
      event.preventDefault()
      dispatch('glossary.queue.leave')
      return
    default:
      return
  }
}
</script>

<template>
  <div
    v-if="queueOverlayIsOpen"
    class="gq-scrim"
    @keydown.esc="dispatch('glossary.queue.close')"
    @keydown.tab="trapTab($event)"
    @keydown="onKeydown"
  >
    <section ref="panel" class="gq-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="gq-head">
        <h2 class="gq-title">{{ t('glossary.queue.title') }}</h2>
        <button type="button" class="gq-close" @click="dispatch('glossary.queue.close')">
          {{ t('command.glossary.queue.close') }}
        </button>
      </header>
      <!-- aura-allow-text: KẾT QUẢ của `t()` (ngưỡng đọc từ cấu hình). -->
      <p class="gq-subtitle">{{ subtitleText }}</p>

      <!--
        🔵 THÊM 2026-08-24 (vòng rà ba lớp), 🔴 DỜI vào `queueEmptyReasonFor` (cụm D vá,
        vòng rà Epic 3) — nhánh "đang tải" (`queueStatus === 'unknown'`, lượt
        `openGlossaryQueue` còn đang bay). Bản trước canh mệnh đề này bằng một `v-if` RIÊNG
        tại đây CỘNG một nhánh trong `queueEmptyReasonFor` — hai chỗ cùng canh MỘT mệnh đề là
        hai nguồn sự thật (§Design Notes của spec). Nay hàm là nguồn DUY NHẤT.
      -->
      <p v-if="queueEmptyReason === 'loading'" class="gq-status" role="status">
        {{ t('glossary.queue.loading') }}
      </p>

      <!-- Ba câu rỗng KHÁC NHAU — §Always của spec: "rỗng phải nói vì sao nó rỗng". -->
      <p v-else-if="queueEmptyReason === 'no_work'" class="gq-empty">
        {{ t('glossary.queue.empty_no_work') }}
      </p>
      <p v-else-if="queueEmptyReason === 'ipc_unavailable'" class="gq-empty">
        {{ t('glossary.queue.empty_ipc_unavailable') }}
      </p>
      <p v-else-if="queueEmptyReason === 'all_reviewed'" class="gq-empty">
        {{ t('glossary.queue.empty_all_reviewed') }}
      </p>
      <p v-else-if="queueStatus === 'error' && queueLoadError !== null" class="gq-empty gq-error" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(queueLoadError) }}
      </p>

      <template v-else-if="queueStatus === 'loaded'">
        <div class="gq-progress" role="group" :aria-label="t('glossary.queue.progress_label')">
          <div class="gq-bar" aria-hidden="true">
            <i class="gq-bar-accepted" :style="{ width: barPercent(queueSessionCounts.accepted) }"></i>
            <i class="gq-bar-rejected" :style="{ width: barPercent(queueSessionCounts.rejected) }"></i>
          </div>
          <div class="gq-legend">
            <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
            <span><span class="gq-sw gq-sw-accepted" aria-hidden="true"></span>{{ t('glossary.queue.legend_accepted', { count: String(queueSessionCounts.accepted) }) }}</span>
            <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
            <span><span class="gq-sw gq-sw-rejected" aria-hidden="true"></span>{{ t('glossary.queue.legend_rejected', { count: String(queueSessionCounts.rejected) }) }}</span>
            <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
            <span v-if="queueSessionCounts.left > 0">{{ t('glossary.queue.legend_left', { count: String(queueSessionCounts.left) }) }}</span>
            <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
            <span><span class="gq-sw gq-sw-remaining" aria-hidden="true"></span>{{ t('glossary.queue.legend_remaining', { count: String(queueSessionCounts.remaining) }) }}</span>
            <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
            <span v-if="paceText !== null" class="gq-pace">{{ paceText }}</span>
          </div>
        </div>

        <ul class="gq-list">
          <li
            v-for="(row, i) in queueRows"
            :key="row.candidate.id"
            class="gq-row"
            :class="{
              'gq-row-current': i === queueCursor,
              'gq-card': i === queueCursor && row.outcome === null,
              'gq-row-done': row.outcome !== null,
              'gq-row-accepted': row.outcome === 'accepted',
              'gq-row-rejected': row.outcome === 'rejected',
              'gq-row-left': row.outcome === null && row.left,
            }"
          >
            <!-- `<label>` wraps the row so a hidden radio can select it; `.gq-row-label` is
                 `display: contents` so the flex layout stays on `.gq-row` (see <style>). -->
            <label class="gq-row-label">
              <input v-model="cursorModel" type="radio" name="gq-row-cursor" class="gq-sr-only" :value="i" />
              <!-- aura-allow-text: DỮ LIỆU (`source_term` của chính hàng). -->
              <span class="gq-term">{{ row.candidate.source_term }}</span>
              <template v-if="i === queueCursor && row.outcome === null">
                <!-- aura-allow-text: KẾT QUẢ của `t()` cộng hai con số đếm. -->
                <span class="gq-chapter-span">{{
                  t('glossary.queue.card_meta', {
                    count: String(row.candidate.occurrence_count),
                    chapters: String(row.candidate.chapter_span_count),
                  })
                }}</span>
                <!-- aura-allow-text: KẾT QUẢ của `t()` cộng hai con số vị trí. -->
                <span class="gq-position">{{
                  t('glossary.queue.card_position', { index: String(i + 1), total: String(queueRows.length) })
                }}</span>
              </template>
              <template v-else>
                <!-- aura-allow-text: KẾT QUẢ của `t()` (nhãn phân loại của phiên). -->
                <span class="gq-cat">{{ categoryLabel(row.category) }}</span>
                <!-- aura-allow-text: KẾT QUẢ của `t()` cộng một con số đếm. -->
                <span class="gq-count">{{ t('glossary.queue.row_count', { count: String(row.candidate.occurrence_count) }) }}</span>
              </template>
              <template v-if="row.candidate.candidate_origin === 'review_harvest'">
                <!-- aura-allow-text: DỮ LIỆU (X → Y của chính hàng) + KẾT QUẢ của `t()`. -->
                <span class="gq-harvest">{{
                  t('glossary.queue.harvest_change', {
                    from: row.candidate.replaced_translation ?? '',
                    to: row.candidate.proposed_translation ?? '',
                    changed: String(row.candidate.changed_count ?? 0),
                    seen: String(row.candidate.seen_count ?? 0),
                  })
                }}</span>
                <span v-if="isInconsistentHarvest(row.candidate)" class="gq-inconsistent">{{
                  t('glossary.queue.harvest_inconsistent')
                }}</span>
              </template>
              <!-- aura-allow-text: DỮ LIỆU (`context_example`), `:title` cho phần bị cắt bởi ellipsis. -->
              <span
                v-if="!(i === queueCursor && row.outcome === null)"
                class="gq-context gq-peek"
                :title="row.candidate.context_example ?? ''"
                >{{ row.candidate.context_example ?? '' }}</span
              >
              <span v-if="row.candidate.han_viet_status === 'ok' && !(i === queueCursor && row.outcome === null)" class="gq-suggestion">
                <!-- aura-allow-text: DỮ LIỆU (`han_viet_suggestion` của chính hàng). -->
                {{ row.candidate.han_viet_suggestion }}
              </span>
              <span v-if="row.outcome === 'accepted'" class="gq-mark" aria-hidden="true">✓</span>
              <span v-else-if="row.outcome === 'rejected'" class="gq-mark" aria-hidden="true">✕</span>
              <!-- aura-allow-text: KẾT QUẢ của `t()`; trạng thái luôn có chữ, vạch lề chỉ là phụ. -->
              <span v-if="row.outcome !== null || row.left" class="gq-sr-only">
                {{
                  row.outcome === 'accepted'
                    ? t('glossary.queue.row_status_accepted')
                    : row.outcome === 'rejected'
                      ? t('glossary.queue.row_status_rejected')
                      : t('glossary.queue.row_status_left')
                }}
              </span>
            </label>

            <div v-if="i === queueCursor && row.outcome === null" class="gq-card-body">
              <div v-if="row.candidate.han_viet_status === 'ok'" class="gq-guess">
                <span class="gq-guess-label">{{ t('glossary.queue.han_viet_label') }}</span>
                <!-- aura-allow-text: DỮ LIỆU (`han_viet_suggestion` của chính hàng). -->
                <span class="gq-suggestion">{{ row.candidate.han_viet_suggestion }}</span>
              </div>

              <div class="gq-category">
                <span class="gq-category-label">{{ t('glossary.queue.category_label') }}</span>
                <!-- A real radio per chip, same precedent as `GlossaryQuickAdd.vue`. -->
                <label
                  v-for="opt in CATEGORY_OPTIONS"
                  :key="opt.value"
                  class="gq-chip"
                  :class="{ on: row.category === opt.value }"
                >
                  <input v-model="currentCategoryModel" type="radio" name="gq-category" :value="opt.value" />
                  {{ t(opt.labelKey) }}
                  <!-- aura-allow-text: chữ số phím tắt. -->
                  <sup class="gq-chip-digit">{{ opt.digit }}</sup>
                </label>
              </div>
              <p class="gq-hint">{{ t('glossary.queue.category_not_persisted_hint') }}</p>

              <div v-if="row.candidate.context_example !== null" class="gq-ctx">
                <div class="gq-ctx-label">{{ t('glossary.queue.context_label') }}</div>
                <!-- aura-allow-text: DỮ LIỆU (`context_example` của chính hàng). -->
                <div class="gq-ctx-quote">{{ row.candidate.context_example }}</div>
              </div>

              <div
                v-if="
                  row.candidate.han_viet_status !== 'ok' &&
                  (row.candidate.candidate_origin !== 'review_harvest' || row.candidate.proposed_translation === null)
                "
                class="gq-notrans"
              >
                <b>{{ t('glossary.queue.no_translation_title') }}</b>
                {{ t('glossary.queue.no_translation_body') }}
              </div>
            </div>
          </li>
        </ul>

        <p v-if="queueActionError !== null" class="gq-status gq-error" role="alert">
          <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
          {{ tError(queueActionError) }}
        </p>
        <p v-else-if="queueSaving" class="gq-status" role="status">{{ t('glossary.queue.saving') }}</p>

        <div class="gq-actions">
          <button type="button" class="gq-act gq-act-primary" @click="dispatch('glossary.queue.accept')">
            <!-- aura-allow-text: tên phím. --><kbd>N</kbd>{{ t('glossary.queue.accept') }}
          </button>
          <button type="button" class="gq-act" @click="dispatch('glossary.queue.reject')">
            <!-- aura-allow-text: tên phím. --><kbd>B</kbd>{{ t('glossary.queue.reject') }}
          </button>
          <button type="button" class="gq-act" @click="dispatch('glossary.queue.leave')">
            <!-- aura-allow-text: tên phím. --><kbd>Space</kbd>{{ t('glossary.queue.key_leave') }}
          </button>
          <button type="button" class="gq-act" @click="dispatch('glossary.queue.prev')">
            <!-- aura-allow-text: tên phím. --><kbd>↑</kbd>{{ t('glossary.queue.prev') }}
          </button>
          <button type="button" class="gq-act" @click="dispatch('glossary.queue.next')">
            <!-- aura-allow-text: tên phím. --><kbd>↓</kbd>{{ t('glossary.queue.next') }}
          </button>
        </div>
        <p class="gq-hint gq-stop">{{ t('glossary.queue.stop_note') }}</p>
        <p class="gq-hint gq-footer">
          {{ t('glossary.queue.footer_rule') }} {{ t('glossary.queue.footer_origin') }}
        </p>
      </template>
    </section>
  </div>
</template>

<style scoped>
.gq-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: xếp lớp CƠ HỌC — cùng lý do các lớp phủ khác. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.gq-panel {
  width: 100%;
  max-width: 720px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.gq-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.gq-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.gq-close {
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

.gq-empty {
  margin: 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.gq-list {
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  max-height: 60vh;
  overflow: auto;
  border: 1px solid var(--color-outline);
}

.gq-row {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 3);
  padding: calc(var(--space-unit) * 2) var(--space-panel-inline);
  border-bottom: 1px solid var(--color-outline);
  border-left: 2px solid transparent;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  cursor: pointer;
}

.gq-row:last-child {
  border-bottom: none;
}

/* Keeps `.gq-row`'s flex layout even though its content is now wrapped in a `<label>`. */
.gq-row-label {
  display: contents;
}

.gq-row-current {
  background: var(--color-surface-accent);
  border-left: 3px solid var(--color-primary);
}

.gq-row-accepted {
  border-left: 2px solid var(--color-confirmed);
}

.gq-row-rejected {
  border-left: 2px solid var(--color-ornament);
}

.gq-row-left {
  border-left: 2px dashed var(--color-draft);
}

.gq-card {
  display: block;
  padding-block: calc(var(--space-unit) * 4);
}

.gq-card .gq-row-label {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 3);
}

.gq-card .gq-term {
  font-family: var(--family-read);
  font-size: var(--font-read-title);
  line-height: var(--leading-read-title);
}

.gq-position {
  margin-left: auto;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gq-card-body {
  margin-top: calc(var(--space-unit) * 3);
  cursor: default;
}

.gq-guess {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
  margin-bottom: calc(var(--space-unit) * 2);
}

.gq-guess-label,
.gq-ctx-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gq-ctx {
  margin-bottom: calc(var(--space-unit) * 3);
  padding-left: calc(var(--space-unit) * 3);
  border-left: 2px solid var(--color-outline-faint);
}

.gq-ctx-quote {
  font-family: var(--family-read);
  font-size: var(--font-read-sm);
  line-height: var(--leading-read-sm);
  color: var(--color-on-surface);
}

.gq-notrans {
  margin-bottom: calc(var(--space-unit) * 3);
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border-left: 2px solid var(--color-tm-rule);
  background: var(--color-surface-tm);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-tm-text);
}

.gq-subtitle {
  margin: 0 0 calc(var(--space-unit) * 3) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.gq-progress {
  margin-bottom: var(--space-panel-block);
}

.gq-bar {
  display: flex;
  height: calc(var(--space-unit) * 1);
  background: var(--color-surface-sunken);
  overflow: hidden;
}

.gq-bar i {
  display: block;
  height: 100%;
}

.gq-bar-accepted,
.gq-sw-accepted {
  background: var(--color-confirmed);
}

.gq-bar-rejected,
.gq-sw-rejected {
  background: var(--color-ornament);
}

.gq-sw-remaining {
  background: var(--color-surface-sunken);
  border: 1px solid var(--color-outline);
}

.gq-legend {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 4);
  margin-top: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gq-sw {
  display: inline-block;
  width: calc(var(--space-unit) * 2);
  height: calc(var(--space-unit) * 2);
  margin-right: calc(var(--space-unit) * 1);
}

.gq-pace {
  margin-left: auto;
}

.gq-cat {
  min-width: 6rem;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gq-chip-digit {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
}

.gq-act kbd {
  margin-right: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface-variant);
}

.gq-stop,
.gq-footer {
  margin: calc(var(--space-unit) * 3) 0 0 0;
}

/*
 * AC — hàng đã xử lý lùi bằng MÀU CHỮ + dấu ✓/✕. Cấm `opacity`, không miễn trừ, kể cả có
 * tên (§Always của spec — chính mockup của bảng chờ này là chỗ luật đó ra đời).
 */
.gq-row-done {
  color: var(--color-on-surface-variant);
}

.gq-term {
  font-weight: var(--weight-ui-md-strong);
}

.gq-row-done .gq-term {
  font-weight: var(--weight-ui-md);
}

.gq-count {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
}

.gq-harvest,
.gq-inconsistent {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
}

.gq-inconsistent {
  color: var(--color-on-surface-variant);
}

.gq-chapter-span {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gq-context {
  flex: 1;
  min-width: 8rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--family-read);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
}

.gq-suggestion {
  color: var(--color-primary);
}

.gq-row-done .gq-suggestion {
  color: var(--color-on-surface-variant);
}

.gq-mark {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
}

/**
 * 🔵 THÊM 2026-08-24 (vòng rà ba lớp) — tín hiệu đọc được cho hàng đã xử lý, cùng khuôn
 * `App.vue::.sr-announcer`. "Visually hidden" bằng `clip`/`position: absolute`, KHÔNG
 * `display: none` (loại khỏi accessibility tree). Các giá trị `1px`/`-1px` là kích thước
 * HÌNH HỌC của kỹ thuật ẩn, không phải màu hay cỡ chữ — `check:tokens` Kiểm B/D không cưỡng
 * chế loại giá trị này.
 */
.gq-sr-only {
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

.gq-category {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  margin-bottom: var(--space-panel-block);
}

.gq-category-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gq-hint {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gq-chip {
  display: inline-flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  cursor: pointer;
}

.gq-chip.on {
  color: var(--color-on-surface);
  border-color: var(--color-primary);
}

.gq-status {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gq-error {
  color: var(--color-error);
}

.gq-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-panel-inline);
}

.gq-act {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: none;
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  cursor: pointer;
}

.gq-act-primary {
  color: var(--color-on-surface);
  border-color: var(--color-primary);
}
</style>
