/**
 * Resizes the e2e window until the layout tier reads `full`, for specs that need Lookup
 * docked in the grid rather than tabbed away or retreated to the drawer.
 *
 * The embedded driver's `setWindowSize` takes physical pixels while the app reports logical
 * ones, so no single request is correct across `devicePixelRatio`s; requesting an oversized
 * window lets the OS clamp it to the real screen instead.
 */

const MAX_WINDOW_SIDE_PX = 10_000

/**
 * @returns {Promise<void>}
 */
export async function ensureFullLayoutTier() {
  await browser.setWindowSize(MAX_WINDOW_SIDE_PX, MAX_WINDOW_SIDE_PX)

  let lastTier = null
  try {
    await browser.waitUntil(
      async () => {
        lastTier = await browser.execute(
          () =>
            document.querySelector('[data-layout-tier]')?.getAttribute('data-layout-tier') ??
            null,
        )
        return lastTier === 'full'
      },
      { timeout: 10_000, interval: 200 },
    )
  } catch {
    throw new Error(
      `[ensureFullLayoutTier] Cửa sổ không lên được tầng \`full\` sau khi phóng to hết cỡ ` +
        `(yêu cầu ${MAX_WINDOW_SIDE_PX}×${MAX_WINDOW_SIDE_PX}, đơn vị vật lý). Tầng đọc được ` +
        `lần cuối: ${lastTier ?? 'chưa đọc được lần nào'}.`,
    )
  }
}
