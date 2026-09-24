# Phiếu quyết — nợ đứng tên Ice, hạng P

**Nguồn:** `sprint-change-proposal-2026-09-24b-no-dung-ten-ice.md` §6 · **Người quyết:** Ice · **Người trình:** John

108 quyết định riêng (119 mục, 11 mục gộp vào câu gốc). `b8f22f7:NNNN` là dòng của mục trên `b8f22f7`; tìm trên HEAD bằng `git show b8f22f7:_bmad-output/implementation-artifacts/deferred-work.md | sed -n NNNNp` rồi grep dòng đầu.

Mỗi câu trả lời một trong: **chọn (x)** · **KHÔNG LÀM** · **giữ Ice đến khi <điều kiện>** · **giao <story/người>**. John thi hành câu trả lời vào `deferred-work.md` sau mỗi phiên.

## Nhập & Library (31)

1. `b8f22f7:1812` — Luật 'câu phải có ít nhất một chữ cái mới chốt ranh giới' khi tách segment: (a) giữ nguyên, (b) nới/thu hẹp?
   - Quyết: 

2. `b8f22f7:1824` — Thêm bước xác nhận ngôn ngữ lúc tạo Tác phẩm (đối chiếu nội dung với nhãn, chỉ cảnh báo) — (a) mở story, (b) không?
   - Quyết: 

3. `b8f22f7:7430` — chapter.source_text có còn là bản lưu thô giữ nguyên byte khi tách Chương — (a) có, (b) không?
   - Quyết: 

4. `b8f22f7:7455` — Mở Tác phẩm ở Chương nào: (a) thêm work.last_chapter_id ngay, hay (b) chờ Epic 6 nhập hàng loạt rồi quyết?
   - Quyết: 

5. `b8f22f7:7490` — Tra Library bằng truy vấn 1-2 ký tự Hán: (a) thêm bảng char_idx thứ ba (tốn CPU rebuild), hay (b) chấp nhận hạn chế?
   - Quyết: 

6. `b8f22f7:7525` (+ `7540`) — Tìm Library khoan dung đ/Đ: (a) dựng hàm gấp dấu Rust riêng, hay (b) chấp nhận đ không khớp d?
   - Quyết: 

7. `b8f22f7:7795` (+ `7844`) — 35 Tác phẩm user_version≤7 ngoài chỉ mục: (a) dựng di trú hàng loạt có đồng ý người dùng, hay (b) giữ mở tay từng cái?
   - Quyết: 

8. `b8f22f7:8282` — Chương nhập trước Story 6.4 còn \r\n thô: (a) di trú một lần (xử lý Chương đã dịch thế nào?), hay (b) để nguyên?
   - Quyết: 

9. `b8f22f7:8337` — Mẫu phân tách neo dòng khớp 0 lần: (a) chấp nhận+cảnh báo, (b) đổi thứ tự bước (AD mới), (c) sửa mẫu mặc định mockup?
   - Quyết: 

10. `b8f22f7:8531` — Mở năng lực 'bật/tắt luật làm sạch cho đúng khối đang chọn' ở màn xem trước nhập — (a) mở, (b) không?
   - Quyết: 

11. `b8f22f7:8594` (+ `8833`) — Đường Blob+mẫu phân tách: (a) xây theo dõi vị trí xuyên bước chuẩn hoá để có báo cáo làm sạch từng Chương, hay (b) giữ một báo cáo?
   - Quyết: 

12. `b8f22f7:8657` — Bề mặt soạn luật làm sạch tầng Tác phẩm: (a) mở rộng Story 6.9, (b) story quản lý Tác phẩm mới, (c) chưa làm?
   - Quyết: 

13. `b8f22f7:8730` — Mở con trỏ Chương ⌥←/⌥→ cho đường tệp/dán tay: (a) xây cơ chế theo dõi vị trí (có thể AD mới), hay (b) giữ no-op?
   - Quyết: 

14. `b8f22f7:8950` — Đặt trần số link mỗi lượt dán (số đo trên lượt thật) và từ chối cả danh sách khi vượt — (a) có, (b) không?
   - Quyết: 

15. `b8f22f7:8969` — Khử trùng URL trong danh sách nhập theo: (a) nguyên văn, (b) chuẩn hoá, (c) nội dung sau tải, (d) không khử?
   - Quyết: 

16. `b8f22f7:8982` — Override khối tầng 2: (a) tách theo từng Chương theo con trỏ, hay (b) giữ chung một vector theo index khối?
   - Quyết: 

17. `b8f22f7:9236` — Hàng I/O Matrix 'Trang 0 khối' gần như không tới được: (a) bỏ hàng, hay (b) giữ nhánh UI làm hàng rào có ghi chú?
   - Quyết: 

18. `b8f22f7:9388` — Dưới bốn Chương 'N cần xem' không hiện: (a) tách 'N cần xem'/'M sạch' thành hai vế độc lập, hay (b) giữ nguyên?
   - Quyết: 

19. `b8f22f7:9646` — Áp bốn trường xuất xứ cho nhiều Chương một lượt: (a) chọn dải theo ord, (b) từng ô, (c) cả bốn ô cùng lúc, (d) không làm?
   - Quyết: 

20. `b8f22f7:9792` — Header/footer/footnote/comment của .docx: (a) đưa vào văn bản Chương, hay (b) bỏ như hiện tại?
   - Quyết: 

21. `b8f22f7:9818` — Dựng bộ đọc bảng Markdown có cấu trúc cho .md/.txt — (a) có, phạm vi tới đâu, (b) không?
   - Quyết: 

22. `b8f22f7:10168` — Đọc bảng Markdown song ngữ .md (FR115): (a) qua correct-course, (b) story theo sau 6.16, (c) không làm?
   - Quyết: 

23. `b8f22f7:10179` — Đọc bảng .docx song ngữ (FR115): (a) qua correct-course, (b) story theo sau 6.16 khi docx reader lộ hàng, (c) không làm?
   - Quyết: 

24. `b8f22f7:10632` — Nhận .docx trong nhập hàng loạt N tệp: (a) mở story đổi DocxSidecar thành Vec, hay (b) giữ từ chối từng mục?
   - Quyết: 

25. `b8f22f7:10653` — Con trỏ Chương LAZY cho đường Files: (a) chấp nhận vĩnh viễn không có, hay (b) dựng tách-lại-để-định-vị O(N tệp)?
   - Quyết: 

26. `b8f22f7:10712` — Nhập song ngữ hai cột có thêm năng lực 'thêm vào Tác phẩm sẵn có' (mục quy hoạch mới qua correct-course) — (a) có, (b) không?
   - Quyết: 

27. `b8f22f7:10727` — Append vào Tác phẩm khác ngôn ngữ: (a) cảnh báo bằng heuristic, hay (b) để người dùng tự nhận qua xem trước?
   - Quyết: 

28. `b8f22f7:10742` — Quét Glossary cho Chương vừa append: (a) không, (b) một Chương đại diện, (c) toàn bộ — ngay sau append hay khi mở?
   - Quyết: 

29. `b8f22f7:10795` — Bộ lọc 'cần xem' kẹt bật không nút tắt ở hai màn xem trước: chọn một cách sửa chung — (a) tự tắt lọc, (b) giữ chip hiển thị?
   - Quyết: 

30. `b8f22f7:10828` — A11y màn xem trước: (a) nối đủ aria-pressed và listbox cho con trỏ song ngữ, hay (b) bỏ vạch con trỏ khỏi màn song ngữ?
   - Quyết: 

31. `b8f22f7:10839` — Lỗi pipeline khác ở ứng viên song ngữ bị nuốt: (a) nới is_bilingual_table_refusal, hay (b) hiện dòng 'chưa đủ dữ liệu'?
   - Quyết: 

## CI & cổng (11)

32. `b8f22f7:1421` — Dựng bộ test tự động trong Chrome cho vế DOM dù nó không bắt được lỗi riêng WKWebView — (a) dựng, (b) giữ nghiệm thu tay?
   - Quyết: 

33. `b8f22f7:1718` — Khôi phục CI bằng: (a) bấm tay khi cần, (b) repo công khai, (c) runner tự quản trên máy Ice, (d) act cục bộ?
   - Quyết: 

34. `b8f22f7:2196` — Cửa rà giấy phép NFR15: (a) dựng cổng máy kiểm đã mở tệp giấy phép, hay (b) giữ quy trình người?
   - Quyết: 

35. `b8f22f7:3574` — Dựng cổng canh 'xuất xứ không có bề mặt UI mới' dù cổng phủ định ấy khó đỏ được — (a) dựng, (b) giữ bằng kỷ luật?
   - Quyết: 

36. `b8f22f7:4664` — Thêm lượt vitest TZ=UTC vào ba danh sách cổng (~4s mỗi lượt) — (a) thêm, (b) không?
   - Quyết: 

37. `b8f22f7:4677` — Canh CI bị bỏ qua: (a) bật thông báo GitHub khi master đỏ, (b) pre-push cảnh báo qua gh run list, (c) mục đọc CI trong retro/sprint-status?
   - Quyết: 

38. `b8f22f7:7080` — Đưa cargo clippy --all-targets -D warnings vào cổng — (a) có, dọn 22 cảnh báo trước, (b) không?
   - Quyết: 

39. `b8f22f7:8756` — check:debt-owner: (a) đổi NEGATIVE_OWNER_RE sang danh sách CHO PHÉP, chấp nhận đỏ hàng loạt để dọn sổ, hay (b) giữ danh sách CẤM?
   - Quyết: 

40. `b8f22f7:9731` — Toolchain Rust máy dev lệch CI: (a) thêm rust-toolchain.toml ghim 1.97.1, hay (b) ghi nhận lệch, chỉ tin số hiệu năng ở CI?
   - Quyết: 

41. `b8f22f7:10529` — Dựng cổng đo kích thước tệp: (a) có, ngưỡng riêng cho Rust và Vue SFC, hay (b) chưa?
   - Quyết: 

42. `b8f22f7:11699` — check:debt-owner đọc 'ĐÓNG MỘT NỬA' thành đóng: (a) đổi vị từ, kèm ca tự kiểm từng nhánh, hay (b) giữ?
   - Quyết: 

## Nhập — ảnh & mạng (10)

43. `b8f22f7:9491` — Hiển thị images_failed/images_saved cho người dùng qua: (a) toast, (b) dòng trong xác nhận nhập, (c) mục trong lưới Tác phẩm?
   - Quyết: 

44. `b8f22f7:9545` — Ảnh nhúng data: URI: (a) xử lý và lưu (không gọi mạng), hay (b) giữ vứt như ảnh trượt?
   - Quyết: 

45. `b8f22f7:9553` — Pha tải ảnh khi nhập nhiều Chương: thêm (a) ngân sách thời gian, (b) tiến độ theo mục, (c) nút huỷ — chọn những gì?
   - Quyết: 

46. `b8f22f7:9562` — Allowlist ảnh tầng 2: (a) nới khoá thành host+port+scheme, hay (b) giữ chỉ host?
   - Quyết: 

47. `b8f22f7:9608` — Cột 'chặng cuối sau chuyển hướng' cho asset.source_url: giao story nào — (a) một story 11.6, (b) story Epic sau, (c) không làm?
   - Quyết: 

48. `b8f22f7:9699` — Ảnh Chương 1 chuyển hướng sang host của ảnh Chương 5 được cho qua: (a) đúng ý, hay (b) lỗ AD-41 cần chặn?
   - Quyết: 

49. `b8f22f7:9777` — Ảnh .docx: (a) mở thêm EMF/WMF/BMP/TIFF (rủi ro AD-16), hay (b) giữ bốn kiểu raster?
   - Quyết: 

50. `b8f22f7:9876` — Ảnh .docx sau ranh giới phân tách: (a) mỗi Chương tự cắt blocks để ảnh vào đúng Chương, hay (b) chấp nhận gắn vào Chương đầu?
   - Quyết: 

51. `b8f22f7:9904` — Caption Word: nhận (a) pStyle 'Caption', (b) đoạn ngay sau ảnh; và có đọc descr/title làm alt cho .docx?
   - Quyết: 

52. `b8f22f7:10064` — Phạm vi asset:// theo AD-23: (a) đúng một Tác phẩm mỗi lúc, thu hồi khi đổi, hay (b) chấp nhận tích luỹ trong phiên?
   - Quyết: 

## Glossary (9)

53. `b8f22f7:5217` — Thêm nhanh thuật ngữ khi có Tác phẩm mở: (a) giữ mặc định Global, hay (b) mặc định tầng Tác phẩm?
   - Quyết: 

54. `b8f22f7:5822` — Đề xuất Hán Việt trên dải có hiện nhãn nguồn (sources_used) cho cả cụm — (a) có, (b) không?
   - Quyết: 

55. `b8f22f7:5834` (+ `5850`) — Tắt nguồn từ điển: (a) thêm chỗ gọi refreshGlossaryMarks thứ tư và sửa doc-comment đã ký, hay (b) để dấu Glossary cũ?
   - Quyết: 

56. `b8f22f7:5868` — Năm năng lực mockup glossary-queue (lọc phân loại, bỏ hàng loạt, số Chương, nhiều ví dụ, phân loại đoán): (a) giữ mockup và xây, (b) bỏ?
   - Quyết: 

57. `b8f22f7:5931` — Thêm cột đếm (migration) cho glossary_entry để hiện cột 'Dùng' và sắp theo tần suất — (a) xây, (b) không?
   - Quyết: 

58. `b8f22f7:5945` — Xoá mục Glossary: (a) hộp xác nhận mọi lượt xoá, (b) đường hoàn tác sau xoá, (c) giữ như hiện tại?
   - Quyết: 

59. `b8f22f7:5960` — Gõ tìm/đổi lọc khi đang sửa mục Glossary: (a) vô hiệu thanh công cụ, (b) giữ bản sửa qua lượt lọc, (c) báo trước khi bỏ?
   - Quyết: 

60. `b8f22f7:6819` — Câu lỗi 'tệp quá lớn': (a) bỏ {size}, chỉ nêu {limit} dùng chung, hay (b) tách khoá riêng cho Glossary?
   - Quyết: 

61. `b8f22f7:6895` — Lô nhập Glossary hỗn hợp: (a) đổi hàng ④ I/O Matrix sang báo va chạm, hay (b) giữ row_missing?
   - Quyết: 

## Editor & tiêu điểm (8)

62. `b8f22f7:136` (+ `2092`) — Khi đổi Chương, resetEditorPanel nên: (a) trao tiêu điểm cho entry-focus GridPanel theo AD-34, hay (b) giữ nguyên, tiêu điểm rơi về body?
   - Quyết: 

63. `b8f22f7:2205` — Sau remount EditorPanel khi đổi preset: (a) chấp nhận mất tiêu điểm tạm, hay (b) cho ngoại lệ giành tiêu điểm lúc mount?
   - Quyết: 

64. `b8f22f7:2514` (+ `3408`) — Bề mặt lỗi UX-DR30: (a) nối lỗi flush và message_key thật vào cột nhãn hàng, kèm e2e canh, hay (b) giữ phạm vi tối thiểu đã ký?
   - Quyết: 

65. `b8f22f7:3745` — Nhánh ornament: (a) dựng bề mặt hàng 'về hưu' mờ để hồi sinh, hay (b) rút UX-DR19 xuống năm giá trị?
   - Quyết: 

66. `b8f22f7:4423` (+ `3865`) — ⌘Z vào lượt gộp/tách: (a) thêm dòng báo StatusBar hướng dẫn, hay (b) ghi quyết định giữ im lặng có lý do?
   - Quyết: 

67. `b8f22f7:9743` — Ảnh neo rơi giữa nhóm gộp/tách mới snap về: (a) SAU nhóm (hiện tại), hay (b) TRƯỚC nhóm?
   - Quyết: 

68. `b8f22f7:10004` — Ô bản dịch trống đối diện ảnh có chấp nhận được, và (a) thêm e2e đo hình học thật, hay (b) dừng ở DOM?
   - Quyết: 

69. `b8f22f7:10082` — Chương toàn ảnh (0 câu văn xuôi): (a) hiện chú riêng thứ ba, hay (b) đổi nghĩa segment_count?
   - Quyết: 

## Test & bàn đo (6)

70. `b8f22f7:5983` — Nhánh changed==0 của promote_to_global: (a) dựng cơ chế chèn điểm dừng để test tất định, hay (b) chấp nhận không canh?
   - Quyết: 

71. `b8f22f7:9247` — Đối chứng đỏ ② của spec 6.9 đã done mới đóng một nửa: (a) chấp nhận như hiện tại, hay (b) đòi đo thêm trước khi sửa bản ghi?
   - Quyết: 

72. `b8f22f7:9293` — cargo test đỏ do LuLu: (a) cấp phép LuLu cho target/debug/deps, (b) tách ca server cục bộ ra #[ignore], (c) chỉ tin CI và ghi AGENTS.md?
   - Quyết: 

73. `b8f22f7:10148` — Ca chuyển hướng origin_url đầu-cuối: (a) sửa để chạy được trên máy Ice dưới LuLu, hay (b) chỉ canh ở CI?
   - Quyết: 

74. `b8f22f7:11617` — AC7 Story 4.8 'test không-AI vẫn xanh': (a) định nghĩa là mọi nhị phân test ngoài Epic 4, hay (b) tập khác?
   - Quyết: 

75. `b8f22f7:11667` (+ `11721`) — Dựng khuôn test tầng vỏ #[tauri::command] với AppHandle, state và Channel thật — (a) dựng, (b) không?
   - Quyết: 

## Tài liệu thiết kế (5)

76. `b8f22f7:1944` — Sửa nhãn Covers của Story 2.2 trong epics.md thành UX-DR19/20/2/12/7 + AD-34§2 — (a) sửa, (b) giữ?
   - Quyết: 

77. `b8f22f7:2132` — Chiều cao StatusBar: xác nhận 34px và sửa DESIGN.md cùng mockup còn ghi 32px — (a) 34px, (b) 32px?
   - Quyết: 

78. `b8f22f7:2436` — Bổ sung hàng 'đã dịch, chưa xác nhận' vào bảng trạng thái EXPERIENCE.md cho khớp Quyết định #3 đã ký — (a) thêm, (b) không?
   - Quyết: 

79. `b8f22f7:3415` — Sửa câu mockup data-integrity.html: phiên bản thứ sáu xuất hiện ở lượt xác nhận KẾ TIẾP, khớp AD-31 — (a) sửa, (b) giữ?
   - Quyết: 

80. `b8f22f7:11911` — Chiều cao titlebar: xác nhận 40px và sửa DESIGN.md cùng UX-DR15 trong epics.md — (a) 40px, (b) 38px?
   - Quyết: 

## Quy trình & kế hoạch (5)

81. `b8f22f7:5343` — 19 mốc dòng epics.md trôi trong tài liệu đã đóng: (a) viết lại bản ghi lịch sử, hay (b) để nguyên như dấu vết thời điểm?
   - Quyết: 

82. `b8f22f7:10406` (+ `10431`) — Cho phép sửa cả sáu chú thích sai 'sync_threadpool', kể cả hai chỗ bị Never của spec đóng băng — (a) có, (b) không?
   - Quyết: 

83. `b8f22f7:10553` — Chặng 2 AI-6: (a) viết lại hệ xem trước nhập, hay (b) chỉ tách ba khối tách được, tệp gốc còn ~2.600 dòng?
   - Quyết: 

84. `b8f22f7:10611` — Số sai trong retro Epic 6 và agent-token-economics.md: (a) sửa qua correct-course, hay (b) ghi chú đính chính có ngày?
   - Quyết: 

85. `b8f22f7:11260` — Sửa AGENTS.md dòng 37: ba hằng DDL thay 'pair' và thay 'No gate guards' bằng tên ca canh — (a) sửa, (b) không?
   - Quyết: 

## Tra cứu, từ điển & Hán Việt (4)

86. `b8f22f7:746` — Ai, khi nào làm mới dict-core.db từ kaikki; nếu kaikki ngừng thì (a) tự lưu bản dump, (b) đổi nguồn, (c) đóng băng bản hiện có?
   - Quyết: 

87. `b8f22f7:779` — Xác nhận mặc định: không đánh dấu ký tự Hán Việt nhiều âm, và hai chuỗi vi.json riêng theo layersLoaded — (a) đồng ý, (b) hướng khác?
   - Quyết: 

88. `b8f22f7:2775` — Hàng Hán Việt song song Ⓑ-2 cao ~388px: (a) giữ nguyên, (b) giới hạn số dòng <rt>, (c) chỉ mở song song ở Ⓑ-1?
   - Quyết: 

89. `b8f22f7:4053` — Dấu cắt Hán Việt song song khi cắt giữa từ: (a) .hv-unit nguyên khối, luôn vẽ được, hay (b) giữ chính xác, chấp nhận dấu vô hình?
   - Quyết: 

## Giao diện chung (4)

90. `b8f22f7:1691` — Nút mở màn Attribution: (a) dời ra titlebar cạnh nút phím tắt (sửa mockup), hay (b) giữ vị trí hiện tại?
   - Quyết: 

91. `b8f22f7:9122` — Gộp các lớp phủ Cài đặt rời rạc (Glossary, Shortcuts…) vào một khung Cài đặt: (a) có — khi nào, (b) giữ rời?
   - Quyết: 

92. `b8f22f7:9135` — Chốt thứ tự/tên 11 mục nav Cài đặt, và có dọn Ngưỡng quét Glossary vào mục glossary — (a) có, (b) chưa?
   - Quyết: 

93. `b8f22f7:11547` — Mở story sửa aria-labelledby cho cả ba lớp phủ PromptLibrary/PromptImport/GlossaryImport: (a) giao story nào, khi nào, (b) chưa?
   - Quyết: 

## Lịch sử & xuất xứ segment (4)

94. `b8f22f7:3394` — Khôi phục lịch sử segment với cờ ngắt đoạn: (a) thêm cột cờ vào segment_version, (b) khai chỉ khôi phục văn bản, (c) hạ cờ về mặc định?
   - Quyết: 

95. `b8f22f7:3490` — Để trả lời 'phiên bản nào đang dùng': (a) thêm con trỏ segment.current_version_id (lược đồ mới), hay (b) không mở năng lực này?
   - Quyết: 

96. `b8f22f7:3645` — 'Xuất xứ lúc nạp' nghĩa là: (a) lúc nạp phiên panel, hay (b) lúc bắt đầu vòng draft hiện tại?
   - Quyết: 

97. `b8f22f7:3677` — Gặp translation_origin lạ từ .atproj tương lai: (a) từ chối mở, (b) hạ về rỗng, (c) báo lỗi?
   - Quyết: 

## AI (4)

98. `b8f22f7:11041` — Sửa cấu hình AI Global khi có Tác phẩm mở: (a) thêm lệnh đóng Tác phẩm, hay (b) bộ chọn tầng tường minh trên Cài đặt?
   - Quyết: 

99. `b8f22f7:11276` — Xuất nhiều bộ prompt một lượt như mockup: (a) xây (chốt thư mục, trùng tên, lỗi giữa chừng), hay (b) giữ một bộ?
   - Quyết: 

100. `b8f22f7:11785` (+ `11803`) — Số cộng dồn cả phiên ở thanh trạng thái như mockup: (a) mở story (có persist qua đóng app?), hay (b) không xây?
   - Quyết: 

101. `b8f22f7:11833` — Bảng giá bundled: bao nhiêu mô hình đi kèm, và ai đối chứng giá với nguồn thứ hai?
   - Quyết: 

## Phím tắt (3)

102. `b8f22f7:7671` — ⌘,: (a) giữ cho shortcuts.open, hay (b) chuyển sang Tinh chỉnh Chế độ đọc và cấp hợp âm mới cho shortcuts.open?
   - Quyết: 

103. `b8f22f7:7715` — Nút 'Dịch tiếp Chương N': (a) chọn một hợp âm mặc định, hay (b) để trống, chỉ dựa ChordOverrides?
   - Quyết: 

104. `b8f22f7:8805` — Hợp âm gửi ở Story 6.10: (a) giữ ⌥⌘↵, (b) đổi phím editor.confirm_segment, (c) bắt ⌘↵ bằng handler DOM cục bộ?
   - Quyết: 

## Phụ thuộc mới (2)

105. `b8f22f7:3691` — Nhận phụ thuộc unicode-normalization (qua cửa NFR15) để chuẩn hoá NFC/NFD khi so mốc segment — (a) nhận, (b) không?
   - Quyết: 

106. `b8f22f7:11211` (+ `11646`) — Nhận phụ thuộc zeroize (qua cửa NFR15) để xoá khoá API khỏi bộ nhớ khi huỷ — (a) nhận, (b) không?
   - Quyết: 

## NFR & hiệu năng (2)

107. `b8f22f7:4764` — Ghi nhận cặp wal_threshold_bytes⟷NFR18 ở spine không tồn tại ở tải này, và giao ai sửa đường flush làm trượt NFR18?
   - Quyết: 

108. `b8f22f7:7729` — NFR5 vượt trần 300MB: (a) nới trần A8 theo số thật, (b) ảo hoá dãy đọc ReadingMode, (c) thả dãy khi KeepAlive deactivate?
   - Quyết: 
