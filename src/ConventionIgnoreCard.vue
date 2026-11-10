<script setup lang="ts">
import { computed } from 'vue'
import { t, tError } from './i18n'
import { dispatch } from './commands'
import {
  conventionManageBusy,
  conventionManageError,
  conventionManageHasLoaded,
  conventionManageIgnoreClearPending,
  conventionManageIgnoreDone,
  conventionManageIgnoreRows,
  conventionManageIgnoreSelectedId,
  conventionManageWorkTierAvailable,
  selectConventionManageIgnore,
} from './conventionManageState'

const lines = computed(() =>
  conventionManageIgnoreRows.value.map((row) => ({
    id: row.id,
    text:
      row.context_before === '' && row.context_after === ''
        ? row.avoid_phrase
        : t('convention.manage.ignored_context', {
            context_before: row.context_before,
            phrase: row.avoid_phrase,
            context_after: row.context_after,
          }),
  })),
)

const count = computed(() => String(conventionManageIgnoreRows.value.length))

const clearText = computed(() =>
  conventionManageIgnoreClearPending.value
    ? t('convention.manage.ignored_clear_confirm', { ignored_count: count.value })
    : t('convention.manage.ignored_clear'),
)

const doneText = computed<string | null>(() => {
  const done = conventionManageIgnoreDone.value
  if (done === null) return null
  return done.kind === 'forgot'
    ? t('convention.manage.ignored_forget_done', { forgotten_count: String(done.count) })
    : t('convention.manage.ignored_clear_done', { ignored_count: String(done.count) })
})

const errorText = computed<string | null>(() =>
  conventionManageError.value === null ? null : tError(conventionManageError.value),
)

const needsWork = computed(() => conventionManageHasLoaded.value && !conventionManageWorkTierAvailable.value)
</script>

<template>
  <section class="cvi-card" data-convention-ignore-card>
    <p v-if="needsWork" class="cvi-note" role="status" data-convention-ignore-needs-work>
      {{ t('convention.manage.ignored_needs_work') }}
    </p>
    <p v-else-if="!conventionManageHasLoaded && conventionManageError === null" class="cvi-note" role="status">
      {{ t('convention.manage.loading') }}
    </p>
    <p v-else-if="conventionManageHasLoaded && lines.length === 0" class="cvi-note" role="status" data-convention-ignore-empty>
      {{ t('convention.manage.ignored_empty') }}
    </p>
    <ul v-else-if="lines.length > 0" class="cvi-list" :aria-label="t('convention.manage.ignored_list_label')">
      <li
        v-for="line in lines"
        :key="line.id"
        class="cvi-row"
        data-convention-ignore-row
        :aria-current="conventionManageIgnoreSelectedId === line.id"
        @pointerdown="selectConventionManageIgnore(line.id)"
        @focusin="selectConventionManageIgnore(line.id)"
      >
        <!-- aura-allow-text: text from the user's own document, shown as text only. -->
        <span class="cvi-text">{{ line.text }}</span>
        <button
          type="button"
          class="cvi-act ui-btn ui-btn--quiet"
          data-convention-ignore-forget
          :disabled="conventionManageBusy"
          @mousedown.prevent
          @click="dispatch('convention.manage.ignored_forget')"
        >
          {{ t('convention.manage.ignored_forget') }}
        </button>
      </li>
    </ul>

    <p v-if="conventionManageIgnoreClearPending" class="cvi-note" role="status">
      {{ t('convention.manage.ignored_clear_hint', { ignored_count: count }) }}
    </p>
    <p v-if="doneText !== null" class="cvi-note" role="status" data-convention-ignore-done>
      <!-- aura-allow-text: result of t() computed in the script. -->
      {{ doneText }}
    </p>
    <p v-if="errorText !== null" class="cvi-note cvi-error" role="alert">
      <!-- aura-allow-text: result of tError(). -->
      {{ errorText }}
    </p>

    <div v-if="conventionManageWorkTierAvailable" class="cvi-actions">
      <button
        type="button"
        class="cvi-act ui-btn"
        :class="{ 'ui-btn--danger': conventionManageIgnoreClearPending }"
        data-convention-ignore-clear
        :disabled="conventionManageBusy || lines.length === 0"
        @click="dispatch('convention.manage.ignored_clear')"
      >
        <!-- aura-allow-text: result of t() computed in the script. -->
        {{ clearText }}
      </button>
    </div>
  </section>
</template>

<style scoped>
.cvi-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-panel-block);
}

.cvi-note {
  margin: 0;
  font-family: var(--face-ui-sm);
  font-size: var(--font-ui-sm);
  line-height: var(--leading-ui-sm);
  color: var(--color-on-surface-variant);
}

.cvi-error {
  color: var(--color-error);
}

.cvi-list {
  display: flex;
  flex-direction: column;
  margin: 0;
  padding: 0;
  list-style: none;
}

.cvi-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-panel-inline);
  padding: calc(var(--space-unit) * 1) calc(var(--space-unit) * 2);
  border-bottom: 1px solid var(--color-outline);
  border-left: 2px solid var(--color-confirmed);
  font-family: var(--face-ui-md);
  font-size: var(--font-ui-md);
  line-height: var(--leading-ui-md);
}

.cvi-row[aria-current='true'] {
  background: var(--color-surface-accent);
}

.cvi-text {
  overflow-wrap: anywhere;
}

.cvi-act {
  flex: none;
}

.cvi-actions {
  display: flex;
  flex-wrap: wrap;
  gap: calc(var(--space-unit) * 1);
}
</style>
