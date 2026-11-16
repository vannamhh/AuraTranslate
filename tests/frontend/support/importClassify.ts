import type { ClassifiedImportFile, ImportClassification } from '../../../src/config/library'

/** A test double for the Rust verdict (`classify_import_paths`); product code never calls this. */
export function fakeClassification(paths: string[]): ImportClassification {
  const ext = (path: string): string => path.split('.').pop()?.toLowerCase() ?? ''
  const kindOf = (path: string): ClassifiedImportFile['kind'] => {
    const e = ext(path)
    if (e === 'txt' || e === 'md') return 'text'
    if (e === 'csv' || e === 'tsv') return 'bilingual'
    if (e === 'docx') return paths.length === 1 ? 'docx' : 'text'
    return 'unsupported'
  }
  const kinds = paths.map(kindOf)
  const text = kinds.filter((k) => k === 'text').length
  const bilingual = kinds.filter((k) => k === 'bilingual').length
  const unsupported = kinds.filter((k) => k === 'unsupported').length
  const files: ClassifiedImportFile[] = paths.map((path, i) => {
    const kind = kinds[i]
    let problem: ClassifiedImportFile['problem'] = null
    if (kind === 'unsupported') problem = 'unsupported'
    else if (kind === 'bilingual' && text > 0) problem = 'kind_mismatch'
    else if (kind === 'bilingual' && bilingual > 1) problem = 'one_bilingual_only'
    const base = path.split(/[\\/]/).pop() ?? ''
    const dot = base.lastIndexOf('.')
    const stem = dot > 0 ? base.slice(0, dot) : base
    return { path, stem, kind, problem }
  })
  let verdict: ImportClassification['verdict'] = 'text'
  if (paths.length === 0) verdict = 'empty'
  else if (unsupported > 0) verdict = 'has_unsupported'
  else if (text > 0 && bilingual > 0) verdict = 'mixed_kinds'
  else if (bilingual > 1) verdict = 'multiple_bilingual'
  else if (bilingual === 1) verdict = 'bilingual'
  else if (kinds.filter((k) => k === 'docx').length === 1) verdict = 'docx'
  return { files, verdict }
}
