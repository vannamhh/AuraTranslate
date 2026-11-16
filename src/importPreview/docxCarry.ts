import type { ChapterPatternInput } from '../config/project'

/** What the docx shape switch carries to the other preview. */
export interface DocxCarry {
  name: string
  sourceLang: string
  genre: string
  destinationWorkId: string | null
  path: string
  chapterPattern: ChapterPatternInput | null
}
