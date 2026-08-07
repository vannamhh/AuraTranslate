/**
 * `CommandRegistry` — nửa thứ nhất của AD-34. Story 1.6 · FR22 · NFR17.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * TỆP NÀY KHÔNG ĐƯỢC `import` BẤT CỨ THỨ GÌ — điều kiện kỹ thuật, không phải
 * sở thích kiến trúc.
 * ─────────────────────────────────────────────────────────────────────────────
 * AC1, AC2 và AC6 là mệnh đề về HÀNH VI LÚC CHẠY (*"id trùng bị phát hiện lúc
 * đăng ký"*, *"liệt kê được thao tác chưa gán phím"*). Nghiệm thu chúng phải GỌI
 * HÀM THẬT. Dự án không có bộ chạy test frontend, và thêm một (`vitest`) là thêm
 * một phụ thuộc phải rà GPLv3 và vào bảng Stack trước (NFR15) — quyết định của
 * Ice, không phải hệ quả phụ của story này.
 *
 * Đường không tốn gì, và nó ĐÃ CHẠY THẬT trong CI từ Story 1.5 với `resolve.ts`:
 * Node ≥ 22.18 bóc kiểu TypeScript mặc định, nên `scripts/check-commands.mjs`
 * `import()` thẳng được tệp này (Kiểm C). Nhưng Node CHỈ bóc kiểu — nó không hiểu
 * `.vue`, không phân giải `./vi.json` theo luật bundler của Vite. Một dòng
 * `import` giá trị ở đây là Kiểm C chết, và ba AC quay về nghiệm thu bằng mắt.
 *
 * Cùng lý do: KHÔNG `enum`, KHÔNG `namespace`, KHÔNG parameter property
 * (`constructor(private x)`). Ba thứ đó Node từ chối bóc kiểu vì chúng SINH MÃ
 * chứ không chỉ mang chú thích. `type` / `interface` / annotation thì được.
 *
 * Nửa thứ hai của AD-34 (sổ điểm vào focus) ở `./focus.ts`; tầng bàn phím ở
 * `./keys.ts`; chỗ DUY NHẤT đăng ký là `./index.ts`.
 */

/** Id của một thao tác. Văn phạm ở `COMMAND_ID_RE`. */
export type CommandId = string

export type CommandSpec = {
  /** Khoá chấm có tiền tố miền: `mode.library`, `focus.next_panel`. */
  id: CommandId
  /** Khoá chuỗi giao diện trong `vi.json`. Quy ước: `'command.' + id`. */
  labelKey: string
  /** Thao tác thật. Không đăng ký một command rỗng cho đủ số. */
  run: () => void
  /** Hợp âm TRUNG LẬP nền tảng: `'Mod+1'`. Vắng hoặc rỗng ⇒ lọt vào `unbound()`. */
  keys?: readonly string[]
}

export type Registry = {
  /** ⚠️ NÉM khi: id trùng · id sai văn phạm · `labelKey` rỗng · `run` không phải hàm. */
  register(spec: CommandSpec): void
  has(id: CommandId): boolean
  /** ⚠️ NÉM khi id chưa đăng ký — nửa cưỡng chế lúc chạy của AC1. */
  dispatch(id: CommandId): void
  /** Thứ tự ĐĂNG KÝ, ổn định. Story 1.21 hiển thị đúng danh sách này. */
  list(): readonly CommandSpec[]
  /** AC6 — các thao tác chưa gán phím nào. */
  unbound(): readonly CommandSpec[]
}

/**
 * 🔴 CHÉP ĐÚNG `KEY_RE` của `scripts/check-i18n.mjs` Kiểm B — không phải một biến thể.
 *
 * AD-34 nói command id *"cùng hình dạng khoá `vi.json`"*. "Cùng hình dạng" nghĩa là
 * CÙNG MỘT BIỂU THỨC, chép đúng. Lượt review Story 1.5 đã bắt được một ca hai phép
 * kiểm cưỡng chế hai văn phạm khoá khác nhau cho cùng một thứ (`ipc_contract.rs` vs
 * Kiểm B); đừng tạo ca thứ hai. ≥ 1 dấu chấm là bắt buộc: id phải có tiền tố miền.
 */
export const COMMAND_ID_RE = /^[a-z0-9]+(\.[a-z0-9_]+)+$/

/**
 * Đóng băng một spec trước khi cất vào kho.
 *
 * ⚠️ `list()` và `unbound()` trả BẢN SAO của mảng, nhưng bản sao mảng không ngăn được
 * người gọi ghi đè `spec.keys` của chính đối tượng bên trong. Story 1.21 dựng màn hình
 * gán phím trên `unbound()`; một lượt "sửa tại chỗ" ở đó sẽ đổi kho mà không đi qua
 * `register()`, tức trốn hết ba phép cưỡng chế. Đóng băng ở cửa vào là chỗ rẻ nhất.
 */
const frozen = (spec: CommandSpec): CommandSpec =>
  Object.freeze({
    id: spec.id,
    labelKey: spec.labelKey,
    run: spec.run,
    keys: Object.freeze([...(spec.keys ?? [])]),
  })

export function createRegistry(): Registry {
  /**
   * `Map` giữ thứ tự chèn theo đặc tả ECMAScript, nên `list()` không cần một mảng
   * song song. ⚠️ `Object.keys` thì KHÔNG đảm bảo điều đó với khoá dạng số — và
   * `list()` là thứ Story 1.21 render ra màn hình, nên một thứ tự nhảy chỗ mỗi lần
   * mở là một lỗi người dùng thấy được.
   */
  const byId = new Map<CommandId, CommandSpec>()

  const register = (spec: CommandSpec): void => {
    // ⚠️ Kiểm hình dạng TRƯỚC kiểm trùng: một `spec` là `undefined` phải nói ra điều
    // đó, không phải ném `TypeError` ở dòng đọc `.id`.
    if (spec === null || typeof spec !== 'object') {
      throw new TypeError(`[commands] register() nhận ${String(spec)}, phải là một CommandSpec.`)
    }
    const id = spec.id
    if (typeof id !== 'string' || !COMMAND_ID_RE.test(id)) {
      throw new Error(
        `[commands] id \`${String(id)}\` sai văn phạm — phải khớp ${COMMAND_ID_RE.source} ` +
          '(khoá chấm có tiền tố miền, cùng hình dạng khoá `vi.json`).',
      )
    }
    /**
     * 🔴 NÉM, không `console.warn`. AD-34 gọi tên hố này: *"hai giai đoạn cách nhau
     * nhiều tháng đăng ký trùng id trần sẽ ghi đè nhau âm thầm"*, và biểu hiện là
     * *"phím tắt X bỗng làm việc Y"* — không ai lần ra được. Một cảnh báo trong biển
     * log lúc khởi động là im lặng theo nghĩa thực dụng; một lần ném thì đỏ ngay ở
     * màn hình đầu tiên, rẻ nhất có thể.
     */
    if (byId.has(id)) {
      throw new Error(
        `[commands] id \`${id}\` đã đăng ký rồi — đăng ký trùng KHÔNG ghi đè im lặng (AC2). ` +
          'Đổi id, hoặc gộp hai chỗ đăng ký thành một.',
      )
    }
    if (typeof spec.labelKey !== 'string' || spec.labelKey.trim() === '') {
      throw new Error(
        `[commands] \`${id}\` thiếu \`labelKey\` — một thao tác không có nhãn là một thao tác ` +
          'không gọi tên được ở màn hình gán phím (Story 1.21).',
      )
    }
    if (typeof spec.run !== 'function') {
      // Một command rỗng đăng ký cho đủ số là đúng thứ story này tồn tại để chặn.
      throw new TypeError(`[commands] \`${id}\` thiếu \`run\` — command phải có thao tác thật.`)
    }
    byId.set(id, frozen(spec))
  }

  const has = (id: CommandId): boolean => byId.has(id)

  const dispatch = (id: CommandId): void => {
    const spec = byId.get(id)
    /**
     * Nửa cưỡng chế LÚC CHẠY của AC1. Cổng canh cú pháp `@click` trong `.vue`;
     * `dispatch` canh mọi đường còn lại — một `dispatch` gọi từ `.ts`, từ một handler
     * bàn phím, từ một chỗ Story 1.21 chưa tồn tại. Không rơi im lặng: một id gõ
     * sai mà không ném là một nút bấm không làm gì và không ai biết vì sao.
     */
    if (!spec) {
      throw new Error(
        `[commands] \`${String(id)}\` chưa đăng ký — mọi thao tác phải đăng ký ở CommandRegistry ` +
          'TRƯỚC khi bind vào chuột hoặc phím (AD-34 §1).',
      )
    }
    spec.run()
  }

  /** ⚠️ Bản sao. Kho nội bộ không rò ra ngoài — xem doc-comment của `frozen()`. */
  const list = (): readonly CommandSpec[] => [...byId.values()]

  const unbound = (): readonly CommandSpec[] =>
    [...byId.values()].filter((spec) => (spec.keys?.length ?? 0) === 0)

  return { register, has, dispatch, list, unbound }
}
