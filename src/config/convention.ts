import { invoke } from '@tauri-apps/api/core'
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

export type ConventionRuleWire = {
  id: number
  rule_key: string
  avoid_phrase: string
  replacement_phrase: string | null
  enabled: boolean
  tier: ConventionTier
}

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
const CMD_SCAN = 'convention_scan_chapter'
const CMD_ACCEPT = 'convention_accept'
const CMD_REVERT = 'convention_revert'

type Label = 'scan_chapter' | 'list_rules' | 'add_rule' | 'accept' | 'revert'

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

function isRuleWire(value: unknown): value is ConventionRuleWire {
  if (typeof value !== 'object' || value === null) return false
  const v = value as Partial<Record<keyof ConventionRuleWire, unknown>>
  return (
    typeof v.id === 'number' &&
    typeof v.rule_key === 'string' &&
    typeof v.avoid_phrase === 'string' &&
    (typeof v.replacement_phrase === 'string' || v.replacement_phrase === null) &&
    typeof v.enabled === 'boolean' &&
    (v.tier === 'global' || v.tier === 'work')
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

export async function listConventionRules(): Promise<{ rules: ConventionRuleWire[] | null; error: IpcError | null }> {
  try {
    const wire = await invoke<unknown>(CMD_LIST)
    if (!Array.isArray(wire) || !wire.every(isRuleWire)) {
      return { rules: null, error: badShape('list_rules', 'ConventionRuleWire[]') }
    }
    return { rules: wire, error: null }
  } catch (err) {
    return { rules: null, error: failure('list_rules', err) }
  }
}

/** A blank replacement is stored as "không dùng". */
export async function addConventionRule(
  avoidPhrase: string,
  replacementPhrase: string | null,
): Promise<{ ok: boolean; error: IpcError | null }> {
  try {
    const id = await invoke<unknown>(CMD_ADD, { avoidPhrase, replacementPhrase })
    if (!isCount(id)) return { ok: false, error: badShape('add_rule', 'a rule id') }
    return { ok: true, error: null }
  } catch (err) {
    return { ok: false, error: failure('add_rule', err) }
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
