<script setup lang="ts">
// Ảnh đúng vị trí — Story 6.14, FR42 · FR43.
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 MỘT BẢN CÀI ĐẶT, DÙNG CHUNG BỞI LƯỚI VÀ CHẾ ĐỘ ĐỌC
// ─────────────────────────────────────────────────────────────────────────────
// `GridPanel.vue` và `ReadingMode.vue` chỉ khác nhau ở chỗ Chế độ đọc bọc thêm một
// `<figcaption>` NGOÀI component này (nó có bản dịch của `caption`, thứ mỗi bề mặt hiển thị ở
// một chỗ khác trong cây DOM riêng của nó) — dựng `<img>`, bắt sự kiện trượt, và khung giữ chỗ
// khi tệp thiếu thì SỐNG Ở ĐÂY, một lần.
//
// Sống PHẲNG ở `src/` như `StatusBar.vue` — kho này không có `src/components/`; một thành phần
// dùng chung bởi hai thư mục khác nhau (`src/panels/`, `src/modes/`) sống ở gốc.
//
// ─────────────────────────────────────────────────────────────────────────────
// VÌ SAO GHÉP ĐƯỜNG DẪN Ở ĐÂY, KHÔNG Ở RUST
// ─────────────────────────────────────────────────────────────────────────────
// Rust gửi `assetsDir` (thư mục `assets/` của Tác phẩm đang mở) ĐÚNG MỘT LẦN cho cả Chương/cả
// lượt đọc, không lặp lại nó ở từng ảnh (`ChapterSegments.assets_dir` / `ReadingRun.assets_dir`)
// — ghép nó với `fileName` (luôn phẳng, `<uuid>.jpg|png|gif|webp`, không thư mục con, khoá bởi
// CHECK của `ASSET_DDL`) là việc RẺ, làm ở đây.
//
// ⚠️ Dấu `/` giữa hai nửa, KỂ CẢ trên Windows (`assetsDir` mang `\` theo native): `std::path::Path`
// của Rust — nơi chuỗi này đáp xuống sau `convertFileSrc` — coi `/` là một dấu phân tách HỢP LỆ
// trên MỌI nền tảng nó biên dịch, Windows kể cả (`std::path::is_separator`). Không cần biết
// dấu phân tách gốc của `assetsDir` là gì để ghép đúng.
//
// ─────────────────────────────────────────────────────────────────────────────
// TỆP ẢNH THIẾU ⇒ KHUNG GIỮ CHỖ MANG DANH TÍNH (Ice chốt 2026-09-10)
// ─────────────────────────────────────────────────────────────────────────────
// `<img>` trượt (tệp bị xoá tay, ổ mạng chưa gắn, `.atproj` thiếu `assets/`) đổi sang một
// khung giữ chỗ — KHÔNG throw, KHÔNG khoảng trắng vô danh: nó mang `file_name` (luôn có) và
// `source_url` (nếu có), CHỌN ĐƯỢC để copy (văn bản thật, không `aria-hidden`).
import { computed, ref, watch } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { t } from './i18n'

const props = defineProps<{
  fileName: string
  assetsDir: string
  sourceUrl: string | null
  /** Alt-text ĐÃ DỊCH — đi thẳng vào thuộc tính `alt`, KHÔNG hiện trên trang (Design Notes spec 6.14). */
  altText: string
}>()

/** `<img>` vừa trượt lần gần nhất cho ĐÚNG cặp `(assetsDir, fileName)` hiện tại. */
const failed = ref(false)

// Đổi ảnh (đổi Chương, hoặc gộp/tách làm mảng ảnh dựng lại) ⇒ cho `<img>` MỘT cơ hội mới —
// một ảnh trượt ở Chương TRƯỚC không được ám vào ảnh Chương SAU chỉ vì component không bị gỡ.
watch(
  () => `${props.assetsDir}\u0000${props.fileName}`,
  () => {
    failed.value = false
  },
)

const src = computed(() => {
  const dir = props.assetsDir.replace(/[/\\]+$/, '')
  return convertFileSrc(`${dir}/${props.fileName}`)
})

function onImageError(): void {
  failed.value = true
}
</script>

<template>
  <img v-if="!failed" class="chapter-image" :src="src" :alt="altText" @error="onImageError" />
  <!--
    Khung giữ chỗ — `role="img"` + `aria-label` cho công nghệ trợ giúp một câu tóm tắt; hai
    dòng BÊN TRONG là văn bản THẬT (chọn được để copy), không phải nội dung của `aria-label`.
  -->
  <div v-else class="chapter-image-missing" role="img" :aria-label="t('image.missing_alt')">
    <p class="chapter-image-missing-line"
      >{{ t('image.missing_file_label') }}: <!-- aura-allow-text: `fileName` là DỮ LIỆU của Tác phẩm (tên tệp), không câu giao diện — nhãn ngay trước đã qua t(). --><span>{{ fileName }}</span></p
    >
    <p v-if="sourceUrl !== null" class="chapter-image-missing-line"
      >{{ t('image.missing_source_label') }}: <!-- aura-allow-text: `sourceUrl` là DỮ LIỆU của Tác phẩm (URL), không câu giao diện — nhãn ngay trước đã qua t(). --><span>{{ sourceUrl }}</span></p
    >
  </div>
</template>

<style scoped>
/* Ảnh không được phép phá bố cục — cả hai bề mặt đều đặt nó trong một hộp chữ tự do chiều rộng. */
.chapter-image {
  display: block;
  max-width: 100%;
  height: auto;
}

.chapter-image-missing {
  display: flex;
  flex-direction: column;
  gap: 7px;
  padding: 7px 7px;
  border: 1px solid var(--color-outline-faint);
  border-radius: var(--radius-sm);
  background-color: var(--color-surface-sunken);
}

.chapter-image-missing-line {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  overflow-wrap: anywhere;
}
</style>
