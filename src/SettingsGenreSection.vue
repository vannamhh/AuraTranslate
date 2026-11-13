<script setup lang="ts">
import { onMounted } from 'vue'
import { dispatch } from './commands'
import {
  genreError,
  genreIsBusy,
  genreListHasLoaded,
  genreNewName,
  genreRenameValue,
  genreSelected,
  genres,
  reloadGenresForSettings,
  resetGenreEditing,
  selectGenreRow,
  setGenreNewName,
  setGenreRenameValue,
} from './genreState'
import { t, tError } from './i18n'

onMounted(() => {
  resetGenreEditing()
  void reloadGenresForSettings()
})

function onNewNameInput(event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) setGenreNewName(target.value)
}

function onRenameInput(id: string, event: Event): void {
  const target = event.target
  if (target instanceof HTMLInputElement) setGenreRenameValue(id, target.value)
}
</script>

<template>
  <div class="sg-body" data-settings-genre>
    <form class="sg-add" @submit.prevent="dispatch('settings.genre.add')">
      <label class="sg-add-label">
        <span class="sg-label">{{ t('settings.genre.add_label') }}</span>
        <input
          type="text"
          class="sg-input ui-field ui-field--lg"
          data-genre-new-name
          autocomplete="off"
          :value="genreNewName"
          :disabled="genreIsBusy"
          @input="onNewNameInput($event)"
        />
      </label>
      <button type="submit" class="ui-btn ui-btn--primary ui-btn--lg" data-genre-add :disabled="genreIsBusy">
        {{ t('settings.genre.add') }}
      </button>
    </form>

    <p v-if="genreError !== null" class="sg-error" role="alert" data-genre-error>
      {{ tError(genreError) }}
    </p>

    <p v-if="!genreListHasLoaded" class="sg-note">{{ t('settings.genre.loading') }}</p>
    <p v-else-if="genres.length === 0" class="sg-note" data-genre-empty>{{ t('settings.genre.empty') }}</p>

    <ul v-else class="sg-list">
      <li
        v-for="row in genres"
        :key="row.id"
        class="sg-row"
        :class="{ 'sg-row-on': genreSelected === row.id }"
        :data-genre-row="row.id"
        @mousedown="selectGenreRow(row.id)"
        @focusin="selectGenreRow(row.id)"
      >
        <form class="sg-row-form" @submit.prevent="dispatch('settings.genre.rename')">
          <input
            type="text"
            class="sg-input ui-field ui-field--lg"
            data-genre-name
            autocomplete="off"
            :aria-label="t('settings.genre.name_aria')"
            :value="genreRenameValue(row.id)"
            :disabled="genreIsBusy"
            @input="onRenameInput(row.id, $event)"
          />
          <button type="submit" class="ui-btn ui-btn--lg" data-genre-rename :disabled="genreIsBusy">
            {{ t('settings.genre.rename') }}
          </button>
          <button
            type="button"
            class="ui-btn ui-btn--quiet ui-btn--lg"
            data-genre-delete
            :disabled="genreIsBusy"
            @click="dispatch('settings.genre.delete')"
          >
            {{ t('settings.genre.delete') }}
          </button>
        </form>
      </li>
    </ul>

    <p class="sg-note">{{ t('settings.genre.note') }}</p>
  </div>
</template>

<style scoped>
.sg-add {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: var(--space-unit);
  padding-bottom: var(--space-panel-block);
}

.sg-add-label {
  display: flex;
  flex-direction: column;
  gap: var(--space-unit);
}

.sg-label {
  font-family: var(--face-ui-label);
  font-size: var(--font-ui-label);
  font-weight: var(--weight-ui-label);
  letter-spacing: var(--tracking-ui-label);
  text-transform: uppercase;
  color: var(--color-on-surface-variant);
}

.sg-input {
  min-width: 280px;
}

.sg-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.sg-row {
  padding: var(--space-unit) 0;
  border-bottom: 1px solid var(--color-outline);
}

.sg-row-form {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-unit);
}

.sg-error {
  margin: 0 0 var(--space-panel-block) 0;
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-error);
}

.sg-note {
  margin: var(--space-panel-block) 0 0 0;
  padding-left: 12px; /* aura-allow-spacing: 2px left border sits inside the 14px inset */
  border-left: 2px solid var(--color-tm-rule);
  font-family: var(--face-ui-md-wrap);
  font-size: var(--font-ui-md-wrap);
  line-height: var(--leading-ui-md-wrap);
  color: var(--color-tm-text);
}
</style>
