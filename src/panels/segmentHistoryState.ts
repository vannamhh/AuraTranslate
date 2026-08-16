/**
 * Trạng thái của bề mặt **lịch sử phiên bản segment** — Story 2.6 · FR101 · AC1 · AC2 · AC3.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 STATE CẤP MODULE, KHÔNG `ref` CỤC BỘ TRONG `<script setup>`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng khuôn `dictSourcesState.ts` và `lookupHistoryState.ts`, và lý do là một cơ chế chứ
 * không một gu: đổi preset bố cục gọi `api.clear()` rồi **dựng lại** panel, nên một `ref`
 * sống trong `<script setup>` của một component bị vứt cùng component đó. Chỉ state cấp
 * module sống sót.
 *
 * ⚠️ Lớp phủ của story này là con của `App.vue` chứ không của một panel, nên nó **không** bị
 * `api.clear()` chạm tới hôm nay. Vẫn theo khuôn: hai tiền lệ lớp phủ đang sống đều làm vậy,
 * và một bề mặt thứ ba đi lối khác là một cái bẫy cho người đọc sau.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHÔNG QUY TẮC NGHIỆP VỤ NÀO Ở TỆP NÀY (AD-1)
 * ─────────────────────────────────────────────────────────────────────────────
 * *"Phiên bản nào tồn tại"*, *"khôi phục có được phép không"*, *"bản nháp này có bản sao
 * chưa"* — cả ba quyết định ở **Rust**. Tệp này giữ trạng thái hiển thị và định tuyến kết
 * quả. Đặc biệt: chốt chống mất bản nháp **không** được tính lại ở đây, kể cả khi nó trông
 * dễ *(so `editorEditedText` với danh sách phiên bản)* — một phép tính thứ hai là một nguồn
 * sự thật thứ hai, và nó sẽ rẽ khỏi Rust ở đúng ca biên.
 */
import type { Ref } from 'vue'
import { readonly, shallowRef, type DeepReadonly } from 'vue'

import type { IpcError } from '../i18n'
import type { SegmentVersion } from '../config/segment'
import { readSegmentHistory, restoreSegmentVersion } from '../config/segment'
import {
  editorCaretSegmentId,
  flushEditorBeforeDiscreteWrite,
  replaceEditorSegment,
} from './editorPanelState'

const isOpen = shallowRef(false)
/** Lớp phủ lịch sử có đang mở không. */
export const historyIsOpen: DeepReadonly<Ref<boolean>> = readonly(isOpen)

const segmentId = shallowRef<number | null>(null)
/** Segment mà lớp phủ đang hiện lịch sử. `null` ⇒ chưa nhắm được câu nào. */
export const historySegmentId: DeepReadonly<Ref<number | null>> = readonly(segmentId)

const versions = shallowRef<readonly SegmentVersion[]>([])
/**
 * Danh sách phiên bản, **mới nhất trước** — thứ tự do Rust quyết (AC1).
 *
 * 🔴 Một mảng **rỗng** ở đây là một câu trả lời **hợp lệ**, không một trạng thái chưa nạp:
 * segment chưa từng được xác nhận. Phân biệt với *"đang nạp"* bằng [`historyPending`] và với
 * *"nạp hỏng"* bằng [`historyLoadError`] — ba trạng thái, ba biến, đúng §*"Rỗng IM LẶNG bị
 * cấm; rỗng CÓ LÝ DO thì không"*.
 */
export const historyVersions: DeepReadonly<Ref<readonly SegmentVersion[]>> = readonly(versions)

const pending = shallowRef(false)
/** Một lượt đọc hoặc ghi đang bay. */
export const historyPending: DeepReadonly<Ref<boolean>> = readonly(pending)

const loadError = shallowRef<IpcError | null>(null)
/** Lỗi của lượt đọc gần nhất. `null` ⇒ lượt đọc gần nhất đạt. */
export const historyLoadError: DeepReadonly<Ref<IpcError | null>> = readonly(loadError)

const restoreError = shallowRef<IpcError | null>(null)
/** Lỗi của lượt khôi phục gần nhất. `null` ⇒ chưa lượt nào bị từ chối. */
export const historyRestoreError: DeepReadonly<Ref<IpcError | null>> = readonly(restoreError)

/**
 * Kết quả của lượt khôi phục gần nhất, để bề mặt nói một câu **đúng việc vừa xảy ra**.
 *
 * `'unchanged'` là một kết quả **thành công** — khôi phục về đúng nội dung đang có. Nó tách
 * khỏi `'restored'` vì hai lượt đó nói hai chuyện khác nhau với người dùng.
 *
 * 🔵 **2026-08-16 (code review): thêm `'flush-failed'` và `'still-dirty'`.** Hai kết quả đó
 * TỪNG không có đường ra nào — chúng `return` trước khi đặt một ô nào và nơi gọi vứt giá trị
 * trả về ⇒ người dùng bấm khôi phục và **không một pixel nào đổi**. Đó đúng lớp *"rỗng im
 * lặng"* mà `main.ts:246-257` ghi lại là code review 2026-08-15 đã bắt và vá cho
 * `confirmSegment`; lượt này thừa hưởng bản vá đó thay vì dựng lại cái hố.
 */
export type RestoreNotice = 'restored' | 'unchanged' | 'flush-failed' | 'still-dirty' | null
const restoreNotice = shallowRef<RestoreNotice>(null)
/** Xem [`RestoreNotice`]. */
export const historyRestoreNotice: DeepReadonly<Ref<RestoreNotice>> = readonly(restoreNotice)

/**
 * 🔴 **Lượt khôi phục đang CHỜ người dùng đồng ý, vì nó sắp ghi đè một bản nháp chưa từng
 * được ký** — chữ ký #2(a) của Ice, 2026-08-16.
 *
 * `null` ⇒ không có lượt nào đang chờ. Khác `null` ⇒ **Rust đã từ chối ghi** ở lượt gọi thứ
 * nhất và **không một byte nào đã xuống đĩa**; bề mặt hiện bản nháp ra rồi hỏi.
 *
 * ⚠️ Đây **không** phải một lỗi và nó **không** đi qua [`historyRestoreError`]. Định tuyến nó
 * vào ô lỗi là biến một câu hỏi thành một lời báo hỏng.
 */
export type PendingRestore = {
  versionId: number
  /** Chính bản nháp sắp mất — để **hiện nó ra**, không chỉ nói *"có thứ sẽ mất"*. */
  draft: string
}
const pendingRestore = shallowRef<PendingRestore | null>(null)
/** Xem [`PendingRestore`]. */
export const historyPendingRestore: DeepReadonly<Ref<PendingRestore | null>> =
  readonly(pendingRestore)

/**
 * Mở lịch sử của **câu đang có con trỏ**. Handler của command `history.open`.
 *
 * 🔴 Không tham số `segmentId`, và đó là §KHÔNG-LÀM viết thành chữ ký — xem doc-comment của
 * `CommandDeps.openSegmentHistory`. Câu đang nhắm đọc từ trạng thái quanh nó, **lúc chạy**.
 *
 * ⚠️ *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU."* Mọi đường hỏng ở đây
 * đều đặt một ô trạng thái rồi trả về, không một câu nào ném.
 */
export function openSegmentHistory(): void {
  const id = editorCaretSegmentId.value
  segmentId.value = id
  versions.value = []
  loadError.value = null
  restoreError.value = null
  restoreNotice.value = null
  pendingRestore.value = null
  // 🔵 2026-08-16 (code review): hang dang nham PHAI ve `null` o day. Bo no lai la de mot
  // `version_id` cua cau TRUOC ro sang cau nay: nham mot hang o cau A, dong, mo lich su cau B,
  // roi goi `history.restore` truoc khi chuot/Tab nham lai ⇒ gui phien ban cua A cho B. Rust
  // tu choi dung nho hang rao `AND segment_id = ?2`, nhung nguoi dung nhan "khong tim thay
  // phien ban" thay vi "chua nham hang nao" -- mot cau tra loi noi sai ca nguyen nhan.
  aimedVersionId.value = null
  isOpen.value = true

  // Chua nham duoc cau nao ⇒ lop phu van MO va noi ra dieu do. Mo mot lop phu rong roi im
  // lang la dung thu "rong im lang" bi cam; khong mo gi ca thi mot cu bam phim khong phan
  // hoi, va nguoi dung khong biet minh vua lam gi sai.
  if (id === null) return

  void loadHistory(id)
}

/** Đóng lớp phủ. Handler của command `history.close`. */
export function closeSegmentHistory(): void {
  isOpen.value = false
  pendingRestore.value = null
  // 🔵 2026-08-16 (code review): cung ly do voi `openSegmentHistory` -- don ca hai dau thay vi
  // tin rang lan mo sau se don ho.
  aimedVersionId.value = null
}

const aimedVersionId = shallowRef<number | null>(null)
/**
 * Hàng phiên bản **đang nhắm** — thứ ba command không-tham-số của story này tác động lên.
 *
 * 🔴 Đây là khuôn `aimedShortcutRow` của Story 1.21, và nó là thứ giữ được **hai** luật cùng
 * lúc mà thoạt nhìn xung khắc:
 * - **AD-34 §1** *(Kiểm A của `check:commands`)*: mọi `@click` phải là **đúng một**
 *   `dispatch('<id>')` với id **literal** — nên một nút không thể mang `@click="restore(row.id)"`.
 * - **§KHÔNG-LÀM**: không một command cho mỗi hàng — nó phá chính cơ chế đếm tĩnh
 *   (`COMMAND_FLOOR`) và Story 1.21 không gán lại phím cho một id không tồn tại lúc dựng bảng.
 *
 * ⇒ Hàng được **nhắm** bằng `@mousedown`/`@focusin` *(Kiểm A nói nguyên văn "chỉ `@click`",
 * nên hai sự kiện đó được xử lý tự do)*, rồi command không tham số đọc mục tiêu **lúc chạy**.
 */
export const historyAimedVersionId: DeepReadonly<Ref<number | null>> = readonly(aimedVersionId)

/** Nhắm một hàng phiên bản. Gọi từ `@mousedown`/`@focusin`, **không** từ `@click`. */
export function aimHistoryVersion(id: number | null): void {
  aimedVersionId.value = id
}

/**
 * Khôi phục **phiên bản đang nhắm**. Handler của command `history.restore`.
 *
 * ⚠️ Chưa nhắm hàng nào ⇒ **kêu**, không ném. *"Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ
 * ném — nó KÊU."*
 */
export function restoreAimedVersion(): void {
  const id = aimedVersionId.value
  if (id === null) {
    console.warn('[history] no version row is aimed; restore does nothing.')
    return
  }
  void restoreVersion(id, false).then(noteRestoreOutcome)
}

/**
 * 🔵 **2026-08-16 (code review): KHÔNG vứt `RestoreResult`.**
 *
 * Bản trước viết `void restoreVersion(...)` và tám giá trị phân biệt được rơi thẳng xuống đất.
 * Đó đúng khuôn mà `main.ts:246-257` ghi lại là code review 2026-08-15 đã bắt cho
 * `confirmSegment`: *"một lượt bị từ chối không đổi một pixel nào trên màn hình"*.
 *
 * Bốn giá trị đã có bề mặt riêng *(`'restored'`/`'unchanged'` qua [`historyRestoreNotice`],
 * `'needs-confirmation'` qua [`historyPendingRestore`], `'refused'` qua
 * [`historyRestoreError`])* nên chúng **không** vào đây. Bốn giá trị còn lại nay cũng có bề
 * mặt, và dòng dưới là đường **chẩn đoán** *(viết KHÔNG DẤU)* — hai người đọc khác nhau, hai
 * mức chi tiết khác nhau, đúng khuôn đã ghi ở `main.ts:274-276`.
 */
function noteRestoreOutcome(result: RestoreResult): void {
  if (result === 'restored' || result === 'unchanged') return
  if (result === 'needs-confirmation' || result === 'refused') return
  console.warn(`[history] khong khoi phuc duoc: ${result}`)
}

/**
 * Đồng ý ghi đè bản nháp chưa ký. Handler của command `history.confirm_restore`.
 *
 * 🔴 Đọc `versionId` từ **chính lượt đang chờ**, không từ hàng đang nhắm: người dùng có thể đã
 * nhắm sang hàng khác trong lúc đọc câu hỏi, và khôi phục **hàng khác** với lời đồng ý dành
 * cho hàng này là biến một lời xác nhận thành một giấy phép ghi bất kỳ đâu.
 */
export function confirmPendingRestore(): void {
  const waiting = pendingRestore.value
  if (waiting === null) return
  void restoreVersion(waiting.versionId, true).then(noteRestoreOutcome)
}

async function loadHistory(id: number): Promise<void> {
  pending.value = true
  try {
    const { versions: rows, error } = await readSegmentHistory(id)
    // ⚠️ Cau da doi trong luc luot doc dang bay (nguoi dung dong roi mo cau khac). Vut ket
    //    qua cu di -- ghi no vao la hien lich su cua cau A duoi tieu de cua cau B.
    if (segmentId.value !== id) return

    if (rows === null) {
      // ⚠️ `error === null` cung vao day: ca "khong co cau IPC". Ca hai deu la "chua doc duoc",
      //    va ca hai deu KHONG duoc doc thanh "cau nay chua co phien ban nao".
      loadError.value = error
      versions.value = []
      return
    }
    loadError.value = null
    versions.value = rows
  } finally {
    if (segmentId.value === id) pending.value = false
  }
}

/**
 * Kết quả một lượt khôi phục, nhìn từ tầng giao diện.
 *
 * `'needs-confirmation'` **không** phải một lỗi — nó là câu *"đang chờ bạn"*.
 */
export type RestoreResult =
  | 'restored'
  | 'unchanged'
  | 'needs-confirmation'
  | 'refused'
  | 'flush-failed'
  | 'still-dirty'
  | 'no-segment'
  | 'stale'

/**
 * **Khôi phục** câu đang xem về một phiên bản.
 *
 * @param force bỏ qua chốt chống mất bản nháp. Chỉ đặt `true` **sau khi** người dùng đã trả
 * lời câu hỏi mà [`historyPendingRestore`] dựng ra.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 FLUSH TRƯỚC, VÀ MỘT MÃ `'saved'` KHÔNG ĐỒNG NGHĨA VỚI "TẬP CHỜ SẠCH"
 * ─────────────────────────────────────────────────────────────────────────────
 * Chốt chống mất bản nháp chạy ở Rust và nó so trên **đĩa**. `editorEditedText` có thể còn
 * giữ ký tự chưa xuống WAL (AD-35: idle 2 s, trần cứng 5 s) ⇒ không flush trước thì chốt so
 * với một bản **cũ hơn thứ người dùng đang nhìn**, và nó sẽ **không hỏi** ở đúng ca cần hỏi
 * nhất — ca người dùng vừa gõ xong một đoạn và chưa ký.
 *
 * 🔵 **2026-08-16 (code review): ba nhịp đó nay đi qua [`flushEditorBeforeDiscreteWrite`].**
 * Bản trước chép **doc-comment** của khuôn hai lượt flush *(Quyết định #8 của Story 2.5b, Ice
 * ký 2026-08-14)* sang đây kèm câu *"áp nguyên ở đây"* — nhưng chép thiếu **mã**: chỉ một lượt
 * `flushEditorNow()`, không `isDirty()`, không đường từ chối. Cổng không đỏ, và bảo đảm mà chữ
 * ký #2(a) mua được mất trong im lặng ở đúng ca cần nó nhất. Lý do đầy đủ nay sống **một chỗ**,
 * ở doc-comment của hàm dùng chung.
 */
export async function restoreVersion(versionId: number, force: boolean): Promise<RestoreResult> {
  const id = segmentId.value
  if (id === null) {
    // 🔵 2026-08-16 (code review): KEU, dung im. Bo lop phu da noi "chua nham duoc cau nao",
    // nhung mot lenh goi tu mot hop am gan lai duoc van toi day duoc.
    console.warn('[history] no segment is open; restore does nothing.')
    return 'no-segment'
  }

  // ⚠️ ANH CHUP HANG PHIEN BAN LAY TRUOC `await`. Sau lenh cho, `versions.value` co the da bi
  //    mot luot `openSegmentHistory` cua cau KHAC don ve `[]` -- tra sau lenh cho thi truot, va
  //    `replaceEditorSegment` khong chay: dia da doi ma luoi thi khong, im lang. Story 2.5c mat
  //    mot vong chan doan o dung lop loi nay (commit `4ce5bb4`).
  const target = versions.value.find((v) => v.id === versionId)

  const flushed = await flushEditorBeforeDiscreteWrite()
  if (flushed === 'failed') {
    restoreError.value = null
    restoreNotice.value = 'flush-failed'
    return 'flush-failed'
  }
  if (flushed === 'still-dirty') {
    // 🔴 *"Ham chay tu mot hop am ban phim KHONG BAO GIO nem -- no KEU."* Cung khuon chan doan
    //    voi `confirmCurrentSegmentUnguarded`.
    console.error(
      `[history] KHONG khoi phuc segment ${id}: tap cho van do sau hai luot flush — ` +
        `tu choi thay vi ghi de len mot ban chua kip xuong dia (Quyet dinh #8).`,
    )
    restoreError.value = null
    restoreNotice.value = 'still-dirty'
    return 'still-dirty'
  }

  pending.value = true
  try {
    const { outcome, error } = await restoreSegmentVersion(id, versionId, force)

    // 🔵 2026-08-16 (code review): VE CHONG KET QUA CU, cung vế mà `loadHistory` đã có. Người
    // dùng có thể đã đóng rồi mở lịch sử của câu KHÁC trong lúc lượt ghi đang bay; ba ô trạng
    // thái dưới đây là **cấp module**, không phân biệt theo segment, nên ghi kết quả của câu A
    // vào là hiện nó dưới tiêu đề của câu B — kể cả hộp thoại *"bản đang soạn sẽ mất"* của một
    // câu người dùng không còn nhìn thấy.
    if (segmentId.value !== id) return 'stale'

    if (outcome === null) {
      restoreError.value = error
      restoreNotice.value = null
      // 🔵 2026-08-16 (code review): mot luot `force` TRUOT phai don cau hoi di. Bo lai thi man
      // hinh vua hoi "co ghi de khong" vua bao "chua khoi phuc duoc" -- hai trang thai mau
      // thuan, va nguoi dung khong doc duoc luot bam vua roi da di toi dau.
      pendingRestore.value = null
      return 'refused'
    }
    restoreError.value = null

    if (outcome.needs_confirmation) {
      // 🔴 KHONG mot byte nao da duoc ghi. `unsigned_draft` khac `null` la mot bat bien cua
      //    hop dong day; neu no `null` thi day la mot payload sai, va mot chuoi rong hien ra
      //    van tot hon mot cau hoi ve mot thu khong ai thay.
      pendingRestore.value = { versionId, draft: outcome.unsigned_draft ?? '' }
      restoreNotice.value = null
      return 'needs-confirmation'
    }

    pendingRestore.value = null

    if (!outcome.restored) {
      // Khoi phuc ve dung noi dung dang co. KHONG cham anh chup -- khong co gi doi, va
      // `status` van la thu Rust vua tra ve (no GIU chu ky, khong ha).
      restoreNotice.value = 'unchanged'
      return 'unchanged'
    }

    // ⚠️ ANH CHUP HIEN THI dung MOT MANG MOI. `segments` la `shallowRef`, no KHONG theo doi
    //    mot luot sua tai cho ⇒ dia doi ma luoi thi khong. Story 2.5c mat mot vong chan doan
    //    o dung cho nay (commit `4ce5bb4`).
    // 🔵 2026-08-16 (code review): `target` chup TRUOC `await` -- xem chu thich o dau ham.
    if (target !== undefined) {
      replaceEditorSegment(id, {
        target_text: target.target_text,
        status: outcome.status,
      })
    } else {
      // 🔴 Dia DA doi ma luoi thi khong, va do la mot lech im lang. Khong mot duong nao hom nay
      //    dan toi day -- `target` chup truoc `await` va `versions` chi doi qua `loadHistory` --
      //    nen day la mot chot TU KEU cho mot khuyet tat tuong lai, khong mot nhanh du kien.
      console.error(
        `[history] segment ${id}: da khoi phuc tren dia nhung KHONG tim thay hang ${versionId} ` +
          `trong anh chup ⇒ luoi khong duoc cap nhat. Nap lai Chuong de doc dung.`,
      )
    }

    restoreNotice.value = 'restored'
    return 'restored'
  } finally {
    // ⚠️ Cung ve chong ket qua cu voi `loadHistory`: mot luot cua cau A khong duoc ha co
    //    `pending` cua cau B vua mo.
    if (segmentId.value === id) pending.value = false
  }
}

/** Bỏ câu hỏi đang chờ mà **không** khôi phục — người dùng giữ bản đang soạn. */
export function cancelPendingRestore(): void {
  pendingRestore.value = null
}

