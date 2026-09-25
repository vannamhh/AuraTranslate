/**
 * Unit tests for `scripts/ci-previous-verdict.mjs`'s pure core, `ciPreviousVerdict`. The
 * real `spawnSync` edge (`spawnRun`) is never exercised here — every case injects a fake
 * `run(cmd, args)` so the suite runs the same on every OS and never touches the network.
 */
import { describe, it, expect, vi } from 'vitest'
import { ciPreviousVerdict, printVerdict } from '../../scripts/ci-previous-verdict.mjs'

interface RunResult {
  status: number | null
  stdout: string
  error?: Error
}

const SHA = 'abc123def456'

function makeRun(overrides: { auth?: RunResult; upstream?: RunResult; list?: RunResult } = {}) {
  const auth = overrides.auth ?? { status: 0, stdout: '' }
  const upstream = overrides.upstream ?? { status: 0, stdout: `${SHA}\n` }
  const list = overrides.list ?? { status: 0, stdout: '[]' }
  const calls: { cmd: string; args: string[] }[] = []
  const run = (cmd: string, args: string[]): RunResult => {
    calls.push({ cmd, args })
    if (cmd === 'gh' && args[0] === 'auth') return auth
    if (cmd === 'git' && args[0] === 'rev-parse') return upstream
    if (cmd === 'gh' && args[0] === 'run') return list
    throw new Error(`unexpected run(${cmd}, ${JSON.stringify(args)})`)
  }
  return Object.assign(run, { calls })
}

const runsJson = (headSha: string, status: string, conclusion: string | null) =>
  JSON.stringify([{ headSha, status, conclusion }])

describe('ciPreviousVerdict — đúng bốn phán quyết có tên', () => {
  it('conclusion failure ⇒ kind failure, mang theo sha', () => {
    const run = makeRun({ list: { status: 0, stdout: runsJson(SHA, 'completed', 'failure') } })
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'failure', sha: SHA })
  })

  it('conclusion success ⇒ kind success', () => {
    const run = makeRun({ list: { status: 0, stdout: runsJson(SHA, 'completed', 'success') } })
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'success' })
  })

  it('status in_progress (chưa completed) ⇒ kind in_progress', () => {
    const run = makeRun({ list: { status: 0, stdout: runsJson(SHA, 'in_progress', null) } })
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'in_progress' })
  })

  it('status queued ⇒ cũng kind in_progress', () => {
    const run = makeRun({ list: { status: 0, stdout: runsJson(SHA, 'queued', null) } })
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'in_progress' })
  })

  it('gọi `gh run list` với `--json` ngay sau đó là `headSha,conclusion,status`', () => {
    // Cột chặt danh sách trường gọi thật với đúng ba trường `ciPreviousVerdict` đọc lại —
    // một fixture bỏ qua `args` sẽ không bắt được việc lỡ tay bớt một trường khỏi `--json`.
    const run = makeRun({ list: { status: 0, stdout: runsJson(SHA, 'completed', 'success') } })
    ciPreviousVerdict({ run })
    const listCall = run.calls.find((c) => c.cmd === 'gh' && c.args[0] === 'run')
    expect(listCall).toBeDefined()
    const jsonAt = listCall?.args.indexOf('--json') ?? -1
    expect(jsonAt).toBeGreaterThanOrEqual(0)
    expect(listCall?.args[jsonAt + 1]).toBe('headSha,conclusion,status')
  })
})

describe('ciPreviousVerdict — im lặng bỏ qua, không phán quyết nào chặn push', () => {
  it('không lượt chạy nào khớp sha thượng nguồn ⇒ silent', () => {
    const run = makeRun({ list: { status: 0, stdout: runsJson('khac-sha-hoan-toan', 'completed', 'failure') } })
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'silent' })
  })

  it('gh không cài (spawn báo lỗi ENOENT) ⇒ silent', () => {
    const enoent = Object.assign(new Error('spawnSync gh ENOENT'), { code: 'ENOENT' })
    const run = makeRun({ auth: { status: null, stdout: '', error: enoent } })
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'silent' })
  })

  it('gh auth status thoát khác 0 (chưa đăng nhập) ⇒ silent', () => {
    const run = makeRun({ auth: { status: 1, stdout: '' } })
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'silent' })
  })

  it('không có nhánh thượng nguồn (git rev-parse rỗng) ⇒ silent', () => {
    const run = makeRun({ upstream: { status: 0, stdout: '' } })
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'silent' })
  })

  it('gh run list hết giờ (spawn báo lỗi ETIMEDOUT) ⇒ silent', () => {
    const timeout = Object.assign(new Error('spawnSync gh ETIMEDOUT'), { code: 'ETIMEDOUT' })
    const run = makeRun({ list: { status: null, stdout: '', error: timeout } })
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'silent' })
  })

  it('gh run list bị giết bởi tín hiệu, có error đúc từ signal ⇒ silent', () => {
    // `spawnRun` (không test trực tiếp ở đây) đúc một `Error` từ `signal` khi Node không
    // tự đặt `.error` — mô phỏng đúng hình dạng kết quả mà nó trả cho `run`.
    const killed = new Error('spawnSync gh killed by signal SIGTERM')
    const run = makeRun({ list: { status: null, stdout: '', error: killed } })
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'silent' })
  })

  it('JSON hỏng từ gh run list ⇒ silent, không ném', () => {
    const run = makeRun({ list: { status: 0, stdout: '{ not json' } })
    expect(() => ciPreviousVerdict({ run })).not.toThrow()
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'silent' })
  })

  it('JSON hợp lệ nhưng không phải mảng ⇒ silent', () => {
    const run = makeRun({ list: { status: 0, stdout: '{"headSha":"x"}' } })
    expect(ciPreviousVerdict({ run })).toEqual({ kind: 'silent' })
  })
})

describe('printVerdict — nhãn và màu giữ nguyên như bản shell cũ', () => {
  it('failure: xuống dòng rồi in cảnh báo vàng có sha', () => {
    const writeSpy = vi.spyOn(process.stdout, 'write').mockImplementation(() => true)
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
    let written = ''
    let logged = ''
    try {
      printVerdict({ kind: 'failure', sha: SHA })
      written = writeSpy.mock.calls.map((c) => String(c[0])).join('')
      logged = logSpy.mock.calls.map((c) => c.join(' ')).join('\n')
    } finally {
      writeSpy.mockRestore()
      logSpy.mockRestore()
    }
    expect(written).toContain('ci (trước)')
    expect(logged).toContain(SHA)
    expect(logged).toContain('CẢNH BÁO')
  })

  it('success: in OK, không cảnh báo', () => {
    const writeSpy = vi.spyOn(process.stdout, 'write').mockImplementation(() => true)
    const logSpy = vi.spyOn(console, 'log').mockImplementation(() => {})
    let logged = ''
    try {
      printVerdict({ kind: 'success' })
      logged = logSpy.mock.calls.map((c) => c.join(' ')).join('\n')
    } finally {
      writeSpy.mockRestore()
      logSpy.mockRestore()
    }
    expect(logged).toContain('OK')
    expect(logged).not.toContain('CẢNH BÁO')
  })
})
