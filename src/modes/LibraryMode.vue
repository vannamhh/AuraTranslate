<script setup lang="ts">
// Chế độ 1/3 — kho Tác phẩm và **điểm vào ứng dụng** (PRD §5.2). Story 1.6 · AC3 · AC4.
// Story 1.15 thêm đường nhập tối thiểu (AC1/AC8/NFR17): dán văn bản · kéo-thả · ô nhập
// đường dẫn — đủ để có văn bản mà Story 1.16 hiển thị ở Panel Source.
//
// Lưới Tác phẩm, bộ lọc, sắp xếp và bốn trạng thái vòng đời thuộc Epic 5 — đây vẫn là
// một khung rỗng cho phần đó, chỉ thêm đường vào.
//
// Không chuỗi tiếng Việt nào trong tệp này (NFR16, AD-21) — mọi nhãn đi qua `t()`.
import { onActivated, onBeforeUnmount, onMounted, useTemplateRef } from 'vue'
import { declareFocus, dispatch, enterFocus, releaseFocus } from '../commands'
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
      <!-- aura-allow-text: `currentLibraryRoot` là ĐƯỜNG DẪN đĩa (dữ liệu), không câu UI; nhánh
           còn lại đi qua t() -- Kiểm A2 không đọc tĩnh được toán tử `??`. -->
      <p class="root-value">{{ currentLibraryRoot ?? t('mode.library.root_not_scanned_yet') }}</p>
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
</style>
