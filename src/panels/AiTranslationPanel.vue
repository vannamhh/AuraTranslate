<script setup lang="ts">
// Panel `Đề xuất AI`. Story 1.14 · AC1 · AC8 — **khung**, không phải nội dung.
//
// Bản dịch AI thật, chọn nhà cung cấp, và ba điểm ra mạng của AD-15 là **Epic 4**.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 "CHƯA CẤU HÌNH" KHÔNG PHẢI MỘT TRẠNG THÁI LỖI — UX-DR27 · FR77
// ─────────────────────────────────────────────────────────────────────────────────
// Panel này **MỜI CẤU HÌNH**. Không cảnh báo, không màu `error`, không dấu chấm
// than. Một người dùng chưa từng dán khoá API vào đâu thì không làm sai gì cả — vẽ
// một cảnh báo ở đây là dạy họ rằng ứng dụng đang hỏng.
//
// ⚠️ Câu trạng thái sống ở `vi.json` (`panel.ai_translation.status`) và Kiểm D của
// `check-i18n.mjs` chấm phần máy chấm được của UX-DR47 (không "chúng tôi", không
// "bạn"). Phần còn lại — giọng MỜI thay vì giọng CẢNH BÁO — là chỗ con người phải đọc.
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 1.18 · AC2 — ĐĂNG KÝ HỢP ĐỒNG VÙNG CHỌN, KHÔNG NỘI DUNG
// ─────────────────────────────────────────────────────────────────────────────────
// Panel này hôm nay **không có chữ**, và đó chính là lý do lượt đăng ký phải nằm ở đây NGAY
// BÂY GIỜ: `epics.md:1762` đòi AI Translation *"nhận được cùng hành vi khi nó có nội dung
// ở các epic sau, **không cần cài lại**"*. Một lượt đăng ký thiếu ở đây không để lại **bất
// kỳ triệu chứng nào** cho tới Epic 4 — tức hai epic sau, và tới lúc đó không ai nhớ AC này
// tồn tại. Cổng đếm của `check-commands.mjs` (Kiểm F) là thứ giữ mệnh đề đó bằng MÁY.
//
// Đừng "dọn" `<div ref="surface">` vì nó trông trống: nó LÀ bề mặt mà Epic 4 sẽ đổ nội
// dung vào, và là phần tử mà hợp đồng đo `contains(anchorNode)` trên.
//
// 🔵 **2026-08-13 — mệnh đề "cùng hành vi" ở trên đã ĐƯỢC THU HẸP** (Sprint Change Proposal,
// Ice ký; FR21). Panel này sẽ mang **bản dịch AI tiếng Việt**, còn từ điển nhúng là
// zh→vi / en→vi ⇒ nó KHÔNG phải nguồn tra cứu: vai nay là `'display'`.
// Phần còn đúng của AC2 — và là phần đắt nhất — vẫn nguyên: hợp đồng KHÔNG phải sửa một
// dòng nào khi Epic 4 đổ nội dung vào. Chỉ **vai** khai lúc đăng ký quyết định hành vi.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 4.4 — DÒNG "BỘ HIỆU LỰC" + BỘ CHUYỂN, KHÔNG CẦN MỞ CÀI ĐẶT (FR69)
// ─────────────────────────────────────────────────────────────────────────────────
// I/O Matrix spec 4.4: *"Switch effective set from AI panel — effective set changes
// without opening Settings."* `<select>` dưới đây gọi THẲNG `setSelectedPromptSetName`
// (`promptSetState.ts`) — 0 lượt `invoke`, khuôn Quyết định 🔵 đầu tệp đó. Đây là nửa màn
// hình của mệnh đề I/O Matrix; nửa Rust (`resolve_two_tiers` đổi kết quả khi tầng đổi) đã
// đóng ở Phase 3 (`prompt_set_contract.rs::switching_between_two_resolvable_sets_needs_no_
// settings_reopen`).
import { computed, onMounted, useTemplateRef, watch } from 'vue'
import PanelFrame from './PanelFrame.vue'
import { useSelectionSurface } from './selectionContract'
import { dispatch } from '../commands'
import { t, tError } from '../i18n'
import { loadPromptSets, promptSets, selectedPromptSetName, setSelectedPromptSetName } from '../promptSetState'
import { editorCaretSegmentId } from './editorPanelState'
import {
  aiPromptAssembleBusy,
  aiPromptAssembleError,
  aiPromptRecord,
  aiPromptRecordIsStale,
  clearAiPromptAssembleError,
  glossaryInjectionSummary,
  refreshAiPromptRecord,
} from '../aiPromptInspectorState'
import {
  aiTranslateAccumulatedText,
  aiTranslateError,
  aiTranslateRunSegmentId,
  aiTranslateStateValue,
  isAiTranslateResultStale,
} from '../aiTranslateState'
import type { DockviewPanelProps } from '../layout/panelProps'

defineProps<DockviewPanelProps>()

const surface = useTemplateRef<HTMLElement>('surface')
// 🔴 ĐỪNG gỡ lời gọi này khi thấy vai là `'display'`. FR48 (Story 3.3) và FR60 (Story 7.7)
// đọc vùng chọn ở đây bằng lệnh của RIÊNG chúng; `'display'` tắt đúng MỘT đường —
// `currentSelectionText()`, tức đường tra TỪ ĐIỂN — chứ không tắt việc bề mặt được đăng ký.
// Ghim bằng máy: `check-commands.mjs` Kiểm F ③.
useSelectionSurface(surface, 'display')

// Nạp danh sách bộ prompt hai tầng khi panel dựng — cùng lý do `openGlossaryManage()` nạp
// lại mỗi lần lớp phủ Quản lý mở: bộ có thể vừa được tạo/xoá/đổi tên ở một phiên trước, hoặc
// một Tác phẩm khác vừa mở.
onMounted(() => {
  void loadPromptSets()
  // Story 4.7 — đồng bộ với bản ghi Rust đang giữ ngay lúc panel này mount: dockview có thể
  // tháo/dựng lại panel trong khi phiên vẫn còn một bản ghi từ một lượt Lắp trước đó.
  void refreshAiPromptRecord()
})

/**
 * ⚠️ **SỬA 2026-09-18, bắt được ở lượt rà soát build.** `aiPromptAssembleError` không mang
 * định danh câu — khác `aiPromptRecord` (§Always spec 4.7: bản ghi PHẢI mang định danh câu
 * tạo ra nó). Không có watcher này, một lỗi Lắp cho câu A đứng nguyên trên `.ai-inspector-alert`
 * sau khi tiêu điểm dời sang câu B mà người dùng chưa bấm Lắp lại — đọc như "câu B đang lỗi",
 * dù lỗi đó thuộc về A. Xem doc-comment [`clearAiPromptAssembleError`] cho đối chứng.
 */
watch(editorCaretSegmentId, () => {
  clearAiPromptAssembleError()
})

function onPromptSetSelectChange(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLSelectElement)) return
  setSelectedPromptSetName(target.value === '' ? null : target.value)
}

// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 4.7 — DÒNG TÓM TẮT + NÚT "LẮP PROMPT" + NÚT "XEM PROMPT" (FR71, AD-14)
// ─────────────────────────────────────────────────────────────────────────────────
// `EXPERIENCE.md:388` (KF-2 bước 4): dòng tóm tắt và "Xem prompt" sống ở panel này. Nút Lắp
// là bề mặt riêng của Decision 2 spec 4.7 — nó GHI bản ghi; nút Xem CHỈ mở lớp phủ ĐỌC bản
// ghi đó (`ai.prompt_inspector.open` không bao giờ lắp ráp). `canAssemble` đọc
// `editorCaretSegmentId` — panel này (và mọi bản dịch AI) vốn gắn với ĐÚNG MỘT câu tại một
// thời điểm, và đó là nguồn DUY NHẤT của "câu đang có tiêu điểm" trong toàn kho (dùng lại
// nguyên, không một ô nhớ "câu hiện tại" thứ hai — `GlossaryConfirmStrip.vue`/
// `segmentHistoryState.ts`/`GridPanel.vue` đều đọc CHÍNH ref này).
const glossarySummary = computed(() => glossaryInjectionSummary(aiPromptRecord.value))

const glossarySummaryText = computed<string>(() => {
  const summary = glossarySummary.value
  if (summary.kind === 'no_record') return t('ai.prompt.summary_no_record')
  if (summary.kind === 'not_asked') return t('ai.prompt.summary_not_asked')
  return t('ai.prompt.summary_asked', { count: String(summary.count) })
})

/**
 * 🔴 finding B6 (loop 1) — dòng tóm tắt phải TỰ ĐÁNH DẤU khi bản ghi nó đọc là CŨ (I/O Matrix
 * "Stale record"): trước bản sửa này, chỉ lớp phủ `AiPromptInspectorOverlay.vue` biết bản ghi
 * đang xem là cũ ([`aiPromptRecordIsStale`] chỉ được `import` ở đó) — dòng tóm tắt LUÔN HIỆN
 * ĐƯỢC ngay trên panel (không cần mở lớp phủ) vẫn đọc như "Đã chèn N thuật ngữ" cho câu ĐANG
 * focus dù bản ghi thật ra thuộc một câu khác. Cùng hàm thuần, cùng cách truyền tham số
 * (`editorCaretSegmentId` — panel này đã là leaf đọc ref đó cho mục đích khác, xem
 * `canAssemble`), không một watcher/state thứ hai.
 */
const isRecordStale = computed<boolean>(() => aiPromptRecordIsStale(aiPromptRecord.value, editorCaretSegmentId.value))

/** `false` ⇔ không câu nào đang được chọn — chưa mở Tác phẩm, hoặc đã mở nhưng chưa đặt tiêu
 * điểm vào câu nào. Vô hiệu hoá nút Lắp ở ĐÂY (chỗ BIẾT trước), không dựa vào lưới phòng thủ
 * "kêu, không ném" của `assembleCurrentAiPrompt` — hai lớp, không chỉ một. */
const canAssemble = computed<boolean>(() => editorCaretSegmentId.value !== null)

// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 4.8 — DỊCH THẬT + HUỶ + ĐƯA SANG BẢN DỊCH (FR72/FR74, AD-22, AD-47①/③)
// ─────────────────────────────────────────────────────────────────────────────────
// Cùng khuôn khối Story 4.7 ngay trên: vô hiệu hoá nút ở CHỖ BIẾT TRƯỚC, hai lớp cùng lưới
// "kêu, không ném" của `runAiTranslate`/`cancelAiTranslate`/`promoteAiTranslate` (`main.ts`).
const canRunAiTranslate = computed<boolean>(
  () => editorCaretSegmentId.value !== null && aiTranslateStateValue.value !== 'generating',
)
const canCancelAiTranslate = computed<boolean>(() => aiTranslateStateValue.value === 'generating')
/** I/O Matrix spec 4.8 "Promote the result": `done`/`cancelled` VÀ văn bản không rỗng. */
const canPromoteAiTranslate = computed<boolean>(
  () =>
    (aiTranslateStateValue.value === 'done' || aiTranslateStateValue.value === 'cancelled') &&
    aiTranslateAccumulatedText.value !== '',
)
/** `true` ⇔ kết quả đang hiện được dịch cho một câu KHÁC câu đang có tiêu điểm bây giờ — cùng
 * khuôn [`isRecordStale`] ngay trên, hàm thuần của `aiTranslateState.ts`. */
const isAiTranslateStale = computed<boolean>(() =>
  isAiTranslateResultStale(aiTranslateRunSegmentId.value, editorCaretSegmentId.value),
)
</script>

<template>
  <PanelFrame owner="panel.ai_translation" status-key="panel.ai_translation.status">
    <div class="ai-prompt-bar">
      <p v-if="promptSets.length === 0" class="ai-prompt-empty">{{ t('panel.ai_translation.prompt_set_empty') }}</p>
      <label v-else class="ai-prompt-select-label">
        <span>{{ t('panel.ai_translation.prompt_set_label') }}</span>
        <select class="ai-prompt-select" :value="selectedPromptSetName ?? ''" @change="onPromptSetSelectChange">
          <option value="">{{ t('panel.ai_translation.prompt_set_placeholder') }}</option>
          <!-- aura-allow-text: DỮ LIỆU (tên bộ do người dùng đặt). -->
          <option v-for="s in promptSets" :key="`${s.tier}-${s.id}`" :value="s.name">{{ s.name }}</option>
        </select>
      </label>
      <p class="ai-prompt-status">
        <!-- aura-allow-text: KẾT QUẢ của `t()` (tên bộ nội suy qua tham số). -->
        {{
          selectedPromptSetName === null
            ? t('panel.ai_translation.prompt_set_none')
            : t('panel.ai_translation.prompt_set_using', { name: selectedPromptSetName })
        }}
      </p>
      <button type="button" class="ai-prompt-open" data-prompt-library-open @click="dispatch('prompt.library.open')">
        {{ t('command.prompt.library.open') }}
      </button>
    </div>
    <div class="ai-inspector-bar">
      <!-- aura-allow-text: KẾT QUẢ của t() (ba-giá-trị: chưa lắp / not_asked / asked kèm số đếm). -->
      <p
        class="ai-inspector-summary"
        :data-ai-prompt-summary-kind="glossarySummary.kind"
        :data-ai-prompt-summary-stale="isRecordStale ? 'true' : null"
      >
        {{ glossarySummaryText }}
      </p>
      <!-- finding B6 (loop 1) -- đánh dấu RIÊNG, tách khỏi dòng tóm tắt: cùng khoá
           `ai.prompt_inspector.stale_notice` lớp phủ đã dùng, không đúc một khoá thứ hai cho
           cùng một sự thật. -->
      <p v-if="isRecordStale && aiPromptRecord !== null" class="ai-inspector-stale" role="status" data-ai-prompt-summary-stale-notice>
        <!-- aura-allow-text: KẾT QUẢ của t() (nội suy số câu). -->
        {{
          t('ai.prompt_inspector.stale_notice', {
            record_segment_id: String(aiPromptRecord.segment_id),
            focused_segment_id: String(editorCaretSegmentId ?? ''),
          })
        }}
      </p>
      <p v-if="aiPromptAssembleError !== null" class="ai-inspector-alert" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của tError(). -->
        {{ tError(aiPromptAssembleError) }}
      </p>
      <p v-if="!canAssemble" class="ai-inspector-hint">{{ t('panel.ai_translation.assemble_no_segment_hint') }}</p>
      <div class="ai-inspector-actions">
        <button
          type="button"
          class="ai-inspector-assemble"
          data-ai-prompt-assemble
          :disabled="!canAssemble || aiPromptAssembleBusy"
          @click="dispatch('ai.prompt.assemble')"
        >
          <!-- aura-allow-text: KẾT QUẢ của t() (hai nhãn, chọn theo cờ đang bận). -->
          {{ aiPromptAssembleBusy ? t('panel.ai_translation.assemble_busy') : t('command.ai.prompt.assemble') }}
        </button>
        <button
          type="button"
          class="ai-inspector-open"
          data-ai-prompt-inspector-open
          @click="dispatch('ai.prompt_inspector.open')"
        >
          {{ t('command.ai.prompt_inspector.open') }}
        </button>
      </div>
    </div>
    <div ref="surface" class="ai-surface">
      <div class="ai-translate-actions">
        <button
          type="button"
          class="ai-translate-run"
          data-ai-translate-run
          :disabled="!canRunAiTranslate"
          @click="dispatch('ai.translate.run')"
        >
          {{ t('command.ai.translate.run') }}
        </button>
        <button
          type="button"
          class="ai-translate-cancel"
          data-ai-translate-cancel
          :disabled="!canCancelAiTranslate"
          @click="dispatch('ai.translate.cancel')"
        >
          {{ t('command.ai.translate.cancel') }}
        </button>
        <button
          type="button"
          class="ai-translate-promote"
          data-ai-translate-promote
          :disabled="!canPromoteAiTranslate"
          @click="dispatch('ai.translate.promote')"
        >
          {{ t('command.ai.translate.promote') }}
        </button>
      </div>
      <p v-if="!canAssemble" class="ai-translate-hint">{{ t('panel.ai_translation.translate_no_segment_hint') }}</p>
      <p v-if="aiTranslateStateValue === 'generating'" class="ai-translate-status" data-ai-translate-state="generating">
        {{ t('panel.ai_translation.state_generating') }}
      </p>
      <p
        v-else-if="aiTranslateStateValue === 'cancelled'"
        class="ai-translate-status"
        data-ai-translate-state="cancelled"
      >
        {{ t('panel.ai_translation.state_cancelled') }}
      </p>
      <p
        v-else-if="aiTranslateStateValue === 'not_configured' && aiTranslateRunSegmentId !== null"
        class="ai-translate-status"
        data-ai-translate-state="not_configured"
      >
        {{ t('panel.ai_translation.status') }}
      </p>
      <p v-if="aiTranslateStateValue === 'error' && aiTranslateError !== null" class="ai-translate-alert" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của tError(). -->
        {{ tError(aiTranslateError) }}
      </p>
      <p v-if="isAiTranslateStale && aiTranslateAccumulatedText !== ''" class="ai-translate-stale" role="status" data-ai-translate-stale-notice>
        <!-- aura-allow-text: KẾT QUẢ của t() (nội suy hai số câu). -->
        {{
          t('ai.translate.stale_notice', {
            record_segment_id: String(aiTranslateRunSegmentId),
            focused_segment_id: String(editorCaretSegmentId ?? ''),
          })
        }}
      </p>
      <p v-if="aiTranslateAccumulatedText !== ''" class="ai-translate-text" data-ai-translate-text>
        <!-- aura-allow-text: DỮ LIỆU (văn bản do AI sinh ra, chảy dần qua Channel). -->
        {{ aiTranslateAccumulatedText }}
      </p>
    </div>
  </PanelFrame>
</template>

<style scoped>
/* Story 4.4 — dòng "bộ hiệu lực" + bộ chuyển. `flex: none`: xem `PanelFrame.vue::.panel-body`
   — mọi con KHÔNG-CUỘN của slot phải khai nó tường minh khi slot mang HƠN MỘT con. */
.ai-prompt-bar {
  flex: none;
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin-bottom: var(--space-panel-block);
}

.ai-prompt-empty,
.ai-prompt-status {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-prompt-select-label {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
}

.ai-prompt-select {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 1.5);
}

.ai-prompt-open {
  align-self: flex-start;
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/* Story 4.7 — dòng tóm tắt Glossary + nút Lắp/Xem prompt. */
.ai-inspector-bar {
  flex: none;
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin-bottom: var(--space-panel-block);
  padding-top: calc(var(--space-unit) * 1);
  border-top: 1px solid var(--color-outline);
}

.ai-inspector-summary,
.ai-inspector-hint {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-inspector-alert,
.ai-inspector-stale {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.ai-inspector-actions {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 3);
}

.ai-inspector-assemble,
.ai-inspector-open {
  align-self: flex-start;
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-inspector-assemble:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

/*
 * `flex: 1; min-height: 0` thay `height: 100%` — bắt buộc từ khi `.panel-body` mang HAI con
 * (bar + bề mặt), xem doc-comment `PanelFrame.vue::.panel-body`: `height: 100%` sẽ tự đo
 * theo chiều cao `.panel-body`, cộng dồn với chiều cao `.ai-prompt-bar` mà tràn khỏi panel.
 */
.ai-surface {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  overflow-y: auto;
}

/* Story 4.8 — nút Dịch/Huỷ/Đưa sang bản dịch. */
.ai-translate-actions {
  flex: none;
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 3);
}

.ai-translate-run,
.ai-translate-cancel,
.ai-translate-promote {
  align-self: flex-start;
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-translate-run:disabled,
.ai-translate-cancel:disabled,
.ai-translate-promote:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.ai-translate-hint,
.ai-translate-status {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-translate-alert,
.ai-translate-stale {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

/* Văn bản AI chảy dần — cùng token `editor` mà `GridPanel.vue` dùng cho bản dịch trong Editor
   (DESIGN.md: "Bản dịch trong Editor", 15px/1,95): đây LÀ một bản dịch, chỉ chưa được đưa vào
   Editor. Giữ nguyên xuống dòng của chính nó (AD-16: không `v-html`, đây vẫn là một text node
   thuần, chỉ CSS đổi cách trình bày khoảng trắng). */
.ai-translate-text {
  margin: 0;
  font-family: var(--face-editor);
  font-size: var(--font-editor);
  line-height: var(--leading-editor);
  color: var(--color-on-surface);
  white-space: pre-wrap;
}
</style>
