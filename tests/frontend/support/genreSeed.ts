import type { GenreWire } from '../../../src/config/genre'

/** The 29 rows migration 18 seeds, in menu order. */
export const SEED_GENRES: readonly GenreWire[] = [
  { id: 'news', name: 'Thời sự' },
  { id: 'commentary', name: 'Bình luận – Chính luận' },
  { id: 'tcm', name: 'Đông y' },
  { id: 'health', name: 'Sức khỏe – Dưỡng sinh' },
  { id: 'cuisine', name: 'Ẩm thực' },
  { id: 'family', name: 'Giáo dục – Gia đình' },
  { id: 'travel', name: 'Du lịch' },
  { id: 'culture', name: 'Văn hóa truyền thống' },
  { id: 'history_figures', name: 'Lịch sử – Nhân vật' },
  { id: 'spiritual', name: 'Tâm linh – Kỳ văn' },
  { id: 'arts', name: 'Nghệ thuật' },
  { id: 'business', name: 'Kinh tế – Kinh doanh' },
  { id: 'science', name: 'Khoa học – Kỹ thuật' },
  { id: 'legal', name: 'Pháp lý' },
  { id: 'nonfiction', name: 'Phi hư cấu' },
  { id: 'xianxia', name: 'Tiên hiệp' },
  { id: 'xuanhuan', name: 'Huyền huyễn' },
  { id: 'wuxia', name: 'Kiếm hiệp' },
  { id: 'urban', name: 'Đô thị' },
  { id: 'romance', name: 'Ngôn tình' },
  { id: 'historical', name: 'Lịch sử – Cổ đại' },
  { id: 'scifi', name: 'Khoa huyễn' },
  { id: 'game', name: 'Võng du – Game' },
  { id: 'horror', name: 'Linh dị – Kinh dị' },
  { id: 'mystery', name: 'Trinh thám' },
  { id: 'fanfic', name: 'Đồng nhân' },
  { id: 'literary', name: 'Văn học' },
  { id: 'children', name: 'Thiếu nhi' },
  { id: 'other', name: 'Khác' },
]

export async function seedGenres(): Promise<void> {
  const state = await import('../../../src/genreState')
  state.setGenreRows(SEED_GENRES)
}
