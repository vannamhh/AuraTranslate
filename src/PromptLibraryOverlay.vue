<script setup lang="ts">
// Lớp phủ **Thư viện prompt** — Story 4.4 (FR69), lớp phủ THỨ MƯỜI MỘT.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT LỚP PHỦ, KHÔNG MỘT CHẾ ĐỘ THỨ TƯ — khuôn `SettingsOverlay.vue`/`AttributionOverlay.vue`
// ─────────────────────────────────────────────────────────────────────────────
// §Never của spec 4.4: "Do not add a fourth mode. A big screen is the 11th overlay
// (`src/SettingsOverlay.vue:4-8`)." AD-24 khai BA chế độ ngang hàng; đây là một lớp phủ dựng
// ở `App.vue`, cùng tầng với mười lớp phủ đã có.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỖI THAO TÁC CÓ THAM SỐ ĐI QUA `<form>+submit`, KHÔNG `@click` TRẦN
// ─────────────────────────────────────────────────────────────────────────────
// `check:commands` Kiểm A đòi MỌI `@click` là ĐÚNG MỘT `dispatch('<id>')` — chọn một hàng,
// tạo/đổi tên/sửa/xoá một bộ đều cần THAM SỐ (hàng nào, tầng nào), nên mỗi thao tác đó là một
// `<form>` một nút `type="submit"` + `@submit.prevent`, khuôn NGUYÊN VĂN `SettingsOverlay.vue`
// (nav 11 mục) và `ImportPreviewOverlay.vue::onStartEditCleanupRule`. Chỉ Đóng lớp phủ là
// `@click="dispatch('prompt.library.close')"` — parameterless, đúng luật Kiểm A.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔵 DỮ LIỆU SỐNG Ở `promptSetState.ts` (Phase 4a) — TỆP NÀY CHỈ TIÊU THỤ
// ─────────────────────────────────────────────────────────────────────────────
// Lựa chọn hàng đang xem/sửa, hai ô soạn (tên/thân), và trạng thái "đang tạo mới"/"chờ xác
// nhận xoá" là state CỤC BỘ của MÀN HÌNH — không sống trong `promptSetState.ts`, vì tệp đó là
// tầng dữ liệu hai tầng đã phân giải, không phải tầng UI của một màn soạn cụ thể (khác
// `aiConfigState.ts`, nơi có ĐÚNG NĂM trường cố định nên draft-theo-trường hợp lý ở tầng dữ
// liệu — ở đây tập bộ dài tuỳ ý, không có một tập khoá cố định).
//
// Không chuỗi tiếng Việt nào trong `.vue` (NFR16, AD-21) — mọi văn bản qua `t()`/`tError()`.
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { useSelectionSurface } from './panels/selectionContract'
import { promptLibraryOverlayIsOpen } from './promptLibraryState'
import {
  clearPromptSetActionFeedback,
  createPromptSet,
  deletePromptSet,
  isPromptSetNameValid,
  promptSetActionError,
  promptSetActionWarnings,
  promptSetBusy,
  promptSetLoadError,
  promptSetLoading,
  promptSets,
  promptSetVariables,
  promptSetWorkTierAvailable,
  renamePromptSet,
  selectedPromptSetName,
  setSelectedPromptSetName,
  updatePromptSetBody,
} from './promptSetState'
import type { PromptSetTier, PromptSetWire } from './config/promptset'

const panel = useTemplateRef<HTMLElement>('panel')
// Lớp phủ chứa chữ thật (tên bộ, thân prompt) nhưng không phải nguồn từ điển — vai `'display'`,
// cùng lý do `GlossaryManageOverlay.vue`/`SettingsOverlay.vue`.
useSelectionSurface(panel, 'display')

/** 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Khuôn chép từ `SettingsOverlay.vue`. */
let returnFocusTo: HTMLElement | null = null

watch(promptLibraryOverlayIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-prompt-library-open]')
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-prompt-library-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  console.warn('[prompt-library] focus-return target is gone; focus falls back to body.')
})

/** Điểm dừng Tab thật, theo đúng thứ tự tài liệu — khuôn chép từ `SettingsOverlay.vue`. */
function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
}

/** Bẫy tiêu điểm — `Tab` xoay vòng TRONG lớp phủ, khuôn chép từ `SettingsOverlay.vue`. */
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

// ═══════════════════════════════════════════════════════════════════════════════
// 🔴 BA BIẾN SỐ RATIFY, ĐỌC TỪ MỘT NƠI DUY NHẤT PHÍA RUST — Quyết định #2 spec 4.4
// ═══════════════════════════════════════════════════════════════════════════════
//
// Phase 4c đóng khoảng hở Phase 4b để lại: `core::promptset::vars::PromptVariable::ALL`
// (`src-tauri/src/core/promptset/vars.rs`) giờ đi trên dây qua `PromptSetListWire::variables`
// (`commands/promptset.rs`, gói vào phong bì `prompt_set_list` ĐÃ CÓ — không một command thứ
// sáu, Phase 2 đã khoá đúng năm wire). `promptSetVariables` bên dưới là state đã nạp từ đó
// (`promptSetState.ts`) — KHÔNG một mảng gõ tay ở tệp này. Khoá i18n của mỗi mô tả suy ra
// theo QUY ƯỚC [`varDescKey`] (`prompt.library.var_<name>_desc`), không một bảng tra tên thứ
// hai keyed lại đúng ba chuỗi đó.
function varDescKey(name: string): string {
  return `prompt.library.var_${name}_desc`
}

// ═══════════════════════════════════════════════════════════════════════════════
// Hàng đang xem/sửa — state CỤC BỘ (xem khối 🔵 đầu tệp)
// ═══════════════════════════════════════════════════════════════════════════════

const selectedTier = ref<PromptSetTier | null>(null)
const selectedId = ref<number | null>(null)
const nameDraft = ref('')
const bodyDraft = ref('')
const deletePending = ref(false)

const createOpen = ref(false)
const createTier = ref<PromptSetTier>('global')
const createNameDraft = ref('')
const createBodyDraft = ref('')
/** `true` khi lượt Tạo GẦN NHẤT thành công nhưng bộ vừa tạo là một bộ Global cùng tên với một
 * bộ Work đang mở — `resolve_two_tiers` gộp cặp đó thành MỘT hàng mang `tier: 'work'` (Quyết
 * định #1), nên `(tier, name)` không còn khớp hàng nào để mà chọn. Không suy `id` của hàng
 * Global vừa tạo (nợ có chủ của Story 4.5) — chỉ báo cho người dùng biết lượt ghi đã thành
 * công nhưng chưa hiện hiệu lực, thay vì đóng form vào một màn hình không đổi gì mà không giải
 * thích. */
const createShadowedNote = ref(false)

/** Hàng đang chọn, đọc TRỰC TIẾP từ `promptSets` hiện tại — tự cập nhật sau mỗi lượt nạp lại
 * (đổi tên/sửa thân không đổi `id`), và tự về `null` nếu hàng vừa bị xoá ở nơi khác. */
const selectedRow = computed<PromptSetWire | null>(() => {
  if (selectedTier.value === null || selectedId.value === null) return null
  return promptSets.value.find((s) => s.tier === selectedTier.value && s.id === selectedId.value) ?? null
})

function isCurrent(row: { tier: PromptSetTier; id: number | null }): boolean {
  return row.id !== null && selectedTier.value === row.tier && selectedId.value === row.id
}

/** Một hàng hiển thị — SỐ THẬT (`id !== null`) hoặc bản HIỂN THỊ của một bộ Global bị che
 * (`id === null`, Quyết định #1: "the shadowed global stays visible in the list"). Bản hiển
 * thị không có `id` riêng — `ResolvedPromptSet.shadowed_body` phía Rust không mang `id` của
 * hàng Global gốc (Phase 2's own note) — nên nó không chọn được, không sửa/xoá/đổi tên được. */
type DisplayRow = {
  key: string
  tier: PromptSetTier
  id: number | null
  name: string
  body: string
  shadowsGlobal: boolean
  isShadowedDisplay: boolean
}

const workRows = computed<DisplayRow[]>(() =>
  promptSets.value
    .filter((s) => s.tier === 'work')
    .map((s) => ({
      key: `work-${s.id}`,
      tier: 'work' as const,
      id: s.id,
      name: s.name,
      body: s.body,
      shadowsGlobal: s.shadowed_body !== null,
      isShadowedDisplay: false,
    })),
)

const globalRows = computed<DisplayRow[]>(() => {
  const rows: DisplayRow[] = []
  for (const s of promptSets.value) {
    if (s.tier === 'global') {
      rows.push({
        key: `global-${s.id}`,
        tier: 'global',
        id: s.id,
        name: s.name,
        body: s.body,
        shadowsGlobal: false,
        isShadowedDisplay: false,
      })
      continue
    }
    if (s.shadowed_body !== null) {
      rows.push({
        key: `shadow-${s.name}`,
        tier: 'global',
        id: null,
        name: s.name,
        body: s.shadowed_body,
        shadowsGlobal: false,
        isShadowedDisplay: true,
      })
    }
  }
  return rows
})

function selectRow(row: DisplayRow): void {
  if (row.id === null) return // bản hiển thị của một bộ bị che — không chọn được
  selectedTier.value = row.tier
  selectedId.value = row.id
  nameDraft.value = row.name
  bodyDraft.value = row.body
  deletePending.value = false
  createOpen.value = false
  clearPromptSetActionFeedback() // banner lỗi/cảnh báo của bộ TRƯỚC không được sống sót sang bộ này
}

function clearSelection(): void {
  selectedTier.value = null
  selectedId.value = null
  deletePending.value = false
}

function onNameInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) nameDraft.value = target.value
}

function onBodyInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLTextAreaElement) bodyDraft.value = target.value
}

function onOpenCreate(): void {
  createOpen.value = true
  createTier.value = promptSetWorkTierAvailable.value ? 'work' : 'global'
  createNameDraft.value = ''
  createBodyDraft.value = ''
  createShadowedNote.value = false
  clearSelection()
  clearPromptSetActionFeedback() // banner lỗi/cảnh báo của bộ ĐANG chọn không được sống sót sang form Tạo
}

function onCancelCreate(): void {
  createOpen.value = false
}

function onCreateNameInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) createNameDraft.value = target.value
}

function onCreateBodyInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLTextAreaElement) createBodyDraft.value = target.value
}

function onCreateTierChange(tier: PromptSetTier, event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement) || !target.checked) return
  createTier.value = tier
}

/** Tạo xong ⇒ CHỌN NGAY bộ vừa tạo (tìm lại theo `(tier, name)` sau khi danh sách nạp lại —
 * `createPromptSet` không trả `id`, chỉ trả `Promise<void>`, cùng khuôn `promptSetState.ts`).
 *
 * ⚠️ Một bộ Global vừa tạo có thể KHÔNG khớp `(tier, name)` nào trong `promptSets` — khi một bộ
 * Work cùng tên đang mở che nó, `resolve_two_tiers` gộp cặp đó thành MỘT hàng mang
 * `tier: 'work'` (Quyết định #1). Lượt ghi đã thành công (không một `IpcError` nào), nhưng
 * không có hàng `tier: 'global'` nào để mà chọn — im lặng đóng form lúc đó để lại một màn hình
 * không đổi gì mà không giải thích. `createShadowedNote` giữ form MỞ và báo rõ tình huống, thay
 * vì suy `id` của hàng bị che hay lọc theo tầng (cả hai đều ngoài phạm vi bản vá này — nợ có
 * chủ của Story 4.5). */
async function onSubmitCreate(): Promise<void> {
  const tier = createTier.value
  const name = createNameDraft.value.trim()
  await createPromptSet(tier, createNameDraft.value, createBodyDraft.value)
  if (promptSetActionError.value !== null) return // ở lại form Tạo để hiện lỗi

  const created = promptSets.value.find((s) => s.tier === tier && s.name === name)
  if (created === undefined) {
    createShadowedNote.value = true
    return
  }

  createOpen.value = false
  createShadowedNote.value = false
  selectedTier.value = tier
  selectedId.value = created.id
  nameDraft.value = created.name
  bodyDraft.value = created.body
}

async function onSubmitRename(): Promise<void> {
  if (selectedTier.value === null || selectedId.value === null) return
  deletePending.value = false // đổi tên là một Ý ĐỊNH KHÁC xoá — nhịp chờ xoá không được sống sót qua nó
  await renamePromptSet(selectedTier.value, selectedId.value, nameDraft.value)
}

async function onSubmitBody(): Promise<void> {
  if (selectedTier.value === null || selectedId.value === null) return
  deletePending.value = false // cùng lý do onSubmitRename — sửa thân không phải xác nhận xoá
  await updatePromptSetBody(selectedTier.value, selectedId.value, bodyDraft.value)
}

/** Xoá hai nhịp — khuôn Cụm D `GlossaryManageOverlay.vue`: nhịp MỘT chỉ đặt cờ chờ, nhịp HAI
 * (cùng nút, đã đổi chữ) mới gọi IPC thật. */
async function onDeleteSubmit(): Promise<void> {
  if (selectedTier.value === null || selectedId.value === null) return
  if (!deletePending.value) {
    deletePending.value = true
    return
  }
  deletePending.value = false
  const tier = selectedTier.value
  const id = selectedId.value
  await deletePromptSet(tier, id)
  if (promptSetActionError.value === null) clearSelection()
}

/** "Dùng bộ này" — Quyết định 🔵 đầu `promptSetState.ts`: 0 lượt `invoke`, chỉ đổi lựa chọn
 * HIỂN THỊ cục bộ. Đây là nửa "từ màn Thư viện prompt" của I/O Matrix "Switch effective set" —
 * nửa "từ AI panel, không cần mở Cài đặt" đóng ở `AiTranslationPanel.vue`. */
/**
 * Toàn văn dấu ngoặc của một biến (`"{{glossary_terms}}"`) — hàm RIÊNG, không một template
 * literal viết thẳng trong `<template>`: `{{ \`{{${x}}}\` }}` làm tokenizer của Vue đọc cặp
 * `}}` ĐẦU TIÊN bên trong chuỗi thành dấu ĐÓNG của chính mustache đang mở, vỡ cú pháp SFC
 * (đo được: `vue-tsc`/`vite build` từ chối biên dịch tệp, không phải một lỗi runtime).
 */
function marker(name: string): string {
  return `{{${name}}}`
}

function onUseSelected(): void {
  if (selectedRow.value === null) return
  setSelectedPromptSetName(selectedRow.value.name)
}

/**
 * `Escape` khi đang chờ xác nhận xoá HUỶ nhịp đó; khi form Tạo đang mở thì đóng form đó;
 * ngoài hai ca đó mới đóng cả lớp phủ — cùng luật "Escape huỷ nhịp một trước" của
 * `GlossaryManageOverlay.vue::onEscape`.
 */
function onEscape(): void {
  if (deletePending.value) {
    deletePending.value = false
    return
  }
  if (createOpen.value) {
    createOpen.value = false
    return
  }
  dispatch('prompt.library.close')
}
</script>

<template>
  <div
    v-if="promptLibraryOverlayIsOpen"
    class="pl-scrim"
    @keydown.esc="onEscape"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="pl-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="pl-head">
        <h2 class="pl-title">{{ t('prompt.library.title') }}</h2>
        <button type="button" class="pl-close" @click="dispatch('prompt.library.close')">
          {{ t('command.prompt.library.close') }}
        </button>
      </header>

      <p v-if="!promptSetWorkTierAvailable" class="pl-note">
        {{ t('prompt.library.work_tier_unavailable_note') }}
      </p>

      <p v-if="promptSetLoadError !== null" class="pl-empty" role="alert">
        <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
        {{ tError(promptSetLoadError) }}
      </p>
      <p v-else-if="promptSetLoading" class="pl-empty" role="status">{{ t('prompt.library.loading') }}</p>

      <!--
        🔴 KHÔNG chặn cột "Bộ mới" khi `promptSets.length === 0` — I/O Matrix "List with zero
        sets anywhere ... Empty state saying nothing is configured" nói RỖNG phải NÓI RÕ vì
        sao rỗng, không nói nó phải CẤM tạo bộ đầu tiên. Câu rỗng đứng NGAY TRONG `.pl-list`,
        cột soạn vẫn còn nút "Bộ mới" — nếu không, một kho chưa ai tạo gì thì không có đường
        nào tạo được bộ ĐẦU TIÊN từ chính màn hình này.
      -->
      <div v-else class="pl-cols">
        <nav class="pl-list" aria-label="prompt-library-list">
          <p v-if="promptSets.length === 0" class="pl-group-empty">{{ t('prompt.library.empty') }}</p>
          <h3 class="pl-group-h">{{ t('prompt.library.group_work') }}</h3>
          <p v-if="workRows.length === 0 && promptSets.length > 0" class="pl-group-empty">{{ t('prompt.library.group_work_empty') }}</p>
          <form v-for="row in workRows" :key="row.key" class="pl-row-form" @submit.prevent="selectRow(row)">
            <button type="submit" class="pl-row" :class="{ 'pl-row-on': isCurrent(row) }" :aria-current="isCurrent(row) ? 'true' : undefined">
              <!-- aura-allow-text: DỮ LIỆU (tên bộ do người dùng đặt). -->
              <span class="pl-row-name">{{ row.name }}</span>
              <span v-if="row.name === selectedPromptSetName" class="pl-badge">{{ t('prompt.library.in_use_badge') }}</span>
              <span v-if="row.shadowsGlobal" class="pl-badge pl-badge-info">{{ t('prompt.library.shadows_badge') }}</span>
            </button>
          </form>

          <h3 class="pl-group-h">{{ t('prompt.library.group_global') }}</h3>
          <p v-if="globalRows.length === 0 && promptSets.length > 0" class="pl-group-empty">{{ t('prompt.library.group_global_empty') }}</p>
          <template v-for="row in globalRows" :key="row.key">
            <form v-if="!row.isShadowedDisplay" class="pl-row-form" @submit.prevent="selectRow(row)">
              <button type="submit" class="pl-row" :class="{ 'pl-row-on': isCurrent(row) }" :aria-current="isCurrent(row) ? 'true' : undefined">
                <!-- aura-allow-text: DỮ LIỆU (tên bộ do người dùng đặt). -->
                <span class="pl-row-name">{{ row.name }}</span>
                <span v-if="row.name === selectedPromptSetName" class="pl-badge">{{ t('prompt.library.in_use_badge') }}</span>
              </button>
            </form>
            <!-- Bản HIỂN THỊ của một bộ Global bị che — không một `<form>`/`<button>`: nó không
                 chọn được (không `id`), nên nó không thuộc thứ tự Tab (Quyết định #1). -->
            <div v-else class="pl-row pl-row-shadowed">
              <!-- aura-allow-text: DỮ LIỆU (tên bộ do người dùng đặt). -->
              <span class="pl-row-name">{{ row.name }}</span>
              <span class="pl-badge pl-badge-shadowed">{{ t('prompt.library.shadowed_badge') }}</span>
            </div>
          </template>

          <form class="pl-row-form" @submit.prevent="onOpenCreate">
            <button type="submit" class="pl-row pl-row-new">{{ t('prompt.library.new_set') }}</button>
          </form>
        </nav>

        <div class="pl-ed">
          <template v-if="createOpen">
            <h3 class="pl-h2">{{ t('prompt.library.create_title') }}</h3>

            <form class="pl-create-form" @submit.prevent="onSubmitCreate">
              <fieldset class="pl-tier" role="radiogroup" :aria-label="t('prompt.library.tier_label')">
                <legend class="pl-field-label">{{ t('prompt.library.tier_label') }}</legend>
                <label class="pl-radio-label">
                  <input type="radio" name="pl-create-tier" :checked="createTier === 'global'" @change="onCreateTierChange('global', $event)" />
                  {{ t('prompt.library.tier_global') }}
                </label>
                <label class="pl-radio-label">
                  <input
                    type="radio"
                    name="pl-create-tier"
                    :disabled="!promptSetWorkTierAvailable"
                    :checked="createTier === 'work'"
                    @change="onCreateTierChange('work', $event)"
                  />
                  {{ t('prompt.library.tier_work') }}
                </label>
              </fieldset>

              <label class="pl-field">
                <span class="pl-field-label">{{ t('prompt.library.name_label') }}</span>
                <input class="pl-input" autocomplete="off" :value="createNameDraft" :disabled="promptSetBusy" @input="onCreateNameInput" />
              </label>
              <p v-if="createNameDraft !== '' && !isPromptSetNameValid(createNameDraft)" class="pl-alert">
                {{ t('err.prompt_set.invalid_name') }}
              </p>

              <label class="pl-field">
                <span class="pl-field-label">{{ t('prompt.library.body_label') }}</span>
                <textarea class="pl-textarea" rows="10" :value="createBodyDraft" :disabled="promptSetBusy" @input="onCreateBodyInput"></textarea>
              </label>

              <p v-if="promptSetActionError !== null" class="pl-alert" role="alert">
                <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
                {{ tError(promptSetActionError) }}
              </p>
              <p v-if="createShadowedNote" class="pl-note" role="status">
                {{ t('prompt.library.create_shadowed_note') }}
              </p>

              <div class="pl-actions">
                <button type="submit" class="pl-act pl-act-primary" :disabled="!isPromptSetNameValid(createNameDraft) || promptSetBusy">
                  {{ t('prompt.library.create_button') }}
                </button>
              </div>
            </form>
            <!-- `<form>` THỨ HAI, ĐỨNG CẠNH — `<form>` không lồng nhau được trong HTML, khuôn
                 `SettingsOverlay.vue::ai-field-clear-form`. -->
            <form class="pl-actions" @submit.prevent="onCancelCreate">
              <button type="submit" class="pl-act" :disabled="promptSetBusy">
                {{ t('prompt.library.create_cancel') }}
              </button>
            </form>
          </template>

          <template v-else-if="selectedRow !== null">
            <!-- aura-allow-text: DỮ LIỆU (tên bộ do người dùng đặt). -->
            <h3 class="pl-h2">{{ selectedRow.name }}</h3>
            <p class="pl-h2s">
              {{ t(selectedRow.tier === 'work' ? 'prompt.library.editing_work_note' : 'prompt.library.editing_global_note') }}
            </p>

            <form class="pl-rename-form" @submit.prevent="onSubmitRename">
              <label class="pl-field">
                <span class="pl-field-label">{{ t('prompt.library.name_label') }}</span>
                <input class="pl-input" autocomplete="off" :value="nameDraft" :disabled="promptSetBusy" @input="onNameInput" />
              </label>
              <p v-if="nameDraft !== '' && !isPromptSetNameValid(nameDraft)" class="pl-alert">
                {{ t('err.prompt_set.invalid_name') }}
              </p>
              <button type="submit" class="pl-act" :disabled="!isPromptSetNameValid(nameDraft) || promptSetBusy">
                {{ t('prompt.library.rename_button') }}
              </button>
            </form>

            <h4 class="pl-sh">{{ t('prompt.library.vars_heading') }}</h4>
            <div class="pl-vars">
              <div v-for="v in promptSetVariables" :key="v" class="pl-vrow">
                <!-- aura-allow-text: định danh máy đọc (tên biến), không phải chữ giao diện. -->
                <code>{{ marker(v) }}</code>
                <span class="pl-vd">{{ t(varDescKey(v)) }}</span>
              </div>
            </div>

            <form class="pl-body-form" @submit.prevent="onSubmitBody">
              <label class="pl-field">
                <span class="pl-field-label">{{ t('prompt.library.body_label') }}</span>
                <textarea class="pl-textarea" rows="12" :value="bodyDraft" :disabled="promptSetBusy" @input="onBodyInput"></textarea>
              </label>

              <p v-if="promptSetActionWarnings !== null && promptSetActionWarnings.unknown_markers.length > 0" class="pl-warn">
                {{ t('prompt.library.warning_unknown_markers', { tokens: promptSetActionWarnings.unknown_markers.join(', ') }) }}
              </p>
              <p v-if="promptSetActionWarnings !== null && promptSetActionWarnings.glossary_terms_missing" class="pl-warn">
                {{ t('prompt.library.warning_glossary_terms_missing') }}
              </p>
              <p v-if="promptSetActionError !== null" class="pl-alert" role="alert">
                <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
                {{ tError(promptSetActionError) }}
              </p>

              <button type="submit" class="pl-act pl-act-primary" :disabled="promptSetBusy">
                {{ t('prompt.library.save_body_button') }}
              </button>
            </form>

            <div class="pl-actions">
              <form class="pl-use-form" @submit.prevent="onUseSelected">
                <button type="submit" class="pl-act" :disabled="selectedRow.name === selectedPromptSetName">
                  {{ t('prompt.library.use_button') }}
                </button>
              </form>
              <form class="pl-delete-form" @submit.prevent="onDeleteSubmit">
                <button type="submit" class="pl-act" :class="{ 'pl-act-danger': deletePending }" :disabled="promptSetBusy">
                  {{ t(deletePending ? 'prompt.library.delete_confirm_button' : 'prompt.library.delete_button') }}
                </button>
              </form>
            </div>
            <p v-if="deletePending" class="pl-status pl-alert" role="status">{{ t('prompt.library.delete_confirm_hint') }}</p>
          </template>

          <p v-else class="pl-hint">{{ t('prompt.library.select_hint') }}</p>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.pl-scrim {
  position: fixed;
  inset: 0;
  z-index: 10; /* aura-allow-z-index: xếp lớp CƠ HỌC — cùng lý do các lớp phủ khác. */
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.pl-panel {
  width: 100%;
  max-width: 960px;
  max-height: 100%;
  overflow: auto;
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  display: flex;
  flex-direction: column;
  padding: var(--space-panel-inline);
}

.pl-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.pl-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.pl-close {
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

.pl-note {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pl-empty {
  margin: 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.pl-cols {
  display: grid;
  grid-template-columns: 260px 1fr;
  gap: var(--space-panel-inline);
  min-height: 0;
}

.pl-list {
  border-right: 1px solid var(--color-outline);
  padding-right: var(--space-panel-inline);
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  overflow: auto;
}

.pl-group-h {
  margin: calc(var(--space-unit) * 3) 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
}

.pl-group-h:first-child {
  margin-top: 0;
}

.pl-group-empty {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.pl-row-form {
  margin: 0;
}

.pl-row {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
  width: 100%;
  text-align: left;
  padding: calc(var(--space-unit) * 2) calc(var(--space-unit) * 2);
  background: none;
  border: none;
  border-left: 2px solid transparent;
  cursor: pointer;
  font-family: var(--face-read);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.pl-row-on {
  color: var(--color-primary);
  border-left-color: var(--color-primary);
  background: var(--color-surface-accent);
}

.pl-row-new {
  font-family: var(--face-ui-md);
  color: var(--color-on-surface-variant);
}

.pl-row-shadowed .pl-row-name {
  color: var(--color-on-surface-variant);
}

.pl-badge {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-primary);
  border: 1px solid var(--color-outline);
  padding: 0 calc(var(--space-unit) * 1);
}

.pl-badge-info {
  color: var(--color-on-surface-variant);
}

.pl-badge-shadowed {
  color: var(--color-on-surface-variant);
  border-color: var(--color-error);
}

.pl-ed {
  min-width: 0;
}

.pl-h2 {
  margin: 0 0 calc(var(--space-unit) * 1) 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  color: var(--color-on-surface);
}

.pl-h2s {
  margin: 0 0 calc(var(--space-unit) * 3) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pl-sh {
  font-size: var(--font-ui-label);
  font-family: var(--face-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
  margin: calc(var(--space-unit) * 3) 0 calc(var(--space-unit) * 2) 0;
  padding-bottom: calc(var(--space-unit) * 1);
  border-bottom: 1px solid var(--color-outline);
}

.pl-vars {
  margin-bottom: calc(var(--space-unit) * 3);
}

.pl-vrow {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 3);
  padding: calc(var(--space-unit) * 1) 0;
  border-bottom: 1px solid var(--color-outline);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
}

.pl-vrow:last-child {
  border-bottom: none;
}

.pl-vrow code {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-primary);
  min-width: 200px;
}

.pl-vd {
  color: var(--color-on-surface-variant);
  flex: 1;
}

.pl-field {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin-bottom: calc(var(--space-unit) * 2);
}

.pl-field-label {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
}

.pl-input {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 1.5);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
}

.pl-textarea {
  padding: calc(var(--space-unit) * 1.5);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  font-family: var(--face-read);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  resize: vertical;
}

.pl-tier {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 3);
  margin: 0 0 calc(var(--space-unit) * 3) 0;
  padding: 0;
  border: none;
}

.pl-radio-label {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
  cursor: pointer;
}

.pl-alert {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-error);
}

.pl-warn {
  margin: 0 0 calc(var(--space-unit) * 2) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.pl-status {
  margin: calc(var(--space-unit) * 2) 0 0 0;
}

.pl-actions {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
  margin-top: calc(var(--space-unit) * 2);
}

.pl-act {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
  background: none;
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  cursor: pointer;
}

.pl-act:disabled {
  cursor: default;
  color: var(--color-on-surface-variant);
}

.pl-act-primary {
  color: var(--color-on-surface);
  border-color: var(--color-primary);
}

.pl-act-danger {
  color: var(--color-error);
  border-color: var(--color-error);
}

.pl-hint {
  margin: 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}
</style>
