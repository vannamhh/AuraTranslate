export const PAIR_ATTRIBUTE = 'data-review-pair'

export type ReviewSide = 'mine' | 'copy'

const scrollers: Record<ReviewSide, HTMLElement | null> = { mine: null, copy: null }
// Scrollers we moved ourselves: their next scroll event is the echo of our own write, not the user.
const echo = new Set<HTMLElement>()

function other(side: ReviewSide): ReviewSide {
  return side === 'mine' ? 'copy' : 'mine'
}

function itemsOf(container: HTMLElement): HTMLElement[] {
  return Array.from(container.querySelectorAll<HTMLElement>(`[${PAIR_ATTRIBUTE}]`))
}

function find(container: HTMLElement, key: string): HTMLElement | null {
  return itemsOf(container).find((el) => el.getAttribute(PAIR_ATTRIBUTE) === key) ?? null
}

function moveBy(container: HTMLElement, delta: number): void {
  if (delta === 0) return
  const before = container.scrollTop
  container.scrollTop = before + delta
  if (container.scrollTop !== before) echo.add(container)
}

export function registerReviewScroller(side: ReviewSide, element: HTMLElement | null): void {
  if (element === null && scrollers[side] !== null) echo.delete(scrollers[side])
  scrollers[side] = element
}

export function resetReviewScrollers(): void {
  scrollers.mine = null
  scrollers.copy = null
  echo.clear()
}

export function onReviewScroll(side: ReviewSide): void {
  const source = scrollers[side]
  const target = scrollers[other(side)]
  if (source === null || target === null) return
  if (echo.delete(source)) return

  const top = source.getBoundingClientRect().top
  const anchor = itemsOf(source).find((el) => el.getBoundingClientRect().bottom > top)
  if (anchor === undefined) return
  const counterpart = find(target, anchor.getAttribute(PAIR_ATTRIBUTE) ?? '')
  if (counterpart === null) return

  const anchorOffset = anchor.getBoundingClientRect().top - top
  const counterpartOffset = counterpart.getBoundingClientRect().top - target.getBoundingClientRect().top
  moveBy(target, counterpartOffset - anchorOffset)
}

export function scrollReviewPairIntoView(key: string): void {
  for (const container of [scrollers.mine, scrollers.copy]) {
    if (container === null) continue
    const item = find(container, key)
    if (item === null) continue
    moveBy(container, item.getBoundingClientRect().top - container.getBoundingClientRect().top)
  }
}
