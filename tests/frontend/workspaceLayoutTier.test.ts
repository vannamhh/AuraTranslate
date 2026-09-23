/**
 * `layoutTierFor` — Story 4.12, Phase 4a. Ma trận I/O của spec, đọc bằng số thuần: không
 * `window`, không DOM, không dockview. `happy-dom` không có hình học thật — mọi khẳng định ở
 * đây là về HÀM THUẦN, không phải về một cửa sổ giả.
 *
 * Hàng ma trận CÒN LẠI cho Phase 4b (cần `WorkspaceDock.vue` mount thật): "Lookup reached from
 * the status bar", "Window grows back", "User hides a panel by hand, then resizes", "Reaching
 * the unsupported tier" (Decision 1, `minWidth` hạ để kéo tay).
 *
 * `FocusRegistry.release(owner, expected)` — Story 4.12 Phase 3a, Task 6, Decision 2.
 */
import { describe, expect, it } from 'vitest'
import { LAYOUT_THRESHOLDS, layoutTierFor, type LayoutTier } from '../../src/layout/workspaceLayout'
import { createFocusRegistry } from '../../src/commands/focus'

const PRESET_IDS = ['layout.preset_grid', 'layout.preset_columns'] as const

describe('layoutTierFor — thang bậc thuần theo số (§I/O Matrix)', () => {
  for (const presetId of PRESET_IDS) {
    const t = LAYOUT_THRESHOLDS[presetId]

    describe(`preset ${presetId}`, () => {
      it('Full layout — đúng biên cả hai chiều ⇒ full (hàng "Full layout" + "Boundary exactly on a number")', () => {
        expect(layoutTierFor({ width: t.minFullWidth, height: t.minFullHeight }, presetId)).toBe('full')
      })

      it('Full layout — rộng hơn/cao hơn biên vẫn full', () => {
        expect(layoutTierFor({ width: t.minFullWidth + 400, height: t.minFullHeight + 400 }, presetId)).toBe('full')
      })

      it('Short window — đúng biên minFullWidth, chiều cao kém minFullHeight 1 (còn ≥ minShortHeight) ⇒ short (hàng "Short window")', () => {
        expect(layoutTierFor({ width: t.minFullWidth, height: t.minFullHeight - 1 }, presetId)).toBe('short')
      })

      it('Short window — đúng biên minShortHeight ⇒ vẫn short, không rơi xuống narrow (biên đóng phía dưới)', () => {
        expect(layoutTierFor({ width: t.minFullWidth, height: t.minShortHeight }, presetId)).toBe('short')
      })

      it('Narrow (vế chiều rộng) — đúng biên minSupportedWidth, kém minFullWidth ⇒ narrow, bất kể chiều cao thoải mái', () => {
        expect(layoutTierFor({ width: t.minSupportedWidth, height: t.minFullHeight + 1000 }, presetId)).toBe('narrow')
      })

      it('Narrow (vế chiều rộng) — minFullWidth kém 1 ⇒ narrow (biên đóng phía trên của minFullWidth)', () => {
        expect(layoutTierFor({ width: t.minFullWidth - 1, height: t.minFullHeight + 1000 }, presetId)).toBe('narrow')
      })

      it('Narrow (vế chiều cao, "very short") — chiều rộng THOẢI MÁI (full-width) nhưng minShortHeight kém 1 ⇒ narrow, không phải short/full', () => {
        expect(layoutTierFor({ width: t.minFullWidth + 1000, height: t.minShortHeight - 1 }, presetId)).toBe('narrow')
      })

      it('Unsupported — minSupportedWidth kém 1 ⇒ unsupported (biên đóng phía dưới của minSupportedWidth)', () => {
        expect(layoutTierFor({ width: t.minSupportedWidth - 1, height: t.minFullHeight }, presetId)).toBe('unsupported')
      })

      it('Unsupported — chiều rộng cực nhỏ, chiều cao cực lớn vẫn unsupported (chiều rộng thắng mọi mệnh đề chiều cao)', () => {
        expect(layoutTierFor({ width: 1, height: 100000 }, presetId)).toBe('unsupported')
      })

      it('Ưu tiên trên-xuống — vừa rất hẹp (dưới minSupportedWidth) vừa rất thấp (dưới minShortHeight) ⇒ unsupported, không phải narrow (hàng "Narrow or very short" thua hàng "Unsupported")', () => {
        expect(
          layoutTierFor({ width: t.minSupportedWidth - 1, height: t.minShortHeight - 1 }, presetId),
        ).toBe('unsupported')
      })
    })
  }

  it('Preset switched at a fixed size — cùng (w, h) tra qua hai preset (hàng "Preset switched at a fixed size")', () => {
    const gridT = LAYOUT_THRESHOLDS['layout.preset_grid']
    const columnsT = LAYOUT_THRESHOLDS['layout.preset_columns']
    const w = gridT.minFullWidth
    const h = gridT.minFullHeight
    const tierGrid = layoutTierFor({ width: w, height: h }, 'layout.preset_grid')
    const tierColumns = layoutTierFor({ width: w, height: h }, 'layout.preset_columns')

    // ⚠️ PHÁT HIỆN, không phải một đối chứng bị giả mạo: Ⓑ-1 (`layout.preset_columns`) và
    // Ⓑ-2 (`layout.preset_grid`) mang CÙNG bốn số hạt giống hôm nay (`workspaceLayout.ts`
    // dòng "🔴 HẠT GIỐNG — cả hai preset cùng bốn số này hôm nay"), nên tầng ra CÙNG một giá
    // trị ở MỌI (w, h) cho tới khi Task 11 (Phase 5, người, hiệu chỉnh trên phần cứng thật)
    // tách hai bộ số ra. Case này ghim MỆNH ĐỀ ĐÚNG cho hôm nay (bằng nhau) — không ghim một
    // sự phân kỳ không có thật. Khi Task 11 đóng, case này phải đổi thành hai kỳ vọng RIÊNG.
    expect(tierColumns).toBe(tierGrid)
    expect(gridT).toEqual(columnsT)
  })
})

describe('layoutTierFor — đầu vào KHÔNG hữu hạn trả `null`, không bao giờ một tầng (🔴 bẫy im-lặng-thành-`full`)', () => {
  const NON_FINITE: readonly unknown[] = [NaN, Infinity, -Infinity, undefined]
  const FIXED = 999

  for (const presetId of PRESET_IDS) {
    for (const bad of NON_FINITE) {
      it(`preset ${presetId} — width=\`${String(bad)}\` ⇒ null`, () => {
        expect(layoutTierFor({ width: bad as number, height: FIXED }, presetId)).toBe(null)
      })
      it(`preset ${presetId} — height=\`${String(bad)}\` ⇒ null`, () => {
        expect(layoutTierFor({ width: FIXED, height: bad as number }, presetId)).toBe(null)
      })
      it(`preset ${presetId} — cả hai chiều=\`${String(bad)}\` ⇒ null`, () => {
        expect(layoutTierFor({ width: bad as number, height: bad as number }, presetId)).toBe(null)
      })
    }
  }

  it('null KHÔNG BAO GIỜ được đọc như một tầng hợp lệ — kiểu trả về đóng đúng bốn tầng + null', () => {
    const out = layoutTierFor({ width: NaN, height: 999 }, 'layout.preset_grid')
    const knownTiers: readonly LayoutTier[] = ['full', 'short', 'narrow', 'unsupported']
    expect(out).toBe(null)
    expect(knownTiers as readonly (LayoutTier | null)[]).not.toContain(out === null ? undefined : out)
  })
})

describe('FocusRegistry.release(owner, expected) — Story 4.12 Phase 3a, Task 6, Decision 2', () => {
  it('gỡ trần (không `expected`) vẫn xoá owner như trước — hành vi cũ giữ nguyên khi bỏ tham số', () => {
    const registry = createFocusRegistry()
    const resolveA = (): HTMLElement | null => null
    registry.declare('panel.lookup', resolveA)

    expect(registry.has('panel.lookup')).toBe(true)
    registry.release('panel.lookup')
    expect(registry.has('panel.lookup')).toBe(false)
  })

  it('`release(owner, expected)` khi `expected` ĐANG khai đúng ⇒ xoá (đường thường)', () => {
    const registry = createFocusRegistry()
    const resolveA = (): HTMLElement | null => null
    registry.declare('panel.lookup', resolveA)

    registry.release('panel.lookup', resolveA)
    expect(registry.has('panel.lookup')).toBe(false)
  })

  it('vượt mặt — khai lại bằng owner khác (b) rồi gỡ MUỘN bằng resolver cũ (a) ⇒ b SỐNG SÓT, không bị gỡ nhầm', () => {
    const registry = createFocusRegistry()
    const resolveA = (): HTMLElement | null => null
    const resolveB = (): HTMLElement | null => null

    registry.declare('panel.lookup', resolveA)
    registry.release('panel.lookup') // gỡ bản a (dock cũ tháo)
    registry.declare('panel.lookup', resolveB) // dock mới khai lại

    // Lượt gỡ MUỘN của a tới sau — expected vẫn là resolveA, nhưng byOwner đang giữ resolveB.
    registry.release('panel.lookup', resolveA)

    expect(registry.has('panel.lookup')).toBe(true)
    expect(registry.owners()).toContain('panel.lookup')
  })
})
