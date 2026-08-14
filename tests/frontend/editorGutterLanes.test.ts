/**
 * Chia làn cho vạch lề — Story 2.5, Quyết định #2(a) do Ice ký 2026-08-14.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ VAI CỦA TỆP NÀY, VÀ THỨ NÓ **KHÔNG** ĐƯỢC PHÉP KHẲNG ĐỊNH
 * ─────────────────────────────────────────────────────────────────────────────
 * `assignGutterLanes` là một **hàm thuần** nhận số và trả số — nó không chạm DOM. Đó là thứ
 * vitest kiểm được, và kiểm được **tất định**.
 *
 * 🔴 Mệnh đề *"hai câu ngắn có thật sự nằm cùng một dòng không"* và *"hai vạch cách 3px có
 * đọc được bằng mắt không"* thuộc **BÀN ĐO** (`2-5-ban-do-hai-vach.html`, hai engine × hai
 * theme), **không** thuộc đây: `happy-dom` không phải WebKit, và mọi mệnh đề về hình học hay
 * engine thật đi đường khác (§Testing Rules, AC25).
 *
 * ⇒ Ở đây: *"cho những khoảng dọc NÀY, phép phát làn cho ra những `left` NÀO"*.
 */
import { describe, expect, it } from 'vitest'
import { assignGutterLanes } from '../../src/panels/editorGutter'

/** Một dòng Editor đo được trên cả hai engine: 17,00px (Blink) · 17,04px (WebKit). */
const LINE = 17

/** `n` vạch cùng một dòng — ca mà Quyết định #2 tồn tại để phân xử. */
function sameLine(n: number, top = 0): { id: number; top: number; height: number }[] {
  return Array.from({ length: n }, (_, i) => ({ id: i + 1, top, height: LINE }))
}

describe('vạch KHÔNG chồng nhau ⇒ tất cả ở làn trong cùng', () => {
  it('ba câu trên ba dòng khác nhau đều ở `left: 8px`', () => {
    const out = assignGutterLanes([
      { id: 1, top: 0, height: LINE },
      { id: 2, top: LINE, height: LINE },
      { id: 3, top: LINE * 2, height: LINE },
    ])
    expect(out.map((r) => r.left)).toEqual([8, 8, 8])
  })

  /**
   * 🔴 Ca **hồi quy** cho toàn bộ Epic 2 tới trước story này: khi chỉ một vạch tồn tại, `left`
   * phải bằng **đúng** con số mà CSS dùng trước lượt này (`.gmark { left: 8px }`). Một lượt
   * đổi hằng làm mọi ảnh chụp của bàn đo 2.2/2.3 hết so sánh được.
   */
  it('một vạch duy nhất giữ nguyên `left: 8px` của trước Story 2.5', () => {
    expect(assignGutterLanes([{ id: 1, top: 0, height: LINE }])[0]?.left).toBe(8)
  })

  it('bộ rỗng không ném và trả bộ rỗng', () => {
    expect(assignGutterLanes([])).toEqual([])
  })
})

describe('vạch chồng nhau ⇒ mỗi vạch một làn, KHÔNG cái nào bị che', () => {
  it.each([
    [2, [8, 13]],
    [3, [8, 13, 18]],
    [4, [8, 12, 16, 20]],
    [5, [8, 11, 14, 17, 20]],
  ])('%i câu cùng một dòng ⇒ %j', (n, expected) => {
    const out = assignGutterLanes(sameLine(n))
    expect(out.map((r) => r.left)).toEqual(expected)
  })

  /**
   * 🔴 **Đây là mệnh đề mà phép đo ngày 2026-08-14 sinh ra.** Bước làn **cố định 5px** —
   * hình dạng mà §Quyết định #2 của story mô tả — cho làn ngoài cùng ở `8 + 4·5 = 28px`, mép
   * phải **30px**, tức **tràn khỏi máng 22px** và rơi đúng chỗ chữ bắt đầu.
   *
   * ⇒ Bước phải **co**. Ca này ĐỎ nếu ai đó đóng băng lại bước 5px.
   */
  it('không vạch nào có mép phải vượt mép máng 22px, ở mọi số làn', () => {
    for (let n = 1; n <= 7; n += 1) {
      for (const r of assignGutterLanes(sameLine(n))) {
        expect(r.left + 2).toBeLessThanOrEqual(22)
      }
    }
  })

  it('mọi `left` PHÂN BIỆT được — không hai vạch nào chồng chỗ, tới 7 làn', () => {
    for (let n = 1; n <= 7; n += 1) {
      const lefts = assignGutterLanes(sameLine(n)).map((r) => r.left)
      expect(new Set(lefts).size).toBe(n)
    }
  })
})

describe('phép phát làn là TÔ MÀU ĐỒ THỊ KHOẢNG, không gom bắc cầu', () => {
  /**
   * 🔴 Ca này bác đúng bản cài đặt đầu tiên, thứ mà **lượt chạy đầu trên bàn đo đã bác**:
   * gom bắc cầu *(A chồng B, B chồng C ⇒ một nhóm)* rồi phát làn theo thứ tự trong nhóm.
   *
   * Hình: một câu **dài** trải ba dòng, và hai câu ngắn — một ở dòng đầu, một ở dòng cuối.
   * Hai câu ngắn **không** chồng nhau, nên chúng được phép **dùng chung** làn 1. Phép gom bắc
   * cầu sẽ đẩy câu ngắn thứ hai ra làn 2 và tiêu một làn không cần tiêu.
   */
  it('hai vạch không chồng nhau được DÙNG CHUNG một làn, dù cùng chồng với một vạch thứ ba', () => {
    const out = assignGutterLanes([
      { id: 1, top: 0, height: LINE * 3 }, // câu dài, ba dòng
      { id: 2, top: 0, height: LINE }, // câu ngắn ở dòng đầu
      { id: 3, top: LINE * 2, height: LINE }, // câu ngắn ở dòng cuối
    ])
    const left = new Map(out.map((r) => [r.id, r.left]))
    expect(left.get(1)).toBe(8)
    expect(left.get(2)).toBe(13)
    expect(left.get(3)).toBe(13)
  })

  /**
   * ⚠️ Hai engine làm tròn `getClientRects()` khác nhau *(17,00 vs 17,04 px)*. Phép so chồng
   * nhau dùng biên nửa pixel chính vì thế: hai câu **liền kề** *(câu này kết thúc đúng chỗ câu
   * kia bắt đầu)* KHÔNG được tính là chồng, nếu không mọi vạch của cả Chương dồn thành một
   * chuỗi bắc cầu.
   */
  it('hai vạch LIỀN KỀ theo chiều dọc không tính là chồng nhau', () => {
    const out = assignGutterLanes([
      { id: 1, top: 0, height: LINE },
      { id: 2, top: LINE, height: LINE },
      { id: 3, top: LINE * 2 + 0.04, height: LINE },
    ])
    expect(out.map((r) => r.left)).toEqual([8, 8, 8])
  })
})

describe('GIỚI HẠN THẬT — từ 8 làn trở lên, máng 22px hết chỗ', () => {
  /**
   * ⚠️ Ghi ra thay vì để người sau tự phát hiện. Bước tối thiểu là 2px *(bằng bề rộng vạch)*,
   * nên máng 22px chứa nhiều nhất **7** làn. Từ làn thứ tám, luật là **dồn về làn cuối** —
   * tức chấp nhận che, có chủ ý, thay vì tràn ra đè lên chữ.
   *
   * Nó đòi **8 câu cùng một dòng**; fixture đối thoại dày nhất của bàn đo mới cho **5**.
   */
  it('8 câu cùng một dòng ⇒ hai vạch cuối dồn chung làn cuối, và KHÔNG cái nào tràn', () => {
    const out = assignGutterLanes(sameLine(8))
    const lefts = out.map((r) => r.left)

    expect(Math.max(...lefts) + 2).toBeLessThanOrEqual(22)
    // Bảy làn phân biệt, cái thứ tám dồn vào làn cuối.
    expect(new Set(lefts).size).toBe(7)
    expect(lefts.at(-1)).toBe(lefts.at(-2))
  })
})
