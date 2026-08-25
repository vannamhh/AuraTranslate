<script setup lang="ts">
// Dải "Thêm thuật ngữ" — Story 3.3, FR48.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 DẢI, KHÔNG PANEL — chèn ở CHÂN WORKSPACE, ngay trên `<StatusBar />` (App.vue:260)
// ─────────────────────────────────────────────────────────────────────────────
// §Design Notes của spec: "vì sao dải ở chân workspace chứ không phải 'dải mọc' của Story
// 3.6" — cơ chế *chỉ một dải mọc tại một thời điểm* là chủ của 3.6; dựng sớm ở đây thì 3.6
// phải dựng lại. Một dải MỘT THỂ HIỆN ở chân cửa sổ dùng được từ cả bốn bề mặt vùng chọn,
// kể cả Panel Lookup nơi không có "câu đang sửa" nào để mọc dưới.
//
// Nó KHÔNG đăng ký `FocusOwner` (`commands/index.ts::FOCUS_OWNERS` — sáu mục, dải không
// thêm vào đó, nó không phải panel).
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
import { nextTick, useTemplateRef, watch } from 'vue'
import { dispatch } from './commands'
import { t, tError } from './i18n'
import type { GlossaryCategory, GlossaryTierWire } from './config/glossary'
import {
  quickAddCategory,
  quickAddEffectiveTier,
  quickAddHasLoaded,
  quickAddIsOpen,
  quickAddLookupError,
  quickAddMode,
  quickAddNote,
  quickAddSaveError,
  quickAddSaving,
  quickAddSourceTerm,
  quickAddTierChoice,
  quickAddTranslation,
  quickAddWorkTierAvailable,
  setQuickAddSourceTerm,
} from './glossaryQuickAddState'

/** Bốn phân loại theo phím số (FR46) — khớp `Category::as_str()` phía Rust. */
const CATEGORY_OPTIONS: ReadonlyArray<{
  value: GlossaryCategory
  labelKey: string
  digit: string
}> = [
  { value: 'person', labelKey: 'glossary.quick_add.category_person', digit: '1' },
  { value: 'place', labelKey: 'glossary.quick_add.category_place', digit: '2' },
  { value: 'domain_term', labelKey: 'glossary.quick_add.category_domain_term', digit: '3' },
  { value: 'other', labelKey: 'glossary.quick_add.category_other', digit: '4' },
]

/**
 * `@keydown` nằm NGOÀI luật Kiểm A của `check:commands` (chỉ `@click` mới bị cưỡng chế),
 * nên phím số được xử lý tự do ở đây — miễn nó không đụng ô gõ nào. Gắn trên `<fieldset>`
 * của NHÓM phân loại, không trên tài liệu: một phím số gõ trong ô Bản dịch/Ghi chú (một
 * bản dịch có chứa số) không nằm trong cây con này, nên nó không bao giờ bị bắt nhầm.
 *
 * 🔵 **LỌC PHÍM BỔ TRỢ 2026-08-25 (vòng rà Epic 3) — chép bản vá đã áp cho anh em nó ở
 * [`GlossaryQueueOverlay.vue`] `onKeydown`.** Dải này **không** nuốt bàn phím (quyết định đã
 * ký, `glossaryQuickAddState.ts::openGlossaryQuickAdd`), nên `Mod+1` vẫn bắn lệnh toàn cục
 * `mode.library` — và bản trước của hàm này *cũng* khớp nó vì `event.key` của `⌘1` vẫn là
 * `'1'`. ⇒ MỘT hợp âm làm HAI việc: đổi chế độ, và lặng lẽ ghi đè phân loại của thuật ngữ
 * đang gõ dở. Đây là đường DOM cục bộ, không đi qua `CommandRegistry`, nên không cổng nào
 * thấy nó.
 *
 * ⚠️ `Shift` CỐ Ý không nằm trong danh sách lọc — cùng lý do đã ghi ở anh em nó: nó không đổi
 * `event.key` của digit theo cách còn khớp (bố cục US: `Shift+1` ra `'!'`), nên lọc nó là
 * thêm một dòng không canh gì cả.
 */
function onCategoryKeydown(event: KeyboardEvent): void {
  if (event.ctrlKey || event.metaKey || event.altKey) return

  const found = CATEGORY_OPTIONS.find((opt) => opt.digit === event.key)
  if (found === undefined) return
  event.preventDefault()
  quickAddCategory.value = found.value
}

/**
 * Getter/setter bọc [`setQuickAddSourceTerm`] — cho phép `v-model` trong khi vẫn ép mọi lượt
 * ghi đi qua đúng MỘT cửa (phát lại lượt tra, xem doc-comment `glossaryQuickAddState.ts`).
 */
function onSourceInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) setQuickAddSourceTerm(target.value)
}

/**
 * Tầng hiển thị trên hai nút chọn — GHIM theo mục tìm thấy ở chế độ SỬA (không sửa được,
 * `:disabled`), hoặc lựa chọn của người dùng ở chế độ THÊM.
 */
function onTierChange(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  if (quickAddMode.value === 'edit') return // ghim — không cho đổi.
  quickAddTierChoice.value = target.value as GlossaryTierWire
}

const sourceInput = useTemplateRef<HTMLInputElement>('sourceInput')

/**
 * 🔴 Focus ô nguồn NGAY khi dải mở — AC "không có vùng chọn ⇒ ô nguồn rỗng và nhận focus,
 * không phải một phím chết". `nextTick`: phần tử chỉ vào DOM SAU khi `v-if` chuyển `true`.
 */
watch(quickAddIsOpen, (open) => {
  if (!open) return
  void nextTick(() => {
    sourceInput.value?.focus()
  })
})
</script>

<template>
  <form
    v-if="quickAddIsOpen"
    class="glossary-quick-add"
    @submit.prevent="dispatch('glossary.save_term')"
    @keydown.esc.prevent="dispatch('glossary.close_quick_add')"
  >
    <div class="gqa-row gqa-title-row">
      <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
      <span class="gqa-title">{{
        quickAddMode === 'edit' ? t('glossary.quick_add.title_edit') : t('glossary.quick_add.title_add')
      }}</span>
    </div>

    <div class="gqa-row gqa-fields">
      <label class="gqa-field">
        <span class="gqa-field-label">{{ t('glossary.quick_add.source_label') }}</span>
        <input
          ref="sourceInput"
          type="text"
          class="gqa-input"
          autocomplete="off"
          :disabled="quickAddSaving"
          :value="quickAddSourceTerm"
          @input="onSourceInput"
        />
      </label>

      <label class="gqa-field">
        <span class="gqa-field-label">{{ t('glossary.quick_add.translation_label') }}</span>
        <input
          v-model="quickAddTranslation"
          type="text"
          class="gqa-input"
          autocomplete="off"
          :disabled="quickAddSaving"
        />
      </label>

      <label class="gqa-field">
        <span class="gqa-field-label">{{ t('glossary.quick_add.note_label') }}</span>
        <input
          v-model="quickAddNote"
          type="text"
          class="gqa-input"
          autocomplete="off"
          :disabled="quickAddSaving"
        />
      </label>
    </div>

    <!--
      🔴 `:disabled="quickAddSaving"` trên CẢ HAI `<fieldset>` — Ice bắt 2026-08-20: nút Lưu
      đã `:disabled` khi đang ghi, nhưng `↵` (Enter) tới được `saveGlossaryQuickAdd` qua
      `@submit` bất kể trạng thái nút, và ba ô nhập ở trên cũng gõ được trong lúc một lượt
      ghi đang bay. `<fieldset disabled>` khoá NGUYÊN khối điều khiển con (mọi `<input
      type="radio">` bên trong) bằng một thuộc tính HTML gốc, không phải lặp `:disabled`
      trên từng ô — cùng lúc với chốt tái nhập ở `glossaryQuickAddState.ts::saveGlossaryQuickAdd`.
    -->
    <div class="gqa-row gqa-groups">
      <fieldset class="gqa-group" :disabled="quickAddSaving" @keydown="onCategoryKeydown">
        <legend>{{ t('glossary.quick_add.category_label') }}</legend>
        <label v-for="opt in CATEGORY_OPTIONS" :key="opt.value" class="gqa-radio">
          <input v-model="quickAddCategory" type="radio" name="gqa-category" :value="opt.value" />
          {{ t(opt.labelKey) }}
        </label>
      </fieldset>

      <fieldset class="gqa-group" :disabled="quickAddSaving">
        <legend>{{ t('glossary.quick_add.tier_label') }}</legend>
        <label class="gqa-radio">
          <input
            type="radio"
            name="gqa-tier"
            value="global"
            :checked="quickAddEffectiveTier === 'global'"
            :disabled="quickAddMode === 'edit'"
            @change="onTierChange"
          />
          {{ t('glossary.quick_add.tier_global') }}
        </label>
        <label class="gqa-radio">
          <input
            type="radio"
            name="gqa-tier"
            value="work"
            :checked="quickAddEffectiveTier === 'work'"
            :disabled="quickAddMode === 'edit'"
            @change="onTierChange"
          />
          {{ t('glossary.quick_add.tier_work') }}
        </label>
        <!-- AC: "Tầng Tác phẩm chưa mở được thì lựa chọn đó hiện KÈM LÝ DO, không biến
             mất" — chỉ hiện khi ta THẬT SỰ biết nó không dùng được (`=== false`, không
             `null`/đang chờ), để không nói sai khi đang chưa tra xong. -->
        <p v-if="quickAddWorkTierAvailable === false" class="gqa-tier-reason">
          {{ t('glossary.quick_add.tier_work_unavailable_reason') }}
        </p>
      </fieldset>
    </div>

    <p v-if="quickAddSaveError !== null" class="gqa-status gqa-error" role="alert">
      {{ tError(quickAddSaveError) }}
    </p>
    <!--
      🔵 THÊM 2026-08-20 (lượt rà soát Story 3.3) — lỗi của LƯỢT TRA phải hiện ra, không
      chìm thành một lượt "đang kiểm tra" vĩnh viễn. Đứng NGAY SAU `quickAddSaveError` và
      NGAY TRƯỚC `quickAddSaving`/`quickAddHasLoaded`: một lượt ghi vừa trượt là tin mới
      hơn một lượt tra đã trượt, còn `checking` bên dưới chỉ đúng khi lượt tra vẫn còn cơ
      hội về. Không có nhánh này, `err.glossary.scope_error` và `store.open_failed` đi qua
      dây rồi chết lặng trong `lookup.value` — dải tắt nút Lưu mà không nói vì sao.
    -->
    <p v-else-if="quickAddLookupError !== null" class="gqa-status gqa-error" role="alert">
      {{ tError(quickAddLookupError) }}
    </p>
    <p v-else-if="quickAddSaving" class="gqa-status" role="status">
      {{ t('glossary.quick_add.saving') }}
    </p>
    <p v-else-if="!quickAddHasLoaded" class="gqa-status" role="status">
      {{ t('glossary.quick_add.checking') }}
    </p>

    <div class="gqa-row gqa-actions">
      <button
        type="submit"
        class="gqa-act gqa-act-primary"
        :disabled="quickAddSaving || quickAddMode === 'unknown' || quickAddSourceTerm.trim() === ''"
      >
        {{ t('glossary.quick_add.save') }}
      </button>
      <button type="button" class="gqa-act" @click="dispatch('glossary.close_quick_add')">
        {{ t('glossary.quick_add.cancel') }}
      </button>
    </div>
  </form>
</template>

<style scoped>
/*
 * AC — dải ĐẨY nội dung workspace lên, không phủ lên nó: KHÔNG `position: fixed`, KHÔNG
 * `z-index`, KHÔNG `box-shadow`. `.shell` (App.vue) là flex column cao đúng `100vh`;
 * `.modeport` là `flex: 1` — một khối `flex: none` chèn giữa nó và `<StatusBar />` chỉ đơn
 * giản CHIẾM CHỖ trong luồng bình thường, đúng cách `.status`/`.config-error` đã làm.
 *
 * Màu và cỡ chữ CHỈ qua token (`check:tokens` Kiểm B/B2/F) — không giá trị viết thẳng.
 */
.glossary-quick-add {
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

.gqa-title {
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.gqa-row {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 4);
}

.gqa-field {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  flex: 1;
  min-width: 8rem;
}

.gqa-field-label,
.gqa-group legend {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gqa-input {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
}

.gqa-group {
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 2);
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
}

.gqa-radio {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.gqa-tier-reason {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gqa-status {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.gqa-error {
  color: var(--color-error);
}

.gqa-actions {
  gap: calc(var(--space-unit) * 2);
}

.gqa-act {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  cursor: pointer;
}

.gqa-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.gqa-act-primary {
  color: var(--color-on-surface);
  border-color: var(--color-primary);
}
</style>
