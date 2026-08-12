/**
 * Sổ điểm vào focus — nửa thứ hai của AD-34. Story 1.6 · AC4 · UX-DR7 · NFR17.
 *
 * AD-34 §2: *"Mỗi chế độ và mỗi panel khai báo điểm vào focus. Chuyển panel phải dời
 * focus DOM tường minh. Không chế độ nào để focus rơi về `body`."* Hai mệnh đề, hai cơ
 * chế khác nhau, và story ghi thẳng ranh giới:
 *
 *   - **Vế "khai báo" là DỮ LIỆU** — máy kiểm được. `owners()` là đầu vào cho Kiểm E
 *     của `scripts/check-commands.mjs`.
 *   - **Vế "không rơi về `body`" là HÀNH VI DOM LÚC CHẠY**, mà dự án không có bộ chạy
 *     test frontend (và không được thêm — NFR15). Nên nó được canh bằng một CHỐT TỰ KÊU
 *     ở `enter()` cộng một lượt nghiệm thu tay có bảng. Không đánh dấu đạt bằng suy
 *     luận — giới hạn ghi thẳng vào `deferred-work.md`.
 *
 * ⚠️ Vì sao tệp này ở `src/commands/` chứ không phải một thư mục thứ bảy: §Câu hỏi cho
 * Ice #1 của story. AD-34 §1 (thao tác) và §2 (focus) là hai mệnh đề của CÙNG một AD;
 * `src/modes/` sai vì panel cũng khai điểm vào, `src/layout/` sai vì nó thuộc `dockview`
 * của Story 1.14.
 *
 * Cùng luật "erasable-only" như `./registry.ts` — Kiểm C và Kiểm E `import()` tệp này
 * bằng Node thuần. Không `enum`, không `namespace`, không parameter property. Lần
 * `import` giá trị DUY NHẤT được phép là `./registry.ts`, chính vì tệp đó cũng thuần.
 */
import { COMMAND_ID_RE } from './registry.ts'

/**
 * Id của một chế độ hoặc một panel: `mode.library`, `panel.source`.
 *
 * ⚠️ Dùng CHUNG văn phạm với command id — cùng một biểu thức, không chép lại một biến
 * thể. Owner và command id sống trong hai không gian tên khác nhau nhưng cùng một hình
 * dạng khoá, đúng tinh thần AD-34.
 */
export type FocusOwner = string

/** Trả về phần tử nhận focus, hoặc `null` khi chế độ/panel chưa dựng xong. */
export type FocusEntry = () => HTMLElement | null

export type FocusRegistry = {
  /** ⚠️ NÉM khi: owner rỗng · owner sai văn phạm · owner TRÙNG · `resolve` không phải hàm. */
  declare(owner: FocusOwner, resolve: FocusEntry): void
  /** Gỡ khai báo khi component tháo. Không có nó thì một lượt mount lại là một lần ném. */
  release(owner: FocusOwner): void
  has(owner: FocusOwner): boolean
  /** Thứ tự KHAI BÁO, ổn định — `next()` xoay vòng theo đúng thứ tự này. */
  owners(): readonly FocusOwner[]
  /** Dời focus DOM tường minh. `false` + `console.error` nêu đích danh owner khi trượt. */
  enter(owner: FocusOwner): boolean
  /** Owner được `enter()` thành công gần nhất — `null` khi chưa có lần nào. */
  current(): FocusOwner | null
  /** Xoay vòng focus giữa các owner có tiền tố `prefix` (`'panel.'`), theo thứ tự KHAI BÁO. */
  next(prefix: string): boolean
  /**
   * Xoay vòng trên một vòng ĐƯỢC TRUYỀN VÀO, theo `step` (`+1` xuôi, `-1` ngược).
   *
   * 🔴 Story 1.14 · AC9 — vòng xoay phải đi theo **thứ tự bố cục hiện tại** (trái→phải,
   * trên→dưới của lưới đang hiện), không theo thứ tự `declare()`. Hai thứ đó khác nhau
   * ngay khi người dùng kéo một panel sang chỗ khác, và [`next`] không biết gì về lưới.
   * Chỗ biết là `src/layout/dockController.ts`; nó truyền vòng xuống đây.
   */
  cycle(ring: readonly FocusOwner[], step: number): boolean
}

export function createFocusRegistry(): FocusRegistry {
  const byOwner = new Map<FocusOwner, FocusEntry>()
  let last: FocusOwner | null = null

  const declare = (owner: FocusOwner, resolve: FocusEntry): void => {
    if (typeof owner !== 'string' || owner.trim() === '') {
      throw new Error('[focus] owner rỗng — mỗi chế độ và mỗi panel phải tự gọi tên mình (AD-34 §2).')
    }
    if (!COMMAND_ID_RE.test(owner)) {
      throw new Error(
        `[focus] owner \`${owner}\` sai văn phạm — phải khớp ${COMMAND_ID_RE.source} ` +
          '(cùng hình dạng khoá với command id và khoá `vi.json`).',
      )
    }
    // Cùng lý lẽ với `register()`: khai trùng nghĩa là hai chỗ cùng nhận một điểm vào,
    // và cái sau sẽ lặng lẽ nuốt cái trước. Ném thì đỏ ngay ở màn hình đầu tiên.
    if (byOwner.has(owner)) {
      throw new Error(
        `[focus] owner \`${owner}\` đã khai rồi — hai điểm vào cùng tên là một lỗi lập trình. ` +
          'Gọi `release()` ở `onBeforeUnmount` nếu đây là một lượt mount lại.',
      )
    }
    if (typeof resolve !== 'function') {
      throw new TypeError(`[focus] \`${owner}\` thiếu hàm phân giải phần tử.`)
    }
    byOwner.set(owner, resolve)
  }

  const release = (owner: FocusOwner): void => {
    /**
     * ⚠️ KÊU khi gỡ một owner chưa khai. Mọi chỗ lệch khác trong tệp này đều ném hoặc
     * `console.error`; một `Map.delete` trả `false` bị bỏ đi là chỗ duy nhất im lặng — và
     * nó im đúng lúc cần nói: một `releaseFocus('panel.sorce')` gõ sai trong
     * `onBeforeUnmount` để owner THẬT khai vĩnh viễn, rồi lượt mount sau ném *"đã khai
     * rồi"* ở một tệp không liên quan gì tới chỗ gõ sai.
     */
    if (!byOwner.delete(owner)) {
      console.error(
        `[focus] \`${String(owner)}\` chưa khai — \`release()\` không gỡ được gì. ` +
          'Kiểm chỗ gõ tên owner: một tên sai ở đây để owner thật kẹt lại và lượt mount ' +
          'sau sẽ ném ở một chỗ khác hẳn.',
      )
      return
    }
    if (last === owner) last = null
  }

  const has = (owner: FocusOwner): boolean => byOwner.has(owner)
  const owners = (): readonly FocusOwner[] => [...byOwner.keys()]
  const current = (): FocusOwner | null => last

  /**
   * 🔴 CHỐT AC4 VẾ SAU — và nó để KÊU, không để VÁ.
   *
   * Kiểm ở frame kế tiếp (`requestAnimationFrame`) vì `el.focus()` chưa chắc đã kết
   * thúc trong cùng một lượt tick khi Vue đang vá DOM. **Đừng "sửa" bằng cách focus
   * lại vòng lặp**: một vòng focus tự phục hồi sẽ đánh nhau với người dùng đang Tab và
   * với hộp thoại của hệ điều hành — hỏng đắt hơn hẳn thứ nó định chữa.
   *
   * ⚠️ Có canh gác `typeof` vì tệp này được `import()` bằng Node thuần trong Kiểm C/E,
   * nơi không có `window` và không có `document`. Không có canh gác thì cổng ném ở một
   * chỗ không liên quan gì tới thứ nó đang kiểm.
   */
  const armBodyGuard = (owner: FocusOwner): void => {
    if (typeof requestAnimationFrame !== 'function' || typeof document === 'undefined') return
    requestAnimationFrame(() => {
      if (document.activeElement === document.body || document.activeElement === null) {
        console.error(
          `[focus] sau khi vào \`${owner}\`, focus rơi về \`body\` — AC4 nói điều đó KHÔNG được ` +
            'xảy ra. Kiểm `tabindex="-1"` trên phần tử gốc và thứ tự render của chế độ này.',
        )
      }
    })
  }

  const enter = (owner: FocusOwner): boolean => {
    const resolve = byOwner.get(owner)
    if (!resolve) {
      console.error(
        `[focus] \`${String(owner)}\` chưa khai điểm vào — không dời focus được. ` +
          'Mỗi chế độ và mỗi panel phải gọi `declare()` lúc mount (AD-34 §2).',
      )
      return false
    }
    let el: HTMLElement | null = null
    try {
      el = resolve()
    } catch (err) {
      console.error(`[focus] hàm phân giải của \`${owner}\` ném: ${String(err)}`)
      return false
    }
    if (!el || typeof el.focus !== 'function') {
      console.error(
        `[focus] \`${owner}\` khai điểm vào nhưng phần tử chưa có trong DOM — focus không dời được.`,
      )
      return false
    }
    /**
     * 🔴 PHẦN TỬ ĐÃ THÁO KHỎI DOM — và `<KeepAlive>` làm ca này thành thường trực.
     *
     * §Quyết định thiết kế #6 bắt buộc `<KeepAlive>`, nên rời Workspace KHÔNG tháo
     * `PanelFrame`: Vue đỗ subtree ở một container **tách rời**. `panel.source` vẫn khai,
     * `resolve()` vẫn trả một `HTMLElement` thật, `typeof el.focus === 'function'` vẫn
     * đúng — nhưng `el.focus()` trên một node đã tách là **no-op**. Không có phép kiểm
     * này thì hàm trả `true`, `last` bị ghi, và `armBodyGuard` im lặng vì `activeElement`
     * đang là gốc chế độ chứ không phải `body`: người gọi tin focus đã dời, `current()`
     * nói dối, và không một dòng chẩn đoán nào được in.
     *
     * ⚠️ `isConnected` không có trong Node thuần — Kiểm C/E `import()` tệp này. Chỉ kiểm
     * khi thuộc tính đó thật sự tồn tại; một phần tử giả trong cổng không bị chặn oan.
     */
    if ('isConnected' in el && el.isConnected === false) {
      console.error(
        `[focus] \`${owner}\` phân giải ra một phần tử ĐÃ THÁO khỏi DOM — \`focus()\` sẽ ` +
          'không làm gì. Chế độ/panel này đang bị `<KeepAlive>` đỗ, hoặc `resolve()` giữ ' +
          'một tham chiếu cũ. Không báo thành công cho một lần dời focus không xảy ra.',
      )
      return false
    }
    el.focus()
    last = owner
    armBodyGuard(owner)
    return true
  }

  /**
   * Handler thật của `focus.next_panel` (AC6, §Quyết định thiết kế #5).
   *
   * Command này CỐ Ý không gán phím — nhưng nó KHÔNG được rỗng. Vòng xoay đi theo
   * thứ tự KHAI BÁO, tức thứ tự panel được dựng, và đó là thứ tự đọc tự nhiên hôm nay.
   * Story 1.14 dựng lưới 2×2 với `dockview` sẽ thay thứ tự này bằng thứ tự bố cục.
   */
  /**
   * 🔴 ĐỌC FOCUS THẬT, không đọc con trỏ ứng dụng tự giữ.
   *
   * `last` chỉ được ghi bởi một `enter()` thành công — chuột và `Tab` KHÔNG bao giờ cập
   * nhật nó. Bản đầu xoay vòng từ `last`, và hệ quả quan sát được: sau `enter('mode.
   * workspace')` rồi người dùng **bấm chuột** vào `panel.editor`, `focus.next_panel` nhảy
   * tới `panel.source` — panel **trước** chỗ người dùng đang đứng.
   *
   * ⚠️ `PanelFrame.vue:26-31` bác thẳng đúng khuôn mẫu này cho cờ tiêu điểm: *"Một cờ do
   * ứng dụng tự giữ sẽ vẫn sáng đúng một panel trong khi focus thật đã rơi ra ngoài."*
   * Logic xoay vòng phải theo cùng một kỷ luật.
   *
   * `last` vẫn là đường lui khi không đọc được `document` (Kiểm C/E chạy trong Node) hoặc
   * khi focus thật đang nằm ngoài vòng.
   */
  const indexOfLiveFocus = (ring: readonly FocusOwner[]): number => {
    if (typeof document === 'undefined') return ring.indexOf(last as FocusOwner)
    const active = document.activeElement
    if (active !== null && active !== document.body) {
      for (let i = 0; i < ring.length; i += 1) {
        const resolve = byOwner.get(ring[i] as FocusOwner)
        if (resolve === undefined) continue
        let el: HTMLElement | null = null
        try {
          el = resolve()
        } catch {
          continue
        }
        if (el !== null && (el === active || el.contains(active))) return i
      }
    }
    return last === null ? -1 : ring.indexOf(last)
  }

  /**
   * Xoay vòng trên một vòng đã cho — thân chung của [`next`] và của `focus.next_panel` /
   * `focus.prev_panel` (Story 1.14 · AC9).
   *
   * ⚠️ `step` được chuẩn hoá về `+1` / `-1` rồi mới dùng. Một `step` bằng 0 sẽ làm vòng
   * lặp dưới đây thử đúng một owner `ring.length` lần — im lặng và vô nghĩa.
   */
  const cycle = (ring: readonly FocusOwner[], step: number): boolean => {
    if (ring.length === 0) {
      console.error(
        '[focus] vòng xoay RỖNG — không có panel nào để đi tới. Kiểm rằng bố cục đã dựng ' +
          'xong và ít nhất một panel đang hiện (AC3 cho phép ẩn, nhưng không cho phép ẩn hết).',
      )
      return false
    }
    const dir = step < 0 ? -1 : 1
    const at = indexOfLiveFocus(ring)
    /**
     * ⚠️ Thử HẾT vòng, không bỏ cuộc ở thành viên hỏng đầu tiên. Bản đầu tính đúng một
     * ứng viên và trả về kết quả của nó, nên một panel đang bị `<KeepAlive>` đỗ *(tức
     * `enter()` trả `false` theo phép kiểm `isConnected` ở trên)* làm cả thao tác chết,
     * kể cả khi panel kế tiếp trong vòng còn sống và nhận focus được.
     *
     * ⚠️ `+ ring.length` trước `%`: JavaScript cho `-1 % 4 === -1`, nên chiều lùi sẽ đọc
     * `ring[-1]` là `undefined` và `enter(undefined)` chỉ ghi một chẩn đoán vô nghĩa.
     */
    for (let n = 1; n <= ring.length; n += 1) {
      const at2 = ((at + dir * n) % ring.length + ring.length) % ring.length
      if (enter(ring[at2] as FocusOwner)) return true
    }
    console.error(
      `[focus] không owner nào trong vòng nhận được focus — cả ${ring.length} điểm vào đều ` +
        'chưa dựng xong hoặc đã tháo khỏi DOM.',
    )
    return false
  }

  const next = (prefix: string): boolean => {
    const ring = owners().filter((o) => o.startsWith(prefix))
    if (ring.length === 0) {
      console.error(`[focus] không owner nào mang tiền tố \`${prefix}\` — không có vòng để xoay.`)
      return false
    }
    return cycle(ring, 1)
  }

  return { declare, release, has, owners, enter, current, next, cycle }
}

/**
 * Chọn node để **trả tiêu điểm về** khi một lớp phủ mở ra — UX-DR17, dùng chung cho
 * `ShortcutsOverlay` và `AttributionOverlay`.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO KHÔNG CHỈ LƯU `document.activeElement` — một phép đo, không một lo xa
 * ═════════════════════════════════════════════════════════════════════════════════
 * UX-DR17 hứa tiêu điểm quay về **NÚT ĐÃ MỞ**. Lưu `activeElement` lúc mở là một phép
 * **xấp xỉ** của lời hứa đó: nó đúng khi engine chịu đặt tiêu điểm lên `<button>` lúc bấm,
 * và sai khi engine không chịu.
 *
 * Đo trên WKWebView thật (2026-08-12, bàn đo `attribution-focus`): với nút mở nằm **trong
 * một panel dockview**, tiêu điểm **không giữ được trên nút**. Chuỗi `focusin`/`focusout`
 * ghi lại được, và nó xảy ra **đồng bộ ngay trong lời gọi `focus()`**:
 *
 *     focusout ← button[data-attribution-open]
 *     focusin  → section.panel        (tổ tiên mang `tabindex="-1"`)
 *
 * Nên `activeElement` lúc mở là `section.panel`, và lớp phủ trả tiêu điểm về **thân panel**
 * thay vì về nút. Nhánh dự phòng `querySelector('[data-…-open]')` sẵn có **không cứu được**:
 * nó chỉ chạy khi node đã lưu **rời DOM**, mà `section.panel` thì vẫn ở nguyên đó.
 *
 * ⚠️ Bản vá trước đó — `@mousedown` ép tiêu điểm lên nút — **có tác dụng thật** ở nút
 * titlebar (không tổ tiên nào focusable) và **bị vô hiệu** ở nút trong panel. Ghi ra để
 * lượt sau không dựng lại nó lần nữa ở chỗ nó không chạy.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * LUẬT, VÀ VÌ SAO NÓ HẸP HƠN *"LUÔN ƯU TIÊN NÚT MỞ"*
 * ═════════════════════════════════════════════════════════════════════════════════
 * Ưu tiên nút mở **chỉ khi** tiêu điểm đang ở chính nó hoặc ở một **tổ tiên** của nó — tức
 * đúng hình dạng *"cú bấm đã rơi vào nút, nhưng engine đỗ tiêu điểm ở khung ngoài"*.
 *
 * 🔴 Không ưu tiên vô điều kiện, vì hai lớp phủ này mở được **bằng phím** (cả hai là command
 * đã đăng ký, và Story 1.21 cho gán phím cho bất kỳ command nào). Khi người dùng đang gõ
 * trong một panel khác rồi bấm phím mở lớp phủ, *"nút đã mở"* **không tồn tại** — trả tiêu
 * điểm về nút khi đó là ném họ ra khỏi chỗ họ đang làm việc. Một luật vô điều kiện sẽ đổi
 * đúng lời hứa của UX-DR17 thành một lời hứa khác.
 *
 * @param openerSelector mối nối `data-` của nút mở, ví dụ `'[data-shortcuts-open]'`
 * @returns node để `focus()` lúc đóng, hoặc `null` khi không có ứng viên nào
 */
export function focusReturnTargetOnOpen(openerSelector: string): HTMLElement | null {
  if (typeof document === 'undefined') return null

  const active = document.activeElement
  const activeEl = active instanceof HTMLElement ? active : null
  const opener = document.querySelector<HTMLElement>(openerSelector)

  if (opener !== null && activeEl !== null && (activeEl === opener || activeEl.contains(opener))) {
    return opener
  }
  return activeEl
}
