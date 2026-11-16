# Cơ sở nội dung website

Đối chiếu ngày 2026-11-16 cho lượt viết lại nội dung sau phản hồi của Ice. PRD cung cấp ý định; các đường mã dưới đây xác nhận khả năng đã được nối vào app. Không suy ra trạng thái nghiệm thu từ việc có mã.

| Nhóm giới thiệu | Nguồn ý định | Mã đã đối chiếu |
| --- | --- | --- |
| Library, tìm cả nguyên văn và bản dịch, quản lý chương | PRD C1 | `commands/library.rs::search_library_scoped`, các command chapter trong `src-tauri/src/lib.rs` |
| Từ điển đa nguồn, Hán Việt và nguồn định nghĩa | PRD C2–C3 | `src/panels/GridPanel.vue`, `LookupPanel.vue`, `src/commands/index.ts::source.select_tab_han_viet`, `dict-manifest.toml` |
| Biên tập, tự lưu, lịch sử xác nhận | PRD FR23–26, FR78, FR100–101 | `src/panels/editorFlush.ts`, `src/panels/editorPanelState.ts`, `commands/segment/history.rs` |
| Duyệt thuật ngữ, ngữ cảnh, Hán Việt và hai tầng Glossary | PRD C4 | `core/glossary/scan.rs`, `src/glossaryQueueState.ts`, `src/GlossaryManageOverlay.vue` |
| Khớp tuyệt đối/gần giống, Concordance, xuất xứ và TMX | PRD C5 | `src/TmFuzzyStrip.vue`, `src/TmManageOverlay.vue`, `src-tauri/src/commands/tm.rs` |
| Dịch câu, vùng chọn, các câu chưa dịch trong chương | PRD FR72–76 | `src/aiTranslateHandlers.ts`, `src-tauri/src/commands/aitranslate.rs` |
| Kết hợp Glossary/TM và ngữ cảnh câu kề | PRD FR70–71 | `commands/aiprompt.rs`, `core/ai/rag.rs`, `src/aiPromptInspectorState.ts` |
| Prompt dịch và chỉ dẫn Proofreader theo tác phẩm | PRD FR69, FR160 | `src/PromptLibraryOverlay.vue`, lựa chọn `proofreadEffectiveName` |
| Các loại lỗi và thao tác nhận/bỏ/không phải lỗi | PRD C7 | `core/segment/proofread_kind.rs::ProofreadKind`, `src/proofreadHandlers.ts` |
| Quy ước chữ Việt ngoại tuyến, thay có xem trước | PRD FR159 | `core/convention/`, `src/conventionHandlers.ts`, `src/ConventionReplaceOverlay.vue` |
| Dịch nhanh tại bảng Glossary và xem trước URL | PRD FR153–158 | `src/glossaryQueueState.ts`, `commands/aitranslate.rs`, `src/config/quickBatch.ts` |
| Reviewer, nối câu, nhận diff, thu hoạch thuật ngữ | PRD C8 | `src/reviewerImportState.ts`, `src/AlignmentOverlay.vue`, `commands/export.rs::reviewer_harvest_refresh` |
| Nhập/xuất, hình ảnh và ghi nguồn | PRD C1, C8 | `src/ImportPreviewOverlay.vue`, `src/ExportOverlay.vue`, `core/export/options.rs`, `core/export/reimport_gate.rs` |
| Đọc song ngữ, đánh dấu và trở về câu | PRD FR120, C1 | `src/modes/ReadingMode.vue`, `src/modes/ReadingMarksList.vue` |

Các giới hạn đã giữ trong câu chữ:

- Glossary và TM được chèn theo phần ngữ cảnh mà bộ prompt hỗ trợ; không hứa AI tự học hay luôn tuân thủ tuyệt đối. Xem `gather_glossary_context` và `assemble_prompt`.
- Số token phụ thuộc dữ liệu nhà cung cấp trả về; chi phí ước tính còn cần giá mô hình. Xem `src/aiUsageLine.ts`.
- Lịch sử được mô tả theo lần xác nhận; không hứa lưu mọi phím gõ.
- Bản dùng để nhập lại cần tránh các tùy chọn đánh dấu làm nó thành bản có chú giải. DOCX một khối dùng để đăng, không để nhập lại.
- Dịch nhanh là nội dung tham khảo tại hai bề mặt duyệt, tách khỏi bản dịch tác phẩm.

Ảnh và nội dung minh họa: xem `website/assets/screenshots/README.md`. Các nút tải mở trang release theo yêu cầu của Ice; thông tin trực tiếp về bộ cài được xác minh trong `website-verification.md`.
