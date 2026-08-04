/**
 * Hai chuỗi chẩn đoán của nhánh `catch` trong `App.vue`, tách thành module riêng vì
 * một lý do rất cụ thể — và đó KHÔNG phải cùng lý do với `eventName.ts`.
 *
 * `eventName.ts` tách ra để tránh một import tĩnh kéo `scopeCheck.ts` vào bundle
 * release. Tệp này tách ra vì AC2 của Story 1.5: *"grep chuỗi tiếng Việt trong `.rs`
 * và `.vue` không ra kết quả"*. Hai chuỗi dưới đây nằm trong template literal ở
 * `App.vue:38` và `:54` — chúng là **vi phạm thật** của cổng `npm run check:i18n`, và
 * là hai vi phạm duy nhất trong cây nguồn lúc Story 1.5 bắt đầu.
 *
 * ⛔ **Chúng KHÔNG đi vào `vi.json`.** Đây là chẩn đoán cho log CI, không phải chuỗi
 * giao diện; `vi.json` là tài nguyên **hiển thị** (NFR16, AD-21). Trộn hai thứ là làm
 * hỏng chính ranh giới mà Story 1.5 dựng: một tệp mà mọi khoá đều là thứ người dùng
 * đọc được, và mọi khoá đều nghiệm thu được theo năm quy tắc giọng văn UX-DR47.
 *
 * ⚠️ **Vì sao hai chuỗi này không làm cổng đỏ — đọc kỹ, câu trả lời KHÔNG phải "miễn
 * trừ".** `src/selftest/**` có một miễn trừ **có tên** trong `scripts/check-i18n.mjs`,
 * nhưng nó **khớp 0 tệp** và cổng in ra đúng con số đó ở mỗi lượt chạy: Kiểm A đi trên
 * `.rs` và `.vue`, còn đây là `.ts`. Thứ che hai chuỗi này là một **lỗ phạm vi** —
 * đúng giới hạn đã khai ở `deferred-work.md` — chứ không phải một miễn trừ đã ai duyệt.
 *
 * Ghi thẳng ra vì lời văn cũ dạy sai mô hình, và mô hình sai đó có một hệ quả rất cụ
 * thể: nó biến "dời chuỗi từ `.vue` sang một tệp `.ts`" thành một cách hợp lệ để cổng
 * xanh. Không phải vậy. Chỗ đúng của một chuỗi HIỂN THỊ vẫn là `vi.json`; hai chuỗi
 * dưới đây được miễn vì chúng là **chẩn đoán cho log CI** — không vượt IPC, không được
 * render, người đọc chúng là người đang sửa self-check — chứ không vì đuôi tệp.
 *
 * ⚠️ Module này chỉ có hai hàm thuần chuỗi, nên `App.vue` import TĨNH được mà không
 * tốn gì — đúng khuôn `eventName.ts`. ⛔ Đừng thêm gì import từ `./scopeCheck` vào
 * đây; làm vậy là kéo cả mã self-check vào bundle release qua cửa sau.
 *
 * ⚠️ `scripts/check-scope.mjs` và `check-scope-bundled.mjs` đọc dòng `VERDICT:` trong
 * chuỗi dưới đây. Đổi khuôn dòng đó là làm hai cổng của Story 1.2/1.3 mù.
 */

/**
 * Báo cáo đầy đủ khi self-check gãy TRƯỚC khi kịp chạy — in ra stdout và gửi kèm
 * event về Rust.
 */
export function fallbackReportText(err: unknown): string {
  return (
    'AuraTranslate — asset protocol scope self-check (Story 1.2 AC3 · Story 1.3 AC8)\n' +
    'mode:     undetermined  (self-check gãy trước khi chạy)\n' +
    '\n' +
    `[FAIL] self-check gãy trước khi chạy: ${String(err)}\n` +
    '\n' +
    'VERDICT: FAIL'
  )
}

/**
 * Đường lui cuối cùng: không còn kênh nào phát về Rust, chỉ còn stdout. Dòng
 * `VERDICT: FAIL` đã ra trước đó nên lượt chạy vẫn kết luận được.
 */
export function emitFailureLine(err: unknown): string {
  return `[FAIL] không phát được event self-check về Rust: ${String(err)}`
}
