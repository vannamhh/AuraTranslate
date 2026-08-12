<script setup lang="ts">
// Panel `Bản dịch` — **trang liền mạch**. Story 2.2 · AC1 · AC2 · AC3 · AC4 · AC5 · AC6 · AC7.
//
// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 BỀ MẶT NÀY **KHÔNG GÕ ĐƯỢC**, VÀ ĐÓ LÀ MỘT MỆNH ĐỀ NGHIỆM THU — Story 2.2 · AC18
// ═════════════════════════════════════════════════════════════════════════════════
// Ice chốt Quyết định #1 ngày 2026-08-12, đường **(b)**: story này thêm cột `target_text`
// (bước di trú 6) và dựng một bề mặt **CHỈ-ĐỌC**. Vùng gõ tự do — cùng hợp đồng flush của
// AD-35, cờ "chưa lưu", và món nợ `isTypingZone` (`deferred-work.md:180`) — là **Story 2.3**.
//
// Lý do nói thẳng: gõ được hôm nay nghĩa là tồn tại một cửa sổ giữa 2.2 và 2.3 mà người dùng
// gõ rồi **mất trắng khi đóng app**, không một dấu hiệu nào. Đó đúng là lớp khuyết tật mà cả
// Epic 2 tồn tại để chống (NFR18 — *"mất tối đa 5 giây"*), nên nó không được lên nhánh chính
// dù chỉ một story.
//
// ⇒ **KHÔNG** `contenteditable`. **KHÔNG** `<textarea>`/`<input>`. **KHÔNG** `v-model`. **KHÔNG**
// handler sửa văn bản. Kiểm J của `scripts/check-commands.mjs` cưỡng chế **đúng năm** mệnh đề đó
// — `contenteditable` · `<textarea>` · `<input>` · `v-model` · `@input`/`@beforeinput`/`@paste`/
// `@cut` — và **đỏ** ngay khi một `contenteditable="true"` được tiêm vào template dưới đây.
//
// ⚠️ **`@keydown` KHÔNG nằm trong danh sách cấm, và đó là có chủ ý** *(làm rõ ở code review
// 2026-08-12, sau khi khối này từng khai rộng hơn thứ cổng canh được)*. Một `@keydown` tự nó
// không sửa được văn bản: không `contenteditable`/`<textarea>`/`<input>` thì không có chỗ nào
// cho ký tự hạ cánh. Và **Story 2.10** *(điều hướng segment)* cần đúng handler đó cho phím mũi
// tên — cấm nó ở đây là dựng một cổng mà story sau buộc phải gỡ vì lý do sai. Luật *"không thao
// tác sửa văn bản trong một `@keydown`"* do **người giữ**, không do máy; máy giữ năm mệnh đề trên.
//
// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 HỆ QUẢ Ice ĐÃ ĐƯỢC BÁO TRƯỚC VÀ VẪN CHỌN
// ═════════════════════════════════════════════════════════════════════════════════
// Trên dữ liệu thật hôm nay, **mọi** `target_text` là chuỗi rỗng và chưa đường nào điền
// chúng ⇒ panel này là **một trang trắng có máng lề trống**. Đó là trạng thái **ĐÚNG** *(mọi
// câu "chưa dịch" ⇒ "không vạch", đúng AC3)*, nhưng nó không nhìn ra được là đã chạy hay đã
// hỏng nếu chỉ mở app — nên có một dòng nói thẳng điều đó (`panel.editor.nothing_translated`),
// và bằng chứng thị giác của story nằm ở **fixture** của Task 7.
//
// ⚠️ Vỏ `PanelFrame` đã lo hợp đồng tiêu điểm (AC7): `declareFocus(owner, () => root.value)`
// chạy từ Story 1.6, và `'panel.editor'` đã có trong `FOCUS_OWNERS`. **Đừng** dựng vạch tiêu
// điểm panel thứ hai — `.panel.focused::before` đã có, và nó là vạch của **panel** (UX-DR8),
// khác hẳn vạch của **segment** (UX-DR19) mà tệp này dựng.
import { computed, onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue'
import PanelFrame from './PanelFrame.vue'
import { useSelectionSurface } from './selectionContract'
import type { DockviewPanelProps } from '../layout/panelProps'
import { t } from '../i18n'
import {
  createRuleScheduler,
  measureGutterRules,
  SEGMENT_ID_ATTR,
  type GutterRule,
} from './editorGutter'
import { resolveSegmentRule, ruleClassOf, segmentRuleInputOf } from './editorSegments'
import {
  editorCaretSegmentId,
  editorChapterId,
  editorHasLoaded,
  editorLoadError,
  editorPending,
  editorSegments,
  ensureSegmentsLoaded,
  setEditorCaret,
} from './editorPanelState'

defineProps<DockviewPanelProps>()

const surface = useTemplateRef<HTMLElement>('surface')
useSelectionSurface(surface, 'source')

const gutter = useTemplateRef<HTMLElement>('gutter')
const doc = useTemplateRef<HTMLElement>('doc')

onMounted(() => {
  // Idempotent — cùng khuôn `SourcePanel.vue::onMounted`. Gọi lại ở mỗi lượt mount (kể cả
  // sau một lượt đổi preset) là AN TOÀN và KHÔNG chạy lại IPC.
  void ensureSegmentsLoaded()
})

// ═════════════════════════════════════════════════════════════════════════════════
// Trạng thái màn hình — bốn ca, và một danh sách rỗng KHÔNG được nói thay cho ba ca kia
// ═════════════════════════════════════════════════════════════════════════════════
const loadErrorKey = computed(() => editorLoadError.value?.message_key ?? null)
const hasSegments = computed(() => editorSegments.value.length > 0)
/** Đã nạp xong, Chương có thật, nhưng **chưa ai bấm lệnh tách** — 25 Chương của Epic 1. */
const showNoSegments = computed(
  () => loadErrorKey.value === null && editorHasLoaded() && !hasSegments.value,
)
/**
 * 🔴 **MỘT LUẬT HIỂN THỊ NGOÀI BẢY AC — ghi ra để nó lật được bằng một dòng.**
 *
 * Bảy AC không nói gì về ca *"đã tách câu, nhưng chưa câu nào có bản dịch"* — vì trước
 * Quyết định #1 nó không tồn tại. Sau phán quyết đường (b) nó là trạng thái **thường trực**
 * của mọi Tác phẩm cho tới Story 2.3, và UX-DR27 nói thẳng cái giá: *"một khung trống câm
 * là thứ người dùng đọc thành hỏng"*. `SourcePanel.vue` đã đặt tiền lệ đúng ca này
 * (`panel.source.empty_chapter`).
 *
 * Dòng chú thích hiện **phía trên** trang văn, không thay nó — AC2/AC4/AC5 vẫn nghiệm thu
 * được trên fixture của Task 7 mà không phải gỡ dòng này ra.
 * **Chủ: Ice** *(một luật ngoài đơn hàng — chỗ lật là chính `v-if` dưới đây)*.
 */
const nothingTranslated = computed(
  () => hasSegments.value && editorSegments.value.every((s) => s.target_text === ''),
)
/**
 * Câu trạng thái mặc định của `PanelFrame` (*"Chưa có Chương nào để dịch."*) chỉ đúng khi
 * **không** có gì để nói — và *"đang chờ IPC"* **là** một thứ để nói.
 *
 * 🔴 `!editorPending` là vế bắt ở code review 2026-08-12. Thiếu nó, khoảng chờ của lượt nạp
 * đọc ra `true` *(chưa lỗi, chưa segment, `editorHasLoaded()` còn `false` nên `showNoSegments`
 * cũng `false`)*, và panel khẳng định **dứt khoát** rằng không có Chương nào — trong lúc
 * Chương đang trên đường về. Đó đúng là lớp lỗi mà doc-comment của [`editorHasLoaded`] tồn
 * tại để chặn *(`editorPanelState.ts:67-75`)*; nó chỉ lọt qua bằng một chuỗi khác.
 *
 * ⚠️ Trong khoảng chờ, panel hiện **trống** — không một câu nào. Đó là có chủ ý: một dòng
 * *"đang tải"* cho một lượt đọc SQLite cục bộ là thứ nháy lên rồi tắt, và UX-DR27 chỉ cấm
 * khung trống **thường trực**, không cấm một khoảng chờ.
 */
const showFrameStatus = computed(
  () =>
    loadErrorKey.value === null &&
    !editorPending.value &&
    !hasSegments.value &&
    !showNoSegments.value,
)

// ═════════════════════════════════════════════════════════════════════════════════
// AC3 · AC12 — năm giá trị vạch, qua ĐÚNG MỘT hàm phân giải (`./editorSegments.ts`)
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * Lớp CSS của vạch từng câu — `null` ⇒ *không vạch*, tức **không phần tử nào** được dựng.
 *
 * ⚠️ Một `Map`, không một phép tìm trong `v-for`: bảng này được tra một lần cho mỗi vạch
 * đang vẽ, và một `Array.find` ở đó cho ra O(n²) trên **9.850** câu.
 */
const ruleClassById = computed(() => {
  const caret = editorCaretSegmentId.value
  const map = new Map<number, string>()
  for (const s of editorSegments.value) {
    const cls = ruleClassOf(resolveSegmentRule(segmentRuleInputOf(s, caret)))
    if (cls !== null) map.set(s.id, cls)
  }
  return map
})

// ═════════════════════════════════════════════════════════════════════════════════
// AC2 · AC14 — chiều cao vạch ĐO từ hình học thật (Quyết định #2)
// ═════════════════════════════════════════════════════════════════════════════════
const rules = ref<GutterRule[]>([])

const remeasure = (): void => {
  const g = gutter.value
  const d = doc.value
  if (g === null || d === null) {
    rules.value = []
    return
  }
  rules.value = measureGutterRules(g, d, new Set(ruleClassById.value.keys()))
}

const scheduler = createRuleScheduler(remeasure)

/**
 * 🔴 BỐN NGUỒN KÍCH HOẠT, và cái thứ tư đi **qua** cái thứ nhất chứ không có listener riêng.
 *
 * ① đổi kích thước panel ⇒ `ResizeObserver` trên `.doc`;
 * ② font web nạp xong ⇒ `document.fonts.ready` *(ba font nhúng — UX-DR4; trước lượt đó hình
 *   học là hình học của font dự phòng, tức một phép đo đúng cho một màn hình sắp đổi)*;
 * ③ nội dung hay tập câu-có-vạch đổi ⇒ `watch` ngay dưới;
 * ④ **đổi theme** — không listener riêng, và đó là một phép đo chứ không phải một lượt bỏ
 *   sót: `applyTheme()` ghi typography từ `tokens.typography`, một bảng **không** phân theo
 *   theme *(chỉ `tokens.colors` phân theo theme)*, nên một lượt đổi theme không đổi được cỡ
 *   chữ, họ chữ hay giãn dòng — tức không đổi hình học. Và nếu một ngày nó có đổi, chiều cao
 *   `.doc` đổi theo ⇒ `ResizeObserver` của ① bắt được. Kho hôm nay cũng chưa có công tắc đổi
 *   theme lúc chạy: `applyTheme` chỉ được gọi một lần ở `main.ts`.
 */
let observer: ResizeObserver | null = null

onMounted(() => {
  // ⚠️ Không `?.` — `document.fonts` là **không-nullish** theo lib DOM, và `src/tokens/fonts.ts`
  // đã đứng trên đúng mệnh đề đó từ Story 1.4. Một `?.` phòng hờ ở đây là một nhánh mà kiểu
  // nói không bao giờ chạy, tức mã chết (`@typescript-eslint/no-unnecessary-condition` bắt
  // đúng nó lúc chạy `check:lint` 2026-08-12).
  void document.fonts.ready.then(() => {
    scheduler.schedule()
  })
  document.addEventListener('selectionchange', onSelectionChange)
})

watch(
  [() => editorSegments.value, ruleClassById, doc],
  () => {
    const d = doc.value
    observer?.disconnect()
    observer = null
    if (d !== null) {
      observer = new ResizeObserver(() => {
        scheduler.schedule()
      })
      observer.observe(d)
    }
    scheduler.schedule()
  },
  { flush: 'post' },
)

onBeforeUnmount(() => {
  // 🔴 Cả ba đều phải nhả. Một lượt đo đã hẹn mà chạy sau khi cây DOM đã tháo sẽ đọc hình
  // học của một phần tử mồ côi và ghi vào một `ref` không ai còn render.
  scheduler.cancel()
  observer?.disconnect()
  observer = null
  document.removeEventListener('selectionchange', onSelectionChange)
})

// ═════════════════════════════════════════════════════════════════════════════════
// AC5 — "tiêu điểm bàn phím chạm tới một câu", trên một bề mặt KHÔNG gõ được
// ═════════════════════════════════════════════════════════════════════════════════
/**
 * 🔴 Không có caret trên một bề mặt không `contenteditable` — nên *"con trỏ chạm tới"* đọc
 * từ **neo của vùng chọn DOM** (`Selection.anchorNode`).
 *
 * Đó là một cơ chế **có thật và đến được bằng bàn phím**, không phải một cách nói tránh:
 * Story 1.18 đã dựng đúng đường đó cho Panel Source *(`tabindex="0"` trên bề mặt chữ, rồi
 * `Shift+Mũi tên` — `deferred-work.md:608`)*, và một cú bấm chuột cũng đặt một vùng chọn
 * thu gọn. Cùng cơ chế ở đây, cùng lý do.
 *
 * ⚠️ Neo **ngoài** `.doc` ⇒ caret về `null`. Đó là câu đúng theo nghĩa đen: người dùng đang
 * chọn ở một panel khác thì *"con trỏ ở đây"* là sai, và giữ vạch `primary` sáng là để vạch
 * nói dối — đúng thứ doc-comment của `PanelFrame.vue::focused` tồn tại để chặn.
 */
function onSelectionChange(): void {
  const host = doc.value
  if (host === null) return

  // ⚠️ `window.getSelection`, KHÔNG `document.getSelection` — hai hàm cùng một API, nhưng
  //    `check-layout.mjs::ALLOWED_GLOBAL_MEMBERS` chỉ cho phép cái thứ nhất (Story 1.17 ·
  //    Quyết định #1a), và `selectionContract.ts` đã đi qua đúng cửa đó. Nới danh sách cho
  //    một bí danh là mở rộng bề mặt được phép để đổi lấy con số không.
  const anchor = window.getSelection()?.anchorNode ?? null
  if (anchor === null || !host.contains(anchor)) {
    setEditorCaret(null)
    return
  }

  const from = anchor instanceof Element ? anchor : anchor.parentElement
  const sent = from?.closest<HTMLElement>(`[${SEGMENT_ID_ATTR}]`) ?? null
  const raw = sent?.getAttribute(SEGMENT_ID_ATTR) ?? null
  const id = raw === null ? Number.NaN : Number(raw)
  setEditorCaret(Number.isFinite(id) ? id : null)
}

/** Rời tiêu điểm khỏi bề mặt ⇒ không câu nào *"đang sửa"* nữa. */
function onSurfaceFocusOut(event: FocusEvent): void {
  const next = event.relatedTarget
  if (next instanceof Node && doc.value?.contains(next)) return
  setEditorCaret(null)
}

/** Chỉ để đọc trong template — `editorChapterId` là dữ kiện chẩn đoán, không hiển thị. */
const chapterId = computed(() => editorChapterId.value)
</script>

<template>
  <PanelFrame owner="panel.editor" status-key="panel.editor.status" :show-status="showFrameStatus">
    <!-- Lỗi nạp nói ra bằng chuỗi CỦA NÓ, không im — cùng khuôn `SourcePanel.vue`. -->
    <p v-if="loadErrorKey !== null" class="load-error">{{ t(loadErrorKey) }}</p>
    <p v-else-if="showNoSegments" class="load-error">{{ t('panel.editor.no_segments') }}</p>

    <div ref="surface" class="editor-surface">
      <div v-if="hasSegments" class="edwrap" :data-chapter-id="chapterId">
        <!-- 🔴 Một luật hiển thị NGOÀI bảy AC — xem `nothingTranslated`. Chỗ lật là dòng này. -->
        <p v-if="nothingTranslated" class="untranslated-note">
          {{ t('panel.editor.nothing_translated') }}
        </p>

        <div class="edbody">
          <!--
            AC2 — MÁNG rộng `gutter-width`, vạch dọc 2px. `aria-hidden`: vạch lề là một lớp
            thông tin **thị giác** trùng lặp với trạng thái mà trình đọc màn hình sẽ đọc từ
            chính câu (Story 2.5 mang `status`); một chuỗi div rỗng không nhãn ở đây chỉ là
            tiếng ồn cho công nghệ trợ giúp.
          -->
          <div ref="gutter" class="gutter" aria-hidden="true">
            <div
              v-for="r in rules"
              :key="r.id"
              class="gmark"
              :class="ruleClassById.get(r.id)"
              :style="{ top: `${r.top}px`, height: `${r.height}px` }"
            ></div>
          </div>

          <!--
            🔴 AC1 — **MỘT** dòng văn liên tục. Không `display: block` cho câu, không grid,
            không bảng, không ô. Mỗi câu là một `<span>` chảy inline; chỗ ngắt đoạn tới từ cờ
            `is_paragraph_end` **đã lưu** (AD-37 cấm suy ra lúc render), và nó là một `<br>` —
            một dấu ngắt dòng, không phải một hộp.

            🔴 `tabindex="0"` — cùng lý do và cùng cái giá mà Story 1.18 đã ghi cho
            `SourcePanel.vue::.original`: một `<div>` không sửa được KHÔNG nhận `Shift+Mũi tên`
            nếu nó không vào được vòng tiêu điểm. Đây là thứ làm vế **bàn phím** của AC5 có
            thật, và làm hợp đồng vùng chọn (Story 1.18) thật sự chạy được trên bề mặt này.
            ⚠️ Nó KHÔNG làm bề mặt gõ được — xem khối AC18 ở đầu tệp.
          -->
          <div ref="doc" class="doc tok-editor" tabindex="0" @focusout="onSurfaceFocusOut">
            <template v-for="s in editorSegments" :key="s.id">
              <!-- aura-allow-text: BẢN DỊCH của người dùng (`segment.target_text`) — dữ liệu,
                   không phải chuỗi giao diện của `vi.json` (NFR16). Không `v-html` (AD-16). -->
              <span
                class="sent"
                :data-segment-id="s.id"
                :data-caret="editorCaretSegmentId === s.id ? '' : null"
              >{{ s.target_text }}</span>
              <br v-if="s.is_paragraph_end" />
            </template>
          </div>
        </div>
      </div>
    </div>
  </PanelFrame>
</template>

<style scoped>
.editor-surface {
  height: 100%;
  min-height: 0;
}

/*
 * Lỗi nạp và ca "Chương chưa được tách câu" — cùng token và cùng lý do với
 * `SourcePanel.vue::.load-error` (Story 1.17 · Quyết định #7: `ui-md-wrap`, giãn dòng 1.66).
 */
.load-error {
  margin: 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.untranslated-note {
  margin: 0 0 var(--space-panel-block) 0;
  flex: none;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-on-surface-variant);
}

.edwrap {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

/*
 * 🔴 HỘP CUỘN LÀ **CHỖ NÀY**, không phải `.doc`.
 *
 * Máng và trang văn phải cuộn CÙNG NHAU: vạch lề được đặt `position: absolute` trong hệ toạ
 * độ của máng, và `measureGutterRules` tính `top` bằng hiệu hai `getBoundingClientRect()`.
 * Cho `.doc` cuộn một mình sẽ làm hiệu đó đổi theo lượt cuộn ⇒ vạch trôi khỏi câu của nó ngay
 * ở dòng thứ hai của một Chương dài.
 *
 * ⚠️ `align-items` để mặc định (`stretch`): trong một hộp cuộn, con của flex row căng theo
 * chiều cao **nội dung**, nên máng luôn cao đúng bằng trang văn — điều kiện để một vạch ở câu
 * cuối cùng có chỗ mà nằm.
 */
.edbody {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: auto;
}

/* AC2 · UX-DR3 — máng rộng ĐÚNG token, không một con số viết thẳng. */
.gutter {
  position: relative;
  flex: none;
  width: var(--space-gutter-width);
  padding-top: 4px;
}

/*
 * AC2 — vạch dọc 2px, thụt trái 8px, bo `sm`.
 *
 * ⚠️ `border-radius` đọc token `--radius-sm` (**2px**), KHÔNG phải `1px` của mockup:
 * `DESIGN.md` component token `segment-gutter-rule` ghi `radius: sm`, và `EXPERIENCE.md:312`
 * phân xử sẵn rằng khi bản dựng mâu thuẫn với tài liệu thì tài liệu thắng.
 *
 * 🔴 KHÔNG `box-shadow` ở đây và không ở đâu trong tệp này — Kiểm F cấm **tuyệt đối**, không
 * miễn trừ. Mẫu `box-shadow: 0 0 0 3px <cùng màu nền>` của mockup là một kỹ thuật đệm chứ
 * không phải bóng đổ, nhưng cổng đọc **thuộc tính** chứ không đọc ý định, và sự bất đối xứng
 * đó là có chủ ý (`check-tokens.mjs:1364-1368`).
 */
.gmark {
  position: absolute;
  left: 8px;
  width: 2px;
  border-radius: var(--radius-sm);
}

/*
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 BỐN MÀU VẠCH — MỖI GIÁ TRỊ MỘT KHỐI, VÀ ĐÓ LÀ ĐIỀU KIỆN ĐỂ CỔNG NHÌN THẤY GÌ
 * ═══════════════════════════════════════════════════════════════════════════════
 * Kiểm B của `check-tokens.mjs` đọc CSS, không đọc TypeScript. Bind màu qua `:style` từ một
 * hàm TS sẽ đi qua cổng mà không một dòng nào được soi. Bốn khối dưới đây là bản khai mà cổng
 * đọc được từng chữ, và `check-commands.mjs` đối chiếu chúng **hai chiều** với
 * `SEGMENT_RULE_VALUES`.
 *
 * ⚠️ Giá trị thứ năm — *không vạch* — **không có khối CSS**, vì nó không vẽ gì: `ruleClassOf`
 * trả `null` và `v-for` không dựng phần tử nào. Bốn khối + một sự vắng mặt có chủ = năm.
 *
 * ⚠️ Ba trong bốn màu này hôm nay **không đường nào tới được** trên dữ liệu thật — nguồn dữ
 * liệu của chúng thuộc Story 2.5 (`confirmed`), Story 2.8 (`ornament`) và Epic 7 (`tm-rule`).
 * Xem `./editorSegments.ts` để có bảng chủ đầy đủ. Chúng nghiệm thu được trong **bàn đo**.
 */
.gmark.rule-confirmed {
  background-color: var(--color-confirmed);
}

.gmark.rule-primary {
  background-color: var(--color-primary);
}

.gmark.rule-tm-rule {
  background-color: var(--color-tm-rule);
}

/*
 * ⚠️ `ornament` ở đây là màu **NỀN của một vạch**, không phải màu chữ — Kiểm C chỉ đỏ với
 * `color:`/`-webkit-text-fill-color:`, và đó là ranh giới đúng: UX-DR5 nói `ornament` là màu
 * của **nét**. Chỗ duy nhất trong tệp này cần một miễn trừ là ký tự `⏐` ở dưới.
 */
.gmark.rule-ornament {
  background-color: var(--color-ornament);
}

/*
 * 🔴 AC6 — BỀ MẶT ĐỌC TỰ KHAI TOKEN CỦA CHÍNH NÓ.
 *
 * Đóng **nửa Editor** của `deferred-work.md:130-133`: mặc định kế thừa từ `body` là `ui-md`
 * ở giãn dòng **1.5** — dưới sàn cứng 1.66 — và Kiểm E chỉ đọc `tokens.json` nên hoàn toàn mù
 * với việc component nào đang kế thừa gì. Token `editor` là họ `read`, 15px, giãn dòng
 * **1.95** (`tokens.json:397-403`).
 *
 * ⚠️ `.doc` thụt trái 8px — `DESIGN.md`, cùng số với `inset-left` của vạch, nên chữ và vạch
 * đứng trên cùng một đường.
 */
.doc {
  flex: 1;
  min-width: 0;
  padding-left: 8px;
}

.tok-editor {
  font-family: var(--face-editor);
  font-size: var(--font-editor);
  line-height: var(--leading-editor);
  color: var(--color-on-surface);
}

/*
 * ⚠️ `position: relative` là điều kiện để `::after` neo vào chính câu. Nó KHÔNG dựng một
 * hộp: một `<span>` inline vẫn chảy inline dưới `position: relative`.
 */
.sent {
  position: relative;
}

/*
 * ═══════════════════════════════════════════════════════════════════════════════
 * 🔴 AC4 — RANH GIỚI CÂU `⏐` LÀ MỘT **PSEUDO-ELEMENT**, KHÔNG MỘT `<span>` THẬT
 * ═══════════════════════════════════════════════════════════════════════════════
 * Quyết định #3. Có một vết sẹo trực tiếp trong kho về đúng lớp lỗi này: Story 1.18b chèn
 * `WORD_JOINER` (`U+2060`) vào DOM Panel Source, và hệ quả là **rò ký tự lúc copy** trên
 * WKWebView — bôi đen bằng phím rồi `⌘C` dán ra chuỗi lẫn ký tự chèn, và `onCopy` phải dựng
 * lại chuỗi từ đường đọc DOM (`deferred-work.md:839-848`).
 *
 * Một `⏐` là `<span>` thật lặp lại đúng lỗi đó, lần này trên một bề mặt **sẽ gõ được ở Story
 * 2.3** — nên nó còn bị gõ đè, bị con trỏ đi xuyên qua, và bị đếm vào độ dài văn bản. Nội
 * dung của pseudo-element không nằm trong cây văn bản: không copy được, không chọn được,
 * không gõ đè được.
 *
 * ⚠️ CÁI GIÁ, ghi ra: pseudo-element không nhận `:hover` riêng *(nên `.sent:hover::after`)*,
 * và nó **không** hiện trong một bàn đo chép DOM thay vì chép CSS.
 *
 * ⚠️ Dấu miễn trừ ngay dưới viết trên **một dòng** có lý do đo được: `exemptAt` của
 * `check-tokens.mjs` so **dòng bắt đầu** của comment với dòng khai báo và đòi khoảng cách
 * ≤ 1. Một khối chú thích bốn dòng đặt ngay trên khai báo vẫn ĐỎ — đo lúc chạy cổng
 * 2026-08-12. Lý lẽ dài sống ở đây, dấu miễn trừ sống một dòng.
 *
 * Lý lẽ đầy đủ của miễn trừ `ornament`: UX-DR5 nói `ornament` và `tm-rule` không bao giờ là
 * màu của chữ, kèm *"ngoại lệ duy nhất đã đặc tả: ký tự ranh giới câu `⏐`"*; `tokens.json:99`
 * hẹn ngoại lệ đó cho **đúng story này**. `⏐` không mang nghĩa đọc được — nó là một đường kẻ
 * dọc vẽ bằng một code point, ở `opacity: 0` mặc định, và nó là NÉT chứ không phải chữ.
 */
.sent::after {
  content: '⏐';
  /* aura-allow-never-text: ornament — NÉT chứ không phải chữ; ngoại lệ `⏐` đã đặc tả ở UX-DR5 + tokens.json:99. */
  color: var(--color-ornament);
  opacity: 0;
}

/*
 * AC5 — hiện ở `0.55` khi rê chuột **hoặc** khi tiêu điểm chạm tới câu.
 *
 * ⚠️ **Cả hai theme cùng một số.** Mockup tối ghi `.75`; `DESIGN.md:382` chốt `0.55` không
 * phân theo theme, và `EXPERIENCE.md:312` phân xử rằng tài liệu thắng bản dựng.
 *
 * ⚠️ `[data-caret]` là vế **bàn phím** của AC5 — `:hover` không có vế đó. Cờ do tầng TS đặt
 * từ neo vùng chọn DOM, cùng nguồn với vạch `primary` của AC3.
 */
.sent:hover::after,
.sent[data-caret]::after {
  /* aura-allow-opacity: `⏐` là NÉT chứ không phải chữ — UX-DR6 cho phép `opacity` nghỉ trên nét và nền. */
  opacity: 0.55;
}
</style>
