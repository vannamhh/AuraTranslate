# Epic 4 Context: AI mở & Smart RAG Injector

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Epic này mở cửa AI dịch cho người dùng tự mang chìa khoá của mình — một API key BYOK hoặc một mô hình local (Ollama/LM Studio) qua cùng một chỗ cấu hình — và cho gọi dịch từng segment hoặc theo lô, kết quả chảy dần và huỷ được giữa chừng. Trước mỗi lời gọi, hệ thống tự chèn các thuật ngữ Glossary đã chốt xuất hiện trong câu, và người dùng luôn mở xem được nguyên văn prompt cuối cùng đã gửi — hộp đen phải mở được. Kết quả AI chỉ nằm ở panel riêng, không bao giờ tự chảy vào Editor: người dịch vẫn là người duy nhất quyết định đưa gì vào bản dịch. Ràng buộc nền của cả epic là AD-13/FR77 — gỡ sạch cấu hình AI thì mọi năng lực khác (Epic 1, 2, 3, và về sau là 5, 6) vẫn phải chạy đầy đủ, cưỡng chế bằng test tự động chứ không bằng kỷ luật lập trình viên.

Cần đọc trước khi viết dòng mã đầu tiên của bất kỳ story nào: `core/glossary/store.rs` (doc-comment của `entries_eligible_for_injection`) và mục nợ "Chủ: Epic 4" trong `deferred-work.md` — cả hai đã dự đoán đúng hình dạng lời gọi mà Story 4.6 sẽ cần.

## Stories

- Story 4.1: Module `ai/` cô lập và test cưỡng chế ranh giới
- Story 4.2: Cấu hình nhà cung cấp AI
- Story 4.3: API key trong keychain
- Story 4.4: Bộ prompt theo thể loại
- Story 4.5: Xuất và nhập bộ prompt
- Story 4.6: Smart RAG Injector là một hàm thuần
- Story 4.7: Xem prompt cuối cùng đã gửi
- Story 4.8: Dịch một segment với kết quả chảy dần
- Story 4.9: Dịch theo lô và huỷ giữa chừng
- Story 4.10: Lỗi mạng và lỗi API
- Story 4.11: Số token và ước tính chi phí
- Story 4.12: Bố cục màn hình hẹp và hiệu chỉnh ngưỡng

## Requirements & Constraints

Cấu hình nhà cung cấp AI (endpoint, tên mô hình, tham số sinh) dùng chung một biểu mẫu cho cloud BYOK lẫn local LLM qua endpoint tương thích OpenAI — không có đường tích hợp riêng cho từng nhà cung cấp. Cấu hình lưu ở tầng Global, ghi đè được theo Tác phẩm, phân giải qua `ScopeResolver`. API key không bao giờ nằm trong một file nào trên máy hay đi qua IPC; frontend chỉ biết "đã cấu hình / chưa cấu hình".

Bộ prompt theo thể loại soạn/sửa/xoá được, tồn tại ở cả hai tầng (Tác phẩm thắng khi trùng tên), và xuất/nhập được bằng file văn bản mở round-trip đầy đủ — trùng tên khi nhập phải hỏi người dùng, không âm thầm ghi đè.

Mọi lời gọi AI nhận đúng một prompt đã lắp sẵn từ `RagInjector`; thuật ngữ Glossary đã chốt (không phải mục chờ chốt) được chèn kèm bản dịch, ở cả hai tầng. Người dùng luôn mở xem được prompt thật đã gửi, khớp 100% với thứ đã gửi đi, phân biệt phần chèn động với phần người dùng soạn.

Dịch đơn lẻ và dịch theo lô đều chảy dần và huỷ được giữa chừng; kết quả luôn dừng ở panel AI Translation, chuyển sang Editor chỉ bằng thao tác tường minh (`⌘⇧↵`). Lỗi mạng/API không đổ lỗi người dùng, không tự động thử lại (BYOK — mỗi lần gọi là tiền thật), và không làm mất việc đang làm ở Editor. Số token luôn hiển thị; ước tính chi phí chỉ hiện với mô hình tính phí, ghi đúng số chứ không làm tròn thành chữ, và khi nhà cung cấp không trả số token thì nói rõ "không có số liệu" thay vì hiện 0.

Bố cục màn hình hẹp: cặp Nguyên văn | Bản dịch không bao giờ bị hy sinh; thứ tự nhường là Đề xuất AI trước, Tra cứu sau (rút về thanh trạng thái, không mất hẳn). Bốn ngưỡng đo theo vùng làm việc (không theo kích thước màn hình) và nay phải hiệu chỉnh riêng cho cả hai bố cục Ⓑ-1 và Ⓑ-2 — một bộ số đúng cho bố cục này không suy ra được cho bố cục kia.

⚠️ Món nợ có chủ chưa có FR: FR20 (đồng bộ cuộn ba panel) đã bị rút ở Epic 2 vì lưới nuốt hai trong ba panel, nhưng cặp còn lại — lưới ↔ AI Translation — vẫn cần đồng bộ khi dịch theo lô chạy trên nhiều segment cùng lúc, và hôm nay không FR nào chứa nhu cầu đó. Epic 4 phải tự quyết có cần hay không; nếu cần thì viết một FR mới, không phải khôi phục nguyên văn FR20.

## Technical Decisions

`ai/` là module một chiều: không module nào ngoài nó được phụ thuộc vào nó (AD-13), cưỡng chế bằng test tự động hoặc bằng crate riêng để trình biên dịch canh — đây là điều kiện để FR77 không thoái hoá thành kỷ luật cá nhân. Chiều ngược lại hợp lệ: `ai/` được đọc `glossary/`, `tm/`, `segment/`. Story 4.1 đã tách khỏi phần còn lại của Epic 4 và chạy ngay sau Epic 3 (thứ tự 3½) — ranh giới này cần dựng từ dòng code đầu tiên, không lùi cùng phần còn lại của epic; khi Story 4.2 trở đi tới lượt, AC ranh giới phải chạy lại trên bộ test của cả Epic 5 và Epic 6, không chỉ Epic 1–3 như văn bản gốc.

`RagInjector` là một hàm thuần `(câu nguồn, scope, Glossary, TM) → prompt đã lắp hoàn chỉnh` (AD-14); không nối chuỗi prompt rải rác tại chỗ gọi; cùng đầu vào luôn ra cùng một prompt. Tham số TM nhận rỗng ở epic này — chữ ký không đổi khi Epic 7 điền nốt nó. Đường duy nhất chạm dữ liệu Glossary là `core::glossary::entries_eligible_for_injection(resolver, global, work)` — mục ở trạng thái chờ chốt không được chèn. Trước khi RagInjector chạy thật, cần đo chi phí của hàm này (nó quét và nhân bản toàn bộ hai bảng Glossary ở mỗi lượt gọi, tức mỗi câu được dịch) và quyết cache theo phiên hay đổi chữ ký; kết quả trả về hôm nay cũng đánh rơi nhãn tầng nguồn và `id` chỉ duy nhất trong một `Store` — cần tự thiết kế hình dạng dữ liệu vào RagInjector để việc này không gây đụng độ.

Streaming token dùng Tauri Channel API, không dùng event rời và không dùng client SSE tự kết nối lại (AD-22); đứt luồng là lỗi tường minh, thử lại chỉ do người dùng chủ động, mọi lời gọi huỷ được giữa chừng. API key lưu qua crate `keyring` gọi trực tiếp trong Rust, cấm `tauri-plugin-keyring` (AD-29/NFR11); khoá không bao giờ đi qua IPC hay xuất hiện trong log lỗi. Lỗi qua IPC mang hình dạng `{ code, message_key, params, retryable }` (AD-21), chuỗi hiển thị phân giải ở `vi.json`. Cổng `TranslationProvider` đã khai (AD-2) nhưng chưa có cài đặt nào — Epic 4 là nơi cắm implementation thật đầu tiên. Khi kết quả AI được đưa sang Editor bằng `⌘⇧↵`, xuất xứ ghi là "người khác dịch" theo bảng AD-47③.

## UX & Interaction Patterns

Panel AI Translation khi chưa cấu hình chỉ mời cấu hình và nói rõ mọi thứ khác vẫn chạy đầy đủ — đây không phải trạng thái lỗi. Trạng thái AI có đúng năm giá trị: chưa cấu hình · đang sinh · xong · lỗi · đã huỷ. Xem prompt là một command đăng ký, gán phím được, hiện rõ thuật ngữ Glossary nào được chèn kèm dòng tóm tắt số lượng. Bộ prompt đang có hiệu lực của Tác phẩm đang mở hiện rõ và đổi được ngay tại chỗ, không phải vào Cài đặt. Mọi thao tác (dịch đơn, dịch lô, huỷ, xem prompt) là command đăng ký và dùng được bằng bàn phím.

## Cross-Story Dependencies

Story 4.1 đứng tách biệt và đi trước toàn bộ phần còn lại (ngay sau Epic 3); các story 4.2–4.12 tự chúng chạy sau Epic 6 theo thứ tự thực thi đã điều chỉnh (1→2→3→3½→5→6→4), dù số Epic trong tên story không đổi. Story 4.6 (RagInjector) cần Glossary đã chốt từ Epic 3 và là điều kiện của Story 4.7 (Xem prompt) lẫn 4.8/4.9 (các lệnh gọi dịch thật). Story 4.2 và 4.3 (cấu hình + keychain) là nền cho mọi lời gọi AI ở các story sau. Story 4.11 (token/chi phí) và 4.10 (lỗi) gắn liền với luồng streaming của 4.8/4.9, không phải tính năng độc lập. Story 4.12 khoá lại đúng bốn ngưỡng và thứ tự hy sinh panel đã chốt từ Story 1.14, chỉ hiệu chỉnh con số trên máy thật cho cả hai bố cục.

Epic 5 và Epic 6 không phụ thuộc Epic 4 (đã xác nhận bằng quét từ khoá `AI`/`ai/`/C6/C7 ra 0 lần ở cả hai). Ngược lại, Epic 7 (TM), Epic 8 (Reviewer) và Epic 9 (AI Proofreader) đều đứng sau Epic 4 và cần các quyết định của nó — đặc biệt là chữ ký `RagInjector` (tham số TM) và mẫu module hai tầng mà `glossary/` đã dựng. Việc mở lại một `.atproj` đã đóng chưa phục dựng được `ScopeResolver` tầng Tác phẩm (nợ thuộc Epic 5) — nghĩa là cho tới khi Epic 5 đóng nợ đó, RagInjector sẽ không thấy mục Glossary tầng Tác phẩm sau một lần mở lại Tác phẩm, chỉ thấy đúng trong phiên vừa tạo mới.
