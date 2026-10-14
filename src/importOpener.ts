let recorded: HTMLElement | null = null

// WKWebView does not focus a button on click, so the pressed opener is recorded on pointerdown/mousedown/focus (the e2e driver sends mousedown only).
export function recordImportOpener(event: Event): void {
  const target = event.currentTarget
  recorded = target instanceof HTMLElement ? target : null
}

export function takeRecordedImportOpener(): HTMLElement | null {
  const opener = recorded
  recorded = null
  return opener !== null && opener.isConnected ? opener : null
}

export function resetImportOpener(): void {
  recorded = null
}
