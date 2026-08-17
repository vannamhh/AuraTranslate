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
  // `source_text`. Xem khối 🔵 ở doc-comment trên.
  if (duoiRt(cell, node)) return null

  let truoc = 0
  const walker = cell.ownerDocument.createTreeWalker(cell, NodeFilter.SHOW_TEXT)
  let n = walker.nextNode()
  while (n !== null && n !== node) {
    if (!duoiRt(cell, n)) truoc += demKyTu(n.textContent)
    n = walker.nextNode()
  }
  // `node` KHÔNG phải một text node của ô *(vd chính `cell` khi ô rỗng)* ⇒ `walker` chạy hết
  // mà không gặp nó. Trả tổng độ dài, tức **cuối** ô — cùng ngữ nghĩa "bấm vào khoảng trống
  // sau câu" mà đường chuột của ô bản dịch đã chọn.
  //
  // 🔴 `offsetInNode` là một chỉ số **UTF-16** *(mọi offset của DOM Range đều thế)*, nên nó
  // phải đi qua `demKyTu` chứ không cộng thẳng — xem khối 🔵 vế ②.
  return truoc + (n === null ? 0 : demKyTu(n.textContent?.slice(0, offsetInNode)))
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
