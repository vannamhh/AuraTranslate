import { readonly, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { adoptQuickAddRestore } from '../glossaryQuickAddState'
import { editorCaretSegmentId, editorSegments, requestCurrentEditorCaretPlacement, setEditorCaret } from './editorPanelState'

const PRESS_FRESH_MS = 1000

export type SentencePress = {
  readonly segmentId: number | null
  readonly x: number
  readonly y: number
  readonly previousCaret: number | null
  readonly previousRange: Range | null
}

export type SentenceMenu = {
  readonly segmentId: number
  readonly x: number
  readonly y: number
}

type Restore = { readonly caret: number | null; readonly range: Range | null }

export function isSecondaryPress(
  event: { button: number; ctrlKey: boolean },
  platform: { isMac: boolean },
): boolean {
  return event.button === 2 || (platform.isMac && event.button === 0 && event.ctrlKey)
}

export function placeMenu(
  at: { x: number; y: number },
  size: { width: number; height: number },
  viewport: { width: number; height: number },
): { x: number; y: number } {
  const flippedX = at.x + size.width > viewport.width ? at.x - size.width : at.x
  const flippedY = at.y + size.height > viewport.height ? at.y - size.height : at.y
  const x = Math.min(Math.max(0, flippedX), Math.max(0, viewport.width - size.width))
  const y = Math.min(Math.max(0, flippedY), Math.max(0, viewport.height - size.height))
  return { x, y }
}

const menu = shallowRef<SentenceMenu | null>(null)
export const sentenceMenu: DeepReadonly<Ref<SentenceMenu | null>> = readonly(menu)

export function sentenceMenuIsOpen(): boolean {
  return menu.value !== null
}

let lastPress: { press: SentencePress; at: number } | null = null
let restore: Restore | null = null

export function recordSentencePress(press: SentencePress): void {
  lastPress = { press, at: performance.now() }
}

export function currentSelectionRange(): Range | null {
  const selection = window.getSelection()
  return selection === null || selection.rangeCount === 0 ? null : selection.getRangeAt(0).cloneRange()
}

function takePress(): SentencePress | null {
  const held = lastPress
  lastPress = null
  return held !== null && performance.now() - held.at <= PRESS_FRESH_MS ? held.press : null
}

function targetCell(segmentId: number): HTMLElement | null {
  return document.querySelector<HTMLElement>(`[data-col="tgt"][data-segment-id="${String(segmentId)}"]`)
}

export function openSentenceMenu(): boolean {
  const press = takePress()
  if (press !== null) {
    if (press.segmentId === null) return false
    if (editorCaretSegmentId.value !== press.segmentId) setEditorCaret(press.segmentId)
    restore = { caret: press.previousCaret, range: press.previousRange }
    menu.value = { segmentId: press.segmentId, x: press.x, y: press.y }
    return true
  }

  const caretId = editorCaretSegmentId.value
  if (caretId === null) return false
  const rect = targetCell(caretId)?.getBoundingClientRect()
  restore = { caret: caretId, range: currentSelectionRange() }
  menu.value = { segmentId: caretId, x: rect?.left ?? 0, y: rect?.bottom ?? 0 }
  return true
}

export function resetSentenceMenu(): void {
  menu.value = null
  restore = null
  lastPress = null
}

function restoreBeforeOpen(before: Restore): void {
  const selection = window.getSelection()
  if (before.range !== null && before.range.startContainer.isConnected && before.range.endContainer.isConnected) {
    const host = before.range.startContainer instanceof Element
      ? before.range.startContainer
      : before.range.startContainer.parentElement
    host?.closest<HTMLElement>('[data-col="tgt"]')?.focus({ preventScroll: true })
    if (selection !== null) {
      selection.removeAllRanges()
      selection.addRange(before.range)
    }
  } else if (selection !== null && selection.rangeCount > 0 && !selection.isCollapsed) {
    selection.collapseToStart()
  }
  const caret = before.caret !== null && editorSegments.value.some((s) => s.id === before.caret) ? before.caret : null
  if (editorCaretSegmentId.value !== caret) setEditorCaret(caret)
}

export function closeSentenceMenu(reason: 'chosen' | 'dismissed' | 'overlay' | 'glossary'): void {
  const open = menu.value
  const before = restore
  menu.value = null
  restore = null
  if (open === null) return
  if (reason === 'dismissed') {
    if (before !== null) restoreBeforeOpen(before)
    return
  }
  if (reason === 'glossary') {
    if (before !== null && !adoptQuickAddRestore(() => restoreBeforeOpen(before))) restoreBeforeOpen(before)
    return
  }
  if (reason === 'overlay') {
    // WebKit keeps the selection when a mouse click moves focus, so a key pressed in the overlay would land in the cell behind it.
    window.getSelection()?.removeAllRanges()
    return
  }
  if (editorCaretSegmentId.value !== open.segmentId) return
  const cell = targetCell(open.segmentId)
  if (cell === null) return
  const selection = window.getSelection()
  const anchor = selection?.anchorNode ?? null
  if (document.activeElement === cell && anchor !== null && cell.contains(anchor)) selection?.collapseToEnd()
  else requestCurrentEditorCaretPlacement()
}
