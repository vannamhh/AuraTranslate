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
 * 🔵 `selectedPromptSetName` — trạng thái phiên, lựa chọn TAY nhớ theo Tác phẩm
 * ─────────────────────────────────────────────────────────────────────────────
 * Quyết định #4 spec 4.4: một bộ là TÊN + THÂN, bảng `prompt_set` không có trường "đang dùng".
 * Tên Ice chọn TAY được nhớ riêng ở `work.chosen_prompt_set_name` (`prompt_set_choice_*`):
 * [`setSelectedPromptSetName`] ghi cột đó khi có Tác phẩm mở, [`restorePromptSetForWork`] đọc
 * lại lúc mở Tác phẩm (bộ đã lưu ⇒ thể loại ⇒ không chọn). Tự chọn theo thể loại KHÔNG ghi.
 * Module này tự xoá tên đang chọn nếu nó không còn giải được sau một lượt [`loadPromptSets`]
 * (bộ vừa bị xoá/đổi tên ở nơi khác), không sửa tên đã lưu.
 *
 * ⚠️ Cùng luật mọi state Vue khác: tệp này KHÔNG được `import` vào `src/commands/index.ts`.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import type { IpcError } from './i18n'
import {
  promptSetChoiceGet,
  promptSetChoiceSet,
  promptSetCreate,
  promptSetDelete,
  promptSetExport,
  promptSetExportMany,
  promptSetList,
  promptSetProofreadChoiceGet,
  promptSetProofreadChoiceSetGlobal,
  promptSetProofreadChoiceSetWork,
  promptSetProofreadDeleteConsequence,
  promptSetRename,
  promptSetUpdateBody,
} from './config/promptset'
import type {
  ProofreadChoiceWire,
  ProofreadDeleteConsequence,
  PromptSetExportFile,
  PromptSetKind,
  PromptSetTier,
  PromptSetWarningsWire,
  PromptSetWire,
} from './config/promptset'
import { genreLabel } from './modes/libraryWorkCard'
import type { GenreId } from './modes/libraryWorkCard'
import { PROMPT_TEMPLATES } from './promptTemplates'
import { glossaryExchangeBusy, setGlossaryExchangeBusy } from './glossaryExchangeGate'

const LIBRARY_KIND: PromptSetKind = 'translate'

const resolvedSets = ref<PromptSetWire[]>([])
const proofreadResolved = ref<PromptSetWire[]>([])
const proofreadChoice = ref<ProofreadChoiceWire | null>(null)
const proofreadChoiceReadError = ref<IpcError | null>(null)
const proofreadChoiceWriteError = ref<IpcError | null>(null)
let proofreadSequence = 0
let proofreadChoiceSequence = 0
const workTierAvailable = ref(false)
const loading = ref(false)
const loadError = ref<IpcError | null>(null)
const busy = ref(false)
const actionError = ref<IpcError | null>(null)
/** Cảnh báo dấu ngoặc của lượt Tạo/Sửa thân GẦN NHẤT — `null` sau một lượt Đổi tên/Xoá (hai
 * thao tác đó không mang thân để mà cảnh báo) hoặc khi chưa có lượt Tạo/Sửa nào. */
const actionWarnings = ref<PromptSetWarningsWire | null>(null)
const selectedSetName = ref<string | null>(null)
let selectionIsAutomatic = false
let pendingGenre: string | null = null
let pendingRestore = false
let lastGenre: string | null = null
/** Từ vựng biến số ratify (`PromptVariable::ALL` phía Rust) — nguồn sự thật DUY NHẤT mà
 * `PromptLibraryOverlay.vue` render từ đó, cùng khối 🔴 đầu `PromptSetListWire::variables`
 * (`commands/promptset.rs`). Không đổi trên một lượt trượt — cùng lý do `resolvedSets` giữ
 * nguyên giá trị CŨ khi `loadPromptSets` gặp lỗi, thay vì tụt về rỗng. */
const variableNames = ref<string[]>([])
/** Lỗi/đường dẫn của lượt XUẤT gần nhất — Story 4.5. RIÊNG với `actionError` (Tạo/Đổi tên/
 * Sửa thân/Xoá): một lỗi xuất không phải một lỗi soạn, và không nên xoá banner của form đang
 * mở. `null` khi chưa xuất lần nào, hoặc lượt gần nhất thành công/bị huỷ. */
const exportError = ref<IpcError | null>(null)
const choiceError = ref<IpcError | null>(null)
const exportManyError = ref<IpcError | null>(null)
const exportFiles = ref<PromptSetExportFile[] | null>(null)

/** Số thứ tự lượt đọc — chỉ lượt MỚI NHẤT được quyền ghi kết quả (khuôn `aiConfigState.ts`). */
let sequence = 0

export const promptSets: DeepReadonly<Ref<PromptSetWire[]>> = readonly(resolvedSets)
/** Kept apart from `promptSets`: `reloadPromptSets` drops a translate choice missing from its list. */
export const proofreadInstructionSets: DeepReadonly<Ref<PromptSetWire[]>> = readonly(proofreadResolved)
export const proofreadInstructionChoice: DeepReadonly<Ref<ProofreadChoiceWire | null>> = readonly(proofreadChoice)
export const proofreadChoiceReadFailure: DeepReadonly<Ref<IpcError | null>> = readonly(proofreadChoiceReadError)
export const proofreadChoiceWriteFailure: DeepReadonly<Ref<IpcError | null>> = readonly(proofreadChoiceWriteError)
export const promptSetWorkTierAvailable: DeepReadonly<Ref<boolean>> = readonly(workTierAvailable)
export const promptSetLoading: DeepReadonly<Ref<boolean>> = readonly(loading)
export const promptSetLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const promptSetBusy: DeepReadonly<Ref<boolean>> = readonly(busy)
export const promptSetActionError: DeepReadonly<Ref<IpcError | null>> = readonly(actionError)
export const promptSetActionWarnings: DeepReadonly<Ref<PromptSetWarningsWire | null>> = readonly(actionWarnings)
export const selectedPromptSetName: DeepReadonly<Ref<string | null>> = readonly(selectedSetName)
export const promptSetVariables: DeepReadonly<Ref<string[]>> = readonly(variableNames)
export const promptSetExportManyError: DeepReadonly<Ref<IpcError | null>> = readonly(exportManyError)
export const promptSetExportFiles: DeepReadonly<Ref<PromptSetExportFile[] | null>> = readonly(exportFiles)
export const promptSetChoiceError: DeepReadonly<Ref<IpcError | null>> = readonly(choiceError)
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
  exportManyError.value = null
  exportFiles.value = null
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

/** Entry load (screen opened, Work opened): shows the loading line. Writes re-read through `reloadPromptSets`. */
export async function loadPromptSets(): Promise<void> {
  loading.value = true
  await reloadPromptSets()
}

function reloadForKind(kind: PromptSetKind): Promise<void> {
  return kind === 'proofread' ? reloadProofreadInstructions() : reloadPromptSets()
}

/** Entry load for the `proofread` card: list and both choices. Never touches the translate list or its selection. */
export async function loadProofreadInstructions(): Promise<void> {
  loading.value = true
  await reloadProofreadInstructions()
}

async function reloadProofreadInstructions(): Promise<void> {
  const mine = ++proofreadSequence
  const myChoice = ++proofreadChoiceSequence
  const [list, choice] = await Promise.all([promptSetList('proofread'), promptSetProofreadChoiceGet()])
  if (mine !== proofreadSequence) return

  loading.value = false
  if (list.error !== null) {
    loadError.value = list.error
    return
  }
  loadError.value = null
  if (list.sets !== null) {
    workTierAvailable.value = list.workTierAvailable
    proofreadResolved.value = list.sets
  }
  if (myChoice !== proofreadChoiceSequence) return
  proofreadChoiceReadError.value = choice.error
  if (choice.error === null) proofreadChoice.value = choice.choice
}

async function refreshProofreadChoice(): Promise<void> {
  const mine = ++proofreadChoiceSequence
  const choice = await promptSetProofreadChoiceGet()
  if (mine !== proofreadChoiceSequence) return
  proofreadChoiceReadError.value = choice.error
  if (choice.error === null) proofreadChoice.value = choice.choice
}

/** Global card box: `null` clears the choice. Re-reads from Rust afterwards, so a failed write shows the stored value. */
export async function chooseProofreadGlobal(name: string | null): Promise<void> {
  proofreadChoiceWriteError.value = null
  proofreadChoiceWriteError.value = await promptSetProofreadChoiceSetGlobal(name)
  await refreshProofreadChoice()
}

/** Work card box: `follow_global` and `off` carry no name. */
export async function chooseProofreadWork(mode: 'follow_global' | 'off' | 'named', name: string | null): Promise<void> {
  proofreadChoiceWriteError.value = null
  proofreadChoiceWriteError.value =
    mode === 'off'
      ? await promptSetProofreadChoiceSetWork(null, true)
      : await promptSetProofreadChoiceSetWork(mode === 'named' ? name : null, false)
  await refreshProofreadChoice()
}

/** What deleting this `proofread` set would do to the chosen instruction, as computed by Rust; `null` when the call failed (error in `actionError`). */
export async function proofreadDeleteConsequence(
  tier: PromptSetTier,
  id: number,
): Promise<ProofreadDeleteConsequence | null> {
  const result = await promptSetProofreadDeleteConsequence(tier, id)
  if (result.error !== null) actionError.value = result.error
  return result.error === null ? result.consequence : null
}

/** Re-reads without raising `loading`: the library hides its list and editor on that flag. */
async function reloadPromptSets(): Promise<void> {
  const mine = ++sequence
  const result = await promptSetList(LIBRARY_KIND)
  if (mine !== sequence) return // một lượt tải MỚI đã vượt mặt lượt này

  loading.value = false
  if (result.error !== null) {
    loadError.value = result.error
    pendingGenre = null
    pendingRestore = false
    return
  }
  loadError.value = null
  if (result.sets === null) {
    pendingGenre = null
    pendingRestore = false
    return
  }

  workTierAvailable.value = result.workTierAvailable
  resolvedSets.value = result.sets
  if (result.variables) variableNames.value = result.variables
  // Bộ đang chọn cho AI Translation có thể đã bị xoá/đổi tên ở nơi khác — đừng tiếp tục trỏ
  // vào một tên không còn giải được (xem khối 🔵 đầu tệp).
  if (selectedSetName.value !== null && !result.sets.some((s) => s.name === selectedSetName.value)) {
    selectedSetName.value = null
    selectionIsAutomatic = false
  }
  if (pendingGenre !== null) {
    const genre = pendingGenre
    if (pendingRestore) {
      const choice = await promptSetChoiceGet()
      if (mine !== sequence) return
      pendingGenre = null
      pendingRestore = false
      if (choice.error !== null) choiceError.value = choice.error
      const saved = choice.name === null ? null : findSetByTrimmedName(choice.name)
      if (saved !== null) {
        selectedSetName.value = saved.name
        selectionIsAutomatic = false
        return
      }
      selectedSetName.value = null
      selectionIsAutomatic = false
    } else {
      pendingGenre = null
    }
    applyGenre(genre)
  }
}

function findSetByTrimmedName(name: string): PromptSetWire | null {
  const wanted = name.trim().toLowerCase()
  return resolvedSets.value.find((s) => s.name.trim().toLowerCase() === wanted) ?? null
}

/**
 * Chọn bộ hiệu lực cho AI Translation bằng TAY (xem khối 🔵 đầu tệp). `name` phải khớp một bộ
 * ĐANG có trong danh sách đã nạp; một tên lạ bị bỏ qua. `null` bỏ chọn. Khi có Tác phẩm mở,
 * lựa chọn được ghi xuống `project.db` (lỗi ghi vào [`promptSetChoiceError`], lựa chọn trong
 * phiên vẫn giữ).
 */
export function setSelectedPromptSetName(name: string | null): void {
  if (name !== null && !resolvedSets.value.some((s) => s.name === name)) return
  selectedSetName.value = name
  selectionIsAutomatic = false
  if (!workTierAvailable.value) return
  choiceError.value = null
  void promptSetChoiceSet(name).then((err) => {
    choiceError.value = err
  })
}

function applyGenre(genre: string): void {
  const label = genreLabel(genre)?.trim().toLowerCase() ?? null
  const match = label === null ? undefined : resolvedSets.value.find((s) => s.name.trim().toLowerCase() === label)
  if (match !== undefined) {
    selectedSetName.value = match.name
    selectionIsAutomatic = true
  } else if (selectionIsAutomatic) {
    selectedSetName.value = null
    selectionIsAutomatic = false
  }
}

/** On opening a Work: its saved hand-picked set, else the genre's set, else none. */
export async function restorePromptSetForWork(genre: string): Promise<void> {
  lastGenre = genre
  pendingGenre = genre
  pendingRestore = true
  choiceError.value = null
  await loadPromptSets()
}

/**
 * Tạo một bộ mới ở tầng `tier` — **hàm thuần re-validate**, không tin chỗ gọi: kiểm lại
 * [`isPromptSetNameValid`] cùng lý do `saveAiConfigField` re-validate trước khi gọi IPC.
 * Thành công ⇒ nạp lại toàn bộ danh sách và ghi [`promptSetActionWarnings`] (Quyết định #3:
 * một thân mang token lạ hoặc thiếu `{{glossary_terms}}` vẫn là một lượt THÀNH CÔNG).
 */
export async function createPromptSet(
  tier: PromptSetTier,
  name: string,
  body: string,
  kind: PromptSetKind = LIBRARY_KIND,
): Promise<void> {
  if (busy.value) return
  if (!isPromptSetNameValid(name)) return

  busy.value = true
  actionError.value = null
  actionWarnings.value = null

  const result = await promptSetCreate(tier, kind, name.trim(), body)

  if (result.error !== null) {
    busy.value = false
    actionError.value = result.error
    return
  }
  actionWarnings.value = result.warnings
  await reloadForKind(kind)
  busy.value = false
}

/** `true` when a set named like the template's genre label already exists at `tier` (trimmed,
 * case-insensitive). A Work row that shadows a Global one counts as Global too. */
export function promptTemplateExists(tier: PromptSetTier, id: GenreId): boolean {
  const label = genreLabel(id)?.trim().toLowerCase()
  if (label === undefined) return false
  return resolvedSets.value.some(
    (s) =>
      s.name.trim().toLowerCase() === label &&
      (tier === 'work' ? s.tier === 'work' : s.tier === 'global' || s.shadowed_body !== null),
  )
}

/**
 * Creates one set per template id at `tier`, sequentially and without rollback, then reloads
 * once and re-applies the latest genre (a hand-picked set still wins). Ids already present at
 * the tier are skipped; a `prompt_set.name_taken` the list did not predict skips that id only.
 * Returns how many sets were created.
 */
export async function addPromptTemplates(tier: PromptSetTier, ids: ReadonlyArray<GenreId>): Promise<number> {
  if (busy.value) return 0

  busy.value = true
  actionError.value = null
  actionWarnings.value = null

  let created = 0
  let attempted = 0
  for (const id of ids) {
    if (promptTemplateExists(tier, id)) continue
    const name = genreLabel(id)
    if (name === null) continue
    attempted += 1
    const result = await promptSetCreate(tier, LIBRARY_KIND, name, PROMPT_TEMPLATES[id])
    if (result.error !== null) {
      if (result.error.code === 'prompt_set.name_taken') continue
      actionError.value = result.error
      break
    }
    created += 1
  }

  if (attempted > 0) {
    pendingGenre = selectedSetName.value === null || selectionIsAutomatic ? lastGenre : null
    await reloadPromptSets()
  }
  busy.value = false
  return created
}

/**
 * Đổi TÊN của bộ `(tier, id)` — **hàm thuần re-validate**, cùng lý do [`createPromptSet`].
 * Đổi tên không mang thân, nên không cảnh báo dấu ngoặc nào đi kèm — [`promptSetActionWarnings`]
 * bị vứt để một cảnh báo CŨ của lượt Tạo/Sửa thân trước đó không còn hiện ra như thể nó thuộc
 * về lượt Đổi tên vừa xong.
 */
export async function renamePromptSet(
  tier: PromptSetTier,
  id: number,
  newName: string,
  kind: PromptSetKind = LIBRARY_KIND,
): Promise<void> {
  if (busy.value) return
  if (!isPromptSetNameValid(newName)) return

  busy.value = true
  actionError.value = null
  actionWarnings.value = null

  const err = await promptSetRename(tier, kind, id, newName.trim())

  if (err !== null) {
    busy.value = false
    actionError.value = err
    return
  }
  await reloadForKind(kind)
  busy.value = false
}

/**
 * Sửa THÂN của bộ `(tier, id)` — thân không bao giờ bị từ chối vì nội dung của nó (Quyết định
 * #3), nên KHÔNG có bước re-validate nào trước lượt gọi, khác [`createPromptSet`]/
 * [`renamePromptSet`]. Thành công ⇒ nạp lại danh sách và ghi cảnh báo dấu ngoặc của thân MỚI.
 */
export async function updatePromptSetBody(
  tier: PromptSetTier,
  id: number,
  body: string,
  kind: PromptSetKind = LIBRARY_KIND,
): Promise<void> {
  if (busy.value) return

  busy.value = true
  actionError.value = null
  actionWarnings.value = null

  const result = await promptSetUpdateBody(tier, kind, id, body)

  if (result.error !== null) {
    busy.value = false
    actionError.value = result.error
    return
  }
  actionWarnings.value = result.warnings
  await reloadForKind(kind)
  busy.value = false
}

/**
 * Xoá bộ `(tier, id)`. Xoá một bộ Work đang che một bộ Global cùng tên làm bộ Global đó hiệu
 * lực trở lại ngay ở lượt [`reloadPromptSets`] mà hàm này gọi sau khi thành công (I/O Matrix
 * spec 4.4 "Delete a shadowing Work set").
 */
export async function deletePromptSet(tier: PromptSetTier, id: number, kind: PromptSetKind = LIBRARY_KIND): Promise<void> {
  if (busy.value) return

  busy.value = true
  actionError.value = null
  actionWarnings.value = null

  const err = await promptSetDelete(tier, kind, id)

  if (err !== null) {
    busy.value = false
    actionError.value = err
    return
  }
  await reloadForKind(kind)
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
export async function exportPromptSet(tier: PromptSetTier, id: number, kind: PromptSetKind = LIBRARY_KIND): Promise<void> {
  if (glossaryExchangeBusy.value) return

  setGlossaryExchangeBusy(true)
  exportError.value = null
  exportFiles.value = null
  const result = await promptSetExport(tier, kind, id)
  setGlossaryExchangeBusy(false)

  if (result.outcome === 'error') {
    exportError.value = result.error
  }
}

export async function exportPromptSets(
  sets: ReadonlyArray<{ tier: PromptSetTier; id: number }>,
  kind: PromptSetKind = LIBRARY_KIND,
): Promise<void> {
  if (glossaryExchangeBusy.value || sets.length === 0) return

  setGlossaryExchangeBusy(true)
  exportManyError.value = null
  exportFiles.value = null
  const result = await promptSetExportMany(sets.map((s) => ({ ...s, kind })))
  setGlossaryExchangeBusy(false)

  if (result.outcome === 'error') exportManyError.value = result.error
  if (result.outcome === 'done') exportFiles.value = result.files
}

/**
 * First free `"<name> (<suffix>)"`, then `"<name> (<suffix> 2)"`, and so on, skipping `taken`
 * and the first `skip` candidates (used to retry after a `name_taken` the list did not predict).
 */
export function duplicatePromptSetName(
  name: string,
  suffix: string,
  taken: ReadonlySet<string>,
  skip = 0,
): string {
  let seen = 0
  for (let n = 1; ; n += 1) {
    const candidate = n === 1 ? `${name} (${suffix})` : `${name} (${suffix} ${n})`
    if (taken.has(candidate)) continue
    if (seen < skip) {
      seen += 1
      continue
    }
    return candidate
  }
}

/**
 * Vứt toàn bộ state của mục — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`, cùng khuôn `resetAiConfigSection`.
 */
export function resetPromptSets(): void {
  sequence += 1
  proofreadSequence += 1
  proofreadChoiceSequence += 1
  proofreadResolved.value = []
  proofreadChoice.value = null
  proofreadChoiceReadError.value = null
  proofreadChoiceWriteError.value = null
  resolvedSets.value = []
  workTierAvailable.value = false
  loading.value = false
  loadError.value = null
  busy.value = false
  actionError.value = null
  actionWarnings.value = null
  selectedSetName.value = null
  selectionIsAutomatic = false
  pendingGenre = null
  pendingRestore = false
  lastGenre = null
  variableNames.value = []
  exportError.value = null
  exportManyError.value = null
  exportFiles.value = null
  choiceError.value = null
}
