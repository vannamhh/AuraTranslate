<script setup lang="ts">
// Panel `Lưới` — **lưới hai cột đối chiếu**. Story 2.5b · AC1–AC14 · UX-DR13 · FR16 · FR19 · FR21.
//
// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 MỘT HÀNG = MỘT CÂU. NĂM CỘT. VÀ LƯỚI LÀ **CHỦ-CỘT**, KHÔNG CHỦ-HÀNG
// ═════════════════════════════════════════════════════════════════════════════════
// AC2 đòi **hàng** *(mỗi câu một hàng: vạch · số câu · nguyên văn · bản dịch · nhãn)*.
// AC7 đòi **cột** là **một bề mặt đăng ký duy nhất**. Hai đòi hỏi ấy **xung khắc trong DOM
// thường**: `selectionContract.ts:168-190` duyệt bằng `el.contains(anchor)`, nên một bề mặt
// phải là **TỔ TIÊN DOM** của chữ trong nó — mà trong `<table>` *(và trong mọi lưới chủ-hàng)*
// một **cột không có phần tử tổ tiên nào**. `<col>`/`<colgroup>` **không chứa** `<td>`.
//
// ⇒ Quyết định #1 đường (b), Ice ký 2026-08-14: **CSS Grid chủ-cột với `subgrid`**. Grid cha
// khai tập track hàng; **năm** con là **năm CỘT**, mỗi cột `grid-row: 1 / -1` +
// `grid-template-rows: subgrid` ⇒ mọi cột chia **chung một** tập track ⇒ hàng thẳng.
//
// **Đo 2026-08-14 trên CẢ HAI engine** *(bàn đo `2-5b-ban-do/`)*: lệch `top` giữa năm ô của
// cùng một hàng = **0 px** trên WKWebView 605.1.15 **và** trên Blink. Không đọc bảng tương
// thích — cùng luật `selectionContract.ts:141`.
//
// ⚠️ **CÁI GIÁ, nói trước chứ không phát hiện sau: HÀNG KHÔNG CÒN LÀ MỘT HỘP.** Mọi kiểu
// dáng cấp hàng *(nền hàng đang sửa · nền hàng TM · đường kẻ dưới hàng · khoảng thở đoạn
// AC5)* phải nhân ra **năm ô**, không một chỗ. Bản dựng `.working/editor-grid-two-column.html`
// viết chúng trên `<tr>` — **không chép thẳng được**.
//
// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 MỖI Ô BẢN DỊCH LÀ MỘT EDITING HOST **RIÊNG** — Quyết định #3(b), và nó LẬT Story 2.3
// ═════════════════════════════════════════════════════════════════════════════════
// Quyết định #1 của Story 2.3 *(Ice ký 2026-08-12)* khai: *"vùng gõ là **MỘT câu tại một
// thời điểm**"*, cài bằng `contenteditable` trên **đúng một** `<span>`. Mệnh đề đó **hết hiệu
// lực từ story này**, và lý do là **tiền đề của nó không còn tồn tại**, không phải nó sai lúc
// được ký: tiền đề là *"một dòng văn liên tục"*, và lưới không còn dòng văn liên tục nào.
//
// Cái nó mua được, đo được:
//   ① Khuyết tật *"sập hố"* chết **theo cấu trúc**. Nó tới từ một `<span>` rỗng **cao 0 px**
//      — caret không có hộp nào để lấy chiều cao. Một ô `min-height: 1.95em` thì có.
//      *(Đo 2026-08-14: ô rỗng **38,00 px** = ô có chữ **38,00 px**, WKWebView.)*
//   ② `data-segment-id` không còn bị trình duyệt gộp/tách qua ranh giới câu — mỗi id sống
//      trong một editing host **riêng**, nên không có ranh giới nào để gộp qua. Đây là **cùng
//      một bảo đảm** mà đường (c) của Story 2.3 mua bằng cách chỉ cho một span gõ được.
//
// ⚠️ Cái nó **KHÔNG** mua được, ghi ra thay vì để Story 2.9 vấp:
//      `Backspace` ở offset 0 **KHÔNG phát `beforeinput`** trên WebKit *(đo 2026-08-14, cả
//      WKWebView lẫn Playwright-WebKit, cả phím vật lý lẫn `execCommand`; Blink thì **có**)*.
//      Cử chỉ gộp của UX-DR32 phải đi `keydown` **kèm chốt `isComposing`**. Nợ có chủ:
//      Story 2.9, `deferred-work.md`.
//
// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 `contenteditable` TRẦN **KHÔNG ĐỦ** TRÊN WKWebView — đường chuột phải đi cùng
// ═════════════════════════════════════════════════════════════════════════════════
// Đo 2026-08-14, cửa sổ Tauri thật, chuột thật, ô **rỗng** đã `contenteditable` từ đầu:
//
// | | `contenteditable` trần | + đường chuột dưới đây |
// |---|---|---|
// | `document.activeElement` | **`SECTION.mode`** | **chính ô** |
// | `getSelection().type` | **`"None"`** | **`"Caret"`** |
// | lượt `focusin` | **0** | có |
//
// *(Cú bấm **có** trúng ô — `mousedown`/`mouseup`/`click` đều target đúng nó.)*
//
// ⇒ Mỗi ô là một editing host riêng **không** miễn cho đường chuột. Cùng họ với khuyết tật
// Story 1.22-C2 *(WKWebView không focus `<button>`)* và Story 2.3 *(không focus `<span>`)*.
// Ba mảnh đã thắng của 2.3 được **chép nguyên**, không diễn giải lại: `setPosition` *(không
// `addRange`)* · đặt caret ở `mouseup` *(không `mousedown`)* · một lượt vá ở frame kế tiếp.
//
// ⚠️ Vỏ `PanelFrame` đã lo hợp đồng tiêu điểm (AD-34 §2): `declareFocus(owner, …)` chạy từ
// Story 1.6, và `'panel.grid'` có trong `FOCUS_OWNERS`. **Đừng** dựng vạch tiêu điểm panel
// thứ hai — `.panel.focused::before` đã có, và nó là vạch của **panel** (UX-DR8), khác hẳn
// vạch của **segment** (UX-DR19) mà tệp này dựng.
import { computed, onBeforeUnmount, onMounted, provide, ref, shallowRef, useTemplateRef, watch } from 'vue'
import { isMeaningKind } from '../config/proofread'
import { GRID_BLOCK_ROWS, gridBlocksOf } from './gridBlocks'
import { repaintProofreadHost, unwrapAllProofread, unwrapProofread } from './proofreadHighlight'
import { clearProofreadFor, proofreadResults } from '../proofreadState'
import PanelFrame from './PanelFrame.vue'
import GridBlock from './GridBlock.vue'
import { GRID_BLOCK_CONTEXT } from './gridBlockContext'
import type { GridBlockContext, GridBlockView, SourcePieceInfo } from './gridBlockContext'
import { useSelectionSurface } from './selectionContract'
import { resolveHanVietSelection } from './hanVietSurfaces'
import { isSourceColumnSelecting, sourceColumnText } from './gridSourceSelection'
import type { DockviewPanelProps } from '../layout/panelProps'
import type { ChapterAsset } from '../config/segment'
import { t, tError } from '../i18n'
import type { IpcError } from '../i18n'
import { detectIsMac, dispatch } from '../commands'
import {
  caretAtCellStart,
  hasPrimaryModifier,
  resolveSegmentRule,
  segmentRuleInputOf,
  selectedRowClassOf,
  sourceCutOffsetOf,
} from './editorSegments'
import type { SegmentRuleValue } from './editorSegments'
import {
  activeTab,
  canUseParallelView,
  ensureChapterLoaded,
  sourceChapter,
  sourceChapterError,
  viewMode,
} from './sourcePanelState'
import {
  clearEditorCaretPlacement,
  editorCaretPlacement,
  editorCaretSegmentId,
  editorChapterAssets,
  editorChapterId,
  editorConfirmError,
  editorEditedShape,
  editorEditedText,
  editorTmFilledSegmentIds,
  editorFlushError,
  editorHasLoaded,
  editorLoadError,
  editorPending,
  editorSegments,
  editorSourceCut,
  ensureSegmentsLoaded,
  noteEditorEdit,
  setEditorCaret,
  setEditorSourceCut,
} from './editorPanelState'
// 🔴 Ba nhánh từ chối của lượt khôi phục FR101 vào CHUNG một bề mặt với lỗi xác nhận/flush
// (cột nhãn trạng thái của hàng).
import { historyRestoreError } from './segmentHistoryState'
import { inlineStripAnchorSegmentId } from '../inlineStripEligibility'
// 🔵 Story 3.4b — tiêu thụ bề mặt IPC `glossary_marks_for_chapter` (Story 3.4). `glossaryMarks`
// là mảng TUYỆT ĐỐI (offset vào chuỗi Chương nối bằng `\n`); `glossaryMarksBySegment` chia nó
// về từng hàng — xem doc-comment của cả hai tệp cho lý do chất nối và lý do KHÔNG gộp tập biên
// thuật ngữ với `pendingCuts`.
import { ensureGlossaryMarksLoaded, glossaryMarks } from './glossaryMarksState'
import { EMPTY_SEGMENT_GLOSSARY_MARKS, glossaryMarksBySegment } from './glossaryMarksMap'
import { concordanceSourceHighlight } from './concordanceState'
import type { SegmentTermSpan } from './glossaryMarksMap'
import { clearHoveredGlossaryTerm, setHoveredGlossaryTerm } from './glossaryTermHoverState'
// 🔵 Story 4.9 — vùng chọn nhiều-segment (Decision 1, batch input). `segmentSelectionIds`
// là DANH SÁCH id theo thứ tự tài liệu; lưới chỉ cần biết hàng nào TRONG danh sách đó, xem
// [`selectedSegmentIds`] ngay dưới.
import { segmentSelectionIds } from './segmentSelectionState'

defineProps<DockviewPanelProps>()

onMounted(() => {
  // Idempotent — cùng khuôn `SourcePanel.vue`/`EditorPanel.vue`. Gọi lại ở mỗi lượt mount
  // (kể cả sau một lượt đổi preset) là AN TOÀN và KHÔNG chạy lại IPC.
  //
  // 🔴 HAI lượt nạp, không một: lưới đọc **cả hai** nguồn mà hai panel cũ đọc riêng —
  // `editorSegments` cho hàng, `sourceChapter` cho `source_lang` và cho bảng âm Hán Việt.
  void ensureSegmentsLoaded()
  void ensureChapterLoaded()
})

/**
 * 🔵 Story 3.4b — funnel của "dấu thuật ngữ Glossary", phủ CẢ HAI đường mở Chương mà tệp này
 * và `editorPanelState.ts` đã có: lượt mount ĐẦU TIÊN ở trên (hai lời gọi `void`, KHÔNG
 * `await`) và `switchChapter()` (chuyển Chương kề, đã `await` cả hai nguồn trước khi gọi
 * tường minh — xem `editorPanelState.ts`).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO `watch`, KHÔNG MỘT LƯỢT `await` THỨ HAI Ở ĐÂY
 * ─────────────────────────────────────────────────────────────────────────────
 * `ensureSegmentsLoaded()`/`ensureChapterLoaded()` ở `onMounted` phía trên là `void` — không
 * `await`. Gọi lại CHÍNH HAI HÀM ĐÓ ở đây rồi `await` chúng KHÔNG chờ được lượt gọi gốc: cả
 * hai đều idempotent (`if (requested) return`) nên lượt gọi THỨ HAI này trả lời **ngay lập
 * tức**, trước khi IPC thật kịp về — `editorChapterId.value`/`sourceChapter.value` vẫn `null`
 * tại điểm `await` đó trả về. `watch` không có vấn đề này: nó chạy lại đúng lúc HAI ref này
 * đổi giá trị, bất kể ai/khi nào phát ra lượt gọi khiến chúng đổi.
 *
 * ⚠️ Watch trên `editorChapterId`/`sourceChapter`, KHÔNG trên `editorSegments`: `editorSegments`
 * đổi tham chiếu ở MỌI lượt sửa (`replaceEditorSegment`, gộp/tách) — watch nó sẽ vi phạm "đúng
 * MỘT lượt IPC mỗi lần mở Chương" ngay trên đường xác nhận câu. `editorChapterId` chỉ đổi khi
 * Chương thật sự đổi.
 */
watch(
  [editorChapterId, sourceChapter],
  ([chapterId, chapter]) => {
    if (chapterId === null || chapter === null) return
    if (!editorHasLoaded() || chapterId !== chapter.chapter_id) return
    void ensureGlossaryMarksLoaded(chapterId, editorSegments.value, chapter.source_lang)
  },
  { immediate: true },
)

// ═════════════════════════════════════════════════════════════════════════════════
// Trạng thái màn hình — NĂM ca, và một danh sách rỗng KHÔNG được nói thay cho bốn ca kia
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * 🔴 **HAI nguồn lỗi, không một** — và vế thứ hai là một lỗ *"rỗng im lặng"* bắt ở code review
 * 2026-08-15.
 *
 * `onMounted` phát **hai** lượt nạp độc lập qua **hai lệnh IPC khác nhau**:
 * `ensureSegmentsLoaded()` *(`readOpenChapterSegments` — dựng hàng)* và `ensureChapterLoaded()`
 * *(`readOpenChapter` — cấp `source_lang` và bảng âm Hán Việt)*. Bản đầu của tệp này chỉ đọc
 * lỗi của lượt thứ nhất.
 *
 * ⇒ Đường hỏng: lượt ① thành công, lượt ② trượt. `sourceChapter` ở lại `null`, `isChinese` và
 * `showHanViet` lặng lẽ thành `false`, **dải tab Hán Việt biến mất**, và không một dòng nào
 * trên màn hình nói có lỗi — trong khi `sourceChapterError` đang cầm một `IpcError` thật.
 * `sourcePanelState.ts` export sẵn biến đó và **không ai đọc**: đúng con bug mà
 * `SourcePanel.vue` *(tệp đã xoá)* từng vá ở lượt review 2026-08-06, dựng lại từ đầu vì người
 * kế thừa chép hình dạng mà không chép danh sách người đọc.
 *
 * ⚠️ Thứ tự ưu tiên: lỗi của lượt ① đứng trước — nó chặn **toàn bộ** nội dung, còn lượt ② chỉ
 * cắt Hán Việt. Khi cả hai cùng có, chúng gần như luôn chung một nguyên nhân *(kho chưa mở,
 * Tác phẩm chưa chọn)*, nên hiện một là đủ. Điều KHÔNG được phép là hiện **không cái nào**.
 */
const loadErrorKey = computed(
  () => editorLoadError.value?.message_key ?? sourceChapterError.value?.message_key ?? null,
)
const hasSegments = computed(() => editorSegments.value.length > 0)
/**
 * Chương đã nạp nhưng **nguyên văn rỗng** — 0 byte từ `create_work_from_file`.
 *
 * 🔵 **Tái lập 2026-08-15 (code review).** `SourcePanel.vue::isEmptyChapter` bắt riêng ca này
 * từ lượt review 2026-08-06 và bị xoá cùng tệp đó, **không ai dựng lại**. Hệ quả đo được: một
 * Chương 0 byte cũng có 0 câu, nên nó rơi vào nhánh `no_segments` và màn hình nói *"chưa được
 * tách thành câu"* — một thông điệp **sai lý do**, ngụ ý người dùng còn một lệnh phải bấm,
 * trong khi thật ra **không có gì để tách**.
 *
 * ⚠️ `sourceChapter.value === null` KHÔNG phải rỗng — đó là *"chưa nạp / chưa có Chương nào"*,
 * một ca khác, đã có chủ ở [`showFrameStatus`].
 */
const isEmptyChapter = computed(() => {
  const chapter = sourceChapter.value
  if (chapter === null) return false
  // ⚠️ `source_text` khai kiểu `string` nhưng đi qua dây nó là `null` được khi một Chương chưa
  //    có nguyên văn — kiểu không nói ra điều đó. Cùng khuôn và cùng lý do với
  //    `sourcePanelState.ts::hanCharOccurrenceCount`.
  // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
  return (chapter.source_text ?? '').trim() === ''
})
const showEmptyChapter = computed(
  () => loadErrorKey.value === null && !hasSegments.value && isEmptyChapter.value,
)
/** Đã nạp xong, Chương có thật **và có chữ**, nhưng **chưa ai bấm lệnh tách** — 25 Chương của Epic 1. */
const showNoSegments = computed(
  () =>
    loadErrorKey.value === null &&
    editorHasLoaded() &&
    !hasSegments.value &&
    !isEmptyChapter.value,
)
/**
 * Câu trạng thái mặc định của `PanelFrame` chỉ đúng khi **không** có gì để nói — và *"đang
 * chờ IPC"* **là** một thứ để nói. `!editorPending` là vế bắt ở code review 2026-08-12:
 * thiếu nó, khoảng chờ của lượt nạp đọc ra `true` và panel khẳng định **dứt khoát** rằng
 * không có Chương nào, trong lúc Chương đang trên đường về.
 */
const showFrameStatus = computed(
  () =>
    loadErrorKey.value === null &&
    !editorPending.value &&
    !hasSegments.value &&
    !showNoSegments.value &&
    // 🔵 Vế thứ năm, 2026-08-15: thiếu nó, một Chương 0 byte hiện ĐỒNG THỜI hai câu — câu
    //    riêng của nó và câu mặc định *"Chưa có Chương nào để dịch"* — tức hai mệnh đề mâu
    //    thuẫn trên cùng một màn hình.
    !showEmptyChapter.value,
)

// ═════════════════════════════════════════════════════════════════════════════════
// AC2 · AC4 — SÁU giá trị vạch, qua ĐÚNG MỘT hàm phân giải (`./editorSegments.ts`)
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * Vạch của từng câu — `null` ⇒ *không vạch*, tức **không phần tử nào** được dựng.
 *
 * ⚠️ Một `Map`, không một phép tìm trong `v-for`: bảng này được tra một lần cho mỗi hàng, và
 * một `Array.find` ở đó cho ra O(n²) trên **9.850** câu.
 *
 * 🔴 **Phân giải ở ĐÂY thì đọc `editedText` TRƯỚC `s.target_text`** — và đó là một khác biệt
 * thật so với `EditorPanel.vue`, không một lượt chép cẩu thả. Giá trị `draft` mới hỏi *"ô này
 * có chữ chưa"*, còn `editorSegments` giữ bản **lúc nạp** cho FR117 (Story 2.7) nên nó ở lại
 * chuỗi rỗng suốt phiên gõ. Không có vế thứ nhất, người dùng gõ xong một câu mà vạch vẫn nói
 * *"chưa dịch"* cho tới lượt nạp lại.
 */
const restRuleById = computed(() => {
  const shapes = editorEditedShape.value
  const tmFilled = editorTmFilledSegmentIds.value
  const map = new Map<number, SegmentRuleValue>()
  for (const s of editorSegments.value) {
    const input = segmentRuleInputOf(s, null, tmFilled)
    const shape = shapes.get(s.id)
    map.set(
      s.id,
      resolveSegmentRule(shape === undefined ? input : { ...input, targetText: shape === 0 ? '' : 'x' }),
    )
  }
  return map
})

// ═════════════════════════════════════════════════════════════════════════════════
// 🔵 THÊM Story 4.9 — VÙNG CHỌN NHIỀU-SEGMENT (Decision 1). §Phase 1 Task 2: "a selection
// the user cannot see is not a selection; no new colour, no shadow, no floating layer."
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * `Set`, không một `Array.includes` trong `v-for` — cùng lý do [`ruleById`] đã ghi: một phép
 * tìm tuyến tính ở đó cho O(n²) trên **9.850** hàng, và Decision 1 chấp nhận CHÍNH XÁC một
 * Chương trọn vẹn bị chọn ("không trần cho N").
 */
const selectedSegmentIds = computed(() => new Set(segmentSelectionIds.value))

/**
 * `null` ⇒ *không tô*, tức **không lớp `:class` nào** được thêm — cùng khuôn `ruleClassById`.
 *
 * ⚠️ **CÙNG TOKEN `--color-surface-accent` với `.cell-tgt.editing`, có chủ ý.** Bảng màu chỉ
 * có 17 token mỗi theme (`tokens.json`) và spec cấm thêm màu mới. Một hàng vừa là caret vừa
 * nằm trong vùng chọn tô cùng một sắc ở cả hai lớp — không một xung đột thị giác, vì cả hai
 * lớp cùng nói "đang bật", cùng tiền lệ tái dùng `--color-primary` đã ghi ở
 * `.cell-tgt.tgt-para-end`.
 */
const selectedRowClassById = computed(() => {
  const map = new Map<number, string>()
  for (const id of selectedSegmentIds.value) {
    const cls = selectedRowClassOf(true)
    if (cls !== null) map.set(id, cls)
  }
  return map
})

// ═════════════════════════════════════════════════════════════════════════════════
// 🔵 THÊM Story 6.14 (FR42 · FR43) — ẢNH ĐÚNG VỊ TRÍ + NHÃN VAI (nợ ③, deferred-work.md)
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * Nhãn vai của `role='alt'`/`role='caption'` — chỗ ĐẦU TIÊN ở webview đọc trường `role`
 * (`ChapterSegment.role`, chở được từ Story 6.13, chưa nơi nào đọc trước story này). Chữ,
 * không màu — cùng luật "phân biệt bằng độ lùi/nhãn chữ" đã dùng cho nhật ký domain.
 */
const ROLE_LABEL_KEYS: Readonly<Record<string, string>> = {
  alt: 'panel.grid.role_alt',
  caption: 'panel.grid.role_caption',
}

/**
 * `s.id` của câu ĐẦU TIÊN trong `editorSegments` — neo `0` (không segment nào đứng trước ảnh)
 * đặt ảnh vào ĐẦU ô của câu này, trước chữ (§Always spec 6.14). `null` khi Chương rỗng — vô
 * hại, [`leadingImagesOf`] không bao giờ khớp `null`.
 */
const firstSegmentId = computed<number | null>(() => editorSegments.value[0]?.id ?? null)

/**
 * `after_segment_id → ChapterAsset[]` — nhóm sẵn MỘT lần cho cả lượt render thay vì lọc mảng
 * `editorChapterAssets` ở mỗi ô (9.850 hàng × N ảnh là một phép `O(rows × assets)` không cần
 * thiết). Hai ảnh cùng neo giữ đúng thứ tự Rust đã trả (`asset.id` tăng dần).
 */
const trailingImagesBySegmentId = computed(() => {
  const map = new Map<number, ChapterAsset[]>()
  for (const asset of editorChapterAssets.value) {
    if (asset.after_segment_id === null) continue
    const list = map.get(asset.after_segment_id) ?? []
    list.push(asset)
    map.set(asset.after_segment_id, list)
  }
  return map
})

/** Ảnh hiện Ở CUỐI ô nguyên văn của segment `id` — mọi neo `> 0` khớp đúng segment đó. */
const NO_IMAGES: ChapterAsset[] = []

function trailingImagesOf(id: number): ChapterAsset[] {
  return trailingImagesBySegmentId.value.get(id) ?? NO_IMAGES
}

/** Ảnh hiện Ở ĐẦU ô nguyên văn — CHỈ cho câu đầu Chương, CHỈ ảnh mang neo `0`. */
function leadingImagesOf(id: number): ChapterAsset[] {
  if (id !== firstSegmentId.value) return []
  return editorChapterAssets.value.filter((asset) => asset.after_segment_id === null)
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC8 · Task 7 — HÁN VIỆT SỐNG **TRONG Ô NGUYÊN VĂN**
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * AC3 của Story 1.16 — phân biệt đọc từ `source_lang`, trường **BẤT BIẾN**, không đoán từ
 * nội dung. Một Chương tiếng Anh không có tab Hán Việt và không dựng bề mặt nào.
 */
const isChinese = computed(() => sourceChapter.value?.source_lang === 'zh')

/**
 * 🔴 **AC8 — KHÔNG MỘT MẶC ĐỊNH THÔNG MINH NÀO.** Chế độ Hán Việt đọc thẳng
 * `sourcePanelState`, tức đúng thứ **người dùng tự bật**. Nó **không** đi theo bố cục và
 * **không** tự mở riêng cho hàng đang sửa.
 *
 * ⚠️ Hai phương án đó đã được nêu và **bị bác** *(`EXPERIENCE.md:249-257`, Ice ký)*. Cám dỗ
 * ở lưới còn mạnh hơn ở panel — *"chỉ mở âm cho hàng đang gõ thì đỡ cao"* — nên nó được ghi
 * ra ở đây thay vì chỉ nằm trong tài liệu UX.
 */
const showHanViet = computed(() => isChinese.value && activeTab.value === 'han_viet')

/**
 * AC1/AC2 của Story 1.16 — ô nguyên văn khai token CỦA CHÍNH NÓ theo `source_lang`:
 * `source-cjk` *(họ `read-cjk`, chỉ tiếng Trung)* cho `zh`, `source-latin` cho `en`. Không để
 * `source-cjk` gánh cả hai — họ `read-cjk` dựng chữ Latin bằng font CJK.
 */
const sourceTokenClass = computed(() => (isChinese.value ? 'tok-source-cjk' : 'tok-source-latin'))

/**
 * 🔴 **KÊNH THỊ GIÁC CHO TẬP ĐIỂM CẮT ĐANG CHỜ** — Story 2.8, AC7, chữ ký của Ice 2026-08-17.
 *
 * Cơ chế tích luỹ *(mỗi cú bấm thêm một điểm, `⌘/` cắt hết)* dựng một trạng thái **giữa hai
 * thao tác**, và một trạng thái người dùng không nhìn thấy là đúng lớp lỗi mà
 * `project-context.md` §*Rỗng IM LẶNG* cấm: họ bấm ba chỗ, quên mất, rồi `⌘/` cắt câu thành
 * bốn mảnh mà không hiểu vì đâu — trên dữ liệu AD-5 **không cho hoàn tác**.
 *
 * 🔵 **CẬP NHẬT 2026-08-17 (Story 2.9 · AC9) — mệnh đề cũ đã HẾT ĐÚNG.** Bản trước viết
 * *"dấu cắt chỉ vẽ được ở đường chữ trần […] ở chế độ Hán Việt ô chỉ nhận viền `has-cuts`,
 * tức người dùng biết 'câu này đang có điểm chờ' nhưng không thấy **ở đâu**"*.
 *
 * Dấu cắt **nay vẽ được ở cả hai kiểu xem Hán Việt**, bằng `::before` trên phần tử mang neo
 * `data-src-start` (`SourceHanViet.vue`). Lo ngại cũ *(chẻ một `<ruby>` làm đôi)* **vẫn
 * đúng** và chính là lý do phải đi đường pseudo-element: `resolveSwitch()` ánh xạ ngược bằng
 * **CHỈ SỐ** `host.children[i]`, nên một phần tử con chen vào làm tra cứu **sai im lặng**.
 * ⇒ Không `<ruby>` nào bị chẻ; không con nào được thêm.
 * ⚠️ Vế **còn hở**: một chỗ cắt nằm GIỮA một từ ở kiểu `parallel` tính đúng nhưng không có
 * chỗ bám để vẽ. Món nợ có chủ (**Ice**) ở `deferred-work.md`.
 */
const pendingCuts = computed<{ segmentId: number; offsets: readonly number[] } | null>(
  () => editorSourceCut.value,
)

/** Tập điểm cắt đang chờ của một hàng — rỗng nếu hàng này không có điểm nào. */
function cutOffsetsOf(segmentId: number): readonly number[] {
  const c = pendingCuts.value
  return c !== null && c.segmentId === segmentId ? c.offsets : EMPTY_CUTS
}
/** Một mảng rỗng DÙNG CHUNG — trả `[]` mới mỗi lượt render làm prop đổi tham chiếu mỗi tick. */
const EMPTY_CUTS: readonly number[] = []

/** Số điểm cắt đang chờ của một hàng. `0` ⇒ hàng này không có điểm nào. */
function cutCountOf(segmentId: number): number {
  const c = pendingCuts.value
  return c !== null && c.segmentId === segmentId ? c.offsets.length : 0
}

/**
 * Chia `source_text` của một hàng thành các mảnh **theo đúng tập điểm cắt đang chờ**.
 *
 * ⚠️ **Cắt bằng code point, không `slice` trần** — cùng đơn vị mà `sourceCutOffsetOf` sinh ra
 * và `regroup.rs::split_at` tiêu thụ. Một `slice` UTF-16 ở đây vẽ dấu cắt **lệch chỗ** so với
 * chỗ Rust sẽ cắt thật, tức một bản xem trước **nói dối**.
 *
 * ⚠️ Điểm nằm ngoài chuỗi bị bỏ qua **ở đây**, không bị chặn: tầng thuần Rust là chỗ từ chối
 * (AD-1), còn ô này chỉ vẽ thứ vẽ được.
 */
/**
 * 🔴 **Dấu thuật ngữ Glossary, đã chia về từng segment** — Story 3.4b.
 *
 * `Map` THƯA: một segment không có mark nào **không có mục** trong `Map` — chỗ đọc dùng
 * [`EMPTY_SEGMENT_GLOSSARY_MARKS`] làm mặc định, cùng khuôn `EMPTY_CUTS`.
 */
const glossaryMarksPerSegment = computed(() => glossaryMarksBySegment(editorSegments.value, glossaryMarks.value))

/** Biên cắt + span thuật ngữ của MỘT hàng — rỗng nếu hàng này không có dấu nào. */
function glossaryTermsOf(segmentId: number) {
  return glossaryMarksPerSegment.value.get(segmentId) ?? EMPTY_SEGMENT_GLOSSARY_MARKS
}

/**
 * 🔴 **Chỉ số ký tự nguồn nơi mỗi mảnh của [`sourcePiecesOf`] bắt đầu** — Story 2.9, AC9.
 *
 * Cùng biên, cùng thứ tự, cùng đơn vị *(điểm mã)* với hàm kia. Nó tồn tại để mỗi mảnh mang
 * được một **NEO** `data-src-start` trong DOM, thứ mà `sourceCutOffsetOf` đọc thay cho phép
 * đếm mù mọi text node.
 *
 * ⚠️ Hai hàm phải giữ **cùng một** phép chia. Tách chúng ra là mời một lượt lệch im lặng —
 * đây là chỗ duy nhất trong story chấp nhận một bản sao, và nó chấp nhận được vì cả hai đọc
 * chung `pendingCuts` VÀ `glossaryTermsOf(segmentId).boundaries`, cùng lọc/sắp y hệt.
 *
 * 🔴 **Story 3.4b — HAI tập điểm HỢP LẠI chỉ để CẮT, không để VẼ.** `pendingCuts` vẽ
 * `cut-here`; biên thuật ngữ chỉ cắt. Gộp chúng ở ĐÂY (nơi quyết định mảnh nào tồn tại) là
 * đúng và BẮT BUỘC — mảnh sinh bởi một biên thuật ngữ vẫn cần một `<span data-src-start>`
 * riêng để tô màu/gắn hover. Việc VẼ `cut-here` vẫn tra RIÊNG `pendingCuts` ở template
 * (`isPendingCutAt`), không suy từ việc một điểm có mặt ở mảng này.
 */
function sourcePieceStartsOf(segmentId: number, text: string): readonly number[] {
  const pending = cutOffsetsOf(segmentId)
  const terms = glossaryTermsOf(segmentId).boundaries
  if (pending.length === 0 && terms.length === 0) return [0]
  const ky = [...text]
  const moc = [...new Set([...pending, ...terms])]
    .filter((o) => o > 0 && o < ky.length)
    .sort((a, b) => a - b)
  return [0, ...moc]
}

function sourcePiecesOf(segmentId: number, text: string): readonly string[] {
  const starts = sourcePieceStartsOf(segmentId, text)
  if (starts.length === 1) return [text]
  const ky = [...text]
  const bien = [...starts, ky.length]
  const ra: string[] = []
  for (let i = 0; i < bien.length - 1; i += 1) {
    ra.push(ky.slice(bien[i], bien[i + 1]).join(''))
  }
  return ra
}

/** `true` ⇔ `offset` là một điểm cắt NGƯỜI DÙNG đang chờ của hàng `segmentId` — dùng để quyết
 * định vẽ `cut-here`, TÁCH khỏi việc `offset` có phải một biên thuật ngữ hay không. */
function isPendingCutAt(segmentId: number, offset: number): boolean {
  const c = pendingCuts.value
  return c !== null && c.segmentId === segmentId && c.offsets.includes(offset)
}

/** Span thuật ngữ (nếu có) phủ TRỌN `[start, end)` của một mảnh — `null` ⇔ mảnh này không
 * mang dấu Glossary nào. */
function glossarySpanAt(segmentId: number, start: number, end: number): SegmentTermSpan | null {
  const spans = glossaryTermsOf(segmentId).spans
  return spans.find((s) => s.start <= start && end <= s.end) ?? null
}

/**
 * Span thuật ngữ (nếu có) chứa ĐIỂM `offset` — khác [`glossarySpanAt`] (chứa TRỌN một
 * KHOẢNG): đây là vế TIÊU ĐIỂM/CARET của I/O Matrix ("rê chuột / đưa tiêu điểm"), dùng bởi
 * [`onSourceSelectionChange`].
 *
 * 🔴 **NỬA-MỞ `[start, end)`, GIỐNG MỌI biên khác trong story này** — KHÔNG hai đầu đóng. Một
 * caret ở offset `end` đứng NGAY ĐẦU ký tự tiếp theo (đúng nghĩa "vị trí `N` nằm GIỮA ký tự
 * `N-1` và ký tự `N`"), tức đã RỜI thuật ngữ. Đo được bằng ca thật
 * (`tests/frontend/glossaryHoverSelection.test.ts` §"caret RỜI khỏi mảnh mang dấu"): hai đầu
 * đóng làm offset `end` (đầu mảnh KẾ TIẾP) vẫn khớp thuật ngữ vừa rời — sai với chính nhánh
 * "rời ra thì phát tín hiệu rời" mà Ice yêu cầu.
 */
function glossarySpanAtPoint(segmentId: number, offset: number): SegmentTermSpan | null {
  const spans = glossaryTermsOf(segmentId).spans
  return spans.find((s) => s.start <= offset && offset < s.end) ?? null
}

/** Gộp [`sourcePiecesOf`]/[`sourcePieceStartsOf`] cộng nhãn Glossary thành MỘT mảng cho `v-for`
 * — tránh gọi lại nhiều hàm rời rạc trên cùng một chỉ số trong template. */
function sourcePieceInfoOf(segmentId: number, text: string): readonly SourcePieceInfo[] {
  const starts = sourcePieceStartsOf(segmentId, text)
  const pieces = sourcePiecesOf(segmentId, text)
  return pieces.map((piece, i) => {
    const start = starts[i] ?? 0
    const end = i + 1 < starts.length ? starts[i + 1] : start + [...piece].length
    return {
      text: piece,
      start,
      isPendingCut: isPendingCutAt(segmentId, start),
      glossary: glossarySpanAt(segmentId, start, end),
      concordance: isConcordanceHighlighted(segmentId, start, end),
    }
  })
}

function isConcordanceHighlighted(segmentId: number, pieceStart: number, pieceEnd: number): boolean {
  const hl = concordanceSourceHighlight.value
  return hl !== null && hl.segmentId === segmentId && pieceStart >= hl.start && pieceEnd <= hl.end
}

/** Handler `@mouseenter` của một mảnh mang dấu — no-op nếu mảnh không mang dấu nào. */
function onGlossaryPieceEnter(span: SegmentTermSpan | null): void {
  if (span === null) return
  setHoveredGlossaryTerm({ isConfirmed: span.isConfirmed, translation: span.translation })
}

/** Handler `@mouseleave` — luôn an toàn gọi, kể cả trên một mảnh không mang dấu. */
function onGlossaryPieceLeave(): void {
  clearHoveredGlossaryTerm()
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC7 · B1 — HỢP ĐỒNG VÙNG CHỌN THEO **CỘT**, hai lời gọi, hai vai KHÔNG được đảo
// ═════════════════════════════════════════════════════════════════════════════════
const gridRoot = useTemplateRef<HTMLElement>('gridRoot')
const sourceSelecting = ref(false)
const stripTrack = shallowRef<HTMLElement | null>(null)
const setStripTrackRef = (el: unknown): void => { stripTrack.value = el instanceof HTMLElement ? el : null }

const stripAnchorSegmentId = computed(() => inlineStripAnchorSegmentId.value)
const stripAnchorIndex = computed(() => {
  const id = stripAnchorSegmentId.value
  return id === null ? -1 : editorSegments.value.findIndex((s) => s.id === id)
})

const gridBlockViews = computed(() => {
  const segments = editorSegments.value
  return gridBlocksOf(segments.length).map((b, index) => ({
    index,
    start: b.start,
    segments: segments.slice(b.start, b.start + b.rows),
  }))
})

function stripRowIn(block: GridBlockView): number {
  const anchor = stripAnchorIndex.value
  return anchor >= block.start && anchor < block.start + block.segments.length ? anchor - block.start : -1
}

function anchorIdIn(block: GridBlockView): number | null {
  return stripRowIn(block) < 0 ? null : stripAnchorSegmentId.value
}

const caretIndex = computed(() => {
  const id = editorCaretSegmentId.value
  return id === null ? -1 : editorSegments.value.findIndex((s) => s.id === id)
})

function caretIdIn(block: GridBlockView): number | null {
  const index = caretIndex.value
  return index >= block.start && index < block.start + block.segments.length ? editorCaretSegmentId.value : null
}

const stripBlockIndex = computed(() => {
  const anchor = stripAnchorIndex.value
  return anchor < 0 ? 0 : Math.floor(anchor / GRID_BLOCK_ROWS)
})

function isInTargetColumn(node: Node): boolean {
  const from = node instanceof Element ? node : node.parentElement
  return from?.closest('.col-tgt') != null
}

function targetCellById(id: number): HTMLElement | null {
  return gridRoot.value?.querySelector<HTMLElement>(`[data-col="tgt"][data-segment-id="${String(id)}"]`) ?? null
}

/**
 * Truy vấn cho một vùng chọn trên **cột nguyên văn**.
 *
 * 🔴 Ba nhánh, ba nghĩa **phân biệt được** — *"rỗng IM LẶNG bị cấm; rỗng CÓ LÝ DO thì không"*:
 *   `undefined` từ sổ ⇒ neo **không** nằm trong một ô Hán Việt nào *(tab `Nguyên văn`, hay
 *   một Chương tiếng Anh)* ⇒ rơi về hành vi mặc định `Selection.toString()`;
 *   `null` ⇒ có ô Hán Việt, và **ô đó** nói *"vùng chọn này không ánh xạ được"* ⇒ **không**
 *   phát lượt tra;
 *   một chuỗi ⇒ ký tự Hán nguồn đã ánh xạ ngược.
 *
 * ⚠️ Gộp hai nhánh đầu làm một sẽ làm mọi lượt bôi đen trên ô nguyên văn **thường** im lặng
 * không tra gì. Xem `hanVietSurfaces.ts`.
 */
function resolveSourceSelection(selection: Selection): string | null {
  const mapped = resolveHanVietSelection(selection)
  if (mapped !== undefined) return mapped
  const root = gridRoot.value
  return root !== null && isSourceColumnSelecting(selection, root)
    ? sourceColumnText(selection, root)
    : selection.toString()
}

function onGridCopy(event: ClipboardEvent): void {
  const selection = window.getSelection()
  const root = gridRoot.value
  if (selection === null || root === null || !isSourceColumnSelecting(selection, root)) return
  event.clipboardData?.setData('text/plain', sourceColumnText(selection, root))
  event.preventDefault()
}

// 🔴 **VAI VIẾT LITERAL, VÀ HAI VAI NÀY KHÔNG ĐƯỢC ĐẢO** (B1).
//
// Cột bản dịch chứa **tiếng Việt đã dịch**; từ điển nhúng là zh→vi / en→vi. Một lượt tra ở đó
// trả **0 hàng, 0 lỗi, 0 ms** rồi **THAY MẤT** kết quả người dùng vừa tra từ cột nguyên văn —
// lớp *"rỗng im lặng"* mà cả dự án đặt ở trung tâm. Lỗi này đã đi qua sạch 11 cổng một lần
// rồi *(commit `1c7658d`)*; Sprint Change 2026-08-13 (Ice ký) đóng nó.
//
// ⚠️ **Kiểm F ③ nay canh YẾU hơn trước, và phải ghi ra:** nó đối chiếu theo **tệp**, mà hai
// vai nay sống trong **cùng một** tệp. Đảo vai giữa hai cột **trong tệp này** có thể đi lọt.
// ⇒ Task 5.4 nới cổng **có chủ**: nó đếm đúng hai lời gọi VÀ đòi đủ **cả hai** vai.
//
// 🔴 `'display'` **không** tắt việc bề mặt được đăng ký — nó tắt đúng MỘT đường
// (`currentSelectionText()`, tức đường tra TỪ ĐIỂN). FR48 (Story 3.3) và FR60 (Story 7.7)
// đọc vùng chọn ở đây bằng lệnh của RIÊNG chúng. Đừng gỡ lời gọi này.
/**
 * Nền tảng, đọc **một lần** lúc dựng component.
 *
 * ⚠️ Đọc một lần chứ không mỗi cú bấm: `detectIsMac()` dò `navigator.userAgentData` rồi rơi
 * về `navigator.platform`, và giá trị đó **không đổi trong một phiên**. Gọi lại ở đường nóng
 * của chuột là trả một phép dò cho mỗi lượt bấm mà không đổi được câu trả lời.
 *
 * ⚠️ **Không** tiêm được từ ngoài, khác `installCommands` — và đó là một đánh đổi có ý thức:
 * vế lái-hai-nền-tảng nằm ở [`hasPrimaryModifier`], vốn nhận nền tảng qua **tham số** và có
 * `tests/frontend/editorSourceCutGesture.test.ts` lái cả hai ca. Chỗ này chỉ là dây nối.
 */
const PLATFORM = { isMac: detectIsMac() }

useSelectionSurface(gridRoot, 'source', resolveSourceSelection, false, '[data-col="src"]')
useSelectionSurface(gridRoot, 'display', undefined, true, '[data-col="tgt"]')

// ═════════════════════════════════════════════════════════════════════════════════
// AC3 — ĐƯỜNG CHUỘT. Chép nguyên ba mảnh đã thắng của Story 2.3, không diễn giải lại
// ═════════════════════════════════════════════════════════════════════════════════
/** Ô bản dịch chứa một node bất kỳ, hoặc `null`. */
function targetCellOf(node: Node | null): HTMLElement | null {
  const from = node instanceof Element ? node : (node?.parentElement ?? null)
  return from?.closest<HTMLElement>('[data-col="tgt"]') ?? null
}

function segmentIdOf(el: HTMLElement | null): number | null {
  const raw = el?.getAttribute('data-segment-id') ?? null
  if (raw === null) return null
  const id = Number(raw)
  return Number.isFinite(id) ? id : null
}

/**
 * Đặt caret thu gọn tại `(node, offset)`.
 *
 * 🔴 `Selection.setPosition`, **KHÔNG** `removeAllRanges()` + `addRange()` — một PHÉP ĐO.
 * Hai đường trông tương đương, và trên Blink chúng tương đương. Trên **WKWebView thì không**,
 * và khác biệt rơi đúng vào ca thường nhất: với một phần tử soạn thảo được **RỖNG**, WebKit
 * **bỏ qua** lượt `addRange` ⇒ `type === "None"` và `execCommand` trả `false`.
 * *(Đo 2026-08-13, `EditorPanel.vue:539-556`. Số vẫn đứng ở hình dạng lưới — bàn đo 2.5b
 * chạy chính `setPosition` và cho `"Caret"`.)*
 */
function setCaret(node: Node, offset: number): boolean {
  const selection = window.getSelection()
  if (selection === null) return false
  selection.setPosition(node, offset)
  return selection.type === 'Caret'
}

/**
 * Đặt caret vào **đúng điểm vừa bấm**.
 *
 * ⚠️ **Không** `?.` trên hai lời gọi dưới: `lib.dom` khai cả hai là phương thức **bắt buộc**
 * của `Document`, nên một optional chain ở đó là một nhánh mà kiểu nói không bao giờ chạy —
 * `@typescript-eslint/no-unnecessary-condition` bắt đúng nó. Cả hai **trả về `null` được**,
 * và đó là nhánh thật sự phải xử.
 */
function placeCaretAtPoint(cell: HTMLElement, x: number, y: number): boolean {
  // 🔵 **2026-08-17 (Story 2.8) — HÀM NÀY ĐÃ NÉM Ở MỌI CÚ BẤM, SUỐT TỪ STORY 2.5b.**
  //
  // Bản cũ gọi `document.caretPositionFromPoint(x, y)` **trần** ở dòng đầu. Đo trong cửa sổ
  // Tauri thật ngày 2026-08-17 (`2-8-ban-do/caret-api-cot-dich.e2e.mjs`):
  //
  // | API | `typeof` |
  // |---|---|
  // | `document.caretPositionFromPoint` | **`"undefined"`** |
  // | `document.caretRangeFromPoint` | `"function"` |
  //
  // Một lời gọi tới thứ `undefined` **NÉM `TypeError`**, nó không trả `null` — nên nhánh
  // *"hồi phòng"* ở dòng dưới **chưa bao giờ chạy tới**, và cú ném giết trọn phần còn lại của
  // `onCellMouseUp`, gồm cả `ensureCaretNextFrame` — thứ mà doc-comment của chính nó gọi là
  // *"đường DUY NHẤT chạy được khi engine không làm"*.
  //
  // 🔴 **Vì sao KHÔNG ai thấy suốt hai story:** caret vẫn hiện. Đo cùng lượt, sau một cú bấm
  // chuột thật vào ô bản dịch: `selectionType = "Caret"`, `rangeCount = 1`,
  // `activeElement` = chính ô — **và** một `TypeError` cộng một `[Vue warn]` trong console.
  // Caret ấy đến từ `cell.focus()` ở trên cộng hành vi mặc định của engine, **không** từ
  // đường vá mà ba vòng chẩn đoán của Story 2.3 và 2.5b đã mua. Ca `grid-empty-cell` xanh
  // trên một sản phẩm mà nửa cơ chế của nó đang chết.
  //
  // ⇒ Ice chốt 2026-08-17: sửa trong Story 2.8, dùng chung [`caretPointAt`].
  const diem = caretPointAt(x, y)
  if (diem === null) return false
  // 🔴 **PHÂN GIẢI ĐƯỢC KHÔNG PHẢI ĐÚNG CHỖ** — bắt bằng chính bộ e2e, 2026-08-12. Một
  // vị trí trỏ ra ngoài ô là "thành công" theo hàm này mà người dùng không gõ được một chữ.
  if (!cell.contains(diem.node)) return false
  return setCaret(diem.node, diem.offset)
}

/**
 * 🔴 **Phân giải một điểm màn hình thành `(node, offset)` — CÓ DÒ NĂNG LỰC, và đó là một
 * phép đo chứ không một dòng phòng xa.**
 *
 * **Đo 2026-08-17 trong cửa sổ Tauri thật** *(`2-8-ban-do/tach-chan-doan.e2e.mjs`, vòng 4)*:
 *
 * | API | `typeof` trên WKWebView này |
 * |---|---|
 * | `document.caretPositionFromPoint` | **`"undefined"`** |
 * | `document.caretRangeFromPoint` | `"function"` — trả `offset: 19`, node **trong** ô |
 *
 * 🔴 Một lời gọi **trần** tới cái thứ nhất không trả `null` — nó **NÉM `TypeError`**. Trong
 * một handler sự kiện, cú ném đó giết trọn phần còn lại của handler và **không** hiện ra ở
 * đâu ngoài một dòng `[Vue warn] Unhandled error during execution of native event handler`.
 * Triệu chứng ở người dùng: *"bấm vào cột nguyên văn thì không có gì xảy ra"* — không lỗi,
 * không cổng nào đỏ.
 *
 * ⚠️ **Thứ tự thử: `caretRangeFromPoint` TRƯỚC.** Nó là API **WebKit thật sự có**;
 * `caretPositionFromPoint` là bản chuẩn hoá mới hơn và có mặt trên Blink. Đặt bản chuẩn
 * trước rồi *"hồi phòng"* là tối ưu cho engine mà sản phẩm **không** chạy trên đó.
 */
function caretPointAt(x: number, y: number): { node: Node; offset: number } | null {
  if (typeof document.caretRangeFromPoint === 'function') {
    const range = document.caretRangeFromPoint(x, y)
    if (range !== null) return { node: range.startContainer, offset: range.startOffset }
  }
  if (typeof document.caretPositionFromPoint === 'function') {
    const pos = document.caretPositionFromPoint(x, y)
    if (pos !== null) return { node: pos.offsetNode, offset: pos.offset }
  }
  return null
}

/**
 * 🔴 **`mouseup` trên CỘT NGUYÊN VĂN — ghi nhận điểm cắt cho `⌘/`** (Story 2.8, Quyết định
 * #2 đường (e), Ice ký 2026-08-17 **sau** phép đo).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO SẢN PHẨM PHẢI TỰ ĐẶT ĐIỂM CẮT — một phép đo, không một dòng phòng xa
 * ─────────────────────────────────────────────────────────────────────────────
 * Chữ ký đầu của Quyết định #2 là *"lấy chỗ cắt từ **vùng chọn đang có** ở cột nguyên văn"*.
 * **Bàn đo `2-8-ban-do/` bác nó bằng số**, trên WKWebView 605.1.15 trong cửa sổ Tauri thật:
 *
 * | Cử chỉ trên một ô `[data-col="src"]` | `selectionType` | `rangeCount` |
 * |---|---|---|
 * | một cú bấm | **`"None"`** | **0** |
 * | kéo chọn, **sáu** bước trung gian | **`"None"`** | **0** |
 * | kéo **sau khi** tài liệu đã có tiêu điểm | **`"None"`** | **0** |
 * | *đối chứng* — bấm ô `[data-col="tgt"]` | `"Caret"` | 1 |
 *
 * ⇒ *"Vùng chọn đang có"* là một tiền đề **không tồn tại**. Hai giả thuyết về bàn đo đã bị
 * loại từng cái *(cú `blur()` của chính bàn đo; lượt kéo quá thô)*.
 *
 * Cái mà phép đo **cho phép**: `caretPositionFromPoint`/`caretRangeFromPoint` phân giải được
 * ở đó, và offset ánh xạ **thẳng** vào chỉ số ký tự của `source_text`. ⇒ Cùng khuôn
 * [`onCellMouseUp`] ngay dưới — sản phẩm tự làm thứ engine không làm. Đây là **lần thứ hai**
 * kho này phải dựng đúng bản vá đó cho đúng engine đó.
 *
 * ⚠️ **KHÔNG `focus()` ô nguồn, khác hẳn [`onCellMouseUp`].** Cột nguyên văn không
 * `contenteditable` và không `tabindex` — một lượt `focus()` ở đây hoặc là no-op, hoặc dựng
 * một bề mặt tiêu điểm **thứ tư** trong lưới và chạm hợp đồng AD-34 §2. Đường này chỉ cần
 * một **toạ độ**, không cần tiêu điểm.
 *
 * ⚠️ `@mouseup`, **không** `@click` — cùng lý do `check-commands.mjs` Kiểm A đã ghi cho ô
 * bản dịch: `@click` là cửa của **thao tác**, còn đây là một lượt ghi nhận vị trí.
 *
 * ⚠️ **Không phân giải được ⇒ KHÔNG ghi nhận gì**, và không có nhánh dự phòng *"neo về cuối"*
 * như ô bản dịch. Một điểm cắt đoán bừa là một lượt tách sai chỗ **trên dữ liệu người dùng**,
 * và AD-5 không cho hoàn tác; còn không ghi nhận thì `⌘/` trả `'no-cut'` và người dùng bấm lại.
 */
function onSourceCellMouseUp(event: MouseEvent): void {
  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 2.9 · AC7 — `Mod`+click MỚI đánh dấu. Một cú bấm TRƠN không đánh dấu gì.
  // ═══════════════════════════════════════════════════════════════════════════════
  // Cột này mang **hai** cử chỉ chuột cùng lúc, và trước lượt này cả hai treo trên cùng một
  // `mouseup` trần: đánh dấu chỗ cắt *(Story 2.8)* và **tra từ điển** *(FR21, Story 1.18, đã
  // phát hành — `useSelectionSurface(gridRoot, 'source', …)` ngay trên)*.
  // ⇒ Mỗi lượt tra một từ để **đọc** cũng rơi một dấu cắt; và một cú **double-click** bắn
  //   **HAI** `mouseup` nên nó để lại **hai** dấu. Người dùng đặt chỗ cắt cho một lượt `⌘/`
  //   họ không định gọi, và AD-5 không cho hoàn tác lượt tách đó.
  //
  // ✅ Ice ký 2026-08-17 — một lượt lật tìm ra bằng cách **DÙNG THẬT**, đúng khuôn đã lặp ở
  //    2.5b *(hàng về hưu trong lưới)* và 2.8 *(chữ ký #6 lật lần thứ ba)*. Cùng lượt ký đó
  //    xác nhận **double-click TRA ĐƯỢC** trên máy thật ⇒ đóng món nợ 🔴 `deferred-work.md §*Deferred from: code review of 2-5d-ngat-doan-ban-dich (2026-08-16)*`.
  //
  // 🔴 `hasPrimaryModifier`, **KHÔNG** `event.metaKey` — §Trap 1 của `keys.ts`, và
  //    `commands/README.md:73` cấm bằng chữ. Nửa Windows của kho không có đường nghiệm thu
  //    tại chỗ, nên một `metaKey` trần đi qua **cả hai nền tảng của CI** rồi hỏng ở tay người
  //    dùng Windows. Vị từ sống ở `editorSegments.ts` và nhận nền tảng qua **tham số**.
  if (!hasPrimaryModifier(event, PLATFORM)) return

  const from = event.target instanceof Element ? event.target : event.target instanceof Node ? event.target.parentElement : null
  const cell = from?.closest<HTMLElement>('[data-col="src"]') ?? null
  const id = segmentIdOf(cell)
  if (cell === null || id === null) return

  const diem = caretPointAt(event.clientX, event.clientY)
  if (diem === null) return
  const { node, offset } = diem
  // 🔴 **PHÂN GIẢI ĐƯỢC KHÔNG PHẢI ĐÚNG CHỖ** — cùng cái bẫy mà `placeCaretAtPoint` đã ghi và
  // chính bộ e2e bắt được ngày 2026-08-12: một vị trí trỏ **ra ngoài** ô là "thành công" theo
  // API mà nó thuộc về một câu khác.
  if (!cell.contains(node)) return

  // 🔴 Phép ánh xạ offset → chỉ số ký tự sống ở `editorSegments.ts` — một **module thuần**
  // kiểm được bằng `vitest` + `happy-dom`. Một mệnh đề nằm trong thân handler này thì chỉ
  // kiểm được bằng e2e. Cùng lý do `segmentNavigation.ts` tồn tại.
  const cut = sourceCutOffsetOf(cell, node, offset)
  if (cut === null) return
  setEditorSourceCut(id, cut)
}

/**
 * `mousedown` — dời **vùng gõ** sang ô vừa bấm.
 *
 * 🔵 **KHÁC `EditorPanel.vue`, và khác vì một lý do cấu trúc chứ không vì một lượt bỏ sót:**
 * bản cũ phải `setAttribute('contenteditable', 'true')` **đồng bộ ngay tại đây**, vì lúc engine
 * xử lý cú bấm thì `<span>` **chưa** soạn thảo được *(Vue vá DOM ở một microtask sau)*. Trong
 * lưới **mọi ô đã soạn thảo được từ đầu** ⇒ câu hỏi đó biến mất, và dòng `setAttribute` cùng
 * với nó. Giữ lại nó ở đây sẽ là một dòng không ai lần được lý do.
 *
 * ⚠️ **Không** `preventDefault()`: hành vi mặc định là thứ đặt caret trên các engine **có**
 * làm việc đó (Blink). Đường chuột dưới đây **thêm** một đường, nó không thay đường nào.
 */
function onCellMouseDown(event: MouseEvent): void {
  const cell = targetCellOf(event.target instanceof Node ? event.target : null)
  const id = segmentIdOf(cell)
  if (id !== null) setEditorCaret(id)
}

/**
 * `mouseup` — đặt caret vào đúng điểm vừa bấm, **sau** khi engine đã xử lý xong cú bấm.
 *
 * ⚠️ `@mouseup`, **không** `@click`: `check-commands.mjs` Kiểm A cưỡng chế AD-34 §1 —
 * *"handler chuột chỉ được `dispatch` một command đã đăng ký"* — và nó canh đúng thuộc tính
 * `@click`. Một `@click` ở đây làm cổng đó **đỏ**, và đúng ra nó phải đỏ: `@click` là cửa của
 * **thao tác**, còn đây là một lượt đặt con trỏ (B8).
 *
 * 🔴 **THỨ TỰ NÀY LÀ MỘT PHÉP ĐO, và bản đầu của Story 2.3 đã sai nó.** Đặt caret trong một
 * `nextTick` ngay sau `mousedown` chạy **trước `mouseup`**, rồi lượt xử lý cú bấm của engine
 * **thu vùng chọn về không** — `rangeCount = 0`, và mọi lượt chèn văn bản trả `false`.
 */
function onCellMouseUp(event: MouseEvent): void {
  const cell = targetCellOf(event.target instanceof Node ? event.target : null)
  if (cell === null) return

  // 🔴 **`focus()` TƯỜNG MINH, VÀ NÓ LÀ MỘT PHÉP ĐO — không một dòng phòng xa.**
  //
  // Bàn đo Task 1.2 chạy lưới trong một lớp phủ **không có tổ tiên nào focus được**, và ở đó
  // `setPosition` một mình đủ: caret hiện, `activeElement` thành chính ô. Trong **sản phẩm**
  // thì không, vì `PanelFrame` dựng một `section[tabindex="-1"]` cho AD-34 §2 — một tổ tiên
  // focus được, và WKWebView trao tiêu điểm cho **nó**.
  //
  // **Đo 2026-08-15 trong cửa sổ Tauri thật, sau một cú bấm chuột thật vào ô rỗng:**
  //
  // | | `getSelection().type` | `document.activeElement` |
  // |---|---|---|
  // | `setPosition(cell, 0)` một mình | `"Caret"` | **`SECTION`** *(gốc panel)* |
  // | `cell.focus()` **rồi** `setPosition` | `"Caret"` | **`DIV`** *(chính ô)* |
  // | *(để engine tự lo)* | **`"None"`**, `rangeCount 0` | `SECTION.mode` |
  //
  // ⇒ Không có dòng này, người dùng bấm vào một ô và **không gì xảy ra**: engine thu vùng
  // chọn về không ngay trong cùng lượt sự kiện, vì editing host chưa bao giờ nhận tiêu điểm.
  //
  // 🔵 **Và đây là chỗ bàn đo NÓI THIẾU, ghi ra thay vì để người sau vấp lại.** Nó dựng đúng
  // hình dạng lưới nhưng **không** dựng lại ngữ cảnh panel, nên nó bỏ lọt đúng biến quyết
  // định. Cùng lớp bài học *"trúng tiền đề chưa phải trúng cơ chế"* mà §Bài học của story
  // ghi tên — lần này bàn đo trúng tiền đề, và **e2e trong WKWebView thật** bắt được cơ chế.
  //
  // ⚠️ `EditorPanel.vue:481-488` ghi rằng `el.focus()` trên `<span contenteditable>` **không**
  // giữ được tiêu điểm. Mệnh đề đó **không mâu thuẫn**: một `<span>` inline rỗng khác một
  // `<div>` khối có `min-height`. Đo lại ở hình dạng mới thay vì mượn kết luận cũ.
  cell.focus()

  // Không phân giải được điểm bấm vào TRONG ô ⇒ neo vào **CUỐI** ô.
  //
  // ⚠️ Cuối chứ không đầu: bấm vào khoảng trống sau một câu là cách người ta nói *"cho tôi
  // viết tiếp"*. Neo về đầu ở đó làm chữ mới chèn trước chữ cũ.
  // ⚠️ Nhánh này KHÔNG lý thuyết — một ô **chưa dịch** không có text node nào, và nó là ô đầu
  // tiên người dùng bấm vào ở mọi Chương mới. `childNodes.length` với ô rỗng là `0`, tức
  // chính nó.
  if (!placeCaretAtPoint(cell, event.clientX, event.clientY)) {
    setCaret(cell, cell.childNodes.length)
  }

  // 🔴 **LƯỢT VÁ Ở FRAME KẾ TIẾP CHẠY VÔ ĐIỀU KIỆN — và bản đầu của story này đã sai chỗ đó.**
  //
  // Bản đầu chỉ hẹn lượt vá ở **nhánh trượt** của `placeCaretAtPoint`, đúng khuôn
  // `EditorPanel.vue`. Đo 2026-08-15 trong WKWebView thật: khi hit-test **phân giải được**
  // *(nhánh "thành công")*, lượt xử lý cú bấm của engine vẫn chạy **sau** handler này và vẫn
  // trao tiêu điểm cho `SECTION.panel` — `activeElement = SECTION.panel.focused`,
  // `selection.type = "None"`, `rangeCount = 0`. Tức nhánh **thành công** là nhánh hỏng, và
  // một lượt vá gắn vào nhánh thất bại **không bao giờ chạy tới**.
  //
  // ⇒ Cả hai nhánh đều hẹn một lượt kiểm ở frame sau. Nó là một no-op quan sát được khi engine
  // đã làm đúng *(Blink)*, và là đường duy nhất chạy được khi engine không làm *(WKWebView)*.
  ensureCaretNextFrame(cell)
}

/**
 * Đặt lại caret vào ô vừa bấm — **HAI lượt, và lượt thứ hai có một nguyên nhân ĐỌC ĐƯỢC**.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════
 * 🔴 AI GIÀNH TIÊU ĐIỂM Ở ĐÂY — và lần này thì **CÓ** người giành, khác hẳn Story 2.3
 * ═══════════════════════════════════════════════════════════════════════════════════
 * Story 2.3 chẩn đoán *"AD-34 giành tiêu điểm"* và **sai** — đo được là **không ai giành cả**.
 * Ở lưới, cùng câu hỏi có câu trả lời **ngược lại**, và nó đọc được từ mã chứ không từ suy
 * đoán: `WorkspaceDock.vue:591-611` nghe `onDidActivePanelChange` và, với `origin === 'user'`,
 * gọi `enterFocus(id)` — tức **dời tiêu điểm DOM về gốc panel một cách tường minh**.
 *
 * Cú bấm ĐẦU TIÊN vào lưới **kích hoạt** panel đó trong dockview ⇒ lượt dời ấy chạy **sau**
 * handler `mouseup` của ta và **sau** cả lượt vá ở frame kế tiếp.
 *
 * **Đo 2026-08-15, cửa sổ Tauri thật, chuột thật, ô rỗng:**
 *
 * | Sau | `activeElement` | `selection.type` |
 * |---|---|---|
 * | `mouseup` handler *(đã `focus()` + `setPosition`)* | `SECTION.panel.focused` | `"None"` |
 * | một lượt `browser.execute` **riêng** *(tức một task sau)* | `DIV` *(chính ô)* | `"Caret"` |
 *
 * ⇒ Cơ chế **chạy được**; thứ thiếu là **thời điểm**. Lượt vá phải đứng sau lượt dời của
 * AD-34, và `requestAnimationFrame` một mình **không** đủ.
 *
 * 🔴 **Và đây KHÔNG phải một lượt lật AD-34.** Mệnh đề của AD-34 §2 — *"chuyển panel phải dời
 * focus DOM tường minh"* — vẫn đúng từng chữ và **không sửa một dòng**. Thứ story này thêm là
 * một mệnh đề **hẹp hơn**, ở tầng dưới: *sau khi panel đã nhận tiêu điểm, con trỏ trả về đúng
 * ô người dùng vừa bấm.* Hai mệnh đề không mâu thuẫn — cái sau chỉ nói tiếp câu chuyện.
 *
 * ⚠️ **HAI lượt, có trần cứng, KHÔNG một vòng lặp tự phục hồi.** `focus.ts:114-118` đã ghi lý
 * do cho đúng hình dạng này: *"đừng sửa bằng cách focus lại vòng lặp — nó sẽ đánh nhau với
 * người dùng đang Tab và với hộp thoại của hệ điều hành."* Hai lượt đủ cho một lượt dời đã
 * biết; một lượt thứ ba là dấu hiệu nguyên nhân nằm ở chỗ khác, và lúc đó phải đo lại chứ
 * không nâng trần.
 *
 * ⚠️ Điều kiện thoát hỏi **CẢ HAI** vế. Đo được: `setPosition` một mình cho `type === "Caret"`
 * trong khi tiêu điểm vẫn ở gốc panel — engine thu vùng chọn về không ngay sau đó. Một guard
 * chỉ đọc `type` vì thế thoát sớm ở **đúng ca nó tồn tại để vá**.
 */
/**
 * Ô mà lượt vá đang **được phép** kéo tiêu điểm về — luôn là ô của cú bấm **gần nhất**.
 *
 * 🔵 **Thêm 2026-08-15 (code review).** Không có nó, `reassert` đóng ô cũ vào closure và
 * `settled()` không phân biệt được hai chuyện khác hẳn nhau:
 *   ① *"tiêu điểm chưa kịp ổn định"* — ca mà lượt vá tồn tại để chữa;
 *   ② *"người dùng vừa bấm sang một ô KHÁC"* — ca mà lượt vá phải im.
 * Cả hai đều cho `document.activeElement !== cell`, nên lượt vá của ô cũ gọi `focus()` và kéo
 * tiêu điểm **ngược lại** ô người dùng vừa rời.
 *
 * ⚠️ **Hậu quả thật NHỎ HƠN vẻ ngoài, ghi đúng mức thay vì thổi lên:** hai lượt vá xếp theo thứ
 * tự đăng ký, nên khi hai cú bấm rơi cùng một frame thì lượt của ô mới luôn chạy **sau** và
 * giành lại. Cửa sổ hỏng thật là hẹp — một cú bấm rơi vào giữa `requestAnimationFrame` và
 * `setTimeout(0)` của cú bấm trước — và biểu hiện là một **nháy tiêu điểm ~1 frame**, không một
 * cú bấm bị nuốt. Vá vì nó rẻ và vì `setCaret` trong lúc nháy có thể để vùng chọn ở một ô còn
 * tiêu điểm ở ô khác; **không** vá vì nó là một lỗi nặng.
 *
 * 🔴 **Đây là một GUARD, không phải lượt thử thứ BA.** `focus.ts:114-118` cấm *"focus lại vòng
 * lặp"* và story chốt **hai lượt, trần cứng**. Biến này không thêm lượt nào — nó chỉ làm hai
 * lượt đã có thôi bắn vào một mục tiêu đã cũ.
 */
let caretTarget: HTMLElement | null = null

/**
 * Chốt chống đệ quy cho nhánh ① của [`onBeforeInput`] — Story 2.5d, AD-46.
 *
 * 🔴 **Một biến module, KHÔNG một `ref`.** Nó không bao giờ đi vào render: vòng đời của nó
 * nằm gọn trong **một** lượt gọi `execCommand` đồng bộ *(bật → gọi → tắt trong `finally`)*,
 * và một `ref` ở đây chỉ mua thêm một lượt đánh thức reactivity cho một giá trị không ai
 * hiển thị. Cùng khuôn `caretTarget` ngay trên.
 *
 * ⚠️ `try`/`finally` chứ không hai dòng thẳng: nếu `execCommand` ném, một chốt kẹt ở `true`
 * sẽ làm **mọi** lượt `Enter` sau đó im lặng không làm gì — đúng lớp lỗi *"hỏng mà không ai
 * lần được"*.
 */
let insertingLineBreak = false

function ensureCaretNextFrame(cell: HTMLElement): void {
  caretTarget = cell

  const settled = (): boolean =>
    document.activeElement === cell && window.getSelection()?.type === 'Caret'

  const reassert = (): void => {
    if (!cell.isConnected) return
    // 🔵 Người dùng đã nhắm sang ô khác ⇒ lượt vá này nói về một thời điểm đã trôi qua. Im.
    if (caretTarget !== cell) return
    if (settled()) return
    cell.focus()
    setCaret(cell, cell.childNodes.length)
  }

  requestAnimationFrame(() => {
    reassert()
    /*
     * 🔴 **LƯỢT THỨ HAI Ở MACROTASK — VÀ NÓ **KHÔNG** PHẢI MÃ CHẾT SAU KHI SỬA `focus.ts`.**
     *
     * Nó được thêm vào để đấu với lượt `enterFocus` của AD-34; lượt đó nay đã đóng **tại gốc**
     * *(`focus.ts::enter()` bỏ qua khi tiêu điểm đã ở trong `el` — Ice ký 2026-08-15, đường A)*,
     * nên giả thuyết hợp lý là *"gỡ được rồi"*.
     *
     * **Phép đo BÁC giả thuyết đó, 2026-08-15:** gỡ lượt này ⇒
     * `e2e/specs/grid-empty-cell.e2e.mjs` **ĐỎ trở lại**; trả lại ⇒ **XANH**. Cùng bàn đo, cùng
     * máy, hai lượt liên tiếp.
     *
     * ⇒ Còn **một** nguồn thu vùng chọn nữa ngoài `enterFocus`, và nó chạy **ngoài** vòng
     * frame này. Nguồn đó **CHƯA được đặt tên** — ghi đúng mức độ chắc chắn thay vì gán cho
     * một nguyên nhân nghe hợp lý. *(Ứng viên chưa loại trừ: lượt xử lý cú bấm của chính
     * WKWebView, thứ Story 2.3 đã đo được là kéo dài qua `mouseup`.)*
     *
     * ⚠️ **HAI lượt, trần cứng, KHÔNG một vòng lặp** — `focus.ts:114-118`: *"đừng sửa bằng cách
     * focus lại vòng lặp"*. Một lượt thứ ba là dấu hiệu nguyên nhân nằm ở chỗ khác, và lúc đó
     * phải **đo lại** chứ không nâng trần.
     */
    setTimeout(reassert, 0)
  })
}

/** Rời tiêu điểm khỏi cột bản dịch ⇒ không câu nào *"đang sửa"* nữa. */
function onColumnFocusOut(event: FocusEvent): void {
  const next = event.relatedTarget
  if (next instanceof Node && (isInTargetColumn(next) || stripTrack.value?.contains(next))) return
  setEditorCaret(null)
}

function onStripFocusOut(event: FocusEvent): void {
  const next = event.relatedTarget
  if (next instanceof Node && (isInTargetColumn(next) || stripTrack.value?.contains(next))) return
  setEditorCaret(null)
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 DOM SỞ HỮU VĂN BẢN BẢN DỊCH. VUE KHÔNG. — bài học đắt nhất của Story 2.3
// ═════════════════════════════════════════════════════════════════════════════════
// Template render `s.target_text` — bản **LÚC NẠP**, một giá trị chỉ đổi ở một lượt nạp lại.
// Nó **không** render văn bản đang gõ. Hệ quả: trong suốt một phiên gõ, vnode của ô không bao
// giờ đổi, nên Vue **không bao giờ** chạm vào text node đó.
//
// Hai khuyết tật thật đã tới từ việc làm ngược lại, Ice bắt cả hai bằng mắt: ① gõ ngược từ
// phải sang trái *(Vue so vnode cũ với vnode mới, không so với DOM ⇒ ghi `textContent` ⇒ dựng
// lại text node ⇒ caret rơi về offset 0)*; ② chữ đã dịch **biến mất** khi bấm xuống dưới *(một
// `frozenText` dùng chung cho mọi câu)*. Nguyên nhân chung: **hai chủ sở hữu cho một text
// node**. Bên đúng là **DOM**, vì đó là bên người dùng đang gõ vào.
//
// ⚠️ Vế *"văn bản đang gõ sống sót một lượt tháo panel"* không mất — nó đổi đường, xem
// [`restoreEditedText`].

/**
 * Chép văn bản đang gõ từ state **ngược lại DOM** — chạy sau mỗi lượt dựng lại lưới.
 *
 * ⚠️ **Không** chạm ô đang được gõ THẬT *(tức đang giữ tiêu điểm)*: DOM ở đó đã đúng, và một
 * lượt ghi sẽ đá caret ra. Phép kiểm hỏi `document.activeElement` — sau một lượt mount lại,
 * mọi ô vẫn `contenteditable` *(chúng luôn thế)* nhưng tiêu điểm thì không, và đó đúng là
 * lượt cần khôi phục nhất.
 */
function restoreEditedText(): void {
  if (gridRoot.value === null) return
  const edited = editorEditedText.value
  if (edited.size === 0) return

  for (const [id, text] of edited) {
    const el = targetCellById(id)
    if (el === null || document.activeElement === el) continue
    if (el.textContent !== text) el.textContent = text
  }
  repaintProofread()
}

watch([() => editorSegments.value, gridRoot], restoreEditedText, { flush: 'post' })

/** Underlines are inline spans inside the cell; drawn only while the cell still holds the scanned text. */
function repaintProofread(): void {
  const host = gridRoot.value
  if (host === null || composing) return
  repaintProofreadHost(host, proofreadResults.value, isMeaningKind)
}

let composing = false
let repaintTimer: ReturnType<typeof setTimeout> | null = null

function scheduleRepaintProofread(): void {
  if (repaintTimer !== null) clearTimeout(repaintTimer)
  repaintTimer = setTimeout(() => {
    repaintTimer = null
    repaintProofread()
  }, 0)
}

function onCompositionStart(): void {
  composing = true
}

function onCompositionEnd(): void {
  composing = false
  scheduleRepaintProofread()
}

watch([proofreadResults, () => editorSegments.value, gridRoot], repaintProofread, { flush: 'post' })
onBeforeUnmount(() => {
  if (repaintTimer !== null) clearTimeout(repaintTimer)
  if (gridRoot.value !== null) unwrapAllProofread(gridRoot.value)
})

// ═════════════════════════════════════════════════════════════════════════════════
// Con trỏ theo NEO VÙNG CHỌN — vế bàn phím của AC2, và nguồn của vạch `primary`
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * 🔵 **ĐƠN GIẢN HƠN `EditorPanel.vue` HẲN MỘT KHỐI, và lý do là cấu trúc:** bản cũ phải chụp
 * `savedCaret` **trước** lượt đổi thuộc tính `contenteditable` rồi trả nó lại ở một watcher
 * `flush: 'post'`, vì lắp `contenteditable` lên một `<span>` bắt engine dựng một editing host
 * **mới** và lượt đó được phép **thả vùng chọn**. Trong lưới **không lượt đổi thuộc tính nào
 * xảy ra** ⇒ engine không thả gì ⇒ không có gì để chụp và trả lại.
 *
 * ⚠️ Neo **ngoài** cột bản dịch ⇒ caret về `null`. Đó là câu đúng theo nghĩa đen: người dùng
 * đang chọn ở cột nguyên văn hay ở một panel khác thì *"con trỏ ở đây"* là sai, và giữ vạch
 * `primary` sáng là để vạch nói dối.
 */
function onSelectionChange(): void {
  if (gridRoot.value === null) return
  const anchor = window.getSelection()?.anchorNode ?? null
  if (anchor === null || !isInTargetColumn(anchor)) return
  const id = segmentIdOf(targetCellOf(anchor))
  if (id !== null) setEditorCaret(id)
}

/**
 * 🔵 **Story 3.4b (bổ sung theo yêu cầu Ice, 2026-08-21) — vế TIÊU ĐIỂM của I/O Matrix "rê
 * chuột / đưa tiêu điểm lên dấu đã chốt".** Bản đầu của story chỉ cài `@mouseenter`/
 * `@mouseleave` và ghi nợ vế bàn phím — Ice bác: hàng đó nằm TRONG khối
 * `<frozen-after-approval>`, không phải phạm vi tự thu hẹp. `epics.md` §Story 3.4b còn đòi
 * "MỘT dòng StatusBar đạt CẢ hai vế bằng MỘT cơ chế".
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ĐẠT VẾ TIÊU ĐIỂM VỚI **0** TAB-STOP MỚI — cùng khuôn [`onSelectionChange`] ngay trên
 * ─────────────────────────────────────────────────────────────────────────────
 * `.hv-switch`/`.hv-parallel` đã mang `tabindex="0"` từ Story 1.18 (AC11); đường mở rộng vùng
 * chọn bằng bàn phím (`Selection.modify()`, `selectionCommands` ở `selectionContract.ts`)
 * chạy trên `window.getSelection()` HIỆN CÓ, không cần phần tử nào đang giữ tiêu điểm DOM —
 * đo được ở chính doc-comment của `modifySelection()` ("`tabindex="0"` trên một `<p>` không
 * sửa được" đã ĐO xong ở Story 1.18, hai engine). ⇒ Nghe `selectionchange` — đúng sự kiện mà
 * [`onSelectionChange`] đã dùng cho cột bản dịch — thay vì thêm một tab-stop mới cho cột
 * nguyên văn: **0** đổi cấu trúc DOM, **0** rủi ro cho AC6/AC11/AC12 đã ký.
 *
 * ⚠️ **Một CƠ CHẾ, hai đường vào**: hàm này và [`onGlossaryPieceEnter`] (chuột) đều ghi vào
 * ĐÚNG MỘT state ([`setHoveredGlossaryTerm`]/[`clearHoveredGlossaryTerm`]) — "một cơ chế" là
 * NƠI TÍN HIỆU HẠ CÁNH, không phải một loại sự kiện DOM duy nhất; chuột và bàn phím vốn dĩ là
 * hai loại sự kiện khác nhau ở tầng trình duyệt.
 *
 * ⚠️ **KHÁC [`onSelectionChange`] ở một điểm có chủ đích: hàm NÀY xoá tường minh khi
 * `span === null`, hàm KIA thì không.** `onSelectionChange` dựa vào `onColumnFocusOut` (cột
 * bản dịch CÓ `contenteditable`, tức focus DOM thật) để dọn khi rời cột. Cột nguyên văn không
 * có tín hiệu blur đáng tin cho đường chữ trần (`.src-piece` không `tabindex`) — nên đây là
 * nguồn XOÁ DUY NHẤT cho vế bàn phím, và nó phải tự xoá mỗi khi caret không còn đứng trên một
 * thuật ngữ, kể cả khi caret rời khỏi cột nguyên văn hoàn toàn.
 *
 * ⚠️ **Có thể ghi đè một lượt hover CHUỘT đang hiện, và ngược lại** — cả hai cùng một state,
 * "tín hiệu tới sau cùng thắng". Ca duy nhất lệch pha: bấm chuột vào một thuật ngữ A rồi, KHÔNG
 * di chuột, đưa chuột đứng yên trên thuật ngữ B từ một cú di chuyển TRƯỚC đó — `selectionchange`
 * của cú bấm đè hiển thị của B bằng A dù chuột vẫn hình học nằm trên B. Biên hiếm, chấp nhận
 * được: không cơ chế lớp-chồng nào được dựng thêm cho ca này.
 * 🔵 **SIẾT 2026-08-21 (rà ba lớp, P13) — ca RỘNG HƠN đã được CHẶN, không còn là "biên hiếm".**
 * Bản đầu của hàm này dọn hover vô điều kiện khi `cell === null` — tức MỌI phím gõ ở cột bản
 * dịch (`selectionchange` bắn theo từng ký tự trong `contenteditable`) xoá một hover chuột
 * đang hiện, HOÀN TOÀN không liên quan tới cột nguồn. Đó KHÔNG phải "biên hiếm", nó là đường
 * THƯỜNG NGÀY nhất của ứng dụng (gõ dịch trong khi đọc lại nguyên văn). Chốt `if (cell ===
 * null) return` ở trên đóng đúng ca đó. Ca lệch pha CÒN LẠI (mô tả ngay trên) hẹp hơn nhiều:
 * nó đòi một cú BẤM (không phải gõ phím) rơi đúng VÀO cột nguồn mà KHÔNG trúng thuật ngữ nào,
 * trong khi chuột đang hình học nằm trên một thuật ngữ KHÁC — vẫn chấp nhận được, không sửa.
 *
 * ⚠️ Dùng chung ĐÚNG neo `data-src-start`/`sourceCutOffsetOf` mà click-để-cắt (Story 2.8/2.9)
 * đã dùng — một hàm ánh xạ offset, hai chỗ tiêu thụ, KHÔNG một đường thứ hai.
 */
function onSourceSelectionChange(): void {
  const selection = window.getSelection()
  sourceSelecting.value = isSourceColumnSelecting(selection, gridRoot.value)
  const anchor = selection?.anchorNode ?? null
  const from = anchor === null ? null : anchor instanceof Element ? anchor : anchor.parentElement
  const cell = from?.closest<HTMLElement>('[data-col="src"]') ?? null

  // 🔴 **SIẾT 2026-08-21 (rà ba lớp, P13) — CHỈ dọn khi vùng chọn THẬT SỰ ở trong cột nguồn mà
  // KHÔNG trúng thuật ngữ nào. KHÔNG dọn khi vùng chọn ở NGOÀI cột nguồn.**
  //
  // `selectionchange` bắn ở **document**, tức bắn cho MỌI lượt đổi vùng chọn trong app — kể cả
  // mỗi phím gõ vào Ô BẢN DỊCH (`contenteditable`, `selectionchange` bắn theo từng ký tự). Bản
  // đầu của hàm này gọi `clearHoveredGlossaryTerm()` vô điều kiện khi `cell === null`, tức MỖI
  // phím gõ ở cột bản dịch xoá một dấu đang hiện do CHUỘT đặt (đứng yên trên một thuật ngữ,
  // không di chuyển) — chuột không bắn lại `@mouseenter` vì nó không di chuyển, nên `StatusBar`
  // rơi về "Đã lưu…" giữa chừng một cú gõ hoàn toàn không liên quan tới cột nguồn.
  //
  // ⇒ Phân biệt HAI ca `cell === null`, chỉ MỘT ca được dọn:
  //   ① vùng chọn **ở NGOÀI cột nguồn** (gõ ở cột bản dịch, click nơi khác) ⇒ đây KHÔNG phải
  //      tín hiệu về thuật ngữ — bỏ qua, KHÔNG đụng `hoveredGlossaryTerm` (chuột vẫn giữ vai).
  //   ② vùng chọn **Ở TRONG cột nguồn** nhưng không trúng dấu nào (`span === null` trong khi
  //      `cell !== null`) ⇒ đây LÀ tín hiệu thật "caret vừa RỜI một thuật ngữ" — dọn.
  if (cell === null) return

  const id = segmentIdOf(cell)
  const offset =
    anchor === null || id === null ? null : sourceCutOffsetOf(cell, anchor, selection?.anchorOffset ?? 0)
  const span = id === null || offset === null ? null : glossarySpanAtPoint(id, offset)

  if (span === null) {
    clearHoveredGlossaryTerm()
    return
  }
  setHoveredGlossaryTerm({ isConfirmed: span.isConfirmed, translation: span.translation })
}

onMounted(() => {
  document.addEventListener('selectionchange', onSelectionChange)
  document.addEventListener('selectionchange', onSourceSelectionChange)
})
onBeforeUnmount(() => {
  document.removeEventListener('selectionchange', onSelectionChange)
  document.removeEventListener('selectionchange', onSourceSelectionChange)
})

/**
 * Đặt caret vào **đầu** một ô theo một yêu cầu **của chương trình** — Story 2.5, Quyết
 * định #1(a): `⌘Enter` xác nhận rồi **sang câu kế**.
 *
 * ⚠️ Chỉ chạy khi `editorCaretPlacement` được đặt, tức chỉ trên đường **lệnh**. Đường chuột
 * không đụng tới nó — nới nhánh này cho cả đường chuột sẽ kéo caret về đầu ô sau một cú bấm
 * vào giữa ô, đúng khuyết tật mà Story 2.3 tốn ba lượt chẩn đoán để làm đúng.
 *
 * 🔵 `focus()` ở đây nay là **đủ**, và đó cũng là một hệ quả của lưới: ô đã soạn thảo được
 * sẵn, nên không cần chờ Vue vá thuộc tính. `flush: 'post'` vẫn giữ — ô của câu kế có thể vừa
 * mới được dựng ở chính lượt render này.
 */
watch(
  editorCaretPlacement,
  (id) => {
    if (id === null) return
    clearEditorCaretPlacement()
    const target = targetCellById(id)
    if (target === null) return
    // 🔵 **STORY 2.10 · AC8 NỬA SAU — `focus()` LÀ VẾ CUỘN, và nó ĐÃ Ở ĐÂY TỪ TRƯỚC.**
    //
    // Story 2.10 mở đầu bằng mệnh đề *"cuộn tới hàng — CHƯA CÓ, Task 4"* *(`grep scrollIntoView`
    // trên tệp này cho 0 kết quả)*. **Phép đo bác mệnh đề ấy**: `focus()` trên một phần tử ngoài
    // vùng nhìn tự cuộn nó vào, và nó cuộn **khéo hơn** đường tự cài — xem khối §AC8 ở dưới.
    // ⇒ Không một dòng cuộn nào được thêm ở đây, và đó là **kết quả của ba lượt đo**, không một
    //   lượt bỏ sót. Ice ký 2026-08-18.
    target.focus()
    setCaret(target.firstChild ?? target, 0)
  },
  { flush: 'post' },
)

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 AC8 NỬA SAU — VÌ SAO KHÔNG CÓ HÀM CUỘN NÀO Ở TỆP NÀY. Story 2.10, Ice ký 2026-08-18
// ═════════════════════════════════════════════════════════════════════════════════
// Story 2.10 giao Task 4 *"cuộn tới hàng, tức thì, không hiệu ứng"* với tiền đề
// *"CHƯA CÓ — `grep scrollIntoView|scrollTop` trên tệp này = 0 kết quả"*. Tiền đề đó **sai**, và
// nó sai theo đúng cách mà AC8 **nửa đầu** đã sai: cả hai vế đã được dựng sẵn ở nơi khác.
// Nửa đầu do `ruleOf` lo; nửa sau do `focus()` lo.
//
// ─────────────────────────────────────────────────────────────────────────────
// BA PHÉP ĐO, và phép thứ nhất là một ĐỘT BIẾN chứ không một lượt đọc mã
// ─────────────────────────────────────────────────────────────────────────────
// ① Bản đầu cài `cuonToiHang()` *(tính `scrollTop` bằng tay, ngữ nghĩa nearest)* và ca e2e
//    §Ⓒ xanh. Task 7.3 **gỡ lời gọi đó** khỏi mã sản phẩm rồi chạy lại: **4/4 vẫn xanh**.
// ② Thêm `focus({ preventScroll: true })` để buộc công thức phải làm việc, rồi gỡ lại
//    `preventScroll`: **4/4 vẫn xanh**, kể cả ca Ⓔ *(ngữ nghĩa nearest)*.
// ③ Bàn đo `2-10-ban-do/focus-co-tu-cuon-khong.e2e.mjs` (WKWebView 605.1.15, hàng 50/60):
//
//    | Lượt | `scrollTop` sau | hàng nằm trọn | `SECTION.panel` |
//    |---|---|---|---|
//    | `focus()` một mình | **1569** | ✅ | 0 — không đụng |
//    | `focus({preventScroll:true})` *(đối chứng ÂM)* | 0 | ❌ | 0 |
//    | `focus({preventScroll:true})` + công thức | **1242** | ✅ | 0 |
//
// 🔴 **Và chênh lệch 1569 vs 1242 KHÔNG phải một khuyết tật của `focus()` — nó là ưu điểm.**
// WebKit **căn giữa** khi hàng đích ở **xa** *(1569 ≈ giữa; đối chứng độc lập: `block:'center'`
// đo 1571)*, và dùng **nearest** khi nó chỉ vừa ló khỏi mép *(đo ở ca Ⓔ: dịch đúng **38 px** =
// một chiều cao hàng)*. ⇒ Nhảy xa thì người dùng có ngữ cảnh trên dưới; bấm liên tục thì vùng
// nhìn không giật. Công thức tự cài **ép nearest ở mọi ca**, tức nó dán hàng đích vào sát mép
// dưới sau một lượt nhảy xa — **xấu hơn**.
//
// ⇒ Giữ lại công thức là giữ một hàm mà **không phép đo nào của dự án phân biệt được**, để đổi
//   lấy một hành vi **kém hơn**. Kho này gọi đó là mã chết, và §Miễn trừ của `project-context.md`
//   cấm đúng hình dạng ấy.
//
// ⚠️ **CÁI GIÁ, ghi ra thay vì để người sau tự phát hiện:** AC8 nửa sau nay dựa vào **hành vi
// engine**, không vào một dòng mã đọc được ở đây. Không chuẩn nào bảo đảm nó, và không cổng nào
// canh nó — một bản WebKit sau có thể đổi. Lưới thật là ca Ⓒ + Ⓔ của
// `e2e/specs/segment-navigation.e2e.mjs` *(chạy tay)*, và món nợ đã vào `deferred-work.md` kèm
// chủ. 🔴 **Đừng thêm `preventScroll` vào lời gọi `focus()` ở trên** — làm thế là tắt vế cuộn
// duy nhất đang có, và mọi ca nghiệm thu tự động vẫn xanh.

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 CỬA DUY NHẤT MÀ MỘT LƯỢT SỬA VĂN BẢN ĐI QUA — chặn theo `inputType`, không theo phím
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * `beforeinput` là điểm chặn mà chuẩn Input Events **bảo đảm** `cancelable`, và nó phủ cả
 * những đường không có phím nào: dán bằng menu chuột phải, kéo-thả văn bản, và lượt thay thế
 * của bộ kiểm chính tả hệ điều hành. Một handler bám `keydown` bỏ lọt trọn ba đường đó.
 *
 * Ba nhóm, ba xử lý:
 * ① **Cấu trúc đoạn** (`insertParagraph`) ⇒ **chặn, rồi tự phát một `insertLineBreak`**.
 *    🔵 **CẬP NHẬT 2026-08-16 (Story 2.5d, AD-46) — nhánh này ĐÃ ĐỔI NGHĨA.** Bản cũ chặn
 *    cả `insertParagraph` lẫn `insertLineBreak` và **không xử lý gì thêm**, với dòng dặn
 *    *"Quyền xuống dòng trong ô bản dịch là FR134/AD-46, Story 2.5d. Đừng mở sớm."* —
 *    story đó là story này, và cửa nay **mở**, nhưng **chỉ ở ô bản dịch** (AC6).
 *    ⚠️ Vế của AD-37 **không** bị nới một chữ: `is_paragraph_end` vẫn là dữ liệu ĐÃ LƯU,
 *    và một lượt xuống dòng ở đây **không** tách câu, **không** đụng cờ nào. Nó chỉ thêm
 *    một ký tự `\n` vào `target_text` của **đúng một** segment. Cấu trúc đoạn của bản dịch
 *    sống ở cột `is_target_paragraph_end` (bước di trú 9), không ở ký tự này.
 * ② **Chèn từ một nguồn NGOÀI bàn phím** ⇒ **chặn, rồi tự chèn văn bản THUẦN đã làm phẳng**.
 *    Đo được: không có nhánh này thì cả hai engine tiêm markup — `<pre>`, `<span style>`, và
 *    trên WebKit cả `<div>` khối — cộng một `\n` thật vào `target_text`.
 * ③ **Mọi thứ còn lại** (`insertText`, các lượt xoá) ⇒ **để engine làm**.
 *
 * 🔵 **Một mệnh đề của `EditorPanel.vue` HẾT ĐÚNG ở đây:** bản cũ ghi *"② chặn cả khả năng
 * gộp hai `<span>`"*. Trong lưới, một vùng chọn **không tràn qua ranh giới ô được** — mỗi ô
 * là một editing host riêng, và trình duyệt không cho một `Range` soạn thảo bắc cầu hai host.
 * Nhánh ② **ở lại** vì vế **markup** của nó vẫn đúng từng chữ.
 */
function onBeforeInput(event: InputEvent): void {
  const cell = targetCellOf(event.target instanceof Node ? event.target : null)
  if (cell === null) return
  if (!composing && !event.isComposing) {
    unwrapProofread(cell, event.inputType === 'historyUndo' || event.inputType === 'historyRedo')
  }

  // ① Cấu trúc đoạn — AD-37 giữ cờ nguồn; AD-46 mở quyền xuống dòng ở ô bản dịch.
  //
  // 🔴 **`insertLineBreak` ĐI QUA, `insertParagraph` bị đổi thành `insertLineBreak`** — và
  // đây là một PHÉP ĐO trên WKWebView thật (bàn đo Task 1, `2-5d-ban-do/README.md`,
  // 2026-08-15), không một sở thích. Bốn số quyết định hình dạng này:
  //
  //   • thả `insertParagraph` chạy ⇒ engine dựng `A<div>B</div>`, và `cell.textContent`
  //     đọc ra **`"AB"`** — DOM hai dòng, đĩa MỘT chuỗi liền. AC1 hỏng ở vế *"ký tự lưu vào
  //     `target_text`"* mà **không cổng nào đỏ**;
  //   • `insertLineBreak` dưới `white-space: pre-line` ⇒ engine dựng **text node `"\n"`**,
  //     `0` phần tử con, `textContent === "A\nB"`. Đúng thứ đường ghi cần;
  //   • ở **cuối** nội dung engine tự thêm một `\n` canh chót ⇒ ô vẽ ra **2 dòng** thật.
  //     Đường `<br>` phải tự làm việc đó: đo được `A<br>` = **1 dòng**, `A<br><br>` = 2;
  //   • đối chứng: **không** `pre-line` thì chính `insertLineBreak` lại dựng `<br>` và
  //     `textContent` về lại `"AB"`.
  //
  // ⇒ 🔴 `white-space: pre-line` ở `<style scoped>` **KHÔNG phải một dòng trang trí** — nó
  // là **tiền đề vận hành** của nhánh này. Đổi nó là lật hình dạng DOM mà engine dựng, và
  // đường ghi im lặng mất `\n`. Hai chỗ đó phải đọc cùng nhau.
  if (event.inputType === 'insertParagraph') {
    event.preventDefault()
    // 🔴 CHỐT CHỐNG ĐỆ QUY. `execCommand('insertLineBreak')` tự phát một `beforeinput`
    // `insertLineBreak` **đồng bộ**, và nó đi qua đúng handler này. Không có chốt thì lượt
    // thứ hai lại rơi vào nhánh ① — hôm nay vô hại vì `insertLineBreak` không còn bị chặn,
    // nhưng nó là một vòng chờ sẵn cho lượt sửa kế tiếp. Đo được: đúng **một** sự kiện phát
    // ra mỗi lượt (bàn đo E1).
    if (insertingLineBreak) return
    insertingLineBreak = true
    // 🔴 GIÁ TRỊ TRẢ VỀ ĐƯỢC ĐỌC — code review 2026-08-16. `execCommand` là một API đã bị
    // khai tử trong đặc tả, và hợp đồng của nó là trả `false` khi trượt, KHÔNG ném. Chốt
    // `insertingLineBreak` ở trên chỉ canh ca NÉM (nó nằm trong `finally`). Không đọc giá
    // trị trả về thì một lượt trả `false` làm `Enter` biến mất **không dấu vết**:
    // `preventDefault()` đã chạy ở trên, nên engine cũng không còn làm gì nữa.
    // ⚠️ Bàn đo chứng minh nó chạy trên WKWebView; nửa Blink là món nợ đã ghi có chủ. Dòng
    // chẩn đoán này là thứ phân biệt *"phím không ăn"* với *"phím ăn rồi nhưng mất chữ"*.
    let inserted = false
    try {
      inserted = document.execCommand('insertLineBreak')
    } finally {
      insertingLineBreak = false
    }
    if (!inserted) {
      // ⚠️ Chẩn đoán viết bằng tiếng Anh, KHÔNG tiếng Việt: Kiểm A của `check-i18n.mjs` cấm
      // chuỗi tiếng Việt ở vị trí mã trong `.vue` và `.rs`.
      console.warn('[grid] insertLineBreak refused by the engine — the newline was dropped')
      scheduleRepaintProofread()
    }
    // `preventDefault()` đã cắt lượt `input` của engine cho sự kiện GỐC, nhưng lượt
    // `insertLineBreak` ở trên phát `input` của chính nó và `onEditInput` bắt được — nên
    // KHÔNG gọi `reportEdit` ở đây, gọi là ghi hai lần cùng một nội dung.
    return
  }
  if (event.inputType === 'insertLineBreak') return

  // ② Chèn từ ngoài bàn phím.
  const FROM_OUTSIDE = ['insertFromPaste', 'insertFromDrop', 'insertReplacementText']
  if (!FROM_OUTSIDE.includes(event.inputType)) return

  // 🔴 `preventDefault()` đứng TRƯỚC mọi cửa thoát — code review 2026-08-13. Cám dỗ là dời nó
  // xuống sau các guard để một lượt dán không bao giờ "biến mất". Đừng: không chặn nghĩa là
  // thả engine chạy hành vi dán mặc định, đúng đường đo được là tiêm markup. Một lượt dán
  // trượt thì người dùng dán lại; một `data-segment-id` mất là hỏng **vĩnh viễn** (AD-3).
  event.preventDefault()
  const raw = event.dataTransfer?.getData('text/plain') ?? event.data ?? ''
  // 🔵 CẬP NHẬT 2026-08-16 (code review, Ice ký đường (b)) — LƯỢT DÁN NAY GIỮ `\n`.
  //
  // Bản cũ viết: *"Xuống dòng → **một khoảng trắng**, không bị bỏ đi: dán hai đoạn vào một ô
  // thì hai chữ ở hai đầu ranh giới phải còn cách nhau, nếu không chúng dính thành một từ
  // không tồn tại"*. Lý lẽ đó đúng **khi nó được viết**, và nó đúng vì một tiền đề đã hết
  // hiệu lực: hồi đó `\n` **không thể** tồn tại trong ô, nên giữ nó lại nghĩa là mất nó.
  // AC1 của Story 2.5d vừa làm `\n` thành một ký tự hợp lệ của `target_text`.
  //
  // 🔴 Vế mà lý lẽ cũ bảo vệ **vẫn được giữ nguyên**: hai chữ ở hai đầu ranh giới vẫn cách
  // nhau — bằng chính `\n`, một dấu tách thật, thay vì bị nuốt. Thứ đã đổi là `\n` nay tới
  // được đĩa; thứ KHÔNG đổi là phép gộp khoảng trắng ngang (`[ \t]+` → một dấu cách).
  //
  // ⚠️ Lớp chặn tiêm markup của nhánh ② **không** bị nới một ly: `preventDefault()` vẫn chạy
  // ở trên và văn bản vẫn do chính hàm này chèn bằng `insertNode`, engine không được thả.
  // ⚠️ `\r\n` và `\r` chuẩn hoá về `\n` — một ô mang `\r` sẽ cho `textContent` lệch khỏi
  // chuỗi trên đĩa ở lượt đọc lại, và `pre-line` không vẽ `\r` thành gì cả.
  // 🔴 Vế thị giác *"`\n` dán vào hiện ra hai dòng thật trên WKWebView"* CHƯA ĐO — ghi nợ có
  // chủ ở `deferred-work.md`, không tự chấm đạt. Vế dữ liệu thì có lưới vitest.
  const flat = raw.replace(/\r\n?/g, '\n').replace(/[ \t]+/g, ' ')
  // ⚠️ Vẫn là `=== ''`, KHÔNG `.trim() === ''`: lượt vá 2026-08-16 chỉ được Ice ký cho vế
  // *"dán giữ `\n`"*. Đổi cửa thoát này thành `trim()` là lặng lẽ thêm một quyết định thứ
  // hai — một lượt dán đúng một `\n` sẽ biến mất thay vì chèn một lần xuống dòng.
  if (flat === '') {
    scheduleRepaintProofread()
    return
  }

  const selection = window.getSelection()
  // ⚠️ Chẩn đoán viết bằng tiếng Anh, KHÔNG tiếng Việt: Kiểm A của `check-i18n.mjs` cấm chuỗi
  // tiếng Việt ở vị trí mã trong `.vue` và `.rs`.
  if (selection === null || selection.rangeCount === 0) {
    console.warn('[grid] insert skipped: no readable selection')
    scheduleRepaintProofread()
    return
  }
  const range = selection.getRangeAt(0)
  if (!cell.contains(range.startContainer)) {
    console.warn(
      '[grid] insert skipped: selection starts outside the edited cell ' +
        `(inputType=${event.inputType}) — reselect within one cell and paste again`,
    )
    scheduleRepaintProofread()
    return
  }

  range.deleteContents()
  const node = document.createTextNode(flat)
  range.insertNode(node)
  range.setStartAfter(node)
  range.collapse(true)
  selection.removeAllRanges()
  selection.addRange(range)

  // `preventDefault()` đã cắt lượt `input` của engine, nên đường ghi phải được gọi tay.
  reportEdit(cell)
}

/**
 * AC11 — `Enter` **trơn KHÔNG BAO GIỜ xác nhận**. 🔵 Vế *"và nó cũng không xuống dòng"* đã
 * **hết đúng từ 2026-08-16** (Story 2.5d, FR134/AD-46): trong ô bản dịch nó **có** xuống
 * dòng. Sửa tại chỗ thay vì để mệnh đề lặng lẽ sai.
 *
 * ⚠️ Hai lớp cho hai thứ khác nhau, **không** hai nguồn sự thật: nhánh ① của [`onBeforeInput`]
 * xử lý **lượt sửa DOM**; hàm này canh **sự kiện phím** trước khi nó rơi xuống `keys.ts`,
 * nơi một `Enter` trần sẽ được đem đi so với bảng hợp âm.
 *
 * 🔵 **VÌ SAO LỚP NÀY KHÔNG CÒN `preventDefault()` — và vì sao AC11 vẫn đứng.**
 * Trước 2.5d, dòng `if (event.key === 'Enter') event.preventDefault()` làm **hai** việc
 * cùng lúc: chặn xuống dòng, **và** chặn `Enter` rơi xuống bảng hợp âm. Story này cần bỏ
 * việc thứ nhất mà giữ việc thứ hai — nên câu hỏi thật là *"ai giữ việc thứ hai?"*.
 *
 * 🔵 **CẬP NHẬT 2026-08-16 (code review) — câu trả lời cũ ở đây GỌI SAI CƠ CHẾ, sửa tại chỗ.**
 * Bản đầu viết: *"`keys.ts:510` bỏ qua mọi hợp âm không mod khi `isTypingZone(event.target)`
 * ⇒ `Enter` trần không bao giờ tới bảng hợp âm"*. Đọc lại `keys.ts:508-510` thì **thứ tự mới
 * là thứ quyết định**, và `isTypingZone` **không bao giờ được chạm tới** cho một `Enter` trần:
 *
 * ```
 * for (const entry of compiled) {
 *   if (entry.code !== event.code || !sameMods(entry.mods, mods)) continue   // ← thoát ở ĐÂY
 *   if (lacksPrimaryMod(entry.mods) && isTypingZone(event.target)) return false
 * ```
 *
 * Hợp âm **duy nhất** gắn phím `Enter` trong toàn registry là `editor.confirm_segment` =
 * `Mod+Enter` (`src/commands/index.ts`). Một `Enter` **trần** không khớp `sameMods` với nó ⇒
 * vòng lặp `continue` và `handle` trả `false` **trước** dòng `isTypingZone`.
 *
 * ⇒ 🔴 **AC11 đứng, nhưng nhờ `sameMods`, KHÔNG nhờ `isTypingZone`.** Phân biệt này không phải
 * chẻ chữ: `isTypingZone` chỉ vào cuộc khi một hợp âm **đã khớp**, nên nó là hàng rào cho
 * *"một hợp âm KHÔNG MOD đã đăng ký"* — hôm nay chưa có cái nào gắn `Enter`. Ngày ai đó thêm
 * một lệnh `Enter` trần *(hoàn toàn hợp lệ)*, `isTypingZone` mới là thứ giữ AC11, và **lúc đó**
 * nó phải được đo lại. Tin nhầm rằng nó đang canh việc này hôm nay là bỏ qua đúng phép đo đó.
 * ⚠️ Cả hai mệnh đề **đo bằng vitest**, không suy: `editorTypingZone.test.ts`.
 *
 * 🔴 **IME đứng trước mọi thứ** — cùng dòng và cùng lý do `keys.ts:504`. Một lượt commit
 * composition của bộ gõ tiếng Việt phát `keydown` mang `code` vật lý; **ăn nó là ăn mất chữ**.
 * Đây cũng chính là lý do AC11 tồn tại: người dùng của sản phẩm này gõ tiếng Việt **bằng bộ
 * gõ**, nơi `Enter` là phím chốt dấu — giao `Enter` cho việc ký nghĩa là một lượt chốt Telex
 * có thể **xác nhận nhầm một câu rồi nhảy đi**.
 *
 * ⚠️ Lớp lỗi đó **không đường nghiệm thu nào của dự án bắt được** — không bộ chạy test nào mô
 * phỏng được một bộ gõ tiếng Việt thật. Nó chỉ lộ ra ở tay người dùng.
 */
function onEditKeydown(event: KeyboardEvent): void {
  // 🔴 **DÒNG NÀY KHÔNG ĐƯỢC CHẠM** — cùng dòng và cùng lý do `keys.ts:506`. Một lượt commit
  // composition của bộ gõ tiếng Việt phát `keydown` mang `code` vật lý; **ăn nó là ăn mất
  // chữ**. Nó đứng TRƯỚC mọi nhánh khác, và nó phải ở lại kể cả khi hàm này không còn nhánh
  // nào sau nó: một hàm rỗng ở đây là một chỗ mà lượt sửa kế tiếp sẽ viết vào, và người viết
  // sẽ không biết về chốt này.
  if (event.isComposing) return
  if (!composing) {
    const typed = targetCellOf(event.target instanceof Node ? event.target : null)
    if (typed !== null) {
      unwrapProofread(typed, true)
      scheduleRepaintProofread()
    }
  }
  // 🔵 2026-08-16 (Story 2.5d, AC6): lượt `preventDefault()` cho `Enter` **đã gỡ** — xem
  // doc-comment ngay trên. Vế *"`Enter` trần không ký câu nào"* nay do `keys.ts` giữ, và cụ
  // thể là do phép so **mods** (`Enter` trần không khớp `Mod+Enter`), KHÔNG do `isTypingZone`
  // — code review 2026-08-16 sửa lại chỗ gọi tên này.

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔵 STORY 2.9 · AC8 — `Esc` XOÁ TẬP ĐIỂM CẮT, và nó cần cửa NÀY chứ không chỉ registry
  // ═══════════════════════════════════════════════════════════════════════════════
  // `editor.clear_source_cuts` **đã đăng ký** với hợp âm `Escape` (`commands/index.ts`), và
  // lượt đăng ký đó không thừa: nó là thứ làm phím gán lại được (FR22) và hiện trong bảng
  // phím của Story 1.21.
  // 🔴 Nhưng nó **một mình không tới nơi**: `Escape` không mang phím bổ trợ chính, nên
  // `keys.ts:510` (`lacksPrimaryMod && isTypingZone`) **chặn** nó khi tiêu điểm nằm trong một
  // ô bản dịch — tức ở đúng chỗ người dùng đang đứng sau khi vừa gõ. Cùng cạm bẫy mà nhánh
  // `Backspace` ngay dưới đã phải đi vòng, ở một phím khác.
  // ⇒ Bắt trực tiếp rồi `dispatch` **CHÍNH** id đã đăng ký. Hai cửa, **một** command — không
  //   một đường xoá thứ hai.
  //
  // ⚠️ **KHÔNG** `preventDefault()`: `Esc` còn là phím đóng của các lớp phủ *(màn hình phím
  // tắt, Attribution, lịch sử phiên bản)*, và ăn nó ở đây là bịt đường thoát của chúng khi
  // tiêu điểm tình cờ nằm trong lưới. Một lượt xoá tập điểm cắt **không** loại trừ một lượt
  // đóng lớp phủ — hai việc ở hai tầng.
  if (event.key === 'Escape') {
    dispatch('editor.clear_source_cuts')
    return
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 2.9 · AC1 · AC6 — CỬ CHỈ `Backspace` Ở ĐẦU Ô LÀ LỆNH GỘP (FR78 · UX-DR32)
  // ═══════════════════════════════════════════════════════════════════════════════
  if (event.key !== 'Backspace') return
  // ⓐ Auto-repeat KHÔNG gộp — Ice ký ③ ngày 2026-08-17, và nó đi **ngược** bản năng của một
  //    trình soạn thảo có chủ ý. Ở một trình soạn thảo thường, giữ `Backspace` xoá liên tục
  //    là đúng. Ở đây mỗi lượt là một lượt **ghi xuống WAL** mà **AD-5 không cho hoàn tác**,
  //    và `⌘Z` (AC5) đang là món nợ chờ `AD-48`. ~30 `keydown`/giây trên một thao tác không
  //    lui được là mất ranh giới câu **hàng loạt**, bằng một phím bấm hàng trăm lần mỗi
  //    Chương. Muốn gộp tiếp: nhả phím, bấm lại.
  //    ⚠️ **GIỚI HẠN THẬT:** `event.repeat === false` ở một lượt bấm rời rạc đã đo được
  //    (`2-9-ban-do/`), nhưng vế `true` thì **KHÔNG** — WebDriver `keyDown` giữ 600 ms cho
  //    đúng một `keydown`, vì auto-repeat là hành vi tầng OS. Món cho Ice, cùng lớp với
  //    *"gõ tiếng Việt bằng bộ gõ"*.
  if (event.repeat) return
  const cell = targetCellOf(event.target instanceof Node ? event.target : null)
  if (cell === null) return
  // ⓑ Phép kiểm *"đầu ô"* sống ở `editorSegments.ts` — một mệnh đề về cây DOM, kiểm được bằng
  //    vitest. Nó **không** hỏi `startOffset === 0`: phép đó sai 2/7 hình dạng đã đo. Lý lẽ
  //    đầy đủ kèm bảng số ở doc-comment của [`caretAtCellStart`].
  if (!caretAtCellStart(cell, cell.ownerDocument.defaultView?.getSelection() ?? null)) return

  // ⓒ `preventDefault()` **khi và chỉ khi** nhánh nhận việc — nếu không, engine xoá thêm một
  //    ký tự sau lượt gộp.
  //    ⚠️ Ở đúng offset 0 của một editing host, WebKit **không có gì để xoá lui** — đo được
  //    (`2-9-ban-do/` §Ⓓ: `execCommand('delete')` trả `true` mà `textContent` không đổi một
  //    byte, trong khi đối chứng ở offset 3 xoá đúng một ký tự). Nên dòng này là **lớp phòng
  //    thứ hai**, cho ca `caretAtCellStart` trả sai chứ không cho ca thường.
  //    🔴 Và nó **không nghiệm thu được qua driver**: mọi sự kiện driver giao đều mang
  //    `isTrusted: false`, và một sự kiện không tin cậy **không có default action** — nên một
  //    phép kiểm ở đó sẽ trả *"chặn được"* trên mọi engine, kể cả engine không cho chặn.
  event.preventDefault()

  // ⓓ 🔴 **ĐI QUA `dispatch`, KHÔNG gọi thẳng `mergeCurrentSegment()`.**
  //    Story viết *"gọi chính `mergeCurrentSegment()` mà `⌘M` gọi"*, và ý đồ của câu đó là
  //    *"đừng viết một đường gộp thứ hai"*. `dispatch` thi hành ý đồ ấy **mạnh hơn** một lời
  //    gọi thẳng: nó dùng chung đường **cao hơn một bậc**, tức cả lượt xử lý kết quả và dòng
  //    chẩn đoán ở `main.ts:320-328`. Một lời gọi thẳng phải chép lại khúc đó — và
  //    `project-context.md` đã ghi bằng chữ rằng *"một lời gọi thẳng dựng một đường thứ hai
  //    mà `check:commands` KHÔNG nhìn thấy"*, cộng AD-34 §1 (*mọi thao tác đi qua một
  //    `CommandRegistry` duy nhất*).
  //    ⚠️ Đây **không** phải một command mới, và `COMMAND_FLOOR` không đổi. Cạm bẫy ② của
  //    story cấm **đăng ký một hợp âm `Backspace`** — `keys.ts:510` chặn mọi hợp âm không-mod
  //    trong vùng gõ nên nó sẽ không bao giờ bắn. Bắt trực tiếp ở đây rồi `dispatch` một id
  //    **đã đăng ký** đi vòng qua bảng hợp âm mà vẫn giữ một đường thao tác duy nhất.
  //    ⇒ Nếu `COMMAND_FLOOR` đổi vì story này, đó là dấu hiệu đã đi sai đường.
  dispatch('editor.merge_segments')
}

/** Đọc văn bản **từ chính DOM** rồi đưa vào tập chờ. Chỗ duy nhất làm việc đó. */
function reportEdit(cell: HTMLElement): void {
  const id = segmentIdOf(cell)
  if (id === null) return
  noteEditorEdit(id, cell.textContent)
  clearProofreadFor(id)
}

/**
 * Một lượt sửa đã hạ cánh trong DOM ⇒ ghi vào tập chờ.
 *
 * ⚠️ Đọc `textContent`, **không** dựng lại chuỗi từ `event.data`: một lượt xoá, một lượt undo
 * của trình duyệt (`⌘Z`) và một lượt commit IME đều không mang văn bản kết quả trong event.
 */
function onEditInput(event: Event): void {
  const cell = targetCellOf(event.target instanceof Node ? event.target : null)
  if (cell !== null) reportEdit(cell)
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC14 · Quyết định #8 — ĐƯỜNG RA MÀN HÌNH CHO LỖI, ở PHẠM VI TỐI THIỂU
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * 🔴 Chữ ký của Ice cho Quyết định #8 **là** hợp đồng UX-DR30 ở phạm vi tối thiểu: cột nhãn
 * trạng thái của **chính hàng** cộng một dòng ở thanh trạng thái. Không hộp thoại, không lớp
 * nổi (UX-DR16). **Đừng nhân lượt ký này thành một hệ thống thông báo.**
 *
 * ⚠️ Hôm nay `editorConfirmError` được export mà **không component nào đọc**, và `main.ts`
 * **vứt** `ConfirmResult` — nên một lượt từ chối **không đổi một pixel nào**. Dòng dưới đây là
 * người đọc đầu tiên.
 */
const confirmErrorParams = computed(() => editorConfirmError.value?.params ?? null)

/**
 * Đọc `segment_id` từ **`params` của chính lỗi**, không từ *"câu đang có con trỏ"*: lượt
 * xác nhận **dời con trỏ sang câu kế** khi nó thành công, và ba khoá `err.segment.*` đều mang
 * `segment_id`. Gắn lỗi vào con trỏ sẽ dán nó lên **hàng sai** ngay ở ca thường nhất.
 *
 * ⚠️ `params` là dữ liệu **đã đi qua dây**, nên nó được kiểm kiểu **lúc chạy** — Rust có thể
 * trả `null` cho `params` sau một lượt đổi lược đồ, và chỗ này là chỗ duy nhất biết.
 */
function segmentIdFromErrorParams(params: Readonly<Record<string, string>> | null | undefined): number | null {
  const raw = params?.segment_id
  if (raw === undefined) return null
  const id = Number(raw)
  return Number.isFinite(id) ? id : null
}

const errorSegmentId = computed<number | null>(() => segmentIdFromErrorParams(confirmErrorParams.value))

// Restore (FR101) and flush (AD-35) errors join confirm errors on the same row surface.
const restoreErrorParams = computed(() => historyRestoreError.value?.params ?? null)
const restoreErrorSegmentId = computed<number | null>(() => segmentIdFromErrorParams(restoreErrorParams.value))

/** Tập `segment.id` của lô flush vừa trượt — `editorFlushError` chở cả lô, không một câu. */
const flushErrorSegmentIds = computed<ReadonlySet<number>>(() => new Set(editorFlushError.value?.segmentIds ?? []))

/**
 * The `IpcError` of each errored row, rendered with `tError` so an empty `message_key` falls
 * back instead of painting a blank label. No entry ⇒ the normal state label renders.
 *
 * Thứ tự ưu tiên khi nhiều nguồn cùng trỏ vào một hàng (hiếm — ví dụ một lô flush trượt
 * ĐÚNG hàng vừa bị từ chối xác nhận): xác nhận trước — thao tác **gần nhất, người dùng vừa
 * chủ động bấm trên chính hàng này** — rồi khôi phục, rồi flush — một LÔ, gắn với một hàng
 * lỏng lẻo nhất trong ba nguồn.
 */
const rowErrorLabelById = computed<ReadonlyMap<number, IpcError>>(() => {
  const map = new Map<number, IpcError>()
  const confirmId = errorSegmentId.value
  const confirm = editorConfirmError.value
  if (confirmId !== null && confirm !== null) map.set(confirmId, confirm)
  const restoreId = restoreErrorSegmentId.value
  const restore = historyRestoreError.value
  if (restoreId !== null && restore !== null && !map.has(restoreId)) map.set(restoreId, restore)
  const flush = editorFlushError.value
  if (flush !== null) {
    for (const id of flushErrorSegmentIds.value) {
      if (!map.has(id)) map.set(id, flush.error)
    }
  }
  return map
})

const rowErrorTextById = computed(() => {
  const map = new Map<number, string>()
  for (const [id, err] of rowErrorLabelById.value) map.set(id, tError(err))
  return map
})

const unlinkedIds = computed<ReadonlySet<number>>(
  () => new Set(editorSegments.value.filter((s) => s.unlinked).map((s) => s.id)),
)

const chapterId = computed(() => editorChapterId.value)

// `@keydown` arrow handlers must also `.focus()` the newly active tab button: dispatch alone
// flips `tabindex`, not DOM focus, so a second arrow press would otherwise be a no-op.
const tabOriginalBtnRef = useTemplateRef<HTMLButtonElement>('tabOriginalBtn')
const tabHanVietBtnRef = useTemplateRef<HTMLButtonElement>('tabHanVietBtn')

/**
 * ⚠️ HAI lời gọi `dispatch('<id>')` **TĨNH**, không một tham số ternary — `check-commands.mjs`
 * Kiểm K đọc thân hàm bằng máy để đối chiếu HANDLER_TABLE (AD-34 §1: mọi `dispatch` phải đọc
 * được tĩnh), và một `dispatch(cond ? 'a' : 'b')` không phải một chuỗi literal nó đọc được.
 */
function selectTabViaArrow(target: 'original' | 'han_viet'): void {
  if (target === 'original') {
    dispatch('source.select_tab_original')
    tabOriginalBtnRef.value?.focus()
    return
  }
  dispatch('source.select_tab_han_viet')
  tabHanVietBtnRef.value?.focus()
}
const gridBlockContext: GridBlockContext = {
  restRuleById,
  selectedRowClassById,
  unlinkedIds,
  rowErrorLabelById,
  rowErrorTextById,
  firstSegmentId,
  showHanViet,
  sourceTokenClass,
  cutOffsetsOf,
  cutCountOf,
  glossaryTermsOf,
  sourcePieceInfoOf,
  trailingImagesOf,
  leadingImagesOf,
  roleLabelKeys: ROLE_LABEL_KEYS,
  setStripTrackRef,
  onSourceCellMouseUp,
  onGlossaryPieceEnter,
  onGlossaryPieceLeave,
  onColumnFocusOut,
  onCellMouseDown,
  onCellMouseUp,
  onBeforeInput,
  onCompositionStart,
  onCompositionEnd,
  onEditInput,
  onEditKeydown,
  onStripFocusOut,
}
provide(GRID_BLOCK_CONTEXT, gridBlockContext)
</script>

<template>
  <PanelFrame owner="panel.grid" status-key="panel.grid.status" :show-status="showFrameStatus">
    <!-- Lỗi nạp nói ra bằng chuỗi CỦA NÓ, không im — cùng khuôn `SourcePanel.vue`. -->
    <p v-if="loadErrorKey !== null" class="load-error">{{ t(loadErrorKey) }}</p>
    <p v-else-if="showEmptyChapter" class="load-error">{{ t('panel.grid.empty_chapter') }}</p>
    <p v-else-if="showNoSegments" class="load-error">{{ t('panel.grid.no_segments') }}</p>

    <!--
      🔴 DẢI TAB HÁN VIỆT — AC8 vế *"người dùng tự bật tắt"*, và nó phải có mặt bằng CHUỘT.

      Ba command đã tồn tại từ Story 1.16 và có phím tắt, nên vế bàn phím của AC8 vốn đã chạy.
      Nhưng dải tab này sống ở `SourcePanel.vue` — tệp bị gỡ ở story này — nên bỏ nó đi là để
      lại một tính năng **chỉ tới được bằng phím tắt người dùng chưa chắc biết. Đó là một lượt
      rút tính năng im lặng, không một lượt gọn gàng.

      ⚠️ Mỗi `@click` là ĐÚNG MỘT lời gọi `dispatch('<id>')` — `check-commands.mjs` Kiểm A
      cưỡng chế AD-34 §1. Không hàm khác, không mã nội tuyến.
      🔴 `tabindex` roving: đúng MỘT tab vào được vòng Tab, mũi tên đi giữa hai tab — một hợp
      đồng `tablist` khai một nửa còn tệ hơn không khai, vì nó hứa một mô hình tương tác không
      tồn tại (bắt ở code review 2026-08-06).
    -->
    <div v-if="hasSegments && isChinese" class="tabs" role="tablist">
      <button
        id="grid-tab-original"
        ref="tabOriginalBtn"
        type="button"
        class="tab"
        role="tab"
        aria-controls="grid-tabpanel"
        :aria-selected="activeTab === 'original'"
        :tabindex="activeTab === 'original' ? 0 : -1"
        :class="{ active: activeTab === 'original' }"
        @click="dispatch('source.select_tab_original')"
        @keydown.right.prevent="selectTabViaArrow('han_viet')"
        @keydown.left.prevent="selectTabViaArrow('han_viet')"
      >{{ t('panel.source.tab_original') }}</button>
      <button
        id="grid-tab-han-viet"
        ref="tabHanVietBtn"
        type="button"
        class="tab"
        role="tab"
        aria-controls="grid-tabpanel"
        :aria-selected="activeTab === 'han_viet'"
        :tabindex="activeTab === 'han_viet' ? 0 : -1"
        :class="{ active: activeTab === 'han_viet' }"
        @click="dispatch('source.select_tab_han_viet')"
        @keydown.right.prevent="selectTabViaArrow('original')"
        @keydown.left.prevent="selectTabViaArrow('original')"
      >{{ t('panel.source.tab_han_viet') }}</button>
      <!-- aura-allow-text: cả hai nhánh đi qua t(), không chuỗi viết thẳng nào — Kiểm A2
           không đọc tĩnh được toán tử ba ngôi (cùng khuôn `LibraryMode.vue:143`). -->
      <button
        v-if="activeTab === 'han_viet' && (viewMode === 'parallel' || canUseParallelView)"
        type="button"
        class="view-toggle"
        @click="dispatch('source.toggle_han_viet_view')"
      >{{ viewMode === 'switch' ? t('panel.source.view_mode_parallel') : t('panel.source.view_mode_switch') }}</button>
    </div>

    <div v-if="hasSegments" id="grid-tabpanel" class="grid-scroll" :data-chapter-id="chapterId">
      <!--
        🔴 LƯỚI CHỦ-CỘT — Quyết định #1(b). Cha khai **tập track hàng**; năm con là năm CỘT.
        `grid-template-rows` đặt qua `:style` vì số hàng là dữ liệu, không phải hằng CSS.

        ⚠️ Đây là chỗ DUY NHẤT của tệp này bind một giá trị hình học qua `:style`. Màu thì
        **không bao giờ** — Kiểm B của `check-tokens.mjs` đọc CSS, không đọc TypeScript, nên
        một màu bind qua `:style` đi qua cổng mà không một dòng nào được soi.
      -->
      <div ref="gridRoot" class="grid" :data-sel-src="sourceSelecting ? '' : null" @copy="onGridCopy">
        <GridBlock
          v-for="block in gridBlockViews"
          :key="block.index"
          :block="block"
          :caret-id="caretIdIn(block)"
          :anchor-id="anchorIdIn(block)"
          :strip-row="stripRowIn(block)"
          :has-strip-track="block.index === stripBlockIndex"
        />
      </div>
    </div>
  </PanelFrame>
</template>

<style scoped>
.grid[data-sel-src] :deep(.col:not(.col-src)::selection),
.grid[data-sel-src] :deep(.col:not(.col-src) *::selection) {
  background: transparent;
}

.load-error {
  margin: 0;
  /*
   * 🔵 `flex: none` thêm 2026-08-17, cùng lượt `.panel-body` thành flex column. Mặc định
   * `flex-shrink: 1` cho phép **co** một dải chữ khi chỗ hẹp — và một câu lỗi bị co là một câu
   * lỗi người dùng đọc không ra, tức đúng lớp *"rỗng IM LẶNG"* mà chuỗi này tồn tại để chống.
   * Luật chung ở `PanelFrame.vue::.panel-body` §GIỚI HẠN THẬT: mọi con không-cuộn khai `flex: none`.
   */
  flex: none;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

/*
 * Dải tab — chép hình dạng và token từ `SourcePanel.vue` (Story 1.16, Quyết định #7).
 */
.tabs {
  display: flex;
  /*
   * 🔵 SỬA 2026-08-18 — `--space-inline-sm` ⇒ `--space-panel-inline`. HỒI QUY của Story 2.5b,
   * Ice tìm ra bằng mắt: hai tab *"Trung"* và *"Hán Việt"* **dính vào nhau**.
   *
   * 🔴 **`--space-inline-sm` CHƯA BAO GIỜ TỒN TẠI.** Khối `spacing` của `tokens.json` có đúng
   * chín khoá và không có `inline-sm`; `applyTheme` phát `--space-<tên>` từ **chính** khối đó
   * (`tokens/index.ts:106-107`), nên biến này không được đặt ở đâu. Một `var()` không xác định
   * làm **cả khai báo** `gap` không hợp lệ ⇒ `gap` về `normal` = **0**.
   *
   * ⚠️ **Vì sao giá trị đúng là `panel-inline`, và đó là một phép đo lịch sử chứ không một lựa
   * chọn của tôi:** `git show ca33072^:src/panels/SourcePanel.vue` cho `gap: var(--space-panel-
   * inline)` ở đúng dải tab này — **16px, chạy được** từ Story 1.16. Commit `ca33072` (2.5b) chép
   * dải tab sang đây và **đổi** token trong lượt chép. ⇒ Đây là hoàn nguyên về giá trị đã có chữ
   * ký, **không** một lượt xin một khoảng cách mới. Chú thích ngay dưới nói *"chép hình dạng và
   * token từ `SourcePanel.vue`"* — nó đúng về hình dạng và **sai về token**, đúng ở dòng này.
   *
   * 🔴 **Đường sai rẻ ở đây là thêm `"inline-sm": "8px"` vào `tokens.json`** cho hết lỗi. Đó là
   * dựng một giá trị thiết kế **chưa ai chọn**, và nó làm dải tab khác 16px mà không ai quyết —
   * trong khi số 16 thì có nguồn. *"Sửa nguồn cho nó nói thật"*, không nhét một token cho vừa mã.
   *
   * ⚠️ **KHÔNG CỔNG NÀO CANH LỚP LỖI NÀY** và nó đi qua trọn mười một cổng suốt ba story:
   * `check:tokens` Kiểm B đọc CSS để bắt **màu viết thẳng**, nó không đối chiếu tên `var()` với
   * bảng token. Một tham chiếu token không tồn tại là **CSS chết im lặng**. Đã ghi nợ có chủ.
   * *(Đo 2026-08-18: đối chiếu chín khoá `spacing` với mọi `--space-*` trong `src/**` ⇒ `inline-sm`
   * là ca DUY NHẤT của toàn cây. Bốn khoá khai mà không dùng: `gutter-width` · ba `read-measure-*`.)*
   */
  gap: var(--space-panel-inline);
  align-items: baseline;
  flex: none;
  margin-bottom: var(--space-panel-block);
}

.tab,
.view-toggle {
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.tab.active {
  color: var(--color-on-surface);
}

.view-toggle {
  margin-left: auto;
}

/*
 * 🔴 HỘP CUỘN LÀ **CHỖ NÀY**, không phải một cột.
 *
 * Năm cột phải cuộn **cùng nhau** — chúng chia chung một tập track hàng, và cho một cột cuộn
 * riêng là phá đúng thứ `subgrid` vừa mua. Cùng lý do `EditorPanel.vue::.edbody` đặt hộp cuộn
 * ở khung bao chứ không ở trang văn.
 */
/*
 * 🔴 HỘP CUỘN DUY NHẤT của lưới — năm cột `subgrid` phải cuộn CÙNG nhau.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 SỬA 2026-08-17 — `height: 100%` ⇒ `flex: 1`. Ice tìm ra bằng mắt (Task 4.5)
 * ─────────────────────────────────────────────────────────────────────────────
 * `height: 100%` ở đây **sai bất kể `.panel-body` là flex hay không**, vì hộp này **không** phải
 * con duy nhất của slot: dải tab Hán Việt (`.tabs`) đứng trước nó khi Chương là `zh`. Xin 100%
 * chiều cao trong khi đã bị đẩy xuống **30px** *(tab 18px + `--space-panel-block` 12px)* làm nội
 * dung tràn `.panel`, và `.panel` mang `overflow: hidden` ⇒ **30px đáy bị cắt vĩnh viễn**.
 *
 * ⚠️ Triệu chứng người dùng thấy, và nó không phải 30px trang trí: khi đã cuộn tới đáy, đáy hàng
 * **cuối** trùng đúng vùng bị cắt ⇒ **dòng cuối của câu nguồn cuối Chương mất chữ**, và không cử
 * chỉ nào lấy lại được. Xảy ra ở **mọi** Chương tiếng Trung — tức ca thường nhất của sản phẩm.
 * `focus()` làm đúng phần việc của nó; lỗi nằm ở hộp, không ở lượt cuộn.
 *
 * 🔴 `flex: 1` **cộng** `min-height: 0` — thiếu vế thứ hai thì một hộp cuộn trong flex column
 * lấy `min-height: auto` theo nội dung và **không co lại**, tức nó tràn y như cũ chỉ bằng một
 * đường khác. Hai dòng là một chốt, không hai lượt gõ.
 *
 * ⚠️ Chốt này dựa vào `.panel-body` khai `display: flex; flex-direction: column` — xem khối lý do
 * ở `PanelFrame.vue::.panel-body`, nơi nguyên nhân được đóng cho **cả ba** panel. Đổi một trong
 * hai chỗ mà không đổi chỗ kia thì lưới **không cuộn được** *(mất `flex` container)* hoặc trở lại
 * cắt chữ *(mất `flex: 1`)*.
 */
.grid-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

/*
 * AC2 — BỐN CỘT. Bề rộng hai cột chữ chia đều; hai cột phụ lấy đúng chỗ chúng cần.
 *
 * ⚠️ `minmax(0, 1fr)` chứ không `1fr`: mặc định `min-width` của một grid item là `auto`, nên
 * một ô nguyên văn dài **không xuống dòng** mà đẩy cả lưới rộng ra. Đây là bẫy thường gặp
 * nhất của CSS Grid, và nó chỉ lộ ra ở Chương có câu dài.
 */
.grid {
  --grid-columns: 3px 30px minmax(0, 1fr) minmax(0, 1fr);
}
</style>
