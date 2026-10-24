import type { InjectionKey, Ref } from 'vue'
import type { IpcError } from '../i18n'
import type { ChapterAsset, ChapterSegment } from '../config/segment'
import type { SegmentRuleValue } from './editorSegments'
import type { SegmentGlossaryMarks, SegmentTermSpan } from './glossaryMarksMap'

export type SourcePieceInfo = {
  text: string
  start: number
  isPendingCut: boolean
  glossary: SegmentTermSpan | null
  concordance: boolean
}

export type GridBlockView = {
  readonly index: number
  readonly start: number
  readonly segments: readonly ChapterSegment[]
  readonly rules: ReadonlyMap<number, SegmentRuleValue>
}

export type GridBlockContext = {
  readonly selectedRowClassById: Readonly<Ref<ReadonlyMap<number, string>>>
  readonly unlinkedIds: Readonly<Ref<ReadonlySet<number>>>
  readonly rowErrorLabelById: Readonly<Ref<ReadonlyMap<number, IpcError>>>
  readonly rowErrorTextById: Readonly<Ref<ReadonlyMap<number, string>>>
  readonly firstSegmentId: Readonly<Ref<number | null>>
  readonly showHanViet: Readonly<Ref<boolean>>
  readonly sourceTokenClass: Readonly<Ref<string>>
  cutOffsetsOf(segmentId: number): readonly number[]
  cutCountOf(segmentId: number): number
  glossaryTermsOf(segmentId: number): SegmentGlossaryMarks
  sourcePieceInfoOf(segmentId: number, text: string): readonly SourcePieceInfo[]
  trailingImagesOf(id: number): ChapterAsset[]
  leadingImagesOf(id: number): ChapterAsset[]
  roleLabelKeys: Readonly<Record<string, string>>
  setStripTrackRef(el: unknown): void
  onSourceCellMouseUp(event: MouseEvent): void
  onGlossaryPieceEnter(span: SegmentTermSpan | null): void
  onGlossaryPieceLeave(): void
  onColumnFocusOut(event: FocusEvent): void
  onCellMouseDown(event: MouseEvent): void
  onCellMouseUp(event: MouseEvent): void
  onBeforeInput(event: InputEvent): void
  onCompositionStart(): void
  onCompositionEnd(): void
  onEditInput(event: Event): void
  onEditKeydown(event: KeyboardEvent): void
  onStripFocusOut(event: FocusEvent): void
}

export const GRID_BLOCK_CONTEXT: InjectionKey<GridBlockContext> = Symbol('grid-block-context')
