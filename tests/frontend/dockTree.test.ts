/**
 * `dockTree.ts` thuần — Story 4.12, Phase 4d. Không mount, không `happy-dom`.
 *
 * Ba cây đầu (`TREE_B2`, `TREE_B2_AI_HIDDEN`, `TREE_B1`) chép ĐÚNG hình dạng
 * `api.toJSON().grid` in ra thật từ một dock đã mount (`dockview-core@7.0.4`), qua kịch bản
 * chuyển preset thật (`layout.preset_grid` mặc định → `layout.preset_columns` → về lại
 * `layout.preset_grid`) rồi `beforeunload` để bắt payload `persist` — không đoán hình dạng.
 * Xem `4-12-phases-2026-09-22.md` §Phase 4d cho JSON gốc và cách bắt nó.
 *
 * `TREE_B2_AI_HIDDEN` xác nhận thêm một sự thật đo được ngoài lề: `api.removePanel` trên
 * panel cuối của một nhánh hai-con làm dockview GỘP nhánh một-con còn lại thẳng vào nhánh
 * cha (nhánh `[lookup, ai]` mất `ai` thì `lookup` thành LÁ TRỰC TIẾP của gốc, không còn một
 * nhánh một-con bọc ngoài) — cùng hình dạng cây đã thấy khi đối chứng lỗi cũ (bản ghi ở
 * `WorkspaceDock.vue::rememberSpot` cũ luôn trả `'right'`).
 */
import { describe, expect, it } from 'vitest'
import { findTreeSpot, unmergeForPersist, viewsIn } from '../../src/layout/dockTree'
import type { GridNode, SerializedDockJSON, SerializedGrid } from '../../src/layout/dockTree'

function leaf(id: string): GridNode {
  return { type: 'leaf', data: { views: [id], activeView: id, id: `g-${id}` } }
}

function branch(children: GridNode[]): GridNode {
  return { type: 'branch', data: children, size: 100 } as unknown as GridNode
}

// Ⓑ-2 (mặc định): lưới trái, [Tra cứu / Đề xuất AI] cột phải — gốc NGANG, nhánh lồng DỌC.
const TREE_B2: SerializedGrid = {
  orientation: 'HORIZONTAL',
  root: branch([leaf('panel.grid'), branch([leaf('panel.lookup'), leaf('panel.ai_translation')])]),
}

// Cùng Ⓑ-2, sau khi `panel.ai_translation` đã bị ẩn (gỡ khỏi cây) — nhánh `[lookup, ai]`
// gộp lại, `lookup` thành lá trực tiếp của gốc.
const TREE_B2_AI_HIDDEN: SerializedGrid = {
  orientation: 'HORIZONTAL',
  root: branch([leaf('panel.grid'), leaf('panel.lookup')]),
}

// Ⓑ-1: lưới trên full-width, [Tra cứu | Đề xuất AI] hàng dưới — gốc DỌC, nhánh lồng NGANG.
const TREE_B1: SerializedGrid = {
  orientation: 'VERTICAL',
  root: branch([leaf('panel.grid'), branch([leaf('panel.lookup'), leaf('panel.ai_translation')])]),
}

// Cùng Ⓑ-1, sau khi `panel.ai_translation` đã bị ẩn.
const TREE_B1_AI_HIDDEN: SerializedGrid = {
  orientation: 'VERTICAL',
  root: branch([leaf('panel.grid'), leaf('panel.lookup')]),
}

// Cây lồng SÂU 3 tầng, dựng thủ công theo ĐÚNG khuôn đã đo (branch/leaf, không orientation
// riêng từng nhánh) để canh luật xen kẽ ở độ sâu ≥ 3 — kho thật của Story 4.12 chỉ có 3
// panel nên không tự sinh được độ sâu này.
//   tầng 0 (gốc, NGANG khai báo)  : [A, tầng1]
//   tầng 1 (đảo ⇒ DỌC)            : [B, tầng2]
//   tầng 2 (đảo ⇒ NGANG)          : [C, D]
const TREE_DEEP: SerializedGrid = {
  orientation: 'HORIZONTAL',
  root: branch([leaf('A'), branch([leaf('B'), branch([leaf('C'), leaf('D')])])]),
}

// Nhánh MỘT-con lồng giữa: `panel.x` là con DUY NHẤT của nhánh tầng 1, nên bước leo đầu
// tiên (đúng nhánh chứa nó) không có anh em nào — phải leo tiếp lên gốc mới gặp `panel.a`.
// Đây là ca doc-comment gốc của `siblingInTree` nêu tên: "một nhánh có thể chỉ có đúng một
// con sau vài lượt kéo–thả".
const TREE_SINGLE_CHILD_BRANCH: SerializedGrid = {
  orientation: 'HORIZONTAL',
  root: branch([leaf('panel.a'), branch([leaf('panel.x')])]),
}

// Panel duy nhất trong lưới — gốc là một LÁ, không một nhánh nào.
const TREE_SOLO: SerializedGrid = {
  orientation: 'HORIZONTAL',
  root: leaf('panel.grid'),
}

describe('findTreeSpot — Ⓑ-2 (gốc NGANG, nhánh lồng DỌC)', () => {
  it('ai_translation so với lookup: cùng nhánh DỌC, đứng SAU ⇒ below', () => {
    expect(findTreeSpot(TREE_B2, 'panel.ai_translation')).toEqual({
      reference: 'panel.lookup',
      direction: 'below',
    })
  })

  it('lookup so với grid (sau khi ai_translation đã ẩn): cùng nhánh NGANG, đứng SAU ⇒ right', () => {
    expect(findTreeSpot(TREE_B2_AI_HIDDEN, 'panel.lookup')).toEqual({
      reference: 'panel.grid',
      direction: 'right',
    })
  })

  it('lookup so với ai_translation (cây đủ 3 panel): cùng nhánh DỌC, đứng TRƯỚC ⇒ above', () => {
    // Đi cây dừng ở nhánh GẦN NHẤT còn anh em — với `lookup` đó là nhánh `[lookup, ai]`,
    // không leo tiếp lên gốc để lấy `grid`. Cùng luật `siblingInTree` cũ (Task 11).
    expect(findTreeSpot(TREE_B2, 'panel.lookup')).toEqual({
      reference: 'panel.ai_translation',
      direction: 'above',
    })
  })
})

describe('findTreeSpot — Ⓑ-1 (gốc DỌC, nhánh lồng NGANG)', () => {
  it('ai_translation so với lookup: cùng nhánh NGANG, đứng SAU ⇒ right', () => {
    expect(findTreeSpot(TREE_B1, 'panel.ai_translation')).toEqual({
      reference: 'panel.lookup',
      direction: 'right',
    })
  })

  it('lookup so với grid (sau khi ai_translation đã ẩn): cùng nhánh DỌC, đứng SAU ⇒ below', () => {
    expect(findTreeSpot(TREE_B1_AI_HIDDEN, 'panel.lookup')).toEqual({
      reference: 'panel.grid',
      direction: 'below',
    })
  })
})

describe('findTreeSpot — lồng sâu ≥ 3 tầng', () => {
  it('D so với C ở tầng 2 (đảo hai lần từ gốc NGANG ⇒ NGANG): đứng SAU ⇒ right', () => {
    expect(findTreeSpot(TREE_DEEP, 'D')).toEqual({ reference: 'C', direction: 'right' })
  })

  it('nhánh một-con: leo QUA nhánh không anh em, dừng ở gốc', () => {
    expect(findTreeSpot(TREE_SINGLE_CHILD_BRANCH, 'panel.x')).toEqual({
      reference: 'panel.a',
      direction: 'right',
    })
  })
})

describe('findTreeSpot — panel duy nhất trong lưới', () => {
  it('gốc là một LÁ ⇒ null (không anh em nào để neo vào)', () => {
    expect(findTreeSpot(TREE_SOLO, 'panel.grid')).toBeNull()
  })
})

describe('viewsIn — mọi id panel trong một cây con, theo thứ tự cây', () => {
  it('gom cả lá lẫn nhánh lồng', () => {
    expect(viewsIn(TREE_B2.root)).toEqual(['panel.grid', 'panel.lookup', 'panel.ai_translation'])
  })
})

// ═══════════════════════════════════════════════════════════════════════════════════
// unmergeForPersist — Story 4.12, Phase 4e (phán quyết Ice 2026-09-23). Fixtures chép ĐÚNG
// hình dạng thật (kể cả `size`/`id`) in ra từ một dock mount thật — xem
// `4-12-phases-2026-09-22.md` §Phase 4e cho JSON gốc: mỗi nút mang `size`, mỗi LÁ mang thêm
// `id` số tăng dần (`"1"`,`"2"`,`"4"` — `"3"` đã bị một nhóm khác chiếm rồi gỡ).
// ═══════════════════════════════════════════════════════════════════════════════════

function leafG(views: string[], opts: { id: string; activeView?: string; size?: number }): GridNode {
  return {
    type: 'leaf',
    data: { views, activeView: opts.activeView ?? views[0], id: opts.id },
    size: opts.size ?? 100,
  } as unknown as GridNode
}

function branchG(children: GridNode[], size = 100): GridNode {
  return { type: 'branch', data: children, size } as unknown as GridNode
}

function dockJSON(root: GridNode, orientation: SerializedGrid['orientation'], activeGroup: string): SerializedDockJSON {
  return {
    grid: { root, orientation, width: 100, height: 100 },
    panels: {
      'panel.grid': { id: 'panel.grid', contentComponent: 'grid', tabComponent: 'aura', params: {}, title: 'panel.grid' },
      'panel.lookup': { id: 'panel.lookup', contentComponent: 'lookup', tabComponent: 'aura', params: {}, title: 'panel.lookup' },
      'panel.ai_translation': {
        id: 'panel.ai_translation',
        contentComponent: 'aiTranslation',
        tabComponent: 'aura',
        params: {},
        title: 'panel.ai_translation',
      },
    },
    activeGroup,
  }
}

/** Hình dạng cây, cùng khuôn ASCII `|`/`/` với `shapeOf` của `workspaceDockTier.test.ts`. */
function shape(node: GridNode, orientation: SerializedGrid['orientation']): string {
  if (node.type === 'leaf') return viewsIn(node).join(',')
  const sep = orientation === 'HORIZONTAL' ? '|' : '/'
  const childOrientation: SerializedGrid['orientation'] = orientation === 'HORIZONTAL' ? 'VERTICAL' : 'HORIZONTAL'
  return `(${(node.data as GridNode[]).map((child) => shape(child, childOrientation)).join(sep)})`
}

// Ⓑ-2 gộp: sau `applyMerge`, nhánh `[lookup, ai]` đã gộp một-con vào gốc (xem
// `TREE_B2_AI_HIDDEN` ở trên), rồi `ai` được tab lại VÀO lá `lookup` đó — lá gộp là con
// TRỰC TIẾP của gốc NGANG.
const B2_MERGED: SerializedDockJSON = dockJSON(
  branchG([leafG(['panel.grid'], { id: '1' }), leafG(['panel.lookup', 'panel.ai_translation'], { id: '2', activeView: 'panel.ai_translation' })]),
  'HORIZONTAL',
  '2',
)

// Ⓑ-1 gộp: cùng cơ chế, gốc DỌC.
const B1_MERGED: SerializedDockJSON = dockJSON(
  branchG([leafG(['panel.grid'], { id: '1' }), leafG(['panel.lookup', 'panel.ai_translation'], { id: '2', activeView: 'panel.ai_translation' })]),
  'VERTICAL',
  '2',
)

// CHƯA gộp — `panel.ai_translation` đã có lá riêng, không chia sẻ lá với `panel.lookup`. Một
// `mergedSpot` cũ (từ lượt gộp trước) đem áp vào đây phải KHÔNG LÀM GÌ.
const B2_UNMERGED: SerializedDockJSON = dockJSON(
  branchG([
    leafG(['panel.grid'], { id: '1' }),
    branchG([leafG(['panel.lookup'], { id: '2' }), leafG(['panel.ai_translation'], { id: '4' })]),
  ]),
  'HORIZONTAL',
  '4',
)

describe('unmergeForPersist — Ⓑ-2 gộp + {lookup, below} ⇒ bọc (sai trục, lá gộp là con trực tiếp của gốc)', () => {
  const result = unmergeForPersist(B2_MERGED, 'panel.ai_translation', { reference: 'panel.lookup', direction: 'below' })

  it('hình dạng trả về đúng preset gốc CHƯA gộp', () => {
    expect(shape(result.grid.root, result.grid.orientation)).toBe('(panel.grid|(panel.lookup/panel.ai_translation))')
  })

  it('không còn lá nào chở cả hai panel', () => {
    const root = result.grid.root as GridNode
    const branchNode = (root.data as GridNode[])[1] as GridNode
    const kids = branchNode.data as GridNode[]
    for (const kid of kids) expect(viewsIn(kid).length).toBe(1)
  })

  it('size chia đôi cho lá cũ và lá mới', () => {
    const root = result.grid.root as GridNode
    const branchNode = (root.data as GridNode[])[1] as GridNode
    const kids = branchNode.data as (GridNode & { size: number })[]
    expect(kids[0]?.size).toBe(50)
    expect(kids[1]?.size).toBe(50)
  })
})

describe('unmergeForPersist — Ⓑ-1 gộp + {lookup, right} ⇒ bọc, gốc DỌC', () => {
  it('hình dạng trả về đúng preset gốc CHƯA gộp', () => {
    const result = unmergeForPersist(B1_MERGED, 'panel.ai_translation', { reference: 'panel.lookup', direction: 'right' })
    expect(shape(result.grid.root, result.grid.orientation)).toBe('(panel.grid/(panel.lookup|panel.ai_translation))')
  })
})

describe('unmergeForPersist — trục KHỚP sẵn ⇒ chèn anh em, không bọc nhánh mới', () => {
  it('lá gộp đã là con trực tiếp của một nhánh ĐÚNG trục ⇒ chèn lá mới ngay cạnh, không thêm tầng lồng', () => {
    // Dựng thủ công (không phải một ca gộp THẬT của Story 4.12 — merge luôn thu lá gộp về
    // làm con trực tiếp của gốc, và hướng nhớ trước gộp luôn ở nhánh lồng SÂU HƠN gốc một
    // bậc, nên trục luôn LỆCH — xem doc-comment `unmergeForPersist`). Ca này canh nhánh CÒN
    // LẠI của thuật toán: gốc NGANG, lá gộp SẴN là con trực tiếp của gốc, hướng cần `right`
    // (trục NGANG) khớp thẳng trục gốc ⇒ không cần bọc, chỉ chèn.
    const dock = dockJSON(
      branchG([
        leafG(['panel.grid'], { id: '1' }),
        leafG(['panel.lookup', 'panel.ai_translation'], { id: '2', activeView: 'panel.ai_translation' }),
        leafG(['panel.other'], { id: '5' }),
      ]),
      'HORIZONTAL',
      '2',
    )
    const result = unmergeForPersist(dock, 'panel.ai_translation', { reference: 'panel.lookup', direction: 'right' })
    expect(shape(result.grid.root, result.grid.orientation)).toBe('(panel.grid|panel.lookup|panel.ai_translation|panel.other)')
    const root = result.grid.root as GridNode
    expect((root.data as GridNode[]).length).toBe(4)
  })
})

describe('unmergeForPersist — không làm gì khi panel không còn chia sẻ lá với neo', () => {
  it('panel đã có lá riêng (đã bị kéo tay ra) ⇒ trả nguyên tham chiếu cũ', () => {
    const result = unmergeForPersist(B2_UNMERGED, 'panel.ai_translation', { reference: 'panel.lookup', direction: 'below' })
    expect(result).toBe(B2_UNMERGED)
  })

  it('panel không tồn tại trong cây ⇒ trả nguyên tham chiếu cũ', () => {
    const result = unmergeForPersist(B2_UNMERGED, 'panel.khong_ton_tai', { reference: 'panel.lookup', direction: 'below' })
    expect(result).toBe(B2_UNMERGED)
  })
})

// Bố cục trên đĩa của Ice (`grid | ai | lookup`) sau khi gộp: `ai` rời lá riêng vào lá
// `lookup`. Anh em trước gộp của `ai` là `grid`, không phải `lookup`.
const ICE_MERGED: SerializedDockJSON = dockJSON(
  branchG([leafG(['panel.grid'], { id: '1' }), leafG(['panel.lookup', 'panel.ai_translation'], { id: '2', activeView: 'panel.ai_translation' })]),
  'HORIZONTAL',
  '2',
)

describe('unmergeForPersist — điểm neo KHÔNG phải panel bị gộp vào (🔵 2026-09-23)', () => {
  it('{grid, right} ⇒ ai quay về giữa grid và lookup, không lá nào chở hai panel', () => {
    const result = unmergeForPersist(ICE_MERGED, 'panel.ai_translation', { reference: 'panel.grid', direction: 'right' })
    expect(shape(result.grid.root, result.grid.orientation)).toBe('(panel.grid|panel.ai_translation|panel.lookup)')
  })

  it('{grid, within} ⇒ ai tab lại cùng grid, rời lá lookup', () => {
    const result = unmergeForPersist(ICE_MERGED, 'panel.ai_translation', { reference: 'panel.grid', direction: 'within' })
    expect(shape(result.grid.root, result.grid.orientation)).toBe('(panel.grid,panel.ai_translation|panel.lookup)')
  })

  it('{lookup, within} khi ai vốn đã tab cùng lookup ⇒ không có gì để gỡ, trả nguyên tham chiếu', () => {
    expect(unmergeForPersist(ICE_MERGED, 'panel.ai_translation', { reference: 'panel.lookup', direction: 'within' })).toBe(ICE_MERGED)
  })
})

describe('unmergeForPersist — không mutate tham số đầu vào', () => {
  it('B2_MERGED giữ nguyên hệt sau lượt gọi có tách thật', () => {
    const before = JSON.stringify(B2_MERGED)
    unmergeForPersist(B2_MERGED, 'panel.ai_translation', { reference: 'panel.lookup', direction: 'below' })
    expect(JSON.stringify(B2_MERGED)).toBe(before)
  })
})

describe('unmergeForPersist — activeView được sửa khi nó từng trỏ vào panel bị tách', () => {
  it('lá còn lại (lookup) có activeView = lookup, không còn trỏ ai_translation đã tách', () => {
    const result = unmergeForPersist(B2_MERGED, 'panel.ai_translation', { reference: 'panel.lookup', direction: 'below' })
    const root = result.grid.root as GridNode
    const branchNode = (root.data as GridNode[])[1] as GridNode
    const oldLeaf = (branchNode.data as GridNode[])[0] as GridNode
    expect((oldLeaf.data as { activeView: string }).activeView).toBe('panel.lookup')
    expect(viewsIn(oldLeaf)).toEqual(['panel.lookup'])
  })
})

describe('unmergeForPersist — id lá mới không trùng id lá đang có', () => {
  it('B2_MERGED có id "1"/"2" đang dùng ⇒ lá mới nhận "3"', () => {
    const result = unmergeForPersist(B2_MERGED, 'panel.ai_translation', { reference: 'panel.lookup', direction: 'below' })
    const root = result.grid.root as GridNode
    const branchNode = (root.data as GridNode[])[1] as GridNode
    const newLeaf = (branchNode.data as GridNode[])[1] as GridNode
    expect((newLeaf.data as { id: string }).id).toBe('3')
  })
})
