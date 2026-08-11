/**
 * Adapter IPC phía webview cho cấu hình khởi động — Story 1.8, AC5.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ VÌ SAO MỘT THƯ MỤC `src/config/` MỚI
 * ─────────────────────────────────────────────────────────────────────────────
 * Cây nguồn của `ARCHITECTURE-SPINE.md` chỉ liệt kê `modes/ panels/ layout/ commands/
 * tokens/ i18n/`, nên đây là một thư mục **ngoài** khai báo và nó cần một lý do:
 *
 * - **Không đặt vào `src/commands/`** — `scripts/check-commands.mjs` (Kiểm C/D/E) và
 *   `scripts/check-i18n.mjs` (Kiểm E) **nạp thẳng các tệp `.ts` ở đó bằng Node thuần**
 *   (type-stripping, Node ≥ 22.18). Một `import` giá trị của `@tauri-apps/api` ở đó giết
 *   **ba** phép kiểm hành vi cùng lúc. Đó chính là lý do `installCommands` nhận phụ thuộc
 *   bằng **tiêm** (Story 1.6 §Completion Notes), và `bindings` đi vào cùng cửa đó.
 * - **Không đặt vào `src/modes/`** — sai khái niệm: đây không phải state chế độ.
 *
 * ⚠️ Và nó **không phải một khái niệm miền mới**. Nó là **adapter**: một lời gọi
 * `invoke`, một `try/catch`, không quy tắc nào. Miền sống ở Rust (`EXPERIENCE.md:23`:
 * *"Tách câu, khớp ngôn ngữ, phân giải scope đều nằm ở Rust"*).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 HÀM NÀY KHÔNG BAO GIỜ NÉM
 * ─────────────────────────────────────────────────────────────────────────────
 * Nó chạy **trước `mount()`**, và `index.html` chỉ có một `<div id="app">` rỗng — nên một
 * lần ném ở đây cho ra **cửa sổ trắng hoàn toàn**, với chẩn đoán nằm trong một console mà
 * người dùng cuối không mở. Cùng lớp lỗi mà `applyTheme()` đã có chốt để chặn ở Story 1.4
 * và mà khối `try` quanh `installCommands` chặn ở Story 1.6.
 *
 * Không đọc được cấu hình **không phải** lý do để ứng dụng không lên: mọi giá trị đều có
 * mặc định, và mặc định là một ứng dụng dùng được.
 *
 * ⚠️ Cấu hình **chỉ** tới webview qua IPC. `assetProtocol.scope` không bao giờ chứa
 * `$APPDATA` (test `asset_protocol_scope_never_contains_appdata`), nên webview **không đọc
 * được** `global.db` — không có đường thứ hai, và đó là chủ ý.
 */
import { invoke } from '@tauri-apps/api/core'
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import type { IpcError } from '../i18n'

/**
 * Hình dạng `BootstrapConfig` phía Rust — **`snake_case`, đúng như trên dây**.
 *
 * Bốn tên trường không phải sở thích: `commands/config.rs` cố ý KHÔNG đặt
 * `#[serde(rename_all = "camelCase")]`, và test `ipc_error_wire_shape` phía Rust là thứ
 * giữ hai đầu khớp nhau — không phải kiểu này. Đổi một tên ở đây mà không đổi ở kia cho ra
 * `undefined` mà TypeScript không hề biết.
 */
export type BootstrapConfig = {
  theme: string
  mode: string
  shortcuts: Record<string, string>
  layout_presets: Record<string, string>
  /**
   * Bố cục panel ĐANG HIỂN THỊ, đã `JSON.stringify` (Story 1.14 · AC4 · §Quyết định #5A).
   *
   * ⚠️ Chuỗi RỖNG = chưa có gì trên đĩa ⇒ preset mặc định (lưới 2×2). Cùng luật với
   * `DEFAULT_THEME` / `DEFAULT_MODE`: tầng Rust quyết mặc định, không để `?? '…'` phía
   * này gánh — `??` chỉ bắt `null`/`undefined`, còn `''` là một giá trị.
   *
   * 🔴 Đừng đọc trường này thành *"preset bố cục"*. `layout_presets` ở trên là **preset
   * đã ĐẶT TÊN** (`ScopeKind::LayoutPreset`, `GlobalOnly`); trường này là *"lần cuối người
   * dùng để bốn panel ở đâu"* và nó sống trong `ScopeKind::AppConfig` cùng cửa với `theme`
   * và `mode`. `kinds.rs:206-213` phân xử ranh giới đó.
   */
  workspace_layout: string
  /**
   * Các nguồn từ điển **đang bị TẮT**, ngăn nhau bằng `,` — Story 1.19 · AC5 · §Quyết định #1a.
   *
   * 🔴 **Giá trị là tập BỊ TẮT, không phải tập được bật.** Mặc định là *mọi nguồn đều bật*,
   * nên một nguồn **mới** (một tệp `.db` thêm ở bản sau) phải tự động bật. Lưu tập được-bật
   * làm nguồn mới im lặng **tắt** ngay khi nó xuất hiện — một lớp dữ liệu có mặt trong bản
   * cài mà không ai thấy, đúng lớp lỗi *"rỗng im lặng"* mà AD-44 ④ cấm.
   *
   * ⚠️ Chuỗi RỖNG = chưa ai tắt gì. Cùng luật `workspace_layout`: `''` và `undefined` phải
   * dẫn về **cùng một** nhánh, và `??` chỉ bắt `null`/`undefined`.
   */
  dict_sources_disabled: string
}

/**
 * Kết quả của một lượt nạp. **Ba trạng thái, không phải hai** — và phân biệt chúng là cả
 * điểm của kiểu này:
 *
 * - `{ config, error: null }` — đọc được.
 * - `{ config: null, error }` — Rust **trả lời** bằng một lỗi thật *(kho không mở được,
 *   đường đọc trượt)*. Đây là thứ người dùng phải nhìn thấy.
 * - `{ config: null, error: null }` — **không có cầu IPC nào cả** *(`npm run dev` trong
 *   một trình duyệt thường)*. KHÔNG phải một lỗi để hiện lên: dựng một `IpcError` giả ở
 *   đây làm mọi phiên `npm run dev` mọc một dải *"Không mở được kho dữ liệu"* — một câu
 *   sai, và một câu sẽ dạy người đọc bỏ qua đúng dải báo lỗi đó.
 */
export type BootstrapResult = {
  config: BootstrapConfig | null
  error: IpcError | null
}

/**
 * Payload lỗi có phải hình dạng bốn trường của AD-21 không.
 *
 * ⚠️ Kiểm **cả bốn** trường chứ không chỉ `message_key`: `tError` đã tự lo nhánh
 * `message_key` thiếu, nhưng một `Error` thường của JavaScript *(không có cầu IPC)* cũng
 * không có `code` lẫn `params` — và phân biệt hai thứ đó là toàn bộ khác biệt giữa trạng
 * thái thứ hai và trạng thái thứ ba ở trên.
 */
function isIpcError(value: unknown): value is IpcError {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<IpcError>
  return (
    typeof v.code === 'string' &&
    typeof v.message_key === 'string' &&
    typeof v.retryable === 'boolean' &&
    typeof v.params === 'object' &&
    v.params !== null
  )
}

/** Tên command trên dây. Khớp `src-tauri/src/commands/config.rs` (module `wire`). */
const CMD_BOOTSTRAP = 'bootstrap_config'
const CMD_PUT = 'put_config'
const CMD_DELETE = 'delete_config'

/**
 * Loại scope và khoá của bố cục đang hiển thị — Story 1.14 · AC4.
 *
 * ⚠️ Hai hằng số, không hai chuỗi viết thẳng ở chỗ gọi: `put_config` nhận `kind` và
 * `key` dưới dạng **chuỗi trên dây**, nên một lỗi gõ ở đây không có kiểu nào bắt được —
 * `scope::save_value` sẽ trả `store.write_failed` lúc CHẠY và lượt lưu im lặng biến mất.
 * Khớp `ScopeKind::AppConfig => "app_config"` (`kinds.rs`) và `KEY_LAYOUT` (`store.rs`).
 */
export const SCOPE_APP_CONFIG = 'app_config'
export const KEY_LAYOUT = 'workspace_layout'
/**
 * Khoá thứ tư của cùng cửa `app_config` — Story 1.19 · AC5.
 *
 * ⚠️ Cùng lý do hai hằng ở trên tồn tại: `put_config` nhận `kind`/`key` là **chuỗi trên
 * dây**, nên một lỗi gõ ở đây không có kiểu nào bắt được — lượt lưu chỉ im lặng biến mất.
 * Khớp `KEY_DICT_DISABLED` ở `src-tauri/src/core/scope/store.rs`.
 */
export const KEY_DICT_DISABLED = 'dict_sources_disabled'
/**
 * Loại thứ hai đi qua cửa này — Story 1.21 · AC2 · AC4.
 *
 * ⚠️ **Không** `app_config`: hợp âm phím tắt là một loại riêng ở `ScopeKind::Shortcut`
 * (`kinds.rs`), với **khoá là chính id thao tác** *(`mode.library`, `lookup.toggle_pin`)*
 * chứ không một danh sách khoá cố định. Nó `GlobalOnly` — `mockups/settings.html:246`:
 * *"một thao tác không nên đổi phím theo từng Tác phẩm"*.
 */
export const SCOPE_SHORTCUT = 'shortcut'

const lastError = ref<IpcError | null>(null)
const layout = ref('')

/**
 * Bố cục đã lưu, đọc **một lần** lúc khởi động (AC4).
 *
 * ⚠️ Một `ref` chứ không một hằng: `loadBootstrapConfig()` là `async` và chạy trước
 * `mount()`, nên giá trị chỉ có sau vòng IPC. `WorkspaceMode` đọc nó qua template (Vue tự
 * bóc `.value`), và nó không bao giờ đổi sau đó — lượt GHI đi đường khác (`putConfig`),
 * không quay ngược về đây. Một vòng đọc–ghi hai chiều ở đây sẽ làm mỗi lượt lưu kích
 * hoạt một lượt dựng lại bố cục.
 */
export const bootstrapLayout: DeepReadonly<Ref<string>> = readonly(layout)

/**
 * Lỗi cấu hình gần nhất mà **Rust trả lời**, để `App.vue` vẽ một dải báo lỗi **không
 * chặn**. Đóng `deferred-work.md:177`.
 *
 * ⚠️ Chỉ đọc ở nơi tiêu thụ; [`loadBootstrapConfig`] là đường đặt duy nhất — cùng khuôn
 * `currentMode` / `setMode` của `src/modes/modeState.ts`.
 *
 * `null` khi **không có cầu IPC** *(`npm run dev`)*: đó không phải một lỗi để hiện lên.
 * Xem [`BootstrapResult`].
 */
export const configError: DeepReadonly<Ref<IpcError | null>> = readonly(lastError)

/**
 * Nạp cấu hình khởi động. Không ném — xem doc-comment đầu tệp.
 */
export async function loadBootstrapConfig(): Promise<BootstrapResult> {
  try {
    const config = await invoke<BootstrapConfig>(CMD_BOOTSTRAP)
    lastError.value = null
    // ⚠️ `?? ''` là canh gác LÚC CHẠY, không phải một mặc định thứ hai: giá trị vừa vượt
    // ranh giới IPC nên kiểu TypeScript không nói được gì về nó, và một bản Rust cũ hơn
    // (trước Story 1.14) không có trường này. Chuỗi rỗng ⇒ preset mặc định — cùng nhánh
    // với "kho rỗng", không phải một nhánh lỗi.
    layout.value = typeof config?.workspace_layout === 'string' ? config.workspace_layout : ''
    return { config, error: null }
  } catch (err) {
    if (isIpcError(err)) {
      // Rust đã trả lời, và câu trả lời là một lỗi. Người dùng phải biết.
      lastError.value = err
      return { config: null, error: err }
    }
    // Không có cầu IPC — `npm run dev` trong một trình duyệt thường. Ứng dụng vẫn lên
    // bằng mặc định; chẩn đoán ra console và không ra màn hình.
    console.info(
      `[config] không gọi được \`${CMD_BOOTSTRAP}\` — chạy ngoài Tauri? ` +
        `Dùng cấu hình mặc định. ${String(err)}`,
    )
    return { config: null, error: null }
  }
}

/**
 * Ghi một giá trị cấu hình xuống tầng Global. Không ném.
 *
 * ⚠️ Trả `IpcError | null` chứ không `void`: một lượt lưu trượt là thứ người dùng có quyền
 * biết *(AD-21, và `store.write_failed` nghĩa đen là "thay đổi vừa rồi chưa được lưu")*.
 * Chỗ gọi hôm nay chỉ ghi log — màn hình Cài đặt là chuyện của story sau.
 */
export async function putConfig(
  kind: string,
  key: string,
  value: string,
): Promise<IpcError | null> {
  try {
    await invoke(CMD_PUT, { kind, key, value })
    return null
  } catch (err) {
    if (isIpcError(err)) return err
    console.info(
      `[config] không gọi được \`${CMD_PUT}\` — chạy ngoài Tauri? ` +
        `Lựa chọn này sẽ không được nhớ. ${String(err)}`,
    )
    return null
  }
}

/**
 * Xoá một khoá cấu hình ở tầng Global. Không ném. Story 1.21 · AC8.
 *
 * 🔴 **Không phải [`putConfig`] với chuỗi rỗng.** Đường đọc phân biệt ba trạng thái bằng
 * **sự có mặt của khoá**: vắng mặt ⇒ *"dùng mặc định của sản phẩm"*; có mặt với giá trị
 * rỗng ⇒ *"thao tác này cố ý không có phím"*. Màn hình phím tắt phải tới được cả hai, và
 * một màn hình chỉ có "bỏ gán" là một cửa **một chiều**.
 *
 * ⚠️ Xoá một khoá **không tồn tại** là thành công — xem `core/scope/store.rs::delete_value`.
 *
 * Trả `IpcError | null` cùng khuôn [`putConfig`]: một lượt xoá trượt là thứ người dùng có
 * quyền biết (AD-21).
 */
export async function deleteConfig(kind: string, key: string): Promise<IpcError | null> {
  try {
    await invoke(CMD_DELETE, { kind, key })
    return null
  } catch (err) {
    if (isIpcError(err)) return err
    console.info(
      `[config] không gọi được \`${CMD_DELETE}\` — chạy ngoài Tauri? ` +
        `Lựa chọn này sẽ không được nhớ. ${String(err)}`,
    )
    return null
  }
}
