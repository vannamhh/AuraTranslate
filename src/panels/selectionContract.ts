/**
 * 🔴 **HỢP ĐỒNG VÙNG CHỌN DÙNG CHUNG** — Story 1.18, AC2 · AC3 · Quyết định #1(a).
 *
 * `epics.md:1762` nói nguyên văn: Auto-Lookup *"gắn vào một hợp đồng vùng chọn dùng chung
 * cho mọi panel văn bản"*, và AI Translation + Editor *"nhận được cùng hành vi khi chúng có
 * nội dung ở các epic sau, không cần cài lại"*. Tệp này LÀ hợp đồng đó.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO **OPT-IN**, KHÔNG PHẢI MỘT LISTENER TOÀN CỤC + DANH SÁCH LOẠI TRỪ
 * ─────────────────────────────────────────────────────────────────────────────
 * Panel Lookup **tự nó chứa chữ** (nghĩa, ví dụ, trích dẫn — `LookupRecord.vue`). Một
 * listener `document` không lọc nguồn dựng một **vòng tự thay thế**: bôi đen một nghĩa để đọc
 * kỹ ⇒ một lượt tra mới thay chính đoạn đang đọc, cộng một hiệu ứng, cộng một lượt cuộn về
 * đầu. Người dùng mất chỗ và không hiểu vì sao — không test nào bắt được, không cổng nào nhìn thấy.
 *
 * Một danh sách LOẠI TRỪ phải được bảo trì tay qua chín epic; quên một bề mặt mới là mở
 * lại lỗ đó **im lặng**. Cùng lớp lỗi mà AD-44 ① bác *"sổ đăng ký tệp nào chứa ngôn ngữ
 * nào"*. ⇒ Bề mặt phải **tự khai** mình là nguồn (AC3, mệnh đề ba).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO **MỘT** LISTENER TRÊN `document`, KHÔNG MỘT LISTENER MỖI PANEL
 * ─────────────────────────────────────────────────────────────────────────────
 * Kéo chọn từ trong panel rồi thả chuột **ngoài** panel là thao tác **thường ngày** (và
 * gần như bắt buộc khi chọn tới cuối một khối). Một listener trên gốc panel không bao giờ
 * thấy `mouseup` đó ⇒ vùng chọn **không được tra**, không lỗi, không dấu hiệu. Vì thế listener sống
 * trên `document` và vị từ *"thuộc nguồn nào"* đọc **`anchorNode`**, không `event.target`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ VÌ SAO TỆP NÀY SỐNG Ở `src/panels/`, không `src/commands/`
 * ─────────────────────────────────────────────────────────────────────────────
 * Nó bị `src/main.ts` **tiêm vào** `installCommands()` — đúng cửa mà `sourcePanelState.ts`
 * và `lookupPanelState.ts` đã đi qua. `src/commands/**` phải nạp được bằng **Node thuần**
 * để Kiểm C/D/E của `npm run check:commands` chạy trên chính bộ command của sản phẩm; một
 * `import` của tệp này ở đó kéo DOM vào cổng và giết ba phép kiểm hành vi cùng lúc.
 *
 * ⚠️ State **module-level**, không trong `<script setup>`: một lượt đổi preset gọi
 * `api.clear()` rồi dựng lại **cả bốn** panel, và một `ref` cục bộ chết cùng lượt đó. Lượt
 * đăng ký của panel vì vậy phải **idempotent** qua mount/unmount — xem [`registerSelectionSurface`].
 */
import { onBeforeUnmount, watch } from 'vue'
import type { Ref } from 'vue'
import { currentQuery } from './lookupPanelState'

/**
 * Vai của một bề mặt chữ trong hợp đồng.
 *
 * 🔴 **`'display'` không phải "chưa làm xong" — nó là một mệnh đề.** Panel Lookup là một bề
 * mặt chữ thật và nó **không được** là nguồn (AC3). Một panel văn bản không đăng ký gì cả là
 * đúng thứ AC2 tồn tại để chặn, nên ngoại lệ này phải **đọc được từ chính mã** thay vì
 * tồn tại như một sự vắng mặt.
 */
export type SelectionRole = 'source' | 'display'

/**
 * Cách lấy **văn bản truy vấn** từ một vùng chọn thuộc bề mặt này.
 *
 * `undefined` ⇒ dùng `Selection.toString()`, đúng hành vi của mọi bề mặt nguyên văn.
 *
 * 🔴 Chỗ cắm này tồn tại cho **đúng một** ca đã biết: tab Hán Việt kiểu **chuyển đổi**, nơi
 * màn hình chỉ có âm Hán Việt (chữ Latin) còn truy vấn phải là **ký tự Hán nguồn** — xem
 * `SourceHanViet.vue`. Trả `null` ⇔ *"vùng chọn này không ánh xạ được"* ⇒ không phát lượt tra.
 */
export type SelectionResolver = (selection: Selection) => string | null

type Surface = {
  el: HTMLElement
  role: SelectionRole
  resolve: SelectionResolver | undefined
}

/**
 * ⚠️ Một **mảng**, không `Map` khoá theo phần tử: thứ tự đăng ký quyết định bề mặt nào thắng
 * khi hai bề mặt lồng nhau, và phép duyệt luôn ưu tiên `'display'` (xem [`surfaceFor`]).
 */
const surfaces: Surface[] = []

/**
 * Đăng ký một bề mặt chữ. Trả về hàm **nhả** — gọi lúc `unmount`.
 *
 * ⚠️ **Idempotent qua mount/unmount**: đăng ký lại cùng một phần tử ⇒ mục cũ bị thay, không
 * nhân đôi. Một lượt đổi preset dựng lại cả ba panel, và `dockview` có thể `mount` bản
 * mới **trước** khi `unmount` bản cũ.
 *
 * ⚠️ `role` phải viết **LITERAL** ở chỗ gọi, không qua biến — cùng luật `owner`/`status-key`
 * mà Kiểm E của `check-commands.mjs` đã đặt: cổng đếm của AC2 đọc **tĩnh**, và một biểu
 * thức ở đó bị đếm rồi **bỏ qua**, tức mất lưới.
 */
export function registerSelectionSurface(
  el: HTMLElement,
  role: SelectionRole,
  resolve?: SelectionResolver,
): () => void {
  const existing = surfaces.findIndex((s) => s.el === el)
  const entry: Surface = { el, role, resolve }
  if (existing === -1) surfaces.push(entry)
  else surfaces[existing] = entry

  return () => {
    const at = surfaces.findIndex((s) => s.el === el)
    // ⚠️ So sánh mục, không chỉ phần tử: nếu một lượt mount mới đã thay mục này rồi thì hàm
    // nhả của bản CŨ không được gỡ đăng ký của bản MỚI (thứ tự mount/unmount của dockview).
    if (at !== -1 && surfaces[at] === entry) surfaces.splice(at, 1)
  }
}

/**
 * 🔴 **CỬA DUY NHẤT MÀ MỘT PANEL ĐI QUA** — AC2. Đăng ký theo vòng đời của phần tử.
 *
 * Bề mặt chữ của một panel xuất hiện và biến mất theo `v-if` (tab đang chọn, Chương đã nạp
 * hay chưa), nên lượt đăng ký không đặt được ở `onMounted` một lần: nó phải **theo `ref`**.
 *
 * ⚠️ `role` viết **LITERAL** ở chỗ gọi — cổng đếm của AC2 đọc tĩnh (xem
 * [`registerSelectionSurface`]).
 *
 * 🔴 **Vì sao không `onUnmounted` mà là `onBeforeUnmount` + `watch`:** một lượt đổi preset gọi
 * `api.clear()` rồi dựng lại cả ba panel, và `dockview` có thể mount bản mới **trước**
 * khi unmount bản cũ. `watch` với `flush: 'sync'` nhả đúng phần tử cũ ngay khi `ref` đổi,
 * không để một phần tử đã tháo ở lại trong sổ (rò như 1.16 đã bắt với `declareFocus`).
 */
export function useSelectionSurface(
  elRef: Readonly<Ref<HTMLElement | null>>,
  role: SelectionRole,
  resolve?: SelectionResolver,
): void {
  let release: (() => void) | null = null

  const sync = (el: HTMLElement | null): void => {
    release?.()
    release = null
    if (el !== null) release = registerSelectionSurface(el, role, resolve)
  }

  watch(elRef, sync, { immediate: true, flush: 'sync' })
  onBeforeUnmount(() => {
    release?.()
    release = null
  })
}

/**
 * 🔴 **VỊ TỪ THUỘC-NGUỒN — ĐO TRÊN CẢ HAI ENGINE, KHÔNG SUY TỪ BẢNG TƯƠNG THÍCH.**
 *
 * Đo 2026-08-07 trên WKWebView (Swift/`WKWebView`, engine Tauri dùng trên macOS) **và**
 * Chromium (Chrome headless — engine WebView2 dùng trên Windows). Hai engine khớp nhau
 * từng dòng. Ba số đo quyết định hình dạng hàm này:
 *
 * ① **`deferred-work.md:635` SAI trên cả hai engine.** Nó khai rằng
 *    `getSelection().toString()` trả `''` cho vùng chọn bên trong `<input>`. Đo được:
 *    `"nội "` — tức văn bản THẬT. ⇒ không dựa vào hành vi đó (mục `:635` đã ghi đúng lời
 *    khuyên, chỉ sai lý do).
 *
 * ② **`anchorNode` không BAO GIỜ nằm TRONG một `<input>`.** Cây shadow của ô nhập không lộ ra,
 *    nên vùng chọn trong ô nhập để lại `anchorNode` = phần tử **CHA** kèm offset trỏ vào
 *    chính ô nhập. Đo được: `anchorNode = DIV`, và `host.contains(anchorNode) === true`.
 *    ⇒ mệnh đề *"`anchorNode` nằm trong `<input>`/`<textarea>` ⇒ không nguồn"* của Task 1
 *    **không cài được như đã viết** — nó không bao giờ đúng.
 *
 * ③ **`document.activeElement` cho ÂM TÍNH GIẢ.** Tiêu điểm ở một ô nhập trong khi vùng
 *    chọn nằm trên một bề mặt chữ khác là trạng thái **hợp lệ** (đo được: `activeElement =
 *    INPUT` với một vùng chọn thật trên bề mặt) ⇒ một cổng đọc `activeElement` sẽ **nuốt**
 *    lượt tra đó. không dùng.
 *
 * ⇒ **Tín hiệu ĐÚNG là `nodeType`.** Một vùng chọn chữ thật luôn neo vào một node **VĂN
 * BẢN** (`nodeType 3`); vùng chọn trong ô nhập neo vào một node **PHẦN TỬ** (`nodeType 1`).
 * Hai ca phân biệt nhau sạch trên cả hai engine. Đó là phép loại trừ **CƠ HỌC** mà AC3
 * đòi — không một tai nạn may mắn.
 */
function surfaceFor(selection: Selection): Surface | null {
  if (selection.rangeCount === 0) return null

  const anchor = selection.anchorNode
  // ② + ③ ở trên — loại trừ CƠ HỌC ô nhập, không qua `activeElement`, không qua chuỗi rỗng.
  if (anchor === null || anchor.nodeType !== Node.TEXT_NODE) return null

  // Đai an toàn thứ hai: một node văn bản con TRỰC TIẾP của ô nhập (không quan sát được trên
  // hai engine đã đo, nhưng cây shadow là chi tiết cài đặt của engine — không cược vào nó).
  const parent = anchor.parentNode
  if (parent instanceof HTMLInputElement || parent instanceof HTMLTextAreaElement) return null

  // 🔴 `'display'` được hỏi TRƯỚC — Bẫy 1. Một bề mặt hiển thị lồng bên trong một bề mặt
  // nguồn (chưa có hôm nay, nhưng Story 3.4 đánh dấu thuật ngữ Glossary NGAY TRONG Panel
  // Source) phải thắng, nếu không vòng tự thay thế mở lại ở đúng chỗ khó thấy nhất.
  for (const surface of surfaces) {
    if (surface.role === 'display' && surface.el.contains(anchor)) return surface
  }
  for (const surface of surfaces) {
    if (surface.role === 'source' && surface.el.contains(anchor)) return surface
  }
  return null
}

/**
 * Văn bản của vùng chọn hiện tại, **nếu nó thuộc một nguồn đã đăng ký** — không thì `''`.
 *
 * 🔴 Đây là thứ thay dep TỐI THIỂU `currentSelection` mà Story 1.17 tiêm ở `main.ts:182`
 * (`() => window.getSelection()?.toString() ?? ''`). Chữ ký giữ nguyên **có chủ ý**:
 * command `lookup.lookup_selection` không phải đổi một dòng nào, và `Mod+Alt+L` với
 * Auto-Lookup vì vậy là **đúng MỘT đường** (Quyết định #4a) — nên Story 1.20 chỉ có một
 * chỗ để cắm lịch sử vào.
 */
export function currentSelectionText(): string {
  const selection = window.getSelection()
  if (selection === null) return ''

  const surface = surfaceFor(selection)
  // không nguồn nào nhận vùng chọn này (Panel Lookup, ô nhập Library, một bề mặt chưa đăng
  // ký) ⇒ rỗng ⇒ command không phát lượt tra. AC3.
  if (surface === null || surface.role !== 'source') return ''

  const text = surface.resolve === undefined ? selection.toString() : surface.resolve(selection)
  return text ?? ''
}

/**
 * 🔴 **TÍN HIỆU *"VÙNG CHỌN ĐÃ DỪNG"* — SỰ KIỆN, KHÔNG HẸN GIỜ** (Quyết định #5a).
 *
 * | Đường | Tín hiệu | Vì sao không cái khác |
 * |---|---|---|
 * | Chuột | `mouseup` | `selectionchange` bắn **liên tục** trong lúc kéo ⇒ *"tra hàng loạt cụm dở dang"* mà `mockups/motion-auto-lookup.html:200` cấm đích danh — mỗi ký tự kéo qua là một lượt IPC, **CI vẫn xanh**, và sản phẩm giật |
 * | Bàn phím | `keyup` khi **`Shift` nhả** | Giữ `Shift` + gõ `→` nhiều lần là **một** thao tác chọn, không phải năm |
 *
 * **Không debounce.** Một hằng số thời gian không ai đo được, đặt vào **đúng đường nóng
 * NFR1**, và nó vẫn không phân biệt được *"đã dừng kéo"* với *"đang kéo chậm"*.
 */
export function attachSelectionWatcher(target: Document, dispatchLookup: () => void): () => void {
  /**
   * 🔴 **DEDUPE — so với truy vấn ĐANG HIỆN, không với truy vấn đang bay** (Quyết định #5).
   *
   * Bấm lại vào **chính** cụm vừa tra là thao tác thường ngày; một lượt IPC cộng một lượt
   * hiệu ứng 90 ms cho **không thông tin mới** là chi phí thuần.
   *
   * ⚠️ **Chỉ đường TỰ ĐỘNG được dedupe.** `Mod+Alt+L` đi thẳng qua command và không qua đây:
   * bấm phím là một yêu cầu **tường minh** của người dùng để tra lại, còn một lượt tự động
   * lặp lại chính truy vấn đang hiện là nhiễu. Hai ý định khác nhau, không hai đường tra khác
   * nhau — cả hai vẫn hội tụ ở `dispatch('lookup.lookup_selection')`.
   *
   * 🔴 **Và so với truy vấn VỪA TỰ PHÁT, không chỉ với truy vấn ĐANG HIỆN** — lượt review
   * 2026-08-07 bắt được rằng Shift+click bắn cả `mouseup` (chọn xong) lẫn `keyup` (nhả
   * `Shift` ngay sau) cho **cùng một** vùng chọn. `currentQuery.value` chưa kịp cập nhật
   * trong khoảng round-trip IPC giữa hai sự kiện đó, nên chỉ so với nó để lọt một lượt
   * dispatch trùng thứ hai. `lastAutoDispatched` đóng khoảng đó; nó hội tụ về đúng
   * `currentQuery.value` khi kết quả về, nên không cần dọn tay.
   */
  let lastAutoDispatched: string | null = null
  const shouldDispatch = (text: string): boolean =>
    text !== '' && text !== currentQuery.value && text !== lastAutoDispatched

  const onSelectionSettled = (): void => {
    const text = currentSelectionText()

    // 🔴 **Bẫy 5 — vùng chọn RỖNG ⇒ không LÀM GÌ, không xoá panel.** Một cú **bấm** (không kéo) cũng
    // bắn `mouseup` và thu vùng chọn về một caret. Xoá panel ở đó nghĩa là kết quả biến
    // mất **mỗi lần** người dùng bấm vào văn bản để đọc tiếp — không lỗi, không cổng nào đỏ, và
    // nó phá đúng thứ FR21 vừa mua.
    if (!shouldDispatch(text)) return
    lastAutoDispatched = text

    // ⚠️ `registry.dispatch` **NÉM** với một id chưa đăng ký. Ném không tháo listener — trình
    // duyệt chỉ báo lỗi ra console rồi lượt sự kiện SAU vẫn chạy bình thường — nhưng lỗi đó
    // vẫn phải hiện với chẩn đoán ĐÚNG NGỮ CẢNH (`lookup.lookup_selection` chưa đăng ký / handler
    // đã ném) thay vì một stack trace chung chung của trình duyệt. Bọc là chi tiết cài đặt;
    // **không im lặng** là điều bị cấm, nên nhánh `catch` kêu đích danh. (Quyết định #4, §hệ
    // quả phải ghi ra.)
    try {
      dispatchLookup()
    } catch (err) {
      console.error(
        '[selection] lượt tra tự động không phát được — `lookup.lookup_selection` chưa đăng ' +
          `ký, hoặc handler của nó đã ném. ${String(err)}`,
      )
    }
  }

  const onMouseUp = (): void => {
    onSelectionSettled()
  }

  /**
   * ⚠️ Chỉ phản ứng khi **`Shift` nhả**, không mọi `keyup`: giữ `Shift` và gõ `→` năm lần là
   * **một** thao tác chọn. `event.key === 'Shift'` là lúc thao tác đó kết thúc.
   *
   * Không lọc theo `event.target`: vùng chọn có thể kết thúc khi tiêu điểm đã ở nơi
   * khác, và vị từ thật là [`surfaceFor`] (đọc `anchorNode`) — cùng lý do listener sống
   * trên `document`.
   */
  const onKeyUp = (event: KeyboardEvent): void => {
    if (event.key !== 'Shift') return
    onSelectionSettled()
  }

  target.addEventListener('mouseup', onMouseUp)
  target.addEventListener('keyup', onKeyUp)

  return () => {
    target.removeEventListener('mouseup', onMouseUp)
    target.removeEventListener('keyup', onKeyUp)
  }
}

/**
 * 🔴 **MỞ RỘNG VÙNG CHỌN BẰNG BÀN PHÍM** — AC11, đóng `deferred-work.md:608`.
 *
 * Sự thật kỹ thuật, không ý kiến: một `<div>`/`<p>` không sửa được **không nhận** `Shift+Mũi tên`.
 * Trình duyệt chỉ cho điều đó khi ① caret browsing bật (mặc định TẮT, không bật được bằng
 * mã), ② phần tử `contenteditable`, hoặc ③ ứng dụng **tự dựng `Range`** — đường này.
 *
 * ✅ **ĐO THẬT 2026-08-07, hai engine, không đọc bảng tương thích** (Task 0, Quyết định #2):
 * `Selection.modify()` **không chuẩn** nhưng có ở **cả** WKWebView lẫn Chromium, và cả hai
 * cho kết quả **giống hệt** trên chín phép thử — kể cả `tabindex="0"` trên một `<p>` không
 * sửa được.
 *
 * 🔴 **Bẫy 9 của story không CÓ THẬT.** Story cảnh báo `'word'` trên văn xuôi tiếng Trung
 * *"có thể nuốt cả câu hoặc không nhảy"*. Đo được trên `他打開了那扇門，走進了黑暗之中。`:
 * `word` ×1 ⇒ `"他"` (1 ký tự) · `word` ×2 ⇒ `"他打開"` (3 ký tự) — **cả hai engine phân
 * đoạn ĐÚNG** (cùng thuật toán ngắt từ ICU). ⇒ bốn command `extend_word_*` giao được, không
 * cần đường lui.
 */
function modifySelection(direction: 'left' | 'right', granularity: 'character' | 'word'): void {
  const selection = window.getSelection()
  if (selection === null || selection.rangeCount === 0) return

  // ⚠️ `Selection.modify` không có trong `lib.dom.d.ts` (nó không chuẩn) — khai hẹp tại chỗ thay
  // vì một `as any`: đây là chỗ DUY NHẤT trong dự án chạm nó, và một kiểu hẹp giữ được
  // phép kiểm tham số mà `any` vứt đi.
  const withModify = selection as Selection & {
    modify?: (alter: string, direction: string, granularity: string) => void
  }
  if (typeof withModify.modify !== 'function') {
    // KHÔNG im lặng: một engine KHÔNG có `modify` nghĩa là NFR17 mất vế bàn phím trên nền tảng
    // đó, và đó là thứ phải lần ra được, không đoán.
    console.error(
      '[selection] `Selection.modify()` không có trên engine này — mở rộng vùng chọn bằng ' +
        'bàn phím KHÔNG hoạt động (NFR17, `deferred-work.md:608`).',
    )
    return
  }
  // Không im lặng nếu engine THẬT SỰ có `modify` nhưng lượt gọi cụ thể này ném (tham số hợp lệ,
  // nhưng chưa có phép đo nào phủ hết mọi trạng thái `Selection` — cùng kỷ luật với nhánh
  // `dispatchLookup` của `attachSelectionWatcher`, không để một lượt ném biến mất không dấu vết.
  try {
    withModify.modify('extend', direction, granularity)
  } catch (err) {
    console.error(`[selection] \`Selection.modify()\` ném khi mở rộng vùng chọn: ${String(err)}`)
  }
}

/** Bốn thao tác mở rộng vùng chọn — handler thật của `selection.extend_*` (AC11). */
export const selectionCommands = {
  extendLeft: (): void => modifySelection('left', 'character'),
  extendRight: (): void => modifySelection('right', 'character'),
  extendWordLeft: (): void => modifySelection('left', 'word'),
  extendWordRight: (): void => modifySelection('right', 'word'),
}

/**
 * Đặt caret vào **bề mặt nguồn đầu tiên đã đăng ký** — điểm vào bàn phím của AC11.
 *
 * ⚠️ không dùng `focus.next_panel`: cái đó dời tiêu điểm tới **gốc panel** (`tabindex="-1"`),
 * và một `Range` không được dựng từ đó. Ở đây ta cần đúng bề mặt CHỮ, và một `Range` rỗng ở
 * đầu nó, để `Selection.modify()` có chỗ bám.
 */
export function focusSelectionSource(): boolean {
  const source = surfaces.find((s) => s.role === 'source' && s.el.isConnected)
  if (source === undefined) return false

  // Cùng kỷ luật với `modifySelection`: dựng `Range` là DOM, không phải nội bộ ta kiểm soát
  // hết — một lượt ném ở đây không được biến mất không dấu vết.
  try {
    source.el.focus()
    const selection = window.getSelection()
    if (selection === null) return false

    const range = document.createRange()
    range.selectNodeContents(source.el)
    range.collapse(true)
    selection.removeAllRanges()
    selection.addRange(range)
    return true
  } catch (err) {
    console.error(`[selection] không đặt được caret vào bề mặt nguồn: ${String(err)}`)
    return false
  }
}
