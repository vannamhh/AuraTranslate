/**
 * Story 2.9 · AC9 — **`SourceHanViet.vue` PHÁT ra neo chỗ cắt nào**, ở cả hai kiểu xem.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔵 TỆP NÀY SINH RA VÌ MỘT ĐỘT BIẾN KHÔNG AI BẮT ĐƯỢC — code review 2026-08-17
 * ═════════════════════════════════════════════════════════════════════════════════
 * Lượt code review thêm `data-src-atomic` cho mảnh `.hv-text` *(Ice ký: từ chối offset giữa mảnh)*.
 * Rồi nó gỡ đúng hai dòng ấy ra để thử — và **191/191 ca vẫn xanh**.
 *
 * 🔴 Nguyên nhân, và nó là một khuyết tật của LƯỚI chứ không của sản phẩm: `editorSourceCut.test.ts`
 * dựng DOM **bằng tay** rồi tự gắn `data-src-atomic`. Nó canh `sourceCutOffsetOf()` **tôn trọng**
 * neo — một mệnh đề đúng và đáng canh — nhưng **không** canh việc component **phát ra** neo. Hai
 * mệnh đề khác nhau, và cả hai phải có người canh: một chỗ cắt tàng hình xuất hiện khi vế **thứ
 * hai** hỏng, và vế thứ hai thì không tệp nào nhìn tới.
 *
 * ⇒ Nhóm này mount **component thật** và đọc DOM nó dựng. Đó là điều kiện duy nhất để một lượt sửa
 *   template trong tương lai không đi qua im lặng.
 *
 * ⚠️ **Không cần fixture từ điển.** Mệnh đề đang kiểm nằm ở mảnh **KHÔNG-Hán**, và nhánh đó không
 * tra âm. Ký tự Hán thiếu âm hiện bằng `READING_PLACEHOLDER` — hợp lệ, và không ca nào dưới đây
 * khẳng định gì về âm.
 */
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import SourceHanViet from '../../src/panels/SourceHanViet.vue'
import type { SegmentTermSpan } from '../../src/panels/glossaryMarksMap'

/** Nguyên văn có **đủ ba** loại mẩu: chữ Hán · mảnh không-Hán dài 2 · một chữ Hán nữa. */
const NGUYEN_VAN = '京都」，春風'
//                  0 1 2 3 4 5   ⇐ chỉ số ký tự nguồn

function dung(
  viewMode: 'switch' | 'parallel',
  cuts: readonly number[] = [],
  glossaryTerms: readonly SegmentTermSpan[] = [],
) {
  return mount(SourceHanViet, {
    props: { sourceText: NGUYEN_VAN, viewMode, cuts, glossaryTerms, surfaceRole: 'cell' as const },
  })
}

function term(start: number, end: number, isConfirmed = true, translation: string | null = 'X'): SegmentTermSpan {
  return { start, end, isConfirmed, translation: isConfirmed ? translation : null }
}

describe('SourceHanViet — neo chỗ cắt phát ra DOM', () => {
  for (const viewMode of ['switch', 'parallel'] as const) {
    it(`🔴 mảnh .hv-text mang data-src-atomic ở kiểu \`${viewMode}\``, () => {
      const w = dung(viewMode)
      // Mảnh không-Hán duy nhất là `」，`, bắt đầu ở chỉ số **2**.
      const manh = w.findAll('[data-src-start="2"]')
      expect(manh).toHaveLength(1)

      // 🔴 Mệnh đề của cả tệp: nếu neo này mất, một cú bấm giữa `」，` cho offset **3** — một chỗ
      // cắt Rust thực thi đúng mà `cutSet.has(seg.srcStart)` không vẽ được, tức tàng hình.
      expect(manh[0].attributes('data-src-atomic')).toBe('1')
      w.unmount()
    })

    it(`mọi phần tử mang neo đều khai một chỉ số HỮU HẠN ở kiểu \`${viewMode}\``, () => {
      const w = dung(viewMode)
      const co = w.findAll('[data-src-start]')
      expect(co.length).toBeGreaterThan(0)
      for (const el of co) {
        const n = Number(el.attributes('data-src-start'))
        expect(Number.isFinite(n)).toBe(true)
        // Chỉ số phải nằm trong biên nguyên văn — `sourceCutOffsetOf` không kiểm lại biên, và
        // Rust từ chối một offset vượt biên bằng một câu nói **sai nguyên nhân** với người dùng.
        expect(n).toBeGreaterThanOrEqual(0)
        expect(n).toBeLessThanOrEqual([...NGUYEN_VAN].length)
      }
      w.unmount()
    })

    it(`dấu cắt VẼ ĐƯỢC ở đầu mảnh .hv-text, kiểu \`${viewMode}\``, () => {
      // Đối chứng **hai chiều**: không có điểm cắt ⇒ không lớp `cut-here` nào ở mảnh đó.
      const khong = dung(viewMode, [])
      expect(khong.find('[data-src-start="2"]').classes()).not.toContain('cut-here')
      khong.unmount()

      const co = dung(viewMode, [2])
      expect(co.find('[data-src-start="2"]').classes()).toContain('cut-here')
      co.unmount()
    })
  }

  it('⚠️ base `<ruby>` ở `parallel` KHÔNG nguyên khối — AC9 đòi chính xác từng chữ', () => {
    // Ghi bằng một ca, không bằng một chú thích: neo `data-src-atomic` **không** được lan sang đây.
    // AC9 viết nguyên văn *"chính xác từng chữ ở `parallel` (base `<ruby>` có mặt)"*, nên một lượt
    // *"cho nhất quán"* thêm cờ vào đây là một lượt **lệch AC**, không một lượt dọn dẹp.
    //
    // 🔴 Và ca *"chỗ cắt giữa một TỪ HÁN ở `parallel` không vẽ được dấu"* vì thế **vẫn hở** — nó là
    // món nợ đã ghi có chủ ở `GridPanel.vue` §`pendingCuts`. Lượt code review 2026-08-17 **không**
    // đóng nó, và ca này là chỗ giữ cho nó không bị lặng lẽ đóng bằng một dòng thuộc tính.
    const w = dung('parallel')
    const unit = w.find('.hv-unit')
    expect(unit.exists()).toBe(true)
    expect(unit.attributes('data-src-atomic')).toBeUndefined()
    w.unmount()
  })

  it('⚠️ `.hv-word` ở `switch` NGUYÊN KHỐI — trên màn hình là ÂM, không chữ Hán', () => {
    const w = dung('switch')
    const word = w.find('.hv-word')
    expect(word.exists()).toBe(true)
    expect(word.attributes('data-src-atomic')).toBe('1')
    w.unmount()
  })
})

/**
 * Story 3.4b — **biên thuật ngữ Glossary cắt ở tầng dữ liệu**, cả hai kiểu xem.
 *
 * 🔴 Mệnh đề trung tâm của story: `buildSegments` phải FLUSH tại biên thuật ngữ bất kể ICU nói
 * gì — ranh giới `Matcher` (Rust) THẮNG ranh giới `Intl.Segmenter` (ICU). Ba ca dưới đây tương
 * ứng ĐÚNG ba hàng của I/O Matrix: *"phủ một phần"*, *"bắc cầu"*, *"Chương không có thuật ngữ"*.
 */
/** '文化' — ICU (tiếng Trung) gộp thành MỘT từ ("văn hoá"), đo bằng chính `Intl.Segmenter`
 * mà `wordBoundary.ts` dùng: `new Intl.Segmenter('zh', {granularity:'word'}).segment('文化')`
 * cho ra đúng MỘT phần `["文化"]`. Dùng làm đối chứng "ICU thắng khi KHÔNG có thuật ngữ nào". */
const MOT_TU_ICU = '文化'

function dungMotTuIcu(viewMode: 'switch' | 'parallel', glossaryTerms: readonly SegmentTermSpan[]) {
  return mount(SourceHanViet, {
    props: { sourceText: MOT_TU_ICU, viewMode, cuts: [], glossaryTerms, surfaceRole: 'cell' as const },
  })
}

describe('SourceHanViet — biên thuật ngữ Glossary cắt tại tầng dữ liệu (Story 3.4b)', () => {
  for (const viewMode of ['switch', 'parallel'] as const) {
    const selector = viewMode === 'switch' ? '.hv-word' : '.hv-unit'

    it(`thuật ngữ phủ MỘT PHẦN một từ ICU ⇒ tách thành HAI phần tử, kiểu \`${viewMode}\``, () => {
      const khong = dungMotTuIcu(viewMode, [])
      // Đối chứng: KHÔNG có thuật ngữ ⇒ ICU thắng ⇒ đúng MỘT phần tử cho cả cụm.
      expect(khong.findAll(selector)).toHaveLength(1)
      khong.unmount()

      const co = dungMotTuIcu(viewMode, [term(0, 1)])
      const nodes = co.findAll(selector)
      // 🔴 Mệnh đề trung tâm: ranh giới Matcher THẮNG ICU ⇒ HAI phần tử, không một.
      expect(nodes).toHaveLength(2)
      expect(nodes[0]?.attributes('data-src-start')).toBe('0')
      expect(nodes[1]?.attributes('data-src-start')).toBe('1')
      // Chỉ phần tử đầu (khớp ĐÚNG span [0,1)) mang dấu; phần tử thứ hai ('化') thì không.
      expect(nodes[0]?.classes()).toContain('glossary-confirmed')
      expect(nodes[1]?.classes()).not.toContain('glossary-confirmed')
      expect(nodes[1]?.classes()).not.toContain('glossary-pending')
      co.unmount()
    })

    it(`bàn ánh xạ DOM ↔ segment vẫn khớp một-một sau khi cắt — data-src-atomic giữ nguyên đầu nhóm cũ, kiểu \`${viewMode}\``, () => {
      const w = dungMotTuIcu(viewMode, [term(0, 1)])
      const nodes = w.findAll(selector)
      // 🔴 Mảnh sinh bởi biên thuật ngữ giữ đúng khuôn mảnh do ICU sinh — `data-src-atomic`
      // báo đầu nhóm nguyên tử CỦA CHÍNH NÓ (không phải đầu nhóm ICU gốc trước khi bị chia),
      // đúng chữ ký của Ice *"từ chối offset giữa mảnh"*: mỗi mảnh MỚI vẫn nguyên khối theo
      // đúng luật cũ của kiểu xem đó (`switch` atomic, `parallel` không).
      if (viewMode === 'switch') {
        for (const n of nodes) expect(n.attributes('data-src-atomic')).toBe('1')
      } else {
        for (const n of nodes) expect(n.attributes('data-src-atomic')).toBeUndefined()
      }
      w.unmount()
    })
  }

  it('thuật ngữ BẮC CẦU hai từ ICU ⇒ cả hai bị tách và cùng mang dấu — liền mạch thị giác', () => {
    // '你好世界' — ICU tách '你好' (chào)/'世界' (thế giới) thành HAI từ riêng. Thuật ngữ giả
    // định phủ '好世' = [1, 3), bắc cầu qua đúng ranh giới đó.
    const w = mount(SourceHanViet, {
      props: {
        sourceText: '你好世界',
        viewMode: 'switch',
        cuts: [],
        glossaryTerms: [term(1, 3)],
        surfaceRole: 'cell' as const,
      },
    })
    const words = w.findAll('.hv-word')
    // Biên thuật ngữ {1, 3} cộng biên ICU tại {0, 2} (đầu mỗi từ) ⇒ bốn mảnh MỘT KÝ TỰ.
    expect(words).toHaveLength(4)
    const starts = words.map((n) => n.attributes('data-src-start'))
    expect(starts).toEqual(['0', '1', '2', '3'])
    // '你'[0,1) và '界'[3,4) KHÔNG thuộc span [1,3); '好'[1,2) và '世'[2,3) THUỘC — cả hai
    // cùng mang lớp `glossary-confirmed`, tức về mặt thị giác nối liền dù là hai node DOM.
    expect(words[0]?.classes()).not.toContain('glossary-confirmed')
    expect(words[1]?.classes()).toContain('glossary-confirmed')
    expect(words[2]?.classes()).toContain('glossary-confirmed')
    expect(words[3]?.classes()).not.toContain('glossary-confirmed')
    w.unmount()
  })

  it('Chương không có thuật ngữ nào (`glossaryTerms` rỗng/không truyền) ⇒ 0 mảnh bị cắt thêm, DOM y hệt trước story', () => {
    const khongTruyen = mount(SourceHanViet, {
      props: { sourceText: NGUYEN_VAN, viewMode: 'switch', cuts: [], surfaceRole: 'cell' as const },
    })
    const rong = dung('switch', [], [])
    // Không truyền prop hoàn toàn (mặc định `undefined`) và truyền mảng RỖNG cho ra ĐÚNG cùng
    // số phần tử — cả hai đường phải vô hại như nhau.
    expect(khongTruyen.findAll('.hv-word, .hv-text').length).toBe(rong.findAll('.hv-word, .hv-text').length)
    for (const el of khongTruyen.findAll('[class*="glossary-"]')) {
      expect(el.classes().some((c) => c.startsWith('glossary-'))).toBe(false)
    }
    khongTruyen.unmount()
    rong.unmount()
  })

  it('chờ chốt (`isConfirmed: false`) ⇒ lớp `glossary-pending`, KHÔNG `glossary-confirmed`', () => {
    const w = dung('switch', [], [term(0, 1, false, null)])
    const first = w.find('.hv-word')
    expect(first.classes()).toContain('glossary-pending')
    expect(first.classes()).not.toContain('glossary-confirmed')
    w.unmount()
  })
})
