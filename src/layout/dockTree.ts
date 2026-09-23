/**
 * Đọc DÂY NỐI của cây lưới dockview (`api.toJSON().grid`) để tìm neo và hướng của một
 * panel — Story 4.12, Phase 4d. Thuần, không import, không đụng DOM: `WorkspaceDock.vue`
 * là nơi duy nhất gọi vào đây với dữ liệu thật.
 *
 * ⇒ Vì sao tệp này tồn tại: `rememberSpot` (`WorkspaceDock.vue`) từng đọc hướng từ
 * `group.api.boundingBox` — toạ độ DOM thật, `0` trước khi dock được đo lần đầu và trong
 * lúc Workspace nằm ẩn dưới `<KeepAlive>`. `dx = dy = 0` luôn đọc thành `'right'`. Cây lưới
 * đã tuần tự hoá (`api.toJSON()`) không có vấn đề đó — nó là dữ liệu công khai, không phụ
 * thuộc kích thước container.
 *
 * 🔴 LUẬT ĐÃ ĐO, KHÔNG ĐOÁN (dockview-core@7.0.4, xem `4-12-phases-2026-09-22.md`
 * §Phase 4d cho JSON in ra thật): `grid.orientation` là hướng của nhánh GỐC (tầng 0).
 * `GridNode` không chở orientation riêng cho từng nhánh — chỉ có `type`/`data` — nên hướng
 * của một nhánh lồng sâu hơn phải TÍNH bằng độ sâu: mỗi tầng lồng thêm một bậc thì đảo
 * hướng so với tầng cha (tầng 1 ngược tầng 0, tầng 2 lại giống tầng 0, …).
 */

export type GridOrientation = 'HORIZONTAL' | 'VERTICAL'

/** Một nút của cây lưới trong `api.toJSON().grid.root`. */
export type GridNode = { type: 'leaf' | 'branch'; data: unknown }

export type SerializedGrid = { root: GridNode; orientation: GridOrientation }

export type PlacementDirection = 'right' | 'below' | 'left' | 'above'

export type RememberedTreeSpot = { reference: string; direction: PlacementDirection }

function flip(orientation: GridOrientation): GridOrientation {
  return orientation === 'HORIZONTAL' ? 'VERTICAL' : 'HORIZONTAL'
}

/**
 * `panelIsAfterAnchor` là vị trí của panel ĐANG TÌM so với neo trong mảng con của nhánh:
 * đứng sau neo ⇒ `true`. Nhánh NGANG: sau ⇒ `'right'`, trước ⇒ `'left'`. Nhánh DỌC: sau ⇒
 * `'below'`, trước ⇒ `'above'`.
 */
function directionFor(orientation: GridOrientation, panelIsAfterAnchor: boolean): PlacementDirection {
  if (orientation === 'HORIZONTAL') return panelIsAfterAnchor ? 'right' : 'left'
  return panelIsAfterAnchor ? 'below' : 'above'
}

/** Mọi id panel nằm trong một cây con, theo thứ tự cây. */
export function viewsIn(node: GridNode): string[] {
  if (node.type === 'leaf') return [...((node.data as { views?: string[] }).views ?? [])]
  return (node.data as GridNode[]).flatMap(viewsIn)
}

type PathStep = { branch: GridNode; index: number; orientation: GridOrientation }

/**
 * Anh em THẬT của `id` trong cây lưới — nút cạnh nó trong CÙNG một nhánh — cộng hướng của
 * `id` so với anh em đó.
 *
 * Đi cây một lần, giống hệt `siblingInTree` cũ đã đo đúng (Story 4.12, Task 11, xem
 * `WorkspaceDock.vue` bản trước Phase 4d): leo NGƯỢC từ lá lên gốc, dừng ở nhánh gần nhất
 * còn một nút khác — cần leo vì một nhánh có thể chỉ còn đúng một con sau vài lượt kéo–thả
 * (dockview tự gộp nhánh một-con vào nhánh cha). `id` **duy nhất** trong lưới ⇒ không nhánh
 * nào có anh em ⇒ `null`.
 */
export function findTreeSpot(grid: SerializedGrid, id: string): RememberedTreeSpot | null {
  const path: PathStep[] = []
  const find = (node: GridNode, orientation: GridOrientation): boolean => {
    if (node.type === 'leaf') return viewsIn(node).includes(id)
    const kids = node.data as GridNode[]
    const childOrientation = flip(orientation)
    for (let i = 0; i < kids.length; i += 1) {
      path.push({ branch: node, index: i, orientation })
      if (find(kids[i] as GridNode, childOrientation)) return true
      path.pop()
    }
    return false
  }
  if (!find(grid.root, grid.orientation)) return null
  for (let i = path.length - 1; i >= 0; i -= 1) {
    const step = path[i] as PathStep
    // ⚠️ `(GridNode | undefined)[]` chứ không `GridNode[]`: hai chỉ số đọc ngay dưới là
    // `index - 1` và `index + 1`, tức ở hai đầu danh sách luôn có một cái ngoài biên.
    const kids = step.branch.data as readonly (GridNode | undefined)[]
    const before = kids[step.index - 1]
    if (before !== undefined) {
      const reference = viewsIn(before).find((v) => v !== id)
      if (reference !== undefined) return { reference, direction: directionFor(step.orientation, true) }
    }
    const after = kids[step.index + 1]
    if (after !== undefined) {
      const reference = viewsIn(after).find((v) => v !== id)
      if (reference !== undefined) return { reference, direction: directionFor(step.orientation, false) }
    }
  }
  return null
}

// ═══════════════════════════════════════════════════════════════════════════════════
// Dựng cây khi gỡ gộp CHỈ ĐỂ LƯU — Story 4.12, Phase 4e (phán quyết Ice 2026-09-23
// "giữ gộp, lưu dạng chưa gộp"). Ngược chiều [`findTreeSpot`] ở trên: hàm đó ĐỌC cây có sẵn,
// [`unmergeForPersist`] DỰNG một cây MỚI.
// ═══════════════════════════════════════════════════════════════════════════════════

/**
 * Cây tuần tự hoá ĐẦY ĐỦ (`api.toJSON()`) — chở thêm `panels`/`activeGroup` mà `SerializedGrid`
 * không có, cộng `width`/`height` trên chính `grid`. Hình dạng đo được thật (in ra từ một dock
 * mount thật, xem `4-12-phases-2026-09-22.md` §Phase 4e): mỗi nút LÁ lẫn NHÁNH đều mang
 * `size` — số dockview dùng để chia không gian giữa các anh em, `0` khi chưa có kích thước cố
 * định (tự chia đều lúc dựng lại) — và mỗi nút LÁ mang thêm `id` (chuỗi số tăng dần dockview
 * tự phát, KHÔNG tái dùng sau khi một nhóm bị xoá — `"1"`,`"2"`,`"4"` là một bộ đo thật,
 * `"3"` đã mất vì một nhóm khác từng chiếm nó rồi bị gỡ).
 */
export type SerializedDockJSON = {
  grid: SerializedGrid & { width?: number; height?: number }
  panels: Record<string, unknown>
  activeGroup?: string
}

type SizedGridNode = GridNode & { size?: number }
type LeafData = { views: string[]; activeView: string; id: string }

function axisForDirection(direction: PlacementDirection): GridOrientation {
  return direction === 'left' || direction === 'right' ? 'HORIZONTAL' : 'VERTICAL'
}

function isAfterDirection(direction: PlacementDirection): boolean {
  return direction === 'right' || direction === 'below'
}

/** Mọi `id` nhóm (lá) đang có trong cây — để [`uniqueLeafId`] không đụng một cái đã dùng. */
function collectLeafIds(node: GridNode): string[] {
  if (node.type === 'leaf') {
    const id = (node.data as { id?: string }).id
    return id === undefined ? [] : [id]
  }
  return (node.data as GridNode[]).flatMap(collectLeafIds)
}

/** `id` mới, số, không trùng bất kỳ `id` lá nào đang có — lớn hơn `id` số lớn nhất một bậc. */
function uniqueLeafId(existing: string[]): string {
  let max = -1
  for (const id of existing) {
    const n = Number(id)
    if (Number.isInteger(n) && n > max) max = n
  }
  let candidate = String(max + 1)
  while (existing.includes(candidate)) {
    max += 1
    candidate = String(max + 1)
  }
  return candidate
}

type LeafLocation = { leaf: GridNode; parent: GridNode | null; index: number; axis: GridOrientation }

/**
 * Định vị nút LÁ chứa `id` — cùng lượt đi cây của [`findTreeSpot`] nhưng trả về chính NÚT (để
 * mutate) thay vì chỉ chuỗi anh em. `axis` trả về là hướng của NHÁNH CHA trực tiếp (mảng
 * `parent.data` chứa lá này được xếp theo trục nào) — `null` cho `parent` khi lá đó CHÍNH LÀ
 * gốc (`grid.root` là một lá, không nhánh nào bọc nó).
 *
 * 🔴 BẪY ĐÃ ĐO (Phase 4e, lượt kiểm đầu tiên bắt được): `orientation` truyền XUỐNG lượt gọi
 * đệ quy cho một nút là trục nút đó dùng để xếp CON CỦA CHÍNH NÓ (đã đảo so với cha) — trục
 * MẸ dùng để xếp CHÍNH NÚT ĐÓ giữa các anh em là giá trị TRƯỚC lượt đảo, tức tham số của lượt
 * gọi CHA, không phải tham số của lượt gọi hiện tại. Đọc thẳng `orientation` của lượt gọi
 * hiện tại khi gặp một LÁ (bản đầu đã làm vậy) cho ra trục ĐẢO một bậc — sai đúng bằng khớp
 * còn lại của mô hình. `findTreeSpot` không mắc bẫy này vì nó `path.push({..., orientation})`
 * TRƯỚC khi đảo và đệ quy — `parentAxis` dưới đây chép lại đúng thứ tự đó.
 */
function locateLeaf(root: GridNode, rootOrientation: GridOrientation, id: string): LeafLocation | null {
  const walk = (
    node: GridNode,
    ownAxis: GridOrientation,
    parent: GridNode | null,
    index: number,
    parentAxis: GridOrientation,
  ): LeafLocation | null => {
    if (node.type === 'leaf') {
      return viewsIn(node).includes(id) ? { leaf: node, parent, index, axis: parentAxis } : null
    }
    const kids = node.data as GridNode[]
    const childOrientation = flip(ownAxis)
    for (let i = 0; i < kids.length; i += 1) {
      const found = walk(kids[i] as GridNode, childOrientation, node, i, ownAxis)
      if (found !== null) return found
    }
    return null
  }
  return walk(root, rootOrientation, null, -1, rootOrientation)
}

/** Chỗ đã nhớ trước lượt gộp — như [`RememberedTreeSpot`], cộng `within` (đã tab cùng một panel). */
export type UnmergeSpot = { reference: string; direction: PlacementDirection | 'within' }

/**
 * Tách `panelId` khỏi nhóm nó đang GỘP TAB, rồi đặt nó về chỗ đã nhớ quanh LÁ CHỨA
 * `spot.reference`. Không đụng DOM, không `import`, không mutate tham số `dock`. Dùng khi
 * PERSIST một bố cục đang gộp (`WorkspaceDock.vue::flush`), để đĩa luôn chở hình dạng CHƯA GỘP
 * dù màn hình đang hiển thị đã gộp.
 *
 * 🔵 SỬA 2026-09-23 (orchestrator, sau Phase 4e) — bản đầu đặt lá mới cạnh NHÓM ĐANG GỘP và
 * không làm gì khi nhóm đó không chứa `spot.reference`. Điểm neo trước gộp là ANH EM TRONG CÂY
 * của panel, không nhất thiết là panel nó bị gộp vào. Với bố cục `grid | ai | lookup` (đúng bố
 * cục đang nằm trên đĩa của Ice), điểm neo là `grid`, nên bản đầu ghi nguyên nhóm đã gộp xuống
 * đĩa. `within` cũng từng bị loại, cùng hệ quả.
 *
 * Các nhánh:
 *   1. `panelId` không nằm chung lá với panel nào (đã bị kéo tay ra từ trước), hoặc không tìm
 *      được `spot.reference` ⇒ KHÔNG LÀM GÌ, trả nguyên `dock`.
 *   2. `within` ⇒ đưa `panelId` vào đúng lá của `spot.reference`.
 *   3. Nhánh CHA của lá tham chiếu đúng trục cần (`HORIZONTAL` cho `left`/`right`, `VERTICAL`
 *      cho `above`/`below`) ⇒ chèn lá mới làm ANH EM trước/sau lá tham chiếu.
 *   4. Sai trục, hoặc lá tham chiếu là gốc ⇒ bọc lá tham chiếu trong một nhánh MỚI đúng trục,
 *      chứa cả hai theo thứ tự hướng. Lá cũ là gốc thì nhánh mới thành `grid.root` và
 *      `grid.orientation` đổi theo.
 *
 * `size` của lá tham chiếu chia đôi cho nó và lá mới. Không có cách tính lại tỉ lệ THẬT (đó là
 * hình học DOM), nên chia đều là một xấp xỉ có chủ; dockview tự chia lại lúc dựng.
 */
export function unmergeForPersist(dock: SerializedDockJSON, panelId: string, spot: UnmergeSpot): SerializedDockJSON {
  if (spot.reference === panelId) return dock
  const cloned = JSON.parse(JSON.stringify(dock)) as SerializedDockJSON
  const shared = locateLeaf(cloned.grid.root, cloned.grid.orientation, panelId)
  if (shared === null || viewsIn(shared.leaf).length < 2) return dock
  const refBefore = locateLeaf(cloned.grid.root, cloned.grid.orientation, spot.reference)
  if (refBefore === null) return dock
  if (spot.direction === 'within' && refBefore.leaf === shared.leaf) return dock

  const sharedData = shared.leaf.data as LeafData
  sharedData.views = sharedData.views.filter((v) => v !== panelId)
  if (sharedData.activeView === panelId) sharedData.activeView = sharedData.views[0] as string

  const loc = locateLeaf(cloned.grid.root, cloned.grid.orientation, spot.reference) as LeafLocation
  if (spot.direction === 'within') {
    ;(loc.leaf.data as LeafData).views.push(panelId)
    return cloned
  }

  const sizedLeaf = loc.leaf as SizedGridNode
  const originalSize = sizedLeaf.size ?? 0
  const half = originalSize / 2
  sizedLeaf.size = half

  const newId = uniqueLeafId(collectLeafIds(cloned.grid.root))
  const newLeaf: SizedGridNode = { type: 'leaf', data: { views: [panelId], activeView: panelId, id: newId }, size: half }

  const axis = axisForDirection(spot.direction)
  const after = isAfterDirection(spot.direction)

  if (loc.parent !== null && loc.axis === axis) {
    const siblings = (loc.parent as { data: GridNode[] }).data
    siblings.splice(after ? loc.index + 1 : loc.index, 0, newLeaf)
    return cloned
  }

  const pair = after ? [loc.leaf, newLeaf] : [newLeaf, loc.leaf]
  const wrapper: SizedGridNode = { type: 'branch', data: pair, size: originalSize }
  if (loc.parent === null) {
    cloned.grid.root = wrapper
    cloned.grid.orientation = axis
  } else {
    ;(loc.parent as { data: GridNode[] }).data[loc.index] = wrapper
  }
  return cloned
}
