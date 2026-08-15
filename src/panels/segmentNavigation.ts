/**
 * Điều hướng segment — **module THUẦN**. Story 2.5b, AC12 · Task 6.2 · Task 11.2.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ **KHÔNG `import` GIÁ TRỊ NÀO, KHÔNG VUE, KHÔNG DOM** — cùng luật và cùng lý do
 * `./editorSegments.ts` đã ghi
 * ─────────────────────────────────────────────────────────────────────────────
 * Một `import` giá trị *(kể cả `../config/segment`, vốn kéo `@tauri-apps/api`)* làm tệp này
 * hết nạp được bằng **Node trần**, tức hết kiểm được bằng một cổng đọc-tệp. Chỉ `import type`.
 *
 * 🔴 **VÌ SAO PHÉP CHỌN SỐNG Ở ĐÂY, KHÔNG TRONG `GridPanel.vue`** — AD-1 và một phép đo.
 * *"Câu chưa dịch kế tiếp"* là một **vị từ trên dữ liệu**, không một hành vi hiển thị. Để nó
 * trong component nghĩa là nó chỉ nghiệm thu được bằng cách mount component — mà kho này
 * **không có một test mount component nào** *(§Cổng nào sẽ nhìn story này)*. Ở đây nó là một
 * hàm nhận một mảng và trả một `id`, tức kiểm được tất định và tức thời.
 */
import type { ChapterSegment } from '../config/segment'

/**
 * Dữ kiện — và **chỉ** những dữ kiện — mà phép chọn cần.
 *
 * ⚠️ Không nhận nguyên `ChapterSegment`: văn bản **đang gõ** *(chưa flush)* không nằm trong
 * hàng dữ liệu, và bỏ qua nó sẽ làm `⌥↓` nhảy **vào chính câu người dùng vừa gõ xong** —
 * đúng ca thường nhất của phím này.
 */
export type NavigationSegment = {
  readonly id: number
  /** `segment.status`. **Chuỗi**, không một enum: nó đi qua dây và được kiểm kiểu lúc chạy. */
  readonly status: string
  /** Văn bản đích **hiệu lực** — bản đang gõ nếu có, không thì bản lúc nạp. */
  readonly targetText: string
  /** `segment.retired_at`. `null` cho mọi segment cho tới Story 2.8. */
  readonly retiredAt: string | null
  /**
   * `segment.is_omitted` — câu đã bị **cắt bỏ khỏi bản dịch** (FR133, Story 2.5c).
   *
   * 🔴 Một **trục độc lập** với [`isUntranslated`], không một vế của nó. Xem khối lý do ở
   * [`nextUntranslatedId`].
   */
  readonly isOmitted: boolean
}

/**
 * 🔴 **ĐỊNH NGHĨA CỦA *"CHƯA DỊCH"*, VÀ NÓ CÓ ĐÚNG MỘT CHỖ** (AC12).
 *
 * `status === 'draft'` **VÀ** `targetText === ''` — **hai vế, không một**.
 *
 * ⚠️ Vế thứ hai là thứ dễ bị bỏ, và bỏ nó là một khuyết tật đo được: từ Story 2.5b,
 * `'draft'` **đã tách khỏi** *"chưa dịch"*. Một câu đã gõ xong mà chưa ai bấm xác nhận vẫn
 * mang `status = 'draft'` — nó là **bản nháp**, không phải **chưa dịch**. Một `⌥↓` chỉ đọc
 * `status` sẽ đưa người dùng quay lại đúng những câu họ vừa dịch, tức phím này **vô dụng**
 * ngay ở Chương đầu tiên.
 *
 * ⚠️ Vế thứ nhất cũng không thừa: một câu **đã xác nhận** rồi bị xoá trắng bản dịch *(ca
 * hiếm nhưng có thật)* mang `status = 'confirmed'` với `targetText === ''`. Nó **không** là
 * *"chưa dịch"* — nó là một câu đã ký đang hỏng, và đó là việc của một story khác.
 *
 * 🔴 Chuỗi `'draft'` viết **THẲNG** ở đây, không `import` từ `../config/segment` — điều kiện
 * kỹ thuật của luật "module thuần" ở đầu tệp. Cái giá, ghi ra: chuỗi này nay sống ở **ba**
 * chỗ *(cùng `editorSegments.ts` và `config/segment.ts`)*. Lưới cho chỗ hở đó là
 * `tests/frontend/segmentNavigation.test.ts`, ca *"ba chỗ đọc trạng thái phải đồng ý"*.
 */
export function isUntranslated(segment: NavigationSegment): boolean {
  return segment.status === 'draft' && segment.targetText === ''
}

/**
 * `segment.id` của **câu chưa dịch kế tiếp** sau `fromId`, hoặc `null`.
 *
 * @param segments theo `ord` — thứ tự do Rust quyết, hàm này **không** sắp lại.
 * @param fromId câu đang đứng. `null` ⇒ tìm từ **đầu Chương** *(ca người dùng vừa mở Chương
 *   và chưa đặt con trỏ vào đâu)*.
 *
 * 🔴 **KHÔNG quay vòng về đầu.** Hết Chương thì trả `null`, và chỗ gọi để con trỏ **ở
 * nguyên**. Một lượt quay vòng im lặng đưa người dùng về đầu Chương mà không dấu hiệu nào —
 * họ đọc thành *"phím này nhảy lung tung"*. Một phím không làm gì ở cuối danh sách là câu trả
 * lời trung thực: **không còn câu nào chưa dịch ở phía dưới**.
 *
 * ⚠️ Segment **đã về hưu** bị bỏ qua ở cả hai vai *(không phải đích, và không chặn đường)*:
 * một câu đã về hưu sau lượt gộp/tách không còn là câu người dùng làm việc trên đó (Story 2.8).
 *
 * ⚠️ Segment **đã cắt bỏ** cũng vậy — AC6 của Story 2.5c (FR133). Người dùng vừa quyết định
 * câu đó **không thuộc bản dịch**; dừng con trỏ ở đó là dẫn họ tới đúng chỗ họ vừa bảo bỏ đi.
 *
 * 🔴 **VÌ SAO LỌC Ở ĐÂY CHỨ KHÔNG NHÉT VÀO [`isUntranslated`]** — tiền lệ đã chốt cho "về
 * hưu", và cờ cắt bỏ đi đúng đường đó. *"Đã cắt bỏ"* và *"chưa dịch"* là **hai mệnh đề khác
 * nhau**: một câu đã cắt bỏ mà chưa dịch **vẫn là** chưa dịch — nó chỉ không phải **đích của
 * phím này**. Gộp chúng lại làm mọi chỗ đọc *"chưa dịch"* trong tương lai đếm sai, và chỗ
 * đầu tiên trả giá là thanh tiến độ Tác phẩm (FR, Story 5.5).
 */
export function nextUntranslatedId(
  segments: readonly NavigationSegment[],
  fromId: number | null,
): number | null {
  // `-1` khi `fromId` là `null` HOẶC không tìm thấy — cả hai đều nghĩa là *"bắt đầu từ đầu"*.
  // ⚠️ Gộp hai ca có chủ ý: một `fromId` trỏ vào một câu vừa bị gộp mất là ca thật (Story
  // 2.8), và nó phải cho ra một hành vi có nghĩa thay vì không hành vi nào.
  const from = fromId === null ? -1 : segments.findIndex((s) => s.id === fromId)
  for (let i = from + 1; i < segments.length; i += 1) {
    const s = segments[i]
    if (s.retiredAt !== null) continue
    if (s.isOmitted) continue
    if (isUntranslated(s)) return s.id
  }
  return null
}

/**
 * Dựng dữ kiện cho [`nextUntranslatedId`] từ một hàng đã nạp cộng tập chờ đang gõ.
 *
 * 🔴 `editedText` đi vào **ở đây**, không trong `nextUntranslatedId`: hàm kia là một vị từ
 * thuần trên dữ liệu, và trộn một `Map` state vào chữ ký của nó là kéo một khái niệm của
 * tầng giao diện xuống tầng thuần.
 */
export function navigationSegmentOf(
  segment: ChapterSegment,
  editedText: ReadonlyMap<number, string>,
): NavigationSegment {
  return {
    id: segment.id,
    status: segment.status,
    targetText: editedText.get(segment.id) ?? segment.target_text,
    retiredAt: segment.retired_at,
    isOmitted: segment.is_omitted,
  }
}
