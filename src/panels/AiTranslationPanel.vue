<script setup lang="ts">
// Panel `Đề xuất AI`. Story 1.14 · AC1 · AC8 — **khung**, không phải nội dung.
//
// Bản dịch AI thật, chọn nhà cung cấp, và ba điểm ra mạng của AD-15 là **Epic 4**.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 "CHƯA CẤU HÌNH" KHÔNG PHẢI MỘT TRẠNG THÁI LỖI — UX-DR27 · FR77
// ─────────────────────────────────────────────────────────────────────────────────
// Panel này **MỜI CẤU HÌNH**. Không cảnh báo, không màu `error`, không dấu chấm
// than. Một người dùng chưa từng dán khoá API vào đâu thì không làm sai gì cả — vẽ
// một cảnh báo ở đây là dạy họ rằng ứng dụng đang hỏng.
//
// ⚠️ Câu trạng thái sống ở `vi.json` (`panel.ai_translation.status`) và Kiểm D của
// `check-i18n.mjs` chấm phần máy chấm được của UX-DR47 (không "chúng tôi", không
// "bạn"). Phần còn lại — giọng MỜI thay vì giọng CẢNH BÁO — là chỗ con người phải đọc.
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 1.18 · AC2 — ĐĂNG KÝ HỢP ĐỒNG VÙNG CHỌN, KHÔNG NỘI DUNG
// ─────────────────────────────────────────────────────────────────────────────────
// Panel này hôm nay **không có chữ**, và đó chính là lý do lượt đăng ký phải nằm ở đây NGAY
// BÂY GIỜ: `epics.md:1762` đòi AI Translation *"nhận được cùng hành vi khi nó có nội dung
// ở các epic sau, **không cần cài lại**"*. Một lượt đăng ký thiếu ở đây không để lại **bất
// kỳ triệu chứng nào** cho tới Epic 4 — tức hai epic sau, và tới lúc đó không ai nhớ AC này
// tồn tại. Cổng đếm của `check-commands.mjs` (Kiểm F) là thứ giữ mệnh đề đó bằng MÁY.
//
// Đừng "dọn" `<div ref="surface">` vì nó trông trống: nó LÀ bề mặt mà Epic 4 sẽ đổ nội
// dung vào, và là phần tử mà hợp đồng đo `contains(anchorNode)` trên.
//
// 🔵 **2026-08-13 — mệnh đề "cùng hành vi" ở trên đã ĐƯỢC THU HẸP** (Sprint Change Proposal,
// Ice ký; FR21). Panel này sẽ mang **bản dịch AI tiếng Việt**, còn từ điển nhúng là
// zh→vi / en→vi ⇒ nó KHÔNG phải nguồn tra cứu: vai nay là `'display'`.
// Phần còn đúng của AC2 — và là phần đắt nhất — vẫn nguyên: hợp đồng KHÔNG phải sửa một
// dòng nào khi Epic 4 đổ nội dung vào. Chỉ **vai** khai lúc đăng ký quyết định hành vi.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 STORY 4.4 — DÒNG "BỘ HIỆU LỰC" + BỘ CHUYỂN, KHÔNG CẦN MỞ CÀI ĐẶT (FR69)
// ─────────────────────────────────────────────────────────────────────────────────
// I/O Matrix spec 4.4: *"Switch effective set from AI panel — effective set changes
// without opening Settings."* `<select>` dưới đây gọi THẲNG `setSelectedPromptSetName`
// (`promptSetState.ts`) — 0 lượt `invoke`, khuôn Quyết định 🔵 đầu tệp đó. Đây là nửa màn
// hình của mệnh đề I/O Matrix; nửa Rust (`resolve_two_tiers` đổi kết quả khi tầng đổi) đã
// đóng ở Phase 3 (`prompt_set_contract.rs::switching_between_two_resolvable_sets_needs_no_
// settings_reopen`).
import { onMounted, useTemplateRef } from 'vue'
import PanelFrame from './PanelFrame.vue'
import { useSelectionSurface } from './selectionContract'
import { dispatch } from '../commands'
import { t } from '../i18n'
import { loadPromptSets, promptSets, selectedPromptSetName, setSelectedPromptSetName } from '../promptSetState'
import type { DockviewPanelProps } from '../layout/panelProps'

defineProps<DockviewPanelProps>()

const surface = useTemplateRef<HTMLElement>('surface')
// 🔴 ĐỪNG gỡ lời gọi này khi thấy vai là `'display'`. FR48 (Story 3.3) và FR60 (Story 7.7)
// đọc vùng chọn ở đây bằng lệnh của RIÊNG chúng; `'display'` tắt đúng MỘT đường —
// `currentSelectionText()`, tức đường tra TỪ ĐIỂN — chứ không tắt việc bề mặt được đăng ký.
// Ghim bằng máy: `check-commands.mjs` Kiểm F ③.
useSelectionSurface(surface, 'display')

// Nạp danh sách bộ prompt hai tầng khi panel dựng — cùng lý do `openGlossaryManage()` nạp
// lại mỗi lần lớp phủ Quản lý mở: bộ có thể vừa được tạo/xoá/đổi tên ở một phiên trước, hoặc
// một Tác phẩm khác vừa mở.
onMounted(() => {
  void loadPromptSets()
})

function onPromptSetSelectChange(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLSelectElement)) return
  setSelectedPromptSetName(target.value === '' ? null : target.value)
}
</script>

<template>
  <PanelFrame owner="panel.ai_translation" status-key="panel.ai_translation.status">
    <div class="ai-prompt-bar">
      <p v-if="promptSets.length === 0" class="ai-prompt-empty">{{ t('panel.ai_translation.prompt_set_empty') }}</p>
      <label v-else class="ai-prompt-select-label">
        <span>{{ t('panel.ai_translation.prompt_set_label') }}</span>
        <select class="ai-prompt-select" :value="selectedPromptSetName ?? ''" @change="onPromptSetSelectChange">
          <option value="">{{ t('panel.ai_translation.prompt_set_placeholder') }}</option>
          <!-- aura-allow-text: DỮ LIỆU (tên bộ do người dùng đặt). -->
          <option v-for="s in promptSets" :key="`${s.tier}-${s.id}`" :value="s.name">{{ s.name }}</option>
        </select>
      </label>
      <p class="ai-prompt-status">
        <!-- aura-allow-text: KẾT QUẢ của `t()` (tên bộ nội suy qua tham số). -->
        {{
          selectedPromptSetName === null
            ? t('panel.ai_translation.prompt_set_none')
            : t('panel.ai_translation.prompt_set_using', { name: selectedPromptSetName })
        }}
      </p>
      <button type="button" class="ai-prompt-open" data-prompt-library-open @click="dispatch('prompt.library.open')">
        {{ t('command.prompt.library.open') }}
      </button>
    </div>
    <div ref="surface" class="ai-surface"></div>
  </PanelFrame>
</template>

<style scoped>
/* Story 4.4 — dòng "bộ hiệu lực" + bộ chuyển. `flex: none`: xem `PanelFrame.vue::.panel-body`
   — mọi con KHÔNG-CUỘN của slot phải khai nó tường minh khi slot mang HƠN MỘT con. */
.ai-prompt-bar {
  flex: none;
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 1);
  margin-bottom: var(--space-panel-block);
}

.ai-prompt-empty,
.ai-prompt-status {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.ai-prompt-select-label {
  display: flex;
  align-items: baseline;
  gap: calc(var(--space-unit) * 2);
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
}

.ai-prompt-select {
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
  background: var(--color-background);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 1.5);
}

.ai-prompt-open {
  align-self: flex-start;
  padding: 0;
  background: none;
  border: none;
  border-bottom: 1px solid var(--color-outline);
  cursor: pointer;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
}

/*
 * `flex: 1; min-height: 0` thay `height: 100%` — bắt buộc từ khi `.panel-body` mang HAI con
 * (bar + bề mặt), xem doc-comment `PanelFrame.vue::.panel-body`: `height: 100%` sẽ tự đo
 * theo chiều cao `.panel-body`, cộng dồn với chiều cao `.ai-prompt-bar` mà tràn khỏi panel.
 */
.ai-surface {
  flex: 1;
  min-height: 0;
}
</style>
