# -*- coding: utf-8 -*-
"""Chapters 1 & 2: Overview and 4 Interactive Simulators."""

def get_chapters_1_and_2():
    return """
    <!-- Page Header -->
    <div style="margin-bottom: 32px;">
      <h1 class="cds--page-title">Sổ tay Hướng dẫn Toàn diện AuraTranslate</h1>
      <p class="cds--page-subtitle">
        Tài liệu đặc tả và hướng dẫn sử dụng chuyên sâu cho nền tảng dịch thuật ngoại tuyến AuraTranslate. 
        Được thiết kế theo tiêu chuẩn <strong>IBM Carbon Design System v11</strong>, hướng đến dịch giả chuyên nghiệp, biên tập viên văn học và nhà nghiên cứu ngôn ngữ.
      </p>
      <div style="display: flex; gap: 8px; flex-wrap: wrap;">
        <span class="cds--tag cds--tag--blue">Bản phát hành chính thức v2.0</span>
        <span class="cds--tag cds--tag--green">100% Ngoại Tuyến (Offline-first)</span>
        <span class="cds--tag cds--tag--purple">3 Chế độ Thao tác Độc lập</span>
        <span class="cds--tag cds--tag--cyan">BYOK & Local AI (Ollama)</span>
      </div>
    </div>

    <!-- CHƯƠNG 1 -->
    <section id="overview" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>01</span>
        <span>Tổng quan AuraTranslate & Triết lý Thiết kế</span>
      </h2>
      
      <p>
        <strong>AuraTranslate</strong> là không gian làm việc dịch thuật và đối chiếu văn bản cao cấp, chạy độc lập trên máy tính cá nhân. 
        Khác biệt hoàn toàn với các công cụ dịch web trực tuyến hay các phần mềm văn phòng thông thường, AuraTranslate được kiến tạo xung quanh nhu cầu cốt lõi của người dịch sách, truyện dài kỳ và tài liệu học thuật: <em>tốc độ phản hồi tức thì, bảo mật tuyệt đối, và khả năng kiểm soát tỉ mỉ từng câu dịch</em>.
      </p>

      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">100% Offline-First</span>
            <span class="cds--tag cds--tag--green">An Toàn</span>
          </div>
          <p>
            Toàn bộ từ điển tra cứu, bộ nhớ dịch (TM), thuật ngữ (Glossary) và bản thảo đều được lưu trữ trực tiếp trên ổ cứng trong định dạng tệp <code>.atproj</code> (cơ sở dữ liệu SQLite tối ưu). Bạn có thể làm việc mượt mà trên máy bay, nơi không có mạng Internet mà không mất bất kỳ tính năng cốt lõi nào.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">3 Chế Độ Thao Tác Độc Lập</span>
            <span class="cds--tag cds--tag--blue">Tiện Lợi</span>
          </div>
          <p>
            Thanh tiêu đề của ứng dụng phân chia rõ ràng thành 3 không gian làm việc chuyên biệt: 
            <strong>Thư viện</strong> (<kbd>Mod+1</kbd>), <strong>Bàn dịch Workspace</strong> (<kbd>Mod+2</kbd>), và <strong>Chế độ đọc sách</strong> (<kbd>Mod+3</kbd>). 
            Không gian soát lỗi (Review Mode) được tích hợp ngay tại bàn dịch khi cần đối chiếu Word.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Thiết Kế Chuẩn IBM Carbon v11</span>
            <span class="cds--tag cds--tag--purple">Công Thái Học</span>
          </div>
          <p>
            Giao diện được nghiên cứu tỉ mỉ về khoảng cách thị giác và độ tương phản quang học. Tông màu xám Carbon trung tính bảo vệ mắt khi dịch liên tục nhiều giờ liền, hỗ trợ đầy đủ 2 chủ đề Giao diện Tối (Dark) và Giao diện Sáng (Light).
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Quyền Riêng Tư AI (BYOK)</span>
            <span class="cds--tag cds--tag--cyan">Bảo Mật</span>
          </div>
          <p>
            Người dùng tự quản lý khóa API riêng (Bring Your Own Key) với Anthropic Claude, OpenAI, Google Gemini, OpenRouter hoặc kết nối trực tiếp với mô hình AI chạy cục bộ trên máy như Ollama và LM Studio. Bản dịch của bạn không bao giờ bị sử dụng để đào tạo AI.
          </p>
        </div>
      </div>

      <div class="cds--inline-notification">
        <div class="cds--inline-notification__content">
          <div class="cds--inline-notification__title">Quy ước phím bấm hệ thống:</div>
          <div>Trong toàn bộ tài liệu này, ký hiệu <kbd>Mod</kbd> đại diện cho phím <strong>Command (⌘)</strong> trên máy Mac và phím <strong>Control (Ctrl)</strong> trên máy Windows / Linux.</div>
        </div>
      </div>
    </section>

    <!-- CHƯƠNG 2: BỘ MÔ PHỎNG TƯƠNG TÁC -->
    <section id="simulators" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>02</span>
        <span>Mô hình Giao diện Trực quan & Bộ Mô phỏng Tương tác</span>
      </h2>
      <p>
        Dưới đây là 4 bộ mô phỏng giao diện động thực tế của AuraTranslate. Bạn có thể tương tác trực tiếp với các nút bấm, chuyển đổi chế độ làm việc và xem cách phần mềm vận hành trực quan.
      </p>

      <!-- DEMO 1: WORKSPACE SIMULATOR -->
      <div class="aura-simulator">
        <div class="aura-simulator-header">
          <div class="aura-simulator-title">
            <div class="aura-simulator-dots">
              <span class="aura-simulator-dot red"></span>
              <span class="aura-simulator-dot yellow"></span>
              <span class="aura-simulator-dot green"></span>
            </div>
            <span>Mô phỏng 1: Bàn Dịch Workspace (Không gian làm việc chính)</span>
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="toggleSimLayout()">
              <span id="simLayoutText">⊞ Đổi sang Bố cục Trên-Dưới</span>
            </button>
            <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="toggleSimContextMenu()">
              <span>🖱️ Thử Menu Chuột Phải</span>
            </button>
          </div>
        </div>

        <!-- Titlebar Mock -->
        <div class="aura-titlebar-mock">
          <div style="display: flex; align-items: center; gap: 12px;">
            <span style="font-weight: 600; font-size: 13px;">AuraTranslate</span>
            <div class="aura-modes-segmented">
              <div class="aura-mode-tab" onclick="switchSimMode('library')">📚 Thư viện (Mod+1)</div>
              <div class="aura-mode-tab active" onclick="switchSimMode('workspace')">✍️ Bàn dịch (Mod+2)</div>
              <div class="aura-mode-tab" onclick="switchSimMode('reading')">📖 Đọc sách (Mod+3)</div>
            </div>
            <button class="cds--btn cds--btn--tertiary cds--btn--sm" style="height: 24px;" onclick="switchSimMode('review')">
              <span>🔍 Soát Lỗi Review</span>
            </button>
          </div>

          <div style="display: flex; align-items: center; gap: 8px; font-size: 12px; color: var(--cds-text-secondary);">
            <span class="cds--tag cds--tag--green">● Tự động lưu: Đã lưu</span>
            <span>Chương 01: Khởi đầu (12/120 câu)</span>
          </div>
        </div>

        <!-- Workspace Body -->
        <div class="aura-workspace-body" id="simWorkspaceBody">
          <!-- Left: Translation Grid -->
          <div class="aura-grid-pane">
            <!-- Row 1: Completed sentence -->
            <div class="aura-grid-row">
              <div class="aura-row-meta">
                <span>#001 · Đã duyệt</span>
                <span class="cds--tag cds--tag--green">100% Hoàn thành</span>
              </div>
              <div class="aura-source-group">
                <div class="aura-source-pinyin">tiān qì qíng lǎng, wàn lǐ wú yún.</div>
                <div class="aura-source-text">天气晴朗，万里无云。</div>
                <div class="aura-source-hanviet">Thiên khí tình lãng, vạn lý vô vân.</div>
              </div>
              <textarea class="aura-target-editor" readonly>Trời trong nắng ấm, muôn dặm không một gợn mây.</textarea>
            </div>

            <!-- Row 2: Active translating sentence -->
            <div class="aura-grid-row active" style="position: relative;">
              <!-- Simulated context menu popover -->
              <div class="aura-context-menu" id="simContextMenu" style="display: none; top: 30px; left: 180px;">
                <div class="aura-context-menu-item" onclick="alert('Đã tách câu thành công tại vị trí con trỏ!')">
                  <span>✂️ Tách câu (Split)</span>
                  <kbd>Mod+K</kbd>
                </div>
                <div class="aura-context-menu-item" onclick="alert('Đã gộp với câu kế tiếp!')">
                  <span>🔗 Gộp câu (Merge)</span>
                  <kbd>Mod+J</kbd>
                </div>
                <div class="aura-context-menu-divider"></div>
                <div class="aura-context-menu-item" onclick="alert('Đã đánh dấu cắt bỏ câu!')">
                  <span>🚫 Cắt bỏ câu (Omit)</span>
                  <kbd>Mod+Del</kbd>
                </div>
                <div class="aura-context-menu-item" onclick="openModal('modalSegmentHistory')">
                  <span>🕒 Lịch sử câu (History)</span>
                  <kbd>Mod+H</kbd>
                </div>
              </div>

              <div class="aura-row-meta">
                <span style="color: var(--cds-link-primary); font-weight: 600;">#002 · Đang dịch</span>
                <span class="cds--tag cds--tag--blue">Con trỏ hiện tại</span>
              </div>

              <!-- Strip 1: Glossary hint -->
              <div class="aura-strip glossary">
                <span>📖 <strong>Thuật ngữ:</strong> <em>“Thanh Vân Môn”</em> (Qingyun Sect)</span>
                <button class="cds--btn cds--btn--tertiary cds--btn--sm" style="height: 20px; padding: 0 6px;" onclick="applyGlossaryTerm()">Áp dụng</button>
              </div>

              <!-- Strip 2: Fuzzy TM hint -->
              <div class="aura-strip tm">
                <span>⚡ <strong>Bộ nhớ dịch (92% khớp):</strong> <em>“Hắn chậm rãi ngẩng đầu nhìn đỉnh núi.”</em></span>
                <button class="cds--btn cds--btn--tertiary cds--btn--sm" style="height: 20px; padding: 0 6px;" onclick="applyTmSuggestion()">Chép vào</button>
              </div>

              <div class="aura-source-group">
                <div class="aura-source-pinyin">tā huǎn huǎn tái tóu, wàng xiàng qīng yún mén de shān diān.</div>
                <div class="aura-source-text">他缓缓抬头，望向青云门的山巅。</div>
                <div class="aura-source-hanviet">Tha hoãn hoãn ngẩng đầu, vọng hướng Thanh Vân Môn đích sơn điên.</div>
              </div>
              <textarea class="aura-target-editor" id="simTargetInput" placeholder="Nhập bản dịch tiếng Việt tại đây... (Nhấn Ctrl+Enter để xác nhận)">Hắn chậm rãi ngẩng đầu, nhìn về phía đỉnh núi của Thanh Vân Môn.</textarea>

              <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 8px;">
                <span style="font-size: 11px; color: var(--cds-text-placeholder);">Phím tắt: <kbd>Ctrl+Enter</kbd> Lưu & sang câu tiếp | Chuột phải để mở menu thao tác</span>
                <button class="cds--btn cds--btn--primary cds--btn--sm" onclick="alert('Đã lưu câu #002 vào bộ nhớ và tự động nhảy sang câu #003!')">Xác nhận câu (Ctrl+Enter)</button>
              </div>
            </div>

            <!-- Row 3: Untranslated sentence -->
            <div class="aura-grid-row">
              <div class="aura-row-meta">
                <span>#003 · Chưa dịch</span>
                <span class="cds--tag cds--tag--yellow">Chờ dịch</span>
              </div>
              <div class="aura-source-group">
                <div class="aura-source-pinyin">fēng shēng xiāo xiāo, dài zhe yī sī liáng yì.</div>
                <div class="aura-source-text">风声萧萧，带着一丝凉意。</div>
                <div class="aura-source-hanviet">Phong thanh tiêu tiêu, đái trước nhất ti lương ý.</div>
              </div>
              <textarea class="aura-target-editor" placeholder="Chưa dịch..."></textarea>
            </div>
          </div>

          <!-- Right: Side Panel (Lookup & AI) -->
          <div class="aura-side-pane">
            <!-- Top half: Lookup Panel -->
            <div class="aura-side-pane-top">
              <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                <div style="font-weight: 600; font-size: 13px; display: flex; gap: 8px;">
                  <span style="color: var(--cds-link-primary); border-bottom: 2px solid var(--cds-link-primary); padding-bottom: 2px; cursor: pointer;">Từ điển</span>
                  <span style="color: var(--cds-text-secondary); cursor: pointer;" onclick="alert('Chuyển sang tab Concordance (Tra chéo toàn bộ bản dịch cũ)!')">Bộ nhớ (TM)</span>
                  <span style="color: var(--cds-text-secondary); cursor: pointer;" onclick="alert('Chuyển sang tab Lịch sử tra cứu có nút Ghim 📌!')">Lịch sử</span>
                </div>
                <button class="cds--btn cds--btn--tertiary cds--btn--sm" style="height: 22px; padding: 0 6px;" onclick="alert('Đã ghim kết quả từ điển này 📌!')" title="Ghim từ điển">📌 Ghim</button>
              </div>

              <div style="background: var(--cds-layer-02); padding: 10px; border-radius: 2px; font-size: 12.5px;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                  <strong style="color: var(--cds-text-primary); font-size: 14px;">山巅 (shān diān)</strong>
                  <span class="cds--tag cds--tag--purple" style="font-size: 10px; padding: 2px 6px;">Thiều Chửu</span>
                </div>
                <p style="margin-bottom: 4px; font-size: 12px; color: var(--cds-text-secondary);"><strong>Hán Việt:</strong> Sơn điên</p>
                <p style="margin-bottom: 0; font-size: 12px; color: var(--cds-text-secondary);"><strong>Định nghĩa:</strong> Đỉnh núi, chỏm núi, ngọn núi cao nhất. (Đồng nghĩa: Sơn đỉnh, phong đỉnh).</p>
              </div>
            </div>

            <!-- Bottom half: AI Translation Panel -->
            <div class="aura-side-pane-bottom">
              <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                <span style="font-weight: 600; font-size: 13px;">Trợ Lý AI (Claude 3.5 Sonnet)</span>
                <div style="display: flex; gap: 4px;">
                  <button class="cds--btn cds--btn--tertiary cds--btn--sm" style="height: 22px; padding: 0 6px;" onclick="openModal('modalPromptInspector')">🔍 Soi Prompt</button>
                  <button class="cds--btn cds--btn--secondary cds--btn--sm" style="height: 22px; padding: 0 6px;" onclick="alert('Đã gửi yêu cầu dịch lại câu!')">↻ Dịch lại</button>
                </div>
              </div>

              <div style="background: var(--cds-layer-02); padding: 10px; border-radius: 2px; font-size: 12.5px; border-left: 3px solid var(--cds-support-cyan); margin-bottom: 8px;">
                <div style="font-size: 11px; color: var(--cds-text-placeholder); margin-bottom: 4px;">Gợi ý dịch theo thể loại: Tiên hiệp / Tu chân</div>
                <div style="color: var(--cds-text-primary); line-height: 1.4;">"Hắn chậm rãi ngẩng đầu lên, phóng tầm mắt nhìn về phía đỉnh núi mù sương của Thanh Vân Môn."</div>
              </div>

              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-size: 11px; color: var(--cds-text-placeholder);">Token: 142 prompt / 28 comp</span>
                <button class="cds--btn cds--btn--primary cds--btn--sm" onclick="document.getElementById('simTargetInput').value = 'Hắn chậm rãi ngẩng đầu lên, phóng tầm mắt nhìn về phía đỉnh núi mù sương của Thanh Vân Môn.'">Áp dụng gợi ý</button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- DEMO 2: THƯ VIỆN SIMULATOR -->
      <div class="aura-simulator">
        <div class="aura-simulator-header">
          <div class="aura-simulator-title">
            <div class="aura-simulator-dots">
              <span class="aura-simulator-dot red"></span>
              <span class="aura-simulator-dot yellow"></span>
              <span class="aura-simulator-dot green"></span>
            </div>
            <span>Mô phỏng 2: Không gian Thư viện (Library Mode — Mod+1)</span>
          </div>
          <div>
            <button class="cds--btn cds--btn--primary cds--btn--sm" onclick="alert('Mở hộp thoại chọn file nhập (TXT, EPUB, Word, Web URL)!')">
              <span>+ Nhập Tác Phẩm Mới</span>
            </button>
          </div>
        </div>

        <div style="padding: 24px; background: var(--cds-background);">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
            <div style="width: 320px;">
              <input type="text" class="cds--header__search input" placeholder="Lọc tác phẩm theo tên sách, tác giả..." style="width: 100%; height: 32px; background: var(--cds-layer-01); border: 1px solid var(--cds-border-subtle-01); color: #fff; padding: 0 10px;">
            </div>
            <div style="display: flex; gap: 8px;">
              <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="alert('Quét tìm các tệp .atproj bị mất liên kết thư mục!')">🔍 Quét Tác Phẩm Mồ Côi</button>
              <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="alert('Đã dựng lại toàn bộ chỉ mục tìm kiếm FTS!')">⚡ Dựng Lại Chỉ Mục</button>
            </div>
          </div>

          <div class="cds--grid-cards">
            <!-- Book Card 1 -->
            <div class="cds--card cds--tile--clickable" onclick="switchSimMode('workspace')">
              <div class="cds--card-header">
                <div>
                  <h4 style="font-size: 15px; font-weight: 600; color: var(--cds-text-primary);">Tru Tiên (Chỉnh sửa đặc biệt)</h4>
                  <div style="font-size: 12px; color: var(--cds-text-placeholder);">Tác giả: Tiêu Đỉnh · 280 Chương</div>
                </div>
                <span class="cds--tag cds--tag--blue">Đang dịch</span>
              </div>
              <p style="font-size: 12.5px; margin-bottom: 12px;">Đã dịch: 1,420 / 3,500 câu (Tiến độ: 40.5%)</p>
              <div style="width: 100%; height: 6px; background: var(--cds-layer-02); border-radius: 3px; overflow: hidden; margin-bottom: 12px;">
                <div style="width: 40.5%; height: 100%; background: var(--cds-button-primary);"></div>
              </div>
              <div style="display: flex; justify-content: space-between; font-size: 11.5px; color: var(--cds-text-helper);">
                <span>Cập nhật: 2 giờ trước</span>
                <span style="color: var(--cds-link-primary);">Nhấp để mở Bàn dịch →</span>
              </div>
            </div>

            <!-- Book Card 2 -->
            <div class="cds--card cds--tile--clickable">
              <div class="cds--card-header">
                <div>
                  <h4 style="font-size: 15px; font-weight: 600; color: var(--cds-text-primary);">Phàm Nhân Tu Tiên Chi Tiên Giới Thiên</h4>
                  <div style="font-size: 12px; color: var(--cds-text-placeholder);">Tác giả: Vong Ngữ · 1,200 Chương</div>
                </div>
                <span class="cds--tag cds--tag--green">Hoàn thành</span>
              </div>
              <p style="font-size: 12.5px; margin-bottom: 12px;">Đã dịch: 15,200 / 15,200 câu (Tiến độ: 100%)</p>
              <div style="width: 100%; height: 6px; background: var(--cds-layer-02); border-radius: 3px; overflow: hidden; margin-bottom: 12px;">
                <div style="width: 100%; height: 100%; background: var(--cds-support-success);"></div>
              </div>
              <div style="display: flex; justify-content: space-between; font-size: 11.5px; color: var(--cds-text-helper);">
                <span>Xuất bản: EPUB, Word, TXT</span>
                <span style="color: var(--cds-link-primary);">Xem lại →</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- DEMO 3: REVIEW MODE (WORD DIFF) SIMULATOR -->
      <div class="aura-simulator">
        <div class="aura-simulator-header">
          <div class="aura-simulator-title">
            <div class="aura-simulator-dots">
              <span class="aura-simulator-dot red"></span>
              <span class="aura-simulator-dot yellow"></span>
              <span class="aura-simulator-dot green"></span>
            </div>
            <span>Mô phỏng 3: Khung Soát Lỗi Review (So sánh Đối chiếu Word .docx)</span>
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="alert('Đã chấp nhận toàn bộ các chỉnh sửa của Biên tập viên!')">
              <span>✓ Nhận Tất Cả Thay Đổi</span>
            </button>
            <button class="cds--btn cds--btn--secondary cds--btn--sm" onclick="switchSimMode('workspace')">
              <span>✕ Đóng Soát Lỗi</span>
            </button>
          </div>
        </div>

        <div style="padding: 16px; background: var(--cds-background);">
          <div style="margin-bottom: 12px; font-size: 13px; color: var(--cds-text-secondary);">
            Đang so sánh bản dịch hiện tại với tệp Word biên tập: <code>Ban_Thao_Bien_Tap_Chuong_1.docx</code>
          </div>

          <div class="diff-container">
            <div class="diff-col original">
              <div style="font-size: 12px; font-weight: 600; color: var(--cds-text-placeholder); margin-bottom: 8px;">BẢN DỊCH HIỆN TẠI (TRÊN MÁY BẠN)</div>
              <div style="margin-bottom: 16px; padding-bottom: 12px; border-bottom: 1px solid var(--cds-border-subtle-01);">
                <div style="font-size: 11px; color: var(--cds-text-placeholder); margin-bottom: 2px;">#001</div>
                <div>Trời <span class="diff-del">trong nắng ấm</span>, muôn dặm không một gợn mây.</div>
              </div>
              <div>
                <div style="font-size: 11px; color: var(--cds-text-placeholder); margin-bottom: 2px;">#002</div>
                <div>Hắn <span class="diff-del">chậm rãi</span> ngẩng đầu, nhìn về phía đỉnh núi của Thanh Vân Môn.</div>
              </div>
            </div>

            <div class="diff-col">
              <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
                <span style="font-size: 12px; font-weight: 600; color: var(--cds-support-cyan);">BẢN SỬA ĐỔI TỪ FILE WORD BIÊN TẬP</span>
                <span class="cds--tag cds--tag--purple">2 Thay đổi</span>
              </div>
              <div style="margin-bottom: 16px; padding-bottom: 12px; border-bottom: 1px solid var(--cds-border-subtle-01); display: flex; justify-content: space-between; align-items: center;">
                <div>
                  <div style="font-size: 11px; color: var(--cds-text-placeholder); margin-bottom: 2px;">#001</div>
                  <div>Trời <span class="diff-add">quang mây tạnh</span>, muôn dặm không một gợn mây.</div>
                </div>
                <div style="display: flex; gap: 4px;">
                  <button class="cds--btn cds--btn--primary cds--btn--sm" style="height: 24px; padding: 0 8px;" onclick="alert('Đã nhận thay đổi câu #001!')">Nhận</button>
                  <button class="cds--btn cds--btn--tertiary cds--btn--sm" style="height: 24px; padding: 0 8px;" onclick="alert('Đã bỏ qua thay đổi câu #001!')">Bỏ</button>
                </div>
              </div>

              <div style="display: flex; justify-content: space-between; align-items: center;">
                <div>
                  <div style="font-size: 11px; color: var(--cds-text-placeholder); margin-bottom: 2px;">#002</div>
                  <div>Hắn <span class="diff-add">từ từ</span> ngẩng đầu, nhìn về phía đỉnh núi của Thanh Vân Môn.</div>
                </div>
                <div style="display: flex; gap: 4px;">
                  <button class="cds--btn cds--btn--primary cds--btn--sm" style="height: 24px; padding: 0 8px;" onclick="alert('Đã nhận thay đổi câu #002!')">Nhận</button>
                  <button class="cds--btn cds--btn--tertiary cds--btn--sm" style="height: 24px; padding: 0 8px;" onclick="alert('Đã bỏ qua thay đổi câu #002!')">Bỏ</button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- DEMO 4: READING MODE SIMULATOR -->
      <div class="aura-simulator">
        <div class="aura-simulator-header">
          <div class="aura-simulator-title">
            <div class="aura-simulator-dots">
              <span class="aura-simulator-dot red"></span>
              <span class="aura-simulator-dot yellow"></span>
              <span class="aura-simulator-dot green"></span>
            </div>
            <span>Mô phỏng 4: Chế Độ Đọc Sách Thư Giãn (Reading Mode — Mod+3)</span>
          </div>
          <div style="display: flex; gap: 8px;">
            <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="alert('Đã đánh dấu cắm cờ câu này để lát nữa quay lại sửa!')">
              <span>⚑ Cắm cờ câu nghi vấn</span>
            </button>
          </div>
        </div>

        <div style="padding: 30px; background: #121212;">
          <div class="reading-mode-preview">
            <div style="text-align: center; margin-bottom: 30px;">
              <h2 style="font-family: var(--font-serif); font-size: 22px; font-weight: 600; margin-bottom: 8px;">Chương 01: Khởi đầu</h2>
              <div style="font-size: 13px; color: #888; font-style: italic;">Tác phẩm: Tru Tiên · Bản dịch chính thức</div>
            </div>

            <p style="text-indent: 2em; margin-bottom: 16px;">
              Trời quang mây tạnh, muôn dặm không một gợn mây. Ánh mặt trời buổi sớm rọi xuống rặng núi điệp trùng như dát một lớp hoàng kim rực rỡ.
            </p>

            <div class="reading-frontier-line">
              <span class="reading-frontier-badge">VỊ TRÍ ĐỌC DỞ (FRONTIER)</span>
            </div>

            <p style="text-indent: 2em; margin-bottom: 16px;">
              Hắn từ từ ngẩng đầu, nhìn về phía đỉnh núi cao vút của Thanh Vân Môn. Tiếng gió rít qua khe đá mang theo một luồng hàn khí thấu xương, dường như báo trước những biến cố kinh thiên động địa sắp sửa ập đến vùng đất thanh bình này...
            </p>

            <div style="display: flex; justify-content: center; margin-top: 32px;">
              <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="switchSimMode('workspace')">
                <span>← Trở lại Bàn dịch sửa chữa (Mod+2)</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </section>
"""
