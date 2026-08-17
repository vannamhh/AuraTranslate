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
import { computed, onBeforeUnmount, onMounted, useTemplateRef, watch } from 'vue'
import PanelFrame from './PanelFrame.vue'
import SourceHanViet from './SourceHanViet.vue'
import { useSelectionSurface } from './selectionContract'
import { resolveHanVietSelection } from './hanVietSurfaces'
import type { DockviewPanelProps } from '../layout/panelProps'
import { t } from '../i18n'
import { detectIsMac, dispatch } from '../commands'
import {
  caretAtCellStart,
  hasPrimaryModifier,
  resolveSegmentRule,
  ruleClassOf,
  segmentRuleInputOf,
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
  editorChapterId,
  editorConfirmError,
  editorEditedText,
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
const ruleById = computed(() => {
  const caret = editorCaretSegmentId.value
  const edited = editorEditedText.value
  const map = new Map<number, SegmentRuleValue>()
  for (const s of editorSegments.value) {
    const input = segmentRuleInputOf(s, caret)
    map.set(s.id, resolveSegmentRule({ ...input, targetText: edited.get(s.id) ?? input.targetText }))
  }
  return map
})

const ruleClassById = computed(() => {
  const map = new Map<number, string>()
  for (const [id, rule] of ruleById.value) {
    const cls = ruleClassOf(rule)
    if (cls !== null) map.set(id, cls)
  }
  return map
})

/**
 * AC4 + Quyết định #8 — **cột nhãn trạng thái**, sáu nhãn, khoá phẳng có tiền tố miền.
 *
 * 🔴 Đây là cột thứ năm, và nó là **kênh đọc được** cho đúng thứ vạch lề nói bằng màu. Vạch
 * là một lớp thông tin **thị giác**; một người dùng đọc bằng bàn phím hoặc bằng trình đọc màn
 * hình không có nó. Cột này là lý do vạch được phép `aria-hidden`.
 */
const STATE_LABEL_KEYS: Readonly<Record<SegmentRuleValue, string>> = {
  confirmed: 'panel.grid.state_confirmed',
  primary: 'panel.grid.state_editing',
  draft: 'panel.grid.state_draft',
  'tm-rule': 'panel.grid.state_tm',
  none: 'panel.grid.state_untranslated',
  ornament: 'panel.grid.state_retired',
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
 * 🔴 **Chỉ số ký tự nguồn nơi mỗi mảnh của [`sourcePiecesOf`] bắt đầu** — Story 2.9, AC9.
 *
 * Cùng biên, cùng thứ tự, cùng đơn vị *(điểm mã)* với hàm kia. Nó tồn tại để mỗi mảnh mang
 * được một **NEO** `data-src-start` trong DOM, thứ mà `sourceCutOffsetOf` đọc thay cho phép
 * đếm mù mọi text node.
 *
 * ⚠️ Hai hàm phải giữ **cùng một** phép chia. Tách chúng ra là mời một lượt lệch im lặng —
 * đây là chỗ duy nhất trong story chấp nhận một bản sao, và nó chấp nhận được vì cả hai đọc
 * chung `pendingCuts` và cùng lọc/sắp y hệt.
 */
function sourcePieceStartsOf(segmentId: number, text: string): readonly number[] {
  const n = cutCountOf(segmentId)
  if (n === 0) return [0]
  const ky = [...text]
  const moc = [...(pendingCuts.value?.offsets ?? [])]
    .filter((o) => o > 0 && o < ky.length)
    .sort((a, b) => a - b)
  return [0, ...moc]
}

function sourcePiecesOf(segmentId: number, text: string): readonly string[] {
  const n = cutCountOf(segmentId)
  if (n === 0) return [text]
  const ky = [...text]
  const moc = [...(pendingCuts.value?.offsets ?? [])]
    .filter((o) => o > 0 && o < ky.length)
    .sort((a, b) => a - b)
  const bien = [0, ...moc, ky.length]
  const ra: string[] = []
  for (let i = 0; i < bien.length - 1; i += 1) {
    ra.push(ky.slice(bien[i], bien[i + 1]).join(''))
  }
  return ra
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC7 · B1 — HỢP ĐỒNG VÙNG CHỌN THEO **CỘT**, hai lời gọi, hai vai KHÔNG được đảo
// ═════════════════════════════════════════════════════════════════════════════════
const colSrc = useTemplateRef<HTMLElement>('colSrc')
const colTgt = useTemplateRef<HTMLElement>('colTgt')

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
  return mapped === undefined ? selection.toString() : mapped
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

useSelectionSurface(colSrc, 'source', resolveSourceSelection)
useSelectionSurface(colTgt, 'display')

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
  // phát hành — `useSelectionSurface(colSrc, 'source', …)` ngay trên)*.
  // ⇒ Mỗi lượt tra một từ để **đọc** cũng rơi một dấu cắt; và một cú **double-click** bắn
  //   **HAI** `mouseup` nên nó để lại **hai** dấu. Người dùng đặt chỗ cắt cho một lượt `⌘/`
  //   họ không định gọi, và AD-5 không cho hoàn tác lượt tách đó.
  //
  // ✅ Ice ký 2026-08-17 — một lượt lật tìm ra bằng cách **DÙNG THẬT**, đúng khuôn đã lặp ở
  //    2.5b *(hàng về hưu trong lưới)* và 2.8 *(chữ ký #6 lật lần thứ ba)*. Cùng lượt ký đó
  //    xác nhận **double-click TRA ĐƯỢC** trên máy thật ⇒ đóng món nợ 🔴 `deferred-work.md:4100`.
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
  if (next instanceof Node && colTgt.value?.contains(next)) return
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
 *
 * 🔵 **RẺ HƠN `EditorPanel.vue` một bậc, và đó là một hệ quả đo được của lưới:** bản cũ quét
 * **toàn bộ** `.doc` bằng `querySelectorAll` mỗi lượt *(`deferred-work.md:2484-2489`, chủ
 * Story 2.4)*. Ở đây phép quét giới hạn trong **cột bản dịch**, và nó chỉ chạy khi tập chờ
 * **khác rỗng**.
 */
function restoreEditedText(): void {
  const host = colTgt.value
  if (host === null) return
  const edited = editorEditedText.value
  if (edited.size === 0) return

  for (const el of host.querySelectorAll<HTMLElement>('[data-segment-id]')) {
    const id = segmentIdOf(el)
    if (id === null) continue
    const text = edited.get(id)
    if (text === undefined) continue
    if (document.activeElement === el) continue
    if (el.textContent !== text) el.textContent = text
  }
}

watch([() => editorSegments.value, colTgt], restoreEditedText, { flush: 'post' })

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
  const host = colTgt.value
  if (host === null) return
  const anchor = window.getSelection()?.anchorNode ?? null
  if (anchor === null || !host.contains(anchor)) return
  const id = segmentIdOf(targetCellOf(anchor))
  if (id !== null) setEditorCaret(id)
}

onMounted(() => {
  document.addEventListener('selectionchange', onSelectionChange)
})
onBeforeUnmount(() => {
  document.removeEventListener('selectionchange', onSelectionChange)
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
    const host = colTgt.value
    if (host === null) return
    const target = host.querySelector<HTMLElement>(`[data-segment-id="${id}"]`)
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
// Nửa đầu do `ruleById` lo; nửa sau do `focus()` lo.
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
  if (flat === '') return

  const selection = window.getSelection()
  // ⚠️ Chẩn đoán viết bằng tiếng Anh, KHÔNG tiếng Việt: Kiểm A của `check-i18n.mjs` cấm chuỗi
  // tiếng Việt ở vị trí mã trong `.vue` và `.rs`.
  if (selection === null || selection.rangeCount === 0) {
    console.warn('[grid] insert skipped: no readable selection')
    return
  }
  const range = selection.getRangeAt(0)
  if (!cell.contains(range.startContainer)) {
    console.warn(
      '[grid] insert skipped: selection starts outside the edited cell ' +
        `(inputType=${event.inputType}) — reselect within one cell and paste again`,
    )
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
const confirmErrorKey = computed(() => editorConfirmError.value?.message_key ?? null)
const confirmErrorParams = computed(() => editorConfirmError.value?.params ?? null)

/**
 * Hàng nào mang lỗi. `null` ⇒ không hàng nào.
 *
 * 🔴 Đọc `segment_id` từ **`params` của chính lỗi**, không từ *"câu đang có con trỏ"*: lượt
 * xác nhận **dời con trỏ sang câu kế** khi nó thành công, và ba khoá `err.segment.*` đều mang
 * `segment_id`. Gắn lỗi vào con trỏ sẽ dán nó lên **hàng sai** ngay ở ca thường nhất.
 *
 * ⚠️ `params` là dữ liệu **đã đi qua dây**, nên nó được kiểm kiểu **lúc chạy** — Rust có thể
 * trả `null` cho `params` sau một lượt đổi lược đồ, và chỗ này là chỗ duy nhất biết.
 */
const errorSegmentId = computed<number | null>(() => {
  const raw = confirmErrorParams.value?.segment_id
  if (raw === undefined) return null
  const id = Number(raw)
  return Number.isFinite(id) ? id : null
})

const chapterId = computed(() => editorChapterId.value)
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
        type="button"
        class="tab"
        role="tab"
        aria-controls="grid-tabpanel"
        :aria-selected="activeTab === 'original'"
        :tabindex="activeTab === 'original' ? 0 : -1"
        :class="{ active: activeTab === 'original' }"
        @click="dispatch('source.select_tab_original')"
        @keydown.right.prevent="dispatch('source.select_tab_han_viet')"
        @keydown.left.prevent="dispatch('source.select_tab_han_viet')"
      >{{ t('panel.source.tab_original') }}</button>
      <button
        id="grid-tab-han-viet"
        type="button"
        class="tab"
        role="tab"
        aria-controls="grid-tabpanel"
        :aria-selected="activeTab === 'han_viet'"
        :tabindex="activeTab === 'han_viet' ? 0 : -1"
        :class="{ active: activeTab === 'han_viet' }"
        @click="dispatch('source.select_tab_han_viet')"
        @keydown.right.prevent="dispatch('source.select_tab_original')"
        @keydown.left.prevent="dispatch('source.select_tab_original')"
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
      <div class="grid" :style="{ gridTemplateRows: `repeat(${editorSegments.length}, auto)` }">
        <!--
          Cột ① — VẠCH TRẠNG THÁI (UX-DR19, sáu giá trị).

          `aria-hidden`: vạch là một lớp thông tin **thị giác** trùng lặp với thứ cột ⑤ nói
          bằng chữ. Một chuỗi div rỗng không nhãn ở đây chỉ là tiếng ồn cho công nghệ trợ giúp.
        -->
        <div class="col col-rule" aria-hidden="true">
          <div v-for="s in editorSegments" :key="s.id" class="cell cell-rule" :class="{ 'para-end': s.is_paragraph_end }">
            <div v-if="ruleClassById.get(s.id)" class="rule" :class="ruleClassById.get(s.id)"></div>
          </div>
        </div>

        <!--
          Cột ② — SỐ CÂU. `on-surface-variant`, KHÔNG `ornament` (Quyết định #9(a)): đây là
          **chữ thật**, không phải nét, và `ornament` đo được 2,44 (sáng) / 2,64 (tối) trên
          `surface` — trượt AA.
          aura-allow-text: số thứ tự hàng là DỮ LIỆU, không một chuỗi giao diện.
        -->
        <div class="col col-num" aria-hidden="true">
          <div
            v-for="(s, i) in editorSegments"
            :key="s.id"
            class="cell cell-num"
            :class="{ 'para-end': s.is_paragraph_end, omitted: s.is_omitted }"
          ><!-- aura-allow-text: số thứ tự hàng — một con số dựng tại chỗ hiển thị, đúng luật "định dạng số và ngày giờ CHỈ ở frontend" (§Consistency Conventions), không một chuỗi giao diện dịch được. -->{{ i + 1 }}</div>
        </div>

        <!-- Cột ③ — NGUYÊN VĂN. Một bề mặt vùng chọn duy nhất, vai `'source'` (AC7). -->
        <div ref="colSrc" class="col col-src" :class="sourceTokenClass">
          <div
            v-for="s in editorSegments"
            :key="s.id"
            class="cell cell-src"
            :class="{
              'para-end': s.is_paragraph_end,
              omitted: s.is_omitted,
            }"
            :data-segment-id="s.id"
            :data-cut-count="cutCountOf(s.id)"
            data-col="src"
            @mouseup="onSourceCellMouseUp"
          >
            <!--
              AC8 — Hán Việt sống TRONG ô nguyên văn, hai chế độ FR19, người dùng tự bật tắt.
              `surface-role="cell"` nhượng lượt đăng ký cho CỘT — xem `hanVietSurfaces.ts`.
            -->
            <SourceHanViet
              v-if="showHanViet"
              :source-text="s.source_text"
              :view-mode="viewMode"
              :cuts="cutOffsetsOf(s.id)"
              surface-role="cell"
            />
            <!--
              🔵 2026-08-17, AC7 — nguyên văn chia thành mảnh theo TẬP ĐIỂM CẮT đang chờ, mỗi
              ranh giới một dấu. Không điểm nào ⇒ `sourcePiecesOf` trả đúng MỘT mảnh, tức
              đường thường giữ nguyên hình dạng cây.

              ⚠️ Dấu cắt RỖNG và mang `aria-hidden` — nó không phải chữ, và một ký tự thật ở
              đây sẽ đi vào `Selection.toString()` của Auto-Lookup rồi vào mọi lượt sao chép.
            -->
            <template v-else
              ><template v-for="(manh, i) in sourcePiecesOf(s.id, s.source_text)" :key="i"
                ><span v-if="i > 0" class="cut-mark" aria-hidden="true"></span
                ><!-- 🔵 2026-08-17 (AC9) — mỗi mảnh nay là một `<span>` mang NEO
                     `data-src-start`. Trước lượt này mảnh là text node trần và
                     `sourceCutOffsetOf` phải **đếm mù** mọi text node trong ô — phép đếm đó
                     sai hẳn ở tab Hán Việt *(đo: 17 và 19 trên một câu 5 chữ)*.
                     ⚠️ `<span>` KHÔNG thêm một ký tự nào vào `Selection.toString()`, nên
                     Auto-Lookup (FR21) không bị chạm.
                     aura-allow-text: nguyên văn của Tác phẩm — DỮ LIỆU, không chuỗi giao
                     diện (NFR16). Không `v-html` (AD-16). --><span
                  class="src-piece"
                  :data-src-start="sourcePieceStartsOf(s.id, s.source_text)[i]"
                >{{ manh }}</span></template
              ></template
            >
          </div>
        </div>

        <!--
          Cột ④ — BẢN DỊCH. Một bề mặt vùng chọn duy nhất, vai `'display'` (B1).

          🔴 Ba handler sống ở **CỘT**, không ở từng ô: sự kiện nổi bọt, và N listener cho
          9.850 hàng là N lượt đăng ký mà một lượt là đủ.
        -->
        <div
          ref="colTgt"
          class="col col-tgt tok-editor"
          @focusout="onColumnFocusOut"
          @mousedown="onCellMouseDown"
          @mouseup="onCellMouseUp"
          @beforeinput="onBeforeInput"
          @input="onEditInput"
          @keydown="onEditKeydown"
        >
          <!--
            aura-allow-text: BẢN DỊCH của người dùng — dữ liệu, không chuỗi giao diện.

            🔴 `contenteditable` trên **MỌI** ô (Quyết định #3(b)). Không một binding động nào:
            giá trị không đổi theo trạng thái, nên Vue không bao giờ vá thuộc tính này và engine
            không bao giờ dựng lại editing host.

            ⚠️ Văn bản render `s.target_text` — bản **LÚC NẠP**. Văn bản đang gõ đi đường
            [`restoreEditedText`], KHÔNG qua một binding phản ứng. Xem khối *"DOM SỞ HỮU VĂN
            BẢN"* ở script.

            🔴 `tgt-para-end` — cờ kết đoạn của **BẢN DỊCH** (Story 2.5d, FR134/AD-46). Nó đọc
            `s.is_target_paragraph_end`, tức **dữ liệu đã lưu** (AC4) — **không** suy từ
            `is_paragraph_end` của cột nguyên văn, và **không** suy từ vị trí các `\n` trong
            `target_text`. Hai phép suy đó đều chạy ra kết quả trông đúng, và cả hai rẽ khỏi
            đĩa đúng vào ngày người dùng đổi cờ đầu tiên.
            ⚠️ Class đặt trên **từng ô**, vì một hàng KHÔNG phải một phần tử DOM — xem khối
            đầu tệp. Cái giá hình học của lựa chọn này đã đo, ghi ở khối style cuối tệp.

            🔵 CẬP NHẬT 2026-08-16 (code review) — `empty` đo bằng `.trim()`, KHÔNG `=== ''`.
            Trước lượt vá này lưới hỏi `=== ''` còn Rust hỏi `target_text.trim().is_empty()`
            (`commands/segment.rs`, đường `confirm_segment`). Hai định nghĩa "rỗng" chỉ trùng
            nhau chừng nào ô KHÔNG chứa được khoảng trắng — và AC1 của Story 2.5d vừa cho nó
            chứa `\n`. Hệ quả đo được: bấm `Enter` trong một ô rỗng cho `"\n"`, ô MẤT viền đứt
            nên trông đã dịch, mà `confirm_segment` vẫn từ chối ký — người dùng không có cách
            nào biết vì sao. 🔴 Rust là nguồn sự thật cho "rỗng"; lưới đi theo, không ngược lại.
          -->
          <div
            v-for="s in editorSegments"
            :key="s.id"
            class="cell cell-tgt"
            :class="{
              'para-end': s.is_paragraph_end,
              empty: (editorEditedText.get(s.id) ?? s.target_text).trim() === '',
              editing: editorCaretSegmentId === s.id,
              omitted: s.is_omitted,
              'tgt-para-end': s.is_target_paragraph_end,
            }"
            :data-segment-id="s.id"
            data-col="tgt"
            contenteditable="true"
          >{{ s.target_text }}</div>
        </div>

        <!-- Cột ⑤ — NHÃN TRẠNG THÁI, và là bề mặt báo lỗi của AC14 (Quyết định #8(a)). -->
        <div class="col col-state">
          <div
            v-for="s in editorSegments"
            :key="s.id"
            class="cell cell-state"
            :class="{
              'para-end': s.is_paragraph_end,
              refused: errorSegmentId === s.id,
              omitted: s.is_omitted,
            }"
          >
            <template v-if="errorSegmentId === s.id && confirmErrorKey !== null">{{
              t('panel.grid.state_refused')
            }}</template>
            <template v-else>{{ t(STATE_LABEL_KEYS[ruleById.get(s.id) ?? 'none']) }}</template>
          </div>
        </div>
      </div>
    </div>
  </PanelFrame>
</template>

<style scoped>
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
 * AC2 — NĂM CỘT. Bề rộng hai cột chữ chia đều; ba cột phụ lấy đúng chỗ chúng cần.
 *
 * ⚠️ `minmax(0, 1fr)` chứ không `1fr`: mặc định `min-width` của một grid item là `auto`, nên
 * một ô nguyên văn dài **không xuống dòng** mà đẩy cả lưới rộng ra. Đây là bẫy thường gặp
 * nhất của CSS Grid, và nó chỉ lộ ra ở Chương có câu dài.
 */
.grid {
  display: grid;
  grid-template-columns: 3px 30px minmax(0, 1fr) minmax(0, 1fr) 96px;
  align-items: stretch;
}

/*
 * 🔴 MỖI CỘT LÀ MỘT SUBGRID CHIẾM TRỌN TẬP TRACK HÀNG CỦA CHA (Quyết định #1(b)).
 *
 * `grid-row: 1 / -1` + `grid-template-rows: subgrid` là **cặp**, không phải hai dòng rời:
 * thiếu vế đầu thì cột chỉ chiếm một track, thiếu vế sau thì nó tự chia track riêng và hàng
 * lệch tích luỹ. Đo 2026-08-14: lệch `top` = **0 px** trên cả hai engine.
 *
 * ⚠️ Sàn: Safari 16+ *(macOS 12.4+)* · Chrome/Edge 117+. Ba engine bất đồng ở **gap** và
 * **auto-sizing** — lưới này cố ý **không dùng `gap`**, khoảng cách đi qua `padding` của ô.
 */
.col {
  display: grid;
  grid-template-rows: subgrid;
  grid-row: 1 / -1;
  min-width: 0;
}

/*
 * 🔴 KIỂU DÁNG CẤP HÀNG PHẢI NHÂN RA NĂM Ô — hàng KHÔNG còn là một hộp.
 *
 * Đường kẻ dưới hàng nằm ở đây, trên **từng ô**, chứ không trên một `<tr>`. Bản dựng
 * `.working/editor-grid-two-column.html:208-229` viết chúng trên `<tr>`; **không chép thẳng
 * được**. Cùng luật cho `.para-end` bên dưới.
 */
.cell {
  padding: 2px 8px;
  border-bottom: 1px solid var(--color-outline-faint);
  min-width: 0;
}

/*
 * AC5 — KHOẢNG THỞ giữa các nhóm hàng, **không** một hàng rỗng.
 *
 * 🔴 Chỉ **ĐỌC** cờ `is_paragraph_end` đã lưu (AD-37, B3). Một dòng `text.split('\n')` lúc
 * render đi qua **mọi cổng** và làm hỏng FR121 ở Epic 8 — không cổng nào đỏ vì chuyện đó.
 * 🔴 **CẢNH BÁO NÀY NẶNG THÊM TỪ 2026-08-16 (Story 2.5d).** Trước đó `target_text` **không
 * bao giờ** chứa `\n`, nên một `split('\n')` lúc render là một đường sai *lý thuyết*. Nay
 * `\n` là một ký tự **hợp lệ và thường gặp** trong đó (AC1, FR134) — tức lượt suy ấy sẽ
 * **chạy ra kết quả**, và kết quả sẽ trông đúng cho tới ngày người dùng đổi cờ đích.
 * ⇒ Cấu trúc đoạn của **bản dịch** đọc từ `is_target_paragraph_end` (bước di trú 9, AC4),
 * **không** từ vị trí các `\n`. Hai khái niệm khác nhau: `\n` là xuống dòng **trong** một
 * câu, cờ là ranh giới đoạn **sau** câu.
 * ⚠️ Rà 2026-08-16 (Task 7.4): `split('\n')` · `split("\n")` · `lines()` trên `src/**` và
 * `src-tauri/src/**` ⇒ **0** đường suy. Con số ghi ra thay vì để im lặng đọc thành "đã rà".
 *
 * ⚠️ Một hàng rỗng bị loại có lý do đo được, không vì gu: nó sẽ là một hàng **không có
 * `segment.id`**, tức một hàng mà mọi phép đếm `[data-segment-id]` phải học cách bỏ qua — và
 * bộ e2e đếm đúng thứ đó. Một `padding-bottom` không sinh phần tử nào.
 */
.cell.para-end {
  padding-bottom: 14px;
  border-bottom-color: var(--color-outline);
}

/* Cột ① — vạch. `padding: 0` để vạch cao đúng bằng track hàng. */
.cell-rule {
  padding: 2px 0;
  border-bottom: none;
}

/*
 * 🔴 Vạch lấy chiều cao từ **track hàng**, không từ `getClientRects()`.
 *
 * Đây là chỗ `editorGutter.ts` *(273 dòng · 31 chỗ nhắc "làn")* mất lý do tồn tại: bài toán
 * *"nhiều câu trên cùng một dòng ⇒ vạch chồng nhau"* biến mất **theo cấu trúc** khi một câu
 * là một hàng. Phép đo của nó *(O(n²) 482,4/254,5/261,6 ms → quét đường 8,3/5,2/4,3 ms trên
 * 9.850 vạch)* sống tiếp trong `deferred-work.md` — mã bị gỡ, bằng chứng thì không.
 */
.rule {
  height: 100%;
  border-radius: var(--radius-sm);
}

/*
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 NĂM MÀU VẠCH — MỖI GIÁ TRỊ MỘT KHỐI, VÀ ĐÓ LÀ ĐIỀU KIỆN ĐỂ CỔNG NHÌN THẤY GÌ
 * ═══════════════════════════════════════════════════════════════════════════════
 * Kiểm B của `check-tokens.mjs` đọc CSS, không đọc TypeScript. Bind màu qua `:style` từ một
 * hàm TS sẽ đi qua cổng mà không một dòng nào được soi. `check-commands.mjs` Kiểm I đối chiếu
 * năm khối này **hai chiều** với `SEGMENT_RULE_VALUES`.
 *
 * ⚠️ Giá trị thứ sáu — *không vạch* — **không có khối CSS**, vì nó không vẽ gì: `ruleClassOf`
 * trả `null` và `v-if` không dựng phần tử nào. Năm khối + một sự vắng mặt có chủ = sáu.
 *
 * ⚠️ Hai trong năm màu hôm nay **không đường nào tới được** trên dữ liệu thật — `tm-rule`
 * thuộc Epic 7 (`isTmFilled` còn là hằng `false`) và `ornament` thuộc Story 2.8.
 */
.rule.rule-confirmed {
  background-color: var(--color-confirmed);
}

.rule.rule-primary {
  background-color: var(--color-primary);
}

.rule.rule-tm-rule {
  background-color: var(--color-tm-rule);
}

/*
 * 🔵 Token thứ **17**, thêm ở Story 2.5b (Quyết định #2(a), Ice ký 2026-08-14).
 *
 * ⚠️ `draft` mượn **đúng giá trị** của `ornament` ở cả hai theme, nên nó là một **cái tên mới
 * cho một màu đã kiểm** — không một màu mới chưa ai đo, và **0 cặp mới** cho `contrast.pairs`.
 * 🔴 Trùng giá trị **không** phải trùng nghĩa: `ornament` nói *đã về hưu*, `draft` nói *đã dịch
 * tay, chưa ai ký*. Hai vạch không bao giờ cùng xuất hiện — `ornament` thắng tất cả trong
 * thứ tự nhánh của `resolveSegmentRule`.
 */
.rule.rule-draft {
  background-color: var(--color-draft);
}

.rule.rule-ornament {
  background-color: var(--color-ornament);
}

/*
 * 🔴 DẤU ĐIỂM CẮT ĐANG CHỜ — Story 2.8, AC7, kênh thị giác của cơ chế tích luỹ.
 *
 * ⚠️ `inline-block` + `width` chứ không một ký tự: một ký tự thật *(`|`, `‸`)* đi vào
 * `Selection.toString()` của Auto-Lookup và vào mọi lượt sao chép của người dùng. Dấu này là
 * **giao diện**, không phải nguyên văn của Tác phẩm.
 *
 * ⚠️ Màu từ token `--color-ornament`, cùng token với vạch hàng về hưu — hai thứ nói cùng một
 * điều: *"đây là một chỗ đánh dấu của công cụ, không phải nội dung"*. `check:tokens` Kiểm B
 * cấm mọi màu viết thẳng, và Kiểm F cấm bóng đổ / gradient / lớp nổi — dấu này không dùng cái
 * nào trong ba.
 *
 * ⚠️ `vertical-align` neo theo chiều cao dòng chứ không theo baseline: cột nguyên văn dựng
 * bằng ba họ font khác nhau *(`read-cjk` và `source-latin`)* và baseline của chúng lệch nhau.
 */
/*
 * 🔵 **2026-08-17 (Ice yêu cầu) — CAO HƠN và ĐỔI MÀU: `ornament` → `primary`.**
 *
 * `ornament` là `#a9a196` (sáng) / `#6a6459` (tối) — một màu **nét trang trí**, cố ý mờ. Trên
 * nền `surface` nó đo được **2,44 / 2,64** *(đã ghi ở Quyết định #9(a) của 2.5b)*, tức mờ đến
 * mức `check:tokens` **cấm** nó làm màu chữ. Một dấu cắt là thứ người dùng phải **tìm thấy**,
 * không một đường viền nền.
 *
 * `primary` (`#2f5d63` / `#7fb3ba`) là màu nhấn của ứng dụng và nằm trong danh sách màu CHỮ
 * hợp lệ của `check-tokens.mjs:274` — tức nó đã qua cửa tương phản, dù dấu này không phải chữ.
 * Nó cũng đúng ngữ nghĩa: một điểm cắt đang chờ là **việc người dùng đang làm**, không một
 * trạng thái của câu.
 *
 * ⚠️ **Vì sao KHÔNG dùng `error`:** một chỗ cắt chưa thực hiện không phải một lỗi, và nhuộm
 * đỏ nó là dạy người dùng một mức khẩn cấp sẽ phải lạm phát ở story sau — cùng lý lẽ mà
 * `StatusBar.vue` đã ghi cho câu báo.
 *
 * 🔴 **`height` đo bằng `em` và `vertical-align: text-bottom` giữ nguyên — chiều cao HÀNG
 * không được đổi.** Lưới dùng `subgrid`, nên một phần tử inline cao hơn line box sẽ đẩy
 * **cả track hàng** và kéo theo ô bản dịch *(cái giá đó đã đo ở 2.5b: hàng Hán Việt song song
 * đẩy ô bản dịch lên 388px)*. `1,3em` nằm trong line box của cỡ chữ nguồn — đo lại nếu ai đổi
 * `--leading-*` của cột này.
 */
.cut-mark {
  display: inline-block;
  width: 2px;
  height: 1.3em;
  vertical-align: text-bottom;
  background-color: var(--color-primary);
}

/*
 * 🔵 **2026-08-17 (Ice chốt) — VIỀN `has-cuts` ĐÃ GỠ, cùng với chính lớp đó.**
 *
 * Nó là `border-left: 2px solid var(--color-ornament)` trên ô nguyên văn, và nó tồn tại vì
 * một lý do **đã hết đúng**: ở chế độ Hán Việt dấu cắt *"không vẽ được"*, nên ô cần một kênh
 * nói *"câu này đang có điểm chờ"* mà không nói **ở đâu**. Story 2.9 · AC9 làm dấu cắt vẽ
 * được ở **cả hai** kiểu xem ⇒ kênh thay thế hết việc.
 *
 * 🔴 Ice dùng thật rồi chốt: *"bỏ dấu gạch đứng ở trước câu đi, nó không cần thiết"*. Hai
 * kênh cho một khái niệm bắt người đọc học hai ký hiệu, và cái mờ hơn thì nói ít hơn.
 *
 * ⚠️ Lớp `has-cuts` **gỡ luôn**, không giữ lại làm mối nối cho test: một lớp không tạo kiểu
 * gì là mã chết, và giữ nó chỉ để `e2e` chọn được là mở đúng tiền lệ mà kho này cố ý chưa mở
 * *(`e2e/support/workspace.mjs` §"không thêm mối nối `data-` chỉ để test chọn được")*.
 * `data-cut-count` ở lại — nó chở **số**, tức nhiều hơn một cờ, và hai spec đọc nó.
 *
 * ⚠️ Gỡ viền cũng xoá một lượt xê dịch 2px mà chỉ những ô có điểm cắt phải chịu.
 */

/*
 * Cột ② — số câu. `on-surface-variant` (Quyết định #9(a)); `DESIGN.md` frontmatter khai
 * `ornament` và **nó sai** — `DESIGN.md:208` cấm `ornament` làm màu chữ, `tokens.json:98-101`
 * cưỡng chế bằng `check:tokens`. Tài liệu tự mâu thuẫn, và một cổng đứng về một phía.
 */
.cell-num {
  text-align: right;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  line-height: var(--leading-ui-label);
  color: var(--color-on-surface-variant);
  font-variant-numeric: tabular-nums;
}

.tok-source-cjk {
  font-family: var(--face-source-cjk);
  font-size: var(--font-source-cjk);
  line-height: var(--leading-source-cjk);
  color: var(--color-on-surface);
}

.tok-source-latin {
  font-family: var(--face-source-latin);
  font-size: var(--font-source-latin);
  line-height: var(--leading-source-latin);
  color: var(--color-on-surface);
}

.tok-editor {
  font-family: var(--face-editor);
  font-size: var(--font-editor);
  line-height: var(--leading-editor);
  color: var(--color-on-surface);
}

/*
 * 🔴 AC3 — Ô TRỐNG CÓ **CHIỀU CAO THẬT** VÀ ĐƯỜNG **ĐỨT NÉT**.
 *
 * Đây là chỗ khuyết tật *"sập hố"* chết theo cấu trúc. Nó tới từ một `<span>` rỗng **cao 0 px**
 * — caret không có hộp nào để lấy chiều cao, và `Backspace` ở offset 0 của editing host duy
 * nhất không có chỗ nào để xoá lui vào.
 *
 * ⚠️ `1.95em` nhân với `font-size` của **chính ô** (token `editor`, 15px) ⇒ đúng **một dòng**.
 * Đừng đổi sang `rem`: gốc tài liệu là `ui`, không phải `editor`.
 * ⚠️ **Đo 2026-08-14, WKWebView 605.1.15:** ô rỗng **38,00 px** = ô có chữ **38,00 px**; bấm
 * chuột thật vào nó cho `type = "Caret"`, gõ một ký tự cho `beforeinput insertText` huỷ được
 * và chữ hạ cánh.
 *
 * 🔵 Và đây là lượt **hoàn nguyên có lý do** một bản vá cũ: Story 2.3 từng đặt
 * `.sent:empty { display: inline-block; min-width: 1ch }` để một câu chưa dịch có chỗ mà bấm.
 * Nó **không** chữa được ca đó và **đẻ ra** một khuyết tật thị giác *(mỗi câu chưa dịch chiếm
 * một khoảng trắng lạ ⇒ một Chương mới hiện ra thành một dải khoảng hở rời rạc)*. Lời giải
 * đúng nằm ở **cấu trúc**, không ở một lượt vá hình học: một Ô có chiều cao, không một span.
 */
/*
 * 🔴 `white-space: pre-line` — TIỀN ĐỀ VẬN HÀNH của FR134/AD-46, không một dòng trang trí.
 * Story 2.5d, Quyết định #2 đường (b) (Ice ký 2026-08-15). **Đừng đổi mà không đọc hết.**
 *
 * Nó làm HAI việc, và cả hai đều đo được trên WKWebView 605.1.15 thật
 * (`2-5d-ban-do/README.md`, 2026-08-15):
 *
 * ① **Vẽ ra `\n` đã lưu.** Ô này render `{{ s.target_text }}`, tức một chuỗi đi thẳng vào
 *    DOM thành text node. Đo được: cùng một `"A\nB"`, `white-space: normal` cho **1 dòng**,
 *    `pre-line` cho **2**. Không có dòng này, một Chương vừa mở hiện bản dịch **mất hết
 *    ngắt đoạn** — đĩa đúng, màn hình sai, không lỗi nào được ném.
 *
 * ② **Quyết định hình dạng DOM mà engine dựng lúc gõ.** Đo được: `execCommand(
 *    'insertLineBreak')` trong ô này dựng một **text node `"\n"`** *(0 phần tử con,
 *    `textContent === "A\nB"`)*; đối chứng **không** `pre-line` thì chính lệnh đó dựng
 *    `<br>` và `textContent` đọc ra **`"AB"`** — mất trắng ranh giới trên đường ghi.
 *    ⇒ Đổi giá trị này là **lật nhánh ① của `onBeforeInput`** ở một tệp khác, trong im lặng.
 *
 * ⚠️ `pre-line` chứ **không** `pre-wrap` (Ice ký): `pre-wrap` giữ cả khoảng trắng đầu/cuối
 * dòng, tức mọi bản dịch cũ trên đĩa có khoảng trắng thừa sẽ **đổi hình dạng hiển thị** ngay
 * lượt cập nhật này. Hai giá trị cho **cùng** số dòng (đo: 2/2) nên vế ① không mất gì.
 * ⚠️ `check:tokens` **không** soi `white-space` (`PURE_COLOR_PROPS`/`COMPOSITE_COLOR_PROPS`)
 * ⇒ viết thẳng ở đây là hợp lệ, không cần token, không cần miễn trừ có tên.
 * ⚠️ Cột nguyên văn **không** nhận dòng này — story không có AC nào đòi đụng nó, và
 * `SourceHanViet.vue` đã có `pre-wrap` riêng của nó.
 */
.cell-tgt {
  white-space: pre-line;
}
.cell-tgt.empty {
  min-height: 1.95em;
  border-bottom-style: dashed;
  border-bottom-color: var(--color-outline);
}

/*
 * 🔴 Ô ĐANG GÕ KHÔNG VẼ VÒNG FOCUS CỦA TRÌNH DUYỆT — Ice bắt bằng mắt 2026-08-13.
 *
 * Triệu chứng ở hình dạng cũ: một khung viền bao quanh câu đang sửa, chiếm chỗ và đẩy dòng
 * văn lệch so với nguyên văn. Trong lưới cái giá còn cao hơn: một khung viền trên một ô sẽ
 * **đẩy chính hàng đó cao lên**, tức bố cục nhảy mỗi lần con trỏ đổi hàng.
 *
 * ⚠️ NFR17 **không** mất gì: chỉ báo tiêu điểm ở đây là một **caret nhấp nháy** cộng vạch lề
 * `primary` (UX-DR19) cộng nền `surface-accent` ngay dưới — ba kênh, mạnh hơn hẳn một đường
 * viền. Luật vẫn giữ nguyên cho mọi nút và ô nhập khác trong sản phẩm.
 */
.cell-tgt:focus {
  /* aura-allow-outline-none: ô đang gõ có CARET nhấp nháy làm chỉ báo tiêu điểm, cộng vạch lề `primary` (UX-DR19) và nền `surface-accent` — một vòng focus nữa là cùng một thông tin nói ba lần, và nó đẩy chính hàng đó cao lên. */
  outline: none;
}

/*
 * 🔴 NỀN HÀNG ĐANG SỬA — và đây là chỗ *"hàng không còn là một hộp"* phải trả giá.
 *
 * Nó tô trên **ô bản dịch**, không trên cả hàng, vì không có phần tử nào là "cả hàng". Tô đủ
 * năm ô đòi năm lớp `:class` song song và năm phép so `editorCaretSegmentId === s.id` — đắt
 * hơn hẳn thứ nó mua. ⇒ Phạm vi tối thiểu: đúng ô người dùng đang gõ.
 *
 * ⚠️ Ghi ra thay vì để người sau tưởng đã xét: nếu UX sau này đòi nền **cả hàng**, chỗ sửa là
 * năm `v-for` chứ không một dòng CSS.
 */
.cell-tgt.editing {
  background-color: var(--color-surface-accent);
}

/*
 * 🔴 CỜ KẾT ĐOẠN CỦA **BẢN DỊCH** — chỉ báo PHI HÌNH HỌC. Story 2.5d, FR134 · AD-46,
 * Quyết định #4 đường (Ⓑ), Ice ký 2026-08-16 **sau khi có số**, không trước.
 *
 * ⚠️ **VÌ SAO KHÔNG DÙNG LẠI KHUÔN `.cell.para-end` CỦA CỜ NGUỒN — một phép đo, không một gu.**
 * Năm cột là năm `subgrid` chia **chung một tập track hàng** và `.cell` mặc định
 * `align-self: stretch`, nên **track = max(chiều cao các ô cùng hàng)**. Đo trên WKWebView
 * 605.1.15 thật (bàn đo vòng 5, 2026-08-15), cùng một hàng, nền **38,00 px**:
 *
 *   | hình dạng | track hàng |
 *   |---|---|
 *   | `padding-bottom: 14px` **chỉ** ở ô bản dịch | **46,00 px** — và ô nguyên văn **cũng** 46 |
 *   | đổi kiểu đường kẻ đáy ở ô bản dịch | **38,00 px** |
 *   | một ký tự ở cột nhãn trạng thái | **38,00 px** |
 *
 * ⇒ Một *"khoảng thở"* đặt riêng ở ô bản dịch **kéo cả năm ô giãn theo**, tức nó nói dối:
 * người đọc thấy cả hàng thở ra và tưởng **bản gốc** cũng kết đoạn ở đó. **Hai cấu trúc
 * đoạn khác nhau không biểu diễn được bằng hai khoảng thở trong cùng một lưới.**
 *
 * 🔴 **`border-bottom-color`, KHÔNG `border-bottom-width`.** Đường viền cộng vào chiều cao
 * hộp, nên một lượt làm dày là đúng cái bẫy 46 px ở trên bằng một cái tên khác. Màu đổi,
 * bề dày **giữ nguyên 1px**.
 *
 * ⚠️ **KHÔNG** thêm giá trị vào `SEGMENT_RULE_VALUES` và **KHÔNG** dựng một khối `.rule-<x>`:
 * Kiểm I (`check-commands.mjs`) đối chiếu **ba chiều và cả chiều ngược lại**, nên một
 * `.rule-<x>` lạ trong CSS làm cổng FAIL. Đây là một class của **ô**, không một giá trị vạch.
 * ⚠️ **KHÔNG** `opacity` trung gian (Kiểm D) và **KHÔNG** `ornament` làm màu chữ — đây là màu
 * **đường viền**, không màu chữ, nên sàn AA của Kiểm C không áp; `--color-primary` đã là token
 * của vạch lề, dùng lại nó giữ *"ranh giới có nghĩa"* nói bằng **một** thứ tiếng.
 *
 * ⚠️ **GIỚI HẠN THẬT:** ô **chưa dịch** đã dùng `border-bottom-style: dashed`. Một ô vừa chưa
 * dịch vừa kết đoạn sẽ hiện **nét đứt màu primary** — đọc được, nhưng đó là một tổ hợp chưa
 * ai vẽ trong `DESIGN.md`. Ghi ra thay vì để người sau tưởng đã được xét.
 *
 * 🔵 **2026-08-16 (code review) — lượt tái dùng token này ĐÃ ĐƯỢC XÉT, Ice ký GIỮ.** Lượt rà
 * nêu đúng một chỗ: `--color-primary` đang mang nghĩa *"hàng đang có con trỏ"* ở cột vạch
 * (`.rule.rule-primary`), nên hai nghĩa khác nhau nay dùng chung một sắc **trong cùng một
 * khung nhìn**, và người đã học *"primary = hàng đang hoạt động"* có thể đọc nhầm viền này
 * thành *"hàng tôi vừa bấm"*. Ice giữ nguyên, vì hai chỗ khác **hình dạng** (nền vạch đặc ở
 * cột ① so với một đường viền 1px ở đáy ô) và cùng nghĩa rộng *"đang bật"*.
 * ⇒ 🔴 Đây là một lượt tái dùng **có chủ ý**, không một lượt với tay lấy token gần nhất. Lượt
 * rà sau đừng hỏi lại; nếu muốn lật thì phải mang một phép đo về **nhầm lẫn thật**, không một
 * lý lẽ về sự đồng nhất — và một token thứ hai là một quyết định thiết kế phải viết vào
 * `DESIGN.md`, không một dòng CSS.
 */
.cell-tgt.tgt-para-end {
  border-bottom-color: var(--color-primary);
}

/*
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 HÀNG ĐÃ CẮT BỎ KHỎI BẢN DỊCH — AC3 của Story 2.5c (FR133)
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * *"Hàng **vẫn nằm trong lưới**, gạch ngang và mờ đi"* — người dùng phải **thấy mình đã bỏ
 * gì**. Một hàng biến mất khỏi lưới là đúng thứ AC3 cấm: quyết định *"đoạn này không thuộc
 * bản dịch"* biến thành một lỗ hổng im lặng.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 `on-surface-variant`, KHÔNG `ornament` — Quyết định #6, Ice ký 2026-08-15
 * ─────────────────────────────────────────────────────────────────────────────
 * `DESIGN.md:148` khai `grid-row-omitted: { color: ornament, decoration: line-through }`, và
 * vế **màu** của dòng đó **không thi hành được**: `ornament` nằm trong
 * `contrast.neverTextTokens` của `tokens.json` với câu *"KHÔNG một ngoại lệ nào — token này
 * không bao giờ là màu chữ"*, và `check-tokens.mjs:1300-1334` cưỡng chế nó.
 *
 * **Đo 2026-08-15 trên nền `surface`** *(sàn AA = 4,5)*:
 *   `ornament` **2,44** (sáng) / **2,64** (tối) — trượt · `on-surface-variant` **5,60** /
 *   **5,56** — đạt.
 *
 * ⚠️ Và đây **không** phải một chỗ đặc tả bị bỏ qua: Quyết định #9(a) của Story 2.5b đã giải
 * **đúng bài toán này** một lần — cột số câu và cột nhãn trạng thái cũng được `DESIGN.md`
 * khai `ornament`, và Ice đổi cả hai sang `on-surface-variant` vì *"đây là **chữ thật**,
 * không phải nét"*. Ô đã cắt bỏ cũng là chữ thật, nên nó đi cùng đường.
 *
 * 🔴 Hai đường tắt đã bị từ chối, ghi ra để không ai thử lại: **①** thêm
 * `aura-allow-never-text: ornament` — tái mở đúng bảng miễn trừ mà lượt ra mã 2.5b vừa dọn
 * rỗng, và đặt chữ 2,44:1 lên màn hình (NFR17); **②** `opacity` trung gian — `DESIGN.md:230`
 * cấm *(`opacity` ở trạng thái nghỉ chỉ áp cho nét và nền, không áp cho chữ)* và Kiểm D của
 * `check-tokens` đỏ với mọi giá trị khác `0`/`1`.
 *
 * ⚠️ `text-decoration: line-through` đi qua Kiểm B tự do: `text-decoration` nằm trong
 * `COMPOSITE_COLOR_PROPS` nên cổng chỉ soi **phần màu** của giá trị ghép, và `line-through`
 * không mang màu nào.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ BỐN CỘT, KHÔNG NĂM — và cột bị bỏ ra là một quyết định
 * ─────────────────────────────────────────────────────────────────────────────
 * Không có phần tử nào là "một hàng" (xem khối đầu tệp), nên kiểu dáng cấp hàng phải nhân ra
 * **từng ô** — đúng khuôn `.para-end` đang lặp ở cả năm `v-for`. Ở đây chỉ **bốn** cột chữ
 * nhận `omitted`; **cột ① (vạch trạng thái) KHÔNG**.
 *
 * 🔴 Lý do là AC2: vạch chở `status`, và *"câu **vẫn giữ** trạng thái riêng của nó"*. Tô vạch
 * thành `on-surface-variant` là xoá đúng thông tin mà AC2 đòi giữ — người dùng sẽ không còn
 * đọc được câu mình vừa cắt bỏ đã ký hay chưa, tức không biết mình đang bỏ đi cái gì. Và một
 * vạch 2px thì `line-through` không nói gì cả.
 */
.cell.omitted {
  color: var(--color-on-surface-variant);
  text-decoration: line-through;
}

/* Cột ⑤ — nhãn trạng thái. `on-surface-variant` (Quyết định #9(a)), cùng lý do cột số câu. */
.cell-state {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  line-height: var(--leading-ui-label);
  color: var(--color-on-surface-variant);
}

/*
 * AC14 — một lượt từ chối đổi **đúng một pixel ở đúng chỗ người dùng vừa bấm**.
 *
 * ⚠️ `error` có vai `text` trong `tokens.json:56-63` và cặp `(error, surface)` đã có trong
 * `contrast.pairs` — không cặp mới nào phát sinh.
 */
.cell-state.refused {
  color: var(--color-error);
}
</style>
