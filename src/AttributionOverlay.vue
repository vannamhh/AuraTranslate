<script setup lang="ts">
// A screen, not a fourth mode (AD-24): mounted in `App.vue` inside `div.stage` below the
// title bar, not in a panel, because it is about the whole app.
import { computed, nextTick, useTemplateRef, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import {
  aimAttributionSource,
  attributionAimedCode,
  attributionCopyFailedCode,
  attributionIsOpen,
  attributionLicenseView,
  conversionNoticeApplies,
  dictSources,
  dictSourcesError,
  layerKeyFor,
  licenseKeyFor,
  roleLangsFor,
  someDictSourceUnreadable,
  sourceIsDisabled,
} from './panels/dictSourcesState'
import { useSelectionSurface } from './panels/selectionContract'

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 BẪY 8 — BỀ MẶT VĂN BẢN THỨ SÁU, VÀ VAI PHẢI LÀ `'display'`
// ═════════════════════════════════════════════════════════════════════════════════
//
// Bảng này chứa **chữ thật** (ghi công, tên giấy phép), nên nó rơi vào đúng lớp câu hỏi mà
// `useSelectionSurface` tồn tại để trả lời — một bề mặt văn bản im lặng đứng ngoài sổ là
// đúng thứ AC2 của Story 1.18 dựng ra để chặn.
//
// 🔴 Vai `'display'`, **không** `'source'` — cùng lý do Bẫy 1 của Story 1.18 đã bắt ở Panel
// Lookup: bôi đen một dòng ghi công để đọc kỹ mà phát ra một lượt tra cứu là thay chính đoạn
// đang đọc dưới tay người đọc. `SELECTION_SURFACE_FLOOR` nâng lên số thật cùng lượt.
const panel = useTemplateRef<HTMLElement>('panel')
const licenseViewEl = useTemplateRef<HTMLElement>('licenseViewEl')
useSelectionSurface(panel, 'display')

const aimedSource = computed(() => dictSources.value.find((s) => s.code === attributionAimedCode.value))
const canOpenLicense = computed(() => (aimedSource.value?.license_text_len ?? 0) > 0)

/**
 * 🔴 UX-DR17 — TRẢ TIÊU ĐIỂM VỀ CHỖ CŨ.
 *
 * Phần tử đang có tiêu điểm **trước** lượt mở. Một lớp phủ nuốt tiêu điểm rồi trả nó về
 * `body` là để người dùng bàn phím rơi ra đầu tài liệu sau mỗi lần đóng — NFR17 vỡ ở đúng
 * chỗ không cổng nào nhìn thấy (`check-commands.mjs` §giới hạn 2 ghi rõ: vế DOM của AC4
 * không kiểm được ở cổng).
 *
 * ⚠️ Một biến module-scope của **component**, không một `ref`: không một bề mặt nào render
 * nó, và làm nó phản ứng chỉ thêm một lượt cập nhật thừa.
 */
let returnFocusTo: HTMLElement | null = null

watch(attributionIsOpen, (open) => {
  if (open) {
    // 🔴 KHÔNG lưu `document.activeElement` trần — xem `focusReturnTargetOnOpen`. Nút này
    // nằm TRONG một panel dockview, và trên WKWebView tiêu điểm không giữ được trên nó:
    // nó rơi lên `section.panel[tabindex="-1"]` ngay trong cùng một tick. Lưu node đó là
    // trả tiêu điểm về THÂN PANEL thay vì về nút, tức UX-DR17 hỏng ở đúng chỗ không cổng
    // nào nhìn thấy. Đo được ở bàn đo `attribution-focus` (2026-08-12).
    returnFocusTo = focusReturnTargetOnOpen('[data-attribution-open]')
    // Sau `nextTick`: `v-if` mới dựng node ở lượt render này, nên gọi `focus()` ngay bây giờ
    // là gọi lên một phần tử chưa có trong DOM.
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  // 🔴 `isConnected` chứ không `back?.focus()` trần (bắt ở code review 2026-08-10). Một node
  // đã rời DOM vẫn nhận được lời gọi `focus()` mà **không** ném và **không** có tác dụng —
  // tiêu điểm rơi về `body`, đúng thứ UX-DR17 cấm, và không một dấu hiệu nào báo. Chặn hợp
  // âm toàn cục khi lớp phủ mở (`main.ts`) đã bịt nguyên nhân chính; đây là vế còn lại, cho
  // mọi đường dựng lại DOM mà lượt này chưa lường tới.
  if (back !== null && back.isConnected) {
    back.focus()
    void nextTick(() => {
      if (back.isConnected && document.activeElement !== back) back.focus()
    })
    return
  }

  // Đường lui: nút đã **mở** lớp phủ này. Hợp đồng đi bằng một thuộc tính `data-`, không một
  // tên lớp CSS — tên lớp là chuyện trình bày và đổi được tự do, còn đây là một mối nối.
  const opener = document.querySelector<HTMLElement>('[data-attribution-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  // ⚠️ Chẩn đoán viết bằng **tiếng Anh**, và đó là luật chứ không một lựa chọn phong cách:
  // Kiểm A của `check:i18n` cấm chuỗi tiếng Việt ở **vị trí mã** trong `.vue` (NFR16), cùng
  // luật mà `eprintln!` của `src-tauri/src/**` đã theo. Chuỗi HIỂN THỊ đi qua `t()`; đây là
  // một dòng cho console, không một bề mặt người dùng.
  console.warn('[attribution] focus-return target is gone; focus falls back to body.')
})

function aimRow(code: string): void {
  aimAttributionSource(code)
}

watch(
  [attributionIsOpen, dictSources, attributionAimedCode],
  () => {
    if (!attributionIsOpen.value) return
    const code = attributionAimedCode.value
    if (code !== null && dictSources.value.some((s) => s.code === code)) return
    aimAttributionSource(dictSources.value[0]?.code ?? null)
  },
  { immediate: true },
)

watch(attributionAimedCode, (code, previous) => {
  if (attributionLicenseView.value === null || code === previous) return
  dispatch(canOpenLicense.value ? 'attribution.open_license' : 'attribution.close_license')
})

// The licence text viewer closes first; the next Escape closes the overlay.
function onEscape(): void {
  if (attributionLicenseView.value === null) dispatch('attribution.close')
  else dispatch('attribution.close_license')
}

// Whichever path closes the viewer, focus goes back to the foot button, or to the aimed row
// when that button is disabled (focus() on a disabled button is a silent no-op).
watch(attributionLicenseView, (view, previous) => {
  if (view !== null) {
    if (previous === null) void nextTick(() => licenseViewEl.value?.focus())
    return
  }
  if (previous === null) return
  void nextTick(() => {
    const button = panel.value?.querySelector<HTMLButtonElement>('button[data-license-open]')
    if (button !== null && button !== undefined && !button.disabled) button.focus()
    else panel.value?.querySelector<HTMLElement>('tr.attr-aimed')?.focus()
  })
})

// `:not([tabindex="-1"])` keeps the panel itself (focusable only by script) out of the Tab ring.
function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
}

// Always preventDefault and steer: the titlebar stays live, so a native Tab would leave the
// screen and Escape (bound on .attr-screen) would stop reaching the handler.
function trapTab(event: KeyboardEvent): void {
  const root = panel.value
  if (root === null) return

  event.preventDefault()
  const stops = focusableWithin(root)
  if (stops.length === 0) {
    // No stops: keep focus on the panel so Escape still reaches .attr-screen.
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
  <!--
    🔴 AC11 — `Escape` đóng lớp phủ và TRẢ TIÊU ĐIỂM về chỗ cũ.

    ⚠️ `@keydown.esc` chứ không một hợp âm trong `CommandRegistry`: `Escape` ở đây là một
    lượt **huỷ trong ngữ cảnh**, không một thao tác toàn cục — gán nó thành một command là
    chiếm phím `Escape` cho **cả** ứng dụng, và Story 1.21 sẽ hiện nó ra như một phím gán lại
    được trong khi nó chỉ có nghĩa khi lớp phủ đang mở.
    Nút đóng thì ĐI QUA command (`attribution.close`) — Kiểm A của `check:commands` đòi mọi
    `@click` là đúng một `dispatch('<id>')`, và một nút đóng không có command là một `@click`
    không đi qua `CommandRegistry`.

    ⚠️ `tabindex="-1"` để `focus()` đặt được tiêu điểm lên khối này mà nó không chen vào thứ
    tự Tab như một điểm dừng thật — cùng khuôn gốc chế độ/panel của Story 1.6 (§Trap 4).
  -->
  <div
    v-if="attributionIsOpen"
    class="attr-screen"
    @keydown.esc="onEscape()"
    @keydown.tab="trapTab($event)"
  >
    <section
      ref="panel"
      class="attr-panel"
      tabindex="-1"
      role="dialog"
      aria-labelledby="attr-title"
    >
      <header class="attr-head">
        <h2 id="attr-title" class="attr-title">{{ t('attribution.title') }}</h2>
        <button type="button" class="attr-close" @click="dispatch('attribution.close')">
          {{ t('command.attribution.close') }}
        </button>
      </header>

      <p class="attr-intro">{{ t('attribution.intro') }}</p>

      <!--
        🔴 **LỖI TẢI ĐỨNG TRƯỚC RỖNG, và thứ tự hai nhánh này là cả điểm** (Ice chốt ở code
        review 2026-08-10). `dictSourcesError` được `dictSourcesState.ts` xuất ra và gán từ
        đầu, nhưng cho tới lượt này **0** bề mặt nào đọc nó — một kênh lỗi giả vờ tồn tại.
        Hệ quả đo được: `list_dict_sources` trượt ⇒ `sources` giữ nguyên `[]` vĩnh viễn
        *(`loadDictSources` gọi đúng MỘT lần lúc khởi động, không retry)* ⇒ bảng này hiện
        `attribution.empty` — *"Chưa gắn lớp từ điển nào"* — một câu **SAI**, vì nguyên nhân
        thật là lỗi đọc chứ không phải bản cài thiếu tệp. Hai nguyên nhân khác nhau dẫn tới
        hai việc khác nhau cho người dùng, nên chúng phải nói hai câu khác nhau.
      -->
      <p v-if="dictSourcesError !== null" class="attr-empty">{{ t('attribution.load_failed') }}</p>

      <template v-else>
        <!--
          A third state, distinct from `dictSourcesError` (the call failed) and from an
          empty `dictSources` with nothing skipped: some layers loaded, one couldn't be read.
        -->
        <p v-if="someDictSourceUnreadable" class="attr-partial">{{ t('attribution.some_unreadable') }}</p>

        <!--
          `attribution.empty` must not show while `someDictSourceUnreadable` is true — a
          directory where every `.db` was skipped would otherwise falsely say no layer is attached.
        -->
        <p v-if="dictSources.length === 0 && !someDictSourceUnreadable" class="attr-empty">
          {{ t('attribution.empty') }}
        </p>

        <table v-else-if="dictSources.length > 0" class="attr-table">
        <thead>
          <tr>
            <th>{{ t('attribution.col_source') }}</th>
            <th>{{ t('attribution.col_license') }}</th>
            <th>{{ t('attribution.col_role') }}</th>
            <th>{{ t('attribution.col_credit') }}</th>
          </tr>
        </thead>
        <tbody>
          <!--
            🔴 AC10 — nguồn đang TẮT **vẫn** liệt kê đầy đủ. `v-for` chạy trên `dictSources`,
            danh sách đầy đủ từ `list_dict_sources` — không một phép lọc nào ở đây, và đó là
            mệnh đề, không một thiếu sót.
          -->
          <tr
            v-for="src in dictSources"
            :key="src.code"
            tabindex="0"
            :class="{ 'attr-aimed': src.code === attributionAimedCode }"
            :aria-current="src.code === attributionAimedCode ? 'true' : undefined"
            @mousedown="aimRow(src.code)"
            @focusin="aimRow(src.code)"
          >
            <td class="attr-name">
              <!-- aura-allow-text: tên nguồn — DỮ LIỆU (`display_name` của chính tệp). -->
              {{ src.display_name }}
              <!-- Trạng thái tắt nói ra bằng CHỮ, không bằng màu một mình (UX-DR27). -->
              <span v-if="sourceIsDisabled(src.code)" class="attr-off">{{
                t('attribution.state_off')
              }}</span>
            </td>
            <td class="attr-license">
              <!--
                🔴 AC9 — câu của `license_kind`, có NHÁNH MẶC ĐỊNH (`licenseKeyFor`).
                Khoá đi qua một hàm thuần chứ không một literal: năm `license_kind` cộng một
                nhánh mặc định thành sáu nhánh `v-if` trong `<template>`, và một bảng ánh xạ
                ở TS test được còn một chuỗi `v-if` thì không.
              -->
              {{ t(licenseKeyFor(src.license_kind)) }}
              <!--
                aura-allow-text: mã giấy phép (`CC-BY-SA-4.0`, `Unicode-3.0`) — DỮ LIỆU đọc
                từ tệp, một định danh máy chứ không một câu giao diện. `null` (đo được ở
                `tran-van-chanh` và `vietphrase`) ⇒ KHÔNG một ô trống: câu của `license_kind`
                ở trên đã đứng một mình (AC9).
              -->
              <span v-if="src.license_id !== null" class="attr-license-id">{{
                src.license_id
              }}</span>
              <p v-if="conversionNoticeApplies(src.license_id)" class="attr-conversion">
                {{ t('attribution.conversion_notice') }}
              </p>
            </td>
            <td>
              <span v-for="role in roleLangsFor(src.lang)" :key="role.lang" class="attr-role">{{
                t(role.key, { lang: role.lang })
              }}</span>
              <span class="attr-layer">{{ t(layerKeyFor(src.is_base)) }}</span>
            </td>
            <!--
              aura-allow-text: ghi công — DỮ LIỆU của chính tệp (`dict_source.attribution`).
              🔴 NGUYÊN VĂN, ĐẦY ĐỦ, không `ellipsis`: `tran-van-chanh` mang một CẢNH BÁO
              PHÁP LÝ trong trường này, và cắt nó là cắt nửa sau của một câu pháp lý (AC7).
              Đây cũng là chỗ DUY NHẤT danh tính tác giả được đọc ra (AC9).
            -->
            <td class="attr-credit">
              {{ src.attribution }}
              <p v-if="src.source_version !== ''" class="attr-provenance">
                {{ t('attribution.source_version', { version: src.source_version }) }}
              </p>
              <!-- aura-allow-text: source_url — DATA read from the file. -->
              <p v-if="src.source_url !== ''" class="attr-provenance">{{ src.source_url }}</p>
            </td>
          </tr>
        </tbody>
        </table>

        <section
          v-if="attributionLicenseView !== null"
          ref="licenseViewEl"
          class="attr-license-view"
          tabindex="-1"
        >
          <h3 class="attr-license-heading">{{ t('attribution.license_text_heading') }}</h3>
          <p v-if="attributionLicenseView.error !== null" class="attr-error" role="alert">
            {{ tError(attributionLicenseView.error) }}
          </p>
          <p v-else-if="attributionLicenseView.text === ''" class="attr-partial">
            {{ t('attribution.license_text_empty') }}
          </p>
          <!-- aura-allow-text: licence text — DATA read from the file at click time. -->
          <pre v-else class="attr-license-text" tabindex="0">{{ attributionLicenseView.text }}</pre>
          <button type="button" class="attr-action" @click="dispatch('attribution.close_license')">
            {{ t('command.attribution.close_license') }}
          </button>
        </section>
      </template>

      <div class="attr-legal">
        <div class="attr-card">
          <h3 class="attr-card-title">{{ t('attribution.legal_software_title') }}</h3>
          <p>{{ t('attribution.legal_software_body') }}</p>
        </div>
        <div class="attr-card">
          <h3 class="attr-card-title">{{ t('attribution.legal_owner_title') }}</h3>
          <p>
            {{ t('attribution.legal_owner_body') }}
            <span class="attr-contact">{{ t('attribution.legal_owner_contact') }}</span>
          </p>
        </div>
      </div>

      <div v-if="dictSourcesError === null" class="attr-foot">
        <button
          type="button"
          class="attr-action attr-action-primary"
          data-license-open
          :disabled="!canOpenLicense"
          @click="dispatch('attribution.open_license')"
        >
          {{ t('command.attribution.open_license') }}
        </button>
        <button
          type="button"
          class="attr-action"
          :disabled="aimedSource === undefined"
          @click="dispatch('attribution.copy_credit')"
        >
          {{ t('command.attribution.copy_credit') }}
        </button>
        <span class="attr-hint">
          {{ t('attribution.hint_licenses_bundled') }} ·
          <!-- aura-allow-text: tên phím. --><kbd>Tab</kbd> {{ t('attribution.hint_browse') }}
        </span>
        <p v-if="attributionCopyFailedCode !== null && attributionCopyFailedCode === attributionAimedCode" class="attr-error" role="alert">
          {{ t('attribution.copy_failed') }}
        </p>
      </div>

      <!-- 🔴 AC10 / FR112 — màn hình phải NÓI RA rằng tắt không phải gỡ. -->
      <p class="attr-note">{{ t('attribution.off_is_not_removed') }}</p>
    </section>
  </div>
</template>

<style scoped>
/* Follows .modeport in tree order and is positioned, so it paints over it without a z-index. */
.attr-screen {
  position: absolute;
  inset: 0;
  overflow: auto;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.attr-panel {
  width: 100%;
}

.attr-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.attr-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.attr-close {
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

.attr-intro,
.attr-note,
.attr-empty,
.attr-partial {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

/* Câu "tắt ≠ gỡ" là một mệnh đề pháp lý, không một chú thích — nó mang một nét dẫn. */
.attr-note {
  padding-left: 12px;
  border-left: 2px solid var(--color-tm-rule);
  color: var(--color-tm-text);
}

.attr-table {
  width: 100%;
  margin-bottom: var(--space-panel-block);
  border-collapse: collapse;
  text-align: left;
}

.attr-table th {
  padding: 0 8px var(--space-panel-block) 0;
  border-bottom: 1px solid var(--color-outline);
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

/*
 * 🔴 `vertical-align: top` và KHÔNG `text-overflow: ellipsis`, KHÔNG `white-space: nowrap`.
 *
 * Ghi công phải hiện NGUYÊN VĂN, ĐẦY ĐỦ (AC7): trường này mang cảnh báo pháp lý dài nhiều
 * dòng ở ít nhất một nguồn thật, và một ô cắt bằng `ellipsis` giấu đúng nửa quan trọng.
 */
.attr-table td {
  padding: var(--space-panel-block) 8px var(--space-panel-block) 0;
  border-bottom: 1px solid var(--color-outline);
  vertical-align: top;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface);
}

/* Tên nguồn — `primary` là màu của **nhãn nguồn từ điển** (`DESIGN.md` §Do's, một trong ba). */
.attr-name {
  color: var(--color-primary);
}

.attr-credit,
.attr-license {
  color: var(--color-on-surface-variant);
}

.attr-license-id,
.attr-layer,
.attr-off {
  display: inline-block;
  margin-left: 8px;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.attr-layer,
.attr-role {
  margin-left: 0;
  margin-right: 8px;
}

.attr-conversion,
.attr-provenance,
.attr-card p {
  margin: 4px 0 0 0;
}

.attr-table td:first-child {
  border-left: 2px solid transparent;
}

.attr-table tr.attr-aimed td {
  background: var(--color-surface);
}

.attr-table tr.attr-aimed td:first-child {
  border-left-color: var(--color-primary);
}

.attr-foot {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 12px;
  margin-bottom: var(--space-panel-block);
}

.attr-hint {
  margin-left: auto;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.attr-hint kbd {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface-variant);
}

.attr-foot .attr-error {
  flex-basis: 100%;
  margin: 0;
}

.attr-action {
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

.attr-action:disabled {
  cursor: default;
  border-color: var(--color-outline);
  background: none;
  color: var(--color-on-surface-variant);
}

.attr-action-primary:not(:disabled) {
  padding: 4px 12px;
  border: 1px solid var(--color-primary);
  background: var(--color-primary);
  color: var(--color-on-primary);
}

.attr-error {
  margin: 4px 0 0 0;
  color: var(--color-on-surface);
}

.attr-license-view {
  margin-bottom: var(--space-panel-block);
}

.attr-license-heading,
.attr-card-title {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.attr-license-text {
  max-height: 50vh;
  margin: 0 0 var(--space-panel-block) 0;
  overflow: auto;
  padding: var(--space-panel-block);
  border: 1px solid var(--color-outline);
  white-space: pre-wrap;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface);
}

.attr-legal {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.attr-card {
  padding: var(--space-panel-block) var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.attr-contact {
  user-select: text;
  color: var(--color-on-surface);
}
</style>
