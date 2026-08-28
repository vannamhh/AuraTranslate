<script setup lang="ts">
// Chế độ 1/3 — kho Tác phẩm và **điểm vào ứng dụng** (PRD §5.2). Story 1.6 · AC3 · AC4.
// Story 1.15 thêm đường nhập tối thiểu (AC1/AC8/NFR17): dán văn bản · kéo-thả · ô nhập
// đường dẫn — đủ để có văn bản mà Story 1.16 hiển thị ở Panel Source.
//
// Lưới Tác phẩm (bìa, sắp xếp, lọc thể loại/ngôn ngữ) thuộc Story 5.6 — đây vẫn là một
// khung rỗng cho phần đó. 🔵 SỬA (2026-08-27, Story 5.4) — câu cũ gộp CẢ "bốn trạng thái
// vòng đời" vào phần chưa dựng đã HẾT ĐÚNG: khối "Tác phẩm" (danh sách phẳng + bốn nút lọc)
// và khối "Trạng thái Tác phẩm đang mở" (ba lệnh vòng đời) nay có mặt ở đây.
//
// Không chuỗi tiếng Việt nào trong tệp này (NFR16, AD-21) — mọi nhãn đi qua `t()`.
import { onActivated, onBeforeUnmount, onMounted, useTemplateRef, watch } from 'vue'
import { declareFocus, dispatch, enterFocus, releaseFocus } from '../commands'
import type { WorkRow } from '../config/library'
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
  libraryStatusFilter,
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
} from './libraryWorks'
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
      Story 5.4 — "Bốn trạng thái vòng đời" (FR5/FR6). Danh sách phẳng TỐI THIỂU CÓ CHỦ
      (§Never: không bìa, không thanh tiến độ, không sắp xếp, không lọc thể loại/ngôn ngữ,
      không điều hướng lưới bằng bàn phím — Story 5.6 sở hữu phần đó) cộng bốn nút lọc và
      khối vòng đời cho Tác phẩm đang mở.
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

      <ul v-if="libraryWorks.length > 0" class="works-list">
        <li v-for="work in libraryWorks" :key="work.work_id" class="works-row">
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
      <p class="big">{{ t('mode.library.status') }}</p>
      <p class="small">{{ t('mode.library.empty_body') }}</p>

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

.works-list {
  list-style: none;
  margin: 10px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.works-row {
  display: flex;
  align-items: baseline;
  /*
   * 🔵 THÊM `flex-wrap` (2026-08-28, Story 5.5) — thanh tiến độ (`.work-progress-track`) đặt
   * `flex-basis: 100%` để luôn XUỐNG DÒNG RIÊNG, không chen vào hàng chữ đầu. Bốn `span` cũ
   * (tên/trạng thái/ghi đè/tiến độ chữ) vẫn cùng một hàng như trước.
   */
  flex-wrap: wrap;
  gap: 8px;
  padding: 6px 8px;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  background: var(--color-surface-sunken);
}

.work-name {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  font-weight: var(--weight-ui-md-strong);
  color: var(--color-on-surface);
  flex: 1;
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
 * `flex-basis: 100%` buộc nó xuống dòng riêng trong `.works-row` đã `flex-wrap`.
 */
.work-progress-track {
  flex-basis: 100%;
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
</style>
