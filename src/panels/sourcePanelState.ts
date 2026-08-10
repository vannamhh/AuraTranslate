/**
 * State của Panel Source — Chương đã nạp, âm Hán Việt đã tra, tab/kiểu xem đang chọn.
 * Story 1.16, AC9 · Quyết định #5.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY, KHÔNG TRONG `SourcePanel.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * AC9: đổi preset bố cục chạy `WorkspaceDock.vue::applyPreset()` → `api.clear()` rồi dựng
 * lại **cả bốn** panel — tức tháo và mount lại instance `SourcePanel.vue`. Một `ref` khai
 * trong `<script setup>` của nó chết cùng lượt tháo đó. State ở đây là **module-level**
 * (singleton của cả tiến trình, cùng khuôn `src/config/bootstrap.ts::layout` và
 * `src/layout/dockController.ts`), nên nó SỐNG SÓT qua một lượt tháo/dựng lại — panel mới
 * đọc lại đúng state cũ thay vì tra lại từ đầu.
 *
 * ⚠️ Cùng lý do, tệp này KHÔNG được `import` vào `src/commands/index.ts` trực tiếp — nó
 * dùng `ref` của Vue, và Kiểm C/D/E của `npm run check:commands` nạp tệp đó bằng Node
 * thuần. Hai hàm [`selectSourceTab`]/[`toggleHanVietView`] được TIÊM VÀO qua `CommandDeps`
 * ở `src/main.ts` — cùng cửa mà `applyPreset`/`togglePanel`/`submitPastedText` đã đi qua.
 */
import { computed, readonly, ref, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { readOpenChapter } from '../config/chapter'
import type { OpenChapter } from '../config/chapter'
import { readHanViet } from '../config/dict'
import type { CharacterReading, HanVietLookup } from '../config/dict'
import type { IpcError } from '../i18n'

/** Tab đang chọn ở thân Panel Source — chỉ có ý nghĩa khi `showHanVietTab` (AC3). */
export type SourceTab = 'original' | 'han_viet'

/** Kiểu xem của tab Hán Việt — Quyết định #4(a). */
export type HanVietViewMode = 'switch' | 'parallel'

/**
 * Bảy dải CJK — **chép nguyên văn** `src-tauri/src/core/dict/mod.rs::is_han`, vốn đã tự
 * chép từ `tools/dict-build/src/char_idx.rs::is_han` (MỘT định nghĩa, ba bản chép có canh —
 * hai bản Rust canh nhau bằng `tests/dict_lookup.rs`; bản TypeScript này KHÔNG có cổng
 * parity vì nó chạy phía webview, ngoài phạm vi hai workspace Rust. Sửa một bên mà quên
 * bên kia là một ký tự Hán không được coi là Hán ở Panel Source dù `dict-core.db` có nó).
 */
const HAN_RANGES: ReadonlyArray<readonly [number, number]> = [
  [0x3400, 0x4dbf],
  [0x4e00, 0x9fff],
  [0xf900, 0xfaff],
  [0x20000, 0x2a6df],
  [0x2a700, 0x2ebef],
  [0x2f800, 0x2fa1f],
  [0x30000, 0x3134f],
]

/** Ký tự (một code point, không một code unit UTF-16) có phải chữ Hán không. */
export function isHanChar(char: string): boolean {
  const cp = char.codePointAt(0)
  if (cp === undefined) return false
  return HAN_RANGES.some(([start, end]) => cp >= start && cp <= end)
}

const chapter = shallowRef<OpenChapter | null>(null)
const chapterError = shallowRef<IpcError | null>(null)
let chapterRequested = false

const hanViet = shallowRef<HanVietLookup | null>(null)
const hanVietError = shallowRef<IpcError | null>(null)
let hanVietRequested = false

/**
 * 🔴 **Số thứ tự lượt tra âm Hán Việt** — không một lượt CŨ nào được ghi đè lượt MỚI.
 *
 * Cùng cơ chế và cùng lý do với `sequence` của `lookupPanelState.ts` *(bắt ở code review
 * 2026-08-07)*, và nó phải có mặt ở đây kể từ Story 1.19 vì story đó mở ra chỗ gọi thứ hai:
 * [`refreshHanViet`] nhả `hanVietRequested` rồi gọi lại [`ensureHanVietLoaded`] **không**
 * chờ lượt đang bay kết thúc. Tắt nguồn A rồi tắt B ngay sau ⇒ **hai** lượt `read_han_viet`
 * cùng bay; nếu lượt mang tập `{A}` về **sau** lượt mang `{A,B}` thì tab Hán Việt hiện âm
 * của một cấu hình nguồn **không còn tồn tại**, và không một dấu hiệu nào báo sai vì
 * `hanVietPending` đã bị lượt về trước tắt đi.
 *
 * ⚠️ Cờ `hanVietRequested` **không** thay được vai trò này: nó trả lời *"đã phát lượt nào
 * chưa"* (chống gọi trùng lúc mount lại), còn số thứ tự trả lời *"lượt trả lời này còn là
 * lượt mới nhất không"*. Hai câu hỏi khác nhau, và Story 1.19 làm cả hai cùng cần thiết.
 */
let hanVietSequence = 0
/**
 * 🔴 `true` từ lúc lượt tra Hán Việt được phát đi tới lúc nó trả lời — trạng thái **THỨ
 * TƯ**, không nằm trong ba trạng thái của AC4.
 *
 * Vì sao nó phải tồn tại: AC4 đòi ba trạng thái **phân biệt được** *(có âm / đã tra mà không
 * không có âm / chưa lớp nào gắn)*. Trong khoảng chờ IPC, `hanViet === null` ⇒ bản đồ âm
 * rỗng ⇒ mọi ký tự rơi vào nhánh placeholder, và `layersLoaded` mặc định `true` ⇒ màn hình
 * khẳng định **dứt khoát và SAI** rằng cả Chương đều *"không rõ"*, rồi nháy sang dữ liệu
 * thật. Một trạng thái *"đang tra"* không phải một trong ba — nó là thứ phải nói ra
 * TRƯỚC khi ba trạng thái kia có nghĩa. Bắt ở lượt code review 2026-08-06.
 */
const hanVietPending = ref(false)

const activeTabState = ref<SourceTab>('original')
const viewModeState = ref<HanVietViewMode>('switch')

/**
 * 🔴 TRẦN RENDER của kiểu xem **song song** — Quyết định #7, Task 8.
 *
 * Đo THẬT (Playwright, headless Chromium, DOM `.hv-unit` của `SourceHanViet.vue` **ở thời
 * điểm Story 1.16** — tức MỘT `.hv-unit` cho MỖI KÝ TỰ; xem Completion Notes của story để có
 * bảng số đầy đủ ở cả ba mức):
 *
 * | Số ký tự Hán | Kiểu song song — tới frame đầu | Kiểu chuyển đổi — tới frame đầu |
 * |---|---|---|
 * | 5.000   | 163,7 ms   | 2,7 ms |
 * | 50.000  | 1.408,5 ms | 24,2 ms |
 * | 500.000 | 13.621,5 ms *(không chấp nhận được)* | 222,4 ms |
 *
 * Kiểu **chuyển đổi** rẻ hơn kiểu **song song** 60× ở 500.000 ký tự — đúng dự đoán của
 * Quyết định #7 *(một khối `white-space: pre-wrap` không sinh node cho mỗi ký tự)*.
 *
 * ⇒ **50.000 ký tự Hán** là trần của kiểu song song — 1,4 s là ranh giới còn CHẤP NHẬN
 * ĐƯỢC cho một thao tác CHẠY MỘT LẦN mỗi lượt nạp Chương (không phải đường nóng NFR1,
 * xem Quyết định #7); 13,6 s ở 500.000 thì không. Trên trần này, kiểu song song **không
 * không khả dụng** — người dùng vẫn đọc được bằng kiểu chuyển đổi (rẻ, tuyến tính).
 *
 * ⚠️ **Đếm THEO SỐ LẦN XUẤT HIỆN của ký tự Hán trong văn bản, không theo số ký tự Hán
 * DUY NHẤT** — chi phí render tỉ lệ với **độ dài văn bản**, không với kích cỡ tập ký tự
 * tra cứu.
 *
 * 🔴 **STORY 1.18b — VÌ SAO PHÉP ĐẾM GIỮ NGUYÊN DÙ MỆNH ĐỀ ĐỠ NÓ ĐÃ ĐỔI.** Bản trước nói
 * *"mỗi lần xuất hiện sinh một `.hv-unit` riêng"*. Câu đó **không còn đúng**: 1.18b gom ký
 * tự Hán thành **TỪ**, và một `.hv-unit` nay mang **cả từ**. [`hanCharOccurrenceCount`] vì
 * vậy **đếm nhiều hơn** số `.hv-unit` thật — đo 2026-08-07 trên văn bản tin tức tiếng Trung:
 * **0,642** node trên mỗi lần xuất hiện *(3.502 node / 5.000 ký tự · 34.714 / 50.000 —
 * cùng tỉ lệ ở cả hai mức)*, và chi phí dựng DOM giảm **~31 %** so với cấu trúc một-ký-tự.
 *
 * ⇒ Nó vẫn là một **proxy hợp lý và AN TOÀN**: nó *quá ước lượng* chi phí, không *dưới*
 * ước lượng, nên trần vẫn giữ đúng vai chặn. Đổi nó thành phép đếm TỪ sẽ bắt phải chạy
 * `Intl.Segmenter` trên **toàn Chương** chỉ để quyết định có cho phép kiểu song song hay
 * không — một lượt tách từ thừa trên đường nạp Chương *(và nó THẬT SỰ thừa: `buildSegments`
 * chỉ chạy khi tab Hán Việt được mở, còn phép đếm này chạy ở **mọi** lượt nạp Chương)*,
 * đổi lấy một con số chính xác hơn mà không quyết định nào cần tới.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 TRẦN NÀY CHỈ KHOÁ KIỂU SONG SONG — VÀ ĐƯỜNG LUI NAY KHÔNG CÒN RẺ NHƯ BẢNG TRÊN NÓI
 * ═════════════════════════════════════════════════════════════════════════════════
 * Bảng đo ở đầu doc-comment này đo kiểu **chuyển đổi** ở thời điểm Story 1.16, khi
 * `.hv-switch` là `<p>{{ switchText }}</p>` — **đúng MỘT text node**. Story 1.18b thay nó
 * bằng một `.hv-word` mỗi **TỪ** và một `.hv-syl` mỗi **KÝ TỰ**. Đo lại 2026-08-08 (lượt code
 * review; cùng bàn đo tái lập được số cũ — **23,6 ms** ở 50.000 so với **24,2 ms** của bảng
 * gốc, nên hai bên so được):
 *
 * | ký tự Hán | chuyển đổi **trước 1.18b** | chuyển đổi **sau 1.18b** |
 * |---|---|---|
 * | 5.000   | 3,1 ms · **1** node  | 41,5 ms · 13.728 node |
 * | 50.000  | 23,6 ms · **1** node | **532,9 ms** · 136.864 node *(gấp 23)* |
 * | 100.000 | 62,1 ms · **1** node | 1.038,5 ms · 273.728 node ⇒ ngoại suy **~5 s** ở 500.000 |
 *
 * ⚠️ Hệ quả phải nhìn thẳng: [`canUseParallelView`] khoá kiểu song song trên trần này, tức
 * Chương lớn bị **ép vào đúng bề mặt vừa nặng lên**. Câu *"người dùng vẫn đọc được bằng kiểu
 * chuyển đổi (rẻ, tuyến tính)"* ở trên vẫn đúng vế **tuyến tính**, và **không còn đúng** vế
 * *rẻ*.
 *
 * 🔴 **Ice chốt 2026-08-08: CHẤP NHẬN, ghi số, không đặt trần thứ hai** — một trần cho kiểu
 * chuyển đổi sẽ để Chương lớn **không còn đường xem nào**, và dựng `.hv-syl` theo yêu cầu là
 * một cơ chế mới đắt hơn khoản nó vá. Món nợ nằm ở `deferred-work.md` §code review 1-18b,
 * **kèm bảng số trên** để lượt sau không phải đo lại từ đầu.
 */
export const PARALLEL_VIEW_RENDER_CEILING = 50_000

/** Chương đang mở — `null` trước khi nạp xong hoặc khi chưa Tác phẩm nào mở. */
export const sourceChapter: DeepReadonly<Ref<OpenChapter | null>> = readonly(chapter)
/**
 * Lỗi gần nhất Rust trả lời cho lượt đọc Chương.
 *
 * `null` ở HAI ca khác hẳn nhau: đọc được, **hoặc** **không có cầu IPC nào** *(chạy
 * ngoài Tauri — `config/chapter.ts` nuốt có chủ ý)*. Khi **có** cầu IPC mà lượt gọi trượt,
 * giá trị này KHÁC `null`. *(Bản đầu của doc-comment này nói ngược — sửa ở lượt code
 * review 2026-08-06.)*
 */
export const sourceChapterError: DeepReadonly<Ref<IpcError | null>> = readonly(chapterError)
/** `true` trong khoảng chờ lượt tra âm Hán Việt — xem [`hanVietPending`]. */
export const sourceHanVietPending: DeepReadonly<Ref<boolean>> = readonly(hanVietPending)
/** Lỗi gần nhất Rust trả lời cho lượt tra âm Hán Việt. */
export const sourceHanVietError: DeepReadonly<Ref<IpcError | null>> = readonly(hanVietError)
export const activeTab: DeepReadonly<Ref<SourceTab>> = readonly(activeTabState)
export const viewMode: DeepReadonly<Ref<HanVietViewMode>> = readonly(viewModeState)

/**
 * `true` ⇔ **ít nhất một** lớp từ điển đang gắn — ca "0 lớp" là trạng thái BÌNH THƯỜNG có
 * tên (AD-25), khác với "đã tra mà ký tự không có âm" (AC4, ba trạng thái).
 *
 * ⚠️ Chỉ có nghĩa khi lượt tra **đã trả lời**. Trong khoảng chờ, và ở mọi ca lượt tra
 * TRƯỢT, `hanViet` là `null` — đọc giá trị này lúc đó là đọc một mặc định, không phải
 * một dữ kiện. Chỗ gọi phải xét [`sourceHanVietPending`] và [`sourceHanVietError`] TRƯỚC.
 */
export const layersLoaded = computed(() => hanViet.value?.layers_loaded ?? true)

/**
 * 🔴 Lượt tra Hán Việt đã trả lời và trả lời **được** — điều kiện để ba trạng thái của AC4
 * có nghĩa. `false` ⇔ đang chờ, hoặc lượt tra trượt, hoặc chưa từng chạy.
 *
 * ⚠️ Bản đầu không có vị từ này: một lượt IPC hỏng hoàn toàn trông **y hệt** một từ điển
 * đầy đủ không tra được chữ nào. Bắt ở lượt code review 2026-08-06.
 */
export const hanVietResolved = computed(
  () => !hanVietPending.value && hanVietError.value === null && hanViet.value !== null,
)

/** Các `dict_source.code` đã đóng góp âm cho lượt hiện tại — Quyết định #1, mệnh đề 3. */
export const sourcesUsed = computed<readonly string[]>(() => hanViet.value?.sources_used ?? [])

/**
 * Số LẦN XUẤT HIỆN của ký tự Hán trong Chương đang mở — biến số của
 * [`PARALLEL_VIEW_RENDER_CEILING`] (không phải số ký tự Hán duy nhất).
 */
const hanCharOccurrenceCount = computed(() => {
  const text = chapter.value?.source_text
  if (text === undefined || text === null) return 0
  let count = 0
  for (const ch of text) {
    if (isHanChar(ch)) count += 1
  }
  return count
})

/** `false` ⇔ Chương vượt [`PARALLEL_VIEW_RENDER_CEILING`] — kiểu song song bị khoá. */
export const canUseParallelView = computed(() => hanCharOccurrenceCount.value <= PARALLEL_VIEW_RENDER_CEILING)

/** Tra cứu O(1) ký tự → âm đã gom, dựng lại mỗi khi `hanViet` đổi. */
export const hanVietByChar = computed<ReadonlyMap<string, CharacterReading>>(() => {
  const map = new Map<string, CharacterReading>()
  if (hanViet.value !== null) {
    for (const c of hanViet.value.characters) map.set(c.character, c)
  }
  return map
})

async function ensureHanVietLoaded(sourceText: string): Promise<void> {
  if (hanVietRequested) return
  hanVietRequested = true

  // Đặt TRƯỚC `await` — xem [`hanVietSequence`].
  const mine = ++hanVietSequence

  const chars = Array.from(new Set(Array.from(sourceText).filter(isHanChar)))
  if (chars.length === 0) {
    // Không lượt IPC nào cho một Chương không một ký tự Hán nào — cùng luật
    // `read_senses(&[])`/`han_viet(&[])` phía Rust: tập rỗng không chạm database.
    hanViet.value = { characters: [], sources_used: [], layers_loaded: true }
    return
  }

  hanVietPending.value = true
  const { lookup, error } = await readHanViet(chars)

  // 🔴 Lượt này đã bị một lượt mới hơn vượt mặt — hoặc bởi [`resetSourcePanel`]. Thoát
  // **trước** mọi lượt gán, kể cả `hanVietPending`: lượt mới vẫn đang bay, và tắt cờ chờ hộ
  // nó là dựng lại đúng lỗ "màn hình khẳng định dứt khoát trong khi IPC còn chưa về" mà
  // [`hanVietPending`] tồn tại để bịt.
  //
  // ⚠️ Cũng **không** nhả `hanVietRequested` ở đây: lượt mới đang giữ cờ đó một cách hợp lệ,
  // và nhả hộ là mở cửa cho một lượt thứ ba chen vào.
  if (mine !== hanVietSequence) return

  hanVietPending.value = false
  hanViet.value = lookup
  hanVietError.value = error

  // 🔴 Một lượt TRƯỢT không được khoá vĩnh viễn đường tra. Bản đầu đặt `hanVietRequested`
  // trước `await` rồi không bao giờ nhả, nên một lỗi IPC nhất thời (hay một lượt chạy
  // ngoài Tauri) biến tab Hán Việt thành hỏng **mãi mãi**, kể cả khi `error.retryable`.
  // Nhả cờ ở đúng ca trượt là đủ: ca thành công vẫn idempotent như AC9 đòi.
  if (lookup === null) hanVietRequested = false
}

/**
 * Nạp Chương đang mở — **idempotent**: gọi nhiều lần (mỗi lượt `SourcePanel.vue` mount lại
 * sau một lượt đổi preset, AC9) chỉ chạy IPC ở lượt ĐẦU TIÊN.
 */
export async function ensureChapterLoaded(): Promise<void> {
  if (chapterRequested) return
  chapterRequested = true

  const { chapter: loaded, error } = await readOpenChapter()
  chapter.value = loaded
  chapterError.value = error

  if (loaded !== null && loaded.source_lang === 'zh') {
    void ensureHanVietLoaded(loaded.source_text)
  }
}

/**
 * 🔴 **STORY 1.19 · §Quyết định #3a, hệ quả thứ hai** — tra LẠI âm Hán Việt khi bộ lọc nguồn
 * đổi.
 *
 * Bộ lọc áp cho **cả** đường âm Hán Việt (`lookup_han_viet` nhận cùng tham số), nên giữ âm cũ
 * trên màn hình là để một nguồn *"đã tắt"* vẫn viết chữ lên tab Hán Việt — một câu tự mâu
 * thuẫn ngay trên màn hình. Và `priority_order` làm bộ lọc **ĐỔI ÂM**, không chỉ giấu bớt:
 * một ký tự rơi từ âm của lớp gỡ rời về âm của lớp nền.
 *
 * ⚠️ Nhả `hanVietRequested` rồi đi qua **đúng** [`ensureHanVietLoaded`] — không một đường
 * nạp thứ hai. Cờ đó là một cache **không có khoá vô hiệu hoá** (xem [`resetSourcePanel`]);
 * đây là lượt vô hiệu hoá thứ hai của nó, và nó phải đi cùng cửa với lượt đầu.
 *
 * Chương chưa nạp, hay nguồn không phải tiếng Trung ⇒ **không có gì để tra lại**, và đó không
 * phải một lỗi (cùng luật `selectSourceTab` khi tab không áp dụng).
 */
export async function refreshHanViet(): Promise<void> {
  const loaded = chapter.value
  if (loaded === null || loaded.source_lang !== 'zh') return
  hanVietRequested = false
  await ensureHanVietLoaded(loaded.source_text)
}

/** Handler thật của `source.select_tab_original`/`source.select_tab_han_viet` (AC6). */
export function selectSourceTab(tab: SourceTab): void {
  if (tab === 'han_viet' && chapter.value?.source_lang !== 'zh') {
    // Tab Hán Việt không tồn tại cho nguồn tiếng Anh (AC3) — thao tác không có hiệu lực,
    // không phải một lỗi: cùng luật `layout.toggle_*` khi panel không áp dụng.
    return
  }
  activeTabState.value = tab
}

/**
 * Handler thật của `source.toggle_han_viet_view` (AC6).
 *
 * 🔴 Chuyển SANG song song bị từ chối khi Chương vượt [`PARALLEL_VIEW_RENDER_CEILING`] —
 * không phải một lỗi, chỉ là thao tác không có hiệu lực (cùng luật `selectSourceTab`
 * khi tab không áp dụng). Chuyển VỀ chuyển đổi luôn được phép — nó không có trần.
 */
export function toggleHanVietView(): void {
  // Cùng guard với `selectSourceTab`: không có tab Hán Việt cho nguồn tiếng Anh (AC3),
  // nên lệnh này không có gì để đổi. Bản đầu thiếu dòng này, nên `Mod+Alt+J`/`Mod+Alt+V`
  // ở một Tác phẩm tiếng Anh lật `viewMode` **vô hình** — và state module-level đó sống
  // sót sang Chương sau. Bắt ở lượt code review 2026-08-06.
  if (chapter.value?.source_lang !== 'zh') return
  if (viewModeState.value === 'switch' && !canUseParallelView.value) return
  viewModeState.value = viewModeState.value === 'switch' ? 'parallel' : 'switch'
}

/**
 * 🔴 **Vứt toàn bộ state của Panel Source** — gọi khi Tác phẩm đang mở BỊ THAY.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO NÓ PHẢI TỒN TẠI
 * ─────────────────────────────────────────────────────────────────────────────
 * `chapterRequested`/`hanVietRequested` là cờ **module-level** — chúng sống sót qua một
 * lượt tháo/dựng lại component, và đó CHÍNH LÀ điều AC9 cần. Nhưng cùng cơ chế đó biến
 * chúng thành một **cache vĩnh viễn không có khoá vô hiệu hoá**: `replace_open_work`
 * phía Rust trỏ `OpenWorkState` sang Tác phẩm mới, còn panel không hay biết.
 *
 * Đường chạm là đường sản phẩm **bình thường**, không phải một ca biên: tạo Tác phẩm A
 * → Workspace → về Library → tạo Tác phẩm B → Workspace vẫn hiện nội dung **A**, âm Hán
 * Việt của **A**, và `source_lang` của **A** *(⇒ tab Hán Việt hiện/ẩn SAI)*. Không lỗi
 * nào, không cảnh báo nào. Bắt ở lượt code review 2026-08-06.
 *
 * ⚠️ Chỗ gọi duy nhất là `modes/libraryImport.ts::finishSubmit` — đúng một điểm nghẽn mà
 * **cả hai** nhánh nhập *(dán văn bản và tệp)* đều đi qua. Đừng rải lời gọi này ra.
 */
export function resetSourcePanel(): void {
  // 🔴 Vô hiệu hoá mọi lượt tra âm Hán Việt ĐANG BAY — cùng dòng và cùng lý do với
  // `resetLookupPanel()`: một promise về **sau** lượt reset sẽ làm âm của Tác phẩm A sống
  // lại dưới Tác phẩm B, đúng thứ hàm này tồn tại để ngăn. Nhả `hanVietRequested` một mình
  // là chưa đủ, vì nó không nói được lượt trả lời nào đã lỗi thời.
  hanVietSequence += 1

  chapter.value = null
  chapterError.value = null
  chapterRequested = false

  hanViet.value = null
  hanVietError.value = null
  hanVietRequested = false
  hanVietPending.value = false

  activeTabState.value = 'original'
  viewModeState.value = 'switch'
}
