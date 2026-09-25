<script setup lang="ts">
// Lớp phủ **Xem prompt cuối cùng đã gửi** — Story 4.7 (FR71, AD-14).
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MÀN HÌNH NÀY CHỈ ĐỌC BẢN GHI — KHÔNG BAO GIỜ LẮP RÁP (Decision 2 spec 4.7)
// ─────────────────────────────────────────────────────────────────────────────
// Không một import nào ở tệp này chạm `config/aiprompt.ts::aiPromptAssemble`. Mở lớp phủ
// (`ai.prompt_inspector.open`, handler `openAiPromptInspector` ở `aiPromptInspectorState.ts`)
// chỉ gọi `ai_prompt_read_record` — nút "Lắp prompt cho câu này" sống ở `AiTranslationPanel.vue`,
// một bề mặt HOÀN TOÀN riêng, và Story 4.8 sẽ thay nó bằng nút Dịch thật (§Consequences
// accepted, Decision 2).
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 BA-VÀ-BA GIÁ TRỊ, KHÔNG COLLAPSE — §Always spec 4.7
// ─────────────────────────────────────────────────────────────────────────────
// (1) chưa Lắp lần nào trong phiên (`aiPromptRecord === null`) KHÁC "một bản ghi có `prompt`
//     rỗng" (`.body_empty_note`, không đọc như "chưa có gì"); (2) Glossary `not_asked` (thân bộ
//     prompt không mang biến chèn — không truy vấn nào chạy) KHÁC `asked` với 0 thuật ngữ khớp;
//     (3) TM `not_built_yet` KHÁC `searched` (kể cả rỗng) — Epic 7 mới có nhánh `searched` thật,
//     nhánh này giữ ở đây để `switch`/`computed` không thiếu ca, không phải vì có dữ liệu để hiện.
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`. Không `v-html`,
// không markup từ dữ liệu (AD-16) — mọi khối dưới đây render từ mô hình có cấu trúc của
// `AssembledPromptWire`, bằng binding Vue thường; văn bản DỮ LIỆU (thuật ngữ, ký hiệu lạ, chuỗi
// prompt) đi qua interpolation `{{ }}` thường — Vue escape HTML tự động, không phải `v-html`.
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import { editorCaretSegmentId } from './panels/editorPanelState'
import {
  aiPromptInspectorIsOpen,
  aiPromptReadError,
  aiPromptRecord,
  aiPromptRecordIsStale,
} from './aiPromptInspectorState'

const panel = useTemplateRef<HTMLElement>('panel')
// Màn hình hiện văn bản THẬT (prompt đã gửi, thuật ngữ Glossary) nhưng không phải nguồn tra
// từ điển — vai `'display'`, cùng lý do `PromptLibraryOverlay.vue`.
useSelectionSurface(panel, 'display')

/** 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Khuôn chép từ `PromptLibraryOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

watch(aiPromptInspectorIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-ai-prompt-inspector-open]')
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-ai-prompt-inspector-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[ai-prompt-inspector] focus-return target is gone; focus falls back to body.')
})

/** Điểm dừng Tab thật, theo đúng thứ tự tài liệu — khuôn chép từ `PromptLibraryOverlay.vue`. */
function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
}

/** Bẫy tiêu điểm — `Tab` xoay vòng TRONG lớp phủ, khuôn chép từ `PromptLibraryOverlay.vue`. */
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

/** Không nhịp chờ nào lồng trong màn hình này (khác `PromptLibraryOverlay.vue`'s "form Tạo"/
 * "chờ xoá") — `Escape` luôn đóng thẳng lớp phủ. */
function onEscape(): void {
  dispatch('ai.prompt_inspector.close')
}

/** Tên bộ prompt hai tầng dùng CHUNG nhãn — [`tierLabel`] tái dùng cho cả tầng của bộ prompt
 * (`prompt_set_tier`) LẪN tầng của mỗi thuật ngữ Glossary (`InjectedGlossaryTermWire::tier`):
 * cùng hai giá trị `'global' | 'work'`, cùng nhãn hiển thị. */
function tierLabel(tier: 'global' | 'work'): string {
  return t(tier === 'work' ? 'ai.prompt_inspector.tier_work' : 'ai.prompt_inspector.tier_global')
}

/** `true` ⇔ bản ghi đang hiện được lắp từ một câu KHÁC câu đang có tiêu điểm bây giờ — I/O
 * Matrix "Stale record". Hàm thuần [`aiPromptRecordIsStale`] sống ở `aiPromptInspectorState.ts`;
 * tệp NÀY là leaf duy nhất `import` `editorCaretSegmentId` để so sánh (cùng lý lẽ
 * `glossaryConfirmStripState.ts` đã ghi cho việc không đặt watcher đó trong một module core). */
const isStale = computed<boolean>(() => aiPromptRecordIsStale(aiPromptRecord.value, editorCaretSegmentId.value))

/**
 * Nhánh `'asked'` của Glossary, đã thu hẹp kiểu — hoặc `null` khi chưa có bản ghi hay
 * `kind === 'not_asked'`. Thu hẹp Ở SCRIPT, không dựa `v-if`/`v-else` thu hẹp một UNION trong
 * template: hai bản còn lại (`injected`/`suppressed_by_pending_overlap`) khi đó đọc được thẳng,
 * có kiểu, không ép `as`.
 */
const glossaryAsked = computed(() => {
  const rec = aiPromptRecord.value
  if (rec === null) return null
  const status = rec.ledger.glossary
  return status.kind === 'asked' ? status : null
})

/** Cùng lý lẽ [`glossaryAsked`], cho nhánh `'searched'` của TM — Epic 7 mới có dữ liệu thật ở
 * nhánh này; giữ để không thiếu ca khi ngày đó tới. */
const tmSearched = computed(() => {
  const rec = aiPromptRecord.value
  if (rec === null) return null
  const status = rec.ledger.tm
  return status.kind === 'searched' ? status : null
})
</script>

<template>
  <div
    v-if="aiPromptInspectorIsOpen"
    class="aip-scrim"
    @keydown.esc="onEscape"
    @keydown.tab="trapTab($event)"
  >
    <section
      ref="panel"
      class="aip-panel"
      tabindex="-1"
      role="dialog"
      aria-modal="true"
      aria-labelledby="aip-title"
    >
      <header class="aip-head">
        <!-- finding B15 (loop 1) -- `id` trỏ đích của `aria-labelledby` ở trên; một lớp phủ
             MỚI không nên thừa hưởng khoảng hở a11y của cả họ (`PromptLibraryOverlay.vue`/
             `PromptImportOverlay.vue`/`GlossaryImportOverlay.vue` — món nợ RIÊNG của chúng,
             xem `deferred-work.md`), việc sửa cả họ cùng lúc là phạm vi ngoài story này. -->
        <h2 id="aip-title" class="aip-title">{{ t('ai.prompt_inspector.title') }}</h2>
        <button type="button" class="aip-close" @click="dispatch('ai.prompt_inspector.close')">
          {{ t('command.ai.prompt_inspector.close') }}
        </button>
      </header>

      <!-- 🔴 findings B4/E3/E10 (loop 1) -- lượt Đọc gần nhất trượt (hình dạng dây sai, mất
           cầu Tauri): bản ghi ĐANG HIỂN THỊ (nếu có) không bị xoá bởi lượt trượt này
           (`refreshAiPromptRecord` giữ nguyên `record` khi `readError !== null` — xem
           doc-comment tại đó), nhưng người xem phải biết nó có thể không phải bản mới nhất. -->
      <p v-if="aiPromptReadError !== null" class="aip-read-error" role="alert" data-aip-read-error="true">
        <!-- aura-allow-text: KẾT QUẢ của tError(). -->
        {{ tError(aiPromptReadError) }}
      </p>

      <p v-if="aiPromptRecord === null" class="aip-empty" role="status" data-aip-record-state="none">
        {{ t('ai.prompt.summary_no_record') }}
      </p>

      <div v-else class="aip-body" data-aip-record-state="present">
        <p class="aip-identity">
          <!-- aura-allow-text: KẾT QUẢ của t() (nội suy dữ liệu bản ghi — số câu/Chương/tên bộ/tầng). -->
          {{
            t('ai.prompt_inspector.identity', {
              segment_id: String(aiPromptRecord.segment_id),
              chapter_id: String(aiPromptRecord.chapter_id),
              name: aiPromptRecord.prompt_set_name,
              tier: tierLabel(aiPromptRecord.prompt_set_tier),
            })
          }}
        </p>

        <p v-if="isStale" class="aip-stale" role="status" data-aip-stale="true">
          <!-- aura-allow-text: KẾT QUẢ của t() (nội suy số câu). -->
          {{
            t('ai.prompt_inspector.stale_notice', {
              record_segment_id: String(aiPromptRecord.segment_id),
              focused_segment_id: String(editorCaretSegmentId ?? ''),
            })
          }}
        </p>

        <section class="aip-section">
          <h3 class="aip-sh">{{ t('ai.prompt_inspector.body_heading') }}</h3>
          <p v-if="aiPromptRecord.prompt === ''" class="aip-note" data-aip-body-state="empty">
            {{ t('ai.prompt_inspector.body_empty_note') }}
          </p>
          <!-- 🔴 finding B1 (loop 1) -- vẽ TỪNG mảnh `ledger.pieces` theo nhãn của nó, KHÔNG
               phải cả `prompt` như MỘT khối màu. Đây là cơ chế khiến phần chèn động (Glossary)
               phân biệt được trực quan khỏi thân do người dùng soạn (§Always spec 4.7), mà
               KHÔNG lắp lại (`pieces` đến thẳng từ bản ghi, không một lượt `assemble_prompt`
               thứ hai) và KHÔNG quét `prompt` lần nữa (chỉ đọc mảng đã có sẵn trên dây). Bản
               thân comment `aura-allow-text` phải đứng NGAY BÊN TRONG `<pre>`, sát `<span>` —
               `check-i18n.mjs` neo miễn trừ vào SIBLING trực tiếp của text node, không phải
               một tổ tiên xa hơn; đặt nó phía trên `<pre>` (như bản nháp đầu) không được nhận
               ra, xem SỬA bên dưới. Không khoảng trắng/xuống dòng nào chen giữa các thẻ bên
               trong `<pre>` — nó giữ nguyên văn mọi khoảng trắng, kể cả comment liền kề. -->
          <pre v-else class="aip-prompt-text" data-aip-body-state="present"><!-- aura-allow-text: DU LIEU (van ban cua tung manh prompt da lap -- ban ghi, khong phai chu giao dien). --><span v-for="(piece, i) in aiPromptRecord.ledger.pieces" :key="i" :class="['aip-piece', `aip-piece-${piece.kind}`]" :data-aip-piece-kind="piece.kind">{{ piece.text }}</span></pre>

          <p v-if="aiPromptRecord.ledger.source_segment_missing" class="aip-warn" role="alert">
            {{ t('ai.prompt_inspector.source_missing_warning') }}
          </p>

          <div v-if="aiPromptRecord.ledger.unknown_markers.length > 0" class="aip-markers">
            <h4 class="aip-sh2">{{ t('ai.prompt_inspector.markers_heading') }}</h4>
            <ul class="aip-marker-list">
              <li v-for="m in aiPromptRecord.ledger.unknown_markers" :key="m">
                <!-- aura-allow-text: DỮ LIỆU (toàn văn ký hiệu lạ, không phải chữ giao diện). -->
                <code>{{ m }}</code>
              </li>
            </ul>
          </div>
        </section>

        <section class="aip-section">
          <h3 class="aip-sh">{{ t('ai.prompt_inspector.glossary_heading') }}</h3>

          <p v-if="glossaryAsked === null" class="aip-note" data-aip-glossary-kind="not_asked">
            {{ t('ai.prompt_inspector.glossary_not_asked') }}
          </p>

          <template v-else>
            <p class="aip-summary" data-aip-glossary-kind="asked">
              {{ t('ai.prompt.summary_asked', { count: String(glossaryAsked.injected.length) }) }}
            </p>

            <h4 class="aip-sh2">{{ t('ai.prompt_inspector.glossary_injected_heading') }}</h4>
            <p v-if="glossaryAsked.injected.length === 0" class="aip-note">
              {{ t('ai.prompt_inspector.glossary_injected_empty') }}
            </p>
            <ul v-else class="aip-term-list aip-term-list-injected">
              <li v-for="(term, i) in glossaryAsked.injected" :key="`inj-${i}`" class="aip-term-row">
                <!-- aura-allow-text: DỮ LIỆU (thuật ngữ Glossary do người dùng đặt). -->
                <span class="aip-term-source">{{ term.source_term }}</span>
                <span class="aip-term-arrow" aria-hidden="true">→</span>
                <!-- aura-allow-text: DỮ LIỆU (bản dịch đã chốt do người dùng đặt). -->
                <span class="aip-term-translation">{{ term.translation }}</span>
                <!-- aura-allow-text: KẾT QUẢ của t() qua hàm cục bộ `tierLabel()`, không một chuỗi viết thẳng. -->
                <span class="aip-term-tier">{{ tierLabel(term.tier) }}</span>
              </li>
            </ul>

            <h4 class="aip-sh2">{{ t('ai.prompt_inspector.glossary_suppressed_heading') }}</h4>
            <p v-if="glossaryAsked.suppressed_by_pending_overlap.length === 0" class="aip-note">
              {{ t('ai.prompt_inspector.glossary_suppressed_empty') }}
            </p>
            <ul v-else class="aip-term-list aip-term-list-suppressed">
              <li
                v-for="(term, i) in glossaryAsked.suppressed_by_pending_overlap"
                :key="`sup-${i}`"
                class="aip-term-row"
              >
                <!-- aura-allow-text: DỮ LIỆU (thuật ngữ Glossary do người dùng đặt). -->
                <span class="aip-term-source">{{ term.source_term }}</span>
                <span class="aip-term-arrow" aria-hidden="true">→</span>
                <!-- aura-allow-text: DỮ LIỆU (bản dịch đã chốt do người dùng đặt). -->
                <span class="aip-term-translation">{{ term.translation }}</span>
                <!-- aura-allow-text: KẾT QUẢ của t() qua hàm cục bộ `tierLabel()`, không một chuỗi viết thẳng. -->
                <span class="aip-term-tier">{{ tierLabel(term.tier) }}</span>
                <span class="aip-term-reason">
                  {{ t('ai.prompt_inspector.glossary_suppressed_reason_pending_overlap') }}
                </span>
              </li>
            </ul>
          </template>
        </section>

        <section class="aip-section">
          <h3 class="aip-sh">{{ t('ai.prompt_inspector.tm_heading') }}</h3>
          <p v-if="tmSearched === null" class="aip-note" data-aip-tm-kind="not_built_yet">
            {{ t('ai.prompt_inspector.tm_not_built_yet') }}
          </p>
          <p v-else class="aip-note" data-aip-tm-kind="searched">
            {{ t('ai.prompt_inspector.tm_searched', { count: String(tmSearched.similar_segments.length) }) }}
          </p>
        </section>
      </div>
    </section>
  </div>
</template>

<style scoped>
.aip-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: xếp lớp CƠ HỌC — cùng lý do các lớp phủ khác. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.aip-panel {
  width: 100%;
  max-width: 860px;
  max-height: 100%;
  overflow: auto;
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  display: flex;
  flex-direction: column;
  padding: var(--space-panel-inline);
}

.aip-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.aip-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.aip-close {
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

.aip-empty {
  margin: 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.aip-identity {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.aip-stale,
.aip-read-error {
  margin: 0 0 calc(var(--space-unit) * 3) 0;
  padding: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-error);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.aip-section {
  margin-bottom: calc(var(--space-unit) * 4);
}

.aip-sh {
  font-size: var(--font-ui-label);
  font-family: var(--face-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  padding-bottom: calc(var(--space-unit) * 1);
  border-bottom: 1px solid var(--color-outline);
}

.aip-sh2 {
  font-size: var(--font-ui-label);
  font-family: var(--face-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
  margin: calc(var(--space-unit) * 3) 0 calc(var(--space-unit) * 1) 0;
}

.aip-note,
.aip-summary {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.aip-summary {
  color: var(--color-on-surface);
  font-weight: var(--weight-read-title);
}

.aip-warn {
  margin: calc(var(--space-unit) * 2) 0 0 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.aip-prompt-text {
  margin: 0;
  padding: calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

/* 🔴 finding B1 (loop 1) -- CHUYỂN từ `.aip-term-list-injected` (một bề mặt KHÁC: danh sách
   thuật ngữ, không phải thân prompt). Đây mới là quy tắc thật sự khiến §Always spec 4.7 đúng
   ("dynamically injected text is visually separable from the user-authored body"): TỪNG mảnh
   Glossary bên trong `.aip-prompt-text` mang nền riêng, phân biệt trực quan khỏi các mảnh
   `authored` liền kề trong CÙNG một khối `<pre>`. */
.aip-piece-glossary {
  background: var(--color-surface-accent);
}

/* 🔴 findings P7/P9 (loop 2) -- hai nhãn `pieces` chưa từng có quy tắc CSS riêng trước bản này.
   `source_segment` (P7): câu nguồn đã thay vào `{{source_segment}}` là phần ĐỘNG NHẤT của
   prompt (đổi mỗi lượt gọi) -- trước bản sửa nó mang nhãn `authored` và render giống hệt thân
   TĨNH do người soạn bộ prompt gõ, đúng khuyết tật §Always cấm. Khối "chìm" (`surface-sunken` +
   `on-surface-variant`) là quy ước đã có sẵn của kho cho một khối trích dẫn/chỉ đọc (cùng cặp
   token `ImportPreviewOverlay.vue` dùng cho khối loại-chìm của nó). */
.aip-piece-source_segment {
  background: var(--color-surface-sunken);
  color: var(--color-on-surface-variant);
}

/* `tm` (P9): dành cho Epic 7 -- không một `prompt` nào story 4.7 lắp tạo ra mảnh `tm` mang văn
   bản (xem `ai_rag_contract.rs`'s `pieces_tag_the_source_sentence_with_its_own_kind_separate_
   from_authored_and_tm_never_produces_a_piece`), nhưng quy tắc phải có mặt NGAY BÂY GIỜ, lúc
   nhãn được giới thiệu -- Epic 7 sẽ là caller thật đầu tiên và không được là story phát hiện ra
   quy tắc còn thiếu. Cùng cặp token `surface-tm`/`tm-rule`/`tm-text` mà `ReadingMode.vue`/
   `AttributionOverlay.vue`/`LookupPanel.vue` đã dùng cho mọi bề mặt liên quan TM trong kho. */
.aip-piece-tm {
  background: var(--color-surface-tm);
}

.aip-markers {
  margin-top: calc(var(--space-unit) * 2);
}

.aip-marker-list {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
}

.aip-marker-list code {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-error);
}

.aip-term-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.aip-term-row {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
  padding: calc(var(--space-unit) * 1) 0;
  border-bottom: 1px solid var(--color-outline);
  font-family: var(--family-read);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.aip-term-row:last-child {
  border-bottom: none;
}

/* Nền phân biệt trực quan mục ĐÃ CHÈN khỏi mục CHỜ CHỐT bị che trong CHÍNH danh sách thuật
   ngữ này — một quy ước trình bày riêng của danh sách, KHÔNG phải quy tắc thoả §Always spec
   4.7 (rà soát finding B1, loop 1: quy tắc đó đã chuyển sang `.aip-piece-glossary`, áp cho
   thân `.aip-prompt-text` — bề mặt §Always thật sự đặt tên). */
.aip-term-list-injected .aip-term-row {
  background: var(--color-surface-accent);
}

.aip-term-arrow {
  color: var(--color-on-surface-variant);
}

.aip-term-tier {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-primary);
  border: 1px solid var(--color-outline);
  padding: 0 calc(var(--space-unit) * 1);
}

.aip-term-reason {
  flex-basis: 100%;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}
</style>
