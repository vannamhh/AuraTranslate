<script setup lang="ts">
// Lớp phủ **Xem trước lượt nhập — bảng mã** (Story 6.3, FR126, AD-39 bước 1).
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 KHUÔN `GlossaryImportOverlay.vue` — scrim + `role="dialog"` + bẫy Tab tự viết +
// focus-return qua `data-import-preview-open` (đặt trên hai nút nộp form của
// `LibraryMode.vue`: dán văn bản VÀ tệp — cả hai mở lớp phủ NÀY).
//
// 🔴 Ba tầng theo thứ tự NHÂN QUẢ, KHÔNG nút "Tiếp theo" (§Always spec 6.3): bảng mã →
// ranh giới nội dung → luật làm sạch, tất cả trên MỘT màn hình. Chỉ tầng 1 có thân; tầng 2
// (Story 6.9) và tầng 3 (Story 6.5) LUÔN hiện, LUÔN rỗng, LUÔN nói ra vì sao.
//
// 🔴 Dải năm ứng viên đi qua `<input type="radio">` + `@change`, KHÔNG `dispatch` — cùng lý
// do đúng hệt `GlossaryImportOverlay.vue`: `dispatch` không nhận tham số, và "hàng thứ N
// được chọn" không diễn đạt được qua nó. `check:commands` Kiểm A chỉ canh `@click`, nên
// `@change` nằm ngoài phạm vi của nó — có tiền lệ, không phải một lỗ hổng.
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
// Không `v-html` ở bất kỳ đâu (AD-16) — mọi bản dựng thật của dải là DỮ LIỆU văn bản thô từ
// Rust, không phải markup.
import { nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import {
  importPreview,
  importPreviewConfirmError,
  importPreviewConfirming,
  importPreviewEmptyReasonForTier,
  importPreviewIsOpen,
  importPreviewLoadError,
  importPreviewSelectedCandidate,
  importPreviewSelectedEncoding,
  importPreviewStatus,
  importPreviewStripIsOpen,
  selectImportPreviewCandidate,
} from './importPreviewState'
import type { ImportConfidence } from './config/project'

/**
 * 🔴 SỬA (vòng rà đối kháng 2, mục 22) — hai hàm này thay cho việc GHÉP CHUỖI khoá i18n
 * bằng nội suy (`` `mode.library.preview.confidence_${…}` ``): một khoá dựng bằng nội suy vô
 * hình với BẤT KỲ cổng nào quét mã tìm literal `t('…')`/`` t(`…`) `` cố định (kể cả
 * `check:i18n`) — xoá nhầm một khoá `confidence_high` ở `vi.json` không lộ ra ở đây, chỉ lộ
 * lúc CHẠY THẬT. `switch` cạn (không nhánh `default`) buộc trình biên dịch TypeScript báo lỗi
 * nếu [`ImportConfidence`]/kiểu trả của [`importPreviewEmptyReasonForTier`] thêm một biến thể
 * mới mà quên cập nhật ở đây — cùng an toàn của một BẢNG dữ liệu, không phải một chuỗi ghép.
 */
function confidenceMessageKey(confidence: ImportConfidence): string {
  switch (confidence) {
    case 'self_declared':
      return 'mode.library.preview.confidence_self_declared'
    case 'high':
      return 'mode.library.preview.confidence_high'
    case 'low':
      return 'mode.library.preview.confidence_low'
  }
}

function tierEmptyMessageKey(reason: 'story_6_9' | 'story_6_5'): string {
  switch (reason) {
    case 'story_6_9':
      return 'mode.library.preview.tier_empty_story_6_9'
    case 'story_6_5':
      return 'mode.library.preview.tier_empty_story_6_5'
  }
}

/** 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Khuôn và lý lẽ chép từ `GlossaryImportOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
// Lớp phủ chứa bản dựng thật của nội dung đang nhập (không phải nguồn từ điển).
useSelectionSurface(panel, 'display')

watch(importPreviewIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-import-preview-open]')
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-import-preview-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[import-preview] focus-return target is gone; focus falls back to body.')
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

function onCandidateChange(encoding: string, event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement) || !target.checked) return
  selectImportPreviewCandidate(encoding)
}

/**
 * 🔴 Vòng rà đối kháng 2, mục 4 — CHẶN THỊ GIÁC, lớp phòng thủ THỨ HAI. Lớp CHÍNH sống ở
 * `importPreviewState.ts::cancelImportPreview` (no-op khi `importPreviewConfirming`) — hàm
 * đó đóng cửa sổ đua triệt để, bất kể tầng `.vue` có gọi tới hay không. Chặn ở đây chỉ để
 * `Esc` không phát một `dispatch` vô ích trong lúc phím tắt khác (Tab) vẫn cần chạy —
 * `@keydown` nằm ngoài Kiểm A của `check:commands` (chỉ canh `@click`), nên một hàm thay vì
 * `dispatch(...)` trần ở đây không phải một lỗ hổng.
 */
function onEscapeCancel(): void {
  if (importPreviewConfirming.value) return
  dispatch('import.preview.cancel')
}
</script>

<template>
  <div
    v-if="importPreviewIsOpen"
    class="ip-scrim"
    @keydown.esc="onEscapeCancel"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="ip-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="ip-head">
        <h2 class="ip-title">{{ t('mode.library.preview.title') }}</h2>
        <button
          type="button"
          class="ip-close"
          :disabled="importPreviewConfirming"
          @click="dispatch('import.preview.cancel')"
        >
          {{ t('command.import.preview.cancel') }}
        </button>
      </header>

      <p v-if="importPreviewStatus === 'unknown'" class="ip-status" role="status">
        {{ t('mode.library.preview.loading') }}
      </p>
      <p v-else-if="importPreviewStatus === 'ipc_unavailable'" class="ip-empty">
        {{ t('mode.library.preview.empty_ipc_unavailable') }}
      </p>
      <p v-else-if="importPreviewStatus === 'error' && importPreviewLoadError !== null" class="ip-empty ip-error" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(importPreviewLoadError) }}
      </p>

      <template v-else-if="importPreviewStatus === 'loaded' && importPreview !== null">
        <!-- ═══════════════════ Tầng 1 — bảng mã (CÓ THÂN) ═══════════════════ -->
        <section class="ip-tier ip-tier-1" aria-labelledby="ip-tier-1-title">
          <h3 id="ip-tier-1-title" class="ip-tier-title">{{ t('mode.library.preview.tier1_title') }}</h3>

          <p class="ip-confidence-chip" :data-confidence="importPreview.confidence">
            <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
            {{ t(confidenceMessageKey(importPreview.confidence)) }}
            <span v-if="importPreviewSelectedCandidate !== null" class="ip-confidence-label">
              <!-- aura-allow-text: DỮ LIỆU (nhãn FR126 của ứng viên đang chọn). -->
              — {{ importPreviewSelectedCandidate.label }}
            </span>
          </p>

          <p
            v-if="!importPreviewStripIsOpen && importPreview.candidates.length > 0"
            class="ip-strip-hint"
          >
            {{ t('mode.library.preview.strip_open_hint') }}
            <button type="button" class="ip-strip-open" @click="dispatch('import.preview.open_picker')">
              {{ t('command.import.preview.open_picker') }}
            </button>
          </p>

          <template v-if="importPreviewStripIsOpen">
            <p class="ip-strip-hint">{{ t('mode.library.preview.strip_hint') }}</p>
            <div class="ip-strip" role="radiogroup" :aria-label="t('mode.library.preview.strip_hint')">
              <label
                v-for="candidate in importPreview.candidates"
                :key="candidate.encoding"
                class="ip-candidate"
                :class="{ 'ip-candidate-selected': importPreviewSelectedEncoding === candidate.encoding }"
              >
                <input
                  type="radio"
                  name="ip-candidate"
                  class="ip-candidate-radio"
                  :checked="importPreviewSelectedEncoding === candidate.encoding"
                  :disabled="importPreviewConfirming"
                  @change="onCandidateChange(candidate.encoding, $event)"
                />
                <!-- aura-allow-text: DỮ LIỆU (nhãn FR126, một trong năm chuỗi cố định). -->
                <span class="ip-candidate-label">{{ candidate.label }}</span>
                <span v-if="candidate.preview !== null" class="ip-candidate-preview">
                  <!-- aura-allow-text: DỮ LIỆU (bản dựng thật từ Rust, KHÔNG markup — AD-16). -->
                  {{ candidate.preview }}
                </span>
                <span v-else class="ip-candidate-preview ip-candidate-undecodable">
                  {{ t('mode.library.preview.candidate_undecodable') }}
                </span>
              </label>
            </div>
          </template>
        </section>

        <!-- ═══════════════ Tầng 2 — ranh giới nội dung (RỖNG, có chủ) ═══════════════ -->
        <section class="ip-tier ip-tier-empty" aria-labelledby="ip-tier-2-title">
          <h3 id="ip-tier-2-title" class="ip-tier-title">{{ t('mode.library.preview.tier2_title') }}</h3>
          <p class="ip-tier-empty-reason">
            {{ t(tierEmptyMessageKey(importPreviewEmptyReasonForTier(2))) }}
          </p>
        </section>

        <!-- ═══════════════ Tầng 3 — luật làm sạch (RỖNG, có chủ) ═══════════════════ -->
        <section class="ip-tier ip-tier-empty" aria-labelledby="ip-tier-3-title">
          <h3 id="ip-tier-3-title" class="ip-tier-title">{{ t('mode.library.preview.tier3_title') }}</h3>
          <p class="ip-tier-empty-reason">
            {{ t(tierEmptyMessageKey(importPreviewEmptyReasonForTier(3))) }}
          </p>
        </section>

        <p v-if="importPreviewConfirmError !== null" class="ip-status ip-error" role="alert">
          <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
          {{ tError(importPreviewConfirmError) }}
        </p>
        <p v-else-if="importPreviewConfirming" class="ip-status" role="status">
          {{ t('mode.library.preview.confirming') }}
        </p>

        <p class="ip-hint">{{ t('mode.library.preview.hint_no_write_before_confirm') }}</p>

        <div class="ip-actions">
          <button
            type="button"
            class="ip-act ip-act-primary"
            :disabled="importPreviewConfirming"
            @click="dispatch('import.preview.confirm')"
          >
            {{ t('command.import.preview.confirm') }}
          </button>
          <button
            type="button"
            class="ip-act"
            :disabled="importPreviewConfirming"
            @click="dispatch('import.preview.cancel')"
          >
            {{ t('command.import.preview.cancel') }}
          </button>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.ip-scrim {
  position: fixed;
  inset: 0;
  z-index: 11; /* aura-allow-z-index: cùng tầng GlossaryImportOverlay — hai lớp phủ không mở đồng thời. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.ip-panel {
  width: 100%;
  max-width: 760px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.ip-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.ip-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.ip-close {
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

.ip-close:disabled {
  cursor: default;
}

.ip-status {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-empty {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.ip-error {
  color: var(--color-error);
}

.ip-tier {
  margin: 0 0 var(--space-panel-block) 0;
  padding-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

.ip-tier-title {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong, var(--weight-ui-md));
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.ip-confidence-chip {
  display: inline-flex;
  gap: calc(var(--space-unit) * 1);
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
}

.ip-strip-hint {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-strip-open {
  margin-left: calc(var(--space-unit) * 1);
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-strip {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: calc(var(--space-unit) * 2);
}

.ip-candidate {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  padding: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  cursor: pointer;
}

.ip-candidate-selected {
  border-color: var(--color-primary);
}

.ip-candidate-radio {
  align-self: flex-start;
}

.ip-candidate-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong, var(--weight-ui-md));
  color: var(--color-on-surface);
}

/* 🔴 §Always spec 6.3 — mẫu chữ trong dải đặt ở CỠ `read`, không cỡ giao diện. */
.ip-candidate-preview {
  font-family: var(--face-read-md);
  font-size: var(--font-read-md);
  line-height: var(--leading-read-md);
  color: var(--color-on-surface);
  word-break: break-word;
}

.ip-candidate-undecodable {
  color: var(--color-on-surface-variant);
}

.ip-tier-empty-reason {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-hint {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ip-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
}

.ip-act {
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.ip-act-primary {
  background: var(--color-primary);
  color: var(--color-on-primary);
  border-color: var(--color-primary);
}

.ip-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}
</style>
