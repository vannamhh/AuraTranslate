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
// biệt được *"chuỗi hiển thị"* với *"chẩn đoán ra console"* — `deferred-work.md:36` đã ghi
// đúng giới hạn đó, và §Quyết định #6 của story này mở rộng cổng theo chiều KHÁC (đo text
// node của template), không nới chiều này.
//
// Đường thoát dễ là dời khối logic dưới đây sang một tệp `.ts` — Kiểm A không quét
// `.ts`. `deferred-work.md:35` gọi tên đúng đường đó và cấm nó bằng chữ: *"dời một chuỗi
// từ `.vue` sang `.ts` là cách hợp lệ về mặt cổng để cho xanh — đừng dùng."*
//
// ⇒ Dùng tiền lệ đã có: `src-tauri/src/commands/config.rs:36` cũng viết không dấu, cùng
// lý do. Người đọc những dòng này là người đang mở DevTools, không phải người dùng cuối.
// ⚠️ Comment tiếng Việt CÓ DẤU thì hợp lệ — Kiểm A che comment trước khi quét.
import { onActivated, onBeforeUnmount, onDeactivated, onMounted, shallowRef } from 'vue'
import { DockviewVue } from 'dockview-vue'
import type { DockviewApi, DockviewReadyEvent, IDockviewPanel, VueComponent } from 'dockview-vue'
import { enterFocus } from '../commands'
import SourcePanel from '../panels/SourcePanel.vue'
import LookupPanel from '../panels/LookupPanel.vue'
import AiTranslationPanel from '../panels/AiTranslationPanel.vue'
import EditorPanel from '../panels/EditorPanel.vue'
import PanelTab from '../panels/PanelTab.vue'
import { setDockController } from './dockController'
import { createWriteSchedule } from './writeSchedule'
import {
  DEFAULT_PRESET_ID,
  PANEL_COMPONENTS,
  PANEL_IDS,
  PANEL_TITLE_KEYS,
  presetById,
} from './workspaceLayout'
import type { PanelId, PlacementDirection } from './workspaceLayout'

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
}>()

/**
 * Bốn component nội dung, tra theo tên đã đăng ký.
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
 * bất cứ prop nào, còn bốn component này đòi `params`.
 *
 * Đường thay thế duy nhất là khai `params?:` (tuỳ chọn) ở CẢ NĂM component. Nó qua được
 * kiểu, và nó nói dối: dockview LUÔN truyền `params`, còn `PanelTab.vue` thì không chạy
 * được nếu thiếu — mọi lời gọi `api` ở đó sẽ phải mọc một `?.` cho một ca không tồn tại.
 * Ép kiểu một lần **ở đúng ranh giới thư viện** rẻ hơn bốn lời nói dối rải trong mã.
 */
const components = {
  source: SourcePanel,
  lookup: LookupPanel,
  aiTranslation: AiTranslationPanel,
  editor: EditorPanel,
} as unknown as Record<string, VueComponent>

/** ⚠️ MỘT tab component cho cả bốn panel — §Quyết định #4A. */
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
    return false
  }
  return true
}

// ═══════════════════════════════════════════════════════════════════════════════════
// Ẩn / hiện panel — AC3
// ═══════════════════════════════════════════════════════════════════════════════════

/** Một nút của cây lưới trong `api.toJSON().grid.root`. */
type GridNode = { type: 'leaf' | 'branch'; data: unknown }

/** Mọi id panel nằm trong một cây con, theo thứ tự cây. */
function viewsIn(node: GridNode): string[] {
  if (node.type === 'leaf') return [...((node.data as { views?: string[] }).views ?? [])]
  return (node.data as GridNode[]).flatMap(viewsIn)
}

/**
 * Anh em THẬT của panel trong cây lưới — tức nút cạnh nó trong **cùng một nhánh**.
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
function siblingInTree(api: DockviewApi, id: PanelId): string | null {
  const root = (api.toJSON() as unknown as { grid: { root: GridNode } }).grid.root
  // Đường từ gốc tới lá chứa `id`, cùng chỉ số của từng bước.
  const path: { branch: GridNode; index: number }[] = []
  const find = (node: GridNode): boolean => {
    if (node.type === 'leaf') return viewsIn(node).includes(id)
    const kids = node.data as GridNode[]
    for (let i = 0; i < kids.length; i += 1) {
      path.push({ branch: node, index: i })
      if (find(kids[i] as GridNode)) return true
      path.pop()
    }
    return false
  }
  if (!find(root)) return null
  // Leo NGƯỢC lên: nhánh gần nhất còn một nút khác là chỗ có anh em thật. Leo lên là cần
  // thiết vì một nhánh có thể chỉ có đúng một con sau vài lượt kéo–thả.
  for (let i = path.length - 1; i >= 0; i -= 1) {
    const step = path[i] as { branch: GridNode; index: number }
    const kids = step.branch.data as GridNode[]
    const neighbours = [kids[step.index - 1], kids[step.index + 1]].filter(
      (n): n is GridNode => n !== undefined,
    )
    for (const n of neighbours) {
      const candidate = viewsIn(n).find((v) => v !== id && isPanelId(v))
      if (candidate !== undefined) return candidate
    }
  }
  return null
}

/**
 * Ghi lại chỗ của một panel TRƯỚC khi gỡ nó.
 *
 * Hai ca, và ca thứ hai là ca dễ quên:
 *   1. panel **gộp tab** với panel khác trong cùng group ⇒ nhớ `within` + một bạn cùng
 *      group. Hiện lại phải quay về đúng group đó, không phải cắt một ô mới.
 *   2. panel một mình trong group ⇒ nhớ **anh em trong cây lưới** *(không phải hàng
 *      xóm hình học — xem [`siblingInTree`])* và hướng ngược lại.
 *
 * ⚠️ Hướng đọc từ **hình học thật** *(`boundingBox`)* chứ không suy ra từ `Orientation`
 * của nhánh: `Orientation` đảo ở mỗi tầng lồng nhau, và một lượt suy luận sai ở đó cho ra
 * một panel về đúng nhánh nhưng sai bên — thứ không cổng nào thấy được.
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
  const siblingId = siblingInTree(api, panel.id as PanelId)
  if (siblingId === null || !isPanelId(siblingId)) return null
  const mine = panel.api.group.api.boundingBox
  const theirs = api.getPanel(siblingId)?.api.group.api.boundingBox
  if (mine === undefined || theirs === undefined) return null
  // Trục nào lệch nhiều hơn thì đó là trục của lần cắt. `direction` là chỗ đặt panel MỚI
  // so với neo, nên nó ngược với chỗ neo đang đứng so với panel.
  const dx = mine.left - theirs.left
  const dy = mine.top - theirs.top
  const direction: PlacementDirection =
    Math.abs(dx) >= Math.abs(dy) ? (dx >= 0 ? 'right' : 'left') : dy >= 0 ? 'below' : 'above'
  return { reference: siblingId, direction }
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
  const fallback = visiblePanelsInLayoutOrder()[0]
  restoreFocusIfLost(showing ? panelId : ((fallback ?? panelId) as PanelId))
  return true
}

// ═══════════════════════════════════════════════════════════════════════════════════
// Lưu và khôi phục — AC4
// ═══════════════════════════════════════════════════════════════════════════════════

const schedule = createWriteSchedule()
let timer: ReturnType<typeof setTimeout> | null = null

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
    emit('persist', JSON.stringify(api.toJSON()))
  } catch (err) {
    // Không ném: một bố cục không serialize được không phải lý do để giết thao tác
    // mà người dùng vừa làm. Nó chỉ có nghĩa là phiên sau mở bằng preset mặc định.
    console.error(`[layout] khong serialize duoc bo cuc -- luot luu nay bo qua. ${String(err)}`)
  }
}

/**
 * Một lượt `onDidLayoutChange`. Xem `src/layout/writeSchedule.ts` cho lý lẽ đầy đủ —
 * tóm tắt: idle 500 ms **cộng** một trần cứng 5 s **không reset bởi sự kiện kế tiếp**.
 */
function onLayoutChange(): void {
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
   * ⚠️ MỘT chỗ nghe cho cả bốn panel, không phải một handler trên mỗi tab. `PanelTab.vue`
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
 * dựng lại bốn panel, rồi TỰ GHI xuống đĩa qua `onDidLayoutChange` → `flush()` — đè mất bố
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
  <div class="dock-host">
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
 * rồi tự chọn 100px, và bốn panel hiện ra cao 100px trong một cửa sổ 900px.
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
