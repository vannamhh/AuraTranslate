/**
 * State của dải "Thêm thuật ngữ" — Story 3.3, FR48.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY (`src/`, không `src/panels/`)
 * ─────────────────────────────────────────────────────────────────────────────
 * Dải không phải một panel — nó là **một thể hiện ở chân workspace**, dùng được từ cả bốn
 * bề mặt vùng chọn cùng lúc (§Design Notes của spec). Nó không đăng ký `FocusOwner`
 * (`commands/index.ts::FOCUS_OWNERS` — sáu mục, dải KHÔNG thêm vào đó) và không sống trong
 * `src/panels/**`, đúng khuôn `StatusBar.vue`/`AttributionOverlay.vue` — những thứ khác
 * cũng là "vỏ ứng dụng" chứ không phải nội thất một chế độ.
 *
 * ⚠️ Tệp này dùng `ref`/`computed` của Vue **và** gọi `@tauri-apps/api` xuyên qua
 * `config/glossary.ts` — KHÔNG được `import` vào `src/commands/index.ts` (Kiểm C/D/E của
 * `npm run check:commands` nạp tệp đó bằng Node thuần). `openGlossaryQuickAdd` được TIÊM
 * VÀO qua `CommandDeps` + lệnh `glossary.add_term`, nối thật ở `src/main.ts` — cùng cửa mà
 * `runLookup`/`selectSourceTab` đã đi qua.
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import {
  addGlossaryTerm,
  lookupGlossaryTerm,
  updateGlossaryTerm,
} from './config/glossary'
import type { GlossaryCategory, GlossaryLookupResult, GlossaryTierWire } from './config/glossary'
import type { IpcError } from './i18n'

/**
 * Chế độ của dải — **hàm THUẦN của `(source_term, lookup)`**, không một cờ đặt lúc mở
 * (§Design Notes của spec, nguyên văn):
 *
 * ```text
 * mode(source_term, lookup) =
 *   lookup == null                  -> 'unknown' (đang chờ IPC — KHÔNG nói "chưa có")
 *   lookup == Found(tier, entry)    -> 'edit',  tầng GHIM = tier
 *   lookup == NotFound              -> 'add', tầng tự chọn
 * ```
 *
 * `source_term` không xuất hiện trực tiếp trong chữ ký — nó đã được RÚT GỌN vào `lookup`
 * trước khi hàm này chạy (chỗ gọi luôn tra lại `lookup` mỗi khi `source_term` đổi, xem
 * [`scheduleQuickAddLookup`]), nên tại BẤT KỲ thời điểm nào, `lookup` là kết quả tra của
 * ĐÚNG `source_term` hiện tại hoặc `null`/`'unknown'` trong lúc chờ — không bao giờ kết quả
 * của một `source_term` cũ.
 */
export type QuickAddMode = 'unknown' | 'add' | 'edit'

/** Hàm thuần — xem doc-comment của [`QuickAddMode`]. Test được không cần Vue/DOM. */
export function quickAddModeFor(lookup: GlossaryLookupResult | null): QuickAddMode {
  if (lookup === null) return 'unknown'
  if (lookup.found === 'unknown') return 'unknown'
  if (lookup.found === 'entry') return 'edit'
  return 'add'
}

/**
 * 🔴 Vị từ *"…HasLoaded"* — Known pitfall trung tâm của dự án (`AGENTS.md`): một danh sách/
 * kết quả rỗng không tự nói vì sao nó rỗng. **BA trạng thái**, không một `boolean`:
 * - `lookup === null` ⇒ chưa từng tra (dải vừa mở, chưa gọi lượt nào);
 * - `lookup.found === 'unknown'` ⇒ đang bay HOẶC lượt tra gần nhất trượt;
 * - còn lại (`'none'`/`'entry'`) ⇒ ĐÃ tra xong và có câu trả lời dùng được.
 *
 * Chỉ trạng thái thứ ba mới cho phép dải khẳng định *"cụm này chưa có trong Glossary"* —
 * hai trạng thái đầu phải hiện *"đang kiểm tra…"*, không phải im lặng rơi vào chế độ THÊM.
 */
export function quickAddLookupHasLoaded(lookup: GlossaryLookupResult | null): boolean {
  return lookup !== null && lookup.found !== 'unknown'
}

// ─────────────────────────────────────────────────────────────────────────────
// State module-level — singleton của cả tiến trình, cùng khuôn lookupPanelState.ts.
// ─────────────────────────────────────────────────────────────────────────────

const isOpen = ref(false)
const sourceTerm = ref('')
const translation = ref('')
const note = ref('')
const category = ref<GlossaryCategory>('person')
/** Tầng NGƯỜI DÙNG chọn — chỉ có hiệu lực ở chế độ THÊM. Chế độ SỬA GHIM tầng theo `lookup`. */
const tierChoice = ref<GlossaryTierWire>('global')

const lookup = ref<GlossaryLookupResult | null>(null)
const saving = ref(false)
const saveError = ref<IpcError | null>(null)

/** Số thứ tự lượt tra — cùng khuôn `lookupPanelState.ts::sequence`, chặn một lượt CHẬM ghi
 * đè lên một lượt SAU nó đã về nhanh hơn (đua round-trip IPC). */
let sequence = 0

/** Phần tử đang giữ tiêu điểm NGAY TRƯỚC khi dải mở — trả lại khi đóng (Esc hoặc sau Lưu). */
let savedFocusEl: HTMLElement | null = null
/** Vùng chọn NGAY TRƯỚC khi dải mở — trả lại khi đóng, cùng lúc với tiêu điểm. */
let savedRange: Range | null = null

export const quickAddIsOpen: DeepReadonly<Ref<boolean>> = readonly(isOpen)
/** ⚠️ CHỈ ĐỌC — mọi lượt ghi phải qua [`setQuickAddSourceTerm`], không gán thẳng `.value`:
 * đổi ô nguồn mà không phát lại lượt tra là làm `lookup`/`mode` lệch khỏi `source_term`
 * hiện tại, đúng bất biến mà doc-comment của [`QuickAddMode`] mô tả. */
export const quickAddSourceTerm: DeepReadonly<Ref<string>> = readonly(sourceTerm)
export const quickAddTranslation: Ref<string> = translation
export const quickAddNote: Ref<string> = note
export const quickAddCategory: Ref<GlossaryCategory> = category
export const quickAddTierChoice: Ref<GlossaryTierWire> = tierChoice
export const quickAddLookup: DeepReadonly<Ref<GlossaryLookupResult | null>> = readonly(lookup)
export const quickAddSaving: DeepReadonly<Ref<boolean>> = readonly(saving)
export const quickAddSaveError: DeepReadonly<Ref<IpcError | null>> = readonly(saveError)

/** Chế độ ĐANG SỐNG — computed trên [`quickAddModeFor`], luôn khớp `lookup` hiện tại. */
export const quickAddMode = computed<QuickAddMode>(() => quickAddModeFor(lookup.value))

/** Xem [`quickAddLookupHasLoaded`]. */
export const quickAddHasLoaded = computed(() => quickAddLookupHasLoaded(lookup.value))

/**
 * Tầng đang GHIM trên màn hình — [`GlossaryTier`] của mục tìm thấy ở chế độ SỬA, hoặc lựa
 * chọn của người dùng ở chế độ THÊM/`'unknown'`.
 */
export const quickAddEffectiveTier = computed<GlossaryTierWire>(() => {
  const l = lookup.value
  if (l !== null && l.found === 'entry') return l.entry.tier
  return tierChoice.value
})

/** Tầng Tác phẩm dùng được cho lượt tra HIỆN TẠI, hay `null` nếu chưa biết (chưa tra xong). */
export const quickAddWorkTierAvailable = computed<boolean | null>(() => {
  const l = lookup.value
  if (l === null || l.found === 'unknown') return null
  return l.workTierAvailable
})

/**
 * 🔵 THÊM 2026-08-20 (lượt rà soát Story 3.3) — lỗi của LƯỢT TRA, tách hẳn khỏi
 * [`quickAddSaveError`] (lỗi của lượt GHI).
 *
 * Trước hằng này, `GlossaryLookupResult` mang một `IpcError` ở biến thể `'unknown'`
 * (`config/glossary.ts`), tầng này cất nó vào `lookup`, và KHÔNG chỗ nào đọc lại: một lượt
 * tra trượt (`global.db` vắng mặt ⇒ `store.open_failed`, hay `apply_override` từ chối ⇒
 * `glossary.scope_error`) làm `quickAddHasLoaded` đứng ở `false` vĩnh viễn, nên dải hiện
 * *"đang kiểm tra…"* mãi mãi và nút Lưu tắt mãi mãi — một trạng thái chờ không bao giờ kết
 * thúc, không một lý do nào trên màn hình. Đó đúng là lớp *rỗng im lặng* mà `AGENTS.md`
 * liệt vào Known pitfalls trung tâm và chính story này viện dẫn ba lần.
 *
 * ⚠️ `null` ở đây có HAI nghĩa và cả hai đều KHÔNG phải "có lỗi": lượt tra đang bay/chưa
 * chạy (`lookup === null`), hoặc lượt tra đã về sạch (`'none'`/`'entry'`). Biến thể
 * `'unknown'` với `error: null` cũng trả `null` — đó là ca *chạy ngoài Tauri*, mà
 * `config/glossary.ts` cố ý phân biệt với một lỗi thật.
 */
export const quickAddLookupError = computed<IpcError | null>(() => {
  const l = lookup.value
  if (l === null || l.found !== 'unknown') return null
  return l.error
})

/**
 * Phát một lượt tra MỚI cho `term` — huỷ hiệu lực mọi lượt đang bay cũ hơn bằng
 * `sequence`, cùng khuôn `lookupPanelState.ts::runLookup`.
 *
 * ⚠️ **Luôn gọi IPC, kể cả với chuỗi rỗng** — và đó không phải một chỗ tối ưu bỏ lỡ. Một
 * đường tắt "chuỗi rỗng thì khỏi tra" trông hợp lý nhưng làm `work_tier_available` không
 * bao giờ được biết cho tới khi người dùng gõ ký tự ĐẦU TIÊN — tức dải mở với ô nguồn rỗng
 * (AC "không có vùng chọn") sẽ hiện sai lý do "chưa mở Tác phẩm" ngay cả khi một Tác phẩm
 * đang mở thật. `resolve_term_for_quick_add` phía Rust xử lý chuỗi rỗng vô hại (`trim()`
 * rồi tra một khoá rỗng, không khớp gì), nên chi phí của một lượt gọi thừa nhỏ hơn hẳn chi
 * phí của một lý do SAI hiện trên màn hình.
 *
 * ⚠️ **Không debounce.** Đây không phải đường nóng NFR1/NFR2 (gõ trong ô nguồn của một
 * dải, không phải kéo chọn hay gõ dịch) — mỗi lượt đổi ô nguồn phát một lượt IPC, và
 * `sequence` đảm bảo chỉ câu trả lời MỚI NHẤT được ghi vào state.
 */
function scheduleQuickAddLookup(term: string): void {
  sequence += 1
  const mySequence = sequence

  lookup.value = null // 'unknown' cấu trúc — xem doc-comment `QuickAddMode`.
  void lookupGlossaryTerm(term).then((result) => {
    if (mySequence !== sequence) return // bị một lượt SAU vượt mặt — bỏ, không ghi đè.
    lookup.value = result
    if (result.found === 'entry') {
      translation.value = result.entry.translation ?? ''
      note.value = result.entry.note
      category.value = result.entry.category
    }
  })
}

/**
 * Ghi một `source_term` MỚI vào ô nguồn — gọi từ `GlossaryQuickAdd.vue` mỗi lượt người
 * dùng gõ (AC: *"đổi ô nguồn sau khi dải đã mở ⇒ dải tự chuyển chế độ"*).
 */
export function setQuickAddSourceTerm(term: string): void {
  sourceTerm.value = term
  scheduleQuickAddLookup(term)
}

/**
 * Mở dải — điểm vào DUY NHẤT, gọi từ handler của lệnh `glossary.add_term`
 * (`src/commands/index.ts` → `src/main.ts`).
 *
 * `initialSourceTerm` đến từ đường ĐỌC RIÊNG của lệnh
 * (`selectionContract.ts::currentSelectionTextForGlossaryQuickAdd`) — KHÔNG qua
 * `currentSelectionText()` (lọc vai `source`, trả rỗng ở ba trong bốn bề mặt FR48).
 */
export function openGlossaryQuickAdd(initialSourceTerm: string): void {
  // 🔴 CHỐT TÁI NHẬP — Ice bắt 2026-08-20: `glossary.add_term` tới được bằng `Mod+Alt+G`
  // TỪ BẤT KỲ bề mặt nào, kể cả khi dải đang mở (hợp âm không tự tắt vì dải không nuốt
  // bàn phím — nó không phải một `KeymapGate`). Không có dòng này, lượt mở THỨ HAI ghi đè
  // `savedFocusEl`/`savedRange` bằng chính ô gõ CỦA DẢI (tiêu điểm lúc đó đã ở trong dải),
  // nên `Esc` sau đó trả tiêu điểm về một phần tử của chính dải vừa ẩn — đúng lớp lỗi mà
  // AC "focus và vùng chọn cũ được trả lại" tồn tại để chặn. Mở lại khi đang mở là VÔ HẠI
  // (bỏ qua), không phải một lỗi cần báo — cùng khuôn `pin_entry`/`unpin_entry` khi thao
  // tác không áp dụng.
  if (isOpen.value) return

  const active = document.activeElement
  savedFocusEl = active instanceof HTMLElement ? active : null

  const selection = window.getSelection()
  savedRange =
    selection !== null && selection.rangeCount > 0 ? selection.getRangeAt(0).cloneRange() : null

  isOpen.value = true
  translation.value = ''
  note.value = ''
  category.value = 'person'
  tierChoice.value = 'global'
  saveError.value = null
  saving.value = false

  sourceTerm.value = initialSourceTerm
  scheduleQuickAddLookup(initialSourceTerm)
}

/**
 * Trả lại tiêu điểm và vùng chọn đã lưu lúc mở — dùng ở CẢ HAI đường đóng (`Esc` và sau khi
 * Lưu thành công), đúng AC *"màn hình không đổi"*.
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
        // `Range` đã lưu có thể trỏ vào một node đã rời DOM (Chương đổi trong lúc dải mở) —
        // không im lặng, nhưng cũng không ném: tiêu điểm đã được trả ở nhánh trên, đây chỉ
        // là phần "đẹp" của trải nghiệm.
        console.warn(`[glossary] khong khoi phuc duoc vung chon cu: ${String(err)}`)
      }
    }
  }
  savedFocusEl = null
  savedRange = null
}

/**
 * Đóng dải bằng `Esc` — huỷ, không lưu gì. AC: *"màn hình không đổi"*.
 *
 * 🔴 CHỐT TÁI NHẬP — Ice bắt 2026-08-20: KHÔNG đóng trong lúc một lượt ghi đang bay
 * (`saving.value`). Trước dòng này, `Esc` giữa lúc `saveGlossaryQuickAdd` đang chờ IPC
 * đóng dải NGAY LẬP TỨC — màn hình nói "đã huỷ" trong khi lượt ghi vẫn tiếp tục hạ xuống
 * đĩa và về sau vẫn có thể thành công. Đúng lớp lỗi "màn hình nói một đằng, đĩa một nẻo"
 * mà `AGENTS.md` gọi là trung tâm của dự án. `saveGlossaryQuickAdd` tự đóng dải SAU KHI
 * lượt ghi xong (thành công) hoặc để dải mở kèm lỗi (thất bại) — không có ca nào cần
 * `Esc` phải thắng một lượt ghi đang bay.
 */
export function closeGlossaryQuickAdd(): void {
  if (saving.value) return

  sequence += 1 // vô hiệu hoá lượt tra đang bay, cùng lý do `resetLookupPanel`.
  isOpen.value = false
  restoreFocusAndSelection()
}

/**
 * Lưu — Thêm hoặc Sửa tuỳ [`quickAddMode`]. Đóng dải và trả tiêu điểm/vùng chọn khi thành
 * công; ở lại mở và hiện lỗi qua [`quickAddSaveError`] khi trượt (AC: lỗi phải đọc được,
 * không rỗng im lặng).
 */
export async function saveGlossaryQuickAdd(): Promise<boolean> {
  // 🔴 CHỐT TÁI NHẬP — Ice bắt 2026-08-20: `saving.value` được ĐẶT ở dưới nhưng KHÔNG hàm
  // nào đọc nó TRƯỚC khi vào. `glossary.save_term` tới được bằng cả `@submit` (Enter) LẪN
  // nút bấm; nút đã `:disabled` khi `quickAddSaving`, nhưng phím `↵` không đi qua thuộc
  // tính đó. Hai lượt Enter liên tiếp (hoặc Enter + bấm chuột gần như cùng lúc) phát HAI
  // lượt `addGlossaryTerm`/`updateGlossaryTerm` cho một thao tác người dùng — một cặp có
  // thể cùng thắng `UNIQUE INDEX` theo hai cách khác nhau tuỳ thứ tự về của round-trip IPC.
  if (saving.value) return false

  const mode = quickAddMode.value
  if (mode === 'unknown') return false // lượt tra chưa xong — chưa biết THÊM hay SỬA.

  saving.value = true
  saveError.value = null

  const tier = quickAddEffectiveTier.value
  const translationValue = translation.value.trim() === '' ? null : translation.value

  if (mode === 'add') {
    const result = await addGlossaryTerm(tier, sourceTerm.value, translationValue, note.value, category.value)
    saving.value = false
    if (result.error !== null) {
      saveError.value = result.error
      return false
    }
    closeGlossaryQuickAdd()
    return true
  }

  // mode === 'edit'
  const current = lookup.value
  if (current === null || current.found !== 'entry') return false // không nên xảy ra.
  const result = await updateGlossaryTerm(tier, current.entry.id, translationValue, note.value, category.value)
  saving.value = false
  if (result.error !== null) {
    saveError.value = result.error
    return false
  }
  closeGlossaryQuickAdd()
  return true
}

/**
 * 🔴 Vứt toàn bộ state của dải — `check:panel-refs` đòi mọi ô nhớ cấp module trong cây
 * nguồn TypeScript của `src/` có một đường `reset*()`. Không chỗ gọi sản phẩm nào cần nó
 * hôm nay (dải không thuộc về một Tác phẩm cụ thể — nó tự đóng và tự xoá state ở mỗi lượt
 * `Esc`/Lưu), nhưng cổng đó áp cho MỌI ô nhớ module-level, không chỉ những ô có chỗ gọi.
 */
export function resetGlossaryQuickAdd(): void {
  sequence += 1
  isOpen.value = false
  sourceTerm.value = ''
  translation.value = ''
  note.value = ''
  category.value = 'person'
  tierChoice.value = 'global'
  lookup.value = null
  saving.value = false
  saveError.value = null
  savedFocusEl = null
  savedRange = null
}
