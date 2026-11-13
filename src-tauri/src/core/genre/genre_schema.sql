CREATE TABLE genre (
  seq        INTEGER PRIMARY KEY AUTOINCREMENT,
  id         TEXT    NOT NULL UNIQUE,
  name       TEXT    NOT NULL,
  CHECK (trim(name, ' ' || char(9) || char(10) || char(11) || char(12) || char(13)
                       || char(133) || char(160) || char(5760)
                       || char(8192) || char(8193) || char(8194) || char(8195)
                       || char(8196) || char(8197) || char(8198) || char(8199)
                       || char(8200) || char(8201) || char(8202)
                       || char(8232) || char(8233) || char(8239) || char(8287)
                       || char(12288)) <> '')
);
CREATE UNIQUE INDEX idx_genre_name ON genre (name);
INSERT INTO genre (seq, id, name) VALUES
(1, 'news', 'Thời sự'),
(2, 'commentary', 'Bình luận – Chính luận'),
(3, 'tcm', 'Đông y'),
(4, 'health', 'Sức khỏe – Dưỡng sinh'),
(5, 'cuisine', 'Ẩm thực'),
(6, 'family', 'Giáo dục – Gia đình'),
(7, 'travel', 'Du lịch'),
(8, 'culture', 'Văn hóa truyền thống'),
(9, 'history_figures', 'Lịch sử – Nhân vật'),
(10, 'spiritual', 'Tâm linh – Kỳ văn'),
(11, 'arts', 'Nghệ thuật'),
(12, 'business', 'Kinh tế – Kinh doanh'),
(13, 'science', 'Khoa học – Kỹ thuật'),
(14, 'legal', 'Pháp lý'),
(15, 'nonfiction', 'Phi hư cấu'),
(16, 'xianxia', 'Tiên hiệp'),
(17, 'xuanhuan', 'Huyền huyễn'),
(18, 'wuxia', 'Kiếm hiệp'),
(19, 'urban', 'Đô thị'),
(20, 'romance', 'Ngôn tình'),
(21, 'historical', 'Lịch sử – Cổ đại'),
(22, 'scifi', 'Khoa huyễn'),
(23, 'game', 'Võng du – Game'),
(24, 'horror', 'Linh dị – Kinh dị'),
(25, 'mystery', 'Trinh thám'),
(26, 'fanfic', 'Đồng nhân'),
(27, 'literary', 'Văn học'),
(28, 'children', 'Thiếu nhi'),
(29, 'other', 'Khác');
