<script setup lang="ts">
// Chế độ 1/3 — kho Tác phẩm và **điểm vào ứng dụng** (PRD §5.2). Story 1.6 · AC3 · AC4.
// Story 1.15 thêm đường nhập tối thiểu (AC1/AC8/NFR17): dán văn bản · kéo-thả · ô nhập
// đường dẫn — đủ để có văn bản mà Story 1.16 hiển thị ở Panel Source.
//
// 🔵 SỬA (2026-08-27, Story 5.4) — câu cũ gộp CẢ "bốn trạng thái vòng đời" vào phần chưa dựng
// đã HẾT ĐÚNG: khối "Tác phẩm" (danh sách phẳng + bốn nút lọc) và khối "Trạng thái Tác phẩm
// đang mở" (ba lệnh vòng đời) nay có mặt ở đây.
// 🔵 SỬA (2026-08-28, Story 5.6) — câu "Lưới Tác phẩm (bìa, sắp xếp, lọc thể loại/ngôn ngữ)
// thuộc Story 5.6 — đây vẫn là một khung rỗng cho phần đó" đã HẾT ĐÚNG: khối `.works-block`
// nay là một LƯỚI (`.works-grid`/`.work-cell`, khung bìa + biểu diễn thay thế), ba `<select>`
// lọc lĩnh vực/ngôn ngữ/sắp xếp, và con trỏ ô (`workCursor`, AC7) đều có mặt.
//
// Không chuỗi tiếng Việt nào trong tệp này (NFR16, AD-21) — mọi nhãn đi qua `t()`.
import { computed, nextTick, onActivated, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'
import { declareFocus, dispatch, enterFocus, releaseFocus } from '../commands'
import type { WorkRow, WorkSortKey } from '../config/library'
import {
  busy,
  createdWork,
  filePath,
  genre,
  isDragOver,
  lastError,
  name,
  noticeKey,
  pastedText,
  sourceLang,
  unwireDragDrop,
  wireDragDropOnce,
} from './libraryImport'
import {
  currentLibraryOrphan,
  currentLibraryRoot,
  firstLibraryConflict,
  libraryConflicts,
  libraryIndexedCount,
  libraryOrphanCursor,
  libraryOrphans,
  libraryRescanBusy,
  libraryRescanError,
  libraryRootMissing,
  libraryScanHasLoadedState,
  libraryConflictCount,
  librarySkippedCount,
} from './libraryRescan'
import {
  libraryFilterIsEmpty,
  libraryGenreFilter,
  libraryGenres,
  librarySortKey,
  librarySourceLangFilter,
  librarySourceLangs,
  libraryStatusFilter,
  libraryWorkCursor,
  libraryWorks,
  libraryWorksBusy,
  libraryWorksError,
  libraryWorksHaveLoaded,
  libraryWorksMatched,
  libraryWorksTotal,
  loadOpenWorkLifecycle,
  loadWorks,
  openWorkLifecycleBusy,
  openWorkLifecycleError,
  openWorkLifecycleIsOverride,
  openWorkLifecycleLoaded,
  openWorkLifecycleStatus,
  setGenreFilter,
  setSortKey,
  setSourceLangFilter,
} from './libraryWorks'
// ── Story 5.7 — "Danh sách Chương và mở Chương vào Workspace" (FR12) ─────────────────
import {
  chapterRenameDraft,
  chapterWindow,
  currentLibraryChapter,
  libraryChapterCursor,
  libraryChapterReorgBusy,
  libraryChapterReorgError,
  libraryChapterReorgNotice,
  libraryChapters,
  libraryChaptersBusy,
  libraryChaptersError,
  libraryChaptersHaveLoaded,
  libraryOpenWorkBusy,
  libraryOpenWorkError,
  libraryOpenWorkNotice,
  loadChapters,
} from './libraryChapters'
// ── Story 5.9 — "Tìm kiếm full-text xuyên Library" (FR8) ─────────────────────────────
// 🔵 SỬA (2026-08-29, Story 5.10) — thêm `librarySearchMode` cho hai nút chế độ + đọc thêm
// `match_kind` của mỗi hit; `role="status"` mở rộng tám nhánh (đọc thẳng `librarySearchStatusKey`,
// không cần một export mới cho `widened` — tám giá trị của nó đã chở đủ thông tin).
import {
  librarySearchBusy,
  librarySearchCursor,
  librarySearchError,
  librarySearchHits,
  librarySearchMode,
  librarySearchQuery,
  librarySearchStatusKey,
  librarySearchTotal,
  librarySearchTruncated,
} from './librarySearch'
import { t, tError } from '../i18n'

const root = useTemplateRef<HTMLElement>('root')

onMounted(() => {
  declareFocus('mode.library', () => root.value)
  // ⚠️ Gắn MỘT LẦN ở `onMounted`, không `onActivated`: `<KeepAlive>` giữ subtree
  // (§Quyết định thiết kế #6) nên `mounted` chỉ chạy lượt đầu — gắn ở `onActivated` sẽ
  // không lặp lại gì thêm, nhưng đặt nó ở `onMounted` khớp đúng vòng đời "một lần" của
  // `wireDragDropOnce()` (tự chốt idempotent, nhưng ý định code phải khớp cơ chế).
  wireDragDropOnce()
})
onBeforeUnmount(() => {
  releaseFocus('mode.library')
  // ⚠️ Bộ nghe kéo-thả là tầng CỬA SỔ, không phải của `.dropzone` — để nó sống sau khi
  // chế độ bị tháo nghĩa là một cú thả ở Workspace vẫn điền vào một form không còn trên
  // màn hình. Code review 2026-08-06.
  unwireDragDrop()
})
// `onActivated` chứ không phải `onMounted`: ba chế độ sống trong `<KeepAlive>` (§Quyết
// định thiết kế #6), nên lần hiện thứ hai trở đi KHÔNG có `mounted`. Ở lượt mount đầu
// tiên Vue gọi `mounted` rồi `activated`, nên điểm vào đã khai xong trước khi dùng.
onActivated(() => {
  void enterFocus('mode.library')
  // Story 5.4 — tải lại MỖI LẦN quay về Library: một Chương có thể vừa đổi trạng thái từ
  // Workspace (qua `lifecycle.set_chapter_done`), và danh sách/khối "Tác phẩm đang mở" phải
  // phản ánh giá trị mới nhất, không giữ ảnh chụp của lần hiện trước.
  void loadWorks()
  void loadOpenWorkLifecycle()
  // Story 5.7 — cùng lý do: danh sách Chương của Tác phẩm ĐANG MỞ phải phản ánh giá trị mới
  // nhất mỗi lần quay lại Library (một Chương có thể vừa đổi trạng thái từ Workspace).
  void loadChapters()
})

// 🔵 THÊM (2026-08-28) — KHUYẾT TẬT ĐO ĐƯỢC, tìm ra ở lượt chạy e2e đầu tiên của Story 5.4.
//
// `onActivated` là lối vào DUY NHẤT của hai lượt tải trên, và ở lượt khởi động nó chạy khi
// CHƯA Tác phẩm nào mở — `read_work_lifecycle` trả `work.none_open`, `openWorkLifecycleLoaded`
// ở lại `false`. Tạo một Tác phẩm ngay tại màn hình này KHÔNG rời chế độ, nên `onActivated`
// không chạy lại: khối "Tác phẩm đang mở" tiếp tục nói "Chưa có Tác phẩm nào đang mở" trong
// khi CÓ một Tác phẩm vừa mở, và cả ba nút vòng đời kẹt ở `disabled` cho tới khi người dùng
// tình cờ chuyển chế độ rồi quay lại. Một màn hình khẳng định điều nó biết là sai — đúng lớp
// lỗi mà `AGENTS.md::Known pitfalls` đặt lên hàng đầu.
//
// ⚠️ Theo dõi `createdWork` chứ không gọi từ `libraryImport.ts`: đường nhập không được biết
// gì về vòng đời (hai module thuần, hai vai), còn `.vue` này vốn đã là nơi điều phối cả hai
// lượt tải ở `onActivated` ngay trên.
watch(createdWork, (created) => {
  if (created === null) return
  void loadOpenWorkLifecycle()
  void loadWorks()
  // Story 5.7 — một Tác phẩm MỚI thay `OpenWorkState`, nên danh sách Chương của nó (một
  // Chương, fresh) phải nạp lại đúng lý lẽ hai lượt tải ngay trên.
  void loadChapters()
})

// **KHÔNG có handler `dragenter`/`dragover`/`dragleave`/`drop` của DOM ở đây, và đó là
// một quyết định** — code review 2026-08-06. `drag_drop_enabled` mặc định `true` ở Tauri
// v2 (`tauri.conf.json` không override), nên bộ xử lý tầng **hệ điều hành** giành lấy
// thao tác kéo và webview **không bao giờ** nhận được các sự kiện DOM đó. Bản trước gắn
// cả bốn, và cả bốn đều là mã chết: `.dropzone.over` không bao giờ bật, nên vùng kéo-thả
// không có một tín hiệu nào cho biết nó còn sống.
//
// ⇒ Cả ba trạng thái (vào · rời · thả) đến từ **Rust** qua `on_window_event`, xem
// `src-tauri/src/lib.rs::wire_drag_drop` và `./libraryImport.ts::wireDragDropOnce`.

/**
 * Story 5.5 — phần trăm bề rộng thanh tiến độ. THUẦN trình bày (không quy tắc nghiệp vụ,
 * AD-1): "đã xong" đến nguyên vẹn từ `WorkRow.chapter_done_count` (đã tính ở Rust,
 * `WorkMeta::rebuild_from_store`), hàm này chỉ đổi một cặp số nguyên thành một tỉ lệ hiển thị.
 *
 * `chapter_count === 0` ⇒ `0`, KHÔNG chia cho 0 (`NaN` sẽ làm `width: NaN%` — CSS im lặng bỏ
 * qua giá trị đó, nhưng đó là một trạng thái không viết ra được, không phải "vẽ ở 0%" như
 * §I/O Matrix đòi). Chỉ gọi khi `chapter_done_count !== null` — chỗ gọi trong `<template>`
 * đã canh bằng `v-if`.
 */
function progressPercent(work: WorkRow): number {
  if (work.chapter_count === 0) return 0
  const done = work.chapter_done_count ?? 0
  return Math.min(100, Math.round((done / work.chapter_count) * 100))
}

/**
 * Story 5.6 — biểu diễn thay thế nhất quán cho khung bìa (AC6): chữ cái đầu của tên, viết
 * hoa. `name` rỗng/toàn khoảng trắng ⇒ `'?'` (§I/O Matrix "Tên rỗng/khoảng trắng") — KHÔNG
 * một ô trống, và KHÔNG chuỗi rỗng (một ô trống là chính điều AC6 cấm).
 */
function coverInitial(name: string): string {
  const trimmed = name.trim()
  if (trimmed.length === 0) return '?'
  return trimmed.charAt(0).toUpperCase()
}

/**
 * Ba `<select>` (lĩnh vực · ngôn ngữ · sắp xếp) dùng `@change`, NGOÀI luật Kiểm A (§Design
 * Notes "Vì sao `<select>` chứ không thêm nút") — chúng gọi THẲNG các hàm mở tường của
 * `libraryWorks.ts`, không qua `dispatch()`. Chuỗi rỗng (`<option value="">`) ⇒ bỏ lọc,
 * đúng quy ước "mọi lĩnh vực"/"mọi ngôn ngữ".
 */
function onGenreFilterChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value
  setGenreFilter(value === '' ? null : value)
}
function onSourceLangFilterChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value
  setSourceLangFilter(value === '' ? null : value)
}
function onSortKeyChange(event: Event): void {
  const value = (event.target as HTMLSelectElement).value as WorkSortKey
  setSortKey(value)
}

// AC7 — ô đang chọn phải LUÔN nhìn thấy được. `workCellRefs` gom MỌI `<li>` của lưới theo
// đúng thứ tự `v-for` (Vue điền mảng này tự động khi `ref="workCellRefs"` sống trong
// `v-for`); `flush: 'post'` để DOM đã cập nhật xong trước khi tính lại ô đích (con trỏ có
// thể đổi CÙNG lượt danh sách rút ngắn, xem `clampWorkCursor` ở `libraryWorks.ts`).
const workCellRefs = useTemplateRef<HTMLLIElement[]>('workCellRefs')
watch(
  libraryWorkCursor,
  () => {
    void nextTick(() => {
      const cell = workCellRefs.value?.[libraryWorkCursor.value]
      // `block: 'nearest'`/`inline: 'nearest'` — tức thì theo mặc định (không `behavior:
      // 'smooth'`), cùng chủ ý "không hiệu ứng" mà `GridPanel.vue` §AC8 đã ghi cho cuộn theo
      // con trỏ.
      cell?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
    })
  },
  { flush: 'post' },
)

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.7 — DANH SÁCH CHƯƠNG CÓ CỬA SỔ (AC2). `chapterWindow()` (`libraryChapters.ts`)
// là hàm THUẦN; mọi thứ ĐỌC DOM (chiều cao khung nhìn thật, `scrollTop`) sống ở ĐÂY.
// ═════════════════════════════════════════════════════════════════════════════════

/**
 * Chiều cao MỘT hàng, PIXEL cố định — không đo hình học thật (§Design Notes: chiều cao thay
 * đổi theo nội dung sẽ đòi bàn đo/e2e, không thuộc vitest). Dùng CẢ cho `<style>` (chiều cao
 * mỗi `<li>`) LẪN phép tính cửa sổ, để hai bên không lệch nhau bằng hai con số viết tay.
 */
const CHAPTER_ROW_HEIGHT_PX = 40
/** Chiều cao khung nhìn của `.chapters-list` — khớp `max-height` khai ở `<style scoped>`. */
const CHAPTER_VIEWPORT_HEIGHT_PX = 240
/** Số hàng đệm THÊM mỗi đầu — một cú cuộn nhanh không thấy khoảng trắng trước khi Vue vá DOM. */
const CHAPTER_OVERSCAN = 4

const chaptersScrollTop = ref(0)
const chaptersScrollRef = useTemplateRef<HTMLUListElement>('chaptersScrollRef')

const chaptersWindowSlice = computed(() =>
  chapterWindow(
    chaptersScrollTop.value,
    CHAPTER_VIEWPORT_HEIGHT_PX,
    CHAPTER_ROW_HEIGHT_PX,
    libraryChapters.value.length,
    CHAPTER_OVERSCAN,
  ),
)

const visibleChapters = computed(() =>
  libraryChapters.value.slice(chaptersWindowSlice.value.start, chaptersWindowSlice.value.end),
)

function onChaptersScroll(event: Event): void {
  chaptersScrollTop.value = (event.target as HTMLElement).scrollTop
}

/**
 * AC2 — "hàng đang chọn luôn nằm trong cửa sổ". `chapterCursor` đổi bằng hai nút thật
 * (`‹`/`›`), KHÔNG bằng cuộn chuột — nên khi con trỏ ra khỏi `[start, end)` hiện thời, kéo
 * `scrollTop` để nó vào lại, rồi đồng bộ xuống phần tử THẬT (`@scroll` chỉ đọc NGƯỢC LẠI,
 * từ DOM vào `chaptersScrollTop`, nên một lượt đổi LẬP TRÌNH phải tự đẩy xuống).
 *
 * ⚠️ KHÔNG cùng cơ chế với `editorCaretPlacement`/AD-3: đây là `scrollTop` của một danh
 * sách CUỘN ẢO ở Library, không phải vị trí làm việc của Editor (AC4) — hai bề mặt khác
 * hẳn, chỉ tình cờ cùng từ "cuộn".
 */
watch(libraryChapterCursor, (cursor) => {
  const slice = chaptersWindowSlice.value
  if (cursor < slice.start) {
    chaptersScrollTop.value = cursor * CHAPTER_ROW_HEIGHT_PX
  } else if (cursor >= slice.end) {
    chaptersScrollTop.value = (cursor + 1) * CHAPTER_ROW_HEIGHT_PX - CHAPTER_VIEWPORT_HEIGHT_PX
  }
  const el = chaptersScrollRef.value
  if (el !== null && el.scrollTop !== chaptersScrollTop.value) el.scrollTop = chaptersScrollTop.value
})
</script>

<template>
  <!-- `tabindex="-1"` để phần tử nhận được focus lập trình. Nó KHÔNG vào thứ tự Tab. -->
  <section ref="root" class="mode" tabindex="-1">
    <!--
      Story 5.3 — "Quét lại thư mục" (FR99). Màn hình TỐI THIỂU CÓ CHỦ: chỉ thư mục gốc,
      nút quét lại, danh sách mục mồ côi (con trỏ một mục, cùng khuôn GlossaryQueueOverlay),
      và ba con số kết quả — KHÔNG lưới Tác phẩm/lọc/sắp xếp/bìa/vòng đời/tiến độ (5.4-5.6
      sở hữu).
    -->
    <div class="root-block">
      <p class="section-heading">{{ t('mode.library.root_heading') }}</p>
      <!--
        🔵 SỬA (2026-08-27, vòng rà bốn lớp P2) — bản trước khẳng định một điều CHƯA BIẾT.
        `currentLibraryRoot === null` chỉ nghĩa là "chưa quét lần nào trong PHIÊN NÀY" -- một
        người dùng đã cấu hình gốc riêng từ phiên trước sẽ thấy màn hình nói sai về thư mục
        của họ. Bản trước còn chép cứng `~/Documents/AuraTranslate/` vào chuỗi hiển thị,
        đúng thứ `default_library_root` (Rust) cố ý tránh ("Không viết cứng $HOME") và không
        biểu diễn được ca `document_dir()` trượt. Câu MỚI chỉ nói đúng trạng thái CHƯA BIẾT;
        đường dẫn THẬT chỉ hiện sau khi `RescanReport.root` (do Rust phân giải) về tới đây.
      -->
      <!--
        🔵 THÊM (2026-08-27, vòng rà THỨ HAI P8) — `role="status"`: ba node hàng xóm
        (`.root-missing`/`.status`/`.error`) đều mang nó, còn node này thì không -- đường dẫn
        gốc đổi (sau một lượt quét/đổi thư mục gốc) mà trình đọc màn hình không được báo.
      -->
      <!-- aura-allow-text: `currentLibraryRoot` là ĐƯỜNG DẪN đĩa (dữ liệu), không câu UI; nhánh
           còn lại đi qua t() -- Kiểm A2 không đọc tĩnh được toán tử `??`. -->
      <p class="root-value" role="status">{{ currentLibraryRoot ?? t('mode.library.root_not_scanned_yet') }}</p>
      <!--
        🔵 THÊM (2026-08-27, vòng rà bốn lớp P1) — câu RIÊNG khi gốc đã quét KHÔNG còn tồn
        tại trên đĩa, phân biệt hẳn với "đã quét, không có Tác phẩm nào" (đó là node
        `.status` bên dưới nói ba con số kết quả). role="status" LUÔN có mặt.
      -->
      <!-- aura-allow-text: nhánh đúng đi qua t() với tham số root (dữ liệu); nhánh sai là chuỗi rỗng. -->
      <p class="root-missing" role="status">
        {{ libraryScanHasLoadedState && libraryRootMissing ? t('mode.library.root_missing', { root: currentLibraryRoot ?? '' }) : '' }}
      </p>
      <div class="root-actions">
        <button
          type="button"
          class="btn"
          :disabled="libraryRescanBusy"
          @click="dispatch('library.choose_root')"
        >
          {{ t('mode.library.choose_root') }}
        </button>
        <button
          type="button"
          class="btn"
          :disabled="libraryRescanBusy"
          @click="dispatch('library.rescan')"
        >
          <!-- aura-allow-text: cả hai nhánh đi qua t() -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi. -->
          {{ libraryRescanBusy ? t('mode.library.rescan_busy') : t('mode.library.rescan') }}
        </button>
      </div>

      <!-- role="status" LUÔN có mặt (không v-if) -- cùng lý lẽ với dải báo lỗi của form nhập. -->
      <!-- aura-allow-text: cả hai nhánh đi qua t()/chuỗi rỗng -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi. -->
      <p class="status" role="status">
        {{
          libraryScanHasLoadedState
            ? t('mode.library.rescan_result', {
                indexed: String(libraryIndexedCount),
                conflicts: String(libraryConflictCount),
                skipped: String(librarySkippedCount),
              })
            : ''
        }}
      </p>
      <!-- aura-allow-text: như trên, qua tError(). -->
      <p class="error" role="status">{{ libraryRescanError ? tError(libraryRescanError) : '' }}</p>

      <!--
        🔵 THÊM (2026-08-27, phán quyết Ice #3) — AC4 nói "phát hiện VÀ CẢNH BÁO", hai vế.
        Trước bản vá, `conflicts` chỉ là một con số nén nằm CÙNG câu với "đã lập chỉ mục"/"bỏ
        qua" ở node `.status` ngay trên — không phải một bề mặt cảnh báo, đúng lỗ mà mục nợ
        AC4 trong deferred-work.md ghi. Node này TÁCH HẲN, chỉ hiện khi có xung đột, nêu ĐÍCH
        DANH chỗ trùng ĐẦU TIÊN kèm CẢ HAI đường dẫn, và "và N chỗ nữa" khi nhiều hơn một —
        KHÔNG dựng danh sách/lưới các chỗ trùng (Story 5.6 sở hữu phần "hiển thị danh sách";
        story này chỉ nợ 5.6 phần đó, không nợ phần "nói ra rằng có chuyện").
        `role="status"`, cùng khuôn `.root-missing` ngay trên: một cảnh báo phát sinh từ một
        lượt quét đã THÀNH CÔNG, không phải một lỗi thao tác (đó là `.error`, `role="alert"`
        thuộc về `GlossaryQueueOverlay.vue`, không phải đây).
      -->
      <p v-if="firstLibraryConflict" class="conflict-warning" role="status">
        <!-- aura-allow-text: hai đường dẫn là DỮ LIỆU đĩa; câu bao quanh đi qua t(). -->
        {{
          t('mode.library.conflict_warning', {
            kept_path: firstLibraryConflict.kept_path,
            duplicate_path: firstLibraryConflict.duplicate_path,
          })
        }}
        <!-- aura-allow-text: số đếm là DỮ LIỆU; câu bao quanh đi qua t(). -->
        <template v-if="libraryConflicts.length > 1">
          {{ t('mode.library.conflict_more', { count: String(libraryConflicts.length - 1) }) }}
        </template>
      </p>

      <p class="section-heading">{{ t('mode.library.orphans_heading') }}</p>
      <!--
        🔴 Vị từ `libraryScanHasLoadedState` TRƯỚC khi kết luận "không có mục mồ côi nào" —
        danh sách rỗng TRƯỚC lượt quét đầu tiên là "chưa biết", không phải "không có"
        (AGENTS.md::Known pitfalls).
      -->
      <!-- aura-allow-text: ba nhánh đều qua t()/chuỗi rỗng. -->
      <p class="status" role="status">
        {{
          !libraryScanHasLoadedState
            ? t('mode.library.orphans_not_loaded')
            : libraryOrphans.length === 0
              ? t('mode.library.orphans_none')
              : ''
        }}
      </p>

      <div v-if="currentLibraryOrphan" class="orphan-row">
        <!-- aura-allow-text: tên Tác phẩm là DỮ LIỆU người dùng gõ, không một câu UI. -->
        <p class="orphan-name">{{ currentLibraryOrphan.name }}</p>
        <p class="orphan-path">{{ t('mode.library.orphan_path', { path: currentLibraryOrphan.atproj_path }) }}</p>
        <div class="orphan-actions">
          <!--
            🔵 THÊM (2026-08-27, vòng rà bốn lớp P9) — `aria-label` cho cả hai nút: `‹`/`›`
            trần không mang nghĩa gì cho trình đọc màn hình (NFR17 đòi mọi thao tác dùng
            được bằng bàn phím VÀ có nhãn rõ ràng). Khoá qua `t()`, không một chuỗi viết
            thẳng — cùng luật NFR16/AD-21.
          -->
          <button
            type="button"
            class="btn"
            :disabled="libraryOrphanCursor === 0"
            :aria-label="t('mode.library.orphan_prev')"
            @click="dispatch('library.orphan_prev')"
          >
            ‹
          </button>
          <!-- aura-allow-text: vị trí con trỏ là SỐ ĐẾM (dữ liệu), không một câu UI. -->
          <span class="orphan-position">{{ libraryOrphanCursor + 1 }} / {{ libraryOrphans.length }}</span>
          <button
            type="button"
            class="btn"
            :disabled="libraryOrphanCursor >= libraryOrphans.length - 1"
            :aria-label="t('mode.library.orphan_next')"
            @click="dispatch('library.orphan_next')"
          >
            ›
          </button>
          <button
            type="button"
            class="btn"
            :disabled="libraryRescanBusy"
            @click="dispatch('library.forget_orphan')"
          >
            {{ t('mode.library.forget_orphan') }}
          </button>
        </div>
      </div>
    </div>

    <!--
      Story 5.9 — "Tìm kiếm full-text xuyên Library" (FR8). Đồng thời trên nguyên văn VÀ bản
      dịch, xuyên MỌI Tác phẩm — không phụ thuộc Tác phẩm nào đang mở/đang chọn trong lưới.
    -->
    <div class="search-block">
      <p class="section-heading">{{ t('mode.library.search_heading') }}</p>
      <div class="search-form">
        <label class="field">
          <span>{{ t('mode.library.search_label') }}</span>
          <input
            v-model="librarySearchQuery"
            type="text"
            autocomplete="off"
            data-library-search-input
            @keyup.enter="dispatch('library.search')"
          />
        </label>
        <button
          type="button"
          class="btn"
          data-library-search-button
          :disabled="librarySearchBusy"
          @click="dispatch('library.search')"
        >
          {{ t('mode.library.search_button') }}
        </button>
      </div>

      <!--
        Story 5.10 (FR9) — hai nút chế độ dấu, cùng khuôn nhóm-lựa-chọn của bốn nút lọc trạng
        thái ở `.filter-actions` bên dưới: mỗi lựa chọn một `dispatch` id RIÊNG (Kiểm A đòi id
        LITERAL), không một nút "đảo". `aria-pressed` đọc `librarySearchMode` -- AC4: người
        dùng đọc được chế độ ĐANG CHỌN mà không phải suy ra từ số kết quả.
      -->
      <div class="mode-actions">
        <span class="mode-actions-label">{{ t('mode.library.search_mode_heading') }}</span>
        <button
          type="button"
          class="btn"
          data-library-search-mode="exact"
          :aria-pressed="librarySearchMode === 'exact'"
          @click="dispatch('library.search_mode_exact')"
        >
          {{ t('mode.library.search_mode_exact') }}
        </button>
        <button
          type="button"
          class="btn"
          data-library-search-mode="lenient"
          :aria-pressed="librarySearchMode === 'lenient'"
          @click="dispatch('library.search_mode_lenient')"
        >
          {{ t('mode.library.search_mode_lenient') }}
        </button>
      </div>

      <!--
        role="status" TÁM NHÁNH, LUÔN có mặt (không v-if) — §Always của story: "một danh sách
        kết quả rỗng phải nói VÌ SAO nó rỗng", tám ca PHÂN BIỆT được, không gộp (xem khối 🔵 đầu
        `librarySearch.ts` cho lý do "tám" chứ không "bảy"). Thứ tự ưu tiên khớp đúng
        `librarySearch.ts` doc-comment đầu tệp.
        🔴 SỬA (vòng rà bốn lớp, mục 1) — `result_widened` PHẢI rẽ theo `librarySearchTruncated`
        NGAY BÊN TRONG nhánh của nó, cùng lý do `SearchReport::truncated` tồn tại từ Story 5.9:
        "không trần nào được cắt trong im lặng" là luật của kho, không một nhánh mới nào được
        miễn — một lượt vừa tự nới VỪA bị trần cắt mà chỉ nói "đã tự chuyển sang khoan dung dấu"
        thì đúng hình dạng, sai sự thật.
        aura-allow-text: mọi nhánh đều qua t() -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi.
      -->
      <p class="status" role="status" data-library-search-status>
        {{
          librarySearchStatusKey === 'searching'
            ? t('mode.library.search_searching')
            : librarySearchStatusKey === 'not_typed'
              ? t('mode.library.search_not_typed')
              : librarySearchStatusKey === 'index_empty'
                ? t('mode.library.search_index_empty')
                : librarySearchStatusKey === 'short_query'
                  ? t('mode.library.search_short_query')
                  : librarySearchStatusKey === 'no_match_widened'
                    ? t('mode.library.search_no_match_widened')
                    : librarySearchStatusKey === 'no_match'
                      ? t('mode.library.search_no_match')
                      : librarySearchStatusKey === 'result_widened'
                        ? librarySearchTruncated
                          ? t('mode.library.search_result_widened_truncated', { total: String(librarySearchTotal) })
                          : t('mode.library.search_result_widened', { total: String(librarySearchTotal) })
                        : librarySearchTruncated
                          ? t('mode.library.search_result_truncated', { total: String(librarySearchTotal) })
                          : t('mode.library.search_result', { total: String(librarySearchTotal) })
        }}
      </p>
      <!-- aura-allow-text: như trên, qua tError(). -->
      <p class="error" role="status">{{ librarySearchError ? tError(librarySearchError) : '' }}</p>

      <div v-if="librarySearchHits.length > 0" class="grid-nav">
        <button
          type="button"
          class="btn"
          data-library-search-prev
          :aria-label="t('mode.library.search_prev')"
          @click="dispatch('library.search_prev')"
        >
          ‹
        </button>
        <span class="grid-nav-position">{{
          t('mode.library.search_position', {
            current: String(librarySearchCursor + 1),
            total: String(librarySearchHits.length),
          })
        }}</span>
        <button
          type="button"
          class="btn"
          data-library-search-next
          :aria-label="t('mode.library.search_next')"
          @click="dispatch('library.search_next')"
        >
          ›
        </button>
      </div>

      <ul v-if="librarySearchHits.length > 0" class="search-results" data-library-search-results>
        <li
          v-for="(hit, idx) in librarySearchHits"
          :key="`${hit.work_id}-${hit.chapter_id}-${hit.segment_id ?? 'chapter'}-${hit.field}`"
          class="search-hit"
          :class="{ 'search-hit--current': idx === librarySearchCursor }"
          :aria-current="idx === librarySearchCursor ? 'true' : undefined"
          data-library-search-hit
        >
          <!-- aura-allow-text: tên Tác phẩm/tiêu đề Chương là DỮ LIỆU người dùng. -->
          <p class="search-hit-locus">
            {{ hit.work_name }} · {{ hit.chapter_title ?? t('mode.library.chapter_untitled', { ord: String(hit.chapter_ord) }) }}
            <!-- aura-allow-text: qua t() -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi. -->
            <span class="search-hit-field">{{
              hit.field === 'target' ? t('mode.library.search_field_target') : t('mode.library.search_field_source')
            }}</span>
            <!-- aura-allow-text: qua t(). -->
            <span v-if="hit.segment_id === null" class="search-hit-field">{{ t('mode.library.search_chapter_level') }}</span>
            <!-- Story 5.10 -- hit chỉ khớp qua chỉ mục KHOAN DUNG mang một nhãn phân biệt được
                 (§Always: "hai loại kết quả phân biệt được trên màn hình"). aura-allow-text: qua t(). -->
            <span v-if="hit.match_kind === 'lenient'" class="search-hit-field search-hit-field--lenient" data-library-search-hit-lenient>{{
              t('mode.library.search_hit_lenient')
            }}</span>
          </p>
          <!-- aura-allow-text: đoạn trích là DỮ LIỆU người dùng, văn bản THUẦN (AD-16) -- không v-html. -->
          <p class="search-hit-snippet">{{ hit.snippet }}</p>
          <button
            v-if="idx === librarySearchCursor"
            type="button"
            class="btn"
            data-library-search-open
            @click="dispatch('library.open_search_hit')"
          >
            {{ t('mode.library.search_open_button') }}
          </button>
        </li>
      </ul>
    </div>

    <!--
      Story 5.4 — "Bốn trạng thái vòng đời" (FR5/FR6): bốn nút lọc trạng thái + khối vòng đời
      cho Tác phẩm đang mở.
      🔵 SỬA (2026-08-28, Story 5.6) — câu cũ ("danh sách phẳng TỐI THIỂU CÓ CHỦ … Story 5.6
      sở hữu phần đó") đã HẾT ĐÚNG: lưới (bìa, tiến độ, sắp xếp, lọc lĩnh vực/ngôn ngữ, điều
      hướng bằng bàn phím) nay có mặt ngay dưới đây.
    -->
    <div class="works-block">
      <p class="section-heading">{{ t('mode.library.works_heading') }}</p>

      <!--
        🔵 THÊM (2026-08-28) `data-lifecycle-filter`/`data-lifecycle-action` — MÓC ĐỊNH DANH
        cho `e2e/specs/story-5-4-lifecycle.e2e.mjs`, cùng khuôn `data-shortcuts-open`
        (`ShortcutsOverlay`) và `data-col` (lưới) đã có. Trước đó spec định vị bằng
        `:nth-of-type(n)`: đổi thứ tự sáu nút làm spec bấm NHẦM nút và vẫn xanh, thay vì đỏ ở
        đúng chỗ vừa đổi. Một móc đọc được là rẻ hơn một lượt đỏ sai nguyên nhân.
      -->
      <!--
        🔴 BỐN nút RIÊNG, KHÔNG một `v-for` chọn id động — `check:commands` Kiểm A đòi mỗi
        `@click` là ĐÚNG MỘT lời gọi `dispatch('<id>')` với id LITERAL, đọc TĨNH được từ mã
        nguồn; một biểu thức ba ngôi chọn id lúc chạy không đọc tĩnh được (đã đo: Kiểm A đỏ).
      -->
      <div class="filter-actions">
        <button
          type="button"
          class="btn"
          data-lifecycle-filter="not_started"
          :aria-pressed="libraryStatusFilter.has('not_started')"
          :disabled="libraryWorksBusy"
          @click="dispatch('library.filter_not_started')"
        >
          {{ t('lifecycle.not_started') }}
        </button>
        <button
          type="button"
          class="btn"
          data-lifecycle-filter="in_progress"
          :aria-pressed="libraryStatusFilter.has('in_progress')"
          :disabled="libraryWorksBusy"
          @click="dispatch('library.filter_in_progress')"
        >
          {{ t('lifecycle.in_progress') }}
        </button>
        <button
          type="button"
          class="btn"
          data-lifecycle-filter="paused"
          :aria-pressed="libraryStatusFilter.has('paused')"
          :disabled="libraryWorksBusy"
          @click="dispatch('library.filter_paused')"
        >
          {{ t('lifecycle.paused') }}
        </button>
        <button
          type="button"
          class="btn"
          data-lifecycle-filter="done"
          :aria-pressed="libraryStatusFilter.has('done')"
          :disabled="libraryWorksBusy"
          @click="dispatch('library.filter_done')"
        >
          {{ t('lifecycle.done') }}
        </button>
        <button
          type="button"
          class="btn"
          data-lifecycle-action="clear_filter"
          :disabled="libraryWorksBusy || libraryFilterIsEmpty"
          @click="dispatch('library.filter_clear')"
        >
          {{ t('mode.library.clear_filter') }}
        </button>
        <button
          type="button"
          class="btn"
          data-lifecycle-action="list_works"
          :disabled="libraryWorksBusy"
          @click="dispatch('library.list_works')"
        >
          {{ t('mode.library.list_works') }}
        </button>
      </div>

      <!--
        Story 5.6 — ba `<select>` (lĩnh vực · ngôn ngữ · sắp xếp), tập-mở nên KHÔNG viết được
        thành nút literal (§Design Notes "Vì sao `<select>` chứ không thêm nút"). `@change`
        NGOÀI luật Kiểm A (`scripts/check-commands.mjs:33`). `<option>` dựng từ hai mảng DO
        RUST TRẢ VỀ (`libraryGenres`/`librarySourceLangs`, `DISTINCT` trên bảng CHƯA LỌC) —
        KHÔNG BAO GIỜ suy từ `libraryWorks` đã lọc (AD-1, §Always).
      -->
      <div class="filter-selects">
        <label class="field">
          <span>{{ t('mode.library.filter_genre_label') }}</span>
          <select
            data-library-genre-filter
            :value="libraryGenreFilter ?? ''"
            :disabled="libraryWorksBusy"
            @change="onGenreFilterChange"
          >
            <option value="">{{ t('mode.library.filter_all_genres') }}</option>
            <!-- aura-allow-text: lĩnh vực là DỮ LIỆU người dùng gõ, không một câu UI. -->
            <option v-for="genreOption in libraryGenres" :key="genreOption" :value="genreOption">
              {{ genreOption }}
            </option>
          </select>
        </label>
        <label class="field">
          <span>{{ t('mode.library.filter_source_lang_label') }}</span>
          <select
            data-library-source-lang-filter
            :value="librarySourceLangFilter ?? ''"
            :disabled="libraryWorksBusy"
            @change="onSourceLangFilterChange"
          >
            <option value="">{{ t('mode.library.filter_all_source_langs') }}</option>
            <!-- aura-allow-text: ngôn ngữ nguồn là DỮ LIỆU (mã ngôn ngữ), không một câu UI. -->
            <option v-for="langOption in librarySourceLangs" :key="langOption" :value="langOption">
              {{ langOption }}
            </option>
          </select>
        </label>
        <label class="field">
          <span>{{ t('mode.library.sort_label') }}</span>
          <select data-library-sort :value="librarySortKey" :disabled="libraryWorksBusy" @change="onSortKeyChange">
            <option value="updated_desc">{{ t('mode.library.sort_updated_desc') }}</option>
            <option value="name_asc">{{ t('mode.library.sort_name_asc') }}</option>
          </select>
        </label>
      </div>

      <!-- role="status" LUÔN có mặt (không v-if) -- cùng khuôn dải trạng thái của khối quét lại. -->
      <!-- aura-allow-text: ba nhánh đều qua t()/chuỗi rỗng -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi. -->
      <p class="status" role="status">
        {{
          !libraryWorksHaveLoaded
            ? t('mode.library.works_not_loaded')
            : libraryWorksTotal === 0
              ? t('mode.library.works_empty')
              : libraryWorksMatched === 0
                ? t('mode.library.works_no_match', { total: String(libraryWorksTotal) })
                : t('mode.library.works_result', { matched: String(libraryWorksMatched), total: String(libraryWorksTotal) })
        }}
      </p>
      <!-- aura-allow-text: như trên, qua tError(). -->
      <p class="error" role="status">{{ libraryWorksError ? tError(libraryWorksError) : '' }}</p>

      <!--
        Story 5.6 — lưới Tác phẩm (AC2/AC3/AC4/AC6/AC7), thay danh sách phẳng của Story 5.4.
        Con trỏ (`libraryWorkCursor`) di chuyển bằng HAI NÚT thật (`‹`/`›`, `dispatch()` id
        literal — Kiểm A) chứ không một `@keydown` tự chế trên từng ô: cùng khuôn
        `library.orphan_next`/`orphan_prev` đã có trong chính tệp này.
      -->
      <div v-if="libraryWorks.length > 0" class="grid-nav">
        <button
          type="button"
          class="btn"
          data-library-work-prev
          :aria-label="t('mode.library.work_prev')"
          @click="dispatch('library.work_prev')"
        >
          ‹
        </button>
        <span class="grid-nav-position">{{ t('mode.library.work_position', { current: String(libraryWorkCursor + 1), total: String(libraryWorks.length) }) }}</span>
        <button
          type="button"
          class="btn"
          data-library-work-next
          :aria-label="t('mode.library.work_next')"
          @click="dispatch('library.work_next')"
        >
          ›
        </button>
      </div>

      <ul v-if="libraryWorks.length > 0" class="works-grid" data-library-grid>
        <li
          v-for="(work, workIndex) in libraryWorks"
          ref="workCellRefs"
          :key="work.work_id"
          class="work-cell"
          :class="{ 'work-cell--current': workIndex === libraryWorkCursor }"
          :aria-current="workIndex === libraryWorkCursor ? 'true' : undefined"
          data-library-work-cell
        >
          <div class="work-cover" role="img" :aria-label="t('mode.library.cover_placeholder_label', { name: work.name })">
            <!-- aura-allow-text: biểu diễn thay thế là chữ cái đầu của TÊN, dữ liệu người dùng. -->
            {{ coverInitial(work.name) }}
          </div>
          <!-- aura-allow-text: tên Tác phẩm là DỮ LIỆU người dùng gõ, không một câu UI. -->
          <span class="work-name">{{ work.name }}</span>
          <!--
            aura-allow-text: mọi nhánh đều qua t().
            🔵 SỬA (2026-08-28, lượt rà) — TÁCH nhánh cuối làm HAI. Bản trước cho `status = null`
            (`meta.json` v1, thật sự CHƯA BIẾT) và một `status` mang giá trị NGOÀI danh mục bốn
            (dữ liệu hỏng) đọc lên GIỐNG HỆT nhau, nên một hàng hỏng đội lốt một hàng chưa di
            trú và không ai lần được. Hai câu khác nhau vì hai trạng thái khác nhau.
          -->
          <span class="work-status">
            {{
              work.status === 'not_started'
                ? t('lifecycle.not_started')
                : work.status === 'in_progress'
                  ? t('lifecycle.in_progress')
                  : work.status === 'paused'
                    ? t('lifecycle.paused')
                    : work.status === 'done'
                      ? t('lifecycle.done')
                      : work.status === null
                        ? t('mode.library.works_status_unknown')
                        : t('mode.library.works_status_invalid', { status: work.status })
            }}
          </span>
          <span v-if="work.status_is_override" class="override-marker">{{ t('mode.library.override_marker') }}</span>
          <!--
            Story 5.5 — "Tiến độ Tác phẩm" (FR7). `chapter_done_count === null` ⇒ câu "chưa
            biết" và KHÔNG vẽ thanh (§I/O Matrix: "không hiện 0 /") — một `meta.json` chưa
            từng qua `WorkMeta::rebuild_from_store` không được phép trông như "0 Chương xong".
            aura-allow-text: cả hai nhánh đi qua t() -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi.
          -->
          <span class="work-progress">
            {{
              work.chapter_done_count === null
                ? t('mode.library.works_progress_unknown')
                : t('mode.library.works_progress', {
                    done: String(work.chapter_done_count),
                    total: String(work.chapter_count),
                  })
            }}
          </span>
          <div
            v-if="work.chapter_done_count !== null"
            class="work-progress-track"
            role="progressbar"
            :aria-valuemin="0"
            :aria-valuemax="work.chapter_count"
            :aria-valuenow="work.chapter_done_count"
            :aria-valuetext="
              t('mode.library.works_progress_aria', {
                done: String(work.chapter_done_count),
                total: String(work.chapter_count),
              })
            "
          >
            <div class="work-progress-fill" :style="{ width: progressPercent(work) + '%' }"></div>
          </div>
          <!--
            Story 5.7 — "Mở Tác phẩm" (FR12). Cùng khuôn `library.forget_orphan`: thao tác
            trên `currentLibraryWork` (con trỏ), không một tham số qua `dispatch()` — nên NÚT
            chỉ hiện ở đúng ô đang chọn (`v-if`), tránh việc bấm nút ở một ô KHÁC ô cursor mà
            lại mở đúng Tác phẩm cursor đang trỏ tới, một hành vi lệch trực quan.
          -->
          <button
            v-if="workIndex === libraryWorkCursor"
            type="button"
            class="btn"
            data-library-open-work
            :disabled="libraryOpenWorkBusy"
            @click="dispatch('library.open_work')"
          >
            {{ t('mode.library.open_work_button') }}
          </button>
        </li>
      </ul>

      <div class="open-work-block">
        <p class="section-heading">{{ t('mode.library.open_work_heading') }}</p>
        <!--
          aura-allow-text: mọi nhánh đều qua t().
          🔵 SỬA (2026-08-28, lượt rà) — nhánh `!openWorkLifecycleLoaded` trước đây LUÔN nói
          "chưa có Tác phẩm nào đang mở", kể cả khi CÓ Tác phẩm mở nhưng lượt đọc trượt vì một
          lỗi khác (IPC/kho). Câu đó đẩy người dùng đi mở một Tác phẩm trong khi vấn đề thật
          nằm ở dòng lỗi ngay dưới. Nay: có lỗi ⇒ nói "chưa đọc được"; không lỗi ⇒ mới được
          nói "chưa có Tác phẩm nào đang mở".
        -->
        <p class="status" role="status">
          {{
            !openWorkLifecycleLoaded
              ? openWorkLifecycleError
                ? t('mode.library.open_work_lifecycle_unreadable')
                : t('mode.library.open_work_not_loaded')
              : openWorkLifecycleStatus === 'not_started'
                ? t('lifecycle.not_started')
                : openWorkLifecycleStatus === 'in_progress'
                  ? t('lifecycle.in_progress')
                  : openWorkLifecycleStatus === 'paused'
                    ? t('lifecycle.paused')
                    : openWorkLifecycleStatus === 'done'
                      ? t('lifecycle.done')
                      : openWorkLifecycleStatus === null
                        ? t('mode.library.works_status_unknown')
                        : t('mode.library.works_status_invalid', { status: openWorkLifecycleStatus })
          }}
          <template v-if="openWorkLifecycleLoaded && openWorkLifecycleIsOverride">
            {{ t('mode.library.override_marker') }}
          </template>
        </p>
        <!-- aura-allow-text: như trên, qua tError(). -->
        <p class="error" role="status">{{ openWorkLifecycleError ? tError(openWorkLifecycleError) : '' }}</p>
        <div class="open-work-actions">
          <button
            type="button"
            class="btn"
            data-lifecycle-action="set_override_paused"
            :disabled="openWorkLifecycleBusy || !openWorkLifecycleLoaded"
            @click="dispatch('lifecycle.set_work_override_paused')"
          >
            {{ t('mode.library.set_override_paused') }}
          </button>
          <button
            type="button"
            class="btn"
            data-lifecycle-action="clear_override"
            :disabled="openWorkLifecycleBusy || !openWorkLifecycleLoaded || !openWorkLifecycleIsOverride"
            @click="dispatch('lifecycle.clear_work_override')"
          >
            {{ t('mode.library.clear_override') }}
          </button>
          <button
            type="button"
            class="btn"
            data-lifecycle-action="set_chapter_done"
            :disabled="openWorkLifecycleBusy || !openWorkLifecycleLoaded"
            @click="dispatch('lifecycle.set_chapter_done')"
          >
            {{ t('mode.library.set_chapter_done') }}
          </button>
        </div>

        <!--
          Story 5.7 — "Danh sách Chương và mở Chương vào Workspace" (FR12). Cùng khối với
          "Trạng thái Tác phẩm đang mở" ở trên: cả hai nói về CHÍNH Tác phẩm đang mở.
        -->
        <div class="chapters-block">
          <p class="section-heading">{{ t('mode.library.chapters_heading') }}</p>
          <!--
            role="status" LUÔN có mặt -- §Always của story: "danh sách rỗng phải nói vì sao nó
            rỗng", ba câu phân biệt được (chưa tải · đã tải rỗng · đã tải có n).
            aura-allow-text: mọi nhánh đều qua t() -- Kiểm A2 không đọc tĩnh được toán tử ba ngôi.
          -->
          <p class="status" role="status">
            {{
              !libraryChaptersHaveLoaded
                ? t('mode.library.chapters_not_loaded')
                : libraryChapters.length === 0
                  ? t('mode.library.chapters_empty')
                  : t('mode.library.chapters_result', { total: String(libraryChapters.length) })
            }}
          </p>
          <!-- aura-allow-text: như trên, qua tError(). -->
          <p class="error" role="status">{{ libraryChaptersError ? tError(libraryChaptersError) : '' }}</p>
          <!--
            Câu báo khi `openWorkById` bị CHẶN vì tập chờ Editor chưa sạch (§Always) — KHÔNG
            phải một `IpcError` (đúng khuôn `err.editor.flush_failed` của `libraryImport.ts`).
            aura-allow-text: mọi nhánh đều qua t()/tError().
          -->
          <p class="error" role="status">
            {{
              libraryOpenWorkError
                ? tError(libraryOpenWorkError)
                : libraryOpenWorkNotice === 'flush-failed'
                  ? t('mode.library.open_work_flush_failed')
                  : libraryOpenWorkNotice === 'still-dirty'
                    ? t('mode.library.open_work_still_dirty')
                    : ''
            }}
          </p>

          <div v-if="libraryChapters.length > 0" class="grid-nav">
            <button
              type="button"
              class="btn"
              data-library-chapter-prev
              :aria-label="t('mode.library.chapter_prev')"
              @click="dispatch('library.chapter_prev')"
            >
              ‹
            </button>
            <span class="grid-nav-position">{{
              t('mode.library.chapter_position', {
                current: String(libraryChapterCursor + 1),
                total: String(libraryChapters.length),
              })
            }}</span>
            <button
              type="button"
              class="btn"
              data-library-chapter-next
              :aria-label="t('mode.library.chapter_next')"
              @click="dispatch('library.chapter_next')"
            >
              ›
            </button>
            <button
              type="button"
              class="btn"
              data-library-open-chapter
              :disabled="libraryChaptersBusy || libraryOpenWorkBusy"
              @click="dispatch('library.open_chapter')"
            >
              {{ t('mode.library.open_chapter_button') }}
            </button>
          </div>

          <!--
            🔴 Story 5.8 — "Tổ chức lại Chương sau khi nhập" (FR15 · AD-32). Ba thao tác làm
            việc trên một CHƯƠNG, nên chúng sống ở đây, cạnh con trỏ Chương. Thao tác thứ tư
            (tách) làm việc trên một CÂU và sống ở Editor — `editor.split_chapter`,
            `Mod+Shift+Slash`; §Never của story cấm dựng một ô nhập số thứ tự câu ở đây.

            Mỗi `@click` là ĐÚNG MỘT lời gọi `dispatch('<id>')` với id literal — `check:commands`
            Kiểm A, AD-34 §1.

            ⚠️ `:disabled` đọc CẢ BA cờ bận: một lượt tổ chức đang bay
            (`libraryChapterReorgBusy`), một lượt tải danh sách (`libraryChaptersBusy`), và một
            lượt đổi Tác phẩm (`libraryOpenWorkBusy`) — cùng lý do khối 🔴 "CỬA SỔ MỞ NHẦM
            CHƯƠNG" của `libraryChapters.ts` đã ghi: `chapter.id` là `AUTOINCREMENT` CỤC BỘ
            trong từng `project.db`, nên một id của Tác phẩm CŨ tồn tại thật trong kho MỚI và
            một thao tác GHI trên nó đi qua sạch, không một lỗi nào.
          -->
          <div v-if="libraryChapters.length > 0" class="chapter-reorg">
            <label class="chapter-rename">
              <span>{{ t('mode.library.chapter_rename_label') }}</span>
              <input v-model="chapterRenameDraft" type="text" autocomplete="off" data-library-chapter-rename-input />
            </label>
            <div class="chapter-reorg-buttons">
              <button
                type="button"
                class="btn"
                data-library-chapter-rename
                :disabled="libraryChapterReorgBusy || libraryChaptersBusy || libraryOpenWorkBusy || currentLibraryChapter === null"
                @click="dispatch('library.chapter_rename')"
              >
                {{ t('mode.library.chapter_rename_button') }}
              </button>
              <button
                type="button"
                class="btn"
                data-library-chapter-move-up
                :disabled="libraryChapterReorgBusy || libraryChaptersBusy || libraryOpenWorkBusy || currentLibraryChapter === null"
                @click="dispatch('library.chapter_move_up')"
              >
                {{ t('mode.library.chapter_move_up') }}
              </button>
              <button
                type="button"
                class="btn"
                data-library-chapter-move-down
                :disabled="libraryChapterReorgBusy || libraryChaptersBusy || libraryOpenWorkBusy || currentLibraryChapter === null"
                @click="dispatch('library.chapter_move_down')"
              >
                {{ t('mode.library.chapter_move_down') }}
              </button>
              <button
                type="button"
                class="btn"
                data-library-chapter-merge-up
                :disabled="libraryChapterReorgBusy || libraryChaptersBusy || libraryOpenWorkBusy || currentLibraryChapter === null"
                @click="dispatch('library.chapter_merge_up')"
              >
                {{ t('mode.library.chapter_merge_up') }}
              </button>
            </div>
          </div>
          <!--
            Câu báo của một lượt tổ chức — cùng khuôn khối `libraryOpenWorkError` ngay trên, và
            cùng lý do: một lượt CHẶN vì tập chờ Editor chưa sạch KHÔNG phải một `IpcError`,
            nên nó đi qua `t()` chứ không qua `tError()`.
            aura-allow-text: mọi nhánh đều qua t()/tError().
          -->
          <p class="error" role="status">
            {{
              libraryChapterReorgError
                ? tError(libraryChapterReorgError)
                : libraryChapterReorgNotice === 'flush-failed'
                  ? t('mode.library.chapter_reorg_flush_failed')
                  : libraryChapterReorgNotice === 'still-dirty'
                    ? t('mode.library.chapter_reorg_still_dirty')
                    : ''
            }}
          </p>

          <!--
            Cuộn CÓ CỬA SỔ (AC2) — `chapterWindow()` là một hàm THUẦN (`libraryChapters.ts`),
            đây chỉ nối `@scroll` → `scrollTop` và render `slice(start, end)` cộng hai `<li>`
            đệm mang chiều cao tính sẵn. KHÔNG một dòng `scrollTop`/`scrollIntoView` nào tính
            vị trí LÀM VIỆC của Editor ở đây — đây là một cơ chế KHÁC hẳn AC4 (AD-3).
          -->
          <ul
            v-if="libraryChapters.length > 0"
            ref="chaptersScrollRef"
            class="chapters-list"
            data-library-chapters-list
            @scroll="onChaptersScroll"
          >
            <li class="chapters-pad" :style="{ height: chaptersWindowSlice.padTop + 'px' }" aria-hidden="true"></li>
            <li
              v-for="(chapter, idx) in visibleChapters"
              :key="chapter.chapter_id"
              class="chapter-row"
              :class="{ 'chapter-row--current': chaptersWindowSlice.start + idx === libraryChapterCursor }"
              :aria-current="chaptersWindowSlice.start + idx === libraryChapterCursor ? 'true' : undefined"
              data-library-chapter-row
              :style="{ height: CHAPTER_ROW_HEIGHT_PX + 'px' }"
            >
              <!-- aura-allow-text: thứ tự Chương là SỐ ĐẾM (dữ liệu), không một câu UI. -->
              <span class="chapter-ord">{{ chapter.ord }}</span>
              <!-- aura-allow-text: tiêu đề Chương là DỮ LIỆU người dùng, `chapter_untitled` qua t(). -->
              <span class="chapter-title">{{
                chapter.title ?? t('mode.library.chapter_untitled', { ord: String(chapter.ord) })
              }}</span>
              <!-- aura-allow-text: mọi nhánh đều qua t(); chapter.status luôn là một trong bốn giá trị (NOT NULL ở Rust). -->
              <span class="chapter-status">
                {{
                  chapter.status === 'not_started'
                    ? t('lifecycle.not_started')
                    : chapter.status === 'in_progress'
                      ? t('lifecycle.in_progress')
                      : chapter.status === 'paused'
                        ? t('lifecycle.paused')
                        : chapter.status === 'done'
                          ? t('lifecycle.done')
                          : chapter.status
                }}
              </span>
              <!-- aura-allow-text: số câu là SỐ ĐẾM (dữ liệu), không một câu UI. -->
              <span class="chapter-segment-count">{{ chapter.segment_count }}</span>
            </li>
            <li class="chapters-pad" :style="{ height: chaptersWindowSlice.padBottom + 'px' }" aria-hidden="true"></li>
          </ul>
        </div>
      </div>
    </div>

    <!--
      🔴 Xác nhận đứng CẠNH form, KHÔNG bọc form trong một `v-else`. Bản trước dùng
      `v-if`/`v-else` và `createdWork` không bao giờ được đặt lại, nên tạo xong Tác phẩm
      đầu tiên là form VÀ dải báo lỗi biến mất vĩnh viễn trong phiên. Code review
      2026-08-06.
    -->
    <div class="empty">
      <div class="rule" aria-hidden="true"></div>
      <!--
        Story 5.6, AC5 — câu "Library chưa có Tác phẩm nào" (`mode.library.status`) không
        được phép nói khi ĐÃ có Tác phẩm, và không được nói TRƯỚC khi lượt tải xong (đó là
        "chưa biết", không phải "không có" — `AGENTS.md::Known pitfalls`). Khối giải thích
        đứng TRƯỚC form (lời mời nhập) khi library rỗng thật; form vẫn LUÔN hiện (thêm Tác
        phẩm không bị khoá lại chỉ vì đã có sẵn Tác phẩm khác).
      -->
      <template v-if="libraryWorksHaveLoaded && libraryWorksTotal === 0">
        <p class="big">{{ t('mode.library.status') }}</p>
        <p class="small">{{ t('mode.library.empty_body') }}</p>
      </template>

      <form class="import-form" @submit.prevent>
        <label class="field">
          <span>{{ t('mode.library.field_name') }}</span>
          <input v-model="name" type="text" autocomplete="off" />
        </label>

        <label class="field">
          <span>{{ t('mode.library.field_source_lang') }}</span>
          <select v-model="sourceLang">
            <option value="zh">{{ t('mode.library.lang_zh') }}</option>
            <option value="en">{{ t('mode.library.lang_en') }}</option>
          </select>
        </label>

        <label class="field">
          <span>{{ t('mode.library.field_genre') }}</span>
          <input v-model="genre" type="text" autocomplete="off" />
        </label>

        <label class="field">
          <span>{{ t('mode.library.field_paste') }}</span>
          <textarea v-model="pastedText" rows="6"></textarea>
        </label>
        <button
          type="button"
          class="btn"
          :disabled="busy || pastedText.trim() === ''"
          @click="dispatch('library.import_text')"
        >
          {{ t('mode.library.submit_text') }}
        </button>

        <!--
          🔴 Vùng kéo-thả — một CHỈ BÁO, không phải một điều khiển. Nó không mang
          `tabindex`: thả một tệp chỉ **điền vào ô đường dẫn ngay dưới**, rồi người dùng
          bấm cùng cái nút mà đường bàn phím bấm — nên không có thao tác nào riêng của
          vùng này để một chặng Tab dẫn tới. Bản trước có `tabindex="0"` mà không
          `role`, không `@keydown`: một chặng Tab ăn tiêu điểm rồi không phản hồi
          phím nào. Code review 2026-08-06.

          ⇒ NFR17 đạt bằng chính cấu trúc này: **mọi** đường mở tệp kết thúc ở ô đường dẫn
          + nút, cả hai đều là điều khiển HTML gốc. Kéo-thả chỉ là lối tắt cho chuột.
        -->
        <div class="dropzone" :class="{ over: isDragOver }">
          {{ t('mode.library.dropzone') }}
        </div>

        <label class="field">
          <span>{{ t('mode.library.field_path') }}</span>
          <input v-model="filePath" type="text" autocomplete="off" />
        </label>
        <button
          type="button"
          class="btn"
          :disabled="busy || filePath.trim() === ''"
          @click="dispatch('library.import_file')"
        >
          {{ t('mode.library.submit_file') }}
        </button>
      </form>

      <!--
        Ba node LUÔN có mặt (không `v-if`) để trình đọc màn hình công bố được nội
        dung ĐỔI — cùng lý lẽ với dải báo lỗi cấu hình của `App.vue`. `role="status"`,
        không `role="alert"`: đây là kết quả một thao tác, không phải tình huống khẩn.
      -->
      <!-- aura-allow-text: cả hai nhánh đi qua t()/chuỗi rỗng, không chuỗi viết thẳng
           nào — Kiểm A2 không đọc tĩnh được toán tử ba ngôi. -->
      <p class="status" role="status">
        {{ createdWork ? t('mode.library.created', { name: createdWork.meta.name, folder: createdWork.folder }) : '' }}
      </p>
      <!-- aura-allow-text: như trên. -->
      <p class="notice" role="status">{{ noticeKey ? t(noticeKey) : '' }}</p>
      <!-- aura-allow-text: như trên, qua tError(). -->
      <p class="error" role="status">{{ lastError ? tError(lastError) : '' }}</p>
    </div>
  </section>
</template>

<style scoped>
.mode {
  height: 100%;
  padding: var(--space-panel-block) var(--space-panel-inline);
  overflow-y: auto;
}

/*
 * ⚠️ `outline: none` CHỈ ở đây, và chỉ vì phần tử này mang `tabindex="-1"`: nó là một
 * vùng chứa nhận focus lập trình, không phải một điều khiển tương tác. Chỉ báo tiêu điểm
 * thật của sản phẩm là vạch dọc ở `PanelFrame` (AC5) — và, ở trong tệp này, vòng focus
 * của `.dropzone:focus-visible` bên dưới.
 *
 * Đừng nhân luật này ra thành `*:focus { outline: none }`. Đó là cách nhanh nhất phá
 * NFR17 (*"trạng thái focus luôn nhìn thấy rõ"*) mà vẫn qua được MỌI cổng hiện có —
 * `check-tokens.mjs` canh màu, cỡ chữ, tương phản, opacity và elevation, KHÔNG canh focus
 * ring. §Trap 4 của story 1.6.
 */
.mode:focus {
  outline: none;
}

.status {
  margin: 12px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.notice {
  margin: 4px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.empty {
  max-width: 420px;
}

.rule {
  width: 34px;
  height: 2px;
  background: var(--color-ornament);
  border-radius: 1px;
  margin-bottom: var(--space-unit);
}

.big {
  margin: 0 0 8px;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.small {
  margin: 0 0 16px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.import-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.field input,
.field select,
.field textarea {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 6px 8px;
}

.btn {
  align-self: flex-start;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-primary);
  background: var(--color-surface-sunken);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 6px 12px;
  cursor: pointer;
}

.btn:disabled {
  color: var(--color-on-surface-variant);
  cursor: not-allowed;
}

.dropzone {
  border: 1px dashed var(--color-outline);
  border-radius: 4px;
  padding: 16px;
  text-align: center;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  background: var(--color-surface-sunken);
}

.dropzone.over {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

/*
 * Không còn luật `:focus-visible` cho `.dropzone` — nó không mang `tabindex` nữa nên
 * không nhận tiêu điểm được (xem comment trong `<template>`). Chỉ báo tiêu điểm thật của
 * màn này là chỉ báo gốc của trình duyệt trên `input`/`select`/`textarea`/`button`.
 *
 * Đừng nhân luật `outline: none` ở `.mode:focus` ra thành `*:focus` — đó là cách nhanh
 * nhất phá NFR17 mà vẫn qua được MỌI cổng hiện có. §Trap 4 của story 1.6.
 */

.error {
  margin: 12px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

/* Story 5.3 — "Quét lại thư mục". Cùng khuôn token/cỡ chữ với `.empty`/`.field` ở trên. */
.root-block {
  max-width: 420px;
  margin-bottom: var(--space-panel-block);
  padding-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

.section-heading {
  margin: 0 0 8px;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.root-value {
  margin: 0 0 10px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  word-break: break-all;
}

.root-missing {
  margin: 4px 0 0;
  min-height: 1em;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}

.root-actions {
  display: flex;
  gap: 8px;
}

/**
 * 🔵 THÊM (2026-08-27, phán quyết Ice #3) — cùng khuôn `.root-missing` ngay trên: một cảnh
 * báo phát sinh TỪ một lượt quét (không phải một lỗi thao tác). Sống bên trong `.root-block`
 * (thừa `max-width` của khối cha) — chỉ khác `.root-missing` ở chỗ nó có thể mang HAI đường
 * dẫn dài trong cùng một câu, nên `word-break` được đặt tường minh tại đây thay vì dựa vào
 * kế thừa. Toàn bộ màu/cỡ chữ qua token — `check:tokens` cấm viết thẳng.
 */
.conflict-warning {
  margin: 8px 0 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
  word-break: break-all;
}

.orphan-row {
  margin-top: 10px;
  padding: 10px;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  background: var(--color-surface-sunken);
}

.orphan-name {
  margin: 0 0 4px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.orphan-path {
  margin: 0 0 8px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  word-break: break-all;
}

.orphan-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.orphan-position {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/* Story 5.4 — "Bốn trạng thái vòng đời". Cùng khuôn token/cỡ chữ với `.root-block` ở trên. */
.works-block {
  max-width: 420px;
  margin-bottom: var(--space-panel-block);
  padding-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

.filter-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 10px;
}

/*
 * Nút lọc đang BẬT phân biệt bằng `aria-pressed` (đọc được bằng trình đọc màn hình) VÀ
 * bằng token màu — cùng khuôn `.dropzone.over` ở trên: không dùng bóng đổ/gradient (AD-21).
 */
.filter-actions .btn[aria-pressed='true'] {
  color: var(--color-primary);
  border-color: var(--color-primary);
}

/*
 * Story 5.6 — ba `<select>` lọc/sắp, cùng khuôn `.field` (đã khai ở trên cho form nhập) —
 * không đúc thêm một kiểu ô nhập thứ hai cho cùng một vai.
 */
.filter-selects {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 10px;
}

.filter-selects .field {
  min-width: 120px;
}

/* Story 5.6 — thanh điều hướng con trỏ lưới (AC7). Hai nút `‹`/`›` cộng vị trí "k / n". */
.grid-nav {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 10px 0 0;
}

.grid-nav-position {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/* Story 5.6 — lưới Tác phẩm, thay `.works-list` (danh sách phẳng) của Story 5.4. */
.works-grid {
  list-style: none;
  margin: 10px 0 0;
  padding: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
  gap: 10px;
}

.work-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  background: var(--color-surface-sunken);
}

/*
 * Ô ĐANG CHỌN (AC7) — phân biệt bằng BỀ RỘNG viền (1px → 2px) CỘNG màu, không chỉ màu
 * (WCAG AA: một tín hiệu chỉ-màu không đủ cho người không phân biệt được màu sắc). Cùng
 * chủ ý `.filter-actions .btn[aria-pressed='true']` ở trên — `aria-current` đã tự nói phần
 * còn lại cho trình đọc màn hình.
 */
.work-cell--current {
  border-width: 2px;
  border-color: var(--color-primary);
}

/*
 * Khung bìa — biểu diễn thay thế NHẤT QUÁN (AC6): ô vuông, chữ cái đầu của tên trên nền
 * token. KHÔNG ảnh, KHÔNG gradient/bóng đổ (AD-21) — xem §Design Notes "Vì sao KHÔNG thêm
 * cột `cover`" của story: đây là TOÀN BỘ khung bìa hôm nay, không phải một trạng thái tạm.
 */
.work-cover {
  display: flex;
  align-items: center;
  justify-content: center;
  aspect-ratio: 1 / 1;
  border-radius: 4px;
  background: var(--color-surface-accent);
  color: var(--color-primary);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  font-weight: var(--weight-ui-md-strong);
}

.work-name {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
  word-break: break-all;
}

.work-status {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/*
 * Dấu ghi đè thủ công — chữ, không ký hiệu đúc mới (§Never của story: "đúc ký hiệu mới cho
 * dấu 'ghi đè thủ công' -- dấu phân biệt viết thành CHỮ qua t()").
 */
.override-marker {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-primary);
}

/* Story 5.5 — "Tiến độ Tác phẩm" (FR7). Chữ "k / n" hoặc "chưa biết", cùng cỡ `.work-status`. */
.work-progress {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/*
 * Thanh tiến độ — chỉ TOKEN, không màu viết thẳng/gradient/bóng đổ (§Always của story).
 * 🔵 SỬA (2026-08-28, Story 5.6) — gỡ `flex-basis: 100%`: `.work-cell` nay là
 * `flex-direction: column` (thay `.works-row` cũ, `flex-wrap: wrap` theo hàng ngang), nên
 * mọi con đã tự trải hết bề rộng theo mặc định `align-items: stretch` — không cần buộc nữa.
 */
.work-progress-track {
  height: 4px;
  border-radius: var(--radius-full);
  background: var(--color-outline);
  overflow: hidden;
}

.work-progress-fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: var(--radius-full);
}

.open-work-block {
  margin-top: var(--space-panel-block);
  padding-top: var(--space-panel-block);
  border-top: 1px solid var(--color-outline);
}

.open-work-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 8px;
}

/*
 * Story 5.8 — khối tổ chức lại Chương. Mọi màu và cỡ chữ đến từ token (`check:tokens` Kiểm
 * B/B2); không bóng đổ, không gradient, không lớp nổi (Kiểm F). `.chapter-rename` chép đúng
 * hình dạng `.field` của form nhập ở trên thay vì đặt một hình dạng thứ hai cho cùng vai.
 */
.chapter-reorg {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 10px;
}

.chapter-rename {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.chapter-rename input {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  padding: 6px 8px;
}

.chapter-reorg-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

/* Story 5.7 — danh sách Chương, cùng khuôn `.open-work-block` (đường viền trên tách khối). */
.chapters-block {
  margin-top: var(--space-panel-block);
  padding-top: var(--space-panel-block);
  border-top: 1px solid var(--color-outline);
}

/*
 * `overflow-y: auto` + `max-height` cố định — điều kiện của cuộn CÓ CỬA SỔ (AC2): `@scroll`
 * chỉ bắn khi phần tử THẬT SỰ cuộn được. `max-height` phải khớp `CHAPTER_VIEWPORT_HEIGHT_PX`
 * ở `<script setup>` — hai con số lệch nhau làm cửa sổ tính sai so với khung nhìn thật.
 */
.chapters-list {
  list-style: none;
  margin: 10px 0 0;
  padding: 0;
  max-height: 240px;
  overflow-y: auto;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
}

.chapters-pad {
  margin: 0;
  padding: 0;
  list-style: none;
}

.chapter-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 8px;
  border-bottom: 1px solid var(--color-outline);
}

.chapter-row--current {
  background: var(--color-surface-sunken);
}

.chapter-ord {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
  min-width: 2em;
  text-align: right;
}

.chapter-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.chapter-status,
.chapter-segment-count {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/* Story 5.9 — "Tìm kiếm full-text xuyên Library". Cùng khuôn token/cỡ chữ với `.root-block`. */
.search-block {
  max-width: 420px;
  margin-bottom: var(--space-panel-block);
  padding-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline);
}

.search-form {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  margin-bottom: 10px;
}

.search-form .field {
  flex: 1;
}

/* Story 5.10 — hai nút chế độ dấu, cùng khuôn `.filter-actions` (token màu cho trạng thái
   ĐANG BẬT, không bóng đổ/gradient — AD-21). */
.mode-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.mode-actions-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

.mode-actions .btn[aria-pressed='true'] {
  color: var(--color-primary);
  border-color: var(--color-primary);
}

.search-results {
  margin: 10px 0 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.search-hit {
  padding: 10px;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  background: var(--color-surface-sunken);
}

.search-hit--current {
  border-color: var(--color-primary);
}

.search-hit-locus {
  margin: 0 0 4px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
}

.search-hit-field {
  margin-left: 6px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: normal;
  color: var(--color-on-surface-variant);
}

/* Story 5.10 — nhãn "khoan dung dấu" trên một hit chỉ khớp qua `_nd`, phân biệt bằng TOKEN
   MÀU (không bóng đổ/gradient — AD-21), cùng token `--color-primary` đã dùng cho trạng thái
   ĐANG BẬT của các nút chế độ ngay trên. */
.search-hit-field--lenient {
  color: var(--color-primary);
}

.search-hit-snippet {
  margin: 0 0 8px;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  word-break: break-word;
}
</style>
