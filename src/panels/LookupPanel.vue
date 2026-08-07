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
  neverLookedUp,
  notFound,
  queryTooShort,
  queryWasTruncated,
  sensesByLayer,
  someLayerFailed,
  someLayerTruncated,
} from './lookupPanelState'
import { computeSpine, sourcesDisagree } from './lookupPanelState'
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
 */
const showFrameStatus = computed(() => neverLookedUp.value)

const groups = computed<readonly SourceGroup[]>(() => groupedLookup.value?.groups ?? [])
const hiddenSources = computed(() => groupedLookup.value?.hidden_sources ?? [])
const spine = computed(() => computeSpine(groups.value, sensesByLayer.value, hiddenSources.value))
const disagree = computed(() => sourcesDisagree(groups.value))

/**
 * 🔴 AC6 — **một lượt tra TRƯỢT phải nói ra**. `lookupError` từng là một export không ai tiêu
 * thụ, nên một lỗi IPC cho ra thân panel TRẮNG CÂM: `neverLookedUp` đã `false` (tắt câu
 * mặc định của `PanelFrame`) còn `lookupResolved` `false` (tắt cả kết quả lẫn bốn chuỗi
 * rỗng). Đúng khuyết tật code review 1.16 đã bắt ở `SourcePanel.vue` — tái phát nguyên
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
    // không `scroll-behavior` trong CSS (`DESIGN.md:342` cấm cả ứng dụng).
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
      <div ref="scroller" class="lookup-scroll" :class="{ 'lookup-fade': fading }">
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
      </div>
    </div>
  </PanelFrame>
</template>

<style scoped>
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
 * 🔴 không `scroll-behavior: smooth` ở đây hay bất kỳ đâu trong `src/**` — `DESIGN.md:342`
 * cấm cho cả ứng dụng, và AC7 đếm lại con số đó ở Task 9.
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
</style>
