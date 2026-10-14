import { computed } from 'vue'
import { t } from './i18n'
import { groupThousands, formatByteSize } from './importFormat'
import {
  bilingualImportPreview,
  bilingualImportPreviewIsOpen,
  bilingualImportPreviewSourceFileName,
} from './bilingualImportPreviewState'
import {
  importPreviewIsOpen,
  importPreviewLastSubmittedFrom,
  importPreviewSourceFileBytes,
  importPreviewSourceFileNames,
  importPreviewSourceTextBytes,
  importPreviewUrlItems,
} from './importPreviewState'
import { pastedUrlCount, pastedUrlDomainCount } from './modes/libraryImport'
import { libraryUrlImportScreenIsOpen } from './modes/libraryImportOverlayState'
import { currentMode } from './modes/modeState'

function joinParts(parts: string[]): string {
  return parts.join(' · ')
}

/** The right-hand title-bar cell while an import screen is open: what is being imported. */
export const importSourceStatusText = computed<string | null>(() => {
  if (currentMode.value !== 'library') return null
  if (bilingualImportPreviewIsOpen.value) {
    const parts = [bilingualImportPreviewSourceFileName.value]
    const rows = bilingualImportPreview.value?.row_count
    if (rows !== undefined) parts.push(t('mode.library.preview.bilingual_row_count', { count: groupThousands(rows) }))
    return joinParts(parts)
  }
  if (importPreviewIsOpen.value) {
    const from = importPreviewLastSubmittedFrom.value
    if (from === 'urls') {
      return t('titlebar.import_source_links', { count: groupThousands(importPreviewUrlItems.value.length) })
    }
    if (from === 'file') {
      const names = importPreviewSourceFileNames.value
      if (names.length === 0) return null
      const extra = names.length > 1 ? t('titlebar.import_source_more_files', { count: String(names.length - 1) }) : ''
      const bytes = importPreviewSourceFileBytes.value
      return joinParts(bytes === null ? [names[0] + extra] : [names[0] + extra, formatByteSize(bytes)])
    }
    const bytes = importPreviewSourceTextBytes.value
    const label = t('titlebar.import_source_pasted_text')
    return bytes === null ? label : joinParts([label, formatByteSize(bytes)])
  }
  if (libraryUrlImportScreenIsOpen.value) {
    return joinParts([
      t('titlebar.import_source_links', { count: groupThousands(pastedUrlCount.value) }),
      t('titlebar.import_source_domains', { count: String(pastedUrlDomainCount.value) }),
    ])
  }
  return null
})
