<script setup lang="ts">
export type AppIconName = 'pin' | 'pin-on' | 'chevron' | 'info' | 'layout-left' | 'layout-top'

defineProps<{ name: AppIconName }>()
</script>

<template>
  <svg
    v-if="name === 'layout-left' || name === 'layout-top'"
    class="app-icon app-icon--wide"
    viewBox="0 0 18 14"
    aria-hidden="true"
    focusable="false"
  >
    <rect x=".75" y=".75" width="16.5" height="12.5" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.3" />
    <path v-if="name === 'layout-left'" d="M10.5 1v12M10.5 7h6.5" fill="none" stroke="currentColor" stroke-width="1.3" />
    <path v-else d="M1 8h16M9 8v5" fill="none" stroke="currentColor" stroke-width="1.3" />
  </svg>
  <svg v-else class="app-icon" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
    <path
      v-if="name === 'pin'"
      d="M6 1.75h4M6.6 1.75l-.5 4.1-2.35 2.4v1.15h8.5V8.25L9.9 5.85l-.5-4.1M8 9.4v4.85"
      fill="none"
      stroke="currentColor"
      stroke-width="1.4"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <template v-else-if="name === 'pin-on'">
      <path d="M6.6 1.75h2.8l.5 4.1 2.35 2.4v1.15h-8.5V8.25L6.1 5.85z" fill="currentColor" />
      <path d="M8 9.4v4.85" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
    </template>
    <path
      v-else-if="name === 'chevron'"
      d="M4 6l4 4 4-4"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
    <template v-else>
      <circle cx="8" cy="8" r="6.25" fill="none" stroke="currentColor" stroke-width="1.3" />
      <path d="M8 7.2v4M8 4.9v.1" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
    </template>
  </svg>
</template>

<style scoped>
.app-icon {
  width: 14px;
  height: 14px;
  flex: none;
}

.app-icon--wide {
  width: 21px;
  height: 14px;
}
</style>
