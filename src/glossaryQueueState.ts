/**
 * State của lớp phủ **Duyệt hàng loạt một phím** — Story 3.8, FR53/FR55.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY (`src/`, không `src/panels/`)
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `glossarySettingsState.ts`/`glossaryQuickAddState.ts`: lớp phủ không phải một
 * panel của một chế độ cụ thể — nó nói về **cả ứng dụng** (danh sách toàn bộ bảng chờ của
 * Tác phẩm đang mở), đúng khuôn `ShortcutsOverlay.vue`/`GlossarySettingsOverlay.vue`.
 *
 * ⚠️ Tệp này dùng `ref`/`computed` của Vue **và** gọi `@tauri-apps/api` xuyên qua
 * `config/glossary.ts` — KHÔNG được `import` vào `src/commands/index.ts` (Kiểm C/D/E của
 * `npm run check:commands` nạp tệp đó bằng Node thuần). Sáu handler được TIÊM VÀO qua
 * `CommandDeps`, nối thật ở `src/main.ts`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 "CHƯA MỞ TÁC PHẨM" VS "ĐÃ MỞ, BẢNG CHỜ SẠCH" — HAI CA CÙNG HÌNH DẠNG `Ok([])`
 * ─────────────────────────────────────────────────────────────────────────────
 * `glossary_pending_candidates(open, ..)` phía Rust trả `Ok(vec![])` cho CẢ HAI ca (xem
 * doc-comment của nó, `commands/glossary.rs`) — và Rust của story này CHỈ được thêm một vỏ
 * IPC (bỏ ứng viên) cộng một `ORDER BY`, không được đổi hình dạng trả về của
 * `glossary_pending_candidates` để tự nó phân biệt hai ca đó (đúng ranh giới §Code Map).
 * Cách phân biệt: khi danh sách rỗng, hỏi THÊM một lượt `lookupGlossaryTerm('')` — adapter
 * ĐÃ CÓ SẴN (Story 3.3), không một vỏ IPC mới — chỉ để đọc `workTierAvailable`, đúng cờ mà
 * `GlossaryQuickAdd`/`quickAddWorkTierAvailable` đã dùng để biết "có Tác phẩm đang mở hay
 * không". Khi danh sách CÓ hàng, câu hỏi tự trả lời (bảng chờ chỉ tồn tại ở `project.db`,
 * có hàng ⇒ chắc chắn có Tác phẩm đang mở) — lượt hỏi thêm chỉ chạy đúng khi cần.
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import {
  approveGlossaryCandidate,
  lookupGlossaryTerm,
  pendingGlossaryCandidates,
  rejectGlossaryCandidate,
} from './config/glossary'
import type { GlossaryCandidate, GlossaryCategory } from './config/glossary'
import type { IpcError } from './i18n'

/** Một hàng của bảng chờ, cùng phân loại đang chọn (mặc định `'other'` — §Design Notes của
 * spec: máy KHÔNG đoán phân loại) và kết quả quyết (nếu đã xử lý trong PHIÊN NÀY). */
export type GlossaryQueueRow = {
  candidate: GlossaryCandidate
  category: GlossaryCategory
  /** `null` == chưa xử lý. Đặt CHỈ KHI lượt ghi thành công — lỗi KHÔNG đổi trường này
   * (§I/O Matrix: "Lỗi ⇒ hàng KHÔNG đổi"). */
  outcome: 'accepted' | 'rejected' | null
}

/**
 * 🔴 Vị từ *"…HasLoaded"* mở rộng — Known pitfall trung tâm của dự án (`AGENTS.md`): một
 * danh sách rỗng không tự nói vì sao nó rỗng. NĂM trạng thái:
 * - `'unknown'` — chưa nạp lần nào (lớp phủ vừa mở, lượt tra đang bay);
 * - `'ipc_unavailable'` — cầu IPC vắng (chạy ngoài Tauri) — KHÔNG một lỗi, xem
 *   `pendingGlossaryCandidates`;
 * - `'error'` — lượt tải trượt THẬT (một `IpcError` đọc được qua [`queueLoadError`]);
 * - `'no_work'` — đã tải xong, nhưng chưa mở Tác phẩm nào;
 * - `'loaded'` — đã tải xong, có Tác phẩm đang mở (`queueRows` có thể rỗng == đã duyệt hết).
 */
export type GlossaryQueueStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'no_work' | 'loaded'

// ─────────────────────────────────────────────────────────────────────────────
// State module-level — singleton của cả tiến trình, cùng khuôn glossaryQuickAddState.ts.
// ─────────────────────────────────────────────────────────────────────────────

const overlayOpen = ref(false)
const status = ref<GlossaryQueueStatus>('unknown')
const loadError = ref<IpcError | null>(null)
const rows = ref<GlossaryQueueRow[]>([])
const cursor = ref(0)
const saving = ref(false)
const actionError = ref<IpcError | null>(null)

/** Số thứ tự lượt mở — chặn một lượt tải CHẬM ghi đè lên một lượt mở SAU nó đã về nhanh
 * hơn (đua round-trip IPC), cùng khuôn `sequence` của các state Glossary khác. Cũng vô hiệu
 * hoá một lượt Nhận/Bỏ đang bay từ một PHIÊN MỞ trước đó. */
let sequence = 0

export const queueOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const queueStatus: DeepReadonly<Ref<GlossaryQueueStatus>> = readonly(status)
export const queueLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)
export const queueRows: DeepReadonly<Ref<readonly GlossaryQueueRow[]>> = readonly(rows)
export const queueCursor: DeepReadonly<Ref<number>> = readonly(cursor)
export const queueSaving: DeepReadonly<Ref<boolean>> = readonly(saving)
export const queueActionError: DeepReadonly<Ref<IpcError | null>> = readonly(actionError)

/**
 * Hàng ĐANG CHỌN, hoặc `null` nếu con trỏ ngoài phạm vi (danh sách rỗng).
 *
 * 🔴 `.at(cursor.value)`, KHÔNG `[cursor.value]` — `noUncheckedIndexedAccess` KHÔNG bật
 * trong `tsconfig.json`, nên phép lập chỉ số trần kiểu `GlossaryQueueRow` (không
 * `| undefined`) dù chỉ số có thể NGOÀI mảng lúc chạy. `.at()` khai đúng `T | undefined`
 * bất kể cờ đó — cùng khuôn `editorPanelState.ts:1044,2151` ("sửa KIỂU cho nó nói thật").
 */
export const queueCurrentRow = computed<GlossaryQueueRow | null>(() => rows.value.at(cursor.value) ?? null)

/**
 * Số hàng CHƯA XỬ LÝ (`outcome === null`) — cụm D vá. Thước đo đúng cho "đã duyệt hết",
 * KHÁC `queueRows.length` (một hằng số trong một phiên mở, xem doc-comment
 * [`queueEmptyReasonFor`]).
 */
export const queueUnprocessedCount = computed<number>(() => rows.value.filter((r) => r.outcome === null).length)

/**
 * Đúng khuôn `quickAddLookupHasLoaded` — chỉ `'loaded'` cho phép khẳng định "bảng chờ sạch";
 * bốn trạng thái còn lại phải hiện câu giải thích RIÊNG của chúng, không im lặng.
 *
 * 🔴 **SỬA (cụm D vá, vòng rà Epic 3) — hai vá gộp:**
 * ① `'unknown'` nay có một ca **CÓ TÊN** (`'loading'`) thay vì rơi vào `return null` — bản
 *    trước để `GlossaryQueueOverlay.vue` tự canh `queueStatus === 'unknown'` bằng một `v-if`
 *    RIÊNG ở template, hai chỗ cùng canh một mệnh đề là hai nguồn sự thật (§Design Notes).
 * ② tham số đổi từ `rowCount` (`queueRows.length`, một HẰNG SỐ trong một phiên mở — `rows`
 *    chỉ bị GÁN LẠI ở `openGlossaryQueue`/`resetGlossaryQueue`, còn Nhận/Bỏ chỉ đổi
 *    `row.outcome`, KHÔNG BAO GIỜ co mảng) sang `unprocessedCount` (số hàng `outcome ===
 *    null`, đo bằng [`queueUnprocessedCount`] — một `computed` lọc TRỌN mảng, KHÔNG
 *    `firstPendingIndexFrom` bên dưới: hàm đó chỉ cần biết CÓ hàng chưa xử lý hay không, kể
 *    từ một điểm bắt đầu, để tiến con trỏ — nó dừng ở phần tử ĐẦU TIÊN khớp, không đếm hết).
 *    Bản trước `rowCount === 0` là mã CHẾT cho mục đích "đã duyệt hết": nó chỉ đúng khi bảng
 *    chờ RỖNG NGAY TỪ ĐẦU (đã đi qua nhánh `no_work`/danh sách rỗng ở `openGlossaryQueue` từ
 *    trước), không bao giờ đúng SAU KHI người dùng Nhận/Bỏ hết mọi hàng của một phiên có dữ liệu.
 */
export function queueEmptyReasonFor(
  s: GlossaryQueueStatus,
  unprocessedCount: number,
): 'loading' | 'no_work' | 'ipc_unavailable' | 'all_reviewed' | null {
  if (s === 'unknown') return 'loading'
  if (s === 'no_work') return 'no_work'
  if (s === 'ipc_unavailable') return 'ipc_unavailable'
  if (s === 'loaded' && unprocessedCount === 0) return 'all_reviewed'
  return null
}

/**
 * Tìm chỉ số hàng CHƯA XỬ LÝ đầu tiên từ `from` (bao gồm), tiến VỀ SAU — không lùi, không
 * vòng. Dùng để con trỏ TỰ TIẾN sau một lượt Nhận/Bỏ thành công (§I/O Matrix).
 */
function firstPendingIndexFrom(from: number): number | null {
  for (let i = from; i < rows.value.length; i += 1) {
    if (rows.value.at(i)?.outcome === null) return i
  }
  return null
}

/**
 * Mở lớp phủ — điểm vào DUY NHẤT, gọi từ handler của lệnh `glossary.queue.open`.
 *
 * 🔴 CHỐT TÁI MỞ — cùng lý do `openGlossaryQuickAdd`: hợp âm mặc định có thể bắn lần hai
 * trong khi lớp phủ đã mở; mở lại là VÔ HẠI (bỏ qua), không một lượt tải thứ hai đè lên
 * lượt đang có trên màn hình.
 */
export async function openGlossaryQueue(): Promise<void> {
  if (overlayOpen.value) return

  sequence += 1
  const mySequence = sequence

  overlayOpen.value = true
  status.value = 'unknown'
  loadError.value = null
  rows.value = []
  cursor.value = 0
  saving.value = false
  actionError.value = null

  const pending = await pendingGlossaryCandidates()
  if (mySequence !== sequence) return // bị một lượt mở SAU vượt mặt.

  if (pending.candidates === null) {
    status.value = pending.error === null ? 'ipc_unavailable' : 'error'
    loadError.value = pending.error
    return
  }

  if (pending.candidates.length > 0) {
    status.value = 'loaded'
    rows.value = pending.candidates.map((candidate) => ({
      candidate,
      category: 'other' as GlossaryCategory,
      outcome: null,
    }))
    cursor.value = 0
    return
  }

  // Rỗng — hỏi thêm MỘT lượt (adapter đã có sẵn từ Story 3.3) để phân biệt "chưa mở Tác
  // phẩm" với "đã mở, bảng chờ sạch". Xem doc-comment đầu tệp.
  const probe = await lookupGlossaryTerm('')
  if (mySequence !== sequence) return

  if (probe.found === 'unknown') {
    status.value = probe.error === null ? 'ipc_unavailable' : 'error'
    loadError.value = probe.error
    return
  }

  status.value = probe.workTierAvailable ? 'loaded' : 'no_work'
  rows.value = []
}

/**
 * Đóng lớp phủ — `Esc` hoặc nút Đóng. **KHÔNG dọn state** (§Tasks của spec: "không reset
 * trong hàm đóng") — mở lại luôn tải lại từ đầu (`openGlossaryQueue`), nên giữ nguyên state
 * cũ trong lúc đóng không có hệ quả sai; dọn nó ở đây chỉ xoá mất cơ hội debug/đọc lại của
 * chính phiên vừa xong.
 *
 * 🔴 Chặn trong lúc một lượt Nhận/Bỏ đang bay — cùng khuôn `closeGlossarySettings`/
 * `closeGlossaryQuickAdd`: đóng giữa chừng làm người dùng mất bề mặt báo lỗi nếu lượt ghi
 * trượt.
 */
export function closeGlossaryQueue(): void {
  if (saving.value) return
  overlayOpen.value = false
}

/** Chuyển con trỏ xuống hàng kế tiếp (mọi hàng, kể cả đã xử lý) — không vòng. */
export function nextGlossaryQueueCandidate(): void {
  if (cursor.value < rows.value.length - 1) cursor.value += 1
}

/** Chuyển con trỏ lên hàng trước (mọi hàng, kể cả đã xử lý) — không vòng. */
export function prevGlossaryQueueCandidate(): void {
  if (cursor.value > 0) cursor.value -= 1
}

/**
 * Đổi phân loại của hàng ĐANG CHỌN — phím `1`–`4`, xử lý CỤC BỘ trong `.vue`, KHÔNG qua
 * `CommandRegistry` (đúng tiền lệ `GlossaryQuickAdd.vue:55-60`, xem §Always của spec).
 * **0** lượt ghi — chỉ đổi state của phiên, dùng khi bấm Nhận.
 *
 * Không làm gì nếu hàng đã xử lý (phân loại của nó đã ghi hoặc không còn ý nghĩa).
 */
export function setGlossaryQueueCategory(category: GlossaryCategory): void {
  const row = rows.value.at(cursor.value) // 🔴 `.at()` — xem doc-comment `queueCurrentRow`.
  if (row === undefined || row.outcome !== null) return
  row.category = category
}

/**
 * Nhận ứng viên ĐANG CHỌN — lệnh `glossary.queue.accept`.
 *
 * `translation` truyền vào `approveGlossaryCandidate` theo ĐÚNG hai nhánh của §I/O Matrix:
 * `han_viet_status === 'ok'` ⇒ đề xuất Hán Việt; bốn nhánh còn lại ⇒ `null` (mục sinh ra ở
 * trạng thái chờ chốt, FR114).
 *
 * Thành công ⇒ hàng lùi màu + `✓`, con trỏ TỰ TIẾN tới hàng chưa xử lý kế tiếp -- CHỈ KHI
 * con trỏ vẫn đứng ở hàng vừa quyết lúc lượt ghi về (chưa bị `next`/`prev` dời đi trong lúc
 * `await`) -- xem chú thích tại chỗ.
 * Trượt ⇒ hàng KHÔNG đổi, con trỏ KHÔNG tiến, lỗi hiện qua [`queueActionError`].
 */
export async function acceptGlossaryQueueCandidate(): Promise<void> {
  if (saving.value) return
  const index = cursor.value
  const row = rows.value.at(index) // 🔴 `.at()` — xem doc-comment `queueCurrentRow`.
  if (row === undefined || row.outcome !== null) return

  saving.value = true
  actionError.value = null
  const mySequence = sequence

  const translation = row.candidate.han_viet_status === 'ok' ? row.candidate.han_viet_suggestion : null
  const result = await approveGlossaryCandidate(row.candidate.id, translation, row.category)
  if (mySequence !== sequence) return // lớp phủ đã đóng rồi mở lại — bỏ, không ghi đè.

  saving.value = false
  if (result.error !== null) {
    actionError.value = result.error
    return
  }

  row.outcome = 'accepted'
  // 🔴 SỬA 2026-08-24 (vòng rà ba lớp) — chỉ TỰ TIẾN khi con trỏ VẪN đứng ở `index`. Trong
  // lúc `await` ở trên, người dùng có thể đã bấm Chuyển (`next`/`prev`) và dời con trỏ sang
  // một hàng KHÁC — `cursor.value = next` vô điều kiện ở đây sẽ ĐÈ MẤT lượt điều hướng đó
  // ngay khi lượt ghi (đang bay TỪ TRƯỚC lượt điều hướng) về tới. Đúng khuôn `sequence`:
  // không phải mọi cuộc đua đều là đua round-trip IPC, đây là đua giữa MỘT async write và
  // MỘT thao tác đồng bộ khác trên cùng ô nhớ.
  const next = firstPendingIndexFrom(index + 1)
  if (next !== null && cursor.value === index) cursor.value = next
}

/**
 * Bỏ ứng viên ĐANG CHỌN — lệnh `glossary.queue.reject`. Cùng khuôn
 * [`acceptGlossaryQueueCandidate`], không tham số `translation`/`category`.
 */
export async function rejectGlossaryQueueCandidate(): Promise<void> {
  if (saving.value) return
  const index = cursor.value
  const row = rows.value.at(index) // 🔴 `.at()` — xem doc-comment `queueCurrentRow`.
  if (row === undefined || row.outcome !== null) return

  saving.value = true
  actionError.value = null
  const mySequence = sequence

  const result = await rejectGlossaryCandidate(row.candidate.id)
  if (mySequence !== sequence) return

  saving.value = false
  if (result.error !== null) {
    actionError.value = result.error
    return
  }

  row.outcome = 'rejected'
  // 🔴 Cùng vệ — xem chú thích ở `acceptGlossaryQueueCandidate`.
  const next = firstPendingIndexFrom(index + 1)
  if (next !== null && cursor.value === index) cursor.value = next
}

/**
 * 🔴 Vứt toàn bộ state của lớp phủ — `check:panel-refs` đòi mọi ô nhớ cấp module có một
 * đường `reset*()`. **0 chỗ gọi sản phẩm hôm nay**, đúng tiền lệ
 * `glossaryQuickAddState.ts::resetGlossaryQuickAdd`: lớp phủ là một MODAL chắn hết tương
 * tác phía sau nó (scrim + bẫy Tab), nên không có đường nào để Tác phẩm/Chương đổi ĐANG khi
 * nó mở; và mỗi lượt mở lại (`openGlossaryQueue`) đã tự nạp lại TOÀN BỘ state từ đầu, nên
 * không có "state cũ của Tác phẩm khác" nào sống sót để cần dọn riêng.
 */
export function resetGlossaryQueue(): void {
  sequence += 1
  overlayOpen.value = false
  status.value = 'unknown'
  loadError.value = null
  rows.value = []
  cursor.value = 0
  saving.value = false
  actionError.value = null
}
