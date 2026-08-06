<script setup lang="ts">
// Một khối NGUỒN của Panel Lookup — Story 1.17, AC1 · AC2 · AC3 · AC4 · Quyết định #5.
//
// 🔴 AD-19 — KHÔNG hợp nhất, KHÔNG xếp hạng, KHÔNG chọn "câu trả lời": component này
// render ĐÚNG những gì `group`/`senses` mang tới, theo ĐÚNG thứ tự nhận được. ⛔ Không
// `sort()`, ⛔ không `new Set()` trên `gloss` — cùng lớp lỗi mà Bẫy 2 của story cảnh báo.
//
// 🔴 **Quyết định #5(a) — nhiều đầu mục CÙNG headword hiện LIỀN NHAU, ⛔ GỘP.** Bản đầu
// của 1.17 lặp thẳng trên một danh sách `senses` PHẲNG, nên 18 ca trùng `headword` của
// `dict-vietphrase.db` đọc ra thành MỘT danh sách nghĩa dài liên tục — gần với *"gộp hiển
// thị"* (đường story ⛔ chọn) hơn là *"hiện liền nhau thành từng cụm"* (bắt ở code review
// 2026-08-07). `groupSensesByEntry` giữ ranh giới đầu mục: mỗi cụm là **một đầu mục**, và
// đầu mục THẬT của nó (`EntryHit.headword`) được ghi ra — ⛔ đánh số, ⛔ hợp nhất.
import { computed } from 'vue'
import type { SenseRecord, SourceGroup } from '../config/dict'

const props = defineProps<{
  group: SourceGroup
  senses: readonly SenseRecord[]
}>()

/** Một đầu mục cùng đúng những nghĩa của RIÊNG nó. */
type EntryCluster = {
  entryId: number
  headword: string
  senses: SenseRecord[]
}

/**
 * 🔴 Gom nghĩa về **đúng đầu mục sinh ra nó**, giữ nguyên thứ tự `group.entries` (Rust đã
 * sắp theo `entry_id`) — hàm THUẦN, ⛔ sắp lại, ⛔ khử trùng, ⛔ chọn đầu mục nào "đúng".
 *
 * ⚠️ Đầu mục hiện ra là [`EntryHit.headword`] — chữ THẬT trong từ điển, ⛔ truy vấn người
 * dùng bôi đen. Hai thứ đó khác nhau ở đúng ca `headword_simp` khớp (tra `国` giản thể mà
 * đầu mục ghi `國` phồn thể), và hiện truy vấn ở đó là hiện SAI CHỮ.
 */
const clusters = computed<EntryCluster[]>(() => {
  const byEntry = new Map<number, SenseRecord[]>()
  for (const sense of props.senses) {
    const bucket = byEntry.get(sense.entry_id)
    if (bucket === undefined) byEntry.set(sense.entry_id, [sense])
    else bucket.push(sense)
  }

  return props.group.entries.map((hit) => ({
    entryId: hit.entry_id,
    headword: hit.headword,
    senses: byEntry.get(hit.entry_id) ?? [],
  }))
})

/**
 * ⚠️ Nhãn đầu mục CHỈ hiện khi nguồn này có **từ hai đầu mục trở lên** — với một đầu mục
 * duy nhất nó lặp lại đúng chữ đã nằm ở vùng đầu mục phía trên, và một dòng thừa trên mọi
 * lượt tra thường là cái giá quá đắt cho một ca thiểu số.
 */
const showEntryHeadwords = computed(() => clusters.value.length >= 2)
</script>

<template>
  <section class="lookup-source">
    <div class="lookup-source-head">
      <!-- aura-allow-text: `display_name` là DỮ LIỆU đọc từ `dict_source` của chính tệp
           .db (AC2, FR31) — ⛔ không chuỗi giao diện của vi.json. -->
      <span class="lookup-source-name">{{ group.source.display_name }}</span>
    </div>

    <template v-for="cluster in clusters" :key="cluster.entryId">
      <!-- Quyết định #5(a) — ranh giới đầu mục hiện ra, ⛔ đánh số, ⛔ gộp. -->
      <!-- aura-allow-text: đầu mục THẬT trong từ điển — DỮ LIỆU (`EntryHit.headword`). -->
      <p v-if="showEntryHeadwords" class="lookup-entry-headword">{{ cluster.headword }}</p>

      <div v-for="sense in cluster.senses" :key="sense.sense_id" class="lookup-sense">
        <!-- AC3 — từ loại vắng mặt (`pos = null`) ⇒ ⛔ render một hàng rỗng nào. -->
        <p v-if="sense.pos !== null" class="lookup-pos">
          <!-- aura-allow-text: nhãn từ loại — DỮ LIỆU từ điển. -->
          <span>{{ sense.pos }}</span>
          <!-- 🔴 AC4/FR35 — đọc cờ `pos_is_foreign` do RUST quyết (AD-1), ⛔ tự so
               `pos_lang !== null`: `pos_lang = "vi"` CÓ ngôn ngữ nhưng ⛔ là ngoại ngữ,
               và bản đầu dán chip `VI` lên đúng những nhãn tiếng Việt. -->
          <!-- aura-allow-text: mã ngôn ngữ — DỮ LIỆU. -->
          <span v-if="sense.pos_is_foreign" class="lookup-foreign-flag">{{ sense.pos_lang }}</span>
        </p>

        <!-- aura-allow-text: nghĩa — DỮ LIỆU từ điển, đúng thứ FR28 gọi là "nghĩa". -->
        <p class="lookup-gloss">{{ sense.gloss }}</p>

        <p v-for="(example, i) in sense.examples" :key="`ex-${i}`" class="lookup-example">
          <!-- aura-allow-text: câu ví dụ + bản dịch — DỮ LIỆU từ điển. -->
          <span>{{ example.text }}</span>
          <template v-if="example.translation !== null">
            <span> — </span>
            <!-- aura-allow-text: bản dịch ví dụ — DỮ LIỆU từ điển. -->
            <span>{{ example.translation }}</span>
            <!-- AC4 — cùng luật `pos_is_foreign`, cùng một hàm quyết ở Rust. -->
            <!-- aura-allow-text: mã ngôn ngữ của bản dịch — DỮ LIỆU (FR35). -->
            <span v-if="example.translation_is_foreign" class="lookup-foreign-flag">{{
              example.translation_lang
            }}</span>
          </template>
        </p>

        <!-- AC3 — trích dẫn có vạch trái `primary`, PHÂN BIỆT với ví dụ (⛔ `--line-2` của khối nguồn). -->
        <p v-for="(citation, i) in sense.citations" :key="`cite-${i}`" class="lookup-citation">
          <!-- aura-allow-text: trích dẫn + xuất xứ — DỮ LIỆU từ điển (FR30: bảng RIÊNG với ví dụ). -->
          <span>{{ citation.text }}</span>
          <template v-if="citation.work !== null || citation.author !== null">
            <span> — </span>
            <!-- aura-allow-text: tên tác phẩm — DỮ LIỆU (xuất xứ trích dẫn, FR30). -->
            <span v-if="citation.work !== null">{{ citation.work }}</span>
            <!-- aura-allow-text: tên tác giả — DỮ LIỆU (xuất xứ trích dẫn, FR30). -->
            <span v-if="citation.author !== null"> ({{ citation.author }})</span>
          </template>
        </p>

        <!-- aura-allow-text: ghi chú — DỮ LIỆU từ điển, phần thứ sáu của FR28. -->
        <p v-if="sense.note !== null" class="lookup-note">{{ sense.note }}</p>
      </div>
    </template>

  </section>
</template>

<style scoped>
.lookup-source {
  margin-top: var(--space-panel-block);
}

.lookup-source-head {
  padding-bottom: 5px;
  margin-bottom: var(--space-panel-block);
  border-bottom: 1px solid var(--color-outline-faint);
}

/* AC2 — nhãn nguồn khai token `ui-label`/`primary`, ⛔ không đường nào làm nó biến mất. */
.lookup-source-name {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-primary);
  text-transform: uppercase;
}

/*
 * 🔴 Mâu thuẫn tài liệu #4 (Task 0, chốt theo mockup) — vạch trái 2px + thụt 13px ở CẤP
 * NGHĨA, ⛔ không cấp nguồn: một nguồn 9 nghĩa cho ra CHÍN vạch, đúng thứ giúp mắt nhặt
 * ranh giới nghĩa trên dữ liệu mật độ thật (18–20+ nghĩa một nguồn).
 */
/*
 * Quyết định #5(a) — nhãn đầu mục của MỘT cụm. Khai token `lookup-gloss` (họ `read` —
 * UX-DR12: đầu mục là **nội dung**, ⛔ bộ máy).
 *
 * ⚠️ Ranh giới đọc ra bằng **NÉT + khoảng trắng**, ⛔ bằng trọng lượng chữ: bảng token ⛔
 * có biến nào cho 600 (`ui-md` khai 400, `ui-label` khai 700 — món nợ đã ghi trong sổ
 * `deviations`), và một `font-weight: 600` viết thẳng ở đây làm Kiểm B của `check-tokens`
 * ĐỎ, đúng thứ AD-34 tồn tại để chặn. `ornament` là màu của nét (UX-DR5), ⛔ của chữ.
 */
.lookup-entry-headword {
  margin: var(--space-panel-block) 0 4px 0;
  padding-top: var(--space-panel-block);
  border-top: 1px solid var(--color-outline-faint);
  font-family: var(--face-lookup-gloss);
  font-size: var(--font-lookup-gloss);
  line-height: var(--leading-lookup-gloss);
  color: var(--color-on-surface);
}

/* Cụm ĐẦU TIÊN ⛔ cần nét ngăn — nhãn nguồn ngay trên nó đã là ranh giới. */
.lookup-entry-headword:first-of-type {
  margin-top: 0;
  padding-top: 0;
  border-top: none;
}

.lookup-sense {
  padding-left: 13px;
  border-left: 2px solid var(--color-outline-faint);
  margin-bottom: var(--space-panel-block);
}

/* AC3 — "từ loại: họ read in nghiêng màu on-surface-variant" — khai token `lookup-example`. */
.lookup-pos {
  margin: 0 0 2px 0;
  font-family: var(--face-lookup-example);
  font-size: var(--font-lookup-example);
  font-style: var(--style-lookup-example);
  font-synthesis: var(--synthesis-lookup-example);
  line-height: var(--leading-lookup-example);
  color: var(--color-on-surface-variant);
}

/* AC3 — nghĩa khai token `lookup-gloss` CỦA CHÍNH NÓ. */
.lookup-gloss {
  margin: 0;
  font-family: var(--face-lookup-gloss);
  font-size: var(--font-lookup-gloss);
  line-height: var(--leading-lookup-gloss);
  color: var(--color-on-surface);
}

/*
 * AC3/AC9 — ví dụ VÀ trích dẫn khai token `lookup-example`; `font-synthesis` là NGƯỜI
 * TIÊU THỤ THỨ HAI của nó (`source-hanviet` đã đóng ở 1.16, `deferred-work.md:133`).
 */
.lookup-example {
  margin: 4px 0 0 0;
  font-family: var(--face-lookup-example);
  font-size: var(--font-lookup-example);
  font-style: var(--style-lookup-example);
  font-synthesis: var(--synthesis-lookup-example);
  line-height: var(--leading-lookup-example);
  color: var(--color-on-surface-variant);
}

/* AC3 — vạch trái `primary`, PHÂN BIỆT với vạch `--color-outline-faint` của khối nghĩa. */
.lookup-citation {
  margin: 4px 0 0 11px;
  padding-left: 11px;
  border-left: 2px solid var(--color-primary);
  font-family: var(--face-lookup-example);
  font-size: var(--font-lookup-example);
  font-style: var(--style-lookup-example);
  font-synthesis: var(--synthesis-lookup-example);
  line-height: var(--leading-lookup-example);
  color: var(--color-on-surface-variant);
}

.lookup-note {
  margin: 4px 0 0 0;
  font-family: var(--face-lookup-example);
  font-size: var(--font-lookup-example);
  font-style: var(--style-lookup-example);
  font-synthesis: var(--synthesis-lookup-example);
  line-height: var(--leading-lookup-example);
  color: var(--color-on-surface-variant);
}

/* AC4 — dấu hiệu ngoại ngữ khai `ui-label`, tách biệt màu để "đánh dấu RÕ". */
.lookup-foreign-flag {
  margin-left: 6px;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  line-height: var(--leading-ui-label);
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-primary);
  text-transform: uppercase;
}
</style>
