/**
 * Hàm phân giải chuỗi giao diện — NFR16 · AD-21. Khoảng 40 dòng, và đó là toàn bộ
 * thứ dự án cần ở v1 (§Vì sao không dùng `vue-i18n` của Story 1.5).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⛔ TỆP NÀY KHÔNG ĐƯỢC `import` BẤT CỨ THỨ GÌ — và đó là một điều kiện kỹ thuật,
 * không phải một sở thích về kiến trúc.
 * ─────────────────────────────────────────────────────────────────────────────
 * AC4 là mệnh đề về HÀNH VI LÚC CHẠY (*"khoá thiếu ⇒ hiện khoá nguyên văn và ghi
 * cảnh báo, không sập"*). Nghiệm thu nó phải GỌI HÀM THẬT. Dự án không có bộ chạy
 * test frontend, và thêm một (`vitest`) là thêm một phụ thuộc phải rà GPLv3 và vào
 * bảng Stack trước (NFR15) — quyết định của Ice, không phải hệ quả phụ của Story 1.5.
 *
 * Đường không tốn gì: Node ≥ 22.18 bóc kiểu TypeScript mặc định, nên
 * `scripts/check-i18n.mjs` `import()` thẳng được tệp này (Kiểm E). Nhưng Node CHỈ
 * bóc kiểu — nó không phân giải `./vi.json` theo luật bundler của Vite và không
 * hiểu `.vue`. Một dòng `import` ở đây là Kiểm E chết, và AC4 quay về nghiệm thu
 * bằng mắt.
 *
 * ⛔ Cùng lý do: KHÔNG `enum`, KHÔNG `namespace`, KHÔNG parameter property
 * (`constructor(private x)`). Ba thứ đó Node từ chối bóc kiểu vì chúng SINH MÃ chứ
 * không chỉ mang chú thích. `type` / `interface` / annotation thì được.
 *
 * Chỗ duy nhất chạm `vi.json` và Vue là `./index.ts`.
 */

/** Danh mục chuỗi đã nạp: object PHẲNG, khoá chấm (AC1). */
export type MessageCatalog = Readonly<Record<string, string>>

/**
 * Tham số nội suy. **Giá trị là chuỗi**, kể cả số.
 *
 * Cùng luật với `params` của `IpcError` phía Rust: tham số mang DỮ LIỆU (đường dẫn,
 * số đếm, tên nhà cung cấp), không mang CÂU. Định dạng số và ngày giờ là việc của
 * nơi gọi, trước khi tới đây.
 */
export type MessageParams = Readonly<Record<string, string>>

export type Translate = (key: string, params?: MessageParams) => string

/**
 * Cú pháp nội suy: `{ten_tham_so}`, tên khớp `[a-z_][a-z0-9_]*`.
 *
 * Hẹp có chủ ý, và `scripts/check-i18n.mjs` Kiểm C cưỡng chế đúng dải này trên mọi
 * giá trị của `vi.json`: `{}`, `{Path}`, `{0}` và `{ path }` là FAIL ở cổng chứ
 * không phải một placeholder lặng lẽ không bao giờ khớp lúc chạy.
 */
const PLACEHOLDER_RE = /\{([a-z_][a-z0-9_]*)\}/g

const has = (obj: object, key: string): boolean =>
  Object.prototype.hasOwnProperty.call(obj, key)

/**
 * Dựng một hàm `t` trên một danh mục.
 *
 * Ba hành vi, và cả ba đều là "không ném":
 *   - khoá có     ⇒ trả giá trị, nội suy tham số
 *   - khoá thiếu  ⇒ trả **đúng khoá nguyên văn** + `console.warn` (AC4)
 *   - tham số thiếu ⇒ **giữ nguyên placeholder** + `console.warn`
 *
 * ⚠️ Tham số thiếu KHÔNG được thay bằng `undefined`. `"Không đọc được tệp tại
 * undefined."` là một câu hoàn chỉnh về mặt ngữ pháp và sẽ đi thẳng ra màn hình
 * người dùng; `{path}` còn nguyên thì ai nhìn cũng biết là lỗi lập trình.
 */
export function createResolver(catalog: MessageCatalog): Translate {
  /**
   * ⚠️ Dedupe KHÔNG phải tối ưu vặt. Một khoá thiếu trong template Vue được phân
   * giải lại MỖI LẦN RENDER — không dedupe thì console ngập vài nghìn dòng giống
   * hệt nhau và mọi cảnh báo thật chìm mất. Hai `Set` riêng: một khoá thiếu và một
   * tham số thiếu là hai lỗi khác nhau, gộp lại thì cái sau che cái trước.
   */
  const warnedKeys = new Set<string>()
  const warnedParams = new Set<string>()

  const warnOnce = (seen: Set<string>, id: string, message: string): void => {
    if (seen.has(id)) return
    seen.add(id)
    console.warn(message)
  }

  return (key: string, params?: MessageParams): string => {
    // ⚠️ Kiểu chỉ tồn tại lúc biên dịch. `t` là hàm xuất khẩu công khai và khoá của nó
    // có thể tới từ một payload IPC hay một binding động, nên một `key` không phải chuỗi
    // vẫn phải TRẢ VỀ CHUỖI: `Translate` khai `=> string`, và một binding Vue nhận
    // `undefined` render RỖNG — hỏng đúng kiểu AC4 tồn tại để chặn, tức im lặng.
    const safeKey = typeof key === 'string' ? key : String(key)
    if (!has(catalog, safeKey)) {
      warnOnce(warnedKeys, safeKey, `[i18n] khoá thiếu trong vi.json: "${safeKey}"`)
      return safeKey
    }
    const template = catalog[safeKey]

    return template.replace(PLACEHOLDER_RE, (placeholder: string, name: string): string => {
      const value = params === undefined ? undefined : params[name]
      // ⛔ CÓ MẶT chưa đủ — phải LÀ CHUỖI. Một phép kiểm `hasOwnProperty` trần đúng
      // cho `{path: null}` (JSON `null` là giá trị hợp lệ trên dây IPC) và cho
      // `{path: undefined}`, rồi `"Không đọc được tệp tại null."` đi thẳng ra màn hình như
      // một câu hoàn chỉnh về ngữ pháp — đúng thứ đoạn ⚠️ ở doc-comment trên cấm.
      if (typeof value !== 'string') {
        // Khoá dedupe gộp cả `key` — cùng một tham số thiếu ở hai chuỗi khác nhau
        // là hai chỗ gọi sai khác nhau, và người sửa cần thấy cả hai.
        //
        // ⛔ Dấu phân cách viết BẰNG ESCAPE `\u0000`, không phải một byte NUL THÔ gõ thẳng
        // vào tệp. Một byte NUL làm git phân loại tệp này là *binary*: `git diff` in
        // "Binary files … differ", `grep`/`git grep` bỏ qua nó, và tệp mang nhiều ⛔ nhất
        // của Story 1.5 trở thành tệp không ai review được bằng diff. Hai dạng tương đương
        // từng byte lúc chạy; chỉ khác ở chỗ một dạng đọc được.
        warnOnce(
          warnedParams,
          `${safeKey}\u0000${name}`,
          `[i18n] thiếu tham số "${name}" cho khoá "${safeKey}" — placeholder giữ nguyên`,
        )
        return placeholder
      }
      return value
    })
  }
}
