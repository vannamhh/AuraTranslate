# -*- coding: utf-8 -*-
"""Chapters 12, 13, 14, Modals, Footer, and Embedded JavaScript."""

def get_chapters_12_13_14():
    return """
    <!-- CHƯƠNG 12 -->
    <section id="vietnamese-rules" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>12</span>
        <span>Quy Ước Chữ Việt & Trợ Lý Soát Lỗi Tự Động</span>
      </h2>
      <p>
        Một tác phẩm hay không chỉ cần dịch đúng nghĩa mà phải chuẩn mực về tiếng Việt. 
        AuraTranslate trang bị hệ thống kiểm tra quy ước ngôn ngữ chạy hoàn toàn tự động <strong>với chi phí 0đ</strong> và không cần gọi tới AI bên ngoài.
      </p>

      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Bộ Soát Lỗi Chữ Việt (Chi Phí 0đ)</span>
            <span class="cds--tag cds--tag--green">Thuật Toán Cục Bộ</span>
          </div>
          <p>
            Tự động dò tìm các lỗi chính tả phổ biến trong tiếng Việt (hỏi/ngã, s/x, tr/ch, d/gi), lỗi lặp từ không chủ đích, và các cấu trúc câu dịch thô bị lai tạp ngữ pháp tiếng nước ngoài (như lạm dụng từ "của", dịch thụ động vụng về).
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Chuẩn Hóa Vị Trí Dấu Thanh</span>
            <span class="cds--tag cds--tag--blue">Xuất Bản</span>
          </div>
          <p>
            Cho phép tùy chọn đồng bộ cách bỏ dấu thanh điệu theo chuẩn truyền thống (hòa, thủy) hoặc chuẩn hiện đại mới (hoà, thuỷ) trên toàn bộ hàng nghìn trang sách chỉ bằng một cú nhấp chuột.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Trợ Lý Soát Lỗi AI (AI Proofreader)</span>
            <span class="cds--tag cds--tag--purple">Trau Chuốt Văn Phong</span>
          </div>
          <p>
            Khi cần trau chuốt các đoạn văn cao trào hoặc thơ ca, hãy kích hoạt Trợ lý Soát lỗi AI. Công cụ này sẽ đọc lại câu dịch của bạn, đề xuất các từ ngữ đắt giá hơn và làm câu văn mềm mại, đậm chất văn học.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Tùy Chọn Phạm Vi Soát Lỗi</span>
            <span class="cds--tag cds--tag--cyan">Linh Hoạt</span>
          </div>
          <p>
            Bạn có thể chọn soát lỗi: <em>Chỉ riêng câu hiện tại</em>, <em>Cụm đoạn văn 5 câu xung quanh</em>, hoặc <em>Toàn bộ chương</em> tùy theo nhu cầu và ngân sách thời gian của mình.
          </p>
        </div>
      </div>
    </section>

    <!-- CHƯƠNG 13 -->
    <section id="word-review-reading" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>13</span>
        <span>Quy Trình Soát Lỗi Qua Word & Chế Độ Đọc Sách</span>
      </h2>
      <p>
        Trong thực tế xuất bản, dịch giả thường phải gửi bản thảo cho Biên tập viên hoặc người đọc duyệt ngoài chỉnh sửa qua Microsoft Word. 
        AuraTranslate hỗ trợ quy trình khép kín hoàn hảo mà không làm đảo lộn cấu trúc câu dịch.
      </p>

      <div class="cds--steps">
        <div class="cds--step-item">
          <div class="cds--step-num">1</div>
          <div class="cds--step-content">
            <h4>Xuất Bản Thảo Sang Word Dạng Bảng 2 Cột</h4>
            <p>AuraTranslate tạo ra tệp Word <code>.docx</code> chuẩn mực với cột trái là nguyên tác và cột phải là bản dịch tiếng Việt, giúp biên tập viên dễ dàng đối chiếu.</p>
          </div>
        </div>
        <div class="cds--step-item">
          <div class="cds--step-num">2</div>
          <div class="cds--step-content">
            <h4>Biên Tập Viên Chỉnh Sửa Tự Do</h4>
            <p>Biên tập viên mở file bằng Word, sửa câu từ, đánh dấu thay đổi hoặc viết lại đoạn văn bình thường.</p>
          </div>
        </div>
        <div class="cds--step-item">
          <div class="cds--step-num">3</div>
          <div class="cds--step-content">
            <h4>Nhập Lại File Word Vào Chế Độ Soát Lỗi (Review Mode)</h4>
            <p>Trở lại AuraTranslate, nhấn nút <strong>Soát Lỗi Review</strong> trên thanh tiêu đề và chọn file Word đã sửa. Hệ thống tự động so khớp từng câu và hiển thị màn hình đối chiếu Diff hai bên.</p>
          </div>
        </div>
        <div class="cds--step-item">
          <div class="cds--step-num">4</div>
          <div class="cds--step-content">
            <h4>Duyệt Từng Thay Đổi: Bấm Nhận (Accept) Hoặc Bỏ Qua (Reject)</h4>
            <p>Bạn xem rõ từng chữ bị xóa (gạch đỏ) và chữ thêm mới (tô xanh). Bạn có quyền bấm <strong>Nhận</strong> câu hợp lý và <strong>Bỏ qua</strong> câu bạn thấy chưa ưng ý. Bản dịch trong máy sẽ được cập nhật chuẩn xác từng câu.</p>
          </div>
        </div>
      </div>

      <h3 class="cds--subsection-title">Chế Độ Đọc Sách Thư Giãn (Reading Mode — Mod+3)</h3>
      <p>
        Sau khi hoàn thành bản dịch, việc ngồi đọc lại toàn bộ tác phẩm như một cuốn sách thực thụ là cách tốt nhất để cảm nhận nhịp điệu của câu chuyện.
      </p>
      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Phông Chữ Serif Trang Nhã</span>
            <span class="cds--tag cds--tag--purple">Chuẩn Xuất Bản</span>
          </div>
          <p>
            Màn hình ẩn đi toàn bộ các nút bấm kỹ thuật, chuyển sang phông chữ IBM Plex Serif với căn lề, khoảng cách dòng và độ rộng trang tối ưu cho mắt.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Vạch Ghi Nhớ Vị Trí Đọc (Frontier Line)</span>
            <span class="cds--tag cds--tag--cyan">Tiện Lợi</span>
          </div>
          <p>
            Vạch đánh dấu màu xanh hiển thị rõ câu cuối cùng bạn vừa đọc trước khi tạm nghỉ, giúp bạn không bao giờ bị lạc khi quay lại đọc tiếp hôm sau.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Cắm Cờ Nghi Vấn (Flag ⚑)</span>
            <span class="cds--tag cds--tag--yellow">Sửa Nhanh</span>
          </div>
          <p>
            Đang đọc mượt mà bỗng gặp một câu dịch gượng gạo? Bấm phím <kbd>F</kbd> để cắm cờ và nhập nhanh ghi chú. Bạn có thể tiếp tục đọc hết chương rồi bấm <kbd>Mod+2</kbd> quay lại Bàn dịch để nhảy thẳng tới các câu có cờ để tinh chỉnh.
          </p>
        </div>
      </div>
    </section>

    <!-- CHƯƠNG 14 -->
    <section id="settings-shortcuts" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>14</span>
        <span>Bảng Cài Đặt Toàn Diện & Tra Cứu Phím Tắt Toàn Năng</span>
      </h2>
      <p>
        AuraTranslate tôn trọng thói quen của từng dịch giả, cho phép cấu hình chi tiết từ nhà cung cấp AI đến từng tổ hợp phím bấm.
      </p>

      <h3 class="cds--subsection-title">Bộ Tùy Biến Phím Tắt Trực Quan (Auto-Capture)</h3>
      <p>
        Trong bảng Cài đặt phím tắt (<kbd>Mod+,</kbd>), bạn không cần phải gõ tên phím bằng chữ. Chỉ cần nhấp vào chức năng muốn đổi và <strong>nhấn trực tiếp tổ hợp phím mong muốn trên bàn phím</strong>: Hệ thống sẽ tự động bắt phím (Auto-capture) và cảnh báo ngay lập tức nếu tổ hợp phím đó bị xung đột với một lệnh khác.
      </p>

      <h3 class="cds--subsection-title" id="shortcuts-table">Bảng Tra Cứu Phím Tắt Nhanh (Thao Tác Thường Dùng)</h3>
      <div class="cds--data-table-container">
        <table class="cds--data-table">
          <thead>
            <tr>
              <th>Nhóm Chức Năng</th>
              <th>Thao Tác</th>
              <th>Phím Tắt</th>
              <th>Công Dụng</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><strong>Chuyển Chế Độ</strong></td>
              <td>Chuyển sang Thư viện</td>
              <td><kbd>Mod+1</kbd></td>
              <td>Mở danh sách tác phẩm và các dự án dịch</td>
            </tr>
            <tr>
              <td><strong>Chuyển Chế Độ</strong></td>
              <td>Chuyển sang Bàn dịch</td>
              <td><kbd>Mod+2</kbd></td>
              <td>Vào không gian dịch đối chiếu chính</td>
            </tr>
            <tr>
              <td><strong>Chuyển Chế Độ</strong></td>
              <td>Chuyển sang Đọc sách</td>
              <td><kbd>Mod+3</kbd></td>
              <td>Xem tác phẩm với giao diện trang sách êm mắt</td>
            </tr>
            <tr>
              <td><strong>Bố Cục</strong></td>
              <td>Bố cục 2 Cột</td>
              <td><kbd>Mod+Alt+1</kbd></td>
              <td>Bảng câu bên trái - Tra cứu & AI bên phải</td>
            </tr>
            <tr>
              <td><strong>Bố Cục</strong></td>
              <td>Bố cục 2 Tầng</td>
              <td><kbd>Mod+Alt+2</kbd></td>
              <td>Bảng câu ở trên - Tra cứu & AI ở dưới</td>
            </tr>
            <tr>
              <td><strong>Điều Hướng Câu</strong></td>
              <td>Xác nhận & Sang câu kế</td>
              <td><kbd>Ctrl+Enter</kbd> / <kbd>Cmd+Enter</kbd></td>
              <td>Lưu câu vào TM và nhảy đến câu tiếp theo</td>
            </tr>
            <tr>
              <td><strong>Điều Hướng Câu</strong></td>
              <td>Nhảy câu trên / dưới</td>
              <td><kbd>Alt+↑</kbd> / <kbd>Alt+↓</kbd></td>
              <td>Di chuyển con trỏ giữa các hàng câu</td>
            </tr>
            <tr>
              <td><strong>Điều Hướng Câu</strong></td>
              <td>Nhảy đến câu chưa dịch</td>
              <td><kbd>Mod+Shift+↓</kbd></td>
              <td>Tìm câu trống gần nhất để dịch tiếp</td>
            </tr>
            <tr>
              <td><strong>Thao Tác Câu</strong></td>
              <td>Tách câu (Split)</td>
              <td><kbd>Mod+K</kbd></td>
              <td>Chia câu dài thành 2 câu độc lập</td>
            </tr>
            <tr>
              <td><strong>Thao Tác Câu</strong></td>
              <td>Gộp câu (Merge)</td>
              <td><kbd>Mod+J</kbd></td>
              <td>Nối câu hiện tại với câu tiếp theo</td>
            </tr>
            <tr>
              <td><strong>Thao Tác Câu</strong></td>
              <td>Cắt bỏ câu (Omit)</td>
              <td><kbd>Mod+Del</kbd></td>
              <td>Ẩn câu thừa, lời bạt, quảng cáo</td>
            </tr>
            <tr>
              <td><strong>Thao Tác Câu</strong></td>
              <td>Lịch sử phiên bản câu</td>
              <td><kbd>Mod+H</kbd></td>
              <td>Xem lại các lần sửa trước đó và khôi phục</td>
            </tr>
            <tr>
              <td><strong>Tra Cứu</strong></td>
              <td>Tra cứu nhanh (Drawer)</td>
              <td><kbd>Mod+Shift+L</kbd></td>
              <td>Mở ngăn kéo từ điển từ bất kỳ màn hình nào</td>
            </tr>
            <tr>
              <td><strong>Trí Tuệ Nhân Tạo</strong></td>
              <td>Dịch AI câu hiện tại</td>
              <td><kbd>Mod+Space</kbd></td>
              <td>Gửi câu đang chọn đến mô hình AI đã cấu hình</td>
            </tr>
            <tr>
              <td><strong>Trí Tuệ Nhân Tạo</strong></td>
              <td>Soi lời nhắc (Inspector)</td>
              <td><kbd>Mod+Shift+I</kbd></td>
              <td>Xem chi tiết lời nhắc và ngữ cảnh gửi đến AI</td>
            </tr>
            <tr>
              <td><strong>Hệ Thống</strong></td>
              <td>Bảng cài đặt</td>
              <td><kbd>Mod+,</kbd></td>
              <td>Cấu hình khóa API, phông chữ, phím tắt</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <!-- Footer -->
    <footer class="cds--footer">
      <div>
        <strong>AuraTranslate v2.0</strong> — Nền tảng Dịch thuật Ngoại tuyến Độc lập & An toàn.
      </div>
      <div>
        Thiết kế theo chuẩn IBM Carbon Design System v11 · Bản quyền GPL-3.0.
      </div>
    </footer>
  </main>

  <!-- MODAL 1: PROMPT INSPECTOR -->
  <div class="cds--modal" id="modalPromptInspector">
    <div class="cds--modal-container">
      <div class="cds--modal-header">
        <h3 class="cds--modal-title">🔍 Cửa Sổ Soi Lời Nhắc (AI Prompt Inspector)</h3>
        <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="closeModal('modalPromptInspector')">✕ Đóng</button>
      </div>
      <div class="cds--modal-body">
        <p style="margin-bottom: 12px; color: var(--cds-text-secondary);">
          Dưới đây là 100% nội dung thực tế mà AuraTranslate tổng hợp và gửi đến API của nhà cung cấp AI cho câu dịch hiện tại:
        </p>

        <div style="margin-bottom: 16px;">
          <strong style="color: var(--cds-support-cyan); font-family: var(--font-mono); font-size: 12px;">SYSTEM PROMPT (LỜI NHẮC HỆ THỐNG):</strong>
          <pre style="background: var(--cds-background); padding: 12px; border: 1px solid var(--cds-border-subtle-01); font-family: var(--font-mono); font-size: 12px; margin-top: 6px; white-space: pre-wrap; color: var(--cds-text-primary);">Bạn là một dịch giả văn học chuyên nghiệp, am hiểu sâu sắc thể loại Tiên hiệp và Cổ trang Trung Quốc.
Nhiệm vụ: Dịch câu văn sang tiếng Việt tự nhiên, đậm chất văn học, giữ nguyên âm Hán Việt chuẩn mực cho danh xưng, môn phái và cảnh giới tu luyện.
Bắt buộc tuân thủ danh mục thuật ngữ đã chỉ định.</pre>
        </div>

        <div style="margin-bottom: 16px;">
          <strong style="color: var(--cds-support-purple); font-family: var(--font-mono); font-size: 12px;">DYNAMIC GLOSSARY INJECTION (THUẬT NGỮ BẮT BUỘC):</strong>
          <pre style="background: var(--cds-background); padding: 12px; border: 1px solid var(--cds-border-subtle-01); font-family: var(--font-mono); font-size: 12px; margin-top: 6px; white-space: pre-wrap; color: var(--cds-text-primary);">- 青云门 (Qingyun Sect) -> Thanh Vân Môn
- 山巅 (Mountain Peak) -> Đỉnh núi / Sơn điên</pre>
        </div>

        <div>
          <strong style="color: var(--cds-support-success); font-family: var(--font-mono); font-size: 12px;">USER CONTENT (CÂU GỐC & NGỮ CẢNH):</strong>
          <pre style="background: var(--cds-background); padding: 12px; border: 1px solid var(--cds-border-subtle-01); font-family: var(--font-mono); font-size: 12px; margin-top: 6px; white-space: pre-wrap; color: var(--cds-text-primary);">[Ngữ cảnh trước]: 天气晴朗，万里无云。
[Câu cần dịch]: 他缓缓抬头，望向青云门的山巅。
[Ngữ cảnh sau]: 风声萧萧，带着一丝凉意。</pre>
        </div>
      </div>
      <div class="cds--modal-footer">
        <button class="cds--btn cds--btn--secondary cds--btn--sm" onclick="closeModal('modalPromptInspector')">Đóng cửa sổ</button>
        <button class="cds--btn cds--btn--primary cds--btn--sm" onclick="alert('Đã sao chép toàn bộ prompt vào khay nhớ tạm!'); closeModal('modalPromptInspector');">Sao chép Prompt</button>
      </div>
    </div>
  </div>

  <!-- MODAL 2: SEGMENT HISTORY -->
  <div class="cds--modal" id="modalSegmentHistory">
    <div class="cds--modal-container">
      <div class="cds--modal-header">
        <h3 class="cds--modal-title">🕒 Lịch Sử Phiên Bản Câu Dịch #002</h3>
        <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="closeModal('modalSegmentHistory')">✕ Đóng</button>
      </div>
      <div class="cds--modal-body">
        <p style="margin-bottom: 12px; color: var(--cds-text-secondary);">
          Mọi lần chỉnh sửa câu dịch này đều được tự động lưu lại. Bạn có thể chọn bất kỳ phiên bản nào để khôi phục:
        </p>

        <div class="cds--steps">
          <div class="cds--step-item">
            <div class="cds--step-num" style="background: var(--cds-support-success);">1</div>
            <div class="cds--step-content">
              <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                <strong>Phiên bản hiện tại (Người dịch gõ trực tiếp)</strong>
                <span style="font-size: 11px; color: var(--cds-text-placeholder);">10:45:12</span>
              </div>
              <p style="color: var(--cds-text-primary); font-size: 13.5px; margin-bottom: 4px;">"Hắn chậm rãi ngẩng đầu, nhìn về phía đỉnh núi của Thanh Vân Môn."</p>
              <span class="cds--tag cds--tag--green">Đang hiển thị</span>
            </div>
          </div>

          <div class="cds--step-item">
            <div class="cds--step-num" style="background: var(--cds-support-cyan);">2</div>
            <div class="cds--step-content">
              <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                <strong>Phiên bản gợi ý AI (Claude 3.5 Sonnet)</strong>
                <span style="font-size: 11px; color: var(--cds-text-placeholder);">10:42:05</span>
              </div>
              <p style="color: var(--cds-text-primary); font-size: 13.5px; margin-bottom: 4px;">"Hắn từ từ ngước mắt, ngóng trông đỉnh núi mù sương Thanh Vân Môn."</p>
              <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="document.getElementById('simTargetInput').value = 'Hắn từ từ ngước mắt, ngóng trông đỉnh núi mù sương Thanh Vân Môn.'; alert('Đã khôi phục câu dịch về phiên bản AI!'); closeModal('modalSegmentHistory');">Khôi phục bản này</button>
            </div>
          </div>

          <div class="cds--step-item">
            <div class="cds--step-num" style="background: var(--cds-support-purple);">3</div>
            <div class="cds--step-content">
              <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                <strong>Phiên bản gợi ý Bộ nhớ dịch TM (92% Khớp)</strong>
                <span style="font-size: 11px; color: var(--cds-text-placeholder);">Hôm qua</span>
              </div>
              <p style="color: var(--cds-text-primary); font-size: 13.5px; margin-bottom: 4px;">"Hắn chậm rãi ngẩng đầu nhìn đỉnh núi."</p>
              <button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="document.getElementById('simTargetInput').value = 'Hắn chậm rãi ngẩng đầu nhìn đỉnh núi.'; alert('Đã khôi phục câu dịch về phiên bản TM!'); closeModal('modalSegmentHistory');">Khôi phục bản này</button>
            </div>
          </div>
        </div>
      </div>
      <div class="cds--modal-footer">
        <button class="cds--btn cds--btn--secondary cds--btn--sm" onclick="closeModal('modalSegmentHistory')">Đóng</button>
      </div>
    </div>
  </div>

  <!-- Interactive JavaScript Logic -->
  <script>
    // Theme toggle (Carbon Dark / Light)
    function toggleTheme() {
      const htmlEl = document.documentElement;
      const currentTheme = htmlEl.getAttribute('data-theme');
      const nextTheme = currentTheme === 'light' ? 'dark' : 'light';
      htmlEl.setAttribute('data-theme', nextTheme);
      
      const btnText = document.getElementById('themeBtnText');
      if (nextTheme === 'light') {
        btnText.innerText = '🌙 Giao diện Tối';
      } else {
        btnText.innerText = '☀️ Giao diện Sáng';
      }
    }

    // Toggle Workspace Layout in Simulator
    function toggleSimLayout() {
      const body = document.getElementById('simWorkspaceBody');
      const text = document.getElementById('simLayoutText');
      if (body.classList.contains('layout-vertical')) {
        body.classList.remove('layout-vertical');
        text.innerText = '⊞ Đổi sang Bố cục Trên-Dưới';
      } else {
        body.classList.add('layout-vertical');
        text.innerText = '⊟ Đổi sang Bố cục 2 Cột';
      }
    }

    // Toggle Context Menu in Simulator
    function toggleSimContextMenu() {
      const menu = document.getElementById('simContextMenu');
      if (menu.style.display === 'none' || menu.style.display === '') {
        menu.style.display = 'block';
      } else {
        menu.style.display = 'none';
      }
    }

    // Switch between modes in simulator
    function switchSimMode(mode) {
      if (mode === 'library') {
        document.getElementById('library-import').scrollIntoView({ behavior: 'smooth' });
      } else if (mode === 'workspace') {
        document.getElementById('workspace-core').scrollIntoView({ behavior: 'smooth' });
      } else if (mode === 'reading') {
        document.getElementById('word-review-reading').scrollIntoView({ behavior: 'smooth' });
      } else if (mode === 'review') {
        document.getElementById('word-review-reading').scrollIntoView({ behavior: 'smooth' });
      }
    }

    // Apply glossary in simulator
    function applyGlossaryTerm() {
      const input = document.getElementById('simTargetInput');
      if (!input.value.includes('Thanh Vân Môn')) {
        input.value += ' Thanh Vân Môn';
      }
      alert('Đã áp dụng chuẩn thuật ngữ "Thanh Vân Môn" vào bản dịch!');
    }

    // Apply TM suggestion in simulator
    function applyTmSuggestion() {
      document.getElementById('simTargetInput').value = 'Hắn chậm rãi ngẩng đầu nhìn đỉnh núi.';
      alert('Đã chép câu từ Bộ nhớ dịch TM vào ô soạn thảo!');
    }

    // Modal Control
    function openModal(modalId) {
      const modal = document.getElementById(modalId);
      if (modal) modal.classList.add('is-visible');
    }

    function closeModal(modalId) {
      const modal = document.getElementById(modalId);
      if (modal) modal.classList.remove('is-visible');
    }

    // Global Search Filter (Cmd+K / Ctrl+K)
    document.addEventListener('keydown', function(e) {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        const searchInput = document.getElementById('globalSearch');
        if (searchInput) searchInput.focus();
      }
    });

    const searchInput = document.getElementById('globalSearch');
    if (searchInput) {
      searchInput.addEventListener('input', function(e) {
        const query = e.target.value.toLowerCase().trim();
        const navLinks = document.querySelectorAll('.cds--side-nav__link');
        navLinks.forEach(link => {
          const text = link.innerText.toLowerCase();
          if (!query || text.includes(query)) {
            link.style.display = 'flex';
          } else {
            link.style.display = 'none';
          }
        });
      });
    }

    // Highlight active nav link on scroll
    window.addEventListener('scroll', function() {
      const sections = document.querySelectorAll('section[id]');
      let currentSectionId = '';
      const scrollPos = window.scrollY + 100;

      sections.forEach(sec => {
        if (sec.offsetTop <= scrollPos) {
          currentSectionId = sec.getAttribute('id');
        }
      });

      if (currentSectionId) {
        const links = document.querySelectorAll('.cds--side-nav__link');
        links.forEach(l => {
          if (l.getAttribute('href') === '#' + currentSectionId) {
            l.classList.add('active');
          } else {
            l.classList.remove('active');
          }
        });
      }
    });
  </script>
</body>
</html>
"""
