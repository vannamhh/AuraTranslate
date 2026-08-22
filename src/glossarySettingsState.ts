/**
 * State của lớp phủ **Cài đặt ngưỡng quét Glossary** — Story 3.5, FR47, lớp phủ THỨ TƯ.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO STATE SỐNG Ở `src/`, KHÔNG Ở `src/panels/`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý lẽ `config/shortcutsState.ts`: ngưỡng quét là cấu hình ỨNG DỤNG (`AppConfig`,
 * `GlobalOnly` — `core/scope/kinds.rs:218`), không phải state của một panel cụ thể, và lớp
 * phủ nói về **cả ứng dụng** đúng khuôn `ShortcutsOverlay.vue`/`AttributionOverlay.vue`.
 *
 * ⚠️ Cùng luật mọi state Vue khác: tệp này **KHÔNG** được `import` vào
 * `src/commands/index.ts` — nó dùng `ref` của Vue **và** gọi `@tauri-apps/api` xuyên qua
 * `config/bootstrap.ts`, mà Kiểm C/D/E của `check:commands` nạp `commands/index.ts` bằng
 * **Node thuần**. Ba handler đi vào bằng **tiêm** qua `CommandDeps` ở `src/main.ts`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHÔNG THANH CHUYỂN PHẠM VI — ngưỡng là `AppConfig` ⇒ `GlobalOnly`
 * ─────────────────────────────────────────────────────────────────────────────
 * `core/scope/kinds.rs:218`: `AppConfig => "app_config" : GlobalOnly`, và `save_value` phía
 * Rust **từ chối** mọi loại khác. Một nút *"Tác phẩm"* bấm được ở đây sẽ ghi trượt, hoặc
 * tệ hơn, không ghi gì và trông như đã ghi — cùng cái bẫy mà `ShortcutsOverlay.vue` đã ghi
 * ra cho `Shortcut`.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import type { IpcError } from './i18n'
import {
  KEY_GLOSSARY_SCAN_THRESHOLD,
  SCOPE_APP_CONFIG,
  bootstrapGlossaryScanThreshold,
  putConfig,
} from './config/bootstrap'

const overlayOpen = ref(false)
/** Giá trị THÔ đang hiển thị trong ô nhập — chuỗi, vì người dùng có thể đang gõ dở
 * (`"-"`, `""`, `"3."`) và những dạng đó không phải một `number` hợp lệ. */
const thresholdInput = ref('')
/**
 * Ngưỡng hiệu lực theo hiểu biết của PHIÊN NÀY. `null` ⇒ chưa lưu gì trong phiên này, dùng
 * `bootstrapGlossaryScanThreshold` (đọc một lần lúc khởi động). Không gộp hai giá trị vào
 * một ref: `bootstrapGlossaryScanThreshold` là ảnh chụp CHỈ ĐỌC của lúc khởi động (cùng
 * khuôn `bootstrapLayout`) và không tự cập nhật sau một lượt lưu — ref riêng này là nơi
 * lượt lưu vừa xong được NHỚ LẠI cho lần mở kế tiếp trong cùng phiên.
 */
const knownThreshold = ref<number | null>(null)
const saving = ref(false)
const saveError = ref<IpcError | null>(null)

export const glossarySettingsOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
/** Ô nhập — `v-model` trực tiếp, cùng khuôn `quickAddTranslation`/`quickAddNote`. */
export const glossarySettingsThresholdInput: Ref<string> = thresholdInput
export const glossarySettingsSaving: DeepReadonly<Ref<boolean>> = readonly(saving)
export const glossarySettingsSaveError: DeepReadonly<Ref<IpcError | null>> = readonly(saveError)

/**
 * Trần trên của `u32` phía Rust — `parse::<u32>()` của
 * `core::scope::store::parse_glossary_scan_threshold` từ chối mọi giá trị vượt số này
 * (lỗi phân tích `ParseIntError`, rơi về mặc định 5) dù chuỗi hợp lệ về MẶT CHỮ SỐ.
 */
const RUST_U32_MAX = 4294967295

/**
 * Phân tích một chuỗi thô thành ngưỡng hợp lệ — `null` nếu không phải số nguyên dương mà
 * `u32::from_str` phía Rust cũng chấp nhận.
 *
 * ⚠️ **Đọc lại, không phát minh, ràng buộc của Rust** —
 * `core::scope::store::parse_glossary_scan_threshold` là chỗ DUY NHẤT quyết định một giá
 * trị có hợp lệ hay không; hàm này tồn tại để tránh một vòng IPC vô ích cho một giá trị mà
 * phía Rust chắc chắn từ chối (khoá nút Lưu, hiện câu từ chối NGAY, không đợi round-trip),
 * không phải một quy tắc nghiệp vụ THỨ HAI (AD-1).
 *
 * 🔵 **VÁ 2026-08-22 (rà ba lớp) — "cả hai lớp cùng nói một điều" HẾT ĐÚNG, sửa tại chỗ.**
 * Bản trước lệch Rust ở HAI chiều:
 * - **Chiều A (nhẹ):** `/^[0-9]+$/` từ chối `"+5"`, trong khi `u32::from_str` NHẬN nó (Rust
 *   cho một dấu `+` tuỳ chọn ở đầu số nguyên không dấu). Vá: `/^\+?[0-9]+$/`.
 * - **Chiều B (nặng — im lặng mất dữ liệu):** không trần trên nào. `"5000000000"` qua được
 *   `Number.isInteger`/`> 0` (JS biểu diễn nó chính xác — dưới `Number.MAX_SAFE_INTEGER`),
 *   TS coi là hợp lệ, `putConfig` ghi chuỗi đó xuống đĩa, lớp phủ ĐÓNG NHƯ ĐÃ LƯU. Rust đọc
 *   lại: `parse::<u32>()` trả `Err` (vượt `u32::MAX = 4294967295`) ⇒ rơi về mặc định 5 —
 *   người dùng gõ 5 tỷ, tưởng đã lưu 5 tỷ, ngưỡng THẬT lặng lẽ thành 5. Vá: chặn trần
 *   `RUST_U32_MAX`.
 */
export function parsedGlossaryScanThreshold(raw: string): number | null {
  const trimmed = raw.trim()
  if (!/^\+?[0-9]+$/.test(trimmed)) return null
  const n = Number(trimmed)
  return Number.isInteger(n) && n > 0 && n <= RUST_U32_MAX ? n : null
}

/** Handler thật của `glossary.settings.open`. */
export function openGlossarySettings(): void {
  const effective = knownThreshold.value ?? bootstrapGlossaryScanThreshold.value
  thresholdInput.value = String(effective)
  saveError.value = null
  overlayOpen.value = true
}

/** Handler thật của `glossary.settings.close` — KHÔNG lưu gì. */
export function closeGlossarySettings(): void {
  // Save đang bay sở hữu vòng đời modal: đóng lúc này làm người dùng mất bề mặt báo lỗi,
  // và lượt save thành công phía dưới vẫn tự đóng đúng một lần sau khi đĩa trả lời.
  if (saving.value) return
  overlayOpen.value = false
  saveError.value = null
}

/**
 * Handler thật của `glossary.settings.save`.
 *
 * ⚠️ Ô nhập không hợp lệ ⇒ trả về NGAY, **0 lượt `putConfig`** — template khoá nút Lưu bằng
 * chính [`parsedGlossaryScanThreshold`], nhưng `Enter` trên ô nhập vẫn đi qua `@submit`
 * (Kiểm A của `check:commands` chỉ canh `@click`), nên hàm này PHẢI tự kiểm lại, không tin
 * template đã chặn.
 */
export async function saveGlossarySettings(): Promise<void> {
  if (saving.value) return

  const parsed = parsedGlossaryScanThreshold(thresholdInput.value)
  if (parsed === null) return

  saving.value = true
  saveError.value = null

  const err = await putConfig(SCOPE_APP_CONFIG, KEY_GLOSSARY_SCAN_THRESHOLD, String(parsed))

  saving.value = false
  if (err !== null) {
    saveError.value = err
    return
  }

  knownThreshold.value = parsed
  overlayOpen.value = false
}


/**
 * 🔵 **KHÔNG có `resetGlossarySettings()` — quyết định, không một chỗ sót** (rà ba lớp
 * 2026-08-22). Một bản trước CÓ hàm đó, nhưng `grep` cho thấy chỉ chính test của nó gọi —
 * mã chết tồn tại riêng để qua `check:panel-refs`, một cổng xanh trên một bất biến KHÔNG
 * được giữ. Năm ô nhớ ở trên là state của MÀN HÌNH (`overlayOpen`/`thresholdInput`/
 * `saving`/`saveError`) hoặc dữ liệu `AppConfig`/`GlobalOnly` sống QUA ranh giới Tác phẩm
 * (`knownThreshold`) — không ô nào gắn với một Tác phẩm cụ thể, nên "đổi Tác phẩm" (đường
 * teardown mà `resetGlossaryMarks` nối vào ở `editorPanelState.ts:647,2021`) không phải
 * một sự kiện của chúng. Cùng đúng tiền lệ đã ký cho `shortcutsState.ts::overlayOpen`
 * (cũng `AppConfig`/`GlobalOnly`, cũng miễn trừ CÓ TÊN thay vì một hàm reset không có chỗ
 * gọi). Năm ô này đi qua `EXEMPT` của `scripts/check-panel-refs.mjs`.
 */
