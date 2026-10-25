<script setup lang="ts">
import { onBeforeUnmount, onMounted, useTemplateRef } from 'vue'

defineProps<{
  variant: 'chip' | 'more'
  label: string
  active?: boolean
  ariaLabel?: string
}>()

const root = useTemplateRef<HTMLDetailsElement>('root')

function closeMenu(): void {
  if (root.value !== null) root.value.open = false
}

function onToggle(): void {
  if (root.value === null || !root.value.open) return
  for (const other of document.querySelectorAll<HTMLDetailsElement>('details[data-library-menu][open]')) {
    if (other !== root.value) other.open = false
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape' || root.value === null || !root.value.open) return
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
  if (target instanceof Element && root.value.contains(target) && target.closest('[data-menu-close]') !== null) closeMenu()
}

onMounted(() => {
  document.addEventListener('mousedown', onDocumentMouseDown)
  document.addEventListener('click', onDocumentClick)
})
onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocumentMouseDown)
  document.removeEventListener('click', onDocumentClick)
})
</script>

<template>
  <details
    ref="root"
    class="menu"
    :class="['menu--' + variant, { 'menu--active': active === true }]"
    data-library-menu
    @toggle="onToggle"
    @keydown="onKeydown"
  >
    <summary class="menu-trigger" :aria-label="ariaLabel">
      <!-- aura-allow-text: label is the output of t() or a user-typed filter value. -->
      <span>{{ label }}</span>
      <!-- aura-allow-text: decorative glyph, hidden from assistive tech. -->
      <span v-if="variant === 'chip'" class="menu-caret" aria-hidden="true">▾</span>
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

.menu--more .menu-trigger {
  padding: 4px 8px;
  border-radius: 3px;
  background: var(--color-surface);
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
