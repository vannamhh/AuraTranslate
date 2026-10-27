---
id: 15
type: story
title: "Dịch lô gom các câu liền nhau thành lượt gọi, chữ chỉ được nhận khi khối của nó đóng đúng"
parent: epic-ai-mo-va-smart-rag-injector
covers: ["FR69", "FR70", "FR71", "FR72", "FR73", "FR74", "FR75", "FR76"]
after: []
assignee: ""
refined: true
hitl: false
risk: high
---

# Dịch lô gom các câu liền nhau thành lượt gọi, chữ chỉ được nhận khi khối của nó đóng đúng

## Description

Hôm nay dịch lô gửi mỗi câu một request. Prompt không có câu kề nào, và phần cố định khoảng 250 token bị trả đủ giá ở từng câu.

Story này thi hành AD-60: dịch vùng chọn và dịch cả Chương gom các câu liền nhau thành lượt gọi, tối đa 100 câu mỗi lượt.

Lượt đo 2026-10-27 chỉ ra hai chỗ hỏng mà AD-60 phải chặn:
- Gửi cả Chương trong một lượt thì bị cắt giữa câu mà không có lỗi nào.
- Mô hình báo `stop` mà vẫn rơi câu.

Vì vậy một câu chỉ có bản dịch khi khối của nó đóng đúng. Mọi câu khác thành *thiếu kết quả AI* và chờ người dùng bấm thử lại.

## Requirements

1. **Gom lượt** (FR73)
   **Given** một thao tác dịch vùng chọn hoặc dịch cả Chương
   **When** thao tác bắt đầu
   **Then** các câu cần dịch được gửi thành các lượt liên tiếp, mỗi lượt chỉ gồm câu liền nhau trong một Chương, tối đa 100 câu cần dịch, chạy lần lượt
   **And** panel biết trước toàn bộ kế hoạch lượt từ Rust, không tự tính ranh giới lượt
   **And** vùng chọn gồm các đoạn rời thì mỗi đoạn đi lượt riêng
   **And** câu đánh dấu bỏ qua (`is_omitted`) không có mặt trong bất kỳ prompt nào
2. **Câu ngăn làm ngữ cảnh** (FR70, FR73)
   **Given** dịch cả Chương, và có câu đã dịch nằm giữa các câu cần dịch
   **When** lượt được lắp
   **Then** câu đã dịch đi kèm thành ngữ cảnh chỉ đọc, không được dịch lại, và không hàng nào của panel đổi vì nó
   **And** một đoạn liền dài hơn 3 câu đã dịch thì cắt sang lượt mới, lượt mới mang theo 3 câu đã dịch đứng ngay trước
3. **Prompt giữ nghĩa bộ prompt** (FR69, FR70)
   **Given** một bộ prompt có hoặc không có từng marker Glossary, TM và câu nguồn
   **When** lượt được lắp
   **Then** marker nào vắng thì phần của nó không được chèn, và màn Xem prompt ghi phần đó là "không hỏi"
   **And** Glossary chỉ gồm mục đã chốt có mặt trong các câu của lượt; mỗi câu mang tối đa 3 cặp TM, đặt ngay trong khối của câu đó
   **And** phần dặn định dạng khối luôn đứng sau system prompt của người dùng, kể cả khi system prompt rỗng
4. **Chữ chỉ được nhận khi khối đóng đúng** (FR72, FR74)
   **Given** một lượt đang chạy
   **When** mô hình trả chữ
   **Then** chữ hiện dần trên hàng của câu đó
   **And** chỉ khi lượt đã kết thúc thì từng câu mới thành *đã nhận*, hoặc *thiếu kết quả AI* nếu khối trùng chỉ số, rỗng, hay chưa đóng
   **And** hàng thành *thiếu kết quả AI* thì phần chữ đã hiện bị xoá, và không đưa được sang Editor
5. **Cắt ngang im lặng thành câu thiếu** (FR75)
   **Given** luồng kết thúc mà không có `finish_reason`, hoặc với `finish_reason` khác `stop`
   **When** lượt kết thúc
   **Then** các khối đã đóng đúng vẫn được nhận, mọi câu còn lại thành *thiếu kết quả AI*
   **And** không có lượt nào tự gửi lại
6. **Lỗi và huỷ giữ phần đã có** (FR73, FR75)
   **Given** nhà cung cấp lỗi ở lượt k, hoặc người dùng huỷ giữa chừng
   **When** thao tác dừng
   **Then** bản dịch đã nhận của các lượt trước, và các khối đã đóng đúng của lượt k, được giữ; các câu còn lại không mất chỗ trong panel
   **And** khi nhà cung cấp lỗi, câu lỗi theo AD-59 nêu lượt đã dừng bằng câu đầu và câu cuối của lượt đó
7. **Thử lại chỉ gửi câu thiếu** (FR75)
   **Given** một thao tác đã kết thúc còn câu *thiếu kết quả AI* hoặc câu chưa từng được gửi
   **When** người dùng bấm thử lại
   **Then** chỉ những câu đó được gửi, gom lại theo luật lượt
   **And** câu đã có bản dịch nằm trong tập đó vẫn được dịch lại, không thành ngữ cảnh
8. **Đưa sang Editor kiểm nguồn còn khớp** (FR72)
   **Given** một câu *đã nhận*
   **When** người dùng đưa nó sang Editor sau khi nguồn của câu đã bị sửa, hoặc câu đã bị gộp hay tách
   **Then** lệnh từ chối và không ghi gì
   **And** khi nguồn còn khớp, lượt ghi đi đúng đường đưa sang hiện có, kể cả phép hỏi ghi đè bản nháp chưa ký
9. **Token và chi phí theo lượt** (FR76)
   **Given** một thao tác nhiều lượt
   **When** panel hiện tổng token và ước tính chi phí
   **Then** tổng bằng tổng usage của các lượt, mỗi lượt cộng đúng một lần
10. **Xem và gửi lại prompt của lượt** (FR71)
    **Given** một câu đã được dịch trong một lượt
    **When** người dùng mở màn Xem prompt của câu đó
    **Then** màn hiện prompt của lượt mới nhất có câu đó là câu cần dịch, đánh dấu khối của nó; câu chỉ là ngữ cảnh thì màn ghi rõ điều đó
    **And** gửi lại prompt của lượt đi qua đúng các phép kiểm ở tiêu chí 4–6
11. **Hai cờ cảnh báo không chặn** (FR72, FR75)
    **Given** bản dịch của một câu còn ký tự Hán trong khi nguồn của nó có chữ Hán, hoặc một lượt dịch MỘT câu kết thúc không có `finish_reason`
    **When** kết quả hiện ra
    **Then** câu mang cờ cảnh báo tương ứng, và vẫn đưa sang Editor được
12. **Test canh đỏ được** (FR73, FR75)
    **Given** bộ test
    **When** nó chạy
    **Then** có đủ các ca (a)–(u) của AD-60 mục 14, mọi ca đi qua cổng nghe cục bộ với khung viết tay
    **And** mỗi ca canh một luật có một phép gỡ thật ghi trong plan, và phép gỡ đó làm ca đỏ đúng vì lý do được đo
13. **Ghi lại những gì chưa đo** (FR75)
    **Given** sổ nợ
    **When** story xong
    **Then** có một mục nợ `Chủ: Epic 4` cho ba việc: đo lại với `gemini-3.8-flash`; đo Ollama và LM Studio xem có gửi `finish_reason` không; và một lượt Ice dùng thật dịch cả một Chương từ 300 câu trở lên

## Boundaries

- Must not change: dịch một câu tại con trỏ vẫn đi đường cũ, chỉ thêm cờ của tiêu chí 11.
- Must not change: Proofreader, cả cách gom câu lẫn cách bỏ qua lô rỗng.
- Must not change: system prompt mặc định; hình dạng lỗi qua IPC `{ code, message_key, params, retryable }` (AD-21); không thêm điểm ra mạng nào (AD-15).
- Must not change: một lượt dịch lô không ghi gì vào `target_text`; chỉ thao tác đưa sang của người dùng mới ghi (FR72, AD-47).
- Must not change: mục Glossary chờ chốt vẫn không vào prompt (AD-36); chính sách TM của Tác phẩm đọc đúng một lần mỗi thao tác (AD-57).

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-ai-mo-va-smart-rag-injector/epic-ai-mo-va-smart-rag-injector.md, Requirements FR69–FR76
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-60 (cùng các dòng 🔵 2026-10-27 dưới AD-14, AD-47, AD-59)
- evidence — _bmad-output/initiative-auratranslate/ad-brief-dich-ca-chuong-mot-luot/do-m2.md, số đo cắt ngang, rơi câu và kích thước lượt
- research — _bmad-output/initiative-auratranslate/research-ai-translation-pipeline/research-ai-translation-pipeline.md, §2 và §5 (đường dịch lô và prompt ở `7bd1f90`)
- research — _bmad-output/initiative-auratranslate/research-llm-translation-cost-and-quality/research-llm-translation-cost-and-quality.md, R2
- plan — _bmad-output/initiative-auratranslate/tickets/epic-ai-mo-va-smart-rag-injector/tickets/story-dich-theo-lo-va-huy-giua-chung-plan.md, quy tắc "lỗi đầu tiên dừng lô" mà story này thay bằng AD-60 mục 7–8

## Notes

- Decision: 2026-10-27, Ice — gom câu (phương án P), trần 100 câu mỗi lượt, tự chia lượt, đoạn chọn rời đi lượt riêng, câu đã dịch làm ngữ cảnh, câu `is_omitted` không bao giờ gửi đi, dịch một câu chỉ gắn cờ `maybe_truncated`.
- Decision: 2026-10-27, Ice — ký AD-60.
- Kiểm thêm ngoài tiêu chí, vì risk high: Ice dùng thật một lượt dịch cả Chương từ 300 câu trở lên bằng khoá thật, đếm câu *thiếu kết quả AI* và đối chiếu tổng tiền với bảng điều khiển của nhà cung cấp. Đây là mục nợ ở tiêu chí 13, không chặn story xong.
- ⚠️ Khớp theo chỉ số không bắt được trường hợp mô hình bỏ một câu rồi đánh lại số cho câu sau (AD-60 mục 7); người duyệt là lớp canh.
- Risk high vì story đổi đường chữ AI đi vào Editor và đổi cách tính tiền của người dùng; hỏng ở đây im lặng và nằm trên dữ liệu thật.
