<script setup lang="ts">
// Lớp phủ **Cài đặt ngưỡng quét Glossary** — Story 3.5, FR47, lớp phủ THỨ TƯ.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT LỚP PHỦ, KHÔNG MỘT CHẾ ĐỘ THỨ TƯ — khuôn `ShortcutsOverlay.vue`/`AttributionOverlay.vue`
// ─────────────────────────────────────────────────────────────────────────────
// AD-24 khai BA chế độ ngang hàng, `MODE_IDS` là hằng ba phần tử. Dựng ở `App.vue`, cùng
// tầng với hai lớp phủ kia — nó nói về CẢ ứng dụng, không về một panel.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 KHÔNG THANH CHUYỂN PHẠM VI — xem doc-comment đầu `glossarySettingsState.ts`
// ─────────────────────────────────────────────────────────────────────────────
// Ngưỡng là `AppConfig` ⇒ `GlobalOnly` (`core/scope/kinds.rs:218`); `save_value` phía Rust
// từ chối mọi loại khác. Một nút "Tác phẩm" bấm được sẽ trông như đã ghi mà không ghi.
import { nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import {
  glossarySettingsOverlayIsOpen,
  glossarySettingsSaveError,
  glossarySettingsSaving,
  glossarySettingsThresholdInput,
  parsedGlossaryScanThreshold,
} from './glossarySettingsState'

/** 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Khuôn và lý lẽ chép từ `AttributionOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

const panel = useTemplateRef<HTMLElement>('panel')
// Lớp phủ chứa chữ thật nhưng không phải nguồn từ điển. Đăng ký `display` để một vùng
// chọn trong modal không phát Auto-Lookup rồi thay nội dung phía sau modal.
useSelectionSurface(panel, 'display')

watch(glossarySettingsOverlayIsOpen, (open) => {
  if (open) {
    // 🔴 KHÔNG lưu `document.activeElement` trần — xem `focusReturnTargetOnOpen`.
    returnFocusTo = focusReturnTargetOnOpen('[data-glossary-settings-open]')
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

  const opener = document.querySelector<HTMLElement>('[data-glossary-settings-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  // Chẩn đoán tiếng Anh — Kiểm A của `check:i18n` cấm chuỗi tiếng Việt ở vị trí mã.
  console.warn('[glossary-settings] focus-return target is gone; focus falls back to body.')
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

function onInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) glossarySettingsThresholdInput.value = target.value
}
</script>

<template>
  <!--
    ⚠️ `@keydown.esc` là DOM thường, KHÔNG một command — cùng lý do `ShortcutsOverlay.vue`:
    `Escape` ở đây là một lượt huỷ TRONG NGỮ CẢNH, không một thao tác toàn cục.
    Nút Huỷ/Lưu ĐI QUA command — Kiểm A đòi mỗi `@click` là đúng một `dispatch('<id>')`.
  -->
  <div
    v-if="glossarySettingsOverlayIsOpen"
    class="gs-scrim"
    @keydown.esc="dispatch('glossary.settings.close')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="gs-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="gs-head">
        <h2 class="gs-title">{{ t('glossary.settings.title') }}</h2>
        <button type="button" class="gs-close" @click="dispatch('glossary.settings.close')">
          {{ t('command.glossary.settings.close') }}
        </button>
      </header>

      <p class="gs-intro">{{ t('glossary.settings.intro') }}</p>

      <!-- Nút Lưu là `type="submit"` — form đi qua ĐÚNG MỘT `dispatch`, cùng khuôn
           `GlossaryQuickAdd.vue`. Không `@click` thứ hai trên nút: hai đường cùng gọi một
           command là dựng một đường dispatch thứ hai mà Kiểm A không nhìn thấy. -->
      <form class="gs-form" @submit.prevent="dispatch('glossary.settings.save')">
        <!-- 🔴 Đúng MỘT ô nhập số — §Boundaries/§Never của story: 0 thanh chuyển phạm vi. -->
        <fieldset class="gs-fieldset" :disabled="glossarySettingsSaving">
          <label class="gs-field">
            <span class="gs-field-label">{{ t('glossary.settings.threshold_label') }}</span>
            <input
              type="number"
              class="gs-input"
              min="1"
              step="1"
              autocomplete="off"
              :value="glossarySettingsThresholdInput"
              @input="onInput"
            />
          </label>

          <p v-if="parsedGlossaryScanThreshold(glossarySettingsThresholdInput) === null" class="gs-alert">
            {{ t('glossary.settings.threshold_invalid') }}
          </p>
          <p v-else-if="glossarySettingsSaveError !== null" class="gs-alert">
            {{ tError(glossarySettingsSaveError) }}
          </p>

          <div class="gs-actions">
            <button
              type="submit"
              class="gs-save"
              :disabled="parsedGlossaryScanThreshold(glossarySettingsThresholdInput) === null"
            >
              {{ t('command.glossary.settings.save') }}
            </button>
            <button type="button" class="gs-cancel" @click="dispatch('glossary.settings.close')">
              {{ t('glossary.settings.cancel') }}
            </button>
          </div>
        </fieldset>
      </form>
    </section>
  </div>
</template>

<style scoped>
.gs-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: xếp lớp CƠ HỌC — cùng lý do ba lớp phủ kia (dockview dựng ngữ cảnh xếp lớp riêng cho mỗi nhóm panel). */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.gs-panel {
  width: 100%;
  max-width: 560px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.gs-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.gs-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.gs-close {
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

.gs-intro {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.gs-form {
  display: block;
}

.gs-fieldset {
  margin: 0;
  padding: 0;
  border: none;
}

.gs-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: var(--space-panel-block);
}

.gs-field-label {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.gs-input {
  width: 8em;
  padding: 4px 6px;
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
}

/*
 * Câu từ chối — UX-DR27: nói ra bằng CHỮ, không bằng màu một mình; màu ở đây chỉ là lớp
 * thứ hai, cùng khuôn `.sc-alert` của `ShortcutsOverlay.vue`.
 */
.gs-alert {
  margin: 0 0 var(--space-panel-block) 0;
  padding-left: 11px;
  border-left: 2px solid var(--color-error);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-error);
}

.gs-actions {
  display: flex;
  gap: var(--space-panel-inline);
}

.gs-save {
  padding: 4px 12px;
  border: 1px solid var(--color-outline);
  background: none;
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.gs-cancel {
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
</style>
