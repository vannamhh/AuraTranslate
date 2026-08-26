/**
 * `CommandRegistry` — §I/O Matrix ⑯ (cụm D vá, vòng rà Epic 3). `src/commands/registry.ts`
 * là một module thuần (không `import` gì cả — banner đầu tệp), an toàn để `import` thẳng
 * trong vitest, cùng khuôn Kiểm C/D/E của `check-commands.mjs` (chạy bằng Node trần).
 */
import { describe, expect, it, vi } from 'vitest'
import { createRegistry } from '../../src/commands/registry'

describe('dispatch — spec.run() ném KHÔNG được thoát ra ngoài (§I/O Matrix ⑯)', () => {
  it('handler ném ⇒ dispatch ghi chẩn đoán nêu đích danh id rồi TRẢ VỀ, không ném lại', () => {
    const registry = createRegistry()
    registry.register({
      id: 'test.throwing',
      labelKey: 'command.test.throwing',
      run: () => {
        throw new Error('handler hỏng')
      },
    })
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    expect(() => registry.dispatch('test.throwing')).not.toThrow()
    expect(errorSpy).toHaveBeenCalledTimes(1)
    expect(String(errorSpy.mock.calls[0]?.[0])).toContain('test.throwing')

    errorSpy.mockRestore()
  })

  it('handler ném KHÔNG ngăn một lượt dispatch KẾ TIẾP (cho command khác) chạy bình thường — cùng khuôn "ngoại lệ không thoát khỏi listener keydown"', () => {
    const registry = createRegistry()
    registry.register({
      id: 'test.throwing',
      labelKey: 'command.test.throwing',
      run: () => {
        throw new Error('handler hỏng')
      },
    })
    const okRun = vi.fn()
    registry.register({ id: 'test.ok', labelKey: 'command.test.ok', run: okRun })
    const errorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    registry.dispatch('test.throwing')
    registry.dispatch('test.ok')

    expect(okRun).toHaveBeenCalledTimes(1)

    errorSpy.mockRestore()
  })

  it('🔴 id CHƯA ĐĂNG KÝ vẫn NÉM — nửa cưỡng chế lúc chạy của AC1, KHÔNG bị bọc trong lượt vá này', () => {
    const registry = createRegistry()
    expect(() => registry.dispatch('test.unregistered')).toThrow(/chưa đăng ký/)
  })
})
