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
import { computed, readonly, ref, watch } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import {
  cleanupAddRule,
  cleanupDeleteRule,
  cleanupEditRule,
  cleanupSetEnabled,
  confirmImportWithEncoding,
  previewChapterDetail,
  previewImportEncodingFromFile,
  previewImportEncodingFromText,
  reloadUrlImportItem,
  removeUrlImportItem,
  startUrlImport,
  tier2BlockConfirmRange,
  tier2BlockSetKept,
} from './config/project'
import type {
  ChapterPatternInput,
  ChapterPatternKindWire,
  ChapterSplitPreviewWire,
  CleanupPreviewWire,
  CleanupRuleKindWire,
  CleanupRuleTierWire,
  CreatedWork,
  EncodingCandidateWire,
  ImportEncodingPreview,
  ImportEncodingPreviewResult,
  NormalizedPreviewWire,
  ChapterBlocksPreviewWire,
  UrlImportItemWire,
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
// 🔵 SỬA 2026-09-06 (Story 6.7) — biến thể thứ BA, `'urls'`. Danh sách URL là một nhánh "đã
// mở lượt xem trước" mới, khác `'text'`/`'file'` ở chỗ nguồn KHÔNG sống trong
// `pendingText`/`pendingPath` — nó sống ở `urlImportItems` (mỗi mục giữ trạng thái RIÊNG,
// không phải MỘT chuỗi/đường dẫn duy nhất).
const lastSubmittedFrom = ref<'text' | 'file' | 'urls' | null>(null)

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

/**
 * **THÊM (Story 6.7)** — ô THỨ BA, song song với `pendingText`/`pendingPath` — danh sách
 * mục-theo-link của lượt nhập URL đang mở. Mỗi mục mang vị trí (INDEX trong mảng), URL, và
 * kết quả (`ok`/`error`) — khác `pendingText`/`pendingPath` (một GIÁ TRỊ trần), đây là N giá
 * trị SONG SONG với hình dạng `PipelineShape::Chapters` phía Rust.
 *
 * 🔴 **Đây là dữ liệu ĐÃ TẢI, không phải `pastedUrls`** (ô đó sống ở `libraryImport.ts`, chỉ
 * là NỘI DUNG Ô DÁN — hai con số *"N link · sẽ tạo N Chương"* tính từ nó, 0 IPC). `urlImportItems`
 * chỉ có giá trị SAU khi người dùng đã bấm nút tải (`openImportPreviewFromUrls` đã chạy).
 */
const urlImportItems = ref<UrlImportItemWire[]>([])
/** Cờ "đang gửi" của tải-lại/bỏ MỘT mục — TÁCH khỏi `opening` (mở CẢ lượt) và bốn cờ CRUD
 * luật làm sạch (ngữ nghĩa khác hẳn: đây là thao tác trên MỘT MỤC trong danh sách URL). */
const urlImportBusy = ref(false)
/** Lỗi hạ tầng của lượt tải-lại/bỏ một mục (ví dụ cầu IPC vắng giữa chừng) — KHÁC lỗi CỦA
 * TỪNG MỤC (đã nằm trong `item.error`, hiển thị inline trên hàng của chính nó). */
const urlImportError = ref<IpcError | null>(null)
/** 🔵 **THÊM Story 6.8 (NFR19)** — số domain PHÂN BIỆT trong nhật ký của CẢ PHIÊN CHẠY, đọc
 * thẳng từ `UrlImportBatchWire.domain_log_domain_count` của lượt tải/tải-lại/bỏ GẦN NHẤT
 * (xem doc-comment [`applyUrlImportBatch`]). Chân màn hiện dòng *"Đã gọi N domain · xem"*
 * khi và CHỈ KHI số này lớn hơn 0 — 0 nghĩa là chưa gọi mạng lần nào (I/O Matrix spec 6.8:
 * "Dán N link, chưa bấm ⇒ 0 lời gọi mạng ⇒ chân màn không có dòng domain nào"). */
const domainLogDomainCount = ref(0)

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

/**
 * **THÊM (Story 6.9)** — trạng thái điều hướng bàn phím tầng 2 (ranh giới bóc), sống Ở ĐÂY
 * (không trong `.vue`, cùng lý do mọi state khác của lớp phủ này) vì nó phải sống sót qua
 * lượt xem trước dựng lại (`Space`/`[`/`]` gọi IPC rồi Rust trả về một `preview` MỚI, index
 * đang chọn không được nhảy về 0 chỉ vì mảng khối vừa được thay bằng một mảng CÙNG NỘI DUNG).
 *
 * `blockFocusedIndex` — chỉ số khối ĐANG CHỌN trong `importPreviewSelectedBlocks.value.blocks`
 * (J/K di chuyển). `blockRangeStart` — mốc `[` (chỉ số khối tại thời điểm bấm), `null` khi
 * chưa đặt. `blockRangeMissingStartNotice` — I/O Matrix spec 6.9 "`]` trước `[` ⇒ kêu, không
 * ném": `confirmImportPreviewBlockRange` bật cờ này thay vì gọi Rust khi chưa có mốc đầu.
 */
const blockFocusedIndex = ref(0)
const blockRangeStart = ref<number | null>(null)
const blockRangeMissingStartNotice = ref(false)
/** Cờ "đang gửi" của `Space` — TÁCH khỏi `blockRangeConfirming` (thao tác khác hẳn, cùng lý
 * do bốn cờ CRUD luật làm sạch tách nhau). */
const blockToggling = ref(false)
/** Cờ "đang gửi" của `]`. */
const blockRangeConfirming = ref(false)
/** Lỗi hạ tầng của lượt `Space`/`]` gần nhất (state Tauri chưa quản lý, …) — KHÁC lỗi CỦA
 * TỪNG MỤC URL (`urlImportError`) và lỗi xác nhận toàn bộ Tác phẩm (`confirmError`). */
const blockActionError = ref<IpcError | null>(null)
/** Đếm lượt bấm `R` — `ImportPreviewOverlay.vue` watch số này để cuộn/đặt tiêu điểm sang
 * tầng 3 (§Spec Change Log spec 6.9: "R nhảy sang tầng 3", không khớp luật theo khối). Một số
 * tăng dần (không phải boolean) để hai lượt bấm `R` LIÊN TIẾP (tầng 3 đã có tiêu điểm từ lượt
 * trước) vẫn kích hoạt lại `watch` — Vue không bắn `watch` khi giá trị mới trùng giá trị cũ. */
const jumpToCleanupRulesSignal = ref(0)

/**
 * **THÊM (Story 6.10a)** — con trỏ *Chương đang chọn*, 0-based, index vào
 * `importPreviewSelectedChapters.value.chapters`. `⌥←`/`⌥→` dời con trỏ; Chương 0 đọc THẲNG
 * từ `candidate.cleanup`/`.blocks` (đã có sẵn EAGER, 0 lời gọi IPC) — bốn ô dưới đây chỉ có
 * ý nghĩa khi con trỏ KHÁC 0 (chi tiết LAZY, xem [`loadImportPreviewChapterDetail`]).
 */
const chapterCursor = ref(0)
/** Chi tiết tầng 3 (làm sạch) của Chương con trỏ đang trỏ tới — `null` khi con trỏ ở Chương 0
 * hoặc lượt dựng lazy đang bay/vừa trượt. */
const chapterDetailCleanup = ref<CleanupPreviewWire | null>(null)
/** Chi tiết tầng 2 (khối) của Chương con trỏ đang trỏ tới — cùng điều kiện `chapterDetailCleanup`. */
const chapterDetailBlocks = ref<ChapterBlocksPreviewWire | null>(null)
/** Cờ "đang gửi" của lệnh IPC lazy `preview_chapter_detail` — chặn hai lượt dời con trỏ chồng
 * lệnh (khuôn `event.repeat` guard ở tầng `.vue` cộng lớp phòng thủ THỨ HAI ở đây). */
const chapterDetailLoading = ref(false)
/** Lỗi RIÊNG của lượt dựng chi tiết Chương gần nhất — TÁCH khỏi mọi lỗi khác, cùng lý do
 * `blockActionError` tách khỏi `urlImportError`/`confirmError`. */
const chapterDetailError = ref<IpcError | null>(null)

/**
 * **THÊM (Story 6.6)** — mẫu phân tách Chương ĐANG GÕ, tham số MỖI LƯỢT NHẬP (§Always spec
 * 6.6: KHÔNG một cơ chế "nhớ mẫu" nào — Ice chốt 2026-09-05 mặc định KHÔNG nhớ giữa hai lượt
 * nhập, xem §Ask First của spec). Chuỗi rỗng ⇒ không mẫu (no-op, N = 1) — xem
 * [`chapterPatternWire`].
 */
const chapterPatternText = ref('')
const chapterPatternKind = ref<ChapterPatternKindWire>('literal')

/** Cờ "đang gửi" của lượt tải lại xem trước SAU MỘT LẦN SỬA MẪU — TÁCH khỏi
 * `cleanupAdding`/`cleanupSavingEdit`/… (bốn cờ của bốn hành động luật làm sạch, ngữ nghĩa
 * khác hẳn: sửa MẪU không phải một lượt GHI luật). */
const chapterPatternSending = ref(false)

/**
 * Lỗi RIÊNG của lượt sửa mẫu gần nhất — TÁCH khỏi `loadError` (lỗi của lượt MỞ màn xem
 * trước). §I/O Matrix spec 6.6, hàng "Regex không biên dịch được": *"xem trước GIỮ kết quả
 * CŨ, hiện thông báo"* — nếu dùng chung `loadError`/`status`, một mẫu hỏng sẽ lật `status`
 * sang `'error'` và `.vue` đổi hẳn nhánh render, NUỐT MẤT dải/khối vừa hiện thay vì giữ
 * nguyên nó. Xem [`reloadImportPreviewAfterChapterPatternChange`].
 */
const chapterPatternError = ref<IpcError | null>(null)

/**
 * Mã lỗi IPC của MỘT mẫu phân tách Chương không biên dịch được
 * (`core::segment::import::ImportError::InvalidChapterPattern`, khoá hiển thị
 * `err.import.invalid_chapter_pattern`) — dùng để RẼ NHÁNH lỗi này ra khỏi mọi lỗi tải lại
 * KHÁC. **SỬA (vòng rà đối kháng 3, mục 2)**: [`runImportPreviewReload`] gửi lại
 * [`chapterPatternWire`] HIỆN HÀNH ở MỌI lượt gọi — kể cả một lượt CRUD luật làm sạch
 * ([`reloadImportPreviewAfterRuleChange`]) hoàn toàn không đụng tới ô mẫu. Một mẫu hỏng còn
 * đứng nguyên trong ô rồi một hành động KHÁC (bật/tắt một luật) kích hoạt tải lại sẽ nhận lại
 * ĐÚNG lỗi này lần nữa — nếu nhánh xử lý lỗi ở đó không tách riêng, nó lật `status` sang
 * `'error'` vô điều kiện và xoá sạch dải bảng mã/danh sách luật/danh sách Chương đang hiện,
 * dù nguyên nhân chỉ là ô mẫu, không phải luật vừa đổi. Xử lý PHẢI giống hệt
 * [`reloadImportPreviewAfterChapterPatternChange`]: đi vào [`chapterPatternError`], GIỮ
 * NGUYÊN `status`/`preview`.
 */
const CHAPTER_PATTERN_INVALID_CODE = 'import.invalid_chapter_pattern'

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
export const importPreviewLastSubmittedFrom: DeepReadonly<Ref<'text' | 'file' | 'urls' | null>> =
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
export const importPreviewBlockFocusedIndex: DeepReadonly<Ref<number>> = readonly(blockFocusedIndex)
export const importPreviewBlockRangeStart: DeepReadonly<Ref<number | null>> = readonly(blockRangeStart)
export const importPreviewBlockRangeMissingStartNotice: DeepReadonly<Ref<boolean>> =
  readonly(blockRangeMissingStartNotice)
export const importPreviewBlockToggling: DeepReadonly<Ref<boolean>> = readonly(blockToggling)
export const importPreviewBlockRangeConfirming: DeepReadonly<Ref<boolean>> = readonly(blockRangeConfirming)
export const importPreviewBlockActionError: DeepReadonly<Ref<IpcError | null>> = readonly(blockActionError)
export const importPreviewJumpToCleanupRulesSignal: DeepReadonly<Ref<number>> =
  readonly(jumpToCleanupRulesSignal)
/** Con trỏ *Chương đang chọn* — Story 6.10a. 0-based. */
export const importPreviewChapterCursor: DeepReadonly<Ref<number>> = readonly(chapterCursor)
/** `true` ⇔ đang bay một lượt dựng lại chi tiết Chương (`⌥←`/`⌥→` vừa bấm). */
export const importPreviewChapterDetailLoading: DeepReadonly<Ref<boolean>> =
  readonly(chapterDetailLoading)
/** Lỗi RIÊNG của lượt dựng chi tiết Chương gần nhất. */
export const importPreviewChapterDetailError: DeepReadonly<Ref<IpcError | null>> =
  readonly(chapterDetailError)
export const importPreviewChapterPatternText: DeepReadonly<Ref<string>> = readonly(chapterPatternText)
export const importPreviewChapterPatternKind: DeepReadonly<Ref<ChapterPatternKindWire>> =
  readonly(chapterPatternKind)
export const importPreviewChapterPatternSending: DeepReadonly<Ref<boolean>> =
  readonly(chapterPatternSending)
export const importPreviewChapterPatternError: DeepReadonly<Ref<IpcError | null>> =
  readonly(chapterPatternError)
/** Danh sách mục-theo-link của lượt nhập URL đang mở — rỗng khi `lastSubmittedFrom !==
 * 'urls'`. Xem doc-comment [`urlImportItems`]. */
export const importPreviewUrlItems: DeepReadonly<Ref<UrlImportItemWire[]>> = readonly(urlImportItems)
/** 🔵 **THÊM Story 6.8 (NFR19)** — số domain PHÂN BIỆT trong nhật ký của CẢ PHIÊN CHẠY, tại
 * thời điểm lượt tải/tải-lại/bỏ GẦN NHẤT trả lời. `0` ⇔ chưa gọi mạng lần nào TRONG lượt
 * xem trước này (không nhất thiết `0` của cả phiên — một Tác phẩm trước đó có thể đã gọi
 * mạng, và số này chỉ được LÀM MỚI khi chính lớp phủ URL đang mở gọi một trong ba lệnh).
 * `ImportPreviewOverlay.vue` chỉ hiện dòng tóm tắt khi số này `> 0`. */
export const importPreviewDomainLogDomainCount: DeepReadonly<Ref<number>> = readonly(domainLogDomainCount)
export const importPreviewUrlImportBusy: DeepReadonly<Ref<boolean>> = readonly(urlImportBusy)
export const importPreviewUrlImportError: DeepReadonly<Ref<IpcError | null>> = readonly(urlImportError)
/** `true` ⇔ còn ít nhất một mục hỏng trong danh sách URL — điều kiện KHOÁ nút xác nhận
 * (§Always spec 6.7). Tính CỤC BỘ từ `urlImportItems` — Rust đã tự khoá THẬT qua
 * `PendingImportSourceState` (một lượt `confirmImportWithEncoding` khi còn mục hỏng luôn trả
 * `import.no_pending_source`); computed này CHỈ để tầng hiển thị disable nút SỚM, không phải
 * nguồn sự thật duy nhất. */
export const importPreviewUrlListHasBrokenItem = computed<boolean>(() => {
  if (lastSubmittedFrom.value !== 'urls') return false
  return urlImportItems.value.length === 0 || urlImportItems.value.some((it) => !it.ok)
})

/**
 * **THÊM (Story 6.10a)** — vị từ GHI cho nút xác nhận, TÁCH khỏi `importPreview !== null`
 * (vị từ XEM). Trước story này hai vị từ trùng nhau TRÊN ĐƯỜNG URL (`encoding_preview` phía
 * Rust là `null` chính xác khi còn mục hỏng), nên `importPreview === null` từng là một cách
 * ĐỌC ĐÚNG (dù gián tiếp) của "còn mục hỏng". Story 6.10a đổi vị từ XEM (`chapters_shape_for_view`,
 * bỏ qua mục hỏng để vẫn dựng được xem trước cho các mục OK) — `importPreview` nay khác `null`
 * NGAY CẢ KHI còn mục hỏng, nên đọc nó để khoá nút là ĐÚNG lỗi mà §Always story 6.10a cấm
 * ("trộn vị từ XEM với vị từ GHI"). Đường URL đọc thẳng [`importPreviewUrlListHasBrokenItem`]
 * (đã có sẵn từ Story 6.7, tính CỤC BỘ trên `urlImportItems[].ok` — không đọc `preview`);
 * đường tệp/dán tay không có khái niệm "mục hỏng", giữ nguyên `preview !== null`.
 */
export const importPreviewCanConfirm = computed<boolean>(() => {
  if (lastSubmittedFrom.value === 'urls') return !importPreviewUrlListHasBrokenItem.value
  return preview.value !== null
})

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
 *
 * 🔵 **SỬA (Story 6.10a) — con trỏ Chương KHÁC 0 đọc từ chi tiết LAZY, không còn LUÔN Chương
 * 0.** `candidate.cleanup`/`self_declared_cleanup` (Rust dựng EAGER) là chi tiết của ĐÚNG
 * Chương 0 (§Design Notes: "tóm tắt eager, chi tiết lazy") — con trỏ dời sang Chương k > 0
 * đọc [`chapterDetailCleanup`] (dựng qua lệnh IPC lazy `preview_chapter_detail` khi con trỏ
 * dời, xem [`loadImportPreviewChapterDetail`]), `null` trong lúc đang bay/vừa trượt.
 */
export const importPreviewSelectedCleanup = computed<CleanupPreviewWire | null>(() => {
  if (chapterCursor.value !== 0) return chapterDetailCleanup.value
  const p = preview.value
  if (p === null) return null
  const candidate = importPreviewSelectedCandidate.value
  if (candidate !== null) return candidate.cleanup
  return p.self_declared_cleanup
})

/**
 * Khối tách Chương (tầng 4) hiện hành — Story 6.6. CÙNG khuôn
 * [`importPreviewSelectedCleanup`]: đọc `candidate.chapters` khi có ứng viên đang chọn, rơi
 * về `preview.self_declared_chapters` khi không.
 *
 * 🔴 **Đổi ứng viên đổi computed này NGAY, 0 lời gọi IPC** — Rust đã dựng sẵn khối tách
 * Chương của CẢ NĂM ứng viên VÀ của nhánh tự khai trên dây, computed này chỉ ĐỌC lại.
 */
export const importPreviewSelectedChapters = computed<ChapterSplitPreviewWire | null>(() => {
  const p = preview.value
  if (p === null) return null
  const candidate = importPreviewSelectedCandidate.value
  if (candidate !== null) return candidate.chapters
  return p.self_declared_chapters
})

/**
 * Khối tầng 2 (ranh giới bóc) hiện hành — Story 6.9. KHÁC ba computed theo-ứng-viên ở trên:
 * KHÔNG rơi về một trường `self_declared_*` — tầng 2 chỉ có nghĩa trên đường URL
 * (`extract_main_content` chỉ `true` ở đó, §Always spec 6.7/6.9), và đường đó LUÔN có ứng
 * viên (byte HTML thật luôn đi qua dò bảng mã) — nhánh tự khai (`candidate === null`, dán văn
 * bản tay) không có khái niệm "khối" để mà rơi về.
 *
 * 🔵 **SỬA (Story 6.10a)** — cùng lý do [`importPreviewSelectedCleanup`]: con trỏ Chương khác
 * 0 đọc từ [`chapterDetailBlocks`] (chi tiết LAZY), không còn LUÔN của Chương 0.
 */
export const importPreviewSelectedBlocks = computed<ChapterBlocksPreviewWire | null>(() => {
  if (chapterCursor.value !== 0) return chapterDetailBlocks.value
  const candidate = importPreviewSelectedCandidate.value
  return candidate !== null ? candidate.blocks : null
})

// 🔴 Giữ `blockFocusedIndex` LUÔN trong phạm vi hợp lệ — dải khối có thể đổi ĐỘ DÀI dưới
// chân nó (đổi ứng viên bảng mã, hoặc một lượt `Space`/`]` dựng lại xem trước với cấu trúc
// khối có thể khác — Quyết định #2 §Spec Change Log spec 6.9 chấp nhận rủi ro hẹp này). Một
// chỉ số vượt quá mảng mới sẽ làm `blocks[blockFocusedIndex.value]` đọc ra `undefined` và mọi
// phép so `kept`/`confirmed` đọc trên nó vỡ IM LẶNG — đúng lớp lỗi AGENTS.md gọi tên là trung
// tâm của dự án.
watch(importPreviewSelectedBlocks, (blocks) => {
  const length = blocks?.blocks.length ?? 0
  if (length === 0) {
    blockFocusedIndex.value = 0
    blockRangeStart.value = null
    return
  }
  if (blockFocusedIndex.value >= length) {
    blockFocusedIndex.value = length - 1
  }
  if (blockRangeStart.value !== null && blockRangeStart.value >= length) {
    blockRangeStart.value = null
  }
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
  // §Ask First spec 6.6: KHÔNG nhớ mẫu phân tách giữa hai lượt nhập — mỗi lượt MỞ mới bắt
  // đầu từ rỗng, kể cả khi lượt trước đó vừa dùng một mẫu.
  chapterPatternText.value = ''
  chapterPatternKind.value = 'literal'
  chapterPatternSending.value = false
  chapterPatternError.value = null
  pendingChapterPatternEdit = null
  blockFocusedIndex.value = 0
  blockRangeStart.value = null
  blockRangeMissingStartNotice.value = false
  blockToggling.value = false
  blockRangeConfirming.value = false
  blockActionError.value = null
  jumpToCleanupRulesSignal.value = 0
  chapterCursor.value = 0
  chapterDetailCleanup.value = null
  chapterDetailBlocks.value = null
  chapterDetailLoading.value = false
  chapterDetailError.value = null

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
  await openWith(
    () => previewImportEncodingFromText(text, sourceLang, null),
    'text',
    name,
    sourceLang,
    genre,
  )
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
  await openWith(
    () => previewImportEncodingFromFile(path, sourceLang, null),
    'file',
    name,
    sourceLang,
    genre,
  )
}

/**
 * Mở màn xem trước — nhánh DANH SÁCH URL (Story 6.7, FR122). Gọi từ handler tiêm của
 * `library.import_urls`, SAU khi `urls` đã được lọc dòng rỗng ở `libraryImport.ts` (hai con
 * số hiển thị TRƯỚC khi bấm nút tính từ CHÍNH phép lọc đó — tính lại ở đây không đổi kết
 * quả, chỉ là cùng một phép lọc chạy hai lần).
 *
 * ⚠️ **KHÔNG tái dùng `openWith`** — hàm đó giả định một `PreviewCall` trả về
 * `ImportEncodingPreviewResult` (một `preview` trần); URL trả về `UrlImportBatchResult`
 * (`items` CỘNG một `encoding_preview` CÓ THỂ `null` khi còn mục hỏng). Thân hàm dưới đây lặp
 * lại phần khởi tạo của `openWith` một cách có chủ ý — xem doc-comment `UrlImportBatchWire`
 * ở `config/project.ts` cho lý do hai hình dạng không gộp được vào MỘT hàm chung mà không
 * làm `openWith` mất khả năng đọc.
 */
export async function openImportPreviewFromUrls(
  name: string,
  sourceLang: string,
  genre: string,
  urls: string[],
): Promise<void> {
  if (opening.value) return

  opening.value = true
  sequence += 1
  const mySequence = sequence

  lastSubmittedFrom.value = 'urls'
  pendingName.value = name
  pendingSourceLang.value = sourceLang
  pendingGenre.value = genre
  pendingText.value = null
  pendingPath.value = null
  confirming.value = false
  confirmError.value = null
  cleanupActionError.value = null
  cleanupAdding.value = false
  cleanupSavingEdit.value = false
  cleanupDeleting.value = false
  cleanupToggling.value = false
  cleanupDeletePendingKey.value = null
  stripForcedOpen.value = false
  chapterPatternText.value = ''
  chapterPatternKind.value = 'literal'
  chapterPatternSending.value = false
  chapterPatternError.value = null
  pendingChapterPatternEdit = null
  urlImportItems.value = []
  domainLogDomainCount.value = 0
  urlImportBusy.value = false
  urlImportError.value = null
  blockFocusedIndex.value = 0
  blockRangeStart.value = null
  blockRangeMissingStartNotice.value = false
  blockToggling.value = false
  blockRangeConfirming.value = false
  blockActionError.value = null
  jumpToCleanupRulesSignal.value = 0
  chapterCursor.value = 0
  chapterDetailCleanup.value = null
  chapterDetailBlocks.value = null
  chapterDetailLoading.value = false
  chapterDetailError.value = null

  const result = await startUrlImport(urls, sourceLang)
  if (mySequence !== sequence) return // một lượt mở/huỷ MỚI đã vượt mặt lượt này

  opening.value = false
  overlayOpen.value = true

  if (result.error !== null) {
    status.value = 'error'
    loadError.value = result.error
    preview.value = null
    selectedEncoding.value = null
    return
  }
  if (result.batch === null) {
    status.value = 'ipc_unavailable'
    loadError.value = null
    preview.value = null
    selectedEncoding.value = null
    return
  }

  urlImportItems.value = result.batch.items
  domainLogDomainCount.value = result.batch.domain_log_domain_count
  // 🔵 SỬA 2026-09-08 (Story 6.10a) — `encoding_preview === null` KHÔNG còn ⇔ "còn mục hỏng".
  // Vị từ XEM phía Rust (`chapters_shape_for_view`) nay bỏ qua mục hỏng để vẫn dựng được xem
  // trước từ các mục OK còn lại — `null` chỉ còn đúng khi KHÔNG mục OK nào (danh sách rỗng,
  // hoặc MỌI mục đều hỏng). KHÔNG có gì để hiện ở tầng 1-4 trong ca đó, nhưng lớp phủ VẪN mở
  // để người dùng thấy danh sách mục và sửa (bỏ/tải lại). Nút xác nhận khoá hay không đọc
  // [`importPreviewCanConfirm`] RIÊNG (vị từ GHI), không đọc trường này.
  if (result.batch.encoding_preview === null) {
    preview.value = null
    selectedEncoding.value = null
  } else {
    preview.value = result.batch.encoding_preview
    selectedEncoding.value = result.batch.encoding_preview.selected_encoding
  }
  status.value = 'loaded'
  loadError.value = null
}

/** Cập nhật state SAU một lượt tải-lại/bỏ-một-mục — dùng chung bởi
 * [`reloadImportPreviewUrlItem`]/[`removeImportPreviewUrlItem`]/[`toggleImportPreviewBlockKept`]/
 * [`confirmImportPreviewBlockRange`]. Cố giữ nguyên ứng viên đang chọn nếu nó vẫn còn trong dải
 * mới, cùng khuôn [`reloadImportPreviewAfterRuleChange`].
 *
 * 🔴 **SỬA 2026-09-07 (vòng rà bước 4, mục 11) — dọn `blockRangeMissingStartNotice`.** MỌI
 * lượt tới đây dựng lại `UrlImportBatchWire` TƯƠI (tải lại/bỏ một mục, hoặc chính lượt
 * `Space`/`]` vừa thành công) — cấu trúc khối tầng 2 có thể đã đổi (mục 0 bị tải lại/bỏ),
 * làm một cảnh báo "`]` trước `[`" còn treo từ TRƯỚC lượt này hết còn đúng ngữ cảnh (mốc `[`
 * cũ, nếu còn, đã bị chính `watch(importPreviewSelectedBlocks, ...)` xét lại rồi). Không dọn
 * ở đây thì cảnh báo cũ có thể đứng treo VĨNH VIỄN qua một thao tác không liên quan gì tới
 * `]`. `blockActionError` KHÔNG dọn ở đây — hai chỗ gọi tầng 2 ([`toggleImportPreviewBlockKept`]/
 * [`confirmImportPreviewBlockRange`]) tự dọn nó NGAY TRƯỚC khi gọi hàm này (đường thành
 * công), giữ nguyên đường LỖI (hàm này không được gọi khi `result.error !== null`).
 */
function applyUrlImportBatch(batch: NonNullable<Awaited<ReturnType<typeof startUrlImport>>['batch']>): void {
  const keepEncoding = selectedEncoding.value
  urlImportItems.value = batch.items
  domainLogDomainCount.value = batch.domain_log_domain_count
  blockRangeMissingStartNotice.value = false
  if (batch.encoding_preview === null) {
    preview.value = null
    selectedEncoding.value = null
    // THÊM (Story 6.10a) — 0 Chương hợp lệ để mà xem, kẹp con trỏ về 0 + dọn chi tiết lazy.
    syncChapterCursorAfterUrlBatch()
    return
  }
  preview.value = batch.encoding_preview
  selectedEncoding.value = batch.encoding_preview.candidates.some((c) => c.encoding === keepEncoding)
    ? keepEncoding
    : batch.encoding_preview.selected_encoding
  // THÊM (Story 6.10a) — số Chương có thể đã đổi dưới chân con trỏ (tải lại/bỏ một mục URL).
  syncChapterCursorAfterUrlBatch()
}

/** Tải lại ĐÚNG MỘT mục hỏng ở vị trí `index` — I/O Matrix spec 6.7: "đúng 1 lời gọi mạng".
 * No-op khi chưa mở lượt URL nào, hoặc một lượt khác đang bay. */
export async function reloadImportPreviewUrlItem(index: number): Promise<void> {
  if (confirming.value || urlImportBusy.value || lastSubmittedFrom.value !== 'urls') return
  urlImportBusy.value = true
  try {
    const result = await reloadUrlImportItem(index, pendingSourceLang.value)
    if (result.error !== null) {
      urlImportError.value = result.error
      return
    }
    if (result.batch === null) return
    urlImportError.value = null
    applyUrlImportBatch(result.batch)
  } finally {
    urlImportBusy.value = false
  }
}

/** Bỏ một mục ở vị trí `index` — I/O Matrix spec 6.7: "N−1 link · N−1 Chương, hai số cùng
 * giảm". **0 lời gọi mạng.** */
export async function removeImportPreviewUrlItem(index: number): Promise<void> {
  if (confirming.value || urlImportBusy.value || lastSubmittedFrom.value !== 'urls') return
  urlImportBusy.value = true
  try {
    const result = await removeUrlImportItem(index, pendingSourceLang.value)
    if (result.error !== null) {
      urlImportError.value = result.error
      return
    }
    if (result.batch === null) return
    urlImportError.value = null
    applyUrlImportBatch(result.batch)
  } finally {
    urlImportBusy.value = false
  }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Story 6.9 — sửa ranh giới bóc bằng bàn phím (FR123). Sáu handler của sáu command
// `import.preview.block_*`/`import.preview.jump_to_cleanup_rules` (`src/commands/index.ts`),
// gọi từ handler DOM cục bộ trên scrim của `ImportPreviewOverlay.vue` — KHÔNG một hợp âm
// toàn cục (đo `keys.ts:510-513`, xem doc-comment `commands/index.ts::CommandDeps`).
// ═══════════════════════════════════════════════════════════════════════════════

/** `J`/xuống — không vòng lặp qua đầu (chạm cuối dải thì dừng, không "quay lại 0" gây mất
 * phương hướng). No-op khi tầng 2 rỗng.
 *
 * 🔴 **SỬA 2026-09-07 (vòng rà bước 4, mục 11) — dọn `blockActionError`.** Một lỗi ghi
 * (`Space`/`]` trượt) treo trên khối K đứng yên vô thời hạn nếu người dùng chỉ ĐIỀU HƯỚNG
 * sang khối khác mà không thử ghi lại — dọn ở đây để một lượt di chuyển đọc lại được coi là
 * "đã thấy lỗi, đang xem khối khác", cùng tinh thần dọn `blockActionError` trên MỌI đường
 * thành công khác trong tệp này. */
export function nextImportPreviewBlock(): void {
  const length = importPreviewSelectedBlocks.value?.blocks.length ?? 0
  if (length === 0) return
  blockFocusedIndex.value = Math.min(blockFocusedIndex.value + 1, length - 1)
  blockActionError.value = null
}

/** `K`/lên — cùng khuôn [`nextImportPreviewBlock`], dừng ở 0. */
export function prevImportPreviewBlock(): void {
  const length = importPreviewSelectedBlocks.value?.blocks.length ?? 0
  if (length === 0) return
  blockFocusedIndex.value = Math.max(blockFocusedIndex.value - 1, 0)
  blockActionError.value = null
}

/**
 * `Space` — đảo trạng thái giữ/loại của khối ĐANG CHỌN. Đây LÀ một lượt ghi THẬT (§Always
 * spec 6.9: trạng thái phải đi xuống Rust) — chặn khi một lượt khác đang bay, cùng khuôn bốn
 * cờ CRUD luật làm sạch ([`toggleImportPreviewCleanupRule`]).
 */
export async function toggleImportPreviewBlockKept(): Promise<void> {
  if (confirming.value || blockToggling.value) return
  const blocks = importPreviewSelectedBlocks.value?.blocks
  // 🔵 SỬA (dọn nợ lint phát hiện khi làm Story 6.10a, KHÔNG liên quan tới con trỏ Chương) —
  // `tsconfig.json` không bật `noUncheckedIndexedAccess`, nên `blocks[i]` được TypeScript
  // gõ THẲNG là `BlockWire` (không `| undefined`) — một phép so `current === undefined` sau
  // đó là "logic không bao giờ đúng" THẬT theo kiểu tĩnh (`@typescript-eslint/no-unnecessary-condition`
  // đúng khi báo lỗi), dù Ý ĐỊNH chạy (chặn `blockFocusedIndex` ngoài phạm vi) vẫn hợp lệ. So
  // trực tiếp với `blocks.length` diễn đạt ĐÚNG ý định đó bằng một kiểu THẬT SỰ đúng.
  if (blocks === undefined || blockFocusedIndex.value < 0 || blockFocusedIndex.value >= blocks.length) {
    return
  }
  const current = blocks[blockFocusedIndex.value]

  blockToggling.value = true
  try {
    const result = await tier2BlockSetKept(blockFocusedIndex.value, !current.kept, pendingSourceLang.value)
    if (result.error !== null) {
      blockActionError.value = result.error
      return
    }
    if (result.batch === null) return
    blockActionError.value = null
    applyUrlImportBatch(result.batch)
  } finally {
    blockToggling.value = false
  }
}

/** `[` — đặt mốc ĐẦU vùng giữ tại khối đang chọn. KHÔNG gọi Rust (chỉ một mốc CỤC BỘ chờ
 * `]`) — cùng lý do [`selectImportPreviewCandidate`] không gọi Rust. */
export function markImportPreviewBlockRangeStart(): void {
  const length = importPreviewSelectedBlocks.value?.blocks.length ?? 0
  if (length === 0) return
  blockRangeStart.value = blockFocusedIndex.value
  blockRangeMissingStartNotice.value = false
}

/**
 * `]` — đặt dải `[blockRangeStart, blockFocusedIndex]` thành giữ, mọi khối NGOÀI dải thành
 * loại, MỘT LƯỢT (I/O Matrix spec 6.9). Chưa có mốc `[` ⇒ **kêu, không ném**
 * ([`blockRangeMissingStartNotice`] bật, 0 lời gọi IPC) — đúng I/O Matrix "`]` trước `[`".
 */
export async function confirmImportPreviewBlockRange(): Promise<void> {
  if (confirming.value || blockRangeConfirming.value) return
  const length = importPreviewSelectedBlocks.value?.blocks.length ?? 0
  if (length === 0) return
  if (blockRangeStart.value === null) {
    blockRangeMissingStartNotice.value = true
    return
  }

  blockRangeConfirming.value = true
  try {
    const result = await tier2BlockConfirmRange(
      blockRangeStart.value,
      blockFocusedIndex.value,
      length,
      pendingSourceLang.value,
    )
    if (result.error !== null) {
      blockActionError.value = result.error
      return
    }
    if (result.batch === null) return
    blockActionError.value = null
    blockRangeStart.value = null
    applyUrlImportBatch(result.batch)
  } finally {
    blockRangeConfirming.value = false
  }
}

/** `R` — nhảy sang tầng 3 (§Spec Change Log spec 6.9: KHÔNG khớp luật theo khối, một điều
 * hướng thật). `ImportPreviewOverlay.vue` watch [`importPreviewJumpToCleanupRulesSignal`] để
 * cuộn/đặt tiêu điểm — state module không cầm DOM. */
export function jumpImportPreviewToCleanupRules(): void {
  jumpToCleanupRulesSignal.value += 1
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
  // 🔴 SỬA 2026-09-07 (vòng rà bước 4, mục 11) — mỗi ứng viên mang MỘT dãy khối RIÊNG (cùng
  // Chương, khác bảng mã dựng chữ) — một cảnh báo/lỗi tầng 2 đứng từ ứng viên CŨ không còn
  // gắn với dữ liệu người dùng đang nhìn thấy sau khi đổi ô.
  blockRangeMissingStartNotice.value = false
  blockActionError.value = null
  // 🔴 THÊM (Story 6.10a) — con trỏ Chương GIỮ NGUYÊN qua một lượt đổi ứng viên (AC spec
  // 6.10a: "hiện Chương k, không nhảy về Chương 0"); chi tiết của Chương k > 0 phải dựng LẠI
  // với bảng mã MỚI — Chương 0 không cần (đọc thẳng `candidate.cleanup`/`.blocks` mới, đã đủ).
  if (chapterCursor.value !== 0) void loadImportPreviewChapterDetail(chapterCursor.value)
}

/**
 * Token của lượt gọi [`loadImportPreviewChapterDetail`] GẦN NHẤT — **THÊM (vòng rà đối
 * kháng bước 4, P3)**. TÁCH khỏi `sequence` (ô đó mang nghĩa "phiên xem trước": mở/huỷ/xác
 * nhận) vì hai lượt gọi chi tiết CHO CÙNG index (đổi ứng viên bảng mã hai lần liên tiếp, hoặc
 * một lượt `syncChapterCursorAfterUrlBatch` xen vào giữa một lượt đổi ứng viên) không đổi
 * `sequence` — bản trước so `chapterCursor.value !== index`, thứ KHÔNG phân biệt được hai
 * lượt gọi cùng index, nên lượt trả về SAU CÙNG thắng bất kể nó cũ hơn.
 */
let chapterDetailRequestToken = 0

/**
 * Dựng lại chi tiết tầng 2/3 cho Chương thứ `index` — **Story 6.10a**, chỗ gọi sản phẩm là
 * [`moveImportPreviewChapterCursor`]/[`selectImportPreviewCandidate`] (đổi ứng viên khi con
 * trỏ khác 0)/[`syncChapterCursorAfterUrlBatch`]. Chương 0 đọc THẲNG từ `candidate.cleanup`/
 * `.blocks` (đã có sẵn EAGER) — hàm này chỉ DỌN hai ô override, KHÔNG gọi Rust.
 *
 * 🔴 **CHỈ hoạt động trên đường URL** (`lastSubmittedFrom === 'urls'`) — lệnh
 * `preview_chapter_detail` phía Rust đọc `UrlImportItemsState`, không có nhánh cho đường
 * tệp/dán tay (`Blob` + `chapter_pattern`, có thể N > 1 Chương nhưng chi tiết Chương k > 0
 * CHƯA dựng ở story này — nợ MỚI, ghi ở `deferred-work.md`, không phải một sơ suất im lặng).
 *
 * 🔴 **SỬA (vòng rà đối kháng bước 4, P2) — lỗi/trạng thái CŨ phải DỌN chi tiết đang hiện.**
 * Bản trước `return` trên cả hai nhánh `result.error !== null`/`result.detail === null` mà
 * không đụng `chapterDetailCleanup`/`chapterDetailBlocks` — nhưng con trỏ đã dời sang `index`
 * TRƯỚC lượt gọi này, nên tầng 2/3 tiếp tục hiện chi tiết của Chương CŨ dưới nhãn "Chương
 * `index`" (kèm một dòng lỗi, nếu có). Rỗng CÓ LÝ DO, không phải nội dung sai — cùng nguyên
 * tắc mà nửa Rust (`display_window_for_chapter`/`chapter_detail_for_index`) đã theo (trả
 * `None` thay vì đoán).
 *
 * 🔵 **SỬA (vòng rà đối kháng bước 4, P4) — gửi `chapterPattern: null`, không còn
 * `chapterPatternWire()`.** Đường eager (`url_import_encoding_preview`, `project.rs`) truyền
 * `chapter_pattern: None` CỨNG cho MỌI ứng viên trên đường URL — `PipelineShape::Chapters`
 * luôn `already_chaptered = true` nên `Step::SplitChapters` bỏ qua tham số này VÔ ĐIỀU KIỆN
 * (`pipeline.rs::split_chapters_step`, nhánh `already_chaptered` return sớm). Gửi
 * `chapterPatternWire()` ở đây tạo ra hai đầu vào KHÁC NHAU cho Chương 0 (eager, `None`) và
 * Chương k (lazy, mẫu ĐANG GÕ) trên GIẤY — vô hại HÔM NAY vì tham số bị bỏ qua như nhau ở cả
 * hai, nhưng là đúng lớp sai lệch mà story này tồn tại để chặn nếu `split_chapters_step` đổi
 * hành vi sau này. Khớp NGUYÊN VĂN đường eager: `null`.
 */
async function loadImportPreviewChapterDetail(index: number): Promise<void> {
  chapterDetailRequestToken += 1
  const myToken = chapterDetailRequestToken
  chapterDetailError.value = null
  if (index === 0) {
    chapterDetailCleanup.value = null
    chapterDetailBlocks.value = null
    return
  }
  if (lastSubmittedFrom.value !== 'urls') return // giới hạn thật, xem doc-comment hàm này
  const encoding = selectedEncoding.value
  if (encoding === null) return

  const mySequence = sequence
  chapterDetailLoading.value = true
  try {
    const result = await previewChapterDetail(index, encoding, pendingSourceLang.value, null)
    // Một lượt mở/huỷ MỚI (sequence đổi) hoặc một lượt dựng chi tiết KHÁC (kể cả cho CÙNG
    // index — token đổi bất kể `chapterCursor` có đổi hay không) đã vượt mặt lượt này — kết
    // quả trễ không còn khớp bất kỳ thứ gì đang hiện.
    if (mySequence !== sequence || myToken !== chapterDetailRequestToken) return
    if (result.error !== null) {
      // P2 — dọn chi tiết ĐANG HIỆN (của Chương/ứng viên CŨ): rỗng có lý do, không phải nội
      // dung sai gắn nhãn Chương mới.
      chapterDetailCleanup.value = null
      chapterDetailBlocks.value = null
      chapterDetailError.value = result.error
      return
    }
    if (result.detail === null) {
      // P2 — cùng lý do trên: trạng thái CŨ (N vừa đổi dưới chân) — không đoán, không giữ lại
      // chi tiết của một Chương/ứng viên khác dưới nhãn "Chương index".
      chapterDetailCleanup.value = null
      chapterDetailBlocks.value = null
      return
    }
    chapterDetailCleanup.value = result.detail.cleanup
    chapterDetailBlocks.value = result.detail.blocks
  } finally {
    if (myToken === chapterDetailRequestToken) chapterDetailLoading.value = false
  }
}

/**
 * Dời con trỏ Chương — `direction` `+1` (`⌥→`) hoặc `-1` (`⌥←`). **Dừng ở hai đầu, KHÔNG cuộn
 * vòng** (§Never spec 6.10a) — không kêu, không lời gọi IPC. No-op khi lớp phủ đã đóng hoặc
 * một lượt dựng chi tiết KHÁC đang bay (chặn chồng lệnh — lớp phòng thủ THỨ HAI, cạnh guard
 * `event.repeat` ở tầng `.vue`).
 *
 * 🔴 **No-op ngoài đường URL, kể cả khi N > 1** (đường tệp/dán tay + mẫu phân tách CÓ THỂ
 * tách ra N > 1 Chương — tầng 4 vẫn hiện tóm tắt đủ N). Chi tiết Chương k > 0 của hình dạng
 * đó CHƯA dựng ở story này ([`loadImportPreviewChapterDetail`] chỉ gọi Rust trên đường URL) —
 * dời con trỏ ở đây sẽ làm tầng 2/3 hiện RỖNG thay vì Chương 0 đã biết, một hồi quy TỆ HƠN
 * "chưa dựng". An toàn duy nhất: giữ con trỏ đứng yên ở 0, cùng tinh thần I/O Matrix spec
 * 6.10a hàng "N = 1 — con trỏ tồn tại nhưng không đi đâu được", tổng quát hoá cho MỌI N trên
 * đường này. Nợ MỚI (mở rộng chi tiết lazy sang đường tệp/dán tay), ghi ở `deferred-work.md`.
 */
function moveImportPreviewChapterCursor(direction: 1 | -1): void {
  if (!overlayOpen.value) return
  if (lastSubmittedFrom.value !== 'urls') return
  if (chapterDetailLoading.value) return
  const chapters = importPreviewSelectedChapters.value
  if (chapters === null || chapters.chapter_count === 0) return
  const next = chapterCursor.value + direction
  if (next < 0 || next >= chapters.chapter_count) return
  chapterCursor.value = next
  void loadImportPreviewChapterDetail(next)
}

/** `⌥→` — handler của `import.preview.chapter_next`. */
export function nextImportPreviewChapter(): void {
  moveImportPreviewChapterCursor(1)
}

/** `⌥←` — handler của `import.preview.chapter_prev`. */
export function prevImportPreviewChapter(): void {
  moveImportPreviewChapterCursor(-1)
}

/**
 * Giữ con trỏ Chương trong phạm vi hợp lệ SAU một lượt dựng lại xem trước đường URL (tải
 * lại/bỏ một mục, đặt/gỡ override tầng 2) — số Chương có thể đổi dưới chân con trỏ. Chỉ số
 * VƯỢT QUÁ bị KẸP về Chương CUỐI (danh sách rỗng ⇒ về 0) — luôn dựng lại chi tiết cho vị trí
 * cuối cùng, kể cả khi vị trí không đổi (dữ liệu bảng mã/khối phía dưới có thể đã đổi).
 */
function syncChapterCursorAfterUrlBatch(): void {
  const chapters = importPreviewSelectedChapters.value
  const count = chapters?.chapter_count ?? 0
  if (count === 0) {
    chapterCursor.value = 0
    chapterDetailCleanup.value = null
    chapterDetailBlocks.value = null
    chapterDetailError.value = null
    return
  }
  if (chapterCursor.value >= count) chapterCursor.value = count - 1
  void loadImportPreviewChapterDetail(chapterCursor.value)
}

/** Mẫu phân tách Chương hiện hành, dạng dây — chuỗi rỗng (hoặc CHỈ khoảng trắng) ⇒ `null`
 * (không mẫu, no-op, N = 1). Chỗ gọi DUY NHẤT khi cần gửi tham số `chapterPattern` cho một
 * trong ba lệnh IPC.
 *
 * 🔴 **SỬA (vòng rà đối kháng 3, mục 5) — gác bằng `.trim()`, không `.length` trần.** Bản
 * trước chỉ kiểm `length === 0`, nên một ô CHỈ CÓ khoảng trắng bị gửi đi như một mẫu literal
 * THẬT (một mẫu vô nghĩa với người dùng, nhưng vẫn hợp lệ ở tầng dây) — lệch với chính hai ô
 * luật làm sạch (`onAddCleanupRule`/`onSaveEditCleanupRule` ở `.vue`), vốn gác bằng
 * `.trim() === ''`. Giá trị GỬI ĐI vẫn NGUYÊN VĂN (`chapterPatternText.value`, không trim) —
 * `.trim()` chỉ dùng để XÉT rỗng, cùng quy ước hai ô luật làm sạch (khoảng trắng ĐẦU/CUỐI một
 * mẫu literal thật có thể có nghĩa, ví dụ `"Chuong "`). */
function chapterPatternWire(): ChapterPatternInput | null {
  if (chapterPatternText.value.trim().length === 0) return null
  return { pattern: chapterPatternText.value, kind: chapterPatternKind.value }
}

/**
 * Lõi DÙNG CHUNG của mọi lượt "tải lại xem trước bằng ĐÚNG nguồn đang treo" — **THÊM (Story
 * 6.5, mở rộng Story 6.6)**. Gửi lại `chapterPatternWire()` HIỆN HÀNH ở MỌI lượt gọi (dù do
 * một lượt CRUD luật làm sạch hay một lượt sửa mẫu kích hoạt) — hai tầng không trôi khỏi
 * nhau: sửa luật không được âm thầm làm rớt mẫu đang gõ, và ngược lại. `null` khi chưa mở
 * lượt xem trước nào (không có gì để tải lại).
 */
async function runImportPreviewReload(): Promise<{ result: ImportEncodingPreviewResult; mySequence: number } | null> {
  const from = lastSubmittedFrom.value
  if (from === null) return null // chưa mở lượt xem trước nào — không có gì để tải lại

  // 🔴 **THÊM (Story 6.7)** — nhánh URL KHÔNG có một lệnh "tải lại xem trước, giữ nguyên byte
  // đã tải" (thân hàm dưới đây chỉ biết `previewImportEncodingFromText`/`_from_file`, hai
  // lệnh nhận lại NGUYÊN VĂN dán tay/đường dẫn — URL không có "nguyên văn" kiểu đó, N link
  // đã tải sống trong `UrlImportItemsState` phía Rust). Một lượt CRUD luật làm sạch hay sửa
  // mẫu phân tách trong lúc xem một lượt URL vì thế KHÔNG dựng lại xem trước — GIỚI HẠN THẬT,
  // ghi ra thay vì giả vờ nó hoạt động: người dùng sửa luật làm sạch trong khi màn URL đang mở
  // sẽ không thấy khối làm sạch cập nhật cho tới lượt xác nhận thật (luật vẫn được NẠP LẠI
  // đúng lúc xác nhận, `confirm_import_with_encoding` luôn đọc luật NGAY LÚC XÁC NHẬN — chỉ
  // riêng BẢN XEM TRƯỚC không tự làm mới).
  if (from === 'urls') return null

  sequence += 1
  const mySequence = sequence
  const pattern = chapterPatternWire()

  const result =
    from === 'text'
      ? await previewImportEncodingFromText(pendingText.value ?? '', pendingSourceLang.value, pattern)
      : await previewImportEncodingFromFile(pendingPath.value ?? '', pendingSourceLang.value, pattern)
  return { result, mySequence }
}

/**
 * Dựng lại xem trước SAU MỘT LƯỢT CRUD LUẬT LÀM SẠCH — chỗ gọi sản phẩm DUY NHẤT là bốn hàm
 * CRUD luật ngay dưới. Khác `openWith`: KHÔNG đổi
 * `lastSubmittedFrom`/`pendingName`/`pendingSourceLang`/`pendingGenre` (đây là một lượt TẢI
 * LẠI, không phải một lượt MỞ mới), và cố giữ nguyên ứng viên đang chọn nếu nó vẫn còn trong
 * dải mới.
 */
async function reloadImportPreviewAfterRuleChange(): Promise<void> {
  // Một lượt tải lại (dù do THÊM/SỬA/BẬT-TẮT nào gọi tới) làm tan mọi "chờ xác nhận xoá" còn
  // đứng trên MỘT hàng khác — danh sách sắp được dựng lại từ đầu, một khoá cũ trỏ vào một
  // luật có thể đã đổi hình dạng không nên tiếp tục hiện "bấm lại để xoá thật".
  cleanupDeletePendingKey.value = null

  const keepEncoding = selectedEncoding.value
  const outcome = await runImportPreviewReload()
  if (outcome === null) return
  const { result, mySequence } = outcome
  if (mySequence !== sequence) return // một lượt mở/huỷ/tải lại MỚI đã vượt mặt lượt này

  if (result.error !== null) {
    if (result.error.code === CHAPTER_PATTERN_INVALID_CODE) {
      // 🔴 SỬA (vòng rà đối kháng 3, mục 2) — xem doc-comment [`CHAPTER_PATTERN_INVALID_CODE`].
      // Lỗi này là CỦA Ô MẪU, không phải của luật làm sạch vừa đổi — không được lật
      // `status`/`preview` vì một nguyên nhân KHÁC tầng.
      chapterPatternError.value = result.error
      return
    }
    status.value = 'error'
    loadError.value = result.error
    return
  }
  // Một lượt tải lại THÀNH CÔNG (dù mẫu phân tách hiện hành có hay không) chứng minh mẫu
  // ĐANG GỬI biên dịch được — một `chapterPatternError` cũ (nếu còn từ một lượt sửa mẫu
  // trước) đã hết hiệu lực, cùng logic thành công của [`reloadImportPreviewAfterChapterPatternChange`].
  chapterPatternError.value = null
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
 * Dựng lại xem trước SAU MỘT LƯỢT SỬA MẪU PHÂN TÁCH — **THÊM (Story 6.6)**, chỗ gọi sản
 * phẩm DUY NHẤT là [`setImportPreviewChapterPattern`] ngay dưới. Tái dùng ĐÚNG đường
 * [`runImportPreviewReload`] mà luật làm sạch đã dùng (§Design Notes spec 6.6: *"'Cập nhật
 * ngay' ĐÃ có cơ chế, đừng dựng cái thứ hai"*) — chỉ khác Ở CÁCH XỬ LÝ LỖI: một mẫu regex
 * không biên dịch được phải GIỮ NGUYÊN kết quả CŨ (§I/O Matrix spec 6.6), nên nhánh lỗi ở
 * đây KHÔNG đụng `status`/`loadError`/`preview` — chỉ báo lỗi RIÊNG qua `chapterPatternError`.
 */
async function reloadImportPreviewAfterChapterPatternChange(): Promise<void> {
  const keepEncoding = selectedEncoding.value
  const outcome = await runImportPreviewReload()
  if (outcome === null) return
  const { result, mySequence } = outcome
  if (mySequence !== sequence) return

  if (result.error !== null) {
    // 🔴 KHÔNG đụng `status`/`loadError`/`preview` — xem doc-comment hàm này.
    chapterPatternError.value = result.error
    return
  }
  chapterPatternError.value = null
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

/** Lượt gõ CUỐI CÙNG đến trong khi một lượt sửa mẫu KHÁC còn đang bay — **THÊM (vòng rà đối
 * kháng 3, mục 4)**. `null` khi không có gì đang chờ. Chỉ giữ lượt CUỐI: một lượt trung gian
 * (nếu có ba lượt @change dồn lại trong lúc lượt đầu bay) bị GHI ĐÈ có chủ ý — chỉ giá trị
 * SAU CÙNG có ý nghĩa gửi lên Rust, cùng khuôn "vé `sequence`" mà module này dùng khắp nơi để
 * bỏ qua kết quả CŨ hơn. */
let pendingChapterPatternEdit: { text: string; kind: ChapterPatternKindWire } | null = null

/**
 * Sửa mẫu phân tách Chương — lệnh `import.preview.set_chapter_pattern` (`@change` của ô
 * nhập/chọn kind, KHÔNG `@click`, AD-34). Đúng MỘT vòng IPC khi giá trị THẬT SỰ đổi — một
 * lượt `@change` không đổi gì (ví dụ blur không sửa gì) không gọi Rust lần nào.
 *
 * 🔴 **SỬA (vòng rà đối kháng 3, mục 4) — một lượt `@change` đến trong lúc lượt trước còn bay
 * KHÔNG còn bị nuốt im lặng.** Bản trước `return` sớm khi `chapterPatternSending`, không ghi
 * gì lại — ô nhập và trạng thái đã cam kết (`chapterPatternText`) trôi khỏi nhau vĩnh viễn
 * nếu lượt đó không bao giờ được gõ lại. Từ bản này, một lượt đến trong lúc đang bay được XẾP
 * HÀNG vào [`pendingChapterPatternEdit`] (đè lượt cũ nếu có) và tự chạy lại NGAY sau khi lượt
 * đang bay xong (`finally` bên dưới) — không cần một `@change` khác kích hoạt.
 */
export async function setImportPreviewChapterPattern(
  text: string,
  kind: ChapterPatternKindWire,
): Promise<void> {
  if (confirming.value) return
  if (chapterPatternSending.value) {
    pendingChapterPatternEdit = { text, kind }
    return
  }

  const changed = text !== chapterPatternText.value || kind !== chapterPatternKind.value
  chapterPatternText.value = text
  chapterPatternKind.value = kind
  if (changed) {
    chapterPatternSending.value = true
    try {
      await reloadImportPreviewAfterChapterPatternChange()
    } finally {
      chapterPatternSending.value = false
    }
  }

  // Một lượt gõ MỚI đã xếp hàng trong lúc lượt này (nếu `changed`) còn bay — chạy nó NGAY,
  // đúng MỘT lượt đệ quy mỗi lần (một lượt thứ ba xếp chồng trong lúc lượt NÀY chạy sẽ được
  // xử lý bởi CHÍNH lượt đệ quy này, không phải ở đây).
  if (pendingChapterPatternEdit !== null) {
    const next = pendingChapterPatternEdit
    pendingChapterPatternEdit = null
    await setImportPreviewChapterPattern(next.text, next.kind)
  }
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
 *
 * 🔴 **SỬA (vòng rà đối kháng 3, mục 3) — chặn khi `chapterPatternSending` còn bay.** Tham số
 * gửi đi ([`chapterPatternWire`]) đọc TRỰC TIẾP `chapterPatternText`/`chapterPatternKind` —
 * cùng ô mà một lượt tải lại mẫu ĐANG BAY có thể đang đọc ĐỂ GỬI một giá trị KHÁC. Không chặn
 * ở đây, `create_work` có thể ghi bằng một mẫu MỚI HƠN mẫu của chính bản xem trước đang hiện
 * trên màn hình — phá thẳng AC "xem trước và xác nhận trùng nhau từng byte". Cùng lý do
 * `confirming` đã chặn bốn hành động CRUD luật làm sạch phía trên.
 *
 * 🔵 **SỬA (Story 6.10a) — gác bằng [`importPreviewCanConfirm`] (vị từ GHI), KHÔNG còn
 * `preview.value === null` trần.** Xem doc-comment computed đó: trên đường URL, `preview`
 * nay có thể khác `null` dù còn mục hỏng (vị từ XEM đã đổi) — gác bằng nó ở đây sẽ cho một
 * lượt xác nhận ĐI QUA tầng state trong khi Rust vẫn từ chối (`no_pending_source`, đúng
 * nhưng TRỄ một vòng IPC), và trên màn hình nút xác nhận phải đọc ĐÚNG cùng vị từ này.
 */
export async function confirmImportPreview(): Promise<{ created: CreatedWork | null; error: IpcError | null }> {
  if (
    confirming.value ||
    chapterPatternSending.value ||
    !importPreviewCanConfirm.value ||
    selectedEncoding.value === null
  ) {
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
    chapterPatternWire(),
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
  chapterPatternText.value = ''
  chapterPatternKind.value = 'literal'
  chapterPatternSending.value = false
  chapterPatternError.value = null
  pendingChapterPatternEdit = null
  urlImportItems.value = []
  domainLogDomainCount.value = 0
  urlImportBusy.value = false
  urlImportError.value = null
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
  // P3 (vòng rà đối kháng bước 4) — vô hiệu hoá mọi lượt gọi `loadImportPreviewChapterDetail`
  // ĐANG BAY, cùng lý do `sequence += 1` ngay trên (một lượt huỷ/mở MỚI làm mọi kết quả trễ
  // hết còn khớp bất kỳ thứ gì đang hiện).
  chapterDetailRequestToken += 1
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
  chapterPatternText.value = ''
  chapterPatternKind.value = 'literal'
  chapterPatternSending.value = false
  chapterPatternError.value = null
  pendingChapterPatternEdit = null
  urlImportItems.value = []
  domainLogDomainCount.value = 0
  urlImportBusy.value = false
  urlImportError.value = null
  blockFocusedIndex.value = 0
  blockRangeStart.value = null
  blockRangeMissingStartNotice.value = false
  blockToggling.value = false
  blockRangeConfirming.value = false
  blockActionError.value = null
  jumpToCleanupRulesSignal.value = 0
  chapterCursor.value = 0
  chapterDetailCleanup.value = null
  chapterDetailBlocks.value = null
  chapterDetailLoading.value = false
  chapterDetailError.value = null
}
