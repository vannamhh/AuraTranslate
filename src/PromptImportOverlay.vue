<script setup lang="ts">
// Lớp phủ **Xem trước lượt nhập bộ prompt** — Story 4.5 (FR79/NFR9, AD-48).
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT LỚP PHỦ, KHUÔN `GlossaryImportOverlay.vue` — scrim + `role="dialog"` + bẫy Tab tự
// viết + focus-return qua `data-prompt-import-open` (đặt trên nút "Nhập từ file" của
// `PromptLibraryOverlay.vue` — lớp phủ này mở TỪ một lớp phủ khác, không từ titlebar).
//
// 🔴 AD-48 §Rule ①: KHÔNG một byte nội dung tệp thô nào — `name`/`body` trong template đọc
// từ `PromptSetImportPreview` (một MÔ HÌNH ĐÃ KIỂM Rust trả về).
//
// 🔴 Chọn TẦNG và quyết định va chạm đi bằng `<input type="radio">` + `@change`, KHÔNG
// `dispatch` — cùng lý do `GlossaryImportOverlay.vue`: `dispatch` không nhận tham số.
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
import { nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import type { PromptSetConflictDecision, PromptSetTier } from './config/promptset'
import {
  promptImportConfirmError,
  promptImportConfirming,
  promptImportDecision,
  promptImportLoadError,
  promptImportOverlayIsOpen,
  promptImportPreview,
  promptImportSelectedTier,
  promptImportSelectedTierPreview,
  promptImportStatus,
  setPromptImportDecision,
  setPromptImportTier,
} from './promptSetImportState'

/** 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Khuôn chép từ `GlossaryImportOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

watch(promptImportOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-prompt-import-open]')
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-prompt-import-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[prompt-import] focus-return target is gone; focus falls back to body.')
})

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

function onTierChange(tier: PromptSetTier, event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement) || !target.checked) return
  setPromptImportTier(tier)
}

function onDecisionChange(decision: PromptSetConflictDecision, event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement) || !target.checked) return
  setPromptImportDecision(decision)
}
</script>

<template>
  <div
    v-if="promptImportOverlayIsOpen"
    class="pi-scrim"
    @keydown.esc="dispatch('prompt.import.cancel')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="pi-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="pi-head">
        <h2 class="pi-title">{{ t('prompt.import.title') }}</h2>
        <button type="button" class="pi-close" @click="dispatch('prompt.import.cancel')">
          {{ t('command.prompt.import.cancel') }}
        </button>
      </header>

      <!-- Story 4.5 review fix — `status === 'unknown'` không bao giờ vẽ được: `overlayOpen`
           chỉ bật SAU khi `await openPromptImportPreview()` xong, CÙNG khối đồng bộ đổi
           `status` khỏi 'unknown' (`promptSetImportState.ts::openPromptImportPreviewOverlay`)
           — nên lớp phủ không bao giờ MỞ trong lúc `status` còn 'unknown'. Nhánh đã bị xoá
           thay vì để một chuỗi sản phẩm không bao giờ hiện được (`prompt.import.loading`
           cũng đã bỏ khỏi `vi.json`). -->
      <p v-if="promptImportStatus === 'ipc_unavailable'" class="pi-empty">
        {{ t('prompt.import.empty_ipc_unavailable') }}
      </p>
      <p v-else-if="promptImportStatus === 'error' && promptImportLoadError !== null" class="pi-empty pi-error" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(promptImportLoadError) }}
      </p>

      <template v-else-if="promptImportStatus === 'loaded' && promptImportPreview !== null">
        <dl class="pi-summary">
          <div class="pi-summary-row">
            <dt>{{ t('prompt.import.file_name_label') }}</dt>
            <!-- aura-allow-text: DỮ LIỆU (tên tệp do người dùng chọn). -->
            <dd>{{ promptImportPreview.file_name }}</dd>
          </div>
          <div class="pi-summary-row">
            <dt>{{ t('prompt.import.name_label') }}</dt>
            <!-- aura-allow-text: DỮ LIỆU (tên bộ đọc từ tệp). -->
            <dd>{{ promptImportPreview.name }}</dd>
          </div>
        </dl>

        <h4 class="pi-sh">{{ t('prompt.import.body_label') }}</h4>
        <!-- aura-allow-text: DỮ LIỆU (thân prompt đọc từ tệp, MÔ HÌNH ĐÃ KIỂM, AD-48 §Rule ①). -->
        <pre class="pi-body">{{ promptImportPreview.body }}</pre>

        <p v-if="promptImportPreview.warnings.unknown_markers.length > 0" class="pi-warn">
          {{ t('prompt.import.warning_unknown_markers', { tokens: promptImportPreview.warnings.unknown_markers.join(', ') }) }}
        </p>
        <p v-if="promptImportPreview.warnings.glossary_terms_missing" class="pi-warn">
          {{ t('prompt.import.warning_glossary_terms_missing') }}
        </p>

        <fieldset class="pi-tier" role="radiogroup" :aria-label="t('prompt.import.tier_label')">
          <legend class="pi-field-label">{{ t('prompt.import.tier_label') }}</legend>
          <label class="pi-radio-label">
            <input
              type="radio"
              name="pi-tier"
              :checked="promptImportSelectedTier === 'global'"
              @change="onTierChange('global', $event)"
            />
            {{ t('prompt.import.tier_global') }}
          </label>
          <label class="pi-radio-label">
            <input
              type="radio"
              name="pi-tier"
              :disabled="promptImportPreview.work === null"
              :checked="promptImportSelectedTier === 'work'"
              @change="onTierChange('work', $event)"
            />
            {{ t('prompt.import.tier_work') }}
          </label>
        </fieldset>

        <p v-if="promptImportSelectedTierPreview?.kind === 'new'" class="pi-note">
          {{ t('prompt.import.status_new') }}
        </p>
        <p v-else-if="promptImportSelectedTierPreview?.kind === 'identical'" class="pi-note">
          {{ t('prompt.import.status_identical') }}
        </p>

        <template v-else-if="promptImportSelectedTierPreview?.kind === 'conflict'">
          <p class="pi-note">{{ t('prompt.import.status_conflict') }}</p>
          <dl class="pi-summary">
            <div class="pi-summary-row">
              <dt>{{ t('prompt.import.conflict_existing_label') }}</dt>
              <!-- aura-allow-text: DỮ LIỆU (thân đang có trong kho). -->
              <dd class="pi-mono">{{ promptImportSelectedTierPreview.existing_body ?? '' }}</dd>
            </div>
            <div class="pi-summary-row">
              <dt>{{ t('prompt.import.conflict_file_label') }}</dt>
              <!-- aura-allow-text: DỮ LIỆU (thân tệp mang). -->
              <dd class="pi-mono">{{ promptImportPreview.body }}</dd>
            </div>
          </dl>
          <div class="pi-conflict-choice" role="radiogroup" :aria-label="t('prompt.import.status_conflict')">
            <label class="pi-radio-label">
              <input
                type="radio"
                name="pi-decision"
                :checked="promptImportDecision === 'keep_mine'"
                @change="onDecisionChange('keep_mine', $event)"
              />
              {{ t('prompt.import.keep_mine') }}
            </label>
            <label class="pi-radio-label">
              <input
                type="radio"
                name="pi-decision"
                :checked="promptImportDecision === 'take_theirs'"
                @change="onDecisionChange('take_theirs', $event)"
              />
              {{ t('prompt.import.take_theirs') }}
            </label>
          </div>
        </template>

        <p v-if="promptImportConfirmError !== null" class="pi-status pi-error" role="alert">
          <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
          {{ tError(promptImportConfirmError) }}
        </p>
        <p v-else-if="promptImportConfirming" class="pi-status" role="status">{{ t('prompt.import.confirming') }}</p>

        <p class="pi-hint">{{ t('prompt.import.hint_no_write_before_confirm') }}</p>

        <div class="pi-actions">
          <button
            type="button"
            class="pi-act pi-act-primary"
            :disabled="promptImportConfirming"
            @click="dispatch('prompt.import.confirm')"
          >
            {{ t('prompt.import.confirm_button') }}
          </button>
          <button type="button" class="pi-act" :disabled="promptImportConfirming" @click="dispatch('prompt.import.cancel')">
            {{ t('command.prompt.import.cancel') }}
          </button>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.pi-scrim {
  position: fixed;
  inset: 0;
  z-index: 11; /* aura-allow-z-index: xếp TRÊN PromptLibraryOverlay (10) — mở từ trong nó. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.pi-panel {
  width: 100%;
  max-width: 720px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.pi-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.pi-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.pi-close {
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

.pi-status {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pi-empty {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.pi-error {
  color: var(--color-error);
}

.pi-summary {
  margin: 0 0 var(--space-panel-block) 0;
}

.pi-summary-row {
  display: flex;
  gap: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  margin-bottom: calc(var(--space-unit) * 1);
}

.pi-summary-row dt {
  color: var(--color-on-surface-variant);
  min-width: 8rem;
}

.pi-summary-row dd {
  margin: 0;
  color: var(--color-on-surface);
  flex: 1;
}

.pi-mono {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  white-space: pre-wrap;
}

.pi-sh {
  font-size: var(--font-ui-label);
  font-family: var(--face-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
  margin: 0 0 calc(var(--space-unit) * 2) 0;
}

.pi-body {
  margin: 0 0 var(--space-panel-block) 0;
  padding: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
  white-space: pre-wrap;
  max-height: 12rem;
  overflow: auto;
}

.pi-warn {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.pi-note {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pi-tier {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 3);
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  padding: 0;
  border: none;
}

.pi-field-label {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
}

.pi-radio-label {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
  cursor: pointer;
}

.pi-conflict-choice {
  display: flex;
  gap: calc(var(--space-unit) * 3);
  margin: 0 0 var(--space-panel-block) 0;
}

.pi-hint {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pi-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}

.pi-act {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.pi-act-primary {
  background: var(--color-primary);
  color: var(--color-on-primary);
  border-color: var(--color-primary);
}

.pi-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}
</style>
