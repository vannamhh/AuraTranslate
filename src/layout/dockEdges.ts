import type { DockviewApi } from 'dockview-vue'
import { edgeGroupIds } from './dockTree'
import type { SerializedGrid } from './dockTree'

/** Marks each grid group that touches a window edge, so its panel can pad that side to `--space-screen-inline`. */
export function syncDockEdgeClasses(api: DockviewApi): void {
  const { left, right } = edgeGroupIds((api.toJSON() as unknown as { grid: SerializedGrid }).grid)
  for (const group of api.groups) {
    group.element.classList.toggle('aura-dock-edge-left', left.has(group.id))
    group.element.classList.toggle('aura-dock-edge-right', right.has(group.id))
  }
}
