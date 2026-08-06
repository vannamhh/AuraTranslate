<script setup lang="ts">
// Vỏ panel + **hợp đồng thị giác tiêu điểm**. Story 1.6 · AC4 · AC5 · UX-DR8 · UX-DR17.
// Story 1.14 · §Quyết định #4A — mổ lại theo đúng chỗ mà doc-comment cũ đã lường trước.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 `<header>` ĐÃ BỊ GỠ — §Quyết định #4A của Story 1.14
// ─────────────────────────────────────────────────────────────────────────────────
// UX-DR17 và `mockups/key-screen-workspace.html:31-34` vẽ MỘT thanh 34px cho mỗi panel.
// dockview cũng vẽ một tab bar riêng cho mỗi group. Hai thanh chồng nhau là hỏng thị giác
// ngay lượt dựng đầu, nên story chọn: **tab bar của dockview LÀ thanh tiêu đề panel**
// (`src/panels/PanelTab.vue`), và vỏ này bỏ `<header>` của nó.
//
// ⛔ Đường ngược lại — ẩn tab bar của dockview, giữ `<header>` ở đây — bị loại vì ẩn tab
// bar là ẩn luôn affordance kéo-thả và gộp-tab mà FR17 đòi; dựng lại chúng bằng tay là tự
// viết lại dockview, đúng thứ `EXPERIENCE.md:21` cấm.
//
// ⚠️ Nét/khe/bo góc phân tách panel cũng đã chuyển sang `.dv-groupview`
// (`src/layout/dockview-theme.css`): hộp panel thật nay bọc CẢ thanh tab LẪN thân, nên
// một `border` ở đây chỉ viền được nửa dưới và thanh tiêu đề nằm ngoài khung.
//
// 🔴 VẠCH TIÊU ĐIỂM 2px Ở LẠI ĐÂY, ⛔ không chuyển sang tab (§Quyết định #4A, AC5 Story
// 1.6): nó là mệnh đề về *panel nào đang giữ focus DOM*, và chỗ duy nhất đọc được điều đó
// là phần tử gốc `tabindex="-1"` ngay dưới.
//
// ⚠️ Ngày Story 1.16/1.17 đổ chữ thật vào thân, bề mặt đó phải khai token `read-*` /
// `source-*` / `lookup-*` của chính nó. Mặc định kế thừa từ `body` là `ui-md` ở giãn dòng
// 1.5 — dưới sàn 1.66 — và Kiểm E của `check-tokens.mjs` chỉ đọc `tokens.json` nên hoàn
// toàn mù với việc component nào đang kế thừa gì.
import { ref, useTemplateRef, onBeforeUnmount, onDeactivated, onMounted } from 'vue'
import { declareFocus, releaseFocus } from '../commands'
import { t } from '../i18n'

const props = withDefaults(
  defineProps<{
    /** Điểm vào focus: `panel.source`. Phải có mặt trong `FOCUS_OWNERS` (Kiểm E của cổng). */
    owner: string
    /**
     * Khoá `vi.json` của câu trạng thái. ⛔ Không nhận chuỗi đã dịch — NFR16.
     *
     * ⚠️ Tiêu đề ⛔ KHÔNG còn ở đây: nó sống trên tab (`PanelTab.vue`). Prop `titleKey` cũ
     * đã bị gỡ cùng `<header>` — giữ lại một prop không ai render là cách một khoá chết
     * lặng lẽ ở lại trong `vi.json` qua chín epic.
     *
     * ⚠️ **Luôn viết LITERAL ở chỗ gọi**, ⛔ không qua biến — Kiểm E của
     * `npm run check:commands` đọc TĨNH thuộc tính này (Story 1.16 · Quyết định #5). Một
     * biểu thức ở đó bị đếm rồi bỏ qua, tức mất lưới — dù panel có nội dung thật hay
     * không, `status-key` vẫn phải là một chuỗi literal.
     */
    statusKey: string
    /**
     * Có vẽ `<p class="status">` hay không — Story 1.16 · Quyết định #5.
     *
     * 🔴 Mặc định `true`: ba panel chưa có nội dung thật (Lookup, AI, Editor) tiếp tục hiện
     * câu trạng thái y hệt trước story này — ⛔ không hồi quy. Panel Source (Task 6) truyền
     * `:show-status="false"` khi một Chương đã nạp xong, và **đó là boolean qua prop**,
     * ⛔ không phải bind điều kiện lên `status-key` (đúng lý do Quyết định #5 chọn đường
     * này thay vì đường kia).
     */
    showStatus?: boolean
  }>(),
  { showStatus: true },
)

const root = useTemplateRef<HTMLElement>('root')

/**
 * Trạng thái tiêu điểm đọc từ DOM THẬT, không từ một biến "panel nào đang hoạt động".
 *
 * ⚠️ Đây là khác biệt có thật, không phải chi tiết cài đặt: AC4 nói *"focus không bao giờ
 * rơi về `body`"*, tức mệnh đề về `document.activeElement`. Một cờ do ứng dụng tự giữ sẽ
 * vẫn sáng đúng một panel trong khi focus thật đã rơi ra ngoài — vạch dọc nói dối, và
 * đúng nửa NFR17 mà AC5 tồn tại để giữ thì mất.
 *
 * 🔴 Story 1.14: dockview có con trỏ `activePanel` riêng. ⛔ **Đừng** dùng nó làm nguồn sự
 * thật cho vạch này — nó là *"tab nào đang chọn"*, ⛔ không phải *"focus DOM đang ở đâu"*,
 * và hai thứ đó tách nhau ngay lần đầu người dùng bấm chuột vào thân một panel khác.
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
 *
 * 🔴 Story 1.14 — `<DockviewPortals>` teleport panel về đúng cây component của host, nên
 * `onDeactivated` VẪN chạy khi rời Workspace. Đó là lý do vỏ này ⛔ không phải đổi gì để
 * sống trong dockview; ⛔ đừng "tối ưu" bằng cách bỏ hook này.
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
 *
 * 🔴 Story 1.14 làm ca đó THẬT: `hidePanel()` gọi `api.removePanel()` và `showPanel()`
 * dựng lại một instance mới với cùng `owner`. Không có `release()` đúng cặp thì lượt hiện
 * lại thứ hai ném.
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
    nhận hợp âm `Mod+Alt+→` từ Story 1.14 (§Quyết định #2).
  -->
  <section
    ref="root"
    class="panel"
    :class="{ focused }"
    tabindex="-1"
    @focusin="onFocusIn"
    @focusout="onFocusOut"
  >
    <!--
      ⛔ Thân panel CHƯA có nội dung ở story này — nó là **khung bố cục**, không phải một
      lượt dựng nội dung. `Source` là Story 1.16, `Lookup` là 1.17, `Editor` là Epic 2,
      `AiTranslation` là Epic 4.

      🔴 Nhưng ⛔ KHÔNG để trống KHÔNG GIẢI THÍCH (AC8, UX-DR27): mỗi panel nêu rõ trạng
      thái của nó bằng một chuỗi trong `vi.json`. Một khung trống câm là thứ người dùng
      đọc thành "hỏng".
    -->
    <p v-if="props.showStatus" class="status">{{ t(props.statusKey) }}</p>
    <div class="panel-body">
      <slot />
    </div>
  </section>
</template>

<style scoped>
.panel {
  /* `relative` là điều kiện để `::before` của vạch tiêu điểm neo vào mép trái panel. */
  position: relative;
  display: flex;
  flex-direction: column;
  min-width: 0;
  height: 100%;
  overflow: hidden;
  background-color: var(--color-surface);
  padding: var(--space-panel-block) var(--space-panel-inline);
  box-sizing: border-box;
  /*
   * ⚠️ Nét/khe/bo góc phân tách panel KHÔNG còn ở đây — chúng chuyển sang `.dv-groupview`
   * (`src/layout/dockview-theme.css`) ở Story 1.14, vì hộp panel thật nay bọc cả thanh
   * tab. Bốn biến `--panel-*` vẫn là nguồn duy nhất; chỉ đổi chỗ tiêu thụ.
   * ⛔ Đừng khai lại `border` ở đây: hai chỗ cùng vẽ một đường kẻ cho ra nét đôi ở theme
   * sáng và không ai nối được nó về một trong hai.
   */
}

/*
 * ⚠️ `outline: none` CHỈ ở gốc `tabindex="-1"` — xem lý do đầy đủ ở `LibraryMode.vue`.
 * ⛔ KHÔNG phải `*:focus { outline: none }` (§Trap 4). Nút, ô nhập và tab của các story
 * sau vẫn phải giữ focus ring của trình duyệt.
 * Kiểm H của `check-tokens.mjs` (Story 1.14 · AC11.2) cưỡng chế mệnh đề đó bằng máy.
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

/* UX-DR27 — câu trạng thái, `ui-md` màu chữ phụ. ⛔ Không màu `error`, kể cả ở panel AI. */
.status {
  margin: 0;
  flex: none;
  color: var(--color-on-surface-variant);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

/* ⛔ Thân để TRỐNG — nội dung là Story 1.16 (Source), 1.17 (Lookup), Epic 2/4. */
.panel-body {
  flex: 1;
  min-height: 0;
}
</style>
