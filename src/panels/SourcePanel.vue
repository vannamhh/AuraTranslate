<script setup lang="ts">
// Panel `Nguyên văn` — Story 1.16, AC1 · AC2 · AC3 · AC6 · AC9.
//
// ⚠️ `owner` và `status-key` viết **literal**, không qua biến: Kiểm E của
// `npm run check:commands` đọc tĩnh hai thuộc tính này để đối chiếu HAI CHIỀU với
// `FOCUS_OWNERS` và với `vi.json`. Một biểu thức ở đây bị đếm rồi bỏ qua — tức mất lưới.
//
// 🔴 State (Chương đã nạp, tab/kiểu xem đang chọn) sống ở `sourcePanelState.ts`, KHÔNG
// trong `ref` cục bộ ở đây — AC9: một lượt đổi preset tháo và dựng lại instance này, và
// state module-level là thứ sống sót qua lượt đó (xem doc-comment đầu tệp kia).
import { computed, onMounted, useTemplateRef } from 'vue'
import PanelFrame from './PanelFrame.vue'
import SourceHanViet from './SourceHanViet.vue'
import type { DockviewPanelProps } from '../layout/panelProps'
import { t } from '../i18n'
import { dispatch } from '../commands'
import { useSelectionSurface } from './selectionContract'
import {
  activeTab,
  canUseParallelView,
  ensureChapterLoaded,
  sourceChapter,
  sourceChapterError,
  viewMode,
} from './sourcePanelState'

// dockview mount mọi component nội dung với đúng một prop tên `params`. Panel này không
// đọc gì trong đó — khai ra để nó không rơi xuống thành thuộc tính DOM `params="[object
// Object]"` trên phần tử gốc.
defineProps<DockviewPanelProps>()

onMounted(() => {
  // Idempotent — xem doc-comment `ensureChapterLoaded`. Gọi lại mỗi lượt mount (kể cả sau
  // một lượt đổi preset) là AN TOÀN và KHÔNG chạy lại IPC.
  void ensureChapterLoaded()
})

const hasChapter = computed(() => sourceChapter.value !== null)
/** AC3 — phân biệt đọc từ `source_lang`, trường BẤT BIẾN, không đoán từ nội dung. */
const showHanVietTab = computed(() => sourceChapter.value?.source_lang === 'zh')
/**
 * Ngắt dòng chuẩn hoá CHỈ ở trình bày (Bẫy `deferred-work.md:527`).
 *
 * 🔴 `\r\n?` → `\n`, không xoá trắng `\r` — xem doc-comment `NEWLINES` ở
 * `SourceHanViet.vue`: xoá trắng làm một tệp khuôn Mac cổ điển dính thành một dòng.
 */
const displayText = computed(() => (sourceChapter.value?.source_text ?? '').replace(/\r\n?/g, '\n'))

/**
 * 🔴 Chương đã nạp nhưng nội dung RỖNG — một khung trống câm là thứ người dùng đọc thành
 * *"hỏng"* (doc-comment `PanelFrame.vue`). `create_work_from_file` không có sàn dưới nên
 * một tệp 0 byte đi thẳng tới đây. Bắt ở lượt code review 2026-08-06.
 */
const isEmptyChapter = computed(() => (sourceChapter.value?.source_text ?? '').trim() === '')

/**
 * 🔴 Lỗi đọc Chương phải HIỆN RA — bản đầu export `sourceChapterError` mà không một chỗ
 * nào tiêu thụ, nên `err.project.no_work_open` *(khoá mới của AC8, có `MessageKey` riêng và
 * ba test Rust)* không bao giờ tới được màn hình, và một `store.read_failed` **kho hỏng**
 * hiện ra thành câu trạng thái bình thường *"Chưa có Chương nào được mở."*
 *
 * Đúng lớp lỗi mà §Trí tuệ từ story trước cảnh báo: *"nuốt nó … cho ra thất bại IM LẶNG ở
 * đúng thao tác đầu tiên."* Bắt ở lượt code review 2026-08-06.
 */
const chapterErrorKey = computed(() => sourceChapterError.value?.message_key ?? null)

/**
 * Câu trạng thái mặc định của `PanelFrame` chỉ đúng khi **không** có gì để nói: có lỗi
 * hay có Chương rỗng thì panel tự nói bằng chuỗi của chính nó.
 */
const showFrameStatus = computed(() => !hasChapter.value && chapterErrorKey.value === null)

/**
 * AC1/AC2 — bề mặt nguyên văn khai token CỦA CHÍNH NÓ theo `source_lang`: `source-cjk`
 * (họ `read-cjk`, chỉ tiếng Trung) cho `zh`, `source-latin` (Quyết định #6) cho `en`.
 * Không để `source-cjk` gánh cả hai — họ `read-cjk` dựng chữ Latin bằng font CJK.
 */
const originalTokenClass = computed(() => (showHanVietTab.value ? 'tok-source-cjk' : 'tok-source-latin'))

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 1.18 · AC2 — ĐĂNG KÝ BỀ MẶT NGUYÊN VĂN LÀM NGUỒN VÙNG CHỌN
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ Đăng ký `.original`, không `.source-body`: dải tab (`role="tablist"`) nằm trong
// `.source-body`, và bôi đen nhãn một cái tab (*"Hán Việt"*) sẽ phát một lượt tra cho một
// chuỗi GIAO DIỆN. Bề mặt nguồn phải đúng bằng bề mặt CHỮ.
//
// ⚠️ Bề mặt Hán Việt tự đăng ký LẤY (`SourceHanViet.vue`) — nó cần một cách lấy truy vấn
// RIÊNG ở kiểu chuyển đổi, và chỉ nó biết bảng ánh xạ đó.
const original = useTemplateRef<HTMLElement>('original')
useSelectionSurface(original, 'source')
</script>

<template>
  <PanelFrame owner="panel.source" status-key="panel.source.status" :show-status="showFrameStatus">
    <!-- 🔴 Lỗi đọc Chương nói ra bằng chuỗi CỦA NÓ, không im — xem `chapterErrorKey`. -->
    <p v-if="chapterErrorKey !== null" class="load-error">{{ t(chapterErrorKey) }}</p>
    <div v-else-if="hasChapter" class="source-body">
      <!-- 🔴 `tabindex` roving: đúng MỘT tab vào được vòng Tab, mũi tên đi giữa hai tab —
           hợp đồng `tablist` khai một nửa còn tệ hơn không khai, vì nó hứa một mô hình
           tương tác không tồn tại. Bắt ở lượt code review 2026-08-06. -->
      <div v-if="showHanVietTab" class="tabs" role="tablist">
        <button
          id="source-tab-original"
          type="button"
          class="tab"
          role="tab"
          aria-controls="source-tabpanel"
          :aria-selected="activeTab === 'original'"
          :tabindex="activeTab === 'original' ? 0 : -1"
          :class="{ active: activeTab === 'original' }"
          @click="dispatch('source.select_tab_original')"
          @keydown.right.prevent="dispatch('source.select_tab_han_viet')"
          @keydown.left.prevent="dispatch('source.select_tab_han_viet')"
        >{{ t('panel.source.tab_original') }}</button>
        <button
          id="source-tab-han-viet"
          type="button"
          class="tab"
          role="tab"
          aria-controls="source-tabpanel"
          :aria-selected="activeTab === 'han_viet'"
          :tabindex="activeTab === 'han_viet' ? 0 : -1"
          :class="{ active: activeTab === 'han_viet' }"
          @click="dispatch('source.select_tab_han_viet')"
          @keydown.right.prevent="dispatch('source.select_tab_original')"
          @keydown.left.prevent="dispatch('source.select_tab_original')"
        >{{ t('panel.source.tab_han_viet') }}</button>
        <!-- aura-allow-text: cả hai nhánh đi qua t(), không chuỗi viết thẳng nào —
             Kiểm A2 không đọc tĩnh được toán tử ba ngôi (cùng khuôn LibraryMode.vue:143). -->
        <button
          v-if="activeTab === 'han_viet' && (viewMode === 'parallel' || canUseParallelView)"
          type="button"
          class="view-toggle"
          @click="dispatch('source.toggle_han_viet_view')"
        >{{ viewMode === 'switch' ? t('panel.source.view_mode_parallel') : t('panel.source.view_mode_switch') }}</button>
      </div>

      <p v-if="isEmptyChapter" class="load-error">{{ t('panel.source.empty_chapter') }}</p>

      <!--
        🔴 STORY 1.18 · AC11 — `tabindex="0"` ĐÓNG `deferred-work.md:608` (bôi đen bằng BÀN
        PHÍM), và nó là một thay đổi **NHÌN THẤY ĐƯỢC** trên hợp đồng tiêu điểm.

        Sự thật kỹ thuật: một `<div>` không sửa được **không nhận** `Shift+Mũi tên`. Trình duyệt
        chỉ cho điều đó khi caret browsing bật (mặc định TẮT, không bật được bằng mã), hoặc
        phần tử `contenteditable` (bác — AD-1 nói nguyên văn là dữ liệu không sửa được, và một
        lỗ ở đó là **mất văn bản gốc** của người dùng), hoặc ứng dụng tự dựng `Range` —
        đường này. `tabindex="0"` là thứ cho `el.focus()` đặt được caret vào đây.

        🔴 **GIÁ PHẢI TRẢ, không giấu:** `PanelFrame` mang `tabindex="-1"` và không vào vòng `Tab`;
        thêm một `tabindex="0"` **bên trong** nó nghĩa là bấm `Tab` nay DỪNG ở thân panel.
        Đây là chỗ DUY NHẤT story 1.18 được chạm hợp đồng tiêu điểm (UX-DR8/UX-DR17, Story
        1.14 dặn không chạm). Ice chốt chấp nhận, 2026-08-07.

        ⚠️ Chú thích này đứng TRƯỚC dấu `aura-allow-text` dưới đây, không chen vào giữa nó và
        thẻ `div`: Kiểm A của `check-i18n.mjs` đòi dấu miễn trừ **ngay trên** khai báo, nên
        chen vào giữa là vô hiệu hoá nó (bắt lúc chạy cổng, 2026-08-07).
      -->
      <!-- aura-allow-text: NGUYÊN VĂN của Tác phẩm — nội dung người dùng đưa vào
           (`chapter.source_text`), không phải chuỗi giao diện của `vi.json`. NFR16 nói
           văn bản GIAO DIỆN sống ở vi.json; đây là DỮ LIỆU của người dùng, đúng thứ AD-16
           nói KHÔNG render thành HTML (không `v-html` — vẫn interpolation text thường). -->
      <div
        v-if="!isEmptyChapter && (!showHanVietTab || activeTab === 'original')"
        id="source-tabpanel"
        ref="original"
        role="tabpanel"
        tabindex="0"
        aria-labelledby="source-tab-original"
        class="original"
        :class="originalTokenClass"
      >{{ displayText }}</div>
      <template v-else-if="!isEmptyChapter">
        <p v-if="!canUseParallelView && viewMode === 'switch'" class="parallel-note">
          {{ t('panel.source.parallel_view_unavailable') }}
        </p>
        <SourceHanViet
          id="source-tabpanel"
          role="tabpanel"
          aria-labelledby="source-tab-han-viet"
          :source-text="sourceChapter!.source_text"
          :view-mode="viewMode"
        />
      </template>
    </div>
  </PanelFrame>
</template>

<style scoped>
.source-body {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

/* Quyết định #5 — dải tab ở ĐẦU THÂN panel, không `rightHeaderActionsComponent`. */
.tabs {
  display: flex;
  align-items: center;
  gap: var(--space-panel-inline);
  flex: none;
  margin-bottom: var(--space-panel-block);
}

.tab {
  appearance: none;
  border: none;
  background: none;
  padding: 0;
  cursor: pointer;
  font-family: var(--face-ui-md-strong);
  font-size: var(--font-ui-md-strong);
  font-weight: var(--weight-ui-md-strong);
  line-height: var(--leading-ui-md-strong);
  color: var(--color-on-surface-variant);
}

.tab.active {
  color: var(--color-primary);
}

/*
 * Lỗi đọc Chương, và ca Chương rỗng — hai thứ TRƯỚC ĐÂY im lặng.
 *
 * 🔴 Story 1.17 · Quyết định #7 (Ice chốt 2026-08-06) ĐÓNG lỗ hổng bảng token
 * `deferred-work.md:115`: token thứ 17 `ui-md-wrap` (giãn dòng 1.66, `wraps: true`) thay `ui-md`
 * (1.5, dưới sàn 1.66) — không chỉ token mới cho chuỗi mới của 1.17 mà để câu này ở lại
 * `ui-md` là món nợ `:115` vẫn KHÔNG đóng.
 */
.load-error {
  margin: 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

/*
 * Quyết định #7/Task 8 — Chương vượt trần render của kiểu song song.
 *
 * 🔴 Story 1.17 · Quyết định #7 — đổi từ `ui-sm` (1.5) sang token thứ 17
 * `ui-md-wrap` (1.66). ⚠️ Hệ quả NHÌN THẤY ĐƯỢC: cỡ chữ nhích lên một bậc — Ice chốt
 * dùng CHUNG một token cho cả ba chỗ (không thêm `ui-sm-wrap` riêng), xem `tokens.json`
 * `deviations.typography.ui-md-wrap`.
 */
.parallel-note {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.view-toggle {
  appearance: none;
  border: none;
  background: none;
  padding: 0;
  margin-left: auto;
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.original {
  flex: 1;
  min-height: 0;
  overflow: auto;
  white-space: pre-wrap;
}

/* AC1 — nguồn tiếng Trung, họ `read-cjk` (chỉ Noto Serif CJK TC). */
.tok-source-cjk {
  font-family: var(--face-source-cjk);
  font-size: var(--font-source-cjk);
  line-height: var(--leading-source-cjk);
  color: var(--color-on-surface);
}

/* AC2 — nguồn tiếng Anh, token thứ 16 (Quyết định #6). */
.tok-source-latin {
  font-family: var(--face-source-latin);
  font-size: var(--font-source-latin);
  line-height: var(--leading-source-latin);
  color: var(--color-on-surface);
}
</style>
