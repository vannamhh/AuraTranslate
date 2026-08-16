<script setup lang="ts">
// Bề mặt **lịch sử phiên bản segment** — Story 2.6 · FR101 · AC1 · AC2 · AC3 · AC5.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT LỚP PHỦ CẤP `App.vue` — Quyết định #3 đường (a), Ice ký 2026-08-16
// ─────────────────────────────────────────────────────────────────────────────
// Ba đường đã cân, và hai đường kia bị loại bằng phép đo chứ không bằng sở thích:
// - **tab thứ hai trong panel Lookup** — cột thật của bố cục Ⓑ-2 rộng **238,5 px**
//   (`deferred-work.md:3131-3162`), còn mockup vẽ một cột danh sách 270 px **cộng** một cột
//   nội dung. Và panel Lookup sẽ mang một khái niệm không thuộc tra cứu.
// - **mở ra trong chính hàng của lưới** — 🔴 **một hàng KHÔNG phải một phần tử DOM**
//   (`GridPanel.vue:5-24`): năm cột là năm `subgrid` chia chung một tập track, nên chèn một
//   khối vào giữa một track là thứ hình dạng đó **không diễn đạt được**.
//
// ⇒ Khuôn `ShortcutsOverlay.vue`/`AttributionOverlay.vue`: con trực tiếp của `App.vue`,
// `role="dialog"`, bẫy `Tab` tự viết, `Escape` **cục bộ**, nút đóng đi qua `dispatch`.
//
// ⚠️ **Không** một `z-index` thứ ba: cả hai lớp phủ đang sống dùng **cùng giá trị `10`** và
// cùng câu miễn trừ có tên `aura-allow-z-index`. Đo 2026-08-16 chứ không đoán.
// ⚠️ **Không** thêm mục vào `FOCUS_OWNERS` — hai lớp phủ đang sống đều không có mục nào ở đó
// (`commands/index.ts`), và đây là tiền lệ đã đo, không một lượt quên.
import { computed, nextTick, useTemplateRef, watch } from 'vue'

import { dispatch } from './commands'
import { focusReturnTargetOnOpen } from './commands/focus'
import { t } from './i18n'
import { historyTimeLabel } from './panels/segmentHistoryTime'
import { useSelectionSurface } from './panels/selectionContract'
import {
  aimHistoryVersion,
  closeSegmentHistory,
  historyAimedVersionId,
  historyIsOpen,
  historyLoadError,
  historyPending,
  historyPendingRestore,
  historyRestoreError,
  historyRestoreNotice,
  historySegmentId,
  historyVersions,
} from './panels/segmentHistoryState'

const panel = useTemplateRef<HTMLElement>('panel')

// 🔴 Vai `'display'`, **không** `'source'` — cùng lý do Bẫy 1 của Story 1.18 và cùng chữ ký
// mà `AttributionOverlay.vue` đã mang: bôi đen một bản dịch cũ để đọc kỹ mà phát ra một lượt
// tra cứu là thay chính đoạn đang đọc dưới tay người đọc. Và AC23 của Story 2.3 đã chốt bằng
// chữ: **Editor không phát lượt tra từ điển**.
useSelectionSurface(panel, 'display')

/**
 * 🔴 UX-DR17 — trả tiêu điểm về chỗ cũ. Xem `AttributionOverlay.vue` cho lý do đầy đủ:
 * `document.activeElement` trần **không** dùng được, vì trên WKWebView tiêu điểm rơi lên
 * `section.panel[tabindex="-1"]` ngay trong cùng một tick.
 */
let returnFocusTo: HTMLElement | null = null

watch(historyIsOpen, (open) => {
  if (open) {
    returnFocusTo = focusReturnTargetOnOpen('[data-history-open]')
    // Sau `nextTick`: `v-if` mới dựng node ở lượt render này.
    void nextTick(() => panel.value?.focus())
    return
  }

  const back = returnFocusTo
  returnFocusTo = null

  // 🔴 `isConnected` chứ không `back?.focus()` trần: một node đã rời DOM vẫn nhận được lời
  // gọi `focus()` mà **không** ném và **không** có tác dụng — tiêu điểm rơi về `body`, đúng
  // thứ UX-DR17 cấm, và không một dấu hiệu nào báo.
  if (back !== null && back.isConnected) {
    back.focus()
    return
  }

  const opener = document.querySelector<HTMLElement>('[data-history-open]')
  if (opener !== null) {
    opener.focus()
    return
  }
  // ⚠️ Chẩn đoán viết bằng **tiếng Anh** — Kiểm A của `check:i18n` cấm chữ tiếng Việt có dấu
  // ở vị trí mã trong `.vue`. Chuỗi HIỂN THỊ đi qua `t()`; đây là một dòng cho console.
  console.warn('[history] focus-return target is gone; focus falls back to body.')
})

/** Xem `AttributionOverlay.vue` — cùng hàm, cùng lý do cho hai bộ lọc `:not(...)`. */
function focusableWithin(root: HTMLElement): HTMLElement[] {
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), ' +
        'textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  )
}

/**
 * 🔴 **BẪY TIÊU ĐIỂM.** `preventDefault()` **luôn luôn** rồi tự lái, chứ không chỉ chặn ở hai
 * đầu danh sách: tự lái đúng ở **mọi** vị trí xuất phát, kể cả khi tiêu điểm đang ở chính
 * `.hist-panel` *(vị trí ngay sau lượt mở)* hay đã lạc ra ngoài vì một lượt dựng lại DOM.
 * Phiên bản *"chỉ chặn ở hai đầu"* đọc gọn hơn và để lọt đúng những ca đó.
 */
function trapTab(event: KeyboardEvent): void {
  const root = panel.value
  if (root === null) return

  event.preventDefault()
  const stops = focusableWithin(root)
  if (stops.length === 0) {
    // Giữ tiêu điểm trên chính khối hộp thoại, vì `Escape` vẫn phải tới được handler.
    root.focus()
    return
  }

  const active = document.activeElement
  const index = active instanceof HTMLElement ? stops.indexOf(active) : -1
  const step = event.shiftKey ? -1 : 1
  const next = index === -1 ? (event.shiftKey ? stops.length - 1 : 0) : index + step
  stops[(next + stops.length) % stops.length].focus()
}

/**
 * Mốc "bây giờ" cho phép định dạng tương đối.
 *
 * ⚠️ Đọc **một lần mỗi lượt render**, không một `setInterval`. Khác `StatusBar.vue` — thanh
 * đó đếm giây trong lúc người dùng nhìn, còn danh sách này đứng yên: một mốc *"12 phút
 * trước"* không cần thành *"13 phút trước"* trong lúc lớp phủ đang mở. Một bộ đếm ở đây là
 * một lượt render mỗi giây cho một thứ không ai chờ.
 *
 * 🔴 `historyTimeLabel` **không** tự đọc đồng hồ — mốc đi vào qua tham số, và đó là thứ làm
 * phép kiểm của nó **tất định**. Chỗ đọc đồng hồ là đây, một chỗ duy nhất.
 */
const nowMs = computed(() => {
  // Phu thuoc vao danh sach de moc duoc doc lai moi luot nap lich su.
  void historyVersions.value
  return Date.now()
})

// 🔵 2026-08-16 (code review): `currentText` ĐÃ GỠ cùng `hist-current`. Nó là chỗ đọc duy nhất
// của phép so nội dung mà Quyết định #5 gọi là không an toàn; giữ lại một `computed` không ai
// đọc là đúng thứ lượt tự rà Ⓠ của story này vừa gỡ ở `reloadHistory()` — mã không ai dùng, và
// `check:lint` KHÔNG bắt được nó. Vế "hàng nào đang dùng" ghi nợ, chủ Story 2.7.

function labelFor(createdAt: string): string {
  const { key, params } = historyTimeLabel(createdAt, nowMs.value)
  return t(key, params)
}

/**
 * Nhắm một hàng phiên bản — gọi từ `@mousedown` và `@focusin`.
 *
 * 🔴 Cả hai sự kiện, và cả hai đều cần (cùng lý do đã đo ở bảng phím của Story 1.21):
 * `mousedown` một mình để hàng đang nhắm sống dai hơn ý định; `focusin` một mình thì một cú
 * bấm chuột lên vùng trống của hàng không nhắm được gì.
 *
 * ⚠️ Kiểm A của `check:commands` nói nguyên văn *"chỉ `@click`"*, nên hai sự kiện này được xử
 * lý tự do — và chúng **phải** được, vì đây là chỗ duy nhất mang một tham số theo hàng.
 */
function aimRow(id: number): void {
  aimHistoryVersion(id)
}
</script>

<template>
  <!--
    🔴 `Escape` là một lượt **huỷ trong ngữ cảnh**, không một thao tác toàn cục — `@keydown.esc`
    chứ không một hợp âm trong `CommandRegistry`. Đăng ký `Escape` toàn cục là chiếm phím đó
    cho **cả** ứng dụng, và Story 1.21 sẽ hiện nó ra như một phím gán lại được trong khi nó chỉ
    có nghĩa khi lớp phủ đang mở. Cùng lý do đã ghi trong hai lớp phủ đang sống.

    Nút đóng thì ĐI QUA command — Kiểm A của `check:commands` đòi mọi `@click` là đúng một
    `dispatch('<id>')`.
  -->
  <div
    v-if="historyIsOpen"
    class="hist-scrim"
    @keydown.esc="closeSegmentHistory()"
    @keydown.tab="trapTab($event)"
  >
    <section ref="panel" class="hist-panel" tabindex="-1" role="dialog" aria-modal="true">
      <header class="hist-head">
        <h2 class="hist-title">{{ t('history.title') }}</h2>
        <button type="button" class="hist-close" @click="dispatch('history.close')">
          {{ t('history.close') }}
        </button>
      </header>

      <!--
        🔴 BA TRẠNG THÁI RỖNG KHÁC NHAU, BA CÂU KHÁC NHAU — và thứ tự các nhánh là cả điểm.

        Gộp chúng lại là đúng §*"Rỗng IM LẶNG bị cấm; rỗng CÓ LÝ DO thì không"*: ba nguyên nhân
        khác nhau dẫn tới ba việc khác nhau cho người dùng.
        ① chưa nhắm được câu nào ⇒ đặt con trỏ vào một câu;
        ② đọc hỏng ⇒ không phải "chưa có phiên bản", và bản dịch **không bị đụng**;
        ③ chưa từng xác nhận ⇒ đây là AC3, và câu này phải **nói ra cơ chế**.
      -->
      <p v-if="historySegmentId === null" class="hist-empty">{{ t('history.no_segment') }}</p>
      <p v-else-if="historyLoadError !== null" class="hist-empty">
        {{ t('history.load_failed') }}
      </p>
      <p v-else-if="!historyPending && historyVersions.length === 0" class="hist-empty">
        {{ t('history.empty') }}
      </p>

      <!--
        🔴 CÂU HỎI CỦA CHỮ KÝ #2(a) — và nó sống NGAY TRONG lớp phủ này.

        Không `window.confirm()` *(một dialog CHẶN của trình duyệt: chuỗi nằm ngoài `vi.json`
        và ngoài Kiểm A/D, không token hoá được, và `check:layout` Kiểm C là một danh sách CHO
        PHÉP cho mọi thành viên của `window` mà `src/**` chạm tới)*. Không một lớp phủ thứ hai
        chồng lên lớp phủ này — đó mới là một tầng `z-index` thứ ba thật sự.

        ⚠️ Bản nháp được **hiện ra**, không chỉ được nhắc tới: người dùng phải nhìn thấy thứ
        họ sắp mất mới quyết định được.
      -->
      <div v-if="historyPendingRestore !== null" class="hist-confirm">
        <p class="hist-confirm-title">{{ t('history.confirm_title') }}</p>
        <p class="hist-confirm-body">{{ t('history.confirm_body') }}</p>
        <!-- aura-allow-text: DỮ LIỆU NGƯỜI DÙNG — chính bản dịch họ vừa gõ, không một chuỗi giao diện. Nó phải hiện NGUYÊN VĂN; đẩy nó qua `t()` là vô nghĩa và sẽ hỏng ở ký tự `{`. -->
        <p class="hist-draft">{{ historyPendingRestore.draft }}</p>
        <div class="hist-confirm-acts">
          <button type="button" class="hist-act" @click="dispatch('history.confirm_restore')">
            {{ t('history.confirm_accept') }}
          </button>
          <button type="button" class="hist-act" @click="dispatch('history.cancel_restore')">
            {{ t('history.confirm_cancel') }}
          </button>
        </div>
      </div>

      <p v-if="historyRestoreError !== null" class="hist-empty">
        {{ t('history.restore_failed') }}
      </p>
      <p v-else-if="historyRestoreNotice === 'restored'" class="hist-notice">
        {{ t('history.restored') }}
      </p>
      <p v-else-if="historyRestoreNotice === 'unchanged'" class="hist-notice">
        {{ t('history.unchanged') }}
      </p>
      <!--
        🔵 2026-08-16 (code review) — HAI KẾT QUẢ TỪNG KHÔNG CÓ ĐƯỜNG RA NÀO.

        `'flush-failed'` và `'still-dirty'` `return` trước khi đặt một ô nào, và nơi gọi vứt giá
        trị trả về ⇒ người dùng bấm khôi phục và **không một pixel nào đổi**. Đúng lớp *"rỗng im
        lặng"* mà code review 2026-08-15 đã bắt cho `confirmSegment`.

        ⚠️ Cả hai nói cùng một điều quan trọng nhất: **bản dịch không bị đụng tới**. Chúng tách
        nhau vì việc người dùng phải làm khác nhau — một cái là thử lại, một cái là chờ một nhịp.
      -->
      <p v-else-if="historyRestoreNotice === 'flush-failed'" class="hist-notice">
        {{ t('history.flush_failed') }}
      </p>
      <p v-else-if="historyRestoreNotice === 'still-dirty'" class="hist-notice">
        {{ t('history.still_dirty') }}
      </p>

      <!--
        🔴 MỖI PHIÊN BẢN HIỆN **TOÀN VĂN**, KHÔNG DIFF — Quyết định #4 đường (a).

        `Cargo.toml` ghi sẵn cả `similar` 3.1.1 lẫn `dissimilar` 1.0.11 và **cố ý không cài
        cái nào**; chốt ở **Story 8.1** sau khi thử cả hai trên bản review thật. Và **không AC
        nào của story này đòi diff**. Vế so sánh ghi nợ có chủ.

        🔴 VÀ KHÔNG NHÃN NÀO — Quyết định #5 đường (a). Bốn nhãn của mockup (`đang dùng` ·
        `từ bản review` · `từ AI` · `từ TM`) trỏ vào bốn năng lực chưa dựng; bảng
        `segment_version` có đúng bốn cột và `schema.rs` khai bằng chữ rằng cột xuất xứ thuộc
        Story 2.7, cột cặp TM thuộc Epic 7.

        🔵 **2026-08-16 (code review) — `hist-current` ĐÃ GỠ, Ice chốt.** Bản trước gắn một
        viền trái `--color-primary` cho hàng có `target_text === currentText` và tự biện minh
        rằng *"nó cố ý không mang chữ"*. Nhưng đó **chính là** phép so mà bảng của Quyết định
        #5 gọi tên là *"suy được, nhưng KHÔNG AN TOÀN — hai phiên bản trùng văn bản thì nhãn
        khớp NHIỀU hàng"*, và chữ ký (a) là *"không nhãn nào"*. Lập luận *chữ thì cấm, màu thì
        không* không đứng được: chuỗi **ký → sửa → hoàn tác về bản cũ** sinh hai hàng trùng văn
        bản, và cả hai cùng sáng viền ⇒ chỉ dấu **nói dối** ở đúng ca nó được dựng để phục vụ.
        ⇒ Vế *"hàng nào đang dùng"* ghi nợ cùng bốn nhãn kia, **chủ Story 2.7** — ở đó cột xuất
        xứ (FR117) làm nó trả lời được theo `id` chứ không theo nội dung, tức đường (b).
      -->
      <ol v-if="historyVersions.length > 0" class="hist-list">
        <li
          v-for="version in historyVersions"
          :key="version.id"
          class="hist-item"
          :class="{ 'hist-aimed': historyAimedVersionId === version.id }"
          @mousedown="aimRow(version.id)"
          @focusin="aimRow(version.id)"
        >
          <div class="hist-row">
            <!-- aura-allow-text: KẾT QUẢ của `t()` — `labelFor` chỉ chọn nhánh rồi gọi `t(key, params)`. Bốn khoá `history.time_*` nằm trong `vi.json` và Kiểm E chạy trên chúng. -->
            <span class="hist-when">{{ labelFor(version.created_at) }}</span>
            <!--
              🔵 2026-08-16 (code review) — KHOÁ CẢ KHI ĐANG CÓ MỘT CÂU HỎI CHỜ TRẢ LỜI.

              `historyPending` một mình **không đủ**: cờ đó về `false` trong khối `finally`
              **trước** khi câu hỏi hiện ra. ⇒ Người dùng bấm sang hàng khác trong lúc hộp thoại
              đang mở, một lượt mới ghi đè `pendingRestore`, và câu hỏi cũ **biến mất không dấu
              vết** — không ai từng trả lời nó. Một chốt an toàn đi vòng qua được bằng đúng thao
              tác mà bề mặt vẫn mời làm.
            -->
            <button
              type="button"
              class="hist-act"
              :disabled="historyPending || historyPendingRestore !== null"
              @click="dispatch('history.restore')"
            >
              {{ t('history.act_restore') }}
            </button>
          </div>
          <!-- aura-allow-text: DỮ LIỆU NGƯỜI DÙNG — một bản dịch cũ, hiện nguyên văn. Cùng lý do với `hist-draft` ở trên. -->
          <p class="hist-text">{{ version.target_text }}</p>
        </li>
      </ol>
    </section>
  </div>
</template>

<style scoped>
/*
 * ⚠️ Màu VÀ cỡ chữ **chỉ** đến từ token (Kiểm B + B2 của `check:tokens`). Không bóng đổ,
 * không gradient (Kiểm F). Không `opacity` trung gian (Kiểm D).
 */
.hist-scrim {
  position: fixed;
  inset: 0;
  /* aura-allow-z-index: xếp lớp CƠ HỌC — dockview dựng ngữ cảnh xếp lớp riêng cho mỗi nhóm panel, nên thứ tự tài liệu một mình không đủ để lớp phủ nằm TRÊN lưới. */
  z-index: 10;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding: var(--space-panel-inline);
  background: var(--color-background);
}

.hist-panel {
  width: 100%;
  max-width: 1100px;
  max-height: 100%;
  overflow: auto;
  padding: var(--space-panel-inline);
  border: 1px solid var(--color-outline);
  background: var(--color-surface);
}

.hist-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  margin-bottom: var(--space-panel-block);
}

.hist-title {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  line-height: var(--leading-read-title);
  color: var(--color-on-surface);
}

.hist-close,
.hist-act {
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.hist-empty,
.hist-notice {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface-variant);
}

.hist-confirm {
  margin-bottom: var(--space-panel-block);
  padding: var(--space-panel-block);
  border: 1px solid var(--color-outline);
}

.hist-confirm-title {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.hist-confirm-body {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.hist-draft {
  margin: 0 0 var(--space-panel-block) 0;
  padding: var(--space-panel-block);
  border-left: 2px solid var(--color-draft);
  font-family: var(--face-read-body);
  font-size: var(--font-read-body);
  line-height: var(--leading-read-body);
  color: var(--color-on-surface);
}

.hist-confirm-acts {
  display: flex;
  gap: var(--space-panel-inline);
}

.hist-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.hist-item {
  padding: var(--space-panel-block) 0;
  border-top: 1px solid var(--color-outline);
}

/*
 * Hàng **đang nhắm** — thứ ba command không-tham-số tác động lên. Khuôn `sc-aimed` của bảng
 * phím tắt. ⚠️ Nền chứ không viền: một đường kẻ lề ở đây từng mang nghĩa khác (`hist-current`,
 * 🔵 gỡ 2026-08-16 — xem chú thích trong template), và giữ nền thì lượt sau thêm lại một nghĩa
 * thứ hai cho viền trái sẽ không âm thầm đâm vào cái này.
 */
.hist-aimed {
  background: var(--color-surface-variant);
}

.hist-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
}

.hist-when {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.hist-text {
  margin: var(--space-panel-block) 0 0 0;
  /*
   * ⚠️ `pre-line` — cùng lý do và cùng chữ ký với ô bản dịch của lưới (Quyết định #2 đường
   * (b) của Story 2.5d): `target_text` **mang ký tự `\n` thật** từ khi AD-46 mở đường ngắt
   * đoạn, và dưới `white-space: normal` mặc định chúng bị gộp thành một khoảng trắng. Một
   * bản dịch hai đoạn sẽ hiện thành một khối liền ở đây, tức lịch sử nói sai về chính thứ
   * nó đang cho xem.
   */
  white-space: pre-line;
  font-family: var(--face-read-body);
  font-size: var(--font-read-body);
  line-height: var(--leading-read-body);
  color: var(--color-on-surface);
}
</style>
