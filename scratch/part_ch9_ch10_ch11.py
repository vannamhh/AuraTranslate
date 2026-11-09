# -*- coding: utf-8 -*-
"""Chapters 9, 10, and 11: Prompt Library, Glossary System, TM & Alignment."""

def get_chapters_9_10_11():
    return """
    <!-- CHƯƠNG 9 -->
    <section id="prompt-library" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>09</span>
        <span>Thư Viện Lời Nhắc (Prompt Library) & Công Cụ Soi Prompt</span>
      </h2>
      <p>
        Chất lượng bản dịch AI phụ thuộc 80% vào chất lượng lời nhắc (Prompt). 
        AuraTranslate tích hợp sẵn một kho <strong>Thư viện Prompt mẫu chuyên biệt</strong> và một <strong>Công cụ Soi Lời Nhắc (AI Prompt Inspector)</strong> độc đáo giúp bạn làm chủ hoàn toàn quá trình tạo bản dịch của máy học.
      </p>

      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Tiên Hiệp / Huyền Huyễn</span>
            <span class="cds--tag cds--tag--purple">Thể Loại</span>
          </div>
          <p>
            Giữ nguyên chuẩn mực âm Hán Việt cho các cảnh giới tu luyện (Luyện Khí, Trúc Cơ, Kim Đan, Nguyên Anh), các loại đan dược, pháp bảo và danh xưng môn phái; ngôn phong cổ kính nhưng gãy gọn.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Kiếm Hiệp / Cổ Trang</span>
            <span class="cds--tag cds--tag--blue">Thể Loại</span>
          </div>
          <p>
            Đậm chất võ hiệp Kim Dung, Cổ Long. Chuẩn xác xưng hô huynh đệ, sư đồ, chưởng môn; miêu tả thế võ, chiêu thức uyển chuyển, mạch lạc và giàu hình tượng võ thuật.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Đô Thị / Hiện Đại / Ngôn Tình</span>
            <span class="cds--tag cds--tag--cyan">Thể Loại</span>
          </div>
          <p>
            Ngôn ngữ đời thường tự nhiên, thoát hẳn lối dịch convert thô kệch; đại từ xưng hô anh/em, cậu/tớ linh hoạt theo quan hệ xã hội, bắt kịp phong cách trò chuyện hiện đại.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Báo Chí & Kỹ Thuật</span>
            <span class="cds--tag cds--tag--green">Thể Loại</span>
          </div>
          <p>
            Đòi hỏi độ chính xác tuyệt đối về thuật ngữ chuyên ngành; câu văn khách quan, đúng ngữ pháp tiếng Việt chuẩn mực xuất bản, không thêm thắt văn hoa mỹ miều.
          </p>
        </div>
      </div>

      <h3 class="cds--subsection-title">Cửa Sổ Soi Lời Nhắc (AI Prompt Inspector)</h3>
      <p>
        Đã bao giờ bạn tự hỏi: <em>“Tại sao AI lại dịch câu này ra từ kỳ lạ như vậy?”</em> 
        Nút <strong>Soi Prompt</strong> ở góc bảng AI giải tỏa hoàn toàn băn khoăn đó bằng cách mở ra toàn bộ nội dung mà phần mềm chuẩn bị gửi đến API:
      </p>
      <div class="cds--steps">
        <div class="cds--step-item">
          <div class="cds--step-num">1</div>
          <div class="cds--step-content">
            <h4>Lời Nhắc Hệ Thống (System Prompt)</h4>
            <p>Quy định vai trò dịch giả, phong cách hành văn và danh sách các điều cấm kỵ (ví dụ: không được dùng từ Hán Việt thô, không được bỏ bớt câu của tác giả).</p>
          </div>
        </div>
        <div class="cds--step-item">
          <div class="cds--step-num">2</div>
          <div class="cds--step-content">
            <h4>Bảng Thuật Ngữ Đi Kèm (Dynamic Glossary Injection)</h4>
            <p>Hệ thống tự động lọc các từ khóa, tên nhân vật xuất hiện trong câu hiện tại và chèn bảng dịch nghĩa bắt buộc để AI phải tuân theo 100%.</p>
          </div>
        </div>
        <div class="cds--step-item">
          <div class="cds--step-num">3</div>
          <div class="cds--step-content">
            <h4>Ngữ Cảnh Các Câu Liền Kề (Context Window)</h4>
            <p>Gửi kèm 2 câu trước và 1 câu sau để AI hiểu rõ ngữ cảnh câu chuyện, tránh hiện tượng dịch sai đại từ xưng hô (ông/bà/hắn/nàng).</p>
          </div>
        </div>
      </div>
    </section>

    <!-- CHƯƠNG 10 -->
    <section id="glossary-system" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>10</span>
        <span>Quản Lý Thuật Ngữ & Tên Riêng (Glossary System)</span>
      </h2>
      <p>
        Một tác phẩm đồ sộ với hàng trăm nhân vật và địa danh sẽ trở thành thảm họa nếu mỗi chương dịch tên một kiểu. 
        Hệ thống Thuật ngữ trong AuraTranslate được xây dựng theo mô hình 2 tầng thông minh:
      </p>

      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">1. Tầng Thuật Ngữ Toàn Cục (Global Glossary)</span>
            <span class="cds--tag cds--tag--purple">Áp Dụng Toàn Bộ</span>
          </div>
          <p>
            Chứa các quy ước xưng hô chung, các thuật ngữ tu tiên, kiếm hiệp hoặc kỹ thuật phổ biến áp dụng cho mọi cuốn sách bạn dịch trên máy tính.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">2. Tầng Thuật Ngữ Tác Phẩm (Project Glossary)</span>
            <span class="cds--tag cds--tag--blue">Riêng Từng Bộ Sách</span>
          </div>
          <p>
            Chỉ có hiệu lực trong cuốn sách đang mở. Chứa tên nhân vật chính, pháp bảo đặc hữu, tên các tông môn và địa danh cụ thể của tác phẩm đó.
          </p>
        </div>
      </div>

      <h3 class="cds--subsection-title">Tính Năng Khảo Sát Tự Động (Glossary Harvest)</h3>
      <p>
        Thay vì phải đọc trước cả bộ sách và ghi chép thủ công từng cái tên vào sổ tay, bạn chỉ cần mở công cụ <strong>Khảo sát Thuật ngữ (Harvest)</strong>:
      </p>
      <ul style="margin-left: 20px; margin-bottom: 20px; color: var(--cds-text-secondary); line-height: 1.8;">
        <li>AuraTranslate quét toàn bộ hàng triệu chữ của cuốn sách trong vài giây.</li>
        <li>Tự động thống kê các cụm danh từ riêng, tên nhân vật xuất hiện với tần suất cao nhất.</li>
        <li>Đưa danh sách vào <strong>Bảng Chờ Duyệt (Glossary Queue)</strong>: bạn chỉ việc gán nghĩa tiếng Việt chuẩn xác và bấm Duyệt một lần để áp dụng cho toàn bộ cuốn sách.</li>
      </ul>

      <h3 class="cds--subsection-title">Dải Gợi Ý Thuật Ngữ Nổi (Glossary Confirm Strip)</h3>
      <p>
        Khi bạn di chuyển con trỏ vào bất kỳ câu nào có chứa thuật ngữ đã lưu, dải gợi ý màu tím sẽ tự động nổi lên ngay trên ô dịch. Bạn chỉ cần nhấn phím tắt hoặc nhấp chuột để điền tên chuẩn vào bản dịch mà không sợ gõ sai chính tả.
      </p>
    </section>

    <!-- CHƯƠNG 11 -->
    <section id="tm-alignment" style="margin-bottom: 60px;">
      <h2 class="cds--section-title">
        <span>11</span>
        <span>Bộ Nhớ Dịch (Translation Memory - TM) & Căn Chỉnh Song Ngữ</span>
      </h2>
      <p>
        <strong>Bộ nhớ dịch (TM)</strong> lưu giữ toàn bộ các cặp câu gốc và câu dịch mà bạn đã hoàn thành. 
        Càng dịch nhiều, kho dữ liệu này càng đồ sộ và thông minh, giúp bạn không bao giờ phải dịch lại lần thứ hai cùng một câu tương tự.
      </p>

      <div class="cds--grid-cards">
        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Gợi Ý Khớp Tương Đồng (Fuzzy Match 70% - 100%)</span>
            <span class="cds--tag cds--tag--green">Tiết Kiệm Thời Gian</span>
          </div>
          <p>
            Nếu câu mới giống 100% với câu cũ trong quá khứ, máy sẽ tự động điền bản dịch cũ. Nếu giống 70% - 99%, dải gợi ý xanh lá sẽ hiển thị sự tương đồng và tô đậm các chữ khác biệt để bạn chỉ cần chỉnh lại vài từ là xong.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Bảng Kiểm Tra Sức Khỏe Bộ Nhớ (TM Health)</span>
            <span class="cds--tag cds--tag--cyan">Quản Lý Chất Lượng</span>
          </div>
          <p>
            Thống kê tổng số lượng cặp câu, phát hiện các trường hợp cùng một câu gốc nhưng có 2 cách dịch mâu thuẫn nhau, dọn dẹp các cặp câu rác để bộ nhớ luôn trong sạch và chuẩn xác.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Xuất / Nhập Định Dạng Quốc Tế (TMX)</span>
            <span class="cds--tag cds--tag--purple">Chuẩn Toàn Cầu</span>
          </div>
          <p>
            Hỗ trợ đầy đủ chuẩn quốc tế <code>.tmx</code> (Translation Memory eXchange). Bạn có thể dễ dàng chuyển kho dữ liệu dịch thuật của mình qua lại giữa AuraTranslate và SDL Trados, memoQ, OmegaT.
          </p>
        </div>

        <div class="cds--card">
          <div class="cds--card-header">
            <span class="cds--card-title">Công Cụ Căn Chỉnh Song Ngữ (Alignment Tool)</span>
            <span class="cds--tag cds--tag--blue">Tái Sử Dụng Sách Cũ</span>
          </div>
          <p>
            Khi bạn có 1 file sách gốc và 1 file bản dịch tiếng Việt có sẵn từ trước, công cụ này sẽ mở 2 văn bản cạnh nhau, tự động so khớp từng câu và cung cấp các nút <strong>Nối câu (Join)</strong>, <strong>Tách câu (Split)</strong> để tạo ra kho TM khổng lồ chỉ trong vài phút.
          </p>
        </div>
      </div>
    </section>
"""
