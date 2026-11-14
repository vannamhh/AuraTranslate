import { readonly, shallowRef } from 'vue'
import type { DeepReadonly, Ref } from 'vue'
import { lookupDictionary } from '../config/dict'
import type { LookupResponse } from '../config/dict'
import type { IpcError } from '../i18n'
import { setPopupPinTargets } from './lookupHistoryState'

// Imports nothing from `lookupPanelState.ts`: that module imports this one to close the popup.

const openChar = shallowRef<string | null>(null)
const anchorButton = shallowRef<HTMLElement | null>(null)
const response = shallowRef<LookupResponse | null>(null)
const error = shallowRef<IpcError | null>(null)
const pending = shallowRef(false)

let sequence = 0

export const lookupCharPopupChar: DeepReadonly<Ref<string | null>> = readonly(openChar)
export const lookupCharPopupAnchor: Readonly<Ref<HTMLElement | null>> = anchorButton
export const lookupCharPopupResponse: Readonly<Ref<LookupResponse | null>> = response
export const lookupCharPopupError: DeepReadonly<Ref<IpcError | null>> = readonly(error)
export const lookupCharPopupPending: DeepReadonly<Ref<boolean>> = readonly(pending)

export function closeLookupCharPopup(): void {
  sequence += 1
  openChar.value = null
  anchorButton.value = null
  response.value = null
  error.value = null
  pending.value = false
  setPopupPinTargets(null)
}

export function resetLookupCharPopup(): void {
  closeLookupCharPopup()
}

/** Opens the popup under `button` for `char`; the same button again closes it. */
export function toggleLookupCharPopup(button: HTMLElement, char: string): void {
  if (openChar.value === char && anchorButton.value === button) {
    closeLookupCharPopup()
    return
  }
  void openLookupCharPopup(button, char)
}

async function openLookupCharPopup(button: HTMLElement, char: string): Promise<void> {
  const mine = ++sequence
  openChar.value = char
  anchorButton.value = button
  response.value = null
  error.value = null
  pending.value = true
  setPopupPinTargets(null)

  const { response: result, error: err } = await lookupDictionary(char)
  if (mine !== sequence) return

  pending.value = false
  response.value = result
  error.value = err
  setPopupPinTargets(err === null ? result : null)
}
