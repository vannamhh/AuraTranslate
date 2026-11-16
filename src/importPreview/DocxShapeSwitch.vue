<script setup lang="ts">
import { computed } from 'vue'
import { t, tError } from '../i18n'
import type { IpcError } from '../i18n'
import type { DocxImportShape } from '../config/project'
import SelectMenu from '../SelectMenu.vue'

const props = defineProps<{
  current: DocxImportShape
  prefix: 'ip' | 'bip'
  disabled: boolean
  error: IpcError | null
}>()

const emit = defineEmits<{
  (e: 'change', target: DocxImportShape): void
}>()

const options = computed(() => [
  { value: 'text', label: t('mode.library.preview.docx_shape_text') },
  { value: 'bilingual', label: t('mode.library.preview.docx_shape_bilingual') },
])

const hook = computed(() => ({ [`data-${props.prefix}-docx-shape`]: '' }))

function onChange(raw: string): void {
  if ((raw === 'text' || raw === 'bilingual') && raw !== props.current) emit('change', raw)
}
</script>

<template>
  <div :class="`${prefix}-docx-shape`" v-bind="hook">
    <SelectMenu
      :id="`${prefix}-docx-shape-select`"
      variant="field"
      option-attr="data-docx-shape-option"
      :model-value="current"
      :options="options"
      :label="t('mode.library.preview.docx_shape_label')"
      :disabled="disabled"
      @change="onChange"
    />
    <p v-if="error !== null" :class="[`${prefix}-status`, `${prefix}-error`, 'docx-shape-error']" role="alert">
      <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
      {{ tError(error) }}
    </p>
  </div>
</template>

<style scoped>
.docx-shape-error {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-error);
}
</style>
