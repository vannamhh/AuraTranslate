<script setup lang="ts">
// Bốn ô xuất xứ tài liệu — Story 6.15, FR128/AD-43.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT BẢN CÀI ĐẶT, DÙNG CHUNG BỞI MÀN XEM TRƯỚC NHẬP VÀ DANH SÁCH CHƯƠNG
// ─────────────────────────────────────────────────────────────────────────────
// `ImportPreviewOverlay.vue` (đầu Chương con trỏ, trong màn xem trước nhập) và
// `LibraryMode.vue` (cạnh cụm `chapter-reorg`, cho Chương con trỏ) đều dựng khối này —
// component PHẲNG, thuần props-xuống, không tự gọi IPC (cùng khuôn `ChapterImage.vue`, kho
// không có `src/components/`). Chỗ gọi tự quyết định LƯU đi đâu: `ImportPreviewOverlay` ghi
// vào state ghi đè (draft, chưa xuống đĩa); `LibraryMode` gọi thẳng lệnh ghi (`update_chapter_
// origin`, xuống đĩa ngay).
//
// ─────────────────────────────────────────────────────────────────────────────
// MỘT NHÃN DUY NHẤT CHO Ô RỖNG — "không tìm thấy" (Ice chốt 2026-09-10)
// ─────────────────────────────────────────────────────────────────────────────
// Dùng CHO MỌI Chương, kể cả Chương nhập từ tệp/dán tay nơi hệ thống chưa từng tìm — xem
// §Design Notes spec 6.15 "Cái giá đã nhận của một nhãn duy nhất". Không nhãn thứ hai.
//
// ⚠️ Kho KHÔNG có tiền lệ `font-style: italic` cho trạng thái rỗng (EXPERIENCE.md nói "chữ
// nghiêng", nhưng đây là lượt ĐẦU TIÊN dựng nó) — quy ước thật đang dùng khắp kho là chữ nhỏ
// (`--font-ui-sm`) + màu phụ (`--color-on-surface-variant`), áp qua `::placeholder` bên dưới.
import { t } from './i18n'

type OriginField = 'author' | 'siteName' | 'url' | 'publishedAt'

const props = defineProps<{
  author: string | null
  siteName: string | null
  url: string | null
  publishedAt: string | null
  /** Vô hiệu hoá cả bốn ô trong lúc một lượt lưu khác đang bay — cùng khuôn `<fieldset
   * :disabled>` của `ImportPreviewOverlay.vue` (form sửa luật làm sạch). */
  disabled?: boolean
}>()

/**
 * Phát ra TOÀN BỘ bốn trường (giá trị vừa gõ ĐÈ lên đúng MỘT ô, ba ô còn lại giữ nguyên giá
 * trị đang hiện) — khớp chữ ký cả hai lệnh Rust đích (`set_chapter_origin_override`/
 * `update_chapter_origin`), cả hai đều nhận bốn trường một lượt, không patch từng ô rời.
 */
const emit = defineEmits<{
  commit: [origin: { author: string; siteName: string; url: string; publishedAt: string }]
}>()

function commitField(field: OriginField, event: Event): void {
  const value = (event.target as HTMLInputElement).value
  emit('commit', {
    author: field === 'author' ? value : (props.author ?? ''),
    siteName: field === 'siteName' ? value : (props.siteName ?? ''),
    url: field === 'url' ? value : (props.url ?? ''),
    publishedAt: field === 'publishedAt' ? value : (props.publishedAt ?? ''),
  })
}
</script>

<template>
  <div class="chapter-origin">
    <h4 class="chapter-origin-heading">{{ t('chapter.origin.heading') }}</h4>
    <div class="chapter-origin-grid">
      <label class="chapter-origin-field">
        <span class="chapter-origin-label">{{ t('chapter.origin.author_label') }}</span>
        <input
          class="chapter-origin-input"
          type="text"
          :value="author ?? ''"
          :placeholder="t('chapter.origin.empty')"
          :disabled="disabled === true"
          @change="(e) => commitField('author', e)"
        />
      </label>
      <label class="chapter-origin-field">
        <span class="chapter-origin-label">{{ t('chapter.origin.site_name_label') }}</span>
        <input
          class="chapter-origin-input"
          type="text"
          :value="siteName ?? ''"
          :placeholder="t('chapter.origin.empty')"
          :disabled="disabled === true"
          @change="(e) => commitField('siteName', e)"
        />
      </label>
      <label class="chapter-origin-field">
        <span class="chapter-origin-label">{{ t('chapter.origin.url_label') }}</span>
        <input
          class="chapter-origin-input"
          type="text"
          :value="url ?? ''"
          :placeholder="t('chapter.origin.empty')"
          :disabled="disabled === true"
          @change="(e) => commitField('url', e)"
        />
      </label>
      <label class="chapter-origin-field">
        <span class="chapter-origin-label">{{ t('chapter.origin.published_at_label') }}</span>
        <input
          class="chapter-origin-input"
          type="text"
          :value="publishedAt ?? ''"
          :placeholder="t('chapter.origin.empty')"
          :disabled="disabled === true"
          @change="(e) => commitField('publishedAt', e)"
        />
      </label>
    </div>
  </div>
</template>

<style scoped>
.chapter-origin {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.chapter-origin-heading {
  margin: 0;
  font-family: var(--face-ui-md-strong);
  font-size: var(--font-ui-md-strong);
  line-height: var(--leading-ui-md-strong);
  color: var(--color-on-surface);
}

.chapter-origin-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 6px 10px;
}

.chapter-origin-field {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.chapter-origin-label {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.chapter-origin-input {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface);
  background-color: var(--color-surface);
  border: 1px solid var(--color-outline-faint);
  border-radius: var(--radius-sm);
  padding: 4px 6px;
}

.chapter-origin-input::placeholder {
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface-variant);
  /* ⚠️ KHÔNG có `font-style: italic` ở đây, dù EXPERIENCE.md:331-336 đòi ĐÍCH DANH chữ nghiêng
     cho "không tìm thấy". Đo 2026-09-10: `check:tokens` Kiểm B xếp `font-style` vào nhóm
     "cỡ/họ chữ viết thẳng" và ĐỎ ngay (`src/ChapterOrigin.vue:161 — cỡ/họ chữ viết thẳng:
     font-style: italic`) — một lượt thêm nó đã bị gỡ lại vì thế. Đóng vế "chữ nghiêng" cần
     một token nghiêng THẬT trong `src/tokens/`, hoặc một miễn trừ CÓ TÊN — cả hai đều là
     quyết định hệ thống thiết kế, không phải một dòng CSS. Nợ có chủ: `deferred-work.md`. */
}

.chapter-origin-input:disabled {
  color: var(--color-on-surface-variant);
  background-color: var(--color-surface-sunken);
}
</style>
