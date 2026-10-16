/**
 * History rows carry their origin, exactly one row is "in use", and every row carries a diff
 * against it, in the real WKWebView over the real IPC and `project.db`.
 */

import { realClick } from '../support/pointer.mjs'
import { openWorkspaceWithWork } from '../support/workspace.mjs'
import { waitForGridRows } from '../support/gridWait.mjs'

async function invokeCommand(name, args) {
  return browser.execute(
    async (n, a) => {
      const internals = window.__TAURI_INTERNALS__
      if (internals === undefined) throw new Error('không có cầu IPC trong webview')
      return internals.invoke(n, a)
    },
    name,
    args,
  )
}

async function signWith(chapterId, id, text) {
  await invokeCommand('save_segment_targets', { chapterId, edits: [{ id, target_text: text }] })
  return invokeCommand('confirm_segment', { segmentId: id })
}

describe('Story 12-19 — xuất xứ từng phiên bản và so với bản đang dùng', () => {
  it('ba phiên bản (tôi dịch · tôi dịch · người khác dịch): xuất xứ, một hàng đang dùng, diff và lớp phủ', async () => {
    await openWorkspaceWithWork('Story 12-19 — xuất xứ phiên bản')
    await waitForGridRows(1)
    const before = await invokeCommand('read_open_chapter_segments', {})
    const id = before.segments[0].id
    const chapterId = before.chapter_id

    await signWith(chapterId, id, 'The dragon roared.')
    await signWith(chapterId, id, 'The dragon whispered.')
    await invokeCommand('promote_ai_translation', {
      segmentId: id,
      targetText: 'The dragon slept.',
      force: false,
    })
    await invokeCommand('confirm_segment', { segmentId: id })

    const history = await invokeCommand('read_segment_history', { segmentId: id })
    await expect(history.versions.length).toBe(3)
    await expect(history.current_version_id).toBe(history.versions[0].id)
    await expect(history.versions.map((row) => row.translation_origin)).toEqual([
      'other',
      'self',
      'self',
    ])
    for (const row of history.versions) {
      await expect(Object.keys(row).some((key) => key.startsWith('baseline_'))).toBe(false)
    }
    await expect(history.versions[0].spans.every((span) => span.kind === 'equal')).toBe(true)
    await expect(history.versions[2].spans.some((span) => span.kind !== 'equal')).toBe(true)

    await realClick(await $(`[data-col="tgt"][data-segment-id="${id}"]`))
    await browser.pause(150)
    await browser.keys(['Meta', 'h'])

    await browser.waitUntil(async () => (await $$('[data-history-row]')).length === 3, {
      timeout: 15_000,
      timeoutMsg: 'Lớp phủ lịch sử không hiện đủ ba hàng sau 15 giây.',
    })
    await expect((await $$('[data-history-current="true"]')).length).toBe(1)
    const origins = await browser.execute(() =>
      Array.from(document.querySelectorAll('[data-history-row]')).map((row) =>
        row.getAttribute('data-history-origin'),
      ),
    )
    await expect(origins).toEqual(['other', 'self', 'self'])

    await expect(await $('[data-history-full]').isExisting()).toBe(true)
    await expect(await $('[data-history-diff]').isExisting()).toBe(false)
    const buttons = await $$('[data-history-detail] button.hist-act')
    await realClick(buttons[buttons.length - 1])
    await browser.waitUntil(async () => await $('[data-history-diff]').isExisting(), {
      timeout: 10_000,
      timeoutMsg: 'Nút "So với bản đang dùng" không bật diff.',
    })
    await expect(await $('[data-history-full]').isExisting()).toBe(false)

    await realClick((await $$('[data-history-detail] button.hist-act'))[1])
    await browser.waitUntil(async () => await $('[data-history-full]').isExisting(), {
      timeout: 10_000,
      timeoutMsg: 'Bấm lần nữa không trở lại toàn văn.',
    })
  })
})
