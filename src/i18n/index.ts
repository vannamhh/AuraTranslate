/**
 * Chỗ DUY NHẤT trong dự án chạm `vi.json`.
 *
 * `./resolve.ts` là hàm thuần và phải ở nguyên như vậy (đọc doc-comment ở đó trước
 * khi thêm bất cứ `import` nào). Mọi thứ cần bundler — nạp JSON, sau này là Vue —
 * sống ở tệp này. `resolveJsonModule` đã bật sẵn ở `tsconfig.json`.
 *
 * Nơi tiêu thụ chỉ nhìn thấy `t` và `tError`. Đó chính là ranh giới khiến việc đổi ý
 * về sau rẻ: khi có ngôn ngữ thứ hai thật, thay `createResolver` ở SAU ranh giới này
 * là xong — không tệp nào khác phải sửa.
 */
import catalog from './vi.json'
import { createResolver } from './resolve'
import type { MessageParams } from './resolve'

export type { MessageCatalog, MessageParams, Translate } from './resolve'

/**
 * Hình dạng lỗi qua ranh giới IPC — AD-21, **hợp đồng nguyên văn**.
 *
 * Bốn tên trường viết đúng `snake_case` như trên dây. Phía Rust (`core::i18n`)
 * KHÔNG đặt `#[serde(rename_all = "camelCase")]` lên struct này; nếu ai đó thêm vào,
 * `message_key` thành `messageKey`, mọi chỗ đọc theo AD-21 nhận `undefined`, và
 * TypeScript ở đây không hề biết. Test `ipc_error_wire_shape` phía Rust là thứ giữ
 * hai đầu khớp nhau — không phải kiểu này.
 *
 * ⚠️ `code` và `message_key` được phép 1:1 hôm nay nhưng là HAI trường, không phải
 * một trường hai tên: rẽ nhánh trên `code`, hiển thị `message_key`.
 * `code` không bao giờ được đưa ra màn hình.
 *
 * ⚠️ `retryable` chỉ là **quyền hiển thị một nút thử lại**. Không mã nào được tự
 * thử lại khi thấy `true` — AD-22 cấm auto-retry, và với BYOK nó là tính tiền hai lần.
 */
export type IpcError = {
  code: string
  message_key: string
  params: Record<string, string>
  retryable: boolean
}

/** Phân giải một khoá chấm thành chuỗi hiển thị (AC1). */
export const t = createResolver(catalog)

/**
 * Khoá dự phòng cuối cùng của AD-21 — mọi lỗi chưa phân loại được rơi vào đây thay
 * vì rơi vào một chuỗi viết tay ở chỗ gọi.
 */
const FALLBACK_KEY = 'err.unknown'

/**
 * ⚠️ Dedupe cảnh báo fallback — cùng lý lẽ với hai `Set` của `createResolver`.
 *
 * `resolve.ts` dựng hẳn hai `Set` để một khoá thiếu trong template Vue không ghi lại
 * cảnh báo ở MỖI LƯỢT RENDER. Nhánh này bỏ qua cả hai `Set` đó vì nó cảnh báo TRƯỚC
 * khi gọi `t()` — nên nếu không có `Set` riêng, đúng lớp lũ log ấy quay lại qua cửa
 * sau: một lỗi hiển thị liên tục ghi một dòng mỗi lần render và mọi cảnh báo thật
 * chìm mất. Khoá dedupe là `code` để hai lỗi khác nhau vẫn nói được thành hai dòng.
 */
const warnedErrors = new Set<string>()

/**
 * Phân giải nguyên payload lỗi của AD-21 thành chuỗi hiển thị (AC3 + AC4).
 *
 * Payload đến từ bên kia ranh giới IPC, nên nó được đối xử như dữ liệu không tin
 * được: một `message_key` vắng mặt hoặc rỗng KHÔNG được ném — nó rơi về
 * `err.unknown` và ghi cảnh báo. Cùng tinh thần AC4: hỏng thì hiện ra, đừng sập.
 *
 * ⚠️ Tham số thứ hai `params` KHÔNG có trong khung Task 2 của story (`tError(err)`).
 * Nó tồn tại cho chỗ gọi cần nội suy dữ liệu mà payload không mang — và vì `t` đã là
 * hàm công khai, nó không mở thêm quyền gì. Ghi ra đây thay vì để nó lặng lẽ.
 */
export function tError(err: IpcError, params?: MessageParams): string {
  const key = typeof err?.message_key === 'string' ? err.message_key.trim() : ''
  if (key === '') {
    const id = typeof err?.code === 'string' ? err.code : '(không có code)'
    if (!warnedErrors.has(id)) {
      warnedErrors.add(id)
      console.warn(
        `[i18n] payload lỗi (code "${id}") thiếu \`message_key\` — dùng khoá dự phòng ${FALLBACK_KEY}`,
      )
    }
    return t(FALLBACK_KEY)
  }
  return t(key, params ?? err.params)
}
