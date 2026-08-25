<script setup lang="ts">
// Lớp phủ **Xem trước lượt nhập Glossary** — Story 3.10b, AD-48, lớp phủ THỨ BẢY.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT LỚP PHỦ, KHUÔN `GlossaryManageOverlay.vue` — scrim + `role="dialog"` + bẫy Tab tự
// viết + focus-return qua `data-glossary-import-open` (đặt trên nút "Nhập CSV" của chính
// `GlossaryManageOverlay.vue`, KHÔNG trên một nút titlebar — lớp phủ này mở TỪ một lớp phủ
// khác, không từ titlebar).
//
// 🔴 AD-48 §Rule ①: KHÔNG một byte nội dung tệp thô nào render ở đây — mọi thứ trong
// template đọc từ `GlossaryImportPreview` (một MÔ HÌNH ĐÃ KIỂM Rust trả về), không từ một
// chuỗi văn bản tệp nào.
//
// 🔴 Quyết định từng hàng bất đồng đi bằng `<input type="radio">` + `@change`, KHÔNG
// `dispatch` — `dispatch` không nhận tham số nên không diễn đạt được "hàng thứ N chọn lấy
// của file" (§Design Notes của spec). `check:commands` Kiểm A chỉ canh `@click`, nên
// `@change` nằm NGOÀI phạm vi của nó — không phải một lỗ hổng, một quyết định có tiền lệ
// (các `<select>` của `GlossaryManageOverlay.vue` cũng dùng `@change`).
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
import { nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import type { GlossaryConflictDecision } from './config/glossary'
import {
  importConfirmError,
  importConfirmSummary,
  importConfirming,
  importDecisionFor,
  importEmptyReasonFor,
  importLoadError,
  importOverlayIsOpen,
  importPreview,
  importStatus,
  setImportDecision,
} from './glossaryImportState'

/** 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Khuôn và lý lẽ chép từ `GlossaryManageOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
// Lớp phủ chứa chữ thật (tên tệp, thuật ngữ, bản dịch) nhưng không phải nguồn từ điển.
useSelectionSurface(panel, 'display')

watch(importOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-glossary-import-open]')
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-glossary-import-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[glossary-import] focus-return target is gone; focus falls back to body.')
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

function onDecisionChange(sourceTerm: string, decision: GlossaryConflictDecision, event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement) || !target.checked) return
  setImportDecision(sourceTerm, decision)
}
</script>

<template>
  <div
    v-if="importOverlayIsOpen"
    class="gi-scrim"
    @keydown.esc="dispatch('glossary.import.cancel')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="gi-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="gi-head">
        <h2 class="gi-title">{{ t('glossary.import.title') }}</h2>
        <button type="button" class="gi-close" @click="dispatch('glossary.import.cancel')">
          {{ t('command.glossary.import.cancel') }}
        </button>
      </header>

      <p v-if="importStatus === 'unknown'" class="gi-status" role="status">{{ t('glossary.import.loading') }}</p>
      <p v-else-if="importStatus === 'ipc_unavailable'" class="gi-empty">
        {{ t('glossary.import.empty_ipc_unavailable') }}
      </p>
      <p v-else-if="importStatus === 'error' && importLoadError !== null" class="gi-empty gi-error" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(importLoadError) }}
      </p>

      <template v-else-if="importStatus === 'loaded' && importPreview !== null">
        <dl class="gi-summary">
          <div class="gi-summary-row">
            <dt>{{ t('glossary.import.file_name_label') }}</dt>
            <!-- aura-allow-text: DỮ LIỆU (tên tệp do người dùng chọn). -->
            <dd>{{ importPreview.file_name }}</dd>
          </div>
          <div class="gi-summary-row">
            <dt>{{ t('glossary.import.rows_label') }}</dt>
            <!-- aura-allow-text: KẾT QUẢ của `t()` (đã nội suy số liệu qua tham số). -->
            <dd>
              {{
                t('glossary.import.rows_summary', {
                  row_count: String(importPreview.row_count),
                  recognized_column_count: String(importPreview.recognized_column_count),
                })
              }}
            </dd>
          </div>
          <div v-if="importPreview.ignored_columns.length > 0" class="gi-summary-row">
            <dt>{{ t('glossary.import.ignored_columns_label') }}</dt>
            <!-- aura-allow-text: DỮ LIỆU (tên cột lạ đọc từ hàng tiêu đề của tệp). -->
            <dd>{{ importPreview.ignored_columns.join(', ') }}</dd>
          </div>
        </dl>

        <p v-if="importPreview.term_origin_column_present" class="gi-note">
          {{ t('glossary.import.term_origin_note') }}
        </p>

        <p class="gi-counts">
          <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
          {{ t('glossary.import.new_count', { count: String(importPreview.new_count) }) }}
          ·
          {{ t('glossary.import.identical_count', { count: String(importPreview.identical_count) }) }}
        </p>

        <h3 class="gi-conflicts-title">{{ t('glossary.import.conflicts_title') }}</h3>

        <p
          v-if="importEmptyReasonFor(importStatus, importPreview.row_count, importPreview.conflicts.length) === 'file_has_no_rows'"
          class="gi-empty"
        >
          {{ t('glossary.import.file_has_no_rows') }}
        </p>
        <p
          v-else-if="importEmptyReasonFor(importStatus, importPreview.row_count, importPreview.conflicts.length) === 'no_conflicts'"
          class="gi-empty"
        >
          {{ t('glossary.import.no_conflicts') }}
        </p>

        <ul v-else class="gi-conflict-list">
          <li v-for="row in importPreview.conflicts" :key="row.source_term" class="gi-conflict-row">
            <!-- aura-allow-text: DỮ LIỆU (`source_term` của chính hàng). -->
            <span class="gi-conflict-term">{{ row.source_term }}</span>
            <span class="gi-conflict-value">
              <!-- aura-allow-text: KẾT QUẢ của `t()` + DỮ LIỆU (bản dịch đang có). -->
              {{ t('glossary.import.conflict_existing', { value: row.existing_translation ?? '' }) }}
            </span>
            <span class="gi-conflict-value">
              <!-- aura-allow-text: KẾT QUẢ của `t()` + DỮ LIỆU (bản dịch tệp mang). -->
              {{ t('glossary.import.conflict_file', { value: row.file_translation ?? '' }) }}
            </span>
            <div class="gi-conflict-choice" role="radiogroup" :aria-label="row.source_term">
              <label class="gi-radio-label">
                <input
                  type="radio"
                  :name="`gi-decision-${row.source_term}`"
                  :checked="importDecisionFor(row.source_term) === 'keep_mine'"
                  @change="onDecisionChange(row.source_term, 'keep_mine', $event)"
                />
                {{ t('glossary.import.keep_mine') }}
              </label>
              <label class="gi-radio-label">
                <input
                  type="radio"
                  :name="`gi-decision-${row.source_term}`"
                  :checked="importDecisionFor(row.source_term) === 'take_theirs'"
                  @change="onDecisionChange(row.source_term, 'take_theirs', $event)"
                />
                {{ t('glossary.import.take_theirs') }}
              </label>
            </div>
          </li>
        </ul>

        <p v-if="importConfirmError !== null" class="gi-status gi-error" role="alert">
          <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
          {{ tError(importConfirmError) }}
        </p>
        <p v-else-if="importConfirming" class="gi-status" role="status">{{ t('glossary.import.confirming') }}</p>

        <p class="gi-hint">{{ t('glossary.import.hint_no_write_before_confirm') }}</p>

        <div class="gi-actions">
          <button
            type="button"
            class="gi-act gi-act-primary"
            :disabled="importConfirming"
            @click="dispatch('glossary.import.confirm')"
          >
            <!-- aura-allow-text: KẾT QUẢ của `t()` (đã nội suy số liệu qua tham số). -->
            {{
              t('glossary.import.confirm_summary', {
                new_count: String(importConfirmSummary.newCount),
                keep_count: String(importConfirmSummary.keepCount),
              })
            }}
          </button>
          <button type="button" class="gi-act" :disabled="importConfirming" @click="dispatch('glossary.import.cancel')">
            {{ t('command.glossary.import.cancel') }}
          </button>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.gi-scrim {
  position: fixed;
  inset: 0;
  z-index: 11; /* aura-allow-z-index: xếp TRÊN GlossaryManageOverlay (10) — mở từ trong nó. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.gi-panel {
  width: 100%;
  max-width: 720px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.gi-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.gi-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.gi-close {
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

.gi-status {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gi-empty {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.gi-error {
  color: var(--color-error);
}

.gi-summary {
  margin: 0 0 var(--space-panel-block) 0;
}

.gi-summary-row {
  display: flex;
  gap: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
}

.gi-summary-row dt {
  color: var(--color-on-surface-variant);
  min-width: 8rem;
}

.gi-summary-row dd {
  margin: 0;
  color: var(--color-on-surface);
}

.gi-note {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gi-counts {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.gi-conflicts-title {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong, var(--weight-ui-md));
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.gi-conflict-list {
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
}

.gi-conflict-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 3);
  padding: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
}

.gi-conflict-term {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong, var(--weight-ui-md));
  color: var(--color-on-surface);
}

.gi-conflict-value {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gi-conflict-choice {
  display: flex;
  gap: calc(var(--space-unit) * 3);
  margin-left: auto;
}

.gi-radio-label {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
  cursor: pointer;
}

.gi-hint {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gi-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}

.gi-act {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.gi-act-primary {
  background: var(--color-primary);
  color: var(--color-on-primary);
  border-color: var(--color-primary);
}

.gi-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}
</style>
