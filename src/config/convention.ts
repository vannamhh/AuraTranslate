import { Channel, invoke } from '@tauri-apps/api/core'
import type { IpcError } from '../i18n'
import { hasIpcBridge, isIpcError, UNKNOWN_IPC_ERROR } from './aitranslate'
import { noteChapterStarted } from './lifecycle'
import type { PromoteAiTranslationOutcome } from './segment'

export type ConventionTier = 'global' | 'work'

/** `start`/`end` are UTF-16 offsets into the owning `scanned_text`, end exclusive. */
export type ConventionSpanWire = {
  start: number
  end: number
  rule_key: string
  avoid: string
  replacement_cased: string | null
  tier: ConventionTier
}

export type ConventionSegmentWire = {
  segment_id: number
  scanned_text: string
  spans: ConventionSpanWire[]
}

export type ConventionScanWire = {
  generation: number
  chapter_id: number
  segments: ConventionSegmentWire[]
  ignored: number
  glossary_suppressed: number
  overlapped: number
  not_scanned: number
}

export type ConventionSuggestionScanWire = {
  generation: number
  chapter_id: number | null
  items: ConventionSegmentWire[]
  ignored: number
  glossary_suppressed: number
  overlapped: number
}

export type ConventionSuggestionItem = { segment_id: number; text: string }

export type ConventionRuleStatus = 'in_use' | 'shadowed' | 'disabled_in_work'

export type ConventionRuleWire = {
  id: number
  rule_key: string
  avoid_phrase: string
  replacement_phrase: string | null
  enabled: boolean
  tier: ConventionTier
  status: ConventionRuleStatus
}

/** `avoid_phrase` is the matched text, not the rule's phrase; empty context means a segment edge. */
export type ConventionIgnoreRowWire = {
  id: number
  rule_key: string
  avoid_phrase: string
  context_before: string
  context_after: string
  created_at: string
}

export type ConventionTierFilter = ConventionTier | 'both'

export const CONVENTION_WORK_TIER_UNAVAILABLE = 'convention.work_tier_unavailable'

export type ConventionWriteOutcome = {
  segment_id: number
  target_text: string
  translation_origin: PromoteAiTranslationOutcome['translation_origin']
  status: 'draft' | 'confirmed'
  chapter_started: boolean
  start: number
  end: number
  phrase: string
  version_created: boolean
}

export type ConventionRangeWrite = {
  segmentId: number
  expectedText: string
  start: number
  end: number
  expectedPhrase: string
}

const CMD_LIST = 'convention_list_rules'
const CMD_ADD = 'convention_add_rule'
const CMD_EDIT = 'convention_edit_rule'
const CMD_DELETE = 'convention_delete_rule'
const CMD_SET_ENABLED = 'convention_set_enabled'
const CMD_SCAN = 'convention_scan_chapter'
const CMD_SCAN_SEGMENTS = 'convention_scan_segments'
const CMD_SCAN_SUGGESTIONS = 'convention_scan_suggestions'
const CMD_ACCEPT = 'convention_accept'
const CMD_REVERT = 'convention_revert'
const CMD_SCAN_PROGRESS = 'convention_scan_chapter_progress'
const CMD_REPLACE_CHAPTER = 'convention_replace_chapter'
const CMD_IGNORE_ADD = 'convention_ignore_add'
const CMD_IGNORE_LIST = 'convention_ignore_list'
const CMD_IGNORE_REMOVE = 'convention_ignore_remove'
const CMD_IGNORE_CLEAR = 'convention_ignore_clear'
const CMD_EXPORT_TIER = 'convention_export_tier'
const CMD_OPEN_IMPORT_PREVIEW = 'convention_open_import_preview'
const CMD_CONFIRM_IMPORT = 'convention_confirm_import'
const CMD_CANCEL_IMPORT = 'convention_cancel_import'

type Label =
  | 'scan_chapter'
  | 'scan_chapter_progress'
  | 'scan_segments'
  | 'scan_suggestions'
  | 'list_rules'
  | 'add_rule'
  | 'edit_rule'
  | 'delete_rule'
  | 'set_enabled'
  | 'accept' | 'revert' | 'replace_chapter'
  | 'ignore_add' | 'ignore_list' | 'ignore_remove' | 'ignore_clear'
  | 'export_tier' | 'open_import_preview' | 'confirm_import' | 'cancel_import'

function isCount(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value) && value >= 0
}

function isSpanWire(value: unknown): value is ConventionSpanWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionSpanWire, unknown>>
  return (
    isCount(v.start) &&
    isCount(v.end) &&
    typeof v.rule_key === 'string' &&
    typeof v.avoid === 'string' &&
    (typeof v.replacement_cased === 'string' || v.replacement_cased === null) &&
    (v.tier === 'global' || v.tier === 'work')
  )
}

function isSegmentWire(value: unknown): value is ConventionSegmentWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionSegmentWire, unknown>>
  return (
    typeof v.segment_id === 'number' &&
    typeof v.scanned_text === 'string' &&
    Array.isArray(v.spans) &&
    v.spans.every(isSpanWire)
  )
}

export function isConventionScanWire(value: unknown): value is ConventionScanWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionScanWire, unknown>>
  return (
    isCount(v.generation) &&
    typeof v.chapter_id === 'number' &&
    Array.isArray(v.segments) &&
    v.segments.every(isSegmentWire) &&
    isCount(v.ignored) &&
    isCount(v.glossary_suppressed) &&
    isCount(v.overlapped) &&
    isCount(v.not_scanned)
  )
}

export function isConventionSuggestionScanWire(value: unknown): value is ConventionSuggestionScanWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionSuggestionScanWire, unknown>>
  return (
    isCount(v.generation) &&
    (typeof v.chapter_id === 'number' || v.chapter_id === null) &&
    Array.isArray(v.items) &&
    v.items.every(isSegmentWire) &&
    isCount(v.ignored) &&
    isCount(v.glossary_suppressed) &&
    isCount(v.overlapped)
  )
}

function isRuleWire(value: unknown): value is ConventionRuleWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionRuleWire, unknown>>
  return (
    typeof v.id === 'number' &&
    typeof v.rule_key === 'string' &&
    typeof v.avoid_phrase === 'string' &&
    (typeof v.replacement_phrase === 'string' || v.replacement_phrase === null) &&
    typeof v.enabled === 'boolean' &&
    (v.tier === 'global' || v.tier === 'work') &&
    (v.status === 'in_use' || v.status === 'shadowed' || v.status === 'disabled_in_work')
  )
}

function isWriteOutcome(value: unknown): value is ConventionWriteOutcome {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionWriteOutcome, unknown>>
  return (
    typeof v.segment_id === 'number' &&
    typeof v.target_text === 'string' &&
    typeof v.translation_origin === 'string' &&
    (v.status === 'draft' || v.status === 'confirmed') &&
    typeof v.chapter_started === 'boolean' &&
    isCount(v.start) &&
    isCount(v.end) &&
    typeof v.phrase === 'string' &&
    typeof v.version_created === 'boolean'
  )
}

function failure(command: Label, err: unknown): IpcError | null {
  if (isIpcError(err)) return err
  if (hasIpcBridge()) {
    console.error(`[convention] \`convention_${command}\` failed with a non-IpcError: ${String(err)}`)
    return UNKNOWN_IPC_ERROR
  }
  console.info(`[convention] cannot call \`convention_${command}\` outside Tauri: ${String(err)}`)
  return null
}

function badShape(command: Label, what: string): IpcError {
  console.error(`[convention] \`convention_${command}\` returned a shape that is not ${what}`)
  return UNKNOWN_IPC_ERROR
}

/** Never throws. `error === null` with `scan === null` means there is no IPC bridge. */
export async function scanConventionChapter(
  generation: number,
): Promise<{ scan: ConventionScanWire | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_SCAN, { generation })
    if (!isConventionScanWire(wire)) return { scan: null, error: badShape('scan_chapter', 'ConventionScanWire') }
    return { scan: wire, error: null }
  } catch (err) {
    return { scan: null, error: failure('scan_chapter', err) }
  }
}

/** Rescans only `segmentIds`, read from disk by Rust. Never throws; `error === null` with `scan === null` means no IPC bridge. */
export async function scanConventionSegments(
  generation: number,
  segmentIds: readonly number[],
): Promise<{ scan: ConventionScanWire | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_SCAN_SEGMENTS, { generation, segmentIds: [...segmentIds] })
    if (!isConventionScanWire(wire)) return { scan: null, error: badShape('scan_segments', 'ConventionScanWire') }
    return { scan: wire, error: null }
  } catch (err) {
    return { scan: null, error: failure('scan_segments', err) }
  }
}

/** Read-only scan of AI suggestion texts in one call. Never throws; no write command ever takes these spans. */
export async function scanConventionSuggestions(
  generation: number,
  items: readonly ConventionSuggestionItem[],
): Promise<{ scan: ConventionSuggestionScanWire | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_SCAN_SUGGESTIONS, { generation, items: [...items] })
    if (!isConventionSuggestionScanWire(wire)) {
      return { scan: null, error: badShape('scan_suggestions', 'ConventionSuggestionScanWire') }
    }
    return { scan: wire, error: null }
  } catch (err) {
    return { scan: null, error: failure('scan_suggestions', err) }
  }
}

export async function listConventionRules(
  tier: ConventionTierFilter,
): Promise<{ rules: ConventionRuleWire[] | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_LIST, { tier })
    if (!Array.isArray(wire) || !wire.every(isRuleWire)) {
      return { rules: null, error: badShape('list_rules', 'ConventionRuleWire[]') }
    }
    return { rules: wire, error: null }
  } catch (err) {
    return { rules: null, error: failure('list_rules', err) }
  }
}

export type ConventionWriteAck = { ok: boolean; error: IpcError | null }

/** A blank replacement is stored as "không dùng". */
export async function addConventionRule(
  tier: ConventionTier,
  avoidPhrase: string,
  replacementPhrase: string | null,
): Promise<ConventionWriteAck> {
  try {
    const id = await invoke<unknown>(CMD_ADD, { tier, avoidPhrase, replacementPhrase })
    if (!isCount(id)) return { ok: false, error: badShape('add_rule', 'a rule id') }
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: failure('add_rule', err) }
  }
}

export async function editConventionRule(
  tier: ConventionTier,
  id: number,
  avoidPhrase: string,
  replacementPhrase: string | null,
): Promise<ConventionWriteAck> {
  try {
    await invoke<unknown>(CMD_EDIT, { tier, id, avoidPhrase, replacementPhrase })
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: failure('edit_rule', err) }
  }
}

export async function deleteConventionRule(tier: ConventionTier, id: number): Promise<ConventionWriteAck> {
  try {
    await invoke<unknown>(CMD_DELETE, { tier, id })
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: failure('delete_rule', err) }
  }
}

/** Switches the Global rule `ruleKey` off or back on for the open Work only. */
export async function setConventionRuleEnabled(ruleKey: string, enabled: boolean): Promise<ConventionWriteAck> {
  try {
    await invoke<unknown>(CMD_SET_ENABLED, { ruleKey, enabled })
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: failure('set_enabled', err) }
  }
}

export type ConventionWriteResult = { outcome: ConventionWriteOutcome | null; error: IpcError | null }

function checkedOutcome(command: Label, wire: unknown): ConventionWriteResult {
  if (!isWriteOutcome(wire)) return { outcome: null, error: badShape(command, 'a write outcome') }
  void noteChapterStarted(wire)
  return { outcome: wire, error: null }
}

/** Rust computes the replacement from the stored rule; the webview names only the rule key. Never throws. */
export async function acceptConventionSpan(
  args: ConventionRangeWrite & { ruleKey: string },
): Promise<ConventionWriteResult> {
  try {
    const wire = await invoke<unknown>(CMD_ACCEPT, {
      segmentId: args.segmentId,
      expectedText: args.expectedText,
      start: args.start,
      end: args.end,
      expectedPhrase: args.expectedPhrase,
      ruleKey: args.ruleKey,
    })
    return checkedOutcome('accept', wire)
  } catch (err) {
    return { outcome: null, error: failure('accept', err) }
  }
}

/** `expectedPhrase` is the phrase now in the text; `originalPhrase` is what it replaced. Never throws. */
export async function revertConventionSpan(
  args: ConventionRangeWrite & { originalPhrase: string },
): Promise<ConventionWriteResult> {
  try {
    const wire = await invoke<unknown>(CMD_REVERT, {
      segmentId: args.segmentId,
      expectedText: args.expectedText,
      start: args.start,
      end: args.end,
      expectedPhrase: args.expectedPhrase,
      originalPhrase: args.originalPhrase,
    })
    return checkedOutcome('revert', wire)
  } catch (err) {
    return { outcome: null, error: failure('revert', err) }
  }
}

export type ConventionScanProgress = { done: number; total: number }

function isScanProgress(value: unknown): value is ConventionScanProgress {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionScanProgress, unknown>>
  return isCount(v.done) && isCount(v.total)
}

/** Same scan as `scanConventionChapter`; `onProgress` may fire zero times. Never throws. */
export async function scanConventionChapterWithProgress(
  generation: number,
  onProgress: (progress: ConventionScanProgress) => void,
): Promise<{ scan: ConventionScanWire | null; error: IpcError | null }> {
  const progress = new Channel<unknown>()
  progress.onmessage = (raw) => {
    if (isScanProgress(raw)) onProgress(raw)
    else console.error('[convention] `convention_scan_chapter_progress` sent a frame that is not a progress count')
  }
  try {
    const wire = await invoke<unknown>(CMD_SCAN_PROGRESS, { generation, progress })
    if (!isConventionScanWire(wire)) return { scan: null, error: badShape('scan_chapter_progress', 'ConventionScanWire') }
    return { scan: wire, error: null }
  } catch (err) {
    return { scan: null, error: failure('scan_chapter_progress', err) }
  }
}

/** Wire names are snake_case: the nested items of `groups` carry no serde rename. */
export type ConventionReplaceSpotArg = {
  start: number
  end: number
  expected_phrase: string
  rule_key: string
}

export type ConventionReplaceGroupArg = {
  segment_id: number
  expected_text: string
  spots: ConventionReplaceSpotArg[]
}

export type ConventionReplaceSentenceOutcome = {
  segment_id: number
  target_text: string
  translation_origin: PromoteAiTranslationOutcome['translation_origin']
  status: 'draft' | 'confirmed'
  version_created: boolean
}

export type ConventionReplaceOutcome = {
  outcomes: ConventionReplaceSentenceOutcome[]
  replaced: number
  sentences: number
  confirmed: number
  drafts: number
  chapter_started: boolean
}

function isSentenceOutcome(value: unknown): value is ConventionReplaceSentenceOutcome {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionReplaceSentenceOutcome, unknown>>
  return (
    typeof v.segment_id === 'number' &&
    typeof v.target_text === 'string' &&
    typeof v.translation_origin === 'string' &&
    (v.status === 'draft' || v.status === 'confirmed') &&
    typeof v.version_created === 'boolean'
  )
}

function isReplaceOutcome(value: unknown): value is ConventionReplaceOutcome {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionReplaceOutcome, unknown>>
  return (
    Array.isArray(v.outcomes) &&
    v.outcomes.every(isSentenceOutcome) &&
    isCount(v.replaced) &&
    isCount(v.sentences) &&
    isCount(v.confirmed) &&
    isCount(v.drafts) &&
    typeof v.chapter_started === 'boolean'
  )
}

/** One transaction for every group: all written or none. Never throws. */
export async function replaceConventionChapter(
  groups: ConventionReplaceGroupArg[],
): Promise<{ outcome: ConventionReplaceOutcome | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_REPLACE_CHAPTER, { groups })
    if (!isReplaceOutcome(wire)) return { outcome: null, error: badShape('replace_chapter', 'a replace outcome') }
    void noteChapterStarted(wire)
    return { outcome: wire, error: null }
  } catch (err) {
    return { outcome: null, error: failure('replace_chapter', err) }
  }
}

function isIgnoreRowWire(value: unknown): value is ConventionIgnoreRowWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionIgnoreRowWire, unknown>>
  return (
    typeof v.id === 'number' &&
    typeof v.rule_key === 'string' &&
    typeof v.avoid_phrase === 'string' &&
    typeof v.context_before === 'string' &&
    typeof v.context_after === 'string' &&
    typeof v.created_at === 'string'
  )
}

/** Remembers the matched place as "not a mistake" for the open Work. Never throws. */
export async function addConventionIgnore(
  args: ConventionRangeWrite & { ruleKey: string },
): Promise<ConventionWriteAck> {
  try {
    await invoke<unknown>(CMD_IGNORE_ADD, {
      segmentId: args.segmentId,
      expectedText: args.expectedText,
      start: args.start,
      end: args.end,
      expectedPhrase: args.expectedPhrase,
      ruleKey: args.ruleKey,
    })
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: failure('ignore_add', err) }
  }
}

/** Newest first. `rows` is null on failure. Never throws. */
export async function listConventionIgnores(): Promise<{ rows: ConventionIgnoreRowWire[] | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_IGNORE_LIST)
    if (!Array.isArray(wire) || !wire.every(isIgnoreRowWire)) {
      return { rows: null, error: badShape('ignore_list', 'ConventionIgnoreRowWire[]') }
    }
    return { rows: wire, error: null }
  } catch (err) {
    return { rows: null, error: failure('ignore_list', err) }
  }
}

/** `removed` is false when the row was already gone. Never throws. */
export async function removeConventionIgnore(id: number): Promise<{ removed: boolean | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_IGNORE_REMOVE, { id })
    if (typeof wire !== 'boolean') return { removed: null, error: badShape('ignore_remove', 'a boolean') }
    return { removed: wire, error: null }
  } catch (err) {
    return { removed: null, error: failure('ignore_remove', err) }
  }
}

/** Rust clears without asking; the two-click guard lives in the caller. Never throws. */
export async function clearConventionIgnores(): Promise<{ cleared: number | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_IGNORE_CLEAR)
    if (!isCount(wire)) return { cleared: null, error: badShape('ignore_clear', 'a row count') }
    return { cleared: wire, error: null }
  } catch (err) {
    return { cleared: null, error: failure('ignore_clear', err) }
  }
}

export type ConventionExportResult =
  | { outcome: 'done'; path: string }
  | { outcome: 'cancelled' }
  | { outcome: 'ipc_unavailable' }
  | { outcome: 'error'; error: IpcError }

/** Rust opens the save dialog and writes; `cancelled` is a closed dialog and stays silent. Never throws. */
export async function exportConventionTier(tier: ConventionTier): Promise<ConventionExportResult> {
  try {
    const path = await invoke<unknown>(CMD_EXPORT_TIER, { tier })
    if (path === null) return { outcome: 'cancelled' }
    if (typeof path !== 'string' || path === '') return { outcome: 'error', error: badShape('export_tier', 'a file path') }
    return { outcome: 'done', path }
  } catch (err) {
    const error = failure('export_tier', err)
    return error === null ? { outcome: 'ipc_unavailable' } : { outcome: 'error', error }
  }
}

export type ConventionImportDecision = 'keep_mine' | 'take_file'

export type ConventionImportConflictWire = {
  rule_key: string
  existing_avoid_phrase: string
  existing_replacement_phrase: string | null
  file_avoid_phrase: string
  file_replacement_phrase: string | null
}

export type ConventionImportPreviewWire = {
  file_name: string
  tier: ConventionTier
  row_count: number
  new_count: number
  identical_count: number
  conflicts: ConventionImportConflictWire[]
}

export type ConventionImportAppliedWire = { added: number; replaced: number; kept: number }

function isNullableText(value: unknown): value is string | null {
  return value === null || typeof value === 'string'
}

function isImportConflictWire(value: unknown): value is ConventionImportConflictWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionImportConflictWire, unknown>>
  return (
    typeof v.rule_key === 'string' &&
    typeof v.existing_avoid_phrase === 'string' &&
    isNullableText(v.existing_replacement_phrase) &&
    typeof v.file_avoid_phrase === 'string' &&
    isNullableText(v.file_replacement_phrase)
  )
}

function isImportPreviewWire(value: unknown): value is ConventionImportPreviewWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionImportPreviewWire, unknown>>
  return (
    typeof v.file_name === 'string' &&
    (v.tier === 'global' || v.tier === 'work') &&
    isCount(v.row_count) &&
    isCount(v.new_count) &&
    isCount(v.identical_count) &&
    Array.isArray(v.conflicts) &&
    v.conflicts.every(isImportConflictWire)
  )
}

export type ConventionImportPreviewResult =
  | { outcome: 'loaded'; preview: ConventionImportPreviewWire }
  | { outcome: 'cancelled' }
  | { outcome: 'ipc_unavailable' }
  | { outcome: 'error'; error: IpcError }

/** Rust opens the pick dialog, parses and keeps the plan; only counts and conflicts cross the wire. Never throws. */
export async function openConventionImportPreview(tier: ConventionTier): Promise<ConventionImportPreviewResult> {
  try {
    const wire = await invoke<unknown>(CMD_OPEN_IMPORT_PREVIEW, { tier })
    if (wire === null) return { outcome: 'cancelled' }
    if (!isImportPreviewWire(wire)) {
      return { outcome: 'error', error: badShape('open_import_preview', 'ConventionImportPreviewWire') }
    }
    return { outcome: 'loaded', preview: wire }
  } catch (err) {
    const error = failure('open_import_preview', err)
    return error === null ? { outcome: 'ipc_unavailable' } : { outcome: 'error', error }
  }
}

/** Only `take_file` entries go out; a missing key means keep mine. Never throws. */
export async function confirmConventionImport(
  decisions: Readonly<Record<string, ConventionImportDecision>>,
): Promise<{ applied: ConventionImportAppliedWire | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_CONFIRM_IMPORT, { decisions: { ...decisions } })
    const v = wire as Partial<Record<keyof ConventionImportAppliedWire, unknown>> | null
    if (typeof v !== 'object' || v === null || !isCount(v.added) || !isCount(v.replaced) || !isCount(v.kept)) {
      return { applied: null, error: badShape('confirm_import', 'an import outcome') }
    }
    return { applied: { added: v.added, replaced: v.replaced, kept: v.kept }, error: null }
  } catch (err) {
    return { applied: null, error: failure('confirm_import', err) ?? UNKNOWN_IPC_ERROR }
  }
}

/** Drops the pending plan in Rust. Never throws. */
export async function cancelConventionImport(): Promise<{ error: IpcError | null }> {
  try {
    await invoke<unknown>(CMD_CANCEL_IMPORT)
    return { error: null }
  } catch (err) {
    return { error: failure('cancel_import', err) }
  }
}
