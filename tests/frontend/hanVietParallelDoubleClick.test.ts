import { afterEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import SourceHanViet from '../../src/panels/SourceHanViet.vue'

/** Two Han words around a non-Han fragment; the second word is a single character. */
const NGUYEN_VAN = '京都」，春'

function dung() {
  return mount(SourceHanViet, {
    props: { sourceText: NGUYEN_VAN, viewMode: 'parallel' as const, cuts: [], glossaryTerms: [], surfaceRole: 'cell' as const },
    attachTo: document.body,
  })
}

function selectAtStartOf(node: Node): void {
  const range = document.createRange()
  range.setStart(node, 0)
  range.collapse(true)
  window.getSelection()?.removeAllRanges()
  window.getSelection()?.addRange(range)
}

afterEach(() => {
  window.getSelection()?.removeAllRanges()
})

describe('SourceHanViet — double-click in the Song song view', () => {
  it('selects the base of the word under the pointer, not the reading of the word before it', () => {
    const w = dung()
    const rubies = w.element.querySelectorAll('.hv-parallel ruby')
    const last = rubies[rubies.length - 1]
    if (last === undefined || !(last.firstChild instanceof Text)) throw new Error('no ruby base')
    selectAtStartOf(last.firstChild)

    last.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, detail: 2 }))

    const range = window.getSelection()?.getRangeAt(0)
    expect(range?.startContainer).toBe(last.firstChild)
    expect(range?.toString()).toBe('春')
    w.unmount()
  })

  it('leaves a single click alone', () => {
    const w = dung()
    const ruby = w.element.querySelector('.hv-parallel ruby')
    if (ruby === null || !(ruby.firstChild instanceof Text)) throw new Error('no ruby base')
    selectAtStartOf(ruby.firstChild)

    ruby.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, detail: 1 }))

    expect(window.getSelection()?.isCollapsed).toBe(true)
    w.unmount()
  })
})
