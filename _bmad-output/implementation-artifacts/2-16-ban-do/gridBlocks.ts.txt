export const GRID_BLOCK_ROWS = 200

export type GridBlock = { readonly start: number; readonly rows: number }

export function gridBlocksOf(total: number, blockRows: number = GRID_BLOCK_ROWS): readonly GridBlock[] {
  const blocks: GridBlock[] = []
  for (let start = 0; start < total; start += blockRows) {
    blocks.push({ start, rows: Math.min(blockRows, total - start) })
  }
  return blocks
}
