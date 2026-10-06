---
title: "Addendum — AuraTranslate"
status: draft
created: 2026-08-02
updated: 2026-08-02
---

# Addendum — AuraTranslate

Tài liệu này giữ lại chiều sâu do Ice cung cấp mà **không thuộc về product brief** (brief cần gọn 1–2 trang), nhưng **thuộc về các tài liệu downstream** — chủ yếu là PRD và Architecture.

---

## A. PRD v8.0 — Nguyên văn do Ice cung cấp (2026-08-02)

> Ghi chú: Đây là đầu vào gốc, được lưu nguyên trạng làm nguồn tham chiếu. Nội dung này là **đặc tả giải pháp**, sẽ được chưng cất xuống brief và mở rộng ở workflow `bmad-prd` / `bmad-architecture`.

**Tên dự án (theo PRD):** Ứng dụng Hỗ trợ Dịch thuật & Biên tập Đa ngôn ngữ (Premium AI Translator Workstation)

### 1. Yêu cầu Nền tảng & Công nghệ (Platform & Architecture)

- **Môi trường:** Desktop Application chạy Native trên cả macOS và Windows. Local-first architecture (dữ liệu lưu 100% trên máy người dùng, không phụ thuộc Cloud).
- **Ngôn ngữ & Kiến trúc:**
  - Rust làm lõi (Backend) xử lý luồng, quản lý file system, local database, Diff-engine và kết nối AI.
  - Tauri kết hợp Web Frontend (React/Vue/TypeScript) để quản lý giao diện đồ họa phức tạp.
- **Cơ sở dữ liệu cục bộ:** SQLite quản lý Project, Segment (đoạn dịch), History (lịch sử) và Glossary.

### 2. Kiến trúc Giao diện (UI/UX) & Workspace

- **Giao diện Đa khung (Multi-Panel Workspace):** Toàn bộ 4 Panel nằm trong một cửa sổ ứng dụng duy nhất.
  - Hỗ trợ Kéo thả (Drag & Drop) để dock/undock, chia tab. Thay đổi kích thước linh hoạt.
  - **Panel 1 — Văn bản Nguồn:** hiển thị text gốc (Anh/Trung) và Tab Hán Việt (chuyển đổi hoặc xem song song).
  - **Panel 2 — Tra cứu (Từ điển & Thuật ngữ):** hiển thị chi tiết giải nghĩa, ngữ cảnh, ví dụ cách dùng.
  - **Panel 3 — Bản dịch AI:** hiển thị kết quả AI xử lý.
  - **Panel 4 — Editor (Soạn thảo):** khu vực biên tập và hoàn thiện bản dịch.
- **Trải nghiệm Nâng cao (UX):**
  - **Cuộn đồng bộ (Sync Scrolling):** đồng bộ vị trí cuộn giữa Panel 1, 3 và 4. Có công tắc Bật/Tắt.
  - **Auto-Lookup:** bôi đen cụm từ ở Panel 1, 3, 4 → kết quả lập tức hiện ở Panel 2 (không cần copy/paste, hiển thị đầy đủ ngữ cảnh & cách dùng).
  - **Global Hotkeys:** bộ phím tắt điều khiển toàn app (ví dụ: dịch câu, focus panel).

### 3. Hệ thống Quản lý Dự án (Project Management)

- **Kiến trúc File-based:** dự án lưu thành thư mục/tập tin cục bộ (`.atproj` hoặc tương tự).
- **Auto-save & Versioning:** tự động lưu định kỳ các phiên bản dịch mà không làm gián đoạn UI.
- **Phân cấp Dữ liệu (Scope Management):**
  - **Global Scope:** cấu hình AI, Prompt chung, từ điển cá nhân dùng chung cho mọi dự án.
  - **Project Scope:** bộ thuật ngữ riêng (tên nhân vật, địa danh, công pháp...) và Prompt riêng chỉ áp dụng cho dự án hiện tại. Ngôn ngữ nguồn (Source Language) được cài đặt tĩnh cho từng dự án để tối ưu thuật toán AI.

### 4. Hệ thống AI & Xử lý Ngôn ngữ

- **Kiến trúc AI Mở:** BYOK (nhập API Key riêng) và hỗ trợ kết nối Local LLM (Ollama, LM Studio).
- **Custom Prompts:** thiết lập prompt theo thể loại (tiên hiệp, khoa học...) và quy chuẩn dịch.
- **AI Proofreader (Trợ lý Kiểm lỗi):**
  - Tự động quét chính tả, ngữ pháp bản dịch của người dùng.
  - Tự động đối chiếu text dịch và text gốc, Highlight trên giao diện các đoạn nghi ngờ dịch sai, dịch thoát nghĩa hoặc cấu trúc câu tối nghĩa.
- **Quản lý Thuật ngữ Thông minh (Smart RAG Injector):**
  - Thuật toán (chạy trên Rust) tự động đọc câu gốc trước khi gửi API, tìm kiếm các thuật ngữ (Glossary) xuất hiện trong câu đó để chèn động (Dynamic Inject) vào Prompt, ép AI ưu tiên sử dụng.
- **Động cơ Nhận diện Đa ngôn ngữ (Language-aware Matching):**
  - **Tiếng Trung:** áp dụng thuật toán Khớp chính xác (Exact Match).
  - **Tiếng Anh:** áp dụng thuật toán Khớp mờ (Fuzzy Match) ở cấp độ hình thái từ (Stemming/Lemmatization) để nhận diện các biến thể của từ.

### 5. Quy trình Làm việc nhóm & Review (Collaboration & Review Workflow)

- **Hệ thống Trích xuất (Export):**
  - Xuất file `.docx` định dạng Bảng 2 cột (cột trái: text gốc — cột phải: text dịch đối xứng).
  - Xuất file `.md` hoặc text thuần (bảo lưu các liên kết hình ảnh và Alt-text).
- **Hệ thống Đối chiếu & Học hỏi (Import & Diff Viewer):**
  - Import file `.docx` / `.md` (đã được Reviewer chỉnh sửa) vào lại dự án hiện tại. Phân tích cấu trúc đoạn (Structural Index Mapping) để khớp dữ liệu.
  - **Chế độ Review Mode UI:** hệ thống chuyển sang giao diện chỉ hiển thị 2 cửa sổ (side-by-side).
    - Cửa sổ trái: bản dịch cũ của người dùng.
    - Cửa sổ phải: bản dịch đã import (của Reviewer).
  - **Highlight Khác biệt:** ứng dụng tự động ẩn text gốc, dùng thuật toán Diff để bôi màu nổi bật (đỏ/xanh) những từ/cụm từ bị xóa, thêm hoặc sửa đổi giữa hai bản dịch. Người dùng chỉ cần lướt qua để đối chiếu và rút kinh nghiệm.

### 6. Dữ liệu Từ điển

- **Từ điển Nhúng (Offline):** dữ liệu Anh-Việt, Trung-Việt, Hán Việt nhúng sẵn vào app. Tra cứu siêu tốc không cần mạng.
- **Từ điển Cá nhân/Dự án (Glossary):** thêm/sửa thuật ngữ. Cung cấp dữ liệu cho hệ thống Smart RAG Injector.

---

## B. Bối cảnh cạnh tranh — khảo sát 2026-08-02

Khảo sát nhanh qua tìm kiếm web. Đây là nguyên liệu cho phần "Khác biệt ở đâu" của brief, chưa phải nghiên cứu thị trường đầy đủ (nếu cần sâu hơn: `bmad-market-research`).

### B.1 Đương nhiệm tại Việt Nam: QuickTranslator (QT)

Công cụ mặc định của giới convert/editor truyện Trung–Việt suốt hơn một thập kỷ.

| Đặc điểm | Ghi nhận |
|---|---|
| Giá | Miễn phí, không cần cài đặt (giải nén là chạy) |
| Nền tảng | **Chỉ Windows** (`.exe`) |
| Phiên bản mới nhất | **2022** — không còn phát triển tích cực |
| Có sẵn | Nhập text Trung, output Việt, **Hán Việt**, từ điển **VietPhrase**, **Name database** |
| Không có | AI/LLM, kiểm lỗi, diff/review, quản lý dự án, macOS |

**Ý nghĩa:** Panel 1 + Panel 2 trong PRD v8.0 gần như tái hiện QT. Phần còn lại (AI, Proofreader, Diff Viewer, Project Management, macOS) là phần QT bỏ trống. Cộng đồng đã quen với mô hình tra cứu của QT — đây vừa là lợi thế (không phải dạy lại) vừa là ràng buộc (lệch quá sẽ bị từ chối).

### B.2 Làn sóng công cụ dịch truyện AI 2026

Thị trường đã đông. Các sản phẩm đáng chú ý:

- **LLM Novel Translator** — extension Chrome, BYOK, glossary tự sinh theo series (mã nguồn mở trên GitHub).
- **Lexilit**, **AbsoluteMystery**, **BookTranslator**, **ainoveltranslation.com** — dịch vụ web, glossary quản lý theo series.

**Mẫu số chung của nhóm này:**
- Chạy trên web/cloud, không local-first.
- **Glossary tự sinh tự động** — công cụ tự phát hiện tên riêng, thuật ngữ và tự cập nhật.
- Định vị "dịch nhanh để đọc" — tối ưu tốc độ và số lượng, người dùng là **độc giả**.

**Vấn đề nghiệp vụ mà cả ngành công nhận:** *consistency drift* — cùng một nhân vật bị dịch thành nhiều tên khác nhau qua các chương. Thuật ngữ tiên hiệp (cảnh giới tu luyện, công pháp) không có trong từ điển chuẩn.

### B.3 Bối cảnh CAT tool truyền thống

- **OmegaT** — CAT tool mã nguồn mở lâu đời: translation memory, fuzzy matching, glossary, concordance.
- **OPUS-CAT** — máy dịch neural chạy offline, cắm vào các CAT tool.
- Nhận định chung của ngành: LLM cho ra bản dịch trôi chảy nhưng **thiếu tính nhất quán, thiếu cưỡng chế glossary và thiếu theo dõi thay đổi** — nên hoạt động tốt nhất khi nằm *bên trong* một môi trường CAT, chứ không đứng một mình.

> Nhận định này ủng hộ trực tiếp luận điểm sản phẩm của AuraTranslate: giá trị nằm ở **môi trường làm việc bao quanh AI**, không phải ở bản thân AI.

### B.4 Khoảng trống trong PRD v8.0 so với chuẩn ngành

Hai điểm cần Ice quyết định — chưa phải lỗi, nhưng là lựa chọn chưa được nêu rõ:

1. **Không có Translation Memory (TM).** Mọi CAT tool đều có TM: tái sử dụng câu đã dịch qua fuzzy match. PRD có `Segment` và `History` trong SQLite nhưng không có cơ chế tái sử dụng. Truyện dài lặp lại câu chữ rất nhiều — đây có thể là khoảng trống lớn, hoặc là quyết định cắt bỏ có chủ đích.
2. **Glossary hoàn toàn thủ công.** Đối thủ tự sinh glossary. PRD yêu cầu người dùng tự nhập tên nhân vật, địa danh, công pháp. Với truyện 2000 chương thì đây là gánh nặng khởi động đáng kể.

---

## C. Câu hỏi mở / Rủi ro đã nhận diện

| # | Vấn đề | Trạng thái |
|---|---|---|
| 1 | Tên "Premium AI Translator Workstation" mâu thuẫn với định hướng mã nguồn mở | ✅ **Đóng** — chốt tên `AuraTranslate` |
| 2 | Không có Translation Memory — thiếu sót hay chủ đích? | ✅ **Đóng** — Ice xác nhận là thiếu sót, cần bổ sung (xem D.2) |
| 3 | Glossary thủ công vs. tự sinh — gánh nặng khởi động | 🔄 Đang xử lý — đề xuất tại D.4 |
| 4 | Thư mục cha `LocalSites/addon/` không khớp kiến trúc desktop | ✅ **Đóng** — thư mục không mang ý nghĩa |
| 5 | Phạm vi: chuyên truyện mạng hay dịch thuật tổng quát? | ✅ **Đóng** — Ice chốt **tổng quát**; ví dụ tiên hiệp chỉ là minh hoạ |
| 6 | Vòng lặp học hỏi đứt gãy: reviewer góp ý nhưng người dịch không đọc lại | 🔴 **Mở** — rủi ro hành vi, xem D.1 |
| 7 | Phạm vi mở rộng sang quản lý tài liệu / thư viện | ✅ **Đóng** — là xương sống, xem D.5 |
| 8 | "Tổng quát" ở mức nào | ✅ **Đóng** — mọi lĩnh vực, cặp ngôn ngữ cố định **Anh/Trung → Việt** |
| 9 | Thư viện là xương sống ⇒ PRD v8.0 thiếu hẳn tầng này (chưa có mô hình dữ liệu, chưa có UI) | 🔴 **Mở** |

---

## D. Quy trình hiện tại & Đề xuất giải pháp

### D.1 Quy trình hiện tại của Ice (ghi nhận 2026-08-02)

| Giai đoạn | Thực tế hôm nay | Nỗi đau |
|---|---|---|
| Chuẩn bị | Mở QuickTranslator với **4–5 cửa sổ** vì mỗi file dịch là một cửa sổ riêng | Không có khái niệm "dự án"; quản lý cửa sổ thủ công; QT không chạy macOS |
| Dịch | Tự dịch, **đối chiếu từ điển thủ công** | **Một chương mất nửa buổi đến trọn một ngày** |
| Bàn giao | Xuất lên **Google Docs** | Rời khỏi công cụ, mất liên kết với bản gốc |
| Review | Người khác vào Google Docs xem và góp ý | — |
| Học hỏi | **Người dịch ít khi xem lại** | 🔴 **Vòng lặp đứt.** Công sức review bị lãng phí; sai lầm lặp lại vô hạn |

> **Nhận định:** Ô đỏ cuối bảng là phát hiện quan trọng nhất của phiên Discovery. PRD mục 5 (Diff Viewer) được sinh ra để chữa nó — nhưng đó là **giải pháp công cụ cho một vấn đề hành vi**, cần thiết kế thận trọng.

### D.2 Đề xuất: Translation Memory (TM)

Ice xác nhận đây là thiếu sót. Đề xuất bổ sung, tận dụng `Segment` + `History` đã có trong SQLite:

- **Ghi tự động:** mỗi khi người dùng xác nhận một segment ở Panel 4, cặp (nguồn → đích) vào TM. Không có thao tác thủ công nào.
- **Gợi ý khi dịch:** segment nguồn mới đến → tìm fuzzy match trong TM → hiển thị các bản dịch cũ tương tự kèm % khớp.
- **Thuật toán khớp, phân theo ngôn ngữ** (khớp với "Language-aware Matching" đã có ở PRD mục 4):
  - Tiếng Trung: n-gram ký tự (không có ranh giới từ).
  - Tiếng Anh: token n-gram sau stemming/lemmatization.
- **Phạm vi kép:** TM riêng theo dự án + TM chung toàn cục, tương ứng Global/Project Scope ở PRD mục 3.

> **Điểm cộng hưởng:** TM có thể **nạp thẳng vào Smart RAG Injector** — ngoài việc chèn thuật ngữ, chèn luôn *"những câu tương tự trước đây được dịch thế này"*. AI học phong cách của chính người dùng thay vì áp phong cách chung. Đây là chỗ TM và RAG Injector nhân giá trị cho nhau thay vì là hai tính năng rời.

### D.3 Đề xuất: tái sử dụng Segment / History

Ice xác nhận "chưa nghĩ tới". Ba tầng tái sử dụng, độc lập nhau:

1. **Khớp tuyệt đối (100%):** segment y hệt đã dịch → điền sẵn, đánh dấu để người dùng xác nhận.
2. **Khớp mờ:** hiển thị bản dịch cũ + diff phần khác biệt để sửa nhanh thay vì dịch lại.
3. **Concordance:** tra ngược "cụm từ này trước đây tôi dịch thế nào?" — tìm trong toàn bộ TM, đưa kết quả vào Panel 2.

### D.4 Đề xuất: tự sinh Glossary

Thay "tự động quyết" bằng **"tự động đề xuất, người dùng duyệt"** — giữ quyền kiểm soát biên tập, bỏ gánh nặng gõ tay.

- **Quét khi nhập tài liệu:** tìm ứng viên thuật ngữ = chuỗi lặp lại nhiều lần **và** không có trong từ điển nhúng.
  - Tiếng Trung: chuỗi ký tự lặp không có trong từ điển; đối chiếu danh sách họ phổ biến để đoán tên người.
  - Tiếng Anh: cụm viết hoa không đứng đầu câu.
- **Duyệt hàng loạt:** đưa ra danh sách xếp theo tần suất, người dùng duyệt/bỏ nhanh — không phải gõ.
- **🔑 Thu hoạch từ bản review (đề xuất mạnh nhất):** khi import bản Reviewer sửa, nếu phát hiện reviewer **nhất quán** đổi thuật ngữ X thành Y, tự đề xuất thêm cặp đó vào Glossary.

> **Vì sao điểm cuối quan trọng:** nó vá được rủi ro #6. Ngay cả khi người dịch **không** đọc lại bản review, công cụ vẫn hấp thụ bài học và ép AI dùng đúng thuật ngữ ở lần sau. Vòng lặp học hỏi được đóng lại ở tầng hệ thống thay vì trông chờ vào kỷ luật con người.

### D.5 Thư viện là xương sống — hệ quả kiến trúc

Ice chốt: quản lý tài liệu **ngang hàng** với dịch thuật. Luồng vào ứng dụng đổi:

```
PRD v8.0:      Mở app  →  Workspace 4 panel  (trung tâm là màn hình dịch)
Chốt mới:      Mở app  →  THƯ VIỆN  →  chọn tài liệu  →  Workspace 4 panel
```

**Hệ quả cần xử lý ở PRD/Architecture:**

1. PRD v8.0 **không có tầng này** — không có mô hình dữ liệu cho thư viện, không có đặc tả UI. Đây là khoảng trống lớn nhất còn lại.
2. Mô hình dữ liệu SQLite phải mở rộng: hiện có `Project / Segment / History / Glossary`. Thư viện cần thêm khái niệm tài liệu, tiến độ, phân loại, metadata, và tìm kiếm xuyên dự án.
3. Kiến trúc file-based `.atproj` (PRD mục 3) cần dung hoà với một chỉ mục thư viện tập trung — dự án nằm rải trên đĩa nhưng thư viện phải thấy toàn cảnh.
4. **Nguồn gốc nhu cầu:** thay thế cảnh mở 4–5 cửa sổ QuickTranslator rời rạc. Nỗi đau không chỉ là dịch chậm, mà là **không nắm được mình đang có những gì**.

### D.6 Phạm vi ngôn ngữ đã chốt

- **Cặp ngôn ngữ:** Anh → Việt, Trung → Việt. Cố định.
- **Lĩnh vực:** không giới hạn — truyện, tài liệu kỹ thuật, báo chí, hợp đồng.
- **Hệ quả:** Custom Prompts theo thể loại (PRD mục 4) từ tính năng phụ trở thành **cơ chế chính** để một công cụ phục vụ nhiều lĩnh vực. Từ điển nhúng Anh-Việt / Trung-Việt / Hán Việt giữ nguyên như PRD mục 6.

### D.7 Đặc tả Thư viện (Ice chốt 2026-08-02)

**Cấu trúc cấp cao:** ứng dụng có **2 khung/chế độ chính**.

| Khung | Vai trò |
|---|---|
| 1 — Dịch thuật | Workspace 4 panel như PRD v8.0 mục 2 |
| 2 — Thư viện | Kho tài liệu **đã dịch xong** |

**Thư viện dùng để làm gì** — ba mục đích, Ice nêu theo thứ tự này:

1. Vào **xem / đọc lại** bài viết, truyện mình đã dịch.
2. **Mở ra dịch lại** (quay về khung 1).
3. Tra cứu và lưu trữ.

> **Phát hiện quan trọng:** mục đích số 1 khiến Thư viện không chỉ là màn hình quản lý — nó là **trải nghiệm đọc**. Người dùng quay lại để *thưởng thức* thành quả, không chỉ để tìm file. Hàm ý thiết kế: cần chế độ đọc thoải mái (typography, bố cục), không chỉ là bảng dữ liệu.

**Tính năng đã chốt:**

| Tính năng | Ghi chú |
|---|---|
| Tìm kiếm | Ice nhấn mạnh "chắc chắn cho tìm kiếm" — bắt buộc |
| Danh sách tiến độ | Tiến độ dịch từng tài liệu |
| Ảnh bìa | Khi tài liệu có bìa |
| Trạng thái vòng đời | Đã dịch xong / Đang dịch / Tạm ngưng |

### D.8 Mô hình cộng tác

- Nhóm review **có thể** dùng app hoặc không — **hoàn toàn tuỳ chọn, không bắt buộc**.
- **Hệ quả bắt buộc:** luồng export/import `.docx` (PRD mục 5) không phải tính năng phụ. Nó là **cầu nối duy nhất** tới các reviewer không bao giờ rời Google Docs. Không có nó, AuraTranslate cắt đứt Ice khỏi nhóm của mình.
- Hàm ý: AuraTranslate là ứng dụng **một người dùng**, cộng tác diễn ra qua trao đổi file, không qua tài khoản hay đồng bộ đám mây. Điều này nhất quán với kiến trúc local-first.

### D.9 Tín hiệu thành công (Ice nêu — còn định tính)

| Tín hiệu | Loại |
|---|---|
| Dễ dàng tra cứu | Trải nghiệm |
| Dễ dàng lưu trữ | Trải nghiệm |
| Chất lượng bản dịch tốt hơn | Chất lượng |
| Dịch nhanh hơn | Hiệu suất |
| Ít sai sót hơn | Chất lượng |
| Không bị lỗi chính tả | Chất lượng |

> **Quan sát:** 4/6 tín hiệu thuộc về **chất lượng và trải nghiệm**, chỉ 1 về tốc độ. Điều này củng cố định vị: AuraTranslate không cạnh tranh ở "dịch nhanh" (mảnh đất của các công cụ AI cloud) mà ở **"dịch chuẩn và không mất dấu"**.
>
> **Baseline đo được duy nhất hiện có:** 1 chương = nửa buổi đến trọn một ngày.

### D.10 Vấn đề còn để ngỏ có chủ ý

**Nguyên nhân gốc của việc không xem lại bản review.** Ice chọn không trả lời. Brief sẽ ghi nhận hiện tượng như một **dữ kiện quan sát được** mà không suy diễn động cơ.

Hệ quả cho thiết kế — cần xử lý ở PRD:

- Không thể xác nhận Diff Viewer (PRD mục 5) sẽ được sử dụng trên thực tế.
- Vì vậy, cơ chế **thu hoạch thuật ngữ tự động từ bản review** (D.4) nên được coi là **đường bảo hiểm**: nó tạo giá trị ngay cả khi người dùng không bao giờ mở Diff Viewer.
- Khuyến nghị: đừng đặt Diff Viewer làm tính năng chủ lực của v1 cho tới khi có bằng chứng nó được dùng.

---

## E. Embedded Dictionary — khảo sát nguồn dữ liệu (2026-08-02)

Ice nêu hai yêu cầu: **(1)** bộ từ điển nhúng offline là **bắt buộc**, không phải tuỳ chọn; **(2)** cần tìm nguồn chuẩn nhất hiện có, đặc biệt lưu ý từ điển tiếng Trung vì nhiều nguồn gốc Trung Quốc có thể giải nghĩa sai.

### E.1 English → Vietnamese

| Nguồn | Nội dung | Giấy phép | Đánh giá |
|---|---|---|---|
| **Free Vietnamese Dictionary Project** (Hồ Ngọc Đức) | Anh-Việt, Việt-Anh và nhiều cặp khác | **GPL v2+** | Nguồn gốc chuẩn mực, gần như mọi từ điển Anh-Việt mã nguồn mở đều dẫn về đây |
| **OVDP** (Open Vietnamese Dictionary Project) | Kế tục không chính thức của FVDP, định dạng StarDict | **GPL** | Đóng gói sẵn, dễ tích hợp |
| `dynamotn/stardict-vi` | Bản StarDict đóng gói từ OVDP | GPL (kế thừa) | Tiện cho tích hợp nhanh |
| ~~ECDICT~~ | English → **Chinese** | — | ❌ Không phải Anh-Việt. Ghi lại để tránh nhầm |

> Đúng như Ice dự đoán: phía tiếng Anh dễ. Toàn bộ hệ sinh thái quy về FVDP của Hồ Ngọc Đức.

### E.2 Chinese → Vietnamese — nơi có rủi ro thật

Không có một bộ từ điển đơn lẻ nào vừa chuẩn, vừa đủ, vừa dùng được về mặt pháp lý. Cần **xếp lớp nhiều nguồn**.

| Tầng | Nguồn | Giấy phép | Ghi chú |
|---|---|---|---|
| Âm Hán Việt (ký tự) | **Unihan** (Unicode Consortium) | Unicode License — dễ dãi | Nền tảng chuẩn quốc tế cho âm đọc |
| Tự điển ký tự | **Thiều Chửu** (1942) | Nhiều khả năng đã hết bản quyền — **cần xác minh** | Được coi là bộ chuẩn mực về Hán Việt suốt hơn 60 năm |
| Tự điển ký tự (hiện đại) | **Trần Văn Chánh** (NXB Trẻ, 1999) | **Còn bản quyền** | Đầy đủ và hiện đại hơn, nhưng **không dùng được nếu không xin phép** |
| Từ/cụm từ | **VietPhrase** | Cộng đồng — **cần xác minh** | Chính bộ dữ liệu đứng sau QuickTranslator; hợp ngữ cảnh dịch thực tế |
| Đối chiếu chéo | **CC-CEDICT** | **CC-BY-SA 4.0** — an toàn | 124.727 mục (bản 2026-07-22), có quy trình biên tập duyệt |
| Hỗn hợp Hán-Nôm | `Trannosaur/published_dicts` | **CC-BY-SA 4.0** | Trộn định nghĩa tiếng Việt hiện đại với Hán-Nôm |

### E.3 Đề xuất giải quyết mối lo "giải nghĩa sai"

Vấn đề Ice nêu là có thật, nhưng **không giải được bằng cách chọn đúng một bộ từ điển** — vì không bộ nào đúng hoàn toàn. Đề xuất ba tầng:

1. **Xếp lớp, không đặt cược một nguồn.** Thiều Chửu + Unihan cho tầng ký tự (cổ điển, đáng tin, có khả năng đã hết bản quyền); VietPhrase cho tầng từ/cụm (đã được cộng đồng dịch giả kiểm chứng qua thực tế).
2. **CC-CEDICT làm ý kiến thứ ba.** Khi các nguồn tiếng Việt mâu thuẫn hoặc thiếu mục từ, hiển thị nghĩa tiếng Anh từ CC-CEDICT. Bộ này có quy trình biên tập duyệt và là chuẩn de-facto quốc tế — nó bắt được đúng loại sai sót mà Ice lo ngại.
3. **🔑 Luôn hiện nguồn của mỗi định nghĩa trong Lookup panel.** Đây mới là câu trả lời thật. Không giấu nguồn nào nói gì — để người dịch tự phán xét khi các nguồn bất đồng.

> Điểm 3 nhất quán tuyệt đối với định vị sản phẩm: **người dịch quyết định, công cụ cung cấp bằng chứng.** Một công cụ hợp nhất các định nghĩa thành một câu trả lời duy nhất chính là công cụ giấu đi sai sót.

### E.4 ⚠️ Rủi ro bản quyền — cần quyết định sớm

| Nguồn | Giấy phép | Hệ quả với dự án mã nguồn mở |
|---|---|---|
| FVDP / OVDP | **GPL v2+** | **Có tính lan truyền.** Đóng gói dữ liệu GPL có thể buộc AuraTranslate phải phát hành theo GPL |
| CC-CEDICT | CC-BY-SA 4.0 | An toàn — cần ghi nguồn và chia sẻ lại phần dữ liệu phái sinh cùng giấy phép |
| Unihan | Unicode License | An toàn |
| Thiều Chửu (1942) | Cần xác minh tình trạng | Nhiều khả năng đã hết bản quyền, nhưng **phải kiểm chứng trước khi phát hành** |
| Trần Văn Chánh (1999) | Còn bản quyền | không Loại khỏi bản phát hành |
| VietPhrase | Không rõ | Phải xác minh nguồn gốc dữ liệu |

**Hai việc phải làm trước khi viết dòng code nhúng từ điển đầu tiên:**

1. Xác minh tình trạng bản quyền của Thiều Chửu và nguồn gốc dữ liệu VietPhrase.
2. Quyết định giấy phép của chính AuraTranslate. Nếu dùng dữ liệu GPL, dự án nhiều khả năng phải là GPL — Ice đã chọn mã nguồn mở nên điều này có thể chấp nhận được, nhưng phải là **quyết định có ý thức**, không phải hệ quả tình cờ.

### E.5 ✅ Quyết định: AuraTranslate phát hành theo GPL (2026-08-02)

Ice chốt **GPL** làm giấy phép dự án. Hệ quả:

| Nguồn dữ liệu | Trạng thái sau quyết định |
|---|---|
| FVDP / OVDP (Anh-Việt) | ✅ **Dùng được không vướng mắc** — GPL tương thích GPL |
| CC-CEDICT (CC-BY-SA 4.0) | ✅ Dùng được; cần ghi nguồn và giữ share-alike cho phần dữ liệu phái sinh |
| Unihan | ✅ Dùng được |
| Thiều Chửu (1942) | ⚠️ Vẫn cần xác minh tình trạng bản quyền |
| VietPhrase | ⚠️ Vẫn cần truy nguồn gốc dữ liệu |
| Trần Văn Chánh (1999) | không Loại — còn bản quyền |

**Hệ quả kèm theo cần ghi nhớ ở PRD/Architecture:**

- Mọi bản phái sinh của AuraTranslate buộc phải mở mã nguồn theo GPL. Đây trở thành một phần định vị sản phẩm, không chỉ là chi tiết pháp lý.
- Cần rà soát tính tương thích GPL của toàn bộ crate Rust và thư viện frontend sẽ dùng. Phần lớn hệ sinh thái Rust là MIT/Apache-2.0 — tương thích một chiều với GPL, nên an toàn.
- Cần kèm văn bản giấy phép và ghi công (attribution) cho từng bộ từ điển trong bản phát hành.

---

## F. Embedded Dictionary — bộ nguồn đã chốt (2026-08-02)

Cập nhật sau khi Ice cung cấp ảnh chụp ứng dụng *Từ điển Hán Việt v7.3.2* và đặt câu hỏi về **từ loại, cách dùng theo từng từ loại, và ví dụ**. Chi tiết nghiên cứu đầy đủ nằm ở Phụ lục A của `technical-auratranslate-tauri-rust-local-first-research-2026-08-02.md`.

### F.1 Khoảng trống đã phát hiện

Đề xuất trước đó (Thiều Chửu + Unihan + CVDICT + CC-CEDICT) **không nguồn nào có hệ thống từ loại hay ví dụ cách dùng có cấu trúc**. CC-CEDICT tuyên bố rõ trong tài liệu của mình là *"human readable descriptive dictionary, not a resource intended for machine processing"* — cố ý không gắn nhãn từ loại.

Điều đó khiến bộ nguồn cũ **không thực hiện được lời hứa của Panel 2** trong PRD v8.0: *"hiển thị chi tiết giải nghĩa, ngữ cảnh, ví dụ cách dùng"*.

### F.2 Bộ nguồn Ice đã chốt

| Lớp | Nguồn | Giấy phép | Vai trò |
|---|---|---|---|
| **Từ loại + cách dùng + ví dụ** | **kaikki.org / Wiktextract** | **CC-BY-SA + GFDL** ✅ | Nội dung có cấu trúc cho Panel 2. Mỗi bản ghi JSON mô tả **một part of speech** của một từ, kèm usage examples, lemma, dạng biến cách, quan hệ từ vựng–ngữ nghĩa |
| Ký tự | **Thiều Chửu** (1942) | Phạm vi công cộng *(cần xác minh bản cụ thể)* | Tự điển ký tự chuẩn mực |
| Âm Hán Việt | **Unihan** (Unicode Consortium) | Unicode License ✅ | Nền tab Hán Việt |
| Từ và cụm từ ZH→VI | **CVDICT** (`ph0ngp/CVDICT`) | **CC-BY-SA 4.0** ✅ | Hơn 122.000 mục, có cả từ hiện đại |
| Từ và cụm từ theo lối dịch thực tế | **VietPhrase** | ❓ Không xác định — **lớp gỡ rời được** | Cách cộng đồng dịch giả thực sự dịch, tích luỹ hơn một thập kỷ |
| Đối chiếu chéo | **CC-CEDICT** | **CC-BY-SA 4.0** ✅ | Ý kiến thứ ba, có quy trình biên tập duyệt |
| Ngữ liệu kinh điển | **Cổ hán văn** — Tam tự kinh, Thiên tự văn, Bách gia tính | Văn bản gốc thuộc phạm vi công cộng | Trích dẫn minh hoạ cách dùng cổ văn |

### F.3 Hai quyết định ngầm cần ghi rõ

**① Không dùng Hán Việt Từ Điển Trích Dẫn.** Đây là nguồn duy nhất có hệ thống từ loại đầy đủ *và* cấu trúc định nghĩa/ví dụ/trích dẫn tách bạch, nhưng **© Đặng Thế Kiệt, Paris 2006–2009 — còn bản quyền**. Ice chọn đi đường hoàn toàn có giấy phép mở thay vì phụ thuộc vào việc xin phép. Hệ quả: chất lượng chú giải từ loại sẽ **kém hơn HVTĐTD**, đổi lại dự án không có điểm phụ thuộc pháp lý nào. Nếu sau này muốn nâng chất lượng, liên hệ `dang.thekiet2022@yahoo.com` vẫn là phương án mở — đã có tiền lệ tác giả cho phép.

**② Không dùng Free Vietnamese Dictionary Project (FVDP/OVDP).** kaikki.org phủ luôn cặp Anh–Việt và còn kèm từ loại, nên nguồn cũ trở nên thừa.

> **Hệ quả đáng chú ý:** FVDP mang giấy phép **GPL v2+ có tính lan truyền**, và trước đây chính nó là lý do buộc phải chọn GPL. Bỏ FVDP đi thì **toàn bộ dữ liệu còn lại là CC-BY-SA / phạm vi công cộng / Unicode License — không có gì buộc dự án phải theo GPL nữa.**
>
> Ice vẫn giữ GPL. Nay đó là **lập trường chủ động**, không phải hệ quả kỹ thuật. Điều này nên được nói rõ trong tài liệu dự án.

**③ VietPhrase — ĐÃ XÁC NHẬN (2026-08-02).** Ice xác nhận tường minh: VietPhrase **được bổ sung vào bộ nguồn**. Không còn là giả định.

Vai trò của nó khác hẳn các lớp còn lại. kaikki.org, CVDICT và CC-CEDICT cho biết một từ **có nghĩa gì**; VietPhrase cho biết **cộng đồng dịch giả thực sự dịch nó ra sao** — thứ chỉ tích luỹ được qua hàng nghìn giờ dịch thật, không sinh ra từ dữ liệu từ điển.

Ràng buộc kiến trúc kèm theo: **đóng gói như một lớp tách rời được**, có ghi công rõ là dữ liệu cộng đồng không xác định được tác giả, kèm chính sách gỡ bỏ nếu chủ sở hữu lên tiếng. Nếu phải gỡ, sản phẩm vẫn hoạt động đầy đủ trên các lớp có giấy phép.

### F.4 Hệ quả với mô hình dữ liệu — thay đổi đáng kể

Schema SQLite cho từ điển **không thể chỉ là `(từ, nghĩa, nguồn)`**. Cần tối thiểu:

```
từ khoá  →  [ nguồn, từ loại, nghĩa, ví dụ[], trích dẫn[], ghi chú ]
```

- **Một từ có nhiều từ loại** → nhiều bản ghi, không phải một chuỗi nghĩa gộp
- **Ví dụ gắn với từng từ loại**, không gắn với cả từ
- **Trích dẫn** là trường riêng, khác ví dụ: trích dẫn có xuất xứ văn bản
- **Nguồn** là trường bắt buộc trên mọi bản ghi

> Panel 2 vì vậy hiển thị một **bản ghi có cấu trúc**, không phải một đoạn văn bản. Ví dụ: cùng một chữ dùng như động từ và như phó từ phải hiện thành hai mục riêng biệt, mỗi mục có ví dụ và nguồn riêng.
>
> Đây là thay đổi so với giả định ngầm trong PRD v8.0 và trong phần Integration Patterns của báo cáo nghiên cứu. **Phải phản ánh vào PRD và Architecture.**

### F.5 Việc còn phải làm

1. Xác minh tình trạng bản quyền của **bản Thiều Chửu số hoá cụ thể** sẽ dùng — bản gốc 1942 nhiều khả năng đã thuộc phạm vi công cộng, nhưng bản số hoá có thể kèm tuyên bố quyền riêng.
2. Chọn bản **Cổ hán văn thuộc phạm vi công cộng** — văn bản gốc (Tam tự kinh, Thiên tự văn, Bách gia tính) đã rất cổ, nhưng **bản có chú giải của người biên soạn hiện đại thì không**.
3. Đánh giá **độ phủ thực tế của kaikki.org cho cặp Trung–Việt và Anh–Việt** — cần đo trên dữ liệu thật ở Giai đoạn 0, vì đây nay là nguồn chính của Panel 2.
4. Ghi công đầy đủ từng nguồn theo yêu cầu CC-BY-SA, và giữ share-alike cho phần dữ liệu phái sinh.
