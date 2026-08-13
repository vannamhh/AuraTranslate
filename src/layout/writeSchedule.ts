/**
 * Nhịp ghi *idle + trần cứng* — **tầng THUẦN**. Story 1.14 · AC4 · AD-11 · **AD-35 từ Story 2.3**.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI RIÊNG — §BẪY 3 CỦA STORY, VÀ NÓ KHÔNG LÀM CỔNG NÀO ĐỎ
 * ─────────────────────────────────────────────────────────────────────────────
 * `onDidLayoutChange` của dockview bắn **liên tục** trong lúc kéo sash. Ghi một lượt
 * `putConfig` ở mỗi lần bắn thì một cú kéo 3 giây là **hàng trăm** job xếp hàng qua
 * `store::Writer` nối tiếp — đúng thứ AD-11/AD-12 tồn tại để chặn. Không cổng nào đỏ vì
 * đó, và biểu hiện sẽ hiện ra ở **Epic 2** dưới dạng *"gõ bị khựng"* mà không ai lần
 * được về dòng nào.
 *
 * Nên nhịp ghi được tách khỏi `.vue` và làm **thuần**: `scripts/check-layout.mjs` `import()`
 * tệp này bằng Node thuần, đẩy một chuỗi sự kiện dày rồi **ĐẾM số lượt ghi**. Đó là bằng
 * chứng mà AC4 đòi, thay cho một lượt đếm tay trong console.
 *
 * ⇒ Luật "erasable-only": không `import` gì cả, không `enum`, không `namespace`.
 * ⚠️ Và **không đọc `Date.now()` bên trong**: mọi thời điểm ĐI VÀO qua tham số. Một hàm
 * tự đọc đồng hồ thì cổng phải `sleep` thật để kiểm nó — tức một phép kiểm chậm và chập
 * chờn thay vì một phép kiểm tức thời và tất định.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 MỘT HÀM, **HAI CHỖ DÙNG**, HAI CẶP HẰNG — và chỉ MỘT chỗ mang bảo đảm của AD-35
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ **Khối này được viết lại ở Story 2.3 · Task 1.5.** Tới hết Story 2.2 nó nói *"đây là
 * MƯỢN hình dạng của AD-35, KHÔNG phải áp AD-35 cho bố cục"* — đúng vào ngày Editor chưa có
 * đường ghi nào. Ice chốt Quyết định #2 đường **(a)** ngày 2026-08-12, nên từ story đó tệp
 * này **là** đường AD-35 thật, và một comment nói ngược sẽ đánh lừa đúng người đọc kỹ nhất.
 *
 * | Chỗ dùng | Hằng | Mang bảo đảm của AD-35? |
 * |---|---|---|
 * | `WorkspaceDock.vue` — nhịp ghi **bố cục** (Story 1.14) | [`IDLE_MS`] `500` / [`HARD_CAP_MS`] `5000` | **KHÔNG** |
 * | `panels/editorFlush.ts` — nhịp flush **Editor** (Story 2.3) | `EDITOR_IDLE_MS` `2000` / `EDITOR_HARD_CAP_MS` `5000` | **CÓ** |
 *
 * Cái dùng chung là **hình dạng** *(idle cộng một trần cứng không reset)*. Các **bảo đảm** —
 * *"đi qua đúng `store::Writer` nối tiếp"*, *"chỉ xong sau khi đã ghi vào WAL"*, NFR18
 * *"mất tối đa 5 giây công việc"* — thuộc **chỗ dùng của Editor**, không thuộc tệp này. Bố cục
 * không có `SegmentVersion`, không có lịch sử, và mất một lượt kéo sash **không phải mất công
 * việc** — nó là mất 40 pixel chiều rộng mà người dùng kéo lại trong một giây.
 *
 * 🔴 **KHÔNG sửa hành vi của [`createWriteSchedule`] để "hợp Editor hơn".** Hai chỗ đứng trên
 * nó, và mỗi chỗ có lưới riêng: `scripts/check-layout.mjs` Kiểm B (`:288`, chạy
 * [`simulateWrites`]) cho bố cục, và `tests/frontend/editorFlush.test.ts` cho Editor. Một lượt
 * sửa hoặc làm đỏ một cổng của story khác, hoặc — tệ hơn — **không** làm đỏ và lặng lẽ đổi
 * nhịp ghi bố cục. Cần một hành vi khác ⇒ một **tham số mới** có giá trị mặc định giữ nguyên
 * hành vi cũ, cộng một test cho **cả hai** giá trị.
 */

/**
 * Yên bao lâu thì ghi — **cặp hằng của BỐ CỤC**. Đủ dài để nuốt trọn một cú kéo sash, đủ ngắn
 * để một lượt đóng cửa sổ ngay sau đó gần như luôn có bản mới nhất trên đĩa.
 *
 * ⚠️ Editor mang cặp riêng (`EDITOR_IDLE_MS` = 2000) khai ở `panels/editorFlush.ts`. **Đừng**
 * đổi con số dưới đây "cho khớp Editor" — nó thuộc Story 1.14, và một lượt đổi ở đây không
 * làm cổng nào đỏ.
 */
export const IDLE_MS = 500

/**
 * 🔴 TRẦN CỨNG — và nó **KHÔNG BAO GIỜ ĐƯỢC RESET BỞI SỰ KIỆN KẾ TIẾP**.
 *
 * Đây là khác biệt giữa "debounce" và thứ tệp này làm. Một debounce thuần: kéo sash liên
 * tục 60 giây ⇒ **không một lượt ghi nào** xảy ra, và một lần tắt máy đột ngột ở giây 59
 * mất trọn 59 giây thao tác. Trần cứng bảo đảm cứ mỗi `HARD_CAP_MS` là có ít nhất một
 * lượt chạm đĩa, bất kể sự kiện còn dày tới đâu.
 */
export const HARD_CAP_MS = 5000

export type WriteSchedule = {
  /**
   * Một thay đổi bố cục vừa xảy ra tại `now` (mili-giây, đơn điệu tăng).
   * @returns thời điểm TUYỆT ĐỐI phải ghi. Chỗ gọi đặt lại timer về đúng mốc này.
   */
  onChange(now: number): number
  /** Vừa ghi xong tại `now` — dọn trạng thái để chu kỳ sau bắt đầu lại từ đầu. */
  onWrite(now: number): void
  /** Có thay đổi chưa ghi không? Điều kiện của lượt ghi lúc rời chế độ / đóng cửa sổ. */
  isDirty(): boolean
  /** Mốc phải ghi đang chờ, hoặc `null` khi sạch. Để chỗ gọi không phải tự nhớ. */
  deadline(): number | null
}

export function createWriteSchedule(
  idleMs: number = IDLE_MS,
  hardCapMs: number = HARD_CAP_MS,
): WriteSchedule {
  /** Thời điểm của thay đổi ĐẦU TIÊN trong chu kỳ hiện tại. `null` ⇒ đang sạch. */
  let firstChangeAt: number | null = null
  let due: number | null = null

  const onChange = (now: number): number => {
    if (firstChangeAt === null) firstChangeAt = now
    /**
     * 🔴 `Math.min`, và cả điểm của tệp này nằm ở đây.
     *
     * `now + idleMs` là vế debounce: nó **trượt về sau** theo mỗi sự kiện.
     * `firstChangeAt + hardCapMs` là trần: `firstChangeAt` **không** được gán lại cho
     * tới khi [`onWrite`] chạy, nên vế này **đứng yên** dù sự kiện có dày tới đâu.
     * Lấy cái nhỏ hơn ⇒ trần luôn thắng khi dòng sự kiện đủ dài.
     */
    due = Math.min(now + idleMs, firstChangeAt + hardCapMs)
    return due
  }

  const onWrite = (now: number): void => {
    // `now` không được dùng để tính gì — nó ở trong chữ ký để chỗ gọi không thể "quên
    // mất mình đang ở thời điểm nào", và để một lượt log/telemetry sau này có sẵn mốc.
    void now
    firstChangeAt = null
    due = null
  }

  return {
    onChange,
    onWrite,
    isDirty: () => firstChangeAt !== null,
    deadline: () => due,
  }
}

/**
 * Chạy một dòng sự kiện qua lịch ghi và trả về **thời điểm của từng lượt ghi**.
 *
 * ⚠️ Đây **không phải** một tiện ích cho test — nó là mô phỏng chính xác vòng lặp mà
 * `WorkspaceDock.vue` chạy (`setTimeout` tới `deadline()`, ghi, `onWrite`), viết ở dạng
 * không có đồng hồ và không có timer. Nhờ vậy `scripts/check-layout.mjs` khẳng định
 * được một mệnh đề ĐỊNH LƯỢNG — *"kéo sash 3 giây ⇒ ≤ 2 lượt ghi"* — thay vì một lượt đếm
 * tay trong console mà không ai chạy lại được.
 *
 * @param eventsAt mốc thời gian của từng sự kiện `onDidLayoutChange`, tăng dần.
 */
export function simulateWrites(
  eventsAt: readonly number[],
  idleMs: number = IDLE_MS,
  hardCapMs: number = HARD_CAP_MS,
): readonly number[] {
  const schedule = createWriteSchedule(idleMs, hardCapMs)
  const writes: number[] = []

  for (const at of eventsAt) {
    // Mọi mốc ghi đã tới hạn TRƯỚC sự kiện này thì đã chạy rồi — timer không chờ sự kiện.
    const pending = schedule.deadline()
    if (pending !== null && pending <= at) {
      writes.push(pending)
      schedule.onWrite(pending)
    }
    schedule.onChange(at)
  }
  // Timer cuối cùng vẫn nổ sau sự kiện cuối.
  const last = schedule.deadline()
  if (last !== null) writes.push(last)
  return writes
}
