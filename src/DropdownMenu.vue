<script lang="ts">
const mountedMenus = new Set<HTMLDetailsElement>()
</script>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, useTemplateRef } from 'vue'

const props = defineProps<{
  variant: 'chip' | 'more' | 'field'
  label: string
  active?: boolean
  ariaLabel?: string
  disabled?: boolean
}>()

const emit = defineEmits<{ open: [] }>()

const root = useTemplateRef<HTMLDetailsElement>('root')

function closeMenu(): void {
  if (root.value !== null) root.value.open = false
}

function onToggle(): void {
  if (root.value === null || !root.value.open) return
  if (props.disabled === true) {
    root.value.open = false
    return
  }
  for (const other of mountedMenus) {
    if (other !== root.value && other.open) other.open = false
  }
  emit('open')
}

function onKeydown(event: KeyboardEvent): void {
  if (root.value === null) return
  if (props.disabled === true && (event.key === 'Enter' || event.key === ' ') && event.target instanceof HTMLElement && event.target.tagName === 'SUMMARY') {
    event.preventDefault()
    return
  }
  if (!root.value.open) return
  const closesOnEnter =
    event.key === 'Enter' && event.target instanceof HTMLInputElement && event.target.type === 'radio'
  if (event.key !== 'Escape' && !closesOnEnter) return
  event.preventDefault()
  event.stopPropagation()
  closeMenu()
  root.value.querySelector<HTMLElement>('summary')?.focus()
}

function onDocumentMouseDown(event: MouseEvent): void {
  const target = event.target
  if (root.value === null || !root.value.open) return
  if (target instanceof Node && root.value.contains(target)) return
  closeMenu()
}

function onDocumentClick(event: MouseEvent): void {
  const target = event.target
  if (root.value === null || !root.value.open) return
  if (!(target instanceof Element) || !root.value.contains(target)) return
  if (target instanceof HTMLInputElement && event.detail === 0) return
  if (target.closest('[data-menu-close]') !== null) closeMenu()
}

onMounted(() => {
  if (root.value !== null) mountedMenus.add(root.value)
  document.addEventListener('mousedown', onDocumentMouseDown)
  document.addEventListener('click', onDocumentClick)
})
onBeforeUnmount(() => {
  if (root.value !== null) mountedMenus.delete(root.value)
  document.removeEventListener('mousedown', onDocumentMouseDown)
  document.removeEventListener('click', onDocumentClick)
})
</script>

<template>
  <details
    ref="root"
    class="menu"
    :class="['menu--' + variant, { 'menu--active': active === true, 'menu--disabled': disabled === true }]"
    data-dropdown-menu
    @toggle="onToggle"
    @keydown="onKeydown"
  >
    <summary
      class="menu-trigger"
      :aria-label="ariaLabel"
      :aria-disabled="disabled === true ? 'true' : undefined"
      :tabindex="disabled === true ? -1 : undefined"
    >
      <!-- aura-allow-text: label is the output of t() or a user-typed filter value. -->
      <span class="menu-label">{{ label }}</span>
      <!-- aura-allow-text: decorative glyph, hidden from assistive tech. -->
      <span v-if="variant !== 'more'" class="menu-caret" aria-hidden="true">▾</span>
    </summary>
    <div class="menu-panel">
      <slot />
    </div>
  </details>
</template>

<style scoped>
.menu {
  position: relative;
}

.menu-trigger {
  list-style: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.menu-trigger::-webkit-details-marker {
  display: none;
}

.menu--chip .menu-trigger {
  padding: 4px 12px;
  border: 1px solid var(--color-outline);
  border-radius: var(--radius-full);
  background: var(--color-surface);
}

.menu--chip.menu--active .menu-trigger {
  border-color: var(--color-primary);
  color: var(--color-primary);
  font-weight: var(--weight-ui-md-strong);
}

.menu--chip:not(.menu--active) .menu-trigger:focus-visible {
  outline: none;
  border-color: var(--color-primary);
}

.menu--field .menu-trigger {
  justify-content: space-between;
  padding: 8px 8px;
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  background: var(--color-surface);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
  color: var(--color-on-surface);
}

.menu--field .menu-trigger:focus-visible {
  outline: none;
  border-color: var(--color-primary);
}

.menu--field .menu-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.menu--disabled .menu-trigger {
  pointer-events: none;
  cursor: not-allowed;
  color: var(--color-on-surface-variant);
}

.menu--more .menu-trigger {
  padding: 4px 8px;
  border-radius: 3px;
  background: var(--color-surface);
}

.menu--field .menu-panel {
  right: 0;
  left: 0;
  max-height: 240px;
  overflow-y: auto;
}

.menu--field.menu--columns .menu-panel {
  max-height: min(480px, 60vh);
}

.menu-panel {
  position: absolute;
  top: 100%;
  right: 0;
  min-width: 180px;
  margin-top: 4px;
  padding: 4px 0;
  display: flex;
  flex-direction: column;
  background: var(--color-surface);
  border: 1px solid var(--color-outline);
  border-radius: 4px;
  /* aura-allow-z-index: the panel must float above the grid cards below it. */
  z-index: 2;
}
</style>
