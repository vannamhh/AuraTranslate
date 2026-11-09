# -*- coding: utf-8 -*-
"""Chapters 3, 4, and 5: Library & Import, Chapter Management, Workspace Operations."""

def get_chapters_3_4_5():
    return """
    <!-- CHƯƠNG 3 -->
    <section id="library-import" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>03</span>
        <span>Thư viện & 4 Cổng Nhập Tài Liệu Đa Dạng</span>
      </h2>
      <p>
        Không gian Thư viện (<kbd>Mod+1</kbd>) là nơi quản lý toàn bộ các tác phẩm dịch. AuraTranslate hỗ trợ <strong>4 cổng nhập dữ liệu thông minh</strong>, giúp người dùng đưa tài liệu vào dự án mà không cần phải chuyển đổi thủ công phức tạp.
      </p>

      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">1. Văn Bản Thô (TXT / Markdown)</span>
            <span class="cds--tag cds--tag--blue">Phổ Biến</span>
          </div>
          <p>
            Tự động nhận diện bảng mã ký tự quốc tế (UTF-8, GBK, GB2312, Big5, Shift-JIS). Hệ thống tự phân tách các đoạn văn dài thành các câu đơn độc lập để sẵn sàng đưa vào bàn dịch đối chiếu.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">2. Sách Điện Tử (EPUB)</span>
            <span class="cds--tag cds--tag--purple">Giữ Nguyên Mục Lục</span>
          </div>
          <p>
            Đọc sâu cấu trúc file EPUB, bóc tách nguyên vẹn cây danh mục chương hồi (Table of Contents), trích xuất kho ảnh minh họa và bìa sách đưa vào cơ sở dữ liệu dự án mà không làm mất liên kết.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">3. Tài Liệu Word (.docx)</span>
            <span class="cds--tag cds--tag--cyan">Cộng Tác Nhanh</span>
          </div>
          <p>
            Nhập trực tiếp các tệp Word do dịch giả hoặc cộng tác viên gửi đến. Giữ nguyên cấu trúc phân đoạn và định dạng chữ cơ bản, chuyển đổi trơn tru thành các đơn vị câu cần dịch.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">4. Trang Web Trực Tuyến (URL)</span>
            <span class="cds--tag cds--tag--green">Tự Động Bóc Tách</span>
          </div>
          <p>
            Dán đường dẫn chương truyện hoặc bài báo trên mạng; AuraTranslate tự động làm sạch mã HTML, loại bỏ quảng cáo, trích xuất chính xác văn bản nội dung và tải toàn bộ ảnh minh họa về máy.
          </p>
        </div>
      </div>

      <h3 class="cds--subsection-title">Cổng Nhập Song Ngữ Đối Chiếu (Bilingual Import)</h3>
      <p>
        Khi bạn đã có sẵn cả file tiếng nước ngoài và file tiếng Việt dịch sẵn từ trước (ví dụ file truyện đã dịch dở trên mạng), bạn có thể dùng tính năng <strong>Nhập Song Ngữ</strong>. Hệ thống sẽ mở giao diện Căn chỉnh (Alignment) để bạn ráp từng câu gốc với câu dịch tương ứng, tạo thành kho Bộ nhớ dịch (TM) cực kỳ giá trị.
      </p>

      <h3 class="cds--subsection-title">Công Cụ Quản Lý Thư Viện Chuyên Sâu</h3>
      <div class="cds--steps">
        <div class="cds--step-item">
          <div class="cds--step-num">A</div>
          <div class="cds--step-content">
            <h4>Quét Tác Phẩm Mồ Côi (Orphan Works Scanner)</h4>
            <p>
              Khi bạn di chuyển thư mục dự án trên máy tính hoặc đổi tên ổ cứng, các tệp cơ sở dữ liệu <code>.atproj</code> có thể bị mất đường dẫn liên kết trong Thư viện. Tính năng này quét toàn bộ các thư mục bạn chỉ định và tự động kết nối lại các cuốn sách vào Thư viện nguyên vẹn 100%.
            </p>
          </div>
        </div>

        <div class="cds--step-item">
          <div class="cds--step-num">B</div>
          <div class="cds--step-content">
            <h4>Dựng Lại Chỉ Mục Toàn Văn (Rebuild Full-Text Search Index)</h4>
            <p>
              Tối ưu hóa và tái tạo lại toàn bộ cây chỉ mục FTS của cơ sở dữ liệu SQLite sau khi nhập hàng triệu từ vựng. Đảm bảo tốc độ tra cứu từ khóa và thuật ngữ trên toàn bộ hàng nghìn chương sách luôn đạt tốc độ tức thì dưới 10 mili-giây.
            </p>
          </div>
        </div>
      </div>
    </section>

    <!-- CHƯƠNG 4 -->
    <section id="chapter-management" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>04</span>
        <span>Quản Lý Chương & Xuất Xứ Bản Thảo</span>
      </h2>
      <p>
        Một tác phẩm dài thường bao gồm hàng trăm đến hàng nghìn chương. AuraTranslate cung cấp hệ thống cây mục lục động giúp bạn tổ chức tác phẩm một cách khoa học.
      </p>

      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Tách Chương (Split Chapter)</span>
            <span class="cds--tag cds--tag--blue">Linh Hoạt</span>
          </div>
          <p>
            Khi tác giả gốc gom nhiều hồi nhỏ vào cùng 1 chương dài hàng vạn chữ, bạn chỉ cần chọn câu ngắt đoạn và nhấn <strong>Tách chương</strong>. Phần mềm sẽ tự động chia thành Chương 1a, Chương 1b hoặc đặt tên chương mới mà không làm xáo trộn tiến độ dịch.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Gộp Chương (Merge Chapters)</span>
            <span class="cds--tag cds--tag--purple">Gọn Gàng</span>
          </div>
          <p>
            Ngược lại, nếu các đoạn văn ngắn bị tải về thành các chương lắt nhắt, bạn có thể chọn nhiều chương liên tiếp và nhấn <strong>Gộp chương</strong>. Toàn bộ các câu dịch và thuật ngữ đi kèm sẽ được giữ nguyên vẹn.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Quản Lý Ảnh Minh Họa Chương</span>
            <span class="cds--tag cds--tag--cyan">Hình Ảnh</span>
          </div>
          <p>
            Xem danh sách ảnh minh họa đính kèm từng chương, chèn thêm tranh vẽ của họa sĩ hoặc sơ đồ bản đồ tu tiên/địa lý. Các bức ảnh này sẽ được nhúng tự động khi bạn xuất sách sang định dạng EPUB hoặc Word.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Lưu Trữ Xuất Xứ (Source Origin)</span>
            <span class="cds--tag cds--tag--green">Bản Quyền</span>
          </div>
          <p>
            Lưu giữ đầy đủ thông tin: Tác giả gốc, Tên trang web nguồn, Đường link URL gốc, Ngày xuất bản lần đầu. Giúp bạn dễ dàng đối chiếu bản quyền hoặc kiểm chứng phiên bản văn bản khi tác giả sửa đổi raw.
          </p>
        </div>
      </div>
    </section>

    <!-- CHƯƠNG 5 -->
    <section id="workspace-core" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>05</span>
        <span>Bàn Dịch Workspace — Thao Tác Chuyên Sâu</span>
      </h2>
      <p>
        Không gian Bàn dịch (<kbd>Mod+2</kbd>) là trái tim của AuraTranslate, nơi dịch giả dành 95% thời gian làm việc. Mọi chi tiết trên màn hình này đều được tinh chỉnh để giảm tối đa thao tác thừa.
      </p>

      <h3 class="cds--subsection-title">1. Dòng Câu Song Ngữ Đa Tầng</h3>
      <p>
        Mỗi đơn vị dịch trong bảng đối chiếu được trình bày theo cấu trúc 3 tầng trực quan:
      </p>
      <ul style="margin-left: 20px; margin-bottom: 20px; color: var(--cds-text-secondary); line-height: 1.8;">
        <li><strong style="color: var(--cds-support-cyan);">Tầng 1 (Phiên âm Pinyin):</strong> Hiển thị phiên âm La-tinh có dấu thanh điệu chuẩn mực của từng chữ tiếng Trung, hỗ trợ người dịch nắm rõ cách phát âm và nhận diện đồng âm dị nghĩa.</li>
        <li><strong style="color: var(--cds-text-primary);">Tầng 2 (Nguyên tác chữ gốc):</strong> Hiển thị chữ Hán rõ nét với độ tương phản cao, phông chữ được thiết kế chống mỏi mắt.</li>
        <li><strong style="color: var(--cds-support-purple);">Tầng 3 (Chuyển tự Hán Việt):</strong> Tự động chuyển đổi từng chữ Hán sang âm Hán Việt chuẩn xác, cực kỳ đắc lực cho người dịch truyện kiếm hiệp, tiên hiệp hoặc tài liệu cổ.</li>
      </ul>

      <h3 class="cds--subsection-title">2. Ô Soạn Thảo Bản Dịch Trực Tiếp & Tự Động Lưu</h3>
      <p>
        Ngay dưới dòng nguyên tác là ô soạn thảo bản dịch tiếng Việt. 
        AuraTranslate áp dụng cơ chế <strong>Lưu tức thời (Instant Auto-save)</strong>: Mọi ký tự bạn vừa gõ đều được ghi nhận trực tiếp vào cơ sở dữ liệu SQLite sau 300ms ngừng gõ. Dù mất điện đột ngột hoặc tắt máy bất ngờ, bạn không bao giờ bị mất dữ liệu.
      </p>

      <h3 class="cds--subsection-title">3. Tùy Chọn Đổi Bố Cục Màn Hình (Layout Presets)</h3>
      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Bố Cục 2 Cột (Mặc Định)</span>
            <kbd>Mod+Alt+1</kbd>
          </div>
          <p>
            Cột bên trái là Bảng Lưới câu đối chiếu chiếm 65% chiều rộng. Cột bên phải chia làm 2 tầng: Nửa trên là Tra cứu Từ điển/TM, nửa dưới là Trợ lý AI. Tối ưu cho màn hình máy tính thông thường (16:9, 16:10).
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Bố Cục 2 Tầng (Trên - Dưới)</span>
            <kbd>Mod+Alt+2</kbd>
          </div>
          <p>
            Bảng Lưới câu nằm ở nửa trên màn hình, khung Tra cứu từ điển và Dịch AI nằm ở nửa dưới. Cực kỳ thuận tiện cho màn hình đặt dọc (Pivot) hoặc khi bạn cần chiều dài câu dịch trải rộng tối đa.
          </p>
        </div>
      </div>

      <h3 class="cds--subsection-title">4. Điều Hướng Câu Cực Nhanh Bằng Bàn Phím</h3>
      <p>
        Dịch giả chuyên nghiệp không cần phải nhấc tay khỏi bàn phím để cầm chuột. Hãy tận dụng hệ thống phím tắt điều hướng:
      </p>
      <div class="cds--data-table-container">
        <table class="cds--data-table">
          <thead>
            <tr>
              <th>Thao tác</th>
              <th>Phím tắt</th>
              <th>Mô tả tác vụ</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td><strong>Xác nhận & Nhảy câu</strong></td>
              <td><kbd>Ctrl+Enter</kbd> hoặc <kbd>Cmd+Enter</kbd></td>
              <td>Lưu câu hiện tại vào bộ nhớ dịch (TM) và tự động chuyển con trỏ sang câu kế tiếp.</td>
            </tr>
            <tr>
              <td><strong>Di chuyển giữa các câu</strong></td>
              <td><kbd>Alt+↓</kbd> / <kbd>Alt+↑</kbd></td>
              <td>Di chuyển lên câu trước hoặc xuống câu sau mà không thay đổi nội dung.</td>
            </tr>
            <tr>
              <td><strong>Nhảy đến câu chưa dịch</strong></td>
              <td><kbd>Mod+Shift+↓</kbd></td>
              <td>Tự động cuộn và đặt con trỏ vào câu trống gần nhất đang cần dịch.</td>
            </tr>
            <tr>
              <td><strong>Nhảy đến câu cắm cờ</strong></td>
              <td><kbd>Mod+Shift+F</kbd></td>
              <td>Nhảy ngay đến các câu có đánh dấu nghi vấn ⚑ cần rà soát lại.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
"""
