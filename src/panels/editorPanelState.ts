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
import { readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import {
  confirmSegment,
  mergeSegments,
  readOpenChapterSegments,
  saveSegmentTargets,
  setSegmentOmitted,
  setSegmentParagraphEnd,
  splitSegment,
} from '../config/segment'
import type { ChapterSegment, RegroupOutcome, SegmentTargetEdit } from '../config/segment'
import type { IpcError } from '../i18n'
import { createEditorFlush, EDITOR_RETRY_FLOOR_MS } from './editorFlush'
import { navigationSegmentOf, nextUntranslatedId } from './segmentNavigation'

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
 * nạp, đang chờ IPC, và *"Chương này thật sự chưa có segment nào"* *(25 Chương của Epic 1,
 * `deferred-work.md:542`)*. Chỉ hoàn cảnh thứ ba được phép hiện câu *"chưa tách câu nào"*;
 * hai hoàn cảnh kia mà hiện câu đó là màn hình khẳng định dứt khoát một điều nó chưa biết —
 * đúng lỗ mà `hanVietPending` của Story 1.16 tồn tại để bịt.
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
  confirmNotice.value = null
  // 🔵 Story 2.9, AC4 — câu báo của lượt gộp/tách đi cùng cửa và cùng lý do. Cả bảy giá trị
  // của `RegroupNotice` mô tả một thời điểm **đã trôi qua**; giữ chúng lại trong lúc người
  // dùng đang gõ là để một câu ĐÚNG-LÚC-ĐÓ nói dối ở hiện tại.
  regroupNotice.value = null
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
  // Cùng lý do: một câu *"chưa lưu được bản dịch"* thuộc Tác phẩm vừa bị thay.
  confirmNotice.value = null

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
  regroupNotice.value = null
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
    confirmNotice.value = result === 'confirmed' || result === 'refused' ? null : result
    // 🔵 2026-08-17 (code review) — vế ĐỐI XỨNG của bất biến ở [`ghiRegroupNotice`]: thao tác vừa
    // xảy ra sở hữu thanh trạng thái. Không có dòng này, một câu *"Đã gộp hai câu…"* của lượt
    // trước sống sót qua một lượt `⌘Enter` **thành công** và che mốc *"Đã lưu N giây trước"* —
    // cùng một khuyết tật, chỉ đổi chiều.
    regroupNotice.value = null
    return result
  } finally {
    confirmInFlight = null
  }
}

/** Lượt xác nhận đang bay, hoặc `null`. Xem khối lý do trong [`confirmCurrentSegment`]. */
let confirmInFlight: Promise<ConfirmResult> | null = null

async function confirmCurrentSegmentUnguarded(): Promise<ConfirmResult> {
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

  const index = segments.value.findIndex((s) => s.id === id)
  if (index >= 0) {
    const next = [...segments.value]
    next[index] = { ...next[index], is_target_paragraph_end: outcome.is_target_paragraph_end }
    segments.value = next
  }

  return outcome.is_target_paragraph_end ? 'ended' : 'joined'
}

/**
 * **Nhảy tới câu chưa dịch kế tiếp** — Story 2.5b, AC12 · `⌥↓`.
 *
 * ⇒ Trả `true` khi con trỏ **đã dời**, `false` khi không còn câu nào.
 *
 * 🔴 *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."* Hết Chương là một câu
 * trả lời **hợp lệ**, không một lỗi: nó nghĩa là **không còn câu nào chưa dịch ở phía dưới**.
 * Con trỏ **ở nguyên**, và chỗ gọi ghi một dòng chẩn đoán.
 *
 * ⚠️ Dùng lại **đúng** đường dời con trỏ mà Quyết định #1 của Story 2.5 đã dựng
 * (`setEditorCaret` + [`caretPlacement`]), không một đường thứ hai. Đó cũng là lý do một lượt
 * nhảy **flush câu vừa rời**: `setEditorCaret` mang vế *"rời segment"* của AD-35 vế (c).
 *
 * 🔴 Phép chọn sống ở `./segmentNavigation.ts` — một **module thuần** kiểm được bằng Node
 * trần. Chỗ này chỉ **xếp thứ tự** hai lượt gọi.
 */
export function goToNextUntranslated(): boolean {
  const list = segments.value.map((s) => navigationSegmentOf(s, editedText.value))
  const next = nextUntranslatedId(list, caretSegmentId.value)
  if (next === null) return false
  setEditorCaret(next)
  caretPlacement.value = next
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
  regroupNotice.value = notice
  confirmNotice.value = null
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
  if (!inserted) next.push(...outcome.new_segments)
  segments.value = next
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
