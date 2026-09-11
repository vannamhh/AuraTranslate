<script setup lang="ts">
// Lớp phủ **Xem trước lượt nhập song ngữ** (Story 6.16, FR115, AD-39 · AD-37/46 · AD-47 ③).
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 KHUÔN `ImportPreviewOverlay.vue` — scrim + `role="dialog"` + bẫy Tab tự viết +
// focus-return qua `data-bilingual-import-preview-open` (đặt trên nút nộp form RIÊNG của
// `LibraryMode.vue` — không dùng chung `data-import-preview-open` với ba nhánh cũ, hai lớp
// phủ độc lập không được tranh cùng một mỏ neo focus-return).
//
// Đổi cột nguồn/đích, bật/tắt tiêu đề, và gửi mẫu phân tách Chương đi qua `@change` (KHÔNG
// `dispatch`, cùng tiền lệ dải năm ứng viên bảng mã và mẫu phân tách Chương của
// `ImportPreviewOverlay.vue` — `check:commands` Kiểm A chỉ canh `@click`).
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
// Không `v-html` ở bất kỳ đâu (AD-16) — mọi ô mẫu là DỮ LIỆU văn bản thô từ Rust.
import { nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import {
  bilingualImportPreview,
  bilingualImportPreviewCanConfirm,
  bilingualImportPreviewConfirmError,
  bilingualImportPreviewConfirming,
  bilingualImportPreviewHasHeader,
  bilingualImportPreviewIsOpen,
  bilingualImportPreviewLoadError,
  bilingualImportPreviewSelectedCandidate,
  bilingualImportPreviewSelectedEncoding,
  bilingualImportPreviewSourceColumn,
  bilingualImportPreviewStatus,
  bilingualImportPreviewTargetColumn,
  selectBilingualEncoding,
  setBilingualChapterPattern,
  setBilingualHasHeader,
  setBilingualSourceColumn,
  setBilingualTargetColumn,
} from './bilingualImportPreviewState'
import type { ChapterPatternKindWire } from './config/project'

/** Cột nào của hàng mẫu đầu tiên đứng làm nhãn cho ô `<select>` thứ `index` — chuỗi rỗng khi
 * không có hàng mẫu nào (tệp rỗng) hoặc chỉ số vượt quá số cột thật của hàng đó. */
function sampleCellFor(index: number): string {
  const rows = bilingualImportPreview.value?.sample_rows ?? []
  if (rows.length === 0) return ''
  return rows[0][index] ?? ''
}

function onSourceColumnChange(event: Event): void {
  const value = Number((event.target as HTMLSelectElement).value)
  void setBilingualSourceColumn(value)
}

function onTargetColumnChange(event: Event): void {
  const value = Number((event.target as HTMLSelectElement).value)
  void setBilingualTargetColumn(value)
}

function onHasHeaderChange(event: Event): void {
  void setBilingualHasHeader((event.target as HTMLInputElement).checked)
}

function onCandidateChange(encoding: string, event: Event): void {
  if (!(event.target as HTMLInputElement).checked) return
  selectBilingualEncoding(encoding)
}

const chapterPatternKinds: ChapterPatternKindWire[] = ['literal', 'regex']

function onChapterPatternTextChange(event: Event): void {
  const text = (event.target as HTMLInputElement).value
  void setBilingualChapterPattern(text, bilingualImportPreview.value === null ? 'literal' : chapterPatternKindValue())
}

function chapterPatternKindValue(): ChapterPatternKindWire {
  const el = document.getElementById('bip-chapter-pattern-kind') as HTMLSelectElement | null
  const value = el?.value
  return chapterPatternKinds.includes(value as ChapterPatternKindWire) ? (value as ChapterPatternKindWire) : 'literal'
}

function onChapterPatternKindChange(event: Event): void {
  const kind = (event.target as HTMLSelectElement).value as ChapterPatternKindWire
  const textEl = document.getElementById('bip-chapter-pattern-text') as HTMLInputElement | null
  void setBilingualChapterPattern(textEl?.value ?? '', kind)
}

function onEscapeCancel(): void {
  if (bilingualImportPreviewConfirming.value) return
  dispatch('import.preview.bilingual_cancel')
}

/** 🔴 Trả tiêu điểm về chỗ cũ — khuôn và lý lẽ chép từ `ImportPreviewOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')

watch(bilingualImportPreviewIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-bilingual-import-preview-open]')
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-bilingual-import-preview-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[bilingual-import-preview] focus-return target is gone; focus falls back to body.')
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
</script>

<template>
  <div
    v-if="bilingualImportPreviewIsOpen"
    class="bip-scrim"
    @keydown.esc="onEscapeCancel"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="bip-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="bip-head">
        <h2 class="bip-title">{{ t('mode.library.preview.bilingual_title') }}</h2>
        <button
          type="button"
          class="bip-close"
          :disabled="bilingualImportPreviewConfirming"
          @click="dispatch('import.preview.bilingual_cancel')"
        >
          {{ t('command.import.preview.bilingual_cancel') }}
        </button>
      </header>

      <p v-if="bilingualImportPreviewStatus === 'unknown'" class="bip-status" role="status">
        {{ t('mode.library.preview.loading') }}
      </p>
      <p v-else-if="bilingualImportPreviewStatus === 'ipc_unavailable'" class="bip-empty">
        {{ t('mode.library.preview.empty_ipc_unavailable') }}
      </p>
      <p
        v-else-if="bilingualImportPreviewStatus === 'error' && bilingualImportPreviewLoadError !== null"
        class="bip-empty bip-error"
        role="alert"
      >
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(bilingualImportPreviewLoadError) }}
      </p>

      <template v-else-if="bilingualImportPreviewStatus === 'loaded' && bilingualImportPreview !== null">
        <!-- ═══════════════════ Dải năm ứng viên bảng mã ═══════════════════ -->
        <section class="bip-tier" aria-labelledby="bip-tier-encoding-title">
          <h3 id="bip-tier-encoding-title" class="bip-tier-title">
            {{ t('mode.library.preview.tier1_title') }}
          </h3>
          <div class="bip-strip" role="radiogroup" :aria-label="t('mode.library.preview.strip_hint')">
            <label
              v-for="candidate in bilingualImportPreview.candidates"
              :key="candidate.encoding"
              class="bip-candidate"
              :class="{ 'bip-candidate-selected': bilingualImportPreviewSelectedEncoding === candidate.encoding }"
            >
              <input
                type="radio"
                name="bip-candidate"
                class="bip-candidate-radio"
                :checked="bilingualImportPreviewSelectedEncoding === candidate.encoding"
                :disabled="bilingualImportPreviewConfirming"
                @change="onCandidateChange(candidate.encoding, $event)"
              />
              <!-- aura-allow-text: DỮ LIỆU (nhãn FR126, một trong năm chuỗi cố định). -->
              <span class="bip-candidate-label">{{ candidate.label }}</span>
              <span v-if="candidate.preview !== null" class="bip-candidate-preview">
                <!-- aura-allow-text: DỮ LIỆU (bản dựng thật từ Rust, KHÔNG markup — AD-16). -->
                {{ candidate.preview }}
              </span>
              <span v-else class="bip-candidate-preview bip-candidate-undecodable">
                {{ t('mode.library.preview.candidate_undecodable') }}
              </span>
            </label>
          </div>
        </section>

        <!-- ═══════════════════ Vai cột + tiêu đề + mẫu phân tách ═══════════════════ -->
        <section class="bip-tier" aria-labelledby="bip-tier-columns-title">
          <h3 id="bip-tier-columns-title" class="bip-tier-title">
            {{ t('mode.library.preview.bilingual_columns_title') }}
          </h3>

          <div class="bip-columns">
            <label class="bip-field">
              <span>{{ t('mode.library.preview.bilingual_source_column') }}</span>
              <select :value="bilingualImportPreviewSourceColumn" @change="onSourceColumnChange">
                <!-- 🔴 SỬA (vòng rà đối kháng) — nhãn hiện số 1-based (cùng khuôn Chương/hàng
                     trên cùng màn hình: "Cột 1" chứ không "Cột 0"); `value` giữ NGUYÊN 0-based
                     (chỉ số cột thật Rust nhận). -->
                <option v-for="i in bilingualImportPreview.column_count" :key="i - 1" :value="i - 1">
                  <!-- aura-allow-text: KẾT QUẢ của `t()`, tham số là DỮ LIỆU. -->
                  {{ t('mode.library.preview.bilingual_column_option', { index: String(i), sample: sampleCellFor(i - 1) }) }}
                </option>
              </select>
            </label>

            <button
              type="button"
              class="bip-swap"
              :disabled="bilingualImportPreviewConfirming"
              @click="dispatch('import.preview.bilingual_swap_columns')"
            >
              {{ t('command.import.preview.bilingual_swap_columns') }}
            </button>

            <label class="bip-field">
              <span>{{ t('mode.library.preview.bilingual_target_column') }}</span>
              <select :value="bilingualImportPreviewTargetColumn" @change="onTargetColumnChange">
                <!-- Xem chú thích cột nguồn ở trên: nhãn 1-based, `value` 0-based. -->
                <option v-for="i in bilingualImportPreview.column_count" :key="i - 1" :value="i - 1">
                  <!-- aura-allow-text: KẾT QUẢ của `t()`, tham số là DỮ LIỆU. -->
                  {{ t('mode.library.preview.bilingual_column_option', { index: String(i), sample: sampleCellFor(i - 1) }) }}
                </option>
              </select>
            </label>
          </div>

          <label class="bip-checkbox-field">
            <input
              type="checkbox"
              :checked="bilingualImportPreviewHasHeader"
              @change="onHasHeaderChange"
            />
            <span>{{ t('mode.library.preview.bilingual_has_header') }}</span>
          </label>

          <form class="bip-chapters-pattern-row" @submit.prevent>
            <label class="bip-field">
              <span>{{ t('mode.library.preview.chapters_pattern_label') }}</span>
              <input
                id="bip-chapter-pattern-text"
                type="text"
                autocomplete="off"
                :placeholder="t('mode.library.preview.chapters_pattern_placeholder')"
                @change="onChapterPatternTextChange"
              />
            </label>
            <select id="bip-chapter-pattern-kind" @change="onChapterPatternKindChange">
              <option value="literal">{{ t('mode.library.preview.cleanup_kind_literal') }}</option>
              <option value="regex">{{ t('mode.library.preview.cleanup_kind_regex') }}</option>
            </select>
          </form>
        </section>

        <!-- ═══════════════════ Số đếm + hàng lệch cặp ═══════════════════ -->
        <section class="bip-tier" aria-labelledby="bip-tier-counts-title">
          <h3 id="bip-tier-counts-title" class="bip-tier-title">
            {{ t('mode.library.preview.bilingual_counts_title') }}
          </h3>

          <template v-if="bilingualImportPreviewSelectedCandidate !== null">
            <p class="bip-counts">
              <!-- aura-allow-text: KẾT QUẢ của `t()`, tham số là DỮ LIỆU (số đếm từ Rust). -->
              {{ t('mode.library.preview.bilingual_row_count', { count: String(bilingualImportPreviewSelectedCandidate.row_count) }) }}
              ·
              {{ t('mode.library.preview.bilingual_chapter_count', { count: String(bilingualImportPreviewSelectedCandidate.chapter_count) }) }}
              ·
              {{ t('mode.library.preview.bilingual_pair_count', { count: String(bilingualImportPreviewSelectedCandidate.pair_count) }) }}
            </p>

            <ul v-if="bilingualImportPreviewSelectedCandidate.mismatches.length > 0" class="bip-mismatch-list">
              <li v-for="(m, i) in bilingualImportPreviewSelectedCandidate.mismatches" :key="i" class="bip-mismatch-item">
                <!-- aura-allow-text: KẾT QUẢ của `t()`, mọi tham số là DỮ LIỆU (chỉ số/số đếm). -->
                {{
                  t('mode.library.preview.bilingual_mismatch_row', {
                    chapter: String(m.chapter_index + 1),
                    row: String(m.row_number),
                    source: String(m.source_sentence_count),
                    target: String(m.target_sentence_count),
                  })
                }}
              </li>
            </ul>
          </template>
        </section>
      </template>

      <p
        v-if="bilingualImportPreviewConfirmError !== null"
        class="bip-status bip-error"
        role="alert"
      >
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(bilingualImportPreviewConfirmError) }}
      </p>

      <div class="bip-actions">
        <button
          type="button"
          class="bip-act bip-act-primary"
          :disabled="bilingualImportPreviewConfirming || !bilingualImportPreviewCanConfirm"
          @click="dispatch('import.preview.bilingual_confirm')"
        >
          {{ t('command.import.preview.bilingual_confirm') }}
        </button>
        <p
          v-if="
            bilingualImportPreviewStatus === 'loaded' &&
            bilingualImportPreviewSelectedCandidate !== null &&
            bilingualImportPreviewSelectedCandidate.mismatches.length > 0
          "
          class="bip-tier-empty-reason"
          role="status"
        >
          {{
            t('mode.library.preview.bilingual_confirm_locked_reason', {
              count: String(bilingualImportPreviewSelectedCandidate.mismatches.length),
            })
          }}
        </p>
      </div>
    </section>
  </div>
</template>

<style scoped>
.bip-scrim {
  position: fixed;
  inset: 0;
  z-index: 11; /* aura-allow-z-index: cùng tầng ImportPreviewOverlay — hai lớp phủ không mở đồng thời. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.bip-panel {
  width: 100%;
  max-width: 760px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.bip-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.bip-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.bip-close {
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

.bip-close:disabled {
  cursor: default;
}

.bip-status {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.bip-empty {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.bip-error {
  color: var(--color-error);
}

.bip-tier {
  margin: 0 0 var(--space-panel-block) 0;
  padding-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

.bip-tier-title {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong, var(--weight-ui-md));
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.bip-strip {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
}

.bip-candidate {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 1);
  border: 1px solid var(--color-outline);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
}

.bip-candidate-selected {
  border-color: var(--color-on-surface);
}

.bip-candidate-preview {
  overflow: hidden;
  color: var(--color-on-surface-variant);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bip-candidate-undecodable {
  color: var(--color-error);
}

.bip-columns {
  display: flex;
  align-items: flex-end;
  gap: calc(var(--space-unit) * 2);
  margin-bottom: calc(var(--space-unit) * 2);
}

.bip-field {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin-bottom: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
}

.bip-chapters-pattern-row {
  display: flex;
  align-items: flex-end;
  gap: calc(var(--space-unit) * 2);
}

.bip-checkbox-field {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  margin-bottom: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
}

.bip-swap {
  height: fit-content;
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  background: none;
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
}

.bip-swap:disabled {
  cursor: default;
}

.bip-counts {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.bip-mismatch-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.bip-mismatch-item {
  padding: calc(var(--space-unit) * 1) 0;
  border-bottom: 1px solid var(--color-outline);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.bip-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: calc(var(--space-unit) * 1);
}

.bip-act {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: none;
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.bip-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.bip-act-primary {
  background: var(--color-primary);
  color: var(--color-on-primary);
  border-color: var(--color-primary);
}

.bip-tier-empty-reason {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}
</style>
