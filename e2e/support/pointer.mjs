/**
 * Chuỗi sự kiện chuột THẬT cho bàn đo e2e — Story 1.22, AC3.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO KHÔNG DÙNG `element.click()` CỦA DRIVER — một phép đo, không một sở thích
 * ═════════════════════════════════════════════════════════════════════════════════
 * Đo được ở lượt dựng bộ e2e (2026-08-11): lệnh `click` của driver **không trung thực về
 * thứ tự sự kiện**. Nó bắn `click` **TRƯỚC** `focusin`, trong khi chuột thật đi
 *
 *     mousedown -> focusin -> mouseup -> click
 *
 * Hệ quả cụ thể, và nó là lý do hàm này tồn tại: `shortcuts.capture` chạy lúc `aimedRow`
 * còn rỗng, màn hình trả về câu *"Chưa nhắm được thao tác nào"*, và lượt chạy **ĐỎ với
 * một câu đổ lỗi cho sản phẩm** trong khi lỗi nằm ở bộ lái. Một ca đỏ nói sai nguyên nhân
 * đắt hơn một ca không tồn tại: nó gửi người đọc đi sửa đúng thứ đang chạy tốt.
 *
 * ⚠️ Và chiều ngược lại còn tệ hơn, vì nó im lặng: ở một hàng mà thứ tự sự kiện quyết
 * định kết quả, `element.click()` có thể cho một lượt **XANH** trên một sản phẩm đang
 * hỏng — tức bàn đo mất đúng lớp lỗi nó được dựng ra để bắt.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * LUẬT, KHÔNG PHẢI QUY ƯỚC
 * ═════════════════════════════════════════════════════════════════════════════════
 * `eslint.config.js` cấm mọi lời gọi `.click()` trong `e2e/**` bằng `no-restricted-syntax`,
 * và `npm run check:lint` chạy trên cả `e2e` từ 2026-08-11.
 *
 * Lệnh cấm là **toàn phần**, rộng hơn phát biểu của AC3 *(chỉ đòi ở nơi thứ tự có nghĩa)*,
 * và đó là một lựa chọn có lý do: câu hỏi *"hàng này có phụ thuộc thứ tự không"* chính là
 * câu hỏi đã trả lời SAI một lần rồi. `shortcuts-focus.e2e.mjs` khai bằng chữ rằng nó
 * *"cố ý đi đường chuột"* để bắt khuyết tật WKWebView, rồi gọi `opener.click()` — tức đi
 * đúng con đường mà spec bên cạnh vừa đo được là không trung thực. Một luật do người đọc
 * phân xử từng ca là một luật sẽ trôi; giá của việc luôn dùng hàm này là vài chục
 * mili-giây.
 *
 * Cần một ngoại lệ thật thì viết `eslint-disable-next-line no-restricted-syntax` kèm lý do
 * ngay tại chỗ. `reportUnusedDisableDirectives: 'error'` đã bật, nên một ngoại lệ hết cần
 * sẽ bị bắt thay vì lặng lẽ ở lại.
 */

/**
 * Bấm một phần tử bằng chuỗi sự kiện thật.
 *
 * Hai lượt `pause` không phải số ma thuật mà là hai vai khác nhau: `30 ms` giữa `down` và
 * `up` để webview kịp phát `focusin` **trước** `click` — đúng thứ tự mà hàm này tồn tại để
 * dựng lại; `100 ms` sau `up` để vòng cập nhật của Vue chạy xong trước khẳng định kế tiếp.
 *
 * @param {WebdriverIO.Element} element phần tử đích
 */
export async function realClick(element) {
  await browser
    .action('pointer')
    .move({ origin: element })
    .down()
    .pause(30)
    .up()
    .pause(100)
    .perform()
}
