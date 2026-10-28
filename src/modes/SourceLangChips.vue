<script setup lang="ts">
import { computed } from 'vue'
import { t } from '../i18n'
import { sourceLangChoices } from './libraryWorkCard'

const props = withDefaults(
  defineProps<{
    modelValue: string
    name: string
    includeAuto?: boolean
  }>(),
  { includeAuto: true },
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const options = computed(() => sourceLangChoices().filter((o) => props.includeAuto || o.value !== 'auto'))
</script>

<template>
  <div class="lang-chips" role="radiogroup" :aria-label="t('mode.library.field_source_lang')">
    <label
      v-for="option in options"
      :key="option.value"
      class="lang-chip ui-chip"
      :class="{ 'lang-chip--on': modelValue === option.value }"
    >
      <input
        type="radio"
        class="lang-chip-input"
        :name="name"
        :value="option.value"
        :checked="modelValue === option.value"
        @change="emit('update:modelValue', option.value)"
      />
      <!-- aura-allow-text: option label is the output of t(). -->
      <span>{{ option.label }}</span>
    </label>
  </div>
</template>

<style scoped>
.lang-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.lang-chip {
  position: relative;
}

.lang-chip-input {
  position: absolute;
  inset: 0;
  margin: 0;
  opacity: 0;
  cursor: pointer;
}
</style>
