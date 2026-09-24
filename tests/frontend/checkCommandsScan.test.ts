/**
 * Unit tests for `scripts/lib/commands-scan.mjs`, the pure text scanner `check-commands.mjs`
 * uses to enforce AD-34 §1 (it lives in its own module because the gate itself is not
 * import-safe: top-level `await`, `process.exit`).
 */
import { describe, it, expect } from 'vitest'
import {
  maskScript,
  maskTemplate,
  attributesIn,
  scanVueAttrs,
  functionBodyRange,
} from '../../scripts/lib/commands-scan.mjs'

function maskedScript(text: string, blankLiterals = false): string {
  const chars = text.split('')
  maskScript(text, 0, text.length, chars, { blankLiterals })
  return chars.join('')
}

function maskedTemplate(text: string): string {
  const chars = text.split('')
  maskTemplate(text, 0, text.length, chars)
  return chars.join('')
}

describe('maskScript — che comment, giữ nguyên offset', () => {
  it('che `//` tới cuối dòng, giữ nguyên số dòng (đếm `\\n`)', () => {
    const src = "const a = 1 // dispatch('nen.bi.che')\nconst b = 2"
    const masked = maskedScript(src)
    expect(masked).not.toContain("dispatch('nen.bi.che')")
    expect(masked.split('\n').length).toBe(src.split('\n').length)
  })

  it('che `/* … */` kể cả nhiều dòng, giữ số dòng bằng cách giữ lại `\\n`', () => {
    const src = "const a = 1 /* dispatch('nen.bi.che')\nvan con o day */ const b = 2"
    const masked = maskedScript(src)
    expect(masked).not.toContain("dispatch('nen.bi.che')")
    expect(masked.split('\n').length).toBe(src.split('\n').length)
  })

  it('KHÔNG che chuỗi ký tự ở view `masked` mặc định — Kiểm B cần đọc literal id', () => {
    const src = "dispatch('mode.library')"
    expect(maskedScript(src)).toBe(src)
  })

  it("một dấu `//` bên TRONG một chuỗi không mở comment giả và không nuốt `dispatch(` sau nó", () => {
    const src = "const url = 'https://x' \n dispatch('mode.library')"
    const masked = maskedScript(src)
    expect(masked).toContain("dispatch('mode.library')")
  })

  it('một regex literal chứa `\\/` không bị đọc thành `//` rồi che nốt dòng', () => {
    const src = "const re = /^https?:\\/\\//\ndispatch('mode.library')"
    const masked = maskedScript(src)
    expect(masked).toContain("dispatch('mode.library')")
  })

  it('một `/` sau `}` (đóng block) vẫn được coi là mở regex, không phải phép chia', () => {
    const src = "function f() {}\n/^https?:\\/\\//.test('x')\ndispatch('mode.library')"
    const masked = maskedScript(src)
    expect(masked).toContain("dispatch('mode.library')")
  })
})

describe('maskScript({ blankLiterals: true }) — view `code`, dựng cho một lời gọi giả trong chuỗi', () => {
  it('che nội dung chuỗi ký tự, KHÔNG đổi độ dài văn bản', () => {
    const src = "const msg = 'toi khong phai mot lenh that'\ndispatch('mode.library')"
    const code = maskedScript(src, true)
    expect(code).not.toContain('toi khong phai mot lenh that')
    expect(code).not.toContain('mode.library')
    expect(code.length).toBe(src.length)
    expect(code).toContain('dispatch(')
  })

  it("che nội dung template literal, giữ nguyên phần `${...}` — đó là mã thật", () => {
    const src = "const msg = `noi dung gia ${dispatch('mode.library')} con lai`"
    const code = maskedScript(src, true)
    expect(code).not.toContain('noi dung gia')
    expect(code).not.toContain('con lai')
    expect(code).toContain('dispatch(')
    expect(code).not.toContain('mode.library')
  })

  it('KHÔNG đổi view `masked` mặc định — `blankLiterals` là bổ sung, không thay thế', () => {
    const src = "const msg = 'toi khong phai mot lenh that'"
    expect(maskedScript(src, false)).toBe(src)
  })

  it('I/O: "lời gọi giả trong một chuỗi" — `masked` đếm 2, `code` đếm ĐÚNG 1 lời gọi thật', () => {
    const src = [
      "const warn = 'đừng viết useSelectionSurface(original, \\'source\\') trong prose'",
      "useSelectionSurface(cellRef, 'source')",
    ].join('\n')
    const CALL_RE = /useSelectionSurface\s*\(/g

    const masked = maskedScript(src)
    const inMasked = [...masked.matchAll(CALL_RE)].length
    expect(inMasked).toBe(2)

    const code = maskedScript(src, true)
    const inCode = [...code.matchAll(CALL_RE)].length
    expect(inCode).toBe(1)
  })
})

describe('maskTemplate — che `<!-- -->`, chỉ ở vùng văn bản', () => {
  it('che một comment HTML, giữ nguyên số dòng', () => {
    const src = '<div>\n<!-- @click="dispatch(\'an.di\')" -->\n<span>x</span>\n</div>'
    const masked = maskedTemplate(src)
    expect(masked).not.toContain("dispatch('an.di')")
    expect(masked.split('\n').length).toBe(src.split('\n').length)
  })

  it('`<!--` bên trong một giá trị attribute KHÔNG mở comment giả', () => {
    const src = '<div title="a <!-- b"><span @click="dispatch(\'that.di\')">x</span></div>'
    const masked = maskedTemplate(src)
    expect(masked).toContain("dispatch('that.di')")
  })
})

describe('attributesIn — bóc thuộc tính trong vùng template, có trạng thái', () => {
  it('đọc tên và giá trị của một thuộc tính trong dấu nháy kép', () => {
    const src = '<button @click="dispatch(\'mode.library\')">Mở</button>'
    const attrs = attributesIn(src, 0, src.length)
    const click = attrs.find((a) => a.name === '@click')
    expect(click?.value).toBe("dispatch('mode.library')")
  })

  it('giá trị attribute được phép chứa `>` — chỗ đóng là dấu nháy, không phải `>`', () => {
    const src = '<button @click="a > b ? f() : g()">Mở</button>'
    const attrs = attributesIn(src, 0, src.length)
    const click = attrs.find((a) => a.name === '@click')
    expect(click?.value).toBe('a > b ? f() : g()')
  })

  it('đọc được giá trị KHÔNG có dấu nháy (kết thúc ở khoảng trắng hoặc `>`)', () => {
    const src = '<input type=text disabled>'
    const attrs = attributesIn(src, 0, src.length)
    expect(attrs.find((a) => a.name === 'type')?.value).toBe('text')
  })
})

describe('functionBodyRange — vùng thân hàm trên view `code`, dựng cho bảng handler của Kiểm K', () => {
  it('tìm đúng thân hàm, bỏ ngoài phần khai chữ ký và phần sau dấu `}` đóng', () => {
    const src = "function onEditKeydown(event) {\n  dispatch('editor.merge_segments')\n}\nconst after = 1"
    const code = maskedScript(src, true)
    const range = functionBodyRange(code, 'onEditKeydown')
    expect(range).not.toBeNull()
    const body = src.slice(range!.start, range!.end)
    expect(body).toContain("dispatch('editor.merge_segments')")
    expect(body).not.toContain('const after')
  })

  it('đếm ngoặc đúng qua một khối lồng nhau (if bên trong hàm)', () => {
    const src = [
      'function onKeydown(event) {',
      '  if (event.key === "Escape") {',
      "    dispatch('a.b')",
      '  }',
      "  dispatch('c.d')",
      '}',
    ].join('\n')
    const code = maskedScript(src, true)
    const range = functionBodyRange(code, 'onKeydown')
    const body = src.slice(range!.start, range!.end)
    expect(body).toContain("dispatch('a.b')")
    expect(body).toContain("dispatch('c.d')")
  })

  it('trả `null` khi hàm không khai báo cục bộ trong tệp — ca hàm đến từ import', () => {
    const src = "closeAttribution()\nconst x = 1"
    const code = maskedScript(src, true)
    expect(functionBodyRange(code, 'closeAttribution')).toBeNull()
  })

  it('một `{` giả bên trong một chuỗi không làm lệch độ sâu ngoặc — nhờ view `code` đã che nó', () => {
    const src = "function onScrimKeydown(event) {\n  const s = 'a { b'\n}\nconst after = 2"
    const code = maskedScript(src, true)
    const range = functionBodyRange(code, 'onScrimKeydown')
    const body = src.slice(range!.start, range!.end)
    expect(body).not.toContain('const after')
  })

  it('không khớp một tên là TIỀN TỐ của tên hàm thật — `\\s*\\(` ngay sau tên đã tự loại nó', () => {
    const src = 'function onKeydownExtra(event) {\n  return 1\n}'
    const code = maskedScript(src, true)
    expect(functionBodyRange(code, 'onKeydown')).toBeNull()
  })
})

describe('scanVueAttrs — gộp hai vòng lặp (tệp × vùng) mà mỗi Kiểm từng viết lại', () => {
  it('chỉ quét tệp `.vue`, bỏ qua tệp `.ts`, và trả về mọi thuộc tính không lọc theo tên', () => {
    const vueText = '<button @click="dispatch(\'a.b\')" title-key="a.title">Mở</button>'
    // The `.ts` fixture carries a real attribute so removing the `isVue` filter surfaces
    // an extra attribute, not just an unchanged empty array.
    const tsText = '<div data-fake="tu-mot-tep-khong-phai-vue">x</div>'
    const found = scanVueAttrs([
      {
        file: 'src/Demo.vue',
        text: vueText,
        masked: vueText,
        templates: [{ start: 0, end: vueText.length }],
        isVue: true,
      },
      {
        file: 'src/demo.ts',
        text: tsText,
        masked: tsText,
        templates: [{ start: 0, end: tsText.length }],
        isVue: false,
      },
    ])
    expect(found.map((f) => f.a.name).sort()).toEqual(['@click', 'title-key'])
    expect(found.every((f) => f.p.file === 'src/Demo.vue')).toBe(true)
  })

  it('không có tệp `.vue` nào ⇒ mảng rỗng, không ném', () => {
    expect(
      scanVueAttrs([
        { file: 'src/demo.ts', text: '', masked: '', templates: [], isVue: false },
      ]),
    ).toEqual([])
  })
})
