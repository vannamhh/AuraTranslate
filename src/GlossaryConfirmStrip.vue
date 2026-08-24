<script setup lang="ts">
// Dải "Chờ chốt lần đầu gặp" — Story 3.6, FR114.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 DẢI, KHÔNG PANEL — chèn ở CHÂN WORKSPACE, ngay dưới `<GlossaryQuickAdd />` và ngay
// trên `<StatusBar />` (App.vue) — cùng slot, cùng khuôn `GlossaryQuickAdd.vue:1-13`.
// ─────────────────────────────────────────────────────────────────────────────
// Nó KHÔNG đăng ký `FocusOwner` (`commands/index.ts::FOCUS_OWNERS` — dải không thêm vào
// đó, nó không phải panel — tiền lệ `GlossaryQuickAdd.vue:12-13`).
//
// `v-if` TỰ QUẢN qua `topmostStrip(...) === 'glossary_confirm'` — sổ ưu tiên
// (`panels/inlineStripPriority.ts`) quyết định dải nào THẮNG khi cả `GlossaryQuickAdd`
// (thao tác người dùng VỪA yêu cầu) LẪN dải này cùng đủ điều kiện.
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
import { computed, nextTick, watch } from 'vue'
import { useTemplateRef } from 'vue'
import { dispatch } from './commands'
import { t, tError } from './i18n'
import {
  confirmStripEmptyInputError,
  confirmStripFocusRequest,
  confirmStripIsOpen,
  confirmStripSaveError,
  confirmStripSaving,
  confirmStripSourceTerm,
  confirmStripSuggestionStatus,
  confirmStripTier,
  confirmStripTranslationInput,
  syncGlossaryConfirmStripTarget,
} from './glossaryConfirmStripState'
import { quickAddIsOpen } from './glossaryQuickAddState'
import { editorCaretSegmentId, editorSegments } from './panels/editorPanelState'
import { glossaryMarks, glossaryMarksHaveLoaded } from './panels/glossaryMarksState'
import { topmostStrip } from './panels/inlineStripPriority'
import type { InlineStripKind } from './panels/inlineStripPriority'

/**
 * `watch` THẬT của `editorCaretSegmentId` sống Ở ĐÂY (một LEAF), không trong
 * `glossaryConfirmStripState.ts` — §Design Notes của spec giải thích lý do: một `import`
 * ngược từ tệp state đó lên `editorPanelState.ts` dựng một VÒNG. Component này được PHÉP
 * import cả hai phía.
 *
 * `glossaryMarksHaveLoaded()` là một HÀM (không một `ref`) nên nó không tự kích `watch` —
 * theo dõi luôn `glossaryMarks` (đổi giá trị mỗi lượt nạp/nạp lại/reset) làm nhiệm vụ đó.
 */
watch(
  [editorCaretSegmentId, editorSegments, glossaryMarks],
  ([segmentId, segments, marks]) => {
    syncGlossaryConfirmStripTarget(segmentId, segments, glossaryMarksHaveLoaded(), marks)
  },
  { immediate: true },
)

const eligible = computed<InlineStripKind[]>(() => {
  const list: InlineStripKind[] = []
  if (quickAddIsOpen.value) list.push('glossary_quick_add')
  if (confirmStripIsOpen.value) list.push('glossary_confirm')
  return list
})
const isVisible = computed(() => topmostStrip(eligible.value) === 'glossary_confirm')

/**
 * 🔵 THÊM 2026-08-22 (rà ba lớp) — `aria-describedby` cho ô nhập, trỏ tới đoạn `role="alert"`/
 * `role="status"` ĐANG RENDER (`id="gcs-status-msg"` — đúng MỘT trong ba đoạn luôn render tại
 * một thời điểm, xem `<template>`). Không có liên kết này, người dùng đọc màn hình đứng
 * trong ô nhập không biết có một câu lỗi/trạng thái đang hiện ngay bên dưới.
 */
/**
 * 🔵 THÊM 2026-08-24 (Story 3.7, FR113) — dòng *"chưa cài dữ liệu từ điển"* nay CŨNG chiếm
 * `id="gcs-status-msg"`, cùng chuỗi `v-if`/`v-else-if` với ba đoạn cũ (đúng MỘT đoạn render
 * tại một thời điểm — xem `<template>`). Ưu tiên THẤP NHẤT: lỗi/đang lưu là phản hồi cho
 * THAO TÁC người dùng vừa làm, còn dòng này chỉ mô tả TRẠNG THÁI của dữ liệu nền.
 */
const statusMessageId = computed<string | undefined>(() =>
  confirmStripSaveError.value !== null ||
  confirmStripEmptyInputError.value ||
  confirmStripSaving.value ||
  confirmStripSuggestionStatus.value === 'dict_unavailable'
    ? 'gcs-status-msg'
    : undefined,
)

const translationInputEl = useTemplateRef<HTMLInputElement>('translationInputEl')

/** Focus ô nhập CHỈ khi được yêu cầu tường minh (`glossary.confirm.focus`) — dải KHÔNG cướp
 * tiêu điểm lúc mọc (§Boundaries). */
watch(confirmStripFocusRequest, () => {
  void nextTick(() => {
    translationInputEl.value?.focus()
  })
})
</script>

<template>
  <form
    v-if="isVisible"
    class="glossary-confirm-strip"
    @submit.prevent="dispatch('glossary.confirm.save')"
    @keydown.esc.prevent="dispatch('glossary.confirm.defer')"
  >
    <fieldset class="gcs-fieldset" :disabled="confirmStripSaving">
      <div class="gcs-row gcs-title-row">
        <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
        <span class="gcs-title">{{ t('glossary.confirm.title', { source_term: confirmStripSourceTerm ?? '' }) }}</span>
        <!-- aura-allow-text: KẾT QUẢ của `t()`, cùng khuôn `GlossaryQuickAdd.vue:104-107`. -->
        <span class="gcs-tier">
          {{ confirmStripTier === 'work' ? t('glossary.quick_add.tier_work') : t('glossary.quick_add.tier_global') }}
        </span>
        <!-- aura-allow-text: KẾT QUẢ của `t()`. Story 3.7, FR113: nhãn RIÊNG cho ca đề xuất
             sẵn sàng (`han_viet_status === 'ok'`) -- đúng tiền lệ
             `panel.source.han_viet_unknown`/`han_viet_unavailable` của Story 1.16.
             🔴 SỐNG Ở HÀNG TIÊU ĐỀ, cạnh `.gcs-tier`, KHÔNG bên trong `<label class="gcs-field">`
             — và đó là một ràng buộc KHẢ TRUY CẬP, không phải một lựa chọn bố cục. Tên khả
             truy cập của `<input>` được tính từ TOÀN BỘ nội dung văn bản của `<label>` bọc nó;
             đặt nhãn này vào trong đó khiến trình đọc màn hình đọc ô nhập thành *"Bản dịch Âm
             Hán Việt"* — hai nhãn dính làm một cụm vô nghĩa. Ở đây nó là một chip ĐỘC LẬP,
             đọc được riêng, đúng khuôn `.gcs-tier` ngay trên. *(Vòng rà Bước 4 bắt, 2026-08-24.)* -->
        <span v-if="confirmStripSuggestionStatus === 'ok'" class="gcs-suggestion-label">
          {{ t('glossary.confirm.suggestion_label') }}
        </span>
      </div>

      <label class="gcs-field">
        <span class="gcs-field-label">{{ t('glossary.confirm.translation_label') }}</span>
        <input
          ref="translationInputEl"
          v-model="confirmStripTranslationInput"
          type="text"
          class="gcs-input"
          autocomplete="off"
          :aria-describedby="statusMessageId"
        />
      </label>

      <!--
        🔵 THÊM 2026-08-22 (rà ba lớp) — `id="gcs-status-msg"` CHUNG cho cả ba đoạn: đúng MỘT
        trong ba luôn render (chuỗi `v-if`/`v-else-if`), nên trùng `id` không bao giờ xảy ra
        THẬT trong DOM — ô nhập nối `aria-describedby` tới ĐÚNG đoạn đang hiện mà không cần
        `computed` chọn id theo nhánh.
        🔵 THÊM 2026-08-24 (Story 3.7) — đoạn thứ TƯ, ưu tiên THẤP NHẤT: dòng *"chưa cài dữ
        liệu từ điển"* khi `han_viet_status === 'dict_unavailable'` -- một ô rỗng câm là
        đúng lớp lỗi RỖNG IM LẶNG mà kho cấm (`AGENTS.md:46`).
      -->
      <p v-if="confirmStripSaveError !== null" id="gcs-status-msg" class="gcs-status gcs-error" role="alert">
        {{ tError(confirmStripSaveError) }}
      </p>
      <p v-else-if="confirmStripEmptyInputError" id="gcs-status-msg" class="gcs-status gcs-error" role="alert">
        {{ t('glossary.confirm.empty_error') }}
      </p>
      <p v-else-if="confirmStripSaving" id="gcs-status-msg" class="gcs-status" role="status">
        {{ t('glossary.confirm.saving') }}
      </p>
      <p
        v-else-if="confirmStripSuggestionStatus === 'dict_unavailable'"
        id="gcs-status-msg"
        class="gcs-status"
        role="status"
      >
        {{ t('glossary.confirm.suggestion_unavailable') }}
      </p>

      <div class="gcs-row gcs-actions">
        <button type="submit" class="gcs-act gcs-act-primary">{{ t('glossary.confirm.save') }}</button>
        <button type="button" class="gcs-act" @click="dispatch('glossary.confirm.defer')">
          {{ t('glossary.confirm.defer') }}
        </button>
      </div>
    </fieldset>
  </form>
</template>

<style scoped>
/*
 * AC — dải ĐẨY nội dung workspace lên, không phủ lên nó: KHÔNG `position: fixed`, KHÔNG
 * `z-index`, KHÔNG `box-shadow` — cùng khuôn `GlossaryQuickAdd.vue`. Màu và cỡ chữ CHỈ qua
 * token (`check:tokens` Kiểm B/B2/F).
 */
.glossary-confirm-strip {
  display: flex;
  flex-direction: column;
  flex: none;
  gap: calc(var(--space-unit) * 2);
  padding: var(--space-panel-inline);
  border-top: 1px solid var(--color-outline);
  background: var(--color-surface);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.gcs-fieldset {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
  border: none;
  margin: 0;
  padding: 0;
}

.gcs-row {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: calc(var(--space-unit) * 4);
}

.gcs-title {
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.gcs-tier {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gcs-field {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  flex: 1;
  min-width: 8rem;
}

.gcs-field-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

/* 🔵 THÊM 2026-08-24 (Story 3.7) — nhãn RIÊNG cạnh ô nhập khi có đề xuất Hán Việt. Màu VÀ
 * cỡ chữ chỉ từ token, cùng khuôn `.gcs-tier` — không bóng đổ, không gradient. */
.gcs-suggestion-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-primary);
}

.gcs-input {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
}

.gcs-status {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gcs-error {
  color: var(--color-error);
}

.gcs-actions {
  gap: calc(var(--space-unit) * 2);
}

.gcs-act {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  cursor: pointer;
}

.gcs-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.gcs-act-primary {
  color: var(--color-on-surface);
  border-color: var(--color-primary);
}
</style>
