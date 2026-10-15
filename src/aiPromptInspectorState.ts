/**
 * State của lớp phủ **Xem prompt cuối cùng đã gửi** — Story 4.7 (FR71, AD-14).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 HAI NHỊP, HAI HÀM — Decision 2 spec 4.7, và tệp này là nơi ranh giới đó SỐNG
 * ─────────────────────────────────────────────────────────────────────────────
 * [`assembleCurrentAiPrompt`] LẮP RÁP + GHI (gọi `ai_prompt_assemble`); [`openAiPromptInspector`]/
 * [`refreshAiPromptRecord`] chỉ ĐỌC (gọi `ai_prompt_read_record`). Không hàm nào ở tệp này gọi
 * hàm kia. Mở lớp phủ KHÔNG BAO GIỜ lắp ráp — nếu nó lắp, AC4 ("đúng chuỗi đã gửi, không phải
 * bản dựng lại") sai theo cấu tạo, dù `assemble_prompt` có thuần tới đâu (§Design Notes spec 4.7).
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 VÌ SAO CẢ HAI GHI VÀO CÙNG MỘT Ô `record` Ở ĐÂY
 * ─────────────────────────────────────────────────────────────────────────────
 * `assembleCurrentAiPrompt` ghi thẳng vào [`aiPromptRecord`] (không đợi người dùng MỞ lớp
 * phủ) để dòng tóm tắt ở `AiTranslationPanel.vue` ("Đã chèn N thuật ngữ Glossary") đổi ngay
 * sau lượt Lắp, không cần mở màn Xem prompt trước. Rust vẫn là nguồn sự thật DUY NHẤT —
 * [`openAiPromptInspector`] tự gọi lại [`refreshAiPromptRecord`] mỗi lần mở, nên một bản ghi
 * cũ trong ô nhớ này (ví dụ nếu Tác phẩm đã đóng ở nơi khác và Rust đã xoá bản ghi phía nó,
 * xem Judgment call của Phase 2 ở `commands/aiprompt.rs`) không bao giờ đứng yên lâu hơn một
 * lượt mở màn hình.
 *
 * ⚠️ Cùng luật mọi state Vue khác: tệp này KHÔNG được `import` vào `src/commands/index.ts`.
 */
import { computed, nextTick, readonly, ref } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import { aiPromptAssemble, aiPromptReadRecord } from './config/aiprompt'
import type { AssembledPromptWire } from './config/aiprompt'
import type { IpcError } from './i18n'

/** Hình dạng mà `aiPromptRecord.value` THẬT SỰ mang — `readonly()` của Vue làm sâu mọi mảng
 * lồng bên trong thành `readonly T[]`. Hai hàm thuần dưới đây nhận đúng hình dạng đó (không
 * phải `AssembledPromptWire` trần) để chỗ gọi (đọc trực tiếp từ ref `readonly`) không phải
 * ép kiểu — cùng lý do các trường đã lồng của `InjectionLedgerWire` không đổi hành vi khi
 * `readonly`, chỉ đổi KIỂU. */
type ReadonlyAssembledPromptWire = DeepReadonly<AssembledPromptWire>

const overlayOpen = ref(false)
const record = ref<AssembledPromptWire | null>(null)
const assembleBusy = ref(false)
const assembleError = ref<IpcError | null>(null)
/** Lỗi của lượt ĐỌC gần nhất — `null` sau một lượt Đọc thành công (kể cả "chưa có bản ghi
 * nào", I/O Matrix). Ô nhớ RIÊNG với [`assembleError`]: hai nhịp khác nhau (Decision 2), hai
 * lỗi khác nhau, cùng lý do `record`/`assembleBusy` đã tách. Xem [`refreshAiPromptRecord`]. */
const readError = ref<IpcError | null>(null)
/** Glossary count of the record before the current one, set only when that record belonged to another segment. */
const previousGlossaryCount = ref<number | null>(null)

export type AiPromptInspectorTab = 'result' | 'prompt'
const tab = ref<AiPromptInspectorTab>('prompt')
export type AiPromptCopyState = 'idle' | 'copied' | 'failed'
const copyState = ref<AiPromptCopyState>('idle')

/** Số thứ tự lượt ghi/đọc — chỉ lượt MỚI NHẤT được quyền ghi [`record`] (khuôn
 * `promptSetState.ts`/`aiConfigState.ts`). Một sequence DUY NHẤT cho cả Lắp và Đọc: hai
 * đường đều ghi vào CÙNG một ô, nên một lượt Đọc trả lời TRỄ không được phép ghi đè lên kết
 * quả của một lượt Lắp đã XẢY RA SAU nó, và ngược lại. */
let sequence = 0

export function selectAiPromptInspectorTab(next: AiPromptInspectorTab): void {
  tab.value = next
}

/** Closes, then runs `next` after the overlay's focus-return watcher, which would otherwise steal the focus `next` sets. */
export async function closeAiPromptInspectorThen(next: () => void): Promise<void> {
  overlayOpen.value = false
  await nextTick()
  next()
}

/** Copies `record.prompt` verbatim; a clipboard refusal becomes `'failed'`. */
export async function copyAiPromptToClipboard(): Promise<void> {
  const current = record.value
  if (current === null) {
    console.warn('[ai-prompt-inspector] khong chep: chua co ban ghi prompt nao')
    return
  }
  try {
    await navigator.clipboard.writeText(current.prompt)
    copyState.value = 'copied'
  } catch (err) {
    console.error(`[ai-prompt-inspector] clipboard tu choi: ${String(err)}`)
    copyState.value = 'failed'
  }
}

/**
 * ⚠️ **SỬA 2026-09-18 — cờ `assembleBusy` có thể TREO MÃI, bắt được ở lượt rà soát build.**
 *
 * `mine` của [`assembleCurrentAiPrompt`] so với `sequence` DÙNG CHUNG cho CẢ [`refreshAiPromptRecord`]
 * (không đụng `assembleBusy`) — nếu chỉ đọc `mine !== sequence` để quyết định có hạ
 * `assembleBusy` hay không, một lượt Đọc (mở lớp phủ, hoặc panel mount lại) chen vào TRƯỚC
 * khi lượt Lắp hiện tại trả lời sẽ nâng `sequence`, khiến nhánh "vượt mặt" bỏ qua luôn dòng
 * hạ cờ — không còn ai hạ nó nữa, nút "Lắp prompt cho câu này" bị khoá vĩnh viễn hết phiên.
 *
 * Ô nhớ RIÊNG này chỉ đổi ở [`assembleCurrentAiPrompt`], không đổi ở [`refreshAiPromptRecord`]
 * — nên một lượt Đọc chen vào không làm nó nhích, và lượt Lắp đang chạy vẫn nhận ra mình còn
 * là lượt Lắp MỚI NHẤT để tự hạ cờ. Ngược lại, khi một lượt Lắp MỚI HƠN thật sự chen vào
 * (khớp lại giá trị này), lượt Lắp CŨ biết mình không còn là lượt mới nhất và CHỦ ĐỘNG không
 * hạ cờ hộ — cờ vẫn đúng nghĩa "một lượt Lắp mới hơn đang chạy", đúng lý do dòng chú thích cũ
 * "kể cả cờ busy" từng cố giữ, chỉ khác đối tượng so sánh.
 */
let latestAssembleSequence = 0

function injectedCountOf(current: ReadonlyAssembledPromptWire | null): number | null {
  if (current === null) return null
  const status = current.ledger.glossary
  return status.kind === 'not_asked' ? null : status.injected.length
}

function adoptRecord(next: AssembledPromptWire | null, afterAssemble: boolean): void {
  const prev = record.value
  if (prev !== null && next !== null) {
    if (prev.segment_id !== next.segment_id) previousGlossaryCount.value = injectedCountOf(prev)
    else if (afterAssemble) previousGlossaryCount.value = null
  }
  record.value = next
}

export const aiPromptInspectorIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
/** Bản ghi hiện tại của phiên, hoặc `null` khi chưa có lượt Lắp nào — I/O Matrix "Nothing
 * recorded yet ... this is a state, not an error". */
export const aiPromptRecord: DeepReadonly<Ref<AssembledPromptWire | null>> = readonly(record)
export const aiPromptInspectorTab: DeepReadonly<Ref<AiPromptInspectorTab>> = readonly(tab)
export const aiPromptCopyState: DeepReadonly<Ref<AiPromptCopyState>> = readonly(copyState)
export const aiPromptAssembleBusy: DeepReadonly<Ref<boolean>> = readonly(assembleBusy)
/** Lỗi của lượt LẮP gần nhất — `null` sau một lượt thành công hoặc khi chưa lắp lần nào.
 * RIÊNG với một lỗi đọc: [`refreshAiPromptRecord`] không có đường lỗi (`ai_prompt_read_record`
 * không mang `Result` phía Rust, xem doc-comment `config/aiprompt.ts`). */
export const aiPromptAssembleError: DeepReadonly<Ref<IpcError | null>> = readonly(assembleError)
/** Lỗi của lượt ĐỌC gần nhất — xem [`readError`]. */
export const aiPromptReadError: DeepReadonly<Ref<IpcError | null>> = readonly(readError)

/**
 * ĐỌC lại bản ghi từ Rust — KHÔNG lắp ráp gì. Gọi khi lớp phủ mở ([`openAiPromptInspector`])
 * và khi panel AI Translation mount (đồng bộ dòng tóm tắt với bản ghi Rust đang giữ, phòng
 * trường hợp component này mount MỚI trong khi một bản ghi đã tồn tại từ một lượt Lắp trước
 * đó trong CÙNG phiên — panel dockview có thể tháo/dựng lại).
 *
 * 🔴 **SỬA — findings B4/E3/E10 (loop 1).** Bản trước ghi `record.value = result` VÔ ĐIỀU
 * KIỆN — một lượt Đọc TRƯỢT (hình dạng dây sai, mất cầu Tauri) khi đó XOÁ một bản ghi TỐT đã
 * có, chỉ vì `aiPromptReadRecord` gộp "chưa có bản ghi nào" và "Đọc trượt" thành cùng một
 * `null`. `config/aiprompt.ts::aiPromptReadRecord` giờ trả `{ value, error }` tách hai sự
 * thật đó — chỉ ghi `record` khi `error === null` (đọc THÀNH CÔNG, kể cả `value === null` là
 * chính I/O Matrix "Nothing recorded yet"); khi `error !== null`, GIỮ NGUYÊN `record` hiện có
 * và chỉ cập nhật [`readError`] để màn hình biết lượt Đọc vừa rồi không đáng tin.
 */
export async function refreshAiPromptRecord(): Promise<void> {
  const mine = ++sequence
  const result = await aiPromptReadRecord()
  if (mine !== sequence) return // một lượt Lắp/Đọc MỚI đã vượt mặt lượt này
  if (result.error !== null) {
    readError.value = result.error
    return
  }
  readError.value = null
  adoptRecord(result.value, false)
}

/** Handler thật của `ai.prompt_inspector.open` — chỉ ĐỌC (Decision 2), không đường nào ở đây
 * gọi lắp ráp. */
export function openAiPromptInspector(): void {
  overlayOpen.value = true
  tab.value = 'prompt'
  copyState.value = 'idle'
  void refreshAiPromptRecord()
}

/** Handler thật của `ai.prompt_inspector.close`. KHÔNG dọn bản ghi đã đọc — mở lại không cần
 * đọc lại NGAY (bản ghi vẫn đúng cho tới lượt Lắp/đóng Tác phẩm kế tiếp), cùng khuôn
 * `closePromptLibrary`. */
export function closeAiPromptInspector(): void {
  overlayOpen.value = false
}

/**
 * ⚠️ **SỬA 2026-09-18, bắt được ở lượt rà soát build (Blind Hunter).** [`aiPromptAssembleError`]
 * không mang định danh câu nào tạo ra nó — khác hẳn [`aiPromptRecord`], thứ §Always spec 4.7
 * đòi phải mang "the identity of what produced it… so a record from an earlier segment cannot
 * be read as describing the segment now focused". Không có định danh đó, một lỗi Lắp cho câu A
 * (ví dụ "chưa chọn bộ nào") đứng nguyên trên `.ai-inspector-alert` sau khi tiêu điểm đã dời
 * sang câu B mà người dùng chưa hề bấm Lắp lại — đúng khuyết tật hạng "bản ghi cũ" §Always
 * cấm, chỉ khác đối tượng (lỗi, không phải bản ghi).
 *
 * Gọi từ `AiTranslationPanel.vue` — nơi DUY NHẤT đọc `editorCaretSegmentId` cho mục này (cùng
 * lý lẽ `AiPromptInspectorOverlay.vue` không tự `import` state đó cho `aiPromptRecordIsStale`)
 * — mỗi khi câu đang tiêu điểm ĐỔI. Không đụng `record`/`overlayOpen`/`assembleBusy`: một lỗi
 * hết hạn không có nghĩa bản ghi ĐÃ GHI cũng hết hạn (đó là việc của `aiPromptRecordIsStale`,
 * hiển thị chứ không xoá).
 */
export function clearAiPromptAssembleError(): void {
  assembleError.value = null
}

/**
 * Handler thật của `ai.prompt.assemble` — LẮP RÁP một prompt cho `segmentId` bằng bộ prompt
 * hiệu lực `promptSetName`, rồi ghi kết quả vào [`aiPromptRecord`]. Đây là "một nút bấm
 * người dùng thấy được mà Story 4.8 sẽ thay thế" — Decision 2 spec 4.7, Consequences accepted.
 *
 * `segmentId: null` ⇔ không có câu nào đang được chọn (chưa mở Tác phẩm, hoặc đã mở nhưng
 * chưa đặt tiêu điểm vào câu nào) — **không ném, chỉ kêu**, cùng luật mọi hàm chạy từ một
 * hành động bàn phím/chuột của dự án này (`editorPanelState.ts::splitChapterHere` là tiền lệ
 * bằng chữ). Chỗ gọi (`AiTranslationPanel.vue`) tự vô hiệu hoá nút khi biết trước `segmentId`
 * sẽ là `null`, nên nhánh này chỉ còn là một lưới phòng thủ thứ hai.
 */
export async function assembleCurrentAiPrompt(promptSetName: string | null, segmentId: number | null): Promise<void> {
  if (assembleBusy.value) return
  if (segmentId === null) {
    console.warn(
      '[ai-prompt] khong lap rap: khong co cau nao dang duoc chon (editorCaretSegmentId === null)',
    )
    return
  }

  const mine = ++sequence
  latestAssembleSequence = mine
  assembleBusy.value = true
  assembleError.value = null

  const result = await aiPromptAssemble(promptSetName, segmentId)

  // Hạ cờ khi và chỉ khi mình vẫn còn là lượt LẮP mới nhất — một lượt Đọc chen vào không đụng
  // `latestAssembleSequence`, nên nó không làm cờ treo; một lượt LẮP MỚI HƠN thì tự nó đang
  // giữ cờ, nên lượt này không được tắt hộ (xem doc-comment `latestAssembleSequence`).
  const stillLatestAssemble = mine === latestAssembleSequence
  if (stillLatestAssemble) assembleBusy.value = false

  // 🔴 **SỬA — finding E4 (loop 1): kiểm lỗi TRƯỚC phép kiểm "vượt mặt toàn cục" (`sequence`),
  // không sau.** Bản trước `if (mine !== sequence) return` đứng TRƯỚC nhánh đọc `result.error`
  // — một lượt ĐỌC (không phải một lượt LẮP mới hơn) chen vào giữa lúc lượt Lắp này đang chờ
  // Rust trả lời cũng nâng `sequence`, nên nhánh vượt-mặt-toàn-cục nổ ra TRƯỚC khi lỗi kịp
  // được ghi — một lượt Lắp THẬT SỰ TRƯỢT báo cáo KHÔNG GÌ CẢ. Sửa: chỉ `latestAssembleSequence`
  // (không đụng bởi một lượt Đọc) quyết định lượt này có còn ĐƯỢC QUYỀN ghi lỗi hay không —
  // đúng ô nhớ đã tách riêng cho mục đích này.
  if (result.error !== null) {
    if (stillLatestAssemble) assembleError.value = result.error
    return
  }

  // 🔴 **SỬA — finding E2 (loop 1): `{ value: null, error: null }` (không có cầu Tauri) không
  // được ghi đè lên một bản ghi TỐT đã có.** `aiPromptAssemble` chỉ trả `value: null` cùng
  // lúc `error: null` khi lượt gọi IPC không hề chạy được (quy ước `hasIpcBridge()` của
  // `config/aiprompt.ts`) — trên đường sản phẩm thật, một lượt LẮP thành công qua Tauri không
  // bao giờ trả `value: null`. Đọc `value === null` Ở ĐÂY (sau khi đã loại nhánh lỗi) chính
  // là phát hiện đúng ca "không gọi được, không có gì để ghi" mà KHÔNG cần đổi hình dạng
  // `{ value, error }` đã thành khuôn của mọi adapter `Result`-mang khác trong kho.
  if (result.value === null) return

  if (mine !== sequence) return // một lượt Lắp/Đọc MỚI đã vượt mặt lượt THÀNH CÔNG này
  adoptRecord(result.value, true)
}

/**
 * Ba trạng thái tóm tắt Glossary của bản ghi — HÀM THUẦN, đọc từ CHÍNH sổ ghi của bản ghi
 * (§Always spec 4.7: "reads from the same record — not from a second count computed on the
 * screen"). `count` của nhánh `'asked'` là `injected.length` của CHÍNH mảng đó — không một ô
 * đếm riêng nào cộng dồn ở nơi khác.
 *
 * `'no_record'` tách khỏi `'not_asked'`: I/O Matrix "Nothing recorded yet" (chưa Lắp lần nào
 * trong phiên) là một trạng thái khác hẳn "đã Lắp, nhưng thân bộ prompt không mang
 * `{{glossary_terms}}`" — gộp hai ca đó là đúng khuyết tật §Always cấm ("no record yet this
 * session" vs "a record whose prompt is empty", áp cùng lý lẽ cho Glossary).
 */
export type GlossarySummary =
  | { kind: 'no_record' }
  | { kind: 'not_asked'; tmCount: number | null }
  | { kind: 'asked'; count: number; tmCount: number | null }

export function glossaryInjectionSummary(current: ReadonlyAssembledPromptWire | null): GlossarySummary {
  if (current === null) return { kind: 'no_record' }
  const status = current.ledger.glossary
  const tm = current.ledger.tm
  const tmCount = tm.kind === 'searched' ? tm.similar_segments.length : null
  if (status.kind === 'not_asked') return { kind: 'not_asked', tmCount }
  return { kind: 'asked', count: status.injected.length, tmCount }
}

/** Signed change in injected Glossary terms against the previous segment's record; `null` when there is no comparable earlier record or no change. */
export const aiPromptGlossaryDelta: ComputedRef<number | null> = computed(() => {
  const now = injectedCountOf(record.value)
  const before = previousGlossaryCount.value
  if (now === null || before === null || now === before) return null
  return now - before
})

/**
 * `true` ⇔ bản ghi hiện tại được lắp từ một câu KHÁC câu đang có tiêu điểm bây giờ — I/O
 * Matrix "Stale record": "Record was built from segment A; segment B is now focused ⇒ Screen
 * shows the record AND names the segment it belongs to". HÀM THUẦN — chỗ gọi
 * ([`AiPromptInspectorOverlay.vue`]) truyền `focusedSegmentId` từ `editorCaretSegmentId`
 * (tệp này không tự `import` state đó — cùng lý lẽ `glossaryConfirmStripState.ts` đã ghi cho
 * việc KHÔNG `watch(editorCaretSegmentId, …)` trong một module core).
 *
 * `focusedSegmentId === null` ⇒ không so được gì (không câu nào đang chọn) ⇒ `false` —
 * không tự xưng "cũ" khi không có gì để đối chiếu.
 */
export function aiPromptRecordIsStale(
  current: ReadonlyAssembledPromptWire | null,
  focusedSegmentId: number | null,
): boolean {
  if (current === null || focusedSegmentId === null) return false
  return current.segment_id !== focusedSegmentId
}

/**
 * Vứt toàn bộ state của mục — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()`, cùng khuôn `resetPromptSets`/`resetPromptLibrary`.
 */
export function resetAiPromptInspector(): void {
  sequence += 1
  latestAssembleSequence = sequence
  overlayOpen.value = false
  tab.value = 'prompt'
  copyState.value = 'idle'
  record.value = null
  previousGlossaryCount.value = null
  assembleBusy.value = false
  assembleError.value = null
  readError.value = null
}

export type AiPromptTierBlockKind = 'authored' | 'glossary' | 'tm' | 'source_segment'

export type AiPromptTierBlock = { kind: AiPromptTierBlockKind; text: string; chars: number }

function charCount(text: string): number {
  return Array.from(text).length
}

/** Consecutive same-kind `ledger.pieces` merged into tier blocks in prompt order; empty pieces dropped. */
export function aiPromptTierBlocks(current: ReadonlyAssembledPromptWire | null): AiPromptTierBlock[] {
  if (current === null) return []
  const blocks: AiPromptTierBlock[] = []
  for (const piece of current.ledger.pieces) {
    if (piece.text === '') continue
    const last = blocks.length > 0 ? blocks[blocks.length - 1] : undefined
    if (last !== undefined && last.kind === piece.kind) {
      last.text += piece.text
      last.chars = charCount(last.text)
      continue
    }
    blocks.push({ kind: piece.kind, text: piece.text, chars: charCount(piece.text) })
  }
  return blocks
}

/** Characters per tier; the unit is characters, never tokens. */
export function aiPromptCharsByKind(blocks: readonly AiPromptTierBlock[]): Record<AiPromptTierBlockKind, number> {
  const totals: Record<AiPromptTierBlockKind, number> = { authored: 0, glossary: 0, tm: 0, source_segment: 0 }
  for (const b of blocks) totals[b.kind] += b.chars
  return totals
}

/** Dynamic share (Glossary + TM) of all prompt characters, 0-100; `null` for an empty prompt. */
export function aiPromptDynamicRatio(blocks: readonly AiPromptTierBlock[]): number | null {
  const totals = aiPromptCharsByKind(blocks)
  const all = totals.authored + totals.glossary + totals.tm + totals.source_segment
  if (all === 0) return null
  return ((totals.glossary + totals.tm) / all) * 100
}

/** One decimal, decimal comma, no locale API (NFR16). */
export function formatPercentParam(percent: number): string {
  return (Math.round(percent * 10) / 10).toFixed(1).replace('.', ',')
}

/** Local `HH:MM:SS` of an ISO-8601 instant; unparseable input is returned as is. */
export function formatSentAtClock(iso: string): string {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  const two = (n: number): string => String(n).padStart(2, '0')
  return `${two(d.getHours())}:${two(d.getMinutes())}:${two(d.getSeconds())}`
}

/** The record's source sentence from its `source_segment` pieces; `''` when the prompt carries none. */
export function aiPromptSourceText(current: ReadonlyAssembledPromptWire | null): string {
  if (current === null) return ''
  return current.ledger.pieces
    .filter((p) => p.kind === 'source_segment')
    .map((p) => p.text)
    .join('')
}

export type AiPromptTokenLine =
  | { kind: 'unsent' }
  | { kind: 'unavailable' }
  | { kind: 'real'; promptTokens: number; costUsd: number | null }

/** Unsent, sent without a count and counted are three states; none is a fake 0. */
export function aiPromptTokenLine(current: ReadonlyAssembledPromptWire | null): AiPromptTokenLine {
  if (current === null || current.sent_at === null) return { kind: 'unsent' }
  if (current.sent_usage === null) return { kind: 'unavailable' }
  return { kind: 'real', promptTokens: current.sent_usage.prompt_tokens, costUsd: current.sent_usage.cost_usd }
}

export type AiPromptResult =
  | { kind: 'empty' }
  | { kind: 'generating'; text: string }
  | { kind: 'text'; text: string; from: 'single' | 'batch' }
  | { kind: 'error' }
  | { kind: 'not_configured' }

export type AiPromptResultInput = {
  recordSegmentId: number
  singleSegmentId: number | null
  singleState: string
  singleText: string
  singleStamp: number
  batchText: string | null
  batchStamp: number
}

/** Translation result of the record's own segment only; another segment's result never shows. */
export function aiPromptResultFor(input: AiPromptResultInput): AiPromptResult {
  const mine = input.singleSegmentId === input.recordSegmentId
  if (mine && input.singleState === 'generating') return { kind: 'generating', text: input.singleText }

  const singleUsable =
    mine && (input.singleState === 'done' || input.singleState === 'cancelled') && input.singleText !== ''
  const batchUsable = input.batchText !== null
  if (batchUsable && (!singleUsable || input.batchStamp > input.singleStamp)) {
    return { kind: 'text', text: input.batchText ?? '', from: 'batch' }
  }
  if (singleUsable) return { kind: 'text', text: input.singleText, from: 'single' }
  if (mine && input.singleState === 'error') return { kind: 'error' }
  if (mine && input.singleState === 'not_configured') return { kind: 'not_configured' }
  return { kind: 'empty' }
}
