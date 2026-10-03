/** The Concordance selection reader works on `display` surfaces, the dictionary path does not. */
import { beforeEach, describe, expect, it } from 'vitest'
import {
  currentSelectionIsVietnameseForConcordance,
  currentSelectionText,
  currentSelectionTextForConcordance,
  registerSelectionSurface,
} from '../../src/panels/selectionContract'

beforeEach(() => {
  document.body.innerHTML = ''
  window.getSelection()?.removeAllRanges()
})

function selectIn(role: 'source' | 'display', text: string, vietnamese = false): void {
  const box = document.createElement('div')
  const inner = document.createElement('span')
  inner.textContent = text
  box.append(inner)
  document.body.append(box)
  registerSelectionSurface(box, role, undefined, vietnamese)
  const node = inner.firstChild as Text
  const range = document.createRange()
  range.setStart(node, 0)
  range.setEnd(node, text.length)
  const selection = window.getSelection()
  selection?.removeAllRanges()
  selection?.addRange(range)
}

describe('currentSelectionTextForConcordance', () => {
  it('returns text on a display surface where currentSelectionText returns empty, and reports it Vietnamese', () => {
    selectIn('display', 'sư phụ', true)
    expect(currentSelectionText()).toBe('')
    expect(currentSelectionTextForConcordance()).toBe('sư phụ')
    expect(currentSelectionIsVietnameseForConcordance()).toBe(true)
  })

  it('returns the same text on a source surface and does not report it Vietnamese', () => {
    selectIn('source', '师父')
    expect(currentSelectionText()).toBe('师父')
    expect(currentSelectionTextForConcordance()).toBe('师父')
    expect(currentSelectionIsVietnameseForConcordance()).toBe(false)
  })

  it('returns empty and not Vietnamese when nothing is selected', () => {
    expect(currentSelectionTextForConcordance()).toBe('')
    expect(currentSelectionIsVietnameseForConcordance()).toBe(false)
  })
})
