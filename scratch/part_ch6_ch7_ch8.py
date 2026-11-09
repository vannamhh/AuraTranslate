# -*- coding: utf-8 -*-
"""Chapters 6, 7, and 8: Sentence Context Menu, Offline Lookup, AI Translation."""

def get_chapters_6_7_8():
    return """
    <!-- CHƯƠNG 6 -->
    <section id="sentence-menu" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>06</span>
        <span>Menu Chuột Từng Câu & Lịch Sử Phiên Bản</span>
      </h2>
      <p>
        Khi nhấp chuột phải vào bất kỳ hàng câu nào trên Bàn dịch, một menu ngữ cảnh mạnh mẽ sẽ xuất hiện giúp bạn xử lý các tình huống cấu trúc câu đặc biệt.
      </p>

      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Cắt Bỏ Câu (Omit Segment)</span>
            <kbd>Mod+Del</kbd>
          </div>
          <p>
            Áp dụng cho các dòng lời bạt của tác giả, quảng cáo trang web nguồn hoặc thông báo xin phiếu bầu. Câu bị cắt bỏ sẽ mờ đi và không được tính vào số từ cần dịch, đồng thời được bỏ qua khi xuất sách cuối cùng.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Khôi Phục Câu (Restore Segment)</span>
            <kbd>Mod+Shift+Del</kbd>
          </div>
          <p>
            Nếu bạn vô tình cắt nhầm một câu quan trọng, chỉ cần nhấp chuột phải và chọn <strong>Khôi phục câu</strong>. Dữ liệu gốc và câu dịch trước đó sẽ hiển thị lại trạng thái hoạt động bình thường.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Tách Câu (Split Segment)</span>
            <kbd>Mod+K</kbd>
          </div>
          <p>
            Văn phong tiếng nước ngoài thường có những câu ghép rất dài với nhiều vế phụ. Đặt con trỏ chuột tại vị trí muốn ngắt và chọn Tách câu: Hệ thống sẽ chia đôi câu gốc và tạo ra 2 hàng dịch độc lập giúp câu văn tiếng Việt trong sáng hơn.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Gộp Câu (Merge Segment)</span>
            <kbd>Mod+J</kbd>
          </div>
          <p>
            Khi tác giả gốc ngắt dòng giữa chừng làm câu văn bị gãy vụn, bạn chỉ cần chọn Gộp câu. Câu hiện tại sẽ tự động nối với câu tiếp theo ngay bên dưới thành một chỉnh thể hoàn chỉnh.
          </p>
        </div>
      </div>

      <h3 class="cds--subsection-title">Lịch Sử Phiên Bản Từng Câu (Segment History)</h3>
      <p>
        AuraTranslate ghi lại toàn bộ dấu vết sửa đổi của từng câu. Nhấp chuột phải chọn <strong>Lịch sử câu</strong> (<kbd>Mod+H</kbd>) để mở cửa sổ đối chiếu:
      </p>
      <div class="cds--data-table-container">
        <table class="cds--data-table">
          <thead>
            <tr>
              <th>Thời gian</th>
              <th>Nguồn gốc dịch</th>
              <th>Nội dung câu dịch đã lưu</th>
              <th>Thao tác</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>10:45:12 (Hôm nay)</td>
              <td><span class="cds--tag cds--tag--blue">Người gõ trực tiếp</span></td>
              <td>Hắn chậm rãi ngẩng đầu, nhìn về phía đỉnh núi của Thanh Vân Môn.</td>
              <td><span class="cds--tag cds--tag--green">Phiên bản hiện tại</span></td>
            </tr>
            <tr>
              <td>10:42:05 (Hôm nay)</td>
              <td><span class="cds--tag cds--tag--cyan">Gợi ý AI (Claude)</span></td>
              <td>Hắn từ từ ngước mắt, ngóng trông đỉnh núi Thanh Vân Môn.</td>
              <td><button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="alert('Đã khôi phục về phiên bản gợi ý của Claude!')">Khôi phục bản này</button></td>
            </tr>
            <tr>
              <td>Hôm qua</td>
              <td><span class="cds--tag cds--tag--purple">Bộ nhớ TM (Fuzzy)</span></td>
              <td>Hắn chậm rãi ngẩng đầu nhìn đỉnh núi.</td>
              <td><button class="cds--btn cds--btn--tertiary cds--btn--sm" onclick="alert('Đã khôi phục về phiên bản Bộ nhớ TM!')">Khôi phục bản này</button></td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <!-- CHƯƠNG 7 -->
    <section id="lookup-offline" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>07</span>
        <span>Tra Cứu Từ Điển Ngoại Tuyến Siêu Tốc</span>
      </h2>
      <p>
        Tốc độ là yếu tố sống còn của dịch giả. Bộ công cụ tra cứu tích hợp sẵn trong AuraTranslate phản hồi với độ trễ <strong>dưới 50 mili-giây</strong> và hoạt động 100% không phụ thuộc vào kết nối mạng.
      </p>

      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Tự Động Tra Khi Bôi Đen (Auto-Lookup)</span>
            <span class="cds--tag cds--tag--green">&lt;50ms</span>
          </div>
          <p>
            Chỉ cần dùng chuột bôi đen bất kỳ từ hoặc cụm từ nào trên câu gốc hoặc bản dịch, bảng tra cứu bên phải sẽ tự động hiển thị nghĩa ngay tức khắc mà không cần bấm thêm bất kỳ phím nào.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Kho Từ Điển Tích Hợp Phong Phú</span>
            <span class="cds--tag cds--tag--purple">Đa Tầng</span>
          </div>
          <p>
            Bao gồm: Từ điển Lạc Việt (Anh-Việt, Trung-Việt), Từ điển Thiều Chửu (phân tích chi tiết bộ thủ Hán tự, ngữ nghĩa cổ xưa), CC-CEDICT (Trung-Anh hiện đại kèm Pinyin), và Tự điển Hán Việt hiện đại.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Concordance — Tra Cứu Chéo Ngữ Cảnh</span>
            <span class="cds--tag cds--tag--blue">Thực Tế</span>
          </div>
          <p>
            Chuyển sang tab <strong>Bộ nhớ (TM)</strong> để tìm xem cụm từ này đã từng được bạn dịch như thế nào trong các chương trước hoặc trong các tác phẩm khác cùng tác giả. Giúp giọng văn luôn nhất quán từ đầu đến cuối sách.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Nút Ghim Kết Quả Tra Cứu (📌)</span>
            <span class="cds--tag cds--tag--cyan">Cố Định</span>
          </div>
          <p>
            Khi bạn tìm thấy một định nghĩa phức tạp cần lưu ý trong suốt cả đoạn văn, hãy bấm nút <strong>Ghim 📌</strong>. Dù bạn có nhấp chuột sang câu khác thì bảng tra cứu vẫn giữ nguyên nội dung từ đó cho bạn tham khảo.
          </p>
        </div>
      </div>

      <h3 class="cds--subsection-title">Ngăn Kéo Tra Cứu Nhanh Bên Phải (Lookup Drawer)</h3>
      <p>
        Nếu bạn đang làm việc ở Thư viện hoặc Chế độ đọc sách mà muốn tra nhanh một từ ngữ mà không muốn chuyển sang Bàn dịch, bạn chỉ cần bấm <kbd>Mod+Shift+L</kbd>. Ngăn kéo tra cứu sẽ trượt ra từ mép phải màn hình, cho phép tra từ điển và tra cứu thuật ngữ rồi tự động thu gọn lại.
      </p>
    </section>

    <!-- CHƯƠNG 8 -->
    <section id="ai-translation" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>08</span>
        <span>Dịch AI Cá Nhân Hóa & Dịch Hàng Loạt (Batch Translation)</span>
      </h2>
      <p>
        AuraTranslate tiếp cận Trí Tuệ Nhân Tạo theo phương châm: <em>AI là người trợ lý đắc lực phục vụ con người, không phải người thay thế</em>. 
        Mọi đề xuất dịch thuật của AI đều phải qua sự kiểm duyệt và phê duyệt của dịch giả.
      </p>

      <div class="cds--inline-notification cds--inline-notification--warning">
        <div class="cds--inline-notification__content">
          <div class="cds--inline-notification__title">Lưu ý quan trọng về các nhà cung cấp AI:</div>
          <div>
            AuraTranslate tích hợp trực tiếp các nhà cung cấp AI hàng đầu: 
            <strong>Anthropic Claude</strong> (Claude 3.5 Sonnet / Haiku / Opus), 
            <strong>Google Gemini</strong> (Gemini 1.5 Pro / Flash), 
            <strong>OpenAI</strong> (GPT-4o, GPT-4o-mini), 
            <strong>OpenRouter</strong>, <strong>Mistral</strong>, <strong>xAI</strong>, 
            và các công cụ AI cục bộ 100% offline như <strong>Ollama</strong> và <strong>LM Studio</strong> (chạy các mô hình Llama, Qwen, Mistral trên GPU riêng của bạn).
          </div>
        </div>
      </div>

      <h3 class="cds--subsection-title">Chế Độ Dịch Từng Câu & Dịch Hàng Loạt (Batch)</h3>
      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Dịch Đơn Câu (Single Segment)</span>
            <kbd>Mod+Space</kbd>
          </div>
          <p>
            Khi con trỏ ở câu nào, bấm phím tắt để gửi riêng câu đó kèm ngữ cảnh xung quanh tới AI. Bản dịch gợi ý sẽ hiển thị ở khung AI; bạn có thể bấm <strong>Áp dụng</strong> hoặc tinh chỉnh trực tiếp trước khi xác nhận.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Dịch Hàng Loạt (Batch Translation)</span>
            <span class="cds--tag cds--tag--purple">Tự Động Cả Chương</span>
          </div>
          <p>
            Chọn chức năng <strong>Dịch hàng loạt chương này</strong>: AuraTranslate sẽ tự động gửi từng đợt câu đến AI, đưa kết quả vào ô dịch và tự động lưu. Bạn có thể thảnh thơi nhâm nhi tách cà phê trong lúc máy dịch hàng trăm câu liên tục.
          </p>
        </div>
      </div>

      <h3 class="cds--subsection-title">Cơ Chế An Toàn Khi Dịch Hàng Loạt</h3>
      <div class="cds--steps">
        <div class="cds--step-item">
          <div class="cds--step-num">1</div>
          <div class="cds--step-content">
            <h4>Nút HỦY DỊCH NGAY LẬP TỨC (Cancel Batch Translation)</h4>
            <p>
              Nếu phát hiện bạn chọn nhầm prompt hoặc muốn đổi mô hình AI giữa chừng, chỉ cần bấm nút <strong>Hủy Dịch</strong> màu đỏ. Quá trình dừng lại ngay lập tức mà <em>không làm mất hay ảnh hưởng đến bất kỳ câu nào đã được dịch xong trước đó</em>.
            </p>
          </div>
        </div>

        <div class="cds--step-item">
          <div class="cds--step-num">2</div>
          <div class="cds--step-content">
            <h4>Nút THỬ LẠI CÂU LỖI (Retry Failed Segments)</h4>
            <p>
              Trong quá trình dịch hàng loạt, nếu gặp sự cố rớt mạng hoặc hết hạn mức (rate limit) ở câu nào, AuraTranslate sẽ gắn cờ báo lỗi riêng câu đó. Bạn chỉ cần bấm <strong>Thử lại câu lỗi</strong>: phần mềm sẽ chỉ gửi lại các câu lỗi mà không dịch lại những câu đã thành công, tiết kiệm tối đa thời gian và chi phí API.
            </p>
          </div>
        </div>

        <div class="cds--step-item">
          <div class="cds--step-num">3</div>
          <div class="cds--step-content">
            <h4>Bảng Thống Kê Chi Phí & Token Minh Bạch</h4>
            <p>
              Phần mềm hiển thị chính xác số lượng Token gửi đi (Prompt tokens) và số lượng Token nhận về (Completion tokens) cho từng câu cũng như tổng kết toàn bộ chương, giúp bạn luôn chủ động kiểm soát chi tiêu API của mình.
            </p>
          </div>
        </div>
      </div>
    </section>
"""
