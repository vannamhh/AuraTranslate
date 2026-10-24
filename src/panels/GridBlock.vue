<script setup lang="ts">
import { inject } from 'vue'
import ChapterImage from '../ChapterImage.vue'
import GlossaryConfirmStrip from '../GlossaryConfirmStrip.vue'
import ProofreaderStrip from '../ProofreaderStrip.vue'
import TmFuzzyStrip from '../TmFuzzyStrip.vue'
import { t } from '../i18n'
import { concordanceSourceHighlight } from './concordanceState'
import { editorAssetsDir, editorChapterAssets, editorEditedShape } from './editorPanelState'
import { ruleClassOf, textShapeOf } from './editorSegments'
import type { SegmentRuleValue } from './editorSegments'
import { GRID_BLOCK_CONTEXT } from './gridBlockContext'
import type { GridBlockView } from './gridBlockContext'
import SourceHanViet from './SourceHanViet.vue'
import { viewMode } from './sourcePanelState'

const props = defineProps<{
  block: GridBlockView
  caretId: number | null
  anchorId: number | null
  stripRow: number
  hasStripTrack: boolean
}>()

const ctx = inject(GRID_BLOCK_CONTEXT)
if (ctx === undefined) throw new Error('GridBlock needs the grid block context')
const {
  selectedRowClassById,
  unlinkedIds,
  rowErrorLabelById,
  rowErrorTextById,
  firstSegmentId,
  showHanViet,
  sourceTokenClass,
  cutOffsetsOf,
  cutCountOf,
  glossaryTermsOf,
  sourcePieceInfoOf,
  trailingImagesOf,
  leadingImagesOf,
  roleLabelKeys: ROLE_LABEL_KEYS,
  setStripTrackRef,
  onSourceCellMouseUp,
  onGlossaryPieceEnter,
  onGlossaryPieceLeave,
  onColumnFocusOut,
  onCellMouseDown,
  onCellMouseUp,
  onBeforeInput,
  onCompositionStart,
  onCompositionEnd,
  onEditInput,
  onEditKeydown,
  onStripFocusOut,
} = ctx

const STATE_LABEL_KEYS: Readonly<Record<SegmentRuleValue, string>> = {
  confirmed: 'panel.grid.state_confirmed',
  primary: 'panel.grid.state_editing',
  draft: 'panel.grid.state_draft',
  'tm-rule': 'panel.grid.state_tm',
  none: 'panel.grid.state_untranslated',
}

function concordanceHighlightOf(id: number) {
  const highlight = concordanceSourceHighlight.value
  return highlight !== null && highlight.segmentId === id ? highlight : null
}

function chapterAssetsOf(id: number) {
  return id === firstSegmentId.value ? editorChapterAssets.value : null
}

function ruleOf(id: number): SegmentRuleValue {
  return props.caretId === id ? 'primary' : (props.block.rules.get(id) ?? 'none')
}

function ruleClassForRow(id: number): string {
  return ruleClassOf(ruleOf(id)) ?? ''
}

function ruleSlotClass(cls: string): string[] | null {
  return cls === '' ? null : ['rule', cls]
}

function cellLabelOf(id: number, index: number): string {
  const rule = t(STATE_LABEL_KEYS[ruleOf(id)])
  const state = unlinkedIds.value.has(id) ? t('panel.grid.state_with_unlinked', { state: rule }) : rule
  const error = rowErrorTextById.value.get(id)
  return error === undefined
    ? t('panel.grid.cell_label', { n: String(index + 1), state })
    : t('panel.grid.cell_label_error', { n: String(index + 1), state, error })
}
// vue-tsc 3.3.9 does not count a `v-memo` expression as a read of these helpers.
defineExpose({ concordanceHighlightOf, chapterAssetsOf })
</script>

<template>
        <div
          class="grid-block"
          :style="{ gridTemplateRows: `repeat(${block.segments.length + (stripRow < 0 ? 0 : 1)}, auto)` }"
        >
        <!--
          Cột ① — VẠCH TRẠNG THÁI (UX-DR19, sáu giá trị).

          `aria-hidden`: vạch là kênh **thị giác** của trạng thái; trình đọc màn hình nghe
          trạng thái qua tên truy cập (`aria-label`) của ô bản dịch.
        -->
        <div class="col col-rule" aria-hidden="true">
          <template
            v-for="s in block.segments"
            :key="s.id"
            v-memo="[s, ruleClassForRow(s.id), selectedRowClassById.get(s.id), s.id === anchorId]"
          >
            <div
              class="cell cell-rule"
              :class="[{ 'para-end': s.is_paragraph_end, 'cell-unlinked': unlinkedIds.has(s.id) }, selectedRowClassById.get(s.id)]"
            >
              <div class="margin-slot" :class="ruleSlotClass(ruleClassForRow(s.id))"></div>
            </div>
            <div v-if="s.id === anchorId" class="strip-pad" aria-hidden="true"></div>
          </template>
        </div>

        <!--
          Cột ② — SỐ CÂU. `on-surface-variant`, KHÔNG `ornament` (Quyết định #9(a)): đây là
          **chữ thật**, không phải nét, và `ornament` đo được 2,44 (sáng) / 2,64 (tối) trên
          `surface` — trượt AA.
          aura-allow-text: số thứ tự hàng là DỮ LIỆU, không một chuỗi giao diện.
        -->
        <div class="col col-num" aria-hidden="true">
          <template
            v-for="(s, j) in block.segments"
            :key="s.id"
            v-memo="[s, block.start + j, selectedRowClassById.get(s.id), s.id === anchorId]"
          >
          <div
            class="cell cell-num"
            :class="[{ 'para-end': s.is_paragraph_end, omitted: s.is_omitted }, selectedRowClassById.get(s.id)]"
          ><!-- aura-allow-text: số thứ tự hàng — một con số dựng tại chỗ hiển thị, đúng luật "định dạng số và ngày giờ CHỈ ở frontend" (§Consistency Conventions), không một chuỗi giao diện dịch được. -->{{ block.start + j + 1 }}</div>
          <div v-if="s.id === anchorId" class="strip-pad" aria-hidden="true"></div>
          </template>
        </div>

        <!-- Cột ③ — NGUYÊN VĂN. Một bề mặt vùng chọn duy nhất, vai `'source'` (AC7). -->
        <div class="col col-src" :class="sourceTokenClass">
          <template
            v-for="s in block.segments"
            :key="s.id"
            v-memo="[
              s,
              selectedRowClassById.get(s.id),
              s.id === anchorId,
              cutOffsetsOf(s.id),
              glossaryTermsOf(s.id),
              concordanceHighlightOf(s.id),
              trailingImagesOf(s.id),
              chapterAssetsOf(s.id),
              editorAssetsDir,
              showHanViet,
              viewMode,
            ]"
          >
          <div
            class="cell cell-src"
            :class="[
              {
                'para-end': s.is_paragraph_end,
                omitted: s.is_omitted,
              },
              selectedRowClassById.get(s.id),
            ]"
            :data-segment-id="s.id"
            :data-cut-count="cutCountOf(s.id)"
            data-col="src"
            @mouseup="onSourceCellMouseUp"
          >
            <!--
              🔵 Story 6.14 (nợ ③) — NHÃN VAI, trước mọi nội dung khác của ô. `aria-hidden`
              KHÔNG đặt: đây là chữ THẬT phân biệt hàng này với văn xuôi, không một hoạ tiết.
              `s.role` không có `data-src-start` ⇒ vô hình với `sourceCutOffsetOf` (nó chỉ đọc
              neo, không đếm mù toàn bộ ô — xem doc-comment `editorSegments.ts:282`).
            -->
            <span v-if="s.role !== null && ROLE_LABEL_KEYS[s.role] !== undefined" class="role-label" :data-role="s.role">{{ t(ROLE_LABEL_KEYS[s.role]!) }}</span>
            <!--
              🔵 Story 6.14 (FR42) — ảnh neo `0` đứng Ở ĐẦU ô của câu ĐẦU TIÊN, TRƯỚC chữ.
              `<ChapterImage>` không có `data-src-start` ⇒ cùng lý do vô hình với cắt/Hán Việt.
            -->
            <figure v-for="asset in leadingImagesOf(s.id)" :key="'lead-' + asset.asset_id" class="grid-image">
              <ChapterImage
                :file-name="asset.file_name"
                :assets-dir="editorAssetsDir"
                :source-url="asset.source_url"
                :alt-text="asset.alt_text ?? ''"
              />
            </figure>
            <!--
              AC8 — Hán Việt sống TRONG ô nguyên văn, hai chế độ FR19, người dùng tự bật tắt.
              `surface-role="cell"` nhượng lượt đăng ký cho CỘT — xem `hanVietSurfaces.ts`.
            -->
            <SourceHanViet
              v-if="showHanViet"
              :source-text="s.source_text"
              :view-mode="viewMode"
              :cuts="cutOffsetsOf(s.id)"
              :glossary-terms="glossaryTermsOf(s.id).spans"
              surface-role="cell"
            />
            <!--
              🔵 2026-08-17, AC7 — nguyên văn chia thành mảnh theo TẬP ĐIỂM CẮT đang chờ CỘNG
              (Story 3.4b) biên thuật ngữ Glossary, mỗi ranh giới một mảnh. Không điểm nào ⇒
              `sourcePiecesOf` trả đúng MỘT mảnh, tức đường thường giữ nguyên hình dạng cây.

              ⚠️ Dấu cắt RỖNG và mang `aria-hidden` — nó không phải chữ, và một ký tự thật ở
              đây sẽ đi vào `Selection.toString()` của Auto-Lookup rồi vào mọi lượt sao chép.

              🔴 Story 3.4b — `cut-mark` vẽ CHỈ khi `piece.isPendingCut`: một mảnh sinh RIÊNG
              bởi biên thuật ngữ (không phải một điểm cắt người dùng) không được vẽ dấu ngắt
              đoạn giả — xem `sourcePieceStartsOf` và Design Notes "ba tập điểm, đừng gộp" của
              `3-4b-…md`.
            -->
            <template v-else
              ><template v-for="(piece, i) in sourcePieceInfoOf(s.id, s.source_text)" :key="i"
                ><span v-if="i > 0 && piece.isPendingCut" class="cut-mark" aria-hidden="true"></span
                ><!-- 🔵 2026-08-17 (AC9) — mỗi mảnh nay là một `<span>` mang NEO
                     `data-src-start`. Trước lượt này mảnh là text node trần và
                     `sourceCutOffsetOf` phải **đếm mù** mọi text node trong ô — phép đếm đó
                     sai hẳn ở tab Hán Việt *(đo: 17 và 19 trên một câu 5 chữ)*.
                     ⚠️ `<span>` KHÔNG thêm một ký tự nào vào `Selection.toString()`, nên
                     Auto-Lookup (FR21) không bị chạm.
                     🔵 Story 3.4b — `glossary-confirmed`/`glossary-pending` tô dấu thuật ngữ
                     đã khớp. `@mouseenter`/`@mouseleave` là vế CHUỘT; vế BÀN PHÍM/TIÊU ĐIỂM
                     đi qua `onSourceSelectionChange()` ở khối kịch bản phía trên — KHÔNG
                     `tabindex` mới ở đây: caret di chuyển bằng `Selection.modify()` (AC11)
                     không cần phần tử này tự focus được. Không đi qua `dispatch()` — Kiểm A
                     của `check:commands` chỉ canh `@click`.
                     aura-allow-text: nguyên văn của Tác phẩm — DỮ LIỆU, không chuỗi giao
                     diện (NFR16). Không `v-html` (AD-16). --><span
                  class="src-piece"
                  :class="{
                    'glossary-confirmed': piece.glossary?.isConfirmed === true,
                    'glossary-pending': piece.glossary?.isConfirmed === false,
                    'concordance-source': piece.concordance,
                  }"
                  :data-src-start="piece.start"
                  @mouseenter="onGlossaryPieceEnter(piece.glossary)"
                  @mouseleave="onGlossaryPieceLeave()"
                >{{ piece.text }}</span></template
              ></template
            >
            <!-- 🔵 Story 6.14 (FR42) — ảnh SAU câu này, ở CUỐI ô, đúng neo hai ảnh cùng neo giữ nguyên thứ tự `asset.id`. -->
            <figure v-for="asset in trailingImagesOf(s.id)" :key="asset.asset_id" class="grid-image">
              <ChapterImage
                :file-name="asset.file_name"
                :assets-dir="editorAssetsDir"
                :source-url="asset.source_url"
                :alt-text="asset.alt_text ?? ''"
              />
            </figure>
          </div>
          <div v-if="s.id === anchorId" class="strip-pad" aria-hidden="true"></div>
          </template>
        </div>

        <!--
          Cột ④ — BẢN DỊCH. Một bề mặt vùng chọn duy nhất, vai `'display'` (B1).

          🔴 Ba handler sống ở **CỘT**, không ở từng ô: sự kiện nổi bọt, và N listener cho
          9.850 hàng là N lượt đăng ký mà một lượt là đủ.
        -->
        <div
          class="col col-tgt tok-editor"
          @focusout="onColumnFocusOut"
          @mousedown="onCellMouseDown"
          @mouseup="onCellMouseUp"
          @beforeinput="onBeforeInput"
          @compositionstart="onCompositionStart"
          @compositionend="onCompositionEnd"
          @input="onEditInput"
          @keydown="onEditKeydown"
        >
          <!--
            aura-allow-text: BẢN DỊCH của người dùng — dữ liệu, không chuỗi giao diện.

            🔴 `contenteditable` trên **MỌI** ô (Quyết định #3(b)). Không một binding động nào:
            giá trị không đổi theo trạng thái, nên Vue không bao giờ vá thuộc tính này và engine
            không bao giờ dựng lại editing host.

            ⚠️ Văn bản render `s.target_text` — bản **LÚC NẠP**. Văn bản đang gõ đi đường
            [`restoreEditedText`], KHÔNG qua một binding phản ứng. Xem khối *"DOM SỞ HỮU VĂN
            BẢN"* ở script.

            🔴 `tgt-para-end` — cờ kết đoạn của **BẢN DỊCH** (Story 2.5d, FR134/AD-46). Nó đọc
            `s.is_target_paragraph_end`, tức **dữ liệu đã lưu** (AC4) — **không** suy từ
            `is_paragraph_end` của cột nguyên văn, và **không** suy từ vị trí các `\n` trong
            `target_text`. Hai phép suy đó đều chạy ra kết quả trông đúng, và cả hai rẽ khỏi
            đĩa đúng vào ngày người dùng đổi cờ đầu tiên.
            ⚠️ Class đặt trên **từng ô**, vì một hàng KHÔNG phải một phần tử DOM — xem khối
            đầu tệp. Cái giá hình học của lựa chọn này đã đo, ghi ở khối style cuối tệp.

            🔵 CẬP NHẬT 2026-08-16 (code review) — `empty` đo bằng `.trim()`, KHÔNG `=== ''`.
            Trước lượt vá này lưới hỏi `=== ''` còn Rust hỏi `target_text.trim().is_empty()`
            (`commands/segment.rs`, đường `confirm_segment`). Hai định nghĩa "rỗng" chỉ trùng
            nhau chừng nào ô KHÔNG chứa được khoảng trắng — và AC1 của Story 2.5d vừa cho nó
            chứa `\n`. Hệ quả đo được: bấm `Enter` trong một ô rỗng cho `"\n"`, ô MẤT viền đứt
            nên trông đã dịch, mà `confirm_segment` vẫn từ chối ký — người dùng không có cách
            nào biết vì sao. 🔴 Rust là nguồn sự thật cho "rỗng"; lưới đi theo, không ngược lại.
          -->
          <template
            v-for="(s, j) in block.segments"
            :key="s.id"
            v-memo="[
              s,
              block.start + j,
              selectedRowClassById.get(s.id),
              s.id === anchorId,
              rowErrorTextById.get(s.id),
              editorEditedShape.get(s.id),
              ruleOf(s.id),
            ]"
          >
          <!-- aura-allow-text: bản dịch của người dùng, dữ liệu. -->
          <div
            class="cell cell-tgt"
            :class="[
              {
                'para-end': s.is_paragraph_end,
                refused: rowErrorLabelById.has(s.id),
                empty: (editorEditedShape.get(s.id) ?? textShapeOf(s.target_text)) < 2,
                editing: caretId === s.id,
                omitted: s.is_omitted,
                'tgt-para-end': s.is_target_paragraph_end,
              },
              selectedRowClassById.get(s.id),
            ]"
            :data-segment-id="s.id"
            data-col="tgt"
            :data-unlinked="unlinkedIds.has(s.id) ? 'true' : undefined"
            contenteditable="true"
            role="textbox"
            aria-multiline="true"
            :aria-label="cellLabelOf(s.id, block.start + j)"
            :data-row-error="rowErrorTextById.get(s.id)"
            :aria-details="s.id === anchorId ? 'strip-track' : undefined"
          >{{ s.target_text }}</div>
          <div v-if="s.id === anchorId" class="strip-pad" aria-hidden="true"></div>
          </template>
        </div>

        <!-- Must follow the four columns in DOM order: it paints over and takes clicks in col-tgt only by coming later. -->
        <div
          v-if="hasStripTrack"
          id="strip-track"
          :ref="setStripTrackRef"
          class="strip-track"
          :class="{ 'strip-track-off': stripRow < 0 }"
          :style="stripRow < 0 ? undefined : { gridRow: stripRow + 2 }"
          @focusout="onStripFocusOut"
        >
          <TmFuzzyStrip />
          <GlossaryConfirmStrip />
          <ProofreaderStrip place="track" />
        </div>
        </div>
</template>

<style scoped>
.grid-block {
  display: grid;
  grid-template-columns: var(--grid-columns);
  align-items: stretch;
}

/*
 * 🔴 MỖI CỘT LÀ MỘT SUBGRID CHIẾM TRỌN TẬP TRACK HÀNG CỦA CHA (Quyết định #1(b)).
 *
 * `grid-row: 1 / -1` + `grid-template-rows: subgrid` là **cặp**, không phải hai dòng rời:
 * thiếu vế đầu thì cột chỉ chiếm một track, thiếu vế sau thì nó tự chia track riêng và hàng
 * lệch tích luỹ. Đo 2026-08-14: lệch `top` = **0 px** trên cả hai engine.
 *
 * ⚠️ Sàn: Safari 16+ *(macOS 12.4+)* · Chrome/Edge 117+. Ba engine bất đồng ở **gap** và
 * **auto-sizing** — lưới này cố ý **không dùng `gap`**, khoảng cách đi qua `padding` của ô.
 */
.col {
  display: grid;
  grid-template-rows: subgrid;
  grid-row: 1 / -1;
  min-width: 0;
}

.col-rule { grid-column: 1; }
.col-num { grid-column: 2; }
.col-src { grid-column: 3; }
.col-tgt { grid-column: 4; }

.strip-pad {
  min-height: 0;
}

.strip-track {
  grid-column: 4;
  min-width: 0;
  animation: strip-track-in 90ms ease-out;
}

.strip-track-off {
  display: none;
}

@keyframes strip-track-in {
  from {
    /* aura-allow-opacity: mốc đầu của quá độ 90 ms, không một trạng thái nghỉ. */
    opacity: 0.4;
  }
  to {
    opacity: 1;
  }
}

@media (prefers-reduced-motion: reduce) {
  .strip-track {
    animation: none;
  }
}

/*
 * 🔴 KIỂU DÁNG CẤP HÀNG PHẢI NHÂN RA NĂM Ô — hàng KHÔNG còn là một hộp.
 *
 * Đường kẻ dưới hàng nằm ở đây, trên **từng ô**, chứ không trên một `<tr>`. Bản dựng
 * `.working/editor-grid-two-column.html:208-229` viết chúng trên `<tr>`; **không chép thẳng
 * được**. Cùng luật cho `.para-end` bên dưới.
 */
.cell {
  padding: 4px 8px;
  border-bottom: 1px solid var(--color-outline-faint);
  min-width: 0;
}

/*
 * AC5 — KHOẢNG THỞ giữa các nhóm hàng, **không** một hàng rỗng.
 *
 * 🔴 Chỉ **ĐỌC** cờ `is_paragraph_end` đã lưu (AD-37, B3). Một dòng `text.split('\n')` lúc
 * render đi qua **mọi cổng** và làm hỏng FR121 ở Epic 8 — không cổng nào đỏ vì chuyện đó.
 * 🔴 **CẢNH BÁO NÀY NẶNG THÊM TỪ 2026-08-16 (Story 2.5d).** Trước đó `target_text` **không
 * bao giờ** chứa `\n`, nên một `split('\n')` lúc render là một đường sai *lý thuyết*. Nay
 * `\n` là một ký tự **hợp lệ và thường gặp** trong đó (AC1, FR134) — tức lượt suy ấy sẽ
 * **chạy ra kết quả**, và kết quả sẽ trông đúng cho tới ngày người dùng đổi cờ đích.
 * ⇒ Cấu trúc đoạn của **bản dịch** đọc từ `is_target_paragraph_end` (bước di trú 9, AC4),
 * **không** từ vị trí các `\n`. Hai khái niệm khác nhau: `\n` là xuống dòng **trong** một
 * câu, cờ là ranh giới đoạn **sau** câu.
 * ⚠️ Rà 2026-08-16 (Task 7.4): `split('\n')` · `split("\n")` · `lines()` trên `src/**` và
 * `src-tauri/src/**` ⇒ **0** đường suy. Con số ghi ra thay vì để im lặng đọc thành "đã rà".
 *
 * ⚠️ Một hàng rỗng bị loại có lý do đo được, không vì gu: nó sẽ là một hàng **không có
 * `segment.id`**, tức một hàng mà mọi phép đếm `[data-segment-id]` phải học cách bỏ qua — và
 * bộ e2e đếm đúng thứ đó. Một `padding-bottom` không sinh phần tử nào.
 */
.cell.para-end {
  padding-bottom: 16px;
  border-bottom-color: var(--color-outline);
}

/* Cột ① — vạch. `padding: 0` để vạch cao đúng bằng track hàng. */
.cell-rule {
  padding: 4px 0;
  border-bottom: none;
}

.cell-rule.cell-unlinked {
  border-right: 2px dotted var(--color-on-surface-variant);
}

/*
 * 🔴 Vạch lấy chiều cao từ **track hàng**, không từ `getClientRects()`.
 *
 * Đây là chỗ `editorGutter.ts` *(273 dòng · 31 chỗ nhắc "làn")* mất lý do tồn tại: bài toán
 * *"nhiều câu trên cùng một dòng ⇒ vạch chồng nhau"* biến mất **theo cấu trúc** khi một câu
 * là một hàng. Phép đo của nó *(O(n²) 482,4/254,5/261,6 ms → quét đường 8,3/5,2/4,3 ms trên
 * 9.850 vạch)* sống tiếp trong `deferred-work.md` — mã bị gỡ, bằng chứng thì không.
 */
.margin-slot {
  height: 100%;
  border-left: 3px solid transparent;
}

.rule {
  border-radius: var(--radius-sm);
}

/*
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 NĂM MÀU VẠCH — MỖI GIÁ TRỊ MỘT KHỐI, VÀ ĐÓ LÀ ĐIỀU KIỆN ĐỂ CỔNG NHÌN THẤY GÌ
 * ═══════════════════════════════════════════════════════════════════════════════
 * Kiểm B của `check-tokens.mjs` đọc CSS, không đọc TypeScript. Bind màu qua `:style` từ một
 * hàm TS sẽ đi qua cổng mà không một dòng nào được soi. `check-commands.mjs` Kiểm I đối chiếu
 * năm khối này **hai chiều** với `SEGMENT_RULE_VALUES`.
 *
 * Every rule keeps the slot's 3px border width and differs only in style and colour: a width
 * change relayouts the whole subgrid on WebKit, once per caret move.
 *
 * ⚠️ Hai trong năm màu hôm nay **không đường nào tới được** trên dữ liệu thật — `tm-rule`
 * thuộc Epic 7 (`isTmFilled` còn là hằng `false`) và `ornament` thuộc Story 2.8.
 */
.rule.rule-confirmed {
  border-left: 3px double var(--color-confirmed);
}

.rule.rule-primary {
  border-left: 3px solid var(--color-primary);
}

.rule.rule-tm-rule {
  border-left: 3px dotted var(--color-tm-rule);
}

/*
 * 🔵 Token thứ **17**, thêm ở Story 2.5b (Quyết định #2(a), Ice ký 2026-08-14).
 *
 * ⚠️ `draft` mượn **đúng giá trị** mà token `--color-ornament` mang ở cả hai theme, nên nó là
 * một **cái tên mới cho một màu đã kiểm** — không một màu mới chưa ai đo, và **0 cặp mới**
 * cho `contrast.pairs`.
 *
 * 🔴 Vạch `ornament` (câu về hưu) đã RÚT khỏi `resolveSegmentRule`/`SEGMENT_RULE_VALUES`;
 * khối `.rule.rule-ornament` từng đứng ngay
 * dưới đây bị GỠ theo (một class ngoài `SEGMENT_RULE_VALUES` làm `check:commands` Kiểm I ①
 * đỏ). Token `--color-ornament` KHÔNG bị xoá — nó còn được dùng ở nơi khác
 * (`LookupPanel.vue`, `ImportPreviewOverlay.vue`, `ReadingMode.vue`, dấu điểm cắt ngay dưới).
 */
.rule.rule-draft {
  border-left: 3px dashed var(--color-draft);
}

/*
 * 🔴 DẤU ĐIỂM CẮT ĐANG CHỜ — Story 2.8, AC7, kênh thị giác của cơ chế tích luỹ.
 *
 * ⚠️ `inline-block` + `width` chứ không một ký tự: một ký tự thật *(`|`, `‸`)* đi vào
 * `Selection.toString()` của Auto-Lookup và vào mọi lượt sao chép của người dùng. Dấu này là
 * **giao diện**, không phải nguyên văn của Tác phẩm.
 *
 * ⚠️ Màu từ token `--color-ornament`, cùng token với vạch hàng về hưu — hai thứ nói cùng một
 * điều: *"đây là một chỗ đánh dấu của công cụ, không phải nội dung"*. `check:tokens` Kiểm B
 * cấm mọi màu viết thẳng, và Kiểm F cấm bóng đổ / gradient / lớp nổi — dấu này không dùng cái
 * nào trong ba.
 *
 * ⚠️ `vertical-align` neo theo chiều cao dòng chứ không theo baseline: cột nguyên văn dựng
 * bằng ba họ font khác nhau *(`read-cjk` và `source-latin`)* và baseline của chúng lệch nhau.
 */
/*
 * 🔵 **2026-08-17 (Ice yêu cầu) — CAO HƠN và ĐỔI MÀU: `ornament` → `primary`.**
 *
 * `ornament` là `#a9a196` (sáng) / `#6a6459` (tối) — một màu **nét trang trí**, cố ý mờ. Trên
 * nền `surface` nó đo được **2,44 / 2,64** *(đã ghi ở Quyết định #9(a) của 2.5b)*, tức mờ đến
 * mức `check:tokens` **cấm** nó làm màu chữ. Một dấu cắt là thứ người dùng phải **tìm thấy**,
 * không một đường viền nền.
 *
 * `primary` (`#2f5d63` / `#7fb3ba`) là màu nhấn của ứng dụng và nằm trong danh sách màu CHỮ
 * hợp lệ của `check-tokens.mjs:274` — tức nó đã qua cửa tương phản, dù dấu này không phải chữ.
 * Nó cũng đúng ngữ nghĩa: một điểm cắt đang chờ là **việc người dùng đang làm**, không một
 * trạng thái của câu.
 *
 * ⚠️ **Vì sao KHÔNG dùng `error`:** một chỗ cắt chưa thực hiện không phải một lỗi, và nhuộm
 * đỏ nó là dạy người dùng một mức khẩn cấp sẽ phải lạm phát ở story sau — cùng lý lẽ mà
 * `StatusBar.vue` đã ghi cho câu báo.
 *
 * 🔴 **`height` đo bằng `em` và `vertical-align: text-bottom` giữ nguyên — chiều cao HÀNG
 * không được đổi.** Lưới dùng `subgrid`, nên một phần tử inline cao hơn line box sẽ đẩy
 * **cả track hàng** và kéo theo ô bản dịch *(cái giá đó đã đo ở 2.5b: hàng Hán Việt song song
 * đẩy ô bản dịch lên 388px)*. `1,3em` nằm trong line box của cỡ chữ nguồn — đo lại nếu ai đổi
 * `--leading-*` của cột này.
 */
.cut-mark {
  display: inline-block;
  width: var(--cut-mark-width);
  height: var(--cut-mark-height);
  vertical-align: text-bottom;
  background-color: var(--color-primary);
}

/*
 * 🔴 DẤU THUẬT NGỮ GLOSSARY ĐÃ KHỚP — Story 3.4b, FR50/FR51.
 *
 * ⚠️ **Hai kiểu, MỘT màu** — cả hai đọc `--color-primary` (đã chốt tô CHỮ; chờ chốt gạch chân
 * CHỮ), đúng UX *"màu nhấn `primary` chỉ dành cho đúng ba việc, đánh dấu thuật ngữ đã chốt là
 * việc thứ nhất"* (`epic-3-context.md`). `check-tokens` Kiểm B cấm màu viết thẳng — cả hai đều
 * qua `var(--color-*)`.
 *
 * 🔴 **KHÔNG `opacity`, KHÔNG bóng đổ/gradient/lớp nổi, `z-index` KHÔNG chạm.** Chữ ký của Ice
 * 2026-08-21 (`3-4b-…md` §Boundaries): kiểu gạch chân là kênh phân biệt DUY NHẤT giữa đã
 * chốt/chờ chốt — không mờ đi bằng `opacity` (`check-tokens` Kiểm D cấm mọi giá trị khác 0/1
 * trung gian).
 *
 * ⚠️ `text-decoration-color`/`-thickness`/`-offset` KHÔNG mang màu literal nào ngoài
 * `var(--color-primary)` ⇒ đi qua Kiểm B tự do (`COMPOSITE_COLOR_PROPS` chỉ soi phần MÀU của
 * một giá trị ghép) — cùng lý lẽ đã ghi cho `.cell.omitted { text-decoration: line-through }`
 * ở dưới tệp này.
 */
.src-piece.glossary-confirmed {
  color: var(--color-primary);
}

.src-piece.concordance-source {
  background-color: var(--color-surface-accent);
}

.src-piece.glossary-pending {
  text-decoration-line: underline;
  text-decoration-color: var(--color-primary);
  text-decoration-thickness: 2px;
  text-underline-offset: 3px;
}

/*
 * 🔵 Story 6.14 (nợ ③) — NHÃN VAI. Chữ nhỏ, `on-surface-variant` (cùng vai "chữ thật" với số
 * câu/nhãn trạng thái — Quyết định #9(a) đã cân cho hai cột đó, dùng lại ở đây), đứng thành
 * một dòng riêng trước nguyên văn — `display: block` để nó không dính vào chữ đi ngay sau.
 */
.role-label {
  display: block;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  line-height: var(--leading-ui-label);
  color: var(--color-on-surface-variant);
}

/*
 * 🔵 Story 6.14 (FR42) — ảnh TRONG ô nguyên văn. `<figure>` không viền/nền riêng — nó chỉ là
 * một hộp gộp `<ChapterImage>` cho đúng ngữ nghĩa HTML (ảnh + chú thích tiềm năng), khoảng
 * cách với chữ đứng cạnh đi qua `margin`. `max-width: 100%` lặp lại `<img>` của
 * `ChapterImage.vue` — phòng khi `<figure>` tự thêm `min-width` mặc định của UA.
 */
.grid-image {
  margin: 4px 0;
  max-width: 100%;
}

/*
 * 🔵 **2026-08-17 (Ice chốt) — VIỀN `has-cuts` ĐÃ GỠ, cùng với chính lớp đó.**
 *
 * Nó là `border-left: 2px solid var(--color-ornament)` trên ô nguyên văn, và nó tồn tại vì
 * một lý do **đã hết đúng**: ở chế độ Hán Việt dấu cắt *"không vẽ được"*, nên ô cần một kênh
 * nói *"câu này đang có điểm chờ"* mà không nói **ở đâu**. Story 2.9 · AC9 làm dấu cắt vẽ
 * được ở **cả hai** kiểu xem ⇒ kênh thay thế hết việc.
 *
 * 🔴 Ice dùng thật rồi chốt: *"bỏ dấu gạch đứng ở trước câu đi, nó không cần thiết"*. Hai
 * kênh cho một khái niệm bắt người đọc học hai ký hiệu, và cái mờ hơn thì nói ít hơn.
 *
 * ⚠️ Lớp `has-cuts` **gỡ luôn**, không giữ lại làm mối nối cho test: một lớp không tạo kiểu
 * gì là mã chết, và giữ nó chỉ để `e2e` chọn được là mở đúng tiền lệ mà kho này cố ý chưa mở
 * *(`e2e/support/workspace.mjs` §"không thêm mối nối `data-` chỉ để test chọn được")*.
 * `data-cut-count` ở lại — nó chở **số**, tức nhiều hơn một cờ, và hai spec đọc nó.
 *
 * ⚠️ Gỡ viền cũng xoá một lượt xê dịch 2px mà chỉ những ô có điểm cắt phải chịu.
 */

/*
 * Cột ② — số câu. `on-surface-variant` (Quyết định #9(a)); `DESIGN.md` frontmatter khai
 * `ornament` và **nó sai** — `DESIGN.md:208` cấm `ornament` làm màu chữ, `tokens.json:98-101`
 * cưỡng chế bằng `check:tokens`. Tài liệu tự mâu thuẫn, và một cổng đứng về một phía.
 */
.cell-num {
  text-align: right;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  line-height: var(--leading-ui-label);
  color: var(--color-on-surface-variant);
  font-variant-numeric: tabular-nums;
}

.tok-source-cjk {
  font-family: var(--face-source-cjk);
  font-size: var(--font-source-cjk);
  line-height: var(--leading-source-cjk);
  color: var(--color-on-surface);
}

.tok-source-latin {
  font-family: var(--face-source-latin);
  font-size: var(--font-source-latin);
  line-height: var(--leading-source-latin);
  color: var(--color-on-surface);
}

.tok-editor {
  font-family: var(--face-editor);
  font-size: var(--font-editor);
  line-height: var(--leading-editor);
  color: var(--color-on-surface);
}

/*
 * 🔴 AC3 — Ô TRỐNG CÓ **CHIỀU CAO THẬT** VÀ ĐƯỜNG **ĐỨT NÉT**.
 *
 * Đây là chỗ khuyết tật *"sập hố"* chết theo cấu trúc. Nó tới từ một `<span>` rỗng **cao 0 px**
 * — caret không có hộp nào để lấy chiều cao, và `Backspace` ở offset 0 của editing host duy
 * nhất không có chỗ nào để xoá lui vào.
 *
 * ⚠️ `1.95em` nhân với `font-size` của **chính ô** (token `editor`, 15px) ⇒ đúng **một dòng**.
 * Đừng đổi sang `rem`: gốc tài liệu là `ui`, không phải `editor`.
 * ⚠️ **Đo 2026-08-14, WKWebView 605.1.15:** ô rỗng **38,00 px** = ô có chữ **38,00 px**; bấm
 * chuột thật vào nó cho `type = "Caret"`, gõ một ký tự cho `beforeinput insertText` huỷ được
 * và chữ hạ cánh.
 *
 * 🔵 Và đây là lượt **hoàn nguyên có lý do** một bản vá cũ: Story 2.3 từng đặt
 * `.sent:empty { display: inline-block; min-width: 1ch }` để một câu chưa dịch có chỗ mà bấm.
 * Nó **không** chữa được ca đó và **đẻ ra** một khuyết tật thị giác *(mỗi câu chưa dịch chiếm
 * một khoảng trắng lạ ⇒ một Chương mới hiện ra thành một dải khoảng hở rời rạc)*. Lời giải
 * đúng nằm ở **cấu trúc**, không ở một lượt vá hình học: một Ô có chiều cao, không một span.
 */
/*
 * 🔴 `white-space: pre-line` — TIỀN ĐỀ VẬN HÀNH của FR134/AD-46, không một dòng trang trí.
 * Story 2.5d, Quyết định #2 đường (b) (Ice ký 2026-08-15). **Đừng đổi mà không đọc hết.**
 *
 * Nó làm HAI việc, và cả hai đều đo được trên WKWebView 605.1.15 thật
 * (`2-5d-ban-do/README.md`, 2026-08-15):
 *
 * ① **Vẽ ra `\n` đã lưu.** Ô này render `{{ s.target_text }}`, tức một chuỗi đi thẳng vào
 *    DOM thành text node. Đo được: cùng một `"A\nB"`, `white-space: normal` cho **1 dòng**,
 *    `pre-line` cho **2**. Không có dòng này, một Chương vừa mở hiện bản dịch **mất hết
 *    ngắt đoạn** — đĩa đúng, màn hình sai, không lỗi nào được ném.
 *
 * ② **Quyết định hình dạng DOM mà engine dựng lúc gõ.** Đo được: `execCommand(
 *    'insertLineBreak')` trong ô này dựng một **text node `"\n"`** *(0 phần tử con,
 *    `textContent === "A\nB"`)*; đối chứng **không** `pre-line` thì chính lệnh đó dựng
 *    `<br>` và `textContent` đọc ra **`"AB"`** — mất trắng ranh giới trên đường ghi.
 *    ⇒ Đổi giá trị này là **lật nhánh ① của `onBeforeInput`** ở một tệp khác, trong im lặng.
 *
 * ⚠️ `pre-line` chứ **không** `pre-wrap` (Ice ký): `pre-wrap` giữ cả khoảng trắng đầu/cuối
 * dòng, tức mọi bản dịch cũ trên đĩa có khoảng trắng thừa sẽ **đổi hình dạng hiển thị** ngay
 * lượt cập nhật này. Hai giá trị cho **cùng** số dòng (đo: 2/2) nên vế ① không mất gì.
 * ⚠️ `check:tokens` **không** soi `white-space` (`PURE_COLOR_PROPS`/`COMPOSITE_COLOR_PROPS`)
 * ⇒ viết thẳng ở đây là hợp lệ, không cần token, không cần miễn trừ có tên.
 * ⚠️ Cột nguyên văn **không** nhận dòng này — story không có AC nào đòi đụng nó, và
 * `SourceHanViet.vue` đã có `pre-wrap` riêng của nó.
 */
.cell-tgt {
  white-space: pre-line;
}

.cell-tgt :deep(.proofread-error),
.cell-tgt :deep(.proofread-meaning) {
  text-decoration-line: underline;
  text-decoration-style: wavy;
  text-decoration-thickness: 1px;
  text-underline-offset: 4px;
}

.cell-tgt :deep(.proofread-error) {
  text-decoration-color: var(--color-error);
}

.cell-tgt :deep(.proofread-meaning) {
  text-decoration-color: var(--color-tm-text);
}

.cell-tgt.empty {
  min-height: 1.95em;
  border-bottom-style: dashed;
  border-bottom-color: var(--color-outline);
}

/*
 * 🔴 Ô ĐANG GÕ KHÔNG VẼ VÒNG FOCUS CỦA TRÌNH DUYỆT — Ice bắt bằng mắt 2026-08-13.
 *
 * Triệu chứng ở hình dạng cũ: một khung viền bao quanh câu đang sửa, chiếm chỗ và đẩy dòng
 * văn lệch so với nguyên văn. Trong lưới cái giá còn cao hơn: một khung viền trên một ô sẽ
 * **đẩy chính hàng đó cao lên**, tức bố cục nhảy mỗi lần con trỏ đổi hàng.
 *
 * ⚠️ NFR17 **không** mất gì: chỉ báo tiêu điểm ở đây là một **caret nhấp nháy** cộng vạch lề
 * `primary` (UX-DR19) cộng nền `surface-accent` ngay dưới — ba kênh, mạnh hơn hẳn một đường
 * viền. Luật vẫn giữ nguyên cho mọi nút và ô nhập khác trong sản phẩm.
 */
.cell-tgt:focus {
  /* aura-allow-outline-none: ô đang gõ có CARET nhấp nháy làm chỉ báo tiêu điểm, cộng vạch lề `primary` (UX-DR19) và nền `surface-accent` — một vòng focus nữa là cùng một thông tin nói ba lần, và nó đẩy chính hàng đó cao lên. */
  outline: none;
}

/*
 * 🔴 NỀN HÀNG ĐANG SỬA — và đây là chỗ *"hàng không còn là một hộp"* phải trả giá.
 *
 * Nó tô trên **ô bản dịch**, không trên cả hàng, vì không có phần tử nào là "cả hàng". Tô đủ
 * năm ô đòi năm lớp `:class` song song và năm phép so `editorCaretSegmentId === s.id` — đắt
 * hơn hẳn thứ nó mua. ⇒ Phạm vi tối thiểu: đúng ô người dùng đang gõ.
 *
 * ⚠️ Ghi ra thay vì để người sau tưởng đã xét: nếu UX sau này đòi nền **cả hàng**, chỗ sửa là
 * năm `v-for` chứ không một dòng CSS.
 */
.cell-tgt.editing {
  background-color: var(--color-surface-accent);
}

/*
 * 🔵 **THÊM Story 4.9 — VÙNG CHỌN NHIỀU-SEGMENT** (Decision 1, batch input). §Phase 1 Task 2
 * cấm bằng chữ: "no new colour, no shadow, no floating layer" — tô CẢ NĂM ô của một hàng
 * bằng token `--color-surface-accent` ĐÃ CÓ, cùng sắc `.cell-tgt.editing` dùng cho hàng đang
 * gõ. Đây là một lượt tái dùng CÓ CHỦ, cùng tiền lệ `.cell-tgt.tgt-para-end` tái dùng
 * `--color-primary`: hai lớp `.editing`/`.row-selected` có thể cùng có mặt trên một hàng
 * (hàng caret là `anchor` của vùng chọn) và tô ra CÙNG một sắc — không một xung đột.
 *
 * ⚠️ Khác `.editing` (chỉ tô ô bản dịch — "phạm vi tối thiểu" đã ghi ở khối trên): vùng chọn
 * phải tô CẢ HÀNG, vì đây chính là câu Story 4.9 viết ra: "a selection the user cannot see
 * is not a selection". Năm `v-for` đều nhận lớp này — không một chỗ nào bị bỏ sót.
 */
.cell.row-selected {
  background-color: var(--color-surface-accent);
}

/*
 * 🔴 CỜ KẾT ĐOẠN CỦA **BẢN DỊCH** — chỉ báo PHI HÌNH HỌC. Story 2.5d, FR134 · AD-46,
 * Quyết định #4 đường (Ⓑ), Ice ký 2026-08-16 **sau khi có số**, không trước.
 *
 * ⚠️ **VÌ SAO KHÔNG DÙNG LẠI KHUÔN `.cell.para-end` CỦA CỜ NGUỒN — một phép đo, không một gu.**
 * Năm cột là năm `subgrid` chia **chung một tập track hàng** và `.cell` mặc định
 * `align-self: stretch`, nên **track = max(chiều cao các ô cùng hàng)**. Đo trên WKWebView
 * 605.1.15 thật (bàn đo vòng 5, 2026-08-15), cùng một hàng, nền **38,00 px**:
 *
 *   | hình dạng | track hàng |
 *   |---|---|
 *   | `padding-bottom: 14px` **chỉ** ở ô bản dịch | **46,00 px** — và ô nguyên văn **cũng** 46 |
 *   | đổi kiểu đường kẻ đáy ở ô bản dịch | **38,00 px** |
 *   | một ký tự ở cột nhãn trạng thái | **38,00 px** |
 *
 * ⇒ Một *"khoảng thở"* đặt riêng ở ô bản dịch **kéo cả năm ô giãn theo**, tức nó nói dối:
 * người đọc thấy cả hàng thở ra và tưởng **bản gốc** cũng kết đoạn ở đó. **Hai cấu trúc
 * đoạn khác nhau không biểu diễn được bằng hai khoảng thở trong cùng một lưới.**
 *
 * 🔴 **`border-bottom-color`, KHÔNG `border-bottom-width`.** Đường viền cộng vào chiều cao
 * hộp, nên một lượt làm dày là đúng cái bẫy 46 px ở trên bằng một cái tên khác. Màu đổi,
 * bề dày **giữ nguyên 1px**.
 *
 * ⚠️ **KHÔNG** thêm giá trị vào `SEGMENT_RULE_VALUES` và **KHÔNG** dựng một khối `.rule-<x>`:
 * Kiểm I (`check-commands.mjs`) đối chiếu **ba chiều và cả chiều ngược lại**, nên một
 * `.rule-<x>` lạ trong CSS làm cổng FAIL. Đây là một class của **ô**, không một giá trị vạch.
 * ⚠️ **KHÔNG** `opacity` trung gian (Kiểm D) và **KHÔNG** `ornament` làm màu chữ — đây là màu
 * **đường viền**, không màu chữ, nên sàn AA của Kiểm C không áp; `--color-primary` đã là token
 * của vạch lề, dùng lại nó giữ *"ranh giới có nghĩa"* nói bằng **một** thứ tiếng.
 *
 * ⚠️ **GIỚI HẠN THẬT:** ô **chưa dịch** đã dùng `border-bottom-style: dashed`. Một ô vừa chưa
 * dịch vừa kết đoạn sẽ hiện **nét đứt màu primary** — đọc được, nhưng đó là một tổ hợp chưa
 * ai vẽ trong `DESIGN.md`. Ghi ra thay vì để người sau tưởng đã được xét.
 *
 * 🔵 **2026-08-16 (code review) — lượt tái dùng token này ĐÃ ĐƯỢC XÉT, Ice ký GIỮ.** Lượt rà
 * nêu đúng một chỗ: `--color-primary` đang mang nghĩa *"hàng đang có con trỏ"* ở cột vạch
 * (`.rule.rule-primary`), nên hai nghĩa khác nhau nay dùng chung một sắc **trong cùng một
 * khung nhìn**, và người đã học *"primary = hàng đang hoạt động"* có thể đọc nhầm viền này
 * thành *"hàng tôi vừa bấm"*. Ice giữ nguyên, vì hai chỗ khác **hình dạng** (nền vạch đặc ở
 * cột ① so với một đường viền 1px ở đáy ô) và cùng nghĩa rộng *"đang bật"*.
 * ⇒ 🔴 Đây là một lượt tái dùng **có chủ ý**, không một lượt với tay lấy token gần nhất. Lượt
 * rà sau đừng hỏi lại; nếu muốn lật thì phải mang một phép đo về **nhầm lẫn thật**, không một
 * lý lẽ về sự đồng nhất — và một token thứ hai là một quyết định thiết kế phải viết vào
 * `DESIGN.md`, không một dòng CSS.
 */
.cell-tgt.tgt-para-end {
  border-bottom-color: var(--color-primary);
}

/*
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 HÀNG ĐÃ CẮT BỎ KHỎI BẢN DỊCH — AC3 của Story 2.5c (FR133)
 * ═══════════════════════════════════════════════════════════════════════════════
 *
 * *"Hàng **vẫn nằm trong lưới**, gạch ngang và mờ đi"* — người dùng phải **thấy mình đã bỏ
 * gì**. Một hàng biến mất khỏi lưới là đúng thứ AC3 cấm: quyết định *"đoạn này không thuộc
 * bản dịch"* biến thành một lỗ hổng im lặng.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 `on-surface-variant`, KHÔNG `ornament` — Quyết định #6, Ice ký 2026-08-15
 * ─────────────────────────────────────────────────────────────────────────────
 * `DESIGN.md:148` khai `grid-row-omitted: { color: ornament, decoration: line-through }`, và
 * vế **màu** của dòng đó **không thi hành được**: `ornament` nằm trong
 * `contrast.neverTextTokens` của `tokens.json` với câu *"KHÔNG một ngoại lệ nào — token này
 * không bao giờ là màu chữ"*, và `check-tokens.mjs:1300-1334` cưỡng chế nó.
 *
 * **Đo 2026-08-15 trên nền `surface`** *(sàn AA = 4,5)*:
 *   `ornament` **2,44** (sáng) / **2,64** (tối) — trượt · `on-surface-variant` **5,60** /
 *   **5,56** — đạt.
 *
 * ⚠️ Và đây **không** phải một chỗ đặc tả bị bỏ qua: Quyết định #9(a) của Story 2.5b đã giải
 * **đúng bài toán này** một lần — cột số câu và cột nhãn trạng thái cũng được `DESIGN.md`
 * khai `ornament`, và Ice đổi cả hai sang `on-surface-variant` vì *"đây là **chữ thật**,
 * không phải nét"*. Ô đã cắt bỏ cũng là chữ thật, nên nó đi cùng đường.
 *
 * 🔴 Hai đường tắt đã bị từ chối, ghi ra để không ai thử lại: **①** thêm
 * `aura-allow-never-text: ornament` — tái mở đúng bảng miễn trừ mà lượt ra mã 2.5b vừa dọn
 * rỗng, và đặt chữ 2,44:1 lên màn hình (NFR17); **②** `opacity` trung gian — `DESIGN.md:230`
 * cấm *(`opacity` ở trạng thái nghỉ chỉ áp cho nét và nền, không áp cho chữ)* và Kiểm D của
 * `check-tokens` đỏ với mọi giá trị khác `0`/`1`.
 *
 * ⚠️ `text-decoration: line-through` đi qua Kiểm B tự do: `text-decoration` nằm trong
 * `COMPOSITE_COLOR_PROPS` nên cổng chỉ soi **phần màu** của giá trị ghép, và `line-through`
 * không mang màu nào.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * ⚠️ BỐN CỘT, KHÔNG NĂM — và cột bị bỏ ra là một quyết định
 * ─────────────────────────────────────────────────────────────────────────────
 * Không có phần tử nào là "một hàng" (xem khối đầu tệp), nên kiểu dáng cấp hàng phải nhân ra
 * **từng ô** — đúng khuôn `.para-end` đang lặp ở cả năm `v-for`. Ở đây chỉ **bốn** cột chữ
 * nhận `omitted`; **cột ① (vạch trạng thái) KHÔNG**.
 *
 * 🔴 Lý do là AC2: vạch chở `status`, và *"câu **vẫn giữ** trạng thái riêng của nó"*. Tô vạch
 * thành `on-surface-variant` là xoá đúng thông tin mà AC2 đòi giữ — người dùng sẽ không còn
 * đọc được câu mình vừa cắt bỏ đã ký hay chưa, tức không biết mình đang bỏ đi cái gì. Và một
 * vạch 2px thì `line-through` không nói gì cả.
 */
.cell.omitted {
  color: var(--color-on-surface-variant);
  text-decoration: line-through;
}

/*
 * AC14 — nhãn lỗi của hàng nằm dưới chữ trong ô bản dịch. Vẽ bằng pseudo-element để không
 * vào `textContent`, nơi đường ghi đọc văn bản đang gõ.
 */
.cell-tgt[data-row-error]::after {
  content: attr(data-row-error);
  display: block;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  line-height: var(--leading-ui-label);
  color: var(--color-error);
  user-select: none;
}
</style>
