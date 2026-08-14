/**
 * 🔴 **BẢNG ÁNH XẠ *trạng thái segment → giá trị vạch lề*** — Story 2.2, AC3 · AC12 ·
 * Quyết định #4(b).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO CẢ NĂM NHÁNH ĐƯỢC CÀI HÔM NAY, TRONG KHI BA NHÁNH CHƯA CÓ NGUỒN DỮ LIỆU
 * ─────────────────────────────────────────────────────────────────────────────
 * `EXPERIENCE.md:99` nói năm giá trị vạch là một **tài nguyên hữu hạn đã tiêu hết** — đó
 * chính là lý do UX-DR22 buộc phát hiện Proofreader phải đi đường gạch chân lượn sóng thay
 * vì xin một giá trị vạch thứ sáu. Một bảng ánh xạ như vậy là một **hợp đồng**, và một hợp
 * đồng cài nửa vời là chỗ để story sau chép sai: 2.5 sẽ thêm nhánh `confirmed` ở một tệp,
 * 2.8 thêm `ornament` ở một tệp khác, và không ai còn đọc được cả năm ở một chỗ.
 *
 * ⇒ Cả năm nhánh sống ở **một** hàm phân giải duy nhất. Ba nhánh chưa có nguồn dữ liệu đọc
 * từ ba trường mà hôm nay **không đường nào bật lên được**, và mỗi trường ghi **đích danh
 * story chủ** của nó. Story chủ chỉ phải nối nguồn, không phải sửa tầng hiển thị.
 *
 * ⚠️ **KHÔNG dữ liệu giả.** Ba nhánh kia chạy trong **bàn đo** (§Task 7 của story), không
 * trong `.atproj` của người dùng.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ TỆP NÀY LÀ **MODULE THUẦN** — KHÔNG `import` GIÁ TRỊ NÀO, KHÔNG VUE, KHÔNG DOM
 * ─────────────────────────────────────────────────────────────────────────────
 * Đó là điều kiện để `scripts/check-commands.mjs` `import()` **thẳng hàm thật** ở đây và
 * chạy nó bằng **Node thuần**. Một `import` giá trị *(kể cả `../config/segment`, vốn kéo
 * `@tauri-apps/api`)* giết phép kiểm đó ngay. State của panel sống ở
 * `./editorPanelState.ts`; hình học sống ở `./editorGutter.ts`.
 *
 * 🔵 **CẬP NHẬT 2026-08-13 — sửa một mệnh đề đã hết đúng.** Bản trước gọi phép kiểm trên là
 * *"đường thay cho một bộ chạy test frontend mà dự án cố ý không có (NFR15)"*. Kho **nay CÓ**
 * bộ chạy đó (`vitest` + `@vue/test-utils` + `happy-dom`, cây test ở `tests/frontend/**`,
 * Story 2.3 — 2026-08-12). ⇒ Cổng ở đây **không còn là đường thay thế**: nó canh vế **khai
 * báo trên toàn cây** *(đếm `SEGMENT_RULE_VALUES`, đỏ ở giá trị thứ sáu)*, một hạng kiểm
 * **khác** với một test khẳng định hàm trả đúng giá trị — và hai đường đó không được chồng
 * nhau (AC25). **Luật "không `import` giá trị" thì KHÔNG đổi một chữ**: nó là điều kiện để
 * cổng chạy được bằng Node thuần, và điều kiện đó độc lập với việc có vitest hay không.
 */
import type { ChapterSegment } from '../config/segment'

/**
 * 🔴 **ĐÚNG NĂM GIÁ TRỊ, VÀ CON SỐ NĂM LÀ MỘT MỆNH ĐỀ NGHIỆM THU** (AC12).
 *
 * `scripts/check-commands.mjs` (Kiểm *"vạch lề segment"*) đếm mảng này và **đỏ** ở giá trị
 * thứ sáu. Đừng thêm một giá trị để "tạm phân biệt" một trạng thái mới — `DESIGN.md:380` và
 * `EXPERIENCE.md:99` đã tiêu hết năm giá trị, và kênh thị giác kế tiếp là gạch chân lượn
 * sóng (UX-DR22), không phải một màu vạch nữa.
 *
 * ⚠️ `'none'` **là** một trong năm, không phải "không có giá trị": *"chưa dịch ⇒ không vạch"*
 * là một mệnh đề hiển thị có chủ, và gộp nó vào `null` sẽ làm phép đếm của cổng đọc ra bốn.
 */
export const SEGMENT_RULE_VALUES = ['confirmed', 'primary', 'tm-rule', 'none', 'ornament'] as const

/** Một trong năm giá trị vạch lề. */
export type SegmentRuleValue = (typeof SEGMENT_RULE_VALUES)[number]

/**
 * Những dữ kiện — và **chỉ** những dữ kiện — mà phép phân giải cần.
 *
 * ⚠️ Không nhận nguyên `ChapterSegment`: hai trong năm nhánh không đọc từ hàng `segment` nào
 * *(tiêu điểm sống ở DOM; `tm-rule` sẽ tới từ tầng TM của Epic 7)*, nên một chữ ký nhận hàng
 * dữ liệu sẽ buộc phải nói dối ở đúng hai chỗ đó.
 */
export type SegmentRuleInput = {
  /**
   * `segment.retired_at` — `null` cho mọi segment hôm nay.
   *
   * 🔴 Cột **đã có** từ Story 2.1, nhưng **chưa đường nào cho segment về hưu**.
   * **Chủ: Story 2.8** *(gộp/tách segment)*.
   */
  retiredAt: string | null
  /** Con trỏ / tiêu điểm bàn phím đang chạm câu này. Nguồn: DOM, xem `./editorPanelState.ts`. */
  hasCaret: boolean
  /**
   * Người dùng đã xác nhận câu này (FR24).
   *
   * 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5) — mệnh đề cũ đã hết đúng.** Bản trước viết *"hôm
   * nay luôn `false`: cột `segment.status` chưa tồn tại"*. Cột **đã có** (bước di trú 7), và
   * trường này nay đọc dữ liệu **thật** qua `isSegmentConfirmed` — xem [`segmentRuleInputOf`].
   *
   * ⚠️ Máy trạng thái sinh ra giá trị này sống **ở Rust** (`commands/segment.rs`), AD-1. Chỗ
   * này chỉ **đọc** nó.
   */
  isConfirmed: boolean
  /**
   * Câu được điền sẵn từ một khớp TM 100% và **chưa ai xác nhận** (FR58).
   *
   * 🔴 Hôm nay **luôn `false`**: chưa có tầng TM nào. **Chủ: Epic 7**.
   */
  isTmFilled: boolean
  /** `segment.target_text`. **Chuỗi rỗng = chưa dịch** ⇒ nhánh *không vạch*. */
  targetText: string
}

/**
 * 🔴 **THỨ TỰ CỦA NĂM NHÁNH LÀ MỘT QUYẾT ĐỊNH, KHÔNG PHẢI THỨ TỰ GÕ RA.**
 *
 * 1. `ornament` **thắng tất cả** — một câu đã về hưu không còn là câu người dùng đang làm
 *    việc trên đó; vẽ nó thành *"đang sửa"* là một lời nói dối về chính thao tác vừa xảy ra.
 * 2. `primary` thắng `confirmed` — `DESIGN.md:380` định nghĩa nó là *"đang sửa, con trỏ ở
 *    đây"*, tức một mệnh đề về **hiện tại**; trạng thái đã xác nhận là mệnh đề về quá khứ, và
 *    vạch chỉ có một chỗ để nói.
 * 3. `confirmed` thắng `tm-rule` — một câu TM điền sẵn rồi được xác nhận **không còn** là
 *    *"máy đề xuất, chưa ai xác nhận"* (`EXPERIENCE.md:99`).
 * 4. `tm-rule` thắng *không vạch* — nó có bản dịch, chỉ là chưa ai ký.
 * 5. Còn lại ⇒ *không vạch*.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MỘT KHE HỞ CÓ THẬT TRONG BẢNG NĂM GIÁ TRỊ, GHI RA THAY VÌ GIẤU
 * ─────────────────────────────────────────────────────────────────────────────
 * Một câu **đã dịch bằng tay**, **chưa xác nhận**, **con trỏ ở chỗ khác** không ứng với giá
 * trị nào trong năm: `confirmed` sai *(chưa ai ký)*, `tm-rule` sai *(không phải máy điền)*,
 * *không vạch* sai *(nó đã có bản dịch)*. Bảng của `EXPERIENCE.md:105-113` đơn giản không
 * có hàng đó.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ✅ **ĐÃ PHÂN XỬ 2026-08-14 — Ice ký Quyết định #3 của Story 2.5, đường (a).**
 * ─────────────────────────────────────────────────────────────────────────────
 * Khe hở **không còn** là một món nợ: hàng đó nay có một câu trả lời viết ra, và câu trả lời
 * là *"không vạch"* — **có chủ ý**, không phải một nhánh quên viết.
 *
 * **Mệnh đề đã ký:** *"đã dịch, chưa xác nhận ⇒ không vạch — vạch chỉ nói **ai đã ký**,
 * không nói **có chữ hay chưa**."* Nó khớp `DESIGN.md:380`, nơi vạch lề được định nghĩa là
 * chỗ đọc **trạng thái xác nhận**.
 *
 * **Cái mất, ghi ra thay vì giấu:** người dùng **không** phân biệt được *chưa dịch* với *đã
 * dịch chưa ký* bằng vạch lề. Nó chấp nhận được vì cái mất đó đã có một kênh khác chở —
 * **văn bản có chữ** là chỉ báo *"đã dịch"* rõ hơn bất kỳ vạch nào, và nó nằm ngay cạnh.
 *
 * ⚠️ Hai đường kia bị **loại**, và mỗi đường bị loại bởi một mệnh đề khác nhau: mượn
 * `tm-rule` phá nghĩa cố định *"máy đề xuất, chưa ai xác nhận"* (`EXPERIENCE.md:97,101`) và
 * làm hỏng cả Proofreader (FR81) lẫn ranh giới bóc; một giá trị **thứ sáu** phá mệnh đề
 * *"năm giá trị là tài nguyên hữu hạn đã tiêu hết"* (`EXPERIENCE.md:99`) và làm Kiểm I đỏ.
 *
 * 🔴 Từ story này khe hở **chạm tới được**: gõ xong một câu rồi bấm sang câu khác mà chưa
 * xác nhận là ca **thường nhật nhất** của tính năng. Nhánh cuối của hàm dưới đây là chỗ nó
 * rơi về, và `tests/frontend/editorSegmentRule.test.ts` khoá mệnh đề đó lại.
 * 🔵 *(Code review 2026-08-14: đường dẫn cũ `tests/frontend/panels/editorSegments.test.ts`
 * KHÔNG tồn tại — nó còn bịa ra một thư mục con `panels/` mà cây test cố ý không dùng;
 * `tests/frontend/**` phẳng. Mệnh đề thì có bị khoá thật, chỉ là ở tệp khác tên.)*
 */
export function resolveSegmentRule(input: SegmentRuleInput): SegmentRuleValue {
  if (input.retiredAt !== null) return 'ornament'
  if (input.hasCaret) return 'primary'
  if (input.isConfirmed) return 'confirmed'
  if (input.isTmFilled) return 'tm-rule'
  // Chuỗi rỗng ⇒ chưa dịch. Chuỗi KHÔNG rỗng rơi vào đây cũng cho `'none'` — xem khe hở
  // ghi ở doc-comment trên; đó là một món nợ có chủ, không phải một nhánh quên viết.
  return 'none'
}

/**
 * Dựng dữ kiện cho [`resolveSegmentRule`] từ một hàng đã nạp.
 *
 * 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5).** Bản trước gọi đây là *"chỗ duy nhất trong sản phẩm
 * nói hai nguồn dữ liệu kia chưa tồn tại"*, bằng **hai** hằng `false`. Story 2.5 đã sửa
 * **đúng một** trong hai dòng đó: `isConfirmed` nay đọc `segment.status` **thật**. Hằng còn
 * lại là `isTmFilled` — **chủ: Epic 7** (FR58), và nó **ở lại**.
 */
export function segmentRuleInputOf(
  segment: ChapterSegment,
  caretSegmentId: number | null,
): SegmentRuleInput {
  return {
    retiredAt: segment.retired_at,
    hasCaret: caretSegmentId === segment.id,
    // 🔴 CHUỖI `'confirmed'` VIẾT THẲNG Ở ĐÂY, KHÔNG `import` TỪ `../config/segment` — và đó
    // là một **điều kiện kỹ thuật**, không một lượt lười.
    //
    // `scripts/check-commands.mjs` Kiểm I `import()` **thẳng tệp này** bằng Node trần để chạy
    // phép kiểm trên chính mã sản phẩm; `../config/segment` kéo theo `@tauri-apps/api`, nên
    // **một** dòng `import` giá trị ở đây giết phép kiểm đó — xem doc-comment đầu tệp.
    //
    // ⚠️ Cái giá, ghi ra: chuỗi này tồn tại ở **hai** chỗ *(chỗ kia là
    // `config/segment.ts::isSegmentConfirmed`, thứ các chỗ gọi KHÔNG bị ràng buộc "module
    // thuần" dùng)*. Lưới cho chỗ hở đó **không** phải kỷ luật: nó là
    // `tests/frontend/editorSegmentRule.test.ts`, ca *"hai chỗ đọc trạng thái phải đồng ý
    // với nhau"* — sai chính tả ở một trong hai chỗ làm ca đó ĐỎ.
    // 🔵 *(Code review 2026-08-14: sửa đường dẫn — bản cũ trỏ vào một tệp không tồn tại.)*
    isConfirmed: segment.status === 'confirmed',
    // 🔴 Epic 7 — chưa tầng TM nào (FR58). Hằng này **ở lại**.
    isTmFilled: false,
    targetText: segment.target_text,
  }
}

/**
 * Lớp CSS của một giá trị vạch — `null` cho *không vạch* ⇒ **không vẽ vạch nào**.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MỘT LỚP CSS, KHÔNG MỘT MÀU. ĐÂY LÀ ĐIỀU KIỆN ĐỂ CỔNG TOKEN CÒN NHÌN THẤY GÌ
 * ─────────────────────────────────────────────────────────────────────────────
 * Kiểm B của `check-tokens.mjs` đọc **CSS**, không đọc TypeScript. Một hàm trả
 * `'#5a6b3f'` — hay thậm chí `` `var(--color-${rule})` `` để `:style` bind vào — đi qua cổng
 * mà không một dòng nào được soi, tức mở đúng cái cửa hậu mà AD-34 tồn tại để đóng. Bốn màu
 * vạch vì thế khai trong `<style scoped>` của `EditorPanel.vue`, mỗi màu một khối
 * `background-color: var(--color-…)` mà cổng đọc được từng chữ.
 *
 * ⚠️ `scripts/check-commands.mjs` đối chiếu **hai chiều**: mỗi giá trị trong
 * [`SEGMENT_RULE_VALUES`] *(trừ `'none'`)* phải có đúng một khối `.rule-<giá trị>` trong
 * `EditorPanel.vue`, và ngược lại.
 */
export function ruleClassOf(rule: SegmentRuleValue): string | null {
  return rule === 'none' ? null : `rule-${rule}`
}
