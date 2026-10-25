import { describe, expect, it } from 'vitest'
import { genreChoices, genreLabel, workCardMeta } from '../../src/modes/libraryWorkCard'

const NOW = new Date('2026-10-13T12:00:00.000Z')

function ago(ms: number): string {
  return new Date(NOW.getTime() - ms).toISOString()
}

const SEC = 1000
const MIN = 60 * SEC
const HOUR = 60 * MIN
const DAY = 24 * HOUR

function work(over: Partial<{ source_lang: string; genre: string; updated_at: string }> = {}) {
  return { source_lang: 'zh', genre: 'Tiên hiệp', updated_at: ago(3 * HOUR), ...over }
}

describe('workCardMeta', () => {
  it('shows genre and relative edit time joined by a middle dot', () => {
    expect(workCardMeta(work(), NOW).line).toBe('Tiên hiệp · sửa 3 giờ trước')
  })

  it('missing genre keeps only the edit time', () => {
    expect(workCardMeta(work({ genre: '' }), NOW).line).toBe('sửa 3 giờ trước')
    expect(workCardMeta(work({ genre: '   ' }), NOW).line).toBe('sửa 3 giờ trước')
  })

  it('unreadable or empty updated_at keeps only the genre, with no stray text', () => {
    for (const bad of ['', 'khong-phai-ngay', '0000-00-00']) {
      const line = workCardMeta(work({ updated_at: bad }), NOW).line
      expect(line).toBe('Tiên hiệp')
      expect(line).not.toMatch(/Invalid|undefined|NaN|\b0\b/)
    }
  })

  it('missing genre and unreadable date give no line at all', () => {
    expect(workCardMeta(work({ genre: '', updated_at: 'xx' }), NOW).line).toBeNull()
  })

  it('a future updated_at is dropped instead of printing a negative age', () => {
    const future = new Date(NOW.getTime() + 5 * MIN).toISOString()
    expect(workCardMeta(work({ updated_at: future }), NOW).line).toBe('Tiên hiệp')
  })

  it('tier boundaries use strict less-than on the elapsed difference', () => {
    const line = (ms: number) => workCardMeta(work({ genre: '', updated_at: ago(ms) }), NOW).line
    expect(line(0)).toBe('vừa sửa')
    expect(line(MIN - 1)).toBe('vừa sửa')
    expect(line(MIN)).toBe('sửa 1 phút trước')
    expect(line(HOUR - 1)).toBe('sửa 59 phút trước')
    expect(line(HOUR)).toBe('sửa 1 giờ trước')
    expect(line(DAY - 1)).toBe('sửa 23 giờ trước')
    expect(line(DAY)).toBe('sửa hôm qua')
    expect(line(2 * DAY - 1)).toBe('sửa hôm qua')
    expect(line(2 * DAY)).toBe('sửa 2 ngày trước')
    expect(line(30 * DAY - 1)).toBe('sửa 29 ngày trước')
    expect(line(30 * DAY)).toBe('sửa 2026-09-13')
  })

  it('old dates print the UTC calendar day cut from the ISO string', () => {
    expect(workCardMeta(work({ genre: '', updated_at: '2025-01-02T23:59:59.000Z' }), NOW).line).toBe('sửa 2025-01-02')
  })

  it('source language label: zh and en are named, an unknown code prints as-is, blank is absent', () => {
    expect(workCardMeta(work({ source_lang: 'zh' }), NOW).langLabel).toBe('Tiếng Trung')
    expect(workCardMeta(work({ source_lang: 'en' }), NOW).langLabel).toBe('Tiếng Anh')
    expect(workCardMeta(work({ source_lang: 'ko' }), NOW).langLabel).toBe('ko')
    expect(workCardMeta(work({ source_lang: '' }), NOW).langLabel).toBeNull()
    expect(workCardMeta(work({ source_lang: '  ' }), NOW).langLabel).toBeNull()
  })
})

describe('genre list', () => {
  it('offers "Chưa chọn", then press genres, then specialist ones, then fiction, with "Khác" last', () => {
    const choices = genreChoices()
    expect(choices[0]).toEqual({ value: '', label: 'Chưa chọn' })
    expect(choices.slice(1).map((c) => c.label)).toEqual([
      'Thời sự',
      'Bình luận – Chính luận',
      'Đông y',
      'Sức khỏe – Dưỡng sinh',
      'Ẩm thực',
      'Giáo dục – Gia đình',
      'Du lịch',
      'Văn hóa truyền thống',
      'Lịch sử – Nhân vật',
      'Tâm linh – Kỳ văn',
      'Nghệ thuật',
      'Kinh tế – Kinh doanh',
      'Khoa học – Kỹ thuật',
      'Pháp lý',
      'Phi hư cấu',
      'Tiên hiệp',
      'Huyền huyễn',
      'Kiếm hiệp',
      'Đô thị',
      'Ngôn tình',
      'Đam mỹ',
      'Lịch sử – Cổ đại',
      'Khoa huyễn',
      'Võng du – Game',
      'Linh dị – Kinh dị',
      'Trinh thám',
      'Đồng nhân',
      'Văn học',
      'Thiếu nhi',
      'Khác',
    ])
    expect(choices[1].value).toBe('news')
  })

  it('maps an id to its label, shows a foreign value verbatim, and empty is null', () => {
    expect(genreLabel('xianxia')).toBe('Tiên hiệp')
    expect(genreLabel('Huyền huyễn')).toBe('Huyền huyễn')
    expect(genreLabel('general')).toBe('general')
    expect(genreLabel('  ')).toBeNull()
  })

  it('the card shows the label for an id and the raw text for a foreign value', () => {
    expect(workCardMeta(work({ genre: 'xianxia' }), NOW).line).toBe('Tiên hiệp · sửa 3 giờ trước')
    expect(workCardMeta(work({ genre: 'Huyền huyễn' }), NOW).line).toBe('Huyền huyễn · sửa 3 giờ trước')
  })
})
