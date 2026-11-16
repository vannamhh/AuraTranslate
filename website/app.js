// AuraTranslate Landing Page Interactive Logic
document.addEventListener('DOMContentLoaded', () => {
  // 1. Theme Toggle (Dark vs Warm Bookish Light)
  const themeToggle = document.getElementById('theme-toggle');
  const htmlRoot = document.documentElement;

  const savedTheme = localStorage.getItem('auratranslate-theme') || 'dark';
  htmlRoot.setAttribute('data-theme', savedTheme);
  updateThemeIcon(savedTheme);

  if (themeToggle) {
    themeToggle.addEventListener('click', () => {
      const currentTheme = htmlRoot.getAttribute('data-theme') || 'dark';
      const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
      htmlRoot.setAttribute('data-theme', newTheme);
      localStorage.setItem('auratranslate-theme', newTheme);
      updateThemeIcon(newTheme);
    });
  }

  function updateThemeIcon(theme) {
    if (!themeToggle) return;
    if (theme === 'light') {
      themeToggle.innerHTML = `
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
        </svg>`;
      themeToggle.setAttribute('title', 'Chuyển sang Giao diện Tối');
    } else {
      themeToggle.innerHTML = `
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="5"></circle>
          <line x1="12" y1="1" x2="12" y2="3"></line>
          <line x1="12" y1="21" x2="12" y2="23"></line>
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
          <line x1="1" y1="12" x2="3" y2="12"></line>
          <line x1="21" y1="12" x2="23" y2="12"></line>
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
        </svg>`;
      themeToggle.setAttribute('title', 'Chuyển sang Giao diện Giấy Cổ (Warm Light)');
    }
  }

  // 2. OS Detection for Primary Download CTA
  const userAgent = navigator.userAgent.toLowerCase();
  const primaryDownloadBtn = document.getElementById('primary-download-btn');
  const macCard = document.getElementById('dl-mac');
  const winCard = document.getElementById('dl-win');

  if (userAgent.includes('mac')) {
    if (primaryDownloadBtn) {
      primaryDownloadBtn.innerHTML = `
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M15.97 6.37c.62-.75 1.04-1.8 0.93-2.85-.9.04-2 .6-2.65 1.35-.58.66-1.09 1.73-.95 2.76.99.08 2.05-.51 2.67-1.26z"/>
        </svg>
        Tải cho macOS (Apple Silicon & Intel)`;
      primaryDownloadBtn.href = "#download";
    }
    if (macCard) macCard.classList.add('featured');
    if (winCard) winCard.classList.remove('featured');
  } else if (userAgent.includes('win')) {
    if (primaryDownloadBtn) {
      primaryDownloadBtn.innerHTML = `
        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
          <path d="M0 3.449L9.75 2.1v9.451H0m10.949-9.602L24 0v11.4h-13.051M0 12.6h9.75v9.451L0 20.699M10.949 12.6H24V24l-13.051-1.949"/>
        </svg>
        Tải cho Windows (MSI & Portable)`;
      primaryDownloadBtn.href = "#download";
    }
    if (winCard) winCard.classList.add('featured');
    if (macCard) macCard.classList.remove('featured');
  }

  // 3. Interactive Hero Lookup Card
  const triggerWords = document.querySelectorAll('.interactive-word');
  const lookupPopover = document.getElementById('hero-lookup-popover');
  const lookupTerm = document.getElementById('lookup-term');
  const lookupPinyin = document.getElementById('lookup-pinyin');
  const lookupHanviet = document.getElementById('lookup-hanviet');
  const lookupDef = document.getElementById('lookup-def');
  const lookupBadge = document.getElementById('lookup-badge');

  const dictData = {
    '星河灿烂': {
      pinyin: 'xīng hé càn làn',
      hanviet: 'Tinh Hà Xán Lạn',
      def: 'Dải ngân hà sáng rực lấp lánh; chỉ cảnh trời đêm muôn ngàn ánh sao tráng lệ.',
      badge: 'Từ vựng Cổ phong / Thơ Tào Thao'
    },
    '心神激荡': {
      pinyin: 'xīn shén jī dàng',
      hanviet: 'Tâm Thần Kích Đãng',
      def: 'Tâm can cuộn trào, cảm xúc xao động dữ dội không kìm nén nổi.',
      badge: 'Thành ngữ văn học'
    },
    '韩立': {
      pinyin: 'hán lì',
      hanviet: 'Hàn Lập',
      def: 'Nhân vật chính trong Phàm Nhân Tu Tiên. Tâm tính cẩn trọng, kiên nghị, sát phạt quyết đoán.',
      badge: 'Glossary: Tên nhân vật (Nhớ tự động)'
    }
  };

  triggerWords.forEach(word => {
    word.addEventListener('click', (e) => {
      e.stopPropagation();
      const text = word.textContent.trim();
      const info = dictData[text];
      if (info && lookupPopover) {
        lookupTerm.textContent = text;
        lookupPinyin.textContent = info.pinyin;
        lookupHanviet.textContent = info.hanviet;
        lookupDef.textContent = info.def;
        lookupBadge.textContent = info.badge;

        lookupPopover.style.display = 'block';
        triggerWords.forEach(w => w.classList.remove('highlight-lookup'));
        word.classList.add('highlight-lookup');
      }
    });
  });

  // 4. Interactive Playground Simulator
  const pgTabs = document.querySelectorAll('.pg-tab-btn');
  const pgSource = document.getElementById('pg-source-text');
  const pgTarget = document.getElementById('pg-target-text');
  const pgInspector = document.getElementById('pg-inspector-content');

  const playgroundPresets = {
    'xianxia': {
      source: '韩立凝视着天际翻滚的紫金色雷云，手中青竹蜂云剑发出阵阵清鸣。他深知此番渡劫绝非儿戏。',
      target: 'Hàn Lập chăm chú ngưng thị tầng mây sấm sét màu tím vàng đang cuồn cuộn nơi chân trời, Thanh Trúc Phong Vân Kiếm trong tay phát ra từng hồi tiếng ngân thanh thúy. Hắn biết rõ độ kiếp lần này tuyệt đối không phải trò đùa.',
      words: [
        { word: '青竹蜂云剑', pinyin: 'qīng zhú fēng yún jiàn', hv: 'Thanh Trúc Phong Vân Kiếm', note: 'Bản mệnh pháp bảo của Hàn Lập, luyện chế từ Kim Lôi Trúc.' },
        { word: '渡劫', pinyin: 'dù jié', hv: 'Độ Kiếp', note: 'Trải qua thiên kiếp sấm sét để tấn thăng cảnh giới tu tiên cao hơn.' },
        { word: '凝视', pinyin: 'níng shì', hv: 'Ngưng Thị', note: 'Chăm chú nhìn, quan sát một cách tập trung cao độ.' }
      ],
      aiSuggestion: '💡 <b>Gợi ý AI (Giữ vững sắc thái Kiếm hiệp):</b> Có thể chuốt cụm "ngưng thị" thành "chăm chú dõi theo" để câu văn tự nhiên và thoát ý hơn mà vẫn giữ nghiêm phong vị tu tiên.'
    },
    'english': {
      source: 'The protagonist stood before the ancient obsidian gate, where whispers of the forgotten empire still echoed in the wind.',
      target: 'Nhân vật chính đứng trước cánh cổng hắc diện thạch cổ kính, nơi những lời thì thầm của đế chế đã bị lãng quên vẫn còn văng vẳng trong gió.',
      words: [
        { word: 'obsidian gate', pinyin: '[noun]', hv: 'Hắc diện thạch / Đá vỏ chai', note: 'Đá thủy tinh núi lửa đen bóng. Trong bối cảnh kỳ ảo thường dịch là Hắc Diện Thạch.' },
        { word: 'whispers', pinyin: '[noun]', hv: 'Tiếng thì thầm', note: 'Âm thanh nhỏ nhẹ, bí ẩn, vang vọng trong không gian tĩnh mịch.' }
      ],
      aiSuggestion: '💡 <b>Gợi ý AI (Văn phong kỳ ảo phương Tây):</b> Dùng "cánh cổng hắc diện thạch" tạo cảm giác trang nghiêm và huyền bí hơn nhiều so với "cổng đá obsidian thô".'
    }
  };

  function loadPlayground(key) {
    const data = playgroundPresets[key];
    if (!data) return;

    pgSource.innerHTML = data.source;
    pgTarget.innerHTML = data.target;

    let inspectorHTML = `
      <div style="font-size:0.875rem; margin-bottom: 0.85rem; color: var(--text-dim); text-transform: uppercase; font-weight:700;">
        🔍 Từ vựng & Thuật ngữ nhận diện trong câu
      </div>
      <div style="display: flex; flex-direction: column; gap: 0.65rem;">
    `;

    data.words.forEach(item => {
      inspectorHTML += `
        <div style="background: var(--bg-primary); border: 1px solid var(--border-subtle); border-radius: 6px; padding: 0.65rem;">
          <div style="display:flex; justify-content:space-between; align-items:baseline; margin-bottom:0.2rem;">
            <strong style="color:var(--text-main); font-size:0.95rem;">${item.word}</strong>
            <span style="color:var(--teal-primary); font-size:0.8rem; font-family:monospace;">${item.pinyin}</span>
          </div>
          <div style="color:var(--amber-accent); font-weight:600; font-size:0.85rem; margin-bottom:0.25rem;">Hán Việt: ${item.hv}</div>
          <div style="color:var(--text-muted); font-size:0.813rem;">${item.note}</div>
        </div>
      `;
    });

    inspectorHTML += `</div><div style="margin-top:1rem; padding:0.75rem; border-radius:6px; background:rgba(127,179,186,0.1); border:1px solid rgba(127,179,186,0.2); font-size:0.85rem; color:var(--text-main);">${data.aiSuggestion}</div>`;

    pgInspector.innerHTML = inspectorHTML;
  }

  pgTabs.forEach(tab => {
    tab.addEventListener('click', () => {
      pgTabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      const preset = tab.getAttribute('data-preset');
      loadPlayground(preset);
    });
  });

  // Load initial preset
  if (pgSource) {
    loadPlayground('xianxia');
  }

  // 5. FAQ Accordion
  const faqItems = document.querySelectorAll('.faq-item');
  faqItems.forEach(item => {
    const questionBtn = item.querySelector('.faq-question');
    if (questionBtn) {
      questionBtn.addEventListener('click', () => {
        const isOpen = item.classList.contains('open');
        faqItems.forEach(i => i.classList.remove('open'));
        if (!isOpen) {
          item.classList.add('open');
        }
      });
    }
  });
});
