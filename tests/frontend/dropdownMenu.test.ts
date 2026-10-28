import { afterEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import DropdownMenu from '../../src/DropdownMenu.vue'
import { installFakeDragJs, pressDragHeader } from './support/dragRegionPress'

const Host = defineComponent({
  render() {
    return h('div', [
      h(DropdownMenu, { variant: 'chip', label: 'one', 'data-id': 'one' }, () => [
        h('button', { 'data-menu-close': '', 'data-item': 'close' }, 'close'),
        h('button', { 'data-item': 'stay' }, 'stay'),
      ]),
      h(DropdownMenu, { variant: 'chip', label: 'two', 'data-id': 'two' }, () => [h('button', 'x')]),
    ])
  },
})

let wrapper: ReturnType<typeof mount> | null = null

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

function menus(): HTMLDetailsElement[] {
  return Array.from(document.querySelectorAll<HTMLDetailsElement>('details[data-dropdown-menu]'))
}

async function open(menu: HTMLDetailsElement): Promise<void> {
  menu.open = true
  menu.dispatchEvent(new Event('toggle'))
  await wrapper!.vm.$nextTick()
}

describe('DropdownMenu.vue', () => {
  it('a more trigger is a quiet button unless the caller names its own trigger classes', () => {
    const quiet = mount(DropdownMenu, { props: { variant: 'more', label: '⋯' } })
    expect(quiet.get('summary').classes()).toEqual(['menu-trigger', 'ui-btn', 'ui-btn--quiet'])
    quiet.unmount()
    const cover = mount(DropdownMenu, { props: { variant: 'more', label: '⋯', triggerClass: 'ui-btn--sm ui-btn--cover-plain' } })
    expect(cover.get('summary').classes()).toEqual(['menu-trigger', 'ui-btn', 'ui-btn--sm', 'ui-btn--cover-plain'])
    cover.unmount()
  })

  it('Escape closes an open menu and focuses its summary', async () => {
    wrapper = mount(Host, { attachTo: document.body })
    const [first] = menus()
    await open(first!)
    first!.querySelector('button')!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(first!.open).toBe(false)
    expect(document.activeElement).toBe(first!.querySelector('summary'))
  })

  it('opening one menu closes another open one', async () => {
    wrapper = mount(Host, { attachTo: document.body })
    const [first, second] = menus()
    await open(first!)
    await open(second!)
    expect(second!.open).toBe(true)
    expect(first!.open).toBe(false)
  })

  it('clicking a [data-menu-close] item closes the menu, other items leave it open', async () => {
    wrapper = mount(Host, { attachTo: document.body })
    const [first] = menus()
    await open(first!)
    first!.querySelector<HTMLElement>('[data-item="stay"]')!.click()
    expect(first!.open).toBe(true)
    first!.querySelector<HTMLElement>('[data-menu-close]')!.click()
    expect(first!.open).toBe(false)
  })

  it('a press on a drag-region header closes an open menu although drag.js swallows the bubble phase', async () => {
    installFakeDragJs()
    wrapper = mount(Host, { attachTo: document.body })
    const [first] = menus()
    await open(first!)
    pressDragHeader()
    expect(first!.open).toBe(false)
  })
})
