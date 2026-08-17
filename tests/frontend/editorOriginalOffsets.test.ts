/**
 * Story 2.9 · AC9 — **phép ánh xạ *chỉ số sau chuẩn hoá → chỉ số GỐC*** cho chỗ cắt.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔵 TỆP NÀY SINH RA Ở LƯỢT CODE REVIEW 2026-08-17, VÀ NÓ ĐÓNG MỘT KHOẢNG **NGHIỆM THU**
 * ═════════════════════════════════════════════════════════════════════════════════
 * Tầng Acceptance Auditor đo được: `editorSourceCut.test.ts` chỉ kiểm `sourceCutOffsetOf()` trên
 * DOM **dựng tay** đã có sẵn `data-src-start`, nên **không ca nào** đi qua phép ánh xạ sinh ra
 * chính con số đó. Đường verify duy nhất là một **bàn đo tay**
 * (`2-9-ban-do/han-viet-cho-cat.e2e.mjs`) — không nằm trong `e2e/specs/`, không chạy trong
 * `npm run test`.
 *
 * 🔴 Vì sao khoảng hở đó đáng một tệp riêng: hàm này quyết định chỗ cắt **rơi vào đâu trong
 * `source_text` như Rust thấy nó**. Một lượt hồi quy ở đây không ném lỗi, không làm đỏ cổng nào,
 * và biểu hiện là `⌘/` **cắt sai chỗ, im lặng**, trên dữ liệu mà **AD-5 không cho hoàn tác**.
 *
 * ⚠️ Ba ca dưới đây là ba **nguyên nhân khác nhau**, không ba biến tấu của một ca:
 *   ① `\r\n` — hai điểm mã gốc thành một sau chuẩn hoá ⇒ mọi chỉ số **sau** nó dời một bậc.
 *   ② `\r` trần — một thành một ⇒ **không** dời. Đây là ca dễ vá quá tay nhất: một phép nhảy vô
 *      điều kiện ở `\r` sẽ đúng ca ① và sai ca ②.
 *   ③ ngoài BMP — bảng phải đếm theo **ĐIỂM MÃ**, cùng đơn vị `chars()` của Rust, không theo đơn
 *      vị mã UTF-16 của `String.length`.
 *
 * 🔴 `\r` **có thể** có mặt trong dữ liệu thật: FR125 *(chuẩn hoá xuống dòng)* thuộc Epic 6 và còn
 * `backlog`, còn `core/segment/import.rs` thì không chuẩn hoá — nên ca ① và ② không phải giả định.
 */
import { describe, expect, it } from 'vitest'
import { originalOffsets } from '../../src/panels/editorSegments'

/**
 * Chuẩn hoá **y hệt** `SourceHanViet.vue` làm trước lượt tách từ.
 *
 * ⚠️ Chép biểu thức chứ không import: `NEWLINES` sống trong `<script setup>` của component. Lượt
 * chép này là chỗ hở duy nhất còn lại của tệp — nếu component đổi phép chuẩn hoá, các ca dưới
 * **vẫn xanh** trong khi sản phẩm đã lệch. Ghi ra để lượt sửa sau biết mình đang đứng ở đâu.
 */
const chuanHoa = (s: string): string => s.replace(/\r\n?/g, '\n')

describe('originalOffsets — ánh xạ chỉ số sau chuẩn hoá về chỉ số gốc', () => {
  it('① văn bản không có ký tự xuống dòng ⇒ ánh xạ đồng nhất', () => {
    const goc = '京都春風。'
    expect(originalOffsets(goc)).toEqual([0, 1, 2, 3, 4])
  })

  it('② `\\r\\n` làm mọi chỉ số SAU nó dời đúng một bậc', () => {
    // Gốc:        京(0) \r(1) \n(2) 都(3)   — 4 điểm mã
    // Chuẩn hoá:  京(0) \n(1)       都(2)   — 3 điểm mã
    const goc = '京\r\n都'
    const bang = originalOffsets(goc)

    expect([...chuanHoa(goc)]).toHaveLength(3)
    expect(bang).toEqual([0, 1, 3])
    // 🔴 Mệnh đề thật của cả tệp: chỉ số 2 sau chuẩn hoá *(chữ 都)* phải trỏ **3** ở bản gốc.
    // Bỏ phép ánh xạ này thì nó trỏ 2 — tức `\n`, và Rust cắt trước chữ 都 một ký tự.
    expect(bang[2]).toBe(3)
  })

  it('③ `\\r` TRẦN thì KHÔNG dời — một điểm mã thành một điểm mã', () => {
    // Gốc và bản chuẩn hoá dài bằng nhau; một phép nhảy vô điều kiện ở `\r` sẽ sai đúng ở đây.
    const goc = '京\r都'
    expect([...chuanHoa(goc)]).toHaveLength(3)
    expect(originalOffsets(goc)).toEqual([0, 1, 2])
  })

  it('④ hai `\\r\\n` cộng dồn — bậc dời TÍCH LUỸ, không phải một lần', () => {
    // Gốc: 一 \r \n 二 \r \n 三  ⇒ chỉ số gốc 0 · 1 · 3 · 4 · 6
    const bang = originalOffsets('一\r\n二\r\n三')
    expect(bang).toEqual([0, 1, 3, 4, 6])
    expect(bang.at(-1)).toBe(6)
  })

  it('⑤ ký tự ngoài BMP chiếm MỘT ô của bảng, không hai', () => {
    // 𠀀 = U+20000, CJK Extension B — **hai** đơn vị mã UTF-16, **một** điểm mã.
    const goc = '𠀀都'
    expect(goc.length).toBe(3) // đơn vị mã: 2 + 1
    expect([...goc]).toHaveLength(2) // điểm mã: 1 + 1

    // 🔴 Bảng đếm theo ĐIỂM MÃ — cùng đơn vị `regroup.rs::split_at` dùng qua `chars()`. Một bảng
    // theo đơn vị mã sẽ dài 3 và mọi chỉ số sau ký tự astral lệch đúng số ký tự astral đứng trước.
    expect(originalOffsets(goc)).toEqual([0, 1])
  })

  it('⑥ ngoài BMP đứng SAU một `\\r\\n` — hai phép đếm cùng lúc, không loại trừ nhau', () => {
    // Gốc theo điểm mã: 一(0) \r(1) \n(2) 𠀀(3) 都(4)
    // Chuẩn hoá:        一(0) \n(1)       𠀀(2) 都(3)
    const bang = originalOffsets('一\r\n𠀀都')
    expect(bang).toEqual([0, 1, 3, 4])
    expect(bang[2]).toBe(3)
  })

  it('⑦ chuỗi rỗng ⇒ bảng rỗng, không một ô rác', () => {
    expect(originalOffsets('')).toEqual([])
  })

  it('⑧ bảng luôn dài đúng bằng số ĐIỂM MÃ của bản đã chuẩn hoá', () => {
    // Bất biến của cả hàm, phát biểu một lần cho mọi hình dạng: `buildSegments` tra bảng bằng một
    // chỉ số điểm mã của bản chuẩn hoá, nên một bảng ngắn hơn là một `?? cp` âm thầm trả số sai.
    for (const goc of ['京都春風。', '京\r\n都', '京\r都', '一\r\n二\r\n三', '𠀀都', '一\r\n𠀀都', '']) {
      expect(originalOffsets(goc)).toHaveLength([...chuanHoa(goc)].length)
    }
  })
})
