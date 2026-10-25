<script setup lang="ts">
import { computed, nextTick, useTemplateRef } from 'vue'
import DropdownMenu from './DropdownMenu.vue'

export interface SelectOption {
  value: string
  label: string
  disabled?: boolean
}

let groupCounter = 0

const props = defineProps<{
  modelValue: string
  options: SelectOption[]
  variant: 'chip' | 'field'
  label: string
  placeholder?: string
  triggerText?: string
  active?: boolean
  disabled?: boolean
  name?: string
  optionAttr?: string
  columns?: 2 | 3
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  change: [value: string]
}>()

const menu = useTemplateRef<InstanceType<typeof DropdownMenu>>('menu')
groupCounter += 1
const groupName = computed(() => props.name ?? `select-menu-${groupCounter}`)

const pickedLabel = computed(() => props.options.find((option) => option.value === props.modelValue)?.label ?? null)
const shownText = computed(() => props.triggerText ?? pickedLabel.value ?? props.placeholder ?? '')
const triggerAria = computed(() =>
  shownText.value === '' || shownText.value === props.label ? props.label : `${props.label}: ${shownText.value}`,
)

function optionAttrs(value: string): Record<string, string> {
  return props.optionAttr === undefined ? {} : { [props.optionAttr]: value }
}

function onPick(value: string): void {
  emit('update:modelValue', value)
  emit('change', value)
}

async function focusPicked(): Promise<void> {
  await nextTick()
  const root = menu.value?.$el as HTMLElement | undefined
  if (root === undefined) return
  const radios = Array.from(root.querySelectorAll<HTMLInputElement>('input[type="radio"]:not(:disabled)'))
  const target = radios.find((radio) => radio.checked) ?? radios.at(0)
  target?.focus()
}
</script>

<template>
  <DropdownMenu
    ref="menu"
    :variant="variant"
    :label="shownText === '' && variant === 'field' ? ' ' : shownText"
    :aria-label="triggerAria"
    :active="active"
    :disabled="disabled"
    :class="{ 'menu--columns': columns !== undefined }"
    @open="focusPicked"
  >
    <div class="select-options" :class="columns === undefined ? undefined : `select-options--cols-${columns}`">
      <label
        v-for="option in options"
        :key="option.value"
        class="select-item"
        :class="{ 'select-item--disabled': option.disabled === true }"
        :data-menu-close="option.disabled === true ? undefined : ''"
      >
        <input
          type="radio"
          :name="groupName"
          :value="option.value"
          :checked="option.value === modelValue"
          :disabled="option.disabled === true"
          data-select-option
          v-bind="optionAttrs(option.value)"
          @change="onPick(option.value)"
        />
        <!-- aura-allow-text: option label is the output of t() or user data. -->
        <span>{{ option.label }}</span>
      </label>
    </div>
  </DropdownMenu>
</template>

<style scoped>
.select-options {
  display: flex;
  flex-direction: column;
}

.select-options--cols-2,
.select-options--cols-3 {
  display: block;
  column-gap: 0;
}

.select-options--cols-2 {
  columns: 2 176px;
}

.select-options--cols-3 {
  columns: 3 176px;
}

.select-options--cols-2 .select-item,
.select-options--cols-3 .select-item {
  break-inside: avoid;
}

.select-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  text-align: left;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  color: var(--color-on-surface);
}

.select-item--disabled {
  color: var(--color-on-surface-variant);
  cursor: not-allowed;
}
</style>
