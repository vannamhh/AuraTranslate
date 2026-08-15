/**
 * Bảng ánh xạ *trạng thái segment → giá trị vạch lề* — Story 2.5, AC10 · Quyết định #1 và #3.
 *
 * ⚠️ **Vai của tệp này, và nó KHÔNG chồng lên cổng tĩnh.** Kiểm I của `check-commands.mjs`
 * canh một mệnh đề **khai báo trên toàn cây** — *"`SEGMENT_RULE_VALUES` có ĐÚNG sáu giá trị,
 * và mỗi giá trị có đúng một khối `.gmark.rule-*`"*. Tệp này canh một hạng **khác**: hàm phân
 * giải **trả về đúng giá trị nào** cho từng tổ hợp dữ kiện. Hai đường, hai mệnh đề (AC25).
 *
 * ⚠️ **Không mệnh đề nào ở đây nói về HÌNH HỌC.** `happy-dom` không phải WebKit; *"hai vạch
 * cùng một dòng"* và *"vạch cao đúng bằng câu"* thuộc **bàn đo**
 * (`2-5-ban-do-hai-vach.html`), không thuộc vitest.
 */
import { describe, expect, it } from 'vitest'
import { isSegmentConfirmed } from '../../src/config/segment'
import type { ChapterSegment } from '../../src/config/segment'
import {
  resolveSegmentRule,
  segmentRuleInputOf,
  SEGMENT_RULE_VALUES,
} from '../../src/panels/editorSegments'

/** Một hàng `segment` như nó đi trên dây, với những trường ca hiện tại quan tâm. */
function segment(over: Partial<ChapterSegment> = {}): ChapterSegment {
  return {
    id: 1,
    ord: 1,
    source_text: '一。',
    target_text: 'Đã dịch rồi.',
    is_paragraph_end: false,
    retired_at: null,
    status: 'draft',
    is_omitted: false,
    ...over,
  }
}

describe('resolveSegmentRule — nhánh `confirmed` nay có nguồn dữ liệu thật', () => {
  it('câu đã xác nhận, con trỏ ở chỗ khác ⇒ `confirmed`', () => {
    const rule = resolveSegmentRule(segmentRuleInputOf(segment({ status: 'confirmed' }), 999))
    expect(rule).toBe('confirmed')
  })

  /**
   * 🔴 **Quyết định #1 (Ice ký 2026-08-14) sống hay chết ở ca này.**
   *
   * `primary` **thắng** `confirmed` — `DESIGN.md:380` định nghĩa nó là *"đang sửa, con trỏ ở
   * đây"*, một mệnh đề về **hiện tại**; trạng thái đã xác nhận là mệnh đề về **quá khứ**, và
   * vạch chỉ có một chỗ để nói.
   *
   * ⇒ Hệ quả: người dùng bấm xác nhận trên chính câu con trỏ đang đứng thì vạch **vẫn**
   * `primary`. Đó **không** phải một lỗi — đó là lý do lượt xác nhận **dời con trỏ sang câu
   * kế tiếp** (đường (a) Ice ký), và ca đó nghiệm thu ở `editorConfirmSegment.test.ts`.
   *
   * ⚠️ Đảo hai dòng ưu tiên trong `resolveSegmentRule` làm ca này ĐỎ. Đó là chủ đích: thứ tự
   * đó là một quyết định 🔴 có lý do ghi tại chỗ, không một chi tiết cài đặt.
   */
  it('câu đã xác nhận MÀ con trỏ đang ở đó ⇒ `primary`, KHÔNG `confirmed`', () => {
    const s = segment({ id: 7, status: 'confirmed' })
    expect(resolveSegmentRule(segmentRuleInputOf(s, 7))).toBe('primary')
  })

  it('câu đã về hưu thắng tất cả, kể cả khi đã xác nhận', () => {
    const s = segment({ id: 7, status: 'confirmed', retired_at: '2026-08-14T00:00:00.000Z' })
    expect(resolveSegmentRule(segmentRuleInputOf(s, 7))).toBe('ornament')
  })
})

describe('hàng còn thiếu của bảng năm giá trị — nay ĐÃ CÓ giá trị riêng', () => {
  /**
   * 🔵 **ĐẢO MỆNH ĐỀ 2026-08-14 (Story 2.5b · AC4), và ghi cả HAI lượt ký theo THỨ TỰ.**
   *
   * | Lượt | Mệnh đề | Trạng thái |
   * |---|---|---|
   * | ① Quyết định #3 của Story 2.5 *(Ice ký 2026-08-14)* | *"đã dịch, chưa xác nhận ⇒ **không vạch**"* | **HẾT HIỆU LỰC** |
   * | ② UX-DR19 viết lại + Quyết định #2 của Story 2.5b *(cùng ngày, **sau** ①)* | *"⇒ vạch **`draft`**"* | **ĐANG HIỆU LỰC** |
   *
   * 🔴 ① **không sai lúc được ký** — tiền đề của nó *(«văn bản có chữ nằm NGAY CẠNH vạch, nên
   * mắt đọc được 'đã dịch' mà không cần một màu riêng»)* đúng cho **trang văn liền mạch** của
   * Story 2.2/2.3. Lưới đặt cột vạch ở **mép trái**, cách chỗ chữ cả một cột nguyên văn ⇒
   * *"nằm ngay cạnh"* hết đúng theo **hình học**, không theo lập luận.
   *
   * ⇒ Ca dưới đây là ca **thường nhật nhất** của tính năng *(gõ xong một câu rồi bấm sang câu
   * khác mà chưa xác nhận)*, không một ca biên.
   */
  it('đã dịch bằng tay · chưa xác nhận · con trỏ chỗ khác ⇒ vạch `draft`', () => {
    const s = segment({ id: 3, status: 'draft', target_text: 'Có chữ hẳn hoi.' })
    expect(resolveSegmentRule(segmentRuleInputOf(s, 999))).toBe('draft')
  })

  /**
   * 🔴 **VẾ THỨ HAI CỦA CÙNG MỘT MỆNH ĐỀ, và nó là chỗ dễ cài sai nhất.**
   *
   * `status = 'draft'` **một mình KHÔNG đủ** để ra `draft`: từ story này, `'draft'` đã **tách
   * khỏi** *"chưa dịch"*. Một cài đặt chỉ đọc `status` sẽ vẽ vạch `draft` cho **mọi** câu của
   * một Chương mới — tức cả Chương hiện ra như đã dịch xong, và không cổng nào đỏ vì chuyện đó.
   */
  it('chưa dịch *(`draft` VÀ `target_text` rỗng)* ⇒ KHÔNG vạch', () => {
    const s = segment({ id: 3, status: 'draft', target_text: '' })
    expect(resolveSegmentRule(segmentRuleInputOf(s, 999))).toBe('none')
  })

  /**
   * ⚠️ Thứ tự nhánh: `draft` đứng **sau** `tm-rule`. Cả hai nói *"có chữ, chưa ai ký"*, nhưng
   * `tm-rule` nói thêm **ai viết chữ đó** — và đó là thứ FR58 cần đọc được.
   */
  it('câu TM điền sẵn có chữ ⇒ `tm-rule`, KHÔNG `draft`', () => {
    const s = segment({ id: 4, status: 'draft', target_text: 'Máy điền.' })
    const input = { ...segmentRuleInputOf(s, 999), isTmFilled: true }
    expect(resolveSegmentRule(input)).toBe('tm-rule')
  })

  /** ⚠️ `primary` là mệnh đề về **hiện tại**, nó thắng `draft` như đã thắng `confirmed`. */
  it('con trỏ đang ở chính câu đó ⇒ `primary`, KHÔNG `draft`', () => {
    const s = segment({ id: 5, status: 'draft', target_text: 'Có chữ.' })
    expect(resolveSegmentRule(segmentRuleInputOf(s, 5))).toBe('primary')
  })
})

describe('hai chỗ đọc trạng thái phải ĐỒNG Ý với nhau', () => {
  /**
   * 🔴 Chuỗi `'confirmed'` tồn tại ở **hai** chỗ, và đó là một điều kiện kỹ thuật chứ không
   * một lượt lười: `editorSegments.ts` là **module thuần** *(Kiểm I `import()` nó bằng Node
   * trần)* nên nó **không được** `import` `isSegmentConfirmed` từ `config/segment.ts`, vốn kéo
   * theo `@tauri-apps/api`.
   *
   * ⇒ Lưới cho chỗ hở đó **không phải kỷ luật**: ca này. Sai chính tả ở một trong hai chỗ —
   * `'Confirmed'`, `'confimed'` — làm nó ĐỎ. Không có nó, một lượt sai chính tả cho `false`
   * **im lặng**: vạch không đổi màu, không lỗi nào được ném, và triệu chứng là *"xác nhận
   * không ăn"* mà không ai lần được về dòng nào.
   */
  it.each([['draft'], ['confirmed'], ['CONFIRMED'], ['']])(
    'status = %j ⇒ `isSegmentConfirmed` và `segmentRuleInputOf().isConfirmed` cho cùng một giá trị',
    (status) => {
      const s = segment({ status })
      expect(segmentRuleInputOf(s, null).isConfirmed).toBe(isSegmentConfirmed(s))
    },
  )
})

describe('bảng giá trị vạch KHÔNG mọc thêm ngoài lượt ký của Story 2.5b', () => {
  /**
   * ⚠️ Mệnh đề *"đúng sáu giá trị"* có chủ ở Kiểm I (cổng tĩnh). Ca này canh vế **khác**: mọi
   * giá trị mà hàm phân giải **thật sự trả về** đều nằm trong bảng đó — một nhánh trả về một
   * chuỗi ngoài bảng sẽ đi lọt Kiểm I, vì Kiểm I đếm **mảng khai báo**, không chạy hàm.
   */
  it('mọi giá trị hàm trả về đều nằm trong `SEGMENT_RULE_VALUES`', () => {
    const cases: ChapterSegment[] = [
      segment({ id: 1, status: 'confirmed' }),
      segment({ id: 2, status: 'draft' }),
      segment({ id: 3, status: 'draft', target_text: '' }),
      segment({ id: 4, retired_at: '2026-08-14T00:00:00.000Z' }),
      segment({ id: 5, status: 'mot-gia-tri-la' }),
    ]
    for (const s of cases) {
      for (const caret of [null, s.id]) {
        expect(SEGMENT_RULE_VALUES).toContain(resolveSegmentRule(segmentRuleInputOf(s, caret)))
      }
    }
  })

  /**
   * 🔴 Một giá trị `status` **lạ** *(một bản ứng dụng tương lai, một lượt sửa tay `project.db`)*
   * phải đọc thành *"chưa xác nhận"*, **không** thành một trạng thái thứ ba và **không** ném.
   * Cưỡng chế giá trị hợp lệ là việc của tầng Rust (AD-1); tầng này chỉ được phép **không nói
   * dối** về nó.
   */
  it('một `status` lạ đọc thành CHƯA xác nhận, không ném và không sinh giá trị mới', () => {
    const s = segment({ id: 9, status: 'mot-gia-tri-tu-tuong-lai' })
    expect(segmentRuleInputOf(s, null).isConfirmed).toBe(false)
    // 🔵 **ĐẢO 2026-08-14 (Story 2.5b).** Mệnh đề cũ đòi `'none'`; nó đúng khi *"chưa xác
    // nhận"* và *"chưa dịch"* còn chung một giá trị. Nay chúng tách: fixture mặc định **CÓ**
    // chữ (`target_text: 'Đã dịch rồi.'`), nên một `status` lạ rơi về **`draft`** — vẫn là
    // *"chưa ai ký"*, và vẫn **không** sinh một giá trị thứ bảy. Đó là điều ca này canh.
    expect(resolveSegmentRule(segmentRuleInputOf(s, null))).toBe('draft')
    // Vế thứ hai: cùng một `status` lạ mà **rỗng** thì rơi về `none`. Hai vế cùng nhau nói
    // rằng nhánh cuối phân xử bằng `target_text`, **không** bằng `status`.
    const empty = segment({ id: 9, status: 'mot-gia-tri-tu-tuong-lai', target_text: '' })
    expect(resolveSegmentRule(segmentRuleInputOf(empty, null))).toBe('none')
  })
})
