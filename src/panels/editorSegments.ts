/**
 * 🔴 **BẢNG ÁNH XẠ *trạng thái segment → giá trị vạch lề*** — Story 2.2, AC3 · AC12 ·
 * Quyết định #4(b).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO CẢ SÁU NHÁNH ĐƯỢC CÀI HÔM NAY, TRONG KHI HAI NHÁNH CHƯA CÓ NGUỒN DỮ LIỆU
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 **2026-08-14 (Story 2.5b): NĂM → SÁU.** `EXPERIENCE.md:99` gọi năm giá trị vạch là một
 * **tài nguyên hữu hạn đã tiêu hết**, và đó vẫn là lý do UX-DR22 buộc phát hiện Proofreader
 * đi đường gạch chân lượn sóng. Giá trị thứ sáu (`draft`) **không** phá mệnh đề đó — nó lấp
 * một hàng vốn đã **thiếu** trong bảng, không xin một kênh thị giác cho một trạng thái mới.
 * Lý lẽ đầy đủ và **hai lượt ký theo thứ tự** ở doc-comment của [`resolveSegmentRule`].
 *
 * Một bảng ánh xạ như vậy là một **hợp đồng**, và một hợp đồng cài nửa vời là chỗ để story
 * sau chép sai: 2.5 sẽ thêm nhánh `confirmed` ở một tệp, 2.8 thêm `ornament` ở một tệp khác,
 * và không ai còn đọc được cả sáu ở một chỗ.
 *
 * ⇒ Cả sáu nhánh sống ở **một** hàm phân giải duy nhất. Hai nhánh chưa có nguồn dữ liệu đọc
 * từ ba trường mà hôm nay **không đường nào bật lên được**, và mỗi trường ghi **đích danh
 * story chủ** của nó. Story chủ chỉ phải nối nguồn, không phải sửa tầng hiển thị.
 *
 * ⚠️ **KHÔNG dữ liệu giả.** Hai nhánh kia chạy trong **bàn đo** (§Task 7 của story), không
 * trong `.atproj` của người dùng.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ TỆP NÀY LÀ **MODULE THUẦN** — KHÔNG `import` GIÁ TRỊ NÀO, KHÔNG VUE, KHÔNG DOM
 * ─────────────────────────────────────────────────────────────────────────────
 * Đó là điều kiện để `scripts/check-commands.mjs` `import()` **thẳng hàm thật** ở đây và
 * chạy nó bằng **Node thuần**. Một `import` giá trị *(kể cả `../config/segment`, vốn kéo
 * `@tauri-apps/api`)* giết phép kiểm đó ngay. State của panel sống ở
 * `./editorPanelState.ts`.
 * 🔵 *(2026-08-15 — mệnh đề "hình học sống ở `./editorGutter.ts`" đã hết đúng: tệp đó bị GỠ ở
 * Story 2.5b. Trong lưới, vạch lấy chiều cao từ **track hàng** của `subgrid`, nên không còn
 * hình học nào để tính. Phép đo của nó sống tiếp trong `deferred-work.md`.)*
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
 * 🔴 **ĐÚNG SÁU GIÁ TRỊ, VÀ CON SỐ SÁU LÀ MỘT MỆNH ĐỀ NGHIỆM THU** (AC4 của Story 2.5b).
 *
 * `scripts/check-commands.mjs` (Kiểm *"vạch lề segment"*) đếm mảng này và **đỏ** ở giá trị
 * thứ bảy.
 *
 * 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5b) — con số NĂM đã hết đúng, và lý do thì KHÔNG đổi.**
 * Bản trước viết: *"đừng thêm một giá trị để 'tạm phân biệt' một trạng thái mới —
 * `DESIGN.md:380` và `EXPERIENCE.md:99` đã tiêu hết năm giá trị, và kênh thị giác kế tiếp là
 * gạch chân lượn sóng (UX-DR22), không phải một màu vạch nữa."*
 *
 * Câu đó vẫn đúng **cho một trạng thái mới**. Thứ xảy ra ở 2.5b **không** phải một trạng thái
 * mới: nó là **một hàng đã thiếu sẵn trong bảng** — *"đã dịch tay, chưa xác nhận, con trỏ ở
 * chỗ khác"* — được ghi ra ở khối [`resolveSegmentRule`] dưới đây từ Story 2.2 dưới nhãn
 * *"MỘT KHE HỞ CÓ THẬT"*. UX-DR19 **viết lại 2026-08-14** cấp cho hàng đó một giá trị riêng,
 * và Ice ký Quyết định #2 đường (a) cùng ngày. ⇒ Đây là lượt **lấp một hàng trống**, không
 * một lượt xin thêm kênh thị giác. UX-DR22 **không bị đụng**: Proofreader vẫn đi gạch chân
 * lượn sóng.
 *
 * ⚠️ `'none'` **là** một trong sáu, không phải "không có giá trị": *"chưa dịch ⇒ không vạch"*
 * là một mệnh đề hiển thị có chủ, và gộp nó vào `null` sẽ làm phép đếm của cổng đọc ra năm.
 */
export const SEGMENT_RULE_VALUES = [
  'confirmed',
  'primary',
  'tm-rule',
  'draft',
  'none',
  'ornament',
] as const

/** Một trong sáu giá trị vạch lề. */
export type SegmentRuleValue = (typeof SEGMENT_RULE_VALUES)[number]

/**
 * Những dữ kiện — và **chỉ** những dữ kiện — mà phép phân giải cần.
 *
 * ⚠️ Không nhận nguyên `ChapterSegment`: hai trong sáu nhánh không đọc từ hàng `segment` nào
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
 * 🔴 **THỨ TỰ CỦA SÁU NHÁNH LÀ MỘT QUYẾT ĐỊNH, KHÔNG PHẢI THỨ TỰ GÕ RA.**
 *
 * 1. `ornament` **thắng tất cả** — một câu đã về hưu không còn là câu người dùng đang làm
 *    việc trên đó; vẽ nó thành *"đang sửa"* là một lời nói dối về chính thao tác vừa xảy ra.
 * 2. `primary` thắng `confirmed` — `DESIGN.md:380` định nghĩa nó là *"đang sửa, con trỏ ở
 *    đây"*, tức một mệnh đề về **hiện tại**; trạng thái đã xác nhận là mệnh đề về quá khứ, và
 *    vạch chỉ có một chỗ để nói.
 * 3. `confirmed` thắng `tm-rule` — một câu TM điền sẵn rồi được xác nhận **không còn** là
 *    *"máy đề xuất, chưa ai xác nhận"* (`EXPERIENCE.md:99`).
 * 4. `tm-rule` thắng `draft` — cả hai đều nói *"có chữ, chưa ai ký"*, nhưng `tm-rule` nói
 *    thêm **ai viết chữ đó**. Một câu máy điền rồi người sửa tay vẫn là một câu **máy khởi
 *    xướng**, và đó là thứ FR58 cần đọc được.
 * 5. `draft` thắng *không vạch* — nó **có** bản dịch, chỉ là do tay người dùng và chưa ai ký.
 * 6. Còn lại ⇒ *không vạch*, và ở đây *"còn lại"* nghĩa là **`target_text` rỗng**.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 **KHE HỞ CỦA BẢNG NĂM GIÁ TRỊ ĐÃ ĐÓNG — 2026-08-14, Story 2.5b.** Ghi cả hai lượt ký
 * và ghi THỨ TỰ của chúng: xoá dấu vết quyết định cũ là lấy mất bằng chứng của quyết định mới.
 * ─────────────────────────────────────────────────────────────────────────────
 * **Khe hở** *(ghi từ Story 2.2)*: một câu **đã dịch bằng tay**, **chưa xác nhận**, **con trỏ
 * ở chỗ khác** không ứng với giá trị nào trong năm — `confirmed` sai *(chưa ai ký)*,
 * `tm-rule` sai *(không phải máy điền)*, *không vạch* sai *(nó đã có bản dịch)*. Bảng của
 * `EXPERIENCE.md:105-113` đơn giản không có hàng đó.
 *
 * | Lượt | Ngày | Mệnh đề | Trạng thái |
 * |---|---|---|---|
 * | ① Quyết định #3 của Story 2.5 | 2026-08-14 | *"đã dịch, chưa xác nhận ⇒ **không vạch** — vạch chỉ nói **ai đã ký**, không nói **có chữ hay chưa**"* | **HẾT HIỆU LỰC** |
 * | ② UX-DR19 viết lại + Quyết định #2 của Story 2.5b | 2026-08-14 *(cùng ngày, **sau** ①)* | *"đã dịch tay, chưa ký ⇒ vạch **`draft`**"* | **ĐANG HIỆU LỰC** |
 *
 * 🔴 **Vì sao ② lật ①, và nó KHÔNG có nghĩa ① sai lúc được ký.** ① đứng trên một tiền đề đo
 * được: *"cái mất đã có một kênh khác chở — **văn bản có chữ** là chỉ báo 'đã dịch' rõ hơn
 * bất kỳ vạch nào, và nó nằm ngay cạnh."* Tiền đề đó đúng cho **trang văn liền mạch** của
 * Story 2.2/2.3, nơi mắt đọc chữ và vạch trên cùng một dòng. Lượt correct-course
 * 2026-08-14 lật bề mặt thành **lưới**: nay có một cột bản dịch riêng, một cột nhãn trạng
 * thái riêng, và một cột vạch ở **mép trái** — cách chỗ chữ cả một cột nguyên văn. *"Nằm
 * ngay cạnh"* hết đúng theo **hình học**, không theo lập luận.
 *
 * ⚠️ Và lý do ② được phép xin một giá trị thứ sáu, trong khi `EXPERIENCE.md:99` gọi năm giá
 * trị là *"tài nguyên hữu hạn đã tiêu hết"*: đây là lượt lấp **một hàng vốn đã thiếu**, không
 * một trạng thái mới xin chỗ. Xem [`SEGMENT_RULE_VALUES`].
 */
export function resolveSegmentRule(input: SegmentRuleInput): SegmentRuleValue {
  if (input.retiredAt !== null) return 'ornament'
  if (input.hasCaret) return 'primary'
  if (input.isConfirmed) return 'confirmed'
  if (input.isTmFilled) return 'tm-rule'
  // 🔵 2026-08-14 (Story 2.5b) — NHÁNH MỚI, và nó lấp đúng khe hở ghi ở doc-comment trên.
  //
  // 🔴 Vị từ là **`targetText !== ''`**, không một cờ dirty và không một phép so với bản lúc
  // nạp. Lý do: `draft` phải đúng cho CẢ hai đường tới — người dùng vừa gõ trong phiên này,
  // VÀ một câu đã có chữ từ một phiên trước rồi nạp lên. Một cờ dirty chỉ biết đường thứ
  // nhất, nên nó sẽ vẽ một Chương đang dịch dở thành *"chưa dịch"* ngay sau lượt mở lại.
  if (input.targetText !== '') return 'draft'
  // Còn lại ⇒ `target_text` RỖNG, tức **chưa dịch**. Đây nay là nhánh duy nhất cho `'none'`.
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
 * mà không một dòng nào được soi, tức mở đúng cái cửa hậu mà AD-34 tồn tại để đóng. Năm màu
 * vạch vì thế khai trong `<style scoped>` của `GridPanel.vue`, mỗi màu một khối
 * `background-color: var(--color-…)` mà cổng đọc được từng chữ.
 *
 * ⚠️ `scripts/check-commands.mjs` đối chiếu **hai chiều**: mỗi giá trị trong
 * [`SEGMENT_RULE_VALUES`] *(trừ `'none'`)* phải có đúng một khối `.rule-<giá trị>` trong
 * `GridPanel.vue`, và ngược lại.
 * 🔵 *(2026-08-14 — tệp đích đổi từ `EditorPanel.vue` sang `GridPanel.vue` cùng lượt lật
 * hình dạng của Story 2.5b. Cổng đọc đường dẫn đó từ một hằng, xem `EDITOR_PANEL_VUE`.)*
 */
export function ruleClassOf(rule: SegmentRuleValue): string | null {
  return rule === 'none' ? null : `rule-${rule}`
}

/**
 * 🔴 **Đổi một vị trí `(node, offset)` trong một ô nguyên văn thành CHỈ SỐ KÝ TỰ trong
 * `source_text`** — Story 2.8, AC2. Trả `null` khi `node` không thuộc `cell`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO KHÔNG LẤY `offset` TRẦN — hôm nay hai cách cho cùng số, và đó là cái bẫy
 * ─────────────────────────────────────────────────────────────────────────────
 * Một ô `[data-col="src"]` chứa **nhiều** text node. Đo trên WKWebView 605.1.15 thật ngày
 * 2026-08-17 *(bàn đo `2-8-ban-do/`, bước ⓪)*: `childNodes` là
 * `[COMMENT, TEXT(0), COMMENT, TEXT(40), TEXT(0)]` — hai `#comment` là chú thích
 * `aura-allow-text` của template, và chúng chia chuỗi làm ba mảnh.
 *
 * `offset` mà `caretPositionFromPoint` trả về đếm **trong một node**, không trong ô. Hôm nay
 * tổng độ dài các node đứng **trước** node văn bản là **0**, nên `offset` trần **bằng** chỉ
 * số ký tự — và đó chính là lý do một lượt lấy trần sẽ đi qua mọi phép kiểm hôm nay rồi
 * **sai im lặng** vào đúng ngày người dùng bật Hán Việt: ô mang thêm `<ruby>`/`<rt>`, tổng
 * kia khác 0, và chỗ cắt lệch đi vài ký tự **trên dữ liệu người dùng** — mà AD-5 không cho
 * hoàn tác.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 2026-08-17, CODE REVIEW — HÀM NÀY ĐÃ SAI HAI LẦN, HAI NGUYÊN NHÂN KHÁC NHAU
 * ─────────────────────────────────────────────────────────────────────────────
 * Cả hai cho **cùng một** biểu hiện: `⌘/` cắt đúng chỗ người dùng **không** bấm, im lặng,
 * trên dữ liệu mà AD-5 không cho hoàn tác, và **không cổng nào đỏ**.
 *
 * **① Đếm nhầm NODE — `<rt>` bị cộng vào.** `TreeWalker(SHOW_TEXT)` gặp **mọi** text node
 * trong ô. Khi Hán Việt bật, `SourceHanViet.vue` dựng `<ruby>{chữ Hán}<rt>{âm}</rt></ruby>`
 * — và `<rt>` **là một text node**, dù nó không phải một ký tự nào của `source_text`. Đo
 * bằng `happy-dom` trên đúng hình dạng DOM sản phẩm *(`<ruby>京<rt>kinh</rt></ruby>都です。`)*:
 * cú bấm đáng lẽ cho chỉ số **1** trả về **5** — lệch đúng độ dài của âm.
 * 🔴 **Kho này đã đo và ghi ra đúng cái bẫy đó từ 2026-08-07** — `SourceHanViet.vue` §*"`ruby.
 * textContent` GỘP CẢ `<rt>`"*. Bẫy có tên rồi mà hàm này vẫn đi vào; và ca test của nó dựng
 * một `<ruby>` **không có `<rt>`** nên phép kiểm **không thể đỏ**. Đúng luật *"đừng bắt chước
 * một ký hiệu chưa hiểu"* ở chiều ngược lại: đừng đi ngang một mệnh đề đã đo mà không đọc.
 *
 * **② Đếm nhầm ĐƠN VỊ — UTF-16 code unit, trong khi Rust đếm code point.** `String.length`
 * và mọi offset của DOM Range đếm **code unit**; `regroup.rs::split_at` đếm `chars()`, tức
 * **code point**. Hai đơn vị chỉ trùng khi mọi ký tự đứng trước chỗ cắt nằm trong BMP. Một
 * ký tự ngoài BMP — CJK Extension B (U+20000+), thứ **có thật** trong văn bản Hán cổ và tên
 * riêng — là **2** đơn vị JS nhưng **1** `char` Rust, nên chỗ cắt lệch phải đúng bằng số ký
 * tự astral đứng trước, và thường **vẫn trong biên** nên không lỗi nào ném.
 *
 * ⇒ Cả hai vá ở đây, tại **nguồn** của con số, không bằng một phép chỉnh ở tầng dưới.
 *
 * ⚠️ Hàm sống ở đây, **không** trong `GridPanel.vue`: một mệnh đề về **phép ánh xạ** kiểm
 * được bằng `vitest` + `happy-dom`, còn một mệnh đề nằm trong thân một handler chuột thì chỉ
 * kiểm được bằng e2e. Cùng lý do `segmentNavigation.ts` tồn tại.
 */
export function sourceCutOffsetOf(cell: Element, node: Node, offsetInNode: number): number | null {
  if (!cell.contains(node)) return null
  // 🔴 Một text node nằm dưới `<rt>` là **ÂM HÁN VIỆT**, không một ký tự nào của
  // `source_text`. Giữ lại dù §🔵 2026-08-17 làm nó **thừa theo cấu tạo** *(`<rt>` không mang
  // neo, nên nhánh dưới đã trả `null`)*: nó là một chốt rẻ cho đúng cái bẫy đã lọt một lần,
  // và một hàng rào thừa ở đây không tốn gì.
  if (duoiRt(cell, node)) return null

  // ── ĐỌC NEO — §🔵 2026-08-17 ────────────────────────────────────────────────────
  const neo = neoNguonCua(cell, node)
  if (neo === null) return null
  const batDau = Number(neo.getAttribute(SRC_START))
  if (!Number.isFinite(batDau)) return null

  // 🔴 **NGUYÊN KHỐI** — `.hv-word` ở kiểu `switch` (chữ ký của Ice, 2026-08-17). Thứ trên
  // màn hình là **ÂM**, không phải chữ Hán, nên một cú bấm ở đó nói được *"TỪ nào"* mà
  // không nói được *"ký tự nào trong từ"*. Cộng thêm offset ở đây là dựng một con số từ một
  // chuỗi **khác hẳn** `source_text` — đúng phép đếm vừa bị bác.
  if (neo.hasAttribute(SRC_ATOMIC)) return batDau

  // Phần tử mang **chính** ký tự nguồn *(mảnh văn bản thuần · `.hv-text` · base `<ruby>`)*
  // ⇒ đếm ký tự đứng trước caret **bên trong nó**, bỏ qua `<rt>`.
  //
  // 🔴 `offsetInNode` là một chỉ số **UTF-16** *(mọi offset của DOM Range đều thế)*, nên nó
  // phải đi qua `demKyTu` chứ không cộng thẳng — xem khối 🔵 vế ②.
  let truoc = 0
  const walker = cell.ownerDocument.createTreeWalker(neo, NodeFilter.SHOW_TEXT)
  let n = walker.nextNode()
  while (n !== null && n !== node) {
    if (!duoiRt(cell, n)) truoc += demKyTu(n.textContent)
    n = walker.nextNode()
  }
  // `node` không phải text node của neo *(vd chính phần tử neo)* ⇒ walker chạy hết mà không
  // gặp nó. Trả `batDau + truoc`, tức **cuối** phần tử — cùng ngữ nghĩa *"bấm vào khoảng
  // trống sau chữ"* mà đường chuột của ô bản dịch đã chọn.
  return batDau + truoc + (n === null ? 0 : demKyTu(n.textContent?.slice(0, offsetInNode)))
}

/** Thuộc tính neo: chỉ số ký tự nguồn nơi phần tử này bắt đầu. */
const SRC_START = 'data-src-start'
/**
 * Chỉ cắt được ở **đầu** phần tử này. **Hai** lý do khác nhau đều dùng cùng một neo:
 *
 * ① **Không mang ký tự nguồn trên màn hình** — `.hv-word` ở kiểu `switch`: thứ hiển thị là
 *    **ÂM** Hán Việt, nên một cú bấm nói được *"TỪ nào"* mà không nói được *"ký tự nào"*.
 *    Chữ ký của Ice, 2026-08-17.
 *
 * ② 🔵 **Không VẼ được dấu ở giữa** — mảnh `.hv-text` *(dấu câu, số, chữ Latin)*, thêm ở lượt
 *    code review 2026-08-17. Mảnh này **mang** chính ký tự nguồn nên phép đếm cho một offset
 *    **chính xác từng ký tự**, và đó lại là vấn đề: `SourceHanViet.vue` vẽ dấu cắt bằng
 *    `cutSet.has(seg.srcStart)` — so với điểm **ĐẦU** mảnh — nên một offset ở **giữa** mảnh là
 *    một chỗ cắt **có hiệu lực mà không hiện ở đâu cả**. Rust vẫn thực thi nó đúng, trên dữ liệu
 *    AD-5 không cho hoàn tác, còn người dùng không thấy nó nằm đâu.
 *    🔴 Chuỗi `」，` hay `……` dài ≥2 ký tự là chuyện **thường** trong tiểu thuyết, không một ca
 *    hiếm. Ice ký *"TỪ CHỐI offset giữa mảnh"*: mất độ chính xác giữa `」，` là một cái giá đọc
 *    được, còn một chỗ cắt tàng hình thì không.
 *    ⚠️ Đường sửa còn lại — chẻ `.hv-text` theo từng điểm mã để vẽ được dấu ở mọi chỗ — **bị
 *    cấm bằng chữ**: nó đổi số con của host, và `resolveSwitch` ánh xạ `children[i]` ↔
 *    `segments[i]`, nên mỗi lượt tra từ sau dấu cắt trả **sai chữ**, không lỗi nào ném.
 *
 * ⚠️ **Neo này KHÔNG phủ ca base `<ruby>` ở kiểu `parallel`** (`.hv-unit`, một TỪ Hán nhiều
 * chữ): ở đó AC9 đòi bằng chữ *"chính xác từng chữ"* nên nó **phải** giữ phép đếm, và một chỗ
 * cắt giữa từ vẫn không vẽ được dấu. Đó là món nợ đã ghi có chủ ở `GridPanel.vue` §`pendingCuts`
 * — lượt này **không** đóng nó, và cũng không được lặng lẽ đóng nó bằng cách thêm neo vào đây.
 */
const SRC_ATOMIC = 'data-src-atomic'

/**
 * Tổ tiên gần nhất **trong `cell`** mang neo `data-src-start`, hoặc `null`.
 *
 * ⚠️ Chặn ở `cell` chứ không đi hết lên `document` — cùng lý do [`duoiRt`] đã ghi.
 *
 * 🔴 **`null` là một câu trả lời ĐÚNG, không một ca sót.** Nó là thứ làm dòng
 * `Nguồn: thieu-chuu` (`.hv-sources`), câu trạng thái (`.hv-notice`) và `<rt>` **không** đẻ
 * ra một chỗ cắt — theo **cấu tạo**, không nhờ một danh sách loại trừ phải bảo trì. Một phần
 * tử mới thêm vào ô mà quên neo sẽ **im lặng không cắt được**, chứ không **cắt sai chỗ**; hai
 * kiểu hỏng đó cách nhau rất xa khi AD-5 không cho hoàn tác.
 */
function neoNguonCua(cell: Element, node: Node): Element | null {
  let p: Element | null = node instanceof Element ? node : node.parentElement
  while (p !== null && cell.contains(p)) {
    if (p.hasAttribute(SRC_START)) return p
    p = p.parentElement
  }
  return null
}

/**
 * 🔴 **Caret có đang ở ĐẦU ô bản dịch không** — Story 2.9, AC1. Cử chỉ `Backspace` gộp câu
 * đứng hay đổ ở đúng hàm này.
 *
 * Trả `false` khi: không có `Selection`, vùng chọn **không collapsed**, caret neo **ngoài**
 * `cell`, hoặc còn ký tự phía trước caret trong ô.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO KHÔNG HỎI `startOffset === 0` — nó SAI HAI TRONG BẢY HÌNH DẠNG
 * ─────────────────────────────────────────────────────────────────────────────
 * Đo trên WKWebView 605.1.15 thật ngày **2026-08-17** *(bàn đo `2-9-ban-do/`, caret đặt bằng
 * `document.caretRangeFromPoint` — tức đúng thứ một cú bấm của người dùng cho)*:
 *
 * | Ca | `startContainer` | `startOffset` | `=== 0`? | đúng |
 * |---|---|---|---|---|
 * | ô RỖNG | `DIV` *(chính ô)* | 0 | true | đầu ô |
 * | một dòng, mép trái | `#text`(11) | 0 | true | đầu ô |
 * | một dòng, giữa chữ | `#text`(11) | 2 | false | không |
 * | hai dòng, mép trái dòng 1 | `#text`(3) | 0 | true | đầu ô |
 * | **hai dòng, mép trái DÒNG 2** | **`#text`(3)** | **0** | 🔴 **true** | **không** |
 * | hai dòng, giữa chữ dòng 2 | `#text`(3) | 1 | false | không |
 * | **vùng chọn TỪ đầu ô, không collapsed** | `#text` | 0 | 🔴 **true** | **không** |
 *
 * Cơ chế của ca sai thứ nhất: dưới `white-space: pre-line` *(Story 2.5d/AD-46 — và
 * `pre-line` là **tiền đề vận hành**, không một dòng CSS trang trí)*, `insertLineBreak` của
 * WebKit để lại **ba text node** — `"AAA"` · `"\n"` · `"BBB"`, **không** một `<br>`. Engine
 * đặt caret ở đầu dòng 2 vào **offset 0 của node THỨ BA**.
 * ⇒ Một phép kiểm hỏi offset **gộp câu khi người dùng chỉ muốn xoá một lần xuống dòng** —
 * mất một segment vì đọc thiếu một node, trên một thao tác **AD-5 không cho hoàn tác**
 * *(và `⌘Z`/AC5 đang là món nợ chờ `AD-48`)*, **không cổng nào đỏ**.
 *
 * ⚠️ **Phép kiểm ngược lại cũng sai.** *"`startContainer` là con ĐẦU của ô"* hỏng ở ca ô
 * rỗng: `startContainer` là **chính ô**, 0 con. Không phép kiểm nào **theo hình dạng** đúng
 * cả bốn — phải hỏi đúng câu định nghĩa, và câu đó là *"không còn ký tự nào phía trước
 * caret trong cả ô"*.
 *
 * ⇒ Cài bằng một `Range` từ `(cell, 0)` tới caret rồi đo `toString().length`. Nó không hỏi
 * node nào, không đếm `childNodes`, không giả định `pre-line` để lại text node hay `<br>` —
 * nên nó **không cần sửa lại** ngày một trong ba thứ đó đổi.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện
 * ─────────────────────────────────────────────────────────────────────────────
 * `Range.toString()` trả **văn bản đã render**, nên nó **bỏ qua** phần tử không sinh văn bản.
 * Ô bản dịch hôm nay render **một mustache duy nhất** (`{{ s.target_text }}`) và người dùng
 * chỉ gõ chữ + `\n` vào đó ⇒ không có phần tử nào để bỏ qua. Ngày ô bản dịch mang một cấu
 * trúc *(một `<img>` neo vị trí của FR127, một `<ruby>`)*, mệnh đề này phải **đo lại** —
 * không suy.
 *
 * ⚠️ Hàm sống ở đây, **không** trong `GridPanel.vue`: một mệnh đề về **phép kiểm** kiểm được
 * bằng `vitest` + `happy-dom`, còn một mệnh đề nằm trong thân một handler bàn phím thì chỉ
 * kiểm được bằng e2e. Cùng lý do `sourceCutOffsetOf` và `segmentNavigation.ts` tồn tại.
 */
export function caretAtCellStart(cell: Element, sel: Selection | null): boolean {
  if (sel === null || sel.rangeCount === 0) return false
  const r = sel.getRangeAt(0)
  // 🔴 Vùng chọn không collapsed ⇒ `Backspace` là "xoá vùng chọn", không "xoá lui". AC6 nói
  // *không chặn*, không nói *cướp phím*: cướp nó làm người dùng mất cả đoạn vừa bôi đen VÀ
  // một ranh giới câu, cùng lúc, bằng một phím họ bấm hàng trăm lần mỗi Chương.
  if (!r.collapsed) return false
  if (!cell.contains(r.startContainer)) return false

  const truoc = cell.ownerDocument.createRange()
  truoc.setStart(cell, 0)
  truoc.setEnd(r.startContainer, r.startOffset)
  return truoc.toString().length === 0
}

/**
 * 🔴 **Cú bấm này có giữ phím bổ trợ CHÍNH của nền tảng không** — Story 2.9, AC7.
 *
 * `Mod` = `⌘` trên macOS, `Ctrl` ở nơi khác. Cùng phím trừu tượng mà `keys.ts` phân giải cho
 * bàn phím, ở đây cho **chuột**.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO CỬ CHỈ ĐÁNH DẤU CHỖ CẮT PHẢI CÓ MỘT PHÍM BỔ TRỢ
 * ─────────────────────────────────────────────────────────────────────────────
 * Cột nguyên văn mang **hai** cử chỉ chuột cùng lúc: đánh dấu chỗ cắt *(Story 2.8)* và
 * **tra từ điển** *(FR21, Story 1.18, **đã phát hành** — `useSelectionSurface(colSrc,
 * 'source', …)`)*. Trước lượt này cả hai treo trên **cùng một** `mouseup` trần, nên mỗi lượt
 * tra một từ để **đọc** cũng rơi một dấu cắt vào câu.
 *
 * ⚠️ Và một cú **double-click** bắn **HAI** `mouseup` ⇒ hai lượt `setEditorSourceCut`, hàm đó
 * **toggle** ⇒ hai offset khác nhau để lại **hai** dấu. Người dùng không có cách nào biết
 * mình vừa đặt chỗ cắt cho một lượt `⌘/` **họ không định gọi**, và AD-5 không cho hoàn tác
 * lượt tách đó.
 *
 * ✅ **Ice ký 2026-08-17** *(một lượt lật tìm ra bằng cách DÙNG THẬT, khuôn đã lặp ở 2.5b và
 * 2.8)*: `Mod`+click đánh dấu; **bấm đơn KHÔNG đánh dấu gì**, để trống cho tra cứu.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 NỀN TẢNG ĐI VÀO QUA THAM SỐ, KHÔNG ĐỌC `navigator` — §Trap 1 của `keys.ts`
 * ─────────────────────────────────────────────────────────────────────────────
 * `src/commands/README.md:73`: *"Đừng viết `event.metaKey`. `⌘` là ký hiệu macOS của một phím
 * **trừu tượng**; trên Windows nó là `Ctrl`. Một cài đặt chỉ đọc `metaKey` đi qua **cả hai nền
 * tảng của CI** rồi hỏng ở tay người dùng Windows."* Nửa Windows của kho **không có đường
 * nghiệm thu tại chỗ** *(action item A5, retro Epic 1)* ⇒ tham số tiêm được là thứ duy nhất
 * cho `tests/frontend/editorSourceCutGesture.test.ts` lái được cả hai ca.
 *
 * ⚠️ **Một cái bẫy riêng của CHUỘT, không có ở bàn phím:** trên macOS `Ctrl`+click là **cú bấm
 * phụ** (menu ngữ cảnh) của hệ điều hành. Đọc `ctrlKey` trên macOS vừa sai phím **vừa** cướp
 * một cử chỉ của OS — nên nhánh `isMac` hỏi **đúng** `metaKey`, không hỏi "một trong hai".
 *
 * ⚠️ Hàm trả lời **đúng một** câu: *"phím bổ trợ chính có được giữ không"*. Nó **không** phải
 * một phép so hợp âm đầy đủ — `sameMods` của `keys.ts` là chỗ đó, và bắt chước nó ở đây sẽ
 * dựng một bảng hợp âm **thứ hai** cho một cử chỉ chuột vốn không có bảng nào.
 */
export function hasPrimaryModifier(
  event: { metaKey: boolean; ctrlKey: boolean },
  platform: { isMac: boolean },
): boolean {
  return platform.isMac ? event.metaKey : event.ctrlKey
}

/**
 * Đếm **ký tự Unicode** (code point) của một chuỗi, không đếm code unit UTF-16.
 *
 * 🔴 `'𠀀'.length` là **2**; `[...'𠀀'].length` là **1**. Rust đếm bằng `chars()`, tức code
 * point — nên `String.length` ở đây là một đơn vị **khác** với đơn vị bên kia dây.
 */
function demKyTu(s: string | null | undefined): number {
  return s === null || s === undefined ? 0 : [...s].length
}

/**
 * `node` có nằm dưới một `<rt>` **bên trong `cell`** không?
 *
 * ⚠️ Chặn ở `cell` chứ không đi hết lên `document`: một `<rt>` ở đâu đó phía trên lưới không
 * nói gì về ô này, và một phép leo không chặn là một phép leo sẽ đọc nhầm ngày ai đó bọc lưới
 * trong một cấu trúc mới.
 */
function duoiRt(cell: Element, node: Node): boolean {
  let p: Element | null = node.parentElement
  while (p !== null && cell.contains(p)) {
    if (p.tagName === 'RT') return true
    p = p.parentElement
  }
  return false
}

/**
 * 🔴 **Chỉ số ĐIỂM MÃ trong văn bản GỐC cho mỗi điểm mã của chuỗi đã chuẩn hoá xuống dòng.**
 *
 * Story 2.9, AC9 — chỗ cắt phải là một chỉ số của `source_text` **như Rust thấy nó**, và
 * `regroup.rs::split_at` đếm bằng `chars()`, tức **điểm mã của bản GỐC**.
 *
 * ⚠️ `SourceHanViet.vue` chuẩn hoá `\r\n` *(hai điểm mã)* và `\r` *(một)* thành `\n` *(một)*
 * **trước** lượt tách từ, vì `wordStartOffsets` trả chỉ số theo chuỗi nó nhận. Bỏ qua phép ánh xạ
 * ngược này làm **mọi** chỗ cắt sau một `\r\n` lệch đúng một ký tự — im lặng, và đúng lớp lỗi mà
 * cả story ấy tồn tại để đóng. `\r` **có thể** có mặt: FR125 *(chuẩn hoá xuống dòng)* thuộc Epic 6
 * và còn `backlog`, còn `core/segment/import.rs` thì không chuẩn hoá.
 *
 * ⚠️ Trả về một bảng theo **ĐIỂM MÃ**, không theo đơn vị mã UTF-16: `[...text]` duyệt theo điểm
 * mã, nên một ký tự ngoài BMP *(CJK Extension B, có thật trong văn Hán cổ và tên riêng)* chiếm
 * **một** ô của bảng chứ không hai. Cùng đơn vị mà `chars()` của Rust đếm.
 *
 * 🔵 **Dời từ `SourceHanViet.vue` sang đây ở lượt code review 2026-08-17 — vì một khoảng hở
 * NGHIỆM THU, không vì một khuyết tật.** Tầng Acceptance Auditor đo được: `editorSourceCut.test.ts`
 * chỉ kiểm `sourceCutOffsetOf()` trên DOM **dựng tay** đã có sẵn `data-src-start`, nên không ca
 * nào đi qua phép ánh xạ này; đường verify duy nhất là một **bàn đo tay**
 * (`2-9-ban-do/han-viet-cho-cat.e2e.mjs`) không nằm trong `npm run test`. Nằm trong một
 * `<script setup>`, hàm không có đường nào để vitest gọi tới. ⇒ Một lượt hồi quy ở đây *(ai đó
 * sửa vòng lặp, hay đổi phép chuẩn hoá)* sẽ **không cổng nào đỏ**, trên đúng con số mà một chỗ
 * cắt sai im lặng đi ra từ.
 */
export function originalOffsets(text: string): number[] {
  const ky = [...text]
  const out: number[] = []
  for (let i = 0; i < ky.length; i += 1) {
    out.push(i)
    // `\r\n` là **một** điểm mã sau chuẩn hoá nhưng **hai** ở bản gốc ⇒ nhảy qua `\n`, và mọi
    // chỉ số sau đó dời đúng một bậc. `\r` trần thì không nhảy: một thành một.
    if (ky[i] === '\r' && ky[i + 1] === '\n') i += 1
  }
  return out
}
