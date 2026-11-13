import { beforeEach, describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { GENRE_IDS, genreLabel } from '../../src/modes/libraryWorkCard'
import { PROMPT_TEMPLATES } from '../../src/promptTemplates'
import { setGenreRows } from '../../src/genreState'
import { SEED_GENRES } from './support/genreSeed'
import { splitPromptBody } from '../../src/promptMarkers'

const known = (JSON.parse(readFileSync('tests/fixtures/prompt-variable-markers.json', 'utf8')) as { known: string[] }).known

describe('PROMPT_TEMPLATES', () => {
  beforeEach(() => {
    setGenreRows(SEED_GENRES)
  })

  it('có đúng 29 khoá, trùng GENRE_IDS', () => {
    expect(GENRE_IDS.length).toBe(29)
    expect(Object.keys(PROMPT_TEMPLATES).sort()).toEqual([...GENRE_IDS].sort())
  })

  it('mỗi thể loại có nhãn và thân không rỗng', () => {
    for (const id of GENRE_IDS) {
      expect(genreLabel(id), id).not.toBeNull()
      expect(PROMPT_TEMPLATES[id].trim().length, id).toBeGreaterThan(0)
    }
  })

  it('mỗi mẫu chứa đủ ba biến và không có dấu ngoặc biến lạ', () => {
    for (const id of GENRE_IDS) {
      const parts = splitPromptBody(PROMPT_TEMPLATES[id], known)
      expect(parts.map((p) => p.text).join(''), id).toBe(PROMPT_TEMPLATES[id])
      expect(parts.filter((p) => p.kind === 'unknown'), id).toEqual([])
      const used = new Set(parts.filter((p) => p.kind === 'known').map((p) => p.text))
      for (const name of known) expect(used.has(`{{${name}}}`), `${id} ${name}`).toBe(true)
    }
  })

  it('không mẫu nào lặp lệnh của system prompt', () => {
    for (const id of GENRE_IDS) {
      expect(PROMPT_TEMPLATES[id].toLowerCase(), id).not.toContain('chỉ trả về bản dịch')
    }
  })

  it('mỗi mẫu có đoạn riêng thể loại, khác nhau từng đôi', () => {
    expect(new Set(GENRE_IDS.map((id) => PROMPT_TEMPLATES[id])).size).toBe(29)
  })
})
