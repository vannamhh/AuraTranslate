<script setup lang="ts">
// Lớp phủ **Cài đặt** — Story 6.8 (NFR19, AD-41), lớp phủ THỨ CHÍN.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT LỚP PHỦ, KHÔNG MỘT CHẾ ĐỘ THỨ TƯ — khuôn `AttributionOverlay.vue` (§Quyết định #4a)
// ─────────────────────────────────────────────────────────────────────────────
// AD-24 khai BA chế độ ngang hàng, `MODE_IDS` là hằng ba phần tử; đây là một lớp phủ dựng ở
// `App.vue`, cùng tầng với tám lớp phủ đã có (`AttributionOverlay`…`ImportPreviewOverlay`).
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MƯỜI MỘT MỤC NAV — ĐÚNG MỘT CÓ THÂN, MƯỜI CÒN LẠI LUÔN HIỆN KÈM TÊN CHỦ
// ─────────────────────────────────────────────────────────────────────────────
// Chỉ `privacy` (Quyền riêng tư — nhật ký domain, AD-41) có thân ở story này. Mười mục còn
// lại KHÔNG bị `v-if` giấu — chúng hiện, và thân của chúng là một câu nói RÕ vì sao rỗng kèm
// tên chủ, khuôn `tier_empty_story_6_9` (`ImportPreviewOverlay.vue`, Story 6.3/6.9).
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 NAV DÙNG `<form>+submit`, KHÔNG `@click` TRẦN
// ─────────────────────────────────────────────────────────────────────────────
// `check:commands` Kiểm A đòi MỌI `@click` là ĐÚNG MỘT `dispatch('<id>')` — 11 mục tĩnh
// không cần một command riêng cho mỗi mục (cùng lý lẽ `lookup.toggle_source` của Story 1.19:
// "danh sách BIẾT TRƯỚC lúc dựng màn phím tắt không cần một command sinh động"). Mỗi mục vì
// thế là một `<form>` một nút `type="submit"` + `@submit.prevent` — khuôn
// `ImportPreviewOverlay.vue::onStartEditCleanupRule`/`onDeleteCleanupRule`, VẪN bấm được
// bằng bàn phím (Enter/Space kích hoạt `submit` của nút đang focus, NFR17), KHÔNG như
// `@mousedown` một mình (chỉ bắt chuột).
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
// Không `v-html` (AD-16) — domain là DỮ LIỆU văn bản thô, không markup.
import { nextTick, useTemplateRef, watch } from 'vue'
import {
  AI_CONFIG_FIELDS,
  aiConfigDraft,
  aiConfigFieldWire,
  aiConfigIsSaving,
  aiConfigKeyBusy,
  aiConfigKeyConfigured,
  aiConfigKeyDraft,
  aiConfigKeyError,
  aiConfigLoadError,
  aiConfigLoading,
  aiConfigSaveErrorFor,
  clearAiConfigOverride,
  deleteAiConfigKey,
  isAiConfigKeyValueValid,
  isAiConfigValueValid,
  saveAiConfigField,
  saveAiConfigKey,
  setAiConfigDraft,
  setAiConfigKeyDraft,
} from './aiConfigState'
import type { AiConfigField } from './config/aiconfig'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import {
  SETTINGS_SECTIONS,
  domainLogKindLabelKey,
  domainLogOutcomeLabelKey,
  domainLogReasonKey,
  selectSettingsSection,
  settingsActiveSection,
  settingsDomainLogError,
  settingsDomainLogIsEmpty,
  settingsDomainLogLoading,
  settingsDomainLogRows,
  settingsOverlayIsOpen,
  settingsSectionHasBody,
  settingsSectionLabelKey,
  settingsSectionOwnerLabel,
} from './settingsState'
import type { SettingsSection } from './settingsState'

// Bảng chứa domain người dùng tự dán (DỮ LIỆU, không markup) — vai `'display'`, cùng lý do
// Bẫy 8 của `AttributionOverlay.vue`: đọc kỹ một dòng nhật ký mà phát ra một lượt tra cứu là
// thay chính đoạn đang đọc dưới tay người đọc.
const panel = useTemplateRef<HTMLElement>('panel')
useSelectionSurface(panel, 'display')

/** Trả tiêu điểm về chỗ cũ khi đóng (UX-DR17) — khuôn NGUYÊN VĂN `AttributionOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

watch(settingsOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-settings-open]')
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-settings-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[settings] focus-return target is gone; focus falls back to body.')
})

/** Điểm dừng Tab thật, theo đúng thứ tự tài liệu — khuôn NGUYÊN VĂN `AttributionOverlay.vue`. */
function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
}

/** Bẫy tiêu điểm — `Tab` xoay vòng TRONG lớp phủ, khuôn NGUYÊN VĂN `AttributionOverlay.vue`. */
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

function onSelectSection(section: SettingsSection): void {
  selectSettingsSection(section)
}

/** Định dạng thời điểm một bản ghi — DỮ LIỆU (giờ:phút:giây cục bộ của máy), không một
 * chuỗi giao diện dịch được (Consistency Conventions: "định dạng hiển thị chỉ ở frontend"). */
function formatCallTime(atEpochMs: number): string {
  return new Date(atEpochMs).toLocaleTimeString()
}

// ═══════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 4.2 — "AI VÀ MÔ HÌNH" (FR68), mục THỨ NHẤT của lớp phủ có thân thật
// ═══════════════════════════════════════════════════════════════════════════════
//
// Năm ô, mỗi ô một `<form>` riêng cho Lưu — `<form>` KHÔNG lồng nhau (HTML cấm): nút
// "Trả về kế thừa" (chỉ hiện khi trường đang ghi đè ở tầng Tác phẩm) là một `<form>` THỨ HAI
// đứng CẠNH, không bên trong. Cùng khuôn `ImportPreviewOverlay.vue::onStartEditCleanupRule`/
// `onDeleteCleanupRule`: handler cục bộ nhận THAM SỐ (`field`), không một `dispatch('<id>')`
// — `check:commands` Kiểm A chỉ canh `@click`, không canh `@submit` (`src/AGENTS.md`).
function aiConfigFieldLabelKey(field: AiConfigField): string {
  return `settings.ai_config.field_${field}`
}

function aiConfigFieldInvalidKey(field: AiConfigField): string {
  return `settings.ai_config.invalid_${field}`
}

/**
 * Bốn trạng thái, không hai — vòng rà bắt được ca `shadowed === null` khi Work ghi đè một
 * trường mà Global CHƯA có gì (câu cũ nội suy `{value}` từ `shadowed ?? ''` ra một câu cụt
 * "...đang là "), và ca một trường CHƯA cấu hình ở tầng nào cũng không được đọc là "Kế thừa
 * Toàn cục" (không có gì để kế thừa). `wire.value === ''` là tín hiệu tin được cho "chưa cấu
 * hình ở đâu cả": mọi validator (Rust lẫn TS) từ chối chuỗi rỗng sau khi trim cho cả năm
 * trường, nên một tầng đã lưu THẬT không bao giờ mang `value === ''`.
 */
function aiConfigStatusKey(field: AiConfigField): string {
  const wire = aiConfigFieldWire(field)
  if (wire === null || wire.value === '') return 'settings.ai_config.status_unset'
  if (wire.tier === 'work') {
    return wire.shadowed === null
      ? 'settings.ai_config.status_overridden_no_global'
      : 'settings.ai_config.status_overridden'
  }
  return 'settings.ai_config.status_inherited'
}

/** `true` cho cả hai trạng thái GHI ĐÈ — dùng để tô `.mark-over` thay vì lặp lại điều kiện
 * `tier === 'work'` ở hai chỗ (nhãn CHỮ và màu là hai lớp, UX-DR42/DR27). */
function aiConfigIsOverridden(field: AiConfigField): boolean {
  return aiConfigFieldWire(field)?.tier === 'work'
}

function onAiConfigInput(field: AiConfigField, event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) setAiConfigDraft(field, target.value)
}

function onSaveAiConfigField(field: AiConfigField): void {
  void saveAiConfigField(field)
}

function onClearAiConfigOverride(field: AiConfigField): void {
  void clearAiConfigOverride(field)
}

/**
 * Ba trạng thái, không hai — I/O Matrix spec 4.3: `null` (keychain không trả lời thăm dò) là
 * một trạng thái RIÊNG, không được vẽ như `false` ("chưa cấu hình"). Khác `aiConfigStatusKey`
 * ở trên: khoá API không có khái niệm tầng để phân biệt ghi đè/kế thừa.
 */
function aiConfigKeyStatusKey(): string {
  const configured = aiConfigKeyConfigured.value
  if (configured === null) return 'settings.ai_config.key_status_unknown'
  return configured ? 'settings.ai_config.key_status_configured' : 'settings.ai_config.key_status_not_configured'
}

function onAiConfigKeyInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) setAiConfigKeyDraft(target.value)
}

function onSaveAiConfigKey(): void {
  void saveAiConfigKey()
}

function onDeleteAiConfigKey(): void {
  void deleteAiConfigKey()
}
</script>

<template>
  <div
    v-if="settingsOverlayIsOpen"
    class="set-scrim"
    @keydown.esc="dispatch('settings.close')"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="set-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="set-head">
        <h2 class="set-title">{{ t('settings.title') }}</h2>
        <button type="button" class="set-close" @click="dispatch('settings.close')">
          {{ t('command.settings.close') }}
        </button>
      </header>

      <div class="set-grid">
        <nav class="set-nav" aria-label="settings-nav">
          <form
            v-for="section in SETTINGS_SECTIONS"
            :key="section"
            class="set-nav-form"
            @submit.prevent="onSelectSection(section)"
          >
            <button
              type="submit"
              class="set-nav-item"
              :class="{ 'set-nav-item-on': settingsActiveSection === section }"
              :aria-current="settingsActiveSection === section ? 'true' : undefined"
            >
              {{ t(settingsSectionLabelKey(section)) }}
            </button>
          </form>
        </nav>

        <div class="set-main">
          <template v-if="settingsActiveSection === 'ai_and_model'">
            <!-- ═══════════════════ AI và mô hình (FR68, Story 4.2) ═══════════════════ -->
            <h3 class="set-h2">{{ t('settings.nav.ai_and_model') }}</h3>
            <p class="set-h2s">{{ t('settings.ai_config.intro') }}</p>

            <p v-if="aiConfigLoadError !== null" class="set-empty" role="alert">
              <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
              {{ tError(aiConfigLoadError) }}
            </p>
            <p v-else-if="aiConfigLoading" class="set-empty" role="status">
              {{ t('settings.ai_config.loading') }}
            </p>
            <template v-else>
              <div v-for="field in AI_CONFIG_FIELDS" :key="field" class="ai-field">
                <!-- Nút Lưu là `type="submit"` của FORM NÀY — form đi qua ĐÚNG MỘT handler,
                     cùng khuôn `GlossarySettingsOverlay.vue`. -->
                <form class="ai-field-form" @submit.prevent="onSaveAiConfigField(field)">
                  <label class="ai-field-label">
                    <span>{{ t(aiConfigFieldLabelKey(field)) }}</span>
                    <input
                      class="ai-field-input"
                      autocomplete="off"
                      :value="aiConfigDraft(field)"
                      :disabled="aiConfigIsSaving(field)"
                      @input="onAiConfigInput(field, $event)"
                    />
                  </label>

                  <p class="ai-field-status">
                    <span :class="aiConfigIsOverridden(field) ? 'mark-over' : 'mark-inh'">
                      {{
                        t(aiConfigStatusKey(field), {
                          value: aiConfigFieldWire(field)?.shadowed ?? '',
                        })
                      }}
                    </span>
                  </p>

                  <!-- "Chưa cấu hình" KHÔNG phải một lỗi (quy tắc chung của Epic 4) — một ô
                       trống lúc mới mở mục không được hiện câu từ chối; chỉ hiện SAU khi
                       người dùng đã gõ gì đó mà giá trị đó không hợp lệ. -->
                  <p
                    v-if="aiConfigDraft(field) !== '' && !isAiConfigValueValid(field, aiConfigDraft(field))"
                    class="ai-field-alert"
                  >
                    {{ t(aiConfigFieldInvalidKey(field)) }}
                  </p>
                  <p v-else-if="aiConfigSaveErrorFor(field) !== null" class="ai-field-alert" role="alert">
                    <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
                    {{ tError(aiConfigSaveErrorFor(field)!) }}
                  </p>

                  <button
                    type="submit"
                    class="ai-field-save"
                    :disabled="!isAiConfigValueValid(field, aiConfigDraft(field)) || aiConfigIsSaving(field)"
                  >
                    {{ t('settings.ai_config.save') }}
                  </button>
                </form>

                <!-- `<form>` THỨ HAI, ĐỨNG CẠNH — `<form>` không lồng nhau được trong HTML.
                     Chỉ hiện khi trường ĐANG ghi đè ở tầng Tác phẩm. -->
                <form
                  v-if="aiConfigFieldWire(field)?.tier === 'work'"
                  class="ai-field-clear-form"
                  @submit.prevent="onClearAiConfigOverride(field)"
                >
                  <button type="submit" class="ai-field-clear" :disabled="aiConfigIsSaving(field)">
                    {{ t('settings.ai_config.clear_override') }}
                  </button>
                </form>
              </div>

              <!-- ═══════════════════ Khoá API (FR65/FR67, NFR11, Story 4.3) ═══════════════════
                   Không đi qua `v-for="field in AI_CONFIG_FIELDS"` ở trên — khoá không có hình
                   dạng `AiConfigFieldWire` (không tầng, không `shadowed`, giá trị KHÔNG BAO GIỜ
                   trở lại qua IPC). Ba trạng thái hiển thị, không hai: xem
                   `aiConfigKeyStatusKey()`. Lớp `ai-key-field` chỉ để test nhắm đúng khối này
                   giữa nhiều `.ai-field`/`<form>` khác trên cùng màn — không đụng CSS. -->
              <div class="ai-field ai-key-field">
                <form class="ai-field-form" @submit.prevent="onSaveAiConfigKey">
                  <label class="ai-field-label">
                    <span>{{ t('settings.ai_config.key_label') }}</span>
                    <input
                      class="ai-field-input"
                      type="password"
                      autocomplete="off"
                      :value="aiConfigKeyDraft()"
                      :disabled="aiConfigKeyBusy"
                      @input="onAiConfigKeyInput($event)"
                    />
                  </label>

                  <p class="ai-field-status">
                    <span
                      :class="{
                        'mark-over': aiConfigKeyConfigured === true,
                        'mark-inh': aiConfigKeyConfigured === false,
                        'mark-unknown': aiConfigKeyConfigured === null,
                      }"
                    >
                      {{ t(aiConfigKeyStatusKey()) }}
                    </span>
                  </p>
                  <!-- Câu nói ra vì sao mục này KHÔNG có nút chọn tầng như năm ô trên —
                       Quyết định 2026-09-17 (spec 4.3): một khoá cho toàn ứng dụng. -->
                  <p class="ai-field-status">{{ t('settings.ai_config.key_note') }}</p>

                  <!-- Cùng luật năm ô trên: "chưa cấu hình" không phải lỗi, chỉ hiện câu từ
                       chối SAU khi người dùng đã gõ gì đó không hợp lệ. -->
                  <p
                    v-if="aiConfigKeyDraft() !== '' && !isAiConfigKeyValueValid(aiConfigKeyDraft())"
                    class="ai-field-alert"
                  >
                    {{ t('settings.ai_config.key_invalid') }}
                  </p>
                  <p v-else-if="aiConfigKeyError !== null" class="ai-field-alert" role="alert">
                    <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
                    {{ tError(aiConfigKeyError) }}
                  </p>

                  <button
                    type="submit"
                    class="ai-field-save"
                    :disabled="!isAiConfigKeyValueValid(aiConfigKeyDraft()) || aiConfigKeyBusy"
                  >
                    {{ t('settings.ai_config.key_save') }}
                  </button>
                </form>

                <!-- `<form>` THỨ HAI, ĐỨNG CẠNH — hiện trừ lúc đã biết CHẮC chưa có khoá nào
                     (`=== false`). Một thăm dò trượt (`null`) không nói gì về việc Xoá có
                     trượt hay không, nên nút vẫn hiện — chỉ khi gọi thật mới biết. -->
                <form
                  v-if="aiConfigKeyConfigured !== false"
                  class="ai-field-clear-form"
                  @submit.prevent="onDeleteAiConfigKey"
                >
                  <button type="submit" class="ai-field-clear" :disabled="aiConfigKeyBusy">
                    {{ t('settings.ai_config.key_delete') }}
                  </button>
                </form>
              </div>
            </template>
          </template>
          <template v-else-if="settingsSectionHasBody(settingsActiveSection)">
            <!-- ═══════════════════ Quyền riêng tư — nhật ký domain (AD-41, NFR19) ═══════════════════ -->
            <h3 class="set-h2">{{ t('settings.privacy.title') }}</h3>
            <p class="set-h2s">{{ t('settings.privacy.intro') }}</p>

            <!--
              🔴 LỖI ĐỨNG TRƯỚC RỖNG (khuôn `AttributionOverlay.vue`, Ice chốt 2026-08-10):
              một lỗi đọc THẬT và "chưa gọi mạng lần nào" là hai nguyên nhân khác nhau, phải
              nói hai câu khác nhau.
            -->
            <p v-if="settingsDomainLogError !== null" class="set-empty" role="alert">
              <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
              {{ tError(settingsDomainLogError) }}
            </p>
            <p v-else-if="settingsDomainLogLoading" class="set-empty" role="status">
              {{ t('settings.privacy.loading') }}
            </p>
            <!--
              🔴 I/O Matrix spec 6.8 — "Rỗng im lặng là lớp lỗi trung tâm": một bảng trắng
              không được phép tự nói dối là "đang tải" hay im lặng. Câu này NÓI RA nhật ký
              sống theo phiên chạy — khởi động lại app là mất nó, không phải một lỗi.
            -->
            <p v-else-if="settingsDomainLogIsEmpty" class="set-empty" role="status">
              {{ t('settings.privacy.empty') }}
            </p>
            <template v-else>
              <table class="set-table">
                <thead>
                  <tr>
                    <th>{{ t('settings.privacy.col_time') }}</th>
                    <th>{{ t('settings.privacy.col_domain') }}</th>
                    <th>{{ t('settings.privacy.col_kind') }}</th>
                    <th>{{ t('settings.privacy.col_reason') }}</th>
                    <th>{{ t('settings.privacy.col_outcome') }}</th>
                    <th>{{ t('settings.privacy.col_result') }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="row in settingsDomainLogRows"
                    :key="`${row.domain} ${row.kind} ${row.tier} ${row.outcome ?? 'null'}`"
                  >
                    <!-- aura-allow-text: thời điểm định dạng cục bộ — DỮ LIỆU, không một chuỗi giao diện. -->
                    <td class="set-mono">{{ formatCallTime(row.firstAtEpochMs) }}</td>
                    <!-- aura-allow-text: domain — DỮ LIỆU (URL người dùng tự dán). -->
                    <td class="set-mono">{{ row.domain }}</td>
                    <td>
                      <span class="set-tag">{{ t(domainLogKindLabelKey(row.kind)) }}</span>
                    </td>
                    <td>{{ t(domainLogReasonKey(row.tier)) }}</td>
                    <!--
                      Story 6.11, mục A (vòng rà đối kháng 3 lớp) — cột MỚI, tách khỏi "Vì sao
                      được phép" (`tier`): một chặng ĐÃ được phép còn có thể tải xong hay
                      trượt mạng/MIME/quá cỡ, đây là câu trả lời cho câu hỏi đó.
                    -->
                    <td>{{ t(domainLogOutcomeLabelKey(row.outcome)) }}</td>
                    <td>{{ t('settings.privacy.col_result_count', { count: String(row.count) }) }}</td>
                  </tr>
                </tbody>
              </table>
              <p class="set-note">{{ t('settings.privacy.session_only_note') }}</p>
            </template>
          </template>
          <!--
            🔴 MƯỜI MỤC CHƯA CÓ THÂN — LUÔN HIỆN, LUÔN NÓI VÌ SAO KÈM TÊN CHỦ (khuôn
            `tier_empty_story_6_9`). Không `v-if` giấu mục nào khỏi nav (đã hiện ở trên); đây
            chỉ là nội dung PHẦN THÂN của mục đang chọn.
          -->
          <template v-else>
            <h3 class="set-h2">{{ t(settingsSectionLabelKey(settingsActiveSection)) }}</h3>
            <p class="set-tier-empty-reason">
              {{ t('settings.nav.no_body_yet', { owner: settingsSectionOwnerLabel(settingsActiveSection) }) }}
            </p>
          </template>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.set-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: xếp lớp CƠ HỌC — cùng lý do các lớp phủ khác (dockview dựng ngữ cảnh xếp lớp riêng cho mỗi nhóm panel). */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.set-panel {
  width: 100%;
  max-width: 920px;
  max-height: 100%;
  overflow: auto;
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  display: flex;
  flex-direction: column;
}

.set-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: calc(var(--space-unit) * 3) calc(var(--space-unit) * 4);
  border-bottom: 1px solid var(--color-outline);
}

.set-title {
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.set-close {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
}

.set-grid {
  display: grid;
  grid-template-columns: 200px 1fr;
  min-height: 0;
}

.set-nav {
  border-right: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 2) 0;
}

.set-nav-form {
  margin: 0;
}

.set-nav-item {
  display: block;
  width: 100%;
  text-align: left;
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 3);
  background: none;
  border: none;
  border-left: 2px solid transparent;
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.set-nav-item-on {
  color: var(--color-primary);
  border-left-color: var(--color-primary);
}

.set-main {
  padding: calc(var(--space-unit) * 4);
  min-width: 0;
}

.set-h2 {
  margin: 0 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.set-h2s {
  margin: 0 0 calc(var(--space-unit) * 3) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.set-empty {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.set-table {
  width: 100%;
  border-collapse: collapse;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
}

.set-table th {
  text-align: left;
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2) calc(var(--space-unit) * 1) 0;
  border-bottom: 1px solid var(--color-outline);
  color: var(--color-on-surface-variant);
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
}

.set-table td {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2) calc(var(--space-unit) * 1) 0;
  border-bottom: 1px solid var(--color-outline);
  color: var(--color-on-surface);
}

.set-mono {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
}

/* 🔴 Nhãn TẦNG bằng CHỮ, không màu phân loại (UX-DR42, `check:tokens` Kiểm D). */
.set-tag {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
  border: 1px solid var(--color-outline);
  padding: 0 calc(var(--space-unit) * 1);
}

.set-note {
  margin: calc(var(--space-unit) * 2) 0 0 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.set-tier-empty-reason {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-field {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  padding: calc(var(--space-unit) * 2) 0;
  border-bottom: 1px solid var(--color-outline);
}

.ai-field:first-child {
  padding-top: 0;
}

.ai-field-form {
  display: flex;
  flex-wrap: wrap;
  align-items: end;
  gap: var(--space-panel-inline);
}

.ai-field-label {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  flex: 1 1 260px;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
}

.ai-field-input {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 1.5);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
}

.ai-field-save,
.ai-field-clear {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: none;
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.ai-field-clear-form {
  margin: 0;
}

.ai-field-status {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/* 🔴 Nhãn ghi đè/kế thừa bằng CHỮ, không màu một mình (UX-DR42/DR27) — màu ở đây chỉ là lớp
   thứ hai, cùng khuôn `.set-tag`. */
.mark-over {
  color: var(--color-primary);
}

.mark-inh {
  color: var(--color-on-surface-variant);
}

/* Trạng thái THỨ BA của khoá API — keychain từ chối trả lời thăm dò (I/O Matrix spec 4.3).
   Cùng tông màu một lỗi retryable khác trong mục này (`.ai-field-alert`), vì đây đúng là một
   sự cố tạm thời, không phải "chưa cấu hình" (đó là `.mark-inh`). */
.mark-unknown {
  color: var(--color-error);
}

.ai-field-alert {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-error);
}
</style>
