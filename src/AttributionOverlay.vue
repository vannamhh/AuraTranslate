<script setup lang="ts">
// Bề mặt **Attribution** — Story 1.19 · AC7 · AC8 · AC9 · AC10 · AC11.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT LỚP PHỦ, KHÔNG MỘT CHẾ ĐỘ THỨ TƯ (§Quyết định #4a, Ice chốt 2026-08-08)
// ─────────────────────────────────────────────────────────────────────────────
// AD-24 khai **BA** chế độ ngang hàng và `MODE_IDS` là một hằng có ba phần tử; thêm chế độ
// thứ tư là một quyết định kiến trúc, và `Mod+4` là phím của Story 8.11. Lớp phủ dựng ở
// `App.vue` — cùng tầng với dải báo lỗi cấu hình, không trong một panel: nó nói về **cả ứng
// dụng**, không về một panel.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 BỀ MẶT NÀY GIAO **NỬA TRÁI** CỦA BẢNG CHIA ĐÔI, KHÔNG HƠN
// ─────────────────────────────────────────────────────────────────────────────
// Story 10.4 giữ: nút *"Mở văn bản giấy phép"* · nút *"Sao chép bản ghi công"* · hai cột
// biên tập **Vai trò**/**Ghi chú xuất xứ** (nội dung KHÔNG có trong `.db`) · hai thẻ pháp lý
// · rà bàn phím đầy đủ. Đừng dựng chúng ở đây, và 10.4 đừng dựng lại cái này.
import { nextTick, useTemplateRef, watch } from 'vue'
import { t } from './i18n'
import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import {
  attributionIsOpen,
  closeAttribution,
  dictSources,
  dictSourcesError,
  layerKeyFor,
  licenseKeyFor,
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
useSelectionSurface(panel, 'display')

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

/**
 * Danh sách điểm dừng Tab **thật** bên trong lớp phủ, theo đúng thứ tự tài liệu.
 *
 * ⚠️ `:not([disabled])` và `:not([tabindex="-1"])` là bắt buộc: một nút vô hiệu hoá hay
 * `.attr-panel` (mang `tabindex="-1"` để nhận `focus()` mà không chen vào vòng Tab) lọt vào
 * danh sách sẽ làm vòng lặp dừng ở một chỗ người dùng không thao tác được.
 */
function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
}

/**
 * 🔴 **BẪY TIÊU ĐIỂM — AC11, và nó là điều kiện để `aria-modal="true"` không phải một lời
 * khai sai** (bắt ở code review 2026-08-10).
 *
 * Trước lượt vá, điểm dừng Tab duy nhất bên trong là nút đóng, và không một cơ chế nào giữ
 * tiêu điểm lại. Bấm Tab thêm một lần là rơi ra các phần tử tương tác của ứng dụng — vẫn
 * **có** trong DOM và vẫn nhận được tiêu điểm, nhưng đang bị `.attr-scrim` che kín, nên
 * người dùng bàn phím mất hoàn toàn dấu vết tiêu điểm. Tệ hơn: `@keydown.esc` gắn trên
 * `.attr-scrim`, nên khi tiêu điểm đã ra ngoài thì `Escape` **không còn tới được** handler,
 * và lớp phủ thành một cái bẫy thật thay vì một hộp thoại.
 *
 * ⚠️ **Luôn `preventDefault()` rồi tự lái**, chứ không chỉ chặn ở hai đầu danh sách: tự lái
 * đúng ở **mọi** vị trí xuất phát, kể cả khi tiêu điểm đang ở `.attr-panel` *(vị trí ngay sau
 * lượt mở)* hay đã lạc ra ngoài vì một lượt dựng lại DOM. Phiên bản "chỉ chặn ở hai đầu" đọc
 * gọn hơn và để lọt đúng những ca đó.
 */
function trapTab(event: KeyboardEvent): void {
  const root = panel.value
  if (root === null) return

  event.preventDefault()
  const stops = focusableWithin(root)
  if (stops.length === 0) {
    // Lớp phủ không một điểm dừng nào (bảng rỗng và nút đóng bị gỡ): giữ tiêu điểm trên
    // chính khối hộp thoại, vì `Escape` vẫn phải tới được `.attr-scrim`.
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
    class="attr-scrim"
    @keydown.esc="closeAttribution()"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="attr-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="attr-head">
        <h2 class="attr-title">{{ t('attribution.title') }}</h2>
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

      <!--
        🔴 AC8 — bảng DẪN XUẤT từ tệp có mặt. Danh sách rỗng là một trạng thái BÌNH THƯỜNG
        có tên (AD-25: `src-tauri/resources/dict/` rỗng trong git), không một lỗi.
      -->
      <p v-else-if="dictSources.length === 0" class="attr-empty">{{ t('attribution.empty') }}</p>

      <table v-else class="attr-table">
        <thead>
          <tr>
            <th>{{ t('attribution.col_source') }}</th>
            <th>{{ t('attribution.col_license') }}</th>
            <th>{{ t('attribution.col_layer') }}</th>
            <th>{{ t('attribution.col_credit') }}</th>
          </tr>
        </thead>
        <tbody>
          <!--
            🔴 AC10 — nguồn đang TẮT **vẫn** liệt kê đầy đủ. `v-for` chạy trên `dictSources`,
            danh sách đầy đủ từ `list_dict_sources` — không một phép lọc nào ở đây, và đó là
            mệnh đề, không một thiếu sót.
          -->
          <tr v-for="src in dictSources" :key="src.code">
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
            </td>
            <td>
              <span class="attr-layer">{{ t(layerKeyFor(src.is_base)) }}</span>
            </td>
            <!--
              aura-allow-text: ghi công — DỮ LIỆU của chính tệp (`dict_source.attribution`).
              🔴 NGUYÊN VĂN, ĐẦY ĐỦ, không `ellipsis`: `tran-van-chanh` mang một CẢNH BÁO
              PHÁP LÝ trong trường này, và cắt nó là cắt nửa sau của một câu pháp lý (AC7).
              Đây cũng là chỗ DUY NHẤT danh tính tác giả được đọc ra (AC9).
            -->
            <td class="attr-credit">{{ src.attribution }}</td>
          </tr>
        </tbody>
      </table>

      <!-- 🔴 AC10 / FR112 — màn hình phải NÓI RA rằng tắt không phải gỡ. -->
      <p class="attr-note">{{ t('attribution.off_is_not_removed') }}</p>
    </section>
  </div>
</template>

<style scoped>
/*
 * Nền phủ. `--color-background` chứ không một `rgba()` viết thẳng — Kiểm B của
 * `check:tokens` đỏ với một giá trị màu viết thẳng, và ở đây không có gì cần lách.
 */
.attr-scrim {
  position: fixed;
  inset: 0;
  /* aura-allow-z-index: xếp lớp CƠ HỌC — dockview dựng ngữ cảnh xếp lớp riêng cho mỗi nhóm panel, nên thứ tự tài liệu một mình không đủ để lớp phủ nằm TRÊN lưới. */
  z-index: 10;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.attr-panel {
  width: 100%;
  max-width: 1100px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
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
.attr-empty {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

/* Câu "tắt ≠ gỡ" là một mệnh đề pháp lý, không một chú thích — nó mang một nét dẫn. */
.attr-note {
  padding-left: 11px;
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
  margin-left: 6px;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.attr-layer {
  margin-left: 0;
}
</style>
