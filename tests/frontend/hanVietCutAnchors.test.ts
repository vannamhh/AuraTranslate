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

/** Nguyên văn có **đủ ba** loại mẩu: chữ Hán · mảnh không-Hán dài 2 · một chữ Hán nữa. */
const NGUYEN_VAN = '京都」，春風'
//                  0 1 2 3 4 5   ⇐ chỉ số ký tự nguồn

function dung(viewMode: 'switch' | 'parallel', cuts: readonly number[] = []) {
  return mount(SourceHanViet, {
    props: { sourceText: NGUYEN_VAN, viewMode, cuts, surfaceRole: 'cell' as const },
  })
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
