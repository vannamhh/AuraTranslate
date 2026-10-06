---
type: epic
title: "Phát hành & tin cậy — vượt rào cản không ký số"
parent: initiative-auratranslate
covers: [FR105, FR106, FR107, FR108, FR109, FR110, FR111, FR112]
after: []
assignee: ""
risk: low
---

# Phát hành & tin cậy — vượt rào cản không ký số

## Description

Một người dịch phổ thông tải bản cài từ GitHub Releases, đối chiếu checksum SHA-256, đi theo **hướng dẫn cài đặt có ảnh chụp màn hình** xử lý tường minh Gatekeeper trên macOS và SmartScreen trên Windows, và cài được — dù bản phát hành **không ký, không notarize**. Trong ứng dụng, màn hình Attribution liệt kê mọi nguồn từ điển kèm giấy phép, **dựng từ các file dữ liệu có mặt**. Cơ chế cập nhật **chỉ kiểm tra và thông báo**.

## Outcome

Một người dịch phổ thông tải bản cài từ GitHub Releases, kiểm checksum, làm theo hướng dẫn có ảnh để vượt Gatekeeper và SmartScreen, và cài được dù bản phát hành không ký số.

## Requirements

- FR105: Phát hành bản cài cho **macOS và Windows** qua **GitHub Releases**.
- FR106: Công bố **checksum SHA-256** cho mọi artifact phát hành.
- FR107: **Build công khai qua GitHub Actions**, để bất kỳ ai cũng kiểm chứng được binary khớp với mã nguồn.
- FR108: **Hướng dẫn cài đặt có ảnh chụp màn hình** cho cả hai hệ điều hành, xử lý tường minh Gatekeeper trên macOS và SmartScreen trên Windows.
- FR109: **Màn hình Attribution trong ứng dụng:** liệt kê mọi nguồn từ điển, giấy phép tương ứng và ghi công đầy đủ.
- FR110: Kèm văn bản giấy phép **GPL v3** và toàn bộ giấy phép của các bộ dữ liệu trong bản phát hành.
- FR111: Cơ chế cập nhật **chỉ kiểm tra và thông báo** phiên bản mới. **Không tự động tải, không tự động cài.**
- FR112: **Chính sách gỡ bỏ dữ liệu:** quy trình gỡ một lớp nguồn khỏi bản phát hành kế tiếp **mà không ảnh hưởng chức năng** — bảo đảm bởi FR36.

## Done when

1. Bản cài macOS và Windows dựng công khai qua GitHub Actions và phát hành trên GitHub Releases kèm checksum SHA-256.
2. Hướng dẫn cài đặt có ảnh chụp màn hình xử lý tường minh Gatekeeper và SmartScreen.
3. Màn hình Attribution dựng từ các file dữ liệu có mặt: gỡ một nguồn bằng xoá một file thì ghi công biến mất, không đổi mã.
4. Cập nhật chỉ kiểm tra và thông báo, không tự tải, không tự cài.
5. Các ngưỡng NFR1, NFR3, NFR4, NFR5, NFR19 được đo lại ở nghiệm thu cuối (Story 10.9) và Q4 đóng.

## Boundaries

Phát hành, giấy phép, Attribution và nghiệm thu cuối. Không thêm năng lực dịch.

## References

- parent — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, functional requirements FR105, FR106, FR107, FR108, FR109, FR110, FR111, FR112
- spec — _bmad-output/initiative-auratranslate/spec-auratranslate/spec-auratranslate.md
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, các AD nêu trong tiêu chí của story

## Notes

- Decision: Story 6.18 (đo lại NFR3, NFR4, NFR5 trên thư viện 5.000 Chương) gộp vào Story 10.9 ngày 2026-09-15 qua `correct-course`; Story 10.9 giữ AC đó.
- Source conflict: epics.md nêu một nghĩa vụ ngoài mã nguồn không mang số FR — thông báo cho tác giả Đặng Thế Kiệt khi công cụ hoàn thành (điều kiện của phép sử dụng) — nhưng không story nào của epic trích nó.
