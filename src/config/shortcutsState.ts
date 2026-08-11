/**
 * State của **màn hình phím tắt** — Story 1.21, AC1 tới AC13.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY, KHÔNG TRONG `ShortcutsOverlay.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `dictSourcesState.ts`: một lượt đổi preset bố cục gọi `api.clear()` rồi dựng
 * lại cả bốn panel. State module-level là singleton của cả tiến trình, nên lựa chọn phím
 * của người dùng sống sót qua lượt tháo/dựng lại đó.
 *
 * ⚠️ **Ở `src/config/`, KHÔNG ở `src/panels/`.** Đây là cấu hình ứng dụng và nó gọi thẳng
 * `putConfig`/`deleteConfig` của cùng thư mục. `dictSourcesState.ts` giữ `attributionOpen`
 * — một lớp phủ toàn ứng dụng — trong khi sống ở `src/panels/`; đó là một **vết**, không
 * một tiền lệ, và story này không nhân nó lên.
 *
 * ⚠️ Cùng luật với mọi state Vue khác: tệp này **KHÔNG** được `import` vào
 * `src/commands/index.ts`. Nó dùng `ref`/`computed` của Vue và gọi `@tauri-apps/api` xuyên
 * qua `./bootstrap`, mà Kiểm C/D/E của `npm run check:commands` nạp tệp đó bằng **Node
 * thuần**. Năm handler đi vào bằng **tiêm** qua `CommandDeps` ở `src/main.ts` — cùng cửa
 * `toggleDictSource`/`toggleLookupPin` đã đi qua.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 BA TRẠNG THÁI CỦA MỘT HÀNG TRÊN ĐĨA — và cả AC8 đứng lên chỗ này
 * ─────────────────────────────────────────────────────────────────────────────
 * | Trên đĩa | Nghĩa | Cách tới |
 * |---|---|---|
 * | khoá **vắng mặt** | dùng hợp âm **mặc định của sản phẩm** | `resetShortcut` ⇒ `deleteConfig` |
 * | khoá có, giá trị **rỗng** | *"thao tác này **cố ý** không có phím"* | `unassignShortcut` ⇒ `putConfig('')` |
 * | khoá có, có hợp âm | lựa chọn của người dùng | một lượt bắt hợp âm sạch |
 *
 * Một màn hình chỉ có *"bỏ gán"* mà không có *"trả về mặc định"* là một cửa **một chiều**:
 * người dùng gỡ phím của `mode.library` rồi không còn đường nào lấy lại `Mod+1`.
 */
import { computed, readonly, ref, shallowRef } from 'vue'
import type { ComputedRef, DeepReadonly, Ref } from 'vue'
import {
  applyBindings,
  chordFromEvent,
  commandRegistry,
  conflictFor,
  currentOverrides,
  defaultChordsFor,
  detectIsMac,
  effectiveBindings,
  effectiveUnbound,
  formatChord,
  shortcutsDiskRejection,
} from '../commands'
import type { ChordEvent, ChordOverrides, CommandId, CommandSpec } from '../commands'
import { t } from '../i18n'
import type { IpcError } from '../i18n'
import { SCOPE_SHORTCUT, deleteConfig, putConfig } from './bootstrap'

/**
 * Nền tảng, đọc **một lần**. Cùng giá trị mà `installCommands` đã dùng — hai lượt đọc khác
 * nhau cho ra hai cách hiển thị `Mod` trên cùng một máy.
 */
const platform = { isMac: detectIsMac() }

const overlayOpen = ref(false)
/** Hàng đang nhắm — id thao tác, hoặc `null`. Xem [`aimRowFrom`] về thứ tự chuột/tiêu điểm. */
const aimedRow = ref<CommandId | null>(null)
/** Có đang **chờ một hợp âm** không. ⚠️ Khác *"lớp phủ đang mở"* — xem [`captureIsArmed`]. */
const capturing = ref(false)
/** Câu gần nhất phải hiện ra cho người dùng. `null` ⇒ không có gì để nói. */
const notice = shallowRef<ShortcutNotice | null>(null)
/**
 * Bộ đếm khiến mọi `computed` đọc `effectiveBindings()` tính lại sau một lượt gán.
 *
 * 🔴 `keymap` là một biến module **thường** trong `src/commands/index.ts`, không một `ref`
 * — và nó phải như vậy: tệp đó nạp bằng Node thuần ở ba phép kiểm của cổng, nên một
 * `import { ref } from 'vue'` ở đó giết cả ba cùng lúc (§Dev Notes ①). Vue không có cách
 * nào biết nó vừa đổi. ⇒ mọi đường ghi trong tệp này tăng biến này lên một, và mọi đường
 * đọc chạm vào nó. Một dòng, và nó là khác biệt giữa AC12 chạy và AC12 chỉ được viết ra.
 */
const bindingsEpoch = ref(0)

/** Một câu phải hiện ra, kèm lý do. Khoá đi qua `vi.json`; `params` là dữ liệu. */
export type ShortcutNotice = {
  key: string
  params?: Readonly<Record<string, string>>
}

/** Một hàng của bảng phím tắt — dẫn xuất lúc chạy, không lưu ở đâu cả. */
export type ShortcutRow = {
  id: CommandId
  labelKey: string
  /** Hợp âm ĐANG CÓ HIỆU LỰC, đã định dạng để đọc. Rỗng ⇒ chưa gán phím nào. */
  chords: readonly string[]
  display: readonly string[]
  /** Người dùng đã đè lên thao tác này chưa — quyết định nút *"trả về mặc định"* có nghĩa. */
  overridden: boolean
  /** Sản phẩm có gán mặc định cho nó không. Cả hai cùng `false` ⇒ trả về mặc định là vô nghĩa. */
  hasDefault: boolean
}

/** Lớp phủ có đang mở không. */
export const shortcutsOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)

/**
 * Có đang chờ một hợp âm không — **vị từ của cửa nuốt hợp âm** ở `src/main.ts`. AC10.
 *
 * 🔴 Cửa hỏi *đang bắt*, **không** hỏi *lớp phủ đang mở*, và khác biệt đó có chủ. Lớp phủ
 * Attribution chặn suốt thời gian mở vì một lượt đổi preset bố cục phía sau nó gọi
 * `api.clear()` — có hậu quả thật. Màn phím tắt thì khác: người dùng đang **đọc bảng phím**
 * và có mọi lý do để thử `Mod+Alt+←`. Chỉ trạng thái chờ-một-hợp-âm mới cần độc quyền
 * bàn phím.
 */
export const captureIsArmed: DeepReadonly<Ref<boolean>> = readonly(capturing)

/** Hàng đang được nhắm, để lớp phủ tô nó lên. */
export const aimedShortcutRow: DeepReadonly<Ref<CommandId | null>> = readonly(aimedRow)

/** Câu gần nhất phải hiện ra. */
export const shortcutNotice: DeepReadonly<Ref<ShortcutNotice | null>> = readonly(notice)

/**
 * Hợp âm trên đĩa bị từ chối ⇒ một câu nói ra điều đó. AC13, đóng `deferred-work.md:243`.
 *
 * Cho tới story này chẩn đoán chỉ đi ra `console.error` — im lặng theo nghĩa thực dụng, vì
 * người dùng chỉ biết nếu họ mở console, và lựa chọn của họ lặng lẽ không được áp.
 */
export const diskBindingsRejected: ComputedRef<boolean> = computed(() => {
  void bindingsEpoch.value
  return shortcutsDiskRejection() !== null
})

/**
 * **MỌI** command đã đăng ký, kèm hợp âm đang có hiệu lực. AC1 · AC12.
 *
 * 🔴 Hợp âm đọc từ `effectiveBindings()`, **không** từ `spec.keys`: spec bị đóng băng lúc
 * `register()` nên nó trả lời *thời điểm cài đặt* và sau lượt gán đầu tiên nó **cũ**. Xem
 * doc-comment của `CommandSpec.keys` ở `src/commands/registry.ts`.
 */
export const shortcutRows: ComputedRef<readonly ShortcutRow[]> = computed(() => {
  void bindingsEpoch.value
  const byId = new Map<CommandId, string[]>()
  for (const binding of effectiveBindings()) {
    const list = byId.get(binding.id)
    if (list === undefined) byId.set(binding.id, [binding.chord])
    else list.push(binding.chord)
  }
  return commandRegistry.list().map((spec: CommandSpec): ShortcutRow => {
    const chords = byId.get(spec.id) ?? []
    return {
      id: spec.id,
      labelKey: spec.labelKey,
      chords,
      display: chords.map((chord) => formatChord(chord, platform)),
      overridden: hasOverride(spec.id),
      hasDefault: defaultChordsFor(spec.id).length > 0,
    }
  })
})

/**
 * Các thao tác **chưa gán phím nào** lúc chạy. AC5 · AC12.
 *
 * ⚠️ **Không** `commandRegistry.unbound()` — nó lọc trên `spec.keys` nên nó không bao giờ
 * biết một lượt gán vừa xảy ra. Hàng 5 của bàn đo canh đúng chỗ này: gán một phím, **không**
 * đóng màn hình, và hàng đó phải rời nhóm ngay.
 */
export const unboundShortcutIds: ComputedRef<readonly CommandId[]> = computed(() => {
  void bindingsEpoch.value
  return effectiveUnbound().map((spec) => spec.id)
})

function hasOverride(id: CommandId): boolean {
  return Object.prototype.hasOwnProperty.call(currentOverrides(), id)
}

/** Nhãn tiếng Việt của một thao tác. Rơi về chính id nếu nó không còn trong sổ đăng ký. */
function labelOf(id: CommandId): string {
  const spec = commandRegistry.list().find((s) => s.id === id)
  return spec === undefined ? id : t(spec.labelKey)
}

/** Hợp âm mặc định của sản phẩm, đã định dạng — để màn hình nói *"mặc định là ⌘1"*. */
export function defaultDisplayFor(id: CommandId): readonly string[] {
  return defaultChordsFor(id).map((chord) => formatChord(chord, platform))
}

// ═══════════════════════════════════════════════════════════════════════════════
// NHẮM HÀNG — `@mousedown` TRƯỚC `activeElement`, và thứ tự đó là một ca ĐO ĐƯỢC
// ═══════════════════════════════════════════════════════════════════════════════
//
// 🔴 Story 1.19 đã trả giá cho bài này, và `dictSourcesState.ts:290-341` ghi lại nguyên
// văn: WebKit (WKWebView — engine Tauri trên macOS) **không đặt tiêu điểm cho `<button>`
// khi bấm chuột**. Đọc mỗi `activeElement` là để cả đường chuột chết trên macOS trong khi
// xanh trên Windows.
//
// Và thứ tự thì cũng đã trả giá: Tab tới hàng A *(giờ `activeElement` là A)*, rồi **bấm
// chuột** hàng B. `mousedown` đặt `aimed = 'B'` đúng, nhưng `activeElement` vẫn là A.
// ⇒ `aimed` ĐỨNG TRƯỚC, luôn luôn.

/**
 * Nhắm hàng nằm dưới một sự kiện chuột — gọi từ `@mousedown` trên **vùng chứa** bảng.
 *
 * ⚠️ Uỷ quyền ở vùng chứa chứ không một handler trên mỗi hàng: một bảng 30+ hàng là 30+ chỗ
 * để sai, và `@click` của mỗi nút phải ở lại **đúng một** lời gọi `dispatch` (Kiểm A).
 *
 * ⚠️ **Không tiêu thụ một lần** — khác `aimedCode` của `dictSourcesState.ts`, và khác có
 * lý do: ở đó một cú bấm là **một** thao tác trọn vẹn *(bật/tắt nguồn)*, còn ở đây nhắm một
 * hàng rồi bấm ba nút khác nhau *(bắt · bỏ gán · trả về mặc định)* là đường đi bình thường.
 * Hàng đang nhắm là một **lựa chọn**, không một sự kiện.
 */
export function aimRowFrom(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLElement)) return
  const row = target.closest<HTMLElement>('[data-command-id]')
  const id = row?.dataset.commandId
  if (id === undefined) return
  aimedRow.value = id
  // Nhắm sang hàng khác giữa chừng ⇒ lượt bắt đang chờ hết đối tượng. Bỏ nó, đừng để nó
  // âm thầm gán vào hàng mới.
  //
  // ⚠️ Đi qua `cancelCapture()` chứ không gán thẳng `capturing.value = false`: một lượt huỷ
  // gồm HAI vế, và bỏ vế thứ hai để câu *"Đang chờ một tổ hợp phím — Escape để huỷ"* treo
  // lại trên màn hình sau khi đã không còn chờ gì nữa (bắt ở code review 2026-08-11).
  if (capturing.value) cancelCapture()
}

/**
 * Ô phím của một hàng — để **ÉP** tiêu điểm vào nó lúc arming.
 *
 * 🔴 Vì sao phải ép *(bắt ở code review 2026-08-11)*: WKWebView **không đặt tiêu điểm cho
 * `<button>` khi bấm chuột** — `panels/dictSourcesState.ts:289-292` ghi bằng chữ và đã trả
 * giá cho bài này ở code review 2026-08-10. `@keydown` bắt hợp âm sống trên ô phím, nên
 * không ép thì đường **chuột** của AC2 chết trên macOS: `capturing` bật, câu *"đang chờ một
 * tổ hợp phím"* hiện ra, `@keydown` không bao giờ nổ, và cửa `isBlocked` của `main.ts` nuốt
 * sạch hợp âm người dùng gõ. *"Bấm mà không có gì xảy ra"* — đúng lớp lỗi AD-44 ④ cấm.
 *
 * ⚠️ Tra bằng `data-key-cell`, **không** bằng lớp CSS: `.sc-key` thuộc khối `style` scoped
 * của `ShortcutsOverlay.vue`, tức một chi tiết trình bày đổi được lúc nào cũng được. Cùng
 * khuôn `[data-shortcuts-open]` mà đường trả tiêu điểm của chính lớp phủ đó đang đi.
 *
 * ⚠️ `document.querySelector` (SỐ ÍT) có chủ đích: `querySelectorAll` **không** nằm trong
 * `ALLOWED_GLOBAL_MEMBERS` của `check-layout.mjs:394-424`, và cổng đó đúng — nới danh sách
 * cho một nhu cầu mà API đã cho phép làm được là nới vô cớ. Bản số ít đã được thêm vào danh
 * sách ở code review 2026-08-10 cho **đúng** nhu cầu này: tìm lại một node theo thuộc tính
 * `data-` để trao tiêu điểm, khi một `ref` không dùng được.
 *
 * ⚠️ Nội suy `id` vào selector không cần escape, và đó là một phép đo chứ không một lời hứa:
 * cả **34** command id đọc từ `commandRegistry.list()` đều khớp `[a-z0-9]+(\.[a-z0-9_]+)+`.
 * Chúng là chuỗi hằng trong `src/commands/index.ts` — không đường nào cho dữ liệu người dùng
 * vào một command id.
 */
function keyCellOf(id: CommandId): HTMLElement | null {
  if (typeof document === 'undefined') return null
  return document.querySelector<HTMLElement>(`[data-command-id="${id}"] [data-key-cell]`)
}

/** Id của hàng đang có tiêu điểm DOM, hoặc `null`. */
function focusedRowId(): CommandId | null {
  if (typeof document === 'undefined') return null
  const active = document.activeElement
  if (!(active instanceof HTMLElement)) return null
  return active.closest<HTMLElement>('[data-command-id]')?.dataset.commandId ?? null
}

/**
 * Hàng mà một thao tác sẽ áp vào. `aimedRow` **trước**, tiêu điểm DOM sau — xem khối trên.
 */
function targetRow(): CommandId | null {
  return aimedRow.value ?? focusedRowId()
}

/**
 * Không xác định được hàng ⇒ **không im lặng**, một câu có lý do.
 *
 * Cùng doctrine `panel.lookup.pin_no_target` của Story 1.20: một thao tác không có mục tiêu
 * mà không nói gì là *"bấm mà không có gì xảy ra"*, đúng lớp lỗi AD-44 ④ cấm.
 */
function requireRow(): CommandId | null {
  const id = targetRow()
  if (id === null) notice.value = { key: 'shortcuts.no_target' }
  return id
}

// ═══════════════════════════════════════════════════════════════════════════════
// NĂM HANDLER — nối vào `CommandDeps` ở `src/main.ts`
// ═══════════════════════════════════════════════════════════════════════════════

/** Handler thật của `shortcuts.open`. AC1. */
export function openShortcuts(): void {
  overlayOpen.value = true
  notice.value = null
}

/** Handler thật của `shortcuts.close`. */
export function closeShortcuts(): void {
  overlayOpen.value = false
  capturing.value = false
  aimedRow.value = null
  notice.value = null
}

/** Handler thật của `shortcuts.capture` — vào trạng thái **chờ một hợp âm**. AC2 · AC10. */
export function captureShortcut(): void {
  const id = requireRow()
  if (id === null) return
  aimedRow.value = id
  // 🔴 ÉP tiêu điểm lên ô phím — xem [`keyCellOf`] về vì sao đường chuột của AC2 chết trên
  // macOS mà không có dòng này.
  //
  // ⚠️ Và ép **TRƯỚC** khi bật `capturing`, không sau. `focus()` phát `focusin`, `focusin`
  // gọi `aimRowFrom`, và `aimRowFrom` có một nhánh **huỷ lượt bắt**. Đặt `capturing = true`
  // trước rồi mới `focus()` là tự bắn vào chân mình: lượt bắt vừa bật bị chính cú ép tiêu
  // điểm tắt đi, và triệu chứng y hệt lỗi mà dòng này tồn tại để sửa.
  keyCellOf(id)?.focus()
  capturing.value = true
  notice.value = { key: 'shortcuts.capturing' }
}

/** Huỷ lượt bắt. `Escape` ở trạng thái *đang bắt* dừng ở đây — xem Bẫy 4 của story. */
export function cancelCapture(): void {
  capturing.value = false
  notice.value = null
}

/** Handler thật của `shortcuts.unassign` — *"cố ý không có phím"*, khác *"về mặc định"*. AC8. */
export function unassignShortcut(): void {
  const id = requireRow()
  if (id === null) return
  const next: Record<CommandId, readonly string[]> = { ...currentOverrides(), [id]: [] }
  if (!commitBindings(next)) return
  // Lượt này đã thành công ⇒ câu cũ hết đúng. Cùng vế mà `handleCaptureKey` đã có; thiếu nó
  // ở đây để một câu xung đột treo lại sau khi người dùng đã sửa xong bằng chính nút này
  // (bắt ở code review 2026-08-11).
  notice.value = null
  void persist(id, () => putConfig(SCOPE_SHORTCUT, id, ''))
}

/** Handler thật của `shortcuts.reset` — **xoá khoá**, hàng rơi về mặc định sản phẩm. AC8. */
export function resetShortcut(): void {
  const id = requireRow()
  if (id === null) return
  const next: Record<CommandId, readonly string[]> = { ...currentOverrides() }
  // Xoá khỏi lớp override ⇒ `createKeymap` rơi về `spec.keys`, tức mặc định sản phẩm.
  delete next[id]
  if (!commitBindings(next)) return
  // Xem [`unassignShortcut`] — cùng lý do, cùng lượt vá.
  notice.value = null
  void persist(id, () => deleteConfig(SCOPE_SHORTCUT, id))
}

/**
 * Một sự kiện bàn phím trong trạng thái **đang bắt** ⇒ gán, hoặc từ chối kèm lý do.
 *
 * Gọi từ `@keydown` của ô phím trong `ShortcutsOverlay.vue`. Trả `true` nếu sự kiện đã được
 * tiêu thụ — chỗ gọi dùng nó để quyết định có `preventDefault()` không.
 *
 * ⚠️ **`⌫` KHÔNG phải phím "bỏ gán" ở đây.** `Backspace` có trong `NAMED_CODES`, tức
 * `Backspace` trần và `Mod+Backspace` đều là hợp âm hợp lệ gán được; đọc nó thành "bỏ gán"
 * ở trạng thái này là **khoá vĩnh viễn** một phím khỏi bảng, không dấu hiệu nào. `⌫` bỏ
 * gán ở trạng thái **nghỉ**, và chỗ gọi phân biệt hai trạng thái đó. Xem Bẫy 5 của story.
 */
export function handleCaptureKey(event: ChordEvent): boolean {
  if (!capturing.value) return false
  const id = aimedRow.value
  if (id === null) return false

  const chord = chordFromEvent(event, platform)
  // `null` ở đây là BA ca: lượt commit của bộ gõ, một keydown chỉ có phím bổ trợ, và một
  // phím ngoài bảng. Hai ca đầu phải **tiếp tục chờ** — người dùng chưa gõ xong. Ca thứ ba
  // phải nói ra (AC11). `chordFromEvent` không phân biệt chúng, nên phân biệt ở đây.
  if (chord === null) {
    if (isStillTyping(event)) return false
    notice.value = { key: 'shortcuts.key_unknown' }
    capturing.value = false
    return true
  }

  const clash = conflictFor(chord, id)
  if (clash !== null) {
    // 🔴 Lệnh gán sau **BỊ CHẶN**, không cướp phím của lệnh gán trước (`settings.html:286`).
    // Hợp âm cũ của hàng này không đổi một chữ.
    //
    // ⚠️ Câu nói ra **NHÃN** của đối thủ, không id của nó: `lookup.toggle_pin` là một định
    // danh máy, và một người dùng đọc nó vẫn không biết thao tác nào đang giữ phím.
    notice.value = { key: 'shortcuts.conflict', params: { other: labelOf(clash.heldBy) } }
    capturing.value = false
    return true
  }

  const next: Record<CommandId, readonly string[]> = { ...currentOverrides(), [id]: [chord] }
  capturing.value = false
  if (!commitBindings(next)) return true
  notice.value = null
  void persist(id, () => putConfig(SCOPE_SHORTCUT, id, chord))
  return true
}

/**
 * Sự kiện này là *"chưa gõ xong"* chứ không *"phím không dùng được"*?
 *
 * ⚠️ Đọc cùng hai điều kiện mà `chordFromEvent` đọc, và đó là một sự trùng lặp CÓ CHỦ: hàm
 * kia trả về đúng một `null` cho ba ca vì nó là một hàm thuần ở tầng dữ liệu và một kiểu
 * trả về ba nhánh ở đó sẽ lây sang mọi chỗ gọi. Phân biệt sống ở đây, nơi duy nhất cần nó.
 */
function isStillTyping(event: ChordEvent): boolean {
  if (event.isComposing === true) return true
  return /^(Meta|Control|Shift|Alt)(Left|Right)$/.test(event.code)
}

/**
 * Áp lớp hợp âm mới vào keymap đang sống. `false` ⇒ **không** áp được, và keymap cũ nguyên vẹn.
 *
 * 🔴 §Bẫy 9 — `applyBindings` dựng vào một biến tạm và chỉ thay khi thành công, nên một
 * lượt trượt ở đây để lại **mọi phím cũ vẫn chạy**. Đó là điều kiện để người dùng còn đường
 * bàn phím mà sửa chính lượt gán vừa hỏng.
 */
function commitBindings(next: ChordOverrides): boolean {
  const outcome = applyBindings(next)
  if (!outcome.ok) {
    console.error(`[shortcuts] lượt dựng lại keymap trượt — giữ nguyên bộ cũ. ${outcome.detail}`)
    notice.value = { key: 'shortcuts.apply_failed' }
    return false
  }
  bindingsEpoch.value += 1
  return true
}

/**
 * Ghi xuống đĩa. **Sau** lượt áp, và tách rời nó có chủ đích (AD-21).
 *
 * ⚠️ Một lượt lưu trượt **không** rút lại lượt áp: phím mới vẫn chạy trong phiên này, và
 * câu `shortcuts.save_failed` nói đúng điều đó — *"lần mở sau sẽ trở về giá trị cũ"*. Rút
 * lại sẽ là một thao tác đã có hiệu lực trước mắt người dùng rồi tự hoàn tác, thứ khó hiểu
 * hơn hẳn một câu nói thẳng.
 */
async function persist(id: CommandId, write: () => Promise<IpcError | null>): Promise<void> {
  const err = await write()
  if (err !== null) {
    console.warn(`[shortcuts] không lưu được hợp âm của \`${id}\`.`)
    notice.value = { key: 'shortcuts.save_failed' }
  }
}
