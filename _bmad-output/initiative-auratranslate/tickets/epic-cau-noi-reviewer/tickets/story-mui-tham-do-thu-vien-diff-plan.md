---
title: 'Mũi thăm dò thư viện diff'
type: 'chore'
ticket: '1'
created: '2026-10-08'
status: 'built'
baseline_revision: '3b686ec5446f4c423ecc0ce131adf47a030db683'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** AD-51 chốt `similar` =3.1.1 (theo từ + gộp `equal` ≤ 2) bằng số đo nguồn Trung/Anh; mục 9 để lại đúng một phép xác nhận: chạy `similar` và `dissimilar` trên bản review thật tiếng Việt của Ice, chuỗi dày dấu. Bộ đo của AD-51 đã mất cùng scratchpad cũ, và repo không có tệp review thật nào.

**Approach:** Dựng lại bộ đo ngoài repo, chạy bốn biến thể (`similar` từ + gộp = thân `diff_spans` hiện tại · `similar` ký tự · `similar` grapheme · `dissimilar`) trên các cặp *(bản dịch của Ice, bản reviewer sửa)* lấy từ bản review thật, sau `trim` + NFC như AD-51 mục 4. Ghi số đo và kết luận *xác nhận* hoặc *đề nghị lật* vào bằng chứng của AD-51; lật thì dừng lại cho Ice và Winston (AD mới), không sửa mã.

**Decisions (2026-10-08, Ice):**
- Bản review duy nhất là `docs/Bản sao của Chuộc tội, trong những ngày tuyết bay (Phần 1).docx`: bảng một hàng, cột trái tiếng Trung, cột phải **một** bản tiếng Việt (40 đoạn, 9 575 ký tự), không track changes. Không có bản trước/sau review nào khác.
- "Có bao nhiêu làm bấy nhiêu": đo trên tập `vi_edit` — đoạn tiếng Việt thật của tệp đó + vết sửa tạo bằng mã, hạt giống cố định, kiểu reviewer (chỉ đổi dấu thanh · thay từ · chèn từ · xoá từ · đảo cụm), khuôn `zh_edit`/`en_edit` của AD-51. Phần "bản review thật" không đo được; ghi rõ điều đó trong bằng chứng, **không thêm mục nợ**.
- Bộ đo ném đi trong scratchpad; tệp `.docx` không commit.

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/matching/mod.rs:486` -- `diff_spans`; chuẩn hoá `trim().nfc()` (`:494`), `Zh` ⇒ `from_chars`, `En` ⇒ `from_words` (`:497`), gộp `ABSORB_MAX_CHARS = 2` (`:492`, `:518`). Bộ đo chép nguyên thân hàm này, ghi commit gốc. `MatchLang` chỉ có `Zh`/`En`: chữ Việt đi đường `En`. Không sửa.
- `src-tauri/tests/matching_contract.rs:659` -- ca bất biến dựng lại (đã có ca NFD tiếng Việt `:667`). Không sửa.
- `src-tauri/src/core/docx/mod.rs:141` -- `read_docx` chỉ trả đoạn phẳng + số đếm bảng; bộ đo tự đọc cột từ `.docx`, không qua mã app.
- `src/tokens/tokens.json:35-38`, `:61-64`, `:96-102`, `:294-300` + `scripts/check-tokens.mjs` -- họ `diff-text`/`diff-fill` hai theme và cặp tương phản; AC WCAG AA đã do `check:tokens` canh.
- `_bmad-output/initiative-auratranslate/ad-51-draft/ad-51-draft.md` §Phần 3 -- chỉ số và khuôn bảng của AD-51; số đo mới nối thành `## Phần 7 — Xác nhận tiếng Việt (Story 8.1)`.
- `_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md:1167` (hàng Deferred) và AD-51 mục 9 -- sửa tại chỗ bằng 🔵 khi xác nhận.

## Tasks & Acceptance

**Execution:**
- [ ] bộ đo trong scratchpad -- dựng tập `vi_edit`, chạy bốn biến thể, tính các chỉ số của AD-51 Phần 3 cộng `tách dấu` (đoạn đổi chỉ chứa dấu kết hợp) và `cắt giữa âm tiết` (biên đoạn đổi rơi trong một dãy chữ liền), kiểm bất biến dựng lại -- dữ liệu thật thay vì suy luận từ tiếng Anh.
- [ ] `ad-51-draft/ad-51-draft.md` -- thêm Phần 7: build, máy và tải, nguồn dữ liệu (không chép văn bản review ngoài vài mẫu ngắn), bảng số đo, kết luận xác nhận/đề nghị lật.
- [ ] `architecture-auratranslate.md` -- xác nhận ⇒ 🔵 hàng Deferred `:1167` và AD-51 mục 9 trỏ Phần 7; lật ⇒ không sửa spine, HALT cho Ice.
- [ ] `npm run check:tokens` -- chạy một lần, ghi kết quả cho AC WCAG.

**Acceptance Criteria:**
- Given tập `vi_edit`, when chạy bộ đo, then cả `similar` và `dissimilar` đều có số trên cùng tập cặp, và bằng chứng nêu mật độ dấu thanh của tập cùng việc không có cặp review thật.
- Given hai kết quả, when so, then bảng nêu đánh đổi grapheme vs semantic cleanup bằng chỉ số, và 0 lỗi dựng lại cho mọi biến thể (hoặc nêu cặp lỗi).
- Given AD-51 mục 10, when kiểm giấy phép, then không thêm crate nào vào `src-tauri/Cargo.toml`; Stack giữ đúng một hàng `similar`.
- Given họ `diff-text`/`diff-fill`, when chạy `check:tokens`, then đạt AA ở cả hai theme.

## Implementation Notes

Route oneshot: thay đổi trong repo chỉ là tài liệu, mã đo nằm ngoài `src-tauri`.
- Kết quả: giữ `similar` theo từ + gộp ≤ 2. Trên 86 cặp `vi_edit` nó cho 0 cắt giữa từ và 0 vùng thừa. `dissimilar` cho 98 cắt giữa từ và 55 đổi 1 ký tự. Grapheme trùng ký tự sau NFC. Số đo và giới hạn ở `ad-51-draft.md` Phần 7.
- Đánh đổi thật duy nhất là sửa chỉ đổi dấu: cấp ký tự tô 2,1 ký tự, theo từ tô cả âm tiết 6,8 ký tự.
- `MatchLang` không có `Vi`. Review Mode phải truyền đường theo từ (`En`).
- Bộ đo ở scratchpad phiên này (`dp/`, `kit/mkvi.py`), không giữ. Tệp `.docx` trong `docs/` không commit.

## Verification

**Commands:**
- `npm run check:tokens` -- expected: xanh.
- `git diff --stat` -- expected: không chạm `src-tauri/`, `src/`.

## Review Triage Log

- medium · patch: 98 lần cắt giữa từ của `dissimilar` bị tính như một cái giá riêng. Đo theo kiểu sửa thì cả 98 lần, cùng 55 đoạn 1 ký tự, đến từ 29 cặp đổi dấu. Đã viết lại mục "Đọc số đo" kèm số theo kiểu sửa.
- medium · patch: câu quyết "một ô tô một ký tự dễ lọt mắt" không có số đo nào đỡ. Đã bỏ câu đó. Kết luận giờ dựa vào số đo: ở cấp ký tự hai crate bằng nhau, và độ mịn đã do AD-51 mục 7 chốt. Đổi độ mịn là việc của Ice.
- low · patch: cột `dấu rời` không thể đỏ sau NFC, và grapheme bằng ký tự là do cấu tạo. Đã bỏ cột và ghi rõ điều này.
- low · patch: câu trong ngoặc về `unicode_words` trái với bảng. Đã sửa câu.
- medium · patch: AC WCAG mới chỉ có tương phản chữ làm bằng chứng. Đã ghi thêm tương phản phi chữ, từ 1,12 đến 1,29:1 (dưới 3:1), và việc `.tmf-ins` chỉ đánh dấu chỗ thêm bằng màu. Phần hiển thị thuộc AC của Story 8.12.
- low · patch: cột thứ ba của hàng Deferred đã cũ. Hàng nay gạch và ghi ✅ ĐÃ ĐÓNG theo khuôn các hàng khác.
- low · patch: dòng µs ghi "= `diff_spans`" nhưng không gồm NFC. Đã ghi rõ.
- low · rejected: `docs/*.docx` không được ignore. Tệp do Ice đặt vào, không thuộc diff này. Commit chỉ nêu đường dẫn cụ thể.

