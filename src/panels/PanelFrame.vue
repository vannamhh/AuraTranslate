<script setup lang="ts">
// Vỏ panel + **hợp đồng thị giác tiêu điểm**. Story 1.6 · AC4 · AC5 · UX-DR8 · UX-DR17.
//
// ⛔ Đây là VỎ, không phải panel. Bốn panel thật (`Source` · `Lookup` · `AiTranslation` ·
// `Editor`) và nội dung của chúng thuộc Story 1.14 / 1.16 / 1.17; `dockview`, dock/undock/
// tab/preset bố cục thuộc Story 1.14. Thân panel ở đây **để trống**.
//
// ⚠️ Ngày Story 1.16/1.17 đổ chữ thật vào thân, bề mặt đó phải khai token `read-*` /
// `source-*` / `lookup-*` của chính nó. Mặc định kế thừa từ `body` là `ui-md` ở giãn dòng
// 1.5 — dưới sàn 1.66 — và Kiểm E của `check-tokens.mjs` chỉ đọc `tokens.json` nên hoàn
// toàn mù với việc component nào đang kế thừa gì.
import { ref, useTemplateRef, onBeforeUnmount, onDeactivated, onMounted } from 'vue'
import { declareFocus, releaseFocus } from '../commands'
import { t } from '../i18n'

const props = defineProps<{
  /** Điểm vào focus: `panel.source`. Phải có mặt trong `FOCUS_OWNERS` (Kiểm E của cổng). */
  owner: string
  /** Khoá `vi.json` của tiêu đề. ⛔ Không nhận chuỗi đã dịch — NFR16. */
  titleKey: string
}>()

const root = useTemplateRef<HTMLElement>('root')

/**
 * Trạng thái tiêu điểm đọc từ DOM THẬT, không từ một biến "panel nào đang hoạt động".
 *
 * ⚠️ Đây là khác biệt có thật, không phải chi tiết cài đặt: AC4 nói *"focus không bao giờ
 * rơi về `body`"*, tức mệnh đề về `document.activeElement`. Một cờ do ứng dụng tự giữ sẽ
 * vẫn sáng đúng một panel trong khi focus thật đã rơi ra ngoài — vạch dọc nói dối, và
 * đúng nửa NFR17 mà AC5 tồn tại để giữ thì mất.
 */
const focused = ref(false)

const onFocusIn = (): void => {
  focused.value = true
}

/**
 * ⚠️ `focusout` cũng bắn khi focus chỉ nhảy giữa hai phần tử BÊN TRONG panel. Tắt cờ
 * ngay ở đó là một lần nháy vạch mỗi lần người dùng Tab trong panel. `relatedTarget` là
 * phần tử sắp nhận focus — còn nằm trong panel thì không đổi gì.
 */
const onFocusOut = (event: FocusEvent): void => {
  const next = event.relatedTarget
  if (next instanceof Node && root.value?.contains(next)) return
  focused.value = false
}

/**
 * 🔴 VẠCH TIÊU ĐIỂM KHÔNG ĐƯỢC SỐNG SÓT QUA MỘT LƯỢT ĐỔI CHẾ ĐỘ.
 *
 * Gỡ hoặc đỗ một phần tử đang giữ focus **không chắc chắn** phát `focusout`, và
 * `<KeepAlive>` (§Quyết định thiết kế #6) thì giữ nguyên subtree thay vì tháo nó. Hệ quả
 * đo được: rời Workspace khi một panel đang có tiêu điểm ⇒ `focused` kẹt ở `true` ⇒ quay
 * lại thấy vạch 2px `primary` trên panel trong khi focus THẬT đã ở gốc chế độ (do
 * `onActivated` → `enterFocus`). Đúng cái mệnh đề *"vạch không nói dối"* mà comment của
 * `focused` ở trên tồn tại để giữ, vỡ trên đúng đường mà story bắt buộc phải đi.
 */
onDeactivated(() => {
  focused.value = false
})

/**
 * ⚠️ Chốt `owner` ở `setup`, KHÔNG đọc `props.owner` lại lúc `onBeforeUnmount`.
 *
 * Hai hook đọc prop tại thời điểm chạy của chúng, nên nếu `owner` từng đổi trên một
 * instance đang sống (`:owner="…"`, hoặc một lượt re-key trong `v-for` — chính là thứ
 * `dockview` của Story 1.14 mang tới), owner ĐÃ ĐĂNG KÝ không bao giờ được gỡ: nó rò
 * vĩnh viễn vào `byOwner`, và mọi lượt mount lại nó sau đó ném *"đã khai rồi"*. Cổng
 * không thấy được ca này — lời gọi owner phi-literal bị đếm rồi bỏ qua.
 */
const owner = props.owner

onMounted(() => {
  declareFocus(owner, () => root.value)
})
onBeforeUnmount(() => {
  releaseFocus(owner)
})
</script>

<template>
  <!--
    `tabindex="-1"` để `focus.ts` dời được focus vào đây bằng `el.focus()`. Panel KHÔNG
    vào thứ tự Tab của trình duyệt: đường đi giữa các panel là `focus.next_panel`, và nó
    nhận phím ở Story 1.14/1.21.
  -->
  <section
    ref="root"
    class="panel"
    :class="{ focused }"
    tabindex="-1"
    @focusin="onFocusIn"
    @focusout="onFocusOut"
  >
    <header class="panel-head">
      <span class="panel-title">{{ t(props.titleKey) }}</span>
    </header>
    <div class="panel-body" />
  </section>
</template>

<style scoped>
.panel {
  /* `relative` là điều kiện để `::before` của vạch tiêu điểm neo vào mép trái panel. */
  position: relative;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
  background-color: var(--color-surface);
  /*
   * Phân tách panel ĐẢO NGƯỢC giữa hai theme (AC6 của Story 1.4): theme sáng dùng NÉT
   * 1px `outline`, theme tối dùng KHE 2px lộ `background` cộng bo góc 3px. Bốn biến dưới
   * đây do `applyTheme()` ghi, nên một component KHÔNG bao giờ phải biết mình đang ở
   * theme nào — và hai cơ chế không bị thống nhất về một cách làm.
   */
  border: var(--panel-border-width) solid var(--panel-border-color);
  border-radius: var(--panel-radius);
}

/*
 * ⚠️ `outline: none` CHỈ ở gốc `tabindex="-1"` — xem lý do đầy đủ ở `LibraryMode.vue`.
 * ⛔ KHÔNG phải `*:focus { outline: none }` (§Trap 4). Nút, ô nhập và tab của các story
 * sau vẫn phải giữ focus ring của trình duyệt.
 */
.panel:focus {
  outline: none;
}

/*
 * 🔴 AC5 — VẠCH DỌC 2px `primary` Ở MÉP TRÁI.
 *
 * ⛔ KHÔNG làm bằng `box-shadow`: Kiểm F của `check-tokens.mjs` cấm `box-shadow` và
 * `text-shadow` **không có đường miễn trừ** (AC7 Story 1.4 — không elevation).
 * ⛔ KHÔNG dùng viền bao quanh để báo tiêu điểm (AC5, UX-DR8). Hình dạng chủ đạo của sản
 * phẩm là VẠCH DỌC, không phải hộp bo tròn (`DESIGN.md §Shapes`).
 */
.panel.focused::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 2px;
  background-color: var(--color-primary);
}

.panel-head {
  display: flex;
  align-items: center;
  flex: none;
  height: var(--space-head-height);
  padding: 0 var(--space-panel-inline);
}

/* UX-DR17 — tiêu đề `ui-md` màu `on-surface-variant`. */
.panel-title {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

/*
 * Vế thứ hai của AC5: tiêu đề chuyển `primary` **in đậm**.
 *
 * ⚠️ `var(--weight-read-title)` là 600 — đúng con số mà `DESIGN.md §Components` và
 * `mockups/key-screen-workspace.html:34` ghi. Nó MƯỢN trọng lượng của một token khác vì
 * bộ token không có biến trọng lượng cho nhãn giao diện đậm: `ui-md` khai 400, `ui-label`
 * khai 700. Viết thẳng `600` thì Kiểm B2 của `check-tokens.mjs` đỏ, và ⛔ khai một biến
 * CSS cục bộ `--weight-…: 600` để lách cổng là đúng thứ AD-34 tồn tại để chặn.
 * Đã mở một mục `deferred-work.md` giao Story 1.14 quyết token thật.
 */
.panel.focused .panel-title {
  color: var(--color-primary);
  font-weight: var(--weight-read-title);
}

/* ⛔ Thân để TRỐNG — nội dung là Story 1.16 (Source) và 1.17 (Lookup). */
.panel-body {
  flex: 1;
  min-height: 0;
}
</style>
