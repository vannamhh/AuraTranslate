import { execFileSync } from 'node:child_process'

const CLICLICK = '/usr/local/bin/cliclick'

function cliclick(...args) {
  execFileSync(CLICLICK, args, { timeout: 15_000 })
}

/**
 * Real OS pointer events through `cliclick`: the WebDriver pointer actions never start a text
 * selection on non-editable text in WKWebView (no `selectstart`), so a real drag over source text
 * can only be made from outside the driver. Needs the window frontmost and an unlocked screen.
 */
function osascript(script) {
  return execFileSync('osascript', ['-e', script], { timeout: 15_000, encoding: 'utf8' }).trim()
}

/**
 * Puts the app window frontmost at a known frame and returns the viewport-to-screen offset.
 * `window.screenX/Y` is not trusted: on a two-display Mac it is in another coordinate space than
 * `cliclick`'s, so the frame is set and read back through System Events instead.
 */
export async function calibrateOsMouse() {
  const pid = execFileSync('pgrep', ['-x', 'auratranslate'], { encoding: 'utf8' }).trim().split('\n')[0]
  const proc = `first application process whose unix id is ${pid}`
  osascript(`tell application "System Events" to tell (${proc}) to set frontmost to true`)
  osascript(`tell application "System Events" to tell (${proc}) to tell window 1 to set size to {1200, 900}`)
  osascript(`tell application "System Events" to tell (${proc}) to tell window 1 to set position to {200, 25}`)
  await browser.pause(500)
  const frame = osascript(
    `tell application "System Events"
tell (${proc})
set p to position of window 1
set z to size of window 1
return ((item 1 of p) as text) & "," & ((item 2 of p) as text) & "," & ((item 1 of z) as text) & "," & ((item 2 of z) as text)
end tell
end tell`,
  )
    .split(',')
    .map(Number)
  const inner = await browser.execute(() => ({ w: window.innerWidth, h: window.innerHeight }))
  const offset = {
    offsetX: frame[0] + (frame[2] - inner.w),
    offsetY: frame[1] + (frame[3] - inner.h),
  }

  await browser.execute(() => {
    window.__osMouse = null
    document.addEventListener(
      'mousedown',
      (e) => {
        window.__osMouse = { cx: e.clientX, cy: e.clientY }
      },
      true,
    )
  })
  const probe = { x: 400, y: 300 }
  for (const dx of [0, 5, -5]) {
    cliclick(`c:${probe.x + offset.offsetX + dx},${probe.y + offset.offsetY}`)
    await browser.pause(250)
    const seen = await browser.execute(() => window.__osMouse)
    if (seen !== null) {
      if (Math.abs(seen.cx - (probe.x + dx)) > 2 || Math.abs(seen.cy - probe.y) > 2) {
        throw new Error(`[BÀN ĐO HỎNG] viewport offset is off: sent (${probe.x + dx},${probe.y}), page saw (${seen.cx},${seen.cy}).`)
      }
      return offset
    }
  }
  throw new Error('[BÀN ĐO HỎNG] cliclick produced no mousedown in the webview (window not frontmost, screen locked or no accessibility grant).')
}

export function osDrag(offset, from, to, steps = 12) {
  const at = (p) => `${Math.round(p.x + offset.offsetX)},${Math.round(p.y + offset.offsetY)}`
  const args = [`m:${at(from)}`, 'w:80', `dd:${at(from)}`, 'w:80']
  for (let i = 1; i <= steps; i += 1) {
    const p = { x: from.x + ((to.x - from.x) * i) / steps, y: from.y + ((to.y - from.y) * i) / steps }
    args.push(`dm:${at(p)}`, 'w:25')
  }
  args.push(`du:${at(to)}`, 'w:150')
  cliclick(...args)
}
