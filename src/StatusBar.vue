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
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { t, tError } from './i18n'
import {
  editorConfirmNotice,
  editorNavNotice,
  editorLastSavedAt,
  editorRegroupError,
  editorRegroupNotice,
  // 🔵 Story 5.8 — ô câu chữ THỨ TƯ: lượt tách CHƯƠNG (`editor.split_chapter`, `Mod+Shift+Slash`).
  editorSplitChapterError,
  editorSplitChapterNotice,
} from './panels/editorPanelState'
// 🔵 Story 3.4b — nhánh THỨ NĂM: bản dịch của một thuật ngữ Glossary đang được rê chuột tới,
// ở cột nguyên văn của lưới. `hoveredGlossaryTerm` ghi bởi `GridPanel.vue`/`SourceHanViet.vue`
// — hai component ANH EM với tệp này, nối qua state module-level cùng khuôn bốn câu kia.
import { hoveredGlossaryTerm } from './panels/glossaryTermHoverState'
import { glossaryMarksError } from './panels/glossaryMarksState'

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

/**
 * 🔴 **Story 3.4b (rà ba lớp, P7) — lỗi tải dấu Glossary có VÒNG ĐỜI HỮU HẠN, không đứng mãi.**
 *
 * Trước bản vá này, `glossaryMarksError` chiếm nhánh thứ năm cho tới khi MỘT lượt tải dấu
 * SAU đó thành công — nhưng không gì đảm bảo còn một lượt như vậy trong phiên (Chương không
 * đổi nữa, không ai gộp/tách hay thêm nhanh thêm). Một lỗi nhất thời (`store` đóng giữa chừng)
 * khi đó CHE VĨNH VIỄN mốc *"Đã lưu N giây trước"* — người dùng đọc ra *"ứng dụng ngừng tự
 * lưu"* trong khi nó vẫn lưu bình thường, đúng lớp đánh đổi tín hiệu THẬT lấy tín hiệu LỖI mà
 * UX-DR30 tồn tại để chống (chỉ khác đối tượng: ở đó là dấu chấm "chưa lưu", ở đây là một câu
 * lỗi không tự biết mình đã cũ).
 *
 * ⇒ Đúng khuôn `now`/`ticker` ngay trên: MỘT `setTimeout` mỗi lượt lỗi MỚI, tự tắt câu sau
 * [`GLOSSARY_MARKS_ERROR_DISPLAY_MS`]. `watch` (không `deep`) so bằng THAM CHIẾU đối tượng —
 * mỗi lượt `loadMarksFor` (`glossaryMarksState.ts`) dựng một `IpcError` MỚI dù nội dung trùng
 * lặp, nên lỗi còn TÁI DIỄN thì đồng hồ tự khởi động lại; lỗi thôi tái diễn thì nó tắt sau
 * đúng một khoảng, và "Đã lưu N giây trước" trở lại chỗ của nó — không cần chờ một Chương/
 * gộp/thêm nhanh MỚI để dọn.
 */
const GLOSSARY_MARKS_ERROR_DISPLAY_MS = 8000
const glossaryMarksErrorVisible = ref(false)
let glossaryMarksErrorTimer: ReturnType<typeof setTimeout> | null = null

watch(glossaryMarksError, (err) => {
  if (glossaryMarksErrorTimer !== null) {
    clearTimeout(glossaryMarksErrorTimer)
    glossaryMarksErrorTimer = null
  }
  if (err === null) {
    glossaryMarksErrorVisible.value = false
    return
  }
  glossaryMarksErrorVisible.value = true
  glossaryMarksErrorTimer = setTimeout(() => {
    glossaryMarksErrorVisible.value = false
    glossaryMarksErrorTimer = null
  }, GLOSSARY_MARKS_ERROR_DISPLAY_MS)
}, { immediate: true }) // 🔴 BẮT BUỘC: một lỗi đã tồn tại TRƯỚC khi `StatusBar` mount (ca
// thường — `GridPanel.vue` nạp Chương/dấu độc lập, `StatusBar` có thể mount SAU) phải hiện
// ngay, không chờ lượt ĐỔI kế tiếp của `glossaryMarksError` mà có thể không bao giờ tới.

onBeforeUnmount(() => {
  // 🔴 Story 3.4b — cùng luật nhả đã ghi ngay dưới, áp cho bộ đếm giờ THỨ HAI của tệp này.
  if (glossaryMarksErrorTimer !== null) clearTimeout(glossaryMarksErrorTimer)
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
 * 🔴 **Story 5.8 — bảng tra THỨ TƯ, và nó cũng ĐÓNG.**
 *
 * ⚠️ **Vì sao nhánh này tồn tại — một phép đo, không một lượt cho đủ bộ.** Đo 2026-08-29 (lượt
 * rà Story 5.8): bản đầu của `editorPanelState.ts::splitChapterHere` báo MỌI đường trượt bằng
 * `console.error` và chỉ thế, kể cả ca thường nhất *(gõ hợp âm khi caret ở câu đầu Chương ⇒
 * `err.chapter.split_leaves_empty`)*. Người dùng gõ `Mod+Shift+Slash` và màn hình **không đổi
 * một pixel nào** — đúng lớp *"rỗng IM LẶNG"*, và lệch hẳn với hai lệnh ANH EM
 * (`editor.merge_segments`/`editor.split_segment`) vốn hiện câu từ chối ngay ở bảng trên.
 *
 * Cùng cơ chế đóng ba bảng kia: thêm một giá trị vào `SplitChapterNotice` mà quên bảng này ⇒
 * `vue-tsc` **đỏ**, vì `Record` đòi đủ khoá.
 */
const SPLIT_CHAPTER_NOTICE_KEYS: Record<
  Exclude<NonNullable<typeof editorSplitChapterNotice.value>, 'refused'>,
  string
> = {
  split: 'panel.grid.split_chapter_done',
  'no-caret': 'panel.grid.split_chapter_no_caret',
  'flush-failed': 'panel.grid.split_chapter_flush_failed',
}

/**
 * Câu của lượt tách Chương gần nhất, hoặc `null`. Nhánh `'refused'` đọc
 * [`editorSplitChapterError`] — cùng khuôn và cùng lý do `regroupNoticeText` ngay trên: một lượt
 * từ chối của Rust *(`chapter.split_leaves_empty`, `segment.not_found`)* phải nói ra ĐÚNG lý do
 * của nó, không rơi về một câu chung.
 *
 * ⚠️ `error === null` cùng lúc với `notice === 'refused'` là ca *"không có cầu IPC"* — vẫn phải
 * nói ra (im lặng ở đây là đúng thứ vừa vá), nên nó mượn câu của `'flush-failed'`: thao tác chưa
 * xong, dữ liệu còn nguyên.
 */
const splitChapterNoticeText = computed<string | null>(() => {
  const notice = editorSplitChapterNotice.value
  if (notice === null) return null
  if (notice !== 'refused') return t(SPLIT_CHAPTER_NOTICE_KEYS[notice])
  const err = editorSplitChapterError.value
  if (err === null) return t(SPLIT_CHAPTER_NOTICE_KEYS['flush-failed'])
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
  // 🔵 2026-08-18 (Story 2.11 · AC4) — khoá thứ SÁU và thứ BẢY. Bảng `Record` đóng lại làm
  //    đúng việc nó được dựng để làm: thêm hai giá trị vào `NavNotice` mà chưa sửa đây thì
  //    `vue-tsc` đỏ. 🔴 Hai khoá RIÊNG, không tái dùng `nav_at_first`/`nav_at_last` — hai
  //    chuỗi đó nói *"câu"*, còn đây là biên **Chương**.
  'at-first-chapter': 'panel.grid.nav_at_first_chapter',
  'at-last-chapter': 'panel.grid.nav_at_last_chapter',
  // 🔵 2026-08-18 (code review ba tầng) — khoá thứ TÁM, CHÍN và MƯỜI. Ba đường hỏng của lượt
  //    chuyển Chương trước lượt rà này nói **sai chỗ**: hai nhánh flush mượn kênh `confirm`
  //    *("…nên chưa **xác nhận**" — người dùng không xác nhận gì cả)*, còn nhánh lỗi IPC ghi
  //    `loadError` và khoá chết cả lưới. 🔴 Cùng lập luận đã dựng ra hai khoá Chương ở trên,
  //    áp cho ba đường còn lại — xem `editorPanelState.ts::NavNotice`.
  'chapter-switch-failed': 'panel.grid.nav_chapter_switch_failed',
  'chapter-flush-failed': 'panel.grid.nav_chapter_flush_failed',
  'chapter-still-dirty': 'panel.grid.nav_chapter_still_dirty',
}

const navNoticeKey = computed<string | null>(() => {
  const notice = editorNavNotice.value
  if (notice === null) return null
  return NAV_NOTICE_KEYS[notice]
})

/**
 * 🔵 **Story 3.4b — câu của dấu thuật ngữ Glossary đang được rê chuột tới.**
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO TÁI DÙNG `glossary.quick_add.translation_label` CHO CA ĐÃ CHỐT, KHÔNG MỘT KHOÁ MỚI
 * ─────────────────────────────────────────────────────────────────────────────
 * `translation` là DỮ LIỆU của người dùng (một mục Glossary họ đã chốt), không một chuỗi giao
 * diện — cùng luật `panel.source.han_viet_sources_prefix` + `sourcesLine` ở
 * `SourceHanViet.vue`: một khoá LÀM TIỀN TỐ, dữ liệu ghép sau. `glossary.quick_add.translation_label`
 * ("Bản dịch") đã tồn tại từ Story 3.3 đúng cho vai này — thêm một khoá thứ hai cho cùng một
 * câu là hai nguồn sự thật cho một mệnh đề, và Story 1.7 §Completion Notes #3 cấm một khoá
 * không có nhánh nào đi qua (ở đây thì NGƯỢC LẠI: một khoá ĐÃ có nhánh, đúc thêm một bản sao
 * mới là điều bị cấm).
 *
 * Ca CHỜ CHỐT thì khác: không có `translation` nào để ghép, nên nó cần đúng MỘT khoá câu
 * hoàn chỉnh — [`glossary.mark.pending_translation`], khoá MỚI duy nhất của story này
 * (`3-4b-…md` §Execution).
 *
 * 🔴 **Đang có câu lỗi xác nhận thì bản dịch KHÔNG được đè lên nó** — I/O Matrix của story.
 * Vị trí `v-else-if` thứ năm ở template (dưới `confirmNoticeKey`/`regroupNoticeText`/
 * `navNoticeKey`) đã tự thoả điều đó: ba nhánh trên thắng vô điều kiện, cùng cơ chế
 * `datThongBao` đã ghi ở `editorPanelState.ts`.
 */
const glossaryHoverText = computed<string | null>(() => {
  const term = hoveredGlossaryTerm.value
  if (term !== null) {
    if (!term.isConfirmed) return t('glossary.mark.pending_translation')
    // `term.translation` chỉ `null` khi `isConfirmed === false` (`GlossaryMark` phía Rust,
    // `commands/glossary.rs::GlossaryMarkWire`) — nhánh trên đã loại ca đó.
    return `${t('glossary.quick_add.translation_label')}: ${term.translation ?? ''}`
  }

  // 🔵 Bổ sung 2026-08-21 (Ice yêu cầu, I/O Matrix "IPC trả lỗi | kho đóng giữa chừng ⇒ lỗi
  // hiện qua tError(), KHÔNG coi như rỗng"). `glossaryMarksState.ts` đã giữ đúng bất biến đó
  // Ở TẦNG STATE (marks rỗng VÀ error khác null là hai điều phân biệt được — xem
  // `glossaryMarksHaveLoaded()`), nhưng trước bản vá này KHÔNG chỗ nào hiện lỗi đó ra màn
  // hình — một lượt IPC trượt lặng lẽ trông giống hệt "Chương này không có thuật ngữ nào".
  // Cùng vị trí ưu tiên với câu hover (bản dịch thuật ngữ): khi không có gì đang hover, một
  // lỗi tải dấu thật thì đáng nói hơn một khoảng trống.
  const err = glossaryMarksError.value
  if (err !== null && glossaryMarksErrorVisible.value) return tError(err)
  return null
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
      aura-allow-text: KẾT QUẢ của `t()`/`tError()`. 🔵 Story 5.8 — câu của lượt tách CHƯƠNG.

      Đứng NGAY SAU câu gộp/tách segment và TRƯỚC câu điều hướng, cùng thang ưu tiên đã ghi ở
      hai nhánh trên: một thao tác GHI (tách Chương) khẩn hơn một câu chỉ trả lời một phím điều
      hướng vừa bấm.

      🔴 Y như ba nhánh trên: thứ tự ở đây **không quan sát được** khi bất biến của
      `datThongBao` đứng — nó nay gán CẢ BỐN ô ở mọi lời gọi. Thấy một câu sai thì sửa bất biến,
      đừng sửa thứ tự.
    -->
    <span v-else-if="splitChapterNoticeText !== null" class="notice">{{ splitChapterNoticeText }}</span>
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
    <!--
      aura-allow-text: KẾT QUẢ của `t()` cộng (ca đã chốt) một chuỗi DỮ LIỆU của Tác phẩm —
      bản dịch một thuật ngữ Glossary do người dùng chốt. 🔵 Story 3.4b — nhánh THỨ NĂM, dưới
      ba thông báo khẩn hơn và TRÊN mốc "Đã lưu" (thứ tự ưu tiên của story: lỗi xác nhận >
      gộp/tách > điều hướng > bản dịch thuật ngữ > "Đã lưu N giây trước").

      role="status" của `<footer>` xướng câu này qua trình đọc màn hình khi nó xuất hiện —
      I/O Matrix: "Thanh hiện bản dịch của thuật ngữ, xướng qua role=status".
    -->
    <span v-else-if="glossaryHoverText !== null" class="notice">{{ glossaryHoverText }}</span>
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
