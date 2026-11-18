/**
 * Shared e2e helpers for the one-width, one-scroll-region overlay frame (`.ui-overlay`).
 * Later overlay migrations reuse them instead of re-deriving the measurements.
 */

/** `setWindowSize` units differ by driver and dpr, so accept whichever factor lands the logical width. */
export async function resizeLogical(width, height) {
  const dpr = await browser.execute(() => window.devicePixelRatio)
  let inner = 0
  for (const factor of [dpr, 1]) {
    await browser.setWindowSize(Math.round(width * factor), Math.round(height * factor))
    await browser.pause(400)
    inner = await browser.execute(() => window.innerWidth)
    if (Math.abs(inner - width) <= 12) return
  }
  throw new Error(`window could not reach logical width ${width} (last innerWidth ${inner}, dpr ${dpr})`)
}

/**
 * Measure a frame element: its width, every vertical scroller inside it (the frame itself
 * included), and how many of those sit inside another one. Elements without a box, or inside
 * a closed `details` (WebKit keeps a box for its hidden panel), are not scroll regions.
 *
 * @param {string} selector CSS selector of the frame element
 * @returns {Promise<{ width: number, scrollers: string[], nested: number }>}
 */
export async function measureOverlayFrame(selector) {
  return browser.execute((sel) => {
    const frame = document.querySelector(sel)
    if (frame === null) throw new Error(`no element matches ${sel}`)
    const declares = (el) => {
      const y = getComputedStyle(el).overflowY
      return y === 'auto' || y === 'scroll'
    }
    const all = [frame, ...frame.querySelectorAll('*')].filter((el) => el.tagName !== 'TEXTAREA' && el.getClientRects().length > 0 && el.closest('details:not([open])') === null)
    const scrollers = all.filter(declares)
    const nested = scrollers.filter((el) => scrollers.some((other) => other !== el && other.contains(el))).length
    const label = (el) => (el.className && typeof el.className === 'string' ? `${el.tagName.toLowerCase()}.${el.className.trim().split(/\s+/).join('.')}` : el.tagName.toLowerCase())
    return { width: frame.getBoundingClientRect().width, scrollers: scrollers.map(label), nested }
  }, selector)
}

/**
 * Is the element's box fully inside the viewport?
 *
 * @param {WebdriverIO.Element} element
 * @returns {Promise<boolean>}
 */
export async function isInViewport(element) {
  return browser.execute((el) => {
    const r = el.getBoundingClientRect()
    return r.top >= 0 && r.left >= 0 && r.bottom <= window.innerHeight && r.right <= window.innerWidth && r.height > 0
  }, element)
}
