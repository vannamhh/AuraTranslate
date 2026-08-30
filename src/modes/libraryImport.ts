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
 * ⇒ Hai thao tác nộp form (`library.import_text`, `library.import_file`) đăng ký ở
 * `src/commands/index.ts` như hai `CommandDeps` TIÊM VÀO — cùng khuôn `applyPreset` /
 * `togglePanel` của `src/layout/dockController.ts`. Module này là **phía cung cấp**:
 * `LibraryMode.vue` đọc/ghi các `ref` ở đây qua `v-model`; `src/main.ts` nối
 * `submitPastedText` / `submitFilePath` vào `installCommands({...})`.
 */
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { createWorkFromFile, createWorkFromText } from '../config/project'
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
import type { CreatedWork } from '../config/project'
import type { IpcError } from '../i18n'

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

let dragDropWired = false
let unlisteners: UnlistenFn[] = []

/** Tên Tác phẩm — dùng cho cả hai nhánh (AC1). */
export const name = ref('')

/**
 * Ngôn ngữ nguồn — **bất biến sau khi tạo** (FR3, AD-18). Hai mã khớp `core::matching`
 * (`lang = 'zh'` / `lang = 'en'`, xem `tests/dict_boundary.rs`).
 */
export const sourceLang = ref<'zh' | 'en'>('zh')

/** Thể loại — tự do, rỗng hợp lệ. */
export const genre = ref('')

/** Nội dung ô dán văn bản (AC1 nhánh a). */
export const pastedText = ref('')

/** Nội dung ô nhập đường dẫn (AC1 nhánh b — vá NFR17: đường bàn phím cho nhánh tệp). */
export const filePath = ref('')

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

function finishSubmit(created: CreatedWork | null, error: IpcError | null): void {
  busy.value = false
  lastError.value = error
  createdWork.value = created

  // 🔴 Tác phẩm MỚI ⇒ vứt state Panel Source VÀ Panel Lookup của Tác phẩm CŨ.
  // `replace_open_work` phía Rust vừa trỏ `OpenWorkState` sang chỗ khác, còn cờ
  // module-level của các panel thì không hay biết — bỏ dòng này là để người dùng đọc nội
  // dung Tác phẩm A dưới nhãn Tác phẩm B. Đây là điểm nghẽn DUY NHẤT mà cả hai nhánh nhập
  // đều đi qua (xem `resetSourcePanel`/`resetLookupPanel`) — Story 1.17 CÙNG LƯỢT, không
  // một lời gọi thứ hai rải ra.
  if (created !== null) {
    resetSourcePanel()
    resetLookupPanel()
    // 🔴 Story 2.2 — Panel Editor mang **cùng** lớp cache module-level, nên nó phải đi cùng
    // lượt vứt này. Bỏ nó ra là để Editor hiện segment của Tác phẩm A dưới nhãn Tác phẩm B,
    // đúng đường hỏng mà `resetSourcePanel` đã ghi lại từ code review 2026-08-06.
    resetEditorPanel()
    // 🔵 THÊM Story 5.11 — Chế độ đọc mang cùng lớp cache module-level, cùng lý do dòng trên.
    resetReading()
    resetReadingToc()

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
    void ensureSegmentsLoaded()
  }
}

/** Nhánh dán văn bản — nộp `pastedText` hiện tại. */
export async function submitPastedText(): Promise<void> {
  if (busy.value) return
  // ⚠️ Chốt ở ĐÂY, không chỉ ở `:disabled` của nút: `dispatch('library.import_text')`
  // là một command đã đăng ký, và một lối vào tương lai (palette, phím tắt) sẽ đi thẳng
  // qua đây mà không đi qua thuộc tính DOM nào.
  if (pastedText.value.trim() === '') return
  // ⚠️ `beginSubmit` trả `false` khi bản dịch cũ chưa chạm WAL — đi tiếp là mất nó im lặng.
  if (!(await beginSubmit())) return
  const result = await createWorkFromText(name.value, sourceLang.value, genre.value, pastedText.value)
  finishSubmit(result.created, result.error)
  if (result.created) {
    pastedText.value = ''
  }
}

/** Nhánh tệp — nộp `filePath` hiện tại (ô nhập đường dẫn, bàn phím **hoặc** kéo-thả). */
export async function submitFilePath(): Promise<void> {
  if (busy.value) return
  const path = filePath.value.trim()
  if (path === '') return
  // ⚠️ Cùng lý do và cùng chốt với `submitPastedText`.
  if (!(await beginSubmit())) return
  const result = await createWorkFromFile(name.value, sourceLang.value, genre.value, path)
  finishSubmit(result.created, result.error)
  if (result.created) {
    filePath.value = ''
  }
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

  wire<unknown>(DRAG_ENTER_EVENT, () => {
    isDragOver.value = true
  })
  wire<unknown>(DRAG_LEAVE_EVENT, () => {
    isDragOver.value = false
  })
  wire<string[]>(DRAG_DROP_EVENT, (paths) => {
    // `Drop` là sự kiện CUỐI của một chuỗi — không còn gì lơ lửng nữa.
    isDragOver.value = false

    const first = paths[0]
    if (typeof first !== 'string' || first.length === 0) return

    filePath.value = first
    // Thả nhiều tệp: lấy tệp đầu và **NÓI RA**, không im lặng vứt phần còn lại. Nhập
    // hàng loạt là Epic 6.
    noticeKey.value = paths.length > 1 ? 'mode.library.drop_only_first' : null
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
}
