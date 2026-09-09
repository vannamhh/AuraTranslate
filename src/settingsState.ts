/**
 * State của lớp phủ **Cài đặt** — Story 6.8 (NFR19, AD-41), lớp phủ THỨ CHÍN. Mười một mục
 * nav; hôm nay chỉ **Quyền riêng tư** có thân (nhật ký domain — AD-41). Mười mục còn lại
 * LUÔN hiện, LUÔN nói vì sao rỗng kèm tên chủ, khuôn `tier_empty_story_6_9` của Story 6.3
 * (`ImportPreviewOverlay.vue::tierEmptyMessageKey`) — không `v-if` giấu mục nào.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 THỨ TỰ 11 MỤC — QUYẾT ĐỊNH CHƯA CÓ ICE KÝ, GHI RÕ ĐỂ KHÔNG AI ĐỌC NHẦM LÀ ĐÃ CHỐT
 * ─────────────────────────────────────────────────────────────────────────────
 * §Ask First của spec 6.8 nêu đích danh "thứ tự và tên 11 mục" là một câu hỏi CHƯA TRẢ LỜI.
 * Mười mục đầu giữ NGUYÊN thứ tự mockup (`settings.html:145-156`); `privacy` được thêm vào
 * CUỐI (không xen giữa) — lựa chọn ÍT GIẢ ĐỊNH NHẤT khi chưa biết ý định thật của thứ tự
 * mockup. Đây là một quyết định TẠM của lượt thi công này, không phải một lời chốt — xem
 * `deferred-work.md` §"Deferred from: 6-8…".
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 `Ngưỡng quét Glossary` (`GlossarySettingsOverlay.vue`, lớp phủ THỨ TƯ) KHÔNG dọn vào
 * đây — §Ask First thứ hai của spec 6.8 cũng CHƯA có câu trả lời, và dọn nó là một thay đổi
 * UI không nhỏ mà KHÔNG một Acceptance Criteria/Task nào của spec 6.8 đòi. Mục nav `glossary`
 * ở đây vì thế cũng RỖNG, cùng khuôn chín mục kia — nó không phải một cửa THỨ HAI vào cùng
 * một tính năng, chỉ là một chỗ giữ chưa có thân.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 KHUÔN `AttributionOverlay.vue`/`dictSourcesState.ts` (Story 1.19) cho bảng nhật ký domain
 * ─────────────────────────────────────────────────────────────────────────────
 * thead từ `t()` · `v-for` từ IPC · nhánh LỖI đứng TRƯỚC nhánh RỖNG · trạng thái nói bằng
 * chữ · hàm ánh xạ trả khoá LITERAL (`domainLogKindLabelKey`/`domainLogReasonKey`, cùng
 * khuôn `cleanupTierLabelKey`) · `aura-allow-text` cho dữ liệu ngoài (domain người dùng dán).
 */
import { computed, readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { listDomainLog } from './config/project'
import type { DomainLogEntryWire, DomainLogOutcomeWire } from './config/project'
import type { IpcError } from './i18n'

/** Mười một mục nav, ĐÚNG thứ tự hiện (xem §quyết định TẠM ở doc-comment đầu tệp). */
export type SettingsSection =
  | 'ai_and_model'
  | 'prompt'
  | 'glossary'
  | 'translation_memory'
  | 'dictionaries_and_sources'
  | 'shortcuts'
  | 'layout'
  | 'reading_mode'
  | 'data_and_backup'
  | 'update'
  | 'privacy'

export const SETTINGS_SECTIONS: readonly SettingsSection[] = [
  'ai_and_model',
  'prompt',
  'glossary',
  'translation_memory',
  'dictionaries_and_sources',
  'shortcuts',
  'layout',
  'reading_mode',
  'data_and_backup',
  'update',
  'privacy',
]

/** Khoá i18n của TÊN mục nav — literal, cùng khuôn `cleanupTierLabelKey`. */
export function settingsSectionLabelKey(section: SettingsSection): string {
  switch (section) {
    case 'ai_and_model':
      return 'settings.nav.ai_and_model'
    case 'prompt':
      return 'settings.nav.prompt'
    case 'glossary':
      return 'settings.nav.glossary'
    case 'translation_memory':
      return 'settings.nav.translation_memory'
    case 'dictionaries_and_sources':
      return 'settings.nav.dictionaries_and_sources'
    case 'shortcuts':
      return 'settings.nav.shortcuts'
    case 'layout':
      return 'settings.nav.layout'
    case 'reading_mode':
      return 'settings.nav.reading_mode'
    case 'data_and_backup':
      return 'settings.nav.data_and_backup'
    case 'update':
      return 'settings.nav.update'
    case 'privacy':
      return 'settings.nav.privacy'
  }
}

/** Mục nào hôm nay có THÂN thật — chỉ `privacy` (Story 6.8). Mười mục còn lại rỗng có tên
 * chủ, khuôn `tier_empty_story_6_9`. */
export function settingsSectionHasBody(section: SettingsSection): boolean {
  return section === 'privacy'
}

/**
 * "Tên chủ" của một mục CHƯA có thân — DỮ LIỆU (không phải câu), tham số `{owner}` của khoá
 * `settings.nav.no_body_yet`. Đo 2026-09-07 từ `epics.md § Traceability`: FR65-68/Prompt →
 * Epic 4 · FR11 (Chế độ đọc) → Epic 5 · FR102 (sao lưu) → Epic 1 · FR111 (cập nhật) → Epic
 * 10 · Translation Memory → Epic 7. Ba mục còn lại (`glossary`/`dictionaries_and_sources`/
 * `shortcuts`/`layout`) đã có NĂNG LỰC thật ở nơi khác trong ứng dụng (`GlossarySettingsOverlay`,
 * dải chip Attribution, `ShortcutsOverlay`, preset bố cục) — "chủ" ở đây là việc GOM chúng
 * vào MỘT khung Cài đặt, một việc CHƯA story nào nhận (xem `deferred-work.md`).
 */
export function settingsSectionOwnerLabel(section: SettingsSection): string {
  switch (section) {
    case 'ai_and_model':
    case 'prompt':
      return 'Epic 4'
    case 'glossary':
      return 'Epic 3'
    case 'translation_memory':
      return 'Epic 7'
    case 'dictionaries_and_sources':
      return 'Epic 1'
    case 'shortcuts':
      return 'Epic 1'
    case 'layout':
      return 'Epic 1'
    case 'reading_mode':
      return 'Epic 5'
    case 'data_and_backup':
      return 'Epic 1'
    case 'update':
      return 'Epic 10'
    case 'privacy':
      return ''
  }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Nhật ký domain (AD-41, NFR19) — Quyền riêng tư
// ═════════════════════════════════════════════════════════════════════════════════

/** Một hàng ĐÃ GỘP theo `(domain, kind, tier)` — bảng HIỆN gộp (§Always spec 6.8), kho
 * (`domain_log.rs`) vẫn giữ THÔ. Gộp ở TẦNG TRÌNH BÀY để một lượt gộp sai không bao giờ làm
 * mất một bản ghi thô phía dưới nó. */
export type DomainLogGroupedRow = {
  domain: string
  kind: DomainLogEntryWire['kind']
  tier: DomainLogEntryWire['tier']
  /** 🔵 THÊM 2026-09-09 (Story 6.11, mục A vòng rà đối kháng 3 lớp) — nay MỘT PHẦN của khoá gộp
   * (xem doc-comment dưới): hai lượt gọi cùng `(domain, kind, tier)` nhưng khác `outcome` (một
   * lượt tải xong, một lượt bị `TooLarge` cắt) phải ra HAI hàng — gộp chung sẽ gán một
   * "kết quả" SAI cho một phần bản ghi trong nhóm. */
  outcome: DomainLogEntryWire['outcome']
  count: number
  /** Mili-giây epoch của lượt gọi ĐẦU TIÊN trong nhóm — dòng "Thời điểm" của mockup. */
  firstAtEpochMs: number
}

/**
 * Gộp `entries` THEO ĐÚNG khoá `(domain, kind, tier, outcome)` — cùng domain nhưng khác
 * `kind`/`tier` (ví dụ: một lượt bị TỪ CHỐI rồi một lượt sau đó cùng domain lại ĐƯỢC PHÉP, thực
 * tế không xảy ra trên đường sản phẩm hôm nay nhưng khả dĩ trên đường test) ra HAI hàng riêng —
 * gộp chúng làm một sẽ làm bảng khai một "vì sao được phép" không khớp thực tế của một phần
 * bản ghi.
 *
 * 🔵 SỬA 2026-09-09 (Story 6.11, mục A vòng rà đối kháng 3 lớp) — `outcome` gia nhập khoá gộp
 * cùng lý do hệt `tier`: một domain ẢNH được phép (`tier2`) có thể vừa tải THÀNH CÔNG ở một
 * lượt vừa bị `TooLarge` cắt ở lượt khác — gộp chung một hàng thì cột "Kết quả" chỉ nói
 * đúng cho MỘT lượt trong khi con số đếm khai cả hai. **Hàm thuần, xuất được, test được.**

 *
 * ⚠️ **QUYẾT ĐỊNH CHƯA CÓ ICE KÝ, GHI RÕ ĐỂ KHÔNG AI ĐỌC NHẦM LÀ ĐÃ CHỐT (vòng rà đối kháng
 * 2, mục E3).** `outcome` gia nhập khoá gộp làm HÌNH DẠNG BẢNG đổi hẳn so với mockup
 * `web-import.html:423` (một hàng mỗi domain): một domain hôm nay có thể nở ra tới CHÍN hàng
 * (tám `outcome` khả dĩ cộng `null`), mỗi hàng một mốc "Thời điểm" riêng. Đây là lựa chọn của
 * lượt thi công này để đóng đúng ô "Error Handling" thứ hai của I/O Matrix spec 6.11 (một
 * domain ĐÃ được phép còn có thể vừa thành công vừa trượt) — chưa phải một mockup đã duyệt.
 * Nếu hình dạng CHÍN-hàng này không phải điều Ice muốn, phương án khác: gộp CHỈ theo
 * `(domain, kind, tier)` như cũ, và hiển thị outcome PHỔ BIẾN NHẤT hoặc một danh sách outcome
 * trong CÙNG một hàng (ô "Kết quả gọi" mang nhiều dòng) thay vì tách hàng.
 */
export function groupDomainLogEntries(entries: readonly DomainLogEntryWire[]): DomainLogGroupedRow[] {
  const rows = new Map<string, DomainLogGroupedRow>()
  for (const entry of entries) {
    const key = `${entry.domain} ${entry.kind} ${entry.tier} ${entry.outcome ?? 'null'}`
    const existing = rows.get(key)
    if (existing === undefined) {
      rows.set(key, {
        domain: entry.domain,
        kind: entry.kind,
        tier: entry.tier,
        outcome: entry.outcome,
        count: 1,
        firstAtEpochMs: entry.at_epoch_ms,
      })
      continue
    }
    existing.count += 1
    if (entry.at_epoch_ms < existing.firstAtEpochMs) existing.firstAtEpochMs = entry.at_epoch_ms
  }
  return [...rows.values()].sort((a, b) => a.firstAtEpochMs - b.firstAtEpochMs)
}

/** Khoá i18n của nhãn TẦNG (cột "Tầng" mockup, thật ra là `ResourceKind`: Tài liệu/Ảnh) —
 * literal, cùng khuôn `cleanupTierLabelKey`. UX-DR42: "hai tầng allowlist phân biệt bằng
 * nhãn chữ, không bằng màu". */
export function domainLogKindLabelKey(kind: DomainLogEntryWire['kind']): string {
  switch (kind) {
    case 'page':
      return 'settings.privacy.kind_page'
    case 'image':
      return 'settings.privacy.kind_image'
  }
}

/** Khoá i18n của cột "Vì sao được phép" (hoặc vì sao TỪ CHỐI khi `tier === 'denied'`) —
 * literal. UX-DR42: "mỗi hàng ghi vì sao được phép". */
export function domainLogReasonKey(tier: DomainLogEntryWire['tier']): string {
  switch (tier) {
    case 'tier1':
      return 'settings.privacy.reason_tier1'
    case 'tier2':
      return 'settings.privacy.reason_tier2'
    case 'denied':
      return 'settings.privacy.reason_denied'
  }
}

/** Khoá i18n của cột "Kết quả" cho một chặng ĐÃ ĐƯỢC PHÉP — `null` (chặng bị TỪ CHỐI, 0 kết
 * nối) trả khoá riêng cho "không áp dụng", KHÔNG dùng chuỗi rỗng (đường vòng qua i18n). 🔵
 * THÊM 2026-09-09 (Story 6.11, mục A vòng rà đối kháng 3 lớp) — `tier`/`allowed` một mình
 * không phân biệt được một chặng ĐÃ CHO PHÉP rồi tải xong với một chặng ĐÃ CHO PHÉP rồi
 * trượt mạng/MIME/quá cỡ; đây là hàm ánh xạ LITERAL, cùng khuôn `domainLogReasonKey`. */
export function domainLogOutcomeLabelKey(outcome: DomainLogOutcomeWire | null): string {
  if (outcome === null) return 'settings.privacy.outcome_not_applicable'
  switch (outcome) {
    case 'fetched':
      return 'settings.privacy.outcome_fetched'
    case 'redirected':
      return 'settings.privacy.outcome_redirected'
    case 'http_status':
      return 'settings.privacy.outcome_http_status'
    case 'timeout':
      return 'settings.privacy.outcome_timeout'
    case 'connect_failed':
      return 'settings.privacy.outcome_connect_failed'
    case 'too_large':
      return 'settings.privacy.outcome_too_large'
    case 'mime_rejected':
      return 'settings.privacy.outcome_mime_rejected'
    case 'other':
      return 'settings.privacy.outcome_other'
  }
}

const overlayOpen = ref(false)
const activeSection = ref<SettingsSection>('privacy')
const domainLogEntries = ref<DomainLogEntryWire[]>([])
const domainLogError = ref<IpcError | null>(null)
const domainLogLoading = ref(false)
/** Số thứ tự lượt đọc — chỉ lượt MỚI NHẤT được quyền ghi kết quả (cùng khuôn `toggleSequence`
 * của `dictSourcesState.ts`: một lượt đọc CŨ về SAU một lượt MỚI không được ghi đè nó). */
let sequence = 0

export const settingsOverlayIsOpen: DeepReadonly<Ref<boolean>> = readonly(overlayOpen)
export const settingsActiveSection: DeepReadonly<Ref<SettingsSection>> = readonly(activeSection)
/** Lỗi gần nhất Rust trả lời khi đọc nhật ký — ĐỨNG TRƯỚC nhánh rỗng ở tầng hiển thị (khuôn
 * `AttributionOverlay.vue`: "lỗi tải đứng trước rỗng"). */
export const settingsDomainLogError: DeepReadonly<Ref<IpcError | null>> = readonly(domainLogError)
export const settingsDomainLogLoading: DeepReadonly<Ref<boolean>> = readonly(domainLogLoading)
/** Hàng ĐÃ GỘP, sắp theo lượt gọi ĐẦU TIÊN — bảng đọc trực tiếp computed này, không tự gộp
 * trong `<template>`. */
export const settingsDomainLogRows = computed<DomainLogGroupedRow[]>(() => groupDomainLogEntries(domainLogEntries.value))
/** Nhật ký hoàn toàn RỖNG (0 bản ghi thô, không phải 0 hàng SAU gộp — hai con số luôn bằng
 * nhau vì gộp không bao giờ sinh thêm bản ghi, nhưng đọc thẳng độ dài mảng THÔ nói đúng ý
 * hơn "0 hàng" khi tầng hiển thị đổi cách gộp sau này). */
export const settingsDomainLogIsEmpty = computed<boolean>(() => domainLogEntries.value.length === 0)

/** Đọc lại nhật ký domain — gọi mỗi lần mục `privacy` trở thành mục ĐANG CHỌN (mở lớp phủ
 * thẳng vào Quyền riêng tư, hoặc bấm sang nó từ một mục khác). Nhật ký sống theo phiên chạy
 * Rust (§Always spec 6.8) — không có gì để mà cache lâu dài ở đây, một lượt đọc lại mỗi lần
 * vào mục là đúng và rẻ (mảng nhỏ, xem phép đo `domain_log.rs`). */
async function loadDomainLog(): Promise<void> {
  const mine = ++sequence
  domainLogLoading.value = true
  const result = await listDomainLog()
  if (mine !== sequence) return // một lượt mở/đóng/chuyển mục MỚI đã vượt mặt lượt này
  domainLogLoading.value = false
  if (result.error !== null) {
    domainLogError.value = result.error
    return
  }
  domainLogError.value = null
  if (result.entries !== null) domainLogEntries.value = result.entries
}

/** Handler thật của `settings.open` — mở lớp phủ vào mục ĐANG CHỌN gần nhất (mặc định
 * `privacy`, mục duy nhất có thân hôm nay). */
export function openSettings(): void {
  overlayOpen.value = true
  if (activeSection.value === 'privacy') void loadDomainLog()
}

/** Handler thật của `settings.privacy.open` — mở lớp phủ THẲNG vào Quyền riêng tư, bất kể
 * mục nào đang chọn trước đó. Nút "xem" ở chân màn xem trước URL (`ImportPreviewOverlay.vue`,
 * §Always spec 6.8) dispatch command này. */
export function openSettingsToPrivacy(): void {
  overlayOpen.value = true
  activeSection.value = 'privacy'
  void loadDomainLog()
}

/** Chuyển mục nav ĐANG CHỌN — gọi từ `@click` trên một hàng `<nav>` (không một command riêng
 * cho MỖI mục, cùng lý lẽ `lookup.toggle_source` của Story 1.19: 11 mục là danh sách TĨNH
 * biết trước lúc dựng màn phím tắt, không cần một command sinh động cho mỗi mục). */
export function selectSettingsSection(section: SettingsSection): void {
  activeSection.value = section
  if (section === 'privacy') void loadDomainLog()
}

/** Đóng lớp phủ — KHÔNG dọn nhật ký đã tải (mở lại không cần tải lại NGAY, cùng khuôn
 * `closeGlossaryManage`). */
export function closeSettings(): void {
  overlayOpen.value = false
}

/**
 * Vứt toàn bộ state của lớp phủ — `check:panel-refs` đòi mọi ô nhớ cấp module có một đường
 * `reset*()` của CHÍNH tệp này. 🔵 Cùng khuôn `resetDictSources()` (`dictSourcesState.ts`):
 * chưa có chỗ gọi SẢN PHẨM nào hôm nay (lớp phủ này không theo Tác phẩm — đổi/đóng một Tác
 * phẩm không có lý do gì để đụng vào Cài đặt hay nhật ký domain của CẢ phiên chạy) — hàm này
 * tồn tại làm chỗ cắm sẵn cho một lượt dựng lại phiên trong tương lai.
 */
export function resetSettings(): void {
  sequence += 1
  overlayOpen.value = false
  activeSection.value = 'privacy'
  domainLogEntries.value = []
  domainLogError.value = null
  domainLogLoading.value = false
}
