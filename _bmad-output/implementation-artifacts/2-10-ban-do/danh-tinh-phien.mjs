/**
 * Bàn đo Story 2.10 · **Task 1.1** — TỰ KIỂM DANH TÍNH PHIÊN, chạy TRƯỚC mọi con số.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 KHUYẾT TẬT CỦA BÀN ĐO 2.9 MÀ TỆP NÀY TỒN TẠI ĐỂ ĐÓNG
 * ═════════════════════════════════════════════════════════════════════════════════
 * Bàn đo 2.9 mở Tác phẩm rồi `await $('[data-col="tgt"]').waitForExist(...)`. Phép đợi đó
 * **không phân biệt được** hai trạng thái hoàn toàn khác nhau:
 *
 *   ① Chương MỚI đã nạp xong  ⇒ số đo nói về fixture mình vừa dựng.
 *   ② Chương CŨ của lượt chạy trước còn nằm đó ⇒ số đo nói về một Chương khác, và nó
 *      **trông y hệt một số đo hợp lệ**.
 *
 * Ca ② có thật trong kho này: `openWorkspaceWithWork` tạo Tác phẩm qua IPC rồi bấm `⌘2`, và
 * lưới nạp segment ở **một lượt IPC nữa**. Giữa hai lượt đó `[data-col="tgt"]` của Chương
 * trước **vẫn tồn tại**. ⇒ Một lượt đo "thành công" trên dữ liệu sai, không dấu hiệu nào —
 * đúng lớp *"rỗng IM LẶNG"*, chỉ khác là nó rỗng ở **danh tính** thay vì ở số hàng.
 *
 * 🔴 **PHÉP KIỂM PHẢI HỎI DANH TÍNH, KHÔNG HỎI SỰ TỒN TẠI.** Ba vế, và cả ba đều cần:
 *   Ⓐ **số hàng** đúng bằng số câu fixture đưa vào — bắt ca "Chương khác, cũng có hàng";
 *   Ⓑ **nội dung câu đầu** khớp fixture — bắt ca "cùng số hàng, khác Chương" *(thật khi hai
 *     lượt chạy liên tiếp dùng cùng một fixture độ dài)*;
 *   Ⓒ `data-chapter-id` — ghi ra để đọc lại được, và nó là thứ **đổi** giữa hai lượt chạy.
 *
 * ⚠️ Vế Ⓑ là vế 2.9 không có, và nó là vế duy nhất bắt được ca ②. Ⓐ một mình **không đủ**:
 * hai lượt chạy của cùng một bàn đo dùng **cùng** fixture, nên số hàng khớp là chuyện đương
 * nhiên chứ không phải một bằng chứng.
 */

/**
 * Đợi lưới nạp xong **đúng Chương mình vừa dựng**, rồi trả ảnh chụp danh tính.
 *
 * 🔴 Ném khi không khớp, **không** trả một cờ để nơi gọi tự nhớ kiểm — một giá trị trả về bị
 * bỏ quên là đúng cách phép kiểm này sẽ chết trong im lặng lần thứ hai. Cùng lý lẽ mà luật
 * *"lỗi hạ tầng KHÔNG phải một phép kiểm đỏ"* của `§Luật của một CỔNG` đã ghi: hỏng thước thì
 * **dừng**, đừng in một con số.
 *
 * @param {number} soHangMongDoi số câu fixture đưa vào
 * @param {string} cauDauMongDoi nguyên văn câu nguồn đầu tiên
 */
export async function doiChuongVaKiemDanhTinh(soHangMongDoi, cauDauMongDoi) {
  await browser.waitUntil(
    async () =>
      await browser.execute(
        (n) => document.querySelectorAll('[data-col="tgt"]').length === n,
        soHangMongDoi,
      ),
    {
      timeout: 60_000,
      timeoutMsg:
        `Sau 60 giây lưới vẫn không có đúng ${soHangMongDoi} hàng.\n` +
        'Đây là lỗi HẠ TẦNG của bàn đo (Chương chưa nạp, hoặc fixture tách ra số câu khác\n' +
        'số mong đợi), KHÔNG một khuyết tật sản phẩm. Đừng đọc nó thành một kết quả.',
    },
  )

  const anh = await browser.execute(() => {
    const hop = document.querySelector('.grid-scroll')
    const src = document.querySelectorAll('[data-col="src"]')
    return {
      href: window.location.href,
      coApp: document.querySelector('#app') !== null,
      userAgent: navigator.userAgent,
      soHang: document.querySelectorAll('[data-col="tgt"]').length,
      chapterId: hop === null ? null : hop.getAttribute('data-chapter-id'),
      cauDau: src.length > 0 ? src[0].textContent : null,
      cauCuoi: src.length > 0 ? src[src.length - 1].textContent : null,
    }
  })

  // Ⓑ — vế mà bàn đo 2.9 không có.
  if (anh.cauDau !== cauDauMongDoi) {
    throw new Error(
      'DANH TÍNH PHIÊN KHÔNG KHỚP — lưới đang hiện một Chương KHÁC Chương vừa dựng.\n\n' +
        `  câu đầu mong đợi: ${JSON.stringify(cauDauMongDoi)}\n` +
        `  câu đầu đang có : ${JSON.stringify(anh.cauDau)}\n` +
        `  chapterId       : ${anh.chapterId}\n\n` +
        'Đây là lỗi HẠ TẦNG. Mọi con số đo sau dòng này sẽ nói về dữ liệu sai.',
    )
  }

  return anh
}

/** In một khối JSON có nhãn — cùng khuôn `2-9-ban-do`, để đọc lại được từ stdout. */
export function in_(nhan, v) {
  console.log(`\n[2.10 · ${nhan}] ` + JSON.stringify(v, null, 2))
}

/**
 * 🔴 **VẠCH LỀ KHÔNG PHẢI MỘT `data-` ATTRIBUTE — đo 2026-08-18, và bản đầu của bàn đo này đã
 * đoán sai đúng chỗ đó.**
 *
 * Bản đầu hỏi `document.querySelectorAll('[data-caret]')`, vì Cạm bẫy ④ của story gọi tên
 * nguyên nhân hiệu năng là *"`:data-caret` buộc Vue tính lại `ruleById`"*. Mở `GridPanel.vue`
 * ra thì **không có thuộc tính nào tên như vậy** trong cây hôm nay: vạch lề là một `class`
 * *(`ruleClassOf` ⇒ `rule-primary`, `editorSegments.ts:230-232`)* đặt trên một `<div>` con của
 * **cột ①** `.col-rule` (`GridPanel.vue:1320-1324`), một cột **riêng** không mang
 * `data-segment-id`.
 *
 * ⇒ Một `querySelectorAll('[data-caret]')` trả **mảng rỗng ở mọi trạng thái**, và bàn đo sẽ in
 * ra *"caret không dời"* cho **cả** ca lệnh chạy lẫn ca lệnh bị nuốt — một thước cho cùng một
 * số ở hai thế giới khác nhau, tức không đo gì. Đúng Cạm bẫy ⑨ *("mở tệp mà đọc trước khi thừa
 * kế một trích dẫn, kể cả trích dẫn mang dấu 🔴")*, gặp lại ở chính lượt dựng bàn đo.
 *
 * ⚠️ Cột vạch lề khớp hàng bằng **thứ tự tài liệu**, không bằng id — nên hàm này trả **chỉ số
 * hàng**, và nơi gọi tự tra `data-segment-id` từ cột `tgt` cùng chỉ số nếu cần một danh tính.
 */
export const VACH_PRIMARY = `
function hangCoVachPrimary() {
  const oVach = document.querySelectorAll('.col-rule .cell-rule')
  for (let i = 0; i < oVach.length; i += 1) {
    if (oVach[i].querySelector('.rule-primary') !== null) {
      const tgt = document.querySelectorAll('[data-col="tgt"]')[i]
      return { hang: i, segmentId: tgt ? tgt.getAttribute('data-segment-id') : null }
    }
  }
  return { hang: null, segmentId: null }
}
`
