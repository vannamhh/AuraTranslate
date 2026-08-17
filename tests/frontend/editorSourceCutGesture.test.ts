/**
 * Story 2.9 · AC7 — **cử chỉ đánh dấu chỗ cắt là `Mod`+click, không phải một cú bấm đơn**.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO CỬ CHỈ ĐỔI — một xung đột Ice tìm ra bằng cách DÙNG THẬT
 * ═════════════════════════════════════════════════════════════════════════════════
 * Cột nguyên văn mang **hai** cử chỉ chuột cùng lúc, và chúng giẫm lên nhau:
 *
 * | Cử chỉ | Việc | Từ story |
 * |---|---|---|
 * | bấm đơn → `@mouseup` → `setEditorSourceCut` | đánh dấu chỗ cắt | **2.8** |
 * | double-click → vùng chọn → `useSelectionSurface(colSrc, 'source')` | **tra từ điển** (FR21) | **1.18, ĐÃ PHÁT HÀNH** |
 *
 * ⇒ Mỗi lượt tra một từ để **đọc** cũng rơi một dấu cắt vào câu. Tệ hơn: một cú double-click
 * bắn **HAI** `mouseup`, tức **hai** lượt `setEditorSourceCut` — và hàm đó **toggle**, nên hai
 * lượt ở hai offset khác nhau để lại **hai** dấu, còn ở cùng offset thì thêm rồi gỡ.
 * Người dùng không có cách nào biết mình vừa đặt chỗ cắt cho một lượt `⌘/` **họ không định
 * gọi** — và AD-5 không cho hoàn tác lượt tách đó.
 *
 * ✅ **Ice ký 2026-08-17:** `Mod`+click đánh dấu; **bấm đơn KHÔNG đánh dấu gì**, để trống cho
 * tra cứu. Cùng lượt ký đó đóng món nợ 🔴 `deferred-work.md:4100` *(Auto-Lookup bằng chuột
 * "có thể đang chết")*: Ice xác nhận **double-click TRA ĐƯỢC** trên máy thật ⇒ vế đó là giới
 * hạn của **BỘ ĐO**, không của sản phẩm.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 `Mod`, KHÔNG `event.metaKey` — cùng §Trap 1 mà `keys.ts` đã trả giá để biết
 * ═════════════════════════════════════════════════════════════════════════════════
 * `src/commands/README.md:73` viết bằng chữ: *"Đừng viết `event.metaKey`. `⌘` là ký hiệu
 * macOS của một phím **trừu tượng**; trên Windows nó là `Ctrl`. Một cài đặt chỉ đọc `metaKey`
 * đi qua **cả hai nền tảng của CI** rồi hỏng ở tay người dùng Windows."*
 *
 * Nửa Windows của kho **không có đường nghiệm thu tại chỗ** *(`pre-push` chạy trên macOS của
 * Ice — action item A5 của retro Epic 1)*, nên tệp này là lưới **duy nhất** lái cả hai ca.
 *
 * ⚠️ Và một cái bẫy riêng của **chuột**, không có ở bàn phím: trên macOS `Ctrl`+click là cú
 * bấm phụ (menu ngữ cảnh) của hệ điều hành. Một cài đặt đọc `ctrlKey` trên macOS vì thế vừa
 * sai phím **vừa** cướp một cử chỉ của OS.
 */
import { describe, expect, it } from 'vitest'

import { hasPrimaryModifier } from '../../src/panels/editorSegments'

const MAC = { isMac: true }
const WIN = { isMac: false }

/** Bốn cờ bổ trợ của một cú bấm, ở dạng hàm này đọc. */
function bam(mods: Partial<{ metaKey: boolean; ctrlKey: boolean }>) {
  return { metaKey: false, ctrlKey: false, ...mods }
}

describe('Story 2.9 · AC7 — vị từ `Mod` cho cú bấm chuột', () => {
  it('① macOS: `⌘`+click ⇒ CÓ', () => {
    expect(hasPrimaryModifier(bam({ metaKey: true }), MAC)).toBe(true)
  })

  it('🔴 ② macOS: `Ctrl`+click ⇒ KHÔNG — đó là cú bấm phụ của hệ điều hành', () => {
    // Nhận nó ở đây nghĩa là cướp menu ngữ cảnh của macOS, và người dùng sẽ đặt một chỗ cắt
    // mỗi lần họ định mở menu.
    expect(hasPrimaryModifier(bam({ ctrlKey: true }), MAC)).toBe(false)
  })

  it('🔴 ③ Windows/Linux: `Ctrl`+click ⇒ CÓ', () => {
    // Đây là ca mà `event.metaKey` trần **đi qua cả hai nền tảng của CI rồi hỏng ở tay người
    // dùng**. Nửa Windows không có đường nghiệm thu tại chỗ ⇒ ca này là lưới duy nhất.
    expect(hasPrimaryModifier(bam({ ctrlKey: true }), WIN)).toBe(true)
  })

  it('④ Windows/Linux: `Meta`+click *(phím Windows)* ⇒ KHÔNG', () => {
    expect(hasPrimaryModifier(bam({ metaKey: true }), WIN)).toBe(false)
  })

  it('⑤ bấm TRƠN ⇒ KHÔNG, trên cả hai nền tảng — đây là vế Ice ký', () => {
    // Vế này là toàn bộ điểm của AC7: một cú bấm trơn để trống cho tra cứu.
    expect(hasPrimaryModifier(bam({}), MAC)).toBe(false)
    expect(hasPrimaryModifier(bam({}), WIN)).toBe(false)
  })

  it('⑥ `Mod` giữ nguyên hiệu lực khi có thêm `Shift`/`Alt` đi kèm', () => {
    // Hàm này chỉ trả lời *"phím bổ trợ CHÍNH có được giữ không"*. Nó không phải một phép so
    // hợp âm đầy đủ — `sameMods` của `keys.ts` là chỗ đó, và bắt chước nó ở đây sẽ là một
    // bảng hợp âm thứ hai cho một cử chỉ chuột không có bảng nào.
    expect(hasPrimaryModifier({ metaKey: true, ctrlKey: true }, MAC)).toBe(true)
    expect(hasPrimaryModifier({ metaKey: true, ctrlKey: true }, WIN)).toBe(true)
  })
})
