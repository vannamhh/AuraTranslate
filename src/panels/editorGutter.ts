/**
 * 🔴 **ĐO HÌNH HỌC THẬT CỦA TỪNG CÂU** để vạch lề cao **đúng bằng câu tương ứng** —
 * Story 2.2, AC2 · AC14 · Quyết định #2(a).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO PHẢI ĐO, TRONG KHI MỌI THỨ KHÁC CỦA DỰ ÁN LÀM ĐƯỢC BẰNG CSS THUẦN
 * ─────────────────────────────────────────────────────────────────────────────
 * AC1 nói văn bản là **một trang liền mạch** — mỗi câu là một `<span>` chảy **inline** trong
 * một dòng văn liên tục, nên một câu bắt đầu giữa dòng, xuống dòng vài lần, rồi kết thúc
 * giữa một dòng khác. Nó chiếm **nhiều hình chữ nhật**, không một hình.
 *
 * *"Cao đúng bằng câu"* vì thế là **từ mép trên hình chữ nhật ĐẦU tới mép dưới hình chữ nhật
 * CUỐI**. Hai đường CSS thuần đều bị loại và mỗi đường bị loại bởi một mệnh đề khác nhau:
 * cho mỗi câu một `display: block` là *"chia thành khối"* mà **AC1 cấm bằng chữ**; còn một
 * `::before` gắn vào chính câu là một pseudo-element **inline**, và nó không trải theo chiều
 * cao nhiều dòng của câu bọc nó.
 *
 * ⇒ `Range.getClientRects()` *(qua `Element.getClientRects()`, cùng hình học)*, rồi vẽ vạch
 * `position: absolute` trong máng.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ TỆP NÀY LÀ MODULE THUẦN DOM — KHÔNG `import` VUE
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `./editorSegments.ts`: `scripts/check-*.mjs` `import()` được nó. *(Phép kiểm
 * hình học thật thì vẫn cần một trình duyệt — xem §Task 7, bàn đo chạy tay.)*
 */

/** Vị trí một vạch lề, tính bằng `px` **trong hệ toạ độ của chính máng**. */
export type GutterRule = {
  /** `segment.id` — khoá `v-for`, và là thứ nối vạch với câu của nó. */
  id: number
  /** Mép trên, tính từ mép trên máng. */
  top: number
  /** Chiều cao, luôn `> 0` cho một câu có hình học. */
  height: number
}

/**
 * Thuộc tính DOM nối một `<span>` câu với hàng `segment` của nó.
 *
 * ⚠️ **Hằng này phủ hai chỗ ĐỌC, không phủ chỗ GHI** *(làm rõ ở code review 2026-08-12 — bản
 * trước khai "một hằng, ba bản chép" và điều đó không đúng)*. Bốn chỗ, và mỗi chỗ có lý do
 * riêng cho hình dạng của nó:
 *
 * 1. `measureGutterRules` ngay dưới — **đọc**, dùng hằng;
 * 2. `EditorPanel.vue::onSelectionChange` — **đọc**, dùng hằng;
 * 3. `EditorPanel.vue` template — **ghi**, viết thẳng `:data-segment-id="s.id"`. Đổi sang tên
 *    thuộc tính động `:[SEGMENT_ID_ATTR]` sẽ **làm đỏ sàn nội dung của Kiểm J**, vốn tìm đúng
 *    chuỗi đó trong bản đã che để chứng minh template còn dựng câu;
 * 4. `scripts/check-commands.mjs` Kiểm J — **viết thẳng có chủ ý**: một cổng không được
 *    `import` từ chính tệp nó đang kiểm, nếu không một lượt đổi hằng sẽ kéo cả cổng đi theo và
 *    phép kiểm tự khớp với chính nó.
 *
 * ⇒ Đổi giá trị hằng này **không** lan tới template và cổng. Đổi thì phải sửa cả ba chỗ, và
 * Kiểm J sẽ đỏ nếu quên chỗ thứ ba — đó là lưới an toàn thật, không phải hằng này.
 */
export const SEGMENT_ID_ATTR = 'data-segment-id'

/**
 * Đo vạch lề cho những câu trong `wanted` — tức những câu **thật sự có một vạch để vẽ**.
 *
 * `gutter` và `doc` phải là hai con của **cùng** một hộp cuộn *(xem `.edwrap` trong
 * `EditorPanel.vue`)*: cả hai cuộn cùng nhau, nên hiệu hai `getBoundingClientRect().top` là
 * một số **không đổi theo lượt cuộn**, và vạch không trôi khỏi câu của nó.
 *
 * 🔴 **Câu không có hình học nào bị BỎ QUA, không vẽ một vạch cao 0.** Ca có thật: panel bị
 * ẩn (`display: none` do một lượt đổi preset) ⇒ mọi `getClientRects()` trả danh sách rỗng.
 * Vẽ một vạch cao 0 ở `top: 0` cho ra một chấm ở mép trên máng — một dấu hiệu **sai** về một
 * câu hoàn toàn bình thường.
 */
export function measureGutterRules(
  gutter: HTMLElement,
  doc: HTMLElement,
  wanted: ReadonlySet<number>,
): GutterRule[] {
  const origin = gutter.getBoundingClientRect().top
  const out: GutterRule[] = []

  for (const el of doc.querySelectorAll<HTMLElement>(`[${SEGMENT_ID_ATTR}]`)) {
    const id = Number(el.getAttribute(SEGMENT_ID_ATTR))
    if (!Number.isFinite(id)) continue
    // 🔴 Câu KHÔNG có vạch (*không vạch* — nhánh "chưa dịch" của AC3) không có gì để vẽ, nên
    // không có gì để đo. Đây **không** phải một lượt tối ưu mù: `getClientRects()` là một
    // lượt đọc hình học đồng bộ, và bỏ nó cho một phần tử mà kết quả không đi đâu cả là
    // tiết kiệm đúng thứ không ai tiêu. Quy mô làm nó đáng nói ra: Chương lớn nhất có thật
    // có **9.850** câu, và hôm nay **nhiều nhất một** câu trong số đó có vạch.
    if (!wanted.has(id)) continue

    const rects = el.getClientRects()
    if (rects.length === 0) continue

    let top = Number.POSITIVE_INFINITY
    let bottom = Number.NEGATIVE_INFINITY
    for (const r of rects) {
      // ⚠️ Hình chữ nhật RỖNG vẫn được trả về ở một số ca xuống dòng (một câu kết thúc đúng
      //    mép phải sinh một rect rộng 0 ở dòng kế). Gộp nó vào sẽ kéo vạch dài thêm một dòng
      //    mà mắt không thấy chữ nào ở đó.
      if (r.height <= 0) continue
      if (r.top < top) top = r.top
      if (r.bottom > bottom) bottom = r.bottom
    }
    if (!Number.isFinite(top) || !Number.isFinite(bottom)) continue

    out.push({ id, top: top - origin, height: bottom - top })
  }

  return out
}

/**
 * Gộp mọi lượt yêu cầu đo vào **một** `requestAnimationFrame` — Quyết định #2, điều kiện 3.
 *
 * 🔴 Vì sao nó phải tồn tại: ba nguồn kích hoạt *(`ResizeObserver`, `document.fonts.ready`,
 * nội dung đổi)* có thể nổ trong **cùng một** frame, và một Chương lớn có **9.850** câu.
 * Đo ba lượt trong một frame là ba lượt reflow đồng bộ trên 9.850 phần tử — đúng chỗ NFR2
 * *(50 ms mỗi frame)* vỡ, và vỡ vì hạ tầng chứ không vì phép đo.
 *
 * Trả về hàm **huỷ** — gọi lúc unmount, nếu không một lượt đo đã hẹn sẽ chạy trên một cây
 * DOM đã tháo.
 */
export function createRuleScheduler(run: () => void): { schedule: () => void; cancel: () => void } {
  let handle: number | null = null

  const schedule = (): void => {
    if (handle !== null) return
    handle = requestAnimationFrame(() => {
      handle = null
      run()
    })
  }

  const cancel = (): void => {
    if (handle === null) return
    cancelAnimationFrame(handle)
    handle = null
  }

  return { schedule, cancel }
}
