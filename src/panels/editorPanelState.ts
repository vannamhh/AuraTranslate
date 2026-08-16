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
  readOpenChapterSegments,
  saveSegmentTargets,
  setSegmentOmitted,
  setSegmentParagraphEnd,
} from '../config/segment'
import type { ChapterSegment, SegmentTargetEdit } from '../config/segment'
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
  if ((await flushEditorNow()) === 'failed') return 'flush-failed'
  if (flush.isDirty()) {
    if ((await flushEditorNow()) === 'failed') return 'flush-failed'
    if (flush.isDirty()) {
      // 🔴 *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."* Chẩn đoán nêu
      // đích danh, cùng khuôn hai dòng `console.error` của `flushEditorNow`.
      console.error(
        `[editor] KHÔNG ký segment ${id}: tập chờ vẫn dơ sau hai lượt flush — ` +
          `từ chối thay vì ký một văn bản cũ hơn thứ đang trên màn hình (Quyết định #8).`,
      )
      return 'still-dirty'
    }
  }

  const { outcome, error } = await confirmSegment(id)
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
