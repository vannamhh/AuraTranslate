/**
 * State + thao tác của form nhập Tác phẩm ở Library — Story 1.15, AC1/AC8/NFR17.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ VÌ SAO MỘT MODULE THUẦN RIÊNG, KHÔNG VIẾT THẲNG TRONG `LibraryMode.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * AD-34 §1 đòi mọi `@click` là **đúng một** `dispatch('<id>')`, và Kiểm B của
 * `scripts/check-commands.mjs` chỉ nhận biết một id là "đã đăng ký" nếu nó đi qua
 * `installCommands()` ở `src/commands/index.ts` — một lượt `commandRegistry.register(...)`
 * gọi TRONG `LibraryMode.vue::onMounted` không bao giờ chạy dưới phép kiểm Node thuần, nên
 * Kiểm B sẽ báo "command chưa đăng ký" trên `dispatch()` mà `.vue` gọi thật.
 *
 * ⇒ Thao tác nộp form (`library.import.submit`) đăng ký ở
 * `src/commands/index.ts` như một `CommandDeps` TIÊM VÀO — cùng khuôn `applyPreset` /
 * `togglePanel` của `src/layout/dockController.ts`. Module này là **phía cung cấp**:
 * `LibraryMode.vue` đọc/ghi các `ref` ở đây qua `v-model`; `src/main.ts` nối
 * `submitActiveImportTab` vào `installCommands({...})`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 SỬA 2026-09-04 (Story 6.3, FR126) — nộp form KHÔNG còn tạo Tác phẩm trực tiếp.
 * ─────────────────────────────────────────────────────────────────────────────
 * `submitPastedText`/`submitFilePath` giờ mở màn xem trước bảng mã
 * (`src/importPreviewState.ts::openImportPreviewFromText`/`openImportPreviewFromFile`) —
 * đúng bất biến "không byte nào xuống đĩa trước khi người dùng xác nhận"
 * (`libraryImport.ts:276-330` cũ, nay ÁP CẢ cho nhánh dán văn bản, không chỉ nhánh tệp).
 * Việc TẠO Tác phẩm thật (`create_work`) chỉ xảy ra sau khi người dùng bấm "Xác nhận" trong
 * `ImportPreviewOverlay.vue`, đi qua `confirmImportPreview()` rồi tới
 * [`finishImportSubmission`] (đổi tên từ `finishSubmit` cũ, giờ EXPORT vì `main.ts` gọi nó
 * SAU khi lượt xác nhận — không còn ngay trong `submitPastedText`/`submitFilePath` nữa).
 */
import { computed, nextTick, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { enterFocus } from '../commands'
import {
  importPreviewIsOpen,
  takeImportPreviewUndetectableLangError,
  importPreviewLastSubmittedFrom,
  openImportPreviewFromFile,
  openImportPreviewFromText,
  openImportPreviewFromUrls,
} from '../importPreviewState'
import {
  bilingualImportPreviewIsOpen,
  cancelBilingualImportPreview,
  openBilingualImportPreview,
} from '../bilingualImportPreviewState'
import { ensureChapterLoaded, resetSourcePanel } from '../panels/sourcePanelState'
import { resetLookupPanel } from '../panels/lookupPanelState'
import {
  ensureSegmentsLoaded,
  flushChapterPositionNow,
  flushEditorNow,
  resetEditorPanel,
} from '../panels/editorPanelState'
import { resetSegmentHistory } from '../panels/segmentHistoryState'
// Story 5.11 — Chế độ đọc mang cùng lớp cache module-level của Tác phẩm đang mở, cùng lý
// lẽ `resetEditorPanel` đã ghi ở `finishSubmit` bên dưới.
import { resetReading, resetReadingToc } from './readingState'
// 🔴 Story 4.7 loop 2, finding P4 — cùng khuyết tật và cùng bản sửa đã đặt ở
// `libraryChapters.ts::openWorkById` (xem doc-comment tại chỗ `import` ở đó): một Tác phẩm MỚI
// TẠO (nhập) đi qua ĐIỂM NGHẼN này, không qua `openWorkById`, nên bản ghi prompt của Tác phẩm
// CŨ (nếu còn mở trước lượt nhập) phải bị vứt Ở ĐÂY riêng — không "rải" một lời gọi thứ hai,
// nhưng cũng không được bỏ sót MỘT trong hai điểm nghẽn thật.
import { resetAiPromptInspector } from '../aiPromptInspectorState'
// 🔴 Story 4.8 — cùng khuyết tật và cùng bản sửa ngay trên: kết quả dịch cũng mang danh tính
// THEO Tác phẩm, và cũng phải bị vứt ở đúng điểm nghẽn này.
import { resetAiTranslate } from '../aiTranslateState'
// 🔴 Story 4.9, Phase 3 — cùng khuyết tật và cùng bản sửa ngay trên: kết quả của một LÔ cũng
// mang danh tính THEO Tác phẩm. Xem doc-comment đầu `aiTranslateBatchState.ts`.
import { resetAiTranslateBatch } from '../aiTranslateBatchState'
import { resetProofread } from '../proofreadState'
// 🔴 Story 4.9 — vùng chọn nhiều-segment mang hai `segment.id` (`anchor`/`focus`), cùng
// khuyết tật và cùng bản sửa ngay trên. Xem doc-comment đầu `panels/segmentSelectionState.ts`.
import { resetSegmentSelection } from '../panels/segmentSelectionState'
import { resetReviewMode } from '../reviewModeState'
import { resetGlossaryQueue } from '../glossaryQueueState'
import { resetAlignment } from '../alignmentState'
import { resetConventionManage } from '../conventionManageState'
import { resetConventionImport } from '../conventionImportState'
import { resetReviewerImport } from '../reviewerImportState'
import { setOpenWork } from '../openWorkIdentity'
import { restorePromptSetForWork } from '../promptSetState'
import type { CreatedWork } from '../config/project'
import { classifyImportPaths, listLibraryWorks, pickImportFiles } from '../config/library'
import type { ClassifiedImportFile, ImportBatchVerdict, ImportClassification, WorkRow } from '../config/library'
import type { IpcError } from '../i18n'
import { closeLibraryImportOverlay } from './libraryImportOverlayState'

/** Khớp `src-tauri/src/lib.rs::DRAG_DROP_EVENT`. */
const DRAG_DROP_EVENT = 'aura://file-dropped'
/** Khớp `src-tauri/src/lib.rs::DRAG_ENTER_EVENT`. */
const DRAG_ENTER_EVENT = 'aura://file-drag-enter'
/** Khớp `src-tauri/src/lib.rs::DRAG_LEAVE_EVENT`. */
const DRAG_LEAVE_EVENT = 'aura://file-drag-leave'

/**
 * Vùng kéo-thả đang có một thao tác kéo lơ lửng trên nó.
 *
 * ⚠️ Bật/tắt bằng **event của Rust**, không bằng `dragenter`/`dragleave` của DOM: với
 * `drag_drop_enabled = true` (mặc định Tauri v2), webview **không bao giờ** nhận được
 * các sự kiện DOM đó — xem `src-tauri/src/lib.rs::DRAG_ENTER_EVENT`.
 */
export const isDragOver = ref(false)

/** Files in the drag that is hovering the window; `0` when the platform reports none. */
export const dragFileCount = ref(0)

let dragDropWired = false
let unlisteners: UnlistenFn[] = []

/** Tên Tác phẩm — dùng cho cả hai nhánh (AC1). */
export const name = ref('')

/**
 * Ngôn ngữ nguồn — **bất biến sau khi tạo** (FR3, AD-18). Hai mã khớp `core::matching`
 * (`lang = 'zh'` / `lang = 'en'`, xem `tests/dict_boundary.rs`); `'auto'` chỉ sống tới lượt xem
 * trước đầu, nơi Rust đoán ra `zh`/`en`.
 */
export const sourceLang = ref<'zh' | 'en' | 'auto'>('auto')

/** Thể loại — tự do, rỗng hợp lệ. */
export const genre = ref('')

// ═══════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 6.7b (FR122 nửa hai) — "ĐƯA VÀO": Tác phẩm mới, hoặc thêm Chương vào cuối một
// Tác phẩm sẵn có. Khớp mockup `web-import.html:198-207` — radio + picker sống CẠNH
// `name`/`sourceLang`/`genre`, cùng một cột, vì đích phải chốt TRƯỚC lượt gọi
// `openImportPreviewFromText`/`…FromFile`/`…FromUrls`: `sourceLang` gửi ở NGAY LƯỢT MỞ ĐẦU
// TIÊN đã phải là ngôn ngữ CỦA ĐÍCH khi đích là một Tác phẩm sẵn có (Decision 2 — "adopted,
// không dò"), và nhánh URL không có lệnh Rust nào cho phép đổi đích SAU lượt `start_url_import`
// đầu (xem doc-comment `startUrlImport` ở `config/project.ts`) — nên đích PHẢI chốt Ở ĐÂY,
// không phải một điều khiển sống bên trong `ImportPreviewOverlay.vue` sau khi đã mở.
// ═══════════════════════════════════════════════════════════════════════════════

/** Chế độ đích đang chọn — `'new'` (mặc định, hành vi hôm nay không đổi) hoặc `'existing'`
 * (đã chọn radio "Tác phẩm đã có" nhưng CÓ THỂ chưa chọn Tác phẩm nào trong picker —
 * [`destinationWorkId`] mới là đích THẬT SỰ sẽ gửi đi). */
export const destinationMode = ref<'new' | 'existing'>('new')

/** Đích đã CHỌN trong picker — `null` khi `destinationMode === 'new'` HOẶC khi đang ở
 * `'existing'` mà chưa chọn Tác phẩm nào (picker còn ở mục giữ chỗ). Đây là giá trị THẬT gửi
 * cho `openImportPreviewFromText`/`…FromFile`/`…FromUrls` (`config/project.ts` gọi nó
 * `destinationWorkId`, khớp `destination: Option<String>` phía Rust). */
export const destinationWorkId = ref<string | null>(null)

/** Toàn bộ Library, KHÔNG lọc — cùng lời gọi `listLibraryWorks()` mà `libraryWorks.ts` dùng
 * cho lưới, nhưng KHÔNG tái dùng `libraryWorks.ts::libraryWorks` (ref đó đã bị lọc theo bộ
 * lọc trạng thái/lĩnh vực/ngôn ngữ đang chọn trên lưới — dùng nó cho picker này sẽ làm một
 * Tác phẩm biến mất khỏi danh sách đích chỉ vì người dùng đang lọc lưới theo một tiêu chí
 * không liên quan, đúng lớp lỗi AD-1 cấm cho `genres`/`source_langs`). */
export const destinationWorks = ref<WorkRow[]>([])

/** Đã tải picker lần nào trong phiên này chưa — cùng lý do `worksHaveLoaded` của
 * `libraryWorks.ts`: "Library trống" chỉ được nói SAU một lượt tải, không phải trước nó. */
export const destinationWorksHaveLoaded = ref(false)

/** Lỗi hạ tầng của lượt tải picker gần nhất — cùng khuôn `lastError` (tách riêng vì đây là
 * lỗi của MỘT lượt đọc phụ, không phải lỗi của lượt nộp form). */
export const destinationWorksError = ref<IpcError | null>(null)

/** Tác phẩm ĐANG CHỌN trong picker, tra trong [`destinationWorks`] — `null` khi
 * `destinationWorkId === null`. */
export const pickedDestinationWork = computed<WorkRow | null>(() => {
  if (destinationWorkId.value === null) return null
  return destinationWorks.value.find((w) => w.work_id === destinationWorkId.value) ?? null
})

/** Ngôn ngữ nguồn THẬT SỰ gửi đi — Decision 2 (spec 6.7b): **được kế thừa** từ đích khi đích
 * là một Tác phẩm sẵn có (không dò, không hỏi lại — `work.source_lang` bất biến sau khi tạo,
 * §Never), giá trị gõ tay [`sourceLang`] chỉ dùng khi đích là Tác phẩm mới. */
export const effectiveSourceLang = computed<string>(() => pickedDestinationWork.value?.source_lang ?? sourceLang.value)

/** Tên Tác phẩm THẬT SỰ hiện/gửi — §Never spec 6.7b: không đổi tên của đích, nên khi đích là
 * một Tác phẩm sẵn có, đây là tên CỦA ĐÍCH — ô gõ tay [`name`] bị bỏ qua (Rust cũng bỏ qua nó
 * ở đường APPEND, xem Implementation Notes Phase 2 spec 6.7b). */
export const effectiveName = computed<string>(() => {
  const picked = pickedDestinationWork.value
  if (picked !== null) return picked.name
  if (name.value.trim() === '' && destinationMode.value === 'new') return importAutoName.value
  return name.value
})

/** Thể loại THẬT SỰ hiện/gửi — cùng lý do [`effectiveName`]. */
export const effectiveGenre = computed<string>(() => pickedDestinationWork.value?.genre ?? genre.value)

/** Radio "Tác phẩm đã có" chọn được hay không — I/O Matrix spec 6.7b "Empty Library": Library
 * rỗng ⇒ chỉ còn Tác phẩm mới, và điều khiển phải NÓI vì sao (không chỉ khoá im lặng). */
export const destinationExistingWorkAvailable = computed<boolean>(
  () => !destinationWorksHaveLoaded.value || destinationWorks.value.length > 0,
)

/** **THÊM (vòng rà, mục E6)** — Library RỖNG thật (đã tải xong, 0 Tác phẩm, KHÔNG có lỗi) —
 * tách khỏi [`destinationExistingWorkAvailable`]: predicate đó dùng CHUNG cho `:disabled` của
 * radio (nơi "rỗng" và "lỗi" đều hợp lý khoá lại) lẫn cho hint "Library trống", nên một lượt
 * tải LỖI (danh sách rơi về `[]`, `destinationWorksHaveLoaded = true`) khiến CẢ hai câu — "Library
 * trống" và câu lỗi thật — cùng hiện một lúc, nói dối về lý do. Hint "Library trống" chỉ được
 * hiện khi ĐÃ tải xong, danh sách thật sự rỗng, VÀ không có lỗi — một danh sách rỗng và một
 * lượt tải trượt phải nói hai câu khác nhau. */
export const destinationLibraryGenuinelyEmpty = computed<boolean>(
  () =>
    destinationWorksHaveLoaded.value &&
    destinationWorks.value.length === 0 &&
    destinationWorksError.value === null,
)

/** Tải (hoặc tải lại) danh sách Tác phẩm cho picker — gọi khi `LibraryMode.vue` mount, khi
 * người dùng chọn radio "Tác phẩm đã có" (lần đầu, hoặc lại sau một lượt tải trượt — mục E7),
 * và sau một lượt nhập thành công (mục B5, xem [`finishImportSubmission`]) — best-effort, cùng
 * triết lý mọi lượt đọc phụ khác trong module này — một lượt trượt không được chặn form chính. */
export async function loadDestinationWorks(): Promise<void> {
  const { report, error } = await listLibraryWorks()
  if (error !== null) {
    destinationWorksError.value = error
    destinationWorksHaveLoaded.value = true
    return
  }
  destinationWorks.value = report?.works ?? []
  destinationWorksError.value = null
  destinationWorksHaveLoaded.value = true
}

/** Đổi chế độ đích — gọi từ `@change` của hai radio (LibraryMode.vue), KHÔNG một `@click` kèm
 * tham số (§Never spec 6.7b: "reached by Tab, committed through `@change`/`@submit`"). Quay
 * về `'new'` xoá LUÔN Tác phẩm đã chọn — không để một `work_id` cũ sống sót dưới một radio
 * đã đổi ý. */
export function setDestinationMode(mode: 'new' | 'existing'): void {
  destinationIsPreset = false
  destinationMode.value = mode
  if (mode === 'new') {
    destinationWorkId.value = null
    return
  }
  // 🔵 SỬA (vòng rà, mục E7) — trước bản sửa này chỉ tải khi CHƯA từng tải
  // (`!destinationWorksHaveLoaded`), nên một lượt tải TRƯỢT đặt `destinationWorksHaveLoaded =
  // true` khoá picker im lặng cho hết phiên: không lối vào nào gọi lại `loadDestinationWorks()`
  // nữa. Thử lại khi CHƯA từng tải, HOẶC lượt gần nhất đã lỗi.
  if (!destinationWorksHaveLoaded.value || destinationWorksError.value !== null) void loadDestinationWorks()
}

/** Chọn một Tác phẩm trong picker — gọi từ `@change` của `<select>`. */
export function setDestinationWorkId(workId: string): void {
  destinationIsPreset = false
  destinationWorkId.value = workId
}

let destinationIsPreset = false

/** Sets the destination for one opening of the overlay: the next plain open resets it. */
export function presetImportDestination(workId: string | null): void {
  destinationMode.value = 'existing'
  destinationWorkId.value = workId
  destinationIsPreset = true
}

/** Undoes a preset left by the previous opening; a destination the user picked by hand stays. */
export function resetPresetImportDestination(): void {
  if (!destinationIsPreset) return
  destinationIsPreset = false
  destinationMode.value = 'new'
  destinationWorkId.value = null
}

/** Nội dung ô dán văn bản (AC1 nhánh a). */
export const pastedText = ref('')

export type ImportTab = 'paste' | 'files' | 'urls'

/** The selected source tab. Lives only as long as the app process, so a fresh start opens on Dán. */
export const importActiveTab = ref<ImportTab>('paste')

export function selectImportTab(tab: ImportTab): void {
  importActiveTab.value = tab
}

const importClassification = ref<ImportClassification | null>(null)

/** `.docx` paths whose row carries the temporary *Đọc như song ngữ* toggle switched on. */
export const docxAsBilingual = ref<string[]>([])

export const importFiles = computed<ClassifiedImportFile[]>(() => importClassification.value?.files ?? [])
export const importFilesVerdict = computed<ImportBatchVerdict>(() => importClassification.value?.verdict ?? 'empty')

/** Name shown while the Name field is empty; follows the selected tab. The URL tab has none
 * (the first Chapter title fills it in the preview). */
export const importAutoName = computed<string>(() => {
  switch (importActiveTab.value) {
    case 'paste':
      return (
        pastedText.value
          .split(/\r\n|\r|\n/)
          .map((line) => line.trim())
          .find((line) => line !== '') ?? ''
      )
    case 'files':
      return importFiles.value[0]?.stem ?? ''
    case 'urls':
      return ''
  }
})

/** Rust's verdict is the only judge; this only names the sentence that explains a locked button. */
export const importFilesLockKey = computed<string | null>(() => {
  switch (importFilesVerdict.value) {
    case 'has_unsupported':
      return 'mode.library.files_locked_unsupported'
    case 'mixed_kinds':
      return 'mode.library.files_locked_mixed'
    case 'multiple_bilingual':
      return 'mode.library.files_locked_one_bilingual'
    default:
      return null
  }
})

let classifySequence = 0

async function applyImportFilePaths(paths: string[]): Promise<void> {
  classifySequence += 1
  const mine = classifySequence
  if (paths.length === 0) {
    importClassification.value = null
    docxAsBilingual.value = []
    return
  }
  const { classification, error } = await classifyImportPaths(paths, docxAsBilingual.value)
  if (mine !== classifySequence) return
  if (error !== null) {
    lastError.value = error
    return
  }
  if (classification !== null) importClassification.value = classification
}

/** The dialog and a drop both end here, so they cannot classify differently. */
function setImportClassification(classification: ImportClassification): void {
  classifySequence += 1
  importClassification.value = classification
}

function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path
}

export function importFileName(path: string): string {
  return baseName(path)
}

export async function chooseImportFiles(): Promise<void> {
  const { classification, error } = await pickImportFiles(docxAsBilingual.value)
  if (error !== null) {
    lastError.value = error
    return
  }
  if (classification === null) return
  lastError.value = null
  noticeKey.value = null
  const known = importFiles.value.map((f) => f.path)
  const added = classification.files.map((f) => f.path).filter((path) => !known.includes(path))
  if (known.length === 0) {
    setImportClassification(classification)
    return
  }
  await applyImportFilePaths([...known, ...added])
}

/** The row the pointer or focus is on; `library.import.file_remove` acts on it. */
const aimedFilePath = ref<string | null>(null)

export function aimImportFile(path: string): void {
  aimedFilePath.value = path
}

export async function removeAimedImportFile(): Promise<void> {
  const target = aimedFilePath.value
  aimedFilePath.value = null
  if (target === null) return
  docxAsBilingual.value = docxAsBilingual.value.filter((p) => p !== target)
  await applyImportFilePaths(importFiles.value.map((f) => f.path).filter((p) => p !== target))
}

/** `@change` of the temporary *Đọc như song ngữ* toggle on a `.docx` row. */
export async function setDocxReadAsBilingual(path: string, on: boolean): Promise<void> {
  docxAsBilingual.value = on
    ? [...docxAsBilingual.value.filter((p) => p !== path), path]
    : docxAsBilingual.value.filter((p) => p !== path)
  await applyImportFilePaths(importFiles.value.map((f) => f.path))
}

export function resetImportFiles(): void {
  classifySequence += 1
  importClassification.value = null
  docxAsBilingual.value = []
  aimedFilePath.value = null
}

/**
 * **THÊM (Story 6.7, FR122)** — nội dung ô dán danh sách URL, MỖI DÒNG một link.
 *
 * 🔴 **Đây là NGUỒN DUY NHẤT của hai con số "N link · sẽ tạo N Chương"** — [`pastedUrlLines`]/
 * [`pastedUrlCount`] tính CỤC BỘ trên ref này, JS thuần, **0 IPC**. Đây CHÍNH LÀ đối chứng
 * cấu tạo cho AC "chưa bấm nút ⇒ 0 lời gọi mạng, 0 lời gọi IPC": không có đường mã nào ở đây
 * gọi `invoke(...)`, nên không cách nào hai con số này "vô tình" kích hoạt một lượt gọi.
 */
export const pastedUrls = ref('')

/**
 * Danh sách URL đã TRIM, bỏ dòng rỗng — cùng phép lọc mà Rust
 * (`commands::project::fetch_url_import_items`) áp lại LẦN THỨ HAI khi lệnh thật chạy (phòng
 * thủ kép, không phải kỳ vọng trùng lặp công việc — xem doc-comment `startUrlImport` ở
 * `config/project.ts`).
 *
 * 🔵 **SỬA 2026-09-07 (vòng rà bước 4) — câu "cùng phép lọc" ở trên từng SAI, đo được.**
 * `str::trim()` của Rust không cắt `U+FEFF` (BOM) trong khi `String.prototype.trim()` của JS
 * có cắt, nên hai bên đếm khác nhau đúng ở ký tự mà người dán từ Windows/Excel/trang web hay
 * mang theo. Rust nay cắt qua `commands::project::trim_like_the_paste_box` — xem doc-comment
 * của hàm đó cho phép đo và hai ca hỏng. Sửa Ở PHÍA RUST có chủ ý: con số người dùng NHÌN
 * THẤY là con số phải đúng, nên nó là chuẩn, không phải thứ đi chỉnh cho khớp máy. Không phân biệt "URL hợp lệ hay không" — đó là việc của Rust khi
 * lệnh thật chạy; ở đây chỉ ĐẾM, không TẢI.
 */
export const pastedUrlLines = computed<string[]>(() =>
  pastedUrls.value
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line !== ''),
)

/** Hai con số hiển thị TRƯỚC khi bấm nút — "N link · sẽ tạo N Chương" (cùng N, khuôn mockup
 * `web-import.html:219`, "màn hình tự tố cáo": ai đó thêm một đường tự quét mục lục thì hai
 * con số lệch nhau, không ai phải nhớ đi kiểm). */
export const pastedUrlCount = computed<number>(() => pastedUrlLines.value.length)

/** Distinct hosts among the pasted lines that parse as URLs; display only. */
export const pastedUrlDomainCount = computed<number>(() => {
  const hosts = new Set<string>()
  for (const line of pastedUrlLines.value) {
    try {
      hosts.add(new URL(line).hostname)
    } catch {
      continue
    }
  }
  return hosts.size
})

/** Đang có một lượt gọi IPC dở dang — vô hiệu hoá nút trong lúc chờ. */
export const busy = ref(false)

/** Lỗi gần nhất Rust trả lời, để `LibraryMode.vue` vẽ bằng `tError()`. */
export const lastError = ref<IpcError | null>(null)

/**
 * Tác phẩm vừa tạo thành công gần nhất — một **dòng xác nhận**, không phải một màn hình
 * thay thế.
 *
 * 🔴 `LibraryMode.vue` vẽ nó **bên cạnh** form, **KHÔNG** trong một nhánh `v-else` bọc
 * lấy form. Bản trước làm đúng chuyện đó và ref này không bao giờ được đặt lại, nên tạo
 * xong Tác phẩm đầu tiên là form **và** dải báo lỗi biến mất vĩnh viễn trong phiên — không
 * không tạo được Tác phẩm thứ hai, không thấy được lỗi nào nữa. Code review 2026-08-06.
 */
export const createdWork = ref<CreatedWork | null>(null)

/**
 * Khoá i18n của một lời nhắn không phải lỗi (ví dụ: thả nhiều tệp một lúc).
 *
 * Giữ **KHOÁ**, không giữ CÂU — NFR16/AD-21: không một câu tiếng Việt nào sống
 * trong mã.
 */
export const noticeKey = ref<string | null>(null)


/**
 * Lượt flush bản dịch cũ trượt ⇒ **dừng** lượt tạo Tác phẩm. Story 2.3 · code review 2026-08-13.
 *
 * ⚠️ Dựng ở frontend, cùng khuôn `UNKNOWN_IPC_ERROR` của `config/segment.ts`: đây là một phán
 * quyết của **tầng giao diện** *(“đừng thay Tác phẩm khi chữ cũ chưa an toàn”)*, không phải một
 * lỗi Rust trả về ⇒ nó **không** có `MessageKey` phía Rust, và không được có. AD-21 vẫn đứng:
 * chỗ này mang một **khoá**, không mang một câu.
 */
const FLUSH_FAILED_ERROR: IpcError = {
  code: 'editor.flush_failed',
  message_key: 'err.editor.flush_failed',
  params: {},
  retryable: true,
}

const BILINGUAL_NEEDS_SOURCE_LANG_ERROR: IpcError = {
  code: 'import.bilingual_needs_source_lang',
  message_key: 'err.import.bilingual_needs_source_lang',
  params: {},
  retryable: false,
}

function surfaceUndetectableLang(): void {
  const error = takeImportPreviewUndetectableLangError()
  if (error !== null) lastError.value = error
}

/** @returns `false` ⇒ lượt nộp bị DỪNG, chỗ gọi không được đi tiếp. */
async function beginSubmit(): Promise<boolean> {
  busy.value = true
  lastError.value = null
  noticeKey.value = null
  // Một lượt nộp mới xoá xác nhận cũ — không để dòng "Đã tạo…" của lần trước đứng cạnh
  // kết quả của lần này và nói dối về cái vừa xảy ra.
  createdWork.value = null

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 FLUSH BẢN DỊCH CHƯA LƯU — Story 2.3 · AC3 vế *"đóng Tác phẩm"* · AD-35
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // Hôm nay **không tồn tại** một lệnh đóng `.atproj`, nên chỗ duy nhất mà vế *"đóng Tác
  // phẩm"* của AC3 chạm tới được là đường này: một Tác phẩm mới **thay** Tác phẩm đang mở.
  //
  // 🔴 **VÌ SAO Ở ĐÂY VÀ KHÔNG Ở `finishSubmit`.** Story ghi *"flush TRƯỚC
  // `resetEditorPanel()`"*, và `resetEditorPanel()` sống trong `finishSubmit`. Nhưng đo lại
  // đường thật thì flush ở đó **SAI**, và sai theo hướng tệ nhất: lúc `finishSubmit` chạy,
  // `create_work_from_*` đã trả về, tức `replace_open_work` phía Rust **đã** trỏ
  // `OpenWorkState` sang Tác phẩm MỚI. Một lô flush ở đó mang `chapter_id` của Tác phẩm **cũ**
  // đi vào `project.db` của Tác phẩm **mới** ⇒ Rust từ chối trọn lô bằng
  // `segment.unknown_ids` *(id không thuộc Chương nào ở đó)*, và bản dịch cũ mất **im lặng**.
  //
  // ⇒ Điều kiện thật của mệnh đề không phải *"trước `resetEditorPanel()`"* mà là **trước lượt
  // `replace_open_work`**. `beginSubmit` là điểm nghẽn duy nhất mà **cả hai** nhánh nhập đi
  // qua trước lời gọi đó — cùng vai mà `finishSubmit` giữ cho nửa sau.
  //
  // ⚠️ `await`, không `void`: lượt tạo Tác phẩm phải chờ bản dịch cũ chạm WAL trước khi
  // `OpenWorkState` đổi chỗ. Bỏ `await` là dựng lại đúng cuộc đua vừa mô tả.
  //
  // 🔴 **Và kết quả PHẢI được đọc** — code review 2026-08-13, Ice ký. Bản cũ gọi `await` rồi
  // bỏ giá trị trả về, nên một lượt flush **trượt** vẫn cho lượt tạo Tác phẩm đi tiếp; và khi
  // nó thành công thì `finishSubmit` gọi `resetEditorPanel()` → `flush.reset()`, hàm **vứt vô
  // điều kiện** mọi mục còn trong tập chờ. ⇒ bản dịch chưa ghi của Tác phẩm cũ mất **im lặng**,
  // đúng cửa sổ mà lượt dời lời gọi lên `beginSubmit` vừa đóng ở nửa kia.
  //
  // Phán quyết: **chặn**. Người dùng bị cản một lượt và thấy lý do; họ thử lại, hoặc họ chép
  // bản dịch ra ngoài. Cả hai đường đều tốt hơn một lượt mất chữ không ai biết.
  const flushed = await flushEditorNow()
  if (flushed === 'failed') {
    lastError.value = FLUSH_FAILED_ERROR
    busy.value = false
    return false
  }

  // 🔵 THÊM Story 5.7 (AC4/AC6) — vị trí làm việc đang chờ ghi cũng thuộc Tác phẩm đang mở
  // (CŨ), và `resetEditorPanel()` bên trong `finishSubmit` sẽ VỨT nó không ghi. Cùng biên với
  // `flushEditorNow()` ngay trên, nhưng KHÔNG chặn lượt nộp: mất vị trí chỉ mất một lời nhắc,
  // không mất bản dịch (§I/O Matrix "Ghi vị trí": "Lỗi ghi ⇒ chẩn đoán, KHÔNG hộp thoại").
  await flushChapterPositionNow()

  return true
}

/**
 * Đóng vòng nộp form — gọi SAU khi lượt xác nhận bảng mã trả về (`main.ts`, handler của
 * `import.preview.confirm`). 🔵 **ĐỔI TÊN 2026-09-04 (Story 6.3)** từ `finishSubmit` (trước
 * đây `private`, gọi ngay trong `submitPastedText`/`submitFilePath`) — EXPORT vì lượt tạo
 * Tác phẩm thật giờ xảy ra ở `confirmImportPreview()` (`src/importPreviewState.ts`), một
 * lượt async tách biệt khỏi hàm nộp form ban đầu; `busy` KHÔNG còn được hạ ở đây (nó đã hạ
 * ngay sau khi màn xem trước MỞ — xem `submitPastedText`), thân hàm còn lại giữ NGUYÊN VẸN.
 */
export function finishImportSubmission(created: CreatedWork | null, error: IpcError | null): void {
  lastError.value = error
  createdWork.value = created
  if (created !== null) closeLibraryImportOverlay()

  // 🔴 Tác phẩm MỚI ⇒ vứt state Panel Source VÀ Panel Lookup của Tác phẩm CŨ.
  // `replace_open_work` phía Rust vừa trỏ `OpenWorkState` sang chỗ khác, còn cờ
  // module-level của các panel thì không hay biết — bỏ dòng này là để người dùng đọc nội
  // dung Tác phẩm A dưới nhãn Tác phẩm B. Đây là điểm nghẽn DUY NHẤT mà cả hai nhánh nhập
  // đều đi qua (xem `resetSourcePanel`/`resetLookupPanel`) — Story 1.17 CÙNG LƯỢT, không
  // một lời gọi thứ hai rải ra.
  if (created !== null) {
    setOpenWork(created.meta.work_id, created.meta.name)
    void restorePromptSetForWork(created.meta.genre)
    resetSourcePanel()
    resetLookupPanel()
    // 🔴 Story 2.2 — Panel Editor mang **cùng** lớp cache module-level, nên nó phải đi cùng
    // lượt vứt này. Bỏ nó ra là để Editor hiện segment của Tác phẩm A dưới nhãn Tác phẩm B,
    // đúng đường hỏng mà `resetSourcePanel` đã ghi lại từ code review 2026-08-06.
    resetEditorPanel()
    // 🔵 THÊM Story 5.11 — Chế độ đọc mang cùng lớp cache module-level, cùng lý do dòng trên.
    resetReading()
    resetReadingToc()
    // 🔴 THÊM Story 4.7 loop 2, finding P4 — cùng lý lẽ dòng trên, bản ghi prompt cũng mang
    // danh tính THEO TÁC PHẨM; xem doc-comment tại chỗ `import`.
    resetAiPromptInspector()
    // 🔴 THÊM Story 4.8 — cùng lý lẽ dòng trên, kết quả dịch cũng mang danh tính THEO TÁC PHẨM.
    resetAiTranslate()
    // 🔴 THÊM Story 4.9, Phase 3 — cùng lý lẽ dòng trên, kết quả của một LÔ cũng mang danh
    // tính THEO TÁC PHẨM. Xem doc-comment tại chỗ `import`.
    resetAiTranslateBatch()
    resetProofread()
    // 🔴 THÊM Story 4.9 — vùng chọn nhiều-segment mang hai `segment.id` (`anchor`/`focus`),
    // cùng lý lẽ dòng trên. Xem doc-comment đầu `panels/segmentSelectionState.ts`.
    resetSegmentSelection()
    resetReviewMode()
    resetGlossaryQueue()
    resetAlignment()
    resetReviewerImport()
    // The convention rules are global, but the manage overlay's open flag, error and drafts
    // belong to the session that opened them, so they go with this batch.
    resetConventionManage()
    resetConventionImport()

    // ═══════════════════════════════════════════════════════════════════════════════
    // 🔵 CODE REVIEW BA TẦNG 2026-08-19 — HÀM ĐÃ VIẾT Ở STORY 2.12 MÀ CHƯA NỐI DÂY
    // ═══════════════════════════════════════════════════════════════════════════════
    // 🔴 **Quyết định #4 của Story 2.12 *(đường (a), "0 dòng mã sản phẩm mới")* HẾT ĐÚNG từ
    // 2026-08-19** — sửa tại chỗ thay vì để nó lặng lẽ sai. Lượt rà ba tầng bắt được:
    // `resetSegmentHistory()` được viết ở `segmentHistoryState.ts:387` kèm một doc-comment mô
    // tả đúng kịch bản hỏng dưới đây, rồi **không một lời gọi nào** trong `src/**`. Chỗ gọi duy
    // nhất trong kho là bảng fixture `e2e/support/panelReset.mjs:64` — tức bộ đo dọn state mà
    // sản phẩm thì không, và cổng `check:panel-refs` vẫn XANH vì nó chỉ hỏi *"tệp có một hàm
    // `reset*` nào không"*, không hỏi **ai gọi** hàm đó.
    //
    // ⚠️ Kịch bản, nguyên văn từ doc-comment của chính hàm ấy: mở lịch sử phiên bản trên Tác
    // phẩm A → tạo Tác phẩm B mà **không** đóng hộp thoại → `isOpen` còn `true` và `segmentId`
    // còn trỏ vào một `segment.id` của A. `restoreVersion()` đọc thẳng `segmentId.value`
    // *(`segmentHistoryState.ts:274`)* và **không** đối chiếu Tác phẩm đang mở. Hai kho đánh số
    // `segment.id` **ĐỘC LẬP**, nên id ấy tồn tại thật ở kho mới và trỏ vào một câu khác hẳn.
    // ⇒ Một lượt khôi phục FR101 ghi bản dịch của câu này đè lên câu kia, và *"chỗ hỏng là
    // VĨNH VIỄN"*.
    //
    // 🔴 **Ice ký 2026-08-19: ĐÚNG MỘT chỗ, chính chỗ này — KHÔNG rải sang lượt đổi Chương.**
    // Lý do là một sự khác hạng, không một sở thích: `segment.id` không tái dùng trong **cùng
    // một** kho, nên hộp thoại còn mở qua một lượt đổi Chương ghi vào một câu **không nhìn
    // thấy**, chứ không ghi vào **câu khác**. Cùng khuôn lý lẽ đã cho `resetLookupPanel()` ở
    // lại ngoài lượt đổi Chương *(`editorPanelState.ts:1530-1532`)*.
    resetSegmentHistory()

    // 🔴 **VỨT state cũ là CHƯA ĐỦ — phải NẠP LẠI ngay tại đây.**
    //
    // `resetSourcePanel()` gỡ cờ `chapterRequested`, nhưng không ai gọi lại hàm nạp. Chỗ
    // DUY NHẤT gọi `ensureChapterLoaded()` là `GridPanel.vue::onMounted`, mà ba chế độ
    // sống trong `<KeepAlive>` (`App.vue`, §Quyết định thiết kế #6) — `src/modes/README.md`
    // viết thẳng: *"lần hiện thứ hai trở đi không có `mounted`"*. Nên với thứ tự thao tác
    // THƯỜNG GẶP NHẤT:
    //
    //   mở app → xem Workspace trước (nạp hụt: không Tác phẩm nào đang mở)
    //          → sang Library tạo Tác phẩm  (`replace_open_work` phía Rust ĐÃ trỏ đúng)
    //          → quay lại Workspace          (KeepAlive ⇒ không `mounted` ⇒ không nạp lại)
    //
    // …Panel Source ở lại *"Chưa có Chương nào được mở"* **vĩnh viễn** dù Tác phẩm đã mở
    // thật — người dùng phải khởi động lại app. Bắt bằng test tay 2026-08-07.
    //
    // ⚠️ Vá ở ĐÂY chứ không đổi `onMounted` → `onActivated` ở `GridPanel.vue`: panel sống
    // trong cây của dockview, nên nó có nhận được `activated` hay không là một câu hỏi về nội
    // tình thư viện thứ ba — còn `finishSubmit` là điểm nghẽn mà **cả hai** nhánh nhập đã
    // đi qua, và *"Tác phẩm đang mở vừa đổi"* là một dữ kiện mà chính chỗ này biết CHẮC.
    // Cùng lý do reset đứng ở đây thay vì rải ra từng panel.
    void ensureChapterLoaded()
    // Cùng nguyên văn lập luận trên, áp cho cột bản dịch (Story 2.2): `GridPanel.vue`
    // cũng chỉ gọi `ensureSegmentsLoaded()` ở `onMounted`, và `<KeepAlive>` làm lượt hiện
    // thứ hai trở đi không có `mounted`.
    //
    // Vue has not patched the grid DOM when the load resolves; an earlier `enterFocus` is a no-op.
    void ensureSegmentsLoaded().then(async () => {
      await nextTick()
      enterFocus('panel.grid')
    })

    // 🔵 THÊM Story 6.3 — xoá ĐÚNG ô đã nộp thành công (nhánh dán hoặc nhánh tệp, theo
    // `importPreviewLastSubmittedFrom` — sống ở `importPreviewState.ts`, đọc CHỈ Ở ĐÂY,
    // không đặt lại: xem doc-comment của nó cho lý do KHÔNG xoá tại chỗ này. Trước story
    // này mỗi nhánh tự xoá ô của chính nó ngay tại `submitPastedText`/`submitFilePath`; giờ
    // việc TẠO Tác phẩm dời sang `confirmImportPreview()` (async, tách rời), nên chỗ DUY
    // NHẤT biết "đã tạo xong" là ở đây.
    if (importPreviewLastSubmittedFrom.value === 'text') {
      pastedText.value = ''
    } else if (importPreviewLastSubmittedFrom.value === 'file') {
      resetImportFiles()
    } else if (importPreviewLastSubmittedFrom.value === 'urls') {
      pastedUrls.value = ''
    }

    // 🔵 THÊM (vòng rà, mục B5) — [`destinationWorks`] chỉ tải MỘT LẦN ở `onMounted`
    // (`loadDestinationWorks`), nên một Tác phẩm vừa tạo TRONG CHÍNH phiên này (kể cả một lượt
    // append lên chính nó) không bao giờ hiện ra làm đích cho lượt nộp KẾ TIẾP mà không cần
    // remount toàn Chế độ. Tải lại NGAY sau một lượt tạo/append THÀNH CÔNG — best-effort, cùng
    // triết lý mọi lượt đọc phụ khác trong module này, không chặn form chính nếu trượt.
    void loadDestinationWorks()
  }
}

/** **THÊM (Story 6.7b)** — radio "Tác phẩm đã có" chọn nhưng picker CHƯA chọn Tác phẩm nào
 * (HOẶC HỢP LỆ): ba hàm nộp dưới đây từ chối đi tiếp thay vì gửi một đích không hợp lệ âm
 * thầm đọc thành "Tác phẩm mới" (đúng radio đang hiện chọn ngược lại). `LibraryMode.vue` đọc
 * CÙNG ô này cho `:disabled` của ba nút nộp đơn ngữ — một nguồn sự thật, không hai phép kiểm
 * lệch nhau giữa nút bị khoá thị giác và hàm thật sự chặn.
 *
 * 🔵 SỬA (vòng rà, mục E4/E5) — trước bản sửa này chỉ kiểm `destinationWorkId.value === null`,
 * nên MỘT id còn sống trong ref nhưng KHÔNG CÒN trong [`destinationWorks`] (Tác phẩm vừa bị
 * xoá/đổi tên khỏi Library giữa lúc chọn và lúc nộp) lẫn chuỗi `''` mà mục giữ chỗ bị vô hiệu
 * hoá của `LibraryMode.vue` phát ra đều KHÁC `null` — cả hai lọt qua guard và gửi đi như một
 * đích thật. [`pickedDestinationWork`] đã tra đúng cả ba trường hợp CHƯA CHỌN/ID LẠ/CHUỖI RỖNG
 * về `null`; kiểm nó thay vì `destinationWorkId` trực tiếp đóng cả hai lỗ cùng lúc. */
export const destinationPending = computed<boolean>(
  () => destinationMode.value === 'existing' && pickedDestinationWork.value === null,
)

/** Nhánh dán văn bản — mở màn xem trước bảng mã cho `pastedText` hiện tại (Story 6.3). */
export async function submitPastedText(): Promise<void> {
  // 🔴 SỬA (vòng rà đối kháng 2, mục 9) — `busy` KHÔNG còn phủ cửa sổ ghi thật. `busy` hạ
  // NGAY sau khi màn xem trước MỞ (`submitPastedText`/`submitFilePath` tự hạ nó vài dòng
  // dưới), rồi lượt GHI thật (`create_work` phía Rust) chỉ chạy SAU đó, ở
  // `confirmImportPreview()` — một cửa sổ mà nút "Nhập" đã hết `:disabled` từ lâu, chỉ còn
  // bị chặn THỊ GIÁC bởi lớp phủ scrim của `ImportPreviewOverlay.vue` (`z-index: 11,
  // position: fixed, inset: 0`). Chuột không xuyên qua được lớp phủ đó, nhưng
  // `dispatch('library.import.submit')` là một command ĐÃ ĐĂNG KÝ (`commands/index.ts`) — một
  // lối vào không đi qua DOM (bàn phím toàn cục, palette lệnh nếu có) vẫn gọi thẳng được
  // hàm này trong khi lớp phủ đang mở, mở một lượt xem trước THỨ HAI đè lên nguồn đang chờ
  // mà lượt xác nhận ĐẦU đang dùng — cùng lớp rủi ro "state bị ghi đè giữa chừng" mà mục 4/14
  // đã đóng ở hai góc khác. Chặn Ở ĐÂY (không chỉ `busy`) đóng góc còn lại: `importPreviewIsOpen`
  // đúng suốt từ lúc mở tới lúc huỷ/xác nhận xong — cửa sổ nó phủ RỘNG HƠN `busy`.
  if (busy.value || importPreviewIsOpen.value) return
  // ⚠️ Chốt ở ĐÂY, không chỉ ở `:disabled` của nút: `dispatch('library.import.submit')`
  // là một command đã đăng ký, và một lối vào tương lai (palette, phím tắt) sẽ đi thẳng
  // qua đây mà không đi qua thuộc tính DOM nào.
  if (pastedText.value.trim() === '') return
  // 🔴 THÊM (Story 6.7b) — xem doc-comment [`destinationPending`].
  if (destinationPending.value) return
  // ⚠️ `beginSubmit` trả `false` khi bản dịch cũ chưa chạm WAL — đi tiếp là mất nó im lặng.
  if (!(await beginSubmit())) return
  // 🔵 SỬA Story 6.3 — KHÔNG còn `createWorkFromText` ở đây: nộp form mở màn xem trước bảng
  // mã (`ImportPreviewOverlay.vue`); tạo Tác phẩm thật chỉ xảy ra sau khi người dùng bấm
  // "Xác nhận" trong đó — [`finishImportSubmission`] đóng vòng lúc đó, không ở đây.
  // `openImportPreviewFromText` tự chốt nhánh `'text'` (`importPreviewState.ts`).
  // 🔴 THÊM (Story 6.7b) — `effectiveName`/`effectiveSourceLang`/`effectiveGenre` thay
  // `name`/`sourceLang`/`genre` trần: khi đích là một Tác phẩm sẵn có, ba ô ĐÃ KẾ THỪA giá
  // trị của đích (Decision 2, §Never) — xem doc-comment ba hàm đó.
  await openImportPreviewFromText(
    effectiveName.value,
    effectiveSourceLang.value,
    effectiveGenre.value,
    pastedText.value,
    destinationWorkId.value,
  )
  surfaceUndetectableLang()
  busy.value = false
}

/** Tab Tệp — Rust's verdict picks the path: `text` opens the encoding preview of every row,
 * `bilingual` opens the bilingual preview of its single row; any other verdict does nothing. */
export async function submitFilePath(): Promise<void> {
  if (busy.value || importPreviewIsOpen.value || bilingualImportPreviewIsOpen.value) return
  const verdict = importFilesVerdict.value
  if (verdict !== 'text' && verdict !== 'bilingual') return
  if (destinationPending.value) return
  const paths = importFiles.value.map((f) => f.path)
  if (!(await beginSubmit())) return
  if (verdict === 'bilingual') {
    await openBilingualFromFiles(paths[0])
    busy.value = false
    return
  }
  // `openImportPreviewFromFile` tự chốt nhánh `'file'` (`importPreviewState.ts`).
  await openImportPreviewFromFile(
    effectiveName.value,
    effectiveSourceLang.value,
    effectiveGenre.value,
    paths,
    destinationWorkId.value,
  )
  surfaceUndetectableLang()
  busy.value = false
}

async function openBilingualFromFiles(path: string): Promise<void> {
  if (effectiveSourceLang.value === 'auto') {
    lastError.value = BILINGUAL_NEEDS_SOURCE_LANG_ERROR
    return
  }
  await openBilingualImportPreview(
    effectiveName.value,
    effectiveSourceLang.value,
    effectiveGenre.value,
    path,
    destinationMode.value,
  )
}

/** The primary button of the overlay: submits only the selected tab. */
export async function submitActiveImportTab(): Promise<void> {
  switch (importActiveTab.value) {
    case 'paste':
      await submitPastedText()
      return
    case 'files':
      await submitFilePath()
      return
    case 'urls':
      await submitPastedUrls()
      return
  }
}

/** Whether the primary button of the selected tab may be pressed. */
export const importSubmitReady = computed<boolean>(() => {
  if (busy.value || destinationPending.value) return false
  switch (importActiveTab.value) {
    case 'paste':
      return pastedText.value.trim() !== ''
    case 'files':
      return (importFilesVerdict.value === 'text' || importFilesVerdict.value === 'bilingual')
    case 'urls':
      return pastedUrlCount.value > 0
  }
})

/** Leaves the bilingual preview for the overlay with the destination set to a new Work. */
export function switchBilingualImportToNewWork(): void {
  setDestinationMode('new')
  cancelBilingualImportPreview()
}

/** Nhánh danh sách URL (Story 6.7, FR122) — tải TUẦN TỰ đúng thứ tự đã dán rồi mở màn xem
 * trước. Cùng khuôn chốt của `submitPastedText`/`submitFilePath`: `busy || importPreviewIsOpen`
 * kiểm TRONG HÀM (không chỉ `:disabled` trên nút), `beginSubmit()` chạy TRƯỚC. */
export async function submitPastedUrls(): Promise<void> {
  if (busy.value || importPreviewIsOpen.value) return
  const urls = pastedUrlLines.value
  if (urls.length === 0) return
  // 🔴 THÊM (Story 6.7b) — xem doc-comment [`destinationPending`]; ĐẶC BIỆT quan trọng ở
  // nhánh này — không lệnh Rust nào cho phép đặt lại đích của một phiên URL sau lượt gọi
  // NGAY DƯỚI ĐÂY (xem doc-comment `startUrlImport` ở `config/project.ts`).
  if (destinationPending.value) return
  if (!(await beginSubmit())) return
  // `openImportPreviewFromUrls` tự chốt nhánh `'urls'` (`importPreviewState.ts`).
  await openImportPreviewFromUrls(
    effectiveName.value,
    effectiveSourceLang.value,
    effectiveGenre.value,
    urls,
    destinationWorkId.value,
  )
  surfaceUndetectableLang()
  busy.value = false
}

/**
 * Gắn ba bộ nghe kéo-thả — gọi từ `LibraryMode.vue::onMounted`, gỡ ở `onBeforeUnmount`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 THẢ MỘT TỆP ĐỔ VÀO Ô ĐƯỜNG DẪN — NÓ KHÔNG GHI XUỐNG ĐĨA
 * ─────────────────────────────────────────────────────────────────────────────
 * AC1 nguyên văn: *"**không có gì được ghi xuống đĩa trước khi người dùng xác nhận**"*
 * (bất biến của mọi đường nhập — `EXPERIENCE.md#KF-1`). Bản trước gọi thẳng lệnh tạo ngay
 * trong bộ nghe, nên một cú thả **bất kỳ đâu trong cửa sổ** — kể cả khi đang ở Workspace
 * và màn Library chưa từng hiện — ghi ngay một `.atproj` xuống đĩa với `name`/`genre`
 * còn sót lại trong ref. Đó là AC1 bị phá ở đúng bất biến nó tồn tại để giữ.
 * Code review 2026-08-06.
 *
 * ⇒ Thả **điền đường dẫn vào ô**, rồi người dùng bấm nút. Cùng một nút, cùng một hàm, cùng
 * một bước xác nhận với đường bàn phím — và không đường nào giữ bản sao pipeline
 * (AD-39:498).
 *
 * ⚠️ Rust chỉ chuyển tiếp ĐƯỜNG DẪN (AD-1/AD-16) — nội dung tệp chưa từng chạm webview.
 *
 * Không ném nếu không có cầu IPC (`npm run dev` ngoài Tauri) — cùng nhánh "không có cầu
 * IPC" của `loadBootstrapConfig`.
 */
export function wireDragDropOnce(): void {
  if (dragDropWired) return
  dragDropWired = true

  const wire = <T,>(name: string, handler: (payload: T) => void): void => {
    void listen<T>(name, (event) => handler(event.payload))
      .then((unlisten) => {
        unlisteners.push(unlisten)
      })
      .catch((err: unknown) => {
        console.info(`[library] không gắn được bộ nghe \`${name}\` — chạy ngoài Tauri? ${String(err)}`)
      })
  }

  wire<unknown>(DRAG_ENTER_EVENT, (count) => {
    dragFileCount.value = typeof count === 'number' && count > 0 ? count : 0
    isDragOver.value = true
  })
  wire<unknown>(DRAG_LEAVE_EVENT, () => {
    isDragOver.value = false
    dragFileCount.value = 0
  })
  wire<string[]>(DRAG_DROP_EVENT, (paths) => {
    isDragOver.value = false
    dragFileCount.value = 0

    const valid = Array.isArray(paths)
      ? paths.filter((p): p is string => typeof p === 'string' && p.length > 0)
      : []
    if (valid.length === 0) return
    selectImportTab('files')
    noticeKey.value = null
    lastError.value = null
    const known = importFiles.value.map((f) => f.path)
    void applyImportFilePaths([...known, ...valid.filter((path) => !known.includes(path))])
  })
}

/**
 * Gỡ ba bộ nghe — gọi từ `LibraryMode.vue::onBeforeUnmount`.
 *
 * ⚠️ Có nó vì bộ nghe là **tầng cửa sổ**, không phải của phần tử `.dropzone`: để nó
 * sống sau khi chế độ bị tháo nghĩa là một cú thả ở Workspace vẫn điền vào một form không
 * không còn trên màn hình.
 */
export function unwireDragDrop(): void {
  for (const unlisten of unlisteners) unlisten()
  unlisteners = []
  dragDropWired = false
  isDragOver.value = false
  dragFileCount.value = 0
}
