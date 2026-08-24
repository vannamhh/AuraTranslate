/**
 * State của dải "Chờ chốt lần đầu gặp" — Story 3.6, FR114.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY (`src/`, không `src/panels/`)
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `glossaryQuickAddState.ts`: dải không phải một panel — nó là một thể hiện ở
 * chân workspace, cùng slot DOM. Nó không đăng ký `FocusOwner`
 * (`commands/index.ts::FOCUS_OWNERS` — dải không thêm vào đó).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO `watch(editorCaretSegmentId, …)` KHÔNG SỐNG TRONG TỆP NÀY
 * ─────────────────────────────────────────────────────────────────────────────
 * `editorPanelState.ts` phải gọi [`resetGlossaryConfirmStrip`] tường minh ở hai chỗ đang gọi
 * `resetGlossaryMarks()` (đổi Chương/Tác phẩm, và gộp/tách segment) — tức chiều phụ thuộc
 * BẮT BUỘC là `editorPanelState.ts` → tệp này, đúng khuôn `resetGlossaryMarks`/
 * `glossaryMarksState.ts`. Nếu tệp này quay lại `import` `editorCaretSegmentId` từ
 * `editorPanelState.ts` để tự đặt watcher, đó là một VÒNG (`editorPanelState.ts:27`: "Không
 * vòng", `glossaryMarksState.ts` §đầu tệp: cùng luật). Giải: mọi hàm ở đây nhận
 * `segmentId`/`segments`/`marks`/`chapterId`/`sourceLang` làm THAM SỐ từ chỗ gọi — cùng khuôn
 * `glossaryMarksState.ts` nhận `chapterId`/`segments`/`sourceLang`. `GlossaryConfirmStrip.vue`
 * (một LEAF, không phải một module lõi) là nơi đặt `watch(editorCaretSegmentId, …)` thật, gọi
 * xuống [`syncGlossaryConfirmStripTarget`] mỗi khi caret/marks đổi.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 SỔ ƯU TIÊN KHÔNG SỐNG Ở ĐÂY
 * ─────────────────────────────────────────────────────────────────────────────
 * Tệp này không biết `GlossaryQuickAdd` có đang mở hay không — nó luôn tính "mục đang chờ
 * chốt của câu hiện tại" một cách độc lập. Việc CÓ HIỆN hay không (quick-add thắng khi cả
 * hai cùng đủ điều kiện) là việc của `GlossaryConfirmStrip.vue` qua
 * `panels/inlineStripPriority.ts::topmostStrip` — hai trách nhiệm tách rời, đúng khuôn state
 * (dữ liệu) / template (hiển thị).
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { confirmPendingGlossaryTranslation } from './config/glossary'
import type { GlossaryMark, GlossaryTierWire, HanVietSuggestionStatus } from './config/glossary'
import type { IpcError } from './i18n'
import { glossaryMarksBySegment } from './panels/glossaryMarksMap'
import type { GlossarySegmentSource } from './panels/glossaryMarksMap'
import { refreshGlossaryMarks } from './panels/glossaryMarksState'

/**
 * Mục đang được hỏi — `tier`+`id` đủ để gọi `confirmPendingGlossaryTranslation`.
 *
 * 🔵 **THÊM 2026-08-24 (Story 3.7, FR113) — `hanVietSuggestion`/`hanVietStatus`.** Chép
 * thẳng từ `SegmentTermSpan` lúc [`applyTarget`] đổi mục — đây là dữ liệu Rust đã cầm sẵn
 * lúc dựng dấu (`marks_for_source_text`), không một vòng IPC thứ hai.
 */
type CurrentTarget = {
  tier: GlossaryTierWire
  id: number
  sourceTerm: string
  hanVietSuggestion: string | null
  hanVietStatus: HanVietSuggestionStatus
}

function targetsEqual(a: CurrentTarget | null, b: CurrentTarget | null): boolean {
  if (a === null || b === null) return a === b
  return a.tier === b.tier && a.id === b.id
}

// ─────────────────────────────────────────────────────────────────────────────
// State module-level — singleton của cả tiến trình, cùng khuôn glossaryQuickAddState.ts.
// ─────────────────────────────────────────────────────────────────────────────

const current = ref<CurrentTarget | null>(null)
const translationInput = ref('')
const saving = ref(false)
const saveError = ref<IpcError | null>(null)
/** Chuỗi gửi lên là rỗng/toàn khoảng trắng — chặn Ở TẦNG DẢI, thông báo qua `t()`, KHÔNG một
 * `IpcError` (không lượt IPC nào được phát cho ca này). */
const emptyInputError = ref(false)

/** `source_term` đã "Để sau" trong Chương ĐANG MỞ — xoá khi đổi Chương (§Design Notes: một
 * mục đã chốt không bao giờ hỏi lại NHỜ CẤU TRÚC `translation IS NOT NULL`, sổ này chỉ tồn
 * tại cho đường `Esc`, nơi không có gì đổi trên đĩa để đọc lại). */
let deferred = new Set<string>()

/**
 * 🔵 THÊM 2026-08-22 (rà ba lớp) — VỆ DANH TÍNH cho lượt ghi ĐANG BAY, cùng khuôn `sequence`
 * của `glossaryMarksState.ts`/`glossaryQuickAddState.ts`.
 *
 * `confirmGlossaryConfirmStrip` chụp `target` rồi `await` một vòng IPC — trong lúc chờ, mục
 * đang hỏi (`current`) có thể đổi (caret rời câu, `deferGlossaryConfirmStrip`, đổi Chương).
 * Không có vệ này, câu trả lời VỀ MUỘN của mục CŨ sẽ dán lỗi lên mục MỚI đang hiện, hoặc giật
 * tiêu điểm khỏi chỗ người dùng vừa bấm sang — đúng lớp đua round-trip IPC mà hai tệp kia đã
 * phải vá. Tăng ở [`applyTarget`] (khi danh tính mục THẬT SỰ đổi), ở
 * [`deferGlossaryConfirmStrip`], và ở [`resetGlossaryConfirmStrip`].
 */
let sequence = 0

/** Đối số của [`syncGlossaryConfirmStripTarget`] — kiểu RIÊNG (không phải `const`/`let`, nên
 * `check:panel-refs` không quét nó), để [`lastSyncArgs`] ở dưới khai được TRÊN MỘT DÒNG. */
type SyncArgs = {
  segmentId: number | null
  segments: readonly GlossarySegmentSource[]
  marksHaveLoaded: boolean
  marks: readonly GlossaryMark[]
}

/** Đối số của lượt `sync` gần nhất — dùng để tính lại mục hiện tại sau một lượt "Để sau"
 * (đổi `deferred` không tự kích một watcher nào ở phía gọi). */
let lastSyncArgs: SyncArgs | null = null

/** Phần tử đang giữ tiêu điểm NGAY TRƯỚC khi vào dải bằng hợp âm — trả lại khi ra (Lưu hoặc
 * Để sau), đúng khuôn `glossaryQuickAddState.ts:219-263`. */
let savedFocusEl: HTMLElement | null = null
/** Vùng chọn NGAY TRƯỚC khi vào dải — trả lại cùng lúc với tiêu điểm. */
let savedRange: Range | null = null
/** Đã vào dải bằng hợp âm chưa — chốt tái nhập, cùng lý do `glossaryQuickAddState.ts`. */
let enteredViaChord = false

export const confirmStripTier = computed<GlossaryTierWire | null>(() => current.value?.tier ?? null)
export const confirmStripSourceTerm = computed<string | null>(() => current.value?.sourceTerm ?? null)
/**
 * 🔵 THÊM 2026-08-24 (Story 3.7, FR113) — trạng thái đề xuất âm Hán Việt của mục đang hỏi,
 * `null` khi không có mục nào. Template đọc trường này để hiện nhãn *âm Hán Việt* (`'ok'`)
 * hoặc dòng *chưa cài dữ liệu từ điển* (`'dict_unavailable'`) — hai trạng thái RIÊNG, đúng
 * tiền lệ `panel.source.han_viet_unknown`/`han_viet_unavailable` của Story 1.16.
 */
export const confirmStripSuggestionStatus = computed<HanVietSuggestionStatus | null>(
  () => current.value?.hanVietStatus ?? null,
)
/** `true` ⇔ có một mục đang chờ hỏi — điều kiện ĐỦ ĐIỀU KIỆN của `'glossary_confirm'` cho
 * `topmostStrip`. Không tự là điều kiện HIỂN THỊ — xem doc-comment đầu tệp. */
export const confirmStripIsOpen = computed<boolean>(() => current.value !== null)
export const confirmStripTranslationInput: Ref<string> = translationInput
export const confirmStripSaving: DeepReadonly<Ref<boolean>> = readonly(saving)
export const confirmStripSaveError: DeepReadonly<Ref<IpcError | null>> = readonly(saveError)
export const confirmStripEmptyInputError: DeepReadonly<Ref<boolean>> = readonly(emptyInputError)

/**
 * 🔴 Tăng ở mỗi lượt [`focusGlossaryConfirmStrip`] — `GlossaryConfirmStrip.vue` `watch` giá
 * trị này để focus ô nhập NGAY khi nó đổi (`nextTick`, cùng khuôn
 * `GlossaryQuickAdd.vue::watch(quickAddIsOpen, …)`). Một số nguyên thay vì một `boolean` vì
 * hai lượt gọi liên tiếp bằng nhau (`focus` → `focus`, đã ở trong dải) vẫn phải đổi giá trị
 * để `watch` bắn — một `boolean` gõ hai lần liên tiếp thành `true`→`true` sẽ KHÔNG bắn watch.
 */
const focusRequest = ref(0)
export const confirmStripFocusRequest: DeepReadonly<Ref<number>> = readonly(focusRequest)

/** Tìm span *chờ chốt* TRÁI NHẤT của segment `segmentId`, bỏ qua mọi `source_term` đã "Để
 * sau" — hàm THUẦN, không ghi state. */
function selectPendingSpan(
  segmentId: number | null,
  segments: readonly GlossarySegmentSource[],
  marksHaveLoaded: boolean,
  marks: readonly GlossaryMark[],
): CurrentTarget | null {
  // "Marks chưa nạp xong ⇒ 0 dải — chưa biết thì không khẳng định 'không có'" (§I/O Matrix).
  if (segmentId === null || !marksHaveLoaded) return null

  const bySegment = glossaryMarksBySegment(segments, marks)
  const spans = bySegment.get(segmentId)?.spans ?? [] // đã sắp theo `start` tăng dần.
  const next = spans.find((s) => !s.isConfirmed && !deferred.has(s.sourceTerm))
  return next === undefined
    ? null
    : {
        tier: next.tier,
        id: next.id,
        sourceTerm: next.sourceTerm,
        hanVietSuggestion: next.hanVietSuggestion,
        hanVietStatus: next.hanVietStatus,
      }
}

/**
 * Áp một mục MỚI — reset ô nhập/lỗi CHỈ KHI danh tính mục đổi (không xoá chữ người dùng
 * đang gõ nếu `sync` được gọi lại cho ĐÚNG mục hiện tại).
 *
 * 🔵 THÊM 2026-08-24 (Story 3.7, FR113) — ô nhập ĐIỀN SẴN bằng `next.hanVietSuggestion` khi
 * có (§I/O Matrix: "Dải chốt mọc ⇒ Ô nhập điền sẵn đề xuất"; đổi sang một mục KHÔNG đề xuất
 * được ⇒ ô RỖNG LẠI, không giữ chữ của mục trước — cùng đúng chỗ mà bản trước đã `= ''`).
 */
function applyTarget(next: CurrentTarget | null): void {
  if (targetsEqual(current.value, next)) return

  // 🔵 SỬA 2026-08-22 (rà ba lớp, vòng 2 — Ice bắt) — danh tính THẬT SỰ đổi: vô hiệu hoá mọi
  // lượt ghi đang bay của mục CŨ ([`sequence`], xem [`confirmGlossaryConfirmStrip`]) VÀ dọn
  // trạng thái "đã vào dải bằng hợp âm" của mục CŨ trong CÙNG một nhánh.
  //
  // Bản trước (rà ba lớp, vòng 1) cố ý ĐỂ NGỎ ba biến này với lý do "giữ phép đối chứng của
  // `sequence` quan sát được qua `document.activeElement`" — lý do đó SAI: nó là một lập luận
  // về PHÉP ĐO, không phải về sản phẩm, và né tránh một lỗi thật đang chạy. Hệ quả đã xảy ra
  // (không phải lý thuyết): vào dải bằng hợp âm cho mục A (lưu `savedFocusEl` = ô câu A) →
  // caret rời câu, `watch` đổi mục sang B → `enteredViaChord`/`savedFocusEl` VẪN mang dữ liệu
  // của A → hợp âm lần nữa cho B rơi vào nhánh "chốt tái nhập" và BỎ QUA bước lưu tiêu điểm
  // mới → lúc Lưu/Để sau cho B, tiêu điểm nhảy về ô của CÂU A — một câu khác hẳn câu đang làm.
  // Test canh kịch bản này: `glossaryConfirmStrip.test.ts` §"chord → đổi mục qua sync → chord
  // lần hai → Lưu".
  sequence += 1
  savedFocusEl = null
  savedRange = null
  enteredViaChord = false

  current.value = next
  translationInput.value = next?.hanVietSuggestion ?? ''
  saveError.value = null
  emptyInputError.value = false
}

/**
 * Tính lại mục đang chờ hỏi cho câu (segment) đang có tiêu điểm — gọi từ `watch` của
 * `GlossaryConfirmStrip.vue` mỗi khi `editorCaretSegmentId`/`editorSegments`/`glossaryMarks`
 * đổi. Hàm ghi state (không thuần) — [`selectPendingSpan`] là nửa thuần bên trong nó.
 */
export function syncGlossaryConfirmStripTarget(
  segmentId: number | null,
  segments: readonly GlossarySegmentSource[],
  marksHaveLoaded: boolean,
  marks: readonly GlossaryMark[],
): void {
  lastSyncArgs = { segmentId, segments, marksHaveLoaded, marks }
  applyTarget(selectPendingSpan(segmentId, segments, marksHaveLoaded, marks))
}

/**
 * Vào dải bằng hợp âm (`Mod+Alt+C`, lệnh `glossary.confirm.focus`) — lưu tiêu điểm/vùng chọn
 * cũ rồi yêu cầu `GlossaryConfirmStrip.vue` focus ô nhập. Không làm gì nếu **0** mục đang
 * chờ hỏi (không có gì để vào).
 *
 * `initialTranslation` — vùng chọn Ở BẤT KỲ bề mặt đã đăng ký nào (kể cả cột bản dịch của
 * lưới, vai `'display'`) TẠI THỜI ĐIỂM bấm hợp âm, do CHỖ GỌI đọc qua
 * `selectionContract.ts::currentSelectionTextForGlossaryQuickAdd` (đường ĐỌC, không phải
 * `import` — cùng khuôn `openGlossaryQuickAdd(initialSourceTerm)`: state không tự đọc DOM
 * vùng chọn, chỗ gọi ở `commands/index.ts`/`main.ts` đọc rồi truyền vào).
 *
 * 🔵 **SỬA 2026-08-24 (Story 3.7) — luật điền tổng quát hoá thành "chỉ điền khi ô CHƯA BỊ
 * NGƯỜI DÙNG SỬA", không còn hẹp thành "chỉ điền khi ô RỖNG".** [`applyTarget`] (Story 3.7)
 * nay có thể điền SẴN một đề xuất Hán Việt vào ô nhập lúc mọc — ô không còn RỖNG theo nghĩa
 * đen ở lượt vào ĐẦU TIÊN. "Chưa bị sửa" = ô đang mang ĐÚNG giá trị mà [`applyTarget`] vừa
 * đặt (rỗng khi không có đề xuất, hoặc đúng chuỗi đề xuất khi có) — so `translationInput.
 * value` với `target.hanVietSuggestion ?? ''` thay vì so với `''` trần. §I/O Matrix: "Vùng
 * chọn THẮNG đề xuất — thao tác người dùng vừa làm đứng trên gợi ý của máy": một mục KHÔNG
 * đề xuất (giá trị "chưa bị sửa" là `''`) suy biến về ĐÚNG luật cũ. Chốt tái nhập (nhánh
 * dưới) không chạm dòng này nên không có nguy cơ ghi đè chữ người dùng đang gõ SAU lượt vào
 * đầu tiên (`enteredViaChord` chặn mọi lượt fill sau đó).
 *
 * `isVisible` — dải chốt có phải dải THẮNG sổ ưu tiên hay không, TẠI THỜI ĐIỂM bấm hợp âm, do
 * CHỖ GỌI tính qua `topmostStrip(...)` (`main.ts`) rồi truyền vào — tệp này cố ý KHÔNG biết
 * `GlossaryQuickAdd` có đang mở hay không (xem doc-comment đầu tệp).
 *
 * 🔵 SỬA 2026-08-22 (rà ba lớp) — THÊM tham số `isVisible` + kiểu trả về `boolean`, cùng khuôn
 * *"hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU"* (`AGENTS.md`). Bản trước chỉ
 * kiểm `current.value === null`, không kiểm dải có ĐANG HIỆN không: khi `GlossaryQuickAdd`
 * thắng sổ ưu tiên (dải chốt bị `v-if` ẩn), hợp âm này vẫn lưu `savedFocusEl`, vẫn bật
 * `enteredViaChord = true`, vẫn tăng `focusRequest` — không ô nào thật sự nhận tiêu điểm (dải
 * ẩn không có `<input>` trong DOM), và KHÔNG một dòng chẩn đoán nào nói ra điều đó. Tệ hơn:
 * `enteredViaChord = true` còn kẹt lại, nên hợp âm LẦN SAU (khi dải chốt đã thắng sổ ưu tiên
 * thật) đi vào nhánh "chốt tái nhập" và bỏ qua bước lưu tiêu điểm — im lặng kép.
 */
export function focusGlossaryConfirmStrip(initialTranslation: string, isVisible: boolean): boolean {
  if (current.value === null) return false
  if (!isVisible) {
    console.error(
      '[glossary] glossary.confirm.focus chay nhung dai cho chot khong phai dai dang HIEN ' +
        '(mot dai khac thang so uu tien topmostStrip) -- bo qua, khong cham state nao',
    )
    return false
  }
  if (enteredViaChord) {
    // Chốt tái nhập, cùng lý do `glossaryQuickAddState.ts::openGlossaryQuickAdd` — hợp âm bắn
    // lần hai trong khi đã ở trong dải không được ghi đè `savedFocusEl`/`savedRange` bằng
    // chính ô nhập CỦA DẢI.
    focusRequest.value += 1
    return true
  }

  const active = document.activeElement
  savedFocusEl = active instanceof HTMLElement ? active : null

  const selection = window.getSelection()
  savedRange =
    selection !== null && selection.rangeCount > 0 ? selection.getRangeAt(0).cloneRange() : null

  const unedited = translationInput.value === (current.value.hanVietSuggestion ?? '')
  if (unedited && initialTranslation !== '') {
    translationInput.value = initialTranslation
  }

  enteredViaChord = true
  focusRequest.value += 1
  return true
}

/**
 * Trả lại tiêu điểm và vùng chọn đã lưu — dùng ở CẢ HAI đường ra (Lưu thành công, Để sau),
 * đúng khuôn `glossaryQuickAddState.ts::restoreFocusAndSelection`.
 */
function restoreFocusAndSelection(): void {
  if (savedFocusEl !== null && savedFocusEl.isConnected) {
    savedFocusEl.focus()
  }
  if (savedRange !== null) {
    const selection = window.getSelection()
    if (selection !== null) {
      try {
        selection.removeAllRanges()
        selection.addRange(savedRange)
      } catch (err) {
        console.warn(`[glossary] khong khoi phuc duoc vung chon cu: ${String(err)}`)
      }
    }
  }
  savedFocusEl = null
  savedRange = null
  enteredViaChord = false
}

/**
 * Để sau (`Esc`, lệnh `glossary.confirm.defer`) — `source_term` hiện tại không hỏi lại trong
 * Chương đang mở; dải thu, tiêu điểm trả về ô gõ cũ. Nếu câu hiện tại còn một mục *chờ chốt*
 * KHÁC, dải kế mọc NGAY (cùng slot, không cần đổi câu) — xem [`selectPendingSpan`].
 *
 * 🔵 THÊM 2026-08-22 (rà ba lớp) — `if (saving.value) return` NGAY ĐẦU HÀM, đúng khuôn
 * `glossaryQuickAddState.ts::closeGlossaryQuickAdd`. `@keydown.esc.prevent` của
 * `GlossaryConfirmStrip.vue` sống NGOÀI `<fieldset :disabled="confirmStripSaving">` (Esc
 * không thuộc luật Kiểm A của `check:commands`, chỉ `@click` mới bị canh), nên đường bàn
 * phím và đường chuột (nút "Để sau" đã `:disabled` trong `<fieldset>`) phải khớp nhau ở TẦNG
 * STATE — không thể khớp ở tầng DOM vì hai luật cổng khác nhau áp cho `@click`/`@keydown`.
 */
export function deferGlossaryConfirmStrip(): void {
  if (saving.value) return

  const target = current.value
  if (target === null) return

  // 🔵 THÊM 2026-08-22 (rà ba lớp) — bump TƯỜNG MINH tại đây, dù [`applyTarget`] ở dưới cũng
  // bump khi danh tính đổi (defer LUÔN đổi danh tính — `deferred` vừa thêm chính `source_term`
  // hiện tại nên [`selectPendingSpan`] không bao giờ chọn lại nó): phòng một lượt sửa
  // `applyTarget` sau này lặng lẽ bỏ điều kiện đó mà không ai để ý đường `defer` phụ thuộc nó.
  sequence += 1
  deferred.add(target.sourceTerm)
  restoreFocusAndSelection()

  if (lastSyncArgs === null) {
    applyTarget(null)
    return
  }
  applyTarget(
    selectPendingSpan(
      lastSyncArgs.segmentId,
      lastSyncArgs.segments,
      lastSyncArgs.marksHaveLoaded,
      lastSyncArgs.marks,
    ),
  )
}

/**
 * Lưu (`Enter`, lệnh `glossary.confirm.save`) — chốt bản dịch cho mục đang hỏi.
 *
 * ⚠️ **Chuỗi rỗng/khoảng trắng chặn TRƯỚC IPC** — `CHECK` phía SQL cũng chặn (`GLOSSARY_
 * ENTRY_DDL`), nhưng người dùng không được phép chạm tới nhánh đó: `0` lượt IPC cho ca này.
 *
 * Thành công ⇒ marks nạp lại (`refreshGlossaryMarks`, KHÔNG chờ — cùng khuôn
 * `glossaryQuickAddState.ts::refreshGlossaryMarksAfterSave`: mục đã ghi XUỐNG ĐĨA THẬT dù
 * lượt làm mới này có trượt), tiêu điểm trả về ô gõ cũ, trả `true`.
 * Trượt ⇒ dải Ở LẠI, lỗi hiện qua [`confirmStripSaveError`], trả `false`.
 *
 * 🔵 THÊM 2026-08-22 (rà ba lớp) — VỆ DANH TÍNH `mySequence`, chụp NGAY TRƯỚC khi phát IPC.
 * Trong lúc `await`, `current` có thể đổi (`watch` bắn caret mới, `deferGlossaryConfirmStrip`)
 * — [`applyTarget`] bump [`sequence`] mỗi lần đó. Câu trả lời VỀ SAU khi danh tính đã lệch:
 * - **Lỗi** ⇒ KHÔNG gán [`confirmStripSaveError`] — lỗi đó thuộc mục CŨ, dán lên mục MỚI đang
 *   hiện là hiện sai chỗ.
 * - **Thành công** ⇒ KHÔNG [`restoreFocusAndSelection`] — tiêu điểm đã thuộc về nơi người dùng
 *   VỪA bấm sang, giật nó về là cướp lại một cách bất ngờ.
 * - **CẢ HAI NHÁNH vẫn `refreshGlossaryMarks`** — bản dịch đã GHI XUỐNG ĐĨA THẬT bất kể mục
 *   đang hiện có đổi hay không; nuốt lượt làm mới vì lệch identity để lại một dấu Glossary CŨ
 *   sai trên lưới, đúng lớp *rỗng im lặng* mà kho cấm.
 */
export async function confirmGlossaryConfirmStrip(
  chapterId: number,
  segments: readonly GlossarySegmentSource[],
  sourceLang: string,
): Promise<boolean> {
  const target = current.value
  if (target === null) return false
  if (saving.value) return false // chốt tái nhập -- cùng lý do `saveGlossaryQuickAdd`.

  const value = translationInput.value.trim()
  if (value === '') {
    emptyInputError.value = true
    return false
  }
  emptyInputError.value = false

  saving.value = true
  saveError.value = null
  const mySequence = sequence

  const result = await confirmPendingGlossaryTranslation(target.tier, target.id, value)
  saving.value = false
  const stillCurrentTarget = mySequence === sequence

  if (result.error !== null) {
    if (stillCurrentTarget) saveError.value = result.error
    return false
  }

  if (stillCurrentTarget) restoreFocusAndSelection()
  void refreshGlossaryMarks(chapterId, segments, sourceLang)
  return true
}

/**
 * 🔴 Vứt toàn bộ state của dải — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`. Nối vào HAI chỗ đang gọi `resetGlossaryMarks()` (`editorPanelState.ts:647,
 * 2021` — đổi Chương/Tác phẩm, và gộp/tách segment): sổ "Để sau" có phạm vi ĐÚNG MỘT Chương,
 * và một dải đang hỏi về một segment vừa về hưu (gộp/tách) không được sống sót.
 */
export function resetGlossaryConfirmStrip(): void {
  sequence += 1 // vô hiệu hoá mọi lượt ghi đang bay -- cùng lý do mọi `reset*()` khác trong kho.
  current.value = null
  translationInput.value = ''
  saving.value = false
  saveError.value = null
  emptyInputError.value = false
  focusRequest.value = 0
  deferred = new Set()
  lastSyncArgs = null
  savedFocusEl = null
  savedRange = null
  enteredViaChord = false
}
