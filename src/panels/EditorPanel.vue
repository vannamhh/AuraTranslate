<script setup lang="ts">
// Panel `Bản dịch` — **trang liền mạch**. Story 2.2 · AC1 · AC2 · AC3 · AC4 · AC5 · AC6 · AC7.
//
// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 VÙNG GÕ LÀ **MỘT CÂU TẠI MỘT THỜI ĐIỂM** — Story 2.3 · AC8, Ice ký đường (c) 2026-08-12
// ═════════════════════════════════════════════════════════════════════════════════
// Story 2.2 dựng bề mặt này **CHỈ-ĐỌC** và một cổng (**Kiểm J**) canh điều đó, vì gõ được mà
// chưa có đường lưu là một cửa sổ người dùng gõ rồi **mất trắng khi đóng app**. Story này
// dựng đường lưu (AD-35), nên cổng đó **hết hạn và đã được gỡ CÙNG LƯỢT** — không sớm hơn:
// đường flush nghiệm thu xanh ở `tests/segment_contract.rs` **trước** khi dòng dưới đây tồn tại.
//
// ⇒ `.doc` **KHÔNG** mang `contenteditable`. Nó sống trên **đúng một** `<span class="sent">` —
// chính câu mà caret đang chạm.
//
// **Vì sao (c) chứ không `contenteditable` trên cả `.doc`:** `data-segment-id` là sổ sách của
// toàn bộ Story 2.2 và nó sống **trong DOM**. Đặt `contenteditable` lên `.doc` cho trình duyệt
// quyền sửa cây đó — gộp span khi xoá qua ranh giới, tách span khi gõ giữa, xoá sạch span khi
// `⌘A` rồi gõ đè. Một `segment.id` biến mất khỏi DOM là một câu **không còn đường về hàng
// `segment` của nó**, và AD-3 nói id đã về hưu không bao giờ được tái dùng ⇒ hỏng vĩnh viễn.
// Đường (c) làm câu hỏi đó **biến mất**: trong một vùng gõ chỉ có **một** id và **không một
// ranh giới câu nào** để trình duyệt gộp qua.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 GIÁ TRỊ `"true"`, KHÔNG `"plaintext-only"` — và đó là một PHÉP ĐO, không một gu
// ─────────────────────────────────────────────────────────────────────────────────
// Mũi thăm dò Task 0.1 chạy trên **cả hai** engine *(Blink 151 · WebKit 26.5)*, bốn biến thể:
//
//   `true` một mình         → Blink tiêm `<pre id="clipsrc">`; WebKit tiêm `<span style>` +
//                             `<pre>`; **cả hai** để lại một `\n` thật trong `textContent`
//   `plaintext-only` một mình → Blink **giữ `\n`**; WebKit **đổi `\n` thành `<div>`** — tức tự
//                             tiêm phần tử KHỐI vào trong một `<span>` inline
//   `true` + lọc dán        → **0** phần tử, **0** `\n`, cả hai engine
//   `plaintext-only` + lọc  → **0** phần tử, **0** `\n`, cả hai engine
//
// ⇒ `plaintext-only` **không** phải lời giải và nó hỏng **khác nhau** trên hai engine; cái lọc
// là đòn bẩy. Khi đã có lọc, hai giá trị cho kết quả **y hệt**, nên `plaintext-only` đổi lấy
// con số **không** trong khi mang hai hành vi và một lịch sử hỗ trợ lệch. Xem [`onBeforeInput`].
//
// ⚠️ Đây không phải mở lại Quyết định #1: Ice ký **đường (c)**, và (c) không khai giá trị của
// thuộc tính. Mũi thăm dò chọn giá trị đó bằng số, trong lòng (c).
//
// ═════════════════════════════════════════════════════════════════════════════════
// Trạng thái dữ liệu thật hôm nay
// ═════════════════════════════════════════════════════════════════════════════════
// Trên dữ liệu thật, mọi `target_text` là chuỗi rỗng cho tới khi người dùng gõ — nên có một
// dòng nói thẳng điều đó (`panel.editor.nothing_translated`), và nó tự tắt ngay khi câu đầu
// tiên có chữ. Bằng chứng thị giác của story nằm ở bàn đo `2-3-ban-do-vung-go.html`.
//
// ⚠️ Vỏ `PanelFrame` đã lo hợp đồng tiêu điểm (AC7): `declareFocus(owner, () => root.value)`
// chạy từ Story 1.6, và `'panel.editor'` đã có trong `FOCUS_OWNERS`. **Đừng** dựng vạch tiêu
// điểm panel thứ hai — `.panel.focused::before` đã có, và nó là vạch của **panel** (UX-DR8),
// khác hẳn vạch của **segment** (UX-DR19) mà tệp này dựng.
import { computed, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'
import PanelFrame from './PanelFrame.vue'
import { useSelectionSurface } from './selectionContract'
import type { DockviewPanelProps } from '../layout/panelProps'
import { t } from '../i18n'
import {
  createRuleScheduler,
  measureGutterRules,
  SEGMENT_ID_ATTR,
  type GutterRule,
} from './editorGutter'
import { resolveSegmentRule, ruleClassOf, segmentRuleInputOf } from './editorSegments'
import {
  editorCaretSegmentId,
  editorChapterId,
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

const surface = useTemplateRef<HTMLElement>('surface')
// 🔵 2026-08-13 (Sprint Change Proposal, Ice ký) — vai đổi `'source'` → `'display'`, và đây
// là lượt ĐẢO CHIỀU một kết quả đã đo: AC23 của story này hỏi *"Auto-Lookup còn chạy trên bề
// mặt Editor không?"*, đo ra CÒN CHẠY, và đóng theo nhánh hợp lệ. Nó **không bao giờ hỏi "có
// NÊN chạy không?"** — Ice trả lời ngày 2026-08-13: không.
//
// Bề mặt này chứa **tiếng Việt đã dịch**; từ điển nhúng là zh→vi / en→vi. Một lượt tra ở đây
// trả **0 hàng, 0 lỗi, 0 ms** rồi THAY MẤT kết quả người dùng vừa tra từ Panel Source — đúng
// vòng tự thay thế mà `selectionContract.ts:11-17` đã bác cho Panel Lookup, chỉ tệ hơn một
// bậc vì thứ thay vào là **rỗng**, tức lớp "rỗng im lặng" mà cả dự án đặt ở trung tâm.
//
// 🔴 ĐỪNG gỡ lời gọi này. FR48 (Story 3.3) và FR60 (Story 7.7) đọc vùng chọn ở đây bằng lệnh
// của RIÊNG chúng; `'display'` tắt đúng MỘT đường — `currentSelectionText()`, tức đường tra
// TỪ ĐIỂN — chứ không tắt việc bề mặt được đăng ký.
// Ghim bằng máy: `check-commands.mjs` Kiểm F ③.
useSelectionSurface(surface, 'display')

const gutter = useTemplateRef<HTMLElement>('gutter')
const doc = useTemplateRef<HTMLElement>('doc')

onMounted(() => {
  // Idempotent — cùng khuôn `SourcePanel.vue::onMounted`. Gọi lại ở mỗi lượt mount (kể cả
  // sau một lượt đổi preset) là AN TOÀN và KHÔNG chạy lại IPC.
  void ensureSegmentsLoaded()
})

// ═════════════════════════════════════════════════════════════════════════════════
// Trạng thái màn hình — bốn ca, và một danh sách rỗng KHÔNG được nói thay cho ba ca kia
// ═════════════════════════════════════════════════════════════════════════════════
const loadErrorKey = computed(() => editorLoadError.value?.message_key ?? null)
const hasSegments = computed(() => editorSegments.value.length > 0)
/** Đã nạp xong, Chương có thật, nhưng **chưa ai bấm lệnh tách** — 25 Chương của Epic 1. */
const showNoSegments = computed(
  () => loadErrorKey.value === null && editorHasLoaded() && !hasSegments.value,
)
/**
 * 🔴 **MỘT LUẬT HIỂN THỊ NGOÀI BẢY AC — ghi ra để nó lật được bằng một dòng.**
 *
 * Bảy AC không nói gì về ca *"đã tách câu, nhưng chưa câu nào có bản dịch"* — vì trước
 * Quyết định #1 nó không tồn tại. Sau phán quyết đường (b) nó là trạng thái **thường trực**
 * của mọi Tác phẩm cho tới Story 2.3, và UX-DR27 nói thẳng cái giá: *"một khung trống câm
 * là thứ người dùng đọc thành hỏng"*. `SourcePanel.vue` đã đặt tiền lệ đúng ca này
 * (`panel.source.empty_chapter`).
 *
 * Dòng chú thích hiện **phía trên** trang văn, không thay nó — AC2/AC4/AC5 vẫn nghiệm thu
 * được trên fixture của Task 7 mà không phải gỡ dòng này ra.
 * **Chủ: Ice** *(một luật ngoài đơn hàng — chỗ lật là chính `v-if` dưới đây)*.
 */
const nothingTranslated = computed(
  () =>
    hasSegments.value &&
    // ⚠️ Vế `editorEditedText` là bắt buộc từ Story 2.3, không phải một lượt phòng xa: dòng này
    // hiện khi **chưa câu nào có bản dịch**, và `editorSegments` giữ bản **lúc nạp** nên nó ở
    // lại chuỗi rỗng suốt phiên gõ. Không có vế đó, người dùng gõ xong một câu mà màn hình vẫn
    // khẳng định *"chưa câu nào có bản dịch"* — một câu nói dối ngay phía trên chữ họ vừa gõ.
    editorSegments.value.every((s) => (editorEditedText.value.get(s.id) ?? s.target_text) === ''),
)
/**
 * Câu trạng thái mặc định của `PanelFrame` (*"Chưa có Chương nào để dịch."*) chỉ đúng khi
 * **không** có gì để nói — và *"đang chờ IPC"* **là** một thứ để nói.
 *
 * 🔴 `!editorPending` là vế bắt ở code review 2026-08-12. Thiếu nó, khoảng chờ của lượt nạp
 * đọc ra `true` *(chưa lỗi, chưa segment, `editorHasLoaded()` còn `false` nên `showNoSegments`
 * cũng `false`)*, và panel khẳng định **dứt khoát** rằng không có Chương nào — trong lúc
 * Chương đang trên đường về. Đó đúng là lớp lỗi mà doc-comment của [`editorHasLoaded`] tồn
 * tại để chặn *(`editorPanelState.ts:67-75`)*; nó chỉ lọt qua bằng một chuỗi khác.
 *
 * ⚠️ Trong khoảng chờ, panel hiện **trống** — không một câu nào. Đó là có chủ ý: một dòng
 * *"đang tải"* cho một lượt đọc SQLite cục bộ là thứ nháy lên rồi tắt, và UX-DR27 chỉ cấm
 * khung trống **thường trực**, không cấm một khoảng chờ.
 */
const showFrameStatus = computed(
  () =>
    loadErrorKey.value === null &&
    !editorPending.value &&
    !hasSegments.value &&
    !showNoSegments.value,
)

// ═════════════════════════════════════════════════════════════════════════════════
// AC3 · AC12 — năm giá trị vạch, qua ĐÚNG MỘT hàm phân giải (`./editorSegments.ts`)
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * Lớp CSS của vạch từng câu — `null` ⇒ *không vạch*, tức **không phần tử nào** được dựng.
 *
 * ⚠️ Một `Map`, không một phép tìm trong `v-for`: bảng này được tra một lần cho mỗi vạch
 * đang vẽ, và một `Array.find` ở đó cho ra O(n²) trên **9.850** câu.
 */
const ruleClassById = computed(() => {
  const caret = editorCaretSegmentId.value
  const map = new Map<number, string>()
  for (const s of editorSegments.value) {
    const cls = ruleClassOf(resolveSegmentRule(segmentRuleInputOf(s, caret)))
    if (cls !== null) map.set(s.id, cls)
  }
  return map
})

// ═════════════════════════════════════════════════════════════════════════════════
// AC2 · AC14 — chiều cao vạch ĐO từ hình học thật (Quyết định #2)
// ═════════════════════════════════════════════════════════════════════════════════
const rules = ref<GutterRule[]>([])

const remeasure = (): void => {
  const g = gutter.value
  const d = doc.value
  if (g === null || d === null) {
    rules.value = []
    return
  }
  rules.value = measureGutterRules(g, d, new Set(ruleClassById.value.keys()))
}

const scheduler = createRuleScheduler(remeasure)

/**
 * 🔴 BỐN NGUỒN KÍCH HOẠT, và cái thứ tư đi **qua** cái thứ nhất chứ không có listener riêng.
 *
 * ① đổi kích thước panel ⇒ `ResizeObserver` trên `.doc`;
 * ② font web nạp xong ⇒ `document.fonts.ready` *(ba font nhúng — UX-DR4; trước lượt đó hình
 *   học là hình học của font dự phòng, tức một phép đo đúng cho một màn hình sắp đổi)*;
 * ③ nội dung hay tập câu-có-vạch đổi ⇒ `watch` ngay dưới;
 * ④ **đổi theme** — không listener riêng, và đó là một phép đo chứ không phải một lượt bỏ
 *   sót: `applyTheme()` ghi typography từ `tokens.typography`, một bảng **không** phân theo
 *   theme *(chỉ `tokens.colors` phân theo theme)*, nên một lượt đổi theme không đổi được cỡ
 *   chữ, họ chữ hay giãn dòng — tức không đổi hình học. Và nếu một ngày nó có đổi, chiều cao
 *   `.doc` đổi theo ⇒ `ResizeObserver` của ① bắt được. Kho hôm nay cũng chưa có công tắc đổi
 *   theme lúc chạy: `applyTheme` chỉ được gọi một lần ở `main.ts`.
 */
let observer: ResizeObserver | null = null

onMounted(() => {
  // ⚠️ Không `?.` — `document.fonts` là **không-nullish** theo lib DOM, và `src/tokens/fonts.ts`
  // đã đứng trên đúng mệnh đề đó từ Story 1.4. Một `?.` phòng hờ ở đây là một nhánh mà kiểu
  // nói không bao giờ chạy, tức mã chết (`@typescript-eslint/no-unnecessary-condition` bắt
  // đúng nó lúc chạy `check:lint` 2026-08-12).
  void document.fonts.ready.then(() => {
    scheduler.schedule()
  })
  document.addEventListener('selectionchange', onSelectionChange)
})

watch(
  [() => editorSegments.value, ruleClassById, doc],
  () => {
    const d = doc.value
    observer?.disconnect()
    observer = null
    if (d !== null) {
      observer = new ResizeObserver(() => {
        scheduler.schedule()
      })
      observer.observe(d)
    }
    // Trang văn vừa được dựng lại ⇒ chép văn bản đang gõ ngược vào DOM. Xem [`restoreEditedText`].
    restoreEditedText()
    scheduler.schedule()
  },
  { flush: 'post' },
)

onBeforeUnmount(() => {
  // 🔴 Cả ba đều phải nhả. Một lượt đo đã hẹn mà chạy sau khi cây DOM đã tháo sẽ đọc hình
  // học của một phần tử mồ côi và ghi vào một `ref` không ai còn render.
  scheduler.cancel()
  observer?.disconnect()
  observer = null
  document.removeEventListener('selectionchange', onSelectionChange)
})

// ═════════════════════════════════════════════════════════════════════════════════
// AC5 — "tiêu điểm bàn phím chạm tới một câu", trên một bề mặt KHÔNG gõ được
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * 🔴 Không có caret trên một bề mặt không `contenteditable` — nên *"con trỏ chạm tới"* đọc
 * từ **neo của vùng chọn DOM** (`Selection.anchorNode`).
 *
 * Đó là một cơ chế **có thật và đến được bằng bàn phím**, không phải một cách nói tránh:
 * Story 1.18 đã dựng đúng đường đó cho Panel Source *(`tabindex="0"` trên bề mặt chữ, rồi
 * `Shift+Mũi tên` — `deferred-work.md:608`)*, và một cú bấm chuột cũng đặt một vùng chọn
 * thu gọn. Cùng cơ chế ở đây, cùng lý do.
 *
 * ⚠️ Neo **ngoài** `.doc` ⇒ caret về `null`. Đó là câu đúng theo nghĩa đen: người dùng đang
 * chọn ở một panel khác thì *"con trỏ ở đây"* là sai, và giữ vạch `primary` sáng là để vạch
 * nói dối — đúng thứ doc-comment của `PanelFrame.vue::focused` tồn tại để chặn.
 */
/**
 * ═══════════════════════════════════════════════════════════════════════════════════
 * 🔴 **DOM SỞ HỮU VĂN BẢN BẢN DỊCH. VUE KHÔNG.** — và đây là bài học đắt nhất của story
 * ═══════════════════════════════════════════════════════════════════════════════════
 * Template render `s.target_text` — **bản LÚC NẠP**, một giá trị chỉ đổi ở một lượt nạp lại.
 * Nó **không** render văn bản đang gõ. Hệ quả: trong suốt một phiên gõ, vnode của câu **không
 * bao giờ đổi**, nên Vue **không bao giờ** chạm vào text node đó.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * HAI KHUYẾT TẬT THẬT ĐÃ TỚI TỪ VIỆC LÀM NGƯỢC LẠI — Ice bắt cả hai bằng mắt
 * ─────────────────────────────────────────────────────────────────────────────────
 * ① **Gõ ngược từ phải sang trái** (2026-08-13). Template từng render
 *    `editorEditedText.get(s.id) ?? s.target_text`. Mỗi phím gõ đổi `editedText` ⇒ Vue render
 *    lại. Bản đầu lý luận *"giá trị render bằng đúng thứ đang có trong DOM nên Vue không ghi"* —
 *    **sai**: Vue so **vnode cũ với vnode mới**, không so với DOM. Vnode cũ mang chuỗi **trước**
 *    lượt gõ ⇒ Vue ghi `textContent`, **dựng lại text node**, caret rơi về **offset 0**, và ký
 *    tự kế tiếp chèn vào đầu.
 * ② **Chữ đã dịch BIẾN MẤT khỏi màn hình khi bấm xuống dưới** (2026-08-13). Bản vá cho ① dùng
 *    một biến `frozenText` **dùng chung cho mọi câu** — nên nó áp chuỗi của câu này lên câu
 *    khác. Một bản vá chữa triệu chứng mà không chữa nguyên nhân **đẻ ra một khuyết tật nặng
 *    hơn**: người dùng thấy bản dịch của mình mất *(dù trên đĩa vẫn còn)*.
 *
 * ⇒ Nguyên nhân chung của cả hai: **hai chủ sở hữu cho một text node**. Cách duy nhất đóng cả
 * lớp lỗi đó là để **một** bên sở hữu. Bên đúng là **DOM**, vì đó là bên người dùng đang gõ vào.
 *
 * ⚠️ Vế *"văn bản đang gõ sống sót một lượt tháo panel"* **không mất** — nó chỉ đổi đường: sau
 * mỗi lượt mount/nạp lại, [`restoreEditedText`] chép `editedText` **ngược trở lại DOM** một lần.
 * Một lượt ghi có chủ đích ở một thời điểm xác định, khác hẳn một binding phản ứng ghi ở mỗi
 * phím gõ.
 */

/**
 * Chép văn bản đang gõ từ state **ngược lại DOM** — chạy sau mỗi lượt dựng lại trang văn.
 *
 * Chỗ gọi: `watch` bố cục ở dưới (`flush: 'post'`), tức sau mỗi lượt `editorSegments` đổi và sau
 * mỗi lượt mount. Đó là **hai** thời điểm duy nhất Vue dựng lại các `<span>`.
 *
 * ⚠️ **Không** chạm câu đang **được gõ THẬT** *(tức đang giữ tiêu điểm)*: DOM ở đó đã đúng, và
 * một lượt ghi sẽ đá caret ra. Phép kiểm hỏi `document.activeElement`, **không** hỏi thuộc tính
 * `contenteditable` — sau một lượt mount lại, thuộc tính vẫn còn mà tiêu điểm thì không, và đó
 * đúng là lượt cần khôi phục nhất.
 */
function restoreEditedText(): void {
  const host = doc.value
  if (host === null) return
  const edited = editorEditedText.value
  if (edited.size === 0) return

  for (const el of host.querySelectorAll<HTMLElement>(`[${SEGMENT_ID_ATTR}]`)) {
    const id = segmentIdOf(el)
    if (id === null) continue
    const text = edited.get(id)
    if (text === undefined) continue
    // ⚠️ Hỏi **tiêu điểm thật**, không hỏi thuộc tính. Một câu vẫn mang `contenteditable` sau
    // một lượt mount lại *(vì `editorCaretSegmentId` còn trỏ vào nó)* nhưng **không ai đang gõ
    // vào nó** — bỏ qua nó ở đó là bỏ đúng câu người dùng vừa gõ dở, tức mất chữ trên màn hình.
    if (document.activeElement === el) continue
    if (el.textContent !== text) el.textContent = text
  }
}

/**
 * Vị trí caret chụp **TRƯỚC** lượt đổi thuộc tính `contenteditable`, để trả lại sau đó.
 *
 * 🔴 Vì sao phải chụp: lắp `contenteditable` lên một `<span>` bắt engine dựng một **editing
 * host** mới ở đó, và một lượt như vậy được phép **thả vùng chọn**. Caret nhảy về đầu câu ở
 * đây là thứ người dùng đọc thành *"ứng dụng ăn mất chỗ tôi đang gõ"* — hệ quả 2 của Quyết
 * định #1, và story ghi nó là ca test đắt nhất.
 */
let savedCaret: { node: Node; offset: number } | null = null

function onSelectionChange(): void {
  const host = doc.value
  if (host === null) return

  // ⚠️ `window.getSelection`, KHÔNG `document.getSelection` — hai hàm cùng một API, nhưng
  //    `check-layout.mjs::ALLOWED_GLOBAL_MEMBERS` chỉ cho phép cái thứ nhất (Story 1.17 ·
  //    Quyết định #1a), và `selectionContract.ts` đã đi qua đúng cửa đó. Nới danh sách cho
  //    một bí danh là mở rộng bề mặt được phép để đổi lấy con số không.
  const anchor = window.getSelection()?.anchorNode ?? null
  if (anchor === null || !host.contains(anchor)) {
    setEditorCaret(null)
    return
  }

  const from = anchor instanceof Element ? anchor : anchor.parentElement
  const sent = from?.closest<HTMLElement>(`[${SEGMENT_ID_ATTR}]`) ?? null
  const raw = sent?.getAttribute(SEGMENT_ID_ATTR) ?? null
  const id = raw === null ? Number.NaN : Number(raw)

  // Chụp caret TRƯỚC `setEditorCaret` — lượt gọi đó kích hoạt một lượt render đổi
  // `contenteditable`. Xem [`savedCaret`].
  const range = window.getSelection()?.rangeCount ? window.getSelection()?.getRangeAt(0) : null
  savedCaret =
    range === null || range === undefined
      ? null
      : { node: range.startContainer, offset: range.startOffset }

  setEditorCaret(Number.isFinite(id) ? id : null)
}

/**
 * Trả caret về đúng chỗ nó vừa ở, **sau** khi DOM đã nhận `contenteditable` mới.
 *
 * `flush: 'post'` là bắt buộc: trước lượt patch của Vue thì `<span>` mới còn chưa gõ được, nên
 * một lượt `focus()` ở đó không đặt được caret vào trong nó.
 *
 * ⚠️ **Không** `focus()` khi caret về `null`: đó là lúc người dùng vừa bấm sang một panel khác,
 * và kéo tiêu điểm về đây là giành nó khỏi chỗ họ vừa chọn — đúng thứ `PanelFrame.vue::focused`
 * tồn tại để không làm.
 */
watch(
  editorCaretSegmentId,
  (id) => {
    const host = doc.value
    if (id === null || host === null) return

    const editable = host.querySelector<HTMLElement>('[contenteditable="true"]')
    const saved = savedCaret
    if (saved === null || !host.contains(saved.node)) return
    if (editable === null || !editable.contains(saved.node)) return
    // Tiêu điểm đã ở trong đúng câu ⇒ engine không thả vùng chọn, đừng đụng vào nó.
    if (document.activeElement === editable) return

    editable.focus()
    const max =
      saved.node.nodeType === Node.TEXT_NODE
        ? (saved.node as Text).data.length
        : saved.node.childNodes.length
    setCaret(saved.node, Math.min(saved.offset, max))
  },
  { flush: 'post' },
)

/** Rời tiêu điểm khỏi bề mặt ⇒ không câu nào *"đang sửa"* nữa. */
function onSurfaceFocusOut(event: FocusEvent): void {
  const next = event.relatedTarget
  if (next instanceof Node && doc.value?.contains(next)) return
  setEditorCaret(null)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 2.3 · AC8 — BA HANDLER CỦA VÙNG GÕ, và mỗi cái đóng một ca ĐO ĐƯỢC
// ═════════════════════════════════════════════════════════════════════════════════
/** Câu mang `<span>` của một node bất kỳ trong `.doc`, hoặc `null`. */
function sentenceOf(node: Node | null): HTMLElement | null {
  const from = node instanceof Element ? node : (node?.parentElement ?? null)
  return from?.closest<HTMLElement>(`[${SEGMENT_ID_ATTR}]`) ?? null
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 ĐƯỜNG CHUỘT — VÀ NÓ TỒN TẠI VÌ MỘT PHÉP ĐO TRONG **WKWebView THẬT**, KHÔNG VÌ PHÒNG XA
// ═════════════════════════════════════════════════════════════════════════════════
//
// Story 2.2 đọc câu-đang-chạm từ **neo vùng chọn DOM** (`Selection.anchorNode`), và
// `deferred-work.md:2120-2126` ghi thẳng rằng chữ ký đó *"phủ cơ chế hôm nay, không phủ Story
// 2.3"*. Đây là lượt xét lại đó (**AC22**), và phán quyết đến từ số đo chứ không từ lý lẽ.
//
// **Đo 2026-08-12 qua bộ e2e, trong cửa sổ Tauri thật (WKWebView 605.1.15), chuột THẬT:**
//
// | Sau một cú bấm vào `<span class="sent">` chỉ-đọc | WKWebView |
// |---|---|
// | `pointerdown` tới `.doc` | **0 lượt** — engine này không phát Pointer Events ở đường đó |
// | `mousedown` · `mouseup` · `click` | **có**, `target` đúng `<span>` |
// | `selectionchange` | **0 lượt** |
// | `getSelection().type` | **`"None"`**, `rangeCount = 0` |
// | `document.activeElement` | `SECTION.mode` — **không** phải `.doc`, dù nó có `tabindex="0"` |
//
// ⇒ Trên WKWebView, một cú bấm vào văn bản **chỉ-đọc không tạo vùng chọn nào**. Đường
// `Selection.anchorNode` của Story 2.2 vì thế **không bao giờ chạy tới** — người dùng bấm vào
// một câu và **không gì xảy ra**: không caret, không vùng gõ, không gõ được. Cùng họ với khuyết
// tật mà Story 1.22 C2 tìm ra (*WKWebView không đặt tiêu điểm cho `<button>` khi bấm chuột*),
// và cùng lý do nó lọt qua tới tận đây: mọi bàn đo trước chạy trên Blink hoặc trên **WebKit của
// Playwright**, hai engine **có** tạo vùng chọn ở lượt bấm.
//
// **Hai phép đo nữa quyết hình dạng bản vá:**
//   ① `el.focus()` trên `<span contenteditable>` ⇒ `activeElement` vẫn là `SECTION.panel`.
//      Tiêu điểm **không giữ được** — cùng bức tường mà `attribution-focus.e2e.mjs` đã ghi:
//      *"trong một `section.panel[tabindex="-1"]`, tiêu điểm không giữ được trên nút dù đặt
//      bằng cách nào"*.
//   ② Dựng một `Range` rồi `selection.addRange(r)` ⇒ `activeElement` thành **`SPAN.sent`**,
//      `type` thành **`"Caret"`**. ⇒ **`addRange` là thứ đặt được tiêu điểm ở đây, không
//      `focus()`.** Bản vá đi đường đó.
//
// ⚠️ Vế *"gõ được"* **không** hỏng: `document.execCommand('insertText', …)` trong cùng phiên đo
// cho `beforeinput` → `input` → chữ hạ cánh. Thứ không đi qua là `browser.keys()` của bộ e2e —
// nó chỉ bắn `keydown`/`keyup`, không đi vào đường nhập văn bản gốc. Một **giới hạn của bộ đo**,
// không một khuyết tật sản phẩm; ghi ra để không ai chẩn đoán lại.

/**
 * Đặt caret vào **đúng điểm vừa bấm**, rồi để lượt render biến câu đó thành vùng gõ.
 *
 * `caretRangeFromPoint` là API của WebKit/Blink cho đúng việc này; `caretPositionFromPoint` là
 * tên chuẩn hoá. Thử cái chuẩn trước, rơi về cái kia — không engine nào của dự án thiếu cả hai.
 *
 * ⚠️ **Không** `preventDefault()`: hành vi mặc định của `mousedown` là thứ đặt caret trên các
 * engine **có** làm việc đó (Blink). Bản vá này **thêm** một đường, nó không thay đường nào.
 */
function placeCaretAtPoint(sent: HTMLElement, x: number, y: number): boolean {
  // ⚠️ **Không** `?.` trên hai lời gọi dưới đây: `lib.dom` khai cả hai là phương thức **bắt
  // buộc** của `Document`, nên một optional chain ở đó là một nhánh mà kiểu nói không bao giờ
  // chạy — `@typescript-eslint/no-unnecessary-condition` bắt đúng nó lúc chạy `check:lint`
  // (đo 2026-08-12, cùng lớp với `document.fonts` mà khối `onMounted` đã ghi). Cả hai TRẢ VỀ
  // `null` được, và đó là nhánh thật sự phải xử.
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
  // 🔴 **PHÂN GIẢI ĐƯỢC KHÔNG PHẢI ĐÚNG CHỖ** — bắt được bằng chính bộ e2e, 2026-08-12.
  //
  // Bản đầu trả `true` ngay khi có một `Range`. Đo trong WKWebView thật: nó đặt caret vào một
  // `#text` **ngoài** câu đang gõ (`anchorInEditable: false`), và mọi lượt chèn văn bản trả
  // `false` — tức "thành công" theo hàm này mà người dùng không gõ được một chữ.
  //
  // ⚠️ Ca này **không** hiếm, nó là ca THƯỜNG: một câu **chưa dịch** là `<span>` rỗng, rộng **0
  // pixel**. Phép hit-test theo toạ độ vì thế rơi vào phần tử liền kề — dòng
  // `panel.editor.nothing_translated` ngay trên, hay một câu khác. Đó đúng là câu đầu tiên người
  // dùng bấm vào ở mọi Chương mới.
  if (!sent.contains(range.startContainer)) return false
  return setCaret(range.startContainer, range.startOffset)
}

/**
 * Đặt caret thu gọn tại `(node, offset)`.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════
 * 🔴 `Selection.setPosition`, **KHÔNG** `removeAllRanges()` + `addRange()` — một phép đo
 * ═══════════════════════════════════════════════════════════════════════════════════
 * Hai đường trông tương đương, và trên Blink chúng tương đương. Trên **WKWebView thì không**,
 * và khác biệt rơi đúng vào ca thường nhất của tính năng này.
 *
 * Đo 2026-08-13 trong cửa sổ Tauri thật, trên một câu **CHƯA DỊCH** *(`<span>` rỗng, 0 childNode)*,
 * ngay sau một cú bấm chuột thật:
 *
 * | Đường đặt caret | `getSelection().type` | `execCommand('insertText')` |
 * |---|---|---|
 * | `removeAllRanges()` + `addRange(r)` | **`"None"`** | **`false`** — chữ không hạ cánh |
 * | `setPosition(el, 0)` | **`"Caret"`** | **`true`** — chữ hạ cánh |
 *
 * ⇒ Với một phần tử soạn thảo được **rỗng**, WebKit bỏ qua lượt `addRange`. Đó là nguyên nhân
 * cuối cùng của triệu chứng *"bấm vào câu chưa dịch thì không gõ được"* mà Ice báo — ba lượt vá
 * trước đó *(đặt thuộc tính đồng bộ · dời sang `mouseup` · siết neo phải trong câu)* đều cần
 * thiết nhưng không đủ, vì cả ba vẫn kết thúc bằng `addRange`.
 */
function setCaret(node: Node, offset: number): boolean {
  const selection = window.getSelection()
  if (selection === null) return false
  selection.setPosition(node, offset)
  return selection.type === 'Caret'
}

/**
 * `mousedown` — dời vùng gõ sang câu vừa bấm, và đặt thuộc tính **ĐỒNG BỘ, NGAY TẠI ĐÂY**.
 *
 * ═══════════════════════════════════════════════════════════════════════════════════
 * 🔴 `setAttribute` TAY BÊN CẠNH MỘT BINDING CỦA VUE — và đó là toàn bộ điểm của hàm này
 * ═══════════════════════════════════════════════════════════════════════════════════
 * Trông như thừa: template đã có `:contenteditable="editorCaretSegmentId === s.id ? 'true' : …"`.
 * Nó **không** thừa, vì hai lượt đó xảy ra ở hai thời điểm khác nhau, và cái sau là **quá muộn**.
 *
 * Trình duyệt xử lý một cú `mousedown` **ngay sau** khi handler này trả về: nó tìm phần tử
 * focus được **gần nhất** tính từ chỗ bấm đi ngược lên, focus nó, rồi đặt caret nếu chỗ đó soạn
 * thảo được. Vue thì vá DOM ở một **microtask sau** — nên nếu chỉ gọi `setEditorCaret`, tại thời
 * điểm engine ra quyết định, `<span>` **vẫn chưa** `contenteditable`.
 *
 * **Đo trong cửa sổ Tauri thật (WebKit 605.1.15, chuột thật), trước bản vá:**
 * `activeElement` = **`SECTION.panel`** *(tổ tiên focus được gần nhất — chính là gốc
 * `tabindex="-1"` mà `PanelFrame` dựng cho AD-34 §2)*, `getSelection().type` = **`"None"`**,
 * `rangeCount` = **0**. Tức người dùng bấm vào một câu và **không gì xảy ra**: không caret,
 * không gõ được. Ice xác nhận cùng triệu chứng khi gõ tay.
 *
 * 🔵 **Và đây là chỗ một chẩn đoán sai suýt lật nhầm một hợp đồng.** Bản chẩn đoán đầu đọc
 * `activeElement = SECTION.panel` thành *"hợp đồng focus AD-34 giành mất tiêu điểm"*, và lời giải
 * theo hướng đó là sửa `focus.ts`/`FOCUS_OWNERS` — tức lật một AD mà **Story 1.6 sở hữu**. Phép
 * đo nói khác: **không ai giành cả**. `focus.ts::enter()` chỉ chạy lúc **đổi chế độ**, và chốt
 * chống-rơi-về-`body` của nó chỉ **ghi console**, không gọi `focus()`. `PanelFrame` chỉ **nghe**
 * `focusin`/`focusout`. Thứ đặt tiêu điểm lên `section.panel` là **hành vi mặc định của trình
 * duyệt**, và nó chọn đúng như vậy chỉ vì phần tử được bấm chưa soạn thảo được. ⇒ AD-34 **không
 * phải nguyên nhân, và không cần đổi một dòng**.
 *
 * ⚠️ **Không** `preventDefault()`: hành vi mặc định là **chính thứ ta cần** — sau dòng
 * `setAttribute` dưới đây, engine thấy một phần tử soạn thảo được và tự đặt caret **đúng ký tự
 * vừa bấm**. Đó cũng là lý do bản vá này không cần `caretRangeFromPoint` cho đường thường.
 *
 * ⚠️ Vue **không** đánh nhau với dòng này: lượt render kế tiếp tính ra **cùng một giá trị**
 * (`editorCaretSegmentId` vừa được đặt ở ngay dưới), nên bản vá của Vue là một no-op trên thuộc
 * tính đó. Hai đường hội tụ thay vì tranh nhau.
 */
/**
 * Câu **gần chỗ bấm nhất**, cho một cú bấm rơi vào khoảng trống của `.doc`.
 *
 * 🔴 Vì sao cần: Ice bắt được 2026-08-13 — *"rất khó click để focus"*. Một trang văn liền mạch
 * để lại **nhiều khoảng trống** mà cú bấm rơi vào: vùng dưới câu cuối, khoảng thừa cuối mỗi
 * dòng, và **chính chỗ của mọi câu chưa dịch** *(một `<span>` rỗng rộng 0 pixel — không có gì
 * để trúng)*. Bấm vào những chỗ đó mà không gì xảy ra là thứ người dùng đọc thành *"panel này
 * không gõ được"*.
 *
 * Luật chọn, theo đúng thứ tự người ta mong đợi:
 *   ① câu nào có hình chữ nhật **chứa** tung độ chỗ bấm ⇒ trong dòng đó, câu gần theo hoành độ;
 *   ② không câu nào ⇒ câu có mép **gần tung độ đó nhất** *(bấm dưới đáy ⇒ câu cuối)*.
 *
 * ⚠️ Duyệt `getClientRects()`, không `getBoundingClientRect()`: một câu chảy inline có thể bắt
 * đầu giữa dòng này và kết thúc giữa dòng khác, nên hộp bao của nó phủ cả những dòng nó không
 * hề chiếm — cùng lý do `editorGutter.ts` đo bằng `getClientRects()`.
 */
function nearestSentenceTo(host: HTMLElement, x: number, y: number): HTMLElement | null {
  let best: HTMLElement | null = null
  let bestScore = Number.POSITIVE_INFINITY

  for (const el of host.querySelectorAll<HTMLElement>(`[${SEGMENT_ID_ATTR}]`)) {
    for (const r of el.getClientRects()) {
      // Khoảng cách dọc tới hộp (0 nếu chỗ bấm nằm trong dải dòng đó), rồi mới tới ngang.
      const dy = y < r.top ? r.top - y : y > r.bottom ? y - r.bottom : 0
      const dx = x < r.left ? r.left - x : x > r.right ? x - r.right : 0
      // Dọc nặng hơn ngang: người dùng nhắm một DÒNG trước, rồi mới tới chỗ trong dòng.
      const score = dy * 1000 + dx
      if (score < bestScore) {
        bestScore = score
        best = el
      }
    }
    // Một câu RỖNG không có hình chữ nhật nào — nó vẫn phải chọn được. Lấy vị trí của nó qua
    // hộp bao (suy biến nhưng có toạ độ), với một điểm phạt để câu CÓ CHỮ luôn thắng khi hoà.
    if (el.getClientRects().length === 0) {
      const r = el.getBoundingClientRect()
      const dy = y < r.top ? r.top - y : y > r.bottom ? y - r.bottom : 0
      const score = dy * 1000 + Math.abs(x - r.left) + 0.5
      if (score < bestScore) {
        bestScore = score
        best = el
      }
    }
  }
  return best
}

function onDocMouseDown(event: MouseEvent): void {
  const host = doc.value
  if (host === null) return
  // Bấm trúng một câu thì dùng câu đó; rơi vào khoảng trống thì chọn câu gần nhất.
  const sent =
    sentenceOf(event.target instanceof Node ? event.target : null) ??
    nearestSentenceTo(host, event.clientX, event.clientY)
  const id = segmentIdOf(sent)
  if (id === null || sent === null) return

  // 🔴 THỨ TỰ: đặt thuộc tính TRƯỚC, rồi mới tới state. Dòng dưới đây là dòng mà lượt xử lý
  // `mousedown` của engine sắp đọc; `setEditorCaret` chỉ dọn dẹp phần state và có thể chạy sau.
  sent.setAttribute('contenteditable', 'true')
  setEditorCaret(id)
}

/**
 * `mouseup` — đặt caret vào đúng điểm vừa bấm, **sau** khi engine đã xử lý xong cú bấm.
 *
 * ⚠️ `@mouseup`, **không** `@click`: `scripts/check-commands.mjs` Kiểm A cưỡng chế AD-34 §1 —
 * *"handler chuột chỉ được `dispatch` một command đã đăng ký"* — và nó canh đúng thuộc tính
 * `@click`. Một `@click="onDocMouseUp"` làm cổng đó **đỏ**, và đúng ra nó phải đỏ: `@click` là
 * cửa của **thao tác**, còn đây là một lượt đặt con trỏ.
 *
 * 🔴 **THỨ TỰ NÀY LÀ MỘT PHÉP ĐO, và bản đầu đã sai nó.** Bản đầu đặt caret trong một `nextTick`
 * ngay sau `mousedown`. Đo trong WKWebView thật: `nextTick` là một microtask nên nó chạy **trước
 * `mouseup`**, rồi lượt xử lý cú bấm của engine **thu vùng chọn về không** — ngay trước
 * `execCommand`, `rangeCount = 0` · `type = "None"` · `activeElement = SECTION.mode`, và mọi lượt
 * chèn văn bản trả `false`. Tức bản vá **trông như** chạy *(câu ĐÃ thành `contenteditable`)* mà
 * người dùng vẫn không gõ được một chữ.
 *
 * ⇒ Lượt đặt caret phải đứng **sau** `mouseup`. Trên Blink hai đường hội tụ: engine tự đặt caret ở
 * `mousedown`, và lượt đặt lại ở đây trỏ vào **cùng một điểm**, nên nó là một no-op quan sát được.
 */
function onDocMouseUp(event: MouseEvent): void {
  const host = doc.value
  if (host === null) return
  // Cùng luật với `mousedown` — một cú bấm rơi vào khoảng trống vẫn phải cho ra một caret.
  const sent =
    sentenceOf(event.target instanceof Node ? event.target : null) ??
    nearestSentenceTo(host, event.clientX, event.clientY)
  if (sent === null) return
  if (placeCaretAtPoint(sent, event.clientX, event.clientY)) return

  // Không phân giải được điểm bấm vào TRONG câu ⇒ neo vào **CUỐI** câu.
  //
  // ⚠️ Cuối chứ không đầu, và đó là một quyết định về hành vi: bấm vào khoảng trống sau một
  // câu là cách người ta nói *"cho tôi viết tiếp"*. Neo về đầu ở đó làm chữ mới chèn trước chữ
  // cũ — cùng triệu chứng *"gõ ngược"* mà doctrine *"DOM sở hữu văn bản"* đã đóng, chỉ tới từ
  // một nguyên nhân khác.
  //
  // ⚠️ Nhánh này KHÔNG lý thuyết: một câu **chưa dịch** là một `<span>` rỗng, không text node
  // nào — đúng câu đầu tiên người dùng bấm vào ở mọi Chương mới.
  // Cuối câu = số node con (với câu rỗng thì là 0, tức chính nó).
  setCaret(sent, sent.childNodes.length)
  ensureCaretNextFrame(sent)
}

/**
 * Đặt lại caret ở **frame kế tiếp** nếu tới lúc đó vẫn chưa có caret nào.
 *
 * 🔴 Vì sao cần một lượt thứ hai: lượt xử lý cú bấm của engine chưa xong khi handler `mouseup`
 * trả về — nó còn chạy tiếp và **thu vùng chọn về không** trong cùng lượt sự kiện. Đo được ở
 * WKWebView: `setPosition` gọi trong `mouseup` cho `type === "Caret"` ngay tại chỗ, nhưng đọc
 * lại sau đó là `"None"`; cùng lời gọi ấy từ một lượt `browser.execute` **riêng** thì đứng.
 *
 * ⚠️ Điều kiện `type !== 'Caret'` là bắt buộc: nếu engine **đã** đặt caret đúng chỗ người dùng
 * bấm *(đường thường trên Blink, và trên WebKit khi câu có chữ)* thì lượt này **không được**
 * dời nó — đó sẽ là một cú nhảy caret do chính ta gây ra, đúng thứ hệ quả 2 của Quyết định #1
 * cấm.
 *
 * ⚠️ Một frame, **không** một vòng lặp tự phục hồi. `focus.ts:114-118` đã ghi lý do cho đúng
 * hình dạng này: *"đừng sửa bằng cách focus lại vòng lặp — nó sẽ đánh nhau với người dùng đang
 * Tab và với hộp thoại của hệ điều hành"*.
 */
function ensureCaretNextFrame(sent: HTMLElement): void {
  requestAnimationFrame(() => {
    if (!sent.isConnected) return
    if (sent.getAttribute('contenteditable') !== 'true') return
    if (window.getSelection()?.type === 'Caret') return
    setCaret(sent, sent.childNodes.length)
  })
}

function segmentIdOf(el: HTMLElement | null): number | null {
  const raw = el?.getAttribute(SEGMENT_ID_ATTR) ?? null
  if (raw === null) return null
  const id = Number(raw)
  return Number.isFinite(id) ? id : null
}

/**
 * 🔴 CỬA DUY NHẤT MÀ MỘT LƯỢT SỬA VĂN BẢN ĐI QUA — và nó chặn theo `inputType`, không theo phím.
 *
 * `beforeinput` là điểm chặn mà chuẩn Input Events **bảo đảm** `cancelable`, và nó phủ cả những
 * đường không có phím nào: dán bằng menu chuột phải, kéo-thả văn bản, và lượt thay thế của bộ
 * kiểm chính tả hệ điều hành. Một handler bám `keydown` bỏ lọt trọn ba đường đó.
 *
 * Ba nhóm, ba xử lý:
 *
 * ① **Cấu trúc đoạn** (`insertParagraph`/`insertLineBreak`) ⇒ **chặn**. AD-37 nói cấu trúc đoạn
 *    là dữ liệu **ĐÃ LƯU** (`segment.is_paragraph_end`), không phải thứ gõ ra. Và nó là ca mà
 *    đường (c) **không tự đóng**: (c) chặn *gộp qua ranh giới*, nhưng một `Enter` giữa câu làm
 *    engine **tách chính `<span>` đó** — tức nhân đôi một `data-segment-id`.
 *
 * ② **Chèn từ một nguồn NGOÀI bàn phím** (dán · kéo-thả · thay thế) ⇒ **chặn, rồi tự chèn văn
 *    bản THUẦN đã làm phẳng**. Đo được ở Task 0.1: không có nhánh này thì cả hai engine tiêm
 *    markup vào trong câu — `<pre>`, `<span style>`, và trên WebKit cả `<div>` khối — cộng một
 *    `\n` thật vào `target_text` của **một** câu.
 *
 * ③ **Mọi thứ còn lại** (`insertText`, các lượt xoá) ⇒ **để engine làm**. Tự cài lại phép xoá
 *    là tự viết một nửa engine editor, đúng thứ hàng Deferred *"thư viện editor"* tồn tại vì nó
 *    — và hàng đó có chủ là **Story 2.4**, không phải story này.
 */
function onBeforeInput(event: InputEvent): void {
  const sent = sentenceOf(event.target instanceof Node ? event.target : null)
  if (sent === null) return

  // ① Cấu trúc đoạn — AD-37.
  if (event.inputType === 'insertParagraph' || event.inputType === 'insertLineBreak') {
    event.preventDefault()
    return
  }

  // ② Chèn từ ngoài bàn phím.
  const FROM_OUTSIDE = ['insertFromPaste', 'insertFromDrop', 'insertReplacementText']
  if (!FROM_OUTSIDE.includes(event.inputType)) return

  // 🔴 `preventDefault()` đứng TRƯỚC mọi cửa thoát, và đó là chủ ý — code review 2026-08-13.
  //
  // Cám dỗ là dời nó xuống sau các guard để một lượt dán không bao giờ "biến mất". Đừng.
  // Không chặn nghĩa là thả engine chạy hành vi dán mặc định trên một vùng chọn tràn qua ranh
  // giới câu — đúng đường mà Task 0.1 đo được là tiêm `<pre>`, `<span style>`, và trên WebKit
  // cả `<div>` khối, cộng khả năng gộp hai `<span>`. Một `data-segment-id` mất là một câu
  // không còn đường về hàng `segment` của nó, và AD-3 nói id đã về hưu **không bao giờ** được
  // tái dùng ⇒ hỏng ở đây là hỏng **vĩnh viễn**. Một lượt dán trượt thì người dùng dán lại.
  event.preventDefault()
  const raw = event.dataTransfer?.getData('text/plain') ?? event.data ?? ''
  // Xuống dòng → **một khoảng trắng**, không bị bỏ đi: dán hai đoạn vào một câu thì hai chữ ở
  // hai đầu ranh giới phải còn cách nhau, nếu không chúng dính thành một từ không tồn tại.
  const flat = raw.replace(/[\r\n]+/g, ' ').replace(/[ \t]+/g, ' ')
  if (flat === '') return

  const selection = window.getSelection()
  // ⚠️ Chẩn đoán viết bằng tiếng Anh, KHÔNG tiếng Việt: Kiểm A của `check-i18n.mjs` cấm chuỗi
  // tiếng Việt ở vị trí mã trong `.vue` và `.rs`. Cùng khuôn `eprintln!` của `lib.rs`.
  if (selection === null || selection.rangeCount === 0) {
    console.warn('[editor] insert skipped: no readable selection')
    return
  }
  const range = selection.getRangeAt(0)
  // ⚠️ Chỉ chèn khi vùng chọn còn NẰM TRONG câu đang gõ. Một `Range` trỏ ra ngoài là đường
  // ghi văn bản vào một chỗ không ai kiểm — kể cả ra ngoài `.doc`.
  //
  // 🔴 Thoát ở đây là **bỏ hẳn** lượt chèn, và người dùng thấy đúng: không một ký tự nào hạ
  // cánh. Đó là hành vi ĐÚNG *(xem khối `preventDefault()` trên)*, nhưng nó không được **im
  // lặng** — một thao tác không có kết quả và không có dấu vết là thứ người dùng đọc thành
  // "ứng dụng hỏng". Chèn vào một chỗ đoán mò thì tệ hơn hẳn: đây là bản dịch, không phải
  // một ô nháp.
  if (!sent.contains(range.startContainer)) {
    console.warn(
      '[editor] insert skipped: selection starts outside the editable sentence ' +
        `(inputType=${event.inputType}) — reselect within one sentence and paste again`,
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
  reportEdit(sent)
}

/**
 * `Enter` bị chặn ở **tầng phím** nữa, cộng thêm nhánh ① của [`onBeforeInput`].
 *
 * ⚠️ Hai lớp cho **một** mệnh đề, và đó không phải hai nguồn sự thật: chúng chặn hai thứ khác
 * nhau. Nhánh ① chặn **lượt sửa DOM**; dòng dưới đây chặn **sự kiện phím** trước khi nó rơi
 * xuống `keys.ts` — nơi một `Enter` trần sẽ được đem đi so với bảng hợp âm. Hôm nay không hợp
 * âm nào dùng `Enter` trần, nên nó đúng do tình cờ; `keys.ts:418-424` ghi đúng lớp lỗi đó cho
 * `Shift+B`.
 */
function onEditKeydown(event: KeyboardEvent): void {
  // 🔴 IME đứng trước mọi thứ — cùng dòng và cùng lý do `keys.ts:504`. Một lượt commit
  // composition của bộ gõ tiếng Việt phát `keydown` mang `code` vật lý; ăn nó là ăn mất chữ.
  if (event.isComposing) return
  if (event.key === 'Enter') event.preventDefault()
}

/** Đọc văn bản **từ chính DOM** rồi đưa vào tập chờ. Chỗ duy nhất làm việc đó. */
function reportEdit(sent: HTMLElement): void {
  const id = segmentIdOf(sent)
  if (id === null) return
  noteEditorEdit(id, sent.textContent)
}

/**
 * Một lượt sửa đã hạ cánh trong DOM ⇒ ghi vào tập chờ.
 *
 * ⚠️ Đọc `textContent`, **không** dựng lại chuỗi từ `event.data`: một lượt xoá, một lượt undo
 * của trình duyệt (`⌘Z`) và một lượt commit IME đều không mang văn bản kết quả trong event.
 * `textContent` là **thứ người dùng đang nhìn thấy**, và đó đúng là thứ phải được lưu.
 *
 * ⚠️ Nội dung của `.sent::after` (`⏐`) **không** nằm trong `textContent` — nó là một
 * pseudo-element (Quyết định #3 của Story 2.2), và Task 0.1 đo lại được điều đó trên bề mặt
 * gõ được: không rò trên cả hai engine, cả khi copy.
 */
function onEditInput(event: Event): void {
  const sent = sentenceOf(event.target instanceof Node ? event.target : null)
  if (sent !== null) reportEdit(sent)
}

/** Chỉ để đọc trong template — `editorChapterId` là dữ kiện chẩn đoán, không hiển thị. */
const chapterId = computed(() => editorChapterId.value)
</script>

<template>
  <PanelFrame owner="panel.editor" status-key="panel.editor.status" :show-status="showFrameStatus">
    <!-- Lỗi nạp nói ra bằng chuỗi CỦA NÓ, không im — cùng khuôn `SourcePanel.vue`. -->
    <p v-if="loadErrorKey !== null" class="load-error">{{ t(loadErrorKey) }}</p>
    <p v-else-if="showNoSegments" class="load-error">{{ t('panel.editor.no_segments') }}</p>

    <div ref="surface" class="editor-surface">
      <div v-if="hasSegments" class="edwrap" :data-chapter-id="chapterId">
        <!-- 🔴 Một luật hiển thị NGOÀI bảy AC — xem `nothingTranslated`. Chỗ lật là dòng này. -->
        <p v-if="nothingTranslated" class="untranslated-note">
          {{ t('panel.editor.nothing_translated') }}
        </p>

        <div class="edbody">
          <!--
            AC2 — MÁNG rộng `gutter-width`, vạch dọc 2px. `aria-hidden`: vạch lề là một lớp
            thông tin **thị giác** trùng lặp với trạng thái mà trình đọc màn hình sẽ đọc từ
            chính câu (Story 2.5 mang `status`); một chuỗi div rỗng không nhãn ở đây chỉ là
            tiếng ồn cho công nghệ trợ giúp.
          -->
          <div ref="gutter" class="gutter" aria-hidden="true">
            <div
              v-for="r in rules"
              :key="r.id"
              class="gmark"
              :class="ruleClassById.get(r.id)"
              :style="{ top: `${r.top}px`, height: `${r.height}px` }"
            ></div>
          </div>

          <!--
            🔴 AC1 — **MỘT** dòng văn liên tục. Không `display: block` cho câu, không grid,
            không bảng, không ô. Mỗi câu là một `<span>` chảy inline; chỗ ngắt đoạn tới từ cờ
            `is_paragraph_end` **đã lưu** (AD-37 cấm suy ra lúc render), và nó là một `<br>` —
            một dấu ngắt dòng, không phải một hộp.

            🔴 `tabindex="0"` — cùng lý do và cùng cái giá mà Story 1.18 đã ghi cho
            `SourcePanel.vue::.original`: một `<div>` không sửa được KHÔNG nhận `Shift+Mũi tên`
            nếu nó không vào được vòng tiêu điểm. Đây là thứ làm vế **bàn phím** của AC5 có
            thật, và làm hợp đồng vùng chọn (Story 1.18) thật sự chạy được trên bề mặt này.
            ⚠️ Nó KHÔNG làm bề mặt gõ được — xem khối AC18 ở đầu tệp.
          -->
          <div
            ref="doc"
            class="doc tok-editor"
            tabindex="0"
            @focusout="onSurfaceFocusOut"
            @mousedown="onDocMouseDown"
            @mouseup="onDocMouseUp"
            @beforeinput="onBeforeInput"
            @input="onEditInput"
            @keydown="onEditKeydown"
          >
            <template v-for="s in editorSegments" :key="s.id">
              <!--
                aura-allow-text: BẢN DỊCH của người dùng — dữ liệu, không phải chuỗi giao diện
                của `vi.json` (NFR16). Không `v-html` (AD-16).

                🔴 `contenteditable` trên ĐÚNG MỘT câu — câu caret đang chạm (Quyết định #1
                đường (c)). `undefined` chứ không `'false'`: Vue bỏ hẳn thuộc tính khi giá
                trị là `undefined` (và `null` cũng vậy, nhưng `null` không thuộc kiểu của thuộc
                tính này — `vue-tsc` đỏ). Còn `contenteditable="false"` là một lời khai KHÁC — nó **chặn** kế
                thừa, và nó sẽ làm `isTypingZone` của `keys.ts` đọc ra `false` cho một câu
                đang không gõ thay vì không thấy gì. Hôm nay hai cái cho cùng hành vi; mai
                một `contenteditable` ở tổ tiên làm chúng khác nhau.

                ⚠️ Văn bản đọc `editorEditedText` TRƯỚC `s.target_text`: `editorSegments` giữ
                bản **lúc nạp** cho FR117 (Story 2.7), nên nó KHÔNG đổi khi người dùng gõ.
                Không có vế thứ nhất, một lượt tháo/mount lại panel (đổi preset bố cục) vẽ
                lại chữ CŨ lên trên chữ người dùng vừa gõ.
              -->
              <span
                class="sent"
                :data-segment-id="s.id"
                :data-caret="editorCaretSegmentId === s.id ? '' : null"
                :contenteditable="editorCaretSegmentId === s.id ? 'true' : undefined"
              >{{ s.target_text }}</span>
              <br v-if="s.is_paragraph_end" />
            </template>
          </div>
        </div>
      </div>
    </div>
  </PanelFrame>
</template>

<style scoped>
.editor-surface {
  height: 100%;
  min-height: 0;
}

/*
 * Lỗi nạp và ca "Chương chưa được tách câu" — cùng token và cùng lý do với
 * `SourcePanel.vue::.load-error` (Story 1.17 · Quyết định #7: `ui-md-wrap`, giãn dòng 1.66).
 */
.load-error {
  margin: 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.untranslated-note {
  margin: 0 0 var(--space-panel-block) 0;
  flex: none;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.edwrap {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

/*
 * 🔴 HỘP CUỘN LÀ **CHỖ NÀY**, không phải `.doc`.
 *
 * Máng và trang văn phải cuộn CÙNG NHAU: vạch lề được đặt `position: absolute` trong hệ toạ
 * độ của máng, và `measureGutterRules` tính `top` bằng hiệu hai `getBoundingClientRect()`.
 * Cho `.doc` cuộn một mình sẽ làm hiệu đó đổi theo lượt cuộn ⇒ vạch trôi khỏi câu của nó ngay
 * ở dòng thứ hai của một Chương dài.
 *
 * ⚠️ `align-items` để mặc định (`stretch`): trong một hộp cuộn, con của flex row căng theo
 * chiều cao **nội dung**, nên máng luôn cao đúng bằng trang văn — điều kiện để một vạch ở câu
 * cuối cùng có chỗ mà nằm.
 */
.edbody {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: auto;
}

/* AC2 · UX-DR3 — máng rộng ĐÚNG token, không một con số viết thẳng. */
.gutter {
  position: relative;
  flex: none;
  width: var(--space-gutter-width);
  padding-top: 4px;
}

/*
 * AC2 — vạch dọc 2px, thụt trái 8px, bo `sm`.
 *
 * ⚠️ `border-radius` đọc token `--radius-sm` (**2px**), KHÔNG phải `1px` của mockup:
 * `DESIGN.md` component token `segment-gutter-rule` ghi `radius: sm`, và `EXPERIENCE.md:312`
 * phân xử sẵn rằng khi bản dựng mâu thuẫn với tài liệu thì tài liệu thắng.
 *
 * 🔴 KHÔNG `box-shadow` ở đây và không ở đâu trong tệp này — Kiểm F cấm **tuyệt đối**, không
 * miễn trừ. Mẫu `box-shadow: 0 0 0 3px <cùng màu nền>` của mockup là một kỹ thuật đệm chứ
 * không phải bóng đổ, nhưng cổng đọc **thuộc tính** chứ không đọc ý định, và sự bất đối xứng
 * đó là có chủ ý (`check-tokens.mjs:1364-1368`).
 */
.gmark {
  position: absolute;
  left: 8px;
  width: 2px;
  border-radius: var(--radius-sm);
}

/*
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 BỐN MÀU VẠCH — MỖI GIÁ TRỊ MỘT KHỐI, VÀ ĐÓ LÀ ĐIỀU KIỆN ĐỂ CỔNG NHÌN THẤY GÌ
 * ═══════════════════════════════════════════════════════════════════════════════
 * Kiểm B của `check-tokens.mjs` đọc CSS, không đọc TypeScript. Bind màu qua `:style` từ một
 * hàm TS sẽ đi qua cổng mà không một dòng nào được soi. Bốn khối dưới đây là bản khai mà cổng
 * đọc được từng chữ, và `check-commands.mjs` đối chiếu chúng **hai chiều** với
 * `SEGMENT_RULE_VALUES`.
 *
 * ⚠️ Giá trị thứ năm — *không vạch* — **không có khối CSS**, vì nó không vẽ gì: `ruleClassOf`
 * trả `null` và `v-for` không dựng phần tử nào. Bốn khối + một sự vắng mặt có chủ = năm.
 *
 * ⚠️ Ba trong bốn màu này hôm nay **không đường nào tới được** trên dữ liệu thật — nguồn dữ
 * liệu của chúng thuộc Story 2.5 (`confirmed`), Story 2.8 (`ornament`) và Epic 7 (`tm-rule`).
 * Xem `./editorSegments.ts` để có bảng chủ đầy đủ. Chúng nghiệm thu được trong **bàn đo**.
 */
.gmark.rule-confirmed {
  background-color: var(--color-confirmed);
}

.gmark.rule-primary {
  background-color: var(--color-primary);
}

.gmark.rule-tm-rule {
  background-color: var(--color-tm-rule);
}

/*
 * ⚠️ `ornament` ở đây là màu **NỀN của một vạch**, không phải màu chữ — Kiểm C chỉ đỏ với
 * `color:`/`-webkit-text-fill-color:`, và đó là ranh giới đúng: UX-DR5 nói `ornament` là màu
 * của **nét**. Chỗ duy nhất trong tệp này cần một miễn trừ là ký tự `⏐` ở dưới.
 */
.gmark.rule-ornament {
  background-color: var(--color-ornament);
}

/*
 * 🔴 AC6 — BỀ MẶT ĐỌC TỰ KHAI TOKEN CỦA CHÍNH NÓ.
 *
 * Đóng **nửa Editor** của `deferred-work.md:130-133`: mặc định kế thừa từ `body` là `ui-md`
 * ở giãn dòng **1.5** — dưới sàn cứng 1.66 — và Kiểm E chỉ đọc `tokens.json` nên hoàn toàn mù
 * với việc component nào đang kế thừa gì. Token `editor` là họ `read`, 15px, giãn dòng
 * **1.95** (`tokens.json:397-403`).
 *
 * ⚠️ `.doc` thụt trái 8px — `DESIGN.md`, cùng số với `inset-left` của vạch, nên chữ và vạch
 * đứng trên cùng một đường.
 */
.doc {
  flex: 1;
  min-width: 0;
  padding-left: 8px;
}

.tok-editor {
  font-family: var(--face-editor);
  font-size: var(--font-editor);
  line-height: var(--leading-editor);
  color: var(--color-on-surface);
}

/*
 * ⚠️ `position: relative` là điều kiện để `::after` neo vào chính câu. Nó KHÔNG dựng một
 * hộp: một `<span>` inline vẫn chảy inline dưới `position: relative`.
 */
.sent {
  position: relative;
}

/*
 * 🔴 CÂU ĐANG GÕ KHÔNG VẼ VÒNG FOCUS CỦA TRÌNH DUYỆT — Ice bắt bằng mắt 2026-08-13.
 *
 * Triệu chứng: một **khung viền** bao quanh đúng câu đang sửa, và nó làm chữ trông **lệch** so
 * với Panel Nguyên văn — khung chiếm chỗ, đẩy dòng văn vào trong.
 *
 * Chỉ báo *"câu nào đang sửa"* của sản phẩm **đã có và khác hẳn**: vạch lề `primary` (UX-DR19)
 * cộng ký tự ranh giới `⏐` sáng lên ở `[data-caret]` (AC5 của Story 2.2). Một vòng focus thứ hai
 * là **cùng một thông tin nói hai lần**, và nó nói bằng thứ tiếng ồn nhất.
 *
 * ⚠️ NFR17 **không** mất gì: vòng focus tồn tại để người đi bàn phím biết tiêu điểm ở đâu, và ở
 * đây tiêu điểm là một **caret nhấp nháy** — chỉ báo mạnh hơn hẳn một đường viền. Luật vẫn giữ
 * nguyên cho mọi nút và ô nhập khác trong sản phẩm.
 */
.sent[contenteditable='true'] {
  /* aura-allow-outline-none: câu đang gõ có CARET nhấp nháy làm chỉ báo tiêu điểm, cộng vạch lề `primary` của UX-DR19 — một vòng focus nữa là cùng một thông tin nói hai lần, và nó đẩy dòng văn lệch khỏi Panel Nguyên văn (Ice bắt 2026-08-13). */
  outline: none;
}

/*
 * 🔴 **KHÔNG** ép câu rỗng chiếm chỗ — và đây là một lượt HOÀN NGUYÊN có lý do đo được.
 *
 * Bản 2026-08-13 từng đặt `.sent:empty { display: inline-block; min-width: 1ch }` để một câu
 * **chưa dịch** *(`<span>` rỗng, rộng 0 px)* có chỗ mà bấm vào. Nó **không** chữa được ca đó, và
 * nó **đẻ ra một khuyết tật thị giác**: mỗi câu chưa dịch chiếm một khoảng trắng lạ, nên một
 * Chương mới *(mọi câu đều rỗng)* hiện ra thành một dải khoảng hở rời rạc — Ice bắt được bằng mắt.
 *
 * Lời giải đúng nằm ở **tầng hành vi, không tầng hình học**: `onDocMouseDown` nhận cả cú bấm rơi
 * vào khoảng trống của `.doc` và tự chọn câu gần nhất *(xem [`nearestSentenceTo`])*. Người dùng
 * không phải trúng một mục tiêu 0 pixel, nên câu rỗng không cần chiếm chỗ.
 *
 * ⚠️ Và một ký tự thật (`U+200B`…) vẫn bị loại vì lý do cũ: nó nằm trong `textContent`, tức đi
 * thẳng vào `target_text` lúc flush — vết sẹo `WORD_JOINER` của Story 1.18b.
 */

/*
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 AC4 — RANH GIỚI CÂU `⏐` LÀ MỘT **PSEUDO-ELEMENT**, KHÔNG MỘT `<span>` THẬT
 * ═══════════════════════════════════════════════════════════════════════════════
 * Quyết định #3. Có một vết sẹo trực tiếp trong kho về đúng lớp lỗi này: Story 1.18b chèn
 * `WORD_JOINER` (`U+2060`) vào DOM Panel Source, và hệ quả là **rò ký tự lúc copy** trên
 * WKWebView — bôi đen bằng phím rồi `⌘C` dán ra chuỗi lẫn ký tự chèn, và `onCopy` phải dựng
 * lại chuỗi từ đường đọc DOM (`deferred-work.md:839-848`).
 *
 * Một `⏐` là `<span>` thật lặp lại đúng lỗi đó, lần này trên một bề mặt **sẽ gõ được ở Story
 * 2.3** — nên nó còn bị gõ đè, bị con trỏ đi xuyên qua, và bị đếm vào độ dài văn bản. Nội
 * dung của pseudo-element không nằm trong cây văn bản: không copy được, không chọn được,
 * không gõ đè được.
 *
 * ⚠️ CÁI GIÁ, ghi ra: pseudo-element không nhận `:hover` riêng *(nên `.sent:hover::after`)*,
 * và nó **không** hiện trong một bàn đo chép DOM thay vì chép CSS.
 *
 * ⚠️ Dấu miễn trừ ngay dưới viết trên **một dòng** có lý do đo được: `exemptAt` của
 * `check-tokens.mjs` so **dòng bắt đầu** của comment với dòng khai báo và đòi khoảng cách
 * ≤ 1. Một khối chú thích bốn dòng đặt ngay trên khai báo vẫn ĐỎ — đo lúc chạy cổng
 * 2026-08-12. Lý lẽ dài sống ở đây, dấu miễn trừ sống một dòng.
 *
 * Lý lẽ đầy đủ của miễn trừ `ornament`: UX-DR5 nói `ornament` và `tm-rule` không bao giờ là
 * màu của chữ, kèm *"ngoại lệ duy nhất đã đặc tả: ký tự ranh giới câu `⏐`"*; `tokens.json:99`
 * hẹn ngoại lệ đó cho **đúng story này**. `⏐` không mang nghĩa đọc được — nó là một đường kẻ
 * dọc vẽ bằng một code point, ở `opacity: 0` mặc định, và nó là NÉT chứ không phải chữ.
 */
.sent::after {
  content: '⏐';
  /* aura-allow-never-text: ornament — NÉT chứ không phải chữ; ngoại lệ `⏐` đã đặc tả ở UX-DR5 + tokens.json:99. */
  color: var(--color-ornament);
  opacity: 0;
}

/*
 * AC5 — hiện ở `0.55` khi rê chuột **hoặc** khi tiêu điểm chạm tới câu.
 *
 * ⚠️ **Cả hai theme cùng một số.** Mockup tối ghi `.75`; `DESIGN.md:382` chốt `0.55` không
 * phân theo theme, và `EXPERIENCE.md:312` phân xử rằng tài liệu thắng bản dựng.
 *
 * ⚠️ `[data-caret]` là vế **bàn phím** của AC5 — `:hover` không có vế đó. Cờ do tầng TS đặt
 * từ neo vùng chọn DOM, cùng nguồn với vạch `primary` của AC3.
 */
/*
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 CÂU **CHƯA DỊCH** KHÔNG VẼ RANH GIỚI — Ice ký 2026-08-13, sau một phép đo
 * ═══════════════════════════════════════════════════════════════════════════════
 * `opacity: 0` **không gỡ chỗ** — khác `display: none`, phần tử vẫn chiếm bề rộng. Nên `⏐` của
 * mỗi câu **chưa dịch** *(một `<span>` rỗng, nhưng `::after` của nó thì không rỗng)* đẩy văn bản
 * phía sau sang phải.
 *
 * **Đo 2026-08-13**, cùng bộ CSS này:
 *
 * | | mép trái câu cuối |
 * |---|---|
 * | không câu rỗng xen giữa | **72,0 px** |
 * | **bốn** câu rỗng xen giữa | **108,2 px** |
 *
 * ⇒ **9,05 px mỗi câu chưa dịch.** Một Chương **mới** có hàng chục câu như vậy liên tiếp, nên
 * khoảng thụt cộng dồn tới hàng trăm pixel — và nó **co lại dần** khi người dùng dịch xong từng
 * câu, tức bố cục **nhảy trong lúc làm việc**. Ice bắt bằng mắt hai lần *("chữ hiển thị lệch so
 * với bản gốc", rồi "chữ thụt vào")*.
 *
 * ⚠️ Lớp khuyết tật này **chỉ lộ ra ở Story 2.3**: tới hết 2.2 thì **mọi** câu đều rỗng, nên
 * không có gì để so lệch với.
 *
 * 🔴 **Đây là một lượt THU HẸP UX-DR20, không phải một lượt sửa lỗi cài đặt** — UX-DR20 nói `⏐`
 * đánh dấu **ranh giới câu**, và ranh giới giữa hai câu rỗng vẫn là một ranh giới. Nên nó cần
 * một chữ ký, và **Ice ký 2026-08-13**. Mệnh đề sau lượt thu hẹp: *một câu chưa có chữ nào thì
 * không có ranh giới nào để mà vẽ.* Mọi câu **có chữ** giữ nguyên hành vi UX-DR20 từng chữ.
 */
.sent:empty::after {
  content: none;
}

.sent:hover::after,
.sent[data-caret]::after {
  /* aura-allow-opacity: `⏐` là NÉT chứ không phải chữ — UX-DR6 cho phép `opacity` nghỉ trên nét và nền. */
  opacity: 0.55;
}
</style>
