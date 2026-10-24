import { beforeEach, describe, expect, it } from 'vitest'
import {
  currentSelectionText,
  currentSelectionTextForConcordance,
  focusSelectionSource,
  registerSelectionSurface,
} from '../../src/panels/selectionContract'

beforeEach(() => {
  document.body.innerHTML = ''
  window.getSelection()?.removeAllRanges()
})

function mountScoped(): { root: HTMLElement; cell: HTMLElement; text: Text } {
  const root = document.createElement('div')
  const other = document.createElement('div')
  other.className = 'other'
  other.textContent = 'xx'
  const cell = document.createElement('div')
  cell.setAttribute('data-col', 'src')
  cell.tabIndex = 0
  cell.textContent = '师父来了'
  root.append(other, cell)
  document.body.append(root)
  return { root, cell, text: cell.firstChild as Text }
}

function extendFromCaretTo(text: Text, offset: number): void {
  const selection = window.getSelection()!
  selection.setBaseAndExtent(selection.anchorNode!, selection.anchorOffset, text, offset)
}

describe('focusSelectionSource with a scoped surface', () => {
  it('puts the caret in the first matching cell and the surface owns the selection made from it', () => {
    const { root, cell, text } = mountScoped()
    registerSelectionSurface(root, 'source', undefined, false, '[data-col="src"]')

    expect(focusSelectionSource()).toBe(true)
    const selection = window.getSelection()!
    expect(cell.contains(selection.anchorNode)).toBe(true)
    expect(selection.isCollapsed).toBe(true)

    extendFromCaretTo(text, 2)
    expect(currentSelectionText()).toBe('师父')
  })

  it('returns false when the scope matches nothing', () => {
    const { root } = mountScoped()
    registerSelectionSurface(root, 'source', undefined, false, '[data-col="nope"]')

    expect(focusSelectionSource()).toBe(false)
  })
})

describe('one element registered under two roles with different scopes', () => {
  it('each role takes anchors in its own scope and releasing one keeps the other', () => {
    const root = document.createElement('div')
    const a = document.createElement('p')
    a.className = 'a'
    a.textContent = 'aaa'
    const b = document.createElement('p')
    b.className = 'b'
    b.textContent = 'bbb'
    root.append(a, b)
    document.body.append(root)
    const releaseSource = registerSelectionSurface(root, 'source', undefined, false, '.a')
    const releaseDisplay = registerSelectionSurface(root, 'display', undefined, false, '.b')
    const select = (el: HTMLElement): void => {
      const node = el.firstChild as Text
      const range = document.createRange()
      range.setStart(node, 0)
      range.setEnd(node, node.length)
      window.getSelection()!.removeAllRanges()
      window.getSelection()!.addRange(range)
    }

    select(a)
    expect(currentSelectionText()).toBe('aaa')
    select(b)
    expect(currentSelectionText()).toBe('')
    expect(currentSelectionTextForConcordance()).toBe('bbb')

    releaseSource()
    select(a)
    expect(currentSelectionTextForConcordance()).toBe('')
    select(b)
    expect(currentSelectionTextForConcordance()).toBe('bbb')

    releaseDisplay()
    expect(currentSelectionTextForConcordance()).toBe('')
  })
})
