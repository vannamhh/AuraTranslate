# -*- coding: utf-8 -*-
"""Header and Side Navigation for AuraTranslate Guide."""

def get_header_and_sidenav():
    return """
  <!-- Carbon Shell Header -->
  <header class="cds--header">
    <a href="#overview" class="cds--header__name">
      <svg width="20" height="20" viewBox="0 0 32 32" fill="currentColor">
        <path d="M16 2L3 9l13 7 13-7-13-7zm0 17.5L5.4 12.4 3 13.7 16 21l13-7.3-2.4-1.3L16 19.5zM16 26.5L5.4 19.4 3 20.7 16 28l13-7.3-2.4-1.3L16 26.5z"/>
      </svg>
      AuraTranslate
      <span class="cds--header__badge">Sổ tay Hướng dẫn Toàn diện v2.0</span>
    </a>

    <div class="cds--header__search">
      <input type="text" id="globalSearch" placeholder="Tìm kiếm chức năng, phím tắt, thao tác...">
      <span class="cds--header__search-kbd">⌘K</span>
    </div>

    <div class="cds--header__actions">
      <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="toggleTheme()" title="Đổi giao diện Sáng / Tối">
        <span id="themeBtnText">☀️ Giao diện Sáng</span>
      </button>
      <a href="#simulators" class="cds--btn cds--btn--secondary cds--btn--sm">
        <span>🎮 Bộ Mô Phỏng</span>
      </a>
      <a href="#shortcuts-table" class="cds--btn cds--btn--primary cds--btn--sm">
        <span>⌨️ Phím Tắt</span>
      </a>
    </div>
  </header>

  <!-- Carbon Side Navigation -->
  <nav class="cds--side-nav" id="sideNav">
    <div class="cds--side-nav__group-title">Khái niệm & Giao diện</div>
    <a href="#overview" class="cds--side-nav__link active">
      <span>1. Tổng quan & Triết lý Thiết kế</span>
      <span class="cds--side-nav__count">Offline</span>
    </a>
    <a href="#simulators" class="cds--side-nav__link">
      <span>2. Mô hình Giao diện & Mô phỏng</span>
      <span class="cds--side-nav__count">4 Demo</span>
    </a>

    <div class="cds--side-nav__group-title">Quản lý Tài liệu</div>
    <a href="#library-import" class="cds--side-nav__link">
      <span>3. Thư viện & 4 Cổng Nhập Tài liệu</span>
      <span class="cds--side-nav__count">EPUB/Word</span>
    </a>
    <a href="#chapter-management" class="cds--side-nav__link">
      <span>4. Quản lý Chương & Xuất xứ</span>
      <span class="cds--side-nav__count">Metadata</span>
    </a>

    <div class="cds--side-nav__group-title">Bàn Dịch Workspace</div>
    <a href="#workspace-core" class="cds--side-nav__link">
      <span>5. Bàn Dịch Workspace Chuyên sâu</span>
      <span class="cds--side-nav__count">Pinyin/HV</span>
    </a>
    <a href="#sentence-menu" class="cds--side-nav__link">
      <span>6. Menu Chuột Câu & Lịch sử Câu</span>
      <span class="cds--side-nav__count">Tách/Gộp</span>
    </a>
    <a href="#lookup-offline" class="cds--side-nav__link">
      <span>7. Tra Từ Điển Ngoại Tuyến Siêu tốc</span>
      <span class="cds--side-nav__count">&lt;50ms</span>
    </a>

    <div class="cds--side-nav__group-title">Trí Tuệ Nhân Tạo (AI)</div>
    <a href="#ai-translation" class="cds--side-nav__link">
      <span>8. Dịch AI Cá nhân & Hàng loạt</span>
      <span class="cds--side-nav__count">BYOK/Ollama</span>
    </a>
    <a href="#prompt-library" class="cds--side-nav__link">
      <span>9. Thư viện Lời nhắc & Soi Prompt</span>
      <span class="cds--side-nav__count">Inspector</span>
    </a>

    <div class="cds--side-nav__group-title">Dữ Liệu Dịch Thuật</div>
    <a href="#glossary-system" class="cds--side-nav__link">
      <span>10. Thuật ngữ & Tên riêng (Glossary)</span>
      <span class="cds--side-nav__count">Harvest</span>
    </a>
    <a href="#tm-alignment" class="cds--side-nav__link">
      <span>11. Bộ nhớ Dịch (TM) & Căn chỉnh</span>
      <span class="cds--side-nav__count">Fuzzy/TMX</span>
    </a>

    <div class="cds--side-nav__group-title">Soát Lỗi & Hoàn Thiện</div>
    <a href="#vietnamese-rules" class="cds--side-nav__link">
      <span>12. Quy ước Chữ Việt & Soát lỗi</span>
      <span class="cds--side-nav__count">0đ AI</span>
    </a>
    <a href="#word-review-reading" class="cds--side-nav__link">
      <span>13. Soát lỗi qua Word & Chế độ Đọc</span>
      <span class="cds--side-nav__count">Diff/Serif</span>
    </a>

    <div class="cds--side-nav__group-title">Hệ Thống & Tham Khảo</div>
    <a href="#settings-shortcuts" class="cds--side-nav__link">
      <span>14. Cài đặt & Bảng Phím tắt Toàn năng</span>
      <span class="cds--side-nav__count">282 Lệnh</span>
    </a>
  </nav>

  <!-- Main Content Container -->
  <main class="cds--main-content">
"""
