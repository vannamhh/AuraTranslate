/**
 * `checkPairDataBarriers` (`e2e/support/pairDataBarriers.mjs`) — ba hàng rào dữ liệu thật của
 * `wdio.conf.mjs::onComplete`, tách ra để lái được bằng cặp GIẢ LẬP thay vì một app Tauri thật
 * (`deferred-work.md`, "cách ly trạng thái giữa các spec": vòng `onComplete` trước đây không
 * hàm nào tách riêng cho một ca đơn vị chạy được). Dùng thư mục tạm THẬT (`mkdtempSync`) —
 * hàm đọc/ghi `node:fs` thật, không giả lập fs.
 */
import { afterEach, describe, expect, it } from 'vitest'
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { tmpdir } from 'node:os'

import { checkPairDataBarriers } from '../../e2e/support/pairDataBarriers.mjs'

const REAL_LIBRARY_PATH = '/Users/nguoi-dung/Documents/AuraTranslate'
const CONTEXT = {
  realLibraryPathValue: REAL_LIBRARY_PATH,
  globalDbFile: 'global.db',
  libraryIndexDbFile: 'library-index.db',
  dataDirEnv: 'AURATRANSLATE_E2E_DATA_DIR',
  libraryRootEnv: 'AURATRANSLATE_E2E_LIBRARY_ROOT',
}

const daTao: string[] = []
function capThuMucTam(): string {
  const dir = mkdtempSync(join(tmpdir(), 'aura-e2e-pair-barrier-'))
  daTao.push(dir)
  return dir
}

afterEach(() => {
  for (const dir of daTao.splice(0)) rmSync(dir, { recursive: true, force: true })
})

function capMotCap(overrides: Partial<{ realLibraryBefore: string; realLibraryAfter: string; exitCode: number | undefined }> = {}) {
  return {
    dataDir: capThuMucTam(),
    libraryDir: capThuMucTam(),
    realLibraryBefore: 'chu-ky-A',
    realLibraryAfter: 'chu-ky-A',
    exitCode: 0,
    ...overrides,
  }
}

describe('checkPairDataBarriers — cặp qua cả ba hàng rào', () => {
  it('chữ ký Library thật y nguyên, không library-index.db, global.db có mặt ⇒ 0 lỗi', () => {
    const pair = capMotCap()
    writeFileSync(join(pair.dataDir, CONTEXT.globalDbFile), 'gia-lap-sqlite')

    expect(checkPairDataBarriers(pair, CONTEXT)).toEqual([])
  })

  it('worker chưa từng chạy (exitCode undefined, cặp cuối cùng) ⇒ bỏ qua hàng rào global.db dù thiếu tệp', () => {
    const pair = capMotCap({ exitCode: undefined })

    expect(checkPairDataBarriers(pair, CONTEXT)).toEqual([])
  })
})

describe('checkPairDataBarriers — hàng rào chiều ÂM (chữ ký Library thật)', () => {
  it('chữ ký ĐỔI giữa trước/sau ⇒ đúng một lỗi nêu tên "ĐỔI"', () => {
    const pair = capMotCap({ realLibraryBefore: 'chu-ky-A', realLibraryAfter: 'chu-ky-B' })
    writeFileSync(join(pair.dataDir, CONTEXT.globalDbFile), 'gia-lap-sqlite')

    const loi = checkPairDataBarriers(pair, CONTEXT)
    expect(loi).toHaveLength(1)
    expect(loi[0]).toContain('ĐỔI')
    expect(loi[0]).toContain(CONTEXT.libraryRootEnv)
  })

  it('đọc `libraryRootEnv` từ `context`, không đúc cứng tên biến', () => {
    const pair = capMotCap({ realLibraryBefore: 'chu-ky-A', realLibraryAfter: 'chu-ky-B' })
    writeFileSync(join(pair.dataDir, CONTEXT.globalDbFile), 'gia-lap-sqlite')

    const loi = checkPairDataBarriers(pair, { ...CONTEXT, libraryRootEnv: 'MOT_TEN_KHAC' })
    expect(loi[0]).toContain('MOT_TEN_KHAC')
    expect(loi[0]).not.toContain('AURATRANSLATE_E2E_LIBRARY_ROOT')
  })
})

describe('checkPairDataBarriers — hàng rào chiều ĐỌC (library-index.db)', () => {
  it('library-index.db KHÔNG chứa đường dẫn Library thật ⇒ 0 lỗi từ hàng rào này', () => {
    const pair = capMotCap()
    writeFileSync(join(pair.dataDir, CONTEXT.libraryIndexDbFile), 'khong-lien-quan')
    writeFileSync(join(pair.dataDir, CONTEXT.globalDbFile), 'gia-lap-sqlite')

    expect(checkPairDataBarriers(pair, CONTEXT)).toEqual([])
  })

  it('library-index.db chứa NGUYÊN VĂN đường dẫn Library thật ⇒ đúng một lỗi nêu tên tệp', () => {
    const pair = capMotCap()
    writeFileSync(
      join(pair.dataDir, CONTEXT.libraryIndexDbFile),
      `...SQLite...${REAL_LIBRARY_PATH}...trang-du-lieu...`,
    )
    writeFileSync(join(pair.dataDir, CONTEXT.globalDbFile), 'gia-lap-sqlite')

    const loi = checkPairDataBarriers(pair, CONTEXT)
    expect(loi).toHaveLength(1)
    expect(loi[0]).toContain(CONTEXT.libraryIndexDbFile)
  })
})

describe('checkPairDataBarriers — hàng rào chiều DƯƠNG (global.db)', () => {
  it('worker xanh (exitCode 0) mà KHÔNG có global.db ⇒ đúng một lỗi nêu tên "KHÔNG thấy"', () => {
    const pair = capMotCap({ exitCode: 0 })

    const loi = checkPairDataBarriers(pair, CONTEXT)
    expect(loi).toHaveLength(1)
    expect(loi[0]).toContain('KHÔNG thấy')
  })

  it('worker ĐỎ (exitCode khác 0) mà KHÔNG có global.db ⇒ 0 lỗi — spec có thể đã dừng trước khi app kịp tạo kho', () => {
    const pair = capMotCap({ exitCode: 1 })

    expect(checkPairDataBarriers(pair, CONTEXT)).toEqual([])
  })
})

describe('checkPairDataBarriers — nhiều hàng rào cùng vỡ', () => {
  it('cả ba hàng rào cùng vỡ ⇒ đúng ba lỗi, không dừng ở lỗi đầu', () => {
    const pair = capMotCap({ realLibraryBefore: 'chu-ky-A', realLibraryAfter: 'chu-ky-B', exitCode: 0 })
    writeFileSync(join(pair.dataDir, CONTEXT.libraryIndexDbFile), REAL_LIBRARY_PATH)

    expect(checkPairDataBarriers(pair, CONTEXT)).toHaveLength(3)
  })
})
