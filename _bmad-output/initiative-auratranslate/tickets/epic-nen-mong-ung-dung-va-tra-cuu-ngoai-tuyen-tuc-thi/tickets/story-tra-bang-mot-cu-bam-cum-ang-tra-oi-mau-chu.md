---
id: 23
type: story
title: "Tra bằng một cú bấm, cụm đang tra đổi màu chữ"
parent: epic-nen-mong-ung-dung-va-tra-cuu-ngoai-tuyen-tuc-thi
covers: [FR21]
after: []
refined: true
hitl: false
risk: medium
---

# Tra bằng một cú bấm, cụm đang tra đổi màu chữ

## Description

Bấm một chữ ở cột nguyên văn là thấy ngay nghĩa của từ chứa nó, như ở QuickTranslator, để tôi không phải bấm đúp hàng trăm lần mỗi Chương.

Story này làm đúng phần 🔵 bổ sung ngày 2026-11-12 của FR21. Đó là thao tác lặp nhiều nhất trong sản phẩm, nên cách chọn cụm và cách tô màu phải đúng với mọi cú bấm, kể cả các cú bấm không nên tra. Kéo chuột bôi đen vẫn là đường tra một cụm tuỳ ý. Story chỉ thêm đường bấm một cái, không thay đường kéo chuột.

## Requirements

Verify: Với một Chương tiếng Trung đang mở, khi người dịch bấm một cái vào một chữ ở cột nguyên văn hoặc âm Hán Việt, thì Panel Lookup hiện kết quả cho thuật ngữ Glossary hoặc từ chứa chữ đó, và cụm đó đổi màu chữ chứ không bị bôi đen.

1. **Bấm một cái tra từ chứa chữ đó** (FR21)
   **Given** một Chương tiếng Trung hoặc tiếng Anh đang mở, và chữ được bấm không nằm trong thuật ngữ Glossary nào
   **When** người dịch bấm một cái vào một chữ ở cột nguyên văn, và lúc nhả chuột không có vùng bôi đen nào (có vùng bôi đen thì là kéo, đi đường kéo như trước)
   **Then** Panel Lookup hiện kết quả cho đúng từ chứa chữ đó theo bộ tách từ của engine, tức đúng cụm mà double-click chọn trên cùng văn bản
   **And** trình duyệt không bôi đen gì

2. **Thuật ngữ Glossary thắng từ của bộ tách** (FR21)
   **Given** chữ được bấm nằm trong một thuật ngữ Glossary đang được đánh dấu ở cột nguyên văn
   **When** người dịch bấm một cái vào chữ đó
   **Then** cụm được tra là trọn thuật ngữ đó, kể cả khi bộ tách từ cắt nó ra nhiều từ (ví dụ một tên riêng)

3. **Âm Hán Việt tra chữ Hán** (FR21, FR19)
   **Given** cột nguyên văn đang hiện âm Hán Việt, ở kiểu chuyển đổi hoặc kiểu song song
   **When** người dịch bấm một cái vào một âm
   **Then** cụm được tra là chữ Hán nguồn, không phải chữ Latin của âm, chọn theo đúng quy tắc của tiêu chí 1 và 2 cho chữ Hán đứng dưới âm đó
   **And** khi không có thuật ngữ Glossary nào chứa chữ đó, cụm được tra trùng với khối từ mà cột Hán Việt đang hiển thị

4. **Cụm đang tra đổi màu chữ, không bôi đen** (FR21)
   **Given** một lượt tra bằng một cú bấm vừa phát
   **When** kết quả hiện ở Panel Lookup
   **Then** đúng các chữ của cụm đó, không thừa không thiếu, hiện bằng màu chữ ấm đã chốt ở cả hai theme
   **And** tương phản của màu đó đạt ít nhất 4,5:1 trên mọi nền mà ô nguyên văn có thể nằm, và cổng token của dự án tự kiểm điều này
   **And** trên engine không có Custom Highlight API, cụm đó được bôi đen như trước, không mất lượt tra

5. **Màu tô không đứng sai chỗ** (FR21)
   **Given** một cụm đang được tô màu
   **When** có lượt tra mới, người dịch đổi Chương, hoặc ô chứa cụm đó render lại (sửa chữ, gộp, tách)
   **Then** màu tô chuyển sang cụm mới hoặc biến mất
   **And** không lúc nào màu nằm trên chữ khác với cụm đang hiện ở Panel Lookup

6. **Cú bấm không có gì để tra thì không làm gì** (FR21)
   **Given** Panel Lookup đang hiện kết quả của lượt trước
   **When** người dịch bấm vào dấu câu, khoảng trắng, hoặc chỗ trống trong ô ngoài chữ
   **Then** không có lượt tra nào phát, Panel Lookup giữ nguyên kết quả cũ và màu tô cũ

7. **Không tra lặp** (FR21)
   **Given** một cụm vừa được tra bằng một cú bấm
   **When** người dịch bấm lại một chữ thuộc cùng cụm đó, hoặc bấm đúp vào nó
   **Then** không có lượt tra thứ hai, và Panel Lookup không nháy
   **And** điều này vẫn đúng khi cú bấm đúp làm engine chọn một cụm khác, ví dụ một phần của thuật ngữ Glossary

8. **Không thêm việc nặng vào cú bấm** (FR21, NFR1)
   **Given** một Chương 300 câu ở kiểu Song song
   **When** người dịch bấm một cái để tra
   **Then** lượt tra gọi đúng một lần cùng đường Rust mà NFR1 đo, không gọi thêm lượt nào để chọn cụm
   **And** vệt Performance của cú bấm không có lượt bố cục lại lưới nào do việc tô màu gây ra

9. **Cú bấm chuột thật tra được trong webview thật** (FR21)
   **Given** app chạy trên WKWebView
   **When** một cú bấm chuột thật, không phải `.click()` tổng hợp, rơi vào một chữ ở cột nguyên văn
   **Then** lượt tra phát và màu tô hiện

## Boundaries

- Must not change: kéo chuột bôi đen ở cột nguyên văn vẫn tra cụm vừa kéo khi nhả chuột, và vẫn copy, Thêm nhanh Glossary (FR48), Concordance (FR60) như trước.
- Must not change: chọn bằng bàn phím (`Shift` + mũi tên, rồi nhả `Shift`) và phím tra tường minh vẫn tra như trước. Phím tra tường minh vẫn tra lại cả khi cụm đang hiện.
- Must not change: `Mod`+click chỉ đánh dấu chỗ cắt, không tra. Chuột phải chỉ mở menu câu, không tra.
- Must not change: bấm vào ô bản dịch vẫn chỉ đặt con trỏ gõ, không tra. Bấm vào cột số câu vẫn chỉ chọn câu.
- Must not change: Chế độ đọc giữ tra bằng double-click mở popup (Ice ký 2026-11-12).
- Must not change: hai lượt sửa hiệu năng lưới của bug 2.18 và 2.19 vẫn giữ số đo của chúng.

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-nen-mong-ung-dung-va-tra-cuu-ngoai-tuyen-tuc-thi/epic-nen-mong-ung-dung-va-tra-cuu-ngoai-tuyen-tuc-thi.md, Requirements FR21
- prd — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, FR21 (🔵 bổ sung 2026-11-12)
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, Interaction Primitives §Auto-Lookup (🔵 bổ sung 2026-11-12)
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-1 (chọn cụm là trạng thái UI), AD-34 (lượt tra đi qua command đã đăng ký)

## Notes

- Decision: 2026-11-12, Ice chọn phương án A của Winston: thuật ngữ Glossary thắng, không thì lấy từ của bộ tách từ ở webview. Không có AD mới. Phương án chọn cụm theo đầu mục dài nhất của từ điển ở Rust đã được đo và bị bác (commit `2358e434`).
- Decision: 2026-11-12, Ice chọn màu ấm cho token `lookup-active`: theme sáng `#a8430c`, theme tối `#fb923c`. Đo bằng `scripts/lib/contrast.mjs`, tương phản thấp nhất trên bốn nền `surface`, `surface-sunken`, `surface-accent`, `surface-tm` là 5,12:1 (sáng) và 5,55:1 (tối). Hue nằm giữa `error` và `tm-text` nên không lẫn với hai họ màu đó.
- Decision: 2026-11-12, Chế độ đọc nằm ngoài story này.
- Decision: 2026-11-12, Ice xác nhận: bấm vào dấu câu hoặc khoảng trắng thì không tra và giữ kết quả cũ (tiêu chí 6); `Mod`+click chỉ đánh dấu chỗ cắt, không tra (Boundaries).
- Mã liên quan, cho người dựng: bề mặt nguồn đăng ký ở `src/panels/GridPanel.vue` (`useSelectionSurface(gridRoot, 'source', …)`). Toạ độ được đổi ra chỉ số ký tự bằng `caretPointAt` và `sourceCutOffsetOf` (`src/panels/editorSegments.ts`), là đường mà `Mod`+click đang dùng. Bộ tách từ ở `src/panels/wordBoundary.ts`. Bộ theo dõi nhả chuột và quy tắc chống tra lặp ở `src/panels/selectionContract.ts` (`attachSelectionWatcher`). Lượt tra hội tụ ở `dispatch('lookup.lookup_selection')`.
- ⚠️ Bàn đo của story 2.8 ghi rằng trên WKWebView, một cú bấm ở cột nguyên văn cho `rangeCount 0`, nên đường bấm một cái không đọc được vùng chọn. Chromium và Vitest không lộ chuyện này. Chỉ bàn đo chuột thật bắt được (tiêu chí 9). Phép kiểm đó phải đỏ khi gỡ thật đường bấm một cái khỏi mã sản phẩm.
- Chọn cụm theo thuật ngữ Glossary ở webview không trái AD-1. Biên thuật ngữ do Rust tính và gửi sang. Webview chỉ chọn khoảng nào chứa chữ được bấm, và đó là trạng thái vùng chọn mà AD-1 cho phép giữ ở frontend.
- Kiểm tay trong app thật (màu tô ở cả hai theme, cảm giác bấm ở cả ba kiểu xem) là việc chỉ người làm được: khi story đóng, ghi một mục nợ `Chủ: Epic 1`.
