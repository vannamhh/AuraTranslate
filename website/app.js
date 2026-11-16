document.addEventListener('DOMContentLoaded', () => {
  const htmlRoot = document.documentElement;
  const themeToggle = document.getElementById('theme-toggle');
  let savedTheme;
  try {
    savedTheme = localStorage.getItem('auratranslate-theme');
  } catch {
    savedTheme = null;
  }

  function setTheme(theme) {
    htmlRoot.dataset.theme = theme;
    themeToggle.textContent = theme === 'dark' ? '☀' : '☾';
    themeToggle.setAttribute('aria-label', theme === 'dark' ? 'Bật giao diện sáng' : 'Bật giao diện tối');
    themeToggle.setAttribute('aria-pressed', String(theme === 'light'));
  }

  setTheme(savedTheme === 'light' ? 'light' : 'dark');
  themeToggle.addEventListener('click', () => {
    const theme = htmlRoot.dataset.theme === 'dark' ? 'light' : 'dark';
    setTheme(theme);
    try {
      localStorage.setItem('auratranslate-theme', theme);
    } catch {
      return;
    }
  });

  const releases = 'https://github.com/vannamhh/AuraTranslate/releases/tag/v1.1.0';
  const primaryDownload = document.getElementById('primary-download-btn');
  primaryDownload.href = releases;
  const platform = navigator.userAgent.toLowerCase();
  if (platform.includes('mac')) {
    primaryDownload.textContent = 'Tải về cho macOS';
    document.getElementById('dl-mac').classList.add('featured');
  } else if (platform.includes('win')) {
    primaryDownload.textContent = 'Tải về cho Windows';
    document.getElementById('dl-win').classList.add('featured');
  }

  const screens = {
    workspace: 'Đối chiếu nguyên văn và bản dịch theo từng câu.',
    glossary: 'Quản lý tên riêng và thuật ngữ theo từng tác phẩm.',
    'translation-memory': 'Tìm lại và tái sử dụng những câu đã dịch.',
    reading: 'Đọc liền mạch, đánh dấu chỗ cần sửa.',
    export: 'Xem trước bản dịch trước khi xuất ra Word.'
  };
  const galleryImage = document.getElementById('gallery-image');
  const galleryCaption = document.getElementById('gallery-caption');
  const galleryButtons = document.querySelectorAll('.gallery-button');
  galleryButtons.forEach(button => {
    button.addEventListener('click', () => {
      const screen = button.dataset.screen;
      galleryImage.src = `assets/screenshots/${screen}.png`;
      galleryImage.alt = `AuraTranslate: ${screens[screen]}`;
      galleryCaption.textContent = screens[screen];
      galleryButtons.forEach(item => {
        const selected = item === button;
        item.classList.toggle('active', selected);
        item.setAttribute('aria-pressed', String(selected));
      });
    });
  });

  const faqItems = document.querySelectorAll('.faq-item');
  function setFaq(item, open) {
    item.classList.toggle('open', open);
    item.querySelector('.faq-question').setAttribute('aria-expanded', String(open));
    item.querySelector('.faq-answer').hidden = !open;
  }
  faqItems.forEach(item => {
    const button = item.querySelector('.faq-question');
    button.addEventListener('click', () => {
      const wasOpen = button.getAttribute('aria-expanded') === 'true';
      faqItems.forEach(other => setFaq(other, other === item && !wasOpen));
    });
    item.addEventListener('keydown', event => {
      if (event.key === 'Escape' && button.getAttribute('aria-expanded') === 'true') {
        setFaq(item, false);
        button.focus();
      }
    });
  });
});
