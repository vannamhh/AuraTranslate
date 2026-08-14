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
  /**
   * Mép trái, tính từ mép trái máng — Story 2.5, Quyết định #2(a) do Ice ký 2026-08-14.
   *
   * ⚠️ Trước story này mọi vạch dùng chung `left: 8px` viết trong CSS, vì **chỉ một** giá trị
   * vạch có nguồn dữ liệu *(`primary`, và caret chỉ có một)*. Nay `confirmed` cũng có nguồn,
   * nên hai vạch **cùng tồn tại** và trường này quyết chúng đứng ở làn nào.
   *
   * 🔴 Nó đi qua `:style` cùng đường với `top`/`height` — **hình học** bind bằng style, **màu**
   * bind bằng lớp CSS. Ranh giới đó là điều kiện để Kiểm B của `check-tokens.mjs` còn nhìn
   * thấy gì: cổng đọc **CSS**, không đọc TypeScript.
   */
  left: number
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 CHIA LÀN — Quyết định #2(a), Ice ký 2026-08-14 sau khi đọc phép đo bác bản đầu
// ═════════════════════════════════════════════════════════════════════════════════
//
// **Vấn đề đo được** (`2-5-ban-do/README.md`, hai engine × hai theme, 2026-08-14): vạch là
// `position: absolute; left: 8px`, nên hai câu **cùng một dòng** cho hai vạch cùng `top` VÀ
// cùng `left` ⇒ vạch vẽ sau **che** vạch vẽ trước. Fixture văn xuôi trộn: 1/7 vạch bị che.
// Fixture **đối thoại**: **6/11 bị che (55 %)**.
//
// **Vì sao bước làn KHÔNG cố định 5px.** Bản đầu của bàn đo chia đúng hai làn (8px và 13px)
// như §Quyết định #2 của story mô tả. Phép đo bác nó: số làn cần bằng số vạch chồng nhau
// **đồng thời** nhiều nhất, và fixture đối thoại đòi **5** ⇒ làn ngoài cùng ở `left: 28px`,
// mép phải **30px** — **tràn khỏi máng 22px**, đúng chỗ chữ bắt đầu (`22 + padding-left 8`).
//
// ⇒ Bước làn **CO cho vừa**: làn ngoài cùng ở `8 + (N-1)·bước`, mép phải `+2`, và nó phải
// ≤ 22 ⇒ `bước ≤ 12/(N-1)`. Với N = 5 thì bước 3px, mép phải đúng 22px, **0 vạch bị che**.
//
// ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện:** bước không xuống dưới 2px
// *(bằng bề rộng vạch — hai vạch chạm nhau)*, nên từ **8 làn** trở lên không có lời giải nào
// trong máng 22px và luật là **dồn về làn cuối**, tức chấp nhận che. Nó đòi **8 câu cùng một
// dòng**; fixture đối thoại dày nhất mới cho **5**. Món nợ có chủ trong `deferred-work.md`.

/** Mép trái của làn trong cùng — bằng `left` mà CSS dùng trước Story 2.5. */
const LANE_ORIGIN = 8
/** Bước giữa hai làn khi máng còn rộng rãi: 2px vạch + 3px khe. */
const LANE_PITCH_MAX = 5
/** Bước tối thiểu — bằng bề rộng vạch, tức hai vạch chạm nhau. */
const LANE_PITCH_MIN = 2
/** `--space-gutter-width`. ⚠️ Chép từ token; đổi token mà quên đây thì vạch tràn. */
const GUTTER_WIDTH = 22
/** Bề rộng một vạch — `.gmark { width: 2px }`. */
const RULE_WIDTH = 2

/** Bước làn vừa khít cho `soLan` làn. Xem khối doc-comment ngay trên. */
function lanePitch(soLan: number): number {
  if (soLan <= 1) return LANE_PITCH_MAX
  const vua = (GUTTER_WIDTH - LANE_ORIGIN - RULE_WIDTH) / (soLan - 1)
  return Math.max(LANE_PITCH_MIN, Math.min(LANE_PITCH_MAX, Math.floor(vua)))
}

/** Hai vạch có chồng nhau theo chiều dọc không. */
function overlaps(a: { top: number; height: number }, b: { top: number; height: number }): boolean {
  // ⚠️ Điều kiện là **giao khoảng dọc khác rỗng**, KHÔNG phải `top` bằng nhau đúng từng pixel:
  //    `getClientRects()` trả số thực và hai engine làm tròn khác nhau, nên một phép so `===`
  //    sẽ xanh ở engine này và đỏ ở engine kia mà không ai hiểu vì sao.
  return a.top < b.top + b.height - 0.5 && b.top < a.top + a.height - 0.5
}

/**
 * Phát làn cho một bộ vạch, rồi tính `left` của từng cái — **hàm thuần, test gọi thẳng**.
 *
 * 🔴 Đây là một phép **tô màu đồ thị khoảng**, không một phép "gom thành cặp". Bản đầu gom
 * bắc cầu *(A chồng B, B chồng C ⇒ một nhóm)* rồi phát làn theo thứ tự trong nhóm, và **lượt
 * chạy đầu trên bàn đo bác nó ngay**: một nhóm năm phần tử dồn cả bốn phần tử sau vào làn 1
 * ⇒ vẫn còn một vạch bị che. Đúng lớp lỗi *"trúng tiền đề chưa phải trúng cơ chế"*.
 *
 * Phép đúng: duyệt theo `top` tăng dần, mỗi vạch nhận **làn nhỏ nhất chưa bị một vạch chồng
 * nào chiếm**. Số làn dùng = số vạch chồng nhau **đồng thời** nhiều nhất.
 *
 * ⚠️ Bước làn tính **một lần cho cả bộ**, không theo từng nhóm: hai nhóm cạnh nhau dùng hai
 * bước khác nhau sẽ làm máng trông như bị lệch khi người dùng cuộn qua chúng.
 */
export function assignGutterLanes(rules: readonly Omit<GutterRule, 'left'>[]): GutterRule[] {
  const sorted = [...rules].sort((a, b) => a.top - b.top || a.id - b.id)
  const laned: (Omit<GutterRule, 'left'> & { lane: number })[] = []

  // ─────────────────────────────────────────────────────────────────────────────
  // 🔴 QUÉT ĐƯỜNG, KHÔNG QUÉT LẠI CẢ BỘ — và đây là một sửa chữa CÓ SỐ, không một gu
  // ─────────────────────────────────────────────────────────────────────────────
  // Bản đầu hỏi *"vạch này chồng với vạch nào trong **tất cả** những vạch đã phát làn"* ⇒
  // `O(n²)`. Tới hết Story 2.3 điều đó vô hại vì `wanted` có **nhiều nhất một** phần tử
  // *(chỉ `primary` có nguồn dữ liệu, và caret chỉ có một)*. **Story 2.5 phá đúng giả định
  // đó**: `wanted` nay chứa **mọi câu đã xác nhận**, nên trên Chương lớn nhất có thật —
  // **9.850** câu (đo 2026-08-12) — một Chương dịch xong cho ~9.850 vạch, tức ~48 triệu cặp
  // mỗi lượt đo. Lượt đo đó chạy trong một `requestAnimationFrame`, nơi trần NFR2 là **50 ms**.
  //
  // ⇒ Vì `sorted` tăng dần theo `top`, một vạch đã **kết thúc phía trên** vạch đang xét thì
  // không bao giờ chồng với nó **hay với bất kỳ vạch nào sau nó**. Giữ một tập `active` và
  // vứt chúng đi ⇒ `O(n log n)` cho phép sắp, cộng `O(n·k)` với `k` = số vạch chồng nhau
  // **đồng thời**, mà `k` bị chặn bởi hình học: nhiều nhất là số câu cùng một dòng.
  //
  // ⚠️ **Đo 2026-08-14** (Node 22.22.2, macOS 15.6, 9.850 vạch, ~3 câu mỗi dòng, ba lượt):
  //
  // | hình dạng | lượt 1 | lượt 2 | lượt 3 |
  // |---|---|---|---|
  // | `O(n²)` *(bản đầu)* | **482,4 ms** | **254,5 ms** | **261,6 ms** |
  // | quét đường *(bản này)* | **8,3 ms** | **5,2 ms** | **4,3 ms** |
  //
  // **Hệ quả đo được:** bản đầu vượt trần một frame của NFR2 *(50 ms)* từ **5 tới 10 lần**, và
  // đó mới là **CPU thuần** — chưa tính lượt `getClientRects()` và lượt bố cục đi kèm. Hai hình
  // dạng cho **kết quả giống hệt** trên cùng bộ dữ liệu, nên đây là một lượt đổi chi phí, không
  // một lượt đổi hành vi.
  // ⚠️ Số trên đo bằng **Node**, không phải WKWebView — nó là một **cận dưới**, và vế frame thật
  // thuộc bàn đo. Vế đó có chủ là Story 2.4 (`deferred-work.md`).
  //
  // ⚠️ `active` giữ **thứ tự tăng dần theo mép dưới**, nên phép cắt tỉa chỉ phải nhìn từ đầu.
  let active: (Omit<GutterRule, 'left'> & { lane: number })[] = []

  for (const r of sorted) {
    active = active.filter((p) => p.top + p.height - 0.5 > r.top)
    const taken = new Set(active.filter((p) => overlaps(p, r)).map((p) => p.lane))
    let lane = 0
    while (taken.has(lane)) lane += 1
    const placed = { ...r, lane }
    active.push(placed)
    laned.push(placed)
  }

  const soLan = laned.reduce((max, r) => Math.max(max, r.lane + 1), 1)
  const buoc = lanePitch(soLan)
  // Trần cứng: làn vượt sức chứa của máng **dồn về làn cuối** — xem §GIỚI HẠN THẬT ở trên.
  const lanCuoi = Math.floor((GUTTER_WIDTH - LANE_ORIGIN - RULE_WIDTH) / buoc)

  return laned.map(({ lane, ...rest }) => ({
    ...rest,
    left: LANE_ORIGIN + Math.min(lane, lanCuoi) * buoc,
  }))
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
  const out: Omit<GutterRule, 'left'>[] = []

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

  // 🔴 Story 2.5 — phát làn NGAY TẠI ĐÂY, không ở chỗ gọi. Một chỗ gọi quên bước này sẽ dựng
  // lại đúng trạng thái *"hai vạch cùng `left`, cái sau che cái trước"* mà Quyết định #2 tồn
  // tại để đóng, và **không cổng nào đỏ** vì `left` vẫn là một số hợp lệ.
  return assignGutterLanes(out)
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
