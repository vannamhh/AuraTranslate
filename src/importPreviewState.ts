/**
 * State của lớp phủ **Xem trước lượt nhập — bảng mã** (Story 6.3, FR126, AD-39 bước 1).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHUÔN `glossaryImportState.ts` — vé `sequence`, export qua `readonly()`, một hàm
 * `reset*()` nuốt TOÀN BỘ state cấp module (`check:panel-refs` Kiểm A).
 * ─────────────────────────────────────────────────────────────────────────────
 * Ba tầng theo thứ tự nhân quả (bảng mã → ranh giới nội dung → luật làm sạch).
 *
 * 🔵 **SỬA 2026-09-05 (Story 6.5) — "chỉ tầng 1 có thân" đã HẾT ĐÚNG.** Tầng 3 (luật làm
 * sạch) nay có thân thật ([`importPreviewSelectedCleanup`] + bốn hành động CRUD luật ngay
 * dưới) — chỉ còn tầng 2 (ranh giới nội dung, Story 6.9) rỗng, và
 * [`importPreviewEmptyReasonForTier`] (hẹp lại còn tham số `2`) nói ra vì sao (§Always spec
 * 6.3: "rỗng phải nói vì sao nó rỗng", khuôn `glossaryImportState.ts::importEmptyReasonFor`).
 *
 * 🔴 **Chọn một ứng viên khác KHÔNG gọi lại Rust** — `preview.candidates` đã mang bản dựng
 * thật của MỖI ứng viên (`core::segment::encoding::render_candidates` giải mã CẢ NĂM trên
 * CÙNG một cửa sổ bằng chứng, một lượt, lúc mở màn xem trước). Đổi
 * `selectedEncoding` chỉ đổi ô nào đang hiện — đây CHÍNH LÀ "chuỗi chạy lại từ bước một,
 * trong bộ nhớ, thấy kết quả ngay" (§Always spec 6.3): bước giải mã (bước 1) đã chạy cho cả
 * năm ứng viên; bảng mã THẬT SỰ chỉ chốt lại một lần nữa, chạy TRỌN bảy bước, ở
 * `confirmImportPreview()`.
 *
 * 🔴 **Huỷ xoá SẠCH nguồn đang chờ** — vòng rà 1 (defect #5): bản lỗi trước không xoá gì,
 * nên huỷ rồi xác nhận vẫn ghi được. `cancelImportPreview()` gọi thẳng
 * [`resetImportPreview`] — sau khi huỷ, `preview.value === null` chặn
 * [`confirmImportPreview`] ở NGAY DÒNG ĐẦU, trước khi có một lời gọi IPC nào — "huỷ rồi xác
 * nhận ⇒ 0 Tác phẩm được tạo" đúng ở TẦNG GIAO DIỆN, không cần một vỏ Rust riêng cho "huỷ"
 * (Rust chỉ có BA vỏ: hai xem trước + một xác nhận — xem doc-comment
 * `commands::project::cancel_import_preview`).
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import {
  cleanupAddRule,
  cleanupDeleteRule,
  cleanupEditRule,
  cleanupSetEnabled,
  confirmImportWithEncoding,
  previewImportEncodingFromFile,
  previewImportEncodingFromText,
} from './config/project'
import type {
  CleanupPreviewWire,
  CleanupRuleKindWire,
  CleanupRuleTierWire,
  CreatedWork,
  EncodingCandidateWire,
  ImportEncodingPreview,
  NormalizedPreviewWire,
} from './config/project'
import type { IpcError } from './i18n'

/**
 * Vị từ *"…HasLoaded"* — BỐN trạng thái, cùng khuôn `GlossaryImportStatus`:
 * - `'unknown'` — chưa mở lần nào, hoặc một lượt mở đang BAY;
 * - `'ipc_unavailable'` — cầu IPC vắng (chạy ngoài Tauri) — KHÔNG một lỗi;
 * - `'error'` — lượt đọc/dò trượt THẬT;
 * - `'loaded'` — đã có một `preview`.
 */
export type ImportPreviewStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'loaded'

const overlayOpen = ref(false)
const status = ref<ImportPreviewStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const preview = ref<ImportEncodingPreview | null>(null)
/** Ứng viên đang CHỌN trong dải — `null` khi chưa có `preview` nào. */
const selectedEncoding = ref<string | null>(null)
const confirming = ref(false)
const confirmError = ref<IpcError | null>(null)
/** Chặn bấm chồng — cùng lý do `importOpening` của `glossaryImportState.ts`. */
const opening = ref(false)

/** Ba tham số nộp gần nhất — lượt xác nhận cần lại chúng, và chúng KHÔNG có mặt trong
 * `ImportEncodingPreview` (Rust không lặp lại dữ liệu người dùng vừa gõ, AD-21). */
const pendingName = ref('')
const pendingSourceLang = ref('')
const pendingGenre = ref('')

/**
 * Nhánh đã mở lượt xem trước ĐANG HIỆN — `libraryImport.ts::finishImportSubmission` đọc ô
 * này để biết xoá `pastedText` hay `filePath` sau một lượt xác nhận THÀNH CÔNG.
 *
 * 🔴 **SỐNG Ở ĐÂY, không ở `libraryImport.ts`** (sửa 2026-09-04, phản biện Ice) — bản đầu
 * đặt ô này trong `libraryImport.ts` làm module đó cần một MIỄN TRỪ `check:panel-refs` (file
 * đó không có, và không NÊN có, một hàm `reset*()` — state của nó là dữ liệu FORM, không
 * phải state theo Tác phẩm). Ô này thật ra là state của LƯỢT XEM TRƯỚC (đặt khi mở, đọc khi
 * đóng), nên nó thuộc VỀ module này — [`resetImportPreview`] đã có sẵn quét qua nó, không
 * cần miễn trừ nào cả.
 *
 * 🔴 **VÀ đặt nó ở đây sửa luôn một lỗi thật** mà bản đặt-ở-`libraryImport.ts` mắc phải: bản
 * đó xoá ô này VÔ ĐIỀU KIỆN ở cuối `finishImportSubmission` (cả nhánh thành công LẪN nhánh
 * trượt) — một lượt xác nhận TRƯỢT (chọn nhầm bảng mã, dải vẫn mở để chọn lại) xoá mất "đã
 * nộp từ đâu" TRƯỚC khi người dùng kịp xác nhận LẠI cho đúng, nên lượt xác nhận LẦN HAI
 * (thành công) không còn biết xoá ô nào. Ô này không bị đụng tới khi TRƯỢT — chỉ
 * [`resetImportPreview`] (huỷ, hoặc mở một lượt xem trước MỚI) mới đổi nó.
 */
const lastSubmittedFrom = ref<'text' | 'file' | null>(null)

/**
 * **THÊM (Story 6.5)** — bản sao của nguồn ĐANG XEM TRƯỚC, giữ Ở ĐÂY (không chỉ ở
 * `PendingImportSourceState` phía Rust): một lượt thêm/sửa/xoá/bật-tắt luật làm sạch phải
 * dựng lại xem trước (§Always spec 6.5 — "bật/tắt và mọi lượt soạn là một lượt ghi THẬT
 * rồi dựng lại xem trước"), và điều đó cần gọi lại ĐÚNG `previewImportEncodingFrom{Text,File}`
 * với ĐÚNG văn bản/đường dẫn ban đầu — JS không đọc lại được từ `PendingImportSourceState`
 * (kho đó chỉ sống phía Rust). Đúng MỘT trong hai ô có giá trị tại một thời điểm, khớp
 * `lastSubmittedFrom`.
 */
const pendingText = ref<string | null>(null)
const pendingPath = ref<string | null>(null)

/** Lỗi của lượt CRUD luật làm sạch gần nhất (thêm/sửa/xoá/bật-tắt) — TÁCH khỏi
 * `confirmError` (lỗi của lượt XÁC NHẬN toàn bộ Tác phẩm, ngữ nghĩa khác hẳn). */
const cleanupActionError = ref<IpcError | null>(null)

/**
 * **THÊM (vòng rà 2026-09-06)** — BỐN cờ "đang gửi" RIÊNG, một cho mỗi hành động CRUD luật
 * làm sạch. Trước lượt này, cả bốn hàm mượn `confirming` (cờ của hành động XÁC NHẬN TOÀN BỘ
 * Tác phẩm, ngữ nghĩa khác hẳn) làm lớp chặn DUY NHẤT — `confirming` không bao giờ `true`
 * trong lúc một lượt CRUD đang bay, nên nhấn hai lần liền một nút (Thêm/Lưu/Xoá/tick) gửi
 * HAI lượt IPC chồng nhau, không cổng nào chặn. Khuôn `GlossaryQuickAdd.vue::quickAddSaving`
 * (một cờ khoá nguyên khối `<fieldset>`), nhân bốn vì bốn hành động độc lập nhau.
 */
const cleanupAdding = ref(false)
const cleanupSavingEdit = ref(false)
const cleanupDeleting = ref(false)
const cleanupToggling = ref(false)

/**
 * **THÊM (vòng rà 2026-09-06)** — khoá `${tier}:${id}` của luật đang CHỜ XÁC NHẬN xoá, hoặc
 * `null`. Khuôn `glossaryManageState.ts::deletePendingKey`/`manageDeletePending`: xoá một
 * luật là một nhịp KHÔNG HOÀN TÁC ĐƯỢC (mất một luật người dùng tự soạn, không phải một lượt
 * bật/tắt có thể bấm lại) — nhịp MỘT chỉ đổi khoá này (0 lời gọi IPC), nhịp HAI (bấm lại ĐÚNG
 * luật đó) mới ghi thật. Chọn "xoá"/"sửa" một luật KHÁC, hay một lượt tải lại thành công, làm
 * khoá này tan (xem `deleteImportPreviewCleanupRule`/`onStartEditCleanupRule`).
 */
const cleanupDeletePendingKey = ref<string | null>(null)

/** Buộc dải năm ứng viên MỞ dù tin cậy cao/tự khai — `E` (`EXPERIENCE.md:182`, "mở bộ chọn
 * bảng mã"). Rust LUÔN tính đủ năm bản dựng khi có byte để dò (`ImportEncodingPreview::candidates`),
 * nên buộc mở không đòi một lượt gọi Rust thứ hai — chỉ đổi cờ HIỂN THỊ ở đây. */
const stripForcedOpen = ref(false)

/** Số thứ tự lượt mở — chặn một lượt CŨ ghi đè state của một lượt MỚI hơn, cùng khuôn mọi
 * state Glossary khác. */
let sequence = 0

export const importPreviewIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const importPreviewOpening: DeepReadonly<Ref<boolean>> = readonly(opening)
export const importPreviewStatus: DeepReadonly<Ref<ImportPreviewStatus>> = readonly(status)
export const importPreviewLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const importPreview: DeepReadonly<Ref<ImportEncodingPreview | null>> = readonly(preview)
export const importPreviewSelectedEncoding: DeepReadonly<Ref<string | null>> = readonly(selectedEncoding)
export const importPreviewConfirming: DeepReadonly<Ref<boolean>> = readonly(confirming)
export const importPreviewConfirmError: DeepReadonly<Ref<IpcError | null>> = readonly(confirmError)
export const importPreviewStripForcedOpen: DeepReadonly<Ref<boolean>> = readonly(stripForcedOpen)
/** Nhánh đã mở lượt xem trước đang hiện — đọc bởi `libraryImport.ts::finishImportSubmission`
 * (xem doc-comment [`lastSubmittedFrom`] cho lý do ô này sống ở đây). */
export const importPreviewLastSubmittedFrom: DeepReadonly<Ref<'text' | 'file' | null>> =
  readonly(lastSubmittedFrom)
export const importPreviewCleanupActionError: DeepReadonly<Ref<IpcError | null>> =
  readonly(cleanupActionError)
export const importPreviewCleanupAdding: DeepReadonly<Ref<boolean>> = readonly(cleanupAdding)
export const importPreviewCleanupSavingEdit: DeepReadonly<Ref<boolean>> = readonly(cleanupSavingEdit)
export const importPreviewCleanupDeleting: DeepReadonly<Ref<boolean>> = readonly(cleanupDeleting)
export const importPreviewCleanupToggling: DeepReadonly<Ref<boolean>> = readonly(cleanupToggling)
/** Khoá `${tier}:${id}` của luật đang CHỜ XÁC NHẬN xoá — `.vue` so bằng chuỗi để biết HÀNG
 * nào đang hiện trạng thái "bấm lại để xoá thật" (xem doc-comment `cleanupDeletePendingKey`). */
export const importPreviewCleanupDeletePendingKey: DeepReadonly<Ref<string | null>> =
  readonly(cleanupDeletePendingKey)

/** Dải năm ô mở khi và chỉ khi tin cậy THẤP **hoặc** người dùng đã buộc mở bằng `E` — một
 * điều kiện, một chỗ. Rust luôn cấp đủ dữ liệu (`ImportEncodingPreview::candidates`); đây
 * CHỈ là quyết định HIỂN THỊ. */
export const importPreviewStripIsOpen = computed<boolean>(() => {
  const p = preview.value
  if (p === null || p.candidates.length === 0) return false
  return p.confidence === 'low' || stripForcedOpen.value
})

/** Buộc mở dải — handler của `import.preview.open_picker` (`E`). No-op khi chưa có `preview`
 * (chưa có gì để mở) hoặc khi dải không có ứng viên nào (nhánh tự khai thật). */
export function openImportPreviewCandidatePicker(): void {
  if (preview.value === null || preview.value.candidates.length === 0) return
  stripForcedOpen.value = true
}

/** Ứng viên ĐANG CHỌN — dùng bởi tầng 1 để hiện chip trạng thái + nổi bật đúng ô trong dải. */
export const importPreviewSelectedCandidate = computed<EncodingCandidateWire | null>(() => {
  const p = preview.value
  const id = selectedEncoding.value
  if (p === null || id === null) return null
  return p.candidates.find((c) => c.encoding === id) ?? null
})

/**
 * Bản dựng ĐÃ CHUẨN HOÁ hiện hành, cộng hai số đếm — dùng bởi tầng MỚI (Story 6.4,
 * FR124/FR125). Hai nguồn, ĐÚNG một trong hai đang có mặt tại một thời điểm:
 * - Có ứng viên đang chọn ⇒ đọc `candidate.normalized` (`null` khi ứng viên đó "không ra
 *   chữ", đồng bộ `candidate.preview === null`).
 * - KHÔNG ứng viên nào (đường DÁN VĂN BẢN TAY, `candidates` rỗng) ⇒ đọc
 *   `preview.self_declared_normalized` — vá vòng rà 1, mục 1: cơ chế theo-ứng-viên không
 *   phủ được đường này, và không có nhánh này thì luật gộp dòng chạy mà người dùng không
 *   thấy gì (§Spec Change Log, Vòng rà 1).
 *
 * 🔴 **Đổi ứng viên đổi computed này NGAY, 0 lời gọi IPC** — cùng lý lẽ
 * [`importPreviewSelectedCandidate`]: Rust đã dựng sẵn bản chuẩn hoá của CẢ NĂM ứng viên
 * VÀ của nhánh tự khai trên dây (`§Always` spec 6.4), computed này chỉ ĐỌC lại từ
 * `preview.value` đang có, không gọi gì thêm.
 */
export const importPreviewSelectedNormalized = computed<NormalizedPreviewWire | null>(() => {
  const p = preview.value
  if (p === null) return null
  const candidate = importPreviewSelectedCandidate.value
  if (candidate !== null) return candidate.normalized
  return p.self_declared_normalized
})

/**
 * Khối làm sạch (tầng 3) hiện hành — Story 6.5. CÙNG khuôn
 * [`importPreviewSelectedNormalized`]: đọc `candidate.cleanup` khi có ứng viên đang chọn,
 * rơi về `preview.self_declared_cleanup` khi không (đường dán văn bản tay, 0 ứng viên).
 *
 * 🔴 **Đổi ứng viên đổi computed này NGAY, 0 lời gọi IPC** — Rust đã dựng sẵn khối làm sạch
 * của CẢ NĂM ứng viên VÀ của nhánh tự khai trên dây, computed này chỉ ĐỌC lại.
 */
export const importPreviewSelectedCleanup = computed<CleanupPreviewWire | null>(() => {
  const p = preview.value
  if (p === null) return null
  const candidate = importPreviewSelectedCandidate.value
  if (candidate !== null) return candidate.cleanup
  return p.self_declared_cleanup
})

/**
 * Tầng 2 CHƯA có thân (§Always spec 6.3) — lý do RỖNG kèm tên story chủ, không phải một
 * chuỗi hiển thị (khoá `mode.library.preview.tier_empty_*`, frontend tự `t()`).
 *
 * 🔵 **SỬA 2026-09-05 (Story 6.5) — nhánh `3` đã CHẾT.** Tầng 3 (luật làm sạch) nay CÓ THÂN
 * (xem [`importPreviewSelectedCleanup`]) — chỉ tầng 2 (ranh giới nội dung, Story 6.9) còn
 * rỗng. Tham số hẹp lại còn `2` để TypeScript bắt được mọi chỗ gọi cũ còn truyền `3`.
 */
export function importPreviewEmptyReasonForTier(tier: 2): 'story_6_9' {
  void tier
  return 'story_6_9'
}

type PreviewCall = () => ReturnType<typeof previewImportEncodingFromText>

async function openWith(
  call: PreviewCall,
  from: 'text' | 'file',
  name: string,
  sourceLang: string,
  genre: string,
): Promise<void> {
  if (opening.value) return

  opening.value = true
  sequence += 1
  const mySequence = sequence

  // Chốt NGAY LÚC MỞ — libraryImport.ts::finishImportSubmission đọc lại sau lượt xác nhận,
  // dù mất mấy vòng trượt-rồi-thử-lại ở giữa (xem doc-comment `lastSubmittedFrom`).
  lastSubmittedFrom.value = from
  pendingName.value = name
  pendingSourceLang.value = sourceLang
  pendingGenre.value = genre
  confirming.value = false
  confirmError.value = null
  cleanupActionError.value = null
  cleanupAdding.value = false
  cleanupSavingEdit.value = false
  cleanupDeleting.value = false
  cleanupToggling.value = false
  cleanupDeletePendingKey.value = null
  stripForcedOpen.value = false

  const result = await call()
  if (mySequence !== sequence) return // Một lượt mở/huỷ MỚI đã vượt mặt lượt này.
  opening.value = false
  overlayOpen.value = true

  if (result.error !== null) {
    status.value = 'error'
    loadError.value = result.error
    preview.value = null
    selectedEncoding.value = null
    return
  }
  if (result.preview === null) {
    status.value = 'ipc_unavailable'
    loadError.value = null
    preview.value = null
    selectedEncoding.value = null
    return
  }

  preview.value = result.preview
  selectedEncoding.value = result.preview.selected_encoding
  status.value = 'loaded'
  loadError.value = null
}

/** Mở màn xem trước — nhánh DÁN VĂN BẢN. Gọi từ handler tiêm của `library.import_text`. */
export async function openImportPreviewFromText(
  name: string,
  sourceLang: string,
  genre: string,
  text: string,
): Promise<void> {
  pendingText.value = text
  pendingPath.value = null
  await openWith(() => previewImportEncodingFromText(text, sourceLang), 'text', name, sourceLang, genre)
}

/** Mở màn xem trước — nhánh TỆP. Gọi từ handler tiêm của `library.import_file`. */
export async function openImportPreviewFromFile(
  name: string,
  sourceLang: string,
  genre: string,
  path: string,
): Promise<void> {
  pendingPath.value = path
  pendingText.value = null
  await openWith(() => previewImportEncodingFromFile(path, sourceLang), 'file', name, sourceLang, genre)
}

/**
 * Chọn một ứng viên khác trong dải — KHÔNG gọi Rust (xem doc-comment đầu tệp). `dispatch`
 * không nhận tham số (§Design Notes spec 6.3), nên đây là handler `@click`/`@keydown` của
 * mỗi ô, KHÔNG một command `dispatch('<id>')` — cùng khuôn `onDecisionChange` của
 * `GlossaryImportOverlay.vue`.
 */
export function selectImportPreviewCandidate(encoding: string): void {
  // 🔴 SỬA (vòng rà đối kháng 2, mục 11) — đổi lựa chọn TRONG LÚC một lượt
  // `confirmImportPreview()` đang bay không đổi kết quả IPC đã gửi (tham số đã chốt trước
  // `await`), nhưng làm ô "đang chọn" trên màn hình không còn khớp với bảng mã THẬT SỰ đang
  // được Rust xác nhận — người dùng thấy mình vừa đổi ý trong lúc hệ thống vẫn ghi bằng lựa
  // chọn CŨ. Chặn ở tầng state (không chỉ `:disabled` trên `<input>`, lớp phòng thủ thị
  // giác) để đúng dù đường vào là gì.
  if (confirming.value) return
  if (preview.value === null) return
  if (!preview.value.candidates.some((c) => c.encoding === encoding)) return
  selectedEncoding.value = encoding
}

/**
 * Dựng lại xem trước bằng ĐÚNG nguồn đang treo (`pendingText`/`pendingPath`) — **THÊM
 * (Story 6.5)**, chỗ gọi sản phẩm DUY NHẤT là bốn hàm CRUD luật ngay dưới. Khác `openWith`:
 * KHÔNG đổi `lastSubmittedFrom`/`pendingName`/`pendingSourceLang`/`pendingGenre` (đây là một
 * lượt TẢI LẠI, không phải một lượt MỞ mới), và cố giữ nguyên ứng viên đang chọn nếu nó vẫn
 * còn trong dải mới.
 */
async function reloadImportPreviewAfterRuleChange(): Promise<void> {
  const from = lastSubmittedFrom.value
  if (from === null) return // chưa mở lượt xem trước nào — không có gì để tải lại

  // Một lượt tải lại (dù do THÊM/SỬA/BẬT-TẮT nào gọi tới) làm tan mọi "chờ xác nhận xoá" còn
  // đứng trên MỘT hàng khác — danh sách sắp được dựng lại từ đầu, một khoá cũ trỏ vào một
  // luật có thể đã đổi hình dạng không nên tiếp tục hiện "bấm lại để xoá thật".
  cleanupDeletePendingKey.value = null

  sequence += 1
  const mySequence = sequence
  const keepEncoding = selectedEncoding.value

  const result =
    from === 'text'
      ? await previewImportEncodingFromText(pendingText.value ?? '', pendingSourceLang.value)
      : await previewImportEncodingFromFile(pendingPath.value ?? '', pendingSourceLang.value)
  if (mySequence !== sequence) return // một lượt mở/huỷ/tải lại MỚI đã vượt mặt lượt này

  if (result.error !== null) {
    status.value = 'error'
    loadError.value = result.error
    return
  }
  if (result.preview === null) {
    status.value = 'ipc_unavailable'
    loadError.value = null
    return
  }

  preview.value = result.preview
  selectedEncoding.value = result.preview.candidates.some((c) => c.encoding === keepEncoding)
    ? keepEncoding
    : result.preview.selected_encoding
  status.value = 'loaded'
  loadError.value = null
}

/**
 * Bốn hành động CRUD luật làm sạch — mỗi hành động là một lượt GHI THẬT (§Always spec 6.5)
 * rồi dựng lại xem trước qua [`reloadImportPreviewAfterRuleChange`]. Chặn trong lúc
 * `confirming` (cùng lý do [`selectImportPreviewCandidate`]) — không sửa luật trong khi một
 * lượt xác nhận đang bay — **cộng** cờ "đang gửi" RIÊNG của CHÍNH hành động đó (vòng rà
 * 2026-09-06: bốn cờ tách biệt, không còn mượn `confirming` làm lớp chặn tái vào DUY NHẤT —
 * xem doc-comment [`cleanupAdding`]).
 */
export async function addImportPreviewCleanupRule(
  tier: CleanupRuleTierWire,
  pattern: string,
  kind: CleanupRuleKindWire,
): Promise<void> {
  if (confirming.value || cleanupAdding.value) return
  cleanupAdding.value = true
  try {
    const result = await cleanupAddRule(tier, pattern, kind)
    cleanupActionError.value = result.error
    if (result.error === null) await reloadImportPreviewAfterRuleChange()
  } finally {
    cleanupAdding.value = false
  }
}

export async function editImportPreviewCleanupRule(
  tier: CleanupRuleTierWire,
  id: number,
  pattern: string,
  kind: CleanupRuleKindWire,
): Promise<void> {
  if (confirming.value || cleanupSavingEdit.value) return
  cleanupSavingEdit.value = true
  try {
    const result = await cleanupEditRule(tier, id, pattern, kind)
    cleanupActionError.value = result.error
    if (result.error === null) await reloadImportPreviewAfterRuleChange()
  } finally {
    cleanupSavingEdit.value = false
  }
}

function cleanupRuleKey(tier: CleanupRuleTierWire, id: number): string {
  return `${tier}:${id}`
}

/**
 * Xoá luật `(tier, id)` — **HAI NHỊP** (vòng rà 2026-09-06, khuôn
 * `glossaryManageState.ts::deleteGlossaryManageEntry`): xoá một luật người dùng tự soạn
 * KHÔNG HOÀN TÁC ĐƯỢC, nên nhịp MỘT (khoá `(tier, id)` chưa khớp
 * [`cleanupDeletePendingKey`]) chỉ đổi trạng thái sang "chờ xác nhận" — **0** lời gọi IPC.
 * Nhịp HAI (gọi LẠI với ĐÚNG `(tier, id)` đó) mới ghi thật. Gọi với một `(tier, id)` KHÁC ở
 * nhịp một của luật đó (đổi ý, chọn xoá hàng khác) làm khoá cũ tan — cùng hành vi
 * `deletePendingKey` (một ref, không phải một tập).
 */
export async function deleteImportPreviewCleanupRule(tier: CleanupRuleTierWire, id: number): Promise<void> {
  if (confirming.value || cleanupDeleting.value) return
  const key = cleanupRuleKey(tier, id)
  if (cleanupDeletePendingKey.value !== key) {
    // Nhịp MỘT — chỉ đổi trạng thái, KHÔNG một lượt IPC nào (§Always spec 6.5 áp dụng cho
    // GHI thật; đây chưa phải một lượt ghi).
    cleanupDeletePendingKey.value = key
    cleanupActionError.value = null
    return
  }

  // Nhịp HAI — xác nhận: ghi thật. Tan khoá TRƯỚC khi gọi IPC (cùng lý do
  // `deleteGlossaryManageEntry`: một lượt tải lại/lỗi giữa chừng không được để lại một khoá
  // "chờ xác nhận" mồ côi trỏ vào một luật ĐÃ xoá hoặc đã đổi hình dạng).
  cleanupDeletePendingKey.value = null
  cleanupDeleting.value = true
  try {
    const result = await cleanupDeleteRule(tier, id)
    cleanupActionError.value = result.error
    if (result.error === null) await reloadImportPreviewAfterRuleChange()
  } finally {
    cleanupDeleting.value = false
  }
}

/** Tan trạng thái "chờ xác nhận xoá" mà KHÔNG xoá gì — gọi khi bắt đầu sửa một luật (đổi ý
 * sang một hành động khác trên cùng danh sách). */
export function cancelImportPreviewCleanupDeletePending(): void {
  cleanupDeletePendingKey.value = null
}

export async function toggleImportPreviewCleanupRule(
  tier: CleanupRuleTierWire,
  id: number,
  enabled: boolean,
): Promise<void> {
  if (confirming.value || cleanupToggling.value) return
  cleanupToggling.value = true
  try {
    const result = await cleanupSetEnabled(tier, id, enabled)
    cleanupActionError.value = result.error
    if (result.error === null) await reloadImportPreviewAfterRuleChange()
  } finally {
    cleanupToggling.value = false
  }
}

/**
 * Xác nhận — lệnh `import.preview.confirm`. Thành công ⇒ đóng lớp phủ (Tác phẩm đã ghi).
 * Trượt ⇒ hiện lỗi, lớp phủ Ở LẠI MỞ, ô đang chờ phía Rust GIỮ NGUYÊN — chọn ứng viên khác
 * rồi xác nhận lại không đòi đọc nguồn lần hai (`commands::project::confirm_import_with_encoding`).
 */
export async function confirmImportPreview(): Promise<{ created: CreatedWork | null; error: IpcError | null }> {
  if (confirming.value || preview.value === null || selectedEncoding.value === null) {
    return { created: null, error: null }
  }

  confirming.value = true
  confirmError.value = null
  const mySequence = sequence

  const result = await confirmImportWithEncoding(
    pendingName.value,
    pendingSourceLang.value,
    pendingGenre.value,
    selectedEncoding.value,
  )
  if (mySequence !== sequence) return { created: null, error: null }

  confirming.value = false
  if (result.error !== null) {
    confirmError.value = result.error
    return { created: null, error: result.error }
  }

  // 🔴 SỬA (vòng rà đối kháng 2, mục 8) — bản trước CHỈ đóng lớp phủ (`overlayOpen = false`)
  // ở nhánh thành công, để nguyên `preview`/`selectedEncoding`/`pendingName`/… — state BẨN
  // của lượt VỪA XONG sống tiếp trong bộ nhớ module cho tới lượt `openWith()` KẾ TIẾP (thứ
  // ghi đè `preview.value` mới). Giữa hai thời điểm đó, bất kỳ code nào đọc
  // `importPreview`/`importPreviewSelectedCandidate` (kể cả để debug, kể cả một computed
  // khác lỡ không canh `importPreviewIsOpen`) thấy dữ liệu của Tác phẩm VỪA TẠO dù lớp phủ
  // đã đóng. Dọn ngay tại đây — GIỮ NGUYÊN `lastSubmittedFrom` (đọc SAU bởi
  // `libraryImport.ts::finishImportSubmission`, xem doc-comment của ô đó) và không đụng
  // `sequence` (đã khớp `mySequence`, không cần một số hiệu mới).
  overlayOpen.value = false
  status.value = 'unknown'
  loadError.value = null
  preview.value = null
  selectedEncoding.value = null
  confirmError.value = null
  pendingName.value = ''
  pendingSourceLang.value = ''
  pendingGenre.value = ''
  stripForcedOpen.value = false
  pendingText.value = null
  pendingPath.value = null
  cleanupActionError.value = null
  cleanupAdding.value = false
  cleanupSavingEdit.value = false
  cleanupDeleting.value = false
  cleanupToggling.value = false
  cleanupDeletePendingKey.value = null
  return { created: result.created, error: null }
}

/**
 * Huỷ — lệnh `import.preview.cancel`. **0** lượt gọi Rust: đóng lớp phủ và xoá TOÀN BỘ
 * state (bao gồm `preview`/`selectedEncoding`, nguồn đang chờ ở tầng giao diện) qua
 * [`resetImportPreview`] — điều kiện DUY NHẤT để hàng ma trận I/O "huỷ rồi xác nhận ⇒ 0 Tác
 * phẩm được tạo" đúng (xem doc-comment đầu tệp).
 *
 * 🔴 **NO-OP TRONG LÚC `confirming` (vòng rà đối kháng 2, mục 4) — Tác phẩm MỒ CÔI.** Trước
 * bản vá: huỷ TRONG LÚC một lượt `confirmImportPreview()` đang bay bump `sequence`, nên khi
 * lượt Rust đó về, `mySequence !== sequence` làm nó trả `{created: null, error: null}` — vô
 * hình với `main.ts` (không `finishImportSubmission` nào chạy, panel không reset). NHƯNG
 * phía Rust đã chạy XONG `create_work` + `reindex_library` + `replace_open_work` TRƯỚC đó:
 * `.atproj` nằm trên đĩa thật, `OpenWorkState` đã trỏ vào nó, và giao diện không biết gì —
 * panel vẫn phục vụ Tác phẩm CŨ trong khi Rust đã âm thầm mở một Tác phẩm MỚI.
 *
 * Chặn Ở ĐÂY (hàm mà CẢ HAI đường bấm — nút đóng lẫn `Esc` — cùng đi qua) đóng cửa sổ đua
 * TRIỆT ĐỂ, không phụ thuộc `:disabled`/guard ở tầng `.vue` (những thứ có thể sai lệch khỏi
 * lúc CHÍNH XÁC async đang bay). `ImportPreviewOverlay.vue` disable nút VÀ chặn `Esc` bằng
 * `importPreviewConfirming` như một chỉ báo thị giác — lớp phòng thủ THỨ HAI, không phải lớp
 * duy nhất.
 */
export function cancelImportPreview(): void {
  if (confirming.value) return
  resetImportPreview()
}

/**
 * Vứt toàn bộ state của lớp phủ — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`. Chỗ gọi sản phẩm DUY NHẤT là [`cancelImportPreview`] — cùng khuôn
 * `resetGlossaryImport` (một hàm nuốt-mọi-thứ dùng lại được cho cả "huỷ" lẫn dọn dẹp module
 * khi cần).
 */
export function resetImportPreview(): void {
  sequence += 1
  overlayOpen.value = false
  status.value = 'unknown'
  loadError.value = null
  preview.value = null
  selectedEncoding.value = null
  confirming.value = false
  confirmError.value = null
  opening.value = false
  pendingName.value = ''
  pendingSourceLang.value = ''
  pendingGenre.value = ''
  stripForcedOpen.value = false
  lastSubmittedFrom.value = null
  pendingText.value = null
  pendingPath.value = null
  cleanupActionError.value = null
  cleanupAdding.value = false
  cleanupSavingEdit.value = false
  cleanupDeleting.value = false
  cleanupToggling.value = false
  cleanupDeletePendingKey.value = null
}
