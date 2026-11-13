type BlockLike = { readonly kind: string; readonly segments?: readonly { readonly id: number; readonly is_marked: boolean }[] }

export function paragraphSegments(block: BlockLike | undefined) {
  return block?.kind === 'paragraph' ? (block.segments ?? []) : []
}
