---
id: 24
type: story
title: "Màn xem trước nhập: văn bản hiện một lần, chỗ luật xoá gạch ngang tại chỗ"
parent: epic-duong-nhap
covers: [FR124]
after: [23]
refined: true
hitl: false
risk: medium
---

# Màn xem trước nhập: văn bản hiện một lần, chỗ luật xoá gạch ngang tại chỗ

## Description

Sau story 23, tầng luật làm sạch ở cột cấu hình vẫn in lại tới ~4 KiB văn bản của Chương để gạch ngang chỗ sắp xoá. Cùng đoạn đó đã có trong cột văn bản, nên người dùng đọc nó hai lần, và chỗ gạch ngang nằm xa chỗ mắt đang đọc.

Story này đưa chỗ gạch ngang về cột văn bản: trong khối chứa nó (URL), hoặc trong văn bản của Chương (tệp, dán tay). Tầng luật chỉ còn danh sách luật và số đếm. Chỗ khớp trong khối do Rust ánh xạ, vì chỉ Rust biết khối ghép thành văn bản ra sao (AD-1).

## Requirements

Verify: Một spec e2e chạy một lượt nhập URL và một lượt nhập `.txt`, mỗi lượt có một luật khớp 3 chỗ; cộng các ca Rust cho phép ánh xạ chỗ khớp vào khối và tệp vitest của màn xem trước.

1. **Văn bản hiện một lần** (FR124)
   **Given** một lượt nhập bất kỳ (URL, tệp, dán tay) có luật bật khớp ít nhất một chỗ
   **When** màn xem trước mở
   **Then** cột cấu hình không in văn bản nào của Chương; tầng luật chỉ còn danh sách luật, hai số đếm, nhãn tầng, sửa, xoá, bật/tắt và thêm luật
   **And** mỗi đoạn văn bản của Chương hiện đúng một lần trên màn

2. **Gạch ngang trong khối** (FR124)
   **Given** lượt nhập URL, một luật bật khớp N chỗ trong các khối giữ của Chương đang chọn
   **When** màn mở, hoặc con trỏ Chương dời tới Chương đó
   **Then** đúng N chỗ gạch ngang bằng nét `ornament`, mỗi chỗ trong khối chứa nó, phần còn lại của khối không gạch
   **And** chỗ khớp nằm sau ~4 KiB đầu Chương cũng gạch ngang, vì khối hiện đủ văn bản

3. **Chỗ khớp vắt qua hai khối** (FR124)
   **Given** một luật regex khớp một chuỗi bắt đầu ở cuối một khối giữ và kết thúc ở đầu khối giữ kế tiếp
   **When** màn mở
   **Then** phần thuộc mỗi khối gạch ngang trong khối đó, và luật vẫn đếm là một chỗ khớp

4. **Khối loại không gạch, giữ lại thì gạch** (FR124)
   **Given** một khối đã loại chứa chuỗi khớp luật
   **When** màn mở
   **Then** khối đó không có gạch ngang, vì nó không vào văn bản nhập
   **And** khi người dùng giữ khối đó, chỗ khớp gạch ngang trong khối và số đếm của luật tăng tương ứng

5. **Tệp và dán tay** (FR124)
   **Given** lượt nhập `.txt`, `.md`, `.docx`, nhiều tệp hoặc dán tay, có luật khớp
   **When** màn mở
   **Then** cột văn bản hiện phần đầu Chương trong giới hạn 4 096 byte đầu vào, với chỗ khớp gạch ngang tại chỗ
   **And** đầu vào dài hơn 4 096 byte thì câu "Chỉ xem trước phần đầu Chương…" hiện ngay trên đoạn; ngắn hơn thì đoạn là toàn Chương và câu đó không hiện
   **And** lượt có chữ nhưng không luật nào khớp vẫn hiện đoạn đó, không gạch; lượt không có chữ hiện câu lý do như hôm nay

6. **Nhãn luật tại chỗ gạch** (FR124)
   **Given** một chỗ gạch ngang do một hoặc nhiều luật khớp
   **When** người dùng rê chuột hoặc đưa tiêu điểm tới chỗ đó
   **Then** chỗ đó nêu mẫu và tầng (Toàn cục, Tác phẩm) của mọi luật đã khớp, và trình đọc màn hình đọc được cùng nội dung

7. **Đổi luật thì chỗ gạch đổi ngay** (FR124)
   **Given** màn đang hiện chỗ gạch ngang của một luật
   **When** người dùng tắt luật đó, bật lại, sửa mẫu, xoá luật, hoặc thêm một luật mới có khớp
   **Then** chỗ gạch ngang trong cột văn bản đổi theo mà không phải đóng màn, và vị trí cuộn của cột văn bản không nhảy

## Boundaries

- Must not change: luật xoá gì và văn bản ghi xuống khi xác nhận. Cùng đầu vào, cùng luật, cùng thao tác sửa thì Tác phẩm tạo ra giống hệt trước story này.
- Must not change: hai số đếm của mỗi luật đo trên toàn Chương, không trên đoạn đang hiện.
- Must not change: luật không áp lên alt-text của ảnh; khối ảnh không có gạch ngang.
- Must not change: dải khối loại, `J`/`K`/`Space`, `[` `]`, `⌥←` `⌥→`, `⌥W`, `E`, chân dính và cột cấu hình của story 23.
- Touch point: nút "Tới luật làm sạch" ở đầu cột văn bản vẫn đưa tiêu điểm tới tầng luật trong cột cấu hình.
- Touch point: màn xem trước song ngữ (FR115) giữ nguyên; nó không nhận phần gạch ngang tại chỗ.

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-duong-nhap/epic-duong-nhap.md, Requirements
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, §Luật làm sạch hiện thứ sắp bị xoá, trước khi xoá
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, §Bố cục: cột cấu hình, cột văn bản, chân dính (đoạn "Tầng sạch thu thành một dòng")
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/web-import.html, khối có `.strike`
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-1, AD-16, AD-18, AD-39
- sibling — _bmad-output/initiative-auratranslate/tickets/epic-duong-nhap/tickets/story-man-xem-truoc-nhap-nut-nhap-luon-trong-tam-tay-plan.md, §Boundaries & Constraints (quyết định tầng luật sang cột cấu hình)

## Notes

- Decision: Rust gửi chỗ khớp theo từng khối, đo trong văn bản của chính khối đó, và tự cắt chỗ khớp vắt qua hai khối (cách A). Đã bỏ cách B: Rust gửi vị trí đầu khối cộng chỗ khớp toàn Chương, webview tự cắt. Webview không tự tính vị trí được, vì phép ghép khối dùng khoảng cách gốc giữa các khối, chỉ có ở Rust (AD-1) (Ice, 2026-11-02).
- Decision: nhãn luật hiện khi rê chuột hoặc đưa tiêu điểm tới chỗ gạch, không có chip luôn hiện; mockup `web-import.html` không vẽ nhãn thấy sẵn (Ice, 2026-11-02).
- Decision: đường tệp và dán tay giữ đúng đoạn văn bản và giới hạn ~4 KiB hôm nay, chỉ đổi chỗ hiện; không hiện toàn Chương, nên đường này không đổi Rust (Ice, 2026-11-02).
- Kiểm ngoài tiêu chí (risk medium, gạch sai chỗ thì người dùng duyệt nhầm thứ sắp xoá): đối chứng đỏ là phép gỡ thật phần ánh xạ ở Rust, ca e2e criterion 2 đỏ vì 0 chỗ gạch.
- Bàn đo thật: tệp vitest `importPreviewCleanup.test.ts` và hai spec e2e của màn xem trước đọc chỗ in văn bản ở tầng luật; build đếm và sửa chúng ở pha Tests.
