<script setup lang="ts">
// Panel `Tra cứu` — Story 1.17. Nội dung THẬT (bản ghi có cấu trúc), ⛔ không còn là khung
// trống của Story 1.14.
//
// ⚠️ UX-DR15 xếp `Tra cứu` vào nhóm ĐƯỢC nhường chỗ — nhưng *"rút về thanh trạng thái,
// ⛔ không bao giờ mất hẳn"*. Vế "thanh trạng thái" là **Story 4.12**; story này chỉ giao
// cơ chế ẩn (`SACRIFICE_ORDER` ở `src/layout/workspaceLayout.ts`), ⛔ không đụng ở đây.
//
// 🔴 §⛔KHÔNG-LÀM ① — đường kích hoạt là MỘT PHÍM tường minh (`lookup.lookup_selection`,
// Quyết định #1a), ⛔ không phải Auto-Lookup. ⛔ Không hiệu ứng 90ms, ⛔ không vạch tiến
// trình 250ms — cả hai thuộc Story 1.18.
import { computed } from 'vue'
import PanelFrame from './PanelFrame.vue'
import LookupRecord from './LookupRecord.vue'
import type { DockviewPanelProps } from '../layout/panelProps'
import { t } from '../i18n'
import {
  currentQuery,
  groupedLookup,
  layersLoaded,
  lookupDisplayable,
  lookupError,
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

// AC7 — ⛔ spinner, ⛔ "đang tải". `lookupPending`/`lookupResolved` tồn tại để bốn trạng
// thái của AC6 ⛔ nháy sang "không tìm thấy" trong khoảng chờ IPC (cùng bẫy đã bắt ở 1.16),
// nhưng bề mặt này ⛔ vẽ một chỉ báo chờ nào — panel giữ nguyên trạng thái TRƯỚC ĐÓ cho tới
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
 * 🔴 AC6 — **một lượt tra TRƯỢT phải nói ra**. `lookupError` từng là một export ⛔ ai tiêu
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
</script>

<template>
  <PanelFrame owner="panel.lookup" status-key="panel.lookup.status" :show-status="showFrameStatus">
    <div class="lookup-body">
      <!--
        🔴 AC7 — vùng đầu mục CHIỀU CAO CỐ ĐỊNH, ⛔ đổi một pixel **kể cả giữa bốn trạng
        thái của AC6**. Vì thế nó render ở MỌI trạng thái, kể cả "chưa tra gì": bản đầu bọc
        cả khối trong `v-if="!neverLookedUp"` nên chiều cao đi 0 → 76px đúng lúc chuyển
        trạng thái, tức hằng có tên thì có mà bất biến nó tồn tại để giữ thì ⛔ (bắt ở code
        review 2026-08-07).
      -->
      <div class="lookup-head">
        <!-- aura-allow-text: truy vấn vừa tra — DỮ LIỆU người dùng chọn, ⛔ chuỗi giao diện. -->
        <p class="lookup-headword">{{ currentQuery }}</p>
        <div v-if="lookupDisplayable && groups.length > 0" class="lookup-spine">
          <!--
            🔴 AC12 — thanh nhịp ⛔ **khẳng định một con số nó ⛔ biết**. Khi trần `LIMIT`
            đã cắt, `senseCountAtLeast` bật và số đọc ra như một CẬN DƯỚI (`≥`). Số nguồn
            thì luôn ĐẦY ĐỦ — nguồn bị cắt sạch vẫn được đếm qua `hidden_sources`.
          -->
          <span class="lookup-spine-count">
            <!-- aura-allow-text: số đếm dẫn xuất từ dữ liệu, ⛔ chuỗi giao diện tĩnh. -->
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
          <!-- 🔴 FR31 — nguồn bị trần cắt SẠCH vẫn phải được GỌI TÊN, ⛔ chỉ đếm. -->
          <span v-for="h in spine.hidden" :key="`hidden-${h.displayName}`" class="lookup-spine-chip is-hidden">
            <!-- aura-allow-text: tên nguồn bị giấu — DỮ LIỆU (`display_name`). -->
            {{ h.displayName }}
            <!-- aura-allow-text: số đầu mục của nguồn bị giấu — DỮ LIỆU. -->
            <i>{{ h.count }}</i>
          </span>
        </div>
      </div>

      <!-- 🔴 AC6 — một lượt tra TRƯỢT phải nói ra, ⛔ để lại một vùng trắng câm. -->
      <p v-if="showLookupError" class="lookup-empty">{{ t('panel.lookup.lookup_failed') }}</p>

      <template v-else-if="!neverLookedUp">
        <p v-if="someLayerFailed" class="lookup-banner">{{ t('panel.lookup.some_layer_failed') }}</p>
        <p v-if="someLayerTruncated" class="lookup-banner">{{ t('panel.lookup.list_incomplete') }}</p>
        <!-- Trần ĐỘ DÀI đã cắt truy vấn — ⛔ được đọc thành "⛔ tìm thấy" (câu đó SAI ở đây). -->
        <p v-if="queryWasTruncated" class="lookup-banner">{{ t('panel.lookup.query_truncated') }}</p>

        <!--
          Bốn trạng thái rỗng đọc `lookupResolved` (⛔ nháy trong khoảng chờ IPC); bản ghi
          và thanh nhịp đọc `lookupDisplayable` (⛔ chớp trắng). Hai vị từ, hai câu hỏi khác
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
  </PanelFrame>
</template>

<style scoped>
.lookup-body {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  overflow: auto;
}

/*
 * 🔴 AC7 / Quyết định #6 — hằng CÓ TÊN cho chiều cao, ⛔ một con số rải trong CSS.
 * `overflow: hidden` là điều kiện CƠ HỌC của "⛔ đổi một pixel": nhiều nguồn có thể làm
 * thanh nhịp tràn quá một dòng (`flex-wrap: wrap`), và cắt bằng `overflow` giữ chiều cao
 * bất biến thay vì để nội dung tự ý đẩy khung xuống.
 */
.lookup-head {
  --lookup-head-height: 76px;
  flex: none;
  height: var(--lookup-head-height);
  overflow: hidden;
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

/* Nguồn bị trần cắt SẠCH — có tên, có số, nhưng ⛔ nghĩa nào lấy về (FR31/AC12). */
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
