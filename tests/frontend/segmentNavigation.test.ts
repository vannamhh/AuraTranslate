/**
 * *"Câu chưa dịch kế tiếp"* — Story 2.5b · AC12 · `⌥↓`.
 *
 * ⚠️ **Vai của tệp này.** Nó kiểm một **vị từ trên dữ liệu**, và nó là đường nghiệm thu
 * **duy nhất** cho AC12: không cổng tĩnh nào đọc được ngữ nghĩa *"chưa dịch"*, và kho này
 * **không có một test mount component nào** cho lưới ngoài `editorTypingZone.test.ts`.
 *
 * 🔴 Mệnh đề trung tâm, và nó là chỗ dễ cài sai nhất: *"chưa dịch"* = `status === 'draft'`
 * **VÀ** `target_text === ''` — **hai vế**. Từ Story 2.5b, `'draft'` đã **tách khỏi** *"chưa
 * dịch"*: một câu đã gõ xong mà chưa ai bấm xác nhận vẫn mang `status = 'draft'`.
 */
import { describe, expect, it } from 'vitest'
import { isUntranslated, navigationSegmentOf, nextUntranslatedId } from '../../src/panels/segmentNavigation'
import type { NavigationSegment } from '../../src/panels/segmentNavigation'
import type { ChapterSegment } from '../../src/config/segment'

function seg(over: Partial<NavigationSegment> = {}): NavigationSegment {
  return { id: 1, status: 'draft', targetText: '', retiredAt: null, isOmitted: false, ...over }
}

describe('isUntranslated — HAI vế, không một', () => {
  it('`draft` + rỗng ⇒ chưa dịch', () => {
    expect(isUntranslated(seg({ status: 'draft', targetText: '' }))).toBe(true)
  })

  /**
   * 🔴 Ca **quyết định** của AC12. Một cài đặt chỉ đọc `status` cho `true` ở đây, và hệ quả
   * đo được là `⌥↓` đưa người dùng quay lại đúng những câu họ **vừa dịch** — tức phím này vô
   * dụng ngay ở Chương đầu tiên, và không cổng nào đỏ vì chuyện đó.
   */
  it('`draft` + CÓ chữ ⇒ **không** phải chưa dịch — đây là một BẢN NHÁP', () => {
    expect(isUntranslated(seg({ status: 'draft', targetText: 'Đã gõ rồi.' }))).toBe(false)
  })

  /**
   * ⚠️ Vế thứ nhất cũng không thừa: một câu **đã xác nhận** rồi bị xoá trắng bản dịch mang
   * `status = 'confirmed'` với `targetText === ''`. Nó **không** là *"chưa dịch"* — nó là một
   * câu đã ký đang hỏng, và đó là việc của một story khác.
   */
  it('`confirmed` + rỗng ⇒ **không** phải chưa dịch', () => {
    expect(isUntranslated(seg({ status: 'confirmed', targetText: '' }))).toBe(false)
  })
})

describe('nextUntranslatedId', () => {
  const list: readonly NavigationSegment[] = [
    seg({ id: 10, targetText: 'Đã dịch.' }),
    seg({ id: 11, targetText: '' }),
    seg({ id: 12, status: 'confirmed', targetText: 'Đã ký.' }),
    seg({ id: 13, targetText: '' }),
  ]

  it('từ đầu Chương (`null`) ⇒ câu chưa dịch ĐẦU TIÊN', () => {
    expect(nextUntranslatedId(list, null)).toBe(11)
  })

  it('đứng ở một câu ⇒ câu chưa dịch kế tiếp SAU nó, không phải chính nó', () => {
    expect(nextUntranslatedId(list, 11)).toBe(13)
  })

  it('nhảy QUA câu đã dịch và câu đã ký', () => {
    expect(nextUntranslatedId(list, 10)).toBe(11)
    expect(nextUntranslatedId(list, 12)).toBe(13)
  })

  /**
   * 🔴 **KHÔNG quay vòng về đầu.** Một lượt quay vòng im lặng đưa người dùng về đầu Chương mà
   * không dấu hiệu nào — họ đọc thành *"phím này nhảy lung tung"*. `null` là câu trả lời
   * trung thực: **không còn câu nào chưa dịch ở phía dưới**.
   */
  it('hết Chương ⇒ `null`, KHÔNG quay vòng về đầu', () => {
    expect(nextUntranslatedId(list, 13)).toBeNull()
  })

  it('không còn câu nào chưa dịch ⇒ `null` từ mọi vị trí', () => {
    const done = [seg({ id: 1, targetText: 'A' }), seg({ id: 2, status: 'confirmed', targetText: 'B' })]
    expect(nextUntranslatedId(done, null)).toBeNull()
    expect(nextUntranslatedId(done, 1)).toBeNull()
  })

  /**
   * ⚠️ Segment **đã về hưu** bị bỏ qua ở cả hai vai — không phải đích, và không chặn đường.
   * Nguồn dữ liệu cho nhánh này tới ở **Story 2.8**; ca ở đây khoá ngữ nghĩa lại trước.
   */
  it('câu đã VỀ HƯU không bao giờ là đích, kể cả khi rỗng', () => {
    const withRetired = [
      seg({ id: 1, targetText: 'A' }),
      seg({ id: 2, targetText: '', retiredAt: '2026-08-14T00:00:00.000Z' }),
      seg({ id: 3, targetText: '' }),
    ]
    expect(nextUntranslatedId(withRetired, 1)).toBe(3)
  })

  /**
   * 🔴 **AC6 của Story 2.5c (FR133) — câu đã CẮT BỎ không bao giờ là đích.**
   *
   * Cùng khuôn và cùng vai với ca *"đã về hưu"* ngay trên: bỏ qua ở **cả hai** vai — không
   * phải đích, và không chặn đường. Một câu người dùng đã quyết định **không thuộc bản dịch**
   * mà `⌥↓` vẫn dừng lại ở đó là phím này dẫn họ tới đúng chỗ họ vừa bảo là bỏ đi.
   *
   * ⚠️ Ca này dùng một câu **rỗng** *(tức "chưa dịch" theo cả hai vế)* chứ không một câu đã
   * có chữ — nếu không nó xanh vì `isUntranslated` chứ không vì cờ cắt bỏ, và phép kiểm sẽ
   * không đo gì cả.
   */
  it('câu đã CẮT BỎ không bao giờ là đích, kể cả khi rỗng', () => {
    const withOmitted = [
      seg({ id: 1, targetText: 'A' }),
      seg({ id: 2, targetText: '', isOmitted: true }),
      seg({ id: 3, targetText: '' }),
    ]
    expect(nextUntranslatedId(withOmitted, 1)).toBe(3)
  })

  /**
   * 🔴 **AC2 nói bằng ngôn ngữ của điều hướng:** cắt bỏ là một **trục độc lập**, nên nó
   * KHÔNG được đổi nghĩa của `isUntranslated`. Một câu đã cắt bỏ mà chưa dịch **vẫn là**
   * *"chưa dịch"* — nó chỉ không phải **đích của phím này**.
   *
   * Đây là tiền lệ đã chốt cho "về hưu", nhắc lại nguyên văn cho cờ mới: hai mệnh đề khác
   * nhau, hai chỗ lọc khác nhau. Nhét cờ vào `isUntranslated` là gộp chúng, và lúc đó mọi
   * chỗ đọc *"chưa dịch"* trong tương lai *(thanh tiến độ, Story 5.5)* sẽ đếm sai.
   */
  it('cắt bỏ KHÔNG đổi nghĩa `isUntranslated`', () => {
    expect(isUntranslated(seg({ status: 'draft', targetText: '', isOmitted: true }))).toBe(true)
  })

  /**
   * ⚠️ `fromId` trỏ vào một câu **không còn trong danh sách** *(vừa bị gộp mất — Story 2.8)*
   * đọc thành *"bắt đầu từ đầu"*. Gộp hai ca có chủ ý: một hành vi có nghĩa vẫn tốt hơn không
   * hành vi nào.
   */
  it('`fromId` không tìm thấy ⇒ đọc thành "từ đầu Chương"', () => {
    expect(nextUntranslatedId(list, 9999)).toBe(11)
  })

  it('danh sách rỗng ⇒ `null`, không ném', () => {
    expect(nextUntranslatedId([], null)).toBeNull()
  })
})

describe('navigationSegmentOf — văn bản ĐANG GÕ thắng bản lúc nạp', () => {
  function row(over: Partial<ChapterSegment> = {}): ChapterSegment {
    return {
      id: 1,
      ord: 1,
      source_text: '一。',
      target_text: '',
      is_paragraph_end: false,
      retired_at: null,
      status: 'draft',
      is_omitted: false,
      // 🔵 2026-08-16 (Story 2.5d) — cột `is_target_paragraph_end`, bước di trú 9.
      // Mặc định BẰNG `is_paragraph_end` ngay trên, đúng AC2 lúc nhập.
      is_target_paragraph_end: false,
      ...over,
    }
  }

  /**
   * 🔴 Ca này đóng đường hỏng **thường nhật nhất** của `⌥↓`: người dùng vừa gõ xong một câu,
   * bấm `⌥↓`, và tập chờ **chưa flush**. `editorSegments` giữ bản **lúc nạp** *(mốc của FR117,
   * Story 2.7)* nên nó vẫn là chuỗi rỗng — một phép chọn đọc thẳng hàng dữ liệu sẽ nhảy **vào
   * chính câu vừa gõ**.
   */
  it('câu vừa gõ (chưa flush) KHÔNG còn là "chưa dịch"', () => {
    const edited = new Map([[1, 'Vừa gõ xong.']])
    expect(isUntranslated(navigationSegmentOf(row(), edited))).toBe(false)
  })

  it('không có trong tập chờ ⇒ đọc bản lúc nạp', () => {
    expect(isUntranslated(navigationSegmentOf(row(), new Map()))).toBe(true)
    expect(isUntranslated(navigationSegmentOf(row({ target_text: 'X' }), new Map()))).toBe(false)
  })

  /**
   * ⚠️ Một lượt **xoá trắng** cũng phải đi qua tập chờ: người dùng xoá hết chữ của một câu rồi
   * bấm `⌥↓` thì câu đó **trở lại** *"chưa dịch"*, dù trên đĩa vẫn còn chữ.
   */
  it('xoá trắng trong phiên ⇒ trở lại "chưa dịch"', () => {
    const edited = new Map([[1, '']])
    expect(isUntranslated(navigationSegmentOf(row({ target_text: 'Có chữ trên đĩa.' }), edited))).toBe(true)
  })

  /**
   * 🔴 Cờ cắt bỏ phải **đi qua** phép ánh xạ. Quên nó ở đây là một khuyết tật im lặng đúng
   * hạng: `nextUntranslatedId` lọc trên `isOmitted`, nên một `undefined` rơi vào đó làm phép
   * lọc **không bao giờ bắt** — và AC6 hỏng trong khi mọi ca gọi thẳng `seg()` vẫn xanh.
   */
  it('`is_omitted` đi qua phép ánh xạ, không rơi mất', () => {
    const edited = new Map<number, string>()
    expect(navigationSegmentOf(row({ is_omitted: true }), edited).isOmitted).toBe(true)
    expect(navigationSegmentOf(row({ is_omitted: false }), edited).isOmitted).toBe(false)
  })
})

/**
 * 🔴 **BA chỗ đọc chuỗi `'draft'`, và chúng phải ĐỒNG Ý** — cùng khuôn ca *"hai chỗ đọc trạng
 * thái"* của `editorSegmentRule.test.ts`.
 *
 * `segmentNavigation.ts` và `editorSegments.ts` đều là **module thuần** *(cổng `import()`
 * chúng bằng Node trần)* nên không tệp nào được `import` hằng từ `config/segment.ts`, vốn kéo
 * theo `@tauri-apps/api`. Cái giá là chuỗi viết thẳng ở ba chỗ; lưới cho chỗ hở đó là ca này.
 * Sai chính tả ở một trong ba — `'Draft'`, `'darft'` — làm nó ĐỎ.
 */
describe('ba chỗ đọc `draft` phải ĐỒNG Ý với nhau', () => {
  it('cùng một `status` cho cùng một phán quyết ở cả hai module thuần', async () => {
    const { resolveSegmentRule } = await import('../../src/panels/editorSegments')

    // Rỗng + `draft` ⇒ *"chưa dịch"* ở cả hai đường: không vạch, và là đích của `⌥↓`.
    expect(isUntranslated(seg({ status: 'draft', targetText: '' }))).toBe(true)
    expect(
      resolveSegmentRule({
        retiredAt: null,
        hasCaret: false,
        isConfirmed: false,
        isTmFilled: false,
        targetText: '',
      }),
    ).toBe('none')

    // Có chữ + `draft` ⇒ *"bản nháp"* ở cả hai đường: vạch `draft`, và KHÔNG là đích.
    expect(isUntranslated(seg({ status: 'draft', targetText: 'X' }))).toBe(false)
    expect(
      resolveSegmentRule({
        retiredAt: null,
        hasCaret: false,
        isConfirmed: false,
        isTmFilled: false,
        targetText: 'X',
      }),
    ).toBe('draft')
  })
})
