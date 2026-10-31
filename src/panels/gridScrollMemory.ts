let remembered: { chapterId: number; top: number } | null = null

export function rememberGridScroll(chapterId: number | null, top: number): void {
  if (chapterId === null) return
  remembered = { chapterId, top }
}

export function recalledGridScroll(chapterId: number | null): number | null {
  if (chapterId === null || remembered === null || remembered.chapterId !== chapterId) return null
  return remembered.top > 0 ? remembered.top : null
}

export function resetGridScrollMemory(): void {
  remembered = null
}
