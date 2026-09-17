/**
 * State của mục **bộ prompt theo thể loại** — Story 4.4, FR69, `core/scope/kinds.rs:165`
 * (`ScopeKind::Prompt`, ghi đè CẢ BỘ theo TÊN, Ice ký 2026-09-17).
 *
 * Cùng khuôn `aiConfigState.ts`: `readonly` refs cộng hàm truy cập, một bộ đếm thứ tự lượt
 * đọc để một lượt tải CŨ không ghi đè lên kết quả của một lượt MỚI hơn, và một `reset*()` cho
 * test (`check:panel-refs` đòi mọi ô nhớ cấp module có đường đó).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MỘT CỜ `busy` DÙNG CHUNG CHO BỐN THAO TÁC GHI, KHÔNG một bản ghi theo từng `id`
 * ─────────────────────────────────────────────────────────────────────────────
 * Khác `aiConfigState.ts` (đúng NĂM trường cố định ⇒ `FieldRecord<T>`), danh sách bộ prompt
 * dài tuỳ ý — không có một tập khoá cố định để dựng một `Record`. Tạo/đổi tên/sửa thân/xoá vì
 * thế dùng CHUNG một cờ `busy`, cùng khuôn `keyBusy` của `aiConfigState.ts` (Lưu và Xoá khoá
 * API cũng dùng chung một cờ): màn soạn chỉ thao tác MỘT bộ tại một thời điểm, nên khoá tái
 * nhập trên cả bốn thao tác là đúng, không phải một giản lược.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 `selectedPromptSetName` — trạng thái CỤC BỘ, KHÔNG một trường Rust
 * ─────────────────────────────────────────────────────────────────────────────
 * Quyết định #4 spec 4.4: một bộ là TÊN + THÂN, không trường "đang dùng"/"mặc định" nào được
 * lưu. "Bộ hiệu lực" mà bảng AI Translation hiển thị vì thế là một lựa chọn HIỂN THỊ thuần
 * phía webview, không một lượt `invoke` nào đứng sau [`setSelectedPromptSetName`] — module
 * này chỉ giữ TÊN đang chọn và tự xoá nó nếu tên đó không còn giải được sau một lượt
 * [`loadPromptSets`] (bộ vừa bị xoá/đổi tên ở nơi khác), để không trỏ vào một bộ đã biến mất.
 *
 * ⚠️ Cùng luật mọi state Vue khác: tệp này KHÔNG được `import` vào `src/commands/index.ts`.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import type { IpcError } from './i18n'
import {
  promptSetCreate,
  promptSetDelete,
  promptSetExport,
  promptSetList,
  promptSetRename,
  promptSetUpdateBody,
} from './config/promptset'
import type { PromptSetTier, PromptSetWarningsWire, PromptSetWire } from './config/promptset'
import { glossaryExchangeBusy, setGlossaryExchangeBusy } from './glossaryExchangeGate'

const resolvedSets = ref<PromptSetWire[]>([])
const workTierAvailable = ref(false)
const loading = ref(false)
const loadError = ref<IpcError | null>(null)
const busy = ref(false)
const actionError = ref<IpcError | null>(null)
/** Cảnh báo dấu ngoặc của lượt Tạo/Sửa thân GẦN NHẤT — `null` sau một lượt Đổi tên/Xoá (hai
 * thao tác đó không mang thân để mà cảnh báo) hoặc khi chưa có lượt Tạo/Sửa nào. */
const actionWarnings = ref<PromptSetWarningsWire | null>(null)
const selectedSetName = ref<string | null>(null)
/** Từ vựng biến số ratify (`PromptVariable::ALL` phía Rust) — nguồn sự thật DUY NHẤT mà
 * `PromptLibraryOverlay.vue` render từ đó, cùng khối 🔴 đầu `PromptSetListWire::variables`
 * (`commands/promptset.rs`). Không đổi trên một lượt trượt — cùng lý do `resolvedSets` giữ
 * nguyên giá trị CŨ khi `loadPromptSets` gặp lỗi, thay vì tụt về rỗng. */
const variableNames = ref<string[]>([])
/** Lỗi/đường dẫn của lượt XUẤT gần nhất — Story 4.5. RIÊNG với `actionError` (Tạo/Đổi tên/
 * Sửa thân/Xoá): một lỗi xuất không phải một lỗi soạn, và không nên xoá banner của form đang
 * mở. `null` khi chưa xuất lần nào, hoặc lượt gần nhất thành công/bị huỷ. */
const exportError = ref<IpcError | null>(null)

/** Số thứ tự lượt đọc — chỉ lượt MỚI NHẤT được quyền ghi kết quả (khuôn `aiConfigState.ts`). */
let sequence = 0

export const promptSets: DeepReadonly<Ref<PromptSetWire[]>> = readonly(resolvedSets)
export const promptSetWorkTierAvailable: DeepReadonly<Ref<boolean>> = readonly(workTierAvailable)
export const promptSetLoading: DeepReadonly<Ref<boolean>> = readonly(loading)
export const promptSetLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const promptSetBusy: DeepReadonly<Ref<boolean>> = readonly(busy)
export const promptSetActionError: DeepReadonly<Ref<IpcError | null>> = readonly(actionError)
export const promptSetActionWarnings: DeepReadonly<Ref<PromptSetWarningsWire | null>> = readonly(actionWarnings)
export const selectedPromptSetName: DeepReadonly<Ref<string | null>> = readonly(selectedSetName)
export const promptSetVariables: DeepReadonly<Ref<string[]>> = readonly(variableNames)
export const promptSetExportError: DeepReadonly<Ref<IpcError | null>> = readonly(exportError)
/** Cờ dùng CHUNG với lượt Nhập (`promptSetImportState.ts`) và với Xuất/Nhập Glossary — cả
 * ba đều mở một hộp thoại hệ điều hành của Rust; xem doc-comment đã cập nhật của
 * `glossaryExchangeGate.ts`. */
export const promptSetExchangeBusy: DeepReadonly<Ref<boolean>> = glossaryExchangeBusy

/**
 * Vứt lỗi/cảnh báo của lượt Tạo/Sửa thân GẦN NHẤT — gọi khi chỗ gọi chuyển sự chú ý sang một
 * bộ/thao tác KHÁC mà không qua một lượt ghi mới (chọn một hàng khác, mở form Tạo), để banner
 * lỗi/cảnh báo của bộ TRƯỚC không hiện lên như thể nó thuộc về bộ/form ĐANG mở. Bốn hàm
 * `createPromptSet`/`renamePromptSet`/`updatePromptSetBody`/`deletePromptSet` đã tự vứt hai ô
 * này ở ĐẦU mỗi lượt ghi thật — hàm này là đường RIÊNG cho chỗ gọi không ghi gì cả.
 */
export function clearPromptSetActionFeedback(): void {
  actionError.value = null
  actionWarnings.value = null
  // Story 4.5 review fix — cùng lý do hai ô trên: chọn sang một hàng KHÁC không nên ngầm
  // định lỗi xuất của hàng TRƯỚC vẫn còn đúng — cùng khuyết tật Story 4.4's review đã vá hai
  // lần (Pass 1 hàng 2 và 3).
  exportError.value = null
}

/** Bộ khớp `id`, hoặc `null` nếu không có (chưa nạp, hoặc `id` không còn tồn tại). */
export function promptSetById(id: number): PromptSetWire | null {
  return resolvedSets.value.find((s) => s.id === id) ?? null
}

/** Bộ khớp `name` — danh sách đã phân giải nên có tối đa MỘT hàng cho mỗi tên (Quyết định #1:
 * ghi đè cả bộ, khoá theo tên). `null` nếu không có. */
export function promptSetByName(name: string): PromptSetWire | null {
  return resolvedSets.value.find((s) => s.name === name) ?? null
}

/** Bộ ĐANG CHỌN cho AI Translation, hoặc `null` khi chưa chọn / bộ đã chọn không còn giải
 * được. */
export function selectedPromptSet(): PromptSetWire | null {
  const name = selectedSetName.value
  return name === null ? null : promptSetByName(name)
}

/**
 * Kiểm tra hình dạng TÊN bộ — **hàm thuần, xuất khẩu**, cùng luật
 * `core::promptset::store::validate_name` phía Rust: trim rồi từ chối rỗng/toàn khoảng
 * trắng. `String.prototype.trim()` cắt cùng lớp Unicode `White_Space` mà `str::trim()` phía
 * Rust dùng (gồm cả `\u{3000}` — cùng cách đọc `isAiConfigKeyValueValid` đã ghi cho
 * `core::aiconfig::validate_key`). Rust vẫn là trọng tài cuối; hàm này chỉ tránh một vòng IPC
 * vô ích cho một giá trị chắc chắn bị từ chối (I/O Matrix spec 4.4 "Blank-ish name").
 */
export function isPromptSetNameValid(raw: string): boolean {
  return raw.trim().length > 0
}

/**
 * Đọc lại toàn bộ danh sách, hai tầng đã phân giải — gọi mỗi lần màn hình đọc bộ prompt trở
 * thành màn ĐANG CHỌN, và sau MỖI lượt Tạo/Đổi tên/Sửa thân/Xoá thành công (cùng lý do
 * `glossaryManageState.ts` nạp lại TRỌN sau một lượt Xoá/Đẩy tầng: `shadowed_body` của các
 * hàng KHÁC có thể đổi theo một thao tác vừa làm, và chỉ Rust mới tính lại đúng cờ đó).
 */
export async function loadPromptSets(): Promise<void> {
  const mine = ++sequence
  loading.value = true
  const result = await promptSetList()
  if (mine !== sequence) return // một lượt tải MỚI đã vượt mặt lượt này

  loading.value = false
  if (result.error !== null) {
    loadError.value = result.error
    return
  }
  loadError.value = null
  if (result.sets === null) return

  workTierAvailable.value = result.workTierAvailable
  resolvedSets.value = result.sets
  if (result.variables) variableNames.value = result.variables
  // Bộ đang chọn cho AI Translation có thể đã bị xoá/đổi tên ở nơi khác — đừng tiếp tục trỏ
  // vào một tên không còn giải được (xem khối 🔵 đầu tệp).
  if (selectedSetName.value !== null && !result.sets.some((s) => s.name === selectedSetName.value)) {
    selectedSetName.value = null
  }
}

/**
 * Chọn bộ hiệu lực cho AI Translation — trạng thái CỤC BỘ, 0 lượt IPC (xem khối 🔵 đầu tệp).
 * `name` phải khớp một bộ ĐANG có trong danh sách đã nạp; một tên lạ bị bỏ qua thay vì trỏ
 * vào một bộ không tồn tại. `null` bỏ chọn.
 */
export function setSelectedPromptSetName(name: string | null): void {
  if (name !== null && !resolvedSets.value.some((s) => s.name === name)) return
  selectedSetName.value = name
}

/**
 * Tạo một bộ mới ở tầng `tier` — **hàm thuần re-validate**, không tin chỗ gọi: kiểm lại
 * [`isPromptSetNameValid`] cùng lý do `saveAiConfigField` re-validate trước khi gọi IPC.
 * Thành công ⇒ nạp lại toàn bộ danh sách và ghi [`promptSetActionWarnings`] (Quyết định #3:
 * một thân mang token lạ hoặc thiếu `{{glossary_terms}}` vẫn là một lượt THÀNH CÔNG).
 */
export async function createPromptSet(tier: PromptSetTier, name: string, body: string): Promise<void> {
  if (busy.value) return
  if (!isPromptSetNameValid(name)) return

  busy.value = true
  actionError.value = null
  actionWarnings.value = null

  const result = await promptSetCreate(tier, name.trim(), body)

  if (result.error !== null) {
    busy.value = false
    actionError.value = result.error
    return
  }
  actionWarnings.value = result.warnings
  await loadPromptSets()
  busy.value = false
}

/**
 * Đổi TÊN của bộ `(tier, id)` — **hàm thuần re-validate**, cùng lý do [`createPromptSet`].
 * Đổi tên không mang thân, nên không cảnh báo dấu ngoặc nào đi kèm — [`promptSetActionWarnings`]
 * bị vứt để một cảnh báo CŨ của lượt Tạo/Sửa thân trước đó không còn hiện ra như thể nó thuộc
 * về lượt Đổi tên vừa xong.
 */
export async function renamePromptSet(tier: PromptSetTier, id: number, newName: string): Promise<void> {
  if (busy.value) return
  if (!isPromptSetNameValid(newName)) return

  busy.value = true
  actionError.value = null
  actionWarnings.value = null

  const err = await promptSetRename(tier, id, newName.trim())

  if (err !== null) {
    busy.value = false
    actionError.value = err
    return
  }
  await loadPromptSets()
  busy.value = false
}

/**
 * Sửa THÂN của bộ `(tier, id)` — thân không bao giờ bị từ chối vì nội dung của nó (Quyết định
 * #3), nên KHÔNG có bước re-validate nào trước lượt gọi, khác [`createPromptSet`]/
 * [`renamePromptSet`]. Thành công ⇒ nạp lại danh sách và ghi cảnh báo dấu ngoặc của thân MỚI.
 */
export async function updatePromptSetBody(tier: PromptSetTier, id: number, body: string): Promise<void> {
  if (busy.value) return

  busy.value = true
  actionError.value = null
  actionWarnings.value = null

  const result = await promptSetUpdateBody(tier, id, body)

  if (result.error !== null) {
    busy.value = false
    actionError.value = result.error
    return
  }
  actionWarnings.value = result.warnings
  await loadPromptSets()
  busy.value = false
}

/**
 * Xoá bộ `(tier, id)`. Xoá một bộ Work đang che một bộ Global cùng tên làm bộ Global đó hiệu
 * lực trở lại ngay ở lượt [`loadPromptSets`] mà hàm này gọi sau khi thành công (I/O Matrix
 * spec 4.4 "Delete a shadowing Work set").
 */
export async function deletePromptSet(tier: PromptSetTier, id: number): Promise<void> {
  if (busy.value) return

  busy.value = true
  actionError.value = null
  actionWarnings.value = null

  const err = await promptSetDelete(tier, id)

  if (err !== null) {
    busy.value = false
    actionError.value = err
    return
  }
  await loadPromptSets()
  busy.value = false
}

/**
 * Xuất bộ `(tier, id)` — mở hộp thoại LƯU trong Rust, khuôn `exportGlossaryManageTier`. Chặn
 * bấm chồng bằng cờ DÙNG CHUNG (`glossaryExchangeBusy`, khuôn `openGlossaryImportPreviewOverlay`'s
 * `if (opening.value || glossaryExchangeBusy.value) return`) — Xuất một bộ prompt và Nhập
 * (bộ prompt lẫn Glossary) đều mở một hộp thoại hệ điều hành, không được chồng nhau.
 *
 * **Không bao giờ ném.** Huỷ hộp thoại (`outcome: 'cancelled'`) là im lặng có chủ — không
 * ghi `exportError`. `outcome: 'ipc_unavailable'` cũng không ghi lỗi (chạy ngoài Tauri không
 * phải một lỗi của người dùng, cùng khuôn mọi adapter khác).
 */
export async function exportPromptSet(tier: PromptSetTier, id: number): Promise<void> {
  if (glossaryExchangeBusy.value) return

  setGlossaryExchangeBusy(true)
  exportError.value = null
  const result = await promptSetExport(tier, id)
  setGlossaryExchangeBusy(false)

  if (result.outcome === 'error') {
    exportError.value = result.error
  }
}

/**
 * Vứt toàn bộ state của mục — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`, cùng khuôn `resetAiConfigSection`.
 */
export function resetPromptSets(): void {
  sequence += 1
  resolvedSets.value = []
  workTierAvailable.value = false
  loading.value = false
  loadError.value = null
  busy.value = false
  actionError.value = null
  actionWarnings.value = null
  selectedSetName.value = null
  variableNames.value = []
  exportError.value = null
}
