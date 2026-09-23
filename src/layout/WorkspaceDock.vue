<script setup lang="ts">
// Vỏ `dockview` của Workspace. Story 1.14 · AC1–AC7 · AC9 · FR16 · FR17 · FR18 · AD-24.
//
// ─────────────────────────────────────────────────────────────────────────────────
// 🔴 KHÔNG MỘT LỜI GỌI `addPopoutGroup()` NÀO — AD-24, và nó là một cổng, không phải
//    một lời hứa
// ─────────────────────────────────────────────────────────────────────────────────
// Đo thật trên bundle đã phát hành (`dockview-core/dist/package/main.esm.mjs`):
// `addPopoutGroup` là đường **DUY NHẤT** gọi `window.open`, và là đường **DUY NHẤT** tạo
// một `<style>` lúc chạy để chép stylesheet sang cửa sổ mới. Nó ⇒ **cửa sổ hệ điều hành
// thứ hai**, tức vi phạm thẳng AD-24 (*một cửa sổ, ba chế độ*) — thứ mà `epics.md` gọi là
// *"trả bằng chính thứ sản phẩm bán"*. Nó cũng là chỗ duy nhất CSP `style-src 'self'` bị
// đụng tới.
//
// ⇒ Undock của FR17 = `api.addFloatingGroup()` — nhóm nổi **trong cùng cửa sổ**.
// ⇒ `scripts/check-layout.mjs` cưỡng chế mệnh đề này trên toàn `src/**` (AC1, AC12).
//
// ─────────────────────────────────────────────────────────────────────────────────
// KHÔNG TỰ VIẾT LẠI DOCK / UNDOCK / GỘP TAB / ĐỔI KÍCH THƯỚC
// ─────────────────────────────────────────────────────────────────────────────────
// `EXPERIENCE.md:21` viết thẳng: *"dock, undock, gộp tab, đổi kích thước và preset đều là
// năng lực SẴN CÓ của nó, không tự viết lại"*. Tệp này vì vậy không có một handler
// kéo–thả nào. Cái nó thêm vào là bốn thứ mà dockview không biết:
//   1. hai preset bố cục của UX-DR13 (AC5, AC6);
//   2. ẩn/hiện panel kèm **sổ vị trí đã nhớ** (AC3, §Quyết định #3A);
//   3. vòng xoay focus theo **thứ tự bố cục** + dời focus DOM tường minh (AC9, AD-34 §2);
//   4. lưu/khôi phục bố cục qua các phiên, có nhịp ghi (AC4, §Quyết định #5A).
//
// ─────────────────────────────────────────────────────────────────────────────────
// ⚠️ MỌI CHUỖI `console.*` TRONG TỆP NÀY VIẾT **KHÔNG DẤU** — và đó không phải cẩu thả
// ─────────────────────────────────────────────────────────────────────────────────
// Kiểm A của `scripts/check-i18n.mjs` quét mọi `.vue` dưới `src/**` và đỏ với một chuỗi
// tiếng Việt CÓ DẤU ở vị trí mã (AC2 của Story 1.5). Nó đo **DẤU**, nên nó không phân
// biệt được *"chuỗi hiển thị"* với *"chẩn đoán ra console"* — `deferred-work.md §*Deferred from: code review of 1-2-scaffold-du-an-va-khoa-pham-vi-filesystem-pham-vi-mang (2026-08-03)*` đã ghi
// đúng giới hạn đó, và §Quyết định #6 của story này mở rộng cổng theo chiều KHÁC (đo text
// node của template), không nới chiều này.
//
// Đường thoát dễ là dời khối logic dưới đây sang một tệp `.ts` — Kiểm A không quét
// `.ts`. `deferred-work.md §*Deferred from: code review of 1-2-scaffold-du-an-va-khoa-pham-vi-filesystem-pham-vi-mang (2026-08-03)*` gọi tên đúng đường đó và cấm nó bằng chữ: *"dời một chuỗi
// từ `.vue` sang `.ts` là cách hợp lệ về mặt cổng để cho xanh — đừng dùng."*
//
// ⇒ Dùng tiền lệ đã có: `src-tauri/src/commands/config.rs:36` cũng viết không dấu, cùng
// lý do. Người đọc những dòng này là người đang mở DevTools, không phải người dùng cuối.
// ⚠️ Comment tiếng Việt CÓ DẤU thì hợp lệ — Kiểm A che comment trước khi quét.
import { onActivated, onBeforeUnmount, onDeactivated, onMounted, shallowRef } from 'vue'
import { DockviewVue } from 'dockview-vue'
import type { DockviewApi, DockviewReadyEvent, IDockviewPanel, VueComponent } from 'dockview-vue'
import { enterFocus } from '../commands'
import GridPanel from '../panels/GridPanel.vue'
import LookupPanel from '../panels/LookupPanel.vue'
import AiTranslationPanel from '../panels/AiTranslationPanel.vue'
import PanelTab from '../panels/PanelTab.vue'
import { setDockController } from './dockController'
import { createWriteSchedule } from './writeSchedule'
// Story 4.12, Phase 3a (sửa 2026-09-23) — đẩy tầng mới vào ngăn kéo Tra cứu ĐỒNG BỘ, thay
// một thăm dò `setInterval` đã gỡ. Xem doc-comment đầu `lookupDrawerState.ts` cho lý lẽ đầy
// đủ (Khuyết tật 1 + Khuyết tật 2) và vì sao chiều import này (từ một `.vue` sang một `.ts`
// dùng `ref`) không đụng ràng buộc "nạp được bằng Node thuần" của `src/commands/index.ts` —
// chiều cấm là NGƯỢC LẠI (`dockController.ts` không được giữ một `ref`, vì `commands/
// index.ts` `import` nó).
import { syncLayoutTier } from './lookupDrawerState'
import {
  DEFAULT_PRESET_ID,
  layoutTierFor,
  nextToSacrifice,
  PANEL_COMPONENTS,
  PANEL_IDS,
  PANEL_TITLE_KEYS,
  presetById,
  SACRIFICE_ORDER,
} from './workspaceLayout'
import type { LayoutTier, PanelId, PlacementDirection, PresetId, WorkArea } from './workspaceLayout'
import { findTreeSpot, unmergeForPersist } from './dockTree'
import type { SerializedDockJSON, SerializedGrid } from './dockTree'

const props = defineProps<{
  /**
   * Bố cục đã lưu, đọc từ `global.db` qua `bootstrap_config` (AC4). Chuỗi rỗng = chưa có
   * gì trên đĩa ⇒ preset mặc định. Không `null` để chỗ này không phải phân biệt
   * *"chưa nạp"* với *"đã nạp, rỗng"* — `main.ts` đã phân xử.
   */
  savedLayout: string
}>()

const emit = defineEmits<{
  /** Bố cục đã ổn định và cần được ghi xuống đĩa. Payload là `api.toJSON()` đã stringify. */
  (e: 'persist', json: string): void
  /**
   * Tầng vừa ÁP THẬT — Story 4.12 Phase 3b. Bắn từ [`measureAndApplyTier`], cùng chỗ gọi
   * [`applyTier`], nên không bao giờ mang `null`: hợp đồng Phase 1 (`layoutTierFor` không đọc
   * được ⇒ giữ nguyên tầng cũ, không bắn sự kiện) không đổi.
   */
  (e: 'tier-change', tier: LayoutTier): void
}>()

/**
 * **Ba** component nội dung, tra theo tên đã đăng ký.
 *
 * 🔵 2026-08-14 (Story 2.5b): bốn → ba. `SourcePanel` + `EditorPanel` gộp thành `GridPanel`.
 *
 * ⚠️ `PANEL_COMPONENTS` (ở tầng thuần) và map này phải khớp nhau, và không cổng nào canh
 * điều đó — một tên lệch cho ra một panel trắng với `console.error` của chính dockview.
 * Giữ hai bảng cạnh nhau về mặt tên biến là thứ rẻ nhất làm được hôm nay.
 *
 * 🔴 VỀ CÁI `as unknown as` — nó là chỗ nối với kiểu của thư viện, không phải một lượt
 * tắt tiếng TypeScript.
 *
 * `dockview-vue` khai `VueComponent<T = any> = DefineComponent<T>` (`utils.d.ts`), tức map
 * của nó đòi `DefineComponent<any>`. Prop của một component là vị trí **nghịch biến**, nên
 * `DefineComponent<DockviewPanelProps>` **không** gán được cho `DefineComponent<any>`:
 * TypeScript đúng khi từ chối — một `Record<string, DefineComponent<any>>` cho phép mount
 * bất cứ prop nào, còn ba component này đòi `params`.
 *
 * Đường thay thế duy nhất là khai `params?:` (tuỳ chọn) ở CẢ NĂM component. Nó qua được
 * kiểu, và nó nói dối: dockview LUÔN truyền `params`, còn `PanelTab.vue` thì không chạy
 * được nếu thiếu — mọi lời gọi `api` ở đó sẽ phải mọc một `?.` cho một ca không tồn tại.
 * Ép kiểu một lần **ở đúng ranh giới thư viện** rẻ hơn bốn lời nói dối rải trong mã.
 */
const components = {
  grid: GridPanel,
  lookup: LookupPanel,
  aiTranslation: AiTranslationPanel,
} as unknown as Record<string, VueComponent>

/** ⚠️ MỘT tab component cho cả ba panel — §Quyết định #4A. */
const tabComponents = { aura: PanelTab } as unknown as Record<string, VueComponent>
const TAB_COMPONENT = 'aura'

/**
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 THEME PHẢI ĐI QUA PROP `theme`, KHÔNG CHỈ QUA `class` — §BẪY 1, BẮT ĐƯỢC LÚC ĐO
 * ═════════════════════════════════════════════════════════════════════════════════
 *
 * Lượt nghiệm thu thị giác (Task 11) đọc DOM thật và thấy dockview **tự dán**
 * `class="dv-shell dockview-theme-abyss"` lên phần tử con khi không ai truyền `theme`.
 *
 * Đó không phải một chi tiết vô hại. Custom property kế thừa theo **phần tử gần nhất**,
 * và `.dv-shell` NẰM TRONG phần tử mang `.dockview-theme-aura` của ta ⇒ mọi biến `--dv-*`
 * mà theme `abyss` khai **thắng** bản của ta. Sản phẩm âm thầm chạy một bảng màu thứ hai
 * (`#10192c`, `rgb(91, 30, 207)`, …) chưa ai kiểm tương phản, và `npm run check:tokens`
 * **vẫn xanh** vì nó chỉ quét `src/**`. Đúng §Bẫy 1 của story, đúng thứ AD-34 §3 tồn tại
 * để chặn — và **không** phép kiểm tĩnh nào bắt được: nó chỉ tồn tại lúc chạy.
 *
 * ⚠️ `colorScheme` cố ý ĐỂ TRỐNG: `applyTheme()` đã ghi `color-scheme` lên
 * `document.documentElement` từ Story 1.4, và khai lại ở đây là dựng nguồn sự thật thứ hai
 * cho một thứ đã có chủ.
 *
 * ⚠️ `gap: 0` cũng cố ý: khe giữa hai panel là **cơ chế token** (`--panel-gap`, đảo ngược
 * giữa hai theme — AC6 của Story 1.4), và nó sống trong `dockview-theme.css`. Để dockview
 * tự thêm một khe bằng một con số cứng là có HAI khe cộng lại, với chỉ một cái đi theo theme.
 */
const auraTheme = {
  name: 'aura',
  className: 'dockview-theme-aura',
  gap: 0,
  tabGroupIndicator: 'none',
} as const

/**
 * ⚠️ `shallowRef`, không `ref`: `DockviewApi` chở tham chiếu tới DOM thật, tới emitter
 * và tới cả cây group. Bọc nó trong một proxy sâu của Vue là mời một vòng phản ứng chạy
 * qua mọi thứ đó — và chính `dockview-vue` cũng ghi thẳng luật này cho `props` của panel
 * (*"the params object carries raw dockview API instances that must NOT be made reactive"*,
 * `utils.d.ts`).
 */
const dock = shallowRef<DockviewApi | null>(null)

/**
 * Sổ vị trí đã nhớ của những panel đang ẩn (AC3, §Quyết định #3A).
 *
 * 🔴 `dockview-core@7.0.4` khai `DockviewPanelApi extends Omit<GridviewPanelApi,
 * 'setVisible' | …>` (`api/dockviewPanelApi.d.ts:21`) — tức **`setVisible` bị GỠ khỏi API
 * của panel**. Không có đường "ẩn tại chỗ". Ẩn = `removePanel`, hiện = `addPanel`.
 *
 * Và **không** ẩn bằng `width: 0`: panel vẫn trong DOM, vẫn nhận `Tab`, vẫn trong
 * vòng focus — *"ẩn hoàn toàn"* của FR17 thành một lời hứa, với mọi cổng xanh (§Bẫy 2).
 *
 * Giá của đường đúng: phải tự nhớ chỗ để trả về. Đó là map này.
 */
type RememberedSpot = { reference: PanelId; direction: PlacementDirection | 'within' }
const hidden = new Map<PanelId, RememberedSpot>()

// ═══════════════════════════════════════════════════════════════════════════════════
// Đọc lưới
// ═══════════════════════════════════════════════════════════════════════════════════

/**
 * Panel đang hiện, theo **thứ tự bố cục**: trên→dưới trước, rồi trái→phải (AC9).
 *
 * 🔴 KHÔNG dùng `api.panels` trần — thứ tự của nó là thứ tự *thêm vào*, và nó không
 * đổi khi người dùng kéo một panel sang chỗ khác. Một vòng xoay đi theo thứ tự đó nhảy
 * lung tung trên màn hình trong khi không cổng nào đỏ, và đó chính là nửa mà AC9 đòi
 * đóng (*"không theo thứ tự khai báo"*).
 *
 * ⚠️ `group.api.boundingBox` là toạ độ thật trong container. Nó `undefined` với group nổi
 * và group popout; group nổi thì xếp **sau** phần lưới — chúng không có chỗ trong trật
 * tự đọc trái→phải, và đẩy chúng lên đầu là làm vòng xoay nhảy ra khỏi lưới rồi quay lại.
 */
function visiblePanelsInLayoutOrder(): readonly string[] {
  const api = dock.value
  if (api === null) return []
  const rows = api.panels.map((panel) => {
    const box = panel.api.group.api.boundingBox
    return {
      id: panel.id,
      top: box?.top ?? Number.MAX_SAFE_INTEGER,
      left: box?.left ?? Number.MAX_SAFE_INTEGER,
    }
  })
  rows.sort((a, b) => (a.top !== b.top ? a.top - b.top : a.left - b.left))
  return rows.map((r) => r.id)
}

const isPanelId = (id: string): id is PanelId => (PANEL_IDS as readonly string[]).includes(id)

// ═══════════════════════════════════════════════════════════════════════════════════
// Dựng bố cục
// ═══════════════════════════════════════════════════════════════════════════════════

function addPanel(api: DockviewApi, id: PanelId, position?: object): IDockviewPanel {
  return api.addPanel({
    id,
    component: PANEL_COMPONENTS[id],
    tabComponent: TAB_COMPONENT,
    // ⚠️ KHOÁ `vi.json`, không chuỗi đã dịch. `PanelTab.vue` là chỗ `t()` chạy —
    // NFR16 nói mọi văn bản hiển thị sống ở `vi.json` và chỉ ở đó, kể cả khi đường đi
    // vòng qua một object `params` của thư viện.
    params: { titleKey: PANEL_TITLE_KEYS[id] },
    ...(position === undefined ? {} : { position }),
  } as Parameters<DockviewApi['addPanel']>[0])
}

/**
 * Áp một preset (AC5, AC6). không `api.clear()` trước — một preset là một bố cục TRỌN VẸN,
 * không phải một lượt sửa lên trên cái đang có.
 */
function applyPreset(presetId: string): boolean {
  const api = dock.value
  if (api === null) return false
  const preset = presetById(presetId)
  if (preset === undefined) {
    console.error(
      `[layout] preset \`${presetId}\` chua khai -- bo cuc KHONG doi. ` +
        'Danh sach khai o `src/layout/workspaceLayout.ts`.',
    )
    return false
  }
  api.clear()
  hidden.clear()
  // 🔵 Story 4.12, Task 3 — một preset MỚI là một bố cục lại từ đầu, nên sổ tầng cũ (panel
  // nào tầng đang ẩn, có đang gộp tab không) không còn nói đúng sự thật nữa. Dọn ba biến này
  // CÙNG LÚC với `hidden.clear()` ở trên — cùng một lý do, cùng một lượt.
  autoHiddenIds.clear()
  tierMerged.value = false
  mergedSpot = null
  // ⚠️ Bọc `try`, cùng kỷ luật với `restore()`/`flush()` cho đúng lớp lỗi: một `addPanel()`
  // ném giữa vòng lặp (component nội dung ném lúc mount, `position` trỏ tới một panel chưa
  // kịp thêm, …) không được để lại một bố cục dở dang rồi văng thẳng lên
  // `registry.dispatch` không bọc (`keys.ts::handle`). `api.clear()` lần hai đưa dockview
  // về một trạng thái sạch, đã biết, thay vì "nửa preset cũ, nửa preset mới".
  try {
    for (const p of preset.placements) {
      addPanel(
        api,
        p.id,
        p.reference === null ? undefined : { referencePanel: p.reference, direction: p.direction },
      )
    }
  } catch (err) {
    console.error(
      `[layout] ap preset \`${presetId}\` that bai giua chung -- roi ve mot Workspace rong. ` +
        `Nguyen nhan: ${String(err)}`,
    )
    api.clear()
    hidden.clear()
    autoHiddenIds.clear()
    tierMerged.value = false
    mergedSpot = null
    return false
  }
  currentPresetId.value = preset.id
  // 🔴 Story 4.12 (2026-09-23): đánh dấu ghi NGAY, trước khi đo tầng. Các `addPanel` ở trên chỉ
  // tới `onLayoutChange` ở microtask sau, sau cả lượt gộp/ẩn của tầng bên dưới, lúc cờ chặn ghi
  // đã bật. Đo được: áp preset ở tầng `short`/`narrow` ghi 0 lượt, nên lần mở sau vẫn về bố cục
  // cũ. Đánh dấu ở đây thì `flush()` dòng đầu của `applyTier` ghi preset nguyên vẹn, trước khi
  // tầng chạm vào dock.
  onLayoutChange()
  // Story 4.12, Task 3 — hai preset mang hai bộ ngưỡng RIÊNG (`LAYOUT_THRESHOLDS`); cùng một
  // kích thước cửa sổ có thể đọc ra hai tầng khác nhau tuỳ preset đang sống (spec I/O Matrix,
  // hàng "Preset switched at a fixed size"). `currentTier` bị đặt về `null` TRƯỚC khi đo lại —
  // không so sánh chuỗi tầng cũ với tầng mới: dock vừa được dựng lại từ đầu (`api.clear()` ở
  // trên), nên tầng "đang áp" cũ không còn mô tả đúng gì cả, kể cả khi tên tầng trùng nhau.
  currentTier.value = null
  measureAndApplyTier()
  return true
}

// ═══════════════════════════════════════════════════════════════════════════════════
// Ẩn / hiện panel — AC3
// ═══════════════════════════════════════════════════════════════════════════════════

/**
 * Anh em THẬT của panel trong cây lưới — tức nút cạnh nó trong **cùng một nhánh**, cộng
 * hướng của panel so với anh em đó. Đi cây sống ở `dockTree.ts` (thuần, không import, có
 * bộ kiểm riêng `dockTree.test.ts`) — gộp làm MỘT bộ đi cây cho cả việc tìm neo lẫn tính
 * hướng, thay vì một bản đọc cây rồi một bản hình học riêng.
 *
 * 🔴 BẢN ĐẦU DÙNG `adjacentGroupInDirection()` VÀ NÓ SAI — bắt được ở lượt đo (Task 11).
 *
 * `adjacentGroupInDirection` trả về group **cạnh nhau TRÊN MÀN HÌNH**, không phải anh em
 * trong cây. Ở lưới 2×2 thật *(`root = row[ col(Nguyên văn, Tra cứu), col(Bản dịch, Đề
 * xuất AI) ]`)*, `Tra cứu` có hai hàng xóm hình học: `Nguyên văn` ở trên **và** `Đề xuất
 * AI` ở bên phải. Chỉ cái thứ nhất là anh em thật; cái thứ hai ở một cột khác.
 *
 * Đo được: chọn nhầm `Đề xuất AI` ⇒ hiện lại `Tra cứu` cắt đôi ô của `Đề xuất AI` ở
 * **góc dưới-phải** thay vì trả nó về **góc dưới-trái**. Bốn cổng đều xanh; chỉ mắt thấy.
 *
 * ⇒ Đọc cây từ `api.toJSON()`. Đó là dữ liệu công khai, ổn định, và là **chính** thứ
 * `fromJSON` đọc lại — nên nó không phải một bản chép của trạng thái nội bộ dockview.
 */
function siblingSpotInTree(api: DockviewApi, id: PanelId): { reference: string; direction: PlacementDirection } | null {
  const grid = (api.toJSON() as unknown as { grid: SerializedGrid }).grid
  return findTreeSpot(grid, id)
}

/**
 * Ghi lại chỗ của một panel TRƯỚC khi gỡ nó.
 *
 * Hai ca, và ca thứ hai là ca dễ quên:
 *   1. panel **gộp tab** với panel khác trong cùng group ⇒ nhớ `within` + một bạn cùng
 *      group. Hiện lại phải quay về đúng group đó, không phải cắt một ô mới.
 *   2. panel một mình trong group ⇒ nhớ **anh em trong cây lưới** *(không phải hàng
 *      xóm hình học — xem [`siblingSpotInTree`])* và hướng của nó so với anh em đó.
 *
 * 🔵 SỬA 2026-09-23 (Phase 4d) — câu cũ ở đây bắt đọc hướng từ **hình học thật**
 * (`group.api.boundingBox`) "chứ không suy ra từ `Orientation` của nhánh", với lý do
 * `Orientation` đảo ở mỗi tầng lồng nhau nên một lượt suy luận sai cho ra đúng nhánh, sai
 * bên. Đó CHÍNH LÀ lỗi đo được ở Phase 4d: `boundingBox` là toạ độ DOM, `0` cho TỚI lượt
 * `measureAndApplyTier()` đầu tiên trong `onReady` VÀ trong khi Workspace còn ẩn dưới
 * `<KeepAlive>` — `dx = dy = 0` đọc thành `'right'` MỌI LẦN, nên hiện lại một panel đơn độc
 * (đúng ca 2 này) luôn đặt nó bên phải neo, kể cả khi nó vốn ở dưới. Hình học "thật" tệ hơn
 * suy luận từ `Orientation` đúng vào lúc cần nó nhất — trước khi dock được đo lần đầu.
 * `dockTree.ts::findTreeSpot` tính hướng bằng ĐỘ SÂU của nhánh trong `api.toJSON()`, dữ liệu
 * không phụ thuộc kích thước container; luật xen kẽ tầng được canh bởi `dockTree.test.ts`
 * (đối chứng: gỡ vế đảo hướng, ca lồng ≥ 3 tầng đỏ) thay vì bởi mắt.
 *
 * ⚠️ Trả `null` khi không tìm được neo nào — tức panel này là panel **duy nhất** đang
 * hiện. Ẩn nốt nó là một Workspace rỗng hoàn toàn, và AC3 nói *"các panel CÒN LẠI lấp đầy
 * chỗ trống"* — không có panel còn lại thì mệnh đề đó vô nghĩa. Từ chối, và nói ra.
 */
function rememberSpot(api: DockviewApi, panel: IDockviewPanel): RememberedSpot | null {
  const tabbed = panel.api.group.panels.find((p) => p.id !== panel.id)
  if (tabbed !== undefined && isPanelId(tabbed.id)) {
    return { reference: tabbed.id, direction: 'within' }
  }
  const spot = siblingSpotInTree(api, panel.id as PanelId)
  if (spot === null || !isPanelId(spot.reference)) return null
  return { reference: spot.reference, direction: spot.direction }
}

function hidePanel(id: PanelId): boolean {
  const api = dock.value
  if (api === null) return false
  const panel = api.getPanel(id)
  if (panel === undefined) return false
  const spot = rememberSpot(api, panel)
  if (spot === null) {
    console.error(
      `[layout] \`${id}\` la panel DUY NHAT dang hien -- an no cho ra mot Workspace rong, ` +
        'va AC3 noi "cac panel CON LAI lap day cho trong". Thao tac bi tu choi.',
    )
    return false
  }
  hidden.set(id, spot)
  // 🔴 `removePanel` gỡ panel khỏi DOM HOÀN TOÀN, và dockview tự cho các panel còn lại lấp
  // đầy chỗ trống — đúng chữ của FR17. Không phải tự tính lại kích thước.
  api.removePanel(panel)
  return true
}

function showPanel(id: PanelId): boolean {
  const api = dock.value
  if (api === null) return false
  const spot = hidden.get(id)
  hidden.delete(id)
  /**
   * ⚠️ Neo đã nhớ có thể không còn: người dùng ẩn `panel.lookup` (neo vào
   * `panel.ai_translation`) rồi ẩn nốt `panel.ai_translation`. Rơi về preset mặc định là
   * quá tay — nó vứt cả bố cục người dùng vừa sắp. Rơi về *"đặt bên phải panel đầu tiên
   * đang hiện"* giữ được phần còn lại và vẫn cho panel một chỗ nhìn thấy được.
   */
  const anchorId = spot !== undefined && api.getPanel(spot.reference) !== undefined
    ? spot.reference
    : (visiblePanelsInLayoutOrder()[0] as PanelId | undefined)
  if (anchorId === undefined) {
    // Không neo nào ⇒ lưới rỗng ⇒ panel này chiếm cả lưới. Hợp lệ, không phải lỗi.
    addPanel(api, id)
    return true
  }
  const direction = spot !== undefined && api.getPanel(spot.reference) !== undefined
    ? spot.direction
    : 'right'
  addPanel(api, id, { referencePanel: anchorId, direction })
  return true
}

/**
 * 🔴 SAU MỘT LƯỢT ẨN/HIỆN, FOCUS KHÔNG ĐƯỢC RƠI VỀ `body` — bắt được lúc đo (Task 11).
 *
 * `removePanel()` gỡ đúng cái phần tử đang giữ focus; `addPanel()` tái cấu trúc group nên
 * dockview đỗ rồi dựng lại DOM của những panel còn lại. Cả hai đường đều để
 * `document.activeElement` về `document.body`. Lượt đo đọc được đúng chữ `BODY` ở cả hai.
 *
 * Đó là vi phạm **AC4 của Story 1.6** (*"focus không bao giờ rơi về `body`"*) và AD-34 §2 —
 * và **không** chốt nào kêu: `armBodyGuard` của `focus.ts` chỉ chạy sau một `enter()`,
 * còn ở đây không ai gọi `enter()` cả. Một lỗ im lặng giữa hai cơ chế đều đúng.
 *
 * ⚠️ CHỈ can thiệp khi focus THẬT SỰ đã mất. Nếu người dùng vẫn đang đứng trong một panel
 * khác thì dời focus giùm họ là cướp chỗ — đúng thứ mà `onDidActivePanelChange` vừa phải
 * lọc `origin === 'user'` để tránh.
 *
 * ⚠️ `requestAnimationFrame` vì Vue vá DOM ở microtask kế tiếp: panel vừa hiện chưa
 * `declareFocus()` tại thời điểm `addPanel()` trả về.
 *
 * ⚠️ HUỶ lượt hẹn TRƯỚC đó, bắt được lúc code review. Không huỷ thì `layout.toggle_*` bấm
 * liên tiếp (khả dĩ khi Story 1.21 gán phím cho hơn một cái) xếp chồng nhiều callback cùng
 * đọc/ghi `document.activeElement` — callback chạy SAU thắng bất kể nó có phải panel người
 * dùng vừa gọi tên hay không. Một hàng đợi độ dài một là đủ: chỉ lượt ẩn/hiện GẦN NHẤT còn
 * ý nghĩa.
 */
let pendingFocusRaf: number | null = null
function restoreFocusIfLost(preferred: PanelId): void {
  if (pendingFocusRaf !== null) cancelAnimationFrame(pendingFocusRaf)
  pendingFocusRaf = requestAnimationFrame(() => {
    pendingFocusRaf = null
    const active = document.activeElement
    if (active !== null && active !== document.body) return
    if (enterFocus(preferred)) return
    for (const id of visiblePanelsInLayoutOrder()) {
      if (isPanelId(id) && enterFocus(id)) return
    }
    console.error(
      '[layout] sau mot luot an/hien panel, khong diem vao focus nao nhan duoc focus — ' +
        'focus dang o `body`. AC4 cua Story 1.6 noi dieu do KHONG duoc xay ra.',
    )
  })
}

/** Handler thật của bốn command `layout.toggle_*` (AC3). */
function togglePanel(panelId: string): boolean {
  if (!isPanelId(panelId)) {
    console.error(`[layout] \`${panelId}\` khong phai mot panel cua Workspace -- bo qua.`)
    return false
  }
  const api = dock.value
  if (api === null) return false
  const showing = api.getPanel(panelId) === undefined
  const done = showing ? showPanel(panelId) : hidePanel(panelId)
  if (!done) return false
  /**
   * Hiện ⇒ ưu tiên chính panel vừa hiện: người dùng vừa gọi tên nó.
   * Ẩn ⇒ ưu tiên panel đầu tiên trong thứ tự bố cục còn lại — không đoán một panel
   * "gần" cái vừa mất, vì "gần" sau một lượt tái cấu trúc lưới không còn nghĩa gì.
   */
  // ⚠️ `.at(0)`, KHÔNG `[0]`: danh sách này RỖNG được (`dock.value === null`, hoặc mọi panel
  // đang ẩn), nên `?? panelId` ngay dưới là nhánh thật. Và một chú thích `: string | undefined`
  // trên `[0]` KHÔNG đủ — TypeScript thu hẹp một `const` theo kiểu của giá trị gán, nên chú
  // thích bị bỏ qua và phép kiểm lại thành "mã dư". `.at()` trả `T | undefined` từ chính chữ
  // ký của nó, nên kiểu nói thật ở nguồn thay vì ở một lời khai cạnh nó.
  const fallback = visiblePanelsInLayoutOrder().at(0)
  restoreFocusIfLost(showing ? panelId : ((fallback ?? panelId) as PanelId))
  return true
}

// ═══════════════════════════════════════════════════════════════════════════════════
// Tầng bố cục tự động theo kích thước cửa sổ — Story 4.12, Task 3 · Task 5
// ═══════════════════════════════════════════════════════════════════════════════════

/**
 * 🔴 CỜ CHẶN GHI — chỗ Phase 2 tồn tại để giữ đúng §Always của spec: *"một lượt đổi tầng tự
 * động không bao giờ được ghi thành bố cục của người dùng"*.
 *
 * [`onLayoutChange`] (chỗ nối với lịch ghi nợ, xem doc-comment ở đó) đọc cờ này ĐẦU TIÊN,
 * trước khi đánh dấu lịch "bẩn". Mọi lượt gọi `hidePanel`/`showPanel`/`api.removePanel`/
 * `api.addPanel` mà TẦNG tự động thực hiện phải bọc `suppressPersist = true` NGAY TRƯỚC và
 * [`endSuppressPersist`] NGAY SAU, đồng bộ — không `await` chen giữa.
 *
 * 🔵 SỬA 2026-09-23 (Story 4.12, sau Phase 4b) — câu cũ ở đây khẳng định `dockview` bắn
 * `onDidLayoutChange` ĐỒNG BỘ trong cùng lượt mutate. SAI: đọc `dockview-core` 7.0.4,
 * `baseComponentGridview.js` gán `onDidLayoutChange = new AsapEvent().onEvent`, và
 * `AsapEvent.fire()` (`events.js`) gom mọi lượt bắn trong một tick vào MỘT `queueMicrotask`.
 * Với lượt tắt cờ đồng bộ cũ, cờ đã về `false` trước khi sự kiện tới, nên nó CHƯA TỪNG chặn
 * được gì. Phase 4b gỡ riêng vế này và cả bảy ca vẫn xanh. Đo được hai hệ quả: `full → short`
 * ghi bố cục đã gộp, và mọi chiều nới cửa sổ trở lại ghi bố cục tầng vừa dựng lại.
 */
let suppressPersist = false

/**
 * Tắt cờ chặn ghi SAU microtask của `dockview`, không phải ngay lập tức.
 *
 * Microtask chạy theo thứ tự FIFO. `AsapEvent` xếp microtask của nó ngay lượt mutate đầu tiên
 * trong tick, tức TRƯỚC lượt xếp ở đây, nên [`onLayoutChange`] luôn thấy cờ còn `true`. Nếu
 * một bản `dockview` sau đổi `AsapEvent` sang một nhịp khác (`setTimeout`, rAF), thứ tự này
 * hết đúng. Các ca `full → short` và nới-trở-lại trong `workspaceDockTier.test.ts` sẽ đỏ.
 */
function endSuppressPersist(): void {
  queueMicrotask(() => {
    suppressPersist = false
  })
}

/** Tầng ĐÃ ÁP gần nhất. `null` = chưa đo lần nào (trước lượt đầu tiên ở `onReady`). */
const currentTier = shallowRef<LayoutTier | null>(null)

/**
 * Preset đang sống theo cách hiểu của tầng — preset gần nhất được ÁP THẬT qua
 * [`applyPreset`], không phải "preset đã lưu trên đĩa".
 *
 * ⚠️ Một bố cục đã lưu tuỳ ý (khôi phục qua `fromJSON` ở [`restore`]) không PHẢI một trong
 * hai preset — nó có thể là bất kỳ sự sắp xếp nào người dùng từng kéo tay, và không có cách
 * suy ngược hình dạng đó ra một `PresetId`. Biến này vì vậy GIỮ NGUYÊN giá trị mặc định cho
 * tới lượt người dùng tự gọi `layout.preset_*` — một xấp xỉ có chủ, cùng luật dự phòng
 * `presetId` lạ mà `layoutTierFor` đã khai.
 */
const currentPresetId = shallowRef<PresetId>(DEFAULT_PRESET_ID)

/** Có đang gộp `panel.ai_translation` vào NHÓM của `panel.lookup` (tầng `short`) không. */
const tierMerged = shallowRef(false)
/** Chỗ của `panel.ai_translation` TRƯỚC lượt gộp — để [`undoMerge`] trả đúng chỗ cũ. */
let mergedSpot: RememberedSpot | null = null

/**
 * Panel đang ẩn VÌ TẦNG, không vì người dùng bấm `layout.toggle_*`. Tập con của khoá trong
 * [`hidden`] — mọi panel ở đây LUÔN có mặt trong `hidden`, chiều ngược thì không.
 *
 * Đây là câu trả lời cho "một lượt ẩn tay và một lượt ẩn tự động phân biệt được bằng gì lúc
 * chạy" (Phase 2 phải để lại cho Phase 4, xem phase file): một `id` nằm trong tập này ⇔ lượt
 * ẩn GẦN NHẤT của nó đi qua [`autoHide`], không phải qua [`hidePanel`] trực tiếp từ
 * [`togglePanel`].
 */
const autoHiddenIds = new Set<PanelId>()

/** Đọc một token chrome (Story 1.4) LÚC CHẠY — không viết số cứng (spec §Always). */
function readChromeToken(name: string): number {
  return parseFloat(window.getComputedStyle(document.documentElement).getPropertyValue(name))
}

/**
 * Diện tích làm việc THẬT — `width` bằng thẳng chiều rộng cửa sổ, `height` đã trừ hai token
 * chrome (`--space-titlebar-height` · `--space-status-height`, Story 1.4, ghi lên `:root`
 * bởi `tokens/index.ts::applyTheme`).
 *
 * ⚠️ Nếu một token đọc lỗi (`getPropertyValue` trả chuỗi rỗng ⇒ `parseFloat('') === NaN`),
 * phép trừ lan `NaN` sang `height` — và `layoutTierFor` đã có luật riêng cho ca đó (trả
 * `null`, không phải `'full'`); xem contract Phase 1 để lại.
 */
function computeWorkArea(): WorkArea {
  const titlebar = readChromeToken('--space-titlebar-height')
  const status = readChromeToken('--space-status-height')
  return { width: window.innerWidth, height: window.innerHeight - titlebar - status }
}

/** Ẩn một panel VÌ TẦNG — bọc cờ chặn ghi, ghi vào sổ "ẩn vì tầng" khi thành công. */
function autoHide(id: PanelId): void {
  suppressPersist = true
  const ok = hidePanel(id)
  endSuppressPersist()
  if (ok) autoHiddenIds.add(id)
}

/** Hiện lại một panel TẦNG từng ẩn — bọc cờ chặn ghi, xoá khỏi sổ "ẩn vì tầng". */
function autoShow(id: PanelId): void {
  suppressPersist = true
  const ok = showPanel(id)
  endSuppressPersist()
  if (ok) autoHiddenIds.delete(id)
}

/**
 * Đồng bộ sổ "ẩn vì tầng" với thực tế — một panel tầng từng ẩn có thể đã được người dùng tự
 * hiện lại bằng `layout.toggle_*` (đi thẳng qua [`showPanel`], không qua [`autoShow`]). Gọi
 * TRƯỚC mỗi lượt áp tầng để [`nextAutoRestoreCandidate`] không đọc một sổ đã cũ.
 */
function reconcileAutoHidden(): void {
  for (const id of [...autoHiddenIds]) {
    if (!hidden.has(id)) autoHiddenIds.delete(id)
  }
}

/**
 * Panel có mức ưu tiên trả lại CAO NHẤT trong số những panel TẦNG đang ẩn — duyệt
 * [`SACRIFICE_ORDER`] NGƯỢC, cùng thứ tự với `nextToRestore` của tầng thuần, nhưng chỉ nhận
 * panel nằm trong [`autoHiddenIds`].
 *
 * ⚠️ Khác `nextToRestore` ở ĐÚNG một chỗ, và đó là chỗ quan trọng: nó không đọc tập panel
 * đang ẩn THẬT (`hidden`, thứ có thể lẫn một panel người dùng tự ẩn tay), nó chỉ đọc sổ CỦA
 * MÌNH. Đọc nhầm sang `hidden` sẽ làm một lượt "mở cửa sổ rộng ra" cố trả lại một panel
 * người dùng đã ẩn tay — đúng điều spec cấm ("lựa chọn thủ công không bị tầng ghi đè").
 */
function nextAutoRestoreCandidate(): PanelId | null {
  for (let i = SACRIFICE_ORDER.length - 1; i >= 0; i -= 1) {
    const id = SACRIFICE_ORDER[i] as PanelId
    if (autoHiddenIds.has(id)) return id
  }
  return null
}

/** Hy sinh theo [`SACRIFICE_ORDER`] tới khi chỉ còn lưới — tầng `narrow`/`unsupported`. */
function applyGridOnlySacrifice(): void {
  for (;;) {
    const visible = visiblePanelsInLayoutOrder()
    if (visible.length <= 1) return
    const id = nextToSacrifice(visible)
    if (id === null) return
    autoHide(id as PanelId)
  }
}

/** Trả lại MỌI panel tầng từng ẩn (và chỉ những cái đó) — tầng `full`/`short`. */
function restoreAllAutoHidden(): void {
  for (;;) {
    const id = nextAutoRestoreCandidate()
    if (id === null) return
    autoShow(id)
  }
}

/**
 * Gộp `panel.ai_translation` vào group của `panel.lookup` — tầng `short` (Task 5). Dùng lại
 * đúng nhánh `within` mà [`rememberSpot`] đã có (§Code Map) thay vì tự dựng một dải tab thứ
 * ba (`GridPanel.vue` và `LookupPanel.vue` đã mỗi cái hand-copy một lần).
 *
 * ⚠️ Từ chối LẶNG LẼ khi một trong hai panel đang KHÔNG hiện (người dùng đã ẩn tay nó) — ép
 * nó hiện lên chỉ để gộp là ghi đè một lựa chọn thủ công, đúng điều spec cấm.
 */
function applyMerge(): void {
  const api = dock.value
  if (api === null || tierMerged.value) return
  const lookupPanel = api.getPanel('panel.lookup')
  const aiPanel = api.getPanel('panel.ai_translation')
  if (lookupPanel === undefined || aiPanel === undefined) return
  if (aiPanel.api.group.panels.some((p) => p.id === lookupPanel.id)) {
    // Already tabbed together by the user, not by the tier: leave `tierMerged` false so
    // `undoMerge` never splits a group the user made, and persist keeps the user's shape.
    return
  }
  const spot = rememberSpot(api, aiPanel)
  suppressPersist = true
  api.removePanel(aiPanel)
  addPanel(api, 'panel.ai_translation', { referencePanel: 'panel.lookup', direction: 'within' })
  endSuppressPersist()
  mergedSpot = spot
  tierMerged.value = true
}

/** Gỡ gộp — trả `panel.ai_translation` về đúng chỗ TRƯỚC lượt gộp ([`mergedSpot`]). */
function undoMerge(): void {
  if (!tierMerged.value) return
  tierMerged.value = false
  const api = dock.value
  if (api === null) return
  const aiPanel = api.getPanel('panel.ai_translation')
  if (aiPanel === undefined) {
    mergedSpot = null
    return
  }
  const spot = mergedSpot
  const anchorStillThere = spot !== null && api.getPanel(spot.reference) !== undefined
  suppressPersist = true
  api.removePanel(aiPanel)
  if (spot !== null && anchorStillThere) {
    addPanel(api, 'panel.ai_translation', { referencePanel: spot.reference, direction: spot.direction })
  } else {
    // Neo cũ không còn (đã bị ẩn/xoá trong lúc gộp) — cùng đường dự phòng của `showPanel`:
    // đặt bên phải panel đầu tiên đang hiện, hoặc chiếm cả lưới nếu không còn panel nào.
    const anchor = visiblePanelsInLayoutOrder()[0] as PanelId | undefined
    if (anchor === undefined) addPanel(api, 'panel.ai_translation')
    else addPanel(api, 'panel.ai_translation', { referencePanel: anchor, direction: 'right' })
  }
  endSuppressPersist()
  mergedSpot = null
}

/**
 * Áp một TẦNG lên dock THẬT — Task 3. Chỉ gọi khi tầng vừa ĐỔI (xem
 * [`measureAndApplyTier`]); mọi nhánh dùng lại nguyên [`hidePanel`]/[`showPanel`] qua
 * [`autoHide`]/[`autoShow`], không một cơ chế ẩn/hiện thứ hai.
 *
 * 🔴 `flush()` NGAY DÒNG ĐẦU — và đây KHÔNG phải một lượt ghi thừa.
 *
 * `suppressPersist` chỉ chặn được lượt ĐÁNH DẤU BẨN MỚI; nó không xoá một lượt đã bẩn TỪ
 * TRƯỚC. Nếu một hành động THẬT của người dùng (vd. vừa `applyPreset()` qua lệnh bàn phím)
 * để lịch ghi đang "bẩn" và đợi hết idle 500 ms, rồi TẦNG mới mutate tiếp (bọc
 * `suppressPersist`, không đánh dấu bẩn thêm) — lượt `flush()` SAU CÙNG, khi nó tự bắn, vẫn
 * đọc `api.toJSON()` ở thời điểm nó chạy, tức SAU cả lượt tầng vừa làm. Không gọi `flush()`
 * ở đây thì bố cục ghi xuống đĩa là bố cục ĐÃ BỊ TẦNG SỬA, không phải bố cục người dùng vừa
 * chọn — đúng thứ §Always cấm, chỉ đi vòng qua một cửa khác. Gọi `flush()` trước khi TẦNG
 * chạm vào dock chốt lại đúng ảnh chụp (nếu có gì đang bẩn) TRƯỚC khi tầng mutate; `flush()`
 * tự no-op khi lịch đang sạch, nên lượt gọi này rẻ ở đường thường (resize không đổi gì khác).
 *
 * 🔵 [`syncLayoutTier`] NGAY SAU `flush()` — Story 4.12 Phase 3a (sửa 2026-09-23). Nó không
 * chạm `api`/dock (chỉ chạm `ref` của ngăn kéo Tra cứu), nên nó không tranh chỗ "dòng đầu"
 * với `flush()` ở trên; điều bắt buộc là nó chạy TRƯỚC mọi nhánh dưới đây — đặc biệt trước
 * `restoreAllAutoHidden()`, chỗ có thể `addPanel` lại `panel.lookup` ĐỒNG BỘ. Xem doc-comment
 * đầu `lookupDrawerState.ts` cho lý lẽ đầy đủ.
 */
function applyTier(tier: LayoutTier): void {
  flush()
  syncLayoutTier(tier)
  reconcileAutoHidden()
  if (tier === 'narrow' || tier === 'unsupported') {
    // Cùng bố cục panel cho cả hai — chỉ lưới, xem doc-comment `LayoutTier` ở tầng thuần.
    undoMerge()
    applyGridOnlySacrifice()
  } else if (tier === 'short') {
    restoreAllAutoHidden()
    applyMerge()
  } else {
    // 'full'
    undoMerge()
    restoreAllAutoHidden()
  }
  // Cùng kỷ luật với `togglePanel`: `restoreFocusIfLost` chỉ can thiệp khi focus THẬT SỰ đã
  // mất, nên gọi vô điều kiện ở đây là an toàn — nó tự no-op khi người dùng vẫn đứng nơi khác.
  const fallback = visiblePanelsInLayoutOrder().at(0)
  if (fallback !== undefined) restoreFocusIfLost(fallback as PanelId)
}

/**
 * Đo và áp tầng — Task 3. Gọi lúc `onReady` (tầng khởi động) và ở mỗi sự kiện `resize`.
 *
 * 🔴 `null` KHÔNG BAO GIỜ được đọc thành một tầng — đúng hợp đồng Phase 1 để lại
 * (`workspaceLayout.ts::layoutTierFor` doc-comment): GIỮ NGUYÊN tầng đang áp, chỉ ghi một
 * chẩn đoán nêu nguyên nhân. Không thử lại ngay — sự kiện `resize` hoặc lượt đổi preset kế
 * tiếp sẽ tự đo lại.
 */
function measureAndApplyTier(): void {
  const workArea = computeWorkArea()
  const tier = layoutTierFor(workArea, currentPresetId.value)
  if (tier === null) {
    console.error(
      `[layout] khong do duoc dien tich lam viec (cua so ${window.innerWidth}x${window.innerHeight}) ` +
        `-- giu nguyen tang dang ap (${currentTier.value ?? 'chua ap tang nao'}).`,
    )
    return
  }
  if (tier === currentTier.value) return
  currentTier.value = tier
  applyTier(tier)
  emit('tier-change', tier)
}

function onWindowResize(): void {
  measureAndApplyTier()
}

// ═══════════════════════════════════════════════════════════════════════════════════
// Lưu và khôi phục — AC4
// ═══════════════════════════════════════════════════════════════════════════════════

const schedule = createWriteSchedule()
let timer: ReturnType<typeof setTimeout> | null = null

/**
 * JSON sẽ ghi xuống đĩa cho lượt `flush()` này — [`api.toJSON()`] thẳng, TRỪ khi đang gộp VÀ
 * còn nhớ chỗ trước gộp: khi đó đi qua [`unmergeForPersist`] để đĩa luôn chở hình dạng CHƯA
 * GỘP (phán quyết Ice 2026-09-23, xem doc-comment [`onLayoutChange`]).
 *
 * 🔵 SỬA 2026-09-23 (orchestrator) — bản Phase 4e loại riêng `within` và ghi JSON thẳng cho
 * ca đó, kèm lời khai "cùng hành vi trước Phase 4e". Lời khai đó sai: trước 4e, vế
 * `tierMerged.value` chặn hẳn lượt ghi, còn JSON thẳng lúc đang gộp chính là nhóm đã gộp. Nay
 * [`unmergeForPersist`] nhận cả `within`, nên không còn nhánh nào ghi hình dạng đã gộp.
 */
function jsonForPersist(api: DockviewApi): unknown {
  const json = api.toJSON() as unknown as SerializedDockJSON
  if (tierMerged.value && mergedSpot !== null) {
    return unmergeForPersist(json, 'panel.ai_translation', mergedSpot)
  }
  return json
}

function flush(): void {
  if (timer !== null) {
    clearTimeout(timer)
    timer = null
  }
  if (!schedule.isDirty()) return
  const api = dock.value
  const now = Date.now()
  schedule.onWrite(now)
  if (api === null) return
  try {
    emit('persist', JSON.stringify(jsonForPersist(api)))
  } catch (err) {
    // Không ném: một bố cục không serialize được không phải lý do để giết thao tác
    // mà người dùng vừa làm. Nó chỉ có nghĩa là phiên sau mở bằng preset mặc định.
    console.error(`[layout] khong serialize duoc bo cuc -- luot luu nay bo qua. ${String(err)}`)
  }
}

/**
 * Một lượt `onDidLayoutChange`. Xem `src/layout/writeSchedule.ts` cho lý lẽ đầy đủ —
 * tóm tắt: idle 500 ms **cộng** một trần cứng 5 s **không reset bởi sự kiện kế tiếp**.
 *
 * 🔴 DÒNG ĐẦU TIÊN LÀ CỜ CHẶN GHI CỦA STORY 4.12 — xem doc-comment đầy đủ ở
 * [`suppressPersist`] (§Tầng bố cục tự động). Mọi lượt `hidePanel`/`showPanel`/
 * `api.removePanel`/`api.addPanel` mà TẦNG tự động gọi đều tới đây, ở microtask ngay sau
 * lượt mutate (🔵 2026-09-23: không phải đồng bộ, xem [`endSuppressPersist`]). Không có vế
 * này thì một lượt cửa sổ co lại hoặc nới ra sẽ TỰ GHI đè bố cục người dùng đã lưu, im lặng,
 * và không cách nào lấy lại (spec §Always, phase file §Phase 2).
 *
 * 🔴 VÀ VẾ `autoHiddenIds.size > 0` — CỬA THỨ HAI VÀO CÙNG CHỖ MẤT DỮ LIỆU ĐÓ.
 *
 * `suppressPersist` chỉ bọc quanh chính lượt mutate của tầng và tắt sau microtask kế tiếp. Nhưng khi
 * tầng đã ẩn một panel, `api.toJSON()` KHÔNG CÒN panel ấy (dockview 7.0.4 không có
 * `setVisible` cho panel — "ẩn" là `removePanel` thật). Nên một thao tác THẬT của người dùng
 * sau đó — kéo một sash, bật/tắt một panel bằng tay — đi qua đây với cờ đã tắt, đánh dấu bẩn
 * bình thường, và `flush()` tuần tự hoá một cây ĐANG THIẾU panel. `hidden` là bộ nhớ trong,
 * chết theo phiên; phiên sau mở ra chỉ còn lưới và không gì mang panel kia về.
 *
 * Ice chốt 2026-09-22: **khi còn panel bị TẦNG ẩn thì không ghi gì cả.** Cái mất là vị trí
 * sash người dùng kéo ở bậc hẹp — mở rộng lại vẫn ra bố cục cũ. Cái giữ được là không bao
 * giờ mất một panel. Hai phương án kia (lắp tạm panel ẩn vào rồi tuần tự hoá, hoặc lưu kèm
 * tập panel bị ẩn) đều giữ được cả sash lẫn panel nhưng một cái phải mutate dock để đọc nó,
 * cái kia đổi hình dạng giá trị đã lưu trong `AppConfig` — cả hai to hơn story này.
 *
 * 🔵 2026-09-23 (Story 4.12, Phase 4e) — VẾ `tierMerged.value` từng đứng Ở ĐÂY (thêm sau
 * Phase 4b) và chặn đứng MỌI lượt ghi trong lúc đang gộp, cùng lý do đo được ở vế
 * `autoHiddenIds` bên trên: gộp tab không ẩn panel nào (`autoHiddenIds` vẫn rỗng), nên thiếu
 * nó thì một lượt bật/tắt tay + `beforeunload` ghi thẳng xuống một NHÓM chứa cả
 * `panel.ai_translation` lẫn `panel.lookup` — phiên sau mở ở cỡ đầy đủ thì `tierMerged` là
 * `false` nên [`undoMerge`] không bao giờ chạy, và bố cục người dùng mất hẳn.
 *
 * Ice chốt LẠI 2026-09-23 ("giữ gộp, lưu dạng chưa gộp"): chặn TUYỆT ĐỐI đó quá tay — một
 * lượt kéo sash hay bật/tắt panel THẬT của người dùng trong lúc đang gộp cũng bị nuốt, không
 * khác gì trạng thái trước khi có `writeSchedule.ts`. Luật MỚI: TRONG LÚC gộp, một thay đổi
 * bố cục THẬT vẫn được ghi — nhưng ghi dưới dạng CHƯA GỘP, như thể lượt gộp của tầng chưa từng
 * xảy ra. [`jsonForPersist`] (ngay trên [`flush`]) là nơi làm việc đó: khi `tierMerged.value`
 * và còn nhớ chỗ trước gộp (`mergedSpot`), nó gọi `dockTree.ts::unmergeForPersist` tách
 * `panel.ai_translation` ra khỏi nhóm chung, đặt lại đúng chỗ đã nhớ, TRƯỚC khi tuần tự hoá —
 * mọi thứ KHÁC người dùng vừa đổi (sash, panel khác) vẫn nguyên trong JSON đó.
 *
 * Cửa chặn CHÍNH LƯỢT GỘP/GỠ GỘP của tầng tự nó thì KHÔNG đổi — đó vẫn là việc của
 * `suppressPersist`/[`endSuppressPersist`] (§Always: một lượt đổi tầng tự động không bao giờ
 * được ghi thành bố cục của người dùng), không phải của vế này. Guard ở đây giờ chỉ còn hai
 * vế, giống hệt trước khi Phase 4b thêm vế thứ ba.
 *
 * ⚠️ Vế `autoHiddenIds` dựa vào một sự thật đã đo: `schedule.onChange` CHỈ được gọi từ đây, và
 * `flush()` thoát sớm khi `!schedule.isDirty()`. Nên chặn ở đây cũng chặn luôn lượt
 * `flush()` thẳng của `onDeactivated`. Nếu một story sau thêm một chỗ gọi `onChange` thứ
 * hai, vế này hở và phải xét lại.
 */
function onLayoutChange(): void {
  if (suppressPersist || autoHiddenIds.size > 0) return
  const due = schedule.onChange(Date.now())
  if (timer !== null) clearTimeout(timer)
  timer = setTimeout(flush, Math.max(0, due - Date.now()))
}

/**
 * 🔴 JSON HỎNG ⇒ RƠI VỀ PRESET MẶC ĐỊNH, KHÔNG CỬA SỔ TRẮNG (AC4).
 *
 * `fromJSON` **NÉM** với dữ liệu sai hình dạng, và `WorkspaceMode` được dựng sau `mount()`
 * — nên một lần ném ở đây giết cả chế độ. Cùng lớp lỗi mà `bindingsAreUsable()`
 * (`commands/index.ts` §Bẫy 5) và khối `try` quanh `installCommands` (`main.ts`) đã chặn,
 * chỉ khác nguồn: ở đó là hợp âm sửa tay trong `global.db`, ở đây là bố cục sửa tay trong
 * cùng cái kho đó.
 *
 * ⇒ Dùng lại đúng khuôn: `try` → `console.error` **nêu đích danh** → `api.clear()` →
 * dựng preset mặc định. Không nuốt lỗi im lặng, không để người dùng nhìn một cửa sổ
 * trắng và mất luôn đường vào để sửa chính cái làm hỏng.
 */
function restore(api: DockviewApi, saved: string): void {
  if (saved.trim() === '') {
    applyPreset(DEFAULT_PRESET_ID)
    return
  }
  try {
    api.fromJSON(JSON.parse(saved))
  } catch (err) {
    console.error(
      '[layout] bo cuc da luu KHONG dung lai duoc -- roi ve preset mac dinh (luoi 2x2). ' +
        'Bo cuc cu coi nhu mat; KHONG co gi khac trong kho bi dung toi. ' +
        `Nguyen nhan: ${String(err)}`,
    )
    api.clear()
    applyPreset(DEFAULT_PRESET_ID)
    return
  }
  /**
   * ⚠️ `fromJSON` không ném với một JSON **hợp lệ về hình dạng nhưng rỗng** — một
   * `{"grid":{"root":{"type":"branch","data":[]},…},"panels":{}}` cho ra một Workspace
   * KHÔNG panel nào, tức đúng cái "cửa sổ trắng" mà khối trên vừa chặn, chỉ đi bằng cửa
   * khác. Kiểm hậu điều kiện thay vì tin vào việc ném.
   */
  if (api.panels.length === 0) {
    console.error('[layout] bo cuc da luu dung ra KHONG panel nao -- roi ve preset mac dinh.')
    api.clear()
    applyPreset(DEFAULT_PRESET_ID)
    return
  }
  // Panel vắng mặt trong bố cục đã lưu ⇒ nó đang ẩn. Đừng dựng lại nó: người dùng đã
  // chọn ẩn, và AC4 nói bố cục khôi phục **nguyên trạng**.
  //
  // ⚠️ `{ reference: id, direction: 'right' }` là một sổ vị trí TỰ THAM CHIẾU, có chủ ý —
  // bắt được ở lượt code review. Ta đã mất chỗ thật của panel này (nó không nằm trong JSON
  // đã lưu), nên không có neo nào để nhớ. Đặt `reference` bằng chính `id` của nó bảo đảm
  // `api.getPanel(spot.reference)` ở `showPanel()` LUÔN trả `undefined` (một panel ẩn thì
  // không có mặt trong dock) — tức cưỡng ép `showPanel()` rơi đúng vào nhánh dự phòng
  // "neo đã nhớ không còn": đặt bên phải panel đầu tiên đang hiện. Đừng "dọn" sentinel
  // này bằng một `reference` khác — nó phải trỏ vào chính nó để nhánh dự phòng đó chạy.
  for (const id of PANEL_IDS) {
    if (api.getPanel(id) === undefined) hidden.set(id, { reference: id, direction: 'right' })
  }
}

// ═══════════════════════════════════════════════════════════════════════════════════
// Vòng đời
// ═══════════════════════════════════════════════════════════════════════════════════

const disposables: { dispose: () => void }[] = []

function onReady(event: DockviewReadyEvent): void {
  const api = event.api
  dock.value = api
  restore(api, props.savedLayout)

  /**
   * 🔴 DỜI FOCUS DOM **TƯỜNG MINH** KHI ĐỔI PANEL — AD-34 §2, UX-DR7, AC9.
   *
   * *"Chuyển panel phải dời focus DOM tường minh"*. Bấm một tab dockview đổi `activePanel`
   * và vẽ lại tab bar, nhưng nó **không** bảo đảm `document.activeElement` đi theo —
   * hành vi focus mặc định của trình duyệt để focus ở chính cái tab vừa bấm, tức ngoài
   * thân panel. Hệ quả: `focus.next_panel` bấm ngay sau đó tính vòng từ một chỗ khác với
   * chỗ người dùng nghĩ mình đang đứng, và vạch tiêu điểm 2px không sáng ở đâu cả.
   *
   * ⚠️ MỘT chỗ nghe cho cả ba panel, không phải một handler trên mỗi tab. `PanelTab.vue`
   * vì vậy không có `@click` nào — xem doc-comment ở đó.
   *
   * ⚠️ `enterFocus` tự KÊU khi trượt (`focus.ts`) và không bao giờ ném, nên không cần
   * bọc `try` ở đây.
   */
  disposables.push(
    api.onDidActivePanelChange((e) => {
      /**
       * 🔴 CHỈ `origin === 'user'` — bắt được lúc đo (Task 11), và nó là hai lỗi trong một.
       *
       * `onDidActivePanelChange` bắn **cả** khi chính ta gọi `addPanel` lúc dựng preset
       * hay lúc hiện lại một panel. Ở thời điểm đó component Vue của panel **chưa mount**,
       * nên nó chưa `declareFocus()` — và `enterFocus` ghi *"chưa khai điểm vào"* cho mỗi
       * lượt dựng bố cục. Lượt đo đếm được **hàng chục** dòng như vậy trong console: một
       * chốt tự kêu bị kêu oan là một chốt sắp bị người sau tắt.
       *
       * Nửa thứ hai nặng hơn: `restore()` chạy `fromJSON`, thứ đặt lại `activePanel` — và
       * không có phép kiểm này thì mỗi lượt quay lại Workspace là một lần **cướp focus
       * DOM** khỏi chỗ người dùng đang đứng. Chính là điều mà `onActivated` của
       * `WorkspaceMode` cố ý không làm.
       *
       * ⇒ AD-34 §2 nói *"CHUYỂN panel phải dời focus DOM tường minh"*. "Chuyển" là một
       * thao tác của NGƯỜI, và `DockviewOrigin` là chỗ dockview phân biệt đúng điều đó.
       */
      if (e.origin !== 'user') return
      const id = e.panel?.id
      if (id !== undefined && isPanelId(id)) void enterFocus(id)
    }),
  )
  disposables.push(api.onDidLayoutChange(onLayoutChange))

  setDockController({ applyPreset, togglePanel, visiblePanelsInLayoutOrder })

  // Story 4.12, Task 3 — tầng khởi động: `restore()` ở trên có thể đã đi qua nhánh
  // `fromJSON` (bố cục tuỳ ý, không qua `applyPreset`), nhánh đó KHÔNG tự đo tầng. Gọi ở đây
  // phủ cả hai nhánh — lượt gọi thứ hai (khi `restore()` đã đi qua `applyPreset` và tự đo
  // rồi) là một no-op rẻ, vì `measureAndApplyTier` tự so `tier === currentTier.value`.
  measureAndApplyTier()
}

/**
 * ⚠️ `beforeunload` là lượt ghi CUỐI — đóng cửa sổ trong khoảng idle 500 ms sau một cú kéo
 * sash là ca thường gặp nhất, không phải một ca hiếm.
 *
 * `flush()` gọi `emit('persist')` đồng bộ, nhưng lượt `putConfig` phía sau là **async**
 * và không có gì bảo đảm nó kịp vượt IPC trước khi tiến trình chết. Đó là một giới hạn
 * THẬT và nó được ghi ra thay vì được giả vờ đã đóng: mất một lượt kéo sash không phải
 * mất công việc (xem `writeSchedule.ts`), nên cái giá chấp nhận được. Trần cứng 5 s là thứ
 * giữ cho khoảng mất tối đa có chặn trên.
 */
const onBeforeUnload = (): void => flush()

onMounted(() => {
  window.addEventListener('beforeunload', onBeforeUnload)
  // Story 4.12, Task 3 — nguồn duy nhất đo lại tầng sau lúc khởi động. Không debounce
  // riêng: `measureAndApplyTier` tự no-op khi tầng chưa đổi, nên một cơn `resize` dày sự
  // kiện chỉ mua thêm vài phép so sánh chuỗi rẻ, không thêm lượt biến đổi dock nào.
  window.addEventListener('resize', onWindowResize)
})

/**
 * Rời Workspace ⇒ ghi ngay, không đợi hết idle. `<KeepAlive>` giữ component sống nên
 * `onBeforeUnmount` KHÔNG chạy ở lượt đổi chế độ — đây là hook duy nhất bắt được nó.
 *
 * 🔴 GỠ CON TRỎ DOCK — bắt được ở lượt code review, không phải lúc dựng.
 *
 * `layout.preset_*`/`layout.toggle_*` là hợp âm TOÀN CỤC (cùng họ với `mode.*`), dispatch
 * bất kể chế độ nào đang hiện — `keys.ts::handle` không lọc theo mode. Trước bản sửa này,
 * `setDockController` chỉ bị gỡ ở `onBeforeUnmount`, mà `<KeepAlive>` không gọi hook đó khi
 * đổi chế độ. Hệ quả: bấm `Mod+Alt+1`/`Mod+Alt+2` lúc đang ở Library/Reading vẫn chạy
 * `applyPreset()`/`togglePanel()` thật lên cái dock đã `<KeepAlive>` đỗ — `api.clear()` +
 * dựng lại ba panel, rồi TỰ GHI xuống đĩa qua `onDidLayoutChange` → `flush()` — đè mất bố
 * cục người dùng vừa sắp mà không có dấu hiệu gì trên màn hình, vì Workspace không hiện.
 *
 * ⇒ Gỡ con trỏ ở đây, y hệt `onBeforeUnmount`. `onActivated` bên dưới đăng ký lại lúc quay
 * về, nên hành vi trong Workspace không đổi — chỉ đóng cửa số khi Workspace KHÔNG hiện.
 */
onDeactivated(() => {
  flush()
  setDockController(null)
})

/**
 * ⚠️ Đăng ký lại con trỏ dock lúc quay lại: `onReady` chỉ chạy MỘT lần, còn
 * [`setDockController`] vừa bị chính `onDeactivated` ở trên gỡ mỗi lượt rời Workspace, và
 * cũng có thể đã bị một lượt `onDeactivated` của một `WorkspaceDock` khác ghi đè. Hôm nay
 * chỉ có một, nhưng Review Mode của Story 8.11 dựng cái thứ hai.
 */
onActivated(() => {
  if (dock.value !== null) {
    setDockController({ applyPreset, togglePanel, visiblePanelsInLayoutOrder })
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('beforeunload', onBeforeUnload)
  window.removeEventListener('resize', onWindowResize)
  flush()
  for (const d of disposables) d.dispose()
  disposables.length = 0
  // BẮT BUỘC gỡ con trỏ: để nó trỏ vào một `DockviewApi` đã tháo thì mọi lời gọi sau đó
  // trượt ở một chỗ sâu trong thư viện thay vì ở dòng `if (live === null)`.
  setDockController(null)
  dock.value = null
  // Một lượt hẹn `restoreFocusIfLost` đang chờ ⇒ huỷ, không để nó chạy sau khi tháo.
  if (pendingFocusRaf !== null) {
    cancelAnimationFrame(pendingFocusRaf)
    pendingFocusRaf = null
  }
})
</script>

<template>
  <!--
    ⚠️ `.dockview-theme-aura` là lớp theme CỦA DỰ ÁN (`src/layout/dockview-theme.css`).
    KHÔNG dùng `dockview-theme-light` / `-dark` / một trong mười hai theme dựng sẵn:
    cả mười hai viết màu thẳng, và `check:tokens` không thấy chúng vì nó chỉ quét
    `src/**`. Đó là §Bẫy 1 của story và là đúng thứ AD-34 §3 tồn tại để chặn.

    ⚠️ `single-tab-mode="fullwidth"`: một panel một mình thì tab của nó trải hết chiều
    rộng — tức thanh tiêu đề panel 34px của UX-DR17, thay vì một cái tab con con nằm nép
    bên trái một dải trống.
  -->
  <!--
    Story 4.12, Task 3 — móc `data-*` cho Phase 4: tầng đang áp, preset tầng đang đọc, và có
    đang gộp tab (tầng `short`) không. Không một phép nào ở đây đọc kích thước PIXEL — đó là
    việc của Phase 5/e2e (`happy-dom` không tính layout); ba thuộc tính này chỉ phơi ra TRẠNG
    THÁI đã tính, để một test mô phỏng `resize` (đặt `window.innerWidth`/`innerHeight` rồi
    phát sự kiện) đọc lại được kết quả mà không cần đọc hình học thật.
  -->
  <div
    class="dock-host"
    :data-layout-tier="currentTier ?? undefined"
    :data-layout-preset="currentPresetId"
    :data-layout-merged="tierMerged ? 'true' : undefined"
  >
    <DockviewVue
      class="dock dockview-theme-aura"
      :theme="auraTheme"
      :components="components"
      :tab-components="tabComponents"
      single-tab-mode="fullwidth"
      @ready="onReady"
    />
  </div>
</template>

<style scoped>
/*
 * 🔴 MỘT `<div>` BỌC NGOÀI, KHÔNG phải style thẳng lên `<DockviewVue>` — bắt được lúc đo.
 *
 * Lượt nghiệm thu thị giác (Task 11) đọc DOM thật: phần tử gốc của `<DockviewVue>` nhận
 * `class="dock dockview-theme-aura"` nhưng **KHÔNG nhận thuộc tính scope `data-v-*`**
 * của tệp này. `<style scoped>` biên dịch `.dock` thành `.dock[data-v-xxx]`, nên luật
 * `height: 100%` **không bao giờ khớp**: dock cao **0px**, dockview đo container rỗng
 * rồi tự chọn 100px, và ba panel hiện ra cao 100px trong một cửa sổ 900px.
 *
 * ⚠️ Và không cổng nào bắt được: `check:tokens` đọc khai báo CSS chứ không đọc chiều
 * cao đã tính; mọi cổng đều xanh với một Workspace cao 100px.
 *
 * ⇒ Chiều cao thuộc về một phần tử **của tệp này** (`.dock-host`, có scope), và phần tử
 * con nhận nó qua `:deep()` — thứ cố ý không đòi scope ở phía sau.
 */
.dock-host {
  display: flex;
  flex: 1;
  min-height: 0;
  min-width: 0;
}

.dock-host > :deep(.dock) {
  flex: 1;
  min-width: 0;
  min-height: 0;
}
</style>
