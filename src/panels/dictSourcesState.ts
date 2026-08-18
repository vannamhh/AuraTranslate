/**
 * State của **dải chip nguồn** và **bề mặt Attribution** — Story 1.19, AC1 · AC2 · AC5 ·
 * AC6 · AC7 · AC10.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY, KHÔNG TRONG `LookupPanel.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `lookupPanelState.ts`/`sourcePanelState.ts`: một lượt đổi preset bố cục gọi
 * `api.clear()` rồi dựng lại cả ba panel. State module-level là singleton của cả tiến
 * trình, nên lựa chọn tắt/bật sống sót qua lượt tháo/dựng lại đó — và nó phải sống sót,
 * vì nó là một lựa chọn của người dùng chứ không một trạng thái hiển thị.
 *
 * ⚠️ Cùng lý do, tệp này KHÔNG được `import` vào `src/commands/index.ts` trực tiếp — nó
 * dùng `ref` của Vue **và** gọi `@tauri-apps/api` xuyên qua `config/dict.ts`, mà Kiểm C/D/E
 * của `npm run check:commands` nạp tệp đó bằng Node thuần. Hai handler đi vào bằng **tiêm**
 * qua `CommandDeps` ở `src/main.ts` — cùng cửa `selectSourceTab`/`runLookup` đã đi qua.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 DANH SÁCH NGUỒN DẪN XUẤT TỪ `list_dict_sources`, KHÔNG TỪ `groups` — AC1
 * ─────────────────────────────────────────────────────────────────────────────
 * Một nguồn **đang tắt** không sinh nhóm nào, nên một dải chip dẫn xuất từ `groups` sẽ
 * **không có chip để bật nó lại** — người dùng tự khoá mình ra ngoài. Và một nguồn không
 * khớp truy vấn hiện tại cũng không có nhóm, nên dải chip sẽ nhấp nháy theo từng lượt tra.
 * Danh sách phải đến từ **tập lớp đang gắn**, đọc **một lần** lúc khởi động.
 */
import { computed, readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { listDictSources } from '../config/dict'
import type { QueryRoute, SourceAttribution } from '../config/dict'
import { KEY_DICT_DISABLED, SCOPE_APP_CONFIG, putConfig } from '../config/bootstrap'
import type { IpcError } from '../i18n'
import { currentQuery, runLookup } from './lookupPanelState'
import { refreshHanViet } from './sourcePanelState'

const sources = shallowRef<readonly SourceAttribution[]>([])
const sourcesError = shallowRef<IpcError | null>(null)
const disabled = ref<ReadonlySet<string>>(new Set())
const attributionOpen = ref(false)

/** Mọi nguồn của mọi tệp đang gắn, kể cả nguồn đang TẮT (AC10). Thứ tự tất định từ Rust. */
export const dictSources: DeepReadonly<Ref<readonly SourceAttribution[]>> = readonly(sources)

/** Lỗi gần nhất Rust trả lời khi đọc danh sách nguồn. */
export const dictSourcesError: DeepReadonly<Ref<IpcError | null>> = readonly(sourcesError)

/** Lớp phủ Attribution có đang mở không — AC11. */
export const attributionIsOpen: DeepReadonly<Ref<boolean>> = readonly(attributionOpen)

/** Một `code` có đang TẮT không. */
export function sourceIsDisabled(code: string): boolean {
  return disabled.value.has(code)
}

/**
 * Nguồn `src` có phục vụ đường `route` không — **hàm THUẦN, xuất được, test được**.
 *
 * Đọc `lang` bằng đúng [`decodeDisabled`], không một bộ tách thứ hai: hai trường khác nhau
 * dùng chung một quy ước mã hoá thì phải dùng chung một bộ tách, nếu không chúng sẽ lệch.
 *
 * ⚠️ `lang` **rỗng** ⇒ nguồn không phục vụ đường nào ⇒ luôn `false`. Đó là hình dạng của một
 * nguồn 0 đầu mục, và cũng là hình dạng một tệp `.db` dựng bằng bản `dict-build` cũ hơn
 * Story 1.19. Trả `false` là an toàn về một phía đúng: [`everySourceOffForRoute`] sẽ **không**
 * khẳng định *"mọi nguồn đều tắt"* dựa trên một nguồn nó không hiểu.
 */
export function sourceServesRoute(src: SourceAttribution, route: QueryRoute): boolean {
  return decodeDisabled(src.lang).has(route)
}

/**
 * 🔴 **AC6 — "MỌI NGUỒN ĐỀU TẮT" LÀ MỘT TRẠNG THÁI CÓ TÊN**, và nó hỏi theo **ĐƯỜNG ĐANG
 * TRA**, không theo toàn tập nguồn.
 *
 * Lý do là một số đo (Bẫy 5): đúng **MỘT** nguồn thật phục vụ đường tiếng Anh
 * (`viwiktionary-en`). Tắt riêng nó ⇒ **mọi** truy vấn tiếng Anh trả rỗng trong khi bảy
 * nguồn tiếng Trung vẫn bật — một vị từ hỏi *"toàn tập có còn nguồn nào không"* trả `false`,
 * và panel nói *"không tìm thấy trong từ điển"*: một câu **SAI**, hệ thống không hề tra.
 *
 * 🔴 **Bản đầu của Story 1.19 hỏi TOÀN TẬP, và đó là AC6 chưa đạt** — Acceptance Auditor bắt
 * ở code review 2026-08-10, Ice chốt vá thật thay vì ghi thành món nợ. Lý lẽ cũ *("webview
 * không biết nguồn nào phục vụ đường nào, và dựng bảng tra `code → lang` ở đây là dựng đúng
 * sổ đăng ký AD-44 ① vá A2 cấm")* đúng ở vế sau và sai ở vế trước: câu trả lời không phải
 * dựng một bảng tra ở webview, mà là **để dữ liệu tự khai** — `dict_source.lang` nay được
 * **đo lúc dựng** từ `dict_entry` của chính tệp *(cùng đường `is_base` đọc `dict_meta`)*, nên
 * webview chỉ **đọc một trường**, không suy luận gì cả.
 *
 * ⚠️ Vẫn giữ `sources.value.length > 0`: 0 nguồn nào gắn là trạng thái *"chưa có từ điển"*,
 * một câu khác hẳn *"bạn đã tắt hết"*, và bảng Attribution đã nói câu đó rồi.
 */
export function everySourceOffForRoute(route: QueryRoute): boolean {
  const serving = sources.value.filter((s) => sourceServesRoute(s, route))
  return serving.length > 0 && serving.every((s) => disabled.value.has(s.code))
}

/** Số nguồn đang bật — dải chip đọc để không phải tự đếm trong `<template>`. */
export const enabledSourceCount = computed(
  () => sources.value.filter((s) => !disabled.value.has(s.code)).length,
)

/**
 * Mã hoá tập bị tắt thành chuỗi trên đĩa — **hàm THUẦN, xuất được, test được**.
 *
 * ⚠️ Sắp xếp trước khi nối: hai lượt lưu cùng một tập phải cho **cùng một** chuỗi, nếu không
 * mỗi lượt bật/tắt ghi một giá trị khác nhau lên `global.db` cho cùng một trạng thái, và một
 * lượt so sánh (test, hay Story 1.21) không đứng được. Cùng quy ước mà
 * `core::scope::parse_disabled_sources` tách ra.
 */
export function encodeDisabled(codes: Iterable<string>): string {
  return [...codes].sort().join(',')
}

/**
 * Tách chuỗi trên đĩa — **hàm THUẦN**, đối xứng với [`encodeDisabled`].
 *
 * 🔴 Một bản chép của `core::scope::parse_disabled_sources`, và đó là **chủ ý có giới hạn**:
 * phép lọc THẬT chạy ở Rust (AD-1) — bản này chỉ để dải chip biết vẽ chip nào mờ. Hai bản
 * không được phép khác nhau, và chúng không thể: quy ước là *"cắt theo `,`, trim, bỏ rỗng"*,
 * ba dòng, không một ca biên nào để trôi.
 */
export function decodeDisabled(raw: string): Set<string> {
  return new Set(
    raw
      .split(',')
      .map((code) => code.trim())
      .filter((code) => code !== ''),
  )
}

/**
 * Nạp danh sách nguồn và tập bị tắt lúc khởi động — gọi **một lần**, từ `src/main.ts`.
 *
 * ⚠️ `raw` đến từ `bootstrap_config`, tức từ **cùng một** vòng IPC đã nạp theme/mode. Không
 * một lượt đọc thứ hai cho cùng một giá trị.
 */
export async function loadDictSources(raw: string): Promise<void> {
  disabled.value = decodeDisabled(raw)
  const { sources: loaded, error } = await listDictSources()
  sourcesError.value = error
  if (loaded !== null) sources.value = loaded
}

/**
 * Hàng đợi **NỐI TIẾP** cho mọi lượt ghi tập bị tắt — mỗi lượt bấm móc vào đuôi lượt trước.
 *
 * 🔴 Vì sao một hàng đợi chứ không `void putConfig(...)` rời rạc (bắt ở code review
 * 2026-08-10): mỗi lượt ghi mang **toàn bộ ảnh chụp** tập bị tắt tại thời điểm bấm, và hai
 * lượt `invoke('put_config')` phát độc lập **không** đảm bảo hoàn tất đúng thứ tự phát. Tắt
 * nguồn A rồi tắt B ngay sau: nếu lượt mang `{A}` về **sau** lượt mang `{A,B}` thì đĩa giữ
 * `{A}` trong khi màn hình hiện `{A,B}`, và sai lệch chỉ lộ ra ở lần khởi động kế tiếp —
 * một lượt tắt bị **âm thầm hoàn tác**.
 */
let writeQueue: Promise<void> = Promise.resolve()

/** Số thứ tự lượt bấm — chỉ lượt MỚI NHẤT được quyền tra lại và được quyền lùi state. */
let toggleSequence = 0

/**
 * 🔴 **Handler thật của `lookup.toggle_source`** — bật/tắt MỘT nguồn, hiệu lực NGAY (AC2).
 *
 * 🔴 **LƯU XONG RỒI MỚI TRA LẠI, và thứ tự đó là bắt buộc** (bắt ở code review 2026-08-10).
 * Bản đầu tra lại **trước** rồi mới lưu, với lý lẽ *"không chờ đĩa hay IPC"*. Lý lẽ ấy đúng
 * cho vế **vẽ chip**, và sai cho vế tra lại: trên dây **không có** tham số `disabled` nào
 * *(`lookupDictionary` gửi đúng `{ query }`, `readHanViet` gửi đúng `{ chars }`)*, còn
 * `commands/dict.rs` mở đầu **cả hai** command bằng `disabled_sources(store)` đọc thẳng từ
 * tầng Global. Phát lượt đọc trước lượt ghi ⇒ Rust lọc theo tập **TRƯỚC KHI BẤM**, và
 * *"hiệu lực NGAY"* của AC2 thành một câu sai ngay ở lượt bấm đầu tiên.
 *
 * Ba việc, đúng thứ tự **mới**:
 * 1. đổi state ⇒ dải chip vẽ lại **tức thì**, không chờ gì cả — vế này giữ nguyên;
 * 2. lưu xuống tầng Global (AC5), **nối vào [`writeQueue`]** để hai lượt bấm liên tiếp ghi
 *    đúng thứ tự bấm;
 * 3. lưu **xong** mới tra lại — qua `runLookup`, tức đi *qua* điểm nghẽn của Story 1.17 chứ
 *    không *quanh* nó, nên Story 1.20 (lịch sử) vẫn chỉ có một chỗ để cắm vào.
 *
 * 🔴 **Lượt lưu TRƯỢT ⇒ LÙI state lại.** Đây là hệ quả trực tiếp của việc bước 3 nay phụ
 * thuộc bước 2: đĩa không đổi nghĩa là Rust vẫn lọc theo tập cũ, nên để chip hiện trạng thái
 * mới là để **màn hình nói dối**. AD-34 *(bật/tắt phải MƯỢT)* vẫn được tôn trọng — không một
 * hộp thoại lỗi nào, chỉ một dòng chẩn đoán và một dải chip **đúng sự thật**. Chỉ lượt mới
 * nhất được lùi: một lượt cũ trượt trong khi một lượt mới đã ghi thành công thì đĩa đang giữ
 * ảnh chụp của lượt mới, và lùi theo lượt cũ mới là chỗ sai.
 *
 * 🔴 **Panel Source cũng phải tra lại** (§Quyết định #3a, hệ quả thứ hai): bộ lọc áp cho cả
 * đường âm Hán Việt, nên giữ âm cũ trên màn hình là để một nguồn *"đã tắt"* vẫn viết chữ lên
 * tab Hán Việt.
 *
 * ⚠️ Một `code` không có trong danh sách là thao tác **VÔ HẠI**, không một lỗi — cùng luật
 * `selectSourceTab`/`toggleHanVietView` khi thao tác không áp dụng.
 */
export function toggleDictSource(code: string): void {
  if (!sources.value.some((s) => s.code === code)) return

  const previous = disabled.value
  const next = new Set(previous)
  if (!next.delete(code)) next.add(code)
  disabled.value = next

  const mine = ++toggleSequence
  writeQueue = writeQueue
    .then(async () => {
      const err = await putConfig(SCOPE_APP_CONFIG, KEY_DICT_DISABLED, encodeDisabled(next))
      if (err !== null) {
        console.warn(`[dict] không lưu được danh sách nguồn đang tắt (\`${err.code}\`).`)
        if (mine === toggleSequence) disabled.value = previous
        return
      }

      // Một lượt bấm mới hơn đã vượt mặt: lượt của nó sẽ tra lại với tập ĐÚNG, và một lượt
      // tra của lượt cũ ở đây chỉ là một lượt IPC thừa có khả năng về sau và ghi đè.
      if (mine !== toggleSequence) return

      const query = currentQuery.value
      if (query !== null) void runLookup(query)
      void refreshHanViet()
    })
    // ⚠️ Chuỗi **không được phép** ở lại trạng thái rejected: một lượt ném ngoài dự kiến sẽ
    // làm MỌI lượt ghi sau đó bị bỏ qua im lặng, tức một lỗi nhất thời khoá vĩnh viễn tính
    // năng — đúng hình dạng đã bắt ở `ensureHanVietLoaded` (cờ không bao giờ nhả).
    .catch((err: unknown) => {
      console.warn(`[dict] hàng đợi lưu nguồn đang tắt ném ngoài dự kiến: ${String(err)}`)
    })
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 AC9 — `license_kind` RA MỘT CÂU, VÀ BẢNG ÁNH XẠ **CÓ NHÁNH MẶC ĐỊNH**
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Bốn `license_kind` **đo được trên dữ liệu thật** (2026-08-08) cộng **một CHỖ GIỮ**.
 *
 * 🔴 `author-grant` là **chỗ giữ**: **0** nguồn thật nào mang nó hôm nay, và HVTĐTD — nguồn
 * mà AC gốc của epic neo vào — **sẽ không tới** (Ice chốt 2026-08-08: không tìm được nguồn
 * dữ liệu). Nó ở lại vì nó **rẻ** (một hàng ở đây, một chuỗi ở `vi.json`) và vì nó là **bằng
 * chứng chạy được** cho mệnh đề AD-10 *"trường giấy phép không phải enum các giấy phép mở"*
 * — thứ mà bốn giá trị thật hiện có không chứng minh nổi một mình, vì chưa cái nào là *"phép
 * riêng do tác giả cấp"*.
 *
 * 🔴 Câu của `author-grant` **KHÔNG mang một cái tên nào**: danh tính tác giả đọc từ
 * `dict_source.attribution` của **chính tệp** (AC9), và `tests/dict_boundary.rs` canh mệnh đề
 * đó bằng máy. Đó là điều kiện để chỗ giữ này dùng lại được cho một nguồn **khác**.
 */
const LICENSE_KIND_KEYS: Readonly<Record<string, string>> = {
  open: 'attribution.license_open',
  'public-domain': 'attribution.license_public_domain',
  copyrighted: 'attribution.license_copyrighted',
  unknown: 'attribution.license_unknown',
  'author-grant': 'attribution.license_author_grant',
}

/**
 * 🔴 **NHÁNH MẶC ĐỊNH — AC9.** Một `license_kind` chưa gặp bao giờ ra một câu **CÓ NGHĨA**:
 * nó nêu rằng giấy phép này chưa được ứng dụng phân loại và trỏ người đọc vào trường
 * `attribution` nguyên văn. **Không** rỗng, **không** chuỗi máy thô, **không** panic.
 */
export const LICENSE_FALLBACK_KEY = 'attribution.license_unrecognised'

/** Khoá `vi.json` cho một `license_kind` — **hàm THUẦN, xuất được, test được**. */
export function licenseKeyFor(kind: string): string {
  return LICENSE_KIND_KEYS[kind] ?? LICENSE_FALLBACK_KEY
}

/**
 * Khoá `vi.json` cho cột **Lớp** — `nền` hay `gỡ rời` (AC7).
 *
 * ⚠️ Một hàm chứ không một biểu thức ba ngôi trong `<template>`: Kiểm A2 của
 * `check-i18n.mjs` đòi mọi mustache là **một** lời gọi `t()`, và `a ? t(x) : t(y)` không
 * phải — nó là hai lời gọi trong một biểu thức, và cổng đọc **tĩnh**.
 */
export function layerKeyFor(isBase: boolean): string {
  return isBase ? 'attribution.layer_base' : 'attribution.layer_detachable'
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 AC11 · §KHÔNG-LÀM ⑤ — COMMAND TĨNH, KHÔNG MỘT COMMAND CHO MỖI NGUỒN
// ═════════════════════════════════════════════════════════════════════════════════
//
// Mockup vẽ `⌥1…6` cạnh dải chip (`sources-attribution.html:140`). **Bác**, ba lý do đo được:
// ① danh sách nguồn **dẫn xuất lúc chạy** (0 tới 10 nguồn tuỳ tệp có mặt), còn
//    `CommandRegistry` là một danh sách TĨNH mà `check-commands.mjs` đếm bằng máy
//    (`COMMAND_FLOOR`) — một command sinh động phá chính cơ chế cưỡng chế của AD-34;
// ② `Mod+Alt+1`/`Mod+Alt+2` **đã thuộc** preset bố cục (Story 1.14);
// ③ FR22/Story 1.21 đòi **mọi** command gán lại được, mà một id không tồn tại lúc dựng màn
//    hình phím thì không gán được.
//
// ⇒ `lookup.toggle_source` bật/tắt nguồn **ĐANG ĐƯỢC NHẮM**, và nó đọc mục tiêu từ trạng
// thái quanh nó tại thời điểm chạy — đúng khuôn `deps.currentSelection` của
// `lookup.lookup_selection` (Story 1.17), không một cửa thứ hai.

/**
 * Chip vừa bị **bấm chuột**, giữ đúng một lượt.
 *
 * 🔴 **Vì sao không chỉ đọc `document.activeElement`:** WebKit (WKWebView — engine Tauri
 * dùng trên macOS, NFR14) **không đặt tiêu điểm cho `<button>` khi bấm chuột**. Đọc mỗi
 * `activeElement` là để cả đường chuột chết trên macOS trong khi xanh trên Windows.
 *
 * ⚠️ **Tiêu thụ MỘT LẦN** rồi trả về `null` — không một giá trị cũ nào được sống tới lượt
 * bấm phím kế tiếp, vì lúc đó tiêu điểm có thể đã ở một chỗ hoàn toàn khác.
 */
let aimedCode: string | null = null

/** `code` của chip đang có tiêu điểm DOM, hoặc `null`. */
function focusedChipCode(): string | null {
  if (typeof document === 'undefined') return null
  const active = document.activeElement
  if (!(active instanceof HTMLElement)) return null
  return active.dataset.sourceCode ?? null
}

/**
 * Nhắm chip nằm dưới một sự kiện chuột — gọi từ `@mousedown` trên **vùng chứa** dải chip.
 *
 * ⚠️ Uỷ quyền sự kiện ở vùng chứa chứ không một handler trên từng chip: một handler cho mỗi
 * chip trên một danh sách sinh động là N chỗ để sai, và `@click` của chip phải ở lại **đúng
 * một** lời gọi `dispatch` (AC11, Kiểm A của `check:commands`).
 */
export function aimDictSourceFrom(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLElement)) return
  const chip = target.closest<HTMLElement>('[data-source-code]')
  aimedCode = chip?.dataset.sourceCode ?? null
}

/**
 * Handler thật của `lookup.toggle_source` — bật/tắt nguồn **đang được nhắm** (AC2, AC11).
 *
 * 🔴 **`aimedCode` ĐỨNG TRƯỚC `focusedChipCode()`, và thứ tự đó là cả điểm** (bắt ở code
 * review 2026-08-10). Bản đầu hỏi tiêu điểm DOM trước, và nó **tự mâu thuẫn** với chính lý
 * do `aimedCode` tồn tại: WKWebView **không** dời tiêu điểm khi bấm chuột vào `<button>`.
 * Ca hỏng đo được trên macOS: Tab tới chip A *(giờ `activeElement` là A)*, rồi **bấm chuột**
 * chip B. `mousedown` đặt `aimedCode = 'B'` đúng, nhưng `activeElement` vẫn là A, nên bản
 * đầu bật/tắt **A** — sai nguồn, và sai lặp lại ở **mọi** cú bấm kế tiếp cho tới khi một
 * lượt Tab dời tiêu điểm đi.
 *
 * Đảo lại là đúng vô điều kiện, không một sự đánh đổi nào: `aimedCode` chỉ khác `null` khi
 * một cú `mousedown` **vừa** rơi trúng một chip, tức nó luôn là ý định **mới nhất**; và nó
 * **tiêu thụ một lần** nên không bao giờ sống sang lượt bấm phím kế tiếp, lúc mà tiêu điểm
 * DOM mới là câu trả lời đúng.
 *
 * Không nguồn nào đang được nhắm ⇒ thao tác **VÔ HẠI**, không một lỗi — cùng luật vùng chọn
 * rỗng của `lookup.lookup_selection`.
 */
export function toggleFocusedDictSource(): void {
  const code = aimedCode ?? focusedChipCode()
  aimedCode = null
  if (code === null) return
  toggleDictSource(code)
}

/** Handler thật của `attribution.open` — AC11. */
export function openAttribution(): void {
  attributionOpen.value = true
}

/**
 * 🔵 **STORY 2.12 · AC5** — đưa module về đúng trạng thái TRƯỚC [`loadDictSources`].
 *
 * Ice ký 2026-08-18 (quyết định #2c): tệp này là một trong hai tệp `src/panels/**` có ô nhớ
 * cấp module mà **không hàm reset nào**, và đường *"ngoài phạm vi, ghi nợ"* bị loại.
 *
 * 🔴 **VÀ NÓ CỐ Ý KHÔNG NẰM TRÊN ĐƯỜNG ĐỔI TÁC PHẨM — đọc dòng này trước khi nối nó vào.**
 * [`disabled`] là cấu hình tầng **Global** (`commands/dict.rs` đọc `disabled_sources(store)`
 * cho MỌI lượt tra, không theo Tác phẩm), và nó chỉ được nạp lại **một lần lúc khởi động**
 * qua [`loadDictSources`]. Gọi hàm này ở chỗ `resetEditorPanel()` được gọi sẽ xoá tập bị tắt
 * trong bộ nhớ mà **không ai đọc lại nó từ đĩa** ⇒ mọi nguồn người dùng đã tắt lặng lẽ bật
 * lại, và đĩa vẫn giữ tập cũ nên lần khởi động sau chúng tắt lại. Một khuyết tật **chỉ hiện
 * ra khi đổi Tác phẩm**, và không cổng nào đỏ.
 *
 * ⇒ Chủ gọi hợp lệ: một lượt dựng lại phiên **có kèm** một lượt [`loadDictSources`] ngay sau.
 */
export function resetDictSources(): void {
  toggleSequence += 1
  sources.value = []
  sourcesError.value = null
  disabled.value = new Set()
  attributionOpen.value = false
  aimedCode = null
  // Hàng đợi ghi về lại trạng thái "không có gì đang bay". Xem [`writeQueue`]: một lượt ghi
  // của phiên CŨ còn treo ở đuôi hàng đợi sẽ nối tiếp vào lượt ghi đầu của phiên MỚI.
  writeQueue = Promise.resolve()
}

/**
 * Đóng lớp phủ Attribution. Gọi từ `Escape` và từ nút đóng.
 *
 * ⚠️ Việc **trả tiêu điểm về chỗ cũ** (UX-DR17) sống trong chính component — nó cần phần tử
 * DOM đang có tiêu điểm lúc mở, và một state module-level không phải chỗ giữ một `HTMLElement`
 * qua vòng đời component.
 */
export function closeAttribution(): void {
  attributionOpen.value = false
}
