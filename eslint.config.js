// @ts-check
/**
 * Cổng thứ MƯỜI — lớp lỗi mà chín cổng kia không canh được.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI — một phép đo, không một sở thích
 * ─────────────────────────────────────────────────────────────────────────────
 * Code review 2026-08-11 của Story 1.21 bắt hai lỗi cùng một hình dạng:
 *
 *     if (captureIsArmed)          // `captureIsArmed` là một Ref, KHÔNG `.value`
 *
 * Vue chỉ tự bóc `Ref` trong `template`; trong khối `script` thì không. Nên phép thử
 * chạy trên chính **đối tượng** `Ref` và luôn luôn đúng. Hậu quả đo được: `Escape`
 * không bao giờ đóng được lớp phủ phím tắt, và nhánh `⌫` bỏ gán là mã CHẾT ở mọi
 * trạng thái — hai hành vi mà `shortcuts.gesture` hứa với người dùng bằng chữ.
 *
 * Nó đi qua được **chín trên chín cổng**, `vue-tsc --noEmit` HAI lượt, và `vite build`,
 * vì `if (someObject)` là TypeScript hoàn toàn hợp lệ. Chín cổng của dự án canh chuỗi
 * hiển thị, token màu, `@click`, `dispatch`, sàn quần thể, phạm vi filesystem — không
 * cái nào canh một biểu thức điều kiện luôn đúng. Đây là lỗ hổng CẤP DỰ ÁN, không một
 * lỗi của Story 1.21.
 *
 * ⚠️ Luật của mọi cổng trong kho này áp cả ở đây: cổng phải trả mã thoát khác 0 khi
 * thất bại. `eslint` làm đúng thế sẵn — không cần vỏ bọc.
 *
 * Chạy:  npm run check:lint
 */
import tseslint from 'typescript-eslint'
import pluginVue from 'eslint-plugin-vue'

/**
 * Bật lượt lint CÓ KIỂU. Bắt buộc — không có nó thì `no-unnecessary-condition` im lặng
 * không làm gì, tức một cổng xanh mà không canh gì: đúng thứ doc-comment của
 * `check-deps.mjs` gọi là *"script không cưỡng chế được gì"*.
 *
 * `extraFileExtensions` để parser chịu đưa tệp `.vue` vào chương trình TypeScript;
 * `tsconfig.json` đã `include` cả `src/**​/*.vue` sẵn.
 */
const TYPED = {
  projectService: true,
  tsconfigRootDir: import.meta.dirname,
  extraFileExtensions: ['.vue'],
}

export default tseslint.config(
  {
    /**
     * 🔴 Một miễn trừ HẾT CẦN phải bị bắt, không được lặng lẽ ở lại.
     *
     * Mặc định của ESLint là `'warn'`, và một cảnh báo trong một cổng chỉ đọc mã thoát là
     * một cảnh báo không ai đọc. Nâng lên `'error'` vì lượt dựng 13 miễn trừ đầu tiên của
     * kho này đã **sai hình dạng** — `eslint-disable-next-line` đứng trước ba dòng chú thích
     * tiếp nối nên nó trỏ vào một dòng chú thích, và 13 guard thật vẫn đỏ. Đúng phép kiểm
     * này bắt được chuyện đó. Nó cũng là thứ ngăn một miễn trừ sống sót sau khi kiểu bên
     * dưới đã được sửa cho trung thực — tức ngăn chính lớp nợ mà tệp này tồn tại để chống.
     */
    linterOptions: { reportUnusedDisableDirectives: 'error' },
  },
  {
    // `src-tauri` là Rust. `scripts/` là chín cổng kia — Node thuần, không kiểu, và
    // chúng tự canh nhau bằng Kiểm D của chính mình. `dist/` là sản phẩm build.
    ignores: ['dist/**', 'src-tauri/**', 'scripts/**', 'eslint.config.js', 'vite.config.ts'],
  },

  // Nền: đăng ký plugin + parser của typescript-eslint.
  // ⚠️ KHÔNG spread — `configs.base` là một object đơn, khác `pluginVue.configs['flat/base']`
  // bên dưới là một mảng. Spread nó cho `TypeError: object is not iterable`.
  tseslint.configs.base,

  // `.vue` cần `vue-eslint-parser` ở ngoài, `tseslint.parser` cho khối `<script lang="ts">`.
  ...pluginVue.configs['flat/base'],

  {
    files: ['src/**/*.ts', 'tests/frontend/**/*.ts'],
    languageOptions: { parserOptions: TYPED },
  },

  {
    // 🔴 `.vue` cần HAI parser lồng nhau, và bỏ vế trong là cổng chết:
    // `vue-eslint-parser` đọc tệp `.vue`, rồi `parserOptions.parser` nói nó chuyển
    // khối `<script lang="ts">` xuống parser CÓ KIỂU. Thiếu dòng đó,
    // `no-unnecessary-condition` ném *"You have used a rule which requires type
    // information"* trên tệp `.vue` đầu tiên — và đúng những tệp `.vue` là chỗ lỗi
    // của Story 1.21 đã sống.
    files: ['src/**/*.vue'],
    languageOptions: { parserOptions: { ...TYPED, parser: tseslint.parser } },
  },

  {
    files: ['src/**/*.ts', 'src/**/*.vue', 'tests/frontend/**/*.ts'],
    rules: {
      /**
       * 🔴 ĐÂY là luật bắt được lỗi đã lọt, và nó là lý do cả tệp này tồn tại.
       *
       * Nó có KIỂU, nên nó thấy `captureIsArmed` mang `DeepReadonly<Ref<boolean>>` —
       * một kiểu đối tượng, luôn truthy — và báo *"Unnecessary conditional, value is
       * always truthy"*. Nó thấy được điều đó **xuyên biên giới module**, tức nó bắt
       * được đúng ca của Story 1.21: `Ref` khai ở `config/shortcutsState.ts`, dùng sai
       * ở `ShortcutsOverlay.vue`.
       *
       * ⚠️ `vue/no-ref-as-operand` KHÔNG bắt được ca đó — nó chỉ theo dõi `ref()` khai
       * trong CÙNG một tệp. Nó vẫn được bật bên dưới vì nó bắt một tập khác *(quên
       * `.value` trên ref cục bộ)*, nhưng nó không thay thế được luật này.
       */
      '@typescript-eslint/no-unnecessary-condition': 'error',
      'vue/no-ref-as-operand': 'error',
    },
  },

  {
    /**
     * 🔴 Bàn đo e2e: cấm `element.click()` của driver — Story 1.22, AC3.
     *
     * ─────────────────────────────────────────────────────────────────────────
     * MỘT PHÉP ĐO, VÀ MỘT MÂU THUẪN ĐÃ XẢY RA THẬT
     * ─────────────────────────────────────────────────────────────────────────
     * Đo 2026-08-11: lệnh `click` của driver bắn `click` **TRƯỚC** `focusin`, ngược
     * chuột thật (`mousedown -> focusin -> mouseup -> click`). Hệ quả đo được ở
     * `shortcuts-capture-mouse`: `shortcuts.capture` chạy lúc `aimedRow` còn rỗng ⇒ một
     * lượt ĐỎ **đổ lỗi cho sản phẩm** trong khi lỗi ở bộ lái.
     *
     * Chiều ngược lại im lặng hơn và đắt hơn: ở một hàng mà thứ tự quyết định kết quả,
     * `.click()` cho một lượt **XANH trên sản phẩm đang hỏng**.
     *
     * ⚠️ Và nó đã xảy ra: `shortcuts-focus.e2e.mjs` khai bằng chữ rằng nó *"cố ý đi
     * đường chuột"* để bắt khuyết tật WKWebView, rồi gọi `opener.click()`. Chú thích nói
     * một đằng, mã đi một nẻo. Đó là lý do luật này là một CỔNG chứ không một quy ước —
     * câu hỏi *"hàng này có phụ thuộc thứ tự không"* đã được trả lời sai một lần rồi.
     *
     * Lệnh cấm TOÀN PHẦN, rộng hơn phát biểu của AC3 *(chỉ đòi ở nơi thứ tự có nghĩa)*,
     * có chủ ý: giá của việc luôn đi `realClick()` là vài chục mili-giây, còn giá của
     * một lần phân xử sai là một bàn đo mất đúng lớp lỗi nó được dựng ra để bắt.
     *
     * Ngoại lệ thật thì viết `eslint-disable-next-line no-restricted-syntax` kèm lý do
     * ngay tại chỗ; `reportUnusedDisableDirectives: 'error'` ở đầu tệp bắt được ngoại lệ
     * hết cần.
     *
     * ⚠️ Giới hạn ghi thẳng: selector bắt theo **tên thuộc tính**, nên
     * `el['cli' + 'ck']()` lọt. Đó là một hình dạng mã không tồn tại trong `e2e/` và
     * viết nó ra là cố tình lách một cổng — thứ không cổng nào chặn được.
     */
    files: ['e2e/**/*.mjs'],
    rules: {
      'no-restricted-syntax': [
        'error',
        {
          selector: "CallExpression[callee.type='MemberExpression'][callee.property.name='click']",
          message:
            'Không dùng `.click()` của driver trong bàn đo e2e — nó bắn `click` TRƯỚC ' +
            '`focusin`, nên nó vừa cho ĐỎ sai nguyên nhân, vừa cho XANH trên một sản ' +
            'phẩm đang hỏng. Dùng `realClick()` ở `e2e/support/pointer.mjs` (Actions API, ' +
            'dựng đúng chuỗi `mousedown -> focusin -> mouseup -> click`). Lý do đầy đủ và ' +
            'số đo ở doc-comment của tệp đó.',
        },
      ],
    },
  },
)
