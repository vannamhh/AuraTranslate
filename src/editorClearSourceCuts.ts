import { confirmStripIsOpen } from './glossaryConfirmStripState'
import { quickAddIsOpen } from './glossaryQuickAddState'
import { clearEditorSourceCut } from './panels/editorPanelState'
import { historyIsOpen } from './panels/segmentHistoryState'
import { shortcutsOverlayIsOpen } from './config/shortcutsState'

/**
 * Guards the bare `Escape` chord against clearing pointer cuts while a Glossary strip is
 * open: a `<button>` inside a strip isn't a typing zone, so `Escape` there would otherwise
 * both close the strip and silently wipe the cut set.
 *
 * 🔴 Cùng lý do, hai bề mặt nữa. `editor.clear_source_cuts` là
 * command bare-`Escape` DUY NHẤT (`isTypingZone` không nuốt nó cho một `<button>` đang có
 * tiêu điểm), nên `Escape` bấm trên một nút BÊN TRONG `SegmentHistoryOverlay` hay
 * `ShortcutsOverlay` vừa ĐÓNG lớp phủ đó VỪA xoá dải chốt điểm cắt — im lặng.
 *
 * ⚠️ `shortcutsOverlayIsOpen`, KHÔNG `captureIsArmed` — cái sau chỉ đúng khi lớp phủ đang
 * CHỜ MỘT HỢP ÂM (armed-for-chord), không phải "đang mở". Lớp phủ mở mà chưa armed vẫn phải
 * chặn — người dùng có thể `Escape` một nút trong đó bất cứ lúc nào lớp phủ còn mở, không
 * chỉ lúc đang ghi phím.
 *
 * ⚠️ KHÔNG thêm hai cờ này vào `isBlocked()` toàn cục ở `main.ts`: một lớp phủ đang mở
 * không được nuốt MỌI phím — nó chỉ chặn ĐÚNG command này, tại ĐÚNG cửa của command này.
 */
export function clearSourceCuts(): void {
  if (
    quickAddIsOpen.value ||
    confirmStripIsOpen.value ||
    historyIsOpen.value ||
    shortcutsOverlayIsOpen.value
  ) {
    return
  }
  clearEditorSourceCut()
}
