<script setup lang="ts">
// Panel `Đề xuất AI`. Story 1.14 · AC1 · AC8 — **khung**, không phải nội dung.
//
// Bản dịch AI thật, chọn nhà cung cấp, và ba điểm ra mạng của AD-15 là **Epic 4**.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 "CHƯA CẤU HÌNH" KHÔNG PHẢI MỘT TRẠNG THÁI LỖI — UX-DR27 · FR77
// ─────────────────────────────────────────────────────────────────────────────────
// Panel này **MỜI CẤU HÌNH**. Không cảnh báo, không màu `error`, không dấu chấm
// than. Một người dùng chưa từng dán khoá API vào đâu thì không làm sai gì cả — vẽ
// một cảnh báo ở đây là dạy họ rằng ứng dụng đang hỏng.
//
// ⚠️ Câu trạng thái sống ở `vi.json` (`panel.ai_translation.status`) và Kiểm D của
// `check-i18n.mjs` chấm phần máy chấm được của UX-DR47 (không "chúng tôi", không
// "bạn"). Phần còn lại — giọng MỜI thay vì giọng CẢNH BÁO — là chỗ con người phải đọc.
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 1.18 · AC2 — ĐĂNG KÝ HỢP ĐỒNG VÙNG CHỌN, KHÔNG NỘI DUNG
// ─────────────────────────────────────────────────────────────────────────────────
// Panel này hôm nay **không có chữ**, và đó chính là lý do lượt đăng ký phải nằm ở đây NGAY
// BÂY GIỜ: FR21 đòi AI Translation *"nhận được cùng hành vi khi nó có nội dung
// ở các epic sau, **không cần cài lại**"*. Một lượt đăng ký thiếu ở đây không để lại **bất
// kỳ triệu chứng nào** cho tới Epic 4 — tức hai epic sau, và tới lúc đó không ai nhớ AC này
// tồn tại. Cổng đếm của `check-commands.mjs` (Kiểm F) là thứ giữ mệnh đề đó bằng MÁY.
//
// Đừng "dọn" `<div ref="surface">` vì nó trông trống: nó LÀ bề mặt mà Epic 4 sẽ đổ nội
// dung vào, và là phần tử mà hợp đồng đo `contains(anchorNode)` trên.
//
// 🔵 **2026-08-13 — mệnh đề "cùng hành vi" ở trên đã ĐƯỢC THU HẸP** (Sprint Change Proposal,
// Ice ký; FR21). Panel này sẽ mang **bản dịch AI tiếng Việt**, còn từ điển nhúng là
// zh→vi / en→vi ⇒ nó KHÔNG phải nguồn tra cứu: vai nay là `'display'`.
// Phần còn đúng của AC2 — và là phần đắt nhất — vẫn nguyên: hợp đồng KHÔNG phải sửa một
// dòng nào khi Epic 4 đổ nội dung vào. Chỉ **vai** khai lúc đăng ký quyết định hành vi.
//
import { computed, onMounted, useTemplateRef, watch } from 'vue'
import PanelFrame from './PanelFrame.vue'
import { useSelectionSurface } from './selectionContract'
import { dispatch } from '../commands'
import { providerMessageOf, t, tError } from '../i18n'
import { loadPromptSets, promptSets, selectedPromptSetName } from '../promptSetState'
import { shortcutRows } from '../config/shortcutsState'
import {
  editorActiveSegmentId,
  editorPendingPromote,
  editorPendingPromoteBatch,
  editorPromoteAiTranslationError,
  editorPromoteBatchReport,
  editorSegments,
} from './editorPanelState'
import {
  AI_PROVIDERS,
  aiConfigEffectiveModel,
  aiConfigEffectiveProvider,
  aiConfigNotReady,
  aiConfigReadiness,
  aiConfigReadinessWorkIsOpen,
  aiConfigWorkOverrides,
  refreshAiConfigReadiness,
} from '../aiConfigState'
import { aiUsageVisible } from '../aiUsageVisibleState'
import { clearAiPromptAssembleError, refreshAiPromptRecord } from '../aiPromptInspectorState'
import {
  aiTranslateAccumulatedText,
  aiTranslateError,
  aiTranslateMaybeTruncated,
  aiTranslateRunSegmentId,
  aiTranslateStateValue,
  aiTranslateUsage,
  isAiTranslateResultStale,
} from '../aiTranslateState'
// Story 4.9, Phase 3 — "Dịch theo LÔ với tiến độ và huỷ giữa chừng" (FR73, AD-22, AC2).
import {
  aiTranslateBatchDoneCount,
  aiTranslateBatchError,
  aiTranslateBatchErrorSpan,
  aiTranslateBatchMayRetry,
  aiTranslateBatchMissingCount,
  aiTranslateBatchNotSentCount,
  aiTranslateBatchPromotableCount,
  aiTranslateBatchRemainingCount,
  aiTranslateBatchRetryIds,
  aiTranslateBatchRows,
  aiTranslateBatchRunningCall,
  aiTranslateBatchStateValue,
  aiTranslateBatchTextForSegment,
  aiTranslateBatchTotalCount,
  aiTranslateBatchUsageSummary,
} from '../aiTranslateBatchState'
import type { AiTranslateBatchRow } from '../aiTranslateBatchState'
import type { AiTranslateUsageWire } from '../config/aitranslate'
import { chapterUntranslatedIds } from '../chapterTranslateScope'
import { aiUsageSummaryLine, formatUsdParam } from '../aiUsageLine'
import type { AiUsageSummary } from '../aiUsageLine'
import { segmentSelectionCount } from './segmentSelectionState'
import type { DockviewPanelProps } from '../layout/panelProps'

defineProps<DockviewPanelProps>()

const surface = useTemplateRef<HTMLElement>('surface')
// 🔴 ĐỪNG gỡ lời gọi này khi thấy vai là `'display'`. FR48 (Story 3.3) và FR60 (Story 7.7)
// đọc vùng chọn ở đây bằng lệnh của RIÊNG chúng; `'display'` tắt đúng MỘT đường —
// `currentSelectionText()`, tức đường tra TỪ ĐIỂN — chứ không tắt việc bề mặt được đăng ký.
// Ghim bằng máy: `check-commands.mjs` Kiểm F ③.
useSelectionSurface(surface, 'display', undefined, true)

// Nạp danh sách bộ prompt hai tầng khi panel dựng — cùng lý do `openGlossaryManage()` nạp
// lại mỗi lần lớp phủ Quản lý mở: bộ có thể vừa được tạo/xoá/đổi tên ở một phiên trước, hoặc
// một Tác phẩm khác vừa mở.
onMounted(() => {
  void loadPromptSets()
  // Story 4.7 — đồng bộ với bản ghi Rust đang giữ ngay lúc panel này mount: dockview có thể
  // tháo/dựng lại panel trong khi phiên vẫn còn một bản ghi từ một lượt Lắp trước đó.
  void refreshAiPromptRecord()
  void refreshAiConfigReadiness()
})

/**
 * ⚠️ **SỬA 2026-09-18, bắt được ở lượt rà soát build.** `aiPromptAssembleError` không mang
 * định danh câu — khác `aiPromptRecord` (§Always spec 4.7: bản ghi PHẢI mang định danh câu
 * tạo ra nó). Không có watcher này, một lỗi Lắp cho câu A đứng nguyên trên màn Prompt
 * sau khi tiêu điểm dời sang câu B mà người dùng chưa bấm Lắp lại — đọc như "câu B đang lỗi",
 * dù lỗi đó thuộc về A. Xem doc-comment [`clearAiPromptAssembleError`] cho đối chứng.
 */
watch(editorActiveSegmentId, () => {
  clearAiPromptAssembleError()
})

watch([aiTranslateStateValue, aiTranslateBatchStateValue], ([single, batch], [prevSingle, prevBatch]) => {
  const finished =
    (prevSingle === 'generating' && single !== 'generating') || (prevBatch === 'generating' && batch !== 'generating')
  if (finished) void refreshAiPromptRecord()
})

const promptSetHeadText = computed<string>(() => {
  const name = selectedPromptSetName.value ?? ''
  return providerModelText.value === null ? name : t('panel.ai_translation.head_set_after_provider', { name })
})

const selectChordText = computed<string>(
  () => shortcutRows.value.find((row) => row.id === 'segment.selection.extend_down')?.display.join(' / ') ?? '',
)

const chapterUntranslatedCount = computed<number>(() => chapterUntranslatedIds.value.length)

const chapterRunLabel = computed<string>(() =>
  chapterUntranslatedCount.value === 0
    ? t('command.ai.translate.chapter_run')
    : t('panel.ai_translation.chapter_run_count', { count: String(chapterUntranslatedCount.value) }),
)

const batchRunLabel = computed<string>(() =>
  segmentSelectionCount.value === 0
    ? t('command.ai.translate.batch_run')
    : t('panel.ai_translation.batch_run_count', { count: String(segmentSelectionCount.value) }),
)


/** `false` ⇔ không câu nào đang được chọn — chưa mở Tác phẩm, hoặc đã mở nhưng chưa đặt tiêu
 * điểm vào câu nào. Vô hiệu hoá nút Lắp ở ĐÂY (chỗ BIẾT trước), không dựa vào lưới phòng thủ
 * "kêu, không ném" của `assembleCurrentAiPrompt` — hai lớp, không chỉ một. */
const hasActiveSegment = computed<boolean>(() => editorActiveSegmentId.value !== null)

// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 4.8 — DỊCH THẬT + HUỶ + ĐƯA SANG BẢN DỊCH (FR72/FR74, AD-22, AD-47①/③)
// ─────────────────────────────────────────────────────────────────────────────────
// Cùng khuôn khối Story 4.7 ngay trên: vô hiệu hoá nút ở CHỖ BIẾT TRƯỚC, hai lớp cùng lưới
// "kêu, không ném" của `runAiTranslate`/`cancelAiTranslate`/`promoteAiTranslate` (`main.ts`).
//
// 🔴 SỬA Story 4.9, Phase 3 — cả hai computed dưới đây đọc THÊM `aiTranslateBatchStateValue`
// (khai ở khối Story 4.9 ngay dưới file này): một lượt LÔ đang chạy cũng phải khoá nút Dịch
// MỘT câu (cổng loại-trừ-lẫn-nhau, §Tasks spec 4.9 Phase 3), và nút Huỷ dùng CHUNG cho cả hai
// hình dạng lời gọi ("driven by whichever call is in flight") nên phải BẬT khi MỘT TRONG HAI
// đang chạy, không chỉ lượt đơn.
const canRunAiTranslate = computed<boolean>(
  () =>
    editorActiveSegmentId.value !== null &&
    aiTranslateStateValue.value !== 'generating' &&
    aiTranslateBatchStateValue.value !== 'generating',
)
const canCancelAiTranslate = computed<boolean>(
  () => aiTranslateStateValue.value === 'generating' || aiTranslateBatchStateValue.value === 'generating',
)
/** I/O Matrix spec 4.8 "Promote the result": `done`/`cancelled` VÀ văn bản không rỗng.
 *
 * 🔴 SỬA Story 4.9, Phase 3 — `main.ts`'s handler thật của `ai.translate.promote` (đúng nút
 * này) từ Story 4.9 cũng chốt được một hàng LÔ đang `done` tại câu có TIÊU ĐIỂM (Decision 3),
 * không chỉ kết quả lượt đơn. Thiếu nhánh đó ở đây thì hợp âm `Mod+Shift+Enter` chốt được
 * trong khi nút trên panel vẫn khoá — cùng hàm thuần `aiTranslateBatchTextForSegment` mà
 * `main.ts` dùng, không một phép kiểm thứ hai có thể trôi khỏi nó. */
const canPromoteAiTranslate = computed<boolean>(() => {
  if (
    !isAiTranslateStale.value &&
    (aiTranslateStateValue.value === 'done' || aiTranslateStateValue.value === 'cancelled') &&
    aiTranslateAccumulatedText.value !== ''
  ) {
    return true
  }
  const caretId = editorActiveSegmentId.value
  return caretId !== null && aiTranslateBatchTextForSegment(aiTranslateBatchRows.value, caretId) !== null
})
/** `true` ⇔ kết quả đang hiện được dịch cho một câu KHÁC câu đang có tiêu điểm bây giờ — cùng
 * khuôn [`canRunAiTranslate`] ngay trên, hàm thuần của `aiTranslateState.ts`. */
const isAiTranslateStale = computed<boolean>(() =>
  isAiTranslateResultStale(aiTranslateRunSegmentId.value, editorActiveSegmentId.value),
)

// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 4.11 — SỐ TOKEN VÀ ƯỚC TÍNH CHI PHÍ (Quyết định Ice 2026-09-22)
// ─────────────────────────────────────────────────────────────────────────────────
// Pure functions trả về KHOÁ + THAM SỐ, không bao giờ một câu tiếng Việt lắp tay — đúng khuôn
// `aiTranslateBatchRowStatusKey` ngay dưới VÀ tiền lệ `Intl.RelativeTimeFormat` đã bị từ chối
// ở `lookupHistoryState.ts:249-256` (NFR16): không `Intl.NumberFormat`/`toLocaleString`/
// `toFixed` NGUYÊN VĂN ở đây — [`formatUsdParam`] tự làm tròn bằng số học, không mượn API
// định dạng theo locale nào.

/** Một dòng hiển thị được — khoá `vi.json` cộng tham số nó cần, hoặc `null` khi KHÔNG có gì
 * để vẽ (không một dòng rỗng nào được render — template đọc `null` để bỏ hẳn phần tử DOM). */
type AiTranslateUsageLine = { key: string; params: Record<string, string> }

/**
 * `usage` của LƯỢT DỊCH MỘT SEGMENT (`aiTranslateState.ts::aiTranslateUsage`) → khoá + tham số
 * — ba trong năm hàng I/O Matrix spec 4.11 canh được ở tầng NÀY (hai hàng còn lại là của LÔ,
 * [`aiTranslateBatchUsageLine`] ngay dưới). `usage === null` ⇔ provider không trả về một khung
 * `usage` nào (I/O Matrix "Provider sends no usage") — chỗ gọi (computed `aiTranslateUsageLineValue`
 * ngay dưới) chỉ gọi hàm này khi `aiTranslateStateValue === 'done'`; một lượt `cancelled`/
 * `error`/`generating` không có dòng usage nào cả (§I/O Matrix spec 4.11 "Cancelled mid-flight":
 * "no figure for that call").
 */
function aiTranslateUsageLine(usage: AiTranslateUsageWire | null): AiTranslateUsageLine {
  if (usage === null) {
    return { key: 'ai.translate.usage_unavailable', params: {} }
  }
  if (usage.cost_usd === null) {
    return { key: 'ai.translate.usage_no_price', params: { token_count: String(usage.total_tokens) } }
  }
  return {
    key: 'ai.translate.usage_with_cost',
    params: { token_count: String(usage.total_tokens), cost_usd: formatUsdParam(usage.cost_usd) },
  }
}

/** Dòng usage của lượt dịch MỘT segment — `null` (không vẽ gì) trừ khi lượt đã `'done'` (xem
 * doc-comment [`aiTranslateUsageLine`]). */
const aiTranslateUsageLineValue = computed<AiTranslateUsageLine | null>(() =>
  aiTranslateStateValue.value === 'done' ? aiTranslateUsageLine(aiTranslateUsage.value) : null,
)

/**
 * Tổng số liệu của MỘT LÔ (`aiTranslateBatchState.ts::aiTranslateBatchUsageSummary`) → khoá +
 * tham số — hai hàng LÔ còn lại của I/O Matrix spec 4.11: "Batch completes" (mọi câu đã dịch
 * đều báo số ⇒ `batch_usage_total*`) và "Batch partly without usage" (chỉ MỘT PHẦN báo số ⇒
 * `batch_usage_partial*`, nêu rõ bao nhiêu câu — §Always: "never silently sum a subset as if
 * whole"). `null` khi CHƯA câu nào báo số (lô chưa xong câu nào, hoặc mọi câu đã xong đều
 * không mang usage) — không một dòng "0 token" giả nào được vẽ.
 *
 * FR76 (AC ký): *"hiển thị tổng token VÀ tổng ước tính của cả lô"* — cùng
 * khuôn lượt dịch MỘT segment ([`aiTranslateUsageLine`] ngay trên: MỘT khoá khi có giá, MỘT
 * khoá khi không), áp riêng cho từng trường hợp đầy-đủ/một-phần — bốn khoá, không hai, vì
 * "đầy đủ hay một phần" và "có giá hay không" là hai trục ĐỘC LẬP.
 */
function aiTranslateBatchUsageLine(summary: AiUsageSummary): AiTranslateUsageLine | null {
  return aiUsageSummaryLine(summary, {
    total: 'ai.translate.batch_usage_total',
    totalWithCost: 'ai.translate.batch_usage_total_with_cost',
    partial: 'ai.translate.batch_usage_partial',
    partialWithCost: 'ai.translate.batch_usage_partial_with_cost',
  })
}

const aiTranslateBatchUsageLineValue = computed<AiTranslateUsageLine | null>(() =>
  aiTranslateBatchUsageLine(aiTranslateBatchUsageSummary.value),
)

// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 4.9, PHASE 3 — "DỊCH THEO LÔ VỚI TIẾN ĐỘ VÀ HUỶ GIỮA CHỪNG" (FR73, AD-22)
// ─────────────────────────────────────────────────────────────────────────────────
// Cùng khuôn khối Story 4.8 ngay trên: vô hiệu hoá nút Ở ĐÂY (chỗ BIẾT TRƯỚC — vùng chọn
// rỗng, hoặc một lượt ĐƠN/LÔ khác đang chạy), lớp phòng thủ thứ hai sống trong
// `runAiTranslateBatch` chính nó (`aiTranslateBatchState.ts`, "kêu, không ném").
const canRunAiTranslateChapter = computed<boolean>(
  () =>
    chapterUntranslatedCount.value > 0 &&
    aiTranslateStateValue.value !== 'generating' &&
    aiTranslateBatchStateValue.value !== 'generating',
)

const canRunAiTranslateBatch = computed<boolean>(
  () =>
    segmentSelectionCount.value > 0 &&
    aiTranslateStateValue.value !== 'generating' &&
    aiTranslateBatchStateValue.value !== 'generating',
)

/** Khoá `vi.json` của nhãn một hàng, theo TRẠNG THÁI của hàng đó (`AiTranslateBatchRowStatus`,
 * `aiTranslateBatchState.ts`) — hàm THUẦN, một `switch` đóng thay vì nối chuỗi trong template
 * (Kiểm A2 của `check:i18n` chỉ chấp nhận `{{ t(...) }}`/`{{ tError(...) }}`, việc nối chuỗi
 * key nằm ở ĐÂY, không ở template). */
function aiTranslateBatchRowStatusKey(row: AiTranslateBatchRow): string {
  switch (row.status) {
    case 'queued':
      return 'panel.ai_translation.batch_row_status_queued'
    case 'streaming':
      return 'panel.ai_translation.batch_row_status_streaming'
    case 'received':
      return 'panel.ai_translation.batch_row_status_received'
    case 'missing':
      return 'panel.ai_translation.batch_row_status_missing'
    case 'not_sent':
      return 'panel.ai_translation.batch_row_status_not_sent'
  }
}

// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 4.10, PHASE 2 — NÚT "THỬ LẠI" (FR75, AD-22, Decision 2)
// ─────────────────────────────────────────────────────────────────────────────────
// `retryable` là thứ DUY NHẤT được cấp quyền HIỆN nút (§Always spec 4.10: "retryable grants
// only the right to SHOW a button — no timer, no loop, no automatic second call at any
// layer") — cả hai computed dưới đây gói TRỌN điều kiện đó thành MỘT chỗ, và cả hai nút dùng
// `v-if` (không `:disabled`) để nút biến mất hẳn khỏi DOM khi không retryable, thay vì hiện
// mờ. Cùng cổng loại-trừ-lẫn-nhau đã ghi cho `canRunAiTranslate`/`canRunAiTranslateBatch` ở
// trên: một lượt KHÁC (đơn hoặc lô) đang chạy thì cũng ẩn nút retry của module kia.
const canRetryAiTranslate = computed<boolean>(
  () =>
    !isAiTranslateStale.value &&
    aiTranslateStateValue.value === 'error' &&
    aiTranslateError.value !== null &&
    aiTranslateError.value.retryable === true &&
    aiTranslateBatchStateValue.value !== 'generating',
)
const canRetryAiTranslateBatch = computed<boolean>(
  () =>
    aiTranslateStateValue.value !== 'generating' &&
    aiTranslateBatchMayRetry(
      aiTranslateBatchStateValue.value,
      aiTranslateBatchError.value,
      aiTranslateBatchRetryIds(aiTranslateBatchRows.value),
    ),
)
const providerModelText = computed<string | null>(() => {
  if (aiConfigReadiness.value !== 'ready') return null
  const model = aiConfigEffectiveModel.value
  const provider = aiConfigEffectiveProvider.value
  return provider === '' ? model : t('panel.ai_translation.provider_model', { provider, model })
})

const cancelLabelKey = computed<string>(() =>
  aiTranslateBatchStateValue.value === 'generating'
    ? 'panel.ai_translation.cancel_batch'
    : 'command.ai.translate.cancel',
)

const canPromoteAiTranslateBatch = computed<boolean>(
  () => aiTranslateBatchStateValue.value !== 'generating' && aiTranslateBatchPromotableCount.value > 0,
)

const batchRowTexts = computed<string[]>(() => {
  const sourceById = new Map(editorSegments.value.map((s) => [s.id, s.source_text]))
  return aiTranslateBatchRows.value.map((row) => (row.text !== '' ? row.text : (sourceById.get(row.segmentId) ?? '')))
})

const batchErrorSpan = computed(() =>
  aiTranslateBatchStateValue.value === 'error'
    ? aiTranslateBatchErrorSpan(aiTranslateBatchRows.value, aiTranslateBatchError.value)
    : null,
)

function formatRange(first: number, last: number): string {
  return first === last ? String(first) : `${first}–${last}`
}

const promoteBatchReportText = computed<string | null>(() => {
  const report = editorPromoteBatchReport.value
  if (report === null) return null
  return report.skipped === 0
    ? t('panel.ai_translation.promote_batch_report', { written: String(report.written) })
    : t('panel.ai_translation.promote_batch_report_skipped', {
        written: String(report.written),
        skipped: String(report.skipped),
      })
})
const providerLabel = computed(() => {
  const raw = aiConfigEffectiveProvider.value.trim()
  const known = AI_PROVIDERS.find((p) => p === raw.toLowerCase())
  return known === undefined ? raw : t(`settings.ai_config.provider_${known}`)
})

const notReadyLines = computed(() =>
  aiConfigNotReady.value.map((reason) => {
    if (reason === 'endpoint_missing') return { key: reason, text: t('panel.ai_translation.reason_endpoint_missing') }
    if (reason === 'model_missing') return { key: reason, text: t('panel.ai_translation.reason_model_missing') }
    const text =
      providerLabel.value === ''
        ? t('panel.ai_translation.reason_key_missing_no_provider')
        : t('panel.ai_translation.reason_key_missing', { provider: providerLabel.value })
    return { key: reason, text }
  }),
)

const workOverrideText = computed(() => {
  if (!aiConfigReadinessWorkIsOpen.value || aiConfigWorkOverrides.value.length === 0) return null
  const names = aiConfigWorkOverrides.value.map((f) => t(`settings.ai_config.field_${f}`)).join(', ')
  return t('panel.ai_translation.work_overrides', { fields: names })
})

const openSettingsLabel = computed(() =>
  workOverrideText.value !== null
    ? t('panel.ai_translation.unconfigured_open_work_settings')
    : t('panel.ai_translation.unconfigured_open_settings'),
)
</script>

<template>
  <PanelFrame owner="panel.ai_translation" status-key="panel.ai_translation.status" :show-status="false">
    <div class="ai-panel-head">
      <!-- aura-allow-text: KẾT QUẢ của t() (nhà cung cấp · mô hình đọc từ cấu hình). -->
      <span v-if="providerModelText !== null" class="ai-panel-provider" data-ai-panel-provider>{{ providerModelText }}</span>
      <button
        v-if="promptSets.length === 0"
        type="button"
        class="ui-btn"
        data-ai-prompt-library-link
        @click="dispatch('prompt.library.open')"
      >
        {{ t('panel.ai_translation.prompt_set_empty') }}
      </button>
      <!-- aura-allow-text: KẾT QUẢ của t() (tên bộ nội suy qua tham số). -->
      <span v-else-if="selectedPromptSetName !== null" class="ai-panel-provider" data-ai-panel-set>{{ promptSetHeadText }}</span>
      <template v-else>
        <span class="ai-panel-provider" data-ai-panel-set-none>{{ t('panel.ai_translation.prompt_set_none') }}</span>
        <button
          type="button"
          class="ui-btn"
          data-ai-prompt-set-pick
          @click="dispatch('ai.prompt_inspector.open')"
        >
          {{ t('panel.ai_translation.prompt_set_pick') }}
        </button>
      </template>
      <button
        type="button"
        class="ai-inspector-open ui-btn ui-btn--quiet"
        data-ai-prompt-inspector-open
        @click="dispatch('ai.prompt_inspector.open')"
      >
        {{ t('command.ai.prompt_inspector.open') }}
      </button>
    </div>
    <div v-if="aiConfigReadiness === 'unconfigured'" class="ai-unconfigured" data-ai-unconfigured>
      <p class="ai-unconfigured-title">{{ t('panel.ai_translation.unconfigured_title') }}</p>
      <p class="ai-unconfigured-text">{{ t('panel.ai_translation.unconfigured_other') }}</p>
      <!-- aura-allow-text: KẾT QUẢ của t() (câu lý do dựng trong notReadyLines). -->
      <p
        v-for="reason in notReadyLines"
        :key="reason.key"
        class="ai-unconfigured-text"
        data-ai-unconfigured-reason
        :data-ai-unconfigured-reason-kind="reason.key"
      >
        {{ reason.text }}
      </p>
      <!-- aura-allow-text: KẾT QUẢ của t() (dòng ghi đè dựng trong workOverrideText). -->
      <p
        v-if="workOverrideText !== null"
        class="ai-unconfigured-text"
        data-ai-unconfigured-overrides
      >
        {{ workOverrideText }}
      </p>
      <!-- aura-allow-text: KẾT QUẢ của t() (nhãn nút dựng trong openSettingsLabel). -->
      <button
        type="button"
        class="ui-btn"
        data-ai-unconfigured-open-settings
        @click="dispatch('settings.ai_config.open')"
      >
        {{ openSettingsLabel }}
      </button>
    </div>
    <p v-else-if="aiConfigReadiness === 'key_unchecked'" class="ai-translate-hint" data-ai-key-unchecked>
      {{ t('panel.ai_translation.key_unchecked') }}
    </p>
    <div ref="surface" class="ai-surface">
      <div class="ai-translate-actions">
        <button
          v-if="!canCancelAiTranslate"
          type="button"
          class="ui-btn ui-btn--primary ai-run-slot"
          data-ai-translate-run
          :disabled="!canRunAiTranslate"
          @click="dispatch('ai.translate.run')"
        >
          {{ t('command.ai.translate.run') }}
        </button>
        <button
          v-else
          type="button"
          class="ui-btn ui-btn--primary ai-run-slot"
          data-ai-translate-cancel
          @click="dispatch('ai.translate.cancel')"
        >
          <!-- aura-allow-text: KẾT QUẢ của t() (hai nhãn: Huỷ dịch / Huỷ lô). -->
          {{ t(cancelLabelKey) }}
        </button>
        <button
          type="button"
          class="ui-btn"
          data-ai-translate-promote
          :disabled="!canPromoteAiTranslate"
          @click="dispatch('ai.translate.promote')"
        >
          {{ t('command.ai.translate.promote') }}
        </button>
        <button
          v-if="canRetryAiTranslate"
          type="button"
          class="ui-btn"
          data-ai-translate-retry
          @click="dispatch('ai.translate.retry')"
        >
          {{ t('command.ai.translate.retry') }}
        </button>
      </div>
      <p v-if="!hasActiveSegment" class="ai-translate-hint" data-ai-no-segment-hint>{{ t('panel.ai_translation.translate_no_segment_hint') }}</p>
      <p
        v-if="aiTranslateStateValue === 'generating' && isAiTranslateStale"
        class="ai-translate-status"
        data-ai-translate-state="generating-other"
      >
        <!-- aura-allow-text: KẾT QUẢ của t() (nội suy số câu đang dịch). -->
        {{ t('panel.ai_translation.state_generating_other', { segment_id: String(aiTranslateRunSegmentId) }) }}
      </p>
      <p v-else-if="aiTranslateStateValue === 'generating'" class="ai-translate-status" data-ai-translate-state="generating">
        {{ t('panel.ai_translation.state_generating') }}
      </p>
      <p
        v-else-if="aiTranslateStateValue === 'cancelled' && !isAiTranslateStale"
        class="ai-translate-status"
        data-ai-translate-state="cancelled"
      >
        {{ t('panel.ai_translation.state_cancelled') }}
      </p>
      <p
        v-else-if="
          aiTranslateStateValue === 'not_configured' &&
          aiTranslateRunSegmentId !== null &&
          aiConfigReadiness !== 'unconfigured'
        "
        class="ai-translate-status"
        data-ai-translate-state="not_configured"
      >
        {{ t('panel.ai_translation.status') }}
      </p>
      <p
        v-if="aiTranslateStateValue === 'error' && aiTranslateError !== null && !isAiTranslateStale"
        class="ai-translate-alert"
        role="alert"
        data-ai-translate-state="error"
      >
        <!-- aura-allow-text: KẾT QUẢ của tError(). -->
        {{ tError(aiTranslateError) }}
        <template v-if="providerMessageOf(aiTranslateError) !== null">
          <br />
          {{ t('ai.error.provider_message', { message: providerMessageOf(aiTranslateError)! }) }}
        </template>
      </p>
      <!--
        🔴 `editorPromoteAiTranslationError` được export mà không component nào đọc: một
        lượt PROMOTE bị Rust từ chối (`segment.not_found`, `work.none_open`, …) không đổi một
        pixel nào. Cùng khuôn `.ai-translate-alert` ngay trên, cùng surface với lỗi dịch.
      -->
      <p
        v-if="editorPromoteAiTranslationError !== null"
        class="ai-translate-alert"
        role="alert"
        data-ai-translate-promote-alert
      >
        <!-- aura-allow-text: KẾT QUẢ của tError(). -->
        {{ tError(editorPromoteAiTranslationError) }}
      </p>
      <!--
        🔴 CÂU HỎI CHỐNG MẤT BẢN NHÁP CỦA PROMOTE, cùng khuôn câu hỏi của
        `SegmentHistoryOverlay.vue` (không `window.confirm()`, không một lớp phủ thứ hai —
        xem lý do đầy đủ ở đó).
      -->
      <div v-if="editorPendingPromote !== null" class="ai-promote-confirm">
        <p class="ai-promote-confirm-title">{{ t('panel.ai_translation.promote_confirm_title') }}</p>
        <p class="ai-promote-confirm-body">{{ t('panel.ai_translation.promote_confirm_body') }}</p>
        <!-- aura-allow-text: DỮ LIỆU NGƯỜI DÙNG — chính bản dịch họ vừa gõ, không một chuỗi giao diện. -->
        <p class="ai-promote-draft">{{ editorPendingPromote.draft }}</p>
        <div class="ai-promote-confirm-acts">
          <button type="button" class="ui-btn" data-ai-translate-promote-confirm @click="dispatch('ai.translate.confirm_promote')">
            {{ t('panel.ai_translation.promote_confirm_accept') }}
          </button>
          <button type="button" class="ui-btn" data-ai-translate-promote-cancel @click="dispatch('ai.translate.cancel_promote')">
            {{ t('panel.ai_translation.promote_confirm_cancel') }}
          </button>
        </div>
      </div>
      <div v-if="editorPendingPromoteBatch !== null" class="ai-promote-confirm" data-ai-translate-promote-batch-confirm>
        <p class="ai-promote-confirm-title">
          {{ t('panel.ai_translation.promote_batch_confirm_title', { count: String(editorPendingPromoteBatch.items.length) }) }}
        </p>
        <p class="ai-promote-confirm-body">{{ t('panel.ai_translation.promote_confirm_body') }}</p>
        <!-- aura-allow-text: DỮ LIỆU NGƯỜI DÙNG — các bản dịch họ vừa gõ, không một chuỗi giao diện. -->
        <p v-for="item in editorPendingPromoteBatch.items" :key="item.segmentId" class="ai-promote-draft">{{ item.draft }}</p>
        <div class="ai-promote-confirm-acts">
          <button type="button" class="ui-btn" data-ai-translate-promote-confirm @click="dispatch('ai.translate.confirm_promote')">
            {{ t('panel.ai_translation.promote_confirm_accept') }}
          </button>
          <button type="button" class="ui-btn" data-ai-translate-promote-cancel @click="dispatch('ai.translate.cancel_promote')">
            {{ t('panel.ai_translation.promote_confirm_cancel') }}
          </button>
        </div>
      </div>
      <p v-if="promoteBatchReportText !== null" class="ai-translate-status" role="status" data-ai-translate-promote-batch-report>
        <!-- aura-allow-text: KẾT QUẢ của t() (số câu đã ghi / bị bỏ). -->
        {{ promoteBatchReportText }}
      </p>
      <div
        v-if="(aiTranslateAccumulatedText !== '' || aiTranslateMaybeTruncated || (aiUsageVisible && aiTranslateUsageLineValue !== null)) && !isAiTranslateStale"
        class="ai-result-card"
        data-ai-result-card
      >
        <p v-if="aiTranslateAccumulatedText !== '' && !isAiTranslateStale" class="ai-translate-text" data-ai-translate-text>
          <!-- aura-allow-text: DỮ LIỆU (văn bản do AI sinh ra, chảy dần qua Channel). -->
          {{ aiTranslateAccumulatedText }}
        </p>
        <p v-if="aiTranslateMaybeTruncated" class="ai-translate-usage" role="status" data-ai-translate-maybe-truncated>
          {{ t('panel.ai_translation.flag_maybe_truncated') }}
        </p>
        <p
          v-if="aiUsageVisible && aiTranslateUsageLineValue !== null && !isAiTranslateStale"
          class="ai-translate-usage"
          role="status"
          data-ai-translate-usage
        >
          <!-- aura-allow-text: KẾT QUẢ của t() (Story 4.11: số token + ước tính chi phí). -->
          {{ t(aiTranslateUsageLineValue.key, aiTranslateUsageLineValue.params) }}
        </p>
      </div>

      <!-- ─────────────────────────────────────────────────────────────────────────────
           Story 4.9, Phase 3 — "Dịch theo LÔ với tiến độ và huỷ giữa chừng" (FR73, AD-22,
           AC1/AC2). Nút Huỷ và trạng thái LỖI dùng LẠI `.ai-translate-cancel`/`.ai-translate-alert`
           ngay trên (một nút, một khối lỗi cho CẢ HAI hình dạng lời gọi — §Tasks spec 4.9
           Phase 3: "reuse the existing cancel button"), khối dưới đây chỉ thêm phần LÔ
           không có ở lượt đơn: nút chạy lô, tiến độ, và danh sách hàng. -->
      <p class="ai-group-label" data-ai-chapter-group>{{ t('panel.ai_translation.chapter_group') }}</p>
      <div class="ai-batch-bar">
        <button
          type="button"
          class="ui-btn"
          data-ai-translate-chapter-run
          :disabled="!canRunAiTranslateChapter"
          @click="dispatch('ai.translate.chapter_run')"
        >
          <!-- aura-allow-text: KẾT QUẢ của t() (hai nhãn: cả Chương / N câu chưa dịch). -->
          {{ chapterRunLabel }}
        </button>
        <button
          type="button"
          class="ui-btn"
          data-ai-translate-batch-run
          :disabled="!canRunAiTranslateBatch"
          @click="dispatch('ai.translate.batch_run')"
        >
          <!-- aura-allow-text: KẾT QUẢ của t() (hai nhãn: chưa chọn / đã chọn N câu). -->
          {{ batchRunLabel }}
        </button>
        <!-- aura-allow-text: KẾT QUẢ của t() (phím đọc từ bảng phím tắt thật). -->
        <span v-if="segmentSelectionCount === 0 && selectChordText !== ''" class="ai-translate-hint" data-ai-batch-select-hint>
          {{ t('panel.ai_translation.batch_select_hint', { chord: selectChordText }) }}
        </span>
        <button
          v-if="segmentSelectionCount > 0"
          type="button"
          class="ui-btn ui-btn--quiet"
          data-ai-selection-clear
          @click="dispatch('segment.selection.clear')"
        >
          {{ t('panel.ai_translation.selection_clear') }}
        </button>
      </div>
      <template v-if="aiTranslateBatchTotalCount > 0">
        <p class="ai-batch-progress" data-ai-translate-batch-progress>
          {{
            t('panel.ai_translation.batch_progress', {
              done_count: String(aiTranslateBatchDoneCount),
              total_count: String(aiTranslateBatchTotalCount),
              remaining_count: String(aiTranslateBatchRemainingCount),
            })
          }}
        </p>
        <p
          v-if="aiUsageVisible && aiTranslateBatchUsageLineValue !== null"
          class="ai-batch-usage"
          role="status"
          data-ai-translate-batch-usage
        >
          <!-- aura-allow-text: KẾT QUẢ của t() (Story 4.11: tổng token của lô). -->
          {{ t(aiTranslateBatchUsageLineValue.key, aiTranslateBatchUsageLineValue.params) }}
        </p>
        <p
          v-if="aiTranslateBatchRunningCall !== null"
          class="ai-batch-running"
          data-ai-translate-batch-running
        >
          {{ t('panel.ai_translation.batch_running', { call_index: String(aiTranslateBatchRunningCall) }) }}
        </p>
        <p
          v-if="aiTranslateBatchMissingCount > 0 || aiTranslateBatchNotSentCount > 0"
          class="ai-batch-gaps"
          role="status"
          data-ai-translate-batch-gaps
        >
          {{
            t('panel.ai_translation.batch_gaps', {
              missing_count: String(aiTranslateBatchMissingCount),
              not_sent_count: String(aiTranslateBatchNotSentCount),
            })
          }}
        </p>
        <div
          v-if="aiTranslateBatchStateValue === 'error' && aiTranslateBatchError !== null"
          class="ai-translate-alert ai-batch-error"
          role="alert"
          data-ai-translate-batch-alert
        >
          <template v-if="batchErrorSpan !== null">
            <p class="ai-batch-error-line" data-ai-batch-error-stopped>
              {{
                t('panel.ai_translation.batch_error_stopped', {
                  call_index: String(batchErrorSpan.callIndex),
                  range: formatRange(batchErrorSpan.first, batchErrorSpan.last),
                })
              }}
            </p>
            <p class="ai-batch-error-line" data-ai-batch-error-cause>
              <!-- aura-allow-text: KẾT QUẢ của tError() trong t(). -->
              {{ t('panel.ai_translation.batch_error_cause', { cause: tError(aiTranslateBatchError) }) }}
            </p>
          </template>
          <template v-else>
            <!-- aura-allow-text: KẾT QUẢ của tError(). -->
            {{ tError(aiTranslateBatchError) }}
          </template>
          <p
            v-if="providerMessageOf(aiTranslateBatchError) !== null"
            class="ai-batch-error-line"
            data-ai-batch-error-provider-message
          >
            {{ t('ai.error.provider_message', { message: providerMessageOf(aiTranslateBatchError)! }) }}
          </p>
        </div>
        <div class="ai-batch-actions" data-ai-translate-batch-actions>
          <button
            v-if="canRetryAiTranslateBatch"
            type="button"
            class="ui-btn"
            data-ai-translate-batch-retry
            @click="dispatch('ai.translate.batch_retry')"
          >
            <!-- aura-allow-text: KẾT QUẢ của t() (số câu thiếu/chưa gửi). -->
            {{
              t('panel.ai_translation.batch_retry_count', {
                count: String(aiTranslateBatchMissingCount + aiTranslateBatchNotSentCount),
              })
            }}
          </button>
          <button
            type="button"
            class="ui-btn"
            data-ai-translate-promote-batch
            :disabled="!canPromoteAiTranslateBatch"
            @click="dispatch('ai.translate.promote_batch')"
          >
            {{ t('panel.ai_translation.promote_batch', { count: String(aiTranslateBatchPromotableCount) }) }}
          </button>
          <button
            type="button"
            class="ui-btn ui-btn--quiet"
            data-ai-translate-batch-discard
            @click="dispatch('ai.translate.batch_discard')"
          >
            {{ t('command.ai.translate.batch_discard') }}
          </button>
        </div>
        <ul class="ai-batch-rows" data-ai-translate-batch-rows>
          <li
            v-for="(row, index) in aiTranslateBatchRows"
            :key="row.segmentId"
            class="ai-batch-row"
            :data-ai-translate-batch-row-status="row.status"
            :data-ai-translate-batch-row-segment-id="row.segmentId"
          >
            <span class="ai-batch-row-segment" data-ai-translate-batch-row-no>
              <!-- aura-allow-text: DỮ LIỆU (số thứ tự của hàng trong lô). -->
              {{ index + 1 }}
            </span>
            <span class="ai-batch-row-text" data-ai-translate-batch-row-text>
              <!-- aura-allow-text: DỮ LIỆU (bản dịch AI, hoặc câu nguồn khi chưa có bản dịch). -->
              {{ batchRowTexts[index] }}
            </span>
            <span v-if="row.hasHan" class="ai-batch-row-flag" data-ai-translate-batch-row-han>
              {{ t('panel.ai_translation.flag_has_han') }}
            </span>
            <span class="ai-batch-row-status" :class="{ 'ai-batch-row-status-error': row.status === 'missing' }">
              <!-- aura-allow-text: KẾT QUẢ của t() (nhãn theo trạng thái hàng). -->
              {{ t(aiTranslateBatchRowStatusKey(row)) }}
            </span>
          </li>
        </ul>
      </template>
    </div>
  </PanelFrame>
</template>

<style scoped>
.ai-panel-head {
  flex: none;
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
  margin-bottom: var(--space-panel-block);
}

.ai-panel-provider {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-batch-row-flag {
  color: var(--color-error);
}

.ai-panel-head .ai-inspector-open {
  margin-left: auto;
}

.ai-unconfigured {
  flex: none;
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin-bottom: var(--space-panel-block);
}

.ai-unconfigured-title {
  margin: 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.ai-unconfigured-text {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-unconfigured .ui-btn {
  align-self: flex-start;
}

/*
 * `flex: 1; min-height: 0` thay `height: 100%` — bắt buộc từ khi `.panel-body` mang HAI con
 * (bar + bề mặt), xem doc-comment `PanelFrame.vue::.panel-body`: `height: 100%` sẽ tự đo
 * theo chiều cao `.panel-body`, cộng dồn với chiều cao `.ai-panel-head` mà tràn khỏi panel.
 */
.ai-surface {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  overflow-y: auto;
  margin: 0 calc(-1 * var(--space-panel-inline)) calc(-1 * var(--space-panel-block));
  padding: 0 var(--space-panel-inline) var(--space-panel-block);
}

.aura-dock-edge-left .ai-surface {
  margin-left: calc(-1 * var(--space-screen-inline));
  padding-left: var(--space-screen-inline);
}

.aura-dock-edge-right .ai-surface {
  margin-right: calc(-1 * var(--space-screen-inline));
  padding-right: var(--space-screen-inline);
}

/* Story 4.8 — nút Dịch/Huỷ/Đưa sang bản dịch. */
.ai-translate-actions {
  flex: none;
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: calc(var(--space-unit) * 1);
}

.ai-translate-hint,
.ai-translate-status,
.ai-batch-progress,
.ai-batch-running,
.ai-translate-usage,
.ai-batch-usage {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-translate-alert {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

/* Văn bản AI chảy dần — cùng token `editor` mà `GridPanel.vue` dùng cho bản dịch trong Editor
   (DESIGN.md: "Bản dịch trong Editor", 15px/1,95): đây LÀ một bản dịch, chỉ chưa được đưa vào
   Editor. Giữ nguyên xuống dòng của chính nó (AD-16: không `v-html`, đây vẫn là một text node
   thuần, chỉ CSS đổi cách trình bày khoảng trắng). */
/* Run and its Cancel share one slot; a common minimum keeps the buttons after it still. */
.ai-run-slot {
  min-width: 8.5em;
  justify-content: center;
}

.ai-result-card {
  flex: none;
  display: flex;
  flex-direction: column;
  gap: var(--space-panel-block);
  padding: 13px; /* aura-allow-spacing: 1px border sits inside the 14px inset */
  background: var(--color-surface-sunken);
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-default);
}

.ai-result-card .ai-translate-usage {
  padding-top: 6px; /* aura-allow-spacing: 1px top border sits inside the 7px inset */
  border-top: 1px solid var(--color-outline);
}

.ai-group-label {
  flex: none;
  margin: calc(var(--space-unit) * 1) 0 0;
  padding-top: 13px; /* aura-allow-spacing: 1px top border sits inside the 14px inset */
  border-top: 1px solid var(--color-outline-faint);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface-variant);
}

.ai-translate-text {
  margin: 0;
  font-family: var(--face-editor);
  font-size: var(--font-editor);
  line-height: var(--leading-editor);
  color: var(--color-on-surface);
  white-space: pre-wrap;
}

/* Story 4.9, Phase 3 — "Dịch theo LÔ với tiến độ và huỷ giữa chừng". Chỉ token đã có (không
   một màu/bóng đổ/lớp phủ mới), cùng luật mọi khối khác của panel này. */
.ai-batch-bar {
  flex: none;
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
}

.ai-batch-rows {
  max-height: calc(var(--space-unit) * 23);
  overflow-y: auto;
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
}

.ai-batch-row {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 1);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-batch-actions {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
}

.ai-batch-error {
  display: flex;
  flex-direction: column;
}

.ai-batch-error-line {
  margin: 0;
}

.ai-batch-row-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--face-read-sm);
  font-size: var(--font-read-sm);
}

.ai-batch-row-tokens,
.ai-batch-row-status {
  white-space: nowrap;
}

.ai-batch-row-status-error {
  color: var(--color-error);
}

.ai-batch-row-segment {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  color: var(--color-on-surface);
}

/*
 * 🔴 Câu hỏi chống mất bản nháp của PROMOTE. Token và khuôn CHÉP NGUYÊN từ
 * `.hist-confirm*`/`.hist-draft`/`.hist-act` của `SegmentHistoryOverlay.vue` — cùng câu hỏi,
 * hai bề mặt khác nhau.
 */
.ai-promote-confirm {
  margin-bottom: var(--space-panel-block);
  padding: 13px; /* aura-allow-spacing: 1px border sits inside the 14px inset */
  border: 1px solid var(--color-outline);
}

.ai-promote-confirm-title {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.ai-promote-confirm-body {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-promote-draft {
  margin: 0 0 var(--space-panel-block) 0;
  padding: 14px 14px 14px 12px; /* aura-allow-spacing: 2px rule sits inside the 14px inset */
  border-left: 2px solid var(--color-draft);
  font-family: var(--face-read-sm);
  font-size: var(--font-read-sm);
  line-height: var(--leading-read-sm);
  color: var(--color-on-surface);
}

.ai-promote-confirm-acts {
  display: flex;
  gap: var(--space-panel-inline);
}
</style>
