/**
 * Fixture + bộ ghi lượt gọi cho cây test frontend. Story 2.3.
 *
 * ─────────────────────────────────────────────────────────────────────────────
 * 🔴 VÌ SAO GIẢ Ở **BIÊN IPC**, CHỨ KHÔNG THÊM MỘT HÀM CHỈ TEST GỌI
 * ─────────────────────────────────────────────────────────────────────────────
 * Đường rẻ hơn là xuất khẩu một `__setLastSavedAtForTests()` từ `editorPanelState.ts`. Kho này
 * đã **từ chối đúng hình dạng đó hai lần** — `src-tauri/tests/store_contract.rs` ghi lại:
 * *"thêm một hàm `pub` vào mã sản phẩm mà **chỉ test gọi** — mã không ai dùng, đúng thứ Story
 * 1.5 và 1.6 đã từ chối hai lần"*.
 *
 * Và nó không chỉ là kỷ luật: một setter chỉ-test **đi vòng qua** chính đường cần nghiệm thu.
 * `editorLastSavedAt` chỉ được phép đi lên khi một lô đã chạm WAL; một setter cho phép đặt nó
 * **mà không** có lượt ghi nào, tức ca test sẽ xanh kể cả khi đường flush đã chết.
 *
 * ⇒ Giả ở **biên** (`src/config/segment.ts` — nơi duy nhất gọi `invoke`), rồi lái **đúng đường
 * sản phẩm**: `ensureSegmentsLoaded` → `noteEditorEdit` → `flushEditorNow`.
 *
 * ⚠️ Vì sao phải giả gì cả: trong `happy-dom` không có `__TAURI_INTERNALS__`, nên mọi lượt gọi
 * `invoke` rơi vào nhánh *"chạy ngoài Tauri"* và trả `{ outcome: null, error: null }` — đúng
 * hợp đồng, nhưng nó làm mọi lượt flush thành một no-op và không mệnh đề nào kiểm được.
 *
 * ⚠️ `vi.mock` **không** sống ở tệp này: nó được hoist theo **tệp test**, và đường dẫn module
 * trong lời gọi được phân giải tương đối với tệp gọi. Mỗi tệp test tự khai `vi.mock`, và dùng
 * bộ ghi dưới đây làm chỗ chung.
 */
import type { ChapterSegment } from '../../../src/config/segment'

/** Cùng bộ câu với bàn đo `2-3-ban-do-vung-go.html` — một fixture, hai chỗ dùng. */
export const FIXTURE_SEGMENTS: readonly ChapterSegment[] = [
  {
    id: 11,
    ord: 1,
    source_text: '一。',
    target_text: 'Hắn đẩy cánh cửa ấy ra.',
    is_paragraph_end: false,
    retired_at: null,
  },
  {
    id: 12,
    ord: 2,
    source_text: '二。',
    target_text: 'Gió thổi tới từ cuối hành lang.',
    is_paragraph_end: true,
    retired_at: null,
  },
  {
    id: 13,
    ord: 3,
    source_text: '三。',
    target_text: '',
    is_paragraph_end: false,
    retired_at: null,
  },
]

export const FIXTURE_CHAPTER_ID = 7

/** Một lượt gọi `saveSegmentTargets` đã ghi lại. */
export type SaveCall = { chapterId: number; edits: { id: number; target_text: string }[] }

/**
 * Bộ ghi dùng chung. **Mảng sống** — ca test đọc lại sau mỗi lượt `await` để đếm **số lượt IPC**,
 * không chỉ hiệu ứng cuối. Đếm số lượt là mệnh đề của AC13 *(một lô, không N lượt `invoke`)* và
 * của AC18 *(kích hoạt ĐÚNG MỘT LẦN cho mỗi lượt đổi caret)*.
 */
export const saveCalls: SaveCall[] = []

/** Bật để lượt `saveSegmentTargets` kế tiếp trả về một lỗi ghi. */
export const failNextSave = { value: false }

export function resetRecorder(): void {
  saveCalls.length = 0
  failNextSave.value = false
}

/** Thân của `saveSegmentTargets` giả — mỗi tệp test cắm nó vào `vi.mock` của chính nó. */
export async function recordSave(
  chapterId: number,
  edits: readonly { id: number; target_text: string }[],
): Promise<{ outcome: { chapter_id: number; saved: number } | null; error: unknown }> {
  saveCalls.push({ chapterId, edits: edits.map((e) => ({ ...e })) })
  if (failNextSave.value) {
    failNextSave.value = false
    return {
      outcome: null,
      error: {
        code: 'store.write_failed',
        message_key: 'err.store.write_failed',
        params: {},
        retryable: true,
      },
    }
  }
  return { outcome: { chapter_id: chapterId, saved: edits.length }, error: null }
}

/** Thân của `readOpenChapterSegments` giả. */
export async function readFixture(): Promise<{
  loaded: { chapter_id: number; segments: ChapterSegment[] }
  error: null
}> {
  return {
    loaded: {
      chapter_id: FIXTURE_CHAPTER_ID,
      segments: FIXTURE_SEGMENTS.map((s) => ({ ...s })),
    },
    error: null,
  }
}
