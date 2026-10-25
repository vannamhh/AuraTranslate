import { onTestFinished } from 'vitest'

export function installFakeDragJs(): void {
  const listener = (event: MouseEvent): void => {
    if (event.target instanceof Element && event.target.closest('[data-tauri-drag-region]') !== null) {
      event.stopImmediatePropagation()
    }
  }
  document.addEventListener('mousedown', listener)
  onTestFinished(() => document.removeEventListener('mousedown', listener))
}

export function pressDragHeader(): void {
  const header = document.createElement('header')
  header.setAttribute('data-tauri-drag-region', 'deep')
  document.body.appendChild(header)
  header.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))
  header.remove()
}
