/**
 * `historyTimeLabel` — Story 2.6 · AC5 · Quyết định #6 đường (b).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHÔNG `vi.useFakeTimers()` TRONG TỆP NÀY, VÀ ĐÓ LÀ MỘT LUẬT
 * ─────────────────────────────────────────────────────────────────────────────
 * Hàm nhận `now` **qua tham số**, nên mọi ca ở đây **tất định và tức thời**. Bọc một đồng hồ
 * giả quanh một hàm đã nhận thời điểm là đổi một **bảo đảm** *(hàm không đọc đồng hồ)* lấy một
 * **thói quen** *(test dựng đồng hồ)* — và nó che mất chính bảo đảm đó ở lượt ai đó thêm một
 * `Date.now()` vào hàm.
 *
 * ⚠️ Bốn nhánh, và ca cuối cùng của tệp này là ca **biên** — một mốc ở tương lai.
 */
import { describe, expect, it } from 'vitest'

import { historyTimeLabel } from '../../src/panels/segmentHistoryTime'

/**
 * Dựng một mốc ISO-8601 UTC có mili giây từ một `Date`, **đúng hình dạng Rust gửi**
 * (`strftime('%Y-%m-%dT%H:%M:%fZ','now')`).
 */
function iso(d: Date): string {
  return d.toISOString().replace(/\.(\d{3})Z$/, '.$1Z')
}

/** Mốc "bây giờ" cố định cho mọi ca: 2026-08-16 14:30 **giờ địa phương**. */
const NOW = new Date(2026, 7, 16, 14, 30, 0, 0).getTime()

describe('historyTimeLabel — bốn nhánh, thời điểm đi vào qua tham số', () => {
  it('dưới một phút đọc là "vừa xong", không kèm một con số nào', () => {
    const at = iso(new Date(NOW - 30_000))
    expect(historyTimeLabel(at, NOW)).toEqual({ key: 'history.time_just_now', params: {} })
  })

  it('đúng mốc 0 ms cũng là "vừa xong"', () => {
    expect(historyTimeLabel(iso(new Date(NOW)), NOW).key).toBe('history.time_just_now')
  })

  it('từ một phút tới dưới một giờ đọc là "N phút trước", và N là DỮ LIỆU chứ không phải câu', () => {
    const at = iso(new Date(NOW - 12 * 60_000))
    expect(historyTimeLabel(at, NOW)).toEqual({
      key: 'history.time_minutes_ago',
      params: { minutes: '12' },
    })
  })

  it('biên dưới của nhánh phút là đúng 60 000 ms — 59 999 vẫn là "vừa xong"', () => {
    expect(historyTimeLabel(iso(new Date(NOW - 59_999)), NOW).key).toBe('history.time_just_now')
    expect(historyTimeLabel(iso(new Date(NOW - 60_000)), NOW)).toEqual({
      key: 'history.time_minutes_ago',
      params: { minutes: '1' },
    })
  })

  it('biên trên của nhánh phút là một giờ — 59 phút còn đếm, 60 phút thì thôi', () => {
    expect(historyTimeLabel(iso(new Date(NOW - 59 * 60_000)), NOW)).toEqual({
      key: 'history.time_minutes_ago',
      params: { minutes: '59' },
    })
    expect(historyTimeLabel(iso(new Date(NOW - 60 * 60_000)), NOW).key).not.toBe(
      'history.time_minutes_ago',
    )
  })

  it('hôm qua đọc là "Hôm qua HH:mm", và giờ là giờ ĐỊA PHƯƠNG', () => {
    // 2026-08-15 21:04 gio dia phuong.
    const at = iso(new Date(2026, 7, 15, 21, 4, 0, 0))
    expect(historyTimeLabel(at, NOW)).toEqual({
      key: 'history.time_yesterday',
      params: { clock: '21:04' },
    })
  })

  /**
   * 🔴 **Cái bẫy `toISOString()`, và ca này tự chọn CHIỀU theo múi giờ đang chạy.**
   *
   * `toISOString()` trả về theo **UTC**, nên một phép so ngày bằng nó rẽ sai ở hai chiều
   * ngược nhau tuỳ dấu của offset:
   * - **offset dương** *(máy của Ice: UTC+7)* — mốc **sáng sớm hôm nay** rơi về ngày hôm
   *   trước theo UTC ⇒ màn hình nói *"hôm qua"* cho một câu vừa ký sáng nay;
   * - **offset âm** — mốc **tối muộn hôm qua** nhảy sang ngày hôm nay theo UTC ⇒ màn hình
   *   **thôi** nói *"hôm qua"* cho một câu ký tối qua.
   *
   * ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tưởng đã được canh:** ở **UTC đúng**
   * (offset 0) cả hai chiều biến mất và ca này **rỗng nghĩa** — nó xanh kể cả trên một hàm
   * dùng `toISOString()`. Runner CI của GitHub chạy **UTC**. ⇒ Mệnh đề này được canh **trên
   * máy của người chạy pre-push**, không trên CI. Đã ghi nợ có chủ.
   */
  it('một mốc CÙNG NGÀY địa phương không được rẽ sai vì UTC — cái bẫy `toISOString()`', () => {
    const offsetMin = -new Date(NOW).getTimezoneOffset()

    if (offsetMin > 0) {
      // Sang som HOM NAY: 06:30 dia phuong. O UTC+7 thi day la 23:30 NGAY HOM TRUOC theo UTC.
      const at = iso(new Date(2026, 7, 16, 6, 30, 0, 0))
      const label = historyTimeLabel(at, NOW)
      expect(label.key).toBe('history.time_absolute')
      expect(label.params.date).toBe('2026-08-16')
      expect(label.params.clock).toBe('06:30')
      return
    }

    if (offsetMin < 0) {
      // Toi muon HOM QUA: 23:30 dia phuong. O UTC-5 thi day la 04:30 HOM NAY theo UTC.
      const at = iso(new Date(2026, 7, 15, 23, 30, 0, 0))
      const label = historyTimeLabel(at, NOW)
      expect(label.key).toBe('history.time_yesterday')
      expect(label.params.clock).toBe('23:30')
      return
    }

    // offset === 0 ⇒ ca nay RONG NGHIA o day. Khang dinh chinh dieu do thay vi gia vo da do.
    expect(offsetMin).toBe(0)
  })

  it('xa hơn hôm qua đọc là mốc tuyệt đối đầy đủ', () => {
    const at = iso(new Date(2026, 7, 10, 9, 5, 0, 0))
    expect(historyTimeLabel(at, NOW)).toEqual({
      key: 'history.time_absolute',
      params: { date: '2026-08-10', clock: '09:05' },
    })
  })

  it('phân giải được một mốc mang MILI GIÂY và hậu tố Z — hình dạng Rust thật sự gửi', () => {
    // Khang dinh no thay vi tin: `%f` cua SQLite cho ba chu so mili giay.
    const at = '2026-08-16T07:29:00.123Z'
    const parsed = new Date(at).getTime()
    expect(Number.isNaN(parsed)).toBe(false)
    // Mot mili giay khong duoc lam nhanh nao re sai.
    expect(historyTimeLabel(at, parsed + 30_000).key).toBe('history.time_just_now')
  })

  it('một mốc KHÔNG phân giải được rơi về nhánh tuyệt đối mang NGUYÊN VĂN, không ném và không rỗng', () => {
    const label = historyTimeLabel('khong-phai-mot-moc', NOW)
    expect(label.key).toBe('history.time_absolute')
    expect(label.params.date).toBe('khong-phai-mot-moc')
  })

  it('🔴 một mốc ở TƯƠNG LAI không được cho ra "-1 phút trước" — chốt chống đồng hồ không đơn điệu', () => {
    // Dong ho he thong vua bi chinh lui, hoac mot `.atproj` chep tu may khac.
    const at = iso(new Date(NOW + 30 * 60_000))
    const label = historyTimeLabel(at, NOW)
    expect(label.key).toBe('history.time_just_now')
    // Khong mot con so am nao lot ra ngoai.
    expect(Object.values(label.params).some((v) => v.startsWith('-'))).toBe(false)
  })
})
