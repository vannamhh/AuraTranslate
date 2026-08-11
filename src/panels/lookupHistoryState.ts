/**
 * State của **tab Lịch sử** — lịch sử tra trong phiên, bộ ghim đã nạp, và tab đang chọn.
 * Story 1.20, AC1 · AC2 · AC3 · AC4 · AC5 · AC7 · AC11 · AC12.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY, KHÔNG TRONG `LookupPanel.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `lookupPanelState.ts`/`dictSourcesState.ts`: một lượt đổi preset bố cục gọi
 * `api.clear()` rồi dựng lại **cả bốn** panel, và **chỉ** state module-level (singleton
 * của tiến trình) sống sót qua lượt tháo/dựng đó. Một `ref` cục bộ trong `<script setup>`
 * làm tab tự nhảy về mặc định mỗi lần người dùng đổi bố cục — AC5 vỡ về cơ học.
 *
 * ⚠️ Cùng lý do, tệp này KHÔNG được `import` vào `src/commands/index.ts` trực tiếp — nó
 * dùng `ref` của Vue **và** gọi `@tauri-apps/api` xuyên qua `config/pinned.ts`, mà Kiểm
 * C/D/E của `npm run check:commands` nạp tệp đó bằng Node thuần. Ba handler đi vào bằng
 * **tiêm** qua `CommandDeps` ở `src/main.ts` — cùng cửa `selectSourceTab`/`runLookup`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 LỊCH SỬ SỐNG Ở RAM, KHÔNG ĐI QUA `core::store` — Quyết định #3
 * ─────────────────────────────────────────────────────────────────────────────
 * AC4 đòi lịch sử **kết thúc khi đóng ứng dụng**. State module-level thoả mệnh đề đó về
 * **cơ học**: không cần một dòng mã xoá nào lúc thoát, và một lượt ghi rồi xoá lúc thoát
 * là hai chỗ để sai. AD-1 cho phép — frontend giữ *"state UI"*, và một danh sách thao tác
 * trong phiên không phải một quy tắc nghiệp vụ.
 *
 * `localStorage` **bị cấm tường minh** (Story 1.19 Quyết định #1c, AD-1 + FR103).
 *
 * ⇒ **0 IPC command mới cho lịch sử.** Ba command IPC của story này phục vụ riêng bộ ghim,
 * thứ AC3 đòi sống qua các phiên.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 GHIM Ở `global.db`, PHẠM VI TOÀN ỨNG DỤNG — Ice ký lại 2026-08-11
 * ─────────────────────────────────────────────────────────────────────────────
 * Quyết định #1 ngày 2026-08-10 chốt `project.db` (phạm vi Tác phẩm). Một phép đo hôm sau
 * lật nó: **không tồn tại đường mở lại một `.atproj` từ đĩa** — `OpenWorkState` khởi động
 * với `null` và chỉ `create_work_*` đặt được giá trị vào đó. ⇒ với ghim ở `project.db`,
 * đóng app rồi mở lại là không Tác phẩm nào đang mở, nên **AC3 không bao giờ đúng trên màn
 * hình** dù dữ liệu vẫn nằm trên đĩa.
 *
 * Ba hệ quả trực tiếp lên tệp này:
 * 1. **Đổi Tác phẩm KHÔNG đụng bộ ghim** — nó không thuộc về Tác phẩm nào. [`resetLookupHistory`]
 *    chỉ vứt **lịch sử**, và một lượt nạp lại bộ ghim ở đó là một vòng IPC thừa cộng một
 *    khoảng nháy không có lý do.
 * 2. **Không còn trạng thái *"chưa mở Tác phẩm nào"*** — câu đó **sai** ở đây. Bẫy 4 từ
 *    bốn trạng thái xuống **ba**.
 * 3. Lượt nạp duy nhất là **một lần lúc khởi động**, từ `src/main.ts`.
 */
import { computed, readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { listPinnedEntries, pinEntry, unpinEntry } from '../config/pinned'
import type { PinnedEntry } from '../config/pinned'
import type { LookupResponse } from '../config/dict'
import type { IpcError } from '../i18n'

/** Hai tab của thân Panel Lookup — Quyết định #4 (KHÔNG có `Concordance`, FR64/Story 7.7). */
export type LookupTab = 'record' | 'history'

/**
 * Một mục từ có thể ghim — **cùng bốn trường** mà `pinned_entry` lưu.
 *
 * ⚠️ `headword` là đầu mục THẬT trong từ điển (`EntryHit.headword`), không truy vấn người
 * dùng bôi đen: hai thứ đó khác nhau ở đúng ca `headword_simp` khớp (tra `国` giản thể mà
 * đầu mục ghi `國` phồn thể), và ghim truy vấn ở đó là ghim SAI CHỮ.
 */
export type PinTarget = {
  source_code: string
  entry_id: number
  headword: string
  gloss: string | null
}

/**
 * Một hàng lịch sử — **một truy vấn**, không một đầu mục.
 *
 * 🔴 `keys` là tập `source_code:entry_id` mà lượt tra này lấy về, và nó tồn tại cho đúng
 * một việc: hàng GHIM đọc số lần tra của phiên qua nó (§Dev Notes ⑨ — *"ghim và lịch sử
 * khớp nhau qua `(source_code, entry_id)`"*). Không có nó, số đếm trên hàng ghim phải so
 * `headword` với `query` bằng chuỗi, và hai thứ đó lệch nhau ở đúng ca giản/phồn thể.
 */
export type LookupHistoryEntry = {
  /** Truy vấn đã tra, đã `trim()`. Khoá dedupe của AC7. */
  query: string
  /** Nghĩa ĐẦU TIÊN lấy về, `null` khi không nguồn nào có mục từ này. */
  gloss: string | null
  /** Số lần tra trong PHIÊN này (AC7). */
  count: number
  /** Mốc lượt tra GẦN NHẤT, `Date.now()`. Hàng hiện thời gian tương đối. */
  at: number
  /** `source_code:entry_id` của mọi đầu mục lượt tra này lấy về. */
  keys: readonly string[]
}

/**
 * Trần số hàng lịch sử của một phiên. **200.**
 *
 * 🔴 Không một AC nào đòi con số này, và đó chính là lý do nó phải có tên: AC7 (dedupe)
 * chặn *"một Chương sinh ra hàng trăm dòng giống nhau"*, nó **không** chặn hàng trăm dòng
 * KHÁC nhau — một Chương dài có thừa từ khác nhau để tra. Không trần thì danh sách lớn
 * đơn điệu suốt phiên, và mỗi lượt tra render lại toàn bộ nó.
 *
 * ⚠️ Cắt ở ĐUÔI (cũ nhất), không ở đầu: AC1 đòi *"gần nhất trước"*, nên thứ đáng mất là
 * thứ xa nhất. Ghi vào `deferred-work.md` — nếu người dùng thật đòi một trần khác thì đó
 * là một con số đo được, không một hằng đoán trong đầu.
 */
const HISTORY_CEILING = 200

/** Khoá `vi.json` cho ca *"bấm ghim mà chưa nhắm được mục nào"* — §Dev Notes ⑧ ca 3. */
const KEY_PIN_NO_TARGET = 'panel.lookup.pin_no_target'

/** `source_code:entry_id` — **một** cách dựng khoá, dùng ở mọi chỗ so khớp. */
export function entryKey(sourceCode: string, entryId: number): string {
  return `${sourceCode}:${entryId}`
}

const tab = ref<LookupTab>('record')
const history = ref<readonly LookupHistoryEntry[]>([])

/**
 * Bộ ghim đọc từ đĩa. **`null` ⇔ chưa có câu trả lời nào** — khác hẳn `[]` (*đã hỏi, chưa
 * ghim gì*). Bẫy 4 sống hay chết ở đúng khác biệt này.
 */
const pinnedRaw = shallowRef<readonly PinnedEntry[] | null>(null)
const pinnedErr = shallowRef<IpcError | null>(null)

/** Lỗi của lượt GHI gần nhất — không phải lỗi của lượt đọc, và hai thứ nói hai câu khác. */
const pinWriteErr = shallowRef<IpcError | null>(null)

/** Khoá `vi.json` của lời nhắc *"chưa nhắm được mục nào"*, hoặc `null`. */
const pinNotice = shallowRef<string | null>(null)

/**
 * Mục từ của **lượt tra đang hiện**, phẳng theo `(source_code, entry_id)`.
 *
 * ⚠️ Không `ref`: không bề mặt nào render nó: nó chỉ là bảng tra cho [`toggleLookupPin`]
 * tại thời điểm chạy, và làm nó phản ứng chỉ thêm một lượt cập nhật thừa mỗi lượt tra.
 */
let currentTargets: readonly PinTarget[] = []

/**
 * Mục từ vừa bị **bấm chuột**, giữ đúng một lượt.
 *
 * 🔴 **Vì sao không chỉ đọc `document.activeElement`:** WKWebView — engine Tauri dùng trên
 * macOS (NFR14) — **không đặt tiêu điểm cho `<button>` khi bấm chuột**. Bài học đã đo ở
 * `dictSourcesState.ts` (Story 1.19); đọc nó trước khi tự chế cách khác.
 *
 * ⚠️ **Tiêu thụ MỘT LẦN** rồi về `null`: lượt bấm phím kế tiếp có thể ở một ngữ cảnh hoàn
 * toàn khác, và một giá trị cũ sống sót tới đó là ghim NHẦM mục.
 */
let aimedKey: string | null = null

/** Số thứ tự lượt nạp bộ ghim — chỉ lượt MỚI NHẤT được quyền ghi vào state. */
let loadSequence = 0

/**
 * Số thứ tự **ngữ cảnh Tác phẩm** — tăng mỗi lượt [`resetLookupHistory`].
 *
 * 🔴 Rào cho một lượt ghi ghim ĐANG BAY, và nó chỉ rào **một** thứ: `pinWriteErr`. Một
 * lượt ghi trượt thuộc về cú bấm đã phát nó, tức thuộc về Tác phẩm đang mở **lúc đó**;
 * để nó dựng lại banner lỗi sau khi Ice đã đổi sang Tác phẩm khác là đặt một câu đúng
 * vào một màn hình nó không còn nói về — cùng lớp lỗi mà `sequence` của
 * `lookupPanelState.ts` tồn tại để chặn (bắt ở code review 2026-08-07).
 *
 * ⚠️ **`pinnedRaw` thì KHÔNG rào, và đó là chủ ý:** từ 2026-08-11 mục ghim sống ở
 * `global.db`, tức nó **không** thuộc Tác phẩm nào. Một danh sách ghim mới về sau lượt
 * đổi Tác phẩm vẫn là sự thật của đĩa, và vứt nó đi là tự tay dựng một khoảng `null` mà
 * tab ghim phải câm suốt trong đó — không lý do gì.
 */
let workSequence = 0

/** Tab đang chọn ở thân Panel Lookup (AC5). */
export const lookupTab: DeepReadonly<Ref<LookupTab>> = readonly(tab)

/** Lịch sử tra của phiên, **gần nhất trước** (AC1). */
export const lookupHistory: DeepReadonly<Ref<readonly LookupHistoryEntry[]>> = readonly(history)

/** Bộ ghim đã nạp. Rỗng khi chưa có câu trả lời — đọc các vị từ dưới đây để biết VÌ SAO. */
export const pinnedEntries = computed<readonly PinnedEntry[]>(() => pinnedRaw.value ?? [])

/**
 * 🔴 **BA trạng thái, BA vị từ RIÊNG** (Bẫy 4) — không một chuỗi `if/else` trong
 * `<template>` được phép gộp chúng, và không vị từ nào ở đây trả lời câu hỏi của vị từ kia.
 *
 * | Trạng thái | Vị từ | Câu phải nói |
 * |---|---|---|
 * | Nạp trượt (lỗi kho) | [`pinnedLoadFailed`] | `panel.lookup.pinned_load_failed` |
 * | Đã hỏi, chưa ghim gì | [`pinnedIsEmpty`] | `panel.lookup.pinned_empty_*` |
 * | Chưa có câu trả lời | cả hai `false` | **không nói gì** — không nháy sang "chưa ghim" |
 *
 * ⚠️ **BỐN xuống BA** ở lượt Ice ký lại 2026-08-11: trạng thái *"chưa mở Tác phẩm nào"*
 * biến mất cùng với phạm vi Tác phẩm. Ô thứ ba vẫn là ô khó nhất — nó là ô **duy nhất**
 * không có câu nào để đọc, nên nó chỉ nghiệm thu được bằng **vắng mặt**.
 */
export const pinnedLoadFailed = computed(() => pinnedErr.value !== null)

/** Đã hỏi, và chưa ghim mục nào. */
export const pinnedIsEmpty = computed(
  () => pinnedErr.value === null && pinnedRaw.value !== null && pinnedRaw.value.length === 0,
)

/** Chưa tra gì trong phiên này (AC8 vế lịch sử). */
export const historyIsEmpty = computed(() => history.value.length === 0)

/**
 * Lỗi của lượt GHIM/BỎ GHIM gần nhất — hiện bằng `tError()`.
 *
 * 🔴 **Không** gộp vào [`pinnedLoadFailed`]: một lượt ghi trượt **không** làm danh sách
 * đang hiện sai, nên đọc nó thành *"không đọc được danh sách đã ghim"* là nói dối theo
 * đúng hướng ngược lại. Bài học `lookupError` của Story 1.17 áp cả hai chiều.
 */
export const pinWriteError: DeepReadonly<Ref<IpcError | null>> = readonly(pinWriteErr)

/** Khoá `vi.json` của lời nhắc *"chưa nhắm được mục nào để ghim"*, hoặc `null` (§⑧ ca 3). */
export const pinNoticeKey: DeepReadonly<Ref<string | null>> = readonly(pinNotice)

/** Một mục từ có đang được ghim không — dải nút ghim đọc để vẽ đúng nhãn. */
export function entryIsPinned(sourceCode: string, entryId: number): boolean {
  const key = entryKey(sourceCode, entryId)
  return pinnedEntries.value.some((p) => entryKey(p.source_code, p.entry_id) === key)
}

/**
 * Khoá `vi.json` cho nhãn nút ghim — **một hàm chứ không một toán tử ba ngôi trong
 * `<template>`**: Kiểm A2 của `check-i18n.mjs` đòi mọi mustache là **một** lời gọi `t()`,
 * và `a ? t(x) : t(y)` không phải — nó là hai lời gọi trong một biểu thức, và cổng đọc
 * TĨNH. Cùng khuôn `layerKeyFor` của Story 1.19.
 */
export function pinLabelKey(pinned: boolean): string {
  return pinned ? 'panel.lookup.unpin' : 'panel.lookup.pin'
}

/**
 * Số lần tra **của phiên này** cho một mục ghim (§Dev Notes ⑨).
 *
 * 🔴 Số đếm **không** bền vững, và đó là một quyết định có số đứng sau: một cột
 * `lookup_count` cần một lượt `Store::write` **mỗi lượt tra** — tức mỗi lần bôi đen chữ —
 * đưa một lượt ghi đĩa vào đúng đường nóng của Auto-Lookup và cho nó cạnh tranh hàng đợi
 * ghi nối tiếp với auto-save Editor (NFR2, AD-11/AD-12). Không AC nào đòi số đếm sống qua
 * phiên; AC3 chỉ đòi **mục ghim** còn.
 *
 * ⇒ mở app mới thì đếm về 0 và hàng ghim vẫn còn. Hàm THUẦN theo state, test được bằng mắt
 * ở bàn đo hàng 3.
 */
export function sessionLookupCount(sourceCode: string, entryId: number): number {
  const key = entryKey(sourceCode, entryId)
  return history.value.reduce((sum, row) => (row.keys.includes(key) ? sum + row.count : sum), 0)
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 THỜI GIAN TƯƠNG ĐỐI — hai hàm THUẦN, không một `Intl.RelativeTimeFormat`
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ `Intl.RelativeTimeFormat` viết ra chuỗi HIỂN THỊ ở tầng webview, tức một câu tiếng
// Việt sinh ngoài `vi.json` — NFR16 nói mọi văn bản giao diện sống ở đó và CHỈ ở đó, và
// Kiểm A2 của cổng cưỡng chế đúng mệnh đề ấy. Hai hàm dưới đây trả về một **khoá** và một
// **tham số**; câu thì `vi.json` viết.

const MINUTE_MS = 60_000
const HOUR_MS = 60 * MINUTE_MS

/** Khoá `vi.json` cho mốc thời gian `at` (epoch ms), so với `now`. Hàm THUẦN. */
export function relativeTimeKey(at: number, now: number = Date.now()): string {
  const delta = Math.max(0, now - at)
  if (delta < MINUTE_MS) return 'panel.lookup.time_just_now'
  if (delta < HOUR_MS) return 'panel.lookup.time_minutes'
  return 'panel.lookup.time_hours'
}

/** Tham số của khoá mà [`relativeTimeKey`] trả về. Hàm THUẦN. */
export function relativeTimeParams(at: number, now: number = Date.now()): Record<string, string> {
  const delta = Math.max(0, now - at)
  if (delta < MINUTE_MS) return {}
  if (delta < HOUR_MS) return { n: String(Math.floor(delta / MINUTE_MS)) }
  return { n: String(Math.floor(delta / HOUR_MS)) }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Tab
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Handler thật của `lookup.select_tab_record`/`lookup.select_tab_history` (AC5, AC6).
 *
 * ⚠️ HAI command CHỌN tab, không MỘT command đổi/toggle — cùng khuôn
 * `source.select_tab_*`: bấm đúng tab đang chọn là một thao tác **VÔ HẠI** (idempotent),
 * không lật sang tab kia.
 */
export function selectLookupTab(next: LookupTab): void {
  tab.value = next
}

// ═════════════════════════════════════════════════════════════════════════════════
// Lịch sử
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * 🔴 **ĐIỂM GHI LỊCH SỬ DUY NHẤT** — gọi từ cuối `lookupPanelState.ts::runLookup`, **sau**
 * guard `mine !== sequence` (AC11).
 *
 * Ba nguồn độc lập đã ghi tên story này bằng chữ rằng đây là điểm nghẽn duy nhất:
 * `src/main.ts` (*"Story 1.20 chỉ có một chỗ để cắm vào"*), `dictSourcesState.ts:171`
 * (*"đi **qua** điểm nghẽn của Story 1.17 chứ không **quanh** nó"*), và story 1.18
 * (*"1.20 chỉ được phép **thêm một dòng ghi** vào chỗ đó"*).
 *
 * 🔴 **Gọi khi `err === null`, KỂ CẢ khi `groups` rỗng.** *"Đã tra mà không thấy"* là một
 * sự kiện thật đáng nhớ, và mockup có một hàng lịch sử cho đúng ca đó. **Không** gọi khi
 * `err !== null`: *"không tra được"* khác *"tra mà không thấy"*, và trộn hai thứ là đúng
 * bẫy `??` mà Story 1.17 đã bắt.
 *
 * 🔴 **AC7 — dedupe:** truy vấn đã có ⇒ hàng đó **lên đầu** và `count + 1`, **không** một
 * hàng thứ hai. Nếu không, một Chương sinh ra hàng trăm dòng giống nhau.
 */
export function recordLookup(query: string, response: LookupResponse): void {
  currentTargets = targetsOf(response)
  aimedKey = null

  // ⚠️ Một lượt tra MỚI trả lời chính câu mà `pin_no_target` vừa hỏi (*"chạm vào một mục
  // trong kết quả tra cứu trước"*), nên giữ lời nhắc đó lại là để một câu đã hết đúng nằm
  // trên màn hình. Cùng luật cho lỗi ghi: nó thuộc về lượt bấm trước, không lượt tra này.
  pinNotice.value = null
  pinWriteErr.value = null

  const keys = currentTargets.map((tgt) => entryKey(tgt.source_code, tgt.entry_id))
  const gloss = currentTargets.find((tgt) => tgt.gloss !== null)?.gloss ?? null
  const at = Date.now()

  const previous = history.value.find((row) => row.query === query)
  const rest = history.value.filter((row) => row.query !== query)

  const row: LookupHistoryEntry = {
    query,
    // ⚠️ Nghĩa MỚI thắng nghĩa cũ: một lượt tra lại sau khi bật lại một nguồn phải hiện
    // đúng thứ nguồn đó vừa trả về, không ảnh chụp của lượt trước.
    gloss,
    count: (previous?.count ?? 0) + 1,
    at,
    keys,
  }

  history.value = [row, ...rest].slice(0, HISTORY_CEILING)
}

/** Handler thật của `lookup.clear_history` — vứt lịch sử của phiên, **giữ nguyên** bộ ghim. */
export function clearLookupHistory(): void {
  history.value = []
}

/**
 * 🔴 **Vứt lịch sử — và KHÔNG đụng bộ ghim.** Gọi từ `resetLookupPanel()`, tức khi Tác
 * phẩm đang mở BỊ THAY (AC12 vế thứ nhất).
 *
 * ⚠️ **Vế thứ hai của AC12 tự rụng ở lượt Ice ký lại 2026-08-11.** Bản đầu nạp lại bộ ghim
 * ở đây vì ghim thuộc phạm vi Tác phẩm; nay nó thuộc phạm vi **ứng dụng**, nên một lượt
 * nạp lại là một vòng IPC thừa cho cùng một sự thật — cộng một khoảng `pinnedRaw === null`
 * mà tab ghim phải câm suốt trong đó, tức một lượt nháy không có lý do gì để tồn tại.
 *
 * ⚠️ `currentTargets` và `aimedKey` **vẫn** bị vứt: chúng trỏ vào kết quả của lượt tra
 * thuộc Tác phẩm cũ, mà `resetLookupPanel()` vừa xoá. Giữ chúng là để `lookup.toggle_pin`
 * ghim một mục từ của một lượt tra không còn trên màn hình.
 */
export function resetLookupHistory(): void {
  // 🔴 Vô hiệu hoá vế BÁO LỖI của mọi lượt ghi ghim đang bay — xem [`workSequence`].
  workSequence += 1

  history.value = []
  currentTargets = []
  aimedKey = null
  pinNotice.value = null
  pinWriteErr.value = null
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bộ ghim
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Nạp bộ ghim. Gọi **một lần** lúc khởi động, từ `src/main.ts`.
 *
 * ⚠️ Số thứ tự lượt nạp giữ lại dù hôm nay chỉ có **một** chỗ gọi, và đó là một quyết định
 * chứ không phải mã thừa: Story 1.21 (màn hình gán phím) và bất kỳ story nào thêm một
 * đường nạp lại đều mở ra ca *"hai lượt bay cùng lúc, lượt cũ về sau"*. Cùng cơ chế và
 * cùng lý do với `sequence` của `lookupPanelState.ts`, và ở đó nó đã là một lỗi THẬT bắt
 * được ở code review 2026-08-07.
 */
export async function loadPinnedEntries(): Promise<void> {
  const mine = ++loadSequence
  const { entries, error } = await listPinnedEntries()
  if (mine !== loadSequence) return

  pinnedErr.value = error
  // ⚠️ `entries === null` VÀ `error === null` là ca *"không có cầu IPC"* (`npm run dev`
  // trong một trình duyệt thường). Giữ `null` — bốn vị từ ở trên đều `false`, và tab nói
  // ĐÚNG một thứ: không gì cả. Ép nó thành `[]` ở đây là dựng câu "chưa ghim mục nào" cho
  // một câu hỏi chưa ai trả lời.
  pinnedRaw.value = entries
}

/**
 * Nhắm mục từ nằm dưới một sự kiện chuột — gọi từ `@mousedown` trên **vùng chứa**.
 *
 * ⚠️ Uỷ quyền sự kiện ở vùng chứa chứ không một handler trên từng mục: một handler cho mỗi
 * mục trên một danh sách sinh động là N chỗ để sai, và `@click` của nút ghim phải ở lại
 * **đúng một** lời gọi `dispatch` (Kiểm A của `check:commands`).
 */
export function aimLookupEntryFrom(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLElement)) return
  const host = target.closest<HTMLElement>('[data-entry-key]')
  aimedKey = host?.dataset.entryKey ?? null
}

/**
 * Mục từ mà [`toggleLookupPin`] sẽ tác động — **hàm THUẦN theo state**, ba nguồn theo thứ
 * tự ưu tiên (§Dev Notes ⑧).
 *
 * 1. một mục vừa được `@mousedown` chạm (ở **cả hai** tab — hàng ghim cũng nhắm được, đó
 *    là đường bỏ ghim bằng chuột);
 * 2. chưa chạm cái nào **và** kết quả chỉ có đúng một mục ⇒ lấy mục đó;
 * 3. không xác định được ⇒ `null`, và chỗ gọi **phải nói ra** — một phím bấm không có hiệu
 *    lực và không giải thích là đúng thứ *"rỗng im lặng"* mà `ARCHITECTURE-SPINE.md:622`
 *    cấm.
 */
function resolvePinTarget(): PinTarget | null {
  const aimed = aimedKey
  aimedKey = null

  if (aimed !== null) {
    const inResults = currentTargets.find((t) => entryKey(t.source_code, t.entry_id) === aimed)
    if (inResults !== undefined) return inResults

    // Hàng ĐÃ GHIM — nguồn duy nhất khi người dùng bỏ ghim từ chính tab Lịch sử, nơi
    // `currentTargets` có thể không chứa mục đó (lượt tra hiện tại là một từ khác).
    const pinnedRow = pinnedEntries.value.find(
      (p) => entryKey(p.source_code, p.entry_id) === aimed,
    )
    if (pinnedRow !== undefined) {
      return {
        source_code: pinnedRow.source_code,
        entry_id: pinnedRow.entry_id,
        headword: pinnedRow.headword,
        gloss: pinnedRow.gloss,
      }
    }
    return null
  }

  return currentTargets.length === 1 ? currentTargets[0] : null
}

/**
 * Hàng đợi **NỐI TIẾP** cho mọi lượt ghi bộ ghim — mỗi lượt bấm móc vào đuôi lượt trước.
 *
 * 🔴 Cùng lý do và cùng bài học `dictSourcesState.ts::writeQueue` (bắt ở code review
 * 2026-08-10): mỗi lượt gọi trả về **toàn bộ** bộ ghim sau lượt ghi của nó, và hai lượt
 * `invoke` phát độc lập **không** đảm bảo về đúng thứ tự phát. Ghim A rồi ghim B ngay sau:
 * nếu lượt mang `{A}` về **sau** lượt mang `{A,B}` thì màn hình rơi về `{A}` trong khi đĩa
 * giữ `{A,B}`.
 */
let pinWriteQueue: Promise<void> = Promise.resolve()

/**
 * Handler thật của `lookup.toggle_pin` — ghim hoặc bỏ ghim mục từ **đang xem** (AC2).
 *
 * 🔴 **Không** một tham số `entry_id`, và đó là §KHÔNG-LÀM ⑤ viết thành chữ ký: một
 * command cho mỗi mục ghim phá chính cơ chế đếm tĩnh của `check-commands.mjs`
 * (`COMMAND_FLOOR`), và một id không tồn tại lúc dựng màn hình phím thì Story 1.21 không
 * gán lại được. Handler đọc mục tiêu từ trạng thái quanh nó — đúng khuôn
 * `deps.currentSelection` của `lookup.lookup_selection` và `deps.toggleDictSource` của
 * Story 1.19.
 *
 * 🔴 **Ghi NGAY, không debounce** (AD-35): một thao tác rời rạc dứt khoát của người dùng
 * không được nằm chờ trong một bộ đệm gõ rồi biến mất nếu app sập.
 */
export function toggleLookupPin(): void {
  const target = resolvePinTarget()
  if (target === null) {
    // §Dev Notes ⑧ ca 3 — nói ra bằng một câu CÓ LÝ DO, không im lặng không hiệu lực.
    pinNotice.value = KEY_PIN_NO_TARGET
    return
  }

  pinNotice.value = null
  pinWriteErr.value = null
  const mine = workSequence

  pinWriteQueue = pinWriteQueue
    .then(async () => {
      // 🔴 HƯỚNG ghim/bỏ-ghim tính **TẠI ĐÂY**, không tại lúc bấm — bắt ở code review
      // 2026-08-11. `pinnedRaw` chỉ đổi **sau** khi lượt IPC trước trả lời, nên đọc nó ở
      // ngoài hàng đợi là đọc một sự thật có thể đã cũ đúng một lượt: hai cú bấm nhanh
      // hơn một vòng IPC cùng thấy *"chưa ghim"* và cùng gửi GHIM, tức ngữ nghĩa toggle
      // vỡ im lặng dưới tay người dùng gõ nhanh. Đặt phép đọc vào trong hàng đợi làm nó
      // chạy **sau** lượt ghi trước đã cập nhật `pinnedRaw` — hàng đợi nối tiếp vốn đã
      // bảo đảm đúng thứ tự đó, chỉ là phép đọc trước đây đứng ngoài nó.
      //
      // ⚠️ Giới hạn còn lại, ghi ra thay vì giấu: trong khoảng `pinnedRaw === null` (lượt
      // nạp khởi động chưa về), vị từ trả `false` ⇒ đi nhánh GHIM. Ca đó **tự sửa** —
      // `pin_entry` là `INSERT OR IGNORE` nên nó vô hại, và lượt trả về mang danh sách
      // ĐẦY ĐỦ của đĩa, tức màn hình khớp lại ngay. Nó cũng gần như không tới được: một
      // lượt ghi chỉ enqueue được sau khi một lượt tra đã xong, mà lượt tra ấy đòi một cử
      // chỉ bôi đen của người dùng — chậm hơn vòng IPC nạp nhiều bậc.
      const removing = entryIsPinned(target.source_code, target.entry_id)

      const { entries, error } = removing
        ? await unpinEntry(target.source_code, target.entry_id)
        : await pinEntry(target.source_code, target.entry_id, target.headword, target.gloss)

      if (error !== null) {
        // 🔴 KHÔNG đổi `pinnedRaw`: danh sách đang hiện vẫn là sự thật của đĩa, vì lượt ghi
        // đã bị từ chối. Đổi nó ở đây là để màn hình nói dối theo hướng ngược lại.
        //
        // ⚠️ Câu lỗi chỉ được nói ra nếu Tác phẩm CHƯA đổi kể từ cú bấm phát ra nó — xem
        // [`workSequence`]. Nhật ký thì ghi ở mọi ca: một lượt ghi trượt vẫn là một sự
        // kiện thật, kể cả khi không còn màn hình nào để nói với.
        console.warn(`[pinned] không ghi được mục ghim (\`${error.code}\`).`)
        if (mine === workSequence) pinWriteErr.value = error
        return
      }
      if (entries !== null) pinnedRaw.value = entries
    })
    // ⚠️ Chuỗi **không được phép** ở lại trạng thái rejected: một lượt ném ngoài dự kiến sẽ
    // làm MỌI lượt ghi sau đó bị bỏ qua im lặng, tức một lỗi nhất thời khoá vĩnh viễn tính
    // năng — cùng hình dạng đã bắt ở `ensureHanVietLoaded` (cờ không bao giờ nhả).
    .catch((err: unknown) => {
      console.warn(`[pinned] hàng đợi ghi mục ghim ném ngoài dự kiến: ${String(err)}`)
    })
}

/**
 * Mọi đầu mục của một lượt tra, phẳng theo `(source_code, entry_id)` — **hàm THUẦN**.
 *
 * ⚠️ `senses_by_layer` khoá theo **LỚP**, không theo `entry_id` phẳng (`entry_id` chỉ duy
 * nhất TRONG một tệp), nên phép zip phải đi qua `group.layer` — cùng luật `sensesFor` của
 * `LookupPanel.vue`.
 */
function targetsOf(response: LookupResponse): readonly PinTarget[] {
  const out: PinTarget[] = []
  for (const group of response.grouped.groups) {
    const senses = response.senses_by_layer[group.layer] ?? []
    for (const hit of group.entries) {
      out.push({
        source_code: hit.source_code,
        entry_id: hit.entry_id,
        headword: hit.headword,
        gloss: senses.find((s) => s.entry_id === hit.entry_id)?.gloss ?? null,
      })
    }
  }
  return out
}
