import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'

const workId = ref<string | null>(null)

/** `null` until a Work was created or opened in this session; Rust holds no Work at startup. */
export const openWorkId: DeepReadonly<Ref<string | null>> = readonly(workId)

export function setOpenWorkId(id: string): void {
  workId.value = id
}

export function resetOpenWorkId(): void {
  workId.value = null
}
