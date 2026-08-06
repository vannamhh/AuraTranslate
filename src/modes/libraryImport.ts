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
 * ⚠️ Bật/tắt bằng **event của Rust**, ⛔ không bằng `dragenter`/`dragleave` của DOM: với
 * `drag_drop_enabled = true` (mặc định Tauri v2), webview ⛔ **không bao giờ** nhận được
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
 * Tác phẩm vừa tạo thành công gần nhất — một **dòng xác nhận**, ⛔ không phải một màn hình
 * thay thế.
 *
 * 🔴 `LibraryMode.vue` vẽ nó **bên cạnh** form, ⛔ **KHÔNG** trong một nhánh `v-else` bọc
 * lấy form. Bản trước làm đúng chuyện đó và ref này ⛔ không bao giờ được đặt lại, nên tạo
 * xong Tác phẩm đầu tiên là form **và** dải báo lỗi biến mất vĩnh viễn trong phiên — ⛔
 * không tạo được Tác phẩm thứ hai, ⛔ không thấy được lỗi nào nữa. Code review 2026-08-06.
 */
export const createdWork = ref<CreatedWork | null>(null)

/**
 * Khoá i18n của một lời nhắn ⛔ không phải lỗi (ví dụ: thả nhiều tệp một lúc).
 *
 * ⛔ Giữ **KHOÁ**, ⛔ không giữ CÂU — NFR16/AD-21: ⛔ không một câu tiếng Việt nào sống
 * trong mã.
 */
export const noticeKey = ref<string | null>(null)

function beginSubmit(): void {
  busy.value = true
  lastError.value = null
  noticeKey.value = null
  // Một lượt nộp mới xoá xác nhận cũ — ⛔ không để dòng "Đã tạo…" của lần trước đứng cạnh
  // kết quả của lần này và nói dối về cái vừa xảy ra.
  createdWork.value = null
}

function finishSubmit(created: CreatedWork | null, error: IpcError | null): void {
  busy.value = false
  lastError.value = error
  createdWork.value = created

  // 🔴 Tác phẩm MỚI ⇒ vứt state Panel Source VÀ Panel Lookup của Tác phẩm CŨ.
  // `replace_open_work` phía Rust vừa trỏ `OpenWorkState` sang chỗ khác, còn cờ
  // module-level của các panel thì ⛔ không hay biết — bỏ dòng này là để người dùng đọc nội
  // dung Tác phẩm A dưới nhãn Tác phẩm B. Đây là điểm nghẽn DUY NHẤT mà cả hai nhánh nhập
  // đều đi qua (xem `resetSourcePanel`/`resetLookupPanel`) — Story 1.17 CÙNG LƯỢT, ⛔ không
  // một lời gọi thứ hai rải ra.
  if (created !== null) {
    resetSourcePanel()
    resetLookupPanel()

    // 🔴 **VỨT state cũ là CHƯA ĐỦ — phải NẠP LẠI ngay tại đây.**
    //
    // `resetSourcePanel()` gỡ cờ `chapterRequested`, nhưng ⛔ ai gọi lại hàm nạp. Chỗ
    // DUY NHẤT gọi `ensureChapterLoaded()` là `SourcePanel.vue::onMounted`, mà ba chế độ
    // sống trong `<KeepAlive>` (`App.vue`, §Quyết định thiết kế #6) — `src/modes/README.md`
    // viết thẳng: *"lần hiện thứ hai trở đi ⛔ có `mounted`"*. Nên với thứ tự thao tác
    // THƯỜNG GẶP NHẤT:
    //
    //   mở app → xem Workspace trước (nạp hụt: ⛔ Tác phẩm nào đang mở)
    //          → sang Library tạo Tác phẩm  (`replace_open_work` phía Rust ĐÃ trỏ đúng)
    //          → quay lại Workspace          (KeepAlive ⇒ ⛔ `mounted` ⇒ ⛔ nạp lại)
    //
    // …Panel Source ở lại *"Chưa có Chương nào được mở"* **vĩnh viễn** dù Tác phẩm đã mở
    // thật — người dùng phải khởi động lại app. Bắt bằng test tay 2026-08-07.
    //
    // ⚠️ Vá ở ĐÂY chứ ⛔ đổi `onMounted` → `onActivated` ở `SourcePanel.vue`: panel sống
    // trong cây của dockview, nên nó có nhận được `activated` hay ⛔ là một câu hỏi về nội
    // tình thư viện thứ ba — còn `finishSubmit` là điểm nghẽn mà **cả hai** nhánh nhập đã
    // đi qua, và *"Tác phẩm đang mở vừa đổi"* là một dữ kiện mà chính chỗ này biết CHẮC.
    // Cùng lý do reset đứng ở đây thay vì rải ra từng panel.
    void ensureChapterLoaded()
  }
}

/** Nhánh dán văn bản — nộp `pastedText` hiện tại. */
export async function submitPastedText(): Promise<void> {
  if (busy.value) return
  // ⚠️ Chốt ở ĐÂY, ⛔ không chỉ ở `:disabled` của nút: `dispatch('library.import_text')`
  // là một command đã đăng ký, và một lối vào tương lai (palette, phím tắt) sẽ đi thẳng
  // qua đây mà ⛔ không đi qua thuộc tính DOM nào.
  if (pastedText.value.trim() === '') return
  beginSubmit()
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
  beginSubmit()
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
 * 🔴 THẢ MỘT TỆP ĐỔ VÀO Ô ĐƯỜNG DẪN — NÓ ⛔ KHÔNG GHI XUỐNG ĐĨA
 * ─────────────────────────────────────────────────────────────────────────────
 * AC1 nguyên văn: *"⛔ **không có gì được ghi xuống đĩa trước khi người dùng xác nhận**"*
 * (bất biến của mọi đường nhập — `EXPERIENCE.md#KF-1`). Bản trước gọi thẳng lệnh tạo ngay
 * trong bộ nghe, nên một cú thả **bất kỳ đâu trong cửa sổ** — kể cả khi đang ở Workspace
 * và màn Library ⛔ chưa từng hiện — ghi ngay một `.atproj` xuống đĩa với `name`/`genre`
 * còn sót lại trong ref. Đó là AC1 bị phá ở đúng bất biến nó tồn tại để giữ.
 * Code review 2026-08-06.
 *
 * ⇒ Thả **điền đường dẫn vào ô**, rồi người dùng bấm nút. Cùng một nút, cùng một hàm, cùng
 * một bước xác nhận với đường bàn phím — và ⛔ không đường nào giữ bản sao pipeline
 * (AD-39:498).
 *
 * ⚠️ Rust chỉ chuyển tiếp ĐƯỜNG DẪN (AD-1/AD-16) — nội dung tệp chưa từng chạm webview.
 *
 * ⛔ Không ném nếu không có cầu IPC (`npm run dev` ngoài Tauri) — cùng nhánh "không có cầu
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
    // `Drop` là sự kiện CUỐI của một chuỗi — ⛔ không còn gì lơ lửng nữa.
    isDragOver.value = false

    const first = paths[0]
    if (typeof first !== 'string' || first.length === 0) return

    filePath.value = first
    // Thả nhiều tệp: lấy tệp đầu và **NÓI RA**, ⛔ không im lặng vứt phần còn lại. Nhập
    // hàng loạt là Epic 6.
    noticeKey.value = paths.length > 1 ? 'mode.library.drop_only_first' : null
  })
}

/**
 * Gỡ ba bộ nghe — gọi từ `LibraryMode.vue::onBeforeUnmount`.
 *
 * ⚠️ Có nó vì bộ nghe là **tầng cửa sổ**, ⛔ không phải của phần tử `.dropzone`: để nó
 * sống sau khi chế độ bị tháo nghĩa là một cú thả ở Workspace vẫn điền vào một form ⛔
 * không còn trên màn hình.
 */
export function unwireDragDrop(): void {
  for (const unlisten of unlisteners) unlisten()
  unlisteners = []
  dragDropWired = false
  isDragOver.value = false
}
