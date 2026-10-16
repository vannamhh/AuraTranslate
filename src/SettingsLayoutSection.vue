<script setup lang="ts">
// Layout section of the Settings frame. Presets are the user's: the two built-in
// layouts are starting points, named presets are saved, renamed, applied and deleted here.
// `LayoutPreset` is `GlobalOnly`, so there is no scope switch. Applying needs a live Workspace dock.
import { ref } from 'vue'
import { t } from './i18n'
import { applyLayoutJson, applyPreset, currentLayoutJson, hasDockController, isDockSuspended } from './layout/dockController'
import {
  activeLayoutPreset,
  applyUserLayoutPreset,
  deleteUserLayoutPreset,
  layoutPresetErrorKey,
  renameUserLayoutPreset,
  saveUserLayoutPreset,
  setLayoutPresetError,
  userLayoutPresets,
} from './layout/layoutPresetState'
import { LAYOUT_PRESETS } from './layout/workspaceLayout'
import { shortcutDisplayFor } from './config/shortcutsState'

const reviewIsOpen = isDockSuspended()
const dockIsLive = hasDockController() && !reviewIsOpen
const newName = ref('')
const renameDraft = ref<Record<string, string>>({})

function onApplyBuiltin(presetId: string): void {
  setLayoutPresetError(applyPreset(presetId) ? null : 'settings.layout.error_no_workspace')
}

function onApplyUser(name: string): void {
  applyUserLayoutPreset(name, applyLayoutJson)
}

function onSavePreset(): void {
  void saveUserLayoutPreset(newName.value, currentLayoutJson()).then((saved) => {
    if (saved) newName.value = ''
  })
}

function onRenamePreset(name: string): void {
  void renameUserLayoutPreset(name, renameDraft.value[name] ?? '').then((renamed) => {
    if (!renamed) return
    renameDraft.value = Object.fromEntries(Object.entries(renameDraft.value).filter(([key]) => key !== name))
  })
}

function onDeletePreset(name: string): void {
  void deleteUserLayoutPreset(name)
}

function onNewNameInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) newName.value = target.value
}

function onRenameInput(name: string, event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) renameDraft.value = { ...renameDraft.value, [name]: target.value }
}
</script>

<template>
  <div class="sl-body" data-settings-layout>
    <p v-if="reviewIsOpen" class="sl-note" role="status" data-layout-review-open>
      {{ t('settings.layout.review_open') }}
    </p>
    <p v-else-if="!dockIsLive" class="sl-note" role="status" data-layout-no-workspace>
      {{ t('settings.layout.no_workspace') }}
    </p>

    <ul class="sl-list" data-layout-builtin-list>
      <li v-for="preset in LAYOUT_PRESETS" :key="preset.id" class="sl-item" :data-layout-builtin="preset.id">
        <span class="sl-name">{{ t(preset.labelKey) }}</span>
        <span v-if="activeLayoutPreset === preset.id" class="sl-tag">{{ t('settings.layout.in_use') }}</span>
        <!-- aura-allow-text: hợp âm đã định dạng — DỮ LIỆU. -->
        <span class="sl-chord">{{ shortcutDisplayFor(preset.id) }}</span>
        <form class="sl-form" @submit.prevent="onApplyBuiltin(preset.id)">
          <button type="submit" class="sl-btn" :disabled="!dockIsLive">{{ t('settings.layout.apply') }}</button>
        </form>
      </li>
    </ul>

    <h4 class="sl-h">{{ t('settings.layout.user_heading') }}</h4>
    <p v-if="userLayoutPresets.length === 0" class="sl-note" data-layout-user-empty>
      {{ t('settings.layout.user_empty') }}
    </p>
    <ul v-else class="sl-list" data-layout-user-list>
      <li v-for="preset in userLayoutPresets" :key="preset.name" class="sl-item" :data-layout-user="preset.name">
        <!-- aura-allow-text: tên preset do người dùng đặt — DỮ LIỆU. -->
        <span class="sl-name">{{ preset.name }}</span>
        <form class="sl-form" @submit.prevent="onApplyUser(preset.name)">
          <button type="submit" class="sl-btn" :disabled="!dockIsLive" data-layout-apply>
            {{ t('settings.layout.apply') }}
          </button>
        </form>
        <form class="sl-form" @submit.prevent="onRenamePreset(preset.name)">
          <input
            class="sl-input"
            autocomplete="off"
            data-layout-rename-input
            :aria-label="t('settings.layout.rename_label')"
            :placeholder="t('settings.layout.rename_label')"
            :value="renameDraft[preset.name] ?? ''"
            @input="onRenameInput(preset.name, $event)"
          />
          <button type="submit" class="sl-btn" data-layout-rename>{{ t('settings.layout.rename') }}</button>
        </form>
        <form class="sl-form" @submit.prevent="onDeletePreset(preset.name)">
          <button type="submit" class="sl-btn" data-layout-delete>{{ t('settings.layout.delete') }}</button>
        </form>
      </li>
    </ul>

    <form class="sl-save" @submit.prevent="onSavePreset()">
      <input
        class="sl-input"
        autocomplete="off"
        data-layout-new-name
        :aria-label="t('settings.layout.new_name_label')"
        :placeholder="t('settings.layout.new_name_label')"
        :value="newName"
        @input="onNewNameInput($event)"
      />
      <button type="submit" class="sl-btn" :disabled="!dockIsLive" data-layout-save>
        {{ t('settings.layout.save_current') }}
      </button>
    </form>
    <p v-if="layoutPresetErrorKey !== null" class="sl-alert" role="alert" data-layout-error>
      {{ t(layoutPresetErrorKey) }}
    </p>
    <p class="sl-note">{{ t('settings.layout.autosave_note') }}</p>
  </div>
</template>

<style scoped>
.sl-list {
  list-style: none;
  margin: 0 0 var(--space-panel-block) 0;
  padding: 0;
}

.sl-item {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-panel-inline);
  padding: var(--space-panel-block) 0;
  border-bottom: 1px solid var(--color-outline);
}

.sl-name {
  min-width: 12em;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.sl-tag,
.sl-chord {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-primary);
}

.sl-chord {
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  color: var(--color-on-surface-variant);
}

.sl-h {
  margin: var(--space-panel-block) 0;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.sl-form,
.sl-save {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 2);
  margin: 0;
}

.sl-save {
  margin-top: var(--space-panel-block);
}

.sl-input {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border: 1px solid var(--color-outline);
  background: var(--color-background);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.sl-btn {
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 3);
  border: 1px solid var(--color-outline);
  background: none;
  cursor: pointer;
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  color: var(--color-on-surface);
}

.sl-note {
  margin: var(--space-panel-block) 0 0 0;
  padding-left: 12px;
  border-left: 2px solid var(--color-tm-rule);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-tm-text);
}

.sl-alert {
  margin: var(--space-panel-block) 0 0 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  color: var(--color-error);
}
</style>
