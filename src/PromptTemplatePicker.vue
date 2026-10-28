<script setup lang="ts">
import { computed, ref } from 'vue'
import { t, tError } from './i18n'
import { GENRE_IDS, genreLabel } from './modes/libraryWorkCard'
import type { GenreId } from './modes/libraryWorkCard'
import { PROMPT_TEMPLATES } from './promptTemplates'
import {
  addPromptTemplates,
  promptSetActionError,
  promptSetBusy,
  promptSetWorkTierAvailable,
  promptTemplateExists,
} from './promptSetState'
import type { PromptSetTier } from './config/promptset'

const tier = ref<PromptSetTier>('global')
const picked = ref<GenreId[]>([])
const previewId = ref<GenreId | null>(null)
const addedCount = ref<number | null>(null)

const rows = computed(() =>
  GENRE_IDS.map((id) => ({ id, label: genreLabel(id) ?? id, exists: promptTemplateExists(tier.value, id) })),
)
const missingIds = computed(() => rows.value.filter((r) => !r.exists).map((r) => r.id))
const pickedMissing = computed(() => picked.value.filter((id) => missingIds.value.includes(id)))
const previewBody = computed(() => (previewId.value === null ? null : PROMPT_TEMPLATES[previewId.value]))

function onTierChange(next: PromptSetTier): void {
  tier.value = next
  picked.value = picked.value.filter((id) => !promptTemplateExists(next, id))
  addedCount.value = null
}

function onToggle(id: GenreId, event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return
  picked.value = target.checked ? [...picked.value.filter((p) => p !== id), id] : picked.value.filter((p) => p !== id)
  addedCount.value = null
}

function onPreview(id: GenreId): void {
  previewId.value = id
}

function onPickAllMissing(): void {
  picked.value = [...missingIds.value]
  addedCount.value = null
}

async function onSubmitAdd(): Promise<void> {
  const ids = pickedMissing.value
  if (ids.length === 0) return
  addedCount.value = await addPromptTemplates(tier.value, ids)
  picked.value = []
}

const emit = defineEmits<{ close: [] }>()

function onCloseTemplates(): void {
  emit('close')
}
</script>

<template>
  <div class="pt-root" data-prompt-template-picker>
    <h3 class="pt-h2">{{ t('prompt.library.template_title') }}</h3>
    <p class="pt-hint">{{ t('prompt.library.template_hint') }}</p>

    <fieldset v-if="promptSetWorkTierAvailable" class="pt-tier" role="radiogroup" :aria-label="t('prompt.library.tier_label')">
      <legend class="pt-label">{{ t('prompt.library.tier_label') }}</legend>
      <label class="pt-radio ui-check">
        <input type="radio" name="pt-tier" data-template-tier="global" :checked="tier === 'global'" @change="onTierChange('global')" />
        {{ t('prompt.library.tier_global') }}
      </label>
      <label class="pt-radio ui-check">
        <input type="radio" name="pt-tier" data-template-tier="work" :checked="tier === 'work'" @change="onTierChange('work')" />
        {{ t('prompt.library.tier_work') }}
      </label>
    </fieldset>

    <ul class="pt-list">
      <li v-for="row in rows" :key="row.id" class="pt-item" :data-template-id="row.id">
        <label class="pt-radio ui-check">
          <input
            type="checkbox"
            data-template-check
            :checked="picked.includes(row.id)"
            :disabled="row.exists || promptSetBusy"
            @change="onToggle(row.id, $event)"
          />
          <!-- aura-allow-text: DỮ LIỆU (nhãn thể loại từ vi.json). -->
          <span>{{ row.label }}</span>
          <span v-if="row.exists" class="pt-badge" data-template-exists>{{ t('prompt.library.template_exists_badge') }}</span>
        </label>
        <form class="pt-inline" @submit.prevent="onPreview(row.id)">
          <button type="submit" class="pt-act ui-btn ui-btn--sm" data-template-preview :aria-pressed="previewId === row.id">
            {{ t('prompt.library.template_preview_button') }}
          </button>
        </form>
      </li>
    </ul>

    <div v-if="previewBody !== null" class="pt-preview" data-template-preview-body>
      <!-- aura-allow-text: DỮ LIỆU (nhãn thể loại). -->
      <h4 class="pt-label">{{ t('prompt.library.template_preview_heading', { name: genreLabel(previewId ?? '') ?? '' }) }}</h4>
      <!-- aura-allow-text: nội dung mẫu, dữ liệu tĩnh. -->
      <pre class="pt-pre">{{ previewBody }}</pre>
    </div>

    <div class="pt-actions">
      <form class="pt-inline" @submit.prevent="onPickAllMissing">
        <button type="submit" class="pt-act ui-btn ui-btn--lg" data-template-pick-missing :disabled="missingIds.length === 0 || promptSetBusy">
          {{ t('prompt.library.template_pick_missing') }}
        </button>
      </form>
      <form class="pt-inline" @submit.prevent="onSubmitAdd">
        <button type="submit" class="pt-act pt-act-primary ui-btn ui-btn--lg ui-btn--primary ui-btn--pair" data-template-add :disabled="pickedMissing.length === 0 || promptSetBusy">
          {{ t('prompt.library.template_add_button', { count: String(pickedMissing.length) }) }}
        </button>
      </form>
      <form class="pt-inline" @submit.prevent="onCloseTemplates">
        <button type="submit" class="pt-act ui-btn ui-btn--lg ui-btn--pair" data-template-close>{{ t('prompt.library.template_close') }}</button>
      </form>
    </div>
    <p v-if="missingIds.length === 0" class="pt-status" role="status" data-template-all-present>
      {{ t('prompt.library.template_all_present') }}
    </p>
    <p v-if="addedCount !== null && promptSetActionError === null" class="pt-status" role="status" data-template-added>
      {{ t('prompt.library.template_added', { count: String(addedCount) }) }}
    </p>
    <p v-if="promptSetActionError !== null" class="pt-status pt-alert" role="alert">
      <!-- aura-allow-text: KẾT QUẢ của `tError()`. -->
      {{ tError(promptSetActionError) }}
    </p>
  </div>
</template>

<style scoped>
.pt-root {
  display: flex;
  flex-direction: column;
  gap: calc(var(--space-unit) * 2);
  min-width: 0;
}

.pt-h2 {
  margin: 0;
  font-family: var(--face-read-title);
  font-size: var(--font-read-title);
  font-weight: var(--weight-read-title);
  color: var(--color-on-surface);
}

.pt-hint,
.pt-status {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.pt-alert {
  color: var(--color-error);
}

.pt-label {
  margin: 0;
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  text-transform: uppercase;
  letter-spacing: var(--tracking-ui-label);
  color: var(--color-on-surface-variant);
}

.pt-tier {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: calc(var(--space-unit) * 3);
  margin: 0;
  padding: 0;
  border: none;
}

.pt-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.pt-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: calc(var(--space-unit) * 2);
  padding: var(--space-unit) 0;
}

.pt-radio {
  display: flex;
  align-items: center;
  gap: calc(var(--space-unit) * 1);
  color: var(--color-on-surface);
  cursor: pointer;
}

.pt-badge {
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
  border: 1px solid var(--color-outline);
  padding: 0 calc(var(--space-unit) * 1);
}

.pt-inline {
  display: inline-flex;
}

.pt-actions {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 2);
}

.pt-pre {
  margin: 0;
  white-space: pre-wrap;
  font-family: var(--face-ui-mono);
  font-size: var(--font-ui-mono);
  line-height: var(--leading-ui-mono);
  color: var(--color-on-surface);
  border: 1px solid var(--color-outline);
  padding: calc(var(--space-unit) * 2);
}
</style>
