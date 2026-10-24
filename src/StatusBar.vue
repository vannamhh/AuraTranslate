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
import { computed, onBeforeUnmount, onMounted, ref, toRaw, watch } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
// 🔵 Story 4.12, Phase 3a (Decision 2) — điểm vào ngăn kéo Tra cứu. `lookupHasRetreated` là
// một `ref` module-level ở `layout/lookupDrawerState.ts`, đẩy bởi `WorkspaceDock.vue::
// applyTier` ĐỒNG BỘ qua `syncLayoutTier` (sửa 2026-09-23 — bản đầu thăm dò DOM mỗi 200ms;
// xem doc-comment đầu tệp đó). `StatusBar` chỉ ĐỌC nó qua `v-if`, không còn vòng đời nào để
// bắt đầu/dừng ở đây.
import { lookupHasRetreated, workspaceLayoutTier } from './layout/lookupDrawerState'
import { isSegmentConfirmed } from './config/segment'
import { EDITOR_HARD_CAP_MS, EDITOR_IDLE_MS } from './panels/editorFlush'
import { isMeaningKind } from './config/proofread'
import { shortcutRows } from './config/shortcutsState'
import { libraryChapters } from './modes/libraryChapters'
import { currentMode } from './modes/modeState'
import { readingMarksWalkProgress } from './modes/readingState'
import { groupedLookup, sensesByLayer } from './panels/lookupPanelState'
import { proofreadResults, proofreadStateValue } from './proofreadState'
import { tmFuzzyThresholdInUse } from './tmSettingsState'
import { refreshWorkspaceCounts, workspaceGlossaryEntryCount, workspaceTmPairCounts } from './workspaceCountsState'
import {
  editorCaretSegmentId,
  editorChapterId,
  editorConfirmNotice,
  editorHasLoaded,
  editorLastCaretSegmentId,
  editorNavNotice,
  editorTmPrefillHiddenCount,
  editorLastSavedAt,
  editorRegroupError,
  editorRegroupNotice,
  editorSegments,
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
 * với lượt `v-for` trên 9.850 `<span>` mà `deferred-work.md §*Deferred from: A4 — cỡ chữ vỏ giao diện, và một đính chính về độ ổn định của bộ e2e (2026-08-12)*` đã ghi cho Panel Editor.
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
 * Permanent information of the status bar. It sits outside the `role="status"` region so a
 * caret move or a counter change is never announced; an unknown value is `null` and its item
 * is hidden, never rendered as 0.
 */
const positionInChapter = computed<number | null>(() => {
  const segments = toRaw(editorSegments.value)
  const id = editorCaretSegmentId.value ?? editorLastCaretSegmentId.value
  if (id === null) return null
  const index = segments.findIndex((s) => s.id === id)
  return index < 0 ? null : index + 1
})

const chapterLabel = computed<string | null>(() => {
  const id = editorChapterId.value
  const row = libraryChapters.value.find((chapter) => chapter.chapter_id === id)
  return row === undefined ? null : t('mode.library.chapter_untitled', { ord: String(row.ord) })
})

const positionText = computed<string | null>(() => {
  if (!editorHasLoaded()) return null
  const n = positionInChapter.value
  if (n === null) return null
  const total = String(editorSegments.value.length)
  const label = chapterLabel.value
  return label === null
    ? t('status.position_sentence_only', { n: String(n), total })
    : t('status.position', { chapter: label, n: String(n), total })
})

const confirmedCounterText = computed<string | null>(() => {
  if (!editorHasLoaded()) return null
  let counted = 0
  let confirmed = 0
  for (const s of toRaw(editorSegments.value)) {
    if (s.is_omitted) continue
    counted += 1
    if (isSegmentConfirmed(s)) confirmed += 1
  }
  if (counted === 0) return null
  return t('status.confirmed_counter', { confirmed: String(confirmed), total: String(counted) })
})

const shortcutHintText = computed<string | null>(() => {
  const chordOf = (id: string): string =>
    shortcutRows.value.find((row) => row.id === id)?.display.join(' / ') ?? ''
  const merge = chordOf('editor.merge_segments')
  const split = chordOf('editor.split_segment')
  if (merge === '' || split === '') return null
  return t('status.regroup_hint', { merge_chord: merge, split_chord: split })
})

const proofreadCountsText = computed<string | null>(() => {
  if (proofreadStateValue.value !== 'done' || !editorHasLoaded()) return null
  let spelling = 0
  let meaning = 0
  for (const entry of proofreadResults.value.values()) {
    for (const finding of entry.findings) {
      if (isMeaningKind(finding.kind)) meaning += 1
      else spelling += 1
    }
  }
  return t('status.proofread_counts', { spelling_count: String(spelling), meaning_count: String(meaning) })
})

const glossaryCountText = computed<string | null>(() => {
  const count = workspaceGlossaryEntryCount.value
  return count === null ? null : t('status.glossary_count', { count: String(count) })
})

const tmCountText = computed<string | null>(() => {
  const counts = workspaceTmPairCounts.value
  if (counts === null) return null
  return t('status.tm_summary', {
    total: String(counts.total),
    mine: String(counts.mine),
    others: String(counts.others),
    threshold: String(tmFuzzyThresholdInUse.value),
  })
})

// WKWebView does not focus a <button> on mouse click; the overlay would then save the wrong
// opener and UX-DR17 would return focus elsewhere.
function focusOnPointerDown(event: MouseEvent): void {
  const target = event.currentTarget
  if (target instanceof HTMLElement) target.focus()
}

const showAutoSave = computed(() => workspaceLayoutTier.value === 'full')

const autoSaveHintText = computed(() =>
  t('status.autosave_hint', {
    idle_seconds: String(EDITOR_IDLE_MS / 1000),
    cap_seconds: String(EDITOR_HARD_CAP_MS / 1000),
  }),
)

const lookupLineText = computed<string | null>(() => {
  const group = groupedLookup.value?.groups[0]
  if (group === undefined) return null
  const entryIds = new Set(group.entries.map((hit) => hit.entry_id))
  const sense = (sensesByLayer.value[group.layer] ?? []).find((s) => entryIds.has(s.entry_id))
  return sense === undefined ? null : sense.gloss
})

const lookupChordText = computed<string>(
  () => shortcutRows.value.find((row) => row.id === 'layout.lookup_drawer_open')?.display.join(' / ') ?? '',
)

onMounted(() => {
  void refreshWorkspaceCounts()
})

watch(editorChapterId, () => {
  void refreshWorkspaceCounts()
})

/**
 * *"N giây trước"* — **tính và định dạng ở frontend**, Rust không trả về một chữ nào (AC10).
 *
 * AD-21 (`ARCHITECTURE-SPINE.md`): Rust không bao giờ trả văn bản hiển thị.
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
  // 🔴 Khoá thứ MƯỜI MỘT. `Record` đóng lại làm đúng việc nó được dựng để làm: thêm
  //    `'confirm-in-flight'` vào `NavNotice` mà chưa sửa đây thì `vue-tsc` đỏ.
  'confirm-in-flight': 'panel.grid.nav_confirm_in_flight',
  'chapter-still-dirty': 'panel.grid.nav_chapter_still_dirty',
  'tm-prefill-skipped': 'panel.grid.nav_tm_prefill_skipped',
  'tm-prefill-hidden': 'panel.grid.nav_tm_prefill_hidden',
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
 * diện — một khoá LÀM TIỀN TỐ, dữ liệu ghép sau. `glossary.quick_add.translation_label`
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
    `role="status"` chỉ bọc chuỗi câu báo (nhánh cuối là mốc "Đã lưu N giây trước"), không bọc
    cả thanh: vị trí câu và bộ đếm đổi theo từng cú gõ, và một vùng sống bọc chúng sẽ bắt trình
    đọc màn hình đọc lại mỗi lần. `aria-live` để mặc định (`polite`).
  -->
  <footer class="status">
    <div class="status-info" data-status-info>
      <!-- aura-allow-text: KẾT QUẢ của `t()` (chuỗi đã qua `vi.json`). -->
      <span v-if="positionText !== null" class="info-item" data-status-position>{{ positionText }}</span>
      <!-- aura-allow-text: KẾT QUẢ của `t()` (chuỗi đã qua `vi.json`). -->
      <span v-if="confirmedCounterText !== null" class="info-item" data-status-confirmed>{{ confirmedCounterText }}</span>
      <!-- aura-allow-text: KẾT QUẢ của `t()` (chuỗi đã qua `vi.json`). -->
      <span v-if="shortcutHintText !== null" class="info-item" data-status-hint>{{ shortcutHintText }}</span>
      <!-- aura-allow-text: KẾT QUẢ của `t()` (chuỗi đã qua `vi.json`). -->
      <span v-if="proofreadCountsText !== null" class="info-item" data-status-proofread>{{ proofreadCountsText }}</span>
    </div>
    <div class="status-live" role="status" data-status-live>
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
    <span v-else-if="navNoticeKey !== null" class="notice">{{ t(navNoticeKey, { count: String(editorTmPrefillHiddenCount) }) }}</span>
    <!--
      aura-allow-text: KẾT QUẢ của `t()` cộng (ca đã chốt) một chuỗi DỮ LIỆU của Tác phẩm —
      bản dịch một thuật ngữ Glossary do người dùng chốt. 🔵 Story 3.4b — nhánh THỨ NĂM, dưới
      ba thông báo khẩn hơn và TRÊN mốc "Đã lưu" (thứ tự ưu tiên của story: lỗi xác nhận >
      gộp/tách > điều hướng > bản dịch thuật ngữ > "Đã lưu N giây trước").

      role="status" của `<footer>` xướng câu này qua trình đọc màn hình khi nó xuất hiện —
      I/O Matrix: "Thanh hiện bản dịch của thuật ngữ, xướng qua role=status".
    -->
    <span v-else-if="glossaryHoverText !== null" class="notice">{{ glossaryHoverText }}</span>
    <!-- aura-allow-text: KẾT QUẢ của `t()`. -->
    <span v-else-if="secondsSinceSave !== null" class="saved" data-status-saved>{{
      t('status.saved_seconds_ago', { seconds: String(secondsSinceSave) })
    }}</span>
    </div>
    <div class="status-end" data-status-end>
      <template v-if="!lookupHasRetreated">
        <span class="info-item" data-status-lookup-offline>{{ t('status.lookup_offline') }}</span>
        <!-- aura-allow-text: KẾT QUẢ của `t()` (chuỗi đã qua `vi.json`). -->
        <span v-if="glossaryCountText !== null" class="info-item" data-status-glossary>{{ glossaryCountText }}</span>
        <!-- aura-allow-text: KẾT QUẢ của `t()` (chuỗi đã qua `vi.json`). -->
        <span v-if="tmCountText !== null" class="info-item" data-status-tm>{{ tmCountText }}</span>
        <!-- aura-allow-text: KẾT QUẢ của `t()` (chuỗi đã qua `vi.json`). -->
        <span v-if="showAutoSave" class="info-item" data-status-autosave-hint>{{ autoSaveHintText }}</span>
      </template>
      <!-- aura-allow-text: nghĩa đầu của nguồn đầu — DỮ LIỆU từ điển, không chuỗi giao diện. -->
      <span v-else-if="lookupLineText !== null" class="info-item lookup-line" data-status-lookup-line>{{ lookupLineText }}</span>
      <!-- aura-allow-text: KẾT QUẢ của `t()` (chuỗi đã qua `vi.json`). -->
      <span v-if="lookupHasRetreated && lookupChordText !== ''" class="info-item" data-status-lookup-chord>{{ lookupChordText }}</span>
    <!--
      🔵 Story 4.12, Phase 3a (Decision 2) — điểm vào ngăn kéo Tra cứu.

      🔴 KHÔNG một nhánh của chuỗi `v-if`/`v-else-if` ở trên: nó không phải một THÔNG BÁO
      tranh chỗ với "Đã lưu N giây trước" và bốn nhóm câu khẩn hơn — nó là một LỐI VÀO,
      phải còn bấm được bất kể thanh đang nói câu gì khác. `margin-left: auto` đẩy nó về
      cạnh phải, tách khỏi luồng của năm nhánh câu (xem `.lookup-entry` ở style).

      `data-lookup-drawer-open` là mối nối UX-DR17 mà `LookupDrawer.vue` đọc lại để trả tiêu
      điểm về đây lúc đóng — một thuộc tính `data-`, không một tên lớp CSS (tên lớp là trình
      bày, đổi được tự do).
    -->
    <button
      v-if="currentMode === 'workspace' && readingMarksWalkProgress !== null"
      type="button"
      class="walk-entry"
      data-reading-walk-next
      @mousedown="focusOnPointerDown($event)"
      @click="dispatch('reading.marks_walk_next')"
    >
      {{ t('status.reading_walk_next', { next: String(readingMarksWalkProgress.next), total: String(readingMarksWalkProgress.total) }) }}
    </button>
    <button
      type="button"
      class="queue-entry"
      data-glossary-queue-open
      @mousedown="focusOnPointerDown($event)"
      @click="dispatch('glossary.queue.open')"
    >
      {{ t('command.glossary.queue.open') }}
    </button>
    <button
      v-if="lookupHasRetreated"
      type="button"
      class="lookup-entry"
      data-lookup-drawer-open
      @click="dispatch('layout.lookup_drawer_open')"
    >
      {{ t('command.layout.lookup_drawer_open') }}
    </button>
    </div>
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
 * `flex: none` is required: `.stage` is the flex:1 column holding .modeport and these strips,
 * and without it the bar shrinks when the work area claims space.
 */
.status {
  display: flex;
  align-items: center;
  flex: none;
  height: var(--space-status-height);
  gap: var(--space-panel-inline);
  padding: 0 var(--space-panel-inline);
  border-top: 1px solid var(--color-outline);
  background: var(--color-surface);
}

/*
 * Permanent items truncate with an ellipsis before they can push the live notice out; the
 * notice keeps its own width, so the row never overflows the 34px bar.
 */
.status-info,
.status-end {
  display: flex;
  align-items: center;
  gap: var(--space-panel-inline);
  min-width: 0;
  white-space: nowrap;
}

.status-info {
  flex: 0 1 auto;
  overflow: hidden;
}

.status-live {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-end {
  flex: 0 1 auto;
  margin-left: auto;
}

.info-item {
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.lookup-line {
  max-width: 40ch;
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

/*
 * 🔵 Story 4.12, Phase 3a — điểm vào ngăn kéo Tra cứu. `margin-left: auto` trên một flex
 * container đẩy nó về cạnh PHẢI bất kể nhánh câu nào (hoặc không nhánh nào) đang chiếm phần
 * còn lại — cùng cơ chế `justify-content: space-between` mà `.attr-head`/`.sc-head` dùng,
 * chỉ viết ở phía item thay vì ở container vì `.status` còn năm anh em khác không cần dời.
 *
 * Không một vai chữ RIÊNG: `ui-sm` là vai của `.saved`/`.notice` ngay trên — một nút chữ
 * cùng cỡ với phần còn lại của thanh, không phải một huy hiệu nổi bật hơn (Kiểm F của
 * `check:tokens` cấm bóng đổ; nổi bật bằng token màu KHÔNG bằng elevation).
 */
.lookup-entry,
.walk-entry,
.queue-entry {
  flex: none;
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}
</style>
