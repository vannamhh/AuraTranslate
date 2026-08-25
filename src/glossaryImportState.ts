/**
 * State của lớp phủ **Xem trước lượt nhập Glossary** — Story 3.10b, AD-48.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KẾ HOẠCH Ở LẠI RUST — TỆP NÀY CHỈ GIỮ MỘT MÔ HÌNH ĐÃ KIỂM
 * ─────────────────────────────────────────────────────────────────────────────
 * AD-48 §Rule ①: nội dung tệp KHÔNG BAO GIỜ đi ra webview. `openGlossaryImportPreview()`
 * trả về `GlossaryImportPreview` — tên tệp, số hàng, cột nhận ra/bỏ qua, đếm mới/giống,
 * và danh sách hàng BẤT ĐỒNG (cả hai bản dịch) — không một byte nội dung tệp thô nào.
 * `confirmGlossaryImport()` chỉ gửi lại BẢN ĐỒ QUYẾT ĐỊNH (`source_term` → giữ của tôi /
 * lấy của file), không gửi lại kế hoạch — Rust tự đối chiếu với lô nó đang giữ.
 *
 * ⚠️ Cùng lý do `glossaryManageState.ts`: dùng `ref`/`computed` và gọi `@tauri-apps/api`
 * xuyên qua `config/glossary.ts` — KHÔNG được `import` vào `src/commands/index.ts`. Hai
 * handler (`glossary.import.confirm`/`…cancel`) được TIÊM VÀO qua `CommandDeps`, nối thật
 * ở `src/main.ts`.
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { cancelGlossaryImport, confirmGlossaryImport, openGlossaryImportPreview } from './config/glossary'
import type { GlossaryConflictDecision, GlossaryImportPreview, GlossaryTierWire } from './config/glossary'
import type { IpcError } from './i18n'

/**
 * 🔴 Vị từ *"…HasLoaded"* — BỐN trạng thái, cùng khuôn `GlossaryManageStatus`:
 * - `'unknown'` — chưa mở hộp thoại lần nào kể từ lúc lớp phủ này được dựng, hoặc một
 *   lượt mở đang BAY;
 * - `'ipc_unavailable'` — cầu IPC vắng (chạy ngoài Tauri) — KHÔNG một lỗi;
 * - `'error'` — lượt mở/đọc/phân tích trượt THẬT (một `IpcError` đọc được);
 * - `'loaded'` — đã có một `preview` (không rỗng, KHÔNG bao gồm ca huỷ hộp thoại — huỷ
 *   không mở lớp phủ này, xem `openGlossaryImportPreviewOverlay`).
 */
export type GlossaryImportStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'loaded'

const overlayOpen = ref(false)
const status = ref<GlossaryImportStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const preview = ref<GlossaryImportPreview | null>(null)
/** `source_term` → quyết định — CHỈ mang khoá cho hàng người dùng đổi ý sang "lấy của
 * file". Vắng mặt == giữ của tôi (mặc định, §Always). */
const decisions = ref<Record<string, GlossaryConflictDecision>>({})
const confirming = ref(false)
const confirmError = ref<IpcError | null>(null)
/** 🔵 THÊM (P9, vòng rà ba lớp 2026-08-25) — chặn bấm chồng: `exportGlossaryManageTier`
 * đã có `if (exportBusy.value) return` (`glossaryManageState.ts`), mở lượt nhập thì
 * KHÔNG — bấm nhanh hai lần "Nhập CSV" xếp chồng HAI hộp thoại hệ điều hành. */
const opening = ref(false)

/** Số thứ tự lượt mở — chặn một lượt CŨ ghi đè state của một lượt MỚI hơn, cùng khuôn mọi
 * state Glossary khác. */
let sequence = 0

export const importOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const importOpening: DeepReadonly<Ref<boolean>> = readonly(opening)
export const importStatus: DeepReadonly<Ref<GlossaryImportStatus>> = readonly(status)
export const importLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const importPreview: DeepReadonly<Ref<GlossaryImportPreview | null>> = readonly(preview)
export const importConfirming: DeepReadonly<Ref<boolean>> = readonly(confirming)
export const importConfirmError: DeepReadonly<Ref<IpcError | null>> = readonly(confirmError)

/**
 * Quyết định HIỆN TẠI của một hàng bất đồng — `'keep_mine'` khi chưa có trong `decisions`
 * (mặc định, §Always). Template đọc qua hàm này thay vì `decisions.value[term]` trực tiếp
 * để không phải lặp lại chỗ mặc định ở nhiều nơi.
 */
export function importDecisionFor(sourceTerm: string): GlossaryConflictDecision {
  return decisions.value[sourceTerm] ?? 'keep_mine'
}

/** Đổi quyết định của một hàng bất đồng — handler `@change` của radiogroup (KHÔNG
 * `dispatch`, xem §Design Notes của spec: `dispatch` không nhận tham số). */
export function setImportDecision(sourceTerm: string, decision: GlossaryConflictDecision): void {
  if (decision === 'keep_mine') {
    // Giu ban do QUYET DINH toi thieu -- chi mang khoa cho hang THAT SU doi y.
    const next = { ...decisions.value }
    delete next[sourceTerm]
    decisions.value = next
    return
  }
  decisions.value = { ...decisions.value, [sourceTerm]: decision }
}

/**
 * BỐN ca rỗng phân biệt được (§Always của spec: "rỗng phải nói vì sao nó rỗng") — áp cho
 * DANH SÁCH BẤT ĐỒNG (`preview.conflicts`), khác `manageEmptyReasonFor` (áp cho toàn bộ
 * danh sách mục Glossary): ca thứ ba/tư ở đây phân biệt *"tệp không có hàng nào"* (I/O
 * Matrix: 0 hàng dữ liệu, không lỗi) với *"không mục nào bất đồng"* (mọi hàng đều mới/
 * giống hệt — cũng không lỗi, một câu chuyện khác).
 */
export function importEmptyReasonFor(
  s: GlossaryImportStatus,
  totalRows: number,
  conflictCount: number,
): 'not_loaded' | 'ipc_unavailable' | 'file_has_no_rows' | 'no_conflicts' | null {
  if (s === 'unknown') return 'not_loaded'
  if (s === 'ipc_unavailable') return 'ipc_unavailable'
  if (s !== 'loaded') return null // 'error' hien qua nhanh rieng (tError), khong qua day.
  if (totalRows === 0) return 'file_has_no_rows'
  if (conflictCount === 0) return 'no_conflicts'
  return null
}

/** Tóm tắt "Nhập N mục mới · giữ M mục đang có" — N = mục mới CỘNG mục bất đồng chọn lấy
 * của file; M = mục bất đồng CÒN LẠI ở giữ của tôi (mục định). Mục *giống hệt* không đếm
 * ở đây — chúng không ghi gì dù chọn gì (§I/O Matrix). */
export const importConfirmSummary = computed<{ newCount: number; keepCount: number }>(() => {
  const p = preview.value
  if (p === null) return { newCount: 0, keepCount: 0 }
  const takeTheirsCount = Object.values(decisions.value).filter((d) => d === 'take_theirs').length
  return { newCount: p.new_count + takeTheirsCount, keepCount: p.conflicts.length - takeTheirsCount }
})

/**
 * Mở hộp thoại CHỌN rồi đọc/xem-trước — điểm vào DUY NHẤT, gọi từ handler tiêm của
 * `glossary.manage.import_csv` (`main.ts`, đọc `manageExchangeTier` để biết tầng).
 *
 * 🔴 **Huỷ hộp thoại KHÔNG mở lớp phủ này** — `outcome: 'cancelled'` là ca "người dùng đổi
 * ý", và mở một lớp phủ trống rồi phải đóng lại là một bước thừa cho họ. MỌI nhánh khác
 * (`'ipc_unavailable'`/`'error'`/`'loaded'`) đều mở lớp phủ — kể cả khi không có gì để vẽ,
 * vì cả ba đều là chuyện PHẢI nói ra (P2, vòng rà ba lớp 2026-08-25).
 */
export async function openGlossaryImportPreviewOverlay(tier: GlossaryTierWire): Promise<void> {
  if (opening.value) return // P9 -- chan bam chong, cung khuon exportGlossaryManageTier.

  opening.value = true
  sequence += 1
  const mySequence = sequence

  const result = await openGlossaryImportPreview(tier)
  opening.value = false
  if (mySequence !== sequence) return

  // 🔴 P2 (vòng rà ba lớp 2026-08-25) — `outcome` PHÂN BIỆT `'cancelled'` (huỷ hộp thoại,
  // im lặng CÓ CHỦ, §Always) khỏi `'ipc_unavailable'` (không có cầu IPC — PHẢI mở lớp phủ
  // và NÓI RA, không còn rơi vào cùng nhánh sớm với "huỷ"). Bản trước gộp cả hai lại vì
  // adapter trả về `{ preview: null, error: null }` cho CẢ hai ca — chạy ngoài Tauri, bấm
  // "Nhập CSV" không xảy ra gì và không nói gì, đúng lớp "rỗng im lặng" mà `AGENTS.md` gọi
  // là lỗi trung tâm của kho.
  if (result.outcome === 'cancelled') return // Huy hop thoai -- im lang, khong mo lop phu.

  overlayOpen.value = true
  decisions.value = {}
  confirming.value = false
  confirmError.value = null

  if (result.outcome === 'ipc_unavailable') {
    status.value = 'ipc_unavailable'
    loadError.value = null
    return
  }
  if (result.outcome === 'error') {
    status.value = 'error'
    loadError.value = result.error
    return
  }

  preview.value = result.preview
  status.value = 'loaded'
  loadError.value = null
}

/** Đóng lớp phủ mà KHÔNG huỷ lô ở Rust — dùng nội bộ, sau một lượt xác nhận THÀNH CÔNG
 * (lô đã tự dọn phía Rust). */
function closeWithoutCancelling(): void {
  overlayOpen.value = false
}

/**
 * Xác nhận lượt nhập — lệnh `glossary.import.confirm`. Thành công ⇒ đóng lớp phủ (lô đã
 * dọn phía Rust). Trượt ⇒ hiện lỗi, lớp phủ Ở LẠI MỞ (lô GIỮ LẠI phía Rust để thử lại,
 * §I/O Matrix).
 */
export async function confirmGlossaryImportPreview(): Promise<void> {
  if (confirming.value || preview.value === null) return

  confirming.value = true
  confirmError.value = null
  const mySequence = sequence

  const result = await confirmGlossaryImport(decisions.value)
  if (mySequence !== sequence) return

  confirming.value = false
  if (result.error !== null) {
    confirmError.value = result.error
    return
  }

  closeWithoutCancelling()
}

/**
 * Huỷ lô đang treo và đóng lớp phủ — lệnh `glossary.import.cancel`. **0** lượt ghi, không
 * lỗi (§I/O Matrix "Nhập, huỷ ở màn hình xem trước").
 */
export async function cancelGlossaryImportPreview(): Promise<void> {
  if (confirming.value) return

  const mySequence = sequence
  overlayOpen.value = false // Dong NGAY -- khong cho nguoi dung cho lot goi huy tra ve.
  void cancelGlossaryImport().then(() => {
    if (mySequence !== sequence) return
  })
}

/**
 * 🔴 Vứt toàn bộ state của lớp phủ — `check:panel-refs` đòi mọi ô nhớ cấp module có một
 * đường `reset*()`. **0 chỗ gọi sản phẩm hôm nay** — lớp phủ là một MODAL, và mỗi lượt mở
 * lại (`openGlossaryImportPreviewOverlay`) đã tự nạp lại state cần thiết.
 */
export function resetGlossaryImport(): void {
  sequence += 1
  overlayOpen.value = false
  status.value = 'unknown'
  loadError.value = null
  preview.value = null
  decisions.value = {}
  confirming.value = false
  confirmError.value = null
  opening.value = false
}
