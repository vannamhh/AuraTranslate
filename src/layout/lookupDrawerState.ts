/**
 * State của **ngăn kéo Tra cứu** — Story 4.12, Phase 3a, Task 6, Decision 2.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO STATE SỐNG Ở ĐÂY, KHÔNG TRONG `LookupDrawer.vue` HAY `StatusBar.vue`
 * ─────────────────────────────────────────────────────────────────────────────
 * Cùng lý do `shortcutsState.ts`/`dictSourcesState.ts`: đây là một `ref` module-level, sống
 * qua các lượt tháo/dựng component (một lượt đổi preset bố cục gọi `api.clear()` rồi dựng
 * lại cả ba panel của Workspace, và `StatusBar`/`LookupDrawer` không nằm trong cây đó).
 *
 * ⚠️ Cùng luật với mọi state Vue khác trong kho: tệp này **KHÔNG** được `import` vào
 * `src/commands/index.ts` — nó dùng `ref` của Vue, và Kiểm C/D/E của `npm run check:commands`
 * nạp tệp đó bằng **Node thuần**. Hai handler (`openLookupDrawer`/`closeLookupDrawer`) đi vào
 * bằng **tiêm** qua `CommandDeps` ở `src/main.ts` — cùng cửa `openShortcuts`/`closeShortcuts`.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔵 SỬA 2026-09-23 — HAI KHUYẾT TẬT CỦA BẢN ĐẦU, VÀ VÌ SAO CẢ HAI TRUY VỀ MỘT RANH GIỚI
 * ─────────────────────────────────────────────────────────────────────────────
 * Bản đầu Phase 3a bị chặn bởi một chỉ dẫn (đọc SAI) rằng `WorkspaceDock.vue` "đã đóng —
 * đừng chạm vào". Ranh giới thật hẹp hơn nhiều: đừng đổi LUẬT của cỗ máy tầng (bốn ngưỡng,
 * thứ tự ưu tiên, dòng `if (suppressPersist || autoHiddenIds.size > 0) return` của
 * `onLayoutChange`). Ice đã nới ranh giới đó ngày 2026-09-23 — tệp này và
 * `WorkspaceDock.vue::applyTier` nay nối trực tiếp với nhau, và cả hai khuyết tật dưới đây
 * đóng được TẬN GỐC nhờ đường nối đó.
 *
 * **Khuyết tật 1 — thăm dò `setInterval` 200ms vĩnh viễn (ĐÃ GỠ).** Bản đầu học tầng hiện
 * tại bằng `document.querySelector('.dock-host')` + đọc `dataset.layoutTier` mỗi 200ms, vì
 * "không có đường nào khác vào". `data-layout-tier` là một MÓC KIỂM (Phase 4 vẫn đọc nó qua
 * DOM cho e2e — xem `WorkspaceDock.vue` template), không phải một KÊNH DỮ LIỆU sản phẩm; và
 * một bộ đếm sống suốt vòng đời ứng dụng mua một độ trễ (tới 200ms) mà một lời gọi trực tiếp
 * không có. `WorkspaceDock.vue::applyTier` nay gọi thẳng [`syncLayoutTier`] ở đây, ĐỒNG BỘ,
 * ngay sau `flush()` — TRƯỚC mọi nhánh mutate dock của tầng (xem doc-comment `applyTier`).
 * `applyTier` sống trong một `.vue`, không phải `dockController.ts` — chiều import này không
 * đụng ràng buộc "nạp được bằng Node thuần" của `commands/index.ts` (chiều cấm là ngược lại:
 * `dockController.ts` không được giữ một `ref`, vì `commands/index.ts` `import` nó). Cảnh báo
 * "TẦNG BỐ CỤC ĐỌC QUA `data-*`" của bản đầu đã bị RÚT — nó đúng cho Phase 3 hiểu sai, không
 * đúng cho ranh giới thật.
 *
 * **Khuyết tật 2 — `declareFocus('panel.lookup')` ném "đã khai rồi" (ĐÃ SỬA TẬN GỐC).** Khi
 * tầng nhảy từ `narrow`/`unsupported` THẲNG về `short`/`full` trong MỘT bước trong khi ngăn
 * kéo đang mở, `WorkspaceDock.vue::applyTier` → `restoreAllAutoHidden` → `autoShow('panel.
 * lookup')` → `addPanel` mount lại `LookupPanel.vue` ở lưới **ĐỒNG BỘ**, cùng lúc bản sao
 * trong ngăn kéo còn đang khai `panel.lookup` (Vue tháo nó **bất đồng bộ**, ở lượt flush kế
 * tiếp). [`syncLayoutTier`] đóng đúng chỗ hở này: nó chạy ngay sau `flush()`, TRƯỚC mọi
 * nhánh của `applyTier` — cả nhánh gọi `restoreAllAutoHidden` — và khi tầng vừa hết đòi rút
 * trong khi ngăn kéo đang mở,
 * nó đóng ngăn kéo VÀ nhả quyền sở hữu `panel.lookup` ngay tại chỗ — `releaseFocus('panel.
 * lookup')` (từ `../commands`) — TRƯỚC KHI dòng `addPanel` chạy tới. Khi Vue cuối cùng tháo
 * bản sao cũ của ngăn kéo (bất đồng bộ), `PanelFrame.vue::onBeforeUnmount` của nó vẫn gọi
 * `releaseFocus('panel.lookup', resolve)` như thường — nhưng `resolve` nó truyền là closure
 * CỦA CHÍNH NÓ, còn đăng ký hiện tại lúc đó đã là closure của bản sao MỚI ở lưới ⇒
 * `FocusRegistry.release` (so `expected` bằng THAM CHIẾU, xem doc-comment ở `focus.ts`) nhận
 * ra đây là một lượt gỡ ĐẾN MUỘN, không phải chủ hiện tại, và thoát êm — không xoá nhầm đăng
 * ký mới, không `console.error` giả. Đã cân nhắc và loại đường "đổi `owner` của bản sao trong
 * ngăn kéo sang một id khác": sẽ làm `enter('panel.lookup')` thất bại khi Tra cứu CHỈ sống
 * trong ngăn kéo — đúng lỗi "resolver trỏ vào panel đã gỡ" mà phase file cấm tạo ra ở chiều
 * ngược lại. `owner` vẫn là đúng MỘT chuỗi `'panel.lookup'` ở cả hai nơi nó sống.
 *
 * ⇒ Không còn giới hạn CHƯA ĐÓNG nào để ghi ở đây cho ca "tầng nhảy trong khi ngăn kéo mở" —
 * khác bản đầu, đường sửa này không phải best-effort.
 */
import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import type { LayoutTier } from './workspaceLayout'
import { releaseFocus } from '../commands'

/** Hai tầng mà Tra cứu đã rút khỏi lưới — spec I/O Matrix hàng "Narrow or very short". */
function requiresRetreat(tier: LayoutTier): boolean {
  return tier === 'narrow' || tier === 'unsupported'
}

const drawerOpen = ref(false)
/** `true` khi ngăn kéo đang mở — nguồn `v-if` DUY NHẤT của `LookupDrawer.vue`. */
export const lookupDrawerIsOpen: DeepReadonly<Ref<boolean>> = readonly(drawerOpen)

const retreated = ref(false)
/** `true` khi tầng đang áp đòi Tra cứu rút khỏi lưới — nguồn hiện/ẩn điểm vào ở `StatusBar.vue`. */
export const lookupHasRetreated: DeepReadonly<Ref<boolean>> = readonly(retreated)

/**
 * Điểm vào DUY NHẤT mà tầng bố cục đẩy trạng thái mới vào ngăn kéo — gọi bởi
 * `WorkspaceDock.vue::applyTier`, ĐỒNG BỘ, ngay sau `flush()` — TRƯỚC mọi nhánh mutate dock
 * của hàm đó (xem doc-comment đầu tệp cho lý do cả hai khuyết tật đều đóng được nhờ đúng vị
 * trí này).
 *
 * ⚠️ Nhận `LayoutTier` cụ thể, không `| null` — `applyTier` chỉ gọi với tầng đã đo (Task 3
 * để lại hợp đồng: `null` KHÔNG BAO GIỜ được đọc thành một tầng, xem `measureAndApplyTier`).
 */
export function syncLayoutTier(tier: LayoutTier): void {
  const next = requiresRetreat(tier)
  if (retreated.value !== next) retreated.value = next
  if (next || !drawerOpen.value) return
  // Tầng vừa hết đòi Tra cứu rút trong khi ngăn kéo đang mở — đóng ngay VÀ nhả quyền sở hữu
  // `panel.lookup` NGAY BÂY GIỜ, đồng bộ, TRƯỚC KHI `applyTier` (người gọi hàm này) chạy tới
  // `restoreAllAutoHidden()` → `autoShow('panel.lookup')` → `addPanel` ở vài dòng dưới nó.
  // Không có phát này thì `declareFocus('panel.lookup')` của dock ném "đã khai rồi" — bản sao
  // của ngăn kéo còn đang sống, Vue tháo nó BẤT ĐỒNG BỘ ở lượt flush sau (xem
  // `PanelFrame.vue::onBeforeUnmount` cho vế nhả MUỘN không xoá nhầm đăng ký mới).
  drawerOpen.value = false
  releaseFocus('panel.lookup')
}

// ═══════════════════════════════════════════════════════════════════════════════
// HAI HANDLER — nối vào `CommandDeps` ở `src/main.ts`
// ═══════════════════════════════════════════════════════════════════════════════

/** Handler thật của `layout.lookup_drawer_open`. */
export function openLookupDrawer(): void {
  drawerOpen.value = true
}

/** Handler thật của `layout.lookup_drawer_close`. */
export function closeLookupDrawer(): void {
  drawerOpen.value = false
}
