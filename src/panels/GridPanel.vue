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
import { dispatch } from '../commands'
import { resolveSegmentRule, ruleClassOf, segmentRuleInputOf } from './editorSegments'
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
  ensureSegmentsLoaded,
  noteEditorEdit,
  setEditorCaret,
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
  const range = (() => {
    const pos = document.caretPositionFromPoint(x, y)
    if (pos !== null) {
      const r = document.createRange()
      r.setStart(pos.offsetNode, pos.offset)
      r.collapse(true)
      return r
    }
    return document.caretRangeFromPoint(x, y)
  })()

  if (range === null) return false
  // 🔴 **PHÂN GIẢI ĐƯỢC KHÔNG PHẢI ĐÚNG CHỖ** — bắt bằng chính bộ e2e, 2026-08-12. Một
  // `Range` trỏ ra ngoài ô là "thành công" theo hàm này mà người dùng không gõ được một chữ.
  if (!cell.contains(range.startContainer)) return false
  return setCaret(range.startContainer, range.startOffset)
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
    target.focus()
    setCaret(target.firstChild ?? target, 0)
  },
  { flush: 'post' },
)

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 CỬA DUY NHẤT MÀ MỘT LƯỢT SỬA VĂN BẢN ĐI QUA — chặn theo `inputType`, không theo phím
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * `beforeinput` là điểm chặn mà chuẩn Input Events **bảo đảm** `cancelable`, và nó phủ cả
 * những đường không có phím nào: dán bằng menu chuột phải, kéo-thả văn bản, và lượt thay thế
 * của bộ kiểm chính tả hệ điều hành. Một handler bám `keydown` bỏ lọt trọn ba đường đó.
 *
 * Ba nhóm, ba xử lý:
 * ① **Cấu trúc đoạn** (`insertParagraph`/`insertLineBreak`) ⇒ **chặn**. AD-37 nói cấu trúc
 *    đoạn là dữ liệu **ĐÃ LƯU** (`segment.is_paragraph_end`), không phải thứ gõ ra.
 *    ⚠️ Quyền xuống dòng **trong ô bản dịch** là FR134/AD-46, **Story 2.5d**. Đừng mở sớm.
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

  // ① Cấu trúc đoạn — AD-37.
  if (event.inputType === 'insertParagraph' || event.inputType === 'insertLineBreak') {
    event.preventDefault()
    return
  }

  // ② Chèn từ ngoài bàn phím.
  const FROM_OUTSIDE = ['insertFromPaste', 'insertFromDrop', 'insertReplacementText']
  if (!FROM_OUTSIDE.includes(event.inputType)) return

  // 🔴 `preventDefault()` đứng TRƯỚC mọi cửa thoát — code review 2026-08-13. Cám dỗ là dời nó
  // xuống sau các guard để một lượt dán không bao giờ "biến mất". Đừng: không chặn nghĩa là
  // thả engine chạy hành vi dán mặc định, đúng đường đo được là tiêm markup. Một lượt dán
  // trượt thì người dùng dán lại; một `data-segment-id` mất là hỏng **vĩnh viễn** (AD-3).
  event.preventDefault()
  const raw = event.dataTransfer?.getData('text/plain') ?? event.data ?? ''
  // Xuống dòng → **một khoảng trắng**, không bị bỏ đi: dán hai đoạn vào một ô thì hai chữ ở
  // hai đầu ranh giới phải còn cách nhau, nếu không chúng dính thành một từ không tồn tại.
  const flat = raw.replace(/[\r\n]+/g, ' ').replace(/[ \t]+/g, ' ')
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
 * AC11 — `Enter` **trơn KHÔNG BAO GIỜ xác nhận**, và nó cũng không xuống dòng ở story này.
 *
 * ⚠️ Hai lớp cho hai thứ khác nhau, **không** hai nguồn sự thật: nhánh ① của [`onBeforeInput`]
 * chặn **lượt sửa DOM**; dòng dưới đây chặn **sự kiện phím** trước khi nó rơi xuống `keys.ts`,
 * nơi một `Enter` trần sẽ được đem đi so với bảng hợp âm.
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
  if (event.isComposing) return
  if (event.key === 'Enter') event.preventDefault()
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
            :class="{ 'para-end': s.is_paragraph_end }"
          ><!-- aura-allow-text: số thứ tự hàng — một con số dựng tại chỗ hiển thị, đúng luật "định dạng số và ngày giờ CHỈ ở frontend" (§Consistency Conventions), không một chuỗi giao diện dịch được. -->{{ i + 1 }}</div>
        </div>

        <!-- Cột ③ — NGUYÊN VĂN. Một bề mặt vùng chọn duy nhất, vai `'source'` (AC7). -->
        <div ref="colSrc" class="col col-src" :class="sourceTokenClass">
          <div
            v-for="s in editorSegments"
            :key="s.id"
            class="cell cell-src"
            :class="{ 'para-end': s.is_paragraph_end }"
            :data-segment-id="s.id"
            data-col="src"
          >
            <!--
              AC8 — Hán Việt sống TRONG ô nguyên văn, hai chế độ FR19, người dùng tự bật tắt.
              `surface-role="cell"` nhượng lượt đăng ký cho CỘT — xem `hanVietSurfaces.ts`.
            -->
            <SourceHanViet
              v-if="showHanViet"
              :source-text="s.source_text"
              :view-mode="viewMode"
              surface-role="cell"
            />
            <!-- aura-allow-text: nguyên văn của Tác phẩm — DỮ LIỆU, không chuỗi giao diện
                 (NFR16). Không `v-html` (AD-16). -->
            <template v-else>{{ s.source_text }}</template>
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
          -->
          <div
            v-for="s in editorSegments"
            :key="s.id"
            class="cell cell-tgt"
            :class="{
              'para-end': s.is_paragraph_end,
              empty: (editorEditedText.get(s.id) ?? s.target_text) === '',
              editing: editorCaretSegmentId === s.id,
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
            :class="{ 'para-end': s.is_paragraph_end, refused: errorSegmentId === s.id }"
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
  gap: var(--space-inline-sm);
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
.grid-scroll {
  height: 100%;
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
