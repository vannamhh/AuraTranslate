/**
 * `modes/libraryImport.ts::finishImportSubmission` — đường đổi Tác phẩm dọn state lớp phủ
 * quản lý luật quy ước.
 *
 * Vì sao cần một ca riêng: `check:panel-refs` chỉ chứng minh mỗi ô nhớ cấp module ĐƯỢC GÁN
 * bên trong một hàm `reset*` của chính tệp đó. Nó KHÔNG chứng minh hàm ấy có ai gọi — đúng
 * lớp lỗi mà `resetSegmentHistory()` đã lọt qua một lần, và chính cổng ấy ghi lại trong
 * `scripts/check-panel-refs.mjs`. Một hàm reset viết ra chỉ để qua cổng là một hàm chết.
 *
 * Danh sách luật thuộc tầng Toàn cục và sống sót một lượt đổi Tác phẩm là đúng; thứ KHÔNG
 * được sống sót là cờ mở, lỗi đang treo và hai ô nháp của phiên đã mở nó.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { readFixture } from './support/segmentFixture'

const TAC_PHAM_MOI = { meta: { work_id: 'w-moi', genre: '', name: 'Tac pham B', chapter_count: 1 } }

vi.mock('../../src/config/project', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/project')>()
  return {
    ...actual,
    previewImportEncodingFromText: async () => ({
      preview: { confidence: 'self_declared', selected_encoding: 'UTF-8', candidates: [] },
      error: null,
    }),
    confirmImportWithEncoding: async () => ({ created: TAC_PHAM_MOI, error: null }),
  }
})

vi.mock('../../src/config/segment', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/segment')>()
  return { ...actual, readOpenChapterSegments: readFixture }
})

vi.mock('../../src/config/convention', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../../src/config/convention')>()
  return {
    ...actual,
    listConventionRules: async (tier: string) => ({
      rules: tier === 'work' ? [] : [{ rule_id: 'r-1', avoid: 'đảm bảo', replacement_cased: 'bảo đảm' }],
      error: null,
    }),
    listConventionIgnores: async () => ({ rows: [], error: null }),
  }
})

vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))

beforeEach(() => {
  vi.resetModules()
})

describe('🔴 đổi Tác phẩm ⇒ state lớp phủ quản lý luật quy ước KHÔNG sống sót', () => {
  it('nộp rồi xác nhận gọi `resetConventionManage()` — lớp phủ đóng, danh sách và hai ô nháp sạch', async () => {
    const manage = await import('../../src/conventionManageState')
    const nhap = await import('../../src/modes/libraryImport')
    const preview = await import('../../src/importPreviewState')

    // Dựng đúng trạng thái nguy hiểm: lớp phủ ĐANG MỞ, danh sách đã nạp, hai ô nháp có chữ.
    await manage.openConventionManage()
    manage.conventionAvoidDraft.value = 'đảm bảo'
    manage.conventionReplacementDraft.value = 'bảo đảm'
    expect(manage.conventionManageIsOpen.value).toBe(true)
    expect(manage.conventionManageHasLoaded.value).toBe(true)
    expect(manage.conventionManageRules.value).toHaveLength(1)

    nhap.name.value = 'Tac pham moi'
    nhap.pastedText.value = 'Cau nguyen van cua Tac pham moi.'
    await nhap.submitPastedText()
    const result = await preview.confirmImportPreview()
    nhap.finishImportSubmission(result.created, result.error)

    // Tiền đề của ca: lượt nhập phải THÀNH CÔNG, không thì ca xanh vì một lý do sai.
    expect(nhap.createdWork.value).not.toBeNull()

    expect(manage.conventionManageIsOpen.value).toBe(false)
    expect(manage.conventionManageHasLoaded.value).toBe(false)
    expect(manage.conventionManageRules.value).toEqual([])
    expect(manage.conventionManageBusy.value).toBe(false)
    expect(manage.conventionManageError.value).toBeNull()
    expect(manage.conventionAvoidDraft.value).toBe('')
    expect(manage.conventionReplacementDraft.value).toBe('')
  })
})
