---
id: 23
type: bug
title: "Library: không Tác phẩm nào có tiến độ"
parent: epic-hoan-thien-giao-dien-theo-mockup
covers: []
after: []
assignee: ""
refined: true
hitl: true
risk: medium
severity: P3
---

# Library: không Tác phẩm nào có tiến độ

## Description

Ở Library của Ice, cả 47 thẻ ghi "Chưa biết tiến độ" và không thẻ nào có huy hiệu trạng thái. Một Tác phẩm có Chương đã xong phải hiện `k / n` Chương. Bug này chốt nguyên nhân và cách xử lý trước khi story 24 giấu dòng chữ đó (L3 của phiếu). Ice là người chọn giữa hai phương án ở tiêu chí 2; đó là bước cần người.

## Reproduction

1. Trên máy Ice, mở app, bấm `⌘1` vào Library (47 Tác phẩm trong `~/Documents/AuraTranslate/`, phần lớn tên `test`, `tieng trung …`).
2. Thực tế: mọi thẻ ghi "Chưa biết tiến độ", không có huy hiệu trạng thái (ảnh Ice chụp 2026-10-25).
3. Mong đợi: Tác phẩm có Chương đã xong hiện số Chương đã xong; Tác phẩm chưa bắt đầu hiện là chưa bắt đầu.

Đọc chỉ mục thật (`library-index.db`, chỉ đọc, 2026-10-25): 47 hàng `library_work`, cả 47 có `chapter_done_count IS NULL` và `status IS NULL`; `updated_at` từ 2026-08-06 tới 2026-08-26. Đọc 47 `meta.json` trong `~/Documents/AuraTranslate/*.atproj/`: cả 47 có `meta_schema_version: 1`.

## Cause Hypothesis

Cả 47 `meta.json` có `meta_schema_version: 1`, viết trước khi trường tiến độ và trạng thái tồn tại (Story 5.5, 2026-08-28). Chỉ mục chép nguyên giá trị từ `meta.json`, nên `NULL` ở chỉ mục là câu trả lời "chưa biết" đúng với dữ liệu. Chỉ có hai đường dựng lại `meta.json` từ `project.db`: lúc tạo Tác phẩm và sau một thao tác đổi trạng thái hay tổ chức Chương. Mở Tác phẩm hay quét lại Library không dựng lại nó, nên một Tác phẩm cũ chưa bị đổi trạng thái Chương lần nào sẽ "chưa biết" mãi.

## Acceptance Criteria

1. **Nguyên nhân có bằng chứng**
   **Given** Library của Ice ở trạng thái của bước tái hiện
   **When** người dựng đọc lại `meta.json` và chỉ mục
   **Then** plan ghi một dòng `Decision:` nêu nguyên nhân, kèm số Tác phẩm ở mỗi `meta_schema_version` và đường nào (nếu có) đã dựng lại `meta.json` của chúng
2. **Ice chọn cách xử lý, có số đo**
   **Given** hai phương án: (A) không đổi mã, `NULL` vẫn là "chưa biết", story 24 chỉ ẩn chữ; (B) dựng lại `meta.json` từ `project.db` khi `meta_schema_version` thấp hơn bản hiện tại
   **When** người dựng đo chi phí của B trên 47 Tác phẩm của Ice (thời gian quét lại, số `project.db` phải mở) và chỉ ra B chạy ở lượt nào
   **Then** cả hai phương án cùng số đo được trình cho Ice, và lựa chọn của Ice được ghi thành `Decision:` có ngày trước khi viết mã
3. **Tác phẩm mới luôn có tiến độ**
   **Given** một Tác phẩm vừa tạo, có một Chương được đánh dấu xong
   **When** Library tải danh sách
   **Then** thẻ của nó nhận `chapter_done_count` khác `NULL` qua đúng đường meta → chỉ mục → danh sách của Library, và một test che đường đó (thêm nếu chưa có; đếm ca thật sự đi tới đó)
4. **Nếu Ice chọn B: Tác phẩm cũ hết "chưa biết"**
   **Given** một Tác phẩm có `meta.json` bản 1 và một Chương đã xong trong `project.db`
   **When** lượt mà B gắn vào chạy
   **Then** thẻ hiện đúng số Chương đã xong, có một test đỏ khi bỏ bước dựng lại và xanh khi có nó, và `meta.json` hỏng hay không ghi được vẫn để lại vết chẩn đoán, không thành `0`
5. **Hoặc: không cần đổi mã, có bằng chứng**
   **Given** Ice chọn A
   **When** tiêu chí 1 và 3 đã đạt
   **Then** bug đóng mà không đổi mã sản phẩm, bằng chứng ghi ở Notes của plan; tiêu chí này thay tiêu chí 4

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-hoan-thien-giao-dien-theo-mockup/epic-hoan-thien-giao-dien-theo-mockup.md
- brief — _bmad-output/initiative-auratranslate/ux-auratranslate/library-nghiem-thu-thi-giac.md, §Câu hỏi cho người dựng, L3
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-33

## Notes

- Dữ liệu đo nằm ngoài repo, trên máy Ice: `~/Library/Application Support/com.auratranslate.desktop/library-index.db` và `~/Documents/AuraTranslate/*.atproj/meta.json`.
- Không sửa hay xoá dữ liệu thật của Ice khi đo; đo B trên bản chép của `~/Documents/AuraTranslate/`.
- B nếu được chọn vẫn ghi `meta.json` qua đúng một đường ghi của AD-33, không đúc đường thứ hai.
- Open question: nếu chọn B, B chạy lúc mở Tác phẩm, lúc "Dựng lại toàn bộ chỉ mục" (FR140), hay lúc quét lại; người dựng đề xuất kèm số đo ở tiêu chí 2.
