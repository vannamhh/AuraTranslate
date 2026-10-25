import { afterEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import SelectMenu from '../../src/SelectMenu.vue'

const OPTIONS = [
  { value: 'a', label: 'Alpha' },
  { value: 'b', label: 'Beta' },
  { value: 'c', label: 'Gamma', disabled: true },
]

let wrapper: ReturnType<typeof mount> | null = null

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

function build(props: Record<string, unknown> = {}): ReturnType<typeof mount> {
  wrapper = mount(SelectMenu, {
    attachTo: document.body,
    props: { modelValue: 'b', options: OPTIONS, variant: 'field', label: 'Letter', ...props },
  })
  return wrapper
}

function details(): HTMLDetailsElement {
  return document.querySelector<HTMLDetailsElement>('details[data-dropdown-menu]')!
}

async function open(): Promise<void> {
  details().open = true
  details().dispatchEvent(new Event('toggle'))
  await wrapper!.vm.$nextTick()
  await wrapper!.vm.$nextTick()
}

function radio(value: string): HTMLInputElement {
  return details().querySelector<HTMLInputElement>(`input[type="radio"][value="${value}"]`)!
}

describe('SelectMenu.vue', () => {
  it('renders every option as a radio with the model checked and its label on the trigger', () => {
    build()
    expect(details().querySelectorAll('input[type="radio"]')).toHaveLength(3)
    expect(radio('b').checked).toBe(true)
    expect(radio('a').checked).toBe(false)
    expect(radio('c').disabled).toBe(true)
    expect(details().querySelector('summary')!.textContent).toContain('Beta')
  })

  it('shows the placeholder when the model matches no option', () => {
    build({ modelValue: '', placeholder: 'Pick one' })
    expect(details().querySelector('summary')!.textContent).toContain('Pick one')
  })

  it('picking an option emits update:modelValue and change, and a mouse click closes the menu', async () => {
    const w = build()
    await open()
    await w.get('input[value="a"]').setValue(true)
    expect(w.emitted('update:modelValue')).toEqual([['a']])
    expect(w.emitted('change')).toEqual([['a']])
    radio('a').closest('label')!.dispatchEvent(new MouseEvent('click', { bubbles: true, detail: 1 }))
    expect(details().open).toBe(false)
  })

  it('a keyboard-originated click on a radio (arrow keys) leaves the menu open', async () => {
    build()
    await open()
    radio('a').dispatchEvent(new MouseEvent('click', { bubbles: true, detail: 0 }))
    expect(details().open).toBe(true)
  })

  it('opening focuses the checked radio', async () => {
    build()
    await open()
    expect(document.activeElement).toBe(radio('b'))
  })

  it('Escape closes and returns focus to the trigger', async () => {
    build()
    await open()
    radio('b').dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    expect(details().open).toBe(false)
    expect(document.activeElement).toBe(details().querySelector('summary'))
  })

  it('Enter on a radio closes and returns focus to the trigger', async () => {
    build()
    await open()
    radio('b').dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))
    expect(details().open).toBe(false)
    expect(document.activeElement).toBe(details().querySelector('summary'))
  })

  it('disabled blocks opening and marks the trigger', async () => {
    build({ disabled: true })
    await open()
    expect(details().open).toBe(false)
    expect(details().querySelector('summary')!.getAttribute('aria-disabled')).toBe('true')
  })
})
