/**
 * State của Panel Editor — segment đã nạp, câu đang có tiêu điểm. Story 2.2, AC3 · AC5 · AC13.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY, KHÔNG TRONG `GridPanel.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do và cùng khuôn `sourcePanelState.ts` (Story 1.16 · AC9): một lượt đổi preset bố
 * cục chạy `WorkspaceDock.vue::applyPreset()` → `api.clear()` rồi dựng lại **cả bốn** panel,
 * tức tháo và mount lại instance `GridPanel.vue`. Một `ref` khai trong `<script setup>`
 * của nó chết cùng lượt tháo đó, và người dùng trả giá bằng một lượt IPC nạp lại toàn bộ
 * segment của Chương *(đo được: **9.850** hàng cho Chương lớn nhất có thật)*.
 *
 * ⚠️ Cùng lý do `sourcePanelState.ts` ghi: tệp này dùng `ref` của Vue, nên nó **không** được
 * `import` vào `src/commands/index.ts` — Kiểm C/D/E của `npm run check:commands` nạp tệp đó
 * bằng Node thuần.
 *
 * ⚠️ Bảng ánh xạ *trạng thái → vạch* **không** ở đây: nó sống ở `./editorSegments.ts`, một
 * module thuần mà cổng `import()` chạy được. Đừng kéo nó về đây cho gần.
 */
import { nextTick, readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { enterFocus } from '../commands'
import { openAdjacentChapter, openChapter, splitChapterAtSegment } from '../config/chapter'
import type { ChapterDirection } from '../config/chapter'
// 🔵 CODE REVIEW 2026-08-18 — lượt đổi CHƯƠNG phải dọn và nạp lại Panel Source, đúng như lượt
// đổi TÁC PHẨM đã làm ở `modes/libraryImport.ts`. Xem khối lý do trong [`switchChapter`].
// ⚠️ Không vòng: `sourcePanelState.ts` chỉ import `config/*` và `i18n`, không import tệp này.
import { ensureChapterLoaded, resetSourcePanel, sourceChapter } from './sourcePanelState'
import {
  confirmSegment,
  mergeSegments,
  readOpenChapterSegments,
  saveChapterPosition,
  saveSegmentTargets,
  setSegmentOmitted,
  setSegmentParagraphEnd,
  splitSegment,
} from '../config/segment'
import type { ChapterSegment, RegroupOutcome, SegmentTargetEdit } from '../config/segment'
import type { IpcError } from '../i18n'
import { createEditorFlush, EDITOR_RETRY_FLOOR_MS } from './editorFlush'
// 🔵 THÊM Story 5.7 (AC4/AC6) — nhịp ghi RIÊNG cho vị trí làm việc của Chương, KHÔNG mang
// bảo đảm AD-35. Xem doc-comment đầu `positionFlush.ts`.
import { createPositionFlush } from './positionFlush'
// 🔵 Story 3.4b — funnel của "dấu thuật ngữ Glossary": `switchChapter()` nạp lại sau khi
// reset (cạnh `ensureChapterLoaded()`), `applyRegroup()` làm mới sau gộp/tách. Xem doc-comment
// đầu `glossaryMarksState.ts` cho lý do tệp đó KHÔNG import ngược lại tệp này.
import { ensureGlossaryMarksLoaded, refreshGlossaryMarks, resetGlossaryMarks } from './glossaryMarksState'
// 🔵 Story 3.6 — dải "Chờ chốt lần đầu gặp" mang một sổ "Để sau" phạm vi ĐÚNG MỘT Chương;
// hai chỗ dọn dấu Glossary (đổi Chương/Tác phẩm, gộp/tách) cũng là hai chỗ dọn sổ đó. Xem
// doc-comment đầu `../glossaryConfirmStripState.ts` cho lý do tệp đó KHÔNG import ngược lại
// tệp này (cùng lý do `glossaryMarksState.ts`).
import { resetGlossaryConfirmStrip } from '../glossaryConfirmStripState'
import {
  navigationSegmentOf,
  nextSegmentId,
  nextUntranslatedId,
  prevSegmentId,
} from './segmentNavigation'
import type { NavigationSegment } from './segmentNavigation'

const segments = shallowRef<readonly ChapterSegment[]>([])
const chapterId = shallowRef<number | null>(null)
const loadError = shallowRef<IpcError | null>(null)
const pending = ref(false)
let requested = false

/**
 * 🔴 **Số thứ tự lượt nạp** — cùng cơ chế và cùng lý do với `hanVietSequence` của
 * `sourcePanelState.ts` *(bắt ở code review 2026-08-07)*.
 *
 * [`resetEditorPanel`] chạy khi Tác phẩm đang mở bị thay; một lượt `read_open_chapter_segments`
 * **đang bay** của Tác phẩm A trả lời **sau** lượt reset sẽ đổ segment của A lên màn hình
 * dưới nhãn Tác phẩm B, và không một dấu hiệu nào báo sai.
 */
let sequence = 0

/**
 * `segment.id` của câu mà con trỏ / tiêu điểm bàn phím đang chạm — nguồn **duy nhất** của
 * giá trị vạch `primary` (AC3) và của `[data-caret]` trong AC5.
 *
 * ⚠️ `null` là trạng thái **bình thường**, không phải "chưa nạp": không câu nào đang được
 * chạm là chuyện thường xuyên nhất trên một panel chỉ-đọc.
 */
const caretSegmentId = ref<number | null>(null)

/** Segment của Chương đang mở, **theo `ord`** — thứ tự do Rust quyết, không sắp lại ở đây. */
export const editorSegments: DeepReadonly<Ref<readonly ChapterSegment[]>> = readonly(segments)
/** `chapter.id` của Chương đang hiện. `null` trước lượt nạp đầu tiên hoặc khi nạp trượt. */
export const editorChapterId: DeepReadonly<Ref<number | null>> = readonly(chapterId)
/**
 * Lỗi gần nhất Rust trả lời cho lượt nạp segment.
 *
 * `null` ở HAI ca khác hẳn nhau — nạp được, **hoặc** không có cầu IPC nào *(chạy ngoài
 * Tauri; `config/segment.ts` nuốt có chủ ý)*. Cùng khuôn `sourceChapterError`.
 */
export const editorLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
/** `true` trong khoảng chờ lượt nạp — xem [`editorResolved`]. */
export const editorPending: DeepReadonly<Ref<boolean>> = readonly(pending)
/** Câu đang có tiêu điểm — xem [`caretSegmentId`]. */
export const editorCaretSegmentId: DeepReadonly<Ref<number | null>> = readonly(caretSegmentId)

/**
 * 🔴 Lượt nạp **đã trả lời và trả lời được** — điều kiện để một danh sách rỗng có nghĩa.
 *
 * Vì sao vị từ này phải tồn tại: `segments` rỗng trong **ba** hoàn cảnh khác hẳn nhau — chưa
 * nạp, đang chờ IPC, và *"Chương này thật sự chưa có segment nào"*. Chỉ hoàn cảnh thứ ba được
 * phép hiện câu *"chưa tách câu nào"*; hai hoàn cảnh kia mà hiện câu đó là màn hình khẳng định
 * dứt khoát một điều nó chưa biết — đúng lỗ mà `hanVietPending` của Story 1.16 tồn tại để bịt.
 *
 * 🔵 **2026-08-18 (code review lượt HAI) — gỡ một dẫn chứng đã hết hạn.** Dòng trên từng viện
 * *"25 Chương của Epic 1, `deferred-work.md:542`"* làm ví dụ cho hoàn cảnh thứ ba. Món nợ ấy
 * **đã ĐÓNG 2026-08-12 ở Story 2.1** *(bước di trú 5 + hai đường tách, không đường nào tính
 * ngầm)*, nên nó không còn chứng minh được gì. Mệnh đề **ba hoàn cảnh** thì vẫn đứng — chỉ dẫn
 * chứng chết. ⚠️ Một lượt rà đã đọc chính dòng này thành *"Chương rỗng chạm tới được"*: một dẫn
 * chứng hết hạn để lại trong mã **không nằm im**, nó đi tiếp thành một kết luận sai.
 *
 * 🔴 **KHÔNG CỔNG NÀO CANH VỊ TỪ NÀY ĐƯỢC DÙNG.** Nó là một hàm export mà chỗ quên gọi vẫn biên
 * dịch sạch — và đường điều hướng của Story 2.10 **đã quên** đúng một lượt *(vá 2026-08-18, xem
 * [`dieuHuongVaBao`])*. Thêm một bề mặt đọc `segments` thì hỏi trước: nó đã đi qua đây chưa.
 */
export function editorHasLoaded(): boolean {
  return !pending.value && loadError.value === null && chapterId.value !== null
}

/**
 * Nạp segment của Chương đang mở — **idempotent**: gọi lại ở mỗi lượt mount (kể cả sau một
 * lượt đổi preset, AC9 của Story 1.16) chỉ chạy IPC ở lượt ĐẦU TIÊN.
 */
export async function ensureSegmentsLoaded(): Promise<void> {
  if (requested) return
  requested = true

  // Đặt TRƯỚC `await` — xem [`sequence`].
  const mine = ++sequence
  pending.value = true

  const { loaded, error } = await readOpenChapterSegments()

  // 🔴 Lượt này đã bị [`resetEditorPanel`] vượt mặt. Thoát **trước** mọi lượt gán, kể cả
  // `pending`: nhả cờ chờ hộ một lượt mới đang bay là dựng lại đúng lỗ mà cờ đó tồn tại để
  // bịt. Cũng **không** nhả `requested` — lượt mới đang giữ nó một cách hợp lệ.
  if (mine !== sequence) return

  pending.value = false
  segments.value = loaded?.segments ?? []
  chapterId.value = loaded?.chapter_id ?? null
  loadError.value = error

  // 🔵 **THÊM Story 5.7 (AC4/AC5).** `caret_segment_id` là RUST QUYẾT — segment đã lưu vị
  // trí, segment đầu (Chương chưa từng mở / vị trí trỏ vào segment về hưu), hoặc `null`
  // (Chương rỗng). Đặt tín hiệu cho watcher `editorCaretPlacement` của `GridPanel.vue` đặt
  // caret VÀ cuộn qua `focus()` — không một hàm cuộn thứ hai (xem doc-comment của watcher
  // đó, §AC8 nửa sau). Đặt Ở ĐÂY, không riêng ở `openChapterById`: mọi đường tới đây —
  // mount Workspace lần đầu, `switchChapter`, `openChapterById` — đều hợp lệ khôi phục vị
  // trí làm việc, và `readOpenChapterSegments()` luôn mang giá trị này về cùng một lượt đọc.
  caretPlacement.value = loaded?.caret_segment_id ?? null

  // 🔴 Một lượt TRƯỢT không được khoá vĩnh viễn đường nạp — cùng dòng và cùng lý do với
  // `ensureHanVietLoaded`. Một lỗi IPC nhất thời (hay một lượt chạy ngoài Tauri) sẽ biến
  // Panel Editor thành trống **mãi mãi**, kể cả khi `error.retryable`.
  if (loaded === null) requested = false
}

/**
 * Đặt câu đang có tiêu điểm. `null` ⇒ không câu nào.
 *
 * ⚠️ Hàm này **không** kiểm `id` có trong danh sách hay không, và đó là có chủ ý: chỗ gọi duy
 * nhất đọc `id` ra từ chính DOM mà `v-for` vừa dựng từ danh sách đó, nên một phép kiểm ở đây
 * chỉ chép lại một bất biến đã có — và nó sẽ chạy trên **9.850** phần tử mỗi lượt rê chuột.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 ĐÂY LÀ ĐỊNH NGHĨA CỦA *"RỜI SEGMENT"* — Story 2.3 · AC18, và nó nằm ở ĐÚNG MỘT CHỖ
 * ─────────────────────────────────────────────────────────────────────────────
 * AC3 của AD-35 liệt kê *"rời segment"* là một trong bốn đường flush ngay. Nhưng AC1 của
 * Story 2.2 cấm ô, bảng và khối — Panel Editor là **một trang liền mạch** — nên **không có
 * một widget nào để "rời"**. Định nghĩa duy nhất còn lại, và nó đã có sẵn nguồn dữ liệu:
 * **`caretSegmentId` đổi giá trị**. Caret đi từ câu A sang câu B **là** một lượt rời A.
 *
 * ⚠️ Cùng sự kiện đó cũng dời `contenteditable` sang câu B *(hệ quả 5 của Quyết định #1,
 * đường (c) do Ice ký)*. Hai đường riêng cho cùng một sự kiện sẽ lệch nhau ở lượt sửa thứ
 * hai — nên vế *"dời vùng gõ"* đọc `editorCaretSegmentId` chứ **không** tự nghe
 * `selectionchange` lần thứ hai.
 *
 * ⚠️ `null` **không** phải một lượt rời segment cần flush riêng: nó xảy ra mỗi lần người dùng
 * bấm sang panel khác, và tập chờ vẫn còn nguyên cho lượt flush theo nhịp. Nhưng một lượt đổi
 * **từ** một câu **sang một câu khác** thì phải flush ngay — đó là lúc người dùng đã xong với
 * câu cũ.
 */
export function setEditorCaret(id: number | null): void {
  const left = caretSegmentId.value
  caretSegmentId.value = id
  // Rời câu A sang câu B (cả hai khác `null`, và khác nhau) ⇒ flush ngay cho A.
  if (left !== null && id !== null && left !== id) void flushEditorNow()

  // 🔵 THÊM Story 5.7 (AC4/AC6) — mọi lượt caret đổi sang một segment CÓ THẬT ghi lại vị
  // trí làm việc của Chương đang mở, qua nhịp `positionFlush` (KHÔNG mang bảo đảm AD-35,
  // xem doc-comment của tệp đó). `id === null` (rời sang panel khác) không xoá vị trí đã
  // biết — Chương vẫn "đang dở ở câu cuối cùng có tiêu điểm".
  if (id !== null && chapterId.value !== null) {
    positionFlush.markMoved(chapterId.value, id, Date.now())
    armPositionFlushTimer()
  }
}

/**
 * Thay **một vài trường** của một segment trong ảnh chụp hiển thị — Story 2.6.
 *
 * 🔴 **Dựng một MẢNG MỚI**, không sửa tại chỗ: [`segments`] là một `shallowRef`, nó **không**
 * theo dõi một lượt sửa vào bên trong phần tử ⇒ đĩa đổi mà lưới thì không. Story 2.5c mất một
 * vòng chẩn đoán ở đúng chỗ này (commit `4ce5bb4`).
 * ⚠️ Đo lại 2026-08-16 chứ không tin: gỡ luật này *(`Object.assign` tại chỗ)* làm ca
 * `tests/frontend/segmentHistory.test.ts::khôi phục ⇒ editorSegments thay bằng mảng mới` **đỏ**.
 *
 * ⚠️ **Vì sao một hàm chứ không để chỗ gọi tự trải mảng:** lượt khôi phục của Story 2.6 sống ở
 * `segmentHistoryState.ts`, tức **ngoài** tệp này, và `segments` là `readonly` với thế giới
 * bên ngoài *(đúng vậy — nó là nguồn sự thật của lưới)*. Một cửa hẹp mang theo luật *"mảng
 * mới"* tốt hơn một lượt nới `segments` thành ghi được, thứ mở đường cho mọi component tự sửa
 * ảnh chụp theo cách riêng.
 *
 * ⚠️ Segment không có trong ảnh chụp ⇒ **không làm gì**, không ném. Ca đó xảy ra thật khi
 * người dùng đổi Chương trong lúc một lượt ghi đang bay.
 */
export function replaceEditorSegment(
  id: number,
  patch: Partial<Pick<ChapterSegment, 'target_text' | 'status'>>,
): void {
  const index = segments.value.findIndex((s) => s.id === id)
  if (index < 0) return
  const next = [...segments.value]
  next[index] = { ...next[index], ...patch }
  segments.value = next
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 NHỊP FLUSH — AD-35, Story 2.3. VÌ SAO NÓ SỐNG Ở ĐÂY VÀ KHÔNG TRONG `GridPanel.vue`
// ═════════════════════════════════════════════════════════════════════════════════
//
// Cùng lý do và cùng phép đo mà doc-comment đầu tệp đã ghi cho `segments`, nhưng ở đây cái
// giá cao hơn hẳn một bậc: một lượt đổi preset bố cục chạy `api.clear()` rồi dựng lại cả bốn
// panel, tức **tháo** instance `GridPanel.vue`. Một `setTimeout` và một tập chờ khai trong
// `<script setup>` của nó chết cùng lượt tháo đó — và thứ chết theo là **văn bản người dùng
// vừa gõ mà chưa flush**. Đó đúng là cửa sổ mất dữ liệu im lặng mà cả Epic 2 tồn tại để đóng
// (NFR18), chỉ là mở ra bởi một lượt đổi bố cục thay vì một lần sập máy.

/** Nhịp `idle + trần cứng` của AD-35 — một cho cả ứng dụng, sống ngoài vòng đời component. */
const flush = createEditorFlush()

// ═════════════════════════════════════════════════════════════════════════════════
// 🔵 THÊM Story 5.7 (AC4/AC6) — NHỊP GHI VỊ TRÍ LÀM VIỆC CỦA CHƯƠNG, KHÔNG MANG AD-35
// ═════════════════════════════════════════════════════════════════════════════════
// Cùng khuôn nhịp flush ngay trên (một cho cả ứng dụng, sống ngoài vòng đời component),
// nhưng nhịp KHÁC: xem doc-comment đầu `positionFlush.ts` cho vì sao mất một lượt ở đây
// chỉ mất MỘT LỜI NHẮC, không mất công việc.
const positionFlush = createPositionFlush()
let positionFlushTimer: ReturnType<typeof setTimeout> | null = null

function clearPositionFlushTimer(): void {
  if (positionFlushTimer !== null) {
    clearTimeout(positionFlushTimer)
    positionFlushTimer = null
  }
}

function armPositionFlushTimer(): void {
  clearPositionFlushTimer()
  const due = positionFlush.deadline()
  if (due === null) return
  positionFlushTimer = setTimeout(() => {
    positionFlushTimer = null
    void flushChapterPositionNow()
  }, Math.max(0, due - Date.now()))
}

/**
 * Ghi NGAY vị trí đang chờ, nếu có — gọi ở ba biên (Task 15 ②): TRƯỚC lượt đổi Chương, TRƯỚC
 * lượt đổi Tác phẩm, và trong `wireExitFlush`. Không ném; lỗi ghi chỉ là một chẩn đoán
 * (§I/O Matrix "Ghi vị trí": *"Lỗi ghi ⇒ chẩn đoán, KHÔNG hộp thoại"*) — hàm này **không**
 * mang bảo đảm AD-35 nên nó không cần trả một kết quả có tên cho chỗ gọi phân nhánh theo.
 */
export async function flushChapterPositionNow(): Promise<void> {
  clearPositionFlushTimer()
  const toWrite = positionFlush.pending()
  if (toWrite === null) return

  const { error } = await saveChapterPosition(toWrite.chapterId, toWrite.segmentId)
  const now = Date.now()
  if (error === null) {
    positionFlush.onFlushed(now, toWrite)
  } else {
    // ⚠️ KHÔNG DẤU — chẩn đoán, không văn bản hiển thị (NFR16). Không hộp thoại: đây là
    // nhịp ghi KHÔNG mang bảo đảm AD-35.
    console.error(
      `[editorPanelState] ghi chapter_position that bai (chapterId=${toWrite.chapterId}, ` +
        `segmentId=${toWrite.segmentId}): ${error.code}`,
    )
  }
  armPositionFlushTimer()
}

/**
 * Văn bản đang gõ, theo `segment.id` — **state cục bộ frontend**, ngoại lệ *duy nhất, tường
 * minh* của AD-1 (`ARCHITECTURE-SPINE.md:75-79`).
 *
 * 🔴 **Tách rời `segments`, và đó là một mệnh đề của AD-31 chứ không một lượt chia cho gọn.**
 * `segments` giữ bản **lúc nạp segment**, và FR117 (*xuất xứ bản dịch*, Story 2.7) so *"văn
 * bản đích **hiện tại** với bản **lúc nạp**"* — **không** dùng cờ dirty. Ghi văn bản đang gõ
 * đè lên `segments` là huỷ đúng cái mốc mà story đó cần, và nó sẽ hỏng ở Epic sau mà không
 * gì nối được về đây.
 *
 * ⚠️ Đây là nguồn hiển thị: template đọc `editedText.get(id) ?? s.target_text`. Giá trị ở đây
 * luôn được đặt **từ chính DOM** *(`textContent` của câu đang gõ)*, nên lượt render kế tiếp
 * thấy chuỗi **y hệt** thứ đang có trong DOM ⇒ Vue không ghi vào text node đó ⇒ caret không
 * bị đụng. Đó là điều kiện để một `contenteditable` và một binding của Vue cùng sống trên một
 * phần tử mà không tranh nhau.
 */
const editedText = ref(new Map<number, string>())
/** Văn bản đang gõ theo `segment.id`, `undefined` ⇒ chưa ai sửa câu đó trong phiên này. */
export const editorEditedText: DeepReadonly<Ref<Map<number, string>>> = readonly(editedText)

/**
 * Mốc lượt flush **thành công** gần nhất, mili-giây epoch. `null` ⇒ chưa lượt nào.
 *
 * 🔴 Một `number`, **không** một câu — AD-21 (`ARCHITECTURE-SPINE.md:302-306`): Rust không bao
 * giờ trả văn bản hiển thị, và §Consistency Conventions nói **định dạng số và ngày giờ chỉ ở
 * frontend**. Câu *"Đã lưu N giây trước"* dựng ở `StatusBar.vue` từ con số này (AC10).
 */
const lastSavedAt = ref<number | null>(null)
export const editorLastSavedAt: DeepReadonly<Ref<number | null>> = readonly(lastSavedAt)

/** Timer tới `flush.deadline()`. Một cái, không một cái mỗi phím. */
let flushTimer: ReturnType<typeof setTimeout> | null = null

/**
 * Lô đang bay — **chính promise của nó**, không một cờ `boolean`.
 *
 * 🔴 Vì sao một promise chứ không một cờ — code review 2026-08-13. Bản cũ giữ `let inFlight =
 * false` và mở đầu bằng `if (inFlight) return`, tức một lượt gọi thứ hai **trả về ngay, không
 * ghi gì, không chờ gì**. Ba chỗ gọi `await flushEditorNow()` vì thế cầm một lời hứa rỗng:
 * `beginSubmit` *(phải chờ bản dịch cũ chạm WAL trước khi `OpenWorkState` đổi chỗ)* và listener
 * thoát ứng dụng *(phải chờ trước khi cửa sổ đóng)*. Đường mất chữ cụ thể: người dùng gõ tiếp
 * **trong lúc** một lô đang bay ⇒ `markChanged` ghi vào tập chờ ⇒ sự kiện thoát bắn ⇒ lượt gọi
 * thứ hai trả về ngay ⇒ những ký tự đó **chưa từng lên dây**.
 */
let inFlight: Promise<FlushResult> | null = null

/**
 * Kết quả một lượt flush — chỗ gọi cần phân biệt *"đã chạm WAL"* với *"trượt"*.
 *
 * - `saved` — đã qua `Store::write` và commit *(AC5: `Ok` là bằng chứng đã ghi vào WAL)*
 * - `clean` — không có gì để ghi *(tập chờ rỗng, hoặc chưa mở Chương nào)*
 * - `failed` — lượt ghi trượt; tập chờ **giữ nguyên** để lượt sau thử lại
 */
export type FlushResult = 'saved' | 'clean' | 'failed'

function clearFlushTimer(): void {
  if (flushTimer !== null) {
    clearTimeout(flushTimer)
    flushTimer = null
  }
}

/**
 * Đặt lại timer về đúng `deadline()` của lịch.
 *
 * @param floorMs sàn dưới cho độ trễ. `0` ở đường bình thường; [`EDITOR_RETRY_FLOOR_MS`] sau
 * một lượt ghi **trượt** — xem doc-comment của hằng đó cho đường hỏng mà nó đóng.
 */
function armFlushTimer(floorMs = 0): void {
  clearFlushTimer()
  const due = flush.deadline()
  if (due === null) return
  // ⚠️ `due` là một mốc TUYỆT ĐỐI (`Date.now()` cộng thêm), nên độ trễ là một hiệu. Kẹp ở
  // `floorMs`: một mốc đã quá hạn phải chạy ở tick kế tiếp — trừ khi lượt vừa rồi TRƯỢT, và
  // khi đó `due` đứng yên ở quá khứ nên một lượt kẹp ở 0 là một vòng lặp chặt.
  flushTimer = setTimeout(() => {
    flushTimer = null
    void flushEditorNow()
  }, Math.max(floorMs, due - Date.now()))
}

/**
 * Người dùng vừa sửa một câu — **chỗ gọi duy nhất là đường gõ của `GridPanel.vue`**.
 *
 * ⚠️ `Date.now()` đọc **ở đây**, không trong `editorFlush.ts`: module đó là tầng thuần và luật
 * *"mọi thời điểm đi vào qua tham số"* của nó là điều kiện để ba mệnh đề định lượng của AC11
 * kiểm được tất định. Tệp này là tầng có tác dụng phụ — nó được phép đọc đồng hồ.
 */
export function noteEditorEdit(segmentId: number, targetText: string): void {
  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔵 CODE REVIEW BA TẦNG 2026-08-18 — CỬA SỔ MẤT CHỮ CỦA LƯỢT CHUYỂN CHƯƠNG
  //    Ice ký đường (a): KHOÁ GÕ suốt lượt chuyển
  // ═══════════════════════════════════════════════════════════════════════════════
  // [`switchChapter`] chứng minh tập chờ sạch ở bước ① rồi `await` một lượt IPC ở bước ② rồi
  // gọi `resetEditorPanel()` ở bước ③, mà `flush.reset()` bên trong nó **vứt vô điều kiện**.
  // Giữa ② và ③ **không phép kiểm nào chạy lại** — nên một ký tự gõ trong cửa sổ round-trip
  // của ② đi vào tập chờ rồi bị xoá không ghi: không lỗi, không log, không cảnh báo, trên dữ
  // liệu mà AD-5 không cho hoàn tác. HAI tầng rà độc lập cùng chỉ đúng dòng này.
  //
  // 🔴 Vá ở ĐÂY chứ không ở [`switchChapter`], và đó là toàn bộ nội dung của chữ ký:
  // đóng cửa sổ **từ gốc** làm mệnh đề *"① là phép chứng minh duy nhất"* đúng trở lại **theo
  // cấu tạo**. Đường bị loại — kiểm `flush.isDirty()` lại trước ③ rồi flush thêm một lượt —
  // còn một cái đuôi không có lời giải: nếu lượt thêm ấy **vẫn** trượt thì con trỏ Chương
  // phía Rust đã dời rồi và không chặn lại được.
  //
  // ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện:** vài chục ms gõ **không
  // ăn**, và người dùng không được báo. Nhận nó có ý thức — một câu trạng thái cho một cửa sổ
  // ngắn hơn một nhịp phím là nhiễu, và nó đẩy mất mốc *"Đã lưu"* của UX-DR30. Ngày lượt ②
  // chậm tới mức đo được, kết luận này hết đúng: đọc lại kèm số đo, đừng kế thừa nó.
  if (dangChuyenChuong) {
    console.info(
      `[editor] bo qua mot luot go tren segment ${segmentId}: dang chuyen Chuong, tap cho sap bi vut`,
    )
    return
  }

  const next = new Map(editedText.value)
  next.set(segmentId, targetText)
  editedText.value = next

  flush.markChanged(segmentId, targetText, Date.now())
  armFlushTimer()

  // 🔵 2026-08-15 — dọn câu báo của lượt xác nhận gần nhất. Người dùng gõ tiếp **là** câu trả
  // lời cho nó: cả ba giá trị đi vào ô nhớ đó (*chưa chọn câu nào* · *chưa lưu được* · *vừa
  // đổi trong lúc lưu*) đều mô tả một thời điểm đã trôi qua, và giữ chúng trên thanh trạng
  // thái trong lúc người dùng đang gõ là để một câu ĐÚNG-LÚC-ĐÓ nói dối ở hiện tại.
  //
  // ⚠️ Dọn ở ĐÂY, không bằng một `setTimeout`: một câu tự biến mất sau N giây là một hẹn giờ
  // phải chọn N, phải test, và nó vẫn sai với người đọc chậm. Sự kiện thì không phải chọn gì.
  // 🔵 Story 2.9, AC4 — câu báo của lượt gộp/tách đi cùng cửa và cùng lý do. Cả bảy giá trị
  // của `RegroupNotice` mô tả một thời điểm **đã trôi qua**; giữ chúng lại trong lúc người
  // dùng đang gõ là để một câu ĐÚNG-LÚC-ĐÓ nói dối ở hiện tại.
  //
  // 🔵 Story 2.10 — nay đi qua [`datThongBao`], **tham số rỗng**: người dùng gõ tiếp nghĩa là
  // KHÔNG thao tác nào sở hữu thanh trạng thái nữa. Câu điều hướng *(ô nhớ thứ ba)* thuộc cùng
  // một lớp và được dọn ở đây nhờ đúng lượt gọi này — không một dòng thứ ba phải nhớ viết.
  datThongBao({})
}

/**
 * Gửi tập chờ xuống Rust **ngay**, và chờ nó chạm WAL.
 *
 * Bốn chỗ gọi, đúng bốn đường của AC3 cộng nhịp theo lịch: hết `deadline()` · rời segment ·
 * đóng Tác phẩm · thoát ứng dụng.
 *
 * ⚠️ **Không ném.** `saveSegmentTargets` đã nuốt và trả `{ outcome, error }`; một lỗi ở đây
 * giữ nguyên tập chờ để lượt sau thử lại, và **không** cập nhật mốc *"Đã lưu"* — một mốc đi
 * lên sau một lượt ghi trượt là màn hình nói dối theo hướng an tâm, đúng thứ UX-DR30 cấm.
 *
 * 🔴 **Nhưng nó TRẢ VỀ kết quả** — code review 2026-08-13. *"Không ném"* và *"không nói gì"* là
 * hai chuyện: `beginSubmit` phải biết lượt flush có chạm WAL không **trước** khi
 * `replace_open_work` chạy, vì `resetEditorPanel()` sau đó **vứt** tập chờ vô điều kiện. Xem
 * [`FlushResult`].
 *
 * ⚠️ Vế *"báo lỗi ghi ra màn hình"* **cố ý chưa làm** ở story này: nó là một lượt thu hẹp
 * UX-DR30 *(“không hộp thoại, không dấu chấm chưa lưu”)* và hợp đồng đó có chủ là Ice. Hôm nay
 * một lượt trượt để lại **một dòng `console.error` và một con số ngừng tăng** trên `StatusBar`.
 */
export async function flushEditorNow(): Promise<FlushResult> {
  // 🔴 Một lô đang bay ⇒ **CHỜ** nó, đừng bỏ qua nó. Xem doc-comment của [`inFlight`].
  const running = inFlight
  if (running !== null) {
    const first = await running
    // Lô cũ thành công mà tập chờ còn sót ⇒ người dùng đã gõ tiếp trong lúc nó bay, và những
    // ký tự đó cần một lượt của riêng chúng.
    //
    // ⚠️ Chỉ đi tiếp khi lô cũ **thành công**. Đệ quy sau một lượt trượt là dựng lại đúng vòng
    // lặp chặt mà [`EDITOR_RETRY_FLOOR_MS`] vừa đóng — ở đó tập chờ không bao giờ vơi.
    if (first !== 'saved' || !flush.isDirty()) return first
    return flushEditorNow()
  }

  if (!flush.isDirty()) return 'clean'
  const chapter = chapterId.value
  if (chapter === null) return 'clean'

  const snapshot = flush.pending()
  const edits: SegmentTargetEdit[] = [...snapshot].map(([id, target_text]) => ({ id, target_text }))

  clearFlushTimer()
  const run = (async (): Promise<FlushResult> => {
    try {
      const { outcome, error } = await saveSegmentTargets(chapter, edits)
      // 🔴 `outcome === null` phủ CẢ HAI ca: một `IpcError` thật, và *"chạy ngoài Tauri"* (không
      // cầu IPC — `npm run dev` trong một trình duyệt thường). Không ca nào được coi là đã lưu.
      if (outcome === null) {
        // 🔴 `error` KHÔNG được nuốt — code review 2026-08-13. Bản cũ destructure mỗi
        // `{ outcome }`, nên một lượt ghi trượt lặp lại mãi mà không để lại một dòng nào.
        // Một đường hỏng im lặng trên tầng lưu dữ liệu là đúng thứ NFR18 tồn tại để chống.
        console.error(
          `[editor] flush TRƯỢT cho ${edits.length} segment của Chương ${chapter} — giữ tập chờ ` +
            `để thử lại: ${error === null ? 'không có cầu IPC (chạy ngoài Tauri?)' : error.code}`,
        )
        return 'failed'
      }
      flush.onFlushed(Date.now(), snapshot)
      lastSavedAt.value = Date.now()
      return 'saved'
    } catch (err) {
      // `saveSegmentTargets` đã nuốt mọi đường ném của nó, nên tới đây là một khuyết tật ở
      // tầng này. Vẫn không ném ra ngoài: hợp đồng của hàm là **không ném**, và một lượt ném
      // ở đường thoát ứng dụng là một cửa sổ không đóng được.
      console.error(`[editor] flush ném ngoài dự kiến — giữ tập chờ: ${String(err)}`)
      return 'failed'
    }
  })()

  inFlight = run
  let result: FlushResult
  try {
    result = await run
  } finally {
    inFlight = null
  }
  // Còn sót ⇒ người dùng đã gõ tiếp trong lúc lô bay, hoặc lượt ghi trượt. Cả hai đều cần một
  // timer mới; `flush.deadline()` vẫn neo trần cứng ở mốc đổi ĐẦU TIÊN (AC2). Lượt trượt nhận
  // thêm một **sàn** để nó không quay vòng ở `setTimeout(…, 0)`.
  armFlushTimer(result === 'failed' ? EDITOR_RETRY_FLOOR_MS : 0)
  return result
}

/**
 * Kết quả của [`flushEditorBeforeDiscreteWrite`]. Ba giá trị, ba việc khác nhau cho nơi gọi.
 *
 * `'clean'` ⇒ tập chờ **thật sự** rỗng, đi tiếp được. `'failed'` ⇒ một lượt ghi trượt.
 * `'still-dirty'` ⇒ ghi được nhưng tập chờ **vẫn** không vơi ⇒ **từ chối**, đừng ghi đè.
 */
export type PreWriteFlushResult = 'clean' | 'failed' | 'still-dirty'

/**
 * 🔴 **Đưa tập chờ xuống đĩa TRƯỚC một lượt ghi RỜI RẠC** — cửa dùng chung của mọi lệnh ghi
 * không đi qua bộ đệm gõ.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 2026-08-16 (code review Story 2.6) — RÚT RA THÀNH MỘT HÀM, VÀ LƯỢT CHÉP LÀ THỦ PHẠM
 * ─────────────────────────────────────────────────────────────────────────────
 * Khuôn này ra đời ở Quyết định #8 của Story 2.5b *(Ice ký 2026-08-14)* và sống **trong thân**
 * `confirmCurrentSegmentUnguarded`. Story 2.6 dựng đường khôi phục, chép **doc-comment** của
 * khuôn sang `segmentHistoryState.ts` — *"áp nguyên ở đây"* — nhưng chép thiếu **mã**: chỉ một
 * lượt flush, không `isDirty()`, không đường từ chối. Cổng không đỏ, và bảo đảm mà chữ ký
 * #2(a) mua được mất trong im lặng.
 * ⇒ Một khuôn được chép là một khuôn sẽ chép thiếu. Nay nó là **một hàm**, và nơi gọi thứ ba
 * không còn cách nào thi hành nó đúng một nửa.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO PHẢI HAI LƯỢT — MÃ KẾT QUẢ CỦA FLUSH KHÔNG ĐỒNG NGHĨA VỚI "TẬP CHỜ SẠCH"
 * ─────────────────────────────────────────────────────────────────────────────
 * Nhánh *originator* của [`flushEditorNow`] chụp `snapshot` **trước** lượt IPC và khi lô về thì
 * chỉ `armFlushTimer(0)` rồi trả `'saved'` — nó **không** đệ quy như nhánh *joiner*. Đó là hành
 * vi ĐÚNG cho auto-save *(đệ quy ở đó là quay vòng trong lúc người dùng gõ liên tục, đúng thứ
 * trần cứng AD-35 tồn tại để tránh)* — nhưng nó sai cho một lượt ghi rời rạc: một ký tự gõ
 * trong lúc lô đang bay nằm **ngoài** ảnh chụp, vẫn `isDirty()`, mà flush đã trả `'saved'`.
 *
 * ⇒ Thử lại **đúng một** lượt *(đóng ca thường: một ký tự lẻ, không phải đặt một trần lặp mới)*;
 * còn dơ nữa thì trả `'still-dirty'` để nơi gọi **từ chối**, chứ không ghi đè lên một bản chưa
 * kịp xuống đĩa.
 *
 * ⚠️ **Không** ném, và **không** tự ghi chẩn đoán: hai nơi gọi hỏng theo hai cách khác nhau và
 * nói hai câu khác nhau. Chẩn đoán thuộc về nơi gọi.
 */
export async function flushEditorBeforeDiscreteWrite(): Promise<PreWriteFlushResult> {
  if ((await flushEditorNow()) === 'failed') return 'failed'
  if (!flush.isDirty()) return 'clean'
  if ((await flushEditorNow()) === 'failed') return 'failed'
  return flush.isDirty() ? 'still-dirty' : 'clean'
}

/**
 * 🔴 **Vứt toàn bộ state của Panel Editor** — gọi khi Tác phẩm đang mở BỊ THAY.
 *
 * Cùng đường hỏng, cùng chỗ gọi và cùng lý do với `resetSourcePanel()`/`resetLookupPanel()`:
 * `requested` là một cache **module-level không có khoá vô hiệu hoá**, và `replace_open_work`
 * phía Rust trỏ `OpenWorkState` sang Tác phẩm mới mà panel không hay biết. Bỏ nó đi thì tạo
 * Tác phẩm B xong, Editor vẫn hiện segment của Tác phẩm **A** — không lỗi, không cảnh báo.
 *
 * ⚠️ Chỗ gọi duy nhất là `modes/libraryImport.ts::finishSubmit`. Đừng rải lời gọi này ra.
 *
 * 🔵 **SỬA 2026-08-18 (Story 2.11) — câu ngay trên HẾT ĐÚNG: nay có HAI chỗ gọi.**
 * Chỗ thứ hai là [`switchChapter`] *(đổi **Chương** trong cùng Tác phẩm)*, và nó đi qua đây
 * theo **chữ ký #8 đường (a) của Ice**, không theo một lượt tiện tay. Ba đường đã cân:
 * - **(a)** tái dùng hàm này ⇒ một nguồn sự thật cho *"dọn state"*, và hai ô sót được vá
 *   **cùng lượt** *(xem khối `sourceCut`/`omitError` bên dưới)*;
 * - **(b)** một `resetChapterState()` riêng ⇒ hai hàm cùng canh một tập ô là **hai nguồn sự
 *   thật**, và luật *"ô nhớ mới phải qua đây"* vốn đã không có cổng nào canh;
 * - **(c)** dọn tối thiểu tại chỗ ⇒ đúng cái đã sinh ra hai ô sót.
 *
 * 🔴 **Câu "đừng rải lời gọi này ra" KHÔNG bị huỷ, nó được làm chặt hơn:** hàm này gọi được
 * từ đúng **hai** đường, và **cả hai** phải flush **XONG** trước — xem khối `flush.reset()`
 * bên dưới, nơi thứ tự ấy là một mệnh đề chứ không một lời khuyên.
 */
export function resetEditorPanel(): void {
  // Vô hiệu hoá mọi lượt nạp ĐANG BAY — xem [`sequence`].
  sequence += 1

  segments.value = []
  chapterId.value = null
  loadError.value = null
  pending.value = false
  caretSegmentId.value = null
  requested = false

  // 🔴 Tập chờ và mốc *"Đã lưu"* thuộc Tác phẩm VỪA BỊ THAY, nên chúng phải đi cùng nó.
  //
  // ⚠️ Chỗ gọi duy nhất (`modes/libraryImport.ts::finishSubmit`) **flush TRƯỚC** khi gọi hàm
  // này — xem AC3 vế *"đóng Tác phẩm"*. Thứ tự đó là mệnh đề: `flush.reset()` ở đây **vứt**
  // tập chờ không ghi, nên gọi nó trước lượt flush là ăn mất bản dịch chưa lưu của Tác phẩm
  // cũ, im lặng. Nếu một chỗ gọi thứ hai xuất hiện, nó phải giữ đúng thứ tự đó.
  clearFlushTimer()
  flush.reset()
  editedText.value = new Map()
  lastSavedAt.value = null

  // 🔵 THÊM Story 5.7 — nhịp ghi vị trí cũng thuộc Tác phẩm/Chương VỪA BỊ THAY, cùng lý lẽ
  // `flush.reset()` ngay trên. Chỗ gọi phải flush TRƯỚC khi gọi hàm này nếu muốn giữ vị trí
  // đang chờ (`flushChapterPositionNow()`, Task 15 ②) — `reset()` ở đây VỨT không ghi.
  clearPositionFlushTimer()
  positionFlush.reset()

  // 🔴 HAI Ô NHỚ NỮA, và bỏ sót chúng là một LỜI NÓI DỐI TRỰC QUAN, không một lượt rò state.
  //
  // 🔵 Thêm 2026-08-15 (code review). Đo được: `segment.id` là `INTEGER PRIMARY KEY
  // AUTOINCREMENT` trong `project.db` của TỪNG Tác phẩm (`schema.rs:337`), nên mỗi Tác phẩm
  // đếm lại từ 1 — đụng id giữa hai Tác phẩm là chuyện gần như CHẮC CHẮN cho các câu đầu
  // Chương, không phải một trùng hợp hiếm.
  //
  // Đường hỏng, từng bước:
  //   ① Ở Tác phẩm A, `⌘Enter` bị Rust từ chối ⇒ `confirmError` mang một `IpcError` có
  //      `params.segment_id = 2`.
  //   ② Người dùng tạo/mở Tác phẩm B ⇒ `finishSubmit` gọi hàm này rồi nạp lại.
  //   ③ `GridPanel.vue::errorSegmentId` đọc `params.segment_id` ⇒ vẫn là 2, và câu số 2 của
  //      Tác phẩm B khớp ngay khi lưới vừa nạp xong.
  //   ⇒ Một hàng của Tác phẩm B hiện `panel.grid.state_refused` — *"chưa ký được"* — TRƯỚC khi
  //      người dùng kịp bấm bất cứ thứ gì ở Tác phẩm đó.
  //
  // ⚠️ `caretPlacement` cùng một lớp: nó chở một `segment.id` để lưới đặt con trỏ vào. Sống
  // sót qua lượt thay Tác phẩm, nó dời con trỏ tới một câu người dùng chưa từng chọn.
  //
  // 🔴 Luật rút ra, áp cho mọi ô nhớ THÊM VÀO TỆP NÀY sau này: hỏi *"ô này thuộc Tác phẩm hay
  // thuộc ứng dụng?"*. Thuộc Tác phẩm thì phải có mặt ở đây. Không cổng nào canh câu hỏi đó —
  // `confirmError` và `caretPlacement` đều được thêm ở Story 2.5 và 2.5b, cả hai đi qua sạch
  // mười một cổng, và cả hai bị bỏ quên ở đây.
  confirmError.value = null
  caretPlacement.value = null

  // 🔵 **Thêm 2026-08-17 (code review ba tầng — HAI tầng độc lập bắt được).** Hai ô nhớ của
  // Story 2.9 rơi vào **đúng cái bẫy mà khối chú thích ngay trên đây mô tả**, ở story liền sau.
  //
  // `regroupError` chở một `IpcError` có `params.segment_id`; `regroupNotice` chở câu chữ của
  // lượt gộp/tách vừa rồi. Cả hai thuộc **Tác phẩm đang mở lúc gộp**.
  //   ① Ở Tác phẩm A, `Backspace` ở câu đầu Chương bị Rust từ chối ⇒ `regroupNotice = 'refused'`
  //      và `regroupError` mang *"Câu số N là câu đầu Chương…"*.
  //   ② Người dùng mở/tạo Tác phẩm B ⇒ `finishSubmit` gọi hàm này rồi nạp lại.
  //   ③ Ở Tác phẩm B, `confirmNotice` là `null` nên nhánh `v-else-if` của `StatusBar.vue` hiện
  //      câu từ chối của Tác phẩm **A** — về một `segment_id` không thuộc Tác phẩm đó — **trước**
  //      khi người dùng bấm bất cứ thứ gì.
  //
  // 🔴 Bằng chứng rằng luật ở khối trên **không có cổng nào canh**: nó đã bị bỏ sót **hai story
  // liên tiếp** *(`sourceCut` ở 2.8 — còn hở, đã ghi nợ có chủ; hai ô nhớ này ở 2.9)*, và cả hai
  // lượt đều đi qua trọn mười một cổng. Một luật chỉ sống trong một khối chú thích là một luật
  // sẽ bị quên lần thứ ba. Món dựng cổng đã vào `deferred-work.md`.
  regroupError.value = null

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔵 STORY 2.11 — HAI Ô SÓT, và một trong hai đã nằm trong sổ nợ TỪ STORY 2.8
  // ═══════════════════════════════════════════════════════════════════════════════
  // Khối ngay trên viết *"một luật chỉ sống trong một khối chú thích là một luật sẽ bị quên
  // lần thứ ba"*. Đúng — và nó cũng đúng **lùi về quá khứ**: hàm này dọn 13 ô và bỏ sót
  // **hai**, cả hai đều thuộc Tác phẩm, cả hai đi qua trọn mười một cổng.
  //
  // 🔴 `sourceCut` chở `{ segmentId, offsets[] }` của Chương/Tác phẩm cũ. Đường hỏng:
  //   ① Người dùng `Mod`+click đánh dấu một chỗ cắt ở câu số 7 của Chương A.
  //   ② Đổi Tác phẩm — hoặc, **từ story này**, đổi **Chương**.
  //   ③ `⌘/` ⇒ lệnh tách chạy trên `segmentId` số 7 **của Chương mới**, một hàng người dùng
  //      chưa từng nhìn thấy, trên dữ liệu mà **AD-5 không cho hoàn tác**.
  //   Món nợ này đã ghi bằng chữ ngay trong hàm này từ Story 2.8 và ở lại hở hai story.
  //
  // 🔴 `omitError` chở một `IpcError` mang `params.segment_id` — **chưa ai nêu nó**, kể cả
  // lượt rà 2.9 vừa vá hai ô cùng hạng (`confirmError` · `regroupError`). Nó lọt vì nó là ô
  // duy nhất trong hạng đó **chưa component nào đọc** (`editorOmitError`, `:956-965`), tức
  // biểu hiện của nó hôm nay là **0 pixel**. Đó là lý do để vá nó **bây giờ**, không phải lý
  // do để hoãn: ngày một component đọc nó, khuyết tật ra đời đã sẵn sàng.
  sourceCut.value = null
  omitError.value = null

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔵 STORY 2.10 — LẦN THỨ BA CỦA CÙNG MỘT LUẬT, và lượt này nó KHÔNG phải một lượt nhớ
  // ═══════════════════════════════════════════════════════════════════════════════
  // Khối ngay trên viết: *"một luật chỉ sống trong một khối chú thích là một luật sẽ bị quên
  // lần thứ ba"*. Story 2.10 **là** lần thứ ba — nó thêm `navNotice`.
  //
  // 🔴 Lượt này ô nhớ mới đi vào đây **theo cấu tạo, không theo trí nhớ**: cả ba ô câu chữ chỉ
  // có một cửa ghi ([`datThongBao`]) và cửa ấy gán **cả ba** ở mọi lời gọi. ⇒ Một lời gọi
  // tham số rỗng ở đây dọn ô thứ ba, và nó sẽ dọn ô thứ **tư** vào ngày ai đó thêm — không cần
  // sửa dòng này.
  //
  // ⚠️ Đây **không** phải cổng mà `deferred-work.md` còn nợ: nó chỉ đóng đúng ba (rồi N) ô câu
  // chữ đi qua [`datThongBao`]. Một ô nhớ **không phải câu chữ** *(khuôn `sourceCut`,
  // `confirmError`, `caretPlacement`)* vẫn phải nhớ bằng tay, và vẫn không cổng nào canh.
  // 🔵 **HẾT ĐÚNG 2026-08-18 (Story 2.12, AC5)** — nay CÓ một cổng canh: `check:panel-refs`
  // đỏ khi một ô nhớ cấp module không đi qua một hàm reset và cũng không có miễn trừ có tên.
  datThongBao({})
  // ⚠️ `splitChapterError` KHÔNG đi qua `datThongBao` (nó chở một `IpcError`, không một mã câu
  // chữ) — cùng khuôn `regroupError`, và cùng lý do nó phải có mặt Ở ĐÂY bằng tay:
  // `check:panel-refs` đòi mọi ô nhớ cấp module có một đường `reset*()`.
  splitChapterError.value = null

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔵 STORY 2.12 — NĂM CỜ/MUTEX TIẾN TRÌNH, và vì sao chúng vào đây chứ không được miễn
  // ═══════════════════════════════════════════════════════════════════════════════
  // Ice ký 2026-08-18 (quyết định #2a): năm ô dưới đây **bắt buộc** đi qua reset, đường
  // *"miễn trừ CÓ TÊN"* bị loại. Lý do là một phép đếm, không một sở thích: đây đúng lớp mà
  // `sourceCut` (Story 2.8) và `omitError` (Story 2.9) đã lọt qua **hai story liên tiếp**, và
  // không cổng nào đỏ cả hai lần.
  //
  // ⚠️ **Ba dòng promise KHÔNG phải mã chết chỉ vì mỗi lượt bay tự dọn mình ở `finally`.**
  // Chúng tự dọn khi lượt bay **kết thúc**; `resetEditorPanel()` chạy đúng lúc Tác phẩm đổi
  // dưới chân một lượt bay **CHƯA** kết thúc. Ô còn giữ promise của Tác phẩm CŨ, nên lời gọi
  // kế tiếp *"chờ lượt đang bay rồi hẵng chạy"* sẽ chờ — rồi đọc kết quả của một Tác phẩm
  // khác. Một câu trả lời sai trông hoàn toàn bình thường.
  //
  // 🔴 `sequence += 1` ở đầu hàm này làm những lượt bay ấy **vô hại**, không làm chúng **biến
  // mất**. Hai vế khác nhau, và chỉ vế thứ hai mới là reset.
  inFlight = null
  confirmInFlight = null
  regroupInFlight = null
  kyTrungCauCuoi = false
  dangChuyenChuong = false

  // 🔵 Story 3.4b — cùng luật đã rút ra ba lần ở trên: dấu thuật ngữ Glossary thuộc Chương
  // ĐANG MỞ (offset của nó chỉ có nghĩa cùng với `segments.value` lúc nạp), nên nó phải dọn ở
  // ĐÚNG hai đường mà hàm này đã phủ — đổi Tác phẩm (`libraryImport.ts`) VÀ đổi Chương
  // (`switchChapter`). Không dọn ở đây thì một khoảnh khắc giữa reset và lượt nạp lại
  // (`ensureGlossaryMarksLoaded` ngay sau `switchChapter`) vẽ dấu của Chương CŨ lên vị trí của
  // Chương MỚI — cùng lớp lỗi mà `sourceCut`/`confirmError` ở trên tồn tại để chặn.
  resetGlossaryMarks()
  // 🔵 Story 3.6 — sổ "Để sau" của dải chốt có phạm vi ĐÚNG MỘT Chương; đổi Chương/Tác phẩm
  // thì dải thu và sổ đó xoá, đúng §I/O Matrix "Đổi Chương giữa chừng".
  resetGlossaryConfirmStrip()
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 AD-35 VẾ (e) — FLUSH LÚC THOÁT ỨNG DỤNG. Story 2.3 · AC17 · Quyết định #4 đường (b)
// ═════════════════════════════════════════════════════════════════════════════════

/** Khớp `src-tauri/src/lib.rs::EXIT_FLUSH_EVENT`. */
const EXIT_FLUSH_EVENT = 'aura://flush-before-exit'
/** Khớp tên hàm `#[tauri::command] confirm_exit_flush` ở `src-tauri/src/lib.rs`. */
const CMD_CONFIRM_EXIT_FLUSH = 'confirm_exit_flush'

/**
 * Nối lượt *"cửa sổ sắp đóng"* của Rust vào một lượt flush, rồi báo lại **xong**.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * Vì sao đường này chứ không một `beforeunload` — Quyết định #4, Ice ký 2026-08-12
 * ─────────────────────────────────────────────────────────────────────────────
 * `invoke()` là **bất đồng bộ** và `RunEvent::Exit` phía Rust **không chờ** ai: `lib.rs` gọi
 * `close_open_work` ngay, và hàm đó `take()` `OpenWork` rồi `store.close()`. Một lượt flush
 * phát ở `beforeunload` có thể vẫn đang bay khi kho đóng ⇒ đúng lớp *"trông như đã lưu"*.
 * Đường này thì `Rust chặn lượt đóng → webview flush → webview báo xong → Rust đóng`, với một
 * **trần thời gian** (`EXIT_FLUSH_BUDGET` = 1 200 ms) để một webview treo không làm ứng dụng
 * không đóng được.
 *
 * 🔴 `confirm_exit_flush` được gọi trong `finally` — **kể cả khi lượt flush trượt**. Không cho
 * đóng khi ghi trượt là biến một lỗi ghi thành một cửa sổ không đóng được, và người dùng sẽ
 * kết thúc bằng một lượt kill tiến trình *(đường mà `panic = "abort"` bỏ qua trọn)*. Trần bên
 * Rust vốn đã phủ ca này; lời gọi ở đây chỉ để lượt đóng **bình thường** không phải chờ hết trần.
 *
 * ⚠️ Gọi **một lần** từ `main.ts`, cùng khuôn `wireDragDrop`. Trả về hàm tháo listener.
 */
export async function wireExitFlush(): Promise<() => void> {
  // ⚠️ `import` động, cùng khuôn `App.vue`: chạy ngoài Tauri (`npm run dev` trong một trình
  // duyệt thường) thì không có cầu IPC, và một `import` tĩnh của `@tauri-apps/api/event` ở
  // tầng state sẽ ném ngay lúc nạp module.
  try {
    const [{ listen }, { invoke }] = await Promise.all([
      import('@tauri-apps/api/event'),
      import('@tauri-apps/api/core'),
    ])
    return await listen(EXIT_FLUSH_EVENT, () => {
      void (async () => {
        try {
          await flushEditorNow()
          // 🔵 THÊM Story 5.7 (AC4/AC6) — biên thứ ba của flushChapterPositionNow(): thoát
          // ứng dụng bình thường (AC6 đòi vị trí "khôi phục đúng" sau một lượt đóng-mở lại).
          await flushChapterPositionNow()
        } finally {
          try {
            await invoke(CMD_CONFIRM_EXIT_FLUSH)
          } catch (err) {
            // Không báo được thì trần bên Rust vẫn đóng cửa sổ — ghi chẩn đoán, đừng im.
            console.error(`[editor] không báo được \`${CMD_CONFIRM_EXIT_FLUSH}\`: ${String(err)}`)
          }
        }
      })()
    })
  } catch (err) {
    console.info(`[editor] không nối được đường flush lúc thoát — chạy ngoài Tauri? ${String(err)}`)
    return () => {}
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 2.5 — XÁC NHẬN SEGMENT (FR24 · AD-31 · AD-35 vế (c))
//
// 🔴 KHÔNG một quy tắc nào của bảng AD-31 sống ở đây (AC12). Máy trạng thái ở Rust
// (`commands/segment.rs`); chỗ này chỉ **xếp thứ tự ba lượt gọi** và cập nhật ảnh chụp
// hiển thị. Xếp thứ tự là việc của tầng giao diện, phân xử trạng thái thì không.
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Kết quả một lượt xác nhận **nhìn từ giao diện**. Năm giá trị, và cả năm **phân biệt được**
 * — *"rỗng IM LẶNG bị cấm; rỗng CÓ LÝ DO thì không"*.
 *
 * ⚠️ `'refused'` mang lỗi ra bằng [`editorConfirmError`], **không** bằng chuỗi ở đây, và
 * **không đoán lại lý do từ chuỗi lỗi** (AC14).
 *
 * 🔵 **Code review 2026-08-14 — mệnh đề cũ ở đây đã hết đúng và được sửa tại chỗ.** Bản trước
 * viết *"chỗ gọi hiển thị bằng `tError()`"*. **Chỗ gọi đó chưa tồn tại:** `main.ts` gọi
 * `void confirmCurrentSegment()` và **vứt** giá trị trả về, còn [`editorConfirmError`] được
 * export mà **không component nào đọc**. ⇒ Hôm nay cả ba khoá `err.segment.*` vừa dựng đều
 * dừng ở biên giới TypeScript, không tới màn hình. AC14 **vẫn đạt** — nó nói về hợp đồng IPC,
 * và Rust trả đúng `IpcError` phân biệt được — nhưng vế **hiển thị** là một món nợ có chủ,
 * ghi ở `deferred-work.md` §*"Deferred from: code review of story 2-5"*. Đừng đọc dòng này
 * thành *"đã có đường ra màn hình"*.
 */
export type ConfirmResult =
  | 'confirmed'
  | 'no-caret'
  | 'flush-failed'
  | 'refused'
  /**
   * 🔵 Quyết định #8, Ice ký 2026-08-14 — tập chờ **vẫn còn dơ** sau **hai** lượt flush ⇒
   * KHÔNG ký. Thà một phím tắt không làm gì còn hơn ký một văn bản người dùng chưa từng thấy.
   */
  | 'still-dirty'

const confirmError = shallowRef<IpcError | null>(null)
/** Lỗi gần nhất Rust trả lời cho một lượt xác nhận. `null` ⇒ chưa lượt nào bị từ chối. */
export const editorConfirmError: DeepReadonly<Ref<IpcError | null>> = readonly(confirmError)

/**
 * 🔴 **BA kết quả xác nhận KHÔNG đi qua Rust — và trước 2026-08-15 chúng không đi tới đâu cả.**
 *
 * 🔵 Thêm ở lượt code review 2026-08-15, thi hành nốt vế còn thiếu của **Quyết định #8** *(Ice
 * ký 2026-08-14)*. Chữ ký khai hợp đồng UX-DR30 tối thiểu là *"cột nhãn trạng thái của **chính
 * hàng** CỘNG **một dòng ở thanh trạng thái**"*. Vế đầu đã dựng *(`GridPanel.vue` đọc
 * [`editorConfirmError`] cho `'refused'`)*; vế sau **chưa từng tồn tại**.
 *
 * ⇒ Đo được trước lượt vá: `'no-caret'` · `'flush-failed'` · `'still-dirty'` chỉ rơi vào một
 * `console.warn` trong `main.ts`. Người dùng bấm `⌘Enter` và **không một pixel nào đổi** — đúng
 * hình dạng *"rỗng im lặng"* ở §Critical Don't-Miss Rules, lần này áp lên một thao tác người
 * dùng **chủ động**, tức ca tệ nhất của lớp lỗi đó.
 *
 * 🔴 **`'refused'` CỐ Ý không vào đây.** Nó đã có đường ra riêng và **giàu hơn**: một `IpcError`
 * mang `params.segment_id`, nên lỗi dán được lên **đúng hàng**. Đẩy nó vào đây nữa là dựng
 * nguồn sự thật thứ hai cho cùng một sự kiện.
 *
 * ⚠️ **Phạm vi ĐÓNG BĂNG ở một dòng chữ.** Story tự cảnh báo *"đừng nhân lượt ký này thành một
 * hệ thống thông báo"* — không hàng đợi, không mức độ, không tự tắt theo giờ, không lớp nổi
 * (UX-DR16). Ô nhớ giữ **một** giá trị gần nhất và bị dọn bởi ba sự kiện có tên ở dưới.
 */
const confirmNotice = shallowRef<Exclude<ConfirmResult, 'confirmed' | 'refused'> | null>(null)
/** Xem [`confirmNotice`]. `StatusBar.vue` đọc. `null` ⇒ không có gì để nói. */
export const editorConfirmNotice: DeepReadonly<Ref<Exclude<
  ConfirmResult,
  'confirmed' | 'refused'
> | null>> = readonly(confirmNotice)

/**
 * `segment.id` mà giao diện được yêu cầu **đặt con trỏ DOM vào đầu câu đó**.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO MỘT TÍN HIỆU RIÊNG, KHÔNG TÁI DÙNG WATCHER SẴN CÓ CỦA `editorCaretSegmentId`
 * ─────────────────────────────────────────────────────────────────────────────
 * Watcher trong `GridPanel.vue` **khôi phục** caret về đúng chỗ nó vừa ở,
 * và nó cố ý **không làm gì** khi vị trí đã lưu không nằm trong câu mới. Nới nhánh đó thành
 * *"không nằm trong thì đặt vào đầu câu"* sẽ chạy cả trên **đường chuột** — và ở đó nó kéo
 * caret về đầu câu sau một cú bấm vào giữa câu.
 *
 * ⇒ Một tín hiệu **tường minh** nói đúng một điều: *"lượt dời này là do CHƯƠNG TRÌNH, hãy đặt
 * caret vào đầu câu."* Đường chuột không đụng tới nó.
 *
 * ⚠️ Đường bấm chuột của Story 2.3 mất **ba lượt chẩn đoán** mới đúng *(hai lượt đầu bị bác
 * bằng phép đo)*. Không nới nó cho một tính năng khác.
 */
const caretPlacement = shallowRef<number | null>(null)
/** Xem [`caretPlacement`]. `GridPanel.vue` đọc, đặt caret, rồi gọi [`clearEditorCaretPlacement`]. */
export const editorCaretPlacement: DeepReadonly<Ref<number | null>> = readonly(caretPlacement)

/** Giao diện đã đặt xong caret ⇒ dọn tín hiệu, để lượt sau còn phân biệt được. */
export function clearEditorCaretPlacement(): void {
  caretPlacement.value = null
}

/**
 * **Xác nhận câu đang có con trỏ** — FR24, và đây là chỗ giao ba mệnh đề cùng lúc.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ① AD-35 vế (c) — FLUSH TRƯỚC, VÀ FLUSH PHẢI **XONG** (đã vào WAL)
 * ─────────────────────────────────────────────────────────────────────────────
 * `confirm_segment` phía Rust chỉ đọc thứ **đã ở trên đĩa**. Gọi nó trước khi lượt flush
 * chạm WAL sẽ ký một văn bản **cũ hơn** thứ người dùng đang nhìn, và `SegmentVersion` sinh ra
 * mang đúng văn bản cũ đó ⇒ FR101 khôi phục về một thứ chưa bao giờ ở trên màn hình.
 *
 * 🔴 `'failed'` ⇒ **DỪNG, không xác nhận**. Ký một câu mà lượt lưu vừa trượt là ghi một chữ ký
 * cho một văn bản không tồn tại trên đĩa.
 *
 * ⚠️ **GIỚI HẠN THẬT:** mệnh đề *"flush trước"* **không** cưỡng chế được ở tầng Rust — lệnh
 * IPC không biết gì về văn bản đang gõ. Lưới duy nhất là hàm này cộng
 * `tests/frontend/editorConfirmSegment.test.ts`. Một chỗ gọi tương lai `invoke` thẳng
 * `confirm_segment` sẽ đi vòng qua nó mà **không cổng nào đỏ**.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ② Quyết định #1 (Ice ký 2026-08-14, đường (a)) — DỜI CON TRỎ SANG CÂU KẾ TIẾP
 * ─────────────────────────────────────────────────────────────────────────────
 * `resolveSegmentRule` cho `primary` **thắng** `confirmed` *(một mệnh đề về **hiện tại** thắng
 * một mệnh đề về **quá khứ**, `editorSegments.ts:95-97`)*. Người dùng bấm xác nhận trên chính
 * câu con trỏ đang đứng ⇒ nếu con trỏ ở lại, vạch **vẫn** `primary` và AC1 mô tả một đổi màu
 * không xảy ra. Dời con trỏ đi làm câu vừa ký hiện `confirmed` **ngay**, và
 * `resolveSegmentRule` đổi **0 dòng**.
 *
 * ⚠️ Đây là **một** đường dời tối thiểu. **Story 2.10** (*điều hướng segment*) **dùng lại** nó,
 * không dựng đường thứ hai.
 * ⚠️ Câu cuối Chương không có câu kế ⇒ con trỏ **ở lại**, và vạch ở lại `primary` cho tới khi
 * người dùng đi chỗ khác. Ca này có thật và không có lời giải nào tốt hơn trong phạm vi story.
 * 🔵 **Code review 2026-08-14:** ca đó là một vế **AC1 không đạt** *(vạch không chuyển
 * `confirmed`)*, xảy ra **đúng một lần mỗi Chương**. Nó nay có chủ — `deferred-work.md`
 * §*"Deferred from: code review of 2-5…"*, **chủ: Story 2.10**. Ghi ra ở đây là chưa đủ: một
 * mệnh đề chỉ sống trong chú thích là một mệnh đề không ai theo dõi.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ③ ẢNH CHỤP HIỂN THỊ — và vì sao nó KHÔNG huỷ mốc so sánh của FR117
 * ─────────────────────────────────────────────────────────────────────────────
 * Vạch lề đọc `segment.status` từ [`segments`], nên trạng thái mới phải vào ảnh chụp đó, nếu
 * không màn hình nói dối cho tới lượt nạp lại. Lượt cập nhật dưới đây thay **đúng một** trường
 * `status` và dựng một mảng mới *(`shallowRef` không theo dõi sửa tại chỗ)*.
 *
 * 🔴 `target_text` **KHÔNG bị đụng**, và đó là điều kiện của AC11(a): [`segments`] giữ bản
 * **lúc nạp**, tức mốc mà FR117 (Story 2.7) so *"văn bản đích hiện tại với bản lúc nạp"* —
 * **không dùng cờ dirty** (hợp đồng phụ AD-31).
 */
export async function confirmCurrentSegment(): Promise<ConfirmResult> {
  // ─────────────────────────────────────────────────────────────────────────────
  // 🔴 KHOÁ CHỐNG-GỌI-LẠI — code review 2026-08-14
  // ─────────────────────────────────────────────────────────────────────────────
  // Cùng khuôn và cùng lý do với [`inFlight`] của `flushEditorNow`, nhưng đường hỏng khác
  // hẳn nên nó cần một khoá **riêng**: `inFlight` chỉ bọc lượt FLUSH bên trong, không bọc
  // cả lượt xác nhận.
  //
  // Đường hỏng đo được: bấm `Mod+Enter` hai lần liên tiếp nhanh *(thói quen "bấm lại cho
  // chắc" — không có chỉ báo đang xử lý)*. Cả hai lượt đọc `caretSegmentId` **trước** khi
  // lượt đầu xong nên cùng thấy id `X`. AC13 lo phần Rust *(lượt hai không tạo
  // `SegmentVersion`)*, nhưng phần GIAO DIỆN thì không: cả hai lượt đều chạy tới nhánh dời
  // con trỏ và cùng gán `caretPlacement.value = Y`.
  //
  // 🔴 Và lượt gán thứ hai **có** bắn watcher, dù giá trị y hệt: watcher ở
  // `GridPanel.vue` gọi `clearEditorCaretPlacement()` ngay dòng đầu — nó là watcher **một
  // phát**, đưa ref về `null` sau khi dùng. Nên `null → Y` là một lượt đổi thật, và
  // `setCaret(first ?? target, 0)` chạy lần hai, **kéo caret về offset 0** của câu kế tiếp —
  // kể cả khi người dùng đã kịp gõ vài ký tự vào đó. Đúng lớp khuyết tật *"caret rơi về
  // offset 0, ký tự kế tiếp chèn vào đầu"* mà `GridPanel.vue` ghi là nặng nhất từng bắt
  // được ở story trước.
  //
  // ⇒ Lượt thứ hai **nhập vào** lượt đang bay và trả cùng một kết quả: một thao tác người
  //   dùng, một lượt dời con trỏ.
  const running = confirmInFlight
  if (running !== null) return running

  const run = confirmCurrentSegmentUnguarded()
  confirmInFlight = run
  try {
    const result = await run
    // 🔵 2026-08-15 — vế "một dòng ở thanh trạng thái" của Quyết định #8. Ghi ở **cửa có khoá**
    // này chứ không rải vào `confirmCurrentSegmentUnguarded`: hàm kia có năm đường trả về, và
    // một đường thêm sau này sẽ quên cập nhật ô nhớ. Ở đây thì không quên được — mọi kết quả
    // đều đi qua đúng dòng này.
    //
    // ⚠️ Nhánh *joiner* ở trên (`return running`) CỐ Ý không ghi: lượt gốc sẽ ghi khi nó xong,
    // và hai lượt cùng ghi một giá trị chỉ là một lần thừa, không một mâu thuẫn.
    // 🔵 2026-08-17 (code review) — vế ĐỐI XỨNG của bất biến ở [`ghiRegroupNotice`]: thao tác vừa
    // xảy ra sở hữu thanh trạng thái. Không có vế dọn, một câu *"Đã gộp hai câu…"* của lượt
    // trước sống sót qua một lượt `⌘Enter` **thành công** và che mốc *"Đã lưu N giây trước"* —
    // cùng một khuyết tật, chỉ đổi chiều.
    //
    // 🔵 Story 2.10 — cả hai vế nay là **một** lời gọi [`datThongBao`], và vế thứ ba *(dọn câu
    // điều hướng)* có mặt **theo cấu tạo** thay vì phải nhớ.
    //
    // 🔴 **Và đây là chỗ Quyết định #6(c) được thi hành** — nhánh duy nhất ghi một câu ở lượt
    // xác nhận **THÀNH CÔNG**. Xem [`kyTrungCauCuoi`] về vì sao tín hiệu đi bằng một cờ chứ
    // không bằng một phép suy ra từ `caretSegmentId`.
    if (result === 'confirmed') datThongBao({ nav: kyTrungCauCuoi ? 'confirmed-last' : null })
    else datThongBao({ confirm: result === 'refused' ? null : result })
    return result
  } finally {
    confirmInFlight = null
  }
}

/**
 * 🔴 **Lượt xác nhận vừa rồi có phải ở câu CUỐI Chương không** — Story 2.10, Quyết định #6(c).
 *
 * ⚠️ **Vì sao một cờ chứ không một phép suy ra ở cửa có khoá.** Ứng viên hiển nhiên là so
 * `caretSegmentId` **trước** và **sau** lượt `await`: không đổi ⇒ không dời được ⇒ câu cuối. Nó
 * sai ở **hai** ca, và cả hai đều im lặng:
 *   ① Người dùng bấm sang một câu khác **trong lúc** lượt IPC đang bay ⇒ caret đổi vì một lý do
 *      khác ⇒ suy ra *"đã dời được"* ⇒ câu báo **không hiện** ở đúng ca nó phải hiện.
 *   ② `index < 0` *(câu vừa ký không có trong ảnh chụp)* cũng để caret nguyên và cũng trả
 *      `'confirmed'` ⇒ suy ra *"câu cuối Chương"* ⇒ câu báo hiện **sai**.
 * ⇒ Cờ được ghi ở **đúng nhánh biết sự thật** (`following === undefined`), nên nó không đoán.
 *
 * 🔴 Đặt lại `false` ở **đầu** mỗi lượt chạy, không ở cuối: [`confirmInFlight`] bảo đảm mỗi lúc
 * chỉ có **một** lượt bay *(nhánh joiner trả lại chính promise đang chạy)*, nên một lượt reset
 * ở đầu là đủ và không có ca hai lượt ghi đè nhau. Đặt lại ở cuối thì một đường `return` sớm
 * nào đó sẽ bỏ qua nó và giá trị của lượt **trước** rò sang lượt sau.
 */
let kyTrungCauCuoi = false

/** Lượt xác nhận đang bay, hoặc `null`. Xem khối lý do trong [`confirmCurrentSegment`]. */
let confirmInFlight: Promise<ConfirmResult> | null = null

async function confirmCurrentSegmentUnguarded(): Promise<ConfirmResult> {
  // 🔵 Story 2.10 — xem khối lý do của [`kyTrungCauCuoi`]. Đặt lại ở ĐÂY, trước mọi `return`.
  kyTrungCauCuoi = false
  const id = caretSegmentId.value
  if (id === null) return 'no-caret'

  // ─────────────────────────────────────────────────────────────────────────────
  // 🔴 ① AD-35 vế (c) — MÃ KẾT QUẢ CỦA FLUSH KHÔNG ĐỒNG NGHĨA VỚI "TẬP CHỜ SẠCH"
  // ─────────────────────────────────────────────────────────────────────────────
  // 🔵 Quyết định #8, Ice ký 2026-08-14, sau một khe hở mà code review bắt được.
  //
  // Bản trước chỉ viết `if ((await flushEditorNow()) === 'failed') return 'flush-failed'`.
  // Hai thứ đó **không** đồng nghĩa, và chỗ lệch nằm ở nhánh *originator* của
  // `flushEditorNow`: nó chụp `snapshot` **trước** lượt IPC, và khi lô về thì chỉ
  // `armFlushTimer(0)` rồi trả `'saved'` — nó **không** đệ quy như nhánh *joiner* ở trên.
  // Đó là hành vi ĐÚNG cho auto-save *(đệ quy ở đó là quay vòng trong lúc người dùng gõ
  // liên tục, đúng thứ trần cứng AD-35 tồn tại để tránh)* — nhưng nó sai cho lượt ký.
  //
  // Đường hỏng đo được: người dùng gõ nốt một ký tự **sau** khi bấm `Mod+Enter`, trong lúc
  // lô đang bay. `Store::write` là hàng đợi **nối tiếp** nên cửa sổ đó rộng ra theo số job
  // đang xếp hàng, không phải một hằng số nhỏ. Ký tự mới nằm ngoài `snapshot`, vẫn
  // `isDirty()`, nhưng flush trả `'saved'` ⇒ lượt ký đi tiếp và Rust ký **văn bản trên
  // đĩa**, tức bản thiếu ký tự cuối. Hậu quả kép, và cả hai đều im lặng:
  //   (1) một `SegmentVersion` mang văn bản **chưa bao giờ ở trên màn hình** đi **vĩnh
  //       viễn** vào lịch sử FR101;
  //   (2) timer vừa lên dây flush nốt ký tự sót qua `flush_segment_targets` — mà hàm đó
  //       **hạ trạng thái trước** ⇒ câu vừa ký **tự trở về `draft`** vài mili-giây sau.
  //
  // ⇒ Thử lại **đúng một** lượt, còn dơ nữa thì **từ chối**. Một lượt đóng ca thường (một
  //   ký tự lẻ) mà không phải đặt một trần lặp mới; đường từ chối giữ cho ca bệnh lý.
  // 🔵 2026-08-16 (code review Story 2.6): ba nhịp trên đi vào
  // [`flushEditorBeforeDiscreteWrite`]. Hành vi **không đổi một bước nào** — lượt rút ra là để
  // đường khôi phục của Story 2.6 không thể chép thiếu nó lần nữa. Lý do đầy đủ ở doc-comment
  // của hàm đó.
  const flushed = await flushEditorBeforeDiscreteWrite()
  if (flushed === 'failed') return 'flush-failed'
  if (flushed === 'still-dirty') {
    // 🔴 *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."* Chẩn đoán nêu
    // đích danh, cùng khuôn hai dòng `console.error` của `flushEditorNow`.
    console.error(
      `[editor] KHÔNG ký segment ${id}: tập chờ vẫn dơ sau hai lượt flush — ` +
        `từ chối thay vì ký một văn bản cũ hơn thứ đang trên màn hình (Quyết định #8).`,
    )
    return 'still-dirty'
  }

  // ─────────────────────────────────────────────────────────────────────────────
  // 🔴 ② MỐC SO CỦA FR117 — đọc TỪ [`segments`], và đọc TRƯỚC bước ③
  // ─────────────────────────────────────────────────────────────────────────────
  // 🔵 Story 2.7, Quyết định #2 đường (b) (Ice ký 2026-08-16). [`segments`] giữ bản **lúc
  // nạp**; doc-comment của [`editedText`] khai mệnh đề đó bằng chữ **từ Story 2.3**, tức
  // trước story này — đây là lượt đầu tiên có ai đọc nó.
  //
  // 🔴 **`segments`, KHÔNG `editedText`.** `editedText` là văn bản **đang gõ**: lấy nó làm mốc
  // thì mốc luôn bằng văn bản Rust vừa đọc trên đĩa ⇒ *"y hệt"* ở **mọi** lượt ⇒ mọi câu người
  // dùng gõ mang nhãn *của người khác*, và kho TM của Epic 7 bị trộn phong cách đúng theo cách
  // mà chính story này tồn tại để chống. Không cổng nào bắt được lượt nhầm đó — nó là hai cái
  // tên cách nhau một chữ, cùng kiểu, cùng khoá.
  //
  // ⚠️ Đọc **trước** bước ③: bước ③ vá `status` vào ảnh chụp và **cố ý không đụng**
  // `target_text` *(khối ③ ở doc-comment của `confirmCurrentSegment` ghi lý do)*. Thứ tự này
  // vì thế không phải một điều kiện đúng **hôm nay**; nó là thứ giữ cho một lượt sửa tương lai
  // ở bước ③ không lặng lẽ đổi nghĩa của mốc.
  const loaded = segments.value.find((s) => s.id === id)
  if (loaded === undefined) {
    // 🔴 *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."*
    //
    // ⚠️ **Vì sao KHÔNG có nhánh dự phòng gửi `''`, ghi ra vì đó là đường sai rẻ ở đây:** một
    // mốc rỗng đọc là *"câu này lúc nạp chưa có bản dịch"*, nên nó cho ra **`self` bất kể sự
    // thật** — tức một lời khai về chữ của ai, dựng từ một chỗ mã đã biết là mình không biết.
    // Từ chối thì người dùng bấm lại; đoán thì nhãn sai đi vĩnh viễn vào kho TM.
    //
    // ⚠️ Trạng thái này **không tới được** hôm nay *(`caretSegmentId` sinh từ `data-segment-id`
    // mà lưới render ra từ chính `segments`)*, và nó vẫn có mặt vì cái giá hai bên lệch nhau
    // rất xa. Dùng lại `'no-caret'` chứ không dựng một mã thứ sáu: câu nói với người dùng y hệt
    // — *"chưa xác định được câu nào để ký"* — và một mã mới đòi một khoá `vi.json` cho một
    // nhánh chưa ai đi qua, đúng thứ luật danh mục đóng của `MessageKey` cấm.
    console.error(
      `[editor] KHÔNG ký segment ${id}: không tìm thấy nó trong ảnh chụp lúc nạp, nên ` +
        `KHÔNG có mốc so cho FR117 — từ chối thay vì đoán một xuất xứ (Story 2.7).`,
    )
    return 'no-caret'
  }

  const { outcome, error } = await confirmSegment(id, loaded.target_text)
  if (outcome === null) {
    // ⚠️ `error === null` cũng vào đây: đó là ca *"không có cầu IPC"* (`npm run dev` trong một
    //    trình duyệt thường). Không ca nào được coi là đã xác nhận.
    confirmError.value = error
    return 'refused'
  }
  confirmError.value = null

  // ③ Ảnh chụp hiển thị.
  const index = segments.value.findIndex((s) => s.id === id)
  if (index >= 0) {
    const next = [...segments.value]
    next[index] = { ...next[index], status: outcome.status }
    segments.value = next

    // ② Quyết định #1(a) — dời con trỏ sang câu kế tiếp, nếu có.
    //
    // 🔴 `.at()` chứ **không** `next[index + 1]`, và đó là một lượt **sửa kiểu cho nó nói
    // thật** chứ không một lượt né cổng. `noUncheckedIndexedAccess` đang TẮT, nên chỉ mục
    // mảng khai kiểu `ChapterSegment` — không `| undefined` — dù lúc chạy nó **là**
    // `undefined` ở câu cuối Chương. Với kiểu đó, `if (following !== undefined)` là một
    // nhánh mà `@typescript-eslint/no-unnecessary-condition` **đỏ** *(cổng thứ mười, đo
    // 2026-08-14)*, và đường sai rẻ là nhét một `eslint-disable`. `.at()` khai đúng
    // `ChapterSegment | undefined`, nên cả kiểu lẫn phép kiểm cùng nói một sự thật.
    const following = next.at(index + 1)
    if (following !== undefined) {
      setEditorCaret(following.id)
      caretPlacement.value = following.id
    } else {
      // 🔵 STORY 2.10, Quyết định #6(c) — MÓN NỢ `deferred-work.md:2837-2847` ĐÓNG MỘT NỬA.
      //
      // Đây là **câu cuối Chương**: không có câu kế để dời con trỏ sang, nên `primary` vẫn
      // thắng `confirmed` ở `resolveSegmentRule` và vạch lề **không đổi màu** dù `segment.status`
      // trong CSDL đã đúng. Nó xảy ra **đúng một lần mỗi Chương**, ở đúng câu cuối.
      //
      // 🔴 Không vá bằng cách đảo thứ tự `primary`/`confirmed` *(quyết định có chữ ký, không
      // đảo)*, và không vá bằng `setEditorCaret(null)` *(đường (b), Ice loại: nó dựng một trạng
      // thái `caretSegmentId === null` trong khi DOM focus VẪN trong ô — hai nguồn sự thật nói
      // ngược nhau, không cổng nào canh — và `onSelectionChange` đặt lại id ở lượt dịch caret
      // kế tiếp nên hiệu lực thị giác chỉ là tạm)*.
      //
      // ⇒ Đóng bằng **thông tin**. 🟡 Vế thị giác **vẫn hụt** và đã ghi lại vào sổ nợ kèm chủ.
      kyTrungCauCuoi = true
    }
  }

  return 'confirmed'
}

const omitError = shallowRef<IpcError | null>(null)
/**
 * Lỗi gần nhất Rust trả lời cho một lượt cắt bỏ / bỏ cờ. `null` ⇒ chưa lượt nào bị từ chối.
 *
 * ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện** — cùng hình dạng và cùng
 * món nợ với [`editorConfirmError`]: giá trị này được export mà **chưa component nào đọc**,
 * nên hai khoá `err.segment.*` mà đường cắt bỏ dùng lại đều dừng ở biên giới TypeScript,
 * không tới màn hình. Đừng đọc dòng này thành *"đã có đường ra màn hình"*.
 */
export const editorOmitError: DeepReadonly<Ref<IpcError | null>> = readonly(omitError)

/**
 * Kết quả một lượt cắt bỏ / bỏ cờ.
 *
 * ⚠️ Hai giá trị **thành công** chứ không một, và đó là chủ ý: chỗ gọi ghi chẩn đoán nêu
 * đích danh việc vừa xảy ra, và hai lệnh của `CommandRegistry` phân biệt được nhau ở đây
 * mà không phải đọc lại tham số của chính mình.
 */
export type OmitResult = 'omitted' | 'restored' | 'no-caret' | 'refused'

/**
 * **Cắt bỏ câu đang có con trỏ khỏi bản dịch, hoặc bỏ cờ đó** — Story 2.5c · FR133 · AC1 ·
 * AC2 · AC4.
 *
 * 🔴 *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."* Không câu nào đang chạm
 * là một câu trả lời **hợp lệ** (`'no-caret'`), không một lỗi.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHÔNG FLUSH TRƯỚC — và đó là một khác biệt CÓ LÝ DO với [`confirmCurrentSegment`]
 * ─────────────────────────────────────────────────────────────────────────────
 * `confirmCurrentSegment` **phải** flush trước vì nó ký **văn bản trên đĩa** (AD-35 vế (c)):
 * ký sớm là ghi một `SegmentVersion` mang văn bản chưa bao giờ ở trên màn hình. Lệnh này
 * không đọc `target_text` một chữ nào và câu `UPDATE` của nó chạm **đúng một cột**
 * (`commands/segment.rs::set_segment_omitted`), nên tập chờ có dơ hay không **không đổi kết
 * quả**. Ép một lượt flush ở đây là bắt người dùng chờ một lượt IPC cho một thứ không ai
 * đọc.
 * ⚠️ Mệnh đề đó **được đo**, không được giả định: `tests/frontend/editorOmitSegment.test.ts`
 * §④ khẳng định không lượt `saveSegmentTargets` nào được phát.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 ẢNH CHỤP DỰNG BẰNG **MẢNG MỚI**, và phần tử dựng bằng **trải phần tử cũ**
 * ─────────────────────────────────────────────────────────────────────────────
 * Hai vế, hai đường hỏng khác nhau:
 * - **Mảng mới** vì [`segments`] là `shallowRef` — sửa tại chỗ không bắn watcher nào, nên
 *   hàng đã cắt bỏ sẽ không bao giờ gạch ngang trong khi đĩa đã đổi. Cùng khuôn và cùng lý
 *   do đã ghi ở [`confirmCurrentSegment`].
 * - **Trải phần tử cũ** (`{ ...next[index], is_omitted }`) chứ không dựng lại từ `outcome`:
 *   `OmitOutcome` chỉ chở hai trường, nên một phần tử dựng từ nó sẽ **mất** `status`,
 *   `target_text` và `source_text`. Đó là AC2 nói bằng ngôn ngữ của ảnh chụp — cắt bỏ đổi
 *   **một** trường, không dựng lại một hàng.
 *
 * ⚠️ **Không** dời con trỏ, khác [`confirmCurrentSegment`]. Quyết định #1(a) của Story 2.5
 * dời con trỏ vì lượt ký **kết thúc** việc với một câu; cắt bỏ thì không — người dùng có thể
 * muốn bỏ cờ ngay nếu vừa bấm nhầm, và AC4 nói thao tác này **đảo ngược được**.
 */
export async function setCurrentSegmentOmitted(omitted: boolean): Promise<OmitResult> {
  const id = caretSegmentId.value
  if (id === null) return 'no-caret'

  const { outcome, error } = await setSegmentOmitted(id, omitted)
  if (outcome === null) {
    // ⚠️ `error === null` cũng vào đây: đó là ca *"không có cầu IPC"* (`npm run dev` trong một
    //    trình duyệt thường). Không ca nào được coi là đã đặt cờ.
    omitError.value = error
    return 'refused'
  }
  omitError.value = null

  // 🔵 2026-08-18 (code review lượt HAI) — BIẾN THỂ THỨ BA của cùng một khuyết tật.
  //
  // Lượt cắt bỏ này **là** thao tác vừa xảy ra, nên nó sở hữu thanh trạng thái; điều nó có để nói
  // là *"không có gì để nói"*. Thiếu dòng này, một `'at-last'` ghi lúc trước **kẹt lại** và —
  // vì `navNoticeKey` đứng trước `secondsSinceSave` trong chuỗi `v-else-if` (`StatusBar.vue:267`)
  // — nó che mốc *"Đã lưu N giây trước"* vô thời hạn. Cùng tai hoạ mà [`dieuHuongVaBao`] mô tả.
  datThongBao({})

  const index = segments.value.findIndex((s) => s.id === id)
  if (index >= 0) {
    const next = [...segments.value]
    next[index] = { ...next[index], is_omitted: outcome.is_omitted }
    segments.value = next
  }

  return outcome.is_omitted ? 'omitted' : 'restored'
}

/** Kết quả một lượt đổi **cờ kết đoạn của bản dịch** — cùng hình dạng với [`OmitResult`]. */
export type ParagraphEndResult = 'no-caret' | 'refused' | 'ended' | 'joined'

/**
 * **Đặt hay bỏ cờ kết đoạn của BẢN DỊCH** cho câu đang có con trỏ — Story 2.5d · FR134 ·
 * AD-46 · AC2 · Quyết định #3 đường (c) (Ice ký 2026-08-15).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 ẢNH CHỤP DỰNG BẰNG **MẢNG MỚI**, phần tử dựng bằng **trải phần tử cũ**
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng hai vế và cùng hai đường hỏng với [`setCurrentSegmentOmitted`] ngay trên:
 * [`segments`] là `shallowRef` *(sửa tại chỗ không bắn watcher nào)*, và `ParagraphEndOutcome`
 * chỉ chở **hai** trường nên một phần tử dựng lại từ nó sẽ **mất** `status`, `target_text`,
 * `source_text` và cả `is_paragraph_end`. Đổi cờ đoạn đổi **một** trường.
 *
 * 🔴 **KHÔNG đụng `is_paragraph_end`** khi trải — AD-37 vẫn sở hữu cờ nguồn, và AD-46 khai
 * bằng chữ *"AD-37 không sửa một chữ"*.
 *
 * ⚠️ **Không** dời con trỏ, cùng lý do [`setCurrentSegmentOmitted`]: thao tác này đảo ngược
 * được, và người dùng có thể muốn bỏ ngay nếu vừa bấm nhầm.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ KHÔNG CÓ Ô LỖI RIÊNG CHO LỆNH NÀY, VÀ ĐÓ LÀ MỘT LỰA CHỌN CÓ GHI NỢ
 * ─────────────────────────────────────────────────────────────────────────────
 * Kho hôm nay có **hai** ô lỗi cho hai lệnh Editor, và **một** trong hai chưa ai đọc:
 * [`editorConfirmError`] được `GridPanel.vue` hiện ở cột trạng thái; [`editorOmitError`] thì
 * được export mà **không component nào đọc**. Thêm một ô thứ **ba** ở đây là nhân một bề mặt
 * chết — nên lượt từ chối đi ra bằng **giá trị trả về**, và `main.ts` ghi chẩn đoán.
 * 🔴 Vế *"người dùng thấy được vì sao lượt đổi cờ trượt"* vì thế **còn hở**, ghi nợ có chủ ở
 * `deferred-work.md`. Đừng đọc dòng này thành *"đã có đường ra màn hình"*.
 */
export async function setCurrentSegmentParagraphEnd(
  endsParagraph: boolean,
): Promise<ParagraphEndResult> {
  const id = caretSegmentId.value
  if (id === null) return 'no-caret'

  const { outcome } = await setSegmentParagraphEnd(id, endsParagraph)
  if (outcome === null) {
    // ⚠️ Ca *"không có cầu IPC"* (`npm run dev` trong một trình duyệt thường) cũng vào đây.
    //    Không ca nào được coi là đã đặt cờ.
    return 'refused'
  }

  // 🔵 2026-08-18 (code review lượt HAI) — xem khối lý do cùng dòng ở [`setCurrentSegmentOmitted`].
  datThongBao({})

  const index = segments.value.findIndex((s) => s.id === id)
  if (index >= 0) {
    const next = [...segments.value]
    next[index] = { ...next[index], is_target_paragraph_end: outcome.is_target_paragraph_end }
    segments.value = next
  }

  return outcome.is_target_paragraph_end ? 'ended' : 'joined'
}

/**
 * **Nhảy tới câu chưa dịch kế tiếp** — Story 2.5b, AC12.
 *
 * ⇒ Trả `true` khi con trỏ **đã dời**, `false` khi không còn câu nào.
 *
 * 🔴 *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."* Hết Chương là một câu
 * trả lời **hợp lệ**, không một lỗi: nó nghĩa là **không còn câu nào chưa dịch ở phía dưới**.
 * Con trỏ **ở nguyên**, và chỗ gọi **báo ra màn hình**.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 SỬA 2026-08-18 (code review lượt HAI) — hai mệnh đề của đoạn trên đã hết đúng
 * ─────────────────────────────────────────────────────────────────────────────
 * ① Hợp âm **không còn là `⌥↓`** — `commands/index.ts:1117` nay khai `Mod+Alt+ArrowDown`
 *    (`⌘⌥↓`), và khối lý do `commands/index.ts:1069-1112` giải thích vì sao `⌥↓` **trần** sai
 *    *(bị nuốt trong vùng gõ)*. Giữ `⌥↓` ở đây là tái tạo đúng ngộ nhận Story 2.10 vừa sửa.
 *    ⇒ **Gỡ hẳn số hợp âm khỏi dòng đầu** thay vì cập nhật nó: hợp âm là thứ người dùng gán
 *    lại được (`ChordOverrides`, Story 1.21), nên một bản chép ở đây sẽ lệch lần nữa.
 * ② *"chỗ gọi ghi một dòng chẩn đoán"* mô tả `console.info` mà `commands/index.ts:517` đã
 *    **gỡ** ở chính Story 2.10 — `console` **là** im lặng theo định nghĩa của dự án (AC6).
 *
 * ⚠️ Dùng lại **đúng** đường dời con trỏ mà Quyết định #1 của Story 2.5 đã dựng
 * (`setEditorCaret` + [`caretPlacement`]), không một đường thứ hai. Đó cũng là lý do một lượt
 * nhảy **flush câu vừa rời**: `setEditorCaret` mang vế *"rời segment"* của AD-35 vế (c).
 *
 * 🔴 Phép chọn sống ở `./segmentNavigation.ts` — một **module thuần** kiểm được bằng Node
 * trần. Chỗ này chỉ **xếp thứ tự** hai lượt gọi.
 */
export function goToNextUntranslated(): boolean {
  return doiConTroToi(nextUntranslatedId(danhSachDieuHuong(), caretSegmentId.value))
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 2.10 — HAI LỆNH TUẦN TỰ. AC1 · AC2 · AC7
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Ảnh chụp hiển thị **cộng** tập chờ đang gõ, ở dạng ba vị từ thuần cần.
 *
 * ⚠️ Vế *"cộng tập chờ đang gõ"* không phải một chi tiết: văn bản **chưa flush** không nằm
 * trong hàng dữ liệu, và bỏ nó làm lệnh *"câu chưa dịch kế tiếp"* nhảy **vào chính câu người
 * dùng vừa gõ xong** — đúng ca thường nhất của phím này. Xem [`navigationSegmentOf`].
 */
function danhSachDieuHuong(): readonly NavigationSegment[] {
  return segments.value.map((s) => navigationSegmentOf(s, editedText.value))
}

/**
 * 🔴 **ĐƯỜNG DỜI CON TRỎ, VÀ NÓ CÓ ĐÚNG MỘT BẢN.** Ba lệnh điều hướng đều đi qua đây.
 *
 * ⇒ Trả `true` khi con trỏ **đã dời**, `false` khi không còn câu nào phía đó.
 *
 * 🔴 *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."* Hết Chương là một câu trả
 * lời **hợp lệ**, không một lỗi. Con trỏ **ở nguyên**, và chỗ gọi báo ra màn hình *(Quyết định
 * #5(a) và AC6 — `console` **là** im lặng theo định nghĩa của dự án)*.
 *
 * 🔴 **Hai dòng dưới đây là hai việc KHÁC nhau, và thứ tự không đảo được:**
 *   ① `setEditorCaret(id)` mang vế **(d) *rời segment*** của hợp đồng flush AD-35 — nó **ghi
 *      xuống đĩa** văn bản của câu vừa rời (`:145-150`). Mỗi lượt điều hướng vì thế lưu bài
 *      **miễn phí**, *nếu* đi qua hàm này.
 *   ② `caretPlacement.value` là yêu cầu *"đặt caret DOM vào đầu ô"* mà watcher **đường lệnh**
 *      ở `GridPanel.vue` đọc. Đường **chuột** không đi qua nó, có chủ ý.
 *
 * ⚠️ **Một đường dời con trỏ thứ hai làm mất chữ người dùng vừa gõ, IM LẶNG, và không cổng nào
 * đỏ.** Đó là toàn bộ lý do hàm này tồn tại thay vì ba bản sao ba dòng.
 *
 * ⚠️ AD-47 **không** bị chạm: flush theo AD-35 chở đúng **bộ đệm gõ**, nên xuất xứ vẫn là *tôi
 * dịch* và không cột xuất xứ nào phải đặt. `ARCHITECTURE-SPINE.md` khai vế này bằng chữ.
 */
function doiConTroToi(id: number | null): boolean {
  if (id === null) return false
  setEditorCaret(id)
  caretPlacement.value = id
  return true
}

/**
 * **Segment kế tiếp** — AC1. Điều hướng **theo vị trí**: nó **không** bỏ qua câu đã cắt bỏ.
 *
 * ⇒ Trả `false` ở **cuối** Chương; chỗ gọi ghi `'at-last'`.
 */
export function goToNextSegment(): boolean {
  return doiConTroToi(nextSegmentId(danhSachDieuHuong(), caretSegmentId.value))
}

/**
 * **Segment trước đó** — AC2. Xem [`goToNextSegment`].
 *
 * ⇒ Trả `false` ở **đầu** Chương; chỗ gọi ghi `'at-first'`.
 */
export function goToPrevSegment(): boolean {
  return doiConTroToi(prevSegmentId(danhSachDieuHuong(), caretSegmentId.value))
}

/**
 * 🔴 **AC6 · AC7 — BÁO RA MÀN HÌNH, và đây là chỗ nó được thi hành.**
 *
 * ⚠️ Trước Story 2.10, nhánh *"hết câu chưa dịch"* chỉ có một `console.info` ở
 * `commands/index.ts`. Theo định nghĩa của dự án, `console` **là** im lặng: người dùng bấm phím
 * và **không một pixel nào đổi**. Đó là *"rỗng IM LẶNG"* áp lên một thao tác người dùng **chủ
 * động** — ca tệ nhất của lớp lỗi đó, và là lý do AC6 tồn tại.
 *
 * 🔴 Ghi câu báo **chỉ khi không dời được**. Một lượt dời **thành công** không được ghi **CÂU**
 * nào: nó đã tự nói bằng vạch lề và bằng vị trí caret, và một câu thừa trên thanh 34px sẽ **đẩy**
 * mốc *"Đã lưu N giây trước"* mà UX-DR30 đòi.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 SỬA 2026-08-17 (code review ba tầng) — *"không ghi CÂU nào"* KHÔNG bằng *"không DỌN gì"*
 * ─────────────────────────────────────────────────────────────────────────────
 * Bản đầu của hàm này viết `if (!daDoi) ghiNavNotice(...)` rồi `return` — nhánh thành công
 * **không chạm** một ô nhớ nào. Hai tầng rà độc lập tìm ra cùng khuyết tật, và nó tái hiện được:
 *
 * > Con trỏ ở câu cuối ⇒ `next_segment` ⇒ thanh hiện *"Đã ở câu cuối Chương…"*. Bấm
 * > `prev_segment` ⇒ **dời được** ⇒ thanh **vẫn** hiện câu đó, dù con trỏ đã đi.
 * > *(Đo: `editorNavNotice.value` còn `'at-last'` sau một lượt trả `true`.)*
 *
 * ⚠️ **Và nó gây ra ĐÚNG tai hoạ mà đoạn lý do trên muốn tránh.** `navNoticeKey` đứng **trước**
 * `secondsSinceSave` trong chuỗi `v-else-if` (`StatusBar.vue:267-269`), nên một câu kẹt lại **che
 * mốc *"Đã lưu N giây trước"* vô thời hạn** — cho tới khi người dùng tình cờ gõ chữ, ký, hoặc
 * gộp/tách *(ba đường duy nhất còn gọi [`datThongBao`])*. Trong một ứng dụng mà NFR18 là *"mất ≤
 * 5 s công việc"*, mốc đó là dấu hiệu **duy nhất** người dùng thấy rằng bài đang được lưu.
 *
 * 🔴 Lời giải là [`datThongBao`] với **tham số rỗng** — nó dọn cả ba ô và **không** thêm câu nào,
 * tức thoả trọn đoạn lý do trên. Khuôn này đã có **hai** tiền lệ trong chính tệp này:
 * [`noteEditorEdit`] *(người dùng gõ tiếp ⇒ mọi câu cũ hết đúng)* và nhánh `'confirmed'` của
 * [`confirmCurrentSegment`] *(`{ nav: … ? 'confirmed-last' : null }` — vế `null` là đúng lượt dọn
 * này)*.
 *
 * ⚠️ **Vì sao lượt dọn này KHÔNG cướp câu của `confirm`/`regroup`:** cả hai đường đó dời con trỏ
 * bằng `setEditorCaret` **trực tiếp** (`:923-924` · `:1616-1617`), **không** qua hàm này. Nên
 * `'confirmed-last'` và `'merged'`/`'split'` không đi qua cửa dọn ở đây. Đã kiểm từng đường gọi.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 SỬA 2026-08-18 (code review lượt HAI) — *"không dời được"* KHÔNG bằng *"không còn câu nào"*
 * ─────────────────────────────────────────────────────────────────────────────
 * Bản trước nhận một `boolean` **đã tính xong** và đọc mọi lượt trả `false` thành *"hết câu"*.
 * Nhưng một ảnh chụp **rỗng vì chưa nạp** cũng cho `false`, và câu đi ra khi đó là một lời khẳng
 * định dứt khoát về một điều màn hình **chưa biết**. Xem [`NavNotice`] nhánh `'loading'` cho
 * đường đi tái hiện được và phép đo.
 *
 * ⚠️ Lỗ này lọt **cả lượt rà thứ nhất**: nó bị nêu ở dạng *"Chương thật sự có 0 segment"* và bị
 * **bác đúng** *(món nợ 25 Chương Epic 1 đã đóng ở Story 2.1)*. Vế sống — cửa sổ `void
 * ensureSegmentsLoaded()` **không `await`** — thì chưa ai xét. Ghi ra vì bài học không nằm ở
 * khuyết tật: **một tiền đề sai không làm kết luận sai**, và một mục bị bác đáng được đọc lại
 * bằng tiền đề khác trước khi coi là đã xong.
 */
function dieuHuongVaBao(doi: () => boolean, khiKhongDoi: NavNotice): boolean {
  // 🔵 2026-08-18 (code review lượt HAI) — CỬA CHẶN ĐỨNG TRƯỚC, và nó phải đứng ở ĐÂY.
  //
  // 🔴 Vì sao tham số thành một HÀM chứ không còn là `boolean`: một `boolean` được tính **xong**
  //    trước khi vào hàm này, nên cửa chặn buộc phải nằm ở **ba** chỗ gọi — và ba bản sao của
  //    cùng một mệnh đề là đúng hình dạng mà một lượt sửa chỉ chạm hai bản sẽ đi qua mọi cổng.
  //    Với một thunk, *không tồn tại cú pháp* để gọi một lệnh điều hướng mà đi vòng qua cửa này.
  //    Cùng lý lẽ và cùng khuôn `datThongBao` §"Chốt nằm ở chữ ký hàm, không ở kỷ luật người viết".
  //
  // ⚠️ Chưa nạp xong thì **không lệnh nào được chạy** — không chỉ không báo. Chạy nó trên một
  //    ảnh chụp rỗng là dời con trỏ theo một danh sách chưa tồn tại.
  if (!editorHasLoaded()) {
    ghiNavNotice('loading')
    return false
  }
  const daDoi = doi()
  // Dời được ⇒ thao tác vừa xảy ra **là** lượt điều hướng này, nên nó sở hữu thanh trạng thái —
  // và điều nó có để nói là *"không có gì để nói"*. Tham số rỗng: dọn ba ô, thêm 0 câu.
  if (daDoi) datThongBao({})
  else ghiNavNotice(khiKhongDoi)
  return daDoi
}

/** [`goToNextUntranslated`] + câu báo AC6. Đây là thứ `commands/index.ts` gọi. */
export function goToNextUntranslatedCoBao(): boolean {
  return dieuHuongVaBao(goToNextUntranslated, 'no-untranslated')
}

/** [`goToNextSegment`] + câu báo AC7. */
export function goToNextSegmentCoBao(): boolean {
  return dieuHuongVaBao(goToNextSegment, 'at-last')
}

/** [`goToPrevSegment`] + câu báo AC7. */
export function goToPrevSegmentCoBao(): boolean {
  return dieuHuongVaBao(goToPrevSegment, 'at-first')
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 2.11 — CHUYỂN CHƯƠNG (FR26 · AC1 · AC2 · AC3 · AC4)
// ═════════════════════════════════════════════════════════════════════════════════
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 THỨ TỰ LÀ TOÀN BỘ NỘI DUNG CỦA HÀM NÀY — không một `try/catch` nào thay được nó
// ─────────────────────────────────────────────────────────────────────────────────
// 🔵 SUA 2026-08-18 (code review ba tang) — DOAN NAY TUNG PHAT BIEU MOT PHEP DO SAI.
//
// Ban cu viet: *"mot lo flush con dang bay luc `open_adjacent_chapter` doi con tro se dap
// xuong mang `chapter_id` CU, Rust tra `segment.unknown_ids`, va ban dich bien mat im lang"*.
// **Doc lai ma Rust thi no khong dung.** `save_segment_targets` (`segment.rs:1171-1193`) kiem
// `SELECT COUNT(*) FROM chapter WHERE id = ?1` roi ghi bang
// `UPDATE segment … WHERE id = ?2 AND chapter_id = ?3` — ca hai chay tren **chinh `project.db`
// dang mo**, va **khong duong nao doc `OpenWork::chapter_id`**. Khac luot doi **Tac pham**
// *(noi ca `Store` bi tro sang mot tep khac)*, Chuong cu **van con nguyen trong cung CSDL**
// sau mot luot doi Chuong ⇒ mot lo toi tre mang `chapter_id` cu **ghi dung vao Chuong cu**.
//
// ⇒ **Ket luan ve thu tu KHONG doi**, nhung **ly do doi**: thu tu duoi day dung vi tinh nhat
// quan con tro/UI *(mot luot nap cu khong duoc do segment Chuong cu len man hinh sau luot
// doi)*, khong vi mot duong mat chu qua `unknown_ids`.
//
// 🔴 **Va menh de sai ay da tra gia:** no hut het chu y ve phia mot moi nguy **khong ton
// tai**, trong khi moi nguy **co that** — nguoi dung go tiep trong cua so giua ② va ③, roi
// `flush.reset()` vut chu ay vo dieu kien — nam cach do sau dong va khong luot ra noi bo nao
// nhin. No duoc dong o [`noteEditorEdit`], bang mot cua khoa go.
//
// ⚠️ **Và điều kiện đó KHÔNG được đọc thành "trước khi dọn state".** `resetEditorPanel()`
// chạy `flush.reset()`, hàm **vứt vô điều kiện** tập chờ — gọi nó trước lượt flush là tự tay
// ăn mất bản dịch chưa lưu. Hai ràng buộc, một thứ tự: **flush → invoke → dọn → nạp**.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 AC3 LÀ MỘT LƯỢT **THI HÀNH** AD-35 VẾ (d), KHÔNG MỘT VẾ THỨ SÁU — Ice ký 2026-08-18
// ─────────────────────────────────────────────────────────────────────────────────
// AD-35 (`SPINE:423`) liệt kê đúng năm đường flush và *"chuyển Chương"* **không có tên**
// trong đó. Hai cách đọc đã được cân, và Ice ký **lập luận A**: chuyển Chương **là** rời
// segment *(vế d)* theo **cấu tạo** — không đường nào rời Chương mà không rời câu đang gõ.
// ⇒ 0 chữ của spine bị sửa, và cửa chặn AD của Task 0.4 **không** kích hoạt.
//
// *(Lập luận B đã bị loại: vế (d) trong mã được định nghĩa là* `caretSegmentId` *đổi giá trị
// — `:144-150` — tức nó đòi có một "câu B", mà một lượt chuyển Chương rời câu A không sang
// một câu B nào của Chương cũ. Nếu Ice đọc theo hướng đó thì đây đã là một AD mới.)*

/**
 * 🔴 **Chốt chống chồng lượt.** Hai lượt `⌘⌥]` liên tiếp trong khi lượt đầu còn đang bay là
 * chuyện **thường**, không một ca biên: một lượt chuyển Chương đi qua **hai** lượt IPC nối
 * tiếp *(flush, rồi `open_adjacent_chapter`)*, và phím thì không chờ ai.
 *
 * ⚠️ [`sequence`] **không** đủ cho việc này, và đó là chỗ dễ đọc nhầm: nó vô hiệu hoá một
 * lượt **nạp** đã bay, nhưng nó không ngăn hai lượt `open_adjacent_chapter` cùng chạy — mà
 * hai lượt ấy **cùng dời con trỏ Chương phía Rust**, mỗi lượt một bước. Người dùng bấm hai
 * lần và nhảy **hai** Chương, trong khi màn hình chỉ kịp nạp một.
 */
let dangChuyenChuong = false

/**
 * **Chuyển sang Chương kề** — Story 2.11 · FR26.
 *
 * Trả `true` khi Chương **thật sự** đổi. Không ném: mọi đường hỏng đi ra bằng một câu trên
 * thanh trạng thái, đúng luật *"hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU"*.
 */
async function switchChapter(direction: ChapterDirection): Promise<boolean> {
  // ⚠️ Cửa chặn *"chưa nạp xong"* — cùng mệnh đề và cùng câu báo với [`dieuHuongVaBao`]. Nó
  // được viết lại ở đây thay vì tái dùng hàm kia vì `dieuHuongVaBao` nhận một thunk **đồng
  // bộ**; bọc một `Promise` vào đó là để nó trả `true` trước khi biết kết quả.
  if (!editorHasLoaded()) {
    ghiNavNotice('loading')
    return false
  }

  if (dangChuyenChuong) {
    console.info('[editor] mot luot chuyen Chuong dang bay — bo qua luot nay')
    return false
  }
  dangChuyenChuong = true

  try {
    // ── ① FLUSH, và ĐỌC KẾT QUẢ ────────────────────────────────────────────────────
    //
    // 🔴 Cả **ba** giá trị được xử lý, và hai trong ba **CHẶN**. Khuôn đã có chữ ký:
    // `libraryImport.ts:145-150` chặn lượt đổi Tác phẩm khi flush trượt, với phán quyết ghi
    // bằng chữ — *"Người dùng bị cản một lượt và thấy lý do; họ thử lại, hoặc họ chép bản
    // dịch ra ngoài. Cả hai đường đều tốt hơn một lượt mất chữ không ai biết."*
    //
    // ⚠️ **Tiền lệ ấy chỉ phủ MỘT NỬA, và nửa kia là mới ở đây.** Đường đổi Tác phẩm gọi
    // `flushEditorNow()` — **một** lượt — nên nó chỉ có `'failed'` để đọc. Hàm hai lượt dưới
    // đây là nơi gọi **đầu tiên** của kho phải phán quyết `'still-dirty'`: một ký tự gõ
    // trong lúc lô đầu đang bay nằm **ngoài** ảnh chụp, nên `'saved'` của lượt đầu **không**
    // đồng nghĩa với *"tập chờ sạch"* (`:429-443`).
    const flushed = await flushEditorBeforeDiscreteWrite()
    if (flushed !== 'clean') {
      console.error(
        `[editor] chuyen Chuong bi CHAN: luot flush tra '${flushed}' — ban dich chua xuong dia`,
      )
      // 🔵 CODE REVIEW 2026-08-18 — thôi mượn kênh `confirm`. Hai câu cũ
      // (`panel.grid.confirm_*`) đọc nguyên văn *"…nên **chưa xác nhận**"*, tức chúng trả lời
      // về một thao tác người dùng **không hề làm**. Xem [`NavNotice`] cho lập luận đầy đủ.
      ghiNavNotice(flushed === 'failed' ? 'chapter-flush-failed' : 'chapter-still-dirty')
      return false
    }

    // 🔵 THÊM Story 5.7 (AC4/AC6) — biên "trước lượt đổi Chương" của `flushChapterPositionNow()`.
    // KHÔNG chặn: một lượt trượt chỉ mất một lời nhắc (chẩn đoán bên trong hàm đó), không
    // mất bản dịch — khác hẳn vế `flushEditorBeforeDiscreteWrite()` ngay trên.
    await flushChapterPositionNow()

    // ── ② DỜI CON TRỎ CHƯƠNG phía Rust ─────────────────────────────────────────────
    const { switched, error } = await openAdjacentChapter(direction)

    if (error !== null) {
      // 🔵 CODE REVIEW 2026-08-18 — **KHÔNG** ghi `loadError` ở đây. Bản cũ làm thế, và
      // `editorHasLoaded()` kiểm `loadError === null` nên một lỗi IPC **nhất thời** *(kể cả
      // `retryable: true`)* khoá chết cả lưới lẫn mọi lệnh điều hướng lẫn chính lượt thử lại,
      // tới khi người dùng rời Tác phẩm. Chương hiện tại **vẫn nạp tốt** — nó không có lỗi
      // gì; thứ trượt là lượt **chuyển**. Xem [`NavNotice`] cho chữ ký của Ice.
      console.error(
        `[editor] chuyen Chuong TRUOT: ${error.code} (${error.message_key}), retryable=${String(error.retryable)}`,
      )
      ghiNavNotice('chapter-switch-failed')
      return false
    }
    // Không lỗi mà cũng không kết quả ⇒ chạy ngoài Tauri (`npm run dev` trong trình duyệt
    // thường). Adapter đã ghi `console.info`; không một câu nào lên màn hình.
    if (switched === null) return false

    if (switched.outcome !== 'moved') {
      // 🔴 KHÔNG quay vòng, và báo bằng **câu của Chương**, không câu của câu.
      ghiNavNotice(switched.outcome === 'at-last' ? 'at-last-chapter' : 'at-first-chapter')
      return false
    }

    // ── ③ DỌN state của Chương cũ, RỒI ④ NẠP Chương mới ────────────────────────────
    //
    // 🔴 `resetEditorPanel()` phải chạy **sau** ②: nó tăng [`sequence`], tức vô hiệu hoá mọi
    // lượt nạp đang bay của Chương **cũ**. Chạy nó trước ② thì một lượt nạp cũ vẫn hợp lệ
    // trong cửa sổ giữa hai lời gọi và đổ segment của Chương cũ lên màn hình sau lượt đổi.
    //
    // ⚠️ `flush.reset()` bên trong nó vứt tập chờ **vô điều kiện** — an toàn ở đây **vì** ①
    // vừa trả `'clean'`, không vì hàm ấy hiền.
    resetEditorPanel()

    // ═════════════════════════════════════════════════════════════════════════════
    // 🔵 CODE REVIEW BA TẦNG 2026-08-18 — PANEL SOURCE PHẢI ĐI CÙNG LƯỢT NÀY
    // ═════════════════════════════════════════════════════════════════════════════
    // Bản cũ dọn và nạp lại **một mình** Panel Editor. Nhưng `chapterRequested` và
    // `hanVietRequested` ở `sourcePanelState.ts` là **cache module-level không có khoá vô
    // hiệu hoá** — chính doc-comment của `resetSourcePanel()` viết ra điều đó, và nó còn ghi
    // *"chỗ gọi duy nhất là `libraryImport.ts::finishSubmit`"*. Câu ấy **vẫn đúng** sau Story
    // 2.11, và đó chính là khuyết tật: sau `⌘⌥]`, lưới bản dịch sang Chương mới trong khi
    // nguyên văn + bảng âm Hán Việt + `source_lang` *(⇒ tab Hán Việt hiện/ẩn)* vẫn là của
    // Chương **cũ**. Hai panel trên cùng một màn hình nói về hai Chương khác nhau, không lỗi
    // nào. Đúng nguyên văn kịch bản `sourcePanelState.ts:352-355` đã ghi cho cấp **Tác
    // phẩm**, tái diễn ở cấp **Chương**.
    //
    // 🔴 **Vứt state cũ là CHƯA ĐỦ — phải NẠP LẠI ngay tại đây**, cùng lý do và cùng chữ đã
    // ghi ở `libraryImport.ts:173-190`: chỗ DUY NHẤT gọi `ensureChapterLoaded()` là
    // `GridPanel.vue::onMounted`, mà ba chế độ sống trong `<KeepAlive>` nên **không có
    // `mounted` lần thứ hai**. Bỏ lời gọi dưới đây là để Panel Source đứng ở *"Chưa có Chương
    // nào được mở"* cho tới lượt khởi động lại app.
    //
    // ⚠️ `resetLookupPanel()` **cố ý không** nằm trong lượt này: lịch sử và bộ ghim tra cứu
    // thuộc **Tác phẩm**, không thuộc Chương *(`lookupHistoryState.ts:348-357`)*. Vứt chúng
    // ở một lượt đổi Chương là xoá đúng thứ người dùng vừa tra để dịch chương này.
    resetSourcePanel()

    // `resetEditorPanel()` đặt `requested = false`, nên lượt gọi này chạy IPC thật.
    await ensureSegmentsLoaded()
    await ensureChapterLoaded()

    // 🔵 Story 3.4b — dấu thuật ngữ Glossary của Chương MỚI, đúng một lượt IPC ở đây. Cả hai
    // `await` phía trên đã xong tại điểm này (không phải một lượt gọi TRÙNG đang bay của một
    // chỗ khác — đây LÀ lượt gọi gốc của chính `switchChapter`), nên `segments.value`/
    // `chapterId.value`/`sourceChapter.value` đều đã khớp Chương mới **trong ca thường**.
    // `chapterId.value` có thể là `null` nếu lượt nạp segment vừa trượt — không gọi gì trong ca
    // đó, cùng luật mọi nhánh khác của hàm này (một lỗi nhất thời không được kéo theo một lỗi
    // thứ hai không liên quan).
    //
    // 🔴 **`chapterId.value === sourceChapter.value.chapter_id` là BẮT BUỘC, không phải một
    // hàng rào thừa** — bắt ở lượt rà 2026-08-21 (ba lớp review). Hai `await` phía trên chạy
    // qua **hai lệnh IPC riêng** (`readOpenChapterSegments` và `readOpenChapter`); mỗi lệnh có
    // `sequence`/vô hiệu hoá RIÊNG của chính nó, không có khoá chung. Hai lượt `switchChapter()`
    // gọi LIÊN TIẾP nhanh (bấm `⌘⌥]` hai lần trước khi lượt đầu về) có thể để lượt ĐẦU trả lời
    // SAU lượt hai: tại điểm này, `chapterId.value` mang Chương B (của lượt hai, mới nhất) mà
    // `sourceChapter.value` vẫn còn mang Chương A (của lượt đầu, chưa bị vượt mặt xong) — hoặc
    // ngược lại. Gọi `ensureGlossaryMarksLoaded` với một cặp LỆCH NHAU đó nạp dấu SAI Chương
    // rồi gán cho Chương đang hiện — đúng bằng chính khuyết tật mà `GridPanel.vue`'s watcher
    // (`chapterId !== chapter.chapter_id`) đã tồn tại để chặn. Hai chỗ gọi phải giữ ĐÚNG một
    // điều kiện, không phải hai biến thể của cùng một ý.
    if (
      chapterId.value !== null &&
      sourceChapter.value !== null &&
      chapterId.value === sourceChapter.value.chapter_id
    ) {
      void ensureGlossaryMarksLoaded(chapterId.value, segments.value, sourceChapter.value.source_lang)
    }

    // ── ⑤ TIÊU ĐIỂM phải Ở LẠI trong lưới — AD-34 §2 ───────────────────────────────
    //
    // 🔴 Lượt chuyển thay **toàn bộ** hàng của `v-for`, và `segment.id` là
    // `AUTOINCREMENT` **theo Tác phẩm** nên Chương mới gần như chắc chắn mang một tập khoá
    // khác ⇒ Vue **gỡ** đúng cái ô `contenteditable` đang giữ tiêu điểm. Trình duyệt trả
    // tiêu điểm về `document.body`, và AD-34 §2 cấm thẳng chuyện đó: từ `body` thì không
    // hợp âm nào của vùng gõ còn nghĩa, và vòng xoay panel mất điểm bắt đầu.
    //
    // ⚠️ `await nextTick()` là **bắt buộc**, không một lượt phòng xa: tại thời điểm
    // `ensureSegmentsLoaded()` trả về, Vue **chưa** vá DOM — ô cũ vẫn còn, tiêu điểm vẫn
    // hợp lệ, và một `enterFocus` ở đó là no-op rồi tiêu điểm rơi ngay sau đó.
    //
    // ⚠️ `enterFocus` **không** cần một thành viên mới trong `FOCUS_OWNERS`: `panel.grid`
    // đã ở đó từ Story 2.5b. Và nó **tự bỏ qua** khi tiêu điểm đã nằm trong owner — bản vá
    // gốc của `focus.ts::enter()` mà Ice ký ở 2.5b — nên lời gọi này chỉ có tác dụng đúng
    // ở ca nó tồn tại để đóng.
    //
    // 🔴 **VẾ NÀY CHƯA CÓ ĐƯỜNG NGHIỆM THU, và tôi KHÔNG chấm nó đạt.** `happy-dom` không
    // phải WebKit, còn e2e thì **không tới được** một lượt chuyển thành công *(không đường
    // sản phẩm nào sinh Chương thứ hai — cùng món nợ với AC1/AC2)*. Ghi nợ có chủ.
    await nextTick()
    enterFocus('panel.grid')
    return true
  } finally {
    dangChuyenChuong = false
  }
}

/**
 * **Chương sau** — AC1. Đây là thứ `commands/index.ts` gọi *(qua `CommandDeps`)*.
 *
 * ⚠️ `void`, không `Promise<boolean>`: `CommandRegistry.run` là đồng bộ, và khuôn
 * fire-and-forget này đã có tiền lệ ở `submitPastedText`/`submitFilePath` *(`index.ts:207`)*.
 * Mọi kết quả — kể cả một lượt bị chặn — đã đi ra bằng thanh trạng thái bên trong
 * [`switchChapter`].
 */
export function goToNextChapter(): void {
  void switchChapter('next')
}

/** **Chương trước** — AC2. Xem [`goToNextChapter`]. */
export function goToPrevChapter(): void {
  void switchChapter('prev')
}

/**
 * **Mở một Chương đích danh** — Story 5.7, AC3. Chép TRỌN khuôn [`switchChapter`]: cờ chống
 * hai lượt cùng bay *(dùng CHUNG `dangChuyenChuong` — cả hai đều là "đang đổi Chương", và
 * chạy đồng thời là cùng một cuộc đua đúng lý lẽ đã ghi ở đầu hàm kia)*, flush đọc CẢ BA giá
 * trị, một lượt IPC, dọn state Chương cũ, nạp lại Chương mới, kiểm `chapterId` khớp giữa hai
 * lượt IPC trước khi nạp dấu Glossary — nguyên văn khuôn của `switchChapter`, chỉ khác bước
 * ② gọi [`openChapter`] (đích danh) thay vì [`openAdjacentChapter`] (theo hướng).
 *
 * Trả `true` khi Chương **thật sự** đổi. Không ném — cùng luật [`switchChapter`].
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 **KHÔNG có cửa `editorHasLoaded()` ở đây, và đó là chỗ hàm này KHÁC [`switchChapter`]**
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 **GỠ 2026-08-29 (Story 5.7), sau một lượt e2e ĐỎ trong WKWebView thật** — bản đầu chép
 * cả cửa ấy từ [`switchChapter`], và nó **chặn đúng đường chính của story**:
 * `openCurrentChapter` chạy khi người dùng đang ở **Library**, nơi Workspace có thể **chưa
 * mount lần nào** trong phiên. `editorHasLoaded()` đọc `chapterId.value !== null`, mà giá trị
 * đó chỉ được đặt bởi [`ensureSegmentsLoaded`] — và chỗ gọi nó là `GridPanel.vue::onMounted`.
 * ⇒ Ở lượt khởi động lạnh, cửa này **luôn** đóng, hàm trả `false`, `setMode('workspace')`
 * không bao giờ chạy, và màn hình đứng im **không một lỗi nào**.
 * *(Đo: bàn đo `story-5-7-open-chapter.e2e.mjs` đỏ với `rows: 1`, nút "Mở Chương" tồn tại,
 * `disabled = false`, hai dải lỗi RỖNG — tức mọi thứ đúng trừ lượt mở.)*
 *
 * 🔴 **Và gỡ nó KHÔNG mở một cửa mất chữ nào.** Cửa thật sự canh dữ liệu là lượt flush ở
 * bước ① ngay dưới, và nó đứng nguyên. Cửa `editorHasLoaded()` ở [`switchChapter`] phục vụ
 * một mệnh đề khác hẳn: *Chương kề* là một đích **TƯƠNG ĐỐI**, tính từ Chương đang mở, nên
 * hỏi nó khi chưa biết Chương nào đang mở là vô nghĩa. Đích của hàm này là **TUYỆT ĐỐI** —
 * một `chapter.id` người dùng vừa chọn trong danh sách — nên nó không cần biết trạng thái nạp
 * của lưới để đúng.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 THÊM (2026-08-29, Story 5.9) — THAM SỐ `targetSegmentId`, CHO KẾT QUẢ TÌM KIẾM
 * ─────────────────────────────────────────────────────────────────────────────
 * `src/modes/librarySearch.ts::openSearchHit` cần đặt con trỏ ĐÚNG câu khớp sau khi mở Chương
 * — không chỉ mở đúng Chương. Sau khi `ensureSegmentsLoaded()` đã nạp xong, nếu
 * `targetSegmentId` có mặt TRONG `segments.value` (`.some((s) => s.id === targetSegmentId)`),
 * lượt này đi qua ĐÚNG MỘT đường dời con trỏ (`doiConTroToi`, `:1367` — không một bản thứ
 * hai, đúng khuôn `goToNextSegment`/`goToPrevSegment`). `targetSegmentId` KHÔNG có trong
 * `segments` (segment đã VỀ HƯU giữa lúc chỉ mục tìm kiếm quét và lúc người dùng bấm kết quả)
 * ⇒ giữ nguyên con trỏ mà RUST đã quyết (`caret_segment_id` từ `ensureSegmentsLoaded`), VÀ ghi
 * một chẩn đoán nêu đích danh — không lượt mở nào bị HUỶ vì chuyện này (§I/O Matrix "Mở một
 * kết quả đã cũ").
 */
export async function openChapterById(
  targetChapterId: number,
  targetSegmentId?: number,
): Promise<boolean> {
  if (dangChuyenChuong) {
    console.info('[editor] mot luot chuyen Chuong dang bay — bo qua luot nay')
    return false
  }
  dangChuyenChuong = true

  try {
    // ── ① FLUSH, và ĐỌC KẾT QUẢ — cùng khuôn `switchChapter` bước ① ───────────────
    const flushed = await flushEditorBeforeDiscreteWrite()
    if (flushed !== 'clean') {
      console.error(
        `[editor] mo Chuong bi CHAN: luot flush tra '${flushed}' — ban dich chua xuong dia`,
      )
      ghiNavNotice(flushed === 'failed' ? 'chapter-flush-failed' : 'chapter-still-dirty')
      return false
    }

    // Biên "trước lượt đổi Chương" của `flushChapterPositionNow()` — KHÔNG chặn, cùng lý do
    // đã ghi ở `switchChapter`.
    await flushChapterPositionNow()

    // ── ② MỞ Chương đích danh phía Rust ────────────────────────────────────────────
    const { chapter, error } = await openChapter(targetChapterId)

    if (error !== null) {
      console.error(
        `[editor] mo Chuong TRUOT: ${error.code} (${error.message_key}), retryable=${String(error.retryable)}`,
      )
      ghiNavNotice('chapter-switch-failed')
      return false
    }
    // Không lỗi mà cũng không kết quả ⇒ chạy ngoài Tauri. Cùng nhánh `switchChapter`.
    if (chapter === null) return false

    // ── ③ DỌN state của Chương cũ, RỒI ④ NẠP Chương mới — cùng khuôn `switchChapter` ──
    resetEditorPanel()
    resetSourcePanel()

    // `resetEditorPanel()` đặt `requested = false`, nên lượt gọi này chạy IPC thật.
    await ensureSegmentsLoaded()
    await ensureChapterLoaded()

    // 🔵 THÊM (2026-08-29, Story 5.9) — đặt con trỏ vào ĐÚNG câu khớp của một kết quả tìm
    // kiếm, SAU khi segments đã nạp xong (danh sách để so `targetSegmentId` vào). Xem khối
    // doc-comment của hàm này.
    if (targetSegmentId !== undefined) {
      if (segments.value.some((s) => s.id === targetSegmentId)) {
        doiConTroToi(targetSegmentId)
      } else {
        // KHÔNG huỷ lượt mở vì chuyện này -- segment đã về hưu giữa lúc chỉ mục tìm kiếm quét
        // và lúc người dùng bấm kết quả (chỉ mục chưa quét lại). Giữ nguyên con trỏ Rust đã
        // quyết (`caret_segment_id`, đặt trong `ensureSegmentsLoaded`).
        console.error(
          `[editor] mo ket qua tim kiem: segmentId=${targetSegmentId} khong con trong Chuong ` +
            `${targetChapterId} (co the da ve huu) — giu con tro Rust da quyet`,
        )
      }
    }

    // Cùng điều kiện `switchChapter` đã ghi: hai `await` phía trên đi qua hai lệnh IPC RIÊNG,
    // mỗi lệnh một `sequence` không chung khoá — kiểm khớp trước khi nạp dấu Glossary.
    if (
      chapterId.value !== null &&
      sourceChapter.value !== null &&
      chapterId.value === sourceChapter.value.chapter_id
    ) {
      void ensureGlossaryMarksLoaded(chapterId.value, segments.value, sourceChapter.value.source_lang)
    }

    // ── ⑤ TIÊU ĐIỂM phải Ở LẠI trong lưới — cùng khuôn `switchChapter` ─────────────
    await nextTick()
    enterFocus('panel.grid')
    return true
  } finally {
    dangChuyenChuong = false
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.8 — TÁCH CHƯƠNG tại câu đang có tiêu điểm (FR15 · AD-32)
// ═════════════════════════════════════════════════════════════════════════════════
// Điểm tách sống ở Editor, không ở Library: `caretSegmentId` là chỗ DUY NHẤT trong kho biết
// câu nào đang được chọn (§Design Notes "Điểm tách sống ở Editor, không ở Library").

/**
 * **Tách Chương đang mở tại câu đang có tiêu điểm** — hợp âm `Mod+Shift+Slash`
 * (`editor.split_chapter`). Story 5.8, Task 16.
 *
 * 🔴 *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."* `caretSegmentId === null`
 * là trạng thái BÌNH THƯỜNG của một panel chỉ-đọc (không câu nào đang được chạm, cùng lý lẽ
 * doc-comment của [`caretSegmentId`]) — ghi chẩn đoán nêu đích danh rồi trả `false`, và
 * **KHÔNG một lượt IPC nào được phát** (§I/O Matrix "Tách khi caret trống").
 *
 * Flush TRƯỚC (AD-35 vế (c), cùng cửa mọi lệnh ghi rời rạc khác của tệp này) — trượt cũng
 * KÊU rồi trả `false`, không ném, đúng cùng luật.
 *
 * Trả `true` khi Chương THẬT SỰ tách. **Không tự chuyển chế độ** — đó là đoán ý người dùng
 * (`src/AGENTS.md`).
 */
export async function splitChapterHere(): Promise<boolean> {
  const segmentId = caretSegmentId.value
  if (segmentId === null) {
    console.error(
      '[editor] KHONG tach Chuong: khong co cau nao dang duoc chon (caretSegmentId === null)',
    )
    // 🔵 Story 5.8, lượt rà — một dòng `console.error` KHÔNG phải một câu trả lời cho người
    // dùng. Xem doc-comment của [`splitChapterNotice`] cho phép đo đứng sau khối này.
    splitChapterError.value = null
    datThongBao({ splitChapter: 'no-caret' })
    return false
  }

  const flushed = await flushEditorBeforeDiscreteWrite()
  if (flushed !== 'clean') {
    console.error(
      `[editor] KHONG tach Chuong: luot flush tra '${flushed}' — ban dich chua xuong dia`,
    )
    splitChapterError.value = null
    datThongBao({ splitChapter: 'flush-failed' })
    return false
  }

  const { ok, error } = await splitChapterAtSegment(segmentId)
  if (error !== null) {
    console.error(
      `[editor] tach Chuong TRUOT: ${error.code} (${error.message_key}), retryable=${String(error.retryable)}`,
    )
    // ⚠️ Ghi `splitChapterError` TRƯỚC `datThongBao` — cùng thứ tự và cùng lý do
    // `ghiRegroupNotice` đã dùng: `StatusBar.vue` đọc hai ô trong một lượt render, và một
    // `'refused'` không kèm lỗi đọc ra thành câu chung của `'flush-failed'`.
    splitChapterError.value = error
    datThongBao({ splitChapter: 'refused' })
    return false
  }
  // `ok === null` mà `error === null` ⇒ khong co cau IPC (chay ngoai Tauri) -- cung nhanh moi
  // adapter khac, khong mot loi nao de hien.
  if (ok === null) {
    splitChapterError.value = null
    datThongBao({ splitChapter: 'refused' })
    return false
  }

  // Chuong A (dang mo) vua mat bot cau cuoi sang B -- vut state CU roi nap lai NGAY, cung
  // khuon `openChapterById`/`switchChapter`.
  resetEditorPanel()
  resetSourcePanel()
  await ensureSegmentsLoaded()
  await ensureChapterLoaded()
  // 🔴 SAU `resetEditorPanel()`, không trước: chính nó gọi `datThongBao({})` để vứt state của
  // Tác phẩm cũ, nên một câu ghi trước lượt đó sẽ bị dọn ngay và người dùng không thấy gì.
  splitChapterError.value = null
  datThongBao({ splitChapter: 'split' })
  return true
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 2.8 — GỘP và TÁCH tường minh (FR78 · AD-5 · AD-47 ①)
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Kết quả một lượt gộp/tách, ở dạng nơi gọi cần. Mỗi giá trị một việc khác nhau.
 *
 * `'done'` ⇒ đĩa đã đổi và ảnh chụp đã theo kịp. `'no-caret'` ⇒ chưa xác định được câu nào.
 * `'flush-failed'` / `'still-dirty'` ⇒ tập chờ chưa xuống đĩa, **không** ghi đè.
 * `'refused'` ⇒ Rust từ chối; lý do nằm ở [`editorRegroupError`].
 * `'busy'` ⇒ một lượt gộp/tách khác đang bay; lượt này **không** được phát. Xem [`regroupInFlight`].
 */
export type RegroupResultCode =
  | 'done'
  | 'no-caret'
  | 'no-cut'
  | 'flush-failed'
  | 'still-dirty'
  | 'refused'
  | 'busy'

/**
 * 🔴 **Điểm cắt đang có ở CỘT NGUYÊN VĂN** — Quyết định #2 đường (e), Ice ký 2026-08-17.
 *
 * `offset` đếm **ký tự Unicode** trong `source_text` của segment đó.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO MỘT TRẠNG THÁI RIÊNG, KHÔNG ĐỌC `window.getSelection()` LÚC CẦN
 * ─────────────────────────────────────────────────────────────────────────────
 * **Đo 2026-08-17 trên WKWebView 605.1.15 thật** *(bàn đo `2-8-ban-do/`)*: một cú bấm vào ô
 * `[data-col="src"]` cho `selectionType = "None"`, `rangeCount = 0`; một lượt **kéo chọn**
 * cũng vậy, kể cả với sáu bước trung gian và kể cả sau khi tài liệu đã có tiêu điểm. Đối
 * chứng ô bản dịch cho `"Caret"` ⇒ thước tốt. ⇒ **Không có vùng chọn nào để đọc** ở cột
 * nguyên văn; đường (a) của Quyết định #2 mất tiền đề.
 *
 * Cái mà phép đo **cho phép**: `setPosition`/`modify` chạy được ở đó, và `anchorOffset` ánh
 * xạ **thẳng** vào chỉ số ký tự của `source_text` *(mọi text node đứng trước node văn bản
 * cộng lại dài **0** — đo ở bước ⓪ của bàn đo)*. ⇒ Sản phẩm **tự đặt** điểm cắt từ toạ độ cú
 * bấm, đúng khuôn đường chuột mà Story 2.5b đã phải dựng cho ô **bản dịch**.
 *
 * 🔵 **2026-08-17, code review — vế *"chưa đo"* ở trên ĐÃ ĐÓNG.** `sourceCutOffsetOf` nay bỏ
 * qua mọi text node nằm dưới `<rt>` và đếm bằng **code point**, nên phép ánh xạ đúng cả khi
 * Hán Việt bật. Hai ca vitest *(③b, ③d)* khoá cả hai vế và cả hai đã đo đỏ-rồi-xanh.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 2026-08-17 — MỘT ĐIỂM thành MỘT TẬP. Chữ ký của Ice cho AC7 vế *"nhiều mảnh"*
 * ─────────────────────────────────────────────────────────────────────────────
 * `epics.md:2522` đòi tách thành **nhiều** mảnh; bản đầu cài đúng **hai**. Ice ký cơ chế
 * **tích luỹ**: mỗi cú bấm vào cột nguyên văn **thêm** một điểm, `⌘/` cắt tại tất cả cùng một
 * lượt. Đây là một tương tác **chưa tài liệu nào của dự án mô tả** — nó được nêu ra và ký,
 * không suy ra từ một đặc tả có sẵn.
 *
 * 🔴 **Tập luôn thuộc về ĐÚNG MỘT segment.** Bấm sang một câu khác **thay** cả tập, không nối
 * thêm: một tập trải trên hai câu không có nghĩa nào ở tầng dưới *(`split_segment` nhận đúng
 * một `segment_id`)*, và giữ nó lại là mời một lượt tách dùng chỗ cắt của câu khác.
 *
 * 🔴 **Bấm TRÙNG một điểm đã có thì GỠ nó ra** — đó là đường lui duy nhất của người dùng, và
 * nó phải có: `⌘Z` chưa được đặc tả *(Quyết định #9, ghi nợ chủ Story 2.9)*, nên nếu bấm nhầm
 * mà không gỡ được thì đường thoát duy nhất là bấm sang câu khác rồi bấm lại từ đầu.
 */
const sourceCut = ref<{ segmentId: number; offsets: number[] } | null>(null)
/** Xem [`sourceCut`]. `null` ⇒ chưa ai bấm vào cột nguyên văn trong phiên này. */
export const editorSourceCut: DeepReadonly<Ref<{ segmentId: number; offsets: number[] } | null>> =
  readonly(sourceCut)

/**
 * Ghi nhận một điểm cắt ở cột nguyên văn. Gọi từ đường chuột của `GridPanel.vue`.
 *
 * Ba nhánh, và cả ba là một mệnh đề về **ý người dùng**, không về dữ liệu:
 * ① segment khác ⇒ **thay** cả tập; ② điểm đã có ⇒ **gỡ** nó; ③ còn lại ⇒ **thêm**.
 *
 * ⚠️ **Không** kiểm `offset` nằm trong chuỗi ở đây: tầng thuần Rust
 * (`core::segment::regroup::split_at`) đã từ chối tường minh mọi chỗ cắt để lại một mảnh
 * rỗng, và một phép kiểm thứ hai ở webview là một bản sao sẽ lệch — đúng AD-1.
 *
 * ⚠️ **Tập giữ nguyên thứ tự bấm, không sắp ở đây.** `split_at` sắp lại tại chỗ nó dùng và có
 * một ca hợp đồng khoá mệnh đề *"thứ tự bấm không đổi kết quả"*. Sắp ở cả hai chỗ là hai
 * nguồn sự thật cho một luật.
 */
export function setEditorSourceCut(segmentId: number, offset: number): void {
  const dang = sourceCut.value
  if (dang === null || dang.segmentId !== segmentId) {
    sourceCut.value = { segmentId, offsets: [offset] }
    return
  }
  const i = dang.offsets.indexOf(offset)
  const offsets = i === -1 ? [...dang.offsets, offset] : dang.offsets.filter((_, j) => j !== i)
  // Gỡ điểm cuối cùng ⇒ về `null`, không một tập RỖNG. Hai giá trị cho cùng một trạng thái
  // là chỗ một phép kiểm `!== null` sẽ nói dối.
  sourceCut.value = offsets.length === 0 ? null : { segmentId, offsets }
}

/**
 * 🔴 **Xoá TRỌN tập điểm cắt đang chờ** — Story 2.9, AC8 *(Ice yêu cầu 2026-08-17)*.
 *
 * Đường lui duy nhất trước lượt này là *"bấm trùng đúng điểm đã đặt để gỡ nó"*, và
 * doc-comment của [`sourceCut`] đã ghi thẳng rằng nó phải có **vì `⌘Z` chưa được đặc tả**.
 * Với một tập **nhiều** điểm *(chữ ký đa-mảnh của Story 2.8)*, đường lui đó chỉ đúng trên
 * giấy: người dùng phải nhớ mình đã bấm ở đâu, và bấm trượt thì **thêm** một điểm nữa thay vì
 * gỡ một điểm.
 *
 * ⚠️ Về `null`, **không** một tập rỗng — cùng bất biến mà [`setEditorSourceCut`] giữ khi gỡ
 * điểm cuối cùng. Hai giá trị cho cùng một trạng thái là chỗ một phép kiểm `!== null` sẽ nói
 * dối, và cả hai đường phải nói **cùng một** thứ.
 *
 * ⚠️ **Vô hại khi chưa có điểm nào** — nó là một lượt gán, không một thao tác cần đích. Người
 * dùng bấm `Esc` theo phản xạ ở mọi chỗ; một lượt kêu ở đây sẽ là tiếng ồn.
 */
export function clearEditorSourceCut(): void {
  sourceCut.value = null
}

/** Lỗi của lượt gộp/tách gần nhất, hoặc `null`. Tầng UI hiển thị bằng `tError()`. */
const regroupError = ref<IpcError | null>(null)
/** Xem [`regroupError`]. */
export const editorRegroupError: DeepReadonly<Ref<IpcError | null>> = readonly(regroupError)

/**
 * 🔴 **Điều thanh trạng thái phải nói sau lượt gộp/tách gần nhất** — Story 2.9, AC4.
 *
 * Danh mục **ĐÓNG**. `'refused'` là ca duy nhất không mang sẵn câu chữ: lý do nằm ở
 * [`editorRegroupError`] và đi ra màn hình qua `tError()` — Rust là nguồn sự thật cho lý do
 * từ chối, và chép nó sang một bảng ở đây là dựng nguồn sự thật thứ hai.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO MỘT Ô NHỚ THỨ HAI, KHÔNG NỚI `confirmNotice` RA
 * ─────────────────────────────────────────────────────────────────────────────
 * `confirmNotice` đóng trên `ConfirmResult`, và `StatusBar.vue::CONFIRM_NOTICE_KEYS` là một
 * `Record` **đủ khoá** trên kiểu đó — nên thêm một kết quả vào `ConfirmResult` mà quên bảng
 * tra làm `vue-tsc` **đỏ**. Đó là toàn bộ giá trị của nó. Nới nó thành `string` để nhét thêm
 * câu của lượt gộp là gỡ đúng cái chốt ấy, cho **cả hai** lượt.
 * ⇒ Hai ô nhớ, hai bảng tra, **cả hai đóng**. Cái giá là một nhánh `v-else-if` nữa ở template.
 *
 * ⚠️ **Hai ô nhớ mà một chỗ hiện** — thanh cao 34px chỉ chứa một mệnh đề. Thứ tự ưu tiên nằm
 * ở `StatusBar.vue`, không ở đây; chỗ này chỉ ghi *"lượt gộp/tách vừa rồi nói gì"*.
 */
export type RegroupNotice = 'merged' | 'split' | Exclude<RegroupResultCode, 'done'>

const regroupNotice = shallowRef<RegroupNotice | null>(null)
/** Xem [`regroupNotice`]. `StatusBar.vue` đọc. `null` ⇒ không có gì để nói. */
export const editorRegroupNotice: DeepReadonly<Ref<RegroupNotice | null>> = readonly(regroupNotice)

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 2.10 — Ô NHỚ THỨ BA, VÀ CÙNG LƯỢT: MỘT CỬA GHI DUY NHẤT CHO CẢ BA
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * 🔴 **Điều thanh trạng thái phải nói sau lượt ĐIỀU HƯỚNG gần nhất** — Story 2.10, AC6 · AC7.
 * Danh mục **ĐÓNG**, và `StatusBar.vue::NAV_NOTICE_KEYS` là một `Record` đủ khoá trên nó.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * Vì sao một ô nhớ THỨ BA — Quyết định #4 đường (b), Ice ký 2026-08-18
 * ─────────────────────────────────────────────────────────────────────────────
 * Đường rẻ hơn là **nới [`RegroupNotice`]** ra vài giá trị nữa. Nó bị loại vì **sai ngữ
 * nghĩa**: cái tên đó nghĩa là *"gộp/tách"*, và một câu về điều hướng nhét vào đó làm nguồn sự
 * thật lệch tên — người đọc sau sẽ đi tìm lượt gộp nào vừa xảy ra.
 *
 * ⚠️ Và tiền lệ đã từ chối đúng lối tắt ấy **một lần rồi**: khối lý do của [`RegroupNotice`]
 * ngay trên đây giải thích vì sao nó **không** nới `confirmNotice` ra. Đi lại lối đó ở story
 * kế tiếp là bỏ qua một bài học đã trả giá.
 */
export type NavNotice =
  /** AC6 — không còn segment chưa dịch nào **phía dưới**. Con trỏ ở nguyên. */
  | 'no-untranslated'
  /** AC7 — đã ở segment **đầu** Chương, không có câu trước. */
  | 'at-first'
  /** AC7 — đã ở segment **cuối** Chương, không có câu sau. */
  | 'at-last'
  /**
   * 🔵 Quyết định #6 đường (c), Ice ký 2026-08-18 — món nợ `deferred-work.md:2837-2847`.
   *
   * `⌘Enter` ở câu **cuối** Chương ký thành công nhưng **không dời được con trỏ** *(không có
   * câu kế)*, nên `resolveSegmentRule` giữ `primary` thắng `confirmed` và vạch lề **không đổi
   * màu**. `segment.status` trong CSDL thì đúng; chỉ vế thị giác hụt.
   *
   * ⇒ Đóng bằng **thông tin** thay vì bằng màu. 🟡 Đây là **một nửa** món nợ, không phải cả —
   * vạch vẫn `primary`. Phần còn hở đã ghi lại vào sổ nợ kèm chủ mới.
   */
  | 'confirmed-last'
  /**
   * 🔵 **Thêm 2026-08-18 (code review lượt HAI) — Ice ký đường (b) của Quyết định #10.**
   *
   * Ảnh chụp segment **chưa dùng được**: đang chờ IPC, chưa nạp, hoặc lượt nạp đã lỗi
   * *(ba hoàn cảnh của [`editorHasLoaded`])*. Con trỏ ở nguyên.
   *
   * 🔴 **VÌ SAO NÓ PHẢI LÀ MỘT GIÁ TRỊ RIÊNG chứ không dùng lại `'no-untranslated'`** — và
   * đây là một khuyết tật đã **đo được**, không một khả năng lý thuyết.
   * `libraryImport.ts:197` gọi `void ensureSegmentsLoaded()` **không `await`**. Trong cửa sổ
   * đó [`segments`] rỗng, và `editor.next_untranslated` **có** phím mặc định
   * (`Mod+Alt+ArrowDown`, `commands/index.ts:1117`) nên bàn phím **chạm tới được**. Đường cũ
   * cho ra *"Không còn câu nào chưa dịch ở phía dưới"* trên một Chương có thể đang có hàng
   * trăm câu chưa dịch — một câu **khẳng định dứt khoát điều màn hình chưa biết**, đúng lớp
   * lỗi mà doc-comment của [`editorHasLoaded`] (`:86-90`) đã gọi tên từ Story 1.16.
   *
   * ⚠️ **Đường (a) — im lặng — đã bị loại, và nó KHÔNG miễn phí như nó trông:** một phím bấm
   * ra **không một pixel nào đổi** là đúng thứ AC6 tồn tại để chống. Cái giá của đường đã
   * chọn, ghi ra: một khoá `vi.json` và một nhánh cho một cửa sổ chỉ dài bằng một lượt IPC.
   *
   * ⚠️ **GIỚI HẠN THẬT:** [`editorHasLoaded`] gộp cả ca `loadError !== null`, nên một lượt
   * nạp **lỗi** cũng hiện câu *"đang tải"* — hơi lệch. Chấp nhận vì `GridPanel.vue` đã hiện
   * `.load-error` nói đúng nguyên nhân ở chỗ khác, và tách ca đó ra đòi một giá trị **thứ
   * sáu** cho một màn hình vốn đã nói ra sự thật.
   */
  | 'loading'
  /**
   * 🔵 **Thêm 2026-08-18 (Story 2.11 · AC4) — Ice ký Quyết định #5 đường (a).**
   *
   * Đã ở **Chương ĐẦU** của Tác phẩm; lệnh *Chương trước* không đi đâu cả.
   *
   * 🔴 **VÌ SAO KHÔNG TÁI DÙNG `'at-first'`** — và đây là một mệnh đề đo được, không một gu
   * đặt tên. `panel.grid.nav_at_first` đọc nguyên văn *"Đã ở **câu** đầu Chương — không có
   * **câu** nào phía trên"* (`vi.json:107`). Dùng lại nó cho biên **Chương** là để màn hình
   * **nói dối** về đúng thứ người dùng vừa cố làm: họ xin sang một Chương khác, và màn hình
   * trả lời về một câu.
   *
   * ⚠️ Đường bị loại — **một ô nhớ thứ tư** *(Quyết định #5(b))*: Quyết định #4(b) của Story
   * 2.10 đã cân đúng chuyện này và ghi ra, một ô mới làm bất biến *"ai ghi một ô thì dọn ô
   * còn lại"* thành **N chiều**. Thêm một giá trị vào danh mục đóng này thì `datThongBao`
   * dọn nó **theo cấu tạo**, không cần một dòng nào mới.
   */
  | 'at-first-chapter'
  /** AC4, nửa đối xứng — đã ở **Chương CUỐI**. Xem [`'at-first-chapter'`] cho lý do đầy đủ. */
  | 'at-last-chapter'
  /**
   * ═══════════════════════════════════════════════════════════════════════════════
   * 🔵 CODE REVIEW BA TẦNG 2026-08-18 — BA GIÁ TRỊ CHO BA ĐƯỜNG HỎNG CỦA LƯỢT CHUYỂN
   * ═══════════════════════════════════════════════════════════════════════════════
   * Ba nhánh chặn của [`switchChapter`] trước lượt rà này nói **sai chỗ** hoặc **không nói**:
   *
   * - hai nhánh flush mượn kênh `confirm` ⇒ thanh hiện *"…nên **chưa xác nhận**"* cho một
   *   người dùng vừa bấm `Mod+Alt+]` và **không xác nhận gì cả** *(`vi.json:99-100`)*;
   * - nhánh lỗi IPC ghi `loadError` ⇒ lưới bị thay bằng một dòng lỗi, `editorHasLoaded()`
   *   trả `false` **vĩnh viễn** *(không đường nào dọn `loadError` ngoài `resetEditorPanel`)*,
   *   nên mọi lệnh điều hướng sau đó báo *"Chương đang tải"* — sai sự thật — và chính
   *   `switchChapter` **tự khoá mình** ở cửa chặn đầu hàm.
   *
   * 🔴 Đây đúng nguyên tắc mà Quyết định #5 của Story 2.11 đã đặt cho `'at-first-chapter'`
   * cách đó bốn dòng — *"tái dùng một câu của CÂU cho biên CHƯƠNG là để màn hình nói dối"* —
   * áp cho `NavNotice` nhưng bỏ sót `ConfirmNotice` và bỏ sót nhánh lỗi. Ice ký đường (a):
   * một kênh **riêng**, `loadError` **không bị đụng**, lưới Chương hiện tại **ở nguyên** và
   * người dùng **thử lại được**. Đúng luật đã ghi ở [`ensureSegmentsLoaded`] *(`:133-136`)*:
   * *"một lượt TRƯỢT không được khoá vĩnh viễn đường nạp"*.
   *
   * ⚠️ **GIỚI HẠN THẬT:** ba câu này là chuỗi **tĩnh** — chúng không chở được `code` hay
   * `params` của `IpcError`. Chi tiết ấy đi vào `console.error` cho người gỡ lỗi, không lên
   * màn hình. Đường đưa nó lên màn hình là **một ô nhớ thứ tư**, thứ Quyết định #4(b) của
   * Story 2.10 đã cân và loại; đảo lại quyết định đó là một lượt riêng, không một lượt vá.
   */
  | 'chapter-switch-failed'
  /** Lượt flush trước khi chuyển **trượt** — bản dịch chưa xuống đĩa nên lượt chuyển bị chặn. */
  | 'chapter-flush-failed'
  /** Văn bản đổi **trong lúc** lô đang bay ⇒ tập chờ còn dơ sau hai lượt flush. */
  | 'chapter-still-dirty'

const navNotice = shallowRef<NavNotice | null>(null)
/** Xem [`navNotice`]. `StatusBar.vue` đọc. `null` ⇒ không có gì để nói. */
export const editorNavNotice: DeepReadonly<Ref<NavNotice | null>> = readonly(navNotice)

/**
 * Kết cục của lượt **tách CHƯƠNG** gần nhất — Story 5.8, ô nhớ câu chữ **thứ TƯ**, và
 * doc-comment của [`datThongBao`] đã hẹn trước đúng lượt thêm này (*"thêm một ô nhớ thứ tư thì
 * thêm một trường vào đây và một dòng gán"*).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO Ô NHỚ NÀY PHẢI TỒN TẠI — MỘT PHÉP ĐO, KHÔNG MỘT LO XA
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ **Đo 2026-08-29 (lượt rà):** bản đầu của [`splitChapterHere`] báo MỌI đường trượt bằng
 * `console.error` và **chỉ thế** — caret trống, flush chưa sạch, và cả `err.chapter.split_leaves_empty`
 * (ca THƯỜNG NHẤT: gõ hợp âm khi caret đang ở câu đầu Chương). Người dùng gõ `Mod+Shift+Slash`
 * và màn hình **không đổi một pixel nào**. Đúng lớp *"rỗng IM LẶNG"* mà `AGENTS.md` đặt lên
 * hàng đầu — và nó lệch hẳn với hai lệnh ANH EM của nó: `editor.merge_segments` và
 * `editor.split_segment` đều hiện câu từ chối trên `StatusBar` qua `regroupNotice`/`regroupError`.
 *
 * ⚠️ **Không tái dùng `RegroupNotice`.** `RegroupNotice` suy ra từ `RegroupResultCode` của lượt
 * gộp/tách **segment** (AD-5: về hưu + tạo mới); tách CHƯƠNG là AD-32 (thao tác tổ chức, không
 * đụng văn bản segment nào). Nhồi nó vào cùng một union là gộp hai khái niệm mà `epics.md` gọi
 * là *"khác biệt cố ý"*, và bảng khoá của `StatusBar` sẽ nói sai một trong hai.
 *
 * Danh mục ĐÓNG bốn giá trị ⇒ bảng `Record` ở `StatusBar.vue` đỏ ở `vue-tsc` nếu ai thêm một
 * giá trị thứ năm mà quên khoá chuỗi — cùng cơ chế ba bảng kia.
 */
export type SplitChapterNotice = 'split' | 'no-caret' | 'flush-failed' | 'refused'
const splitChapterNotice = shallowRef<SplitChapterNotice | null>(null)
/** Xem [`splitChapterNotice`]. `StatusBar.vue` đọc. */
export const editorSplitChapterNotice: DeepReadonly<Ref<SplitChapterNotice | null>> =
  readonly(splitChapterNotice)
const splitChapterError = shallowRef<IpcError | null>(null)
/** `IpcError` của nhánh `'refused'` — `StatusBar.vue` đọc nó qua `tError()`. */
export const editorSplitChapterError: DeepReadonly<Ref<IpcError | null>> = readonly(splitChapterError)

/**
 * 🔴 **CỬA GHI DUY NHẤT CHO CẢ BA Ô NHỚ CỦA THANH TRẠNG THÁI.** Story 2.10, Quyết định #4(b).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO MỘT HÀM CHỨ KHÔNG BA LỜI GỌI — và vì sao nó ghi CẢ BA, mỗi lượt
 * ─────────────────────────────────────────────────────────────────────────────
 * Bất biến là *"thao tác vừa xảy ra sở hữu thanh trạng thái: ai ghi một ô thì dọn các ô còn
 * lại"*. Với **hai** ô, [`ghiRegroupNotice`] cài nó bằng hai dòng và nó đứng. Với **ba** ô,
 * cùng cách viết cần **sáu** dòng rải ở ba chỗ — và một chỗ quên một dòng là bất biến hở, im
 * lặng, không cổng nào đỏ. Chi phí ấy đã được nêu **trước** khi ký, không sau.
 *
 * 🔴 **Chốt nằm ở chữ ký hàm, không ở kỷ luật người viết:** hàm này gán **cả ba** ref ở **mọi**
 * lời gọi. Một tham số **vắng mặt NGHĨA LÀ `null`** — có chủ ý, và đó là lý do `?? null` ở đây
 * không phải một lượt cẩu thả. ⇒ *Không tồn tại cú pháp* để ghi một ô mà không dọn hai ô kia.
 * Bất biến trở thành một mệnh đề về **cấu tạo**, không một mệnh đề về thói quen.
 *
 * ⚠️ **Hệ quả cho lượt sửa kế tiếp:** thêm một ô nhớ thứ tư thì thêm một trường vào đây và
 * **một dòng gán** — không phải đi tìm ba chỗ gọi. Và 🔴 nhớ [`resetEditorPanel`]: luật *"ô nhớ
 * thuộc Tác phẩm phải có mặt ở đó"* **không có cổng nào canh** và đã bị bỏ sót **hai story liên
 * tiếp** (`sourceCut` ở 2.8 · `regroupNotice`/`regroupError` ở 2.9).
 *
 * 🔴 **Không gán trần `confirmNotice.value` / `regroupNotice.value` / `navNotice.value` ở đâu
 * nữa** — trừ [`resetEditorPanel`], nơi lượt gán là *"vứt state của Tác phẩm cũ"* chứ không
 * phải *"thao tác này sở hữu thanh"*, và nó đi qua hàm này với **tham số rỗng**.
 */
function datThongBao(o: {
  confirm?: Exclude<ConfirmResult, 'confirmed' | 'refused'> | null
  regroup?: RegroupNotice | null
  nav?: NavNotice | null
  // 🔵 Ô thứ TƯ (Story 5.8) — thêm ĐÚNG như doc-comment trên đã hẹn: một trường ở đây, một
  // dòng gán bên dưới, và mọi chỗ gọi cũ dọn nó mà không phải sửa một dòng nào.
  splitChapter?: SplitChapterNotice | null
}): void {
  confirmNotice.value = o.confirm ?? null
  regroupNotice.value = o.regroup ?? null
  navNotice.value = o.nav ?? null
  splitChapterNotice.value = o.splitChapter ?? null
}

/**
 * Ghi câu của lượt **điều hướng** vừa rồi, và dọn hai ô kia. Vỏ mỏng trên [`datThongBao`].
 *
 * ⚠️ Tồn tại để chỗ gọi đọc ra **ý định** chứ không đọc ra một object literal — cùng vai và
 * cùng khuôn [`ghiRegroupNotice`].
 */
function ghiNavNotice(notice: NavNotice): void {
  datThongBao({ nav: notice })
}

/**
 * 🔴 **Ghi câu của lượt gộp/tách, và DỌN câu của lượt xác nhận cùng lúc.**
 *
 * 🔵 **Thêm 2026-08-17 (code review ba tầng — HAI tầng độc lập tái hiện bằng vitest).**
 * `StatusBar.vue` khai `v-if="confirmNoticeKey !== null"` **trước** `v-else-if` của câu gộp, và
 * chú thích tại chỗ biện minh thứ tự đó bằng một mệnh đề: *"hai ô nhớ không bao giờ cùng mang
 * giá trị trong một lượt dùng thật, cả hai được dọn ở `noteEditorEdit`"*. **Mệnh đề ấy SAI**, và
 * đo được: `confirmNotice` chỉ bị dọn ở **ba** cửa *(`noteEditorEdit` · lượt đổi Tác phẩm · chính
 * `confirmCurrentSegment` khi kết quả là `'confirmed'`/`'refused'`)*, và `regroup()` **không** là
 * một trong ba. Ba giá trị `'no-caret'`/`'flush-failed'`/`'still-dirty'` ở lại **vô thời hạn**.
 *
 * Đường hỏng, từng bước:
 *   ① `⌘Enter` lúc chưa có caret ⇒ `confirmNotice = 'no-caret'`, thanh hiện *"chưa chọn câu nào"*.
 *   ② Người dùng **không gõ chữ nào** *(nên `noteEditorEdit` không chạy)*, click vào đầu ô một câu
 *      khác rồi bấm `Backspace`.
 *   ③ Cử chỉ `Backspace` **không** đi qua `noteEditorEdit`: `preventDefault()` cắt hẳn chuỗi
 *      `beforeinput`→`input`→`reportEdit`.
 *   ④ Gộp **THÀNH CÔNG THẬT** ⇒ `regroupNotice = 'merged'`, nhưng `v-if` phía trên thắng vô điều
 *      kiện ⇒ thanh **vẫn hiện câu ở bước ①**.
 *   ⇒ AC4 không đạt, và đúng lớp *"rỗng IM LẶNG"* mà cả story này tồn tại để đóng — chỉ khác là
 *     nó xảy ra ở **thành công**, không ở lượt từ chối.
 *
 * ⚠️ Chiều ngược lại cũng hỏng: một `regroupNotice` cũ *(không được dọn khi `⌘Enter` chạy)* che
 * mốc *"Đã lưu N giây trước"* mà UX-DR30 đòi.
 *
 * 🔴 **Vá bằng một BẤT BIẾN, không bằng một lượt sửa thứ tự `v-if`.** Đổi thứ tự chỉ dời chỗ nói
 * dối sang ô nhớ kia. Luật đúng là *"thao tác vừa xảy ra sở hữu thanh trạng thái"*: ai ghi một ô
 * thì dọn ô còn lại. Khi bất biến này đứng, thứ tự `v-if`/`v-else-if` ở `StatusBar.vue` trở thành
 * **không quan sát được** — và mệnh đề trong chú thích ở đó thành đúng theo **cấu tạo**.
 *
 * ⇒ Mọi lượt ghi `regroupNotice` phải đi qua hàm này. Không gán trần `regroupNotice.value` ở đâu
 *   nữa — một lượt gán trần là đúng chỗ bất biến này sẽ hở lần thứ hai.
 */
function ghiRegroupNotice(notice: RegroupNotice): void {
  // 🔵 Story 2.10 — thân hàm chuyển sang [`datThongBao`]. Hành vi **không đổi một bước nào**
  // cho hai ô cũ; cái đổi là vế thứ ba *(dọn câu điều hướng)* nay có mặt mà không ai phải nhớ.
  datThongBao({ regroup: notice })
}

/**
 * 🔴 **Vá ảnh chụp sau một lượt gộp/tách** — và đây là chỗ AD-47 ① được thi hành ở webview.
 *
 * AD-47 ① đòi mỗi lượt ghi không-phải-người-dùng đặt lại **mốc so sánh** của segment về đúng
 * văn bản vừa ghi. Mốc **không sống trên đĩa**: Quyết định #2(b) của Story 2.7 đặt nó ở
 * [`segments`]. Chèn nguyên hàng Rust vừa trả về **là** lượt đặt mốc đó — không một bước thứ
 * hai, và vì thế không một nửa nào rơi được.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 2026-08-17 — HÀNG VỀ HƯU **BỊ GỠ** KHỎI ẢNH CHỤP. Chữ ký #6(b) đã bị LẬT
 * ─────────────────────────────────────────────────────────────────────────────
 * Bản đầu **giữ** hàng về hưu trong mảng, mang `retired_at` khác `null` để
 * `resolveSegmentRule` vẽ vạch `ornament` mờ — chữ ký #6(b) của Ice, ký cùng ngày.
 *
 * **Ice lật nó sau khi dùng thật:** *"đã tách ra 2 câu, nhưng câu cũ vẫn tồn tại và số thứ tự
 * vẫn chiếm, gây rối nội dung"*. Đó là một phép đo mà không bảng đường nào thay được — cái
 * giá *"lưới phình theo số lần sửa"* đã được viết ra và nhận trước khi ký, nhưng nó chỉ đọc
 * được thành *"gây rối"* khi có người thật nhìn vào một Chương thật.
 *
 * 🔴 **GỠ KHỎI LƯỚI, KHÔNG XOÁ KHỎI ĐĨA.** Hàng vẫn nằm trong `project.db` với `retired_at`
 * khác `NULL`; `read_segment_history` **không** hỏi cột đó, nên AC4 *(lịch sử phiên bản của
 * một segment đã về hưu vẫn tra lại được)* còn nguyên. Xoá hàng là mất lịch sử **vĩnh viễn**,
 * đúng thứ AD-5 tồn tại để chống.
 *
 * ⚠️ **Số thứ tự tự sửa theo, không cần một dòng nào:** lưới đánh số bằng **chỉ số mảng**
 * (`GridPanel.vue`, `{{ i + 1 }}`), nên một hàng không có trong mảng thì không chiếm số.
 */
function applyRegroup(outcome: RegroupOutcome): void {
  const veHuu = new Set(outcome.retired.map((row) => row.id))
  const next: ChapterSegment[] = []
  let inserted = false
  for (const row of segments.value) {
    if (!veHuu.has(row.id)) {
      next.push(row)
      continue
    }
    // 🔴 Hàng mới thế chỗ nhóm vừa về hưu, tại **đúng vị trí** nhóm đó đang đứng — và đúng
    // MỘT lần cho cả nhóm. Nối vào cuối mảng thay vì thế chỗ sẽ làm thứ tự đọc của lưới
    // khác thứ tự trên đĩa, im lặng: `ord` phía Rust đã đánh lại 1..N theo vị trí cũ.
    if (!inserted) {
      next.push(...outcome.new_segments)
      inserted = true
    }
  }
  // Ảnh chụp không có hàng nào của nhóm ⇒ nối vào cuối thay vì đánh rơi. Ca này không tới
  // được hôm nay *(caret sinh từ chính mảng này)*, và nó viết ra vì cái giá hai bên lệch xa:
  // đánh rơi một hàng mới là để đĩa và màn hình nói hai điều khác nhau, im lặng.
  //
  // 🔵 **NỐI LẠI 2026-08-25 — dòng này ĐÃ BỊ XOÁ mà đoạn chú thích trên thì còn nguyên.**
  // Lượt Story 3.4b thay đúng dòng `if (!inserted) next.push(...)` (`a2eaf7c~1`, dòng 1973)
  // bằng hai lời gọi `reset*` ngay dưới và **không nối lại**, nên từ đó tới nay đoạn văn trên
  // treo lơ lửng trên một chỗ trống: nó tả một vệ không còn tồn tại. Bắt ở vòng rà Epic 3
  // (2026-08-25, lăng kính adversarial), đối chứng bằng
  // `git show a2eaf7c~1:src/panels/editorPanelState.ts | sed -n '1973p'`.
  // ⚠️ Đây là lớp lỗi *"khôi phục trung thành ≠ đúng"* đọc ngược: một lượt sửa máy móc giữ
  // nguyên LỜI KHAI và đánh rơi THỨ NÓ KHAI. Chú thích không phải cổng — nên vệ này nay có
  // một ca canh riêng ở `tests/frontend/editorRegroupGuards.test.ts`.
  if (!inserted) next.push(...outcome.new_segments)
  // 🔴 **DỌN DẤU ĐỒNG BỘ, TRƯỚC khi `segments.value` đổi — bắt ở lượt rà 2026-08-21 (ba lớp
  // review).** `refreshGlossaryMarks()` ngay dưới là **BẤT ĐỒNG BỘ** (một round-trip IPC), còn
  // `segments.value = next` thì đồng bộ ngay tại đây. Giữa hai thời điểm đó — từ lúc hàm này
  // trả về tới lúc `refreshGlossaryMarks` đáp lời — Vue render lại lưới với `editorSegments`
  // MỚI trong khi `glossaryMarks` (tầng TUYỆT ĐỐI, tính theo `\n`-join của segment CŨ) vẫn còn
  // nguyên: `glossaryMarksBySegment` (GridPanel.vue) sẽ chia những offset đó lên bố cục MỚI —
  // đúng hàng I/O Matrix *"không dấu nào trỏ vào segment đã về hưu"* cấm. Xoá TRƯỚC khi đổi
  // `segments.value` đóng cửa sổ đó hoàn toàn: không một lượt render nào thấy được cặp
  // (segment mới, mark cũ) lẫn nhau.
  resetGlossaryMarks()
  // 🔵 Story 3.6 — cùng lý do: một dải đang hỏi về một segment vừa về hưu (gộp/tách) không
  // được sống sót qua lượt đổi bố cục này.
  resetGlossaryConfirmStrip()
  segments.value = next

  // 🔵 Story 3.4b — gộp/tách đổi RANH GIỚI segment, tức đổi phép cộng dồn `\n`-join mà
  // `glossaryMarksMap.ts` dùng để chia mark tuyệt đối về từng segment (chuỗi Chương THÔ không
  // đổi, nhưng chuỗi GỬI CHO Rust — nối từ `segments.value` — thì có). `ensureGlossaryMarksLoaded`
  // sẽ coi Chương này là "đã nạp rồi" và bỏ qua *(🔵 hết đúng theo hình dạng cũ: `resetGlossaryMarks()`
  // ngay trên vừa nhả `requestedForChapterId` về `null`, nên `ensureGlossaryMarksLoaded` giờ
  // KHÔNG còn coi Chương này là "đã nạp" — nhưng lời gọi ở đây vẫn dùng `refreshGlossaryMarks`
  // tường minh, không dựa vào hiệu ứng phụ đó, để ý định đọc được tại chỗ)* — một trong hai
  // lượt IPC PHỤ mà `3-4b-…md` §Intent cho phép, ngoài đúng-một-lượt-mỗi-lần-mở.
  //
  // 🔵 **THÊM VẾ THỨ BA 2026-08-25 — `chapterId.value === sourceChapter.value.chapter_id`.**
  // Chỗ gọi anh em ở [`switchChapter`] (`:1577-1581`) đã mang vế này từ lượt rà 2026-08-21
  // kèm một câu viết thẳng: *"BẮT BUỘC, không phải một hàng rào thừa"*. Chỗ gọi ở ĐÂY thì chỉ
  // kiểm `!== null`, tức hai chỗ gọi giữ HAI điều kiện khác nhau cho cùng một cặp ref — đúng
  // thứ mà chú thích ở `:1577` cấm bằng chữ (*"Hai chỗ gọi phải giữ ĐÚNG một điều kiện, không
  // phải hai biến thể của cùng một ý"*). Bắt ở vòng rà Epic 3 (2026-08-25).
  // ⚠️ Cửa sổ lệch tới được từ ĐÂY chứ không chỉ từ `switchChapter`: `regroup` là `async` và
  // `applyRegroup` chạy sau `await`, nên một lượt chuyển Chương bay giữa chừng để `chapterId`
  // mang Chương B trong khi `sourceChapter` còn mang Chương A. Nạp dấu SAI Chương rồi gán cho
  // Chương đang hiện là đúng khuyết tật mà watcher của `GridPanel.vue` đã tồn tại để chặn.
  if (
    chapterId.value !== null &&
    sourceChapter.value !== null &&
    chapterId.value === sourceChapter.value.chapter_id
  ) {
    void refreshGlossaryMarks(chapterId.value, next, sourceChapter.value.source_lang)
  }
}

/**
 * Ba nhịp chung của **mọi** lượt gộp/tách: flush tập chờ, gọi Rust, vá ảnh chụp.
 *
 * ⚠️ Rút ra vì **hai** lệnh dùng chung, không vì nó dài — cùng lý do
 * [`flushEditorBeforeDiscreteWrite`] đã phải rút ra ở code review 2026-08-16: *"một khuôn
 * được chép là một khuôn sẽ chép thiếu"*, và lượt chép ấy đã xảy ra thật một lần.
 */
async function regroup(
  id: number,
  goi: () => Promise<{ outcome: RegroupOutcome | null; error: IpcError | null }>,
  ten: 'gop' | 'tach',
): Promise<RegroupResultCode> {
  // ─────────────────────────────────────────────────────────────────────────────
  // 🔴 ⓿ KHOÁ CHỐNG-GỌI-LẠI — code review 2026-08-17
  // ─────────────────────────────────────────────────────────────────────────────
  // Cùng lớp đường hỏng mà [`confirmCurrentSegment`] đã phải khoá ở code review 2026-08-14, và
  // Story 2.9 làm nó **dễ chạm hơn hẳn**: `⌘M` đòi một hợp âm hai phím, còn `Backspace` là phím
  // người dùng bấm hàng trăm lần mỗi Chương.
  //
  // `event.repeat` ở `GridPanel.vue` chặn auto-repeat của **hệ điều hành** (giữ phím) — nó
  // **không** chặn hai cú bấm **RỜI RẠC** nhanh, tức đúng thói quen *"bấm lại cho chắc"* khi
  // không có chỉ báo đang xử lý. Và vì nhánh đó gọi `preventDefault()`, DOM cùng caret của ô
  // **y nguyên ở offset 0** sau cú bấm đầu, nên cú thứ hai vẫn qua trọn `caretAtCellStart` và
  // dispatch lại cho **cùng** một `id`.
  //
  // Hậu quả đo được ở lớp thông điệp: lượt IPC thứ hai trả `'refused'` *(segment đã về hưu bởi
  // lượt đầu)* và **ghi đè** `regroupNotice` từ `'merged'` thành `'refused'` ⇒ thanh trạng thái
  // báo *"chưa gộp được"* cho một thao tác **đã gộp xong**. Nói dối đúng chiều nguy hiểm, trên
  // một lượt ghi mà **AD-5 không cho hoàn tác** và `⌘Z` (AC5) còn là món nợ chờ `AD-48`.
  //
  // 🔴 **TỪ CHỐI và KÊU, không NHẬP vào lượt đang bay** — khác `confirmInFlight` một cách có chủ
  // ý, và đây là chỗ hai khoá tách nhau. Lượt xác nhận là **một** thao tác trên **một** câu, nên
  // nhập vào là đúng. Còn `regroup` là cửa chung của **hai** lệnh khác nhau: một `⌘M` rồi một
  // `⌘/` bấm sát nhau là hai thao tác **khác nhau** trên dữ liệu khác nhau, và cho lượt sau nhận
  // kết quả của lượt trước là **đánh rơi một thao tác người dùng trong im lặng** — đúng lớp mà
  // `project-context.md` cấm, chỉ đổi chỗ chứ không mất đi.
  // ⇒ `'busy'` là một câu trả lời **hợp lệ** và nó **nói ra**: bảng tra đóng ở `StatusBar.vue`
  //   buộc nó có câu chữ, `vue-tsc` không cho bỏ sót.
  if (regroupInFlight !== null) {
    ghiRegroupNotice('busy')
    return 'busy'
  }
  const run = regroupUnguarded(id, goi, ten)
  regroupInFlight = run
  try {
    return await run
  } finally {
    regroupInFlight = null
  }
}

/** Lượt gộp/tách đang bay, hoặc `null`. Xem khối lý do ⓿ trong [`regroup`]. */
let regroupInFlight: Promise<RegroupResultCode> | null = null

async function regroupUnguarded(
  id: number,
  goi: () => Promise<{ outcome: RegroupOutcome | null; error: IpcError | null }>,
  ten: 'gop' | 'tach',
): Promise<RegroupResultCode> {
  // ① AD-35 vế (c) — bản dịch đi vào hàng mới đọc từ **ĐĨA**, nên mọi ký tự đang chờ phải
  //    xuống WAL trước. Cùng cửa với lượt ký và lượt khôi phục.
  const flushed = await flushEditorBeforeDiscreteWrite()
  if (flushed === 'failed') {
    ghiRegroupNotice('flush-failed')
    return 'flush-failed'
  }
  if (flushed === 'still-dirty') {
    // 🔴 *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."*
    console.error(
      `[editor] KHONG ${ten} segment ${id}: tap cho van do sau hai luot flush — ` +
        `tu choi thay vi dung mot hang moi tu mot van ban cu hon thu dang tren man hinh.`,
    )
    ghiRegroupNotice('still-dirty')
    return 'still-dirty'
  }

  const { outcome, error } = await goi()
  if (outcome === null) {
    // ⚠️ `error === null` cũng vào đây: ca *"không có cầu IPC"*. Không ca nào coi là đã xong.
    regroupError.value = error
    ghiRegroupNotice('refused')
    return 'refused'
  }
  regroupError.value = null
  // 🔴 Story 2.9, AC4 — ô nhớ dòng báo đặt **cùng chỗ** `regroupError` được dọn, không ở một
  // nhánh thứ hai. Hai ô nhớ cho cùng một sự kiện mà đặt ở hai chỗ là chỗ một lượt sửa sau
  // sẽ dọn một cái và quên cái kia — và thanh trạng thái sẽ nói dối **về thao tác vừa xảy
  // ra**, thứ khó phát hiện nhất vì nó luôn hiện *một* câu đúng ngữ pháp.
  ghiRegroupNotice(ten === 'gop' ? 'merged' : 'split')

  // ② Vá ảnh chụp — và đó là lượt đặt mốc AD-47 ①. Xem [`applyRegroup`].
  applyRegroup(outcome)

  // ③ Con trỏ về **hàng mới đầu tiên**. Không để nó trỏ một `id` vừa về hưu: bốn lệnh ghi
  //    hiện có đều từ chối một segment về hưu, nên caret ở đó là một caret mà mọi phím tiếp
  //    theo sẽ từ chối — im lặng với người dùng, và họ không biết vì sao.
  // 🔴 `.at(0)`, KHÔNG `[0]` — và đó là *"sửa KIỂU cho nó nói thật"*, không một lượt đổi cú
  //    pháp cho đẹp. `[0]` khai kiểu `ChapterSegment` **đặc**, nên một phép kiểm `undefined`
  //    cạnh nó là *"thừa"* theo trình biên dịch và `@typescript-eslint/no-unnecessary-condition`
  //    **đỏ** — trong khi mảng này đến **qua dây IPC**, tức một **lời khai**, không một bảo đảm
  //    của trình biên dịch. Đường sai rẻ ở đây là một `eslint-disable`; nó cho exit 0 trên đúng
  //    chỗ mà một payload rỗng sẽ ném `undefined.id` lúc chạy. `.at()` trả `T | undefined`
  //    **theo cấu tạo**, nên kiểu nói đúng sự thật và không cần một miễn trừ nào.
  const dich = outcome.new_segments.at(0)
  if (dich !== undefined) {
    setEditorCaret(dich.id)
    caretPlacement.value = dich.id
  }
  // ④ Điểm cắt vừa tiêu thụ **phải chết**. Nó trỏ một `segment.id` mà lượt này vừa cho về
  //    hưu; giữ lại là để lượt `⌘/` kế tiếp chạy trên một câu đã không còn — và Rust sẽ từ
  //    chối bằng `segment.retired`, một câu **đúng chữ nhưng sai nguyên nhân** với người dùng.
  //
  // 🔵 **2026-08-17, code review — CÓ ĐIỀU KIỆN, không vô điều kiện.** Bản đầu xoá sau **mọi**
  //    lượt gộp hoặc tách, và câu biện minh ở trên chỉ đúng ở **đường tách**. Ở đường gộp,
  //    điểm cắt có thể trỏ một câu **không nằm trong nhóm vừa về hưu**: bấm cột nguyên văn ở
  //    câu 5 để sắp tách, rồi `⌘M` gộp câu 1+2 — câu 5 nguyên vẹn nhưng cả tập điểm cắt bay.
  //    `⌘/` kế tiếp trả `'no-cut'`, mà `'no-cut'` chỉ đi ra `console.warn` ⇒ người dùng không
  //    thấy gì và phải tự đoán vì sao thao tác không chạy. Đúng lớp *"im lặng"* mà
  //    `project-context.md` cấm.
  const cut = sourceCut.value
  if (cut !== null && outcome.retired.some((r) => r.id === cut.segmentId)) sourceCut.value = null
  return 'done'
}

/**
 * **Gộp câu đang có caret với câu liền trên nó** — Story 2.8, AC1.
 *
 * 🔴 *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."* Câu đầu Chương là một
 * câu trả lời **hợp lệ** *(`segment.no_previous`)*, không một lỗi lập trình.
 */
export async function mergeCurrentSegment(): Promise<RegroupResultCode> {
  const id = caretSegmentId.value
  // 🔴 Story 2.9, AC4 — lượt trả sớm này KHÔNG đi qua `regroup()`, nên nó phải tự đặt ô nhớ.
  // Trước bản vá, đây là ca **im lặng nhất** của cả đường: không IPC nào được phát ⇒ không lỗi
  // nào tồn tại để hiển thị ⇒ màn hình không đổi một pixel. Cùng khuôn `confirmCurrentSegment`
  // đã phải vá ở code review 2026-08-15.
  if (id === null) {
    ghiRegroupNotice('no-caret')
    return 'no-caret'
  }
  return await regroup(id, () => mergeSegments(id), 'gop')
}

/**
 * **Tách câu tại điểm cắt đang có ở CỘT NGUYÊN VĂN** — Story 2.8, AC2.
 *
 * 🔴 **Đích là `sourceCut.segmentId`, KHÔNG `caretSegmentId`** — và đó là ngữ nghĩa của
 * `epics.md:2552`: *"không có phép chiếu nào từ vị trí con trỏ bên tiếng Việt sang chỗ cắt
 * bên tiếng Trung. Cùng lý do Trados và memoQ đều bắt tách ở cột nguồn"*. Người dùng bấm vào
 * cột nguyên văn của **hàng X** rồi gõ `⌘/`; caret gõ chữ có thể đang ở một hàng khác, và
 * lấy nó làm đích là tách nhầm câu.
 *
 * ⇒ Chưa bấm vào cột nguyên văn ⇒ `'no-cut'`, một câu trả lời **hợp lệ**.
 */
export async function splitCurrentSegment(): Promise<RegroupResultCode> {
  const cut = sourceCut.value
  // Cùng lý do lượt trả sớm của `mergeCurrentSegment` phải đặt ô nhớ — xem chú thích ở đó.
  if (cut === null) {
    ghiRegroupNotice('no-cut')
    return 'no-cut'
  }
  // 🔵 2026-08-17 — gửi **cả tập**, một lượt. Xem [`sourceCut`] và `regroup.rs::split_at`:
  // gọi `⌘/` `n` lần để có `n + 1` mảnh là đường (c) đã bị bác bằng số đo *(nó cho 5 hàng về
  // hưu thay vì 3, cộng một segment trung gian mang một `id` không ai từng thấy)*.
  const offsets = [...cut.offsets]
  return await regroup(cut.segmentId, () => splitSegment(cut.segmentId, offsets), 'tach')
}
