---
id: 2
type: story
title: "Cổng sổ nợ và bộ chọn test đọc cây vé v7"
parent: none
covers: []
after: []
assignee: ""
refined: true
hitl: false
risk: medium
---

# Cổng sổ nợ và bộ chọn test đọc cây vé v7

## Description

Sau đợt di trú BMad v7, `check:debt-owner` dừng với lỗi hạ tầng vì sổ nợ đã chuyển sang thư mục initiative và `sprint-status.yaml` chỉ còn bản lưu trữ. `test:story` cũng không tìm thấy story nào vì spec v6 đã thành plan v7. Khi story này xong, cả hai đọc cây vé v7: Kiểm C lấy trạng thái story và epic từ cây vé, `test:story` tìm plan theo ref của ticket. Story này cũng chuyển chủ cho các mục nợ còn mở có chủ cuối là Story 4.x, để Kiểm C xanh trên sổ thật. Story phải xong trước lần push kế, vì `pre-push` chạy cổng này.

## Acceptance Criteria

1. **Cổng chạy trên sổ nợ ở vị trí mới, đếm như trước di trú**
   **Given** sổ nợ ở gốc thư mục initiative, chưa nối thêm dòng nào của story này
   **When** chạy `npm run check:debt-owner`
   **Then** Kiểm A, B và C đều chạy, và mã thoát là phán quyết của cổng
   **And** dòng tổng của Kiểm A in `0/260 mục mở thiếu Chủ — 795 mục tổng, 63 nửa, 339 đóng`, đúng như CI run 37446210896 (lần chạy xanh cuối trước di trú, cùng nội dung sổ)

2. **Kiểm C lấy trạng thái từ cây vé**
   **Given** một mục mở hoặc 🟡 có `Chủ:` cụ thể cuối cùng là `Story X.Y` hoặc `Epic N`
   **When** cổng chạy
   **Then** `Story X.Y` được tra như ticket có ref `X.Y`, và `Epic N` như epic có `id = N` trong `tickets.toml` của initiative
   **And** mục bị báo khi ticket hoặc epic đó là `done`, hoặc không có trong cây vé
   **And** mục không bị báo khi ticket hoặc epic đó ở bất kỳ trạng thái nào khác, kể cả `planned`, `built`, `in-review` và trạng thái trống
   **And** cổng không đọc `archive-v6/sprint-status.yaml`

3. **Ref có chữ cái được nhận**
   **Given** một mục có chủ cuối là một ticket có id chữ, như `Story 6.16c`
   **When** cổng chạy
   **Then** chủ đó được tra đúng ticket `6.16c`, không bị coi là không tồn tại

4. **Cây vé không đọc được thì đỏ, không xanh**
   **Given** cây vé không đọc được, hoặc đọc ra 0 ticket
   **When** cổng chạy
   **Then** cổng thoát khác 0 với thông báo lỗi hạ tầng, không bao giờ in "đạt"

5. **Tự kiểm phủ nguồn trạng thái mới**
   **Given** một trạng thái cây vé giả, cố định
   **When** Kiểm B chạy
   **Then** có ít nhất một ca cho mỗi nhánh: chủ `done`, chủ không tồn tại, chủ chưa `done`, epic `done`, chủ id chữ, cây vé rỗng
   **And** khi phép tra trạng thái trong mã sản phẩm bị gỡ thật rồi chạy lại `npm run check:debt-owner`, Kiểm B đỏ đúng vì lý do đó

6. **Nợ của Epic 4 có chủ mới**
   **Given** HEAD có 12 story 4.1–4.12 đã `done`
   **When** cổng chạy trên sổ thật
   **Then** Kiểm C xanh, và không mục nào bị báo vì chủ cuối là một Story 4.x
   **And** mỗi mục đó mang thêm một dòng `→ … Chủ: <chủ mới>` kèm lý do một câu, hoặc được đóng bằng lời; không mục nào bị xoá hay sửa nội dung cũ

7. **`test:story` tìm plan theo ref**
   **Given** ref của một ticket có plan, như `4.8`, `6.16c` hoặc dạng `4-8`
   **When** chạy `npm run test:story <ref> -- --list`
   **Then** lệnh tìm plan qua cây vé, và `4.8` với `4-8` ra cùng một plan
   **And** plan có khai test thì lệnh liệt kê đúng các test đó
   **And** plan không khai test nào thì lệnh in cảnh báo nói rõ như vậy và vẫn chạy test lấy từ diff; lệnh chỉ thoát khác 0 khi không còn test nào để chạy
   **And** ref không có trong cây vé thì lệnh thoát khác 0 và nói đã tìm ở đâu

8. **Luật trong AGENTS.md mô tả đúng cổng**
   **Given** các câu trong AGENTS.md gốc nói Kiểm C đọc `sprint-status.yaml`
   **When** story xong
   **Then** các câu đó nói Kiểm C đọc trạng thái từ cây vé
   **And** test `naming_boundary` vẫn xanh

## Boundaries

- Must not change: luật hẹp của Kiểm A (chỉ `Chủ:`, đọc nội dung sau `Chủ:`), cách nhận diện một mục và trạng thái của mục.
- Must not change: `--file` không bao giờ làm Kiểm A đọc tệp khác sổ thật.
- Must not change: danh sách cổng ở `package.json`, `ci.yml` và `.githooks/pre-push`. Không thêm Kiểm mới.
- Must not change: nội dung đã có trong `deferred-work.md`; chỉ nối thêm dòng.
- Must not change: cách `tickets.py` và quy trình BMAD ghi trạng thái.

## References

- parent — none
- decision — _bmad-output/initiative-auratranslate/migration-v6-v7/migration-v6-v7.md, §3 Q17
- surface — _bmad-output/initiative-auratranslate/migration-v6-v7/migration-v6-v7.md, §15 (bảng tham chiếu ngoài store)
- rule — AGENTS.md, §Specs, handoffs, ledger
- rule — scripts/AGENTS.md

## Notes

- Decision: Q17 (Ice chốt 2026-10-06) — một story riêng viết lại cả hai script; Kiểm C đọc trạng thái từ plan v7 thay vì `sprint-status.yaml`.
- Decision: Ice giao story này chuyển chủ các mục nợ mở có chủ cuối là Story 4.x (2026-10-06, sau commit `72ec621`). Đếm bằng script lúc ký: khoảng 20 mục (4.8: 12, 4.12: 5, 4.2, 4.9, 4.10: mỗi story 1).
- Đo lúc soạn (2026-10-06): 153 khoá story của `archive-v6/sprint-status.yaml` đều có ref trong cây v7; chỗ lệch `done` duy nhất là 12 story 4.1–4.12 vừa ký. Không epic nào `done` ở v6; ở v7 epic 8–10 có trạng thái trống, các epic còn lại `in-progress`.
- Assumption: trạng thái của `Epic N` là `status:` trong tệp epic của nó.
- Decision: (Ice đồng ý 2026-10-06) chủ mới mặc định cho một mục Story 4.x là `Epic 4` (epic còn mở). Mục nào rõ là thuộc một story đã lên kế hoạch ở epic khác thì chuyển sang story đó.
- Assumption: các con trỏ chết khác trong AGENTS.md gốc (§15) nằm ngoài story này; story chỉ sửa câu mô tả cổng.
- Decision: (Ice, 2026-10-06, lúc lập plan) AC7 nới: plan không khai test chỉ là cảnh báo, vì 126/141 plan không có dòng "Tests that move".
- Risk medium: phần mềm không bị ảnh hưởng, nhưng cổng này chặn mọi lần push, và lỗi kiểu "xanh mà không quét gì" thì không ai thấy.
