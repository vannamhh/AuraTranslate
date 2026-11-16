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

  const releases = 'https://github.com/vannamhh/AuraTranslate/releases';
  const primaryDownload = document.getElementById('primary-download-btn');
  primaryDownload.href = releases;
  const platform = navigator.userAgent.toLowerCase();
  if (platform.includes('mac')) {
    primaryDownload.textContent = 'Tải DMG v1.1.0 cho macOS';
    primaryDownload.href = 'https://github.com/vannamhh/AuraTranslate/releases/download/v1.1.0/AuraTranslate_1.1.0_universal.dmg';
    document.getElementById('dl-mac').classList.add('featured');
  } else if (platform.includes('win')) {
    primaryDownload.textContent = 'Tải MSI v1.1.0 cho Windows';
    primaryDownload.href = 'https://github.com/vannamhh/AuraTranslate/releases/download/v1.1.0/AuraTranslate_1.1.0_x64_en-US.msi';
    document.getElementById('dl-win').classList.add('featured');
  }

  const screens = {
    workspace: 'Đối chiếu nguyên văn và bản dịch theo từng câu.',
    glossary: 'Glossary: ba thuật ngữ của riêng tác phẩm minh họa.',
    'translation-memory': 'Bộ nhớ dịch: các cặp câu từ những lần xác nhận trong tác phẩm minh họa.',
    reading: 'Chế độ đọc: đọc liền mạch và phân biệt các câu chưa xác nhận.',
    export: 'Xuất bản dịch: xem trước DOCX hai cột.'
  };
  const galleryImage = document.getElementById('gallery-image');
  const galleryCaption = document.getElementById('gallery-caption');
  const galleryOriginal = document.getElementById('gallery-original');
  const galleryButtons = document.querySelectorAll('.gallery-button');
  galleryButtons.forEach(button => {
    button.addEventListener('click', () => {
      const screen = button.dataset.screen;
      galleryImage.src = `assets/screenshots/${screen}.png`;
      galleryImage.alt = `Ảnh chụp AuraTranslate với dữ liệu mẫu. ${screens[screen]}`;
      galleryCaption.textContent = screens[screen];
      galleryOriginal.href = galleryImage.getAttribute('src');
      galleryButtons.forEach(item => {
        const selected = item === button;
        item.classList.toggle('active', selected);
        item.setAttribute('aria-pressed', String(selected));
      });
    });
  });

  const presets = {
    chinese: {
      lang: 'zh',
      source: '清晨，河面上飘着一层薄雾。林安走到旧码头，看见一只蓝色的小船。',
      target: 'Sáng sớm, một làn sương mỏng lững lờ trên mặt sông. Lâm An bước ra bến cũ, trông thấy một chiếc thuyền nhỏ màu xanh.',
      words: [
        { term: '薄雾', pronunciation: 'bó wù', label: 'Hán Việt', translation: 'Bạc vụ', note: 'Nghĩa trong câu: sương mỏng.' },
        { term: '林安', pronunciation: 'lín ān', label: 'Tên riêng mẫu', translation: 'Lâm An', note: 'Cách dịch được chọn cho ví dụ này.' }
      ]
    },
    english: {
      lang: 'en',
      source: 'Morning mist drifted above the river. Linh opened her notebook and sketched the old wooden pier.',
      target: 'Sương sớm lững lờ trên mặt sông. Linh mở cuốn sổ và phác họa chiếc cầu tàu cũ bằng gỗ.',
      words: [
        { term: 'morning mist', pronunciation: '', label: 'Nghĩa tiếng Việt', translation: 'Sương sớm', note: 'Ghi chú được soạn sẵn cho ví dụ.' },
        { term: 'wooden pier', pronunciation: '', label: 'Nghĩa tiếng Việt', translation: 'Cầu tàu bằng gỗ', note: 'Chọn cách dịch theo bối cảnh ven sông.' }
      ]
    }
  };
  const presetButtons = document.querySelectorAll('.pg-tab-btn');
  function loadPreset(key) {
    const preset = presets[key];
    const source = document.getElementById('pg-source-text');
    source.textContent = preset.source;
    source.lang = preset.lang;
    document.getElementById('pg-target-text').textContent = preset.target;
    const inspector = document.getElementById('pg-inspector-content');
    inspector.replaceChildren();
    preset.words.forEach(word => {
      const card = document.createElement('article');
      card.className = 'term-card';
      const term = document.createElement('h4');
      term.textContent = [word.term, word.pronunciation].filter(Boolean).join(' · ');
      term.lang = preset.lang;
      const meaning = document.createElement('p');
      meaning.className = 'term-meaning';
      meaning.textContent = `${word.label}: ${word.translation}`;
      const note = document.createElement('p');
      note.textContent = word.note;
      card.append(term, meaning, note);
      inspector.append(card);
    });
    presetButtons.forEach(button => {
      const selected = button.dataset.preset === key;
      button.classList.toggle('active', selected);
      button.setAttribute('aria-pressed', String(selected));
    });
  }
  presetButtons.forEach(button => button.addEventListener('click', () => loadPreset(button.dataset.preset)));
  loadPreset('chinese');

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
