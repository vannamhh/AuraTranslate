/**
 * Định dạng **thời điểm ký** của một phiên bản segment — Story 2.6 · FR101 · AC5 ·
 * Quyết định #6 đường (b) (Ice ký 2026-08-16).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 ĐÂY LÀ QUY ƯỚC ĐỊNH DẠNG THỜI GIAN **ĐẦU TIÊN** CỦA CẢ KHO
 * ─────────────────────────────────────────────────────────────────────────────
 * **Số đo 2026-08-16:** `grep -rn "toLocale\|Intl\.DateTimeFormat\|new Date(" src` cho **0**
 * kết quả. Bề mặt *"thời gian trôi"* duy nhất đang sống — `StatusBar.vue` (*"Đã lưu N giây
 * trước"*) — làm **số học epoch thuần**, không gọi một API ngày nào.
 *
 * ⇒ Mọi thứ ở đây là một tiền lệ, không một lượt theo khuôn có sẵn. Ghi ra thay vì để người
 * sau tưởng đã có luật.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 HÀM TRẢ VỀ MỘT **MÔ TẢ**, KHÔNG MỘT CHUỖI — và đó là thứ giữ được ba luật cùng lúc
 * ─────────────────────────────────────────────────────────────────────────────
 * Trả `{ key, params }` thay vì một câu đã dựng sẵn:
 * - **chuỗi ở lại `vi.json`** — một câu dựng trong `.ts` là một câu nằm ngoài `check:i18n`;
 * - **tham số mang DỮ LIỆU, không mang CÂU** *(`project-context.md`)* — nơi gọi ghép, hàm này
 *   chỉ quyết **nhánh nào** và **số nào**;
 * - hàm **kiểm được tất định** mà không cần dựng một bảng chuỗi giả trong test.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 HÀM **KHÔNG** TỰ ĐỌC `Date.now()` — mọi thời điểm đi vào qua tham số
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng luật đã có cho `layout/writeSchedule.ts`, và lý do ghi tại chỗ ở đó: một hàm tự đọc
 * đồng hồ buộc phép kiểm phải `sleep` thật hoặc bọc một đồng hồ giả — tức một phép kiểm
 * **chậm và chập chờn**. ⇒ Test truyền `now` cố định và khẳng định từng nhánh, và **không**
 * `vi.useFakeTimers()`: bọc đồng hồ giả cho một hàm đã nhận thời điểm qua tham số là đổi một
 * **bảo đảm** lấy một **thói quen**.
 *
 * ⚠️ Tệp này **không** chịu luật *"erasable-only"*. Đã kiểm 2026-08-16: các cổng `import()`
 * đúng tám tệp — `commands/{registry,keys,index,focus}.ts` · `panels/editorSegments.ts` ·
 * `layout/{workspaceLayout,writeSchedule}.ts` · `i18n/resolve.ts` — và tệp này không nằm
 * trong số đó. *(Nó vẫn được viết không `import` gì, nên nếu một cổng sau này nạp nó thì
 * không có gì phải gỡ.)*
 */

/** Khoá `vi.json` cho từng nhánh. Danh mục **đóng** — bốn nhánh, đúng bốn khoá. */
export type HistoryTimeLabel = {
  key:
    | 'history.time_just_now'
    | 'history.time_minutes_ago'
    | 'history.time_yesterday'
    | 'history.time_absolute'
  /** Tham số mang **dữ liệu**, không mang câu. Rỗng cho nhánh *"vừa xong"*. */
  params: Record<string, string>
}

/** Dưới ngưỡng này đọc là *"vừa xong"* — một phút. */
const JUST_NOW_MS = 60_000
/** Trên ngưỡng này thôi đếm phút. Một giờ. */
const MINUTES_LIMIT_MS = 3_600_000

/** Hai chữ số, đệm `0` — dùng cho giờ và phút. */
function pad2(n: number): string {
  return n < 10 ? `0${n}` : String(n)
}

/**
 * `Date` → `YYYY-MM-DD`, đọc theo **giờ địa phương**.
 *
 * ⚠️ Cố ý **không** dùng `toISOString()`: nó trả về theo **UTC**, nên với một người dùng ở
 * UTC+7 thì mọi mốc trước 07:00 giờ địa phương sẽ rơi về **ngày hôm trước**. Người dịch ký
 * một câu lúc 06:30 sáng và màn hình nói *"hôm qua"* — sai một ngày, im lặng, và chỉ sai với
 * người ở múi giờ dương.
 */
function localDateKey(d: Date): string {
  return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}`
}

/** `Date` → `HH:mm`, giờ địa phương. */
function localClock(d: Date): string {
  return `${pad2(d.getHours())}:${pad2(d.getMinutes())}`
}

/**
 * Chọn nhánh hiển thị cho một mốc ký.
 *
 * @param createdAt `segment_version.created_at` — ISO-8601 UTC có mili giây, đúng như trên dây.
 * @param nowMs thời điểm hiện tại, **epoch mili giây**, đi vào qua tham số.
 *
 * Bốn nhánh, theo thứ tự xét:
 * 1. dưới một phút ⇒ `history.time_just_now`;
 * 2. dưới một giờ ⇒ `history.time_minutes_ago` kèm `{minutes}`;
 * 3. đúng ngày hôm qua *(theo lịch địa phương)* ⇒ `history.time_yesterday` kèm `{clock}`;
 * 4. còn lại ⇒ `history.time_absolute` kèm `{date}` và `{clock}`.
 *
 * ⚠️ **Chốt chống đồng hồ không đơn điệu**, khuôn `Math.max(0, …)` của `StatusBar.vue`: một
 * mốc ở **tương lai** *(đồng hồ hệ thống vừa bị chỉnh lùi, hoặc một tệp `.atproj` chép từ máy
 * khác)* cho `diff` âm. Không có chốt này, `Math.floor(-30000 / 60000)` cho **-1** và màn hình
 * hiện *"-1 phút trước"*. Kẹp về 0 ⇒ nó đọc là *"vừa xong"*, sai một cách **vô hại** thay vì
 * sai một cách khó hiểu.
 *
 * ⚠️ `created_at` không phân giải được ⇒ rơi thẳng về nhánh **tuyệt đối** mang nguyên văn
 * chuỗi đọc được. Không ném, và **không** hiện một câu rỗng: một mốc hỏng phải **nhìn thấy
 * được**, không được biến thành một dòng trống mà không ai lần ra.
 */
export function historyTimeLabel(createdAt: string, nowMs: number): HistoryTimeLabel {
  const at = new Date(createdAt)
  const atMs = at.getTime()

  if (Number.isNaN(atMs)) {
    return { key: 'history.time_absolute', params: { date: createdAt, clock: '' } }
  }

  const diff = Math.max(0, nowMs - atMs)

  if (diff < JUST_NOW_MS) {
    return { key: 'history.time_just_now', params: {} }
  }

  if (diff < MINUTES_LIMIT_MS) {
    return {
      key: 'history.time_minutes_ago',
      params: { minutes: String(Math.floor(diff / JUST_NOW_MS)) },
    }
  }

  const now = new Date(nowMs)
  const yesterday = new Date(nowMs)
  yesterday.setDate(yesterday.getDate() - 1)

  if (localDateKey(at) === localDateKey(yesterday) && localDateKey(at) !== localDateKey(now)) {
    return { key: 'history.time_yesterday', params: { clock: localClock(at) } }
  }

  return {
    key: 'history.time_absolute',
    params: { date: localDateKey(at), clock: localClock(at) },
  }
}
