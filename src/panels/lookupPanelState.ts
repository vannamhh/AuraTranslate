/**
 * State của Panel Lookup — truy vấn hiện tại, kết quả, lỗi. Story 1.17, AC6 · AC10 ·
 * Quyết định #1.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY, KHÔNG TRONG `LookupPanel.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `sourcePanelState.ts`: AC10 đòi kết quả tra cứu SỐNG SÓT qua một lượt đổi
 * preset bố cục (`applyPreset()` → `api.clear()` → dựng lại cả ba panel). State
 * module-level là singleton của cả tiến trình, nên nó sống sót qua lượt tháo/dựng lại đó.
 *
 * ⚠️ Cùng lý do, tệp này KHÔNG được `import` vào `src/commands/index.ts` trực tiếp — nó
 * dùng `ref` của Vue, và Kiểm C/D/E của `npm run check:commands` nạp tệp đó bằng Node
 * thuần. `runLookup` được TIÊM VÀO qua `CommandDeps.currentSelection` + command
 * `lookup.lookup_selection` ở `src/main.ts` — cùng cửa mà `selectSourceTab`/
 * `toggleHanVietView` đã đi qua.
 */
import { computed, readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { lookupDictionary } from '../config/dict'
import type { LookupResponse, QueryRoute, SenseRecord, SourceGroup } from '../config/dict'
// 🔴 STORY 1.20 — điểm ghi lịch sử. Chiều phụ thuộc đi MỘT hướng: tệp này biết
// `lookupHistoryState`, tệp kia KHÔNG biết tệp này. Đảo lại là một vòng import.
import { recordLookup, resetLookupHistory } from './lookupHistoryState'
import type { IpcError } from '../i18n'

/**
 * 🔴 **AC5 — vị từ "bất đồng", hàm THUẦN xuất được** (không một biểu thức chôn trong
 * `<template>`). không chỉ MỘT nguồn có kết quả ⇒ không dòng dẫn ("một nguồn không bất
 * đồng với ai") — nên vị từ chỉ cần hỏi *"có từ hai nguồn trở lên không"*: dòng dẫn không
 * phán xét nguồn nào đúng, không tóm tắt sự khác biệt — nó chỉ báo rằng có KHẢ NĂNG bất đồng
 * để người dịch tự đối chiếu các khối bên dưới, đúng mệnh đề AD-19.
 */
export function sourcesDisagree(groups: readonly SourceGroup[]): boolean {
  return groups.length >= 2
}

/**
 * 🔴 **Số nghĩa của một nguồn có phải một CẬN DƯỚI không** (AC12, Quyết định #4 §hệ quả ③).
 *
 * `total_entries !== null` ⇔ trần đã cắt lớp này ⇒ số nghĩa đếm được từ `senses_by_layer`
 * chỉ là số của **phần đã lấy về**, không số thật. Thanh nhịp phải đọc ra như một cận dưới,
 * không khẳng định — *"thanh nhịp không bao giờ khẳng định một con số nó không biết"*.
 *
 * Hàm THUẦN, xuất được, test được — cùng doctrine [`sourcesDisagree`].
 */
export function senseCountIsLowerBound(group: SourceGroup): boolean {
  return group.total_entries !== null
}

/** Một mục của thanh nhịp — một nguồn kèm số nghĩa của riêng nó. */
export type SpineSource = {
  code: string
  displayName: string
  senseCount: number
  /** `true` ⇔ [`senseCount`] là một CẬN DƯỚI (trần đã cắt nguồn này) — AC12. */
  atLeast: boolean
}

/** Một nguồn bị trần cắt SẠCH khỏi trang — có tên, có số, không có nghĩa nào lấy về. */
export type SpineHiddenSource = {
  displayName: string
  count: number
}

/** Thanh nhịp đã dẫn xuất — `N nguồn · M nghĩa` + một mục cho mỗi nguồn (Quyết định #6). */
export type Spine = {
  sourceCount: number
  senseCount: number
  /** `true` ⇔ [`senseCount`] là CẬN DƯỚI — thanh nhịp phải đọc ra như vậy (AC12). */
  senseCountAtLeast: boolean
  sources: SpineSource[]
  /** Nguồn bị giấu hẳn — FR31 đòi gọi TÊN, không chỉ đếm. */
  hidden: SpineHiddenSource[]
}

/**
 * 🔴 Thanh nhịp **dẫn xuất từ `groups`**, không một danh sách nguồn viết cứng (§KHÔNG-LÀM ②)
 * — Story 1.19 chỉ được phép THÊM một cờ vào từng mục, không dựng lại cả hàm này. Hàm THUẦN,
 * không phụ thuộc Vue.
 */
export function computeSpine(
  groups: readonly SourceGroup[],
  sensesByLayer: Readonly<Record<string, readonly SenseRecord[]>>,
  hiddenSources: readonly (readonly [string, number])[] = [],
): Spine {
  const sources = groups.map((group) => {
    const entryIds = new Set(group.entries.map((hit) => hit.entry_id))
    const senseCount = (sensesByLayer[group.layer] ?? []).filter((sense) =>
      entryIds.has(sense.entry_id),
    ).length
    return {
      code: group.source.code,
      displayName: group.source.display_name,
      senseCount,
      atLeast: senseCountIsLowerBound(group),
    }
  })
  const senseCount = sources.reduce((sum, s) => sum + s.senseCount, 0)
  return {
    // 🔴 Nguồn bị trần cắt SẠCH vẫn được ĐẾM (AC12/FR31): một thanh nhịp nói "1 nguồn"
    // trong khi có hai nguồn khớp là đúng con số sai mà §hệ quả ③ tồn tại để chặn.
    sourceCount: groups.length + hiddenSources.length,
    senseCount,
    // Cận dưới ở CẤP TỔNG khi bất kỳ nguồn nào bị cắt, hoặc có nguồn bị giấu hẳn.
    senseCountAtLeast: sources.some((s) => s.atLeast) || hiddenSources.length > 0,
    sources,
    hidden: hiddenSources.map(([displayName, count]) => ({ displayName, count })),
  }
}

const query = shallowRef<string | null>(null)
const response = shallowRef<LookupResponse | null>(null)
const error = shallowRef<IpcError | null>(null)

/**
 * 🔴 **Truy vấn của kết quả ĐANG HIỆN** — không phải [`query`], và khoảng cách giữa hai thứ
 * đó là một lỗi thật đã bắt ở code review 2026-08-07.
 *
 * `query` đổi NGAY khi lượt tra được phát đi; kết quả thì mãi sau mới về. Nếu đầu mục đọc
 * `query` còn thân panel đọc `response`, thì trong cả khoảng chờ IPC panel hiện **đầu mục
 * MỚI kèm bản ghi CŨ** — một bản ghi có cấu trúc dán nhãn sai từ, đúng thứ FR28 tồn tại
 * để không xảy ra. Hai giá trị này luôn được đặt **cùng một lượt**, nên chúng không lệch nhau.
 */
const resolvedQuery = shallowRef<string | null>(null)

/**
 * `true` từ lúc lượt tra được phát đi tới lúc nó trả lời — cùng vai trò
 * `sourceHanVietPending` (Story 1.16): AC7 cấm spinner, nhưng bản thân trạng thái *"đang
 * tra"* vẫn phải phân biệt được với *"đã tra mà không có gì"*, nếu không một khoảng chờ
 * IPC sẽ nháy sang chuỗi *"không tìm thấy"* rồi lại đổi — đúng bẫy đã bắt ở 1.16.
 */
const pending = ref(false)

/**
 * 🔴 Số thứ tự lượt tra — xem [`runLookup`]. Tăng ở **mỗi** lượt phát đi VÀ ở
 * [`resetLookupPanel`], nên một promise đang bay không bao giờ ghi được vào state sau khi
 * Tác phẩm đã đổi (đúng thứ reset tồn tại để chặn, Bẫy 8).
 *
 * ⚠️ không phải `ref` — không một bề mặt nào render nó, và làm nó phản ứng chỉ thêm một lượt
 * cập nhật thừa cho mỗi lượt tra.
 */
let sequence = 0

/**
 * Đầu mục ĐANG HIỆN — truy vấn của **kết quả trên màn hình**, không của lượt đang bay.
 *
 * ⚠️ Trong khoảng chờ IPC nó giữ nguyên giá trị cũ, đúng lời hứa AC7 *"panel giữ nguyên
 * trạng thái TRƯỚC ĐÓ cho tới khi lượt tra mới trả lời"* — xem [`resolvedQuery`].
 */
export const currentQuery: DeepReadonly<Ref<string | null>> = readonly(resolvedQuery)
/** `true` trong khoảng chờ lượt tra. */
export const lookupPending: DeepReadonly<Ref<boolean>> = readonly(pending)
/** Lỗi gần nhất Rust trả lời. */
export const lookupError: DeepReadonly<Ref<IpcError | null>> = readonly(error)

/**
 * 🔴 Lượt tra đã trả lời và trả lời **được** — điều kiện để các vị từ trạng thái dưới đây
 * có nghĩa. `false` ⇔ đang chờ, hoặc lượt tra trượt, hoặc chưa từng chạy. Cùng doctrine
 * `hanVietResolved` (Story 1.16, bắt ở lượt code review 2026-08-06 — bản đầu không có
 * vị từ này, và một lượt IPC hỏng hoàn toàn trông y hệt một từ điển rỗng).
 */
export const lookupResolved = computed(() => !pending.value && error.value === null && response.value !== null)

/**
 * 🔴 **Có DỮ LIỆU để hiện hay không** — không đồng nghĩa [`lookupResolved`], và đó là điểm.
 *
 * `lookupResolved` trả lời *"lượt tra GẦN NHẤT đã xong chưa"* ⇒ nó `false` suốt khoảng
 * chờ IPC. Cổng **bề mặt hiển thị** qua nó làm toàn bộ bản ghi biến mất mỗi lượt tra lại
 * — trong khi chú thích AC7 ngay tại `LookupPanel.vue` hứa điều ngược lại (bắt ở code
 * review 2026-08-07). Vị từ này trả lời câu khác: *"có một kết quả HỢP LỆ trên tay không"*,
 * và nó không quan tâm tới lượt đang bay.
 *
 * ⇒ **Bốn trạng thái rỗng của AC6 đọc [`lookupResolved`]** (không nháy trong khoảng chờ);
 * **bản ghi, thanh nhịp và hai banner đọc vị từ NÀY** (không chớp trắng).
 */
export const lookupDisplayable = computed(() => error.value === null && response.value !== null)

/**
 * 🔴 **NĂM trạng thái, NĂM vị từ RIÊNG** (AC6 · §hệ quả ② của Quyết định #4) — không một
 * chuỗi `if/else` nào trong `<template>` được phép gộp chúng. Mỗi vị từ độc lập, và
 * `LookupPanel.vue`/`LookupRecord.vue` tự chọn ĐÚNG MỘT nhánh theo thứ tự ưu tiên: pending
 * → error → !resolved → !layersLoaded → queryTooShort → notFound → có kết quả.
 */

/** Trạng thái 1 — chưa tra gì lần nào (dạy thao tác bôi đen). */
export const neverLookedUp = computed(() => query.value === null)

/** Trạng thái 2 — đã tra mà không tìm thấy gì (`groups` rỗng, `branch ≠ query_too_short`). */
export const notFound = computed(() => {
  if (!lookupResolved.value || response.value === null) return false
  const g = response.value.grouped
  return g.layers_loaded && g.branch !== 'query_too_short' && g.groups.length === 0
})

/** Trạng thái 3 — `branch === 'query_too_short'` (AD-44 ④, không phải "không tìm thấy"). */
export const queryTooShort = computed(
  () => lookupResolved.value && response.value?.grouped.branch === 'query_too_short',
)

/**
 * Trạng thái 4 (ca thứ năm của AC6, không phải một trạng thái rỗng) — không một lớp từ
 * điển nào đang gắn (AD-25). Mặc định `true` khi chưa có dữ liệu — cùng luật `layersLoaded`
 * của `sourcePanelState.ts`: chỉ có nghĩa SAU khi [`lookupResolved`] đã `true`.
 */
export const layersLoaded = computed(() => response.value?.grouped.layers_loaded ?? true)

/**
 * 🔴 **Đường ngôn ngữ của lượt tra gần nhất** — Story 1.19 AC6, thêm ở code review
 * 2026-08-10.
 *
 * `pick_route` chạy **đúng một lần** phía Rust và giá trị đi về trong `grouped.route`, nên
 * webview **đọc** nó chứ không tính lại — cùng mệnh đề AD-44 ① vá A1 mà `core/dict/mod.rs`
 * đã ghi: *"một adapter không tự phân xử"*. Một bản sao của `pick_route` ở đây sẽ trả lời
 * khác đường thật vào đúng ngày luật phân đường đổi.
 *
 * ⚠️ `null` khi chưa tra lượt nào — chỗ tiêu thụ phải xử ca đó, vì *"mọi nguồn của đường
 * đang tra đều tắt"* không có nghĩa khi chưa có đường nào.
 */
export const lookupRoute = computed<QueryRoute | null>(() => response.value?.grouped.route ?? null)

/**
 * Trạng thái 5 — một phần từ điển không trả lời (`skipped` khác rỗng). ⚠️ **không
 * loại trừ lẫn nhau với có kết quả** — banner này hiện SONG SONG với `groups` khi có, khác
 * với bốn vị từ trên (chọn đúng một nhánh).
 */
export const someLayerFailed = computed(
  () => lookupDisplayable.value && (response.value?.grouped.skipped.length ?? 0) > 0,
)

/**
 * §Hệ quả ② (Quyết định #4) — `LIMIT` đã cắt bớt kết quả của ít nhất một lớp. Panel nói
 * "danh sách nguồn chưa đầy đủ" khi `true` — không im (AC12).
 */
export const someLayerTruncated = computed(
  () => lookupDisplayable.value && (response.value?.grouped.truncated_layers.length ?? 0) > 0,
)

/**
 * 🔴 Vùng chọn dài hơn trần `QUERY_LENGTH_CEILING` ⇒ truy vấn đã bị CẮT trước khi tra.
 * Banner riêng, không gộp vào "không tìm thấy" — câu đó SAI ở đây (hệ thống không hề tra thứ người
 * dùng chọn). Cùng hạng banner với hai vị từ trên: hiện SONG SONG, không loại trừ.
 */
export const queryWasTruncated = computed(
  () => lookupDisplayable.value && response.value?.query_truncated === true,
)

/**
 * Kết quả pha một GOM — `null` khi chưa tra hoặc lượt tra trượt.
 *
 * ⚠️ Cổng qua [`lookupDisplayable`], không [`lookupResolved`]: trong khoảng chờ IPC nó GIỮ
 * kết quả cũ (AC7), không trả `null` làm cả thân panel chớp trắng.
 */
export const groupedLookup = computed(() => (lookupDisplayable.value ? (response.value?.grouped ?? null) : null))

/** Nghĩa đã hydrate theo lớp (pha hai) — cùng luật [`groupedLookup`]. */
export const sensesByLayer = computed(() =>
  lookupDisplayable.value ? (response.value?.senses_by_layer ?? {}) : {},
)

/**
 * Handler thật của `lookup.lookup_selection` (Quyết định #1a) — **MỘT** đường vào duy
 * nhất. `deps.currentSelection` (tiêm ở `src/main.ts`) cấp văn bản; Story 1.18 chỉ cần
 * thay ĐÚNG dep đó bằng hợp đồng vùng chọn thật, không phải chạm hàm này.
 *
 * ⚠️ **Không đoán chế độ tra** — `LookupMode::Exact` cố định phía Rust (Quyết định #3);
 * adapter này chỉ chuyển tiếp `query`.
 */
export async function runLookup(rawQuery: string): Promise<void> {
  const trimmed = rawQuery.trim()
  if (trimmed === '') return

  // 🔴 **Số thứ tự lượt — không một lượt CŨ về sau được ghi đè lượt MỚI.**
  //
  // `query.value` đặt TRƯỚC `await`, còn `response.value` đặt bởi *bất kỳ* lượt nào về
  // trước. Hai lần `Mod+Alt+L` liên tiếp: nếu lượt A về sau lượt B thì panel hiện đầu mục
  // B kèm bản ghi A — và `pending` đã bị B tắt nên `lookupResolved` là `true`, không một dấu
  // hiệu nào báo sai, VĨNH VIỄN. Bắt ở code review 2026-08-07; Story 1.18 (Auto-Lookup)
  // biến lỗ này thành thường trực.
  const mine = ++sequence

  query.value = trimmed
  pending.value = true
  const { response: result, error: err } = await lookupDictionary(trimmed)

  // Lượt này đã bị một lượt mới hơn — hoặc bởi `resetLookupPanel()` — vượt mặt.
  if (mine !== sequence) return

  pending.value = false
  // ⚠️ MỘT lượt đặt cho cả ba — xem [`resolvedQuery`]. Tách chúng ra là mở lại đúng lỗ
  // "đầu mục mới, bản ghi cũ".
  resolvedQuery.value = err === null ? trimmed : null
  response.value = result
  error.value = err

  // ═════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 1.20 · AC11 — LỊCH SỬ TRA CỨU GHI Ở **ĐÚNG MỘT** CHỖ, VÀ ĐÂY LÀ CHỖ ĐÓ
  // ═════════════════════════════════════════════════════════════════════════════
  //
  // ⚠️ **SAU guard `mine !== sequence` ở trên, không trước.** Ghi trước guard làm một lượt
  // tra đã bị vượt mặt — hoặc đã bị `resetLookupPanel()` huỷ — vẫn để lại một dòng lịch sử,
  // tức lịch sử của Tác phẩm A rò sang Tác phẩm B: đúng thứ `sequence` tồn tại để chặn.
  //
  // 🔴 Chỉ khi `err === null`, **kể cả** khi `groups` rỗng: *"đã tra mà không thấy"* là một
  // sự kiện thật đáng nhớ, còn *"không tra được"* là một câu khác hẳn — trộn hai thứ là
  // đúng bẫy `??` mà Story 1.17 đã bắt.
  if (err === null && result !== null) recordLookup(trimmed, result)
}

/**
 * 🔴 **Vứt toàn bộ state của Panel Lookup** — gọi khi Tác phẩm đang mở BỊ THAY.
 *
 * Cùng lý do và cùng điểm nghẽn `resetSourcePanel()`: kết quả tra cứu của Tác phẩm A không
 * được sống sót sang Tác phẩm B. Chỗ gọi duy nhất là `modes/libraryImport.ts::finishSubmit`
 * — **cùng lượt** với `resetSourcePanel()`, không một lời gọi thứ hai rải ra (đúng bẫy đã bắt
 * ở 1.16: "không đây không phải một lời gọi thứ hai rải ra — nó đi qua đúng điểm nghẽn").
 */
export function resetLookupPanel(): void {
  // 🔴 Vô hiệu hoá mọi lượt tra ĐANG BAY — nếu không, một promise về sau lượt reset sẽ
  // làm kết quả của Tác phẩm A **sống lại** dưới Tác phẩm B, đúng thứ hàm này tồn tại để
  // ngăn (bắt ở code review 2026-08-07).
  sequence += 1

  query.value = null
  resolvedQuery.value = null
  response.value = null
  error.value = null
  pending.value = false

  // 🔴 STORY 1.20 · AC12 — lịch sử của Tác phẩm A không được sống sót sang Tác phẩm B.
  // Cùng lượt, cùng điểm nghẽn — không một lời gọi thứ hai rải ra ở `libraryImport.ts`.
  //
  // ⚠️ **Bộ ghim thì KHÔNG nạp lại**, và đó là hệ quả của lượt Ice ký lại 2026-08-11: mục
  // ghim sống ở `global.db`, tức nó không thuộc Tác phẩm nào. Vế *"nạp lại theo Tác phẩm
  // mới"* của AC12 tự rụng cùng phạm vi cũ — đọc doc-comment của `resetLookupHistory()`
  // để thấy hàm đó vứt đúng cái gì và cố ý giữ lại cái gì.
  resetLookupHistory()
}
