/**
 * Hai vệ của `editorPanelState.ts::applyRegroup` — vòng rà Epic 3, 2026-08-25.
 *
 * ═════════════════════════════════════════════════════════════════════════════════
 * 🔴 VÌ SAO TỆP NÀY TỒN TẠI TÁCH KHỎI `editorRegroupNotice.test.ts`
 * ═════════════════════════════════════════════════════════════════════════════════
 * Tệp kia canh **dòng báo** mà một lượt gộp/tách sinh ra (Story 2.9 AC4). Tệp này canh hai
 * **vệ trong chính `applyRegroup`**, và cả hai đều là *"khuôn đã có bản vá đúng ở ngay cạnh
 * mà chỗ này bỏ sót"* — lớp lỗi rẻ nhất tìm bằng `grep`, đắt nhất tìm bằng test, và cho tới
 * lượt này **không cổng nào bắt được**.
 *
 * ① **Vệ BỊ XOÁ mà chú thích của nó còn nguyên.** `a2eaf7c~1:src/panels/editorPanelState.ts`
 *    dòng 1973 có `if (!inserted) next.push(...outcome.new_segments)`. Lượt Story 3.4b thay
 *    đúng dòng đó bằng `resetGlossaryMarks()`/`resetGlossaryConfirmStrip()` và **không nối
 *    lại**, nên đoạn văn *"đánh rơi một hàng mới là để đĩa và màn hình nói hai điều khác
 *    nhau, im lặng"* treo lơ lửng trên một chỗ trống suốt từ đó.
 *    ⚠️ Đây là *"khôi phục trung thành ≠ đúng"* đọc ngược: một lượt sửa máy móc giữ nguyên
 *    LỜI KHAI và đánh rơi THỨ NÓ KHAI. Chú thích không phải cổng — ca ① là cổng.
 *
 * ② **Phép kiểm mà chính tệp gọi là BẮT BUỘC, chỉ có ở MỘT trong hai chỗ gọi.** Chỗ gọi ở
 *    `switchChapter` mang `chapterId.value === sourceChapter.value.chapter_id` kèm một câu
 *    viết thẳng: *"Hai chỗ gọi phải giữ ĐÚNG một điều kiện, không phải hai biến thể của cùng
 *    một ý"* (rà 2026-08-21). Chỗ gọi trong `applyRegroup` thì chỉ kiểm `!== null`.
 *
 * ⚠️ **PHẠM VI:** `applyRegroup` là hàm RIÊNG TƯ, chỉ chạm được qua `mergeCurrentSegment()` —
 * khuôn dàn dựng chép từ `editorRegroupNotice.test.ts` (giả ở **biên IPC**, không một hàm
 * chỉ-test nào thêm vào mã sản phẩm).
 *
 * 🔴 **Đối chứng gỡ chỗ nối** *(chạy tay, ghi số vào §Spec Change Log)*: gỡ lại dòng
 * `if (!inserted) next.push(...)` ⇒ ca ① ĐỎ; gỡ vế `=== sourceChapter.value.chapter_id` ⇒
 * ca ③ ĐỎ.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { FIXTURE_SEGMENTS, readFixture, recordSave, resetRecorder } from './support/segmentFixture'
import type { ChapterSegment } from '../../src/config/segment'

/** Chương mà `readOpenChapterSegments` khai — nguồn của `editorChapterId`. */
const CHUONG_CUA_SEGMENT = 900
/** Chương mà `readOpenChapter` khai — nguồn của `sourceChapter.chapter_id`. */
const chuongCuaNguyenVan = { value: CHUONG_CUA_SEGMENT }

/** Hàng mới mà lượt gộp trả về — hình dạng dây, đúng `RegroupOutcome`. */
const HANG_MOI: ChapterSegment = {
  id: 14,
  ord: 1,
  source_text: '一。二。',
  target_text: 'Câu gộp.',
  is_paragraph_end: true,
  retired_at: null,
  status: 'draft',
  is_omitted: false,
  is_target_paragraph_end: true,
}

const ketQuaGop: {
  value: { outcome: { retired: ChapterSegment[]; new_segments: ChapterSegment[] } | null; error: unknown }
} = { value: { outcome: null, error: null } }

async function mergeGia() {
  return ketQuaGop.value
}

async function docSegmentGia() {
  return {
    loaded: {
      chapter_id: CHUONG_CUA_SEGMENT,
      segments: FIXTURE_SEGMENTS.map((s) => ({ ...s })),
    },
    error: null,
  }
}

async function docNguyenVanGia() {
  return {
    chapter: {
      chapter_id: chuongCuaNguyenVan.value,
      source_text: 'khong dung o day',
      source_lang: 'zh',
    },
    error: null,
  }
}

/** Đếm lượt gọi `glossary_marks_for_chapter` — mệnh đề trung tâm của ca ③/④. */
let soLuotGoiMarks = 0
async function traDauGia() {
  soLuotGoiMarks += 1
  return { marks: [], error: null }
}

// ⚠️ `vi.mock` HOIST lên đầu tệp; đường dẫn phân giải tương đối với TỆP NÀY.
vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return {
    ...actual,
    readOpenChapterSegments: docSegmentGia,
    saveSegmentTargets: recordSave,
    mergeSegments: mergeGia,
  }
})
vi.mock('../../src/config/chapter', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/chapter')>()
  return { ...actual, readOpenChapter: docNguyenVanGia }
})
vi.mock('../../src/config/glossary', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/glossary')>()
  return { ...actual, glossaryMarksForChapter: traDauGia }
})

/** Nạp lại module mỗi ca — state của Panel Editor là singleton cấp module. */
async function tuoi() {
  vi.resetModules()
  const editorState = await import('../../src/panels/editorPanelState')
  const sourceState = await import('../../src/panels/sourcePanelState')
  await editorState.ensureSegmentsLoaded()
  await sourceState.ensureChapterLoaded()
  // Lượt nạp Chương ở trên có thể tự phát một lượt tra dấu; ca dưới đây chỉ quan tâm tới các
  // lượt SAU mốc này, nên đặt lại bộ đếm tại đây.
  await new Promise((resolve) => setTimeout(resolve, 0))
  soLuotGoiMarks = 0
  return { editorState, sourceState }
}

beforeEach(() => {
  resetRecorder()
  readFixture // giữ import "đã dùng" — fixture chung nạp qua `docSegmentGia`
  ketQuaGop.value = { outcome: null, error: null }
  chuongCuaNguyenVan.value = CHUONG_CUA_SEGMENT
  soLuotGoiMarks = 0
})

describe('applyRegroup — vệ ① ảnh chụp KHÔNG có hàng nào của nhóm về hưu', () => {
  it('🔴 ① nhóm về hưu VẮNG MẶT hoàn toàn ⇒ hàng mới vẫn vào mảng, KHÔNG bị đánh rơi', async () => {
    const { editorState } = await tuoi()
    editorState.setEditorCaret(12)

    // Nhóm về hưu mang những id KHÔNG hàng nào trong ảnh chụp có — đúng ca mà đoạn chú thích
    // ở `applyRegroup` tả, và đúng ca mà vệ bị xoá đã tồn tại để đỡ.
    ketQuaGop.value = {
      outcome: {
        retired: [
          { ...HANG_MOI, id: 8001 },
          { ...HANG_MOI, id: 8002 },
        ],
        new_segments: [HANG_MOI],
      },
      error: null,
    }

    expect(await editorState.mergeCurrentSegment()).toBe('done')

    const ids = editorState.editorSegments.value.map((s) => s.id)
    // 🔴 Mệnh đề trung tâm: hàng mới CÓ MẶT. Không có vệ, `next` chỉ gồm ba hàng cũ và
    // `outcome.new_segments` biến mất — đĩa có một câu mà màn hình không bao giờ vẽ.
    expect(ids).toContain(HANG_MOI.id)
    // Nối vào CUỐI (không hàng cũ nào bị thay chỗ, vì không hàng nào về hưu trong ảnh chụp).
    expect(ids).toEqual([11, 12, 13, HANG_MOI.id])
  })

  it('② ca THƯỜNG (nhóm về hưu CÓ trong ảnh chụp) ⇒ hàng mới THẾ CHỖ, không nối thêm lần hai', async () => {
    // Vệ ① không được làm hỏng đường thường: nếu `inserted` bị đặt sai, hàng mới sẽ vào mảng
    // HAI lần — một lần thế chỗ, một lần nối đuôi.
    const { editorState } = await tuoi()
    editorState.setEditorCaret(12)
    ketQuaGop.value = {
      outcome: {
        retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })),
        new_segments: [HANG_MOI],
      },
      error: null,
    }

    expect(await editorState.mergeCurrentSegment()).toBe('done')

    const ids = editorState.editorSegments.value.map((s) => s.id)
    expect(ids).toEqual([HANG_MOI.id, 13])
    expect(ids.filter((id) => id === HANG_MOI.id)).toHaveLength(1)
  })
})

describe('applyRegroup — vệ ② hai ref phải cùng nói về MỘT Chương', () => {
  it('🔴 ③ `chapterId` và `sourceChapter.chapter_id` LỆCH nhau ⇒ KHÔNG một lượt tra dấu nào', async () => {
    // Cửa sổ lệch: `applyRegroup` chạy sau một `await`, nên một lượt chuyển Chương bay giữa
    // chừng để hai ref trỏ hai Chương khác nhau. Dàn dựng nó tường minh bằng cách cho hai
    // biên IPC khai hai số khác nhau.
    //
    // ⚠️ Phải đặt TRƯỚC `tuoi()`: `ensureChapterLoaded` là **idempotent** có chủ ý
    // (`sourcePanelState.ts` — `if (chapterRequested) return`, để `GridPanel` mount lại sau
    // một lượt đổi preset không phát IPC lần hai). Một lượt gọi lại SAU khi Chương đã nạp là
    // no-op, nên bản đầu của ca này dựng ra một cặp ref vẫn KHỚP rồi kết luận nhầm là sản
    // phẩm hỏng. Đo 2026-08-25.
    chuongCuaNguyenVan.value = CHUONG_CUA_SEGMENT + 1

    const { editorState, sourceState } = await tuoi()
    // Đối chứng của chính bàn test: hai ref THẬT SỰ lệch trước khi ca đo cái gì.
    expect(editorState.editorChapterId.value).toBe(CHUONG_CUA_SEGMENT)
    expect(sourceState.sourceChapter.value?.chapter_id).toBe(CHUONG_CUA_SEGMENT + 1)

    editorState.setEditorCaret(12)
    ketQuaGop.value = {
      outcome: {
        retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })),
        new_segments: [HANG_MOI],
      },
      error: null,
    }
    expect(await editorState.mergeCurrentSegment()).toBe('done')
    await new Promise((resolve) => setTimeout(resolve, 0))

    // 🔴 Mệnh đề trung tâm: 0 lượt tra. Nạp dấu của Chương A rồi gán cho Chương B đang hiện là
    // đúng khuyết tật mà watcher của `GridPanel.vue` đã tồn tại để chặn — một lỗi nhất thời
    // không được kéo theo một lỗi thứ hai không liên quan.
    expect(soLuotGoiMarks).toBe(0)
  })

  it('④ hai ref KHỚP nhau ⇒ có đúng một lượt tra dấu — vệ không được lấy mất tính năng', async () => {
    const { editorState } = await tuoi()
    editorState.setEditorCaret(12)
    ketQuaGop.value = {
      outcome: {
        retired: FIXTURE_SEGMENTS.slice(0, 2).map((s) => ({ ...s })),
        new_segments: [HANG_MOI],
      },
      error: null,
    }

    expect(await editorState.mergeCurrentSegment()).toBe('done')
    await new Promise((resolve) => setTimeout(resolve, 0))

    expect(soLuotGoiMarks).toBe(1)
  })
})
