<script setup lang="ts">
// Shortcuts section of the Settings frame (FR22, NFR17, AD-34). Shortcut is `GlobalOnly`
// (`core/scope/kinds.rs`), so there is deliberately no scope switch: `shortcuts.scope_note`
// says so instead. State lives in `config/shortcutsState.ts`.
import { onBeforeUnmount, onMounted } from 'vue'
import { t } from './i18n'
import { dispatch } from './commands'
import {
  aimRowFrom,
  aimedShortcutRow,
  cancelCapture,
  captureIsArmed,
  defaultDisplayFor,
  diskBindingsRejected,
  enterShortcutsScreen,
  handleCaptureKey,
  leaveShortcutsScreen,
  shortcutNotice,
  shortcutRows,
  unboundShortcutIds,
} from './config/shortcutsState'

onMounted(enterShortcutsScreen)
onBeforeUnmount(leaveShortcutsScreen)

/**
 * 🔴 Tiêu điểm rời ô phím trong lúc **đang bắt** ⇒ huỷ lượt bắt.
 *
 * Không có vế này thì `capturing` treo `true` sau một cú bấm trượt hàng *(một `<th>`, một
 * khoảng trắng ngoài bảng)* hay một lượt Tab đi: câu *"đang chờ một tổ hợp phím"* vẫn hiện,
 * `@keydown` của ô phím không còn nhận gì, và cửa `isBlocked` của `main.ts` vẫn nuốt mọi
 * command toàn cục. Bản bất biến giữ ở đây: **đang bắt ⇔ ô phím đang giữ tiêu điểm** — tức
 * đúng khi và chỉ khi handler tới được. Vế còn lại của nó là cú ép tiêu điểm lúc arming ở
 * `config/shortcutsState.ts::captureShortcut`; một vế không có vế kia thì vô nghĩa.
 */
function onKeyCellFocusOut(): void {
  if (captureIsArmed.value) cancelCapture()
}

/**
 * Bàn phím trên **ô phím** của một hàng. `@keydown` nằm NGOÀI luật Kiểm A (`check-commands.mjs`
 * nói nguyên văn *"Chỉ `@click`"*), nên nó được xử lý tự do — và nó phải được, vì đây là chỗ
 * duy nhất trong ứng dụng đọc một hợp âm **thô**.
 *
 * ⚠️ Ô phím là một `<button>`, **không** một `<input>`: `isTypingZone` của `keys.ts` đọc
 * `tagName === 'INPUT' | 'TEXTAREA' | 'SELECT'`, và một `<input>` làm luật vùng gõ đổi hành
 * vi ngay giữa lúc ta đang cố bắt một phím trần (Bẫy 6).
 */
function onKeyCellKeydown(event: KeyboardEvent): void {
  // `.value`: a bare `Ref` is always truthy in `<script>`, which would kill the `Backspace` branch below.
  if (captureIsArmed.value) {
    if (event.code === 'Escape') {
      // Stops here so the Settings overlay's `@keydown.esc` does not close it while cancelling a capture.
      event.preventDefault()
      event.stopPropagation()
      cancelCapture()
      return
    }
    if (handleCaptureKey(event)) {
      event.preventDefault()
      event.stopPropagation()
    }
    return
  }

  // 🔴 BẪY 5 — `⌫` bỏ gán **chỉ ở trạng thái nghỉ**. `Backspace` có trong `NAMED_CODES`, tức
  // `Backspace` trần và `Mod+Backspace` đều là hợp âm hợp lệ gán được; đọc nó thành "bỏ gán"
  // trong lúc đang bắt là khoá vĩnh viễn một phím khỏi bảng, không dấu hiệu nào.
  //
  // ⚠️ `!event.repeat` là vế thứ hai của cùng cái bẫy, và nó chỉ tới được sau khi lỗi `.value`
  // ở trên được vá. Gán `⌫` rồi **giữ phím thêm một nhịp**: keydown đầu tiêu thụ ở nhánh đang
  // bắt và tắt `capturing`, rồi nhịp auto-repeat của **cùng một cú bấm** rơi xuống đây và bỏ
  // gán đúng cái phím vừa gán. `preventDefault()` không chặn auto-repeat.
  if (event.code === 'Backspace' && !event.repeat) {
    event.preventDefault()
    dispatch('shortcuts.unassign')
  }
}
</script>

<template>
  <div class="sc-body">
      <p class="sc-note">{{ t('shortcuts.scope_note') }}</p>

      <!--
        🔴 AC13 — hợp âm trên đĩa bị từ chối phải NÓI RA. Cho tới story này chẩn đoán chỉ đi
        ra `console.error`, tức im lặng theo nghĩa thực dụng. Đóng `deferred-work.md §*Deferred from: 1-7-tang-ghi-du-lieu-mot-writer-noi-tiep-va-luoc-do-co-phien-ban (2026-08-04)*`.
      -->
      <p v-if="diskBindingsRejected" class="sc-alert">{{ t('shortcuts.disk_rejected') }}</p>

      <!-- Câu gần nhất: đang chờ hợp âm · xung đột · phím lạ · lưu trượt · chưa nhắm hàng. -->
      <p v-if="shortcutNotice !== null" class="sc-alert">
        {{ t(shortcutNotice.key, shortcutNotice.params) }}
      </p>

      <p class="sc-gesture">{{ t('shortcuts.gesture') }}</p>

      <!--
        🔴 `@mousedown` uỷ quyền ở VÙNG CHỨA, không một handler trên mỗi hàng: một bảng 30+
        hàng là 30+ chỗ để sai, và `@click` của mỗi nút phải ở lại đúng một `dispatch`.
        Vì sao cần nó cùng với `document.activeElement`: WKWebView **không** đặt tiêu điểm
        cho `<button>` khi bấm chuột — xem `config/shortcutsState.ts`.

        🔴 `@focusin` CÙNG với `@mousedown`, và cả hai đều cần (bắt ở code review 2026-08-11).
        `mousedown` một mình để `aimedRow` sống dai hơn ý định: `targetRow()` cho nó thắng tiêu
        điểm DOM, và nó chỉ bị xoá lúc đóng lớp phủ — nên sau lượt gán đầu tiên, một người
        dùng THUẦN BÀN PHÍM Tab sang hàng khác rồi bấm "Bỏ gán" sẽ xoá phím của hàng CŨ.
        `focusin` giữ `aimedRow` đi theo tiêu điểm, mà không lật thứ tự chuột-trước: khi
        WKWebView không đặt tiêu điểm cho `<button>` thì `focusin` không nổ, và giá trị của
        `mousedown` ở lại đúng như §Dev Notes ⑨ đòi.
      -->
      <table class="sc-table" @mousedown="aimRowFrom($event)" @focusin="aimRowFrom($event)">
        <thead>
          <tr>
            <th>{{ t('shortcuts.col_command') }}</th>
            <th>{{ t('shortcuts.col_id') }}</th>
            <th>{{ t('shortcuts.col_key') }}</th>
            <!--
              Cột này chứa HAI NÚT cộng hợp âm mặc định, không một lời ghi chú. Nhãn cũ
              (`col_note` = "Ghi chú") nói sai nội dung cột — bắt ở code review 2026-08-11.
            -->
            <th>{{ t('shortcuts.col_actions') }}</th>
          </tr>
        </thead>
        <tbody>
          <!--
            🔴 AC1 — **MỌI** command của `commandRegistry.list()`, không một phép lọc nào.
            🔴 AC12 — hợp âm đến từ `effectiveBindings()`, không từ `spec.keys`: spec bị đóng
            băng lúc `register()` nên nó trả lời *thời điểm cài đặt*, và sau lượt gán đầu tiên
            nó CŨ. Doc-comment đầy đủ ở `src/commands/registry.ts`.
          -->
          <tr
            v-for="row in shortcutRows"
            :key="row.id"
            :data-command-id="row.id"
            :class="{ 'sc-aimed': aimedShortcutRow === row.id }"
            :aria-current="aimedShortcutRow === row.id ? 'true' : undefined"
          >
            <td class="sc-label">
              {{ t(row.labelKey) }}
              <!--
                🔴 UX-DR27 — hàng đang nhắm nói ra bằng **CHỮ**, không bằng màu một mình. Ba
                nút thao tác đều áp vào hàng này, nên "hàng nào đang nhắm" là một trạng thái
                người dùng **phải** biết. Cùng khuôn `attribution.state_off` của Story 1.19.
              -->
              <span v-if="aimedShortcutRow === row.id" class="sc-aimed-tag">{{
                t('shortcuts.state_aimed')
              }}</span>
            </td>
            <!--
              aura-allow-text: mã lệnh (`lookup.toggle_pin`) — DỮ LIỆU, một định danh máy chứ
              không một câu giao diện. Cùng hạng `src.display_name` ở `AttributionOverlay.vue`.
            -->
            <td class="sc-id">{{ row.id }}</td>
            <td class="sc-keycell">
              <!--
                Ô phím là một `<button>` (Bẫy 6). `@click` đi qua command, nên Enter/Space
                của chuẩn HTML gốc cũng vào được trạng thái bắt — đó là vế bàn phím của AC6.
              -->
              <!--
                `data-key-cell` là cách `captureShortcut` tìm ra ô phím để ÉP tiêu điểm vào
                nó lúc arming — một thuộc tính `data-`, không lớp CSS, vì `sc-key` thuộc khối
                `style` scoped tức một chi tiết trình bày. Cùng khuôn `data-shortcuts-open`.
              -->
              <button
                type="button"
                class="sc-key"
                data-key-cell
                :class="{ 'sc-key-armed': captureIsArmed && aimedShortcutRow === row.id }"
                @click="dispatch('shortcuts.capture')"
                @keydown="onKeyCellKeydown($event)"
                @focusout="onKeyCellFocusOut()"
              >
                <!--
                  aura-allow-text: hợp âm đã định dạng (`⌘D`, `Ctrl+Alt+→`) — DỮ LIỆU dẫn xuất
                  từ bảng phím, không một câu giao diện.
                -->
                <span v-if="row.display.length > 0">{{ row.display.join(' · ') }}</span>
                <span v-else class="sc-unassigned">{{ t('shortcuts.unassigned') }}</span>
              </button>
            </td>
            <td class="sc-actions">
              <!--
                🔴 AC8 — HAI nút, và chúng là hai trạng thái khác nhau. Một màn hình chỉ có
                "bỏ gán" là một cửa MỘT CHIỀU: gỡ phím của `mode.library` rồi không còn đường
                nào lấy lại `⌘1`.
                ⚠️ Mỗi `@click` là đúng MỘT `dispatch('<id>')` với id **literal** (Kiểm A) —
                hàng đang nhắm đi bằng `@mousedown` ở vùng chứa, không bằng một tham số.
              -->
              <button type="button" class="sc-act" @click="dispatch('shortcuts.unassign')">
                {{ t('shortcuts.act_unassign') }}
              </button>
              <button
                v-if="row.overridden"
                type="button"
                class="sc-act"
                @click="dispatch('shortcuts.reset')"
              >
                {{ t('shortcuts.act_reset') }}
              </button>
              <!--
                Mặc định của sản phẩm, hiện ra khi người dùng đã đè lên nó — người ta cần biết
                mình đang trả về CÁI GÌ trước khi bấm.
                aura-allow-text: hợp âm đã định dạng — DỮ LIỆU.
              -->
              <span v-if="row.overridden && row.hasDefault" class="sc-default">{{
                defaultDisplayFor(row.id).join(' · ')
              }}</span>
            </td>
          </tr>
        </tbody>
      </table>

      <!--
        🔴 AC5 — nhóm *"chưa gán phím nào"* đọc từ `effectiveUnbound()` lúc chạy. Đây là câu
        trả lời mà AD-34 §1 tồn tại để làm cho **liệt kê được bằng máy** thay vì kiểm bằng mắt.
      -->
      <p class="sc-unbound-line">
        {{ t('shortcuts.unbound_count', { so: String(unboundShortcutIds.length) }) }}
      </p>
  </div>
</template>

<style scoped>
.sc-gesture,
.sc-unbound-line {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

/* Câu "một tầng" là một mệnh đề kiến trúc, không một chú thích — nó mang một nét dẫn. */
.sc-note {
  margin: 0 0 var(--space-panel-block) 0;
  padding-left: 12px;
  border-left: 2px solid var(--color-tm-rule);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-tm-text);
}

/*
 * Câu trạng thái — xung đột, phím lạ, lưu trượt. UX-DR27: nói ra bằng CHỮ, không bằng màu
 * một mình; màu ở đây chỉ là lớp thứ hai.
 */
.sc-alert {
  margin: 0 0 var(--space-panel-block) 0;
  padding-left: 12px;
  border-left: 2px solid var(--color-error);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-error);
}

.sc-table {
  width: 100%;
  margin-bottom: var(--space-panel-block);
  border-collapse: collapse;
  text-align: left;
}

.sc-table th {
  padding: 0 8px var(--space-panel-block) 0;
  border-bottom: 1px solid var(--color-outline);
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.sc-table td {
  padding: var(--space-panel-block) 8px var(--space-panel-block) 0;
  border-bottom: 1px solid var(--color-outline);
  vertical-align: top;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface);
}

/*
 * Hàng đang nhắm — một nét dẫn, không một nền màu (tương phản).
 *
 * ⚠️ Nét trong suốt khai ở **mọi** hàng, không chỉ hàng đang nhắm: thêm một đường viền vào
 * đúng lúc chọn sẽ đẩy cả bảng sang phải 2px mỗi lần người dùng đổi hàng.
 * ⚠️ `border`, KHÔNG `box-shadow` — Kiểm F của `check:tokens` cấm bóng đổ và lớp nổi (AC7).
 * Trạng thái này còn nói bằng CHỮ ở `.sc-aimed-tag`; màu chỉ là lớp thứ hai (UX-DR27).
 */
.sc-table td:first-child {
  border-left: 2px solid transparent;
  padding-left: 8px;
}

.sc-aimed td:first-child {
  border-left-color: var(--color-primary);
}

.sc-label {
  color: var(--color-on-surface);
}

.sc-id,
.sc-default {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface-variant);
}

.sc-keycell {
  white-space: nowrap;
}

.sc-key {
  padding: 4px 8px;
  border: 1px solid var(--color-outline);
  background: none;
  cursor: pointer;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
}

/* Đang chờ một hợp âm — trạng thái ② của Quyết định #6. */
.sc-key-armed {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.sc-unassigned {
  color: var(--color-on-surface-variant);
}

.sc-aimed-tag {
  display: inline-block;
  margin-left: 8px;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-primary);
}

.sc-actions {
  white-space: nowrap;
}

.sc-act {
  margin-right: 8px;
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

.sc-default {
  margin-left: 4px;
}
</style>
