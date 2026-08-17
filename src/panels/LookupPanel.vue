<script setup lang="ts">
// Panel `Tra cứu` — Story 1.17. Nội dung THẬT (bản ghi có cấu trúc), không còn là khung
// trống của Story 1.14.
//
// ⚠️ UX-DR15 xếp `Tra cứu` vào nhóm ĐƯỢC nhường chỗ — nhưng *"rút về thanh trạng thái,
// không bao giờ mất hẳn"*. Vế "thanh trạng thái" là **Story 4.12**; story này chỉ giao
// cơ chế ẩn (`SACRIFICE_ORDER` ở `src/layout/workspaceLayout.ts`), không đụng ở đây.
//
// 🔴 STORY 1.18 — Auto-Lookup nay là đường kích hoạt CHÍNH (`lookup.lookup_selection` vẫn
// là điểm nghẽn duy nhất; hợp đồng vùng chọn phát nó). Hiệu ứng 90ms (AC5/AC6), cuộn về
// đầu tức thì (AC7) và vạch tiến trình 250ms (AC8/AC9) sống ở tệp này.
import { computed, nextTick, onBeforeUnmount, ref, useTemplateRef, watch } from 'vue'
import PanelFrame from './PanelFrame.vue'
import LookupRecord from './LookupRecord.vue'
import type { DockviewPanelProps } from '../layout/panelProps'
import { t } from '../i18n'
import { dispatch } from '../commands'
import {
  aimDictSourceFrom,
  dictSources,
  everySourceOffForRoute,
  sourceIsDisabled,
} from './dictSourcesState'
import { useSelectionSurface } from './selectionContract'
import { markPainted } from './lookupTiming'
import {
  currentQuery,
  groupedLookup,
  layersLoaded,
  lookupDisplayable,
  lookupError,
  lookupPending,
  lookupResolved,
  lookupRoute,
  neverLookedUp,
  notFound,
  queryTooShort,
  queryWasTruncated,
  sensesByLayer,
  someLayerFailed,
  someLayerTruncated,
} from './lookupPanelState'
import { computeSpine, sourcesDisagree } from './lookupPanelState'
// ── Story 1.20 — dải tab, lịch sử trong phiên, bộ ghim ──────────────────────────────
//
// 🔴 State sống ở `lookupHistoryState.ts`, KHÔNG trong một `ref` cục bộ ở đây (AC5): đổi
// preset bố cục gọi `api.clear()` rồi dựng lại cả ba panel, và chỉ state module-level
// sống sót qua lượt tháo/dựng đó — một `ref` cục bộ làm tab tự nhảy về mặc định.
import {
  aimLookupEntryFrom,
  entryKey,
  historyIsEmpty,
  lookupHistory,
  lookupTab,
  pinNoticeKey,
  pinWriteError,
  pinnedEntries,
  pinnedIsEmpty,
  pinnedLoadFailed,
  relativeTimeKey,
  relativeTimeParams,
  sessionLookupCount,
} from './lookupHistoryState'
import { tError } from '../i18n'
import type { SenseRecord, SourceGroup } from '../config/dict'

defineProps<DockviewPanelProps>()

// AC7 — không spinner, không "đang tải". `lookupPending`/`lookupResolved` tồn tại để bốn trạng
// thái của AC6 không nháy sang "không tìm thấy" trong khoảng chờ IPC (cùng bẫy đã bắt ở 1.16),
// nhưng bề mặt này không vẽ một chỉ báo chờ nào — panel giữ nguyên trạng thái TRƯỚC ĐÓ cho tới
// khi lượt tra mới trả lời.

/**
 * AC6 trạng thái 1 — chỉ khi chưa tra gì thì `PanelFrame` mới hiện câu dạy thao tác.
 *
 * ⚠️ Vùng đầu mục vẫn được RENDER ở trạng thái này (chiều cao bất biến, AC7) — chỉ chữ
 * bên trong nó rỗng. Xem chú thích `.lookup-head`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 STORY 1.20 · AC14 — VÀ TAB ĐANG CHỌN PHẢI LÀ `record`
 * ═════════════════════════════════════════════════════════════════════════════════
 *
 * `PanelFrame.vue` render câu trạng thái **phía trên `<slot />`**, tức phía trên toàn bộ
 * thân panel. Mở app sạch rồi chuyển thẳng sang tab Lịch sử — **đúng ca nghiệm thu AC3**
 * (*"đóng rồi mở lại, mục ghim vẫn còn"*) — cho `neverLookedUp === true`, nên panel sẽ
 * hiện *"Chọn một từ trong Nguyên văn để tra cứu."* **đè trên danh sách ghim**. Câu đó sai
 * ngữ cảnh: người dùng đang xem mục đã ghim, không đang chờ tra cứu.
 *
 * ⚠️ Một dòng, nhưng **không** tìm ra được nếu chỉ đọc tệp này: nguyên nhân nằm ở
 * `PanelFrame.vue`, một tệp story này không sửa. Bắt lúc dựng story, thành AC14.
 */
const showFrameStatus = computed(() => neverLookedUp.value && lookupTab.value === 'record')

/** `id` của tab đang chọn — `aria-labelledby` của vùng nội dung trỏ về đúng nút đó. */
const tabpanelLabelledBy = computed(() =>
  lookupTab.value === 'record' ? 'lookup-tab-record' : 'lookup-tab-history',
)

/**
 * Đổi tab bằng MŨI TÊN — dispatch **và** dời tiêu điểm DOM sang nút vừa được chọn.
 *
 * 🔴 **Vì sao lời gọi `focus()` là bắt buộc, không trang trí** (bắt ở code review
 * 2026-08-11): `tabindex` roving giữ đúng MỘT nút trong vòng `Tab`, nên sau một lượt đổi
 * tab, nút cũ nhận `tabindex="-1"`. Nếu tiêu điểm ở lại đó thì lượt bấm mũi tên **thứ
 * hai** vẫn phát từ nút cũ và dispatch đúng cái id vừa chạy — một no-op. Với **hai** tab,
 * hệ quả đo được là người dùng bàn phím đi được một chiều rồi **kẹt**: không mũi tên nào
 * đưa họ về tab đầu. Đó đúng là *"hợp đồng `tablist` khai một nửa"* mà dải tab Hán Việt
 * cảnh báo bằng chữ, và là hàng **9** của bàn đo (*"mọi thao tác tới được, không chạm
 * chuột"*).
 *
 * ⚠️ Focus TRƯỚC lượt render kế tiếp là hợp lệ và cố ý: `focus()` chạy được trên một phần
 * tử đang mang `tabindex="-1"`, nên không cần `nextTick`. Đợi Vue vẽ xong chỉ thêm một
 * khung hình mà tiêu điểm nằm sai chỗ.
 *
 * ⚠️ Kiểm A của `check:commands` **không** áp cho `@keydown` (`check-commands.mjs:33`),
 * nên hàm này hợp lệ; luật *"đúng một `dispatch()`"* vẫn giữ nguyên cho mọi `@click`.
 */
function moveTabFocus(commandId: string, targetTabId: string): void {
  dispatch(commandId)
  document.getElementById(targetTabId)?.focus()
}

/**
 * Tên hiển thị của một nguồn, dẫn xuất từ **danh sách nguồn thật** — không một bảng viết
 * cứng trong `src/**`.
 *
 * ⚠️ Rơi về chính `code` khi chưa nạp được danh sách (hoặc khi nguồn đã bị gỡ khỏi bản
 * cài sau khi một mục của nó được ghim): một ô TRỐNG ở cột nguồn là FR31 vỡ — *"mọi định
 * nghĩa hiển thị nguồn"* — trong khi mã máy vẫn nói được nguồn nào.
 */
function sourceLabel(code: string): string {
  return dictSources.value.find((s) => s.code === code)?.display_name ?? code
}

const groups = computed<readonly SourceGroup[]>(() => groupedLookup.value?.groups ?? [])
const hiddenSources = computed(() => groupedLookup.value?.hidden_sources ?? [])
const spine = computed(() => computeSpine(groups.value, sensesByLayer.value, hiddenSources.value))
const disagree = computed(() => sourcesDisagree(groups.value))

/**
 * 🔴 AC6 — **một lượt tra TRƯỢT phải nói ra**. `lookupError` từng là một export không ai tiêu
 * thụ, nên một lỗi IPC cho ra thân panel TRẮNG CÂM: `neverLookedUp` đã `false` (tắt câu
 * mặc định của `PanelFrame`) còn `lookupResolved` `false` (tắt cả kết quả lẫn bốn chuỗi
 * rỗng). Đúng khuyết tật code review 1.16 đã bắt ở Panel Nguyên văn *(tệp đó gỡ ở Story
 * 2.5b — bề mặt nay là cột nguyên văn của `GridPanel.vue`)* — tái phát nguyên
 * văn ở 1.17, bắt lại 2026-08-07.
 */
const showLookupError = computed(() => lookupError.value !== null)

/** Nghĩa đã hydrate CỦA RIÊNG một nhóm — lọc theo `entry_id` khỏi danh sách chung của lớp. */
function sensesFor(group: SourceGroup): SenseRecord[] {
  const entryIds = new Set(group.entries.map((hit) => hit.entry_id))
  return (sensesByLayer.value[group.layer] ?? []).filter((sense) => entryIds.has(sense.entry_id))
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 1.18 — AC2/AC3 · ĐĂNG KÝ VỚI VAI `display`, không `source`
// ═════════════════════════════════════════════════════════════════════════════════
//
// Panel này **tự nó chứa chữ** (nghĩa, ví dụ, trích dẫn — `LookupRecord.vue`), nên nó là
// một bề mặt văn bản thật. Nhưng nó **không được** là nguồn vùng chọn: bôi đen một nghĩa để
// đọc kỹ mà phát ra một lượt tra mới sẽ **thay chính đoạn đang đọc** dưới tay người đọc,
// cộng một hiệu ứng 90 ms, cộng một lượt cuộn về đầu (Bẫy 1 — nguy hiểm nhất của story).
//
// 🔴 Nó vẫn ĐĂNG KÝ, không im lặng đứng ngoài: một panel văn bản không đăng ký gì cả là đúng thứ
// AC2 tồn tại để chặn, nên ngoại lệ này phải **đọc được từ chính mã** thay vì tồn tại như
// một sự vắng mặt mà không ai giải thích. Vai `'display'` LÀ mệnh đề đó, và hợp đồng hỏi nó
// TRƯỚC mọi nguồn (xem `surfaceFor`).
const body = useTemplateRef<HTMLElement>('body')
useSelectionSurface(body, 'display')

/** Vùng cuộn — AC7. `ref` riêng vì `.lookup-head` nay nằm NGOÀI nó (xem `<style>`). */
const scroller = useTemplateRef<HTMLElement>('scroller')

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 AC5 · AC6 — HIỆU ỨNG 90 ms, VÀ *"HUỶ"* KHÔNG ĐỒNG NGHĨA *"CHẠY LẠI"*
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ **CSS animation, không `Element.animate()` (WAAPI)** — và đó là hệ quả trực tiếp của
// Quyết định #6: `prefers-reduced-motion` phải đi bằng **CSS thuần**, mà một `@media` không
// dừng được một hiệu ứng WAAPI. Chọn WAAPI ở đây là tự tay làm AC9 không cài được bằng CSS,
// rồi phải mở `window.matchMedia` — đúng cánh cửa §⑥ của Story 1.17 vừa khoá (Bẫy 10).
//
// 🔴 **Đây là chỗ dễ cài sai nhất của story.** Mockup nói bằng chữ (`:187`): *"Tra nhanh
// liên tiếp thì thành **không có hiệu ứng** — đúng như người dùng muốn."* Một bản cài `:key`
// lại node, hoặc gỡ-rồi-gắn class để hiệu ứng chạy lại, **trông giống** *"huỷ hiệu ứng
// cũ"* và **là** thứ ngược lại: nó cho ra một chuỗi nháy 0.4 liên tục (Bẫy 4).
const fading = ref(false)
let fadeTimer: ReturnType<typeof setTimeout> | null = null

/** Hiệu ứng vào — **90 ms**, `opacity` 0.4 → 1, `ease-out`. không `translate`, không `scale`. */
const FADE_MS = 90

function playFade(): void {
  if (fading.value) {
    // 🔴 AC6 — một lượt tra mới **trong lúc hiệu ứng đang chạy**: huỷ, `opacity` về **1**
    // THẲNG, và **không chạy lại**. không xếp hàng, không cộng dồn.
    stopFade()
    return
  }
  fading.value = true
  // ⚠️ Hẹn giờ chứ không `animationend`: dưới `prefers-reduced-motion` CSS đặt `animation:
  // none` nên `animationend` **không bao giờ bắn**, và cờ sẽ kẹt `true` vĩnh viễn — lượt tra
  // kế tiếp rơi vào nhánh "huỷ" ở trên và không hiệu ứng nào chạy lại nữa. Một hằng thời gian
  // ở đây là **cùng** 90 ms mà CSS khai, và nó nhả ở MỌI nhánh thoát (bài học 1.16: một
  // cờ không được nhả ở nhánh lỗi khoá vĩnh viễn đường tra).
  fadeTimer = setTimeout(stopFade, FADE_MS)
}

function stopFade(): void {
  if (fadeTimer !== null) clearTimeout(fadeTimer)
  fadeTimer = null
  fading.value = false
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 AC8 · AC9 — VẠCH TIẾN TRÌNH 250 ms, KHÔNG SPINNER
// ═════════════════════════════════════════════════════════════════════════════════
//
// AC7 của Story 1.17 cấm spinner **vĩnh viễn**, và story này không nới nó. Vạch này là một
// **nét mảnh**: không quay, không nhấp nháy, và nó chỉ xuất hiện ở một ca mà `DESIGN.md` tự nói
// *"lẽ ra không xảy ra"* (NFR1 p95 < 100 ms).
const SLOW_LOOKUP_MS = 250
const showProgress = ref(false)
let progressTimer: ReturnType<typeof setTimeout> | null = null

function startProgressTimer(): void {
  cancelProgressTimer()
  progressTimer = setTimeout(() => {
    showProgress.value = true
  }, SLOW_LOOKUP_MS)
}

/**
 * Huỷ ở **mọi** nhánh thoát — lượt tra trả lời, `resetLookupPanel()` chạy, một lượt tra
 * mới vượt mặt, hoặc panel bị tháo. không một vạch mồ côi nào ở lại trên màn hình (AC8).
 */
function cancelProgressTimer(): void {
  if (progressTimer !== null) clearTimeout(progressTimer)
  progressTimer = null
  showProgress.value = false
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 MỘT TÍN HIỆU CHO CẢ BA — VÀ NÓ KHÔNG ĐỤNG MỘT DÒNG NÀO CỦA `runLookup`
// ═════════════════════════════════════════════════════════════════════════════════
//
// §KHÔNG-LÀM ① khoá `lookupPanelState.ts::runLookup`, kể cả bộ đếm `sequence`. `pending`
// đã mang đúng hai lượt chuyển mà cả ba cơ chế cần, và nó đã tồn tại:
//
//   `false → true`  = lượt tra vừa được PHÁT  ⇒ khởi động hẹn giờ 250 ms
//   `true → false`  = kết quả vừa TỚI         ⇒ huỷ hẹn giờ · hiệu ứng · cuộn về đầu · mốc NFR1
//
// ⚠️ `resetLookupPanel()` cũng đặt `pending = false`, nên nhánh thứ hai bắt ca đó — **nếu**
// một lượt tra đang bay lúc reset chạy (đúng mệnh đề AC8 *"hoặc `resetLookupPanel()` chạy"*).
// 🔴 Lượt review 2026-08-07 sửa lại khẳng định cũ ("bắt LUÔN mọi ca"): reset chạy khi
// `pending` ĐANG `false` (không lượt nào bay) thì nhánh này không bắn — xem watcher
// `neverLookedUp` riêng ở dưới cho ca đó (vị trí cuộn).
watch(lookupPending, (now, before) => {
  if (before === false && now === true) {
    startProgressTimer()
    return
  }
  if (before === true && now === false) {
    cancelProgressTimer()
    playFade()

    // 🔴 AC7 — vị trí cuộn về đầu **TỨC THÌ**. không `scrollIntoView`, không `behavior: 'smooth'`,
    // không `scroll-behavior` trong CSS.
    //
    // 🔵 **SỬA 2026-08-18 (Story 2.10) — vế trong ngoặc của dòng trên ĐÃ HẾT ĐÚNG, và nó chưa
    // từng đúng.** Bản cũ viết *"(`DESIGN.md:342` cấm cả ứng dụng)"*. Đo lại từ nguồn:
    // `DESIGN.md:342` nói về **chiều rộng `ch` của Chế độ đọc**, không một chữ nào về cuộn; và
    // `grep -rn "scroll-behavior"` trên **toàn** `_bmad-output/` trả **0 kết quả**. ⇒ Luật
    // *"cấm `scroll-behavior` toàn ứng dụng"* **chưa từng tồn tại** ở dạng đó.
    //
    // Hai mệnh đề gần nhất còn thật: `DESIGN.md:373` *(“Vị trí cuộn — về đầu tức thì, không bao
    // giờ cuộn có hiệu ứng”)* nằm trong bảng Motion **của chính Panel Lookup** — tức nó chống
    // đỡ AC7 ở đây, nhưng **chỉ ở đây**; và `DESIGN.md:379` *(ba cấm chung toàn ứng dụng)* thì
    // thật sự toàn ứng dụng nhưng **không nêu tên** `scroll-behavior`.
    //
    // ⚠️ **Hệ quả phải biết, không phải một chi tiết trích dẫn:** mệnh đề *"không `scroll-behavior`
    // ở đâu trong `src/**`"* là một **quy ước sống trong hai khối chú thích** *(đây và `:830`)*,
    // **không** một luật có cổng canh. Ai thêm một dòng `scroll-behavior: smooth` sẽ đi qua trọn
    // cả mười một cổng và làm hỏng **im lặng** cả AC7 ở đây lẫn AC8 của `GridPanel.vue`.
    // Đúng §Bẫy tài liệu của `project-context.md`: *"tin cây nguồn hiện tại hơn một chú thích"*.
    //
    // ⚠️ Sau `nextTick`: `scrollTop = 0` trên nội dung CŨ rồi Vue thay nội dung mới là một
    // lượt đặt vào hư không — chiều cao vùng cuộn đổi cùng lượt đó.
    // 🔴 Lượt review 2026-08-07 bắt được: `resetLookupPanel()` CŨNG đi qua nhánh này (đúng
    // ý — không cần đường nghe thứ hai), nhưng nó không phải một lượt tra đã VẼ kết quả —
    // nó là huỷ. Không lấy `sequence` nội bộ (§KHÔNG-LÀM ① khoá `lookupPanelState.ts`), nhưng
    // `neverLookedUp` đã phân biệt đúng: reset đặt `query.value = null` ⇒ `neverLookedUp` là
    // `true`; một lượt round-trip THẬT (dù thành công hay lỗi) giữ `query.value`. Không lọc ở
    // đây thì một Tác phẩm bị đổi giữa lúc đang tra ghi một mẫu NFR1 giả vào bảng AC4.
    const wasGenuineRoundTrip = !neverLookedUp.value

    void nextTick(() => {
      if (scroller.value !== null) scroller.value.scrollTop = 0

      // Mốc CUỐI của phép đo NFR1 — **sau khi trình duyệt đã VẼ**, không chỉ sau khi Vue ghi
      // xong DOM (Quyết định #7, Bẫy 7). Khi cờ đo TẮT đây là một lời gọi rỗng.
      if (wasGenuineRoundTrip) requestAnimationFrame(markPainted)
    })
  }
})

// 🔴 Lượt review 2026-08-07 — hàng đợi ở trên chỉ bắt `resetLookupPanel()` khi nó đổi `pending`
// (tức chạy giữa lúc một lượt tra ĐANG BAY). `resetLookupPanel()` gọi khi tạo Tác phẩm mới
// (`libraryImport.ts::finishSubmit`) mà không có lượt nào đang bay thì `pending` vẫn `false`
// trước lẫn sau ⇒ watcher trên không bắn ⇒ vị trí cuộn CŨ ở lại trên bản ghi cũ trong khi nội
// dung đã đổi về trạng thái "chưa tra" ngắn hơn — panel có thể trông TRỐNG. `neverLookedUp` bắt
// đúng lượt chuyển sang trạng thái đó bất kể `pending` đi đường nào.
watch(neverLookedUp, (now) => {
  if (now) void nextTick(() => {
    if (scroller.value !== null) scroller.value.scrollTop = 0
  })
})

// Panel bị tháo (đổi preset, ẩn panel) giữa một lượt tra chậm ⇒ không để lại hẹn giờ nào.
onBeforeUnmount(() => {
  cancelProgressTimer()
  stopFade()
})

</script>

<template>
  <PanelFrame owner="panel.lookup" status-key="panel.lookup.status" :show-status="showFrameStatus">
    <div ref="body" class="lookup-body">
      <!--
        🔴 STORY 1.19 · AC1 · AC2 · AC11 — DẢI CHIP NGUỒN.

        ⚠️ **NGOÀI `.lookup-head`, không TRONG nó** — và đó là một phép đo, không một sở
        thích. `.lookup-head` khoá `height: 76px; overflow: hidden`, và Story 1.17/1.18 đã
        vỡ đúng chỗ này **hai lần** (một lần với thanh nhịp, một lần với vạch tiến trình).
        Đo trên bố cục hiện tại: đầu mục 24px/1.3 ≈ 31px + `margin-top` 7px + thanh nhịp
        ≈ 15px + `padding-bottom` ⇒ vùng 76px **đã đầy**. Nhồi một hàng chip thứ ba vào đó
        là nới hằng trong im lặng, đúng thứ Bẫy 4 cấm.
        ⇒ dải chip là một hàng RIÊNG, `flex: none`, **trên** vùng đầu mục — đúng thứ tự mà
        `mockups/sources-attribution.html:132-141` vẽ (`.chips` đứng trước `.hw`).
        `--lookup-head-height` giữ NGUYÊN giá trị và NGUYÊN vai trò.

        🔴 AC11 — `@click` của mỗi chip là **đúng một** `dispatch('<id>')` (Kiểm A của
        `check:commands`). Mục tiêu đi bằng `@mousedown` uỷ quyền ở vùng chứa, cùng khuôn
        `deps.currentSelection` của `lookup.lookup_selection`: command đọc trạng thái quanh
        nó tại thời điểm chạy. Xem `dictSourcesState.ts` về vì sao WKWebView bắt buộc phải
        có `@mousedown` chứ không chỉ `document.activeElement`.
      -->
      <div
        v-if="dictSources.length > 0"
        class="lookup-sources"
        @mousedown="aimDictSourceFrom($event)"
      >
        <span class="lookup-sources-label">{{ t('panel.lookup.sources_label') }}</span>
        <!--
          🔴 **VÙNG CUỘN RIÊNG CHO CHIP — Ice chốt 2026-08-10 sau khi CHẠY THẬT.**

          Bản đầu để mọi thứ trong **một** hộp `flex-wrap` có `max-height: 52px; overflow:
          hidden`, với lý lẽ *"hàng thứ ba tới được bằng bàn phím"*. Ảnh chụp app thật với
          **mười** nguồn cho thấy lý lẽ đó bỏ sót hai thứ, và cả hai đều nặng:
          ① thứ bị cắt đầu tiên là **nút "Nguồn dữ liệu"** — nó là con CUỐI trong dải, tức
             **đường chuột DUY NHẤT** vào màn hình Attribution (AC11) biến mất;
          ② chip thứ 9 là **Trần Văn Chánh** (`license_kind = "copyrighted"`, mang cảnh báo
             pháp lý) và thứ 10 là VietPhrase — hai nguồn **không tắt được bằng chuột**, trong
             khi FR112 dựng cả cơ chế lớp gỡ rời chính vì rủi ro của nguồn thứ nhất.
          ⇒ nút tách **ra ngoài** vùng wrap (`flex: none`, không bao giờ bị cắt), và phần chip
          **cuộn** thay vì biến mất không dấu vết.

          ⚠️ Đây là một lượt **cầm máu**, không phải thiết kế cuối. Ice đã nêu hướng khác
          *(đưa bật tắt sang trang Settings, thu gọn dòng nhịp)* — nó đảo AC1/AC2/AC11 của
          Story 1.19 và cần một bề mặt Settings chưa tồn tại, nên đi qua `correct-course`.
        -->
        <div class="lookup-sources-chips">
          <button
            v-for="src in dictSources"
            :key="src.code"
            type="button"
            class="source-chip"
            :class="{ off: sourceIsDisabled(src.code) }"
            :data-source-code="src.code"
            :title="t('panel.lookup.source_toggle_hint')"
            @click="dispatch('lookup.toggle_source')"
          >
            <!-- aura-allow-text: tên nguồn — DỮ LIỆU (`display_name` của chính tệp, FR31/AC1). -->
            {{ src.display_name }}
          </button>
        </div>
        <!--
          ⚠️ `data-attribution-open` là một **mối nối**, không một móc kiểu dáng:
          `AttributionOverlay.vue` tìm lại nút này để trả tiêu điểm về khi node giữ tiêu điểm
          lúc mở đã rời DOM (UX-DR17). Đi bằng thuộc tính `data-` chứ không tên lớp CSS, vì
          tên lớp là chuyện trình bày và phải đổi được tự do.
        -->
        <button
          type="button"
          class="lookup-sources-attr"
          data-attribution-open
          @click="dispatch('attribution.open')"
        >
          {{ t('command.attribution.open') }}
        </button>
      </div>

      <!--
        🔴 STORY 1.20 · AC5 · AC10 — DẢI TAB, MỘT HÀNG **RIÊNG**.

        ⚠️ **NGOÀI `.lookup-head`, không TRONG nó** — cùng phép đo mà dải chip nguồn đã đi
        qua ở Story 1.19, và dải tab là thứ **thứ tư** muốn chỗ trong 76px đó: Story 1.17
        vỡ ở đây với thanh nhịp, 1.18 vỡ lần hai với vạch tiến trình, 1.19 phải tách dải
        chip ra ngoài. `--lookup-head-height` giữ NGUYÊN giá trị **và** NGUYÊN vai trò.

        🔴 **HAI tab, không ba** — Quyết định #4. `lookup-history-pins.html:103` vẽ ba tab
        gồm `Concordance`, nhưng đo trên mã thật 2026-08-10: `grep -rn "Concordance" src/`
        trả **0** lần, và hai doc-comment duy nhất ở `commands/dict.rs` đều nói Concordance
        là **FR64, Story 7.7** — một năng lực khác chưa dựng. Chữ *"tab thứ ba"* trong AC5
        là ngôn ngữ của mockup; mệnh đề thật của AC5 là *"trong Panel Lookup, không phải
        một cửa sổ riêng"*, và điều đó được thoả. Mockup **không** sửa (Quyết định #3 của
        Story 1.3); lệch ghi vào §Change Log.

        🔴 Năm thuộc tính là bắt buộc, không trang trí: `role="tab"` · `aria-selected` ·
        `aria-controls` · `tabindex` roving (đúng MỘT tab trong vòng Tab) · mũi tên
        trái/phải đổi tab. Dải tab Hán Việt ghi lý do bằng chữ: *"hợp đồng `tablist` khai
        một nửa còn tệ hơn không khai, vì nó hứa một mô hình tương tác không tồn tại"*.
      -->
      <div class="lookup-tabs" role="tablist">
        <button
          id="lookup-tab-record"
          type="button"
          class="lookup-tab"
          role="tab"
          aria-controls="lookup-tabpanel"
          :aria-selected="lookupTab === 'record'"
          :tabindex="lookupTab === 'record' ? 0 : -1"
          :class="{ active: lookupTab === 'record' }"
          @click="dispatch('lookup.select_tab_record')"
          @keydown.right.prevent="moveTabFocus('lookup.select_tab_history', 'lookup-tab-history')"
          @keydown.left.prevent="moveTabFocus('lookup.select_tab_history', 'lookup-tab-history')"
        >{{ t('panel.lookup.tab_record') }}</button>
        <button
          id="lookup-tab-history"
          type="button"
          class="lookup-tab"
          role="tab"
          aria-controls="lookup-tabpanel"
          :aria-selected="lookupTab === 'history'"
          :tabindex="lookupTab === 'history' ? 0 : -1"
          :class="{ active: lookupTab === 'history' }"
          @click="dispatch('lookup.select_tab_history')"
          @keydown.right.prevent="moveTabFocus('lookup.select_tab_record', 'lookup-tab-record')"
          @keydown.left.prevent="moveTabFocus('lookup.select_tab_record', 'lookup-tab-record')"
        >{{ t('panel.lookup.tab_history') }}</button>
      </div>

      <!--
        🔴 AC7 — vùng đầu mục CHIỀU CAO CỐ ĐỊNH, không đổi một pixel **kể cả giữa bốn trạng
        thái của AC6**. Vì thế nó render ở MỌI trạng thái, kể cả "chưa tra gì": bản đầu bọc
        cả khối trong `v-if="!neverLookedUp"` nên chiều cao đi 0 → 76px đúng lúc chuyển
        trạng thái, tức hằng có tên thì có mà bất biến nó tồn tại để giữ thì không (bắt ở code
        review 2026-08-07).

        🔴 STORY 1.18 · AC7 — khối này nay nằm **NGOÀI** vùng cuộn. Xem khối kiểu dáng ở
        cuối tệp.

        ⚠️ không viết tên thẻ `style`/`template` kèm dấu ngoặc nhọn trong một chú thích `.vue`:
        `check-i18n.mjs` cắt tệp thành ba khối bằng chính hai thẻ đó, nên một lượt nhắc tên
        chúng ở đây làm cổng cắt nhầm và **mọi** miễn trừ `aura-allow-text` phía dưới mất
        hiệu lực cùng lúc (bắt lúc chạy cổng, 2026-08-07).
      -->
      <div class="lookup-head">
        <!-- aura-allow-text: truy vấn vừa tra — DỮ LIỆU người dùng chọn, không chuỗi giao diện. -->
        <p class="lookup-headword">{{ currentQuery }}</p>
        <div v-if="lookupDisplayable && groups.length > 0" class="lookup-spine">
          <!--
            🔴 AC12 — thanh nhịp không **khẳng định một con số nó không biết**. Khi trần `LIMIT`
            đã cắt, `senseCountAtLeast` bật và số đọc ra như một CẬN DƯỚI (`≥`). Số nguồn
            thì luôn ĐẦY ĐỦ — nguồn bị cắt sạch vẫn được đếm qua `hidden_sources`.
          -->
          <span class="lookup-spine-count">
            <!-- aura-allow-text: số đếm dẫn xuất từ dữ liệu, không chuỗi giao diện tĩnh. -->
            {{ spine.sourceCount }} {{ t('panel.lookup.unit_sources') }} ·
            <span v-if="spine.senseCountAtLeast">{{ t('panel.lookup.at_least') }} </span
            ><!-- aura-allow-text: số nghĩa — DỮ LIỆU dẫn xuất. -->{{ spine.senseCount }}
            {{ t('panel.lookup.unit_senses') }}
          </span>
          <span v-for="src in spine.sources" :key="src.code" class="lookup-spine-chip">
            <!-- aura-allow-text: tên nguồn — DỮ LIỆU (`display_name`, AC2/FR31). -->
            {{ src.displayName }}
            <!-- aura-allow-text: số nghĩa của nguồn này — DỮ LIỆU dẫn xuất. -->
            <i>{{ src.atLeast ? '≥' : '' }}{{ src.senseCount }}</i>
          </span>
          <!-- 🔴 FR31 — nguồn bị trần cắt SẠCH vẫn phải được GỌI TÊN, không chỉ đếm. -->
          <span v-for="h in spine.hidden" :key="`hidden-${h.displayName}`" class="lookup-spine-chip is-hidden">
            <!-- aura-allow-text: tên nguồn bị giấu — DỮ LIỆU (`display_name`). -->
            {{ h.displayName }}
            <!-- aura-allow-text: số đầu mục của nguồn bị giấu — DỮ LIỆU. -->
            <i>{{ h.count }}</i>
          </span>
        </div>

        <!--
          🔴 AC8 — VẠCH TIẾN TRÌNH, không SPINNER. Neo **TUYỆT ĐỐI** ở đáy vùng đầu mục.

          `DESIGN.md:346` nói vạch nằm *"ở đáy vùng đầu mục"*, và đọc tự nhiên thì đó là
          *"trong luồng, dưới cùng"* — nhưng đường đó **phá AC7 của Story 1.17**: một vạch
          2px thêm vào LUỒNG đẩy `.lookup-head` từ 76px lên 78px, tức đầu mục và thanh nhịp
          **dịch chỗ mỗi lượt tra chậm**, đúng *"layout nhảy"* mà `DESIGN.md:336` gọi là
          thủ phạm gây giật (Bẫy 6 — và bản đầu của 1.17 đã vỡ đúng chỗ này một lần rồi).

          ⇒ `position: absolute` bên trong một `.lookup-head` mang `position: relative`.
          `--lookup-head-height` giữ NGUYÊN giá trị **và** giữ nguyên vai trò; `overflow:
          hidden` không bị bỏ.
        -->
        <div v-if="showProgress" class="lookup-progress"></div>
      </div>

      <!--
        🔴 AC7 — VÙNG CUỘN, và `.lookup-head` nay nằm NGOÀI nó.

        ⚠️ Story viết *"`.lookup-head` không cuộn — nó `flex: none`"*. `flex: none` chặn co
        giãn, nó **không chặn cuộn**: một phần tử nằm TRONG hộp `overflow: auto` thì cuộn đi
        cùng nội dung, bất kể `flex`. Bản 1.17 để `.lookup-head` bên trong `.lookup-body`
        (`overflow: auto`), nên đầu mục trôi mất khi đọc một bản ghi dài — mệnh đề AC7 vỡ
        về CƠ HỌC trong khi ý định thì rõ. ⇒ tách vùng cuộn ra một lớp riêng; `.lookup-body`
        thôi cuộn, `.lookup-scroll` nhận `overflow: auto`.
      -->
      <div
        id="lookup-tabpanel"
        ref="scroller"
        role="tabpanel"
        :aria-labelledby="tabpanelLabelledBy"
        class="lookup-scroll"
        :class="{ 'lookup-fade': fading }"
        @mousedown="aimLookupEntryFrom($event)"
      >
        <!--
          🔴 STORY 1.20 · §Dev Notes ⑧ ca 3 — **KHÔNG im lặng không hiệu lực.** Một phím
          ghim bấm khi chưa nhắm được mục nào phải nói ra bằng một câu CÓ LÝ DO (UX-DR27,
          AD-44 ④). Và một lượt GHI trượt là một câu KHÁC hẳn — nó không làm danh sách đang
          hiện sai, nên nó không được đọc thành *"không đọc được danh sách đã ghim"*.

          ⚠️ Cả hai đứng **ngoài** nhánh tab, có chủ ý: `Mod+D` bấm được ở cả hai tab, nên
          câu trả lời của nó cũng phải thấy được ở cả hai.
        -->
        <p v-if="pinNoticeKey !== null" class="lookup-banner">{{ t(pinNoticeKey) }}</p>
        <p v-if="pinWriteError !== null" class="lookup-banner">{{ tError(pinWriteError) }}</p>

        <template v-if="lookupTab === 'record'">
        <!-- 🔴 AC6 — một lượt tra TRƯỢT phải nói ra, không để lại một vùng trắng câm. -->
        <p v-if="showLookupError" class="lookup-empty">{{ t('panel.lookup.lookup_failed') }}</p>

        <template v-else-if="!neverLookedUp">
          <p v-if="someLayerFailed" class="lookup-banner">{{ t('panel.lookup.some_layer_failed') }}</p>
          <p v-if="someLayerTruncated" class="lookup-banner">{{ t('panel.lookup.list_incomplete') }}</p>
          <!-- Trần ĐỘ DÀI đã cắt truy vấn — không được đọc thành "không tìm thấy" (câu đó SAI ở đây). -->
          <p v-if="queryWasTruncated" class="lookup-banner">{{ t('panel.lookup.query_truncated') }}</p>

          <!--
            Bốn trạng thái rỗng đọc `lookupResolved` (không nháy trong khoảng chờ IPC); bản ghi
            và thanh nhịp đọc `lookupDisplayable` (không chớp trắng). Hai vị từ, hai câu hỏi khác
            nhau — xem `lookupPanelState.ts`.
          -->
          <p v-if="lookupResolved && !layersLoaded" class="lookup-empty">{{ t('panel.lookup.no_layers') }}</p>
          <p v-else-if="lookupResolved && queryTooShort" class="lookup-empty">
            {{ t('panel.lookup.query_too_short') }}
          </p>
          <!--
            🔴 STORY 1.19 · AC6 — "MỌI NGUỒN ĐỀU TẮT" LÀ MỘT TRẠNG THÁI CÓ TÊN.

            Nó đứng **TRƯỚC** `not_found`, và thứ tự đó là cả nội dung của AC6: với mọi nguồn
            tắt, `groups` rỗng · `branch = exact_btree` · `layers_loaded = true` ⇒ `notFound`
            là `true`, và panel sẽ nói *"không tìm thấy trong từ điển"* — một câu **SAI**, hệ
            thống không hề tra. Chuỗi riêng này chỉ đường về dải chip, không nói *không tìm thấy*.

            🔴 **Hỏi theo ĐƯỜNG ĐANG TRA, không theo toàn tập** (Acceptance Auditor bắt ở code
            review 2026-08-10, Ice chốt vá thật). Vị từ cũ hỏi toàn tập, nên tắt riêng
            `viwiktionary-en` — nguồn **DUY NHẤT** của đường tiếng Anh — vẫn cho `false` vì bảy
            nguồn tiếng Trung còn bật, và mọi truy vấn tiếng Anh rơi xuống đúng câu `not_found`
            SAI mà nhánh này tồn tại để chặn. `lookupRoute` đọc `grouped.route` do Rust trả về,
            không tính lại phía webview.
          -->
          <p v-else-if="lookupResolved && lookupRoute !== null && everySourceOffForRoute(lookupRoute)" class="lookup-empty">
            {{ t('panel.lookup.all_sources_off') }}
          </p>
          <p v-else-if="lookupResolved && notFound" class="lookup-empty">{{ t('panel.lookup.not_found') }}</p>
          <template v-else-if="lookupDisplayable">
            <!-- AC5 — dòng dẫn bất đồng ĐỨNG TRƯỚC khi liệt kê các khối nguồn. -->
            <p v-if="disagree" class="lookup-disagree">{{ t('panel.lookup.sources_disagree') }}</p>
            <LookupRecord
              v-for="group in groups"
              :key="`${group.layer}:${group.source.code}`"
              :group="group"
              :senses="sensesFor(group)"
            />
          </template>
        </template>
        </template>

        <!--
          ═══════════════════════════════════════════════════════════════════════════
          🔴 STORY 1.20 — TAB LỊCH SỬ: hai mục, hai trạng thái rỗng KHÁC NHAU (AC8)
          ═══════════════════════════════════════════════════════════════════════════

          AD-44 ④ (`ARCHITECTURE-SPINE.md:622`): *"rỗng im lặng bị cấm; rỗng có lý do thì
          không."* Lịch sử rỗng và danh sách ghim rỗng là **hai câu riêng**, không một khung
          trắng chung — và bộ ghim còn có thêm hai ca mà mockup không vẽ (chưa mở Tác phẩm ·
          nạp trượt), mỗi ca một vị từ riêng ở `lookupHistoryState.ts` (Bẫy 4).
        -->
        <template v-else>
          <div class="lookup-section">
            <span class="lookup-section-label">{{ t('panel.lookup.section_pinned') }}</span>
            <!-- aura-allow-text: số mục đã ghim — DỮ LIỆU dẫn xuất, không chuỗi giao diện. -->
            <span class="lookup-section-count">{{ pinnedEntries.length }}</span>
          </div>

          <!--
            🔴 BA vị từ, KHÔNG một chuỗi `??`. Thứ tự là nội dung: *nạp trượt* đứng trước
            *chưa ghim gì*; và khi cả hai `false` mà danh sách rỗng thì đó là **đang nạp** —
            panel nói ĐÚNG một thứ: không gì cả.

            ⚠️ Bản đầu có **bốn** nhánh; trạng thái *"chưa mở Tác phẩm nào"* biến mất cùng
            phạm vi Tác phẩm ở lượt Ice ký lại 2026-08-11 (ghim nay ở `global.db`). Câu đó
            **sai** ở đây, nên nó bị gỡ khỏi cả `vi.json` chứ không để lại làm khoá chết.
          -->
          <p v-if="pinnedLoadFailed" class="lookup-empty">
            {{ t('panel.lookup.pinned_load_failed') }}
          </p>
          <template v-else-if="pinnedIsEmpty">
            <p class="lookup-empty-title">{{ t('panel.lookup.pinned_empty_title') }}</p>
            <p class="lookup-empty">{{ t('panel.lookup.pinned_empty_body') }}</p>
            <p class="lookup-empty-note">{{ t('panel.lookup.pinned_empty_note') }}</p>
          </template>

          <div
            v-for="entry in pinnedEntries"
            :key="entry.id"
            class="lookup-row is-pin"
            :data-entry-key="entryKey(entry.source_code, entry.entry_id)"
          >
            <!-- aura-allow-text: đầu mục đã ghim — DỮ LIỆU (`EntryHit.headword` lúc ghim). -->
            <span class="lookup-row-word">{{ entry.headword }}</span>
            <!-- aura-allow-text: nghĩa rút gọn — DỮ LIỆU từ điển (ảnh chụp lúc ghim). -->
            <span v-if="entry.gloss !== null" class="lookup-row-gloss">{{ entry.gloss }}</span>
            <span v-else class="lookup-row-gloss">{{ t('panel.lookup.history_no_gloss') }}</span>
            <!-- aura-allow-text: tên nguồn — DỮ LIỆU (`display_name`, FR31). -->
            <span class="lookup-row-source">{{ sourceLabel(entry.source_code) }}</span>
            <!-- aura-allow-text: số lần tra CỦA PHIÊN NÀY — DỮ LIỆU dẫn xuất (§⑨). -->
            <span class="lookup-row-count">{{ sessionLookupCount(entry.source_code, entry.entry_id) }}</span>
            <button
              type="button"
              class="lookup-pin"
              @click="dispatch('lookup.toggle_pin')"
            >{{ t('panel.lookup.unpin') }}</button>
          </div>

          <div class="lookup-section">
            <span class="lookup-section-label">{{ t('panel.lookup.section_recent') }}</span>
            <!--
              🔴 Quyết định #5 — thanh bộ lọc ba chip của mockup (`:106-111`) bị LOẠI: hai
              chip *"Chương 47"* và *"Cả Tác phẩm"* mâu thuẫn trực tiếp với AC4 (*"lịch sử
              của phiên kết thúc"*), và một bộ lọc phạm vi trên một tập dữ liệu chỉ có đúng
              một phạm vi là một hứa hẹn rỗng. Nút xoá thì có nghĩa thật, nên nó ở lại.
            -->
            <button
              type="button"
              class="lookup-clear"
              @click="dispatch('lookup.clear_history')"
            >{{ t('panel.lookup.clear_history') }}</button>
          </div>

          <template v-if="historyIsEmpty">
            <p class="lookup-empty-title">{{ t('panel.lookup.history_empty_title') }}</p>
            <p class="lookup-empty">{{ t('panel.lookup.history_empty_body') }}</p>
            <p class="lookup-empty-note">{{ t('panel.lookup.history_empty_note') }}</p>
          </template>

          <!--
            🔴 AC1 — `lookupHistory` đã ở thứ tự **gần nhất trước**; không `sort()` ở đây.
            AC7 — khoá `:key` là `query`, và đó là chính khoá dedupe: một truy vấn có đúng
            MỘT hàng, và tra lại nó đẩy hàng đó lên đầu thay vì thêm hàng thứ hai.
          -->
          <div
            v-for="row in lookupHistory"
            :key="row.query"
            class="lookup-row"
            :class="{ 'is-current': row.query === currentQuery }"
          >
            <!-- aura-allow-text: truy vấn đã tra — DỮ LIỆU người dùng chọn. -->
            <span class="lookup-row-word">{{ row.query }}</span>
            <!-- aura-allow-text: nghĩa rút gọn — DỮ LIỆU từ điển. -->
            <span v-if="row.gloss !== null" class="lookup-row-gloss">{{ row.gloss }}</span>
            <span v-else class="lookup-row-gloss">{{ t('panel.lookup.history_no_gloss') }}</span>
            <!--
              ⚠️ Thời gian tương đối dựng từ một **khoá + tham số** của `vi.json`, không từ
              `Intl.RelativeTimeFormat`: NFR16 nói mọi văn bản giao diện sống ở `vi.json` và
              CHỈ ở đó. Nhãn tính lại mỗi lượt render — nó KHÔNG có đồng hồ riêng, nên
              *"vừa xong"* chỉ thành *"1 ph"* ở lượt tra kế tiếp. Ghi vào `deferred-work.md`.
            -->
            <span class="lookup-row-when">{{ t(relativeTimeKey(row.at), relativeTimeParams(row.at)) }}</span>
          </div>

          <!--
            Dòng gợi ý cuối tab — nguyên văn `lookup-history-pins.html:157-158`. Nó là một
            **quy tắc hành vi viết thành chữ cho người dùng**, không trang trí: nó nói ra
            chính khác biệt AC3/AC4 mà cả story đứng lên.
          -->
          <p class="lookup-hint">{{ t('panel.lookup.history_hint') }}</p>
        </template>
      </div>
    </div>
  </PanelFrame>
</template>

<style scoped>
/*
 * 🔴 STORY 1.19 — DẢI CHIP NGUỒN: một hàng RIÊNG, `flex: none`, KHÔNG một pixel nào vào
 * `--lookup-head-height`. Xem lý lẽ đo được ở chú thích `<template>` (Bẫy 4).
 *
 * 🔴 **BA VÙNG, KHÔNG MỘT — Ice chốt 2026-08-10 sau khi chạy thật.** Hộp ngoài **không** cắt
 * gì cả; chỉ vùng chip ở giữa mới có trần và mới cuộn. Nhãn và nút *"Nguồn dữ liệu"* đều
 * `flex: none`, nên chúng **không bao giờ** bị đẩy ra khỏi tầm nhìn.
 *
 * Bản đầu gộp cả ba vào một hộp `flex-wrap` có `overflow: hidden`, và chú thích cũ biện hộ
 * *"hàng thứ ba tới được bằng bàn phím"*. Ảnh chụp app thật với mười nguồn bác nó bằng số:
 * thứ bị cắt là **nút mở Attribution** *(con cuối trong dải ⇒ đường chuột duy nhất vào AC11)*
 * cộng hai chip cuối, mà chip thứ 9 là **Trần Văn Chánh** — nguồn `copyrighted` có cảnh báo
 * pháp lý, tức đúng nguồn người dùng cần tắt được nhất. *"Tới được bằng Tab"* không cứu được
 * một nút mà người dùng chuột không nhìn thấy.
 */
.lookup-sources {
  display: flex;
  align-items: center;
  flex: none;
  gap: 6px;
  padding-bottom: var(--space-panel-block);
  margin-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

/*
 * Vùng chip: nơi DUY NHẤT được phép tràn, và nó **cuộn** chứ không nuốt.
 *
 * ⚠️ `overflow-y: auto`, không `hidden`: một chip bị cắt khỏi tầm nhìn mà không để lại dấu
 * vết nào là đúng thứ vừa giấu mất hai nguồn thật. Thanh cuộn là dấu vết đó.
 * ⚠️ `min-width: 0` bắt buộc trên một con flex có nội dung tràn — thiếu nó, hộp con lấy
 * `min-content` làm chiều rộng tối thiểu và đẩy nút *"Nguồn dữ liệu"* ra ngoài mép, tức
 * dựng lại đúng lỗi vừa vá bằng một cơ chế khác.
 */
.lookup-sources-chips {
  display: flex;
  align-items: center;
  flex: 1 1 auto;
  min-width: 0;
  flex-wrap: wrap;
  gap: 6px;
  max-height: 52px;
  overflow-y: auto;
}

.lookup-sources-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

/*
 * 🔴 CHIP NGUỒN — `primary` là màu của **nhãn nguồn từ điển**, một trong đúng ba việc mà
 * `DESIGN.md` §Do's dành cho nó (cùng vai với `.lookup-spine-chip`).
 *
 * ⚠️ `<button>` chứ không `<span @click>`: NFR17 đòi mọi thao tác gọi được bằng bàn phím,
 * và một `<span>` không vào được thứ tự Tab ở bất kỳ trình duyệt nào. Mỗi chip là **một
 * điểm dừng Tab** — số điểm dừng mới khai ra thành số ở §Completion Notes (AC11).
 *
 * KHÔNG `outline: none` — focus ring của trình duyệt là nửa còn lại của NFR17 (§Trap 4 của
 * Story 1.6 giới hạn `outline: none` cho gốc `tabindex="-1"` của chế độ và panel).
 */
.source-chip {
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid transparent;
  cursor: pointer;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-primary);
}

/*
 * 🔴 UX-DR6 — TRẠNG THÁI TẮT PHÂN BIỆT BẰNG **MÀU + GẠCH NGANG**, KHÔNG BẰNG `opacity`.
 *
 * `DESIGN.md:216-219` cấm làm mờ **chữ** ở trạng thái **NGHỈ**, và một chip tắt là một
 * trạng thái nghỉ — nó đứng đó cho tới khi người dùng bấm lại. Hai tín hiệu chứ không một:
 * màu một mình không đọc được với người mù màu, và `text-decoration` một mình mờ nhạt ở cỡ
 * chữ `ui-label`. Cả hai đều là thuộc tính của **NÉT VÀ MÀU CHỮ**, không phải độ đục.
 */
.source-chip.off {
  color: var(--color-on-surface-variant);
  text-decoration: line-through;
}

/* Đường vào màn hình ghi công — một thao tác phụ, nên nó mang màu chữ phụ chứ không `primary`. */
/*
 * 🔴 `flex: none` + `white-space: nowrap` — nút này **không bao giờ** được co lại hay bị đẩy
 * đi: nó là đường chuột DUY NHẤT vào màn hình Attribution (AC11), và một lượt chạy thật ngày
 * 2026-08-10 đã cho thấy nó là thứ đầu tiên biến mất khi dải chip tràn.
 *
 * ⚠️ `margin-left: auto` gỡ đi: nó có nghĩa khi nút là con của một hộp `flex-wrap` chung với
 * chip. Nay vùng chip đã là một con `flex: 1 1 auto` riêng và tự đẩy nút sang phải, nên
 * `auto` ở đây chỉ thừa.
 */
.lookup-sources-attr {
  padding: 0;
  flex: none;
  white-space: nowrap;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

/*
 * 🔴 STORY 1.18 · AC7 — `.lookup-body` **thôi cuộn**; vùng cuộn là `.lookup-scroll`.
 *
 * `overflow: auto` ở đây (bản 1.17) làm `.lookup-head` cuộn đi cùng nội dung, nên đầu mục
 * trôi mất khi đọc một bản ghi dài — xem chú thích ở `<template>`.
 */
.lookup-body {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

/*
 * Vùng cuộn thật. `scrollTop = 0` của AC7 đặt lên phần tử NÀY.
 *
 * 🔴 không `scroll-behavior: smooth` ở đây hay bất kỳ đâu trong `src/**`.
 *
 * 🔵 **SỬA 2026-08-18 (Story 2.10) — lý do trích dẫn ở đây ĐÃ HẾT ĐÚNG.** Bản cũ viết
 * *"`DESIGN.md:342` cấm cho cả ứng dụng"*; dòng 342 hôm nay nói về chiều rộng `ch` của Chế độ
 * đọc, và `grep -rn "scroll-behavior"` trên toàn `_bmad-output/` trả **0 kết quả**. Nguồn thật
 * cho AC7 là `DESIGN.md:373`, bảng Motion **của Panel Lookup** — đủ cho tệp này, **không** cho
 * cả ứng dụng. Khối lý do đầy đủ ở chỗ đặt `scrollTop = 0`.
 *
 * ⚠️ ⇒ Mệnh đề *"không ở bất kỳ đâu trong `src/**`"* là một **quy ước**, không một luật có cổng
 * canh. Story 2.10 nhận nó làm tiền đề cho AC8 của `GridPanel.vue` và ghi rõ chỗ hở đó tại chỗ.
 */
.lookup-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

/*
 * 🔴 AC5 — HIỆU ỨNG NỘI DUNG MỚI VÀO: 90 ms · `opacity` 0.4 → 1 · `ease-out`.
 *
 * không `translate`, không `scale` (`DESIGN.md:341` cấm cho cả ứng dụng). không hiệu ứng RA — nội
 * dung cũ bị thay THẲNG.
 *
 * 🔴 **Vì sao 0.4 → 1 không vi phạm UX-DR6:** luật `opacity` của `DESIGN.md:216-219` cấm làm
 * mờ **chữ** ở trạng thái **NGHỈ**. Đây là một **QUÁ ĐỘ** — `opacity` đi giữa 0.4 và 1 rồi
 * dừng ở **1**, không đậu lại ở một mức trung gian nào. Chữ mờ thường trực mới là UX-DR6 vỡ.
 *
 * ⚠️ **Cùng 90 ms cho CẢ bốn trạng thái rỗng của AC6 Story 1.17** — chúng nằm trong chính
 * `.lookup-scroll` này, nên chúng nhận cùng hiệu ứng mà không cần một luật thứ hai.
 * `DESIGN.md:347`: một trạng thái rỗng hiện **chậm hơn** sẽ bị đọc thành *"đang tìm tiếp"*.
 */
.lookup-fade {
  animation: lookup-fade-in 90ms ease-out;
}

/*
 * 🔴 Vì sao `0.4` đi qua được Kiểm D của `check:tokens` (AC4 — cấm `opacity` trung gian):
 * luật `DESIGN.md:216-219` cấm làm mờ **CHỮ** ở trạng thái **NGHỈ** (UX-DR6). Giá trị dưới
 * đây là mốc ĐẦU của một **QUÁ ĐỘ 90 ms** — `opacity` đi 0.4 → 1 rồi dừng ở **1**, và không
 * tồn tại một khung hình nào sau 90 ms mà nó còn dưới 1. Chính `DESIGN.md` §Motion đặc tả
 * con số 0.4 này **bằng chữ**, trong cùng bảng chín hàng mà AC5–AC9 dẫn ra.
 *
 * ⚠️ Dấu miễn trừ phải nằm trong **±1 dòng** của khai báo (`check-tokens.mjs:612`), nên nó
 * là một dòng riêng sát ngay dưới, không phải khối lý lẽ này.
 */
@keyframes lookup-fade-in {
  from {
    /* aura-allow-opacity: mốc đầu của quá độ 90 ms, không một trạng thái nghỉ — xem trên. */
    opacity: 0.4;
  }
  to {
    opacity: 1;
  }
}

/*
 * 🔴 AC8 — VẠCH TIẾN TRÌNH MẢNH, KHÔNG SPINNER.
 *
 * Neo TUYỆT ĐỐI ở đáy `.lookup-head` (xem `position: relative` dưới đây) ⇒ nó không góp một
 * pixel nào vào chiều cao 76px. Bẫy 6.
 *
 * ⚠️ Màu từ **token** `ornament` — UX-DR5: `ornament` là màu của **NÉT**, và vạch này LÀ
 * một nét. `primary` bị `DESIGN.md` §Do's dành cho đúng ba việc (thuật ngữ Glossary, nhãn
 * nguồn, tiêu điểm bàn phím), và một vạch tiến trình không nằm trong ba việc đó.
 * KHÔNG màu viết thẳng (AD-34 §3, Kiểm B của `check:tokens`).
 */
.lookup-progress {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 2px;
  overflow: hidden;
}

/* Một NÉT chạy qua — không quay, không nhấp nháy, không đổi độ đục. */
.lookup-progress::after {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  width: 30%;
  background-color: var(--color-ornament);
  animation: lookup-progress-slide 1100ms ease-in-out infinite;
}

@keyframes lookup-progress-slide {
  from {
    left: -30%;
  }
  to {
    left: 100%;
  }
}

/*
 * 🔴 AC9 — `prefers-reduced-motion` ⇒ BỎ TOÀN BỘ hiệu ứng, đổi TỨC THÌ.
 *
 * ⚠️ **CSS thuần, không `window.matchMedia`** (Quyết định #6): ① `matchMedia` không có trong
 * `ALLOWED_GLOBAL_MEMBERS` và §⑥ của Story 1.17 đòi giữ số đó ở **0** cho Story 4.12;
 * ② CSS phản ứng **tức thì** khi người dùng đổi thiết lập hệ điều hành giữa phiên, một
 * `matchMedia` đọc một lần thì không; ③ `DESIGN.md:348` gọi đây là **sàn khả năng tiếp cận**,
 * và một sàn không nên phụ thuộc một nhánh JavaScript chạy đúng.
 *
 * 🔴 Vạch tiến trình **vẫn HIỆN nhưng TĨNH** — nó là một **thông tin** (*"lượt tra này
 * đang lâu"*), không một trang trí. Bỏ hẳn nó là biến một trạng thái có tên thành im lặng,
 * đúng thứ UX-DR27 cấm.
 */
@media (prefers-reduced-motion: reduce) {
  .lookup-fade {
    animation: none;
    opacity: 1;
  }

  .lookup-progress::after {
    animation: none;
    left: 0;
    width: 100%;
  }
}

/*
 * 🔴 AC7 / Quyết định #6 — hằng CÓ TÊN cho chiều cao, không một con số rải trong CSS.
 * `overflow: hidden` là điều kiện CƠ HỌC của "không đổi một pixel": nhiều nguồn có thể làm
 * thanh nhịp tràn quá một dòng (`flex-wrap: wrap`), và cắt bằng `overflow` giữ chiều cao
 * bất biến thay vì để nội dung tự ý đẩy khung xuống.
 */
.lookup-head {
  --lookup-head-height: 76px;
  flex: none;
  height: var(--lookup-head-height);
  overflow: hidden;
  /*
   * 🔴 STORY 1.18 · AC8 — điều kiện để `.lookup-progress` neo TUYỆT ĐỐI vào đáy khối này.
   *
   * ⚠️ Đây là **thay đổi DUY NHẤT** của story này lên `.lookup-head`, và nó không chạm một
   * pixel nào: `position: relative` không đổi bố cục của một khối đã có `height` cố định.
   * `--lookup-head-height` giữ NGUYÊN giá trị **và** vai trò; `overflow: hidden` ở lại.
   */
  position: relative;
  padding-bottom: var(--space-panel-block);
  margin-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

/* `lookup-headword` khai `wraps: false` (24px/1.3) — đầu mục dài CẮT bằng ellipsis. */
.lookup-headword {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--face-lookup-headword);
  font-size: var(--font-lookup-headword);
  line-height: var(--leading-lookup-headword);
  color: var(--color-on-surface);
}

.lookup-spine {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 7px;
}

.lookup-spine-count,
.lookup-spine-chip {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.lookup-spine-chip {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-primary);
  text-transform: uppercase;
}

/* Nguồn bị trần cắt SẠCH — có tên, có số, nhưng không nghĩa nào lấy về (FR31/AC12). */
.lookup-spine-chip.is-hidden {
  color: var(--color-on-surface-variant);
}

.lookup-spine-chip i {
  font-style: normal;
  font-family: var(--face-ui-sm);
  color: var(--color-on-surface-variant);
  text-transform: none;
  letter-spacing: normal;
  margin-left: 4px;
}

/*
 * Bốn trạng thái rỗng + hai banner — khai token thứ 17 `ui-md-wrap` (Quyết định #7):
 * đây là NGƯỜI TIÊU THỤ MỚI đầu tiên của token, cùng lượt với ba chỗ dùng cũ được vá.
 */
.lookup-empty,
.lookup-banner,
.lookup-disagree {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.lookup-disagree {
  padding-left: 11px;
  border-left: 2px solid var(--color-tm-rule);
  color: var(--color-tm-text);
}

/*
 * ═══════════════════════════════════════════════════════════════════════════════════
 * 🔴 STORY 1.20 — DẢI TAB, HAI MỤC CỦA TAB LỊCH SỬ, VÀ HÌNH DẠNG MỘT HÀNG
 * ═══════════════════════════════════════════════════════════════════════════════════
 *
 * 🔴 **AC10 — dải tab là một hàng RIÊNG, `flex: none`, KHÔNG một pixel nào vào
 * `--lookup-head-height`.** Đó là hằng đã vỡ HAI lần (1.17 với thanh nhịp, 1.18 với vạch
 * tiến trình) và đã phải nhường một lần nữa ở 1.19 (dải chip). Đo lại thay vì nới nó.
 */
.lookup-tabs {
  display: flex;
  align-items: center;
  flex: none;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

/* Cùng khai token với `.tab` của `GridPanel.vue` — một dải tab, một hình dạng. */
.lookup-tab {
  appearance: none;
  border: none;
  background: none;
  padding: 0;
  cursor: pointer;
  font-family: var(--face-ui-md-strong);
  font-size: var(--font-ui-md-strong);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-md-strong);
  color: var(--color-on-surface-variant);
}

/* `primary` cho tab đang chọn — **tiêu điểm bàn phím** là một trong đúng ba việc mà
 * `DESIGN.md` §Do's dành cho màu này, và một tab đang chọn là chỗ tiêu điểm đang ở. */
.lookup-tab.active {
  color: var(--color-primary);
}

/* Đầu mục *Đã ghim* / *Tra gần đây* — một hàng nhãn cộng một thao tác bên phải. */
.lookup-section {
  display: flex;
  align-items: baseline;
  gap: var(--space-panel-inline);
  margin: var(--space-panel-block) 0 var(--space-panel-block) 0;
  padding-bottom: 5px;
  border-bottom: 1px solid var(--color-outline-faint);
}

.lookup-section-label {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.lookup-section-count,
.lookup-clear {
  margin-left: auto;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.lookup-clear {
  appearance: none;
  border: none;
  background: none;
  padding: 0;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
}

/*
 * 🔴 HÌNH DẠNG CHỦ ĐẠO LÀ **VẠCH DỌC**, không hộp bo tròn (`DESIGN.md:354`).
 *
 * Hàng ghim mang vạch `tm`; hàng lịch sử ĐANG XEM mang vạch `primary` — và hai màu khác
 * nhau là **ngữ nghĩa**, không thẩm mỹ: `DESIGN.md` §Do's dành `primary` cho đúng ba việc
 * (thuật ngữ Glossary, nhãn nguồn từ điển, tiêu điểm bàn phím), nên vạch của một mục ghim
 * — thứ không thuộc ba việc đó — dùng `tm`.
 *
 * Giãn dòng 1.66 là sàn của chữ họ `read` (`DESIGN.md:297`) và `ui-md-wrap` đã khai đúng số
 * đó — dùng lại token, không một con số rải trong CSS.
 */
.lookup-row {
  display: flex;
  align-items: baseline;
  gap: var(--space-panel-inline);
  padding: 2px 0 2px 11px;
  border-left: 2px solid transparent;
}

.lookup-row.is-pin {
  border-left-color: var(--color-tm-rule);
}

/* Hàng của truy vấn ĐANG HIỆN ở tab Từ điển — nền nhấn cộng vạch `primary`. */
.lookup-row.is-current {
  border-left-color: var(--color-primary);
  background-color: var(--color-surface-accent);
}

/* Đầu mục — họ `read`, và nó KHÔNG co lại khi nghĩa dài. */
.lookup-row-word {
  flex: none;
  min-width: 74px;
  font-family: var(--face-lookup-gloss);
  font-size: var(--font-lookup-gloss);
  line-height: var(--leading-lookup-gloss);
  color: var(--color-on-surface);
}

/*
 * 🔴 Nghĩa rút gọn: **một dòng, cắt bằng ellipsis**. Một danh sách mà mỗi hàng cao một
 * kiểu thì mắt không quét dọc được, và đó là cả công dụng của tab này.
 *
 * ⚠️ `min-width: 0` bắt buộc trên một con flex có nội dung tràn — thiếu nó, hộp con lấy
 * `min-content` làm chiều rộng tối thiểu và đẩy cột nguồn ra ngoài mép (đúng lỗi mà dải
 * chip nguồn đã vá ở Story 1.19).
 */
.lookup-row-gloss {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

/* Nhãn nguồn — một trong đúng ba việc mà `DESIGN.md` §Do's dành cho `primary`. */
.lookup-row-source {
  flex: none;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-primary);
}

.lookup-row-count,
.lookup-row-when {
  flex: none;
  text-align: right;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.lookup-row-count {
  min-width: 34px;
}

.lookup-row-when {
  min-width: 56px;
}

/* Nút bỏ ghim trên một hàng ghim — cùng hình dạng nút ghim của `LookupRecord.vue`. */
.lookup-pin {
  flex: none;
  padding: 0;
  white-space: nowrap;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

/* Tiêu đề của một trạng thái rỗng — khai token `ui-md-strong`, cùng cấp với nhãn tab. */
.lookup-empty-title {
  margin: var(--space-panel-block) 0 4px 0;
  font-family: var(--face-ui-md-strong);
  font-size: var(--font-ui-md-strong);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-md-strong);
  color: var(--color-on-surface);
}

.lookup-empty-note,
.lookup-hint {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

/* Dòng gợi ý cuối tab — một quy tắc hành vi, nên nó có nét ngăn của riêng nó. */
.lookup-hint {
  margin-top: var(--space-panel-block);
  padding-top: var(--space-panel-block);
  border-top: 1px solid var(--color-outline-faint);
}
</style>
