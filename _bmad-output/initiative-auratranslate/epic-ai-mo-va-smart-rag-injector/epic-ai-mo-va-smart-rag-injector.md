---
type: epic
title: "AI mở & Smart RAG Injector"
parent: initiative-auratranslate
covers: [FR65, FR66, FR67, FR68, FR69, FR70, FR71, FR72, FR73, FR74, FR75, FR76, FR77, FR79]
after: []
assignee: ""
status: in-progress
risk: low
---

# AI mở & Smart RAG Injector

## Description

Người dịch cấu hình **một API key của chính mình** hoặc trỏ tới Ollama/LM Studio qua **cùng một đường cấu hình**, rồi gọi AI dịch từng segment hoặc theo lô — kết quả chảy dần theo dòng, huỷ được giữa chừng, kèm số token và ước tính chi phí. Trước mỗi lần gọi, hệ thống tự chèn các thuật ngữ Glossary **đã chốt** xuất hiện trong câu; người dùng **mở xem được prompt cuối cùng đã gửi đi**. Kết quả nằm ở panel AI Translation và **không bao giờ tự chảy vào Editor**. Gỡ sạch cấu hình AI thì Epic 1, 2, 3 vẫn chạy đầy đủ — **cưỡng chế bằng test tự động**, không bằng kỷ luật.

## Outcome

Người dịch dùng AI của chính mình (API key riêng hoặc Ollama/LM Studio) để dịch từng segment hoặc theo lô, xem được prompt cuối cùng đã gửi, và vẫn dùng đầy đủ ứng dụng khi không cấu hình AI nào.

## Requirements

- FR65: **BYOK:** người dùng nhập API key của nhà cung cấp mình chọn.
- FR66: **Local LLM:** kết nối tới endpoint tương thích OpenAI (Ollama, LM Studio) qua **cùng một đường cấu hình** với BYOK.
- FR67: **API key lưu trong keychain / credential manager của hệ điều hành.** Không lưu văn bản thuần trong file dự án hay file cấu hình, không đồng bộ đi đâu.
- FR68: Cấu hình AI (nhà cung cấp, mô hình, tham số sinh) đặt ở **tầng toàn cục**, **ghi đè được theo từng Tác phẩm**.
- FR69: **Custom prompt theo thể loại** và theo quy chuẩn dịch. Tồn tại ở cả hai tầng; **tầng Tác phẩm thắng**.
- FR70: Trước mỗi lần gọi AI, hệ thống quét câu nguồn và **chèn động vào prompt**: (a) các thuật ngữ Glossary xuất hiện trong câu kèm **bản dịch đã chốt**; (b) các segment tương tự tìm được trong TM. **Chỉ mục đã chốt được chèn** (FR114 không tham gia).
- FR71: Người dùng **xem được prompt cuối cùng đã gửi đi**, bao gồm toàn bộ phần chèn động.
- FR72: Kết quả AI hiện ở **panel AI Translation** và **không tự động ghi vào Editor**. Người dùng chủ động đưa sang.
- FR73: Dịch theo **từng segment** và theo **lô nhiều segment liên tiếp**, **huỷ được giữa chừng**.
- FR74: Kết quả hiện **dần theo dòng chảy (streaming)** khi mô hình đang sinh.
- FR75: Khi gặp lỗi mạng hoặc lỗi API: thông báo rõ nguyên nhân, **không mất công việc đang làm**, cho phép thử lại **do người dùng chủ động**. Hệ thống **không được tự động thử lại**.
- FR76: Hiển thị **số token đã dùng và ước tính chi phí** cho mỗi lần gọi.
- FR77: **Ứng dụng phải hoạt động đầy đủ khi không cấu hình AI.** Mọi năng lực ngoài C6 và C7 phải chạy được mà không cần một API key nào.
- FR79: **Xuất và nhập bộ prompt** dưới dạng file văn bản mở, để người dịch chia sẻ prompt theo thể loại.

## Done when

1. Cấu hình một nhà cung cấp AI (API key trong keychain hoặc Ollama/LM Studio) rồi dịch một segment và một lô, kết quả chảy dần, huỷ được giữa chừng, và không bao giờ tự chảy vào Editor.
2. Prompt cuối cùng đã gửi mở xem được, gồm toàn bộ phần thuật ngữ Glossary đã chốt được chèn động bởi `RagInjector` (hàm thuần).
3. Gỡ sạch cấu hình AI thì Library, Workspace, tra cứu và Glossary vẫn chạy đầy đủ, và test tự động cưỡng chế ranh giới `ai/` đỏ khi một module ngoài `ai/` import nó.
4. Lỗi mạng và lỗi API hiện ra bằng lý do cụ thể; số token và ước tính chi phí hiện sau mỗi lần gọi.
5. Xuất và nhập được bộ prompt theo thể loại.

## Boundaries

Phần AI chủ động của người dịch (BYOK). Không phải Proofreader (epic AI Proofreader) và không phải nửa Translation Memory của FR70 (epic Translation Memory).

## References

- parent — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, functional requirements FR65, FR66, FR67, FR68, FR69, FR70, FR71, FR72, FR73, FR74, FR75, FR76, FR77, FR79
- spec — _bmad-output/initiative-auratranslate/spec-auratranslate/spec-auratranslate.md
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, các AD nêu trong tiêu chí của story
- change — _bmad-output/initiative-auratranslate/change-thu-tu-thuc-thi-epic/change-thu-tu-thuc-thi-epic.md, thứ tự thực thi epic

## Notes

- Decision: Story 4.1 chạy ngay sau epic Glossary, trước các epic Library và Đường nhập; phần còn lại của epic chạy sau epic Đường nhập (thứ tự thực thi do Ice chốt 2026-10-06, vị trí epic ở `tickets.toml` của initiative).
- Parked: FR70 có nghiệm thu chia đôi; nửa Translation Memory đóng ở epic Translation Memory.
- Open question: khi Stories 4.2–4.12 tới lượt, AC của Story 4.1 phải chạy lại trên bộ test của các epic Library và Đường nhập (xem ghi chú 🔵 trong tiêu chí Story 4.1).
- Retrospective: epic-ai-mo-va-smart-rag-injector-retrospective.md
