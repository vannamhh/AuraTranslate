<script setup lang="ts">
// Bề mặt Hán Việt của Panel Source — Story 1.16, AC4 · AC6 · AC7 · Quyết định #1/#3/#4(a).
// Story 1.18b viết lại đơn vị của bề mặt này: **TỪ**, không còn **KÝ TỰ**.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 ĐƠN VỊ CỦA CẢ BỀ MẶT LÀ **MỘT TỪ** — VÀ ĐÓ LÀ MỘT LƯỢT PHÁ CÓ CHỦ ĐÍCH (Story 1.18b)
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 **Banner cũ tuyên bố "MỘT NODE CHO MỖI KÝ TỰ" là điều kiện tiên quyết của 1.18/3.4.
// Tuyên bố đó KHÔNG CÒN ĐÚNG.** Story 1.18b thay nó bằng: **một node cho mỗi TỪ**, ranh
// giới từ do [`wordStartOffsets`] quyết (`Intl.Segmenter`, ICU — cùng bộ tách từ mà trình
// duyệt dùng cho double-click trên tab nguyên văn).
//
// Vì sao phá: ở tab nguyên văn, double-click chọn **cả cụm** (`台湾`); ở tab Hán Việt nó chỉ
// chọn được **một ký tự**, và ở kiểu song song thì trả về `""` — hỏng hoàn toàn (Ice bắt
// bằng mắt 2026-08-07). Nguyên nhân là cấu trúc: mỗi ký tự một `.hv-unit` `inline-block`
// nên ICU **không bao giờ** với qua được biên node.
//
//   kiểu SONG SONG   → một `<ruby>` mỗi **TỪ**, base mang trọn cụm ký tự Hán
//   kiểu CHUYỂN ĐỔI  → một `.hv-word` mỗi **TỪ**, mỗi âm một `.hv-syl` nối bằng
//                      `WORD_JOINER` (`U+2060`, rộng 0) — khe hở nhìn thấy do **CSS** vẽ
//
// ⚠️ **Hệ quả bắt buộc, chỗ dễ vỡ nhất:** một segment `han` nay mang NHIỀU ký tự nguồn, nên
// *"bôi đen nửa từ"* từ **bất khả** trở thành **khả thi**. Cả `resolveParallel` lẫn
// `resolveSwitch` vì vậy phải **CẮT theo range**, không lấy trọn node — xem [`overlapsRange`]
// và [`sliceTextNode`]. Lượt review 1.18 đã bắt đúng lớp lỗi này **hai lần** ở nhánh `text`.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 VÙNG CHỌN — BA ĐIỀU KIỆN GIỮ AC6 CỦA STORY 1.16, ĐO LẠI TRÊN CẤU TRÚC MỚI
// ─────────────────────────────────────────────────────────────────────────────────
// AC6 cưỡng chế bằng một phép kiểm thật: `window.getSelection().toString()` trên một đoạn
// bôi đen phải bằng ĐÚNG chuỗi ký tự nguồn — không lẫn âm Hán Việt, không lẫn khoảng
// trắng chèn thêm, và (mới từ 1.18b) **không lẫn `U+2060`**:
//   1. `<rt>` mang `user-select: none` — nằm trong luồng chọn của trình duyệt nhưng KHÔNG
//      BAO GIỜ được thêm vào chuỗi đã chọn. Đo 2026-08-07: thiếu nó ⇒ `"台đài北"`, có ⇒
//      `"台北"`. Đây là hàng rào cho lượt COPY/PASTE của người dùng ở kiểu song song.
//   2. `resolveParallel()`/`resolveSwitch()` đọc node DOM TRỰC TIẾP, không `toString()`.
//      Đây là hàng rào cho TRUY VẤN TRA CỨU — đường riêng, vì `user-select` không ràng
//      buộc `Selection.modify()` trên WebKit (AC11/AC12 của 1.18).
//   3. 🔴 [`onCopy`] đổi `U+2060` về một dấu cách trước khi nó ra clipboard — **hàng rào
//      MỚI**, cho một ký tự mà chưa cơ chế nào trước 1.18b biết tới.
//   4. Template KHÔNG được để lọt một khoảng trắng/newline THẬT giữa hai phần tử
//      `<span>` liền nhau trong `v-for` — một khoảng trắng như vậy là một ký tự CHÈN
//      THÊM vào vùng chọn mà văn bản nguồn không có. Xem cách viết dính liền `><` dưới
//      template; dấu cách phân tách hai TỪ là thứ DUY NHẤT được chèn, và nó đi qua
//      [`WORD_SEPARATOR`] một cách tường minh.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 AC7 — `font-synthesis` PHẢI ĐƯỢC TIÊU THỤ, ĐỪNG BỎ SÓT DÒNG NÀY
// ─────────────────────────────────────────────────────────────────────────────────
// `deferred-work.md:133` ghi nguyên văn: "Bỏ sót dòng đó là cách lời giải này chết im
// lặng." Đây là NGƯỜI TIÊU THỤ ĐẦU TIÊN của token `source-hanviet` — xem `rt`
// trong `<style>` dưới đây, cả năm biến `--*-source-hanviet`, không chỉ `font-synthesis`.
import { computed, useTemplateRef } from 'vue'
import { t } from '../i18n'
import { useSelectionSurface } from './selectionContract'
import {
  canUseParallelView,
  hanVietByChar,
  hanVietResolved,
  isHanChar,
  layersLoaded,
  sourceHanVietPending,
  sourcesUsed,
} from './sourcePanelState'
import { everySourceOffForRoute } from './dictSourcesState'
import { WORD_JOINER, wordStartOffsets } from './wordBoundary'

const props = defineProps<{
  sourceText: string
  viewMode: 'switch' | 'parallel'
}>()

/**
 * 🔴 Chốt CHẶN THỨ HAI (Task 8) — `SourcePanel.vue` đã khoá nút đổi kiểu xem khi vượt
 * [`PARALLEL_VIEW_RENDER_CEILING`], nhưng bề mặt render **không tin** riêng chốt đó: nếu
 * `viewMode` prop vẫn là `'parallel'` cho một Chương đã vượt trần (ca lý thuyết — ví dụ
 * `viewMode` module-level giữ nguyên từ MỘT Chương nhỏ trước đó rồi Story sau cho phép
 * chuyển Chương giữa chừng), bề mặt vẫn RƠI VỀ chuyển đổi thay vì dựng 500.000 `.hv-unit`
 * và treo webview. Đây là hàng rào cuối, không phải hàng rào duy nhất.
 */
const effectiveViewMode = computed(() => (props.viewMode === 'parallel' && canUseParallelView.value ? 'parallel' : 'switch'))

/**
 * Một mẩu đã phân loại của văn bản nguồn — KHÔNG tách câu/đoạn (AD-4), chỉ phân loại
 * Hán/không-Hán/ngắt dòng để dựng bề mặt Hán Việt.
 *
 * 🔴 **Story 1.18b — nhánh `han` mang MỘT TỪ, không một ký tự** (Quyết định #2a).
 * `chars.length === readings.length` là bất biến của kiểu này: âm đọc vẫn tra theo **TỪNG
 * KÝ TỰ** qua `hanVietByChar` (bảng khoá theo ký tự, `sourcePanelState.ts:166` — story 1.18b
 * **không** chạm nó), chỉ việc **gom** là mới.
 *
 * ⚠️ AD-4 khoá ranh giới **SEGMENT (câu)**, tính một lần lúc nhập và mang `id`. Đây là ranh
 * giới **TỪ**, thuần trình bày, không lưu xuống đĩa, không `id` — khác đơn vị, khác vòng đời.
 */
type Segment =
  | { kind: 'break' }
  | { kind: 'text'; text: string }
  | { kind: 'han'; chars: string[]; readings: (string | null)[] }

/**
 * ⚠️ Ngắt dòng được CHUẨN HOÁ ở đây — Bẫy `deferred-work.md:527`: văn bản nhập từ tệp
 * Windows mang `\r\n`, và `\r` KHÔNG được hiện thành một ô trống trên màn hình. Đây là một
 * phép biến đổi TRÌNH BÀY thuần tuý, không phải một quy tắc nghiệp vụ — dữ liệu gốc
 * trong `chapter.source_text` KHÔNG bị đổi.
 *
 * 🔴 `\r\n?` → `\n`, **không** xoá trắng `\r`. Bản đầu xoá trắng, nên một tệp dùng `\r`
 * **đơn** làm ký tự kết dòng (khuôn Mac cổ điển) bị nối hết thành **một dòng duy nhất** —
 * và `core/segment/import.rs` không chuẩn hoá xuống dòng, nên không ai chặn phía
 * trước. Bắt ở lượt code review 2026-08-06.
 *
 * 🔴 **Và nó phải chạy TRƯỚC lượt tách từ** (Story 1.18b): `wordStartOffsets` trả chỉ số
 * theo đơn vị mã của chuỗi nó nhận, nên đưa chuỗi chưa chuẩn hoá vào là lệch đúng số ký tự
 * `\r` đã bỏ. `SourcePanel.vue:47` chuẩn hoá **cùng một biểu thức** cho tab nguyên văn ⇒
 * hai tab tách từ trên **cùng một chuỗi** — điều kiện cấu trúc của AC2.
 */
const NEWLINES = /\r\n?/g

function buildSegments(text: string): Segment[] {
  const normalized = text.replace(NEWLINES, '\n')
  const wordStarts = wordStartOffsets(normalized)

  const out: Segment[] = []
  let buffer = ''
  let word: { chars: string[]; readings: (string | null)[] } | null = null

  const flushText = (): void => {
    if (buffer !== '') {
      out.push({ kind: 'text', text: buffer })
      buffer = ''
    }
  }
  const flushWord = (): void => {
    if (word !== null) {
      out.push({ kind: 'han', chars: word.chars, readings: word.readings })
      word = null
    }
  }

  // ⚠️ Duyệt theo **điểm mã** (`for…of` trên chuỗi) nhưng đếm vị trí theo **đơn vị mã**
  // (`ch.length`) — `wordStartOffsets` trả chỉ số theo đơn vị mã, cùng đơn vị mà
  // `Range.startOffset` đếm. Hai trong bảy dải của `isHanChar` nằm ngoài BMP.
  let at = 0
  for (const ch of normalized) {
    const index = at
    at += ch.length

    if (ch === '\n') {
      flushText()
      flushWord()
      out.push({ kind: 'break' })
      continue
    }
    if (isHanChar(ch)) {
      flushText()
      // Ranh giới TỪ do ICU quyết. Ký tự Hán đầu tiên của một cụm luôn mở một từ mới; các
      // ký tự sau chỉ mở từ mới khi ICU nói đây là một điểm bắt đầu.
      if (word !== null && wordStarts.has(index)) flushWord()
      if (word === null) word = { chars: [], readings: [] }
      word.chars.push(ch)
      // 🔴 Âm vẫn tra theo TỪNG KÝ TỰ — `hanVietByChar` khoá theo ký tự và story này không
      // sửa nó. Ký tự thiếu âm để `null` ở ĐÚNG âm tiết của nó (Quyết định #4a): `台湾` với
      // `湾` thiếu âm cho ra `thai⁠·`, không phải `·` cho cả cụm — giấu âm đã có là làm
      // người đọc tưởng cả cụm không tra được.
      word.readings.push(hanVietByChar.value.get(ch)?.reading?.primary ?? null)
      continue
    }
    // Mẩu KHÔNG-Hán (dấu câu, số, chữ Latin) vẫn là một segment `text` như trước.
    flushWord()
    buffer += ch
  }
  flushText()
  flushWord()
  return out
}

const segments = computed(() => buildSegments(props.sourceText))

/**
 * 🔴 Ký tự GIỮ CHỖ cho một ký tự Hán không có âm — **một ký tự, không một câu**.
 *
 * Ice chốt ở lượt code review 2026-08-06, đúng đề xuất §Câu hỏi cho Ice #3 của story:
 * *"một ký tự giữ chỗ … **giữ được cột** ở chế độ song song"*. Bản đầu nhét nguyên câu
 * `t('panel.source.han_viet_unavailable')` vào **mỗi** ký tự, và vì không lớp từ điển
 * nào có trong git (AD-25) nên đó là trạng thái **mặc định của mọi bản build hôm nay**:
 * một Chương 3.000 ký tự cho ra ~45.000 ký tự `"chưa có dữ liệuchưa có dữ liệu…"`.
 *
 * ⚠️ Màu đi ở `rt` qua `--color-on-surface-variant`. Không `ornament` *(UX-DR5:
 * màu của NÉT, không bao giờ của chữ)*, không `opacity` *(UX-DR6)*.
 */
const READING_PLACEHOLDER = '·'

/**
 * 🔴 **DẤU CÁCH THẬT — và nó chỉ đứng giữa hai TỪ** (Story 1.18b, AC8).
 *
 * Đây là thứ DUY NHẤT làm ICU coi hai cụm âm Hán Việt là **hai từ**. Trong **cùng** một từ,
 * chỗ của nó là [`WORD_JOINER`] (rộng 0) + một khe do CSS vẽ. Hai vai, hai ký tự, một lý do:
 * đo 2026-08-07 cho thấy mọi khoảng trắng THẬT (`U+0020`, `U+00A0`, `U+2009`) đều **cắt** từ
 * với ICU, nên không ký tự nào vừa giữ được từ liền vừa vẽ được khe.
 */
const WORD_SEPARATOR = ' '

/**
 * Ký tự này có TỰ MANG khoảng cách hai bên không — tức đặt một âm đọc sát nó vẫn đọc được?
 *
 * 🔴 `true` cho khoảng trắng (hiển nhiên) và cho **dấu câu TOÀN RỘNG** của chữ Hán
 * (`，` `。` `《` `》` `：` `；` `！` `？` `、` …): một glyph toàn rộng chiếm trọn ô em nhưng
 * vẽ dấu ở một góc, nên phần còn lại của ô ĐÃ là khoảng trắng thị giác. Chèn thêm một dấu
 * cách ở đó đẩy dấu câu rời khỏi chữ — đúng thứ chú thích gốc của story 1.16 cảnh báo.
 *
 * 🔴 `false` cho chữ số và chữ Latin NỬA RỘNG (`8` `5` `A`): chúng không có phần đệm nào,
 * nên `会8月` cho ra `"hội8nguyệt"` nếu không tách. Ice báo 2026-08-07.
 *
 * ⚠️ Dải toàn rộng lấy theo khối Unicode, không liệt kê tay từng dấu: `U+3000–U+303F`
 * (CJK Symbols and Punctuation) · `U+FF00–U+FFEF` (Halfwidth and Fullwidth Forms — phần
 * TOÀN rộng nằm ở đây, `U+FF01–U+FF5E`) · `U+FE30–U+FE4F` (CJK Compatibility Forms).
 */
const FULLWIDTH_OR_SPACE = /[\s　-〿！-～︰-﹏]/
const selfSpacing = (ch: string | undefined): boolean =>
  ch === undefined || FULLWIDTH_OR_SPACE.test(ch)

/**
 * 🔴 **KIỂU CHUYỂN ĐỔI — `true` ⇔ segment `i` cần một [`WORD_SEPARATOR`] đứng trước.**
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO KHÔNG CÒN BẢNG `text`/`map`/`starts` PHẲNG (Story 1.18b, Quyết định #3a)
 * ─────────────────────────────────────────────────────────────────────────────
 * Trước 1.18b, `.hv-switch` là **một text node thuần** và ánh xạ ngược đi qua ba bảng phẳng
 * phải giữ đồng bộ tay. Cấu trúc `.hv-word`/`.hv-syl` mới **phá** bất biến "một text node"
 * ⇒ giữ ba bảng đó nghĩa là phải dịch giữa *"offset trong chuỗi phẳng"* và *"offset trong
 * node cụ thể"* ở **mọi** biên — đúng lớp lỗi off-by-one mà lượt review 1.18 đã bắt, nhân
 * lên vì nay có thêm `U+2060` **vô hình** nằm trong chuỗi.
 *
 * ⇒ Ánh xạ ngược nay **đọc thẳng DOM** ([`resolveSwitch`]), cùng khuôn với
 * [`resolveParallel`] — một lỗi sửa một lần ăn cả hai kiểu xem. Thứ duy nhất còn phải tính
 * trước là *"chỗ nào cần một dấu cách thật"*, và đó là bảng này.
 *
 * ⚠️ Luật giữ **nguyên văn** logic [`selfSpacing`] của 1.16/1.18, chỉ đổi **đơn vị áp**:
 * *giữa hai âm* → *giữa hai TỪ*.
 */
const switchLeads = computed<boolean[]>(() => {
  const leads: boolean[] = []
  let prevWasWord = false
  /** Ký tự CUỐI của mẩu `text` vừa đi qua — `null` nếu segment trước không phải `text`. */
  let pendingTextTail: string | undefined | null = null

  segments.value.forEach((seg, index) => {
    if (seg.kind === 'break') {
      leads[index] = false
      prevWasWord = false
      pendingTextTail = null
      return
    }
    if (seg.kind === 'text') {
      // 🔴 Vế thứ nhất của lượt sửa 2026-08-07 (Ice báo: `"hội8nguyệt5nhật"`). Dấu cách chỉ
      // chèn khi ký tự CHẠM VÀO ranh giới không tự mang khoảng cách — `，` `。` thì không,
      // `8` `5` `A` thì có. Xem [`selfSpacing`].
      leads[index] = prevWasWord && !selfSpacing(seg.text[0])
      prevWasWord = false
      pendingTextTail = seg.text[seg.text.length - 1]
      return
    }
    // Hai TỪ liền nhau luôn tách (không có mẩu `text` nào ở giữa vì văn xuôi tiếng Trung
    // không có khoảng trắng) — cộng vế đối xứng: mẩu `text` vừa đi qua kết thúc bằng một ký
    // tự không tự mang khoảng cách (`8`) và ngay sau là một từ (`nguyệt`) ⇒ cũng phải tách.
    leads[index] = prevWasWord || (pendingTextTail !== null && !selfSpacing(pendingTextTail))
    prevWasWord = true
    pendingTextTail = null
  })

  return leads
})

/**
 * Các âm của một TỪ, hiện dưới `<ruby>` ở kiểu **song song**.
 *
 * 🔴 **Dấu cách THẬT ở đây, KHÔNG `WORD_JOINER`** — và đây là một lệch có chủ ý so với chữ
 * của Task 3 (*"`<rt>` mang các âm nối `U+2060`"*), giữ nguyên **ý** của nó:
 *   ① `U+2060` tồn tại để một cụm âm **là một từ với ICU**, tức để **double-click trúng nó**.
 *      Ở kiểu song song, thứ người dùng double-click là **chữ Hán** (base của `<ruby>`),
 *      không phải `<rt>` — `<rt>` mang `user-select: none` và không phải mục tiêu chọn.
 *      Không vai nào ở đây cần ký tự nối.
 *   ② `<rt>` là **một** text node, nên CSS **không** vẽ được khe giữa hai âm trong nó ⇒ nối
 *      bằng `U+2060` cho ra `thailoan` dính liền — đúng thứ Ice bác nguyên văn ở 1.16
 *      (`phảnđốitrungcộngkhoác…`).
 *   ③ `user-select: none` **không** ràng buộc `Selection.modify()` trên WebKit (số đo AC12
 *      của 1.18) ⇒ `<rt>` **rò được** vào `toString()` trên engine đó. Rò một dấu cách còn
 *      lần ra được; rò một ký tự **vô hình** thì không (AC5).
 */
function readingLine(readings: (string | null)[]): string {
  return readings.map((r) => r ?? READING_PLACEHOLDER).join(WORD_SEPARATOR)
}

/**
 * 🔴 Trạng thái của **cả bề mặt**, hiện ra bằng MỘT dòng — không nhân theo số ký tự.
 *
 * AC4 đòi ba trạng thái phân biệt được, và `sourcePanelState.ts` bổ sung một trạng thái
 * **thứ tư** *(đang tra)* mà bản đầu gộp nhầm vào *"đã tra mà không có âm"*.
 */
const surfaceNoticeKey = computed<string | null>(() => {
  if (sourceHanVietPending.value) return 'panel.source.han_viet_pending'
  if (!hanVietResolved.value) return 'panel.source.han_viet_failed'
  if (!layersLoaded.value) return 'panel.source.han_viet_unavailable'
  // 🔴 **"MỌI NGUỒN ĐỀU TẮT" LÀ MỘT TRẠNG THÁI CÓ TÊN Ở ĐÂY NỮA** — Ice chốt ở code review
  // 2026-08-10. §Quyết định #3a áp bộ lọc nguồn cho **cả** đường âm Hán Việt, nên tắt hết
  // nguồn tiếng Trung làm **mọi** ký tự rơi về `READING_PLACEHOLDER` — không phân biệt được
  // với ca *"ký tự này thật sự không có âm ghi nhận"* *(một nguồn còn bật nhưng thiếu dữ
  // liệu cho ký tự đó)*. Panel Lookup đã có nhánh riêng cho đúng ca này từ đầu; sự bất đối
  // xứng giữa hai bề mặt cho cùng một nguyên nhân là chỗ người dùng đọc sai.
  //
  // ⚠️ Đường cố định `'zh'`, và đó là một dữ kiện chứ không một giả định: tab Hán Việt chỉ
  // tồn tại cho Chương `source_lang === 'zh'` (AC3, `selectSourceTab` chặn ca còn lại).
  if (everySourceOffForRoute('zh')) return 'panel.source.han_viet_all_sources_off'
  return null
})

const sourcesLine = computed(() => sourcesUsed.value.join(', '))

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 1.18 · AC2 · AC12 — BỀ MẶT HÁN VIỆT LÀ MỘT NGUỒN VÙNG CHỌN
// ═════════════════════════════════════════════════════════════════════════════════
const switchEl = useTemplateRef<HTMLElement>('switchEl')
const parallelEl = useTemplateRef<HTMLElement>('parallelEl')

/**
 * 🔴 **BỀ MẶT ĐĂNG KÝ — ĐÚNG đoạn văn bản đang hiện, KHÔNG bọc `.hv-surface`** (lượt review
 * 2026-08-07, AC11).
 *
 * Bản đầu đăng ký `.hv-surface` — cái div BỌC cả `.hv-notice`/`.hv-sources` (dòng trạng thái/
 * nguồn) LẪN đoạn văn bản. `focusSelectionSource()` gọi `range.selectNodeContents(source.el)`
 * rồi `collapse(true)` — tức đặt caret vào **con ĐẦU TIÊN** của phần tử đã đăng ký. Khi
 * `.hv-notice`/`.hv-sources` render (ca THƯỜNG, không ca biên — chúng hiện bất cứ khi nào có
 * dòng nguồn), caret rơi vào dòng trạng thái/nguồn, không vào văn bản — `Selection.modify()`
 * mở rộng từ đó đi qua đúng thứ không phải nội dung tra cứu trước.
 *
 * ⇒ Đăng ký đúng đoạn `<p>` đang hiện (`.hv-switch` hoặc `.hv-parallel`), cùng mẫu với
 * `.original` của `SourcePanel.vue`.
 */
const surface = computed<HTMLElement | null>(() =>
  effectiveViewMode.value === 'switch' ? switchEl.value : parallelEl.value,
)

/**
 * 🔴 **PHÉP THỬ PHỦ NHAU CHẶT — `intersectsNode` MỘT MÌNH LÀ THỪA MỘT KÝ TỰ** (1.18b · AC6).
 *
 * `Range.intersectsNode()` trả `true` cả khi vùng chọn chỉ **CHẠM** vào biên node: một cú kéo
 * dừng đúng ở đầu ô kế tiếp vẫn "giao" với ô đó. Khi một ô = một **ký tự** (trước 1.18b),
 * thứ đó chưa bao giờ lộ ra vì mọi vùng chọn đều dừng ở biên ô. Khi một ô = một **TỪ**, và
 * mỗi âm là một `.hv-syl` riêng, nó lộ ra ngay: chọn `thai` rồi thả đúng ở đầu `loan` cho
 * ra `台湾` thay vì `台` — **thừa một ký tự**, vỡ AC6.
 *
 * ⇒ Phủ nhau **CHẶT** (`>`/`<`, không `>=`/`<=`): `range.end` phải đứng **sau** đầu node
 * **và** `range.start` phải đứng **trước** cuối node. Chạm biên không tính.
 *
 * 🔴 **RANH GIỚI PHẢI ĐO Ở MỨC TEXT NODE, KHÔNG Ở MỨC PHẦN TỬ** — và đây là một số đo, không
 * một lo xa. Bản đầu dùng `selectNode()` cho mọi node, và lượt đo 2026-08-07 bắt ngay:
 *
 * | vùng chọn | mong đợi | `selectNode()` | ở mức text node |
 * |---|---|---|---|
 * | `đài` → dừng ở **đầu** `loan` | `台` | ❌ `台湾` | ✅ `台` |
 *
 * Lý do: `selectNode(el)` đặt ranh giới **NGOÀI** phần tử (toạ độ của cha), nên một điểm cuối
 * nằm **TRONG** phần tử ở offset 0 vẫn đứng *sau* ranh giới đó — dù nó phủ **không một ký
 * tự nào**. Đo ở mức text node thì hai điểm **bằng nhau**, và phép so chặt loại đúng.
 * `selectNodeContents()` **không** sửa được: `(span, 0)` vẫn đứng trước `(textNode, 0)`.
 *
 * ⚠️ Node **không** có đúng một text node con (`<br>`, `.hv-word` bọc các `.hv-syl`,
 * `.hv-unit` bọc `<ruby>`) rơi về `selectNode()`. Với `.hv-word`/`.hv-unit` điều đó **an
 * toàn** — chúng chỉ là **cổng vào**, độ chính xác thật nằm ở lớp trong (`.hv-syl` đo ở mức
 * text node · `<ruby>` cắt bằng `sliceTextNode`, vốn tự cho ra chuỗi rỗng khi range dừng ở
 * offset 0).
 *
 * 🔴 **`<br>` thì KHÔNG an toàn theo nghĩa đó, và bản đầu của chú thích này nói sai.** Nó
 * viết *"`<br>` không có nội dung để chạm hụt"*; lượt đo 2026-08-08 (code review) bác:
 *
 * | vùng chọn | `overlapsRange(range, br)` |
 * |---|---|
 * | **cuối** dòng trên → **đầu** dòng dưới | ❌ `true` — dù phủ **không một ký tự nào** |
 *
 * `<br>` có **0** `childNodes` nên rơi thẳng vào `selectNode()`, đúng nhánh gây ra lỗi thừa
 * ký tự ở trên, và không có "lớp trong" nào đỡ nó.
 *
 * ⇒ **Vì sao vẫn để nguyên:** chuỗi thừa duy nhất nó sinh ra là `'\n'`, và
 * `lookupPanelState.ts::runLookup` mở đầu bằng `if (trimmed === '') return` ⇒ một vùng chọn
 * chỉ chạm `<br>` **không phát lượt tra nào**; một `'\n'` nằm **giữa** vùng chọn thì đúng
 * bằng thứ người dùng đã kéo qua. Mã **trước** Story 1.18b cũng làm y hệt
 * (`range.intersectsNode(child)` trần, không tinh chỉnh biên) ⇒ **không** một hồi quy.
 * Sửa nó đòi một nhánh riêng cho node rỗng, đổi lấy **không** một thay đổi hành vi nào.
 *
 * ⚠️ `intersectsNode` vẫn chạy TRƯỚC như một bộ lọc rẻ: hàm này dựng một `Range` mỗi lần
 * gọi, và vòng lặp đi qua **mọi** con của bề mặt (tới 50.000 ở trần render). Lọc trước giữ
 * chi phí ở đúng số node mà vùng chọn thật sự đi qua — NFR1.
 */
function overlapsRange(range: Range, node: Node): boolean {
  if (!range.intersectsNode(node)) return false

  const nodeRange = document.createRange()
  const only = node.childNodes.length === 1 ? node.firstChild : null
  if (only !== null && only.nodeType === Node.TEXT_NODE) {
    nodeRange.setStart(only, 0)
    nodeRange.setEnd(only, only.textContent?.length ?? 0)
  } else {
    nodeRange.selectNode(node)
  }

  return (
    range.compareBoundaryPoints(Range.START_TO_END, nodeRange) > 0 &&
    range.compareBoundaryPoints(Range.END_TO_START, nodeRange) < 0
  )
}

/**
 * Phần văn bản của một phần tử **một-text-node** mà `range` thật sự phủ.
 *
 * 🔴 Lượt review 1.18 bắt được **hai lần** rằng lấy trọn `textContent` khi vùng chọn chỉ
 * chạm **một phần** là rò thêm ký tự người dùng không chọn (vỡ AC12). Dùng chung cho mẩu
 * `text` của **cả hai** kiểu xem và cho base của `<ruby>` — cùng một khuôn, một chỗ sửa.
 */
function sliceTextNode(range: Range, textNode: Node): string {
  const full = textNode.textContent ?? ''
  const from = textNode === range.startContainer ? range.startOffset : 0
  const to = textNode === range.endContainer ? range.endOffset : full.length
  return full.slice(from, to)
}

/**
 * 🔴 **KIỂU SONG SONG — ĐỌC NODE VĂN BẢN CỦA `<ruby>`, không TIN `Selection.toString()`.**
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO ĐỌC NODE, VÀ ĐÂY LÀ MỘT SỐ ĐO, KHÔNG PHẢI MỘT SỞ THÍCH (Story 1.18 · AC12)
 * ─────────────────────────────────────────────────────────────────────────────
 * AC6 của Story 1.16 đo bằng một cú **kéo chuột thật** (Playwright) và kết luận đúng:
 * âm đọc mang `user-select: none` nên `toString()` trả về đúng chuỗi ký tự nguồn.
 *
 * AC12 đòi chạy **LẠI** phép kiểm đó, không tin số đo cũ — và lượt chạy lại (2026-08-07,
 * hai engine) lật một nửa kết luận:
 *
 * | vùng chọn cả đoạn | Chromium | WKWebView (macOS) |
 * |---|---|---|
 * | `Selection.toString()` | ✅ đúng | ❌ `他tha打đả開khai…` — **rò âm Hán Việt** |
 * | đọc node `<ruby>`     | ✅ đúng | ✅ đúng |
 *
 * `user-select: none` chi phối vùng chọn do **NGƯỜI DÙNG KÉO** tạo ra. Nó **không ràng buộc**
 * `Selection.modify()` trên WebKit — và `Selection.modify()` chính là đường mà **AC11 của
 * 1.18** dựng để bôi đen bằng bàn phím. Tức: số đo của 1.16 vẫn đúng cho chuột, và 1.18 tự
 * tạo ra một đường thứ hai mà nó không đúng.
 *
 * ⇒ Đọc thẳng ký tự NGUỒN từ những `.hv-unit` mà vùng chọn chạm tới. Một nguồn sự thật,
 * đúng trên cả hai engine, và không phụ thuộc việc engine nào tôn trọng `user-select` ở đâu.
 *
 * 🔴 **STORY 1.18b — base của `<ruby>` nay là CẢ TỪ, nên nó CẮT ĐƯỢC.** Trước 1.18b hàm này
 * lấy **trọn** node văn bản của `<ruby>`, và điều đó hợp lệ vì một `<ruby>` = một ký tự nên
 * *"chọn nửa"* là bất khả. Một `<ruby>` = một **TỪ** làm nó khả thi **ngay lập tức** ⇒ phải
 * cắt theo range, y hệt nhánh `text`. Không cắt ⇒ bôi đen `台` trả về `台湾` ⇒ vỡ AC12.
 *
 * ⚠️ `user-select: none` ở `<rt>` **không được gỡ** — nó vẫn là thứ giữ cho vùng chọn
 * do chuột kéo *trông đúng* trên màn hình. Hàm này là hàng rào thứ hai, không phải bản thay.
 */
function resolveParallel(selection: Selection): string | null {
  const host = parallelEl.value
  if (host === null || selection.rangeCount === 0) return null

  const range = selection.getRangeAt(0)
  // Vùng chọn rỗng (một cú bấm) ⇒ không truy vấn nào — `intersectsNode` trả `true` cho một
  // range thu về một điểm nằm sát biên node, nên phải chặn TRƯỚC vòng lặp.
  if (range.collapsed) return null

  let out = ''
  for (const child of Array.from(host.children)) {
    if (!overlapsRange(range, child)) continue
    if (child.tagName === 'BR') {
      out += '\n'
      continue
    }
    // `.hv-unit` mang một `<ruby>`: node văn bản đầu là chuỗi KÝ TỰ NGUỒN của cả từ, theo
    // sau là `<rt>` mang các âm Hán Việt.
    //
    // 🔴 `ruby.textContent` GỘP CẢ `<rt>` — đo được 2026-08-07 trên Chromium:
    // `Selection.toString()` của một vùng chọn hai ký tự trả `"台đài北"`, không `"台北"`.
    // `<rt>` mang `user-select: none` (đo lại: trả đúng `"台北"`), nhưng hàm này KHÔNG dựa
    // vào đó — `user-select` không ràng buộc `Selection.modify()` trên WebKit (xem
    // doc-comment ở trên), và `textContent` thì bỏ qua nó hoàn toàn. ⇒ lấy đúng node văn
    // bản TRỰC TIẾP đầu tiên của `<ruby>`, không `textContent`.
    const rubyEl = child.querySelector('ruby')
    if (rubyEl !== null) {
      const baseNode = Array.from(rubyEl.childNodes).find((n) => n.nodeType === Node.TEXT_NODE)
      if (baseNode !== undefined) out += sliceTextNode(range, baseNode)
      continue
    }
    // 🔴 Một `<span>` trơn là mẩu KHÔNG-Hán *(dấu câu, chữ Latin — có thể NHIỀU ký tự)*. Lượt
    // review 2026-08-07 bắt được rằng `intersectsNode` trả `true` cho một lượt chạm MỘT PHẦN
    // — bôi đen dừng giữa đoạn này rồi lấy TRỌN `textContent` là rò thêm ký tự người dùng
    // không chọn, vỡ AC12. ⇒ cắt đúng phần range thật sự phủ, bằng chính node văn bản
    // (một `<span>` chỉ có đúng một text node con — xem template).
    const textNode = child.firstChild
    if (textNode === null || textNode.nodeType !== Node.TEXT_NODE) {
      // ⚠️ `Node.textContent` là `string | null` theo đúng chuẩn DOM. Lib DOM của TypeScript hẹp nó lại
      //    trên `Element`, nhưng nhánh này chạy trên một node lấy từ DOM thật.
      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
      out += child.textContent ?? ''
      continue
    }
    out += sliceTextNode(range, textNode)
  }
  return out === '' ? null : out
}

/**
 * 🔴 **KIỂU CHUYỂN ĐỔI — VIẾT LẠI HOÀN TOÀN Ở 1.18b, ĐỌC DOM CÙNG KHUÔN `resolveParallel`.**
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHÔNG SỬA HÀM NÀY = AUTO-LOOKUP **CHẾT HẲN** Ở KIỂU CHUYỂN ĐỔI
 * ─────────────────────────────────────────────────────────────────────────────
 * Bản trước đòi `.hv-switch` chứa **đúng một** text node (`range.startContainer !== node ⇒
 * null`). Cấu trúc `.hv-word`/`.hv-syl` của 1.18b **phá thẳng** bất biến đó, nên bản cũ sẽ
 * trả `null` cho **mọi** vùng chọn — không lỗi, không cổng nào đỏ, chỉ là tra cứu **im lặng
 * thôi chạy**. Đây là hồi quy nặng nhất story 1.18b có thể gây ra.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ÁNH XẠ NGƯỢC ĐI BẰNG **VỊ TRÍ**, KHÔNG BẰNG ÂM — mệnh đề này KHÔNG đổi từ 1.18
 * ─────────────────────────────────────────────────────────────────────────────
 * Bôi đen ở kiểu chuyển đổi phải tra **ký tự Hán nguồn**, không tra chuỗi âm Latin đang hiện
 * (chuỗi đó đi đường tiếng Anh của AD-44 và chắc chắn *"không tìm thấy"*). Và nó **không**
 * là một bảng tra âm→chữ: ánh xạ đó **đa trị** (`"lương"` → 良 · 涼 · 糧 · 量 · 粱…), giải nó
 * cần ngữ cảnh, tức một **quy tắc nghiệp vụ mới ở webview** — đúng thứ AD-1 cấm, và đúng
 * phần việc của **FR113 / Story 3.7**.
 *
 * ⇒ Đường đi là **vị trí trong DOM**: `host.children[i]` ứng **một-một** với
 * `segments.value[i]` (mỗi segment sinh **đúng một** phần tử — `<br>`, `.hv-text`, hoặc
 * `.hv-word`; dấu cách phân tách là **text node**, không phải phần tử, nên nó không lệch
 * bảng). Bên trong một `.hv-word`, `.hv-syl` thứ `j` ứng với `seg.chars[j]` — chính dữ liệu
 * đã dựng ra âm đó.
 *
 * 🔴 **Và đó là thứ làm "bôi đen NỬA TỪ" trả về đúng nửa** (AC6): mỗi âm là một phần tử
 * riêng, nên phép thử [`overlapsRange`] chạy ở mức **âm tiết**, không ở mức từ.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 CHỐT *"VÙNG CHỌN PHẢI NẰM TRỌN TRONG BỀ MẶT"* ĐÃ BỎ — CÓ CHỦ ĐÍCH (Ice chốt 2026-08-08)
 * ─────────────────────────────────────────────────────────────────────────────
 * Bản trước 1.18b có một chốt tường minh: `range.startContainer !== node ⇒ null`, kèm lý do
 * *"kéo qua dòng trạng thái, qua dòng nguồn thì không ánh xạ được, và `null` nghĩa là **không
 * phát lượt tra**, không phải **tra chuỗi âm**"*. Chốt đó **không còn**, và đây là lý do —
 * ghi ra vì lượt code review 2026-08-08 hỏi đúng câu này:
 *
 * ① Thứ chốt cũ sinh ra để chặn — *tra nhầm chuỗi âm Latin* — nay **bất khả về cấu trúc**:
 *    hàm này không đọc `toString()` một lần nào, nó đọc `seg.chars[j]`, tức **ký tự nguồn**.
 *    Kéo ra ngoài bề mặt cũng không moi được một chữ Latin nào vào truy vấn.
 * ② [`resolveParallel`] **chưa bao giờ** có chốt này (từ 1.18). Giữ nó ở riêng kiểu chuyển
 *    đổi là hai kiểu xem hành xử khác nhau ở cùng một thao tác — đúng thứ Quyết định #3a
 *    của 1.18b đặt ra để xoá.
 * ③ Hành vi mới là *"tra đúng phần ký tự Hán mà người dùng đã kéo qua"*, thay cho *"im lặng
 *    không tra gì"*. `surfaceFor()` vẫn chặn ở lớp trên: vùng chọn **bắt đầu** ngoài bề mặt
 *    thì không tới được đây.
 */
function resolveSwitch(selection: Selection): string | null {
  const host = switchEl.value
  if (host === null || selection.rangeCount === 0) return null

  const range = selection.getRangeAt(0)
  if (range.collapsed) return null

  const children = host.children
  let out = ''
  for (let i = 0; i < children.length; i += 1) {
    // ⚠️ `.item(i)` và `.at(i)`, KHÔNG `[i]`. Vòng lặp chạy theo `children.length` của DOM
    // trong khi `segments.value` là state của Vue: hai độ dài lệch nhau được đúng trong một
    // nhịp render, và đó là ca mà `continue` dưới đây tồn tại để bắt.
    //
    // Vì sao không chỉ chú thích `| undefined`: `HTMLCollection` khai chỉ số trả `Element`
    // *(không `| undefined`)* và mảng cũng vậy, nên TypeScript thu hẹp `const` theo giá trị
    // gán và bỏ qua chú thích — phép kiểm lại bị đọc thành mã dư. `.item()` trả
    // `Element | null` và `.at()` trả `T | undefined` từ chính chữ ký, nên kiểu nói thật ở
    // nguồn. `.item()` là API DOM có từ đời đầu; `.at()` nằm trong `lib: ES2022` mà
    // `tsconfig.json` đã khai.
    const child = children.item(i)
    const seg = segments.value.at(i)
    if (child === null || seg === undefined) continue
    if (!overlapsRange(range, child)) continue

    if (seg.kind === 'break') {
      out += '\n'
      continue
    }
    if (seg.kind === 'text') {
      const textNode = child.firstChild
      out +=
        textNode !== null && textNode.nodeType === Node.TEXT_NODE
          ? sliceTextNode(range, textNode)
          // ⚠️ Cùng lý do lượt đọc `textContent` phía trên.
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- xem chú thích ngay trên
          : (child.textContent ?? '')
      continue
    }
    const syllables = child.children
    for (let j = 0; j < syllables.length; j += 1) {
      // ⚠️ Cùng lý do và cùng cách khối trên: `syllables` là `HTMLCollection` của DOM,
      // `seg.chars` là dữ liệu đã tách âm; hai độ dài lệch nhau được, và `continue` chịu ca đó.
      const syllable = syllables.item(j)
      const char = seg.chars.at(j)
      if (syllable === null || char === undefined) continue
      if (!overlapsRange(range, syllable)) continue
      // 🔴 Một âm tiết là ATOM với ký tự nguồn của nó: chọn `th` của `thai` vẫn tra `台`.
      // Chia nhỏ hơn nữa là vô nghĩa — không có "nửa ký tự Hán".
      out += char
    }
  }
  return out === '' ? null : out
}

/**
 * Lấy truy vấn từ một vùng chọn trên bề mặt này — **hai kiểu xem, hai đường, một khuôn**.
 *
 * 🔴 Cả hai đều **đọc DOM**, không `toString()` — xem doc-comment của [`resolveParallel`].
 */
function resolveSelection(selection: Selection): string | null {
  return effectiveViewMode.value === 'parallel' ? resolveParallel(selection) : resolveSwitch(selection)
}

/**
 * 🔴 **`U+2060` KHÔNG ĐƯỢC RA CLIPBOARD** — Story 1.18b, AC5. Hàng rào **MỚI**: trước story
 * này không cơ chế nào trong dự án biết tới ký tự đó.
 *
 * Người dùng bôi đen ở kiểu **chuyển đổi** rồi `⌘C`. `Selection.toString()` mang trọn
 * `WORD_JOINER` — một ký tự **rộng bằng 0, không nhìn thấy được** — và nó dán ra Word,
 * ra ô tìm kiếm, ra một tin nhắn. Không ai lần ra được vì sao chuỗi *"trông đúng"* lại không
 * khớp với chính nó.
 *
 * 🔴 **Đổi về một dấu cách, KHÔNG xoá trắng.** Xoá trắng cho ra `thailoan` dính liền — đúng
 * thứ mà `WORD_JOINER` sinh ra để tránh trên màn hình, chỉ dời sang clipboard.
 *
 * ⚠️ Đặt trên `.hv-surface` (bọc ngoài), không trên từng đoạn: sự kiện `copy` nổi bọt, và
 * một handler phủ được **cả hai** kiểu xem. Kiểu song song không sinh `WORD_JOINER` nào, nên
 * ở đó nhánh `includes` thoát sớm và lượt copy đi đường mặc định của trình duyệt.
 */
function onCopy(event: ClipboardEvent): void {
  const selection = window.getSelection()
  if (selection === null) return

  const text = selection.toString()
  if (!text.includes(WORD_JOINER)) return

  const clipboard = event.clipboardData
  if (clipboard === null) return
  clipboard.setData('text/plain', text.replaceAll(WORD_JOINER, WORD_SEPARATOR))
  event.preventDefault()
}

useSelectionSurface(surface, 'source', resolveSelection)
</script>

<template>
  <div class="hv-surface" @copy="onCopy">
    <!-- 🔴 MỘT dòng cho trạng thái của CẢ bề mặt (đang tra / lượt tra trượt / chưa lớp
         nào gắn) — không nhân câu đó ra theo số ký tự. Xem `surfaceNoticeKey`. -->
    <p v-if="surfaceNoticeKey !== null" class="hv-notice">{{ t(surfaceNoticeKey) }}</p>
    <!-- aura-allow-text: `sourcesLine` là danh sách `dict_source.code` đã đóng góp cho
         lượt hiện tại (Quyết định #1, mệnh đề 3) — DỮ LIỆU, không chuỗi giao diện. -->
    <p v-if="sourcesLine !== ''" class="hv-sources">
      {{ t('panel.source.han_viet_sources_prefix') }} {{ sourcesLine }}
    </p>
    <!-- 🔴 STORY 1.18 · AC11 — `tabindex="0"` ĐÚNG TRÊN ĐOẠN VĂN BẢN, không trên `.hv-surface`
         bọc ngoài (lượt review 2026-08-07 — xem doc-comment của `surface` ở script). Cùng lý
         do và cùng giá với `.original` ở `SourcePanel.vue`.

         🔴 STORY 1.18b — `.hv-switch` KHÔNG còn là một text node thuần. Mỗi segment sinh
         ĐÚNG MỘT phần tử (`<br>` · `.hv-text` · `.hv-word`), theo đúng thứ tự của
         `segments`, vì `resolveSwitch()` ánh xạ ngược bằng CHỈ SỐ `host.children[i]`.
         Dấu cách phân tách là một **text node**, không phải phần tử ⇒ nó không lệch bảng.
         Thêm/bớt/đổi thứ tự một phần tử ở đây là làm truy vấn tra cứu sai im lặng.

         🔴 KHÔNG được để lọt một khoảng trắng THẬT nào giữa hai thẻ liền nhau — xem cách
         viết dính liền `><`. Dấu cách DUY NHẤT được phép là `WORD_SEPARATOR`, và nó đi qua
         một interpolation tường minh. -->
    <p
      v-if="effectiveViewMode === 'switch'"
      ref="switchEl"
      class="hv-switch"
      tabindex="0"
    ><template
        v-for="(seg, i) in segments"
        :key="i"
      ><!-- aura-allow-text: `WORD_SEPARATOR` là một dấu cách — ký tự PHÂN TÁCH hai TỪ cho
        ICU, tức HÌNH DẠNG của bề mặt, không một chuỗi giao diện dịch được. --><template
          v-if="switchLeads[i]"
        >{{ WORD_SEPARATOR }}</template><br v-if="seg.kind === 'break'" /><!-- aura-allow-text:
        mẩu KHÔNG-Hán của nguyên văn (khoảng trắng, dấu câu, chữ Latin xen giữa) — DỮ LIỆU
        của Tác phẩm. --><span
          v-else-if="seg.kind === 'text'"
          class="hv-text"
        >{{ seg.text }}</span><span
          v-else
          class="hv-word"
        ><!-- aura-allow-text: âm Hán Việt đã gom (DỮ LIỆU từ điển) hoặc ký tự giữ chỗ
        `READING_PLACEHOLDER`, kèm `WORD_JOINER` giữ cụm âm liền nhau với ICU. --><span
            v-for="(reading, j) in seg.readings"
            :key="j"
            class="hv-syl"
          >{{ (j === 0 ? '' : WORD_JOINER) + (reading ?? READING_PLACEHOLDER) }}</span
        ></span></template
    ></p>
    <p v-else ref="parallelEl" class="hv-parallel" tabindex="0"
      ><template
        v-for="(seg, i) in segments"
        :key="i"
      ><br v-if="seg.kind === 'break'" /><!-- aura-allow-text: mẩu KHÔNG-Hán của nguyên
        văn (khoảng trắng, dấu câu, chữ Latin xen giữa) — DỮ LIỆU của Tác phẩm. --><span
        v-else-if="seg.kind === 'text'"
      >{{ seg.text
      }}</span><span
        v-else
        class="hv-unit"
      ><!-- aura-allow-text: chuỗi ký tự Hán của MỘT TỪ trong nguyên
        văn — DỮ LIỆU. --><ruby>{{ seg.chars.join('')
      }}<!-- aura-allow-text: các âm Hán Việt đã gom của từ đó (DỮ LIỆU từ điển), ký tự
        thiếu âm hiện bằng `READING_PLACEHOLDER`. --><rt>{{ readingLine(seg.readings) }}</rt></ruby></span></template
    ></p>
  </div>
</template>

<style scoped>
/*
 * 🔴 VÙNG CUỘN — ĐỪNG BỎ `flex: 1` HAY `overflow: auto`.
 *
 * `.panel` của `PanelFrame.vue` mang `overflow: hidden`, và `.panel-body` không khai
 * `overflow` nào. Nhánh nguyên văn có vùng cuộn riêng (`.original` ở `SourcePanel.vue`);
 * bản đầu của bề mặt này thì không, nên **mọi thứ vượt chiều cao panel bị cắt và không
 * không với tới được bằng bất kỳ thao tác nào** — với trần 50.000 ký tự mà Quyết định #7
 * vừa chốt, đó là ≥99 % nội dung. Chính AC9 mở đầu bằng *"đã cuộn xuống"*, tức trạng thái
 * đó không tồn tại được. Bắt ở lượt code review 2026-08-06 bởi cả ba tầng review.
 */
.hv-surface {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

/*
 * Một dòng trạng thái cho CẢ bề mặt — không nhân theo số ký tự.
 *
 * 🔴 Story 1.17 · Quyết định #7 (Ice chốt 2026-08-06) — token thứ 17 `ui-md-wrap`
 * (12px/1.66/`wraps: true`) thay `ui-md` (12px/1.5, dưới sàn), đóng `deferred-work.md:115`.
 */
.hv-notice {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.hv-sources {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-primary);
  text-transform: uppercase;
}

/* Chuyển đổi — khối văn bản khai token source-hanviet của CHÍNH NÓ (AC1/AC7). */
.hv-switch {
  margin: 0;
  white-space: pre-wrap;
  font-family: var(--face-source-hanviet);
  font-size: var(--font-source-hanviet);
  font-style: var(--style-source-hanviet);
  font-synthesis: var(--synthesis-source-hanviet);
  line-height: var(--leading-source-hanviet);
  color: var(--color-on-surface);
}

/*
 * 🔴 KHE HỞ TRONG TỪ vs KHE HỞ GIỮA HAI TỪ — AC8 của Story 1.18b.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO KHE TRONG TỪ PHẢI DO CSS VẼ, KHÔNG DO MỘT KÝ TỰ
 * ─────────────────────────────────────────────────────────────────────────────
 * Mọi khoảng trắng THẬT đều **cắt từ** với ICU — đo 2026-08-07: `U+0020`, `U+00A0` và
 * `U+2009` đều cho ra `"thai"` (một âm) khi double-click. Chỉ `U+2060` (rộng 0) giữ được
 * cụm liền, và vì nó rộng 0 nên nó cho ra `"thailoan"` dính liền nếu không có gì khác.
 * ⇒ hai vai tách ra: **ký tự** giữ tính liền từ, **CSS** vẽ khoảng cách.
 *
 * 🔴 Và hai khe phải PHÂN BIỆT ĐƯỢC bằng mắt, nếu không thì ranh giới từ vô hình:
 *   trong từ   = `--space-unit`                        (4px)
 *   giữa hai từ = một dấu cách THẬT + `--space-unit`   (≈3,6px + 4px ở cỡ 14,5px)
 * ⇒ khe giữa hai từ rộng gần **gấp đôi** khe trong từ. `--space-unit` là đơn vị nền của bộ
 * token; không khai một token mới cho một thứ token sẵn có phục vụ đúng (AC8, Kiểm B2).
 *
 * ⚠️ `+` bỏ qua text node, nên `.hv-word + .hv-word` vẫn khớp dù có một dấu cách ở giữa.
 * Và nó KHÔNG khớp qua một mẩu `.hv-text` (dấu câu) — đúng ý: `，` không được bị đẩy ra xa
 * chữ, đó là cả lý do `selfSpacing()` tồn tại.
 */
.hv-syl + .hv-syl {
  margin-inline-start: var(--space-unit);
}

.hv-word + .hv-word {
  margin-inline-start: var(--space-unit);
}

/*
 * Song song — đoạn dài, mỗi TỪ một khối dọc (Quyết định #4a của 1.16, đơn vị đổi ở 1.18b).
 *
 * ⚠️ `line-height` DÙNG CHUNG token `source-cjk` với tab "Nguyên văn" thuần — và điều đó
 * chỉ đúng được kể từ khi âm đọc chuyển sang `<ruby>` (xem khối dưới): `<rt>` chiếm chỗ
 * THẬT trong layout nên trình duyệt tự tính chiều cao dòng, không cần một con số giãn thêm.
 * Đo 2026-08-07: không đè ở mọi mức đã thử, kể cả `normal`.
 *
 * 🔴 LỊCH SỬ — HAI TOKEN VÀ HAI LƯỢT VÁ ĐÃ BỊ GỠ, ghi lại để không ai dựng lại chúng:
 * lượt 1 kéo `line-height` lên **4.8** (token riêng `source-cjk-parallel`) để âm đọc
 * `position: absolute` thôi đè dòng sau; lượt 2 hạ xuống **3.2** bằng cách neo âm đọc vào
 * một `.hv-char` mang `line-height: normal`. Cả hai đều VÁ TRIỆU CHỨNG: gốc rễ là âm đọc
 * không chiếm chỗ trong layout. `<ruby>` bỏ được cả hai, nên token `source-cjk-parallel`
 * đã được **gỡ khỏi `tokens.json`** — bộ token về lại 17.
 */
.hv-parallel {
  margin: 0;
  white-space: pre-wrap;
  font-family: var(--face-source-cjk);
  font-size: var(--font-source-cjk);
  line-height: var(--leading-source-cjk);
  color: var(--color-on-surface);
}

/*
 * Vỏ của MỘT TỪ + các âm của nó. GIỮ NGUYÊN dù `<ruby>` bên trong đã tự lo bố cục —
 * `resolveParallel()` duyệt `host.children` và phân biệt "ô Hán" với "mẩu không-Hán" bằng
 * chính lớp này, nên nó là một mốc CẤU TRÚC, không một mốc trang trí.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 `display: inline`, KHÔNG `inline-block` — VÀ ĐÂY LÀ MỘT SỐ ĐO (1.18b · AC7)
 * ─────────────────────────────────────────────────────────────────────────────
 * `inline-block` dựng một **hộp dòng riêng** cho mỗi ô, và ICU không với qua biên hộp đó.
 * Trước 1.18b (một ô = một **ký tự**) chính tính chất ấy làm double-click trả về `""` —
 * đúng lỗi story này sinh ra để sửa. Gom thành một ô = một **TỪ** thì double-click chạy
 * lại được, nên `inline-block` **trông như** vô hại. Nó không vô hại. Đo 2026-08-07,
 * `Selection.modify('extend','right', …)` từ đầu đoạn, số ký tự nguồn thu được:
 *
 * | số lần bấm | 1 | 2 | 3 | 4 | 6 | 8 | 10 | 12 |
 * |---|---|---|---|---|---|---|---|---|
 * | `word` · cấu trúc CŨ (ô = ký tự) | 台 | 台 | 台湾 | 台湾 | 台湾地 | 台湾地方 | …议 | …议会 |
 * | `word` · `inline-block` (ô = từ) | 台湾 | 台湾 | 台湾 | 台湾 | 台湾 | 台湾 | 台湾 | 台湾 |
 * | `word` · **`inline`** (ô = từ) | 台湾 | 台湾 | 台湾地方 | 台湾地方 | 台湾地方议会 | …接连 | | |
 *
 * ⇒ `inline-block` làm **mở rộng vùng chọn theo TỪ bằng bàn phím KẸT HẲN** ở ô đầu tiên —
 * một hồi quy thẳng của AC11/1.18, và AC7 của 1.18b cấm đúng điều đó. `inline` gỡ được nó,
 * và còn **nhanh hơn cấu trúc cũ** (một lần bấm = một TỪ, không một ký tự).
 *
 * ⚠️ Và nó **không** đánh đổi gì: với `inline`, ICU đọc chuỗi ký tự nguồn **liền mạch** qua
 * các ô, nên double-click vẫn phủ đúng trọn từ (bảng đối chiếu 26 vị trí ở §Debug Log
 * References) — ranh giới từ là do **ICU**, không do biên node. Và `Selection.toString()`
 * của **cả đoạn** trả về đúng chuỗi nguồn: **0** ký tự `\n` chèn thêm, **0** ký tự Latin rò
 * từ `<rt>` (đo 2026-08-07). Bài học `inline-flex` của 1.16 là về thứ tạo **hộp dòng mới**;
 * `inline` không tạo hộp nào.
 *
 * 🔴 KHÔNG khai `min-width` ở đây nữa. Bản trước giãn ô theo độ dài âm
 * (`--hv-reading-len × 0.56em`) vì `.hv-reading` `position: absolute` không góp bề rộng —
 * nhưng nó đẩy các ký tự Hán rời xa nhau, làm kéo chọn một từ ghép rất khó nhắm (Ice báo
 * 2026-08-07). `<ruby>` tự nới ô vừa đúng bề rộng `<rt>`, nên chỗ giãn nay do
 * `padding-inline` của `<rt>` quyết — một chỗ, không hai.
 */
.hv-unit {
  display: inline;
}

/*
 * 🔴 ÂM ĐỌC ĐI BẰNG `<ruby>`/`<rt>` — Ice chốt 2026-08-07 sau BA lượt vá thất bại.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO BỎ `position: absolute` — VÀ VÌ SAO RÀNG BUỘC CŨ KHÔNG CÒN GIỮ NỮA
 * ─────────────────────────────────────────────────────────────────────────────
 * Story 1.16 đẩy `.hv-reading` ra khỏi luồng bằng `position: absolute` vì đo được
 * (Playwright) rằng `display: inline-flex; flex-direction: column` làm Chromium chèn một
 * `\n` vào `Selection.toString()` ở MỖI ranh giới `.hv-unit` — mọi thứ tạo một **hộp dòng
 * mới** đều bị. Lý lẽ đó ĐÚNG lúc nó được viết, và nó đã **hết hiệu lực**: `resolveParallel()`
 * nay đọc thẳng node DOM thay vì tin `toString()` (lượt sửa AC12) ⇒ chuỗi truy vấn không
 * còn phụ thuộc việc engine chèn `\n` hay không.
 *
 * 🔴 CÁI GIÁ CỦA `position: absolute` mà ba lượt vá liên tiếp mới lộ hết — âm đọc KHÔNG
 * chiếm chỗ trong layout, nên:
 *   ① không gì chừa chỗ cho nó ⇒ đè lên dòng sau (vá lần 1: kéo `line-height` lên 4.8);
 *   ② `top: 100%` neo theo hộp dòng đã giãn ⇒ âm đọc trôi xa ký tự của nó (vá lần 2: neo
 *      vào `.hv-char` mang `line-height: normal`, hạ được xuống 3.2);
 *   ③ hai thứ vá xong vẫn còn BA lỗi Ice bắt bằng mắt: vùng tô khi bôi đen trùm cả hộp
 *      dòng cao (trình duyệt tô theo hộp dòng, không theo glyph); âm đọc DÒNG CUỐI bị cắt
 *      và cuộn không tới (không gì đẩy `scrollHeight` ra); và `min-width` giãn ô theo độ
 *      dài âm làm chữ Hán rời rạc, kéo chọn từ ghép rất khó nhắm.
 *
 * ⇒ `<ruby>` là cơ chế trình duyệt sinh ra ĐÚNG cho việc này. Đo 2026-08-07 (Chromium):
 * âm đọc dòng cuối cách đáy vùng cuộn 31px và cuộn tới được (① + dòng cuối: XONG); vùng
 * tô ôm sát glyph (②: XONG); chữ Hán liền nhau tự nhiên (③: XONG); và **không đè ở MỌI
 * mức `line-height` đã thử — kể cả `normal`** ⇒ token `source-cjk-parallel` của vá lần 1/2
 * trở nên THỪA và đã được gỡ, `.hv-parallel` về lại `source-cjk` như tab thuần.
 *
 * ⚠️ `<rt>` PHẢI mang `user-select: none`: đo được `Selection.toString()` trả `"台đài北"`
 * (lẫn âm đọc) khi thiếu nó, `"台北"` khi có. Đây là hàng rào cho lượt **copy/paste của
 * người dùng**; hàng rào cho **truy vấn tra cứu** là `resolveParallel()` đọc node trực
 * tiếp — hai đường khác nhau, cần cả hai.
 *
 * ⚠️ `padding-inline` trên `<rt>` là thứ giữ cho âm đọc TÁCH BẠCH (Ice: *"âm hán việt
 * không được đè lên nhau, phải có khoảng cách để đọc"*). Nó nới bề rộng ô ruby ⇒ chữ Hán
 * giãn ra theo — Ice chốt chấp nhận (*"chữ hán xa nhau cũng được"*). Không có nó, âm đọc
 * dính thành một chuỗi liền: `phảnđốitrungcộngkhoác…` (đo thật). Từ 1.18b nó tách hai
 * **TỪ**, còn các âm TRONG một từ tách nhau bằng `WORD_SEPARATOR` — xem [`readingLine`].
 */
ruby {
  /* ⚠️ Tiền tố `-webkit-` đứng TRƯỚC bản chuẩn, có chủ ý: WKWebView (macOS — một trong hai
     engine đích, NFR14) có lịch sử chỉ nhận `-webkit-ruby-position`, và môi trường đo của
     lượt sửa này chỉ có Chromium nên vế đó CHƯA nghiệm thu được. Bản chuẩn viết sau để nó
     thắng ở engine hiểu cả hai. Nếu đo được trên WKWebView mà bản chuẩn chạy, gỡ dòng
     tiền tố — đừng để nó nằm lại như một lời khấn. */
  -webkit-ruby-position: under;
  ruby-position: under;
}

rt {
  /* 🔴 AC6 — loại khỏi vùng chọn, KHÔNG bao giờ đi vào `window.getSelection()`. */
  user-select: none;
  /* 🔴 Khoảng cách giữa hai ô ruby liền nhau — xem khối trên. `--space-unit` (4px) là
     đơn vị nền của bộ token và đúng bằng con số đã đo, nên KHÔNG khai một token mới:
     `EXPECTED_SPACING` là bảng ĐÓNG BĂNG, thêm hàng vào đó cần một mục `deviations`
     cho một thứ mà token sẵn có đã phục vụ đúng. Khe thị giác = 2× giá trị này. */
  padding-inline: var(--space-unit);
  /* 🔴 AC7 — người tiêu thụ ĐẦU TIÊN của `source-hanviet`. Năm biến, không chỉ synthesis. */
  font-family: var(--face-source-hanviet);
  font-size: var(--font-source-hanviet);
  font-style: var(--style-source-hanviet);
  font-synthesis: var(--synthesis-source-hanviet);
  line-height: var(--leading-source-hanviet);
  color: var(--color-on-surface-variant);
}
</style>
