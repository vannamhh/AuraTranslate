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
  bilingualImportPreviewActiveCuts,
  bilingualImportPreviewActiveMismatch,
  bilingualImportPreviewActiveMismatchIndex,
  bilingualImportPreviewCanConfirm,
  bilingualImportPreviewCanSkipActiveRow,
  bilingualImportPreviewCaretPosition,
  bilingualImportPreviewConfirmError,
  bilingualImportPreviewConfirming,
  bilingualImportPreviewHasHeader,
  bilingualImportPreviewIsOpen,
  bilingualImportPreviewLoadError,
  bilingualImportPreviewMismatches,
  bilingualImportPreviewSelectedCandidate,
  bilingualImportPreviewSelectedEncoding,
  bilingualImportPreviewSkippedTargetSentenceCount,
  bilingualImportPreviewSourceColumn,
  bilingualImportPreviewStatus,
  bilingualImportPreviewTargetColumn,
  selectBilingualEncoding,
  setBilingualChapterPattern,
  setBilingualHasHeader,
  setBilingualSourceColumn,
  setBilingualTargetColumn,
} from './bilingualImportPreviewState'
import type { BilingualMismatchWire, ChapterPatternKindWire } from './config/project'

/** Cắt `line` tại `cuts` (chỉ số KÝ TỰ UNICODE, KHÔNG byte — `Array.from` để tách theo CODE
 * POINT, cùng đơn vị Rust dùng qua `chars()`) — DỰNG HIỂN THỊ thuần, không quyết định gì:
 * Rust là nơi DUY NHẤT phán quyết một tập cắt có cặp được hay không (AD-1). MỖI mảnh được
 * `.trim()` trước khi hiện — cùng phép `apply_cuts` phía Rust làm trước khi từ chối một mảnh
 * rỗng, nên hiển thị KHỚP ĐÚNG cái Rust sẽ thấy. Một mảnh trống hiện ra khi hai chỗ cắt (hay
 * một chỗ cắt và một đầu dòng) chỉ bọc quanh khoảng trắng — trim xong không còn gì. */
function piecesFor(line: string, cuts: number[]): string[] {
  const chars = Array.from(line)
  const sorted = [...cuts].sort((a, b) => a - b)
  const bounds = [0, ...sorted, chars.length]
  const pieces: string[] = []
  for (let i = 0; i + 1 < bounds.length; i += 1) {
    pieces.push(chars.slice(bounds[i], bounds[i + 1]).join('').trim())
  }
  return pieces
}

/** Mảnh đích HIỆN HÀNH của hàng đang lấy tiêu điểm — trống khi chưa có hàng nào. */
function activeTargetPieces(): string[] {
  const active = bilingualImportPreviewActiveMismatch.value
  if (active === null) return []
  return piecesFor(active.target_line, bilingualImportPreviewActiveCuts.value)
}

/** Mảnh đích ĐỀ XUẤT của hàng đang lấy tiêu điểm — dùng cho dòng xem trước đề xuất, KHÔNG
 * đổi tập cắt hiện hành (§Always: "applied only by an explicit act"). */
function proposedTargetPieces(mismatch: BilingualMismatchWire): string[] {
  return piecesFor(mismatch.target_line, mismatch.proposed_cuts)
}

/** `target_line` của hàng đang lấy tiêu điểm, chèn `▏` tại caret và `‖` tại mỗi chỗ cắt hiện
 * hành — dòng THÔ cho người dùng thấy đúng caret/cuts đang di, cùng đơn vị Rust dùng (chỉ số
 * KÝ TỰ UNICODE, `Array.from` để tách theo CODE POINT). Bổ sung cho hai cột mảnh ở trên, không
 * thay thế: cột mảnh cho biết KẾT QUẢ, dòng này cho biết ĐANG SỬA Ở ĐÂU. */
function annotatedTargetLine(): string {
  const active = bilingualImportPreviewActiveMismatch.value
  if (active === null) return ''
  const chars = Array.from(active.target_line)
  const cuts = new Set(bilingualImportPreviewActiveCuts.value)
  const caret = bilingualImportPreviewCaretPosition.value
  let out = ''
  for (let i = 0; i <= chars.length; i += 1) {
    if (i === caret) out += '▏'
    if (cuts.has(i)) out += '‖'
    if (i < chars.length) out += chars[i]
  }
  return out
}

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

/**
 * Handler DOM CỤC BỘ trên `.bip-scrim` cho bảy lệnh quy nhóm (Story 6.17, FR116) — cùng khuôn
 * `ImportPreviewOverlay.vue::onTier2Keydown`: lọc vùng gõ TRƯỚC, gác `event.repeat` (một lượt
 * giữ phím không được bắn một tràng IPC — cùng lý lẽ `onChapterCursorKeydown`), mọi phím
 * `dispatch()` một lệnh ĐÃ ĐĂNG KÝ thay vì gọi thẳng state.
 *
 * Bảy phím: `↑ ↓` chuyển hàng lệch cặp, `← →` di caret giữa các điểm ứng viên, `Enter` bật/tắt
 * chỗ cắt tại caret, `A` áp mọi đề xuất, `S` bỏ qua hàng đang lấy tiêu điểm. KHÔNG `Space` trần
 * (Design Notes spec 6.17), KHÔNG rebind `Esc` (đã cancel lớp phủ, `onEscapeCancel` ở trên).
 */
function onMismatchKeydown(event: KeyboardEvent): void {
  if (event.ctrlKey || event.metaKey || event.altKey) return

  const target = event.target
  const isFormField =
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    target instanceof HTMLButtonElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  if (isFormField) return

  if (event.repeat) return

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      dispatch('import.preview.bilingual_next_mismatch')
      return
    case 'ArrowUp':
      event.preventDefault()
      dispatch('import.preview.bilingual_previous_mismatch')
      return
    case 'ArrowLeft':
      event.preventDefault()
      dispatch('import.preview.bilingual_caret_left')
      return
    case 'ArrowRight':
      event.preventDefault()
      dispatch('import.preview.bilingual_caret_right')
      return
    case 'Enter':
      event.preventDefault()
      dispatch('import.preview.bilingual_toggle_cut')
      return
    case 'a':
    case 'A':
      event.preventDefault()
      dispatch('import.preview.bilingual_accept_all_proposals')
      return
    case 's':
    case 'S':
      event.preventDefault()
      dispatch('import.preview.bilingual_skip_row')
      return
    default:
      return
  }
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
    @keydown="onMismatchKeydown"
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

            <p v-if="bilingualImportPreviewSkippedTargetSentenceCount > 0" class="bip-counts" role="status">
              <!-- aura-allow-text: KẾT QUẢ của `t()`, tham số là DỮ LIỆU (tổng câu đích của các
                   hàng đang đánh dấu Skip, Rust tính qua `target_sentence_count`). -->
              {{
                t('mode.library.preview.bilingual_skip_dropped_target_count', {
                  count: String(bilingualImportPreviewSkippedTargetSentenceCount),
                })
              }}
            </p>

            <div
              v-if="bilingualImportPreviewMismatches.length > 0 && bilingualImportPreviewActiveMismatch !== null"
              class="bip-mismatch-focus"
            >
              <p class="bip-mismatch-heading" role="status">
                <!-- aura-allow-text: KẾT QUẢ của `t()`, mọi tham số là DỮ LIỆU (chỉ số/số đếm). -->
                {{
                  t('mode.library.preview.bilingual_mismatch_heading', {
                    index: String(bilingualImportPreviewActiveMismatchIndex + 1),
                    total: String(bilingualImportPreviewMismatches.length),
                    row: String(bilingualImportPreviewActiveMismatch.row_number),
                    chapter: String(bilingualImportPreviewActiveMismatch.chapter_index + 1),
                  })
                }}
              </p>

              <div class="bip-mismatch-columns">
                <div class="bip-mismatch-column">
                  <h4 class="bip-mismatch-column-title">{{ t('mode.library.preview.bilingual_source_column') }}</h4>
                  <ol class="bip-sentence-list">
                    <li v-for="(s, i) in bilingualImportPreviewActiveMismatch.source_sentences" :key="i">
                      <!-- aura-allow-text: DỮ LIỆU (câu nguồn thật từ Rust). -->
                      {{ s }}
                    </li>
                  </ol>
                </div>
                <div class="bip-mismatch-column">
                  <h4 class="bip-mismatch-column-title">{{ t('mode.library.preview.bilingual_target_column') }}</h4>
                  <ol class="bip-sentence-list">
                    <li v-for="(piece, i) in activeTargetPieces()" :key="i">
                      <!-- aura-allow-text: DỮ LIỆU (mảnh đích, hiện tính từ tập cắt hiện hành). -->
                      {{ piece === '' ? t('mode.library.preview.bilingual_empty_piece') : piece }}
                    </li>
                  </ol>
                </div>
              </div>

              <p
                v-if="bilingualImportPreviewActiveMismatch.source_sentences.length > 0 && bilingualImportPreviewActiveMismatch.target_line !== ''"
                class="bip-mismatch-caret-line"
              >
                {{ t('mode.library.preview.bilingual_caret_line_label') }}
                <!-- aura-allow-text: DỮ LIỆU (dòng đích thô, chèn ký hiệu caret/cắt). -->
                <span class="bip-caret-line">{{ annotatedTargetLine() }}</span>
              </p>

              <p
                v-if="bilingualImportPreviewActiveMismatch.source_sentences.length > 0 && bilingualImportPreviewActiveMismatch.target_line !== ''"
                class="bip-mismatch-proposal"
              >
                {{ t('mode.library.preview.bilingual_proposal_label') }}
                <!-- aura-allow-text: DỮ LIỆU (mảnh đề xuất, KHÔNG áp cho tới khi bấm "Áp mọi đề xuất"). -->
                {{ proposedTargetPieces(bilingualImportPreviewActiveMismatch).join(' / ') }}
              </p>

              <div class="bip-mismatch-row-actions">
                <button
                  type="button"
                  class="bip-mismatch-action"
                  :disabled="bilingualImportPreviewConfirming"
                  @click="dispatch('import.preview.bilingual_accept_all_proposals')"
                >
                  {{ t('command.import.preview.bilingual_accept_all_proposals') }}
                </button>
                <button
                  v-if="bilingualImportPreviewCanSkipActiveRow"
                  type="button"
                  class="bip-mismatch-action"
                  :disabled="bilingualImportPreviewConfirming"
                  @click="dispatch('import.preview.bilingual_skip_row')"
                >
                  {{ t('command.import.preview.bilingual_skip_row') }}
                </button>
              </div>

              <p class="bip-mismatch-hint">{{ t('mode.library.preview.bilingual_keyboard_hint') }}</p>
            </div>
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

.bip-swap,
.bip-mismatch-action {
  height: fit-content;
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  background: none;
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
}

.bip-swap:disabled,
.bip-mismatch-action:disabled {
  cursor: default;
}

.bip-counts {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.bip-mismatch-focus {
  padding: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-error);
}

.bip-mismatch-heading {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.bip-mismatch-columns {
  display: flex;
  gap: calc(var(--space-unit) * 3);
  margin-bottom: calc(var(--space-unit) * 2);
}

.bip-mismatch-column {
  flex: 1;
  min-width: 0;
}

.bip-mismatch-column-title {
  margin: 0 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong, var(--weight-ui-md));
  color: var(--color-on-surface-variant);
}

.bip-sentence-list {
  margin: 0;
  padding: 0 0 0 calc(var(--space-unit) * 3);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
}

.bip-mismatch-caret-line {
  margin: 0 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.bip-caret-line {
  font-family: var(--face-ui-sm);
  color: var(--color-on-surface);
}

.bip-mismatch-proposal {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.bip-mismatch-row-actions {
  display: flex;
  gap: calc(var(--space-unit) * 2);
  margin-bottom: calc(var(--space-unit) * 2);
}

.bip-mismatch-hint {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
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
