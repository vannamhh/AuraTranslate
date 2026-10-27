import { readonly, ref } from 'vue'
import type { DeepReadonly, Ref } from 'vue'

const workId = ref<string | null>(null)
const workName = ref<string | null>(null)

/** `null` until a Work was created or opened in this session; Rust holds no Work at startup. */
export const openWorkId: DeepReadonly<Ref<string | null>> = readonly(workId)

export const openWorkName: DeepReadonly<Ref<string | null>> = readonly(workName)

export function setOpenWork(id: string, name: string): void {
  workId.value = id
  workName.value = name
}

export function resetOpenWorkId(): void {
  workId.value = null
  workName.value = null
}
