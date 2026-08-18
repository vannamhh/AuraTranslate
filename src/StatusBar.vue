<script setup lang="ts">
// Thanh trạng thái — Story 2.3 · AC7 · AC9 · AC10 · UX-DR30 · `EXPERIENCE.md:127`.
//
// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 ĐÂY LÀ LƯỢT ĐẦU TIÊN THANH TRẠNG THÁI TỒN TẠI TRONG ỨNG DỤNG
// ═════════════════════════════════════════════════════════════════════════════════
// Đo 2026-08-12 lúc dựng story: `App.vue` có `header.titlebar` · `div.modeport` ·
// `pre.selftest` · dải báo lỗi cấu hình, và **không một phần tử nào** là thanh trạng thái.
// Token thì đã có từ Story 1.4 (`spacing.status-height`), phần tử thì chưa.
//
// **Chỗ ở — `App.vue::main.shell`, dưới `.modeport`** (Quyết định #5, đường (a)). Nó là **vỏ
// ứng dụng**, không nội thất một chế độ: `EXPERIENCE.md:417` đã dùng nó để tính chiều cao
// vùng làm việc cho **cả ba** chế độ, và UX-DR15 hứa Panel Lookup *"rút về thanh trạng thái"*
// ở màn hình hẹp (Story 4.12) — một thanh chỉ sống trong `WorkspaceMode.vue` sẽ phải chuyển
// chỗ lần nữa ở đó.
//
// ⚠️ Story này dựng **cái vỏ + ĐÚNG MỘT thông điệp**. **Không** dựng khung mở rộng cho các
// thông điệp tương lai — chưa story nào đặt hàng chúng, và một khung cho nhu cầu chưa tồn tại
// là một khung sẽ sai.
//
// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 UX-DR30 — CẤM DẤU CHẤM *"CHƯA LƯU"*, CẤM HỘP THOẠI, VÀ CẤM CẢ BIẾN THỂ ĐỘI LỐT
// ═════════════════════════════════════════════════════════════════════════════════
// AC7 nguyên văn: *"thanh trạng thái ghi «Đã lưu N giây trước» · và không có hộp thoại và
// không có dấu chấm «chưa lưu»"*. `EXPERIENCE.md:127` nói cùng một điều.
//
// ⚠️ Và một câu *"đang lưu…"* nhấp nháy mỗi 2 giây là **cùng một tiếng ồn dưới một cái tên
// khác** — nên tệp này không có nó. Trạng thái *"đang bay"* của một lô flush **không** hiện ra:
// hợp đồng với người dùng là *"bạn không phải bận tâm về việc lưu"*, và một chỉ báo nhấp nháy
// là đúng thứ bắt họ bận tâm.
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { t, tError } from './i18n'
import {
  editorConfirmNotice,
  editorNavNotice,
  editorLastSavedAt,
  editorRegroupError,
  editorRegroupNotice,
} from './panels/editorPanelState'

/**
 * Đồng hồ của thanh này — **một `setInterval` 1 giây DUY NHẤT**, và nó chỉ chạy khi có gì để đếm.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * Vì sao một bộ đếm chứ không *"chỉ cập nhật khi có flush"* — Quyết định #5
 * ─────────────────────────────────────────────────────────────────────────────
 * Cập nhật-khi-có-flush làm câu đứng yên ở *"Đã lưu 0 giây trước"* trong suốt 5 giây giữa hai
 * lượt flush — tức nó **nói dối theo hướng an tâm**, đúng thứ UX-DR30 tồn tại để tránh.
 *
 * Cái giá đã cân bằng số: một lượt gán `ref` mỗi giây chạm **một** text node — khác hẳn hạng
 * với lượt `v-for` trên 9.850 `<span>` mà `deferred-work.md:2143-2152` đã ghi cho Panel Editor.
 */
const now = ref(Date.now())
let ticker: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  ticker = setInterval(() => {
    // ⚠️ Chỉ đánh nhịp khi ĐÃ có một lượt flush. Trước đó thanh không hiển thị gì, nên một
    // lượt gán mỗi giây là một lượt render không ai đọc.
    if (editorLastSavedAt.value !== null) now.value = Date.now()
  }, 1000)
})

onBeforeUnmount(() => {
  // 🔴 Phải nhả. Một `setInterval` sống sót lượt tháo là một lượt gán `ref` mỗi giây vào một
  // component không ai còn render — cùng luật `GridPanel.vue::onBeforeUnmount` đã giữ cho
  // `ResizeObserver` và bộ hẹn đo vạch.
  if (ticker !== null) clearInterval(ticker)
  ticker = null
})

/**
 * *"N giây trước"* — **tính và định dạng ở frontend**, Rust không trả về một chữ nào (AC10).
 *
 * AD-21 (`ARCHITECTURE-SPINE.md:302-306`): Rust không bao giờ trả văn bản hiển thị.
 * §Consistency Conventions: **định dạng số và ngày giờ chỉ ở frontend**, và `params` của
 * `IpcError` là `chuỗi → chuỗi`. ⇒ mốc flush cuối là một `number` trong state TS
 * (`editorLastSavedAt`), `N` là một phép trừ ở đây, và câu là một khoá `vi.json` có placeholder.
 * 🔴 **Cấm** một command IPC trả `"Đã lưu 3 giây trước"`.
 *
 * ⚠️ `Math.max(0, …)`: `Date.now()` **không đơn điệu tăng** — một lượt đồng bộ NTP hay người
 * dùng đổi giờ hệ thống lùi lại cho ra một hiệu **âm**, và *"Đã lưu −4 giây trước"* là một câu
 * không ai đọc được. Kẹp ở 0 nói đúng thứ duy nhất còn đúng: *vừa mới xong*.
 */
const secondsSinceSave = computed<number | null>(() => {
  const at = editorLastSavedAt.value
  if (at === null) return null
  return Math.max(0, Math.floor((now.value - at) / 1000))
})

/**
 * 🔵 **2026-08-15 (code review) — vế còn thiếu của Quyết định #8.**
 *
 * Chữ ký của Ice 2026-08-14 khai hợp đồng UX-DR30 tối thiểu là *"cột nhãn trạng thái của chính
 * hàng CỘNG **một dòng ở thanh trạng thái**"*. Vế sau chưa từng được dựng: ba kết quả
 * `'no-caret'` · `'flush-failed'` · `'still-dirty'` chỉ rơi vào một `console.warn`, nên một cú
 * `⌘Enter` không đổi **một pixel nào**. Đây là người đọc đầu tiên của [`editorConfirmNotice`].
 *
 * 🔴 **Bảng tra ĐÓNG, không một nhánh mặc định.** Ba giá trị là một danh mục đóng ở
 * `ConfirmResult`; một `?? 'khoá nào đó'` ở đây sẽ nuốt im lặng một giá trị thứ tư được thêm
 * sau này — đúng lớp lỗi mà cả tệp này tồn tại để chống. Thêm một kết quả vào `ConfirmResult`
 * mà quên bảng này ⇒ `vue-tsc` đỏ, vì `Record` đòi đủ khoá.
 *
 * ⚠️ **Câu này KHÔNG đi cùng mốc *"Đã lưu"*, nó THAY chỗ đó** — xem `v-if`/`v-else` ở template.
 * Hai câu cùng lúc trên một thanh 34px là hai mệnh đề tranh nhau một chỗ, và mệnh đề *"chưa lưu
 * được"* thì khẩn hơn *"đã lưu 12 giây trước"*.
 */
const CONFIRM_NOTICE_KEYS: Record<NonNullable<typeof editorConfirmNotice.value>, string> = {
  'no-caret': 'panel.grid.confirm_no_caret',
  'flush-failed': 'panel.grid.confirm_flush_failed',
  'still-dirty': 'panel.grid.confirm_still_dirty',
}

const confirmNoticeKey = computed<string | null>(() => {
  const notice = editorConfirmNotice.value
  if (notice === null) return null
  return CONFIRM_NOTICE_KEYS[notice]
})

/**
 * 🔴 **Story 2.9 · AC4 — bảng tra THỨ HAI, và nó cũng ĐÓNG.**
 *
 * Bảy khoá, đóng trên `Exclude<RegroupNotice, 'refused'>`. `'refused'` **không** có mặt ở đây
 * có chủ ý: lý do từ chối là câu chữ của **Rust** *(`err.segment.no_previous` và họ hàng)*, đi
 * ra màn hình qua `tError()`. Chép nó thành một khoá thứ bảy ở đây là dựng nguồn sự thật thứ
 * hai cho cùng một mệnh đề — và bản sao sẽ lệch vào ngày Rust thêm một lý do.
 *
 * ⚠️ **Vì sao KHÔNG nới `CONFIRM_NOTICE_KEYS` ra:** bảng kia đóng trên `ConfirmResult`, và
 * toàn bộ giá trị của nó là `vue-tsc` **đỏ** khi ai đó thêm một kết quả mà quên bảng. Nới nó
 * thành `string` để nhét thêm câu của lượt gộp gỡ đúng cái chốt ấy, cho **cả hai** lượt.
 * ⇒ Hai bảng, cả hai đóng. Cái giá là một nhánh `v-else-if` nữa ở template.
 *
 * ⚠️ `'split'` · `'no-cut'` **không** thuộc phạm vi Story 2.9 *(chúng là đường `⌘/` của Story
 * 2.8)*, nhưng chúng đi qua **cùng** `regroup()` nên bảng phải phủ chúng — một `Record` thiếu
 * khoá không biên dịch được, và đó chính là chốt đang làm việc.
 *
 * ✅ **Và chốt ấy đã làm việc thật, 2026-08-17.** Lượt code review thêm `'busy'` vào
 * `RegroupResultCode` cho khoá chống-gọi-lại của `regroup()`; `RegroupNotice` suy ra từ kiểu đó
 * nên bảng này **đỏ ngay** ở `vue-tsc` cho tới khi khoá thứ bảy có mặt. Đó là một phép đo, không
 * một lời hứa: cơ chế được nêu ở đây đã bắt đúng lượt sửa đầu tiên chạm vào nó.
 */
const REGROUP_NOTICE_KEYS: Record<
  Exclude<NonNullable<typeof editorRegroupNotice.value>, 'refused'>,
  string
> = {
  merged: 'panel.grid.regroup_merged',
  split: 'panel.grid.regroup_split',
  'no-caret': 'panel.grid.regroup_no_caret',
  'no-cut': 'panel.grid.regroup_no_cut',
  'flush-failed': 'panel.grid.regroup_flush_failed',
  'still-dirty': 'panel.grid.regroup_still_dirty',
  busy: 'panel.grid.regroup_busy',
}

/**
 * Câu của lượt gộp/tách gần nhất, hoặc `null`.
 *
 * 🔴 Nhánh `'refused'` đọc `editorRegroupError` — **người đọc đầu tiên** của ô nhớ đó. Trước
 * story này nó được export mà không component nào đọc, nên một lượt từ chối *(ví dụ
 * `segment.no_previous` ở câu đầu Chương — ca **thường nhất** của cử chỉ `Backspace`)* **không
 * đổi một pixel nào**. Đúng lớp *"rỗng IM LẶNG"* mà `project-context.md` cấm.
 *
 * ⚠️ `error === null` cùng lúc với `notice === 'refused'` là ca *"không có cầu IPC"*
 * (`regroup()` ghi cả hai). Nó vẫn phải **nói ra** — im lặng ở đây là đúng thứ vừa vá xong,
 * nên nhánh dùng câu chung của `'flush-failed'`: thao tác chưa xong, dữ liệu còn nguyên.
 */
const regroupNoticeText = computed<string | null>(() => {
  const notice = editorRegroupNotice.value
  if (notice === null) return null
  if (notice !== 'refused') return t(REGROUP_NOTICE_KEYS[notice])
  const err = editorRegroupError.value
  if (err === null) return t(REGROUP_NOTICE_KEYS['flush-failed'])
  // ⚠️ **MỘT tham số, không hai.** `tError(err, err.params)` là thừa và `check:lint` đỏ đúng
  // chỗ: `tError` đã tự đọc `err.params` ở nhánh cuối (`i18n/index.ts`, `t(key, params ??
  // err.params)`). Tham số thứ hai chỉ dành cho nơi gọi muốn **ghi đè** bảng tham số.
  return tError(err)
})

/**
 * 🔴 **Story 2.10 · AC6 · AC7 — bảng tra THỨ BA, và nó cũng ĐÓNG.**
 *
 * Bốn khoá, đóng trên `NavNotice`. Cùng cơ chế và cùng lý do hai bảng trên: thêm một giá trị
 * vào `NavNotice` mà quên bảng này ⇒ `vue-tsc` **đỏ**, vì `Record` đòi đủ khoá.
 *
 * ⚠️ **Vì sao không có nhánh `'refused'` như bảng gộp/tách:** ba lệnh điều hướng **không đi qua
 * Rust**. Chúng là phép chọn trên một ảnh chụp đã nằm sẵn trong bộ nhớ, nên không có `IpcError`
 * nào để đọc và không có lý do từ chối nào thuộc về Rust.
 *
 * 🔴 **Đây là ô nhớ thứ BA, và cái giá của nó đã được nêu trước khi ký** *(Quyết định #4 đường
 * (b), Ice ký 2026-08-18)*: bất biến *"ai ghi một ô thì dọn các ô còn lại"* thành **N chiều**.
 * Nó được trả bằng cách dựng **một cửa ghi duy nhất** — `editorPanelState.ts::datThongBao` —
 * thay vì bằng ba lời gọi dọn rải rác. ⇒ Thứ tự `v-if`/`v-else-if` ở template dưới đây, y như
 * hai nhánh trước, **không quan sát được**; nó là hàng rào cho ca không lường trước.
 */
const NAV_NOTICE_KEYS: Record<NonNullable<typeof editorNavNotice.value>, string> = {
  'no-untranslated': 'panel.grid.nav_no_untranslated',
  'at-first': 'panel.grid.nav_at_first',
  'at-last': 'panel.grid.nav_at_last',
  'confirmed-last': 'panel.grid.nav_confirmed_last',
  // 🔵 2026-08-18 (code review lượt HAI) — khoá thứ NĂM. Bảng `Record` đóng đã làm đúng việc
  //    nó được dựng để làm: thêm `'loading'` vào `NavNotice` mà chưa sửa đây thì `vue-tsc` đỏ.
  loading: 'panel.grid.nav_loading',
}

const navNoticeKey = computed<string | null>(() => {
  const notice = editorNavNotice.value
  if (notice === null) return null
  return NAV_NOTICE_KEYS[notice]
})
</script>

<template>
  <!--
    `role="status"` chứ không `role="alert"`: `alert` cắt ngang trình đọc màn hình giữa câu, và
    đây là một thông báo về **trạng thái** — cùng phân xử mà dải báo lỗi cấu hình của Story 1.8
    đã ghi ở `App.vue`.

    ⚠️ `aria-live` để mặc định của `role="status"` (`polite`). Một mốc *"Đã lưu N giây trước"*
    đổi **mỗi giây**, nên `assertive` sẽ là một trình đọc màn hình nói liên tục không dứt.
  -->
  <footer class="status" role="status">
    <!--
      aura-allow-text: KẾT QUẢ của `t()` — chuỗi đã đi qua `vi.json`. `v-if` chứ không một chuỗi
      rỗng: trước lượt flush ĐẦU TIÊN thanh này **không khẳng định gì**, vì *"Đã lưu 0 giây
      trước"* lúc chưa lưu gì là câu nói dối tệ nhất mà UX-DR30 tồn tại để chặn.
    -->
    <!--
      aura-allow-text: KẾT QUẢ của `t()`. 🔵 2026-08-15 — câu báo của lượt xác nhận đứng **TRƯỚC**
      mốc *"Đã lưu"* và **thay** chỗ nó (`v-if`/`v-else-if`, không hai `v-if` rời). Lý do ở
      [`CONFIRM_NOTICE_KEYS`]: thanh cao 34px chỉ đủ một mệnh đề, và *"chưa lưu được bản dịch"*
      khẩn hơn *"đã lưu 12 giây trước"*. Câu này tự tắt khi người dùng gõ tiếp — dọn bằng SỰ KIỆN
      ở `noteEditorEdit`, không bằng một hẹn giờ.
    -->
    <span v-if="confirmNoticeKey !== null" class="notice">{{ t(confirmNoticeKey) }}</span>
    <!--
      aura-allow-text: KẾT QUẢ của `t()`/`tError()`. 🔵 Story 2.9, AC4 — câu của lượt gộp/tách.

      🔴 Nó đứng **SAU** câu xác nhận và **TRƯỚC** mốc *"Đã lưu"*, và thứ tự đó là một quyết
      định: một lượt `⌘Enter` bị từ chối nói *"chưa lưu được bản dịch"* — khẩn hơn *"đã gộp
      hai câu"*, vì một cái nói dữ liệu chưa an toàn còn cái kia nói một thao tác đã xong.
      Và cả hai đều khẩn hơn *"đã lưu 12 giây trước"*.

      🔵 **2026-08-17 — mệnh đề cũ ở đây ĐÃ HẾT ĐÚNG, và nó sai ngay từ lượt viết ra.** Bản cũ
      khẳng định *"hai ô nhớ không bao giờ cùng mang giá trị trong một lượt dùng thật: cả hai được
      dọn ở `noteEditorEdit`"*. Code review ba tầng bác nó bằng số đo, và **hai tầng độc lập tái
      hiện bằng vitest**: `confirmNotice` chỉ bị dọn ở **ba** cửa, `regroup()` không là một trong
      ba, và cử chỉ `Backspace` **không** đi qua `noteEditorEdit` *(`preventDefault()` cắt hẳn
      chuỗi `beforeinput`→`input`→`reportEdit`)*. ⇒ Một `⌘Enter` hụt rồi một lượt gộp **thành
      công** để thanh này hiện câu của lượt hụt, tức AC4 không đạt.

      ✅ **Nay mệnh đề ấy đúng theo CẤU TẠO, không theo một lời hứa.** Bất biến sống ở
      `editorPanelState.ts::ghiRegroupNotice` — *"thao tác vừa xảy ra sở hữu thanh trạng thái"*:
      ai ghi một ô thì dọn ô còn lại, và vế đối xứng nằm ở cửa có khoá của `confirmCurrentSegment`.
      🔴 Vì thế thứ tự `v-if`/`v-else-if` ở đây **không còn quan sát được** — nó là hàng rào cho ca
      không lường trước, và **đừng** đọc nó thành một luật ưu tiên đang làm việc. Sửa thứ tự khi
      thấy một câu sai chỉ dời chỗ nói dối sang ô nhớ kia; chỗ phải sửa là bất biến ở tệp trạng thái.
    -->
    <span v-else-if="regroupNoticeText !== null" class="notice">{{ regroupNoticeText }}</span>
    <!--
      aura-allow-text: KẾT QUẢ của `t()`. 🔵 Story 2.10, AC6 · AC7 — câu của lượt điều hướng.

      Nó đứng **SAU** hai câu kia và **TRƯỚC** mốc *"Đã lưu"*, cùng thang ưu tiên: một câu nói
      *"dữ liệu chưa an toàn"* khẩn hơn một câu nói *"một thao tác đã xong"*, và cả hai khẩn hơn
      *"đã ở câu cuối Chương"* — vế cuối này chỉ trả lời một phím vừa bấm, không cảnh báo gì.

      🔴 Y như hai nhánh trên: thứ tự ở đây **không quan sát được** khi bất biến của
      `datThongBao` đứng. Thấy một câu sai thì sửa **bất biến**, đừng sửa thứ tự — đổi thứ tự
      chỉ dời chỗ nói dối sang một ô nhớ khác.
    -->
    <span v-else-if="navNoticeKey !== null" class="notice">{{ t(navNoticeKey) }}</span>
    <span v-else-if="secondsSinceSave !== null" class="saved">{{
      t('status.saved_seconds_ago', { seconds: String(secondsSinceSave) })
    }}</span>
  </footer>
</template>

<style scoped>
/*
 * AC9 — ba luật đã có, cả ba tuân qua token:
 *   (a) chiều cao đọc `var(--space-status-height)` — **34px**, KHÔNG `32px` của mockup
 *       (`key-screen-workspace.html:73`) và không `32px` của `DESIGN.md:132`. 34px là số trong
 *       bảng token và trong `tokens.json:480`, và `EXPERIENCE.md:312` phân xử rằng tài liệu
 *       thắng bản dựng. Lệch giữa hai chỗ ghi trong `DESIGN.md` đã ghi vào `deferred-work.md`
 *       với **chủ là Ice** — Dev KHÔNG sửa `DESIGN.md`.
 *   (b) typography `ui-sm` — vai *"Nhãn phụ — một dòng"* của `tokens.json:442`, đúng vai một
 *       thanh trạng thái. Qua token, không một con số viết thẳng ⇒ Kiểm B/B2 của `check-tokens`.
 *   (c) mọi text node đi qua `t()` ⇒ Kiểm A2 của `check-i18n`.
 *
 * ⚠️ `flex: none` là bắt buộc, không làm đẹp: `.shell` là một flex container dọc cao đúng
 * `100vh` và `.modeport` là `flex: 1`. Không có nó, thanh này chạy ở `flex: 0 1 auto` và bị co
 * lại khi vùng làm việc đòi chỗ — cùng bài học mà `.config-error` và `.selftest` đã ghi.
 */
.status {
  display: flex;
  align-items: center;
  flex: none;
  height: var(--space-status-height);
  padding: 0 var(--space-panel-inline);
  border-top: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.saved {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

/*
 * 🔵 2026-08-15 — câu báo của lượt xác nhận. **Cùng vai chữ với `.saved`**, không một vai cảnh
 * báo riêng.
 *
 * 🔴 Vì sao KHÔNG tô màu lỗi: `check-tokens` Kiểm F cấm bóng đổ / gradient / lớp nổi, và
 * UX-DR16 cấm hộp thoại — nhưng vế quyết định là ở chỗ khác. Ba câu đi qua đây nói *"thao tác
 * chưa xong"*, không *"dữ liệu hỏng"*; nhuộm đỏ chúng là dạy người dùng rằng thanh trạng thái
 * có một mức khẩn cấp, và mức đó sẽ phải lạm phát ở story sau. Chữ nói đủ.
 */
.notice {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}
</style>
