/**
 * Unit tests for `scripts/lib/floor-judge.mjs`, the pure floor judge every JS gate's
 * `*_FLOOR` population check routes through. Mirrors
 * `src-tauri/tests/boundary_scan_contract.rs`'s cases for the Rust counterpart.
 */
import { describe, it, expect } from 'vitest'
import { judgeFloor } from '../../scripts/lib/floor-judge.mjs'

describe('judgeFloor — không đỏ oan ở đúng biên', () => {
  it('floor đúng 80% của live: qua (biên dưới của dải trôi, chưa trôi)', () => {
    expect(judgeFloor(80, 100, 'TEST_FLOOR', 'tệp giả')).toEqual({ ok: true })
  })

  it('floor đúng 85% của live: qua', () => {
    expect(judgeFloor(85, 100, 'TEST_FLOOR', 'tệp giả')).toEqual({ ok: true })
  })

  it('floor bằng đúng live (100%): qua', () => {
    expect(judgeFloor(100, 100, 'TEST_FLOOR', 'tệp giả')).toEqual({ ok: true })
  })
})

describe('judgeFloor — đỏ được, đúng vế', () => {
  it('live < floor: đỏ với "Cây quá nhỏ để là thật"', () => {
    const v = judgeFloor(50, 49, 'TEST_FLOOR', 'tệp giả')
    expect(v.ok).toBe(false)
    expect(v.ok === false && v.message).toContain('Cây quá nhỏ để là thật')
    expect(v.ok === false && v.message).toContain('TEST_FLOOR')
  })

  it('floor ngay dưới 80% của live (79/100): đỏ với "đã trôi dưới 80%"', () => {
    const v = judgeFloor(79, 100, 'TEST_FLOOR', 'tệp giả')
    expect(v.ok).toBe(false)
    expect(v.ok === false && v.message).toContain('đã trôi dưới 80%')
  })

  it('thông báo trôi nêu đúng tên hằng và ceil(0.85 × live)', () => {
    const v = judgeFloor(21, 98, 'SRC_RS_FLOOR', 'tệp `.rs`')
    expect(v.ok).toBe(false)
    // ceil(0.85 * 98) = ceil(83.3) = 84
    expect(v.ok === false && v.message).toContain('SRC_RS_FLOOR')
    expect(v.ok === false && v.message).toContain('98')
    expect(v.ok === false && v.message).toContain('ceil(0.85 × live) = 84')
  })

  it('thông báo cây-quá-nhỏ nêu đúng tên hằng và sàn', () => {
    const v = judgeFloor(99, 98, 'RS_FILE_FLOOR', 'tệp `.rs`')
    expect(v.ok).toBe(false)
    expect(v.ok === false && v.message).toContain('RS_FILE_FLOOR')
    expect(v.ok === false && v.message).toContain('98')
    expect(v.ok === false && v.message).toContain('99')
  })
})
