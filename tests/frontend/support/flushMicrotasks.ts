import { nextTick } from 'vue'

/**
 * Cùng vai `flushPromises()` của `@vue/test-utils`, nhưng không đi qua `setImmediate`/
 * `setTimeout`: dưới tải máy nặng một macrotask phải chờ tới lượt pha timer của vòng lặp sự
 * kiện, còn một microtask chạy ngay khi ngăn xếp hiện tại rỗng — nhường `rounds` lượt microtask
 * rồi một `nextTick()` để cả chuỗi mock IPC lẫn phản ứng của Vue kịp chạy xong.
 */
export async function flushPromises(rounds = 12): Promise<void> {
  for (let i = 0; i < rounds; i += 1) {
    await Promise.resolve()
  }
  await nextTick()
}
