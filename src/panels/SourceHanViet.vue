<script setup lang="ts">
// Bề mặt Hán Việt của Panel Source — Story 1.16, AC4 · AC6 · AC7 · Quyết định #1/#3/#4(a).
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 CHẾ ĐỘ SONG SONG: MỘT NODE CHO MỖI KÝ TỰ — VÀ VÙNG CHỌN LÀ ĐIỀU KIỆN TIÊN QUYẾT
// CỦA STORY 1.18/3.4
// ─────────────────────────────────────────────────────────────────────────────────
// Quyết định #4(a): mỗi ký tự Hán là một khối dọc `chữ trên / âm dưới`. AC6 cưỡng chế
// bằng một phép kiểm thật: `window.getSelection().toString()` trên một đoạn bôi đen phải
// bằng ĐÚNG chuỗi ký tự nguồn — không lẫn âm Hán Việt, không lẫn khoảng trắng chèn
// thêm. Hai điều kiện giữ mệnh đề đó:
//   1. `.hv-reading` mang `user-select: none` — nằm trong luồng chọn của trình duyệt
//      nhưng KHÔNG BAO GIỜ được thêm vào chuỗi đã chọn.
//   2. Template KHÔNG được để lọt một khoảng trắng/newline THẬT giữa hai phần tử
//      `<span>` liền nhau trong `v-for` — một khoảng trắng như vậy là một ký tự CHÈN
//      THÊM vào vùng chọn mà văn bản nguồn không có. Xem cách viết dính liền `><` dưới
//      template.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 AC7 — `font-synthesis` PHẢI ĐƯỢC TIÊU THỤ, ĐỪNG BỎ SÓT DÒNG NÀY
// ─────────────────────────────────────────────────────────────────────────────────
// `deferred-work.md:133` ghi nguyên văn: "Bỏ sót dòng đó là cách lời giải này chết im
// lặng." Đây là NGƯỜI TIÊU THỤ ĐẦU TIÊN của token `source-hanviet` — xem `.hv-reading`
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

/** Một mẩu đã phân loại của văn bản nguồn — KHÔNG tách câu/đoạn (AD-4), chỉ phân loại
 * Hán/không-Hán/ngắt dòng để dựng bề mặt Hán Việt. */
type Segment =
  | { kind: 'break' }
  | { kind: 'text'; text: string }
  | { kind: 'han'; char: string; reading: string | null }

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
 */
const NEWLINES = /\r\n?/g

function buildSegments(text: string): Segment[] {
  const out: Segment[] = []
  let buffer = ''
  const flush = (): void => {
    if (buffer !== '') {
      out.push({ kind: 'text', text: buffer })
      buffer = ''
    }
  }
  for (const ch of Array.from(text.replace(NEWLINES, '\n'))) {
    if (ch === '\n') {
      flush()
      out.push({ kind: 'break' })
      continue
    }
    if (isHanChar(ch)) {
      flush()
      const hit = hanVietByChar.value.get(ch)
      out.push({ kind: 'han', char: ch, reading: hit?.reading?.primary ?? null })
      continue
    }
    buffer += ch
  }
  flush()
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
 * ⚠️ Màu đi ở `.hv-reading` qua `--color-on-surface-variant`. Không `ornament` *(UX-DR5:
 * màu của NÉT, không bao giờ của chữ)*, không `opacity` *(UX-DR6)*.
 */
const READING_PLACEHOLDER = '·'

/**
 * Kiểu **chuyển đổi** — một khối văn bản THUẦN, `white-space: pre-wrap`, không sinh node cho
 * mỗi ký tự (Quyết định #7: rẻ hơn nhiều bậc độ lớn so với kiểu song song).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 CÁC ÂM PHẢI CÁCH NHAU BẰNG MỘT KHOẢNG TRẮNG — ĐỪNG `.join('')`
 * ─────────────────────────────────────────────────────────────────────────────
 * Văn xuôi tiếng Trung **không có** khoảng trắng giữa các ký tự, nên mọi cụm Hán liền
 * nhau đi vào đây với mẩu `text` **rỗng** ở giữa. Bản đầu nối bằng chuỗi rỗng và cho ra
 * `北涼 → "bắclương"` — trong khi `EXPERIENCE.md:410`, ví dụ **trụ cột** của FR113 mà chính
 * story trích, viết `北涼 → **Bắc Lương**`; mockup `key-screen-workspace.html:99` cũng ghi
 * `tha đả khai liễu na phiến môn, …` có dấu cách. Một âm tiết tiếng Việt phân tách bằng
 * khoảng trắng **là** điều kiện để nó đọc được. Bắt ở lượt code review 2026-08-06.
 *
 * ⚠️ Khoảng trắng chỉ chèn giữa **hai âm liền nhau**, không sau một mẩu `text` (dấu câu,
 * chữ Latin) vốn đã tự mang khoảng cách của nó — nếu không, `，` sẽ bị đẩy ra khỏi chữ.
 */
/**
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 STORY 1.18 — BẢNG ÁNH XẠ NGƯỢC **THEO VỊ TRÍ**, KHÔNG THEO ÂM
 * ─────────────────────────────────────────────────────────────────────────────
 * Ice chốt 2026-08-07: bôi đen ở kiểu **chuyển đổi** phải tra **ký tự Hán nguồn**, không tra
 * chuỗi âm Latin đang hiện trên màn hình (chuỗi đó đi đường tiếng Anh của AD-44 và chắc
 * chắn *"không tìm thấy"*).
 *
 * 🔴 **VÀ NÓ không PHẢI MỘT BẢNG TRA ÂM → CHỮ.** Một bảng như vậy là **đa trị** và không giải
 * được: `"lương"` ứng với 良 · 涼 · 糧 · 量 · 粱… — chọn một trong số đó cần ngữ cảnh, tức
 * một **quy tắc nghiệp vụ mới đặt ở webview**, đúng thứ AD-1 cấm, và đúng phần việc của
 * **FR113 / Story 3.7**.
 *
 * ⇒ Đường đi ở đây là **vị trí**: `switchText` được dựng **xác định** từ `segments`, nên
 * dựng kèm một bảng `vị trí trong chuỗi ra → chỉ số segment` cùng lượt `computed` đã có là
 * **O(n)**, không node mới, không bảng tra, không quy tắc nghiệp vụ. Ký tự nguồn đọc thẳng từ
 * `segments[i].char` — chính dữ liệu đã dựng ra âm đó.
 *
 * ⚠️ Bảng đếm theo **đơn vị mã UTF-16**, không theo ký tự Unicode: `Range.startOffset` của DOM
 * đếm bằng đơn vị mã, và một âm Hán Việt là chữ Latin có dấu (có thể nhiều đơn vị mã).
 */
/**
 * ⚠️ `starts[i]` — vị trí trong `text` nơi PHẦN CỦA CHÍNH segment `i` bắt đầu *(không tính
 * khoảng trắng phân tách attribute ngược cho segment `i - 1` ở dưới)*. Lượt review 2026-08-07
 * thêm bảng này: `resolveSelection` cần nó để CẮT `seg.text` đúng biên khi vùng chọn dừng
 * giữa một đoạn `text` nhiều ký tự, thay vì lấy trọn cả đoạn (AC12).
 */
const switchView = computed<{ text: string; map: number[]; starts: number[] }>(() => {
  let out = ''
  const map: number[] = []
  const starts: number[] = []
  let prevWasReading = false

  const push = (piece: string, segIndex: number): void => {
    out += piece
    for (let i = 0; i < piece.length; i += 1) map.push(segIndex)
  }

  segments.value.forEach((seg, index) => {
    if (seg.kind === 'break') {
      starts[index] = out.length
      push('\n', index)
      prevWasReading = false
      return
    }
    if (seg.kind === 'text') {
      starts[index] = out.length
      push(seg.text, index)
      prevWasReading = false
      return
    }
    // Khoảng trắng phân tách hai âm liền nhau thuộc về segment ĐỨNG TRƯỚC: bôi đen trúng
    // nó thì ký tự nguồn gần nhất vẫn đúng, không rơi ra ngoài bảng.
    if (prevWasReading) push(' ', index - 1)
    starts[index] = out.length
    push(seg.reading ?? READING_PLACEHOLDER, index)
    prevWasReading = true
  })

  return { text: out, map, starts }
})

const switchText = computed(() => switchView.value.text)

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
 * 🔴 **KIỂU SONG SONG — ĐỌC CÁC NODE `.hv-char`, không TIN `Selection.toString()`.**
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * VÌ SAO ĐỔI, VÀ ĐÂY LÀ MỘT SỐ ĐO, KHÔNG PHẢI MỘT SỞ THÍCH (Story 1.18 · AC12)
 * ─────────────────────────────────────────────────────────────────────────────
 * AC6 của Story 1.16 đo bằng một cú **kéo chuột thật** (Playwright) và kết luận đúng:
 * `.hv-reading` mang `user-select: none` nên `toString()` trả về đúng chuỗi ký tự nguồn.
 *
 * AC12 đòi chạy **LẠI** phép kiểm đó trong story này, không tin số đo cũ — và lượt chạy lại
 * (2026-08-07, hai engine) lật một nửa kết luận:
 *
 * | vùng chọn cả đoạn | Chromium | WKWebView (macOS) |
 * |---|---|---|
 * | `Selection.toString()` | ✅ đúng | ❌ `他tha打đả開khai…` — **rò âm Hán Việt** |
 * | đọc node `.hv-char`    | ✅ đúng | ✅ đúng |
 *
 * `user-select: none` chi phối vùng chọn do **NGƯỜI DÙNG KÉO** tạo ra. Nó **không ràng buộc**
 * `Selection.modify()` trên WebKit — và `Selection.modify()` chính là đường mà **AC11 của
 * story này** vừa dựng để bôi đen bằng bàn phím. Tức: số đo của 1.16 vẫn đúng cho chuột,
 * và story này tự tạo ra một đường thứ hai mà nó không đúng.
 *
 * ⇒ Đọc thẳng ký tự NGUỒN từ những `.hv-unit` mà vùng chọn chạm tới. Một nguồn sự thật,
 * đúng trên cả hai engine, và không phụ thuộc việc engine nào tôn trọng `user-select` ở đâu.
 *
 * ⚠️ `user-select: none` ở `.hv-reading` **không được gỡ** — nó vẫn là thứ giữ cho vùng chọn
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
    if (!range.intersectsNode(child)) continue
    if (child.tagName === 'BR') {
      out += '\n'
      continue
    }
    // `.hv-unit` mang hai con: `.hv-char` (ký tự nguồn, MỘT ký tự — không cắt được nửa) và
    // `.hv-reading` (âm — không lấy).
    const charEl = child.querySelector('.hv-char')
    if (charEl !== null) {
      out += charEl.textContent ?? ''
      continue
    }
    // 🔴 Một `<span>` trơn là mẩu KHÔNG-Hán *(dấu câu, chữ Latin — có thể NHIỀU ký tự)*. Lượt
    // review 2026-08-07 bắt được rằng `intersectsNode` trả `true` cho một lượt chạm MỘT PHẦN
    // — bôi đen dừng giữa đoạn này rồi lấy TRỌN `textContent` là rò thêm ký tự người dùng
    // không chọn, vỡ AC12. ⇒ cắt đúng phần range thật sự phủ, bằng chính node văn bản
    // (một `<span>` chỉ có đúng một text node con — xem template).
    const textNode = child.firstChild
    if (textNode === null || textNode.nodeType !== Node.TEXT_NODE) {
      out += child.textContent ?? ''
      continue
    }
    const len = textNode.textContent?.length ?? 0
    const from = textNode === range.startContainer ? range.startOffset : 0
    const to = textNode === range.endContainer ? range.endOffset : len
    out += (textNode.textContent ?? '').slice(from, to)
  }
  return out === '' ? null : out
}

/**
 * Lấy truy vấn từ một vùng chọn trên bề mặt này — **hai kiểu xem, hai đường**.
 *
 * 🔴 Kiểu SONG SONG: [`resolveParallel`] — đọc node, không `toString()`.
 * 🔴 Kiểu CHUYỂN ĐỔI: ánh xạ ngược theo VỊ TRÍ — xem [`switchView`].
 */
function resolveSelection(selection: Selection): string | null {
  if (effectiveViewMode.value === 'parallel') return resolveParallel(selection)

  const host = switchEl.value
  if (host === null || selection.rangeCount === 0) return null

  // `.hv-switch` chứa đúng MỘT node văn bản (một lượt interpolation). Vùng chọn phải nằm
  // trọn trong nó — nếu không (kéo qua dòng trạng thái, qua dòng nguồn) thì không ánh xạ được, và
  // `null` nghĩa là *"không phát lượt tra"*, không phải *"tra chuỗi âm"*.
  const node = host.firstChild
  const range = selection.getRangeAt(0)
  if (node === null || range.startContainer !== node || range.endContainer !== node) return null

  const { text, map, starts } = switchView.value
  const start = Math.max(0, Math.min(range.startOffset, text.length))
  const end = Math.max(start, Math.min(range.endOffset, text.length))
  if (end <= start) return null

  const firstSeg = map[start]
  const lastSeg = map[end - 1]
  if (firstSeg === undefined || lastSeg === undefined) return null

  let out = ''
  for (let i = firstSeg; i <= lastSeg; i += 1) {
    const seg = segments.value[i]
    if (seg === undefined) continue
    if (seg.kind === 'han') out += seg.char
    else if (seg.kind === 'text') {
      // 🔴 Vùng chọn có thể dừng GIỮA một đoạn `text` nhiều ký tự (dấu câu, chữ Latin) — lượt
      // review 2026-08-07 bắt được rằng lấy trọn `seg.text` ở đây rò thêm ký tự không được
      // chọn, vỡ AC12. Cắt về đúng phần [start, end) phủ lên segment này, quy về toạ độ cục
      // bộ qua `starts[i]`.
      const segStart = starts[i] ?? 0
      const from = i === firstSeg ? Math.max(0, start - segStart) : 0
      const to = i === lastSeg ? Math.min(seg.text.length, end - segStart) : seg.text.length
      out += seg.text.slice(from, to)
    } else out += '\n'
  }
  return out === '' ? null : out
}

useSelectionSurface(surface, 'source', resolveSelection)
</script>

<template>
  <div class="hv-surface">
    <!-- 🔴 MỘT dòng cho trạng thái của CẢ bề mặt (đang tra / lượt tra trượt / chưa lớp
         nào gắn) — không nhân câu đó ra theo số ký tự. Xem `surfaceNoticeKey`. -->
    <p v-if="surfaceNoticeKey !== null" class="hv-notice">{{ t(surfaceNoticeKey) }}</p>
    <!-- aura-allow-text: `sourcesLine` là danh sách `dict_source.code` đã đóng góp cho
         lượt hiện tại (Quyết định #1, mệnh đề 3) — DỮ LIỆU, không chuỗi giao diện. -->
    <p v-if="sourcesLine !== ''" class="hv-sources">
      {{ t('panel.source.han_viet_sources_prefix') }} {{ sourcesLine }}
    </p>
    <!-- aura-allow-text: kết quả GOM âm Hán Việt của Chương (`lookup_han_viet`) — DỮ LIỆU
         từ điển, không chuỗi giao diện. Ký tự không có âm hiện bằng `READING_PLACEHOLDER`
         (một ký tự `·`, không phải một câu — xem doc-comment của hằng đó).

         🔴 STORY 1.18 · AC11 — `tabindex="0"` ĐÚNG TRÊN ĐOẠN VĂN BẢN, không trên `.hv-surface`
         bọc ngoài (lượt review 2026-08-07 — xem doc-comment của `surface` ở script). Cùng lý
         do và cùng giá với `.original` ở `SourcePanel.vue`. -->
    <p v-if="effectiveViewMode === 'switch'" ref="switchEl" class="hv-switch" tabindex="0">{{ switchText }}</p>
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
        :style="{ '--hv-reading-len': (seg.reading ?? READING_PLACEHOLDER).length }"
      ><!-- aura-allow-text: một ký tự Hán của nguyên
        văn — DỮ LIỆU. --><span class="hv-char">{{ seg.char
      }}</span><!-- aura-allow-text: âm Hán Việt đã gom (DỮ LIỆU từ điển) hoặc ký tự giữ
        chỗ `READING_PLACEHOLDER`. --><span class="hv-reading">{{ seg.reading ?? READING_PLACEHOLDER }}</span></span></template
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

/* Chuyển đổi — khối văn bản thuần khai token source-hanviet của CHÍNH NÓ (AC1/AC7). */
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
 * Song song — đoạn dài, mỗi ký tự một khối dọc (Quyết định #4a).
 *
 * ⚠️ `line-height` LÀ TOKEN `source-cjk` (2.05) — không một con số viết thẳng: Kiểm B2
 * của `check-tokens.mjs` từ chối MỌI giá trị `line-height` không phải `var(--…)`, không
 * có miễn trừ cho nhóm thuộc tính chữ (khác Kiểm B màu/Kiểm F z-index, có miễn trừ có tên).
 * Giãn dòng 2.05 của chính token đã để đủ khoảng trống cho âm đọc (`.hv-reading`, ~12.5px)
 * nằm trong phần leading giữa hai dòng, không đè lên dòng kế tiếp.
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
 * 🔴 BỀ RỘNG Ô NỞ THEO ĐỘ DÀI ÂM — và nó KHÔNG đụng tới vùng chọn (AC6).
 *
 * Vấn đề bản đầu: `.hv-reading` là `position: absolute` ⇒ nó ra khỏi luồng và **không
 * góp một pixel nào** vào bề rộng `.hv-unit` (đúng bằng một glyph CJK). Âm `"chênh"` ở
 * 12,5px rộng gần gấp đôi ô đó, nên mọi cụm Hán liền nhau cho ra các âm **đè lên nhau** —
 * kiểu song song, thứ AC6 đòi, không đọc được. Bắt ở lượt code review 2026-08-06.
 *
 * 🔴 VÌ SAO CÁCH NÀY AN TOÀN VỚI AC6, trong khi cách hiển nhiên thì không: mọi cách đưa
 * âm **trở lại luồng** (một bản chép ẩn để đo, một `display: block` bên trong) đều dựng
 * lại một **hộp dòng** mới — đúng thứ làm `Selection.toString()` chèn `\n` và là lý do
 * `.hv-reading` được đẩy ra `position: absolute` ngay từ đầu. Ở đây không một node văn
 * bản nào được thêm: chỉ **bề rộng** của ô đổi. `Selection.toString()` nối các NODE VĂN
 * BẢN, không đọc bề rộng — nên chuỗi bôi đen không đổi một ký tự nào.
 *
 * `--hv-reading-len` do template đặt (số ký tự của âm đang hiện). `0.56em` là bề rộng
 * trung bình một glyph Latin thường ở họ chữ nghiêng — một **sàn xấp xỉ** để không đè,
 * không phải một phép đo pixel-perfect; token vẫn là thứ quyết cỡ chữ (AD-34 ③).
 */
.hv-unit {
  position: relative;
  display: inline-block;
  vertical-align: bottom;
  text-align: center;
  min-width: calc(var(--font-source-hanviet) * 0.56 * var(--hv-reading-len, 1));
}

/*
 * 🔴 AC6 — BẮT ĐƯỢC LÚC ĐO THẬT (Playwright, không phải lời hứa): `display: inline-flex;
 * flex-direction: column` (bản đầu) làm Chromium chèn MỘT `\n` vào `Selection.toString()`
 * ở MỖI ranh giới `.hv-unit` — vùng chọn nhận `"他\n打\n開了…"` thay vì `"他打開了…"`, dù
 * `.hv-reading` đã `user-select: none`. Nguyên nhân: cả flex container LẪN một `display:
 * block` bên trong một inline-block đều tạo một **hộp dòng (line box) mới**, và thuật toán
 * `Selection.toString()` của Chromium chèn ngắt dòng ở MỌI ranh giới hộp dòng — bất kể nội
 * dung trong đó có được chọn hay không.
 *
 * ⇒ ĐƯỜNG ĐÚNG: `.hv-reading` ra khỏi luồng bố cục bằng `position: absolute`. Một phần tử
 * định vị tuyệt đối không tham gia tính hộp dòng của phần tử cha, nên `.hv-char` ở lại
 * ĐÚNG MỘT hộp dòng với các ký tự lân cận — không ranh giới nào để `Selection.toString()`
 * chèn `\n` vào. Đã đo lại: `他打開了那扇門，走進了黑暗之中。` chọn ra ĐÚNG chính nó.
 */
.hv-reading {
  position: absolute;
  top: 100%;
  left: 50%;
  transform: translateX(-50%);
  white-space: nowrap;
  /* 🔴 AC6 — loại khỏi vùng chọn, KHÔNG bao giờ đi vào `window.getSelection()`. */
  user-select: none;
  /* 🔴 AC7 — người tiêu thụ ĐẦU TIÊN của `source-hanviet`. Năm biến, không chỉ synthesis. */
  font-family: var(--face-source-hanviet);
  font-size: var(--font-source-hanviet);
  font-style: var(--style-source-hanviet);
  font-synthesis: var(--synthesis-source-hanviet);
  line-height: var(--leading-source-hanviet);
  color: var(--color-on-surface-variant);
}
</style>
