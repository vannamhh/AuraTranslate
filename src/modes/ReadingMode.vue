<script setup lang="ts">
// Chế độ 3/3 — đọc lại bản dịch đã hoàn thành, không có công cụ biên tập. Story 1.6 ·
// AC3 · AC4. Story 5.11 đổ chữ THẬT vào đây: typography ba mức, song ngữ, mục lục. Story
// 5.12 đổi bề mặt từ MỘT Chương thành MỘT LƯỢT ĐỌC (`ReadingRun`, FR120): chỉ Chương
// `Done` mới ra trang, và cuối trang luôn có một mốc biên nói vì sao dãy dừng ở đó.
//
// 🔵 SỬA (2026-08-30, Story 5.11) — khối chú thích cũ tự khai "KHUNG RỖNG có chủ ý" và
// cảnh báo "body chạy ở ui-md, giãn dòng 1.5 — dưới sàn 1.66, không phép kiểm nào canh
// được chỗ đó" đã HẾT ĐÚNG: bề mặt này nay khai token `read-*` của chính nó
// (`readingStyle` của `readingState.ts`), đặt trên CHÍNH phần tử mang cỡ chữ (`.column`)
// — không còn kế thừa `ui-md` của `body`.
//
// 🔴 Không công cụ biên tập nào ở đây (AD-1, §Always của story): không vạch lề segment,
// không nút xác nhận, không panel, không `contenteditable`, không lưới hai cột. Chỉ
// bản dịch — song ngữ (nếu bật) hiện nguyên văn ở LỀ, không chen vào dòng đọc.
//
// 🔴 Cắt bỏ, cấu trúc đoạn, VÀ chọn Chương nào được đọc đều đến từ Rust
// (`read_reading_run`) — không một `v-if="!s.is_omitted"` nào ở đây (không CÓ `is_omitted`
// để mà lọc), và không một đường nào ở tệp này hỏi `chapter.status` để quyết định hiển thị
// hay không (§Always của Story 5.12): trang chỉ render thứ Rust ĐÃ gửi.
import { computed, nextTick, onActivated, onBeforeUnmount, onMounted, useTemplateRef, watch } from 'vue'
import type { DeepReadonly } from 'vue'
import { declareFocus, dispatch, enterFocus, releaseFocus } from '../commands'
import { hasMessageKey, t, tError } from '../i18n'
import { currentTheme } from '../tokens/themeState'
import {
  currentReadingLevel,
  effectiveFontSize,
  effectiveLineHeight,
  ensureReadingLoaded,
  READING_LINE_HEIGHT_FLOOR,
  readingBilingual,
  readingRun,
  readingLoadError,
  readingStatusKind,
  readingStyle,
  readingTocBusy,
  readingTocChapters,
  readingTocCursor,
  readingTocError,
  readingTocHaveLoaded,
  readingTocOpen,
  readingTunerOpen,
  setFontSize,
  setLineHeight,
} from './readingState'
import type { ReadingChapter, ReadingFrontierChapter } from '../config/reading'

const root = useTemplateRef<HTMLElement>('root')
const tocPanel = useTemplateRef<HTMLElement>('tocPanel')

/**
 * Tiêu điểm đi theo lớp phủ mục lục — 🔵 THÊM ở lượt rà 2026-08-30 (NFR17).
 *
 * ⚠️ `await nextTick()` là bắt buộc ở CẢ HAI chiều: lúc mở, `tocPanel` chưa có trong DOM tại
 * thời điểm `readingTocOpen` đổi (`v-if`); lúc đóng, gốc chế độ mới lại là thứ nhận được
 * tiêu điểm sau khi lớp phủ bị gỡ.
 *
 * 🔴 Không ném, kể cả khi phần tử vắng mặt — hàm này chạy sau một hợp âm bàn phím
 * (`src/AGENTS.md`: *"hàm chạy từ một hợp âm KHÔNG BAO GIỜ ném — nó ghi chẩn đoán rồi trả"*).
 */
watch(readingTocOpen, async (open) => {
  await nextTick()
  if (open) {
    const panel = tocPanel.value
    if (panel === null) {
      console.warn('[reading] muc luc mo ma `tocPanel` khong co trong DOM — bo qua luot dat tieu diem')
      return
    }
    panel.focus()
    return
  }
  void enterFocus('mode.reading')
})

onMounted(() => {
  declareFocus('mode.reading', () => root.value)
})
onBeforeUnmount(() => {
  releaseFocus('mode.reading')
})
onActivated(() => {
  void enterFocus('mode.reading')
  void ensureReadingLoaded()
})

/**
 * Câu trạng thái — TÁM nhánh của [`readingStatusKind`]. Một `switch` thay vì một chuỗi
 * ternary dài: mỗi nhánh gọi ĐÚNG MỘT `t()`/`tError()`, không một biểu thức trộn — cùng
 * khuôn mà `check:i18n` Kiểm A2 đòi (xem `<!-- aura-allow-text -->` ngay dưới template,
 * vì bản thân biến này không "bắt đầu bằng `t(`" ở góc nhìn tĩnh của cổng).
 *
 * 🔵 SỬA TẠI CHỖ (2026-08-30, Story 5.12) — bốn nhánh cuối đổi hẳn theo hình dạng
 * `ReadingRun`: không còn `'empty-chapter'`/`'empty-unknown'` của một Chương đơn.
 */
const statusMessage = computed(() => {
  switch (readingStatusKind.value) {
    case 'not-loaded':
    case 'content':
      return ''
    case 'pending':
      return t('mode.reading.status_loading')
    case 'no-work':
      return t('mode.reading.status_no_work')
    case 'error':
      return readingLoadError.value === null ? '' : tError(readingLoadError.value)
    case 'frontier-only':
      return t('mode.reading.status_frontier_only')
    case 'all-omitted':
      return t('mode.reading.status_all_omitted')
    case 'empty-chapters':
      return t('mode.reading.status_empty_chapters')
  }
})

function onFontSizeInput(event: Event): void {
  setFontSize(Number((event.target as HTMLInputElement).value))
}
function onLineHeightInput(event: Event): void {
  setLineHeight(Number((event.target as HTMLInputElement).value))
}

/**
 * Nhãn "Chương {ord}" cho một hàng chưa đặt tên.
 *
 * 🔵 SỬA (rà 2026-08-30) — khoá RIÊNG của miền `mode.reading.*`, không mượn
 * `mode.library.chapter_untitled` nữa. Mượn khoá của miền khác làm câu chữ của màn hình này
 * đổi theo một quyết định biên tập của Library, và văn phạm khoá chấm có tiền tố miền tồn tại
 * đúng để chặn chuyện đó (`check:i18n` Kiểm B).
 */
function untitled(ord: number): string {
  return t('mode.reading.chapter_untitled', { ord: String(ord) })
}

/** Nhãn hiển thị của một Chương — tên thật nếu có, ngược lại nhãn "Chương {ord}". */
function chapterLabel(chapter: { chapter_ord: number; chapter_title: string | null }): string {
  return chapter.chapter_title ?? untitled(chapter.chapter_ord)
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.12 — MỐC BIÊN (`.frontier`)
// ═════════════════════════════════════════════════════════════════════════════════
// Bốn `computed` dưới đây dựng câu chữ của khối `.frontier` — mỗi cái đúng MỘT `t()`,
// cùng kỷ luật `statusMessage`. Tham số mang DỮ LIỆU (tên Chương, nhãn trạng thái đã tra),
// không mang CÂU — định dạng câu là việc của khoá `vi.json`, không việc của đây.

/**
 * Nhãn trạng thái cho mốc biên — `chapter.status` NGOÀI bốn giá trị `LifecycleStatus`
 * (§I/O Matrix "Trạng thái lạ") hiện NGUYÊN VĂN, không tra nhãn.
 *
 * 🔵 SỬA (rà 2026-08-30, Bản vá 7) — bản trước chép tay bốn giá trị vào một `Set` RIÊNG,
 * bản chép THỨ BA của cùng bốn giá trị mà `LifecycleStatus`/`vi.json` đã khai — không cổng
 * nào nối nó với hai đầu kia. Nay hỏi THẲNG `vi.json` qua [`hasMessageKey`], đóng đúng mắt
 * xích còn thiếu (xem doc-comment của hàm đó) thay vì một bản chép thứ tư.
 */
function frontierStatusLabel(status: string): string {
  const key = `lifecycle.${status}`
  return hasMessageKey(key) ? t(key) : status
}

/**
 * Câu nói dãy vừa đọc từ Chương nào tới Chương nào — hoặc dãy rỗng (AC5).
 *
 * 🔵 SỬA (rà 2026-08-30, Bản vá 6) — dãy ĐÚNG MỘT Chương (ca THƯỜNG NHẤT hôm nay: chưa
 * đường sản phẩm nào tạo Chương thứ hai ngoài `split_chapter_at_segment`) đi qua một khoá
 * RIÊNG (`frontier_range_single`) thay vì `frontier_range` — bản trước cho "từ Chương 1 đến
 * Chương 1" khi `first === last`, một câu đúng nghĩa đen nhưng đọc như lỗi lặp từ.
 */
const frontierRangeText = computed(() => {
  const run = readingRun.value
  if (run === null) return ''
  const first = run.chapters.at(0)
  const last = run.chapters.at(-1)
  if (first === undefined || last === undefined) return t('mode.reading.frontier_range_empty')
  if (run.chapters.length === 1) return t('mode.reading.frontier_range_single', { ten: chapterLabel(first) })
  return t('mode.reading.frontier_range', { tu: chapterLabel(first), den: chapterLabel(last) })
})

/** Câu nói Chương nào chặn kèm nhãn trạng thái — chỉ có nghĩa khi `kind === 'next-not-done'`. */
const frontierBlockedText = computed(() => {
  const frontier = readingRun.value?.frontier
  if (frontier === undefined || frontier.kind !== 'next-not-done') return ''
  const blocked: DeepReadonly<ReadingFrontierChapter> | null = frontier.chapter
  if (blocked === null) return ''
  return t('mode.reading.frontier_blocked', {
    title: chapterLabel(blocked),
    status_label: frontierStatusLabel(blocked.status),
  })
})

/** Câu nói hết Tác phẩm — chỉ có nghĩa khi `kind === 'end-of-work'`. */
const frontierEndOfWorkText = computed(() => {
  const frontier = readingRun.value?.frontier
  return frontier !== undefined && frontier.kind === 'end-of-work' ? t('mode.reading.frontier_end_of_work') : ''
})

/** Chữ trên nút *Dịch tiếp Chương N* — chỉ có nghĩa khi `kind === 'next-not-done'`. */
const frontierButtonText = computed(() => {
  const chapter = readingRun.value?.frontier.chapter
  return chapter === null || chapter === undefined ? '' : t('mode.reading.frontier_button', { ord: String(chapter.chapter_ord) })
})

/** Vì sao MỘT Chương của dãy không hiện câu nào — Chương rỗng, hoặc mọi câu đã cắt bỏ. */
function chapterEmptyNote(chapter: DeepReadonly<ReadingChapter>): string {
  return chapter.segment_count > 0 ? t('mode.reading.chapter_all_omitted') : t('mode.reading.chapter_empty')
}

/**
 * Chương của dãy đọc, hay một mảng RỖNG khi chưa nạp — tránh phối `v-if`/`v-for` trên
 * CÙNG một `<template>` (Vue 3 cảnh báo cách đó, và giá trị `null` sẽ ném ngay khi
 * `v-for` cố đọc `.chapters` của nó).
 */
const runChapters = computed<DeepReadonly<ReadingChapter>[]>(() => [...(readingRun.value?.chapters ?? [])])
</script>

<template>
  <section ref="root" class="mode" tabindex="-1">
    <div class="toolbar">
      <button
        type="button"
        class="btn"
        data-reading-toggle-bilingual
        :aria-pressed="readingBilingual"
        @click="dispatch('reading.toggle_bilingual')"
      >
        {{ t('mode.reading.bilingual') }}
      </button>
      <button
        type="button"
        class="btn"
        data-reading-level="lg"
        :aria-pressed="currentReadingLevel === 'lg'"
        @click="dispatch('reading.level_airy')"
      >
        {{ t('mode.reading.level_airy') }}
      </button>
      <button
        type="button"
        class="btn"
        data-reading-level="md"
        :aria-pressed="currentReadingLevel === 'md'"
        @click="dispatch('reading.level_balanced')"
      >
        {{ t('mode.reading.level_balanced') }}
      </button>
      <button
        type="button"
        class="btn"
        data-reading-level="sm"
        :aria-pressed="currentReadingLevel === 'sm'"
        @click="dispatch('reading.level_dense')"
      >
        {{ t('mode.reading.level_dense') }}
      </button>
      <button
        type="button"
        class="btn"
        data-reading-toggle-tuner
        :aria-pressed="readingTunerOpen"
        @click="dispatch('reading.toggle_tuner')"
      >
        {{ t('mode.reading.tuner') }}
      </button>
      <button
        type="button"
        class="btn"
        data-reading-toggle-theme
        :aria-pressed="currentTheme === 'dark'"
        @click="dispatch('reading.toggle_theme')"
      >
        {{ t('mode.reading.theme') }}
      </button>
      <button type="button" class="btn" data-reading-toc :aria-expanded="readingTocOpen" @click="dispatch('reading.toc')">
        {{ t('mode.reading.toc') }}
      </button>
    </div>

    <div v-if="readingTunerOpen" class="tuner">
      <label class="tuner-row">
        <span>{{ t('mode.reading.tuner_font_size') }}</span>
        <input
          type="range"
          data-reading-tuner-font-size
          min="14"
          max="28"
          step="0.5"
          :value="effectiveFontSize"
          @input="onFontSizeInput"
        />
      </label>
      <label class="tuner-row">
        <span>{{ t('mode.reading.tuner_line_height') }}</span>
        <input
          type="range"
          data-reading-tuner-line-height
          :min="READING_LINE_HEIGHT_FLOOR"
          max="2.4"
          step="0.01"
          :value="effectiveLineHeight"
          @input="onLineHeightInput"
        />
      </label>
    </div>

    <!-- role="status" LUÔN có mặt, không v-if trên chính node -- TÁM nhánh phân biệt
         được qua `data-reading-status`, không chỉ qua chữ hiển thị.
         🔵 SỬA TẠI CHỖ (2026-08-30, Story 5.12) — bốn nhánh cuối đổi tên theo `ReadingRun`,
         xem `readingStatusKind` ở `readingState.ts`. -->
    <!-- aura-allow-text: `statusMessage` là một computed đã tự đi qua t()/tError() bên
         trong phần kịch bản phía trên -- xem doc-comment của nó ngay trên. -->
    <p class="status" role="status" :data-reading-status="readingStatusKind">{{ statusMessage }}</p>

    <!--
      🔴 THÊM (Story 5.12) — vòng lặp CHƯƠNG bọc ngoài khối `.page`: mỗi Chương của
      `readingRun.chapters` (đã lọc `Done` ở Rust) sinh MỘT `.page` (lề song ngữ + cột dịch)
      của riêng nó. Dãy rỗng (§I/O Matrix "Chạm biên ngay") ⇒ không `.page` nào render, chỉ
      `.frontier` bên dưới còn lại trên trang.
    -->
    <!--
      🔵 SỬA (rà 2026-08-30, Bản vá 4) — `<section>`, không `<div>`, và tiêu đề Chương hạ
      xuống `<h2>` (xem template bên trong). Bản trước mỗi Chương một `<h1>` ngang hàng, nên
      một dãy bốn Chương cho ra BỐN `<h1>` rồi một `<h2>` (mốc biên) treo dưới cái CUỐI cùng
      — cây tiêu đề nói sai cấu trúc trang cho trình đọc màn hình. `<section>` là nội dung
      PHÂN ĐOẠN có tên (qua `<h2>` bên trong nó, theo thuật toán outline HTML), và mốc biên
      giữ `<h2>` NGANG HÀNG — đúng thứ tự đọc tuyến tính "Chương 1, Chương 2, …, Mốc biên".
    -->
    <template v-for="chapter in runChapters" :key="chapter.chapter_id">
      <section class="page">
        <!--
          🔵 SỬA (rà 2026-08-30) — GỠ `aria-hidden="true"`. Nguyên văn là NỘI DUNG người dùng
          bật lên có chủ ý (`B`), không một hoạ tiết trang trí; ẩn nó khỏi trình đọc màn hình là
          dựng đúng một cột chữ mà người dùng bàn phím/AT không với tới được — ngược NFR17.
          🔵 SỬA cùng lượt — ngăn câu bằng MỘT DẤU CÁCH, không `join('')`: hai câu dán liền nhau
          là một lỗi đọc thật. ⚠️ Dấu cách hơi thừa với nguyên văn tiếng Trung (`。` đã tự chở
          khoảng nghỉ), và ta KHÔNG rẽ nhánh theo ngôn ngữ ở đây vì `ReadingChapter` không chở
          `source_lang` trên dây — món nợ có chủ ở `deferred-work.md`, không một phép đoán tại chỗ.
        -->
        <div v-if="readingBilingual" class="margin">
          <!-- aura-allow-text: nguyên văn là DỮ LIỆU của Tác phẩm, không câu giao diện. -->
          <p v-for="(paragraph, index) in chapter.paragraphs" :key="index" class="source-note">
            {{ paragraph.segments.map((s) => s.source_text).join(' ') }}
          </p>
        </div>
        <!--
          🔴 `font-size`/`line-height` KHÔNG đi thẳng vào `:style` — `check:tokens` Kiểm B2 đòi
          hai thuộc tính đó luôn là `var(--font-…)`/`var(--leading-…)` LITERAL trong khối kiểu
          scoped bên dưới. Giá trị HIỆU LỰC (token thuần, hoặc override từ thanh trượt) đi qua
          HAI biến CSS riêng (`--reading-font-size`/`--reading-line-height`, không nằm trong
          danh sách thuộc tính Kiểm B2 canh) rồi khối kiểu đó đọc lại bằng `var(...)`.
          🔵 SỬA (rà 2026-08-30) — câu cũ nói `max-width` đi thẳng. Thước nay là `width` (xem khối
          🔵 của `readingStyle`); `width` cũng không nằm trong danh sách Kiểm B2 canh, nên nó vẫn
          đi thẳng, và giá trị vẫn LUÔN là token `ch`.
        -->
        <div
          class="column"
          :style="{
            '--reading-font-size': readingStyle.fontSize,
            '--reading-line-height': readingStyle.lineHeight,
            width: readingStyle.measure,
          }"
        >
          <!-- aura-allow-text: `chapterLabel()` trả tên Chương (dữ liệu) hoặc gọi `untitled()`
               (qua t()) cho hàng chưa đặt tên -- không một biểu thức trộn nào lộ ra template. -->
          <h2 class="chapter-title">{{ chapterLabel(chapter) }}</h2>
          <template v-if="chapter.paragraphs.length > 0">
            <p v-for="(paragraph, index) in chapter.paragraphs" :key="index" class="paragraph">
              <!--
                aura-allow-text: bản dịch là DỮ LIỆU của Tác phẩm, không câu giao diện.
                🔴 **DẤU CÁCH GIỮA HAI CÂU LÀ MỘT NỘI DUNG, KHÔNG MỘT KHOẢNG TRẮNG THỪA** — bắt ở
                lượt rà 2026-08-30. Bản đầu viết hai `<span>` liền nhau không gì ngăn, nên một đoạn
                hai câu render thành *"…giữa bóng tối.Gió thổi tới…"*. Bản dựng UX
                (`mockups/reading-mode.html`) không dính vì HTML nguồn của nó XUỐNG DÒNG giữa hai
                `<span>` và khoảng trắng ấy co lại thành một dấu cách; một `v-for` trên một dòng thì
                không có gì để co. Bàn đo e2e dùng `toContain` nên mù với chỗ này.
                ⚠️ Dấu cách đặt Ở ĐẦU mọi câu TRỪ câu đầu — đặt ở CUỐI sẽ để lại một khoảng trắng
                lơ lửng sau câu chót của mỗi đoạn.
                🔴 THÊM (Story 5.12, AC6) — `:class="{ unconfirmed: !segment.is_confirmed }"`: câu
                CHƯA xác nhận mang gạch chấm nhẹ, dấu hiệu đến từ `is_confirmed` trên dây.
              -->
              <span
                v-for="(segment, i) in paragraph.segments"
                :key="segment.id"
                :class="{ unconfirmed: !segment.is_confirmed }"
                >{{ i === 0 ? '' : ' ' }}{{ segment.target_text }}</span
              >
            </p>
          </template>
          <!-- aura-allow-text: `chapterEmptyNote()` tự đi qua t() bên trong. -->
          <p v-else class="chapter-note">{{ chapterEmptyNote(chapter) }}</p>
        </div>
      </section>
    </template>

    <!--
      🔴 THÊM (Story 5.12) — MỐC BIÊN, luôn có mặt khi một lượt đọc đã nạp (kể cả dãy rỗng):
      nó là lý do trang dừng ở đó, không phải một phần thưởng chỉ hiện khi có nội dung.
    -->
    <div v-if="readingRun !== null" class="frontier">
      <h2 class="frontier-heading">{{ t('mode.reading.frontier_heading') }}</h2>
      <!-- aura-allow-text: `frontierRangeText` là một computed đã tự đi qua t() bên trong
           phần kịch bản phía trên -- xem doc-comment của nó ngay trên. -->
      <p class="frontier-range">{{ frontierRangeText }}</p>
      <!-- aura-allow-text: `frontierBlockedText` tự đi qua t() bên trong. -->
      <p v-if="readingRun.frontier.kind === 'next-not-done'" class="frontier-note" data-reading-frontier-kind="next-not-done">{{ frontierBlockedText }}</p>
      <!-- aura-allow-text: `frontierEndOfWorkText` tự đi qua t() bên trong. -->
      <p v-else class="frontier-note" data-reading-frontier-kind="end-of-work">{{ frontierEndOfWorkText }}</p>
      <button
        v-if="readingRun.frontier.kind === 'next-not-done'"
        type="button"
        class="btn"
        data-reading-frontier-continue
        @click="dispatch('reading.continue_in_workspace')"
      >
        <!-- aura-allow-text: `frontierButtonText` tự đi qua t() bên trong. -->
        {{ frontierButtonText }}
      </button>
    </div>

    <!--
      🔵 SỬA (rà 2026-08-30) — BA lỗ bàn phím của lớp phủ này, cả ba đều ngược NFR17:
      ① lớp phủ mở ra mà KHÔNG gì nhận tiêu điểm ⇒ người dùng bấm `⌘L` rồi phải Tab mò từ
         đầu tài liệu. Nay `tocPanel` được `focus()` ngay sau lượt mở (watcher dưới đây).
      ② `Escape` không đóng được ⇒ lối ra duy nhất là Tab tới nút "Đóng". Nay `@keydown.esc`
         bắt TẠI CHỖ trên lớp phủ. ⚠️ **Không** đăng ký một hợp âm `Escape` TRẦN toàn ứng
         dụng: `registry` không có phạm vi theo chế độ, nên một `Escape` toàn cục sẽ bắn
         trong mọi lớp phủ khác — Kiểm A của `check:commands` chỉ canh `@click`, nên
         `@keydown` tại chỗ là đúng cửa.
      ③ đóng xong tiêu điểm rơi tự do ⇒ trả nó về gốc Chế độ đọc.
    -->
    <div
      v-if="readingTocOpen"
      class="toc-overlay"
      role="dialog"
      aria-modal="true"
      @keydown.esc="dispatch('reading.toc_close')"
    >
      <div ref="tocPanel" class="toc-panel" tabindex="-1">
        <p class="toc-heading">{{ t('mode.reading.toc_heading') }}</p>
        <!-- role="status" LUÔN có mặt (không v-if) -- cùng khuôn LibraryMode. -->
        <!-- aura-allow-text: cả hai nhánh đi qua tError()/chuỗi rỗng. -->
        <p class="toc-error" role="status">{{ readingTocError !== null ? tError(readingTocError) : '' }}</p>
        <!-- aura-allow-text: cả hai nhánh đi qua t()/chuỗi rỗng. -->
        <p class="toc-empty" role="status">
          {{ readingTocHaveLoaded && readingTocError === null && readingTocChapters.length === 0 ? t('mode.reading.toc_empty') : '' }}
        </p>
        <ul class="toc-list">
          <!-- 🔵 THÊM (rà 2026-08-30) `aria-current` — con trỏ danh sách chỉ được báo bằng
               MÀU NỀN thì một người dùng trình đọc màn hình đi `toc_next`/`toc_prev` không có
               đường nào biết đang chọn Chương nào trước khi bấm "Mở". Cùng khuôn `aria-current`
               mà `LibraryMode.vue` đã dùng cho hàng kết quả tìm kiếm. -->
          <li
            v-for="(row, index) in readingTocChapters"
            :key="row.chapter_id"
            :class="{ current: index === readingTocCursor }"
            :aria-current="index === readingTocCursor ? 'true' : undefined"
          >
            <!-- aura-allow-text: nhánh trái là tên Chương (dữ liệu); nhánh phải qua t(). -->
            {{ row.title ?? untitled(row.ord) }}
          </li>
        </ul>
        <div class="toc-actions">
          <button type="button" class="btn" :disabled="readingTocBusy" @click="dispatch('reading.toc_prev')">
            {{ t('mode.reading.toc_prev') }}
          </button>
          <button type="button" class="btn" :disabled="readingTocBusy" @click="dispatch('reading.toc_next')">
            {{ t('mode.reading.toc_next') }}
          </button>
          <button type="button" class="btn" :disabled="readingTocBusy" @click="dispatch('reading.toc_open')">
            {{ t('mode.reading.toc_open_button') }}
          </button>
          <button type="button" class="btn" @click="dispatch('reading.toc_close')">
            {{ t('mode.reading.toc_close_button') }}
          </button>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.mode {
  height: 100%;
  padding: var(--space-panel-block) var(--space-panel-inline);
  overflow: auto;
}

/* Xem lý do đầy đủ ở `LibraryMode.vue` — `outline: none` chỉ áp cho gốc `tabindex="-1"`,
   không bao giờ áp cho `*:focus` (§Trap 4). */
.mode:focus {
  outline: none;
}

.toolbar {
  display: flex;
  gap: var(--space-panel-inline);
  flex-wrap: wrap;
  margin-bottom: var(--space-panel-block);
}

.tuner {
  display: flex;
  flex-direction: column;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.tuner-row {
  display: flex;
  align-items: center;
  gap: var(--space-panel-inline);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  font-family: var(--face-ui-sm);
  color: var(--color-on-surface-variant);
}

.status {
  margin: 0 0 var(--space-panel-block) 0;
  color: var(--color-on-surface-variant);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  font-family: var(--face-ui-md-wrap);
}

.page {
  display: flex;
  align-items: flex-start;
  gap: var(--space-panel-inline);
}

/*
 * ⚠️ KHÔNG một token mới cho ghi chú song ngữ — `source-latin` đã tồn tại đúng cho vai
 * "nguyên văn" (Panel Source), và AD-34 đòi mọi token mới qua sổ `deviations` của
 * `check-tokens.mjs` kèm chữ ký Ice. `ReadingChapter` không mang `source_lang` (hình dạng
 * dây đã CHỐT ở bốn trường của `commands::segment::ReadingChapter`), nên không có tín
 * hiệu nào để chọn `source-cjk` thay `source-latin` — dùng một token DUY NHẤT cho cả
 * hai kịch bản là lựa chọn hẹp hơn, không phải một token thứ mười bảy.
 */
.margin {
  flex: 0 0 auto;
  max-width: 22ch;
  font-size: var(--font-source-latin);
  line-height: var(--leading-source-latin);
  font-family: var(--face-source-latin);
  color: var(--color-on-surface-variant);
}

.source-note {
  margin: 0 0 1em 0;
}

.column {
  flex: 0 0 auto;
  /* 🔴 `max-width: 100%` là RÀO cửa sổ hẹp, KHÔNG phải thước đọc. Thước là `width` (`ch`), đặt
     ở `:style`. Dùng `max-width` làm thước cho phép cột (`flex: 0 0 auto`) co về bề rộng NỘI
     DUNG, và lúc đó số ký tự mỗi dòng không còn do thước quyết: đo bằng một phép GỠ thật trong
     WKWebView (2026-08-30) cho **57,27 ch** thay vì **68,00 ch**. Lý lẽ và bảng số đầy đủ ở
     khối 🔵 của `readingStyle` trong `readingState.ts`. */
  max-width: 100%;
  /* Ba mức Thoáng/Cân/Đặc đều khai `family: read` — họ chữ không đổi theo mức, chỉ cỡ
     chữ/giãn dòng/thước đo đổi (qua hai biến CSS đặt ở `:style`, xem chú thích template). */
  font-family: var(--family-read);
  font-size: var(--reading-font-size);
  line-height: var(--reading-line-height);
}

.chapter-title {
  margin: 0 0 var(--space-panel-block) 0;
  font-size: var(--font-read-title);
  line-height: var(--leading-read-title);
  font-weight: var(--weight-read-title);
  font-family: var(--face-read-title);
}

.paragraph {
  margin: 0 0 1em 0;
}

/* Câu CHƯA xác nhận (Story 5.12, AC6) — gạch chấm nhẹ dưới chân, không đổi màu chữ và
   không `opacity` (Kiểm D cấm opacity lùi chữ). `ornament` là token vai `stroke` ĐÃ CÓ,
   đúng giá trị `--orn` mà mockup dùng cho `.s.unconf` -- 0 cặp tương phản mới. */
.unconfirmed {
  border-bottom: 1px dotted var(--color-ornament);
}

.chapter-note {
  margin: 0 0 var(--space-panel-block) 0;
  color: var(--color-on-surface-variant);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  font-family: var(--face-ui-md);
}

/* Mốc biên (Story 5.12) — chỉ token đã có: vạch ngăn dùng `--color-outline`, chữ phụ dùng
   `--color-on-surface-variant`. Không `box-shadow`, không gradient, không `opacity` lùi chữ. */
.frontier {
  margin-top: var(--space-panel-block);
  padding-top: var(--space-panel-block);
  border-top: 1px solid var(--color-outline);
}

.frontier-heading {
  margin: 0 0 var(--space-panel-block) 0;
  font-size: var(--font-ui-md-strong);
  line-height: var(--leading-ui-md-strong);
  font-weight: var(--weight-ui-md-strong);
  font-family: var(--face-ui-md-strong);
}

/* 🔵 SỬA (rà 2026-08-30, Bản vá 9) — `.frontier-blocked` đổi tên `.frontier-note`: lớp đó
   dùng cho CẢ HAI nhánh (`next-not-done` VÀ `end-of-work`), và "blocked" nói sai một nửa số
   ca -- nhánh `end-of-work` không hề bị chặn, nó báo tin đã đọc hết. */
.frontier-range,
.frontier-note {
  margin: 0 0 var(--space-panel-inline) 0;
  color: var(--color-on-surface-variant);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  font-family: var(--face-ui-md);
}

.toc-overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  /* Chỉ token có sẵn — không `--color-scrim` (không tồn tại) và không `opacity`/`rgba`. */
  background: var(--color-background);
}

.toc-panel {
  background: var(--color-surface);
  padding: var(--space-panel-block) var(--space-panel-inline);
  max-width: 60ch;
  max-height: 80vh;
  overflow: auto;
  border-radius: var(--radius-default);
}

.toc-heading {
  margin: 0 0 var(--space-panel-block) 0;
  font-size: var(--font-ui-md-strong);
  line-height: var(--leading-ui-md-strong);
  font-weight: var(--weight-ui-md-strong);
  font-family: var(--face-ui-md-strong);
}

.toc-list {
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
}

.toc-list li {
  padding: var(--space-panel-inline);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  font-family: var(--face-ui-md);
}

.toc-list li.current {
  background: var(--color-surface-accent);
}

.toc-actions {
  display: flex;
  gap: var(--space-panel-inline);
  flex-wrap: wrap;
}
</style>
