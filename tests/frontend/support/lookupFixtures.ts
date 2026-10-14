import type {
  EntryHit,
  GroupedLookup,
  LookupResponse,
  SenseRecord,
  SourceAttribution,
  SourceGroup,
} from '../../../src/config/dict'

export function source(code: string, name: string, layer: string): SourceAttribution {
  return {
    code,
    display_name: name,
    license_kind: 'open',
    license_id: null,
    license_text_len: 0,
    attribution: name,
    source_version: '1',
    source_url: 'https://example.invalid',
    lang: 'zh',
    layer,
    is_base: true,
  }
}

export function hit(sourceCode: string, entryId: number, headword: string, reading: string | null): EntryHit {
  return { entry_id: entryId, source_code: sourceCode, lang: 'zh', headword, headword_simp: null, reading }
}

export function sense(entryId: number, senseId: number, gloss: string): SenseRecord {
  return {
    entry_id: entryId,
    sense_id: senseId,
    pos: null,
    pos_lang: null,
    pos_is_foreign: false,
    gloss,
    note: null,
    ord: senseId,
    examples: [],
    citations: [],
  }
}

export function group(
  layer: string,
  code: string,
  name: string,
  entries: EntryHit[],
  totalEntries: number | null = null,
): SourceGroup {
  return { layer, source: { code, display_name: name }, entries, total_entries: totalEntries }
}

export function response(
  groups: SourceGroup[],
  senses: Record<string, SenseRecord[]>,
  patch: Partial<GroupedLookup> = {},
): LookupResponse {
  return {
    grouped: {
      route: 'zh',
      branch: 'exact_btree',
      groups,
      skipped: [],
      truncated_layers: [],
      hidden_sources: [],
      layers_loaded: true,
      ...patch,
    },
    senses_by_layer: senses,
    query_truncated: false,
    senses_failed: [],
  }
}

export const SOURCES = [
  source('cvdict', 'CVDICT', 'layer-cv'),
  source('cc-cedict', 'CC-CEDICT', 'layer-cc'),
  source('vietphrase', 'Vietphrase', 'layer-vp'),
]

/** 打: CVDICT has two readings (1 + 2 senses), Vietphrase has one entry with no reading. */
export const DA = response(
  [
    group('layer-cv', 'cvdict', 'CVDICT', [hit('cvdict', 1, '打', 'da2'), hit('cvdict', 2, '打', 'da3')]),
    group('layer-vp', 'vietphrase', 'Vietphrase', [hit('vietphrase', 7, '打', null)]),
  ],
  {
    'layer-cv': [sense(1, 11, 'đánh nhau'), sense(2, 21, 'đánh'), sense(2, 22, 'gõ')],
    'layer-vp': [sense(7, 71, 'đánh (vp)'), sense(7, 72, 'làm (vp)')],
  },
)

/** 走廊: two sources, one entry each, the same reading in two spellings. */
export const ZOULANG = response(
  [
    group('layer-cv', 'cvdict', 'CVDICT', [hit('cvdict', 5, '走廊', 'zou3 lang2')]),
    group('layer-cc', 'cc-cedict', 'CC-CEDICT', [hit('cc-cedict', 9, '走廊', 'zǒuláng')]),
  ],
  { 'layer-cv': [sense(5, 51, 'hành lang')], 'layer-cc': [sense(9, 91, 'corridor')] },
)

export const NOT_FOUND = response([], {})
