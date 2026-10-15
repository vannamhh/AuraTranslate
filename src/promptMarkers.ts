export type PromptBodySegment = { text: string; kind: 'plain' | 'known' | 'unknown' }

/**
 * Splits a prompt body into plain text and `{{name}}` markers with the same left-to-right rule as
 * `core::promptset::vars::scan_markers`: the nearest `}}` closes a `{{`, a token that itself holds
 * `{{` leaves the outer `{{` as plain text, and a `{{` with no closing `}}` ends the scan.
 * `known` lists the recognised variable names.
 */
export function splitPromptBody(body: string, known: readonly string[]): PromptBodySegment[] {
  const out: PromptBodySegment[] = []
  let plainFrom = 0
  let at = 0
  for (;;) {
    const open = body.indexOf('{{', at)
    if (open === -1) break
    const close = body.indexOf('}}', open + 2)
    if (close === -1) break
    const token = body.slice(open + 2, close)
    if (token.includes('{{')) {
      at = open + 2
      continue
    }
    if (open > plainFrom) out.push({ text: body.slice(plainFrom, open), kind: 'plain' })
    out.push({ text: body.slice(open, close + 2), kind: known.includes(token) ? 'known' : 'unknown' })
    plainFrom = close + 2
    at = plainFrom
  }
  if (plainFrom < body.length) out.push({ text: body.slice(plainFrom), kind: 'plain' })
  return out
}
