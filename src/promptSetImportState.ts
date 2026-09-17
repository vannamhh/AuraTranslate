/**
 * State của lớp phủ **Xem trước lượt nhập bộ prompt** — Story 4.5, FR79/NFR9, AD-48. Khuôn
 * chép `glossaryImportState.ts` (Story 3.10b), rút gọn cho MỘT bộ thay vì N hàng, và với một
 * khác biệt có chủ: TẦNG được chọn Ở CHÍNH màn hình xem trước này, sau khi tệp đã đọc — I/O
 * Matrix spec 4.5 "Import picks the tier: User chooses Global or Work at preview" (Glossary
 * chọn tầng TRƯỚC khi mở hộp thoại).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KẾ HOẠCH Ở LẠI RUST — TỆP NÀY CHỈ GIỮ MỘT MÔ HÌNH ĐÃ KIỂM
 * ─────────────────────────────────────────────────────────────────────────────
 * AD-48 §Rule ①: nội dung tệp KHÔNG BAO GIỜ đi ra webview nguyên văn — `promptSetOpenImportPreview()`
 * trả `name`/`body` đã PHÂN TÍCH (mô hình đã kiểm), không byte thô. `promptSetConfirmImport()`
 * chỉ gửi `tier`/`decision` — Rust tự đối chiếu với lô nó đang giữ.
 *
 * ⚠️ Cùng lý do `glossaryImportState.ts`: dùng `ref`/`computed` của Vue và gọi
 * `@tauri-apps/api` xuyên qua `config/promptset.ts` — KHÔNG được `import` vào
 * `src/commands/index.ts`. Ba handler (`prompt.import.{open,confirm,cancel}`) được TIÊM VÀO
 * qua `CommandDeps`, nối thật ở `src/main.ts`.
 *
 * ⚠️ Dùng CHUNG cửa loại trừ hộp thoại với Glossary (`glossaryExchangeGate.ts`) — cả hai
 * domain đều mở một hộp thoại HỆ ĐIỀU HÀNH của Rust, và ứng dụng không có mô hình nào cho
 * "hai hộp thoại chọn tệp cùng mở" — xem doc-comment đã cập nhật của tệp đó.
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import {
  promptSetCancelImport as cancelPromptImport,
  promptSetConfirmImport as confirmPromptImport,
  promptSetOpenImportPreview as openPromptImportPreview,
} from './config/promptset'
import type { PromptSetConflictDecision, PromptSetImportPreview, PromptSetTier } from './config/promptset'
import { glossaryExchangeBusy, resetGlossaryExchangeGate, setGlossaryExchangeBusy } from './glossaryExchangeGate'
import type { IpcError } from './i18n'

/** Bốn trạng thái, cùng khuôn `GlossaryImportStatus`. */
export type PromptImportStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'loaded'

const overlayOpen = ref(false)
const status = ref<PromptImportStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const preview = ref<PromptSetImportPreview | null>(null)
/** Tầng người dùng chọn Ở MÀN XEM TRƯỚC — mặc định `'work'` khi có, ngược lại `'global'`
 * (đặt lại mỗi lượt mở mới, xem [`openPromptImportPreviewOverlay`]). */
const selectedTier = ref<PromptSetTier>('global')
/** Quyết định va chạm hiện tại — MỘT bộ, MỘT quyết định (khác Glossary, nơi đó là một bản đồ
 * theo `source_term`). `'keep_mine'` mặc định (§Always). */
const decision = ref<PromptSetConflictDecision>('keep_mine')
const confirming = ref(false)
const confirmError = ref<IpcError | null>(null)
const confirmedOutcome = ref<'inserted' | 'updated' | 'skipped' | null>(null)
/** Chặn bấm chồng — cùng lý do `importOpening` của Glossary. */
const opening = ref(false)

let sequence = 0

export const promptImportOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const promptImportOpening: DeepReadonly<Ref<boolean>> = readonly(opening)
export const promptImportStatus: DeepReadonly<Ref<PromptImportStatus>> = readonly(status)
export const promptImportLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const promptImportPreview: DeepReadonly<Ref<PromptSetImportPreview | null>> = readonly(preview)
export const promptImportSelectedTier: DeepReadonly<Ref<PromptSetTier>> = readonly(selectedTier)
export const promptImportDecision: DeepReadonly<Ref<PromptSetConflictDecision>> = readonly(decision)
export const promptImportConfirming: DeepReadonly<Ref<boolean>> = readonly(confirming)
export const promptImportConfirmError: DeepReadonly<Ref<IpcError | null>> = readonly(confirmError)
export const promptImportConfirmedOutcome: DeepReadonly<Ref<'inserted' | 'updated' | 'skipped' | null>> =
  readonly(confirmedOutcome)

/** Phân loại của TẦNG ĐANG CHỌN — `null` khi chưa có preview, hoặc khi đã chọn `'work'` mà
 * preview không mang phân loại Tác phẩm (không có Tác phẩm nào đang mở). */
export const promptImportSelectedTierPreview = computed(() => {
  const p = preview.value
  if (p === null) return null
  return selectedTier.value === 'work' ? p.work : p.global
})

export function setPromptImportTier(tier: PromptSetTier): void {
  selectedTier.value = tier
}

export function setPromptImportDecision(d: PromptSetConflictDecision): void {
  decision.value = d
}

/**
 * Mở hộp thoại CHỌN rồi đọc/xem-trước — điểm vào DUY NHẤT, gọi từ handler tiêm của
 * `prompt.import.open` (`main.ts`). **KHÔNG nhận tham số** — tầng chưa cần biết ở đây.
 *
 * Cùng luật `openGlossaryImportPreviewOverlay`: huỷ hộp thoại KHÔNG mở lớp phủ này; mọi
 * nhánh khác (`ipc_unavailable`/`error`/`loaded`) đều mở, kể cả khi không có gì để vẽ.
 */
export async function openPromptImportPreviewOverlay(): Promise<void> {
  if (opening.value || glossaryExchangeBusy.value) return

  opening.value = true
  setGlossaryExchangeBusy(true)
  sequence += 1
  const mySequence = sequence

  confirming.value = false
  confirmError.value = null
  confirmedOutcome.value = null
  decision.value = 'keep_mine'

  const result = await openPromptImportPreview()
  if (mySequence !== sequence) return
  opening.value = false
  setGlossaryExchangeBusy(false)

  if (result.outcome === 'cancelled') return

  overlayOpen.value = true

  if (result.outcome === 'ipc_unavailable') {
    status.value = 'ipc_unavailable'
    loadError.value = null
    preview.value = null
    return
  }
  if (result.outcome === 'error') {
    status.value = 'error'
    loadError.value = result.error
    preview.value = null
    return
  }

  preview.value = result.preview
  status.value = 'loaded'
  loadError.value = null
  selectedTier.value = result.preview.work !== null ? 'work' : 'global'
}

function closeWithoutCancelling(outcome: 'inserted' | 'updated' | 'skipped'): void {
  confirmedOutcome.value = outcome
  overlayOpen.value = false
}

/**
 * Xác nhận lượt nhập — lệnh `prompt.import.confirm`. Đọc `selectedTier`/`decision` từ CHÍNH
 * module này (tham số của `dispatch` không mang được, §Design Notes: `dispatch` không nhận
 * tham số). Thành công ⇒ đóng lớp phủ. Trượt (kể cả va lạc quan) ⇒ hiện lỗi, lớp phủ Ở LẠI
 * MỞ (lô GIỮ LẠI phía Rust để thử lại).
 */
export async function confirmPromptImportPreview(): Promise<void> {
  if (confirming.value || preview.value === null) return

  confirming.value = true
  confirmError.value = null
  const mySequence = sequence

  const tierPreview = promptImportSelectedTierPreview.value
  const sendDecision = tierPreview?.kind === 'conflict' ? decision.value : null

  const result = await confirmPromptImport(selectedTier.value, sendDecision)
  if (mySequence !== sequence) return

  confirming.value = false
  if (result.error !== null || result.outcome === null) {
    confirmError.value = result.error
    return
  }

  closeWithoutCancelling(result.outcome)
}

/** Huỷ lô đang treo và đóng lớp phủ — lệnh `prompt.import.cancel`. **0** lượt ghi. */
export async function cancelPromptImportPreview(): Promise<void> {
  if (confirming.value) return

  const mySequence = sequence
  overlayOpen.value = false

  const result = await cancelPromptImport()
  if (mySequence !== sequence) return
  if (result !== null) {
    console.error(
      `[prompt-import] \`prompt_set_cancel_import\` tra ve loi khi huy lo dang treo: ${JSON.stringify(result)}`,
    )
  }
}

/** Vứt toàn bộ state — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường `reset*()`. */
export function resetPromptImport(): void {
  sequence += 1
  overlayOpen.value = false
  status.value = 'unknown'
  loadError.value = null
  preview.value = null
  selectedTier.value = 'global'
  decision.value = 'keep_mine'
  confirming.value = false
  confirmError.value = null
  confirmedOutcome.value = null
  opening.value = false
  resetGlossaryExchangeGate()
}
