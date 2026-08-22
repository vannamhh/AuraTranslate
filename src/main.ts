import { createApp, watch } from 'vue'
import App from './App.vue'
import './tokens/reset.css'
// ── Story 1.14 — dockview: MỘT lượt import CSS cho cả ứng dụng ──────────────────────
//
// ⚠️ Import ở đây chứ không trong `WorkspaceDock.vue`: một `<style>` scoped của Vue
// không chở được stylesheet của thư viện, và `@import` trong tệp scoped sẽ nhân bản
// theo mỗi lượt dựng component. `main.ts` là chỗ `reset.css` đã đi qua — cùng cửa.
//
// 🔴 Thứ tự BẮT BUỘC: `dockview.css` TRƯỚC `dockview-theme.css`. Tệp thư viện khai giá
// trị mặc định cho 113 biến `--dv-*`; lớp `.dockview-theme-aura` của ta ghi đè chúng bằng
// token. Đảo thứ tự thì với những khai báo cùng độ đặc hiệu, bản của thư viện thắng.
//
// ⚠️ CSP `style-src 'self'` KHÔNG được nới, và không cần: đo thật trên
// `dist/styles/dockview.css` (3.436 dòng) — không `@import`, không `url(...)`,
// không `@font-face`. Vite gộp nó thành một tệp CSS cùng gốc, hợp lệ. Chỗ DUY NHẤT
// dockview tạo `<style>` lúc chạy là đường `addPopoutGroup`, và `scripts/check-layout.mjs`
// cấm đường đó (AC1).
import 'dockview-vue/dist/styles/dockview.css'
import './layout/dockview-theme.css'
import { applyTheme, DEFAULT_THEME, isTheme } from './tokens'
import { loadFonts } from './tokens/fonts'
import { attachKeyboard, dispatch, installCommands } from './commands'
import type { ModeId } from './commands'
import { currentMode, setMode } from './modes/modeState'
import { loadBootstrapConfig, putConfig } from './config/bootstrap'
// Story 2.3 — AD-35 vế (e): flush bản dịch chưa lưu TRƯỚC khi cửa sổ đóng.
import {
  confirmCurrentSegment,
  goToNextChapter,
  goToNextSegmentCoBao,
  goToNextUntranslatedCoBao,
  goToPrevChapter,
  goToPrevSegmentCoBao,
  clearEditorSourceCut,
  mergeCurrentSegment,
  setCurrentSegmentOmitted,
  setCurrentSegmentParagraphEnd,
  splitCurrentSegment,
  wireExitFlush,
} from './panels/editorPanelState'
// ── Story 1.14 — ba cổng của tầng bố cục ────────────────────────────────────────────
//
// ⚠️ Import ở ĐÂY, không ở `src/commands/index.ts`: tệp đó phải nạp được bằng Node thuần
// để Kiểm C/D/E của `npm run check:commands` chạy trên chính bộ command của sản phẩm. Cùng
// cửa mà `setMode` và `bindings` đã đi qua từ Story 1.6 / 1.8.
import { applyPreset, panelRing, togglePanel } from './layout/dockController'
// ── Story 1.15 — form nhập Tác phẩm ở Library ───────────────────────────────────────
//
// ⚠️ Cùng lý do và cùng cửa với ba cổng bố cục ở trên: `libraryImport.ts` là một module
// Vue thật (`ref`) và gọi `@tauri-apps/api` xuyên qua `config/project.ts` — import nó ở
// `src/commands/index.ts` giết Kiểm C/D/E.
import { submitFilePath, submitPastedText } from './modes/libraryImport'
// ── Story 1.16 — dải tab và kiểu xem của Panel Source ───────────────────────────────
//
// ⚠️ Cùng lý do và cùng cửa với `libraryImport.ts`: `sourcePanelState.ts` dùng `ref` của
// Vue — import nó ở `src/commands/index.ts` giết Kiểm C/D/E.
import { selectSourceTab, toggleHanVietView } from './panels/sourcePanelState'
// ── Story 1.17 — một lượt tra Panel Lookup ───────────────────────────────────────────
//
// ⚠️ Cùng lý do và cùng cửa với `sourcePanelState.ts`: `lookupPanelState.ts` dùng
// `ref`/`computed` của Vue.
import { runLookup } from './panels/lookupPanelState'
// ── Story 1.18 — hợp đồng vùng chọn dùng chung ──────────────────────────────────────
//
// ⚠️ Cùng lý do và cùng cửa với `lookupPanelState.ts`: `selectionContract.ts` dùng
// `watch`/`onBeforeUnmount` của Vue và chạm DOM.
import {
  attachSelectionWatcher,
  currentSelectionText,
  currentSelectionTextForGlossaryQuickAdd,
  focusSelectionSource,
  selectionCommands,
} from './panels/selectionContract'
import { installTimingProbe, markDispatch } from './panels/lookupTiming'
// ── Story 1.19 — bật/tắt nguồn từ điển và bề mặt ghi công ────────────────────────────
//
// ⚠️ Cùng lý do và cùng cửa với `lookupPanelState.ts`: `dictSourcesState.ts` dùng `ref` của
// Vue và gọi `@tauri-apps/api` xuyên qua `config/dict.ts`.
import {
  attributionIsOpen,
  closeAttribution,
  loadDictSources,
  openAttribution,
  toggleFocusedDictSource,
} from './panels/dictSourcesState'
// ── Story 1.20 — lịch sử tra cứu và mục đã ghim ──────────────────────────────────────
//
// ⚠️ Cùng lý do và cùng cửa với `dictSourcesState.ts`: `lookupHistoryState.ts` dùng `ref`
// của Vue và gọi `@tauri-apps/api` xuyên qua `config/pinned.ts`.
import {
  clearLookupHistory,
  loadPinnedEntries,
  selectLookupTab,
  toggleLookupPin,
} from './panels/lookupHistoryState'
// ── Story 1.21 — màn hình phím tắt ───────────────────────────────────────────────────
//
// ⚠️ Cùng lý do và cùng cửa với `lookupHistoryState.ts`: `config/shortcutsState.ts` dùng
// `ref`/`computed` của Vue và gọi `@tauri-apps/api` xuyên qua `config/bootstrap.ts`.
import {
  captureIsArmed,
  captureShortcut,
  closeShortcuts,
  openShortcuts,
  resetShortcut,
  unassignShortcut,
} from './config/shortcutsState'
// ── Story 2.6 — lịch sử phiên bản segment ────────────────────────────────────────────
//
// ⚠️ Cùng lý do và cùng cửa với `dictSourcesState.ts`: `segmentHistoryState.ts` dùng `ref`
// của Vue và gọi `@tauri-apps/api` xuyên qua `config/segment.ts`.
import {
  cancelPendingRestore,
  closeSegmentHistory,
  confirmPendingRestore,
  openSegmentHistory,
  restoreAimedVersion,
} from './panels/segmentHistoryState'
// ── Story 3.3 — dải "Thêm thuật ngữ" (FR48) ──────────────────────────────────────────
//
// ⚠️ Cùng lý do và cùng cửa với `lookupPanelState.ts`: `glossaryQuickAddState.ts` dùng
// `ref`/`computed` của Vue và gọi `@tauri-apps/api` xuyên qua `config/glossary.ts`.
import {
  closeGlossaryQuickAdd,
  openGlossaryQuickAdd,
  saveGlossaryQuickAdd,
} from './glossaryQuickAddState'
// ── Story 3.5 — lớp phủ "Cài đặt ngưỡng quét Glossary" (FR47) ────────────────────────
//
// ⚠️ Cùng lý do và cùng cửa với `glossaryQuickAddState.ts`: `glossarySettingsState.ts` dùng
// `ref` của Vue và gọi `@tauri-apps/api` xuyên qua `config/bootstrap.ts`.
import {
  closeGlossarySettings,
  glossarySettingsOverlayIsOpen,
  openGlossarySettings,
  saveGlossarySettings,
} from './glossarySettingsState'

/**
 * Hợp âm trên đĩa là **một chuỗi**; `CommandSpec.keys` là một **mảng**. Đây là chỗ nối.
 *
 * ⚠️ **Mã hoá TẠM, chủ sở hữu là Story 1.21** (màn hình gán phím). Quy ước hôm nay: các
 * hợp âm ngăn nhau bằng dấu phẩy, khoảng trắng thừa bị cắt, phần rỗng bị bỏ.
 *
 * 🔴 Nên một giá trị RỖNG trên đĩa nghĩa là *"thao tác này cố ý không có phím"* — một
 * phát biểu hợp lệ mà Story 1.21 phải lưu được — chứ không phải *"chưa ai đặt gì"*.
 * Hai thứ đó phân biệt nhau ở chỗ khoá **có mặt hay không** trong `shortcuts`, không ở
 * giá trị của nó.
 */
function toBindings(
  shortcuts: Readonly<Record<string, string>>,
): Readonly<Record<string, readonly string[]>> {
  const out: Record<string, readonly string[]> = {}
  for (const [id, raw] of Object.entries(shortcuts)) {
    out[id] = raw
      .split(',')
      .map((chord) => chord.trim())
      .filter((chord) => chord !== '')
  }
  return out
}

/**
 * Lượt khởi động, trọn vẹn — Story 1.8 nối cấu hình từ đĩa vào đầu chuỗi này.
 *
 * ⚠️ Một hàm `async` chứ không phải top-level `await`, để không ràng buộc mục tiêu biên
 * dịch của `vite build` vào ES2022. Thứ tự bên trong là **đúng thứ tự cũ**: hai khối
 * *"THỨ TỰ BẮT BUỘC"* không đổi một dòng lý lẽ nào.
 *
 * ⚠️ Vì sao chấp nhận một vòng IPC **trước** lượt vẽ đầu tiên: `applyTheme()` phải chạy
 * trước `mount()`, và theme là thứ **đọc từ đĩa**. Vẽ bằng `light` rồi đổi sang `dark` một
 * nhịp sau là một cú nháy TRẮNG vào mặt người đang dùng theme tối — đúng lớp lỗi mà khối
 * *"THỨ TỰ BẮT BUỘC"* số 1 tồn tại để chặn, chỉ khác nguồn.
 *
 * `loadBootstrapConfig()` **không bao giờ ném** (xem `src/config/bootstrap.ts`), nên
 * không có đường nào để lượt khởi động chết ở dòng đầu tiên.
 *
 * 🔴 **`loadFonts()` khởi động TRƯỚC `await loadBootstrapConfig()`, không phải sau** — lượt
 * review 2026-08-04 bắt được rằng đặt nó sau vòng IPC cấu hình sẽ trì hoãn lượt nạp font
 * đúng bằng thời gian round-trip đó, kéo dài đúng khoảng nháy chữ-hệ-thống mà `void`
 * (không `await`) ở dưới tồn tại để giảm thiểu. Hai lời gọi không phụ thuộc nhau nên
 * không có lý do để xếp hàng.
 */
async function boot(): Promise<void> {
  // Nạp font KHÔNG chặn `mount()` và KHÔNG chờ cấu hình: bốn tệp là ~26 MiB, và chờ chúng
  // xong (hay chờ một vòng IPC không liên quan) mới dựng cửa sổ là tự tay thêm một khoảng
  // trắng vào lúc khởi động. Chữ hiện bằng font hệ thống trong vài trăm mili-giây đầu rồi
  // đổi — `display: 'swap'` ở `fonts.ts` là thứ quyết định đó.
  //
  // Không `await` ở đây, và cũng không nuốt lỗi im lặng: `loadFonts()` đã tự bắt mọi
  // lỗi và trả về báo cáo, nên `catch` dưới đây chỉ còn bắt được lỗi của chính đường dẫn.
  void loadFonts()
    .then((results) => {
      const failed = results.filter((r) => r.status === 'failed')
      for (const r of failed) console.warn(`[fonts] ${r.family} — ${r.file}\n  ${r.detail}`)
    })
    .catch((err) => console.warn(`[fonts] đường nạp font gãy trước khi chạy: ${String(err)}`))

  const { config } = await loadBootstrapConfig()

  // ⚠️ THỨ TỰ BẮT BUỘC — `applyTheme()` phải chạy TRƯỚC `mount()`.
  //
  // Nó ghi toàn bộ token thành CSS custom properties lên `document.documentElement`. Nếu
  // `mount()` đi trước, lượt render đầu tiên chạy khi `--color-*` chưa tồn tại: mọi
  // `var(--color-…)` rơi về giá trị rỗng và người dùng thấy một nháy trắng trước khi giao
  // diện lên màu. Trên bản đã đóng gói nháy đó ngắn hơn hẳn so với máy dev, nên đây là
  // loại lỗi chỉ lộ ra ở máy người khác.
  //
  // Theme nay đến TỪ ĐĨA (Story 1.8, AC5). `DEFAULT_THEME` là đường lui khi chưa đọc được
  // gì — nền giấy, đúng hướng "Bàn viết".
  //
  // ⚠️ `isTheme` ở đây chứ không phải một `as Theme`: giá trị vừa vượt ranh giới IPC từ
  // một tệp trên đĩa, tức đúng loại dữ liệu mà kiểu TypeScript không nói được gì về nó.
  // `applyTheme` tự chốt lần nữa lúc chạy (`tokens/index.ts:72`), và hai lớp chốt ở đây rẻ
  // hơn một `documentElement` không có token nào.
  applyTheme(isTheme(config?.theme) ? config.theme : DEFAULT_THEME)

  // ⚠️ THỨ TỰ BẮT BUỘC #2 — `installCommands()` phải chạy TRƯỚC `mount()`.
  //
  // `App.vue` render ba tab chế độ với `@click="dispatch('mode.…')"`, và `dispatch` NÉM
  // với một id chưa đăng ký (AC1). Nếu lượt đăng ký đi sau `mount()` thì cú bấm đầu tiên
  // trong khoảng giữa hai lệnh sẽ ném — hẹp, nhưng có thật, và đúng loại lỗi chỉ lộ ra
  // trên máy chậm hơn máy dev.
  //
  // Đăng ký ở đây chứ KHÔNG ở trong `App.vue`: một lượt HMR dựng lại component sẽ gọi
  // `installCommands()` lần thứ hai, và `register()` ném vì id trùng — đúng hành vi AC2,
  // sai chỗ để gặp nó.
  //
  // `setMode` **và nay cả `bindings`** được TIÊM VÀO thay vì `src/commands/index.ts` tự
  // import: tệp đó phải nạp được bằng Node thuần để Kiểm C/D/E của `npm run check:commands`
  // chạy trên chính bộ command của sản phẩm. Một dòng `import { invoke } from
  // '@tauri-apps/api/core'` ở đó là kéo cầu IPC vào cổng và làm ba phép kiểm hành vi chết
  // cùng lúc (§Bẫy 6).
  //
  // 🔴 CANH GÁC — vì "ném thì đỏ ngay ở màn hình đầu tiên" chỉ đúng khi CÓ một màn hình.
  //
  // `register()`, `createKeymap()` và `parseChord()` đều ném theo thiết kế, và cả ba lý lẽ
  // biện minh cho việc ném (`registry.ts`, `commands/index.ts`) đều dựa trên tiền đề rằng
  // lỗi sẽ hiện ra trước mắt người dùng. Nhưng hai lệnh dưới đây chạy TRƯỚC `mount()` và
  // `index.html` chỉ có một `<div id="app">` rỗng — nên một lần ném ở đây cho ra **cửa sổ
  // trắng hoàn toàn**, với chẩn đoán nằm trong một console mà người dùng cuối không mở.
  //
  // Ca thật đã biết: một xung đột hợp âm chỉ tồn tại trên MỘT nền tảng (`Mod+1` phân giải
  // thành `Ctrl+Digit1` ngoài macOS) — cổng xanh trên CI, cửa sổ trắng trên máy Windows.
  //
  // ⚠️ Story 1.8 gỡ bớt MỘT nguồn của lần ném đó: hợp âm đọc từ `global.db` được thử trên
  // một registry nháp trước, và một xung đột ⇒ rơi về hợp âm mặc định thay vì ném
  // (`installCommands` §Bẫy 5). Khối `try` này vẫn ở lại — nó canh phần CÒN LẠI.
  //
  // Không nuốt lỗi: vẫn `throw` lại sau khi đã vẽ. Mục đích của khối này là làm cho lần
  // ném đó NHÌN THẤY ĐƯỢC, không phải làm cho nó biến mất.
  try {
    installCommands({
      setMode,
      bindings: config === null ? undefined : toBindings(config.shortcuts),
      // ⚠️ Ba hàm này TRA CỨU cái dock đang sống tại thời điểm CHẠY — chúng không ôm một
      // `DockviewApi` nào ở đây, vì lúc này chưa `mount()` nên chưa có cái nào tồn tại.
      // Đừng "đơn giản hoá" bằng cách truyền thẳng `api`: xem doc-comment đầu
      // `src/layout/dockController.ts`.
      applyPreset,
      togglePanel,
      panelRing,
      submitPastedText,
      submitFilePath,
      selectSourceTab,
      toggleHanVietView,
      runLookup,
      // Story 2.5 — xác nhận segment (FR24 · AD-31). ⚠️ Cổng này đi qua
      // `confirmCurrentSegment`, KHÔNG thẳng `confirmSegment` của `config/segment.ts`:
      // vế (c) của AD-35 (*flush xong rồi mới ghi trạng thái*) sống trong hàm đó, và một
      // lượt nối tắt ở đây sẽ ký một văn bản cũ hơn thứ người dùng đang nhìn.
      // 🔵 **2026-08-14 (Story 2.5b · AC14 · Quyết định #8) — KẾT QUẢ KHÔNG CÒN BỊ VỨT.**
      //
      // Bản trước viết `() => void confirmCurrentSegment()`, và `ConfirmResult` **năm giá trị
      // phân biệt được** rơi thẳng xuống đất. Cộng với việc `editorConfirmError` được export
      // mà **không component nào đọc**, hệ quả đo được là: một lượt xác nhận **bị từ chối
      // không đổi một pixel nào trên màn hình**. Người dùng bấm `⌘Enter` và không gì xảy ra —
      // đúng lớp *"rỗng im lặng"* mà cả dự án đặt ở trung tâm.
      //
      // Nay `GridPanel.vue` đọc `editorConfirmError` và hiện nhãn ở **cột trạng thái của
      // chính hàng** *(bề mặt UX-DR30 ở phạm vi tối thiểu, Ice ký)*. Bốn giá trị **không** đi
      // qua `IpcError` thì không có bề mặt nào khác để nói, nên chúng nói ở đây — **kêu, không
      // ném**: một hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném.
      //
      // ⚠️ Chẩn đoán viết KHÔNG DẤU — Kiểm A của `check:i18n` cấm chữ tiếng Việt ở vị trí mã.
      // Story 2.5b · AC12. Cùng cửa và cùng lý do với `confirmSegment`: phép chọn
      // sống ở `panels/segmentNavigation.ts` (module thuần), state ở `editorPanelState.ts`,
      // và `commands/index.ts` không được `import` cả hai (luật erasable-only).
      //
      // 🔵 Story 2.10 — ba cổng nay trỏ vào bản **CÓ BÁO** (`…CoBao`). Bản không báo vẫn được
      // export vì `tests/frontend/**` gọi thẳng chúng để kiểm vế dời con trỏ tách khỏi vế câu
      // chữ; chỗ nối dây của sản phẩm thì luôn là bản có báo, vì AC6/AC7 đòi **báo ra màn
      // hình** và `console` theo định nghĩa của dự án **là** im lặng.
      goToNextUntranslated: goToNextUntranslatedCoBao,
      goToNextSegment: goToNextSegmentCoBao,
      goToPrevSegment: goToPrevSegmentCoBao,
      // Story 2.11 (FR26) — hai lenh chuyen Chuong. Hop dong `void`, khong `boolean`: ket
      // qua chua ton tai luc `run()` tra ve (hai luot IPC noi tiep), nen no di ra bang thanh
      // trang thai. Xem `CommandDeps.goToNextChapter`.
      goToNextChapter,
      goToPrevChapter,
      confirmSegment: () => {
        void confirmCurrentSegment().then((result) => {
          if (result === 'confirmed' || result === 'refused') return
          // `'refused'` KHÔNG vào đây: nó đã có đường ra màn hình qua `editorConfirmError`.
          //
          // 🔵 2026-08-15 (code review): dòng `console.warn` này TỪNG là đường ra DUY NHẤT cho
          // ba kết quả còn lại, tức một thao tác người dùng chủ động không đổi một pixel nào —
          // vế *"một dòng ở thanh trạng thái"* của Quyết định #8 chưa từng được dựng. Nay
          // `confirmCurrentSegment` ghi vào `editorConfirmNotice` và `StatusBar.vue` đọc.
          //
          // ⚠️ `console.warn` Ở LẠI, và nó không thừa: nó là đường **chẩn đoán** *(viết KHÔNG
          // DẤU — Kiểm A của `check:i18n`)*, còn thanh trạng thái là đường **cho người dùng**.
          // Hai người đọc khác nhau, hai mức chi tiết khác nhau.
          console.warn(`[grid] khong ky duoc segment: ${result}`)
        })
      },
      // Story 2.5c · FR133 — cắt bỏ / bỏ cờ. Cùng cửa và cùng lý do với `confirmSegment`:
      // đi qua `setCurrentSegmentOmitted` của `editorPanelState.ts`, KHÔNG thẳng
      // `setSegmentOmitted` của `config/segment.ts` — ảnh chụp hiển thị (`editorSegments`)
      // sống trong hàm đó, và một lượt nối tắt đổi đĩa mà không đổi lưới.
      setSegmentOmitted: (omitted: boolean) => {
        void setCurrentSegmentOmitted(omitted).then((result) => {
          if (result === 'omitted' || result === 'restored' || result === 'refused') return
          // `'refused'` KHÔNG vào đây: nó có đường ra qua `editorOmitError` — ⚠️ và đường
          // đó **chưa có component nào đọc**, đúng món nợ mà `editorPanelState.ts` ghi tại
          // chỗ khai nó. Đừng đọc dòng này thành "đã có đường ra màn hình".
          //
          // ⚠️ Chẩn đoán viết KHÔNG DẤU — Kiểm A của `check:i18n`.
          console.warn(`[grid] khong dat duoc co cat bo: ${result}`)
        })
      },
      // Story 2.5d · FR134/AD-46 — cờ kết đoạn của BẢN DỊCH. Cùng cửa và cùng lý do với hai
      // dep ngay trên: đi qua `setCurrentSegmentParagraphEnd` của `editorPanelState.ts`,
      // KHÔNG thẳng `setSegmentParagraphEnd` của `config/segment.ts` — ảnh chụp hiển thị
      // (`editorSegments`) sống trong hàm đó, và một lượt nối tắt đổi đĩa mà không đổi lưới.
      setSegmentParagraphEnd: (endsParagraph: boolean) => {
        void setCurrentSegmentParagraphEnd(endsParagraph).then((result) => {
          if (result === 'ended' || result === 'joined') return
          // ⚠️ `'refused'` VÀO đây, khác hai dep trên: lệnh này KHÔNG có ô lỗi riêng —
          // xem doc-comment của `setCurrentSegmentParagraphEnd` và món nợ ghi ở đó.
          // ⚠️ Chẩn đoán viết KHÔNG DẤU — **quy ước TAY, không một cổng**. Kiểm A của
          //    `check:i18n` quét đúng hai quần thể: `.rs` và `.vue` (`check-i18n.mjs:860-861`);
          //    `main.ts` là `.ts` nên nằm NGOÀI phạm vi và dòng này có dấu cũng không ai đỏ.
          //    🔵 Ba dòng cùng kiểu ở trên (`:248` · `:264` · `:281`) còn ghi sai điều đó —
          //    món nợ có chủ ở `deferred-work.md`, code review 2026-08-16.
          console.warn(`[grid] khong dat duoc co ket doan ban dich: ${result}`)
        })
      },
      // Story 2.8 · FR78/AD-5 — GỘP và TÁCH tường minh. Cùng cửa và cùng lý do với ba dep
      // ngay trên: đi qua `editorPanelState.ts`, KHÔNG thẳng `mergeSegments`/`splitSegment`
      // của `config/segment.ts`. Ở đây cái giá của một lượt nối tắt cao hơn hẳn: hai hàm kia
      // còn mang **flush tập chờ trước lượt ghi** (AD-35 vế (c) — bản dịch đi vào hàng mới
      // đọc từ ĐĨA) và **lượt đặt mốc AD-47 ①** (chèn hàng mới vào `segments`). Nối tắt là
      // ghi một hàng mới từ một văn bản cũ hơn thứ đang trên màn hình, im lặng.
      mergeSegments: () => {
        void mergeCurrentSegment().then((result) => {
          if (result === 'done' || result === 'refused') return
          // ⚠️ `'refused'` KHÔNG vào đây: nó có đường ra qua `editorRegroupError` — ⚠️ và
          // đường đó **chưa có component nào đọc**, đúng khuôn `editorOmitError` đã ghi.
          // Đừng đọc dòng này thành "đã có đường ra màn hình". Món nợ có chủ.
          console.warn(`[grid] khong gop duoc segment: ${result}`)
        })
      },
      // Story 2.9 · AC8 — xoá tập điểm cắt. Cùng cửa `editorPanelState.ts` với hai dep trên,
      // nhưng KHÔNG cùng nghĩa vụ: đây là một lượt xoá state webview, không một lượt ghi đĩa,
      // nên nó không trả về gì để mà xử lý.
      clearSourceCuts: () => {
        clearEditorSourceCut()
      },
      splitSegment: () => {
        void splitCurrentSegment().then((result) => {
          if (result === 'done' || result === 'refused') return
          // ⚠️ `'no-cut'` là câu trả lời **hợp lệ**, không một lỗi: người dùng chưa bấm vào
          // cột nguyên văn nên chưa có chỗ nào để cắt. Nó kêu chứ không ném — cùng luật
          // *"hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném"*.
          console.warn(`[grid] khong tach duoc segment: ${result}`)
        })
      },
      // 🔴 STORY 1.18 — LƯỢT GỠ DEP TỐI THIỂU MÀ STORY 1.17 ĐÃ HẸN.
      //
      // Bản 1.17 là `() => window.getSelection()?.toString() ?? ''` — một dep TỐI THIỂU cố
      // ý, kèm lời hứa ghi thành chữ ở đây và ở `commands/index.ts` rằng 1.18 sẽ thay đúng
      // dòng này. Nay nó được thay, và lời hứa không ở lại quá hạn.
      //
      // Khác biệt thật: bản cũ trả về **mọi** vùng chọn trong tài liệu — kể cả trong Panel
      // Lookup (vòng tự thay thế, Bẫy 1) và trong ô nhập của Library (`:635`). Hợp đồng chỉ
      // trả về vùng chọn thuộc một bề mặt đã **đăng ký làm nguồn** (AC3), và nó biết cách
      // lấy **ký tự Hán nguồn** từ tab Hán Việt kiểu chuyển đổi.
      currentSelection: currentSelectionText,
      // Story 1.18 · AC11 — bôi đen bằng bàn phím, đóng `deferred-work.md:608`.
      focusSelectionSource,
      extendSelectionLeft: selectionCommands.extendLeft,
      extendSelectionRight: selectionCommands.extendRight,
      extendSelectionWordLeft: selectionCommands.extendWordLeft,
      extendSelectionWordRight: selectionCommands.extendWordRight,
      // Story 1.19 · AC2 · AC7 · AC11 — ba handler tĩnh, không một command cho mỗi nguồn.
      toggleDictSource: toggleFocusedDictSource,
      openAttribution,
      closeAttribution,
      // Story 1.20 · AC5 · AC6 — bốn command tĩnh cho dải tab, ghim và xoá lịch sử.
      selectLookupTab,
      toggleLookupPin,
      clearLookupHistory,
      // Story 1.21 · AC1 · AC2 · AC8 — năm handler tĩnh; hàng đang nhắm đọc lúc chạy.
      openShortcuts,
      closeShortcuts,
      captureShortcut,
      unassignShortcut,
      resetShortcut,
      // Story 2.6 · FR101 · AC1 — HAI handler tĩnh, không một lệnh bập bênh. Câu đang nhắm
      // đọc lúc chạy, không qua một tham số `segment_id`.
      openSegmentHistory,
      closeSegmentHistory,
      restoreAimedVersion,
      confirmPendingRestore,
      cancelPendingRestore,
      // Story 3.3 · FR48 — "Thêm nhanh thuật ngữ". `currentSelectionForGlossary` là đường
      // RIÊNG của lệnh này (KHÔNG `currentSelectionText`, đọc doc-comment của
      // `currentSelectionTextForGlossaryQuickAdd`).
      currentSelectionForGlossary: currentSelectionTextForGlossaryQuickAdd,
      openGlossaryQuickAdd,
      saveGlossaryQuickAdd: () => {
        void saveGlossaryQuickAdd()
      },
      closeGlossaryQuickAdd,
      // Story 3.5 · FR47 — lớp phủ ngưỡng quét Glossary.
      openGlossarySettings,
      saveGlossarySettings: () => {
        void saveGlossarySettings()
      },
      closeGlossarySettings,
    })

    // `void` tường minh: `attachKeyboard` trả về hàm gỡ, `noUnusedLocals` đang bật, và cửa
    // sổ này sống đúng bằng vòng đời tiến trình nên không có chỗ nào để gọi hàm gỡ cả.
    //
    // 🔴 **Cửa nuốt hợp âm — Story 1.19, Ice chốt ở code review 2026-08-10.** Lớp phủ
    // Attribution khai `aria-modal="true"`, và cho tới lượt này nó **không** hành xử như
    // vậy: `attachKeyboard` gắn ở `window` không hỏi ai cả, nên một hợp âm đổi preset bố cục
    // vẫn chạy được phía sau lớp phủ, gọi `api.clear()` và **dựng lại** cả ba panel bên
    // dưới. Hệ quả kéo theo là `returnFocusTo` của lớp phủ ôm một node đã rời DOM, nên lượt
    // trả tiêu điểm lúc đóng thành một lời gọi không tác dụng (UX-DR17 vỡ im lặng).
    //
    // ⚠️ Vị từ đi bằng **tiêm**, cùng cửa `toggleDictSource`/`runLookup` — xem [`KeymapGate`]
    // để biết vì sao `keys.ts` không được phép tự `import` state này.
    //
    // 🔴 **STORY 1.21 — VỊ TỪ THỨ HAI, VÀ NÓ HỎI MỘT CÂU KHÁC HẲN CÂU THỨ NHẤT.**
    //
    // `attributionIsOpen` nuốt **suốt thời gian lớp phủ mở**: Attribution khai `aria-modal`
    // và một lượt đổi preset bố cục phía sau nó gọi `api.clear()` — có hậu quả thật.
    //
    // `captureIsArmed` nuốt **chỉ khi đang chờ một hợp âm**, không suốt thời gian màn phím
    // tắt mở. Người dùng ở đó đang **đọc bảng phím** và có mọi lý do để thử `Mod+Alt+←` —
    // nó không phá gì cả (hàng 18 của bàn đo canh đúng ca này). Chỉ trạng thái chờ-một-hợp-âm
    // mới cần độc quyền bàn phím, và nó cần thật: không có cửa thì gõ `Mod+1` để **gán** phím
    // sẽ **đổi chế độ** giữa lúc gán (Bẫy 3).
    //
    // ⚠️ Lớp phủ phím tắt vẫn khai `aria-modal="true"` và vẫn `trapTab` — hai thứ đó nói về
    // **tiêu điểm**, không về hợp âm. Đừng gộp.
    void attachKeyboard(window, {
      isBlocked: () =>
        attributionIsOpen.value || captureIsArmed.value || glossarySettingsOverlayIsOpen.value,
    })
  } catch (err) {
    // ⚠️ Cố ý KHÔNG đi qua `t()`: lượt cài đặt vừa gãy, nên mọi giả định về trạng thái ứng
    // dụng đều đáng ngờ — kể cả catalog chuỗi. Một câu tiếng Việt viết thẳng ở đây là đúng
    // chỗ duy nhất trong dự án mà điều đó được phép, và lý do là nó phải hiện ra kể cả khi
    // phần còn lại đã chết.
    const host = document.getElementById('app')
    if (host !== null) {
      // ⚠️ Token, KHÔNG màu viết thẳng — và điều đó khả thi vì `applyTheme()` ở trên đã ghi
      // xong toàn bộ custom property lên `documentElement` TRƯỚC khối `try` này.
      // Một miễn trừ `aura-allow-literal` ở đây sẽ là lách cổng cho một thứ không cần lách.
      const box = document.createElement('pre')
      box.style.cssText =
        'margin:0;padding:var(--space-panel-inline);white-space:pre-wrap;' +
        'font-family:var(--face-ui-mono);font-size:var(--font-ui-mono);' +
        'line-height:var(--leading-ui-mono);color:var(--color-error);' +
        'background:var(--color-background);height:100vh;overflow:auto;box-sizing:border-box'
      box.textContent =
        'AuraTranslate không khởi động được.\n\n' +
        'Lượt đăng ký thao tác hoặc lượt dựng bàn phím đã ném, nên giao diện chưa được ' +
        'dựng.\nĐây là một lỗi lập trình, không phải lỗi dữ liệu của bạn.\n\n' +
        String(err instanceof Error ? (err.stack ?? err.message) : err)
      host.appendChild(box)
    }
    throw err
  }

  // Chế độ cuối cùng, đọc từ đĩa (AC5). Đi qua `setMode` chứ không gán thẳng: `setMode`
  // chốt tính hợp lệ lúc chạy và rơi về mặc định kèm cảnh báo, nên một `global.db` sửa tay
  // mang `"lbrary"` không dựng ra một chế độ thứ tư.
  //
  // 🔴 Chỉ gọi khi giá trị KHÁC chế độ hiện tại, và đó không phải một phép tối ưu:
  // `setMode(x)` với `x` đang là chế độ hiện tại đi vào nhánh *"bấm ⌘1 khi đang ở chính chế
  // độ đó"* và gọi `enterFocus` — mà lúc này chưa `mount()`, nên chưa chế độ nào khai điểm
  // vào focus, và `enterFocus` ghi một `console.error` vô nghĩa ở mỗi lần khởi động.
  const wanted = config?.mode ?? 'library'
  if (wanted !== currentMode.value) {
    // `as ModeId` an toàn vì `setMode` tự kiểm lúc chạy — xem `modes/modeState.ts:39`.
    setMode(wanted as ModeId)
  }

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 1.19 — DANH SÁCH NGUỒN VÀ TẬP BỊ TẮT, NẠP **TRƯỚC** `mount()`
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // ⚠️ **Không `await`**, và đó là chủ ý: `list_dict_sources` mở lại `dict_source` của mọi
  // tệp `.db`, và chờ nó xong mới dựng cửa sổ là tự tay thêm một khoảng trắng vào lúc khởi
  // động — cùng lý lẽ `loadFonts()` ở đầu `boot()`. Dải chip dựng khi danh sách về
  // (`v-if="dictSources.length > 0"`), và trong khoảng đó Panel Lookup vẫn dùng được.
  //
  // 🔴 Tập bị tắt thì đi **ĐỒNG BỘ** từ `config` đã `await` ở trên — nó là tham số của lượt
  // tra ĐẦU TIÊN, và một lượt Auto-Lookup chạy trước khi nó về sẽ trả kết quả của một bộ
  // lọc rỗng. Rust đọc `global.db` của chính nó ở mỗi lượt tra (`commands::dict`), nên đường
  // sản phẩm vẫn ĐÚNG; giá trị ở đây chỉ để dải chip vẽ đúng chip nào mờ.
  //
  // ⚠️ `?? ''` là canh gác LÚC CHẠY: giá trị vừa vượt ranh giới IPC, và một bản Rust cũ hơn
  // (trước story này) không có trường đó. Chuỗi rỗng ⇒ **mọi nguồn đều bật**.
  void loadDictSources(
    typeof config?.dict_sources_disabled === 'string' ? config.dict_sources_disabled : '',
  )

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 1.20 — BỘ GHIM, NẠP **TRƯỚC** `mount()` VÀ KHÔNG `await`
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // ⚠️ Cùng lý lẽ `loadDictSources` ngay trên: chờ một vòng IPC mới dựng cửa sổ là tự tay
  // thêm một khoảng trắng vào lúc khởi động. Tab Lịch sử đọc bốn vị từ của
  // `lookupHistoryState.ts`, và cả bốn đều `false` trong khoảng chờ ⇒ tab nói ĐÚNG một
  // thứ: không gì cả. Đó là hàng 3 của Bẫy 4 — *"đang nạp"* KHÔNG được nháy sang *"chưa
  // ghim mục nào"*.
  //
  // 🔴 **Đây là lượt nạp DUY NHẤT**, và đó là hệ quả của phạm vi: từ 2026-08-11 mục ghim
  // sống ở `global.db` (Ice ký lại), tức nó không thuộc về Tác phẩm nào và **không** phải
  // nạp lại khi Tác phẩm đổi. Bản đầu gọi thêm một lượt trong `resetLookupHistory()`; lượt
  // đó nay là một vòng IPC thừa cho cùng một sự thật.
  void loadPinnedEntries()

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 2.3 — AD-35 vế (e): NỐI ĐƯỜNG FLUSH LÚC THOÁT, **TRƯỚC** `mount()`
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // Trước `mount()` vì lượt nối này là một `listen()` trên một event của Rust, không một
  // lượt đọc DOM — nó không cần cây nào tồn tại. Và đặt sau `mount()` mở một khoảng, dù
  // ngắn, mà `WindowEvent::CloseRequested` chặn lượt đóng rồi **không ai trả lời**: người
  // dùng bấm tắt ngay khi cửa sổ vừa hiện sẽ phải chờ hết trần 1 200 ms không lý do.
  //
  // ⚠️ `void` chứ không `await`: `wireExitFlush()` không bao giờ ném (nó tự nuốt ca *"chạy
  // ngoài Tauri"*), và chờ nó là chèn một vòng IPC vào giữa đường tới lượt vẽ đầu tiên.
  // Hàm tháo listener bị bỏ có chủ ý — nó sống trọn tuổi tiến trình, cùng khuôn
  // `attachKeyboard(window, …)` ngay dưới.
  void wireExitFlush()

  createApp(App).mount('#app')

  // ═══════════════════════════════════════════════════════════════════════════════
  // 🔴 STORY 1.18 — AUTO-LOOKUP: GẮN HỢP ĐỒNG VÙNG CHỌN, **SAU** `mount()`
  // ═══════════════════════════════════════════════════════════════════════════════
  //
  // ⚠️ SAU `mount()` chứ không trước, và đó không phải một sở thích: listener sống trên
  // `document` nên nó gắn được bất cứ lúc nào, nhưng các panel chỉ **đăng ký bề mặt của
  // chúng** trong `onMounted`. Gắn trước `mount()` để lại một khoảng mà mỗi `mouseup` chạy
  // qua một sổ đăng ký RỖNG — vô hại hôm nay, nhưng nó là đúng loại "hẹp, nhưng có thật"
  // mà khối THỨ TỰ BẮT BUỘC #2 ở trên tồn tại để chặn.
  //
  // 🔴 Hợp đồng phát `dispatch('lookup.lookup_selection')` — **không gọi thẳng `runLookup`**
  // (Quyết định #4a). Nhờ vậy `Mod+Alt+L` và Auto-Lookup là **đúng MỘT đường**, nên không có
  // ca nào một đường sửa mà đường kia quên, và **Story 1.20 (lịch sử tra cứu) chỉ có một
  // chỗ để cắm vào**. Một lời gọi thẳng dựng đường thứ hai mà `check:commands` **không nhìn
  // thấy** (Kiểm A chỉ canh `@click`).
  //
  // `void`: `attachSelectionWatcher` trả hàm gỡ, và cửa sổ này sống đúng bằng vòng đời
  // tiến trình — cùng lý lẽ `attachKeyboard` ở trên.
  void attachSelectionWatcher(document, () => {
    // Mốc ĐẦU của phép đo NFR1 — *"từ lúc thả chuột"* (`epics.md:1774`), tức TRƯỚC
    // `dispatch`. Khi cờ đo TẮT (mặc định) đây là một lời gọi rỗng. Quyết định #7.
    markDispatch(currentSelectionText())
    dispatch('lookup.lookup_selection')
  })

  // Cửa bật/tắt phép đo NFR1 cho devtools — không chạy gì khi chưa ai gọi `enable()`.
  //
  // 🔴 CHỈ treo ở dev (`import.meta.env.DEV`) — lượt review 2026-08-07 bắt được rằng
  // `Reflect.set(globalThis, …)` cố ý né Kiểm C của `check-layout.mjs` (danh sách đó phục
  // vụ Story 4.12, không phải cổng đo thủ công), và vì vậy không đi qua AC13. Chấp nhận
  // né cổng đó ở dev; KHÔNG chấp nhận nó treo vô điều kiện trên cửa sổ production thật —
  // Ice chốt ở lượt review Nhóm A.
  if (import.meta.env.DEV) installTimingProbe()

  // ⚠️ Đăng ký SAU lượt đặt chế độ ban đầu, có chủ ý: nếu không, chính lượt đặt đó kích
  // hoạt một lượt ghi và mỗi lần khởi động lại viết lại đúng giá trị vừa đọc lên.
  //
  // 🔴 Đóng `deferred-work.md:140` — *"chế độ mặc định lúc khởi động là `library` và không
  // phép kiểm nào canh"*. Nay nó được lưu, và `scope_contract.rs::the_last_mode_survives_a_
  // write_and_a_reopen` canh vòng ghi-đọc-mở-lại.
  //
  // `putConfig` không bao giờ ném; một lượt lưu trượt chỉ ghi chẩn đoán. Chọn chế độ là
  // một thao tác phải MƯỢT (AD-34), và một hộp thoại lỗi ở đây là quy tắc nghiệp vụ giả
  // đặt sai chỗ.
  watch(currentMode, (mode) => {
    void putConfig('app_config', 'mode', mode).then((err) => {
      if (err !== null) {
        console.warn(`[config] không lưu được chế độ cuối (\`${err.code}\`).`)
      }
    })
  })
}

void boot()
