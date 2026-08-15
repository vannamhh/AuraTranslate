/**
 * Sổ **bộ phân giải vùng chọn Hán Việt theo Ô** — Story 2.5b, AC7 · Task 5.3.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO MODULE NÀY PHẢI TỒN TẠI — HAI ĐÒI HỎI ĐÚNG, XUNG KHẮC Ở TẦNG DOM
 * ═════════════════════════════════════════════════════════════════════════════════
 * ① **AC7 đòi ĐÚNG MỘT bề mặt cho cả CỘT nguyên văn.** Nguyên văn: *"đăng ký theo CỘT,
 *    KHÔNG theo từng ô"*. Lý do là một phép đếm **tĩnh**: `SELECTION_SURFACE_FLOOR` của
 *    `check-commands.mjs` đếm số lời gọi `useSelectionSurface` trong `src/**`, và mảng
 *    `surfaces` ở `selectionContract.ts:75` là một mảng **tuyến tính** — N ô cho ra hàng
 *    nghìn bề mặt, tức cổng vô nghĩa **và** O(N) mỗi lượt chọn.
 * ② **Bộ phân giải Hán Việt là của TỪNG Ô.** `SourceHanViet.vue` ánh xạ *âm Hán Việt trên
 *    màn hình → ký tự Hán nguồn* bằng cách duyệt **DOM con của chính instance đó**
 *    (`resolveSwitch` đọc `host.children[i]`, `resolveParallel` đọc `<rt>`/`<rb>`). Bảng ánh
 *    xạ ấy **không** rút lên cấp cột được: mỗi ô có văn bản nguồn riêng và cây con riêng.
 *
 * ⇒ Lời giải: **một** đăng ký ở cấp cột, và nó **uỷ quyền** xuống đúng ô chứa neo vùng chọn.
 * Sổ dưới đây là chỗ hai vế gặp nhau.
 *
 * ─────────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO KHÔNG PHẢI MỘT `WeakMap` — và đây là một quyết định, không một lượt tiện tay
 * ─────────────────────────────────────────────────────────────────────────────────
 * Một `WeakMap<HTMLElement, resolver>` tra được **đúng phần tử gốc**, nhưng neo vùng chọn là
 * một **text node nằm sâu bên trong**. Tra ngược lên đòi một lượt `closest()` với một selector
 * — tức một **cái tên lớp CSS** trở thành hợp đồng giữa hai tệp, thứ không cổng nào canh.
 * Một mảng nhỏ + `el.contains(anchor)` dùng lại **đúng vị từ** mà `selectionContract.ts:184`
 * đã đo trên cả hai engine, nên hai tầng không thể lệch nhau về nghĩa *"thuộc bề mặt này"*.
 *
 * ⚠️ Kích thước mảng là **số ô nguyên văn ĐANG hiển thị Hán Việt**, không phải số câu của
 * Chương: một ô chỉ ghi tên khi bề mặt Hán Việt của nó thực sự được dựng (`v-if`). Với lưới
 * không ảo hoá *(Quyết định #7(a) — chủ số: Story 2.4)* con số đó **bằng** số câu, nên phép
 * duyệt tuyến tính ở đây là O(N) mỗi lượt chọn — cùng hạng với chính `selectionContract`.
 * 🔴 Ghi ra thay vì để người sau tự phát hiện: **nếu Giai đoạn 3 dựng ảo hoá, con số này tụt
 * xuống theo cửa sổ hiển thị và mục nợ đó tự đóng.** Đừng tối ưu ở đây trước lượt đo.
 */

/** Cách lấy truy vấn từ một vùng chọn — cùng chữ ký với `SelectionResolver`. */
export type HanVietResolver = (selection: Selection) => string | null

type Entry = { el: HTMLElement; resolve: HanVietResolver }

/**
 * ⚠️ Một **mảng**, không `Map` khoá theo phần tử — xem khối lý do ở đầu tệp. Thứ tự không
 * mang nghĩa: hai ô Hán Việt **không bao giờ lồng nhau**, nên nhiều nhất một mục khớp.
 */
const entries: Entry[] = []

/**
 * Ghi tên một ô Hán Việt vào sổ. Trả về hàm **nhả** — gọi lúc `unmount`.
 *
 * ⚠️ **Idempotent qua mount/unmount**, cùng luật và cùng lý do với
 * `registerSelectionSurface`: một lượt đổi preset bố cục dựng lại cả lưới, và `dockview` có
 * thể mount bản mới **trước** khi unmount bản cũ. Đăng ký lại cùng một phần tử ⇒ mục cũ bị
 * **thay**, không nhân đôi.
 */
export function registerHanVietSurface(el: HTMLElement, resolve: HanVietResolver): () => void {
  const at = entries.findIndex((e) => e.el === el)
  const entry: Entry = { el, resolve }
  if (at === -1) entries.push(entry)
  else entries[at] = entry

  return () => {
    const i = entries.findIndex((e) => e.el === el)
    // ⚠️ So sánh **mục**, không chỉ phần tử: nếu một lượt mount mới đã thay mục này rồi thì
    // hàm nhả của bản CŨ không được gỡ đăng ký của bản MỚI.
    if (i !== -1 && entries[i] === entry) entries.splice(i, 1)
  }
}

/**
 * Truy vấn cho một vùng chọn, uỷ quyền xuống ô Hán Việt chứa neo của nó.
 *
 * @returns `undefined` ⇒ **neo không nằm trong một ô Hán Việt nào** — chỗ gọi phải rơi về
 *   hành vi mặc định (`Selection.toString()`). `null` ⇒ có ô, và ô đó nói *"vùng chọn này
 *   không ánh xạ được"* ⇒ **không phát lượt tra**.
 *
 * 🔴 Ba giá trị trả về, ba nghĩa **phân biệt được** — *"rỗng IM LẶNG bị cấm; rỗng CÓ LÝ DO
 * thì không"*. Gộp `undefined` với `null` sẽ làm một lượt bôi đen trên ô nguyên văn **thường**
 * *(tab `Nguyên văn`, hay một Chương tiếng Anh)* im lặng không tra gì.
 */
export function resolveHanVietSelection(selection: Selection): string | null | undefined {
  const anchor = selection.anchorNode
  if (anchor === null) return undefined
  for (const entry of entries) {
    if (entry.el.contains(anchor)) return entry.resolve(selection)
  }
  return undefined
}

/** Số ô đang ghi tên — **chỉ cho test và chẩn đoán**. Đừng dựng luật hiển thị trên nó. */
export function hanVietSurfaceCount(): number {
  return entries.length
}
