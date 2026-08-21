/**
 * State "dấu thuật ngữ Glossary của Chương đang mở" — Story 3.4b, FR50/FR51.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY KHÔNG `import` GÌ TỪ `editorPanelState.ts`/`sourcePanelState.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `editorPanelState.ts` phải GỌI [`ensureGlossaryMarksLoaded`]/[`refreshGlossaryMarks`] tường
 * minh (task của story: cạnh `ensureChapterLoaded()` trong `switchChapter()`, và trong
 * `applyRegroup()`) — tức chiều phụ thuộc BẮT BUỘC là `editorPanelState.ts` →
 * `glossaryMarksState.ts`. Nếu tệp này quay lại `import` state của `editorPanelState.ts` để tự
 * đọc `editorSegments`/`editorChapterId`, đó là một VÒNG — đúng thứ `editorPanelState.ts:27`
 * đã tự cấm cho `sourcePanelState.ts` ("Không vòng"). Giải: mọi hàm ở đây nhận
 * `chapterId`/`segments`/`sourceLang` làm THAM SỐ từ chỗ gọi, thay vì tự đọc state của module
 * khác — cùng khuôn `glossary_marks_for_chapter` phía Rust nhận `text`/`source_lang` làm tham
 * số thay vì tự đọc Chương từ đĩa (`3-4b-…md` §Code Map).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 ĐÚNG MỘT LƯỢT IPC MỖI LẦN MỞ CHƯƠNG — VÀ ĐÓ LÀ KỶ LUẬT CỦA TỆP NÀY, KHÔNG CỦA ADAPTER
 * ─────────────────────────────────────────────────────────────────────────────
 * `requestedForChapterId` là chốt idempotent: [`ensureGlossaryMarksLoaded`] không phát một
 * lượt IPC thứ hai cho CÙNG một `chapterId`. [`refreshGlossaryMarks`] là đường THỨ HAI có chủ
 * — gọi tường minh sau gộp/tách (`applyRegroup`) và sau thêm nhanh một thuật ngữ
 * (`glossaryQuickAddState.ts`), đúng hai ca `3-4b-…md` §Intent liệt kê. KHÔNG chỗ nào khác
 * được gọi hai hàm này — nhất là KHÔNG trên đường gõ (Ice ký 2026-08-21).
 */
import { readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { glossaryMarksForChapter } from '../config/glossary'
import type { GlossaryMark } from '../config/glossary'
import type { IpcError } from '../i18n'
import { joinSegmentSourceText } from './glossaryMarksMap'
import type { GlossarySegmentSource } from './glossaryMarksMap'
import { clearHoveredGlossaryTerm } from './glossaryTermHoverState'

const marks = shallowRef<readonly GlossaryMark[]>([])
const marksError = shallowRef<IpcError | null>(null)
const pending = ref(false)

/** `chapter_id` mà lượt gọi gần nhất (đã xong hoặc đang bay) nhắm tới — `null` == chưa gọi. */
let requestedForChapterId: number | null = null

/**
 * 🔴 Số thứ tự lượt nạp — cùng cơ chế và cùng lý do `sequence` của `editorPanelState.ts`/
 * `hanVietSequence` của `sourcePanelState.ts`: một lượt CŨ trả lời SAU một lượt MỚI (chuyển
 * Chương rồi chuyển lại nhanh, hoặc `refreshGlossaryMarks` chồng lên một lượt mở Chương chưa
 * kịp về) không được ghi đè state hiện tại.
 */
let sequence = 0

export const glossaryMarks: DeepReadonly<Ref<readonly GlossaryMark[]>> = readonly(marks)
export const glossaryMarksError: DeepReadonly<Ref<IpcError | null>> = readonly(marksError)
export const glossaryMarksPending: DeepReadonly<Ref<boolean>> = readonly(pending)

/**
 * Vị từ *"…HasLoaded"* — Known pitfall trung tâm của dự án (`AGENTS.md`). `false` ⇔ đang chờ,
 * lượt gần nhất trượt, hoặc chưa lượt nào chạy cho Chương hiện tại.
 */
export function glossaryMarksHaveLoaded(): boolean {
  return !pending.value && marksError.value === null && requestedForChapterId !== null
}

async function loadMarksFor(
  chapterId: number,
  segments: readonly GlossarySegmentSource[],
  sourceLang: string,
): Promise<void> {
  sequence += 1
  const mine = sequence
  requestedForChapterId = chapterId
  pending.value = true

  const text = joinSegmentSourceText(segments)
  const { marks: loaded, error } = await glossaryMarksForChapter(text, sourceLang)

  // 🔴 Lượt này đã bị một lượt MỚI hơn vượt mặt (chuyển Chương / `refreshGlossaryMarks` khác)
  // — thoát TRƯỚC mọi lượt gán, cùng khuôn `ensureSegmentsLoaded`/`ensureHanVietLoaded`.
  if (mine !== sequence) return

  pending.value = false
  marks.value = loaded ?? []
  marksError.value = error

  // 🔴 Bắt ở lượt rà 2026-08-21 (ba lớp review, P5) — thuật ngữ đang được RÊ CHUỘT tới có thể
  // bị SỬA/XOÁ bởi chính lượt tải lại này (Glossary đổi giữa hai lượt mở Chương, hoặc gộp/tách
  // đổi segment mà mảnh đang hover thuộc về). `hoveredGlossaryTerm` không tự biết marks vừa
  // đổi — nó chỉ ghi khi `@mouseenter`/`onSourceSelectionChange` bắn, và chuột đứng yên thì
  // không bắn gì cả. Không dọn ở đây, `StatusBar` giữ bản dịch CŨ (hoặc của một dấu đã XOÁ)
  // cho tới khi chuột rời rồi vào lại — đúng lớp *"trạng thái treo"* mà cả story này tồn tại
  // để chống ở nơi khác. Dọn ở MỌI lượt gán (kể cả lượt nạp lần đầu — vô hại, chưa ai hover)
  // là đường AN TOÀN: người dùng rê lại chuột/di caret để thấy dấu MỚI, không bao giờ thấy dấu
  // SAI.
  clearHoveredGlossaryTerm()

  // Một lượt TRƯỢT không được khoá vĩnh viễn đường nạp — nhả `requestedForChapterId` để lượt
  // gọi kế tiếp (kể cả cho ĐÚNG `chapterId` này) không bị `ensureGlossaryMarksLoaded` coi là
  // "đã xong" một cách sai.
  if (loaded === null) requestedForChapterId = null
}

/**
 * Nạp dấu thuật ngữ cho Chương `chapterId` — **idempotent theo `chapterId`**: gọi lại cho
 * CÙNG một Chương không phát IPC lần hai. Gọi cho một `chapterId` KHÁC với lượt trước sẽ nạp
 * lại (đúng ca "mở Chương kề", `switchChapter`).
 */
export async function ensureGlossaryMarksLoaded(
  chapterId: number,
  segments: readonly GlossarySegmentSource[],
  sourceLang: string,
): Promise<void> {
  if (requestedForChapterId === chapterId) return
  await loadMarksFor(chapterId, segments, sourceLang)
}

/**
 * Nạp LẠI dấu thuật ngữ cho Chương `chapterId` — **luôn** phát một lượt IPC mới, bất kể
 * `requestedForChapterId`. Hai chỗ gọi có chủ: sau gộp/tách segment (`applyRegroup`,
 * `editorPanelState.ts`) và sau thêm nhanh một thuật ngữ thành công
 * (`glossaryQuickAddState.ts`) — cả hai đổi dữ liệu mà `ensureGlossaryMarksLoaded` sẽ coi là
 * "đã nạp rồi" và bỏ qua.
 */
export async function refreshGlossaryMarks(
  chapterId: number,
  segments: readonly GlossarySegmentSource[],
  sourceLang: string,
): Promise<void> {
  await loadMarksFor(chapterId, segments, sourceLang)
}

/**
 * 🔴 Vứt toàn bộ state — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường `reset*()`.
 */
export function resetGlossaryMarks(): void {
  sequence += 1 // vô hiệu hoá mọi lượt đang bay, cùng lý do mọi `reset*()` khác trong kho.
  marks.value = []
  marksError.value = null
  pending.value = false
  requestedForChapterId = null
  clearHoveredGlossaryTerm() // cùng lý do đã ghi ở `loadMarksFor` — marks đổi thì hover phải dọn.
}
